//! Changer le compte analytique d'une pièce **et de toutes ses écritures**,
//! après coup — Story 36-16a (#1167).
//!
//! Le geste ne touche ni montant, ni compte du plan, ni date, ni numéro, ni
//! lettrage, ni rapprochement : seulement `project_id`, sur la pièce et sur
//! **toutes les lignes** de ses écritures (l'invariant des créations, où
//! `NewJournalEntry::project_id` est posé sur chaque ligne), et les `version`
//! de ce qui change.
//!
//! # Le patron, réemployé par les Stories 36-16b et 36-16c
//!
//! Les fonctions génériques de ce module ([`lock_company_sentinel_in_tx`],
//! [`entry_ids_in_tx`], [`lock_entries_and_lines_in_tx`],
//! [`period_exposure_in_tx`], [`reassign_lines_in_tx`]) ne connaissent pas la
//! pièce ; [`reassign_invoice`] les enchaîne pour la facture client.
//!
//! # Ordre des verrous et des refus (fiche 36-16a, D9 ; C-36-16-13)
//!
//! 1. sentinelle `companies … FOR UPDATE`, **toujours**, qui lit aussi
//!    `books_locked_through` : la pose du verrou des livres
//!    (`companies::lock_books`, même ligne `FOR UPDATE`) est sérialisée avec le
//!    geste ;
//! 2. la pièce `FOR UPDATE` (`404`), puis sa `version` (`409
//!    OPTIMISTIC_LOCK_CONFLICT`) ;
//! 3. l'ensemble relu **sous verrou, après la pièce** — une lecture ordinaire
//!    manquerait un règlement commité depuis l'instantané ;
//! 4. en-têtes puis lignes de ces écritures `FOR UPDATE`, par `id` croissant ;
//!    court-circuit « rien à faire » (`200`, rien d'écrit) ;
//! 5. le compte visé (`projects::validate_taggable_in_tx`), **seulement s'il
//!    diffère** du compte actuel de la pièce ;
//! 6. les exercices des écritures qui changeraient, **tous** `FOR UPDATE` par
//!    `start_date` croissant, leur statut lu sous ce verrou ; **puis**, pour
//!    chacun resté ouvert, l'exercice postérieur clos, verrouillé
//!    ([`fiscal_years::find_later_closed_in_tx`]) ;
//! 7. l'exposition → `403` (pas un Administrateur en session) ou `409` (sans
//!    accusé) ;
//! 8. les `UPDATE`, l'audit, le `COMMIT`.
//!
//! **Aucun `UPDATE` ne précède l'étape 8** : un refus n'écrit rien. Les cycles
//! connus (facture ↔ `companies` avec la validation, le règlement, le solde,
//! l'annulation d'un règlement et l'avoir ; exercice ↔ `companies` avec
//! l'acceptation par rapprochement ; et, plus généralement, toute route qui
//! insère une écriture ou prend la sentinelle) sont bornés par le rejeu des
//! deux côtés : toutes ces routes sont `Rejouee`, le geste compris.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{MySql, MySqlPool, Transaction};

use crate::entities::NewAuditLogEntry;
use crate::entities::audit_log::Acteur;
use crate::entities::invoice::{Invoice, InvoiceLine};
use crate::errors::{ClosedPeriodExposure, DbError, map_db_error};
use crate::repositories::{audit_log, fiscal_years, invoice_settlements, invoices, projects};

/// Une écriture de l'ensemble, son en-tête verrouillé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedEntry {
    pub id: i64,
    pub fiscal_year_id: i64,
    pub entry_date: NaiveDate,
}

/// Une ligne d'écriture de l'ensemble, verrouillée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockedLine {
    pub id: i64,
    pub entry_id: i64,
    pub project_id: Option<i64>,
}

/// Ce que rend le geste sur une facture client : la facture et ses lignes lues
/// **dans la transaction**, le réglé et le reste dû (pour la même réponse
/// qu'un `GET`), et les écritures qui ont changé — vide si rien n'était à
/// faire, ou pour un brouillon.
#[derive(Debug, Clone)]
pub struct ReassignedInvoice {
    pub invoice: Invoice,
    pub lines: Vec<InvoiceLine>,
    pub amount_settled: Decimal,
    pub amount_due: Decimal,
    pub journal_entry_ids: Vec<i64>,
}

/// Étape 1 — la sentinelle de société, `FOR UPDATE`, qui rend aussi la borne du
/// verrou des livres lue **sous ce verrou**. Société absente → `NotFound`.
pub async fn lock_company_sentinel_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<Option<NaiveDate>, DbError> {
    let row: Option<Option<NaiveDate>> =
        sqlx::query_scalar("SELECT books_locked_through FROM companies WHERE id = ? FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    row.ok_or(DbError::NotFound)
}

/// Les identifiants des écritures d'une pièce, triés et sans doublon :
/// `base_ids` (celles que la pièce relie aujourd'hui), plus, si
/// `include_reversals`, les écritures qui contre-passent l'une d'elles
/// (`reverses_entry_id`).
///
/// - **Faux pour la facture client** : une écriture encore liée n'y a jamais de
///   contre-passation (la vente se supprime à la dévalidation, le règlement
///   annulé perd sa ligne, l'avoir ne se contre-passe pas).
/// - **Vrai pour la facture fournisseur** (Story 36-16b : contre-passation de
///   l'achat d'une facture annulée).
///
/// Lecture ordinaire des contre-passations : elle suit la sentinelle de société,
/// que toute création d'écriture prend en partagé (clé étrangère
/// `company_id`) — aucune contre-passation ne peut naître entre cette lecture et
/// la fin du geste. Les en-têtes sont verrouillés ensuite par
/// [`lock_entries_and_lines_in_tx`].
pub async fn entry_ids_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    base_ids: &[i64],
    include_reversals: bool,
) -> Result<Vec<i64>, DbError> {
    let mut ids: Vec<i64> = base_ids.to_vec();
    if include_reversals && !base_ids.is_empty() {
        let placeholders = placeholders(base_ids.len());
        let sql = format!(
            "SELECT id FROM journal_entries \
             WHERE company_id = ? AND reverses_entry_id IN ({placeholders})"
        );
        let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(company_id);
        for id in base_ids {
            q = q.bind(id);
        }
        ids.extend(q.fetch_all(&mut **tx).await.map_err(map_db_error)?);
    }
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}

/// Étapes 3 et 4 — les en-têtes des écritures `ids` puis leurs lignes,
/// `FOR UPDATE`, par `id` croissant. Une écriture absente (ou d'une autre
/// société) est un invariant rompu : une pièce ne relie que ses écritures.
pub async fn lock_entries_and_lines_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    ids: &[i64],
) -> Result<(Vec<LockedEntry>, Vec<LockedLine>), DbError> {
    if ids.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let placeholders = placeholders(ids.len());
    let sql = format!(
        "SELECT id, fiscal_year_id, entry_date FROM journal_entries \
         WHERE company_id = ? AND id IN ({placeholders}) ORDER BY id FOR UPDATE"
    );
    let mut q = sqlx::query_as::<_, (i64, i64, NaiveDate)>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    let entries: Vec<LockedEntry> = q
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(|(id, fiscal_year_id, entry_date)| LockedEntry {
            id,
            fiscal_year_id,
            entry_date,
        })
        .collect();
    if entries.len() != ids.len() {
        return Err(DbError::Invariant(format!(
            "changement de compte analytique : {} écriture(s) liée(s) attendue(s), {} trouvée(s)",
            ids.len(),
            entries.len()
        )));
    }

    let sql = format!(
        "SELECT id, entry_id, project_id FROM journal_entry_lines \
         WHERE entry_id IN ({placeholders}) ORDER BY id FOR UPDATE"
    );
    let mut q = sqlx::query_as::<_, (i64, i64, Option<i64>)>(&sql);
    for id in ids {
        q = q.bind(id);
    }
    let lines = q
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(|(id, entry_id, project_id)| LockedLine {
            id,
            entry_id,
            project_id,
        })
        .collect();
    Ok((entries, lines))
}

/// Les écritures qui changeraient : celles dont au moins une ligne ne porte pas
/// `target` (un `NULL` compris). Une écriture dont toutes les lignes le portent
/// déjà n'est pas touchée — sa `version` ne bouge pas.
pub fn entries_that_change(
    entries: &[LockedEntry],
    lines: &[LockedLine],
    target: Option<i64>,
) -> Vec<LockedEntry> {
    entries
        .iter()
        .filter(|e| {
            lines
                .iter()
                .any(|l| l.entry_id == e.id && l.project_id != target)
        })
        .cloned()
        .collect()
}

/// Étape 6 — l'exposition des écritures `changing` à une période close.
///
/// Les exercices de ces écritures sont **tous** verrouillés d'abord, **par
/// `start_date` croissant** (l'ordre de la clôture et du `PUT` d'une écriture),
/// leur statut lu **sous ce verrou** ; **puis**, pour chaque exercice resté
/// ouvert, le plus proche exercice postérieur clos, lu en lecture verrouillante.
/// Une clôture concurrente attend donc le geste, ou le geste attend la clôture et
/// lit `Closed`. `fiscal_years` nomme les exercices **clos en cause** : celui de
/// l'écriture s'il est clos, sinon l'exercice postérieur clos qui l'expose.
///
/// `books_locked_through` est la borne lue sous la sentinelle (étape 1) ; le
/// seuil est **inclusif**, comme partout ailleurs.
pub async fn period_exposure_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    changing: &[LockedEntry],
    books_locked_through: Option<NaiveDate>,
) -> Result<ClosedPeriodExposure, DbError> {
    let mut fy_ids: Vec<i64> = changing.iter().map(|e| e.fiscal_year_id).collect();
    fy_ids.sort_unstable();
    fy_ids.dedup();

    // L'ordre des verrous se lit sur `start_date`, par une lecture ordinaire :
    // la date de début d'un exercice qui porte des écritures ne change pas.
    let mut ordered: Vec<(i64, NaiveDate)> = Vec::new();
    if !fy_ids.is_empty() {
        let sql = format!(
            "SELECT id, start_date FROM fiscal_years WHERE company_id = ? AND id IN ({})",
            placeholders(fy_ids.len())
        );
        let mut q = sqlx::query_as::<_, (i64, NaiveDate)>(&sql).bind(company_id);
        for id in &fy_ids {
            q = q.bind(id);
        }
        ordered = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;
        ordered.sort_by_key(|(id, start)| (*start, *id));
    }

    // (a) Tous les exercices de l'ensemble, verrouillés par `start_date`
    //     croissant, leur statut lu sous ce verrou — avant toute recherche
    //     d'exercice postérieur, pour que l'ordre des verrous reste celui de la
    //     clôture (revue P1, A-5).
    let mut locked_years: Vec<(i64, String, NaiveDate, bool)> = Vec::new();
    for (fy_id, _) in &ordered {
        let (name, start_date, status): (String, NaiveDate, String) = sqlx::query_as(
            "SELECT name, start_date, status FROM fiscal_years \
             WHERE id = ? AND company_id = ? FOR UPDATE",
        )
        .bind(fy_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant(format!("exercice {fy_id} introuvable")))?;
        locked_years.push((*fy_id, name, start_date, status == "Closed"));
    }

    // (b) Pour chaque exercice resté ouvert, le plus proche exercice postérieur
    //     clos (lecture verrouillante). L'exercice exposé est nommé par
    //     l'exercice **clos en cause** — le sien, ou ce postérieur —, comme le
    //     garde de la modification d'une écriture (`journal_entries.rs`,
    //     `LaterFiscalYearClosed { fiscal_year_name: later.name }`).
    let mut exposed_years: Vec<i64> = Vec::new();
    let mut closed_names: Vec<String> = Vec::new();
    for (fy_id, name, start_date, closed) in &locked_years {
        let cause = if *closed {
            Some(name.clone())
        } else {
            fiscal_years::find_later_closed_in_tx(tx, company_id, *start_date)
                .await?
                .map(|later| later.name)
        };
        if let Some(cause) = cause {
            exposed_years.push(*fy_id);
            if !closed_names.contains(&cause) {
                closed_names.push(cause);
            }
        }
    }

    let mut entry_count = 0usize;
    let mut by_lock = false;
    for e in changing {
        let by_fy = exposed_years.contains(&e.fiscal_year_id);
        let locked = books_locked_through.is_some_and(|through| e.entry_date <= through);
        if by_fy || locked {
            entry_count += 1;
        }
        by_lock |= locked;
    }
    Ok(ClosedPeriodExposure {
        entry_count,
        fiscal_years: closed_names,
        locked_through: if by_lock { books_locked_through } else { None },
    })
}

/// Le refus de l'étape 7 : rien si aucune écriture exposée ne changerait ;
/// sinon `403` pour qui n'est pas un Administrateur en session
/// (`may_acknowledge` faux), quel que soit l'accusé, et `409` pour un
/// Administrateur qui n'a pas accusé.
pub fn refuse_if_exposed(
    exposure: &ClosedPeriodExposure,
    may_acknowledge: bool,
    acknowledge: bool,
) -> Result<(), DbError> {
    if exposure.entry_count == 0 {
        return Ok(());
    }
    if !may_acknowledge {
        return Err(DbError::ProjectReassignmentClosedPeriodAdminOnly(
            exposure.clone(),
        ));
    }
    if !acknowledge {
        return Err(DbError::ProjectReassignmentClosedPeriod(exposure.clone()));
    }
    Ok(())
}

/// Étape 8 — toutes les lignes des écritures `entry_ids` prennent `project_id`
/// (un `NULL` compris), et la `version` de chacune augmente d'une unité. Rien
/// d'autre ne change.
pub async fn reassign_lines_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    entry_ids: &[i64],
    project_id: Option<i64>,
) -> Result<(), DbError> {
    if entry_ids.is_empty() {
        return Ok(());
    }
    let placeholders = placeholders(entry_ids.len());
    let sql =
        format!("UPDATE journal_entry_lines SET project_id = ? WHERE entry_id IN ({placeholders})");
    let mut q = sqlx::query(&sql).bind(project_id);
    for id in entry_ids {
        q = q.bind(id);
    }
    q.execute(&mut **tx).await.map_err(map_db_error)?;

    let sql = format!(
        "UPDATE journal_entries SET version = version + 1 \
         WHERE company_id = ? AND id IN ({placeholders})"
    );
    let mut q = sqlx::query(&sql).bind(company_id);
    for id in entry_ids {
        q = q.bind(id);
    }
    let done = q.execute(&mut **tx).await.map_err(map_db_error)?;
    if done.rows_affected() != entry_ids.len() as u64 {
        return Err(DbError::Invariant(
            "changement de compte analytique : version d'écriture non incrémentée".into(),
        ));
    }
    Ok(())
}

/// Change le compte analytique d'une **facture client**, quel que soit son
/// statut, et de toutes ses écritures : la vente, les règlements (rapprochés
/// compris) et le solde du reste d'une facture payée ; la vente et l'avoir
/// d'une facture annulée (Story 36-16a, #1167).
///
/// - `version` est celle de la facture, exigée : périmée → `409
///   OPTIMISTIC_LOCK_CONFLICT`, même si le compte ne change pas.
/// - **Rien à faire** — la facture et toutes les lignes portent déjà
///   `project_id` — : rien n'est écrit, ni version, ni audit (`200`).
/// - Sinon la `version` de la facture augmente d'une unité. ⛔ **C'est ce qui
///   protège l'encaissement par rapprochement** : il lit la facture sans verrou
///   et ne l'écrit qu'à `version = ?` (`routes/reconciliation.rs`, invariant
///   « tout écrit qui change le reste dû incrémente `version` ») ; une
///   acceptation fondée sur un instantané antérieur au geste est refusée en
///   `race_during_update`, et l'écriture à l'ancien compte n'est pas
///   conservée.
/// - `may_acknowledge` dit si l'auteur est un Administrateur **en session**
///   (pas une clé d'API) ; `acknowledge`, s'il a accusé l'avertissement de
///   période close. Cf. [`refuse_if_exposed`].
/// - La trace `invoice.project_reassigned` est écrite **avant le `COMMIT`**, par
///   l'`acteur` (la clé d'API quand il y en a une) : un échec de la trace fait
///   échouer le geste entier.
#[allow(clippy::too_many_arguments)]
pub async fn reassign_invoice(
    pool: &MySqlPool,
    acteur: Acteur,
    may_acknowledge: bool,
    company_id: i64,
    invoice_id: i64,
    project_id: Option<i64>,
    version: i32,
    acknowledge: bool,
) -> Result<ReassignedInvoice, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // (1) La sentinelle, et la borne du verrou des livres sous elle.
    let books_locked_through = lock_company_sentinel_in_tx(&mut tx, company_id).await?;

    // (2) La facture, puis sa version.
    let (from_project_id, current_version, sale_entry_id): (Option<i64>, i32, Option<i64>) =
        sqlx::query_as(
            "SELECT project_id, version, journal_entry_id FROM invoices \
             WHERE id = ? AND company_id = ? FOR UPDATE",
        )
        .bind(invoice_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    if current_version != version {
        return Err(DbError::OptimisticLockConflict);
    }

    // (3) L'ensemble, relu sous verrou après la facture : la vente, les
    //     règlements (et le solde du reste), l'avoir.
    let mut base_ids: Vec<i64> = sale_entry_id.into_iter().collect();
    let settlements: Vec<i64> = sqlx::query_scalar(
        "SELECT journal_entry_id FROM invoice_settlements \
         WHERE invoice_id = ? AND company_id = ? ORDER BY id FOR UPDATE",
    )
    .bind(invoice_id)
    .bind(company_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;
    base_ids.extend(settlements);
    let credit_notes: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT journal_entry_id FROM credit_notes \
         WHERE invoice_id = ? AND company_id = ? ORDER BY id FOR UPDATE",
    )
    .bind(invoice_id)
    .bind(company_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;
    base_ids.extend(credit_notes.into_iter().flatten());
    let ids = entry_ids_in_tx(&mut tx, company_id, &base_ids, false).await?;

    // (4) En-têtes et lignes, puis « rien à faire ».
    let (entries, lines) = lock_entries_and_lines_in_tx(&mut tx, company_id, &ids).await?;
    let changing = entries_that_change(&entries, &lines, project_id);
    if from_project_id == project_id && changing.is_empty() {
        let read = read_invoice_in_tx(&mut tx, company_id, invoice_id, Vec::new()).await?;
        tx.commit().await.map_err(map_db_error)?;
        return Ok(read);
    }

    // (5) Le compte visé, seulement s'il diffère de l'actuel (un compte actuel
    //     archivé, visé de nouveau, réaligne les lignes sans refus).
    if project_id != from_project_id
        && let Some(target) = project_id
    {
        projects::validate_taggable_in_tx(&mut tx, company_id, &[target]).await?;
    }

    // (6) et (7) L'exposition à une période close, et son refus.
    let exposure =
        period_exposure_in_tx(&mut tx, company_id, &changing, books_locked_through).await?;
    refuse_if_exposed(&exposure, may_acknowledge, acknowledge)?;

    // (8) Les écritures, la facture, l'audit.
    let changed_ids: Vec<i64> = changing.iter().map(|e| e.id).collect();
    reassign_lines_in_tx(&mut tx, company_id, &changed_ids, project_id).await?;
    let marked = sqlx::query(
        "UPDATE invoices SET project_id = ?, version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ?",
    )
    .bind(project_id)
    .bind(invoice_id)
    .bind(company_id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;
    if marked.rows_affected() != 1 {
        return Err(DbError::Invariant(
            "changement de compte analytique : la facture n'a pas pu être mise à jour".into(),
        ));
    }
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::par(
            acteur,
            "invoice.project_reassigned",
            "invoice",
            invoice_id,
            Some(serde_json::json!({
                "fromProjectId": from_project_id,
                "toProjectId": project_id,
                "journalEntryIds": changed_ids,
                "closedPeriodsAcknowledged": exposure.entry_count > 0,
            })),
        ),
    )
    .await?;

    let read = read_invoice_in_tx(&mut tx, company_id, invoice_id, changed_ids).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(read)
}

/// La facture, ses lignes, le réglé et le reste dû, lus dans la transaction du
/// geste — la réponse égale le `GET` qui la suit, sans seconde lecture après le
/// `COMMIT` (patron de la Story 33-6, #450).
async fn read_invoice_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    invoice_id: i64,
    journal_entry_ids: Vec<i64>,
) -> Result<ReassignedInvoice, DbError> {
    let (invoice, lines) = invoices::find_with_lines_in_tx(&mut *tx, company_id, invoice_id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!(
                "facture {invoice_id} introuvable dans la transaction de son changement de compte analytique"
            ))
        })?;
    let amount_settled = invoice_settlements::amount_settled(&mut **tx, invoice_id).await?;
    let amount_due = invoice_settlements::amount_due(&mut **tx, invoice_id).await?;
    Ok(ReassignedInvoice {
        invoice,
        lines,
        amount_settled,
        amount_due,
        journal_entry_ids,
    })
}

/// `?, ?, …` pour une liste `IN (…)` de `n` éléments (`n ≥ 1`).
fn placeholders(n: usize) -> String {
    vec!["?"; n].join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: i64) -> LockedEntry {
        LockedEntry {
            id,
            fiscal_year_id: 1,
            entry_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        }
    }

    fn line(id: i64, entry_id: i64, project_id: Option<i64>) -> LockedLine {
        LockedLine {
            id,
            entry_id,
            project_id,
        }
    }

    #[test]
    fn an_entry_changes_when_one_of_its_lines_differs_null_included() {
        let entries = [entry(1), entry(2), entry(3)];
        let lines = [
            line(1, 1, Some(7)),
            line(2, 1, Some(7)),
            line(3, 2, Some(7)),
            line(4, 2, None),
            line(5, 3, Some(8)),
        ];
        let ids: Vec<i64> = entries_that_change(&entries, &lines, Some(7))
            .iter()
            .map(|e| e.id)
            .collect();
        assert_eq!(ids, vec![2, 3]);
        let ids: Vec<i64> = entries_that_change(&entries, &lines, None)
            .iter()
            .map(|e| e.id)
            .collect();
        assert_eq!(
            ids,
            vec![1, 2, 3],
            "retirer le compte change toute ligne qui en porte un"
        );
    }

    fn exposure(entry_count: usize) -> ClosedPeriodExposure {
        ClosedPeriodExposure {
            entry_count,
            fiscal_years: vec!["2025".into()],
            locked_through: None,
        }
    }

    #[test]
    fn nothing_exposed_is_never_refused() {
        assert!(refuse_if_exposed(&exposure(0), false, false).is_ok());
    }

    #[test]
    fn a_non_admin_is_refused_with_or_without_acknowledgement() {
        for ack in [false, true] {
            assert!(matches!(
                refuse_if_exposed(&exposure(2), false, ack),
                Err(DbError::ProjectReassignmentClosedPeriodAdminOnly(e)) if e.entry_count == 2
            ));
        }
    }

    #[test]
    fn an_admin_must_acknowledge() {
        assert!(matches!(
            refuse_if_exposed(&exposure(1), true, false),
            Err(DbError::ProjectReassignmentClosedPeriod(_))
        ));
        assert!(refuse_if_exposed(&exposure(1), true, true).is_ok());
    }

    #[test]
    fn placeholders_match_the_count() {
        assert_eq!(placeholders(1), "?");
        assert_eq!(placeholders(3), "?, ?, ?");
    }
}
