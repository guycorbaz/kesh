//! Le service des écritures comptables — la liste paginée et le détail
//! (Story 32-4a), lus par `GET /api/v1/journal-entries` et
//! `GET /api/v1/journal-entries/{id}`, puis par l'outil MCP de la 32-9a.
//!
//! Deux lectures **sans trace** (aucune n'écrit au journal d'audit) : elles
//! rendent directement (règle R5). La validation de la liste — analyse des
//! dates et des montants, contrôles croisés, `accountId` — et ses bornes de
//! pagination vivent ici, dans l'ordre et avec les messages d'avant la 32-4a
//! (C-32-4a-5) : le MCP, qui n'a pas le DTO d'Axum, les reçoit aussi.

use std::str::FromStr;

use chrono::NaiveDate;
use kesh_core::accounting::Journal as CoreJournal;
use kesh_core::listing::{SortBy, SortDirection};
use kesh_db::entities::{Journal as DbJournal, JournalEntryWithLines, Role};
use kesh_db::errors::{DbError, ModificationBlocker, map_db_error};
use kesh_db::repositories::imported_supplier_invoices::{self, EntrySourceDocument};
use kesh_db::repositories::journal_entries::{
    self, JournalEntryListQuery, JournalEntryListResult, ReversalBlockerHit,
};
use rust_decimal::Decimal;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`].
pub const LIMITE_DEFAUT: i64 = 50;
/// Plafond de la taille de page de [`lister`] (le dépôt écrête aussi à 500,
/// défensivement).
pub const LIMITE_MAX: i64 = 500;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/journal-entries`,
/// `authenticated_routes`) ; portée MCP `BooksRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// Exigence de [`detail`] : tous rôles (`GET /api/v1/journal-entries/{id}`,
/// `authenticated_routes`) ; portée MCP `BooksRead`.
pub const EXIGENCE_DETAIL: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::BooksRead),
};

/// La demande de [`lister`] : les paramètres de `GET /api/v1/journal-entries`,
/// dates et montants en **chaînes brutes** (analysées par le service), `limit`
/// et `offset` absents → leurs défauts.
#[derive(Debug, Clone, Default)]
pub struct DemandeListeEcritures {
    /// Filtre sur le libellé ; rogné, ignoré s'il est blanc.
    pub description: Option<String>,
    /// Ne garder que les écritures touchant ce compte (issue #374) ; doit être
    /// positif.
    pub account_id: Option<i64>,
    /// Montant minimal, en décimal (`"12.50"`) ; vide → absent.
    pub amount_min: Option<String>,
    /// Montant maximal, en décimal ; vide → absent.
    pub amount_max: Option<String>,
    /// Date de début, `AAAA-MM-JJ` ; vide → absente.
    pub date_from: Option<String>,
    /// Date de fin, `AAAA-MM-JJ` ; vide → absente.
    pub date_to: Option<String>,
    /// Journal.
    pub journal: Option<CoreJournal>,
    /// Colonne de tri.
    pub sort_by: SortBy,
    /// Sens du tri.
    pub sort_dir: SortDirection,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], ramenée entre 1 et
    /// [`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, ramené à 0 s'il est négatif.
    pub offset: Option<i64>,
}

/// Une date optionnelle de la demande ; le message nomme le paramètre de l'API.
fn analyser_date(valeur: Option<&str>, nom: &str) -> Result<Option<NaiveDate>, AppError> {
    match valeur {
        Some(s) if !s.is_empty() => {
            Ok(Some(NaiveDate::from_str(s).map_err(|e| {
                AppError::Validation(format!("{nom} invalide ({e})"))
            })?))
        }
        _ => Ok(None),
    }
}

/// Un montant optionnel de la demande ; le message nomme le paramètre de l'API.
fn analyser_montant(valeur: Option<&str>, nom: &str) -> Result<Option<Decimal>, AppError> {
    match valeur {
        Some(s) if !s.is_empty() => {
            Ok(Some(Decimal::from_str(s).map_err(|e| {
                AppError::Validation(format!("{nom} invalide ({e})"))
            })?))
        }
        _ => Ok(None),
    }
}

/// La page d'écritures de la société du contexte.
///
/// Dans l'ordre d'avant la 32-4a : bornes (défaut puis écrêtage) ; analyse de
/// `dateFrom`, `dateTo`, `amountMin`, `amountMax` ; montants négatifs ;
/// `amountMin > amountMax` ; `dateFrom > dateTo` ; `accountId <= 0` — neuf
/// refus `AppError::Validation` aux messages inchangés — ; description rognée.
///
/// ⚠️ **Aucune lecture de société** (C-32-4a-6) : la liste passe la société du
/// contexte au dépôt, qui filtre ; un jeton orphelin rend une page vide, non
/// une erreur — comportement d'avant la 32-4a, conservé.
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeListeEcritures,
) -> Result<JournalEntryListResult, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    let limit = demande.limit.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);
    let offset = demande.offset.unwrap_or(0).max(0);

    let date_from = analyser_date(demande.date_from.as_deref(), "dateFrom")?;
    let date_to = analyser_date(demande.date_to.as_deref(), "dateTo")?;
    let amount_min = analyser_montant(demande.amount_min.as_deref(), "amountMin")?;
    let amount_max = analyser_montant(demande.amount_max.as_deref(), "amountMax")?;

    // P6 : rejeter les montants négatifs (évite BETWEEN -100 AND max
    // qui ne filtre rien, ou BETWEEN 0 AND -100 qui renvoie toujours vide).
    if let Some(min) = amount_min
        && min < Decimal::ZERO
    {
        return Err(AppError::Validation(
            "amountMin ne peut pas être négatif".into(),
        ));
    }
    if let Some(max) = amount_max
        && max < Decimal::ZERO
    {
        return Err(AppError::Validation(
            "amountMax ne peut pas être négatif".into(),
        ));
    }

    // P2 : cross-validation des bornes. Un filtre min > max retournerait
    // 0 résultats silencieusement — l'utilisateur croirait à une absence
    // de données au lieu d'un filtre incohérent. Mieux vaut rejeter.
    if let (Some(min), Some(max)) = (amount_min, amount_max)
        && min > max
    {
        return Err(AppError::Validation(
            "amountMin doit être inférieur ou égal à amountMax".into(),
        ));
    }
    if let (Some(from), Some(to)) = (date_from, date_to)
        && from > to
    {
        return Err(AppError::Validation(
            "dateFrom doit être inférieur ou égal à dateTo".into(),
        ));
    }

    // `accountId <= 0` est un 400 plutôt qu'un filtre qui ne rendrait rien :
    // un filtre silencieusement vide se lit comme « pas d'écritures », ce qui
    // est le pire des deux messages.
    if let Some(id) = demande.account_id
        && id <= 0
    {
        return Err(AppError::Validation(
            "accountId doit être un identifiant positif".into(),
        ));
    }

    // Trim description (garde-fou cohérent avec create/update).
    let description = demande
        .description
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let query = JournalEntryListQuery {
        description,
        account_id: demande.account_id,
        amount_min,
        amount_max,
        date_from,
        date_to,
        journal: demande.journal.map(DbJournal::from),
        sort_by: demande.sort_by,
        sort_dir: demande.sort_dir,
        limit,
        offset,
    };
    Ok(
        journal_entries::list_by_company_paginated(lecture.pool(), acces.company_id(), query)
            .await?,
    )
}

/// Le détail d'une écriture et ce que l'écran peut en faire.
#[derive(Debug)]
pub struct DetailEcriture {
    /// L'écriture et ses lignes.
    pub ecriture: JournalEntryWithLines,
    /// L'écriture qui la contre-passe, s'il y en a une.
    pub contre_passee_par: Option<i64>,
    /// Le premier motif qui interdit de la contre-passer (`None` : permis).
    pub blocage_contre_passation: Option<ReversalBlockerHit>,
    /// Le motif qui interdit de la modifier (`None` : permis).
    pub blocage_modification: Option<ModificationBlocker>,
    /// Le document de la facture fournisseur dont elle est l'écriture d'achat
    /// ou de règlement (Story 30-2c).
    pub piece_source: Option<EntrySourceDocument>,
}

/// Le détail de l'écriture `id` de la société du contexte.
///
/// La société est lue d'abord (`Internal` si elle manque), puis l'écriture
/// (`AppError::Database(DbError::NotFound)` si elle est absente ou d'une autre
/// société — jamais `Forbidden`, anti-énumération KF-002), puis les quatre
/// motifs dans **une** transaction de lecture (Story 15-1b-0, D2 : un
/// instantané `REPEATABLE READ`, sans quoi un motif pourrait s'afficher contre
/// un état qu'aucun instant n'a connu), refermée par `rollback`. ⚠️
/// `find_by_id` reste hors de cette transaction — écart assumé : l'en-tête
/// n'est pas un motif. Garde : `get_journal_entry_reads_in_one_transaction`
/// (`mod tests` ci-dessous, déplacée de `routes/journal_entries.rs`).
pub async fn detail(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<DetailEcriture, AppError> {
    acces.exiger(EXIGENCE_DETAIL)?;
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;

    let ecriture = journal_entries::find_by_id(lecture.pool(), company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let mut tx = lecture.pool().begin().await.map_err(map_db_error)?;
    let blocage_contre_passation =
        journal_entries::reversal_blocker(&mut tx, company.id, id).await?;
    let contre_passee_par = journal_entries::reversed_by(&mut *tx, company.id, id).await?;
    let blocage_modification =
        journal_entries::modification_blocker(&mut tx, company.id, id).await?;
    // Story 30-2c (AC21) — le document de la facture fournisseur, dans le même
    // instantané (lecture ordinaire).
    let piece_source =
        imported_supplier_invoices::find_source_document_of_entry(&mut *tx, company.id, id).await?;
    tx.rollback().await.map_err(map_db_error)?;

    Ok(DetailEcriture {
        ecriture,
        contre_passee_par,
        blocage_contre_passation,
        blocage_modification,
        piece_source,
    })
}

#[cfg(test)]
mod tests {
    /// Story 15-1b-0 (AC4, D2 ; C-15-1b-0-4) — garde LEXICALE de la
    /// transaction de lecture du détail d'une écriture, déplacée de
    /// `routes/journal_entries.rs` avec le corps qu'elle garde (Story 32-4a,
    /// C-32-4a-7). Le compilateur garde `modification_blocker` (il exige une
    /// `Transaction`) mais ne voit pas une connexion acquise à côté :
    /// `reversal_blocker` et `reversed_by` prennent une connexion, que
    /// `lecture.pool().acquire()` fournirait aussi. Lit ce fichier, tronqué à
    /// la première ligne `#[cfg(test)]`, commentaires `//` écartés (patron
    /// `admin.rs`, test 20) ; le corps va de la signature à la fonction
    /// suivante ; une instruction court jusqu'au `;` (rustfmt peut la couper).
    /// La transaction se compte sur le corps débarrassé de ses blancs
    /// (`lecture\n.pool()\n.begin()` reste une ouverture). Écart assumé :
    /// `find_by_id` reste sur `lecture.pool()`, hors transaction.
    #[test]
    fn get_journal_entry_reads_in_one_transaction() {
        let code: String = include_str!("ecritures.rs")
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let debut = code
            .find("pub async fn detail(")
            .expect("detail introuvable");
        let fin = code[debut + 1..]
            .find("\npub async fn ")
            .map_or(code.len(), |i| debut + 1 + i);
        let corps = &code[debut..fin];
        let sans_blancs: String = corps.chars().filter(|c| !c.is_whitespace()).collect();
        assert_eq!(
            sans_blancs.matches("pool().begin()").count(),
            1,
            "une transaction de lecture, ouverte une fois : {corps}"
        );
        assert_eq!(
            corps.matches("acquire()").count(),
            0,
            "aucune connexion acquise à côté de la transaction : {corps}"
        );
        // Couples (module, lecteur) : le motif est paramétré par module (Story
        // 30-2c, AC21 — un lecteur d'un autre module ne s'inscrit pas en
        // ajoutant un nom à une liste qui chercherait `journal_entries::`).
        for (module, lecteur) in [
            ("journal_entries", "reversal_blocker"),
            ("journal_entries", "reversed_by"),
            ("journal_entries", "modification_blocker"),
            (
                "imported_supplier_invoices",
                "find_source_document_of_entry",
            ),
        ] {
            let motif = format!("{module}::{lecteur}(");
            let appels: Vec<&str> = corps
                .match_indices(&motif)
                .map(|(pos, _)| {
                    let fin = corps[pos..].find(';').map_or(corps.len(), |i| pos + i);
                    &corps[pos..fin]
                })
                .collect();
            assert_eq!(
                appels.len(),
                1,
                "assertion de montage : un appel de {lecteur}"
            );
            assert!(
                !appels[0].contains(".pool()")
                    && [
                        format!("{lecteur}(&mut tx,"),
                        format!("{lecteur}(&mut *tx,")
                    ]
                    .iter()
                    .any(|forme| appels[0].contains(forme.as_str())),
                "{lecteur} doit lire dans la transaction, non sur le pool : {}",
                appels[0]
            );
        }
        assert!(
            corps.contains("journal_entries::find_by_id(lecture.pool()"),
            "écart assumé (D2) : find_by_id hors transaction — s'il change, relire la garde"
        );
    }
}
