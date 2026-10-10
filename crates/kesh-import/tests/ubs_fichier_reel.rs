//! Contrôle local du parseur CSV sur un **vrai** relevé UBS, hors dépôt
//! (Story 26-1, AC17, C-26-1-1).
//!
//! ⛔ **Données personnelles.** Le relevé n'est jamais copié dans le dépôt :
//! son chemin se passe par la variable `KESH_UBS_REAL_CSV`. Ce test
//! n'imprime **que** « conforme », ou le nom de l'assertion qui a échoué —
//! jamais un montant, une somme, un solde, un nombre de mouvements, une
//! ligne, un libellé ni un IBAN, **même en échec** : aucun `assert_eq!` sur
//! une valeur lue (il imprimerait les deux côtés), aucune valeur
//! interpolée, et les erreurs du parseur s'affichent par leur code et leur
//! numéro de ligne seulement.
//!
//! Ignoré par défaut (hors CI) ; lancé au gate de la story :
//!
//! ```sh
//! KESH_UBS_REAL_CSV=/chemin/vers/transactions.csv \
//!   cargo test -p kesh-import --test ubs_fichier_reel -- --ignored --nocapture
//! ```
//!
//! Il vérifie, en lisant **lui-même** le préambule du fichier : import sans
//! erreur avec le profil UBS recommandé (C-26-1-13) ; nombre de transactions
//! = « Nombre de transactions dans cette période » ; somme des montants
//! signés = solde final − solde initial, au centime ; chaque transaction
//! issue de la colonne Débit est négative, chaque transaction issue de la
//! colonne Crédit positive.

use kesh_import::{ColumnMapping, CsvProfile, DebitSign, ParseCsvOutcome, parse_csv_collect};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Profil UBS recommandé (C-26-1-13).
fn ubs_profile() -> CsvProfile {
    CsvProfile {
        bank_name: "UBS".to_string(),
        filename_pattern: None,
        column_mapping: ColumnMapping {
            date: 2,
            amount: None,
            debit_credit_split: Some((5, 6)),
            reference: Some(9),
            details: Some(10),
            counterparty: None,
        },
        date_format: "%Y-%m-%d".to_string(),
        decimal_separator: '.',
        field_separator: ';',
        encoding: None,
        header_row_count: 9,
        debit_sign: DebitSign::Negative,
    }
}

/// Valeur brute d'une clé du préambule (`Clé:;valeur;`). Panique sans
/// citer la valeur.
fn preamble(text: &str, key: &str) -> String {
    text.lines()
        .take(9)
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split(';').nth(1))
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| panic!("préambule : clé « {key} » absente"))
}

/// Lit un montant du préambule (séparateurs de milliers tolérés).
fn preamble_amount(text: &str, key: &str) -> Decimal {
    let raw = preamble(text, key).replace(['\'', ' ', '\u{00A0}'], "");
    Decimal::from_str(&raw)
        .unwrap_or_else(|_| panic!("préambule : « {key} » n'est pas un montant lisible"))
}

#[test]
#[ignore = "lit un relevé réel hors dépôt (KESH_UBS_REAL_CSV) — gate local de la Story 26-1"]
fn ubs_real_statement_is_conform() {
    let path = std::env::var("KESH_UBS_REAL_CSV").unwrap_or_else(|_| {
        panic!(
            "KESH_UBS_REAL_CSV absente : ce test ne passe pas à vide, donner le chemin du relevé"
        )
    });
    let bytes =
        std::fs::read(&path).unwrap_or_else(|_| panic!("KESH_UBS_REAL_CSV : fichier illisible"));
    let body = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    let text = std::str::from_utf8(body).expect("relevé non UTF-8");

    let stmt = match parse_csv_collect(&bytes, &ubs_profile()) {
        ParseCsvOutcome::AllValid(stmt) => stmt,
        ParseCsvOutcome::PartialFailure { errors, .. } => {
            let where_: Vec<String> = errors
                .iter()
                .map(|e| format!("{} ligne {}", e.code.as_str(), e.line))
                .collect();
            panic!("erreurs de ligne : {}", where_.join(", "));
        }
        ParseCsvOutcome::HardFailure(e) => {
            // Le Display des variants ne porte ni montant ni libellé, sauf
            // les erreurs de ligne embarquées, réduites ici à code + ligne.
            let detail = match &e {
                kesh_import::CsvError::AmountSignMismatch { errors, .. } => errors
                    .iter()
                    .map(|l| format!("{} ligne {}", l.code.as_str(), l.line))
                    .collect::<Vec<_>>()
                    .join(", "),
                kesh_import::CsvError::HeaderSkipsData { line } => format!("ligne {line}"),
                _ => "refus avant la lecture des lignes".to_string(),
            };
            panic!("import refusé : {detail}");
        }
    };

    let announced: usize = preamble(text, "Nombre de transactions")
        .parse()
        .unwrap_or_else(|_| panic!("préambule : nombre de transactions illisible"));
    assert!(
        stmt.transactions.len() == announced,
        "nombre ≠ nombre annoncé par le préambule"
    );

    let sum: Decimal = stmt.transactions.iter().map(|t| t.amount).sum();
    let expected = preamble_amount(text, "Solde final:") - preamble_amount(text, "Solde initial:");
    assert!(sum == expected, "somme ≠ solde final − solde initial");

    // Répartition : relire les lignes de données pour savoir de quelle
    // colonne vient chaque transaction (dans l'ordre du fichier).
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .from_reader(text.as_bytes());
    let rows: Vec<csv::StringRecord> = reader
        .records()
        .skip(8)
        .collect::<Result<_, _>>()
        .unwrap_or_else(|_| panic!("relecture des lignes impossible"));
    assert!(
        rows.len() == stmt.transactions.len(),
        "relecture : nombre de lignes ≠ nombre de transactions"
    );
    for (row, tx) in rows.iter().zip(&stmt.transactions) {
        let from_debit = !row.get(5).unwrap_or("").trim().is_empty();
        if from_debit {
            assert!(tx.amount < Decimal::ZERO, "un débit n'est pas négatif");
        } else {
            assert!(tx.amount > Decimal::ZERO, "un crédit n'est pas positif");
        }
    }

    println!("conforme");
}
