//! Story 17-4b — `SmtpMailer` : implémentation de [`Mailer`] via `lettre`
//! (rustls, DC2). Construit depuis la config SMTP ; rend le sujet/corps via
//! Fluent (`config.locale`, DC10) et envoie en SMTP STARTTLS (défaut) ou
//! plaintext (`KESH_SMTP_TLS=false`, LAN strict).

use std::sync::Arc;

use kesh_i18n::{FluentArgs, I18nBundle, Locale};
use lettre::message::header::ContentType;
use lettre::message::{Attachment, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncTransport, Message, Tokio1Executor};

use super::{MailFuture, Mailer, OutgoingEmail};
use crate::config::Config;
use crate::errors::AppError;

/// Mailer de production : envoie les emails transactionnels via SMTP.
///
/// Construit à partir de [`Config`] (vars `KESH_SMTP_*`). Le transport `lettre`
/// (relay + paramètres TLS + credentials) et la [`Mailbox`] expéditrice sont
/// construits **une seule fois** ici — pas à chaque envoi — et le secret SMTP
/// n'est **pas** conservé dans la struct (il vit uniquement dans les
/// `Credentials` du transport, qui n'expose pas de `Debug` le divulguant).
/// Parser le `from` en `Mailbox` au boot aligne le fail-fast sur le parser
/// RFC réellement utilisé à l'envoi (review Pass 3 : `is_valid_email_simple`
/// seul laissait passer des adresses que lettre refuserait à chaque envoi —
/// recovery silencieusement cassé). Le TTL du lien (en minutes) est injecté
/// dans le corps via le placeholder `{ $ttlMinutes }`.
pub struct SmtpMailer {
    i18n: Arc<I18nBundle>,
    from: Mailbox,
    transport: AsyncSmtpTransport<Tokio1Executor>,
    ttl_minutes: i64,
}

impl SmtpMailer {
    /// Construit un `SmtpMailer` depuis la config. Retourne `Err(detail)` si
    /// une var SMTP requise est absente, si `KESH_SMTP_FROM` n'est pas une
    /// adresse acceptée par lettre, ou si l'initialisation du transport
    /// STARTTLS échoue — défensif : si le feature recovery est activé, le boot
    /// fail-fast (`ConfigError::IncompleteSmtpConfig`) a déjà garanti la
    /// présence des vars ; `main.rs` n'instancie ce mailer que dans ce cas et
    /// loggue + `exit(1)` sur `Err` (même pattern que les autres erreurs boot).
    pub fn from_config(
        config: &Config,
        i18n: Arc<I18nBundle>,
        ttl_minutes: i64,
    ) -> Result<Self, String> {
        let from: Mailbox = config
            .smtp_from
            .as_deref()
            .ok_or("KESH_SMTP_FROM absente au build du mailer")?
            .parse()
            .map_err(|e| format!("KESH_SMTP_FROM refusée par lettre: {e}"))?;
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
            // déconseille (token brut en clair dans l'email + AUTH SMTP en
            // clair sur le réseau ; main.rs loggue un warn boot explicite).
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

    /// Construit le `Message` lettre (from/to/sujet/corps `text/plain`) avec
    /// rendu Fluent. Factorisé hors de `send_password_reset` pour être
    /// testable sans transport ni I/O réseau (review Pass 3 : le rendu réel —
    /// placeholders, ContentType, erreurs de parse `to` — n'était couvert par
    /// aucun test, MockMailer court-circuitant ce module).
    fn build_message(
        &self,
        to: &str,
        reset_url: &str,
        locale: Locale,
    ) -> Result<Message, AppError> {
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

        Message::builder()
            .from(self.from.clone())
            .to(to
                .parse()
                .map_err(|e| AppError::SmtpSendFailed(format!("to invalide: {e}")))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)
            .map_err(|e| AppError::SmtpSendFailed(format!("build message: {e}")))
    }

    /// Construit le `Message` lettre d'un e-mail métier (Story 20-3b1) :
    /// `From` = adresse `KESH_SMTP_FROM` avec display-name dynamique (nom de
    /// la société — décision #2 epic-20, l'adresse elle-même n'est jamais
    /// fournie par l'appelant), `Reply-To` tolérant (invalide → omis +
    /// warning), corps `text/plain`, pièce jointe optionnelle en
    /// `MultiPart::mixed`. Factorisé hors de `send_email` pour être testable
    /// sans transport ni I/O réseau (pattern `build_message`).
    fn build_outgoing_message(&self, email: &OutgoingEmail) -> Result<Message, AppError> {
        // Display-name dynamique : même adresse que KESH_SMTP_FROM, seul le
        // nom d'affichage change. Le builder lettre encode le display-name
        // (RFC 2047) — anti-injection CRLF préservée.
        let from = match &email.from_display_name {
            Some(name) => Mailbox::new(Some(name.clone()), self.from.email.clone()),
            None => self.from.clone(),
        };

        let mut builder = Message::builder()
            .from(from)
            .to(email
                .to
                .parse()
                .map_err(|e| AppError::SmtpSendFailed(format!("to invalide: {e}")))?)
            .subject(email.subject.clone());

        if let Some(reply_to) = &email.reply_to {
            match reply_to.parse::<Mailbox>() {
                Ok(mb) => builder = builder.reply_to(mb),
                Err(e) => {
                    // Jamais d'échec d'envoi pour un Reply-To malformé
                    // (AC #8 Story 20-3b1) — l'e-mail part sans Reply-To.
                    tracing::warn!(reply_to = %reply_to, "Reply-To invalide, omis: {e}");
                }
            }
        }

        match &email.attachment {
            Some(att) => {
                let content_type = ContentType::parse(&att.content_type).map_err(|e| {
                    AppError::SmtpSendFailed(format!(
                        "content-type attachment invalide ({}): {e}",
                        att.content_type
                    ))
                })?;
                builder
                    .multipart(
                        MultiPart::mixed()
                            .singlepart(SinglePart::plain(email.body.clone()))
                            .singlepart(
                                Attachment::new(att.filename.clone())
                                    .body(att.bytes.clone(), content_type),
                            ),
                    )
                    .map_err(|e| AppError::SmtpSendFailed(format!("build message: {e}")))
            }
            None => builder
                .header(ContentType::TEXT_PLAIN)
                .body(email.body.clone())
                .map_err(|e| AppError::SmtpSendFailed(format!("build message: {e}"))),
        }
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
            let email = self.build_message(to, reset_url, locale)?;
            self.transport
                .send(email)
                .await
                .map_err(|e| AppError::SmtpSendFailed(format!("send: {e}")))?;
            Ok(())
        })
    }

    fn send_email<'a>(&'a self, email: &'a OutgoingEmail) -> MailFuture<'a> {
        Box::pin(async move {
            let message = self.build_outgoing_message(email)?;
            self.transport
                .send(message)
                .await
                .map_err(|e| AppError::SmtpSendFailed(format!("send: {e}")))?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SmtpMailer` de test : transport plaintext jamais utilisé (les tests
    /// n'appellent que `build_message`, pas `send`). Champs accessibles ici
    /// (même module).
    fn test_mailer() -> SmtpMailer {
        let i18n = kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("locales chargées");
        SmtpMailer {
            i18n: Arc::new(i18n),
            from: "noreply@example.com".parse().expect("from de test valide"),
            transport: AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("localhost").build(),
            ttl_minutes: 30,
        }
    }

    /// Le corps rendu contient l'URL de reset, le TTL substitué et le sujet
    /// est non-vide — détecte une régression de placeholder ou de clé FTL.
    ///
    /// NB (review Pass 4) : URLs de test volontairement < 75 bytes — au-delà,
    /// l'encodage quoted-printable de lettre insère un soft line break `=\r\n`
    /// qui couperait l'URL dans le MIME brut et ferait échouer `contains()`
    /// (le lien resterait fonctionnel une fois décodé). Pour des tokens longs
    /// (17-4e), asserter sur `CapturedMail.reset_url` du MockMailer, pas sur
    /// `msg.formatted()`.
    #[test]
    fn build_message_renders_url_and_ttl() {
        let mailer = test_mailer();
        let msg = mailer
            .build_message(
                "user@example.ch",
                "https://kesh.example.com/reset-password?token=tok123",
                Locale::FrCh,
            )
            .expect("message construit");
        let raw = String::from_utf8(msg.formatted()).expect("message UTF-8");
        assert!(
            raw.contains("tok123"),
            "le corps doit contenir l'URL de reset (token)"
        );
        assert!(raw.contains("30"), "le TTL doit être substitué");
        assert!(
            raw.contains("Subject:"),
            "le sujet doit être présent dans le message"
        );
        assert!(raw.contains("text/plain"), "ContentType text/plain attendu");
    }

    /// Les 4 locales rendent un corps contenant l'URL (parité de placeholder
    /// au niveau du rendu réel, pas seulement de la présence de la clé).
    #[test]
    fn build_message_all_locales_render_url() {
        let mailer = test_mailer();
        for locale in [Locale::FrCh, Locale::DeCh, Locale::ItCh, Locale::EnCh] {
            let msg = mailer
                .build_message("user@example.ch", "https://x/reset?token=t42", locale)
                .expect("message construit");
            let raw = String::from_utf8(msg.formatted()).expect("UTF-8");
            assert!(
                raw.contains("t42"),
                "locale {locale:?} : URL absente du corps rendu"
            );
        }
    }

    /// `to` non parsable → `SmtpSendFailed` (pas de panic, pas d'envoi).
    #[test]
    fn build_message_invalid_to_errors() {
        let mailer = test_mailer();
        let r = mailer.build_message("pas un email", "https://x/r?token=t", Locale::FrCh);
        assert!(matches!(r, Err(AppError::SmtpSendFailed(_))));
    }

    // --- Story 20-3b1 : build_outgoing_message (e-mail métier) --------------

    fn sample_outgoing(attachment: bool, reply_to: Option<&str>) -> super::super::OutgoingEmail {
        super::super::OutgoingEmail {
            to: "client@example.ch".to_string(),
            subject: "Facture F-2026-0042".to_string(),
            body: "Bonjour,\nvoici la facture.".to_string(),
            from_display_name: Some("Ma PME SA".to_string()),
            reply_to: reply_to.map(String::from),
            attachment: attachment.then(|| super::super::EmailAttachment {
                filename: "facture-F-2026-0042.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                bytes: b"%PDF-1.4 stub".to_vec(),
            }),
        }
    }

    /// Multipart : corps + attachment présents, display-name dans le From,
    /// Reply-To présent.
    #[test]
    fn build_outgoing_message_multipart_with_attachment() {
        let mailer = test_mailer();
        let msg = mailer
            .build_outgoing_message(&sample_outgoing(true, Some("info@mapme.ch")))
            .expect("message construit");
        let raw = String::from_utf8(msg.formatted()).expect("UTF-8");
        assert!(raw.contains("multipart/mixed"), "MultiPart::mixed attendu");
        assert!(
            raw.contains("facture-F-2026-0042.pdf"),
            "filename attachment présent"
        );
        assert!(raw.contains("application/pdf"), "content-type attachment");
        assert!(
            raw.contains("Ma PME SA"),
            "display-name société dans le From"
        );
        assert!(
            raw.contains("noreply@example.com"),
            "adresse From = KESH_SMTP_FROM (jamais fournie par l'appelant)"
        );
        assert!(raw.contains("Reply-To"), "Reply-To présent");
        assert!(raw.contains("info@mapme.ch"), "adresse Reply-To");
    }

    /// Sans attachment : text/plain simple. Sans Reply-To : header absent.
    #[test]
    fn build_outgoing_message_plain_without_attachment() {
        let mailer = test_mailer();
        let msg = mailer
            .build_outgoing_message(&sample_outgoing(false, None))
            .expect("message construit");
        let raw = String::from_utf8(msg.formatted()).expect("UTF-8");
        assert!(!raw.contains("multipart/mixed"), "pas de multipart");
        assert!(raw.contains("text/plain"), "corps text/plain");
        assert!(!raw.contains("Reply-To"), "pas de Reply-To");
    }

    /// Reply-To invalide → omis (warning), l'envoi n'échoue PAS (AC #8).
    #[test]
    fn build_outgoing_message_invalid_reply_to_is_omitted() {
        let mailer = test_mailer();
        let msg = mailer
            .build_outgoing_message(&sample_outgoing(false, Some("pas une adresse")))
            .expect("message construit malgré Reply-To invalide");
        let raw = String::from_utf8(msg.formatted()).expect("UTF-8");
        assert!(!raw.contains("Reply-To"), "Reply-To invalide omis");
    }

    /// `to` invalide → SmtpSendFailed (parité build_message).
    #[test]
    fn build_outgoing_message_invalid_to_errors() {
        let mailer = test_mailer();
        let mut email = sample_outgoing(false, None);
        email.to = "pas un email".to_string();
        let r = mailer.build_outgoing_message(&email);
        assert!(matches!(r, Err(AppError::SmtpSendFailed(_))));
    }
}
