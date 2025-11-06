use prost::Message;

use crate::error::Hook0ProtobufError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectStorageResponse {
    pub body: Vec<u8>,
    pub headers: serde_json::Value,
}

impl TryFrom<crate::raw_proto::object_storage_response::ObjectStorageResponse>
    for ObjectStorageResponse
{
    type Error = Hook0ProtobufError;

    fn try_from(
        value: crate::raw_proto::object_storage_response::ObjectStorageResponse,
    ) -> Result<Self, Self::Error> {
        let headers = serde_json::to_value(value.headers.unwrap_or_default()).map_err(|e| {
            Hook0ProtobufError::ProstWktTypesToSerdeJsonValue {
                error: e.to_string(),
            }
        })?;

        Ok(Self {
            body: value.body,
            headers,
        })
    }
}

impl TryFrom<ObjectStorageResponse>
    for crate::raw_proto::object_storage_response::ObjectStorageResponse
{
    type Error = Hook0ProtobufError;

    fn try_from(value: ObjectStorageResponse) -> Result<Self, Self::Error> {
        let headers = Some(
            serde_json::from_value::<prost_wkt_types::Value>(value.headers).map_err(|e| {
                Hook0ProtobufError::SerdeJsonToProstWktTypesValue {
                    error: e.to_string(),
                }
            })?,
        );

        Ok(Self {
            body: value.body,
            headers,
        })
    }
}

impl TryFrom<&[u8]> for ObjectStorageResponse {
    type Error = Hook0ProtobufError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let proto_value =
            crate::raw_proto::object_storage_response::ObjectStorageResponse::decode(value)?;
        proto_value.try_into()
    }
}

impl TryFrom<ObjectStorageResponse> for Vec<u8> {
    type Error = Hook0ProtobufError;

    fn try_from(value: ObjectStorageResponse) -> Result<Self, Self::Error> {
        let proto: crate::raw_proto::object_storage_response::ObjectStorageResponse =
            value.try_into()?;
        let mut buf = Vec::new();
        proto
            .encode(&mut buf)
            .map_err(Hook0ProtobufError::ProtoEncode)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn protobuf_conversion() {
        let request_attempt = ObjectStorageResponse {
            body: b"test".to_vec(),
            headers: json!({
                "x-test-1": "test1",
                "x-test-2": "test2",
            }),
        };
        let proto_request_attempt: crate::raw_proto::object_storage_response::ObjectStorageResponse =
            request_attempt.clone().try_into().unwrap();
        let output: ObjectStorageResponse = proto_request_attempt.try_into().unwrap();
        assert_eq!(output, request_attempt)
    }
}
