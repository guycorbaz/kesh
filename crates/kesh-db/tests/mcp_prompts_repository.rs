//! Story 32-10a-i (refs #723) — les tables des prompts du serveur MCP : leurs
//! contraintes (AC-i3), la version en vigueur (AC-i6), l'approbation et le
//! refus (AC-i7), les noms et le plafond (AC-i8), l'amorçage des standards
//! (AC-i10), et les lectures du dépôt (AC-i5).
//!
//! Les tests de contrainte écrivent en SQL brut, sans passer par le dépôt :
//! c'est MariaDB qui doit refuser. Les standards de l'amorçage sont des
//! standards **de test**, passés en argument, sauf mention.
//!
//! Fixtures synthétiques seulement (`PIEGE_*_3210A`).

use std::time::Duration;

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::mcp_prompt::{
    Contenu, ErreurEcriturePrompt, Etat, Nature, Origine, PLAFOND_A_VALIDER,
};
use kesh_db::entities::mcp_prompt_standards::{STANDARDS, Standard};
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::mcp_prompts as depot;
use kesh_db::repositories::{companies, users};
use sqlx::MySqlPool;

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

async fn company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

async fn user(pool: &MySqlPool, company_id: i64, username: &str) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id
}

async fn key(pool: &MySqlPool, company_id: i64, user_id: i64, nom: &str) -> i64 {
    sqlx::query(
        "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes) \
         VALUES (?, ?, ?, SHA2(?, 256), '[\"prompts:write\"]')",
    )
    .bind(company_id)
    .bind(user_id)
    .bind(nom)
    .bind(nom)
    .execute(pool)
    .await
    .expect("clé MCP")
    .last_insert_id() as i64
}

struct Montage {
    c: i64,
    u: i64,
    k: i64,
}

async fn montage(pool: &MySqlPool, nom: &str) -> Montage {
    let c = company(pool, nom).await;
    let u = user(pool, c, &format!("admin-{nom}")).await;
    let k = key(pool, c, u, &format!("cle-{nom}")).await;
    Montage { c, u, k }
}

const S1: Standard = Standard {
    nom: "kesh-un",
    description: "Standard un.",
    texte: "Texte du standard un.",
};
const S1_MODIFIE: Standard = Standard {
    nom: "kesh-un",
    description: "Standard un.",
    texte: "Texte du standard un, modifié par le code.",
};
const S2: Standard = Standard {
    nom: "kesh-deux",
    description: "Standard deux.",
    texte: "Texte du standard deux.",
};

fn texte(d: &str, t: &str) -> Contenu {
    Contenu::Texte {
        description: d.into(),
        texte: t.into(),
    }
}

async fn creer_admin(pool: &MySqlPool, m: &Montage, nom: &str) -> i64 {
    let mut tx = pool.begin().await.unwrap();
    let e = depot::creer_prompt_admin_in_tx(&mut tx, m.c, m.u, nom, "Description.", "Texte.")
        .await
        .expect("création");
    tx.commit().await.unwrap();
    e.prompt_id
}

async fn proposer(
    pool: &MySqlPool,
    m: &Montage,
    nom: &str,
    t: &str,
) -> Result<kesh_db::entities::mcp_prompt::VersionEcrite, ErreurEcriturePrompt> {
    let mut tx = pool.begin().await.unwrap();
    let r = depot::proposer_version_agent_in_tx(
        &mut tx,
        m.c,
        m.k,
        nom,
        "Proposition.",
        t,
        PLAFOND_A_VALIDER,
    )
    .await;
    match &r {
        Ok(_) => tx.commit().await.unwrap(),
        Err(_) => tx.rollback().await.unwrap(),
    }
    r
}

async fn version_du_prompt(pool: &MySqlPool, prompt_id: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM mcp_prompts WHERE id = ?")
        .bind(prompt_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn nb_versions(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM mcp_prompt_versions")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Les lignes `(number, origin, state, nature, body)` d'un prompt, par numéro.
async fn versions_de(
    pool: &MySqlPool,
    prompt_id: i64,
) -> Vec<(i32, String, String, String, Option<String>)> {
    sqlx::query_as(
        "SELECT number, origin, state, nature, body FROM mcp_prompt_versions \
         WHERE prompt_id = ? ORDER BY number",
    )
    .bind(prompt_id)
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn prompt_par_nom(pool: &MySqlPool, company_id: i64, nom: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM mcp_prompts WHERE company_id = ? AND name = ?")
        .bind(company_id)
        .bind(nom)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Décide en une transaction ; rend l'erreur, ou `Ok(())`.
async fn decider(
    pool: &MySqlPool,
    m: &Montage,
    prompt_id: i64,
    numero: i32,
    version: i32,
    approuver: bool,
) -> Result<(), ErreurEcriturePrompt> {
    let mut tx = pool.begin().await.unwrap();
    let r = if approuver {
        depot::approuver_in_tx(&mut tx, m.c, prompt_id, numero, version, m.u).await
    } else {
        depot::refuser_in_tx(&mut tx, m.c, prompt_id, numero, version, m.u).await
    };
    match r {
        Ok(_) => {
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// AC-i3 — contraintes
// ---------------------------------------------------------------------------

async fn brut_prompt(pool: &MySqlPool, c: i64, name: &str, std: bool) -> Result<i64, String> {
    match sqlx::query("INSERT INTO mcp_prompts (company_id, name, is_standard) VALUES (?, ?, ?)")
        .bind(c)
        .bind(name)
        .bind(std)
        .execute(pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_id() as i64),
        Err(sqlx::Error::Database(e)) => Err(e.message().to_string()),
        Err(e) => panic!("{e}"),
    }
}

#[allow(clippy::too_many_arguments)]
async fn brut_version(
    pool: &MySqlPool,
    prompt_id: i64,
    number: i32,
    origin: &str,
    state: &str,
    nature: &str,
    description: Option<&str>,
    body: Option<&str>,
    author_user: Option<i64>,
    author_key: Option<i64>,
    decided_by: Option<i64>,
    decided_at: bool,
) -> Result<(), String> {
    let at = if decided_at { "NOW(3)" } else { "NULL" };
    match sqlx::query(&format!(
        "INSERT INTO mcp_prompt_versions (prompt_id, number, origin, state, nature, description, \
         body, author_user_id, author_mcp_key_id, decided_by_user_id, decided_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, {at})"
    ))
    .bind(prompt_id)
    .bind(number)
    .bind(origin)
    .bind(state)
    .bind(nature)
    .bind(description)
    .bind(body)
    .bind(author_user)
    .bind(author_key)
    .bind(decided_by)
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(e)) => Err(e.message().to_string()),
        Err(e) => panic!("{e}"),
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn mcp_prompts_contraintes(pool: MySqlPool) {
    let m = montage(&pool, "contraintes").await;
    let p = brut_prompt(&pool, m.c, "libre", false).await.expect("témoin");
    let s = brut_prompt(&pool, m.c, "kesh-std", true).await.expect("témoin");

    // Les prompts : chaque cas refusé.
    let soixante_cinq = "a".repeat(65);
    let refus_prompts: &[(&str, bool, &str)] = &[
        ("Valide", false, "majuscule"),
        ("valide\n", false, "saut de ligne final (R7)"),
        ("valide ", false, "blanc final"),
        ("", false, "vide"),
        (&soixante_cinq, false, "65 caractères"),
        ("kesh-x", false, "préfixe réservé, non standard"),
        ("keshx", false, "kesh sans tiret, non standard"),
        ("kesh_x", false, "kesh souligné, non standard"),
        ("autre", true, "standard sans préfixe"),
        ("libre", false, "doublon (company, name)"),
    ];
    for (nom, std, cas) in refus_prompts {
        assert!(
            brut_prompt(&pool, m.c, nom, *std).await.is_err(),
            "accepté : {cas}"
        );
    }
    assert!(
        brut_prompt(&pool, 999_999, "orphelin", false).await.is_err(),
        "accepté : société inexistante"
    );

    // Les versions : chaque cas refusé. Numéros distincts, pour qu'aucun cas ne
    // soit refusé par l'unicité à la place de la contrainte visée.
    let (u, k) = (Some(m.u), Some(m.k));
    let d = Some("Description.");
    let b = Some("Texte.");
    let long = "a".repeat(16_001);
    let desc_longue = "a".repeat(501);
    #[allow(clippy::type_complexity)]
    let refus_versions: Vec<(
        &str,
        &str,
        &str,
        Option<&str>,
        Option<&str>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        bool,
        &str,
    )> = vec![
        ("Standard", "approved", "text", d, b, None, None, None, false, "origine en capitale"),
        ("standard", "approved ", "text", d, b, None, None, None, false, "état à blanc final"),
        ("standard", "approved", "Text", d, b, None, None, None, false, "nature en capitale"),
        ("standard", "pending", "text", d, b, None, None, None, false, "standard ⇒ approved"),
        ("standard", "approved", "text", d, b, u, None, None, false, "standard avec auteur"),
        ("standard", "approved", "text", d, b, None, None, u, true, "standard avec décision"),
        ("admin", "approved", "text", d, b, None, None, None, false, "admin sans auteur"),
        ("admin", "pending", "text", d, b, u, None, None, false, "admin ⇒ approved"),
        ("admin", "approved", "text", d, b, u, k, None, false, "admin avec clé"),
        ("admin", "approved", "text", d, b, u, None, u, true, "admin avec décision"),
        ("agent", "pending", "text", d, b, None, None, None, false, "agent sans clé"),
        ("agent", "pending", "text", d, b, u, k, None, false, "agent avec utilisateur"),
        ("agent", "pending", "withdrawal", None, None, None, k, None, false, "agent retrait"),
        ("agent", "pending", "text", d, b, None, k, u, true, "agent pending décidé"),
        ("agent", "approved", "text", d, b, None, k, None, false, "agent approved sans décision"),
        ("agent", "refused", "text", d, b, None, k, u, false, "décision sans date"),
        ("agent", "approved", "text", d, b, None, k, None, true, "date sans décideur"),
        ("admin", "approved", "text", d, None, u, None, None, false, "texte sans corps"),
        ("admin", "approved", "text", None, b, u, None, None, false, "texte sans description"),
        ("admin", "approved", "text", Some(" \t\n"), b, u, None, None, false, "description blanche"),
        ("admin", "approved", "text", d, Some("  \n "), u, None, None, false, "corps blanc"),
        ("admin", "approved", "withdrawal", None, b, u, None, None, false, "retrait avec corps"),
        ("admin", "approved", "withdrawal", d, None, u, None, None, false, "retrait avec description"),
        ("admin", "approved", "text", d, Some(&long), u, None, None, false, "corps > 16000"),
        ("admin", "approved", "text", Some(&desc_longue), b, u, None, None, false, "description > 500"),
    ];
    let mut numero = 100;
    for (o, st, n, de, bo, au, ke, dec, at, cas) in &refus_versions {
        numero += 1;
        assert!(
            brut_version(&pool, p, numero, o, st, n, *de, *bo, *au, *ke, *dec, *at)
                .await
                .is_err(),
            "accepté : {cas}"
        );
    }
    // Unicité, numéro positif, clés étrangères.
    brut_version(&pool, p, 1, "admin", "approved", "text", d, b, u, None, None, false)
        .await
        .expect("témoin du doublon");
    let structurels = [
        (
            brut_version(&pool, p, 1, "admin", "approved", "text", d, b, u, None, None, false)
                .await,
            "doublon (prompt_id, number)",
        ),
        (
            brut_version(&pool, p, 0, "admin", "approved", "text", d, b, u, None, None, false)
                .await,
            "numéro 0",
        ),
        (
            brut_version(&pool, 999_999, 1, "admin", "approved", "text", d, b, u, None, None, false)
                .await,
            "prompt inexistant",
        ),
        (
            brut_version(&pool, p, 50, "admin", "approved", "text", d, b, Some(999_999), None, None, false)
                .await,
            "auteur inexistant",
        ),
        (
            brut_version(&pool, p, 51, "agent", "pending", "text", d, b, None, Some(999_999), None, false)
                .await,
            "clé inexistante",
        ),
    ];
    for (r, cas) in &structurels {
        assert!(r.is_err(), "accepté : {cas}");
    }
    // La liste est comptée : un cas retiré par mégarde se voit.
    assert_eq!(
        refus_prompts.len() + 1 + refus_versions.len() + structurels.len(),
        10 + 1 + 25 + 5
    );

    // Témoins : chaque combinaison permise passe.
    let permis = [
        brut_version(&pool, s, 1, "standard", "approved", "text", d, b, None, None, None, false).await,
        brut_version(&pool, s, 2, "standard", "approved", "withdrawal", None, None, None, None, None, false).await,
        brut_version(&pool, p, 2, "admin", "approved", "withdrawal", None, None, u, None, None, false).await,
        brut_version(&pool, p, 3, "agent", "pending", "text", d, b, None, k, None, false).await,
        brut_version(&pool, p, 4, "agent", "approved", "text", d, b, None, k, u, true).await,
        brut_version(&pool, p, 5, "agent", "refused", "text", d, b, None, k, u, true).await,
        brut_version(&pool, p, 6, "admin", "approved", "text", Some(&"é".repeat(500)), Some(&"é".repeat(16_000)), u, None, None, false).await,
    ];
    for (i, r) in permis.iter().enumerate() {
        assert!(r.is_ok(), "témoin {i} refusé : {r:?}");
    }
    let tous: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_prompt_versions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(tous, 8, "le doublon témoin et les sept combinaisons permises");
}

// ---------------------------------------------------------------------------
// AC-i6 — la version en vigueur
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn version_en_vigueur(pool: MySqlPool) {
    let m = montage(&pool, "vigueur").await;
    let p = creer_admin(&pool, &m, "aide").await;
    // Une proposition postérieure, à valider, ne change pas la version en
    // vigueur ; refusée, non plus.
    proposer(&pool, &m, "aide", "PIEGE_PENDING_3210A").await.unwrap();
    let v = depot::en_vigueur_par_nom(&pool, m.c, "aide").await.unwrap().unwrap();
    assert_eq!((v.numero, v.origine, v.texte.as_str()), (1, Origine::Admin, "Texte."));
    decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, false)
        .await
        .unwrap();
    let v = depot::en_vigueur_par_nom(&pool, m.c, "aide").await.unwrap().unwrap();
    assert_eq!(v.numero, 1);
    // Une proposition approuvée devient la version en vigueur.
    proposer(&pool, &m, "aide", "Texte de l'agent.").await.unwrap();
    decider(&pool, &m, p, 3, version_du_prompt(&pool, p).await, true)
        .await
        .unwrap();
    let v = depot::en_vigueur_par_nom(&pool, m.c, "aide").await.unwrap().unwrap();
    assert_eq!((v.numero, v.origine, v.cle_auteur), (3, Origine::Agent, Some(m.k)));
    // Un retrait l'ôte des versions servies.
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, p, version_du_prompt(&pool, p).await, m.u, &Contenu::Retrait)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(depot::en_vigueur_par_nom(&pool, m.c, "aide").await.unwrap().is_none());
    assert!(depot::en_vigueur_par_societe(&pool, m.c).await.unwrap().is_empty());
}

#[sqlx::test(migrations = "./test-schema")]
async fn deux_societes_ne_se_voient_pas(pool: MySqlPool) {
    let a = montage(&pool, "a").await;
    let b = montage(&pool, "b").await;
    creer_admin(&pool, &a, "commun").await;
    let mut tx = pool.begin().await.unwrap();
    depot::creer_prompt_admin_in_tx(&mut tx, b.c, b.u, "commun", "De B.", "PIEGE_B_3210A")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let va = depot::en_vigueur_par_nom(&pool, a.c, "commun").await.unwrap().unwrap();
    assert_eq!(va.texte, "Texte.");
    let liste_a = depot::en_vigueur_par_societe(&pool, a.c).await.unwrap();
    assert_eq!(liste_a.len(), 1);
    assert_eq!(liste_a[0].texte, "Texte.");
    // Un nom d'une autre société, ou non conforme, ne rend rien.
    let mut tx = pool.begin().await.unwrap();
    depot::creer_prompt_admin_in_tx(&mut tx, b.c, b.u, "seulement-b", "D.", "T.")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(depot::en_vigueur_par_nom(&pool, a.c, "seulement-b").await.unwrap().is_none());
    assert!(depot::en_vigueur_par_nom(&pool, a.c, "COMMUN").await.unwrap().is_none());
    // L'historique d'un prompt d'une autre société : indiscernable d'un absent.
    let pb = prompt_par_nom(&pool, b.c, "commun").await;
    assert!(depot::historique(&pool, a.c, pb).await.unwrap().is_none());
    assert!(depot::historique(&pool, b.c, pb).await.unwrap().is_some());
}

// ---------------------------------------------------------------------------
// AC-i7 — approuver et refuser
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn approuver_exige_la_derniere_version(pool: MySqlPool) {
    let m = montage(&pool, "derniere").await;
    let p = creer_admin(&pool, &m, "aide").await;
    proposer(&pool, &m, "aide", "Proposé.").await.unwrap();
    // Une version d'administrateur vient entre-temps.
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, p, version_du_prompt(&pool, p).await, m.u, &texte("D.", "Admin."))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let avant = versions_de(&pool, p).await;
    let r = decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, true).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::PasLaDerniere)), "{r:?}");
    assert_eq!(versions_de(&pool, p).await, avant, "rien d'écrit");
    // Le refus, lui, ne l'exige pas.
    decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, false)
        .await
        .unwrap();
    assert_eq!(versions_de(&pool, p).await[1].2, "refused");
}

#[sqlx::test(migrations = "./test-schema")]
async fn etats_terminaux_et_verrou(pool: MySqlPool) {
    let m = montage(&pool, "terminaux").await;
    let p = creer_admin(&pool, &m, "aide").await;
    // Verrou périmé.
    proposer(&pool, &m, "aide", "Un.").await.unwrap();
    let v = version_du_prompt(&pool, p).await;
    let r = decider(&pool, &m, p, 2, v - 1, true).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::VersionPerimee)), "{r:?}");
    let r = decider(&pool, &m, p, 2, v + 1, false).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::VersionPerimee)), "{r:?}");
    // Approuvée puis re-décidée : refusé dans les deux sens.
    decider(&pool, &m, p, 2, v, true).await.unwrap();
    for approuver in [true, false] {
        let r = decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, approuver).await;
        assert!(matches!(r, Err(ErreurEcriturePrompt::DejaDecidee)), "{r:?}");
    }
    // Refusée puis re-décidée.
    proposer(&pool, &m, "aide", "Deux.").await.unwrap();
    decider(&pool, &m, p, 3, version_du_prompt(&pool, p).await, false)
        .await
        .unwrap();
    for approuver in [true, false] {
        let r = decider(&pool, &m, p, 3, version_du_prompt(&pool, p).await, approuver).await;
        assert!(matches!(r, Err(ErreurEcriturePrompt::DejaDecidee)), "{r:?}");
    }
    // Une version d'administrateur ou standard ne se décide pas.
    for approuver in [true, false] {
        let r = decider(&pool, &m, p, 1, version_du_prompt(&pool, p).await, approuver).await;
        assert!(matches!(r, Err(ErreurEcriturePrompt::PasUneProposition)), "{r:?}");
    }
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let s = prompt_par_nom(&pool, m.c, "kesh-un").await;
    for approuver in [true, false] {
        let r = decider(&pool, &m, s, 1, version_du_prompt(&pool, s).await, approuver).await;
        assert!(matches!(r, Err(ErreurEcriturePrompt::PasUneProposition)), "{r:?}");
    }
    // Numéro inexistant, prompt d'une autre société : NotFound.
    let r = decider(&pool, &m, p, 99, version_du_prompt(&pool, p).await, true).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::NotFound)), "{r:?}");
    let autre = montage(&pool, "autre").await;
    let r = decider(&pool, &autre, p, 2, version_du_prompt(&pool, p).await, true).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::NotFound)), "{r:?}");
}

// ---------------------------------------------------------------------------
// AC-i8 — les noms et le plafond
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn noms_refuses_et_pris(pool: MySqlPool) {
    let m = montage(&pool, "noms").await;
    creer_admin(&pool, &m, "pris").await;
    let avant = nb_versions(&pool).await;
    for (nom, attendu) in [
        ("Pris", "NomInvalide"),
        ("valide\n", "NomInvalide"),
        ("", "NomInvalide"),
        ("kesh-x", "NomReserve"),
        ("keshx", "NomReserve"),
        ("kesh_x", "NomReserve"),
        ("pris", "NomDejaPris"),
    ] {
        let mut tx = pool.begin().await.unwrap();
        let r = depot::creer_prompt_admin_in_tx(&mut tx, m.c, m.u, nom, "D.", "T.").await;
        tx.rollback().await.unwrap();
        let nom_erreur = match r {
            Err(ErreurEcriturePrompt::NomInvalide) => "NomInvalide",
            Err(ErreurEcriturePrompt::NomReserve) => "NomReserve",
            Err(ErreurEcriturePrompt::NomDejaPris) => "NomDejaPris",
            other => panic!("{nom:?} : {other:?}"),
        };
        assert_eq!(nom_erreur, attendu, "{nom:?}");
    }
    // Contenu hors des bornes.
    let mut tx = pool.begin().await.unwrap();
    let r = depot::creer_prompt_admin_in_tx(&mut tx, m.c, m.u, "vide", " ", "T.").await;
    tx.rollback().await.unwrap();
    assert!(matches!(r, Err(ErreurEcriturePrompt::ContenuInvalide)), "{r:?}");
    assert_eq!(nb_versions(&pool).await, avant, "rien d'écrit");
}

#[sqlx::test(migrations = "./test-schema")]
async fn propositions_d_agent(pool: MySqlPool) {
    let m = montage(&pool, "agent").await;
    depot::amorcer_standards(&pool, STANDARDS).await.unwrap();
    let avant = nb_versions(&pool).await;
    // Aucune proposition sur un nom en `kesh`, même existant (C-32-10a-27).
    for nom in ["kesh-developpement", "kesh-assistant", "kesh-neuf", "keshx"] {
        let r = proposer(&pool, &m, nom, "PIEGE_STANDARD_3210A").await;
        assert!(matches!(r, Err(ErreurEcriturePrompt::NomReserve)), "{nom} : {r:?}");
    }
    let r = proposer(&pool, &m, "Majuscule", "T.").await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::NomInvalide)), "{r:?}");
    assert_eq!(nb_versions(&pool).await, avant, "aucune version ajoutée");
    // Nom neuf : prompt et version 1 à valider.
    let e = proposer(&pool, &m, "neuf", "Un.").await.unwrap();
    assert_eq!((e.numero, e.origine, e.etat, e.version), (1, Origine::Agent, Etat::EnAttente, 1));
    // Nom existant non standard : version ajoutée à ce prompt.
    let p = creer_admin(&pool, &m, "existant").await;
    let e = proposer(&pool, &m, "existant", "Deux.").await.unwrap();
    assert_eq!((e.prompt_id, e.numero, e.etat), (p, 2, Etat::EnAttente));
    assert_eq!(e.version, version_du_prompt(&pool, p).await);
    let (_, versions) = depot::historique(&pool, m.c, p).await.unwrap().unwrap();
    assert_eq!(versions[1].version.author_mcp_key_id, Some(m.k));
    assert_eq!(versions[1].cle_nom.as_deref(), Some("cle-agent"));
    assert_eq!(versions[1].cle_revoquee, Some(false));
    assert_eq!(versions[0].auteur_nom.as_deref(), Some("admin-agent"));
}

#[sqlx::test(migrations = "./test-schema")]
async fn plafond_des_propositions(pool: MySqlPool) {
    let m = montage(&pool, "plafond").await;
    for i in 0..PLAFOND_A_VALIDER {
        proposer(&pool, &m, &format!("p{i}"), "T.").await.unwrap();
    }
    let avant = nb_versions(&pool).await;
    let r = proposer(&pool, &m, "un-de-trop", "T.").await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::PlafondAtteint)), "{r:?}");
    let r = proposer(&pool, &m, "p0", "T.").await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::PlafondAtteint)), "{r:?}");
    assert_eq!(nb_versions(&pool).await, avant);
    // Une autre société n'est pas comptée.
    let autre = montage(&pool, "autre").await;
    proposer(&pool, &autre, "libre", "T.").await.unwrap();
    // Un refus libère une place.
    let p = prompt_par_nom(&pool, m.c, "p0").await;
    decider(&pool, &m, p, 1, version_du_prompt(&pool, p).await, false)
        .await
        .unwrap();
    proposer(&pool, &m, "un-de-trop", "T.").await.unwrap();
}

/// Le plafond est compté **sous le verrou de société** (Mi20) : deux
/// propositions concurrentes sur deux prompts **différents**, la seconde attend
/// la première et reçoit « plafond atteint ».
#[sqlx::test(migrations = "./test-schema")]
async fn plafond_sous_le_verrou_de_societe(pool: MySqlPool) {
    let m = montage(&pool, "concurrence").await;
    for i in 0..PLAFOND_A_VALIDER - 1 {
        proposer(&pool, &m, &format!("p{i}"), "T.").await.unwrap();
    }
    let mut tx_a = pool.begin().await.unwrap();
    depot::proposer_version_agent_in_tx(&mut tx_a, m.c, m.k, "premier", "D.", "T.", PLAFOND_A_VALIDER)
        .await
        .expect("la vingtième passe");
    let (pool_b, c, k) = (pool.clone(), m.c, m.k);
    let b = tokio::spawn(async move {
        let mut tx_b = pool_b.begin().await.unwrap();
        let r = depot::proposer_version_agent_in_tx(&mut tx_b, c, k, "second", "D.", "T.", PLAFOND_A_VALIDER)
            .await;
        match &r {
            Ok(_) => tx_b.commit().await.unwrap(),
            Err(_) => tx_b.rollback().await.unwrap(),
        }
        r
    });
    tokio::time::sleep(Duration::from_millis(700)).await;
    assert!(!b.is_finished(), "la seconde n'attend pas le verrou de société");
    tx_a.commit().await.unwrap();
    let r = b.await.unwrap();
    assert!(matches!(r, Err(ErreurEcriturePrompt::PlafondAtteint)), "{r:?}");
    let a_valider: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM mcp_prompt_versions WHERE state = 'pending'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(a_valider, i64::from(PLAFOND_A_VALIDER), "aucune vingt et unième");
}

/// Ce qui se refuse sans lire la base se refuse **avant** le verrou de société
/// (Mi27, validation P4 H11) : une autre transaction tient ce verrou, et la
/// proposition d'un nom en `kesh` ou non conforme rend l'erreur sans attendre.
/// Le délai d'attente d'un verrou est posé à une seconde sur la connexion de
/// l'appel (patron d'`onboarding_audit_e2e.rs`) ; celui du serveur n'est pas
/// relu.
#[sqlx::test(migrations = "./test-schema")]
async fn refus_avant_le_verrou(pool: MySqlPool) {
    let m = montage(&pool, "verrou").await;
    let mut tient = pool.begin().await.unwrap();
    sqlx::query_scalar::<_, i64>("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(m.c)
        .fetch_one(&mut *tient)
        .await
        .unwrap();
    for nom in ["kesh-x", "Non Conforme"] {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET SESSION innodb_lock_wait_timeout = 1")
            .execute(&mut *tx)
            .await
            .unwrap();
        let r = tokio::time::timeout(
            Duration::from_millis(500),
            depot::proposer_version_agent_in_tx(&mut tx, m.c, m.k, nom, "D.", "T.", PLAFOND_A_VALIDER),
        )
        .await
        .unwrap_or_else(|_| panic!("{nom} : l'appel attend le verrou de société"));
        assert!(
            matches!(
                r,
                Err(ErreurEcriturePrompt::NomReserve | ErreurEcriturePrompt::NomInvalide)
            ),
            "{nom} : {r:?}"
        );
        tx.rollback().await.unwrap();
    }
    // Témoin : un nom valide, lui, attend le verrou.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET SESSION innodb_lock_wait_timeout = 1")
        .execute(&mut *tx)
        .await
        .unwrap();
    let r = tokio::time::timeout(
        Duration::from_millis(500),
        depot::proposer_version_agent_in_tx(&mut tx, m.c, m.k, "valide", "D.", "T.", PLAFOND_A_VALIDER),
    )
    .await;
    assert!(r.is_err(), "témoin : un nom valide n'a pas attendu");
    drop(tx);
    tient.rollback().await.unwrap();
}

// ---------------------------------------------------------------------------
// AC-i10 — l'amorçage
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_premiere_et_deuxieme_execution(pool: MySqlPool) {
    let m = montage(&pool, "amorce").await;
    let r = depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    assert_eq!((r.societes, r.creations, r.mises_a_jour, r.retraits), (1, 2, 0, 0));
    for s in [S1, S2] {
        let p = prompt_par_nom(&pool, m.c, s.nom).await;
        assert_eq!(
            versions_de(&pool, p).await,
            vec![(1, "standard".into(), "approved".into(), "text".into(), Some(s.texte.into()))]
        );
        assert_eq!(version_du_prompt(&pool, p).await, 1);
    }
    let avant = nb_versions(&pool).await;
    let r = depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    assert_eq!((r.creations, r.mises_a_jour, r.retraits), (0, 0, 0));
    assert_eq!(nb_versions(&pool).await, avant, "deuxième exécution sans effet");
    // Aucune entrée d'audit.
    let audit: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audit, 0);
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_code_modifie(pool: MySqlPool) {
    let m = montage(&pool, "modifie").await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let r = depot::amorcer_standards(&pool, &[S1_MODIFIE]).await.unwrap();
    assert_eq!(r.mises_a_jour, 1);
    let p = prompt_par_nom(&pool, m.c, "kesh-un").await;
    let v = versions_de(&pool, p).await;
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].4.as_deref(), Some(S1.texte), "la précédente intacte, octet pour octet");
    assert_eq!(v[1].4.as_deref(), Some(S1_MODIFIE.texte));
    assert_eq!(v[1].1, "standard");
    let en_vigueur = depot::en_vigueur_par_nom(&pool, m.c, "kesh-un").await.unwrap().unwrap();
    assert_eq!(en_vigueur.texte, S1_MODIFIE.texte);
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_respecte_les_personnalisations(pool: MySqlPool) {
    let m = montage(&pool, "perso").await;
    depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    let un = prompt_par_nom(&pool, m.c, "kesh-un").await;
    let deux = prompt_par_nom(&pool, m.c, "kesh-deux").await;
    // kesh-un : personnalisé par l'administrateur.
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, un, 1, m.u, &texte("D.", "Mien."))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    // kesh-deux : retiré par l'administrateur.
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, deux, 1, m.u, &Contenu::Retrait)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let avant = nb_versions(&pool).await;
    let s2_modifie = Standard {
        texte: "Deux, modifié.",
        ..S2
    };
    let r = depot::amorcer_standards(&pool, &[S1_MODIFIE, s2_modifie]).await.unwrap();
    assert_eq!((r.creations, r.mises_a_jour, r.retraits), (0, 0, 0));
    assert_eq!(nb_versions(&pool).await, avant, "personnalisés : inchangés");
    // Retiré du code alors que personnalisé : rien non plus.
    depot::amorcer_standards(&pool, &[]).await.unwrap();
    assert_eq!(nb_versions(&pool).await, avant);
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_respecte_une_version_d_agent_approuvee(pool: MySqlPool) {
    let m = montage(&pool, "agentapprouve").await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let p = prompt_par_nom(&pool, m.c, "kesh-un").await;
    // Une proposition d'agent ne vise jamais un standard par le dépôt : posée
    // en SQL brut, décidée.
    brut_version(&pool, p, 2, "agent", "approved", "text", Some("D."), Some("Agent."), None, Some(m.k), Some(m.u), true)
        .await
        .unwrap();
    let avant = nb_versions(&pool).await;
    depot::amorcer_standards(&pool, &[S1_MODIFIE]).await.unwrap();
    assert_eq!(nb_versions(&pool).await, avant);
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_et_proposition_en_attente(pool: MySqlPool) {
    let m = montage(&pool, "attente").await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let p = prompt_par_nom(&pool, m.c, "kesh-un").await;
    brut_version(&pool, p, 2, "agent", "pending", "text", Some("D."), Some("Agent."), None, Some(m.k), None, false)
        .await
        .unwrap();
    let r = depot::amorcer_standards(&pool, &[S1_MODIFIE]).await.unwrap();
    assert_eq!(r.mises_a_jour, 1);
    let v = versions_de(&pool, p).await;
    assert_eq!(v.len(), 3);
    assert_eq!((v[1].2.as_str(), v[2].1.as_str()), ("pending", "standard"));
    let r = decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, true).await;
    assert!(matches!(r, Err(ErreurEcriturePrompt::PasLaDerniere)), "{r:?}");
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_standard_retire_du_code(pool: MySqlPool) {
    let m = montage(&pool, "retire").await;
    depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    let r = depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    assert_eq!((r.mises_a_jour, r.retraits), (0, 1));
    let deux = prompt_par_nom(&pool, m.c, "kesh-deux").await;
    let v = versions_de(&pool, deux).await;
    assert_eq!(v[1], (2, "standard".into(), "approved".into(), "withdrawal".into(), None));
    let servis: Vec<_> = depot::en_vigueur_par_societe(&pool, m.c)
        .await
        .unwrap()
        .into_iter()
        .map(|v| v.nom)
        .collect();
    assert_eq!(servis, ["kesh-un"]);
    // Idempotent ; et rétabli dans le code, il revient.
    let avant = nb_versions(&pool).await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    assert_eq!(nb_versions(&pool).await, avant);
    let r = depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    assert_eq!(r.mises_a_jour, 1);
    assert!(depot::en_vigueur_par_nom(&pool, m.c, "kesh-deux").await.unwrap().is_some());
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_standard_altere_en_base(pool: MySqlPool) {
    let m = montage(&pool, "altere").await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let p = prompt_par_nom(&pool, m.c, "kesh-un").await;
    // Cas 1 : le texte entier remplacé.
    sqlx::query("UPDATE mcp_prompt_versions SET body = 'PIEGE_ALTERE_3210A' WHERE prompt_id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(depot::amorcer_standards(&pool, &[S1]).await.unwrap().mises_a_jour, 1);
    // Cas 2 : la seule casse d'une lettre (F12 : la collation des tables ne la
    // verrait pas).
    let altere = S1.texte.replacen('T', "t", 1);
    assert_ne!(altere, S1.texte);
    sqlx::query("UPDATE mcp_prompt_versions SET body = ? WHERE prompt_id = ? AND number = 2")
        .bind(&altere)
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(depot::amorcer_standards(&pool, &[S1]).await.unwrap().mises_a_jour, 1);
    let v = versions_de(&pool, p).await;
    assert_eq!(v.len(), 3);
    assert_eq!(v[2].4.as_deref(), Some(S1.texte));
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_de_chaque_societe(pool: MySqlPool) {
    let a = montage(&pool, "a").await;
    let b = montage(&pool, "b").await;
    let r = depot::amorcer_standards(&pool, STANDARDS).await.unwrap();
    assert_eq!(r.societes, 2);
    for c in [a.c, b.c] {
        let servis: Vec<_> = depot::en_vigueur_par_societe(&pool, c)
            .await
            .unwrap()
            .into_iter()
            .map(|v| (v.nom, v.origine, v.texte))
            .collect();
        let mut attendus: Vec<_> = STANDARDS
            .iter()
            .map(|s| (s.nom.to_string(), Origine::Standard, s.texte.to_string()))
            .collect();
        attendus.sort_by(|x, y| x.0.cmp(&y.0));
        assert_eq!(servis, attendus);
    }
}

#[sqlx::test(migrations = "./test-schema")]
async fn amorcage_refuse_un_standard_invalide(pool: MySqlPool) {
    montage(&pool, "invalide").await;
    for s in [
        Standard {
            nom: "sans-prefixe",
            ..S1
        },
        Standard {
            texte: "  ",
            ..S1
        },
    ] {
        assert!(depot::amorcer_standards(&pool, &[s]).await.is_err());
    }
    assert_eq!(nb_versions(&pool).await, 0);
}

// ---------------------------------------------------------------------------
// AC-i5 — synthèse, historique, retour au standard
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "./test-schema")]
async fn synthese_par_societe(pool: MySqlPool) {
    let m = montage(&pool, "synthese").await;
    depot::amorcer_standards(&pool, &[S1, S2]).await.unwrap();
    let libre = creer_admin(&pool, &m, "libre").await;
    proposer(&pool, &m, "libre", "A.").await.unwrap();
    proposer(&pool, &m, "libre", "B.").await.unwrap();
    proposer(&pool, &m, "attente-seule", "C.").await.unwrap();
    let deux = prompt_par_nom(&pool, m.c, "kesh-deux").await;
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, deux, 1, m.u, &Contenu::Retrait)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    // Une autre société n'apparaît pas.
    let autre = montage(&pool, "autre").await;
    proposer(&pool, &autre, "ailleurs", "D.").await.unwrap();

    let s = depot::synthese_par_societe(&pool, m.c, &[S1, S2]).await.unwrap();
    let noms: Vec<_> = s.iter().map(|l| l.nom.as_str()).collect();
    assert_eq!(noms, ["attente-seule", "kesh-deux", "kesh-un", "libre"]);
    let par = |n: &str| s.iter().find(|l| l.nom == n).unwrap().clone();

    let l = par("libre");
    assert_eq!((l.id, l.is_standard, l.a_valider, l.standard_differe), (libre, false, 2, false));
    assert_eq!((l.derniere.numero, l.derniere.etat), (3, Etat::EnAttente));
    assert_eq!(l.en_vigueur.map(|v| (v.numero, v.origine)), Some((1, Origine::Admin)));

    let l = par("attente-seule");
    assert_eq!((l.a_valider, l.en_vigueur), (1, None));

    let l = par("kesh-un");
    assert!(!l.standard_differe);
    assert_eq!(l.en_vigueur.map(|v| v.nature), Some(Nature::Texte));

    let l = par("kesh-deux");
    assert!(l.standard_differe, "retrait d'administrateur : diffère");
    assert_eq!(l.en_vigueur.map(|v| (v.origine, v.nature)), Some((Origine::Admin, Nature::Retrait)));

    // Le code a changé : le standard non personnalisé diffère (jusqu'à
    // l'amorçage) ; la comparaison est en Rust, octet pour octet.
    let s = depot::synthese_par_societe(&pool, m.c, &[S1_MODIFIE, S2]).await.unwrap();
    assert!(s.iter().find(|l| l.nom == "kesh-un").unwrap().standard_differe);
    let casse = Standard {
        texte: "texte du standard un.",
        ..S1
    };
    let s = depot::synthese_par_societe(&pool, m.c, &[casse, S2]).await.unwrap();
    assert!(s.iter().find(|l| l.nom == "kesh-un").unwrap().standard_differe);
}

#[sqlx::test(migrations = "./test-schema")]
async fn historique_et_retour_au_standard(pool: MySqlPool) {
    let m = montage(&pool, "histo").await;
    depot::amorcer_standards(&pool, &[S1]).await.unwrap();
    let p = prompt_par_nom(&pool, m.c, "kesh-un").await;
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, m.c, p, 1, m.u, &texte("D.", "Mien."))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    // Retour au standard : refusé avec un mauvais standard, une version
    // périmée, ou sur un prompt non standard.
    let libre = creer_admin(&pool, &m, "libre").await;
    for (prompt, version, standard, cas) in [
        (p, 1, S1, "périmée"),
        (p, 2, S2, "autre standard"),
        (libre, 1, S1, "non standard"),
    ] {
        let mut tx = pool.begin().await.unwrap();
        let r = depot::revenir_au_standard_in_tx(&mut tx, m.c, prompt, version, &standard).await;
        tx.rollback().await.unwrap();
        let ok = matches!(
            (&r, cas),
            (Err(ErreurEcriturePrompt::VersionPerimee), "périmée")
                | (Err(ErreurEcriturePrompt::StandardInconnu), "autre standard")
                | (Err(ErreurEcriturePrompt::NonStandard), "non standard")
        );
        assert!(ok, "{cas} : {r:?}");
    }
    let mut tx = pool.begin().await.unwrap();
    let e = depot::revenir_au_standard_in_tx(&mut tx, m.c, p, 2, &S1).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!((e.numero, e.origine, e.version), (3, Origine::Standard, 3));
    let v = depot::en_vigueur_par_nom(&pool, m.c, "kesh-un").await.unwrap().unwrap();
    assert_eq!((v.numero, v.texte.as_str()), (3, S1.texte));

    let (prompt, versions) = depot::historique(&pool, m.c, p).await.unwrap().unwrap();
    assert_eq!((prompt.name.as_str(), prompt.is_standard, prompt.version), ("kesh-un", true, 3));
    let resume: Vec<_> = versions
        .iter()
        .map(|v| (v.version.number, v.version.origin, v.auteur_nom.clone(), v.cle_revoquee))
        .collect();
    assert_eq!(
        resume,
        vec![
            (1, Origine::Standard, None, None),
            (2, Origine::Admin, Some("admin-histo".to_string()), None),
            (3, Origine::Standard, None, None),
        ]
    );
    assert_eq!(versions[1].version.contenu().unwrap(), texte("D.", "Mien."));
}

#[sqlx::test(migrations = "./test-schema")]
async fn historique_nomme_le_decideur_et_la_cle_revoquee(pool: MySqlPool) {
    let m = montage(&pool, "decideur").await;
    let p = creer_admin(&pool, &m, "aide").await;
    proposer(&pool, &m, "aide", "Agent.").await.unwrap();
    decider(&pool, &m, p, 2, version_du_prompt(&pool, p).await, true)
        .await
        .unwrap();
    sqlx::query("UPDATE mcp_keys SET revoked_at = NOW(3) WHERE id = ?")
        .bind(m.k)
        .execute(&pool)
        .await
        .unwrap();
    let (_, versions) = depot::historique(&pool, m.c, p).await.unwrap().unwrap();
    let v = &versions[1];
    assert_eq!(v.decideur_nom.as_deref(), Some("admin-decideur"));
    assert_eq!(v.cle_revoquee, Some(true));
    assert!(v.version.decided_at.is_some());
}
