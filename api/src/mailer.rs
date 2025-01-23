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
    website_url: String,
}

pub enum Mail {
    VerifyUserEmail {
        url: String,
    },
    ResetPassword {
        url: String,
    },
    // Welcome { name: String },
    // QuotaWarning { quota_name: String, pricing_url_hash: String, informations: String },
    QuotaReached {
        quota_name: String,
        pricing_url_hash: String,
        informations: String,
        entity_type: String,
        entity_url: String,
    },
}

impl Mail {
    pub fn template(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => include_str!("mail_templates/verify_user_email.mjml"),
            Mail::ResetPassword { .. } => include_str!("mail_templates/reset_password.mjml"),
            // Mail::Welcome { .. } => include_str!("mail_templates/welcome.mjml"),
            // Mail::QuotaWarning { .. } => include_str!("mail_templates/quota_warning.mjml"),
            Mail::QuotaReached { .. } => include_str!("mail_templates/quotas_reached.mjml"),
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Mail::VerifyUserEmail { .. } => "[Hook0] Verify your email address".to_owned(),
            Mail::ResetPassword { .. } => "[Hook0] Reset your password".to_owned(),
            // Mail::Welcome { .. } => "Welcome to our platform".to_owned(),
            // Mail::QuotaWarning { .. } => "[Hook0] Quota at 90%".to_owned(),
            Mail::QuotaReached { .. } => "[Hook0] Quota reached".to_owned(),
        }
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mail::VerifyUserEmail { url } => vec![("url".to_owned(), url.to_owned())],
            Mail::ResetPassword { url } => vec![("url".to_owned(), url.to_owned())],
            // Mail::Welcome { name } => vec![("name".to_owned(), name.to_owned())],
            // Mail::QuotaWarning { quota_name, pricing_url_hash, informations } => vec![("quota_name".to_owned(), quota_name.to_owned()), ("pricing_url_hash".to_owned(), pricing_url_hash.to_owned()), ("informations".to_owned(), informations.to_owned())],
            Mail::QuotaReached {
                quota_name,
                pricing_url_hash,
                informations,
                entity_type,
                entity_url,
            } => vec![
                ("quota_name".to_owned(), quota_name.to_owned()),
                ("pricing_url_hash".to_owned(), pricing_url_hash.to_owned()),
                ("informations".to_owned(), informations.to_owned()),
                ("entity_type".to_owned(), entity_type.to_owned()),
                ("entity_url".to_owned(), entity_url.to_owned()),
            ],
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
        website_url: String,
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
            website_url,
        })
    }

    pub async fn send_mail(&self, mail: Mail, recipient: Mailbox) -> Result<(), Hook0Problem> {
        let template = mail.template();
        let mut mjml = template.to_owned();
        for (key, value) in mail.variables() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &value);
        }

        mjml = mjml.replace("{ $logo_url }", self.logo_url.as_str());
        mjml = mjml.replace("{ $website_url }", self.website_url.as_str());
        mjml = mjml.replace("{ $app_url }", self.website_url.as_str()); // TODO: replace with real app_url

        let parsed = mrml::parse(mjml)?;
        let rendered = parsed.render(&Default::default())?;

        let text_mail = from_read(rendered.as_bytes(), 80)?;

        let email = Message::builder()
            .from(self.sender.to_owned())
            .to(recipient)
            .subject(mail.subject())
            .multipart(MultiPart::alternative_plain_html(text_mail, rendered))?;

        self.transport.send(email).await?;
        Ok(())
    }
}
