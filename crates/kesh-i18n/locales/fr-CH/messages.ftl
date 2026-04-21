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

# Navigation sidebar (Story 6.3)
nav-home = Accueil
nav-contacts = Carnet d'adresses
nav-products = Catalogue
nav-invoices = Factures
nav-invoicing-due-dates = Échéancier
nav-settings = Paramètres

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

# --- Story 5.1 : Factures brouillon ---
invoices-page-title = Factures
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
error-invoice-too-many-lines-for-pdf = La facture contient { $count } lignes — le PDF A4 est limité à { $max } lignes en v0.1.
error-pdf-generation-failed = Échec de la génération du PDF.

# Story 5.4 — Échéancier factures
due-dates-title = Échéancier
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
