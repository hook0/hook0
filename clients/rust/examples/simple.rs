use std::time::Duration;

use hook0_client::Hook0ClientError;

const SIGNATURE: &str =
    "t=1737981303,v0=fb1010dc3b7b6a3b0c0be62e4acd5b0d2771acd94ba9ae6894a9711262f1a3ac";
const PAYLOAD: &str = "{\"test\": true}";
const SUBSCRIPTION_SECRET: &str = "ebc17f0b-566e-4d02-be72-df8ec3a6d16c"; // Replace with your actual signing secret
const TOLERANCE: Duration = Duration::from_secs(300); // 5-minute tolerance for the timestamp

fn main() {
    match hook0_client::verify_webhook_signature(
        SIGNATURE,
        PAYLOAD.as_bytes(),
        SUBSCRIPTION_SECRET,
        TOLERANCE,
    ) {
        Ok(_) => println!("Signature verification successful!"),
        Err(Hook0ClientError::InvalidSignature) => {
            println!("Signature verification failed: Invalid signature.")
        }
        Err(Hook0ClientError::ExpiredWebhook {
            signed_at,
            tolerance,
            current_time,
        }) => {
            println!("Signature verification failed: The webhook has expired because it was sent too long ago (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})")
        }
        Err(Hook0ClientError::SignatureParsing(signature)) => {
            println!("Signature verification failed: Could not parse signature: {signature}")
        }
        Err(Hook0ClientError::TimestampParsingInSignature(timestamp)) => {
            println!("Signature verification failed: Could not parse timestamp in signature: {timestamp}")
        }
        Err(Hook0ClientError::InvalidTolerance(err)) => {
            println!("Signature verification failed: Invalid tolerance: {err}")
        }
        Err(err) => println!("Signature verification failed: {err}"),
    }
}
