//! Le rattrapage du lettrage — Story 15-1a2-ii (#518), AC6 : les migrations
//! `20261010000001_lettering_documents_backfill.sql` (M1, registre de rejeu)
//! et `20261010000002_lettering_reversal_pairs_backfill.sql` (M2, exemptée)
//! posent, sur des données d'avant la mise à jour, **ce que les gestes vivants
//! auraient posé** — la règle de la synchronisation des pièces
//! (`letterings::sync_invoice_in_tx`, `sync_supplier_invoice_in_tx`) et celle
//! de la contre-passation (15-1a-ii R6), recopiées en SQL parce qu'une migration
//! appliquée ne se modifie plus (P8). Ce test est ce qui tient les deux copies
//! ensemble.
//!
//! # Montage — vrai `MIGRATOR`, fenêtre résolue par version (P6)
//!
//! 1. migrations **jusqu'à M1 exclue** (`migrations_before`) ;
//! 2. (a) la fixture, **par les gestes vivants** (qui lettrent), puis le relevé
//!    des marques ;
//! 3. (b) effacement en SQL brut des marques `document` et `reversal` — **pas**
//!    des groupes `manual` : R6 laisse une ligne d'origine dans son groupe
//!    `manual` et son miroir ouvert, et l'effacer ferait apparier cette ligne
//!    par M2, à tort ;
//! 4. (c) `MIGRATOR.run()` — M1 puis M2 ;
//! 5. (d) comparaison, en **deux régimes** classés **par construction de la
//!    fixture** — jamais en appelant `lines_in_open_period`, le prédicat même
//!    que la synchronisation emploie (la frontière sortirait du code comparé,
//!    assertion verte par construction — D4-ter) ;
//! 6. (e) la synchronisation, appelée sur chaque pièce, n'écrit rien.
//!
//! Pré-requis : MariaDB démarré.

use chrono::NaiveDate;
use kesh_db::entities::{Journal, NewJournalEntry, NewJournalEntryLine, SettlementChoice};
use kesh_db::repositories::letterings::{self, Actor, SyncOutcome};
use kesh_db::repositories::{companies, journal_entries, supplier_invoices};
use kesh_db::test_fixtures::SeededCompany;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

mod common;

use common::{apply_migrations_up_to, migrations_before};

#[path = "support/lettering_documents.rs"]
mod lettering_support;
use lettering_support::*;

/// Version de M1 (registre de rejeu).
const M1: i64 = 20261010000001;
/// Version de M2 (exemptée du rejeu).
const M2: i64 = 20261010000002;

/// Une marque relevée : `(ligne, clé, origine)`.
type Marque = (i64, Option<i64>, Option<String>);

/// Toutes les marques d'une société, triées par ligne.
async fn marques(pool: &MySqlPool, company_id: i64) -> Vec<Marque> {
    sqlx::query_as(
        "SELECT jel.id, jel.lettering_key, jel.lettering_origin FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id WHERE je.company_id = ? ORDER BY jel.id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .expect("marques")
}

/// Les lignes d'une écriture, dans l'ordre de leur position.
async fn lignes_de(pool: &MySqlPool, entry_id: i64) -> Vec<i64> {
    sqlx::query_scalar("SELECT id FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order")
        .bind(entry_id)
        .fetch_all(pool)
        .await
        .expect("lignes")
}

/// L'écriture qui contre-passe `entry_id`.
async fn miroir(pool: &MySqlPool, entry_id: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
        .bind(entry_id)
        .fetch_one(pool)
        .await
        .expect("miroir")
}

/// Une écriture manuelle à N lignes `(compte, débit, crédit)` dans l'exercice
/// seedé ; rend `(écriture, lignes dans l'ordre)`.
async fn ecriture(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    le: NaiveDate,
    lignes: &[(i64, Decimal, Decimal)],
) -> (i64, Vec<i64>) {
    let je = journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: le,
            journal: Journal::OD,
            description: "Rattrapage".into(),
            project_id: None,
            lines: lignes
                .iter()
                .map(|(account_id, debit, credit)| NewJournalEntryLine {
                    account_id: *account_id,
                    debit: *debit,
                    credit: *credit,
                    project_id: None,
                })
                .collect(),
        },
    )
    .await
    .expect("écriture");
    let ids = je.lines.iter().map(|l| l.id).collect();
    (je.entry.id, ids)
}

/// Lettre à la main (mode `Manual`) les lignes données.
async fn lettrer_a_la_main(pool: &MySqlPool, seeded: &SeededCompany, lignes: &[i64]) {
    let mut tx = pool.begin().await.expect("tx");
    letterings::create_group_in_tx(
        &mut tx,
        seeded.company_id,
        lignes,
        letterings::Origin::Manual,
        letterings::Mode::Manual,
        Actor {
            user_id: seeded.admin_user_id,
            api_key_id: None,
        },
    )
    .await
    .expect("lettrage manuel");
    tx.commit().await.expect("commit");
}

/// Une pièce de la fixture.
#[derive(Debug, Clone, Copy)]
enum Piece {
    Client(i64),
    Fournisseur(i64),
}

/// Ce que la fixture du rattrapage rend au test.
struct Rattrapage {
    seeded: SeededCompany,
    /// (a) — les marques posées par les gestes vivants.
    vivant: Vec<Marque>,
    /// Les marques après (b) effacement et (c) M1 + M2.
    rattrape: Vec<Marque>,
    /// Lignes effacées en (b) — anti-muet.
    effacees: u64,
    /// **(d-ii)**, classé par construction : `(libellé, lignes)` des pièces et
    /// paires lettrées par le geste PUIS figées (verrou, exercice clos, exercice
    /// postérieur clos) — le rattrapage s'abstient. Sans borne, seules les deux
    /// pièces figées par un exercice y sont.
    figees: Vec<(&'static str, Vec<i64>)>,
    /// Toutes les pièces, pour (e).
    pieces: Vec<(String, Piece)>,
    /// La pièce dont le compte devient non lettrable APRÈS le rattrapage (C104).
    c104: (i64, i64),
}

/// La fixture étendue, puis (a), (b), (c). La base doit être montée jusqu'à M1
/// exclue.
///
/// **Calendrier.** Exercice seedé ramené à `2024-01-01 … 2030-12-31` ; deux
/// exercices antérieurs posés en SQL : `2018-2019` (resté **ouvert**) et
/// `2020-2023` (clos en SQL brut **en dernier**, hors de l'ordre de la clôture
/// — l'état hérité « exercice postérieur clos » ne se fabrique pas autrement).
/// La fixture de la 15-1a2-i pose une borne à `aujourd'hui − 300` ; elle est
/// levée le temps des pièces anciennes, puis — `verrou` — reposée à
/// `aujourd'hui − 250`.
///
/// ⚠️ **Deux montages, et c'est nécessaire** (mutations MU8, MU9 survivantes
/// au premier passage) : la borne couvre aussi les exercices 2019 et 2022, si
/// bien qu'avec elle la règle « exercice clos » et « exercice postérieur clos »
/// n'est jamais seule à décider — une mutation qui la retire passerait. Sans
/// borne (`verrou = false`), les pièces figées par la borne sont en période
/// ouverte, donc relettrées comme le vivant (d-i), et seules les deux pièces
/// figées par un exercice restent en (d-ii).
async fn rattrapage(pool: &MySqlPool, verrou: bool) -> Rattrapage {
    let base = seed_lettering_documents(pool).await;
    let seeded = base.seeded;
    let societe_id = seeded.company_id;
    // `(libellé, figée par la borne seule ?, lignes)`.
    let mut figees: Vec<(&'static str, bool, Vec<i64>)> = Vec::new();
    let mut pieces: Vec<(String, Piece)> = base
        .factures
        .iter()
        .map(|(l, id)| (format!("client : {l}"), Piece::Client(*id)))
        .collect();

    // La pièce de la 15-1a2-i passée sous la borne — (d-ii) par le verrou.
    let sous_la_borne = base
        .factures
        .iter()
        .find(|(l, _)| *l == "soldée puis sous la borne")
        .expect("pièce sous la borne")
        .1;
    figees.push((
        "client sous la borne",
        true,
        lignes_de_piece(pool, sous_la_borne)
            .await
            .iter()
            .map(|l| l.0)
            .collect(),
    ));

    // Exercices antérieurs, et la borne levée le temps des pièces anciennes.
    sqlx::query("UPDATE fiscal_years SET start_date = '2024-01-01' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .expect("exercice seedé");
    let exercice = |nom: &'static str, debut: &'static str, fin: &'static str| async move {
        sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, ?, ?, 'Open')",
        )
        .bind(societe_id)
        .bind(nom)
        .bind(debut)
        .bind(fin)
        .execute(pool)
        .await
        .expect("exercice")
        .last_insert_id() as i64
    };
    exercice("2018-2019", "2018-01-01", "2019-12-31").await;
    let fy_ancien = exercice("2020-2023", "2020-01-01", "2023-12-31").await;
    companies::unlock_books(
        pool,
        seeded.admin_user_id,
        societe_id,
        None,
        "montage du rattrapage".into(),
    )
    .await
    .expect("borne levée");

    let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("date");
    // (d-ii) par un exercice POSTÉRIEUR clos : facture client de 2019.
    let p0 = facture(pool, &seeded, dec!(55.00), d("2019-06-01")).await;
    regler(pool, &seeded, p0, dec!(55.00), d("2019-07-01")).await;
    assert!(lettree_document(pool, p0).await, "montage : p0 lettrée");
    figees.push((
        "client sous un exercice postérieur clos",
        false,
        lignes_de_piece(pool, p0)
            .await
            .iter()
            .map(|l| l.0)
            .collect(),
    ));
    pieces.push(("client 2019".into(), Piece::Client(p0)));

    // Les achats.
    let ach = achats(pool, &seeded).await;
    let jour = jours_avant;
    let ff = |ttc: Decimal, date: NaiveDate, numero: Option<&'static str>| {
        let seeded = &seeded;
        let ach = &ach;
        async move { facture_fournisseur(pool, seeded, ach, ttc, date, numero).await }
    };

    // (d-ii) par un exercice clos : facture fournisseur de 2022.
    let p1 = ff(dec!(66.00), d("2022-03-01"), Some("R-2022")).await;
    payer(pool, &seeded, &ach, p1, d("2022-04-01")).await;
    assert!(
        fournisseur_lettree_document(pool, p1).await,
        "montage : p1 lettrée"
    );
    figees.push((
        "fournisseur dans un exercice clos",
        false,
        lignes_de_piece_fournisseur(pool, p1)
            .await
            .iter()
            .map(|l| l.0)
            .collect(),
    ));
    pieces.push(("fournisseur 2022".into(), Piece::Fournisseur(p1)));

    // (d-ii) par le verrou : facture fournisseur, puis paire de contre-passation
    // dont le miroir est ANTIDATÉ en SQL brut (la contre-passation date du jour,
    // et le verrou refuse une borne `>= aujourd'hui` : seule façon de fabriquer
    // une paire entièrement sous la borne).
    let figee = ff(dec!(77.00), jour(290), Some("R-figee")).await;
    payer(pool, &seeded, &ach, figee, jour(280)).await;
    figees.push((
        "fournisseur sous la borne",
        true,
        lignes_de_piece_fournisseur(pool, figee)
            .await
            .iter()
            .map(|l| l.0)
            .collect(),
    ));
    pieces.push((
        "fournisseur sous la borne".into(),
        Piece::Fournisseur(figee),
    ));
    let (e_figee, l_figee) = ecriture(
        pool,
        &seeded,
        jour(270),
        &[
            (seeded.accounts["1000"], dec!(12.00), Decimal::ZERO),
            (seeded.accounts["2000"], Decimal::ZERO, dec!(12.00)),
        ],
    )
    .await;
    journal_entries::reverse(pool, societe_id, e_figee, seeded.admin_user_id)
        .await
        .expect("contre-passation");
    let m_figee = miroir(pool, e_figee).await;
    sqlx::query("UPDATE journal_entries SET entry_date = ? WHERE id = ?")
        .bind(jour(260))
        .bind(m_figee)
        .execute(pool)
        .await
        .expect("miroir antidaté");
    let mut paire_figee = l_figee.clone();
    paire_figee.extend(lignes_de(pool, m_figee).await);
    figees.push(("paire de contre-passation sous la borne", true, paire_figee));

    // (d-i) — les gestes fournisseurs.
    let virement = ff(dec!(100.00), jour(200), Some("V-1")).await;
    payer(pool, &seeded, &ach, virement, jour(150)).await;
    let lot1 = ff(dec!(101.00), jour(200), Some("L-1")).await;
    let lot2 = ff(dec!(102.00), jour(200), Some("L-2")).await;
    payer_par_lot(pool, &seeded, &ach, vec![lot1, lot2], jour(140)).await;
    let interne = ff(dec!(103.00), jour(200), Some("I-1")).await;
    payer_par(
        pool,
        &seeded,
        interne,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        jour(140),
    )
    .await;
    supplier_invoices::cancel_settlement(pool, societe_id, interne, seeded.admin_user_id)
        .await
        .expect("paiement interne annulé");
    let payee_annulee = ff(dec!(104.00), jour(200), Some("A-1")).await;
    payer(pool, &seeded, &ach, payee_annulee, jour(140)).await;
    supplier_invoices::cancel(pool, societe_id, payee_annulee, seeded.admin_user_id)
        .await
        .expect("facture payée annulée");
    let detache_cp = ff(dec!(105.00), jour(200), Some("A-2")).await;
    let detache = payer(pool, &seeded, &ach, detache_cp, jour(140)).await;
    supplier_invoices::cancel(pool, societe_id, detache_cp, seeded.admin_user_id)
        .await
        .expect("facture payée annulée");
    journal_entries::reverse(pool, societe_id, detache, seeded.admin_user_id)
        .await
        .expect("règlement détaché contre-passé");
    let ouverte_annulee = ff(dec!(106.00), jour(200), Some("A-3")).await;
    supplier_invoices::cancel(pool, societe_id, ouverte_annulee, seeded.admin_user_id)
        .await
        .expect("facture ouverte annulée");
    let ouverte = ff(dec!(107.00), jour(200), Some("O-1")).await;
    let sans_numero = ff(dec!(108.00), jour(200), None).await;
    payer(pool, &seeded, &ach, sans_numero, jour(140)).await;

    // C104 : une dette propre (2011), lettrable pendant (a)–(d), rendue non
    // lettrable APRÈS le rattrapage — (e) l'éprouve.
    let dette_c104 = compte(pool, societe_id, "2011", "Liability").await;
    designer_dette(pool, societe_id, dette_c104).await;
    let c104 = ff(dec!(109.00), jour(200), Some("C-104")).await;
    designer_dette(pool, societe_id, ach.payable).await;
    payer(pool, &seeded, &ach, c104, jour(140)).await;
    assert!(
        fournisseur_lettree_document(pool, c104).await,
        "montage : C104 lettrée"
    );

    // Dette non lettrable dès le départ (2012) : jamais de groupe.
    let dette_bancaire = compte(pool, societe_id, "2012", "Liability").await;
    designer_dette(pool, societe_id, dette_bancaire).await;
    let non_lettrable = ff(dec!(110.00), jour(200), Some("N-1")).await;
    designer_dette(pool, societe_id, ach.payable).await;
    rattacher_a_un_compte_bancaire(pool, societe_id, dette_bancaire).await;
    payer(pool, &seeded, &ach, non_lettrable, jour(140)).await;

    for (libelle, id) in [
        ("virement", virement),
        ("lot 1", lot1),
        ("lot 2", lot2),
        ("compte interne, paiement annulé", interne),
        ("payée puis annulée", payee_annulee),
        ("payée, annulée, règlement contre-passé", detache_cp),
        ("ouverte annulée", ouverte_annulee),
        ("ouverte", ouverte),
        ("sans numéro", sans_numero),
        ("C104", c104),
        ("dette non lettrable", non_lettrable),
    ] {
        pieces.push((format!("fournisseur : {libelle}"), Piece::Fournisseur(id)));
    }

    // Contre-passations d'écritures manuelles : une paire libre (M2) ; une
    // ligne d'origine lettrée À LA MAIN avant la contre-passation (R6 la saute,
    // son miroir reste ouvert — le groupe `manual` n'est pas effacé en (b)).
    let (libre, _) = ecriture(
        pool,
        &seeded,
        jour(120),
        &[
            (seeded.accounts["1000"], dec!(20.00), Decimal::ZERO),
            (seeded.accounts["2000"], Decimal::ZERO, dec!(20.00)),
        ],
    )
    .await;
    journal_entries::reverse(pool, societe_id, libre, seeded.admin_user_id)
        .await
        .expect("contre-passation libre");
    let (manuelle, l_manuelle) = ecriture(
        pool,
        &seeded,
        jour(120),
        &[
            (seeded.accounts["1000"], dec!(15.00), Decimal::ZERO),
            (seeded.accounts["4000"], Decimal::ZERO, dec!(15.00)),
        ],
    )
    .await;
    let (_, l_solde) = ecriture(
        pool,
        &seeded,
        jour(120),
        &[
            (seeded.accounts["4000"], dec!(15.00), Decimal::ZERO),
            (seeded.accounts["1000"], Decimal::ZERO, dec!(15.00)),
        ],
    )
    .await;
    lettrer_a_la_main(pool, &seeded, &[l_manuelle[0], l_solde[1]]).await;
    journal_entries::reverse(pool, societe_id, manuelle, seeded.admin_user_id)
        .await
        .expect("contre-passation d'une ligne lettrée à la main");

    // La borne reposée (montage `verrou`), l'exercice 2020-2023 clos en SQL brut.
    if verrou {
        companies::lock_books(pool, seeded.admin_user_id, societe_id, jour(250))
            .await
            .expect("verrou");
    }
    let figees: Vec<(&'static str, Vec<i64>)> = figees
        .into_iter()
        .filter(|(_, par_la_borne, _)| verrou || !par_la_borne)
        .map(|(libelle, _, lignes)| (libelle, lignes))
        .collect();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy_ancien)
        .execute(pool)
        .await
        .expect("exercice 2020-2023 clos");

    // (a) le relevé, (b) l'effacement, (c) M1 puis M2.
    let vivant = marques(pool, societe_id).await;
    let effacees = sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_origin IN ('document', 'reversal')",
    )
    .execute(pool)
    .await
    .expect("effacement")
    .rows_affected();
    kesh_db::MIGRATOR.run(pool).await.expect("M1 et M2");
    let rattrape = marques(pool, societe_id).await;

    Rattrapage {
        seeded,
        vivant,
        rattrape,
        effacees,
        figees,
        pieces,
        c104: (c104, dette_c104),
    }
}

/// AC6 (d-i) — hors des lignes figées, le rattrapage rend exactement les
/// marques du vivant.
fn assert_d_i(r: &Rattrapage) {
    let figees: Vec<i64> = r.figees.iter().flat_map(|(_, l)| l.clone()).collect();
    let ecarts: Vec<String> = r
        .vivant
        .iter()
        .zip(r.rattrape.iter())
        .filter(|(v, _)| !figees.contains(&v.0))
        .filter(|(v, a)| v != a)
        .map(|(v, a)| {
            format!(
                "ligne {} : vivant {:?}/{:?}, rattrapage {:?}/{:?}",
                v.0, v.1, v.2, a.1, a.2
            )
        })
        .collect();
    assert_eq!(r.vivant.len(), r.rattrape.len(), "mêmes lignes");
    assert!(
        ecarts.is_empty(),
        "⛔ le rattrapage diverge de la synchronisation (AC6 d-i) :\n{}",
        ecarts.join("\n")
    );
}

/// AC6 (d-ii) — chaque ligne figée a été lettrée par le geste et reste ouverte
/// après le rattrapage.
fn assert_d_ii(r: &Rattrapage) {
    let marque_de = |marques: &[Marque], id: i64| {
        marques
            .iter()
            .find(|m| m.0 == id)
            .map(|m| (m.1, m.2.clone()))
            .expect("ligne relevée")
    };
    for (libelle, lignes) in &r.figees {
        assert!(lignes.len() >= 2, "montage : {libelle}");
        for id in lignes {
            assert!(
                marque_de(&r.vivant, *id).0.is_some(),
                "montage : {libelle}, ligne {id} lettrée par le geste"
            );
            assert_eq!(
                marque_de(&r.rattrape, *id),
                (None, None),
                "{libelle}, ligne {id} : le rattrapage s'abstient"
            );
        }
    }
}

/// Le nombre de lignes `(a)` d'origine `origine`.
fn compte_origine(marques: &[Marque], origine: &str) -> usize {
    marques
        .iter()
        .filter(|m| m.2.as_deref() == Some(origine))
        .count()
}

/// AC6 (d-i), (e) — hors des pièces figées, les marques du rattrapage sont
/// **identiques** à celles des gestes vivants, sans exception ; puis la
/// synchronisation, appelée sur chaque pièce — y compris celle dont le compte
/// devient non lettrable après coup (C104 : son groupe survit) —, n'écrit rien,
/// ne rend jamais `Invariant` et n'ajoute aucune entrée d'audit.
#[sqlx::test(migrations = false)]
async fn backfill_matches_live_sync(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before(M1, "lettering_documents_backfill"))
        .await
        .expect("migrations avant M1");
    let r = rattrapage(&pool, true).await;

    // Anti-muet : le vivant a posé des trois origines, et (b) a effacé.
    for origine in ["document", "reversal", "manual"] {
        assert!(
            compte_origine(&r.vivant, origine) > 0,
            "montage : aucune ligne `{origine}` posée par les gestes"
        );
    }
    assert_eq!(
        r.effacees as usize,
        compte_origine(&r.vivant, "document") + compte_origine(&r.vivant, "reversal"),
        "montage : (b) efface toutes les marques `document` et `reversal`"
    );

    // (d-i) — identiques, hors (d-ii).
    assert_d_i(&r);

    // (e) — la synchronisation n'écrit rien. Le compte de C104 devient non
    // lettrable ICI, après le rattrapage : son groupe survit (C104), et l'étape 3
    // terminale rend `AccountNotLetterable` sans rien écrire.
    let (c104, dette_c104) = r.c104;
    rattacher_a_un_compte_bancaire(&pool, r.seeded.company_id, dette_c104).await;
    assert!(
        fournisseur_lettree_document(&pool, c104).await,
        "montage : le groupe de C104 a été reposé par M1"
    );
    let marques_avant = marques(&pool, r.seeded.company_id).await;
    let audits_avant: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut issues = Vec::new();
    for (libelle, piece) in &r.pieces {
        let mut tx = pool.begin().await.expect("tx");
        sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
            .bind(r.seeded.fiscal_year_id)
            .execute(&mut *tx)
            .await
            .expect("exercice tenu");
        let acteur = Actor {
            user_id: r.seeded.admin_user_id,
            api_key_id: None,
        };
        let issue = match piece {
            Piece::Client(id) => {
                letterings::sync_invoice_in_tx(
                    &mut tx,
                    r.seeded.company_id,
                    *id,
                    r.seeded.fiscal_year_id,
                    acteur,
                )
                .await
            }
            Piece::Fournisseur(id) => {
                letterings::sync_supplier_invoice_in_tx(
                    &mut tx,
                    r.seeded.company_id,
                    *id,
                    r.seeded.fiscal_year_id,
                    acteur,
                )
                .await
            }
        };
        tx.commit().await.expect("commit");
        match &issue {
            Ok(
                SyncOutcome::Unchanged
                | SyncOutcome::AbstainedClosedPeriods
                | SyncOutcome::AccountNotLetterable,
            ) => {}
            autre => issues.push(format!("{libelle} : {:?}", autre.as_ref())),
        }
        if let Piece::Fournisseur(id) = piece
            && *id == c104
        {
            assert!(
                matches!(issue, Ok(SyncOutcome::AccountNotLetterable)),
                "C104 : {issue:?}"
            );
        }
    }
    assert!(
        issues.is_empty(),
        "⛔ la synchronisation écrirait après le rattrapage (AC6 e) :\n{}",
        issues.join("\n")
    );
    assert_eq!(
        marques(&pool, r.seeded.company_id).await,
        marques_avant,
        "(e) rien d'écrit"
    );
    let audits_apres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audits_apres, audits_avant, "(e) aucune entrée d'audit");
}

/// AC6 (d-ii) — une pièce ou une paire lettrée par le geste PUIS figée — sous
/// la borne, dans un exercice clos, sous un exercice postérieur clos — n'est
/// pas lettrée par le rattrapage : ses lignes, lettrées par le vivant, restent
/// **ouvertes**. Différence attendue, assertée pièce par pièce.
#[sqlx::test(migrations = false)]
async fn backfill_abstains_on_closed_and_locked_history(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before(M1, "lettering_documents_backfill"))
        .await
        .expect("migrations avant M1");
    let r = rattrapage(&pool, true).await;
    assert_eq!(r.figees.len(), 5, "montage : les cinq pièces figées");
    assert_d_ii(&r);
}

/// AC6 (d-ii), **sans borne** — la règle « exercice clos » et « exercice
/// postérieur clos » décide seule : les deux pièces des exercices 2019 et 2022
/// restent ouvertes, et les pièces que la borne aurait figées sont relettrées
/// comme le vivant (d-i).
#[sqlx::test(migrations = false)]
async fn backfill_abstains_on_closed_years_without_lock(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before(M1, "lettering_documents_backfill"))
        .await
        .expect("migrations avant M1");
    let r = rattrapage(&pool, false).await;
    assert_eq!(
        r.figees.len(),
        2,
        "montage : les deux pièces figées par un exercice"
    );
    assert_d_ii(&r);
    assert_d_i(&r);
}

/// AC6 — les paires de contre-passation sont appariées **par position** (rang
/// dans `ORDER BY line_order`), non par compte et montant : une écriture qui
/// porte deux lignes identiques voit chacune lettrée avec SON miroir. Et le
/// partage entre les deux migrations : M1 porte la paire de l'**achat** d'une
/// facture annulée, M2 les autres — M1 seule ne pose pas la paire manuelle.
#[sqlx::test(migrations = false)]
async fn backfill_pairs_reversals_by_position(pool: MySqlPool) {
    apply_migrations_up_to(&pool, migrations_before(M1, "lettering_documents_backfill"))
        .await
        .expect("migrations avant M1");
    let seeded = societe(&pool).await;
    let ach = achats(&pool, &seeded).await;
    let (jumelles, lignes) = ecriture(
        &pool,
        &seeded,
        jours_avant(100),
        &[
            (seeded.accounts["1000"], dec!(30.00), Decimal::ZERO),
            (seeded.accounts["1000"], dec!(30.00), Decimal::ZERO),
            (seeded.accounts["2000"], Decimal::ZERO, dec!(60.00)),
        ],
    )
    .await;
    journal_entries::reverse(&pool, seeded.company_id, jumelles, seeded.admin_user_id)
        .await
        .expect("contre-passation");
    let lignes_miroir = lignes_de(&pool, miroir(&pool, jumelles).await).await;
    let annulee = facture_fournisseur(
        &pool,
        &seeded,
        &ach,
        dec!(40.00),
        jours_avant(100),
        Some("X"),
    )
    .await;
    supplier_invoices::cancel(&pool, seeded.company_id, annulee, seeded.admin_user_id)
        .await
        .expect("facture annulée");
    let achat_annule = achat(&pool, annulee).await;
    let ligne_achat: i64 = sqlx::query_scalar(
        "SELECT id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(achat_annule)
    .bind(ach.payable)
    .fetch_one(&pool)
    .await
    .unwrap();
    let vivant = marques(&pool, seeded.company_id).await;
    sqlx::query(
        "UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL \
         WHERE lettering_origin IS NOT NULL",
    )
    .execute(&pool)
    .await
    .unwrap();

    // M1 seule : la paire de l'achat, pas celle de l'écriture manuelle.
    apply_migrations_up_to(
        &pool,
        migrations_before(M2, "lettering_reversal_pairs_backfill"),
    )
    .await
    .expect("M1");
    let apres_m1 = marques(&pool, seeded.company_id).await;
    let cle = |marques: &[Marque], id: i64| marques.iter().find(|m| m.0 == id).expect("ligne").1;
    assert_eq!(
        cle(&apres_m1, ligne_achat),
        Some(ligne_achat),
        "M1 : la paire de l'achat"
    );
    for id in lignes.iter().chain(lignes_miroir.iter()) {
        assert_eq!(
            cle(&apres_m1, *id),
            None,
            "M1 ne pose pas la paire manuelle ({id})"
        );
    }

    // M2 : chaque ligne avec le miroir de SA position.
    kesh_db::MIGRATOR.run(&pool).await.expect("M2");
    let apres = marques(&pool, seeded.company_id).await;
    for (origine, miroir) in lignes.iter().zip(lignes_miroir.iter()) {
        assert_eq!(cle(&apres, *origine), Some(*origine), "ligne {origine}");
        assert_eq!(
            cle(&apres, *miroir),
            Some(*origine),
            "miroir de la ligne {origine}"
        );
    }
    assert_eq!(apres, vivant, "le rattrapage rend exactement le vivant");
}
