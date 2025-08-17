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
    website_url: Url,
    app_url: Url,
    support_email_address: Address,
}

#[derive(Debug, Clone)]
pub struct MailerSmtpConfig {
    pub smtp_connection_url: String,
    pub smtp_timeout: Duration,
    pub sender_name: String,
    pub sender_address: Address,
}

#[derive(Debug, Clone)]
pub enum Mail {
    VerifyUserEmail {
        url: Url,
    },
    ResetPassword {
        url: Url,
    },
    // Welcome { name: String },
    QuotaEventsPerDayWarning {
        pricing_url_hash: String,
        actual_consumption_percent: i32,
        current_events_per_day: i32,
        events_per_days_limit: i32,
        extra_variables: Vec<(String, String)>,
    },
    QuotaEventsPerDayReached {
        pricing_url_hash: String,
        current_events_per_day: i32,
        events_per_days_limit: i32,
        extra_variables: Vec<(String, String)>,
    },
}

impl Mail {
    pub fn template(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => include_str!("mail_templates/verify_user_email.mjml"),
            Mail::ResetPassword { .. } => include_str!("mail_templates/reset_password.mjml"),
            // Mail::Welcome { .. } => include_str!("mail_templates/welcome.mjml"),
            Mail::QuotaEventsPerDayWarning { .. } => {
                include_str!("mail_templates/quotas/events_per_day_warning.mjml")
            }
            Mail::QuotaEventsPerDayReached { .. } => {
                include_str!("mail_templates/quotas/events_per_day_reached.mjml")
            }
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Mail::VerifyUserEmail { .. } => "[Hook0] Verify your email address".to_owned(),
            Mail::ResetPassword { .. } => "[Hook0] Reset your password".to_owned(),
            // Mail::Welcome { .. } => "Welcome to our platform".to_owned(),
            Mail::QuotaEventsPerDayWarning { .. } => "[Hook0] Quota Warning".to_owned(),
            Mail::QuotaEventsPerDayReached { .. } => "[Hook0] Quota Reached".to_owned(),
        }
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mail::VerifyUserEmail { url } => vec![("url".to_owned(), url.to_string())],
            Mail::ResetPassword { url } => vec![("url".to_owned(), url.to_string())],
            // Mail::Welcome { name } => vec![("name".to_owned(), name.to_owned())],
            Mail::QuotaEventsPerDayWarning {
                pricing_url_hash,
                actual_consumption_percent,
                current_events_per_day,
                events_per_days_limit,
                extra_variables,
            } => {
                let mut vars = vec![
                    ("pricing_url_hash".to_owned(), pricing_url_hash.to_owned()),
                    (
                        "actual_consumption_percent".to_owned(),
                        actual_consumption_percent.to_string(),
                    ),
                    (
                        "current_events_per_day".to_owned(),
                        current_events_per_day.to_string(),
                    ),
                    (
                        "events_per_days_limit".to_owned(),
                        events_per_days_limit.to_string(),
                    ),
                ];
                vars.extend(extra_variables.clone());
                vars
            }
            Mail::QuotaEventsPerDayReached {
                pricing_url_hash,
                current_events_per_day,
                events_per_days_limit,
                extra_variables,
            } => {
                let mut vars = vec![
                    ("pricing_url_hash".to_owned(), pricing_url_hash.to_owned()),
                    (
                        "current_events_per_day".to_owned(),
                        current_events_per_day.to_string(),
                    ),
                    (
                        "events_per_days_limit".to_owned(),
                        events_per_days_limit.to_string(),
                    ),
                ];
                vars.extend(extra_variables.clone());
                vars
            }
        }
    }

    pub fn add_variable(&mut self, key: String, value: String) {
        match self {
            Mail::QuotaEventsPerDayWarning {
                extra_variables, ..
            } => {
                extra_variables.push((key, value));
            }
            Mail::QuotaEventsPerDayReached {
                extra_variables, ..
            } => {
                extra_variables.push((key, value));
            }
            _ => {}
        }
    }

    pub fn render(
        &self,
        logo_url: &Url,
        website_url: &Url,
        app_url: &Url,
        support_email_address: &Address,
    ) -> Result<String, Hook0Problem> {
        let template = self.template();
        let mut mjml = template.to_owned();
        for (key, value) in self.variables() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &value);
        }

        mjml = mjml.replace("{ $logo_url }", logo_url.as_str());
        mjml = mjml.replace("{ $website_url }", website_url.as_str());
        mjml = mjml.replace("{ $app_url }", app_url.as_str());
        mjml = mjml.replace("{ $support_email_address }", support_email_address.as_ref());

        let parsed = mrml::parse(mjml)?;
        let rendered = parsed.element.render(&Default::default())?;

        Ok(rendered)
    }
}

impl Mailer {
    pub async fn new(
        smtp_config: MailerSmtpConfig,
        logo_url: Url,
        website_url: Url,
        app_url: Url,
        support_email_address: Address,
    ) -> Result<Mailer, lettre::transport::smtp::Error> {
        let transport =
            AsyncSmtpTransport::<Tokio1Executor>::from_url(&smtp_config.smtp_connection_url)?
                .timeout(Some(smtp_config.smtp_timeout))
                .build();
        let sender = Mailbox::new(Some(smtp_config.sender_name), smtp_config.sender_address);

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
            app_url,
            support_email_address,
        })
    }

    pub async fn send_mail(&self, mail: Mail, recipient: Mailbox) -> Result<(), Hook0Problem> {
        let rendered = mail.render(
            &self.logo_url,
            &self.website_url,
            &self.app_url,
            &self.support_email_address,
        )?;
        let text_mail = from_read(rendered.as_bytes(), 80)?;

        let email = Message::builder()
            .from(self.sender.to_owned())
            .to(recipient)
            .subject(mail.subject())
            .multipart(MultiPart::alternative_plain_html(text_mail, rendered))?;

        self.transport.send(email).await?;
        Ok(())
    }

    /// Send a plain text email
    pub async fn send_text_email(
        &self,
        recipient_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Hook0Problem> {
        use std::str::FromStr;
        
        let recipient = Mailbox::new(None, Address::from_str(recipient_email)?);
        
        let email = Message::builder()
            .from(self.sender.to_owned())
            .to(recipient)
            .subject(subject)
            .body(body.to_owned())?;

        self.transport.send(email).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn mrml_rendering() {
        let logo_url = Url::from_str("http://localhost/logo").unwrap();
        let website_url = Url::from_str("http://localhost/website").unwrap();
        let app_url = Url::from_str("http://localhost/app").unwrap();
        let support_email_address = Address::new("test", "hook0.com").unwrap();

        let mails = [
            Mail::VerifyUserEmail {
                url: Url::from_str("http://localhost/verify").unwrap(),
            },
            Mail::ResetPassword {
                url: Url::from_str("http://localhost/verify").unwrap(),
            },
            Mail::QuotaEventsPerDayWarning {
                pricing_url_hash: "test".to_owned(),
                actual_consumption_percent: 0,
                current_events_per_day: 0,
                events_per_days_limit: 0,
                extra_variables: vec![("test".to_owned(), "test".to_owned())],
            },
            Mail::QuotaEventsPerDayReached {
                pricing_url_hash: "test".to_owned(),
                current_events_per_day: 0,
                events_per_days_limit: 0,
                extra_variables: vec![("test".to_owned(), "test".to_owned())],
            },
        ];

        for m in mails {
            assert!(
                m.render(&logo_url, &website_url, &app_url, &support_email_address)
                    .is_ok()
            );
        }
    }
}
