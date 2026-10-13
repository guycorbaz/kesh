//! Story 27-3 (#626) — empreinte d'aperçu d'un import bancaire.
//!
//! `POST /api/v1/bank-imports/preview` rend une empreinte (`previewFingerprint`)
//! des **entrées** qui ont produit l'aperçu ; `POST /api/v1/bank-imports` accepte
//! cette empreinte en champ multipart facultatif, la recalcule sur ses propres
//! entrées et refuse l'import (`409 BANK_IMPORT_PREVIEW_STALE`) si elles diffèrent.
//! Un client qui a vu un aperçu ne peut donc plus importer avec un profil, un
//! compte ou un fichier qui a changé depuis.
//!
//! # Ce que l'empreinte couvre
//!
//! SHA-256 (hex minuscule) de la sérialisation `serde_json` d'un **tableau**
//! ordonné — un tableau, et non un objet, pour que l'ordre ne dépende d'aucune
//! implémentation de `Map` :
//!
//! 1. la balise de domaine [`DOMAIN_TAG`] ;
//! 2. `company_id` ;
//! 3. l'id du compte bancaire et son IBAN **normalisé** (espaces retirés,
//!    majuscules — même normalisation que la sélection du relevé CAMT) ;
//! 4. le hash SHA-256 du fichier ;
//! 5. le format détecté (`"csv"` / `"camt053"`) ;
//! 6. pour un CSV, le **profil résolu** — tous ses champs sauf les deux
//!    horodatages ; pour un CAMT, `null`.
//!
//! Le profil est lu par **déstructuration exhaustive** : un champ ajouté plus tard
//! à [`BankProfile`] casse la compilation de [`preview_fingerprint`] au lieu d'en
//! être absent en silence.
//!
//! # Ce qu'elle ne couvre pas
//!
//! - l'état de la base (doublons de fichier ou de lignes) : la création les
//!   recalcule de toute façon ;
//! - les cases de confirmation ;
//! - le nom du fichier : le profil **résolu** y est déjà, si bien qu'un aperçu
//!   sur *Auto-détection* puis une création avec l'id résolu donnent la même
//!   empreinte ;
//! - `created_at` / `updated_at` du profil : un enregistrement sans changement de
//!   contenu ne périme pas l'aperçu.
//!
//! # Pourquoi pas de secret
//!
//! L'empreinte n'authentifie rien et n'est pas un secret : elle détecte une
//! divergence entre l'aperçu et la création. La menace visée est le client
//! **périmé** (écran resté sur un ancien aperçu, profil modifié dans un autre
//! onglet), pas le client malveillant — qui a de toute façon le droit d'importer.
//! Un HMAC n'apporterait rien, un jeton stocké imposerait une table et une
//! expiration sans rien fermer de plus. L'empreinte est **facultative** à la
//! création : un client qui ne l'envoie pas n'est pas contrôlé (angle mort
//! assumé, `docs/api-external.md`).

use kesh_db::entities::BankAccount;
use kesh_db::entities::bank_profile::BankProfile;
use sha2::{Digest, Sha256};

use crate::errors::AppError;
use crate::routes::bank_imports::normalize_iban;

/// Balise de domaine, première entrée du tableau haché. La changer périme toutes
/// les empreintes en circulation (ce qui ne coûte qu'un recalcul d'aperçu).
pub(crate) const DOMAIN_TAG: &str = "kesh-bank-import-preview/v1";

/// Longueur d'une empreinte : SHA-256 en hexadécimal.
const FINGERPRINT_LEN: usize = 64;

/// Format du fichier tel que détecté par la route, dans l'empreinte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FingerprintFormat {
    Csv,
    Camt053,
}

impl FingerprintFormat {
    fn as_str(self) -> &'static str {
        match self {
            FingerprintFormat::Csv => "csv",
            FingerprintFormat::Camt053 => "camt053",
        }
    }
}

/// Calcule l'empreinte d'aperçu. **Seule** fonction qui la calcule : `preview`,
/// `preview_csv`, `create` et `create_csv` l'appellent toutes, si bien que
/// l'aperçu et la création ne peuvent pas sérialiser différemment.
///
/// `profile` vaut `Some` pour un CSV (le profil **résolu**, celui qui sert au
/// parse) et `None` pour un CAMT.053.
pub(crate) fn preview_fingerprint(
    company_id: i64,
    bank_account: &BankAccount,
    file_hash: &str,
    format: FingerprintFormat,
    profile: Option<&BankProfile>,
) -> String {
    let profile_json = profile.map(|p| {
        // Déstructuration exhaustive, sans `..` : un champ neuf de `BankProfile`
        // doit être classé ici (inclus, ou lié à `_` avec une raison).
        let BankProfile {
            id,
            company_id: profile_company_id,
            bank_name,
            filename_pattern,
            column_mapping_json,
            date_format,
            decimal_separator,
            field_separator,
            encoding,
            header_row_count,
            debit_sign,
            // Horodatages exclus : un enregistrement sans changement de contenu
            // ne doit pas périmer l'aperçu.
            created_at: _,
            updated_at: _,
        } = p;
        serde_json::json!([
            id,
            profile_company_id,
            bank_name,
            filename_pattern,
            column_mapping_json,
            date_format,
            decimal_separator,
            field_separator,
            encoding,
            header_row_count,
            debit_sign,
        ])
    });
    let payload = serde_json::json!([
        DOMAIN_TAG,
        company_id,
        bank_account.id,
        normalize_iban(&bank_account.iban),
        file_hash,
        format.as_str(),
        profile_json,
    ]);
    // `to_vec` d'un `Value` ne peut pas échouer (aucune clé non-chaîne).
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    crate::util::hex_encode(&Sha256::digest(&bytes))
}

/// Valide le champ multipart `previewFingerprint` : exactement 64 caractères
/// hexadécimaux **minuscules** (la forme que rend l'aperçu). Toute autre forme →
/// `400 VALIDATION_ERROR` — une empreinte mal formée est un défaut du client, pas
/// un aperçu périmé.
pub(crate) fn parse_fingerprint_field(text: &str) -> Result<String, AppError> {
    let well_formed = text.len() == FINGERPRINT_LEN
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !well_formed {
        return Err(AppError::Validation(format!(
            "previewFingerprint invalide : {FINGERPRINT_LEN} caractères hexadécimaux minuscules attendus"
        )));
    }
    Ok(text.to_string())
}

/// Compare l'empreinte envoyée (si elle l'est) à celle que la création recalcule.
/// Absente → `Ok` (compatibilité, angle mort assumé) ; différente →
/// `409 BANK_IMPORT_PREVIEW_STALE`.
pub(crate) fn check_preview_fingerprint(
    sent: Option<&str>,
    recomputed: &str,
) -> Result<(), AppError> {
    match sent {
        Some(fp) if fp != recomputed => Err(AppError::BankImportPreviewStale),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn ts(day: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    }

    fn account() -> BankAccount {
        BankAccount {
            id: 7,
            company_id: 1,
            bank_name: "UBS".into(),
            description: None,
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
            journal_account_id: None,
            version: 1,
            archived: false,
            closed_on: None,
            created_at: ts(1),
            updated_at: ts(1),
        }
    }

    fn profile() -> BankProfile {
        BankProfile {
            id: 3,
            company_id: 1,
            bank_name: "UBS CSV".into(),
            filename_pattern: Some("^ubs".into()),
            column_mapping_json: r#"{"date":0,"amount":1}"#.into(),
            date_format: "%d.%m.%Y".into(),
            decimal_separator: ".".into(),
            field_separator: ";".into(),
            encoding: Some("UTF-8".into()),
            header_row_count: 1,
            debit_sign: "negative".into(),
            created_at: ts(1),
            updated_at: ts(2),
        }
    }

    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn fp_csv(acc: &BankAccount, p: &BankProfile) -> String {
        preview_fingerprint(1, acc, HASH, FingerprintFormat::Csv, Some(p))
    }

    fn is_lower_hex_64(s: &str) -> bool {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }

    /// U1 — déterminisme et forme.
    #[test]
    fn u1_deterministic_and_lower_hex_64() {
        let a = fp_csv(&account(), &profile());
        let b = fp_csv(&account(), &profile());
        assert_eq!(a, b);
        assert!(is_lower_hex_64(&a), "{a}");
        let camt = preview_fingerprint(1, &account(), HASH, FingerprintFormat::Camt053, None);
        assert!(is_lower_hex_64(&camt));
        assert_ne!(camt, a);
    }

    /// U2 — sensibilité : chaque champ du profil hors horodatages (11), puis
    /// la société, le compte (id, IBAN), le hash du fichier et le format.
    #[test]
    fn u2_every_covered_input_changes_the_fingerprint() {
        let base = fp_csv(&account(), &profile());
        type Mutation = (&'static str, fn(&mut BankProfile));
        let profile_mutations: [Mutation; 11] = [
            ("id", |p| p.id = 4),
            ("company_id", |p| p.company_id = 2),
            ("bank_name", |p| p.bank_name = "Autre".into()),
            ("filename_pattern", |p| p.filename_pattern = None),
            ("column_mapping_json", |p| {
                p.column_mapping_json = r#"{"date":1,"amount":0}"#.into()
            }),
            ("date_format", |p| p.date_format = "%Y-%m-%d".into()),
            ("decimal_separator", |p| p.decimal_separator = ",".into()),
            ("field_separator", |p| p.field_separator = ",".into()),
            ("encoding", |p| p.encoding = None),
            ("header_row_count", |p| p.header_row_count = 2),
            ("debit_sign", |p| p.debit_sign = "positive".into()),
        ];
        for (name, mutate) in profile_mutations {
            let mut p = profile();
            mutate(&mut p);
            assert_ne!(
                fp_csv(&account(), &p),
                base,
                "champ de profil {name} non couvert"
            );
        }

        assert_ne!(
            preview_fingerprint(
                2,
                &account(),
                HASH,
                FingerprintFormat::Csv,
                Some(&profile())
            ),
            base,
            "company_id non couvert"
        );
        let mut acc = account();
        acc.id = 8;
        assert_ne!(fp_csv(&acc, &profile()), base, "id du compte non couvert");
        let mut acc = account();
        acc.iban = "CH5604835012345678009".into();
        assert_ne!(fp_csv(&acc, &profile()), base, "IBAN non couvert");
        let other_hash = "b".repeat(64);
        assert_ne!(
            preview_fingerprint(
                1,
                &account(),
                &other_hash,
                FingerprintFormat::Csv,
                Some(&profile())
            ),
            base,
            "hash du fichier non couvert"
        );
        assert_ne!(
            preview_fingerprint(
                1,
                &account(),
                HASH,
                FingerprintFormat::Camt053,
                Some(&profile())
            ),
            base,
            "format non couvert"
        );
    }

    /// U3 — insensibilité : horodatages du profil, graphie de l'IBAN.
    #[test]
    fn u3_timestamps_and_iban_spelling_do_not_change_the_fingerprint() {
        let base = fp_csv(&account(), &profile());
        let mut p = profile();
        p.created_at = ts(20);
        p.updated_at = ts(21);
        assert_eq!(fp_csv(&account(), &p), base, "horodatages inclus à tort");

        let mut acc = account();
        acc.iban = "ch93 0076 2011 6238 5295 7".into();
        assert_eq!(fp_csv(&acc, &profile()), base, "IBAN non normalisé");
        // Les champs du compte hors id et IBAN n'entrent pas non plus.
        let mut acc = account();
        acc.bank_name = "Autre".into();
        acc.version = 9;
        acc.updated_at = ts(22);
        assert_eq!(fp_csv(&acc, &profile()), base);
    }

    /// U4 — forme du champ multipart.
    #[test]
    fn u4_parse_fingerprint_field() {
        let ok = "0123456789abcdef".repeat(4);
        assert_eq!(parse_fingerprint_field(&ok).unwrap(), ok);
        for bad in [
            "a".repeat(63),
            "a".repeat(65),
            "A".repeat(64),
            format!("g{}", "a".repeat(63)),
            String::new(),
            format!(" {}", "a".repeat(63)),
        ] {
            assert!(
                matches!(parse_fingerprint_field(&bad), Err(AppError::Validation(_))),
                "accepté à tort : {bad:?}"
            );
        }
    }

    #[test]
    fn check_accepts_absent_and_equal_refuses_different() {
        let fp = "a".repeat(64);
        assert!(check_preview_fingerprint(None, &fp).is_ok());
        assert!(check_preview_fingerprint(Some(&fp), &fp).is_ok());
        assert!(matches!(
            check_preview_fingerprint(Some(&"b".repeat(64)), &fp),
            Err(AppError::BankImportPreviewStale)
        ));
    }
}
