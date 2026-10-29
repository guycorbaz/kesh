//! Story 32-9a (refs #723) — les propriétés de chaque outil de lecture,
//! éprouvées depuis le registre et la table des témoins (AC-a11, AC-a17 à
//! AC-a22, AC-a7).
//!
//! ⛔ « Égal » veut dire égal octet pour octet : une réponse se compare entière
//! à la réponse attendue, construite des constantes de `mcp::textes`. Aucune
//! donnée réelle : la démonstration et les pièges `PIEGE_329_*`.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use axum::http::StatusCode;
use chrono::Utc;
use kesh_api::mcp::Registre;
use kesh_api::mcp::outils;
use kesh_api::mcp::textes::*;
use kesh_api::services::acces::Portee;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::supplier_invoice::{NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::entities::{
    AccountType, Journal, McpScope, NewAccount, NewJournalEntry, NewJournalEntryLine, Role,
    SendValidation,
};
use kesh_db::repositories::{
    accounts, contacts, imported_supplier_invoices, invoices, journal_entries, supplier_invoices,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::mcp::{
    appel, attendu_depose, attendu_erreur, attendu_outil_en_erreur, attendu_valide, envoyer,
    lire_envoi, numero_depose, requete_api, session,
};
use common::mcp_pieges::*;

/// Un identifiant qu'aucune fixture ne pose.
const INEXISTANT: i64 = 999_999_999;

async fn deux(pool: &MySqlPool) -> (Societe, Societe) {
    let a = societe_piegee(pool, "Société A 329", 'A').await;
    let b = societe_piegee(pool, "Société B 329", 'B').await;
    (a, b)
}

fn texte(reponse: &Value) -> String {
    let c = reponse["result"]["content"].as_array().unwrap();
    assert_eq!(c.len(), 1, "un seul bloc : {reponse}");
    c[0]["text"].as_str().unwrap().to_string()
}

fn schema(outil: &str) -> Value {
    let (_, s, _) = Registre::standard()
        .inventaire()
        .into_iter()
        .find(|(n, _, _)| *n == outil)
        .unwrap();
    serde_json::from_str(s).unwrap()
}

fn portee(outil: &str) -> Portee {
    Registre::standard()
        .inventaire()
        .into_iter()
        .find(|(n, _, _)| *n == outil)
        .unwrap()
        .2
}

// ---------------------------------------------------------------------------
// AC-a17 — la table des témoins
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn chaque_outil_a_son_temoin(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let liste: BTreeSet<String> = banc(&pool).liste(&jeton).await.into_iter().collect();
    let temoins: BTreeSet<String> = TEMOINS
        .iter()
        .map(|(n, _, _)| n.to_string())
        .chain(HORS_REGISTRE.iter().map(|s| s.to_string()))
        .collect();
    assert_eq!(temoins, liste);
    for (n, jeux, _) in TEMOINS {
        assert!(!jeux(&a.p).is_empty(), "{n} : au moins un jeu");
    }
}

// ---------------------------------------------------------------------------
// AC-a11 — les schémas
// ---------------------------------------------------------------------------

/// (outil, maximum attendu, défaut attendu) : maximum = min(plafond du
/// service, maximum de l'outil).
fn bornes_attendues() -> Vec<(&'static str, i64, i64)> {
    use kesh_api::services::{ecritures, factures, factures_fournisseur, postes_ouverts};
    vec![
        (
            "kesh_plan_comptable",
            outils::plan_comptable::LIMITE_MAX_MCP,
            outils::plan_comptable::LIMITE_DEFAUT,
        ),
        (
            "kesh_ecritures",
            ecritures::LIMITE_MAX.min(outils::ecritures::LIMITE_MAX_MCP),
            outils::ecritures::LIMITE_DEFAUT,
        ),
        (
            "kesh_grand_livre",
            kesh_report::general_ledger::MAX_LEDGER_LIMIT.min(outils::grand_livre::LIMITE_MAX_MCP),
            outils::grand_livre::LIMITE_DEFAUT,
        ),
        (
            "kesh_balance",
            outils::rapports::LIMITE_MAX_MCP_BALANCE,
            outils::rapports::LIMITE_DEFAUT_BALANCE,
        ),
        (
            "kesh_postes_ouverts",
            postes_ouverts::LIMITE_MAX.min(outils::postes_ouverts::LIMITE_MAX_MCP),
            outils::postes_ouverts::LIMITE_DEFAUT,
        ),
        (
            "kesh_factures",
            factures::LIMITE_MAX.min(outils::factures::LIMITE_MAX_MCP),
            outils::factures::LIMITE_DEFAUT,
        ),
        (
            "kesh_echeancier",
            factures::LIMITE_MAX.min(outils::factures::LIMITE_MAX_MCP),
            outils::factures::LIMITE_DEFAUT,
        ),
        (
            "kesh_balance_agee",
            outils::balance_agee::LIMITE_MAX_MCP,
            outils::balance_agee::LIMITE_DEFAUT,
        ),
        (
            "kesh_factures_fournisseur",
            factures_fournisseur::LIMITE_MAX.min(outils::factures_fournisseur::LIMITE_MAX_MCP),
            outils::factures_fournisseur::LIMITE_DEFAUT,
        ),
        // Story 32-9b (AC-b2).
        (
            "kesh_contacts",
            kesh_api::services::carnet::LIMITE_MAX.min(outils::contacts::LIMITE_MAX_MCP),
            outils::contacts::LIMITE_DEFAUT,
        ),
        (
            "kesh_pieces",
            kesh_api::services::pieces::LIMITE_MAX.min(outils::pieces::LIMITE_MAX_MCP),
            outils::pieces::LIMITE_DEFAUT,
        ),
    ]
}

#[test]
fn bornes_des_schemas() {
    let attendues = bornes_attendues();
    let mut pagines = BTreeSet::new();
    for (nom, s, _) in Registre::standard().inventaire() {
        let s: Value = serde_json::from_str(s).unwrap();
        if s["properties"].get("limit").is_some() {
            pagines.insert(nom);
        }
    }
    assert_eq!(
        pagines,
        attendues
            .iter()
            .map(|(n, _, _)| *n)
            .collect::<BTreeSet<_>>(),
        "les outils paginés"
    );
    // Valeurs mesurées (§ *Les tailles mesurées*), écrites en dur ; la
    // balance âgée révisée sur la mesure (C-32-9-44 : 40 / 75 → 38 / 68).
    let mesurees: BTreeMap<&str, (i64, i64)> = [
        ("kesh_plan_comptable", (70, 40)),
        ("kesh_ecritures", (22, 12)),
        ("kesh_grand_livre", (30, 15)),
        ("kesh_balance", (55, 30)),
        ("kesh_postes_ouverts", (24, 12)),
        ("kesh_factures", (40, 20)),
        ("kesh_echeancier", (40, 20)),
        ("kesh_balance_agee", (68, 38)),
        ("kesh_factures_fournisseur", (70, 40)),
        ("kesh_contacts", (22, 12)),
        ("kesh_pieces", (65, 35)),
    ]
    .into();
    for (nom, max, defaut) in attendues {
        let l = &schema(nom)["properties"]["limit"];
        assert_eq!(l["maximum"], json!(max), "{nom} maximum");
        assert_eq!(l["default"], json!(defaut), "{nom} défaut");
        assert_eq!(l["minimum"], json!(1), "{nom} minimum");
        assert_eq!(mesurees[nom], (max, defaut), "{nom} : valeurs de la fiche");
    }
}

/// Story 32-9b — le statut d'une pièce dit une seule valeur partout : la
/// contrainte de la base (`services::pieces::STATUTS`), la table du miroir,
/// `code()`, sa sérialisation, sa lecture en argument et l'énumération du
/// schéma de `kesh_pieces`.
#[test]
fn statut_piece_une_seule_valeur() {
    use kesh_api::mcp::outils::commun::{STATUTS_PIECE, StatutPiece};
    let service: Vec<&str> = kesh_api::services::pieces::STATUTS.to_vec();
    let table: Vec<&str> = STATUTS_PIECE.iter().map(|(c, _)| *c).collect();
    assert_eq!(table, service);
    assert_eq!(
        schema("kesh_pieces")["properties"]["statut"]["enum"],
        json!(service)
    );
    assert_eq!(schema("kesh_pieces")["required"], json!(["statut"]));
    for (c, s) in STATUTS_PIECE {
        assert_eq!(s.code(), *c);
        assert_eq!(serde_json::to_value(s).unwrap(), json!(c));
        let lu: StatutPiece = serde_json::from_value(json!(c)).unwrap();
        assert_eq!(lu, *s);
    }
    assert!(serde_json::from_value::<StatutPiece>(json!("PIEGE")).is_err());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn proprietes_du_schema_acceptees(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for (outil, _, complet) in TEMOINS {
        let args = complet(&a.p);
        let cles: BTreeSet<&String> = args.as_object().unwrap().keys().collect();
        let s = schema(outil);
        let proprietes: BTreeSet<&String> = s["properties"].as_object().unwrap().keys().collect();
        assert_eq!(
            cles, proprietes,
            "{outil} : l'argument complet nomme chaque propriété"
        );
        assert_eq!(s["additionalProperties"], json!(false), "{outil}");
        let r = b.appeler(&jeton, outil, args.clone()).await;
        assert_eq!(r["result"]["isError"], json!(false), "{outil} {args} : {r}");
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn arguments_inconnus_refuses(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for (outil, _, _) in TEMOINS {
        let r = b.appeler(&jeton, outil, json!({"PIEGE_ARG_329": 1})).await;
        assert_eq!(
            r,
            attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES),
            "{outil}"
        );
    }
    // Un `limit` au-delà du maximum de l'outil, sous le plafond du service,
    // est refusé par l'outil (non transmis).
    let r = b
        .appeler(&jeton, "kesh_ecritures", json!({"limit": 23}))
        .await;
    assert_eq!(r, attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES));
}

// ---------------------------------------------------------------------------
// AC-a18 — l'autorisation
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn autorisation_par_outil(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let b = banc(&pool);
    let mut cles: BTreeMap<McpScope, (String, String)> = BTreeMap::new();
    for p in LECTURE {
        let seule = cle_de(&pool, &a, &[p], SendValidation::Exempt).await;
        let autres: Vec<McpScope> = LECTURE.into_iter().filter(|x| *x != p).collect();
        let autres = cle_de(&pool, &a, &autres, SendValidation::Exempt).await;
        cles.insert(p, (seule, autres));
    }
    for (outil, _, _) in TEMOINS {
        let (seule, autres) = &cles[&portee(outil)];
        let args = premier_jeu(outil, &a.p);
        let r = b.appeler(seule, outil, args.clone()).await;
        assert_eq!(
            r["result"]["isError"],
            json!(false),
            "{outil} sous sa portée : {r}"
        );
        let r = b.appeler(autres, outil, args).await;
        assert_eq!(
            r,
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_OUTIL_INCONNU),
            "{outil} hors portée = inconnu"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tools_list_par_portee(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let b = banc(&pool);
    // Les listes attendues sont **écrites** (fiche, AC-a10), non tirées du
    // registre : comparé à lui-même, le registre ne verrait pas une portée
    // fausse (Ma9 survivait à la première version de ce test).
    let livres = [
        "kesh_version",
        "kesh_exercices",
        "kesh_plan_comptable",
        "kesh_ecritures",
        "kesh_ecriture",
        "kesh_grand_livre",
        "kesh_balance",
        "kesh_bilan",
        "kesh_compte_de_resultat",
        "kesh_postes_ouverts",
        "kesh_lire_envoi",
    ];
    let factures = [
        "kesh_factures",
        "kesh_facture",
        "kesh_echeancier",
        "kesh_balance_agee",
        "kesh_factures_fournisseur",
        "kesh_facture_fournisseur",
        "kesh_lire_envoi",
    ];
    // Story 32-9b (AC-b2) : les contacts et les pièces.
    let contacts = ["kesh_contacts", "kesh_contact", "kesh_lire_envoi"];
    let pieces = ["kesh_pieces", "kesh_lire_envoi"];
    for (p, attendu) in [
        (McpScope::BooksRead, &livres[..]),
        (McpScope::InvoicesRead, &factures[..]),
        (McpScope::ContactsRead, &contacts[..]),
        (McpScope::DocumentsRead, &pieces[..]),
    ] {
        let jeton = cle_de(&pool, &a, &[p], SendValidation::Exempt).await;
        assert_eq!(b.liste(&jeton).await, attendu, "{p:?}");
    }
}

// ---------------------------------------------------------------------------
// AC-a19 — la lecture seule ; AC-a7 — la trace des rapports
// ---------------------------------------------------------------------------

const RAPPORTS: [&str; 5] = [
    "kesh_grand_livre",
    "kesh_balance",
    "kesh_bilan",
    "kesh_compte_de_resultat",
    "kesh_balance_agee",
];

async fn sommes_de_controle(pool: &MySqlPool) -> BTreeMap<String, (i64, Option<i64>)> {
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(table_name AS CHAR) FROM information_schema.tables \
         WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE'",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    // Témoin que la liste ne passe pas à vide : le schéma de test en porte 45
    // au relevé du 2026-10-27.
    assert!(tables.len() > 40, "tables lues : {}", tables.len());
    // `CHECKSUM TABLE` n'est pas stable sur une table à colonne générée
    // virtuelle (`accounts.singleton_role` : deux mesures sans aucune écriture
    // différaient, relevé du 2026-10-27, C-32-9-45) : la somme porte sur les
    // seules colonnes **stockées**, ligne par ligne (CRC32 combinés par XOR,
    // et le nombre de lignes).
    //
    // ⚠️ Angles morts écrits (revue de code P1, B1, A9) : le XOR ne voit pas
    // l'échange des contenus de deux lignes d'une même table, ni un `|`
    // déplacé d'une colonne à la voisine ; une écriture qui ferait l'un ou
    // l'autre, et rien d'autre, passerait. Le témoin positif de
    // `aucun_outil_n_ecrit` établit qu'un UPDATE ordinaire est vu.
    let mut v = BTreeMap::new();
    for t in tables {
        let colonnes: Vec<String> = sqlx::query_scalar(
            "SELECT CAST(column_name AS CHAR) FROM information_schema.columns \
             WHERE table_schema = DATABASE() AND table_name = ? \
             AND extra NOT LIKE '%GENERATED%' ORDER BY ordinal_position",
        )
        .bind(&t)
        .fetch_all(pool)
        .await
        .unwrap();
        let liste = colonnes
            .iter()
            .map(|c| format!("COALESCE(CAST(`{c}` AS CHAR), '<nul>')"))
            .collect::<Vec<_>>()
            .join(", ");
        let (n, somme): (i64, Option<i64>) = sqlx::query_as(&format!(
            "SELECT COUNT(*), CAST(BIT_XOR(CRC32(CONCAT_WS('|', {liste}))) AS SIGNED) FROM `{t}`"
        ))
        .fetch_one(pool)
        .await
        .unwrap();
        v.insert(t, (n, somme));
    }
    v
}

async fn compte(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aucun_outil_n_ecrit(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let exempte = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    // Témoin positif de la mesure (revue de code P1, B1) : un UPDATE direct
    // d'une colonne stockée — sans changer le nombre de lignes — fait paraître
    // sa table parmi les tables changées. Sans lui, une mesure dégénérée
    // (somme constante) passerait le témoin de stabilité et le test entier.
    let avant = sommes_de_controle(&pool).await;
    sqlx::query("UPDATE accounts SET name = CONCAT(name, '+') WHERE id = ?")
        .bind(a.p.compte_k2)
        .execute(&pool)
        .await
        .unwrap();
    let apres = sommes_de_controle(&pool).await;
    let changees: Vec<&String> = avant.keys().filter(|t| avant[*t] != apres[*t]).collect();
    assert_eq!(changees, ["accounts"], "témoin : un UPDATE est vu (Ma35)");
    assert_eq!(
        avant["accounts"].0, apres["accounts"].0,
        "même nombre de lignes : seule la somme l'a vu"
    );
    for (validation, admises) in [
        (SendValidation::Exempt, vec!["audit_log", "mcp_keys"]),
        (
            SendValidation::Required,
            vec!["audit_log", "mcp_keys", "mcp_sends"],
        ),
    ] {
        for (outil, _, _) in TEMOINS {
            let jeton = if validation == SendValidation::Exempt {
                exempte.clone()
            } else {
                cle_de(&pool, &a, &LECTURE, SendValidation::Required).await
            };
            let avant = sommes_de_controle(&pool).await;
            // Témoin de la mesure : deux relevés sans appel sont égaux.
            assert_eq!(avant, sommes_de_controle(&pool).await, "mesure stable");
            let audit = compte(&pool, "SELECT COUNT(*) FROM audit_log").await;
            let rapports = compte(
                &pool,
                "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'report.%'",
            )
            .await;
            let envois = compte(&pool, "SELECT COUNT(*) FROM mcp_sends").await;
            let r = b.appeler(&jeton, outil, premier_jeu(outil, &a.p)).await;
            assert_eq!(r["result"]["isError"], json!(false), "{outil} : {r}");
            let apres = sommes_de_controle(&pool).await;
            let changees: Vec<&String> = avant.keys().filter(|t| avant[*t] != apres[*t]).collect();
            for t in &changees {
                assert!(admises.contains(&t.as_str()), "{outil} a écrit dans {t}");
            }
            // Témoin positif à chaque appel (B1) : l'appel s'inscrit au journal
            // d'audit, et la mesure le voit.
            assert!(
                changees.iter().any(|t| t.as_str() == "audit_log"),
                "{outil} : audit_log parmi les tables changées ({changees:?})"
            );
            let rapport = i64::from(RAPPORTS.contains(outil));
            assert_eq!(
                compte(&pool, "SELECT COUNT(*) FROM audit_log").await - audit,
                1 + rapport,
                "{outil} : une ligne d'appel, et une de rapport"
            );
            assert_eq!(
                compte(
                    &pool,
                    "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'report.%'"
                )
                .await
                    - rapports,
                rapport,
                "{outil}"
            );
            let attendus = i64::from(validation == SendValidation::Required);
            assert_eq!(
                compte(&pool, "SELECT COUNT(*) FROM mcp_sends").await - envois,
                attendus,
                "{outil}"
            );
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rapport_trace_par_l_outil(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let rapports = "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'report.%'";
    let appels = "SELECT COUNT(*) FROM audit_log WHERE action = 'mcp.tool_called'";
    let (r0, a0) = (compte(&pool, rapports).await, compte(&pool, appels).await);
    let r = banc(&pool)
        .appeler(&jeton, "kesh_bilan", premier_jeu("kesh_bilan", &a.p))
        .await;
    assert_eq!(r["result"]["isError"], json!(false));
    assert_eq!(compte(&pool, rapports).await - r0, 1);
    assert_eq!(compte(&pool, appels).await - a0, 1);
}

// ---------------------------------------------------------------------------
// AC-a20 — la société
// ---------------------------------------------------------------------------

/// Les identifiants de marque `marque` dans tout le corps.
fn marques(corps: &Value, marque: char) -> Vec<String> {
    let t = serde_json::to_string(corps).unwrap();
    motif()
        .find_iter(&t)
        .map(|m| m.as_str().to_string())
        .filter(|id| id.contains(&format!("329{marque}")) || id.contains(&format!("329_{marque}")))
        .collect()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn outils_bornes_a_la_societe(pool: MySqlPool) {
    let (a, b_soc) = deux(&pool).await;
    let b = banc(&pool);
    for (s, autre) in [(&a, 'B'), (&b_soc, 'A')] {
        let jeton = cle_de(&pool, s, &LECTURE, SendValidation::Exempt).await;
        let propre = if autre == 'B' { 'A' } else { 'B' };
        let mut vus = 0;
        for (outil, jeux, _) in TEMOINS {
            for args in jeux(&s.p) {
                let r = b.appeler(&jeton, outil, args.clone()).await;
                assert_eq!(
                    marques(&r, autre),
                    Vec::<String>::new(),
                    "{outil} {args} : la société {autre} sort sous la clé de {propre}"
                );
                vus += marques(&r, propre).len();
            }
        }
        assert!(vus > 20, "témoin : les pièges de {propre} sortent ({vus})");
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn introuvable_indiscernable(pool: MySqlPool) {
    let (a, b_soc) = deux(&pool).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let (debut, fin) = (a.p.exercice_debut.to_string(), a.p.exercice_fin.to_string());
    type Arguments = Box<dyn Fn(i64) -> Value>;
    let cas: Vec<(&str, Arguments, i64)> = vec![
        (
            "kesh_ecriture",
            Box::new(|id| json!({"id": id})),
            b_soc.p.ecriture,
        ),
        (
            "kesh_facture",
            Box::new(|id| json!({"id": id})),
            b_soc.p.facture,
        ),
        (
            "kesh_facture_fournisseur",
            Box::new(|id| json!({"id": id})),
            b_soc.p.facture_fournisseur,
        ),
        (
            "kesh_postes_ouverts",
            Box::new(|id| json!({"accountId": id})),
            b_soc.p.compte_k1,
        ),
        (
            "kesh_grand_livre",
            Box::new(move |id| json!({"accountId": id, "from": debut, "to": fin})),
            b_soc.p.compte_k1,
        ),
        (
            "kesh_factures",
            Box::new(|id| json!({"contactId": id})),
            b_soc.p.client,
        ),
        (
            "kesh_echeancier",
            Box::new(|id| json!({"contactId": id, "paymentStatus": "all"})),
            b_soc.p.client,
        ),
        (
            "kesh_ecritures",
            Box::new(|id| json!({"accountId": id})),
            b_soc.p.compte_k1,
        ),
        // Story 32-9b : le contact d'une autre société = inexistant.
        (
            "kesh_contact",
            Box::new(|id| json!({"id": id})),
            b_soc.p.client,
        ),
    ];
    for (outil, args, de_b) in cas {
        let r_b = b.appeler(&jeton, outil, args(de_b)).await;
        let r_inexistant = b.appeler(&jeton, outil, args(INEXISTANT)).await;
        assert_eq!(
            r_b, r_inexistant,
            "{outil} : un identifiant de B = inexistant"
        );
        assert!(marques(&r_b, 'B').is_empty());
        if [
            "kesh_ecriture",
            "kesh_facture",
            "kesh_facture_fournisseur",
            "kesh_postes_ouverts",
            "kesh_contact",
        ]
        .contains(&outil)
        {
            assert_eq!(
                r_b,
                attendu_outil_en_erreur(1, TEXTE_INTROUVABLE),
                "{outil}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-a22 — les erreurs
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn erreurs_de_demande_comprehensibles(pool: MySqlPool) {
    let (a, b_soc) = deux(&pool).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let exercice = attendu_outil_en_erreur(1, TEXTE_EXERCICE_INTROUVABLE);
    for id in [INEXISTANT, b_soc.p.exercice_id] {
        let r = b
            .appeler(&jeton, "kesh_bilan", json!({"fiscalYearId": id}))
            .await;
        assert_eq!(r, exercice, "exercice {id}");
    }
    let avant = a.p.exercice_debut.pred_opt().unwrap().to_string();
    let r = b
        .appeler(
            &jeton,
            "kesh_balance",
            json!({"fiscalYearId": a.p.exercice_id, "periodStart": avant}),
        )
        .await;
    assert_eq!(r, attendu_outil_en_erreur(1, TEXTE_PERIODE_HORS_EXERCICE));
    let r = b
        .appeler(
            &jeton,
            "kesh_postes_ouverts",
            json!({"accountId": a.p.compte_k2}),
        )
        .await;
    assert_eq!(r, attendu_outil_en_erreur(1, TEXTE_COMPTE_NON_LETTRABLE));
}

// ---------------------------------------------------------------------------
// AC-a21 — les deux politiques
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_dispensee_sert_chaque_outil(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for (outil, _, _) in TEMOINS {
        let r = b.appeler(&jeton, outil, premier_jeu(outil, &a.p)).await;
        // `structure` dérive `s` du texte (v016-1) : l'égalité épingle la forme
        // canonique indentée du texte émis.
        let s = structure(&r, outil);
        assert_eq!(
            texte(&r),
            format!(
                "{TEXTE_ENTETE_DONNEES}\n\n{}",
                serde_json::to_string_pretty(&s).unwrap()
            ),
            "{outil} : forme canonique indentée"
        );
        assert_eq!(
            r["result"].as_object().unwrap().len(),
            2,
            "{outil} : content, isError (v016-1, C-v016-1-1)"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_requise_depose_chaque_outil(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let exempte = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let jwt = session(a.u, Role::Admin, a.c);
    let b = banc(&pool);
    for (outil, _, _) in TEMOINS {
        let requise = cle_de(&pool, &a, &LECTURE, SendValidation::Required).await;
        let args = premier_jeu(outil, &a.p);
        let r = b.mcp(&requise, &appel(1, outil, args.clone())).await;
        let n = numero_depose(&r);
        assert_eq!(r.json(), attendu_depose(1, n), "{outil}");
        let etat: String = sqlx::query_scalar("SELECT state FROM mcp_sends WHERE id = ?")
            .bind(n)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(etat, "pending");
        let detail = envoyer(
            &b.app,
            requete_api(
                "GET",
                &format!("/api/v1/admin/mcp/sends/{n}"),
                Some(&jwt),
                None,
            ),
        )
        .await;
        let version = detail.json()["version"].clone();
        let decision = envoyer(
            &b.app,
            requete_api(
                "POST",
                &format!("/api/v1/admin/mcp/sends/{n}/decision"),
                Some(&jwt),
                Some(json!({"version": version, "decision": "approve"})),
            ),
        )
        .await;
        assert_eq!(decision.statut, StatusCode::OK, "{outil}");
        let servi = texte(&b.appeler(&exempte, outil, args).await);
        let relu = b.mcp(&requise, &lire_envoi(1, n)).await;
        assert_eq!(relu.json(), attendu_valide(1, n, false, &servi), "{outil}");
    }
}

// ---------------------------------------------------------------------------
// AC-a21 — la pagination et les tailles
// ---------------------------------------------------------------------------

/// Où sont les éléments d'une réponse paginée.
fn elements<'a>(outil: &str, s: &'a Value) -> &'a Vec<Value> {
    let v = match outil {
        "kesh_plan_comptable" => &s["comptes"],
        "kesh_ecritures" => &s["ecritures"],
        "kesh_grand_livre" => &s["sections"][0]["lignes"],
        "kesh_balance" | "kesh_balance_agee" => &s["lignes"],
        "kesh_postes_ouverts" => &s["postes"],
        "kesh_factures" | "kesh_echeancier" | "kesh_factures_fournisseur" => &s["factures"],
        "kesh_contacts" => &s["contacts"],
        "kesh_pieces" => &s["pieces"],
        autre => panic!("{autre} : non paginé"),
    };
    v.as_array().unwrap_or_else(|| panic!("{outil} : éléments"))
}

/// Les arguments sans `limit` d'un outil paginé, sur la fixture des tailles.
fn base(outil: &str, s: &Societe) -> Value {
    let p = &s.p;
    match outil {
        "kesh_grand_livre" => json!({"accountId": p.compte_k1,
            "from": p.exercice_debut.to_string(), "to": p.exercice_fin.to_string()}),
        // Les écritures de K1 : celles de la fixture (les achats, plus récents,
        // portent un libellé composé par Kesh, plus court).
        "kesh_postes_ouverts" | "kesh_ecritures" => json!({"accountId": p.compte_k1}),
        "kesh_balance" => json!({"fiscalYearId": p.exercice_id}),
        "kesh_echeancier" => json!({"paymentStatus": "all"}),
        "kesh_pieces" => json!({"statut": "to_complete"}),
        _ => json!({}),
    }
}

/// Un texte de `n` caractères, sans piège.
fn texte_de(n: usize, graine: &str) -> String {
    let mut s = graine.to_string();
    while s.chars().count() < n {
        s.push_str(" lorem");
    }
    s.chars().take(n).collect()
}

/// La fixture des tailles (§ *Les tailles mesurées*) : plus d'objets que le
/// maximum de chaque outil paginé. `large` : chaque colonne libre à sa largeur
/// (validation P2, L5) ; sinon la fixture ordinaire (libellé de 55 caractères,
/// nom de compte de 34, nom de tiers de 22).
async fn fixture_tailles(pool: &MySqlPool, s: &Societe, large: bool) {
    let (libelle, compte, tiers, conditions) = if large {
        (500, 255, 255, 255)
    } else {
        (55, 34, 22, 12)
    };
    let p = &s.p;
    let jour = Utc::now().naive_utc().date();
    // 71 comptes en tête du plan et de la balance (numéros en `00`).
    for i in 0..71 {
        accounts::create(
            pool,
            s.u,
            NewAccount::new(
                s.c,
                format!("00{i:03}"),
                texte_de(compte, &format!("Compte de fixture {i}")),
                AccountType::Asset,
                None,
            ),
        )
        .await
        .unwrap();
    }
    // 31 écritures sur K1 : écritures, grand livre, postes ouverts — chacune à
    // **trois** lignes, comme le prescrit la fiche (§ *Les tailles mesurées* ;
    // revue de code P1, A5) : K2 au débit, K1 et CR au crédit.
    for i in 0..31 {
        journal_entries::create(
            pool,
            p.exercice_id,
            s.u,
            NewJournalEntry {
                company_id: s.c,
                entry_date: jour,
                journal: Journal::OD,
                description: texte_de(libelle, &format!("Écriture de fixture {i}")),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: p.compte_k2,
                        debit: Decimal::new(1234, 2),
                        credit: Decimal::ZERO,
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_k1,
                        debit: Decimal::ZERO,
                        credit: Decimal::new(617, 2),
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_cr,
                        debit: Decimal::ZERO,
                        credit: Decimal::new(617, 2),
                        project_id: None,
                    },
                ],
            },
        )
        .await
        .unwrap();
    }
    // 76 clients débiteurs, une facture validée non réglée chacun ; 71
    // factures fournisseur.
    for i in 0..76 {
        let c = contacts::create(
            pool,
            s.u,
            contact_complet(s.c, texte_de(tiers, &format!("Client {i}")), true, i, large),
        )
        .await
        .unwrap();
        let (f, _) = invoices::create(
            pool,
            s.u,
            NewInvoice {
                company_id: s.c,
                contact_id: c.id,
                date: jour,
                due_date: Some(jour),
                payment_terms: Some(texte_de(conditions, "30 jours net")),
                project_id: None,
                lines: vec![NewInvoiceLine {
                    description: texte_de(libelle, "Prestation"),
                    quantity: Decimal::ONE,
                    unit_price: Decimal::new(12_345, 2),
                    vat_rate: Decimal::ZERO,
                    revenue_account_id: None,
                }],
            },
        )
        .await
        .unwrap();
        invoices::validate_invoice(pool, s.c, f.id, s.u)
            .await
            .unwrap();
    }
    for i in 0..71 {
        let fo = contacts::create(
            pool,
            s.u,
            contact_complet(
                s.c,
                texte_de(tiers, &format!("Fournisseur {i}")),
                false,
                100 + i,
                large,
            ),
        )
        .await
        .unwrap();
        supplier_invoices::create(
            pool,
            NewSupplierInvoice {
                company_id: s.c,
                contact_id: fo.id,
                supplier_invoice_number: Some(texte_de(
                    if large { 64 } else { 8 },
                    &format!("F{i}"),
                )),
                invoice_date: jour,
                due_date: Some(jour),
                creditor_iban: None,
                creditor_qr_iban: None,
                payment_reference: None,
                expected_payment_amount: None,
                project_id: None,
                lines: vec![NewSupplierInvoiceLine {
                    description: texte_de(libelle, "Achat"),
                    quantity: Decimal::ONE,
                    unit_price: Decimal::new(4321, 2),
                    vat_rate: Decimal::ZERO,
                    expense_account_id: p.compte_k2,
                }],
            },
            s.u,
        )
        .await
        .unwrap();
    }
    // Story 32-9b : 66 pièces à compléter, sans fichier ni QR (le nom et le
    // type à leur largeur sous `large`).
    for i in 0..66 {
        imported_supplier_invoices::create(
            pool,
            &NewImportedSupplierInvoice {
                company_id: s.c,
                file_hash: format!("{:0>64}", format!("fixture{i}")),
                storage_path: format!("fixture-329b/{i}.pdf"),
                original_filename: if large {
                    texte_de(255, &format!("piece-{i}"))
                } else {
                    format!("facture-fournisseur-{i:03}.pdf")
                },
                mime_type: if large {
                    texte_de(100, "application/pdf")
                } else {
                    "application/pdf".into()
                },
                byte_size: 48_213 + i,
                creditor_iban: None,
                is_qr_iban: None,
                creditor_address_type: None,
                creditor_name: None,
                creditor_line1: None,
                creditor_line2: None,
                creditor_postal_code: None,
                creditor_town: None,
                creditor_country: None,
                reference_type: None,
                reference_value: None,
                amount: None,
                currency: None,
                unstructured_message: None,
                billing_information: None,
            },
        )
        .await
        .unwrap();
    }
}

/// Un contact de la fixture des tailles, chaque champ rempli (Story 32-9b) —
/// une valeur ordinaire, ou, sous `large`, chaque colonne à sa largeur ; le
/// numéro IDE et le numéro de client uniques par `n`.
fn contact_complet(company_id: i64, nom: String, client: bool, n: i64, large: bool) -> NewContact {
    let (rue, bat, npa, ville, email, tel, cond) = if large {
        (70, 16, 16, 35, 320, 50, 100)
    } else {
        (16, 2, 4, 8, 22, 16, 12)
    };
    NewContact {
        address_street: Some(texte_de(rue, "Rue des Fixtures")),
        address_building: Some(texte_de(bat, "12")),
        address_postal_code: Some(texte_de(npa, "1000")),
        address_city: Some(texte_de(ville, "Lausanne")),
        address_country: Some("CH".into()),
        email: Some(texte_de(email, &format!("contact{n}@exemple.invalid"))),
        phone: Some(texte_de(tel, "+41 21 000 00 00")),
        ide_number: Some(format!("CHE{n:09}")),
        client_number: Some(texte_de(if large { 50 } else { 6 }, &format!("C{n:03}-"))),
        default_payment_terms: Some(texte_de(cond, "30 jours net")),
        default_payment_terms_days: Some(30),
        language: Some(kesh_db::entities::Language::Fr),
        ..contact(company_id, nom, client)
    }
}

fn contact(company_id: i64, nom: String, client: bool) -> NewContact {
    NewContact {
        company_id,
        contact_type: ContactType::Entreprise,
        name: nom,
        first_name: None,
        last_name: None,
        is_client: client,
        is_supplier: !client,
        address: None,
        address_street: None,
        address_building: None,
        address_postal_code: None,
        address_city: None,
        address_country: None,
        email: None,
        phone: None,
        ide_number: None,
        client_number: None,
        default_payment_terms: None,
        default_payment_terms_days: None,
        language: None,
        salutation: Salutation::Neutre,
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pagination_transmise(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    fixture_tailles(&pool, &a, false).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for (outil, _, defaut) in bornes_attendues() {
        let mut args = base(outil, &a);
        args["limit"] = json!(1);
        let s = structure(&b.appeler(&jeton, outil, args).await, outil);
        assert!(elements(outil, &s).len() <= 1, "{outil} limit=1");
        assert_eq!(s["limit"], json!(1), "{outil} : limit rendu");
        let s = structure(&b.appeler(&jeton, outil, base(outil, &a)).await, outil);
        assert_eq!(
            elements(outil, &s).len(),
            usize::try_from(defaut).unwrap(),
            "{outil} : le défaut appliqué"
        );
        assert_eq!(s["limit"], json!(defaut), "{outil} : limit par défaut");
        // `offset` transmis (revue de code P1, E2) : la page d'un élément à
        // `offset = 1` commence où la page de deux à `offset = 0` s'arrête —
        // l'élément entier, solde courant du grand livre compris (#775).
        let mut args = base(outil, &a);
        args["limit"] = json!(2);
        let page1 = structure(&b.appeler(&jeton, outil, args).await, outil);
        let mut args = base(outil, &a);
        args["limit"] = json!(1);
        args["offset"] = json!(1);
        let page2 = structure(&b.appeler(&jeton, outil, args).await, outil);
        assert_eq!(page1["offset"], json!(0), "{outil} : offset rendu (0)");
        assert_eq!(page2["offset"], json!(1), "{outil} : offset rendu (1)");
        let (e1, e2) = (elements(outil, &page1), elements(outil, &page2));
        assert_eq!((e1.len(), e2.len()), (2, 1), "{outil} : tailles des pages");
        assert_ne!(e1[0], e1[1], "{outil} : témoin, deux éléments distincts");
        assert_eq!(
            e2[0], e1[1],
            "{outil} : la page 2 commence où la page 1 s'arrête (offset transmis)"
        );
    }
}

/// Les identifiants des contacts rendus par `kesh_contacts` sous `args`, dans
/// l'ordre.
async fn ids_de_contacts(b: &Banc, jeton: &str, args: Value) -> Vec<i64> {
    let s = structure(
        &b.appeler(jeton, "kesh_contacts", args.clone()).await,
        "kesh_contacts",
    );
    s["contacts"]
        .as_array()
        .unwrap_or_else(|| panic!("{args} : {s}"))
        .iter()
        .map(|c| c["id"].as_i64().unwrap())
        .collect()
}

/// Revue de code P1 de la 32-9b, E2 = B2 — les filtres de `kesh_contacts`
/// atteignent le service et changent le résultat : sur la société piégée, le
/// client C (personne), le fournisseur Fo (entreprise) et un contact archivé
/// X (entreprise, fournisseur). Les dates de création sont posées pour que le
/// tri par date (Fo, C, X) diffère du tri par nom (X, C, Fo).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn filtres_de_kesh_contacts_transmis(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let x = contacts::create(&pool, a.u, contact(a.c, "AAA archivé 329b".into(), false))
        .await
        .unwrap();
    contacts::archive(&pool, x.id, x.version, a.u)
        .await
        .unwrap();
    let (c, fo, x) = (a.p.client, a.p.fournisseur, x.id);
    // Dates de modification posées avec celles de création (une mise à jour
    // séparée les réécrirait) : C, Fo, X — ni l'ordre par nom, ni par création.
    for (id, cree, modifie) in [
        (fo, "2020-01-01", "2023-02-01"),
        (c, "2021-01-01", "2023-01-01"),
        (x, "2022-01-01", "2023-03-01"),
    ] {
        sqlx::query("UPDATE contacts SET created_at = ?, updated_at = ? WHERE id = ?")
            .bind(format!("{cree} 00:00:00"))
            .bind(format!("{modifie} 00:00:00"))
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let jeton = cle_de(&pool, &a, &[McpScope::ContactsRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    // Le numéro IDE de C (`P329A26`, sans `_` : le plein texte du nom ne le
    // découpe pas), retrouvé par le `LIKE` de l'IDE.
    let ide = kesh_seed::pieges::identifiant('A', kesh_seed::pieges::par_cle("client_ide"));
    for (args, attendu) in [
        // Les défauts : actifs seulement, triés par nom.
        (json!({}), vec![c, fo]),
        (json!({"includeArchived": false}), vec![c, fo]),
        (json!({"includeArchived": true}), vec![x, c, fo]),
        (json!({"isClient": true}), vec![c]),
        (json!({"isClient": false}), vec![fo]),
        (json!({"isSupplier": true}), vec![fo]),
        (json!({"isSupplier": false}), vec![c]),
        (json!({"contactType": "Personne"}), vec![c]),
        (json!({"contactType": "Entreprise"}), vec![fo]),
        (
            json!({"contactType": "Entreprise", "includeArchived": true}),
            vec![x, fo],
        ),
        (
            json!({"includeArchived": true, "sortBy": "name"}),
            vec![x, c, fo],
        ),
        (
            json!({"includeArchived": true, "sortBy": "name", "sortDirection": "desc"}),
            vec![fo, c, x],
        ),
        (
            json!({"includeArchived": true, "sortBy": "createdAt"}),
            vec![fo, c, x],
        ),
        (
            json!({"includeArchived": true, "sortBy": "createdAt", "sortDirection": "desc"}),
            vec![x, c, fo],
        ),
        (json!({"search": ide}), vec![c]),
        // Revue de code P2, R3 : le sens seul (tri par défaut, par nom), et le
        // tri par date de modification.
        (json!({"sortDirection": "desc"}), vec![fo, c]),
        (
            json!({"includeArchived": true, "sortDirection": "desc"}),
            vec![fo, c, x],
        ),
        (
            json!({"includeArchived": true, "sortBy": "updatedAt"}),
            vec![c, fo, x],
        ),
        (
            json!({"includeArchived": true, "sortBy": "updatedAt", "sortDirection": "desc"}),
            vec![x, fo, c],
        ),
    ] {
        assert_eq!(
            ids_de_contacts(&b, &jeton, args.clone()).await,
            attendu,
            "kesh_contacts {args} (C {c}, Fo {fo}, X {x})"
        );
    }
    // Revue de code P2, R1 : `actif` est lu de la ligne, non posé en dur —
    // faux pour le contact archivé, vrai pour les deux autres.
    let s = structure(
        &b.appeler(&jeton, "kesh_contacts", json!({"includeArchived": true}))
            .await,
        "kesh_contacts",
    );
    let actifs: Vec<(i64, bool)> = s["contacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| (k["id"].as_i64().unwrap(), k["actif"].as_bool().unwrap()))
        .collect();
    assert_eq!(actifs, vec![(x, false), (c, true), (fo, true)]);
}

/// Revue de code P1 de la 32-9b, B5 — `search` de `kesh_contacts` borné à
/// 100 caractères une fois rognée, comme `kesh_factures` : au schéma
/// (`maxLength`) et à l'outil ; 100 servis, 101 refusés d'un texte fixe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recherche_de_kesh_contacts_bornee(pool: MySqlPool) {
    assert_eq!(
        schema("kesh_contacts")["properties"]["search"]["maxLength"],
        json!(kesh_api::services::factures::RECHERCHE_MAX)
    );
    assert_eq!(
        schema("kesh_factures")["properties"]["search"]["maxLength"],
        schema("kesh_contacts")["properties"]["search"]["maxLength"]
    );
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &[McpScope::ContactsRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let cent = format!("  {}  ", "é".repeat(100));
    let r = b
        .appeler(&jeton, "kesh_contacts", json!({"search": cent}))
        .await;
    structure(&r, "kesh_contacts"); // 100 caractères, rognés : servis

    let r = b
        .appeler(&jeton, "kesh_contacts", json!({"search": "é".repeat(101)}))
        .await;
    assert_eq!(r, attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES));
}

/// Le grand livre d'un compte lu par `kesh_grand_livre` page après page
/// (`offset`) : les lignes et leurs soldes courants égalent ceux d'une lecture
/// d'un seul tenant, et les ruptures d'exercice aussi — sur un compte de bilan
/// (K1) et sur un compte de résultat (K2), qui franchit une fin d'exercice
/// (revue de code P1, E1 ; #775, corrigé dans `kesh-report`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn grand_livre_pagine_egal_a_la_lecture_entiere(pool: MySqlPool) {
    use chrono::{Datelike, NaiveDate};

    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let p = &a.p;
    // L'exercice précédent, ouvert : le solde de K2 y repart de zéro à son
    // franchissement.
    let annee = p.exercice_debut.year() - 1;
    let debut = NaiveDate::from_ymd_opt(annee, 1, 1).unwrap();
    let precedent: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, \
         updated_at) VALUES (?, 'Exercice précédent', ?, ?, 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(a.c)
    .bind(debut)
    .bind(NaiveDate::from_ymd_opt(annee, 12, 31).unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    // Six écritures dans chacun des deux exercices, de montants distincts.
    for i in 0..12_u32 {
        let (exercice, date) = if i < 6 {
            (
                precedent,
                NaiveDate::from_ymd_opt(annee, 2 + i, 10).unwrap(),
            )
        } else {
            (
                p.exercice_id,
                p.exercice_debut + chrono::Days::new(u64::from(i - 6)),
            )
        };
        let montant = Decimal::new(1000 + 137 * i64::from(i), 2);
        journal_entries::create(
            &pool,
            exercice,
            a.u,
            NewJournalEntry {
                company_id: a.c,
                entry_date: date,
                journal: Journal::OD,
                description: format!("Écriture paginée {i}"),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: p.compte_k2,
                        debit: montant,
                        credit: Decimal::ZERO,
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_k1,
                        debit: Decimal::ZERO,
                        credit: montant,
                        project_id: None,
                    },
                ],
            },
        )
        .await
        .unwrap();
    }
    let jeton = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let (du, au) = (debut.to_string(), p.exercice_fin.to_string());
    let lire = |compte: i64, offset: usize, limit: usize| json!({"accountId": compte, "from": du, "to": au, "offset": offset, "limit": limit});
    /// Les lignes d'une section, réduites à (ligne, solde courant).
    fn lignes(section: &Value) -> Vec<(Value, Value)> {
        section["lignes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| (l["ligneId"].clone(), l["soldeCourant"].clone()))
            .collect()
    }
    for (compte, ruptures_attendues) in [(p.compte_k1, 0), (p.compte_k2, 1)] {
        let entiere = structure(
            &b.appeler(&jeton, "kesh_grand_livre", lire(compte, 0, 30))
                .await,
            "entière",
        );
        let section = &entiere["sections"][0];
        let n = usize::try_from(section["nombreLignes"].as_i64().unwrap()).unwrap();
        let attendu = lignes(section);
        assert_eq!(
            attendu.len(),
            n,
            "{compte} : la lecture entière tient en une page"
        );
        assert!(
            n >= 12,
            "{compte} : témoin, au moins les douze écritures ({n})"
        );
        let ruptures = section["ruptures"].as_array().unwrap().clone();
        assert_eq!(
            ruptures.len(),
            ruptures_attendues,
            "{compte} : ruptures d'exercice de la lecture entière"
        );
        for taille in 1..=7 {
            let mut vues = Vec::new();
            let mut ruptures_vues = Vec::new();
            let mut offset = 0;
            while offset < n {
                let page = structure(
                    &b.appeler(&jeton, "kesh_grand_livre", lire(compte, offset, taille))
                        .await,
                    "page",
                );
                let s = &page["sections"][0];
                vues.extend(lignes(s));
                ruptures_vues.extend(s["ruptures"].as_array().unwrap().iter().cloned());
                offset += taille;
            }
            assert_eq!(
                vues, attendu,
                "{compte}, pages de {taille} : (ligne, solde courant) égaux à la lecture entière"
            );
            assert_eq!(
                ruptures_vues, ruptures,
                "{compte}, pages de {taille} : ruptures égales à la lecture entière"
            );
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tailles_mesurees(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    fixture_tailles(&pool, &a, false).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    // Mesure de l'élément : la différence entre une page de deux et d'un.
    for (outil, _, _) in bornes_attendues() {
        let mut tailles = Vec::new();
        for n in [1, 2] {
            let mut args = base(outil, &a);
            args["limit"] = json!(n);
            tailles.push(texte(&b.appeler(&jeton, outil, args).await).len());
        }
        eprintln!("ELEMENT {outil} : {} o", tailles[1] - tailles[0]);
    }
    for (outil, max, defaut) in bornes_attendues() {
        let r = b.appeler(&jeton, outil, base(outil, &a)).await;
        let n_defaut = texte(&r).len();
        let s = structure(&r, outil);
        assert_eq!(elements(outil, &s).len(), usize::try_from(defaut).unwrap());
        let mut args = base(outil, &a);
        args["limit"] = json!(max);
        let r = b.appeler(&jeton, outil, args).await;
        let n_max = texte(&r).len();
        let s = structure(&r, outil);
        assert_eq!(
            elements(outil, &s).len(),
            usize::try_from(max).unwrap(),
            "{outil}"
        );
        eprintln!("TAILLE {outil} : défaut {defaut} → {n_defaut} o ; maximum {max} → {n_max} o");
        assert!(n_defaut <= 13_312, "{outil} : défaut {n_defaut} o");
        assert!(n_max <= 23_552, "{outil} : maximum {n_max} o");
    }
    for outil in ["kesh_bilan", "kesh_compte_de_resultat"] {
        let r = b
            .appeler(&jeton, outil, json!({"fiscalYearId": a.p.exercice_id}))
            .await;
        eprintln!("TAILLE {outil} : {} o", texte(&r).len());
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn plafond_au_maximum(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    fixture_tailles(&pool, &a, true).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for (outil, max, _) in bornes_attendues() {
        let mut args = base(outil, &a);
        args["limit"] = json!(max);
        let r = b.appeler(&jeton, outil, args).await;
        assert_eq!(
            r,
            attendu_outil_en_erreur(1, TEXTE_ENVOI_TROP_VOLUMINEUX),
            "{outil} : refus pour taille"
        );
    }
    // Témoin : sous la même fixture, une page d'un élément est servie.
    let mut args = base("kesh_ecritures", &a);
    args["limit"] = json!(1);
    let r = b.appeler(&jeton, "kesh_ecritures", args).await;
    assert_eq!(r["result"]["isError"], json!(false));
}

// ---------------------------------------------------------------------------
// Story v016-1 (AC-7, #812) — une seule forme de montant
// ---------------------------------------------------------------------------

/// Sur un exercice **sans écriture**, chaque montant de `kesh_bilan` — les
/// totaux, le report, le résultat et le solde de chaque ligne — sort sous la
/// même chaîne `"0.00"` (C-v016-1-6). Avant la 0.16.1, un montant lu en base
/// gardait l'échelle de sa colonne (`"0.0000"`) et une somme calculée la sienne
/// (`"0"`), dans le même objet (#812). Le JSON est lu dans le texte, après
/// l'en-tête et la ligne vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bilan_montants_une_seule_forme(pool: MySqlPool) {
    let c = kesh_seed::pieges::societe_complete(&pool, "Société vide v0161")
        .await
        .expect("société");
    let u = common::mcp::administrateur(&pool, c, "admin-v0161").await;
    common::mcp::activer(&pool, c).await;
    let exercice: i64 = sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ?")
        .bind(c)
        .fetch_one(&pool)
        .await
        .expect("l'exercice de la société");
    let ecritures: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .bind(c)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ecritures, 0, "montage : un exercice sans écriture");
    let (_, jeton) = common::mcp::cle(&pool, c, u, &LECTURE, SendValidation::Exempt).await;
    let r = banc(&pool)
        .appeler(&jeton, "kesh_bilan", json!({"fiscalYearId": exercice}))
        .await;
    let t = texte(&r);
    let (_, json_du_texte) = t.split_once("\n\n").expect("en-tête, ligne vide, JSON");
    let s: Value = serde_json::from_str(json_du_texte).expect("le JSON du texte");
    let mut montants = Vec::new();
    for champ in [
        "totalActifs",
        "totalPassifs",
        "totalFondsPropres",
        "report",
        "resultat",
    ] {
        montants.push((champ.to_string(), s[champ].clone()));
    }
    for section in ["actifs", "passifs", "fondsPropres"] {
        for (i, l) in s[section].as_array().expect(section).iter().enumerate() {
            montants.push((format!("{section}[{i}].solde"), l["solde"].clone()));
        }
    }
    let fautifs: Vec<_> = montants
        .iter()
        .filter(|(_, v)| *v != json!("0.00"))
        .collect();
    assert!(
        fautifs.is_empty(),
        "#812 : chaque montant du bilan vide vaut \"0.00\" : {fautifs:?} — {s}"
    );
}
