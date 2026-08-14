//! Backfill Rust du parc existant — Story 22-1 (#294/#295), décision **D6**.
//!
//! La migration `20260814000001` est du DDL pur : MariaDB ne sait ni
//! normaliser NFKC ni retirer un jeu ouvert de caractères invisibles, le
//! remplissage de `contacts.client_number_canonical` se fait donc ici, en
//! Rust, par [`backfill_client_number_canonical`] — appelée à **deux**
//! endroits, les seuls où du Rust s'exécute :
//!
//! - au **boot** (`kesh-api/src/main.rs`, juste après `MIGRATOR.run`) — c'est
//!   la forme concrète du « la migration refuse » de D5 : en cas de refus,
//!   **le boot refuse**, avec le rapport pour message ;
//! - en fin d'**import d'installation** (`admin.rs`, après
//!   `replay_post_restore_backfills`) — un `.keshbackup` antérieur à la story
//!   arrive avec la colonne vide ; c'est l'esprit du garde-fou **P7**, tenu
//!   sans entrée au registre SQL. Un backup refusé l'est en `400` nominatif.
//!
//! Depuis la passe 1 de revue, la fonction est une **réconciliation**, pas un
//! simple remplissage : elle recalcule la canonique de CHAQUE ligne porteuse
//! d'un numéro et répare toute valeur stockée divergente — une canonique
//! périmée (un `client_number` modifié par SQL direct sans sa canonique)
//! rouvrirait #294 en silence, l'index d'unicité ne comparant que la valeur
//! STOCKÉE. Idempotente : sur une base saine, zéro écriture.
//!
//! ⚠️ **Aucune écriture n'est auditée dans `audit_log`** — c'est la convention
//! des backfills du registre P7 (`post_restore.rs`), qui n'auditent pas non
//! plus : ce sont des opérations de MAINTENANCE DE SCHÉMA, pas des actions
//! utilisateur, et le boot n'a d'ailleurs aucun acteur à inscrire. *(Relevé en
//! passe 1 de revue, classé conforme à la convention — documenté ici pour que
//! la question ne se repose pas.)*

use kesh_core::text::{CLIENT_NUMBER_MAX_CHARS, canonical_key};
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

/// Un contact dont la forme canonique du numéro DÉPASSE la colonne qui doit la
/// recevoir — NFKC et le repli de casse peuvent ALLONGER (`ﬁ` → `fi`, `İ` →
/// `i`+U+0307) : 50 caractères saisis peuvent en canoniser 100. Sans cette
/// catégorie, la ligne échouerait en erreur SQL `1406` brute, sans nommer la
/// fiche — un boot en boucle au message inutilisable. *(Relevé en passe 1 de
/// revue, prouvé par exécution.)*
#[derive(Debug)]
pub struct OverlongClientNumber {
    pub company_id: i64,
    pub contact_id: i64,
    pub displayed: String,
    pub canonical_chars: usize,
}

/// Rapport de refus du backfill : collisions et/ou canoniques trop longues.
/// **Rien n'a été écrit.** Le `Display` est le message que voient l'exploitant
/// (boot) et l'appelant de l'import (`details.report`) — les valeurs y sont
/// rendues en `{:?}`, ce qui ESCAPE les invisibles : on ne répare pas ce qu'on
/// ne voit pas.
#[derive(Debug, Default)]
pub struct ClientNumberBackfillRefusal {
    pub collisions: Vec<ClientNumberCollision>,
    pub overlong: Vec<OverlongClientNumber>,
}

impl ClientNumberBackfillRefusal {
    fn is_empty(&self) -> bool {
        self.collisions.is_empty() && self.overlong.is_empty()
    }
}

impl fmt::Display for ClientNumberBackfillRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "numéros de client irrecevables — corriger les fiches nommées puis relancer :"
        )?;
        if !self.collisions.is_empty() {
            writeln!(
                f,
                "  {} groupe(s) en collision canonique :",
                self.collisions.len()
            )?;
            for c in &self.collisions {
                let details: Vec<String> = c
                    .contacts
                    .iter()
                    .map(|(id, v)| format!("contact {id} : {v:?}"))
                    .collect();
                writeln!(
                    f,
                    "    société {} — canonique {:?} : {}",
                    c.company_id,
                    c.canonical,
                    details.join(" / ")
                )?;
            }
        }
        if !self.overlong.is_empty() {
            writeln!(
                f,
                "  {} numéro(s) dont la forme normalisée dépasse {} caractères \
                 (les accents et ligatures décomposés comptent) :",
                self.overlong.len(),
                CLIENT_NUMBER_MAX_CHARS
            )?;
            for o in &self.overlong {
                writeln!(
                    f,
                    "    société {} — contact {} : {:?} ({} caractères normalisés)",
                    o.company_id, o.contact_id, o.displayed, o.canonical_chars
                )?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ClientNumberBackfillRefusal {}

/// Erreurs du backfill : la base est injoignable, ou le parc est irrecevable
/// (auquel cas **rien** n'a été écrit).
#[derive(Debug)]
pub enum BackfillError {
    Db(sqlx::Error),
    Refused(ClientNumberBackfillRefusal),
}

impl fmt::Display for BackfillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackfillError::Db(e) => write!(f, "backfill client_number_canonical : {e}"),
            BackfillError::Refused(r) => write!(f, "{r}"),
        }
    }
}

impl std::error::Error for BackfillError {}

impl From<sqlx::Error> for BackfillError {
    fn from(e: sqlx::Error) -> Self {
        BackfillError::Db(e)
    }
}

/// Réconcilie `contacts.client_number_canonical` avec le parc, ou refuse en
/// nommant **tout** ce qui est irrecevable, sans rien écrire (D5/D6, 22-1).
///
/// Quatre propriétés, chacune tenue par un test :
///
/// - **fail-loud d'abord** : le pré-scan porte sur l'ensemble des contacts
///   porteurs d'un numéro et rend le rapport COMPLET (collisions entre actifs
///   + canoniques trop longues, actifs ou archivés) avant la moindre écriture ;
/// - **réconciliation, pas remplissage** : toute canonique stockée divergente
///   de la canonique RECALCULÉE est réparée — y compris une valeur périmée
///   laissée par une modification SQL directe de `client_number` ;
/// - **idempotente** : sur une base saine, zéro écriture ;
/// - **la vacuité de D2 s'applique au parc** : une valeur dont la canonique
///   est vide est ramenée à `NULL` sur **les deux** colonnes.
///
/// Les contacts **archivés** sont réconciliés aussi (cohérence du parc) mais
/// ne participent pas aux collisions : la colonne générée les exclut de la
/// contrainte, et aucune route ne réactive (cf. § Périmètre de la story). Une
/// canonique trop longue, elle, est refusée MÊME sur un archivé — la colonne
/// la refuse physiquement, actif ou non.
///
/// Retourne le nombre de lignes écrites.
pub async fn backfill_client_number_canonical(
    conn: &mut MySqlConnection,
) -> Result<u64, BackfillError> {
    // Un seul chargement : tous les contacts porteurs d'un numéro. Le parc se
    // compte en centaines de fiches sur une installation Kesh — tenir tout en
    // mémoire est le cas nominal, pas une approximation. Le `CAST(... AS CHAR)`
    // est nécessaire : la colonne porte `utf8mb4_bin`, que sqlx expose sinon
    // comme VARBINARY.
    let rows: Vec<(i64, i64, bool, String, Option<String>)> = sqlx::query_as(
        "SELECT id, company_id, active, client_number, \
         CAST(client_number_canonical AS CHAR) \
         FROM contacts WHERE client_number IS NOT NULL",
    )
    .fetch_all(&mut *conn)
    .await?;

    // Pré-scan : canonique RECALCULÉE pour chaque ligne. Collisions sur les
    // actifs, longueur sur tout le monde.
    let mut refusal = ClientNumberBackfillRefusal::default();
    let mut by_key: HashMap<(i64, String), Vec<(i64, String)>> = HashMap::new();
    for (id, company_id, active, displayed, _) in &rows {
        let canonical = canonical_key(displayed);
        if canonical.is_empty() {
            continue; // vacuité : sort de l'unicité, ramené à NULL plus bas
        }
        if canonical.chars().count() > CLIENT_NUMBER_MAX_CHARS {
            refusal.overlong.push(OverlongClientNumber {
                company_id: *company_id,
                contact_id: *id,
                displayed: displayed.clone(),
                canonical_chars: canonical.chars().count(),
            });
            continue; // irrecevable — inutile de la mettre en collision
        }
        if !active {
            continue; // hors contrainte, mais réconcilié plus bas
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
    collisions.sort_by_key(|c| (c.company_id, c.canonical.clone()));
    refusal.collisions = collisions;
    refusal
        .overlong
        .sort_by_key(|o| (o.company_id, o.contact_id));
    if !refusal.is_empty() {
        return Err(BackfillError::Refused(refusal));
    }

    // Réconciliation — n'écrit que là où la canonique stockée diverge de la
    // canonique recalculée (idempotence : base saine = zéro écriture).
    let mut written = 0u64;
    for (id, _, _, displayed, stored) in &rows {
        let canonical = canonical_key(displayed);
        let result = if canonical.is_empty() {
            // Vacuité D2 : les deux colonnes à NULL. `client_number IS NOT
            // NULL` dans le WHERE rend l'écriture auto-idempotente.
            sqlx::query(
                "UPDATE contacts SET client_number = NULL, client_number_canonical = NULL \
                 WHERE id = ? AND client_number IS NOT NULL",
            )
            .bind(id)
            .execute(&mut *conn)
            .await?
        } else if stored.as_deref() != Some(canonical.as_str()) {
            sqlx::query("UPDATE contacts SET client_number_canonical = ? WHERE id = ?")
                .bind(&canonical)
                .bind(id)
                .execute(&mut *conn)
                .await?
        } else {
            continue;
        };
        written += result.rows_affected();
    }
    Ok(written)
}
