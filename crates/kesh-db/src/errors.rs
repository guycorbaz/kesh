//! Erreurs de la couche de persistance.

use thiserror::Error;

/// Raison du refus d'un compte de produit référencé par une ligne de facture
/// (Story 16-1a, #152 — décision D3).
///
/// Les quatre critères sont contrôlés à la **saisie** (création / modification
/// du brouillon) et **re-contrôlés au posting** (validation). Deux d'entre eux
/// ne sont couverts par aucune garde existante : `validate_lines_accounts_in_tx`
/// vérifie `active` inconditionnellement mais laisse passer `postable` (le flux
/// facture appelle `create_in_tx` avec `enforce_postable = false`) et ne
/// consulte **jamais** `account_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevenueAccountRejection {
    /// Compte inexistant, ou appartenant à une autre société (anti-IDOR — on
    /// ne distingue pas les deux cas, pour ne pas révéler l'existence d'un id).
    UnknownOrCrossCompany,
    /// Compte archivé (`active = FALSE`).
    Inactive,
    /// `account_type` différent de `Revenue` — un compte peut avoir été retypé
    /// par `accounts::update` après avoir été choisi sur une ligne.
    NotRevenue,
    /// Compte non imputable (`postable = FALSE`) **et** différent du compte de
    /// produit par défaut de la société, qui bénéficie de l'exemption D3-bis.
    NotPostable,
}

/// Un site en défaut lors du contrôle des comptes de produit (Story 16-1a).
///
/// Le contrôle est batché (une requête pour toute la facture, décision D6) et
/// remonte **tous** les sites en défaut à la fois : un compte partagé archivé
/// invalide plusieurs lignes d'un coup, et l'utilisateur doit toutes les voir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRevenueAccount {
    /// Numéro de ligne **1-based**, tel qu'affiché à l'utilisateur.
    ///
    /// `None` désigne le **compte de produit par défaut de la société** :
    /// aucune ligne ne le porte, il ne peut donc pas être nommé par un numéro
    /// (AC8-bis). L'API le nomme explicitement dans le message.
    pub line_number: Option<i32>,
    pub account_id: i64,
    /// Numéro du compte au plan comptable. `None` quand le compte est inconnu
    /// de la société — il n'y a alors rien à afficher.
    pub account_number: Option<String>,
    pub reason: RevenueAccountRejection,
}

/// Motif pour lequel une écriture ne peut **pas** être contre-passée
/// (Story 24-4a, #380).
///
/// ⚠️ **Les causes se CUMULENT** — une écriture peut être possédée par une
/// facture, déjà contre-passée *et* porter un compte archivé. Le champ exposé
/// étant scalaire, la précédence est figée par l'ordre des variantes ci-dessous,
/// sans quoi le test « une cause, un test » deviendrait non déterministe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReversalBlocker {
    /// L'écriture EST une contre-passation. En contre-passer une reviendrait à
    /// réécrire l'original en trois écritures au lieu d'une.
    IsAReversal,
    /// Une contre-passation existe déjà (garantie par `uq_journal_entries_reverses`).
    AlreadyReversed,
    /// Écriture de vente d'une facture client → le chemin est l'**avoir**.
    OwnedByInvoice,
    /// L'avoir EST déjà la contre-passation de la facture.
    OwnedByCreditNote,
    /// Écriture d'achat ou de règlement d'une facture fournisseur → le chemin
    /// est la fiche de la facture : `supplier_invoices::cancel` pour l'achat
    /// (par le socle depuis la Story 25-3-c), `cancel_settlement` pour le
    /// règlement (Story 25-3-a-2) — tous deux contre-passent au titre de la
    /// facture.
    OwnedBySupplierInvoice,
    /// ⛔ Le cas le plus grave : le résiduel se calcule depuis
    /// `invoice_settlements.amount`, que la contre-passation ne toucherait pas —
    /// grand livre et résiduel divergeraient **en silence**. Chemin :
    /// `invoice_settlements_write::cancel_settlement` (Story 25-3-a-1), qui
    /// contre-passe **au titre** du règlement et retire sa ligne dans la même
    /// transaction — la contre-passation directe, elle, reste refusée.
    OwnedBySettlement,
    /// Écriture rapprochée d'une transaction bancaire. Le chemin est le
    /// **dé-rapprochement** (Story 25-3-b, `reconciliation_cancel::cancel_in_tx`,
    /// `POST /reconciliation/transactions/{id}/cancel`), qui défait le lien puis
    /// contre-passe dans la même transaction — la contre-passation directe, elle,
    /// reste refusée : elle laisserait la transaction pointer une écriture annulée.
    MatchedBankTransaction,
    /// Un compte de l'écriture a été **archivé** depuis.
    ///
    /// ⛔ **En DERNIER de la précédence, et c'est voulu** : c'est le seul motif
    /// que l'utilisateur peut lever lui-même (`PUT /accounts/{id}/reactivate`).
    /// Le dire en premier ferait croire qu'une écriture possédée par une facture
    /// deviendrait contre-passable une fois le compte réactivé — elle ne le
    /// serait pas.
    ///
    /// ⚠️ Le refus à l'ÉCRITURE reste un **400** [`DbError::ReversalAccountsArchived`],
    /// qui NOMME les comptes ; ce code-ci sert la LECTURE, pour que l'écran
    /// masque le bouton **avant** le clic (AC 11).
    AccountArchived,
}

impl ReversalBlocker {
    /// Code canonique, jamais une phrase : la traduction se fait à l'écran.
    pub fn code(self) -> &'static str {
        match self {
            Self::IsAReversal => "IS_A_REVERSAL",
            Self::AlreadyReversed => "ALREADY_REVERSED",
            Self::OwnedByInvoice => "OWNED_BY_INVOICE",
            Self::OwnedByCreditNote => "OWNED_BY_CREDIT_NOTE",
            Self::OwnedBySupplierInvoice => "OWNED_BY_SUPPLIER_INVOICE",
            Self::OwnedBySettlement => "OWNED_BY_SETTLEMENT",
            Self::MatchedBankTransaction => "MATCHED_BANK_TRANSACTION",
            Self::AccountArchived => "ACCOUNT_ARCHIVED",
        }
    }
}

/// Pourquoi une écriture ne se **modifie** pas (Story 15-8a, #532) — hors
/// exercices clos (le sien, un postérieur) et verrou de période, qui ont leurs
/// variantes propres ([`DbError::FiscalYearClosed`],
/// [`DbError::LaterFiscalYearClosed`], [`DbError::PeriodLocked`]).
///
/// C'est la **garde d'écriture** : rendue par
/// `journal_entries::modification_guard`, convertie en erreur par
/// `journal_entries::modification_refusal`. La Story 15-8b l'applique aussi à
/// la suppression (`journal_entries::delete_in_tx`, étape 3-ter).
///
/// ⛔ **L'inventaire des propriétaires n'est pas réécrit ici** : `Owned` porte un
/// motif de `reversal_blockers` — jamais `AccountArchived`, qui n'est pas un gel
/// (on remplace le compte, et l'enregistrement le refuse tant qu'il reste).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModificationGuard {
    /// Un motif de `reversal_blockers`, jamais [`ReversalBlocker::AccountArchived`].
    Owned {
        blocker: ReversalBlocker,
        /// Identifiant de la pièce propriétaire (ou de la contre-passation).
        document_id: Option<i64>,
        /// Numéro lisible de la pièce, quand elle en a un.
        document_label: Option<String>,
    },
    /// Le paiement d'une facture fournisseur **annulée**, détaché par
    /// `supplier_invoices::cancel_in_tx` (Story 25-3-c) : plus aucune colonne ne
    /// le référence, seule la trace d'audit `supplier_invoice.cancelled` le
    /// relie encore à sa facture (C-15-8-20, dette #541). Il représente une
    /// sortie de banque réelle : il ne se modifie pas, il se contre-passe.
    DetachedSupplierSettlement {
        supplier_invoice_id: i64,
        /// `supplier_invoices.supplier_invoice_number` est nullable.
        supplier_invoice_number: Option<String>,
    },
}

impl ModificationGuard {
    /// Code canonique exposé : celui du motif de contre-passation, ou
    /// `DETACHED_SUPPLIER_SETTLEMENT`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Owned { blocker, .. } => blocker.code(),
            Self::DetachedSupplierSettlement { .. } => "DETACHED_SUPPLIER_SETTLEMENT",
        }
    }

    /// Identifiant de la pièce (facture fournisseur annulée pour le paiement détaché).
    pub fn document_id(&self) -> Option<i64> {
        match self {
            Self::Owned { document_id, .. } => *document_id,
            Self::DetachedSupplierSettlement {
                supplier_invoice_id,
                ..
            } => Some(*supplier_invoice_id),
        }
    }

    /// Étiquette lisible de la pièce (numéro de facture, d'avoir…), s'il y en a une.
    pub fn label(&self) -> Option<String> {
        match self {
            Self::Owned { document_label, .. } => document_label.clone(),
            Self::DetachedSupplierSettlement {
                supplier_invoice_number,
                ..
            } => supplier_invoice_number.clone(),
        }
    }
}

/// Motif d'**écran** : pourquoi la fiche n'offre pas « Modifier » (Story 15-8a,
/// D8). Rendu par `journal_entries::modification_blocker`, sur l'état
/// **présent** — la nouvelle date, le corps et la version d'un `PUT` restent
/// contrôlés à l'écriture seule.
///
/// ⚠️ Ordre de précédence = celui des refus du `PUT` qui ne dépendent pas du
/// corps : exercice clos, exercice postérieur clos, garde d'écriture, verrou de
/// période (ancienne date).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModificationBlocker {
    /// L'exercice de l'écriture est clôturé.
    FiscalYearClosed,
    /// Un exercice **postérieur** est clôturé : le bilan est cumulatif, il
    /// reprend cette écriture (C-15-8-22).
    LaterFiscalYearClosed { fiscal_year_name: String },
    /// Garde d'écriture (pièce, contre-passation, paiement détaché).
    Guard(ModificationGuard),
    /// La date de l'écriture est dans une période verrouillée.
    PeriodLocked { locked_through: chrono::NaiveDate },
}

impl ModificationBlocker {
    /// Code d'écran — l'une des onze valeurs de `modificationBlockedBy`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::FiscalYearClosed => "FISCAL_YEAR_CLOSED",
            Self::LaterFiscalYearClosed { .. } => "LATER_FISCAL_YEAR_CLOSED",
            Self::Guard(guard) => guard.code(),
            Self::PeriodLocked { .. } => "PERIOD_LOCKED",
        }
    }

    /// Étiquette : nom de l'exercice postérieur clos, numéro de pièce, ou borne
    /// du verrou (`AAAA-MM-JJ`).
    pub fn label(&self) -> Option<String> {
        match self {
            Self::FiscalYearClosed => None,
            Self::LaterFiscalYearClosed { fiscal_year_name } => Some(fiscal_year_name.clone()),
            Self::Guard(guard) => guard.label(),
            Self::PeriodLocked { locked_through } => Some(locked_through.to_string()),
        }
    }
}

/// Ce qui empêche de **dévalider** une facture (Story 25-2-b-1, #440).
///
/// ⛔ **Un code par motif, et jamais le générique.** Les trois gardes que la
/// suppression de #219 portait rendaient toutes `IllegalStateTransition`, dont
/// le message n'est que journalisé : à l'écran, « transition interdite » ne dit
/// **ni** ce qui bloque **ni** quoi faire. Le gabarit suivi est celui de
/// [`ReversalBlocker`], en service depuis la 24-4a.
///
/// ⚠️ **Les causes se CUMULENT** — une facture peut être réglée, créditée *et*
/// envoyée. Le champ exposé étant scalaire, la précédence est figée par l'ordre
/// des variantes ci-dessous, et c'est celle qu'un test vérifie ; sans quoi le
/// motif rendu dépendrait de l'ordre des requêtes.
///
/// ⚠️ **Les trois autres empêchements (exercice clos, contre-passée, période
/// verrouillée) ne sont PAS ici** : ils vivent dans
/// [`super::repositories::journal_entries::delete_in_tx`] et gardent leurs
/// variantes propres — `FiscalYearClosed`, `EntryIsReversed`, `PeriodLocked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnvalidationBlocker {
    /// Un règlement, même **partiel**, pointe la facture.
    ///
    /// ⛔ La garde lit l'**existence d'une ligne** `invoice_settlements` **OU**
    /// `paid_at IS NOT NULL` — les deux, jamais l'une seule. `paid_at` seul rate
    /// le partiel (il n'est posé qu'au résiduel nul) ; la table seule rate les
    /// factures réglées **avant** sa création (`20260827000001`, DDL pur, sans
    /// rattrapage), qui portent `paid_at` sans aucune ligne.
    Settled,
    /// Un avoir référence la facture : il est déjà la contre-passation.
    Credited,
    /// La facture a un historique de rappels : le dévalider effacerait la
    /// preuve du recouvrement (#260).
    HasReminders,
    /// La facture a été **envoyée au client** (`emailed_at`).
    ///
    /// ⛔ **Refus sec, non levable par confirmation** — arbitrage de Guy du
    /// 2026-09-16 : le client détient un document que les livres ne porteraient
    /// plus. Le chemin de correction redevient l'**avoir**.
    Emailed,
    /// L'écriture est **rapprochée** d'une transaction bancaire.
    ///
    /// ⚠️ La FK `bank_transactions.matched_entry_id` est `ON DELETE SET NULL` :
    /// sans cette garde, le lien s'effacerait **en silence** au moment de la
    /// suppression de l'écriture.
    MatchedBankTransaction,
}

impl UnvalidationBlocker {
    /// Code canonique, jamais une phrase — même discipline que
    /// [`ReversalBlocker::code`].
    ///
    /// ⚠️ `MATCHED_BANK_TRANSACTION` **réemploie** le code de `ReversalBlocker` :
    /// c'est le **même état** du monde, et lui donner un second nom ferait deux
    /// vocabulaires pour un fait.
    pub fn code(self) -> &'static str {
        match self {
            Self::Settled => "INVOICE_HAS_SETTLEMENTS",
            Self::Credited => "INVOICE_CREDITED",
            Self::HasReminders => "INVOICE_HAS_REMINDERS",
            Self::Emailed => "INVOICE_EMAILED",
            Self::MatchedBankTransaction => "MATCHED_BANK_TRANSACTION",
        }
    }
}

/// Ce qui empêche d'**annuler un règlement** (Story 25-3-a-1, #414).
///
/// ⛔ **Une tête propre à chaque pièce, une queue commune.** Le rang 1 est
/// propre à la pièce — `InvoiceCredited` pour le client,
/// `SupplierInvoiceNotPaid` pour le fournisseur (Story 25-3-a-2) ; les rangs 2 à 5 s'évaluent sur
/// l'**écriture de règlement**, sans rien savoir de la pièce qui la possède
/// (`settlement_cancellation::settlement_entry_cancel_blocker`), pour que le
/// règlement fournisseur (25-3-a-2) les réutilise tels quels — une seconde
/// précédence pour le même socle divergerait.
///
/// ⚠️ **L'ordre des variantes EST la précédence**, testée paire par paire : le
/// définitif d'abord, puis ce qui se lève. Les rangs 4 et 5 suivent l'ordre
/// **réel** de `reverse_in_tx`, qui contrôle les comptes archivés (étape 3)
/// avant l'exercice du jour (étape 4) : la lecture se règle sur l'écriture.
///
/// ⚠️ **Qui refuse, à l'écriture** : le geste ne refuse lui-même que les rangs
/// 1 et 2 ([`DbError::SettlementNotCancellable`]) ; les rangs 3 à 5 sont
/// refusés par le socle, avec son erreur canonique — c'est ce qui garde le 400
/// qui **nomme** les comptes archivés.
///
/// ⚠️ **L'annulation d'une FACTURE fournisseur** (Story 25-3-c, #454) emprunte
/// cet enum sans en suivre l'ordre de déclaration : sa précédence est
/// **composée par sa fonction de tête**
/// (`supplier_invoices::supplier_invoice_cancel_blocker` — rang 1
/// `SupplierInvoiceCancelled`, la queue sur l'écriture d'ACHAT, puis
/// `SupplierInvoiceInPaymentBatch` en dernier), et c'est cette fonction qu'un
/// test fixe. Refusés par son geste ([`DbError::SupplierInvoiceNotCancellable`]) :
/// les deux têtes et l'exercice clos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementCancelBlocker {
    /// Tête du **dé-rapprochement** (Story 25-3-b, #418), rang 0 : la
    /// transaction bancaire n'est pas `reconciled` — il n'y a pas de
    /// rapprochement à annuler. Coupe court : aucun rang suivant ne s'évalue.
    BankTransactionNotReconciled,
    /// La facture a été **créditée** par un avoir après ce règlement. Le
    /// règlement reste ouvert au compte débiteurs ; il ne se lettre pas à la
    /// main (R5 de la 15-1a-i), son traitement est la 15-1a2 — et il ne
    /// s'annule pas. ⚠️ Le statut `cancelled` d'une facture ne naît en
    /// production que de l'avoir (`credit_notes.rs`).
    ///
    /// ⚠️ **État hérité depuis la Story 25-4-a (#456)** : un avoir est refusé
    /// sur une facture réglée, même en partie. L'état n'est plus produit par
    /// l'application, mais des **données antérieures à la 0.12.1** peuvent le
    /// porter — la 0.12.0 publiée acceptait cet avoir —, qu'elles soient
    /// **restaurées** d'une sauvegarde ou **mises à jour sur place**. D'où ce
    /// motif, gardé.
    InvoiceCredited,
    /// Un **solde** existe sur la facture (Story 25-4-d2a, #384), rang 1 bis :
    /// un règlement qui n'est pas lui-même un solde ne s'annule pas avant lui.
    /// Sans ce motif, annuler un règlement après un escompte rouvrirait la
    /// facture (`paid_at = NULL`) avec l'escompte toujours passé et compté en
    /// « déjà réglé ». Propriété de la **facture**, comme `InvoiceCredited`,
    /// d'où sa place avant la queue sur l'écriture : placé après, il ferait
    /// rouvrir un exercice clos pour rien.
    WriteOffExists,
    /// Tête **fournisseur** (Story 25-3-a-2) : la facture n'est pas `paid` — il
    /// n'y a pas de règlement à annuler. ⚠️ Coupe court par construction : une
    /// facture non `paid` n'a pas d'écriture de règlement, la queue ne s'évalue
    /// pas. Même rang que `InvoiceCredited` : chaque pièce n'a que sa propre tête.
    SupplierInvoiceNotPaid,
    /// Tête de l'**annulation d'une facture fournisseur** (Story 25-3-c), rang
    /// 1 : la facture est déjà `cancelled`. Coupe court.
    SupplierInvoiceCancelled,
    /// L'écriture de règlement est dans un exercice **clos** : un
    /// Administrateur peut le rouvrir (`fiscal_years::reopen`), et c'est le
    /// chemin (arbitrage Q5).
    FiscalYearClosed,
    /// L'écriture de règlement est rapprochée d'une transaction bancaire : le
    /// dé-rapprochement (Story 25-3-b, `reconciliation_cancel`) défait ce lien
    /// d'abord. Pour le dé-rapprochement lui-même, ce rang ne tient que si une
    /// **autre** transaction pointe la même écriture (exemption étroite).
    MatchedBankTransaction,
    /// Un compte de l'écriture de règlement a été archivé depuis.
    AccountArchived,
    /// Aucun exercice **ouvert** ne couvre le jour, où la contre-passation
    /// serait datée.
    NoOpenFiscalYearToday,
    /// Annulation d'une facture fournisseur (Story 25-3-c), **dernier** rang :
    /// la facture est engagée dans un lot de paiement `generated`. Dernier
    /// parce que le lever est le geste le plus lourd — annuler le lot — et
    /// qu'il serait vain avant un exercice clos.
    SupplierInvoiceInPaymentBatch,
}

impl SettlementCancelBlocker {
    /// Code canonique, jamais une phrase.
    ///
    /// ⚠️ **Tous** ces codes réemploient ceux d'états du monde déjà nommés
    /// (`INVOICE_CREDITED` de [`UnvalidationBlocker`], `FISCAL_YEAR_CLOSED`,
    /// `MATCHED_BANK_TRANSACTION`, `ACCOUNT_ARCHIVED`, `FISCAL_YEAR_INVALID`) :
    /// un même fait ne reçoit pas un second nom.
    pub fn code(self) -> &'static str {
        match self {
            Self::BankTransactionNotReconciled => "BANK_TRANSACTION_NOT_RECONCILED",
            Self::InvoiceCredited => "INVOICE_CREDITED",
            Self::WriteOffExists => "INVOICE_WRITTEN_OFF",
            Self::SupplierInvoiceNotPaid => "SUPPLIER_INVOICE_NOT_PAID",
            Self::SupplierInvoiceCancelled => "SUPPLIER_INVOICE_CANCELLED",
            Self::FiscalYearClosed => "FISCAL_YEAR_CLOSED",
            Self::MatchedBankTransaction => "MATCHED_BANK_TRANSACTION",
            Self::AccountArchived => "ACCOUNT_ARCHIVED",
            Self::NoOpenFiscalYearToday => "FISCAL_YEAR_INVALID",
            Self::SupplierInvoiceInPaymentBatch => "SUPPLIER_INVOICE_IN_PAYMENT_BATCH",
        }
    }
}

/// Un compte de l'écriture d'origine, archivé depuis (Story 24-4a, #380).
///
/// Le refus **nomme** le compte à réactiver — un « interdit » sec ne serait pas
/// utilisable. Gabarit : [`RejectedRevenueAccount`] et son `details.rejected[]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchivedAccount {
    pub account_id: i64,
    /// `None` quand le compte est inconnu de la société — rien à afficher.
    pub account_number: Option<String>,
}

/// Un compte refusé parce qu'il n'est **pas imputable** (Story 15-5a, #429).
///
/// Jumeau d'[`ArchivedAccount`], à une différence près : le numéro n'est pas
/// optionnel. La variante [`DbError::AccountsNotPostable`] n'est émise que pour
/// un compte **de la société et actif** — un compte inconnu ou d'une autre
/// société reste [`DbError::InactiveOrInvalidAccounts`] (anti-énumération
/// KF-002) —, si bien que son numéro est toujours connu (choix C16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPostableAccount {
    pub account_id: i64,
    /// Numéro du compte au plan comptable, tel que l'utilisateur le connaît.
    pub account_number: String,
}

/// Liste **triée, dédoublonnée et non vide** des comptes non imputables d'un
/// refus (Story 15-5a, choix C17 et C31).
///
/// Le champ est privé : la seule façon d'en construire une est
/// [`NonPostableAccounts::new`], qui trie par numéro dans l'**ordre
/// lexicographique de la chaîne** (`"1000" < "10000" < "1010" < "2000"` — le
/// numéro est une chaîne, un tri numérique échouerait sur un numéro non
/// numérique), puis par identifiant à numéro égal, et dédoublonne par
/// identifiant. Le message et le détail JSON sont donc déterministes quel que
/// soit l'ordre des lignes de la requête.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonPostableAccounts(Vec<NonPostableAccount>);

impl NonPostableAccounts {
    /// Construit la liste : dédoublonnage par identifiant, puis tri (numéro
    /// lexicographique, puis identifiant).
    ///
    /// **Précondition** : au moins un compte. Une liste vide produirait un
    /// message sans numéro ; elle est vérifiée par `debug_assert!` — un appelant
    /// ne construit ce refus que lorsqu'il a trouvé un compte en défaut.
    pub fn new(accounts: impl IntoIterator<Item = NonPostableAccount>) -> Self {
        let mut list: Vec<NonPostableAccount> = accounts.into_iter().collect();
        // Dédoublonnage par identifiant d'abord (un même compte porté par
        // plusieurs lignes), puis tri d'affichage.
        list.sort_by_key(|a| a.account_id);
        list.dedup_by_key(|a| a.account_id);
        list.sort_by(|a, b| {
            a.account_number
                .cmp(&b.account_number)
                .then(a.account_id.cmp(&b.account_id))
        });
        debug_assert!(
            !list.is_empty(),
            "NonPostableAccounts::new : la liste des comptes refusés est vide"
        );
        Self(list)
    }

    /// Les comptes, dans l'ordre d'affichage.
    pub fn iter(&self) -> impl Iterator<Item = &NonPostableAccount> {
        self.0.iter()
    }

    /// Les numéros, dans l'ordre d'affichage.
    pub fn numbers(&self) -> Vec<&str> {
        self.0.iter().map(|a| a.account_number.as_str()).collect()
    }

    /// Nombre de comptes refusés.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Toujours faux pour une liste construite par [`Self::new`] (précondition).
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// **Seul constructeur** du détail JSON du refus (choix C29) :
    /// `{ "rejected": [{ "accountId", "accountNumber" }] }`, dans l'ordre de la
    /// liste — la forme du jumeau `ACCOUNT_ARCHIVED`.
    ///
    /// Le bras HTTP 400 de `kesh-api` l'appelle, ainsi que les `failed[].details`
    /// de `POST /reconciliation/accept` (Story 15-5b) ; aucun autre site ne
    /// construit ce JSON, pour qu'un même refus n'ait qu'une forme.
    pub fn details(&self) -> serde_json::Value {
        let rejected: Vec<serde_json::Value> = self
            .0
            .iter()
            .map(|a| {
                serde_json::json!({
                    "accountId": a.account_id,
                    "accountNumber": a.account_number,
                })
            })
            .collect();
        serde_json::json!({ "rejected": rejected })
    }
}

/// Erreurs des opérations de persistance MariaDB.
///
/// Les messages `Display` sont destinés au logging serveur uniquement.
/// `kesh-api` mappe chaque variante vers un code HTTP et un message traduit
/// via `kesh-i18n`. Ne jamais exposer le `Display` au frontend.
///
/// **Important** : cette enum ne dérive PAS `From<sqlx::Error>` pour forcer
/// tous les call sites à passer par `map_db_error`, garantissant ainsi que
/// les violations de contraintes sont correctement classifiées.
#[derive(Debug, Error)]
pub enum DbError {
    /// Entité introuvable (SELECT sans résultat sur une opération qui en attend un).
    #[error("Entité non trouvée")]
    NotFound,

    /// Verrouillage optimiste : version en base ≠ version fournie dans l'UPDATE.
    #[error("Conflit de version — l'entité a été modifiée par un autre utilisateur")]
    OptimisticLockConflict,

    /// Contrainte d'unicité violée (code MariaDB 1062).
    #[error("Contrainte d'unicité violée : {0}")]
    UniqueConstraintViolation(String),

    /// Contrainte de clé étrangère violée (codes MariaDB 1451/1452).
    #[error("Contrainte de clé étrangère violée : {0}")]
    ForeignKeyViolation(String),

    /// Contrainte CHECK violée (code MariaDB 4025, MySQL 3819).
    #[error("Contrainte CHECK violée : {0}")]
    CheckConstraintViolation(String),

    /// Transition d'état métier interdite (ex: re-clôturer un exercice déjà
    /// clos — idempotence de `close`). Mappé vers HTTP 409 Conflict côté API.
    ///
    /// **Note (Story 14-2)** : la réouverture d'un exercice clos est désormais
    /// **autorisée** (via `fiscal_years::reopen`, Admin + motif + audit). Ses
    /// conflits métier (déjà ouvert, garde LIFO) sont émis en
    /// [`DbError::Invariant`] namespacés (message distinct localisé au mapper),
    /// **pas** via cette variante dont le `Display` est log-only.
    #[error("Transition d'état interdite : {0}")]
    IllegalStateTransition(String),

    /// L'exercice comptable est clôturé (FR24, CO art. 957-964) — aucune
    /// écriture ne peut y être ajoutée, modifiée ou supprimée. Variante
    /// dédiée (séparée d'`IllegalStateTransition`) pour permettre un
    /// mapping API stable, non dépendant du contenu du message texte.
    #[error("Exercice clôturé — modification interdite (CO art. 957-964)")]
    FiscalYearClosed,

    /// Un exercice **postérieur** est clôturé (Story 15-8a, #532, C-15-8-22 ;
    /// Stories 15-12a et 15-12b, #543).
    ///
    /// Le bilan est **cumulatif** (`kesh-report/src/balance_sheet.rs`) : toucher
    /// à ce qui précède un exercice clos change son bilan. Rendue par :
    /// - le `PUT` d'une écriture (`journal_entries::update`) dont un exercice
    ///   postérieur est clos ;
    /// - **toute création** d'écriture dans un tel exercice
    ///   (`journal_entries::create_in_tx_inner`, Story 15-12b — saisie,
    ///   contre-passation, et les dix-sept autres routes qui écrivent au
    ///   journal ; dans `failed[]` pour l'acceptation par lot du rapprochement) ;
    /// - **toute suppression** (`journal_entries::delete_in_tx` : la route
    ///   `DELETE` et, depuis la Story 15-12b, la dévalidation d'une facture) ;
    /// - la **création** d'un exercice (`fiscal_years::create`, Story 15-12a)
    ///   dont la date de début précède celle d'un exercice clos.
    ///
    /// Depuis la Story 15-12a, `fiscal_years::close` refuse de clôturer tant
    /// qu'un exercice antérieur est ouvert ([`DbError::EarlierFiscalYearOpen`]) :
    /// « N ouvert, N+1 clos » n'est plus atteignable **à partir d'un état sain**.
    /// Il subsiste dans les données héritées (version antérieure, sauvegarde
    /// restaurée) — d'où ces gardes, et le bandeau de l'écran des exercices qui
    /// le signale (Story 15-12b). Le refus nomme le **plus proche**
    /// postérieur clos. Mappé vers HTTP **400** `LATER_FISCAL_YEAR_CLOSED` —
    /// l'état d'un exercice, comme `FISCAL_YEAR_CLOSED`, pas un conflit sur
    /// l'objet.
    #[error("Exercice postérieur {fiscal_year_name} clôturé — écriture refusée")]
    LaterFiscalYearClosed {
        fiscal_year_id: i64,
        fiscal_year_name: String,
    },

    /// La clôture d'un exercice est refusée : un exercice **antérieur** de la
    /// même société est encore ouvert (Story 15-12a, #543).
    ///
    /// Invariant I (`repositories::fiscal_years`, doc du module) : les exercices
    /// clôturés forment un **préfixe** de l'ordre chronologique. La variante
    /// nomme le **plus ancien** antérieur ouvert — celui qu'il faut clôturer
    /// d'abord (un plus proche serait refusé à son tour). Mappée vers HTTP
    /// **409** `EARLIER_FISCAL_YEAR_OPEN` : un refus de transition, comme la
    /// garde LIFO de la réouverture, mais avec un code dédié — l'écran traduit
    /// tout `ILLEGAL_STATE_TRANSITION` de la clôture en « déjà clôturé ».
    #[error("Exercice antérieur {fiscal_year_name} encore ouvert — clôture refusée")]
    EarlierFiscalYearOpen {
        fiscal_year_id: i64,
        fiscal_year_name: String,
    },

    /// Un ou plusieurs comptes référencés sont archivés ou n'appartiennent
    /// pas à la company courante. Variante dédiée pour exposer un message
    /// UX clair sans leak du détail interne.
    #[error("Un ou plusieurs comptes sont archivés ou invalides")]
    InactiveOrInvalidAccounts,

    /// Un ou plusieurs comptes **de la société, actifs**, ne sont pas
    /// imputables : compte de regroupement (`postable = FALSE`), de résultat
    /// (rôle `CurrentYearResult`) ou de clôture (Story 15-5a, #429).
    ///
    /// Pourquoi une variante dédiée (choix C3) : ces refus étaient rendus en
    /// [`DbError::InactiveOrInvalidAccounts`], « archivés ou invalides », message
    /// faux pour ce motif — l'utilisateur cherchait un compte archivé qui ne
    /// l'était pas. Précédent : [`RevenueAccountRejection::NotPostable`], propre
    /// aux lignes de facture et donc non réutilisé.
    ///
    /// Ordre des causes : un compte inconnu, d'une autre société ou archivé
    /// reste `InactiveOrInvalidAccounts` et **prime** — cette variante ne nomme
    /// que des comptes dont l'appartenance a été contrôlée (KF-002).
    ///
    /// Construite par [`DbError::accounts_not_postable`] ; la liste est triée,
    /// dédoublonnée et non vide par construction ([`NonPostableAccounts`]).
    /// Mappé vers HTTP **400** `ACCOUNT_NOT_POSTABLE`, détail
    /// [`NonPostableAccounts::details`].
    #[error("Un ou plusieurs comptes ne sont pas imputables")]
    AccountsNotPostable(NonPostableAccounts),

    /// Un ou plusieurs comptes **désignés dans les réglages de facturation** —
    /// créance, TVA due, créanciers, TVA récupérable —, de la société et actifs,
    /// ne sont pas imputables au moment où un flux automatique veut y écrire
    /// (Story 15-5d, #429 ; choix C27, C28).
    ///
    /// Jumelle de [`DbError::AccountsNotPostable`] : **même code**
    /// (`ACCOUNT_NOT_POSTABLE`), **même détail** ([`NonPostableAccounts::details`]),
    /// un seul contrat pour l'intégrateur. Elle n'en diffère que par le
    /// **message**, qui dit où agir — *Paramètres → Facturation* — et qui peut le
    /// faire (un administrateur, choix C36) : le compte n'a pas été choisi dans
    /// la requête, il vient des réglages.
    ///
    /// Émise par le contrôle des comptes désignés
    /// (`company_invoice_settings::DesignatedAccountsSnapshot::check_written`),
    /// à la validation d'une facture et à la saisie (ou la complétion) d'une
    /// facture fournisseur. Un compte désigné absent, d'une autre société ou
    /// archivé reste [`DbError::InactiveOrInvalidAccounts`] et prime (C49).
    #[error(
        "Un ou plusieurs comptes désignés dans les réglages de facturation ne sont pas imputables"
    )]
    DesignatedAccountsNotPostable(NonPostableAccounts),

    /// Un paiement solde une facture au centime et produit un écart d'arrondi,
    /// mais aucun compte de différences d'arrondi utilisable n'est désigné dans
    /// les paramètres de facturation (Story 25-4-c3-b, #476) : réglage vide, ou
    /// compte devenu archivé, non imputable ou d'un autre type depuis (#486).
    ///
    /// ⛔ **Variante dédiée, et non `ConfigurationRequired`** : le mapping de
    /// celle-ci jette le champ et rend un code générique, alors que le refus doit
    /// dire QUOI configurer — *Paramètres → Facturation*.
    ///
    /// Story 25-4-c4-a (#494) : le **contexte** dit qui a besoin du compte — un
    /// paiement qui solde au centime, ou une pièce émise arrondie à 5 centimes —,
    /// pour que le message ne parle pas de paiement là où il n'y en a pas.
    #[error("Aucun compte de différences d'arrondi utilisable n'est désigné")]
    RoundingAccountNotConfigured { context: RoundingContext },

    /// Le compte de la **nature** d'un solde (Story 25-4-d2a, #384) n'est pas
    /// désigné dans les paramètres de facturation, ou ne l'est plus utilement
    /// (archivé, non imputable, retypé). `nature` est la graphie persistée
    /// (`discount`, `bank_fees`, `bad_debt`, `rounding`), pour que le message
    /// nomme le réglage à remplir.
    #[error("Aucun compte utilisable n'est désigné pour la nature de solde {nature}")]
    WriteOffAccountNotConfigured { nature: &'static str },

    /// Le total TTC arrondi d'une facture est inférieur au montant minimum fixé
    /// dans les paramètres de facturation (Story 25-4-e, #495) : elle ne s'émet
    /// pas. Les deux montants voyagent avec l'erreur pour que le message les nomme.
    #[error("Total {total} inférieur au montant minimum {minimum}")]
    InvoiceBelowMinimum {
        total: rust_decimal::Decimal,
        minimum: rust_decimal::Decimal,
    },

    /// La date fournie ne tombe pas dans l'exercice courant de l'entité
    /// modifiée. Story 3.3 : empêche le déplacement d'une écriture vers
    /// un autre exercice via un simple changement de date.
    #[error("La date n'est pas dans l'exercice courant de cette écriture")]
    DateOutsideFiscalYear,

    /// Le compte porte des écritures et l'appelant veut changer son
    /// `account_type` **sans l'avoir confirmé** (Story 25-2-a, [#382], [#274]).
    ///
    /// ⚠️ **Ce n'est pas un refus définitif** — l'arbitrage retenu est
    /// l'*avertissement bloquant*, non le refus sec : un compte mal typé à la
    /// création le resterait sinon à vie, rien ne permettant de déplacer ses
    /// écritures ailleurs. L'appelant lève l'obstacle en confirmant.
    ///
    /// Les champs portent l'**ampleur** parce qu'un refus qui ne dit pas ce
    /// qu'il protège ne sert qu'à être contourné : le comptable doit lire
    /// combien d'écritures et quels exercices **clos** basculeraient d'un état à
    /// l'autre avant de décider. C'est là tout l'enjeu — retyper un compte
    /// mouvementé change le résultat d'un exercice clos sans qu'aucune écriture
    /// ne l'explique.
    #[error(
        "Le compte porte {entry_count} écriture(s) : changer son type reclasserait tout son historique"
    )]
    AccountHasEntries {
        /// Écritures **distinctes**, jamais des lignes.
        entry_count: i64,
        /// Noms des exercices **clos** touchés, triés ; vide si aucun.
        closed_fiscal_years: Vec<String>,
        from_type: &'static str,
        to_type: &'static str,
    },

    /// Un rôle de compte **singleton** est déjà porté par un autre compte actif
    /// de la même société (Story 14-3a).
    ///
    /// Variante dédiée — et non le générique [`DbError::UniqueConstraintViolation`]
    /// — pour que l'API puisse **nommer le compte en conflit** : le mapping
    /// générique du code MariaDB 1062 produit un message fixe et jette le détail.
    /// Les champs viennent d'un `SELECT` fait dans la même transaction, avant
    /// l'écriture ; la contrainte `uq_accounts_company_singleton_role` reste la
    /// source de vérité (elle rattrape les courses perdues en 1062).
    #[error("Le rôle {role} est déjà attribué au compte {account_number}")]
    AccountRoleAlreadyAssigned {
        /// Rôle en conflit, en PascalCase (ex. `"Receivable"`).
        role: String,
        /// ID du compte qui porte déjà le rôle.
        account_id: i64,
        /// Numéro du compte qui porte déjà le rôle.
        account_number: String,
        /// Libellé du compte qui porte déjà le rôle.
        account_name: String,
    },

    /// Réactivation refusée : le compte parent est archivé (Story 14-3a, #269).
    ///
    /// Variante dédiée — et non le générique [`DbError::IllegalStateTransition`]
    /// — parce que ce dernier est mappé sur un message fixe (« Transition d'état
    /// interdite ») qui ne dit **pas** à l'utilisateur ce qu'il doit faire. Le
    /// code review Pass 1 a montré que le message explicatif écrit côté
    /// repository n'atteignait jamais le client. Le numéro du parent permet à
    /// l'API de produire un message actionnable et traduit.
    #[error("Le compte parent {parent_number} est archivé")]
    AccountParentArchived {
        /// Numéro du compte parent archivé.
        parent_number: String,
    },

    /// Le rôle demandé est incompatible avec le type du compte (Story 14-3a).
    ///
    /// Contrainte volontairement **minimale** : seule la frontière bilan /
    /// résultat est vérifiée (cf. `AccountRole::accepts_account_type`). Mappée
    /// vers HTTP 400 côté API.
    #[error("Le rôle {role} est incompatible avec un compte de type {account_type}")]
    AccountRoleInvalidForType {
        /// Rôle demandé, en PascalCase (ex. `"Payable"`).
        role: String,
        /// Type du compte, en PascalCase (ex. `"Expense"`).
        account_type: String,
    },

    /// Un ou plusieurs comptes de produit de ligne de facture sont refusés
    /// (Story 16-1a, #152 — D3, D3-bis, D6, AC8-bis).
    ///
    /// Variante dédiée — et non le générique [`DbError::InactiveOrInvalidAccounts`]
    /// — parce que ce dernier est mappé sur « Un ou plusieurs comptes sont
    /// archivés ou invalides », qui **ne nomme aucune ligne**. Sur une facture
    /// pouvant porter 200 lignes, ce message n'est pas actionnable. Les sites
    /// en défaut sont donc transportés en structuré jusqu'à `kesh-api`, qui
    /// compose un message traduit les nommant tous.
    ///
    /// Mappé vers HTTP 400 `INVOICE_LINE_REVENUE_ACCOUNT_INVALID`.
    #[error("Comptes de produit de ligne invalides ({} site(s) en défaut)", .0.len())]
    InvalidRevenueAccounts(Vec<RejectedRevenueAccount>),

    /// Émission d'avoir bloquée : au moins un compte de produit du snapshot de
    /// la facture est archivé (Story 16-1a, décision D5-bis).
    ///
    /// La contre-passation doit viser **les mêmes comptes** que l'écriture
    /// d'origine — se replier sur le défaut société recréerait exactement le
    /// résidu que D5 combat. Poster sur un compte archivé est impossible (la
    /// garde `active` de `create_in_tx` est inconditionnelle). L'avoir échoue
    /// donc, en nommant la ligne et le compte à réactiver : un avoir bloqué
    /// est préférable à un avoir sur le mauvais compte.
    ///
    /// Seul `active` est concerné — ni `postable` ni `account_type` ne sont
    /// re-vérifiés côté avoir (D5-bis).
    ///
    /// Mappé vers HTTP 400 `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED`.
    #[error("Comptes de produit archivés sur l'avoir ({} ligne(s))", .0.len())]
    CreditNoteRevenueAccountsArchived(Vec<RejectedRevenueAccount>),

    /// La facture ne peut pas être **dévalidée** (Story 25-2-b-1, #440).
    ///
    /// Conflit d'état → HTTP **409**, avec le code canonique de
    /// l'[`UnvalidationBlocker`] et, quand le motif désigne une pièce, son
    /// identifiant et son étiquette lisible.
    #[error("Facture non dévalidable ({})", .blocker.code())]
    InvoiceNotUnvalidatable {
        blocker: UnvalidationBlocker,
        /// Identifiant de la pièce qui bloque, quand le motif en désigne une
        /// (l'avoir, le règlement…).
        document_id: Option<i64>,
        /// **Étiquette lisible** de ce qui bloque — le numéro de l'avoir, la
        /// date du règlement. ⚠️ Un identifiant de base ne se comprend pas.
        document_label: Option<String>,
    },

    /// Un avoir ne peut pas viser cette facture : elle porte un **règlement**,
    /// même partiel (Story 25-4-a, #456).
    ///
    /// ⛔ **Les deux conditions, jamais l'une seule** : une ligne
    /// `invoice_settlements` **ou** `paid_at` posé — comme
    /// [`UnvalidationBlocker::Settled`]. Depuis la 24-2, `paid_at` n'est posé
    /// qu'au solde : la garde qui ne lisait que lui laissait passer une facture
    /// réglée en partie, et l'avoir, qui contre-passe tout le TTC, rendait la
    /// créance **créditrice** du montant encaissé.
    ///
    /// Conflit d'état → HTTP **409** `CREDIT_NOTE_INVOICE_SETTLED`. Remplace le
    /// générique `IllegalStateTransition` de l'ancienne garde AC2bis, qui ne
    /// disait ni ce qui bloquait ni quoi faire.
    #[error("Avoir refusé : la facture porte un règlement")]
    CreditNoteBlockedBySettlement {
        invoice_id: i64,
        /// Le premier règlement trouvé ; `None` quand seul `paid_at` est posé
        /// (facture soldée avant la 24-2, sans ligne de règlement).
        settlement_id: Option<i64>,
        /// Le **numéro** de la facture, que l'utilisateur connaît.
        invoice_number: Option<String>,
    },

    /// Le règlement ne peut pas être annulé (Story 25-3-a-1, #414).
    ///
    /// Conflit d'état → HTTP **409**, avec le code canonique du
    /// [`SettlementCancelBlocker`]. ⚠️ Seuls les rangs que le **geste** refuse
    /// lui-même passent par ici (facture créditée, exercice clos) ; les autres
    /// sont refusés par la contre-passation, avec son erreur propre.
    #[error("Règlement non annulable ({})", .blocker.code())]
    SettlementNotCancellable { blocker: SettlementCancelBlocker },

    /// Le rapprochement bancaire ne peut pas être annulé (Story 25-3-b, #418).
    ///
    /// Conflit d'état → HTTP **409**, avec le code canonique du
    /// [`SettlementCancelBlocker`]. ⛔ **Distincte de `SettlementNotCancellable`**,
    /// dont le texte dit « ce règlement » : une écriture d'éclatement, de règle
    /// ou de rapprochement manuel n'est pas un règlement. Seuls les rangs que
    /// le dé-rapprochement refuse **lui-même** passent par ici (rang 0 :
    /// transaction non rapprochée ; rang 2 : exercice clos) ; le rang 1 est
    /// refusé par le geste d'annulation du règlement client, les rangs 3 à 5
    /// par la contre-passation.
    #[error("Rapprochement non annulable ({})", .blocker.code())]
    ReconciliationNotCancellable { blocker: SettlementCancelBlocker },

    /// La facture fournisseur ne peut pas être annulée (Story 25-3-c, #454).
    ///
    /// Conflit d'état → HTTP **409**, avec le code canonique du
    /// [`SettlementCancelBlocker`]. ⛔ **Distincte de `SettlementNotCancellable`**
    /// pour la même raison que `ReconciliationNotCancellable` : ses textes
    /// disent « cette facture », non « ce règlement ». Seuls les rangs que le
    /// geste refuse lui-même passent par ici (facture déjà annulée, exercice de
    /// l'achat clos, lot en cours) ; les autres sont refusés par la
    /// contre-passation, avec son erreur propre.
    #[error("Facture fournisseur non annulable ({})", .blocker.code())]
    SupplierInvoiceNotCancellable { blocker: SettlementCancelBlocker },

    /// Un brouillon **numéroté** changerait d'exercice (Story 25-2-b-1, #440).
    ///
    /// Le numéro vient du compteur de l'exercice qui couvre la date : le
    /// déplacer lui ferait porter le numéro d'une autre séquence. Conflit
    /// d'état → **409**.
    #[error("Un brouillon numéroté ne peut pas changer d'exercice")]
    InvoiceNumberFiscalYearMismatch,

    /// `delete` a reçu une facture **validée** (Story 25-2-b-2, #440).
    ///
    /// ⛔ **Un code propre, et non le générique `ILLEGAL_STATE_TRANSITION`** : le
    /// message doit **orienter vers la dévalidation**, sans quoi l'utilisateur
    /// lit « transition interdite » sur le seul chemin qui lui reste.
    #[error("Facture validée : dévalider d'abord")]
    InvoiceMustBeUnvalidatedFirst,

    /// L'écriture ne peut pas être contre-passée (Story 24-4a, #380).
    ///
    /// C'est un **conflit d'état**, pas une donnée invalide → HTTP **409**, avec
    /// le code canonique du [`ReversalBlocker`] et l'identifiant de la pièce
    /// propriétaire quand il y en a une. Le message doit nommer la pièce ET le
    /// chemin de correction : un utilisateur qui lit « corrigez la facture
    /// F-2026-014 par un avoir » sait quoi faire, un « interdit » sec non.
    #[error("Écriture non contre-passable ({})", .blocker.code())]
    EntryNotReversable {
        blocker: ReversalBlocker,
        /// Identifiant de la pièce propriétaire, quand le motif en désigne une.
        document_id: Option<i64>,
        /// **Étiquette lisible** de ce qui bloque : le numéro de la pièce
        /// (`F-2026-014`, `AV-2026-3`…), ou le **numéro du compte** archivé.
        ///
        /// ⚠️ Un identifiant de base de données ne se comprend pas ; c'est ce que
        /// l'utilisateur voit sur son document, ou dans son plan comptable.
        document_label: Option<String>,
    },

    /// Un ou plusieurs comptes de l'écriture d'origine ont été **archivés**
    /// depuis (Story 24-4a, #380).
    ///
    /// ⛔ `enforce_postable = false` NE SUFFIT PAS : la garde `active` de la
    /// validation des comptes (décidée en Rust depuis la Story 15-5a) est
    /// **inconditionnelle**, seule `postable` est gouvernée par le drapeau.
    ///
    /// Mappé vers HTTP **400** `ACCOUNT_ARCHIVED` — même statut que le gabarit
    /// [`DbError::CreditNoteRevenueAccountsArchived`] dont il reprend la forme.
    #[error("Comptes archivés sur l'écriture à contre-passer ({})", .0.len())]
    ReversalAccountsArchived(Vec<ArchivedAccount>),

    /// Un compte que l'**avoir** écrit hors des comptes de produit — la
    /// **créance** et le compte d'**arrondi** que la vente a mouvementés, ou le
    /// compte de **TVA due** des réglages — a été **archivé** (Story 15-6a, #473,
    /// #523 ; choix C-15-6-25, C-15-6-29).
    ///
    /// Lu **sous verrou partagé**, avant l'exercice (`create_credit_note`) : un
    /// archivage concurrent attend la fin de l'avoir, et l'état actif est frais.
    /// Jumelle de [`DbError::ReversalAccountsArchived`] — l'avoir **est** une
    /// contre-passation — dont elle reprend la forme et le code
    /// (`ACCOUNT_ARCHIVED`, HTTP **400**, `details.rejected[]`, ordre des
    /// identifiants) ; seul le message diffère (« Impossible d'émettre
    /// l'avoir »), pour parler le vocabulaire du geste. Un compte de **produit**
    /// archivé reste nommé par ligne par
    /// [`DbError::CreditNoteRevenueAccountsArchived`].
    #[error("Comptes archivés sur l'avoir à émettre ({})", .0.len())]
    CreditNoteAccountsArchived(Vec<ArchivedAccount>),

    /// Un règlement viserait **le compte même qu'il doit solder** (Story 15-6b,
    /// #474) : la contrepartie — ou un compte d'écart du même geste — est le
    /// compte débiteurs de la facture client, ou le compte créanciers de la
    /// facture fournisseur.
    ///
    /// ⛔ **Le motif** : l'écriture serait `D X / C X`. Elle s'équilibre (rien
    /// n'interdit un même compte au débit et au crédit), le règlement
    /// s'enregistre, le reste dû baisse, la facture peut passer « payée » — et le
    /// grand livre **ne bouge pas**. Le refus vient avant toute écriture.
    ///
    /// `role` dit **d'où vient** le compte en cause, donc le remède
    /// ([`SettlementAccountRole`]). ⚠️ **Le rôle suit la colonne de réglage lue,
    /// pas la route** : le compte d'arrondi du règlement, celui du rapprochement,
    /// le reste d'arrondi d'un solde et le compte de la nature `rounding` lisent
    /// tous `default_rounding_account_id` — ils sortent tous sous
    /// `role: Rounding`. `claim: Payable` ne se combine qu'avec `Counterparty` :
    /// le règlement fournisseur n'écrit aucun compte d'écart, et le constructeur
    /// unique (`invoice_settlements::claim_account_refusal`) rend la combinaison
    /// irreprésentable.
    ///
    /// `batch` ne vaut `Some` qu'à la **confirmation d'un lot** de paiement
    /// (`payment_batches::confirm_batch`), qui intercepte le refus de
    /// `pay_in_tx` pour dire lequel de ses règlements est en cause.
    ///
    /// Mappé vers HTTP **400** `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` — un
    /// seul code pour les quatre rôles, le défaut étant le même ; `details.role`
    /// et le message distinguent le remède.
    #[error("La contrepartie du règlement est le compte qu'il solde (compte {account_id})")]
    SettlementCounterpartyIsClaimAccount {
        /// Le compte en cause — qui **est** le compte soldé.
        account_id: i64,
        /// Son numéro, lu à l'échec ; `None` s'il n'a pas pu être résolu.
        account_number: Option<String>,
        /// Le côté soldé : créance client ou dette fournisseur.
        claim: ClaimSide,
        /// D'où vient le compte en cause, donc quel est le remède.
        role: SettlementAccountRole,
        /// Le lot et la facture, à la seule confirmation d'un lot.
        batch: Option<SettlementBatchContext>,
    },

    /// Un compte bancaire serait lié au **compte débiteurs** ou au **compte
    /// créanciers** désigné dans les réglages de facturation (Story 15-6c,
    /// #474 ; choix C-15-6-5, C-15-6-13).
    ///
    /// ⛔ **Le motif** : chaque encaissement par ce compte bancaire écrirait
    /// `D 1100 / C 1100` — une écriture nulle, que la garde à l'usage de la
    /// 15-6b ([`DbError::SettlementCounterpartyIsClaimAccount`]) refuse à chaque
    /// règlement. Ce refus l'arrête **à la configuration**, une fois.
    ///
    /// Levé par la création, le remplacement et le lien d'un compte bancaire,
    /// **seulement si le compte lié change** (exemption « inchangé », patron
    /// C10 de la 15-5b). Mappé vers HTTP **400**
    /// `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` — routes ouvertes aux clés d'API.
    #[error(
        "Le compte {account_id} est le compte de créance désigné : un compte bancaire ne peut pas y être lié"
    )]
    BankAccountLedgerIsClaimAccount {
        /// Le compte du grand livre visé par le lien.
        account_id: i64,
        /// Son numéro, lu au refus ; `None` s'il n'a pas pu être résolu.
        account_number: Option<String>,
        /// Le réglage qu'il occupe : compte débiteurs ou compte créanciers.
        claim: ClaimSide,
    },

    /// Le compte qu'on désigne comme **compte débiteurs** ou **compte
    /// créanciers** est lié à un compte bancaire **non archivé** (Story 15-6c,
    /// #474) — le symétrique de [`DbError::BankAccountLedgerIsClaimAccount`].
    ///
    /// Levé par `company_invoice_settings::update`, sous verrou, **seulement si
    /// la valeur change**. Nomme le **premier** compte bancaire lié par `id`.
    /// Mappé vers HTTP **400** `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` — émis par
    /// une route d'administration fermée aux clés d'API : il ne sert que l'écran.
    #[error(
        "Le compte {account_id} est lié au compte bancaire {bank_account_id} : il ne peut pas être désigné comme compte de créance"
    )]
    ClaimAccountLinkedToBankAccount {
        /// Le compte qu'on voudrait désigner.
        account_id: i64,
        /// Son numéro, lu au refus ; `None` s'il n'a pas pu être résolu.
        account_number: Option<String>,
        /// Le réglage visé : compte débiteurs ou compte créanciers.
        claim: ClaimSide,
        /// Le premier compte bancaire non archivé lié à ce compte, par `id`.
        bank_account_id: i64,
        /// Son nom, tel que le rend `bank_accounts::first_active_bank_account_linked_to`.
        bank_name: String,
    },

    /// L'écriture a été contre-passée : on ne la modifie ni ne la supprime
    /// plus (Story 24-4a ; la modification, Story 15-8a).
    ///
    /// Supprimer une écriture qu'on a corrigée **effacerait la correction** —
    /// exactement ce que l'art. 958f CO interdit. Le refus est donc voulu ; il
    /// est rendu explicite plutôt que laissé remonter comme une violation de
    /// clé étrangère au message opaque. Mappé vers HTTP **409**
    /// `ENTRY_IS_REVERSED`.
    #[error("Écriture contre-passée : modification et suppression refusées")]
    EntryIsReversed,

    /// L'écriture ne se **modifie** ni ne se **supprime** : une pièce la
    /// possède, c'est une contre-passation, ou c'est le paiement détaché d'une
    /// facture fournisseur annulée (Story 15-8a, #532, D2 ; la suppression,
    /// Story 15-8b, passe par la même garde).
    ///
    /// ⛔ Jamais construite avec `Owned { blocker: AlreadyReversed }` : une
    /// écriture contre-passée rend [`DbError::EntryIsReversed`], le code de la
    /// 24-4a. Mappé vers HTTP **409**, sous le code de la garde
    /// ([`ModificationGuard::code`]) ; `error_code()` n'en rend que le repli.
    #[error("Écriture non modifiable ({})", .0.code())]
    EntryNotModifiable(ModificationGuard),

    /// La date de l'écriture tombe dans une période verrouillée
    /// (Story 24-4c, #380).
    ///
    /// ⛔ Le seuil est **inclusif** : une borne au 31.03 refuse le 31.03.
    ///
    /// ⚠️ C'est un **400**, pas un 409 : ce qui est invalide, c'est la **date
    /// proposée** — ou la date de l'écriture qu'on voudrait supprimer —, pas
    /// l'état d'une ressource. Les refus d'état (pièce, contre-passation :
    /// [`DbError::EntryNotModifiable`], [`DbError::EntryIsReversed`]) sont des
    /// 409, et parlent AVANT ce verrou au `PUT` comme au `DELETE`.
    ///
    /// Les deux dates voyagent avec l'erreur pour que le message les NOMME :
    /// un refus qui ne dit pas jusqu'où les livres sont fermés n'est pas
    /// utilisable.
    #[error(
        "Les écritures sont verrouillées jusqu'au {locked_through} ; celle-ci est datée du {attempted}"
    )]
    PeriodLocked {
        locked_through: chrono::NaiveDate,
        attempted: chrono::NaiveDate,
    },

    // --- Lettrage (Story 15-1a-i, #518) — refus de la primitive unique ---
    //
    // ⚠️ L'ordre des variantes ci-dessous suit les RANGS des refus de
    // `letterings::create_group_in_tx` (AC3) ; une requête qui cumule deux
    // causes rend la première. Le 404 (rang 2) est [`DbError::NotFound`],
    // indiscernable d'une ligne inexistante (AC11). Tous sont des refus
    // MÉTIER, jamais un `Invariant` : mappés vers 400 (forme) ou 409 (état).
    /// Rang 1 : moins de deux identifiants de lignes, ou un doublon. → 400.
    #[error("Un lettrage réunit au moins deux lignes distinctes")]
    LetteringTooFewLines,

    /// Plus de lignes que le plafond d'un groupe manuel (200, AC6). → 400.
    #[error("Un lettrage réunit au plus {max} lignes")]
    LetteringTooManyLines { max: usize },

    /// Rang 3 : les lignes ne portent pas toutes sur le même compte. → 409.
    #[error("Les lignes d'un lettrage doivent toutes porter sur le même compte")]
    LetteringAccountsDiffer,

    /// Rang 4 : le compte n'est pas lettrable (R4 — ni actif ni passif, ou
    /// désigné par un compte bancaire, archivé compris). Exigé à la CRÉATION
    /// d'un groupe seulement (C104). → 409.
    #[error("Compte non lettrable")]
    LetteringAccountNotLetterable,

    /// Rang 4 bis (mode manuel) : aucune ligne du groupe n'est « en période
    /// ouverte » — exercice clos, exercice postérieur clos, ou date ≤ verrou de
    /// période (R7). Vaut pour le lettrage comme pour le délettrage. → 409.
    #[error("Toutes les lignes du lettrage sont dans une période close")]
    LetteringAllLinesInClosedPeriods,

    /// Rang 5 (mode manuel) : une ligne appartient à une pièce (motifs
    /// `OwnedBy*` de `reversal_blockers`, R5) — son lettrage est celui de sa
    /// pièce. Rendu aussi au délettrage d'un groupe `reversal` dont une ligne
    /// appartient à une pièce (AC5, refus 2). → 409, avec `documentId` /
    /// `documentNumber`.
    #[error("Une ligne du lettrage appartient à une pièce ({})", .blocker.code())]
    LetteringLineOwnedByDocument {
        blocker: ReversalBlocker,
        document_id: Option<i64>,
        document_label: Option<String>,
    },

    /// Rang 6 : une ligne est déjà lettrée — `code` est celui de son groupe.
    /// → 409.
    #[error("Une ligne est déjà lettrée (code {code})")]
    LetteringLineAlreadyLettered { code: String },

    /// Rang 7 : la somme `Σ(débit − crédit)` des lignes n'est pas nulle. → 409.
    #[error("Les lignes ne se soldent pas : écart de {difference}")]
    LetteringUnbalanced { difference: rust_decimal::Decimal },

    /// Délettrage manuel d'un groupe d'origine `document` : son lettrage suit
    /// la pièce (annuler le règlement, pas délettrer — AC5, refus 1). → 409.
    #[error("Ce lettrage est celui d'une pièce")]
    LetteringIsDocument,

    /// L'`UPDATE` final d'une primitive n'a pas trouvé le nombre de lignes
    /// attendu (R7 point 4). Les lignes sont tenues depuis la lecture
    /// verrouillante : seul un défaut y mène — mais c'est un refus MÉTIER
    /// (« réessayez »), jamais un succès partiel ni un 500. → 409.
    #[error("Le lettrage a changé entre-temps")]
    LetteringConcurrentChange,

    /// Complément des soldes de départ refusé (Story 25-7, #445).
    ///
    /// Le statut HTTP et le code se dérivent de la raison côté API ;
    /// `account_id` / `account_number` désignent le compte fautif pour les
    /// refus par compte.
    /// ⛔ Jamais un `Invariant` : ce sont des refus métier, pas des défauts.
    #[error("Complément des soldes de départ refusé ({})", .reason.code())]
    OpeningComplementRefused {
        reason: crate::repositories::opening_complement::OpeningComplementRefusal,
        account_id: Option<i64>,
        /// Numéro du compte fautif, quand il appartient à la société.
        account_number: Option<String>,
    },

    /// Aucun exercice ouvert ne couvre la date fournie (Story 5.2).
    /// Distinct de `FiscalYearClosed` — l'exercice est peut-être
    /// inexistant (date hors de tous les exercices connus) OU clôturé.
    /// Mappé vers HTTP 400 `FISCAL_YEAR_INVALID` côté API.
    #[error("Aucun exercice ouvert ne couvre cette date")]
    FiscalYearInvalid,

    /// Un champ de configuration requis pour l'opération est absent
    /// (Story 5.2 : `default_receivable_account_id` ou
    /// `default_revenue_account_id` manquant dans `company_invoice_settings`).
    /// Depuis la Story 15-6a, l'avoir ne lit plus la créance dans les
    /// réglages (il la lit sur l'écriture de vente) : un débiteurs vide ne
    /// l'atteint plus ; la validation de facture, si. L'avoir peut encore le
    /// rendre pour le produit de repli ou la TVA due.
    /// Mappé vers HTTP 400 `CONFIGURATION_REQUIRED` côté API.
    #[error("Configuration manquante : {0}")]
    ConfigurationRequired(String),

    /// Pool épuisé ou timeout d'acquisition (retry-able côté API → 503).
    #[error("Pool de connexions épuisé ou timeout : {0}")]
    ConnectionUnavailable(String),

    /// Entrée invalide détectée côté repository (validation métier qui
    /// nécessite un round-trip DB, ex. `paid_at` antérieur à `invoice.date`).
    /// Le payload est un code stable (non i18n) — le handler le mappe vers
    /// une clé i18n FTL. Mappé vers HTTP 400 côté API.
    #[error("Entrée invalide : {0}")]
    InvalidInput(String),

    /// Invariant du crate violé (ex: AUTO_INCREMENT retourne une valeur impossible).
    /// Indique un bug ou un état de DB corrompu, jamais une erreur utilisateur.
    #[error("Invariant kesh-db violé : {0}")]
    Invariant(String),

    /// Donnée trop longue pour sa colonne (MariaDB **1406** `ER_DATA_TOO_LONG`)
    /// ou hors de la plage du type (**1264** `ER_WARN_DATA_OUT_OF_RANGE`).
    /// Story 12-5c (dette D2) : un champ d'un QR tiers **non conforme SIX 2.2**
    /// (nom créancier > 70 chars, message > 140, etc.) dépasse la largeur de sa
    /// colonne à l'INSERT staging. Variante typée (séparée du repli `Sqlx`) pour
    /// que la couche d'ingestion 12-5c mappe cet échec **par-fichier** (`failed[]`
    /// avec `error_code = "FIELD_TOO_LONG"`, HTTP 200) au lieu d'un 500 global.
    #[error("Donnée trop longue ou hors plage : {0}")]
    DataLengthOrRange(String),

    /// Erreur SQLx non classifiée (syntaxe, type mismatch, etc.).
    ///
    /// `#[source]` préserve la chaîne d'erreur pour anyhow/tracing —
    /// `DbError::source()` renvoie bien la `sqlx::Error` sous-jacente.
    #[error("Erreur SQLx : {0}")]
    Sqlx(#[source] sqlx::Error),
}

/// Qui réclame le compte de différences d'arrondi (Story 25-4-c4-a, #494).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingContext {
    /// Un paiement égal au reste arrondi au centime solde la facture
    /// (règlement manuel, rapprochement — Story 25-4-c3-b).
    Payment,
    /// Une facture validée porte un arrondi à 5 centimes. L'**avoir** n'y passe
    /// plus : il contre-passe l'arrondi sur le compte que la vente a mouvementé
    /// (Story 15-6a, #523 — `invoice_settlements::sale_rounding_account`).
    Issuance,
}

/// Le côté que solde un règlement (Story 15-6b, #474) : la **créance** d'une
/// facture client, ou la **dette** d'une facture fournisseur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimSide {
    /// Le compte débiteurs, lu sur l'écriture de vente.
    Receivable,
    /// Le compte créanciers, lu sur l'écriture d'achat.
    Payable,
}

impl ClaimSide {
    /// Discriminant machine exposé dans `details.claim`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Receivable => "receivable",
            Self::Payable => "payable",
        }
    }
}

/// D'où vient le compte qui coïncide avec le compte soldé — donc quel est le
/// remède (Story 15-6b, #474 ; choix C-15-6-26, C-15-6-30).
///
/// ⚠️ **Le rôle suit la colonne de réglage lue, pas la route** : un même compte
/// désigné sort sous le même rôle partout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementAccountRole {
    /// La contrepartie que l'utilisateur **choisit** (compte interne, compte
    /// bancaire du virement) ou que porte le **compte bancaire** du geste (lot,
    /// rapprochement). Remède : un autre compte, ou relier le compte bancaire à
    /// son propre compte de banque.
    Counterparty,
    /// Le compte de différences d'arrondi des réglages
    /// (`default_rounding_account_id`), **quel que soit le geste** : règlement,
    /// rapprochement, reste d'arrondi d'un solde, nature `rounding`.
    Rounding,
    /// Le compte d'une nature de solde des réglages — `discount`, `bank_fees`,
    /// `bad_debt` seulement (la nature `rounding` est [`Self::Rounding`]).
    WriteOffNature,
    /// Le compte de TVA due des réglages (solde du reste).
    VatPayable,
}

impl SettlementAccountRole {
    /// Discriminant machine exposé dans `details.role`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Counterparty => "counterparty",
            Self::Rounding => "rounding",
            Self::WriteOffNature => "write_off_nature",
            Self::VatPayable => "vat_payable",
        }
    }
}

/// Le lot de paiement et la facture en cause, quand le refus naît à la
/// **confirmation d'un lot** (Story 15-6b, AC6) : le remède n'y est pas celui
/// du règlement unitaire — l'utilisateur n'y choisit aucun compte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettlementBatchContext {
    pub payment_batch_id: i64,
    pub supplier_invoice_id: i64,
    /// Numéro de la facture fournisseur, relu par `confirm_batch`.
    pub supplier_invoice_number: Option<String>,
}

impl DbError {
    /// Raccourci de construction de [`DbError::AccountsNotPostable`] : trie,
    /// dédoublonne et vérifie la non-vacuité par [`NonPostableAccounts::new`].
    pub fn accounts_not_postable(accounts: impl IntoIterator<Item = NonPostableAccount>) -> Self {
        Self::AccountsNotPostable(NonPostableAccounts::new(accounts))
    }

    /// Code d'erreur structuré pour le mapping API (utilisé par `kesh-api`
    /// pour construire les réponses d'erreur JSON).
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::NotFound => "NOT_FOUND",
            Self::OptimisticLockConflict => "OPTIMISTIC_LOCK_CONFLICT",
            Self::UniqueConstraintViolation(_) => "UNIQUE_CONSTRAINT_VIOLATION",
            Self::ForeignKeyViolation(_) => "FOREIGN_KEY_VIOLATION",
            Self::CheckConstraintViolation(_) => "CHECK_CONSTRAINT_VIOLATION",
            Self::IllegalStateTransition(_) => "ILLEGAL_STATE_TRANSITION",
            Self::FiscalYearClosed => "FISCAL_YEAR_CLOSED",
            Self::LaterFiscalYearClosed { .. } => "LATER_FISCAL_YEAR_CLOSED",
            Self::EarlierFiscalYearOpen { .. } => "EARLIER_FISCAL_YEAR_OPEN",
            Self::InactiveOrInvalidAccounts => "INACTIVE_OR_INVALID_ACCOUNTS",
            Self::AccountsNotPostable(_) => "ACCOUNT_NOT_POSTABLE",
            Self::DesignatedAccountsNotPostable(_) => "ACCOUNT_NOT_POSTABLE",
            Self::RoundingAccountNotConfigured { .. } => "ROUNDING_ACCOUNT_NOT_CONFIGURED",
            Self::WriteOffAccountNotConfigured { .. } => "WRITE_OFF_ACCOUNT_NOT_CONFIGURED",
            Self::InvoiceBelowMinimum { .. } => "INVOICE_BELOW_MINIMUM",
            Self::DateOutsideFiscalYear => "DATE_OUTSIDE_FISCAL_YEAR",
            Self::AccountHasEntries { .. } => "ACCOUNT_HAS_ENTRIES",
            Self::AccountRoleAlreadyAssigned { .. } => "ACCOUNT_ROLE_ALREADY_ASSIGNED",
            Self::AccountParentArchived { .. } => "ACCOUNT_PARENT_ARCHIVED",
            Self::AccountRoleInvalidForType { .. } => "ACCOUNT_ROLE_INVALID_FOR_TYPE",
            Self::InvalidRevenueAccounts(_) => "INVOICE_LINE_REVENUE_ACCOUNT_INVALID",
            Self::CreditNoteRevenueAccountsArchived(_) => "CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED",
            Self::CreditNoteBlockedBySettlement { .. } => "CREDIT_NOTE_INVOICE_SETTLED",
            // ⚠️ Le code EXPOSÉ est celui du `ReversalBlocker` (huit valeurs) ;
            // celui-ci n'est que le repli générique du mapping structuré.
            Self::EntryNotReversable { .. } => "ENTRY_NOT_REVERSABLE",
            Self::ReversalAccountsArchived(_) => "ACCOUNT_ARCHIVED",
            Self::CreditNoteAccountsArchived(_) => "ACCOUNT_ARCHIVED",
            Self::SettlementCounterpartyIsClaimAccount { .. } => {
                "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
            }
            Self::BankAccountLedgerIsClaimAccount { .. } => "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
            Self::ClaimAccountLinkedToBankAccount { .. } => "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT",
            Self::InvoiceNotUnvalidatable { blocker, .. } => blocker.code(),
            Self::SettlementNotCancellable { blocker } => blocker.code(),
            Self::ReconciliationNotCancellable { blocker } => blocker.code(),
            Self::SupplierInvoiceNotCancellable { blocker } => blocker.code(),
            Self::InvoiceNumberFiscalYearMismatch => "INVOICE_NUMBER_FISCAL_YEAR_MISMATCH",
            Self::InvoiceMustBeUnvalidatedFirst => "INVOICE_MUST_BE_UNVALIDATED_FIRST",
            Self::EntryIsReversed => "ENTRY_IS_REVERSED",
            // ⚠️ Repli générique, comme `EntryNotReversable` : le code EXPOSÉ
            // vient du mappage `kesh-api`, qui rend `guard.code()`.
            Self::EntryNotModifiable(_) => "ENTRY_NOT_MODIFIABLE",
            Self::PeriodLocked { .. } => "PERIOD_LOCKED",
            Self::LetteringTooFewLines => "LETTERING_TOO_FEW_LINES",
            Self::LetteringTooManyLines { .. } => "LETTERING_TOO_MANY_LINES",
            Self::LetteringAccountsDiffer => "LETTERING_ACCOUNTS_DIFFER",
            Self::LetteringAccountNotLetterable => "LETTERING_ACCOUNT_NOT_LETTERABLE",
            Self::LetteringAllLinesInClosedPeriods => "LETTERING_ALL_LINES_IN_CLOSED_PERIODS",
            Self::LetteringLineOwnedByDocument { .. } => "LETTERING_LINE_OWNED_BY_DOCUMENT",
            Self::LetteringLineAlreadyLettered { .. } => "LETTERING_LINE_ALREADY_LETTERED",
            Self::LetteringUnbalanced { .. } => "LETTERING_UNBALANCED",
            Self::LetteringIsDocument => "LETTERING_IS_DOCUMENT",
            Self::LetteringConcurrentChange => "LETTERING_CONCURRENT_CHANGE",
            Self::FiscalYearInvalid => "FISCAL_YEAR_INVALID",
            Self::OpeningComplementRefused { reason, .. } => reason.code(),
            Self::ConfigurationRequired(_) => "CONFIGURATION_REQUIRED",
            Self::ConnectionUnavailable(_) => "CONNECTION_UNAVAILABLE",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::Invariant(_) => "INVARIANT_VIOLATION",
            Self::DataLengthOrRange(_) => "DATA_LENGTH_OR_RANGE",
            Self::Sqlx(_) => "DATABASE_ERROR",
        }
    }
}

/// Convertit une `sqlx::Error` en `DbError` en détectant les violations de
/// contraintes via les codes d'erreur numériques MariaDB/MySQL (stables et
/// locale-indépendants).
///
/// Codes détectés :
/// - **1062** : `ER_DUP_ENTRY` — contrainte unique
/// - **1451/1452** : violations de clé étrangère
/// - **4025** : `ER_CONSTRAINT_FAILED` (MariaDB 10.2+)
/// - **3819** : `ER_CHECK_CONSTRAINT_VIOLATED` (MySQL 8.0.16+, fallback)
/// - **1406** : `ER_DATA_TOO_LONG` — donnée trop longue pour la colonne (D2 12-5c)
/// - **1264** : `ER_WARN_DATA_OUT_OF_RANGE` — valeur hors plage du type (D2 12-5c)
///
/// Les erreurs de connexion (pool timeout, pool closed, IO) sont mappées
/// vers `DbError::ConnectionUnavailable` pour permettre un retry côté API.
pub fn map_db_error(err: sqlx::Error) -> DbError {
    // Erreurs de connexion / pool — retry-able
    match &err {
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed => {
            return DbError::ConnectionUnavailable(err.to_string());
        }
        sqlx::Error::Io(io_err) => {
            return DbError::ConnectionUnavailable(io_err.to_string());
        }
        sqlx::Error::RowNotFound => {
            return DbError::NotFound;
        }
        _ => {}
    }

    if let Some(db_err) = err.as_database_error()
        && let Some(my_err) = db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
    {
        match my_err.number() {
            1062 => return DbError::UniqueConstraintViolation(my_err.message().to_string()),
            1451 | 1452 => {
                return DbError::ForeignKeyViolation(my_err.message().to_string());
            }
            4025 | 3819 => {
                return DbError::CheckConstraintViolation(my_err.message().to_string());
            }
            // Story 12-5c (D2) : donnée trop longue / hors plage à l'INSERT
            // staging d'un QR tiers non conforme → variante typée pour un
            // échec par-fichier propre (vs 500 global) côté ingestion 12-5c.
            1406 | 1264 => {
                return DbError::DataLengthOrRange(my_err.message().to_string());
            }
            _ => {}
        }
    }
    DbError::Sqlx(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(id: i64, number: &str) -> NonPostableAccount {
        NonPostableAccount {
            account_id: id,
            account_number: number.to_string(),
        }
    }

    /// Story 15-5a (C17, C31) — ordre lexicographique du numéro, puis
    /// identifiant ; dédoublonnage par identifiant.
    #[test]
    fn non_postable_accounts_sorted_lexicographically_then_by_id_and_deduped() {
        let list = NonPostableAccounts::new([
            acc(4, "2000"),
            acc(3, "1010"),
            acc(2, "10000"),
            acc(1, "1000"),
            acc(3, "1010"), // doublon
        ]);
        assert_eq!(list.numbers(), vec!["1000", "10000", "1010", "2000"]);
        assert_eq!(list.len(), 4);

        // Numéro égal (deux sociétés ne se mélangent jamais, mais l'ordre doit
        // rester total) : départage par identifiant.
        let tie = NonPostableAccounts::new([acc(9, "1000"), acc(5, "1000")]);
        let ids: Vec<i64> = tie.iter().map(|a| a.account_id).collect();
        assert_eq!(ids, vec![5, 9]);
    }

    /// Story 15-5a (C29) — `details()` rend exactement la forme du jumeau
    /// `ACCOUNT_ARCHIVED`, dans l'ordre de la liste.
    #[test]
    fn non_postable_accounts_details_shape() {
        let list = NonPostableAccounts::new([acc(7, "2000"), acc(3, "1000")]);
        assert_eq!(
            list.details(),
            serde_json::json!({
                "rejected": [
                    { "accountId": 3, "accountNumber": "1000" },
                    { "accountId": 7, "accountNumber": "2000" },
                ]
            })
        );
    }

    /// Story 15-5a (C17) — précondition « non vide ».
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "liste des comptes refusés est vide")]
    fn non_postable_accounts_empty_panics_in_debug() {
        let _ = NonPostableAccounts::new(std::iter::empty());
    }

    #[test]
    fn accounts_not_postable_error_code() {
        let err = DbError::accounts_not_postable([acc(1, "1000")]);
        assert_eq!(err.error_code(), "ACCOUNT_NOT_POSTABLE");
    }

    /// Story 15-6b (#474) — un code pour les quatre rôles.
    #[test]
    fn settlement_counterparty_is_claim_account_error_code() {
        for role in [
            SettlementAccountRole::Counterparty,
            SettlementAccountRole::Rounding,
            SettlementAccountRole::WriteOffNature,
            SettlementAccountRole::VatPayable,
        ] {
            let err = DbError::SettlementCounterpartyIsClaimAccount {
                account_id: 1,
                account_number: Some("1100".into()),
                claim: ClaimSide::Receivable,
                role,
                batch: None,
            };
            assert_eq!(err.error_code(), "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT");
        }
    }

    /// Story 15-6c (#474) — les deux refus de configuration, un code chacun.
    #[test]
    fn configuration_claim_refusals_error_codes() {
        let ledger = DbError::BankAccountLedgerIsClaimAccount {
            account_id: 1,
            account_number: Some("1100".into()),
            claim: ClaimSide::Receivable,
        };
        assert_eq!(ledger.error_code(), "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT");
        let linked = DbError::ClaimAccountLinkedToBankAccount {
            account_id: 1,
            account_number: None,
            claim: ClaimSide::Payable,
            bank_account_id: 7,
            bank_name: "UBS".into(),
        };
        assert_eq!(linked.error_code(), "CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT");
    }

    /// Story 15-6b — les discriminants machine de `details.claim` et `details.role`.
    #[test]
    fn claim_side_and_role_discriminants() {
        assert_eq!(ClaimSide::Receivable.as_str(), "receivable");
        assert_eq!(ClaimSide::Payable.as_str(), "payable");
        assert_eq!(SettlementAccountRole::Counterparty.as_str(), "counterparty");
        assert_eq!(SettlementAccountRole::Rounding.as_str(), "rounding");
        assert_eq!(
            SettlementAccountRole::WriteOffNature.as_str(),
            "write_off_nature"
        );
        assert_eq!(SettlementAccountRole::VatPayable.as_str(), "vat_payable");
    }
}
