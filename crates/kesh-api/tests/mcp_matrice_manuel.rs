//! Story 32-9a (refs #723) — la matrice portée × outil, générée depuis
//! `tools/list` et comparée au manuel d'administration (AC-a23, C-32-9-16).
//!
//! Le manuel porte, entre `% matrice-mcp:debut` et `% matrice-mcp:fin`, une
//! ligne par outil : `\texttt{<nom>} & \texttt{<portée>} & … \\`, ou
//! `… & toute clé & … \\` pour un outil listé sous toutes les portées
//! (`kesh_lire_envoi`). Les portées parcourues sont les quatre de lecture et,
//! depuis la Story 32-10a-iii, `prompts:write` (`proposer_prompt`). Un bloc absent, une ligne en double ou une
//! ligne que l'analyse ne lit pas rougissent.

mod common;

use std::collections::BTreeSet;

use kesh_db::entities::{McpScope, SendValidation};
use regex::Regex;
use sqlx::MySqlPool;

use common::mcp_pieges::*;

/// Le nom d'une portée, tel que le manuel l'écrit.
fn nom_de(p: McpScope) -> &'static str {
    match p {
        McpScope::BooksRead => "books:read",
        McpScope::InvoicesRead => "invoices:read",
        McpScope::ContactsRead => "contacts:read",
        McpScope::DocumentsRead => "documents:read",
        McpScope::PromptsWrite => "prompts:write",
    }
}

/// Les couples (outil, portée) du bloc de matrice d'un texte LaTeX.
fn matrice_du_texte(tex: &str) -> Result<BTreeSet<(String, String)>, String> {
    let debut = tex
        .find("% matrice-mcp:debut")
        .ok_or("bloc absent : % matrice-mcp:debut")?;
    let fin = tex[debut..]
        .find("% matrice-mcp:fin")
        .ok_or("bloc non fermé : % matrice-mcp:fin")?
        + debut;
    let ligne = Regex::new(
        r"^\\texttt\{([a-z0-9\\_]+)\}\s*&\s*(?:\\texttt\{([a-z:]+)\}|(toute clé))\s*&.*\\\\$",
    )
    .unwrap();
    let mut v = BTreeSet::new();
    for l in tex[debut..fin].lines().skip(1) {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        let c = ligne
            .captures(l)
            .ok_or_else(|| format!("ligne illisible : {l}"))?;
        let outil = c[1].replace("\\_", "_");
        let portee = c.get(2).or_else(|| c.get(3)).unwrap().as_str().to_string();
        if !v.insert((outil.clone(), portee)) {
            return Err(format!("ligne en double : {outil}"));
        }
    }
    Ok(v)
}

#[test]
fn analyse_du_bloc_temoins() {
    let bon = "x\n% matrice-mcp:debut\n\\texttt{kesh\\_bilan} & \\texttt{books:read} & le bilan \\\\\n\\texttt{kesh\\_lire\\_envoi} & toute clé & relecture \\\\\n% matrice-mcp:fin\n";
    assert_eq!(
        matrice_du_texte(bon).unwrap(),
        [
            ("kesh_bilan".to_string(), "books:read".to_string()),
            ("kesh_lire_envoi".to_string(), "toute clé".to_string()),
        ]
        .into()
    );
    assert!(matrice_du_texte("sans bloc").is_err(), "bloc absent");
    let double = bon.replace(
        "% matrice-mcp:fin",
        "\\texttt{kesh\\_bilan} & \\texttt{books:read} & encore \\\\\n% matrice-mcp:fin",
    );
    assert!(matrice_du_texte(&double).is_err(), "ligne en double");
    let illisible = bon.replace(
        "% matrice-mcp:fin",
        "kesh_bilan books:read\n% matrice-mcp:fin",
    );
    assert!(matrice_du_texte(&illisible).is_err(), "ligne illisible");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn matrice_du_manuel(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let b = banc(&pool);
    let mut par_portee: Vec<(McpScope, Vec<String>)> = Vec::new();
    // Les quatre portées de lecture **et** `prompts:write` (Story 32-10a-iii,
    // revue de code P1, E2) : `proposer_prompt` n'est servi que sous cette
    // portée, et le manuel le dit.
    for p in LECTURE.into_iter().chain([McpScope::PromptsWrite]) {
        let jeton = cle_de(&pool, &a, &[p], SendValidation::Exempt).await;
        par_portee.push((p, b.liste(&jeton).await));
    }
    let partout: BTreeSet<String> = par_portee[0]
        .1
        .iter()
        .filter(|o| par_portee.iter().all(|(_, l)| l.contains(o)))
        .cloned()
        .collect();
    let mut generee = BTreeSet::new();
    for (p, liste) in &par_portee {
        for o in liste {
            let portee = if partout.contains(o) {
                "toute clé".to_string()
            } else {
                nom_de(*p).to_string()
            };
            generee.insert((o.clone(), portee));
        }
    }
    assert!(
        generee.len() >= 20,
        "témoin : {} couples générés",
        generee.len()
    );
    let tex = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/manual/fr/admin-manual.tex"),
    )
    .unwrap();
    assert_eq!(matrice_du_texte(&tex).unwrap(), generee);
}
