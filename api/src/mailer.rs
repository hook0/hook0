use html2text::from_read;
use lettre::message::{Mailbox, MultiPart};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::string::String;

use crate::problems::Hook0Problem;

pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    sender: Mailbox,
}

pub enum Mail {
    VerifyMail { url: String },
    ResetPassword { url: String },
    Welcome { name: String },
}

impl Mail {
    pub fn template(&self) -> &'static str {
        match self {
            Mail::VerifyMail { .. } => include_str!("mails_templates/verify_mail.mjml"),
            Mail::ResetPassword { .. } => include_str!("mails_templates/reset_password.mjml"),
            Mail::Welcome { .. } => include_str!("mails_templates/welcome.mjml"),
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Mail::VerifyMail { .. } => "Please verify your email address".to_owned(),
            Mail::ResetPassword { .. } => "Reset your password".to_owned(),
            Mail::Welcome { .. } => "Welcome to our platform".to_owned(),
        }
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mail::VerifyMail { url, .. } => vec![("url".to_owned(), url.to_owned())],
            Mail::ResetPassword { url, .. } => vec![("url".to_owned(), url.to_owned())],
            Mail::Welcome { name, .. } => vec![("name".to_owned(), name.to_owned())],
        }
    }
}

impl Mailer {
    pub fn new(
        server: String,
        port: u16,
        tls: bool,
        sender_name: String,
        sender_address: Address,
    ) -> Mailer {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(server)
            .port(port)
            .build();
        let sender = Mailbox::new(Some(sender_name), sender_address);

        Mailer { transport, sender }
    }

    pub async fn send_mail(&self, mail: Mail, recipient: Mailbox) -> Result<(), Hook0Problem> {
        let template = mail.template();
        let mut mjml = template.to_owned();
        for (key, value) in mail.variables() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &value);
        }

        let parsed = mrml::parse(mjml)?;
        let rendered = parsed.render(&Default::default())?;

        let text_mail = from_read(rendered.as_bytes(), 80);

        let email = Message::builder()
            .from(self.sender.to_owned())
            .to(recipient)
            .subject(mail.subject())
            .multipart(MultiPart::alternative_plain_html(text_mail, rendered))?;

        self.transport.send(email).await?;
        Ok(())
    }
}
