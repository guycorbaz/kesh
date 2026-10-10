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
//!
//! ⚠️ **Compté PAR ENTRÉE, depuis la Story 15-1a2-ii (#518).** Le rattrapage du
//! lettrage (`20261010000001`, M1) entre au registre en classe A, à côté de
//! l'entrée retirée `20260729000001`. Une somme sur toutes les entrées laisserait
//! l'une tourner à vide sans que rien ne rougisse : chaque entrée doit toucher
//! au moins une ligne sur la base d'avant, et aucune sur la base à jour. La
//! fixture porte donc aussi une facture client **soldée** et une facture
//! fournisseur **payée**, fabriquées en SQL brut — marques à `NULL` sur la base
//! d'avant, posées par un `UPDATE` brut et explicite sur la base à jour (le
//! groupe que la synchronisation aurait posé), **jamais** par M1 elle-même : le
//! test deviendrait tautologique sur l'accord rattrapage ↔ vivant, que tient
//! `lettering_documents_backfill.rs`.

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
    let lines: [(i64, &str, &str); 3] = [
        (seeded.accounts["1100"], TTC, ZERO), // créance
        (seeded.accounts["3000"], ZERO, HT),  // produit — le candidat attendu
        (seeded.accounts["2000"], ZERO, VAT), // TVA due
    ];
    insert_entry(pool, seeded, entry_number, &lines).await.0
}

/// Écriture quelconque en SQL brut ; rend `(écriture, lignes dans l'ordre)`.
async fn insert_entry(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    entry_number: i64,
    lines: &[(i64, &str, &str)],
) -> (i64, Vec<i64>) {
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

    let mut ids = Vec::new();
    for (order, (account_id, debit, credit)) in lines.iter().enumerate() {
        let id = sqlx::query(
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
        .expect("insert journal_entry_line")
        .last_insert_id() as i64;
        ids.push(id);
    }

    (entry_id, ids)
}

/// Les pièces SOLDÉES de la fixture (Story 15-1a2-ii), en SQL brut : un
/// règlement complet de la facture `invoice_id` (écriture de vente
/// `sale_entry`) et une facture fournisseur payée. Rend les deux groupes
/// `document` que la synchronisation poserait — `[vente, règlement]` sur la
/// créance `1100`, `[achat, paiement]` sur la dette `2000` — **sans** les poser.
async fn insert_settled_pieces(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    contact_id: i64,
    invoice_id: i64,
    sale_entry: i64,
) -> Vec<Vec<i64>> {
    let (creance, caisse, dette, charge) = (
        seeded.accounts["1100"],
        seeded.accounts["1000"],
        seeded.accounts["2000"],
        seeded.accounts["4000"],
    );
    let vente: i64 = sqlx::query_scalar(
        "SELECT id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(sale_entry)
    .bind(creance)
    .fetch_one(pool)
    .await
    .expect("ligne de vente sur la créance");
    let (reglement, l_reglement) = insert_entry(
        pool,
        seeded,
        2,
        &[(caisse, TTC, ZERO), (creance, ZERO, TTC)],
    )
    .await;
    sqlx::query(
        "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, amount, \
         settled_on, settlement_type, settlement_account_id) \
         VALUES (?, ?, ?, ?, '2026-03-01', 'internal_account', ?)",
    )
    .bind(seeded.company_id)
    .bind(invoice_id)
    .bind(reglement)
    .bind(TTC)
    .bind(caisse)
    .execute(pool)
    .await
    .expect("insert invoice_settlements");

    const ACHAT: &str = "500.0000";
    let (achat, l_achat) = insert_entry(
        pool,
        seeded,
        3,
        &[(charge, ACHAT, ZERO), (dette, ZERO, ACHAT)],
    )
    .await;
    let (paiement, l_paiement) = insert_entry(
        pool,
        seeded,
        4,
        &[(dette, ACHAT, ZERO), (caisse, ZERO, ACHAT)],
    )
    .await;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, total_amount, purchase_journal_entry_id, settlement_type, \
         settlement_account_id, settlement_journal_entry_id, paid_at) \
         VALUES (?, ?, 'FF-16-1c', 'paid', '2026-03-01', ?, ?, 'internal_account', ?, ?, \
                 '2026-03-01 00:00:00')",
    )
    .bind(seeded.company_id)
    .bind(contact_id)
    .bind(ACHAT)
    .bind(achat)
    .bind(caisse)
    .bind(paiement)
    .execute(pool)
    .await
    .expect("insert supplier_invoices");

    vec![vec![vente, l_reglement[1]], vec![l_achat[1], l_paiement[0]]]
}

/// Les marques des lignes données.
async fn marks(pool: &MySqlPool, lines: &[i64]) -> Vec<(Option<i64>, Option<String>)> {
    let mut out = Vec::new();
    for id in lines {
        out.push(
            sqlx::query_as(
                "SELECT lettering_key, lettering_origin FROM journal_entry_lines WHERE id = ?",
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .expect("marque"),
        );
    }
    out
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

/// Rejoue les entrées de classe A dans une transaction **committée**, et rend les
/// lignes touchées **par entrée** : `(label, rows_affected)`.
///
/// Le manifeste passé est vide : il n'entre pas dans la décision d'une entrée
/// inconditionnelle, et c'est précisément la propriété testée.
async fn replay_class_a(
    pool: &MySqlPool,
    entries: &[PostRestoreBackfill],
) -> Vec<(&'static str, u64)> {
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
    report.iter().map(|r| (r.label, r.rows_affected)).collect()
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
    // L'état d'avant la 15-1a2-ii : pièces soldées, marques à `NULL`.
    let groupes = insert_settled_pieces(&pool, &seeded, contact_id, invoice_id, entry).await;
    let toutes: Vec<i64> = groupes.iter().flatten().copied().collect();
    assert!(
        marks(&pool, &toutes).await.iter().all(|m| m.0.is_none()),
        "pré-condition : les pièces soldées ne sont pas lettrées"
    );

    let touched = replay_class_a(&pool, &entries).await;

    for (label, rows) in &touched {
        assert!(
            *rows > 0,
            "l'entrée de classe A {label} n'a touché AUCUNE ligne alors que la base est dans \
             l'état que son SQL vise. Le registre embarque du SQL qui n'agit jamais — et le test \
             de no-op qui l'accompagne est alors vrai à vide."
        );
    }
    for groupe in &groupes {
        let k = groupe.iter().min().copied();
        assert_eq!(
            marks(&pool, groupe).await,
            vec![(k, Some("document".to_string())); groupe.len()],
            "M1 pose le groupe `document` de la pièce soldée"
        );
    }
    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "le rejeu doit poser, sur TOUTES les lignes, le compte que l'écriture a réellement crédité"
    );
}

/// **AC-C6.3 — no-op sur une base nominale à jour.** Le rejeu inconditionnel ne
/// doit toucher **aucune** ligne d'une installation en usage courant.
///
/// # Le montage `3200` ≠ `3000` : valide, mais plus le seul discriminant
///
/// La facture validée porte sur ses lignes un compte **différent** de celui que
/// son écriture crédite : lignes sur `3200`, écriture sur `3000`. C'est un état
/// nominal — la divergence entre le compte d'une ligne et celui de l'écriture est
/// documentée et légitime (défaut société changé depuis, ou compte corrigé à la
/// main) — et il distingue un rejeu qui **écraserait** une valeur posée : privé
/// de sa garde `revenue_account_id IS NULL`, il écrirait `3000` sur `3200`.
///
/// Ce n'est plus le seul discriminant. sqlx pose `CLIENT_FOUND_ROWS`
/// (`sqlx-mysql`, `connection/stream.rs` ; `letterings.rs` s'y appuie) :
/// `rows_affected` compte les lignes **trouvées**, non les lignes modifiées. Un
/// rejeu privé de sa garde qui réécrirait une valeur à l'identique compterait
/// donc aussi la ligne — c'est ce qui rend discriminant le volet du lettrage
/// (Story 15-1a2-ii) : les groupes `document` posés à l'identique de ce que M1
/// écrirait ferait compter à M1 privée de sa garde `lettering_key IS NULL` les
/// lignes qu'elle réécrit.
///
/// « Nominale » s'entend **sans** les deux états où M1 peut légitimement poser
/// une marque sur une base à jour (pièce historique dont l'exercice a été rouvert,
/// compte devenu lettrable — cf. sa justification au registre).
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

    // Les pièces soldées, et LEURS marques posées par un `UPDATE` brut : le
    // groupe que la synchronisation aurait posé (clé = plus petite ligne,
    // origine `document`) — jamais par M1 elle-même.
    let groupes = insert_settled_pieces(&pool, &seeded, contact_id, validated, entry).await;
    for groupe in &groupes {
        let k = *groupe.iter().min().expect("groupe");
        let posees = sqlx::query(
            "UPDATE journal_entry_lines SET lettering_key = ?, lettering_origin = 'document' \
             WHERE id IN (?, ?)",
        )
        .bind(k)
        .bind(groupe[0])
        .bind(groupe[1])
        .execute(&pool)
        .await
        .expect("marques posées")
        .rows_affected();
        assert_eq!(posees, 2, "montage : les deux lignes du groupe trouvées");
    }
    let toutes: Vec<i64> = groupes.iter().flatten().copied().collect();
    let marques_avant = marks(&pool, &toutes).await;

    let touched = replay_class_a(&pool, &entries).await;

    for (label, rows) in &touched {
        assert_eq!(
            *rows, 0,
            "l'entrée de classe A {label} a touché {rows} ligne(s) sur une base NOMINALE À JOUR \
             (sans pièce rendue lettrable depuis). Une entrée inconditionnelle rejouée à chaque \
             import n'y délettre ni n'y réécrit rien : sinon elle réécrit une donnée établie. \
             ⚠️ `rows_affected` compte les lignes TROUVÉES (`CLIENT_FOUND_ROWS`) : une réécriture \
             à l'identique compte aussi. Ce n'est PAS la propriété « idempotent au second \
             passage » — 20260628000001 satisfaisait la seconde et violait celle-ci."
        );
    }
    assert_eq!(
        marks(&pool, &toutes).await,
        marques_avant,
        "les marques des pièces lettrées restent inchangées"
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
