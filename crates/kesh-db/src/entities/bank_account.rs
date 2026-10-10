//! Entité `BankAccount` : compte bancaire associé à une company.
//!
//! Story 26-2a (#622) — la description optionnelle du compte, sa borne
//! [`BANK_ACCOUNT_DESCRIPTION_MAX_CHARS`] et le **nom d'affichage** d'un compte
//! ([`bank_account_display_name`]), règle partagée avec le frontend
//! (`frontend/src/lib/features/bank-accounts/format.ts`, `bankAccountName`).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Compte bancaire persisté en base.
///
/// Story 8-5a-zero — ajout du champ `journal_account_id` (Option<i64>)
/// qui lie le `bank_account` à un compte du plan comptable
/// (typiquement classe 1 — 1010 Poste / 1020 Banque). Initialement
/// NULL pour les rows pré-migration : le user **doit** configurer
/// avant d'utiliser FR45 (8-5a-base) ou FR48 (8-5a-bis), via la route
/// `PATCH /api/v1/bank-accounts/{id}`.
///
/// Sérialisation `journalAccountId` (camelCase) cohérente avec la
/// convention `kesh-api`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub id: i64,
    pub company_id: i64,
    pub bank_name: String,
    /// Story 26-2a (#622) — description optionnelle du compte (« Personnel »,
    /// « Épargne »), qui distingue deux comptes de la même banque. Normalisée
    /// par la route (`trim`, vide → `None`, au plus
    /// [`BANK_ACCOUNT_DESCRIPTION_MAX_CHARS`] caractères) ; `None` = absente,
    /// l'affichage est alors celui de la banque seule. Non unique.
    ///
    /// Écrite **seulement** par la création de la route et par le `PUT`
    /// (`update_for_company`, remplacement complet) ; aucun autre écrivain du
    /// dépôt ne la touche.
    pub description: Option<String>,
    /// IBAN normalisé sans espaces.
    pub iban: String,
    /// QR-IBAN optionnel (plage QR-IID 30000-31999).
    pub qr_iban: Option<String>,
    pub is_primary: bool,
    /// Compte du plan comptable lié à ce bank_account (classe 1 typique).
    /// `None` = non configuré → FR45/FR48 retourneront 412 BANK_ACCOUNT_NOT_CONFIGURED
    /// (livré par 8-5a-base, pas par 8-5a-zero).
    pub journal_account_id: Option<i64>,
    pub version: i32,
    /// Story v014-1 — `true` = soft-deleted, exclu de `list_by_company` par
    /// défaut + des mutations post-archivage (PATCH/PUT/DELETE/imports/reco).
    /// v0.1 : pas de UI restoration (L1 limitation v0.1).
    pub archived: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un compte bancaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBankAccount {
    pub company_id: i64,
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
    pub is_primary: bool,
}

/// Longueur maximale, en caractères Unicode (`chars().count()`, pas en octets),
/// de [`BankAccount::description`] — Story 26-2a (#622).
///
/// Source unique de la borne côté serveur : la validation de la route la lit,
/// et un test confronte cette constante au `varchar(…)` de la colonne dans le
/// schéma de test régénéré. Le frontend porte la sienne
/// (`BANK_ACCOUNT_DESCRIPTION_MAX`, `format.ts`).
pub const BANK_ACCOUNT_DESCRIPTION_MAX_CHARS: usize = 60;

/// Séparateur entre la banque et la description : tiret cadratin (U+2014)
/// entouré d'espaces — celui des sélecteurs de comptes du frontend.
const DISPLAY_SEPARATOR: &str = " \u{2014} ";

/// Nom d'affichage d'un compte bancaire — Story 26-2a (#622).
///
/// `"UBS — Personnel"` avec description, `"UBS"` sans. Une description `None`,
/// vide ou faite d'espaces vaut absente (elle est normalement déjà normalisée
/// par la route, la règle est défensive). Mêmes règles que `bankAccountName`
/// du frontend.
pub fn bank_account_display_name(bank_name: &str, description: Option<&str>) -> String {
    match description.map(str::trim) {
        Some(d) if !d.is_empty() => format!("{bank_name}{DISPLAY_SEPARATOR}{d}"),
        _ => bank_name.to_string(),
    }
}

impl BankAccount {
    /// Nom d'affichage du compte : voir [`bank_account_display_name`].
    pub fn display_name(&self) -> String {
        bank_account_display_name(&self.bank_name, self.description.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC3 (C-26-2-21) — la constante est la borne **réelle** de la colonne :
    /// confrontée au `varchar(…)` du schéma de test régénéré (lui-même tenu égal
    /// au schéma des migrations par `test_schema_guard.rs`).
    #[test]
    fn la_borne_de_la_description_est_celle_de_la_colonne() {
        const SCHEMA: &str = include_str!("../../test-schema/0001_schema_squash.sql");
        let debut = SCHEMA
            .find("CREATE TABLE `bank_accounts` (")
            .expect("table bank_accounts absente du schéma");
        let bloc = &SCHEMA[debut..];
        let bloc = &bloc[..bloc.find(") ENGINE").expect("fin de table")];
        let ligne = bloc
            .lines()
            .find(|l| l.trim_start().starts_with("`description`"))
            .expect("colonne description absente de bank_accounts");
        assert!(
            ligne.contains(&format!("varchar({BANK_ACCOUNT_DESCRIPTION_MAX_CHARS})")),
            "la colonne doit être varchar({BANK_ACCOUNT_DESCRIPTION_MAX_CHARS}), trouvé : {ligne}"
        );
        assert!(
            ligne.contains("DEFAULT NULL"),
            "la colonne doit être nullable : {ligne}"
        );
    }

    #[test]
    fn display_name_avec_description() {
        assert_eq!(
            bank_account_display_name("UBS", Some("Personnel")),
            "UBS \u{2014} Personnel"
        );
        assert_eq!(
            bank_account_display_name("UBS", Some("Personnel")),
            "UBS — Personnel"
        );
    }

    #[test]
    fn display_name_sans_description() {
        assert_eq!(bank_account_display_name("UBS", None), "UBS");
    }

    #[test]
    fn display_name_description_vide_ou_espaces_vaut_absente() {
        assert_eq!(bank_account_display_name("UBS", Some("")), "UBS");
        assert_eq!(bank_account_display_name("UBS", Some("   ")), "UBS");
    }

    #[test]
    fn display_name_de_l_entite() {
        let now = chrono::NaiveDate::from_ymd_opt(2026, 10, 10)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let mut a = BankAccount {
            id: 1,
            company_id: 1,
            bank_name: "UBS".into(),
            description: Some("Épargne".into()),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: false,
            journal_account_id: None,
            version: 1,
            archived: false,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(a.display_name(), "UBS — Épargne");
        a.description = None;
        assert_eq!(a.display_name(), "UBS");
    }
}
