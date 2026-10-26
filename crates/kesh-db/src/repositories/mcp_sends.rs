//! Dépôt des envois du serveur MCP (`mcp_sends`, Story 32-8a, refs #723).
//!
//! Tout est **scopé par société** (KF-002) : une ligne d'une autre société est
//! indiscernable d'une ligne absente. La relecture par l'agent
//! ([`lire_pour_cle`]) filtre **en plus** sur la clé qui lit — et sur la
//! société **et** la clé à la fois, aucune clé étrangère composite ne
//! garantissant que la société d'une ligne est celle de sa clé.
//!
//! **L'horloge est celle de MariaDB** : l'échéance se pose au dépôt
//! (`NOW(3) + INTERVAL ? SECOND`) et se lit par l'expression unique
//! [`ETAT_DERIVE_SQL`], employée par toutes les lectures et par la purge. La
//! table y est toujours désignée par l'alias `s`.
//!
//! **Le contenu se compare en Rust, sur les octets** ([`decider_in_tx`]) : la
//! collation `utf8mb4_unicode_ci` de la colonne tiendrait pour égaux `É` et
//! `é`, ou deux textes qui ne diffèrent que par un blanc final.
//!
//! **Aucune journalisation du contenu** : ce module n'émet aucun évènement
//! `tracing`, et ses types masquent contenu et résumé dans leur `Debug`
//! (C-32-8-40).

use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::mcp_send::{
    Decision, ETAT_DERIVE_SQL, EnvoiDecide, EnvoiDetail, EnvoiPourCle, EnvoiResume, ErreurDecision,
    ErreurDepot, EtatEnvoi, NouvelEnvoi,
};
use crate::errors::{DbError, map_db_error};

/// Nombre maximal de lignes que rend [`lister`].
pub const LISTE_MAX: u32 = 200;

/// Dépose un envoi **dans la transaction reçue** et rend son identifiant.
///
/// Compte d'abord les envois **en attente et vivants** de la clé ; si ce
/// compte atteint `plafond`, rend [`ErreurDepot::TropNombreux`] sans rien
/// écrire. Sinon insère l'envoi en attente, non modifié, d'échéance
/// `NOW(3) + duree_s` secondes à l'horloge de la base.
///
/// Le plafond est **souple** (C-32-8-13) : deux dépôts concurrents d'une même
/// clé peuvent le dépasser d'une unité, le compte n'étant pas verrouillé.
pub async fn deposer_in_tx(
    tx: &mut Transaction<'_, MySql>,
    envoi: &NouvelEnvoi,
    duree_s: u32,
    plafond: u32,
) -> Result<i64, ErreurDepot> {
    let en_attente: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM mcp_sends s \
         WHERE s.mcp_key_id = ? AND ({ETAT_DERIVE_SQL}) = 'pending'"
    ))
    .bind(envoi.mcp_key_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if en_attente >= i64::from(plafond) {
        return Err(ErreurDepot::TropNombreux);
    }

    let result = sqlx::query(
        "INSERT INTO mcp_sends \
         (company_id, mcp_key_id, tool_name, state, summary, content, modified, expires_at) \
         VALUES (?, ?, ?, 'pending', ?, ?, FALSE, NOW(3) + INTERVAL ? SECOND)",
    )
    .bind(envoi.company_id)
    .bind(envoi.mcp_key_id)
    .bind(&envoi.tool_name)
    .bind(&envoi.summary)
    .bind(&envoi.content)
    .bind(duree_s)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant("last_insert_id == 0 après INSERT mcp_sends".into()).into());
    }
    i64::try_from(last_id).map_err(|_| {
        DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")).into()
    })
}

/// Efface contenu et résumé des envois **échus** de la société qui en portent
/// encore, en une instruction ; rend le nombre de lignes purgées. Aucune autre
/// ligne n'est touchée.
pub async fn purger_echus(pool: &MySqlPool, company_id: i64) -> Result<u64, DbError> {
    Ok(sqlx::query(&format!(
        "UPDATE mcp_sends s SET s.content = NULL, s.summary = NULL \
         WHERE s.company_id = ? AND ({ETAT_DERIVE_SQL}) = 'expired' \
           AND (s.content IS NOT NULL OR s.summary IS NOT NULL)"
    ))
    .bind(company_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected())
}

#[derive(sqlx::FromRow)]
struct PourCleRow {
    id: i64,
    etat: EtatEnvoi,
    modified: bool,
    content: Option<String>,
}

/// Relit un envoi pour la clé qui l'a déposé.
///
/// `None` pour un envoi inexistant, d'une autre clé **ou** d'une autre société
/// — le même `None`. Sinon l'état dérivé, `modified`, et le contenu
/// **seulement** si l'état est [`EtatEnvoi::Valide`]. Un envoi validé vivant
/// sans contenu viole l'invariant de la table : erreur, jamais un texte vide.
pub async fn lire_pour_cle(
    pool: &MySqlPool,
    company_id: i64,
    mcp_key_id: i64,
    id: i64,
) -> Result<Option<EnvoiPourCle>, DbError> {
    let Some(row) = sqlx::query_as::<_, PourCleRow>(&format!(
        "SELECT s.id, ({ETAT_DERIVE_SQL}) AS etat, s.modified, s.content \
         FROM mcp_sends s \
         WHERE s.id = ? AND s.company_id = ? AND s.mcp_key_id = ? LIMIT 1"
    ))
    .bind(id)
    .bind(company_id)
    .bind(mcp_key_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?
    else {
        return Ok(None);
    };
    let contenu = match row.etat {
        EtatEnvoi::Valide => Some(row.content.ok_or_else(|| {
            DbError::Invariant(format!(
                "envoi MCP {} validé et vivant sans contenu",
                row.id
            ))
        })?),
        EtatEnvoi::EnAttente | EtatEnvoi::Refuse | EtatEnvoi::Echu => None,
    };
    Ok(Some(EnvoiPourCle {
        id: row.id,
        etat: row.etat,
        modified: row.modified,
        contenu,
    }))
}

/// Colonnes d'une ligne de liste (alias `s` pour l'envoi, `k` pour la clé,
/// `u` pour qui a décidé).
fn colonnes_resume() -> String {
    format!(
        "s.id, s.mcp_key_id, k.name AS mcp_key_name, \
         (k.revoked_at IS NOT NULL) AS mcp_key_revoked, s.tool_name, \
         ({ETAT_DERIVE_SQL}) AS etat, s.modified, s.expires_at, s.decided_at, \
         u.username AS decided_by, s.created_at"
    )
}

const JOINTURES: &str = "FROM mcp_sends s \
     JOIN mcp_keys k ON k.id = s.mcp_key_id \
     LEFT JOIN users u ON u.id = s.decided_by_user_id";

#[derive(sqlx::FromRow)]
struct ResumeRow {
    id: i64,
    mcp_key_id: i64,
    mcp_key_name: String,
    mcp_key_revoked: bool,
    tool_name: String,
    etat: EtatEnvoi,
    modified: bool,
    expires_at: NaiveDateTime,
    decided_at: Option<NaiveDateTime>,
    decided_by: Option<String>,
    created_at: NaiveDateTime,
}

impl From<ResumeRow> for EnvoiResume {
    fn from(r: ResumeRow) -> Self {
        EnvoiResume {
            id: r.id,
            mcp_key_id: r.mcp_key_id,
            mcp_key_name: r.mcp_key_name,
            mcp_key_revoked: r.mcp_key_revoked,
            tool_name: r.tool_name,
            etat: r.etat,
            modified: r.modified,
            expires_at: r.expires_at,
            decided_at: r.decided_at,
            decided_by: r.decided_by,
            created_at: r.created_at,
        }
    }
}

/// Liste les envois d'une société, **sans contenu ni résumé**, au plus
/// [`LISTE_MAX`] lignes : les envois en attente vivants d'abord, par échéance
/// croissante, puis les autres par création décroissante.
pub async fn lister(pool: &MySqlPool, company_id: i64) -> Result<Vec<EnvoiResume>, DbError> {
    Ok(sqlx::query_as::<_, ResumeRow>(&format!(
        "SELECT {} {JOINTURES} WHERE s.company_id = ? \
         ORDER BY (({ETAT_DERIVE_SQL}) = 'pending') DESC, \
                  CASE WHEN ({ETAT_DERIVE_SQL}) = 'pending' THEN s.expires_at END ASC, \
                  s.created_at DESC, s.id DESC \
         LIMIT ?",
        colonnes_resume()
    ))
    .bind(company_id)
    .bind(LISTE_MAX)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?
    .into_iter()
    .map(EnvoiResume::from)
    .collect())
}

#[derive(sqlx::FromRow)]
struct DetailRow {
    #[sqlx(flatten)]
    resume: ResumeRow,
    summary: Option<String>,
    content: Option<String>,
    version: i32,
}

/// Le détail d'un envoi de la société, ou `None` s'il n'existe pas ou
/// appartient à une autre société. Résumé et contenu sont `None` si l'envoi
/// est refusé ou échu — même si la purge n'est pas encore passée.
pub async fn trouver(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<EnvoiDetail>, DbError> {
    let Some(row) = sqlx::query_as::<_, DetailRow>(&format!(
        "SELECT {}, s.summary, s.content, s.version {JOINTURES} \
         WHERE s.company_id = ? AND s.id = ? LIMIT 1",
        colonnes_resume()
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?
    else {
        return Ok(None);
    };
    let lisible = matches!(row.resume.etat, EtatEnvoi::EnAttente | EtatEnvoi::Valide);
    let (summary, content) = if lisible {
        (row.summary, row.content)
    } else {
        (None, None)
    };
    Ok(Some(EnvoiDetail {
        content_bytes: content.as_ref().map(|c| c.len() as u64),
        resume: row.resume.into(),
        summary,
        content,
        version: row.version,
    }))
}

#[derive(sqlx::FromRow)]
struct VerrouRow {
    mcp_key_id: i64,
    tool_name: String,
    state: String,
    echu: bool,
    version: i32,
    content: Option<String>,
}

/// Décide d'un envoi **dans la transaction reçue**.
///
/// Verrouille la ligne **de la société** (`SELECT … FOR UPDATE`, échéance lue
/// dans la même requête), puis contrôle, dans cet ordre : absente →
/// [`ErreurDecision::Introuvable`] ; non `pending` →
/// [`ErreurDecision::DejaDecidee`] ; échue → [`ErreurDecision::Echue`] ;
/// version différente → [`ErreurDecision::VersionPerimee`].
///
/// **Valider** pose `approved` et, si un texte est reçu, l'écrit tel quel ;
/// `modified` vaut « le texte reçu diffère, octet pour octet, du texte
/// stocké ». **Refuser** pose `refused` et efface contenu et résumé. Les deux
/// posent `decided_at = NOW(3)`, `decided_by_user_id` et `version + 1`.
pub async fn decider_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    decision: Decision,
    user_id: i64,
) -> Result<EnvoiDecide, ErreurDecision> {
    let Some(row) = sqlx::query_as::<_, VerrouRow>(
        "SELECT s.mcp_key_id, s.tool_name, s.state, (s.expires_at <= NOW(3)) AS echu, \
                s.version, s.content \
         FROM mcp_sends s WHERE s.id = ? AND s.company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    else {
        return Err(ErreurDecision::Introuvable);
    };
    if row.state != "pending" {
        return Err(ErreurDecision::DejaDecidee);
    }
    if row.echu {
        return Err(ErreurDecision::Echue);
    }
    if row.version != version {
        return Err(ErreurDecision::VersionPerimee);
    }

    let (etat, modified, rows) = match decision {
        Decision::Valider { contenu } => {
            let modified = match &contenu {
                Some(recu) => row.content.as_deref().map(str::as_bytes) != Some(recu.as_bytes()),
                None => false,
            };
            let rows = sqlx::query(
                "UPDATE mcp_sends SET state = 'approved', content = COALESCE(?, content), \
                 modified = ?, decided_at = NOW(3), decided_by_user_id = ?, \
                 version = version + 1 \
                 WHERE id = ? AND company_id = ?",
            )
            .bind(contenu)
            .bind(modified)
            .bind(user_id)
            .bind(id)
            .bind(company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();
            (EtatEnvoi::Valide, modified, rows)
        }
        Decision::Refuser => {
            let rows = sqlx::query(
                "UPDATE mcp_sends SET state = 'refused', content = NULL, summary = NULL, \
                 modified = FALSE, decided_at = NOW(3), decided_by_user_id = ?, \
                 version = version + 1 \
                 WHERE id = ? AND company_id = ?",
            )
            .bind(user_id)
            .bind(id)
            .bind(company_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();
            (EtatEnvoi::Refuse, false, rows)
        }
    };
    if rows != 1 {
        return Err(DbError::Invariant(format!(
            "décision de l'envoi MCP {id} : {rows} ligne(s) écrite(s) sous verrou"
        ))
        .into());
    }

    Ok(EnvoiDecide {
        id,
        mcp_key_id: row.mcp_key_id,
        tool_name: row.tool_name,
        etat,
        modified,
        version: row.version + 1,
    })
}
