pub mod decoderbufs;
pub(crate) mod serde;
pub mod stream;

//
// #[tokio::test]
// async fn run() {
//     use futures::StreamExt;
//     let db_config = format!(
//         "user=postgres password=postgres host=localhost port=5432 dbname=postgres replication=database",
//     );
//     println!("CONNECT");
//
//     // connect to the database
//     let (client, connection) = tokio_postgres::connect(&db_config, tokio_postgres::NoTls)
//         .await
//         .unwrap();
//
//     // the connection object performs the actual communication with the database, so spawn it off to run on its own
//     tokio::spawn(async move { connection.await });
//
//     let stream =
//         ReplicationSlot::open_at("stream_nft".to_string(), PgLsn::from_str("0/0").unwrap(), &client)
//             .await
//             .unwrap();
//     let mut stream = Box::pin(stream);
//     // let (a, mut b) = spawn("postgres".to_string(), "sloot".to_string());
//     while let Some(v) = stream.as_mut().next().await {
//         match v {
//             DecoderBufMessage::Transaction(t) => {
//                 // stream
//                 //     .as_mut()
//                 //     .standby_status_update(
//                 //         PgLsn::from(t.start),
//                 //         PgLsn::from(t.start),
//                 //         PgLsn::from(t.start),
//                 //         pgtime(SystemTime::now()),
//                 //         0,
//                 //     )
//                 //     .await.unwrap();
//                 panic!("{t:#?}")
//             }
//             DecoderBufMessage::PrimaryKeepAlive(a) => {
//                 // stream.standby_status_update(PgLsn(25501))
//                 // stream.standby_status_update()
//             }
//         }
//     }
// }
