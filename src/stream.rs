pub use tokio_postgres;

use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::time::{SystemTime, UNIX_EPOCH};

use bytes::Bytes;
use futures::{Stream, StreamExt};
use postgres_protocol::message::backend::PrimaryKeepAliveBody;
use prost::Message;
use tokio_postgres::replication::ReplicationStream;
use tokio_postgres::types::PgLsn;
use tokio_postgres::{CopyBothDuplex, Error, SimpleQueryMessage, SimpleQueryRow};

use crate::decoderbufs::{DatumMessage, RowMessage, TypeInfo};

static MICROSECONDS_FROM_UNIX_EPOCH_TO_2000: u128 = 946_684_800_000_000;

pub fn pgtime(time: SystemTime) -> i64 {
    (time.duration_since(UNIX_EPOCH).unwrap().as_micros() - MICROSECONDS_FROM_UNIX_EPOCH_TO_2000)
        as i64
}

pin_project_lite::pin_project! {
    pub struct DecoderBufStream {
        #[pin]
        stream: ReplicationStream,
        transaction: Option<WalTransaction>
    }
}

type UntypedColumns = Vec<DatumMessage>;

/// Generic
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Change<T = UntypedColumns> {
    Insert(T),
    Update(T, T),
    Delete(T),
}

/// Represents a change of one row in the WAL.
///
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WalRowEntry {
    change: Change<UntypedColumns>,

    table: String,
    typeinfo: Vec<TypeInfo>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WalTransaction {
    pub start: PgLsn,
    pub txid: u32,
    pub commit_time: u64,
    pub events: Vec<RowMessage>,
}

#[derive(Debug)]
pub enum DecoderBufMessage {
    Transaction(WalTransaction),
    PrimaryKeepAlive(PrimaryKeepAliveBody),
}

impl DecoderBufStream {
    /// Creates a new DecoderbufStream that will wrap the underlying CopyBoth stream
    pub fn new(stream: CopyBothDuplex<Bytes>) -> Self {
        Self {
            stream: ReplicationStream::new(stream),
            transaction: None,
        }
    }
    /// Send standby update to server.
    pub async fn standby_status_update(
        self: Pin<&mut Self>,
        write_lsn: PgLsn,
        flush_lsn: PgLsn,
        apply_lsn: PgLsn,
        ts: i64,
        reply: u8,
    ) -> Result<(), Error> {
        self.project()
            .stream
            .standby_status_update(write_lsn, flush_lsn, apply_lsn, ts, reply)
            .await
    }

    /// Send hot standby feedback message to server.
    #[inline(always)]
    pub async fn hot_standby_feedback(
        self: Pin<&mut Self>,
        timestamp: i64,
        global_xmin: u32,
        global_xmin_epoch: u32,
        catalog_xmin: u32,
        catalog_xmin_epoch: u32,
    ) -> Result<(), Error> {
        self.project()
            .stream
            .hot_standby_feedback(
                timestamp,
                global_xmin,
                global_xmin_epoch,
                catalog_xmin,
                catalog_xmin_epoch,
            )
            .await
    }
}

impl Stream for DecoderBufStream {
    type Item = DecoderBufMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use postgres_protocol::message::backend::ReplicationMessage;

        let mut this = self.project();
        loop {
            match ready!(this.stream.as_mut().poll_next(cx)) {
                Some(Ok(ReplicationMessage::XLogData(body))) => {
                    let wal_start = body.wal_start();
                    let row_message =
                        crate::decoderbufs::RowMessage::decode(body.into_data()).unwrap();

                    match row_message.op {
                        Some(op) if op == crate::decoderbufs::Op::Begin as i32 => {
                            *this.transaction = Some(WalTransaction {
                                start: PgLsn::from(wal_start),
                                txid: row_message.transaction_id(),
                                commit_time: row_message.commit_time(),
                                events: vec![],
                            })
                        }
                        Some(op) if op == crate::decoderbufs::Op::Commit as i32 => {
                            return Poll::Ready(Some(DecoderBufMessage::Transaction(
                                this.transaction.take().unwrap(),
                            )));
                        }
                        Some(_) => {
                            this.transaction.as_mut().unwrap().events.push(row_message);
                        }
                        None => unimplemented!(),
                    }
                }
                Some(Ok(ReplicationMessage::PrimaryKeepAlive(body))) => {
                    return Poll::Ready(Some(DecoderBufMessage::PrimaryKeepAlive(body)));
                }
                Some(Ok(other)) => panic!("Unexpected: {:?}", other),
                Some(Err(_e)) => return Poll::Ready(None),
                None => return Poll::Ready(None),
            }
        }
    }
}

pub struct ReplicationSlot;

pub struct ReplicationSlotCreate {
    slot_name: String,
    consistent_point: String,
    snapshot_name: String,
    output_plugin: String,
}

impl ReplicationSlot {
    pub async fn try_create(
        name: String,
        client: &tokio_postgres::Client,
    ) -> Result<ReplicationSlotCreate, tokio_postgres::Error> {
        let slot_query = format!("CREATE_REPLICATION_SLOT {} LOGICAL \"decoderbufs\"", name);

        let data: SimpleQueryRow = client
            .simple_query(&slot_query)
            .await?
            .into_iter()
            .filter_map(|msg| match msg {
                SimpleQueryMessage::Row(row) => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>()
            .remove(0);

        let res = ReplicationSlotCreate {
            slot_name: data.get("slot_name").unwrap().to_string(),
            consistent_point: data.get("consistent_point").unwrap().to_string(),
            snapshot_name: data.get("snapshot_name").unwrap().to_string(),
            output_plugin: data.get("output_plugin").unwrap().to_string(),
        };

        Ok(res)
    }

    pub async fn delete(
        name: &str,
        client: &tokio_postgres::Client,
    ) -> Result<(), tokio_postgres::Error> {
        let slot_query = format!("DROP_REPLICATION_SLOT {} WAIT", name);

        client.simple_query(&slot_query).await?;
        Ok(())
    }

    pub async fn open_at(
        name: String,
        lsn: PgLsn,
        client: &tokio_postgres::Client,
    ) -> Result<DecoderBufStream, tokio_postgres::Error> {
        let query = format!("START_REPLICATION SLOT {} LOGICAL {}", name, lsn);
        // see here for format details: https://www.postgresql.org/docs/current/protocol-replication.html
        let duplex_stream = client
            .copy_both_simple::<bytes::Bytes>(&query)
            .await
            .unwrap();

        return Ok(DecoderBufStream::new(duplex_stream));
    }
}
