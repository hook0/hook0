use html2text::from_read;
use lettre::message::{Mailbox, MultiPart};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use log::{info, warn};
use std::string::String;
use std::time::Duration;
use url::Url;

use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    sender: Mailbox,
    logo_url: Url,
}

pub enum Mail {
    VerifyUserEmail { url: String },
    ResetPassword { url: String },
    // Welcome { name: String },
}

impl Mail {
    pub fn template(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => include_str!("mail_templates/verify_user_email.mjml"),
            Mail::ResetPassword { .. } => include_str!("mail_templates/reset_password.mjml"),
            // Mail::Welcome { .. } => include_str!("mail_templates/welcome.mjml"),
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Mail::VerifyUserEmail { .. } => "Please verify your email address".to_owned(),
            Mail::ResetPassword { .. } => "Reset your password".to_owned(),
            // Mail::Welcome { .. } => "Welcome to our platform".to_owned(),
        }
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mail::VerifyUserEmail { url } => vec![("url".to_owned(), url.to_owned())],
            Mail::ResetPassword { url } => vec![("url".to_owned(), url.to_owned())],
            // Mail::Welcome { name } => vec![("name".to_owned(), name.to_owned())],
        }
    }
}

impl Mailer {
    pub async fn new(
        smtp_connection_url: &str,
        smtp_timeout: Duration,
        sender_name: String,
        sender_address: Address,
        logo_url: Url,
    ) -> Result<Mailer, lettre::transport::smtp::Error> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_connection_url)?
            .timeout(Some(smtp_timeout))
            .build();
        let sender = Mailbox::new(Some(sender_name), sender_address);

        let test = transport.test_connection().await;
        match test {
            Ok(true) => info!("SMTP server is up"),
            Ok(false) => warn!("SMTP server connection test failed"),
            Err(e) => warn!("SMTP server connection test failed: {e}"),
        }

        Ok(Mailer {
            transport,
            sender,
            logo_url,
        })
    }

    pub async fn send_mail(&self, mail: Mail, recipient: Mailbox) -> Result<(), Hook0Problem> {
        let template = mail.template();
        let mut mjml = template.to_owned();
        for (key, value) in mail.variables() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &value);
        }

        // Replace the logo_url variable with the actual logo_url value if { $logo_url } is present in the template
        mjml = mjml.replace("{ $logo_url }", self.logo_url.as_str());

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
