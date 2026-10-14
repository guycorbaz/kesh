//! Repository pour la réconciliation bancaire (Story 8-4).
//!
//! Helpers :
//!
//! - [`InvoiceDateWindow`] : **la** fenêtre de dates dans laquelle une
//!   facture peut être réglée par une transaction (Story 28-7, #548) —
//!   seule définition du dépôt, lue par la proposition
//!   ([`find_unpaid_invoices_for_window`]) **et** par l'acceptation
//!   (`accept_one_invoice`, `kesh-api`). Sa borne basse porte sur la
//!   **date d'ancrage** de la facture (Story 29-2, #529) :
//!   [`INVOICE_PAYMENT_ANCHOR_SQL`], la plus tardive de sa date, de son
//!   échéance et de son dernier rappel non annulé, lue par l'acceptation
//!   au moyen de [`invoice_payment_anchor_date`].
//! - [`invoice_exigibility_date`], [`imputation_order`] et
//!   [`INVOICE_EXIGIBILITY_DATE_SQL`] : la **date d'exigibilité** d'une
//!   facture et l'**ordre d'imputation** qui en découle (art. 87 CO ;
//!   Story 29-5, #668) — le départage, à score égal, des candidates d'un
//!   paiement, et l'ordre de la troncature de la proposition.
//! - [`find_unpaid_invoices_for_window`] : charge les factures
//!   candidates pour le matching d'une transaction bancaire — filtre
//!   multi-tenant + [`InvoiceDateWindow`] + reste dû entre la plus petite
//!   valeur que l'acceptation n'estime pas trop-perçue
//!   ([`least_due_without_overpayment`], Story 29-6, #665)
//!   et le montant plus `underpayment_tolerance` (paiement partiel), dans
//!   l'ordre d'imputation (date d'exigibilité,
//!   date, `id` croissants ; Story 29-5), 50 au plus.
//!   **PAS de filtre currency** (S4-1 Pass 4 revert : colonne
//!   `invoices.currency` n'existe pas v0.1 mono-CHF, cf. L38).
//! - [`find_pending_transactions_for_account`] : charge les
//!   transactions `status='pending'` non-rejetées (`auto_match_rejected_at IS NULL`)
//!   pour un compte donné, multi-tenant scoped, paginées (limit
//!   hardcodé 100 au caller, cf. L24).
//! - [`find_contacts_by_ids`] : batch-load Contacts par liste d'IDs
//!   (M5 Pass 1 + MP4-3 Pass 4 — évite N×M queries dans le handler
//!   GET proposals 4-pass architecture).
//! - [`find_by_ids_for_account`] : pré-vol par lot de POST /accept et
//!   POST /reject (1 SELECT IN avant le verrou). **Story 27-4** : sans
//!   filtre de statut ni de rejet (remplace `find_pending_by_ids`, qui
//!   filtrait et faisait refuser tout le lot en 400) — l'appelant classe
//!   chaque ligne et refuse par proposition dans `failed[]`.
//! - [`find_by_id_for_account_for_update`] : relecture **verrouillante**
//!   (lecture courante) d'une transaction par le rejet, quand son
//!   `UPDATE` n'a touché aucune ligne (Story 27-4, AC4), et lecture de
//!   l'annulation d'un rejet, sous verrou, avant son `UPDATE` (Story 28-5a).
//! - [`find_pending_by_id_for_account`] : recharge UNE
//!   `BankTransaction` pending par id (C2 Pass 1 — TOCTOU fix : appelé
//!   inside lock dans `accept_one` pour fermer la fenêtre entre le
//!   pré-flight batch et l'UPDATE).
//!
//! Tous les helpers utilisent l'Executor générique pour accepter
//! `&MySqlPool` (preview) ou `&mut Transaction<MySql>` (handler create
//! dans tx ouverte).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySql;
use std::collections::HashMap;

use crate::entities::bank_transaction::BankTransaction;
use crate::entities::contact::Contact;
use crate::entities::invoice::Invoice;
use crate::errors::{DbError, map_db_error};
use crate::repositories::invoice_settlements::{
    INVOICE_AMOUNT_DUE_DERIVED_SQL, amount_due_derived_joins, least_due_without_overpayment,
};

// Colonnes Invoice pour SELECT — la liste unique de `repositories::invoices`
// (Story 25-6-b).
use super::invoices::INVOICE_COLUMNS;

/// Facture candidate à la réconciliation, accompagnée de son **reste dû** et
/// de son **TTC**.
///
/// - `amount_due` (#420, Story 25-4-c) — le montant réellement comparé à
///   l'encaissement bancaire : `TTC − avoir émis − Σ règlements`. Une facture
///   déjà réglée en partie attend son **solde**, pas son TTC ; comparer le TTC
///   l'excluait des candidates du virement qui la soldait.
/// - `total_ttc` (#246, Story 21-2b) — conservé pour l'**affichage** seulement
///   (« reste dû sur 1 000.00 »), jamais pour comparer. `invoice.total_amount`
///   est le HT et ne matcherait jamais une facture avec TVA.
///
/// Le reste dû est **brut** (jusqu'à 4 décimales) : l'arrondi au centime est
/// la Story 25-4-c3 (#476), pas celle-ci.
///
/// `#[sqlx(flatten)]` : **premier usage du workspace** — `Invoice` dérive
/// `FromRow` et le SELECT liste ses colonnes via `INVOICE_COLUMNS`, donc le
/// flatten hydrate proprement. `pub` (pas `pub(crate)`) : consommé et
/// destructuré par `kesh-api` (crate séparé).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UnpaidInvoiceCandidate {
    #[sqlx(flatten)]
    pub invoice: Invoice,
    pub amount_due: Decimal,
    pub total_ttc: Decimal,
}

/// Colonnes BankTransaction pour SELECT (cohérent bank_transactions::COLUMNS).
const BANK_TX_COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Un paiement règle une facture au plus **30 jours après sa date
/// d'ancrage** — la plus tardive de sa date, de son échéance et de son
/// dernier rappel non annulé ([`INVOICE_PAYMENT_ANCHOR_SQL`]) ; borne de
/// [`InvoiceDateWindow::earliest_anchor_date`].
///
/// Au-delà, le paiement n'est ni proposé ni accepté. Jusqu'à la Story 29-2
/// (#529), les 30 jours couraient de la seule date de la facture : un
/// paiement après l'échéance, ou après un rappel, n'était jamais proposé.
/// Le nom de la constante reste : c'est le `details.window_days` du contrat
/// d'API.
pub const INVOICE_WINDOW_DAYS_AFTER_INVOICE: i64 = 30;

/// La table dérivée du **dernier rappel non annulé** de chaque facture
/// (alias `rmd`, colonne `last_sent_at`), à placer après `FROM invoices i …`
/// (Story 29-2, #529). **Prérequis : alias `i` sur `invoices`.**
///
/// Un rappel **annulé** (`cancelled_at` posé) n'y compte pas : l'annulation
/// d'un Admin retire déjà la ligne du niveau courant des rappels
/// (`entities::invoice_reminder`) ; elle la retire aussi de l'ancre.
///
/// Patron des jointures du reste dû (`invoice_settlements`) : sans paramètre
/// lié ; la jointure par `invoice_id` porte le scope, la facture étant
/// filtrée par `i.company_id = ?`. Alias `rmd`, distinct de `lt`, `cnt`,
/// `cl_ttc` et `st`, avec lesquels il cohabite dans la proposition.
///
/// ⚠️ **Coût, angle mort écrit (C-29-2-17)** : le scope porte la justesse,
/// pas le coût. Sans `company_id` dans le sous-`SELECT`, la dérivée peut
/// agréger les rappels de **toutes** les sociétés, et la proposition la lit
/// une fois par transaction en attente. MariaDB peut pousser la jointure
/// dans la dérivée (`split_materialized`, index
/// `idx_invoice_reminders_invoice`), sans que cela ait été mesuré ici. Sans
/// effet à l'échelle d'une installation à une société ; un filtre par
/// société demanderait un paramètre lié de plus dans les deux lecteurs.
///
/// ⚠️ **Site jumeau** de la notion « dernier rappel non annulé » :
/// `repositories::dunning_eligibility` (`MAX(sent_at) AS last_reminder_at …
/// cancelled_at IS NULL GROUP BY invoice_id`). Qui change la notion
/// d'annulation change les deux ; elles ne sont pas factorisées (C-29-2-12 :
/// les agrégats diffèrent, et celle de l'éligibilité prend un `company_id`
/// lié).
///
/// Macro et non `const` : `concat!` n'accepte que des littéraux, et la
/// requête scalaire d'[`invoice_payment_anchor_date`] s'y compose.
macro_rules! invoice_payment_anchor_join_sql {
    () => {
        "LEFT JOIN (SELECT invoice_id, MAX(sent_at) AS last_sent_at FROM invoice_reminders \
         WHERE cancelled_at IS NULL GROUP BY invoice_id) rmd ON rmd.invoice_id = i.id"
    };
}

/// La **date d'exigibilité** d'une facture, en SQL : `X = max(F, E)`, `F` sa
/// date, `E` son échéance, `E` absente valant `F` (Story 29-5, #668).
/// **Prérequis : alias `i` sur `invoices`.** Seconde écriture de
/// [`invoice_exigibility_date`], tenue d'accord avec elle par le test
/// `exigibility_date_sql_and_rust_agree` (`tests/reconciliation_repository.rs`).
///
/// ⛔ **`COALESCE` obligatoire** : en MariaDB, `GREATEST` rend `NULL` dès
/// qu'un argument l'est, et un `NULL` trié en `ASC` passe **en tête** — une
/// facture sans échéance passerait devant toutes les autres (ou, dans
/// l'ancre, sortirait des candidates, `NULL >= ?` étant faux). Le `GREATEST`
/// sur `i.date` couvre une échéance antérieure à la date (données d'avant
/// #245). Le `CAST(… AS DATE)` fait du type rendu une garantie : il est
/// décodé en `NaiveDate`.
///
/// Macro et non `const` seule : `concat!` n'accepte que des littéraux, et la
/// macro d'ancre de la Story 29-2 la compose (`invoice_payment_anchor_sql!`,
/// premier terme réservé à cet usage, C-29-2-15). Elle est donc définie
/// **avant** elle : une `macro_rules!` n'est visible qu'après sa définition
/// dans le texte.
macro_rules! invoice_exigibility_date_sql {
    () => {
        "CAST(GREATEST(i.date, COALESCE(i.due_date, i.date)) AS DATE)"
    };
}

/// `invoice_exigibility_date_sql!` en `const`, pour les requêtes composées
/// par `format!` (l'`ORDER BY` de [`find_unpaid_invoices_for_window`]).
pub const INVOICE_EXIGIBILITY_DATE_SQL: &str = invoice_exigibility_date_sql!();

/// La **date d'ancrage** d'une facture, en SQL : `A = max(F, E, R)`, `F`
/// sa date, `E` son échéance, `R` le jour de son dernier rappel non annulé,
/// `E` et `R` absents valant `F` (Story 29-2, #529). **Prérequis : alias `i`
/// sur `invoices` et la jointure [`INVOICE_PAYMENT_ANCHOR_JOIN_SQL`].**
///
/// Elle se compose en deux termes, `GREATEST(<F, E>, <R>)` : le premier est
/// la date à laquelle la facture devient **exigible**,
/// [`INVOICE_EXIGIBILITY_DATE_SQL`] — composée ici par sa macro depuis la
/// Story 29-5 (#668), à la place de `GREATEST(i.date, COALESCE(i.due_date,
/// i.date))` qui y était écrit (C-29-2-15) : le `CAST(… AS DATE)` qu'elle
/// ajoute est l'identité sur une `DATE` jamais `NULL` (`i.date` est
/// `NOT NULL`), si bien que l'ancre ne change pas.
///
/// ⛔ **Chaque argument porte son `COALESCE`** : en MariaDB, `GREATEST`
/// rend `NULL` dès qu'un argument est `NULL`, et `NULL >= ?` est faux — une
/// facture sans échéance ou sans rappel disparaîtrait des candidates. Le
/// `GREATEST` sur `i.date` couvre une échéance antérieure à la date
/// (données d'avant #245, la garde de l'API ne portant que sur une paire
/// modifiée). Le repli `i.date` du terme du rappel ne la couvre **pas** à
/// lui seul : il ne vaut `F` que sans rappel. Une échéance **et** un rappel
/// non annulé tous deux antérieurs à la date (rien n'interdit un rappel
/// manuel daté avant la facture) rendraient, sans le `i.date` du premier
/// terme, une ancre antérieure à `F` — la mutation M14, tuée par le cas l
/// de `the_anchor_is_the_latest_of_date_due_date_and_last_reminder`. Le
/// premier terme doit donc valoir l'exigibilité à lui seul.
///
/// `DATE(rmd.last_sent_at)` : `sent_at` est un `DATETIME(6)`, saisi (rappel
/// manuel) ou UTC (rappel envoyé par e-mail) ; l'écart de fuseau d'un jour
/// au plus est un angle mort écrit (C-29-2-3), sans effet mesurable sur une
/// borne de 30 jours. Le `CAST(… AS DATE)` extérieur fait du type rendu une
/// garantie (C-29-2-10) : il est décodé en `NaiveDate`.
///
/// **Deux lecteurs, une expression** : la proposition
/// ([`find_unpaid_invoices_for_window`], dans son `WHERE`) et l'acceptation
/// ([`invoice_payment_anchor_date`], forme scalaire, puis
/// [`InvoiceDateWindow::check`]).
macro_rules! invoice_payment_anchor_sql {
    () => {
        concat!(
            "CAST(GREATEST(",
            invoice_exigibility_date_sql!(),
            ", ",
            "COALESCE(DATE(rmd.last_sent_at), i.date)",
            ") AS DATE)"
        )
    };
}

/// `invoice_payment_anchor_join_sql!` en `const`, pour les requêtes
/// composées par `format!`.
pub const INVOICE_PAYMENT_ANCHOR_JOIN_SQL: &str = invoice_payment_anchor_join_sql!();

/// `invoice_payment_anchor_sql!` en `const`, pour les requêtes composées
/// par `format!`.
pub const INVOICE_PAYMENT_ANCHOR_SQL: &str = invoice_payment_anchor_sql!();

/// La requête scalaire d'[`invoice_payment_anchor_date`], assemblée à la
/// compilation sur les mêmes macros que la proposition.
const INVOICE_PAYMENT_ANCHOR_SCALAR_SQL: &str = concat!(
    "SELECT ",
    invoice_payment_anchor_sql!(),
    " FROM invoices i ",
    invoice_payment_anchor_join_sql!(),
    " WHERE i.company_id = ? AND i.id = ?"
);

/// Un paiement peut précéder la facture qu'il règle d'**un jour** au plus
/// (borne de [`InvoiceDateWindow::latest_invoice_date`]).
///
/// La date de paiement est une date bancaire, la date de facture une date
/// métier locale : l'écart de fuseau suffit à faire paraître un paiement
/// « la veille ». Un paiement antérieur de plus d'un jour — un acompte, par
/// exemple — ne règle pas cette facture : Kesh n'a pas de modèle d'acompte,
/// il se rapproche à la main.
pub const PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS: i64 = 1;

/// La date de paiement d'une transaction bancaire : sa **date de valeur**,
/// à défaut sa date de comptabilisation.
///
/// C'est la date que lisent la fenêtre ([`InvoiceDateWindow::for_transaction`])
/// et le règlement écrit à l'acceptation (`paid_at`, `settled_on`, date de
/// l'écriture d'encaissement).
pub fn payment_date(tx: &BankTransaction) -> NaiveDate {
    tx.value_date.unwrap_or(tx.booking_date)
}

/// La **date d'exigibilité** d'une facture : `X = max(F, E)`, `F` sa date,
/// `E` son échéance, `E` absente valant `F` (Story 29-5, #668).
///
/// **Le critère légal.** Faute de déclaration du débiteur (art. 86 CO), le
/// paiement s'impute — en substance, art. 87 CO (RS 220), texte cité d'après
/// un résumé, Fedlex n'ayant pas été relu à la source — sur la dette
/// **exigible** ; entre plusieurs, sur celle qui a fait l'objet des
/// premières poursuites, à défaut sur la dette **échue la première**
/// (al. 1) ; entre dettes échues en même temps, proportionnellement
/// (al. 2) ; si aucune n'est échue, sur la moins garantie (al. 3).
///
/// **La facture sans échéance** est exigible dès sa date — en substance,
/// art. 75 CO pour une obligation sans terme —, et Kesh la tient aussi pour
/// **échue** à cette date : c'est une **convention** (C-29-5-12), non une
/// lecture du texte. Conséquence : une facture sans échéance datée au plus
/// tard du jour du paiement compte toujours pour échue, et l'al. 3 ne la vise
/// jamais. Le `max` couvre une échéance antérieure à la date (données
/// d'avant #245) : `X = F`.
///
/// ⛔ Jamais une comparaison d'`Option<NaiveDate>` : `None < Some(_)` en Rust
/// placerait une facture **sans** échéance avant toutes les autres.
///
/// **Deux écritures** : celle-ci, et [`INVOICE_EXIGIBILITY_DATE_SQL`] pour la
/// requête ; le test `exigibility_date_sql_and_rust_agree`
/// (`tests/reconciliation_repository.rs`) les tient d'accord.
pub fn invoice_exigibility_date(invoice: &Invoice) -> NaiveDate {
    invoice.date.max(invoice.due_date.unwrap_or(invoice.date))
}

/// L'**ordre d'imputation** de deux factures candidates d'un même paiement :
/// [date d'exigibilité](invoice_exigibility_date) `X` croissante, puis date
/// `F` croissante, puis identifiant croissant (Story 29-5, #668 ; art. 87 CO).
///
/// **Pourquoi cela suffit à « d'abord les échues »** : pour un paiement de
/// date `D`, une facture est échue si `X ≤ D`, non échue si `X > D` ; toute
/// facture échue a donc une `X` plus petite que toute facture non échue.
/// Trier par `X` place les échues en tête **sans lire `D`**, puis, parmi
/// elles, celle échue la première (al. 1). ⚠️ Cette équivalence ne vaut que
/// tant que l'ordre des **non échues** est lui aussi `X` croissante : qui les
/// ordonnera un jour autrement (garanties, al. 3) devra réintroduire la date
/// du paiement dans la clé.
///
/// **Choix par défaut, écrits** : parmi les non échues, la plus proche de
/// son échéance d'abord — l'al. 3 renvoie aux garanties, que Kesh ignore
/// (C-29-5-5) ; à échéance égale, la plus ancienne, puis l'identifiant — Kesh
/// propose **une** facture et n'impute pas proportionnellement (al. 2) ;
/// l'identifiant rend l'ordre total et déterministe.
///
/// **Ce que l'ordre ne fait pas** :
/// - **il ne touche pas au score**, qui reste la première clé de
///   `propose_matches` (`kesh-reconciliation`). Une facture désignée par la
///   référence **et** au montant exact passe devant toute facture que la
///   référence ne désigne pas aussi (art. 86 : la déclaration du débiteur
///   prime). Le montant pesant plus que la référence (0,50 contre 0,40), une
///   facture au montant exact **du même client** passe devant une facture
///   désignée dont le montant diffère de quelques centimes ; celle d'un
///   **autre** client **peut** y passer — leurs scores peuvent être égaux
///   (0,40 + 0,10 contre 0,50), et c'est alors cet ordre qui tranche ;
/// - il ne vaut, en droit, qu'entre dettes d'un **même débiteur** : entre
///   factures de clients différents que rien ne distingue, c'est un ordre par
///   défaut (C-29-5-7) — la proposition reste une proposition ;
/// - il n'impute pas proportionnellement (al. 2), ne lit pas les
///   **poursuites** (Kesh n'en enregistre aucune) ni les **rappels**, qui
///   n'en sont pas (C-29-5-3).
pub fn imputation_order(a: &Invoice, b: &Invoice) -> std::cmp::Ordering {
    invoice_exigibility_date(a)
        .cmp(&invoice_exigibility_date(b))
        .then_with(|| a.date.cmp(&b.date))
        .then_with(|| a.id.cmp(&b.id))
}

/// La fenêtre de dates dans laquelle une transaction peut régler une facture
/// (Story 28-7, #548 ; Story 29-2, #529) : `F ≤ D + 1 jour` **et**
/// `A ≥ D − 30 jours`, `D` étant la [date de paiement](payment_date), `F` la
/// date de la facture et `A` sa **date d'ancrage**
/// ([`INVOICE_PAYMENT_ANCHOR_SQL`] : la plus tardive de `F`, de l'échéance et
/// du dernier rappel non annulé), bornes incluses. Une facture sans échéance
/// ni rappel (`A = F`) garde la fenêtre `D − 30 ≤ F ≤ D + 1` de la 28-7.
///
/// ⛔ **La borne haute reste sur `F`**, jamais sur `A` : la porter sur l'ancre
/// ferait disparaître toute facture non échue (paiement avant l'échéance).
///
/// **Une seule définition, deux lecteurs.** La proposition
/// ([`find_unpaid_invoices_for_window`], appelée par `get_proposals`) et
/// l'acceptation (`accept_one_invoice`, `kesh-api`) la lisent toutes deux :
/// une facture proposée est une facture que l'acceptation admet, et
/// réciproquement. Avant la Story 28-7, la proposition cherchait
/// `± 30 jours` autour de la date de comptabilisation, et proposait des
/// factures que l'acceptation refusait ensuite.
///
/// **Pourquoi l'acceptation fait foi** (C-28-7-1) : admettre un paiement
/// antérieur à sa facture changerait ce que Kesh écrit au journal — un
/// règlement daté avant la créance —, romprait l'accord avec le règlement
/// saisi depuis la fiche de la facture, et se heurterait à la base.
///
/// **Sites qui bornent la date de facture par rapport à une date de
/// paiement sans lire cette fenêtre** — la tolérance d'un jour de la borne
/// haute, `paiement ≥ F − 1` ; non résolus, assumés (C-28-7-7) :
/// - `settle_invoice` et `write_off_invoice`
///   (`repositories::invoice_settlements_write`, le règlement et le solde par
///   escompte, perte, frais ou arrondi) : même tolérance d'un jour sur une
///   date **saisie**, sans borne haute — ils ne règlent pas une transaction ;
/// - la contrainte `chk_invoices_paid_at_after_date` (migration
///   `20260419000001_invoice_paid_at.sql`, copiée dans le squash de
///   `test-schema`) : `DATE(paid_at) >= date - INTERVAL 1 DAY`. La base
///   double la tolérance d'un jour, qui ne se desserre que par une migration.
///
/// Aucune borne n'est ramenée à la plage d'une colonne `DATE` : `check`
/// compare exactement ce que comparait l'acceptation. Une transaction datée
/// aux extrémités de cette plage est un angle mort écrit (C-28-7-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvoiceDateWindow {
    /// Date d'ancrage la plus ancienne admise : `D − 30 jours` (Story 29-2 ;
    /// `earliest_invoice_date` jusque-là, quand la borne portait sur `F`).
    pub earliest_anchor_date: NaiveDate,
    /// Date de facture la plus récente admise : `D + 1 jour`.
    pub latest_invoice_date: NaiveDate,
}

/// Pourquoi une facture sort de l'[`InvoiceDateWindow`] d'une transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceDateRefusal {
    /// La facture est datée de plus d'un jour **après** le paiement : le
    /// paiement la précède (raison d'API `payment_date_before_invoice_date`).
    PaymentBeforeInvoice,
    /// Le paiement arrive plus de 30 jours après la date d'ancrage de la
    /// facture — la plus tardive de sa date, de son échéance et de son
    /// dernier rappel non annulé (raison d'API `payment_date_outside_window`).
    PaymentTooLate,
}

impl InvoiceDateWindow {
    /// La fenêtre d'une date de paiement `d` : ancre `≥ d − 30`, date de
    /// facture `≤ d + 1`.
    pub fn for_payment_date(d: NaiveDate) -> Self {
        Self {
            earliest_anchor_date: d - chrono::Duration::days(INVOICE_WINDOW_DAYS_AFTER_INVOICE),
            latest_invoice_date: d + chrono::Duration::days(PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS),
        }
    }

    /// La fenêtre d'une transaction, sur sa [date de paiement](payment_date).
    pub fn for_transaction(tx: &BankTransaction) -> Self {
        Self::for_payment_date(payment_date(tx))
    }

    /// Une facture datée `invoice_date`, de date d'ancrage `anchor_date`
    /// (lue par [`invoice_payment_anchor_date`]), est-elle dans la fenêtre ?
    ///
    /// # Errors
    ///
    /// Dans cet ordre :
    /// - [`InvoiceDateRefusal::PaymentBeforeInvoice`] si
    ///   `invoice_date > latest_invoice_date` ;
    /// - [`InvoiceDateRefusal::PaymentTooLate`] si
    ///   `anchor_date < earliest_anchor_date`.
    pub fn check(
        &self,
        invoice_date: NaiveDate,
        anchor_date: NaiveDate,
    ) -> Result<(), InvoiceDateRefusal> {
        if invoice_date > self.latest_invoice_date {
            return Err(InvoiceDateRefusal::PaymentBeforeInvoice);
        }
        if anchor_date < self.earliest_anchor_date {
            return Err(InvoiceDateRefusal::PaymentTooLate);
        }
        Ok(())
    }
}

/// La **date d'ancrage** d'une facture ([`INVOICE_PAYMENT_ANCHOR_SQL`]),
/// sous forme scalaire — la lecture de l'acceptation (`accept_one_invoice`,
/// `kesh-api`), qui la compare ensuite par [`InvoiceDateWindow::check`]
/// (Story 29-2, #529).
///
/// Même expression, même jointure que la proposition
/// ([`find_unpaid_invoices_for_window`]) : aucune seconde écriture de la
/// règle. Scopée société (KF-002) : `Ok(None)` si la facture n'existe pas
/// dans `company_id`.
///
/// `SELECT` ordinaire, **sans** `FOR UPDATE` : ne prend aucun verrou.
pub async fn invoice_payment_anchor_date<'e, E>(
    executor: E,
    company_id: i64,
    invoice_id: i64,
) -> Result<Option<NaiveDate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_scalar::<_, NaiveDate>(INVOICE_PAYMENT_ANCHOR_SCALAR_SQL)
        .bind(company_id)
        .bind(invoice_id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Charge les factures candidates pour la réconciliation d'une
/// transaction bancaire (Story 8-4 §candidate-window).
///
/// Filtres :
/// 1. `company_id` (KF-002 Pattern 1, multi-tenant scoping).
/// 2. `status = 'validated' AND paid_at IS NULL AND journal_entry_id IS NOT NULL`
///    (factures éligibles à la réconciliation v0.1).
/// 3. `date <= window.latest_invoice_date AND <ancre> >= window.earliest_anchor_date`
///    — la fenêtre de l'acceptation ([`InvoiceDateWindow`], Story 28-7), dont
///    la borne basse porte sur la date d'ancrage ([`INVOICE_PAYMENT_ANCHOR_SQL`],
///    sur la jointure [`INVOICE_PAYMENT_ANCHOR_JOIN_SQL`] ; Story 29-2, #529).
///    Résultat trié dans l'**ordre d'imputation** ([`imputation_order`] :
///    [`INVOICE_EXIGIBILITY_DATE_SQL`], puis `date`, puis `id`, croissants ;
///    Story 29-5, #668 — `date ASC, id ASC` jusque-là, C-28-7-10) **avant**
///    le `LIMIT 50`, pour que la troncature n'écarte jamais, parmi des
///    candidates de même score, celle que `propose_matches` placerait en
///    tête : les deux ordres coïncident, si bien que les candidates gardées
///    forment un préfixe de l'ordre d'imputation dans chaque classe de score.
///    ⚠️ La troncature reste aveugle au **score** : une facture désignée par
///    sa référence peut encore être écartée au-delà de 50 candidates
///    (C-28-7-10). Ce qui a changé, c'est la victime : au-delà de 50, sont
///    écartées les factures dont l'**exigibilité est la plus lointaine** —
///    par exemple une facture désignée, à 60 jours net, derrière cinquante
///    factures de même reste dû plus tôt exigibles —, et non plus les plus
///    récentes par date. Même fragilité, autre population. Aucun index ne
///    sert cet ordre (tri de quelques factures ouvertes de même montant).
/// 4. **Reste dû** `BETWEEN least_due_without_overpayment(tx_amount) AND
///    tx_amount + underpayment_tolerance` (#420, Story 25-4-c ; borne basse,
///    Story 29-6, #665) — après le HT (#246 : une facture avec TVA ne
///    matchait jamais) puis le TTC (une facture réglée en partie ne matchait
///    jamais son solde). Le reste dû est pris **brut**.
///    - **Borne basse** : le plus petit reste que l'acceptation
///      (`invoice_settlements::classify_payment`) n'estime pas trop-perçu,
///      [`least_due_without_overpayment`] — `tx − 0.005`,
///      l'écart d'arrondi au centime d'un reste à quatre décimales. Un
///      paiement qui dépasse le reste de plus d'un demi-centime est un
///      trop-perçu, refusé à l'acceptation : la proposition ne l'offre plus.
///    - **Borne haute** : `tx + underpayment_tolerance`, un paiement
///      **partiel** de quelques centimes, accepté en `Ordinary` (la facture
///      reste ouverte pour la différence).
///
///    Le filtre est **dans la requête**, avant `ORDER BY … LIMIT 50` : posé
///    après coup, des trop-perçus occuperaient des places du `LIMIT 50` et
///    feraient disparaître une facture valable au-delà (C-29-6-3).
///    ⛔ Pas de `ROUND` ici : la borne basse est calculée en Rust et **liée** ;
///    l'arrondi a UNE définition, `invoice_settlements::amount_due_to_centime`,
///    appliquée par l'appelant au score et à l'affichage (Story 25-4-c3-b).
///
/// ⛔ **Forme jointe** ([`amount_due_derived_joins`] +
/// [`INVOICE_AMOUNT_DUE_DERIVED_SQL`]) : c'est une liste, la forme corrélée y
/// serait réévaluée par ligne — un N+1 déguisé.
///
/// **Pas de filtre currency v0.1** — colonne inexistante (cf. L38 + S4-1 Pass 4).
///
/// **Multi-tenant safety (KF-002 Pattern 1)** : filtrage systématique
/// par `company_id`.
///
/// **Index** : `idx_invoices_company_validated_unpaid_date` créé en
/// migration `20260507100001_reconciliation_8_4.sql` (couvre le `WHERE` ;
/// le filtre du reste dû est appliqué en `HAVING` après retrait des non-éligibles).
pub async fn find_unpaid_invoices_for_window<'e, E>(
    executor: E,
    company_id: i64,
    window: InvoiceDateWindow,
    tx_amount: Decimal,
    underpayment_tolerance: Decimal,
) -> Result<Vec<UnpaidInvoiceCandidate>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    // Alias `i` requis par les tables dérivées du reste dû (`… ON … = i.id`).
    // Elles n'exposent que `invoice_id` et leur agrégat : les colonnes de
    // `INVOICE_COLUMNS` restent sans ambiguïté. Filtre en HAVING pour
    // référencer l'alias `amount_due` sans dupliquer l'expression (SQL
    // n'autorise pas un alias de SELECT dans le WHERE). L'ancre, elle, se lit
    // dans le WHERE : elle ne cite que `i` et la table dérivée `rmd`, aucun
    // alias de SELECT.
    sqlx::query_as::<_, UnpaidInvoiceCandidate>(&format!(
        "SELECT {INVOICE_COLUMNS}, {due} AS amount_due, {ttc} AS total_ttc \
         FROM invoices i {joins} {INVOICE_PAYMENT_ANCHOR_JOIN_SQL} \
         WHERE i.company_id = ? \
           AND i.status = 'validated' \
           AND i.paid_at IS NULL \
           AND i.journal_entry_id IS NOT NULL \
           AND i.date <= ? \
           AND {INVOICE_PAYMENT_ANCHOR_SQL} >= ? \
         HAVING amount_due BETWEEN ? AND ? + ? \
         ORDER BY {INVOICE_EXIGIBILITY_DATE_SQL} ASC, i.date ASC, i.id ASC \
         LIMIT 50",
        due = INVOICE_AMOUNT_DUE_DERIVED_SQL,
        ttc = crate::repositories::invoices::INVOICE_TTC_DERIVED_SQL,
        joins = amount_due_derived_joins(),
    ))
    .bind(company_id)
    .bind(window.latest_invoice_date)
    .bind(window.earliest_anchor_date)
    .bind(least_due_without_overpayment(tx_amount))
    .bind(tx_amount)
    .bind(underpayment_tolerance)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Charge les transactions bancaires `pending` non-rejetées pour un
/// compte donné, scopées multi-tenant, paginées via `limit`.
///
/// Filtres :
/// - `company_id` (KF-002 Pattern 1).
/// - `bank_account_id` (scope local).
/// - `status = 'pending'`.
/// - `auto_match_rejected_at IS NULL` (exclut les transactions
///   rejetées manuellement, réservées pour 8-5 manual).
///
/// Tri par `booking_date DESC, id DESC` pour présenter les
/// transactions récentes d'abord (UX nominal).
pub async fn find_pending_transactions_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    limit: i64,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? \
           AND bank_account_id = ? \
           AND status = 'pending' \
           AND auto_match_rejected_at IS NULL \
         ORDER BY booking_date DESC, id DESC \
         LIMIT ?",
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(limit)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Batch-load Contacts par liste d'IDs (MP4-3 Pass 4 — éviter N×M
/// queries dans le handler GET proposals 4-pass architecture).
/// Retourne une `HashMap<i64, Contact>` keyed par `contact.id`.
///
/// **Multi-tenant safety** : filtre `company_id`.
/// **Soft-delete** (cf. L37) : ne filtre PAS `active = false` v0.1
/// — si un contact est désactivé entre import et reconciliation, le
/// matching produit quand même un contact_score (poids 0.10 donc
/// impact mineur).
pub async fn find_contacts_by_ids<'e, E>(
    executor: E,
    company_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, Contact>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    if ids.len() > MAX_IN_CLAUSE_IDS {
        return Err(DbError::Sqlx(sqlx::Error::Protocol(format!(
            "find_contacts_by_ids: ids.len()={} exceeds MAX_IN_CLAUSE_IDS={MAX_IN_CLAUSE_IDS}",
            ids.len()
        ))));
    }
    // Build IN clause manuellement avec placeholders.
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {cols} FROM contacts \
         WHERE company_id = ? AND id IN ({placeholders})",
        cols = super::contacts::COLUMNS,
    );
    let mut q = sqlx::query_as::<_, Contact>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|c| (c.id, c)).collect())
}

/// Helper inline pour charger une `Invoice` par id scopée multi-tenant.
/// Utilisé par le handler accept (step 5) — équivalent à
/// `FIND_INVOICE_SCOPED_SQL` d'`invoices.rs:41` mais accessible
/// publiquement.
pub async fn find_invoice_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<Invoice>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Invoice>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Cap maximum de la liste `ids` pour éviter un IN clause unbounded
/// (M9 Pass 1 code review). MariaDB tolère des IN larges mais le
/// chemin nominal de réconciliation traite des batches < 100 ; cap
/// défensif à 500 pour éviter un payload abusif.
///
/// **Publique depuis la Story 27-4** (C-27-4-20) : les routes
/// `POST /reconciliation/accept` et `/reject` refusent en `400` un lot
/// de plus de `MAX_IN_CLAUSE_IDS` identifiants **avant** toute lecture,
/// en lisant cette constante — une seule borne, pas de second littéral.
/// Elle borne aussi [`find_contacts_by_ids`], d'où son nom générique.
pub const MAX_IN_CLAUSE_IDS: usize = 500;

/// Lecture par lot des transactions d'un compte, **sans filtre de
/// statut ni de rejet** (Story 27-4, AC11 — remplace
/// `find_pending_by_ids`).
///
/// Scopée `company_id` **et** `bank_account_id` : un identifiant
/// supprimé, d'une autre société ou d'un autre compte est simplement
/// absent de la `HashMap` rendue, par le **même** chemin — c'est ce qui
/// garantit l'anti-énumération du refus `BANK_TRANSACTION_NOT_FOUND`.
///
/// Pourquoi aucun filtre : la **classification** (éligible, rapprochée,
/// rejetée, introuvable) est faite par l'appelant, une seule fois, par
/// la fonction pure des routes de réconciliation, qui appelle
/// [`super::bank_imports::is_reconciled`]. Filtrer ici ferait de toute
/// transaction non éligible une « introuvable ».
///
/// Appelants : le pré-vol de `post_accept` et celui de `post_reject`
/// (`kesh-api/src/routes/reconciliation.rs`), hors transaction et hors
/// verrou.
///
/// Retourne `Err(DbError::Sqlx(Protocol))` si
/// `ids.len() > MAX_IN_CLAUSE_IDS` (les routes refusent le lot en `400`
/// avant d'arriver ici).
pub async fn find_by_ids_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    if ids.len() > MAX_IN_CLAUSE_IDS {
        return Err(DbError::Sqlx(sqlx::Error::Protocol(format!(
            "find_by_ids_for_account: ids.len()={} exceeds MAX_IN_CLAUSE_IDS={MAX_IN_CLAUSE_IDS}",
            ids.len()
        ))));
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? \
           AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql)
        .bind(company_id)
        .bind(bank_account_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|t| (t.id, t)).collect())
}

/// Recharge UNE `BankTransaction` `status='pending'` (sans filtre
/// `auto_match_rejected_at`) par id pour un compte donné, scopée
/// multi-tenant. Utilisé par `accept_one` (C2 Pass 1 — TOCTOU fix)
/// pour rafraîchir l'état dans le lock après le pré-flight batch
/// hors-lock.
///
/// **Comme [`find_by_ids_for_account`]** (qui a remplacé
/// `find_pending_by_ids` à la Story 27-4), ce helper ne filtre
/// PAS `auto_match_rejected_at IS NULL`. Une transaction rejetée
/// manuellement entre le pré-flight et le lock peut encore être
/// acceptée (pattern « j'ai rejeté trop vite, je rétracte ») — le
/// caller laisse le step 4 status check trancher.
///
/// **Naming caveat (dette `dette-naming-reconciliation-helpers` —
/// F8'' Pass 3 Opus 8-5a-base)** : malgré le nom, ce helper ne
/// filtre PAS `status='pending'` côté SQL. Il appelle juste « par
/// id pour un compte ». Le filtre status est délégué au caller
/// (8-4 `accept_one` step 4). 8-5a-base introduit
/// [`find_strictly_pending_by_id_for_account`] qui, lui, filtre
/// strictement `status='pending'` côté SQL — utilisé par le flow
/// manual où l'on ne veut pas réutiliser une transaction déjà
/// `reconciled`.
pub async fn find_pending_by_id_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&bank_tx_by_id_for_account_sql(""))
        .bind(company_id)
        .bind(bank_account_id)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Texte SQL de la lecture d'une transaction par identifiant, scopée
/// `(company_id, bank_account_id)`, sans filtre de statut — partagé par
/// [`find_pending_by_id_for_account`] (`suffix` vide) et
/// [`find_by_id_for_account_for_update`] (`suffix` = `" FOR UPDATE"`),
/// pour que les deux lectures ne divergent pas (Story 27-4, C-27-4-32).
fn bank_tx_by_id_for_account_sql(suffix: &str) -> String {
    format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND id = ?{suffix}"
    )
}

/// Relit UNE transaction par identifiant, scopée `(company_id,
/// bank_account_id)`, sans filtre de statut, par une **lecture
/// verrouillante** (`FOR UPDATE`) — Story 27-4, AC4 (C-27-4-21).
///
/// **Lecture courante, et c'est tout son objet.** Sous REPEATABLE READ,
/// une `SELECT` ordinaire dans la transaction d'un lot lit la vue
/// cohérente ouverte par la première lecture du lot ; une transaction
/// supprimée (avec son import) **après** cette ouverture y apparaîtrait
/// encore. `FOR UPDATE` lit la dernière version validée et la verrouille
/// jusqu'à la fin de la transaction (verrou d'intervalle si la ligne est
/// absente) : l'appelant sait ce que la ligne est **maintenant**.
///
/// Deux appelants, dans `kesh-api/src/routes/reconciliation.rs` :
///
/// - `reject_batch`, quand l'`UPDATE` du rejet n'a touché aucune ligne — la
///   ligne relue est reclassée (introuvable, rapprochée, rejetée). Le verrou
///   porte sur une ligne que l'`UPDATE` qui précède visait déjà : même ordre,
///   aucun cycle neuf ;
/// - `unreject_once` (Story 28-5a, #526), qui lit la ligne sous le verrou du
///   compte **avant** son `UPDATE` et la classe (`classify_unreject`) : une
///   transaction supprimée avec son import pendant l'attente est vue absente
///   (`404`), là où la vue cohérente la verrait encore.
///
/// ⚠️ Appelée **hors** transaction (autocommit), elle ne verrouille rien
/// au-delà de l'instruction : réservée à un exécuteur transactionnel.
pub async fn find_by_id_for_account_for_update<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&bank_tx_by_id_for_account_sql(" FOR UPDATE"))
        .bind(company_id)
        .bind(bank_account_id)
        .bind(id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Charge UNE `BankTransaction` **strictement** `status='pending'` par
/// id, scopée multi-tenant `(company_id, bank_account_id, id)`. Utilisé
/// par le flow `/manual` (8-5a-base) et `/split` (8-5a-bis) pour
/// pré-flight ownership AVANT lock ET inside lock.
///
/// Retourne `None` si introuvable, status ≠ 'pending', cross-tenant,
/// ou cross-account. Couvre en un seul code 4 cas — le caller mappe
/// vers `RECONCILIATION_TRANSACTION_NOT_PENDING` 404.
///
/// **Distinct de `find_pending_by_id_for_account` (8-4)** qui ne
/// filtre PAS `status` (cf. `dette-naming-reconciliation-helpers`).
pub async fn find_strictly_pending_by_id_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    id: i64,
) -> Result<Option<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND id = ? \
           AND status = 'pending'"
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    //! Tests purs de la fenêtre de dates (Story 28-7, AC6) — aucune base.

    use super::*;
    use crate::entities::bank_transaction::BankTransactionStatus;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn days(n: i64) -> chrono::Duration {
        chrono::Duration::days(n)
    }

    fn tx(booking_date: NaiveDate, value_date: Option<NaiveDate>) -> BankTransaction {
        let at = date(2026, 6, 1).and_hms_opt(0, 0, 0).unwrap();
        BankTransaction {
            id: 1,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date,
            value_date,
            amount: Decimal::new(10_000, 2),
            currency: "CHF".into(),
            reference: None,
            details: String::new(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn the_window_of_a_payment_date_is_thirty_days_before_to_one_day_after() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        assert_eq!(w.earliest_anchor_date, d - days(30));
        assert_eq!(w.latest_invoice_date, d + days(1));
    }

    /// Story 29-2 (#529) — `check(F, A)` : la borne haute porte sur la date de
    /// facture `F`, la borne basse sur la date d'ancrage `A`. Une facture sans
    /// échéance ni rappel (`A = F`) garde la fenêtre de la Story 28-7.
    #[test]
    fn check_admits_the_bounds_and_refuses_one_day_beyond_each() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        // A = F : la fenêtre d'avant la story, inchangée.
        for (k, expected) in [
            (1, Ok(())),
            (2, Err(InvoiceDateRefusal::PaymentBeforeInvoice)),
            (0, Ok(())),
            (-1, Ok(())),
            (-30, Ok(())),
            (-31, Err(InvoiceDateRefusal::PaymentTooLate)),
        ] {
            assert_eq!(
                w.check(d + days(k), d + days(k)),
                expected,
                "F = A = D{k:+}"
            );
        }
        // L'ancre seule porte la borne basse.
        assert_eq!(w.check(d - days(45), d - days(30)), Ok(()), "A = D − 30");
        assert_eq!(
            w.check(d - days(45), d - days(31)),
            Err(InvoiceDateRefusal::PaymentTooLate),
            "A = D − 31"
        );
        // La date de facture seule porte la borne haute, ancre dans la fenêtre.
        assert_eq!(w.check(d + days(1), d + days(31)), Ok(()), "F = D + 1");
        assert_eq!(
            w.check(d + days(2), d + days(32)),
            Err(InvoiceDateRefusal::PaymentBeforeInvoice),
            "F = D + 2, A dans la fenêtre"
        );
    }

    /// L'ordre des refus : `PaymentBeforeInvoice` d'abord, même quand l'ancre
    /// est aussi hors de la fenêtre (impossible si `A ≥ F`, mais `check` est
    /// public) — mutation M8.
    #[test]
    fn check_refuses_payment_before_invoice_first() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        assert_eq!(
            w.check(d + days(2), d - days(31)),
            Err(InvoiceDateRefusal::PaymentBeforeInvoice)
        );
    }

    /// Une facture non échue : l'ancre au-delà de `D + 1` (échéance future)
    /// ne déclenche pas `PaymentBeforeInvoice` — la borne haute ne lit que la
    /// date de facture (mutation M10b).
    #[test]
    fn check_admits_an_invoice_not_yet_due() {
        let d = date(2026, 6, 15);
        let w = InvoiceDateWindow::for_payment_date(d);
        assert_eq!(w.check(d - days(10), d + days(20)), Ok(()), "cas j");
        assert_eq!(w.check(d - days(45), d + days(15)), Ok(()), "cas k");
    }

    #[test]
    fn the_payment_date_is_the_value_date_when_there_is_one() {
        let booking = date(2026, 6, 15);
        let value = date(2026, 6, 10);
        let t = tx(booking, Some(value));
        assert_eq!(payment_date(&t), value);
        assert_eq!(
            InvoiceDateWindow::for_transaction(&t),
            InvoiceDateWindow::for_payment_date(value)
        );
    }

    #[test]
    fn the_payment_date_falls_back_to_the_booking_date() {
        let booking = date(2026, 6, 15);
        let t = tx(booking, None);
        assert_eq!(payment_date(&t), booking);
        assert_eq!(
            InvoiceDateWindow::for_transaction(&t),
            InvoiceDateWindow::for_payment_date(booking)
        );
    }

    // -----------------------------------------------------------------------
    // Story 29-5 (#668) — date d'exigibilité et ordre d'imputation
    // -----------------------------------------------------------------------

    /// Une facture validée de date `f` et d'échéance `e`, le reste à défaut.
    fn inv(id: i64, f: NaiveDate, e: Option<NaiveDate>) -> Invoice {
        let at = date(2026, 1, 1).and_hms_opt(0, 0, 0).unwrap();
        Invoice {
            id,
            company_id: 1,
            contact_id: 1,
            invoice_number: None,
            status: "validated".into(),
            date: f,
            due_date: e,
            payment_terms: None,
            total_amount: Decimal::new(10_000, 2),
            rounding_amount: Decimal::ZERO,
            journal_entry_id: Some(1),
            paid_at: None,
            emailed_at: None,
            emailed_to: None,
            project_id: None,
            dunning_paused_at: None,
            dunning_paused_note: None,
            pdf_storage_path: None,
            pdf_sha256: None,
            pdf_frozen_at: None,
            pdf_language: None,
            version: 1,
            created_at: at,
            updated_at: at,
        }
    }

    /// AC4 — `X = max(F, E)`, `E` absente valant `F` : les quatre positions
    /// de l'échéance par rapport à la date (mutations M2, M4).
    #[test]
    fn the_exigibility_date_is_the_later_of_date_and_due_date() {
        let f = date(2026, 5, 20);
        assert_eq!(invoice_exigibility_date(&inv(1, f, None)), f, "E nulle");
        assert_eq!(
            invoice_exigibility_date(&inv(1, f, Some(date(2026, 6, 19)))),
            date(2026, 6, 19),
            "E > F"
        );
        assert_eq!(invoice_exigibility_date(&inv(1, f, Some(f))), f, "E = F");
        assert_eq!(
            invoice_exigibility_date(&inv(1, f, Some(date(2026, 5, 10)))),
            f,
            "E < F"
        );
    }

    /// AC4 — les six lignes du tableau d'exemples de la fiche, `D` =
    /// 2026-06-15 : A créée d'abord (plus petit `id`), l'attendu écrit en dur.
    /// La sixième tue M3 (comparaison d'`Option`, `None` en tête).
    #[test]
    fn the_imputation_order_follows_the_examples_of_the_story() {
        use std::cmp::Ordering::{Greater, Less};
        let d = |m: u32, j: u32| date(2026, m, j);
        // (ligne, A, B, A comparée à B)
        let rows = [
            (
                1,
                inv(1, d(5, 26), Some(d(7, 25))),
                inv(2, d(6, 10), Some(d(6, 15))),
                Greater,
            ),
            (
                2,
                inv(1, d(5, 20), Some(d(6, 14))),
                inv(2, d(5, 25), Some(d(6, 5))),
                Greater,
            ),
            (
                3,
                inv(1, d(5, 20), None),
                inv(2, d(5, 25), Some(d(6, 20))),
                Less,
            ),
            (
                4,
                inv(1, d(5, 20), Some(d(6, 10))),
                inv(2, d(5, 25), Some(d(6, 10))),
                Less,
            ),
            (
                5,
                inv(1, d(5, 20), Some(d(5, 10))),
                inv(2, d(5, 15), None),
                Greater,
            ),
            (
                6,
                inv(1, d(6, 1), None),
                inv(2, d(5, 20), Some(d(5, 25))),
                Greater,
            ),
        ];
        for (row, a, b, expected) in rows {
            assert_eq!(imputation_order(&a, &b), expected, "ligne {row} : A / B");
            assert_eq!(
                imputation_order(&b, &a),
                expected.reverse(),
                "ligne {row} : B / A"
            );
        }
    }

    /// AC4 — `X` et `F` égales : l'identifiant départage (mutation M13), et
    /// l'ordre reste total (une facture égale à elle-même).
    #[test]
    fn the_imputation_order_falls_back_to_the_invoice_id() {
        let f = date(2026, 5, 20);
        let e = Some(date(2026, 6, 19));
        assert_eq!(
            imputation_order(&inv(101, f, e), &inv(102, f, e)),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            imputation_order(&inv(102, f, e), &inv(101, f, e)),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            imputation_order(&inv(101, f, e), &inv(101, f, e)),
            std::cmp::Ordering::Equal
        );
    }
}
