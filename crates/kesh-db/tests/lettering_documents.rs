//! Le lettrage des pièces clientes — Story 15-1a2-i (#518) — et fournisseurs —
//! Story 15-1a2-ii (section en fin de fichier).
//!
//! Une facture client **soldée** — par ses règlements, son solde ou son avoir —
//! est lettrée `document` d'office ; l'annulation d'un règlement la délettre ;
//! une pièce historique entièrement close n'est pas lettrée après coup, et un
//! lettrage figé par une période close ne se défait pas sans qu'un
//! administrateur la rouvre.
//!
//! La fixture partagée d'AC5 et les gestes de montage sont dans
//! `support/lettering_documents.rs` (C-15-1a2-28) ; les états hérités y sont
//! fabriqués en SQL brut.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::{SettlementChoice, SettlementWriteOffNature as Nature};
use kesh_db::errors::{DbError, SettlementCancelBlocker};
use kesh_db::repositories::letterings::{self, Actor, SyncOutcome};
use kesh_db::repositories::{
    companies, fiscal_years, invoice_settlements_write, journal_entries, reconciliation_cancel,
    supplier_invoices,
};
use kesh_db::test_fixtures::{SeededCompany, disable_rounding_to_5_centimes};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

#[path = "support/lettering_documents.rs"]
mod lettering_support;
use lettering_support::*;

// ---------------------------------------------------------------------------
// Outils
// ---------------------------------------------------------------------------

/// La marque `(clé, origine)` de la ligne de l'écriture `entry_id` sur le compte
/// `account_id` — l'unique telle ligne (assertion de montage).
async fn marque(pool: &MySqlPool, entry_id: i64, account_id: i64) -> (Option<i64>, Option<String>) {
    let lignes: Vec<(Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT lettering_key, lettering_origin FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ?",
    )
    .bind(entry_id)
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("marque");
    assert_eq!(lignes.len(), 1, "une ligne de {entry_id} sur {account_id}");
    lignes.into_iter().next().expect("ligne")
}

/// Toutes les marques d'une société, pour constater « rien n'a bougé ».
async fn toutes_les_marques(
    pool: &MySqlPool,
    company_id: i64,
) -> Vec<(i64, Option<i64>, Option<String>)> {
    sqlx::query_as(
        "SELECT jel.id, jel.lettering_key, jel.lettering_origin FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id WHERE je.company_id = ? ORDER BY jel.id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .expect("marques")
}

async fn compter(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.expect(sql)
}

async fn audits_de_lettrage(pool: &MySqlPool) -> i64 {
    compter(
        pool,
        "SELECT COUNT(*) FROM audit_log WHERE action IN ('lettering.created', 'lettering.removed')",
    )
    .await
}

/// Appelle la synchronisation dans une transaction qui tient l'exercice
/// `fy` `FOR UPDATE` (le contrat du mode `System`) ; valide si elle réussit.
async fn synchroniser(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    inv: i64,
    fy: i64,
) -> Result<SyncOutcome, DbError> {
    let mut tx = pool.begin().await.expect("tx");
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(fy)
        .execute(&mut *tx)
        .await
        .expect("exercice tenu");
    let issue = letterings::sync_invoice_in_tx(
        &mut tx,
        seeded.company_id,
        inv,
        fy,
        Actor {
            user_id: seeded.admin_user_id,
            api_key_id: None,
        },
    )
    .await;
    if issue.is_ok() {
        tx.commit().await.expect("commit");
    }
    issue
}

/// L'écriture qui contre-passe `entry_id`.
async fn contre_passation(pool: &MySqlPool, entry_id: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
        .bind(entry_id)
        .fetch_one(pool)
        .await
        .expect("contre-passation")
}

/// Le règlement `entry_id` et son miroir sont lettrés `reversal` ensemble sur
/// `account_id`.
async fn assert_paire_reversal(pool: &MySqlPool, entry_id: i64, account_id: i64) {
    let miroir = contre_passation(pool, entry_id).await;
    let (k1, o1) = marque(pool, entry_id, account_id).await;
    let (k2, o2) = marque(pool, miroir, account_id).await;
    assert!(
        k1.is_some(),
        "le règlement {entry_id} est lettré sur {account_id}"
    );
    assert_eq!(k1, k2, "règlement et miroir sous la même clé");
    assert_eq!(o1.as_deref(), Some("reversal"));
    assert_eq!(o2.as_deref(), Some("reversal"));
}

// ---------------------------------------------------------------------------
// AC1 — règlements
// ---------------------------------------------------------------------------

/// AC1 — une facture entièrement réglée est lettrée `document` : créance et
/// ligne de règlement sur `A`, une seule clé.
#[sqlx::test(migrations = "./test-schema")]
async fn full_settlement_letters_sale_and_settlements(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (_, entry) = regler(&pool, &seeded, inv, dec!(100.00), jours_avant(90)).await;

    assert!(lettree_document(&pool, inv).await, "lettrée document");
    let a = seeded.accounts["1100"];
    let (k_vente, _) = marque(&pool, vente(&pool, inv).await, a).await;
    let (k_reglement, o) = marque(&pool, entry, a).await;
    assert_eq!(k_vente, k_reglement, "une seule clé");
    assert_eq!(o.as_deref(), Some("document"));
    // La contrepartie (caisse) n'est pas dans le groupe.
    assert_eq!(marque(&pool, entry, seeded.accounts["1000"]).await.0, None);
}

/// AC1 — trois règlements partiels, dont un `internal_account` : rien n'est
/// lettré avant le dernier, qui lettre les quatre lignes sous une clé.
#[sqlx::test(migrations = "./test-schema")]
async fn three_partials_with_internal_account_letter_on_the_last(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    // Un compte bancaire lié (1020) pour le règlement par virement.
    let banque = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1020', 'Banque virements', 'Asset')",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let bank_account_id = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Banque', 'CH9300762011623852957', ?)",
    )
    .bind(seeded.company_id)
    .bind(banque)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let inv = facture(&pool, &seeded, dec!(90.00), jours_avant(100)).await;
    let le = jours_avant(90);
    regler_par(
        &pool,
        &seeded,
        inv,
        SettlementChoice::BankTransfer { bank_account_id },
        dec!(30.00),
        le,
    )
    .await;
    assert_eq!(cle_de(&pool, inv).await, None, "un partiel : rien");
    regler(&pool, &seeded, inv, dec!(30.00), le).await;
    assert_eq!(cle_de(&pool, inv).await, None, "deux partiels : rien");
    assert_eq!(audits_de_lettrage(&pool).await, 0);
    regler(&pool, &seeded, inv, dec!(30.00), le).await;

    assert!(lettree_document(&pool, inv).await);
    assert_eq!(
        lignes_de_piece(&pool, inv).await.len(),
        4,
        "vente + trois règlements"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'lettering.created'"
        )
        .await,
        1
    );
}

/// AC1 — un règlement partiel seul ne lettre rien.
#[sqlx::test(migrations = "./test-schema")]
async fn partial_settlement_letters_nothing(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv, dec!(40.00), jours_avant(90)).await;
    assert!(!lettree_document(&pool, inv).await);
    assert!(
        lignes_de_piece(&pool, inv)
            .await
            .iter()
            .all(|l| l.2.is_none()),
        "aucune marque"
    );
    assert_eq!(audits_de_lettrage(&pool).await, 0);
}

// ---------------------------------------------------------------------------
// AC2 — soldes et arrondis
// ---------------------------------------------------------------------------

/// AC2 — le solde du reste, pour chacune des quatre natures, éteint la facture
/// et la lettre, la ligne de solde comprise.
#[sqlx::test(migrations = "./test-schema")]
async fn write_off_each_kind_letters(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    disable_rounding_to_5_centimes(&pool, seeded.company_id)
        .await
        .unwrap();
    let le = jours_avant(90);
    for (nature, ht, paye) in [
        (Nature::Discount, dec!(100.00), dec!(90.00)),
        (Nature::BankFees, dec!(100.00), dec!(97.00)),
        (Nature::BadDebt, dec!(100.00), dec!(50.00)),
        (Nature::Rounding, dec!(100.03), dec!(100.00)),
    ] {
        let inv = facture(&pool, &seeded, ht, jours_avant(100)).await;
        regler(&pool, &seeded, inv, paye, le).await;
        let (_, solde) = solder(&pool, &seeded, inv, nature, le).await;
        assert!(lettree_document(&pool, inv).await, "{nature:?} : lettrée");
        assert_eq!(
            marque(&pool, solde, seeded.accounts["1100"])
                .await
                .1
                .as_deref(),
            Some("document"),
            "{nature:?} : la ligne de solde est du groupe"
        );
        assert_eq!(lignes_de_piece(&pool, inv).await.len(), 3, "{nature:?}");
    }
}

/// AC2 — l'arrondi à 5 centimes de la facture et l'écart d'un règlement
/// `SettlesWithRounding` ne sont pas sur `A` : ils restent hors du groupe.
#[sqlx::test(migrations = "./test-schema")]
async fn rounding_line_is_left_out(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let arrondi: i64 = sqlx::query_scalar(
        "SELECT default_rounding_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // (a) Facture arrondie à 5 centimes (réglage actif par défaut) : 100.03 →
    // 100.05, l'écart sur le compte d'arrondi de la vente.
    let inv = facture(&pool, &seeded, dec!(100.03), jours_avant(100)).await;
    let v = vente(&pool, inv).await;
    assert_eq!(
        marque(&pool, v, arrondi).await,
        (None, None),
        "montage : la vente porte une ligne d'arrondi, ouverte"
    );
    regler(&pool, &seeded, inv, dec!(100.05), jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await, "(a) lettrée");
    assert_eq!(
        marque(&pool, v, arrondi).await,
        (None, None),
        "(a) l'arrondi reste ouvert"
    );

    // (b) Règlement `SettlesWithRounding` : reste brut 10.0050, payé 10.01.
    disable_rounding_to_5_centimes(&pool, seeded.company_id)
        .await
        .unwrap();
    let inv = facture(&pool, &seeded, dec!(10.005), jours_avant(100)).await;
    let (_, entry) = regler(&pool, &seeded, inv, dec!(10.01), jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await, "(b) lettrée");
    assert_eq!(
        marque(&pool, entry, arrondi).await,
        (None, None),
        "(b) la ligne d'écart du règlement est exclue"
    );
}

// ---------------------------------------------------------------------------
// AC3 — avoirs
// ---------------------------------------------------------------------------

/// AC3 — l'avoir total lettre la facture et l'avoir (il crédite `A`, 15-6a).
#[sqlx::test(migrations = "./test-schema")]
async fn credit_note_letters_invoice_and_note(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let avoir = crediter(&pool, &seeded, inv, jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await);
    let a = seeded.accounts["1100"];
    assert_eq!(
        marque(&pool, avoir, a).await.0,
        marque(&pool, vente(&pool, inv).await, a).await.0
    );
}

/// AC3 — un avoir HÉRITÉ crédité sur un autre compte que `A` : la
/// synchronisation ne forme aucun groupe, et ne rend aucune erreur.
#[sqlx::test(migrations = "./test-schema")]
async fn legacy_credit_note_on_other_account_forms_no_group(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let avoir = crediter(&pool, &seeded, inv, jours_avant(90)).await;
    effacer_marques(&pool, cle_de(&pool, inv).await.expect("lettrée")).await;
    let deplacees = sqlx::query(
        "UPDATE journal_entry_lines SET account_id = ? WHERE entry_id = ? AND account_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(avoir)
    .bind(seeded.accounts["1100"])
    .execute(&pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(deplacees, 1, "montage : la ligne de créance de l'avoir");
    let audits = audits_de_lettrage(&pool).await;

    let issue = synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
        .await
        .expect("aucune erreur");
    assert_eq!(issue, SyncOutcome::Unchanged, "|C(I)| = 1 : aucun groupe");
    assert!(!lettree_document(&pool, inv).await);
    assert_eq!(audits_de_lettrage(&pool).await, audits);
}

// ---------------------------------------------------------------------------
// AC4 — annulations
// ---------------------------------------------------------------------------

/// AC4 — annuler un règlement d'une facture soldée dissout le groupe ; le
/// règlement et son miroir sont lettrés `reversal` (et, pour la caisse,
/// lettrable, la contrepartie avec la sienne) ; la créance et l'autre
/// règlement partiel sont ouverts.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_settlement_dissolves_and_pairs(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (_, autre) = regler(&pool, &seeded, inv, dec!(60.00), jours_avant(90)).await;
    let (sid, annule) = regler(&pool, &seeded, inv, dec!(40.00), jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await, "montage : lettrée");

    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation");

    let a = seeded.accounts["1100"];
    assert_paire_reversal(&pool, annule, a).await;
    assert_paire_reversal(&pool, annule, seeded.accounts["1000"]).await;
    assert_eq!(
        marque(&pool, vente(&pool, inv).await, a).await,
        (None, None),
        "créance ouverte"
    );
    assert_eq!(
        marque(&pool, autre, a).await,
        (None, None),
        "l'autre partiel est ouvert"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'lettering.removed'"
        )
        .await,
        1
    );
}

/// AC4 (validation P2, L-5) — même résultat par l'annulation du SOLDE qui
/// éteignait la facture.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_write_off_dissolves_and_pairs(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (_, partiel) = regler(&pool, &seeded, inv, dec!(90.00), jours_avant(90)).await;
    let (sid, solde) = solder(&pool, &seeded, inv, Nature::Discount, jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await, "montage : lettrée");

    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation du solde");

    let a = seeded.accounts["1100"];
    assert_paire_reversal(&pool, solde, a).await;
    assert_eq!(
        marque(&pool, vente(&pool, inv).await, a).await,
        (None, None)
    );
    assert_eq!(marque(&pool, partiel, a).await, (None, None));
    assert!(!lettree_document(&pool, inv).await);
}

/// AC4 — même résultat par le DÉ-RAPPROCHEMENT d'une facture encaissée par
/// rapprochement (montage F4-4).
#[sqlx::test(migrations = "./test-schema")]
async fn unreconcile_dissolves_and_pairs(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (_, entry) = regler(&pool, &seeded, inv, dec!(100.00), jours_avant(90)).await;
    let bt = rapprocher(&pool, &seeded, entry, dec!(100.00), jours_avant(90)).await;
    assert!(lettree_document(&pool, inv).await, "montage : lettrée");

    reconciliation_cancel::cancel(&pool, seeded.company_id, bt, seeded.admin_user_id)
        .await
        .expect("dé-rapprochement");

    let a = seeded.accounts["1100"];
    assert_paire_reversal(&pool, entry, a).await;
    assert_eq!(
        marque(&pool, vente(&pool, inv).await, a).await,
        (None, None)
    );
    assert!(!lettree_document(&pool, inv).await);
}

/// AC4 (F-11) — annuler un règlement PARTIEL (aucun groupe) réussit, sans
/// aucune dissolution ni entrée `lettering.removed`.
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_partial_without_group_is_a_noop(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (sid, _) = regler(&pool, &seeded, inv, dec!(40.00), jours_avant(90)).await;
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation d'un partiel");
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action = 'lettering.removed'"
        )
        .await,
        0
    );
}

/// AC4, P5 — facture de l'exercice N CLOS, règlement et annulation en N+1 :
/// le groupe a une ligne en N+1, ouverte — l'annulation réussit.
#[sqlx::test(migrations = "./test-schema")]
async fn closed_year_n_settled_and_cancelled_in_n1(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let fin_n = jours_avant(60);
    sqlx::query("UPDATE fiscal_years SET end_date = ? WHERE id = ?")
        .bind(fin_n)
        .bind(seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    fiscal_years::create(
        &pool,
        seeded.admin_user_id,
        kesh_db::entities::NewFiscalYear {
            company_id: seeded.company_id,
            name: "N+1".into(),
            start_date: fin_n + chrono::Duration::days(1),
            end_date: NaiveDate::from_ymd_opt(2030, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice N+1");
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    fiscal_years::close(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        seeded.fiscal_year_id,
    )
    .await
    .expect("clôture de N");
    let (sid, entry) = regler(&pool, &seeded, inv, dec!(100.00), jours_avant(30)).await;
    assert!(
        lettree_document(&pool, inv).await,
        "lettrée : une ligne en N+1"
    );

    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation en N+1");
    assert_paire_reversal(&pool, entry, seeded.accounts["1100"]).await;
    assert_eq!(
        marque(&pool, vente(&pool, inv).await, seeded.accounts["1100"]).await,
        (None, None),
        "la vente de N est délettrée"
    );
}

// ---------------------------------------------------------------------------
// AC5 — accord grand livre ↔ pièce
// ---------------------------------------------------------------------------

/// AC5 — sur la fixture partagée : *lettrée `document`* ⇔ *reste dû nul*, pour
/// toute facture dont une ligne de `C(I)` est en période ouverte, hors les deux
/// exceptions nommées — qui, elles, divergent réellement (sans quoi le filtre
/// d'exceptions ne serait exercé par rien).
#[sqlx::test(migrations = "./test-schema")]
async fn ledger_agrees_with_amount_due(pool: MySqlPool) {
    let fixture = seed_lettering_documents(&pool).await;
    let company_id = fixture.seeded.company_id;

    let fautes = divergences_ac5(&pool, company_id, &fixture.exceptions).await;
    assert!(
        fautes.is_empty(),
        "⛔ divergences d'AC5 :\n{}",
        fautes.join("\n")
    );

    // Les exceptions nommées divergent pour de vrai : soldées, non lettrées.
    assert_eq!(fixture.exceptions.len(), 2);
    for (inv, pourquoi) in &fixture.exceptions {
        assert!(
            en_periode_ouverte(&pool, company_id, *inv).await,
            "{pourquoi:?}"
        );
        assert_eq!(
            reste_du(&pool, *inv).await,
            Decimal::ZERO,
            "{pourquoi:?} : soldée"
        );
        assert!(
            !lettree_document(&pool, *inv).await,
            "{pourquoi:?} : non lettrée"
        );
    }
    // Et sans le filtre, le test rougirait en les nommant.
    let sans_filtre = divergences_ac5(&pool, company_id, &[]).await;
    assert_eq!(sans_filtre.len(), 2, "{sans_filtre:#?}");

    // La fixture couvre ce qu'elle annonce — chaque cas, nommé.
    let cas = |nom: &str| {
        fixture
            .factures
            .iter()
            .find(|(n, _)| *n == nom)
            .map(|(_, id)| *id)
            .unwrap_or_else(|| panic!("cas « {nom} » absent de la fixture"))
    };
    for nom in [
        "paiement total",
        "trois partiels",
        "solde",
        "avoir",
        "rapprochement",
    ] {
        assert!(lettree_document(&pool, cas(nom)).await, "{nom} : lettrée");
    }
    for nom in [
        "partiel seul",
        "annulation",
        "paid_at hérité",
        "créditée et réglée",
        "facture de détachement",
    ] {
        assert!(
            !lettree_document(&pool, cas(nom)).await,
            "{nom} : non lettrée"
        );
    }
    // La pièce passée sous la borne est HORS du critère, lettrée avant le verrou.
    let sous_la_borne = cas("soldée puis sous la borne");
    assert!(!en_periode_ouverte(&pool, company_id, sous_la_borne).await);
    assert!(lettree_document(&pool, sous_la_borne).await);
}

// ---------------------------------------------------------------------------
// AC10 — audit
// ---------------------------------------------------------------------------

/// AC10 — `lettering.created` (par le règlement) et `lettering.removed` (par
/// l'annulation) portent la pièce : `documentType`, `documentId`,
/// `documentNumber` ; l'acteur est l'auteur du geste, sans clé d'API.
#[sqlx::test(migrations = "./test-schema")]
async fn audit_details_carry_the_invoice(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let numero: Option<String> =
        sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ?")
            .bind(inv)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(numero.is_some(), "montage : facture numérotée");
    let (sid, _) = regler(&pool, &seeded, inv, dec!(100.00), jours_avant(90)).await;
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("annulation");

    for action in ["lettering.created", "lettering.removed"] {
        let (user_id, api_key_id, details): (i64, Option<i64>, serde_json::Value) = sqlx::query_as(
            "SELECT user_id, actor_api_key_id, details_json FROM audit_log WHERE action = ? \
                 AND JSON_EXTRACT(details_json, '$.origin') = 'document'",
        )
        .bind(action)
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|e| panic!("{action} : {e}"));
        assert_eq!(user_id, seeded.admin_user_id, "{action} : acteur");
        assert_eq!(
            api_key_id, None,
            "{action} : aucune clé hors du rapprochement"
        );
        assert_eq!(details["documentType"], "invoice", "{action}");
        assert_eq!(details["documentId"], inv, "{action}");
        assert_eq!(
            details["documentNumber"],
            serde_json::json!(numero),
            "{action}"
        );
        assert_eq!(
            details["lines"].as_array().map(Vec::len),
            Some(2),
            "{action}"
        );
    }
    // La contre-passation lettre `reversal` SANS clé `document*` (inchangé).
    let reversal: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'lettering.created' \
         AND JSON_EXTRACT(details_json, '$.origin') = 'reversal' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("lettrage reversal");
    assert!(reversal.get("documentType").is_none());
    assert!(reversal.get("documentNumber").is_none());
}

// ---------------------------------------------------------------------------
// AC13 — compte de créance non lettrable
// ---------------------------------------------------------------------------

/// AC13 — trois volets, et la précédence : (a) un règlement sur `A` non
/// lettrable réussit sans lettrer ; (b) un groupe posé avant SURVIT — la
/// synchronisation rend `AccountNotLetterable`, marques et audit inchangés —
/// (c) et se dissout à l'annulation ; (d) `AccountNotLetterable` l'emporte sur
/// `AbstainedClosedPeriods`.
#[sqlx::test(migrations = "./test-schema")]
async fn receivable_not_letterable_is_skipped(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = seeded.accounts["1100"];
    let survivante = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (sid, entry) = regler(&pool, &seeded, survivante, dec!(100.00), jours_avant(90)).await;
    let cle = cle_de(&pool, survivante).await.expect("montage : lettrée");
    let autre = facture(&pool, &seeded, dec!(50.00), jours_avant(100)).await;

    rattacher_a_un_compte_bancaire(&pool, seeded.company_id, a).await;

    // (a) Règlement complet sur `A` non lettrable.
    let audits = audits_de_lettrage(&pool).await;
    regler(&pool, &seeded, autre, dec!(50.00), jours_avant(90)).await;
    assert!(!lettree_document(&pool, autre).await, "(a) aucun groupe");
    assert_eq!(audits_de_lettrage(&pool).await, audits, "(a) aucun audit");

    // (b) Le survivant : étape 3 terminale.
    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    let issue = synchroniser(&pool, &seeded, survivante, seeded.fiscal_year_id)
        .await
        .expect("(b) aucune erreur");
    assert_eq!(issue, SyncOutcome::AccountNotLetterable);
    assert_eq!(
        toutes_les_marques(&pool, seeded.company_id).await,
        marques,
        "(b) marques"
    );
    assert_eq!(audits_de_lettrage(&pool).await, audits, "(b) aucun audit");
    assert!(
        lettree_document(&pool, survivante).await,
        "(b) il survit (C104)"
    );

    // (c) Il se dissout à l'annulation ; la contre-passation saute le compte
    // non lettrable (R6) : la ligne de règlement reste libre.
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        survivante,
        sid,
    )
    .await
    .expect("(c) annulation");
    assert_eq!(
        compter(
            &pool,
            &format!("SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_key = {cle}")
        )
        .await,
        0,
        "(c) dissous"
    );
    assert_eq!(marque(&pool, entry, a).await, (None, None), "(c) libre");

    // (d) Précédence : `autre` est soldée, non lettrée ; sous la borne, son
    // compte non lettrable parle d'abord…
    companies::lock_books(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        jours_avant(90),
    )
    .await
    .expect("verrou");
    assert_eq!(
        synchroniser(&pool, &seeded, autre, seeded.fiscal_year_id)
            .await
            .unwrap(),
        SyncOutcome::AccountNotLetterable,
        "(d) le compte non lettrable l'emporte"
    );
    // … et la même pièce, compte redevenu lettrable, s'abstient : la
    // précédence n'est pas vraie à vide.
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE journal_account_id = ?")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        synchroniser(&pool, &seeded, autre, seeded.fiscal_year_id)
            .await
            .unwrap(),
        SyncOutcome::AbstainedClosedPeriods,
        "(d) contrôle : sinon, l'abstention"
    );
}

// ---------------------------------------------------------------------------
// AC14 — périodes closes, intégrées au geste
// ---------------------------------------------------------------------------

/// AC14 (a), (b) — un groupe posé PAR LE RÈGLEMENT, puis figé par le verrou :
/// l'annulation (règlement, puis solde sur une autre facture) est refusée sans
/// rien écrire ; un déverrouillage avant la date la plus récente la permet, et
/// un nouveau règlement complet après la nouvelle borne relettre — sans
/// `Invariant`.
#[sqlx::test(migrations = "./test-schema")]
async fn locked_period_cancel_is_refused_until_unlocked(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let le = jours_avant(90);
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let (sid, entry) = regler(&pool, &seeded, inv, dec!(100.00), le).await;
    let inv2 = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv2, dec!(80.00), le).await;
    let (sid2, _) = solder(&pool, &seeded, inv2, Nature::Discount, le).await;
    assert!(lettree_document(&pool, inv).await && lettree_document(&pool, inv2).await);

    companies::lock_books(&pool, seeded.admin_user_id, seeded.company_id, le)
        .await
        .expect("verrou");

    let traces = |pool: MySqlPool| async move {
        [
            compter(&pool, "SELECT COUNT(*) FROM journal_entries").await,
            compter(&pool, "SELECT COUNT(*) FROM invoice_settlements").await,
            compter(&pool, "SELECT COUNT(*) FROM audit_log").await,
        ]
    };
    for (facture_id, ligne) in [(inv, sid), (inv2, sid2)] {
        let avant = traces(pool.clone()).await;
        let marques = toutes_les_marques(&pool, seeded.company_id).await;
        let err = invoice_settlements_write::cancel_settlement(
            &pool,
            seeded.admin_user_id,
            seeded.company_id,
            facture_id,
            ligne,
        )
        .await
        .expect_err("(a) refusée");
        assert!(
            matches!(
                err,
                DbError::SettlementNotCancellable {
                    blocker: SettlementCancelBlocker::DocumentLetteringInClosedPeriods
                }
            ),
            "{err:?}"
        );
        assert_eq!(traces(pool.clone()).await, avant, "(a) rien d'écrit");
        assert_eq!(
            toutes_les_marques(&pool, seeded.company_id).await,
            marques,
            "(a) intact"
        );
    }

    // (b) Déverrouillage AVANT la date la plus récente du groupe.
    let nouvelle_borne = le - chrono::Duration::days(1);
    companies::unlock_books(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        Some(nouvelle_borne),
        "annuler un règlement".into(),
    )
    .await
    .expect("déverrouillage");
    invoice_settlements_write::cancel_settlement(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        inv,
        sid,
    )
    .await
    .expect("(b) annulation permise");
    assert_paire_reversal(&pool, entry, seeded.accounts["1100"]).await;
    assert!(!lettree_document(&pool, inv).await, "(b) créance ouverte");

    // Un nouveau règlement complet, après la nouvelle borne : lettre.
    regler(
        &pool,
        &seeded,
        inv,
        dec!(100.00),
        nouvelle_borne + chrono::Duration::days(5),
    )
    .await;
    assert!(
        lettree_document(&pool, inv).await,
        "(b) relettrée, aucun Invariant"
    );
}

/// AC14 (c) — facture et règlements historiques entièrement sous la borne, sans
/// lettrage (marques effacées en SQL brut) : la synchronisation s'abstient et
/// n'écrit rien.
#[sqlx::test(migrations = "./test-schema")]
async fn historical_closed_history_abstains(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let le = jours_avant(90);
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv, dec!(100.00), le).await;
    effacer_marques(&pool, cle_de(&pool, inv).await.expect("lettrée")).await;
    companies::lock_books(&pool, seeded.admin_user_id, seeded.company_id, le)
        .await
        .expect("verrou");
    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    let audits = audits_de_lettrage(&pool).await;

    let issue = synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
        .await
        .expect("aucune erreur");
    assert_eq!(issue, SyncOutcome::AbstainedClosedPeriods);
    assert_eq!(toutes_les_marques(&pool, seeded.company_id).await, marques);
    assert_eq!(audits_de_lettrage(&pool).await, audits);
}

// ---------------------------------------------------------------------------
// P3 — la synchronisation elle-même
// ---------------------------------------------------------------------------

/// P3 (validation P3, F-5) — un second appel sur une facture lettrée rend
/// `Unchanged`, n'écrit rien, ne produit aucune entrée d'audit.
#[sqlx::test(migrations = "./test-schema")]
async fn sync_is_idempotent(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv, dec!(100.00), jours_avant(90)).await;
    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    let audits = audits_de_lettrage(&pool).await;
    for _ in 0..2 {
        assert_eq!(
            synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
                .await
                .unwrap(),
            SyncOutcome::Unchanged
        );
    }
    assert_eq!(toutes_les_marques(&pool, seeded.company_id).await, marques);
    assert_eq!(audits_de_lettrage(&pool).await, audits);
}

/// P3 étape 2 — une marque étrangère sur `C(I)` (`manual`, `reversal`, ou deux
/// clés distinctes, fabriquées en SQL brut) rend `Invariant`, jamais un
/// écrasement : marques inchangées.
#[sqlx::test(migrations = "./test-schema")]
async fn sync_never_overwrites_a_foreign_mark(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv, dec!(60.00), jours_avant(90)).await;
    regler(&pool, &seeded, inv, dec!(40.00), jours_avant(90)).await;
    effacer_marques(&pool, cle_de(&pool, inv).await.expect("lettrée")).await;
    let lignes = lignes_de_piece(&pool, inv).await;
    let (ancre, r1) = (lignes[0].0, lignes[1].0);

    let poser = |pool: MySqlPool, sql: &'static str, binds: Vec<i64>| async move {
        // Base éphémère du test : effacer TOUTES les marques est sans portée.
        sqlx::query("UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL")
            .execute(&pool)
            .await
            .unwrap();
        let mut q = sqlx::query(sql);
        for b in binds {
            q = q.bind(b);
        }
        assert!(
            q.execute(&pool).await.unwrap().rows_affected() >= 1,
            "montage : {sql}"
        );
    };
    for (nom, sql, binds) in [
        (
            "manual",
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'manual' WHERE id = ?",
            vec![ancre, ancre],
        ),
        (
            "reversal",
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'reversal' WHERE id = ?",
            vec![ancre, ancre],
        ),
        (
            "deux clés",
            "UPDATE journal_entry_lines SET lettering_key = id, lettering_origin = 'document' \
             WHERE id IN (?, ?)",
            vec![ancre, r1],
        ),
    ] {
        poser(pool.clone(), sql, binds).await;
        let marques = toutes_les_marques(&pool, seeded.company_id).await;
        let err = synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
            .await
            .expect_err(nom);
        assert!(matches!(err, DbError::Invariant(_)), "{nom} : {err:?}");
        assert_eq!(
            toutes_les_marques(&pool, seeded.company_id).await,
            marques,
            "{nom} : marques inchangées"
        );
    }
}

/// P3 étape 5 — défensive, fabriquée en SQL brut : un groupe `document` d'une
/// AUTRE cible avec une ligne ouverte → `Recreated` ; le même entièrement sous
/// la borne → `AbstainedClosedPeriods`, rien d'écrit. ⚠️ Montage : l'exercice
/// tenu (seedé) couvre une ligne du groupe `k` et une de la cible.
#[sqlx::test(migrations = "./test-schema")]
async fn sync_step_5_recreates_or_abstains(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    regler(&pool, &seeded, inv, dec!(60.00), jours_avant(90)).await;
    regler(&pool, &seeded, inv, dec!(40.00), jours_avant(90)).await;
    let cle = cle_de(&pool, inv).await.expect("lettrée");
    let lignes = lignes_de_piece(&pool, inv).await;
    let (ancre, r1) = (lignes[0].0, lignes[1].0);

    let fabriquer = |pool: MySqlPool| async move {
        effacer_marques(&pool, cle).await;
        let n = sqlx::query(
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'document' \
             WHERE id IN (?, ?)",
        )
        .bind(ancre)
        .bind(ancre)
        .bind(r1)
        .execute(&pool)
        .await
        .unwrap()
        .rows_affected();
        assert_eq!(n, 2, "montage : groupe {{ancre, r1}}");
    };

    // Une ligne ouverte : défait, puis pose la cible (trois lignes).
    fabriquer(pool.clone()).await;
    let issue = synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
        .await
        .expect("recréation");
    assert_eq!(
        issue,
        SyncOutcome::Recreated {
            dissolved: ancre,
            created: ancre
        }
    );
    assert!(lettree_document(&pool, inv).await);
    assert_eq!(lignes_de_piece(&pool, inv).await.len(), 3);

    // Entièrement sous la borne : abstention, rien d'écrit.
    fabriquer(pool.clone()).await;
    companies::lock_books(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        jours_avant(90),
    )
    .await
    .expect("verrou");
    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    let audits = audits_de_lettrage(&pool).await;
    assert_eq!(
        synchroniser(&pool, &seeded, inv, seeded.fiscal_year_id)
            .await
            .unwrap(),
        SyncOutcome::AbstainedClosedPeriods
    );
    assert_eq!(toutes_les_marques(&pool, seeded.company_id).await, marques);
    assert_eq!(audits_de_lettrage(&pool).await, audits);
}

/// P1 — l'ANCRE est la première ligne au débit de l'écriture de vente, seule :
/// une autre ligne de la vente sur `A` (un arrondi négatif qui débiterait la
/// créance — fabriqué ici en SQL brut, ligne équilibrée par un crédit au
/// produit) n'entre pas dans `C(I)`. Sans ce filtre, la somme ne serait plus
/// nulle et la facture soldée ne serait pas lettrée (mutation M12).
#[sqlx::test(migrations = "./test-schema")]
async fn only_the_anchor_of_the_sale_is_in_the_group(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let inv = facture(&pool, &seeded, dec!(100.00), jours_avant(100)).await;
    let v = vente(&pool, inv).await;
    let ordre: i32 =
        sqlx::query_scalar("SELECT MAX(line_order) FROM journal_entry_lines WHERE entry_id = ?")
            .bind(v)
            .fetch_one(&pool)
            .await
            .unwrap();
    for (n, compte, debit, credit) in [
        (1, seeded.accounts["1100"], dec!(5.00), dec!(0)),
        (2, seeded.accounts["3000"], dec!(0), dec!(5.00)),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(v)
        .bind(compte)
        .bind(ordre + n)
        .bind(debit)
        .bind(credit)
        .execute(&pool)
        .await
        .expect("ligne de vente supplémentaire (montage)");
    }
    regler(&pool, &seeded, inv, dec!(100.00), jours_avant(90)).await;

    assert!(
        lettree_document(&pool, inv).await,
        "l'ancre et le règlement"
    );
    let seconde: (Option<i64>, Option<String>) = sqlx::query_as(
        "SELECT lettering_key, lettering_origin FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ? AND debit = 5.00",
    )
    .bind(v)
    .bind(seeded.accounts["1100"])
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        seconde,
        (None, None),
        "la seconde ligne de la vente sur A reste ouverte"
    );
}

// ===========================================================================
// Story 15-1a2-ii — les pièces FOURNISSEURS
// ===========================================================================

/// La synchronisation d'une facture fournisseur, dans une transaction qui tient
/// l'exercice `fy` `FOR UPDATE` ; valide si elle réussit.
async fn synchroniser_fournisseur(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    id: i64,
    fy: i64,
) -> Result<SyncOutcome, DbError> {
    let mut tx = pool.begin().await.expect("tx");
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(fy)
        .execute(&mut *tx)
        .await
        .expect("exercice tenu");
    let issue = letterings::sync_supplier_invoice_in_tx(
        &mut tx,
        seeded.company_id,
        id,
        fy,
        Actor {
            user_id: seeded.admin_user_id,
            api_key_id: None,
        },
    )
    .await;
    if issue.is_ok() {
        tx.commit().await.expect("commit");
    }
    issue
}

/// Le statut d'une facture fournisseur.
async fn statut_fournisseur(pool: &MySqlPool, id: i64) -> String {
    sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("statut")
}

/// AC7 — un paiement direct lettre `document` l'achat et le paiement sur la
/// dette `B` ; la clé est la ligne d'achat ; la ligne bancaire (compte non
/// lettrable) reste ouverte ; la synchronisation rejouée ne change rien.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_payment_letters_purchase_and_payment(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(120.00),
        jours_avant(100),
        Some("FF-1"),
    )
    .await;
    let reglement = payer(&pool, &seeded, &a, s, jours_avant(90)).await;

    assert!(
        fournisseur_lettree_document(&pool, s).await,
        "achat et paiement lettrés"
    );
    let lignes = lignes_de_piece_fournisseur(&pool, s).await;
    assert_eq!(lignes.len(), 2, "l'ancre et la ligne du règlement");
    assert_eq!(
        lignes[0].1,
        achat(&pool, s).await,
        "l'ancre est sur l'achat"
    );
    assert_eq!(lignes[0].2, Some(lignes[0].0), "clé = plus petite ligne");
    assert_eq!(
        marque(&pool, reglement, a.bank_ledger).await,
        (None, None),
        "compte bancaire : non lettrable"
    );
    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    assert_eq!(
        synchroniser_fournisseur(&pool, &seeded, s, seeded.fiscal_year_id)
            .await
            .expect("synchronisation"),
        SyncOutcome::Unchanged,
        "idempotente"
    );
    assert_eq!(toutes_les_marques(&pool, seeded.company_id).await, marques);
}

/// AC7 — la confirmation d'un lot pain.001 (`confirm_batch`, N paiements dans
/// une transaction) lettre chaque facture, chacune sous sa propre clé.
#[sqlx::test(migrations = "./test-schema")]
async fn batch_confirm_letters(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s1 = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(100.00),
        jours_avant(100),
        Some("L-1"),
    )
    .await;
    let s2 = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(200.00),
        jours_avant(100),
        Some("L-2"),
    )
    .await;
    payer_par_lot(&pool, &seeded, &a, vec![s1, s2], jours_avant(80)).await;

    for s in [s1, s2] {
        assert_eq!(statut_fournisseur(&pool, s).await, "paid");
        assert!(
            fournisseur_lettree_document(&pool, s).await,
            "facture {s} lettrée"
        );
    }
    let k1 = lignes_de_piece_fournisseur(&pool, s1).await[0].2;
    let k2 = lignes_de_piece_fournisseur(&pool, s2).await[0].2;
    assert_ne!(k1, k2, "un groupe par facture");
}

/// AC7 — l'annulation du règlement dissout le groupe, lettre règlement et
/// miroir `reversal`, et laisse l'achat ouvert ; la facture redevient `open`.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_settlement_cancel_dissolves_and_pairs(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(75.00),
        jours_avant(100),
        Some("FF-2"),
    )
    .await;
    let reglement = payer(&pool, &seeded, &a, s, jours_avant(90)).await;
    assert!(fournisseur_lettree_document(&pool, s).await);

    supplier_invoices::cancel_settlement(&pool, seeded.company_id, s, seeded.admin_user_id)
        .await
        .expect("annulation du règlement");
    assert_eq!(statut_fournisseur(&pool, s).await, "open");
    assert_eq!(
        marque(&pool, achat(&pool, s).await, a.payable).await,
        (None, None),
        "l'achat est ouvert"
    );
    assert_paire_reversal(&pool, reglement, a.payable).await;
    let miroir = contre_passation(&pool, reglement).await;
    assert_eq!(
        marque(&pool, miroir, a.bank_ledger).await,
        (None, None),
        "banque : non lettrable, son miroir reste ouvert"
    );
}

/// AC7 — l'annulation d'une facture PAYÉE dissout le groupe, lettre l'achat et
/// son miroir `reversal`, et laisse le règlement DÉTACHÉ ouvert — lettrable à la
/// main avec une ligne de même compte qui le solde (il n'est plus possédé).
#[sqlx::test(migrations = "./test-schema")]
async fn cancel_paid_supplier_invoice_detaches_an_open_payment(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(60.00),
        jours_avant(100),
        Some("FF-3"),
    )
    .await;
    let reglement = payer(&pool, &seeded, &a, s, jours_avant(90)).await;
    let purchase = achat(&pool, s).await;

    supplier_invoices::cancel(&pool, seeded.company_id, s, seeded.admin_user_id)
        .await
        .expect("annulation de la facture payée");
    assert_eq!(statut_fournisseur(&pool, s).await, "cancelled");
    assert_paire_reversal(&pool, purchase, a.payable).await;
    assert_eq!(
        marque(&pool, reglement, a.payable).await,
        (None, None),
        "le règlement détaché est ouvert"
    );

    // Lettrable à la main : une écriture qui crédite `B` du même montant.
    let (_, _, credit) = ecriture_manuelle(
        &pool,
        &seeded,
        seeded.accounts["4000"],
        a.payable,
        dec!(60.00),
        jours_avant(50),
    )
    .await;
    let detache: i64 = sqlx::query_scalar(
        "SELECT id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(reglement)
    .bind(a.payable)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    let groupe = letterings::create_group_in_tx(
        &mut tx,
        seeded.company_id,
        &[detache, credit],
        letterings::Origin::Manual,
        letterings::Mode::Manual,
        Actor {
            user_id: seeded.admin_user_id,
            api_key_id: None,
        },
    )
    .await
    .expect("le règlement détaché se lettre à la main");
    tx.commit().await.unwrap();
    assert_eq!(groupe.key, detache);
}

/// AC7 (second volet), AC9 — le règlement détaché, CONTRE-PASSÉ par sa fiche
/// d'écriture, est lettré `reversal` avec son miroir ; il n'est jamais `document`.
#[sqlx::test(migrations = "./test-schema")]
async fn detached_payment_reversed_is_lettered_reversal(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(45.00),
        jours_avant(100),
        Some("FF-4"),
    )
    .await;
    let reglement = payer(&pool, &seeded, &a, s, jours_avant(90)).await;
    supplier_invoices::cancel(&pool, seeded.company_id, s, seeded.admin_user_id)
        .await
        .expect("annulation de la facture payée");

    journal_entries::reverse(&pool, seeded.company_id, reglement, seeded.admin_user_id)
        .await
        .expect("le règlement détaché se contre-passe");
    assert_paire_reversal(&pool, reglement, a.payable).await;
    assert_paire_reversal(&pool, achat(&pool, s).await, a.payable).await;
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_origin = 'document'"
        )
        .await,
        0,
        "plus aucune ligne `document`"
    );
}

/// AC7 — paiement par COMPTE INTERNE lettrable (`1000`), puis annulation : la
/// ligne de contrepartie et son miroir sont aussi lettrés `reversal` (R6).
#[sqlx::test(migrations = "./test-schema")]
async fn internal_account_payment_pairs_its_counterpart_on_cancel(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(33.00),
        jours_avant(100),
        Some("FF-5"),
    )
    .await;
    let reglement = payer_par(
        &pool,
        &seeded,
        s,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        jours_avant(90),
    )
    .await;
    assert!(fournisseur_lettree_document(&pool, s).await);
    assert_eq!(
        marque(&pool, reglement, seeded.accounts["1000"]).await,
        (None, None),
        "la contrepartie n'est pas dans le groupe de la pièce"
    );

    supplier_invoices::cancel_settlement(&pool, seeded.company_id, s, seeded.admin_user_id)
        .await
        .expect("annulation du règlement");
    assert_paire_reversal(&pool, reglement, a.payable).await;
    assert_paire_reversal(&pool, reglement, seeded.accounts["1000"]).await;
}

/// AC7 — dette `B` non lettrable (rattachée à un compte bancaire) : le paiement
/// réussit, aucun groupe, aucune entrée d'audit de lettrage.
#[sqlx::test(migrations = "./test-schema")]
async fn payable_not_letterable_is_skipped(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    rattacher_a_un_compte_bancaire(&pool, seeded.company_id, a.payable).await;
    let s = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(25.00),
        jours_avant(100),
        Some("FF-6"),
    )
    .await;
    payer(&pool, &seeded, &a, s, jours_avant(90)).await;

    assert_eq!(
        statut_fournisseur(&pool, s).await,
        "paid",
        "paiement réussi"
    );
    assert!(
        lignes_de_piece_fournisseur(&pool, s)
            .await
            .iter()
            .all(|l| l.2.is_none()),
        "aucune marque"
    );
    assert_eq!(audits_de_lettrage(&pool).await, 0);
    assert_eq!(
        synchroniser_fournisseur(&pool, &seeded, s, seeded.fiscal_year_id)
            .await
            .expect("synchronisation"),
        SyncOutcome::AccountNotLetterable
    );
}

/// AC7 (périodes closes) — groupe posé PAR LE PAIEMENT, verrou posé ensuite :
/// au dépôt, l'annulation du paiement et celle de la facture sont refusées
/// (`DocumentLetteringInClosedPeriods`), rien n'est écrit, le groupe est intact,
/// la facture reste `paid`, et les deux prédicteurs rendent le motif. Après
/// déverrouillage (borne avant la date la plus récente du groupe), les deux
/// passent — dissolution et paires `reversal`.
#[sqlx::test(migrations = "./test-schema")]
async fn locked_period_supplier_cancels_are_refused_until_unlocked(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let le = jours_avant(90);
    let s1 = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(40.00),
        jours_avant(100),
        Some("V-1"),
    )
    .await;
    let r1 = payer(&pool, &seeded, &a, s1, le).await;
    let s2 = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(50.00),
        jours_avant(100),
        Some("V-2"),
    )
    .await;
    let r2 = payer(&pool, &seeded, &a, s2, le).await;
    assert!(fournisseur_lettree_document(&pool, s1).await);
    assert!(fournisseur_lettree_document(&pool, s2).await);
    companies::lock_books(&pool, seeded.admin_user_id, seeded.company_id, le)
        .await
        .expect("verrou");

    let traces = |pool: MySqlPool| async move {
        [
            compter(&pool, "SELECT COUNT(*) FROM journal_entries").await,
            compter(&pool, "SELECT COUNT(*) FROM audit_log").await,
        ]
    };
    let avant = traces(pool.clone()).await;
    let marques = toutes_les_marques(&pool, seeded.company_id).await;

    let err =
        supplier_invoices::cancel_settlement(&pool, seeded.company_id, s1, seeded.admin_user_id)
            .await
            .expect_err("annulation du paiement refusée");
    assert!(
        matches!(
            err,
            DbError::SettlementNotCancellable {
                blocker: SettlementCancelBlocker::DocumentLetteringInClosedPeriods
            }
        ),
        "{err:?}"
    );
    let err = supplier_invoices::cancel(&pool, seeded.company_id, s2, seeded.admin_user_id)
        .await
        .expect_err("annulation de la facture refusée");
    assert!(
        matches!(
            err,
            DbError::SupplierInvoiceNotCancellable {
                blocker: SettlementCancelBlocker::DocumentLetteringInClosedPeriods
            }
        ),
        "{err:?}"
    );
    assert_eq!(traces(pool.clone()).await, avant, "rien d'écrit");
    assert_eq!(
        toutes_les_marques(&pool, seeded.company_id).await,
        marques,
        "groupes intacts"
    );
    assert_eq!(statut_fournisseur(&pool, s1).await, "paid");
    assert_eq!(statut_fournisseur(&pool, s2).await, "paid");

    let mut conn = pool.acquire().await.unwrap();
    let p1 =
        supplier_invoices::supplier_settlement_cancel_blocker(&mut conn, seeded.company_id, s1)
            .await
            .unwrap()
            .map(|h| h.0);
    let p2 = supplier_invoices::supplier_invoice_cancel_blocker(&mut conn, seeded.company_id, s2)
        .await
        .unwrap()
        .map(|h| h.0);
    drop(conn);
    assert_eq!(
        p1,
        Some(SettlementCancelBlocker::DocumentLetteringInClosedPeriods)
    );
    assert_eq!(
        p2,
        Some(SettlementCancelBlocker::DocumentLetteringInClosedPeriods)
    );

    companies::unlock_books(
        &pool,
        seeded.admin_user_id,
        seeded.company_id,
        Some(le - chrono::Duration::days(1)),
        "annuler des paiements".into(),
    )
    .await
    .expect("déverrouillage");
    supplier_invoices::cancel_settlement(&pool, seeded.company_id, s1, seeded.admin_user_id)
        .await
        .expect("paiement annulé après déverrouillage");
    assert_paire_reversal(&pool, r1, a.payable).await;
    assert_eq!(
        marque(&pool, achat(&pool, s1).await, a.payable).await,
        (None, None)
    );
    supplier_invoices::cancel(&pool, seeded.company_id, s2, seeded.admin_user_id)
        .await
        .expect("facture annulée après déverrouillage");
    assert_paire_reversal(&pool, achat(&pool, s2).await, a.payable).await;
    assert_eq!(
        marque(&pool, r2, a.payable).await,
        (None, None),
        "règlement détaché ouvert"
    );
}

/// P2 (découverte par statut), AC6 (e) — une facture fournisseur ANNULÉE (son
/// achat lettré `reversal`) et une facture OUVERTE : la synchronisation rend
/// `Unchanged`, sans écriture ni audit — jamais `Invariant`.
#[sqlx::test(migrations = "./test-schema")]
async fn cancelled_supplier_invoice_sync_is_unchanged(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let annulee = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(10.00),
        jours_avant(100),
        Some("A-1"),
    )
    .await;
    payer(&pool, &seeded, &a, annulee, jours_avant(90)).await;
    supplier_invoices::cancel(&pool, seeded.company_id, annulee, seeded.admin_user_id)
        .await
        .expect("annulation");
    assert_paire_reversal(&pool, achat(&pool, annulee).await, a.payable).await;
    let ouverte = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(11.00),
        jours_avant(100),
        Some("A-2"),
    )
    .await;

    let marques = toutes_les_marques(&pool, seeded.company_id).await;
    let audits = audits_de_lettrage(&pool).await;
    for s in [annulee, ouverte] {
        assert_eq!(
            synchroniser_fournisseur(&pool, &seeded, s, seeded.fiscal_year_id)
                .await
                .expect("synchronisation"),
            SyncOutcome::Unchanged,
            "facture {s}"
        );
    }
    assert_eq!(toutes_les_marques(&pool, seeded.company_id).await, marques);
    assert_eq!(audits_de_lettrage(&pool).await, audits);
}

/// AC10 (part ii) — `lettering.created` (paiement) et `lettering.removed`
/// (annulation) portent la facture fournisseur : `documentType =
/// "supplierInvoice"`, `documentId`, `documentNumber` — présent et `null` pour
/// une facture SANS numéro ; acteur : l'auteur du geste, sans clé d'API.
#[sqlx::test(migrations = "./test-schema")]
async fn supplier_audit_details_carry_the_invoice(pool: MySqlPool) {
    let seeded = societe(&pool).await;
    let a = achats(&pool, &seeded).await;
    let numerotee = facture_fournisseur(
        &pool,
        &seeded,
        &a,
        dec!(70.00),
        jours_avant(100),
        Some("FF-77"),
    )
    .await;
    let anonyme =
        facture_fournisseur(&pool, &seeded, &a, dec!(80.00), jours_avant(100), None).await;
    for s in [numerotee, anonyme] {
        payer(&pool, &seeded, &a, s, jours_avant(90)).await;
        supplier_invoices::cancel_settlement(&pool, seeded.company_id, s, seeded.admin_user_id)
            .await
            .expect("annulation du règlement");
    }

    for (s, numero) in [
        (numerotee, serde_json::json!("FF-77")),
        (anonyme, serde_json::Value::Null),
    ] {
        for action in ["lettering.created", "lettering.removed"] {
            let (user_id, api_key_id, details): (i64, Option<i64>, serde_json::Value) =
                sqlx::query_as(
                    "SELECT user_id, actor_api_key_id, details_json FROM audit_log \
                     WHERE action = ? AND JSON_EXTRACT(details_json, '$.origin') = 'document' \
                       AND JSON_EXTRACT(details_json, '$.documentId') = ?",
                )
                .bind(action)
                .bind(s)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|e| panic!("{action} / {s} : {e}"));
            assert_eq!(user_id, seeded.admin_user_id, "{action}");
            assert_eq!(api_key_id, None, "{action}");
            assert_eq!(details["documentType"], "supplierInvoice", "{action}");
            assert_eq!(details["documentId"], s, "{action}");
            let objet = details.as_object().expect("objet");
            assert!(
                objet.contains_key("documentNumber"),
                "{action} : clé présente"
            );
            assert_eq!(details["documentNumber"], numero, "{action}");
            assert_eq!(
                details["lines"].as_array().map(Vec::len),
                Some(2),
                "{action}"
            );
        }
    }
}
