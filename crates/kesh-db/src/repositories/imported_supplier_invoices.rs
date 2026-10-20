//! Repository `imported_supplier_invoices` (Story 12.5b — #194).
//!
//! Socle de persistance des factures fournisseurs **importées** (staging). La
//! logique d'orchestration de l'import (verrou de run, réactivation `discarded`,
//! déplacement `failed/`) et la complétion atomique (`create_in_tx`, DC6) vivent
//! en **12-5c** ; ce module n'expose que le CRUD de base, scopé multi-tenant.

use crate::entities::imported_supplier_invoice::{
    ImportedSupplierInvoice, NewImportedSupplierInvoice,
};
use crate::errors::{DbError, map_db_error};
use sqlx::MySqlPool;
use sqlx::{MySql, Transaction};

/// SELECT scopé multi-tenant (anti-IDOR — toujours `AND company_id = ?`).
const FIND_BY_ID_SQL: &str =
    "SELECT * FROM imported_supplier_invoices WHERE id = ? AND company_id = ?";

/// Insère une facture importée en staging (`status = 'to_complete'`).
///
/// # Errors
/// - [`DbError::UniqueConstraintViolation`] si `(company_id, file_hash)` existe
///   déjà (doublon — mappé `DUPLICATE` par le service 12-5c).
/// - [`DbError::DataLengthOrRange`] si un champ QR dépasse la largeur de sa
///   colonne, et [`DbError::CheckConstraintViolation`] si une contrainte CHECK
///   est violée (dont « tout ou rien » des sept colonnes QR). Depuis la Story
///   30-2a, l'import contrôle le QR **avant** l'insertion
///   (`entities::imported_supplier_invoice::check_scanned_fits_columns`) et
///   construit la ligne par `from_scanned` ou `without_qr`, qui respectent la
///   contrainte par construction : ces deux erreurs ne sont plus que des filets.
pub async fn create(
    pool: &MySqlPool,
    new: &NewImportedSupplierInvoice,
) -> Result<ImportedSupplierInvoice, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let created = create_in_tx(&mut tx, new).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(created)
}

/// Variante transaction-aware de [`create`] — Story 25-1b.
///
/// **Pourquoi** : l'entrée d'audit `imported_supplier_invoice.created` doit
/// partager la transaction de l'INSERT. Sans ce variant, la trace se serait
/// écrite dans une transaction distincte, **après** l'autocommit — exactement
/// la fenêtre non atomique que la story ferme sur les treize autres routes.
///
/// Ne commite jamais.
pub async fn create_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: &NewImportedSupplierInvoice,
) -> Result<ImportedSupplierInvoice, DbError> {
    let id = sqlx::query(
        "INSERT INTO imported_supplier_invoices \
         (company_id, file_hash, storage_path, original_filename, mime_type, byte_size, \
          creditor_iban, is_qr_iban, creditor_address_type, creditor_name, creditor_line1, \
          creditor_line2, creditor_postal_code, creditor_town, creditor_country, \
          reference_type, reference_value, amount, currency, unstructured_message, billing_information) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.file_hash)
    .bind(&new.storage_path)
    .bind(&new.original_filename)
    .bind(&new.mime_type)
    .bind(new.byte_size)
    .bind(&new.creditor_iban)
    .bind(new.is_qr_iban)
    .bind(&new.creditor_address_type)
    .bind(&new.creditor_name)
    .bind(&new.creditor_line1)
    .bind(&new.creditor_line2)
    .bind(&new.creditor_postal_code)
    .bind(&new.creditor_town)
    .bind(&new.creditor_country)
    .bind(&new.reference_type)
    .bind(&new.reference_value)
    .bind(new.amount)
    .bind(&new.currency)
    .bind(&new.unstructured_message)
    .bind(&new.billing_information)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .last_insert_id() as i64;

    find_by_id_scoped_in_tx(tx, new.company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Lecture scopée dans une transaction en cours — Story 25-1b.
pub async fn find_by_id_scoped_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(FIND_BY_ID_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Récupère une facture importée par id, scopée à la company (anti-IDOR).
pub async fn find_by_id_scoped(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(FIND_BY_ID_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Recherche par hash de fichier dans une company (idempotence / réactivation 12-5c).
pub async fn find_by_company_hash(
    pool: &MySqlPool,
    company_id: i64,
    file_hash: &str,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices WHERE company_id = ? AND file_hash = ?",
    )
    .bind(company_id)
    .bind(file_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Verrouille (`FOR UPDATE`) et lit une facture importée scopée company, pour la
/// **complétion atomique** (DC6 12-5c) et le **discard** (AC8). Le verrou ligne
/// sérialise deux complétions concurrentes de la même row → pas de double
/// facture. À appeler **dans** la transaction qui fera l'`UPDATE` suivant.
pub async fn find_by_id_scoped_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Résout le document d'une facture fournisseur via son `supplier_invoice_id`
/// (lecture `GET /supplier-invoices/{id}/source-document`, et `sourceDocument`
/// de la réponse de la facture). Scopé company (anti-IDOR). `None` si la facture
/// n'a pas de document (créée sans document, ou document détaché → 404 côté
/// handler).
///
/// Story 30-2c (AC18) :
/// - **exécuteur générique** : le GET de la facture la lit dans l'instantané de
///   [`super::supplier_invoices::get_settlement_view`] (`&mut *tx`), le
///   téléchargement sur le pool ;
/// - **`AND status = 'completed'`** : l'invariant « un document lié est un
///   document traité » devient explicite pour la fiche, l'écriture et la
///   lecture. Le détachement remet le lien à `NULL` en même temps qu'il repasse
///   le document `to_complete` ([`detach_in_tx`]) ; le filtre tient même si un
///   chemin futur oubliait l'une des deux colonnes.
///
/// ⚠️ Lecture **ordinaire** (jamais `FOR UPDATE`) : dans la vue d'instantané, une
/// lecture verrouillante lirait le dernier état validé, hors de l'instantané.
pub async fn find_by_supplier_invoice_id_scoped<'e, E>(
    executor: E,
    company_id: i64,
    supplier_invoice_id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices \
         WHERE supplier_invoice_id = ? AND company_id = ? AND status = 'completed'",
    )
    .bind(supplier_invoice_id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Story 30-2c (#685) — joindre un document après coup, le détacher
// ---------------------------------------------------------------------------

/// Pourquoi un rattachement est refusé (AC19) — refus **métier**, que la route
/// traduit en `404` ou `409` ; une erreur de base reste un [`DbError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachRefusal {
    /// Document inconnu, ou d'une autre société → `404 IMPORTED_INVOICE_NOT_FOUND`.
    DocumentNotFound,
    /// Document qui n'est plus « à traiter » → `409 IMPORT_NOT_PENDING_COMPLETION`.
    DocumentNotPending { current_status: String },
    /// Facture inconnue, ou d'une autre société → `404 NOT_FOUND`.
    InvoiceNotFound,
    /// La facture a déjà un document → `409 SOURCE_DOCUMENT_ALREADY_ATTACHED`.
    AlreadyAttached,
}

/// Nom de l'index d'unicité de `supplier_invoice_id` (migration
/// `20261015000001`), tel que MariaDB le cite dans le message de l'erreur 1062.
const UNIQUE_SUPPLIER_INVOICE_KEY: &str = "uq_imported_supplier_invoice";

/// Joint le document à traiter `imported_id` à la facture fournisseur
/// `supplier_invoice_id`, **dans la transaction fournie**, sans `COMMIT`
/// (Story 30-2c, AC19). Rend la ligne du document lue sous verrou — avant la
/// mise à jour : l'appelant en tire le nom du fichier pour l'audit —, ou le refus.
///
/// Les étapes, dans l'ordre des verrous :
/// 1. `SELECT … FOR UPDATE` du document par sa clé primaire, borné à la société ;
///    statut ≠ `to_complete` → refus ;
/// 2. `SELECT … FOR UPDATE` de la facture par sa clé primaire, bornée à la
///    société ;
/// 3. la **sentinelle de la société** (`companies` `FOR UPDATE`,
///    [`super::bank_accounts::acquire_company_sentinel_lock`]) — bascule écrite
///    de l'AC19, cf. ci-dessous ;
/// 4. [`mark_completed`] — **aucune lecture préalable** d'un document déjà lié
///    (C-30-2-21) : c'est l'unicité de `supplier_invoice_id` qui refuse, et sa
///    violation se traduit en [`AttachRefusal::AlreadyAttached`] ; toute autre
///    erreur de base reste une erreur.
///
/// ⛔ **Pourquoi la sentinelle (C-30-2-46).** Le banc de la fiche (tables
/// jetables) concluait que deux rattachements vers deux factures différentes ne
/// s'attendent pas. Sur le **vrai schéma**, le test
/// `two_attachments_to_two_invoices_are_serialized_by_company` a montré le
/// contraire quand l'index d'unicité ne porte encore aucune clé au-delà.
/// **Mesuré** : sans sentinelle, la seconde attend dans son `UPDATE` le
/// `COMMIT` de la première. **Explication retenue, raisonnée et non mesurée**
/// (aucun test ne force l'interblocage) : le contrôle de doublon de la
/// première insertion poserait un verrou partagé sur l'intervalle qui court
/// jusqu'au supremum, où l'insertion de la seconde attendrait ; deux contrôles
/// de doublon accordés avant les deux insertions s'attendraient l'un l'autre.
/// La sentinelle, prise **après** le document et la facture et **avant**
/// l'`UPDATE`, sérialise les rattachements d'une société avant leur
/// insertion dans l'index : l'attente reste, et deux rattachements ne peuvent
/// plus s'interbloquer entre eux. Prise en tête, elle formait deux cycles (P4
/// de la validation). Elle forme un cycle **connu** avec toute transaction
/// qui règle plusieurs factures fournisseur — l'acceptation d'un lot de
/// rapprochement (Story 30-1b) et la confirmation d'un lot de paiement
/// (`payment_batches::confirm_batch`) : l'une et l'autre tiennent `companies`
/// en partagé après leur première écriture, puis verrouillent la facture
/// suivante (`pay_in_tx_as`). Rare ; les trois routes sont rejouées sur
/// interblocage (C-30-2-46, revue P1 de la 30-2c).
///
/// Deux rattachements vers la même facture se sérialisent sur l'étape 2 ; le
/// second est refusé à l'étape 4.
///
/// Tout statut de facture est accepté (ouverte, payée, annulée) ; aucun champ de
/// la facture n'est modifié (C-30-2-12). Aucune lecture **ordinaire** : aucun
/// instantané ne naît dans cette transaction.
pub async fn attach_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    imported_id: i64,
    supplier_invoice_id: i64,
) -> Result<Result<ImportedSupplierInvoice, AttachRefusal>, DbError> {
    // (1) Le document, sous verrou.
    let Some(staging) = find_by_id_scoped_for_update(tx, company_id, imported_id).await? else {
        return Ok(Err(AttachRefusal::DocumentNotFound));
    };
    if staging.status != "to_complete" {
        return Ok(Err(AttachRefusal::DocumentNotPending {
            current_status: staging.status,
        }));
    }

    // (2) La facture, sous verrou, par sa clé primaire.
    let invoice: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(supplier_invoice_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if invoice.is_none() {
        return Ok(Err(AttachRefusal::InvoiceNotFound));
    }

    // (3) Sentinelle de la société, APRÈS le document et la facture (C-30-2-46).
    super::bank_accounts::acquire_company_sentinel_lock(tx, company_id).await?;

    // (4) Le lien : l'unicité refuse une facture qui a déjà un document.
    match mark_completed(tx, company_id, imported_id, supplier_invoice_id).await {
        Ok(()) => Ok(Ok(staging)),
        Err(DbError::UniqueConstraintViolation(msg))
            if msg.contains(UNIQUE_SUPPLIER_INVOICE_KEY) =>
        {
            Ok(Err(AttachRefusal::AlreadyAttached))
        }
        Err(e) => Err(e),
    }
}

/// Détache le document de la facture fournisseur `supplier_invoice_id`, **dans
/// la transaction fournie**, sans `COMMIT` (Story 30-2c, AC24) : le document
/// repasse `to_complete`, son lien est remis à `NULL`. Rend la ligne du document
/// telle que lue sous verrou (pour l'audit), ou `None` si la facture n'a pas de
/// document — facture inconnue, d'une autre société ou sans document.
///
/// ⛔ **Ne verrouille que la ligne du document**, jamais la facture : un
/// détachement qui demanderait la facture **après** le document formerait un
/// cycle avec [`attach_in_tx`] (document neuf, puis facture, puis entrée
/// d'unicité). Sur le chemin `None`, la lecture verrouillante d'une clé absente
/// pose brièvement un verrou d'intervalle, rendu au `ROLLBACK`. La facture, ses
/// lignes et ses écritures ne sont pas touchées ; aucun fichier n'est supprimé.
///
/// ⚠️ Angle mort assumé : un document **avec QR** détaché peut servir à créer une
/// seconde facture — geste humain explicite, la première n'est ni annulée ni
/// signalée (manuel utilisateur).
pub async fn detach_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    supplier_invoice_id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    let Some(staging) = sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices \
         WHERE supplier_invoice_id = ? AND company_id = ? AND status = 'completed' FOR UPDATE",
    )
    .bind(supplier_invoice_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    else {
        return Ok(None);
    };
    let affected = sqlx::query(
        "UPDATE imported_supplier_invoices \
         SET status = 'to_complete', supplier_invoice_id = NULL \
         WHERE id = ? AND company_id = ? AND status = 'completed'",
    )
    .bind(staging.id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    // 0 ligne alors que la ligne est tenue sous verrou, `completed` : invariant
    // cassé. L'erreur fait annuler la transaction.
    if affected == 0 {
        return Err(DbError::Invariant(
            "detach_in_tx: aucune ligne 'completed' mise à jour".into(),
        ));
    }
    Ok(Some(staging))
}

/// Le document d'une écriture d'achat ou de règlement d'une facture fournisseur
/// (Story 30-2c, AC21, C-30-2-15).
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct EntrySourceDocument {
    pub supplier_invoice_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub original_filename: String,
}

/// Le document de la facture fournisseur dont l'écriture `journal_entry_id` est
/// l'**achat** ou le **règlement**, borné à la société ; `None` si l'écriture
/// n'appartient à aucune facture fournisseur, ou si celle-ci n'a pas de document
/// (`completed`, comme [`find_by_supplier_invoice_id_scoped`]).
///
/// Lecture **ordinaire**, faite par le détail d'une écriture dans sa transaction
/// de lecture (exécuteur générique : `&mut *tx`). Un règlement annulé, comme le
/// règlement d'une facture payée puis annulée, n'est plus rattaché à sa facture
/// (colonnes de règlement vidées) : son écriture ne montre plus le document ;
/// l'écriture d'achat d'une facture annulée le montre toujours (test
/// `cancelled_invoices_take_a_document_and_the_detached_settlement_loses_it`).
pub async fn find_source_document_of_entry<'e, E>(
    executor: E,
    company_id: i64,
    journal_entry_id: i64,
) -> Result<Option<EntrySourceDocument>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, EntrySourceDocument>(
        "SELECT si.id AS supplier_invoice_id, si.supplier_invoice_number, \
                isi.original_filename \
         FROM supplier_invoices si \
         JOIN imported_supplier_invoices isi \
           ON isi.supplier_invoice_id = si.id AND isi.company_id = si.company_id \
          AND isi.status = 'completed' \
         WHERE si.company_id = ? \
           AND (si.purchase_journal_entry_id = ? OR si.settlement_journal_entry_id = ?) \
         ORDER BY si.id LIMIT 1",
    )
    .bind(company_id)
    .bind(journal_entry_id)
    .bind(journal_entry_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Marque une facture importée `completed` et lie la facture fournisseur réelle
/// (AC7 step 7). À appeler **dans la transaction de complétion**, après
/// `supplier_invoices::create_in_tx`. Le `WHERE status='to_complete'` est une
/// défense en profondeur (la garde principale est le `FOR UPDATE` + check amont).
pub async fn mark_completed(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    supplier_invoice_id: i64,
) -> Result<(), DbError> {
    let affected = sqlx::query(
        "UPDATE imported_supplier_invoices \
         SET status = 'completed', supplier_invoice_id = ? \
         WHERE id = ? AND company_id = ? AND status = 'to_complete'",
    )
    .bind(supplier_invoice_id)
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    // 0 ligne = la garde `status='to_complete'` n'a pas matché alors que le caller
    // a déjà créé la facture réelle dans la même tx → invariant cassé (refactor
    // futur, mauvais id). Remonter une erreur fait rollback la tx (pas de facture
    // orpheline sans lien staging). Code-review 12-5c BH3/EC3.
    if affected == 0 {
        return Err(DbError::Invariant(
            "mark_completed: aucune row staging 'to_complete' mise à jour".into(),
        ));
    }
    Ok(())
}

/// Marque une facture importée `discarded` (AC8). À appeler dans une transaction
/// après `find_by_id_scoped_for_update` + check `status == 'to_complete'`.
/// Le fichier archivé est **conservé** (pas de suppression du justificatif v0.4).
pub async fn mark_discarded(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
) -> Result<(), DbError> {
    let affected = sqlx::query(
        "UPDATE imported_supplier_invoices \
         SET status = 'discarded' \
         WHERE id = ? AND company_id = ? AND status = 'to_complete'",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    // 0 ligne = garde statut non matchée (le caller a déjà vérifié `to_complete`
    // sous FOR UPDATE → invariant cassé). Erreur → rollback. Code-review 12-5c BH3/EC3.
    if affected == 0 {
        return Err(DbError::Invariant(
            "mark_discarded: aucune row staging 'to_complete' mise à jour".into(),
        ));
    }
    Ok(())
}

/// Réactive une facture importée `discarded` → `to_complete` (AC3 step 5,
/// court-circuit doublon hash à l'import). **Remet aussi `supplier_invoice_id`
/// à NULL** : une row `discarded` peut porter un pointeur résiduel d'une
/// complétion antérieure ; sans le reset, la réactivation laisserait une
/// référence vers une ancienne facture, écrasée à la prochaine complétion
/// (donnée incohérente). Le `WHERE status='discarded'` rend l'opération
/// idempotente et sans effet sur une row déjà active. Retourne `true` si une
/// row a effectivement été réactivée.
/// ⚠️ Prend la transaction (Story 25-1b) : la réactivation est un **chemin
/// d'entrée** comme la création, et sa trace doit partager sa transaction.
/// Ne commite jamais.
pub async fn reactivate_to_complete(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<bool, DbError> {
    let affected = sqlx::query(
        "UPDATE imported_supplier_invoices \
         SET status = 'to_complete', supplier_invoice_id = NULL \
         WHERE id = ? AND company_id = ? AND status = 'discarded'",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(affected > 0)
}

/// Liste les factures importées d'une company filtrées par statut
/// (use-case principal `to_complete`).
///
/// **Contrat appelant** : `status` DOIT être l'une des valeurs du domaine
/// (`to_complete` | `completed` | `discarded`). Un statut hors-domaine n'est pas
/// rejeté ici (paramètre bindé, pas d'injection) mais retourne une liste vide —
/// la validation du query param HTTP est à la charge de l'endpoint 12-5c
/// (frontière d'entrée), cf. dette déférée code-review.
pub async fn list_by_status(
    pool: &MySqlPool,
    company_id: i64,
    status: &str,
) -> Result<Vec<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices \
         WHERE company_id = ? AND status = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(company_id)
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Story 25-5-a (#386) — lecture exhaustive pour l'export de souveraineté
// ---------------------------------------------------------------------------

/// Toutes les pièces fournisseurs importées d'une société (Story 25-5-a, #386).
///
/// ⚠️ **Tous statuts** — `to_complete` comme `completed` : une pièce reçue fait
/// partie de ce qu'on emporte, qu'elle ait été transformée en facture ou non.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices WHERE company_id = ? ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}
