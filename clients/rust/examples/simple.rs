use chrono::Duration;
use hook0_client::{Hook0Client, Hook0ClientError};

const API_URL: &str = "http://localhost:8080";
const APPLICATION_ID: &str = "7cab1450-5820-471e-9396-4ee87384d535";
const SUBSCRIPTION_SECRET: &str = "ebc17f0b-566e-4d02-be72-df8ec3a6d16c"; // Replace with your actual signing secret

const SIGNATURE: &str = "t=1737981303,v0=fb1010dc3b7b6a3b0c0be62e4acd5b0d2771acd94ba9ae6894a9711262f1a3ac";
const PAYLOAD: &str = "{\"test\": true}";
const TOLERANCE: Duration = Duration::seconds(300); // 5-minute tolerance for the timestamp

fn main() {
    let client = Hook0Client::new(
        API_URL.parse().unwrap(), // Replace with your URL
        APPLICATION_ID.parse().unwrap(),
        SUBSCRIPTION_SECRET,
    )
    .unwrap();

    match client.verifying_webhook_signature(SIGNATURE, PAYLOAD.as_bytes(), SUBSCRIPTION_SECRET, TOLERANCE) {
        Ok(_) => println!("Signature verification successful!"),
        Err(Hook0ClientError::InvalidSignature) => {
            println!("Signature verification failed: Invalid signature.")
        }
        Err(Hook0ClientError::ToleranceRefused) => {
            println!("Signature verification failed: Tolerance refused.")
        }
        Err(err) => println!("Signature verification failed: {err}"),
    }
}
