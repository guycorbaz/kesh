//! Dépôt des prompts du serveur MCP (`mcp_prompts`, `mcp_prompt_versions`,
//! Story 32-10a-i, refs #723) — **liste close** (AC-i5).
//!
//! **Lectures** (sur un pool, toutes filtrées par société — une ligne d'une
//! autre société est indiscernable d'une ligne absente) : [`synthese_par_societe`],
//! [`historique`], [`en_vigueur_par_societe`], [`en_vigueur_par_nom`].
//!
//! **Écritures** (dans la transaction de l'appelant, qui valide ; patron de
//! `mcp_keys::create_in_tx`) : [`creer_prompt_admin_in_tx`],
//! [`ajouter_version_admin_in_tx`], [`proposer_version_agent_in_tx`],
//! [`approuver_in_tx`], [`refuser_in_tx`], [`revenir_au_standard_in_tx`] ; et
//! [`amorcer_standards`], seule à prendre un pool et à ouvrir ses transactions
//! (une par société). Chacune rend [`VersionEcrite`], sans texte.
//!
//! **Ordre des verrous** (C-32-10a-28, C-32-10a-32 ; Pattern 5 de
//! `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) : la ligne de la **société** par
//! `SELECT id FROM companies WHERE id = ? FOR UPDATE` d'abord, pour **toute**
//! écriture — l'amorçage compris —, puis la ligne du prompt `FOR UPDATE`, puis
//! `MAX(number) + 1` sous ce verrou. Le plafond des propositions est ainsi
//! compté sous le verrou de société (atomique), deux créations du même nom neuf
//! se sérialisent, et une remise à zéro (`kesh_seed::reset_demo`, qui tient
//! `companies` avant de vider) ne forme aucun cycle avec une écriture des
//! prompts. Ce qui se refuse **sans lire la base** — nom non conforme ou en
//! `kesh`, contenu hors des bornes — se refuse **avant** le verrou (validation
//! P3, F9).
//!
//! **Le contenu d'une version ne se modifie jamais, et aucune version ne
//! s'efface** : la seule instruction qui écrit une version existante est
//! l'`UPDATE` de sa décision (`state`, `decided_by_user_id`, `decided_at`) —
//! garde `versions_jamais_reecrites` (`tests/mcp_prompts_lexical.rs`).
//!
//! **Aucune journalisation de texte** : ce module n'émet aucun évènement
//! `tracing`, et les types qu'il rend masquent les textes dans leur `Debug`.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::mcp_prompt::{
    Contenu, ErreurEcriturePrompt, Etat, McpPrompt, McpPromptVersion, Nature, Origine,
    PREFIXE_STANDARD, RapportAmorcage, ResumeVersion, SynthesePrompt, VersionEcrite,
    VersionEnVigueur, VersionHistorique, contenu_attendu, contenu_dans_les_bornes,
    differe_du_standard, nom_conforme, nom_reserve,
};
use crate::entities::mcp_prompt_standards::Standard;
use crate::errors::{DbError, map_db_error};

type Tx<'a> = Transaction<'a, MySql>;

const COLONNES_PROMPT: &str =
    "p.id, p.company_id, p.name, p.is_standard, p.version, p.created_at, p.updated_at";

const COLONNES_VERSION: &str = "v.id, v.prompt_id, v.number, v.origin, v.state, v.nature, \
     v.description, v.body, v.author_user_id, v.author_mcp_key_id, v.decided_by_user_id, \
     v.decided_at, v.created_at";

/// Le numéro de la version **en vigueur** du prompt d'alias `p` : la version
/// approuvée de plus grand numéro (C-32-10a-4). Seule définition, employée par
/// toutes les lectures.
const NUMERO_EN_VIGUEUR_SQL: &str = "(SELECT MAX(x.number) FROM mcp_prompt_versions x \
     WHERE x.prompt_id = p.id AND x.state = 'approved')";

// ---------------------------------------------------------------------------
// Lectures
// ---------------------------------------------------------------------------

#[derive(sqlx::FromRow)]
struct SyntheseRow {
    id: i64,
    name: String,
    is_standard: bool,
    version: i32,
    d_numero: Option<i32>,
    d_origine: Option<Origine>,
    d_etat: Option<Etat>,
    d_nature: Option<Nature>,
    v_numero: Option<i32>,
    v_origine: Option<Origine>,
    v_nature: Option<Nature>,
    v_description: Option<String>,
    v_body: Option<String>,
    a_valider: i64,
}

/// La synthèse des prompts d'une société, triés par nom, en **une** requête :
/// pour chacun, son identité, sa dernière version, sa version en vigueur (texte
/// ou retrait, ou aucune), son nombre de versions à valider, et
/// `standard_differe` — calculé **en Rust, octet pour octet** sur le contenu
/// lu de la version en vigueur ([`differe_du_standard`]) ; les textes lus ne
/// sortent pas de la synthèse. Les retraits et les prompts sans version
/// approuvée sont **compris**.
pub async fn synthese_par_societe(
    pool: &MySqlPool,
    company_id: i64,
    standards: &[Standard],
) -> Result<Vec<SynthesePrompt>, DbError> {
    let rows = sqlx::query_as::<_, SyntheseRow>(&format!(
        "SELECT p.id, p.name, p.is_standard, p.version, \
                d.number AS d_numero, d.origin AS d_origine, d.state AS d_etat, \
                d.nature AS d_nature, \
                v.number AS v_numero, v.origin AS v_origine, v.nature AS v_nature, \
                v.description AS v_description, v.body AS v_body, \
                (SELECT COUNT(*) FROM mcp_prompt_versions c \
                  WHERE c.prompt_id = p.id AND c.state = 'pending') AS a_valider \
         FROM mcp_prompts p \
         LEFT JOIN mcp_prompt_versions d ON d.prompt_id = p.id AND d.number = \
              (SELECT MAX(y.number) FROM mcp_prompt_versions y WHERE y.prompt_id = p.id) \
         LEFT JOIN mcp_prompt_versions v ON v.prompt_id = p.id \
              AND v.number = {NUMERO_EN_VIGUEUR_SQL} \
         WHERE p.company_id = ? \
         ORDER BY p.name, p.id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    rows.into_iter()
        .map(|r| {
            let (Some(numero), Some(origine), Some(etat), Some(nature)) =
                (r.d_numero, r.d_origine, r.d_etat, r.d_nature)
            else {
                return Err(DbError::Invariant(format!(
                    "prompt MCP {} sans aucune version",
                    r.id
                )));
            };
            let derniere = ResumeVersion {
                numero,
                origine,
                etat,
                nature,
            };
            let (en_vigueur, contenu) = match (r.v_numero, r.v_origine, r.v_nature) {
                (Some(numero), Some(origine), Some(nature)) => (
                    Some(ResumeVersion {
                        numero,
                        origine,
                        etat: Etat::Approuvee,
                        nature,
                    }),
                    Some(Contenu::depuis_colonnes(nature, r.v_description, r.v_body)?),
                ),
                _ => (None, None),
            };
            let standard_differe = r.is_standard
                && differe_du_standard(
                    &r.name,
                    en_vigueur
                        .as_ref()
                        .zip(contenu.as_ref())
                        .map(|(e, c)| (e.origine, c)),
                    standards,
                );
            Ok(SynthesePrompt {
                id: r.id,
                nom: r.name,
                is_standard: r.is_standard,
                version: r.version,
                derniere,
                en_vigueur,
                a_valider: r.a_valider,
                standard_differe,
            })
        })
        .collect()
}

#[derive(sqlx::FromRow)]
struct HistoriqueRow {
    #[sqlx(flatten)]
    version: McpPromptVersion,
    auteur_nom: Option<String>,
    decideur_nom: Option<String>,
    cle_nom: Option<String>,
    cle_revoquee: Option<i64>,
}

/// Un prompt de la société et **toutes** ses versions, textes compris, par
/// numéro, avec les noms de l'auteur, du décideur et de la clé auteur
/// ([`VersionHistorique`]) ; `None` si le prompt n'existe pas ou appartient à
/// une autre société.
pub async fn historique(
    pool: &MySqlPool,
    company_id: i64,
    prompt_id: i64,
) -> Result<Option<(McpPrompt, Vec<VersionHistorique>)>, DbError> {
    let Some(prompt) = sqlx::query_as::<_, McpPrompt>(&format!(
        "SELECT {COLONNES_PROMPT} FROM mcp_prompts p WHERE p.id = ? AND p.company_id = ?"
    ))
    .bind(prompt_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?
    else {
        return Ok(None);
    };
    let versions = sqlx::query_as::<_, HistoriqueRow>(&format!(
        "SELECT {COLONNES_VERSION}, au.username AS auteur_nom, de.username AS decideur_nom, \
                k.name AS cle_nom, \
                CASE WHEN k.id IS NULL THEN NULL \
                     WHEN k.revoked_at IS NULL THEN 0 ELSE 1 END AS cle_revoquee \
         FROM mcp_prompt_versions v \
         LEFT JOIN users au ON au.id = v.author_user_id \
         LEFT JOIN users de ON de.id = v.decided_by_user_id \
         LEFT JOIN mcp_keys k ON k.id = v.author_mcp_key_id \
         WHERE v.prompt_id = ? ORDER BY v.number"
    ))
    .bind(prompt.id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?
    .into_iter()
    .map(|r| VersionHistorique {
        version: r.version,
        auteur_nom: r.auteur_nom,
        decideur_nom: r.decideur_nom,
        cle_nom: r.cle_nom,
        cle_revoquee: r.cle_revoquee.map(|n| n != 0),
    })
    .collect();
    Ok(Some((prompt, versions)))
}

#[derive(sqlx::FromRow)]
struct VigueurRow {
    prompt_id: i64,
    nom: String,
    numero: i32,
    origine: Origine,
    description: Option<String>,
    body: Option<String>,
    cle_auteur: Option<i64>,
}

impl TryFrom<VigueurRow> for VersionEnVigueur {
    type Error = DbError;
    fn try_from(r: VigueurRow) -> Result<Self, DbError> {
        let (Some(description), Some(texte)) = (r.description, r.body) else {
            return Err(DbError::Invariant(format!(
                "version de texte {} du prompt {} sans contenu",
                r.numero, r.prompt_id
            )));
        };
        Ok(Self {
            prompt_id: r.prompt_id,
            nom: r.nom,
            numero: r.numero,
            origine: r.origine,
            description,
            texte,
            cle_auteur: r.cle_auteur,
        })
    }
}

/// La requête des versions en vigueur **de nature texte** ; `filtre` ajoute
/// une condition sur `p`.
fn requete_en_vigueur(filtre: &str) -> String {
    format!(
        "SELECT p.id AS prompt_id, p.name AS nom, v.number AS numero, v.origin AS origine, \
                v.description, v.body, v.author_mcp_key_id AS cle_auteur \
         FROM mcp_prompts p \
         JOIN mcp_prompt_versions v ON v.prompt_id = p.id \
              AND v.number = {NUMERO_EN_VIGUEUR_SQL} \
         WHERE p.company_id = ? AND v.nature = 'text' {filtre} \
         ORDER BY p.name, p.id"
    )
}

/// Les prompts de la société dont la version en vigueur est un **texte**
/// (AC-i6), triés par nom. Un retrait, une version à valider ou refusée, un
/// prompt sans version approuvée n'y sont pas.
pub async fn en_vigueur_par_societe(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<VersionEnVigueur>, DbError> {
    sqlx::query_as::<_, VigueurRow>(&requete_en_vigueur(""))
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .map(VersionEnVigueur::try_from)
        .collect()
}

/// La version en vigueur du prompt `nom` de la société, si c'est un texte.
/// Un nom non conforme rend `None` **sans lire la base** : la comparaison SQL
/// passe par la collation de la table, insensible à la casse, et seuls des noms
/// conformes (minuscules ASCII) y sont alors exacts.
pub async fn en_vigueur_par_nom(
    pool: &MySqlPool,
    company_id: i64,
    nom: &str,
) -> Result<Option<VersionEnVigueur>, DbError> {
    if !nom_conforme(nom) {
        return Ok(None);
    }
    sqlx::query_as::<_, VigueurRow>(&requete_en_vigueur("AND p.name = ?"))
        .bind(company_id)
        .bind(nom)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?
        .map(VersionEnVigueur::try_from)
        .transpose()
}

// ---------------------------------------------------------------------------
// Écritures — briques
// ---------------------------------------------------------------------------

/// Verrouille la ligne de la société (premier verrou de toute écriture).
async fn verrouiller_societe(tx: &mut Tx<'_>, company_id: i64) -> Result<bool, DbError> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT id FROM companies WHERE id = ? FOR UPDATE")
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .is_some(),
    )
}

/// Verrouille le prompt `prompt_id` **de la société**.
async fn verrouiller_prompt(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
) -> Result<Option<McpPrompt>, DbError> {
    sqlx::query_as::<_, McpPrompt>(&format!(
        "SELECT {COLONNES_PROMPT} FROM mcp_prompts p \
         WHERE p.id = ? AND p.company_id = ? FOR UPDATE"
    ))
    .bind(prompt_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Verrouille la société puis le prompt ; `NotFound` si l'un manque.
async fn verrouiller(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
) -> Result<McpPrompt, ErreurEcriturePrompt> {
    if !verrouiller_societe(tx, company_id).await? {
        return Err(ErreurEcriturePrompt::NotFound);
    }
    verrouiller_prompt(tx, company_id, prompt_id)
        .await?
        .ok_or(ErreurEcriturePrompt::NotFound)
}

/// Crée un prompt (version 1 de son verrou optimiste) et rend son identifiant.
async fn creer_prompt(
    tx: &mut Tx<'_>,
    company_id: i64,
    nom: &str,
    is_standard: bool,
) -> Result<i64, DbError> {
    let id =
        sqlx::query("INSERT INTO mcp_prompts (company_id, name, is_standard) VALUES (?, ?, ?)")
            .bind(company_id)
            .bind(nom)
            .bind(is_standard)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .last_insert_id();
    if id == 0 {
        return Err(DbError::Invariant(
            "identifiant nul à la création d'un prompt MCP".into(),
        ));
    }
    i64::try_from(id).map_err(|_| DbError::Invariant(format!("identifiant {id} hors de i64")))
}

/// Les auteurs d'une version : un utilisateur, une clé MCP, ou aucun (standard).
#[derive(Clone, Copy)]
enum Auteur {
    Code,
    Utilisateur(i64),
    Cle(i64),
}

/// Ajoute une version au prompt (numéro `MAX + 1`, sous le verrou du prompt
/// tenu par l'appelant) et, si `incrementer`, incrémente la `version` du
/// prompt. Rend `(numero, version du prompt)`.
async fn inserer_version(
    tx: &mut Tx<'_>,
    prompt_id: i64,
    origine: Origine,
    etat: Etat,
    contenu: &Contenu,
    auteur: Auteur,
    incrementer: bool,
) -> Result<(i32, i32), DbError> {
    let numero: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(MAX(number), 0) + 1 AS SIGNED) \
         FROM mcp_prompt_versions WHERE prompt_id = ?",
    )
    .bind(prompt_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let numero = i32::try_from(numero)
        .map_err(|_| DbError::Invariant(format!("numéro de version {numero} hors de i32")))?;
    let (description, texte) = contenu.colonnes();
    let (user, cle) = match auteur {
        Auteur::Code => (None, None),
        Auteur::Utilisateur(u) => (Some(u), None),
        Auteur::Cle(k) => (None, Some(k)),
    };
    sqlx::query(
        "INSERT INTO mcp_prompt_versions \
         (prompt_id, number, origin, state, nature, description, body, \
          author_user_id, author_mcp_key_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(prompt_id)
    .bind(numero)
    .bind(origine)
    .bind(etat)
    .bind(contenu.nature())
    .bind(description)
    .bind(texte)
    .bind(user)
    .bind(cle)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if incrementer {
        sqlx::query("UPDATE mcp_prompts SET version = version + 1 WHERE id = ?")
            .bind(prompt_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
    }
    let version: i32 = sqlx::query_scalar("SELECT version FROM mcp_prompts WHERE id = ?")
        .bind(prompt_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok((numero, version))
}

/// Contrôles qui ne lisent pas la base, faits **avant** tout verrou.
fn controler_nom_non_standard(nom: &str) -> Result<(), ErreurEcriturePrompt> {
    if !nom_conforme(nom) {
        return Err(ErreurEcriturePrompt::NomInvalide);
    }
    if nom_reserve(nom) {
        return Err(ErreurEcriturePrompt::NomReserve);
    }
    Ok(())
}

fn controler_contenu(contenu: &Contenu) -> Result<(), ErreurEcriturePrompt> {
    if contenu_dans_les_bornes(contenu) {
        Ok(())
    } else {
        Err(ErreurEcriturePrompt::ContenuInvalide)
    }
}

fn ecrite(
    prompt_id: i64,
    nom: &str,
    (numero, version): (i32, i32),
    origine: Origine,
    nature: Nature,
    etat: Etat,
) -> VersionEcrite {
    VersionEcrite {
        prompt_id,
        nom: nom.to_string(),
        numero,
        origine,
        nature,
        etat,
        version,
    }
}

// ---------------------------------------------------------------------------
// Écritures — liste close
// ---------------------------------------------------------------------------
//
// ⚠️ PRÉCONDITION DE L'APPELANT (#783 point 1, Story 32-10a-iii) : le plafond
// (`COUNT` des versions à valider), la numérotation (`MAX(number) + 1`) et le
// contrôle de la dernière version (`MAX(number)`) sont des lectures SIMPLES,
// faites sous le verrou de société. En `REPEATABLE READ`, InnoDB fige la vue
// d'une transaction à sa PREMIÈRE lecture simple : ces lectures ne sont justes
// que si l'appelant n'a RIEN lu dans sa transaction avant le premier appel de
// ce dépôt (dont le premier acte, `SELECT … FOR UPDATE` sur la société, est une
// lecture verrouillante qui ne fige aucune vue). Lire avant — même la table des
// prompts — rendrait le plafond et la numérotation faux, en silence. Les
// appelants d'aujourd'hui (`EcriturePrompts::proposer`, `routes/mcp_prompts.rs`)
// lisent l'historique sur le pool, hors de la transaction ; test de concurrence
// `plafond_juste_apres_l_attente_du_verrou` (`kesh-api/tests/mcp_prompts_e2e.rs`).
// Cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5.

/// Crée un prompt d'administrateur et sa version 1 (`admin`, approuvée).
/// Refuse avant le verrou un nom non conforme ou en `kesh`, un contenu hors des
/// bornes ; sous le verrou de société, un nom déjà pris.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section).
pub async fn creer_prompt_admin_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    user_id: i64,
    nom: &str,
    description: &str,
    texte: &str,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    controler_nom_non_standard(nom)?;
    let contenu = Contenu::Texte {
        description: description.to_string(),
        texte: texte.to_string(),
    };
    controler_contenu(&contenu)?;
    if !verrouiller_societe(tx, company_id).await? {
        return Err(ErreurEcriturePrompt::NotFound);
    }
    let pris: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM mcp_prompts WHERE company_id = ? AND name = ?)",
    )
    .bind(company_id)
    .bind(nom)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if pris {
        return Err(ErreurEcriturePrompt::NomDejaPris);
    }
    let prompt_id = creer_prompt(tx, company_id, nom, false).await?;
    let n = inserer_version(
        tx,
        prompt_id,
        Origine::Admin,
        Etat::Approuvee,
        &contenu,
        Auteur::Utilisateur(user_id),
        false,
    )
    .await?;
    Ok(ecrite(
        prompt_id,
        nom,
        n,
        Origine::Admin,
        Nature::Texte,
        Etat::Approuvee,
    ))
}

/// Ajoute une version d'administrateur (`admin`, approuvée) — un texte ou un
/// retrait — à un prompt de la société, standard compris (il est alors
/// **personnalisé**). Verrou optimiste sur `version_attendue`.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section).
pub async fn ajouter_version_admin_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
    version_attendue: i32,
    user_id: i64,
    contenu: &Contenu,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    controler_contenu(contenu)?;
    let prompt = verrouiller(tx, company_id, prompt_id).await?;
    if prompt.version != version_attendue {
        return Err(ErreurEcriturePrompt::VersionPerimee);
    }
    let n = inserer_version(
        tx,
        prompt_id,
        Origine::Admin,
        Etat::Approuvee,
        contenu,
        Auteur::Utilisateur(user_id),
        true,
    )
    .await?;
    Ok(ecrite(
        prompt_id,
        &prompt.name,
        n,
        Origine::Admin,
        contenu.nature(),
        Etat::Approuvee,
    ))
}

/// Une proposition d'agent : une version `agent` **à valider**, jamais servie
/// avant son approbation. Refuse **avant le verrou** un nom non conforme ou
/// **en `kesh`, même existant** (C-32-10a-27 : un agent ne propose pas de
/// modifier un standard) et un contenu hors des bornes ; **sous le verrou de
/// société**, le plafond (`plafond` versions à valider dans la société au plus,
/// [`crate::entities::mcp_prompt::PLAFOND_A_VALIDER`] en production). Sur un nom
/// existant, ajoute la version à ce prompt ; sur un nom neuf, crée le prompt et
/// sa version 1.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section)
/// — sans quoi le plafond se compte sur une vue d'avant l'attente du verrou.
#[allow(clippy::too_many_arguments)]
pub async fn proposer_version_agent_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    cle_id: i64,
    nom: &str,
    description: &str,
    texte: &str,
    plafond: u32,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    controler_nom_non_standard(nom)?;
    let contenu = Contenu::Texte {
        description: description.to_string(),
        texte: texte.to_string(),
    };
    controler_contenu(&contenu)?;
    if !verrouiller_societe(tx, company_id).await? {
        return Err(ErreurEcriturePrompt::NotFound);
    }
    let a_valider: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_prompt_versions v \
         JOIN mcp_prompts p ON p.id = v.prompt_id \
         WHERE p.company_id = ? AND v.state = 'pending'",
    )
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if a_valider >= i64::from(plafond) {
        return Err(ErreurEcriturePrompt::PlafondAtteint);
    }
    let existant: Option<McpPrompt> = sqlx::query_as(&format!(
        "SELECT {COLONNES_PROMPT} FROM mcp_prompts p \
         WHERE p.company_id = ? AND p.name = ? FOR UPDATE"
    ))
    .bind(company_id)
    .bind(nom)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let (prompt_id, incrementer) = match existant {
        // Inatteignable : un prompt standard porte un nom en `kesh-`, refusé
        // plus haut. Gardé pour qu'aucune proposition n'atteigne un standard.
        Some(p) if p.is_standard => return Err(ErreurEcriturePrompt::NomReserve),
        Some(p) => (p.id, true),
        None => (creer_prompt(tx, company_id, nom, false).await?, false),
    };
    let n = inserer_version(
        tx,
        prompt_id,
        Origine::Agent,
        Etat::EnAttente,
        &contenu,
        Auteur::Cle(cle_id),
        incrementer,
    )
    .await?;
    Ok(ecrite(
        prompt_id,
        nom,
        n,
        Origine::Agent,
        Nature::Texte,
        Etat::EnAttente,
    ))
}

/// Décide une proposition d'agent (C-32-10a-20). Ordre des contrôles :
/// prompt de la société, `version` attendue, version existante, origine
/// `agent`, état `pending` (les états décidés sont **terminaux**), et — pour
/// l'approbation seulement — version **dernière** du prompt.
async fn decider(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
    numero: i32,
    version_attendue: i32,
    user_id: i64,
    approuver: bool,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    let prompt = verrouiller(tx, company_id, prompt_id).await?;
    if prompt.version != version_attendue {
        return Err(ErreurEcriturePrompt::VersionPerimee);
    }
    let cible: Option<(i64, Origine, Etat)> = sqlx::query_as(
        "SELECT id, origin, state FROM mcp_prompt_versions WHERE prompt_id = ? AND number = ?",
    )
    .bind(prompt_id)
    .bind(numero)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let Some((version_id, origine, etat)) = cible else {
        return Err(ErreurEcriturePrompt::NotFound);
    };
    if origine != Origine::Agent {
        return Err(ErreurEcriturePrompt::PasUneProposition);
    }
    if etat != Etat::EnAttente {
        return Err(ErreurEcriturePrompt::DejaDecidee);
    }
    if approuver {
        let derniere: i32 =
            sqlx::query_scalar("SELECT MAX(number) FROM mcp_prompt_versions WHERE prompt_id = ?")
                .bind(prompt_id)
                .fetch_one(&mut **tx)
                .await
                .map_err(map_db_error)?;
        if derniere != numero {
            return Err(ErreurEcriturePrompt::PasLaDerniere);
        }
    }
    let nouvel_etat = if approuver {
        Etat::Approuvee
    } else {
        Etat::Refusee
    };
    sqlx::query(
        "UPDATE mcp_prompt_versions \
         SET state = ?, decided_by_user_id = ?, decided_at = NOW(3) \
         WHERE id = ? AND state = 'pending'",
    )
    .bind(nouvel_etat)
    .bind(user_id)
    .bind(version_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    sqlx::query("UPDATE mcp_prompts SET version = version + 1 WHERE id = ?")
        .bind(prompt_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok(VersionEcrite {
        prompt_id,
        nom: prompt.name,
        numero,
        origine: Origine::Agent,
        nature: Nature::Texte,
        etat: nouvel_etat,
        version: prompt.version + 1,
    })
}

/// Approuve une proposition d'agent **à valider** qui est la **dernière**
/// version du prompt ; verrou optimiste sur `version_attendue`.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section).
pub async fn approuver_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
    numero: i32,
    version_attendue: i32,
    user_id: i64,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    decider(
        tx,
        company_id,
        prompt_id,
        numero,
        version_attendue,
        user_id,
        true,
    )
    .await
}

/// Refuse une proposition d'agent **à valider**, dernière ou non ; verrou
/// optimiste sur `version_attendue`.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section).
pub async fn refuser_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
    numero: i32,
    version_attendue: i32,
    user_id: i64,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    decider(
        tx,
        company_id,
        prompt_id,
        numero,
        version_attendue,
        user_id,
        false,
    )
    .await
}

/// Le retour au standard : une version `standard` au contenu de `standard`,
/// qui redevient la version en vigueur (le prompt cesse d'être personnalisé).
/// Refuse un prompt non standard, et un `standard` qui ne porte pas le nom du
/// prompt.
///
/// Précondition : rien n'est lu dans `tx` avant cet appel (en tête de section).
pub async fn revenir_au_standard_in_tx(
    tx: &mut Tx<'_>,
    company_id: i64,
    prompt_id: i64,
    version_attendue: i32,
    standard: &Standard,
) -> Result<VersionEcrite, ErreurEcriturePrompt> {
    let prompt = verrouiller(tx, company_id, prompt_id).await?;
    if prompt.version != version_attendue {
        return Err(ErreurEcriturePrompt::VersionPerimee);
    }
    if !prompt.is_standard {
        return Err(ErreurEcriturePrompt::NonStandard);
    }
    if standard.nom != prompt.name {
        return Err(ErreurEcriturePrompt::StandardInconnu);
    }
    let n = inserer_version(
        tx,
        prompt_id,
        Origine::Standard,
        Etat::Approuvee,
        &Contenu::du_standard(standard),
        Auteur::Code,
        true,
    )
    .await?;
    Ok(ecrite(
        prompt_id,
        &prompt.name,
        n,
        Origine::Standard,
        Nature::Texte,
        Etat::Approuvee,
    ))
}

// ---------------------------------------------------------------------------
// Amorçage
// ---------------------------------------------------------------------------

/// Installe les prompts standard dans **chaque** société, de façon
/// **idempotente** (AC-i10, C-32-10a-6) — une transaction par société, qui
/// prend le verrou de société d'abord. Pour chaque standard `S` de `standards` :
///
/// - aucun prompt standard de nom `S.nom` → création du prompt et d'une version
///   1 `standard` au contenu de `S` ;
/// - version en vigueur d'origine `standard` (texte ou retrait) au contenu
///   **différent** de `S` → nouvelle version `standard` au contenu de `S` ;
/// - version en vigueur d'origine `admin` ou `agent` (**personnalisé**, retrait
///   d'administrateur compris) → rien.
///
/// Et pour un prompt standard dont le nom n'est **plus** dans `standards` :
/// version en vigueur `standard` de texte → nouvelle version `standard` de
/// **retrait** ; personnalisé → rien. La comparaison porte sur le **contenu**,
/// en Rust, octet pour octet : un standard altéré en base reprend le texte du
/// code. **Aucune entrée d'audit** ; le rapport compté, sans texte, revient à
/// l'appelant, qui le journalise.
///
/// Un standard au nom non conforme ou sans le préfixe `kesh-`, ou hors des
/// bornes, est refusé en `DbError::Invariant` avant toute écriture.
pub async fn amorcer_standards(
    pool: &MySqlPool,
    standards: &[Standard],
) -> Result<RapportAmorcage, DbError> {
    for s in standards {
        if !nom_conforme(s.nom)
            || !s.nom.starts_with(PREFIXE_STANDARD)
            || !contenu_dans_les_bornes(&Contenu::du_standard(s))
        {
            return Err(DbError::Invariant(format!(
                "prompt standard invalide : {}",
                s.nom
            )));
        }
    }
    let societes: Vec<i64> = sqlx::query_scalar("SELECT id FROM companies ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    let mut rapport = RapportAmorcage::default();
    for company_id in societes {
        let mut tx = pool.begin().await.map_err(map_db_error)?;
        if amorcer_societe(&mut tx, company_id, standards, &mut rapport).await? {
            tx.commit().await.map_err(map_db_error)?;
            rapport.societes += 1;
        } else {
            tx.rollback().await.map_err(map_db_error)?;
        }
    }
    Ok(rapport)
}

#[derive(sqlx::FromRow)]
struct StandardEnBase {
    id: i64,
    name: String,
    v_origine: Option<Origine>,
    v_nature: Option<Nature>,
    v_description: Option<String>,
    v_body: Option<String>,
}

/// Amorce une société, sous son verrou ; `false` si elle a disparu.
async fn amorcer_societe(
    tx: &mut Tx<'_>,
    company_id: i64,
    standards: &[Standard],
    rapport: &mut RapportAmorcage,
) -> Result<bool, DbError> {
    if !verrouiller_societe(tx, company_id).await? {
        return Ok(false);
    }
    let en_base: Vec<StandardEnBase> = sqlx::query_as(&format!(
        "SELECT p.id, p.name, v.origin AS v_origine, v.nature AS v_nature, \
                v.description AS v_description, v.body AS v_body \
         FROM mcp_prompts p \
         LEFT JOIN mcp_prompt_versions v ON v.prompt_id = p.id \
              AND v.number = {NUMERO_EN_VIGUEUR_SQL} \
         WHERE p.company_id = ? AND p.is_standard = TRUE \
         ORDER BY p.id FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    for s in standards {
        if !en_base.iter().any(|p| p.name == s.nom) {
            let prompt_id = creer_prompt(tx, company_id, s.nom, true).await?;
            inserer_version(
                tx,
                prompt_id,
                Origine::Standard,
                Etat::Approuvee,
                &Contenu::du_standard(s),
                Auteur::Code,
                false,
            )
            .await?;
            rapport.creations += 1;
        }
    }
    for p in en_base {
        let attendu = contenu_attendu(&p.name, standards);
        let en_vigueur = match (p.v_origine, p.v_nature) {
            (Some(origine), Some(nature)) => Some((
                origine,
                Contenu::depuis_colonnes(nature, p.v_description, p.v_body)?,
            )),
            _ => None,
        };
        let a_ecrire = match &en_vigueur {
            // Personnalisé : l'administrateur ou une proposition approuvée.
            Some((Origine::Admin | Origine::Agent, _)) => false,
            // Aucune version approuvée et plus rien à servir : rien.
            None => attendu != Contenu::Retrait,
            Some((origine, contenu)) => {
                differe_du_standard(&p.name, Some((*origine, contenu)), standards)
            }
        };
        if !a_ecrire {
            continue;
        }
        inserer_version(
            tx,
            p.id,
            Origine::Standard,
            Etat::Approuvee,
            &attendu,
            Auteur::Code,
            true,
        )
        .await?;
        match attendu {
            Contenu::Retrait => rapport.retraits += 1,
            Contenu::Texte { .. } => rapport.mises_a_jour += 1,
        }
    }
    Ok(true)
}
