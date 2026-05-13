use chrono::Datelike;
use html2text::from_read;
use lettre::message::{Mailbox, MultiPart};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::string::String;
use std::time::Duration;
use tracing::{info, warn};
use url::Url;

use crate::problems::Hook0Problem;

#[derive(Debug, Clone)]
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    sender: Mailbox,
    logo_url: Url,
    website_url: Url,
    app_url: Url,
    doc_url: Url,
    privacy_policy_url: Url,
    support_email_address: Address,
    company_legal_name: String,
    company_postal_address: String,
    company_rcs: String,
}

#[derive(Debug, Clone)]
pub struct MailerSmtpConfig {
    pub smtp_connection_url: String,
    pub smtp_timeout: Duration,
    pub sender_name: String,
    pub sender_address: Address,
}

/// Every variant requires `recipient_first_name` to be hydrated before
/// the mail is rendered. The field is `Option<String>` because quota mails
/// are constructed as a *template* (one Mail value, fanned out to N admins)
/// and the per-recipient first name is set inside the send loop. Render-time
/// validation (see `Mail::render`) guarantees we never ship a mail with an
/// unhydrated `{ $recipient_first_name }` placeholder.
#[derive(Debug, Clone)]
pub enum Mail {
    VerifyUserEmail {
        recipient_first_name: Option<String>,
        url: Url,
    },
    ResetPassword {
        recipient_first_name: Option<String>,
        url: Url,
    },
    Welcome {
        recipient_first_name: Option<String>,
    },
    QuotaEventsPerDayWarning {
        recipient_first_name: Option<String>,
        pricing_url_hash: String,
        actual_consumption_percent: i32,
        current_events_per_day: i32,
        events_per_days_limit: i32,
        extra_variables: Vec<(String, String)>,
    },
    QuotaEventsPerDayReached {
        recipient_first_name: Option<String>,
        pricing_url_hash: String,
        current_events_per_day: i32,
        events_per_days_limit: i32,
        extra_variables: Vec<(String, String)>,
    },
}

// Design tokens — sourced from www.hook0.com brand:
//   Primary  : #22c55e (green-500) for the single primary CTA
//   Accent   : #6366f1 (indigo-500) for links + brand mark accent
//   Ink      : #0f172a (slate-900) for body copy
//   Mute     : #475569 (slate-600) for secondary copy
//   Hush     : #94a3b8 (slate-400) for footer disclaimers
//   Canvas   : #f8fafc (slate-50) for the body background
//   Card     : #ffffff with a slate-100 border instead of a heavy shadow
//   Border   : #e2e8f0 (slate-200)
//
// CRITICAL: MRML expands every `<mj-text>` into an inner `<div>` with inline
// `font-size`/`color`/`line-height`. That inline style WINS over `<mj-style>`
// CSS classes, so we set every font-size/color directly via MJML attributes,
// never via `css-class`.
//
// The header is a typographic wordmark ("Hook0" with a green zero) — the
// stock PNG has a baked-in white background that reads as a sticker on the
// canvas, and a wordmark renders identically in every email client without
// images-disabled fallbacks.

const MJML_HEADER: &str = r##"<mjml>
  <mj-head>
    <mj-title>Hook0</mj-title>
    <mj-preview>{ $preheader }</mj-preview>
    <mj-attributes>
      <mj-all font-family="Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif" />
      <mj-text font-size="15px" line-height="1.65" color="#0f172a" padding="0" />
      <mj-button background-color="#22c55e" color="#ffffff" font-weight="600" font-size="15px" border-radius="10px" inner-padding="14px 28px" padding="8px 0 0 0" />
      <mj-section padding="0" />
    </mj-attributes>
    <mj-style inline="inline">
      a { color: #6366f1; text-decoration: none; }
      a:hover { text-decoration: underline; }
      h1 { font-size: 28px; line-height: 1.2; margin: 0; color: #0f172a; font-weight: 700; letter-spacing: -0.01em; }
      .codelink { display: inline-block; font-family: ui-monospace, 'SF Mono', Menlo, Consolas, monospace; font-size: 12px; color: #475569; background: #f1f5f9; border: 1px solid #e2e8f0; border-radius: 6px; padding: 6px 10px; word-break: break-all; line-height: 1.5; }
      .codelink a { color: #475569; text-decoration: none; }
    </mj-style>
  </mj-head>
  <mj-body background-color="#f8fafc" width="600px">
    <mj-section padding="36px 0 20px 0">
      <mj-column>
        <mj-image src="{ $logo_url }" alt="Hook0" width="160px" height="63px" align="left" padding="0 16px" />
      </mj-column>
    </mj-section>
    <mj-section background-color="#ffffff" border="1px solid #e2e8f0" border-radius="14px" padding="40px 36px 32px 36px">
      <mj-column>
"##;

// Footer text rendered at 11-12 px slate-400 — discreet legal mention without
// stealing visual weight from the main action. The brand identity line keeps
// the Hook0 tagline visible even when the legal entity (FGRibreau SARL) below
// it carries the bulk of the LCEN/RCPS information.
const MJML_FOOTER_TRANSACTIONAL: &str = r##"      </mj-column>
    </mj-section>
    <mj-section padding="20px 24px 32px 24px">
      <mj-column>
        <mj-text align="left" font-size="11px" color="#94a3b8" line-height="1.7">
          Hook0 &middot; Open-Source Webhooks-as-a-Service &middot; Made in Europe
        </mj-text>
        <mj-text align="left" font-size="12px" color="#94a3b8" line-height="1.7" padding="6px 0 0 0">
          Need a hand? <a href="mailto:{ $support_email_address }" style="color:#475569;">{ $support_email_address }</a> &middot; <a href="{ $app_url_tracked }" style="color:#475569;">Open dashboard</a> &middot; <a href="{ $privacy_policy_url_tracked }" style="color:#475569;">Privacy &amp; data rights</a>
        </mj-text>
        <mj-text align="left" font-size="11px" color="#94a3b8" line-height="1.7" padding="10px 0 0 0">
          Transactional notification sent to the address tied to your Hook0 account. You have rights over your personal data (access, correction, deletion, portability, objection).
        </mj-text>
        <mj-text align="left" font-size="11px" color="#cbd5e1" line-height="1.7" padding="10px 0 0 0">
          &copy; { $current_year } { $company_legal_name } &middot; { $company_postal_address } &middot; { $company_rcs }
        </mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;

const MJML_FOOTER_COMMERCIAL: &str = r##"      </mj-column>
    </mj-section>
    <mj-section padding="20px 24px 32px 24px">
      <mj-column>
        <mj-text align="left" font-size="11px" color="#94a3b8" line-height="1.7">
          Hook0 &middot; Open-Source Webhooks-as-a-Service &middot; Made in Europe
        </mj-text>
        <mj-text align="left" font-size="12px" color="#94a3b8" line-height="1.7" padding="6px 0 0 0">
          Need a hand? <a href="mailto:{ $support_email_address }" style="color:#475569;">{ $support_email_address }</a> &middot; <a href="{ $app_url_tracked }" style="color:#475569;">Open dashboard</a> &middot; <a href="{ $privacy_policy_url_tracked }" style="color:#475569;">Privacy &amp; data rights</a>
        </mj-text>
        <mj-text align="left" font-size="11px" color="#94a3b8" line-height="1.7" padding="10px 0 0 0">
          You're an admin on a Hook0 account so we send these account-status alerts. To stop the upgrade suggestions only, email <a href="mailto:{ $support_email_address }?subject=Unsubscribe%20upgrade%20suggestions" style="color:#475569;">{ $support_email_address }</a> with subject &laquo;&nbsp;Unsubscribe upgrade suggestions&nbsp;&raquo;. You have rights over your personal data (access, correction, deletion, portability, objection).
        </mj-text>
        <mj-text align="left" font-size="11px" color="#cbd5e1" line-height="1.7" padding="10px 0 0 0">
          &copy; { $current_year } { $company_legal_name } &middot; { $company_postal_address } &middot; { $company_rcs }
        </mj-text>
      </mj-column>
    </mj-section>
  </mj-body>
</mjml>
"##;

/// Append Matomo tracking parameters to a URL while preserving existing
/// query string and fragment. Used to tag every clickable link in
/// transactional emails so analytics can attribute downstream activity.
fn with_matomo(url: &Url, campaign: &str) -> String {
    let mut u = url.clone();
    u.query_pairs_mut()
        .append_pair("mtm_source", "email")
        .append_pair("mtm_medium", "transactional")
        .append_pair("mtm_campaign", campaign);
    u.to_string()
}

impl Mail {
    pub fn template(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => include_str!("mail_templates/verify_user_email.mjml"),
            Mail::ResetPassword { .. } => include_str!("mail_templates/reset_password.mjml"),
            Mail::Welcome { .. } => include_str!("mail_templates/welcome.mjml"),
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
            Mail::VerifyUserEmail { .. } => "Verify your Hook0 email".to_owned(),
            Mail::ResetPassword { .. } => "Reset your Hook0 password".to_owned(),
            Mail::Welcome { .. } => "You're in. Let's ship your first webhook.".to_owned(),
            Mail::QuotaEventsPerDayWarning {
                actual_consumption_percent,
                ..
            } => format!("You're at {actual_consumption_percent}% of your daily events"),
            Mail::QuotaEventsPerDayReached { .. } => {
                "Daily event limit reached. Events paused.".to_owned()
            }
        }
    }

    pub fn preheader(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => {
                "One click to activate your account. Link expires in 5 minutes."
            }
            Mail::ResetPassword { .. } => {
                "Link expires in 5 minutes. All sessions will be signed out."
            }
            Mail::Welcome { .. } => {
                "Create an application, send a test event, done in under 5 minutes."
            }
            Mail::QuotaEventsPerDayWarning { .. } => {
                "Heads-up before you hit the limit and events start getting blocked."
            }
            Mail::QuotaEventsPerDayReached { .. } => {
                "Hook0 will resume at the next daily reset, or as soon as you upgrade."
            }
        }
    }

    /// Matomo campaign slug for this mail. Used by `with_matomo` to tag
    /// every clickable link with `mtm_campaign=<this>`.
    pub fn matomo_campaign(&self) -> &'static str {
        match self {
            Mail::VerifyUserEmail { .. } => "verify_email",
            Mail::ResetPassword { .. } => "reset_password",
            Mail::Welcome { .. } => "welcome",
            Mail::QuotaEventsPerDayWarning { .. } => "quota_warning",
            Mail::QuotaEventsPerDayReached { .. } => "quota_reached",
        }
    }

    /// Whether this mail contains a commercial component (upgrade CTA).
    /// Drives footer selection so quota emails carry the L34-5 al.3 CPCE
    /// opt-out mention.
    pub fn has_commercial_component(&self) -> bool {
        matches!(
            self,
            Mail::QuotaEventsPerDayWarning { .. } | Mail::QuotaEventsPerDayReached { .. }
        )
    }

    /// Read the `recipient_first_name` from any variant. Every variant
    /// currently requires it — kept as an accessor so a future variant
    /// that legitimately doesn't need a recipient name can return None
    /// without leaking the implementation.
    pub fn recipient_first_name(&self) -> Option<&str> {
        match self {
            Mail::VerifyUserEmail {
                recipient_first_name,
                ..
            }
            | Mail::ResetPassword {
                recipient_first_name,
                ..
            }
            | Mail::Welcome {
                recipient_first_name,
            }
            | Mail::QuotaEventsPerDayWarning {
                recipient_first_name,
                ..
            }
            | Mail::QuotaEventsPerDayReached {
                recipient_first_name,
                ..
            } => recipient_first_name.as_deref(),
        }
    }

    /// Per-mail text variables, NOT including `recipient_first_name`
    /// (that one is validated and substituted directly in `render`).
    /// No URL tracking applied here — `tracked_urls()` covers that.
    pub fn variables(&self) -> Vec<(String, String)> {
        match self {
            Mail::VerifyUserEmail { .. } => vec![],
            Mail::ResetPassword { .. } => vec![],
            Mail::Welcome { .. } => vec![],
            Mail::QuotaEventsPerDayWarning {
                actual_consumption_percent,
                current_events_per_day,
                events_per_days_limit,
                extra_variables,
                ..
            } => {
                let mut vars = vec![
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
                current_events_per_day,
                events_per_days_limit,
                extra_variables,
                ..
            } => {
                let mut vars = vec![
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

    /// Per-mail URL variables that must be Matomo-tracked before substitution.
    fn tracked_urls(&self) -> Vec<(String, Url)> {
        match self {
            Mail::VerifyUserEmail { url, .. } => vec![("url".to_owned(), url.clone())],
            Mail::ResetPassword { url, .. } => vec![("url".to_owned(), url.clone())],
            Mail::Welcome { .. } => vec![],
            Mail::QuotaEventsPerDayWarning { .. } => vec![],
            Mail::QuotaEventsPerDayReached { .. } => vec![],
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

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        logo_url: &Url,
        website_url: &Url,
        app_url: &Url,
        doc_url: &Url,
        privacy_policy_url: &Url,
        support_email_address: &Address,
        company_legal_name: &str,
        company_postal_address: &str,
        company_rcs: &str,
    ) -> Result<String, Hook0Problem> {
        let campaign = self.matomo_campaign();
        // Recipient name is optional: quota mails are constructed as a
        // template in `handlers/events.rs` before the recipient list is
        // known, then cloned and hydrated per-admin inside the send loop.
        // When the name is missing (template stage, or any caller that
        // doesn't have it), fall back to a generic "there" greeting so
        // the mail still renders cleanly as "Hi there, …". All currently
        // shipped templates greet via a body-level "Hi {name}," so the
        // fallback reads naturally; the welcome H1 was restructured to
        // not depend on the name structurally.
        let recipient_first_name = self.recipient_first_name().unwrap_or("there");

        let footer = if self.has_commercial_component() {
            MJML_FOOTER_COMMERCIAL
        } else {
            MJML_FOOTER_TRANSACTIONAL
        };

        let mut mjml = format!("{}{}{}", MJML_HEADER, self.template(), footer);

        // Preheader (per-mail static text)
        mjml = mjml.replace("{ $preheader }", self.preheader());

        // Recipient greeting — substituted unconditionally; the fallback
        // above ensures the placeholder is never left empty.
        mjml = mjml.replace("{ $recipient_first_name }", recipient_first_name);

        // Untracked globals
        mjml = mjml.replace("{ $logo_url }", logo_url.as_str());
        mjml = mjml.replace("{ $website_url }", website_url.as_str());
        mjml = mjml.replace("{ $app_url }", app_url.as_str());
        mjml = mjml.replace("{ $support_email_address }", support_email_address.as_ref());

        // Tracked globals
        mjml = mjml.replace("{ $app_url_tracked }", &with_matomo(app_url, campaign));
        mjml = mjml.replace("{ $doc_url_tracked }", &with_matomo(doc_url, campaign));
        mjml = mjml.replace(
            "{ $privacy_policy_url_tracked }",
            &with_matomo(privacy_policy_url, campaign),
        );

        // Text globals
        mjml = mjml.replace("{ $company_legal_name }", company_legal_name);
        mjml = mjml.replace("{ $company_postal_address }", company_postal_address);
        mjml = mjml.replace("{ $company_rcs }", company_rcs);
        mjml = mjml.replace("{ $current_year }", &chrono::Utc::now().year().to_string());

        // Per-mail composite tracked URL: pricing_url_tracked = website_url + pricing_url_hash
        if let Some(pricing_hash) = match self {
            Mail::QuotaEventsPerDayWarning {
                pricing_url_hash, ..
            }
            | Mail::QuotaEventsPerDayReached {
                pricing_url_hash, ..
            } => Some(pricing_url_hash.as_str()),
            _ => None,
        } {
            let composite = format!("{}{}", website_url.as_str(), pricing_hash);
            let pricing_tracked = match Url::parse(&composite) {
                Ok(parsed) => with_matomo(&parsed, campaign),
                Err(_) => composite,
            };
            mjml = mjml.replace("{ $pricing_url_tracked }", &pricing_tracked);
        }

        // Per-mail tracked URLs
        for (key, url) in self.tracked_urls() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &with_matomo(&url, campaign));
        }

        // Per-mail text variables (including extra_variables which may already
        // contain pre-tracked URLs computed by handlers, e.g. quotas dashboard).
        for (key, value) in self.variables() {
            mjml = mjml.replace(&format!("{{ ${key} }}"), &value);
        }

        let parsed = mrml::parse(mjml)?;
        let rendered = parsed.element.render(&Default::default())?;

        Ok(rendered)
    }
}

impl Mailer {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        smtp_config: MailerSmtpConfig,
        logo_url: Url,
        website_url: Url,
        app_url: Url,
        doc_url: Url,
        privacy_policy_url: Url,
        support_email_address: Address,
        company_legal_name: String,
        company_postal_address: String,
        company_rcs: String,
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
            doc_url,
            privacy_policy_url,
            support_email_address,
            company_legal_name,
            company_postal_address,
            company_rcs,
        })
    }

    /// Build the absolute, Matomo-tagged URL for an in-app destination
    /// given a path (e.g. `/organizations/x/dashboard`). Used by handlers
    /// that need to inject a fully-qualified dashboard URL into a quota
    /// email's `extra_variables`.
    pub fn build_tracked_app_url(&self, mail: &Mail, path: &str) -> String {
        let mut url = self.app_url.clone();
        url.set_path(path);
        with_matomo(&url, mail.matomo_campaign())
    }

    pub async fn send_mail(&self, mail: Mail, recipient: Mailbox) -> Result<(), Hook0Problem> {
        let rendered = mail.render(
            &self.logo_url,
            &self.website_url,
            &self.app_url,
            &self.doc_url,
            &self.privacy_policy_url,
            &self.support_email_address,
            &self.company_legal_name,
            &self.company_postal_address,
            &self.company_rcs,
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
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn fixture() -> (Url, Url, Url, Url, Url, Address, String, String, String) {
        (
            Url::from_str("https://www.hook0.com/mediakit/logo/512x512-banner-transparent.png")
                .unwrap(),
            Url::from_str("https://www.hook0.com").unwrap(),
            Url::from_str("https://app.hook0.com/").unwrap(),
            Url::from_str("https://documentation.hook0.com/").unwrap(),
            Url::from_str("https://www.hook0.com/privacy-policy").unwrap(),
            Address::new("support", "hook0.com").unwrap(),
            "FGRibreau SARL".to_owned(),
            "3 rue de l'Aubépine, 85110 Chantonnay, France".to_owned(),
            "RCS La Roche-sur-Yon 850 824 350".to_owned(),
        )
    }

    fn all_variants() -> Vec<Mail> {
        let verify_url = Url::from_str("https://app.hook0.com/verify-email?token=abc").unwrap();
        let reset_url = Url::from_str("https://app.hook0.com/reset-password?token=xyz").unwrap();
        let mut quota_warning = Mail::QuotaEventsPerDayWarning {
            recipient_first_name: Some("Sarah".to_owned()),
            pricing_url_hash: "#pricing".to_owned(),
            actual_consumption_percent: 80,
            current_events_per_day: 8_000,
            events_per_days_limit: 10_000,
            extra_variables: vec![],
        };
        let mut quota_reached = Mail::QuotaEventsPerDayReached {
            recipient_first_name: Some("Sarah".to_owned()),
            pricing_url_hash: "#pricing".to_owned(),
            current_events_per_day: 10_000,
            events_per_days_limit: 10_000,
            extra_variables: vec![],
        };
        // Mimic the quotas handler injecting the pre-tracked dashboard URL.
        quota_warning.add_variable(
            "dashboard_url_tracked".to_owned(),
            "https://app.hook0.com/organizations/x/dashboard?mtm_source=email&mtm_medium=transactional&mtm_campaign=quota_warning".to_owned(),
        );
        quota_reached.add_variable(
            "dashboard_url_tracked".to_owned(),
            "https://app.hook0.com/organizations/x/dashboard?mtm_source=email&mtm_medium=transactional&mtm_campaign=quota_reached".to_owned(),
        );

        vec![
            Mail::VerifyUserEmail {
                recipient_first_name: Some("Sarah".to_owned()),
                url: verify_url,
            },
            Mail::ResetPassword {
                recipient_first_name: Some("Sarah".to_owned()),
                url: reset_url,
            },
            Mail::Welcome {
                recipient_first_name: Some("Sarah".to_owned()),
            },
            quota_warning,
            quota_reached,
        ]
    }

    fn render(mail: &Mail) -> String {
        let (logo, website, app, doc, privacy, support, legal_name, postal, rcs) = fixture();
        mail.render(
            &logo,
            &website,
            &app,
            &doc,
            &privacy,
            &support,
            &legal_name,
            &postal,
            &rcs,
        )
        .expect("render must succeed")
    }

    /// Test #1 — every Mail variant renders successfully.
    #[test]
    fn mrml_rendering_all_variants_succeed() {
        for m in all_variants() {
            let _ = render(&m);
        }
    }

    /// Test #2 — no unresolved `{ $...}` placeholders leak into output.
    #[test]
    fn no_unresolved_placeholders_in_rendered_html() {
        let pattern = regex::Regex::new(r"\{\s*\$\w+\s*\}").expect("regex must compile");
        for m in all_variants() {
            let html = render(&m);
            assert!(
                !pattern.is_match(&html),
                "Unresolved placeholder in {:?}: {}",
                m.matomo_campaign(),
                pattern.find(&html).map(|m| m.as_str()).unwrap_or("")
            );
        }
    }

    /// Test #3 — the default logo URL must never be the broken LFS pointer
    /// previously served by app.hook0.com.
    #[test]
    fn default_logo_url_is_not_broken_lfs_pointer() {
        let (logo, _, _, _, _, _, _, _, _) = fixture();
        let logo_str = logo.as_str();
        assert!(
            !logo_str.contains("app.hook0.com/256x256.png"),
            "Logo URL must not point to the broken LFS pointer path. Got: {logo_str}"
        );
        assert!(
            logo_str.contains("/mediakit/logo/"),
            "Logo URL must be served from the website mediakit path. Got: {logo_str}"
        );
    }

    /// Test #4 — verify_user_email must not contain the historic "you account" typo.
    #[test]
    fn verify_user_email_does_not_contain_typo_you_account() {
        let mail = Mail::VerifyUserEmail {
            recipient_first_name: Some("Sarah".to_owned()),
            url: Url::from_str("https://app.hook0.com/verify").unwrap(),
        };
        let html = render(&mail);
        assert!(
            !html.contains("you account"),
            "Verify email must not contain the historic 'you account' typo"
        );
    }

    /// Test #5 — reset_password CTA must say 'Reset password', not 'Verify email'.
    #[test]
    fn reset_password_cta_label_is_reset_not_verify() {
        let mail = Mail::ResetPassword {
            recipient_first_name: Some("Sarah".to_owned()),
            url: Url::from_str("https://app.hook0.com/reset").unwrap(),
        };
        let html = render(&mail);
        assert!(
            html.contains("Reset password"),
            "Reset password email must contain CTA label 'Reset password'"
        );
        assert!(
            !html.contains(">Verify email<"),
            "Reset password email must NOT contain CTA label 'Verify email' (anti-regression)"
        );
    }

    /// Test #6 — welcome email contains a CTA to the app and a link to docs.
    #[test]
    fn welcome_email_contains_app_url_cta() {
        let mail = Mail::Welcome {
            recipient_first_name: Some("Sarah".to_owned()),
        };
        let html = render(&mail);
        assert!(
            html.contains("app.hook0.com"),
            "Welcome email must contain a CTA to the app"
        );
        assert!(
            html.contains("documentation.hook0.com"),
            "Welcome email must link to the documentation site"
        );
    }

    /// Test #7 — quota emails contain the support email address.
    #[test]
    fn quota_emails_contain_support_email_address() {
        for m in all_variants() {
            if !m.has_commercial_component() {
                continue;
            }
            let html = render(&m);
            assert!(
                html.contains("support@hook0.com"),
                "Quota email {:?} must contain support email address",
                m.matomo_campaign()
            );
        }
    }

    /// Test #8 — plain text wraps to ≤ 80 columns per line.
    #[test]
    fn plain_text_lines_under_80_cols() {
        for m in all_variants() {
            let html = render(&m);
            let plain = from_read(html.as_bytes(), 80).expect("plain text rendering");
            for line in plain.lines() {
                // chars().count() handles multi-byte unicode correctly (é, à, …).
                assert!(
                    line.chars().count() <= 80,
                    "Plain text line >80 chars in {:?}: {:?}",
                    m.matomo_campaign(),
                    line
                );
            }
        }
    }

    /// Test #9 — the shared footer is present in every variant.
    #[test]
    fn footer_present_in_all_templates() {
        for m in all_variants() {
            let html = render(&m);
            assert!(
                html.contains("FGRibreau SARL"),
                "Footer legal name missing in {:?}",
                m.matomo_campaign()
            );
            assert!(
                html.contains("Privacy"),
                "Footer Privacy link missing in {:?}",
                m.matomo_campaign()
            );
            assert!(
                html.contains("Chantonnay"),
                "Footer postal address missing in {:?}",
                m.matomo_campaign()
            );
        }
    }

    /// Test #10 — the header references the provided logo URL. Anti-regression
    /// against the previous PNG path (`/256x256.png`) which shipped a
    /// baked-in white background and rendered as a sticker on the canvas.
    /// The default in `main.rs` points to the transparent banner variant.
    #[test]
    fn header_logo_uses_provided_logo_url() {
        for m in all_variants() {
            let html = render(&m);
            assert!(
                html.contains("https://www.hook0.com/mediakit/logo/512x512-banner-transparent.png"),
                "Header must reference the provided logo URL in {:?}",
                m.matomo_campaign()
            );
            assert!(
                !html.contains("/mediakit/logo/256x256.png"),
                "Header must NOT reference the legacy white-background logo in {:?}",
                m.matomo_campaign()
            );
        }
    }

    /// Test #11 — recipient_first_name is substituted when provided.
    #[test]
    fn recipient_first_name_substituted_when_provided() {
        for m in all_variants() {
            let html = render(&m);
            assert!(
                html.contains("Sarah"),
                "recipient_first_name 'Sarah' missing in {:?}",
                m.matomo_campaign()
            );
        }
    }

    /// Test #12 — extra_variables injected by handlers (e.g. dashboard_url_tracked)
    /// are substituted in quota templates.
    #[test]
    fn entity_hash_variable_substituted_in_quota() {
        for m in all_variants() {
            if !m.has_commercial_component() {
                continue;
            }
            let html = render(&m);
            assert!(
                html.contains("organizations/x/dashboard"),
                "Pre-tracked dashboard URL must be substituted in {:?}",
                m.matomo_campaign()
            );
        }
    }

    /// Test #13 — every clickable link (non-mailto, non-image) carries the
    /// Matomo tagging trio for the mail's campaign.
    #[test]
    fn all_clickable_links_have_matomo_params() {
        let href_re = regex::Regex::new(r#"href="([^"]+)""#).expect("regex must compile");
        for m in all_variants() {
            let html = render(&m);
            let expected_campaign = m.matomo_campaign();
            for caps in href_re.captures_iter(&html) {
                let url = &caps[1];
                if url.starts_with("mailto:") {
                    continue;
                }
                assert!(
                    url.contains("mtm_source=email"),
                    "Link missing mtm_source in {:?}: {}",
                    expected_campaign,
                    url
                );
                assert!(
                    url.contains("mtm_medium=transactional"),
                    "Link missing mtm_medium in {:?}: {}",
                    expected_campaign,
                    url
                );
                assert!(
                    url.contains(&format!("mtm_campaign={expected_campaign}")),
                    "Link missing mtm_campaign={expected_campaign} in {:?}: {}",
                    expected_campaign,
                    url
                );
            }
        }
    }

    /// Test #14 — with_matomo preserves pre-existing query parameters
    /// (critical for verify/reset URLs that already carry a token).
    #[test]
    fn with_matomo_helper_preserves_existing_query_params() {
        let url = Url::from_str("https://app.hook0.com/verify-email?token=abc123").unwrap();
        let tracked = with_matomo(&url, "verify_email");
        assert!(tracked.contains("token=abc123"), "Token must be preserved");
        assert!(tracked.contains("mtm_source=email"));
        assert!(tracked.contains("mtm_medium=transactional"));
        assert!(tracked.contains("mtm_campaign=verify_email"));
    }

    /// Test #15 — graceful fallback: rendering with `recipient_first_name`
    /// = None must succeed and produce a clean "Hi there, …" greeting
    /// instead of an empty "Hi , …". Required because quota mails are
    /// constructed as a template in handlers/events.rs (without a known
    /// recipient) and may be rendered at any stage of their lifecycle.
    #[test]
    fn render_falls_back_to_generic_greeting_when_recipient_first_name_is_none() {
        let (logo, website, app, doc, privacy, support, legal_name, postal, rcs) = fixture();
        let unhydrated_variants = [
            Mail::QuotaEventsPerDayWarning {
                recipient_first_name: None,
                pricing_url_hash: "#pricing".to_owned(),
                actual_consumption_percent: 80,
                current_events_per_day: 8_000,
                events_per_days_limit: 10_000,
                extra_variables: vec![(
                    "dashboard_url_tracked".to_owned(),
                    "https://app.hook0.com/x?mtm_source=email&mtm_medium=transactional&mtm_campaign=quota_warning".to_owned(),
                )],
            },
            Mail::QuotaEventsPerDayReached {
                recipient_first_name: None,
                pricing_url_hash: "#pricing".to_owned(),
                current_events_per_day: 10_000,
                events_per_days_limit: 10_000,
                extra_variables: vec![(
                    "dashboard_url_tracked".to_owned(),
                    "https://app.hook0.com/x?mtm_source=email&mtm_medium=transactional&mtm_campaign=quota_reached".to_owned(),
                )],
            },
            Mail::Welcome {
                recipient_first_name: None,
            },
            Mail::VerifyUserEmail {
                recipient_first_name: None,
                url: Url::from_str("https://app.hook0.com/verify").unwrap(),
            },
            Mail::ResetPassword {
                recipient_first_name: None,
                url: Url::from_str("https://app.hook0.com/reset").unwrap(),
            },
        ];
        for mail in unhydrated_variants {
            let html = mail
                .render(
                    &logo,
                    &website,
                    &app,
                    &doc,
                    &privacy,
                    &support,
                    &legal_name,
                    &postal,
                    &rcs,
                )
                .expect("render must succeed without recipient_first_name");
            assert!(
                html.contains("there"),
                "Generic 'there' fallback must appear in {:?}",
                mail.matomo_campaign()
            );
            assert!(
                !html.contains("Hi ,") && !html.contains("Hi  "),
                "Greeting must not have an empty name placeholder in {:?}",
                mail.matomo_campaign()
            );
        }
    }

    /// Dumps every rendered template to /tmp/hook0-mails/<slug>.html for
    /// visual review in a browser. Marked #[ignore] so it doesn't run with
    /// the regular `cargo test` suite.
    ///
    /// Usage:
    ///   cargo test -p hook0-api --bin hook0-api mailer::tests::dump_html_to_tmp -- --ignored --nocapture
    ///   open /tmp/hook0-mails/*.html
    #[test]
    #[ignore]
    fn dump_html_to_tmp() {
        let out_dir = std::path::PathBuf::from("/tmp/hook0-mails");
        std::fs::create_dir_all(&out_dir).expect("create /tmp/hook0-mails");

        let (logo, website, app, doc, privacy, support, legal_name, postal, rcs) = fixture();

        for m in all_variants() {
            let html = m
                .render(
                    &logo,
                    &website,
                    &app,
                    &doc,
                    &privacy,
                    &support,
                    &legal_name,
                    &postal,
                    &rcs,
                )
                .expect("render must succeed");
            let slug = m.matomo_campaign();
            let path = out_dir.join(format!("{slug}.html"));
            std::fs::write(&path, html).expect("write html");
            println!("Wrote {}", path.display());
        }

        println!("\nReview in browser:");
        println!("  open /tmp/hook0-mails/*.html");
    }
}
