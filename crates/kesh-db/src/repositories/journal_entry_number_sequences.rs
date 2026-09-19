//! Compteur de numéros d'écriture, par société et par exercice — Story 25-2-c (#381).
//!
//! # Pourquoi un compteur, et pas un `MAX + 1`
//!
//! `create_in_tx` tirait son numéro d'un `SELECT COALESCE(MAX(entry_number), 0) + 1`.
//! L'`UNIQUE (company_id, fiscal_year_id, entry_number)` garantit l'unicité **à un
//! instant donné** — ni la contiguïté, ni l'**univocité dans le temps** :
//!
//! - supprimer l'écriture n° 42 au milieu laisse un **trou** définitif. Il est
//!   **visible**, et un contrôleur en demandera l'explication — que le journal
//!   d'audit porte ;
//! - supprimer la **dernière** fait **réattribuer** son numéro à une écriture au
//!   contenu différent. Celui-là est **muet** : rien ne le signale, jamais.
//!
//! ⛔ **C'est la réattribution que ce module ferme**, parce qu'un compteur ne
//! redescend jamais. Le trou, lui, subsiste — et c'est assumé : le combler
//! exigerait de renuméroter des écritures existantes, ce que le gel de l'Epic 24
//! interdit précisément.
//!
//! # Ce module n'invente rien
//!
//! Il transpose [`super::invoice_number_sequences`], en service depuis avril et
//! déjà réemployé par [`super::credit_note_number_sequences`]. Même portée, même
//! séquence d'opérations, mêmes garde-fous — jusqu'au contrôle de
//! `rows_affected()`. *Deux compteurs qui font la même chose doivent la faire de
//! la même façon : sans quoi le prochain correctif n'est appliqué qu'à une
//! moitié.*

use sqlx::MySql;

use crate::errors::{DbError, map_db_error};

/// Rend le prochain numéro d'écriture pour `(company_id, fiscal_year_id)` et
/// **consomme** ce numéro.
///
/// ⚠️ **La sérialisation change de nature, pas d'existence.** Le `MAX + 1`
/// qu'elle remplace était sérialisé par un *gap lock* sur l'index
/// `(company_id, fiscal_year_id, entry_number)` ; ici c'est le verrou de **ligne**
/// posé par le `SELECT … FOR UPDATE` sur le compteur qui tient ce rôle. Deux
/// créations concurrentes dans le même exercice ne peuvent donc toujours pas
/// obtenir le même numéro — et l'`UNIQUE` reste le filet, exactement comme pour
/// les factures.
///
/// La ligne est créée **à la demande** : une société qui n'a jamais écrit dans un
/// exercice n'a pas de ligne, et la première écriture la pose. L'`INSERT IGNORE`
/// absorbe la course où deux transactions la créeraient en même temps.
///
/// ⚠️ **Précondition : l'appelant a vérifié que la société et l'exercice
/// existent.** En MariaDB, `INSERT IGNORE` ne se contente pas d'absorber le
/// doublon : il change aussi une violation de clé étrangère en simple
/// avertissement. Sur un exercice inexistant, la ligne ne serait donc pas créée,
/// et la relecture échouerait en `DbError::NotFound` sans nommer la cause.
/// `create_in_tx`, seul appelant de production, verrouille l'exercice et rejette
/// son absence avant d'arriver ici.
pub async fn next_number_for(
    tx: &mut sqlx::Transaction<'_, MySql>,
    company_id: i64,
    fiscal_year_id: i64,
) -> Result<i64, DbError> {
    // Étape 1 : verrouiller la ligne du compteur si elle existe.
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT next_number FROM journal_entry_number_sequences \
         WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let next_number = match current {
        Some(n) => n,
        None => {
            // Création paresseuse. `INSERT IGNORE` parce qu'un concurrent a pu
            // créer la ligne entre notre SELECT et cet INSERT : l'erreur 1062
            // serait alors un faux échec.
            sqlx::query(
                "INSERT IGNORE INTO journal_entry_number_sequences \
                 (company_id, fiscal_year_id, next_number) VALUES (?, ?, 1)",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;

            // Re-lecture VERROUILLANTE : que la ligne vienne de nous ou d'un
            // concurrent, il faut la tenir avant de l'incrémenter.
            sqlx::query_scalar(
                "SELECT next_number FROM journal_entry_number_sequences \
                 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?
        }
    };

    // Étape 2 : **le plus grand des deux** — le compteur, ou le plus grand numéro
    // réellement en service augmenté de un.
    //
    // ⛔ **Pourquoi, et ce n'est pas une précaution de confort.** Le compteur
    // n'avance que si l'on passe par ici. Qu'une écriture arrive autrement — SQL
    // direct, import partiel, intervention en base, choses qui arrivent en
    // comptabilité — et le compteur reste en retard : il rendrait alors un numéro
    // déjà pris, l'`UNIQUE` refuserait l'insertion, et **toute création
    // d'écriture échouerait indéfiniment** jusqu'à une intervention manuelle.
    // Pour un logiciel de comptabilité, un blocage total de la saisie est un mode
    // de panne inacceptable ; ce rattrapage le remplace par une réparation
    // silencieuse.
    //
    // ⚠️ **Il ne coûte RIEN à la propriété que la story installe.** Supprimer la
    // dernière écriture fait redescendre `MAX + 1`, jamais le compteur : c'est
    // donc le compteur qui l'emporte, et le numéro libéré n'est pas réattribué.
    // Le rattrapage ne peut jouer que vers le HAUT.
    //
    // C'est exactement ce que fait la migration à l'amorçage — ici en continu.
    //
    // ⛔ **La lecture est VERROUILLANTE, et c'est ce qui la rend juste.** Sous
    // `REPEATABLE READ`, un `SELECT` ordinaire lit l'instantané pris à la
    // première lecture de la transaction — dans `create_in_tx`, bien avant
    // d'arriver ici. Il ignorerait une écriture validée entre-temps, rendrait
    // son numéro, et l'`UNIQUE` refuserait l'insertion : le rattrapage raterait
    // exactement le cas pour lequel il existe. `FOR UPDATE` lit le dernier état
    // validé. Le verrou d'intervalle qu'il pose sur
    // `(company_id, fiscal_year_id, entry_number)` est celui que posait déjà le
    // `MAX + 1` d'avant cette story ; il est pris APRÈS le verrou du compteur,
    // que toute création prend en premier — pas de cycle entre créations.
    // Prouvé par `le_plancher_voit_une_ecriture_validee_pendant_la_transaction`.
    let plancher: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let next_number = next_number.max(plancher);

    // Étape 3 : consommer le numéro retenu.
    let rows = sqlx::query(
        "UPDATE journal_entry_number_sequences \
         SET next_number = ?, version = version + 1 \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(next_number + 1)
    .bind(company_id)
    .bind(fiscal_year_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    // ⚠️ Un `UPDATE` qui touche 0 ou 2 lignes signifierait que la ligne a disparu
    // sous le verrou, ou que l'`UNIQUE (company_id, fiscal_year_id)` a cédé. Dans
    // les deux cas le numéro rendu serait faux, et le silence pire que l'échec.
    if rows != 1 {
        return Err(DbError::Invariant(format!(
            "UPDATE journal_entry_number_sequences : {rows} rows affectées (attendu 1)"
        )));
    }

    Ok(next_number)
}
