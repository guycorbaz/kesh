//! Cœur commun de l'import des mouvements d'un prestataire — socle S6 (Story
//! 38-12a, extrait de la route PayPal de la Story 38-11b).
//!
//! Le cœur reçoit des lignes **neutres** ([`ProviderLine`]), déjà ventilées par
//! devise : le format d'origine (CSV PayPal, téléchargement par l'API, puis
//! Wise et Stripe) est l'affaire de l'appelant. Il fait, et lui seul :
//!
//! 1. la recherche des comptes bancaires par devise et les refus indépendants
//!    du format, dans l'ordre : `422 PROVIDER_CURRENCY_ACCOUNT_MISSING` → `409
//!    PROVIDER_CURRENCY_DUPLICATE` → `409 BANK_ACCOUNT_CLOSED` ;
//! 2. les **verrous** de chaque compte visé, par identifiant croissant
//!    (`with_account_locks`), sur une connexion dédiée, et l'écriture dans une
//!    **seconde** transaction, une seule pour toutes les devises (tout ou rien),
//!    rejouée sur interblocage (`retry_app_on_deadlock`) ;
//! 3. sous les verrous : la relecture verrouillante des soldes, l'empreinte
//!    (refus `422 BANK_IMPORT_DUPLICATE_FILE` pour un fichier, aucun pour un
//!    téléchargement), les **doublons** par identifiant de transaction, une
//!    entrée `bank_imports` par devise qui insère au moins une transaction, les
//!    traces `bank_import.created` (une par entrée) et la trace de la source
//!    (`paypal_import.created` ou `paypal_download.created`, jamais les deux).
//!
//! **Doublons** : l'unité est le **groupe** d'une ligne de la source (le brut
//! et ses frais, [`ProviderLine::group_id`]). Un groupe dont l'identifiant a
//! déjà paru plus haut, ou dont une transaction (brut ou frais) existe déjà sur
//! le compte bancaire, est ignoré entier. La fenêtre date-montant-référence des
//! imports bancaires ne s'applique pas.
//!
//! **Rejeu sur interblocage** (Story 38-11c1, P2-L1 de la 38-11b) : la
//! relecture verrouillante des soldes pose ses verrous partagés sur les lignes
//! `bank_accounts` de la société, puis l'`INSERT bank_imports` demande
//! `companies` en partage ; un geste de compte bancaire qui tient la sentinelle
//! `companies` en exclusif puis demande une de ces lignes forme un
//! interblocage, dont l'import, victime, est rejoué : rien n'est écrit hors de
//! la transaction annulée.

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::bank_import::{BankImportSourceFormat, NewBankImport};
use kesh_db::entities::bank_transaction::NewBankTransaction;
use kesh_db::errors::DbError;
use kesh_db::repositories::bank_accounts::{self, ProviderBalance};
use kesh_db::repositories::{audit_log, bank_imports, bank_transactions};
use kesh_reconciliation::{ReconciliationError, with_account_locks};

use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::bank_imports::{
    ImportAuditAction, compute_sha256_hex, import_create_error, insert_canonical_audit_log,
};
use crate::routes::bank_statements_manual::MANUAL_STATEMENT_LOCK_TIMEOUT_SECS;

/// Une transaction à importer, indépendante du format d'origine.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderLine {
    /// Identifiant de la ligne de la source dont la transaction vient (le brut
    /// et ses frais partagent le même) : l'unité des doublons.
    pub group_id: String,
    /// Identifiant de la transaction (`bank_transactions.transaction_id`) : le
    /// brut, ou `<id>:FEE` pour des frais.
    pub transaction_id: String,
    /// Date de comptabilisation.
    pub booking_date: NaiveDate,
    /// Date de valeur (Stripe : disponibilité des fonds) ; `None` pour PayPal.
    /// *Ajout de la 40-7b à la forme de la 38-12a.*
    pub value_date: Option<NaiveDate>,
    /// Montant signé.
    pub amount: Decimal,
    /// Devise (celle du compte bancaire visé).
    pub currency: String,
    /// Référence affichée.
    pub reference: Option<String>,
    /// Libellé.
    pub details: String,
    /// Référence de bout en bout (transaction liée).
    pub end_to_end_id: Option<String>,
    /// Nom de la contrepartie.
    pub counterparty_name: Option<String>,
    /// Adresse électronique de la contrepartie.
    pub counterparty_email: Option<String>,
    /// Type de mouvement chez le prestataire.
    pub provider_type: Option<String>,
    /// Rang de la ligne dans la source (ordre, messages, solde de clôture).
    pub rank: usize,
    /// Solde du compte du prestataire après la ligne, s'il est connu.
    pub balance_after: Option<Decimal>,
}

impl ProviderLine {
    /// La transaction bancaire de la ligne.
    pub fn into_new_bank_transaction(
        self,
        company_id: i64,
        bank_account_id: i64,
    ) -> NewBankTransaction {
        NewBankTransaction {
            company_id,
            bank_account_id,
            booking_date: self.booking_date,
            value_date: self.value_date,
            amount: self.amount,
            currency: self.currency,
            reference: self.reference,
            details: self.details,
            end_to_end_id: self.end_to_end_id,
            transaction_id: Some(self.transaction_id),
            counterparty_iban: None,
            counterparty_name: self.counterparty_name,
            provider_type: self.provider_type,
            counterparty_email: self.counterparty_email,
        }
    }
}

/// Compte rendu d'une devise : son compte bancaire, l'entrée `bank_imports`
/// créée (`null` si toutes ses lignes étaient des doublons), et les nombres de
/// **transactions** insérées et ignorées comme doublons.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyReport {
    pub currency: String,
    pub bank_account_id: i64,
    pub import_id: Option<i64>,
    pub inserted: usize,
    pub duplicates: usize,
}

/// Ce sur quoi porte l'empreinte d'une devise.
#[derive(Debug, Clone, Copy)]
pub enum ImportSource<'a> {
    /// Un fichier : empreinte = SHA-256 des octets ‖ `:` ‖ devise, et refus
    /// `422 BANK_IMPORT_DUPLICATE_FILE` si elle est déjà importée.
    File(&'a [u8]),
    /// Un téléchargement : empreinte = SHA-256 des identifiants de la devise,
    /// triés, ‖ `:` ‖ devise ; **aucun** refus d'empreinte (relancer la même
    /// période rend « 0 insérée »).
    Download,
}

/// Une demande d'import au cœur.
pub struct ImportRequest<'a> {
    /// Prestataire (`bank_accounts.provider`).
    pub provider: &'static str,
    /// Libellé du compte du prestataire.
    pub account: &'a str,
    /// Nom de l'import (`bank_imports.filename`), borné par l'appelant.
    pub filename: &'a str,
    /// Fichier ou téléchargement.
    pub source: ImportSource<'a>,
    /// Nom de l'opération du rejeu sur interblocage (journal et tests).
    pub retry_operation: &'static str,
    /// Action de la trace de la source (`paypal_import.created`…).
    pub audit_action: &'static str,
    /// `details` de cette trace, sans `currencies` (que le cœur ajoute).
    pub audit_details: serde_json::Map<String, serde_json::Value>,
    /// Les lignes, ventilées par devise (ordre de la source).
    pub by_currency: BTreeMap<String, Vec<ProviderLine>>,
}

/// Les soldes non archivés d'un compte du prestataire ; `404
/// PROVIDER_ACCOUNT_NOT_FOUND` s'il n'en a aucun.
pub async fn load_balances(
    pool: &sqlx::MySqlPool,
    company_id: i64,
    provider: &'static str,
    account: &str,
) -> Result<Vec<ProviderBalance>, AppError> {
    let balances =
        bank_accounts::list_provider_balances(pool, company_id, provider, account).await?;
    if balances.is_empty() {
        return Err(AppError::ProviderAccountNotFound {
            account: account.to_string(),
        });
    }
    Ok(balances)
}

/// Une devise et son compte bancaire.
struct Target<'a> {
    currency: String,
    balance: &'a ProviderBalance,
    lines: Vec<ProviderLine>,
    file_hash: String,
}

/// L'empreinte d'une devise ([`ImportSource`]).
fn file_hash(source: ImportSource<'_>, currency: &str, lines: &[ProviderLine]) -> String {
    let mut hashed = match source {
        ImportSource::File(bytes) => bytes.to_vec(),
        ImportSource::Download => {
            let mut ids: Vec<&str> = lines.iter().map(|l| l.transaction_id.as_str()).collect();
            ids.sort_unstable();
            ids.join("\n").into_bytes()
        }
    };
    hashed.push(b':');
    hashed.extend_from_slice(currency.as_bytes());
    compute_sha256_hex(&hashed)
}

/// Écarte les **doublons** — par groupe ([`ProviderLine::group_id`]) : un
/// groupe dont l'identifiant a déjà paru plus haut, ou dont une transaction
/// existe déjà sur le compte (`existing`), est ignoré entier. Rend les lignes
/// gardées et le nombre de **transactions** ignorées.
pub fn dedupe_lines<'a>(
    lines: &'a [ProviderLine],
    existing: &HashSet<String>,
) -> (Vec<&'a ProviderLine>, usize) {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut kept = Vec::new();
    let mut duplicates = 0;
    let mut i = 0;
    while i < lines.len() {
        let group = lines[i].group_id.as_str();
        // Un groupe est la suite de lignes de même `group_id` **sans identifiant
        // de transaction répété** : deux lignes identiques de la source, côte à
        // côte, sont deux groupes (correctif de la 40-7b : sans cela, la seconde
        // d'un doublon voisin était gardée avec la première).
        let mut ids: HashSet<&str> = HashSet::new();
        let mut end = i;
        while end < lines.len()
            && lines[end].group_id == group
            && ids.insert(lines[end].transaction_id.as_str())
        {
            end += 1;
        }
        let members = &lines[i..end];
        let in_source = !seen.insert(group);
        if in_source || members.iter().any(|l| existing.contains(&l.transaction_id)) {
            duplicates += members.len();
        } else {
            kept.extend(members);
        }
        i = end;
    }
    (kept, duplicates)
}

/// Importe les lignes d'un compte du prestataire — le cœur (voir l'en-tête du
/// module). `balances` vient de [`load_balances`].
pub async fn import_lines(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    balances: &[ProviderBalance],
    request: &ImportRequest<'_>,
) -> Result<Vec<CurrencyReport>, AppError> {
    let missing: Vec<String> = request
        .by_currency
        .keys()
        .filter(|c| !balances.iter().any(|b| b.currency() == c.as_str()))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(AppError::ProviderCurrencyAccountMissing {
            currencies: missing,
        });
    }

    let mut targets: Vec<Target<'_>> = Vec::new();
    for (currency, lines) in &request.by_currency {
        let mut matching = balances
            .iter()
            .filter(|b| b.currency() == currency.as_str());
        let first = matching
            .next()
            .expect("devise couverte, contrôlée ci-dessus");
        if let Some(second) = matching.next() {
            return Err(AppError::ProviderCurrencyDuplicate {
                account: request.account.to_string(),
                currency: currency.clone(),
                bank_account_id: second.account.id,
            });
        }
        targets.push(Target {
            currency: currency.clone(),
            balance: first,
            lines: lines.clone(),
            file_hash: file_hash(request.source, currency, lines),
        });
    }
    if let Some(closed) = targets.iter().find(|t| t.balance.account.is_closed()) {
        return Err(AppError::BankAccountClosed {
            bank_account_id: closed.balance.account.id,
        });
    }

    let ids: Vec<i64> = targets.iter().map(|t| t.balance.account.id).collect();
    let audit_entity_id = balances.iter().map(|b| b.account.id).min().ok_or_else(|| {
        AppError::ProviderAccountNotFound {
            account: request.account.to_string(),
        }
    })?;
    crate::retry::retry_app_on_deadlock(request.retry_operation, || {
        import_under_locks(pool, current_user, request, &targets, &ids, audit_entity_id)
    })
    .await
}

/// **Une tentative** de l'import : les verrous nommés des comptes `ids`, par
/// identifiant croissant, sur une connexion dédiée, puis l'écriture
/// ([`write_under_locks`]), conclue avant qu'ils soient rendus. Rejouée sur
/// interblocage par l'appelant : une erreur sqlx sort en
/// `AppError::Database(DbError::Sqlx(_))`, que reconnaît
/// `crate::retry::is_app_deadlock`.
async fn import_under_locks(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    request: &ImportRequest<'_>,
    targets: &[Target<'_>],
    ids: &[i64],
    audit_entity_id: i64,
) -> Result<Vec<CurrencyReport>, AppError> {
    let mut lock_tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let mut outcome: Option<Result<Vec<CurrencyReport>, AppError>> = None;
    let slot = &mut outcome;
    let locked = with_account_locks(
        &mut lock_tx,
        current_user.company_id,
        ids,
        MANUAL_STATEMENT_LOCK_TIMEOUT_SECS,
        async move |_held| {
            *slot = Some(
                write_under_locks(pool, current_user, request, targets, audit_entity_id).await,
            );
            Ok(())
        },
    )
    .await;
    drop(lock_tx);

    match (outcome, locked) {
        (Some(written), Ok(())) => written,
        // L'écriture a conclu ; seul un `RELEASE_LOCK` a échoué (verrou rendu
        // au retour de la connexion au pool, #842) : on rend son issue.
        (Some(written), Err(e)) => {
            tracing::error!(error = ?e, provider = request.provider, "import : libération des verrous en échec");
            written
        }
        (
            None,
            Err(ReconciliationError::AccountLocked {
                bank_account_id,
                timeout_secs,
            }),
        ) => Err(AppError::ReconciliationAccountLocked {
            bank_account_id,
            timeout_secs,
        }),
        (None, Err(ReconciliationError::Database(e))) => Err(AppError::Database(DbError::Sqlx(e))),
        (None, other) => {
            tracing::error!(
                ?other,
                provider = request.provider,
                "import : verrous des comptes non acquis"
            );
            Err(AppError::Internal(
                "verrous des comptes bancaires non acquis".to_string(),
            ))
        }
    }
}

/// Relit, **sous les verrous nommés** et par lectures verrouillantes, les
/// soldes du compte du prestataire, et refuse si les comptes visés ont changé
/// depuis la lecture du pool — Story 38-11b, revue P1 (L1). Un `PUT` qui retire
/// les réglages d'un compte encore sans transaction, ou change son compte du
/// plan, ne prend pas le verrou nommé : sans cette relecture, l'import
/// écrirait des transactions du prestataire sur un compte devenu ordinaire. Le
/// `PUT` tient la ligne en exclusif jusqu'à son commit ; la lecture partagée
/// l'attend, puis voit son effet — ou le `PUT`, venu après le premier
/// `INSERT`, trouve les transactions et refuse (garde 5). ⚠️ Entre cette
/// relecture et le premier `INSERT`, un geste de compte bancaire qui tient la
/// sentinelle `companies` et demande une ligne relue ici forme un
/// **interblocage** (P2-L1 de la 38-11b) : l'import, victime, est rejoué
/// ([`import_under_locks`]). Pour chaque devise visée : aucun solde → `422
/// PROVIDER_CURRENCY_ACCOUNT_MISSING` ; deux → `409
/// PROVIDER_CURRENCY_DUPLICATE` ; un autre compte que celui verrouillé → `409
/// OPTIMISTIC_LOCK_CONFLICT` (à rejouer).
async fn recheck_targets_locked(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    provider: &'static str,
    account: &str,
    targets: &[Target<'_>],
) -> Result<(), AppError> {
    let balances =
        bank_accounts::list_provider_balances_locked(tx, company_id, provider, account).await?;
    for target in targets {
        let matching: Vec<&ProviderBalance> = balances
            .iter()
            .filter(|b| b.currency() == target.currency)
            .collect();
        match matching.as_slice() {
            [] => {
                return Err(AppError::ProviderCurrencyAccountMissing {
                    currencies: vec![target.currency.clone()],
                });
            }
            [only] if only.account.id == target.balance.account.id => {}
            [_] => return Err(AppError::Database(DbError::OptimisticLockConflict)),
            [_, second, ..] => {
                return Err(AppError::ProviderCurrencyDuplicate {
                    account: account.to_string(),
                    currency: target.currency.clone(),
                    bank_account_id: second.account.id,
                });
            }
        }
    }
    Ok(())
}

/// L'écriture, **sous les verrous** des comptes visés : une seule transaction
/// pour toutes les devises (tout ou rien), conclue avant que les verrous soient
/// rendus.
async fn write_under_locks(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    request: &ImportRequest<'_>,
    targets: &[Target<'_>],
    audit_entity_id: i64,
) -> Result<Vec<CurrencyReport>, AppError> {
    let company_id = current_user.company_id;
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    if let Err(e) = recheck_targets_locked(
        &mut tx,
        company_id,
        request.provider,
        request.account,
        targets,
    )
    .await
    {
        let _ = tx.rollback().await;
        return Err(e);
    }

    if let ImportSource::File(_) = request.source {
        for target in targets {
            if let Some(existing) =
                bank_imports::find_by_company_and_hash(&mut *tx, company_id, &target.file_hash)
                    .await?
            {
                let _ = tx.rollback().await;
                return Err(AppError::BankImportDuplicateFile {
                    existing_import_id: existing.id,
                    existing_filename: existing.filename,
                });
            }
        }
    }

    let mut reports = Vec::with_capacity(targets.len());
    for target in targets {
        let bank_account_id = target.balance.account.id;
        let candidate_ids: Vec<String> = target
            .lines
            .iter()
            .map(|l| l.transaction_id.clone())
            .collect();
        let existing = bank_transactions::existing_transaction_ids(
            &mut tx,
            company_id,
            bank_account_id,
            &candidate_ids,
        )
        .await?;
        let (kept, duplicates) = dedupe_lines(&target.lines, &existing);
        if kept.is_empty() {
            reports.push(CurrencyReport {
                currency: target.currency.clone(),
                bank_account_id,
                import_id: None,
                inserted: 0,
                duplicates,
            });
            continue;
        }
        let period_from = kept
            .iter()
            .map(|l| l.booking_date)
            .min()
            .unwrap_or(NaiveDate::MIN);
        let period_to = kept
            .iter()
            .map(|l| l.booking_date)
            .max()
            .unwrap_or(NaiveDate::MIN);
        // Le solde de la dernière ligne (date la plus tardive ; à date égale, la
        // dernière de la source) qui en porte un.
        let closing_balance = kept
            .iter()
            .filter(|l| l.balance_after.is_some())
            .max_by_key(|l| (l.booking_date, l.rank))
            .and_then(|l| l.balance_after);
        let new_txs: Vec<NewBankTransaction> = kept
            .into_iter()
            .map(|l| {
                l.clone()
                    .into_new_bank_transaction(company_id, bank_account_id)
            })
            .collect();
        let inserted = new_txs.len();
        let transaction_count = i32::try_from(inserted)
            .map_err(|_| AppError::Internal("import : trop de transactions".into()))?;
        let new_import = NewBankImport {
            company_id,
            bank_account_id,
            filename: request.filename.to_string(),
            file_hash: target.file_hash.clone(),
            source_format: BankImportSourceFormat::Csv,
            statement_id: None,
            period_from,
            period_to,
            opening_balance: None,
            closing_balance,
            transaction_count,
            imported_by_user_id: current_user.user_id,
        };
        let header =
            match bank_imports::create_with_transactions(&mut tx, new_import, new_txs).await {
                Ok((header, _)) => header,
                Err(e) => {
                    let _ = tx.rollback().await;
                    return Err(import_create_error(e));
                }
            };
        let mut details_extra = serde_json::Map::new();
        details_extra.insert("provider".into(), serde_json::json!(request.provider));
        details_extra.insert(
            "provider_account".into(),
            serde_json::json!(request.account),
        );
        details_extra.insert("currency".into(), serde_json::json!(target.currency));
        details_extra.insert("duplicates_skipped".into(), serde_json::json!(duplicates));
        insert_canonical_audit_log(
            &mut tx,
            ImportAuditAction::Created,
            current_user.user_id,
            current_user.api_key_id,
            header.id,
            &header.filename,
            header.transaction_count,
            "CSV",
            vec![request.provider],
            details_extra,
            None,
        )
        .await?;
        reports.push(CurrencyReport {
            currency: target.currency.clone(),
            bank_account_id,
            import_id: Some(header.id),
            inserted,
            duplicates,
        });
    }

    let mut details = request.audit_details.clone();
    details.insert("currencies".into(), serde_json::json!(reports));
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            request.audit_action,
            "bank_account",
            audit_entity_id,
            Some(serde_json::Value::Object(details)),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    Ok(reports)
}

// ---------------------------------------------------------------------------
// Aides communes aux routes d'import de prestataire (ajout de la 40-7b :
// la route Stripe en a besoin, celle de PayPal garde ses copies privées —
// à replier à l'intégration, voir le Dev Agent Record de la 40-7b)
// ---------------------------------------------------------------------------

/// Borne des colonnes texte `details`, en caractères.
pub const DETAILS_MAX_CHARS: usize = 500;
/// Borne des noms et références, en caractères.
pub const NAME_MAX_CHARS: usize = 255;
/// Borne du type chez le prestataire, en caractères.
pub const PROVIDER_TYPE_MAX_CHARS: usize = 64;
/// Borne du nom de fichier, en octets.
pub const FILENAME_MAX_BYTES: usize = 255;
/// Longueur de la colonne `bank_transactions.transaction_id`, et de
/// `reference`, en caractères.
pub const TXID_COLUMN_MAX_CHARS: usize = 255;
/// Suffixe de l'identifiant de la transaction des **frais** : `<id>:FEE`.
pub const FEE_SUFFIX: &str = ":FEE";

/// Coupe `s` à `max` caractères.
pub fn truncate_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// Coupe `s` à `max` octets, sur une frontière de caractère.
pub fn truncate_bytes(s: &str, max: usize) -> String {
    let mut end = s.len().min(max);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Une valeur facultative, rognée, non vide, bornée à `max` caractères.
pub fn bounded(value: &Option<String>, max: usize) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| truncate_chars(s, max))
}

/// Le formulaire d'un import de fichier de prestataire : `file` et `account`.
pub struct ProviderImportForm {
    pub file_bytes: axum::body::Bytes,
    pub filename: String,
    pub account: String,
}

/// Lit le formulaire multipart (`file`, `account`) ; `default_filename` sert
/// quand le fichier n'a pas de nom. Champ manquant ou en double → `400
/// VALIDATION_ERROR`.
pub async fn parse_form(
    mut multipart: axum::extract::Multipart,
    default_filename: &str,
) -> Result<ProviderImportForm, AppError> {
    let mut file: Option<(axum::body::Bytes, String)> = None;
    let mut account: Option<String> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart : {e}")))?
    {
        match field.name() {
            Some("file") => {
                if file.is_some() {
                    return Err(AppError::Validation("champ `file` en double".into()));
                }
                let name = field.file_name().unwrap_or(default_filename).to_string();
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Validation(format!("multipart : {e}")))?;
                file = Some((bytes, name));
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
    let (file_bytes, filename) =
        file.ok_or_else(|| AppError::Validation("champ `file` manquant".into()))?;
    let account = account
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .ok_or_else(|| AppError::Validation("champ `account` manquant".into()))?;
    let filename = truncate_bytes(filename.trim(), FILENAME_MAX_BYTES);
    Ok(ProviderImportForm {
        file_bytes,
        filename: if filename.is_empty() {
            default_filename.to_string()
        } else {
            filename
        },
        account,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(group: &str, id: &str, amount: &str) -> ProviderLine {
        ProviderLine {
            group_id: group.into(),
            transaction_id: id.into(),
            booking_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            value_date: None,
            amount: amount.parse().unwrap(),
            currency: "EUR".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            counterparty_name: None,
            counterparty_email: None,
            provider_type: None,
            rank: 1,
            balance_after: None,
        }
    }

    #[test]
    fn l_empreinte_d_un_telechargement_ne_depend_pas_de_l_ordre() {
        let a = [line("A", "A", "1"), line("B", "B", "2")];
        let b = [line("B", "B", "2"), line("A", "A", "1")];
        assert_eq!(
            file_hash(ImportSource::Download, "EUR", &a),
            file_hash(ImportSource::Download, "EUR", &b)
        );
        assert_ne!(
            file_hash(ImportSource::Download, "EUR", &a),
            file_hash(ImportSource::Download, "USD", &a)
        );
        assert_ne!(
            file_hash(ImportSource::Download, "EUR", &a),
            file_hash(ImportSource::Download, "EUR", &a[..1])
        );
    }

    #[test]
    fn un_groupe_se_garde_ou_s_ecarte_entier() {
        let lines = [
            line("A", "A", "10"),
            line("A", "A:FEE", "-1"),
            line("B", "B", "5"),
            line("B", "B:FEE", "-0.5"),
            line("A", "A", "10"),
        ];
        let existing: HashSet<String> = ["B:FEE".to_string()].into_iter().collect();
        let (kept, duplicates) = dedupe_lines(&lines, &existing);
        let ids: Vec<&str> = kept.iter().map(|l| l.transaction_id.as_str()).collect();
        assert_eq!(ids, ["A", "A:FEE"]);
        assert_eq!(duplicates, 3, "B (deux) et la seconde A (une)");
    }

    /// Deux lignes identiques **côte à côte** dans la source sont deux groupes :
    /// la seconde est un doublon (correctif de la 40-7b).
    #[test]
    fn deux_lignes_identiques_voisines_sont_deux_groupes() {
        let lines = [
            line("A", "A", "10"),
            line("A", "A:FEE", "-1"),
            line("A", "A", "10"),
            line("A", "A:FEE", "-1"),
        ];
        let (kept, duplicates) = dedupe_lines(&lines, &HashSet::new());
        assert_eq!(kept.len(), 2);
        assert_eq!(duplicates, 2);
    }
}
