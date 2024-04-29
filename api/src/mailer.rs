use crate::problems::Hook0Problem;
use lettre::message::header;
use lettre::{Address, Message, Transport};
use std::string::String;

pub(crate) struct Mailer {
    mailer: lettre::SmtpTransport,
    server: String,
    port: u16,
    tls: bool,
}

pub(crate) enum Mails {
    VerifyMail {
        subject: String,
        variables: Vec<(String, String)>,
    },
    ResetPassword {
        subject: String,
        variables: Vec<(String, String)>,
    },
}

impl Mails {
    pub fn template(&self) -> &'static str {
        match self {
            Mails::VerifyMail { .. } => include_str!("mails_templates/verify_mail.mjml"),
            Mails::ResetPassword { .. } => include_str!("mails_templates/reset_password.mjml"),
        }
    }

    pub fn subject(&self) -> String {
        match self {
            Mails::VerifyMail { subject, .. } => subject.clone(),
            Mails::ResetPassword { subject, .. } => subject.clone(),
        }
    }

    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mails::VerifyMail { variables, .. } => variables.clone(),
            Mails::ResetPassword { variables, .. } => variables.clone(),
        }
    }
}

impl Mailer {
    pub fn new(
        server: String,
        port: u16,
        tls: bool,
        name: String,
        mail: String,
    ) -> Result<(Mailer, Address), Hook0Problem> {
        let adress = Address::new(name, mail);
        match adress {
            Ok(adress) => {
                let mailer = lettre::SmtpTransport::builder_dangerous(&server)
                    .port(port)
                    .build();

                Ok((
                    Mailer {
                        mailer,
                        server,
                        port,
                        tls,
                    },
                    adress,
                ))
            }
            Err(e) => Err(Hook0Problem::ErrorInBuildAdress(e.to_string())),
        }
    }

    pub fn send_mail(
        &self,
        mail: Mails,
        address: Address,
        from: Address,
    ) -> Result<(), Hook0Problem> {
        let template = mail.template();
        let mut mjml = template.to_string();
        for (key, value) in mail.variables() {
            mjml = mjml.replace(&format!("{{ ${} }}", key), &value);
        }

        match mrml::parse(mjml) {
            Ok(parsed) => match parsed.render(&Default::default()) {
                Ok(rendered) => {
                    let email = Message::builder()
                        .from(from.to_string().as_str().parse()?)
                        .to(address.to_string().as_str().parse()?)
                        .subject(mail.subject())
                        .header(header::ContentType::TEXT_HTML)
                        .body(rendered)
                        .unwrap();

                    let result = self.mailer.send(&email);
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(Hook0Problem::EmailSendFailed(e.to_string())),
                    }
                }
                Err(e) => Err(Hook0Problem::EmailTemplateRenderFailed(e.to_string())),
            },
            Err(e) => Err(Hook0Problem::EmailTemplateParseFailed(e.to_string())),
        }
    }
}
