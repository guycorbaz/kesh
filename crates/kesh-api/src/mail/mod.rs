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

/// Abstraction d'envoi d'email transactionnel (recovery). `Send + Sync` pour
/// être portée par `AppState` (`Arc<dyn Mailer>`) à travers les handlers Axum.
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
#[derive(Debug, Default, Clone)]
pub struct MockMailer {
    sent: Arc<Mutex<Vec<CapturedMail>>>,
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
            sent: Arc::new(Mutex::new(Vec::new())),
            fail: true,
        }
    }

    /// Emails capturés jusqu'ici (clone du buffer).
    pub fn sent(&self) -> Vec<CapturedMail> {
        self.sent.lock().expect("MockMailer lock poisoned").clone()
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
}
