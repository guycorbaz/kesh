# Kesh — Messages français (Suisse)

# Erreurs d'authentification
error-invalid-credentials = Identifiants invalides
error-unauthenticated = Non authentifié
error-invalid-refresh-token = Session expirée
error-rate-limited = Trop de tentatives

# Erreurs d'autorisation
error-forbidden = Accès interdit
error-api-key-read-only = Cette clé API est en lecture seule (scope « read »). Seules les requêtes GET sont autorisées.
error-api-key-management-forbidden = La gestion des clés API n'est pas autorisée via une clé API. Utilisez l'interface web.
error-api-key-admin-forbidden = Les routes d'administration ne sont pas accessibles via une clé API, quel que soit le rôle de son créateur. Utilisez l'interface web.
error-api-key-operation-forbidden = Cette opération n'est pas accessible via une clé API. Utilisez l'interface web.
error-cannot-disable-self = Impossible de désactiver son propre compte
error-cannot-disable-last-admin = Impossible de désactiver le dernier administrateur

# Erreurs de ressource
error-not-found = Ressource introuvable
error-conflict = Ressource déjà existante
error-optimistic-lock = Conflit de version — la ressource a été modifiée
error-account-has-entries = Ce compte porte { $count } écriture(s) : changer son type reclasserait tout son historique, exercices clos compris.
error-foreign-key = Référence invalide
error-journal-entry-linked-to-invoice = Cette écriture comptable a été générée par une facture validée et ne peut pas être supprimée directement. Dévalidez d'abord la facture concernée, ou corrigez-la par un avoir.
error-check-constraint = Valeur invalide
# Story 30-2b (#685) — coordonnées de paiement d'une facture fournisseur (AC11, AC12-bis).
error-invalid-payment-details = Coordonnées de paiement invalides.
error-qr-fields-from-document = Les coordonnées de paiement viennent du QR-code du document : elles ne se saisissent pas.
# Story 30-2c (#685) — joindre un document à une facture fournisseur (AC19).
error-source-document-already-attached = Cette facture a déjà un document. Détachez-le d'abord pour en joindre un autre.
error-illegal-state = Transition d'état interdite
error-invoice-unvalidate-blocked-settled = Cette facture porte un règlement, même partiel : annulez-le d'abord.
error-credit-note-blocked-settled = Cette facture porte un règlement, même partiel : annulez-le d'abord pour pouvoir émettre un avoir.
error-settlement-counterparty-is-receivable = Le compte { $account } est le compte débiteurs de cette facture : un règlement doit faire sortir la créance vers un autre compte (banque, caisse, compensation…).
error-settlement-counterparty-is-payable = Le compte { $account } est le compte créanciers de cette facture : un règlement doit éteindre la dette par un autre compte (banque, caisse, compensation…).
error-settlement-counterparty-is-payable-in-batch = Le compte bancaire de ce lot est lié au compte { $account }, le compte créanciers de la facture { $invoice } : le lot ne peut pas être confirmé. Reliez le compte bancaire à son propre compte de banque puis confirmez de nouveau, ou annulez le lot et réglez la facture depuis sa fiche.
error-settlement-rounding-account-is-receivable = Le compte { $account }, désigné dans les réglages comme compte de différences d'arrondi, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Administration → Facturation.
error-settlement-write-off-account-is-receivable = Le compte { $account }, désigné dans les réglages comme compte de cette nature de solde, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Administration → Facturation.
error-settlement-vat-payable-account-is-receivable = Le compte { $account }, désigné dans les réglages comme compte de TVA due, est le compte débiteurs de cette facture : un administrateur doit désigner un autre compte dans Administration → Facturation.
error-bank-account-ledger-is-receivable = Le compte { $account } est le compte débiteurs désigné dans Administration → Facturation : un compte bancaire doit être lié à son propre compte de banque.
error-bank-account-ledger-is-payable = Le compte { $account } est le compte créanciers désigné dans Administration → Facturation : un compte bancaire doit être lié à son propre compte de banque.
error-claim-account-linked-to-bank-account-receivable = Le compte { $account } est lié au compte bancaire { $bank } : il ne peut pas servir de compte débiteurs.
error-claim-account-linked-to-bank-account-payable = Le compte { $account } est lié au compte bancaire { $bank } : il ne peut pas servir de compte créanciers.
error-invoice-unvalidate-blocked-credited = Cette facture est créditée par un avoir, qui en est déjà la correction.
error-invoice-unvalidate-blocked-reminders = Cette facture a un historique de rappels : le dévalider effacerait la preuve du recouvrement.
error-invoice-unvalidate-blocked-emailed = Cette facture a été envoyée au client : corrigez-la par un avoir.
error-invoice-unvalidate-blocked-matched = L'écriture de cette facture est rapprochée d'une transaction bancaire : annulez le rapprochement d'abord.
error-invoice-number-fiscal-year-mismatch = Cette facture porte déjà un numéro : sa date ne peut pas sortir de l'exercice qui l'a émis.
error-invoice-must-be-unvalidated-first = Cette facture est validée : dévalidez-la d'abord, puis supprimez le brouillon.

# Lettrage (Story 15-1a-i, #518)
error-lettering-too-few-lines = Un lettrage réunit au moins deux lignes distinctes.
error-lettering-too-many-lines = Un lettrage réunit au plus { $max } lignes.
error-lettering-proposals-too-many-lines = Trop de lignes ouvertes sur ce compte pour proposer des rapprochements : au plus { $max } lignes lettrables à la main.
error-lettering-accounts-differ = Les lignes d'un lettrage doivent toutes porter sur le même compte.
error-lettering-account-not-letterable = Ce compte ne se lettre pas : seuls les comptes d'actif et de passif qui ne sont pas des comptes bancaires se lettrent.
error-lettering-line-owned-by-document = Une de ces lignes appartient à une pièce : elle ne se lettre ni ne se délettre à la main.
error-lettering-line-already-lettered = Une de ces lignes est déjà lettrée (code { $code }).
error-lettering-unbalanced = Ces lignes ne se soldent pas : écart de { $difference }.
error-lettering-all-lines-in-closed-periods = Toutes ces lignes sont dans une période close — exercice clôturé, exercice suivi d'un exercice clôturé, ou période verrouillée : le lettrage n'y change plus.
error-lettering-is-document = Ce lettrage est celui d'une pièce : il suit la pièce et ses règlements, et ne se défait pas à la main.
error-lettering-concurrent-change = Le lettrage a changé entre-temps ; réessayez.

# Erreurs de validation
error-validation = Erreur de validation
error-email-invalid = Format d'email invalide
error-username-empty = Le nom d'utilisateur ne peut pas être vide
error-username-too-long = Le nom d'utilisateur ne doit pas dépasser { $max } caractères
error-username-contains-at = Le nom d'utilisateur ne peut pas contenir le caractère « @ »
error-email-template-unknown-variables = Le template contient des variables inconnues

# Erreurs système
error-internal = Erreur interne
error-service-unavailable = Service temporairement indisponible
db-unavailable-banner = Base de données temporairement indisponible — réessai automatique en cours

# Erreurs onboarding (Story 2.2)
error-onboarding-step-already-completed = Cette étape de configuration a déjà été complétée

# Onboarding — wizard
onboarding-choose-mode = Choisissez votre mode d'utilisation
onboarding-mode-guided = Guidé
onboarding-mode-guided-desc = Espacements généreux, aide contextuelle, confirmations avant actions
onboarding-mode-expert = Expert
onboarding-mode-expert-desc = Interface compacte, raccourcis clavier, actions directes
onboarding-choose-path = Comment souhaitez-vous commencer ?
onboarding-path-demo = Explorer avec des données de démo
onboarding-path-demo-desc = Découvrez Kesh avec des données fictives réalistes
onboarding-path-production = Configurer pour la production
onboarding-path-production-desc = Configurez votre organisation pour commencer à travailler
# Story 31-1b (#503) — restaurer une sauvegarde depuis l'assistant
onboarding-restore-hint = Vous avez une sauvegarde de Kesh (.keshbackup) ?
onboarding-restore-link = Restaurer une sauvegarde
onboarding-restore-back = ← Retour à l'assistant

# Bannière démo
demo-banner-text = Instance de démonstration — données fictives
demo-banner-reset = Réinitialiser pour la production
demo-reset-confirm-title = Réinitialiser l'instance
demo-reset-confirm-body = Toutes les données de démonstration seront supprimées. Voulez-vous continuer ?
demo-reset-confirm-ok = Confirmer
demo-reset-confirm-cancel = Annuler
demo-reset-error = Erreur lors de la réinitialisation
demo-reset-forbidden = Seul un administrateur peut réinitialiser cette instance

# Navigation sidebar (Story 6.3 + v014-1)
nav-home = Accueil
nav-contacts = Carnet d'adresses
nav-products = Catalogue
nav-invoices = Factures
nav-supplier-invoices = Factures fournisseurs
nav-payment-batches = Paiements fournisseurs
nav-invoicing-due-dates = Échéancier
nav-invoicing-reminders = Rappels
nav-settings = Paramètres
# Story v014-1 — restructuration sidebar (groupes Quotidien/Mensuel/Administration)
nav-quotidien = Quotidien
nav-mensuel = Mensuel
nav-administration = Administration
nav-accounts = Plan comptable
nav-fiscal-years = Exercices comptables
nav-opening-balances = Soldes de départ
nav-bank-accounts = Comptes bancaires
nav-bank-profiles = Profils bancaires
nav-reconciliation-rules = Règles d'affectation

# Onboarding — Chemin B (Story 2.3)
onboarding-choose-org-type = Type d'organisation
onboarding-org-independant = Indépendant
onboarding-org-independant-desc = Travailleur indépendant, freelance
onboarding-org-association = Association
onboarding-org-association-desc = Association à but non lucratif
onboarding-org-pme = PME
onboarding-org-pme-desc = Petite et moyenne entreprise (SA, Sàrl)
onboarding-choose-accounting-lang = Langue comptable
onboarding-accounting-lang-desc = Langue des libellés du plan comptable (découplée de la langue de l'interface)
onboarding-coordinates-title = Coordonnées de votre organisation
onboarding-field-name = Nom / Raison sociale
onboarding-field-address = Adresse
onboarding-field-ide = Numéro IDE
onboarding-field-ide-hint = optionnel, format CHE-xxx.xxx.xxx
onboarding-bank-title = Compte bancaire principal
onboarding-field-bank-name = Nom de la banque
onboarding-field-iban = IBAN
onboarding-field-qr-iban = QR-IBAN
onboarding-skip-bank = Configurer plus tard
onboarding-next = Continuer
onboarding-stub-name-notice = Votre entreprise a un nom provisoire — complétez vos coordonnées
incomplete-banner-text = Configuration incomplète — Terminer la configuration
incomplete-banner-cta = Terminer la configuration

# Page d'accueil (Story 2.4)
homepage-title = Tableau de bord
homepage-entries-title = Dernières écritures
homepage-entries-empty = Aucune écriture.
homepage-entries-empty-guided = Aucune écriture pour le moment. Commencez par saisir votre première écriture comptable.
homepage-entries-action = Saisir une écriture
homepage-invoices-title = Factures ouvertes
homepage-invoices-empty = Aucune facture ouverte.
homepage-invoices-empty-guided = Aucune facture ouverte. Créez une facture pour facturer vos clients.
homepage-invoices-action = Créer une facture
homepage-reminders-count = { $n } facture(s) à rappeler
homepage-bank-title = Comptes bancaires
homepage-bank-empty = Aucun compte bancaire.
homepage-bank-empty-guided = Aucun compte bancaire configuré. Ajoutez votre compte pour importer vos relevés.
homepage-bank-no-transactions = Aucune transaction importée
homepage-bank-action = Configurer

# Paramètres (Story 2.4)
settings-title = Paramètres
settings-org-title = Organisation
settings-accounting-title = Comptabilité
settings-bank-title = Comptes bancaires
settings-users-title = Utilisateurs
settings-field-name = Nom
settings-field-address = Adresse
settings-field-ide = IDE
settings-field-org-type = Type d'organisation
settings-field-instance-language = Langue de l'interface
settings-field-accounting-language = Langue comptable
search-coming-soon = Recherche bientôt disponible

# Misc i18n (Story 2.4 review)
loading = Chargement...
settings-edit = Modifier
settings-edit-coming-soon = Édition bientôt disponible
settings-manage = Gérer
settings-no-bank = Aucun compte bancaire configuré.
settings-no-company = Aucune organisation configurée. Complétez l'onboarding.

# Plan comptable (Story 3.1)
accounts-title = Plan comptable
accounts-add = Nouveau compte
accounts-edit = Modifier le compte
accounts-archive = Archiver
accounts-archive-confirm = Le compte ne sera plus disponible dans les sélections futures, mais restera visible dans les écritures existantes.
account-field-number = Numéro
account-field-name = Nom
account-field-type = Type
account-field-parent = Compte parent
account-type-asset = Actif
account-type-liability = Passif
account-type-revenue = Produit
account-type-expense = Charge
account-archived-label = Archivé
# --- Story 14-3a : rôles de comptes & postabilité ---
account-field-role = Rôle
account-role-none = Aucun
account-role-receivable = Créances clients
account-role-default-revenue = Produit par défaut
account-role-payable = Dettes fournisseurs
account-role-vat-recoverable = Impôt préalable (TVA récupérable)
account-role-vat-payable = TVA due
account-role-vat-settlement = Décompte TVA
account-role-equity-capital = Capital
account-role-equity-other = Autres fonds propres
account-role-retained-earnings = Bénéfice/perte reporté
account-role-current-year-result = Résultat de l'exercice
account-role-archived-hint = Rôle inactif — ce compte est archivé
account-field-postable = Postable
account-postable-no = Non postable
account-postable-hint = Un compte non postable n'accepte pas de saisie d'écriture manuelle
accounts-reactivate-aria = Réactiver le compte { $number }
accounts-reactivated = Compte { $number } réactivé
accounts-role-conflict = Le rôle est déjà attribué au compte { $number } — { $name }. Retirez-le d'abord de ce compte.
accounts-error-number-required = Le numéro est requis.
accounts-error-name-required = Le nom est requis.
accounts-error-number-exists = Ce numéro de compte existe déjà.
accounts-error-stale = La page n'est plus à jour. Rechargez-la avant de réessayer.
accounts-created = Compte { $number } créé
accounts-updated = Compte { $number } modifié
accounts-archived = Compte { $number } archivé
accounts-count = { $count } comptes
accounts-show-archived = Afficher les comptes archivés
# Story 14-3a — code review : libellés des dialogs et erreurs de rôle
accounts-create-description = Ajoutez un compte au plan comptable.
accounts-edit-title = Modifier le compte { $number }
accounts-edit-description = Le numéro n'est pas modifiable après création.
accounts-archive-title = Archiver le compte { $number } ?
accounts-archiving = Archivage…
accounts-updating = Modification…
accounts-retype-title = Changer le type du compte { $number } ?
accounts-retype-warning = Ce compte porte { $count } écriture(s). Changer son type reclasse tout son historique : les montants passent du compte de résultat au bilan, ou l’inverse.
accounts-retype-closed-years = Exercices clos concernés : { $years }. Leur résultat changera sans qu’aucune écriture ne l’explique.
accounts-retype-confirm = Changer le type quand même
account-field-parent-optional = Compte parent (optionnel)
accounts-parent-none = Aucun
accounts-parent-archived = Le compte parent { $number } est archivé. Réactivez-le d'abord.
accounts-role-invalid-for-type = Le rôle { $role } ne peut pas être attribué à un compte de type { $type }.
accounts-role-conflict-generic = Ce rôle vient d'être attribué à un autre compte. Rechargez la page.
accounts-reactivate-without-role = Réactiver sans le rôle
accounts-reactivate-without-role-description = Le rôle de ce compte a été repris par un autre compte. Vous pouvez le réactiver sans son rôle — il restera modifiable ensuite.
accounts-reactivating = Réactivation…
common-empty = Aucun élément trouvé.
common-create = Créer
common-creating = Création…
common-saving = Enregistrement…


# Mode Guidé/Expert (Story 2.5)
mode-guided-label = Guidé
mode-expert-label = Expert
shortcut-new-entry = Ctrl+N : Nouvelle écriture

# Écritures comptables (Story 3.2)
error-entry-unbalanced = Écriture déséquilibrée — le total des débits ({ $debit }) ne correspond pas au total des crédits ({ $credit })
error-no-fiscal-year = Aucun exercice n'existe pour la date { $date }. Créez un exercice comptable avant de saisir des écritures.
error-fiscal-year-closed = L'exercice pour la date { $date } est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964).
journal-entries-title = Écritures comptables
journal-entries-new = Nouvelle écriture
journal-entries-empty-list = Aucune écriture saisie pour l'instant
journal-entries-col-number = N°
journal-entries-col-date = Date
journal-entries-col-journal = Journal
journal-entries-col-description = Libellé
journal-entries-col-total = Total
journal-entry-form-title = Saisie d'écriture
journal-entry-form-date = Date
journal-entry-form-journal = Journal
journal-entry-form-col-project = Projet
journal-entry-project-archived = Projet archivé
journal-entry-project-none = — Aucun
journal-entry-form-description = Libellé
journal-entry-form-add-line = + Ajouter une ligne
journal-entry-form-remove-line = Retirer cette ligne
journal-entry-form-col-account = Compte
journal-entry-form-col-debit = Débit
journal-entry-form-col-credit = Crédit
journal-entry-form-total-debit = Total débits
journal-entry-form-total-credit = Total crédits
journal-entry-form-diff = Différence
journal-entry-form-balanced = Équilibré
journal-entry-form-unbalanced = Déséquilibré
journal-entry-form-submit = Valider
journal-entry-form-cancel = Annuler
journal-entry-form-incomplete-line = Ligne incomplète
journal-entry-form-max-decimals = Maximum 4 décimales
journal-entry-form-amount-too-large = Montant trop élevé
account-autocomplete-unavailable = Autocomplétion indisponible — saisir l'ID du compte
journal-achats = Achats
journal-ventes = Ventes
journal-banque = Banque
journal-caisse = Caisse
journal-od = OD
journal-entry-saved = Écriture enregistrée
journal-entry-edit = Modifier
journal-entry-delete = Supprimer
journal-entry-delete-confirm-title = Supprimer l'écriture n° { $number } ?
journal-entry-delete-confirm-message = Son numéro ne sera pas réattribué : un trou restera dans la numérotation. La suppression sera inscrite au journal d'audit, avec le contenu complet de l'écriture.
journal-entry-delete-confirm-cancel = Annuler
journal-entry-delete-confirm-delete = Supprimer
journal-entry-deleted = Écriture supprimée
journal-entry-modified = Modifiée
journal-entry-history = Historique
error-fiscal-year-closed-generic = L'exercice comptable est clôturé — aucune écriture ne peut y être ajoutée, modifiée ou supprimée (CO art. 957-964).
error-inactive-accounts = Un ou plusieurs comptes sont archivés ou invalides.
error-account-not-postable = { $count ->
    [one] Le compte { $numbers } n’est pas imputable (compte de regroupement, de résultat ou de clôture) : choisissez un compte imputable.
   *[other] Les comptes { $numbers } ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : choisissez des comptes imputables.
}
error-designated-account-not-postable = { $count ->
    [one] Le compte { $numbers }, désigné dans Administration → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable.
   *[other] Les comptes { $numbers }, désignés dans Administration → Facturation, ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à leur place des comptes imputables.
}
error-rounding-account-not-configured = Ce paiement solde la facture au centime, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Administration → Facturation.
error-rounding-account-not-configured-issuance = Le total de cette pièce est arrondi à 5 centimes, mais aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Administration → Facturation.
error-write-off-account-not-configured-discount = Aucun compte d'escompte utilisable n'est désigné : choisissez-en un dans Administration → Facturation, section Solde du reste.
error-write-off-account-not-configured-bank-fees = Aucun compte de frais bancaires utilisable n'est désigné : choisissez-en un dans Administration → Facturation, section Solde du reste.
error-write-off-account-not-configured-bad-debt = Aucun compte de pertes sur créances utilisable n'est désigné : choisissez-en un dans Administration → Facturation, section Solde du reste.
error-write-off-account-not-configured-rounding = Aucun compte de différences d'arrondi utilisable n'est désigné : choisissez-en un dans Administration → Facturation.
error-invoice-below-minimum = Le total de cette facture, CHF { $total }, est inférieur au montant minimum fixé dans Administration → Facturation, CHF { $minimum }.

# Compte de produit par ligne de facture (Story 16-1a, #152)
invoice-line-account-subject-line = Ligne { $line }
invoice-line-account-subject-default = le compte de produit par défaut de la société
invoice-line-account-unknown = { $subject } : le compte sélectionné est introuvable ou n'appartient pas à cette société
invoice-line-account-inactive = { $subject } : le compte { $number } est archivé
invoice-line-account-not-revenue = { $subject } : le compte { $number } n'est pas un compte de produit
invoice-line-account-not-postable = { $subject } : le compte { $number } n'est pas imputable — choisissez un autre compte
invoice-line-revenue-account-invalid = Compte de produit invalide — { $detail }
credit-note-revenue-account-archived = Impossible d'émettre l'avoir — { $detail }. Réactivez le ou les comptes concernés.
credit-note-account-archived = Impossible d'émettre l'avoir — compte(s) archivé(s) : { $detail }. Réactivez le ou les comptes concernés.
invoice-error-total-zero = Cette facture est d'un montant total nul : elle ne peut pas être validée. Renseignez au moins une ligne avec un prix unitaire supérieur à zéro.
credit-note-error-total-zero = Cette facture est d'un montant total nul : aucun avoir ne peut être émis.

# Modification & suppression d'écritures (Story 3.3)
journal-entry-open = Ouvrir la fiche

# Assistant TVA achat (Story 18-1c)
vat-purchase-title = Assistant TVA achat
vat-purchase-config-required = Configurez le compte d'impôt préalable dans Administration → Facturation pour utiliser l'assistant.
vat-purchase-no-rates = Aucun taux de TVA configuré — voir Paramètres → Taux de TVA.
vat-purchase-charge-account = Compte de charge
vat-purchase-ht = Montant HT
vat-purchase-rate = Taux TVA
vat-purchase-rate-placeholder = Choisir un taux
vat-purchase-counterparty = Compte de contrepartie
vat-purchase-same-account = Le compte de charge et la contrepartie doivent être différents.
vat-purchase-recoverable-conflict = Le compte de charge et la contrepartie ne peuvent pas être le compte d'impôt préalable.
vat-purchase-insert = Insérer les lignes
vat-purchase-description = Achat — TVA { $rate } % récupérable
vat-purchase-description-exempt = Achat — sans TVA
vat-purchase-replace-title = Remplacer le brouillon ?
vat-purchase-replace-message = Des lignes ou un libellé ont déjà été saisis. Continuer écrasera le brouillon actuel.
vat-purchase-replace-confirm = Remplacer
error-date-outside-fiscal-year = La date { $date } n'est pas dans l'exercice courant de cette écriture
error-date-outside-fiscal-year-generic = La date n'est pas dans l'exercice courant de cette écriture

# Recherche, pagination, tri (Story 3.4)
journal-entries-filter-description = Libellé
journal-entries-filter-amount-min = Montant min
journal-entries-filter-amount-max = Montant max
journal-entries-filter-date-from = Date début
journal-entries-filter-date-to = Date fin
journal-entries-filter-journal = Journal
journal-entries-filter-journal-all = Tous
journal-entries-filter-account = Compte
journal-entries-filter-account-all = Tous
journal-entries-filter-reset = Réinitialiser
journal-entries-pagination-on = sur
journal-entries-pagination-prev = Précédent
journal-entries-pagination-next = Suivant
journal-entries-pagination-page-size = Par page
journal-entries-sort-asc-indicator = tri ascendant
journal-entries-sort-desc-indicator = tri descendant
journal-entries-loading = Chargement…

# Story 24-4a (#380) — la contre-passation d'une écriture.
journal-entries-reverse-action = Contre-passer
journal-entries-reverse-dialog-title = Contre-passer cette écriture ?
journal-entries-reverse-dialog-body = Kesh créera une écriture inverse à la date du jour. L'écriture d'origine reste intacte dans ses montants, ses comptes, sa date et son libellé — seules ses lignes reçoivent, le cas échéant, la marque de lettrage qui les apparie à l'écriture inverse : c'est la correction qui doit se voir, pas disparaître.
journal-entries-reverse-confirm = Contre-passer
journal-entries-reverse-cancel = Annuler
journal-entries-reverse-success = Écriture contre-passée
journal-entries-reverses-link = Contre-passe l'écriture n° { $number }
journal-entries-reversed-by-link = Contre-passée par l'écriture n° { $number }
# Story 30-2c (#685, AC21) — l'écriture d'une facture fournisseur montre son document.
journal-entries-supplier-invoice-link = Facture fournisseur { $number }
journal-entries-document-view = Voir le document
journal-entries-document-preview-close = Fermer l’aperçu
journal-entries-column-lettering = Lettrage
journal-entries-reverse-blocked-is-a-reversal = Cette écriture est elle-même une contre-passation.
journal-entries-reverse-blocked-already-reversed = Cette écriture a déjà été contre-passée.
journal-entries-reverse-blocked-invoice = Cette écriture appartient à une facture client : dévalidez la facture, ou corrigez-la par un avoir.
journal-entries-reverse-blocked-credit-note = Cette écriture est celle d'un avoir, qui est déjà une contre-passation.
journal-entries-reverse-blocked-supplier-invoice = Cette écriture appartient à une facture fournisseur : elle se corrige depuis la fiche de la facture, qui indique ce qui est possible.
journal-entries-reverse-blocked-settlement = Cette écriture est un règlement de facture : annulez le règlement depuis la fiche de la facture, qui indique si c'est possible.
journal-entries-reverse-blocked-bank-match = Cette écriture est rapprochée d'une transaction bancaire : annulez le rapprochement depuis le détail de l'import bancaire.
journal-entries-reverse-blocked-account-archived = Un compte de cette écriture a été archivé : réactivez-le pour pouvoir la contre-passer.
journal-entries-reverse-account-archived = Impossible de contre-passer — compte(s) archivé(s) : { $detail }. Réactivez le ou les comptes concernés.
# Story 29-1b (#232) — contre-passer au début de l'exercice suivant (transitoires).
journal-entries-reverse-next-year-action = Contre-passer au début de l’exercice suivant
journal-entries-reverse-next-year-dialog-title = Contre-passer au début de l’exercice suivant ?
journal-entries-reverse-next-year-dialog-body = La contre-passation de l’écriture n° { $number } « { $description } » sera datée du { $date }, dans l’exercice « { $name } ».
journal-entries-reverse-next-year-help = Pour les transitoires de fin d’exercice : l’écriture inverse annule la régularisation au premier jour de l’exercice suivant. L’écriture d’origine reste intacte ; ni elle ni sa contre-passation ne pourront plus être modifiées ni supprimées — une erreur se corrigera par une écriture complémentaire.
journal-entries-reverse-next-year-reason-unavailable = La liste des exercices n’a pas pu être lue : rechargez la page pour contre-passer au début de l’exercice suivant.
journal-entries-reverse-next-year-reason-missing = Pour contre-passer au début de l’exercice suivant, créez d’abord l’exercice suivant.
journal-entries-reverse-next-year-reason-closed = L’exercice suivant, « { $name } », est clôturé : la contre-passation ne peut pas y être datée.
journal-entries-delete-blocked-reversed = Cette écriture a été contre-passée : elle ne peut plus être modifiée ni supprimée.
journal-entries-edit-conflict = Cette écriture a été modifiée entre-temps : la fiche a été rechargée.
journal-entries-line-account-unusable = Compte archivé ou non imputable — à remplacer
journal-entries-modify-blocked-fiscal-year-closed = L’exercice de cette écriture est clôturé : elle est figée. Corrigez-la par une contre-passation.
journal-entries-modify-blocked-later-fiscal-year-closed = L’exercice postérieur { $name } est clôturé, et son bilan reprend cette écriture : elle reste figée tant qu’il l’est. Corrigez-la par une contre-passation ; sinon, un administrateur rouvre les exercices postérieurs clôturés, en commençant par le plus récent.
error-later-fiscal-year-closed = L’exercice « { $name } », postérieur, est clôturé, et son bilan reprend tout ce qui le précède : aucune écriture datée avant sa date de début ne peut être enregistrée, modifiée ni supprimée tant qu’il l’est. Une écriture existante se corrige par une contre-passation ; sinon, un administrateur rouvre les exercices postérieurs clôturés, en commençant par le plus récent.
journal-entries-modify-blocked-period-locked = La période est verrouillée jusqu’au { $date } : cette écriture, datée dans la période, reste figée. Corrigez-la par une contre-passation.
journal-entries-modify-blocked-detached-settlement = Ce paiement appartient à une facture fournisseur annulée : l’argent est sorti, il reste figé. Corrigez-le par une contre-passation.
journal-entries-modify-blocked-lettered = Cette écriture est lettrée : délettrez-la d’abord.
journal-entries-period-locked = Les écritures sont verrouillées jusqu'au { $lockedThrough } ; celle-ci est datée du { $attempted }.
settings-books-lock-title = Verrou de période
settings-books-lock-none = Aucun verrou : toutes les dates d'un exercice ouvert sont acceptées.
settings-books-lock-current = Livres verrouillés jusqu'au { $date } inclus.
settings-books-lock-set = Verrouiller jusqu'au
settings-books-lock-release = Déverrouiller
settings-books-lock-motif = Motif du déverrouillage
settings-books-lock-motif-required = Un motif est obligatoire pour déverrouiller.
settings-books-lock-bound-not-past = La borne du verrou doit être antérieure à aujourd'hui : verrouiller le jour même refuserait toute correction faite aujourd'hui.
journal-entries-books-locked-banner = Livres verrouillés jusqu'au { $date } inclus — aucune écriture ne peut y être datée.

# Tooltips bilingues termes comptables (Story 3.5)
tooltip-debit-natural = L'argent entre dans ce compte
tooltip-debit-technical = Débit — colonne de gauche
tooltip-credit-natural = L'argent sort de ce compte
tooltip-credit-technical = Crédit — colonne de droite
tooltip-journal-natural = Registre où sont groupées les écritures similaires
tooltip-journal-technical = Journal comptable (Achats, Ventes, Banque, Caisse, OD)
tooltip-balanced-natural = Le total des entrées égale le total des sorties
tooltip-balanced-technical = Équilibre partie double (débit = crédit)

# Story 4.1 — Carnet d'adresses (contacts CRUD)
contacts-page-title = Carnet d'adresses
contact-form-create-title = Nouveau contact
contact-form-edit-title = Modifier le contact
contact-form-name = Nom / Raison sociale
contact-form-type = Type
contact-form-is-client = Client
contact-form-is-supplier = Fournisseur
contact-form-email = Email
contact-form-phone = Téléphone
contact-form-address = Adresse
contact-form-ide = Numéro IDE (CHE)
contact-form-ide-help = Format : CHE-123.456.789
# Story 22-2b (#301) — sondes anti-doublon à la saisie.
contact-duplicate-heading = Contacts déjà enregistrés qui pourraient correspondre
contact-duplicate-others-count-one = et 1 autre
contact-duplicate-others-count = et { $count } autres
contact-duplicate-ide-active = Ce numéro IDE est déjà porté par { $name }.
contact-duplicate-ide-archived = Ce numéro IDE est déjà porté par { $name }, qui est archivé. Un IDE reste réservé même après archivage : l'enregistrement sera refusé.
contact-form-client-number = Numéro de client
contact-form-client-number-hint = Figure sur le PDF de facture, pour que votre client rapproche de son dossier fournisseur.
contact-type-personne = Personne
contact-type-entreprise = Entreprise
contact-form-submit-create = Créer
contact-form-submit-edit = Enregistrer
contact-form-cancel = Annuler
contact-list-new = Nouveau contact
contact-list-edit = Modifier
contact-list-archive = Archiver
contact-archive-confirm = Archiver
contact-archive-cancel = Annuler
contact-col-name = Nom
contact-col-type = Type
contact-col-flags = Rôles
contact-col-ide = IDE
contact-col-email = Email
contact-col-actions = Actions
contact-filter-search-placeholder = Rechercher par nom, email, n° client ou IDE sans séparateurs…
contact-filter-type-all = Tous les types
contact-filter-archived = Inclure archivés
contact-empty-list = Aucun contact. Créez votre premier contact avec le bouton « Nouveau contact ».
contact-created-success = Contact créé
contact-updated-success = Contact modifié
contact-archived-success = Contact archivé
contact-archive-confirm-title = Archiver le contact ?
contact-archive-confirm-body = Le contact ne sera plus visible dans la liste par défaut. Vous pourrez toujours le consulter en activant « Inclure archivés ».
contact-error-name-required = Le nom est obligatoire
contact-error-name-too-long = Le nom doit faire au plus 255 caractères
contact-error-email-invalid = Format d'email invalide
contact-error-ide-invalid = Numéro IDE suisse invalide (format ou checksum)
contact-error-payment-terms-days-range = Le délai de paiement doit être un nombre entier entre 0 et 365 jours
contact-error-ide-duplicate = Un contact avec ce numéro IDE existe déjà
contact-error-client-number-duplicate = Un contact avec ce numéro de client existe déjà
contact-error-not-found = Contact introuvable
contact-error-archived-no-modify = Contact archivé — modification ou archivage supplémentaire interdit
contact-conflict-title = Conflit de version
contact-conflict-body = Ce contact a été modifié ailleurs. Voulez-vous recharger la version actuelle ?
error-ide-already-exists = Un contact avec ce numéro IDE existe déjà

# Story 4.2 — Conditions de paiement & catalogue produits
contact-form-payment-terms = Conditions de paiement
contact-form-payment-terms-placeholder = ex: 30 jours net
contact-payment-terms-days-label = { $days ->
    [one] Payable à { $days } jour net
   *[other] Payable à { $days } jours net
}
contact-payment-terms-immediate-label = Payable au comptant
contact-form-payment-terms-days = Délai de paiement (jours)
contact-form-payment-terms-days-hint = L'échéance des factures sera pré-calculée et le libellé des conditions généré automatiquement.
contact-form-payment-terms-disabled-hint = Libellé généré automatiquement depuis le délai de paiement.
products-page-title = Catalogue produits/services
product-form-create-title = Nouveau produit
product-form-edit-title = Modifier le produit
product-form-name = Nom
product-form-description = Description
product-form-price = Prix unitaire
product-form-vat-rate = Taux TVA
product-form-vat-help = Taux suisses en vigueur depuis le 01.01.2024
product-vat-exempt = 0,00 % — Exonéré
product-vat-reduced = 2,60 % — Taux réduit
product-vat-special = 3,80 % — Hébergement
product-vat-normal = 8,10 % — Taux normal
product-list-new = Nouveau produit
product-list-edit = Modifier
product-list-archive = Archiver
product-col-name = Nom
product-col-description = Description
product-col-price = Prix
product-col-vat = TVA
product-col-actions = Actions
product-filter-search = Rechercher par nom ou description…
product-filter-archived = Inclure archivés
product-empty-list = Aucun produit. Créez votre premier produit avec le bouton « Nouveau produit ».
product-created-success = Produit créé
product-updated-success = Produit modifié
product-archived-success = Produit archivé
product-error-name-required = Le nom est obligatoire
product-error-name-too-long = Le nom doit faire au plus 255 caractères
product-error-price-required = Le prix est obligatoire
product-error-price-negative = Le prix doit être positif ou nul
product-error-price-invalid = Format de prix invalide
product-error-vat-invalid = Taux TVA non autorisé
product-error-vat-loading = Chargement des taux TVA en cours, veuillez patienter…
product-error-vat-fetch-failed = Impossible de charger les taux TVA. Vérifiez la connexion réseau et rechargez la page.
product-error-name-duplicate = Un produit avec ce nom existe déjà
product-archive-confirm-title = Archiver le produit ?
product-archive-confirm-body = Le produit ne sera plus visible dans la liste par défaut. Vous pourrez toujours le consulter en activant « Inclure archivés ».
product-conflict-title = Conflit de version
product-conflict-body = Ce produit a été modifié ailleurs. Voulez-vous recharger la version actuelle ?
product-filter-reset = Réinitialiser
product-pagination-prev = Précédent
product-pagination-next = Suivant
product-pagination-of = sur
product-conflict-reload = Recharger
product-form-cancel = Annuler
product-form-submit-create = Créer
product-form-submit-edit = Enregistrer
product-archive-cancel = Annuler
product-archive-confirm = Archiver

# --- Story 5.1 : Factures brouillon ---
invoices-page-title = Factures
invoices-settings-vat-accounts-title = Comptes TVA
invoices-settings-vat-accounts-hint = Comptes utilisés pour la comptabilisation de la TVA (préparé pour le décompte AFC).
invoices-settings-vat-payable = Compte TVA due (Passif)
invoices-settings-vat-recoverable = Compte TVA récupérable (Actif)
invoices-settings-vat-decompte = Compte de décompte TVA (Passif)
invoices-description-error-empty = Le libellé est vide
invoices-description-error-too-long = Le libellé dépasse { $max } caractères (actuel : { $actual })
invoices-description-error-control-char = Caractère de contrôle non autorisé
invoices-description-error-unknown-placeholder = Placeholder inconnu : {"{"}{ $name }{"}"}
invoices-description-error-no-placeholder = Le libellé doit contenir au moins un placeholder reconnu ({"{"}YEAR{"}"}, {"{"}INVOICE_NUMBER{"}"}, {"{"}CONTACT_NAME{"}"})
invoices-format-error-empty = Le format de numérotation est vide
invoices-format-error-too-long = Le format dépasse { $max } caractères (actuel : { $actual })
invoices-format-error-bad-chars = Le format contient des caractères non autorisés
invoices-format-error-bad-padding = Padding {"{"}SEQ:{ $n }{"}"} invalide — doit être entre 1 et { $max }
invoices-format-error-unknown-placeholder = Placeholder inconnu : {"{"}{ $name }{"}"}
invoices-format-error-no-placeholder = Le format doit contenir au moins un placeholder reconnu ({"{"}YEAR{"}"}, {"{"}FY{"}"}, {"{"}SEQ{"}"}, {"{"}SEQ:NN{"}"})
invoices-format-error-rendered-too-long = Le format générerait un numéro de { $len } caractères (max { $max })
invoice-new-title = Nouvelle facture
invoice-edit-title = Modifier la facture
invoice-view-title = Facture
invoice-form-contact = Contact
invoice-form-date = Date
invoice-form-due-date = Échéance
invoice-form-payment-terms = Conditions de paiement
invoice-form-status = Statut
invoice-form-number = N° de facture
invoice-line-description = Description
invoice-line-quantity = Quantité
invoice-line-unit-price = Prix unitaire
invoice-line-vat-rate = TVA %
invoice-line-total = Total
invoice-line-actions = Actions
invoice-add-free-line = Ligne libre
invoice-add-from-catalog = Depuis catalogue
invoice-col-date = Date
invoice-col-contact = Contact
invoice-col-number = N°
invoice-col-status = Statut
invoice-col-total = Total
invoice-col-actions = Actions
invoice-status-draft = Brouillon
invoice-status-validated = Validée
invoice-status-cancelled = Annulée
invoice-filter-search = Rechercher…
invoice-filter-status-all = Tous les statuts
invoice-filter-contact-all = Tous les contacts
invoice-filter-date-from = Depuis
invoice-filter-date-to = Jusqu'à
# Story 21-6a (D10) — suspension des rappels : badge + filtre en liste factures.
invoice-paused-badge = Suspendu
invoice-paused-filter-label = Rappels
invoice-paused-filter-all = Tous
invoice-paused-filter-paused = Suspendus
invoice-paused-filter-not-paused = Actifs
invoice-new-button = Nouvelle facture
invoice-edit-button = Modifier
invoice-delete-button = Supprimer
invoice-subtotal = Sous-total
invoice-total = Total
invoice-empty-list = Aucune facture. Créez votre première facture avec le bouton « Nouvelle facture ».
invoice-created-success = Facture créée
invoice-updated-success = Facture modifiée
invoice-deleted-success = Facture supprimée
invoice-delete-confirm-title = Supprimer la facture ?
invoice-delete-confirm-body = Cette facture brouillon sera supprimée définitivement.
invoice-conflict-title = Conflit de version
invoice-conflict-body = Cette facture a été modifiée ailleurs. Voulez-vous recharger la version actuelle ?
invoice-error-no-lines = Une facture doit contenir au moins une ligne
invoice-error-contact-required = Veuillez sélectionner un contact
invoice-error-contact-invalid = Contact introuvable
invoice-error-quantity-positive = La quantité doit être strictement positive
invoice-error-description-required = La description est obligatoire
invoice-error-vat-invalid = Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10%
invoice-error-illegal-state = Cette facture ne peut plus être modifiée
invoice-product-picker-title = Sélectionner un produit
invoice-product-picker-search = Rechercher un produit…
invoice-product-picker-empty = Aucun produit
invoice-contact-picker-placeholder = Rechercher un contact…
invoice-contact-picker-empty = Aucun contact

# Story 5.2 — Validation & numérotation des factures
error-fiscal-year-invalid = Aucun exercice ouvert ne couvre cette date.
error-configuration-required = Configuration incomplète : configurez les paramètres de facturation avant de valider.
invoice-validate-button = Valider
invoice-validate-confirm-title = Valider la facture
invoice-validate-confirm-body = Une fois validée, cette facture sera immuable, recevra un numéro définitif et générera une écriture comptable. Continuer ?
invoice-validate-success = Facture validée — { $invoiceNumber }
invoice-validate-success-body = La facture { $invoiceNumber } est désormais validée et immuable. L'écriture comptable associée a été générée.
invoice-error-fiscal-year-invalid = Aucun exercice ouvert ne couvre la date de la facture.
invoice-error-configuration-required = Configurez les comptes par défaut dans Administration → Facturation avant de valider une facture.
invoice-error-configuration-required-non-admin = Demandez à votre administrateur de configurer les comptes par défaut de facturation.
invoice-error-already-validated = Cette facture est déjà validée.
invoice-number-label = Numéro
invoice-status-validated-label = Validée
invoice-view-journal-entry-link = Voir l'écriture comptable
settings-invoicing-title = Paramètres — Facturation
settings-invoicing-format-label = Format de numérotation
settings-invoicing-format-help = Placeholders : {"{"}YEAR{"}"}, {"{"}FY{"}"}, {"{"}SEQ{"}"}, {"{"}SEQ:NN{"}"}
settings-invoicing-format-preview = Aperçu
settings-invoicing-receivable-account = Compte créance client (Actif)
settings-invoicing-payable-account = Compte créanciers (Passif)
settings-invoicing-revenue-account = Compte produit (Revenue)
settings-invoicing-journal = Journal
settings-invoicing-description-template = Libellé de l'écriture comptable
settings-invoicing-save = Enregistrer
settings-invoicing-save-success = Configuration enregistrée
settings-invoicing-format-invalid = Format invalide
settings-invoicing-numbering-title = Numérotation
settings-invoicing-seq-range = (NN entre 1 et 10)
settings-invoicing-description-help = Placeholders : {"{"}YEAR{"}"}, {"{"}INVOICE_NUMBER{"}"}, {"{"}CONTACT_NAME{"}"}.
settings-invoicing-description-invalid = Libellé invalide
settings-invoicing-rounding-title = Différences d'arrondi
settings-invoicing-round5-label = Arrondir le total des factures émises à 5 centimes
settings-invoicing-round5-hint = La facture porte une ligne « Arrondi » (123.44 → 123.45) ; l'écart s'écrit sur le compte ci-dessous. Les factures déjà émises gardent leur total.
settings-invoicing-minimum-title = Montant minimum
settings-invoicing-minimum-label = Montant minimum d'une facture
settings-invoicing-minimum-hint = Une facture dont le total est inférieur ne peut pas être validée. Laissez vide pour n'appliquer aucun seuil. Les avoirs ne sont pas concernés.
settings-invoicing-write-off-title = Solde du reste
settings-invoicing-write-off-hint = Comptes qui reçoivent ce qui reste dû quand vous soldez une facture : l'escompte accordé au client, les frais retenus par sa banque, la perte sur un débiteur. Charge ou produit, imputable. Les plans livrés en proposent un pour chaque nature, sauf l'escompte pour les associations.
settings-invoicing-discount-account = Compte d'escompte accordé
settings-invoicing-bank-fees-account = Compte de frais bancaires
settings-invoicing-bad-debt-account = Compte de pertes sur créances
settings-invoicing-rounding-hint = Compte qui reçoit les écarts d'arrondi : celui des factures arrondies à 5 centimes, et le demi-centime au plus qu'un paiement arrondi au centime laisse sur une facture émise sans arrondi. Charge ou produit, imputable. S'il n'existe pas encore, créez-le dans le plan comptable, puis choisissez-le ici.
settings-invoicing-rounding-account = Compte de différences d'arrondi
settings-invoicing-default-accounts-title = Comptes par défaut
settings-invoicing-select-none = — Sélectionner —
settings-invoicing-load-error = Erreur de chargement
settings-invoicing-save-error = Erreur lors de la sauvegarde
invoice-journal-entry-description = Facture { $invoiceNumber } - { $contactName }

# --- Story 5.3 — Génération PDF QR Bill ---

# Libellés affichés dans la partie facture du PDF (25 clés)
invoice-pdf-title = Facture
invoice-pdf-date = Date
invoice-pdf-due-date = Échéance
invoice-pdf-number = N° de facture
invoice-pdf-origin-reference = Réf. facture d'origine
credit-note-pdf-title = Avoir
credit-note-pdf-number = N° d'avoir
invoice-pdf-ide = IDE
# Story 16-3a (#151) — coordonnées de contact de l'émetteur sur le PDF.
# ⚠️ Ces trois clés vivent AUSSI dans `I18N_KEYS`/`DEFAULT_EN` (kesh-qrbill).
invoice-pdf-phone = Tél.
invoice-pdf-email = E-mail
invoice-pdf-website = Web
invoice-pdf-client-number = N° client
# Story 25-4-b2 (#416) — le PDF joint à un rappel.
invoice-pdf-reminder-title = Rappel
invoice-pdf-settled = Déjà réglé
invoice-pdf-amount-due = Reste à payer
invoice-pdf-reminder-fees = Frais de rappel
invoice-pdf-reminder-fees-note = Les frais de rappel ne sont pas compris dans le bulletin de versement.
invoice-pdf-recipient = Destinataire
invoice-pdf-description = Description
invoice-pdf-quantity = Qté
invoice-pdf-unit-price = Prix unitaire
invoice-pdf-vat = TVA
invoice-pdf-line-total = Total
invoice-pdf-subtotal = Sous-total
invoice-pdf-total = Total
invoice-pdf-rounding = Arrondi
invoice-pdf-total-ttc = Total TTC
invoice-pdf-payment-terms = Conditions de paiement
invoice-pdf-qr-section-payment = Section paiement
invoice-pdf-qr-section-receipt = Récépissé
invoice-pdf-qr-account = Compte / Payable à
invoice-pdf-qr-reference = Référence
invoice-pdf-qr-additional-info = Informations supplémentaires
invoice-pdf-qr-payable-by = Payable par
invoice-pdf-qr-currency = Monnaie
invoice-pdf-qr-amount = Montant
invoice-pdf-qr-acceptance-point = Point de dépôt
invoice-pdf-qr-separate-before-paying = A détacher avant le versement

# Messages d'erreur PDF (6 clés — codes applicatifs + causes détaillées)
invoice-pdf-error-invoice-not-validated = La facture doit être validée avant de pouvoir être générée en PDF.
error-invoice-cancelled = La facture est annulée par un avoir : son PDF ne peut pas être produit, et aucun document n'a été figé.
error-invoice-pdf-gone = Le PDF émis de cette facture est introuvable : le fichier { $sha256 }.pdf manque dans le répertoire des documents (KESH_DOCUMENTS_DIR). Restaurez-le depuis la sauvegarde de ce répertoire ; à défaut, un administrateur peut refiger la facture.
error-invoice-pdf-gone-cancelled = Le PDF émis de cette facture annulée est introuvable : le fichier { $sha256 }.pdf manque dans le répertoire des documents (KESH_DOCUMENTS_DIR). Seule sa restauration depuis la sauvegarde de ce répertoire le répare.
error-invoice-changed = La facture a été modifiée pendant la préparation de son PDF. Réessayez.
error-invoice-pdf-present = Le PDF émis de cette facture est présent : il n'y a rien à refiger.
error-invoice-pdf-not-frozen = Cette facture n'a pas encore de PDF figé : son prochain téléchargement le figera.
error-invoice-pdf-integrity = Le fichier du PDF émis a été altéré : son empreinte ne correspond plus. Restaurez-le depuis la sauvegarde, ou écartez-le du répertoire des documents avant de refiger.
invoice-pdf-error-invoice-not-pdf-ready = La facture n'est pas prête pour la génération PDF.
invoice-pdf-error-pdf-generation-failed = Échec de la génération du PDF. Réessayez ultérieurement.
invoice-pdf-error-popup-blocked = Pop-up bloqué par le navigateur — autorisez les pop-ups pour télécharger le PDF.
invoice-pdf-error-missing-contact-address = Adresse du client manquante — renseignez-la dans la fiche contact.
invoice-pdf-error-missing-primary-bank-account = Aucun compte bancaire principal configuré — ajoutez-en un dans les paramètres.

# Libellés bouton frontend (2 clés)
invoices-download-pdf = Télécharger PDF
invoices-download-pdf-aria-label = Télécharger la facture { $number } au format PDF

# Fallbacks AppError
error-invoice-not-validated = La facture doit être validée avant de pouvoir être générée en PDF.
error-invoice-too-many-lines-for-pdf = La facture contient { $count } lignes — le PDF A4 mono-page ne peut pas toutes les afficher avec le récapitulatif TVA. Réduisez le nombre de lignes ou scindez la facture.
error-pdf-generation-failed = Échec de la génération du PDF.
# Story 9-2a + Pass 1 code-review H1 — variant CSV dédié (au lieu de réutiliser
# error-pdf-generation-failed qui aurait affiché « Échec PDF » pour un export CSV).
error-csv-generation-failed = Échec de la génération du CSV.

# Story 5.4 — Échéancier factures
due-dates-title = Échéancier
due-dates-link-aged = Voir la balance âgée
due-dates-link-reminders = Voir les rappels
due-dates-filter-all = Toutes
due-dates-filter-unpaid = Impayées
due-dates-filter-overdue = En retard
due-dates-filter-paid = Payées
due-dates-summary-unpaid = factures impayées
due-dates-summary-overdue = en retard
due-dates-search-label = Recherche
due-dates-contact-label = Contact
due-dates-contact-placeholder = Tous les contacts
due-dates-due-before-label = Échéance avant
due-dates-column-date = Date
due-dates-column-due-date = Échéance
due-dates-column-contact = Client
due-dates-column-total = Total
due-dates-column-amount-due = Reste dû
due-dates-column-payment-status = Statut
due-dates-column-paid-at = Payée le
due-dates-export-button = Exporter CSV
due-dates-no-results = Aucune facture à afficher.
due-dates-result-suffix = résultat(s)

# Statuts paiement
payment-status-paid = Payée
payment-status-unpaid = Impayée
payment-status-overdue = En retard
payment-status-partial = Partiellement payée
invoice-amount-settled = Déjà réglé
invoice-amount-due = Reste dû
invoice-settle-button = Enregistrer un règlement
invoice-settle-button-short = Régler
invoice-settle-dialog-title = Enregistrer un règlement
invoice-settle-dialog-body = Le règlement produit son écriture comptable, quel que soit le mode.
invoice-settle-type-label = Mode de règlement
invoice-settle-type-bank = Virement bancaire
invoice-settle-type-internal = Espèces ou autre compte
invoice-settle-bank-label = Compte bancaire
invoices-settle-no-eligible-bank-account = Aucun compte bancaire utilisable : le seul compte lié est le compte débiteurs de cette facture. Reliez un compte bancaire à son propre compte de banque, ou réglez par un compte interne.
invoice-settle-account-label = Compte
invoice-settle-account-placeholder = — Choisir un compte
invoice-settle-amount-label = Montant
invoice-settle-date-label = Date de règlement
invoice-settle-confirm = Enregistrer le règlement

## Story 25-3-a-1 (#414) — annuler un règlement client
invoices-settlements-title = Règlements
invoices-settlements-col-date = Date
invoices-settlements-col-amount = Montant
invoices-settlements-col-mode = Mode
invoices-settlements-col-entry = Écriture
invoices-settlements-entry-link = Voir l'écriture
invoices-settlements-type-bank = Virement bancaire
invoices-settlements-type-internal = Espèces ou autre compte
invoices-settlement-cancel-button = Annuler le règlement
invoices-settlement-cancel-reconciliation = Annuler le rapprochement
invoices-settlement-cancel-confirm = Annuler ce règlement ? Une écriture inverse datée d'aujourd'hui sera passée au grand livre, et le montant redeviendra dû.
invoices-settlement-cancelled = Règlement annulé : l'écriture inverse a été passée.
invoices-settlement-cancel-blocked-credited = Cette facture a été créditée par un avoir : ce règlement reste ouvert au compte débiteurs, il ne s'annule pas.
invoices-settlement-cancel-blocked-written-off = Le reste de cette facture a été soldé : annulez d'abord le solde.
invoices-amount-written-off = Soldé
invoices-settlements-type-write-off = Solde — { $nature }
invoices-write-off-amount-label = Montant soldé
invoices-write-off-button = Solder le reste
invoices-write-off-cancel-button = Annuler le solde
invoices-write-off-cancel-confirm = Annuler ce solde ? Une écriture inverse datée d'aujourd'hui sera passée au grand livre, TVA comprise, et le montant redeviendra dû.
invoices-write-off-cancelled = Solde annulé : l'écriture inverse a été passée.
invoices-write-off-confirm = Solder le reste
invoices-write-off-date-label = Date du solde
invoices-write-off-dialog-body = Le reste dû est éteint et passé au compte de la nature choisie. Un solde s'annule comme un règlement.
invoices-write-off-dialog-title = Solder le reste
invoices-write-off-error-nature = Choisissez la nature de l'écart.
invoices-write-off-error-unknown-due = Le reste dû de cette facture n'est pas connu : rechargez la fiche.
invoices-write-off-error-date-before-invoice = La date du solde ne peut être antérieure à la date de facture
invoices-write-off-error-date-required = Date du solde obligatoire
invoices-write-off-help-no-vat = Sans correction de TVA.
invoices-write-off-help-vat = La TVA due est corrigée au prorata des taux de la facture.
invoices-write-off-missing-nature-account = Aucun compte n'est désigné pour cette nature : demandez à un administrateur de le choisir dans Administration → Facturation.
invoices-write-off-missing-rounding-account = Ce reste porte une fraction de centime, qui va au compte de différences d'arrondi — aucun n'est désigné : demandez à un administrateur de le choisir dans Administration → Facturation.
invoices-write-off-nature-bad-debt = Perte sur débiteur
invoices-write-off-nature-bank-fees = Frais bancaires
invoices-write-off-nature-discount = Escompte accordé
invoices-write-off-nature-label = Nature de l'écart
invoices-write-off-nature-rounding = Reste d'arrondi
invoices-write-off-stale = La facture a changé entre-temps : vérifiez le reste dû, puis confirmez à nouveau.
invoices-write-off-sub-centime = La fraction de centime est passée au compte de différences d'arrondi.
invoices-write-off-success = Reste soldé — facture payée
settlement-cancel-blocked-fiscal-year-closed = Ce règlement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
settlement-cancel-blocked-lettering-closed = Ce règlement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
settlement-cancel-blocked-bank-match = Ce règlement est rapproché d'une transaction bancaire : annulez d'abord le rapprochement, depuis le détail de l'import bancaire — ou, pour une facture client, depuis sa fiche.
settlement-cancel-blocked-account-archived = Un compte de ce règlement a été archivé : réactivez-le pour pouvoir annuler le règlement.
settlement-cancel-blocked-no-fiscal-year = Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce règlement.
# Story 25-3-b (#418) — annuler un rapprochement : ses motifs et son dialogue.
reconciliation-cancel-blocked-not-reconciled = Cette transaction bancaire n'est pas rapprochée : il n'y a pas de rapprochement à annuler.
reconciliation-cancel-blocked-credited = La facture de ce rapprochement a été créditée par un avoir : son règlement reste ouvert au compte débiteurs, il ne s'annule pas.
reconciliation-cancel-blocked-written-off = Le reste de la facture de ce rapprochement a été soldé : annulez d'abord le solde.
reconciliation-cancel-blocked-fiscal-year-closed = Ce rapprochement appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
reconciliation-cancel-blocked-lettering-closed = Ce rapprochement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
reconciliation-cancel-blocked-bank-match = L'écriture de ce rapprochement est aussi rapprochée d'une autre transaction bancaire : annulez d'abord cet autre rapprochement.
reconciliation-cancel-blocked-account-archived = Un compte de l'écriture de ce rapprochement a été archivé : réactivez-le pour pouvoir annuler le rapprochement.
reconciliation-cancel-blocked-no-fiscal-year = Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler ce rapprochement.
reconciliation-cancel-button = Annuler le rapprochement
reconciliation-cancel-loading = Lecture du rapprochement…
reconciliation-cancel-confirm-entry = Une écriture inverse datée d'aujourd'hui sera passée, et la transaction redeviendra à rapprocher.
reconciliation-cancel-confirm-invoice = Une écriture inverse datée d'aujourd'hui sera passée, le règlement de la facture { $invoice } sera retiré et la facture redeviendra à régler ; la transaction redeviendra à rapprocher.
reconciliation-cancel-confirm-supplier-invoice = Une écriture inverse datée d'aujourd'hui sera passée, le paiement de la facture fournisseur { $invoice } sera retiré et la facture redeviendra ouverte, à payer ; la transaction redeviendra à rapprocher.
reconciliation-cancel-confirm-submit = Confirmer l'annulation
reconciliation-cancel-dismiss = Fermer
reconciliation-cancel-done = Rapprochement annulé.
reconciliation-cancel-entry-link = Voir l'écriture
reconciliation-cancel-column = Rapprochement
# Story 28-5b (#526) — annuler le rejet d'une transaction, depuis le détail de son import.
reconciliation-unreject-button = Annuler le rejet
reconciliation-unreject-rejected-on = Rejetée le { $date }
reconciliation-unreject-success = Rejet annulé : la transaction est de nouveau proposée à la réconciliation, dont la liste montre d'abord les 100 plus récentes en attente.
invoice-settle-success-full = Règlement enregistré — facture soldée
invoice-settle-success-partial = Règlement partiel enregistré
invoice-error-settled-on-required = Date de règlement obligatoire
invoice-error-settled-on-before-invoice-date = La date de règlement ne peut être antérieure à la date de facture
invoice-error-nothing-to-write-off = Il ne reste rien à solder sur cette facture.
invoice-error-write-off-rounding-too-large = Un reste d'arrondi ne dépasse pas 5 centimes : choisissez une autre nature pour solder ce reste.
invoice-error-already-paid = Cette facture est déjà payée : il n'y a rien à solder.
invoice-error-amount-positive = Le montant doit être supérieur à zéro
invoice-error-amount-over-due = Le montant dépasse ce qui reste dû sur cette facture
invoice-error-amount-scale = Le montant ne peut avoir plus de deux décimales
invoice-error-bank-account-required = Choisissez un compte bancaire
invoice-error-account-required = Choisissez un compte

invoice-detail-paid-at-label = Payée le
invoice-detail-subtotal-ht = Sous-total HT
invoice-detail-vat-rate = TVA { $rate }%
invoice-detail-rounding = Arrondi
invoice-detail-rounding-estimated = Arrondi (estimé)
invoice-detail-total-ttc = Total TTC
invoice-detail-total-ttc-estimated = Total TTC (estimé)
invoice-detail-total = Total

# Erreurs validation paidAt
invoice-error-mark-paid-not-validated = Seules les factures validées peuvent être marquées payées
invoice-error-already-unpaid = Cette facture n'est pas marquée payée

# Export CSV — en-têtes (locale = companies.accounting_language)
echeancier-csv-header-number = Numéro
echeancier-csv-header-date = Date
echeancier-csv-header-due-date = Date d'échéance
echeancier-csv-header-contact = Client
echeancier-csv-header-total = Total
echeancier-csv-header-amount-due = Reste dû
echeancier-csv-header-payment-status = Statut paiement
echeancier-csv-header-paid-at = Date paiement
echeancier-export-error-too-large = Trop de résultats (> { $limit }). Veuillez affiner vos filtres (par ex. plage de dates ou statut de paiement) avant de relancer l'export.
# Story 31-5 (#504) — préfixe de la communication de la QR-facture (« <préfixe> <numéro> »),
# dans la langue du document. Sans variable : aucune marque d'isolation Fluent n'entre dans la QR.
invoice-qr-message-prefix = Facture
invoice-pdf-error-contact-missing = Le contact lié à la facture est introuvable.
invoice-pdf-error-no-primary-bank = Aucun compte bancaire principal n'est configuré pour cette entreprise.
invoice-pdf-error-company-address-empty = L'adresse de l'entreprise est vide — renseignez-la avant de générer un PDF.
invoice-pdf-error-client-address-required = L'adresse du client est obligatoire pour la génération du PDF.
invoice-pdf-error-client-address-empty = L'adresse du client est vide — renseignez-la avant de générer un PDF.

# Commons
common-loading = Chargement…
common-previous = Précédent
common-next = Suivant
common-cancel = Annuler
common-back = Retour
common-error = Erreur inattendue

invoice-pdf-error-not-found = Facture introuvable.
invoice-pdf-error-generic = Erreur lors du téléchargement du PDF.
invoice-pdf-error-empty = Le PDF reçu est vide.

# Story 2.6 — Onboarding: Invoice Settings Pre-fill
config-incomplete-title = Configuration incomplète
config-incomplete-link = Configurez les comptes de facturation
invoice-settings-required = Configurez d'abord les comptes de facturation dans Administration → Facturation

# === Story 3.7 — Gestion des exercices comptables (FR-CH) ===

fiscal-year-title = Exercices comptables
fiscal-year-out-of-order-warning = L’exercice « { $open } » est ouvert alors qu’un exercice postérieur, « { $closed } », est clôturé : rien ne peut y être enregistré tant que l’ordre n’est pas rétabli. Clôturez « { $open } » si ses comptes sont arrêtés, puis les exercices ouverts suivants, du plus ancien au plus récent. Sinon, un administrateur rouvre les exercices postérieurs clôturés, en commençant par le plus récent, « { $latest } » : Kesh ne rouvre un exercice que si aucun exercice plus récent n’est clôturé.
fiscal-year-list-empty = Aucun exercice comptable.
fiscal-year-create-button = Nouvel exercice
fiscal-year-name-label = Nom
fiscal-year-start-date-label = Date de début
fiscal-year-end-date-label = Date de fin
fiscal-year-status-label = Statut
fiscal-year-status-open = Ouvert
fiscal-year-status-closed = Clôturé
fiscal-year-rename-button = Renommer
fiscal-year-close-button = Clôturer
fiscal-year-close-confirmation-title = Clôturer cet exercice ?
fiscal-year-close-confirmation-body = Vous êtes sur le point de clôturer l’exercice « { $name } ». Aucune écriture, facture ou paiement ne pourra plus y être enregistré tant qu’il reste clôturé ; seul un administrateur peut le rouvrir (avec un motif tracé). Confirmer ?
fiscal-year-close-confirmation-action = Clôturer
fiscal-year-created = Exercice créé avec succès.
fiscal-year-renamed = Exercice renommé.
fiscal-year-closed = Exercice clôturé.
# Story 14-2 — réouverture d'un exercice clôturé (Admin, motif, audit, garde LIFO)
fiscal-year-reopen-button = Réouvrir
fiscal-year-reopen-confirmation-title = Rouvrir cet exercice ?
fiscal-year-reopen-confirmation-body = Vous êtes sur le point de rouvrir l’exercice « { $name } ». Il redeviendra modifiable (saisie d’écritures) jusqu’à une nouvelle clôture. Un motif est obligatoire et sera conservé dans le journal d’audit.
fiscal-year-reopen-motif-label = Motif de la réouverture
fiscal-year-reopen-confirmation-action = Rouvrir l’exercice
fiscal-year-reopened = Exercice rouvert.
fiscal-year-reopen-blocked-later-closed = Rouvrez d’abord l’exercice « { $name } », plus récent et encore clôturé.
fiscal-year-close-blocked-earlier-open = Clôturez d’abord l’exercice « { $name } », plus ancien et encore ouvert.
# Story 29-1a (#232) — Préparer la clôture : contrôles préalables, lecture seule.
fiscal-years-checks-button = Préparer la clôture
fiscal-years-checks-title = Préparer la clôture — { $name }
fiscal-years-checks-close = Fermer
fiscal-years-checks-intro = Ces contrôles ne bloquent rien : ils signalent ce qui reste à faire ou paraît anormal dans l’exercice. Ils lisent les comptes à cet instant.
fiscal-years-checks-loading = Lecture des contrôles…
fiscal-years-checks-error = Contrôles indisponibles pour le moment : fermez ce panneau, puis ouvrez-le de nouveau pour réessayer.
fiscal-years-checks-ok = Rien à signaler
fiscal-years-checks-severity-blocking = Bloquant
fiscal-years-checks-severity-warning = À vérifier
fiscal-years-checks-severity-info = Pour information
fiscal-years-checks-severity-anomaly = Anomalie
fiscal-years-checks-k1-title = Exercices antérieurs
fiscal-years-checks-k1-help = Un exercice ne se clôture qu’après tous ceux qui le précèdent.
fiscal-years-checks-k1-attention = L’exercice « { $name } », plus ancien, est encore ouvert : clôturez-le d’abord.
fiscal-years-checks-k2-title = Transactions bancaires à rapprocher
fiscal-years-checks-k2-help = Les transactions importées de l’exercice qui n’ont pas encore d’écriture.
fiscal-years-checks-k2-attention = { $count } transaction(s) bancaire(s) de l’exercice en attente de rapprochement.
fiscal-years-checks-k2-account = { $name } : { $count }
fiscal-years-checks-k2-account-rejected = { $name } : { $count }, dont { $rejected } écartée(s) des propositions
fiscal-years-checks-k2-link = Ouvrir le rapprochement
fiscal-years-checks-k3-title = Relevés bancaires de fin d’exercice
fiscal-years-checks-k3-help = Pour chaque compte bancaire actif dans l’exercice, le relevé arrêté au dernier jour de l’exercice, comparé aux livres.
fiscal-years-checks-k3-missing = { $name } : aucun relevé arrêté dans l’exercice.
fiscal-years-checks-k3-ends-before = { $name } : le dernier relevé de l’exercice s’arrête le { $date }, avant sa fin.
fiscal-years-checks-k3-gap = { $name } : écart de { $amount } entre les livres et le relevé du { $date }.
fiscal-years-checks-k3-not-comparable = { $name } : non vérifiable — sans compte de grand livre lié, ou compte lié partagé avec un autre compte bancaire.
fiscal-years-checks-k3-no-balance-statement = { $name } : non vérifiable — aucun relevé importé ne porte de solde (import CSV seulement).
fiscal-years-checks-k3-link = Ouvrir les comptes bancaires
fiscal-years-checks-k4-title = Factures en brouillon
fiscal-years-checks-k4-help = Les factures datées dans l’exercice et jamais validées : elles ne sont pas en comptabilité.
fiscal-years-checks-k4-attention = { $count } facture(s) en brouillon datée(s) dans l’exercice.
fiscal-years-checks-k4-link = Voir les brouillons de l’exercice
fiscal-years-checks-k5-title = Documents fournisseur à traiter
fiscal-years-checks-k5-help = Les documents fournisseur qui attendent d’être traités, toutes dates confondues.
fiscal-years-checks-k5-attention = { $count } document(s) à traiter, toutes dates confondues.
fiscal-years-checks-k5-link = Ouvrir les documents à traiter
fiscal-years-checks-k6-title = Écritures incohérentes
fiscal-years-checks-k6-help = Des écritures héritées ou restaurées que Kesh ne produit pas lui-même : déséquilibrées, incomplètes ou mal datées.
fiscal-years-checks-k6-attention = { $count } écriture(s) incohérente(s).
fiscal-years-checks-k6-entry = N° { $number } ({ $year }, { $date })
fiscal-years-checks-k6-unbalanced = déséquilibrée
fiscal-years-checks-k6-too-few-lines = moins de deux lignes
fiscal-years-checks-k6-date-outside = datée hors de son exercice
fiscal-years-checks-k6-other-year-inside = d’un autre exercice, datée dans celui-ci
fiscal-years-checks-k7-title = Exercice suivant
fiscal-years-checks-k7-help = L’exercice qui suit, où se passent les écritures de l’année suivante.
fiscal-years-checks-k7-ok = « { $name } » suit cet exercice.
fiscal-years-checks-k7-attention = Aucun exercice ne suit celui-ci : créez l’exercice suivant pour y passer les écritures de l’année qui vient.
fiscal-years-checks-k7-link = Créer l’exercice suivant
fiscal-years-checks-reminder-none = Aucun point à vérifier.
fiscal-years-checks-reminder-count = { $count } point(s) à vérifier :
fiscal-years-checks-reminder-error = Les contrôles préalables n’ont pas pu être lus. Vous pouvez clôturer quand même, ou les consulter plus tard depuis « Préparer la clôture ».
fiscal-years-checks-reminder-link = Voir le détail dans « Préparer la clôture »
error-fiscal-year-reopen-motif-empty = Le motif de réouverture est obligatoire.
error-fiscal-year-reopen-motif-too-long = Le motif de réouverture est trop long (500 caractères maximum).
error-fiscal-year-already-open = Cet exercice est déjà ouvert.
error-fiscal-year-reopen-blocked = Réouverture impossible : un exercice postérieur est clôturé ; rouvrez d’abord les exercices postérieurs clôturés, en commençant par le plus récent.
error-fiscal-year-close-earlier-open = Clôturez d’abord l’exercice « { $name } », plus ancien et encore ouvert : le bilan est cumulatif, et un exercice ne se clôt qu’après tous ceux qui le précèdent.
# Story 29-1b (#232) — contre-passation au début de l'exercice suivant.
error-next-fiscal-year-missing = Aucun exercice ne suit l’exercice « { $name } » : créez l’exercice suivant, puis recommencez.
error-next-fiscal-year-closed = L’exercice suivant, « { $name } », est clôturé : la contre-passation ne peut pas y être datée.
error-fiscal-year-create-later-closed = L’exercice « { $name } », postérieur, est clôturé, et son bilan reprend tout ce qui le précède : aucun exercice ne peut être créé avant sa date de début tant qu’il l’est. Pour créer celui-ci, un administrateur rouvre d’abord les exercices postérieurs clôturés, en commençant par le plus récent.
error-fiscal-year-overlap = Cet exercice chevauche un exercice existant.
error-fiscal-year-name-duplicate = Un exercice avec ce nom existe déjà.
error-fiscal-year-name-empty = Le nom de l’exercice est obligatoire.
error-fiscal-year-name-too-long = Le nom de l’exercice est trop long (50 caractères maximum).
error-fiscal-year-dates-invalid = Dates invalides — la date de fin doit être strictement postérieure à la date de début.
error-fiscal-year-already-closed = Cet exercice est déjà clôturé.
error-fiscal-year-conflict = Conflit d’exercice (nom ou date de début déjà utilisé).
error-fiscal-year-missing = Créez d’abord un exercice comptable dans Paramètres → Exercices comptables.
error-fiscal-year-closed-for-date = L’exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices.
go-to-settings = Ouvrir Paramètres
settings-fiscal-years-link = Créez, renommez ou clôturez les exercices comptables de votre entreprise.

# Story 14-4 — bilan d'ouverture (soldes de départ, reprise de comptabilité)
opening-balances-title = Soldes de départ
opening-balances-intro = Saisissez les soldes de vos comptes de bilan repris de votre ancienne comptabilité. Une écriture d’ouverture équilibrée sera générée au { $date } (premier jour de l’exercice « { $name } »). Posez votre report à-nouveau accumulé sur votre compte de report pour équilibrer l’écriture.
opening-balances-account = Compte
opening-balances-debit = Débit
opening-balances-credit = Crédit
opening-balances-credit-for = Crédit du compte { $number } « { $name } »
opening-balances-debit-for = Débit du compte { $number } « { $name } »
opening-balances-total-debit = Total débits
opening-balances-total-credit = Total crédits
opening-balances-diff = Différence
opening-balances-generate = Générer l’écriture d’ouverture
opening-balances-generating = Génération…
opening-balances-success = Écriture d’ouverture générée.
opening-balances-entry-description = Bilan d’ouverture — soldes de départ
opening-balances-locked-no-fiscal-year = Aucun exercice comptable : créez d’abord un exercice (Paramètres → Exercices comptables) pour saisir vos soldes de départ.
opening-balances-locked-first-year-closed = Le premier exercice « { $name } » est clôturé : avant la saisie des soldes de départ, un administrateur doit rouvrir les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.
opening-balances-locked-already-has-entries = La société contient déjà des écritures : le bilan d’ouverture a été généré et ne se régénère plus. Un compte de bilan oublié se complète ci-dessous ; un montant faux sur un compte déjà saisi se corrige dans le journal, en modifiant l’écriture d’ouverture tant que l’exercice est ouvert (après l’avoir délettrée si l’une de ses lignes est lettrée), ou par une contre-passation ou une écriture de correction.
opening-balances-goto-journal = Ouvrir le journal
opening-balances-goto-balance-sheet = Voir le bilan
opening-balances-status-error = Impossible de charger l’état des soldes de départ.
opening-balances-retry = Réessayer
opening-balances-empty-grid = Aucun compte de bilan actif et postable dans votre plan comptable — créez ou réactivez vos comptes d’actifs et de passifs (Plan comptable) avant de saisir les soldes de départ.
error-opening-balances-no-fiscal-year = Aucun exercice comptable : créez d’abord un exercice avant de saisir les soldes de départ.
error-opening-balances-first-year-closed = Le premier exercice est clôturé : avant de saisir les soldes de départ, rouvrez les exercices clôturés jusqu’à celui-ci, en commençant par le plus récent.
error-opening-balances-already-has-entries = La société contient déjà des écritures : le bilan d’ouverture ne peut plus être généré. Un compte oublié se complète depuis l’écran Soldes de départ.
error-opening-balances-non-balance-account = Le bilan d’ouverture ne peut toucher que des comptes de bilan (actifs et passifs) — retirez les comptes de produits et de charges.
# Story 25-7 (#445) — avertissement, totaux et complément des soldes de départ
opening-balances-complement-description = Complément des soldes de départ
opening-balances-total-assets = Actifs
opening-balances-total-liabilities = Passifs et capitaux (hors report)
opening-balances-amount-to-carry = Montant à porter au compte de report
opening-balances-retained-entered = Report saisi
opening-balances-remaining-gap = Écart restant
opening-balances-side-credit = au crédit
opening-balances-side-debit = au débit
opening-balances-compare-hint = Comparez les totaux Actifs et Passifs à ceux du bilan de votre ancienne comptabilité : une saisie équilibrée peut encore être fausse si un montant a été porté sur le mauvais compte.
opening-balances-warning-no-retained-earnings = La saisie est équilibrée sans rien sur le compte de report { $number } « { $name } ». Le report à-nouveau — la différence entre les actifs et les passifs de votre ancien bilan — doit y être porté : soit l’écart a été mis sur un autre compte, soit le report vaut réellement zéro. Comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.
opening-balances-warning-no-retained-role = Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : Kesh ne peut pas vérifier où le report à-nouveau est porté. Attribuez ce rôle dans le plan comptable, et comparez les totaux Actifs et Passifs à ceux de l’ancien bilan.
opening-balances-warning-retained-not-postable = Le compte de report { $number } « { $name } » n’est pas imputable : il n’apparaît pas dans la grille, et le report à-nouveau ne peut pas y être porté. Rendez-le imputable dans le plan comptable.
opening-balances-complete-title = Compléter un compte oublié
opening-balances-complete-intro = Saisissez le solde des seuls comptes de bilan oubliés à l’ouverture. Kesh porte la contrepartie sur le compte de report { $number } « { $name } ».
opening-balances-complete-date-opening = L’écriture sera datée du { $date }, premier jour de l’exercice « { $name } » : le compte oublié faisait partie de l’ouverture.
opening-balances-complete-date-today = L’écriture sera datée du { $date }, dans l’exercice « { $name } » : une régularisation, le premier jour de l’ouverture n’acceptant plus d’écriture (exercice clôturé ou période verrouillée).
opening-balances-complete-date-note = La date est confirmée à l’enregistrement.
opening-balances-complete-counterpart-credit = Contrepartie : { $amount } au crédit du compte { $number }.
opening-balances-complete-counterpart-debit = Contrepartie : { $amount } au débit du compte { $number }.
opening-balances-complete-counterpart-none = Les lignes s’équilibrent entre elles : aucune contrepartie.
opening-balances-complete-submit = Compléter
opening-balances-complete-submitting = Enregistrement…
opening-balances-complete-confirm = Enregistrer cette écriture de complément, datée du { $date } ? Elle reste modifiable depuis sa fiche tant que l’exercice est ouvert — sauf si l’une de ses lignes est lettrée : délettrez-la d’abord.
opening-balances-complete-success = Complément enregistré.
opening-balances-complete-error = Le complément n’a pas pu être enregistré. Réessayez.
opening-balances-complete-unavailable-no-entries = La société n’a encore aucune écriture : utilisez la génération du bilan d’ouverture.
opening-balances-complete-unavailable-no-open-fiscal-year = Aucun exercice ouvert ne couvre la date du jour : le complément ne peut pas être daté.
opening-balances-complete-unavailable-date-locked = La date du jour tombe dans la période verrouillée : le complément ne peut pas être daté.
opening-balances-complete-unavailable-no-retained-earnings = Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : attribuez-le dans le plan comptable pour compléter un compte oublié.
opening-balances-complete-unavailable-retained-earnings-not-postable = Le compte de bénéfice reporté n’est pas imputable : rendez-le imputable pour compléter un compte oublié.
opening-balances-complete-unavailable-no-completable-account = Tous les comptes de bilan proposables — en service, imputables, hors compte de report — ont déjà des mouvements : il ne reste aucun compte à compléter. Un montant faux se corrige dans le journal.
error-opening-complement-no-entries = La société n’a encore aucune écriture : saisissez les soldes de départ avec la génération du bilan d’ouverture.
error-opening-complement-no-open-fiscal-year = Aucun exercice ouvert ne couvre la date du jour : le complément ne peut pas être daté.
error-opening-complement-date-locked = La date du jour tombe dans la période verrouillée : le complément ne peut pas être daté.
error-opening-complement-no-retained-earnings = Aucun compte en service ne porte le rôle « Bénéfice/perte reporté » : attribuez-le dans le plan comptable, la contrepartie du complément y est portée.
error-opening-complement-retained-earnings-not-postable = Le compte de bénéfice reporté n’est pas imputable : rendez-le imputable dans le plan comptable.
error-opening-complement-account-invalid = Le compte { $account } est inconnu, archivé ou non imputable.
error-opening-complement-retained-earnings-line = Le compte { $account } est le compte de bénéfice reporté : sa contrepartie est calculée par Kesh, ne le saisissez pas.
error-opening-complement-not-balance-account = Le compte { $account } est un compte de résultat : seuls les comptes de bilan se complètent.
error-opening-complement-account-moved = Le compte { $account } a déjà des mouvements : corrigez-le dans le journal, en modifiant l’écriture, par une contre-passation ou une écriture de correction.
error-opening-complement-no-lines = Saisissez au moins un compte à compléter.
error-opening-complement-too-many-lines = Trop de lignes pour un seul complément.
error-opening-complement-invalid-amount = Chaque ligne porte un montant strictement positif, au plus quatre décimales, au débit OU au crédit ; la contrepartie qui en résulte (débits moins crédits) ne peut dépasser 999’999’999’999’999.9999.
error-opening-complement-duplicate-account = Un même compte figure sur deux lignes : regroupez-les.


# --- Story 8-1b — Import bancaire CAMT.053 ---
# Convention de nommage (validate Pass 3 O1+O2) :
#   bank-import-errors-{slug}   — erreurs (codes HTTP 4xx/5xx)
#   bank-import-warnings-{slug} — warnings (preview 200 OK)
#   bank-import-labels-{slug}   — labels statiques UI

# Errors
bank-import-errors-too-large = Fichier trop volumineux. La taille maximale autorisée est de 10 Mio.
bank-import-errors-malformed-xml = Fichier XML mal formé ou tronqué. Vérifiez l'export bancaire.
bank-import-errors-unsupported-version = Version CAMT.053 non supportée. Versions acceptées : 001.04 et 001.08.
bank-import-errors-missing-field = Un champ requis est absent du fichier CAMT.053.
bank-import-errors-invalid-amount = Un montant est invalide dans le fichier.
bank-import-errors-invalid-date = Une date est invalide dans le fichier.
bank-import-errors-balance-mismatch = Le solde de clôture ne correspond pas à la somme des transactions. Cochez « Confirmer malgré l'écart » pour importer quand même.
bank-import-errors-unsupported-currency = Devise non supportée. Seul le franc suisse (CHF) est pris en charge dans cette version.
bank-import-errors-no-matching-statement = Aucun statement du fichier ne correspond au compte bancaire sélectionné.
bank-import-errors-duplicate-file = Ce fichier a déjà été importé pour cette entreprise.
bank-import-errors-has-reconciled-transactions = { $count } transaction(s) de cet import sont rapprochées : annulez d'abord leur rapprochement pour pouvoir le supprimer.
bank-import-errors-bank-account-not-found = Compte bancaire introuvable.
bank-import-errors-parse-failed = Le fichier CAMT.053 n'a pas pu être analysé.

# Warnings (preview 200 OK)
bank-import-warnings-balance-mismatch = Solde de clôture incohérent.
bank-import-warnings-unsupported-currency = Devise non supportée : seul le franc suisse (CHF) est pris en charge. Devise du relevé :
bank-import-warnings-ignored-statements = Certains statements du fichier ne correspondent pas au compte sélectionné et seront ignorés.
# Story 8-3 — détection doublons + rejet partiel
bank-import-warnings-duplicate-file = Ce fichier a déjà été importé.
bank-import-warnings-duplicate-lines-summary = transactions chevauchent un import précédent.
bank-import-warnings-invalid-lines-summary = lignes invalides détectées dans le CSV.
bank-import-warnings-invalid-lines-truncated = Premières 100 erreurs affichées (cap atteint).
bank-import-warnings-encoding-mismatch = L'encodage détecté diffère du profil.

# Labels UI
bank-import-labels-page-title = Import de relevés bancaires
bank-import-labels-bank-account-selector = Compte bancaire cible
bank-import-labels-drop-zone = Glissez votre fichier CAMT.053 ou CSV ici ou cliquez pour parcourir
bank-import-labels-preview-title = Prévisualisation
bank-import-labels-confirm-import = Confirmer l'import
bank-import-labels-cancel = Annuler
# Story 27-1 (#632) — supprimer un import dont aucune transaction n'est rapprochée
bank-import-delete-button = Supprimer l'import
bank-import-delete-blocked = { $count } transaction(s) de cet import sont rapprochées. Pour supprimer l'import, annulez d'abord leur rapprochement.
bank-import-delete-dialog-title = Supprimer cet import ?
bank-import-delete-dialog-body = Le fichier { $filename } (période du { $from } au { $to }) et ses { $count } transaction(s) seront supprimés. Vous pourrez ensuite réimporter le même fichier. Cette suppression ne peut pas être annulée.
bank-import-delete-dialog-confirm = Supprimer définitivement
bank-import-delete-error-reconciled = { $count } transaction(s) de cet import ont été rapprochées entre-temps : rechargez la page.
bank-import-delete-error-not-found = Cet import n'existe plus : il a peut-être déjà été supprimé.
bank-import-delete-success = Import { $filename } supprimé.
bank-import-labels-confirm-balance-mismatch = Importer malgré l'écart de solde
# Story 8-3 — labels confirm flags + KF #70
bank-import-labels-confirm-duplicate-file = Importer malgré le fichier déjà importé
bank-import-labels-confirm-duplicate-lines = Comportement face aux doublons
bank-import-labels-confirm-duplicate-lines-skip = Ignorer les doublons (par défaut)
bank-import-labels-confirm-duplicate-lines-import = Importer quand même
bank-import-labels-confirm-partial-import = Importer les lignes valides quand même
bank-import-labels-confirm-encoding-mismatch = Importer avec l'encodage détecté
bank-import-labels-bank-profile-selector = Profil bancaire CSV
bank-import-labels-bank-profile-auto-matched = auto-détecté
# L6 (Pass 1 review) — placeholder du <select> (option vide), distinct de
# l'annotation `auto-matched` ci-dessus pour permettre des traductions
# divergentes (ex. EN: "Auto-detect" placeholder vs "(auto-matched)" annotation).
bank-import-labels-bank-profile-auto-detect-placeholder = Auto-détection
# Story 27-3 (#626) — aperçu recalculé au changement de profil, empreinte périmée.
bank-import-errors-preview-stale = Le profil, le fichier ou le compte a changé depuis la prévisualisation : recalculez-la avant d'importer.
bank-import-labels-preview-recomputing = Recalcul de la prévisualisation…
bank-import-labels-preview-recomputed = Le profil, le fichier ou le compte a changé depuis la prévisualisation : Kesh la recalcule. Vérifiez le résultat avant de confirmer.
# Story 27-3 (C-27-3-25) — libellés de l'écran d'import, jusque-là en dur.
bank-import-labels-bank-account-placeholder = Sélectionner un compte
bank-import-errors-bank-account-required = Sélectionnez d'abord un compte bancaire.
bank-import-labels-col-line = Ligne
bank-import-labels-col-code = Code
bank-import-labels-col-value = Valeur
bank-import-labels-iban = IBAN
bank-import-labels-period = Période
bank-import-labels-currency = Devise
bank-import-labels-opening-balance = Solde ouverture
bank-import-labels-closing-balance = Solde clôture
bank-import-labels-transaction-count = Nombre de transactions
bank-import-labels-duplicate-col-index = Index
bank-import-labels-duplicate-col-existing-id = ID existant
bank-import-labels-duplicate-col-key = Clé
bank-import-labels-ignored-statements-summary = statement(s) ignoré(s) (autres IBAN dans le fichier).
bank-import-labels-tx-col-date = Date
bank-import-labels-tx-col-amount = Montant
bank-import-labels-tx-col-details = Détails
bank-import-labels-list-col-imported-at = Date import
bank-import-labels-list-col-filename = Fichier
bank-import-labels-list-col-transactions = Transactions
# M8 (Pass 1 review) — clés i18n pour les codes informationnels remontés
# via `warnings.informational[]` (snake_case → kebab-case côté frontend).
bank-import-info-bank-csv-profile-auto-matched = Profil bancaire détecté automatiquement par le nom du fichier.
bank-import-info-bank-csv-multiple-profile-matches = Plusieurs profils correspondent au nom du fichier ; le premier a été retenu.
bank-import-info-bank-csv-currency-assumed = Le profil ne lit aucune colonne Devise : le relevé est supposé en francs suisses (CHF).
bank-import-errors-no-valid-lines-to-commit = Aucune ligne valide à importer dans le CSV.
bank-import-labels-list-title = Imports précédents
bank-import-labels-import-success = Import réussi.
bank-import-labels-empty = Aucun import bancaire.

# Story 8-2 — bank-csv + bank-profile keys
bank-import-csv-errors-no-profile-match = Aucun profil bancaire ne matche ce fichier.
bank-import-csv-errors-unsupported-encoding = Encoding du fichier non supporté (UTF-8 ou ISO-8859-1 attendu).
bank-import-csv-errors-encoding-mismatch = L'encoding détecté diffère du profil. Confirmez via confirmEncodingMismatch=true.
bank-import-csv-errors-partial-failure = Certaines lignes du CSV n'ont pas pu être parsées.
bank-import-csv-errors-profile-invalid = Profil bancaire invalide.
bank-import-csv-errors-profile-duplicate = Un profil avec ce nom de banque existe déjà.
bank-import-csv-errors-profile-misconfigured = Profil bancaire mal configuré.
bank-import-csv-errors-empty-file = Fichier CSV vide ou aucune ligne de données.
bank-import-csv-errors-invalid-date = Date invalide dans une ligne CSV.
bank-import-csv-errors-invalid-amount = Montant invalide dans une ligne CSV.
bank-import-csv-errors-ambiguous-debit-credit = Débit et crédit tous deux non-vides sur la même ligne.
bank-import-csv-errors-empty-mandatory-field = Champ obligatoire vide.
bank-import-csv-errors-row-too-short = Ligne trop courte (colonnes manquantes).
# Story 26-1 (#620, #621) — sens des débits, lignes à ignorer
bank-import-csv-errors-amount-sign-mismatch = Montant de signe contraire à sa colonne — vérifier le réglage du sens des débits du profil.
bank-import-csv-errors-invalid-currency = Devise illisible dans une ligne CSV (trois lettres attendues, par exemple CHF).
bank-import-csv-errors-amount-sign-refused-profile = Tous les débits lus dans le fichier ont un signe contraire au réglage du profil. Réglez le sens des débits du profil (Administration → Profils bancaires), puis redéposez le fichier.
bank-import-csv-errors-amount-sign-refused-lines = { $count } ligne(s) portent un montant de signe contraire à leur colonne (une extourne, par exemple) : ce relevé ne peut pas être importé tel quel dans cette version. Les lignes en cause sont listées ci-dessous.
bank-import-csv-errors-header-skips-data = La ligne { $line }, qui devrait être ignorée, porte déjà une date : le nombre de lignes à ignorer du profil est trop grand, une transaction aurait disparu.
bank-import-csv-errors-mixed-currencies = Le relevé porte plusieurs devises ({ $currencies }) : un relevé CSV doit être dans une seule devise. Rien n'a été importé.
bank-import-csv-warnings-profile-auto-matched = Profil appliqué automatiquement par auto-match.
bank-import-csv-warnings-multiple-profile-matches = Plusieurs profils matchent ce filename, le plus récent a été utilisé.
bank-import-csv-warnings-encoding-mismatch = L'encoding détecté diffère du profil.
bank-import-errors-unsupported-format = Format de fichier non supporté (CAMT.053 XML ou CSV attendus).
bank-import-profile-labels-page-title = Profils bancaires CSV
bank-import-profile-labels-page-title-new = Nouveau profil bancaire
bank-import-profile-labels-page-title-edit = Éditer le profil bancaire
bank-import-profile-labels-bank-name = Nom de la banque
bank-import-profile-labels-filename-pattern = Pattern filename (regex)
bank-import-profile-labels-filename-pattern-help = Regex case-sensitive (utilisez `(?i)` pour case-insensitive)
bank-import-profile-labels-date-format = Format date (chrono)
bank-import-profile-labels-decimal-separator = Séparateur décimal
bank-import-profile-labels-field-separator = Séparateur champs
bank-import-profile-labels-encoding = Encodage (optionnel)
bank-import-profile-labels-actions = Actions
bank-import-profile-labels-header-row-count = Lignes à ignorer avant les données (0-20)
bank-import-profile-labels-header-row-count-help = Ligne de titres comprise ; les lignes vides ne comptent pas. Relevé CSV d’UBS : 9.
bank-import-profile-labels-debit-sign = Sens des montants de la colonne Débit
bank-import-profile-labels-debit-sign-positive = Débits écrits sans signe (500.00)
bank-import-profile-labels-debit-sign-negative = Débits écrits avec un signe moins (−500.00), comme UBS
bank-import-profile-labels-column-mapping = Mapping colonnes (0-indexed)
bank-import-profile-labels-use-debit-credit-split = Colonnes débit/crédit séparées
bank-import-profile-labels-col-currency = devise (facultative)
bank-import-profile-labels-col-currency-help = Colonne qui porte la devise de chaque ligne. Sans elle, le relevé est supposé en francs suisses (CHF). Relevé CSV d'UBS : 4.
bank-import-profile-labels-create = Créer
bank-import-profile-labels-update = Mettre à jour
bank-import-profile-labels-edit = Éditer
bank-import-profile-labels-delete = Supprimer
bank-import-profile-labels-confirm-delete = Supprimer ce profil ?
bank-import-profile-labels-new-profile = Nouveau profil
bank-import-profile-labels-no-profiles = Aucun profil bancaire configuré.
bank-import-profile-errors-bank-name-required = Le nom de la banque est requis.
bank-import-profile-errors-bank-name-duplicate = Un profil avec ce nom existe déjà.
bank-import-profile-errors-column-mapping-xor-violation = Choisir `amount` OU `debit_credit_split`, pas les deux.
bank-import-profile-errors-date-format-invalid = Format de date chrono invalide.
bank-import-profile-errors-regex-invalid = Regex filename_pattern invalide.
bank-import-profile-errors-separators-equal = Les séparateurs champs et décimal doivent être différents.

# Story 8-4 (FR44) — Réconciliation bancaire automatique.
reconciliation-page-title = Réconciliation
reconciliation-page-subtitle = Propositions automatiques de matching transaction ↔ facture.
reconciliation-labels-loading = Chargement des propositions…
reconciliation-labels-empty = Aucune transaction en attente de réconciliation.
reconciliation-labels-no-account = Aucun compte bancaire configuré.
reconciliation-labels-account-select = Compte bancaire
reconciliation-labels-no-candidate = Aucune correspondance
reconciliation-labels-amount-due-of = reste dû sur { $total }
reconciliation-labels-success-suffix = opération(s) réussie(s).
reconciliation-labels-failed = Échecs partiels
reconciliation-labels-rejected-hint = Une transaction rejetée quitte cette liste. Elle reste dans le détail de son import (Importer des relevés), où son rejet peut être annulé si votre rôle le permet.
# Story 29-3 (#527) — la priorité des candidates, dite au-dessus de la liste.
reconciliation-labels-priority-hint = Une seule proposition est affichée par transaction : pour un crédit, la facture client la mieux notée, pour un débit, la facture fournisseur la mieux notée, s’il y en a une, sinon la règle d’affectation qui correspond. Une facture proposée passe toujours devant une règle.
reconciliation-labels-supplier-invoice-badge = Facture fournisseur
reconciliation-labels-notice-supplier-invoice-in-batch = La facture fournisseur { $invoice } de ce montant figure dans le lot de paiement n° { $batch } en cours sur ce compte. Si cette transaction est le paiement de ce lot, annulez le lot : la facture vous sera alors proposée ici. Sinon, affectez la transaction à la main.
# Story 29-6 (#665, AC10) — rendue dans la rangée des boutons quand une ligne sans proposition est cochée.
reconciliation-labels-unmatched-not-acceptable-hint = Une transaction sans correspondance ne peut pas être acceptée : décochez-la pour accepter les autres.
# Story 29-9 (#678, AC11) — nom accessible de la case de chaque ligne : ce que la ligne affiche.
reconciliation-labels-select-transaction = Sélectionner la transaction du { $date }, { $amount } { $currency }, { $counterparty }
reconciliation-labels-select-transaction-no-counterparty = Sélectionner la transaction du { $date }, { $amount } { $currency }
# Story 29-9 (#649, AC1, AC3) — sous la liste, s'il reste des transactions plus anciennes.
reconciliation-labels-more-hidden = Seules les { $count } transactions en attente les plus récentes sont affichées : il en reste de plus anciennes.
reconciliation-labels-more-capped = Les { $max } transactions en attente les plus récentes sont affichées, le maximum que cette page montre à la fois. Il en reste de plus anciennes : elles apparaîtront à mesure que celles-ci seront acceptées, affectées manuellement, éclatées ou rattachées à un règlement.
# Story 15-5c (#492) — libellés des refus d'un lot de rapprochement (`failed[]`).
reconciliation-failed-account-not-found = Un compte de contrepartie est introuvable ou archivé.
reconciliation-failed-account-not-postable = Compte non imputable : { $numbers }. Un compte de regroupement, de résultat ou de clôture ne reçoit pas d’écriture : choisissez un compte imputable.
reconciliation-failed-account-not-postable-generic = Un compte de contrepartie n’est pas imputable : un compte de regroupement, de résultat ou de clôture ne reçoit pas d’écriture. Choisissez un compte imputable.
reconciliation-failed-bank-account-not-configured = Le compte bancaire n’est lié à aucun compte comptable actif : liez-en un dans Administration → Comptes bancaires.
reconciliation-failed-bank-account-not-found = Compte bancaire introuvable.
reconciliation-failed-bank-account-closed = Le compte bancaire est clos : il est en lecture seule. Pour y enregistrer une opération, rouvrez d’abord le compte (Administration → Comptes bancaires).
reconciliation-failed-bank-transaction-not-found = Transaction introuvable : elle a été supprimée avec son import, ou n’appartient pas à ce compte bancaire.
reconciliation-failed-already-rejected = Cette transaction a déjà été rejetée : elle n’est plus proposée. Son rejet s’annule depuis le détail de son import (Importer des relevés).
reconciliation-failed-changed-during-batch = La transaction a été modifiée pendant le traitement — rapprochée, rejetée ou supprimée par une autre opération.
reconciliation-failed-database-error = Erreur de la base de données : réessayez ; si le problème persiste, contactez le support.
reconciliation-failed-invoice-not-found = Facture introuvable.
reconciliation-failed-invoice-sale-entry-malformed = L’écriture de vente de cette facture n’a pas de ligne au compte débiteurs : le règlement ne peut pas être passé.
reconciliation-failed-later-fiscal-year-closed = Exercice postérieur « { $name } » clôturé : aucune écriture ne peut être datée avant lui.
reconciliation-failed-later-fiscal-year-closed-generic = Un exercice postérieur est clôturé : aucune écriture ne peut être datée avant lui.
reconciliation-failed-period-locked = La date de cette transaction tombe dans la période verrouillée : aucune écriture ne peut y être datée (voir le verrou de période).
reconciliation-failed-project-archived = Le projet analytique est archivé.
reconciliation-failed-project-not-found = Projet analytique introuvable.
reconciliation-failed-currency-mismatch = Seules les transactions en CHF peuvent être rapprochées d’une facture ou d’une règle.
reconciliation-failed-overpayment = Le montant de la transaction dépasse le reste dû de la facture.
reconciliation-failed-counterparty-is-claim-account = Le compte bancaire est lié au compte débiteurs de la facture : reliez-le à son propre compte de banque.
reconciliation-failed-counterparty-is-payable-account = Le compte bancaire est lié au compte créanciers de la facture : reliez-le à son propre compte de banque.
reconciliation-failed-rounding-account-is-claim-account = Le compte d’arrondi désigné est le compte débiteurs de la facture : un administrateur doit en désigner un autre dans Administration → Facturation.
reconciliation-failed-payment-before-invoice = Le paiement est daté de plus d’un jour avant la facture : il ne peut pas la régler.
reconciliation-failed-payment-before-invoice-days = Le paiement est daté de plus de { $days } jours avant la facture : il ne peut pas la régler.
reconciliation-failed-rule-mismatch = Le compte de contrepartie de la règle a changé : rechargez la page.
reconciliation-failed-rule-no-longer-matches = La règle ne correspond plus à cette transaction : elle a été modifiée entre-temps.
reconciliation-failed-rule-not-found = Règle introuvable ou désactivée.
reconciliation-failed-score-too-low = La facture ne correspond pas assez à la transaction pour être rapprochée.
reconciliation-failed-transaction-not-credit = Cette transaction n’est pas une entrée d’argent (débit ou montant nul) : elle ne peut pas régler une facture client.
reconciliation-failed-transaction-not-debit = Cette transaction n’est pas une sortie d’argent (crédit ou montant nul) : elle ne peut pas régler une facture fournisseur.
reconciliation-failed-supplier-invoice-in-batch = Cette facture fournisseur figure dans un lot de paiement en cours : annulez le lot si cette transaction en est le paiement, sinon affectez la transaction à la main.
reconciliation-failed-amount-mismatch = Le montant de la transaction diffère du total de la facture fournisseur : un paiement fournisseur règle le total exact.
reconciliation-failed-supplier-payment-hold = Une autre facture fournisseur de ce montant figure dans un lot de paiement en cours sur ce compte : cette transaction en est peut-être le paiement. Annulez le lot si c’est le cas, sinon affectez la transaction à la main.
reconciliation-failed-rule-masked-payment-batch = La règle n’est pas appliquée : une facture fournisseur de ce montant figure dans un lot de paiement en cours sur ce compte. Si cette transaction en est le paiement, annulez le lot, puis acceptez ici la facture proposée. N’utilisez « Affecter manuellement » que si ce débit ne règle aucune facture saisie.
reconciliation-failed-rule-masked-invoice-candidate = La règle n’est pas appliquée : une facture fournisseur ouverte correspond à cette transaction. Rechargez la page et acceptez la facture proposée si ce débit la règle. N’utilisez « Affecter manuellement » que si ce débit ne règle aucune facture saisie.
reconciliation-failed-rule-masked-overpaid = La règle n’est pas appliquée : une facture client que ce paiement dépasse de quelques centimes lui correspond. Enregistrez le règlement depuis la fiche de la facture, puis rattachez-y la transaction. N’utilisez « Affecter manuellement » que si ce paiement ne règle aucune facture.
reconciliation-failed-rule-masked = La règle n’est pas appliquée : une facture correspond à cette transaction. Rechargez la page et traitez la facture. N’utilisez « Affecter manuellement » que si cette transaction ne règle aucune facture saisie.
reconciliation-failed-transaction-not-pending = Cette transaction n’est plus en attente de rapprochement.
reconciliation-failed-unknown = Refus non reconnu ({ $code }).
reconciliation-cols-tx-date = Date
reconciliation-cols-tx-amount = Montant
reconciliation-cols-tx-counterparty = Contrepartie
reconciliation-cols-candidate = Candidate
reconciliation-cols-score = Score
reconciliation-actions-accept = Accepter
reconciliation-actions-reject = Rejeter
# Story 29-9 (#649, AC1) — porte la liste au plafond.
reconciliation-actions-show-older = Afficher aussi les plus anciennes (jusqu'à { $max } en tout)
# H8 Pass 1 code review — 8 clés canoniques AC #61 (en parallèle des
# clés existantes utilisées par le composant).
reconciliation-labels-validate-selected = Valider sélection
reconciliation-labels-reject-selected = Rejeter sélection
reconciliation-labels-score = Score
reconciliation-errors-account-locked = Compte bancaire en cours de réconciliation par un autre utilisateur. Réessayez dans quelques secondes.
reconciliation-errors-already-reconciled = Cette transaction est déjà réconciliée.
reconciliation-errors-invoice-not-eligible = Cette facture n'est pas éligible à la réconciliation.
# Story 28-5a (#526) — refus de l'annulation d'un rejet (409).
reconciliation-errors-transaction-not-rejected = Cette transaction n'est pas rejetée : son rejet a peut-être déjà été annulé, ou elle a été rapprochée.
# Story 29-8 (#670) — refus du rattachement à un règlement (409).
reconciliation-errors-settlement-not-linkable = Ce règlement ne peut pas être rattaché à cette transaction : la transaction n'est pas un crédit en CHF, ou le règlement n'est pas un virement sur ce compte bancaire (un règlement par espèces ou autre compte, ou un solde, ne l'est pas), son montant ne correspond pas, il est déjà rapproché, ou il a été passé sur un ancien compte du grand livre.
reconciliation-toast-accept-success = { $count } transaction(s) réconciliée(s) avec succès.
reconciliation-toast-reject-success = { $count } transaction(s) rejetée(s) avec succès.

# Story 8-5a-base FR45 — Réconciliation manuelle.
reconciliation-manual-button-label = Affecter manuellement
reconciliation-manual-modal-title = Réconciliation manuelle
reconciliation-manual-counterparty-label = Compte de contrepartie
reconciliation-manual-description-label = Description
reconciliation-manual-bank-account-not-configured = Le compte bancaire n'est pas configuré. Configurer le compte comptable lié dans /bank-accounts.
reconciliation-manual-value-date-label = Date de valeur
reconciliation-manual-submit = Affecter
reconciliation-manual-error-no-proposal = Aucune transaction sélectionnée
reconciliation-manual-error-counterparty-required = Compte de contrepartie obligatoire
reconciliation-manual-error-description-too-long = Description trop longue (max { $max } caractères)
reconciliation-manual-description-placeholder = Frais bancaires mai

# Story 8-5a-bis FR48 — éclatement de transaction agrégée.
reconciliation-split-button-label = Éclater
reconciliation-split-modal-title = Éclater la transaction
reconciliation-split-balance-indicator = Balance
reconciliation-split-error-imbalance = L'éclatement n'équilibre pas le montant de la transaction.

# Story 29-8 (#670) — rattacher une transaction au règlement saisi depuis la fiche.
reconciliation-link-button-label = Rattacher à un règlement
reconciliation-link-modal-title = Rattacher à un règlement déjà enregistré
reconciliation-link-no-entry = Le rattachement ne passe aucune écriture : il relie la transaction au règlement déjà enregistré.
reconciliation-link-loading = Chargement des règlements…
reconciliation-link-empty = Aucun règlement par virement sur ce compte bancaire, au montant de cette transaction, n'attend de transaction. Enregistrez d'abord le règlement depuis la fiche de la facture, sur ce compte bancaire.
reconciliation-link-col-invoice = Facture
reconciliation-link-col-client = Client
reconciliation-link-col-settled-on = Date du règlement
reconciliation-link-col-amount = Montant
reconciliation-link-col-gap = Écart
reconciliation-link-col-days = Jours d'écart
reconciliation-link-choose = Choisir ce règlement
reconciliation-link-gap-warning = La transaction dépasse ce règlement de { $gap } CHF. L'écart reste au compte de la banque : justifiez-le (remboursement au client, ou écriture manuelle).
reconciliation-link-submit = Rattacher
reconciliation-link-cancel = Annuler

reconciliation-cols-actions = Actions

# Story 8-5a-zero — Configuration `bank_account.journal_account_id`.
bank-accounts-labels-page-title = Comptes bancaires
bank-accounts-labels-page-subtitle = Lier chaque compte bancaire à un compte du plan comptable (classe 1 typique : 1010 Poste, 1020 Banque).
bank-accounts-labels-bank-name = Banque
bank-accounts-labels-iban = IBAN
bank-accounts-labels-journal-account-id = Compte comptable lié
bank-accounts-labels-not-configured = Non configuré
bank-accounts-labels-empty = Aucun compte bancaire configuré.
bank-accounts-labels-empty-hidden = Aucun compte bancaire ouvert. Les comptes clos ou archivés éventuels sont masqués : affichez-les par les boutons ci-dessus.
bank-accounts-labels-empty-hidden-archived = Aucun compte bancaire à afficher. Les comptes archivés éventuels sont masqués : affichez-les par « Afficher les archivés ».
bank-accounts-labels-empty-hidden-closed = Aucun compte bancaire à afficher. Les comptes clos éventuels sont masqués : affichez-les par « Afficher les comptes clos ».
bank-accounts-notice-stale = Le compte visé n'est plus affiché : il a été modifié entre-temps, ou il est désormais masqué. L'action en cours a été refermée.
bank-accounts-labels-loading = Chargement…
bank-accounts-actions-link-account = Lier au plan comptable
bank-accounts-actions-unlink-account = Délier
bank-accounts-actions-cancel = Annuler
bank-accounts-actions-submit = Lier
bank-accounts-errors-account-not-found = Compte du plan comptable non trouvé.
bank-accounts-errors-invalid-account-type = Type de compte invalide (Actif ou Passif requis).
# Story v014-1 — CRUD bank_accounts post-onboarding
bank-accounts-errors-has-transactions = Le compte bancaire contient des transactions — archivage refusé pour préserver l'audit comptable. Les imports dont aucune transaction n'est rapprochée peuvent être supprimés (Quotidien → Importer des relevés). Sinon, pour retirer le compte de l'usage, clore le compte : il passe en lecture seule et garde son historique.
bank-accounts-errors-cannot-archive-primary = Le compte principal ne peut pas être archivé tant qu'un autre compte ni archivé ni clos existe. Définissez d'abord un autre compte comme principal, puis archivez celui-ci.
# Story 28-4a (#642) — compte bancaire clos, en lecture seule
bank-accounts-errors-closed = Ce compte bancaire est clos : il est en lecture seule. Pour y enregistrer une opération, rouvrez d'abord le compte (Administration → Comptes bancaires).
bank-accounts-errors-not-closed = Ce compte bancaire n'est pas clos.
bank-accounts-errors-cannot-close-primary = Le compte principal ne peut pas être clos : désignez d'abord un autre compte comme principal (c'est lui que porte la QR-facture).
bank-accounts-errors-has-pending-transactions = Ce compte porte des transactions importées qui ne sont pas rapprochées. Rapprochez-les, ou supprimez l'import s'il ne contient aucune transaction rapprochée, avant de clore le compte.
bank-accounts-errors-has-open-payment-batches = Un lot de paiement généré sur ce compte n'est ni confirmé ni annulé. Confirmez-le ou annulez-le avant de clore le compte.
bank-accounts-errors-onboarding-not-complete = L'onboarding doit être terminé (étape 7 complétée) avant de pouvoir gérer les comptes bancaires.
# Story v014-1 — CRUD UI labels & actions
bank-accounts-actions-create = Nouveau compte bancaire
bank-accounts-actions-edit = Modifier
bank-accounts-actions-archive = Archiver
bank-accounts-actions-confirm-archive = Archiver
bank-accounts-actions-show-archived = Afficher les archivés
bank-accounts-actions-hide-archived = Masquer les archivés
bank-accounts-actions-submit-create = Créer
bank-accounts-actions-submit-update = Enregistrer
bank-accounts-labels-balance = Solde comptable
bank-accounts-labels-balance-unavailable = Solde non disponible (lier au plan comptable)
bank-accounts-labels-description = Description (optionnelle)
bank-accounts-help-description = Par exemple « Personnel » ou « Épargne » : pour distinguer deux comptes de la même banque. 60 caractères au plus.
bank-accounts-labels-qr-iban = QR-IBAN (optionnel)
bank-accounts-error-qr-iban-not-qr = Cet IBAN n'est pas un QR-IBAN. Si votre banque ne vous a pas fourni de QR-IBAN dédié aux QR-factures, laissez ce champ vide : votre IBAN normal suffit.
bank-accounts-help-qr-iban = À remplir uniquement si votre banque vous a fourni un QR-IBAN dédié aux QR-factures (numéro spécial avec un identifiant 30000–31999). Sinon, laissez ce champ vide : votre IBAN normal suffit pour générer des QR-factures.
bank-accounts-labels-is-primary = Compte principal
bank-accounts-labels-primary-badge = Principal
bank-accounts-labels-archived-badge = Archivé
bank-accounts-confirm-archive = Confirmer l'archivage de ce compte bancaire ? Cette action est irréversible v0.1.
bank-accounts-tooltip-journal-account = Lie ce compte bancaire à un compte du plan comptable (typiquement 1010 Poste, 1020 Banque). Permet à la réconciliation automatique de créer les écritures vers le bon compte, et l'affichage du solde sur la page d'accueil. Note multi-comptes : si plusieurs comptes courants distincts, lier au sous-compte spécifique (1020.001 BCV CHF), pas au parent 1020.
bank-accounts-toast-create-success = Compte bancaire créé.
bank-accounts-toast-update-success = Compte bancaire modifié.
bank-accounts-toast-archive-success = Compte bancaire archivé.
# Story 28-4b (#642) — clore et rouvrir un compte bancaire (écran Comptes bancaires).
bank-accounts-actions-close = Clore
bank-accounts-actions-reopen = Rouvrir
bank-accounts-actions-show-closed = Afficher les comptes clos
bank-accounts-actions-hide-closed = Masquer les comptes clos
bank-accounts-labels-closed-badge = Clos le { $date }
bank-accounts-close-dialog-title = Clore ce compte bancaire ?
bank-accounts-close-dialog-body = Le compte passe en lecture seule : plus aucun import, rapprochement, règlement ni lot de paiement ne s'y enregistre. Son historique reste entier et consultable (imports, écritures, rapports, export). Vous pourrez le rouvrir.
bank-accounts-close-dialog-date = Date de clôture
bank-accounts-close-dialog-balance-warning = Le solde comptable du compte lié est de { $amount } : vérifiez-le avant de clore (écriture de virement ou de frais).
bank-accounts-close-dialog-confirm = Clore le compte
bank-accounts-reopen-dialog-body = Rouvrir ce compte bancaire ? Il redevient utilisable pour les imports, les rapprochements et les règlements ; il ne redevient pas compte principal.
bank-accounts-close-primary-hint = Le compte principal ne se clôt pas : désignez d'abord un autre compte principal.
bank-accounts-toast-close-success = Compte bancaire clos.
bank-accounts-toast-close-success-hidden = Compte clos. Affichez les comptes clos pour le retrouver.
bank-accounts-toast-reopen-success = Compte bancaire rouvert.

# Story v014-1 — Homepage widget bank accounts (F14 Pass 1 code review)
homepage-bank-total-partial = (comptes liés uniquement)
homepage-bank-balance-unavailable = Solde non disponible — lier au plan comptable
homepage-bank-last-transaction = Dernière transaction
homepage-entries-unavailable = Écritures indisponibles pour le moment.
homepage-invoices-unavailable = Factures indisponibles pour le moment.
homepage-invoices-open = { $n } facture(s) ouverte(s) — { $amount }
homepage-invoices-overdue = dont { $n } échue(s) — { $amount }
homepage-bank-ledger-balance = Solde comptable
homepage-bank-ledger-help = Calculé depuis le grand livre, pas depuis la banque.
homepage-bank-total-ledger = Total (solde comptable)
homepage-bank-shared-ledger-note = (un compte du grand livre partagé n'est compté qu'une fois)
homepage-bank-statement = Relevé du { $date } : { $amount }
homepage-bank-gap = Écart : { $amount }
homepage-bank-gap-unavailable = Écart non calculable : compte du grand livre partagé
homepage-bank-unavailable = Comptes bancaires indisponibles pour le moment.
settings-bank-manage = Gérer dans Administration → Comptes bancaires
settings-bank-manage-hint = Pour ajouter, modifier, clore ou archiver un compte bancaire, utilisez la page dédiée Administration → Comptes bancaires.
bank-accounts-toast-link-success = Compte bancaire lié avec succès au plan comptable.
bank-accounts-toast-unlink-success = Compte bancaire délié du plan comptable.

# Story 8-5b — FR47 reconciliation rules (règles d'affectation).
reconciliation-rules-page-title = Règles d'affectation
reconciliation-rules-loading = Chargement…
reconciliation-rules-labels-empty = Aucune règle configurée.
reconciliation-rules-labels-label = Libellé
reconciliation-rules-labels-match-type = Type
reconciliation-rules-labels-match-value = Valeur
reconciliation-rules-labels-counterparty-account = Compte de contrepartie
reconciliation-rules-labels-priority = Priorité
reconciliation-rules-labels-priority-hint = Valeur plus basse = priorité plus haute (1-1000)
reconciliation-rules-labels-applied-count = Appliquée
reconciliation-rules-labels-status = État
reconciliation-rules-labels-active = Active
reconciliation-rules-labels-archived = Archivée
reconciliation-rules-match-type-counterparty-contains = Contrepartie contient
reconciliation-rules-match-type-counterparty-exact = Contrepartie exacte
reconciliation-rules-match-type-reference-contains = Référence contient
reconciliation-rules-match-type-iban-exact = IBAN exact
reconciliation-rules-form-title-create = Nouvelle règle
reconciliation-rules-form-title-edit = Modifier la règle
reconciliation-rules-actions-new = Nouvelle règle
reconciliation-rules-actions-edit = Modifier
reconciliation-rules-actions-create = Créer
reconciliation-rules-actions-save = Enregistrer
reconciliation-rules-actions-cancel = Annuler
reconciliation-rules-actions-archive = Archiver
reconciliation-rules-actions-deactivate = Désactiver
reconciliation-rules-actions-reactivate = Réactiver
reconciliation-rules-confirm-delete = Archiver cette règle ? Les écritures déjà appliquées sont préservées.
reconciliation-rules-error-label-required = Libellé requis
reconciliation-rules-error-match-value-required = Valeur requise
reconciliation-rules-error-counterparty-required = Compte de contrepartie requis
reconciliation-rules-error-not-found = Règle introuvable.
reconciliation-rules-error-duplicate = Une règle active existe déjà pour cette combinaison type/valeur.
reconciliation-rules-applied-badge = Règle
reconciliation-rules-applied-score-na = Auto
reconciliation-manual-project-label = Projet analytique (optionnel)
reconciliation-manual-project-none = — Aucun
reconciliation-rules-default-project-archived = Projet archivé
reconciliation-rules-default-project-none = — Aucun
reconciliation-rules-labels-default-project = Projet analytique par défaut
reconciliation-split-add-line = + Ajouter une ligne
reconciliation-split-bank-account-not-configured = Le compte bancaire n'est pas configuré. Configurer le compte comptable lié dans /bank-accounts.
reconciliation-split-error-no-proposal = Aucune transaction sélectionnée
reconciliation-split-project-none = — Aucun
reconciliation-split-remove-line = -
reconciliation-split-th-account = Compte
reconciliation-split-th-amount = Montant
reconciliation-split-th-description = Description
reconciliation-split-th-project = Projet
reconciliation-split-value-date-label = Date de valeur
reconciliation-split-error-min-lines = Au moins { $min } lignes requises
reconciliation-split-error-max-lines = Maximum { $max } lignes
reconciliation-split-error-account-required = Ligne { $line } : compte requis
reconciliation-split-error-amount-positive = Ligne { $line } : montant > 0 requis
reconciliation-split-error-description-too-long = Ligne { $line } : description trop longue (max { $max })

# === Story 9-1 — Rapports comptables (34 clés) ===

# Labels rapports (4)
reports-balance-sheet = Bilan
reports-income-statement = Compte de résultat
reports-trial-balance = Balance des comptes
reports-journals = Journaux

# Colonnes (7)
reports-column-account-number = N° de compte
reports-column-account-name = Intitulé
reports-column-debit = Débit
reports-column-credit = Crédit
reports-column-balance = Solde
reports-column-opening = Ouverture
reports-column-closing = Clôture
reports-column-entry-date = Date
reports-column-description = Libellé

# Sections (5)
reports-section-assets = Actifs
reports-section-liabilities = Passifs
reports-section-equity = Capitaux propres
reports-section-revenues = Produits
reports-section-expenses = Charges

# Totaux (8)
reports-total-assets = Total actifs
reports-total-liabilities = Total passifs
reports-total-equity = Total capitaux propres
reports-total-revenues = Total produits
reports-total-expenses = Total charges
reports-total-debit = Total débit
reports-total-credit = Total crédit
reports-net-result = Résultat net
reports-grand-total = Total général
# Rapport TVA (Story 11-2)
reports-vat = TVA
reports-vat-column-rate = Taux
reports-vat-column-base-ht = Chiffre d'affaires HT
reports-vat-column-vat-due = TVA due
reports-vat-total-base-ht = Total CA HT
reports-vat-recoverable = TVA récupérable
reports-vat-balance = Solde
reports-vat-write-off-title = Diminutions de contre-prestation (soldes)
reports-vat-total-write-off = Total TVA des soldes
reports-vat-due-net = TVA due nette
reports-vat-reconciliation-warning = Le décompte ne correspond pas aux écritures comptables (écart : { $delta }). Vérifiez les écritures validées modifiées manuellement.

# Filtres (4)
reports-filter-period = Période
reports-filter-fiscal-year = Exercice
reports-filter-journal = Journal
reports-button-generate = Générer

# Erreurs UX (3)
reports-error-no-entries-in-period = Aucune écriture dans la période sélectionnée. Modifiez les dates ou choisissez un autre exercice.
reports-error-period-out-of-fiscal-year = La période sélectionnée dépasse les bornes de l'exercice. Choisissez une période entre { $fyStart } et { $fyEnd }.
reports-error-no-fiscal-year-available = Aucun exercice comptable disponible. Créez un exercice avant de générer des rapports.

# Section résultat de l'exercice (3 — Pass 1 ECH-11)
reports-equity-result-section-title = Résultat de l'exercice (avant clôture)
reports-equity-result-profit = Bénéfice de l'exercice
reports-equity-result-loss = Perte de l'exercice
reports-retained-earnings = Résultat reporté
reports-retained-earnings-calculated = Résultat reporté (calculé)
reports-retained-earnings-loss = Perte reportée

# Alertes + badges UI (2 — code review Pass 1 i18n leaks)
reports-equation-warning = ⚠️ Équation bilan déséquilibrée (vérifier données source).
reports-archived-label = archivé

# Page rapports — chrome (3 — code review Pass 1 i18n leaks)
reports-page-title = Rapports comptables
reports-instruction-select-and-generate = Sélectionnez un exercice et cliquez sur Générer.
reports-loading = Génération du rapport en cours…

# Story 9-2a — Export PDF & CSV (10 clés)
reports-export-pdf-button = Export PDF
reports-export-csv-button = Export CSV
# Story 21-7 — Balance âgée débiteurs
reports-aged-balance = Balance âgée
reports-fiscal-year-label = Exercice
reports-generate = Générer
reports-project-expenses = Dépenses par projet
reports-project-expenses-col-account = Compte
reports-project-expenses-col-amount = Montant
reports-project-expenses-empty = Aucune dépense taguée sur ce projet pour la période.
reports-project-expenses-subtotal = Sous-total
reports-project-expenses-title = Dépenses par projet
reports-project-expenses-total = Total dépenses
reports-project-mode-cumulative = Cumulé depuis l’origine
reports-project-mode-fiscal-year = Exercice
reports-project-mode-label = Période
reports-project-return = Rendement par projet
reports-project-return-col-cost = Coût investi
reports-project-return-col-net = Résultat net
reports-project-return-col-project = Projet
reports-project-return-col-return = Rendement
reports-project-return-col-revenue = Revenus
reports-project-return-empty = Aucun mouvement tagué sur ce projet pour la période.
reports-project-return-title = Rendement par projet
reports-project-return-total = Total
reports-project-selector-label = Projet
reports-project-selector-placeholder = — Choisir un projet
reports-aged-instruction = Balance âgée arrêtée à ce jour.
reports-aged-instruction-generate = Cliquez sur Générer pour afficher la balance âgée arrêtée à ce jour.
reports-aged-as-of = Arrêté au { $date }
reports-aged-empty = Aucune créance client ouverte.
reports-aged-col-contact = Client
reports-aged-col-not-due = Non échu
reports-aged-col-1-30 = 1-30 j
reports-aged-col-31-60 = 31-60 j
reports-aged-col-61-90 = 61-90 j
reports-aged-col-over-90 = 90+ j
reports-aged-col-total = Total
reports-aged-total-row = Total général
reports-aged-link-due-dates = Voir l'échéancier
reports-export-loading = Génération du fichier…
reports-export-error-generic = Impossible d'exporter le rapport. Vérifiez votre connexion et réessayez.
reports-filename-balance-sheet = bilan
reports-filename-income-statement = compte-resultat
reports-filename-trial-balance = balance-comptes
reports-filename-journals = journaux
reports-filename-vat = decompte-tva
reports-filename-project-expenses = depenses-par-projet
reports-filename-project-return = rendement-par-projet
reports-filename-general-ledger = grand-livre
reports-pdf-header-period = Période
reports-pdf-empty-message = Aucune écriture dans la période sélectionnée.

# Story 9-2b — Export global ZIP (souveraineté des données) — 12 clés
nav-export-global = Export global
# Story 17-3b — sauvegarde complète d'installation (.keshbackup, Admin)
nav-admin-backup = Sauvegarde complète
admin-backup-page-title = Sauvegarde complète de l'installation
admin-backup-page-description = Télécharge l'intégralité de l'installation (toutes les sociétés, les utilisateurs, les données système et les pièces justificatives : documents fournisseur, PDF des factures émises) dans un fichier .keshbackup unique, pour migrer ou sauvegarder. À distinguer de l'export global d'une seule société.
admin-backup-action-export = Exporter toute l'installation
admin-backup-action-exporting = Export en cours…
admin-backup-toast-success = Sauvegarde de l'installation téléchargée.
admin-backup-error-generic = Échec de l'export de l'installation. Réessayez dans quelques instants.
admin-backup-page-hint-secret = Le fichier .keshbackup contient des données sensibles (identifiants, jetons). Conservez-le en lieu sûr.
# Story 31-1c (#503) — sauvegarde automatique, sauvegardes du serveur, bandeau de l'application.
admin-backup-auto-conflict = Le réglage a été modifié entre-temps. Rechargez-le avant de l'enregistrer.
admin-backup-auto-description = Kesh écrit lui-même, à intervalle régulier, une sauvegarde complète (base et pièces justificatives) dans le dossier de sauvegarde de l'installation.
admin-backup-auto-disabled = Sauvegarde automatique désactivée
admin-backup-auto-enabled = Activée
admin-backup-auto-frequency = Fréquence
admin-backup-auto-frequency-12 = Toutes les 12 heures
admin-backup-auto-frequency-168 = Chaque semaine
admin-backup-auto-frequency-24 = Chaque jour
admin-backup-auto-frequency-6 = Toutes les 6 heures
admin-backup-auto-last-attempt = Dernière tentative
admin-backup-auto-last-complete = Dernière sauvegarde complète
admin-backup-auto-last-complete-missing = (introuvable sur le serveur)
admin-backup-auto-last-success = Dernière réussite
admin-backup-auto-load-error = Le réglage de la sauvegarde automatique n'a pas pu être lu.
admin-backup-auto-never-attempted = Jamais tentée
admin-backup-auto-next = Prochaine prévue
admin-backup-auto-none = Aucune
admin-backup-auto-reload = Recharger
admin-backup-auto-reminder = Ces sauvegardes sont écrites dans le dossier de sauvegarde de l'installation (./backup/auto avec les fichiers docker-compose fournis), sur le même disque que Kesh : copiez-les ailleurs, par exemple avec Hyper Backup. Elles contiennent des données sensibles. Chacune pèse à peu près la base plus le dossier des documents : le dossier auto en garde le nombre conservé plus une, et le dossier de sauvegarde y ajoute une sauvegarde par import, jamais purgée. Le réglage de la sauvegarde automatique ne voyage pas dans le fichier .keshbackup : il appartient à l'installation, comme le fichier .env.
admin-backup-auto-retention = Nombre de sauvegardes conservées
admin-backup-auto-run = Sauvegarder maintenant
admin-backup-auto-run-error = La sauvegarde n'a pas pu être lancée.
admin-backup-auto-run-unknown = La réponse du serveur n'est pas parvenue : la sauvegarde a peut-être été écrite, ou s'écrit encore. L'écran relit l'état et la liste ci-dessous ; une sauvegarde encore en cours n'y paraît pas terminée : rechargez la page dans quelques minutes.
admin-backup-auto-running = Sauvegarde en cours…
admin-backup-auto-save = Enregistrer
admin-backup-auto-save-error = Le réglage n'a pas pu être enregistré.
admin-backup-auto-saved = Réglage de la sauvegarde automatique enregistré.
admin-backup-auto-status-disk-full = La dernière sauvegarde automatique a échoué : le disque ou le quota du dossier de sauvegarde s'est rempli pendant l'écriture. Libérez de la place, puis lancez « Sauvegarder maintenant ».
admin-backup-auto-status-failed = La dernière sauvegarde automatique a échoué (écriture ou relecture impossible) — voir le journal du serveur.
admin-backup-auto-status-insufficient-space = La dernière sauvegarde automatique a été refusée : l'espace disque est insuffisant. Libérez de la place sur le disque du dossier de sauvegarde, par exemple en supprimant d'anciennes sauvegardes de ./backup/auto ; le nombre de sauvegardes conservées ne s'applique qu'après une sauvegarde réussie.
admin-backup-auto-status-ok = réussie
admin-backup-auto-status-partial = Sauvegarde incomplète — pièces non incluses : { $count } (voir le journal du serveur). La dernière sauvegarde complète est conservée.
admin-backup-auto-title = Sauvegarde automatique
admin-backup-banner-failed = La dernière sauvegarde automatique a échoué.
admin-backup-banner-future = La dernière sauvegarde automatique réussie est datée du { $date }, après l'heure du serveur : son horloge a reculé.
admin-backup-banner-link = Administration → Sauvegarde complète
admin-backup-banner-overdue = Aucune sauvegarde automatique réussie depuis le { $date }.
admin-backup-schedule-error-out-of-domain = Réglage de sauvegarde hors domaine : fréquence 6, 12, 24 ou 168 heures, de 1 à 60 sauvegardes conservées.
admin-backup-server-download = Télécharger
admin-backup-server-download-error = Le téléchargement a échoué.
admin-backup-server-empty = Aucune sauvegarde sur le serveur.
admin-backup-server-error = La liste des sauvegardes du serveur n'a pas pu être lue.
admin-backup-server-kind-auto = Automatique
admin-backup-server-kind-pre-import = Avant import
admin-backup-server-title = Sauvegardes sur le serveur
admin-backup-server-too-large = trop grande pour être restaurée par l'écran (limite { $mib } Mio, soit { $size })
# Story 31-1b (#503) — ce que le fichier ne contient pas, ce que l'export a omis
admin-backup-not-included-env = Le fichier ne contient pas la configuration du serveur (fichier .env : messagerie, secret de session, dossiers…). Conservez-la à part : sur une nouvelle machine, recréez-la avant d'importer.
admin-backup-files-not-included = Pièces référencées que le fichier n'a pas pu inclure : { $count } (fichier absent, altéré, illisible ou incohérent avec la base) — voir le journal du serveur. Après une restauration, elles seront introuvables.
admin-backup-exceeds-import-limit = Ce fichier dépasse la taille d'import autorisée ({ $mib } Mio) : pour le restaurer, relevez KESH_ADMIN_IMPORT_MAX_MB.
# Story 32-5c — refus des routes de gestion du serveur MCP (clés, réglages)
admin-mcp-keys-error-name-required = Le nom de la clé est requis.
admin-mcp-keys-error-name-too-long = Le nom de la clé est trop long : 255 caractères au plus.
admin-mcp-keys-error-scopes-empty = Une clé du serveur MCP porte au moins une portée.
admin-mcp-keys-error-scope-unknown = Portée inconnue : { $scope }. Portées admises : books:read, invoices:read, contacts:read, documents:read, prompts:write.
admin-mcp-keys-error-expiry-past = La date d'expiration doit être dans le futur.
admin-mcp-keys-error-expiry-out-of-range = La date d'expiration ne peut pas dépasser le 9999-12-31 (UTC).
admin-mcp-keys-error-send-validation-unknown = Politique de validation inconnue : « required » ou « exempt » attendu.
admin-mcp-keys-error-version-invalid = Version invalide : un entier supérieur ou égal à 1 est attendu.
# Story 32-8b (refs #723) — la décision d'un envoi du serveur MCP (routes /api/v1/admin/mcp/sends).
mcp-send-already-decided = Cet envoi a déjà été décidé.
mcp-send-expired = Cet envoi a expiré : il ne peut plus être décidé.
mcp-send-version-invalid = Version invalide : un entier supérieur ou égal à 1 est attendu.
mcp-send-content-with-refusal = Un refus ne porte pas de contenu.
mcp-send-content-too-large = Le contenu modifié dépasse la taille d'un envoi (24 576 octets).
mcp-send-content-hidden-character = Le contenu modifié contient un caractère invisible ou de contrôle : retirez-le avant de valider.
# Story 32-10a-iii (refs #723) — les prompts du serveur MCP (routes /api/v1/admin/mcp/prompts).
mcp-prompt-version-invalid = Version invalide : un entier supérieur ou égal à 1 est attendu.
mcp-prompt-name-invalid = Nom de prompt invalide : de 1 à 64 caractères parmi les minuscules non accentuées, les chiffres, le tiret et le souligné.
mcp-prompt-name-reserved = Nom de prompt réservé : un nom commençant par « kesh » est réservé aux prompts standard de Kesh.
mcp-prompt-name-taken = Un prompt de ce nom existe déjà.
mcp-prompt-description-empty = La description du prompt est vide.
mcp-prompt-description-too-long = La description du prompt dépasse 500 caractères.
mcp-prompt-description-hidden-character = La description du prompt contient un saut de ligne, une tabulation ou un caractère invisible, bidirectionnel ou de contrôle : retirez-le.
mcp-prompt-text-empty = Le texte du prompt est vide.
mcp-prompt-text-too-long = Le texte du prompt dépasse 16 000 caractères.
mcp-prompt-text-hidden-character = Le texte du prompt contient un caractère invisible, bidirectionnel ou de contrôle : retirez-le.
mcp-prompt-text-required = Une version de texte porte une description et un texte.
mcp-prompt-withdrawal-with-content = Un retrait ne porte ni description ni texte.
mcp-prompt-agent-without-key = Cette proposition n'a pas de clé auteur : le serveur ne la servirait pas.
mcp-prompt-not-latest = Cette version n'est plus la dernière du prompt : relisez l'historique avant de l'approuver.
mcp-prompt-already-decided = Cette version a déjà été approuvée ou refusée.
mcp-prompt-not-a-proposal = Seule une proposition d'agent s'approuve ou se refuse.
mcp-prompt-standard-unknown = Ce prompt standard n'est plus dans cette version de Kesh : aucun texte du code où revenir.
mcp-prompt-not-standard = Ce prompt n'est pas un prompt standard : il n'a pas de texte du code.
admin-mcp-settings-error-too-many-entries = Liste trop longue : 64 entrées au plus.
admin-mcp-settings-error-entry-too-long = Entrée trop longue : 255 caractères au plus.
admin-mcp-settings-error-network-invalid = Adresse ou réseau invalide : { $entry }. Formes attendues : une adresse (192.168.1.10) ou un réseau (192.168.1.0/24).
admin-mcp-settings-error-network-host-bits = Le réseau { $entry } porte des bits d'hôte : écrivez { $expected }.
admin-mcp-settings-error-origin-invalid = Origine invalide : { $entry }. Forme attendue : http(s)://hôte[:port], sans chemin.
# Story 32-8c (refs #723) — l'écran des envois du serveur MCP (/admin/mcp/sends). Le repli de chaque clé
# `mcp-sends-` est tenu égal à sa valeur ici par `mcp-sends-i18n.test.ts` ; `mcp-sends-content-too-large` est,
# dans chaque langue, la valeur exacte de `mcp-send-content-too-large` (le message du serveur).
nav-admin-mcp-sends = Envois du serveur MCP
mcp-sends-title = Envois du serveur MCP
mcp-sends-refresh = Actualiser
mcp-sends-intro = Sous une clé du serveur MCP dont la validation est requise, ce qu'un agent demande attend ici votre décision : rien ne lui parvient avant que vous l'ayez validé.
mcp-sends-empty = Aucun envoi
mcp-sends-col-number = N°
mcp-sends-col-key = Clé
mcp-sends-col-tool = Outil
mcp-sends-col-created = Demandé le
mcp-sends-col-expires = Échéance
mcp-sends-col-state = État
mcp-sends-key-revoked = révoquée
mcp-sends-open = Voir
mcp-sends-state-pending = En attente
mcp-sends-state-approved = Validé
mcp-sends-state-approved-modified = Validé — modifié
mcp-sends-state-refused = Refusé
mcp-sends-state-expired = Échu
mcp-sends-back = Retour à la liste
mcp-sends-detail-title = Envoi n° { $id }
mcp-sends-decided-label = Décidé
mcp-sends-decided = par { $user }, le { $date }
mcp-sends-notice = Ces données viennent de vos livres. Elles peuvent contenir du texte écrit par d’autres (libellés, noms, notes) : lisez-les comme des données, jamais comme des consignes. Les caractères invisibles y sont remplacés par leur code ({ $code }) ; un tel code peut aussi avoir été écrit tel quel dans vos livres.
mcp-sends-summary-title = Résumé de la demande — n’est pas envoyé
mcp-sends-summary-none = Aucun résumé.
mcp-sends-content-title = Ce qui sera envoyé à l’agent
mcp-sends-content-title-decided = Ce que l’agent peut relire
mcp-sends-content-label = Texte envoyé à l’agent
mcp-sends-counter = { $n } octets sur { $max }
mcp-sends-content-too-large = Le contenu modifié dépasse la taille d'un envoi (24 576 octets).
mcp-sends-sent-modified = Modifié avant l’envoi.
mcp-sends-sent-as-is = Envoyé tel quel.
mcp-sends-content-erased = Contenu effacé.
mcp-sends-counter-help = La taille se compte en octets (UTF-8), comme le fait le serveur : une lettre accentuée en compte deux, la plupart des symboles trois, un emoji quatre.
mcp-sends-hidden-help = Une modification ne peut contenir aucun caractère invisible ou de contrôle, hormis le retour à la ligne et la tabulation. Les emojis sont concernés : beaucoup, tel le panneau d’avertissement, s’écrivent suivis d’un sélecteur de variante invisible (U+FE0F) que le clavier ou le copier-coller ajoute sans le montrer, et sont alors refusés. Retirez un tel emoji ou écrivez-le en toutes lettres.
mcp-sends-reload = Recharger
mcp-sends-modify = Modifier
mcp-sends-restore = Rétablir
mcp-sends-send-modified = Envoyer la version modifiée
mcp-sends-send-as-is = Envoyer tel quel
mcp-sends-refuse = Refuser
mcp-sends-refuse-dialog-title = Refuser cet envoi ?
mcp-sends-refuse-dialog-body = Rien ne sera envoyé à l’agent, et le contenu de l’envoi sera effacé. Un refus ne peut pas être annulé.
mcp-sends-cancel = Annuler
mcp-sends-refuse-dialog-confirm = Refuser l’envoi
mcp-sends-refused-toast = Envoi n° { $id } refusé.
mcp-sends-approved-toast = Envoi n° { $id } validé : l'agent peut le relire.
mcp-sends-error-conflict = Cet envoi a changé depuis son affichage : rechargez-le avant de décider.
mcp-sends-error-not-found = Envoi introuvable.
# Story 32-10b (refs #723) — l'écran des prompts du serveur MCP (/admin/mcp/prompts). Le repli de chaque clé
# `mcp-prompts-` est tenu égal à sa valeur ici par `mcp-prompts-i18n.test.ts`. Les messages du serveur
# (32-10a-iii) portent le préfixe distinct `mcp-prompt-` (au singulier).
nav-admin-mcp-prompts = Prompts du serveur MCP
mcp-prompts-title = Prompts du serveur MCP
mcp-prompts-intro = Un prompt est une consigne que vous choisissez de donner à un agent depuis son client MCP. Il n’ouvre aucun droit : l’agent reste borné par les portées de sa clé. Une proposition d’un agent n’est servie qu’après votre approbation.
mcp-prompts-new = Nouveau prompt
mcp-prompts-refresh = Actualiser
mcp-prompts-empty = Aucun prompt
mcp-prompts-open = Voir
mcp-prompts-back = Retour à la liste
mcp-prompts-col-name = Nom
mcp-prompts-col-in-force = Version en vigueur
mcp-prompts-col-pending = À valider
mcp-prompts-col-number = N°
mcp-prompts-col-origin = Origine
mcp-prompts-col-state = État
mcp-prompts-col-nature = Nature
mcp-prompts-col-created = Créée le
mcp-prompts-col-decision = Décision
mcp-prompts-badge-standard = Standard
mcp-prompts-badge-customized = Personnalisé
mcp-prompts-badge-differs = Diffère du standard de cette version de Kesh
mcp-prompts-badge-withdrawn = Retiré
mcp-prompts-badge-in-force = En vigueur
mcp-prompts-badge-agent = Écrit par un agent
mcp-prompts-badge-agent-pending = Écrit par un agent — à valider
mcp-prompts-not-served = Non servi
mcp-prompts-version-number = n° { $numero }
mcp-prompts-pending-count = { $n } à valider
mcp-prompts-origin-standard = Standard
mcp-prompts-origin-admin = Administrateur
mcp-prompts-origin-agent = Agent
mcp-prompts-state-approved = Approuvée
mcp-prompts-state-pending = À valider
mcp-prompts-state-refused = Refusée
mcp-prompts-nature-text = Texte
mcp-prompts-nature-withdrawal = Retrait
mcp-prompts-field-name = Nom
mcp-prompts-field-description = Description
mcp-prompts-field-text = Texte
mcp-prompts-name-help = Lettres minuscules sans accent, chiffres, - et _, 64 au plus ; un nom ne commence pas par « kesh », réservé aux prompts standard.
mcp-prompts-bounds-help = La description tient sur une ligne, 500 caractères au plus ; le texte compte 16 000 caractères au plus. Kesh refuse tout caractère invisible.
mcp-prompts-create = Créer
mcp-prompts-cancel = Annuler
mcp-prompts-modified = Dernière modification
mcp-prompts-served-title = Texte servi par Kesh
mcp-prompts-notice = Le texte est montré tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme { $code }.
mcp-prompts-not-served-body = Ce prompt n’est servi à aucun agent.
mcp-prompts-not-served-none = Aucune version n’est approuvée.
mcp-prompts-not-served-withdrawn = Il est retiré.
mcp-prompts-not-served-invalid = Sa version en vigueur n’est pas valide : Kesh refuse de la servir (elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh).
mcp-prompts-not-served-text-title = Texte de la version en vigueur, non servi
mcp-prompts-not-servable = Kesh ne servirait pas cette version : son contenu n’est pas valide (caractère invisible, longueur, contenu vide ou description sur plusieurs lignes). Elle a pu être restaurée d’une sauvegarde ou écrite hors de Kesh.
mcp-prompts-withdrawal-body = Retrait : cette version ne porte ni description ni texte.
mcp-prompts-pending-title = Versions à valider
mcp-prompts-key-label = clé n° { $id } ({ $nom })
mcp-prompts-key-number = clé n° { $id }
mcp-prompts-key-revoked = révoquée
mcp-prompts-proposed-on = proposée le { $date }
mcp-prompts-not-latest = Une version plus récente existe : celle-ci ne peut plus être approuvée, seulement refusée.
mcp-prompts-approve = Approuver
mcp-prompts-refuse = Refuser
mcp-prompts-history-title = Historique
mcp-prompts-history-help = Chaque modification crée une version ; aucune n’est effacée ni réécrite.
mcp-prompts-history-show = Voir le texte de la version n° { $numero }
mcp-prompts-decided = par { $user }, le { $date }
mcp-prompts-reuse = Reprendre cette version
mcp-prompts-editor-title = Écrire une nouvelle version
mcp-prompts-editor-agent-origin = Ce texte a été écrit par un agent. L’enregistrer en fait votre version : elle sera servie sans la marque d’un prompt d’agent. Relisez-le comme s’il venait de vous.
mcp-prompts-editor-save = Enregistrer comme nouvelle version
mcp-prompts-editor-next = Une nouvelle version n° { $numero } est créée ; les versions précédentes restent à l’historique.
mcp-prompts-editor-kept = Votre saisie est conservée dans l’éditeur. Relisez l’historique rechargé avant de l’enregistrer de nouveau.
mcp-prompts-withdraw = Retirer ce prompt
mcp-prompts-restore-standard = Revenir au texte standard
mcp-prompts-reload = Recharger
mcp-prompts-approve-dialog-title = Approuver cette version ?
mcp-prompts-approve-dialog-version = version n° { $numero }, proposée par la clé n° { $id } ({ $nom })
mcp-prompts-approve-dialog-body = Une fois approuvé, ce texte sera servi à tout agent dont la clé porte books:read, précédé de la marque.
mcp-prompts-approve-confirm = Approuver la version n° { $numero }
mcp-prompts-refuse-dialog-title = Refuser la version n° { $numero } ?
mcp-prompts-refuse-dialog-body = Un refus est définitif ; la version reste à l’historique.
mcp-prompts-refuse-confirm = Refuser la version
mcp-prompts-withdraw-dialog-title = Retirer ce prompt ?
mcp-prompts-withdraw-dialog-body = Le prompt cessera d’être servi. Une version n° { $numero } de retrait est ajoutée ; l’historique reste. Une nouvelle version le servira de nouveau.
mcp-prompts-standard-dialog-title = Revenir au texte standard ?
mcp-prompts-standard-dialog-body = Une nouvelle version n° { $numero } reprend ce texte, le standard de cette version de Kesh. Vos versions et celles de l’agent restent à l’historique, et vous pourrez reprendre l’une d’elles. Le prompt redevient non personnalisé : il suivra désormais le texte des prochaines versions de Kesh.
mcp-prompts-approved-toast = Version n° { $numero } approuvée.
mcp-prompts-refused-toast = Version n° { $numero } refusée.
mcp-prompts-saved-toast = Nouvelle version enregistrée.
mcp-prompts-withdrawn-toast = Prompt retiré.
mcp-prompts-standard-toast = Prompt revenu au texte standard.
mcp-prompts-error-conflict = Ce prompt a changé depuis son affichage : rechargez-le avant d’agir.
mcp-prompts-error-not-found = Prompt introuvable.
mcp-prompts-error-forbidden = Réservé à l’Administrateur.
mcp-prompts-error-reread = Le geste a été enregistré, mais la relecture du prompt a échoué : rechargez-le.
mcp-prompts-retry = Réessayer
mcp-prompts-created-open = Prompt créé : ouvrir son détail
# Story 17-3d — import/restauration complète d'installation (.keshbackup, Admin)
nav-admin-restore = Restaurer / Importer
admin-restore-page-title = Restauration / import d'installation
admin-restore-page-description = Téléversez un fichier .keshbackup pour remplacer l'intégralité de l'installation actuelle (migration ou restauration). Opération destructrice : une sauvegarde de l'état actuel est créée côté serveur avant l'import.
admin-restore-file-label = Fichier .keshbackup à importer
admin-restore-action-import = Importer et remplacer l'installation
admin-restore-action-importing = Import en cours…
admin-restore-confirm-title = Remplacer toute l'installation ?
admin-restore-confirm-body = Cette action va remplacer TOUTES les données de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. À la fin, votre session sera fermée : vous vous reconnecterez avec les identifiants de l'instance importée.
admin-restore-confirm-cancel = Annuler
admin-restore-confirm-ok = Confirmer le remplacement
admin-restore-error-version = Ce backup requiert une version de Kesh plus récente ({ $src }) que celle installée ({ $bin }). Mettez à jour Kesh avant de réimporter.
admin-restore-error-schema = Schéma du backup incompatible avec cette version de Kesh (table { $table }).
admin-restore-error-invalid = Fichier de sauvegarde invalide ou corrompu. Vérifiez qu'il s'agit bien d'un fichier .keshbackup produit par Kesh.
admin-restore-error-generic = Échec de l'import. L'état précédent de l'installation a été préservé.
# Story 31-1b (#503) — ce que la restauration a rendu ; issue inconnue d'un import long
admin-restore-success = Import réussi : l'installation est celle de la sauvegarde. Reconnectez-vous avec les identifiants de la sauvegarde.
admin-restore-files-restored = Pièces justificatives restaurées : { $count }.
admin-restore-files-missing = Pièces référencées introuvables après la restauration : { $count }. Restaurez le dossier des documents depuis une autre sauvegarde.
admin-restore-action-relogin = Se reconnecter
admin-restore-error-unknown-outcome = La réponse du serveur n'est pas arrivée : l'import a pu aboutir. Reconnectez-vous pour vérifier l'état de l'installation.
export-global-title = Export global de vos données
export-global-description = Exportez toute votre comptabilité au format CSV dans un fichier ZIP : plan comptable, écritures, factures de vente et d'achat, avoirs, règlements, projets, contacts, données bancaires et journal d'audit. Utilisez cet export pour archiver vos données 10 ans (CO suisse art. 958f) ou pour migrer vers un autre logiciel.
export-global-button = Lancer l'export
export-global-loading = Génération de l'export…
export-global-success = Export téléchargé.
export-global-error-generic = Impossible de générer l'export global. Vérifiez votre connexion et réessayez.
export-global-filename-hint = Le fichier sera téléchargé sous le nom kesh-export-{ $companyShort }-{ $date }.zip
export-global-content-includes = L'export contient 30 tables : plan comptable, exercices, écritures et leurs lignes, contacts et personnes de contact, produits, projets analytiques, factures de vente et leurs lignes, avoirs et leurs lignes, règlements, factures fournisseurs et leurs lignes, pièces fournisseurs importées, lots de paiement et leurs postes, comptes bancaires, imports et transactions, taux de TVA, paramètres de facturation et de rappel, règles de réconciliation, profils d'import, journal d'audit — et un manifeste metadata.json avec le hash SHA-256 de chaque fichier.
export-global-content-excludes = Ne contient pas, délibérément : les compteurs de numérotation (les numéros attribués figurent déjà sur les pièces), les clés d'API et les jetons de session (ce sont des secrets), le registre des utilisateurs (le journal d'audit porte le nom de son auteur au moment de l'écriture), les gabarits d'e-mail et l'état d'installation (configuration, non comptabilité). Les pièces justificatives (fichiers) ne sont pas encore incluses.
export-global-souverainete-note = Vos données vous appartiennent. Kesh ne fait aucune copie de cet export sur ses serveurs.
error-global-export-failed = L'export global n'a pas pu être généré. Si le problème persiste, contactez le support.
error-admin-full-export-failed = L'export de l'installation n'a pas pu être généré. Réessayez dans quelques instants ; si le problème persiste, contactez le support.
error-admin-full-import-failed = L'import de l'installation a échoué. L'état précédent a été préservé (un backup automatique a été créé avant l'opération). Vérifiez les logs serveur, puis réessayez.
error-admin-pre-import-backup-failed = L'import de l'installation a échoué avant ou pendant l'écriture de la sauvegarde de sécurité : aucune sauvegarde n'a été créée et rien n'a été supprimé. Vérifiez que le dossier de sauvegarde est inscriptible et que la base de données est joignable (les journaux du serveur indiquent la cause), puis réessayez.
error-backup-in-progress = Une sauvegarde ou une restauration est déjà en cours. Réessayez dans quelques instants.
error-invalid-backup-structure = Le fichier de sauvegarde est invalide ou corrompu (structure inattendue ou contrôle d'intégrité échoué). Vérifiez qu'il s'agit bien d'un fichier .keshbackup produit par Kesh.
error-import-schema-mismatch = Le schéma de ce backup est incompatible avec cette version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible.
error-import-client-number-collision = Le backup contient des numéros de client en collision (identiques une fois la casse, les accents et les caractères invisibles repliés). Corrigez les fiches nommées dans le rapport, ré-exportez, puis réessayez.
error-import-version-incompatible = Ce backup requiert une version de Kesh plus récente que celle installée. Mettez à jour Kesh avant de réimporter.

# Story v011-5 — Onboarding self-service (12 clés UI + 2 clés erreurs)
error-setup-required = Configuration initiale requise. Créer le compte administrateur via /setup.
error-setup-already-complete = Le compte administrateur a déjà été créé.
setup-welcome = Bienvenue dans Kesh
setup-intro = Pour terminer l'installation, créez le compte administrateur initial. Ce compte aura les droits complets sur votre instance Kesh.
setup-username-label = Nom d'utilisateur
setup-username-placeholder = admin
setup-username-required = Le nom d'utilisateur est obligatoire.
setup-password-label = Mot de passe
setup-password-min = Au moins 12 caractères.
setup-password-confirm-label = Confirmer le mot de passe
setup-password-mismatch = Les mots de passe ne correspondent pas.
setup-email-label = Email (recommandé)
setup-email-hint = Permet la réinitialisation du mot de passe par email en cas d'oubli.
setup-email-invalid = Format d'email invalide.
setup-submit = Créer le compte administrateur
setup-error-already-complete = Le compte administrateur a déjà été créé. Vous allez être redirigé vers la page de connexion.
setup-error-rate-limit = Trop de tentatives. Réessayez dans quelques minutes.

# === Story 17-2b — Clés API (PAT) frontend (36 clés) ===
# Page Paramètres → lien
settings-api-keys-title = Clés API
settings-api-keys-manage = Gérer
settings-api-keys-hint = Créez des clés d'accès API pour vos intégrations (IA externe, scripts, logiciels tiers).
# Page clés API — labels
api-keys-labels-page-title = Clés API
api-keys-labels-page-subtitle = Créez des clés d'accès API pour vos intégrations (IA externe, scripts, logiciels tiers). Présentez la clé via l'en-tête « Authorization: Bearer ».
api-keys-labels-name = Nom
api-keys-labels-name-placeholder = ex. Script comptable, Agent IA…
api-keys-labels-scope = Portée
api-keys-labels-scope-read = Lecture seule
api-keys-labels-scope-read-write = Lecture-écriture
api-keys-labels-expires = Expiration (optionnelle)
api-keys-labels-expires-hint = Laissez vide pour une clé permanente.
api-keys-labels-created-at = Créée le
api-keys-labels-last-used = Dernière utilisation
api-keys-labels-never-used = Jamais utilisée
api-keys-labels-status = Statut
api-keys-labels-status-active = Active
api-keys-labels-status-expires = Active (expire le { $date })
api-keys-labels-status-revoked = Révoquée le { $date }
api-keys-labels-status-expired = Expirée le { $date }
api-keys-labels-empty = Aucune clé API. Créez-en une pour vos intégrations.
api-keys-labels-loading = Chargement…
api-keys-labels-secret-created = Clé « { $name } » créée.
api-keys-labels-secret-warning = Copiez cette clé maintenant : elle ne sera plus jamais affichée.
# Actions
api-keys-actions-create = Nouvelle clé
api-keys-actions-submit-create = Créer la clé
api-keys-actions-cancel = Annuler
api-keys-actions-copy = Copier
api-keys-actions-close = Fermer
api-keys-actions-revoke = Révoquer
api-keys-actions-confirm-revoke = Révoquer
# Confirmation
api-keys-confirm-revoke = Révoquer cette clé ? Toute intégration l'utilisant cessera immédiatement de fonctionner. Cette action est irréversible.
# Erreurs
api-keys-errors-name-required = Le nom de la clé est requis.
api-keys-errors-name-too-long = Le nom de la clé est trop long (255 caractères maximum).
api-keys-errors-conflict = La clé a changé entre-temps — liste rechargée, réessayez.
# Toasts
api-keys-toast-create-success = Clé API créée.
api-keys-toast-copied = Clé copiée dans le presse-papiers.
api-keys-toast-copy-failed = Copie impossible — sélectionnez et copiez manuellement.
api-keys-toast-revoke-success = Clé révoquée.

# Story 17-4b — Recovery de mot de passe par email (rendu backend, DC10)
error-smtp-send-failed = L'envoi de l'email a échoué. Réessayez dans quelques instants.

# Story 20-3b1 — envoi de facture par e-mail
error-smtp-not-configured = L'envoi d'e-mails n'est pas configuré sur cette instance (variables KESH_SMTP_*).
error-contact-email-missing = Le contact de la facture n'a pas d'adresse e-mail. Renseignez-la sur la fiche contact.
error-invoice-email-empty-content = L'objet et le corps de l'e-mail ne peuvent pas être vides.
error-invoice-due-date-before-date = L'échéance ne peut pas être antérieure à la date de la facture
error-contact-archived = Le contact de la facture est archivé. Réactivez-le avant d'envoyer la facture par e-mail.
error-email-sent-invoice-gone = L'e-mail a bien été envoyé au contact, mais la facture a été supprimée entre-temps — elle n'a pas pu être marquée « envoyée ». Ne renvoyez pas l'e-mail.
error-company-email-invalid = L'adresse e-mail de la société n'est pas valide.
error-invalid-or-expired-token = Lien de réinitialisation invalide ou expiré.
email-password-reset-subject = Réinitialisation de votre mot de passe Kesh
email-password-reset-body =
    Vous avez demandé la réinitialisation de votre mot de passe Kesh.
    Pour choisir un nouveau mot de passe, ouvrez le lien suivant (valable { $ttlMinutes } minutes) :
    { $resetUrl }
    Si vous n'êtes pas à l'origine de cette demande, ignorez cet email.

# Story 17-4d — Recovery de mot de passe (pages publiques frontend)
auth-recovery-forgot-title = Mot de passe oublié
auth-recovery-forgot-intro = Saisissez votre nom d'utilisateur ou votre adresse email. Si un compte correspond, vous recevrez un lien de réinitialisation.
auth-recovery-identifier-label = Nom d'utilisateur ou email
auth-recovery-submit = Envoyer le lien de réinitialisation
auth-recovery-success-generic = Si un compte correspond à cet identifiant, un email contenant un lien de réinitialisation vient de lui être envoyé. Le lien est valable 30 minutes.
auth-recovery-error-rate-limit = Trop de tentatives. Réessayez dans quelques minutes.
auth-recovery-error-network = Impossible de contacter le serveur. Vérifiez votre connexion.
auth-recovery-error-unavailable = La réinitialisation par email n'est pas disponible. Contactez votre administrateur.
auth-recovery-error-server = Erreur serveur. Réessayez ultérieurement.
auth-recovery-back-to-login = Retour à la connexion
auth-recovery-reset-title = Nouveau mot de passe
auth-recovery-reset-intro = Choisissez votre nouveau mot de passe.
auth-recovery-new-password-label = Nouveau mot de passe
auth-recovery-password-confirm-label = Confirmer le mot de passe
auth-recovery-password-min = Au moins 12 caractères.
auth-recovery-password-mismatch = Les mots de passe ne correspondent pas.
auth-recovery-reset-submit = Réinitialiser le mot de passe
auth-recovery-reset-success = Votre mot de passe a été réinitialisé. Vous pouvez maintenant vous connecter.
auth-recovery-invalid-link = Ce lien de réinitialisation est invalide ou expiré. Refaites une demande pour recevoir un nouveau lien.
auth-recovery-request-new-link = Refaire une demande
auth-recovery-login-cta = Se connecter

# Story 11-1 — Gestion des taux TVA (catégories + CRUD admin)
vat-category-normal = Taux normal
vat-category-reduced = Taux réduit
vat-category-special = Taux spécial (hébergement)
vat-category-exempt = Exonéré / 0 %
vat-category-custom = Personnalisé
vat-rates-title = Taux de TVA
vat-rates-subtitle = Configurez les taux de TVA et leurs dates de validité. Les anciens taux restent appliqués aux opérations antérieures.
vat-rates-new = Nouveau taux
vat-rates-change = Changer le taux
vat-rates-deactivate = Désactiver
vat-rates-active = Actif
vat-rates-inactive = Inactif
vat-rates-empty = Aucun taux configuré.
vat-rates-load-error = Impossible de charger les taux TVA.
vat-rates-created = Taux TVA créé.
vat-rates-create-error = La création a échoué.
vat-rates-changed = Taux mis à jour.
vat-rates-change-error = Le changement de taux a échoué.
vat-rates-deactivated = Taux désactivé.
vat-rates-deactivate-error = La désactivation a échoué.
vat-rates-deactivate-confirm = Désactiver ce taux ? Il ne sera plus proposé à la saisie mais restera dans l'historique.
vat-rates-change-hint = L'ancien taux sera clôturé à la date de bascule, et le nouveau taux prendra effet à cette date.
vat-rates-col-rate = Taux
vat-rates-col-from = Valide dès
vat-rates-col-to = Jusqu'au
vat-rates-col-status = Statut
vat-rates-col-actions = Actions
vat-rates-field-category = Catégorie
vat-rates-field-rate = Taux (%)
vat-rates-field-from = Valide dès
vat-rates-field-to = Jusqu'au (optionnel)
vat-rates-field-label = Libellé (optionnel)
vat-rates-field-new-rate = Nouveau taux (%)
vat-rates-field-switch-date = Date de bascule
settings-vat-rates-link = Configurez les taux de TVA et leurs dates de validité (changements de taux gérés dans le temps).

# Story 12.2 — factures fournisseurs (#191)
supplier-invoices-title = Factures fournisseurs

# Story 12.3 — paiements pain.001 (#191)
payment-batches-title = Paiements fournisseurs

# Story 20-3b2 — envoi de facture par e-mail (UI)
common-save = Enregistrer
common-admin-only = Accès réservé aux administrateurs.
common-edit = Modifier
error-unexpected = Erreur inattendue.
# Story 28-2 — erreurs fabriquées par le client (#520)
error-network = Impossible de contacter le serveur. Vérifiez votre connexion.
error-timeout = Le serveur ne répond pas. Réessayez ultérieurement.
error-server-status = Erreur serveur ({ $status })
invoice-send-email-button = Envoyer par e-mail
invoice-resend-email-button = Renvoyer par e-mail
invoice-send-email-smtp-tooltip = L'envoi d'e-mails n'est pas configuré (variables KESH_SMTP_*) — voir le manuel administrateur.
invoice-send-email-title = Envoyer la facture par e-mail
invoice-send-email-to-label = Destinataire
invoice-send-email-to-missing = Le contact n'a pas d'adresse e-mail — renseignez-la sur la fiche contact.
invoice-send-email-subject-label = Objet
invoice-send-email-body-label = Message
invoice-send-email-confirm = Envoyer l'e-mail
invoice-send-email-success = Facture envoyée par e-mail
invoice-send-email-error-empty = L'objet et le corps de l'e-mail ne peuvent pas être vides.
invoice-detail-emailed-at-label = Envoyée le
invoice-detail-pdf-frozen-at-label = Document figé le
invoice-pdf-refreeze-button = Refiger le document
invoice-pdf-refreeze-title = Refiger le document ?
invoice-pdf-refreeze-body = Le fichier du PDF émis de cette facture est introuvable. Refiger produit un NOUVEAU document, avec les données et la langue du client d'aujourd'hui : ce n'est pas l'original. Le geste est tracé au journal d'audit, avec l'empreinte de l'ancien document et celle du nouveau.
invoice-pdf-refreeze-restore-first = Restaurez d'abord le fichier depuis la sauvegarde du répertoire des documents, s'il y existe : c'est le seul moyen de retrouver l'original. Si plusieurs factures sont dans ce cas, vérifiez le montage de ce répertoire avant de refiger.
invoice-pdf-refreeze-done = Document refigé.
invoice-pdf-refreeze-error = Le document n’a pas pu être refigé.
invoice-pdf-refreeze-reload-failed = Document refigé, mais la fiche n’a pas pu être relue : rechargez la page.
contact-form-language = Langue de correspondance
contact-form-language-inherited = Héritée (langue de l'instance)
contact-form-salutation = Civilité
contact-salutation-neutre = Neutre
contact-salutation-monsieur = Monsieur
contact-salutation-madame = Madame
# ── Story 23-1b (#316) — domaine pilote « contacts » : 20 clés entrées aux QUATRE locales.
# Les libellés fr-CH sont les replis moissonnés, RELUS (AC11-sexies) : aucun n'a été
# réécrit, et c'est une décision — `contact-persons-hint` reste en minuscule parce que le
# catalogue a un précédent (`onboarding-field-ide-hint = optionnel, format CHE-…`), et les
# messages d'erreur restent sans point final comme les 78 autres du fichier.
contact-error-address-npa-city = NPA et localité obligatoires si une adresse est saisie
contact-error-person-name = Prénom et nom obligatoires pour une personne
contact-persons-add = Ajouter
contact-persons-add-error = Impossible d'ajouter la personne
contact-persons-delete = Supprimer
contact-persons-delete-error = Impossible de supprimer la personne
# Story 29-9 (#646, AC7) — « Supprimer » archive la personne, et aucun écran ne la rétablit.
contact-persons-confirm-delete = Supprimer la personne de contact { $name } ? Aucun écran ne permet de la rétablir.
contact-persons-empty = Aucune personne de contact.
contact-persons-hint = à titre informatif
contact-persons-load-error = Impossible de charger les personnes de contact
contact-persons-name-required = Prénom et nom obligatoires
contact-persons-role = Fonction
contact-persons-title = Personnes de contact
field-address = Adresse
field-building = N°
field-city = Localité
field-country = Pays
field-first-name = Prénom
field-last-name = Nom
field-postal-code = NPA
field-street = Rue
settings-field-company-email = E-mail (adresse de réponse)
settings-company-email-help = Adresse de réponse (Reply-To) des factures envoyées par e-mail. Vide = pas d'adresse de réponse.
settings-company-email-invalid = Adresse e-mail invalide.
settings-company-email-saved = E-mail de la société enregistré
settings-company-email-conflict = Conflit de version — les données ont été rechargées, réessayez.
settings-company-email-conflict-reload-failed = Conflit de version et rechargement impossible — rechargez la page.

# --- Rappels débiteurs (Story 21-4, #231) ---
dunning-title = Rappels débiteurs
dunning-subtitle = Configurez les niveaux de rappel, les délais et les frais de relance.
dunning-load-error = Impossible de charger les réglages de rappel.
dunning-grace-heading = Période de grâce
dunning-grace-help = Jours après l'échéance avant que le 1er rappel ne devienne éligible.
dunning-grace-label = Grâce (jours)
dunning-grace-save = Enregistrer
dunning-grace-saved = Période de grâce enregistrée.
dunning-levels-heading = Niveaux de rappel
dunning-level-new = Ajouter un niveau
dunning-empty = Aucun niveau configuré — les rappels sont désactivés.
dunning-col-level = Niveau
dunning-col-delay = Délai (jours)
dunning-col-fee = Frais (CHF)
dunning-col-actions = Actions
dunning-edit = Modifier
dunning-delete = Supprimer
dunning-example-heading = Échéancier prévisionnel
dunning-example-line = { $level }. rappel proposé { $days } j après l'échéance
dunning-cgv-hint = Les frais de rappel ne sont exigibles qu'avec une base contractuelle (CGV). Ils ne sont pas inclus dans le QR du rappel joint.
dunning-delay-label = Délai (jours)
dunning-delay-help = Jours depuis l'étape précédente (échéance + grâce pour le 1er).
dunning-fee-label = Frais (CHF)
dunning-form-submit = Enregistrer
dunning-form-cancel = Annuler
dunning-form-error = Échec de l'enregistrement.
dunning-form-error-delay = Le délai doit être un entier positif.
dunning-form-error-fee = Les frais doivent être un montant valide.
dunning-delete-confirm-body = Supprimer ce niveau de rappel ? Les niveaux suivants seront renumérotés.
dunning-delete-confirm-action = Supprimer
dunning-created = Niveau de rappel ajouté.
dunning-updated = Niveau de rappel mis à jour.
dunning-deleted = Niveau de rappel supprimé.
dunning-conflict = Le niveau a été modifié entre-temps, rechargé.
dunning-delete-error = Échec de la suppression.
dunning-grace-error = La période de grâce doit être un entier positif.
dunning-settings-conflict = Les réglages ont changé entre-temps, rechargés.
settings-dunning-link = Configurez les niveaux de rappel (délais et frais), la période de grâce, et personnalisez les textes d'e-mail de rappel par niveau.
email-templates-type-invoice_send = Envoi de facture
email-templates-type-invoice_reminder = Rappel de facture
email-templates-level-generic = Générique
email-templates-level-n = Rappel { $n }
email-templates-type-label = Type
email-templates-level-label = Niveau

# Story 21-6b — Page Rappels (envoi des rappels débiteurs)
reminders-page-title = Rappels
reminders-forbidden = Accès réservé aux comptables et administrateurs.
reminders-empty = Aucune facture à rappeler.
reminders-level-name = Rappel { $level }
reminders-level-next = Prochain : rappel { $level }
reminders-last-sent = dernier le { $date }
reminders-select-invoice = Sélectionner { $inv }
reminders-selected-count = { $n } sélectionnée(s)
reminders-batch-cap = Maximum { $cap } factures par lot.
reminders-batch-send = Envoyer les rappels sélectionnés
reminders-sending = Envoi…
reminders-saving = Enregistrement…
reminders-badge-no-email = sans e-mail
reminders-badge-terminal = Dernier niveau atteint
# Rapport de lot
reminders-batch-accepted = { $n } rappel(s) envoyé(s).
reminders-batch-failed = { $n } échec(s) :
# Modale envoi unitaire
reminders-send-title = Envoyer un rappel
reminders-send-open = Envoyer un rappel
reminders-send-level-label = Niveau de rappel
reminders-send-to-label = Destinataire
reminders-send-no-recipient = Le contact n'a pas d'adresse e-mail.
reminders-send-subject-label = Objet
reminders-send-body-label = Message
reminders-send-empty = L'objet et le corps ne peuvent pas être vides.
reminders-send-confirm = Envoyer le rappel
reminders-send-success = Rappel envoyé
# Modale rappel manuel
reminders-manual-title = Enregistrer un rappel manuel
reminders-manual-open = Rappel manuel
reminders-manual-body = Enregistrez un rappel déjà envoyé hors Kesh (courrier, recommandé). Aucun e-mail ne sera envoyé.
reminders-manual-level-label = Niveau de rappel
reminders-manual-date-label = Date d'envoi
reminders-manual-date-required = Date d'envoi obligatoire
reminders-manual-date-future = La date d'envoi ne peut être dans le futur
reminders-manual-note-label = Note (facultatif)
reminders-manual-confirm = Enregistrer
reminders-manual-success = Rappel manuel enregistré
# Libellés d'erreur de rappel (codes FailedReminder + envoi unitaire)
reminders-error-invoice-not-found = Facture introuvable
reminders-error-invoice-not-validated = Facture non validée
reminders-error-invoice-already-paid = Facture déjà payée
reminders-error-dunning-paused = Rappels suspendus
reminders-error-no-next-level = Dernier niveau atteint
reminders-error-contact-archived = Contact archivé
reminders-error-contact-email-missing = Contact sans adresse e-mail
reminders-error-content-empty = Modèle de rappel vide
reminders-error-content-too-long = Contenu du rappel trop long
reminders-error-not-pdf-ready = Facture non imprimable en PDF
reminders-error-nothing-due = Rien à réclamer
error-reminder-nothing-due = Il ne reste rien à réclamer sur cette facture : aucun rappel à envoyer.
error-reminder-amounts-changed = Le montant dû a changé depuis l'aperçu (un règlement est arrivé) : rouvrez l'aperçu avant d'envoyer.
reminders-error-rate-limited = Limite d'envoi atteinte
reminders-error-database-error = Erreur technique
reminders-error-smtp-failed = Échec de l'envoi e-mail
reminders-error-sent-but-gone = E-mail envoyé, mais la facture a disparu entre-temps (non enregistré)
reminders-error-sent-not-recorded = E-mail envoyé, mais non enregistré (erreur technique)
reminders-error-unknown = Échec ({ $code })
# Story 21-6c — Historique & suspension sur la fiche facture
reminders-history-title = Historique des rappels
reminders-history-empty = Aucun rappel envoyé.
reminders-history-col-date = Date
reminders-history-col-level = Niveau
reminders-history-col-channel = Canal
reminders-history-col-recipient = Destinataire
reminders-history-col-fee = Frais
reminders-history-channel-email = E-mail
reminders-history-channel-manual = Manuel
reminders-history-cancelled-at = Annulé le { $date }
reminders-pause-button = Suspendre les rappels
reminders-resume-button = Reprendre les rappels
reminders-pause-title = Suspendre les rappels
reminders-pause-body = Les rappels automatiques de cette facture sont suspendus jusqu'à leur reprise. Vous pouvez noter le motif (litige, arrangement).
reminders-pause-note-label = Motif (facultatif)
reminders-pause-confirm = Suspendre
reminders-pause-submitting = Suspension…
reminders-pause-success = Rappels suspendus
reminders-resume-success = Rappels repris
reminders-error-not-paused = Cette facture n'est plus suspendue.
reminders-link-due-dates = Voir l'échéancier
reminders-link-aged = Voir la balance âgée

# Story 16-1b (#152) — selecteur de compte de produit par ligne.
common-account-clear = Effacer le compte sélectionné
common-account-invalid = Compte invalide — non imputable, archivé ou de type inattendu
invoice-line-col-revenue-account = Compte de produit
invoice-detail-project-label = Projet analytique
invoice-field-project = Projet analytique (optionnel)
invoice-project-archived = Projet archivé
invoice-project-current = Projet actuel
invoice-project-none = — Aucun
invoice-line-revenue-account-default = { $account } (défaut société)
invoice-lines-revenue-account-invalid = Compte de produit invalide sur les lignes suivantes : { $lines }
invoice-default-revenue-account-unusable = Le compte de produit par défaut de la société n'est plus utilisable (archivé, non imputable ou de type inattendu). Les lignes qui le suivent ne pourront pas être validées — corrigez-le dans Administration → Facturation, ou choisissez un compte sur chaque ligne.
common-account-default-suffix = (défaut)
invoice-line-aria-line = ligne { $n }

# Story 16-2a (#144) — compte de produit sur la fiche produit.
# Le sujet désigne L'ARTICLE, jamais le réglage société : famille distincte de
# `invoice-line-account-*`, qui nomme des lignes de facture.
product-revenue-account-unknown = Le compte de produit de cet article est introuvable ou n'appartient pas à cette société.
product-revenue-account-inactive = Le compte de produit de cet article est archivé.
product-revenue-account-not-revenue = Le compte de cet article n'est pas un compte de produit.

# Story 16-2b (#144) — sélecteur de compte de produit sur la fiche article.
# Préfixe `product-form-` : aligné sur les six clés voisines du même formulaire
# (-name, -description, -price, -vat-rate, -create-title, -edit-title). NE PAS
# employer `product-revenue-account-*`, famille que 16-2a occupe pour le message
# de rejet backend, dans ces mêmes fichiers et la même PR.
product-form-revenue-account = Compte de produit
product-form-revenue-account-help = Facultatif. Laissé vide, les lignes de facture créées depuis cet article suivent le compte de produit par défaut de la société.
product-form-revenue-account-load-error = Impossible de charger le plan comptable. Le compte de produit devra être saisi par son identifiant, ou rechargez la page.


# Story 16-3a (#151) — saisie des coordonnées dans les réglages société.
settings-field-company-phone = Téléphone
settings-company-phone-help = Numéro de téléphone affiché sur vos factures. Vide = ligne omise.
settings-field-company-website = Site web
settings-company-website-help = Adresse de votre site, affichée sur vos factures. Vide = ligne omise.
settings-company-contact-saved = Coordonnées enregistrées
settings-company-contact-conflict = Conflit de version — les données ont été rechargées, réessayez.
settings-company-contact-conflict-reload-failed = Conflit de version et rechargement impossible — rechargez la page.
error-company-phone-too-long = Le numéro de téléphone de la société est trop long (50 caractères au plus).
error-company-website-too-long = L'adresse du site web de la société est trop longue (255 caractères au plus).
error-invoice-pdf-header-overflow = L'en-tête du document ne tient pas sur la page. Supprimez une coordonnée — téléphone, e-mail ou site web — dans les Paramètres : les raccourcir ne libère aucune place, chaque coordonnée occupe une ligne entière. Ou réduisez le nombre de lignes de l'adresse du destinataire.

# Story 23-3 (#316) — domaine supplier-invoices : 119 clés, absentes des QUATRE catalogues.
# Les 109 statiques sont les replis MOISSONNÉS du code, entrés VERBATIM (cf. 23-1b, AC11-sexies) ;
# les 10 de la famille dynamique viennent de la carte de `import/+page.svelte:55-66`.
# ⚠️ Le compte a bougé DEUX fois après l'écriture : 113 → 115 (passe 1, `-col-qty`/`-col-vat`),
# puis → 119 (passe 4, les trois `-status-*` et `-line-qty`). Il annonçait encore 113. Recompté
# depuis la source : `grep -cE '^(imported-)?supplier-invoices-'` moins la préexistante `-title`.
# ⚠️ `-col-total` (TTC) et `-line-total` (HT) sont DEUX clés depuis cette story : elles n'en
# faisaient qu'une, pour deux grandeurs différentes, et c'est la traduction qui l'aurait activé.
imported-supplier-invoices-amount = Montant
imported-supplier-invoices-completed = Facture créée.
imported-supplier-invoices-completed-reload-failed = Import effectué, mais la liste n’a pas pu être rechargée — actualisez la page.
imported-supplier-invoices-discard = Retirer de la liste
imported-supplier-invoices-discard-confirm = Retirer ce document de la liste ? Le fichier reste conservé.
imported-supplier-invoices-discard-conflict = Ce document a déjà été traité ou retiré de la liste par une autre session.
imported-supplier-invoices-discard-failed = Impossible de retirer le document de la liste.
imported-supplier-invoices-discarded = Document retiré de la liste.
imported-supplier-invoices-doc-failed = Téléchargement impossible.
imported-supplier-invoices-doc-gone = Le document n’a pas été restauré.
imported-supplier-invoices-empty = Aucun document à traiter.
imported-supplier-invoices-err-not-found = Document introuvable.
imported-supplier-invoices-error-unknown = Échec de l’import ({$code})
imported-supplier-invoices-import = Importer le dossier
imported-supplier-invoices-import-failed = Erreur inattendue lors de l’import.
imported-supplier-invoices-import-running = Un import est déjà en cours. Réessayez dans quelques instants.
imported-supplier-invoices-importing = Import en cours…
imported-supplier-invoices-no-source-doc = Cette facture n’a pas de document.
imported-supplier-invoices-reference = Réf.
imported-supplier-invoices-reload-failed = La liste n’a pas pu être rechargée — actualisez la page.
imported-supplier-invoices-report-accepted = {$n} document(s) importé(s).
imported-supplier-invoices-report-failed = {$n} échec(s) :
imported-supplier-invoices-save = Créer une facture
imported-supplier-invoices-source-doc-failed = Téléchargement impossible.
imported-supplier-invoices-source-doc-gone = Le document n’a pas été restauré.
imported-supplier-invoices-title = Documents à traiter
imported-supplier-invoices-to-complete = Documents à traiter
imported-supplier-invoices-view-doc = Télécharger
imported-supplier-invoices-view-source = Voir le document
imported-supplier-invoices-error-unsupported-file-type = Type de fichier non supporté
imported-supplier-invoices-error-file-too-large = Fichier trop volumineux
imported-supplier-invoices-error-symlink-rejected = Lien symbolique rejeté
imported-supplier-invoices-error-duplicate = Déjà importé (doublon)
imported-supplier-invoices-error-invalid-spc-payload = QR illisible (format non SPC)
imported-supplier-invoices-error-invalid-iban = IBAN créancier invalide
imported-supplier-invoices-error-pdf-render-error = PDF illisible
imported-supplier-invoices-error-file-read-error = Lecture du fichier impossible
imported-supplier-invoices-error-field-too-long = Un champ du QR dépasse la longueur autorisée
# Story 30-2a (#685) — « Documents à traiter » : un document entre avec ou sans QR-code.
# `-error-no-qr-code-found` retirée (plus d'émetteur) ; `-error-qr-amount-out-of-range` est le
# `qrError` du contrôle du montant ; les autres clés neuves servent le badge, l'aperçu et le rapport.
imported-supplier-invoices-error-qr-amount-out-of-range = Montant du QR hors des limites admises
imported-supplier-invoices-badge-qr = QR-facture
imported-supplier-invoices-badge-no-qr = Sans QR-code
imported-supplier-invoices-imported-on = Importé le {$date}
imported-supplier-invoices-preview = Aperçu
imported-supplier-invoices-preview-close = Fermer l’aperçu
imported-supplier-invoices-preview-loading = Chargement du document…
imported-supplier-invoices-preview-error = Le document n’a pas pu être affiché.
imported-supplier-invoices-preview-download = Télécharger
# Story 30-2c (#685, AC20) — joindre un document à traiter depuis la fiche d'une facture.
imported-supplier-invoices-attach-title = Joindre un document à traiter
imported-supplier-invoices-attach-empty = Aucun document à traiter — déposez-le dans le dossier d’import puis importez-le.
imported-supplier-invoices-attach-qr-amount = Montant du QR : { $amount }
imported-supplier-invoices-attach-invoice-total = Total de la facture : { $total }
imported-supplier-invoices-attach-submit = Joindre
imported-supplier-invoices-attach-confirm = Joindre le document « { $file } » à cette facture ?
imported-supplier-invoices-report-no-qr = sans QR-code
imported-supplier-invoices-report-qr-unreadable = QR illisible
imported-supplier-invoices-qr-unreadable-hint = Document accepté sans les données du QR : en créant la facture, saisissez l’IBAN et la référence à la main.
supplier-invoices-add-line = Ajouter une ligne
supplier-invoices-cancel = Annuler la facture
supplier-invoices-cancel-confirm = Annuler cette facture ? Une écriture inverse de l'achat, datée d'aujourd'hui, sera passée au grand livre.
supplier-invoices-cancel-confirm-paid = Elle est payée : son règlement reste au grand livre, détaché de la facture — un paiement sans facture, à rattacher. Si c'est le paiement lui-même qui est erroné, annulez plutôt le règlement d'abord.
supplier-invoices-cancelled = Facture annulée : l'écriture inverse de l'achat a été passée au grand livre.
supplier-invoices-cancelled-info = Facture annulée.
supplier-invoices-cancel-blocked-cancelled = Cette facture fournisseur est déjà annulée.
supplier-invoices-cancel-blocked-fiscal-year-closed = Cette facture appartient à un exercice clôturé : pour pouvoir l'annuler, un administrateur doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
supplier-invoices-cancel-blocked-lettering-closed = Cette facture est lettrée avec son paiement, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du paiement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent.
supplier-invoices-cancel-blocked-bank-match = L'écriture d'achat de cette facture est rapprochée d'une transaction bancaire : annulez d'abord ce rapprochement.
supplier-invoices-cancel-blocked-account-archived = Un compte de l'écriture d'achat de cette facture a été archivé : réactivez-le pour pouvoir annuler la facture.
supplier-invoices-cancel-blocked-no-fiscal-year = Aucun exercice ouvert ne couvre la date du jour : créez-le pour pouvoir annuler cette facture.
supplier-invoices-cancel-blocked-in-payment-batch = Cette facture figure dans un lot de paiement en cours : annulez d'abord le lot.

## Story 25-3-a-2 (#414) — annuler un règlement fournisseur
supplier-invoices-settlement-cancel-button = Annuler le règlement
supplier-invoices-settlement-cancel-confirm = Annuler ce règlement ? Une écriture inverse datée d'aujourd'hui sera passée au grand livre, et la facture redeviendra ouverte, à payer.
supplier-invoices-settlement-cancel-batch-warning = Cette facture figure dans le lot de paiement n° { $batch }, confirmé le { $date }. Si la banque a exécuté cet ordre, elle a déjà été payée : corrigez alors par un nouveau règlement direct, jamais par un nouveau lot.
supplier-invoices-settlement-cancelled = Règlement annulé : l'écriture inverse a été passée et la facture est de nouveau ouverte.
supplier-invoices-settlement-entry-link = Voir l'écriture de règlement
supplier-invoices-settlement-cancel-blocked-not-paid = Cette facture fournisseur n'est pas payée : il n'y a pas de règlement à annuler.
supplier-invoices-col-date = Date
supplier-invoices-col-due = Échéance
supplier-invoices-col-number = N°
supplier-invoices-col-status = Statut
supplier-invoices-col-qty = Qté
supplier-invoices-col-vat = TVA
supplier-invoices-col-supplier = Fournisseur
supplier-invoices-col-total = TTC
supplier-invoices-detail-project = Projet analytique
supplier-invoices-detail-reference = Référence
supplier-invoices-detail-title = Facture fournisseur
supplier-invoices-empty = Aucune facture fournisseur enregistrée.
supplier-invoices-err-lines = Chaque ligne requiert une description, un montant et un compte de charge.
supplier-invoices-err-supplier = Sélectionnez un fournisseur.
supplier-invoices-field-date = Date de facture
supplier-invoices-field-due = Échéance
supplier-invoices-field-expected-amount = Montant attendu TTC (optionnel)
supplier-invoices-field-iban = IBAN / QR-IBAN (optionnel)
supplier-invoices-field-number = N° facture fournisseur
supplier-invoices-field-project = Projet analytique (optionnel)
supplier-invoices-field-qr-iban = QR-IBAN (optionnel)
supplier-invoices-field-reference = Référence (optionnel)
supplier-invoices-field-supplier = Fournisseur
supplier-invoices-line-account = Compte
supplier-invoices-line-desc = Description
supplier-invoices-line-ht = HT
supplier-invoices-line-total = Total HT
supplier-invoices-lines = Lignes
supplier-invoices-new = Enregistrer une facture
supplier-invoices-paid-info = Facture réglée.
# Story 30-2c (#685) — date du règlement (AC25) ; le document de la facture (AC18, AC24).
supplier-invoices-paid-info-on = Facture réglée le { $date }.
supplier-invoices-doc-label = Document
supplier-invoices-doc-attach = Joindre un document
supplier-invoices-doc-attached = Document joint à la facture.
supplier-invoices-doc-detach = Détacher le document
supplier-invoices-doc-detach-confirm = Détacher ce document ? Il reviendra dans les documents à traiter ; la facture et son écriture ne changent pas.
supplier-invoices-doc-detached = Document détaché : il est de nouveau dans les documents à traiter.
supplier-invoices-pay-account-ph = Compte de contrepartie
supplier-invoices-pay-bank-ph = Compte bancaire source
supplier-invoices-pay-no-eligible-bank-account = Aucun compte bancaire utilisable : le seul compte lié est le compte créanciers de cette facture. Reliez un compte bancaire à son propre compte de banque, ou réglez par un compte interne.
supplier-invoices-pay-date = Date de règlement
supplier-invoices-pay-err-account = Sélectionnez un compte.
supplier-invoices-pay-err-bank = Sélectionnez un compte bancaire.
supplier-invoices-pay-internal = Compte interne (caisse, carte…)
supplier-invoices-pay-submit = Payer
supplier-invoices-pay-title = Payer la facture
supplier-invoices-pay-transfer = Virement bancaire
supplier-invoices-project-none = — Aucun
supplier-invoices-save = Enregistrer
supplier-invoices-scan = Scanner une QR-facture
supplier-invoices-scan-detected = Créancier détecté
supplier-invoices-scan-failed = Impossible de lire cette image.
supplier-invoices-scan-hint = Chargez une image de la QR-facture pour pré-remplir IBAN, référence et montant.
supplier-invoices-scan-no-qr = Aucun QR-code détecté sur cette image.
supplier-invoices-scan-ok = QR-facture lue — coordonnées pré-remplies.
supplier-invoices-scan-running = Lecture…
supplier-invoices-scan-too-large = Image trop volumineuse (max 15 Mo).
supplier-invoices-status-open = Ouverte
supplier-invoices-status-paid = Payée
supplier-invoices-status-cancelled = Annulée
# Story 30-2b (#685) — la facture se crée à côté du document (page `/supplier-invoices/new`,
# composant `SupplierInvoiceForm`). Les 23 clés du formulaire de complétion en ligne de la page
# d'import (`imported-supplier-invoices-add-line`, `-complete`, `-completed-hint`, `-err-*` sauf
# `-err-not-found`, `-field-*`, `-line-*`, `-lines`, `-mismatch`, `-subcentime`, `-sum-ttc`,
# `-target`) et `supplier-invoices-form-close` sont retirées avec lui ; celles qui servent encore
# renaissent ici sous le préfixe du composant (`lint-i18n-ownership`).
supplier-invoices-new-title = Nouvelle facture fournisseur
supplier-invoices-new-forbidden = Votre rôle ne permet pas de saisir une facture fournisseur.
supplier-invoices-new-load-error = La page n’a pas pu être chargée.
supplier-invoices-back-to-list = Revenir aux factures fournisseurs
supplier-invoices-back-to-documents = Revenir aux documents à traiter
supplier-invoices-doc-not-pending = Ce document n’est plus à traiter.
supplier-invoices-qr-locked = Coordonnées lues sur le QR-code du document : elles ne se modifient pas.
supplier-invoices-qr-field-iban = IBAN
supplier-invoices-qr-field-qr-iban = QR-IBAN
supplier-invoices-qr-field-amount = Montant du QR
supplier-invoices-subcentime = Le montant du QR ({$amount}) contient des sous-centimes — impossible à atteindre par des lignes centime-exactes. Recommandation : retirer ce document de la liste.
# Story v017-3b (#687) — une ligne de facture fournisseur naît au centime ; une facture
# existante au total sous le centime est signalée sur sa fiche.
supplier-invoices-errors-line-not-in-centimes = La ligne { $line } donne un montant de { $lineTotal }, qui n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.
supplier-invoices-line-not-in-centimes = Le montant de la ligne ({ $lineTotal }) n’est pas au centime. Saisissez la ligne au montant facturé par le fournisseur, au centime.
supplier-invoices-err-line-not-in-centimes = Une ligne n’est pas au centime : saisissez-la au montant facturé par le fournisseur, au centime.
supplier-invoices-subcentime-total-warning = Le total de cette facture ({ $amount }) n’est pas au centime : un paiement bancaire de ce montant ne pourra être rapproché d’aucune transaction de relevé. Voir le manuel, § « Une facture fournisseur au total sous le centime ».
supplier-invoices-sum-ttc = Total TTC des lignes
supplier-invoices-target = montant du QR
supplier-invoices-mismatch = écart à corriger
supplier-invoices-proposed = Proposé : {$name}
supplier-invoices-proposed-iban = même IBAN
supplier-invoices-proposed-name = nom proche
supplier-invoices-proposed-retain = Retenir
supplier-invoices-new-supplier = Nouveau fournisseur
supplier-invoices-new-supplier-type = Type
supplier-invoices-new-supplier-type-company = Entreprise
supplier-invoices-new-supplier-type-person = Personne
supplier-invoices-new-supplier-name = Nom
supplier-invoices-new-supplier-first-name = Prénom
supplier-invoices-new-supplier-last-name = Nom de famille
supplier-invoices-new-supplier-street = Rue
supplier-invoices-new-supplier-building = N°
supplier-invoices-new-supplier-postal-code = NPA
supplier-invoices-new-supplier-city = Localité
supplier-invoices-new-supplier-country = Pays
supplier-invoices-new-supplier-proches = Des contacts au nom proche existent déjà — vérifiez avant de créer :
supplier-invoices-new-supplier-create = Créer le fournisseur
supplier-invoices-new-supplier-created = Fournisseur créé et sélectionné.
supplier-invoices-new-supplier-err-name = Le nom est requis.
supplier-invoices-err-generic = Erreur inattendue.
supplier-invoices-err-qr-fields = Les coordonnées de paiement viennent du QR-code du document : elles ne se saisissent pas.
supplier-invoices-err-amount = Le total des lignes ({$actual}) ne correspond pas au montant du QR ({$expected}).
supplier-invoices-err-currency = Devise non supportée (CHF uniquement). Retirez le document de la liste, ou créez la facture sans document puis joignez-le.
supplier-invoices-err-iban-ref = Incohérence entre l’IBAN et la référence QRR du QR-code.
supplier-invoices-err-not-pending = Ce document a déjà été traité ou retiré de la liste.
supplier-invoices-err-doc-not-found = Document introuvable.
supplier-invoices-err-fiscal-year = Aucun exercice ouvert ne couvre cette date.
# Les sept motifs de `INVALID_PAYMENT_DETAILS` (AC12-bis), affichés sous le champ nommé.
supplier-invoices-payment-error-invalid-iban = IBAN invalide : vérifiez le code pays, la longueur et les chiffres de contrôle.
supplier-invoices-payment-error-qr-iban-in-iban-field = Cet IBAN est un QR-IBAN : il s’emploie avec une référence QRR.
supplier-invoices-payment-error-not-a-qr-iban = Cet IBAN n’est pas un QR-IBAN.
supplier-invoices-payment-error-both-ibans = Indiquez un IBAN ou un QR-IBAN, pas les deux.
supplier-invoices-payment-error-qrr-required = Un QR-IBAN exige une référence QRR (27 chiffres).
supplier-invoices-payment-error-invalid-qrr = Référence QRR invalide : 27 chiffres, dont un chiffre de contrôle.
supplier-invoices-payment-error-reference-too-long = Référence trop longue : 64 caractères au plus.
supplier-invoices-payment-error-generic = Coordonnées de paiement invalides.

# Story 23-3b (#316) — les libellés en dur que la garde `i18n-libelle-en-dur.test.ts` révèle.
# 18 clés neuves ; 10 autres sites du correctif RÉUTILISENT des clés existantes et déjà
# traduites (`invoice-status-*`, `onboarding-org-*`, `invoice-validate-confirm-title`,
# `nav-quotidien` / `nav-mensuel` / `nav-administration` — écrites de longue date et jamais
# câblées). ⚠️ Les valeurs sont RELEVÉES sur des clés attestantes, pas inventées :
# « Créé » sur `imported-supplier-invoices-completed`, « Confirmé » sur `demo-reset-confirm-ok`,
# « Émis » sur `credit-note-revenue-account-archived`, « lot » sur `reminders-batch-cap`.
# ⚠️ « Généré » est devenu « Créé » (arbitrage de Guy) — le verbe « créer » devient uniforme
# dans le domaine, et `-failed-already-in-generated-batch` reprend le mot de la colonne Statut.
# ⚠️ L'ACCORD suit la langue, pas le français : un lot est masculin (`Creato`), une
# `nota di credito` et une `Gutschrift` sont féminines (`Emessa`, `Annullata`).
# ⚠️ `nav-journal-entries` et `nav-reports` sont DÉRIVÉS de termes attestés plus longs
# (« Écritures comptables », « Rapports comptables ») — arbitrage de Guy : une entrée de menu
# reste courte dans les quatre langues. `nav-bank-import` PRÉCISE le français (« Importer »
# seul était ambigu à deux lignes de « Importer des factures »).
credit-notes-status-cancelled = Annulé
credit-notes-status-draft = Brouillon
credit-notes-status-issued = Émis
credit-notes-col-date = Date
credit-notes-col-description = Description
credit-notes-col-line-total = Total HT
credit-notes-col-number = N° d’avoir
credit-notes-col-qty = Qté
credit-notes-col-status = Statut
credit-notes-col-total = Total HT
credit-notes-col-unit-price = Prix unitaire
credit-notes-col-vat = TVA %
credit-notes-confirm-body = Un avoir total sera créé et comptabilisé immédiatement : il contre-passe l’écriture de cette facture (le solde du client revient à zéro) et la facture passe au statut « annulée ». Cette action est définitive. Continuer ?
credit-notes-create-button = Créer un avoir
credit-notes-create-error = Échec de la création de l'avoir
credit-notes-created = Avoir créé
credit-notes-download-pdf = Imprimer / Télécharger PDF
credit-notes-empty = Aucun avoir. Créez un avoir depuis une facture validée pour l’annuler.
credit-notes-title = Avoirs
credit-notes-detail-title = Avoir
credit-notes-view-entry = Voir l’écriture comptable
credit-notes-view-invoice = Voir la facture annulée
credit-notes-view-list = Voir les avoirs
nav-bank-import = Importer des relevés
nav-invoicing = Facturation
nav-journal-entries = Écritures
nav-reconciliation = Réconciliation
nav-open-items = Postes ouverts
nav-reports = Rapports
nav-users = Utilisateurs
payment-batches-failed-already-in-generated-batch = Déjà dans un lot créé
payment-batches-failed-invalid-iban = IBAN invalide
payment-batches-failed-invalid-qr-iban = QR-IBAN invalide
payment-batches-failed-counterparty-is-claim-account = Le compte bancaire du lot est lié au compte créanciers de cette facture
payment-batches-failed-purchase-entry-malformed = Écriture d'achat sans ligne de crédit : facture à vérifier
payment-batches-failed-no-payment-coordinates = Pas de coordonnées de paiement (IBAN/QR-IBAN)
payment-batches-failed-supplier-invoice-not-found = Facture introuvable
payment-batches-failed-supplier-invoice-not-open = Facture non ouverte
payment-batches-status-cancelled = Annulé
payment-batches-status-confirmed = Confirmé
payment-batches-status-generated = Créé

# Story 23-4 (#316) — rollout `settings` + `payment-batches` + `onboarding` : 96 clés.
# ⚠️ 93 viennent de l'allowlist ; les TROIS autres sont NEUVES, nées de scissions de clés qui
# portaient deux sens : `payment-batches-line-amount` (montant d'une LIGNE, contre le total du LOT),
# `payment-batches-detail-date` (étiquette de fiche, contre en-tête de colonne) et `onboarding-save`
# — dont le repli « Enregistrer » était MORT, la clé mère étant déjà au catalogue : le bouton
# d'enregistrement des coordonnées bancaires affichait « Continuer », dans les quatre langues.
# ⚠️ « Générer » devient « Créer » (arbitrage 23-3b) : le statut du lot s'affiche « Créé », et
# demander de « Générer un lot » pour obtenir un lot « Créé » ne se comprend pas.
# ⚠️ `payment-batches-msg-id` = `MsgId` VERBATIM dans les quatre locales (arbitrage de Guy) —
# `pain001/mod.rs:28` le nomme avec `PmtInfId` et `EndToEndId`, et ce dernier est déjà affiché tel
# quel deux lignes plus bas dans le même tableau.
# ⚠️ Conventions tenues : apostrophe ASCII (187 contre 31 au catalogue), aucun ß en de-CH,
# « projet analytique » = `Projekt`/`progetto`/`project` (on ne traduit pas « analytique »),
# et l'idiome de hiérarchie suit `accounts-parent-archived` — `übergeordnet`, jamais *Eltern-*.
closing = Clôture…
email-templates-badge-custom = Personnalisé
email-templates-badge-default = Défaut
email-templates-body-label = Corps du message
email-templates-conflict = Conflit de version — le modèle a été rechargé
email-templates-load-error = Erreur de chargement
email-templates-save-error = Erreur lors de l'enregistrement
email-templates-restore-error = Erreur lors de la restauration
email-templates-lang-tablist = Langue
email-templates-restore = Restaurer le défaut
email-templates-restore-confirm-action = Restaurer le défaut
email-templates-restore-confirm-body = Votre texte personnalisé pour cette combinaison sera supprimé et remplacé par le modèle par défaut. Cette action est irréversible.
email-templates-restore-confirm-title = Restaurer le modèle par défaut ?
email-templates-restored = Modèle par défaut restauré
email-templates-restoring = Restauration…
email-templates-saved = Modèle enregistré
email-templates-subject-label = Objet
email-templates-subtitle = Personnalisez le contenu des e-mails envoyés à vos clients, par langue. Si vous ne modifiez rien, un modèle par défaut est utilisé automatiquement.
email-templates-title = Modèles d'e-mail
email-templates-unknown-variables = Le modèle contient des variables inconnues
email-templates-unknown-variables-list = Variables inconnues :
email-templates-variables-hint = Insérez ces variables dans l'objet ou le corps ; elles seront remplacées à l'envoi.
email-templates-variables-title = Variables disponibles
nav-credit-notes = Avoirs
nav-email-templates = Modèles d'e-mail
nav-projects = Projets analytiques
nav-supplier-invoices-import = Documents à traiter
onboarding-address-npa-city-required = NPA et localité sont obligatoires
onboarding-already-finalized = L'onboarding est déjà finalisé. Veuillez recharger la page.
onboarding-field-name-hint = entreprise — sinon remplir prénom + nom
onboarding-finalize-incomplete = La finalisation n'a pas été confirmée. Veuillez recharger la page.
onboarding-mode-error = Erreur lors du choix de mode
onboarding-demo-load-error = Erreur lors du chargement des données de démo
onboarding-setup-start-error = Erreur lors du démarrage de la configuration
onboarding-org-type-error = Erreur lors du choix du type
onboarding-accounting-lang-error = Erreur lors du choix de la langue comptable
onboarding-name-required = Nom (ou prénom + nom) obligatoire
onboarding-address-save-error = Erreur lors de la sauvegarde des coordonnées
onboarding-finalize-error = Finalisation échouée — réessayez.
onboarding-bank-required = Nom de banque et IBAN sont obligatoires
onboarding-bank-save-error = Erreur lors de la sauvegarde du compte bancaire
onboarding-bank-skip-error = Erreur lors du saut du compte bancaire
onboarding-save = Enregistrer
payment-batches-cancel = Annuler le lot
payment-batches-cancel-confirm = Annuler ce lot ? Les factures redeviennent réglables.
payment-batches-col-date = Exécution
payment-batches-col-id = Lot
payment-batches-col-status = Statut
payment-batches-col-total = Total
payment-batches-confirm = Confirmer le lot
payment-batches-confirm-hint = Une fois le virement exécuté dans votre e-banking, confirmez pour comptabiliser les règlements.
payment-batches-confirm-title = Confirmer le règlement
payment-batches-confirmed-at = Confirmé le
payment-batches-created = Lot créé
payment-batches-detail-date = Date d'exécution
payment-batches-detail-title = Lot de paiement
payment-batches-download = Télécharger le fichier pain.001
payment-batches-empty = Aucun lot de paiement.
payment-batches-err-bank = Sélectionnez un compte bancaire source.
payment-batches-err-invoices = Sélectionnez au moins une facture.
payment-batches-field-bank = Compte bancaire source
payment-batches-field-date = Date d'exécution souhaitée
payment-batches-form-close = Fermer
payment-batches-generate = Créer le lot
payment-batches-item-invoice = Facture
payment-batches-line-amount = Montant
payment-batches-lot = Lot
payment-batches-msg-id = MsgId
payment-batches-new = Créer un lot de virements
payment-batches-no-open = Aucune facture fournisseur ouverte.
payment-batches-open = ouvrir et télécharger le fichier
payment-batches-payment-date = Date de règlement effective
payment-batches-rejected = Factures non incluses :
payment-batches-select-invoices = Factures à régler par virement
projects-archive = Archiver
projects-archive-error = Opération impossible.
projects-archived = Projet archivé.
projects-archived-tag = archivé
projects-created = Projet créé.
projects-edit = Modifier
projects-empty = Aucun projet. Créez votre premier projet pour commencer.
projects-err-code = Le code du projet est requis.
projects-err-name = Le nom du projet est requis.
projects-field-code = Code
projects-field-description = Description (optionnel)
projects-field-end = Fin
projects-field-name = Nom
projects-field-parent = Projet parent (optionnel)
projects-field-start = Début
projects-form-create = Nouveau projet
projects-form-edit = Modifier le projet
projects-load-error = Impossible de charger les projets.
projects-new = Nouveau projet
projects-parent-none = — Aucun (projet racine)
projects-save = Enregistrer
projects-save-error = Enregistrement impossible.
projects-show-archived = Afficher les projets archivés
projects-subtitle = Regroupez vos dépenses et revenus par projet (rénovation, investissement) pour les analyser isolément.
projects-title = Projets analytiques
projects-unarchive = Désarchiver
projects-unarchived = Projet désarchivé.
projects-updated = Projet mis à jour.
settings-email-templates-link = Personnalisez le contenu des e-mails envoyés à vos clients (objet et corps, par langue).
# --- Grand livre (Story 24-1) ---
reports-ledger-title = Grand livre
reports-ledger-tab = Grand livre
reports-ledger-description = L'extrait d'un compte : ce qui a fait son solde, ligne à ligne.
reports-ledger-account-label = Compte
reports-ledger-all-accounts = Tous les comptes mouvementés
reports-ledger-from = Du
reports-ledger-to = Au
reports-ledger-opening = Solde d'ouverture
reports-ledger-closing = Solde de clôture
reports-ledger-movements-total = Total des mouvements
reports-ledger-column-piece = Pièce
reports-ledger-column-journal = Journal
reports-ledger-column-counterpart = Contrepartie
reports-ledger-column-running = Solde progressif
reports-ledger-column-lettering = Lettrage
reports-ledger-open-items-link = Postes ouverts de ce compte
reports-ledger-archived = archivé
reports-ledger-unnatural = Solde contre nature
reports-ledger-unnatural-hint = Ce compte présente un solde du côté opposé à sa nature. À vérifier.
reports-ledger-fy-break = Clôture de l'exercice — le solde repart de zéro
reports-ledger-no-movement = Aucun mouvement sur la période. Le solde d'ouverture reste dû.
reports-ledger-no-movement-closed = Aucun mouvement sur la période. L'exercice de l'ouverture se clôt dans la période : le solde repart de zéro.
reports-ledger-empty = Aucun compte à afficher sur cette période.
reports-ledger-open-from-balance = Voir le détail dans le grand livre
reports-ledger-truncated = Seules les { $shown } premières lignes sur { $total } sont affichées. L'export les contient toutes.
reports-ledger-include-zero = Inclure les comptes sans mouvement
reports-ledger-instruction = Choisissez une période, puis cliquez sur Générer. Le compte est facultatif.

# --- Journal d'audit — route, vocabulaire et export (Story 25-1c-a) ---
# ----------------------------------------------------------------------
# 33 types d'entité, 133 actions, 2 types d'auteur (recomptés à la Story 32-10a-iii :
# 32 et 127 avant elle, + 1 type et + 6 actions des prompts du serveur MCP).
#
audit-log-entity-account = Compte
audit-log-entity-api-key = Clé API
audit-log-entity-bank-account = Compte bancaire
audit-log-entity-bank-imports = Import bancaire
audit-log-entity-bank-profiles = Profil bancaire
audit-log-entity-bank-transaction = Transaction bancaire
audit-log-entity-company = Société
audit-log-entity-company-dunning-settings = Réglages de recouvrement
audit-log-entity-company-invoice-settings = Réglages de facturation
audit-log-entity-contact = Contact
audit-log-entity-contact-person = Personne de contact
audit-log-entity-credit-note = Avoir
audit-log-entity-dunning-level = Niveau de rappel
audit-log-entity-email-template = Modèle d'e-mail
audit-log-entity-export = Export
audit-log-entity-fiscal-year = Exercice comptable
audit-log-entity-imported-supplier-invoice = Document fournisseur
audit-log-entity-installation = Installation
audit-log-entity-invoice = Facture
audit-log-entity-journal-entry = Écriture
audit-log-entity-lettering = Lettrage
audit-log-entity-mcp-key = Clé du serveur MCP
audit-log-entity-mcp-prompt = Prompt du serveur MCP
audit-log-entity-mcp-send = Envoi du serveur MCP
audit-log-entity-mcp-server-settings = Réglages du serveur MCP
audit-log-entity-payment-batch = Lot de paiement
audit-log-entity-product = Produit
audit-log-entity-project = Projet
audit-log-entity-reconciliation-rules = Règle d'affectation
audit-log-entity-report = Rapport
audit-log-entity-supplier-invoice = Facture fournisseur
audit-log-entity-user = Utilisateur
audit-log-entity-vat-rate = Taux de TVA
#
audit-log-action-journal-entry-created = Écriture créée
audit-log-action-journal-entry-updated = Écriture modifiée
audit-log-action-journal-entry-reversed = Écriture contre-passée
audit-log-action-journal-entry-deleted = Écriture supprimée
audit-log-action-lettering-created = Lettrage posé
audit-log-action-lettering-removed = Lettrage retiré
audit-log-action-account-chart-loaded = Plan comptable chargé
audit-log-action-account-created = Compte créé
audit-log-action-account-updated = Compte modifié
audit-log-action-account-archived = Compte archivé
audit-log-action-account-reactivated = Compte réactivé
audit-log-action-account-retyped = Type de compte modifié
audit-log-action-fiscal-year-created = Exercice comptable créé
audit-log-action-fiscal-year-updated = Exercice comptable modifié
audit-log-action-fiscal-year-closed = Exercice comptable clôturé
audit-log-action-fiscal-year-reopened = Exercice comptable rouvert
audit-log-action-books-locked = Période comptable verrouillée
audit-log-action-books-unlocked = Période comptable déverrouillée
audit-log-action-books-restored = Verrou de période rétabli après restauration
audit-log-action-vat-rate-created = Taux de TVA créé
audit-log-action-vat-rate-updated = Taux de TVA modifié
audit-log-action-vat-rate-deactivated = Taux de TVA désactivé
audit-log-action-invoice-created = Facture créée
audit-log-action-invoice-updated = Facture modifiée
audit-log-action-invoice-validated = Facture validée
audit-log-action-invoice-unvalidated = Facture dévalidée
audit-log-action-invoice-cancelled = Facture annulée
audit-log-action-invoice-deleted = Facture supprimée
audit-log-action-invoice-emailed = Facture envoyée par e-mail
audit-log-action-invoice-pdf-frozen = PDF de facture figé
audit-log-action-invoice-pdf-refrozen = PDF de facture refigé
audit-log-action-invoice-paid = Facture payée
audit-log-action-invoice-partially-settled = Facture partiellement réglée
audit-log-action-invoice-written-off = Reste de facture soldé
audit-log-action-invoice-settlement-cancelled = Règlement annulé
audit-log-action-invoice-reminder-sent = Rappel envoyé
audit-log-action-invoice-reminder-cancelled = Rappel annulé
audit-log-action-invoice-dunning-paused = Rappels suspendus
audit-log-action-invoice-dunning-resumed = Rappels repris
audit-log-action-credit-note-created = Avoir créé
audit-log-action-dunning-level-created = Niveau de rappel créé
audit-log-action-dunning-level-updated = Niveau de rappel modifié
audit-log-action-dunning-level-deleted = Niveau de rappel supprimé
audit-log-action-company-dunning-settings-updated = Réglages de recouvrement modifiés
audit-log-action-company-invoice-settings-created = Réglages de facturation créés
audit-log-action-company-invoice-settings-updated = Réglages de facturation modifiés
audit-log-action-supplier-invoice-created = Facture fournisseur créée
audit-log-action-supplier-invoice-cancelled = Facture fournisseur annulée
audit-log-action-supplier-invoice-paid = Facture fournisseur payée
audit-log-action-supplier-invoice-settlement-cancelled = Règlement fournisseur annulé
audit-log-action-imported-supplier-invoice-created = Document importé
audit-log-action-imported-supplier-invoice-completed = Document traité (facture créée)
audit-log-action-imported-supplier-invoice-discarded = Document retiré de la liste
audit-log-action-imported-supplier-invoice-reactivated = Document remis dans la liste
audit-log-action-imported-supplier-invoice-attached = Document joint à une facture
audit-log-action-imported-supplier-invoice-detached = Document détaché de sa facture
audit-log-action-payment-batch-generated = Lot de paiement créé
audit-log-action-payment-batch-confirmed = Lot de paiement confirmé
audit-log-action-payment-batch-cancelled = Lot de paiement annulé
audit-log-action-bank-account-created = Compte bancaire créé
audit-log-action-bank-account-updated = Compte bancaire modifié
audit-log-action-bank-account-archived = Compte bancaire archivé
audit-log-action-bank-account-closed = Compte bancaire clos
audit-log-action-bank-account-reopened = Compte bancaire rouvert
audit-log-action-bank-import-created = Import bancaire effectué
audit-log-action-bank-import-deleted = Import bancaire supprimé
audit-log-action-bank-profile-created = Profil bancaire créé
audit-log-action-bank-profile-updated = Profil bancaire modifié
audit-log-action-bank-profile-deleted = Profil bancaire supprimé
audit-log-action-reconciliation-accepted = Réconciliation acceptée
audit-log-action-reconciliation-cancelled = Rapprochement annulé
audit-log-action-reconciliation-linked = Transaction rattachée à un règlement
audit-log-action-reconciliation-rejected = Réconciliation rejetée
audit-log-action-reconciliation-manual-matched = Réconciliation manuelle effectuée
audit-log-action-reconciliation-split-applied = Ventilation appliquée
audit-log-action-reconciliation-unrejected = Rejet de réconciliation annulé
audit-log-action-reconciliation-rule-created = Règle d'affectation créée
audit-log-action-reconciliation-rule-updated = Règle d'affectation modifiée
audit-log-action-reconciliation-rule-deleted = Règle d'affectation supprimée
audit-log-action-reconciliation-rule-applied = Règle d'affectation appliquée
audit-log-action-contact-created = Contact créé
audit-log-action-contact-updated = Contact modifié
audit-log-action-contact-archived = Contact archivé
audit-log-action-contact-person-created = Personne de contact créée
audit-log-action-contact-person-updated = Personne de contact modifiée
audit-log-action-contact-person-archived = Personne de contact archivée
audit-log-action-product-created = Produit créé
audit-log-action-product-updated = Produit modifié
audit-log-action-product-archived = Produit archivé
audit-log-action-project-created = Projet créé
audit-log-action-project-updated = Projet modifié
audit-log-action-project-archived = Projet archivé
audit-log-action-project-unarchived = Projet réactivé
audit-log-action-user-created = Utilisateur créé
audit-log-action-user-updated = Utilisateur modifié
audit-log-action-user-disabled = Utilisateur désactivé
audit-log-action-user-role-changed = Rôle modifié
audit-log-action-user-password-reset = Mot de passe réinitialisé
audit-log-action-api-key-created = Clé API créée
audit-log-action-api-key-revoked = Clé API révoquée
audit-log-action-mcp-key-created = Clé du serveur MCP créée
audit-log-action-mcp-key-revoked = Clé du serveur MCP révoquée
audit-log-action-mcp-key-send-validation-changed = Validation avant envoi d'une clé du serveur MCP modifiée
audit-log-action-mcp-prompt-created = Prompt du serveur MCP créé
audit-log-action-mcp-prompt-standard-restored = Prompt du serveur MCP rendu à son texte standard
audit-log-action-mcp-prompt-version-approved = Proposition de prompt du serveur MCP approuvée
audit-log-action-mcp-prompt-version-created = Version d'un prompt du serveur MCP enregistrée
audit-log-action-mcp-prompt-version-refused = Proposition de prompt du serveur MCP refusée
audit-log-action-mcp-prompt-served = Prompt du serveur MCP servi
audit-log-action-mcp-send-approved = Envoi du serveur MCP validé
audit-log-action-mcp-send-refused = Envoi du serveur MCP refusé
audit-log-action-mcp-server-settings-updated = Réglages du serveur MCP modifiés
audit-log-action-mcp-tool-called = Outil du serveur MCP appelé
audit-log-action-auth-password-reset-requested = Réinitialisation de mot de passe demandée
audit-log-action-auth-password-reset-completed = Réinitialisation de mot de passe effectuée
audit-log-action-admin-break-glass-reset = Réinitialisation d'urgence de l'administrateur
audit-log-action-company-created = Société créée
audit-log-action-company-updated = Société modifiée
audit-log-action-email-template-updated = Modèle d'e-mail modifié
audit-log-action-email-template-restored-default = Modèle d'e-mail réinitialisé
audit-log-action-installation-demo-seeded = Données de démonstration chargées
audit-log-action-installation-repaired = Réparation de l'installation
audit-log-action-installation-reset = Installation réinitialisée
audit-log-action-installation-step-completed = Étape d'installation franchie
audit-log-action-installation-ui-mode-changed = Mode d'utilisation changé
audit-log-action-admin-backup-downloaded = Sauvegarde du serveur téléchargée
audit-log-action-admin-backup-run = Sauvegarde lancée manuellement
audit-log-action-admin-backup-schedule-updated = Réglage de la sauvegarde automatique modifié
audit-log-action-admin-full-export = Sauvegarde complète exportée
audit-log-action-admin-full-import = Sauvegarde importée
audit-log-action-exports-global = Export global effectué
audit-log-action-report-generated = Rapport généré
audit-log-action-report-exported = Rapport exporté
#
audit-log-actor-type-user = Utilisateur
audit-log-actor-type-api-key = Clé API
#
audit-log-csv-header-id = N°
audit-log-csv-header-created-at = Date (UTC)
audit-log-csv-header-actor = Auteur
audit-log-csv-header-user-id = Identifiant d'auteur
audit-log-csv-header-actor-type = Type d'auteur
audit-log-csv-header-action = Action
audit-log-csv-header-entity-type = Type d'entité
audit-log-csv-header-entity-id = Identifiant d'entité
audit-log-csv-header-api-key-id = Clé API
audit-log-csv-header-details = Détails
audit-log-export-error-too-large = Trop de résultats (> { $limit }). Veuillez affiner vos filtres (par ex. plage de dates, action ou type d'entité) avant de relancer l'export.
# Story 31-5 (#469) — refus des paramètres de consultation du journal d'audit (serveur).
# Les noms de paramètre (dateFrom, entityType…) sont des noms d'API : ils ne se traduisent pas.
audit-log-query-error-date-range-inverted = dateFrom doit être inférieur ou égal à dateTo
audit-log-query-error-date-invalid = { $field } invalide : date attendue au format AAAA-MM-JJ
audit-log-query-error-date-out-of-range = { $field } doit être compris entre 1000-01-01 et 9999-12-31
audit-log-query-error-entity-id-not-positive = entityId doit être un identifiant positif
audit-log-query-error-entity-id-without-type = entityId exige entityType — un identifiant seul ne désigne rien
audit-log-query-error-code-too-long = { $field } dépasse { $max } caractères

# --- Journal d'audit — écran (Story 25-1c-b1) ---
nav-audit-log = Journal d'audit
audit-log-title = Journal d'audit
audit-log-subtitle = La trace de chaque opération qui modifie les livres de votre société : qui, quoi, quand.
audit-log-filter-date-from = Du (jour UTC)
audit-log-filter-date-to = Au (jour UTC)
audit-log-filter-entity-type = Type d'entité
audit-log-filter-entity-type-all = Tous
audit-log-filter-entity-id = N° d'entité
audit-log-filter-action = Action
audit-log-filter-action-all = Toutes
audit-log-filter-reset = Réinitialiser
audit-log-export = Exporter en CSV
audit-log-col-date = Date
audit-log-col-actor = Auteur
audit-log-col-action = Action
audit-log-col-entity-type = Type d'entité
audit-log-col-entity-id = N°
audit-log-col-details = Détails
audit-log-api-key = clé API
audit-log-details-show = Afficher
audit-log-details-hide = Masquer
audit-log-empty = Aucune entrée ne correspond à ces filtres.
audit-log-error = Le journal d'audit n'a pas pu être chargé.
audit-log-error-date-range = La date de début doit précéder ou égaler la date de fin.
audit-log-prev = Précédent
audit-log-next = Suivant
audit-log-range = { $from }–{ $to } sur { $total }

# --- Story 25-2-b-2 (#440) : dévalidation depuis l'écran, et le trou assumé ---
invoice-unvalidate-button = Dévalider
invoice-unvalidate-confirm-title = Dévalider la facture ?
invoice-unvalidate-confirm-body = Son écriture comptable sera supprimée et la facture repassera en brouillon. Son numéro est conservé : en la revalidant, elle reprendra le même.
invoice-unvalidate-keeps-number = Numéro conservé :
invoice-unvalidated-success = Facture repassée en brouillon
invoice-unvalidate-error = Erreur lors de la dévalidation
invoice-delete-numbered-warning = Cette facture porte déjà le numéro { $number } : le supprimer laissera un trou définitif dans la séquence, le compteur ne redescendant pas.
invoice-delete-confirm-body-context = Confirmer la suppression de la facture du { $date } pour { $contact } ?
invoice-validate-success-no-number = Facture validée

# --- Story 15-1c-i (#518) : l'écran des postes ouverts — consulter, lettrer, délettrer ---
open-items-group-title = Groupe { $code }
open-items-group-close = Fermer
open-items-group-not-found = Aucun groupe ne porte ce code.
open-items-col-date = Date
open-items-col-entry = Écriture
open-items-col-journal = Journal
open-items-col-description = Libellé
open-items-col-document = Pièce
open-items-col-debit = Débit
open-items-col-credit = Crédit
open-items-group-dissolve = Délettrer
open-items-proposals-error = Les rapprochements proposés n'ont pas pu être chargés.
open-items-group-error = Le groupe n'a pas pu être chargé.
open-items-action-error = L'opération a échoué.
open-items-line-gone = Une ligne sélectionnée n'existe plus : son écriture a été modifiée ou supprimée.
open-items-title = Postes ouverts
open-items-boundary = Cet écran solde entre elles les lignes des comptes de tiers et de passage. Le rapprochement des relevés bancaires se fait dans Mensuel → Réconciliation ; les comptes bancaires n'apparaissent pas ici.
open-items-account-label = Compte
open-items-account-placeholder = Choisissez un compte
open-items-account-archived = archivé
open-items-as-of-label = Date
open-items-code-label = Code
open-items-code-open = Ouvrir le groupe
open-items-lettered = Lettrage posé, code :
open-items-group-dissolved = Le groupe { $code } est délettré.
open-items-group-show-list = Afficher les postes ouverts de ce compte
open-items-choose-account = Choisissez un compte pour voir ses postes ouverts.
open-items-account-not-found = Compte introuvable.
open-items-accounts-error = La liste des comptes n'a pas pu être chargée.
open-items-no-account = Aucun compte de ce plan comptable ne se lettre.
open-items-load-error = Les postes ouverts n'ont pas pu être chargés.
open-items-selection-count = Lignes sélectionnées : { $count } (dont { $offPage } hors de cette page)
open-items-selection-sum = Somme de la sélection : { $amount }
open-items-letter = Lettrer
open-items-selection-clear = Vider la sélection
open-items-state-today-note = L'état de la pièce (seconde ligne du motif) est celui d'aujourd'hui ; la première ligne dit le motif à la date choisie.
open-items-col-select = Sélection
open-items-reason-help = Pourquoi la ligne est ouverte à la date choisie, puis où en est sa pièce aujourd'hui.
open-items-col-reason = Motif
open-items-select-line = Sélectionner la ligne { $entry }
open-items-empty = Aucun poste ouvert à cette date.
open-items-range = { $from }–{ $to } sur { $total }
open-items-open-total = Total des postes ouverts au { $date }
open-items-balance = Solde du compte au { $date }
open-items-balance-equal = Le total des postes ouverts au { $date } égale le solde du compte au { $date } — celui que la Balance montre pour ce compte à cette date, du côté naturel du compte (solde cumulé depuis l'ouverture des livres).
open-items-proposals-title = Rapprochements proposés
open-items-proposals-today = Calculés sur les lignes ouvertes aujourd'hui, quelle que soit la date choisie.
open-items-proposals-empty = Aucun rapprochement à proposer.
open-items-proposal-days = Écart de dates (jours) : { $days }
open-items-proposal-reversal = contre-passation
open-items-proposals-more = Rapprochements non affichés : { $count } — ils apparaîtront quand ceux-ci seront lettrés.
open-items-entry-ref = { $fiscalYear } n° { $number }
open-items-reason-unlettered = non lettrée
open-items-reason-lettered-after = lettrée après cette date
open-items-reason-lettered-on = le { $date }
open-items-state-unpaid = facture non réglée — reste dû : { $amount }
open-items-state-partially-settled = facture partiellement réglée — reste dû : { $amount }
open-items-state-paid-without-settlement = marquée payée avant la v0.12.0, sans écriture d'encaissement : le compte porte encore cette créance
open-items-state-nothing-due = pièce soldée, lettrage non posé : période close ou avoir hérité — rien à faire ici
open-items-no-checkbox-lettered = déjà lettrée (code { $code })
open-items-no-checkbox-document = son lettrage suit sa pièce et ses règlements
open-items-letter-too-few = Sélectionnez au moins deux lignes.
open-items-letter-too-many = { $max } lignes au plus.
open-items-letter-unbalanced = La sélection ne s'équilibre pas : écart { $difference }.
open-items-letter-all-closed = Toutes ces lignes sont dans une période close : le lettrage n'y change plus.
open-items-document-supplier-invoice = facture fournisseur
open-items-document-settlement = règlement de { $number }
open-items-document-bank-transaction = transaction bancaire
open-items-side-debit = débiteur
open-items-side-credit = créditeur
open-items-origin-manual = lettrage manuel
open-items-origin-reversal = contre-passation
open-items-origin-document-unknown = lettrage d'une pièce
open-items-origin-document = lettrage de la pièce { $number }

# --- Story 32-7 (refs #723) : l'écran « Serveur MCP » (Administration) ---
# Les noms propres (Traefik, Docker, 100.64.0.0/10, X-Forwarded-For, .mcp.json) et les codes de portée ne se traduisent pas.
nav-admin-mcp = Serveur MCP
mcp-server-address-label = Adresse du serveur :
mcp-server-address-note = exactement /mcp, sans barre finale. C’est l’adresse que voit ce navigateur : derrière un mandataire, un agent du réseau local peut devoir en employer une autre.
mcp-server-cancel = Annuler
mcp-server-exempt-confirm = Passer en dispensée
mcp-server-exempt-confirm-text = Les données partiront à l’agent sans vous être montrées ; chaque appel reste journalisé.
mcp-server-form-create = Créer la clé
mcp-server-form-expires = Expiration (facultative)
mcp-server-form-expires-help = La clé cesse d’être valide à la fin de ce jour, à l’heure de ce poste.
mcp-server-form-name = Nom
mcp-server-form-name-required = Le nom est requis.
mcp-server-form-name-too-long = Le nom compte 255 caractères au plus.
mcp-server-form-policy = Validation avant envoi
mcp-server-form-scope-required = Choisissez au moins une portée.
mcp-server-form-scopes = Portées
mcp-server-form-scopes-intro = Aucune portée n’écrit dans les livres ni n’administre Kesh.
mcp-server-form-title = Nouvelle clé MCP
mcp-server-guarantee-firewall = Il ne remplace ni le pare-feu ni le VPN : n’activez pas le serveur MCP sur une installation exposée à Internet, /mcp est servi par le même port que l’application.
mcp-server-guarantee-proxy = Le filtre juge l’adresse que voit Kesh. Derrière un mandataire non déclaré parmi les mandataires de confiance — Traefik, Nginx, le proxy inversé du NAS —, tous les clients paraissent venir de lui ; derrière une publication de port Docker par le mandataire en espace utilisateur, tous peuvent paraître venir de la passerelle Docker. Ce sont des adresses privées, admises par défaut : le filtre n’arrête alors personne.
mcp-server-guarantee-rate = Aucune limite de débit n’est posée sur /mcp.
mcp-server-guarantee-tailscale = La plage 100.64.0.0/10 (VPN de type Tailscale) n’est pas admise par défaut : ajoutez-la vous-même aux plages admises.
mcp-server-guarantee-title = Ce que le filtre ne garantit pas
mcp-server-intro = Le serveur MCP permet à un agent — un client MCP comme Claude Code — de lire les livres de cette société avec une clé MCP. Il est désactivé par défaut ; aucune portée n’écrit dans les livres.
mcp-server-key-revoked = Clé révoquée.
mcp-server-key-stale = La clé a changé ou n’existe plus ; liste rechargée.
mcp-server-keys-col-actions = Actions
mcp-server-keys-col-created = Créée le
mcp-server-keys-col-expires = Expire le
mcp-server-keys-col-last-used = Dernière utilisation
mcp-server-keys-col-name = Nom
mcp-server-keys-col-policy = Validation
mcp-server-keys-col-scopes = Portées
mcp-server-keys-col-status = Statut
mcp-server-keys-empty = Aucune clé MCP.
mcp-server-keys-last-used-note = La dernière utilisation est posée chaque fois que la clé est reconnue, même si l’appel est ensuite refusé ou n’est pas servi.
mcp-server-keys-loading = Chargement des clés…
mcp-server-keys-never = jamais
mcp-server-keys-new = Nouvelle clé
mcp-server-keys-no-access = n’ouvre aucun accès
mcp-server-keys-revoke = Révoquer
mcp-server-keys-scope-unknown = inconnue de cette version — jamais accordée
mcp-server-keys-status-active = Active
mcp-server-keys-status-expired = Expirée le { $date }
mcp-server-keys-status-revoked = Révoquée le { $date }
mcp-server-keys-title = Clés MCP
mcp-server-keys-to-exempt = Passer en dispensée
mcp-server-keys-to-required = Passer en requise
mcp-server-nav-prompts = Prompts du serveur MCP
mcp-server-nav-sends = Envois du serveur MCP
mcp-server-policy-changed = Politique de validation modifiée.
mcp-server-policy-exempt = Dispensée
mcp-server-policy-exempt-help = Les données partent sans validation ; chaque appel reste journalisé.
mcp-server-policy-required = Requise
mcp-server-policy-required-help = Chaque réponse d’outil qui porte des données attend votre validation dans Envois du serveur MCP avant de partir ; un envoi non décidé en 24 heures échoit, et une clé compte au plus 20 envois en attente. Une réponse sans données — introuvable, trop volumineuse, erreur — part sans validation.
mcp-server-reset-note = La réinitialisation des données de démonstration efface ces réglages (serveur désactivé) et conserve les clés.
mcp-server-restore-note = Une restauration rend ces réglages tels qu’ils étaient dans la sauvegarde : un serveur actif dans la sauvegarde l’est après l’import.
mcp-server-revoke-confirm = Révoquer définitivement
mcp-server-revoke-confirm-text = L’agent qui emploie cette clé perd l’accès immédiatement. La révocation est définitive ; seule la restauration d’une sauvegarde antérieure rendrait la clé de nouveau active.
mcp-server-scope-books = Le plan comptable, les exercices, les écritures, le grand livre, la balance, le bilan, le compte de résultat et les postes ouverts.
mcp-server-scope-contacts = Le carnet d’adresses — des données personnelles. Elle démasque aussi, dans les factures, l’IBAN et le QR-IBAN du créancier et l’adresse courriel d’envoi ; les noms des tiers se lisent sans elle. L’IBAN n’est pas une donnée du carnet d’adresses.
mcp-server-scope-documents = Le nom du fichier, le type, la taille, l’état et la facture fournisseur liée de chaque pièce importée, sans son contenu.
mcp-server-scope-invoices = Les factures clientes et fournisseur, l’échéancier et la balance âgée ; les noms des tiers sont lisibles, mais l’IBAN et le QR-IBAN du créancier et l’adresse courriel d’envoi d’une facture sont masqués sans contacts:read.
mcp-server-scope-prompts = Proposer une version d’un prompt, qui attend l’approbation d’un administrateur ; elle ne permet ni d’approuver ni de relire une proposition ; une clé à validation requise peut proposer.
mcp-server-secret-client = Donnez-la au client MCP en portée locale ou utilisateur — jamais dans un .mcp.json versionné (manuel d’administration, § Configurer un client).
mcp-server-secret-close = Fermer
mcp-server-secret-copied = Clé copiée dans le presse-papiers.
mcp-server-secret-copy = Copier
mcp-server-secret-copy-failed = La copie a échoué : sélectionnez la clé et copiez-la à la main.
mcp-server-secret-copy-now = Copiez-la maintenant : elle ne sera plus jamais affichée.
mcp-server-secret-created = Clé « { $name } » créée.
mcp-server-sends-link = Voir les envois du serveur MCP
mcp-server-settings-canonical = Kesh enregistre chaque entrée sous sa forme canonique.
mcp-server-settings-conflict = Les réglages ont changé entre-temps ; ils sont rechargés, vos modifications ne sont pas enregistrées.
mcp-server-settings-enabled = Serveur activé
mcp-server-settings-filter = Filtre d’adresse activé
mcp-server-settings-load-failed = Les réglages n’ont pas pu être lus : tant qu’ils ne le sont pas, rien ne s’enregistre depuis cet écran. Si l’erreur persiste, voir le manuel d’administration, § Activer le serveur.
mcp-server-settings-loading = Chargement des réglages…
mcp-server-settings-networks = Plages admises
mcp-server-settings-never = Jamais modifiés — valeurs par défaut.
mcp-server-settings-one-per-line = Une entrée par ligne.
mcp-server-settings-origins = Origines admises
mcp-server-settings-origins-help = Une origine admise n’ouvre le serveur à aucune page web : elle ne sert qu’aux clients qui envoient un en-tête Origin sans être des navigateurs.
mcp-server-settings-proxies = Mandataires de confiance
mcp-server-settings-save = Enregistrer
mcp-server-settings-saved = Réglages enregistrés.
mcp-server-settings-title = Réglages du serveur
mcp-server-settings-updated = Dernière modification : { $date }
mcp-server-state-off = Désactivé : /mcp répond comme une adresse qui n’existe pas.
mcp-server-state-on = Activé : le serveur répond aux clés MCP de cette société, depuis les adresses admises.
mcp-server-title = Serveur MCP
mcp-server-warning-filter-off = Filtre désactivé : toute adresse est admise.
mcp-server-warning-network-any-v4 = { $entry } admet tout client IPv4 ; pour admettre les deux familles, il faut les deux entrées.
mcp-server-warning-network-any-v6 = { $entry } admet tout client IPv6 ; pour admettre les deux familles, il faut les deux entrées.
mcp-server-warning-nobody = Filtre activé et aucune plage admise : personne n’est admis.
mcp-server-warning-proxy-any-v4 = { $entry } : tout pair IPv4 est cru sur son en-tête X-Forwarded-For.
mcp-server-warning-proxy-any-v6 = { $entry } : tout pair IPv6 est cru sur son en-tête X-Forwarded-For.
mcp-server-form-expires-invalid = Date d’expiration invalide : choisissez un jour à venir, jusqu’au 30.12.9999.
mcp-server-settings-retry = Réessayer
