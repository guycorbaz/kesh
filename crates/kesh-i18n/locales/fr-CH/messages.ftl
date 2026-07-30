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
error-cannot-disable-self = Impossible de désactiver son propre compte
error-cannot-disable-last-admin = Impossible de désactiver le dernier administrateur

# Erreurs de ressource
error-not-found = Ressource introuvable
error-conflict = Ressource déjà existante
error-optimistic-lock = Conflit de version — la ressource a été modifiée
error-foreign-key = Référence invalide
error-journal-entry-linked-to-invoice = Cette écriture comptable a été générée par une facture validée et ne peut pas être supprimée directement. Annulez d'abord la facture concernée.
error-check-constraint = Valeur invalide
error-illegal-state = Transition d'état interdite

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

# Bannière démo
demo-banner-text = Instance de démonstration — données fictives
demo-banner-reset = Réinitialiser pour la production
demo-reset-confirm-title = Réinitialiser l'instance
demo-reset-confirm-body = Toutes les données de démonstration seront supprimées. Voulez-vous continuer ?
demo-reset-confirm-ok = Confirmer
demo-reset-confirm-cancel = Annuler

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
homepage-invoices-empty-guided = Aucune facture ouverte. Créez votre première facture pour facturer vos clients.
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
error-fiscal-year-closed-generic = L'exercice comptable est clôturé — aucune écriture ne peut y être ajoutée ou modifiée (CO art. 957-964).
error-inactive-accounts = Un ou plusieurs comptes sont archivés ou invalides.

# Compte de produit par ligne de facture (Story 16-1a, #152)
invoice-line-account-subject-line = Ligne { $line }
invoice-line-account-subject-default = le compte de produit par défaut de la société
invoice-line-account-unknown = { $subject } : le compte sélectionné est introuvable ou n'appartient pas à cette société
invoice-line-account-inactive = { $subject } : le compte { $number } est archivé
invoice-line-account-not-revenue = { $subject } : le compte { $number } n'est pas un compte de produit
invoice-line-account-not-postable = { $subject } : le compte { $number } n'est pas imputable — choisissez un autre compte
invoice-line-revenue-account-invalid = Compte de produit invalide — { $detail }
credit-note-revenue-account-archived = Impossible d'émettre l'avoir — { $detail }. Réactivez le ou les comptes concernés.
invoice-error-total-zero = Cette facture est d'un montant total nul : elle ne peut pas être validée. Renseignez au moins une ligne avec un prix unitaire supérieur à zéro.
credit-note-error-total-zero = Cette facture est d'un montant total nul : aucun avoir ne peut être émis.

# Modification & suppression d'écritures (Story 3.3)
journal-entry-edit = Modifier
journal-entry-delete = Supprimer
journal-entry-delete-confirm-title = Supprimer l'écriture N°{ $number } ?
journal-entry-delete-confirm-message = Cette action est irréversible. L'action sera enregistrée dans le journal d'audit.
journal-entry-delete-confirm-cancel = Annuler
journal-entry-delete-confirm-delete = Supprimer
journal-entry-deleted = Écriture supprimée
journal-entry-conflict-title = Conflit de version
journal-entry-conflict-message = Cette écriture a été modifiée par un autre utilisateur. Voulez-vous recharger ?
journal-entry-conflict-reload = Recharger
journal-entry-conflict-reloaded = Liste rechargée — cliquez à nouveau sur modifier pour reprendre

# Assistant TVA achat (Story 18-1c)
vat-purchase-title = Assistant TVA achat
vat-purchase-config-required = Configurez le compte d'impôt préalable dans Paramètres → Facturation pour utiliser l'assistant.
vat-purchase-no-rates = Aucun taux TVA configuré — voir Paramètres → Taux TVA.
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
journal-entries-filter-reset = Réinitialiser
journal-entries-pagination-on = sur
journal-entries-pagination-prev = Précédent
journal-entries-pagination-next = Suivant
journal-entries-pagination-page-size = Par page
journal-entries-sort-asc-indicator = tri ascendant
journal-entries-sort-desc-indicator = tri descendant
journal-entries-loading = Chargement…

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
contact-filter-search-placeholder = Rechercher par nom ou email…
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
invoice-error-configuration-required = Configurez les comptes par défaut dans Paramètres > Facturation avant de valider une facture.
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
settings-invoicing-revenue-account = Compte produit (Revenue)
settings-invoicing-journal = Journal
settings-invoicing-description-template = Libellé de l'écriture comptable
settings-invoicing-save = Enregistrer
settings-invoicing-save-success = Configuration enregistrée
settings-invoicing-format-invalid = Format invalide
settings-invoicing-format-too-long = Le format est trop long
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
invoice-pdf-recipient = Destinataire
invoice-pdf-description = Description
invoice-pdf-quantity = Qté
invoice-pdf-unit-price = Prix unitaire
invoice-pdf-vat = TVA
invoice-pdf-line-total = Total
invoice-pdf-subtotal = Sous-total
invoice-pdf-total = Total
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
due-dates-column-payment-status = Statut
due-dates-column-paid-at = Payée le
due-dates-export-button = Exporter CSV
due-dates-no-results = Aucune facture à afficher.
due-dates-result-suffix = résultat(s)

# Statuts paiement
payment-status-paid = Payée
payment-status-unpaid = Impayée
payment-status-overdue = En retard

# Marquer payée / Dé-marquer payée
invoice-mark-paid-button = Marquer payée
invoice-mark-paid-dialog-title = Marquer la facture comme payée
invoice-mark-paid-dialog-body = Indiquez la date à laquelle vous avez reçu le paiement.
invoice-mark-paid-date-label = Date de paiement
invoice-mark-paid-confirm = Confirmer le paiement
invoice-mark-paid-success = Facture marquée payée
invoice-unmark-paid-button = Dé-marquer payée
invoice-unmark-paid-dialog-title = Dé-marquer payée
invoice-unmark-paid-dialog-body = Cette facture sera à nouveau considérée comme impayée. Utile pour corriger une erreur. Continuer ?
invoice-unmark-paid-confirm = Dé-marquer
invoice-unmark-paid-success = Marquage paiement annulé
invoice-detail-paid-at-label = Payée le

# Erreurs validation paidAt
invoice-error-paid-at-required = Date de paiement obligatoire
invoice-error-paid-at-before-invoice-date = La date de paiement ne peut être antérieure à la date de facture
invoice-error-mark-paid-not-validated = Seules les factures validées peuvent être marquées payées
invoice-error-already-unpaid = Cette facture n'est pas marquée payée

# Export CSV — en-têtes (locale = companies.accounting_language)
echeancier-csv-header-number = Numéro
echeancier-csv-header-date = Date
echeancier-csv-header-due-date = Date d'échéance
echeancier-csv-header-contact = Client
echeancier-csv-header-total = Total
echeancier-csv-header-payment-status = Statut paiement
echeancier-csv-header-paid-at = Date paiement
echeancier-export-error-too-large = Trop de résultats (> { $limit }). Veuillez affiner vos filtres (par ex. plage de dates ou statut de paiement) avant de relancer l'export.
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
common-error = Erreur inattendue

invoice-pdf-error-not-found = Facture introuvable.
invoice-pdf-error-generic = Erreur lors du téléchargement du PDF.
invoice-pdf-error-empty = Le PDF reçu est vide.

# Story 2.6 — Onboarding: Invoice Settings Pre-fill
config-incomplete-title = Configuration incomplète
config-incomplete-link = Configurez les comptes de facturation
invoice-settings-required = Configurez d'abord les comptes de facturation dans les paramètres

# === Story 3.7 — Gestion des exercices comptables (FR-CH) ===

fiscal-year-title = Exercices comptables
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
fiscal-year-reopen-confirmation-body = Vous êtes sur le point de rouvrir l’exercice « { $name } ». Il redeviendra modifiable (saisie d’écritures) jusqu’à une nouvelle clôture. Un motif est obligatoire et sera conservé dans la piste d’audit.
fiscal-year-reopen-motif-label = Motif de la réouverture
fiscal-year-reopen-confirmation-action = Rouvrir l’exercice
fiscal-year-reopened = Exercice rouvert.
fiscal-year-reopen-blocked-later-closed = Rouvrez d’abord l’exercice « { $name } », plus récent et encore clôturé.
error-fiscal-year-reopen-motif-empty = Le motif de réouverture est obligatoire.
error-fiscal-year-reopen-motif-too-long = Le motif de réouverture est trop long (500 caractères maximum).
error-fiscal-year-already-open = Cet exercice est déjà ouvert.
error-fiscal-year-reopen-blocked = Réouverture impossible : un exercice postérieur est clôturé ; rouvrez-le d’abord.
error-fiscal-year-overlap = Cet exercice chevauche un exercice existant.
error-fiscal-year-name-duplicate = Un exercice avec ce nom existe déjà.
error-fiscal-year-name-empty = Le nom de l’exercice est obligatoire.
error-fiscal-year-name-too-long = Le nom de l’exercice est trop long (50 caractères maximum).
error-fiscal-year-dates-invalid = Dates invalides — la date de fin doit être strictement postérieure à la date de début.
error-fiscal-year-already-closed = Cet exercice est déjà clôturé.
error-fiscal-year-conflict = Conflit d’exercice (nom ou date de début déjà utilisé).
error-fiscal-year-missing = Créez d’abord un exercice comptable dans Paramètres → Exercices.
error-fiscal-year-closed-for-date = L’exercice qui couvre cette date est clôturé. Vérifiez la date saisie ou consultez vos exercices.
go-to-settings = Ouvrir Paramètres
settings-fiscal-years-link = Créez, renommez ou clôturez les exercices comptables de votre entreprise.

# Story 14-4 — bilan d'ouverture (soldes de départ, reprise de comptabilité)
opening-balances-title = Soldes de départ
opening-balances-intro = Saisissez les soldes de vos comptes de bilan repris de votre ancienne comptabilité. Une écriture d’ouverture équilibrée sera générée au { $date } (premier jour de l’exercice « { $name } »). Posez votre report à-nouveau accumulé sur votre compte de report pour équilibrer l’écriture.
opening-balances-account = Compte
opening-balances-debit = Débit
opening-balances-credit = Crédit
opening-balances-total-debit = Total débits
opening-balances-total-credit = Total crédits
opening-balances-diff = Différence
opening-balances-generate = Générer l’écriture d’ouverture
opening-balances-generating = Génération…
opening-balances-success = Écriture d’ouverture générée.
opening-balances-entry-description = Bilan d’ouverture — soldes de départ
opening-balances-locked-no-fiscal-year = Aucun exercice comptable : créez d’abord un exercice (Paramètres → Exercices) pour saisir vos soldes de départ.
opening-balances-locked-first-year-closed = Le premier exercice « { $name } » est clôturé : un administrateur doit le rouvrir avant la saisie des soldes de départ.
opening-balances-locked-already-has-entries = La société contient déjà des écritures : le bilan d’ouverture est verrouillé. Corrigez l’écriture d’ouverture directement dans le journal, ou supprimez toutes les écritures pour recommencer.
opening-balances-goto-journal = Ouvrir le journal
opening-balances-goto-balance-sheet = Voir le bilan
opening-balances-status-error = Impossible de charger l’état des soldes de départ.
opening-balances-retry = Réessayer
opening-balances-empty-grid = Aucun compte de bilan actif et postable dans votre plan comptable — créez ou réactivez vos comptes d’actifs et de passifs (Plan comptable) avant de saisir les soldes de départ.
error-opening-balances-no-fiscal-year = Aucun exercice comptable : créez d’abord un exercice avant de saisir les soldes de départ.
error-opening-balances-first-year-closed = Le premier exercice est clôturé : rouvrez-le avant de saisir les soldes de départ.
error-opening-balances-already-has-entries = La société contient déjà des écritures : le bilan d’ouverture ne peut plus être généré. Corrigez l’écriture d’ouverture via le journal.
error-opening-balances-non-balance-account = Le bilan d’ouverture ne peut toucher que des comptes de bilan (actifs et passifs) — retirez les comptes de produits et de charges.


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
bank-import-errors-bank-account-not-found = Compte bancaire introuvable.
bank-import-errors-parse-failed = Le fichier CAMT.053 n'a pas pu être analysé.

# Warnings (preview 200 OK)
bank-import-warnings-balance-mismatch = Solde de clôture incohérent.
bank-import-warnings-unsupported-currency = Devise non supportée v0.1.
bank-import-warnings-ignored-statements = Certains statements du fichier ne correspondent pas au compte sélectionné et seront ignorés.
# Story 8-3 — détection doublons + rejet partiel
bank-import-warnings-duplicate-file = Ce fichier a déjà été importé.
bank-import-warnings-duplicate-lines-summary = transactions chevauchent un import précédent.
bank-import-warnings-invalid-lines-summary = lignes invalides détectées dans le CSV.
bank-import-warnings-invalid-lines-truncated = Premières 100 erreurs affichées (cap atteint).
bank-import-warnings-encoding-mismatch = L'encodage détecté diffère du profil.

# Labels UI
bank-import-labels-page-title = Import bancaire CAMT.053
bank-import-labels-bank-account-selector = Compte bancaire cible
bank-import-labels-drop-zone = Glissez votre fichier CAMT.053 ici ou cliquez pour parcourir
bank-import-labels-preview-title = Prévisualisation
bank-import-labels-confirm-import = Confirmer l'import
bank-import-labels-cancel = Annuler
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
# M8 (Pass 1 review) — clés i18n pour les codes informationnels remontés
# via `warnings.informational[]` (snake_case → kebab-case côté frontend).
bank-import-info-bank-csv-profile-auto-matched = Profil bancaire détecté automatiquement par le nom du fichier.
bank-import-info-bank-csv-multiple-profile-matches = Plusieurs profils correspondent au nom du fichier ; le premier a été retenu.
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
bank-import-profile-labels-header-row-count = Nb lignes header (0-5)
bank-import-profile-labels-column-mapping = Mapping colonnes (0-indexed)
bank-import-profile-labels-use-debit-credit-split = Colonnes débit/crédit séparées
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
reconciliation-labels-success-suffix = opération(s) réussie(s).
reconciliation-labels-failed = Échecs partiels
reconciliation-cols-tx-date = Date
reconciliation-cols-tx-amount = Montant
reconciliation-cols-tx-counterparty = Contrepartie
reconciliation-cols-candidate = Candidate
reconciliation-cols-score = Score
reconciliation-actions-accept = Accepter
reconciliation-actions-reject = Rejeter
# H8 Pass 1 code review — 8 clés canoniques AC #61 (en parallèle des
# clés existantes utilisées par le composant).
reconciliation-labels-validate-selected = Valider sélection
reconciliation-labels-reject-selected = Rejeter sélection
reconciliation-labels-score = Score
reconciliation-errors-account-locked = Compte bancaire en cours de réconciliation par un autre utilisateur. Réessayez dans quelques secondes.
reconciliation-errors-already-reconciled = Cette transaction est déjà réconciliée.
reconciliation-errors-invoice-not-eligible = Cette facture n'est pas éligible à la réconciliation.
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

reconciliation-cols-actions = Actions

# Story 8-5a-zero — Configuration `bank_account.journal_account_id`.
bank-accounts-labels-page-title = Comptes bancaires
bank-accounts-labels-page-subtitle = Lier chaque compte bancaire à un compte du plan comptable (classe 1 typique : 1020 Caisse, 1030 Banque).
bank-accounts-labels-bank-name = Banque
bank-accounts-labels-iban = IBAN
bank-accounts-labels-journal-account-id = Compte comptable lié
bank-accounts-labels-not-configured = Non configuré
bank-accounts-labels-empty = Aucun compte bancaire configuré.
bank-accounts-labels-loading = Chargement…
bank-accounts-actions-link-account = Lier au plan comptable
bank-accounts-actions-unlink-account = Délier
bank-accounts-actions-cancel = Annuler
bank-accounts-actions-submit = Lier
bank-accounts-errors-account-not-found = Compte du plan comptable non trouvé.
bank-accounts-errors-invalid-account-type = Type de compte invalide (Actif ou Passif requis).
# Story v014-1 — CRUD bank_accounts post-onboarding
bank-accounts-errors-has-transactions = Le compte bancaire contient des transactions — archivage refusé pour préserver l'audit comptable.
bank-accounts-errors-cannot-archive-primary = Le compte principal ne peut pas être archivé tant qu'un autre compte non-archivé existe. Définissez d'abord un autre compte comme principal, puis archivez celui-ci.
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
bank-accounts-labels-balance = Solde
bank-accounts-labels-balance-unavailable = Solde non disponible (lier au plan comptable)
bank-accounts-labels-qr-iban = QR-IBAN (optionnel)
bank-accounts-labels-is-primary = Compte principal
bank-accounts-labels-primary-badge = Principal
bank-accounts-labels-archived-badge = Archivé
bank-accounts-confirm-archive = Confirmer l'archivage de ce compte bancaire ? Cette action est irréversible v0.1.
bank-accounts-tooltip-journal-account = Lie ce compte bancaire à un compte du plan comptable (typiquement 1020 Caisse, 1030 Banque). Permet à la réconciliation automatique de créer les écritures vers le bon compte, et l'affichage du solde sur la page d'accueil. Note multi-comptes : si plusieurs comptes courants distincts, lier au sous-compte spécifique (1030.001 BCV CHF), pas au parent 1030.
bank-accounts-toast-create-success = Compte bancaire créé.
bank-accounts-toast-update-success = Compte bancaire modifié.
bank-accounts-toast-archive-success = Compte bancaire archivé.

# Story v014-1 — Homepage widget bank accounts (F14 Pass 1 code review)
homepage-bank-total-liquidity = Total liquidités
homepage-bank-total-partial = (comptes liés uniquement)
homepage-bank-balance-unavailable = Solde non disponible — lier au plan comptable
homepage-bank-last-transaction = Dernière transaction
settings-bank-manage = Gérer dans Administration → Comptes bancaires
settings-bank-manage-hint = Pour ajouter, modifier ou archiver un compte bancaire, utilisez la page dédiée Administration → Comptes bancaires.
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
reports-trial-balance-period-note = La balance de vérification affiche le mouvement de la période (par exercice). Le total par compte n'est pas comparable au solde cumulé du même compte au bilan (report à-nouveau depuis l'origine).

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
reports-pdf-header-period = Période
reports-pdf-empty-message = Aucune écriture dans la période sélectionnée.

# Story 9-2b — Export global ZIP (souveraineté des données) — 12 clés
nav-export-global = Export global
# Story 17-3b — sauvegarde complète d'installation (.keshbackup, Admin)
nav-admin-backup = Sauvegarde complète
admin-backup-page-title = Sauvegarde complète de l'installation
admin-backup-page-description = Télécharge l'intégralité de l'installation (toutes les sociétés, les utilisateurs et les données système) dans un fichier .keshbackup unique, pour migrer ou sauvegarder. À distinguer de l'export global d'une seule société.
admin-backup-action-export = Exporter toute l'installation
admin-backup-action-exporting = Export en cours…
admin-backup-toast-success = Sauvegarde de l'installation téléchargée.
admin-backup-error-generic = Échec de l'export de l'installation. Réessayez dans quelques instants.
admin-backup-page-hint-secret = Le fichier .keshbackup contient des données sensibles (identifiants, jetons). Conservez-le en lieu sûr.
# Story 17-3d — import/restauration complète d'installation (.keshbackup, Admin)
nav-admin-restore = Restaurer / Importer
admin-restore-page-title = Restauration / import d'installation
admin-restore-page-description = Téléversez un fichier .keshbackup pour remplacer l'intégralité de l'installation actuelle (migration ou restauration). Opération destructrice : une sauvegarde de l'état actuel est créée côté serveur avant l'import.
admin-restore-file-label = Fichier .keshbackup à importer
admin-restore-action-import = Importer et remplacer l'installation
admin-restore-action-importing = Import en cours…
admin-restore-confirm-title = Remplacer toute l'installation ?
admin-restore-confirm-body = Cette action va remplacer TOUTES les données de l'installation actuelle. Une sauvegarde de l'état actuel sera créée côté serveur avant l'import. Vous serez déconnecté et devrez vous reconnecter avec les identifiants de l'instance importée.
admin-restore-confirm-cancel = Annuler
admin-restore-confirm-ok = Confirmer le remplacement
admin-restore-toast-success = Import réussi — vous allez être déconnecté.
admin-restore-error-version = Ce backup requiert une version de Kesh plus récente ({ $src }) que celle installée ({ $bin }). Mettez à jour Kesh avant de réimporter.
admin-restore-error-schema = Schéma du backup incompatible avec cette version de Kesh (table { $table }).
admin-restore-error-invalid = Fichier de sauvegarde invalide ou corrompu. Vérifiez qu'il s'agit bien d'un fichier .keshbackup produit par Kesh.
admin-restore-error-generic = Échec de l'import. L'état précédent de l'installation a été préservé.
export-global-title = Export global de vos données
export-global-description = Exportez toutes vos données comptables (comptes, écritures, contacts, factures, transactions bancaires) au format CSV dans un fichier ZIP. Utilisez cet export pour archiver, migrer vers un autre logiciel, ou conserver vos données 10 ans (Swiss CO Art. 958f).
export-global-button = Lancer l'export
export-global-loading = Génération de l'export…
export-global-success = Export téléchargé.
export-global-error-generic = Impossible de générer l'export global. Vérifiez votre connexion et réessayez.
export-global-filename-hint = Le fichier sera téléchargé sous le nom kesh-export-{ $companyShort }-{ $date }.zip
export-global-content-includes = L'export contient : plan comptable, exercices, écritures, contacts, produits, factures, comptes bancaires, historique des imports bancaires, transactions, taux de TVA actifs et historiques, paramètres de facturation, règles de réconciliation, profils d'import bancaire, et un manifeste metadata.json avec hash SHA-256 de chaque fichier pour vérification d'intégrité.
export-global-content-excludes = Ne contient pas : utilisateurs (PII + mots de passe), tokens de session, journal d'audit interne, état d'onboarding (raisons de sécurité et technicité).
export-global-souverainete-note = Vos données vous appartiennent. Kesh ne fait aucune copie de cet export sur ses serveurs.
error-global-export-failed = L'export global n'a pas pu être généré. Si le problème persiste, contactez le support.
error-admin-full-export-failed = L'export de l'installation n'a pas pu être généré. Réessayez dans quelques instants ; si le problème persiste, contactez le support.
error-admin-full-import-failed = L'import de l'installation a échoué. L'état précédent a été préservé (un backup automatique a été créé avant l'opération). Vérifiez les logs serveur, puis réessayez.
error-invalid-backup-structure = Le fichier de sauvegarde est invalide ou corrompu (structure inattendue ou contrôle d'intégrité échoué). Vérifiez qu'il s'agit bien d'un fichier .keshbackup produit par Kesh.
error-import-schema-mismatch = Le schéma de ce backup est incompatible avec cette version de Kesh. Mettez à jour Kesh ou utilisez un backup compatible.
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
common-edit = Modifier
error-unexpected = Erreur inattendue.
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
contact-form-language = Langue de correspondance
contact-form-language-inherited = Héritée (langue de l'instance)
contact-form-salutation = Civilité
contact-salutation-neutre = Neutre
contact-salutation-monsieur = Monsieur
contact-salutation-madame = Madame
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
dunning-cgv-hint = Les frais de rappel ne sont exigibles qu'avec une base contractuelle (CGV). Ils ne sont pas inclus dans le QR de la facture jointe.
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
