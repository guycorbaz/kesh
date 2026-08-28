//! Story 16-1c (#281) — **AC-C6.3 et AC-C6.4** : les deux propriétés qui rendent
//! une entrée de **classe A** légitime.
//!
//! # Pourquoi ces deux tests, et pourquoi ensemble
//!
//! Une entrée de classe A est rejouée **inconditionnellement, à chaque import**.
//! Rien ne la conditionne : ni sentinelle, ni version du backup. Sa sûreté tient
//! donc entièrement à une affirmation faite en prose au registre — *« tous ses
//! statements sont gardés contre l'écrasement d'une valeur posée par
//! l'utilisateur »*. Tant que cette affirmation n'est pas exécutée contre une
//! base, elle n'est qu'un commentaire.
//!
//! - **AC-C6.3 — no-op sur une base nominale à jour** (`rows_affected == 0`).
//!   C'est la propriété de **sûreté**. Elle a un précédent : `20260628000001`
//!   satisfaisait « idempotent au second passage » et violait pourtant celle-ci,
//!   en réattribuant le compte 2000. **Ce n'est pas la même propriété**, et c'est
//!   la distinction qui compte — un test d'idempotence n'aurait rien attrapé.
//! - **AC-C6.4 — non-vacuité** (`rows_affected > 0` sur une base en amont).
//!   C'est la propriété d'**utilité**, et elle existe parce que sans elle le test
//!   précédent serait vrai à vide : un SQL qui ne touche jamais rien est
//!   trivialement un no-op.
//!
//! Aucun des deux ne prouve quoi que ce soit seul. Ensemble, ils encadrent le
//! SQL réellement embarqué : il agit **là où il doit**, et **nulle part
//! ailleurs**.
//!
//! # Le montage, et le piège qu'il évite
//!
//! Les deux tests tournent sous `MIGRATOR` **complet** — pas de fenêtre de
//! migrations partielle. C'est délibéré : indexer les migrations par position est
//! le mode d'échec du garde-fou **P6** de `CLAUDE.md`, et il est inutile ici. Le
//! backfill de `20260729000001` a déjà tourné quand le test démarre, sur des
//! tables vides ; les données sont insérées **après**, et c'est le rejeu
//! post-restore — pas la migration — que ces tests exercent.
//!
//! ⚠️ **Reconstituer l'état pré-migration doit être EXPLICITE.** La 16-1a
//! matérialise `revenue_account_id` à la validation : une fixture qui se
//! contenterait de créer une facture validée mesurerait `0` ligne touchée et le
//! test de non-vacuité passerait **à vide**. D'où l'`UPDATE … SET
//! revenue_account_id = NULL` posé explicitement, sur le patron de
//! `invoice_lines_revenue_account_backfill.rs`.

use std::collections::BTreeMap;

use kesh_db::post_restore::{
    BackfillTrigger, POST_RESTORE_BACKFILLS, PostRestoreBackfill, RETIRED_BACKFILLS, ReplayOutcome,
    replay_with_registry,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use sqlx::MySqlPool;

/// Montant HT de la facture de fixture — égal au crédit de produit de son
/// écriture, ce qui satisfait la condition (3) du backfill.
const HT: &str = "1000.0000";
const VAT: &str = "81.0000";
const TTC: &str = "1081.0000";
const ZERO: &str = "0.0000";

/// Les entrées de **classe A** à éprouver.
///
/// Résolues depuis les deux registres et non énumérées à la main : une entrée de
/// classe A ajoutée plus tard est automatiquement soumise aux deux propriétés,
/// sans que personne ait à y penser.
///
/// ⚠️ **`RETIRED_BACKFILLS` entre dans la source depuis la Story 24-2 (#371).**
/// La création de `invoice_settlements` a refermé la fenêtre d'importabilité
/// au-delà des deux entrées que portait `POST_RESTORE_BACKFILLS`, qui est donc
/// **vide** — et ces deux tests, qui en dérivaient, tourneraient à vide. Les
/// entrées retirées restent des cas d'espèce parfaitement valides pour éprouver
/// la MACHINERIE : c'est elle qu'on protège ici, pas le fait qu'une entrée soit
/// aujourd'hui atteignable en production.
fn class_a_entries() -> Vec<PostRestoreBackfill> {
    POST_RESTORE_BACKFILLS
        .iter()
        .chain(RETIRED_BACKFILLS.iter())
        .filter(|e| matches!(e.trigger, BackfillTrigger::Unconditional))
        .copied()
        .collect()
}

/// Garde de non-vacuité des deux tests : sans entrée de classe A, leurs boucles
/// ne tournent pas et ils passent en ne vérifiant rien.
fn assert_class_a_is_not_empty(entries: &[PostRestoreBackfill]) {
    assert!(
        !entries.is_empty(),
        "aucune entrée de classe A au registre — ces deux tests tourneraient À VIDE. Si la \
         dernière entrée de classe A a été retirée, retirer aussi ces tests plutôt que de les \
         laisser verts sans objet."
    );
}

async fn insert_contact(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', 'Client 16-1c', TRUE, FALSE)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("insert contact")
    .last_insert_id() as i64
}

/// Écriture **canonique** de vente : débit créance TTC, crédit produit HT,
/// crédit TVA. C'est la structure que reconnaît le critère d'unicité en trois
/// conditions du backfill — l'unique ligne dont le crédit égale `total_amount`
/// est celle du compte de produit.
async fn insert_canonical_entry(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    entry_number: i64,
) -> i64 {
    let entry_id = sqlx::query(
        "INSERT INTO journal_entries \
         (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-03-01', 'Ventes', 'Fixture 16-1c')",
    )
    .bind(seeded.company_id)
    .bind(seeded.fiscal_year_id)
    .bind(entry_number)
    .execute(pool)
    .await
    .expect("insert journal_entry")
    .last_insert_id() as i64;

    let lines: [(i64, &str, &str); 3] = [
        (seeded.accounts["1100"], TTC, ZERO), // créance
        (seeded.accounts["3000"], ZERO, HT),  // produit — le candidat attendu
        (seeded.accounts["2000"], ZERO, VAT), // TVA due
    ];
    for (order, (account_id, debit, credit)) in lines.iter().enumerate() {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(account_id)
        .bind(order as i32 + 1)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .expect("insert journal_entry_line");
    }

    entry_id
}

/// Facture à **deux** lignes — le backfill pose le compte sur *toutes* les
/// lignes de la pièce, ce qu'une facture mono-ligne ne prouverait pas.
async fn insert_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    number: &str,
    status: &str,
    journal_entry_id: Option<i64>,
    revenue_account_id: Option<i64>,
) -> i64 {
    let invoice_id = sqlx::query(
        "INSERT INTO invoices \
         (company_id, contact_id, invoice_number, status, date, total_amount, journal_entry_id) \
         VALUES (?, ?, ?, ?, '2026-03-01', ?, ?)",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(number)
    .bind(status)
    .bind(HT)
    .bind(journal_entry_id)
    .execute(pool)
    .await
    .expect("insert invoice")
    .last_insert_id() as i64;

    for (position, line_total) in [(1, "600.0000"), (2, "400.0000")] {
        sqlx::query(
            "INSERT INTO invoice_lines \
             (invoice_id, position, description, quantity, unit_price, vat_rate, line_total, \
              revenue_account_id) \
             VALUES (?, ?, 'Prestation', 1, ?, 8.10, ?, ?)",
        )
        .bind(invoice_id)
        .bind(position)
        .bind(line_total)
        .bind(line_total)
        .bind(revenue_account_id)
        .execute(pool)
        .await
        .expect("insert invoice_line");
    }

    invoice_id
}

/// Second compte de produit — le plan de la fixture n'en porte qu'un.
async fn insert_revenue_account(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Ventes secondaires', 'Revenue')",
    )
    .bind(company_id)
    .bind(number)
    .execute(pool)
    .await
    .expect("insert compte de produit")
    .last_insert_id() as i64
}

async fn line_accounts(pool: &MySqlPool, invoice_id: i64) -> Vec<Option<i64>> {
    sqlx::query_scalar(
        "SELECT revenue_account_id FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .expect("select invoice_lines.revenue_account_id")
}

/// Rejoue les entrées de classe A dans une transaction **committée**, et rend le
/// total des lignes touchées.
///
/// Le manifeste passé est vide : il n'entre pas dans la décision d'une entrée
/// inconditionnelle, et c'est précisément la propriété testée.
async fn replay_class_a(pool: &MySqlPool, entries: &[PostRestoreBackfill]) -> u64 {
    let mut tx = pool.begin().await.expect("begin");
    let report = replay_with_registry(&mut tx, &BTreeMap::new(), entries)
        .await
        .expect("le rejeu des entrées de classe A doit réussir");
    tx.commit().await.expect("commit");

    assert_eq!(
        report.len(),
        entries.len(),
        "le rapport doit porter une entrée par entrée rejouée"
    );
    for r in &report {
        assert_eq!(
            r.outcome,
            ReplayOutcome::ReplayedUnconditional,
            "une entrée de classe A ne peut être ni conditionnée ni sautée — entrée {}",
            r.label
        );
    }
    report.iter().map(|r| r.rows_affected).sum()
}

/// **AC-C6.4 — non-vacuité.** Sur une base reconstituée dans son état
/// **antérieur** à la migration, le rejeu de classe A touche au moins une ligne.
///
/// Sans ce test, `class_a_entries_are_no_ops_on_a_nominal_up_to_date_base`
/// serait vrai à vide : un `UPDATE` dont la clause `WHERE` ne désigne jamais
/// rien est trivialement un no-op, et le registre pourrait embarquer du SQL
/// mort sans que rien ne le dise.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn class_a_entries_are_not_vacuous_on_a_pre_migration_base(pool: MySqlPool) {
    let entries = class_a_entries();
    assert_class_a_is_not_empty(&entries);

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let entry = insert_canonical_entry(&pool, &seeded, 1).await;

    // L'état du parc AVANT la 16-1a : facture validée, lignes à `NULL`.
    let invoice_id = insert_invoice(
        &pool,
        seeded.company_id,
        contact_id,
        "F-16-1c-001",
        "validated",
        Some(entry),
        None,
    )
    .await;
    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![None, None],
        "pré-condition : la fixture reconstitue bien l'état pré-migration"
    );

    let touched = replay_class_a(&pool, &entries).await;

    assert!(
        touched > 0,
        "le rejeu de classe A n'a touché AUCUNE ligne alors que la base est dans l'état que son \
         SQL vise. Le registre embarque du SQL qui n'agit jamais — et le test de no-op qui \
         l'accompagne est alors vrai à vide."
    );
    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "le rejeu doit poser, sur TOUTES les lignes, le compte que l'écriture a réellement crédité"
    );
}

/// **AC-C6.3 — no-op sur une base nominale à jour.** Le rejeu inconditionnel ne
/// doit toucher **aucune** ligne d'une installation en usage courant.
///
/// # ⚠️ Le montage qui rend ce test DISCRIMINANT, et le piège qu'il évite
///
/// La facture validée porte sur ses lignes un compte **différent** de celui que
/// son écriture crédite : lignes sur `3200`, écriture sur `3000`. C'est un état
/// nominal — la divergence entre le compte d'une ligne et celui de l'écriture est
/// documentée et légitime (défaut société changé depuis, ou compte corrigé à la
/// main) — et c'est surtout le **seul** montage qui teste quelque chose.
///
/// Monté avec des lignes portant `3000`, soit la valeur même que le backfill
/// écrirait, le test serait **muet** : MariaDB ne compte dans `rows_affected` que
/// les lignes réellement **modifiées**, donc un rejeu ayant perdu sa garde
/// `revenue_account_id IS NULL` réécrirait `3000` sur `3000` et rapporterait
/// quand même `0`. Le test aurait alors la forme d'une preuve sans en être une.
///
/// S'y ajoute une facture **brouillon** aux lignes `NULL` — état nominal, la
/// liaison du compte étant *tardive* par conception — qui borne la population
/// visée par le bas.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn class_a_entries_are_no_ops_on_a_nominal_up_to_date_base(pool: MySqlPool) {
    let entries = class_a_entries();
    assert_class_a_is_not_empty(&entries);

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let entry = insert_canonical_entry(&pool, &seeded, 1).await;
    // Compte porté par les lignes, DIFFÉRENT de celui que l'écriture crédite.
    let line_account = insert_revenue_account(&pool, seeded.company_id, "3200").await;
    assert_ne!(
        line_account, seeded.accounts["3000"],
        "le montage EXIGE deux comptes distincts — cf. le piège décrit ci-dessus"
    );

    let validated = insert_invoice(
        &pool,
        seeded.company_id,
        contact_id,
        "F-16-1c-010",
        "validated",
        Some(entry),
        Some(line_account),
    )
    .await;
    let draft = insert_invoice(
        &pool,
        seeded.company_id,
        contact_id,
        "F-16-1c-011",
        "draft",
        None,
        None,
    )
    .await;

    let touched = replay_class_a(&pool, &entries).await;

    assert_eq!(
        touched, 0,
        "le rejeu de classe A a touché {touched} ligne(s) sur une base NOMINALE À JOUR. Une \
         entrée inconditionnelle rejouée à chaque import doit y être un no-op STRICT : sinon \
         elle réécrit une donnée que l'utilisateur a établie. ⚠️ Ce n'est PAS la propriété \
         « idempotent au second passage » — 20260628000001 satisfaisait la seconde et violait \
         celle-ci."
    );
    assert_eq!(
        line_accounts(&pool, validated).await,
        vec![Some(line_account), Some(line_account)],
        "la facture validée doit garder SON compte ({line_account}), et non recevoir celui de son \
         écriture ({}) — c'est l'écrasement d'une donnée établie que ce test interdit",
        seeded.accounts["3000"]
    );
    assert_eq!(
        line_accounts(&pool, draft).await,
        vec![None, None],
        "le brouillon doit rester à NULL : la liaison du compte est TARDIVE par conception, et un \
         rejeu qui le renseignerait aurait perdu la restriction aux pièces validées"
    );
}
