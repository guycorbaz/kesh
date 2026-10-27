//! Le service des factures clientes — la liste paginée, la fiche, l'échéancier
//! et les règlements d'une facture (Story 32-4b), lus par `GET /api/v1/invoices`,
//! `GET /api/v1/invoices/{id}`, `GET /api/v1/invoices/due-dates` et
//! `GET /api/v1/invoices/{id}/settlements`, puis par l'outil MCP de la 32-9a.
//!
//! Quatre lectures **sans trace** (aucune n'écrit au journal d'audit) : elles
//! rendent directement (règle R5). La validation des deux listes — bornes de
//! pagination, recherche, statut, dates — vit ici, dans l'ordre et avec les
//! messages d'avant la 32-4b : le MCP, qui n'a pas le DTO d'Axum, la reçoit
//! aussi. ⚠️ Les deux listes **refusent** une taille de page hors de
//! `1..=`[`LIMITE_MAX`] au lieu de l'écrêter (C-32-4b-6) : c'est le comportement
//! de l'écran, conservé ; l'outil MCP borne plus bas s'il le faut, le service ne
//! change pas le plafond de l'écran. Aucun dépôt n'écrête : la borne de ce
//! service est la **seule**.

use chrono::NaiveDate;
use kesh_core::listing::SortDirection;
use kesh_db::entities::Role;
use kesh_db::entities::invoice::{Invoice, InvoiceLine};
use kesh_db::entities::invoice_settlement::InvoiceSettlement;
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::invoices::{self, InvoiceListQuery};

/// Réexportés pour l'outil MCP de la 32-9a (C-32-9-25) — les types rendus et
/// ceux de la demande : `src/mcp/` ne nomme pas `kesh_db::repositories`.
pub use kesh_db::repositories::invoices::{
    DueDatesSummary, InvoiceListItem, InvoiceListResult, InvoiceSortBy, PausedFilter,
    PaymentStatusFilter,
};
use kesh_db::repositories::settlement_cancellation::SettlementCancelHit;
use kesh_db::repositories::{
    company_invoice_settings, invoice_settlements, invoice_settlements_write,
};
use rust_decimal::Decimal;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`] et de [`echeances`].
pub const LIMITE_DEFAUT: i64 = 20;
/// Taille de page maximale de [`lister`] et de [`echeances`] : au-delà, la
/// demande est **refusée** (jamais écrêtée).
pub const LIMITE_MAX: i64 = 100;
/// Longueur maximale de la recherche, en caractères, une fois rognée.
pub const RECHERCHE_MAX: usize = 100;
/// Décalage maximal de [`echeances`] (B16) ; la liste n'a pas de plafond.
pub const OFFSET_MAX_ECHEANCES: i64 = 1_000_000;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/invoices`,
/// `authenticated_routes`) ; portée MCP `InvoicesRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// Exigence de [`detail`] : tous rôles (`GET /api/v1/invoices/{id}`,
/// `authenticated_routes`) ; portée MCP `InvoicesRead`.
pub const EXIGENCE_DETAIL: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// Exigence de [`echeances`] : tous rôles (`GET /api/v1/invoices/due-dates`,
/// `authenticated_routes`) ; portée MCP `InvoicesRead`. L'export CSV
/// (Comptable+) n'est pas ce service : il n'en emploie que
/// [`requete_echeances`].
pub const EXIGENCE_ECHEANCES: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// Exigence de [`reglements`] : tous rôles
/// (`GET /api/v1/invoices/{id}/settlements`, `authenticated_routes`) ; portée
/// MCP `InvoicesRead`.
pub const EXIGENCE_REGLEMENTS: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// La demande de [`lister`] : les paramètres de `GET /api/v1/invoices`.
#[derive(Debug, Clone, Default)]
pub struct DemandeListeFactures {
    /// Recherche (client, numéro, conditions) ; rognée, blanche → absente,
    /// refusée au-delà de [`RECHERCHE_MAX`] caractères.
    pub search: Option<String>,
    /// Statut : `draft`, `validated` ou `cancelled` ; toute autre valeur est
    /// refusée.
    pub status: Option<String>,
    /// Ne garder que les factures de ce client.
    pub contact_id: Option<i64>,
    /// Date de facture minimale (incluse).
    pub date_from: Option<NaiveDate>,
    /// Date de facture maximale (incluse) ; `date_from > date_to` est refusé.
    pub date_to: Option<NaiveDate>,
    /// Suspension des rappels (Story 21-6a) ; absente → toutes.
    pub paused: Option<PausedFilter>,
    /// Colonne de tri ; absente → celle par défaut du dépôt (`Date`).
    pub sort_by: Option<InvoiceSortBy>,
    /// Sens du tri ; absent → descendant.
    pub sort_direction: Option<SortDirection>,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], refusée hors de
    /// `1..=`[`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, refusé s'il est négatif.
    pub offset: Option<i64>,
}

/// La demande de [`echeances`] : les paramètres de
/// `GET /api/v1/invoices/due-dates` (et de son export CSV, par
/// [`requete_echeances`]).
#[derive(Debug, Clone, Default)]
pub struct DemandeEcheances {
    /// Recherche ; mêmes règles que [`DemandeListeFactures::search`].
    pub search: Option<String>,
    /// Ne garder que les factures de ce client.
    pub contact_id: Option<i64>,
    /// Date de facture minimale (incluse).
    pub date_from: Option<NaiveDate>,
    /// Date de facture maximale (incluse) ; `date_from > date_to` est refusé.
    pub date_to: Option<NaiveDate>,
    /// Échéance maximale (incluse) ; refusée si elle précède `date_from` ou
    /// `date_to`.
    pub due_before: Option<NaiveDate>,
    /// Statut de paiement ; absent → impayées (B1).
    pub payment_status: Option<PaymentStatusFilter>,
    /// Colonne de tri ; absente → échéance.
    pub sort_by: Option<InvoiceSortBy>,
    /// Sens du tri ; absent → ascendant.
    pub sort_direction: Option<SortDirection>,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], refusée hors de
    /// `1..=`[`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, refusé s'il est négatif ou supérieur à
    /// [`OFFSET_MAX_ECHEANCES`].
    pub offset: Option<i64>,
}

/// La fiche d'une facture : l'en-tête, les lignes, ce qui a été réglé et ce
/// qui reste dû (Story 24-2), et, pour un brouillon, le réglage d'arrondi à 5
/// centimes dont l'écran tire l'aperçu (Story 25-4-c4-b).
#[derive(Debug)]
pub struct FicheFacture {
    /// La facture.
    pub facture: Invoice,
    /// Ses lignes.
    pub lignes: Vec<InvoiceLine>,
    /// Total déjà réglé (forme scalaire).
    pub montant_regle: Decimal,
    /// Reste dû (forme scalaire).
    pub reste_du: Decimal,
    /// Le réglage `round_to_5_centimes`, lu pour un **brouillon** seulement ;
    /// `None` : facture émise, réglage non lu (son arrondi est figé).
    pub arrondi_5_centimes: Option<bool>,
}

/// L'échéancier : une page de factures validées, et le résumé des impayées et
/// des échues sur les mêmes filtres, `payment_status` mis à part.
#[derive(Debug)]
pub struct Echeancier {
    /// La page.
    pub page: InvoiceListResult,
    /// Le résumé.
    pub resume: DueDatesSummary,
}

/// Un règlement d'une facture, et ce qui empêche de l'annuler.
#[derive(Debug)]
pub struct ReglementFacture {
    /// Le règlement.
    pub reglement: InvoiceSettlement,
    /// Le premier motif qui interdit de l'annuler (`None` : permis).
    pub motif: Option<SettlementCancelHit>,
}

/// La taille de page : défaut, puis refus hors de `1..=`[`LIMITE_MAX`].
fn limite(limit: Option<i64>) -> Result<i64, AppError> {
    let limit = limit.unwrap_or(LIMITE_DEFAUT);
    if !(1..=LIMITE_MAX).contains(&limit) {
        return Err(AppError::Validation(format!(
            "limit doit être compris entre 1 et {LIMITE_MAX}"
        )));
    }
    Ok(limit)
}

/// Le décalage : défaut 0, refus s'il est négatif.
fn decalage(offset: Option<i64>) -> Result<i64, AppError> {
    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::Validation(
            "offset doit être positif ou nul".into(),
        ));
    }
    Ok(offset)
}

/// La recherche : rognée, blanche → absente, refusée au-delà de
/// [`RECHERCHE_MAX`] caractères (cohérent `payment_terms`).
fn recherche(search: Option<String>) -> Result<Option<String>, AppError> {
    match search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) if s.chars().count() > RECHERCHE_MAX => Err(AppError::Validation(format!(
            "search doit faire au plus {RECHERCHE_MAX} caractères"
        ))),
        Some(s) => Ok(Some(s.to_string())),
        None => Ok(None),
    }
}

/// `dateFrom > dateTo` est refusé.
fn periode(date_from: Option<NaiveDate>, date_to: Option<NaiveDate>) -> Result<(), AppError> {
    if let (Some(df), Some(dt)) = (date_from, date_to)
        && df > dt
    {
        return Err(AppError::Validation(
            "dateFrom doit être antérieur ou égal à dateTo".into(),
        ));
    }
    Ok(())
}

/// La page de factures de la société du contexte.
///
/// La société est lue **d'abord** (`Internal` si elle manque — contrôle
/// défensif de la fenêtre d'obsolescence du `company_id`), puis, dans l'ordre
/// d'avant la 32-4b, cinq refus `AppError::Validation` : `limit`, `offset`,
/// `search`, `status`, `dateFrom > dateTo` ; puis la page, triée par défaut par
/// `Date` descendante. Gardes : `ac324b_a_les_refus_de_la_liste_et_de_l_echeancier`,
/// `ac324b_c_la_societe_est_lue_d_abord` et, pour la recopie de chaque filtre et
/// de chaque tri, `ac324b_d_e_f_la_liste_des_factures_par_la_route`
/// (`tests/services_rollout.rs`).
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeListeFactures,
) -> Result<InvoiceListResult, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    // Validate company exists (defensive: company_id staleness window)
    crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id()).await?;

    let limit = limite(demande.limit)?;
    let offset = decalage(demande.offset)?;
    let search = recherche(demande.search)?;
    if let Some(ref st) = demande.status
        && !matches!(st.as_str(), "draft" | "validated" | "cancelled")
    {
        return Err(AppError::Validation("status invalide".into()));
    }
    periode(demande.date_from, demande.date_to)?;

    let query = InvoiceListQuery {
        search,
        status: demande.status,
        contact_id: demande.contact_id,
        date_from: demande.date_from,
        date_to: demande.date_to,
        payment_status: None,
        due_before: None,
        // Story 21-6a (D10) — seule surface qui expose ce filtre. `None` quand
        // le client ne fournit rien → `All` → no-op.
        paused: demande.paused,
        sort_by: demande.sort_by.unwrap_or_default(),
        sort_direction: demande.sort_direction.unwrap_or(SortDirection::Desc),
        limit,
        offset,
    };
    Ok(invoices::list_by_company_paginated(lecture.pool(), acces.company_id(), query).await?)
}

/// La fiche de la facture `id` de la société du contexte.
///
/// ⚠️ **Aucune lecture de société** (fait 5 de la 32-4b, comportement d'avant
/// conservé, C-32-4b-8) : la facture est lue sous la société du contexte, un
/// jeton orphelin rend `NotFound`, non `Internal`. Garde :
/// `ac324b_d_detail_et_echeances_ne_lisent_pas_la_societe`. Absente ou d'une
/// autre société → `AppError::Database(DbError::NotFound)`. Puis le réglé et le
/// reste dû (Story 24-2 : la forme **scalaire**, une requête de plus sur **une**
/// facture, sans N+1 possible ; les agrégats emploient la forme jointe, tenue à
/// parité par `invoice_amount_due_parity.rs`), et, pour un brouillon seulement,
/// le réglage d'arrondi — par `round_to_5_centimes`, lecture pure, jamais un
/// `get_or_create` : un `GET` n'écrit rien.
pub async fn detail(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<FicheFacture, AppError> {
    acces.exiger(EXIGENCE_DETAIL)?;
    let (facture, lignes) = invoices::find_by_id_with_lines(lecture.pool(), acces.company_id(), id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    let montant_regle = invoice_settlements::amount_settled(lecture.pool(), id)
        .await
        .map_err(AppError::Database)?;
    let reste_du = invoice_settlements::amount_due(lecture.pool(), id)
        .await
        .map_err(AppError::Database)?;
    let arrondi_5_centimes = if facture.status == "draft" {
        Some(
            company_invoice_settings::round_to_5_centimes(lecture.pool(), facture.company_id)
                .await?,
        )
    } else {
        None
    };
    Ok(FicheFacture {
        facture,
        lignes,
        montant_regle,
        reste_du,
        arrondi_5_centimes,
    })
}

/// La requête de l'échéancier, sans accès ni lecture — **fonction pure**,
/// partagée par [`echeances`] et par l'export CSV de l'échéancier (qui la reçoit
/// sans `limit` ni `offset`, la pagination ne s'y appliquant pas).
///
/// Sept refus `AppError::Validation`, dans l'ordre d'avant la 32-4b : `limit`,
/// `offset` négatif, `offset` au-delà de [`OFFSET_MAX_ECHEANCES`], `search`,
/// `dateFrom > dateTo`, `dueBefore < dateFrom`, `dueBefore < dateTo`. Puis les
/// constantes : `status` forcé à `validated`, `payment_status` absent →
/// `Unpaid`, `paused` forcé à `None` (invariant anti-dissimulation), tri par
/// défaut par échéance ascendante. (L'ancienne construction de l'échéancier du
/// handler, déplacée avec ses commentaires.)
pub fn requete_echeances(demande: DemandeEcheances) -> Result<InvoiceListQuery, AppError> {
    let limit = limite(demande.limit)?;
    let offset = decalage(demande.offset)?;
    // B16 (review pass 1 G2 B) : cap raisonnable (cf. list_invoices).
    if offset > OFFSET_MAX_ECHEANCES {
        return Err(AppError::Validation(format!(
            "offset doit être ≤ {OFFSET_MAX_ECHEANCES}"
        )));
    }
    let search = recherche(demande.search)?;
    periode(demande.date_from, demande.date_to)?;
    // B11 (review pass 1 G2 B) : `dueBefore` doit être ≥ `dateFrom` (sinon
    // intersection vide silencieuse — l'utilisateur s'attend à un message
    // explicite).
    if let (Some(df), Some(db)) = (demande.date_from, demande.due_before)
        && db < df
    {
        return Err(AppError::Validation(
            "dueBefore doit être ≥ dateFrom".into(),
        ));
    }
    // B21 (review pass 2 G2 B) : pareil pour `dateTo` — une intersection
    // vide due_before < date_to mérite un 400 explicite (uniquement quand
    // les deux sont posés ; le cas indépendant reste autorisé).
    if let (Some(dt), Some(db)) = (demande.date_to, demande.due_before)
        && db < dt
    {
        return Err(AppError::Validation("dueBefore doit être ≥ dateTo".into()));
    }

    Ok(InvoiceListQuery {
        search,
        // Forcé côté backend — sécurité par défaut (Scope §5).
        status: Some("validated".into()),
        contact_id: demande.contact_id,
        date_from: demande.date_from,
        date_to: demande.date_to,
        // B1 (review pass 1 G2 B) : défaut backend = `Unpaid` pour respecter
        // AC#3 (échéancier chargé par défaut → impayées). Tout override
        // explicite du caller (`?paymentStatus=all|paid|overdue`) reste
        // honoré — seul le cas « pas de valeur fournie » bascule sur Unpaid.
        payment_status: Some(
            demande
                .payment_status
                .unwrap_or(PaymentStatusFilter::Unpaid),
        ),
        due_before: demande.due_before,
        // Story 21-6a (D10) — INVARIANT ANTI-DISSIMULATION : l'échéancier ne
        // filtre JAMAIS sur la suspension. Une facture suspendue sort de la
        // liste « à rappeler » (`dunning_eligibility`) et de nulle part
        // ailleurs. `None` → `PausedFilter::All` → no-op dans
        // `push_where_clauses`, que les items de cette page traversent via
        // `list_by_company_paginated`. L'échéancier n'expose aucun `?paused=`.
        // Ne PAS remplacer par un champ de la demande : `DemandeEcheances` n'a
        // volontairement pas ce champ.
        paused: None,
        sort_by: demande.sort_by.unwrap_or(InvoiceSortBy::DueDate),
        sort_direction: demande.sort_direction.unwrap_or(SortDirection::Asc),
        limit,
        offset,
    })
}

/// L'échéancier de la société du contexte : [`requete_echeances`], puis la page
/// et le résumé, lus en parallèle (lecture seule, sans transaction commune).
///
/// ⚠️ **Aucune lecture de société** (fait 5 de la 32-4b, comportement d'avant
/// conservé, C-32-4b-8) : la page et le résumé sont filtrés par la société du
/// contexte, un jeton orphelin rend une page vide et un résumé à zéro. Garde :
/// `ac324b_d_detail_et_echeances_ne_lisent_pas_la_societe`.
pub async fn echeances(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeEcheances,
) -> Result<Echeancier, AppError> {
    acces.exiger(EXIGENCE_ECHEANCES)?;
    let query = requete_echeances(demande)?;

    // B15 (review pass 1 G2 B) : strip explicite de `payment_status` avant
    // la summary — rend l'invariant AC#11 (« le summary ignore paymentStatus »)
    // visible côté service au lieu de dépendre uniquement du contrat repo.
    let summary_query = InvoiceListQuery {
        payment_status: None,
        ..query.clone()
    };
    // List + summary en parallèle (read-only, pas de tx commune).
    let (page, resume) = tokio::join!(
        invoices::list_by_company_paginated(lecture.pool(), acces.company_id(), query),
        invoices::due_dates_summary(lecture.pool(), acces.company_id(), &summary_query),
    );
    Ok(Echeancier {
        page: page?,
        resume: resume?,
    })
}

/// Les règlements de la facture `id`, du plus ancien au plus récent (l'ordre du
/// dépôt), chacun avec ce qui empêche de l'annuler (Story 25-3-a-1).
///
/// La société est lue d'abord (`Internal` si elle manque), puis la facture
/// (`NotFound` si elle n'est pas de la société, en plus du scoping des lignes),
/// puis la liste et les motifs dans **une** transaction de lecture, refermée par
/// `commit` (C-32-4b-11) : lus séparément, un règlement annulé entre la liste et
/// son motif rendait `NotFound` — un 404 sur toute la liste (revue P1 de la
/// 25-3-a-2). ⚠️ Un calcul de motifs par règlement : quelques petites requêtes,
/// bornées par le nombre de règlements d'**une** facture. Garde :
/// `reglements_lisent_dans_une_transaction` (`mod tests` ci-dessous).
pub async fn reglements(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<Vec<ReglementFacture>, AppError> {
    acces.exiger(EXIGENCE_REGLEMENTS)?;
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    invoices::find_by_id_with_lines(lecture.pool(), company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    // ⛔ Une transaction de lecture, pour un seul instantané.
    let mut tx = lecture.pool().begin().await.map_err(map_db_error)?;
    let liste = invoice_settlements::list_for_invoice(&mut *tx, company.id, id).await?;
    let mut out = Vec::with_capacity(liste.len());
    for reglement in liste {
        let motif =
            invoice_settlements_write::settlement_cancel_blocker(&mut tx, company.id, reglement.id)
                .await?;
        out.push(ReglementFacture { reglement, motif });
    }
    tx.commit().await.map_err(map_db_error)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    /// Story 32-4b (AC6, C-32-4b-11) — garde LEXICALE de la transaction de
    /// lecture des règlements d'une facture, au patron de
    /// `get_journal_entry_reads_in_one_transaction` (`services/ecritures.rs`).
    /// Le compilateur ne voit pas une connexion acquise à côté de la
    /// transaction : `list_for_invoice` prend un exécuteur, que
    /// `lecture.pool()` fournirait aussi, et `settlement_cancel_blocker` une
    /// connexion, que `lecture.pool().acquire()` fournirait. Lit ce fichier,
    /// tronqué à la première ligne `#[cfg(test)]`, commentaires `//` écartés ;
    /// le corps va de la signature de `reglements` à l'élément `pub` suivant ;
    /// une instruction court jusqu'au `;` (rustfmt peut la couper). Le corps se
    /// compte débarrassé de ses blancs (`&mut tx` → `&muttx`).
    #[test]
    fn reglements_lisent_dans_une_transaction() {
        let code: String = include_str!("factures.rs")
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let debut = code
            .find("pub async fn reglements(")
            .expect("reglements introuvable");
        let fin = code[debut + 1..]
            .find("\npub ")
            .map_or(code.len(), |i| debut + 1 + i);
        let corps: String = code[debut..fin]
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        assert_eq!(
            corps.matches("pool().begin()").count(),
            1,
            "une transaction de lecture, ouverte une fois : {corps}"
        );
        assert_eq!(
            corps.matches("acquire()").count(),
            0,
            "aucune connexion acquise à côté de la transaction : {corps}"
        );
        for forme in [
            "list_for_invoice(&mut*tx,",
            "settlement_cancel_blocker(&muttx,",
        ] {
            let appels: Vec<&str> = corps
                .match_indices(forme)
                .map(|(pos, _)| {
                    let fin = corps[pos..].find(';').map_or(corps.len(), |i| pos + i);
                    &corps[pos..fin]
                })
                .collect();
            assert_eq!(appels.len(), 1, "un appel {forme} dans la transaction");
            assert!(
                !appels[0].contains(".pool()"),
                "{forme} ne doit pas lire sur le pool : {}",
                appels[0]
            );
        }
        let lecteurs = ["list_for_invoice(", "settlement_cancel_blocker("];
        for lecteur in lecteurs {
            assert_eq!(
                corps.matches(lecteur).count(),
                1,
                "assertion de montage : un seul appel de {lecteur}"
            );
        }
        assert!(
            corps.contains("tx.commit()"),
            "la transaction de lecture se referme par commit (C-32-4b-11) : {corps}"
        );
    }
}
