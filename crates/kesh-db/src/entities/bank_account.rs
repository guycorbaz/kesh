//! Entité `BankAccount` : compte bancaire associé à une company.
//!
//! Story 26-2a (#622) — la description optionnelle du compte, sa borne
//! [`BANK_ACCOUNT_DESCRIPTION_MAX_CHARS`] et le **nom d'affichage** d'un compte
//! ([`bank_account_display_name`]), règle partagée avec le frontend
//! (`frontend/src/lib/features/bank-accounts/format.ts`, `bankAccountName`).

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type d'un compte de `bank_accounts` — Story 34-11a1 (#972, C-e341-13).
///
/// Mappé en `VARCHAR(20)` (`account_kind`), domaine fermé **à l'octet** par
/// `chk_bank_accounts_account_kind` (`BINARY … IN`) : la collation de la table
/// admettrait sinon `BANK` ou `bank `, que [`std::str::FromStr`] refuse — et le
/// décodage de toute liste de comptes échouerait. Sérialisé en JSON `"bank"` /
/// `"credit_card"` (champ `accountKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BankAccountKind {
    /// Compte bancaire : IBAN obligatoire.
    #[default]
    Bank,
    /// Carte de crédit : sans IBAN, ni QR-IBAN, jamais principale ; seuls les
    /// quatre derniers chiffres sont gardés. Créée par la Story 34-11a2.
    CreditCard,
}

impl BankAccountKind {
    /// Valeur persistée (colonne `account_kind`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bank => "bank",
            Self::CreditCard => "credit_card",
        }
    }
}

impl std::fmt::Display for BankAccountKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BankAccountKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bank" => Ok(Self::Bank),
            "credit_card" => Ok(Self::CreditCard),
            other => Err(format!("BankAccountKind inconnu : {other}")),
        }
    }
}

impl Type<MySql> for BankAccountKind {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for BankAccountKind {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for BankAccountKind {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

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
    ///
    /// Story 34-11a1 (#972) — `None` pour un compte **sans IBAN** (carte de
    /// crédit). Le schéma garantit `Some` pour un compte
    /// [`BankAccountKind::Bank`] (`chk_bank_accounts_bank_has_iban`) et `None`
    /// pour une carte (`chk_bank_accounts_card_shape`). Les sites qui ont
    /// besoin d'un IBAN (lot `pain.001`, import CAMT, QR-facture) refusent sur
    /// `iban.is_none()`, jamais sur le type (C-e341-18) : un futur type sans
    /// IBAN y tombe sans changement.
    pub iban: Option<String>,
    /// QR-IBAN optionnel (plage QR-IID 30000-31999).
    pub qr_iban: Option<String>,
    /// Story 34-11a1 (#972, C-e341-13) — type du compte, déclaré et non déduit
    /// d'un IBAN nul. `bank` pour tout compte d'avant la migration
    /// `20261110000001_bank_accounts_without_iban` (valeur par défaut).
    pub account_kind: BankAccountKind,
    /// Story 34-11a1 (#972) — quatre derniers chiffres d'une carte, seuls
    /// gardés ; `None` pour un compte bancaire (`chk_bank_accounts_card_last4`).
    pub card_last4: Option<String>,
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
    /// Story 28-4a (#642) — date de clôture : `None` = compte **ouvert** ;
    /// `Some(date)` = compte **clos**, en lecture seule (sérialisé `closedOn`,
    /// `"AAAA-MM-JJ"` ou `null`).
    ///
    /// Distinct d'[`archived`](Self::archived) : la clôture retire de l'usage
    /// un compte qui a servi, garde son historique lisible et se défait
    /// (réouverture). La date est **informative** — c'est l'état qui garde.
    /// Un compte clos n'est jamais principal (contrainte
    /// `chk_bank_accounts_closed_not_primary`).
    ///
    /// Écrite **seulement** par `bank_accounts::close_in_tx` et
    /// `bank_accounts::reopen_in_tx`.
    pub closed_on: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl BankAccount {
    /// `true` si le compte est clos (Story 28-4a, #642).
    pub fn is_closed(&self) -> bool {
        self.closed_on.is_some()
    }
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
        // Séparateur : tiret cadratin U+2014 entouré d'espaces.
        assert_eq!(
            bank_account_display_name("UBS", Some("Personnel")),
            "UBS \u{2014} Personnel"
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
            iban: Some("CH9300762011623852957".into()),
            qr_iban: None,
            account_kind: BankAccountKind::Bank,
            card_last4: None,
            is_primary: false,
            journal_account_id: None,
            version: 1,
            archived: false,
            closed_on: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(a.display_name(), "UBS — Épargne");
        a.description = None;
        assert_eq!(a.display_name(), "UBS");
    }
}
