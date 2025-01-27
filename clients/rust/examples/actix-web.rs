use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use chrono::Duration;
use hook0_client::{Hook0Client, Hook0ClientError};
use std::sync::Arc;

const API_URL: &str = "http://localhost:8080";
const APPLICATION_ID: &str = "7cab1450-5820-471e-9396-4ee87384d535";
const SUBSCRIPTION_SECRET: &str = "ebc17f0b-566e-4d02-be72-df8ec3a6d16c";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Hook0Client::new(
        API_URL.parse().unwrap(),
        uuid::Uuid::parse_str(APPLICATION_ID).unwrap(),
        SUBSCRIPTION_SECRET,
    )
    .unwrap();

    let shared_client = web::Data::new(Arc::new(client));

    HttpServer::new(move || {
        App::new()
            .app_data(shared_client.clone())
            .route("/webhook", web::post().to(handle_webhook))
    })
    .bind("127.0.0.1:8081")?
    .workers(1)
    .run()
    .await
}

async fn handle_webhook(
    req: HttpRequest,
    body: web::Bytes,
    client: web::Data<Arc<Hook0Client>>,
) -> impl Responder {
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
        let signature = signature.to_str().unwrap();
        let tolerance = Duration::seconds(300);

        match client
            .verifying_webhook_signature(signature, &body, SUBSCRIPTION_SECRET, tolerance)
        {
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

    "Ok"
}
