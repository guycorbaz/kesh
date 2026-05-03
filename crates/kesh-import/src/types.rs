//! Types autonomes représentant un relevé bancaire importé.
//!
//! Aucun champ n'est lié au modèle de persistance Kesh (pas de `bank_account_id`,
//! `import_id`, `company_id`). Ces clés étrangères sont assignées par
//! `kesh-core` ou `kesh-api` lors de la conversion vers les types domaine, à
//! partir du contexte de la requête HTTP (compte bancaire sélectionné par
//! l'utilisateur, tenant courant, hash du fichier).
//!
//! Les types sont volontairement permissifs sur les contenus opaques (IBAN,
//! références) : la validation stricte (checksum IBAN ISO 13616, format des
//! références ESR) appartient à `kesh-core`. Le rôle du parseur est d'extraire
//! les données telles qu'elles apparaissent dans le fichier source, sans les
//! rejeter prématurément. Un fichier mal formé (XML invalide, colonnes CSV
//! manquantes) doit échouer à la frontière du parseur ; un fichier
//! syntaxiquement valide mais contenant un IBAN au checksum incorrect doit
//! produire un [`ImportedTransaction`] que `kesh-core` rejettera ensuite avec
//! un message métier explicite.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Format source du relevé importé.
///
/// Permet aux consommateurs de tracer la provenance (audit) et de produire
/// des messages d'erreur adaptés (« CAMT.053 v04 mal formé » vs « ligne CSV
/// invalide »).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceFormat {
    /// Relevé CAMT.053 ISO 20022. `version` reflète le suffixe de schéma
    /// (ex. `"001.04"` pour `camt.053.001.04`).
    Camt053 { version: String },
    /// Relevé CSV. `encoding` est l'encodage détecté (`"UTF-8"`,
    /// `"ISO-8859-1"`). `profile_name` référence le profil banque appliqué
    /// (Story 8-2), `None` si aucun profil n'a matché.
    Csv {
        encoding: String,
        profile_name: Option<String>,
    },
}

/// Une transaction unitaire telle qu'extraite d'un relevé.
///
/// Le signe d'`amount` suit la convention CAMT.053 du point de vue du compte
/// titulaire : positif pour un crédit (entrée), négatif pour un débit (sortie).
/// Les fichiers CSV sont normalisés vers cette convention par le profil banque
/// (Story 8-2).
///
/// Les champs optionnels reflètent les variations entre versions CAMT.053 et
/// entre banques : `value_date` peut manquer dans certaines exports CSV ;
/// `end_to_end_id` n'est présent que sur les paiements SEPA structurés ;
/// `counterparty_iban` peut être omis si la banque ne le transmet pas (espèces).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportedTransaction {
    /// Date de comptabilisation (`<BookgDt>` CAMT.053).
    pub booking_date: NaiveDate,
    /// Date valeur (`<ValDt>`), si présente.
    pub value_date: Option<NaiveDate>,
    /// Montant signé (positif = crédit titulaire, négatif = débit titulaire).
    pub amount: Decimal,
    /// Code devise ISO 4217 (`"CHF"`, `"EUR"`).
    pub currency: String,
    /// Référence structurée (ESR/QR-Référence pour les CH, `<CdtrRefInf><Ref>`
    /// SEPA) si présente.
    pub reference: Option<String>,
    /// Détails libres (`<RmtInf><Ustrd>` ou champ texte CSV).
    pub details: String,
    /// Identifiant end-to-end fourni par le donneur d'ordre
    /// (`<EndToEndId>` SEPA), si présent.
    pub end_to_end_id: Option<String>,
    /// Identifiant unique attribué par la banque
    /// (`<AcctSvcrRef>`), si présent. Permet la détection de doublons stricte
    /// (Story 8-3) en complément de la combinaison
    /// `date+montant+référence+compte`.
    pub transaction_id: Option<String>,
    /// IBAN de la contrepartie si présent (chaîne brute, non validée). La
    /// validation MOD-97 et le format ISO 13616 sont du ressort de
    /// `kesh-core::types::Iban` lors de la conversion.
    pub counterparty_iban: Option<String>,
    /// Nom de la contrepartie (`<Dbtr><Nm>` ou `<Cdtr><Nm>`), si présent.
    pub counterparty_name: Option<String>,
}

/// Un relevé bancaire complet (un fichier CAMT.053 ou un fichier CSV).
///
/// Un même fichier CAMT.053 peut contenir plusieurs `<Stmt>`, chacun pour un
/// compte distinct. Le parseur émet alors un [`ImportedStatement`] par
/// `<Stmt>`. Pour les CSV, un fichier = un relevé.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportedStatement {
    /// Identifiant du relevé (`<Id>` CAMT.053). `None` pour les CSV qui n'en
    /// fournissent pas.
    pub statement_id: Option<String>,
    /// IBAN du compte titulaire (chaîne brute, non validée).
    pub account_iban: String,
    /// Devise du compte (ISO 4217).
    pub currency: String,
    /// Solde d'ouverture (`<OpngBal>` CAMT.053). Optionnel pour CSV.
    pub opening_balance: Option<Decimal>,
    /// Solde de clôture (`<ClsgBal>` CAMT.053). Optionnel pour CSV.
    /// Comparé à la somme algébrique des transactions pour détecter les
    /// fichiers tronqués (CR-010 #62, à intégrer Story 8-1).
    pub closing_balance: Option<Decimal>,
    /// Début de la période couverte (`<FrToDt><FrDtTm>`).
    pub period_from: NaiveDate,
    /// Fin de la période couverte (`<FrToDt><ToDtTm>`).
    pub period_to: NaiveDate,
    /// Liste des transactions du relevé.
    pub transactions: Vec<ImportedTransaction>,
    /// Format source et métadonnées de parsing.
    pub source_format: SourceFormat,
}

impl ImportedStatement {
    /// Calcule la somme algébrique des montants des transactions du relevé.
    ///
    /// Sert au check « solde de clôture cohérent » prévu par CR-010 (#62) :
    /// `opening_balance + sum_transactions ≈ closing_balance` à la tolérance
    /// d'arrondi près (0.01 unité). L'implémentation effective de la
    /// vérification appartient à Story 8-1.
    pub fn sum_transactions(&self) -> Decimal {
        self.transactions.iter().map(|tx| tx.amount).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn sample_transaction() -> ImportedTransaction {
        ImportedTransaction {
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 3).unwrap()),
            amount: dec!(1234.56),
            currency: "CHF".to_string(),
            reference: Some("RF18539007547034".to_string()),
            details: "Loyer mai 2026".to_string(),
            end_to_end_id: Some("E2E-2026-05-001".to_string()),
            transaction_id: Some("BANK-TX-42".to_string()),
            counterparty_iban: Some("CH9300762011623852957".to_string()),
            counterparty_name: Some("Acme SA".to_string()),
        }
    }

    fn sample_statement() -> ImportedStatement {
        ImportedStatement {
            statement_id: Some("STMT-2026-05".to_string()),
            account_iban: "CH4431999123000889012".to_string(),
            currency: "CHF".to_string(),
            opening_balance: Some(dec!(10000.00)),
            closing_balance: Some(dec!(11234.56)),
            period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            transactions: vec![sample_transaction()],
            source_format: SourceFormat::Camt053 {
                version: "001.04".to_string(),
            },
        }
    }

    #[test]
    fn transaction_serde_roundtrip() {
        let tx = sample_transaction();
        let json = serde_json::to_string(&tx).unwrap();
        let back: ImportedTransaction = serde_json::from_str(&json).unwrap();
        assert_eq!(tx, back);
    }

    #[test]
    fn statement_serde_roundtrip() {
        let stmt = sample_statement();
        let json = serde_json::to_string(&stmt).unwrap();
        let back: ImportedStatement = serde_json::from_str(&json).unwrap();
        assert_eq!(stmt, back);
    }

    #[test]
    fn source_format_camt053_serializes_as_tagged_enum() {
        let fmt = SourceFormat::Camt053 {
            version: "001.08".to_string(),
        };
        let json = serde_json::to_string(&fmt).unwrap();
        assert!(json.contains(r#""kind":"camt053""#));
        assert!(json.contains(r#""version":"001.08""#));
    }

    #[test]
    fn source_format_csv_carries_encoding_and_profile() {
        let fmt = SourceFormat::Csv {
            encoding: "ISO-8859-1".to_string(),
            profile_name: Some("UBS".to_string()),
        };
        let json = serde_json::to_string(&fmt).unwrap();
        let back: SourceFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(fmt, back);
    }

    #[test]
    fn amount_preserves_decimal_precision() {
        // rust_decimal preserves trailing zeros via serde-str.
        let tx = ImportedTransaction {
            amount: dec!(0.01),
            ..sample_transaction()
        };
        let json = serde_json::to_string(&tx).unwrap();
        assert!(json.contains(r#""amount":"0.01""#));
    }

    #[test]
    fn sum_transactions_matches_expected_delta() {
        let mut stmt = sample_statement();
        stmt.transactions.push(ImportedTransaction {
            amount: dec!(-100.00),
            ..sample_transaction()
        });
        // 1234.56 + (-100.00) = 1134.56
        assert_eq!(stmt.sum_transactions(), dec!(1134.56));
    }

    #[test]
    fn negative_amount_signals_debit() {
        let tx = ImportedTransaction {
            amount: dec!(-50.00),
            ..sample_transaction()
        };
        assert!(tx.amount.is_sign_negative());
    }
}
