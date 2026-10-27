//! Story 32-9a (refs #723) — l'enveloppe « donnée non fiable » : les pièges,
//! l'inventaire clos des champs libres, les coordonnées des tiers (AC-a13 à
//! AC-a16).
//!
//! ⛔ Le seul motif des pièges est `kesh_seed::pieges::MOTIF_PIEGE` ; aucun
//! test ne le réécrit. Aucune donnée réelle : les pièges `PIEGE_329_*` et
//! `P329*` seulement, posés par les dépôts de production.

mod common;

use std::collections::BTreeSet;

use kesh_api::mcp::textes::{TEXTE_ENTETE_DONNEES, TEXTE_TIERS_MASQUE};
use kesh_db::entities::{McpScope, SendValidation};
use kesh_seed::pieges::{self, CHAMPS, EVASION, MOTIF_PIEGE, par_cle};
use regex::Regex;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::mcp::numero_depose;
use common::mcp_pieges::*;

/// Le traitement d'une ligne de l'inventaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Traitement {
    /// Enveloppé (`TexteNonFiable`).
    NonFiable,
    /// Coordonnée de tiers (`Tiers`).
    Tiers,
}

/// Une ligne de l'inventaire clos (§ *L'inventaire clos des champs libres
/// rendus* de la fiche, lignes `NF` et `Tiers`) : outil, champ, traitement,
/// piège (clé de `CHAMPS`) **ou** exemption écrite, et les couples (outil,
/// jeu de `TEMOINS`) qui la font sortir.
struct LigneInventaire {
    outil: &'static str,
    champ: &'static str,
    traitement: Traitement,
    piege: Option<&'static str>,
    exemption: Option<&'static str>,
    jeux: &'static [(&'static str, usize)],
}

const fn nf(
    outil: &'static str,
    champ: &'static str,
    piege: &'static str,
    jeux: &'static [(&'static str, usize)],
) -> LigneInventaire {
    LigneInventaire {
        outil,
        champ,
        traitement: Traitement::NonFiable,
        piege: Some(piege),
        exemption: None,
        jeux,
    }
}

const fn exempte(
    outil: &'static str,
    champ: &'static str,
    exemption: &'static str,
) -> LigneInventaire {
    LigneInventaire {
        outil,
        champ,
        traitement: Traitement::NonFiable,
        piege: None,
        exemption: Some(exemption),
        jeux: &[],
    }
}

const fn tiers(
    outil: &'static str,
    champ: &'static str,
    piege: &'static str,
    jeux: &'static [(&'static str, usize)],
) -> LigneInventaire {
    LigneInventaire {
        outil,
        champ,
        traitement: Traitement::Tiers,
        piege: Some(piege),
        exemption: None,
        jeux,
    }
}

/// AC-a13 — l'inventaire, recopié de la fiche.
///
/// ⚠️ **Ce qui est clos, et ce qui ne l'est pas** (revue de code P1, B6) :
/// l'inventaire est clos **par outil** (`inventaire_couvre_le_registre`
/// compare des noms d'outils) et **par colonne piégée** (chaque ligne exige
/// sa trouvaille) — non par champ rendu. Un champ `TexteNonFiable` d'une
/// structure de sortie qui partage la colonne d'une autre ligne n'y a pas sa
/// propre ligne (`kesh_facture` : `numero`, `conditions`, `noteSuspension` ;
/// `kesh_facture_fournisseur` : `numero` ; `kesh_bilan` : `passifs[]`,
/// `fondsPropres[]` ; `kesh_compte_de_resultat` : `produits[]`), et un champ
/// neuf oublié ici ne rougit nulle part : c'est le typage (`sortie!` exige
/// `ChampSur`) qui tient l'enveloppe, et la revue du tableau de l'AC-a10 qui
/// tient l'inventaire.
const INVENTAIRE: &[LigneInventaire] = &[
    // books:read
    nf(
        "kesh_exercices",
        "exercices[].nom",
        "exercice",
        &[("kesh_exercices", 0)],
    ),
    nf(
        "kesh_plan_comptable",
        "comptes[].numero (K1)",
        "compte_k1_numero",
        &[("kesh_plan_comptable", 0)],
    ),
    nf(
        "kesh_plan_comptable",
        "comptes[].numero (K2)",
        "compte_k2_numero",
        &[("kesh_plan_comptable", 0)],
    ),
    nf(
        "kesh_plan_comptable",
        "comptes[].nom (K1)",
        "compte_k1_nom",
        &[("kesh_plan_comptable", 0)],
    ),
    nf(
        "kesh_plan_comptable",
        "comptes[].nom (K2)",
        "compte_k2_nom",
        &[("kesh_plan_comptable", 0)],
    ),
    nf(
        "kesh_ecritures",
        "ecritures[].libelle (E)",
        "libelle_ecriture",
        &[("kesh_ecritures", 0), ("kesh_ecriture", 0)],
    ),
    nf(
        "kesh_ecriture",
        "ecriture.libelle (EA, nom du fournisseur composé par Kesh)",
        "fournisseur_nom",
        &[("kesh_ecritures", 1), ("kesh_ecriture", 1)],
    ),
    nf(
        "kesh_ecriture",
        "pieceSource.numeroFactureFournisseur",
        "ff_numero",
        &[("kesh_ecriture", 1)],
    ),
    nf(
        "kesh_grand_livre",
        "sections[].numero",
        "compte_k1_numero",
        &[("kesh_grand_livre", 0)],
    ),
    nf(
        "kesh_grand_livre",
        "sections[].nom",
        "compte_k1_nom",
        &[("kesh_grand_livre", 0)],
    ),
    nf(
        "kesh_grand_livre",
        "lignes[].exercice",
        "exercice",
        &[("kesh_grand_livre", 0)],
    ),
    nf(
        "kesh_grand_livre",
        "lignes[].libelle (E)",
        "libelle_ecriture",
        &[("kesh_grand_livre", 0)],
    ),
    nf(
        "kesh_grand_livre",
        "lignes[].libelle (EA)",
        "fournisseur_nom",
        &[("kesh_grand_livre", 1)],
    ),
    nf(
        "kesh_grand_livre",
        "lignes[].contreparties[]",
        "compte_k2_numero",
        &[("kesh_grand_livre", 0)],
    ),
    exempte(
        "kesh_grand_livre",
        "lignes[].codeLettrage",
        "calculé par Kesh d'un entier : aucun texte saisi n'y entre",
    ),
    nf(
        "kesh_balance",
        "lignes[].numero (K1)",
        "compte_k1_numero",
        &[("kesh_balance", 0)],
    ),
    nf(
        "kesh_balance",
        "lignes[].numero (K2)",
        "compte_k2_numero",
        &[("kesh_balance", 0)],
    ),
    nf(
        "kesh_balance",
        "lignes[].nom (K1)",
        "compte_k1_nom",
        &[("kesh_balance", 0)],
    ),
    nf(
        "kesh_balance",
        "lignes[].nom (K2)",
        "compte_k2_nom",
        &[("kesh_balance", 0)],
    ),
    nf(
        "kesh_bilan",
        "actifs[].numero",
        "compte_k1_numero",
        &[("kesh_bilan", 0)],
    ),
    nf(
        "kesh_bilan",
        "actifs[].nom",
        "compte_k1_nom",
        &[("kesh_bilan", 0)],
    ),
    nf(
        "kesh_compte_de_resultat",
        "charges[].numero",
        "compte_k2_numero",
        &[("kesh_compte_de_resultat", 0)],
    ),
    nf(
        "kesh_compte_de_resultat",
        "charges[].nom",
        "compte_k2_nom",
        &[("kesh_compte_de_resultat", 0)],
    ),
    nf(
        "kesh_postes_ouverts",
        "numeroCompte",
        "compte_k1_numero",
        &[("kesh_postes_ouverts", 0)],
    ),
    nf(
        "kesh_postes_ouverts",
        "postes[].exercice",
        "exercice",
        &[("kesh_postes_ouverts", 0)],
    ),
    nf(
        "kesh_postes_ouverts",
        "postes[].libelle (E)",
        "libelle_ecriture",
        &[("kesh_postes_ouverts", 0)],
    ),
    nf(
        "kesh_postes_ouverts",
        "postes[].libelle (EA)",
        "fournisseur_nom",
        &[("kesh_postes_ouverts", 1)],
    ),
    nf(
        "kesh_postes_ouverts",
        "postes[].piece.numero",
        "ff_numero",
        &[("kesh_postes_ouverts", 1)],
    ),
    exempte(
        "kesh_postes_ouverts",
        "postes[].piece.numeroFacture",
        "posé pour un règlement seulement (DocumentKind::Settlement), que semer ne crée pas",
    ),
    exempte(
        "kesh_postes_ouverts",
        "postes[].codeLettrage",
        "calculé par Kesh d'un entier : aucun texte saisi n'y entre",
    ),
    // invoices:read
    nf(
        "kesh_factures",
        "factures[].client",
        "client_nom",
        &[("kesh_factures", 0), ("kesh_echeancier", 0)],
    ),
    exempte(
        "kesh_factures",
        "factures[].numero",
        "calculé par Kesh à la validation, du format de numérotation (défaut {YEAR}-{INVOICE_NUMBER}) : non posable par les dépôts",
    ),
    nf(
        "kesh_factures",
        "factures[].conditions",
        "facture_conditions",
        &[
            ("kesh_factures", 0),
            ("kesh_echeancier", 0),
            ("kesh_facture", 0),
        ],
    ),
    nf(
        "kesh_factures",
        "factures[].noteSuspension",
        "facture_note_suspension",
        &[
            ("kesh_factures", 0),
            ("kesh_echeancier", 0),
            ("kesh_facture", 0),
        ],
    ),
    tiers(
        "kesh_facture",
        "envoyeeA",
        "facture_envoyee_a",
        &[("kesh_facture", 0)],
    ),
    nf(
        "kesh_facture",
        "lignes[].description",
        "facture_ligne",
        &[("kesh_facture", 0)],
    ),
    nf(
        "kesh_balance_agee",
        "lignes[].client",
        "client_nom",
        &[("kesh_balance_agee", 0)],
    ),
    nf(
        "kesh_factures_fournisseur",
        "factures[].fournisseur",
        "fournisseur_nom",
        &[("kesh_factures_fournisseur", 0)],
    ),
    nf(
        "kesh_factures_fournisseur",
        "factures[].numero",
        "ff_numero",
        &[
            ("kesh_factures_fournisseur", 0),
            ("kesh_facture_fournisseur", 0),
        ],
    ),
    tiers(
        "kesh_facture_fournisseur",
        "ibanCreancier",
        "ff_iban",
        &[("kesh_facture_fournisseur", 0)],
    ),
    tiers(
        "kesh_facture_fournisseur",
        "qrIbanCreancier",
        "ff_qr_iban",
        &[("kesh_facture_fournisseur", 0)],
    ),
    nf(
        "kesh_facture_fournisseur",
        "referencePaiement",
        "ff_reference",
        &[("kesh_facture_fournisseur", 0)],
    ),
    nf(
        "kesh_facture_fournisseur",
        "lignes[].description",
        "ff_ligne",
        &[("kesh_facture_fournisseur", 0)],
    ),
    // contacts:read (32-9b) — la portée est celle des données personnelles :
    // tout est enveloppé, rien n'est masqué (pas de `Tiers`).
    nf("kesh_contacts", "contacts[].nom (C)", "client_nom", CLIENT),
    nf(
        "kesh_contacts",
        "contacts[].nom (Fo)",
        "fournisseur_nom",
        &[("kesh_contacts", 0), ("kesh_contact", 1)],
    ),
    nf(
        "kesh_contacts",
        "contacts[].prenom",
        "client_prenom",
        CLIENT,
    ),
    nf(
        "kesh_contacts",
        "contacts[].nomDeFamille",
        "client_nom_de_famille",
        CLIENT,
    ),
    // L'adresse d'affichage est composée par le dépôt de ses composants
    // (`derive_contact_address_display`) : elle porte le piège de la rue.
    nf(
        "kesh_contacts",
        "contacts[].adresse (composée par Kesh)",
        "client_rue",
        CLIENT,
    ),
    nf("kesh_contacts", "contacts[].rue", "client_rue", CLIENT),
    nf(
        "kesh_contacts",
        "contacts[].batiment",
        "client_batiment",
        CLIENT,
    ),
    nf("kesh_contacts", "contacts[].npa", "client_npa", CLIENT),
    nf(
        "kesh_contacts",
        "contacts[].localite",
        "client_localite",
        CLIENT,
    ),
    exempte(
        "kesh_contacts",
        "contacts[].pays",
        "deux caractères (CHAR(2)) : aucun piège n'y tient ; enveloppé quand même",
    ),
    nf("kesh_contacts", "contacts[].email", "client_email", CLIENT),
    nf(
        "kesh_contacts",
        "contacts[].telephone",
        "client_telephone",
        CLIENT,
    ),
    nf("kesh_contacts", "contacts[].ide", "client_ide", CLIENT),
    nf(
        "kesh_contacts",
        "contacts[].numeroClient",
        "client_numero",
        CLIENT,
    ),
    nf(
        "kesh_contacts",
        "contacts[].conditions",
        "client_conditions",
        CLIENT,
    ),
    // documents:read (32-9b) — les métadonnées de P2 ; les champs de son QR,
    // posés, ne sortent jamais (`pieces_sans_donnees_du_qr`).
    nf(
        "kesh_pieces",
        "pieces[].nomFichier",
        "piece_nom_fichier",
        &[("kesh_pieces", 0)],
    ),
    nf(
        "kesh_pieces",
        "pieces[].typeMime",
        "piece_type_mime",
        &[("kesh_pieces", 0)],
    ),
];

/// Les jeux qui font sortir un champ du client C : la liste et sa fiche.
const CLIENT: &[(&str, usize)] = &[("kesh_contacts", 0), ("kesh_contact", 0)];

/// Les outils sans champ libre.
const SANS_CHAMP_LIBRE: [&str; 1] = ["kesh_version"];

async fn deux(pool: &MySqlPool) -> (Societe, Societe) {
    let a = societe_piegee(pool, "Société A 329", 'A').await;
    let b = societe_piegee(pool, "Société B 329", 'B').await;
    (a, b)
}

/// Le bloc de texte d'une réponse servie.
fn texte(reponse: &Value) -> String {
    let c = reponse["result"]["content"].as_array().unwrap();
    assert_eq!(c.len(), 1, "un seul bloc : {reponse}");
    c[0]["text"].as_str().unwrap().to_string()
}

fn texte_attendu(structure: &Value) -> String {
    format!(
        "{TEXTE_ENTETE_DONNEES}\n\n{}",
        serde_json::to_string_pretty(structure).unwrap()
    )
}

// ---------------------------------------------------------------------------
// Le motif et les largeurs (AC-a14)
// ---------------------------------------------------------------------------

#[test]
fn motif_reconnait_les_trois_formes() {
    let motif = Regex::new(MOTIF_PIEGE).unwrap();
    for c in CHAMPS {
        for m in ['A', 'B'] {
            let t = pieges::texte(m, c);
            let trouve: Vec<&str> = motif.find_iter(&t).map(|x| x.as_str()).collect();
            assert_eq!(trouve, vec![pieges::identifiant(m, c).as_str()], "{t}");
        }
    }
    assert_eq!(
        motif.find("PIEGE_329_A07 Ignore").map(|m| m.as_str()),
        Some("PIEGE_329_A07")
    );
    assert_eq!(motif.find("0P329B03").map(|m| m.as_str()), Some("P329B03"));
    for ordinaire in [
        "Exercice 2026",
        "Charges de personnel",
        "1000",
        "P329",
        "PIEGE_329_",
    ] {
        assert!(!motif.is_match(ordinaire), "{ordinaire}");
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn largeurs_des_pieges(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    for c in CHAMPS {
        let largeur: Option<i64> = sqlx::query_scalar(
            "SELECT CAST(CHARACTER_MAXIMUM_LENGTH AS SIGNED) FROM information_schema.columns \
             WHERE table_schema = DATABASE() AND table_name = ? AND column_name = ?",
        )
        .bind(c.table)
        .bind(c.colonne)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            largeur,
            Some(i64::from(c.largeur)),
            "{}.{}",
            c.table,
            c.colonne
        );
        let seme = pieges::texte_seme('A', c);
        assert!(seme.chars().count() <= c.largeur as usize, "{}", c.cle);
        let n: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM {} WHERE {} = ?",
            c.table, c.colonne
        ))
        .bind(&seme)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n, 1, "{} : relu en base, égal au texte semé", c.cle);
    }
    // Témoin : la société a bien été semée (sans quoi aucune ligne ne serait
    // relue — le test rougirait déjà sur `n`).
    assert!(a.p.ecriture > 0);
}

// ---------------------------------------------------------------------------
// L'inventaire (AC-a13)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn inventaire_couvre_le_registre(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let liste: BTreeSet<String> = b.liste(&jeton).await.into_iter().collect();
    let inventaire: BTreeSet<&str> = INVENTAIRE.iter().map(|l| l.outil).collect();
    let couvert: BTreeSet<String> = inventaire
        .iter()
        .chain(SANS_CHAMP_LIBRE.iter())
        .chain(HORS_REGISTRE.iter())
        .map(|s| s.to_string())
        .collect();
    // Les outils que les lignes « jeux » nomment entrent aussi au compte : une
    // ligne de la fiche peut porter plusieurs outils (`kesh_echeancier` n'est
    // le champ `outil` d'aucune ligne, il n'y entre que par les jeux des lignes
    // `kesh_factures`) — écart à la lettre de l'AC-a13, dit ici (revue de code
    // P1, A10).
    let nommes_par_jeux: BTreeSet<&str> = INVENTAIRE
        .iter()
        .flat_map(|l| l.jeux.iter().map(|(o, _)| *o))
        .collect();
    let mut par_jeux: BTreeSet<String> = couvert.clone();
    par_jeux.extend(nommes_par_jeux.iter().map(|o| o.to_string()));
    assert_eq!(
        par_jeux, liste,
        "inventaire ∪ sans champ libre ∪ hors registre"
    );
    // « Aucun outil dans deux listes » porte aussi sur les outils nommés par
    // les jeux (A10).
    for o in SANS_CHAMP_LIBRE.iter().chain(HORS_REGISTRE.iter()) {
        assert!(!inventaire.contains(o), "{o} dans deux listes");
        assert!(
            !nommes_par_jeux.contains(o),
            "{o} nommé par un jeu et sans champ libre ou hors registre"
        );
    }
    assert!(!SANS_CHAMP_LIBRE.iter().any(|o| HORS_REGISTRE.contains(o)));
    for l in INVENTAIRE {
        assert!(
            l.piege.is_some() != l.exemption.is_some(),
            "{} {} : un piège ou une exemption",
            l.outil,
            l.champ
        );
        assert_eq!(
            l.piege.is_some(),
            !l.jeux.is_empty(),
            "{} {}",
            l.outil,
            l.champ
        );
        for (o, i) in l.jeux {
            assert!(
                *i < (temoin(o).1)(&a.p).len(),
                "{o} jeu {i} : existe dans TEMOINS"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Les pièges sortent enveloppés (AC-a15)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pieges_toujours_enveloppes(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let motif = motif();
    for (outil, jeux, _) in TEMOINS {
        for args in jeux(&a.p) {
            let r = b.appeler(&jeton, outil, args.clone()).await;
            let s = structure(&r, outil);
            assert_eq!(fautes(&s, &motif), Vec::<String>::new(), "{outil} {args}");
            assert_eq!(texte(&r), texte_attendu(&s), "{outil} : une seule source");
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pieges_enveloppes_au_depot(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let exempte = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let motif = motif();
    for (outil, _, _) in TEMOINS {
        // Une clé par outil : le plafond de vingt envois en attente par clé.
        let requise = cle_de(&pool, &a, &LECTURE, SendValidation::Required).await;
        let args = premier_jeu(outil, &a.p);
        let servi = texte(&b.appeler(&exempte, outil, args.clone()).await);
        let r = b.mcp(&requise, &common::mcp::appel(1, outil, args)).await;
        let n = numero_depose(&r);
        let (contenu, resume): (Option<String>, Option<String>) =
            sqlx::query_as("SELECT content, summary FROM mcp_sends WHERE id = ?")
                .bind(n)
                .fetch_one(&pool)
                .await
                .unwrap();
        let contenu = contenu.expect("contenu déposé");
        assert_eq!(contenu, servi, "{outil} : le dépôt égale le texte servi");
        let json = contenu
            .strip_prefix(&format!("{TEXTE_ENTETE_DONNEES}\n\n"))
            .expect("en-tête");
        let v: Value = serde_json::from_str(json).unwrap();
        assert_eq!(fautes(&v, &motif), Vec::<String>::new(), "{outil}");
        let resume = resume.expect("résumé");
        assert!(!motif.is_match(&resume), "{outil} : résumé {resume}");
        assert!(resume.starts_with(&format!("Outil {outil}")), "{resume}");
    }
}

/// Le témoin que le parcours ne passe pas à vide : chaque ligne à piège voit
/// **son** identifiant, enveloppé, dans la sortie de **son** outil.
pub async fn verifier_inventaire(b: &Banc, jeton: &str, s: &Societe, marque: char) {
    let motif = motif();
    for l in INVENTAIRE {
        let Some(cle) = l.piege else { continue };
        let id = pieges::identifiant(marque, par_cle(cle));
        for (outil, i) in l.jeux {
            let args = (temoin(outil).1)(&s.p).remove(*i);
            let r = b.appeler(jeton, outil, args.clone()).await;
            let trouves = identifiants_enveloppes(&structure(&r, outil), &motif);
            assert!(
                trouves.contains(&id),
                "{} / {} : {id} absent de {outil} {args} (trouvés : {trouves:?})",
                l.outil,
                l.champ
            );
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn inventaire_couvert(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    verifier_inventaire(&banc(&pool), &jeton, &a, 'A').await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn caracteres_caches_neutralises(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let r = banc(&pool)
        .appeler(&jeton, "kesh_ecriture", json!({"id": a.p.ecriture}))
        .await;
    let libelle = structure(&r, "kesh_ecriture")["ecriture"]["libelle"]["nonFiable"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        libelle.contains("\\u{202E}") && libelle.contains("\\u{E0049}"),
        "{libelle}"
    );
    assert!(
        !libelle.chars().any(kesh_core::text::is_hidden_in_text),
        "{libelle:?}"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn evasion_json_impossible(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let r = banc(&pool)
        .appeler(&jeton, "kesh_facture", json!({"id": a.p.facture}))
        .await;
    let ligne = &structure(&r, "kesh_facture")["lignes"][0]["description"];
    let seme = pieges::texte_seme('A', par_cle("facture_ligne"));
    assert!(seme.ends_with(EVASION));
    assert_eq!(ligne, &json!({ "nonFiable": seme }));
}

// ---------------------------------------------------------------------------
// Les tiers (AC-a16)
// ---------------------------------------------------------------------------

/// Les pièges des lignes `Tiers` de l'inventaire — trois : l'adresse d'envoi,
/// l'IBAN et le QR-IBAN du créancier.
fn coordonnees() -> Vec<&'static str> {
    let v: Vec<&str> = INVENTAIRE
        .iter()
        .filter(|l| l.traitement == Traitement::Tiers)
        .filter_map(|l| l.piege)
        .collect();
    assert_eq!(v, ["facture_envoyee_a", "ff_iban", "ff_qr_iban"]);
    v
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn coordonnees_masquees_sans_contacts_read(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &[McpScope::InvoicesRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let masque = json!(TEXTE_TIERS_MASQUE);
    let f = structure(
        &b.appeler(&jeton, "kesh_facture", json!({"id": a.p.facture}))
            .await,
        "kesh_facture",
    );
    assert_eq!(f["envoyeeA"], masque);
    let ff = structure(
        &b.appeler(
            &jeton,
            "kesh_facture_fournisseur",
            json!({"id": a.p.facture_fournisseur}),
        )
        .await,
        "kesh_facture_fournisseur",
    );
    assert_eq!(ff["ibanCreancier"], masque);
    assert_eq!(ff["qrIbanCreancier"], masque);
    // Sous cette clé, aucun outil ne fait sortir ces trois pièges.
    for outil in b.liste(&jeton).await {
        if HORS_REGISTRE.contains(&outil.as_str()) {
            continue;
        }
        for args in (temoin(&outil).1)(&a.p) {
            let r = b.appeler(&jeton, &outil, args).await;
            let t = serde_json::to_string(&r).unwrap();
            for c in coordonnees() {
                let id = pieges::identifiant('A', par_cle(c));
                assert!(!t.contains(&id), "{outil} : {id} sorti sans contacts:read");
            }
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn coordonnees_visibles_avec_contacts_read(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(
        &pool,
        &a,
        &[McpScope::InvoicesRead, McpScope::ContactsRead],
        SendValidation::Exempt,
    )
    .await;
    let b = banc(&pool);
    let f = structure(
        &b.appeler(&jeton, "kesh_facture", json!({"id": a.p.facture}))
            .await,
        "kesh_facture",
    );
    assert_eq!(
        f["envoyeeA"],
        json!({"nonFiable": pieges::texte_seme('A', par_cle("facture_envoyee_a"))})
    );
    let ff = structure(
        &b.appeler(
            &jeton,
            "kesh_facture_fournisseur",
            json!({"id": a.p.facture_fournisseur}),
        )
        .await,
        "kesh_facture_fournisseur",
    );
    assert_eq!(
        ff["ibanCreancier"],
        json!({"nonFiable": pieges::texte_seme('A', par_cle("ff_iban"))})
    );
    assert_eq!(
        ff["qrIbanCreancier"],
        json!({"nonFiable": pieges::texte_seme('A', par_cle("ff_qr_iban"))})
    );
}

/// Les pièges des champs d'un contact que seul `contacts:read` rend — tous
/// ceux du client C, **hors** son nom (que les factures et les libellés
/// portent, C-32-9-9). Lus de `CHAMPS` : un piège de contact ajouté y entre.
fn champs_du_contact() -> Vec<&'static str> {
    let v: Vec<&str> = CHAMPS
        .iter()
        .filter(|c| c.table == "contacts" && c.colonne != "name")
        .map(|c| c.cle)
        .collect();
    assert_eq!(v.len(), 11, "témoin : onze champs du client piégés ({v:?})");
    v
}

/// #781, point 3 (revue de code P1 de la 32-9a, B5) : le masquage éprouvé sous
/// les portées **combinées** sans `contacts:read` — livres, factures et pièces
/// ensemble. Les trois coordonnées des tiers sont masquées dans les fiches ;
/// et, sous cette clé, **aucun** outil — `kesh_pieces` compris — ne fait sortir
/// ni ces trois pièges, ni aucun piège d'un champ du contact autre que son nom.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn coordonnees_masquees_sous_portees_combinees(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let portees = [
        McpScope::BooksRead,
        McpScope::InvoicesRead,
        McpScope::DocumentsRead,
    ];
    let jeton = cle_de(&pool, &a, &portees, SendValidation::Exempt).await;
    let b = banc(&pool);
    let masque = json!(TEXTE_TIERS_MASQUE);
    let f = structure(
        &b.appeler(&jeton, "kesh_facture", json!({"id": a.p.facture}))
            .await,
        "kesh_facture",
    );
    assert_eq!(f["envoyeeA"], masque);
    let ff = structure(
        &b.appeler(
            &jeton,
            "kesh_facture_fournisseur",
            json!({"id": a.p.facture_fournisseur}),
        )
        .await,
        "kesh_facture_fournisseur",
    );
    assert_eq!(ff["ibanCreancier"], masque);
    assert_eq!(ff["qrIbanCreancier"], masque);
    let interdits: Vec<String> = coordonnees()
        .into_iter()
        .chain(champs_du_contact())
        .map(|c| pieges::identifiant('A', par_cle(c)))
        .collect();
    let motif = motif();
    let mut outils_vus = BTreeSet::new();
    let mut trouves_ailleurs = 0;
    for outil in b.liste(&jeton).await {
        if HORS_REGISTRE.contains(&outil.as_str()) {
            continue;
        }
        outils_vus.insert(outil.clone());
        for args in (temoin(&outil).1)(&a.p) {
            let r = b.appeler(&jeton, &outil, args.clone()).await;
            let t = serde_json::to_string(&r).unwrap();
            for id in &interdits {
                assert!(
                    !t.contains(id),
                    "{outil} {args} : {id} sorti sans contacts:read"
                );
            }
            trouves_ailleurs += motif.find_iter(&t).count();
        }
    }
    // Témoins : la clé voit bien les outils des trois portées, et des pièges
    // sortent (ceux qu'elle a le droit de lire).
    for o in ["kesh_bilan", "kesh_facture", "kesh_pieces"] {
        assert!(outils_vus.contains(o), "{o} servi sous la clé combinée");
    }
    assert!(!outils_vus.contains("kesh_contacts"));
    assert!(
        trouves_ailleurs > 20,
        "témoin : {trouves_ailleurs} pièges vus"
    );
}

/// AC-b3 — la pièce sans les données de son QR-code : sous toutes les portées
/// de lecture, `MOTIF_PIEGE` ne trouve **aucun** des pièges posés dans le QR de
/// P2 dans la réponse **entière** d'aucun outil — `kesh_pieces` d'abord ; et
/// `qrLu` vaut `true` pour P2 (témoin que la pièce est lue, et que son QR
/// existe), son nom de fichier sortant, enveloppé (témoin du parcours).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pieces_sans_donnees_du_qr(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let jeton = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let b = banc(&pool);
    let motif = motif();
    let du_qr: BTreeSet<String> = pieges::CHAMPS_DU_QR
        .iter()
        .map(|c| pieges::identifiant('A', par_cle(c)))
        .collect();
    assert_eq!(du_qr.len(), 9, "témoin : neuf pièges dans le QR");
    let r = b
        .appeler(&jeton, "kesh_pieces", json!({"statut": "to_complete"}))
        .await;
    let s = structure(&r, "kesh_pieces");
    let p2: Vec<&Value> = s["pieces"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["id"] == json!(a.p.piece_qr))
        .collect();
    assert_eq!(p2.len(), 1, "P2 listée : {s}");
    assert_eq!(p2[0]["qrLu"], json!(true));
    assert_eq!(
        p2[0]["nomFichier"],
        json!({"nonFiable": pieges::texte_seme('A', par_cle("piece_nom_fichier"))})
    );
    for (outil, jeux, _) in TEMOINS {
        for args in jeux(&a.p) {
            let r = b.appeler(&jeton, outil, args.clone()).await;
            let t = serde_json::to_string(&r).unwrap();
            let vus: BTreeSet<String> = motif
                .find_iter(&t)
                .map(|m| m.as_str().to_string())
                .collect();
            let fuites: Vec<&String> = vus.intersection(&du_qr).collect();
            assert!(fuites.is_empty(), "{outil} {args} : {fuites:?}");
        }
    }
    // La pièce rattachée (sans QR) : `qrLu` faux, sous le filtre `completed`.
    let s = structure(
        &b.appeler(&jeton, "kesh_pieces", json!({"statut": "completed"}))
            .await,
        "kesh_pieces",
    );
    let p1: Vec<&Value> = s["pieces"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["id"] == json!(a.p.piece))
        .collect();
    assert_eq!(p1.len(), 1, "P1 listée sous completed : {s}");
    assert_eq!(p1[0]["qrLu"], json!(false));
    assert_eq!(
        p1[0]["factureFournisseurId"],
        json!(a.p.facture_fournisseur)
    );
}

/// Constat de la conception (C-32-9-9) : les noms des tiers ne sont pas
/// masqués — Kesh les écrit lui-même dans les libellés. Si ce test rougit, la
/// phrase du manuel (§ *Les outils et leurs portées*) ment.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nom_du_tiers_non_masque(pool: MySqlPool) {
    let a = societe_piegee(&pool, "Société A 329", 'A').await;
    let livres = cle_de(&pool, &a, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let factures = cle_de(&pool, &a, &[McpScope::InvoicesRead], SendValidation::Exempt).await;
    let b = banc(&pool);
    let motif = motif();
    let fournisseur = pieges::identifiant('A', par_cle("fournisseur_nom"));
    let client = pieges::identifiant('A', par_cle("client_nom"));
    for (outil, i) in [
        ("kesh_ecritures", 1),
        ("kesh_grand_livre", 1),
        ("kesh_postes_ouverts", 1),
    ] {
        let args = (temoin(outil).1)(&a.p).remove(i);
        let s = structure(&b.appeler(&livres, outil, args).await, outil);
        assert!(
            identifiants_enveloppes(&s, &motif).contains(&fournisseur),
            "{outil}"
        );
    }
    let s = structure(
        &b.appeler(&factures, "kesh_factures", json!({})).await,
        "kesh_factures",
    );
    assert!(identifiants_enveloppes(&s, &motif).contains(&client));
    let s = structure(
        &b.appeler(&factures, "kesh_factures_fournisseur", json!({}))
            .await,
        "kesh_factures_fournisseur",
    );
    assert!(identifiants_enveloppes(&s, &motif).contains(&fournisseur));
}

/// AC-a20 (témoin vert de `outils_bornes_a_la_societe`, `mcp_outils_e2e.rs`) :
/// les deux sociétés ont chacune leur inventaire couvert, sous leur clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn inventaire_couvert_dans_chaque_societe(pool: MySqlPool) {
    let (a, b_soc) = deux(&pool).await;
    let b = banc(&pool);
    let ja = cle_de(&pool, &a, &LECTURE, SendValidation::Exempt).await;
    let jb = cle_de(&pool, &b_soc, &LECTURE, SendValidation::Exempt).await;
    verifier_inventaire(&b, &ja, &a, 'A').await;
    verifier_inventaire(&b, &jb, &b_soc, 'B').await;
}
