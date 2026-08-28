//! Story 16-1a-bis — backfill du compte de produit sur le parc de factures et
//! d'avoirs déjà validés (migration `20260729000001`).
//!
//! # Pourquoi ces tests ne peuvent PAS être écrits sous `#[sqlx::test]` ordinaire
//!
//! Le mode par défaut crée la base éphémère vide, applique **toutes** les
//! migrations, **puis** laisse le test insérer ses données : le backfill
//! tournerait sur des tables vides et ne prouverait **rien**.
//! `migrations_fresh_install` est dans le même cas — il n'y a rien à
//! backfiller sur une base vierge.
//!
//! Tous les tests d'ici utilisent donc le montage en trois temps de
//! `tests/common/mod.rs` : migrations **jusqu'à** celle du backfill, insertion
//! des données en SQL brut, puis `MIGRATOR.run()`.
//!
//! # Ce que chaque test verrouille
//!
//! | Test | Décision | Ce qui casse sans lui |
//! |---|---|---|
//! | [`backfills_from_journal_entry_not_from_current_default`] | D-B1 | un backfill lisant `settings` écrirait un compte que la facture n'a jamais crédité |
//! | [`leaves_null_when_no_line_matches_total`] | D-B2 / AC-B2 | un backfill « par élimination » écrirait un compte faux sur une écriture éditée |
//! | [`leaves_null_when_several_lines_match_total`] | D-B2 | idem, cas à plusieurs candidats |
//! | [`backfills_when_vat_config_is_null`] | **D-B3** | **le backfill no-ope, côté FACTURE, sur toute installation exonérée de TVA — indiscernable du succès** |
//! | [`backfills_credit_note_when_vat_config_is_null`] | **D-B3 miroir** | **idem côté AVOIR : muter le seul `<=>` du 2ᵉ `UPDATE` laissait toute la suite verte** |
//! | [`skips_draft_and_cancelled_invoices`] | D-B5 | la liaison tardive des brouillons serait figée à tort |
//! | [`leaves_null_when_credited_account_is_not_postable`] | **cond. (4)** | un compte collectif atterrirait dans `invoice_lines`, donnée que la saisie elle-même refuse |
//! | [`backfills_credit_note_from_its_own_entry`] | D-B1 miroir | l'avoir hériterait du compte de la facture au lieu du sien |
//! | [`divergent_invoice_and_credit_note_are_recorded_as_is`] | **D-B7** | on « réparerait » un résidu historique en falsifiant les pièces |
//! | [`backfill_is_idempotent`] | D-B6 / AC-B4 | un re-jeu écraserait un compte déjà posé — la ligne `F-PRE`, dont la valeur stockée DIFFÈRE du candidat, est la seule qui exerce la garde |
//! | [`validated_invoice_from_the_real_engine_is_recovered_by_the_backfill`] | **D-B2 cond. (3)** | le critère no-operait sur 100 % du parc si `total_amount` cessait d'égaler le crédit posé par le moteur |
//! | [`backfills_when_company_has_no_invoice_settings_row`] | **`LEFT JOIN` cis** | un `INNER JOIN` écarterait SANS TRACE les factures d'une société sans ligne de configuration |
//! | [`backfills_credit_note_when_company_has_no_invoice_settings_row`] | **`LEFT JOIN` cis, miroir** | idem côté AVOIR : muter le seul `LEFT` du 2ᵉ `UPDATE` laissait les 12 autres tests verts |
//!
//! # Montage : chaque test doit prouver que le backfill a TOURNÉ
//!
//! Les tests qui attendent un `NULL` (`leaves_null_*`, `skips_*`) portent tous
//! une **facture témoin** dont la ligne, elle, DOIT être backfillée. Sans ce
//! témoin, ils passeraient à vide si le montage se décalait — exactement la
//! régression subie par `accounts_role_backfill.rs` en Story 16-1a, où un test
//! est devenu muet en gardant ses assertions vertes pour la mauvaise raison.

use sqlx::MySqlPool;

use kesh_db::test_fixtures::seed_accounting_company;

mod common;

use common::{apply_migrations_up_to, migrations_before};

/// Version de la migration sous test.
const BACKFILL_MIGRATION: i64 = 20260729000001;

/// Nombre de migrations à appliquer pour se placer **juste avant** le backfill.
/// Résolu par version (garde-fou P6) — surtout pas `total - 1`.
fn migrations_before_backfill() -> usize {
    migrations_before(BACKFILL_MIGRATION, "invoice_lines_revenue_account_backfill")
}

// Montants de référence : HT 1000.00, TVA 8.1 % = 81.00, TTC 1081.00.
const HT: &str = "1000.0000";
const VAT: &str = "81.0000";
const TTC: &str = "1081.0000";
const ZERO: &str = "0.0000";

/// Une ligne d'écriture à insérer : `(account_id, debit, credit)`.
type RawLine = (i64, &'static str, &'static str);

/// Insère une écriture comptable **en SQL brut**, lignes comprises.
async fn insert_entry(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    entry_number: i64,
    lines: &[RawLine],
) -> i64 {
    let entry_id = sqlx::query(
        "INSERT INTO journal_entries \
         (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, '2026-03-01', 'Ventes', 'Test 16-1a-bis')",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .bind(entry_number)
    .execute(pool)
    .await
    .expect("insert journal_entry")
    .last_insert_id() as i64;

    for (idx, (account_id, debit, credit)) in lines.iter().enumerate() {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(account_id)
        .bind(idx as i32 + 1)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .expect("insert journal_entry_line");
    }

    entry_id
}

/// Insère une facture **avec deux lignes** dont `revenue_account_id` est `NULL`
/// (l'état exact du parc antérieur au déploiement de la 16-1a).
///
/// Deux lignes et non une : le backfill pose le compte de l'écriture sur
/// **toutes** les lignes de la facture, et une facture mono-ligne ne le
/// prouverait pas.
async fn insert_invoice_with_lines(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    number: &str,
    status: &str,
    journal_entry_id: Option<i64>,
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
             (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
             VALUES (?, ?, 'Prestation', 1, ?, 8.10, ?)",
        )
        .bind(invoice_id)
        .bind(position)
        .bind(line_total)
        .bind(line_total)
        .execute(pool)
        .await
        .expect("insert invoice_line");
    }

    invoice_id
}

/// Insère un avoir `issued` avec deux lignes `revenue_account_id` `NULL`.
async fn insert_credit_note_with_lines(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    invoice_id: i64,
    number: &str,
    journal_entry_id: i64,
) -> i64 {
    let credit_note_id = sqlx::query(
        "INSERT INTO credit_notes \
         (company_id, contact_id, invoice_id, credit_note_number, status, date, total_amount, \
          journal_entry_id) \
         VALUES (?, ?, ?, ?, 'issued', '2026-04-01', ?, ?)",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(invoice_id)
    .bind(number)
    .bind(HT)
    .bind(journal_entry_id)
    .execute(pool)
    .await
    .expect("insert credit_note")
    .last_insert_id() as i64;

    for (position, line_total) in [(1, "600.0000"), (2, "400.0000")] {
        sqlx::query(
            "INSERT INTO credit_note_lines \
             (credit_note_id, position, description, quantity, unit_price, vat_rate, line_total) \
             VALUES (?, ?, 'Prestation', 1, ?, 8.10, ?)",
        )
        .bind(credit_note_id)
        .bind(position)
        .bind(line_total)
        .bind(line_total)
        .execute(pool)
        .await
        .expect("insert credit_note_line");
    }

    credit_note_id
}

/// Crée un compte de produit **non imputable** (collectif). Atteignable dans un
/// parc réel : la validation d'écriture tourne avec `enforce_postable = false`,
/// donc une écriture ancienne a pu créditer un compte de regroupement.
async fn insert_non_postable_account(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, postable) \
         VALUES (?, ?, 'Groupe produits', 'Revenue', FALSE)",
    )
    .bind(company_id)
    .bind(number)
    .execute(pool)
    .await
    .expect("insert compte non imputable");

    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
        .bind(company_id)
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("select account id")
}

/// Crée un second compte de produit (le plan de la fixture n'en a qu'un).
async fn insert_revenue_account(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Ventes secondaires', 'Revenue')",
    )
    .bind(company_id)
    .bind(number)
    .execute(pool)
    .await
    .expect("insert account")
    .last_insert_id() as i64;

    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
        .bind(company_id)
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("select account id")
}

/// Comptes posés sur les lignes d'une facture, triés par position.
async fn invoice_line_accounts(pool: &MySqlPool, invoice_id: i64) -> Vec<Option<i64>> {
    sqlx::query_scalar(
        "SELECT revenue_account_id FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .expect("select invoice_lines.revenue_account_id")
}

/// Comptes posés sur les lignes d'un avoir, triés par position.
async fn credit_note_line_accounts(pool: &MySqlPool, credit_note_id: i64) -> Vec<Option<i64>> {
    sqlx::query_scalar(
        "SELECT revenue_account_id FROM credit_note_lines WHERE credit_note_id = ? \
         ORDER BY position",
    )
    .bind(credit_note_id)
    .fetch_all(pool)
    .await
    .expect("select credit_note_lines.revenue_account_id")
}

/// La requête de diagnostic du CHANGELOG (D-B4) — celle que l'exploitant
/// exécutera après la mise à jour. Aucune table, aucun log : c'est **elle**
/// qui matérialise le décompte du reliquat, et c'est donc elle que les tests
/// doivent asserter, mot pour mot.
async fn remaining_null_invoice_lines(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM invoice_lines il JOIN invoices i ON i.id = il.invoice_id \
         WHERE i.status = 'validated' AND il.revenue_account_id IS NULL",
    )
    .fetch_one(pool)
    .await
    .expect("requête de diagnostic D-B4 (factures)")
}

/// Pendant avoir de la requête de diagnostic (D-B4), publiée elle aussi au
/// CHANGELOG. Elle est exécutée ici pour la même raison que sa jumelle : une
/// requête livrée à l'exploitant et jamais jouée par aucun test est une
/// affirmation non vérifiée — une coquille de nom de table ou de colonne y
/// passerait jusqu'en production.
async fn remaining_null_credit_note_lines(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM credit_note_lines cnl \
         JOIN credit_notes cn ON cn.id = cnl.credit_note_id \
         WHERE cn.status = 'issued' AND cnl.revenue_account_id IS NULL",
    )
    .fetch_one(pool)
    .await
    .expect("requête de diagnostic D-B4 (avoirs)")
}

/// Insère un contact client (FK obligatoire des factures).
async fn insert_contact(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', 'Client 16-1a-bis', TRUE, FALSE)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("insert contact")
    .last_insert_id() as i64
}

/// **Le test de D-B1.** La ligne doit recevoir le compte que l'écriture a
/// réellement crédité (3000), et **pas** le défaut société courant (3200).
///
/// Le changement de défaut **après** la validation n'est pas décoratif : c'est
/// lui qui rend le test discriminant. Sans lui, un backfill lisant
/// `settings.default_revenue_account_id` passerait aussi et ne prouverait rien.
#[sqlx::test(migrations = false)]
async fn backfills_from_journal_entry_not_from_current_default(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    // Écriture canonique : débit créance TTC, crédit produit HT, crédit TVA.
    let entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-001",
        "validated",
        Some(entry),
    )
    .await;

    // T1+ : l'administrateur change le défaut. Le backfill ne doit PAS le voir.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_revenue_account_id = ? WHERE company_id = ?",
    )
    .bind(other_revenue)
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .expect("changement du défaut société");

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "le backfill doit poser le compte CRÉDITÉ PAR L'ÉCRITURE ({revenue}) sur toutes les \
         lignes, jamais le défaut société courant ({other_revenue}) — D-B1"
    );
}

/// **Le test d'AC-B2.** Écriture retouchée : le produit y est ventilé sur deux
/// comptes, donc **aucune** ligne ne crédite exactement `total_amount`. Zéro
/// candidat → la ligne reste `NULL` et la migration **réussit**.
#[sqlx::test(migrations = false)]
async fn leaves_null_when_no_line_matches_total(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    // Écriture ÉDITÉE : 600 + 400 au lieu d'un crédit unique de 1000.
    let edited = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, "600.0000"),
            (other_revenue, ZERO, "400.0000"),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let edited_invoice = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-EDIT",
        "validated",
        Some(edited),
    )
    .await;

    // Facture témoin à écriture canonique : prouve que le backfill a TOURNÉ.
    let canonical = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let witness = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-OK",
        "validated",
        Some(canonical),
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, edited_invoice).await,
        vec![None, None],
        "écriture éditée sans ligne créditant exactement total_amount : la ligne DOIT rester \
         NULL — écrire un compte ici fabriquerait la corruption que la story ferme (AC-B2)"
    );
    assert_eq!(
        invoice_line_accounts(&pool, witness).await,
        vec![Some(revenue), Some(revenue)],
        "assertion de montage : la facture témoin prouve que le backfill a bien tourné — sans \
         elle ce test passerait à vide si la fenêtre de migrations se décalait"
    );
    assert_eq!(
        remaining_null_invoice_lines(&pool).await,
        2,
        "la requête de diagnostic de D-B4 doit dénombrer exactement les 2 lignes non reprises"
    );
}

/// Second volet de D-B2 : **plusieurs** candidats. Une écriture retouchée peut
/// laisser deux lignes créditant chacune `total_amount` — le critère exige
/// l'unicité, donc la ligne reste `NULL`.
#[sqlx::test(migrations = false)]
async fn leaves_null_when_several_lines_match_total(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    // Deux crédits valant chacun exactement total_amount → ambiguïté.
    let ambiguous = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, "2081.0000", ZERO),
            (revenue, ZERO, HT),
            (other_revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-AMBI",
        "validated",
        Some(ambiguous),
    )
    .await;

    let canonical = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let witness = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-OK",
        "validated",
        Some(canonical),
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![None, None],
        "deux candidats : le critère exige l'UNICITÉ, la ligne doit rester NULL (D-B2)"
    );
    assert_eq!(
        invoice_line_accounts(&pool, witness).await,
        vec![Some(revenue), Some(revenue)],
        "assertion de montage : le backfill a bien tourné"
    );
}

/// **LE test de D-B3— le plus important du fichier.**
///
/// `default_vat_payable_account_id` est `NULL` par **défaut** (ni l'onboarding,
/// ni le lazy-create, ni aucune migration ne la renseigne). Écrite `<>` au lieu
/// de `<=>`, la condition (2) propage `NULL` en logique ternaire, aucune ligne
/// n'est candidate et le backfill **no-ope intégralement sur toute installation
/// concernée**.
///
/// Portée exacte, à ne pas surestimer : une société dont cette colonne est
/// `NULL` n'a pu valider **que** des factures sans TVA — `validate_invoice`
/// passe la colonne à `generate_invoice_journal_lines` (`invoices.rs:1794`),
/// qui échoue en `ConfigurationRequired` dès que `total_vat > 0`
/// (`invoices.rs:1497-1500`). La population n'est donc pas « 100 % du parc »
/// mais les installations **exonérées de TVA**, cas parfaitement réel en Suisse
/// sous le seuil de CHF 100'000 et cœur de cible de Kesh.
/// *(Portée corrigée en passe 2 de `bmad-code-review`.)*
///
/// Ce mode de défaillance est **indiscernable du succès** sans ce test : la
/// migration réussit, et la story pré-autorise explicitement un reliquat élevé
/// comme comportement conservateur normal (AC-B2). C'est exactement ce que ce
/// test existe pour attraper.
#[sqlx::test(migrations = false)]
async fn backfills_when_vat_config_is_null(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];

    // L'état par défaut d'une installation : colonnes TVA jamais renseignées.
    // La fixture les pose, on les remet à NULL pour retrouver le cas réel.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_vat_payable_account_id = NULL, \
         default_vat_recoverable_account_id = NULL WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .expect("remise à NULL des colonnes TVA de config");

    let entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-VATNULL",
        "validated",
        Some(entry),
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "société sans compte de TVA configuré (le cas par DÉFAUT) : la ligne DOIT être \
         backfillée. Si elle est NULL, la condition (2) a été écrite avec `<>` au lieu de \
         `<=>` et le backfill no-ope intégralement sur toute installation exonérée — D-B3"
    );
    assert_eq!(
        remaining_null_invoice_lines(&pool).await,
        0,
        "aucun reliquat attendu sur ce montage"
    );
}

/// D-B5 : les `draft` gardent leur liaison tardive, les `cancelled` sont
/// exclues délibérément (leur facture a déjà été créditée, et
/// `uq_credit_notes_invoice` interdit un second avoir : aucun résidu futur
/// n'est possible, il n'y a rien à prévenir).
#[sqlx::test(migrations = false)]
async fn skips_draft_and_cancelled_invoices(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];

    let canonical_lines: [RawLine; 3] = [
        (receivable, TTC, ZERO),
        (revenue, ZERO, HT),
        (vat, ZERO, VAT),
    ];

    // Brouillon : aucune écriture (une facture draft n'en a pas).
    let draft = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-DRAFT",
        "draft",
        None,
    )
    .await;

    // Annulée : écriture canonique présente, mais statut hors périmètre.
    let cancelled_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &canonical_lines,
    )
    .await;
    let cancelled = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-CANCEL",
        "cancelled",
        Some(cancelled_entry),
    )
    .await;

    // Témoin validé : prouve que le backfill a tourné.
    let witness_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &canonical_lines,
    )
    .await;
    let witness = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-OK",
        "validated",
        Some(witness_entry),
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, draft).await,
        vec![None, None],
        "une facture brouillon garde `NULL` : c'est le sens même de la liaison tardive (16-1a D2)"
    );
    assert_eq!(
        invoice_line_accounts(&pool, cancelled).await,
        vec![None, None],
        "une facture annulée est hors périmètre (D-B5), bien qu'elle ait une écriture canonique"
    );
    assert_eq!(
        invoice_line_accounts(&pool, witness).await,
        vec![Some(revenue), Some(revenue)],
        "assertion de montage : le backfill a bien tourné sur la facture validée"
    );
}

/// Miroir avoir : le compte est celui que l'écriture de **contre-passation** a
/// réellement **débité**, déterminé indépendamment de la facture d'origine.
#[sqlx::test(migrations = false)]
async fn backfills_credit_note_from_its_own_entry(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];

    let invoice_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    // L'émission d'un avoir bascule TOUJOURS la facture en `cancelled`
    // (`credit_notes.rs`, DC6) : c'est l'état réel du parc.
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-002",
        "cancelled",
        Some(invoice_entry),
    )
    .await;

    // Contre-passation : swap débit/crédit, montants positifs.
    let cn_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (revenue, HT, ZERO),
            (vat, VAT, ZERO),
            (receivable, ZERO, TTC),
        ],
    )
    .await;
    let credit_note_id = insert_credit_note_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        invoice_id,
        "A-001",
        cn_entry,
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        credit_note_line_accounts(&pool, credit_note_id).await,
        vec![Some(revenue), Some(revenue)],
        "l'avoir doit recevoir le compte que SON écriture a débité (miroir de D-B1)"
    );
    assert_eq!(
        remaining_null_credit_note_lines(&pool).await,
        0,
        "la requête de diagnostic D-B4 côté avoirs — publiée au CHANGELOG — doit être \
         exécutable et ne dénombrer aucun reliquat sur ce montage"
    );
}

/// **Le test de D-B7 — le cas qui motive l'existence de la story.**
///
/// Facture validée à T1 sur défaut 3000 ; l'administrateur passe le défaut à
/// 3200 ; avoir émis à T2 par l'ancien code, qui relisait la configuration à
/// l'émission — son écriture débite donc 3200. Le backfill **enregistre
/// fidèlement** ce que chaque pièce a mouvementé : l'avoir reçoit 3200, et
/// **surtout pas** 3000 « harmonisé » depuis la facture. Le résidu existe déjà
/// dans les écritures, il est passé et irréversible ; le maquiller falsifierait
/// les pièces sans corriger la moindre écriture.
///
/// # Pourquoi la facture reste `NULL`, et pourquoi c'est correct
///
/// L'émission d'un avoir bascule **toujours** la facture d'origine en
/// `cancelled` (`credit_notes.rs`, DC6, `UPDATE … WHERE … AND status =
/// 'validated'`, et un `rows == 0` fait échouer la transaction). Or D-B5 exclut
/// délibérément les `cancelled` et énonce lui-même la conséquence : « sur une
/// facture créditée, `credit_note_lines.revenue_account_id` sera renseigné
/// alors que `invoice_lines.revenue_account_id` restera `NULL` ».
///
/// Ce qui **diverge** — la substance de D-B7 — n'est donc pas « le compte de la
/// facture vs celui de l'avoir » mais **le compte que l'écriture de la facture
/// a réellement crédité (3000, lisible dans `journal_entry_lines`) vs celui que
/// l'avoir a débité (3200)**. Les deux pièces sont enregistrées telles quelles,
/// aucune harmonisation n'est tentée.
///
/// *(La rédaction d'AC-B3 cas 6 annonçait « le backfill écrit **3000 sur la
/// facture** ». Cette moitié était **inatteignable** ; l'AC a été corrigée en
/// passe 1 de `bmad-code-review`, arbitrage Guy du 2026-07-29. Il n'y a donc
/// plus de contradiction spec ↔ code à consigner ici : ce test décrit
/// désormais exactement ce que l'AC énonce.)*
#[sqlx::test(migrations = false)]
async fn divergent_invoice_and_credit_note_are_recorded_as_is(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    // T1 — la facture a crédité 3000.
    let invoice_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-003",
        "cancelled",
        Some(invoice_entry),
    )
    .await;

    // T1+ — le défaut société passe à 3200.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_revenue_account_id = ? WHERE company_id = ?",
    )
    .bind(other_revenue)
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .expect("changement du défaut société");

    // T2 — l'avoir, généré par l'ancien code, a débité 3200. C'EST LE BUG.
    let cn_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (other_revenue, HT, ZERO),
            (vat, VAT, ZERO),
            (receivable, ZERO, TTC),
        ],
    )
    .await;
    let credit_note_id = insert_credit_note_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        invoice_id,
        "A-002",
        cn_entry,
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    let cn_accounts = credit_note_line_accounts(&pool, credit_note_id).await;
    assert_eq!(
        cn_accounts,
        vec![Some(other_revenue), Some(other_revenue)],
        "l'avoir doit enregistrer le compte que SON écriture a débité ({other_revenue}), et \
         surtout pas celui de la facture ({revenue}) : le backfill ENREGISTRE le résidu \
         historique, il ne le RÉPARE pas (D-B7)"
    );
    assert_ne!(
        cn_accounts[0],
        Some(revenue),
        "toute tentative d'harmoniser l'avoir sur la facture falsifierait les pièces sans \
         corriger la moindre écriture (D-B7)"
    );
    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![None, None],
        "la facture créditée est `cancelled`, donc hors périmètre — conséquence explicitement \
         assumée par D-B5"
    );

    // AC-B3 cas 6 affirme que le compte historique reste « lisible dans
    // `journal_entry_lines`, jamais recopié dans `invoice_lines` ». La fixture
    // le CONSTRUIT ; sans cette assertion elle ne l'OBSERVE pas, et la moitié
    // de l'AC ne serait verrouillée par rien.
    // *(Ajouté en passe 2 de `bmad-code-review`, finding Acceptance Auditor.)*
    let historical: Vec<i64> = sqlx::query_scalar(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.id = ? AND jel.credit > 0 AND jel.credit = i.total_amount",
    )
    .bind(invoice_id)
    .fetch_all(&pool)
    .await
    .expect("lecture du compte historiquement crédité");
    assert_eq!(
        historical,
        vec![revenue],
        "le compte que la facture a RÉELLEMENT crédité ({revenue}) doit rester lisible dans \
         `journal_entry_lines` — c'est la seconde moitié d'AC-B3 cas 6, et la seule trace du \
         compte d'origine puisque `invoice_lines` reste `NULL`"
    );
}

/// AC-B4 : rejouer le backfill ne change **aucune** ligne — ni celles qu'il a
/// posées (garde `IS NULL`), ni celles qu'il a laissées (critère déterministe).
///
/// # Le montage porte un avoir, et ce n'est pas décoratif
///
/// La migration contient **deux** `UPDATE`, chacun avec sa propre garde
/// `IS NULL` : `invoice_lines` et `credit_note_lines`. Un test d'idempotence
/// qui n'insère aucun avoir rejoue bien le second `UPDATE`, mais **sur un
/// ensemble vide** — ce qui est vrai de n'importe quel SQL et ne démontre
/// rien. Si la garde `WHERE cnl.revenue_account_id IS NULL` du volet avoir
/// disparaissait dans un refactor, un tel test resterait vert.
///
/// *(Volet avoir ajouté en passe 1 de `bmad-code-review` — convergence Blind
/// Hunter + Edge Case Hunter. `docs/migrations-idempotence-audit.md` affirme
/// l'idempotence de la migration sans distinguer les deux `UPDATE` : le test
/// doit couvrir ce que la doc affirme.)*
#[sqlx::test(migrations = false)]
async fn backfill_is_idempotent(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    let canonical = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let backfilled = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-OK",
        "validated",
        Some(canonical),
    )
    .await;

    // Une facture que le backfill laisse `NULL` : le re-jeu ne doit pas non
    // plus « finir le travail » en relâchant le critère.
    let edited = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, "600.0000"),
            (other_revenue, ZERO, "400.0000"),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let untouched = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-EDIT",
        "validated",
        Some(edited),
    )
    .await;

    // Volet avoir — sans lui le second `UPDATE` se rejoue sur zéro ligne.
    // La facture porteuse est `cancelled` : c'est l'état réel de toute facture
    // créditée, et le backfill l'exclut (D-B5), ce qui rend l'avoir seul
    // responsable de ce qu'on observe côté `credit_note_lines`.
    let credited_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        3,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let credited_invoice = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-CRED",
        "cancelled",
        Some(credited_entry),
    )
    .await;
    // Contre-passation canonique : swap débit/crédit, montants positifs.
    let cn_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        4,
        &[
            (revenue, HT, ZERO),
            (vat, VAT, ZERO),
            (receivable, ZERO, TTC),
        ],
    )
    .await;
    let credit_note = insert_credit_note_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        credited_invoice,
        "A-IDEM",
        cn_entry,
    )
    .await;

    // LE MONTAGE QUI REND CE TEST DISCRIMINANT — sans lui, la garde `IS NULL`
    // n'est exercée nulle part.
    //
    // Les trois pièces ci-dessus portent toutes une valeur qui est le POINT
    // FIXE du critère : ce que le backfill recalculerait est exactement ce qui
    // est déjà stocké. Retirer les deux gardes `IS NULL` les réécrirait donc à
    // l'identique, et un test qui ne compare qu'« avant re-jeu » à « après
    // re-jeu » resterait VERT. C'est le défaut qu'ont convergé les trois
    // lentilles de la passe 2.
    //
    // Il faut une ligne dont la valeur stockée DIFFÈRE du candidat : facture
    // dont l'écriture a été éditée après validation (`PUT /journal-entries` n'a
    // aucune garde de provenance) pour créditer un AUTRE compte du même
    // montant. La ligne porte `other_revenue`, l'écriture crédite `revenue`.
    //   - avec la garde   -> la ligne reste `other_revenue` ;
    //   - sans la garde   -> elle devient `revenue`, DÈS LE PREMIER PASSAGE.
    // D'où l'assertion sur la valeur ABSOLUE plus bas, et pas seulement sur la
    // stabilité : c'est elle qui tombe si la garde disparaît.
    let diverging_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        5,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let prematerialized = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-PRE",
        "validated",
        Some(diverging_entry),
    )
    .await;
    sqlx::query("UPDATE invoice_lines SET revenue_account_id = ? WHERE invoice_id = ?")
        .bind(other_revenue)
        .bind(prematerialized)
        .execute(&pool)
        .await
        .expect("pré-matérialisation divergente");

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, prematerialized).await,
        vec![Some(other_revenue), Some(other_revenue)],
        "LA GARDE `IS NULL` : une ligne DÉJÀ renseignée ne doit jamais être réécrite, même \
         quand le critère désignerait un autre compte ({revenue}) que celui posé \
         ({other_revenue}). Sans la garde, cette assertion tombe dès le premier passage — \
         c'est le seul montage de ce test qui l'exerce."
    );

    let after_first = (
        invoice_line_accounts(&pool, backfilled).await,
        invoice_line_accounts(&pool, untouched).await,
        credit_note_line_accounts(&pool, credit_note).await,
        invoice_line_accounts(&pool, prematerialized).await,
    );
    assert_eq!(
        after_first.0,
        vec![Some(revenue), Some(revenue)],
        "pré-condition du test d'idempotence : le premier passage doit avoir backfillé"
    );
    assert_eq!(
        after_first.1,
        vec![None, None],
        "pré-condition : la facture à écriture ventilée doit rester `NULL` — sinon le rôle que \
         lui assigne ce test (le re-jeu ne doit pas « finir le travail ») n'est pas tenu"
    );
    assert_eq!(
        after_first.2,
        vec![Some(revenue), Some(revenue)],
        "pré-condition du VOLET AVOIR : sans lui, le second `UPDATE` se rejouerait sur un \
         ensemble vide et l'idempotence de `credit_note_lines` ne serait pas mesurée"
    );

    // Re-jeu du SQL RÉELLEMENT EMBARQUÉ dans le binaire — pas d'une copie
    // paraphrasée, qui ne prouverait rien sur la migration livrée.
    let sql = kesh_db::MIGRATOR
        .migrations
        .iter()
        .find(|m| m.version == BACKFILL_MIGRATION)
        .expect("migration de backfill introuvable dans le MIGRATOR")
        .sql
        .clone();
    sqlx::raw_sql(&sql)
        .execute(&pool)
        .await
        .expect("re-jeu du backfill");

    assert_eq!(
        (
            invoice_line_accounts(&pool, backfilled).await,
            invoice_line_accounts(&pool, untouched).await,
            credit_note_line_accounts(&pool, credit_note).await,
            invoice_line_accounts(&pool, prematerialized).await,
        ),
        after_first,
        "le backfill est intrinsèquement idempotent (garde `IS NULL` + critère déterministe) : \
         un re-jeu doit être un no-op strict sur les DEUX `UPDATE` — AC-B4 / D-B6"
    );
}

/// **Le test de jonction — le seul qui relie le moteur réel au backfill.**
///
/// Toute la puissance discriminante du critère de D-B2 tient à sa condition
/// (3) : `jel.credit = invoices.total_amount`. Le commentaire de la migration
/// affirme que les deux valeurs sont égales **par construction**, parce que
/// `generate_invoice_journal_lines` pousse `credit = Σ line_total` et que
/// `compute_total` calcule `total_amount` avec la même fonction.
///
/// Or les huit autres tests de ce fichier montent leurs écritures en **SQL
/// brut**, avec le *même littéral* des deux côtés : ils **supposent** l'égalité
/// au lieu de l'**observer**. Si le moteur cessait un jour de la respecter — un
/// centime d'écart sur une ventilation TVA multi-taux, un arrondi introduit sur
/// le HT, une remise appliquée après `compute_total` — la condition (3) ne
/// matcherait plus **rien** : le backfill no-operait sur 100 % du parc,
/// silencieusement, et **aucun** de ces huit tests ne bougerait. C'est le même
/// mode de défaillance que celui de D-B3, indiscernable du succès puisque la
/// spec pré-autorise un reliquat élevé.
///
/// Le recours au SQL brut ailleurs est **légitime** : le code actuel ne sait
/// plus produire l'état pré-16-1a (`validate_invoice` matérialise le compte).
/// Ce qui manquait est le test de jonction, pas un abandon du SQL brut.
///
/// Ici la facture est créée et validée par le **vrai chemin applicatif**
/// (`invoices::create` + `invoices::validate_invoice`) : `total_amount` sort de
/// `compute_total`, et l'écriture de `generate_invoice_journal_lines`. On
/// efface ensuite la matérialisation que 16-1a vient de poser — reconstituant
/// l'état exact du parc antérieur — et on laisse le backfill la retrouver.
///
/// Les montants ne sont **pas ronds** et les taux de TVA **diffèrent d'une
/// ligne à l'autre** : l'écriture porte donc plusieurs crédits de TVA, et c'est
/// bien la condition (3) qui isole le crédit de produit.
///
/// *(Ajouté en passe 1 de `bmad-code-review`, finding Blind Hunter.)*
#[sqlx::test(migrations = false)]
async fn validated_invoice_from_the_real_engine_is_recovered_by_the_backfill(pool: MySqlPool) {
    use kesh_db::entities::{NewInvoice, NewInvoiceLine};
    use kesh_db::repositories::invoices;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    // ⛔ **Schéma COMPLET, et non « jusqu'au backfill ».** Ce test est le seul du
    // fichier à faire tourner le MOTEUR RÉEL (`invoices::create` +
    // `validate_invoice`), et le moteur d'aujourd'hui écrit et relit le schéma
    // d'aujourd'hui : monter un schéma de juillet lui fait rendre
    // `1054 Unknown column` dès qu'une colonne naît sur une table qu'il touche.
    // L'état pré-backfill est reconstitué plus bas par les `UPDATE … = NULL`,
    // qui sont de toute façon ce que le test mesure — et le backfill est rejoué
    // explicitement à la fin, depuis le SQL EMBARQUÉ dans le binaire.
    //
    // *(Relevé au gate complet de la Story 24-4a (#380), première migration à
    // ajouter une colonne à `journal_entries` depuis avril. Le montage
    // d'origine couplait ce test à l'immobilité du schéma, sans que rien ne le
    // dise.)*
    kesh_db::MIGRATOR.run(&pool).await.expect("schéma complet");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let other_revenue = insert_revenue_account(&pool, seeded.company_id, "3200").await;

    // 3 × 33.35 = 100.05 et 7 × 12.55 = 87.85 → HT = 187.90. Quantités ≠ 1 et
    // prix non ronds : `compute_line_total` fait un vrai produit arrondi, et
    // les deux taux distincts forcent DEUX lignes de crédit de TVA.
    let invoice_id = invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: vec![
                NewInvoiceLine {
                    description: "Prestation".into(),
                    quantity: dec!(3),
                    unit_price: dec!(33.35),
                    vat_rate: dec!(8.10),
                    revenue_account_id: None,
                },
                NewInvoiceLine {
                    description: "Marchandise".into(),
                    quantity: dec!(7),
                    unit_price: dec!(12.55),
                    vat_rate: dec!(2.60),
                    revenue_account_id: None,
                },
            ],
        },
    )
    .await
    .map(|(inv, _)| inv.id)
    .expect("création de la facture par le vrai chemin applicatif");

    invoices::validate_invoice(&pool, seeded.company_id, invoice_id, seeded.admin_user_id)
        .await
        .expect("validation par le vrai chemin applicatif");

    // L'invariant de la condition (3), MESURÉ sur la sortie du moteur et non
    // supposé : l'écriture porte exactement UNE ligne de crédit égale à
    // `total_amount`, et c'est celle du compte de produit. Si le moteur venait
    // à décorréler les deux, c'est CETTE assertion qui tomberait — avant que le
    // backfill ne devienne muet en production.
    let candidates: Vec<(i64, Decimal)> = sqlx::query_as(
        "SELECT jel.account_id, jel.credit \
         FROM journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.id = ? AND jel.credit > 0 AND jel.credit = i.total_amount",
    )
    .bind(invoice_id)
    .fetch_all(&pool)
    .await
    .expect("candidats de la condition (3) sur l'écriture réelle");
    assert_eq!(
        candidates.len(),
        1,
        "la condition (3) doit désigner EXACTEMENT un candidat sur une écriture MONO-COMPTE \
         produite par `generate_invoice_journal_lines` — c'est le cas de tout le parc \
         pré-16-1a, où aucune ligne ne porte de compte explicite. (Une facture ventilée sur \
         plusieurs comptes produit plusieurs crédits dont aucun n'égale `total_amount` : 0 \
         candidat, ligne laissée `NULL`, ce qui est le comportement voulu — ne pas lire cette \
         assertion comme une propriété de TOUTE facture.) Obtenu : {candidates:?}. Un 0 ici \
         signifie que \
         `total_amount` a cessé d'égaler le crédit de produit posé par le moteur, et donc que \
         le backfill no-opère sur 100 % du parc sans qu'aucun autre test ne le voie."
    );
    assert_eq!(
        candidates[0].0, revenue,
        "le candidat unique doit être le compte de produit, pas une ligne de TVA ni la créance"
    );
    assert_eq!(
        candidates[0].1,
        dec!(187.90),
        "HT attendu = 3 × 33.35 + 7 × 12.55"
    );

    // Reconstitution de l'état du parc antérieur : on efface la matérialisation
    // que 16-1a vient de poser. Le backfill n'a pas encore tourné (montage en
    // trois temps), il verra donc bien ces lignes.
    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "pré-condition : 16-1a matérialise le compte à la validation"
    );
    sqlx::query("UPDATE invoice_lines SET revenue_account_id = NULL WHERE invoice_id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .expect("retour à l'état pré-16-1a");

    // Et le défaut société change, comme dans le scénario réel : un backfill
    // qui lirait la configuration courante écrirait 3200 et serait démasqué.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_revenue_account_id = ? WHERE company_id = ?",
    )
    .bind(other_revenue)
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .expect("changement du défaut société");

    // Re-jeu du SQL RÉELLEMENT EMBARQUÉ dans le binaire — même idiome que
    // `backfill_is_idempotent` plus haut. Le backfill est idempotent par
    // construction (gardé par `revenue_account_id IS NULL`), ce qui est
    // précisément ce qui rend ce rejeu légitime.
    let backfill_sql = kesh_db::MIGRATOR
        .migrations
        .iter()
        .find(|m| m.version == BACKFILL_MIGRATION)
        .expect("migration de backfill introuvable dans le MIGRATOR")
        .sql
        .clone();
    sqlx::raw_sql(&backfill_sql)
        .execute(&pool)
        .await
        .expect("re-jeu du backfill");

    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "le backfill doit retrouver, sur une écriture produite par le MOTEUR RÉEL, le compte \
         que celui-ci a crédité ({revenue}) — et non le défaut société courant ({other_revenue})"
    );
    assert_eq!(
        remaining_null_invoice_lines(&pool).await,
        0,
        "aucun reliquat sur ce montage : la facture vient du chemin nominal"
    );
}

/// **Le pendant avoir du piège D-B3** — sans lui, la moitié du filet manque.
///
/// `backfills_when_vat_config_is_null` ne couvre que le volet facture : les
/// trois autres tests qui manipulent des avoirs tournent avec la config TVA
/// **posée par la fixture**. Muter le seul `<=>` du SECOND `UPDATE` en `<>`
/// laissait donc l'intégralité de la suite verte, alors qu'en exploitation le
/// backfill des avoirs no-operait sur toutes les installations concernées.
///
/// # Portée réelle, et pourquoi la fixture n'a PAS de ligne de TVA
///
/// Une société dont `default_vat_payable_account_id` est `NULL` n'a pu émettre
/// que des pièces **sans TVA** : `validate_invoice` passe cette colonne à
/// `generate_invoice_journal_lines` (`invoices.rs:1794`), qui échoue en
/// `ConfigurationRequired` dès que `total_vat > 0` (`invoices.rs:1497-1500`).
/// L'écriture montée ici est donc volontairement **exonérée** — c'est l'état
/// réellement atteignable, celui d'une PME suisse sous le seuil de CHF 100'000.
///
/// *(Ajouté en passe 2 de `bmad-code-review`, finding Blind Hunter.)*
#[sqlx::test(migrations = false)]
async fn backfills_credit_note_when_vat_config_is_null(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];

    // L'état par défaut de toute installation non configurée manuellement.
    sqlx::query(
        "UPDATE company_invoice_settings \
         SET default_vat_payable_account_id = NULL, default_vat_recoverable_account_id = NULL, \
             default_vat_decompte_account_id = NULL \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .expect("remise à NULL de la config TVA");

    // Facture exonérée : ni ligne de TVA, ni TTC — HT des deux côtés.
    let invoice_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[(receivable, HT, ZERO), (revenue, ZERO, HT)],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-EX",
        "cancelled",
        Some(invoice_entry),
    )
    .await;

    // Contre-passation exonérée, miroir strict.
    let cn_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[(revenue, HT, ZERO), (receivable, ZERO, HT)],
    )
    .await;
    let credit_note_id = insert_credit_note_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        invoice_id,
        "A-EX",
        cn_entry,
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        credit_note_line_accounts(&pool, credit_note_id).await,
        vec![Some(revenue), Some(revenue)],
        "config TVA `NULL` : le volet AVOIR doit backfiller malgré tout. Un `<>` au lieu du \
         `<=>` NULL-safe rendrait le prédicat NULL, aucune ligne ne serait candidate, et le \
         backfill des avoirs no-operait en silence sur toute installation exonérée (D-B3)"
    );
    assert_eq!(
        remaining_null_credit_note_lines(&pool).await,
        0,
        "aucun reliquat côté avoirs sur ce montage"
    );
}

/// **La garde `postable` du volet facture (condition 4).**
///
/// `invoice_lines.revenue_account_id` n'est pas un instantané du passé : c'est
/// la source de vérité que 16-1a D5 recopie dans tout avoir futur. Y écrire un
/// compte **collectif** produirait une donnée que l'application refuse
/// elle-même à la saisie (`RevenueAccountRejection::NotPostable`).
///
/// La facture témoin est indispensable : sans elle, le test passerait à vide si
/// le montage de fenêtre se décalait.
///
/// *(Ajouté en passe 2 de `bmad-code-review`, arbitrage Guy du 2026-07-29.)*
#[sqlx::test(migrations = false)]
async fn leaves_null_when_credited_account_is_not_postable(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];
    let collective = insert_non_postable_account(&pool, seeded.company_id, "3900").await;

    // Écriture ancienne créditant un compte de regroupement — possible parce
    // que la validation d'écriture ne contrôle pas `postable`.
    let collective_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (collective, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let not_postable = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-COLL",
        "validated",
        Some(collective_entry),
    )
    .await;

    // Témoin : prouve que le backfill a bien tourné sur ce montage.
    let canonical = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let witness = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-OK",
        "validated",
        Some(canonical),
    )
    .await;

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, not_postable).await,
        vec![None, None],
        "un compte NON IMPUTABLE ne doit jamais atterrir dans `invoice_lines.revenue_account_id` \
         ({collective}) : le backfill fabriquerait une donnée que la saisie refuse. Écarté avant \
         le `HAVING`, donc zéro candidat, donc `NULL` — conservateur, comme le veut AC-B2"
    );
    assert_eq!(
        invoice_line_accounts(&pool, witness).await,
        vec![Some(revenue), Some(revenue)],
        "TÉMOIN : le backfill a bien tourné — sans cette assertion le test passerait à vide"
    );
}

/// **Le test du `LEFT JOIN` sur `company_invoice_settings`** — sans lui, repasser
/// la jointure en `INNER JOIN` laisse toute la suite verte.
///
/// Les autres tests appellent tous `seed_accounting_company`, qui **crée** la
/// ligne de configuration. Aucun n'exerce donc le choix `LEFT` : une société
/// sans ligne de `company_invoice_settings` doit voir ses factures backfillées
/// quand même, l'absence de config valant **ensemble d'exclusion vide** —
/// exactement comme une colonne `NULL` (D-B3, un cran plus haut).
///
/// # Pourquoi ce garde-fou existe alors que le cas est inatteignable
///
/// En exploitation la ligne existe toujours pour une facture validée :
/// `get_or_create_default_in_tx` fait un `INSERT IGNORE` sur le chemin de
/// validation. Un `INNER JOIN` serait donc *probablement* sans conséquence — et
/// c'est précisément l'argument que la story refuse, parce qu'il fait dépendre
/// la correction du backfill d'une propriété d'un autre module. Le mode de
/// défaillance d'un `INNER JOIN` ici serait **le no-op silencieux de D-B3, un
/// cran plus haut** : des factures écartées sans trace, indiscernables d'un
/// reliquat conservateur légitime.
///
/// Ce test verrouille donc une décision de conception, pas un cas d'usage. Il
/// n'a pas besoin de facture témoin : il attend un backfill **positif**, donc
/// il tombe aussi si le montage se décale et que le backfill ne tourne pas.
///
/// *(Ajouté à la demande de Guy après la convergence de la boucle de revue ;
/// signalé LOW et reporté en passe 2, finding Edge Case Hunter.)*
#[sqlx::test(migrations = false)]
async fn backfills_when_company_has_no_invoice_settings_row(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];
    let vat = seeded.accounts["2000"];

    let entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[
            (receivable, TTC, ZERO),
            (revenue, ZERO, HT),
            (vat, ZERO, VAT),
        ],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-NOCFG",
        "validated",
        Some(entry),
    )
    .await;

    // Reconstitution du cas : société SANS ligne de configuration. La fixture en
    // crée une, on la retire — aucune FK ne pointe vers cette table, la
    // suppression est donc sans effet de bord.
    let deleted = sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .expect("suppression de la ligne de config")
        .rows_affected();
    assert_eq!(
        deleted, 1,
        "pré-condition : la fixture DOIT avoir créé une ligne de configuration, sinon ce test \
         ne reconstitue pas le cas qu'il prétend éprouver et devient tautologique"
    );

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        invoice_line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "société SANS ligne de `company_invoice_settings` : la ligne DOIT être backfillée. Si \
         elle est `NULL`, la jointure a été écrite `INNER JOIN` — les factures de cette société \
         sont alors écartées SANS TRACE, mode de défaillance identique au no-op silencieux de \
         D-B3 mais un cran plus haut"
    );
    assert_eq!(
        remaining_null_invoice_lines(&pool).await,
        0,
        "aucun reliquat : l'absence de configuration vaut ensemble d'exclusion VIDE, pas exclusion"
    );
}

/// **Le miroir avoir du test précédent** — et il n'est pas décoratif.
///
/// Mesuré : muter le `LEFT JOIN` du **seul** volet avoir en `INNER JOIN`
/// laissait les 12 autres tests **verts**. C'est la même classe d'asymétrie que
/// celle découverte en passe 2 sur le `<=>` — un garde-fou fileté d'un côté
/// seulement, et l'absence de filet de l'autre indiscernable du succès.
///
/// La fixture est montée **exonérée de TVA** (ni ligne de TVA, ni TTC) : c'est
/// l'état réellement atteignable pour une société sans configuration, pour la
/// même raison que dans `backfills_credit_note_when_vat_config_is_null`.
///
/// *(Ajouté dans le même geste que son pendant facture : corriger le site
/// signalé sans greper son symétrique est le mode d'échec que la § Propagation
/// post-patch de `CLAUDE.md` existe pour empêcher.)*
#[sqlx::test(migrations = false)]
async fn backfills_credit_note_when_company_has_no_invoice_settings_row(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before_backfill())
        .await
        .expect("migrations jusqu'au backfill");

    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let contact_id = insert_contact(&pool, seeded.company_id).await;
    let revenue = seeded.accounts["3000"];
    let receivable = seeded.accounts["1100"];

    let invoice_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        1,
        &[(receivable, HT, ZERO), (revenue, ZERO, HT)],
    )
    .await;
    let invoice_id = insert_invoice_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        "F-NOCFG-CN",
        "cancelled",
        Some(invoice_entry),
    )
    .await;

    let cn_entry = insert_entry(
        &pool,
        seeded.company_id,
        seeded.fiscal_year_id,
        2,
        &[(revenue, HT, ZERO), (receivable, ZERO, HT)],
    )
    .await;
    let credit_note_id = insert_credit_note_with_lines(
        &pool,
        seeded.company_id,
        contact_id,
        invoice_id,
        "A-NOCFG",
        cn_entry,
    )
    .await;

    let deleted = sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .expect("suppression de la ligne de config")
        .rows_affected();
    assert_eq!(
        deleted, 1,
        "pré-condition : la fixture DOIT avoir créé une ligne de configuration, sinon ce test \
         ne reconstitue pas le cas qu'il prétend éprouver"
    );

    kesh_db::MIGRATOR.run(&pool).await.expect("MIGRATOR.run()");

    assert_eq!(
        credit_note_line_accounts(&pool, credit_note_id).await,
        vec![Some(revenue), Some(revenue)],
        "société SANS ligne de `company_invoice_settings` : le volet AVOIR doit backfiller lui \
         aussi. Si la ligne est `NULL`, la jointure du second `UPDATE` a été écrite `INNER JOIN` \
         — et aucun autre test du fichier ne le verrait"
    );
    assert_eq!(
        remaining_null_credit_note_lines(&pool).await,
        0,
        "aucun reliquat côté avoirs"
    );
}
