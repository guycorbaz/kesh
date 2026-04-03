---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/planning-artifacts/ux-design-specification.md
---

# Kesh - Epic Breakdown

## Vue d'ensemble

Ce document fournit le découpage complet en epics et stories pour Kesh, en décomposant les exigences du PRD, du UX Design et de l'Architecture en stories implémentables.

## Inventaire des Exigences

### Exigences Fonctionnelles

#### Configuration & Onboarding

- FR1 : L'administrateur peut installer Kesh via docker-compose en moins de 15 minutes
- FR2 : L'administrateur peut configurer l'application via des variables d'environnement
- FR3 : L'administrateur peut configurer le nom d'utilisateur et le mot de passe du compte admin initial via des variables d'environnement
- FR4 : L'assistant d'onboarding guide l'utilisateur à travers le type d'organisation, les coordonnées, et la configuration des comptes bancaires
- FR5 : Le système installe automatiquement le plan comptable et les journaux adaptés au type d'organisation choisi
- FR6 : L'administrateur peut configurer la politique de mot de passe
- FR7 : Le système affiche un disclaimer légal « ne remplace pas un fiduciaire »
- FR8 : Le système expose un endpoint de healthcheck (`/health`)

#### Gestion des Utilisateurs & Sécurité

- FR9 : L'administrateur peut créer, désactiver et gérer les comptes utilisateurs
- FR10 : L'administrateur peut attribuer un ou plusieurs rôles à un utilisateur (Admin, Comptable, Consultation)
- FR11 : Le système contrôle l'accès aux fonctionnalités en fonction des rôles (RBAC)
- FR12 : L'utilisateur peut s'authentifier avec un identifiant et un mot de passe
- FR13 : Le système renouvelle silencieusement la session et la termine après 15 minutes d'inactivité (configurable)
- FR14 : L'utilisateur peut changer son propre mot de passe
- FR15 : L'administrateur peut réinitialiser le mot de passe d'un utilisateur
- FR16 : Le système bloque les tentatives de connexion après 5 échecs en 15 minutes, avec un délai de déblocage de 30 minutes
- FR17 : Le système utilise le verrouillage optimiste pour gérer la concurrence entre utilisateurs

#### Plan Comptable & Écritures

- FR18 : L'utilisateur peut charger un plan comptable suisse standard (PME, association, indépendant)
- FR19 : L'utilisateur peut ajouter, modifier et archiver des comptes dans le plan comptable
- FR20 : L'utilisateur peut saisir des écritures en partie double (débit/crédit)
- FR21 : Le système refuse toute écriture déséquilibrée (débit ≠ crédit)
- FR22 : L'utilisateur peut saisir des écritures dans différents journaux (Achats, Ventes, Banque, Caisse, OD)
- FR23 : L'utilisateur peut supprimer des écritures tant que l'exercice est ouvert
- FR24 : Le système interdit la modification et la suppression des écritures d'un exercice clôturé

#### Carnet d'Adresses & Contacts

- FR25 : L'utilisateur peut gérer un carnet d'adresses unifié (personnes et entreprises)
- FR26 : L'utilisateur peut marquer un contact comme client, fournisseur, ou les deux
- FR27 : Le système valide le format et le checksum du numéro IDE (CHE) lorsqu'il est saisi (champ optionnel)
- FR28 : L'utilisateur peut associer des conditions de paiement par défaut à un contact

#### Catalogue Produits/Services

- FR29 : L'utilisateur peut gérer un catalogue de produits/services (nom, description, prix unitaire, taux TVA)
- FR30 : L'utilisateur peut sélectionner des articles du catalogue lors de la création d'une facture

#### Facturation

- FR31 : L'utilisateur peut créer une facture avec des lignes libres ou depuis le catalogue (quantité, prix, TVA)
- FR32 : L'utilisateur peut supprimer une facture en brouillon
- FR33 : L'utilisateur peut valider une facture, ce qui lui attribue un numéro séquentiel définitif
- FR34 : Le système génère un PDF QR Bill conforme aux spécifications SIX 2.2 pour chaque facture validée
- FR35 : L'utilisateur peut configurer le format de numérotation des factures
- FR36 : L'utilisateur peut annuler une facture validée uniquement par la création d'un avoir (v0.2)
- FR37 : Le système gère une séquence de numérotation séparée pour les avoirs (v0.2)
- FR38 : Le PDF s'ouvre dans un nouvel onglet du navigateur

#### Paiements (v0.2)

- FR39 : L'utilisateur peut sélectionner des factures fournisseurs ouvertes pour générer un lot de paiements
- FR40 : Le système génère un fichier pain.001.001.03 conforme au schéma XSD SIX
- FR41 : Le système valide le format IBAN avant la génération du fichier de paiement

#### Import Bancaire & Réconciliation

- FR42 : L'utilisateur peut importer des relevés bancaires aux formats CAMT.053 et CSV
- FR43 : Le système détecte les fichiers déjà importés et les transactions en doublon
- FR44 : Le système propose automatiquement des contreparties pour les transactions connues
- FR45 : L'utilisateur peut créer manuellement une contrepartie pour les transactions inconnues et réconcilier
- FR46 : Le système propose la création d'une règle d'affectation après une affectation manuelle
- FR47 : L'utilisateur peut gérer les règles d'affectation automatique
- FR48 : L'utilisateur peut éclater manuellement une transaction agrégée en sous-lignes
- FR49 : Le système extrait les sous-transactions (`TxDtls`) des fichiers CAMT quand elles sont présentes
- FR50 : L'utilisateur peut importer un relevé lié à un compte bancaire spécifique (multi-comptes)
- FR51 : Le système gère le rejet partiel d'un import avec un listing détaillé des erreurs
- FR52 : Le parseur CSV supporte UTF-8 et ISO-8859-1 avec détection automatique
- FR53 : L'utilisateur peut configurer des profils de format par banque pour l'import CSV

#### TVA (v0.2)

- FR54 : L'administrateur peut configurer les taux de TVA avec des dates de validité
- FR55 : Le système applique les arrondis TVA au centime par ligne (arrondi commercial)
- FR56 : L'utilisateur peut générer un rapport TVA par période (trimestriel/semestriel)

#### Budgets (v0.2)

- FR57 : L'utilisateur peut créer un budget annuel par compte individuel avec des montants mensuels
- FR58 : L'utilisateur peut gérer plusieurs versions d'un budget par exercice (initial, révisé)
- FR59 : L'utilisateur peut générer un rapport comparatif budget vs réalisé avec écarts

#### Clôture & Exercice (v0.2)

- FR60 : L'utilisateur peut clôturer un exercice comptable
- FR61 : Le système reporte automatiquement les soldes vers le nouvel exercice
- FR62 : L'utilisateur peut saisir un bilan d'ouverture (soldes de départ)

#### Pièces Justificatives (v0.2)

- FR63 : L'utilisateur peut attacher des fichiers (PDF, images) à une écriture
- FR64 : Le système stocke les justificatifs dans un volume de stockage dédié, séparé de la base de données

#### Rapports & Exports

- FR65 : L'utilisateur peut générer un bilan, un compte de résultat, une balance des comptes et des journaux
- FR66 : L'utilisateur peut exporter les rapports en PDF et CSV
- FR67 : Les PDF générés respectent les formats suisses (apostrophe séparateur de milliers, dates dd.mm.yyyy)
- FR68 : L'utilisateur peut exporter l'ensemble des données par table en CSV
- FR69 : L'utilisateur peut rechercher des écritures par montant, libellé, numéro de facture ou date
- FR70 : L'utilisateur peut trier, filtrer et paginer toutes les listes

#### Interface & Aide

- FR71 : Le système affiche des notifications banner pour les succès, avertissements et erreurs non-bloquantes
- FR72 : Le système affiche des modals pour les erreurs bloquantes (session expirée, conflit de version)
- FR73 : Le système fournit des tooltips contextuels sur les termes comptables et champs techniques
- FR74 : L'utilisateur peut accéder à 3 manuels PDF embarqués (démarrage, utilisateur, administrateur) (v0.2)

#### Internationalisation

- FR75 : Le système est disponible en français, allemand, italien et anglais
- FR76 : La langue de l'interface est configurée au niveau de l'instance. Elle est découplée de la langue des données comptables (libellés, noms de comptes). L'utilisateur peut changer la langue de l'interface sans impacter les données saisies.

#### Déploiement & Maintenance

- FR77 : Le système se déploie via une seule commande et un fichier de configuration (reverse proxy + application + base de données)
- FR78 : Le système détecte une nouvelle version au démarrage et avertit de faire un backup avant migration
- FR79 : Le système applique les migrations de schéma automatiquement
- FR80 : Le système fournit un script de seed rechargeable pour la démo et les tests

#### Modèles de Documents (v0.2)

- FR81 : L'utilisateur peut personnaliser les modèles de documents générés (factures, rapports) : logo, coordonnées, textes de pied de page

#### Page d'Accueil

- FR82 : Le système affiche une page d'accueil après connexion avec accès rapide aux fonctions principales (dernières écritures, factures ouvertes, soldes des comptes bancaires)

#### Comptes Bancaires

- FR83 : L'utilisateur peut configurer et gérer ses comptes bancaires (nom, IBAN, banque)
- FR84 : L'assistant d'onboarding propose la configuration des comptes bancaires avec validation QR-IBAN/IBAN

#### Lettrage (v0.2)

- FR85 : L'utilisateur peut lettrer des écritures entre elles pour marquer les correspondances (facture ↔ paiement)
- FR86 : L'utilisateur peut délettrer des écritures précédemment lettrées tant que l'exercice est ouvert

#### Versioning des parseurs (v0.2)

- FR87 : Le système identifie la version du format utilisé lors de l'import (CAMT.053, pain.001) et sélectionne le parseur/générateur correspondant

#### Traçabilité & Audit

- FR88 : Le système enregistre un journal d'audit des actions utilisateurs sur les données comptables (création, modification, suppression d'écritures, clôture d'exercice) avec l'identifiant utilisateur et l'horodatage

#### Résilience Frontend

- FR89 : Le frontend s'affiche même si la base de données est inaccessible, avec un message d'erreur explicite invitant à vérifier l'état du serveur

### Exigences Non-Fonctionnelles

#### Performance

- NFR-PERF-1 : Les pages se chargent en moins de 300ms
- NFR-PERF-2 : L'import d'un relevé mensuel (200 transactions) s'exécute en moins de 2s
- NFR-PERF-3 : La génération d'un PDF (facture QR Bill, rapport) s'exécute en moins de 3s
- NFR-PERF-4 : Le système supporte 2-5 utilisateurs simultanés par instance sans dégradation

#### Sécurité

- NFR-SEC-1 : Tous les mots de passe sont hashés avec argon2 ou bcrypt, jamais stockés en clair
- NFR-SEC-2 : Aucune donnée n'est accessible sans authentification JWT valide
- NFR-SEC-3 : La communication est chiffrée via TLS en production (nginx reverse proxy)
- NFR-SEC-4 : Le rate limiting protège l'endpoint de connexion contre le brute-force
- NFR-SEC-5 : Les données comptables ne sont jamais transmises à un tiers (sauf IA externe avec consentement explicite, post-MVP)
- NFR-SEC-6 : Les credentials de base de données ne sont jamais exposés dans les logs ou l'API

#### Fiabilité & Intégrité

- NFR-REL-1 : La balance débit/crédit est garantie correcte à tout moment — aucun écart possible
- NFR-REL-2 : Aucune donnée comptable n'est perdue en cas de crash ou de redémarrage
- NFR-REL-3 : Les fichiers pain.001 générés passent la validation XSD officielle SIX
- NFR-REL-4 : Les QR Bill générés sont conformes aux spécifications SIX 2.2 (dimensions, position, contenu)
- NFR-REL-5 : Les migrations de schéma préservent l'intégrité des données des exercices passés
- NFR-REL-6 : L'arithmétique monétaire utilise exclusivement des types décimaux exacts (`rust_decimal`), jamais de flottants

#### Accessibilité

- NFR-ACC-1 : L'interface s'inspire de WCAG AA (contraste suffisant, navigation clavier, labels sur les formulaires) sans contrainte stricte de conformité
- NFR-ACC-2 : Le zoom navigateur reste fonctionnel à 200%

#### Internationalisation

- NFR-I18N-1 : Toutes les chaînes de l'interface sont externalisées (format Fluent `.ftl`)
- NFR-I18N-2 : Les formats régionaux suisses sont respectés : dates `dd.mm.yyyy`, montants avec apostrophe `1'234.56`
- NFR-I18N-3 : Les PDF générés respectent les mêmes formats régionaux

#### Maintenabilité

- NFR-MAINT-1 : Le code source est documenté selon les meilleures pratiques (doc comments Rust, JSDoc Svelte)
- NFR-MAINT-2 : Aucun code dupliqué (principe DRY)
- NFR-MAINT-3 : Tests unitaires sur toute la logique métier (moteur comptable, TVA, calculs financiers)
- NFR-MAINT-4 : Tests d'intégration sur les parseurs (CAMT.053, QR Bill, pain.001) avec les fichiers de test officiels SIX
- NFR-MAINT-5 : Tests E2E avec Playwright couvrant chaque parcours utilisateur
- NFR-MAINT-6 : L'API est versionnée (`/api/v1/`) dès le premier jour

#### Déploiement

- NFR-DEPLOY-1 : L'application se déploie via une seule commande `docker-compose up`
- NFR-DEPLOY-2 : L'image Docker résultante pèse moins de 100 Mo
- NFR-DEPLOY-3 : Les logs applicatifs sont émis sur stdout/stderr (standard Docker)
- NFR-DEPLOY-4 : Le healthcheck `/health` vérifie la connexion à la base de données

### Exigences Additionnelles (Architecture)

#### Starter Template & Infrastructure

- ARCH-1 : Frontend initialisé via SvelteKit SPA (`adapter-static`, `ssr=false`) avec `npx sv create frontend`
- ARCH-2 : Backend organisé en Cargo workspace manuel avec 10 crates (`kesh-core`, `kesh-db`, `kesh-api`, `kesh-reconciliation`, `kesh-i18n`, `kesh-report`, `kesh-seed`, `kesh-import`, `kesh-payment`, `kesh-qrbill`)
- ARCH-3 : Axum sert le frontend en production via `tower-http::ServeDir` (plus nginx par défaut)
- ARCH-4 : TLS non géré par Kesh — HTTP pur, TLS = infrastructure (nginx/Traefik/Caddy)
- ARCH-5 : Docker-compose simplifié à 2 containers : kesh + mariadb (nginx optionnel pour TLS)
- ARCH-6 : Le frontend s'affiche même si la base de données est inaccessible (message d'erreur côté client)
- ARCH-7 : Workflow de développement : Vite dev server + proxy vers Axum pour hot reload frontend
- ARCH-8 : Outils recommandés : `cargo-nextest`, `cargo-deny` (audit licences/sécurité), `cargo-udeps` (dépendances inutilisées)

#### Séparation des Responsabilités

- ARCH-9 : `kesh-core` contient la logique métier pure, sans I/O — types forts, newtypes (Money, Iban, CheNumber), validation métier
- ARCH-10 : `kesh-db` implémente le repository pattern avec SQLx direct — contrôle total sur le SQL
- ARCH-11 : Crates publiables (`kesh-import`, `kesh-payment`, `kesh-qrbill`) avec zéro dépendance interne — types autonomes, conversion via `From/Into` côté kesh-core/api
- ARCH-12 : Migrations SQLx dans `crates/kesh-db/migrations/` — fichiers versionnés, zéro perte de données
- ARCH-13 : `kesh-reconciliation` dédié — matching, règles d'affectation, mutex par compte bancaire

#### Authentification & API

- ARCH-14 : JWT via `jsonwebtoken` crate — access token ~15 min + refresh token UUID opaque en base
- ARCH-15 : Hashing mots de passe avec Argon2id
- ARCH-16 : RBAC hiérarchique : Consultation < Comptable < Admin — chaque rôle hérite du précédent
- ARCH-17 : Rate limiting sur `/api/v1/auth/login` uniquement — compteur en mémoire par IP, middleware tower
- ARCH-18 : API REST préfixe `/api/v1/`, routes kebab-case, pagination offset/limit
- ARCH-19 : Format d'erreur structuré avec code métier + message + details
- ARCH-20 : Sérialisation `serde`/`serde_json`, champs JSON en camelCase (`#[serde(rename_all = "camelCase")]`)
- ARCH-21 : Montants transmis en string décimal (`"1234.56"`) — jamais de float dans le JSON

#### Frontend Architecture

- ARCH-22 : State management via Svelte stores natifs (writable, derived)
- ARCH-23 : Composants UI via shadcn-svelte (Svelte 5, Tailwind CSS v4) — code copié dans le projet, modifiable
- ARCH-24 : Communication API via `fetch` natif + wrapper léger (JWT, refresh, erreurs)
- ARCH-25 : Formatage montants/dates via `Intl.NumberFormat` / `Intl.DateTimeFormat` (locale `de-CH`)
- ARCH-26 : Organisation frontend par feature (`features/journal-entries/`, `features/invoicing/`, etc.)

#### Patterns d'Implémentation Obligatoires

- ARCH-27 : Jamais de f64 pour les montants — `rust_decimal::Decimal` exclusivement
- ARCH-28 : Toute écriture comptable doit être équilibrée — validation dans `kesh-core` avant persistance
- ARCH-29 : Tout code public documenté — `///` Rust, JSDoc Svelte
- ARCH-30 : Tests unitaires pour toute logique métier — pas de code métier sans test
- ARCH-31 : Verrouillage optimiste sur toute entité modifiable — champ `version` systématique
- ARCH-32 : Erreurs structurées avec code métier — jamais de string d'erreur en dur côté frontend

#### Conventions de Nommage

- ARCH-33 : Base de données — tables snake_case pluriel, colonnes snake_case, FK `{table_singulier}_id`, index `idx_{table}_{colonnes}`
- ARCH-34 : API REST — routes kebab-case pluriel, query params camelCase
- ARCH-35 : Code Rust — structs/enums PascalCase, fonctions/modules snake_case, newtypes PascalCase
- ARCH-36 : Code Svelte/TS — composants PascalCase, routes kebab-case, fonctions/variables camelCase, stores camelCase

#### Stratégie de Test

- ARCH-37 : Tests unitaires — `kesh-core` seul (logique pure, pas d'I/O), co-localisés (`#[cfg(test)] mod tests`)
- ARCH-38 : Tests d'intégration — `kesh-seed` + `kesh-db` (données réalistes en base), dans `crates/{crate}/tests/`
- ARCH-39 : Tests E2E Playwright — `kesh-api` complet avec base seedée, dans `frontend/tests/e2e/`
- ARCH-40 : Tests Svelte co-localisés (`Component.test.ts` à côté de `Component.svelte`)

#### Infrastructure CI/CD

- ARCH-41 : GitHub Actions pour CI/CD (build, tests, clippy, fmt, release Docker image)
- ARCH-42 : Logging via `tracing` crate (écosystème Tokio/Axum), stdout/stderr
- ARCH-43 : Configuration via variables d'environnement (`dotenvy` + fichier `.env`)

#### Séquence d'Implémentation Recommandée

- ARCH-44 : 1. Scaffold workspace Cargo + SvelteKit SPA → 2. `kesh-core` (types, newtypes, validation) → 3. `kesh-db` (repository, migrations, schéma) → 4. `kesh-api` (Axum, auth JWT, RBAC, ServeDir) → 5. Frontend (SvelteKit, shadcn-svelte, wrapper fetch, stores) → 6. Crates métier (import, qrbill, reconciliation, etc.)

### Exigences UX Design

#### Design Tokens & Fondation Visuelle

- UX-DR1 : Configurer Tailwind CSS avec les design tokens Kesh : palette fonctionnelle (bleu ardoise `#1e40af` primaire, vert `#16a34a` succès, rouge `#dc2626` erreur, ambre `#d97706` attention, bleu clair `#0ea5e9` info), fonds (`#ffffff` principal, `#f8fafc` secondaire), textes (`#1e293b` principal, `#64748b` secondaire), bordures (`#e2e8f0`)
- UX-DR2 : Implémenter le système typographique unique basé sur Inter — H1 24px/600, H2 20px/600, H3 16px/600, corps 14px/400, montants 14px/500 avec `tabular-nums`, labels 14px/500, petit texte 12px/400
- UX-DR3 : Implémenter le système d'espacement adaptatif selon le mode — Mode Guidé : `gap-4`, `p-6`, `my-8`, hauteur ligne tableau 48px ; Mode Expert : `gap-2`, `p-4`, `my-4`, hauteur ligne tableau 36px
- UX-DR4 : Définir la structure de page fixe — header fixe (logo, recherche globale, profil), sidebar fixe gauche 200-240px (navigation par activité organisée par fréquence), zone de contenu fluide, footer discret (version, disclaimer)
- UX-DR5 : Appliquer `font-variant-numeric: tabular-nums` sur toutes les colonnes de montants pour alignement parfait au centime

#### Composants Sur Mesure à Créer

- UX-DR6 : Créer le composant formulaire de saisie d'écriture — débit/crédit avec autocomplétion des comptes par numéro ou nom, validation instantanée de l'équilibre, flux Tab gauche→droite, indicateur d'équilibre en temps réel
- UX-DR7 : Créer le composant split view factures fournisseurs — aperçu PDF à gauche, formulaire de saisie à droite, pré-remplissage depuis extraction QR Code
- UX-DR8 : Créer le composant tableau de réconciliation bancaire — matching automatique affiché, sélection en lot, création de contrepartie en 2 clics, filtres et tri, pagination
- UX-DR9 : Créer les bannières contextuelles — onboarding incomplet (bleu), mode démo (jaune), erreurs persistantes ; conditions d'affichage et de disparition définies
- UX-DR10 : Créer l'indicateur d'équilibre débit/crédit en temps réel — feedback visuel immédiat pendant la saisie, vert si équilibré, rouge si déséquilibré
- UX-DR11 : Créer le composant de prévisualisation PDF QR Bill — rendu dans l'interface avant ouverture dans un nouvel onglet
- UX-DR12 : Créer le composant d'import par glisser-déposer — zone de drop pour fichiers bancaires (CAMT.053, CSV), prévisualisation avant import, détection automatique du format

#### Accessibilité

- UX-DR13 : Garantir les ratios de contraste WCAG AA — minimum 4.5:1 pour texte normal, 3:1 pour grand texte ; vérifier texte principal sur blanc (13.5:1), texte secondaire sur blanc (4.6:1), bouton primaire blanc sur bleu (7.2:1)
- UX-DR14 : Implémenter la navigation clavier complète — focus visible sur tous les éléments interactifs (outline bleu `#3b82f6`), Tab/Shift+Tab dans les formulaires, navigation clavier dans les tableaux
- UX-DR15 : Garantir que les états ne sont jamais communiqués uniquement par la couleur — vert succès accompagné de icone check, rouge erreur accompagné de icone croix, etc.
- UX-DR16 : Respecter les tailles de cible cliquable — minimum 44x44px en mode Guidé, minimum 32x32px en mode Expert
- UX-DR17 : Interface fonctionnelle à 200% de zoom navigateur
- UX-DR18 : Intégrer axe-core dans la CI pour détecter les régressions d'accessibilité

#### Interaction Mode Guidé / Expert

- UX-DR19 : Implémenter le choix Mode Guidé / Expert à l'onboarding (« Comment préférez-vous utiliser Kesh ? ») avec possibilité de changer à tout moment dans le profil utilisateur
- UX-DR20 : En mode Guidé — espacements généreux, boutons plus grands avec labels explicites, confirmations avant actions, aide contextuelle visible, états vides avec explication + suggestion d'ordre + bouton
- UX-DR21 : En mode Expert — espacements compacts, boutons avec icones et labels en tooltip, raccourcis clavier actifs (`Ctrl+N` nouvelle écriture, `Ctrl+S` sauvegarder), actions en lot, états vides avec bouton d'action uniquement
- UX-DR22 : Appliquer les variantes Mode Guidé / Expert sur tous les composants métier — formulaire d'écriture (assistant pas-à-pas vs formulaire direct), import bancaire (étapes guidées vs écran compact), réconciliation (une par une vs validation en lot)

#### Flux d'Onboarding

- UX-DR23 : Implémenter le Chemin A (Exploration) — 1. Langue d'interface (4 langues, noms dans leur langue), 2. Mode Guidé/Expert, 3. Charger données de démo ; bannière permanente jaune « Instance de démonstration — données fictives » ; réinitialisation complète pour passer en production
- UX-DR24 : Implémenter le Chemin B (Production) — 1. Langue d'interface, 2. Mode Guidé/Expert, 3. Type d'organisation (Indépendant/Association/PME), 4. Langue comptable (FR/DE/IT/EN), 5. Coordonnées (nom/raison sociale, adresse, IDE optionnel), 6. Compte bancaire principal (banque, IBAN, QR-IBAN) avec « Configurer plus tard »
- UX-DR25 : Chaque étape d'onboarding est atomique et persistée immédiatement en base — l'onboarding peut être interrompu et repris via bannière contextuelle bleue « Configuration incomplète — Terminer la configuration »
- UX-DR26 : Kesh fonctionne même avec un onboarding partiel — les fonctionnalités dépendantes des étapes manquantes affichent un message avec lien vers la configuration
- UX-DR27 : L'onboarding est un raccourci vers les paramètres — correspondance explicite : Langue interface → Profil, Mode → Profil, Type orga → Paramètres > Organisation, Langue comptable → Paramètres > Comptabilité, Coordonnées → Paramètres > Organisation, Compte bancaire → Paramètres > Comptes bancaires

#### Navigation & Structure

- UX-DR28 : Navigation par activité avec vocabulaire utilisateur — « Facturer » pas « Module Facturation », « Importer un relevé » pas « Import bancaire CAMT.053 », « Voir mes comptes » pas « Balance des comptes »
- UX-DR29 : Hiérarchie de navigation par fréquence d'usage — Quotidien (accès direct) : Facturer, Paiements, Import relevé ; Mensuel (accès secondaire) : Réconciliation, Écritures manuelles, Rapports ; Rare (paramètres) : Plan comptable, Configuration, Export
- UX-DR30 : Configuration centralisée — un seul menu « Paramètres » avec sous-sections claires, pas de dispersion (anti-pattern Bexio)
- UX-DR31 : Export global accessible — bouton visible dans le menu principal, ZIP complet (CSV par table + pièces justificatives + métadonnées)
- UX-DR32 : Aucun cul-de-sac dans la navigation — toujours un chemin de retour ou une action suivante, bouton navigateur (retour arrière) fonctionnel
- UX-DR33 : Recherche globale accessible partout depuis le header

#### Patterns d'Interaction

- UX-DR34 : Implémenter le pattern unifié « Sélectionner → Vérifier → Valider → Résultat » pour les trois flux principaux — Facturer (Client + lignes → Récapitulatif → Confirmer → PDF QR Bill), Payer (Factures en attente → Montants + IBAN → Confirmer → pain.001), Importer (Fichier bancaire → Prévisualisation → Confirmer → Écritures)
- UX-DR35 : Implémenter le flux de paiement en deux temps — Temps 1 (au fil de l'eau) : facture reçue → créer un paiement → ajouté à la liste d'attente ; Temps 2 (jour de paiement) : sélectionner → vérifier → générer pain.001
- UX-DR36 : Implémenter l'autocomplétion des comptes — par numéro (« 1020 ») ou par nom (« Banque »), dans tous les champs de saisie de compte
- UX-DR37 : Raccourcis clavier v0.1 — `Ctrl+N` nouvelle écriture, `Ctrl+S` sauvegarder, `Tab`/`Shift+Tab` navigation formulaires, navigation clavier complète dans les tableaux

#### Messages & Feedback

- UX-DR38 : Chaque message d'erreur dit ce qui s'est passé ET ce que l'utilisateur peut faire — « Écriture déséquilibrée — le total des débits (500.00) ne correspond pas au total des crédits (0.00) » plutôt que « Erreur 422 »
- UX-DR39 : Les tooltips comptables utilisent un langage bilingue — terme en langage naturel (« l'argent entre dans ce compte ») couplé à la terminologie comptable correcte (« Débit »)
- UX-DR40 : Feedback visuel immédiat sur chaque action — écriture validée → bilan mis à jour, facture validée → PDF visible, import → transactions listées ; pas d'action silencieuse
- UX-DR41 : Confirmations avant actions destructives (suppression, clôture) — bannières au lieu de modals quand possible, pour préserver le calme de l'interface

#### Expérience d'Installation

- UX-DR42 : Logs Docker affichent l'URL de l'application au démarrage — « Kesh est pret → http://localhost:{KESH_PORT} »
- UX-DR43 : Si services pas prêts au premier accès — page d'attente élégante, pas d'erreur nginx brute
- UX-DR44 : Fichiers livrés : `docker-compose.yml`, `.env` documenté (port configurable, mot de passe admin initial, nom du projet), `guide-installation.pdf` (4-5 pages : prérequis, installation 5 étapes, vérification, problèmes fréquents, section NAS, backup)

#### Stratégie de Test UX

- UX-DR45 : Tests unitaires des composants shadcn personnalisés — rendu avec différentes props/états, comportement Mode Guidé vs Expert, cas limites (valeurs vides, texte long, montants négatifs, caractères UTF-8), formats suisses (montants apostrophe, dates dd.mm.yyyy)
- UX-DR46 : Tests E2E Playwright structurés par parcours utilisateur — `marc-independent/` (onboarding, facturation, import, paiements), `sophie-association/` (onboarding, première écriture, réconciliation), `thomas-admin/` (déploiement, gestion utilisateurs), `lisa-fiduciary/` (opérations en lot, clôture)
- UX-DR47 : Chaque flux E2E testé dans les deux modes (Guidé / Expert) + chemin A (démo) et chemin B (production) pour l'onboarding
- UX-DR48 : Tests de performance — chaque page < 300ms (Lighthouse CI), tableaux 1000+ lignes rendu fluide, génération PDF < 3s
- UX-DR49 : Tests de sécurité frontend — XSS (champs échappent le HTML), CSRF (token valide sur formulaires de modification), entrées malveillantes sur tous les champs texte

### Carte de Couverture des Exigences

| FR | Epic | Description |
|---|---|---|
| FR1-FR3 | Epic 1 | Installation, configuration env, compte admin |
| FR6 | Epic 1 | Politique de mot de passe |
| FR8 | Epic 1 | Healthcheck |
| FR9-FR17 | Epic 1 | Gestion utilisateurs, auth, RBAC, JWT, rate limiting, verrouillage optimiste |
| FR4-FR5 | Epic 2 | Onboarding, plan comptable auto |
| FR7 | Epic 2 | Disclaimer légal |
| FR75-FR76 | Epic 2 | Internationalisation, découplage langue interface/données |
| FR80 | Epic 2 | Script de seed |
| FR82 | Epic 2 | Page d'accueil |
| FR83-FR84 | Epic 2 | Comptes bancaires, onboarding banque |
| FR18-FR24 | Epic 3 | Plan comptable, écritures, journaux, immutabilité |
| FR69-FR70 | Epic 3 | Recherche, tri, filtre, pagination |
| FR71-FR73 | Epic 3 | Notifications banner/modal, tooltips |
| FR88 | Epic 3 | Journal d'audit |
| FR25-FR28 | Epic 4 | Carnet d'adresses, contacts, IDE |
| FR29-FR30 | Epic 4 | Catalogue produits/services |
| FR31-FR35 | Epic 5 | Facturation, QR Bill PDF, numérotation |
| FR38 | Epic 5 | PDF nouvel onglet |
| FR42-FR53 | Epic 6 | Import CAMT.053/CSV, doublons, réconciliation, règles, profils banque |
| FR65-FR68 | Epic 7 | Bilan, résultat, balance, journaux, export PDF/CSV |
| FR77-FR79 | Epic 8 | Docker-compose (2 containers), migrations, détection version |
| FR89 | Epic 8 | Frontend accessible même si DB inaccessible |
| FR54-FR56 | Epic 9 | TVA taux configurables, arrondis, rapport |
| FR36-FR37 | Epic 10 | Avoirs, numérotation séparée |
| FR39-FR41 | Epic 10 | Paiements pain.001, validation IBAN |
| FR57-FR59 | Epic 11 | Budgets annuels, versions, rapport comparatif |
| FR60-FR62 | Epic 12 | Clôture exercice, report soldes, bilan ouverture |
| FR63-FR64 | Epic 13 | Justificatifs, stockage filesystem |
| FR74 | Epic 13 | Manuels PDF embarqués |
| FR81 | Epic 13 | Modèles documents personnalisables |
| FR85-FR86 | Epic 13 | Lettrage et délettrage |
| FR87 | Epic 13 | Versioning parseurs SIX |

**Couverture :** 89/89 FRs mappées (100%)

## Liste des Epics

### v0.1 — Cœur comptable + facturation

### Epic 1 : Fondations & Authentification
L'administrateur peut installer Kesh, se connecter et gérer les utilisateurs. Le système d'authentification JWT avec RBAC est opérationnel. L'infrastructure technique (workspace Cargo, SvelteKit SPA, MariaDB, design tokens) est en place.
**FRs couvertes :** FR1-FR3, FR6, FR8-FR17
**ARCH :** Scaffold workspace Cargo + SvelteKit, kesh-core (types, newtypes, validation), kesh-db (schema, migrations, repositories), kesh-api (Axum, auth JWT, RBAC, ServeDir), frontend (shadcn-svelte, wrapper fetch, stores)
**UX-DR :** UX-DR1-DR5 (design tokens, typographie, espacements, layout, tabular-nums), UX-DR13-DR18 (accessibilité fondation), UX-DR37 (raccourcis clavier base)

### Epic 2 : Onboarding & Configuration
L'utilisateur peut configurer son organisation, choisir sa langue, explorer la démo et accéder à une page d'accueil contextuelle. L'internationalisation 4 langues est fonctionnelle.
**FRs couvertes :** FR4-FR5, FR7, FR75-FR76, FR80, FR82-FR84
**UX-DR :** UX-DR19-DR27 (mode Guidé/Expert, onboarding chemins A/B, atomicité, bannières), UX-DR28-DR33 (navigation, configuration centralisée, recherche globale), UX-DR42-DR44 (expérience installation)

### Epic 3 : Plan Comptable & Écritures
L'utilisateur peut gérer le plan comptable, saisir des écritures en partie double avec validation d'équilibre en temps réel, rechercher et filtrer les écritures. Le journal d'audit trace toutes les actions.
**FRs couvertes :** FR18-FR24, FR69-FR70, FR71-FR73, FR88
**UX-DR :** UX-DR6 (formulaire écriture), UX-DR10 (indicateur équilibre), UX-DR36 (autocomplétion comptes), UX-DR38-DR41 (messages, tooltips, feedback)

### Epic 4 : Carnet d'Adresses & Catalogue
L'utilisateur peut gérer ses clients et fournisseurs avec validation IDE, et maintenir un catalogue de produits/services pour la facturation.
**FRs couvertes :** FR25-FR30

### Epic 5 : Facturation QR Bill
L'utilisateur peut créer des factures avec lignes libres ou depuis le catalogue, valider et générer des PDF QR Bill conformes SIX 2.2.
**FRs couvertes :** FR31-FR35, FR38
**ARCH :** kesh-qrbill (crate publiable indépendante)
**UX-DR :** UX-DR11 (prévisualisation PDF), UX-DR34 (pattern Sélectionner→Vérifier→Valider→Résultat)

### Epic 6 : Import Bancaire & Réconciliation
L'utilisateur peut importer des relevés CAMT.053 et CSV, détecter les doublons, réconcilier les transactions avec matching automatique et règles d'affectation évolutives.
**FRs couvertes :** FR42-FR53
**ARCH :** kesh-import (crate publiable), kesh-reconciliation (matching, rules, mutex)
**UX-DR :** UX-DR7 (split view factures), UX-DR8 (tableau réconciliation), UX-DR12 (glisser-déposer)

### Epic 7 : Rapports & Exports
L'utilisateur peut générer bilan, compte de résultat, balance des comptes et journaux en PDF et CSV avec formats suisses.
**FRs couvertes :** FR65-FR68
**ARCH :** kesh-report (PDF tabulaire + CSV), kesh-i18n (formatage suisse)
**UX-DR :** UX-DR31 (export global)

### Epic 8 : Déploiement & Opérations
L'administrateur peut déployer Kesh via docker-compose, les migrations s'appliquent automatiquement et la détection de version avertit avant migration.
**FRs couvertes :** FR77-FR79, FR89
**UX-DR :** UX-DR42-DR44 (logs, page d'attente, fichiers livrés)

### v0.2 — Complétude comptable

### Epic 9 : TVA Suisse
L'utilisateur peut configurer les taux TVA avec historique, appliquer les arrondis commerciaux et générer des rapports TVA par période.
**FRs couvertes :** FR54-FR56

### Epic 10 : Avoirs & Paiements
L'utilisateur peut créer des avoirs liés aux factures et générer des fichiers de paiement pain.001 conformes SIX.
**FRs couvertes :** FR36-FR37, FR39-FR41
**ARCH :** kesh-payment (crate publiable)
**UX-DR :** UX-DR35 (flux paiement en deux temps)

### Epic 11 : Budgets
L'utilisateur peut créer des budgets annuels par compte, gérer des versions et suivre les écarts budget vs réalisé.
**FRs couvertes :** FR57-FR59

### Epic 12 : Clôture d'Exercice
L'utilisateur peut clôturer un exercice comptable, reporter les soldes et saisir un bilan d'ouverture.
**FRs couvertes :** FR60-FR62

### Epic 13 : Justificatifs, Lettrage & Compléments v0.2
L'utilisateur peut attacher des justificatifs aux écritures, lettrer des écritures entre elles, personnaliser les modèles de documents et accéder aux manuels embarqués.
**FRs couvertes :** FR63-FR64, FR74, FR81, FR85-FR87

## Epic 1 : Fondations & Authentification

**Objectif :** L'administrateur peut installer Kesh, se connecter et gérer les utilisateurs. Le système d'authentification JWT avec RBAC est opérationnel. L'infrastructure technique est en place.

### Story 1.1 : Scaffold Cargo workspace & SvelteKit

**As a** développeur
**I want** un workspace Cargo structuré et un projet SvelteKit initialisé
**So that** je puisse commencer à développer sur des fondations propres

**Acceptance Criteria:**

- **Given** workspace vide, **When** cargo init pour chaque crate (kesh-core, kesh-db, kesh-api, kesh-reconciliation, kesh-i18n, kesh-report, kesh-seed, kesh-import, kesh-payment, kesh-qrbill), **Then** Cargo.toml racine avec [workspace] members compile sans erreur
- **Given** pas de frontend, **When** npx sv create frontend (Svelte 5, TypeScript, Playwright, adapter-static), **Then** SvelteKit démarre en mode dev avec ssr=false
- **Given** workspace complet, **When** cargo build --workspace, **Then** tous les crates compilent sans erreur
- **And** .gitignore, README.md, et structure de répertoires conforme à l'architecture
- **And** docker-compose.dev.yml avec MariaDB locale pour le développement (kesh-db testable dès cette story)

### Story 1.2 : Docker & configuration

**As a** administrateur
**I want** installer Kesh via docker-compose en une commande
**So that** l'application soit opérationnelle en moins de 15 minutes

**Acceptance Criteria:**

- **Given** docker-compose.dev.yml avec 2 containers (kesh + mariadb), **When** docker-compose up, **Then** les 2 containers démarrent et le healthcheck passe au vert
- **Given** .env.example documenté, **When** l'admin copie .env.example vers .env et configure KESH_ADMIN_PASSWORD et KESH_PORT, **Then** l'application démarre avec ces valeurs
- **Given** Dockerfile multi-stage (build Rust + build Svelte → image finale), **When** docker build, **Then** l'image résultante pèse moins de 100 Mo
- **Given** application démarrée, **When** GET /health, **Then** réponse 200 avec statut de connexion DB
- **And** les logs Docker affichent l'URL de l'application au démarrage

### Story 1.3 : Types core (newtypes & validation)

**As a** développeur
**I want** des types métier forts avec validation intégrée
**So that** l'intégrité des données soit garantie dès la couche logique

**Acceptance Criteria:**

- **Given** un montant, **When** création de Money(Decimal), **Then** le type utilise rust_decimal exclusivement, jamais de f64
- **Given** un IBAN, **When** création de Iban(String), **Then** le format et le checksum sont validés (retour d'erreur si invalide)
- **Given** un numéro IDE, **When** création de CheNumber(String), **Then** le format CHE-xxx.xxx.xxx et le checksum sont validés
- **Given** un QR-IBAN, **When** création de QrIban(String), **Then** le format QR-IBAN est validé (plage QR-IID 30000-31999)
- **And** chaque type a des tests unitaires couvrant les cas valides et invalides
- **And** documentation /// sur chaque type public

### Story 1.4 : Schéma de base & repository pattern

**As a** développeur
**I want** un schéma de base MariaDB et un pattern d'accès aux données
**So that** les stories suivantes puissent persister des données

**Acceptance Criteria:**

- **Given** kesh-db configuré, **When** sqlx migrate run, **Then** les tables users et companies sont créées
- **Given** table users, **Then** colonnes: id, username, password_hash, role, active, version, created_at, updated_at
- **Given** table companies, **Then** colonnes: id, name, address, ide_number, org_type, accounting_language, instance_language, version, created_at, updated_at
- **Given** pool MariaDB configuré, **When** connexion, **Then** le pool SQLx se connecte via la variable DATABASE_URL
- **And** repository pattern implémenté pour users et companies (create, find_by_id, update, list)
- **And** tests d'intégration avec base de données de test
- **And** schéma: table fiscal_years (id, company_id, name, start_date, end_date, status (open/closed), created_at, updated_at) — nécessaire dès les écritures pour le contrôle d'immutabilité post-clôture

### Story 1.5 : Authentification (login/logout/JWT)

**As a** utilisateur
**I want** me connecter avec un identifiant et un mot de passe
**So that** je puisse accéder à l'application de manière sécurisée

**Acceptance Criteria:**

- **Given** un utilisateur valide, **When** POST /api/v1/auth/login avec username et password, **Then** réponse 200 avec access_token JWT et refresh_token
- **Given** un access_token valide, **When** requête avec header Authorization: Bearer {token}, **Then** la requête est autorisée
- **Given** un access_token expiré, **When** requête API, **Then** réponse 401
- **Given** un utilisateur connecté, **When** POST /api/v1/auth/logout, **Then** le refresh_token est invalidé en base
- **And** les mots de passe sont hashés avec Argon2id, jamais stockés en clair
- **And** le JWT contient: user_id, role, exp (15 min)

### Story 1.6 : Refresh token & gestion de session

**As a** utilisateur
**I want** que ma session se renouvelle silencieusement
**So that** je ne sois pas déconnecté pendant que je travaille

**Acceptance Criteria:**

- **Given** un refresh_token valide, **When** POST /api/v1/auth/refresh, **Then** nouveau access_token + nouveau refresh_token
- **Given** inactivité de 15 minutes (configurable via env), **When** tentative de refresh, **Then** refresh_token expiré, réponse 401
- **Given** 5 tentatives de login échouées en 15 minutes pour une IP, **When** 6ème tentative, **Then** réponse 429 Too Many Requests
- **Given** IP bloquée, **When** attente de 30 minutes, **Then** les tentatives de login sont à nouveau autorisées
- **And** le refresh_token est un UUID opaque stocké en base (table refresh_tokens: id, user_id, token, expires_at)
- **And** un changement de mot de passe ou une désactivation de compte invalide tous les refresh_tokens de l'utilisateur

### Story 1.7 : Gestion des utilisateurs (CRUD)

**As a** administrateur
**I want** créer et gérer les comptes utilisateurs
**So that** chaque personne ait son propre accès avec le bon niveau de droits

**Acceptance Criteria:**

- **Given** rôle Admin, **When** POST /api/v1/users avec username, password, role, **Then** l'utilisateur est créé avec le rôle spécifié
- **Given** rôle Admin, **When** PUT /api/v1/users/:id/disable, **Then** le compte est désactivé et ses refresh_tokens invalidés
- **Given** un utilisateur, **When** PUT /api/v1/auth/password avec ancien et nouveau mot de passe, **Then** le mot de passe est mis à jour et les autres sessions invalidées
- **Given** rôle Admin, **When** PUT /api/v1/users/:id/reset-password, **Then** un nouveau mot de passe temporaire est défini
- **Given** politique de mot de passe configurée (longueur minimale via env), **When** création ou changement de mot de passe, **Then** la politique est appliquée
- **And** les rôles possibles sont: Admin, Comptable, Consultation
- **And** un utilisateur désactivé ne peut plus se connecter mais son historique d'actions reste intact

### Story 1.8 : RBAC & verrouillage optimiste

**As a** administrateur
**I want** que les accès soient contrôlés par rôle et que la concurrence soit gérée
**So that** les données soient protégées contre les accès non autorisés et les conflits

**Acceptance Criteria:**

- **Given** rôle Consultation, **When** tentative de POST/PUT/DELETE sur une ressource protégée, **Then** réponse 403 Forbidden
- **Given** rôle Comptable, **When** opérations CRUD sur les données comptables, **Then** autorisé
- **Given** rôle Comptable, **When** tentative de gestion des utilisateurs, **Then** réponse 403
- **Given** rôle Admin, **When** toute opération, **Then** autorisé (hérite Comptable + gestion users)
- **Given** une entité avec version=3, **When** PUT avec version=3, **Then** mise à jour réussie, version passe à 4
- **Given** une entité avec version=4, **When** PUT avec version=3 (stale), **Then** réponse 409 Conflict avec message explicite
- **And** le middleware RBAC est appliqué sur toutes les routes /api/v1/* (sauf auth)

### Story 1.9 : Design system & tokens

**As a** développeur frontend
**I want** un design system Kesh configuré avec les tokens visuels
**So that** tous les composants partagent une identité visuelle cohérente

**Critères d'acceptation :**

- **Given** Tailwind CSS + shadcn-svelte installés, **When** inspection des composants, **Then** les design tokens Kesh sont appliqués (palette: bleu ardoise #1e40af primaire, vert #16a34a succès, rouge #dc2626 erreur, ambre #d97706 attention)
- **Given** typographie, **When** inspection, **Then** Inter est la police unique, avec `font-variant-numeric: tabular-nums` sur les montants
- **Given** mode Guidé, **When** espacements, **Then** gap-4, p-6, my-8, hauteur ligne tableau 48px
- **Given** mode Expert, **When** espacements, **Then** gap-2, p-4, my-4, hauteur ligne tableau 36px
- **And** composants shadcn-svelte de base importés : Button, Input, Select, Table, Dialog, Toast, Tooltip, DropdownMenu

### Story 1.10 : Layout & page de login

**As a** utilisateur
**I want** une interface structurée avec une page de connexion
**So that** je puisse me connecter et naviguer dans l'application

**Critères d'acceptation :**

- **Given** le layout, **When** affichage, **Then** header fixe (logo, zone recherche, profil), sidebar fixe gauche (200-240px), zone contenu fluide, footer discret
- **Given** la page de login, **When** saisie username/password et soumission, **Then** appel API login, stockage JWT, redirection vers accueil
- **Given** largeur navigateur, **When** inférieure à 1280px, **Then** l'interface reste fonctionnelle (pas de responsive mobile, mais pas de cassure)
- **And** navigation clavier complète (focus visible outline bleu #3b82f6)
- **And** contraste WCAG AA sur tous les textes (minimum 4.5:1)

### Story 1.11 : Wrapper fetch & accessibilité

**As a** développeur frontend
**I want** un client API robuste et une base d'accessibilité
**So that** toutes les communications avec l'API soient fiables et l'interface accessible

**Critères d'acceptation :**

- **Given** un access_token expiré, **When** le wrapper fetch détecte 401, **Then** refresh automatique du token, si échec → redirection login
- **Given** une erreur API, **When** réponse 4xx/5xx, **Then** le wrapper parse l'erreur structurée et la rend disponible au composant
- **Given** une requête en cours, **When** loading, **Then** variable loading booléenne disponible pour afficher spinner/skeleton
- **And** interface fonctionnelle à 200% de zoom navigateur
- **And** raccourcis clavier : Ctrl+S sauvegarder, Tab/Shift+Tab navigation formulaires
- **And** axe-core configuré pour les tests d'accessibilité

## Epic 2 : Onboarding & Configuration

**Objectif :** L'utilisateur peut configurer son organisation, choisir sa langue, explorer la démo et accéder à une page d'accueil contextuelle.

### Story 2.1 : Internationalisation (i18n) backend

**As a** utilisateur
**I want** utiliser Kesh dans ma langue
**So that** l'interface soit dans la langue que je comprends

**Critères d'acceptation :**

- **Given** kesh-i18n configuré, **When** chargement des fichiers Fluent .ftl pour FR/DE/IT/EN, **Then** toutes les chaînes UI sont disponibles dans les 4 langues
- **Given** langue instance configurée via env (KESH_LANG), **When** démarrage, **Then** l'API expose les traductions dans la langue configurée
- **Given** un montant 1234.56, **When** formatage suisse, **Then** affichage "1'234.56" (apostrophe séparateur de milliers)
- **Given** une date 2026-04-03, **When** formatage suisse, **Then** affichage "03.04.2026" (dd.mm.yyyy)
- **And** les fichiers .ftl sont organisés par locale: fr-CH/, de-CH/, it-CH/, en-CH/
- **And** tests unitaires couvrant le formatage montants et dates pour les 4 langues

### Story 2.2 : Flux d'onboarding — Chemin A (Exploration)

**As a** utilisateur curieux
**I want** explorer Kesh avec des données de démo
**So that** je puisse comprendre l'application avant de l'utiliser en production

**Critères d'acceptation :**

- **Given** premier accès à Kesh, **When** affichage, **Then** écran de choix de langue (4 langues, noms dans leur langue, sans texte explicatif)
- **Given** langue choisie, **When** étape suivante, **Then** choix du mode d'utilisation (Guidé / Expert)
- **Given** mode choisi, **When** sélection "Explorer avec des données de démo", **Then** le script de seed charge des données réalistes et l'utilisateur accède à un Kesh fonctionnel
- **Given** mode démo actif, **When** navigation, **Then** bannière permanente jaune "Instance de démonstration — données fictives"
- **Given** mode démo, **When** clic "Réinitialiser pour la production", **Then** toutes les données de démo sont supprimées et l'onboarding Chemin B démarre
- **And** le script de seed (kesh-seed) passe par kesh-core pour respecter les validations métier
- **And** chaque étape est atomique et persistée immédiatement en base

### Story 2.3 : Flux d'onboarding — Chemin B (Production)

**As a** utilisateur
**I want** configurer mon organisation pour commencer à travailler
**So that** Kesh soit opérationnel avec mes données

**Critères d'acceptation :**

- **Given** onboarding Chemin B, **When** étape "Type d'organisation", **Then** choix entre Indépendant, Association, PME
- **Given** type choisi, **When** validation, **Then** le plan comptable et les journaux adaptés sont installés automatiquement (FR4, FR5)
- **Given** étape "Langue comptable", **When** choix FR/DE/IT/EN, **Then** la langue comptable est fixée au niveau instance (découplée de la langue interface)
- **Given** étape "Coordonnées", **When** saisie nom/raison sociale, adresse, IDE optionnel, **Then** données persistées, IDE validé si saisi (format CHE + checksum)
- **Given** étape "Compte bancaire", **When** saisie banque, IBAN, QR-IBAN, **Then** validation IBAN/QR-IBAN, données persistées ; bouton "Configurer plus tard" disponible
- **Given** onboarding interrompu, **When** retour dans l'application, **Then** bannière bleue "Configuration incomplète — Terminer la configuration"
- **And** Kesh fonctionne même avec onboarding partiel — fonctionnalités dépendantes affichent un message avec lien vers la configuration
- **And** chaque étape correspond à une section dans Administration (Profil, Paramètres > Organisation, etc.)
- **And** schéma: table bank_accounts (id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at)

### Story 2.4 : Page d'accueil & navigation

**As a** utilisateur
**I want** accéder rapidement aux fonctions principales depuis la page d'accueil
**So that** je puisse travailler efficacement

**Critères d'acceptation :**

- **Given** utilisateur connecté, **When** affichage page d'accueil, **Then** affichage des dernières écritures, factures ouvertes, soldes des comptes bancaires (FR82)
- **Given** sidebar, **When** affichage, **Then** navigation par activité organisée par fréquence — Quotidien: Facturer, Paiements, Import; Mensuel: Écritures, Réconciliation, Rapports; Rare: Paramètres
- **Given** vocabulaire de navigation, **When** labels, **Then** langage utilisateur ("Facturer" pas "Module Facturation", "Importer un relevé" pas "Import CAMT.053")
- **Given** menu Paramètres, **When** affichage, **Then** configuration centralisée avec sous-sections claires (un seul endroit, anti-pattern Bexio)
- **Given** recherche globale, **When** saisie dans le header, **Then** recherche accessible partout
- **And** disclaimer légal "ne remplace pas un fiduciaire" visible (FR7)
- **And** aucun cul-de-sac — toujours un chemin de retour, bouton navigateur fonctionnel

### Story 2.5 : Mode Guidé / Expert

**As a** utilisateur
**I want** choisir entre un mode guidé et un mode expert
**So that** l'interface s'adapte à mon niveau de compétence

**Critères d'acceptation :**

- **Given** profil utilisateur, **When** changement du mode (Guidé ↔ Expert), **Then** l'interface s'adapte immédiatement sans rechargement
- **Given** mode Guidé, **When** affichage, **Then** espacements généreux (gap-4, p-6), boutons plus grands avec labels explicites, confirmations avant actions, aide contextuelle visible
- **Given** mode Expert, **When** affichage, **Then** espacements compacts (gap-2, p-4), boutons avec icônes et labels en tooltip, raccourcis clavier actifs (Ctrl+N nouvelle écriture)
- **Given** liste vide en mode Guidé, **When** affichage, **Then** explication + suggestion d'ordre + bouton d'action
- **Given** liste vide en mode Expert, **When** affichage, **Then** bouton d'action uniquement
- **And** le mode est choisi à l'onboarding et modifiable à tout moment dans le profil

## Epic 3 : Plan Comptable & Écritures

**Objectif :** L'utilisateur peut gérer le plan comptable, saisir des écritures en partie double avec validation d'équilibre en temps réel, et rechercher les écritures.

### Story 3.1 : Plan comptable (chargement & gestion)

**As a** utilisateur
**I want** disposer d'un plan comptable suisse et le personnaliser
**So that** ma comptabilité soit structurée correctement

**Critères d'acceptation :**

- **Given** type d'organisation choisi à l'onboarding, **When** chargement plan comptable, **Then** le plan standard correspondant (PME, association, indépendant) est chargé depuis les fichiers JSON dans charts/
- **Given** plan comptable chargé, **When** affichage, **Then** arborescence des comptes avec numéro, nom, type (actif/passif/charge/produit)
- **Given** un compte, **When** ajout d'un nouveau compte, **Then** le compte est créé avec numéro, nom, type, et rattachement au plan
- **Given** un compte existant, **When** modification, **Then** le nom et les attributs modifiables sont mis à jour (verrouillage optimiste)
- **Given** un compte, **When** archivage, **Then** le compte n'apparaît plus dans les sélections mais reste visible dans les écritures existantes
- **And** les comptes sont stockés dans kesh-db avec repository pattern
- **And** schéma: table accounts (id, company_id, number, name, type, parent_id, active, version, created_at, updated_at)
- **And** les fichiers JSON des plans comptables suisses (charts/pme.json, charts/association.json, charts/independant.json) sont créés dans cette story, basés sur les plans comptables standards suisses

### Story 3.2 : Saisie d'écritures en partie double

**As a** utilisateur
**I want** saisir des écritures comptables avec validation instantanée
**So that** ma comptabilité soit toujours équilibrée

**Critères d'acceptation :**

- **Given** formulaire d'écriture, **When** saisie, **Then** champs: date, journal (Achats/Ventes/Banque/Caisse/OD), libellé, lignes débit/crédit avec compte et montant
- **Given** champ compte, **When** saisie "1020" ou "Banque", **Then** autocomplétion par numéro ou nom du compte (UX-DR36)
- **Given** lignes saisies, **When** calcul en temps réel, **Then** indicateur d'équilibre affiché — vert si débit=crédit, rouge si déséquilibré (UX-DR10)
- **Given** écriture déséquilibrée (débit ≠ crédit), **When** tentative de validation, **Then** refus avec message explicite: "Écriture déséquilibrée — le total des débits (X) ne correspond pas au total des crédits (Y)" (FR21)
- **Given** écriture équilibrée, **When** validation, **Then** écriture persistée avec numéro séquentiel, toutes les lignes stockées
- **And** navigation Tab gauche→droite entre les champs
- **And** montants en rust_decimal, jamais de f64
- **And** schéma: tables journal_entries (id, company_id, entry_number, date, journal, description, version, created_at, updated_at) et journal_entry_lines (id, entry_id, account_id, debit, credit)

### Story 3.3 : Modification & suppression d'écritures

**As a** utilisateur
**I want** modifier ou supprimer mes écritures tant que l'exercice est ouvert
**So that** je puisse corriger mes erreurs

**Critères d'acceptation :**

- **Given** une écriture dans un exercice ouvert, **When** modification, **Then** les champs sont éditables, le verrouillage optimiste s'applique (version)
- **Given** une écriture modifiée, **When** validation, **Then** la balance débit/crédit est re-vérifiée, l'écriture est mise à jour
- **Given** une écriture dans un exercice ouvert, **When** suppression, **Then** l'écriture est supprimée après confirmation
- **Given** une écriture dans un exercice clôturé, **When** tentative de modification ou suppression, **Then** refus avec message: "Impossible de modifier une écriture d'un exercice clôturé" (FR24)
- **And** le journal d'audit enregistre chaque modification et suppression (FR88)

### Story 3.4 : Recherche, pagination & tri

**As a** utilisateur
**I want** retrouver facilement mes écritures
**So that** je puisse naviguer efficacement dans ma comptabilité

**Critères d'acceptation :**

- **Given** liste d'écritures, **When** recherche par montant, **Then** les écritures correspondantes sont affichées
- **Given** liste d'écritures, **When** recherche par libellé, **Then** correspondance partielle (LIKE)
- **Given** liste d'écritures, **When** recherche par numéro de facture, **Then** correspondance exacte
- **Given** liste d'écritures, **When** recherche par plage de dates, **Then** filtrage par date début/fin
- **Given** une liste, **When** clic sur un en-tête de colonne, **Then** tri ascendant/descendant
- **Given** une liste longue, **When** pagination, **Then** affichage par pages avec offset/limit, total affiché
- **And** le pattern de recherche/tri/pagination est réutilisable sur toutes les listes (contacts, factures, etc.)
- **And** réponse API format: { "items": [...], "total": N, "offset": 0, "limit": 50 }

### Story 3.5 : Notifications, aide contextuelle & audit

**As a** utilisateur
**I want** être informé clairement de chaque action et comprendre les termes comptables
**So that** je puisse utiliser l'application en confiance

**Critères d'acceptation :**

- **Given** action réussie (saisie, sauvegarde), **When** feedback, **Then** notification banner verte avec message de confirmation (FR71)
- **Given** avertissement (import partiel, doublon détecté), **When** feedback, **Then** notification banner orange avec détails
- **Given** erreur non-bloquante, **When** feedback, **Then** notification banner rouge avec message explicite et action suggérée
- **Given** erreur bloquante (session expirée, conflit de version), **When** feedback, **Then** modal avec explication et actions possibles (FR72)
- **Given** un terme comptable (débit, crédit, bilan), **When** survol ou focus, **Then** tooltip bilingue: terme en langage naturel + terminologie comptable (FR73, UX-DR39)
- **Given** une action sur une donnée comptable, **When** création/modification/suppression, **Then** le journal d'audit enregistre: user_id, action, entité, horodatage (FR88)
- **And** schéma: table audit_log (id, user_id, action, entity_type, entity_id, details_json, created_at)

## Epic 4 : Carnet d'Adresses & Catalogue

**Objectif :** L'utilisateur peut gérer ses clients et fournisseurs avec validation IDE, et maintenir un catalogue de produits/services pour la facturation.

### Story 4.1 : Carnet d'adresses (CRUD contacts)

**As a** utilisateur
**I want** gérer mes clients et fournisseurs dans un carnet d'adresses unifié
**So that** je puisse les utiliser pour la facturation et les paiements

**Critères d'acceptation :**

- **Given** carnet d'adresses, **When** création d'un contact, **Then** saisie: nom/raison sociale, adresse, email, téléphone, type (personne/entreprise)
- **Given** un contact, **When** marquage client/fournisseur, **Then** le contact peut être flagué client, fournisseur, ou les deux (FR26)
- **Given** champ IDE, **When** saisie d'un numéro IDE, **Then** validation format CHE-xxx.xxx.xxx et checksum (champ optionnel) (FR27)
- **Given** un contact, **When** modification, **Then** verrouillage optimiste appliqué (version)
- **Given** liste de contacts, **When** affichage, **Then** recherche, tri, pagination
- **And** schéma: table contacts (id, company_id, name, contact_type, is_client, is_supplier, address, email, phone, ide_number, default_payment_terms, version, created_at, updated_at)
- **And** tests unitaires sur la validation IDE (CHE checksum)

### Story 4.2 : Conditions de paiement & catalogue produits/services

**As a** utilisateur
**I want** associer des conditions de paiement à mes contacts et gérer un catalogue
**So that** la création de factures soit rapide et cohérente

**Critères d'acceptation :**

- **Given** un contact, **When** association de conditions de paiement par défaut, **Then** les conditions sont utilisées automatiquement lors de la facturation (FR28)
- **Given** catalogue, **When** création d'un produit/service, **Then** saisie: nom, description, prix unitaire, taux TVA applicable (FR29)
- **Given** catalogue, **When** modification/archivage d'un article, **Then** verrouillage optimiste appliqué
- **Given** création de facture, **When** sélection d'articles du catalogue, **Then** les lignes sont pré-remplies avec nom, prix, TVA (FR30)
- **And** schéma: table products (id, company_id, name, description, unit_price, vat_rate, active, version, created_at, updated_at)
- **And** les conditions de paiement sont un champ texte libre sur le contact (ex: "30 jours net", "10 jours 2% escompte")

## Epic 5 : Facturation QR Bill

**Objectif :** L'utilisateur peut créer des factures avec lignes libres ou depuis le catalogue, valider et générer des PDF QR Bill conformes SIX 2.2.

### Story 5.1 : Création de factures (brouillon)

**As a** utilisateur
**I want** créer une facture avec des lignes de détail
**So that** je puisse facturer mes clients

**Critères d'acceptation :**

- **Given** formulaire facture, **When** création, **Then** sélection du client (depuis carnet d'adresses), date, conditions de paiement (pré-remplies depuis le contact)
- **Given** facture, **When** ajout de ligne libre, **Then** saisie: description, quantité, prix unitaire, taux TVA ; montant calculé automatiquement (FR31)
- **Given** facture, **When** ajout depuis catalogue, **Then** sélection d'un article → ligne pré-remplie, quantité modifiable (FR31)
- **Given** facture en brouillon, **When** suppression, **Then** la facture est supprimée définitivement (FR32)
- **Given** facture en brouillon, **When** modification, **Then** toutes les lignes sont éditables
- **And** schéma: tables invoices (id, company_id, contact_id, invoice_number, status (draft/validated/cancelled), date, due_date, payment_terms, total_amount, version, created_at, updated_at) et invoice_lines (id, invoice_id, description, quantity, unit_price, vat_rate, line_total)
- **And** le total facture est calculé comme la somme des lignes

### Story 5.2 : Validation & numérotation des factures

**As a** utilisateur
**I want** valider une facture pour qu'elle reçoive un numéro définitif
**So that** la facture soit officielle et comptabilisée

**Critères d'acceptation :**

- **Given** facture en brouillon, **When** validation, **Then** un numéro séquentiel définitif est attribué (FR33)
- **Given** facture validée, **When** tentative de modification, **Then** refus — une facture validée est immuable (seul un avoir peut l'annuler)
- **Given** facture validée, **When** vérification, **Then** l'écriture comptable correspondante est générée automatiquement (débit client, crédit produit/service)
- **Given** configuration, **When** paramétrage du format de numérotation, **Then** format configurable (ex: "F-2026-0001", "FACT-001") (FR35)
- **And** le compteur de numérotation est par exercice, sans trou dans la séquence

### Story 5.3 : Génération PDF QR Bill

**As a** utilisateur
**I want** générer un PDF QR Bill conforme SIX pour chaque facture validée
**So that** je puisse l'envoyer à mon client

**Critères d'acceptation :**

- **Given** facture validée, **When** génération PDF, **Then** le PDF contient la facture et la partie paiement QR Bill conforme SIX 2.2 (FR34)
- **Given** QR Bill, **When** validation conformité, **Then** dimensions 46×46mm, position A4, police conforme, croix suisse, QR Code scannable
- **Given** QR Bill, **When** contenu QR Code, **Then** les données de paiement sont conformes (IBAN/QR-IBAN, montant, référence, débiteur)
- **Given** PDF généré, **When** affichage, **Then** le PDF s'ouvre dans un nouvel onglet du navigateur (FR38)
- **Given** PDF, **When** génération, **Then** temps < 3 secondes
- **And** kesh-qrbill est une crate publiable indépendante (zéro dépendance sur kesh-core)
- **And** les montants dans le PDF utilisent le format suisse (apostrophe séparateur: 1'234.56)
- **And** tests unitaires avec les fichiers de test SIX officiels (docs/six-references/)

## Epic 6 : Import Bancaire & Réconciliation

**Objectif :** L'utilisateur peut importer ses relevés bancaires et réconcilier les transactions avec matching automatique et règles d'affectation.

### Story 6.1 : Import CAMT.053

**As a** utilisateur
**I want** importer mes relevés bancaires au format CAMT.053
**So that** les transactions apparaissent dans Kesh

**Critères d'acceptation :**

- **Given** fichier CAMT.053 valide, **When** import, **Then** toutes les transactions sont extraites avec date, montant, référence, détails
- **Given** fichier CAMT.053 avec sous-transactions (TxDtls), **When** import, **Then** les sous-transactions sont extraites individuellement (FR49)
- **Given** import, **When** lié à un compte bancaire, **Then** les transactions sont associées au bon compte (FR50)
- **Given** fichier, **When** glisser-déposer ou sélection, **Then** prévisualisation des transactions avant confirmation d'import
- **Given** parseur, **When** version du format détectée, **Then** le parseur sélectionne la version correspondante (multi-version)
- **And** kesh-import est une crate publiable indépendante (zéro dépendance sur kesh-core)
- **And** types autonomes dans kesh-import, conversion via From/Into vers kesh-core
- **And** tests d'intégration avec les fichiers de test SIX officiels (docs/six-references/)
- **And** schéma: tables bank_imports (id, company_id, bank_account_id, file_hash, filename, imported_at) et bank_transactions (id, import_id, bank_account_id, date, amount, reference, details, status (pending/reconciled), matched_entry_id)

### Story 6.2 : Import CSV (multi-encodage & profils banque)

**As a** utilisateur
**I want** importer des relevés CSV de différentes banques
**So that** je puisse gérer toutes mes banques dans Kesh

**Critères d'acceptation :**

- **Given** fichier CSV UTF-8, **When** import, **Then** les transactions sont correctement parsées (FR42)
- **Given** fichier CSV ISO-8859-1, **When** import, **Then** détection automatique de l'encodage et parsage correct (FR52)
- **Given** format CSV inconnu, **When** import, **Then** l'utilisateur peut configurer un profil de format par banque (mapping colonnes) (FR53)
- **Given** profil banque configuré, **When** import suivant de la même banque, **Then** le profil est appliqué automatiquement
- **Given** rejet de lignes, **When** format de date non reconnu, **Then** listing détaillé des erreurs avec numéros de ligne (FR51)
- **And** schéma: table bank_profiles (id, company_id, bank_name, column_mapping_json, date_format, encoding, created_at)

### Story 6.3 : Détection de doublons & rejet partiel

**As a** utilisateur
**I want** que Kesh détecte les doublons et gère les imports partiels
**So that** aucune transaction ne soit comptée deux fois

**Critères d'acceptation :**

- **Given** fichier déjà importé (même hash), **When** tentative de réimport, **Then** avertissement "fichier déjà importé" avec option de forcer (FR43)
- **Given** transactions qui chevauchent un import précédent, **When** import, **Then** les doublons sont détectés et signalés — aucune transaction en double
- **Given** fichier avec erreurs partielles, **When** import, **Then** les transactions valides sont importées, les invalides sont rejetées avec listing détaillé (FR51)
- **And** détection de doublons par combinaison: date + montant + référence + compte bancaire

### Story 6.4 : Réconciliation & matching automatique

**As a** utilisateur
**I want** que les transactions connues soient automatiquement proposées pour réconciliation
**So that** le travail de réconciliation soit minimal

**Critères d'acceptation :**

- **Given** transactions importées, **When** réconciliation, **Then** le système propose automatiquement des contreparties pour les transactions connues (factures en attente de paiement) (FR44)
- **Given** proposition de matching, **When** affichage, **Then** tableau avec transaction bancaire ↔ écriture comptable proposée, score de confiance
- **Given** propositions, **When** validation en lot, **Then** toutes les propositions acceptées sont réconciliées en une action
- **Given** transaction réconciliée, **When** vérification, **Then** l'écriture comptable correspondante est créée ou liée
- **And** le matching considère: montant exact, référence facture, nom client/fournisseur
- **And** mutex par compte bancaire pour éviter les imports concurrents (kesh-reconciliation)

### Story 6.5 : Réconciliation manuelle & règles d'affectation

**As a** utilisateur
**I want** réconcilier manuellement les transactions et créer des règles pour l'avenir
**So that** les prochains imports soient plus automatisés

**Critères d'acceptation :**

- **Given** transaction sans proposition, **When** création manuelle de contrepartie, **Then** l'utilisateur sélectionne le compte comptable et crée l'écriture (FR45)
- **Given** affectation manuelle effectuée, **When** feedback, **Then** le système propose de créer une règle d'affectation automatique (FR46)
- **Given** règles d'affectation, **When** gestion, **Then** CRUD des règles (description contient "X" → compte Y) (FR47)
- **Given** transaction agrégée (plusieurs sous-montants), **When** éclatement, **Then** l'utilisateur peut diviser en sous-lignes avec comptes différents (FR48)
- **And** les règles sont appliquées par priorité lors des imports suivants
- **And** schéma: table import_rules (id, company_id, match_pattern, account_id, priority, active, created_at)

## Epic 7 : Rapports & Exports

**Objectif :** L'utilisateur peut générer bilan, compte de résultat, balance des comptes et journaux en PDF et CSV avec formats suisses.

### Story 7.1 : Rapports comptables (bilan, résultat, balance, journaux)

**As a** utilisateur
**I want** générer mes rapports comptables
**So that** je puisse vérifier ma situation financière et préparer ma clôture

**Critères d'acceptation :**

- **Given** données comptables, **When** génération du bilan, **Then** actifs et passifs affichés par classe de compte, totaux calculés, balance vérifiée (FR65)
- **Given** données comptables, **When** génération du compte de résultat, **Then** charges et produits affichés, résultat net calculé
- **Given** données comptables, **When** génération de la balance des comptes, **Then** tous les comptes avec soldes débit/crédit affichés
- **Given** données comptables, **When** génération des journaux, **Then** écritures listées par journal (Achats, Ventes, Banque, Caisse, OD) avec totaux
- **Given** un rapport, **When** filtrage par période, **Then** seules les écritures de la période sont incluses
- **And** kesh-report génère les données, kesh-i18n formate les montants et dates

### Story 7.2 : Export PDF & CSV

**As a** utilisateur
**I want** exporter mes rapports en PDF et CSV
**So that** je puisse les partager ou les archiver

**Critères d'acceptation :**

- **Given** un rapport généré, **When** export PDF, **Then** le PDF respecte les formats suisses: apostrophe séparateur de milliers (1'234.56), dates dd.mm.yyyy (FR67)
- **Given** un rapport, **When** export CSV, **Then** fichier CSV avec séparateur point-virgule, encodage UTF-8
- **Given** menu export, **When** export global par table, **Then** l'utilisateur peut exporter l'ensemble des données (comptes, écritures, contacts, factures) en CSV (FR68)
- **Given** export global, **When** génération, **Then** ZIP contenant un CSV par table + métadonnées
- **Given** export, **When** bouton dans le menu principal, **Then** accessible directement (pas caché dans les paramètres) — souveraineté des données
- **And** génération PDF < 3 secondes
- **And** les messages d'erreur disent ce qui s'est passé ET ce que l'utilisateur peut faire (UX-DR38)

## Epic 8 : Déploiement & Opérations

**Objectif :** L'administrateur peut déployer Kesh via docker-compose avec migrations automatiques et le frontend reste accessible même si la DB est indisponible.

### Story 8.1 : Docker-compose & Dockerfile production

**As a** administrateur
**I want** déployer Kesh en production avec une seule commande
**So that** l'application soit opérationnelle rapidement

**Critères d'acceptation :**

- **Given** docker-compose.yml avec 2 containers (kesh + mariadb), **When** docker-compose up -d, **Then** les deux containers démarrent et communiquent
- **Given** Dockerfile multi-stage (build Rust + build Svelte → image finale Alpine), **When** docker build, **Then** l'image pèse moins de 100 Mo
- **Given** application en production, **When** Axum sert le frontend via tower-http::ServeDir + API /api/v1/*, **Then** tout est servi par un seul point d'entrée HTTP
- **Given** nginx/Traefik/Caddy en amont (optionnel), **When** reverse proxy TLS configuré, **Then** Kesh fonctionne en HTTP pur derrière le proxy
- **And** les logs sont émis sur stdout/stderr (standard Docker)
- **And** l'image Docker expose le port configurable via KESH_PORT
- **And** cette story concerne le déploiement production — distinct du docker-compose.dev.yml créé en Story 1.2 pour le développement

### Story 8.2 : Migrations automatiques & détection de version

**As a** administrateur
**I want** que les migrations s'appliquent automatiquement au démarrage
**So that** les mises à jour soient simples et sûres

**Critères d'acceptation :**

- **Given** nouvelle version de Kesh, **When** démarrage du container, **Then** le système détecte la nouvelle version et affiche "Backup recommandé avant de continuer" (FR78)
- **Given** migrations pendantes, **When** démarrage, **Then** sqlx migrate run s'exécute automatiquement (FR79)
- **Given** migration échouée, **When** erreur, **Then** le système log l'erreur clairement et ne démarre pas (pas de données corrompues)
- **Given** migrations réussies, **When** vérification, **Then** les données des exercices passés restent intactes et lisibles
- **And** les fichiers de migration sont versionnés dans crates/kesh-db/migrations/

### Story 8.3 : Résilience frontend (DB inaccessible)

**As a** utilisateur
**I want** voir une page d'erreur claire si la base de données est indisponible
**So that** je comprenne le problème sans paniquer

**Critères d'acceptation :**

- **Given** Axum démarré mais MariaDB indisponible, **When** accès à l'application, **Then** le frontend SPA est servi normalement (fichiers statiques) (FR89)
- **Given** frontend chargé sans DB, **When** appel API, **Then** message d'erreur explicite invitant à vérifier l'état du serveur
- **Given** page d'erreur frontend, **When** affichage, **Then** page d'attente élégante avec message clair, pas d'erreur technique brute (UX-DR43)
- **Given** DB redevenue accessible, **When** refresh, **Then** l'application fonctionne normalement sans redémarrage
- **And** le healthcheck /health retourne 503 quand la DB est inaccessible, 200 quand tout est OK

### Story 8.4 : CI/CD GitHub Actions

**As a** développeur
**I want** une pipeline CI/CD automatisée
**So that** chaque changement soit validé automatiquement avant merge

**Critères d'acceptation :**

- **Given** push ou pull request sur le repo, **When** GitHub Actions se déclenche, **Then** le workflow CI exécute : cargo build, cargo test, cargo clippy, cargo fmt --check
- **Given** workflow CI, **When** tests frontend, **Then** npm run check, npm run test (Vitest), npm run build exécutés
- **Given** merge sur main, **When** workflow release, **Then** l'image Docker est buildée et publiée
- **Given** échec de CI, **When** notification, **Then** le développeur voit clairement quel check a échoué
- **And** fichiers dans .github/workflows/ : ci.yml (build + tests) et release.yml (Docker image)
- **And** axe-core intégré dans la CI pour détecter les régressions d'accessibilité (UX-DR18)

### Story 8.5 : Guide d'installation PDF

**As a** administrateur
**I want** un guide d'installation clair et concis
**So that** je puisse installer Kesh sans aide externe

**Critères d'acceptation :**

- **Given** guide-installation.pdf, **When** lecture, **Then** contenu de 4-5 pages couvrant : prérequis (Docker, docker-compose), installation en 5 étapes, configuration du .env, vérification (URL, healthcheck, premier login)
- **Given** section problèmes fréquents, **When** consultation, **Then** les cas courants sont documentés : port occupé, container qui ne démarre pas, DB inaccessible
- **Given** section NAS, **When** consultation, **Then** instructions spécifiques pour Synology et QNAP
- **Given** section backup, **When** consultation, **Then** responsabilisation claire : dump MariaDB + volume fichiers, pas de fausse promesse
- **And** le guide est livré avec docker-compose.yml et .env.example (UX-DR44)
- **And** le guide est dans la langue de l'instance

## Epic 9 : TVA Suisse

**Objectif :** L'utilisateur peut configurer les taux TVA avec historique, appliquer les arrondis commerciaux et générer des rapports TVA par période.

### Story 9.1 : Configuration des taux TVA

**As a** administrateur
**I want** configurer les taux de TVA avec des dates de validité
**So that** les changements de taux soient gérés correctement dans le temps

**Critères d'acceptation :**

- **Given** configuration TVA, **When** création d'un taux, **Then** saisie: libellé, pourcentage, date début validité, date fin validité (optionnel) (FR54)
- **Given** taux TVA avec historique, **When** saisie d'une écriture ou facture, **Then** le taux applicable à la date de l'opération est automatiquement sélectionné
- **Given** changement de taux (ex: 7.7% → 8.1%), **When** nouveau taux configuré, **Then** l'ancien taux reste applicable pour les opérations antérieures à la date de bascule
- **Given** taux TVA, **When** affichage, **Then** liste complète avec historique des taux (actifs et expirés)
- **And** schéma: table vat_rates (id, company_id, label, rate, valid_from, valid_to, active, created_at)
- **And** les taux standard suisses sont pré-configurés à l'onboarding (normal, réduit, spécial, exonéré)

### Story 9.2 : Calcul TVA & rapport par période

**As a** utilisateur
**I want** que la TVA soit calculée correctement et pouvoir générer un rapport
**So that** je puisse remplir mon décompte AFC

**Critères d'acceptation :**

- **Given** une ligne de facture ou écriture avec TVA, **When** calcul, **Then** arrondi commercial au centime par ligne (FR55)
- **Given** arrondi, **When** montant 123.455, **Then** résultat 123.46 (arrondi commercial: .5 → vers le haut)
- **Given** données comptables, **When** génération rapport TVA trimestriel ou semestriel, **Then** détail par taux: chiffre d'affaires, TVA due, TVA récupérable, solde (FR56)
- **Given** rapport TVA, **When** vérification, **Then** les montants correspondent aux écritures comptables (vérifiable manuellement)
- **And** tous les calculs TVA utilisent rust_decimal (jamais de f64)
- **And** export du rapport en PDF et CSV

## Epic 10 : Avoirs & Paiements

**Objectif :** L'utilisateur peut créer des avoirs liés aux factures et générer des fichiers de paiement pain.001 conformes SIX.

### Story 10.1 : Avoirs (notes de crédit)

**As a** utilisateur
**I want** annuler une facture validée par la création d'un avoir
**So that** la comptabilité reste intègre sans supprimer de documents

**Critères d'acceptation :**

- **Given** facture validée, **When** création d'un avoir, **Then** l'avoir est lié à la facture d'origine avec référence (FR36)
- **Given** avoir créé, **When** validation, **Then** numéro séquentiel attribué depuis une séquence de numérotation séparée (ex: AV-2026-0001) (FR37)
- **Given** avoir validé, **When** comptabilisation, **Then** l'écriture de contre-passation est générée automatiquement (débit produit, crédit client)
- **Given** avoir validé, **When** vérification du solde client, **Then** le solde revient à zéro (facture + avoir = 0)
- **And** un avoir génère un PDF similaire à la facture, avec mention "Avoir" et référence à la facture d'origine

### Story 10.2 : Génération de fichiers de paiement pain.001

**As a** utilisateur
**I want** générer un fichier de paiement pour mes factures fournisseurs
**So that** je puisse l'uploader dans mon e-banking

**Critères d'acceptation :**

- **Given** factures fournisseurs ouvertes, **When** sélection pour paiement, **Then** l'utilisateur peut sélectionner une ou plusieurs factures à payer (FR39)
- **Given** sélection effectuée, **When** vérification, **Then** affichage récapitulatif: fournisseur, montant, IBAN pour chaque paiement
- **Given** IBAN invalide pour un fournisseur, **When** validation, **Then** le système signale l'erreur et exclut ce paiement du lot (FR41)
- **Given** validation OK, **When** génération, **Then** fichier pain.001.001.09.ch.03 conforme au schéma XSD SIX (FR40)
- **And** kesh-payment est une crate publiable indépendante (zéro dépendance sur kesh-core)
- **And** tests de validation contre le schéma XSD officiel SIX (docs/six-references/)

### Story 10.3 : Flux de paiement en deux temps

**As a** utilisateur
**I want** préparer mes paiements au fil de l'eau et les envoyer en lot
**So that** je puisse séparer la décision individuelle de l'action groupée

**Critères d'acceptation :**

- **Given** facture fournisseur reçue, **When** création d'un paiement, **Then** le paiement est ajouté à la liste d'attente (Temps 1)
- **Given** jour de paiement, **When** sélection des paiements en attente, **Then** l'utilisateur sélectionne, vérifie et génère le fichier pain.001 (Temps 2)
- **Given** le flux, **When** exécution, **Then** le pattern "Sélectionner → Vérifier → Valider → Résultat" est respecté (UX-DR34)
- **And** le fichier généré est disponible pour téléchargement et upload dans l'e-banking (UX-DR35)

## Epic 11 : Budgets

**Objectif :** L'utilisateur peut créer des budgets annuels par compte, gérer des versions et suivre les écarts budget vs réalisé.

### Story 11.1 : Budgets annuels & suivi

**As a** utilisateur
**I want** créer un budget et suivre les écarts avec le réalisé
**So that** je puisse piloter mes finances

**Critères d'acceptation :**

- **Given** exercice comptable, **When** création de budget, **Then** saisie de montants mensuels par compte individuel (FR57)
- **Given** budget existant, **When** création d'une nouvelle version, **Then** le budget peut avoir plusieurs versions par exercice (initial, révisé) (FR58)
- **Given** budget et données comptables, **When** génération rapport comparatif, **Then** affichage par compte: budget mensuel, réalisé mensuel, écart en montant et pourcentage (FR59)
- **Given** rapport budget vs réalisé, **When** filtrage, **Then** par période (mois, trimestre, année) et par compte
- **And** schéma: tables budgets (id, company_id, fiscal_year, version_name, created_at) et budget_lines (id, budget_id, account_id, month, amount)
- **And** export du rapport en PDF et CSV

## Epic 12 : Clôture d'Exercice

**Objectif :** L'utilisateur peut clôturer un exercice comptable, reporter les soldes et saisir un bilan d'ouverture.

### Story 12.1 : Clôture & report des soldes

**As a** utilisateur
**I want** clôturer mon exercice et démarrer le suivant
**So that** ma comptabilité soit propre d'année en année

**Critères d'acceptation :**

- **Given** exercice ouvert, **When** clôture, **Then** confirmation requise (action irréversible), toutes les écritures de l'exercice deviennent immuables (FR60)
- **Given** exercice clôturé, **When** report des soldes, **Then** les soldes des comptes de bilan sont automatiquement reportés dans le nouvel exercice (FR61)
- **Given** exercice clôturé, **When** tentative de modification d'écriture, **Then** refus avec message explicite (lié à FR24)
- **Given** nouvel exercice, **When** saisie du bilan d'ouverture, **Then** l'utilisateur peut saisir ou ajuster les soldes de départ (FR62)
- **Given** clôture, **When** journal d'audit, **Then** l'action est enregistrée avec utilisateur et horodatage
- **And** la clôture est une opération privilégiée (rôle Admin ou Comptable)
- **And** un exercice clôturé ne peut pas être ré-ouvert — seules des contre-passations dans le nouvel exercice sont possibles

## Epic 13 : Justificatifs, Lettrage & Compléments v0.2

**Objectif :** L'utilisateur peut attacher des justificatifs aux écritures, lettrer des écritures entre elles, personnaliser les modèles de documents et accéder aux manuels embarqués.

### Story 13.1 : Pièces justificatives

**As a** utilisateur
**I want** attacher des fichiers à mes écritures
**So that** je puisse conserver les justificatifs avec les données comptables

**Critères d'acceptation :**

- **Given** une écriture, **When** ajout d'un fichier (PDF, image JPG/PNG), **Then** le fichier est uploadé et lié à l'écriture (FR63)
- **Given** fichier uploadé, **When** stockage, **Then** le fichier est stocké dans un volume Docker dédié, séparé de la base de données (FR64)
- **Given** écriture avec justificatif, **When** consultation, **Then** le fichier est téléchargeable/visualisable depuis la vue de l'écriture
- **Given** fichier volumineux, **When** upload, **Then** taille maximale configurée (ex: 10 Mo par fichier)
- **And** les justificatifs sont inclus dans l'export global (ZIP)
- **And** la stratégie de backup doit couvrir le volume de fichiers en plus du dump MariaDB

### Story 13.2 : Lettrage

**As a** utilisateur
**I want** lettrer des écritures entre elles
**So that** je puisse marquer les correspondances facture ↔ paiement

**Critères d'acceptation :**

- **Given** deux ou plusieurs écritures, **When** lettrage, **Then** les écritures sont marquées avec un code de lettrage commun (FR85)
- **Given** écritures lettrées, **When** affichage, **Then** le code de lettrage est visible et les écritures liées sont identifiables
- **Given** écritures lettrées dans un exercice ouvert, **When** délettrage, **Then** le lien de lettrage est supprimé (FR86)
- **Given** exercice clôturé, **When** tentative de délettrage, **Then** refus (cohérent avec l'immutabilité post-clôture)
- **And** le lettrage aide à identifier les factures non payées et les paiements non affectés
- **And** schéma: colonne lettering_code sur journal_entry_lines (nullable, même code = même lettrage)

### Story 13.3 : Versioning parseurs, modèles & manuels

**As a** utilisateur
**I want** que Kesh gère les versions des formats SIX et me fournisse de la documentation
**So that** l'application reste compatible et je puisse me former

**Critères d'acceptation :**

- **Given** import CAMT.053 ou génération pain.001, **When** détection du format, **Then** le système identifie la version et sélectionne le parseur/générateur correspondant (FR87)
- **Given** modèles de documents (factures, rapports), **When** personnalisation, **Then** l'utilisateur peut modifier: logo, coordonnées, textes de pied de page (FR81)
- **Given** application, **When** accès à l'aide, **Then** 3 manuels PDF embarqués accessibles: guide de démarrage, manuel utilisateur, manuel administrateur (FR74)
- **And** les manuels sont dans la langue de l'instance
- **And** les modèles personnalisés sont stockés en base et appliqués à tous les PDF générés
