//! Dépôt des cours de change (Story 37-2, Epic 37 multi-devises).
//!
//! Deux tables, deux rôles (U1, CR-5) :
//! - `_kesh_fetched_exchange_rates` (**système**) : la **dernière** valeur
//!   publiée par la source, par (source, devise, genre, date). Elle ne sert
//!   qu'à la copie et à l'**information** (valeur publiée affichée, écart
//!   signalé) — jamais à un contrôle d'écriture ;
//! - `exchange_rates` (**applicative**, sauvegardée) : le cours **retenu par la
//!   société**, avec sa source. C'est le **cours effectif**, lu par une seule
//!   fonction, [`find_company_rate_in_tx`].
//!
//! Écritures de `exchange_rates`, et rien d'autre : la copie des cours
//! récupérés ([`copy_fetched_in_tx`], insertion seule, sans audit), la saisie
//! ([`set_manual_rate`]), la reprise de la valeur publiée ([`reset_rate`]),
//! l'acquittement ([`acknowledge_corrections`], colonnes `acknowledged_*`
//! seulement), la restauration et la remise à zéro. Une récupération
//! ([`write_fetched_unit_in_tx`]) n'en modifie **jamais** une ligne.
//!
//! Toutes les fonctions prennent une connexion ou une transaction
//! (`&mut MySqlConnection`) fournie par l'appelant, qui y joint l'audit et la
//! rejoue sur interblocage (`retry::retry_on_deadlock`). Les codes de devise
//! sont attendus **validés** (trois majuscules, `kesh_core::currency`) : la
//! collation `ascii_general_ci` des colonnes ignorerait la casse.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use chrono::{NaiveDate, NaiveDateTime};
use kesh_core::exchange_rates::{ExerciceDevises, PublishedRate, RateKind, RateUnit};
use rust_decimal::Decimal;
use sqlx::MySqlConnection;

use crate::errors::{DbError, map_db_error};

/// La source des cours officiels (AFC / OFDF), seule écrite aujourd'hui.
pub const SOURCE_AFC: &str = "afc";

/// Valeur de `exchange_rates.source` d'un cours saisi par la société.
const SOURCE_MANUAL: &str = "manual";

/// Nom de la contrainte d'unicité de `exchange_rates`.
const UNIQUE_KEY: &str = "uq_exchange_rates_key";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Source d'un cours retenu par la société (`exchange_rates.source`).
///
/// La lecture est **tolérante** (A30) : une valeur inconnue (p. ex. `ecb`,
/// écrite par une version future ou par SQL) se lit [`RateOrigin::Other`],
/// jamais une erreur interne. Seules `afc` et `manual` s'écrivent aujourd'hui,
/// et une ligne `Other` se traite partout comme `manual` (hors l'affichage de
/// sa source) : elle n'est pas un cours `afc`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RateOrigin {
    /// Copie d'une valeur publiée, ou valeur publiée reprise par ↑.
    Afc,
    /// Cours saisi par la société.
    Manual,
    /// Toute autre valeur, gardée telle quelle.
    Other(String),
}

impl RateOrigin {
    /// La valeur telle qu'en base.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Afc => SOURCE_AFC,
            Self::Manual => SOURCE_MANUAL,
            Self::Other(s) => s,
        }
    }

    /// Vrai pour un cours `afc` — le seul qui s'annonce, s'acquitte, et que
    /// le contrôle `FX_RATE_NOT_AFC` de 37-3a reconnaît.
    pub fn is_afc(&self) -> bool {
        matches!(self, Self::Afc)
    }
}

/// Lecture tolérante de la colonne (`#[sqlx(try_from = "String")]`).
impl From<String> for RateOrigin {
    fn from(s: String) -> Self {
        match s.as_str() {
            SOURCE_AFC => Self::Afc,
            SOURCE_MANUAL => Self::Manual,
            _ => Self::Other(s),
        }
    }
}

/// Le cours **retenu par la société** pour une devise, un genre et une date
/// (ligne `exchange_rates`).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CompanyRate {
    /// Identifiant (`exchange_rates.id`, identifiant d'audit).
    pub id: i64,
    /// Société.
    pub company_id: i64,
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    #[sqlx(try_from = "String")]
    pub kind: RateKind,
    /// 1er du mois (`month`) ou jour (`day`).
    pub rate_date: NaiveDate,
    /// Le cours retenu, par unité de devise (10 décimales en base).
    pub rate: Decimal,
    /// Sa source.
    #[sqlx(try_from = "String")]
    pub source: RateOrigin,
    /// Verrou optimiste.
    pub version: i32,
    /// Valeur publiée (par unité) dont la correction a été acquittée.
    pub acknowledged_rate: Option<Decimal>,
    /// Moment de l'acquittement (UTC).
    pub acknowledged_at: Option<NaiveDateTime>,
    /// Auteur de la dernière écriture ; `None` = ligne écrite par Kesh (copie).
    pub updated_by: Option<i64>,
    /// Création.
    pub created_at: NaiveDateTime,
    /// Dernière modification.
    pub updated_at: NaiveDateTime,
}

/// Projection d'une ligne `exchange_rates` (alias `x`), partagée par
/// [`find_company_rate_in_tx`], la lecture groupée [`list_cells_in_tx`] et les
/// écritures (DRY).
const COMPANY_RATE_COLUMNS: &str = "x.id, x.company_id, x.currency, x.kind, x.rate_date, x.rate, \
     x.source, x.version, x.acknowledged_rate, x.acknowledged_at, x.updated_by, x.created_at, \
     x.updated_at";

/// La dernière valeur **publiée** par la source (ligne
/// `_kesh_fetched_exchange_rates`) — pour l'information et la copie, jamais
/// pour un contrôle.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct FetchedRate {
    /// Source (`afc`).
    pub source: String,
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    #[sqlx(try_from = "String")]
    pub kind: RateKind,
    /// 1er du mois (`month`) ou date demandée (`day`).
    pub rate_date: NaiveDate,
    /// Cours par unité (10 décimales).
    pub rate_per_unit: Decimal,
    /// Unité publiée.
    pub published_unit: i32,
    /// Cours publié pour l'unité.
    pub published_rate: Decimal,
    /// Date de publication lue (`day`), `None` pour `month`.
    pub source_date: Option<NaiveDate>,
    /// Première obtention (UTC), jamais avancée.
    pub first_fetched_at: NaiveDateTime,
    /// Dernière récupération qui a confirmé la ligne (UTC).
    pub fetched_at: NaiveDateTime,
    /// Récupération qui a apporté la valeur actuelle (UTC).
    pub value_changed_at: NaiveDateTime,
}

/// Projection d'une ligne `_kesh_fetched_exchange_rates` (alias `f`).
const FETCHED_COLUMNS: &str = "f.source, f.currency, f.kind, f.rate_date, f.rate_per_unit, \
     f.published_unit, f.published_rate, f.source_date, f.first_fetched_at, f.fetched_at, \
     f.value_changed_at";

/// Une correction de la source trouvée par une récupération (règle (c) de
/// l'AC-3), pour le `tracing::warn!` de l'appelant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchCorrection {
    /// Code ISO 4217.
    pub currency: String,
    /// Ancienne unité publiée.
    pub old_unit: i32,
    /// Ancien cours publié.
    pub old_rate: Decimal,
    /// Nouvelle unité publiée.
    pub new_unit: i32,
    /// Nouveau cours publié.
    pub new_rate: Decimal,
}

/// Ce qu'a écrit une récupération d'une unité ([`write_fetched_unit_in_tx`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FetchWriteOutcome {
    /// (a) Devises insérées.
    pub inserted: usize,
    /// (b) Devises confirmées à la même valeur (`fetched_at` avancé).
    pub confirmed: usize,
    /// (c) Devises corrigées par la source (valeur remplacée).
    pub corrected: Vec<FetchCorrection>,
}

impl FetchWriteOutcome {
    /// Nombre de devises écrites : insérées, confirmées et corrigées.
    pub fn written(&self) -> usize {
        self.inserted + self.confirmed + self.corrected.len()
    }
}

/// Une case du tableau des cours : le cours retenu par la société (s'il y en
/// a un) et la dernière valeur publiée (s'il y en a une), pour une devise, un
/// genre et une date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateCell {
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// Le cours retenu par la société.
    pub company: Option<CompanyRate>,
    /// La dernière valeur publiée (source `afc`).
    pub published: Option<FetchedRate>,
}

impl RateCell {
    /// Vrai si la ligne est `afc` et que la valeur publiée (par unité) en
    /// diffère : correction de la source non reprise.
    pub fn corrected(&self) -> bool {
        match (&self.company, &self.published) {
            (Some(c), Some(p)) => c.source.is_afc() && p.rate_per_unit != c.rate,
            _ => false,
        }
    }

    /// Vrai si [`RateCell::corrected`] et que la société a acquitté cette
    /// valeur publiée (une correction suivante le rouvre).
    pub fn acknowledged(&self) -> bool {
        self.corrected()
            && match (&self.company, &self.published) {
                (Some(c), Some(p)) => c.acknowledged_rate == Some(p.rate_per_unit),
                _ => false,
            }
    }
}

/// Une ligne `exchange_rates` avant et après une écriture, pour l'audit de
/// l'appelant. `before` à `None` : création ; `after` à `None` : suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RateChange {
    /// Avant.
    pub before: Option<CompanyRate>,
    /// Après.
    pub after: Option<CompanyRate>,
    /// La valeur publiée reprise (par unité) — [`reset_rate`] seulement.
    pub published_rate: Option<Decimal>,
}

/// Une correction de la source à annoncer à la société
/// ([`list_notices`]).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Notice {
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    #[sqlx(try_from = "String")]
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// Le cours retenu par la société (`afc`).
    pub effective_rate: Decimal,
    /// La valeur publiée, **par unité**.
    pub published_rate: Decimal,
    /// L'unité publiée.
    pub published_unit: i32,
    /// La récupération qui a apporté la valeur publiée (UTC).
    pub value_changed_at: NaiveDateTime,
}

/// Une correction que la société acquitte, à la valeur qu'elle a vue
/// ([`acknowledge_corrections`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeAck {
    /// Code ISO 4217.
    pub currency: String,
    /// Genre.
    pub kind: RateKind,
    /// 1er du mois ou jour.
    pub rate_date: NaiveDate,
    /// La valeur publiée (par unité) que l'écran a montrée.
    pub published_rate: Decimal,
}

/// Qui a lancé une récupération (`_kesh_exchange_rate_fetch.triggered_by`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchTrigger {
    /// Tâche automatique ou déclenchement à la connexion.
    Auto,
    /// `POST /exchange-rates/fetch`.
    Manual,
}

impl FetchTrigger {
    /// Valeur en base et au HTTP.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Statut d'une récupération. L'ordre dérivé est la gravité : le **pire**
/// statut d'un tour est le maximum (`error` > `not_published` > `ok`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FetchOutcomeStatus {
    /// Réussie (des devises ont pu être écartées : `invalid_rates:<n>`).
    Ok,
    /// Pas encore publiée.
    NotPublished,
    /// Échec (code classé).
    Error,
}

impl FetchOutcomeStatus {
    /// Valeur en base.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NotPublished => "not_published",
            Self::Error => "error",
        }
    }
}

/// L'état de la dernière récupération (`_kesh_exchange_rate_fetch`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchStatus {
    /// Qui l'a lancée.
    pub trigger: FetchTrigger,
    /// Fin (UTC).
    pub last_run_at: NaiveDateTime,
    /// Statut (le pire d'un tour).
    pub status: FetchOutcomeStatus,
    /// Code classé, jamais un texte libre (≤ 64 caractères).
    pub error: Option<String>,
    /// Genre de l'unité en cause.
    pub kind: RateKind,
    /// Date de l'unité en cause.
    pub rate_date: NaiveDate,
}

/// Un exercice multi-devises ouvert, pour le calcul des unités utiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenMultiCurrencyFiscalYear {
    /// Identifiant de l'exercice.
    pub id: i64,
    /// Société.
    pub company_id: i64,
    /// Dates et genres.
    pub exercice: ExerciceDevises,
}

// ---------------------------------------------------------------------------
// Lectures
// ---------------------------------------------------------------------------

/// Le **cours effectif** : le cours retenu par la société pour une devise, un
/// genre et une date — **seule** lecture de cours des contrôles et des
/// propositions (`lookup`, les `GET`, 37-3a, 37-5). `None` : aucune ligne
/// (`RATE_UNAVAILABLE` chez l'appelant), même si une valeur publiée existe.
pub async fn find_company_rate_in_tx(
    conn: &mut MySqlConnection,
    company_id: i64,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
) -> Result<Option<CompanyRate>, DbError> {
    select_company_rate(conn, company_id, currency, kind, rate_date, false).await
}

/// [`find_company_rate_in_tx`], ou sa variante `FOR UPDATE` des écritures.
async fn select_company_rate(
    conn: &mut MySqlConnection,
    company_id: i64,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
    for_update: bool,
) -> Result<Option<CompanyRate>, DbError> {
    let sql = format!(
        "SELECT {COMPANY_RATE_COLUMNS} FROM exchange_rates x \
         WHERE x.company_id = ? AND x.currency = ? AND x.kind = ? AND x.rate_date = ?{}",
        if for_update { " FOR UPDATE" } else { "" }
    );
    sqlx::query_as::<_, CompanyRate>(&sql)
        .bind(company_id)
        .bind(currency)
        .bind(kind.as_str())
        .bind(rate_date)
        .fetch_optional(conn)
        .await
        .map_err(map_db_error)
}

/// La dernière valeur publiée par `source` pour une devise, un genre et une
/// date — pour l'affichage et la copie seulement, **jamais** pour un contrôle
/// d'écriture. Sans verrou.
pub async fn find_fetched_rate_in_tx(
    conn: &mut MySqlConnection,
    source: &str,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
) -> Result<Option<FetchedRate>, DbError> {
    let sql = format!(
        "SELECT {FETCHED_COLUMNS} FROM _kesh_fetched_exchange_rates f \
         WHERE f.source = ? AND f.currency = ? AND f.kind = ? AND f.rate_date = ?"
    );
    sqlx::query_as::<_, FetchedRate>(&sql)
        .bind(source)
        .bind(currency)
        .bind(kind.as_str())
        .bind(rate_date)
        .fetch_optional(conn)
        .await
        .map_err(map_db_error)
}

/// Lecture **groupée** des cases d'un genre sur une plage de dates
/// (`from` ≤ `rate_date` ≤ `to`, bornes comprises) : les lignes de la société
/// et les valeurs publiées (`afc`), en deux lectures sur la même connexion —
/// jamais une requête par case. Rend une case par (date, devise) présente dans
/// l'une ou l'autre table, triées par date puis devise. Sert le tableau annuel
/// (un appel par genre) et `GET /exchange-rates` (`from == to`).
pub async fn list_cells_in_tx(
    conn: &mut MySqlConnection,
    company_id: i64,
    kind: RateKind,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<RateCell>, DbError> {
    let sql = format!(
        "SELECT {COMPANY_RATE_COLUMNS} FROM exchange_rates x \
         WHERE x.company_id = ? AND x.kind = ? AND x.rate_date BETWEEN ? AND ?"
    );
    let rows = sqlx::query_as::<_, CompanyRate>(&sql)
        .bind(company_id)
        .bind(kind.as_str())
        .bind(from)
        .bind(to)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;
    let sql = format!(
        "SELECT {FETCHED_COLUMNS} FROM _kesh_fetched_exchange_rates f \
         WHERE f.source = ? AND f.kind = ? AND f.rate_date BETWEEN ? AND ?"
    );
    let published = sqlx::query_as::<_, FetchedRate>(&sql)
        .bind(SOURCE_AFC)
        .bind(kind.as_str())
        .bind(from)
        .bind(to)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;

    let mut cells: BTreeMap<(NaiveDate, String), RateCell> = BTreeMap::new();
    fn case<'a>(
        cells: &'a mut BTreeMap<(NaiveDate, String), RateCell>,
        kind: RateKind,
        currency: &str,
        rate_date: NaiveDate,
    ) -> &'a mut RateCell {
        cells
            .entry((rate_date, currency.to_string()))
            .or_insert_with(|| RateCell {
                currency: currency.to_string(),
                kind,
                rate_date,
                company: None,
                published: None,
            })
    }
    for r in rows {
        let (c, d) = (r.currency.clone(), r.rate_date);
        case(&mut cells, kind, &c, d).company = Some(r);
    }
    for p in published {
        let (c, d) = (p.currency.clone(), p.rate_date);
        case(&mut cells, kind, &c, d).published = Some(p);
    }
    Ok(cells.into_values().collect())
}

/// Les devises des comptes de la société, comptes archivés compris, triées et
/// sans doublon (`hasAccount` du tableau, filtre de la copie).
pub async fn list_account_currencies(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<Vec<String>, DbError> {
    sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT currency FROM accounts \
         WHERE company_id = ? AND currency IS NOT NULL ORDER BY currency",
    )
    .bind(company_id)
    .fetch_all(conn)
    .await
    .map_err(map_db_error)
}

/// Toutes les lignes `exchange_rates` de la société (cours retenus), triées
/// par date, genre puis devise — non bornée, pour l'export de souveraineté
/// (`exports::global`), comme les autres `list_all_by_company`.
pub async fn list_all_by_company(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<Vec<CompanyRate>, DbError> {
    let sql = format!(
        "SELECT {COMPANY_RATE_COLUMNS} FROM exchange_rates x WHERE x.company_id = ? \
         ORDER BY x.rate_date, x.kind, x.currency"
    );
    sqlx::query_as::<_, CompanyRate>(&sql)
        .bind(company_id)
        .fetch_all(conn)
        .await
        .map_err(map_db_error)
}

/// Les devises des comptes de la société, triées, chacune avec un drapeau :
/// vrai si au moins un compte **actif** la porte (faux : seuls des comptes
/// archivés). Sert les lignes du tableau annuel (un compte archivé n'y amène
/// sa devise que s'il a une ligne dans l'exercice).
pub async fn list_account_currency_states(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<Vec<(String, bool)>, DbError> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT currency, CAST(MAX(active) AS SIGNED) FROM accounts \
         WHERE company_id = ? AND currency IS NOT NULL GROUP BY currency ORDER BY currency",
    )
    .bind(company_id)
    .fetch_all(conn)
    .await
    .map_err(map_db_error)?;
    Ok(rows.into_iter().map(|(c, a)| (c, a != 0)).collect())
}

/// La copie (AC-3 bis) quand un **compte reçoit une devise**
/// (`accounts::create`, `accounts::update`) : pour chaque exercice
/// **multi-devises ouvert** de la société, les cours déjà récupérés de cette
/// seule devise, sur les unités utiles de l'exercice. Exercices lus sans
/// verrou dans la transaction de l'appelant, qui tient déjà `companies`.
/// Rend le nombre de lignes copiées (0 sans exercice multi-devises ouvert).
pub async fn copy_currency_to_open_years_in_tx(
    conn: &mut MySqlConnection,
    company_id: i64,
    currency: &str,
) -> Result<u64, DbError> {
    let exercices = list_open_multi_currency_fiscal_years(conn, Some(company_id)).await?;
    let units: BTreeSet<RateUnit> = exercices
        .iter()
        .flat_map(|fy| kesh_core::exchange_rates::unites_utiles_exercice(&fy.exercice))
        .collect();
    let units: Vec<RateUnit> = units.into_iter().collect();
    copy_fetched_in_tx(conn, company_id, &units, Some(currency)).await
}

/// Les exercices **multi-devises ouverts** (`multi_currency = TRUE`,
/// `status = 'Open'`), de toutes les sociétés (`company_id` à `None`) ou d'une
/// seule, triés par société puis début. Sans verrou.
pub async fn list_open_multi_currency_fiscal_years(
    conn: &mut MySqlConnection,
    company_id: Option<i64>,
) -> Result<Vec<OpenMultiCurrencyFiscalYear>, DbError> {
    let rows: Vec<(i64, i64, NaiveDate, NaiveDate, String, String)> = sqlx::query_as(
        "SELECT id, company_id, start_date, end_date, opening_rate_kind, closing_rate_kind \
         FROM fiscal_years \
         WHERE multi_currency = TRUE AND status = 'Open' AND (? IS NULL OR company_id = ?) \
         ORDER BY company_id, start_date",
    )
    .bind(company_id)
    .bind(company_id)
    .fetch_all(conn)
    .await
    .map_err(map_db_error)?;
    rows.into_iter()
        .map(|(id, company_id, start, end, o, c)| {
            let genre =
                |s: String| RateKind::try_from(s).map_err(|e| DbError::Invariant(e.to_string()));
            Ok(OpenMultiCurrencyFiscalYear {
                id,
                company_id,
                exercice: ExerciceDevises {
                    start,
                    end,
                    opening_kind: genre(o)?,
                    closing_kind: genre(c)?,
                },
            })
        })
        .collect()
}

/// Les sociétés dont un exercice multi-devises **ouvert** a `unit` parmi ses
/// unités utiles (`kesh_core::exchange_rates::unites_utiles_exercice`), par
/// identifiant croissant, sans doublon — les sociétés à qui une récupération
/// copie l'unité (AC-3 bis). Lecture des exercices sans verrou.
pub async fn companies_with_unit_in_tx(
    conn: &mut MySqlConnection,
    unit: RateUnit,
) -> Result<Vec<i64>, DbError> {
    let exercices = list_open_multi_currency_fiscal_years(conn, None).await?;
    let ids: BTreeSet<i64> = exercices
        .iter()
        .filter(|fy| {
            kesh_core::exchange_rates::unites_utiles_exercice(&fy.exercice).contains(&unit)
        })
        .map(|fy| fy.company_id)
        .collect();
    Ok(ids.into_iter().collect())
}

/// Les unités de `units` **manquantes** : aucune valeur publiée `afc` de ce
/// genre et de cette date (jamais d'après `exchange_rates` : une installation
/// restaurée a ses cours retenus, pas les valeurs publiées). Ordre de `units`
/// conservé.
pub async fn missing_units(
    conn: &mut MySqlConnection,
    units: &[RateUnit],
) -> Result<Vec<RateUnit>, DbError> {
    if units.is_empty() {
        return Ok(Vec::new());
    }
    let sql = format!(
        "SELECT DISTINCT kind, rate_date FROM _kesh_fetched_exchange_rates \
         WHERE source = ? AND (kind, rate_date) IN ({})",
        vec!["(?, ?)"; units.len()].join(", ")
    );
    let mut q = sqlx::query_as::<_, (String, NaiveDate)>(&sql).bind(SOURCE_AFC);
    for u in units {
        q = q.bind(u.kind.as_str()).bind(u.date);
    }
    let presentes: BTreeSet<(String, NaiveDate)> = q
        .fetch_all(conn)
        .await
        .map_err(map_db_error)?
        .into_iter()
        .collect();
    Ok(units
        .iter()
        .copied()
        .filter(|u| !presentes.contains(&(u.kind.as_str().to_string(), u.date)))
        .collect())
}

/// La **date d'obtention** d'un mois (calendrier de l'AC-4) : le plus petit
/// `first_fetched_at` (UTC) des lignes `month` `afc` de ce mois (`month` au
/// 1er). `None` si le mois manque.
pub async fn month_first_fetched_at(
    conn: &mut MySqlConnection,
    month: NaiveDate,
) -> Result<Option<NaiveDateTime>, DbError> {
    sqlx::query_scalar::<_, Option<NaiveDateTime>>(
        "SELECT MIN(first_fetched_at) FROM _kesh_fetched_exchange_rates \
         WHERE source = ? AND kind = 'month' AND rate_date = ?",
    )
    .bind(SOURCE_AFC)
    .bind(month)
    .fetch_one(conn)
    .await
    .map_err(map_db_error)
}

/// Résumé des valeurs publiées d'une unité ([`fetched_unit_summary`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FetchedUnitSummary {
    /// Nombre de devises publiées.
    pub count: i64,
    /// Plus grand `fetched_at` (UTC) : l'instant pris avant l'appel sortant de
    /// la dernière récupération réussie.
    pub last_fetched_at: NaiveDateTime,
    /// Plus grande `source_date` (date de publication lue, `None` pour un mois).
    pub source_date: Option<NaiveDate>,
}

/// Résumé des valeurs publiées d'**une** unité pour une source ; `None` si
/// l'unité manque. Sert au raccourci du `POST /exchange-rates/fetch` qui a
/// attendu le verrou (AC-3 : l'unité vient d'être écrite par une récupération
/// commencée après son arrivée).
pub async fn fetched_unit_summary(
    conn: &mut MySqlConnection,
    source: &str,
    unit: RateUnit,
) -> Result<Option<FetchedUnitSummary>, DbError> {
    let (count, last, source_date): (i64, Option<NaiveDateTime>, Option<NaiveDate>) =
        sqlx::query_as(
            "SELECT COUNT(*), MAX(fetched_at), MAX(source_date) FROM _kesh_fetched_exchange_rates \
             WHERE source = ? AND kind = ? AND rate_date = ?",
        )
        .bind(source)
        .bind(unit.kind.as_str())
        .bind(unit.date)
        .fetch_one(conn)
        .await
        .map_err(map_db_error)?;
    Ok(last.map(|last_fetched_at| FetchedUnitSummary {
        count,
        last_fetched_at,
        source_date,
    }))
}

/// Les corrections de la source à annoncer à la société : ses lignes `afc`
/// dont la valeur publiée (même source, devise, genre et date) **diffère** du
/// cours retenu et n'est **pas acquittée** à cette valeur, triées par date
/// puis devise. Une ligne sans valeur publiée (installation restaurée) n'est
/// pas une notice, ni une ligne `manual` ou d'une autre source.
pub async fn list_notices(
    conn: &mut MySqlConnection,
    company_id: i64,
) -> Result<Vec<Notice>, DbError> {
    sqlx::query_as::<_, Notice>(
        "SELECT x.currency, x.kind, x.rate_date, x.rate AS effective_rate, \
                f.rate_per_unit AS published_rate, f.published_unit, f.value_changed_at \
         FROM exchange_rates x \
         JOIN _kesh_fetched_exchange_rates f \
           ON f.source = ? AND f.currency = x.currency AND f.kind = x.kind \
          AND f.rate_date = x.rate_date \
         WHERE x.company_id = ? AND x.source = ? AND f.rate_per_unit <> x.rate \
           AND (x.acknowledged_rate IS NULL OR x.acknowledged_rate <> f.rate_per_unit) \
         ORDER BY x.rate_date, x.currency, x.kind",
    )
    .bind(SOURCE_AFC)
    .bind(company_id)
    .bind(SOURCE_AFC)
    .fetch_all(conn)
    .await
    .map_err(map_db_error)
}

/// L'état de la dernière récupération ; `None` tant qu'aucune n'a eu lieu
/// (ligne créée paresseusement).
pub async fn get_fetch_status(conn: &mut MySqlConnection) -> Result<Option<FetchStatus>, DbError> {
    let row: Option<(
        String,
        NaiveDateTime,
        String,
        Option<String>,
        String,
        NaiveDate,
    )> = sqlx::query_as(
        "SELECT triggered_by, last_run_at, last_status, last_error, last_kind, last_rate_date \
             FROM _kesh_exchange_rate_fetch WHERE id = 1",
    )
    .fetch_optional(conn)
    .await
    .map_err(map_db_error)?;
    let Some((trigger, last_run_at, status, error, kind, rate_date)) = row else {
        return Ok(None);
    };
    let trigger = match trigger.as_str() {
        "auto" => FetchTrigger::Auto,
        "manual" => FetchTrigger::Manual,
        other => {
            return Err(DbError::Invariant(format!(
                "triggered_by inconnu : {other}"
            )));
        }
    };
    let status = match status.as_str() {
        "ok" => FetchOutcomeStatus::Ok,
        "not_published" => FetchOutcomeStatus::NotPublished,
        "error" => FetchOutcomeStatus::Error,
        other => return Err(DbError::Invariant(format!("last_status inconnu : {other}"))),
    };
    let kind = RateKind::try_from(kind).map_err(|e| DbError::Invariant(e.to_string()))?;
    Ok(Some(FetchStatus {
        trigger,
        last_run_at,
        status,
        error,
        kind,
        rate_date,
    }))
}

// ---------------------------------------------------------------------------
// Écritures : valeurs publiées, état, copie
// ---------------------------------------------------------------------------

/// Écrit l'état de la dernière récupération (`INSERT … ON DUPLICATE KEY
/// UPDATE` du singleton).
pub async fn record_fetch_status(
    conn: &mut MySqlConnection,
    status: &FetchStatus,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO _kesh_exchange_rate_fetch \
             (id, triggered_by, last_run_at, last_status, last_error, last_kind, last_rate_date) \
         VALUES (1, ?, ?, ?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE triggered_by = VALUES(triggered_by), \
             last_run_at = VALUES(last_run_at), last_status = VALUES(last_status), \
             last_error = VALUES(last_error), last_kind = VALUES(last_kind), \
             last_rate_date = VALUES(last_rate_date)",
    )
    .bind(status.trigger.as_str())
    .bind(status.last_run_at)
    .bind(status.status.as_str())
    .bind(status.error.as_deref())
    .bind(status.kind.as_str())
    .bind(status.rate_date)
    .execute(conn)
    .await
    .map_err(map_db_error)?;
    Ok(())
}

/// Écrit les valeurs publiées d'**une** unité (genre et date), dans la
/// transaction de l'appelant, par la règle de l'AC-3 (U1) — les lignes déjà
/// présentes de l'unité lues `FOR UPDATE` :
/// - (a) devise absente → insérée, `first_fetched_at` = `fetched_at` =
///   `value_changed_at` = `fetched_at` (l'instant pris **avant** l'appel
///   sortant) ;
/// - (b) même valeur publiée (`published_unit` et `published_rate` égaux, en
///   valeur — jamais le `rate_per_unit` dérivé) → seul `fetched_at` avance ;
///   pour un jour, `source_date` reste celle de la valeur actuelle ;
/// - (c) valeur différente (correction de la source) → la ligne prend la
///   nouvelle valeur (et, pour un jour, `source_date`), `value_changed_at` =
///   `fetched_at` ; rendue dans [`FetchWriteOutcome::corrected`].
///
/// `first_fetched_at` n'avance **jamais**. ⛔ Aucune ligne de
/// `exchange_rates` n'est modifiée : la copie ([`copy_fetched_in_tx`]) n'y
/// insère que ce qui manque. `source_date` : la date de publication lue pour
/// `day`, `None` pour `month`.
pub async fn write_fetched_unit_in_tx(
    conn: &mut MySqlConnection,
    source: &str,
    unit: RateUnit,
    source_date: Option<NaiveDate>,
    rates: &[PublishedRate],
    fetched_at: NaiveDateTime,
) -> Result<FetchWriteOutcome, DbError> {
    if (unit.kind == RateKind::Day) != source_date.is_some() {
        return Err(DbError::Invariant(
            "source_date : présente pour un cours journalier, absente pour un mensuel".into(),
        ));
    }
    let existantes: HashMap<String, (i32, Decimal)> = sqlx::query_as::<_, (String, i32, Decimal)>(
        "SELECT currency, published_unit, published_rate FROM _kesh_fetched_exchange_rates \
             WHERE source = ? AND kind = ? AND rate_date = ? FOR UPDATE",
    )
    .bind(source)
    .bind(unit.kind.as_str())
    .bind(unit.date)
    .fetch_all(&mut *conn)
    .await
    .map_err(map_db_error)?
    .into_iter()
    .map(|(c, u, r)| (c.to_ascii_uppercase(), (u, r)))
    .collect();

    let mut outcome = FetchWriteOutcome::default();
    for r in rates {
        let currency = r.currency.as_str();
        match existantes.get(currency) {
            None => {
                sqlx::query(
                    "INSERT INTO _kesh_fetched_exchange_rates \
                         (source, currency, kind, rate_date, rate_per_unit, published_unit, \
                          published_rate, source_date, first_fetched_at, fetched_at, \
                          value_changed_at) \
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(source)
                .bind(currency)
                .bind(unit.kind.as_str())
                .bind(unit.date)
                .bind(r.rate_per_unit)
                .bind(r.published_unit)
                .bind(r.published_rate)
                .bind(source_date)
                .bind(fetched_at)
                .bind(fetched_at)
                .bind(fetched_at)
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
                outcome.inserted += 1;
            }
            Some((u, p)) if *u == r.published_unit && *p == r.published_rate => {
                sqlx::query(
                    "UPDATE _kesh_fetched_exchange_rates SET fetched_at = ? \
                     WHERE source = ? AND currency = ? AND kind = ? AND rate_date = ?",
                )
                .bind(fetched_at)
                .bind(source)
                .bind(currency)
                .bind(unit.kind.as_str())
                .bind(unit.date)
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
                outcome.confirmed += 1;
            }
            Some((u, p)) => {
                sqlx::query(
                    "UPDATE _kesh_fetched_exchange_rates SET rate_per_unit = ?, \
                         published_unit = ?, published_rate = ?, source_date = ?, \
                         fetched_at = ?, value_changed_at = ? \
                     WHERE source = ? AND currency = ? AND kind = ? AND rate_date = ?",
                )
                .bind(r.rate_per_unit)
                .bind(r.published_unit)
                .bind(r.published_rate)
                .bind(source_date)
                .bind(fetched_at)
                .bind(fetched_at)
                .bind(source)
                .bind(currency)
                .bind(unit.kind.as_str())
                .bind(unit.date)
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
                outcome.corrected.push(FetchCorrection {
                    currency: currency.to_string(),
                    old_unit: *u,
                    old_rate: *p,
                    new_unit: r.published_unit,
                    new_rate: r.published_rate,
                });
            }
        }
    }
    Ok(outcome)
}

/// **Copie** dans `exchange_rates` les valeurs publiées (`afc`) des `units`
/// pour la société (AC-3 bis) : devises des **comptes** de la société
/// seulement (archivés compris), ou la seule `currency` donnée (un compte qui
/// reçoit une devise) ; **insertion seule** (`NOT EXISTS`) — une ligne
/// présente, `afc` ou `manual`, n'est jamais modifiée ; `source = 'afc'`,
/// `updated_by = NULL`, aucune ligne d'audit. Rend le nombre de lignes copiées.
///
/// **Lecture, puis une insertion par ligne**, dans l'ordre de la clé (date,
/// genre, devise) — et non un `INSERT … SELECT` : celui-ci est une insertion
/// « en masse » qui prend le verrou `AUTO-INC` de toute la table, et deux
/// copies concurrentes s'y interbloquent sous un code que le rejeu ne
/// reconnaît pas (1467, « Failed to read auto-increment value », constaté au
/// développement). Une course avec une copie concurrente est rattrapée sur
/// l'unicité : la ligne déjà posée par l'autre est sautée (`1062` sur
/// `uq_exchange_rates_key`), jamais une 500 ; aucune autre erreur n'est
/// masquée (pas d'`INSERT IGNORE`). Un interblocage éventuel (1213) remonte à
/// l'appelant, qui tient la transaction sous `retry_on_deadlock`.
pub async fn copy_fetched_in_tx(
    conn: &mut MySqlConnection,
    company_id: i64,
    units: &[RateUnit],
    currency: Option<&str>,
) -> Result<u64, DbError> {
    if units.is_empty() {
        return Ok(0);
    }
    let sql = format!(
        "SELECT f.currency, f.kind, f.rate_date, f.rate_per_unit \
         FROM _kesh_fetched_exchange_rates f \
         WHERE f.source = ? AND (f.kind, f.rate_date) IN ({}) \
           AND f.currency IN (SELECT a.currency FROM accounts a \
                              WHERE a.company_id = ? AND a.currency IS NOT NULL) \
           {} \
           AND NOT EXISTS (SELECT 1 FROM exchange_rates x WHERE x.company_id = ? \
                           AND x.currency = f.currency AND x.kind = f.kind \
                           AND x.rate_date = f.rate_date) \
         ORDER BY f.rate_date, f.kind, f.currency",
        vec!["(?, ?)"; units.len()].join(", "),
        if currency.is_some() {
            "AND f.currency = ?"
        } else {
            ""
        }
    );
    let mut q = sqlx::query_as::<_, (String, String, NaiveDate, Decimal)>(&sql).bind(SOURCE_AFC);
    for u in units {
        q = q.bind(u.kind.as_str()).bind(u.date);
    }
    q = q.bind(company_id);
    if let Some(c) = currency {
        q = q.bind(c);
    }
    let candidates = q
        .bind(company_id)
        .fetch_all(&mut *conn)
        .await
        .map_err(map_db_error)?;

    let mut copiees = 0;
    for (devise, kind, rate_date, rate) in candidates {
        let inseree = sqlx::query(
            "INSERT INTO exchange_rates \
                 (company_id, currency, kind, rate_date, rate, source, updated_by) \
             VALUES (?, ?, ?, ?, ?, ?, NULL)",
        )
        .bind(company_id)
        .bind(&devise)
        .bind(&kind)
        .bind(rate_date)
        .bind(rate)
        .bind(SOURCE_AFC)
        .execute(&mut *conn)
        .await
        .map_err(map_db_error);
        match inseree {
            Ok(_) => copiees += 1,
            Err(e) if is_key_duplicate(&e) => {
                tracing::info!(company_id, %devise, %rate_date, "copie des cours : ligne déjà posée");
            }
            Err(e) => return Err(e),
        }
    }
    Ok(copiees)
}

/// Vide les deux tables système des cours (`_test/seed`, `_test/reset`).
pub async fn clear_fetched_state(conn: &mut MySqlConnection) -> Result<(), DbError> {
    for table in ["_kesh_fetched_exchange_rates", "_kesh_exchange_rate_fetch"] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *conn)
            .await
            .map_err(map_db_error)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Écritures : cours retenus par la société
// ---------------------------------------------------------------------------

/// Vrai si l'erreur est la violation de `uq_exchange_rates_key`.
fn is_key_duplicate(e: &DbError) -> bool {
    matches!(e, DbError::UniqueConstraintViolation(m) if m.contains(UNIQUE_KEY))
}

/// Contrôle de version d'une écriture d'un cours retenu : `expected` absent =
/// création (une ligne présente → conflit), présent = remplacement de cette
/// version (ligne absente ou autre version → conflit).
fn check_version(row: Option<&CompanyRate>, expected: Option<i32>) -> Result<(), DbError> {
    match (row, expected) {
        (None, None) => Ok(()),
        (Some(r), Some(v)) if r.version == v => Ok(()),
        _ => Err(DbError::OptimisticLockConflict),
    }
}

/// Insère une ligne `exchange_rates` écrite par un utilisateur ; une création
/// concurrente (unicité) → [`DbError::OptimisticLockConflict`].
#[allow(clippy::too_many_arguments)]
async fn insert_company_rate(
    conn: &mut MySqlConnection,
    company_id: i64,
    user_id: i64,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
    rate: Decimal,
    source: &str,
) -> Result<(), DbError> {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source, updated_by) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(currency)
    .bind(kind.as_str())
    .bind(rate_date)
    .bind(rate)
    .bind(source)
    .bind(user_id)
    .execute(conn)
    .await
    .map(|_| ())
    .map_err(map_db_error)
    .map_err(|e| {
        if is_key_duplicate(&e) {
            DbError::OptimisticLockConflict
        } else {
            e
        }
    })
}

/// Remplace le cours d'une ligne `exchange_rates` à sa version : `rate`,
/// `source`, `acknowledged_*` à `NULL`, `updated_by`, `version` + 1.
async fn replace_company_rate(
    conn: &mut MySqlConnection,
    row: &CompanyRate,
    user_id: i64,
    rate: Decimal,
    source: &str,
) -> Result<(), DbError> {
    let r = sqlx::query(
        "UPDATE exchange_rates SET rate = ?, source = ?, acknowledged_rate = NULL, \
             acknowledged_at = NULL, updated_by = ?, version = version + 1 \
         WHERE id = ? AND version = ?",
    )
    .bind(rate)
    .bind(source)
    .bind(user_id)
    .bind(row.id)
    .bind(row.version)
    .execute(conn)
    .await
    .map_err(map_db_error)?;
    if r.rows_affected() == 1 {
        Ok(())
    } else {
        Err(DbError::OptimisticLockConflict)
    }
}

/// **Saisit** un cours (`PUT /exchange-rates/rates`) : la ligne prend `rate`
/// et `source = 'manual'` (créée si absente), `acknowledged_*` à `NULL`,
/// `updated_by` = l'utilisateur. Concurrence optimiste : `expected_version`
/// absent = création (ligne présente, ou création concurrente → conflit),
/// présent = remplacement de cette version (ligne absente ou autre version →
/// conflit) ; `version` + 1 à chaque remplacement. Un cours égal à la valeur
/// publiée reste `manual`.
///
/// Les bornes du cours ([`kesh_core::exchange_rates::check_rate_bounds`]), la
/// devise et la date sont contrôlées par l'appelant, avant. Rend la ligne
/// avant et après, pour l'audit.
#[allow(clippy::too_many_arguments)]
pub async fn set_manual_rate(
    conn: &mut MySqlConnection,
    company_id: i64,
    user_id: i64,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
    rate: Decimal,
    expected_version: Option<i32>,
) -> Result<RateChange, DbError> {
    let before = select_company_rate(conn, company_id, currency, kind, rate_date, true).await?;
    check_version(before.as_ref(), expected_version)?;
    match &before {
        None => {
            insert_company_rate(
                conn,
                company_id,
                user_id,
                currency,
                kind,
                rate_date,
                rate,
                SOURCE_MANUAL,
            )
            .await?
        }
        Some(row) => replace_company_rate(conn, row, user_id, rate, SOURCE_MANUAL).await?,
    }
    let after = select_company_rate(conn, company_id, currency, kind, rate_date, false).await?;
    Ok(RateChange {
        before,
        after,
        published_rate: None,
    })
}

/// Le **↑** (`POST /exchange-rates/rates/reset`) : reprendre la dernière
/// valeur publiée (`afc`, `rate_per_unit`).
///
/// - Valeur publiée présente : la ligne prend cette valeur, `source = 'afc'`,
///   `acknowledged_*` à `NULL`, `updated_by` = l'utilisateur — qu'elle soit
///   `manual`, d'une autre source, ou `afc` (correction reprise) ; créée si
///   absente.
/// - Sans valeur publiée : une ligne `manual` ou d'une autre source est
///   **supprimée** (« Retirer le cours saisi ») ; une ligne `afc` →
///   [`DbError::NotFound`] ; aucune ligne → [`DbError::NotFound`].
///
/// Contrôles, dans l'ordre : absence (404), puis `expected_version`
/// (obligatoire si la ligne existe, sinon conflit ; donné sans ligne →
/// [`DbError::InvalidInput`] `versionForbidden`), puis `expected_published`
/// (la valeur que l'écran a montrée, comparée en valeur ; changée → conflit).
/// La valeur publiée est lue sans verrou. Rend la ligne avant et après, et la
/// valeur reprise, pour l'audit.
#[allow(clippy::too_many_arguments)]
pub async fn reset_rate(
    conn: &mut MySqlConnection,
    company_id: i64,
    user_id: i64,
    currency: &str,
    kind: RateKind,
    rate_date: NaiveDate,
    expected_version: Option<i32>,
    expected_published: Option<Decimal>,
) -> Result<RateChange, DbError> {
    let before = select_company_rate(conn, company_id, currency, kind, rate_date, true).await?;
    let published = find_fetched_rate_in_tx(conn, SOURCE_AFC, currency, kind, rate_date).await?;
    match (&before, &published) {
        (None, None) => return Err(DbError::NotFound),
        (Some(row), None) if row.source.is_afc() => return Err(DbError::NotFound),
        _ => {}
    }
    if before.is_none() && expected_version.is_some() {
        return Err(DbError::InvalidInput("versionForbidden".into()));
    }
    if before.is_some() {
        check_version(before.as_ref(), expected_version)?;
    }
    let published_rate = published.as_ref().map(|p| p.rate_per_unit);
    if let (Some(attendu), Some(actuel)) = (expected_published, published_rate)
        && attendu != actuel
    {
        return Err(DbError::OptimisticLockConflict);
    }
    match (&before, published_rate) {
        (None, Some(p)) => {
            insert_company_rate(
                conn, company_id, user_id, currency, kind, rate_date, p, SOURCE_AFC,
            )
            .await?
        }
        (Some(row), Some(p)) => replace_company_rate(conn, row, user_id, p, SOURCE_AFC).await?,
        (Some(row), None) => {
            let r = sqlx::query("DELETE FROM exchange_rates WHERE id = ? AND version = ?")
                .bind(row.id)
                .bind(row.version)
                .execute(&mut *conn)
                .await
                .map_err(map_db_error)?;
            if r.rows_affected() != 1 {
                return Err(DbError::OptimisticLockConflict);
            }
        }
        // Déjà rendu plus haut ; répété pour que le `match` reste exhaustif
        // sans `unreachable!()`.
        (None, None) => return Err(DbError::NotFound),
    }
    let after = select_company_rate(conn, company_id, currency, kind, rate_date, false).await?;
    Ok(RateChange {
        before,
        after,
        published_rate,
    })
}

/// **Acquitte** des corrections de la source pour la société : sous
/// `FOR UPDATE` de chaque ligne, pose `acknowledged_rate` = la valeur du corps
/// et `acknowledged_at` = `now` (UTC) sur chaque ligne **encore `afc`** dont la
/// valeur publiée **égale encore** celle du corps (une valeur changée
/// entre-temps n'est pas acquittée : la nouvelle s'annonce). Ne change ni
/// `rate`, ni `source`, ni `version`, ni `updated_at`. Lignes traitées dans
/// l'ordre de leur clé (date, devise, genre), doublons du corps ignorés. Rend
/// les seules entrées acquittées, pour l'audit.
pub async fn acknowledge_corrections(
    conn: &mut MySqlConnection,
    company_id: i64,
    items: &[NoticeAck],
    now: NaiveDateTime,
) -> Result<Vec<NoticeAck>, DbError> {
    let mut tries: Vec<&NoticeAck> = items.iter().collect();
    tries.sort_by(|a, b| {
        (a.rate_date, &a.currency, a.kind).cmp(&(b.rate_date, &b.currency, b.kind))
    });
    tries.dedup_by(|a, b| {
        a.rate_date == b.rate_date && a.currency == b.currency && a.kind == b.kind
    });
    let mut acquittees = Vec::new();
    for item in tries {
        let Some(row) = select_company_rate(
            conn,
            company_id,
            &item.currency,
            item.kind,
            item.rate_date,
            true,
        )
        .await?
        else {
            continue;
        };
        if !row.source.is_afc() {
            continue;
        }
        let publiee =
            find_fetched_rate_in_tx(conn, SOURCE_AFC, &item.currency, item.kind, item.rate_date)
                .await?;
        if publiee.map(|p| p.rate_per_unit) != Some(item.published_rate) {
            continue;
        }
        sqlx::query(
            "UPDATE exchange_rates SET acknowledged_rate = ?, acknowledged_at = ?, \
                 updated_at = updated_at \
             WHERE id = ? AND source = ?",
        )
        .bind(item.published_rate)
        .bind(now)
        .bind(row.id)
        .bind(SOURCE_AFC)
        .execute(&mut *conn)
        .await
        .map_err(map_db_error)?;
        acquittees.push(item.clone());
    }
    Ok(acquittees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origine_lue_tolerante() {
        assert_eq!(RateOrigin::from("afc".to_string()), RateOrigin::Afc);
        assert_eq!(RateOrigin::from("manual".to_string()), RateOrigin::Manual);
        assert_eq!(
            RateOrigin::from("ecb".to_string()),
            RateOrigin::Other("ecb".into())
        );
        assert_eq!(RateOrigin::Other("ecb".into()).as_str(), "ecb");
        assert!(RateOrigin::Afc.is_afc());
        assert!(!RateOrigin::Other("afcx".into()).is_afc());
    }

    #[test]
    fn pire_statut_d_un_tour() {
        use FetchOutcomeStatus::*;
        assert_eq!([Ok, Error, NotPublished].into_iter().max(), Some(Error));
        assert_eq!([Ok, NotPublished].into_iter().max(), Some(NotPublished));
        assert_eq!(Ok.as_str(), "ok");
        assert_eq!(NotPublished.as_str(), "not_published");
        assert_eq!(FetchTrigger::Auto.as_str(), "auto");
    }

    #[test]
    fn controle_de_version() {
        assert!(check_version(None, None).is_ok());
        assert!(matches!(
            check_version(None, Some(1)),
            Err(DbError::OptimisticLockConflict)
        ));
    }
}
