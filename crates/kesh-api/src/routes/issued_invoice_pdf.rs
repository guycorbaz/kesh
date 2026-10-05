//! Story 25-6-b (#387) — le PDF **émis** d'une facture : figé au premier rendu
//! après validation, puis relu à l'identique.
//!
//! Ce module est le **seul** chemin vers le PDF d'une facture, pour le
//! téléchargement comme pour l'envoi par e-mail. Il encadre
//! [`invoice_pdf_service::render_document`], qui reste la fonction de rendu
//! (préconditions et codes d'erreur inchangés) :
//!
//! - facture **figée** → relit le fichier sous `KESH_DOCUMENTS_DIR`, vérifie
//!   son SHA-256 contre `pdf_sha256`, rend ces octets — y compris pour une
//!   facture annulée par un avoir après le gel ;
//! - facture `validated` **non figée** → rend dans la langue du **client**,
//!   écrit le fichier, puis pose les quatre colonnes et l'audit
//!   `invoice.pdf_frozen` dans une transaction (`invoices::freeze_pdf`).
//!
//! ⛔ Un PDF figé introuvable n'est **jamais** régénéré en silence : ce serait
//! fabriquer une pièce qui n'a pas été émise. Il répond 410, et seul le
//! refigeage explicite d'un administrateur ([`refreeze`]) produit un nouveau
//! document, tracé avec les deux empreintes.
//!
//! Les rendus ne sont pas reproductibles (date de création et `/ID` aléatoire
//! du PDF) : deux rendus concurrents produisent deux fichiers distincts. Celui
//! dont la pose échoue reste orphelin sur le disque — inoffensif, jamais servi.

use std::path::{Path, PathBuf};

use kesh_db::entities::{Company, Invoice, Language};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, invoices};
use sha2::{Digest, Sha256};

use crate::document_storage::{self, ReadDocumentError};
use crate::errors::AppError;
use crate::routes::invoice_email::resolve_language;
use crate::routes::invoice_pdf_service::{self, PdfDocument};

/// Ce dont le service a besoin, et **qui** agit — l'audit du gel nomme
/// l'acteur, quel que soit son rôle (arbitrage 8 : une pièce sortie de Kesh
/// est figée, qui que ce soit qui l'ait fait sortir).
pub struct PdfContext<'a> {
    pub pool: &'a sqlx::MySqlPool,
    pub i18n: &'a kesh_i18n::I18nBundle,
    pub documents_dir: &'a Path,
    /// Société **déjà autorisée** pour l'acteur (`get_company_for`) : le
    /// service ne re-vérifie pas ce droit, il scope chaque lecture par elle.
    pub company: &'a Company,
    pub user_id: i64,
    pub actor_api_key_id: Option<i64>,
}

/// Le PDF émis d'une facture.
#[derive(Debug)]
pub struct IssuedPdf {
    pub bytes: Vec<u8>,
    /// Nom de fichier assaini, sans extension (cf. `RenderedInvoicePdf`).
    pub filename_base: String,
}

/// Issue d'une tentative de pose ([`pose`]).
#[derive(Debug)]
pub enum PoseOutcome {
    /// La pose a eu lieu : ces octets sont désormais le document émis.
    Posed(IssuedPdf),
    /// Un rendu concurrent a figé avant nous : c'est **son** document qui est
    /// servi, jamais le nôtre.
    Adopted(IssuedPdf),
    /// La facture a changé de version pendant le rendu : à rendre de nouveau.
    Changed,
}

/// Le PDF émis de la facture `invoice_id` — le figeant s'il ne l'est pas.
///
/// # Errors
/// - les erreurs de rendu inchangées (`InvoiceNotValidated` pour un brouillon
///   ou une facture annulée **jamais** figée, préconditions du PDF, …) ;
/// - [`AppError::InvoicePdfGone`] (410) si le fichier figé manque ;
/// - `Internal` (500) si son empreinte ne correspond plus, ou si le fichier
///   ne peut pas être écrit — jamais un rendu non figé présenté comme émis ;
/// - [`AppError::InvoiceChanged`] (409) si la facture change deux fois de
///   suite pendant le rendu ; [`AppError::InvoiceCancelled`] si un avoir
///   l'annule pendant le rendu.
pub async fn get_or_freeze(ctx: &PdfContext<'_>, invoice_id: i64) -> Result<IssuedPdf, AppError> {
    let invoice = load(ctx, invoice_id).await?;
    if is_frozen(&invoice) {
        return serve_frozen(ctx, &invoice).await;
    }
    if invoice.status != "validated" {
        return Err(AppError::InvoiceNotValidated);
    }

    with_one_retry(|| render_and_pose(ctx, &invoice)).await
}

/// Une seule nouvelle tentative : une pause des rappels, un règlement ou un
/// avoir concurrents dans la seconde du rendu la déclenchent ; une course qui
/// se répète répond 409 `INVOICE_CHANGED` plutôt que de boucler.
async fn with_one_retry<F, Fut>(mut attempt: F) -> Result<IssuedPdf, AppError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<PoseOutcome, AppError>>,
{
    for _ in 0..2 {
        match attempt().await? {
            PoseOutcome::Posed(pdf) | PoseOutcome::Adopted(pdf) => return Ok(pdf),
            PoseOutcome::Changed => continue,
        }
    }
    Err(AppError::InvoiceChanged)
}

/// Un rendu dans la langue du client, puis sa pose.
async fn render_and_pose(ctx: &PdfContext<'_>, invoice: &Invoice) -> Result<PoseOutcome, AppError> {
    let language = client_language(ctx, invoice).await?;
    let rendered = invoice_pdf_service::render_document(
        ctx.pool,
        ctx.i18n,
        kesh_i18n::Locale::from(language.as_str()),
        ctx.company,
        invoice.id,
        PdfDocument::Invoice,
    )
    .await?;
    pose(
        ctx,
        invoice.id,
        rendered.invoice_version,
        IssuedPdf {
            bytes: rendered.bytes,
            filename_base: rendered.filename_base,
        },
        language,
    )
    .await
}

/// Écrit `rendered` sous `KESH_DOCUMENTS_DIR` et le pose comme document émis
/// de la facture, **si** elle est toujours `validated`, non figée, et à la
/// version `rendered_version` lue au rendu.
///
/// Exposé pour les tests, qui y intercalent une dévalidation ou un gel
/// concurrent de façon déterministe ; le seul appelant applicatif est
/// [`get_or_freeze`].
#[doc(hidden)]
pub async fn pose(
    ctx: &PdfContext<'_>,
    invoice_id: i64,
    rendered_version: i32,
    rendered: IssuedPdf,
    language: Language,
) -> Result<PoseOutcome, AppError> {
    let (frozen, rendered) = store(ctx.documents_dir, rendered, language).await?;
    let applied = invoices::freeze_pdf(
        ctx.pool,
        ctx.company.id,
        invoice_id,
        rendered_version,
        &frozen,
        ctx.user_id,
        ctx.actor_api_key_id,
    )
    .await?;
    if applied {
        tracing::info!(
            company_id = ctx.company.id,
            invoice_id,
            pdf_sha256 = %frozen.sha256,
            language = %frozen.language,
            "PDF de facture figé"
        );
        return Ok(PoseOutcome::Posed(rendered));
    }

    // Zéro ligne : relire pour savoir laquelle des trois gardes a refusé.
    let after = load(ctx, invoice_id).await?;
    if is_frozen(&after) {
        return Ok(PoseOutcome::Adopted(serve_frozen(ctx, &after).await?));
    }
    match after.status.as_str() {
        "validated" => Ok(PoseOutcome::Changed),
        "cancelled" => Err(AppError::InvoiceCancelled),
        _ => Err(AppError::InvoiceNotValidated),
    }
}

/// Refige le PDF d'une facture dont le fichier a disparu — geste
/// d'administrateur (arbitrage 7). Rend un **nouveau** document (langue
/// actuelle du client, données du moment) : ce n'est pas l'original, et
/// l'audit `invoice.pdf_refrozen` garde les deux empreintes.
///
/// # Errors
/// - [`AppError::InvoicePdfNotFrozen`] : rien à refiger, le prochain rendu figera ;
/// - [`AppError::InvoicePdfPresent`] : le fichier existe, on ne le remplace pas ;
/// - [`AppError::InvoicePdfIntegrity`] : le fichier existe mais est altéré ;
/// - [`AppError::InvoiceCancelled`] : une facture annulée ne se rend plus ;
/// - les erreurs de rendu inchangées.
pub async fn refreeze(ctx: &PdfContext<'_>, invoice_id: i64) -> Result<IssuedPdf, AppError> {
    let invoice = load(ctx, invoice_id).await?;
    let (Some(storage_path), Some(old_sha256)) = (
        invoice.pdf_storage_path.as_deref(),
        invoice.pdf_sha256.clone(),
    ) else {
        return Err(AppError::InvoicePdfNotFrozen);
    };
    match read_checked(ctx.documents_dir, storage_path, &old_sha256).await? {
        FrozenFile::Intact(_) => return Err(AppError::InvoicePdfPresent),
        FrozenFile::Altered => return Err(AppError::InvoicePdfIntegrity),
        FrozenFile::Missing => {}
    }
    if invoice.status == "cancelled" {
        return Err(AppError::InvoiceCancelled);
    }

    let language = client_language(ctx, &invoice).await?;
    let rendered = invoice_pdf_service::render_document(
        ctx.pool,
        ctx.i18n,
        kesh_i18n::Locale::from(language.as_str()),
        ctx.company,
        invoice_id,
        PdfDocument::Invoice,
    )
    .await?;
    let (frozen, rendered) = store(
        ctx.documents_dir,
        IssuedPdf {
            bytes: rendered.bytes,
            filename_base: rendered.filename_base,
        },
        language,
    )
    .await?;
    let applied = invoices::refreeze_pdf(
        ctx.pool,
        ctx.company.id,
        invoice_id,
        &old_sha256,
        &frozen,
        ctx.user_id,
        ctx.actor_api_key_id,
    )
    .await?;
    if applied {
        tracing::warn!(
            company_id = ctx.company.id,
            invoice_id,
            old_pdf_sha256 = %old_sha256,
            pdf_sha256 = %frozen.sha256,
            "PDF de facture refigé — nouveau document, l'original est perdu"
        );
        return Ok(rendered);
    }

    // Refusé : un avoir intercalé, ou un refigeage concurrent qui a gagné.
    let after = load(ctx, invoice_id).await?;
    if after.status == "cancelled" {
        Err(AppError::InvoiceCancelled)
    } else if after.pdf_sha256.as_deref() != Some(old_sha256.as_str()) {
        Err(AppError::InvoicePdfPresent)
    } else {
        Err(AppError::InvoiceChanged)
    }
}

/// `true` si la facture porte un PDF figé (les quatre colonnes vont ensemble,
/// CHECK `chk_invoices_frozen_pdf`).
fn is_frozen(invoice: &Invoice) -> bool {
    invoice.pdf_storage_path.is_some()
}

async fn load(ctx: &PdfContext<'_>, invoice_id: i64) -> Result<Invoice, AppError> {
    invoices::find_by_id(ctx.pool, ctx.company.id, invoice_id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))
}

/// La langue du client, sinon celle de l'installation — la règle de l'e-mail
/// (arbitrage 2). Un contact introuvable laisse `render_document` produire son
/// erreur habituelle : on rend alors dans la langue de l'installation.
async fn client_language(ctx: &PdfContext<'_>, invoice: &Invoice) -> Result<Language, AppError> {
    Ok(
        match contacts::find_by_id(ctx.pool, invoice.contact_id).await? {
            Some(contact) => resolve_language(&contact, ctx.company),
            None => ctx.company.instance_language,
        },
    )
}

/// Sert le document figé, après en avoir vérifié l'empreinte.
async fn serve_frozen(ctx: &PdfContext<'_>, invoice: &Invoice) -> Result<IssuedPdf, AppError> {
    let (Some(storage_path), Some(sha256)) = (
        invoice.pdf_storage_path.as_deref(),
        invoice.pdf_sha256.as_deref(),
    ) else {
        return Err(AppError::Internal(format!(
            "facture {} : PDF figé incomplet",
            invoice.id
        )));
    };
    match read_checked(ctx.documents_dir, storage_path, sha256).await? {
        FrozenFile::Intact(bytes) => Ok(IssuedPdf {
            bytes,
            filename_base: invoice_pdf_service::sanitize_filename(
                invoice.invoice_number.as_deref().unwrap_or("facture"),
            ),
        }),
        FrozenFile::Missing => Err(AppError::InvoicePdfGone(sha256.to_string())),
        FrozenFile::Altered => {
            tracing::error!(
                company_id = ctx.company.id,
                invoice_id = invoice.id,
                storage_path,
                expected_sha256 = sha256,
                "PDF de facture figé ALTÉRÉ — empreinte différente, document non servi"
            );
            Err(AppError::Internal(format!(
                "facture {} : le PDF figé ne correspond plus à son empreinte",
                invoice.id
            )))
        }
    }
}

/// État du fichier d'un PDF figé.
enum FrozenFile {
    Intact(Vec<u8>),
    Missing,
    Altered,
}

/// Lit le fichier figé et compare son SHA-256 à `expected_sha256`.
async fn read_checked(
    documents_dir: &Path,
    storage_path: &str,
    expected_sha256: &str,
) -> Result<FrozenFile, AppError> {
    let dir: PathBuf = documents_dir.to_path_buf();
    let path = storage_path.to_string();
    let read = tokio::task::spawn_blocking(move || document_storage::read_document(&dir, &path))
        .await
        .map_err(|e| AppError::Internal(format!("lecture du PDF figé : {e}")))?;
    let bytes = match read {
        Ok(bytes) => bytes,
        Err(ReadDocumentError::NotFound) => return Ok(FrozenFile::Missing),
        Err(e @ (ReadDocumentError::InvalidPath | ReadDocumentError::Io(_))) => {
            return Err(AppError::Internal(format!(
                "lecture du PDF figé {storage_path} : {e}"
            )));
        }
    };
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual == expected_sha256 {
        Ok(FrozenFile::Intact(bytes))
    } else {
        Ok(FrozenFile::Altered)
    }
}

/// Écrit le rendu sous `KESH_DOCUMENTS_DIR` (`store_document` est synchrone et
/// fait `fsync` : hors du runtime). Un échec d'écriture est un 500 — rien
/// n'est servi.
async fn store(
    documents_dir: &Path,
    rendered: IssuedPdf,
    language: Language,
) -> Result<(invoices::FrozenPdf, IssuedPdf), AppError> {
    let dir = documents_dir.to_path_buf();
    let (meta, rendered) = tokio::task::spawn_blocking(move || {
        let original = format!("facture-{}.pdf", rendered.filename_base);
        document_storage::store_document(&dir, &rendered.bytes, "pdf", &original, "application/pdf")
            .map(|meta| (meta, rendered))
    })
    .await
    .map_err(|e| AppError::Internal(format!("écriture du PDF figé : {e}")))?
    .map_err(|e| {
        tracing::error!(error = %e, "écriture du PDF figé impossible (KESH_DOCUMENTS_DIR)");
        AppError::Internal(format!("écriture du PDF figé : {e}"))
    })?;
    Ok((
        invoices::FrozenPdf {
            storage_path: meta.storage_path,
            sha256: meta.sha256,
            language: language.as_str().to_string(),
        },
        rendered,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pdf(tag: u8) -> IssuedPdf {
        IssuedPdf {
            bytes: vec![tag],
            filename_base: "F-1".into(),
        }
    }

    /// Une course qui se répète → 409, après exactement deux tentatives
    /// (mutation : boucle sans borne, ou sans nouvelle tentative).
    #[tokio::test]
    async fn une_course_qui_se_repete_repond_invoice_changed() {
        let mut calls = 0;
        let res = with_one_retry(|| {
            calls += 1;
            async { Ok(PoseOutcome::Changed) }
        })
        .await;
        assert!(matches!(res, Err(AppError::InvoiceChanged)), "{res:?}");
        assert_eq!(calls, 2);
    }

    /// Une seule course → la nouvelle tentative pose (mutation : aucune
    /// nouvelle tentative, 409 dès la première).
    #[tokio::test]
    async fn une_course_unique_est_rattrapee_par_la_nouvelle_tentative() {
        let mut calls = 0;
        let res = with_one_retry(|| {
            calls += 1;
            let n = calls;
            async move {
                Ok(if n == 1 {
                    PoseOutcome::Changed
                } else {
                    PoseOutcome::Posed(pdf(2))
                })
            }
        })
        .await
        .expect("posé à la seconde tentative");
        assert_eq!(res.bytes, vec![2]);
        assert_eq!(calls, 2);
    }

    /// Le document d'un rendu concurrent est servi tel quel, sans nouvelle
    /// tentative.
    #[tokio::test]
    async fn le_document_adopte_est_servi_sans_nouvelle_tentative() {
        let mut calls = 0;
        let res = with_one_retry(|| {
            calls += 1;
            async { Ok(PoseOutcome::Adopted(pdf(7))) }
        })
        .await
        .expect("adopté");
        assert_eq!(res.bytes, vec![7]);
        assert_eq!(calls, 1);
    }
}
