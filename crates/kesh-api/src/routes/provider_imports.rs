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

/// Les actions de trace que les appelants passent à [`import_lines`]
/// (`ImportRequest::audit_action`) : écrites en toutes lettres ici pour que
/// l'inventaire des sites d'audit indirects (`audit_label_registry`) les relise,
/// et contrôlées en tête d'[`import_lines`], avant toute écriture
/// ([`check_audit_action`]), y compris dans la version publiée. Une tranche et
/// non un tableau de taille fixe (revue P1, F4) : chaque prestataire y ajoute
/// sa ligne sans toucher aux autres — à la fusion avec la 40-7b
/// (`stripe_import.created`), l'union des deux listes. Un prestataire qui
/// ajoute une action l'écrit ici, à `audit_labels::ACTIONS`, aux quatre `.ftl`
/// et à l'inventaire `SITES_INDIRECTS` d'`audit_label_registry`.
pub const AUDIT_ACTIONS: &[&str] = &["paypal_download.created", "paypal_import.created"];

/// Refuse une action de trace absente d'[`AUDIT_ACTIONS`] : faute de Kesh (un
/// appelant neuf non inscrit, dont la trace n'aurait pas de libellé), non de
/// l'utilisateur — `AppError::Internal`, journalisée, rien d'écrit.
pub fn check_audit_action(action: &str) -> Result<(), AppError> {
    if AUDIT_ACTIONS.contains(&action) {
        return Ok(());
    }
    tracing::error!(
        action,
        "import de prestataire : action de trace non inscrite"
    );
    Err(AppError::Internal(
        "action de trace d'import non inscrite".into(),
    ))
}

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
            value_date: None,
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

/// Les transactions d'**une** ligne de la source (le brut et ses frais) :
/// l'unité des doublons. Jamais vide.
pub type ProviderGroup = Vec<ProviderLine>;

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
    /// Les groupes (une ligne de la source chacun), ventilés par devise,
    /// dans l'ordre de la source.
    pub by_currency: BTreeMap<String, Vec<ProviderGroup>>,
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
    groups: Vec<ProviderGroup>,
    file_hash: String,
}

/// L'empreinte d'une devise ([`ImportSource`]).
fn file_hash(source: ImportSource<'_>, currency: &str, groups: &[ProviderGroup]) -> String {
    let mut hashed = match source {
        ImportSource::File(bytes) => bytes.to_vec(),
        ImportSource::Download => {
            let mut ids: Vec<&str> = groups
                .iter()
                .flatten()
                .map(|l| l.transaction_id.as_str())
                .collect();
            ids.sort_unstable();
            ids.join("\n").into_bytes()
        }
    };
    hashed.push(b':');
    hashed.extend_from_slice(currency.as_bytes());
    compute_sha256_hex(&hashed)
}

/// Écarte les **doublons** — par groupe ([`ProviderGroup`], une ligne de la
/// source) : un groupe dont l'identifiant ([`ProviderLine::group_id`]) a déjà
/// paru plus haut, ou dont une transaction existe déjà sur le compte
/// (`existing`), est ignoré entier. Rend les lignes gardées et le nombre de
/// **transactions** ignorées.
pub fn dedupe_groups<'a>(
    groups: &'a [ProviderGroup],
    existing: &HashSet<String>,
) -> (Vec<&'a ProviderLine>, usize) {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut kept = Vec::new();
    let mut duplicates = 0;
    for group in groups {
        let Some(first) = group.first() else {
            continue;
        };
        let in_source = !seen.insert(first.group_id.as_str());
        if in_source || group.iter().any(|l| existing.contains(&l.transaction_id)) {
            duplicates += group.len();
        } else {
            kept.extend(group.iter());
        }
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
    check_audit_action(request.audit_action)?;
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
    for (currency, groups) in &request.by_currency {
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
            groups: groups.clone(),
            file_hash: file_hash(request.source, currency, groups),
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
            .groups
            .iter()
            .flatten()
            .map(|l| l.transaction_id.clone())
            .collect();
        let existing = bank_transactions::existing_transaction_ids(
            &mut tx,
            company_id,
            bank_account_id,
            &candidate_ids,
        )
        .await?;
        let (kept, duplicates) = dedupe_groups(&target.groups, &existing);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Revue P1, F4 — une action non inscrite se refuse aussi en version
    /// publiée (plus de `debug_assert!`), les actions des appelants passent.
    #[test]
    fn action_de_trace_inscrite_ou_refusee() {
        assert!(check_audit_action("paypal_import.created").is_ok());
        assert!(check_audit_action("paypal_download.created").is_ok());
        assert!(matches!(
            check_audit_action("inconnue"),
            Err(AppError::Internal(_))
        ));
    }

    fn line(group: &str, id: &str, amount: &str) -> ProviderLine {
        ProviderLine {
            group_id: group.into(),
            transaction_id: id.into(),
            booking_date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
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
        let a = [vec![line("A", "A", "1")], vec![line("B", "B", "2")]];
        let b = [vec![line("B", "B", "2")], vec![line("A", "A", "1")]];
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
        // Deux lignes de la source au même identifiant, à la suite : deux
        // groupes, le second doublon.
        let groups = [
            vec![line("A", "A", "10"), line("A", "A:FEE", "-1")],
            vec![line("A", "A", "10"), line("A", "A:FEE", "-1")],
            vec![line("B", "B", "5"), line("B", "B:FEE", "-0.5")],
            vec![line("C", "C", "7")],
        ];
        let existing: HashSet<String> = ["B:FEE".to_string()].into_iter().collect();
        let (kept, duplicates) = dedupe_groups(&groups, &existing);
        let ids: Vec<&str> = kept.iter().map(|l| l.transaction_id.as_str()).collect();
        assert_eq!(ids, ["A", "A:FEE", "C"]);
        assert_eq!(duplicates, 4, "la seconde A (deux) et B (deux)");
    }
}
