//! Route `POST /api/v1/wise-imports` — Story 39-11b (Epic 39, phase 1).
//!
//! Importe d'un geste un ou plusieurs relevés CSV d'un **compte Wise**
//! (multipart : un ou plusieurs `file`, `account` = le libellé). Un `file`
//! peut être le **zip** que Wise fait télécharger (Story 39-11e) : reconnu à
//! sa signature, quel que soit son nom, il est remplacé, à sa place, par ses
//! CSV à la racine (par nom) — bornes et refus dans [`crate::zip_read`] —, et
//! la suite est celle d'un fichier envoyé à part. Wise exporte
//! un relevé **par solde** : chaque fichier porte une devise (lecteur de la
//! Story 39-11a) et va au compte bancaire Kesh, solde de ce compte Wise dans
//! cette devise (`bank_accounts.provider = 'wise'`, `provider_account` = le
//! libellé, devise = celle du compte du plan lié).
//!
//! Comptable+, clés d'API `read-write` admises, comme `POST
//! /api/v1/bank-imports`. Taille maximale du corps entier : celle de l'import
//! bancaire.
//!
//! **Ordre des refus**, rien d'écrit à aucun : `400 VALIDATION_ERROR` (champ
//! manquant, aucun fichier) → `404 PROVIDER_ACCOUNT_NOT_FOUND` → `422
//! WISE_ZIP_INVALID` (première archive refusée, dans l'ordre du formulaire)
//! → `422 WISE_CSV_INVALID` ou `422 WISE_CSV_FEES_NOT_SEPARATED` (premier
//! fichier refusé par le lecteur, nommé, avec son archive) → `422
//! WISE_DUPLICATE_CURRENCY_FILE` → `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` →
//! `409 PROVIDER_CURRENCY_DUPLICATE` → `409 BANK_ACCOUNT_CLOSED` → `422
//! BANK_IMPORT_DUPLICATE_FILE` (jugé sous les verrous, dans la transaction
//! d'écriture). Un verrou de compte non obtenu rend `409
//! RECONCILIATION_ACCOUNT_LOCKED`.
//!
//! **Rangement** : par le cœur commun des imports de prestataire
//! ([`import_lines`], Story 38-12a ; Wise y passe depuis la Story 39-14), un
//! relevé par devise ([`ImportSource::Statements`], dans l'ordre du formulaire)
//! — verrous de chaque compte visé par identifiant croissant, une seule
//! transaction pour tous les fichiers (tout ou rien), relecture verrouillante
//! des soldes, empreinte, rejeu sur interblocage (opération
//! `wise_imports::create`). Par fichier, une entrée `bank_imports` (`CSV`,
//! période `first_date..last_date`, soldes d'ouverture et de clôture du
//! relevé) ; une ligne = une transaction ([`line_of_row`]) ; `Total fees`
//! n'en est jamais une (les frais sont leurs propres lignes).
//!
//! **Doublons** : une ligne dont l'identifiant Wise est déjà sur le compte
//! bancaire **dans le même sens** est ignorée et comptée (la fenêtre
//! date-montant-référence des imports bancaires ne s'applique pas) ; un
//! fichier tout en doublons ne crée pas d'entrée. La clé est (identifiant,
//! sens), comme au lecteur : Wise écrit sous un même identifiant le débit puis
//! le recrédit d'une réserve de carte expirée ou d'une vérification de carte
//! (Story 39-11a2 ; `ProviderLine::direction`, jugée par
//! `provider_imports::ExistingKeys`). **Continuité** : avertissement `gap`
//! quand le relevé ne prolonge pas le précédent du compte
//! (`kesh_db::repositories::bank_imports::continuity_gap`, la règle unique
//! que relit aussi la revue de la Story 39-11c, appelée par le cœur) ;
//! l'import passe.
//!
//! **Trace** `wise_import.created` : `provider`, `provider_account`, `files`
//! (le compte rendu de chaque fichier, tel que la réponse) et
//! `ignoredEntries`.
//!
//! ⚠️ Les règles de doublon propres à Wise R1 à R3 (frais de conversion inclus,
//! conversion automatique) n'ont d'objet qu'avec le téléchargement par l'API,
//! sorti de l'epic (branche `futur/wise-api`) : sans ligne téléchargée en base,
//! aucune ne se déclenche, et le cœur commun n'en porte pas encore le point
//! d'accroche.

use std::collections::BTreeMap;

use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::Serialize;

use kesh_db::entities::bank_account::PROVIDER_WISE;
use kesh_db::entities::bank_transaction::NewBankTransaction;
use kesh_import::wise::{WiseCsv, WiseCsvError, WiseDirection, WiseRow, parse_wise_csv};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::provider_imports::{
    CurrencyReport, CurrencyStatement, DETAILS_MAX_CHARS, FILENAME_MAX_BYTES, ImportRequest,
    ImportSource, LineDirection, NAME_MAX_CHARS, PROVIDER_TYPE_MAX_CHARS, ProviderGroup,
    ProviderLine, TXID_COLUMN_MAX_CHARS, import_lines, load_balances, truncate_bytes,
    truncate_chars,
};
pub use crate::routes::provider_imports::{ImportWarning as FileWarning, WARNING_GAP};
use crate::zip_read::{ZipRefusal, ZipRefusalReason, extract_csv_entries, looks_like_zip};

/// Nom d'un fichier sans nom dans le formulaire.
const DEFAULT_FILENAME: &str = "wise.csv";
/// Longueur maximale d'un IBAN (`counterparty_iban VARCHAR(34)`).
const IBAN_MAX_CHARS: usize = 34;
/// Séparateur des éléments du libellé.
const DETAILS_SEPARATOR: &str = " · ";
/// Nombre maximal d'entrées d'un zip de relevés (Story 39-11e, D4) : trois
/// aujourd'hui, une par solde ; la marge couvre un compte à nombreuses devises.
pub const WISE_ZIP_MAX_ENTRIES: usize = 64;

// ---------------------------------------------------------------------------
// DTO
// ---------------------------------------------------------------------------

/// Compte rendu d'un fichier : son compte bancaire, l'entrée `bank_imports`
/// créée (`null` si toutes ses lignes étaient des doublons), les nombres de
/// transactions insérées et ignorées, ses soldes et ses avertissements.
///
/// Story 39-11e : `archive`, le zip d'où vient le fichier (`null` pour un
/// fichier envoyé à part), et `rows`, les lignes lues (`inserted +
/// duplicates`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReport {
    pub filename: String,
    pub archive: Option<String>,
    pub currency: String,
    pub bank_account_id: i64,
    pub import_id: Option<i64>,
    pub rows: usize,
    pub inserted: usize,
    pub duplicates: usize,
    pub opening_balance: Decimal,
    pub closing_balance: Decimal,
    pub warnings: Vec<FileWarning>,
}

/// Une entrée d'un zip que l'import n'a pas lue (Story 39-11e, D3) : un
/// dossier, une entrée dans un dossier, une autre extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredEntry {
    pub archive: String,
    pub entry: String,
}

/// Réponse `201` de l'import (AC-5 ; `ignoredEntries`, toujours présent, vide
/// sans zip : Story 39-11e).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WiseImportResponse {
    pub account: String,
    pub files: Vec<FileReport>,
    pub ignored_entries: Vec<IgnoredEntry>,
}

// ---------------------------------------------------------------------------
// Une ligne → une transaction (pure)
// ---------------------------------------------------------------------------

/// Le texte rogné d'une cellule facultative, `None` s'il est vide.
fn non_empty(value: &Option<String>) -> Option<&str> {
    value.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Un numéro de compte en **forme d'IBAN** — Story 39-11b (AC-4) : sans
/// espaces, en majuscules, deux lettres, deux chiffres puis 11 à 30
/// caractères alphanumériques ASCII, 34 au plus. Sinon `None` (les relevés
/// portent aussi des numéros nationaux et des numéros masqués d'étoiles).
/// Contrôle de forme seul, sans clé de contrôle : le numéro est celui que Wise
/// a écrit.
pub fn iban_shaped(account: &str) -> Option<String> {
    let compact: String = account
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let bytes = compact.as_bytes();
    let shaped = (15..=IBAN_MAX_CHARS).contains(&bytes.len())
        && bytes[..2].iter().all(u8::is_ascii_uppercase)
        && bytes[2..4].iter().all(u8::is_ascii_digit)
        && bytes[4..].iter().all(u8::is_ascii_alphanumeric);
    shaped.then_some(compact)
}

/// La ligne neutre ([`ProviderLine`]) d'une ligne du relevé Wise — Story
/// 39-11b (AC-4), pour le cœur commun depuis la Story 39-14 : montant
/// signé, date de la ligne, identifiant Wise, référence de paiement,
/// contrepartie (`Merchant`, sinon `Payee Name` d'un débit ou `Payer Name`
/// d'un crédit), IBAN du bénéficiaire s'il en a la forme, type = préfixe de
/// l'identifiant (filtre des règles, G5), libellé = description, type de
/// détail, numéro de compte non IBAN, conversion et note, joints par « · » et
/// tronqués à la frontière d'un caractère. Un groupe = une ligne
/// (`group_id` = identifiant Wise), sens du lecteur (doublon sur
/// (identifiant, sens), Story 39-11a2).
pub fn line_of_row(row: &WiseRow) -> ProviderLine {
    let account = non_empty(&row.payee_account);
    let iban = account.and_then(iban_shaped);
    let other_account = if iban.is_none() { account } else { None };
    let exchange = match (non_empty(&row.exchange_from), non_empty(&row.exchange_to)) {
        (Some(from), Some(to)) => {
            // Segment fixe et relisible (Story 39-11c) :
            // « DE→VERS [MONTANT] [au cours COURS] », le montant étant celui
            // reçu dans la devise VERS (`Exchange To Amount`). Aucun champ
            // structuré de `bank_transactions` ne le porte ; il n'existe qu'au
            // CSV, qui n'est pas conservé.
            let mut segment = format!("{from}→{to}");
            if let Some(to_amount) = row.exchange_to_amount {
                segment.push_str(&format!(" {to_amount}"));
            }
            if let Some(rate) = row.exchange_rate {
                segment.push_str(&format!(" au cours {rate}"));
            }
            Some(segment)
        }
        _ => None,
    };
    // La tête (description, type de détail, numéro) est rognée avant le segment
    // de conversion : une longue description ne doit pas faire perdre le
    // montant de la conversion (revue P2, L1). La note, en queue, reste rognée
    // la première par la troncature finale.
    let head = [
        non_empty(&row.description).map(str::to_string),
        Some(row.details_type.trim().to_string()).filter(|s| !s.is_empty()),
        other_account.map(str::to_string),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(DETAILS_SEPARATOR);
    let head = match &exchange {
        Some(segment) => {
            let reserved = DETAILS_SEPARATOR.chars().count() + segment.chars().count();
            truncate_chars(&head, DETAILS_MAX_CHARS.saturating_sub(reserved))
        }
        None => head,
    };
    let details = [
        Some(head).filter(|s| !s.is_empty()),
        exchange,
        non_empty(&row.note).map(str::to_string),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(DETAILS_SEPARATOR);
    let counterparty = non_empty(&row.merchant).or(match row.direction {
        WiseDirection::Debit => non_empty(&row.payee_name),
        WiseDirection::Credit => non_empty(&row.payer_name),
    });
    ProviderLine {
        group_id: row.id.clone(),
        transaction_id: row.id.clone(),
        booking_date: row.date,
        amount: row.amount,
        currency: row.currency.clone(),
        reference: non_empty(&row.payment_reference)
            .map(|s| truncate_chars(s, TXID_COLUMN_MAX_CHARS)),
        details: truncate_chars(&details, DETAILS_MAX_CHARS),
        end_to_end_id: None,
        counterparty_name: counterparty.map(|s| truncate_chars(s, NAME_MAX_CHARS)),
        counterparty_email: None,
        provider_type: Some(truncate_chars(&row.id_prefix, PROVIDER_TYPE_MAX_CHARS)),
        rank: row.line,
        // Les soldes de l'entrée sont ceux du relevé (`CurrencyStatement`).
        balance_after: Some(row.running_balance),
        value_date: None,
        counterparty_iban: iban,
        direction: Some(match row.direction {
            WiseDirection::Debit => LineDirection::Debit,
            WiseDirection::Credit => LineDirection::Credit,
        }),
    }
}

/// La transaction bancaire d'une ligne du relevé ([`line_of_row`]).
pub fn transaction_of_row(
    row: &WiseRow,
    company_id: i64,
    bank_account_id: i64,
) -> NewBankTransaction {
    line_of_row(row).into_new_bank_transaction(company_id, bank_account_id)
}

// ---------------------------------------------------------------------------
// Multipart
// ---------------------------------------------------------------------------

/// Un fichier du formulaire, ou un CSV extrait d'un zip du formulaire
/// (`archive` = le nom du champ zip).
struct UploadedFile {
    filename: String,
    archive: Option<String>,
    bytes: axum::body::Bytes,
}

struct WiseImportForm {
    files: Vec<UploadedFile>,
    account: String,
}

async fn parse_form(mut multipart: Multipart) -> Result<WiseImportForm, AppError> {
    let mut files: Vec<UploadedFile> = Vec::new();
    let mut account: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart : {e}")))?
    {
        match field.name() {
            Some("file") => {
                let name = field.file_name().unwrap_or(DEFAULT_FILENAME).to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                let filename = truncate_bytes(name.trim(), FILENAME_MAX_BYTES);
                files.push(UploadedFile {
                    filename: if filename.is_empty() {
                        DEFAULT_FILENAME.into()
                    } else {
                        filename
                    },
                    archive: None,
                    bytes,
                });
            }
            Some("account") => {
                if account.is_some() {
                    return Err(AppError::Validation("champ `account` en double".into()));
                }
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                account = Some(text);
            }
            _ => {}
        }
    }
    if files.is_empty() {
        return Err(AppError::Validation("champ `file` manquant".into()));
    }
    let account = account
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Validation("champ `account` manquant".into()))?;
    Ok(WiseImportForm { files, account })
}

/// Traduit le refus du lecteur (39-11a) en `422 WISE_CSV_INVALID`, le fichier
/// nommé — sauf le relevé exporté sans les frais séparés (Story 39-11e, D5),
/// qui a son code propre, `422 WISE_CSV_FEES_NOT_SEPARATED`. L'un et l'autre
/// portent l'archive d'où vient le fichier (`None`, rendu `null`, pour un
/// fichier envoyé à part) : deux zips d'un même envoi peuvent porter des
/// entrées de même nom (Story 39-11d, revue P1, M2).
fn csv_error(filename: &str, archive: Option<&str>, e: WiseCsvError) -> AppError {
    if let WiseCsvError::FeesNotSeparated { lines, count } = e {
        return AppError::WiseCsvFeesNotSeparated {
            filename: filename.to_string(),
            archive: archive.map(str::to_string),
            lines,
            count,
        };
    }
    let reason = e.code();
    let mut details = match &e {
        WiseCsvError::MissingColumns(columns) => serde_json::json!({ "columns": columns }),
        WiseCsvError::DuplicateColumn(column) => serde_json::json!({ "columns": [column] }),
        WiseCsvError::Lines(lines) => serde_json::json!({ "lines": lines }),
        WiseCsvError::Encoding(inner) => serde_json::json!({ "cause": inner.to_string() }),
        WiseCsvError::MixedCurrencies(currencies) => {
            serde_json::json!({ "currencies": currencies })
        }
        WiseCsvError::DuplicateId { id, lines } => serde_json::json!({ "id": id, "lines": lines }),
        WiseCsvError::RunningBalanceBreak {
            line,
            expected,
            found,
        } => serde_json::json!({ "line": line, "expected": expected, "found": found }),
        // `FeesNotSeparated` est rendu plus haut, par son propre code.
        WiseCsvError::Empty | WiseCsvError::FeesNotSeparated { .. } => serde_json::json!({}),
    };
    if let serde_json::Value::Object(map) = &mut details {
        map.insert("archive".into(), serde_json::json!(archive));
    }
    AppError::WiseCsvInvalid {
        filename: filename.to_string(),
        reason,
        details,
    }
}

/// Le nom d'une entrée de zip, borné pour la réponse, la trace et
/// `bank_imports.filename` (Story 39-11e, AC-1 et AC-3) : caractères de
/// contrôle remplacés par `?`, rogné à [`FILENAME_MAX_BYTES`] à la frontière
/// d'un caractère (un nom de zip peut faire 65 535 octets).
fn entry_name(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect();
    truncate_bytes(&clean, FILENAME_MAX_BYTES)
}

/// Le refus d'une archive en `422 WISE_ZIP_INVALID` (Story 39-11e).
fn zip_error(archive: &str, refusal: ZipRefusal, max_bytes: u64) -> AppError {
    let extra = match refusal.reason {
        ZipRefusalReason::TooManyEntries => serde_json::json!({ "max": WISE_ZIP_MAX_ENTRIES }),
        ZipRefusalReason::TooLarge => serde_json::json!({ "maxBytes": max_bytes }),
        _ => serde_json::json!({}),
    };
    AppError::WiseZipInvalid {
        filename: archive.to_string(),
        reason: refusal.reason.as_str(),
        entry: refusal.entry.as_deref().map(entry_name),
        extra,
    }
}

/// Développe les zips du formulaire (Story 39-11e, D1 à D4) : chaque fichier
/// à signature de zip est remplacé, **à sa place**, par ses CSV retenus (par
/// nom, `archive` = son nom) ; les autres restent tels quels. La taille
/// décompressée se cumule sur toutes les archives, bornée par `max_bytes`
/// (la limite du corps de la route). Rend aussi les entrées ignorées.
/// Synchrone : à exécuter en `spawn_blocking`.
fn expand_archives(
    files: Vec<UploadedFile>,
    max_bytes: u64,
) -> Result<(Vec<UploadedFile>, Vec<IgnoredEntry>), AppError> {
    let mut budget = max_bytes;
    let mut out = Vec::with_capacity(files.len());
    let mut ignored = Vec::new();
    for file in files {
        if !looks_like_zip(&file.bytes) {
            out.push(file);
            continue;
        }
        let extracted = extract_csv_entries(&file.bytes, WISE_ZIP_MAX_ENTRIES, &mut budget)
            .map_err(|r| zip_error(&file.filename, r, max_bytes))?;
        ignored.extend(extracted.ignored.iter().map(|entry| IgnoredEntry {
            archive: file.filename.clone(),
            entry: entry_name(entry),
        }));
        out.extend(extracted.csvs.into_iter().map(|csv| UploadedFile {
            filename: entry_name(&csv.name),
            archive: Some(file.filename.clone()),
            bytes: csv.bytes.into(),
        }));
    }
    Ok((out, ignored))
}

/// Le premier couple de fichiers de même devise, dans l'ordre du formulaire
/// (`422 WISE_DUPLICATE_CURRENCY_FILE`, tous les fichiers de cette devise).
fn duplicate_currency(read: &[(String, WiseCsv)]) -> Option<AppError> {
    read.iter().enumerate().find_map(|(i, (_, csv))| {
        let same: Vec<String> = read
            .iter()
            .filter(|(_, other)| other.currency == csv.currency)
            .map(|(name, _)| name.clone())
            .collect();
        let first = read
            .iter()
            .position(|(_, other)| other.currency == csv.currency);
        (same.len() > 1 && first == Some(i)).then(|| AppError::WiseDuplicateCurrencyFile {
            currency: csv.currency.clone(),
            filenames: same,
        })
    })
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// La partie « fichier » du compte rendu d'un fichier : ce qui ne dépend que
/// du fichier lu. Portée à la trace avec le compte rendu de la devise
/// (`CurrencyStatement::trace`), si bien que `files` de la trace égale `files`
/// de la réponse.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FilePart<'a> {
    filename: &'a str,
    archive: Option<&'a str>,
    rows: usize,
    opening_balance: Decimal,
    closing_balance: Decimal,
}

impl FilePart<'_> {
    /// Le compte rendu du fichier, complété de celui de sa devise.
    fn report(&self, currency: CurrencyReport) -> FileReport {
        FileReport {
            filename: self.filename.to_string(),
            archive: self.archive.map(str::to_string),
            currency: currency.currency,
            bank_account_id: currency.bank_account_id,
            import_id: currency.import_id,
            rows: self.rows,
            inserted: currency.inserted,
            duplicates: currency.duplicates,
            opening_balance: self.opening_balance,
            closing_balance: self.closing_balance,
            warnings: currency.warnings.unwrap_or_default(),
        }
    }
}

/// `POST /api/v1/wise-imports` — voir l'en-tête du module.
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<WiseImportResponse>), AppError> {
    let form = parse_form(multipart).await?;
    let balances = load_balances(
        &state.pool,
        current_user.company_id,
        PROVIDER_WISE,
        &form.account,
    )
    .await?;

    let max_bytes = u64::from(state.config.bank_import_max_mib) * 1024 * 1024;
    let (files, ignored_entries) =
        tokio::task::spawn_blocking(move || expand_archives(form.files, max_bytes))
            .await
            .map_err(|e| AppError::Internal(format!("lecture des archives Wise : {e}")))??;

    let mut read: Vec<(String, WiseCsv)> = Vec::with_capacity(files.len());
    for file in &files {
        let csv = parse_wise_csv(&file.bytes)
            .map_err(|e| csv_error(&file.filename, file.archive.as_deref(), e))?;
        read.push((file.filename.clone(), csv));
    }
    if let Some(refusal) = duplicate_currency(&read) {
        return Err(refusal);
    }

    let parts: Vec<FilePart<'_>> = read
        .iter()
        .zip(&files)
        .map(|((filename, csv), file)| FilePart {
            filename,
            archive: file.archive.as_deref(),
            rows: csv.rows.len(),
            opening_balance: csv.opening_balance,
            closing_balance: csv.closing_balance,
        })
        .collect();
    let mut statements = Vec::with_capacity(read.len());
    for (((filename, csv), file), part) in read.iter().zip(&files).zip(&parts) {
        let trace = match serde_json::to_value(part) {
            Ok(serde_json::Value::Object(trace)) => trace,
            other => {
                return Err(AppError::Internal(format!(
                    "trace d'un fichier Wise : {other:?}"
                )));
            }
        };
        statements.push(CurrencyStatement {
            currency: csv.currency.clone(),
            filename: filename.clone(),
            bytes: &file.bytes,
            period_from: csv.first_date,
            period_to: csv.last_date,
            opening_balance: csv.opening_balance,
            closing_balance: csv.closing_balance,
            trace,
        });
    }
    let by_currency: BTreeMap<String, Vec<ProviderGroup>> = read
        .iter()
        .map(|(_, csv)| {
            let groups = csv.rows.iter().map(|r| vec![line_of_row(r)]).collect();
            (csv.currency.clone(), groups)
        })
        .collect();

    let mut audit_details = serde_json::Map::new();
    audit_details.insert("provider".into(), serde_json::json!(PROVIDER_WISE));
    audit_details.insert("provider_account".into(), serde_json::json!(form.account));
    audit_details.insert("ignoredEntries".into(), serde_json::json!(ignored_entries));
    let request = ImportRequest {
        provider: PROVIDER_WISE,
        account: &form.account,
        // Ignoré pour des relevés : chacun porte son nom.
        filename: DEFAULT_FILENAME,
        source: ImportSource::Statements(&statements),
        retry_operation: "wise_imports::create",
        audit_action: "wise_import.created",
        audit_details,
        by_currency,
    };
    let reports = import_lines(&state.pool, &current_user, &balances, &request).await?;
    let files = parts
        .iter()
        .zip(reports)
        .map(|(part, report)| part.report(report))
        .collect();

    Ok((
        StatusCode::CREATED,
        Json(WiseImportResponse {
            account: form.account,
            files,
            ignored_entries,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    const CHF: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-chf-fictif.csv");
    const EUR: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-eur-fictif.csv");
    const USD: &[u8] =
        include_bytes!("../../../kesh-import/tests/fixtures/wise-releve-usd-fictif.csv");

    fn rows(bytes: &[u8]) -> Vec<WiseRow> {
        parse_wise_csv(bytes).expect("fixture lisible").rows
    }

    fn row(bytes: &[u8], id: &str) -> WiseRow {
        rows(bytes).into_iter().find(|r| r.id == id).unwrap()
    }

    #[test]
    fn la_forme_d_un_iban() {
        assert_eq!(
            iban_shaped("CH56 0483 5012 3456 7800 9").as_deref(),
            Some("CH5604835012345678009")
        );
        assert_eq!(
            iban_shaped("ch93 0076 2011 6238 5295 7").as_deref(),
            Some("CH9300762011623852957")
        );
        // Numéro national, numéro masqué, trop court, trop long.
        assert_eq!(iban_shaped("0123456789"), None);
        assert_eq!(iban_shaped("12345678"), None);
        assert_eq!(iban_shaped("CH56 **** **** **** 7800 9"), None);
        assert_eq!(
            iban_shaped("CH5604835012345"),
            Some("CH5604835012345".into())
        );
        assert_eq!(iban_shaped("CH560483501234"), None, "14 caractères");
        assert_eq!(
            iban_shaped(&format!("CH56{}", "1".repeat(30))).map(|s| s.len()),
            Some(34)
        );
        assert_eq!(
            iban_shaped(&format!("CH56{}", "1".repeat(31))),
            None,
            "35 caractères"
        );
        assert_eq!(
            iban_shaped("1H5604835012345678009"),
            None,
            "pays non alphabétique"
        );
        assert_eq!(
            iban_shaped("CHX604835012345678009"),
            None,
            "clé non numérique"
        );
    }

    #[test]
    fn un_virement_vers_un_iban() {
        let r = row(CHF, "TRANSFER-90000005");
        let t = transaction_of_row(&r, 7, 9);
        assert_eq!((t.company_id, t.bank_account_id), (7, 9));
        assert_eq!(t.amount, dec!(-120.00));
        assert_eq!(t.currency, "CHF");
        assert_eq!(
            t.booking_date,
            chrono::NaiveDate::from_ymd_opt(2020, 2, 15).unwrap()
        );
        assert_eq!(t.value_date, None);
        assert_eq!(t.transaction_id.as_deref(), Some("TRANSFER-90000005"));
        assert_eq!(t.reference.as_deref(), Some("Facture 2020-001"));
        assert_eq!(t.end_to_end_id, None);
        assert_eq!(t.counterparty_name.as_deref(), Some("Atelier Exemple Sàrl"));
        assert_eq!(
            t.counterparty_iban.as_deref(),
            Some("CH5604835012345678009")
        );
        assert_eq!(t.counterparty_email, None);
        assert_eq!(t.provider_type.as_deref(), Some("TRANSFER"));
        assert_eq!(
            t.details,
            "Envoi de 120\u{202f}CHF à Atelier Exemple Sàrl · TRANSFER"
        );
    }

    #[test]
    fn un_numero_de_compte_non_iban_va_au_libelle() {
        let t = transaction_of_row(&row(CHF, "TRANSFER-90000006"), 1, 1);
        assert_eq!(t.counterparty_iban, None);
        assert_eq!(
            t.details,
            "Envoi vers Association Fictive · TRANSFER · 0123456789"
        );
        assert_eq!(t.reference, None);
    }

    #[test]
    fn contrepartie_marchand_payeur_et_frais() {
        let carte = transaction_of_row(&row(CHF, "CARD-90000007"), 1, 1);
        assert_eq!(carte.counterparty_name.as_deref(), Some("Boutique Exemple"));
        assert_eq!(carte.provider_type.as_deref(), Some("CARD"));
        let recu = transaction_of_row(&row(CHF, "BALANCE-90000008"), 1, 1);
        assert_eq!(
            recu.counterparty_name.as_deref(),
            Some("Stripe Payments Europe")
        );
        assert_eq!(recu.amount, dec!(1500.00));
        let frais = transaction_of_row(&row(CHF, "FEE-TRANSFER-90000004"), 1, 1);
        assert_eq!(frais.amount, dec!(-1.20));
        assert_eq!(frais.provider_type.as_deref(), Some("FEE-TRANSFER"));
        assert_eq!(frais.counterparty_name, None);
        // `Total fees` (1.20) du virement n'est jamais une transaction : le
        // virement garde son montant.
        let virement = transaction_of_row(&row(CHF, "TRANSFER-90000004"), 1, 1);
        assert_eq!(virement.amount, dec!(-300.00));
    }

    #[test]
    fn une_conversion_porte_son_montant_et_son_cours() {
        let t = transaction_of_row(&row(EUR, "CONVERSION_ORDER-90000011"), 1, 1);
        assert_eq!(
            t.details,
            "Converti 100.00 EUR en 109.00 USD · CONVERSION · EUR→USD 109.00 au cours 1.09"
        );
        assert_eq!(t.provider_type.as_deref(), Some("CONVERSION_ORDER"));
    }

    /// Revue P1, M1 — le montant de l'autre devise (`Exchange To Amount`)
    /// est écrit au segment de conversion ; sans lui ni cours, le segment
    /// reste « DE→VERS ».
    #[test]
    fn le_segment_de_conversion_a_une_forme_fixe() {
        let mut r = row(EUR, "CONVERSION_ORDER-90000011");
        r.exchange_to_amount = Some(dec!(41.85));
        r.exchange_rate = Some(dec!(0.9329));
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.contains(" · EUR→USD 41.85 au cours 0.9329"));
        r.exchange_rate = None;
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.ends_with(" · EUR→USD 41.85"), "{}", t.details);
        r.exchange_to_amount = None;
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.ends_with(" · EUR→USD"), "{}", t.details);
    }

    /// AC-4 — la somme des transactions d'un fichier est sa clôture moins son
    /// ouverture.
    #[test]
    fn la_somme_d_un_fichier_est_cloture_moins_ouverture() {
        for bytes in [CHF, EUR, USD] {
            let csv = parse_wise_csv(bytes).unwrap();
            let somme: Decimal = csv
                .rows
                .iter()
                .map(|r| transaction_of_row(r, 1, 1).amount)
                .sum();
            assert_eq!(
                somme,
                csv.closing_balance - csv.opening_balance,
                "{}",
                csv.currency
            );
        }
    }

    /// AC-4 — le libellé se tronque à la frontière d'un caractère (U+202F tient
    /// sur trois octets).
    #[test]
    fn libelle_et_contrepartie_bornes() {
        let mut r = row(CHF, "TRANSFER-90000005");
        r.note = Some("\u{202f}é".repeat(400));
        r.payee_name = Some("n".repeat(300));
        let t = transaction_of_row(&r, 1, 1);
        assert_eq!(t.details.chars().count(), DETAILS_MAX_CHARS);
        assert_eq!(t.counterparty_name.unwrap().chars().count(), NAME_MAX_CHARS);
    }

    /// Revue P2, L1 — une très longue description ne rogne pas le segment de
    /// conversion (montant et cours) ; la note, en queue, est rognée la première.
    #[test]
    fn une_longue_description_ne_rogne_pas_la_conversion() {
        let mut r = row(EUR, "CONVERSION_ORDER-90000011");
        r.description = Some("d".repeat(800));
        r.note = Some("n".repeat(300));
        let t = transaction_of_row(&r, 1, 1);
        assert!(t.details.chars().count() <= DETAILS_MAX_CHARS);
        assert!(
            t.details.contains(" · EUR→USD 109.00 au cours 1.09"),
            "{}",
            t.details
        );
    }

    #[test]
    fn deux_fichiers_de_meme_devise() {
        let read = vec![
            ("a.csv".to_string(), parse_wise_csv(CHF).unwrap()),
            ("b.csv".to_string(), parse_wise_csv(EUR).unwrap()),
            ("c.csv".to_string(), parse_wise_csv(CHF).unwrap()),
        ];
        match duplicate_currency(&read) {
            Some(AppError::WiseDuplicateCurrencyFile {
                currency,
                filenames,
            }) => {
                assert_eq!(currency, "CHF");
                assert_eq!(filenames, ["a.csv", "c.csv"]);
            }
            other => panic!("refus attendu : {other:?}"),
        }
        assert!(duplicate_currency(&read[..2]).is_none());
    }
}
