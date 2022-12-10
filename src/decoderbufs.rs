#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Point {
    #[prost(double, required, tag = "1")]
    pub x: f64,
    #[prost(double, required, tag = "2")]
    pub y: f64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DatumMessage {
    #[prost(string, optional, tag = "1")]
    pub column_name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int64, optional, tag = "2")]
    pub column_type: ::core::option::Option<i64>,
    #[prost(oneof = "datum_message::Datum", tags = "3, 4, 5, 6, 7, 8, 9, 10, 11")]
    pub datum: ::core::option::Option<datum_message::Datum>,
}
/// Nested message and enum types in `DatumMessage`.
pub mod datum_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Datum {
        #[prost(int32, tag = "3")]
        DatumInt32(i32),
        #[prost(int64, tag = "4")]
        DatumInt64(i64),
        #[prost(float, tag = "5")]
        DatumFloat(f32),
        #[prost(double, tag = "6")]
        DatumDouble(f64),
        #[prost(bool, tag = "7")]
        DatumBool(bool),
        #[prost(string, tag = "8")]
        DatumString(::prost::alloc::string::String),
        #[prost(bytes, tag = "9")]
        DatumBytes(::prost::alloc::vec::Vec<u8>),
        #[prost(message, tag = "10")]
        DatumPoint(super::Point),
        #[prost(bool, tag = "11")]
        DatumMissing(bool),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TypeInfo {
    #[prost(string, required, tag = "1")]
    pub modifier: ::prost::alloc::string::String,
    #[prost(bool, required, tag = "2")]
    pub value_optional: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RowMessage {
    #[prost(uint32, optional, tag = "1")]
    pub transaction_id: ::core::option::Option<u32>,
    #[prost(uint64, optional, tag = "2")]
    pub commit_time: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub table: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "Op", optional, tag = "4")]
    pub op: ::core::option::Option<i32>,
    #[prost(message, repeated, tag = "5")]
    pub new_tuple: ::prost::alloc::vec::Vec<DatumMessage>,
    #[prost(message, repeated, tag = "6")]
    pub old_tuple: ::prost::alloc::vec::Vec<DatumMessage>,
    #[prost(message, repeated, tag = "7")]
    pub new_typeinfo: ::prost::alloc::vec::Vec<TypeInfo>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Op {
    Unknown = -1,
    Insert = 0,
    Update = 1,
    Delete = 2,
    Begin = 3,
    Commit = 4,
}
impl Op {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Op::Unknown => "UNKNOWN",
            Op::Insert => "INSERT",
            Op::Update => "UPDATE",
            Op::Delete => "DELETE",
            Op::Begin => "BEGIN",
            Op::Commit => "COMMIT",
        }
    }
}
