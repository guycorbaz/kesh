//! Les poignées sur la base : [`Lecture`] pour lire les livres, [`Trace`]
//! pour écrire le journal d'audit — et, depuis la Story 32-8b, les deux
//! poignées des envois du serveur MCP, [`LectureEnvois`] et
//! [`EcritureEnvois`], qui n'exposent aucun pool ; depuis la Story 32-10a-ii,
//! la lecture des prompts, [`LecturePrompts`], de même, et depuis la
//! 32-10a-iii leur écriture par un agent, [`EcriturePrompts`].
//!
//! Deux types distincts, sans conversion de l'un à l'autre, à champ privé : un
//! service qui reçoit une `Lecture` ne peut pas en faire une `Trace`, et le pool
//! d'une poignée n'est lisible que dans `crate::services`. En production, elles
//! ne se fabriquent que par `AppState::lecture()`, `AppState::trace()` et, pour
//! la lecture des outils du MCP, au câblage de `build_router_avec_registre`
//! (Story 32-6a).
//!
//! La lecture seule du MCP (Q11, tranchée le 2026-10-24) est tenue par
//! l'application — les outils n'atteignent que des services qui reçoivent une
//! `Lecture` — et par le mode de session `READ ONLY` de la connexion des
//! outils (`kesh_db::pool::pool_lecture_seule`, 32-6a) ; aucun second compte
//! MariaDB. `AppState::lecture()` et `AppState::trace()` enveloppent le pool
//! ordinaire.

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::mcp_prompt::{ErreurEcriturePrompt, VersionEcrite, VersionEnVigueur};
use kesh_db::entities::mcp_send::{EnvoiPourCle, ErreurDepot, NouvelEnvoi};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::repositories::{audit_log, mcp_prompts, mcp_sends};
use sqlx::MySqlPool;

/// Poignée de **lecture** des livres : dépôts, générateurs de rapports.
///
/// Aucune écriture ne passe par elle (règle R4 de `services`).
#[derive(Clone)]
pub struct Lecture(MySqlPool);

impl Lecture {
    /// Enveloppe un pool. En production, `AppState::lecture()` et le câblage du
    /// MCP, `build_router_avec_registre` (sur le pool en lecture seule des
    /// outils), l'appellent.
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Le pool de lecture, pour les dépôts et les générateurs — visible de la
    /// seule couche de services.
    pub(in crate::services) fn pool(&self) -> &MySqlPool {
        &self.0
    }
}

/// Poignée de **relecture des envois** du serveur MCP (Story 32-8b) — posée sur
/// le pool **en lecture seule** du MCP, celui des outils. Une seule méthode,
/// [`LectureEnvois::lire_pour_cle`] ; **aucun** accès au pool : champ privé,
/// sans méthode `pool()`.
#[derive(Clone)]
pub struct LectureEnvois(MySqlPool);

impl LectureEnvois {
    /// Enveloppe le pool en lecture seule du MCP — appelé une fois, au câblage
    /// de `build_router_avec_registre` (garde (v-2) de `services_motif.rs`).
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Relit un envoi pour la clé qui l'a déposé, dans sa société
    /// (`mcp_sends::lire_pour_cle`).
    pub(in crate::services) async fn lire_pour_cle(
        &self,
        company_id: i64,
        mcp_key_id: i64,
        id: i64,
    ) -> Result<Option<EnvoiPourCle>, DbError> {
        mcp_sends::lire_pour_cle(&self.0, company_id, mcp_key_id, id).await
    }
}

/// Poignée de **lecture des prompts** du serveur MCP (Story 32-10a-ii, AC-ii3,
/// C-32-10a-30) — posée sur le pool **en lecture seule** du MCP, celui des
/// outils. Deux méthodes, les versions en vigueur de la société ou d'un nom ;
/// **aucun** accès au pool : champ privé, sans méthode `pool()`. Le service des
/// prompts (`services::prompts_mcp`) ne reçoit qu'elle : il ne peut lire ni les
/// livres ni une autre table que celles des prompts (P19 de la fiche).
#[derive(Clone)]
pub struct LecturePrompts(MySqlPool);

impl LecturePrompts {
    /// Enveloppe le pool en lecture seule du MCP — appelé une fois, au câblage
    /// de `build_router_avec_registre` (garde (v-2) de `services_motif.rs`).
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Les versions en vigueur, de nature texte, des prompts de la société
    /// (`mcp_prompts::en_vigueur_par_societe`).
    pub(in crate::services) async fn en_vigueur_par_societe(
        &self,
        company_id: i64,
    ) -> Result<Vec<VersionEnVigueur>, DbError> {
        mcp_prompts::en_vigueur_par_societe(&self.0, company_id).await
    }

    /// La version en vigueur, de nature texte, du prompt `nom` de la société
    /// (`mcp_prompts::en_vigueur_par_nom`).
    pub(in crate::services) async fn en_vigueur_par_nom(
        &self,
        company_id: i64,
        nom: &str,
    ) -> Result<Option<VersionEnVigueur>, DbError> {
        mcp_prompts::en_vigueur_par_nom(&self.0, company_id, nom).await
    }
}

/// Poignée d'**écriture des envois** du serveur MCP (Story 32-8b) — sur le
/// pool **ordinaire**. Deux méthodes : le dépôt d'un envoi **et** l'entrée
/// d'audit de son appel, en une transaction ([`EcritureEnvois::deposer`]), et
/// la purge des contenus échus ([`EcritureEnvois::purger_echus`]). Champ privé,
/// sans méthode `pool()`.
#[derive(Clone)]
pub struct EcritureEnvois(MySqlPool);

impl EcritureEnvois {
    /// Enveloppe le pool ordinaire — appelé une fois, au câblage de
    /// `build_router_avec_registre`.
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Dépose `envoi` et écrit l'entrée d'audit que `entree` construit à
    /// partir de son numéro, **dans une seule transaction** : un échec de
    /// l'une annule l'autre. `TropNombreux` n'écrit rien.
    pub(in crate::services) async fn deposer(
        &self,
        envoi: &NouvelEnvoi,
        duree_s: u32,
        plafond: u32,
        entree: impl FnOnce(i64) -> NewAuditLogEntry,
    ) -> Result<i64, ErreurDepot> {
        let mut tx = self.0.begin().await.map_err(map_db_error)?;
        let id = mcp_sends::deposer_in_tx(&mut tx, envoi, duree_s, plafond).await?;
        audit_log::insert_in_tx(&mut tx, entree(id)).await?;
        tx.commit().await.map_err(map_db_error)?;
        Ok(id)
    }

    /// Efface contenu et résumé des envois échus de la société
    /// (`mcp_sends::purger_echus`).
    pub(in crate::services) async fn purger_echus(&self, company_id: i64) -> Result<u64, DbError> {
        mcp_sends::purger_echus(&self.0, company_id).await
    }
}

/// Poignée d'**écriture des prompts** du serveur MCP (Story 32-10a-iii,
/// AC-iii1, C-32-10a-30) — sur le pool **ordinaire**. Une seule méthode,
/// [`EcriturePrompts::proposer`] : la version d'agent **à valider** et l'entrée
/// d'audit de son appel, en une transaction. Champ privé, sans méthode
/// `pool()` : le service des prompts ne peut rien écrire d'autre.
#[derive(Clone)]
pub struct EcriturePrompts(MySqlPool);

impl EcriturePrompts {
    /// Enveloppe le pool ordinaire — appelé une fois, au câblage de
    /// `build_router_avec_registre` (garde (v-2) de `services_motif.rs`).
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Écrit la proposition d'un agent (`mcp_prompts::proposer_version_agent_in_tx`)
    /// **et** l'entrée d'audit que `entree` construit, **dans une seule
    /// transaction** : un échec de l'une annule l'autre (AC-iii3), et un refus
    /// du dépôt (nom réservé, plafond…) n'écrit rien.
    ///
    /// ⚠️ **Précondition de lecture** (revue de code P1 de la 32-10a-i, #783
    /// point 1) : le plafond (`COUNT`) et la numérotation (`MAX(number) + 1`)
    /// du dépôt sont des lectures **simples** sous le verrou de société ; en
    /// `REPEATABLE READ`, la vue cohérente d'InnoDB naît à la première lecture
    /// simple de la transaction. Ils ne sont donc justes que si **rien n'est lu
    /// dans la transaction avant** l'appel du dépôt, dont la première
    /// instruction est le verrou de société (une lecture verrouillante, qui ne
    /// fige aucune vue). C'est le cas ici : la transaction s'ouvre, le dépôt
    /// est appelé aussitôt. Test `plafond_juste_apres_l_attente_du_verrou`
    /// (`tests/mcp_prompts_e2e.rs`).
    ///
    /// La société et la clé sont celles de la clé authentifiée (le service les
    /// tire de `CleMcpAuthentifiee`) : une clé n'authentifie que dans sa société
    /// (`cles_mcp::authentifier`), si bien que la clé auteur appartient toujours
    /// à la société de la version (#783 point 4).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::services) async fn proposer(
        &self,
        company_id: i64,
        cle_id: i64,
        nom: &str,
        description: &str,
        texte: &str,
        plafond: u32,
        entree: impl FnOnce(&VersionEcrite) -> NewAuditLogEntry,
    ) -> Result<VersionEcrite, ErreurEcriturePrompt> {
        let mut tx = self.0.begin().await.map_err(map_db_error)?;
        let ecrite = mcp_prompts::proposer_version_agent_in_tx(
            &mut tx,
            company_id,
            cle_id,
            nom,
            description,
            texte,
            plafond,
        )
        .await?;
        audit_log::insert_in_tx(&mut tx, entree(&ecrite)).await?;
        tx.commit().await.map_err(map_db_error)?;
        Ok(ecrite)
    }
}

/// Poignée d'**écriture de la trace** : le journal d'audit des lectures.
#[derive(Clone)]
pub struct Trace(MySqlPool);

impl Trace {
    /// Enveloppe un pool. En production, seul `AppState::trace()` l'appelle.
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Le pool de la trace, destiné aux seules fonctions d'audit de
    /// `services/rapports.rs`, à la dernière utilisation d'une clé MCP
    /// (`services/cles_mcp.rs`, Story 32-5b) et au journal des appels d'outil
    /// MCP (`serveur_mcp::journaliser_appel`, Story 32-6b) et des `prompts/get`
    /// (`serveur_mcp::journaliser_prompt`, Story 32-10a-iii) — les deux seules
    /// de ces écritures qui ne sont pas *best-effort*. ⚠️ La visibilité ne peut pas être
    /// plus étroite (`pub(in …)` n'accepte qu'un module ancêtre, E0742) :
    /// qu'aucun autre service ne l'appelle est tenu par la revue.
    pub(in crate::services) fn pool(&self) -> &MySqlPool {
        &self.0
    }
}
