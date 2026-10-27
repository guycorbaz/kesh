//! Montage des tests des outils de lecture — Story 32-9a (refs #723) : deux
//! sociétés piégées (`kesh_seed::pieges`), les clés, la table des témoins
//! (AC-a17) et le parcours qui vérifie que tout piège sort **enveloppé**
//! (AC-a15).
//!
//! Aucune donnée réelle : la démonstration et les pièges `PIEGE_329_*`
//! seulement.

use std::collections::BTreeSet;

use axum::Router;
use kesh_api::mcp::Registre;
use kesh_api::mcp::registre::NOM_LIRE_ENVOI;
use kesh_db::entities::{McpScope, SendValidation};
use kesh_seed::pieges::{self, MOTIF_PIEGE, Pieges};
use regex::Regex;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use super::mcp::{
    Rendu, activer, administrateur, appel, cle, config, envoyer, post_mcp, routeur, static_dir,
};

/// Les quatre portées de lecture.
pub const LECTURE: [McpScope; 4] = [
    McpScope::BooksRead,
    McpScope::InvoicesRead,
    McpScope::ContactsRead,
    McpScope::DocumentsRead,
];

/// Les outils hors registre listés par `tools/list` pour toute clé.
pub const HORS_REGISTRE: [&str; 1] = [NOM_LIRE_ENVOI];

/// Le motif des pièges, compilé.
pub fn motif() -> Regex {
    Regex::new(MOTIF_PIEGE).unwrap()
}

/// Une société piégée : son id, son administrateur, ses pièges.
pub struct Societe {
    pub c: i64,
    pub u: i64,
    pub p: Pieges,
}

/// Une société complète, servie (réglages MCP activés), son administrateur et
/// ses pièges de marque `marque`.
pub async fn societe_piegee(pool: &MySqlPool, nom: &str, marque: char) -> Societe {
    let c = pieges::societe_complete(pool, nom).await.expect("société");
    let u = administrateur(pool, c, &format!("admin-329-{marque}")).await;
    activer(pool, c).await;
    let p = pieges::semer(pool, (u, None), c, marque)
        .await
        .expect("pièges semés");
    Societe { c, u, p }
}

/// Une clé de la société `s`, aux portées et à la politique données.
pub async fn cle_de(
    pool: &MySqlPool,
    s: &Societe,
    portees: &[McpScope],
    validation: SendValidation,
) -> String {
    cle(pool, s.c, s.u, portees, validation).await.1
}

/// Le banc : le routeur de production, registre standard.
pub struct Banc {
    pub app: Router,
    _statique: tempfile::TempDir,
}

/// Le banc du registre standard.
pub fn banc(pool: &MySqlPool) -> Banc {
    banc_avec(pool, Registre::standard())
}

/// Le banc d'un registre donné.
pub fn banc_avec(pool: &MySqlPool, registre: Registre) -> Banc {
    let statique = static_dir();
    Banc {
        app: routeur(
            pool.clone(),
            config(),
            statique.path(),
            registre,
            "127.0.0.1",
        ),
        _statique: statique,
    }
}

impl Banc {
    /// Un message JSON-RPC sous `jeton`.
    pub async fn mcp(&self, jeton: &str, corps: &str) -> Rendu {
        envoyer(&self.app, post_mcp(Some(jeton), corps)).await
    }

    /// `tools/call` de `outil` ; le corps JSON entier.
    pub async fn appeler(&self, jeton: &str, outil: &str, arguments: Value) -> Value {
        self.mcp(jeton, &appel(1, outil, arguments)).await.json()
    }

    /// Les noms de `tools/list` sous `jeton`, dans l'ordre.
    pub async fn liste(&self, jeton: &str) -> Vec<String> {
        let r = self
            .mcp(jeton, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#)
            .await
            .json();
        r["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["name"].as_str().unwrap().to_string())
            .collect()
    }
}

/// Le `structuredContent` d'une réponse servie ; panique si elle ne l'est pas.
pub fn structure(reponse: &Value, contexte: &str) -> Value {
    assert_eq!(
        reponse["result"]["isError"],
        json!(false),
        "{contexte} : non servi : {reponse}"
    );
    reponse["result"]["structuredContent"].clone()
}

/// Les jeux d'arguments d'un outil, sur les pièges d'une société.
pub type Jeux = fn(&Pieges) -> Vec<Value>;
/// L'argument complet d'un outil : chaque propriété du schéma, valide.
pub type Complet = fn(&Pieges) -> Value;

fn d(date: chrono::NaiveDate) -> String {
    date.to_string()
}

/// AC-a17 — la table des témoins : (outil, jeux d'arguments, argument
/// complet). Les jeux suivent le § *Les jeux d'arguments de l'inventaire*.
pub const TEMOINS: &[(&str, Jeux, Complet)] = &[
    ("kesh_version", |_| vec![json!({})], |_| json!({})),
    ("kesh_exercices", |_| vec![json!({})], |_| json!({})),
    (
        "kesh_plan_comptable",
        |_| vec![json!({"limit": 70})],
        |_| json!({"inclureArchives": true, "limit": 70, "offset": 0}),
    ),
    (
        "kesh_ecritures",
        |p| {
            vec![
                json!({"accountId": p.compte_k1}),
                json!({"accountId": p.compte_cr}),
            ]
        },
        |p| {
            json!({
                "description": "P", "accountId": p.compte_k1, "amountMin": "0",
                "amountMax": "100000", "dateFrom": d(p.exercice_debut),
                "dateTo": d(p.exercice_fin), "journal": "OD", "sortBy": "entryDate",
                "sortDir": "asc", "limit": 22, "offset": 0,
            })
        },
    ),
    (
        "kesh_ecriture",
        |p| vec![json!({"id": p.ecriture}), json!({"id": p.ecriture_achat})],
        |p| json!({"id": p.ecriture}),
    ),
    (
        "kesh_grand_livre",
        |p| {
            vec![
                json!({"accountId": p.compte_k1, "from": d(p.exercice_debut), "to": d(p.exercice_fin)}),
                json!({"accountId": p.compte_cr, "from": d(p.exercice_debut), "to": d(p.exercice_fin)}),
            ]
        },
        |p| {
            json!({"accountId": p.compte_k1, "from": d(p.exercice_debut),
                   "to": d(p.exercice_fin), "limit": 30, "offset": 0})
        },
    ),
    (
        "kesh_balance",
        |p| vec![json!({"fiscalYearId": p.exercice_id, "limit": 55})],
        |p| {
            json!({"fiscalYearId": p.exercice_id, "periodStart": d(p.exercice_debut),
                   "periodEnd": d(p.exercice_fin), "limit": 55, "offset": 0})
        },
    ),
    (
        "kesh_bilan",
        |p| vec![json!({"fiscalYearId": p.exercice_id})],
        |p| {
            json!({"fiscalYearId": p.exercice_id, "periodStart": d(p.exercice_debut),
                   "periodEnd": d(p.exercice_fin)})
        },
    ),
    (
        "kesh_compte_de_resultat",
        |p| vec![json!({"fiscalYearId": p.exercice_id})],
        |p| {
            json!({"fiscalYearId": p.exercice_id, "periodStart": d(p.exercice_debut),
                   "periodEnd": d(p.exercice_fin)})
        },
    ),
    (
        "kesh_postes_ouverts",
        |p| {
            vec![
                json!({"accountId": p.compte_k1}),
                json!({"accountId": p.compte_cr}),
            ]
        },
        |p| json!({"accountId": p.compte_k1, "asOf": d(p.exercice_fin), "limit": 24, "offset": 0}),
    ),
    (
        "kesh_factures",
        |_| vec![json!({})],
        |p| {
            json!({"search": "P", "status": "validated", "contactId": p.client,
                   "dateFrom": d(p.exercice_debut), "dateTo": d(p.exercice_fin),
                   "paused": "all", "sortBy": "date", "sortDirection": "desc",
                   "limit": 40, "offset": 0})
        },
    ),
    (
        "kesh_facture",
        |p| vec![json!({"id": p.facture})],
        |p| json!({"id": p.facture}),
    ),
    (
        "kesh_echeancier",
        |_| vec![json!({"paymentStatus": "all"})],
        |p| {
            json!({"search": "P", "contactId": p.client, "dateFrom": d(p.exercice_debut),
                   "dateTo": d(p.exercice_fin), "dueBefore": d(p.exercice_fin),
                   "paymentStatus": "all", "sortBy": "dueDate", "sortDirection": "asc",
                   "limit": 40, "offset": 0})
        },
    ),
    (
        "kesh_balance_agee",
        |_| vec![json!({"limit": 68})],
        |_| json!({"limit": 68, "offset": 0}),
    ),
    (
        "kesh_factures_fournisseur",
        |_| vec![json!({})],
        |_| json!({"limit": 70, "offset": 0}),
    ),
    (
        "kesh_facture_fournisseur",
        |p| vec![json!({"id": p.facture_fournisseur})],
        |p| json!({"id": p.facture_fournisseur}),
    ),
];

/// La ligne de `TEMOINS` de `outil`.
pub fn temoin(outil: &str) -> &'static (&'static str, Jeux, Complet) {
    TEMOINS
        .iter()
        .find(|(n, _, _)| *n == outil)
        .unwrap_or_else(|| panic!("aucun témoin pour {outil}"))
}

/// Le premier jeu d'arguments d'un outil.
pub fn premier_jeu(outil: &str, p: &Pieges) -> Value {
    (temoin(outil).1)(p).remove(0)
}

/// Les fautes d'une valeur : toute chaîne — clé comprise — où le motif
/// correspond **hors** de la valeur de l'unique clé `nonFiable` d'un objet.
pub fn fautes(v: &Value, motif: &Regex) -> Vec<String> {
    let mut f = Vec::new();
    parcourir(v, motif, &mut f, &mut Vec::new());
    f
}

/// Les textes enveloppés d'une valeur (valeurs des objets `{"nonFiable": …}`).
pub fn enveloppes(v: &Value) -> Vec<String> {
    let mut f = Vec::new();
    let mut e = Vec::new();
    parcourir(v, &Regex::new("$^").unwrap(), &mut f, &mut e);
    e
}

fn parcourir(v: &Value, motif: &Regex, fautes: &mut Vec<String>, env: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            if m.len() == 1
                && let Some(Value::String(s)) = m.get("nonFiable")
            {
                env.push(s.clone());
                return;
            }
            for (k, val) in m {
                if motif.is_match(k) {
                    fautes.push(format!("clé : {k}"));
                }
                parcourir(val, motif, fautes, env);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| parcourir(x, motif, fautes, env)),
        Value::String(s) if motif.is_match(s) => fautes.push(format!("chaîne nue : {s}")),
        _ => {}
    }
}

/// Les identifiants de pièges trouvés, enveloppés, dans une valeur.
pub fn identifiants_enveloppes(v: &Value, motif: &Regex) -> BTreeSet<String> {
    enveloppes(v)
        .iter()
        .flat_map(|s| {
            motif
                .find_iter(s)
                .map(|m| m.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}
