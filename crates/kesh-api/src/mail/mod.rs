//! Story 17-4b — Couche d'envoi d'email transactionnel (recovery, DC1/DC2).
//!
//! Module **in-`kesh-api`** (pas de crate dédiée, DC1) : un seul type d'email
//! transactionnel (magic-link de reset), couplage fort à `config` + i18n +
//! `AppError`. Une crate `kesh-mail` serait de la sur-ingénierie (cf. le
//! placeholder vide `kesh-payment`).
//!
//! Le trait [`Mailer`] est injecté dans `AppState` sous forme `Arc<dyn Mailer>`,
//! ce qui permet de substituer un [`NoopMailer`] (défaut / feature-off) ou un
//! [`MockMailer`] (tests, capture sans I/O réseau) au [`SmtpMailer`] de prod.
//!
//! **Objet-safe sans `async_trait`** : les méthodes async renvoient une future
//! boxée ([`MailFuture`]) → `Arc<dyn Mailer>` reste dyn-compatible sans
//! dépendance supplémentaire (seul `lettre` est ajouté, DC2).

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use kesh_i18n::Locale;

use crate::errors::AppError;

pub mod smtp;
pub use smtp::SmtpMailer;

/// TTL du lien de réinitialisation, en minutes (DC8). Partagé entre le rendu de
/// l'email (placeholder `{ $ttlMinutes }`) et la génération du token (17-4c) pour
/// éviter toute divergence entre la durée annoncée et la durée réelle.
pub const PASSWORD_RESET_TTL_MINUTES: i64 = 30;

/// Future renvoyée par les méthodes de [`Mailer`] (objet-safe sans `async_trait`).
pub type MailFuture<'a> = Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>>;

/// Pièce jointe d'un [`OutgoingEmail`] (Story 20-3b1 — PDF QR-facture).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAttachment {
    /// Nom de fichier proposé au client mail (ex. `facture-F-2026-0042.pdf`).
    pub filename: String,
    /// Type MIME (ex. `application/pdf`).
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// E-mail métier générique (Story 20-3b1, décision #5 epic-20 : le mailer ne
/// connaît pas le métier facture — futurs consommateurs : rappels #231,
/// factures récurrentes #223).
///
/// Corps **text/plain uniquement** (décision #9 epic-20 — HTML hors scope).
/// L'adresse `From` reste toujours `KESH_SMTP_FROM` ; seul le **display-name**
/// est dynamique (`from_display_name` = nom de la société) — garde L20-1 :
/// jamais de `From` fourni par l'appelant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
    /// Display-name du `From` (nom de la société). `None` = `From` nu.
    pub from_display_name: Option<String>,
    /// `Reply-To` (e-mail de la société). Invalide → omis avec warning
    /// (jamais d'échec d'envoi pour un Reply-To malformé).
    pub reply_to: Option<String>,
    pub attachment: Option<EmailAttachment>,
}

/// Abstraction d'envoi d'email transactionnel (recovery + e-mails métier).
/// `Send + Sync` pour être portée par `AppState` (`Arc<dyn Mailer>`) à
/// travers les handlers Axum.
pub trait Mailer: Send + Sync {
    /// Envoie l'email de réinitialisation de mot de passe (magic-link).
    ///
    /// - `to` : adresse destinataire (email du compte).
    /// - `reset_url` : lien complet `{base}/reset-password?token=...` (token brut).
    /// - `locale` : langue de l'instance (DC10 — pas de locale per-utilisateur).
    ///
    /// Le sujet/corps sont rendus via Fluent (`email-password-reset-subject` /
    /// `email-password-reset-body`). En cas d'échec → [`AppError::SmtpSendFailed`]
    /// (loggée ; côté 17-4c le flux forgot-password est fire-and-forget, l'erreur
    /// n'atteint jamais le client — anti-énumération DC4).
    fn send_password_reset<'a>(
        &'a self,
        to: &'a str,
        reset_url: &'a str,
        locale: Locale,
    ) -> MailFuture<'a>;

    /// Envoie un e-mail métier générique (objet + corps fournis, pièce jointe
    /// optionnelle — Story 20-3b1). Échec → [`AppError::SmtpSendFailed`],
    /// remontée au client par l'appelant (contrairement au recovery
    /// fire-and-forget).
    fn send_email<'a>(&'a self, email: &'a OutgoingEmail) -> MailFuture<'a>;
}

/// Mailer no-op : ne fait rien, renvoie `Ok`. Défaut quand le recovery est
/// désactivé (feature-off) ou dans les tests qui n'exercent pas l'envoi.
#[derive(Debug, Default, Clone)]
pub struct NoopMailer;

impl Mailer for NoopMailer {
    fn send_password_reset<'a>(
        &'a self,
        _to: &'a str,
        _reset_url: &'a str,
        _locale: Locale,
    ) -> MailFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn send_email<'a>(&'a self, _email: &'a OutgoingEmail) -> MailFuture<'a> {
        // Ok(()) silencieux — les endpoints métier DOIVENT garder la garde
        // `smtp_configured()` (412) en amont pour ne jamais marquer un envoi
        // fantôme comme réussi (AC #15.3 Story 20-3b1).
        Box::pin(async { Ok(()) })
    }
}

/// Email capturé par [`MockMailer`] (tests d'intégration 17-4e).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedMail {
    pub to: String,
    pub reset_url: String,
    pub locale: Locale,
}

/// Mailer de test : capture les emails en mémoire, **sans aucune I/O réseau**.
/// Les tests 17-4e muteront `state.mailer = Arc::new(MockMailer::new())` puis
/// inspecteront [`MockMailer::sent`]. [`MockMailer::failing`] simule un SMTP
/// indisponible (AC23-g) pour vérifier que le flux forgot-password reste `200`.
/// E-mail métier capturé par [`MockMailer`] (Story 20-3b1). L'attachment est
/// capturé par (filename, content_type, taille) — pas les bytes complets —
/// suffisant pour asserter la présence et le nommage de la pièce jointe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from_display_name: Option<String>,
    pub reply_to: Option<String>,
    pub attachment_filename: Option<String>,
    pub attachment_content_type: Option<String>,
    pub attachment_size: usize,
}

#[derive(Debug, Default, Clone)]
pub struct MockMailer {
    sent: Arc<Mutex<Vec<CapturedMail>>>,
    /// Buffer séparé pour les e-mails métier (Story 20-3b1) — `CapturedMail`
    /// (recovery) reste intact (33+ call-sites existants).
    sent_emails: Arc<Mutex<Vec<CapturedEmail>>>,
    fail: bool,
}

impl MockMailer {
    /// MockMailer nominal (capture, ne fait jamais échouer l'envoi).
    pub fn new() -> Self {
        Self::default()
    }

    /// MockMailer qui échoue à chaque envoi (test SMTP-down, AC23-g).
    pub fn failing() -> Self {
        Self {
            fail: true,
            ..Self::default()
        }
    }

    /// Emails capturés jusqu'ici (clone du buffer).
    pub fn sent(&self) -> Vec<CapturedMail> {
        self.sent.lock().expect("MockMailer lock poisoned").clone()
    }

    /// E-mails métier capturés jusqu'ici (Story 20-3b1, clone du buffer).
    pub fn sent_emails(&self) -> Vec<CapturedEmail> {
        self.sent_emails
            .lock()
            .expect("MockMailer lock poisoned")
            .clone()
    }
}

impl Mailer for MockMailer {
    fn send_password_reset<'a>(
        &'a self,
        to: &'a str,
        reset_url: &'a str,
        locale: Locale,
    ) -> MailFuture<'a> {
        let captured = CapturedMail {
            to: to.to_string(),
            reset_url: reset_url.to_string(),
            locale,
        };
        let fail = self.fail;
        Box::pin(async move {
            if fail {
                return Err(AppError::SmtpSendFailed(
                    "MockMailer forced failure".to_string(),
                ));
            }
            self.sent
                .lock()
                .expect("MockMailer lock poisoned")
                .push(captured);
            Ok(())
        })
    }

    fn send_email<'a>(&'a self, email: &'a OutgoingEmail) -> MailFuture<'a> {
        let captured = CapturedEmail {
            to: email.to.clone(),
            subject: email.subject.clone(),
            body: email.body.clone(),
            from_display_name: email.from_display_name.clone(),
            reply_to: email.reply_to.clone(),
            attachment_filename: email.attachment.as_ref().map(|a| a.filename.clone()),
            attachment_content_type: email.attachment.as_ref().map(|a| a.content_type.clone()),
            attachment_size: email.attachment.as_ref().map_or(0, |a| a.bytes.len()),
        };
        let fail = self.fail;
        Box::pin(async move {
            if fail {
                return Err(AppError::SmtpSendFailed(
                    "MockMailer forced failure".to_string(),
                ));
            }
            self.sent_emails
                .lock()
                .expect("MockMailer lock poisoned")
                .push(captured);
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn noop_mailer_returns_ok() {
        let m = NoopMailer;
        let r = m
            .send_password_reset("a@b.ch", "https://x/reset?token=t", Locale::FrCh)
            .await;
        assert!(r.is_ok());
    }

    #[tokio::test]
    async fn mock_mailer_captures_sent_mail() {
        let m = MockMailer::new();
        m.send_password_reset("a@b.ch", "https://x/reset?token=tok", Locale::DeCh)
            .await
            .expect("mock send ok");
        let sent = m.sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].to, "a@b.ch");
        assert_eq!(sent[0].reset_url, "https://x/reset?token=tok");
        assert_eq!(sent[0].locale, Locale::DeCh);
    }

    #[tokio::test]
    async fn failing_mock_mailer_errors_and_captures_nothing() {
        let m = MockMailer::failing();
        let r = m
            .send_password_reset("a@b.ch", "https://x/reset?token=t", Locale::FrCh)
            .await;
        assert!(matches!(r, Err(AppError::SmtpSendFailed(_))));
        assert!(m.sent().is_empty());
    }

    // --- Story 20-3b1 : send_email -------------------------------------

    fn sample_email() -> OutgoingEmail {
        OutgoingEmail {
            to: "client@example.ch".to_string(),
            subject: "Facture".to_string(),
            body: "Bonjour".to_string(),
            from_display_name: Some("Ma PME SA".to_string()),
            reply_to: Some("info@mapme.ch".to_string()),
            attachment: Some(EmailAttachment {
                filename: "facture-1.pdf".to_string(),
                content_type: "application/pdf".to_string(),
                bytes: vec![1, 2, 3],
            }),
        }
    }

    #[tokio::test]
    async fn noop_mailer_send_email_returns_ok() {
        assert!(NoopMailer.send_email(&sample_email()).await.is_ok());
    }

    #[tokio::test]
    async fn mock_mailer_captures_sent_business_email() {
        let m = MockMailer::new();
        m.send_email(&sample_email()).await.expect("mock send ok");
        let sent = m.sent_emails();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].to, "client@example.ch");
        assert_eq!(sent[0].subject, "Facture");
        assert_eq!(sent[0].from_display_name.as_deref(), Some("Ma PME SA"));
        assert_eq!(sent[0].reply_to.as_deref(), Some("info@mapme.ch"));
        assert_eq!(
            sent[0].attachment_filename.as_deref(),
            Some("facture-1.pdf")
        );
        assert_eq!(sent[0].attachment_size, 3);
        // Les deux buffers sont indépendants (recovery intact).
        assert!(m.sent().is_empty());
    }

    #[tokio::test]
    async fn failing_mock_mailer_send_email_errors() {
        let m = MockMailer::failing();
        let r = m.send_email(&sample_email()).await;
        assert!(matches!(r, Err(AppError::SmtpSendFailed(_))));
        assert!(m.sent_emails().is_empty());
    }
}
