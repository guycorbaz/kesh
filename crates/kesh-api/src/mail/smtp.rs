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
/// Construit à partir de [`Config`] (vars `KESH_SMTP_*`). Le transport `lettre`
/// (résolution relay + paramètres TLS + credentials) est construit **une seule
/// fois** ici — pas à chaque envoi — et le secret SMTP n'est **pas** conservé
/// dans la struct (il vit uniquement dans les `Credentials` du transport, qui
/// n'expose pas de `Debug` le divulguant). Le TTL du lien (en minutes) est
/// injecté dans le corps de l'email via le placeholder `{ $ttlMinutes }`.
pub struct SmtpMailer {
    i18n: Arc<I18nBundle>,
    from: String,
    transport: AsyncSmtpTransport<Tokio1Executor>,
    ttl_minutes: i64,
}

impl SmtpMailer {
    /// Construit un `SmtpMailer` depuis la config. Retourne `Err(detail)` si
    /// une var SMTP requise est absente ou si l'initialisation du transport
    /// STARTTLS échoue — défensif : si le feature recovery est activé, le boot
    /// fail-fast (`ConfigError::IncompleteSmtpConfig`) a déjà garanti la
    /// présence des vars ; `main.rs` n'instancie ce mailer que dans ce cas et
    /// loggue + `exit(1)` sur `Err` (même pattern que les autres erreurs boot).
    pub fn from_config(
        config: &Config,
        i18n: Arc<I18nBundle>,
        ttl_minutes: i64,
    ) -> Result<Self, String> {
        let from = config
            .smtp_from
            .clone()
            .ok_or("KESH_SMTP_FROM absente au build du mailer")?;
        let host = config
            .smtp_host
            .clone()
            .ok_or("KESH_SMTP_HOST absente au build du mailer")?;
        let user = config
            .smtp_user
            .clone()
            .ok_or("KESH_SMTP_USER absente au build du mailer")?;
        let password = config
            .smtp_password()
            .ok_or("KESH_SMTP_PASSWORD absente au build du mailer")?
            .to_string();

        let creds = Credentials::new(user, password);
        let builder = if config.smtp_tls {
            // STARTTLS submission (défaut, port 587).
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
                .map_err(|e| format!("init STARTTLS ({host}): {e}"))?
        } else {
            // Plaintext (LAN strict, KESH_SMTP_TLS=false) — la doc 17-4f
            // déconseille (token brut en clair dans l'email).
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&host)
        };
        let transport = builder.port(config.smtp_port).credentials(creds).build();

        Ok(Self {
            i18n,
            from,
            transport,
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

            self.transport
                .send(email)
                .await
                .map_err(|e| AppError::SmtpSendFailed(format!("send: {e}")))?;
            Ok(())
        })
    }
}
