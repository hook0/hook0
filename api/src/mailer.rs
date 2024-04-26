use lettre::{Message, SmtpTransport, Transport};

fn send_email() {
    let email = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse()?)
        .reply_to("Yuin <yuin@domain.tld>".parse()?)
        .to("Hei <hei@domain.tld>".parse()?)
        .subject("Happy new year")
        .body(String::from("Be happy!"))?;

// Create TLS transport on port 465
    let sender = SmtpTransport::relay("smtp.example.com")?.build();
// Send the email via remote relay
    let result = sender.send(&email);
    assert!(result.is_ok());
}