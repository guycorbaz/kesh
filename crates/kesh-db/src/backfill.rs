//! Backfill Rust du parc existant — Story 22-1 (#294/#295), décision **D6**.
//!
//! La migration `20260814000001` est du DDL pur : MariaDB ne sait ni
//! normaliser NFKC ni retirer un jeu ouvert de caractères invisibles, le
//! remplissage de `contacts.client_number_canonical` se fait donc ici, en
//! Rust, par [`backfill_client_number_canonical`] — appelée à **deux**
//! endroits, les seuls où du Rust s'exécute :
//!
//! - au **boot** (`kesh-api/src/main.rs`, juste après `MIGRATOR.run`) — c'est
//!   la forme concrète du « la migration refuse » de D5 : en cas de collision,
//!   **le boot refuse**, avec le rapport pour message ;
//! - en fin d'**import d'installation** (`admin.rs`, après
//!   `replay_post_restore_backfills`) — un `.keshbackup` antérieur à la story
//!   arrive avec la colonne vide ; c'est l'esprit du garde-fou **P7**, tenu
//!   sans entrée au registre SQL. Un backup en collision est refusé en `400`.
//!
//! La fonction est **idempotente** : elle ne remplit que les lignes dont la
//! canonique est `NULL`, et ne coûte qu'une requête quand tout est rempli.

use kesh_core::text::canonical_key;
use sqlx::MySqlConnection;
use std::collections::HashMap;
use std::fmt;

/// Un groupe de contacts **actifs** d'une même société dont les numéros de
/// client, distincts à l'octet, partagent la même forme canonique.
#[derive(Debug)]
pub struct ClientNumberCollision {
    pub company_id: i64,
    pub canonical: String,
    /// `(contact_id, valeur affichée)` — le rapport EST l'outil de réparation :
    /// il doit permettre de retrouver chaque fiche sans requête supplémentaire.
    pub contacts: Vec<(i64, String)>,
}

/// Échec du backfill : des collisions existent, **rien n'a été écrit**.
#[derive(Debug)]
pub struct ClientNumberCollisions(pub Vec<ClientNumberCollision>);

impl fmt::Display for ClientNumberCollisions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} groupe(s) de numéros de client en collision canonique — corriger \
             les fiches puis relancer :",
            self.0.len()
        )?;
        for c in &self.0 {
            let details: Vec<String> = c
                .contacts
                .iter()
                .map(|(id, v)| format!("contact {id} : {v:?}"))
                .collect();
            writeln!(
                f,
                "  société {} — canonique {:?} : {}",
                c.company_id,
                c.canonical,
                details.join(" / ")
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for ClientNumberCollisions {}

/// Erreurs du backfill : la base est injoignable, ou le parc porte des
/// collisions (auquel cas **rien** n'a été écrit).
#[derive(Debug)]
pub enum BackfillError {
    Db(sqlx::Error),
    Collisions(ClientNumberCollisions),
}

impl fmt::Display for BackfillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackfillError::Db(e) => write!(f, "backfill client_number_canonical : {e}"),
            BackfillError::Collisions(c) => write!(f, "{c}"),
        }
    }
}

impl std::error::Error for BackfillError {}

impl From<sqlx::Error> for BackfillError {
    fn from(e: sqlx::Error) -> Self {
        BackfillError::Db(e)
    }
}

/// Remplit `contacts.client_number_canonical` pour tout le parc, ou refuse en
/// nommant **toutes** les collisions sans rien écrire (D5/D6, Story 22-1).
///
/// Trois propriétés, chacune tenue par un test :
///
/// - **fail-loud d'abord** : le pré-scan porte sur l'ensemble des contacts
///   **actifs** de chaque société — y compris ceux dont la canonique est déjà
///   remplie — et rend le rapport complet avant la moindre écriture ;
/// - **idempotente** : seules les lignes à canonique `NULL` sont écrites ; un
///   second appel sur base remplie ne touche rien ;
/// - **la vacuité de D2 s'applique au parc** : une valeur historique dont la
///   canonique est vide (intégralement invisible) est ramenée à `NULL` sur
///   **les deux** colonnes — un « numéro » que personne ne voit n'identifie
///   rien, et il ne doit pas squatter l'unicité.
///
/// Les contacts **archivés** sont remplis aussi (cohérence du parc) mais ne
/// participent pas aux collisions : la colonne générée les exclut de la
/// contrainte, et aucune route ne réactive (cf. § Périmètre de la story).
///
/// Retourne le nombre de lignes écrites.
pub async fn backfill_client_number_canonical(
    conn: &mut MySqlConnection,
) -> Result<u64, BackfillError> {
    // Un seul chargement : tous les contacts porteurs d'un numéro. Le parc se
    // compte en centaines de fiches sur une installation Kesh — tenir tout en
    // mémoire est le cas nominal, pas une approximation.
    // `client_number_canonical IS NOT NULL` plutôt que la valeur : la colonne
    // porte `utf8mb4_bin`, que sqlx expose comme VARBINARY — et seule
    // l'EXISTENCE compte ici (idempotence), la valeur est recalculée en Rust.
    let rows: Vec<(i64, i64, bool, String, bool)> = sqlx::query_as(
        "SELECT id, company_id, active, client_number, \
         (client_number_canonical IS NOT NULL) \
         FROM contacts WHERE client_number IS NOT NULL",
    )
    .fetch_all(&mut *conn)
    .await?;

    // Pré-scan des collisions sur les ACTIFS, canonique calculée en Rust.
    let mut by_key: HashMap<(i64, String), Vec<(i64, String)>> = HashMap::new();
    for (id, company_id, active, displayed, _) in &rows {
        if !active {
            continue;
        }
        let canonical = canonical_key(displayed);
        if canonical.is_empty() {
            continue; // vacuité : sort de l'unicité, ramené à NULL plus bas
        }
        by_key
            .entry((*company_id, canonical))
            .or_default()
            .push((*id, displayed.clone()));
    }
    let mut collisions: Vec<ClientNumberCollision> = by_key
        .into_iter()
        .filter(|(_, contacts)| contacts.len() > 1)
        .map(|((company_id, canonical), mut contacts)| {
            contacts.sort();
            ClientNumberCollision {
                company_id,
                canonical,
                contacts,
            }
        })
        .collect();
    if !collisions.is_empty() {
        collisions.sort_by_key(|c| (c.company_id, c.canonical.clone()));
        return Err(BackfillError::Collisions(ClientNumberCollisions(
            collisions,
        )));
    }

    // Remplissage — uniquement les lignes encore NULL (idempotence).
    let mut written = 0u64;
    for (id, _, _, displayed, already_filled) in &rows {
        if *already_filled {
            continue;
        }
        let canonical = canonical_key(displayed);
        let result = if canonical.is_empty() {
            sqlx::query(
                "UPDATE contacts SET client_number = NULL, client_number_canonical = NULL \
                 WHERE id = ?",
            )
            .bind(id)
            .execute(&mut *conn)
            .await?
        } else {
            sqlx::query("UPDATE contacts SET client_number_canonical = ? WHERE id = ?")
                .bind(&canonical)
                .bind(id)
                .execute(&mut *conn)
                .await?
        };
        written += result.rows_affected();
    }
    Ok(written)
}
