# Kesh — Messages français (Suisse)

# Erreurs d'authentification
error-invalid-credentials = Identifiants invalides
error-unauthenticated = Non authentifié
error-invalid-refresh-token = Session expirée
error-rate-limited = Trop de tentatives

# Erreurs d'autorisation
error-forbidden = Accès interdit
error-cannot-disable-self = Impossible de désactiver son propre compte
error-cannot-disable-last-admin = Impossible de désactiver le dernier administrateur

# Erreurs de ressource
error-not-found = Ressource introuvable
error-conflict = Ressource déjà existante
error-optimistic-lock = Conflit de version — la ressource a été modifiée
error-foreign-key = Référence invalide
error-check-constraint = Valeur invalide
error-illegal-state = Transition d'état interdite

# Erreurs de validation
error-validation = Erreur de validation
error-username-empty = Le nom d'utilisateur ne peut pas être vide
error-username-too-long = Le nom d'utilisateur ne doit pas dépasser { $max } caractères

# Erreurs système
error-internal = Erreur interne
error-service-unavailable = Service temporairement indisponible

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
nav-contacts = Carnet d'adresses
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
contact-error-ide-duplicate = Un contact avec ce numéro IDE existe déjà
contact-error-not-found = Contact introuvable
contact-error-archived-no-modify = Contact archivé — modification ou archivage supplémentaire interdit
contact-conflict-title = Conflit de version
contact-conflict-body = Ce contact a été modifié ailleurs. Voulez-vous recharger la version actuelle ?
error-ide-already-exists = Un contact avec ce numéro IDE existe déjà

# Story 4.2 — Conditions de paiement & catalogue produits
contact-form-payment-terms = Conditions de paiement
contact-form-payment-terms-placeholder = ex: 30 jours net
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
