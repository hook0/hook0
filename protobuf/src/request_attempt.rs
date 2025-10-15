use chrono::{DateTime, Utc};
use prost::Message;
use pulsar::producer::Message as PulsarMessage;
use pulsar::{DeserializeMessage, SerializeMessage};
use uuid::Uuid;

use crate::error::Hook0ProtobufError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestAttempt {
    pub application_id: Uuid,
    pub request_attempt_id: Uuid,
    pub event_id: Uuid,
    pub event_received_at: DateTime<Utc>,
    pub subscription_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub retry_count: i16,
    pub http_method: String,
    pub http_url: String,
    pub http_headers: serde_json::Value,
    pub event_type_name: String,
    pub payload: Vec<u8>,
    pub payload_content_type: String,
    pub secret: Uuid,
}

impl TryFrom<crate::raw_proto::request_attempt::RequestAttempt> for RequestAttempt {
    type Error = Hook0ProtobufError;

    fn try_from(
        value: crate::raw_proto::request_attempt::RequestAttempt,
    ) -> Result<Self, Self::Error> {
        let application_id = if value.application_id.is_empty() {
            // This field was added afterwards so we need to avoid failing if it is empty
            Uuid::nil()
        } else {
            Uuid::parse_str(&value.application_id).map_err(|error| {
                Hook0ProtobufError::InvalidUuid {
                    error,
                    str: value.application_id,
                }
            })?
        };
        let request_attempt_id = Uuid::parse_str(&value.request_attempt_id).map_err(|error| {
            Hook0ProtobufError::InvalidUuid {
                error,
                str: value.request_attempt_id,
            }
        })?;
        let event_id =
            Uuid::parse_str(&value.event_id).map_err(|error| Hook0ProtobufError::InvalidUuid {
                error,
                str: value.event_id,
            })?;
        let event_received_at = value
            .event_received_at
            .map(DateTime::from)
            // This field was added afterwards so we need to avoid failing if it is empty
            .unwrap_or(DateTime::<Utc>::MIN_UTC);
        let subscription_id = Uuid::parse_str(&value.subscription_id).map_err(|error| {
            Hook0ProtobufError::InvalidUuid {
                error,
                str: value.subscription_id,
            }
        })?;
        let created_at = value
            .created_at
            .map(DateTime::from)
            .ok_or(Hook0ProtobufError::MissingTimestamp)?;

        let retry_count = i16::try_from(value.retry_count)
            .map_err(|_| Hook0ProtobufError::U32toI16(value.retry_count))?;
        let http_header =
            serde_json::to_value(value.http_headers.unwrap_or_default()).map_err(|e| {
                Hook0ProtobufError::ProstWktTypesToSerdeJsonValue {
                    error: e.to_string(),
                }
            })?;
        let secret =
            Uuid::parse_str(&value.secret).map_err(|error| Hook0ProtobufError::InvalidUuid {
                error,
                str: value.secret,
            })?;

        Ok(Self {
            application_id,
            request_attempt_id,
            event_id,
            event_received_at,
            subscription_id,
            created_at,
            retry_count,
            http_method: value.http_method,
            http_url: value.http_url,
            http_headers: http_header,
            event_type_name: value.event_type_name,
            payload: value.payload,
            payload_content_type: value.payload_content_type,
            secret,
        })
    }
}

impl TryFrom<RequestAttempt> for crate::raw_proto::request_attempt::RequestAttempt {
    type Error = Hook0ProtobufError;

    fn try_from(value: RequestAttempt) -> Result<Self, Self::Error> {
        let retry_count = u32::try_from(value.retry_count)
            .map_err(|_| Hook0ProtobufError::I16ToU32(value.retry_count))?;
        let http_headers = Some(
            serde_json::from_value::<prost_wkt_types::Value>(value.http_headers).map_err(|e| {
                Hook0ProtobufError::SerdeJsonToProstWktTypesValue {
                    error: e.to_string(),
                }
            })?,
        );

        Ok(Self {
            application_id: value.application_id.to_string(),
            request_attempt_id: value.request_attempt_id.to_string(),
            event_id: value.event_id.to_string(),
            event_received_at: Some(value.event_received_at.into()),
            subscription_id: value.subscription_id.to_string(),
            created_at: Some(value.created_at.into()),
            retry_count,
            http_method: value.http_method,
            http_url: value.http_url,
            http_headers,
            event_type_name: value.event_type_name,
            payload: value.payload,
            payload_content_type: value.payload_content_type,
            secret: value.secret.to_string(),
        })
    }
}

impl TryFrom<&[u8]> for RequestAttempt {
    type Error = Hook0ProtobufError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let proto_value = crate::raw_proto::request_attempt::RequestAttempt::decode(value)?;
        proto_value.try_into()
    }
}

impl DeserializeMessage for RequestAttempt {
    type Output = Result<Self, Hook0ProtobufError>;

    fn deserialize_message(payload: &pulsar::Payload) -> Self::Output {
        payload.data.as_slice().try_into()
    }
}

impl SerializeMessage for RequestAttempt {
    fn serialize_message(input: Self) -> Result<PulsarMessage, pulsar::Error> {
        let proto: crate::raw_proto::request_attempt::RequestAttempt = input
            .try_into()
            .map_err(|e: Hook0ProtobufError| pulsar::Error::Custom(e.to_string()))?;
        let mut payload = Vec::new();
        proto
            .encode(&mut payload)
            .map_err(|e| pulsar::Error::Custom(e.to_string()))?;

        Ok(PulsarMessage {
            payload,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::TimeZone;
    use serde_json::json;
    use uuid::uuid;

    #[test]
    fn protobuf_conversion() {
        let request_attempt = RequestAttempt {
            application_id: uuid!("00000000-0000-0000-0000-000000000000"),
            request_attempt_id: uuid!("00000000-0000-0000-0000-000000000001"),
            event_id: uuid!("00000000-0000-0000-0000-000000000002"),
            event_received_at: Utc.with_ymd_and_hms(2025, 10, 5, 16, 0, 41).unwrap(),
            subscription_id: uuid!("00000000-0000-0000-0000-000000000003"),
            created_at: Utc.with_ymd_and_hms(2025, 10, 5, 16, 0, 42).unwrap(),
            retry_count: 42,
            http_method: "POST".to_owned(),
            http_url: "http://localhost/target".to_owned(),
            http_headers: json!({
                "x-test-1": "test1",
                "x-test-2": "test2",
            }),
            event_type_name: "test.test.test".to_owned(),
            payload: b"this is a test payload".to_vec(),
            payload_content_type: "text/plain".to_owned(),
            secret: uuid!("00000000-0000-0000-0000-000000000004"),
        };
        let proto_request_attempt: crate::raw_proto::request_attempt::RequestAttempt =
            request_attempt.clone().try_into().unwrap();
        let output: RequestAttempt = proto_request_attempt.try_into().unwrap();
        assert_eq!(output, request_attempt)
    }
}
