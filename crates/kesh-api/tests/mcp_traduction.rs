//! Story 32-6a (refs #723, AC-a18) — la seule traduction d'une erreur de
//! service en erreur d'outil, `mcp::textes::depuis_app_error`.
//!
//! Hors de `src/` à dessein : les `AppError` construites ici ne tombent pas
//! sous la garde d'inventaire `reponses_par_rendre_seul` (AC-a31).
//!
//! Chaque bras de la liste blanche rend sa variante ; des variantes **hors
//! liste**, prises dans plusieurs familles, rendent toutes l'erreur interne
//! fixe ; et le texte envoyé au client est la constante, jamais le message.

use kesh_api::errors::AppError;
use kesh_api::mcp::textes::{
    TEXTE_ARGUMENTS_INVALIDES, TEXTE_COMPTE_NON_LETTRABLE, TEXTE_ERREUR_INTERNE,
    TEXTE_EXERCICE_INTROUVABLE, TEXTE_INTROUVABLE, TEXTE_PERIODE_HORS_EXERCICE, depuis_app_error,
};
use kesh_api::mcp::{ErreurOutil, MotifArgument};
use kesh_db::errors::DbError;

const PIEGE: &str = "PIEGE_APP_326";

#[test]
fn traduction_des_erreurs_de_service() {
    assert_eq!(depuis_app_error(AppError::Forbidden), ErreurOutil::Inconnu);
    assert_eq!(
        depuis_app_error(AppError::Database(DbError::NotFound)),
        ErreurOutil::Introuvable
    );
    assert_eq!(
        depuis_app_error(AppError::Validation(PIEGE.to_string())),
        ErreurOutil::ArgumentsInvalides {
            argument: "(demande)",
            motif: MotifArgument::Invalide
        }
    );
    // Story 32-9a (AC-a22) : trois erreurs de demande, à texte fixe, sans
    // recopier leurs champs.
    let jour = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    assert_eq!(
        depuis_app_error(AppError::ReportFiscalYearNotFound { fiscal_year_id: 7 }),
        ErreurOutil::ExerciceIntrouvable
    );
    assert_eq!(
        depuis_app_error(AppError::ReportPeriodOutOfFiscalYear {
            fy_start: jour,
            fy_end: jour,
            requested_start: jour,
            requested_end: jour,
        }),
        ErreurOutil::PeriodeHorsExercice
    );
    assert_eq!(
        depuis_app_error(AppError::Database(DbError::LetteringAccountNotLetterable)),
        ErreurOutil::CompteNonLettrable
    );
    // Les textes ne portent aucune valeur de l'erreur (identifiant, dates).
    for t in [
        TEXTE_EXERCICE_INTROUVABLE,
        TEXTE_PERIODE_HORS_EXERCICE,
        TEXTE_COMPTE_NON_LETTRABLE,
    ] {
        assert!(
            !t.is_empty() && !t.contains("2026") && !t.contains('7'),
            "{t}"
        );
    }

    let hors_liste = [
        AppError::Internal(PIEGE.to_string()),
        AppError::Database(DbError::OptimisticLockConflict),
        AppError::Database(DbError::Invariant(PIEGE.to_string())),
        AppError::FiscalYearClosed {
            date: PIEGE.to_string(),
        },
        AppError::IllegalState(PIEGE.to_string()),
        AppError::RateLimited { retry_after: 7 },
        AppError::Unauthenticated(PIEGE.to_string()),
        AppError::AccountNotFound {
            account_id: 7,
            missing_account_ids: None,
        },
    ];
    let n = hors_liste.len();
    for e in hors_liste {
        let texte = format!("{e}");
        assert_eq!(depuis_app_error(e), ErreurOutil::Interne, "{texte}");
    }
    assert_eq!(n, 8, "plusieurs familles éprouvées");
}

/// Le texte d'outil est une constante : le message de l'erreur n'y est jamais.
#[test]
fn les_textes_d_outil_sont_fixes() {
    for t in [
        TEXTE_ARGUMENTS_INVALIDES,
        TEXTE_ERREUR_INTERNE,
        TEXTE_INTROUVABLE,
    ] {
        assert!(!t.contains(PIEGE));
        assert!(!t.is_empty());
    }
}
