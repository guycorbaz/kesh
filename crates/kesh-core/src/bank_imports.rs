//! Conversion des relevés bancaires importés (`kesh-import`) vers les types
//! domaine `kesh-core`.
//!
//! Ce module matérialise la décision architecture #7 : les crates publiables
//! (`kesh-import`, `kesh-payment`, `kesh-qrbill`) ont des types autonomes,
//! et les conversions vers les types domaine vivent côté `kesh-core` via
//! `From`/`Into`. La direction de dépendance Cargo est donc
//! `kesh-core → kesh-import` (jamais l'inverse).
//!
//! # Story 8-1a — extensions livrées
//!
//! - [`BankImportDraft`] : entête de relevé bancaire prêt à être persisté
//!   (clés étrangères incluses).
//! - [`SourceFormatTag`] : projection finie de
//!   [`kesh_import::SourceFormat`] vers les valeurs effectivement stockées
//!   en base (8-1b).
//! - [`from_imported`] : conversion `ImportedStatement` →
//!   `(BankImportDraft, Vec<BankTransactionDraft>)`. Les FK
//!   (`bank_account_id`, `company_id`, `imported_at`,
//!   `imported_by_user_id`) sont passées en paramètres pour préserver la
//!   pureté `kesh-core` (zéro I/O, zéro horloge interne).
//! - [`validate_balance`] : check de cohérence
//!   `opening + Σ transactions ≈ closing` à 0.01 près (CR-010 #62). Skip
//!   silencieux si les soldes sont absents.
//! - [`validate_currency_supported_v0_1`] : enforce le périmètre de cette
//!   version, limité à CHF (relevés CAMT.053, et CSV dont le profil lit la
//!   colonne Devise — Story 27-2).

use std::fmt;

use chrono::{NaiveDate, NaiveDateTime};
use kesh_import::{ImportedStatement, ImportedTransaction, SourceFormat};
use rust_decimal::Decimal;

use crate::errors::CoreError;
use crate::types::Money;

pub mod manual;

/// Brouillon de transaction bancaire — image projetée d'un
/// [`ImportedTransaction`] dans le vocabulaire domaine `kesh-core`.
///
/// Les clés étrangères (`bank_account_id`, `import_id`, `company_id`) ne
/// figurent pas dans ce type : elles sont injectées par
/// `kesh-api::routes::bank_imports` au moment de la persistance, à partir du
/// contexte de la requête (compte sélectionné par l'utilisateur, tenant
/// courant, hash du fichier importé).
///
/// L'IBAN de la contrepartie est conservé sous forme de `String` brute à ce
/// stade : la validation MOD-97 via [`crate::types::Iban`] est laissée à
/// Story 8-1 pour permettre un import « tolérant » (transaction conservée
/// même si l'IBAN est mal formé, avec un statut d'avertissement) plutôt
/// qu'un rejet strict du fichier entier.
#[derive(Clone, Debug, PartialEq)]
pub struct BankTransactionDraft {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Money,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
}

impl From<ImportedTransaction> for BankTransactionDraft {
    fn from(tx: ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency,
            reference: tx.reference,
            details: tx.details,
            end_to_end_id: tx.end_to_end_id,
            transaction_id: tx.transaction_id,
            counterparty_iban: tx.counterparty_iban,
            counterparty_name: tx.counterparty_name,
        }
    }
}

impl From<&ImportedTransaction> for BankTransactionDraft {
    fn from(tx: &ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency.clone(),
            reference: tx.reference.clone(),
            details: tx.details.clone(),
            end_to_end_id: tx.end_to_end_id.clone(),
            transaction_id: tx.transaction_id.clone(),
            counterparty_iban: tx.counterparty_iban.clone(),
            counterparty_name: tx.counterparty_name.clone(),
        }
    }
}

/// Entête d'import bancaire prêt à être persisté.
///
/// Le `BankImportDraft` matérialise la trace d'un fichier CAMT.053 (ou
/// CSV, dès Story 8-2) traité par l'application : le crate
/// `kesh-import` produit un [`ImportedStatement`] sans clés étrangères ;
/// `from_imported` complète celui-ci avec les FK injectées par le
/// handler API à partir du contexte de la requête (compte sélectionné,
/// tenant courant, hash du fichier, timestamp serveur, user du JWT).
///
/// Le champ [`SourceFormatTag`] est une projection finie pour la base
/// (`'CAMT053_V04'`, `'CAMT053_V08'`, `'CSV'`, et `'MANUAL'` pour un relevé
/// saisi à la main depuis la v016-2a ; la colonne est un `VARCHAR(32)`) — il évite d'embarquer
/// la `String` arbitraire de `kesh_import::SourceFormat::Camt053 { version }`.
///
/// `imported_by_user_id` est passé en paramètre par le caller (handler
/// API 8-1b) pour rester compatible avec la pureté `kesh-core` : zéro
/// horloge interne, zéro lookup user.
#[derive(Clone, Debug, PartialEq)]
pub struct BankImportDraft {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub filename: String,
    /// SHA-256 hex du fichier source (64 caractères). Utilisé pour la
    /// détection de doublons stricte (Story 8-3).
    pub file_hash: String,
    pub source_format: SourceFormatTag,
    pub statement_id: Option<String>,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub opening_balance: Option<Money>,
    pub closing_balance: Option<Money>,
    pub transaction_count: i32,
    pub imported_at: NaiveDateTime,
    pub imported_by_user_id: i64,
}

/// Projection finie du format source d'un import bancaire.
///
/// Stockée en base comme une chaîne courte (`ENUM` MariaDB) pour rester
/// indexable et stable face à l'évolution du schéma `SourceFormat` côté
/// `kesh-import` (qui peut accumuler des versions et des profils CSV
/// arbitraires).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceFormatTag {
    Camt053V04,
    Camt053V08,
    /// Story 8-2 T5.0.b — import CSV (multi-encodage + profil banque).
    /// La distinction par profil bancaire spécifique vit dans
    /// `audit_log.details_json.bank_profile_id` (cf. Limitation L4).
    Csv,
    /// Story v016-2a (#816) — relevé saisi à la main (aucun fichier).
    /// Seul `kesh_core::bank_imports::manual` le produit ; aucun chemin
    /// d'import de fichier ne peut le rendre (`from_imported`).
    Manual,
}

impl SourceFormatTag {
    /// Code stable utilisé pour la persistance (colonne `source_format`).
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Camt053V04 => "CAMT053_V04",
            Self::Camt053V08 => "CAMT053_V08",
            Self::Csv => "CSV",
            Self::Manual => "MANUAL",
        }
    }
}

/// Délègue à [`SourceFormatTag::as_db_str`] (review code Pass 2 finding
/// F18 — `format!("{}", tag)` sera utilisé en 8-1b pour le logging et
/// les messages d'erreur sans dépendre du `Debug` derive).
impl fmt::Display for SourceFormatTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_db_str())
    }
}

/// Convertit un [`ImportedStatement`] (sortie du parseur) en couple
/// `(BankImportDraft, Vec<BankTransactionDraft>)` prêt à être persisté.
///
/// Les clés étrangères (`bank_account_id`, `company_id`,
/// `imported_at`, `imported_by_user_id`) ne sont pas dérivables du
/// fichier importé — elles viennent du contexte HTTP (JWT, formulaire
/// de l'utilisateur). Elles sont passées en paramètres pour préserver
/// la pureté `kesh-core` (zéro I/O, zéro horloge interne).
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportUnknownVersion`] si le suffixe de
/// version porté par `stmt.source_format` ne correspond à aucune
/// variante connue de [`SourceFormatTag`].
pub fn from_imported(
    stmt: &ImportedStatement,
    bank_account_id: i64,
    company_id: i64,
    file_hash: String,
    filename: String,
    imported_at: NaiveDateTime,
    imported_by_user_id: i64,
) -> Result<(BankImportDraft, Vec<BankTransactionDraft>), CoreError> {
    let source_format = match &stmt.source_format {
        SourceFormat::Camt053 { version } if version == "001.04" => SourceFormatTag::Camt053V04,
        SourceFormat::Camt053 { version } if version == "001.08" => SourceFormatTag::Camt053V08,
        SourceFormat::Camt053 { version } => {
            return Err(CoreError::BankImportUnknownVersion(version.clone()));
        }
        SourceFormat::Csv { .. } => SourceFormatTag::Csv,
    };

    let transactions: Vec<BankTransactionDraft> = stmt
        .transactions
        .iter()
        .map(BankTransactionDraft::from)
        .collect();

    let draft = BankImportDraft {
        company_id,
        bank_account_id,
        filename,
        file_hash,
        source_format,
        statement_id: stmt.statement_id.clone(),
        period_from: stmt.period_from,
        period_to: stmt.period_to,
        opening_balance: stmt.opening_balance.map(Money::new),
        closing_balance: stmt.closing_balance.map(Money::new),
        transaction_count: i32::try_from(stmt.transactions.len()).unwrap_or(i32::MAX),
        imported_at,
        imported_by_user_id,
    };

    Ok((draft, transactions))
}

/// Vérifie la cohérence du solde de clôture déclaré (CR-010 #62).
///
/// Pour un relevé `stmt` portant à la fois `opening_balance` et
/// `closing_balance`, calcule `opening + Σ transactions` et compare au
/// `closing` déclaré. Tolère un écart absolu de **0.01** (un centime,
/// règle d'arrondi commerciale).
///
/// La comparaison utilise `>` strict : un écart **égal** à 0.01 est
/// accepté (dans la tolérance), seul un écart **strictement supérieur**
/// déclenche l'erreur. Voir tests `validate_balance_passes_at_exactly_one_cent`
/// et `validate_balance_fails_just_above_one_cent` pour les bornes.
///
/// Si l'un des deux soldes est absent, retourne `Ok(())` (skip
/// silencieux). Cette tolérance est calibrée pour les CSV (Story 8-2)
/// dont certains profils n'embarquent pas les soldes. **Pour CAMT.053
/// spécifiquement**, le schéma ISO 20022 impose `OPBD` et `CLBD` : un
/// `ImportedStatement` issu du parseur `kesh-import::camt053` sans
/// soldes doit être traité comme un signal de fichier malformé en
/// amont (le parseur lève `MissingRequiredField` sur un `<Bal>`
/// partiellement renseigné depuis Story 8-1a Pass 1 review). Le
/// présent skip ne s'applique donc en pratique qu'aux sources CSV.
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportBalanceMismatch`] si l'écart absolu
/// dépasse 0.01.
pub fn validate_balance(stmt: &ImportedStatement) -> Result<(), CoreError> {
    let (Some(opening), Some(closing)) = (stmt.opening_balance, stmt.closing_balance) else {
        return Ok(());
    };
    let sum = stmt.sum_transactions();
    let expected = opening + sum;
    let diff = (expected - closing).abs();
    if diff > Decimal::new(1, 2) {
        return Err(CoreError::BankImportBalanceMismatch {
            opening: Money::new(opening),
            closing: Money::new(closing),
            sum: Money::new(sum),
            diff: Money::new(diff),
        });
    }
    Ok(())
}

/// Clé composite stable de détection de doublon ligne-par-ligne (Story
/// 8-3 FR43 partie 2).
///
/// La clé compose `(booking_date, amount, reference_normalized,
/// bank_account_id)`, où `reference_normalized` est obtenue via
/// `trim().to_lowercase()` appliqué au premier identifiant non-vide
/// parmi `reference`, `end_to_end_id`, `transaction_id` (sinon chaîne
/// vide). Cf. spec §dedup-key.
///
/// Pourquoi pas de tolérance sur `booking_date` ni sur `amount` :
/// décision R1 epic-8 — les imports CAMT.053 et CSV publient des
/// dates exactes (XSD CAMT, formats CSV bien définis), une tolérance
/// invitee des faux positifs sans gain. Pour les écarts d'arrondi,
/// la valeur est remontée telle quelle (pas de `round_to_centimes`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DuplicateKey {
    pub booking_date: NaiveDate,
    pub amount: Decimal,
    pub reference_normalized: String,
    pub bank_account_id: i64,
}

impl fmt::Display for DuplicateKey {
    /// Forme `"yyyy-mm-dd|amount|ref|account_id"` utilisée par
    /// [`DuplicateLine::key`] pour le debug et l'affichage UI.
    ///
    /// **L-2 (Pass 2 review)** : la référence peut théoriquement
    /// contenir le séparateur `|`. On l'échappe en `\|` pour éviter
    /// que le frontend qui re-parse cette clé pour des affichages
    /// debug ne confonde un délimiteur avec un caractère de
    /// référence (cas rare CAMT.053 / CSV mais possible).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let escaped_ref = self.reference_normalized.replace('|', "\\|");
        write!(
            f,
            "{}|{}|{}|{}",
            self.booking_date, self.amount, escaped_ref, self.bank_account_id
        )
    }
}

/// Construit une [`DuplicateKey`] depuis un brouillon de transaction.
///
/// Utilisé par `kesh-api` côté nouvel import (pré-INSERT) pour calculer
/// les clés des transactions candidates et les comparer aux clés
/// existantes en base via [`detect_duplicate_lines`].
pub fn dedup_key_from_draft(t: &BankTransactionDraft, bank_account_id: i64) -> DuplicateKey {
    DuplicateKey {
        booking_date: t.booking_date,
        // M4 (Pass 1 review) — `Decimal::normalize()` removes trailing zeros so that
        // 1.50 and 1.5 produce identical Hash + Eq. Without it, sqlx may return DB
        // rows with a different scale than the parsed file, breaking the HashMap match.
        amount: t.amount.amount().normalize(),
        reference_normalized: normalize_reference_fallback(
            t.reference.as_deref(),
            t.end_to_end_id.as_deref(),
            t.transaction_id.as_deref(),
        ),
        bank_account_id,
    }
}

/// Construit une [`DuplicateKey`] depuis des scalars — utilisé par
/// `kesh-api` pour mapper les `BankTransaction` chargés via
/// `kesh-db::repositories::bank_transactions::find_in_dedup_window`
/// sans introduire de dépendance `kesh-core → kesh-db`.
pub fn dedup_key_scalar(
    booking_date: NaiveDate,
    amount: Decimal,
    reference: Option<&str>,
    end_to_end_id: Option<&str>,
    transaction_id: Option<&str>,
    bank_account_id: i64,
) -> DuplicateKey {
    DuplicateKey {
        booking_date,
        // M4 — see dedup_key_from_draft: scale-stable Hash via .normalize().
        amount: amount.normalize(),
        reference_normalized: normalize_reference_fallback(
            reference,
            end_to_end_id,
            transaction_id,
        ),
        bank_account_id,
    }
}

/// `trim().to_lowercase()` sur le premier `Some(s)` non-vide de la
/// chaîne `reference → end_to_end_id → transaction_id`, sinon retourne
/// la chaîne vide. Le cas chaîne vide est conservateur : deux
/// transactions sans aucun identifiant et même date/montant/compte
/// matchent comme doublons (l'utilisateur peut override via
/// `confirmDuplicateLines=import` si c'est un faux positif).
fn normalize_reference_fallback(
    reference: Option<&str>,
    end_to_end_id: Option<&str>,
    transaction_id: Option<&str>,
) -> String {
    let raw = reference
        .filter(|s| !s.trim().is_empty())
        .or_else(|| end_to_end_id.filter(|s| !s.trim().is_empty()))
        .or_else(|| transaction_id.filter(|s| !s.trim().is_empty()))
        .unwrap_or("");
    raw.trim().to_lowercase()
}

/// Résultat d'une détection de doublon ligne-par-ligne — pointe une
/// transaction du nouvel import (`new_index`) qui matche une
/// transaction existante en base (`existing_transaction_id`).
///
/// Le champ `key` contient la forme texte de [`DuplicateKey`] pour
/// debug et affichage UI.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct DuplicateLine {
    pub new_index: usize,
    pub existing_transaction_id: i64,
    pub key: String,
}

/// Largeur de `bank_transactions.reference`, `end_to_end_id`, `transaction_id`
/// et `counterparty_name` : `VARCHAR(255)`, en caractères
/// (`20260504000001_bank_imports.sql`).
const TEXT_COLUMN_CHARS: usize = 255;
/// Largeur de `bank_transactions.counterparty_iban` : `VARCHAR(34)`.
const IBAN_COLUMN_CHARS: usize = 34;
/// Largeur de `bank_transactions.details` : `TEXT`, en octets.
const DETAILS_COLUMN_BYTES: usize = 65_535;

/// Les **sept textes** qu'une ligne porte en base en dehors de sa clé, et qui ne
/// changent plus après l'import (33-2, C-e332-3). Sans `currency` : son calcul a
/// changé depuis les premières versions (e04122dc), une ligne ancienne ne l'égale
/// donc pas forcément.
#[derive(Clone, Debug)]
pub struct LineTexts {
    pub value_date: Option<NaiveDate>,
    pub reference: Option<String>,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
    pub details: String,
}

impl LineTexts {
    /// Les textes d'un brouillon, tels qu'il les porte.
    pub fn from_draft(t: &BankTransactionDraft) -> Self {
        Self {
            value_date: t.value_date,
            reference: t.reference.clone(),
            end_to_end_id: t.end_to_end_id.clone(),
            transaction_id: t.transaction_id.clone(),
            counterparty_iban: t.counterparty_iban.clone(),
            counterparty_name: t.counterparty_name.clone(),
            details: t.details.clone(),
        }
    }

    /// Les mêmes textes, coupés à la largeur de leur colonne : ce qu'un serveur sans
    /// mode strict en aurait gardé à l'écriture (33-2, AC-F4).
    fn cut_to_column_width(self) -> Self {
        let cut = |s: Option<String>, chars: usize| s.map(|s| cut_to_chars(s, chars));
        let mut details = self.details;
        details.truncate(details.floor_char_boundary(DETAILS_COLUMN_BYTES));
        Self {
            value_date: self.value_date,
            reference: cut(self.reference, TEXT_COLUMN_CHARS),
            end_to_end_id: cut(self.end_to_end_id, TEXT_COLUMN_CHARS),
            transaction_id: cut(self.transaction_id, TEXT_COLUMN_CHARS),
            counterparty_iban: cut(self.counterparty_iban, IBAN_COLUMN_CHARS),
            counterparty_name: cut(self.counterparty_name, TEXT_COLUMN_CHARS),
            details,
        }
    }
}

/// Les `chars` premiers caractères de `s`.
fn cut_to_chars(mut s: String, chars: usize) -> String {
    if let Some((end, _)) = s.char_indices().nth(chars) {
        s.truncate(end);
    }
    s
}

/// Égalité stricte des sept textes. **Écrite à la main**, par destructuration
/// exhaustive : un champ ajouté à [`LineTexts`] ne compile pas sans être nommé ici.
impl PartialEq for LineTexts {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            value_date,
            reference,
            end_to_end_id,
            transaction_id,
            counterparty_iban,
            counterparty_name,
            details,
        } = self;
        *value_date == other.value_date
            && *reference == other.reference
            && *end_to_end_id == other.end_to_end_id
            && *transaction_id == other.transaction_id
            && *counterparty_iban == other.counterparty_iban
            && *counterparty_name == other.counterparty_name
            && *details == other.details
    }
}

impl Eq for LineTexts {}

/// Ce que la lecture de la 0.17.3 donnait d'une ligne dont la clé a changé depuis
/// (#819, 33-2) : ses textes coupés à la largeur de leur colonne, et la clé qui en
/// dérive, ce qui les rend comparables à une ligne stockée quel que soit le `sql_mode`
/// du serveur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyLine {
    key: DuplicateKey,
    texts: LineTexts,
}

impl LegacyLine {
    /// La ligne que porte le brouillon d'une lecture héritée. La coupe se fait ici, une
    /// seule fois : l'égalité de [`LineTexts`] reste stricte. La clé se calcule **après**
    /// la coupe : une clé stockée dérive d'un identifiant déjà coupé (revue P1, E-3).
    pub fn from_draft(t: &BankTransactionDraft, bank_account_id: i64) -> Self {
        let texts = LineTexts::from_draft(t).cut_to_column_width();
        let key = dedup_key_scalar(
            t.booking_date,
            t.amount.amount(),
            texts.reference.as_deref(),
            texts.end_to_end_id.as_deref(),
            texts.transaction_id.as_deref(),
            bank_account_id,
        );
        Self { key, texts }
    }

    /// La clé de doublon que la ligne porte en base si la 0.17.3 l'a importée.
    pub fn key(&self) -> &DuplicateKey {
        &self.key
    }
}

/// Une ligne dont l'alias égale la clé d'une ligne existante, **sans** en porter les
/// textes (33-2, C-e332-4) : ni doublon ni silence. Elle s'importe, et l'aperçu la
/// signale.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UncertainLegacyLine {
    pub new_index: usize,
    /// Le plus petit `id` des lignes existantes de cette clé.
    pub existing_transaction_id: i64,
}

/// Le résultat de [`detect_duplicate_lines_with_aliases`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DuplicateLines {
    pub duplicates: Vec<DuplicateLine>,
    pub uncertain: Vec<UncertainLegacyLine>,
}

/// Lignes existantes par clé, chacune avec sa charge (`()` sans alias, ses textes
/// sinon).
type KeyIndex<'a, V> = std::collections::HashMap<&'a DuplicateKey, Vec<(i64, V)>>;

/// Le plus petit `id` d'un groupe de lignes de même clé : déterministe quel que soit
/// l'ordre de lecture (`find_in_dedup_window` ne trie pas).
fn smallest_id<V>(rows: &[(i64, V)]) -> Option<i64> {
    rows.iter().map(|(id, _)| *id).min()
}

/// Le parcours commun aux deux détections : une ligne dont la clé égale celle d'une
/// ligne existante est un doublon, **quels que soient ses textes** ; sinon `on_miss`
/// décide, à partir de son index.
fn scan_lines<V>(
    new: &[BankTransactionDraft],
    bank_account_id: i64,
    index: &KeyIndex<'_, V>,
    mut on_miss: impl FnMut(usize) -> Option<DuplicateLine>,
) -> Vec<DuplicateLine> {
    let mut out = Vec::new();
    for (idx, tx) in new.iter().enumerate() {
        let key = dedup_key_from_draft(tx, bank_account_id);
        let direct = index.get(&key).and_then(|rows| smallest_id(rows));
        if let Some(existing_id) = direct {
            out.push(DuplicateLine {
                new_index: idx,
                existing_transaction_id: existing_id,
                key: key.to_string(),
            });
        } else if let Some(line) = on_miss(idx) {
            out.push(line);
        }
    }
    out
}

/// Détecte les doublons ligne-par-ligne entre un nouvel import et les
/// transactions existantes (Story 8-3 FR43 partie 2).
///
/// Compare en `O(N + M)` via un index par [`DuplicateKey`]
/// construit sur les transactions existantes. Pour chaque transaction
/// nouvelle, retourne un [`DuplicateLine`] si la clé matche.
///
/// Le caller (`kesh-api`) construit `existing_keys` via :
/// ```ignore
/// existing.iter().map(|t| (t.id, dedup_key_scalar(
///     t.booking_date, t.amount,
///     t.reference.as_deref(), t.end_to_end_id.as_deref(),
///     t.transaction_id.as_deref(), t.bank_account_id))).collect()
/// ```
///
/// **Hors-scope helper** : la fenêtre `period_from..period_to` est
/// filtrée en amont par
/// `kesh-db::repositories::bank_transactions::find_in_dedup_window`.
/// Le helper n'applique aucun filtrage de date supplémentaire — si un
/// caller passe des keys hors-fenêtre, les matchs sont retournés.
///
/// **Hors-scope helper** : doublons intra-import (`new vs new`) non
/// détectés en v0.1. Cf. Limitations connues v0.1 L15.
pub fn detect_duplicate_lines(
    new: &[BankTransactionDraft],
    bank_account_id: i64,
    existing_keys: &[(i64, DuplicateKey)],
) -> Vec<DuplicateLine> {
    let mut index: KeyIndex<'_, ()> = KeyIndex::with_capacity(existing_keys.len());
    for (id, key) in existing_keys {
        index.entry(key).or_default().push((*id, ()));
    }
    scan_lines(new, bank_account_id, &index, |_| None)
}

/// Variante de [`detect_duplicate_lines`] qui reconnaît aussi une ligne par son
/// **alias**, ce que la 0.17.3 en lisait (0.17.4, #819 ; 33-2, #915).
///
/// `aliases` est aligné par index sur `new` (un tableau plus court, ou vide, vaut
/// `None` pour les lignes restantes). La clé de la ligne est essayée d'abord : si
/// elle égale une clé existante, c'est un doublon, **quels que soient ses textes**.
/// Sinon, si son alias égale une clé existante :
///
/// - une ligne de cette clé porte les mêmes sept textes ([`LineTexts`]) : doublon,
///   désignant le plus petit `id` parmi celles qui les portent ;
/// - aucune : [`UncertainLegacyLine`], désignant le plus petit `id` de la clé. La
///   clé d'alias seule ne suffit plus, car elle ne compare qu'un morceau de la
///   référence (#915).
///
/// Le champ [`DuplicateLine::key`] porte la clé qui a égalé. **Limite (AC-F5)** : une
/// ligne neuve dont les sept textes, lus à l'ancienne, égalent ceux d'une ligne
/// existante reste un doublon : rien ne la distingue d'une ligne déjà importée.
///
/// L'usage : la lecture CAMT.053 de la 0.17.4 lit en entier un identifiant qui
/// porte une entité (`REF &amp; Co`), là où la 0.17.3 n'en gardait que le
/// dernier morceau ; la clé d'une ligne déjà importée diffère donc de celle que
/// calcule la lecture neuve. Sans alias, les doublons sont exactement ceux de
/// [`detect_duplicate_lines`] (CSV), et aucune ligne n'est incertaine.
///
/// **Plusieurs lignes de même clé** (F10, Pass 3 review) : possible après
/// `confirmDuplicateLines=import`. Le comptage côté **nouvelles** transactions reste
/// correct ; `existing_transaction_id` n'en désigne qu'une.
pub fn detect_duplicate_lines_with_aliases(
    new: &[BankTransactionDraft],
    bank_account_id: i64,
    existing: &[(i64, DuplicateKey, LineTexts)],
    aliases: &[Option<LegacyLine>],
) -> DuplicateLines {
    let mut index: KeyIndex<'_, &LineTexts> = KeyIndex::with_capacity(existing.len());
    for (id, key, texts) in existing {
        index.entry(key).or_default().push((*id, texts));
    }

    let mut uncertain = Vec::new();
    let duplicates = scan_lines(new, bank_account_id, &index, |idx| {
        let alias = aliases.get(idx)?.as_ref()?;
        let rows = index.get(&alias.key)?;
        let same_texts = rows
            .iter()
            .filter(|(_, texts)| **texts == alias.texts)
            .map(|(id, _)| *id)
            .min();
        match same_texts {
            Some(existing_transaction_id) => Some(DuplicateLine {
                new_index: idx,
                existing_transaction_id,
                key: alias.key.to_string(),
            }),
            None => {
                if let Some(existing_transaction_id) = smallest_id(rows) {
                    uncertain.push(UncertainLegacyLine {
                        new_index: idx,
                        existing_transaction_id,
                    });
                }
                None
            }
        }
    });
    DuplicateLines {
        duplicates,
        uncertain,
    }
}

/// Vérifie que la devise du relevé importé est supportée par le
/// périmètre de cette version (CHF uniquement).
///
/// Le rejet vit côté `kesh-core` plutôt que côté `kesh-import` pour
/// conserver le parseur agnostique du périmètre fonctionnel (un autre
/// produit consommant `kesh-import` pourrait accepter EUR ou USD).
///
/// # Erreurs
///
/// Retourne [`CoreError::BankImportUnsupportedCurrency`] avec le code
/// devise rencontré pour toute devise autre que `"CHF"`.
pub fn validate_currency_supported_v0_1(stmt: &ImportedStatement) -> Result<(), CoreError> {
    if stmt.currency != "CHF" {
        return Err(CoreError::BankImportUnsupportedCurrency(
            stmt.currency.clone(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn fixture() -> ImportedTransaction {
        ImportedTransaction {
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 3).unwrap()),
            amount: dec!(1234.56),
            currency: "CHF".to_string(),
            reference: Some("RF18539007547034".to_string()),
            details: "Loyer mai 2026".to_string(),
            end_to_end_id: None,
            transaction_id: Some("BANK-TX-42".to_string()),
            counterparty_iban: Some("CH9300762011623852957".to_string()),
            counterparty_name: Some("Acme SA".to_string()),
        }
    }

    #[test]
    fn from_owned_imported_transaction() {
        let tx = fixture();
        let draft: BankTransactionDraft = tx.into();
        assert_eq!(
            draft.booking_date,
            NaiveDate::from_ymd_opt(2026, 5, 2).unwrap()
        );
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
        assert_eq!(draft.currency, "CHF");
        assert_eq!(draft.reference.as_deref(), Some("RF18539007547034"));
        assert_eq!(draft.transaction_id.as_deref(), Some("BANK-TX-42"));
    }

    #[test]
    fn from_borrowed_imported_transaction_does_not_consume() {
        let tx = fixture();
        let draft: BankTransactionDraft = (&tx).into();
        // tx still usable after the conversion.
        assert_eq!(tx.amount, dec!(1234.56));
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
    }

    #[test]
    fn negative_amount_preserved_as_signed_money() {
        let tx = ImportedTransaction {
            amount: dec!(-50.00),
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.amount.is_negative());
        assert_eq!(draft.amount, Money::new(dec!(-50.00)));
    }

    #[test]
    fn missing_optional_fields_propagate_as_none() {
        let tx = ImportedTransaction {
            value_date: None,
            reference: None,
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.value_date.is_none());
        assert!(draft.reference.is_none());
        assert!(draft.end_to_end_id.is_none());
        assert!(draft.transaction_id.is_none());
        assert!(draft.counterparty_iban.is_none());
        assert!(draft.counterparty_name.is_none());
    }

    fn statement_fixture(opening: Option<Decimal>, closing: Option<Decimal>) -> ImportedStatement {
        ImportedStatement {
            statement_id: Some("STMT-TEST".into()),
            account_iban: "CH4431999123000889012".into(),
            currency: "CHF".into(),
            opening_balance: opening,
            closing_balance: closing,
            period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            transactions: vec![ImportedTransaction {
                amount: dec!(100.00),
                ..fixture()
            }],
            source_format: SourceFormat::Camt053 {
                version: "001.04".into(),
            },
        }
    }

    #[test]
    fn validate_balance_passes_when_within_tolerance() {
        // Test du chemin Ok pour deux valeurs de diff sous la borne :
        //   - diff = 0    (cas idéal : opening + Σ = closing)
        //   - diff = 0.005 (sous tolérance, pas à la borne)
        //
        // **Note review code Pass 2 finding F17** : ce test seul est
        // vacuux (une impl `Ok(())` constante le passerait). Il faut
        // le lire en paire avec :
        //   - `validate_balance_passes_at_exactly_one_cent` (borne =)
        //   - `validate_balance_fails_just_above_one_cent` (borne +ε)
        //   - `validate_balance_fails_when_diff_exceeds_one_cent` (loin)
        // Ensemble, ces 4 tests verrouillent la sémantique `> 0.01`
        // strict avec la borne incluse.

        // opening 1000.00 + Σ 100.00 = closing 1100.00 → diff 0
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        validate_balance(&stmt).expect("balance check passes");

        // diff = 0.005 < 0.01 tolerance
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.005)));
        validate_balance(&stmt).expect("balance check passes within tolerance");
    }

    #[test]
    fn validate_balance_fails_when_diff_exceeds_one_cent() {
        // opening 1000 + 100 = 1100, but closing = 1500 → diff 400
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1500.00)));
        let err = validate_balance(&stmt).expect_err("balance check fails");
        match err {
            CoreError::BankImportBalanceMismatch {
                opening,
                closing,
                sum,
                diff,
            } => {
                assert_eq!(opening, Money::new(dec!(1000.00)));
                assert_eq!(closing, Money::new(dec!(1500.00)));
                assert_eq!(sum, Money::new(dec!(100.00)));
                assert_eq!(diff, Money::new(dec!(400.00)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_balance_skipped_when_balances_missing() {
        // opening absent → skip
        let stmt = statement_fixture(None, Some(dec!(1100.00)));
        validate_balance(&stmt).expect("skip silencieux si opening absent");

        // closing absent → skip
        let stmt = statement_fixture(Some(dec!(1000.00)), None);
        validate_balance(&stmt).expect("skip silencieux si closing absent");

        // les deux absents → skip
        let stmt = statement_fixture(None, None);
        validate_balance(&stmt).expect("skip silencieux si les deux absents");
    }

    #[test]
    fn validate_balance_passes_at_exactly_one_cent() {
        // Régression review code Pass 1 finding F10 : la borne `> 0.01`
        // (strict) accepte un écart **exactement** égal à 0.01. Ce test
        // verrouille la sémantique inclusive de la tolérance contre une
        // régression `>` → `>=`.
        // opening 1000 + Σ 100 = 1100, closing = 1100.01 → diff = 0.01
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.01)));
        validate_balance(&stmt)
            .expect("écart de 0.01 exactement doit passer (tolérance inclusive)");
    }

    #[test]
    fn validate_balance_fails_just_above_one_cent() {
        // Régression review code Pass 1 finding F10 : juste au-dessus
        // de 0.01 (0.011) doit échouer. Verrouille la sémantique
        // contre une régression `>` → `<`.
        // opening 1000 + Σ 100 = 1100, closing = 1100.011 → diff = 0.011
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.011)));
        let err = validate_balance(&stmt).expect_err("écart de 0.011 doit échouer");
        match err {
            CoreError::BankImportBalanceMismatch { diff, .. } => {
                assert_eq!(diff, Money::new(dec!(0.011)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_balance_handles_signed_mix_correctly() {
        // Régression review code Pass 2 finding F19 : verrouille la
        // somme signée des transactions (CRDT positif + DBIT négatif).
        // Sans ça, un bug de `sum_transactions` qui sommerait des
        // valeurs absolues au lieu des valeurs signées resterait
        // invisible — tous les autres tests utilisent uniquement
        // amount = +100.
        // opening 1000 + (+200) + (-50) + (+30) = 1180 = closing 1180 → diff 0
        let mut stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1180.00)));
        stmt.transactions = vec![
            ImportedTransaction {
                amount: dec!(200.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(-50.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(30.00),
                ..fixture()
            },
        ];
        validate_balance(&stmt)
            .expect("somme signée 200 + (-50) + 30 = 180 doit aligner avec 1000 → 1180");
    }

    #[test]
    fn validate_balance_detects_balance_mismatch_fixture() {
        // Régression review code Pass 2 finding F16 (verdict NO-GO
        // conditionnel Auditor) : la fixture `v04_balance_mismatch.xml`
        // était orpheline — aucun test d'intégration ne la chargeait,
        // ce qui rendait AC #9 PARTIAL. Ce test parse la fixture via
        // `kesh-import` et vérifie que `validate_balance` détecte
        // l'incohérence (opening 1000 + Σ 100 = 1100 ≠ closing 1500
        // → diff 400).
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-import/tests/fixtures/camt053/v04_balance_mismatch.xml");
        let xml = std::fs::read(&fixture_path)
            .unwrap_or_else(|e| panic!("fixture {} introuvable : {e}", fixture_path.display()));
        let stmts = kesh_import::parse_camt053(&xml).expect("parse OK");
        assert_eq!(stmts.len(), 1);
        let s = &stmts[0];
        assert_eq!(s.opening_balance, Some(dec!(1000.00)));
        assert_eq!(s.closing_balance, Some(dec!(1500.00)));
        assert_eq!(s.transactions.len(), 1);
        assert_eq!(s.transactions[0].amount, dec!(100.00));

        let err = validate_balance(s).expect_err("balance check doit fail");
        match err {
            CoreError::BankImportBalanceMismatch { diff, .. } => {
                assert_eq!(diff, Money::new(dec!(400.00)));
            }
            other => panic!("attendu BankImportBalanceMismatch, obtenu {other:?}"),
        }
    }

    #[test]
    fn validate_currency_chf_passes() {
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        validate_currency_supported_v0_1(&stmt).expect("CHF accepté");
    }

    #[test]
    fn validate_currency_eur_rejected_v0_1() {
        let mut stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        stmt.currency = "EUR".into();
        let err = validate_currency_supported_v0_1(&stmt).expect_err("EUR rejeté v0.1");
        match err {
            CoreError::BankImportUnsupportedCurrency(c) => assert_eq!(c, "EUR"),
            other => panic!("attendu BankImportUnsupportedCurrency, obtenu {other:?}"),
        }
    }

    #[test]
    fn from_imported_injects_fk() {
        let stmt = statement_fixture(Some(dec!(1000.00)), Some(dec!(1100.00)));
        let imported_at = NaiveDate::from_ymd_opt(2026, 5, 4)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let (draft, txs) = from_imported(
            &stmt,
            42, // bank_account_id
            7,  // company_id
            "abcdef0123456789".into(),
            "stmt.xml".into(),
            imported_at,
            13, // imported_by_user_id
        )
        .expect("conversion OK");

        assert_eq!(draft.bank_account_id, 42);
        assert_eq!(draft.company_id, 7);
        assert_eq!(draft.file_hash, "abcdef0123456789");
        assert_eq!(draft.filename, "stmt.xml");
        assert_eq!(draft.imported_at, imported_at);
        assert_eq!(draft.imported_by_user_id, 13);
        assert_eq!(draft.source_format, SourceFormatTag::Camt053V04);
        assert_eq!(draft.source_format.as_db_str(), "CAMT053_V04");
        assert_eq!(draft.statement_id.as_deref(), Some("STMT-TEST"));
        assert_eq!(draft.opening_balance, Some(Money::new(dec!(1000.00))));
        assert_eq!(draft.closing_balance, Some(Money::new(dec!(1100.00))));
        assert_eq!(draft.transaction_count, 1);
        assert_eq!(txs.len(), 1);
    }

    #[test]
    fn from_imported_returns_one_draft_per_transaction() {
        let mut stmt = statement_fixture(None, None);
        stmt.transactions = vec![
            ImportedTransaction {
                amount: dec!(10.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(20.00),
                ..fixture()
            },
            ImportedTransaction {
                amount: dec!(-5.00),
                ..fixture()
            },
        ];
        let (draft, txs) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("conversion OK");

        assert_eq!(txs.len(), 3);
        assert_eq!(draft.transaction_count, 3);
        assert_eq!(txs[0].amount, Money::new(dec!(10.00)));
        assert_eq!(txs[1].amount, Money::new(dec!(20.00)));
        assert_eq!(txs[2].amount, Money::new(dec!(-5.00)));
    }

    #[test]
    fn from_imported_unknown_version_returns_err() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Camt053 {
            version: "001.99".into(),
        };
        let err = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect_err("version inconnue rejetée");
        match err {
            CoreError::BankImportUnknownVersion(v) => assert_eq!(v, "001.99"),
            other => panic!("attendu BankImportUnknownVersion, obtenu {other:?}"),
        }
    }

    #[test]
    fn from_imported_v08_maps_to_correct_tag() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Camt053 {
            version: "001.08".into(),
        };
        let (draft, _) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("v08 OK");
        assert_eq!(draft.source_format, SourceFormatTag::Camt053V08);
        assert_eq!(draft.source_format.as_db_str(), "CAMT053_V08");
    }

    // ── Story 8-3 — détection de doublons ligne-par-ligne (T2.2) ────

    fn draft(
        booking_date: NaiveDate,
        amount: Decimal,
        reference: Option<&str>,
        end_to_end_id: Option<&str>,
        transaction_id: Option<&str>,
    ) -> BankTransactionDraft {
        BankTransactionDraft {
            booking_date,
            value_date: None,
            amount: Money::new(amount),
            currency: "CHF".into(),
            reference: reference.map(|s| s.to_string()),
            details: String::new(),
            end_to_end_id: end_to_end_id.map(|s| s.to_string()),
            transaction_id: transaction_id.map(|s| s.to_string()),
            counterparty_iban: None,
            counterparty_name: None,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn detect_duplicate_lines_finds_match_on_full_key() {
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some("REF-ABC"),
            None,
            None,
        )];
        let existing = vec![(
            42,
            dedup_key_scalar(
                date(2026, 5, 15),
                dec!(150.00),
                Some("REF-ABC"),
                None,
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].new_index, 0);
        assert_eq!(result[0].existing_transaction_id, 42);
        // M4 (Pass 1 review) — Decimal::normalize() strips trailing zeros,
        // so "150.00" formats as "150" via DuplicateKey::Display.
        assert_eq!(result[0].key, "2026-05-15|150|ref-abc|17");
    }

    // ── Story v0174-correctifs-1 (#819, AC-1.6) — l'alias de clé ────

    fn key(reference: &str) -> DuplicateKey {
        dedup_key_scalar(
            date(2026, 10, 10),
            dec!(10.00),
            Some(reference),
            None,
            None,
            17,
        )
    }

    /// Des textes de référence : tous remplis, pour qu'une colonne qui diffère se voie.
    fn texts() -> LineTexts {
        LineTexts {
            value_date: Some(date(2026, 10, 11)),
            reference: Some("Co".into()),
            end_to_end_id: Some("E2E".into()),
            transaction_id: Some("TX".into()),
            counterparty_iban: Some("CH00".into()),
            counterparty_name: Some("Fils".into()),
            details: "solde".into(),
        }
    }

    fn existing_row(id: i64, reference: &str, t: LineTexts) -> (i64, DuplicateKey, LineTexts) {
        (id, key(reference), t)
    }

    fn alias(reference: &str, t: LineTexts) -> Option<LegacyLine> {
        Some(LegacyLine {
            key: key(reference),
            texts: t.cut_to_column_width(),
        })
    }

    fn new_line() -> Vec<BankTransactionDraft> {
        vec![draft(
            date(2026, 10, 10),
            dec!(10.00),
            Some("REF & Co"),
            None,
            None,
        )]
    }

    /// Des alias vides rendent exactement ce que rend `detect_duplicate_lines`.
    #[test]
    fn aliases_empty_behaves_like_detect_duplicate_lines_819() {
        let new = vec![
            draft(date(2026, 10, 10), dec!(10.00), Some("A"), None, None),
            draft(date(2026, 10, 10), dec!(10.00), Some("B"), None, None),
        ];
        let plain = detect_duplicate_lines(&new, 17, &[(42, key("b"))]);
        let existing = vec![existing_row(42, "b", texts())];
        for aliases in [&[][..], &[None, None][..]] {
            let r = detect_duplicate_lines_with_aliases(&new, 17, &existing, aliases);
            assert_eq!(r.duplicates, plain);
            assert!(r.uncertain.is_empty());
        }
        assert_eq!(plain.len(), 1);
    }

    /// Un alias qui égale une clé existante, aux mêmes textes, fait un doublon, sous
    /// la clé de l'alias.
    #[test]
    fn alias_with_equal_texts_is_a_duplicate_819() {
        let new = new_line();
        assert!(detect_duplicate_lines(&new, 17, &[(42, key("Co"))]).is_empty());
        let existing = vec![existing_row(42, "Co", texts())];
        let r = detect_duplicate_lines_with_aliases(&new, 17, &existing, &[alias("Co", texts())]);
        assert_eq!(r.duplicates.len(), 1);
        assert_eq!(r.duplicates[0].new_index, 0);
        assert_eq!(r.duplicates[0].existing_transaction_id, 42);
        assert_eq!(r.duplicates[0].key, key("Co").to_string());
        assert!(r.uncertain.is_empty());
    }

    /// AC-S4 — un alias égal à la clé neuve n'est pas compté deux fois, et la clé
    /// directe ne regarde pas les textes (« le libellé n'en fait pas partie »).
    #[test]
    fn direct_key_is_a_duplicate_whatever_its_texts_819() {
        let new = vec![draft(
            date(2026, 10, 10),
            dec!(10.00),
            Some("A"),
            None,
            None,
        )];
        let other = LineTexts {
            details: "autre libellé".into(),
            ..texts()
        };
        let existing = vec![existing_row(42, "a", other)];
        let r = detect_duplicate_lines_with_aliases(&new, 17, &existing, &[alias("a", texts())]);
        assert_eq!(r.duplicates.len(), 1);
        assert_eq!(r.duplicates[0].key, key("a").to_string());
        assert!(r.uncertain.is_empty());
    }

    /// AC-F4 — **chacune** des sept colonnes, seule à différer, fait passer de doublon
    /// à incertain.
    #[test]
    fn each_of_the_seven_columns_counts_915() {
        type Change = fn(&mut LineTexts);
        let cases: [(&str, Change); 7] = [
            ("value_date", |t| t.value_date = None),
            ("reference", |t| t.reference = Some("Autre".into())),
            ("end_to_end_id", |t| t.end_to_end_id = None),
            ("transaction_id", |t| t.transaction_id = None),
            ("counterparty_iban", |t| t.counterparty_iban = None),
            ("counterparty_name", |t| t.counterparty_name = None),
            ("details", |t| t.details = "autre".into()),
        ];
        let new = new_line();
        for (column, change) in cases {
            let mut stored = texts();
            change(&mut stored);
            let existing = vec![existing_row(42, "Co", stored)];
            let r =
                detect_duplicate_lines_with_aliases(&new, 17, &existing, &[alias("Co", texts())]);
            assert!(r.duplicates.is_empty(), "{column} : {r:?}");
            assert_eq!(
                r.uncertain,
                vec![UncertainLegacyLine {
                    new_index: 0,
                    existing_transaction_id: 42
                }],
                "{column}"
            );
        }
    }

    /// AC-S3 — une clé portée par plusieurs lignes : celle qui a les mêmes textes est
    /// désignée, où qu'elle soit dans la lecture.
    #[test]
    fn alias_finds_the_row_with_equal_texts_915() {
        let other = LineTexts {
            details: "autre".into(),
            ..texts()
        };
        let existing = vec![
            existing_row(50, "Co", other.clone()),
            existing_row(60, "Co", texts()),
            existing_row(40, "Co", other),
        ];
        let r = detect_duplicate_lines_with_aliases(
            &new_line(),
            17,
            &existing,
            &[alias("Co", texts())],
        );
        assert_eq!(r.duplicates[0].existing_transaction_id, 60);
    }

    /// AC-S3 — plusieurs lignes ont les mêmes textes, ou aucune : le plus petit `id`
    /// de la clé, quel que soit l'ordre de lecture (la requête ne trie pas).
    #[test]
    fn smallest_id_when_several_or_none_have_the_texts_915() {
        let existing = vec![
            existing_row(60, "Co", texts()),
            existing_row(40, "Co", texts()),
        ];
        let r = detect_duplicate_lines_with_aliases(
            &new_line(),
            17,
            &existing,
            &[alias("Co", texts())],
        );
        assert_eq!(r.duplicates[0].existing_transaction_id, 40);

        let other = LineTexts {
            details: "lu autrement".into(),
            ..texts()
        };
        let r =
            detect_duplicate_lines_with_aliases(&new_line(), 17, &existing, &[alias("Co", other)]);
        assert!(r.duplicates.is_empty());
        assert_eq!(r.uncertain[0].existing_transaction_id, 40);
    }

    /// AC-S3 (revue P1, L-A1, C-e332-27) — un doublon désigne une ligne qui a vraiment
    /// les mêmes textes : le plus petit `id` parmi elles, non le plus petit de la clé.
    #[test]
    fn duplicate_designates_the_smallest_id_among_equal_texts_915() {
        let other = LineTexts {
            details: "autre".into(),
            ..texts()
        };
        let existing = vec![
            existing_row(40, "Co", other),
            existing_row(60, "Co", texts()),
            existing_row(50, "Co", texts()),
        ];
        let r = detect_duplicate_lines_with_aliases(
            &new_line(),
            17,
            &existing,
            &[alias("Co", texts())],
        );
        assert_eq!(r.duplicates[0].existing_transaction_id, 50);
    }

    /// Un alias qui n'égale aucune clé existante : ligne neuve, ni doublon ni incertaine.
    #[test]
    fn alias_without_a_matching_key_is_nothing_915() {
        let existing = vec![existing_row(42, "autre", texts())];
        let r = detect_duplicate_lines_with_aliases(
            &new_line(),
            17,
            &existing,
            &[alias("Co", texts())],
        );
        assert_eq!(r, DuplicateLines::default());
    }

    /// AC-F4 — un serveur sans mode strict a tronqué le texte à l'écriture : les
    /// 255 premiers caractères stockés égalent les 300 lus ; les 254 premiers, non. Pur :
    /// aucun test ne dépend du `sql_mode`.
    #[test]
    fn stored_text_is_compared_modulo_the_column_width_915() {
        let lu = "é".repeat(300);
        let with_name = |name: String| LineTexts {
            counterparty_name: Some(name),
            ..texts()
        };
        let read = alias("Co", with_name(lu.clone()));
        let run = |stored: LineTexts| {
            detect_duplicate_lines_with_aliases(
                &new_line(),
                17,
                &[existing_row(42, "Co", stored)],
                std::slice::from_ref(&read),
            )
        };
        let r = run(with_name("é".repeat(255)));
        assert_eq!(r.duplicates.len(), 1, "{r:?}");
        let r = run(with_name("é".repeat(254)));
        assert_eq!(r.uncertain.len(), 1, "{r:?}");
    }

    /// AC-F4 — les largeurs : 255 caractères, 34 pour l'IBAN, 65 535 octets pour le
    /// libellé, à une frontière de caractère ; un texte plus court ne change pas.
    #[test]
    fn legacy_line_cuts_each_text_to_its_column_915() {
        let long = BankTransactionDraft {
            reference: Some("r".repeat(256)),
            end_to_end_id: Some("e".repeat(256)),
            transaction_id: Some("t".repeat(256)),
            counterparty_iban: Some("I".repeat(35)),
            counterparty_name: Some("n".repeat(256)),
            // 2 octets par « é » : le 65 535e octet tombe au milieu d'un caractère.
            details: "é".repeat(40_000),
            ..draft(date(2026, 10, 10), dec!(10.00), None, None, None)
        };
        let cut = LegacyLine::from_draft(&long, 17).texts;
        assert_eq!(cut.reference.unwrap().chars().count(), 255);
        assert_eq!(cut.end_to_end_id.unwrap().chars().count(), 255);
        assert_eq!(cut.transaction_id.unwrap().chars().count(), 255);
        assert_eq!(cut.counterparty_iban.unwrap().chars().count(), 34);
        assert_eq!(cut.counterparty_name.unwrap().chars().count(), 255);
        assert_eq!(cut.details.len(), 65_534);

        // Exactement la largeur : rien ne change.
        let exact = BankTransactionDraft {
            reference: Some("r".repeat(255)),
            counterparty_iban: Some("I".repeat(34)),
            details: "d".repeat(65_535),
            ..draft(date(2026, 10, 10), dec!(10.00), None, None, None)
        };
        let kept = LegacyLine::from_draft(&exact, 17).texts;
        assert_eq!(kept, LineTexts::from_draft(&exact));
    }

    /// Revue P1 (E-3, B2) — la clé de l'alias se calcule sur les textes coupés : un
    /// identifiant de plus de 255 caractères, tronqué à l'écriture par un serveur sans
    /// mode strict, reste reconnu, qu'il soit la référence ou un de ses replis.
    #[test]
    fn alias_key_comes_from_the_cut_texts_915() {
        let cases: [fn(String) -> BankTransactionDraft; 3] = [
            |s| BankTransactionDraft {
                reference: Some(s),
                ..draft(date(2026, 10, 10), dec!(10.00), None, None, None)
            },
            |s| BankTransactionDraft {
                end_to_end_id: Some(s),
                ..draft(date(2026, 10, 10), dec!(10.00), None, None, None)
            },
            |s| BankTransactionDraft {
                transaction_id: Some(s),
                ..draft(date(2026, 10, 10), dec!(10.00), None, None, None)
            },
        ];
        for build in cases {
            let read = build("x".repeat(300));
            let stored = build("x".repeat(255));
            let existing = vec![(
                42,
                dedup_key_from_draft(&stored, 17),
                LineTexts::from_draft(&stored),
            )];
            let alias = LegacyLine::from_draft(&read, 17);
            let r = detect_duplicate_lines_with_aliases(
                std::slice::from_ref(&read),
                17,
                &existing,
                &[Some(alias)],
            );
            assert_eq!(r.duplicates.len(), 1, "{r:?}");
            assert!(r.uncertain.is_empty(), "{r:?}");
        }
    }

    /// `detect_duplicate_lines` rend le plus petit `id` quel que soit l'ordre d'entrée.
    #[test]
    fn plain_detection_gives_the_smallest_id_915() {
        let new = vec![draft(
            date(2026, 10, 10),
            dec!(10.00),
            Some("A"),
            None,
            None,
        )];
        let r = detect_duplicate_lines(&new, 17, &[(60, key("a")), (40, key("a"))]);
        assert_eq!(r[0].existing_transaction_id, 40);
    }

    #[test]
    fn detect_duplicate_lines_uses_end_to_end_id_when_reference_null() {
        // AC #8 — fallback `coalesce(reference, end_to_end_id, transaction_id, "")`
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            None,
            Some("EID-42"),
            None,
        )];
        let existing = vec![(
            7,
            dedup_key_scalar(
                date(2026, 5, 15),
                dec!(150.00),
                None,
                Some("EID-42"),
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].existing_transaction_id, 7);
        assert!(result[0].key.contains("eid-42"));
    }

    #[test]
    fn detect_duplicate_lines_uses_transaction_id_when_eid_null() {
        // Fallback chain : reference -> end_to_end_id -> transaction_id
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            None,
            None,
            Some("TX-99"),
        )];
        let existing = vec![(
            8,
            dedup_key_scalar(
                date(2026, 5, 15),
                dec!(150.00),
                None,
                None,
                Some("TX-99"),
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert_eq!(result.len(), 1);
        assert!(result[0].key.contains("tx-99"));
    }

    #[test]
    fn detect_duplicate_lines_normalizes_reference_whitespace_and_case() {
        // AC #9 — `"  ABC-123  "` doit matcher `"abc-123"`.
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some("  ABC-123  "),
            None,
            None,
        )];
        let existing = vec![(
            9,
            dedup_key_scalar(
                date(2026, 5, 15),
                dec!(150.00),
                Some("abc-123"),
                None,
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn detect_duplicate_lines_treats_empty_string_as_distinct_from_null() {
        // Note edge case : `coalesce(reference="", end_to_end_id=Some("X"))`
        // — `reference` est filtrée par `!s.trim().is_empty()` donc `""` est
        // ignoré et le fallback prend `end_to_end_id`. Documenter via test.
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some(""),
            Some("X"),
            None,
        )];
        let existing = vec![(
            10,
            dedup_key_scalar(date(2026, 5, 15), dec!(150.00), None, Some("X"), None, 17),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert_eq!(result.len(), 1, "ref vide doit fallback sur end_to_end_id");
    }

    #[test]
    fn detect_duplicate_lines_does_not_filter_by_date() {
        // AC #11 (T2.2#6) — la fenêtre est filtrée côté SQL par
        // `find_in_dedup_window`. Si le caller passe des keys
        // hors-fenêtre, le helper retourne quand même les matchs.
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some("REF-A"),
            None,
            None,
        )];
        // Même clé sauf la date — pas de match (booking_date fait partie de la clé).
        let existing_diff_date = vec![(
            42,
            dedup_key_scalar(
                date(2025, 1, 1),
                dec!(150.00),
                Some("REF-A"),
                None,
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing_diff_date);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn detect_duplicate_lines_finds_match_when_existing_key_in_input() {
        // AC #11 (T2.2#7) — symétrique : keys dans la fenêtre → matchs.
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some("REF-A"),
            None,
            None,
        )];
        let existing_in_window = vec![(
            55,
            dedup_key_scalar(
                date(2026, 5, 15),
                dec!(150.00),
                Some("REF-A"),
                None,
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing_in_window);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].existing_transaction_id, 55);
    }

    #[test]
    fn detect_duplicate_lines_returns_empty_when_no_match() {
        let new = vec![draft(
            date(2026, 5, 15),
            dec!(150.00),
            Some("REF-A"),
            None,
            None,
        )];
        let existing = vec![(
            1,
            dedup_key_scalar(
                date(2026, 5, 16),
                dec!(150.00),
                Some("REF-B"),
                None,
                None,
                17,
            ),
        )];
        let result = detect_duplicate_lines(&new, 17, &existing);
        assert!(result.is_empty());
    }

    #[test]
    fn detect_duplicate_lines_handles_n_to_m_in_o_n_plus_m() {
        // Perf smoke : new=1000, existing=5000, < 50ms via HashSet.
        let mut new = Vec::with_capacity(1000);
        for i in 0..1000 {
            new.push(draft(
                date(2026, 5, 15),
                Decimal::new(i, 2),
                Some(&format!("REF-NEW-{i}")),
                None,
                None,
            ));
        }
        let mut existing = Vec::with_capacity(5000);
        for i in 0..5000 {
            existing.push((
                i + 100_000,
                dedup_key_scalar(
                    date(2026, 5, 15),
                    Decimal::new(i, 2),
                    Some(&format!("REF-EXISTING-{i}")),
                    None,
                    None,
                    17,
                ),
            ));
        }
        // Inject 3 colliding refs at known indexes so we know how many to expect.
        for i in [10_i64, 500, 999] {
            existing.push((
                900_000 + i,
                dedup_key_scalar(
                    date(2026, 5, 15),
                    Decimal::new(i, 2),
                    Some(&format!("REF-NEW-{i}")),
                    None,
                    None,
                    17,
                ),
            ));
        }

        let start = std::time::Instant::now();
        let result = detect_duplicate_lines(&new, 17, &existing);
        let elapsed = start.elapsed();

        assert_eq!(result.len(), 3, "3 collisions injectées attendues");
        assert!(
            elapsed.as_millis() < 200,
            "performance dégradée : {elapsed:?} > 200ms (cible smoke 50ms)"
        );
    }

    #[test]
    fn dedup_key_display_escapes_pipe_in_reference() {
        // L-2 (Pass 2 review) — une référence contenant le séparateur
        // `|` doit être échappée en `\|` dans la forme texte.
        let key = dedup_key_scalar(
            date(2026, 5, 15),
            dec!(150.00),
            Some("REF|PIPE"),
            None,
            None,
            17,
        );
        // normalize_reference_fallback fait trim+lowercase → "ref|pipe"
        // → escape pipe → "ref\\|pipe"
        assert_eq!(key.to_string(), "2026-05-15|150|ref\\|pipe|17");
    }

    #[test]
    fn dedup_key_zero_amount_scale_invariant() {
        // E11 (Pass 3 review) — `Decimal::ZERO` peut prendre plusieurs
        // scales (`Decimal::new(0, 0)` vs `Decimal::new(0, 4)`) ; après
        // `normalize()` les deux doivent produire la même clé Hash + Eq
        // pour ne pas créer de faux négatifs sur les transactions à
        // montant 0 (rare mais possible : compensation, virement à 0
        // CHF pour mise à jour de référence).
        let key_zero_no_scale = dedup_key_scalar(
            date(2026, 5, 15),
            Decimal::new(0, 0),
            Some("REF-A"),
            None,
            None,
            17,
        );
        let key_zero_with_scale = dedup_key_scalar(
            date(2026, 5, 15),
            Decimal::new(0, 4),
            Some("REF-A"),
            None,
            None,
            17,
        );
        assert_eq!(
            key_zero_no_scale, key_zero_with_scale,
            "Decimal::ZERO scale 0 vs scale 4 must compare equal post-normalize"
        );

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        key_zero_no_scale.hash(&mut h1);
        let mut h2 = DefaultHasher::new();
        key_zero_with_scale.hash(&mut h2);
        assert_eq!(
            h1.finish(),
            h2.finish(),
            "Decimal::ZERO scales must hash identically post-normalize"
        );
    }

    #[test]
    fn dedup_key_normalizes_decimal_scale() {
        // M4 (Pass 1 review) — `Decimal::normalize()` strips trailing zeros so
        // 1.50 (scale 2) and 1.5 (scale 1) hash + compare as equal in `DuplicateKey`.
        // Without normalization, sqlx returning a different scale than the parsed
        // file would silently miss the duplicate.
        let key_scale_2 = dedup_key_scalar(
            date(2026, 5, 15),
            Decimal::new(150, 2), // 1.50
            Some("REF-A"),
            None,
            None,
            17,
        );
        let key_scale_1 = dedup_key_scalar(
            date(2026, 5, 15),
            Decimal::new(15, 1), // 1.5
            Some("REF-A"),
            None,
            None,
            17,
        );
        assert_eq!(key_scale_2, key_scale_1, "Eq must hold across scales");

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        key_scale_2.hash(&mut h1);
        let mut h2 = DefaultHasher::new();
        key_scale_1.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish(), "Hash must hold across scales");
    }

    /// Pass 1 review G1 AA-1 : Story 8-2 T5.0.b — vérifie que le branch
    /// `SourceFormat::Csv { .. }` retourne `Ok(SourceFormatTag::Csv)`
    /// (vs ancien `Err(BankImportUnknownVersion("csv"))` du 8-1a).
    #[test]
    fn from_imported_csv_maps_to_csv_tag() {
        let mut stmt = statement_fixture(None, None);
        stmt.source_format = SourceFormat::Csv {
            encoding: "UTF-8".into(),
            profile_name: Some("UBS".into()),
        };
        let (draft, _) = from_imported(
            &stmt,
            1,
            1,
            String::new(),
            String::new(),
            NaiveDate::from_ymd_opt(2026, 5, 4)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            1,
        )
        .expect("csv branch OK");
        assert_eq!(draft.source_format, SourceFormatTag::Csv);
        assert_eq!(draft.source_format.as_db_str(), "CSV");
    }
}
