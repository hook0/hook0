use actix_web::web::Data;
use actix_web::{App, HttpRequest, HttpServer, Responder, web};
use std::env;
use std::time::Duration;

use hook0_client::Hook0ClientError;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let ip = env::var("IP").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT")
        .ok()
        .and_then(|str| str.parse::<u16>().ok())
        .unwrap_or(8082);

    let subscription_secret = env::var("SUBSCRIPTION_SECRET")
        .expect("You must define a SUBSCRIPTION_SECRET environment variable");

    println!("Waiting webhooks as POST on http://{ip}:{port}/webhook");
    HttpServer::new(move || {
        App::new()
            .route("/webhook", web::post().to(handle_webhook))
            .app_data(Data::new(subscription_secret.to_owned()))
    })
    .bind((ip, port))?
    .workers(1)
    .run()
    .await
}

async fn handle_webhook(
    subscription_secret: Data<String>,
    req: HttpRequest,
    body: web::Bytes,
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
        let signature: &str = signature.to_str().unwrap();
        let tolerance = Duration::from_secs(300);

        match hook0_client::verify_webhook_signature(
            signature,
            &body,
            &req.headers().iter().collect::<Vec<_>>(),
            subscription_secret.into_inner().as_str(),
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
                println!(
                    "Signature verification failed: The webhook has expired because it was sent too long ago (signed_at={signed_at}, tolerance={tolerance}, current_time={current_time})"
                )
            }
            Err(e) => {
                println!("Signature verification failed: {e}")
            }
        }
    }

    "Ok"
}
