use prost::{DecodeError, EncodeError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Hook0ProtobufError {
    // Decoding from ProtoBuf error
    #[error("Could not decode from ProtoBuf: {0}")]
    ProtoDecode(#[from] DecodeError),

    /// Encoding to ProtoBuf error
    #[error("Could not encode to ProtoBuf: {0}")]
    ProtoEncode(#[from] EncodeError),

    /// Invalid UUID error
    #[error("Could not parse '{str}' as a UUID: {error}")]
    InvalidUuid {
        /// Parsing error from the `uuid` crate
        error: uuid::Error,
        /// String that could not be parsed as a UUID
        str: String,
    },

    /// Missing timestamp error
    #[error("Message is missing a timestamp")]
    MissingTimestamp,

    /// u32 to i16 conversion error
    #[error("Value '{0}' is too big to be a valid i16")]
    U32toI16(u32),

    /// i16 to u32 conversion error
    #[error("Value '{0}' is negative so it can't be a valid u32")]
    I16ToU32(i16),

    /// serde_json to prost-wkt-types Value conversion error
    #[error("Could not convert serde_json Value to a prost-wkt-types Value: {error}")]
    SerdeJsonToProstWktTypesValue {
        /// Convertion error
        error: String,
    },

    /// prost-wkt-types to serde_json Value conversion error
    #[error("Could not convert prost-wkt-types Value to a Vserde_json alue: {error}")]
    ProstWktTypesToSerdeJsonValue {
        /// Convertion error
        error: String,
    },
}
