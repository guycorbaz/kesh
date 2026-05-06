//! Conversion des relevés bancaires importés (`kesh-import`) vers les types
//! domaine `kesh-core`.
//!
//! Ce module matérialise la décision architecture #7 : les crates publiables
//! (`kesh-import`, `kesh-payment`, `kesh-qrbill`) ont des types autonomes,
//! et les conversions vers les types domaine vivent côté `kesh-core` via
//! `From`/`Into`. La direction de dépendance Cargo est donc
//! `kesh-core → kesh-import` (jamais l'inverse).
//!
//! # Story 8-1a — extensions livrées
//!
//! - [`BankImportDraft`] : entête de relevé bancaire prêt à être persisté
//!   (clés étrangères incluses).
//! - [`SourceFormatTag`] : projection finie de
//!   [`kesh_import::SourceFormat`] vers les valeurs effectivement stockées
//!   en base (8-1b).
//! - [`from_imported`] : conversion `ImportedStatement` →
//!   `(BankImportDraft, Vec<BankTransactionDraft>)`. Les FK
//!   (`bank_account_id`, `company_id`, `imported_at`,
//!   `imported_by_user_id`) sont passées en paramètres pour préserver la
//!   pureté `kesh-core` (zéro I/O, zéro horloge interne).
//! - [`validate_balance`] : check de cohérence
//!   `opening + Σ transactions ≈ closing` à 0.01 près (CR-010 #62). Skip
//!   silencieux si les soldes sont absents.
//! - [`validate_currency_supported_v0_1`] : enforce le périmètre v0.1
//!   limité à CHF.

use std::fmt;

use chrono::{NaiveDate, NaiveDateTime};
use kesh_import::{ImportedStatement, ImportedTransaction, SourceFormat};
use rust_decimal::Decimal;

use crate::errors::CoreError;
use crate::types::Money;

/// Brouillon de transaction bancaire — image projetée d'un
/// [`ImportedTransaction`] dans le vocabulaire domaine `kesh-core`.
///
/// Les clés étrangères (`bank_account_id`, `import_id`, `company_id`) ne
/// figurent pas dans ce type : elles sont injectées par
/// `kesh-api::routes::bank_imports` au moment de la persistance, à partir du
/// contexte de la requête (compte sélectionné par l'utilisateur, tenant
/// courant, hash du fichier importé).
///
/// L'IBAN de la contrepartie est conservé sous forme de `String` brute à ce
/// stade : la validation MOD-97 via [`crate::types::Iban`] est laissée à
/// Story 8-1 pour permettre un import « tolérant » (transaction conservée
/// même si l'IBAN est mal formé, avec un statut d'avertissement) plutôt
/// qu'un rejet strict du fichier entier.
#[derive(Clone, Debug, PartialEq)]
pub struct BankTransactionDraft {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Money,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
}

impl From<ImportedTransaction> for BankTransactionDraft {
    fn from(tx: ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency,
            reference: tx.reference,
            details: tx.details,
            end_to_end_id: tx.end_to_end_id,
            transaction_id: tx.transaction_id,
            counterparty_iban: tx.counterparty_iban,
            counterparty_name: tx.counterparty_name,
        }
    }
}

impl From<&ImportedTransaction> for BankTransactionDraft {
    fn from(tx: &ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency.clone(),
            reference: tx.reference.clone(),
            details: tx.details.clone(),
            end_to_end_id: tx.end_to_end_id.clone(),
            transaction_id: tx.transaction_id.clone(),
            counterparty_iban: tx.counterparty_iban.clone(),
            counterparty_name: tx.counterparty_name.clone(),
        }
    }
}

/// Entête d'import bancaire prêt à être persisté.
///
/// Le `BankImportDraft` matérialise la trace d'un fichier CAMT.053 (ou
/// CSV, dès Story 8-2) traité par l'application : le crate
/// `kesh-import` produit un [`ImportedStatement`] sans clés étrangères ;
/// `from_imported` complète celui-ci avec les FK injectées par le
/// handler API à partir du contexte de la requête (compte sélectionné,
/// tenant courant, hash du fichier, timestamp serveur, user du JWT).
///
/// Le champ [`SourceFormatTag`] est une projection finie pour la base
/// (`ENUM('CAMT053_V04', 'CAMT053_V08', 'CSV')`) — il évite d'embarquer
/// la `String` arbitraire de `kesh_import::SourceFormat::Camt053 { version }`.
///
/// `imported_by_user_id` est passé en paramètre par le caller (handler
/// API 8-1b) pour rester compatible avec la pureté `kesh-core` : zéro
/// horloge interne, zéro lookup user.
#[derive(Clone, Debug, PartialEq)]
pub struct BankImportDraft {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub filename: String,
    /// SHA-256 hex du fichier source (64 caractères). Utilisé pour la
    /// détection de doublons stricte (Story 8-3).
    pub file_hash: String,
    pub source_format: SourceFormatTag,
    pub statement_id: Option<String>,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub opening_balance: Option<Money>,
    pub closing_balance: Option<Money>,
    pub transaction_count: i32,
    pub imported_at: NaiveDateTime,
    pub imported_by_user_id: i64,
}

/// Projection finie du format source d'un import bancaire.
///
/// Stockée en base comme une chaîne courte (`ENUM` MariaDB) pour rester
/// indexable et stable face à l'évolution du schéma `SourceFormat` côté
/// `kesh-import` (qui peut accumuler des versions et des profils CSV
/// arbitraires).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFormatTag {
    Camt053V04,
    Camt053V08,
    /// Story 8-2 T5.0.b — import CSV (multi-encodage + profil banque).
    /// La distinction par profil bancaire spécifique vit dans
    /// `audit_log.details_json.bank_profile_id` (cf. Limitation L4).
    Csv,
}

impl SourceFormatTag {
    /// Code stable utilisé pour la persistance (colonne `source_format`).
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Camt053V04 => "CAMT053_V04",
            Self::Camt053V08 => "CAMT053_V08",
            Self::Csv => "CSV",
        }
    }
}

/// Délègue à [`SourceFormatTag::as_db_str`] (review code Pass 2 finding
/// F18 — `format!("{}", tag)` sera utilisé en 8-1b pour le logging et
/// les messages d'erreur sans dépendre du `Debug` derive).
impl fmt::Display for SourceFormatTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// Convertit un [`ImportedStatement`] (sortie du parseur) en couple
/// `(BankImportDraft, Vec<BankTransactionDraft>)` prêt à être persisté.
///
/// Les clés étrangères (`bank_account_id`, `company_id`,
/// `imported_at`, `imported_by_user_id`) ne sont pas dérivables du
/// fichier importé — elles viennent du contexte HTTP (JWT, formulaire
/// de l'utilisateur). Elles sont passées en paramètres pour préserver
/// la pureté `kesh-core` (zéro I/O, zéro horloge interne).
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportUnknownVersion`] si le suffixe de
/// version porté par `stmt.source_format` ne correspond à aucune
/// variante connue de [`SourceFormatTag`].
pub fn from_imported(
    stmt: &ImportedStatement,
    bank_account_id: i64,
    company_id: i64,
    file_hash: String,
    filename: String,
    imported_at: NaiveDateTime,
    imported_by_user_id: i64,
) -> Result<(BankImportDraft, Vec<BankTransactionDraft>), CoreError> {
    let source_format = match &stmt.source_format {
        SourceFormat::Camt053 { version } if version == "001.04" => SourceFormatTag::Camt053V04,
        SourceFormat::Camt053 { version } if version == "001.08" => SourceFormatTag::Camt053V08,
        SourceFormat::Camt053 { version } => {
            return Err(CoreError::BankImportUnknownVersion(version.clone()));
        }
        SourceFormat::Csv { .. } => SourceFormatTag::Csv,
    };

    let transactions: Vec<BankTransactionDraft> = stmt
        .transactions
        .iter()
        .map(BankTransactionDraft::from)
        .collect();

    let draft = BankImportDraft {
        company_id,
        bank_account_id,
        filename,
        file_hash,
        source_format,
        statement_id: stmt.statement_id.clone(),
        period_from: stmt.period_from,
        period_to: stmt.period_to,
        opening_balance: stmt.opening_balance.map(Money::new),
        closing_balance: stmt.closing_balance.map(Money::new),
        transaction_count: i32::try_from(stmt.transactions.len()).unwrap_or(i32::MAX),
        imported_at,
        imported_by_user_id,
    };

    Ok((draft, transactions))
}

/// Vérifie la cohérence du solde de clôture déclaré (CR-010 #62).
///
/// Pour un relevé `stmt` portant à la fois `opening_balance` et
/// `closing_balance`, calcule `opening + Σ transactions` et compare au
/// `closing` déclaré. Tolère un écart absolu de **0.01** (un centime,
/// règle d'arrondi commerciale).
///
/// La comparaison utilise `>` strict : un écart **égal** à 0.01 est
/// accepté (dans la tolérance), seul un écart **strictement supérieur**
/// déclenche l'erreur. Voir tests `validate_balance_passes_at_exactly_one_cent`
/// et `validate_balance_fails_just_above_one_cent` pour les bornes.
///
/// Si l'un des deux soldes est absent, retourne `Ok(())` (skip
/// silencieux). Cette tolérance est calibrée pour les CSV (Story 8-2)
/// dont certains profils n'embarquent pas les soldes. **Pour CAMT.053
/// spécifiquement**, le schéma ISO 20022 impose `OPBD` et `CLBD` : un
/// `ImportedStatement` issu du parseur `kesh-import::camt053` sans
/// soldes doit être traité comme un signal de fichier malformé en
/// amont (le parseur lève `MissingRequiredField` sur un `<Bal>`
/// partiellement renseigné depuis Story 8-1a Pass 1 review). Le
/// présent skip ne s'applique donc en pratique qu'aux sources CSV.
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportBalanceMismatch`] si l'écart absolu
/// dépasse 0.01.
pub fn validate_balance(stmt: &ImportedStatement) -> Result<(), CoreError> {
    let (Some(opening), Some(closing)) = (stmt.opening_balance, stmt.closing_balance) else {
        return Ok(());
    };
    let sum = stmt.sum_transactions();
    let expected = opening + sum;
    let diff = (expected - closing).abs();
    if diff > Decimal::new(1, 2) {
        return Err(CoreError::BankImportBalanceMismatch {
            opening: Money::new(opening),
            closing: Money::new(closing),
            sum: Money::new(sum),
            diff: Money::new(diff),
        });
    }
    Ok(())
}

/// Vérifie que la devise du relevé importé est supportée par le
/// périmètre v0.1 (CHF uniquement).
///
/// Le rejet vit côté `kesh-core` plutôt que côté `kesh-import` pour
/// conserver le parseur agnostique du périmètre fonctionnel (un autre
/// produit consommant `kesh-import` pourrait accepter EUR ou USD).
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportUnsupportedCurrency`] avec le code
/// devise rencontré pour toute devise autre que `"CHF"`.
pub fn validate_currency_supported_v0_1(stmt: &ImportedStatement) -> Result<(), CoreError> {
    if stmt.currency != "CHF" {
        return Err(CoreError::BankImportUnsupportedCurrency(
            stmt.currency.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn fixture() -> ImportedTransaction {
        ImportedTransaction {
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 3).unwrap()),
            amount: dec!(1234.56),
            currency: "CHF".to_string(),
            reference: Some("RF18539007547034".to_string()),
            details: "Loyer mai 2026".to_string(),
            end_to_end_id: None,
            transaction_id: Some("BANK-TX-42".to_string()),
            counterparty_iban: Some("CH9300762011623852957".to_string()),
            counterparty_name: Some("Acme SA".to_string()),
        }
    }

    #[test]
    fn from_owned_imported_transaction() {
        let tx = fixture();
        let draft: BankTransactionDraft = tx.into();
        assert_eq!(
            draft.booking_date,
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap()
        );
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
        assert_eq!(draft.currency, "CHF");
        assert_eq!(draft.reference.as_deref(), Some("RF18539007547034"));
        assert_eq!(draft.transaction_id.as_deref(), Some("BANK-TX-42"));
    }

    #[test]
    fn from_borrowed_imported_transaction_does_not_consume() {
        let tx = fixture();
        let draft: BankTransactionDraft = (&tx).into();
        // tx still usable after the conversion.
        assert_eq!(tx.amount, dec!(1234.56));
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
    }

    #[test]
    fn negative_amount_preserved_as_signed_money() {
        let tx = ImportedTransaction {
            amount: dec!(-50.00),
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.amount.is_negative());
        assert_eq!(draft.amount, Money::new(dec!(-50.00)));
    }

    #[test]
    fn missing_optional_fields_propagate_as_none() {
        let tx = ImportedTransaction {
            value_date: None,
            reference: None,
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.value_date.is_none());
        assert!(draft.reference.is_none());
        assert!(draft.end_to_end_id.is_none());
        assert!(draft.transaction_id.is_none());
        assert!(draft.counterparty_iban.is_none());
        assert!(draft.counterparty_name.is_none());
    }

    fn statement_fixture(opening: Option<Decimal>, closing: Option<Decimal>) -> ImportedStatement {
        ImportedStatement {
            statement_id: Some("STMT-TEST".into()),
            account_iban: "CH4431999123000889012".into(),
            currency: "CHF".into(),
            opening_balance: opening,
            closing_balance: closing,
            period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            transactions: vec![ImportedTransaction {
                amount: dec!(100.00),
                ..fixture()
            }],
            source_format: SourceFormat::Camt053 {
                version: "001.04".into(),
            },
        }
    }

    #[test]
    fn validate_balance_passes_when_within_tolerance() {
        // Test du chemin Ok pour deux valeurs de diff sous la borne :
        //   - diff = 0    (cas idéal : opening + Σ = closing)
        //   - diff = 0.005 (sous tolérance, pas à la borne)
        //
        // **Note review code Pass 2 finding F17** : ce test seul est
        // vacuux (une impl `Ok(())` constante le passerait). Il faut
        // le lire en paire avec :
        //   - `validate_balance_passes_at_exactly_one_cent` (borne =)
        //   - `validate_balance_fails_just_above_one_cent` (borne +ε)
        //   - `validate_balance_fails_when_diff_exceeds_one_cent` (loin)
        // Ensemble, ces 4 tests verrouillent la sémantique `> 0.01`
        // strict avec la borne incluse.

        // opening 1000.00 + Σ 100.00 = closing 1100.00 → diff 0
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        validate_balance(&stmt).expect("balance check passes");

        // diff = 0.005 < 0.01 tolerance
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.005)));
        validate_balance(&stmt).expect("balance check passes within tolerance");
    }

    #[test]
    fn validate_balance_fails_when_diff_exceeds_one_cent() {
        // opening 1000 + 100 = 1100, but closing = 1500 → diff 400
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1500.00)));
        let err = validate_balance(&stmt).expect_err("balance check fails");
        match err {
            CoreError::BankImportBalanceMismatch {
                opening,
                closing,
                sum,
                diff,
            } => {
                assert_eq!(opening, Money::new(dec!(1000.00)));
                assert_eq!(closing, Money::new(dec!(1500.00)));
                assert_eq!(sum, Money::new(dec!(100.00)));
                assert_eq!(diff, Money::new(dec!(400.00)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_balance_skipped_when_balances_missing() {
        // opening absent → skip
        let stmt = statement_fixture(None, Some(dec!(1100.00)));
        validate_balance(&stmt).expect("skip silencieux si opening absent");

        // closing absent → skip
        let stmt = statement_fixture(Some(dec!(1000.00)), None);
        validate_balance(&stmt).expect("skip silencieux si closing absent");

        // les deux absents → skip
        let stmt = statement_fixture(None, None);
        validate_balance(&stmt).expect("skip silencieux si les deux absents");
    }

    #[test]
    fn validate_balance_passes_at_exactly_one_cent() {
        // Régression review code Pass 1 finding F10 : la borne `> 0.01`
        // (strict) accepte un écart **exactement** égal à 0.01. Ce test
        // verrouille la sémantique inclusive de la tolérance contre une
        // régression `>` → `>=`.
        // opening 1000 + Σ 100 = 1100, closing = 1100.01 → diff = 0.01
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.01)));
        validate_balance(&stmt)
            .expect("écart de 0.01 exactement doit passer (tolérance inclusive)");
    }

    #[test]
    fn validate_balance_fails_just_above_one_cent() {
        // Régression review code Pass 1 finding F10 : juste au-dessus
        // de 0.01 (0.011) doit échouer. Verrouille la sémantique
        // contre une régression `>` → `<`.
        // opening 1000 + Σ 100 = 1100, closing = 1100.011 → diff = 0.011
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.011)));
        let err = validate_balance(&stmt).expect_err("écart de 0.011 doit échouer");
        match err {
            CoreError::BankImportBalanceMismatch { diff, .. } => {
                assert_eq!(diff, Money::new(dec!(0.011)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_balance_handles_signed_mix_correctly() {
        // Régression review code Pass 2 finding F19 : verrouille la
        // somme signée des transactions (CRDT positif + DBIT négatif).
        // Sans ça, un bug de `sum_transactions` qui sommerait des
        // valeurs absolues au lieu des valeurs signées resterait
        // invisible — tous les autres tests utilisent uniquement
        // amount = +100.
        // opening 1000 + (+200) + (-50) + (+30) = 1180 = closing 1180 → diff 0
        let mut stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1180.00)));
        stmt.transactions = vec![
            ImportedTransaction {
                amount: dec!(200.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(-50.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(30.00),
                ..fixture()
            },
        ];
        validate_balance(&stmt)
            .expect("somme signée 200 + (-50) + 30 = 180 doit aligner avec 1000 → 1180");
    }

    #[test]
    fn validate_balance_detects_balance_mismatch_fixture() {
        // Régression review code Pass 2 finding F16 (verdict NO-GO
        // conditionnel Auditor) : la fixture `v04_balance_mismatch.xml`
        // était orpheline — aucun test d'intégration ne la chargeait,
        // ce qui rendait AC #9 PARTIAL. Ce test parse la fixture via
        // `kesh-import` et vérifie que `validate_balance` détecte
        // l'incohérence (opening 1000 + Σ 100 = 1100 ≠ closing 1500
        // → diff 400).
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-import/tests/fixtures/camt053/v04_balance_mismatch.xml");
        let xml = std::fs::read(&fixture_path)
            .unwrap_or_else(|e| panic!("fixture {} introuvable : {e}", fixture_path.display()));
        let stmts = kesh_import::parse_camt053(&xml).expect("parse OK");
        assert_eq!(stmts.len(), 1);
        let s = &stmts[0];
        assert_eq!(s.opening_balance, Some(dec!(1000.00)));
        assert_eq!(s.closing_balance, Some(dec!(1500.00)));
        assert_eq!(s.transactions.len(), 1);
        assert_eq!(s.transactions[0].amount, dec!(100.00));

        let err = validate_balance(s).expect_err("balance check doit fail");
        match err {
            CoreError::BankImportBalanceMismatch { diff, .. } => {
                assert_eq!(diff, Money::new(dec!(400.00)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_currency_chf_passes() {
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        validate_currency_supported_v0_1(&stmt).expect("CHF accepté");
    }

    #[test]
    fn validate_currency_eur_rejected_v0_1() {
        let mut stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        stmt.currency = "EUR".into();
        let err = validate_currency_supported_v0_1(&stmt).expect_err("EUR rejeté v0.1");
        match err {
            CoreError::BankImportUnsupportedCurrency(c) => assert_eq!(c, "EUR"),
            other => panic!("attendu BankImportUnsupportedCurrency, obtenu {other:?}"),
        }
    }

    #[test]
    fn from_imported_injects_fk() {
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        let imported_at = NaiveDate::from_ymd_opt(2026, 5, 4)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let (draft, txs) = from_imported(
            &stmt,
            42, // bank_account_id
            7,  // company_id
            "abcdef0123456789".into(),
            "stmt.xml".into(),
            imported_at,
            13, // imported_by_user_id
        )
        .expect("conversion OK");

        assert_eq!(draft.bank_account_id, 42);
        assert_eq!(draft.company_id, 7);
        assert_eq!(draft.file_hash, "abcdef0123456789");
        assert_eq!(draft.filename, "stmt.xml");
        assert_eq!(draft.imported_at, imported_at);
        assert_eq!(draft.imported_by_user_id, 13);
        assert_eq!(draft.source_format, SourceFormatTag::Camt053V04);
        assert_eq!(draft.source_format.as_db_str(), "CAMT053_V04");
        assert_eq!(draft.statement_id.as_deref(), Some("STMT-TEST"));
        assert_eq!(draft.opening_balance, Some(Money::new(dec!(1000.00))));
        assert_eq!(draft.closing_balance, Some(Money::new(dec!(1100.00))));
        assert_eq!(draft.transaction_count, 1);
        assert_eq!(txs.len(), 1);
    }

    #[test]
    fn from_imported_returns_one_draft_per_transaction() {
        let mut stmt = statement_fixture(None, None);
        stmt.transactions = vec![
            ImportedTransaction {
                amount: dec!(10.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(20.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(-5.00),
                ..fixture()
            },
        ];
        let (draft, txs) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("conversion OK");

        assert_eq!(txs.len(), 3);
        assert_eq!(draft.transaction_count, 3);
        assert_eq!(txs[0].amount, Money::new(dec!(10.00)));
        assert_eq!(txs[1].amount, Money::new(dec!(20.00)));
        assert_eq!(txs[2].amount, Money::new(dec!(-5.00)));
    }

    #[test]
    fn from_imported_unknown_version_returns_err() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Camt053 {
            version: "001.99".into(),
        };
        let err = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect_err("version inconnue rejetée");
        match err {
            CoreError::BankImportUnknownVersion(v) => assert_eq!(v, "001.99"),
            other => panic!("attendu BankImportUnknownVersion, obtenu {other:?}"),
        }
    }

    #[test]
    fn from_imported_v08_maps_to_correct_tag() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Camt053 {
            version: "001.08".into(),
        };
        let (draft, _) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("v08 OK");
        assert_eq!(draft.source_format, SourceFormatTag::Camt053V08);
        assert_eq!(draft.source_format.as_db_str(), "CAMT053_V08");
    }

    /// Pass 1 review G1 AA-1 : Story 8-2 T5.0.b — vérifie que le branch
    /// `SourceFormat::Csv { .. }` retourne `Ok(SourceFormatTag::Csv)`
    /// (vs ancien `Err(BankImportUnknownVersion("csv"))` du 8-1a).
    #[test]
    fn from_imported_csv_maps_to_csv_tag() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Csv {
            encoding: "UTF-8".into(),
            profile_name: Some("UBS".into()),
        };
        let (draft, _) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("csv branch OK");
        assert_eq!(draft.source_format, SourceFormatTag::Csv);
        assert_eq!(draft.source_format.as_db_str(), "CSV");
    }
}
