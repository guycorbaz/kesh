//! Soldes de comptes **en devise** (Story 37-4, AC-1 et AC-1 quater).
//!
//! # La règle unique
//!
//! Le solde en devise d'un compte est Σ `amount_currency` des lignes au débit
//! moins Σ des lignes au crédit. Le **côté** d'une ligne est celui de son
//! montant en devise principale (`debit > 0` ⇒ débit, `credit > 0` ⇒ crédit) :
//! `amount_currency` n'a pas de signe (C-37-3). La **devise** est celle du
//! compte (`accounts.currency`), jamais celle d'une ligne — la ligne n'en porte
//! pas (CR-4, T3) ; un compte en devise n'a que des lignes dans sa devise
//! (verrou de la 37-1, contrôle central de la 37-3a).
//!
//! Une ligne en devise principale porte `amount_currency` à `NULL`, que `SUM`
//! ignore ; une ligne `revaluation` porte un montant nul et n'ajoute rien.
//!
//! # Pourquoi ce module vit dans `kesh-db`
//!
//! Le fragment SQL de la somme ([`somme_en_devise`]) est partagé par les
//! rapports (`kesh-report`) et par [`balances_as_of`], qu'appelle la
//! réévaluation de fin d'exercice (37-5), fonction de dépôt : `kesh-db` ne
//! dépend de `kesh-report` qu'en développement, l'inverse ferait un cycle.
//! C'est le seul crate visible des deux côtés.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySqlConnection;

use crate::errors::{DbError, map_db_error};

/// Côté d'une ligne d'écriture, lu sur son montant en devise principale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cote {
    /// `debit > 0`.
    Debit,
    /// `credit > 0`.
    Credit,
}

/// Fragment SQL de la somme **en devise** d'un côté, sur les lignes d'alias
/// `alias` (qui doit projeter `debit`, `credit` et `amount_currency`).
///
/// `condition`, si elle est donnée, est un prédicat SQL ajouté au `CASE`
/// (avec ses éventuels `?`, à lier par l'appelant **à la place** où le
/// fragment est écrit) : c'est ce qui permet aux rapports de borner chaque
/// somme comme leur somme en devise principale (avant la période, depuis le
/// début de l'exercice, sur la période…).
///
/// Aucune condition de devise sur la ligne (la colonne n'existe pas, T3) ; la
/// somme est enveloppée `COALESCE(…, 0)`, comme celles en devise principale :
/// un compte en devise sans ligne rend 0, non `NULL`. C'est à l'appelant de
/// rendre `NULL` pour un compte sans devise (`a.currency IS NULL`).
///
/// ```
/// use kesh_db::repositories::account_balances::{Cote, somme_en_devise};
/// assert_eq!(
///     somme_en_devise("jel", Cote::Debit, None),
///     "COALESCE(SUM(CASE WHEN jel.debit > 0 THEN jel.amount_currency ELSE 0 END), 0)"
/// );
/// assert_eq!(
///     somme_en_devise("x", Cote::Credit, Some("x.entry_date < ?")),
///     "COALESCE(SUM(CASE WHEN (x.entry_date < ?) AND x.credit > 0 \
///      THEN x.amount_currency ELSE 0 END), 0)"
/// );
/// ```
pub fn somme_en_devise(alias: &str, cote: Cote, condition: Option<&str>) -> String {
    let colonne = match cote {
        Cote::Debit => "debit",
        Cote::Credit => "credit",
    };
    let garde = match condition {
        Some(c) => format!("({c}) AND {alias}.{colonne} > 0"),
        None => format!("{alias}.{colonne} > 0"),
    };
    format!("COALESCE(SUM(CASE WHEN {garde} THEN {alias}.amount_currency ELSE 0 END), 0)")
}

/// Solde d'un compte à une date : `(account_id, chf_net, devise_net)`.
///
/// `chf_net` = Σ débit − Σ crédit en devise principale ; `devise_net` = la même
/// grandeur en devise (règle de [`somme_en_devise`]), `None` pour un compte
/// sans devise. **Sens débit brut**, sans signe par type de compte.
pub type SoldeALaDate = (i64, Decimal, Option<Decimal>);

/// Soldes cumulés à `as_of` (inclus), pour la réévaluation de fin d'exercice
/// (Story 37-5) — AC-1 quater de la 37-4.
///
/// - Borne **inclusive** `entry_date <= as_of`, **tous exercices**, sans borne
///   basse : une réévaluation déjà passée au même jour est comptée (en devise
///   principale ; elle n'ajoute rien en devise), ce qui rend exact le rejeu de
///   la réévaluation (seul le résidu reste à passer).
/// - `account_ids = None` : tous les comptes **en devise** de la société ;
///   `Some(ids)` : exactement ces comptes, bornés à la société — un identifiant
///   étranger à la société est **ignoré**, sans erreur (KF-002 ; l'appelant
///   compare le nombre de lignes rendues à celui demandé) —, chacun rendu même
///   sans ligne (`(id, 0, Some(0))`, ou `(id, 0, None)` pour un compte sans
///   devise).
/// - `Some(&[])` rend un vecteur vide **sans requête** (un `IN ()` serait une
///   erreur de syntaxe ; une liste vide ne vaut pas `None`).
/// - Lecture ordinaire, **non verrouillante**, dans la connexion fournie :
///   c'est le verrou d'exercice de l'appelant qui la stabilise.
///
/// Résultat trié par numéro de compte.
pub async fn balances_as_of(
    conn: &mut MySqlConnection,
    company_id: i64,
    as_of: NaiveDate,
    account_ids: Option<&[i64]>,
) -> Result<Vec<SoldeALaDate>, DbError> {
    if account_ids.is_some_and(|ids| ids.is_empty()) {
        return Ok(Vec::new());
    }
    let filtre = match account_ids {
        Some(ids) => format!(
            "a.id IN ({})",
            std::iter::repeat_n("?", ids.len())
                .collect::<Vec<_>>()
                .join(",")
        ),
        None => "a.currency IS NOT NULL".to_string(),
    };
    let sql = format!(
        "SELECT a.id, \
                COALESCE(SUM(x.debit), 0) - COALESCE(SUM(x.credit), 0) AS chf_net, \
                CASE WHEN a.currency IS NULL THEN NULL \
                     ELSE {d} - {c} END AS devise_net \
         FROM accounts a \
         LEFT JOIN ( \
             SELECT jel.account_id, jel.debit, jel.credit, jel.amount_currency \
             FROM journal_entry_lines jel \
             INNER JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE je.company_id = ? AND je.entry_date <= ? \
         ) AS x ON x.account_id = a.id \
         WHERE a.company_id = ? AND {filtre} \
         GROUP BY a.id, a.number, a.currency \
         ORDER BY a.number ASC, a.id ASC",
        d = somme_en_devise("x", Cote::Debit, None),
        c = somme_en_devise("x", Cote::Credit, None),
    );
    let mut q = sqlx::query_as::<_, SoldeALaDate>(&sql)
        .bind(company_id)
        .bind(as_of)
        .bind(company_id);
    if let Some(ids) = account_ids {
        for id in ids {
            q = q.bind(id);
        }
    }
    q.fetch_all(&mut *conn).await.map_err(map_db_error)
}
