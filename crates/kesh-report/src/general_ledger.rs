//! Grand livre — l'extrait d'un compte : ce qui a fait son solde, ligne à ligne.
//!
//! # Pourquoi ce rapport ne ressemble pas aux autres
//!
//! C'est le **premier rapport de Kesh qui franchit la borne d'exercice**, et c'est
//! délibéré : le bilan est cumulatif depuis l'origine, donc un grand livre borné
//! par exercice ne concorderait pas avec lui. Il n'utilise donc **pas**
//! [`crate::period::ReportPeriod`], qui exige un `fiscal_year_id` et refuse toute
//! borne hors exercice, mais [`LedgerPeriod`].
//!
//! # La règle du solde d'ouverture — le cœur du module
//!
//! Elle vit dans [`crate::opening`], **partagée avec la balance des comptes**
//! (Story 25-5-b) : une seule règle, que les deux rapports appliquent.
//!
//! Elle **diffère selon le type de compte**, et c'est ce qu'un lecteur pressé
//! prendra de travers :
//!
//! - `Asset` / `Liability` : cumul sur `entry_date < from`, **tous exercices
//!   confondus**, sans borne basse — même patron que
//!   [`crate::balance_sheet::generate`].
//! - `Revenue` / `Expense` : cumul depuis le **début de l'exercice contenant
//!   `from`** seulement. Si `from` est le premier jour d'un exercice, l'ouverture
//!   vaut 0.
//!
//! Un compte de bilan reporte son solde d'un exercice à l'autre ; un compte de
//! résultat est soldé au bouclement et repart de zéro. Cumuler un compte de
//! produits depuis l'origine donnerait un nombre qui ne correspond à **rien** —
//! ni au compte de résultat, ni à la balance, ni au bilan.
//!
//! ⚠️ **Kesh ne passe aucune écriture de clôture** : cette remise à zéro n'existe
//! que comme borne basse du `SUM`. Elle est **entièrement à la charge de ce
//! module**, et son échec serait **muet** — une ouverture mal bornée garderait
//! l'identité de la section (`closing = opening + mouvements` sur une période
//! d'un seul exercice ; la même, moins le solde des ruptures, quand la période
//! franchit une fin d'exercice — cf. ci-dessous), les totaux s'additionneraient,
//! le rapport serait intérieurement cohérent et extérieurement faux. C'est
//! pourquoi les tests de concordance appellent réellement
//! `generate_balance_sheet`, `generate_trial_balance` et
//! `generate_income_statement`.
//!
//! # Le solde de clôture et les fins d'exercice franchies
//!
//! Sur un compte de résultat, le solde progressif **repart de zéro** à chaque
//! fin d'exercice que la période franchit, et une rupture le dit
//! ([`FiscalYearBreak`]). Deux conséquences, que ce module tient ensemble :
//!
//! - l'exercice courant **avant la première ligne** est celui qui contient
//!   `from` : une première ligne d'un exercice postérieur pose une rupture qui
//!   clôt l'ouverture (#777) ;
//! - la **clôture** (`closing`) est le solde de l'exercice en cours à `to` —
//!   le dernier solde progressif, ruptures comprises, y compris une rupture
//!   franchie après la dernière ligne (#778). Elle ne vaut donc **pas**
//!   `opening + mouvements` quand la période franchit une fin d'exercice ; le
//!   cumul de la période reste dans `total_debit` et `total_credit`. Sur un
//!   compte de bilan, la chaîne est continue et rien ne change.
//!
//! # Deux pièges du schéma
//!
//! - `journal_entries.entry_number` est unique **par exercice** : il repart à 1.
//!   Le tri pose donc `fiscal_year_id` **avant** lui, sans quoi deux écritures de
//!   même date sur deux exercices s'entrelacent.
//! - `journal_entry_lines` **n'a aucun libellé** : il vient de l'écriture.
//!
//! # Limite connue
//!
//! Le signe est lu sur le `account_type` **courant** du compte. Retyper un compte
//! mouvementé re-signe donc silencieusement tout son historique — défaut
//! pré-existant, hors périmètre ici (issues #274 et #382).

use chrono::NaiveDate;
use kesh_db::entities::AccountType;
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;
use crate::opening::{is_bilan, is_debit_natured, opening_from, signed};

/// Plafond de lignes rendues en une fois, aligné sur le patron du dépôt.
///
/// ⚠️ Il borne **l'écran**, jamais l'export. Un grand livre tronqué n'est pas
/// un grand livre : l'Olico exige qu'il soit produisible **en entier**, et c'est
/// l'export qui porte cette obligation. D'où [`LedgerOptions::limit`] à `None`,
/// qui ne pose aucune borne.
pub const MAX_LEDGER_LIMIT: i64 = 500;

/// Fenêtre du grand livre — **sans exercice**, contrairement à `ReportPeriod`.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerPeriod {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl LedgerPeriod {
    /// Valide l'ordre des bornes. Aucune contrainte d'exercice : c'est le propre
    /// de ce rapport.
    pub fn new(from: NaiveDate, to: NaiveDate) -> Result<Self, ReportError> {
        if from > to {
            return Err(ReportError::PeriodInvalid {
                reason: format!("from ({from}) doit être ≤ to ({to})"),
            });
        }
        Ok(Self { from, to })
    }
}

/// Le grand livre : une section par compte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralLedger {
    pub period: LedgerPeriod,
    pub sections: Vec<LedgerSection>,
}

/// L'extrait d'un compte sur la période.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerSection {
    pub account_id: i64,
    pub account_number: String,
    pub account_name: String,
    pub account_type: AccountType,
    /// `false` pour un compte archivé — il reste **inclus** s'il porte un solde.
    pub active: bool,
    /// Côté naturel du compte, pour l'affichage.
    pub balance_side: &'static str,
    pub opening: Decimal,
    pub lines: Vec<LedgerLine>,
    pub total_debit: Decimal,
    pub total_credit: Decimal,
    pub closing: Decimal,
    /// Vrai quand le solde de clôture est du côté contraire à la nature du
    /// compte — un compte de produits à solde débiteur, par exemple. C'est
    /// exactement l'anomalie que ce rapport doit rendre visible.
    pub unnatural_balance: bool,
    /// Ruptures d'exercice traversées, pour les comptes de résultat.
    pub fiscal_year_breaks: Vec<FiscalYearBreak>,
    /// Nombre total de lignes sur la période, **avant pagination**.
    pub line_count: i64,
}

/// Fin d'exercice traversée par la période, sur un compte de résultat : le solde
/// progressif y repart de zéro.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearBreak {
    /// Date de la **première ligne du nouvel exercice**, avant laquelle la
    /// rupture se rend ; pour une rupture qu'aucune ligne de la période ne suit
    /// (« rupture finale »), **premier jour** de l'exercice en cours à `to`.
    pub date: NaiveDate,
    pub closing_fiscal_year_id: i64,
    pub closing_balance: Decimal,
}

impl LedgerSection {
    /// La rupture d'exercice à rendre **avant** la ligne d'indice `idx`, s'il y
    /// en a une — règle commune au PDF et au CSV (l'écran l'applique en
    /// TypeScript, `GeneralLedgerView.svelte`).
    ///
    /// Une rupture précède la première ligne de son nouvel exercice, dont elle
    /// porte la date. Entre deux lignes, elle se reconnaît au **changement
    /// d'exercice** : c'est la rupture qui clôt l'exercice de la ligne
    /// précédente — jamais à la date seule. En tête de section, aucune ligne
    /// précédente ne dit quel exercice se clôt : c'est alors la rupture datée du
    /// jour de la ligne, qui clôt un autre exercice que le sien (#777 : la
    /// rupture entre l'exercice de `from` et la première ligne ; #775 : celle
    /// qui tombe à la frontière de deux pages).
    pub fn break_before(&self, idx: usize) -> Option<&FiscalYearBreak> {
        let cur = self.lines.get(idx)?;
        match idx.checked_sub(1).map(|p| self.lines[p].fiscal_year_id) {
            Some(prev) if prev == cur.fiscal_year_id => None,
            Some(prev) => self
                .fiscal_year_breaks
                .iter()
                .find(|b| b.closing_fiscal_year_id == prev),
            None => self.fiscal_year_breaks.iter().find(|b| {
                b.date == cur.entry_date && b.closing_fiscal_year_id != cur.fiscal_year_id
            }),
        }
    }

    /// Les ruptures qu'aucune ligne rendue ne suit : une fin d'exercice franchie
    /// après la dernière ligne de la période (#778). Elles se rendent après les
    /// lignes, avant les totaux — toutes, quand la section n'a aucune ligne.
    ///
    /// Ce sont, par construction, celles que [`Self::break_before`] ne place
    /// devant aucune ligne : chaque rupture est rendue **une fois**, aucune ne se
    /// perd. Un exercice ne se clôt qu'une fois par section, d'où la
    /// comparaison sur `closing_fiscal_year_id`.
    pub fn trailing_breaks(&self) -> impl Iterator<Item = &FiscalYearBreak> {
        let placees: Vec<i64> = (0..self.lines.len())
            .filter_map(|idx| self.break_before(idx))
            .map(|b| b.closing_fiscal_year_id)
            .collect();
        self.fiscal_year_breaks
            .iter()
            .filter(move |b| !placees.contains(&b.closing_fiscal_year_id))
    }
}

/// Une ligne d'écriture vue depuis le compte.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerLine {
    pub line_id: i64,
    pub entry_id: i64,
    pub entry_date: NaiveDate,
    pub fiscal_year_id: i64,
    /// Nom de l'exercice (`fiscal_years.name`). ⚠️ Il n'est pas décoratif : sans
    /// lui, un numéro de pièce est **ambigu** dès que la période traverse deux
    /// exercices, puisqu'il y repart à 1. C'est la pièce « n° 12 » de quel
    /// exercice ? La question doit se lire, pas se déduire.
    pub fiscal_year_name: String,
    /// ⚠️ Unique **par exercice** seulement.
    pub entry_number: i64,
    pub journal: String,
    /// Libellé de l'**écriture** : les lignes n'en portent pas.
    pub description: String,
    /// Comptes de l'écriture autres que celui-ci — ce qui rend l'extrait lisible.
    pub counterpart: Vec<String>,
    pub debit: Decimal,
    pub credit: Decimal,
    pub running_balance: Decimal,
    /// Code du groupe de lettrage d'**aujourd'hui** (`AA`), `None` si la ligne
    /// est ouverte (Story 15-1b, AC6) — par `letterings::code_of`. Le JSON seul
    /// le porte : l'export CSV et le PDF écrivent leurs colonnes une à une, et
    /// restent tels quels (C-15-1b-6).
    pub lettering_code: Option<String>,
}

/// Ligne brute, telle que la lit SQL.
#[derive(Debug, sqlx::FromRow)]
struct RawLine {
    line_id: i64,
    entry_id: i64,
    entry_date: NaiveDate,
    fiscal_year_id: i64,
    fiscal_year_name: String,
    entry_number: i64,
    journal: String,
    description: String,
    debit: Decimal,
    credit: Decimal,
    lettering_key: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct RawAccount {
    account_id: i64,
    number: String,
    name: String,
    account_type: AccountType,
    active: bool,
}

/// Un exercice, réduit à ce dont le grand livre a besoin : son identifiant — la
/// clé sur laquelle [`Cumul`] pose ses ruptures — et son premier jour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Exercice {
    id: i64,
    start: NaiveDate,
}

/// L'exercice qui contient `date`, s'il en existe un.
///
/// C'est l'**exercice en cours au début de la période** : son premier jour borne
/// le solde d'ouverture d'un compte de résultat, et son identifiant est
/// l'exercice courant de [`Cumul`] avant la première ligne (#777). `None` si
/// aucun exercice ne couvre la date — l'ouverture d'un compte de résultat est
/// alors nulle.
async fn fiscal_year_containing(
    pool: &MySqlPool,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<Exercice>, ReportError> {
    let row: Option<(i64, NaiveDate)> = sqlx::query_as(
        "SELECT id, start_date FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         ORDER BY start_date DESC LIMIT 1",
    )
    .bind(company_id)
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(row.map(|(id, start)| Exercice { id, start }))
}

/// Le dernier exercice commencé au plus tard le `date` — l'**exercice en cours
/// à la fin de la période** (#778).
///
/// ⚠️ Pas « l'exercice qui contient `date` » : si `to` tombe après le dernier
/// exercice créé, ou entre deux exercices, c'est le dernier exercice commencé
/// qui reste en cours. Aucune écriture ne se date hors d'un exercice, si bien que
/// ce choix ne change rien aux lignes ; il évite seulement de prendre pour
/// « aucun exercice » une période qui déborde du dernier.
async fn fiscal_year_current_at(
    pool: &MySqlPool,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<Exercice>, ReportError> {
    let row: Option<(i64, NaiveDate)> = sqlx::query_as(
        "SELECT id, start_date FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? \
         ORDER BY start_date DESC LIMIT 1",
    )
    .bind(company_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(row.map(|(id, start)| Exercice { id, start }))
}

/// Solde d'ouverture d'un compte au `from` — **la règle centrale du module**,
/// appliquée par [`opening_from`] (module partagé avec la balance des comptes).
///
/// Une requête lit les deux paires de sommes brutes : avant `from`, et depuis le
/// début de l'exercice contenant `from` (`exercice_initial`). Quand aucun
/// exercice ne couvre `from`, la borne basse vaut `from` lui-même : la seconde
/// paire est nulle, et l'ouverture d'un compte de résultat aussi.
async fn opening_balance(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    account_type: AccountType,
    from: NaiveDate,
    exercice_initial: Option<Exercice>,
) -> Result<Decimal, ReportError> {
    let fy_start = exercice_initial.map_or(from, |e| e.start);

    let (bd, bc, sd, sc) = sqlx::query_as::<_, (Decimal, Decimal, Decimal, Decimal)>(
        "SELECT COALESCE(SUM(jel.debit), 0), COALESCE(SUM(jel.credit), 0), \
                COALESCE(SUM(CASE WHEN je.entry_date >= ? THEN jel.debit ELSE 0 END), 0), \
                COALESCE(SUM(CASE WHEN je.entry_date >= ? THEN jel.credit ELSE 0 END), 0) \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.account_id = ? AND je.company_id = ? AND je.entry_date < ?",
    )
    .bind(fy_start)
    .bind(fy_start)
    .bind(account_id)
    .bind(company_id)
    .bind(from)
    .fetch_one(pool)
    .await
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(opening_from(account_type, (bd, bc), (sd, sc)))
}

/// Comptes à rendre : ceux qui ont un mouvement sur la période **ou** un solde
/// d'ouverture non nul. Un compte explicitement demandé est toujours rendu.
async fn select_accounts(
    pool: &MySqlPool,
    company_id: i64,
    account_ids: Option<&[i64]>,
) -> Result<Vec<RawAccount>, ReportError> {
    let base = "SELECT a.id AS account_id, a.number, a.name, a.account_type, a.active \
                FROM accounts a WHERE a.company_id = ?";

    let rows = match account_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders = std::iter::repeat_n("?", ids.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!("{base} AND a.id IN ({placeholders}) ORDER BY a.number ASC");
            let mut q = sqlx::query_as::<_, RawAccount>(&sql).bind(company_id);
            for id in ids {
                q = q.bind(id);
            }
            q.fetch_all(pool).await
        }
        _ => {
            let sql = format!("{base} ORDER BY a.number ASC");
            sqlx::query_as::<_, RawAccount>(&sql)
                .bind(company_id)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(kesh_db::errors::map_db_error)?;

    Ok(rows)
}

/// Lignes d'un compte sur la période, dans l'ordre de lecture d'un réviseur.
///
/// ⚠️ `fiscal_year_id` est trié **avant** `entry_number` : ce dernier repart à 1
/// à chaque exercice. `jel.id` en dernier ressort rend l'ordre **totalement
/// déterministe** — deux appels rendent la même chose, et c'est testé.
async fn fetch_lines(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &LedgerPeriod,
    offset: i64,
    // `None` ⇒ pas de borne (export).
    limit: Option<i64>,
) -> Result<Vec<RawLine>, ReportError> {
    let sql = "
        SELECT jel.id AS line_id, je.id AS entry_id, je.entry_date, je.fiscal_year_id,
               fy.name AS fiscal_year_name,
               je.entry_number, je.journal, je.description, jel.debit, jel.credit,
               jel.lettering_key
        FROM journal_entry_lines jel
        INNER JOIN journal_entries je ON je.id = jel.entry_id
        INNER JOIN fiscal_years fy ON fy.id = je.fiscal_year_id
        WHERE jel.account_id = ? AND je.company_id = ?
          AND je.entry_date >= ? AND je.entry_date <= ?
        ORDER BY je.entry_date ASC, je.fiscal_year_id ASC, je.entry_number ASC,
                 jel.line_order ASC, jel.id ASC
        LIMIT ? OFFSET ?";

    sqlx::query_as::<_, RawLine>(sql)
        .bind(account_id)
        .bind(company_id)
        .bind(period.from)
        .bind(period.to)
        // MariaDB n'a pas de « LIMIT ALL » : la borne absente se dit i64::MAX.
        .bind(limit.unwrap_or(i64::MAX))
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| ReportError::Db(kesh_db::errors::map_db_error(e)))
}

/// Totaux d'un compte sur **toute la période**.
struct Totaux {
    debit: Decimal,
    credit: Decimal,
    /// Nombre de lignes de la période, avant pagination.
    count: i64,
    /// Débit et crédit des seules lignes de l'exercice en cours à la fin de la
    /// période — la clôture d'un compte de résultat quand la période a franchi
    /// une fin d'exercice (#778).
    debit_exercice_final: Decimal,
    credit_exercice_final: Decimal,
}

/// Totaux et compte de lignes sur **toute la période**.
///
/// ⚠️ Requête distincte de [`fetch_lines`] **à dessein** : les totaux ne doivent
/// jamais être calculés sur la page rendue. `limit=5` et `limit=1000` doivent
/// donner les mêmes totaux — c'est testé.
///
/// La somme de l'exercice final se lit sur `fiscal_year_id`, **la clé même** sur
/// laquelle [`Cumul`] remet le solde à zéro : la clôture d'un compte de résultat
/// et le dernier solde progressif ne peuvent donc pas diverger sur la borne.
async fn fetch_totals(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    period: &LedgerPeriod,
    exercice_final: Option<i64>,
) -> Result<Totaux, ReportError> {
    // `exercice_final` à NULL : la comparaison n'est jamais vraie, les deux
    // sommes valent zéro — et ne sont pas lues (cf. `closing`).
    let sql = "
        SELECT COALESCE(SUM(jel.debit), 0) AS d, COALESCE(SUM(jel.credit), 0) AS c,
               COUNT(*) AS n,
               COALESCE(SUM(CASE WHEN je.fiscal_year_id = ? THEN jel.debit ELSE 0 END), 0),
               COALESCE(SUM(CASE WHEN je.fiscal_year_id = ? THEN jel.credit ELSE 0 END), 0)
        FROM journal_entry_lines jel
        INNER JOIN journal_entries je ON je.id = jel.entry_id
        WHERE jel.account_id = ? AND je.company_id = ?
          AND je.entry_date >= ? AND je.entry_date <= ?";

    let (debit, credit, count, debit_exercice_final, credit_exercice_final) =
        sqlx::query_as::<_, (Decimal, Decimal, i64, Decimal, Decimal)>(sql)
            .bind(exercice_final)
            .bind(exercice_final)
            .bind(account_id)
            .bind(company_id)
            .bind(period.from)
            .bind(period.to)
            .fetch_one(pool)
            .await
            .map_err(|e| ReportError::Db(kesh_db::errors::map_db_error(e)))?;

    Ok(Totaux {
        debit,
        credit,
        count,
        debit_exercice_final,
        credit_exercice_final,
    })
}

/// Contreparties des écritures rendues, en **une seule requête** pour toutes.
///
/// ⚠️ Surtout pas une sous-requête corrélée par ligne.
async fn fetch_counterparts(
    pool: &MySqlPool,
    company_id: i64,
    account_id: i64,
    entry_ids: &[i64],
) -> Result<std::collections::HashMap<i64, Vec<String>>, ReportError> {
    use std::collections::HashMap;
    if entry_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = std::iter::repeat_n("?", entry_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT jel.entry_id, a.number \
         FROM journal_entry_lines jel \
         INNER JOIN journal_entries je ON je.id = jel.entry_id \
         INNER JOIN accounts a ON a.id = jel.account_id \
         WHERE je.company_id = ? AND jel.account_id <> ? \
           AND jel.entry_id IN ({placeholders}) \
         ORDER BY jel.entry_id ASC, jel.line_order ASC"
    );

    let mut q = sqlx::query_as::<_, (i64, String)>(&sql)
        .bind(company_id)
        .bind(account_id);
    for id in entry_ids {
        q = q.bind(id);
    }
    let rows = q
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    for (entry_id, number) in rows {
        let v = map.entry(entry_id).or_default();
        if !v.contains(&number) {
            v.push(number);
        }
    }
    Ok(map)
}

/// Le solde progressif d'un compte, ligne après ligne — **une seule règle**,
/// appliquée aux lignes sautées par `offset` comme aux lignes rendues.
///
/// ⚠️ C'est ce partage qui rend une page juste au-delà de la première (#775) :
/// le solde de départ d'une page est l'ouverture **plus** les lignes qui la
/// précèdent, prises dans le même ordre et avec les mêmes remises à zéro. Une
/// règle recopiée pour les lignes sautées pourrait diverger de celle des lignes
/// rendues sans que rien ne le signale ; une règle partagée ne le peut pas.
struct Cumul {
    account_type: AccountType,
    /// Compte de RÉSULTAT : le solde repart de zéro à chaque exercice franchi.
    /// Compte de BILAN : la chaîne est continue.
    resets: bool,
    running: Decimal,
    /// Exercice de la **dernière ligne lue**, sautée ou rendue ; avant la
    /// première, l'**exercice qui contient `from`** — celui dont l'ouverture
    /// porte le solde.
    ///
    /// ⚠️ #777 — il valait `None` avant la première ligne : quand celle-ci tombait
    /// dans un exercice postérieur à celui de `from`, aucune rupture n'était posée
    /// et l'ouverture d'un compte de résultat se reportait dans l'exercice
    /// suivant. `None` ne reste possible que si aucun exercice ne contient
    /// `from` ; l'ouverture d'un compte de résultat est alors nulle, et la
    /// première ligne n'a rien à remettre à zéro.
    current_fy: Option<i64>,
}

impl Cumul {
    fn new(account_type: AccountType, opening: Decimal, exercice_initial: Option<i64>) -> Self {
        Self {
            account_type,
            resets: !is_bilan(account_type),
            running: opening,
            current_fy: exercice_initial,
        }
    }

    /// Clôt le cumul à la fin de la période : si l'exercice en cours à `to` n'est
    /// pas celui de la dernière ligne lue, la période a franchi une fin
    /// d'exercice **après** sa dernière ligne. La rupture est alors rendue, datée
    /// du premier jour de cet exercice, et le solde repart de zéro — c'est ce qui
    /// fait du dernier solde progressif la clôture de la section (#778).
    fn termine(&mut self, exercice_final: Option<Exercice>) -> Option<FiscalYearBreak> {
        let fin = exercice_final?;
        let fy = self.current_fy?;
        if !self.resets || fy == fin.id {
            return None;
        }
        let rupture = FiscalYearBreak {
            date: fin.start,
            closing_fiscal_year_id: fy,
            closing_balance: self.running,
        };
        self.running = Decimal::ZERO;
        self.current_fy = Some(fin.id);
        Some(rupture)
    }

    /// Ajoute une ligne au solde. Rend la rupture d'exercice franchie **avant**
    /// elle, s'il y en a une — la remise à zéro est alors déjà faite.
    fn avance(&mut self, r: &RawLine) -> Option<FiscalYearBreak> {
        let mut rupture = None;
        if self.resets
            && let Some(fy) = self.current_fy
            && r.fiscal_year_id != fy
        {
            rupture = Some(FiscalYearBreak {
                date: r.entry_date,
                closing_fiscal_year_id: fy,
                closing_balance: self.running,
            });
            self.running = Decimal::ZERO;
        }
        self.current_fy = Some(r.fiscal_year_id);
        self.running += signed(self.account_type, r.debit, r.credit);
        rupture
    }
}

/// Options de rendu du grand livre.
#[derive(Debug, Clone, Default)]
pub struct LedgerOptions {
    /// Comptes demandés. `None` ⇒ ceux qui ont un mouvement ou un solde.
    pub account_ids: Option<Vec<i64>>,
    /// Rendre aussi les comptes sans mouvement **et** sans solde.
    pub include_zero: bool,
    /// Lignes sautées **par compte**. Le solde progressif de la page en tient
    /// compte : il part de l'ouverture plus ces lignes (#775).
    pub offset: i64,
    /// Écrêté à [`MAX_LEDGER_LIMIT`]. **`None` ⇒ aucune borne** — c'est ce que
    /// prend l'export, qui doit rendre le livre entier. L'appelant qui veut le
    /// défaut d'écran passe `Some(MAX_LEDGER_LIMIT)` explicitement.
    pub limit: Option<i64>,
}

/// Génère le grand livre.
///
/// Une section par compte, triées par numéro. Chaque section porte son solde
/// d'ouverture, ses lignes avec solde progressif, ses totaux et son solde de
/// clôture — les trois lignes d'encadrement sans lesquelles ce n'est pas un
/// extrait de compte.
pub async fn generate(
    pool: &MySqlPool,
    company_id: i64,
    period: &LedgerPeriod,
    options: &LedgerOptions,
) -> Result<GeneralLedger, ReportError> {
    let explicit = options
        .account_ids
        .as_ref()
        .is_some_and(|ids| !ids.is_empty());
    let accounts = select_accounts(pool, company_id, options.account_ids.as_deref()).await?;

    let limit = options.limit.map(|l| l.clamp(1, MAX_LEDGER_LIMIT));
    let offset = options.offset.max(0);

    // Les deux exercices qui encadrent la période — communs à tous les comptes.
    let exercice_initial = fiscal_year_containing(pool, company_id, period.from).await?;
    let exercice_final = fiscal_year_current_at(pool, company_id, period.to).await?;

    let mut sections = Vec::new();

    for acc in accounts {
        let opening = opening_balance(
            pool,
            company_id,
            acc.account_id,
            acc.account_type,
            period.from,
            exercice_initial,
        )
        .await?;
        let totaux = fetch_totals(
            pool,
            company_id,
            acc.account_id,
            period,
            exercice_final.map(|e| e.id),
        )
        .await?;
        let (total_debit, total_credit, line_count) = (totaux.debit, totaux.credit, totaux.count);

        // Un compte sans mouvement NI solde n'a rien à montrer — sauf s'il a été
        // explicitement demandé, ou si l'appelant veut tout voir.
        if !explicit && !options.include_zero && line_count == 0 && opening.is_zero() {
            continue;
        }

        let raw = fetch_lines(pool, company_id, acc.account_id, period, offset, limit).await?;
        let entry_ids: Vec<i64> = raw.iter().map(|r| r.entry_id).collect();
        let counterparts = fetch_counterparts(pool, company_id, acc.account_id, &entry_ids).await?;

        // Solde progressif + ruptures d'exercice.
        //
        // Sur un compte de RÉSULTAT, franchir une fin d'exercice remet le solde à
        // zéro : sinon le progressif affiché sur deux ans ne veut rien dire. Sur
        // un compte de BILAN, la chaîne est continue.
        let mut cumul = Cumul::new(acc.account_type, opening, exercice_initial.map(|e| e.id));

        // ⚠️ #775 — au-delà de la première page, le solde de départ n'est PAS
        // l'ouverture : c'est l'ouverture plus les lignes sautées par `offset`.
        // Elles sont lues par `fetch_lines` lui-même — même requête, même
        // `ORDER BY` — puis passées dans le même `Cumul` : les remises à zéro des
        // exercices franchis s'y appliquent, et l'exercice courant s'initialise
        // sur la dernière ligne sautée, si bien qu'une rupture qui tombe pile à
        // la frontière de deux pages est posée sur la page suivante. Les
        // ruptures franchies DANS les lignes sautées appartiennent aux pages
        // précédentes : elles ne sont pas rendues ici.
        let sautees = offset.min(line_count);
        if sautees > 0 {
            for r in fetch_lines(pool, company_id, acc.account_id, period, 0, Some(sautees)).await?
            {
                cumul.avance(&r);
            }
        }

        let mut breaks = Vec::new();
        let mut lines = Vec::with_capacity(raw.len());

        for r in raw {
            breaks.extend(cumul.avance(&r));
            let running = cumul.running;

            lines.push(LedgerLine {
                line_id: r.line_id,
                entry_id: r.entry_id,
                entry_date: r.entry_date,
                fiscal_year_id: r.fiscal_year_id,
                fiscal_year_name: r.fiscal_year_name,
                entry_number: r.entry_number,
                journal: r.journal,
                description: r.description,
                counterpart: counterparts.get(&r.entry_id).cloned().unwrap_or_default(),
                debit: r.debit,
                credit: r.credit,
                running_balance: running,
                lettering_code: r
                    .lettering_key
                    .map(kesh_db::repositories::letterings::code_of),
            });
        }

        // Une fin d'exercice franchie APRÈS la dernière ligne de la période se rend
        // sur la page qui porte cette dernière ligne — ou sur la première, quand
        // la période n'a aucune ligne.
        let page_finale = if line_count == 0 {
            offset == 0
        } else {
            !lines.is_empty() && offset + lines.len() as i64 == line_count
        };
        if page_finale {
            breaks.extend(cumul.termine(exercice_final));
        }

        // ⚠️ La clôture se calcule sur les totaux de la PÉRIODE, jamais sur la
        // page rendue — c'est ce qui rend `limit` sans effet sur les chiffres.
        //
        // #778 — compte de RÉSULTAT dont la période franchit une fin d'exercice :
        // la clôture est le solde de l'exercice en cours à `to`, c'est-à-dire ses
        // seules lignes, puisque le solde y est reparti de zéro. Elle égale le
        // dernier solde progressif, ruptures comprises. Le cumul de la période
        // reste lisible dans `total_debit` et `total_credit`. Compte de BILAN, ou
        // période d'un seul exercice : ouverture plus mouvements.
        let franchit_un_exercice = match (exercice_initial, exercice_final) {
            (Some(debut), Some(fin)) => debut.id != fin.id,
            (None, Some(_)) => true,
            (_, None) => false,
        };
        let closing = if !is_bilan(acc.account_type) && franchit_un_exercice {
            signed(
                acc.account_type,
                totaux.debit_exercice_final,
                totaux.credit_exercice_final,
            )
        } else {
            opening + signed(acc.account_type, total_debit, total_credit)
        };

        sections.push(LedgerSection {
            account_id: acc.account_id,
            account_number: acc.number,
            account_name: acc.name,
            account_type: acc.account_type,
            active: acc.active,
            balance_side: if is_debit_natured(acc.account_type) {
                "debit"
            } else {
                "credit"
            },
            opening,
            lines,
            total_debit,
            total_credit,
            closing,
            unnatural_balance: closing.is_sign_negative() && !closing.is_zero(),
            fiscal_year_breaks: breaks,
            line_count,
        });
    }

    Ok(GeneralLedger {
        period: *period,
        sections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periode_inversee_est_refusee() {
        let from = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert!(LedgerPeriod::new(from, to).is_err());
        assert!(LedgerPeriod::new(to, from).is_ok());
        // Une période d'un seul jour est valide.
        assert!(LedgerPeriod::new(from, from).is_ok());
    }

    fn ligne(id: i64, fy: i64, date: NaiveDate) -> LedgerLine {
        LedgerLine {
            line_id: id,
            entry_id: id,
            entry_date: date,
            fiscal_year_id: fy,
            fiscal_year_name: format!("FY {fy}"),
            entry_number: id,
            journal: "OD".into(),
            description: "synthétique".into(),
            counterpart: vec![],
            debit: Decimal::ZERO,
            credit: Decimal::ONE,
            running_balance: Decimal::ONE,
            lettering_code: None,
        }
    }

    fn rupture(date: NaiveDate, clos: i64) -> FiscalYearBreak {
        FiscalYearBreak {
            date,
            closing_fiscal_year_id: clos,
            closing_balance: Decimal::TEN,
        }
    }

    fn section(lines: Vec<LedgerLine>, breaks: Vec<FiscalYearBreak>) -> LedgerSection {
        LedgerSection {
            account_id: 1,
            account_number: "3000".into(),
            account_name: "Ventes".into(),
            account_type: AccountType::Revenue,
            active: true,
            balance_side: "credit",
            opening: Decimal::ZERO,
            lines,
            total_debit: Decimal::ZERO,
            total_credit: Decimal::ZERO,
            closing: Decimal::ZERO,
            unnatural_balance: false,
            fiscal_year_breaks: breaks,
            line_count: 0,
        }
    }

    fn d(y: i32, m: u32, j: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, j).unwrap()
    }

    /// #777 — la rupture qui clôt l'exercice de `from` se place devant la
    /// première ligne, sans ligne précédente pour le dire.
    #[test]
    fn une_rupture_en_tete_de_section_se_place_devant_la_premiere_ligne() {
        let s = section(
            vec![ligne(1, 26, d(2026, 2, 1)), ligne(2, 26, d(2026, 3, 1))],
            vec![rupture(d(2026, 2, 1), 25)],
        );
        assert_eq!(
            s.break_before(0).map(|b| b.closing_fiscal_year_id),
            Some(25)
        );
        assert!(s.break_before(1).is_none());
        assert_eq!(s.trailing_breaks().count(), 0, "rendue une fois, en tête");
    }

    /// Entre deux lignes, la rupture se lit au changement d'exercice.
    #[test]
    fn une_rupture_entre_deux_lignes_se_lit_au_changement_d_exercice() {
        let s = section(
            vec![ligne(1, 25, d(2025, 12, 31)), ligne(2, 26, d(2026, 1, 1))],
            vec![rupture(d(2026, 1, 1), 25)],
        );
        assert!(s.break_before(0).is_none(), "la ligne 1 ouvre la section");
        assert_eq!(
            s.break_before(1).map(|b| b.closing_fiscal_year_id),
            Some(25)
        );
        assert_eq!(s.trailing_breaks().count(), 0);
    }

    /// #778 — une fin d'exercice franchie après la dernière ligne se rend après
    /// elle ; sans aucune ligne, elle se rend quand même.
    #[test]
    fn une_rupture_sans_ligne_apres_elle_se_rend_a_la_fin() {
        let s = section(
            vec![ligne(1, 25, d(2025, 3, 1))],
            vec![rupture(d(2026, 1, 1), 25)],
        );
        assert!(s.break_before(0).is_none());
        let fin: Vec<_> = s
            .trailing_breaks()
            .map(|b| b.closing_fiscal_year_id)
            .collect();
        assert_eq!(fin, vec![25]);

        let vide = section(vec![], vec![rupture(d(2026, 1, 1), 25)]);
        assert_eq!(vide.trailing_breaks().count(), 1);
    }

    #[test]
    fn periode_traverse_les_exercices_sans_broncher() {
        // Le propre de ce rapport : aucune contrainte d'exercice, contrairement
        // à `ReportPeriod`.
        let from = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let p = LedgerPeriod::new(from, to).expect("deux ans doivent passer");
        assert_eq!(p.from, from);
        assert_eq!(p.to, to);
    }
}
