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
    // balance âgée révisée sur la mesure (C-32-9-44 : 40 / 75 → 38 / 68), les factures
    // fournisseur aussi, qui portent le numéro interne (C-e331-27 : 70 / 40 → 62 / 34).
    let mesurees: BTreeMap<&str, (i64, i64)> = [
        // Story 37-1 (AC-8) : le champ `devise` re-mesuré, 70 / 40 → 68 / 38.
        ("kesh_plan_comptable", (68, 38)),
        // Story 37-3a (AC-8) : lignes en devise re-mesurées, 22 / 12 → 18 / 10.
        ("kesh_ecritures", (18, 10)),
        ("kesh_grand_livre", (26, 14)),
        // Story 37-4 (AC-6) : champs en devise re-mesurés (`tailles_mesurees_en_devise`),
        // grand livre 30 / 15 → 26 / 14 (27 donnait 23 614 o dans la fixture des tailles),
        // balance 55 / 30 → 42 / 23. Story 46-6a (AC-6) : le schéma garde ceux du
        // mode plat ; le mode groupé (613 o la ligne, socle de 756 o) a 37 / 20,
        // dits par la description et appliqués selon `detailLevel`.
        ("kesh_balance", (42, 23)),
        ("kesh_postes_ouverts", (24, 12)),
        ("kesh_factures", (40, 20)),
        ("kesh_echeancier", (40, 20)),
        ("kesh_balance_agee", (68, 38)),
        ("kesh_factures_fournisseur", (62, 34)),
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
    // `kesh_balance` : le maximum avec `detailLevel` (mode groupé) est porté
    // par `dependentSchemas`, et vaut la constante du code.
    assert_eq!(
        schema("kesh_balance")["dependentSchemas"]["detailLevel"]["properties"]["limit"]["maximum"],
        json!(outils::rapports::LIMITE_MAX_MCP_BALANCE_GROUPEE)
    );
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
    // Story 35-5b : le jour de l'horloge de test, celui de l'exercice des pièges.
    let jour = common::jour::aujourd_hui_test();
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
    // Story 37-3a (AC-8) — le pire cas des écritures compte des lignes en
    // devise : société et exercice multi-devises, un compte en USD et sa
    // contrepartie CHF, hors des comptes que mesurent les autres outils.
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.c)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(p.exercice_id)
        .execute(pool)
        .await
        .unwrap();
    let compte_usd = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '09901', ?, 'Asset', 'USD')",
    )
    .bind(s.c)
    .bind(texte_de(compte, "Compte en devise 09901"))
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    // 31 écritures sur K1 : écritures, grand livre, postes ouverts — chacune à
    // **trois** lignes, comme le prescrit la fiche (§ *Les tailles mesurées* ;
    // revue de code P1, A5) : une ligne au débit, K1 et CR au crédit. Story
    // 37-3a : la ligne au débit est **en devise**, aux décimaux les plus larges
    // (montant à 4 décimales, cours à 10), sur un compte USD (elle était sur K2,
    // que le pire cas n'exige pas).
    for i in 0..31 {
        journal_entries::create(
            pool,
            p.exercice_id,
            kesh_db::entities::audit_log::Acteur::utilisateur(s.u),
            NewJournalEntry {
                company_id: s.c,
                entry_date: jour,
                journal: Journal::OD,
                description: texte_de(libelle, &format!("Écriture de fixture {i}")),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: compte_usd,
                        debit: Decimal::new(1234, 2),
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: Some(kesh_db::entities::ForeignAmount {
                            declared_currency: None,
                            amount: Decimal::new(12_345_678, 4),
                            rate: Decimal::new(99_956_789, 10),
                            source: Some(kesh_db::entities::RateProvenance::Manual),
                            rate_derived: false,
                        }),
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_k1,
                        debit: Decimal::ZERO,
                        credit: Decimal::new(617, 2),
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_cr,
                        debit: Decimal::ZERO,
                        credit: Decimal::new(617, 2),
                        project_id: None,
                        foreign: None,
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
                    line_type: kesh_db::entities::LineType::Priced,
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
        supplier_invoices::create_validated(
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
            kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
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
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: p.compte_k1,
                        debit: Decimal::ZERO,
                        credit: montant,
                        project_id: None,
                        foreign: None,
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
            // Story 37-4 : le maximum de l'outil (30 → re-mesuré), non une valeur en dur.
            &b.appeler(
                &jeton,
                "kesh_grand_livre",
                lire(
                    compte,
                    0,
                    usize::try_from(outils::grand_livre::LIMITE_MAX_MCP).unwrap(),
                ),
            )
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
/// même chaîne `"0.00"` (C-v016-1-6). Avant la 0.17.0, un montant lu en base
/// gardait l'échelle de sa colonne (`"0.0000"`) et une somme calculée la sienne
/// (`"0"`), dans le même objet (#812). Le JSON est lu dans le texte, après
/// l'en-tête et la ligne vide. Un bilan vide ne listant aucune ligne, une
/// écriture de 10.0000 est ensuite passée, et le solde de chaque ligne lue —
/// une au moins — doit sortir `"10.00"` ou `"-10.00"` (revue de code P1, B2),
/// comme les totaux des actifs et des passifs (revue de code P2, B4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bilan_montants_une_seule_forme(pool: MySqlPool) {
    let c = kesh_seed::pieges::societe_complete(
        &pool,
        "Société vide v0161",
        common::jour::aujourd_hui_test(),
    )
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

    // Un bilan sans écriture ne liste aucune ligne : la boucle des sections
    // ci-dessus y passe à vide (revue de code P1, B2). Une écriture de 10.0000
    // — l'échelle de la colonne `DECIMAL(19,4)` — d'un actif à un passif fait
    // paraître des lignes ; au moins une doit être lue, et chaque solde sortir
    // sous la même forme, `"10.00"` ou `"-10.00"`.
    let comptes = accounts::list_by_company(&pool, c, false).await.unwrap();
    let premier = |t: AccountType| {
        comptes
            .iter()
            .find(|a| a.active && a.postable && a.account_type == t)
            .map(|a| a.id)
            .expect("un compte postable du type")
    };
    let (actif, passif) = (premier(AccountType::Asset), premier(AccountType::Liability));
    let debut: chrono::NaiveDate =
        sqlx::query_scalar("SELECT start_date FROM fiscal_years WHERE id = ?")
            .bind(exercice)
            .fetch_one(&pool)
            .await
            .unwrap();
    let dix = Decimal::new(100_000, 4);
    journal_entries::create(
        &pool,
        exercice,
        kesh_db::entities::audit_log::Acteur::utilisateur(u),
        NewJournalEntry {
            company_id: c,
            entry_date: debut,
            journal: Journal::OD,
            description: "Écriture v0161".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: actif,
                    debit: dix,
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: passif,
                    debit: Decimal::ZERO,
                    credit: dix,
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .expect("l'écriture de 10.0000");
    let r = banc(&pool)
        .appeler(&jeton, "kesh_bilan", json!({"fiscalYearId": exercice}))
        .await;
    let t = texte(&r);
    let (_, json_du_texte) = t.split_once("\n\n").expect("en-tête, ligne vide, JSON");
    let s: Value = serde_json::from_str(json_du_texte).expect("le JSON du texte");
    let mut soldes = Vec::new();
    for section in ["actifs", "passifs", "fondsPropres"] {
        for (i, l) in s[section].as_array().expect(section).iter().enumerate() {
            soldes.push((format!("{section}[{i}].solde"), l["solde"].clone()));
        }
    }
    assert!(
        !soldes.is_empty(),
        "montage : l'écriture fait paraître au moins une ligne — {s}"
    );
    let fautifs: Vec<_> = soldes
        .iter()
        .filter(|(_, v)| *v != json!("10.00") && *v != json!("-10.00"))
        .collect();
    assert!(
        fautifs.is_empty(),
        "#812 : chaque solde de ligne sort \"10.00\" ou \"-10.00\" : {fautifs:?} — {s}"
    );
    // Les totaux sont des sommes calculées — le lieu même où #812 rendait `"0"`
    // à côté de `"0.0000"` — et ne s'éprouvent qu'ici à une valeur non nulle,
    // la seule où la forme d'une somme se distingue (revue de code P2, B4).
    assert_eq!(
        s["totalActifs"],
        json!("10.00"),
        "#812 : le total des actifs, somme calculée — {s}"
    );
    assert!(
        s["totalPassifs"] == json!("10.00") || s["totalPassifs"] == json!("-10.00"),
        "#812 : le total des passifs, somme calculée, \"10.00\" ou \"-10.00\" — {s}"
    );
}

// ---------------------------------------------------------------------------
// Story v017-4a (#849, AC-17) — un brouillon lu par les outils
// ---------------------------------------------------------------------------

/// AC-17 (revue P1, A-2) — la facture fournisseur des pièges, **dévalidée**,
/// redevient un brouillon : la fiche (`kesh_facture_fournisseur`) rend
/// `statut: "draft"` et `ecritureAchatId` **présent et nul** — non omis —, et la
/// liste (`kesh_factures_fournisseur`, sans filtre de statut) la rend aussi,
/// `statut: "draft"`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_brouillon_lu_par_les_outils(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let id = a.p.facture_fournisseur;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);

    // Avant : validée, l'écriture d'achat est nommée.
    let avant = structure(
        &b.appeler(&jeton, "kesh_facture_fournisseur", json!({"id": id}))
            .await,
        "kesh_facture_fournisseur (validée)",
    );
    assert_eq!(avant["statut"], json!("open"), "{avant}");
    assert!(avant["ecritureAchatId"].is_i64(), "{avant}");

    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    supplier_invoices::unvalidate(
        &pool,
        a.c,
        id,
        version,
        kesh_db::entities::Acteur::utilisateur(a.u),
        crate::common::jour::aujourd_hui_test(),
    )
    .await
    .expect("dévalidation");

    let fiche = structure(
        &b.appeler(&jeton, "kesh_facture_fournisseur", json!({"id": id}))
            .await,
        "kesh_facture_fournisseur (brouillon)",
    );
    assert_eq!(fiche["statut"], json!("draft"), "{fiche}");
    assert_eq!(
        fiche.get("ecritureAchatId"),
        Some(&Value::Null),
        "ecritureAchatId présent et nul : {fiche}"
    );

    let liste = structure(
        &b.appeler(&jeton, "kesh_factures_fournisseur", json!({}))
            .await,
        "kesh_factures_fournisseur",
    );
    let ligne = elements("kesh_factures_fournisseur", &liste)
        .iter()
        .find(|f| f["id"] == json!(id))
        .unwrap_or_else(|| panic!("le brouillon est listé : {liste}"));
    assert_eq!(ligne["statut"], json!("draft"), "{ligne}");

    // Story 33-1a (#926, AC18) — le numéro interne s'ajoute à `numero`, à la fiche et à
    // la liste, et survit au retour en brouillon ; `numero` garde celui du fournisseur.
    use kesh_seed::pieges;
    let interne = pieges::texte_seme('A', pieges::par_cle("ff_numero_interne"));
    assert_eq!(
        avant["numeroInterne"]["nonFiable"],
        json!(interne),
        "{avant}"
    );
    assert_eq!(fiche["numeroInterne"], avant["numeroInterne"], "{fiche}");
    assert_eq!(ligne["numeroInterne"], avant["numeroInterne"], "{ligne}");
    assert_eq!(
        avant["numero"]["nonFiable"],
        json!(pieges::texte_seme('A', pieges::par_cle("ff_numero"))),
        "`numero` reste le numéro du fournisseur : {avant}"
    );
}

/// Story 37-3a (AC-8) — `kesh_ecriture` et `kesh_ecritures` rendent la partie en
/// devise d'une ligne : la devise **du compte**, le montant, le cours et la
/// provenance ; sur une ligne CHF, les quatre champs sont présents à `null`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_ligne_en_devise_dans_kesh_ecriture(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(a.c)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(a.p.exercice_id)
        .execute(&pool)
        .await
        .unwrap();
    let usd = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '09911', 'Banque USD', 'Asset', 'USD')",
    )
    .bind(a.c)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let je = journal_entries::create(
        &pool,
        a.p.exercice_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
        NewJournalEntry {
            company_id: a.c,
            entry_date: common::jour::aujourd_hui_test(),
            journal: Journal::OD,
            description: "Encaissement USD".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: usd,
                    debit: Decimal::new(912, 2),
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: Some(kesh_db::entities::ForeignAmount {
                        declared_currency: None,
                        amount: Decimal::new(10, 0),
                        rate: Decimal::new(912, 3),
                        source: Some(kesh_db::entities::RateProvenance::Manual),
                        rate_derived: false,
                    }),
                },
                NewJournalEntryLine {
                    account_id: a.p.compte_k2,
                    debit: Decimal::ZERO,
                    credit: Decimal::new(912, 2),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .unwrap();
    let ecriture = structure(
        &b.appeler(&jeton, "kesh_ecriture", json!({"id": je.entry.id}))
            .await,
        "kesh_ecriture",
    );
    let lignes = &ecriture["ecriture"]["lignes"];
    assert_eq!(lignes[0]["devise"], "USD", "{ecriture}");
    assert_eq!(lignes[0]["montantDevise"], "10.00", "{ecriture}");
    assert_eq!(lignes[0]["cours"], "0.912", "{ecriture}");
    assert_eq!(lignes[0]["sourceCours"], "manual", "{ecriture}");
    for champ in ["devise", "montantDevise", "cours", "sourceCours"] {
        assert!(
            lignes[1].get(champ).is_some_and(Value::is_null),
            "{champ} présent à null sur une ligne CHF : {ecriture}"
        );
    }
    let liste = structure(
        &b.appeler(&jeton, "kesh_ecritures", json!({"accountId": usd}))
            .await,
        "kesh_ecritures",
    );
    assert_eq!(
        liste["ecritures"][0]["lignes"][0]["devise"], "USD",
        "{liste}"
    );
}

/// Story 37-5 (AC-5, V12) — une écriture de réévaluation se lit par
/// `kesh_ecriture` : la ligne du compte en devise rend sa devise (celle du
/// compte), un montant en devise nul et la source `revaluation` ; la
/// contrepartie est une ligne CHF.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_ecriture_de_reevaluation_dans_kesh_ecriture(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(a.c)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(a.p.exercice_id)
        .execute(&pool)
        .await
        .unwrap();
    let eur = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '09912', 'Banque EUR', 'Asset', 'EUR')",
    )
    .bind(a.c)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let je = journal_entries::create(
        &pool,
        a.p.exercice_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
        NewJournalEntry {
            company_id: a.c,
            entry_date: common::jour::aujourd_hui_test(),
            journal: Journal::OD,
            description: "Gain de change Banque EUR EUR 0.94947 (non réalisé)".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: eur,
                    debit: Decimal::new(1234, 2),
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: Some(kesh_db::entities::ForeignAmount {
                        declared_currency: None,
                        amount: Decimal::ZERO,
                        rate: Decimal::new(94947, 5),
                        source: Some(kesh_db::entities::RateProvenance::Revaluation),
                        rate_derived: false,
                    }),
                },
                NewJournalEntryLine {
                    account_id: a.p.compte_k2,
                    debit: Decimal::ZERO,
                    credit: Decimal::new(1234, 2),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .unwrap();
    let ecriture = structure(
        &b.appeler(&jeton, "kesh_ecriture", json!({"id": je.entry.id}))
            .await,
        "kesh_ecriture",
    );
    let lignes = &ecriture["ecriture"]["lignes"];
    assert_eq!(lignes[0]["devise"], "EUR", "{ecriture}");
    assert_eq!(lignes[0]["montantDevise"], "0.00", "{ecriture}");
    assert_eq!(lignes[0]["sourceCours"], "revaluation", "{ecriture}");
    assert!(lignes[1]["devise"].is_null(), "{ecriture}");
}

/// Story 33-1b (#926, AC8 ; revue P1, A-F2) — `kesh_ecriture` et `kesh_postes_ouverts`
/// disent le numéro interne de la facture fournisseur à côté de celui du fournisseur,
/// chacun sous son nom : le test nomme le champ, là où l'inventaire de
/// `mcp_enveloppe_e2e` ne voit que la sortie de l'outil (M18, M19 de la 33-1b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_numero_interne_d_une_ecriture_et_d_un_poste(pool: MySqlPool) {
    use kesh_seed::pieges;
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let interne = json!(pieges::texte_seme(
        'A',
        pieges::par_cle("ff_numero_interne")
    ));
    let fournisseur = json!(pieges::texte_seme('A', pieges::par_cle("ff_numero")));

    let ecriture = structure(
        &b.appeler(&jeton, "kesh_ecriture", json!({"id": a.p.ecriture_achat}))
            .await,
        "kesh_ecriture",
    );
    let source = &ecriture["pieceSource"];
    assert_eq!(
        source["numeroInterneFactureFournisseur"]["nonFiable"], interne,
        "{ecriture}"
    );
    assert_eq!(
        source["numeroFactureFournisseur"]["nonFiable"], fournisseur,
        "{ecriture}"
    );

    let postes = structure(
        &b.appeler(
            &jeton,
            "kesh_postes_ouverts",
            json!({"accountId": a.p.compte_cr}),
        )
        .await,
        "kesh_postes_ouverts",
    );
    let piece = elements("kesh_postes_ouverts", &postes)
        .iter()
        .map(|p| &p["piece"])
        .find(|p| p["id"] == json!(a.p.facture_fournisseur))
        .unwrap_or_else(|| panic!("le poste de la facture : {postes}"));
    assert_eq!(piece["numeroInterne"]["nonFiable"], interne, "{piece}");
    assert_eq!(piece["numero"]["nonFiable"], fournisseur, "{piece}");
}

// ---------------------------------------------------------------------------
// Story 33-7, AC-7 (#776) — la trace d'un rapport lu par un agent
// ---------------------------------------------------------------------------

/// AC-7 — chacun des cinq outils de rapport inscrit `channel: "mcp"` et le numéro
/// de **sa** clé, au nom du créateur de la clé, `actor_type = user`. Le numéro de
/// la clé diffère de celui de l'utilisateur (assertion de montage : sans quoi
/// `mcp_key_id = user_id` passerait à vide).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_cinq_rapports_disent_le_canal_et_la_cle(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 337", 'A').await;
    // Des clés leurres : le numéro de la clé servie s'écarte de celui de l'utilisateur.
    for _ in 0..3 {
        common::mcp::cle(&pool, a.c, a.u, &LECTURE, SendValidation::Exempt).await;
    }
    let (cle_id, jeton) = common::mcp::cle(&pool, a.c, a.u, &LECTURE, SendValidation::Exempt).await;
    assert_ne!(cle_id, a.u, "montage : numéro de clé ≠ utilisateur");
    let b = banc(&pool);

    let mut fautes = Vec::new();
    for outil in [
        "kesh_grand_livre",
        "kesh_balance",
        "kesh_bilan",
        "kesh_compte_de_resultat",
        "kesh_balance_agee",
    ] {
        let repere: Option<i64> = sqlx::query_scalar("SELECT MAX(id) FROM audit_log")
            .fetch_one(&pool)
            .await
            .unwrap();
        let args = premier_jeu(outil, &a.p);
        let r = b.appeler(&jeton, outil, args).await;
        assert_eq!(r["result"]["isError"], json!(false), "{outil} : {r}");
        let lignes: Vec<(i64, String, Option<i64>, Vec<u8>)> = sqlx::query_as(
            "SELECT user_id, actor_type, actor_api_key_id, details_json FROM audit_log \
             WHERE id > ? AND action = 'report.generated' ORDER BY id",
        )
        .bind(repere.unwrap_or(0))
        .fetch_all(&pool)
        .await
        .unwrap();
        let [(user_id, actor_type, cle_api, details)] = lignes.as_slice() else {
            fautes.push(format!("{outil} : {} trace(s) de rapport", lignes.len()));
            continue;
        };
        let details: Value = serde_json::from_slice(details).unwrap();
        if details["channel"] != json!("mcp") || details["mcp_key_id"] != json!(cle_id) {
            fautes.push(format!("{outil} : détail {details}"));
        }
        if (*user_id, actor_type.as_str(), *cle_api) != (a.u, "user", None) {
            fautes.push(format!(
                "{outil} : auteur {user_id}, {actor_type}, {cle_api:?}"
            ));
        }
    }
    assert!(fautes.is_empty(), "AC-7 :\n{}", fautes.join("\n"));
}

// ---------------------------------------------------------------------------
// Story 35-5b (AC5, #929) — le serveur MCP juge au jour de Zurich
// ---------------------------------------------------------------------------

fn instant(s: &str) -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::parse_from_rfc3339(s)
        .unwrap()
        .with_timezone(&chrono::Utc)
}

fn date(a: i32, m: u32, j: u32) -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(a, m, j).unwrap()
}

/// Le JSON d'une réponse d'outil : après l'en-tête et la ligne vide.
fn donnees_de(r: &Value) -> Value {
    let t = texte(r);
    let (_, json_du_texte) = t.split_once("\n\n").expect("en-tête, ligne vide, JSON");
    serde_json::from_str(json_du_texte).expect("le JSON du texte")
}

/// Une société complète **sans pièges** (D8 : `semer` pose une facture échue au
/// jour du montage, qui fausserait le résumé), servie, son exercice de l'année
/// d'`aujourd_hui`, et une clé de lecture : `(société, administrateur, jeton)`.
async fn societe_servie(
    pool: &MySqlPool,
    nom: &str,
    aujourd_hui: chrono::NaiveDate,
) -> (i64, i64, String) {
    let c = kesh_seed::pieges::societe_complete(pool, nom, aujourd_hui)
        .await
        .expect("société");
    let u = common::mcp::administrateur(pool, c, &format!("admin-{c}")).await;
    common::mcp::activer(pool, c).await;
    let (_, jeton) = common::mcp::cle(pool, c, u, &LECTURE, SendValidation::Exempt).await;
    (c, u, jeton)
}

/// AC5 — banc à `2025-12-31T23:30:00Z` (00 h 30 le 1er janvier à Zurich) :
/// `kesh_balance_agee` est arrêtée au `2026-01-01`, et `kesh_postes_ouverts`
/// sans `asOf` lit au `2026-01-01` (le jour UTC rendait le 31 décembre).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac5_le_mcp_lit_le_jour_suisse_la_nuit_du_nouvel_an(pool: MySqlPool) {
    let (c, _, jeton) = societe_servie(&pool, "Société nuit 355b", date(2026, 1, 1)).await;
    let debiteurs: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '1100'")
            .bind(c)
            .fetch_one(&pool)
            .await
            .unwrap();
    let banc = banc_avec_horloge(&pool, instant("2025-12-31T23:30:00Z"));
    let agee = donnees_de(&banc.appeler(&jeton, "kesh_balance_agee", json!({})).await);
    let postes = donnees_de(
        &banc
            .appeler(
                &jeton,
                "kesh_postes_ouverts",
                json!({ "accountId": debiteurs }),
            )
            .await,
    );
    assert_eq!(
        (agee["au"].clone(), postes["au"].clone()),
        (json!("2026-01-01"), json!("2026-01-01")),
        "(balance âgée, postes ouverts) : {agee} — {postes}"
    );
}

/// AC5 — une facture échue le `2026-03-10` : sur un banc à
/// `2026-03-10T23:30:00Z` (00 h 30 le 11 à Zurich), `kesh_echeancier` filtré
/// sur les échues la rend et compte `echuesNombre = 1` ; à `22:30:00Z` (23 h 30
/// le 10), aucune, `echuesNombre = 0`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac5_l_echeancier_du_mcp_juge_au_jour_suisse(pool: MySqlPool) {
    let (c, u, jeton) = societe_servie(&pool, "Société échéances 355b", date(2026, 3, 10)).await;
    let client = contacts::create(&pool, u, contact(c, "Client 355b".into(), true))
        .await
        .unwrap();
    let (f, _) = invoices::create(
        &pool,
        u,
        NewInvoice {
            company_id: c,
            contact_id: client.id,
            date: date(2026, 3, 1),
            due_date: Some(date(2026, 3, 10)),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: Decimal::ONE,
                unit_price: Decimal::new(10_000, 2),
                vat_rate: Decimal::ZERO,
                revenue_account_id: None,
                line_type: kesh_db::entities::LineType::Priced,
            }],
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(&pool, c, f.id, u).await.unwrap();

    let mut vues = Vec::new();
    for maintenant in ["2026-03-10T23:30:00Z", "2026-03-10T22:30:00Z"] {
        let e = donnees_de(
            &banc_avec_horloge(&pool, instant(maintenant))
                .appeler(
                    &jeton,
                    "kesh_echeancier",
                    json!({ "paymentStatus": "overdue" }),
                )
                .await,
        );
        let ids: Vec<i64> = e["factures"]
            .as_array()
            .unwrap_or_else(|| panic!("{e}"))
            .iter()
            .map(|x| x["id"].as_i64().unwrap())
            .collect();
        vues.push((ids, e["resume"]["echuesNombre"].clone()));
    }
    assert_eq!(
        vues,
        vec![(vec![f.id], json!(1)), (vec![], json!(0))],
        "(00 h 30 le 11 mars, 23 h 30 le 10 mars)"
    );
}

// ---------------------------------------------------------------------------
// Story 37-1 (AC-2 V10, AC-8) — le mode et les genres des exercices, la devise
// d'un compte
// ---------------------------------------------------------------------------

/// Pose un exercice ouvert en SQL direct, aux bornes de l'année `annee`.
async fn exercice_brut(pool: &MySqlPool, c: i64, nom: &str, annee: i32) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, created_at, \
         updated_at) VALUES (?, ?, ?, ?, 'Open', NOW(3), NOW(3)) RETURNING id",
    )
    .bind(c)
    .bind(nom)
    .bind(date(annee, 1, 1))
    .bind(date(annee, 12, 31))
    .fetch_one(pool)
    .await
    .unwrap()
}

/// AC-2 (V10) — `kesh_exercices` rend `multiDevises`, `genreOuverture` et
/// `genreCloture` pour chaque exercice : un exercice multi-devises au genre
/// `month`/`day` et un exercice mono-devise (`day`/`day`), présents en
/// mono-devise aussi. Le mode est posé en SQL direct (l'activation vient en
/// phase suivante).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn kesh_exercices_rend_le_mode_et_les_genres(pool: MySqlPool) {
    use chrono::Datelike;
    let a = societe_piegee(&pool, "Société A 371", 'A').await;
    sqlx::query(
        "UPDATE fiscal_years SET multi_currency = TRUE, opening_rate_kind = 'month' WHERE id = ?",
    )
    .bind(a.p.exercice_id)
    .execute(&pool)
    .await
    .unwrap();
    let mono = exercice_brut(&pool, a.c, "Exercice mono", a.p.exercice_debut.year() - 1).await;
    let jeton = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let s = structure(
        &banc(&pool)
            .appeler(&jeton, "kesh_exercices", json!({}))
            .await,
        "kesh_exercices",
    );
    let par_id = |id: i64| {
        s["exercices"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .cloned()
            .unwrap_or_else(|| panic!("exercice {id} absent : {s}"))
    };
    let multi = par_id(a.p.exercice_id);
    assert_eq!(multi["multiDevises"], json!(true));
    assert_eq!(multi["genreOuverture"], json!("month"));
    assert_eq!(multi["genreCloture"], json!("day"));
    let m = par_id(mono);
    assert_eq!(m["multiDevises"], json!(false));
    assert_eq!(m["genreOuverture"], json!("day"));
    assert_eq!(m["genreCloture"], json!("day"));
}

/// AC-2 (V10) — **mesure de taille** de `kesh_exercices`, non paginé (aucune
/// `limit`, son `SansArgument` refuse tout argument) : la différence de texte
/// entre deux réponses qui diffèrent d'un exercice au nom de 50 caractères (la
/// longueur maximale de la colonne), trois champs neufs compris, donne la
/// taille d'un exercice ; le nombre d'exercices qui tient sous
/// `MCP_ENVOI_OCTETS_MAX` s'en déduit (écrit au Change Log). Au-delà,
/// `TropVolumineux`, sans recours : état existant, aggravé de trois champs.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn kesh_exercices_taille_mesuree(pool: MySqlPool) {
    use chrono::Datelike;
    use kesh_api::services::envois_mcp::MCP_ENVOI_OCTETS_MAX;
    let a = societe_piegee(&pool, "Société A 371", 'A').await;
    let jeton = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let mut annee = a.p.exercice_debut.year() - 1;
    let nom_long = |annee: i32| format!("Exercice {annee} {}", "x".repeat(36));
    assert_eq!(nom_long(annee).chars().count(), 50, "nom de 50 caractères");
    exercice_brut(&pool, a.c, &nom_long(annee), annee).await;
    let lire = || async {
        let r = b.appeler(&jeton, "kesh_exercices", json!({})).await;
        structure(&r, "kesh_exercices");
        texte(&r).len()
    };
    let t1 = lire().await;
    annee -= 1;
    exercice_brut(&pool, a.c, &nom_long(annee), annee).await;
    let t2 = lire().await;
    let element = t2 - t1;
    let compte: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fiscal_years WHERE company_id = ?")
        .bind(a.c)
        .fetch_one(&pool)
        .await
        .unwrap();
    // Les exercices présents, plus autant d'exercices à nom de 50 caractères
    // que la marge en admet.
    let tient = usize::try_from(compte).unwrap() + (MCP_ENVOI_OCTETS_MAX - t2) / element;
    eprintln!(
        "TAILLE kesh_exercices : un exercice (nom de 50 caractères) {element} o ; \
         {compte} exercices → {t2} o ; {tient} exercices sous {MCP_ENVOI_OCTETS_MAX} o"
    );
    assert!(
        (150..=400).contains(&element),
        "taille d'un exercice hors de l'ordre attendu : {element} o"
    );
    assert!(
        tient >= 60,
        "au moins soixante exercices doivent tenir : {tient}"
    );
}

/// AC-8 — `kesh_plan_comptable` donne la devise d'un compte : le code ISO 4217
/// d'un compte en devise (posé en SQL direct), `null` pour un compte CHF —
/// clé toujours présente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn kesh_plan_comptable_rend_la_devise(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 371", 'A').await;
    let comptes = accounts::list_by_company(&pool, a.c, false).await.unwrap();
    let usd = comptes
        .iter()
        .find(|c| c.account_type == AccountType::Asset && c.role.is_none() && c.postable)
        .expect("un compte d'actif sans rôle")
        .id;
    let chf = comptes
        .iter()
        .find(|c| c.id != usd)
        .expect("un second compte")
        .id;
    sqlx::query("UPDATE accounts SET currency = 'USD' WHERE id = ?")
        .bind(usd)
        .execute(&pool)
        .await
        .unwrap();
    let jeton = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let mut vus = BTreeMap::new();
    let mut offset = 0;
    loop {
        let s = structure(
            &b.appeler(
                &jeton,
                "kesh_plan_comptable",
                json!({"limit": outils::plan_comptable::LIMITE_MAX_MCP, "offset": offset}),
            )
            .await,
            "kesh_plan_comptable",
        );
        let page = s["comptes"].as_array().unwrap().clone();
        for c in &page {
            assert!(
                c.get("devise").is_some(),
                "clé devise toujours présente : {c}"
            );
            vus.insert(c["id"].as_i64().unwrap(), c["devise"].clone());
        }
        offset += usize::try_from(outils::plan_comptable::LIMITE_MAX_MCP).unwrap();
        if offset >= usize::try_from(s["total"].as_i64().unwrap()).unwrap() {
            break;
        }
    }
    assert_eq!(vus[&usd], json!("USD"));
    assert_eq!(vus[&chf], Value::Null);
}

// ---------------------------------------------------------------------------
// Story 37-4 (AC-6, AC-7, CR-6) — les rapports en devise par le MCP
// ---------------------------------------------------------------------------

/// La société A passée multi-devises (société et exercices, en SQL), un compte
/// d'actif USD lettrable et une écriture : USD 100 au cours 0.91 = CHF 91.00,
/// contre K2. Rend l'identifiant du compte USD.
async fn compte_usd_mouvemente(pool: &MySqlPool, a: &Societe) -> i64 {
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(a.c)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE company_id = ?")
        .bind(a.c)
        .execute(pool)
        .await
        .unwrap();
    let usd: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '1105', 'Débiteurs USD', 'Asset', 'USD') RETURNING id",
    )
    .bind(a.c)
    .fetch_one(pool)
    .await
    .unwrap();
    journal_entries::create(
        pool,
        a.p.exercice_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
        NewJournalEntry {
            company_id: a.c,
            entry_date: common::jour::aujourd_hui_test(),
            journal: Journal::OD,
            description: "Créance en USD".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: usd,
                    debit: Decimal::new(9100, 2),
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: Some(kesh_db::entities::ForeignAmount {
                        declared_currency: None,
                        amount: Decimal::new(100, 0),
                        rate: Decimal::new(91, 2),
                        source: Some(kesh_db::entities::RateProvenance::Manual),
                        rate_derived: false,
                    }),
                },
                NewJournalEntryLine {
                    account_id: a.p.compte_k2,
                    debit: Decimal::ZERO,
                    credit: Decimal::new(9100, 2),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .expect("écriture en USD");
    usd
}

/// AC-6 — `kesh_grand_livre`, `kesh_balance` et `kesh_bilan` portent les champs en
/// devise d'un compte USD, et les mêmes clés à `null` sur un compte CHF (la macro
/// `sortie!` n'omet aucun champ, C-37-9).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rapports_en_devise_par_le_mcp(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 374", 'A').await;
    let usd = compte_usd_mouvemente(&pool, &a).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let p = &a.p;
    let periode = |id: i64| json!({"accountId": id, "from": p.exercice_debut.to_string(), "to": p.exercice_fin.to_string()});

    let gl = structure(
        &b.appeler(&jeton, "kesh_grand_livre", periode(usd)).await,
        "kesh_grand_livre",
    );
    let s = &gl["sections"][0];
    assert_eq!(s["devise"], json!("USD"), "{s}");
    assert_eq!(s["ouvertureDevise"], json!("0.00"), "{s}");
    assert_eq!(s["clotureDevise"], json!("100.00"), "{s}");
    assert_eq!(s["totalDebitDevise"], json!("100.00"), "{s}");
    let l = &s["lignes"][0];
    assert_eq!(
        (
            &l["debitDevise"],
            &l["creditDevise"],
            &l["cours"],
            &l["sourceCours"],
            &l["soldeCourantDevise"]
        ),
        (
            &json!("100.00"),
            &json!("0.00"),
            &json!("0.91"),
            &json!("manual"),
            &json!("100.00")
        ),
        "{l}"
    );
    let chf = structure(
        &b.appeler(&jeton, "kesh_grand_livre", periode(p.compte_k2))
            .await,
        "kesh_grand_livre",
    );
    let s = &chf["sections"][0];
    for k in [
        "devise",
        "ouvertureDevise",
        "totalDebitDevise",
        "totalCreditDevise",
        "clotureDevise",
    ] {
        assert_eq!(s.get(k), Some(&Value::Null), "{k} présent à null : {s}");
    }
    for k in [
        "debitDevise",
        "creditDevise",
        "cours",
        "sourceCours",
        "soldeCourantDevise",
    ] {
        assert_eq!(s["lignes"][0].get(k), Some(&Value::Null), "{k} : {s}");
    }

    let balance = structure(
        &b.appeler(
            &jeton,
            "kesh_balance",
            json!({"fiscalYearId": p.exercice_id,
                   "limit": outils::rapports::LIMITE_MAX_MCP_BALANCE}),
        )
        .await,
        "kesh_balance",
    );
    let lignes = balance["lignes"].as_array().unwrap();
    let u = lignes
        .iter()
        .find(|l| l["compteId"] == json!(usd))
        .expect("ligne USD");
    assert_eq!(
        (&u["devise"], &u["debitDevise"], &u["clotureDevise"]),
        (&json!("USD"), &json!("100.00"), &json!("100.00")),
        "{u}"
    );
    let k2 = lignes
        .iter()
        .find(|l| l["compteId"] == json!(p.compte_k2))
        .expect("ligne K2");
    assert_eq!(k2.get("devise"), Some(&Value::Null), "{k2}");
    assert_eq!(k2.get("clotureDevise"), Some(&Value::Null), "{k2}");

    let bilan = structure(
        &b.appeler(&jeton, "kesh_bilan", json!({"fiscalYearId": p.exercice_id}))
            .await,
        "kesh_bilan",
    );
    let actif = bilan["actifs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["compteId"] == json!(usd))
        .expect("compte USD au bilan");
    assert_eq!(
        (&actif["devise"], &actif["soldeDevise"]),
        (&json!("USD"), &json!("100.00"))
    );
    for l in bilan["actifs"].as_array().unwrap() {
        assert!(l.get("soldeDevise").is_some(), "clé présente : {l}");
    }

    // `kesh_compte_de_resultat` (revue de code P1) : un compte de produits USD
    // porte sa devise et son solde en devise ; les autres, les clés à `null`.
    let ventes_usd: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, '3205', 'Ventes USD', 'Revenue', 'USD') RETURNING id",
    )
    .bind(a.c)
    .fetch_one(&pool)
    .await
    .unwrap();
    let usd_50 = || {
        Some(kesh_db::entities::ForeignAmount {
            declared_currency: None,
            amount: Decimal::new(50, 0),
            rate: Decimal::new(91, 2),
            source: Some(kesh_db::entities::RateProvenance::Manual),
            rate_derived: false,
        })
    };
    journal_entries::create(
        &pool,
        p.exercice_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
        NewJournalEntry {
            company_id: a.c,
            entry_date: common::jour::aujourd_hui_test(),
            journal: Journal::Ventes,
            description: "Vente en USD".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: usd,
                    debit: Decimal::new(4550, 2),
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: usd_50(),
                },
                NewJournalEntryLine {
                    account_id: ventes_usd,
                    debit: Decimal::ZERO,
                    credit: Decimal::new(4550, 2),
                    project_id: None,
                    foreign: usd_50(),
                },
            ],
        },
    )
    .await
    .expect("vente en USD");
    let resultat = structure(
        &b.appeler(
            &jeton,
            "kesh_compte_de_resultat",
            json!({"fiscalYearId": p.exercice_id}),
        )
        .await,
        "kesh_compte_de_resultat",
    );
    let produits = resultat["produits"].as_array().unwrap();
    let v = produits
        .iter()
        .find(|l| l["compteId"] == json!(ventes_usd))
        .expect("produits USD au compte de résultat");
    assert_eq!(
        (&v["devise"], &v["solde"], &v["soldeDevise"]),
        (&json!("USD"), &json!("45.50"), &json!("50.00")),
        "{v}"
    );
    for l in produits
        .iter()
        .chain(resultat["charges"].as_array().unwrap())
        .filter(|l| l["compteId"] != json!(ventes_usd))
    {
        assert_eq!(l.get("devise"), Some(&Value::Null), "{l}");
        assert_eq!(l.get("soldeDevise"), Some(&Value::Null), "{l}");
    }
}

/// CR-6 (V13, AC-7) — `kesh_postes_ouverts` sur un compte de débiteurs USD
/// lettrable rend `solde`, `totalOuvert` et les débits et crédits des postes au
/// CHF de leurs lignes, sans champ en devise ; `resteDu` à `null` (aucune
/// facture en devise).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn postes_ouverts_d_un_compte_en_devise_en_chf(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 374b", 'A').await;
    let usd = compte_usd_mouvemente(&pool, &a).await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let s = structure(
        &banc(&pool)
            .appeler(&jeton, "kesh_postes_ouverts", json!({"accountId": usd}))
            .await,
        "kesh_postes_ouverts",
    );
    assert_eq!(
        (&s["solde"], &s["totalOuvert"]),
        (&json!("91.00"), &json!("91.00")),
        "{s}"
    );
    let postes = elements("kesh_postes_ouverts", &s);
    assert_eq!(postes.len(), 1, "{s}");
    assert_eq!(
        (&postes[0]["debit"], &postes[0]["credit"]),
        (&json!("91.00"), &json!("0.00"))
    );
    assert_eq!(postes[0]["resteDu"], Value::Null);
    let cles = |v: &Value| v.as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    for k in cles(&s).into_iter().chain(cles(&postes[0])) {
        assert!(
            !k.contains("evise") && k != "cours",
            "aucun champ en devise : {k}"
        );
    }
}

/// Story 37-4 (AC-6) — les tailles **re-mesurées avec les champs en devise
/// remplis** (le pire cas : chaque élément en devise) : grand livre d'un compte
/// USD de 31 lignes, balance de 56 comptes USD mouvementés, bilan de 57 comptes
/// USD à solde non nul (les 56, plus le 1105 de `compte_usd_mouvemente`). Mêmes largeurs que la fixture des tailles (nom de compte
/// de 34 caractères, libellé de 55). Les bornes de `bornes_attendues` ne changent
/// que si ces mesures dépassent les plafonds du test des tailles.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tailles_mesurees_en_devise(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 374t", 'A').await;
    compte_usd_mouvemente(&pool, &a).await;
    let jour = common::jour::aujourd_hui_test();
    let p = &a.p;
    let ligne_usd = |account_id: i64| NewJournalEntryLine {
        account_id,
        debit: Decimal::new(123_455, 2),
        credit: Decimal::ZERO,
        project_id: None,
        foreign: Some(kesh_db::entities::ForeignAmount {
            declared_currency: None,
            amount: Decimal::new(1_355_789, 3),
            rate: Decimal::new(9_105_786_123, 10),
            source: Some(kesh_db::entities::RateProvenance::Manual),
            rate_derived: false,
        }),
    };
    let ecrire = |account_id: i64, i: usize| {
        let pool = pool.clone();
        let lines = vec![
            ligne_usd(account_id),
            NewJournalEntryLine {
                account_id: p.compte_k2,
                debit: Decimal::ZERO,
                credit: Decimal::new(123_455, 2),
                project_id: None,
                foreign: None,
            },
        ];
        async move {
            journal_entries::create(
                &pool,
                p.exercice_id,
                kesh_db::entities::audit_log::Acteur::utilisateur(a.u),
                NewJournalEntry {
                    company_id: a.c,
                    entry_date: jour,
                    journal: Journal::OD,
                    description: texte_de(55, &format!("Écriture en devise {i}")),
                    project_id: None,
                    lines,
                },
            )
            .await
            .expect("écriture en devise");
        }
    };
    // 56 comptes USD en tête de la balance et du bilan (numéros en `00`).
    for i in 0..56 {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type, currency) \
             VALUES (?, ?, ?, 'Asset', 'USD') RETURNING id",
        )
        .bind(a.c)
        .bind(format!("00{i:03}"))
        .bind(texte_de(34, &format!("Compte en devise {i}")))
        .fetch_one(&pool)
        .await
        .unwrap();
        ecrire(id, i).await;
        if i == 0 {
            // Le compte du grand livre : 31 lignes en devise.
            for k in 1..31 {
                ecrire(id, 100 + k).await;
            }
        }
    }
    let gl_compte: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '00000'")
            .bind(a.c)
            .fetch_one(&pool)
            .await
            .unwrap();
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let args = |outil: &str, limit: i64| match outil {
        "kesh_grand_livre" => json!({"accountId": gl_compte,
            "from": p.exercice_debut.to_string(), "to": p.exercice_fin.to_string(),
            "limit": limit}),
        _ => json!({"fiscalYearId": p.exercice_id, "limit": limit}),
    };
    let outils_pagines = [
        (
            "kesh_grand_livre",
            outils::grand_livre::LIMITE_MAX_MCP,
            outils::grand_livre::LIMITE_DEFAUT,
        ),
        (
            "kesh_balance",
            outils::rapports::LIMITE_MAX_MCP_BALANCE,
            outils::rapports::LIMITE_DEFAUT_BALANCE,
        ),
    ];
    // La mesure d'abord, pour chaque outil (page d'un et de deux éléments),
    // avant toute assertion : elle fixe les bornes.
    for (outil, _, _) in outils_pagines {
        let mesure: Vec<usize> = {
            let mut m = Vec::new();
            for n in [1, 2] {
                m.push(texte(&b.appeler(&jeton, outil, args(outil, n)).await).len());
            }
            m
        };
        let element = mesure[1] - mesure[0];
        let socle = mesure[0] - element;
        eprintln!(
            "MESURE EN DEVISE {outil} : élément {element} o, socle {socle} o ; bornes \
             possibles : défaut {} / maximum {}",
            (13_312 - socle) / element,
            (23_552 - socle) / element
        );
    }
    for (outil, max, defaut) in outils_pagines {
        let mut tailles = Vec::new();
        for n in [1, 2, defaut, max] {
            let r = b.appeler(&jeton, outil, args(outil, n)).await;
            let s = structure(&r, outil);
            assert_eq!(
                elements(outil, &s).len(),
                usize::try_from(n).unwrap(),
                "{outil}"
            );
            assert!(
                elements(outil, &s)[0]
                    .as_object()
                    .unwrap()
                    .iter()
                    .any(|(k, v)| k.ends_with("Devise") && !v.is_null()),
                "{outil} : éléments en devise remplis"
            );
            tailles.push(texte(&r).len());
        }
        eprintln!(
            "TAILLE EN DEVISE {outil} : élément {} o ; défaut {defaut} → {} o ; maximum {max} → {} o",
            tailles[1] - tailles[0],
            tailles[2],
            tailles[3]
        );
        assert!(tailles[2] <= 13_312, "{outil} : défaut {} o", tailles[2]);
        assert!(tailles[3] <= 23_552, "{outil} : maximum {} o", tailles[3]);
    }
    // Story 46-6a (AC-6) — la balance **groupée**, mêmes bornes : les 56
    // comptes USD rangés sous un groupe `00` au nom de 34 caractères, tous les
    // niveaux, sous-totaux en pied ; la tranche commence par le groupe (en
    // USD : toute sa descendance l'est) et ses comptes, colonnes en devise
    // remplies. Une ligne groupée pèse plus que la plate (613 o contre 542 o) :
    // le mode groupé a ses propres bornes, la sortie à plat garde les siennes.
    let groupe: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, postable) \
         VALUES (?, '00', ?, 'Asset', FALSE) RETURNING id",
    )
    .bind(a.c)
    .bind(texte_de(34, "Groupe en devise"))
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE accounts SET parent_id = ? WHERE company_id = ? AND number LIKE '00___'")
        .bind(groupe)
        .bind(a.c)
        .execute(&pool)
        .await
        .unwrap();
    let groupee = |limit: i64| {
        json!({"fiscalYearId": p.exercice_id, "limit": limit, "detailLevel": "all",
               "subtotalPosition": "footer"})
    };
    let mesure: Vec<usize> = {
        let mut m = Vec::new();
        for n in [1, 2] {
            m.push(texte(&b.appeler(&jeton, "kesh_balance", groupee(n)).await).len());
        }
        m
    };
    let (element, socle) = (mesure[1] - mesure[0], mesure[0] - (mesure[1] - mesure[0]));
    eprintln!(
        "MESURE EN DEVISE kesh_balance groupée : élément {element} o, socle {socle} o ; bornes \
         possibles : défaut {} / maximum {}",
        (13_312 - socle) / element,
        (23_552 - socle) / element
    );
    let mut tailles = Vec::new();
    for n in [
        1,
        2,
        outils::rapports::LIMITE_DEFAUT_BALANCE_GROUPEE,
        outils::rapports::LIMITE_MAX_MCP_BALANCE_GROUPEE,
    ] {
        let r = b.appeler(&jeton, "kesh_balance", groupee(n)).await;
        let s = structure(&r, "kesh_balance groupée");
        let lignes = s["groupes"]["lignes"].as_array().unwrap();
        assert_eq!(lignes.len(), usize::try_from(n).unwrap());
        assert_eq!(lignes[0]["compteId"], json!(groupe), "{}", lignes[0]);
        assert_eq!(lignes[0]["devise"], json!("USD"), "{}", lignes[0]);
        for l in lignes {
            assert!(!l["clotureDevise"].is_null(), "colonnes en devise : {l}");
        }
        tailles.push(texte(&r).len());
    }
    eprintln!(
        "TAILLE EN DEVISE kesh_balance groupée : élément {} o ; défaut {} o ; maximum {} o",
        tailles[1] - tailles[0],
        tailles[2],
        tailles[3]
    );
    assert!(
        tailles[2] <= 13_312,
        "balance groupée : défaut {} o",
        tailles[2]
    );
    assert!(
        tailles[3] <= 23_552,
        "balance groupée : maximum {} o",
        tailles[3]
    );
    // Un écho chargé et un sous-total : le groupe ouvert à la main (écho
    // `ouverts` non vide) et la dernière tranche pleine, qui porte le
    // sous-total en pied du groupe (genre `subtotal`).
    let max_g = outils::rapports::LIMITE_MAX_MCP_BALANCE_GROUPEE;
    let total_groupe = structure(
        &b.appeler(&jeton, "kesh_balance", groupee(1)).await,
        "kesh_balance groupée",
    )["total"]
        .as_i64()
        .unwrap();
    let r = b
        .appeler(
            &jeton,
            "kesh_balance",
            json!({"fiscalYearId": p.exercice_id, "limit": max_g, "offset": total_groupe - max_g,
                   "detailLevel": "all", "subtotalPosition": "footer", "open": [groupe]}),
        )
        .await;
    let s = structure(&r, "kesh_balance groupée avec écho");
    assert_eq!(s["groupes"]["ouverts"], json!([groupe]), "écho chargé");
    assert!(
        s["groupes"]["lignes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["genre"] == json!("subtotal")),
        "un sous-total dans la tranche : {s}"
    );
    eprintln!(
        "TAILLE EN DEVISE kesh_balance groupée, écho et sous-total : {} o",
        texte(&r).len()
    );
    assert!(
        texte(&r).len() <= 23_552,
        "balance groupée avec écho : {} o",
        texte(&r).len()
    );
    let r = b
        .appeler(&jeton, "kesh_bilan", json!({"fiscalYearId": p.exercice_id}))
        .await;
    assert_eq!(
        r["result"]["isError"],
        json!(false),
        "57 comptes en devise : servi"
    );
    eprintln!(
        "TAILLE EN DEVISE kesh_bilan (57 comptes en devise) : {} o",
        texte(&r).len()
    );
}

// ---------------------------------------------------------------------------
// Story 49-1a (#1242) — T11 : la sortie `kesh_facture` porte `typeLigne` sur
// chaque ligne (AC13 b, D14)
// ---------------------------------------------------------------------------

/// Une facture brouillon de `lignes` (créée par le dépôt, qui accepte les trois
/// types) : rend son id.
async fn facture_typee(
    pool: &MySqlPool,
    c: i64,
    u: i64,
    client: i64,
    lignes: &[kesh_db::entities::LineType],
) -> i64 {
    use kesh_db::entities::LineType;
    let lines = lignes
        .iter()
        .map(|lt| {
            let priced = *lt == LineType::Priced;
            NewInvoiceLine {
                description: if priced { "Prestation" } else { "Note" }.into(),
                quantity: if priced { Decimal::ONE } else { Decimal::ZERO },
                unit_price: if priced {
                    Decimal::new(10_000, 2)
                } else {
                    Decimal::ZERO
                },
                vat_rate: Decimal::ZERO,
                revenue_account_id: None,
                line_type: *lt,
            }
        })
        .collect();
    invoices::create(
        pool,
        u,
        NewInvoice {
            company_id: c,
            contact_id: client,
            date: date(2026, 3, 1),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines,
        },
    )
    .await
    .unwrap()
    .0
    .id
}

/// T11 — la **clé elle-même** `typeLigne` est présente sur chaque ligne (jamais
/// `is_null()` : l'indexation d'un `Value` rend `Null` pour une clé absente),
/// avec la valeur du type ; puis la mesure de V6-2 : la taille d'une ligne, ce
/// que la clé y ajoute, et le nombre maximal de lignes qu'une facture peut
/// porter sous le plafond d'un envoi, avec et sans la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn kesh_facture_porte_le_type_de_chaque_ligne(pool: MySqlPool) {
    use kesh_api::services::envois_mcp::MCP_ENVOI_OCTETS_MAX;
    use kesh_db::entities::LineType;

    let (c, u, jeton) = societe_servie(&pool, "Société types 49-1a", date(2026, 3, 10)).await;
    let client = contacts::create(&pool, u, contact(c, "Client 49-1a".into(), true))
        .await
        .unwrap()
        .id;
    let b = banc(&pool);
    let types = [LineType::Priced, LineType::Comment, LineType::Heading];
    let f = facture_typee(&pool, c, u, client, &types).await;
    let fiche = donnees_de(&b.appeler(&jeton, "kesh_facture", json!({ "id": f })).await);
    let lignes = fiche["lignes"]
        .as_array()
        .unwrap_or_else(|| panic!("{fiche}"));
    assert_eq!(lignes.len(), 3, "{fiche}");
    for (l, attendu) in lignes.iter().zip(["priced", "comment", "heading"]) {
        let objet = l.as_object().unwrap();
        assert!(objet.contains_key("typeLigne"), "clé absente : {l}");
        assert_eq!(objet["typeLigne"], json!(attendu), "{l}");
    }

    // Mesure (V6-2) : une facture d'une, puis de deux lignes chiffrées.
    let mut tailles = Vec::new();
    let mut texte_deux = String::new();
    for n in [1, 2] {
        let f = facture_typee(&pool, c, u, client, &vec![LineType::Priced; n]).await;
        let t = texte(&b.appeler(&jeton, "kesh_facture", json!({ "id": f })).await);
        tailles.push(t.len());
        texte_deux = t;
    }
    let par_ligne = tailles[1] - tailles[0];
    let socle = tailles[0] - par_ligne;
    // La clé en JSON joli : sa ligne, son saut de ligne, et la virgule qu'elle
    // ajoute au champ qui la précède.
    let ligne_cle = texte_deux
        .lines()
        .find(|l| l.contains("\"typeLigne\""))
        .expect("typeLigne dans le texte");
    let cle = ligne_cle.len() + 2;
    let max_avec = (MCP_ENVOI_OCTETS_MAX - socle) / par_ligne;
    let max_sans = (MCP_ENVOI_OCTETS_MAX - socle) / (par_ligne - cle);
    eprintln!(
        "MESURE kesh_facture : socle {socle} o, ligne {par_ligne} o dont typeLigne {cle} o ; \
         lignes max sous {MCP_ENVOI_OCTETS_MAX} o : {max_sans} sans la clé, {max_avec} avec"
    );
    assert!(cle > 0 && cle < par_ligne);
    assert!(
        max_avec >= 40,
        "une facture courante doit rester servie : {max_avec} lignes au plus"
    );

    // Revue P1 (A1) : la plus grosse facture des données semées. `seed_demo`
    // n'en sème aucune ; la plus lourde est celle des pièges (`pieges::semer`,
    // une ligne à la description de 1 000 caractères).
    let p = societe_piegee(&pool, "Société pièges 49-1a", 'A').await;
    let jeton_p = cle_de(&pool, &p, &LECTURE, SendValidation::Exempt).await;
    let t = texte(
        &b.appeler(&jeton_p, "kesh_facture", json!({ "id": p.p.facture }))
            .await,
    );
    let cle_p: usize = t
        .lines()
        .filter(|l| l.contains("\"typeLigne\""))
        .map(|l| l.len() + 2)
        .sum();
    eprintln!(
        "MESURE kesh_facture (facture des pièges, la plus lourde semée) : {} o avec la clé, {} o sans",
        t.len(),
        t.len() - cle_p
    );
    assert!(t.len() < MCP_ENVOI_OCTETS_MAX);
}

// ---------------------------------------------------------------------------
// Story 46-4 — le bilan et le compte de résultat par groupes
// ---------------------------------------------------------------------------

/// Le littéral de base d'un outil (AC-2), écrit au commit de base de la story,
/// **avant** les arguments de groupement : la sortie sans argument neuf, sur la
/// société d'essai, les dates de la période (qui suivent l'horloge de test,
/// `KESH_TEST_INSTANT`) et l'exercice remplacés par des jetons (`{debut}`,
/// `{fin}`, `{exercice}`). Les identifiants de compte sont ceux d'une base
/// éphémère neuve, stables d'un passage à l'autre.
fn litteral_de_base(outil: &str, p: &kesh_seed::pieges::Pieges) -> String {
    let chemin = format!(
        "{}/tests/fixtures/mcp-groupes/{outil}.txt",
        env!("CARGO_MANIFEST_DIR")
    );
    let brut = std::fs::read_to_string(&chemin).unwrap_or_else(|e| panic!("{chemin} : {e}"));
    brut.replace("{debut}", &p.exercice_debut.to_string())
        .replace("{fin}", &p.exercice_fin.to_string())
        .replace("{exercice}", &p.exercice_id.to_string())
}

/// AC-2 — sans argument neuf, le texte rendu égale, octet pour octet, le
/// littéral du commit de base, **au seul champ `groupes: null` près**
/// (C-46-4-5) : le littéral reste tel qu'écrit avant les arguments de
/// groupement ; le test en analyse la partie JSON, y ajoute `groupes` à `null`
/// au niveau supérieur, la re-sérialise par le chemin de la sortie
/// (`to_value` puis `to_string_pretty`) et compare le texte entier, en-tête
/// compris. `kesh_balance` (Story 46-6a, AC-6, C-46-6a-5) : même règle, sur
/// le littéral écrit au commit de base de la 46-6a.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sans_groupes_sortie_du_commit_de_base(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for outil in ["kesh_balance", "kesh_bilan", "kesh_compte_de_resultat"] {
        // `kesh_balance` comprise : sans `limit` ni argument de groupement,
        // la limite par défaut à plat est celle de T0 (23), inchangée.
        let args = json!({"fiscalYearId": a.p.exercice_id});
        let r = b.appeler(&jeton, outil, args).await;
        let rendu = texte(&r);
        let base = litteral_de_base(outil, &a.p);
        let (entete, json_de_base) = base
            .split_once("\n\n")
            .unwrap_or_else(|| panic!("{outil} : en-tête, ligne vide, JSON"));
        assert_eq!(
            entete, TEXTE_ENTETE_DONNEES,
            "{outil} : en-tête du littéral"
        );
        let mut valeur: Value = serde_json::from_str(json_de_base).unwrap();
        valeur
            .as_object_mut()
            .unwrap()
            .insert("groupes".into(), Value::Null);
        let valeur = serde_json::to_value(&valeur).unwrap();
        let attendu = format!(
            "{entete}\n\n{}",
            serde_json::to_string_pretty(&valeur).unwrap()
        );
        assert_eq!(rendu, attendu, "{outil} : texte rendu\n{rendu}");
    }
}

/// L'identifiant d'un compte de la société, par son numéro.
async fn compte_numero(pool: &MySqlPool, s: &Societe, numero: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
        .bind(s.c)
        .bind(numero)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("compte {numero} : {e}"))
}

/// Les clés d'une ligne groupée (AC-3) : ni `typeDeCompte`, ni `actif`, ni
/// `role`.
const CLES_LIGNE_GROUPEE: [&str; 9] = [
    "compteId",
    "devise",
    "genre",
    "nom",
    "numero",
    "ouvert",
    "profondeur",
    "solde",
    "soldeDevise",
];

/// Les sections d'un outil : (listes plates, sections groupées).
fn sections_de(outil: &str) -> &'static [&'static str] {
    match outil {
        "kesh_bilan" => &["actifs", "passifs", "fondsPropres"],
        _ => &["produits", "charges"],
    }
}

/// AC-3, AC-6 — avec `detailLevel`, la sortie porte `groupes` (écho et lignes
/// groupées), les listes plates sont vides, les totaux inchangés ; sans, le
/// témoin : `groupes` à `null` et les listes plates remplies.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupes_par_le_mcp(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464", 'A').await;
    let actifs = compte_numero(&pool, &a, "1").await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    for outil in ["kesh_bilan", "kesh_compte_de_resultat"] {
        let plat = structure(
            &b.appeler(&jeton, outil, json!({"fiscalYearId": fy})).await,
            outil,
        );
        assert_eq!(plat["groupes"], Value::Null, "{outil} : témoin hors groupe");
        assert!(
            sections_de(outil)
                .iter()
                .any(|s| !plat[s].as_array().unwrap().is_empty()),
            "{outil} : témoin, listes plates remplies : {plat}"
        );

        // Niveau 1, sous-totaux sur le titre : les racines, fermées.
        let g = structure(
            &b.appeler(&jeton, outil, json!({"fiscalYearId": fy, "detailLevel": 1}))
                .await,
            outil,
        );
        let groupes = &g["groupes"];
        assert_eq!(groupes["niveau"], json!(1), "{outil} : {groupes}");
        assert_eq!(groupes["tousLesNiveaux"], json!(false));
        assert_eq!(groupes["positionSousTotaux"], json!("header"));
        assert!(groupes["profondeurMax"].as_u64().unwrap() >= 2, "{groupes}");
        assert_eq!(groupes["ouverts"], json!([]));
        assert_eq!(groupes["fermes"], json!([]));
        let mut lignes = 0;
        for s in sections_de(outil) {
            assert_eq!(g[s], json!([]), "{outil} : liste plate {s} vide");
            for l in groupes[s].as_array().unwrap() {
                lignes += 1;
                let cles: Vec<&str> = l.as_object().unwrap().keys().map(String::as_str).collect();
                assert_eq!(cles, CLES_LIGNE_GROUPEE, "{outil} {s} : {l}");
                assert_eq!(l["profondeur"], json!(1), "{outil} {s} : niveau 1 : {l}");
                match l["genre"].as_str().unwrap() {
                    "group" => assert_eq!(l["ouvert"], json!(false), "{l}"),
                    "account" => assert_eq!(l["ouvert"], Value::Null, "{l}"),
                    autre => panic!("{outil} : genre {autre} au niveau 1 : {l}"),
                }
            }
        }
        assert!(lignes > 0, "{outil} : lignes groupées : {g}");
        // Les totaux et le résultat : ceux du rapport à plat.
        for k in plat.as_object().unwrap().keys() {
            if !sections_de(outil).contains(&k.as_str()) && k != "groupes" {
                assert_eq!(g[k], plat[k], "{outil} : {k} inchangé");
            }
        }

        // Tous les niveaux, sous-totaux en pied ; un groupe ouvert à la main,
        // un identifiant inconnu, nul et négatif ignorés.
        let g = structure(
            &b.appeler(
                &jeton,
                outil,
                json!({"fiscalYearId": fy, "detailLevel": "all", "subtotalPosition": "footer",
                       "open": [actifs], "closed": [INEXISTANT, 0, -5]}),
            )
            .await,
            outil,
        );
        let groupes = &g["groupes"];
        assert_eq!(groupes["niveau"], Value::Null, "{groupes}");
        assert_eq!(groupes["tousLesNiveaux"], json!(true));
        assert_eq!(groupes["positionSousTotaux"], json!("footer"));
        assert_eq!(groupes["fermes"], json!([]), "inconnus retirés de l'écho");
        let ouverts = if outil == "kesh_bilan" {
            json!([actifs])
        } else {
            json!([])
        };
        assert_eq!(groupes["ouverts"], ouverts, "{outil} : {groupes}");
        let genres: BTreeSet<&str> = sections_de(outil)
            .iter()
            .flat_map(|s| groupes[s].as_array().unwrap())
            .map(|l| l["genre"].as_str().unwrap())
            .collect();
        assert!(genres.contains("subtotal"), "{outil} : {genres:?}");
        assert!(genres.contains("account"), "{outil} : {genres:?}");
    }
}

/// AC-3 (revue de code P1, M1) — la fidélité des lignes groupées : chaque ligne
/// de compte porte les valeurs de la ligne plate de même `compteId` (numéro,
/// nom, solde, devise, solde en devise), dont un compte en USD ; un groupe
/// ouvert à la main porte `ouvert` à `true`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupes_lignes_fideles_au_plat(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464f", 'A').await;
    let usd = compte_usd_mouvemente(&pool, &a).await;
    let actifs = compte_numero(&pool, &a, "1").await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    let mut usd_compare = false;
    for outil in ["kesh_bilan", "kesh_compte_de_resultat"] {
        let plat = structure(
            &b.appeler(&jeton, outil, json!({"fiscalYearId": fy})).await,
            outil,
        );
        let g = structure(
            &b.appeler(
                &jeton,
                outil,
                json!({"fiscalYearId": fy, "detailLevel": "all"}),
            )
            .await,
            outil,
        );
        let mut comptes = 0;
        for s in sections_de(outil) {
            for l in g["groupes"][s].as_array().unwrap() {
                if l["genre"] != json!("account") {
                    continue;
                }
                let f = plat[s]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|f| f["compteId"] == l["compteId"])
                    .unwrap_or_else(|| panic!("{outil} {s} : pas de ligne plate pour {l}"));
                for k in ["numero", "nom", "solde", "devise", "soldeDevise"] {
                    assert_eq!(l[k], f[k], "{outil} {s} : {k} de {l}");
                }
                comptes += 1;
                if l["compteId"] == json!(usd) {
                    assert_eq!(l["devise"], json!("USD"), "{l}");
                    assert_eq!(l["soldeDevise"], json!("100.00"), "{l}");
                    usd_compare = true;
                }
            }
        }
        assert!(comptes > 0, "{outil} : aucune ligne de compte comparée");
    }
    assert!(usd_compare, "le compte en USD n'a pas été comparé");

    // Un groupe ouvert à la main : `ouvert` vrai (bilan, niveau 1).
    let g = structure(
        &b.appeler(
            &jeton,
            "kesh_bilan",
            json!({"fiscalYearId": fy, "detailLevel": 1, "open": [actifs]}),
        )
        .await,
        "kesh_bilan",
    );
    let ouvert = g["groupes"]["actifs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["compteId"] == json!(actifs))
        .expect("le groupe ouvert est servi");
    assert_eq!(ouvert["genre"], json!("group"), "{ouvert}");
    assert_eq!(ouvert["ouvert"], json!(true), "{ouvert}");
}

/// AC-1 — un refus par contrainte : chacun rend le même texte fixe, qui ne
/// nomme pas l'argument ; `kesh_balance` de même (Story 46-6a, AC-6). Les formes valides voisines sont servies (sonde
/// positive : le refus n'est pas celui de tout appel).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupes_arguments_refuses(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    let trop: Vec<i64> = (1..=501).collect();
    let assez: Vec<i64> = (1..=500).collect();
    let refuses = [
        // detailLevel : hors de 1 à 20 (46-1), ou ni entier ni « all » (lecture).
        json!({"detailLevel": 0}),
        json!({"detailLevel": 21}),
        json!({"detailLevel": -1}),
        json!({"detailLevel": 2.5}),
        json!({"detailLevel": "2"}),
        json!({"detailLevel": "ALL"}),
        json!({"detailLevel": true}),
        json!({"detailLevel": [2]}),
        // subtotalPosition : hors de l'énumération, ou sans detailLevel.
        json!({"detailLevel": 2, "subtotalPosition": "middle"}),
        json!({"subtotalPosition": "header"}),
        // open, closed : sans detailLevel, pas un tableau d'entiers, trop longs,
        // un même identifiant des deux côtés.
        json!({"open": [1]}),
        json!({"closed": []}),
        json!({"detailLevel": 2, "open": "1,2"}),
        json!({"detailLevel": 2, "closed": ["1"]}),
        json!({"detailLevel": 2, "open": trop}),
        json!({"detailLevel": 2, "closed": trop}),
        json!({"detailLevel": 2, "open": [7], "closed": [7]}),
    ];
    let servis = [
        json!({"detailLevel": 1}),
        json!({"detailLevel": 20}),
        json!({"detailLevel": "all"}),
        json!({"detailLevel": 2, "subtotalPosition": "header"}),
        json!({"detailLevel": 2, "open": assez.clone(), "closed": []}),
        json!({"detailLevel": 2, "closed": assez}),
        json!({"detailLevel": null}),
    ];
    for outil in ["kesh_balance", "kesh_bilan", "kesh_compte_de_resultat"] {
        for args in &refuses {
            let mut args = args.clone();
            args["fiscalYearId"] = json!(fy);
            let r = b.appeler(&jeton, outil, args.clone()).await;
            assert_eq!(
                r,
                attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES),
                "{outil} {args}"
            );
        }
        for args in &servis {
            let mut args = args.clone();
            args["fiscalYearId"] = json!(fy);
            let r = b.appeler(&jeton, outil, args.clone()).await;
            assert_eq!(r["result"]["isError"], json!(false), "{outil} {args} : {r}");
        }
    }
}

/// Les clés d'une ligne groupée de la balance (Story 46-6a, AC-6) : ni
/// `typeDeCompte` ni `actif`.
const CLES_LIGNE_BALANCE_GROUPEE: [&str; 17] = [
    "cloture",
    "clotureDevise",
    "compteId",
    "credit",
    "creditDevise",
    "debit",
    "debitDevise",
    "devise",
    "genre",
    "nom",
    "numero",
    "ouvert",
    "ouverture",
    "ouvertureDevise",
    "profondeur",
    "solde",
    "soldeDevise",
];

/// Story 46-6a, AC-6 — les bornes de `limit` de `kesh_balance` suivent le
/// mode : à plat 23 par défaut et 42 au plus (inchangés), avec `detailLevel`
/// 20 et 37 ; 38 refusé en mode groupé, accepté à plat.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn balance_bornes_selon_le_mode(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 467", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    let appel = |args: Value| {
        let b = &b;
        let jeton = &jeton;
        async move {
            structure(
                &b.appeler(jeton, "kesh_balance", args).await,
                "kesh_balance",
            )
        }
    };
    let plat = appel(json!({"fiscalYearId": fy})).await;
    let total = plat["total"].as_i64().unwrap();
    assert!(total > 42, "assez de comptes pour borner : {total}");
    assert_eq!(
        plat["lignes"].as_array().unwrap().len(),
        23,
        "défaut à plat"
    );
    assert_eq!(plat["limit"], json!(23));
    let plat42 = appel(json!({"fiscalYearId": fy, "limit": 42})).await;
    assert_eq!(plat42["lignes"].as_array().unwrap().len(), 42, "42 à plat");
    let groupe = appel(json!({"fiscalYearId": fy, "detailLevel": "all"})).await;
    assert!(groupe["total"].as_i64().unwrap() > 37);
    assert_eq!(
        groupe["groupes"]["lignes"].as_array().unwrap().len(),
        20,
        "défaut groupé"
    );
    assert_eq!(groupe["limit"], json!(20));
    let groupe37 = appel(json!({"fiscalYearId": fy, "detailLevel": "all", "limit": 37})).await;
    assert_eq!(groupe37["groupes"]["lignes"].as_array().unwrap().len(), 37);
    let r = b
        .appeler(
            &jeton,
            "kesh_balance",
            json!({"fiscalYearId": fy, "detailLevel": "all", "limit": 38}),
        )
        .await;
    assert_eq!(r, attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES));
}

/// Story 46-6a, AC-6 — `kesh_balance` par groupes : l'écho, `groupes.lignes`
/// et la liste plate vide ; la tranche porte sur les lignes groupées, que
/// `total` compte, et les tranches mises bout à bout redonnent la liste
/// entière ; les totaux du rapport sont inchangés ; chaque ligne de compte
/// porte les valeurs de sa ligne plate (dont un compte en USD).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn balance_groupes_par_le_mcp(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 466", 'A').await;
    let usd = compte_usd_mouvemente(&pool, &a).await;
    let actifs = compte_numero(&pool, &a, "1").await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let fy = a.p.exercice_id;
    let max = outils::rapports::LIMITE_MAX_MCP_BALANCE_GROUPEE;
    let appel = |args: Value| {
        let b = &b;
        let jeton = &jeton;
        async move {
            structure(
                &b.appeler(jeton, "kesh_balance", args).await,
                "kesh_balance",
            )
        }
    };

    // Le plat, en entier (par tranches).
    let mut plat: Vec<Value> = Vec::new();
    let temoin = appel(json!({"fiscalYearId": fy, "limit": max})).await;
    assert_eq!(temoin["groupes"], Value::Null, "témoin hors groupe");
    let total_plat = temoin["total"].as_i64().unwrap();
    let mut offset = 0;
    while offset < total_plat {
        let s = appel(json!({"fiscalYearId": fy, "limit": max, "offset": offset})).await;
        plat.extend(s["lignes"].as_array().unwrap().iter().cloned());
        offset += max;
    }
    assert_eq!(plat.len() as i64, total_plat);

    // Niveau 1 : les racines, fermées ; liste plate vide, totaux inchangés.
    let g = appel(json!({"fiscalYearId": fy, "detailLevel": 1, "limit": max})).await;
    let groupes = &g["groupes"];
    assert_eq!(groupes["niveau"], json!(1), "{groupes}");
    assert_eq!(groupes["tousLesNiveaux"], json!(false));
    assert_eq!(groupes["positionSousTotaux"], json!("header"));
    assert!(groupes["profondeurMax"].as_u64().unwrap() >= 2, "{groupes}");
    assert_eq!(
        (&groupes["ouverts"], &groupes["fermes"]),
        (&json!([]), &json!([]))
    );
    assert_eq!(g["lignes"], json!([]), "liste plate vide");
    let lignes = groupes["lignes"].as_array().unwrap();
    assert!(!lignes.is_empty());
    assert_eq!(g["total"], json!(lignes.len()), "total des lignes groupées");
    for l in lignes {
        let cles: Vec<&str> = l.as_object().unwrap().keys().map(String::as_str).collect();
        assert_eq!(cles, CLES_LIGNE_BALANCE_GROUPEE, "{l}");
        assert_eq!(l["profondeur"], json!(1), "{l}");
    }
    for k in [
        "periode",
        "totalDebit",
        "totalCredit",
        "equilibree",
        "reportBenefice",
        "ouvertureEquilibree",
        "limit",
        "offset",
    ] {
        assert_eq!(g[k], temoin[k], "{k} inchangé");
    }

    // Tous les niveaux, en pied, un groupe ouvert à la main et des
    // identifiants ignorés : tranches bout à bout = la liste entière.
    let args = |limit: i64, offset: i64| {
        json!({"fiscalYearId": fy, "detailLevel": "all", "subtotalPosition": "footer",
               "open": [actifs], "closed": [INEXISTANT, 0, -5],
               "limit": limit, "offset": offset})
    };
    let tete = appel(args(max, 0)).await;
    let total = tete["total"].as_i64().unwrap();
    assert!(total > max, "plus d'une tranche : {total}");
    assert_eq!(tete["groupes"]["tousLesNiveaux"], json!(true));
    assert_eq!(tete["groupes"]["ouverts"], json!([actifs]));
    assert_eq!(
        tete["groupes"]["fermes"],
        json!([]),
        "inconnus retirés de l'écho"
    );
    let mut toutes: Vec<Value> = Vec::new();
    let mut offset = 0;
    while offset < total {
        let s = appel(args(max, offset)).await;
        assert_eq!(s["total"], json!(total));
        assert_eq!(s["lignes"], json!([]));
        toutes.extend(s["groupes"]["lignes"].as_array().unwrap().iter().cloned());
        offset += max;
    }
    assert_eq!(toutes.len() as i64, total, "chaque ligne groupée une fois");
    let deux = appel(args(2, 1)).await;
    assert_eq!(
        deux["groupes"]["lignes"].as_array().unwrap()[..],
        toutes[1..3],
        "tranche (2, 1)"
    );
    let genres: BTreeSet<&str> = toutes
        .iter()
        .map(|l| l["genre"].as_str().unwrap())
        .collect();
    assert!(
        genres.contains("subtotal") && genres.contains("account"),
        "{genres:?}"
    );

    // Fidélité au plat : chaque ligne de compte porte les valeurs de sa ligne
    // plate ; le compte en USD compris.
    let mut usd_compare = false;
    let mut comptes = 0;
    for l in toutes.iter().filter(|l| l["genre"] == json!("account")) {
        let f = plat
            .iter()
            .find(|f| f["compteId"] == l["compteId"])
            .unwrap_or_else(|| panic!("pas de ligne plate pour {l}"));
        for k in [
            "numero",
            "nom",
            "ouverture",
            "debit",
            "credit",
            "solde",
            "cloture",
            "devise",
            "ouvertureDevise",
            "debitDevise",
            "creditDevise",
            "soldeDevise",
            "clotureDevise",
        ] {
            assert_eq!(l[k], f[k], "{k} de {l}");
        }
        comptes += 1;
        usd_compare |= l["compteId"] == json!(usd);
    }
    assert!(
        comptes > 0 && usd_compare,
        "comptes comparés : {comptes}, USD : {usd_compare}"
    );
}

/// Une écriture d'un montant en USD au débit de `compte` (aux décimaux larges
/// de `tailles_mesurees_en_devise`), K2 au crédit.
async fn ecriture_usd(pool: &MySqlPool, s: &Societe, compte: i64, i: usize) {
    journal_entries::create(
        pool,
        s.p.exercice_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(s.u),
        NewJournalEntry {
            company_id: s.c,
            entry_date: common::jour::aujourd_hui_test(),
            journal: Journal::OD,
            description: texte_de(55, &format!("Écriture groupée {i}")),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: compte,
                    debit: Decimal::new(123_455, 2),
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: Some(kesh_db::entities::ForeignAmount {
                        declared_currency: None,
                        amount: Decimal::new(1_355_789, 3),
                        rate: Decimal::new(9_105_786_123, 10),
                        source: Some(kesh_db::entities::RateProvenance::Manual),
                        rate_derived: false,
                    }),
                },
                NewJournalEntryLine {
                    account_id: s.p.compte_k2,
                    debit: Decimal::ZERO,
                    credit: Decimal::new(123_455, 2),
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .unwrap_or_else(|e| panic!("écriture en USD sur {compte} : {e}"));
}

/// Le nombre de lignes groupées d'une sortie.
fn lignes_groupees(outil: &str, s: &Value) -> usize {
    sections_de(outil)
        .iter()
        .map(|k| s["groupes"][k].as_array().unwrap().len())
        .sum()
}

/// AC-5 (a) — le plan PME (l'arbre de `societe_complete`, hors comptes à rôle,
/// que ce test ne mouvemente pas), chaque
/// compte imputable de bilan, de produits et de charges sans rôle ni écriture
/// passé en USD et mouvementé : tous à solde non nul et en devise. Aux niveaux 2
/// et `all`, sous-totaux sur le titre puis en pied, la réponse tient sous
/// 23 552 o (la marge des autres outils).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tailles_groupees_plan_pme(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464p", 'A').await;
    compte_usd_mouvemente(&pool, &a).await;
    let comptes: Vec<i64> = sqlx::query_scalar(
        "SELECT a.id FROM accounts a WHERE a.company_id = ? AND a.postable AND a.role IS NULL \
         AND a.currency IS NULL AND NOT EXISTS \
         (SELECT 1 FROM journal_entry_lines l WHERE l.account_id = a.id) ORDER BY a.number",
    )
    .bind(a.c)
    .fetch_all(&pool)
    .await
    .unwrap();
    for (i, id) in comptes.iter().enumerate() {
        sqlx::query("UPDATE accounts SET currency = 'USD' WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        ecriture_usd(&pool, &a, *id, i).await;
    }
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    for outil in ["kesh_bilan", "kesh_compte_de_resultat"] {
        for niveau in [json!(2), json!("all")] {
            for position in ["header", "footer"] {
                let r = b
                    .appeler(
                        &jeton,
                        outil,
                        json!({"fiscalYearId": a.p.exercice_id, "detailLevel": niveau,
                               "subtotalPosition": position}),
                    )
                    .await;
                let n = texte(&r).len();
                let s = structure(&r, outil);
                eprintln!(
                    "TAILLE GROUPÉE PME {outil} niveau {niveau} {position} : {} lignes, {n} o \
                     ({} comptes en USD)",
                    lignes_groupees(outil, &s),
                    comptes.len()
                );
                assert!(n <= 23_552, "{outil} {niveau} {position} : {n} o");
            }
        }
    }
}

/// AC-5 (b), (c) — une fixture synthétique : un groupe posé par SQL et ses
/// comptes en USD (noms de 34 caractères), au niveau `all`, sous-totaux en
/// pied. Mesure l'élément (une ligne groupée de compte), puis ajoute un compte
/// à la fois jusqu'au refus pour taille : le nombre de lignes groupées servies
/// au plus est au moins le seuil écrit dans la doc du module et les
/// descriptions, et au-delà la réponse est refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tailles_groupees_seuil(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 464s", 'A').await;
    compte_usd_mouvemente(&pool, &a).await;
    let groupe: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, postable) \
         VALUES (?, '09', ?, 'Asset', FALSE) RETURNING id",
    )
    .bind(a.c)
    .bind(texte_de(34, "Groupe de fixture"))
    .fetch_one(&pool)
    .await
    .unwrap();
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let args = json!({"fiscalYearId": a.p.exercice_id, "detailLevel": "all",
                      "subtotalPosition": "footer"});
    let mut servi: Option<(usize, usize)> = None;
    let mut tailles = Vec::new();
    for i in 0..200 {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type, currency, parent_id) \
             VALUES (?, ?, ?, 'Asset', 'USD', ?) RETURNING id",
        )
        .bind(a.c)
        .bind(format!("09{i:03}"))
        .bind(texte_de(34, &format!("Compte groupé {i}")))
        .bind(groupe)
        .fetch_one(&pool)
        .await
        .unwrap();
        ecriture_usd(&pool, &a, id, i).await;
        let r = b.appeler(&jeton, "kesh_bilan", args.clone()).await;
        if r["result"]["isError"] == json!(true) {
            assert_eq!(
                r,
                attendu_outil_en_erreur(1, TEXTE_ENVOI_TROP_VOLUMINEUX),
                "refus pour taille"
            );
            break;
        }
        let n = texte(&r).len();
        tailles.push(n);
        servi = Some((
            lignes_groupees("kesh_bilan", &structure(&r, "kesh_bilan")),
            n,
        ));
    }
    let (lignes, octets) = servi.expect("au moins une réponse servie");
    assert!(tailles.len() < 200, "le refus pour taille est atteint");
    eprintln!(
        "SEUIL GROUPÉ kesh_bilan (all, footer, USD, noms de 34) : élément {} o ; {lignes} lignes \
         servies au plus ({octets} o), refus au-delà",
        tailles[1] - tailles[0]
    );
    assert!(
        lignes >= outils::rapports::SEUIL_LIGNES_GROUPEES,
        "le seuil écrit ({}) est une borne basse : {lignes} lignes servies",
        outils::rapports::SEUIL_LIGNES_GROUPEES
    );
}
