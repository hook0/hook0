use std::time::Duration;

use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use hook0_client::Hook0ClientError;

const SUBSCRIPTION_SECRET: &str = "ebc17f0b-566e-4d02-be72-df8ec3a6d16c";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || App::new().route("/webhook", web::post().to(handle_webhook)))
        .bind("127.0.0.1:8081")?
        .workers(1)
        .run()
        .await
}

async fn handle_webhook(req: HttpRequest, body: web::Bytes) -> impl Responder {
    let content_type = req.headers().get("Content-Type");
    let event_id = req.headers().get("X-Event-Id");
    let event_type = req.headers().get("X-Event-Type");
    let signature = req.headers().get("X-Hook0-Signature");
    let payload = String::from_utf8(body.to_vec()).unwrap();

    println!("Content-Type: {content_type:?}");
    println!("Event ID: {event_id:?}");
    println!("Event Type: {event_type:?}");
    println!("Signature: {signature:?}");
    println!("Payload: {payload}");

    if let Some(signature) = signature {
        let signature: &str = signature.to_str().unwrap();
        let tolerance = Duration::from_secs(300);

        match hook0_client::verify_webhook_signature(
            signature,
            &body,
            SUBSCRIPTION_SECRET,
            tolerance,
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

    "Ok"
}
