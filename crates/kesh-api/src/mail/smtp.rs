//! Story 17-4b — `SmtpMailer` : implémentation de [`Mailer`] via `lettre`
//! (rustls, DC2). Construit depuis la config SMTP ; rend le sujet/corps via
//! Fluent (`config.locale`, DC10) et envoie en SMTP STARTTLS (défaut) ou
//! plaintext (`KESH_SMTP_TLS=false`, LAN strict).

use std::sync::Arc;

use kesh_i18n::{FluentArgs, I18nBundle, Locale};
use lettre::message::header::ContentType;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncTransport, Message, Tokio1Executor};

use super::{MailFuture, Mailer};
use crate::config::Config;
use crate::errors::AppError;

/// Mailer de production : envoie les emails transactionnels via SMTP.
///
/// Construit à partir de [`Config`] (vars `KESH_SMTP_*`). Le secret SMTP est
/// copié ici depuis le champ privé `Config::smtp_password()` ; ne jamais le
/// logger. Le TTL du lien (en minutes) est injecté dans le corps de l'email
/// via le placeholder `{ $ttlMinutes }` (17-4c fixera la valeur, 30 min DC8).
pub struct SmtpMailer {
    i18n: Arc<I18nBundle>,
    from: String,
    host: String,
    port: u16,
    user: String,
    password: String,
    tls: bool,
    ttl_minutes: i64,
}

impl SmtpMailer {
    /// Construit un `SmtpMailer` depuis la config. Retourne `None` si une des
    /// vars SMTP requises est absente — défensif : si le feature recovery est
    /// activé, le boot fail-fast (`ConfigError::IncompleteSmtpConfig`) a déjà
    /// garanti leur présence ; `main.rs` n'instancie ce mailer que dans ce cas.
    pub fn from_config(config: &Config, i18n: Arc<I18nBundle>, ttl_minutes: i64) -> Option<Self> {
        Some(Self {
            i18n,
            from: config.smtp_from.clone()?,
            host: config.smtp_host.clone()?,
            port: config.smtp_port,
            user: config.smtp_user.clone()?,
            password: config.smtp_password()?.to_string(),
            tls: config.smtp_tls,
            ttl_minutes,
        })
    }
}

impl Mailer for SmtpMailer {
    fn send_password_reset<'a>(
        &'a self,
        to: &'a str,
        reset_url: &'a str,
        locale: Locale,
    ) -> MailFuture<'a> {
        Box::pin(async move {
            // Rendu i18n du sujet/corps (DC10 — langue de l'instance).
            let subject = self
                .i18n
                .format(&locale, "email-password-reset-subject", None);
            let mut args = FluentArgs::new();
            args.set("resetUrl", reset_url);
            args.set("ttlMinutes", self.ttl_minutes);
            let body = self
                .i18n
                .format(&locale, "email-password-reset-body", Some(&args));

            let email = Message::builder()
                .from(
                    self.from
                        .parse()
                        .map_err(|e| AppError::SmtpSendFailed(format!("from invalide: {e}")))?,
                )
                .to(to
                    .parse()
                    .map_err(|e| AppError::SmtpSendFailed(format!("to invalide: {e}")))?)
                .subject(subject)
                .header(ContentType::TEXT_PLAIN)
                .body(body)
                .map_err(|e| AppError::SmtpSendFailed(format!("build message: {e}")))?;

            let creds = Credentials::new(self.user.clone(), self.password.clone());
            let builder = if self.tls {
                // STARTTLS submission (défaut, port 587).
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.host)
                    .map_err(|e| AppError::SmtpSendFailed(format!("starttls relay: {e}")))?
            } else {
                // Plaintext (LAN strict, KESH_SMTP_TLS=false) — la doc 17-4f
                // déconseille (token brut en clair dans l'email).
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.host)
            };
            let transport = builder.port(self.port).credentials(creds).build();

            transport
                .send(email)
                .await
                .map_err(|e| AppError::SmtpSendFailed(format!("send: {e}")))?;
            Ok(())
        })
    }
}
