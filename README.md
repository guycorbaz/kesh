# Kesh

[![CI](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/guycorbaz/kesh/actions/workflows/ci.yml)
[![Release](https://github.com/guycorbaz/kesh/actions/workflows/release.yml/badge.svg)](https://github.com/guycorbaz/kesh/actions/workflows/release.yml)
[![Docker Pulls](https://img.shields.io/docker/pulls/gcorbaz/kesh)](https://hub.docker.com/r/gcorbaz/kesh)
[![Docker Image Version](https://img.shields.io/docker/v/gcorbaz/kesh?sort=semver&label=docker)](https://hub.docker.com/r/gcorbaz/kesh/tags)
[![Docker Image Size](https://img.shields.io/docker/image-size/gcorbaz/kesh?sort=semver)](https://hub.docker.com/r/gcorbaz/kesh/tags)
[![License: EUPL 1.2](https://img.shields.io/badge/license-EUPL--1.2-blue.svg)](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12)
[![Rust](https://img.shields.io/badge/rust-1.96-orange.svg)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/svelte-5-ff3e00.svg)](https://svelte.dev/)

**Kesh** est un logiciel de comptabilité et de gestion pour indépendants, TPE et associations en Suisse. Gratuit, open source, auto-hébergé.

## Table des matières

- [Fonctionnalités](#fonctionnalités)
- [Pile technique](#pile-technique)
- [Démarrage rapide](#démarrage-rapide)
- [Structure du projet](#structure-du-projet)
- [Documentation](#documentation)
- [Développement](#développement)
- [Tests](#tests)
- [Feuille de route](#feuille-de-route)
- [Contribuer](#contribuer)
- [Licence](#licence)

## Fonctionnalités

- **Comptabilité en partie double** — plan comptable suisse, écritures validées, audit log
- **Carnet d'adresses & catalogue produits** — contacts, conditions de paiement, TVA
- **Facturation QR Bill 2.2** — génération PDF conforme au standard suisse, **adresses structurées (type S)** conformes à l'exigence SIX en vigueur (rue / n° / NPA / localité séparés) ✓
- **Ventilation du chiffre d'affaires par ligne de facture** — chaque ligne peut porter son propre **compte de produit** (honoraires, marchandises, prestations…), sélectionnable dans le formulaire ; une ligne laissée vide suit le compte par défaut de la société, affiché explicitement. L'écriture comptable ventile le crédit sur les bons comptes dès la validation, avoirs compris — compte de résultat exploitable sans reclassement manuel ✓
- **Compte de produit sur la fiche article** — chaque article du catalogue porte **son** compte de produit, choisi une fois plutôt que ressaisi à chaque facture ; la ligne montée depuis le catalogue le **recopie** automatiquement, puis reste autonome — vidée, elle suit le compte par défaut de la société ✓
- **Coordonnées de l'émetteur sur la facture** — téléphone, e-mail et site web figurent sous votre adresse sur le PDF, saisis une fois dans les réglages ; chaque champ est facultatif et n'occupe aucune place s'il est vide ✓
- **Carnet de contacts structuré** — contacts **Personne** (prénom + nom) ou **Entreprise** (raison sociale), **adresse structurée** (QR-facture), et **personnes de contact** rattachées à une entreprise (interlocuteurs, à titre informatif) ✓
- **Prévention des doublons à la saisie** — pendant que vous tapez le nom d'un nouveau contact, Kesh signale ceux qui lui ressemblent déjà (casse, accents, traits d'union et espaces indifférents), et avertit si l'**IDE** saisi appartient à une fiche existante — y compris **archivée**, cas où l'enregistrement serait refusé sans que la fiche soit visible dans le carnet. Kesh **signale, il ne bloque jamais** : deux clients peuvent légitimement porter des noms très proches ✓
- **Avoirs (notes de crédit)** — annulation d'une facture validée par création d'un avoir lié (séquence séparée `AV-…`), contre-passation comptable automatique (TVA comprise), PDF « Avoir » ✓
- **Envoi de factures par e-mail** — bouton sur la facture validée : QR-facture PDF jointe, objet/message pré-remplis dans la **langue du contact** avec civilité personnalisée (modèles multilingues FR/DE/IT/EN configurables par l'admin), destinataire verrouillé sur la fiche contact, Reply-To = e-mail de la société, traçage « Envoyée le … à … » et renvoi possible ✓
- **Import bancaire CAMT.053 + CSV multi-encodage** — parser + persistance + UI ✓, profils banque réutilisables ✓, réconciliation automatique avec score ✓, réconciliation manuelle ✓, éclatement de transaction agrégée ✓ et règles d'affectation automatique ✓
- **Factures fournisseurs & règlement** — enregistrement d'une facture reçue (écriture d'achat automatique : charge + impôt préalable + dette créancier), règlement binaire en un clic (virement bancaire → compte source, ou compte interne libre), annulation par contre-passation ✓
- **Paiement par fichier pain.001** — génération d'un fichier de virement ISO 20022 `pain.001.001.09` (Swiss Payment Standards / SIX) à partir des factures fournisseurs ouvertes, flux deux temps (lot → import e-banking → confirmation comptabilise les règlements) ✓
- **Import de factures depuis un dossier** — dépôt de factures (PDF/image porteurs d'un Swiss QR-facture) dans un dossier surveillé, décodage du QR côté serveur, archivage du justificatif et création de factures « à compléter » (coordonnées de paiement pré-remplies), avec rapport d'import et lien « Voir la facture d'origine » ✓
- **TVA suisse** — calcul et rapport par période ✓, comptabilisation de la TVA due aux ventes ✓, assistant d'achat avec impôt préalable ✓, décompte TVA (solde net dû à l'AFC) et réconciliation rapport ↔ grand livre ✓ *(décompte officiel AFC / e-décompte ESTV à venir)*
- **Comptabilité analytique par projet** — dimension « projet » (2 niveaux, projet → sous-projets) affectable sur **tous les flux** de saisie : écritures manuelles (par ligne), factures de vente et fournisseurs (document), et réconciliation bancaire (rapprochement, ventilation, projet par défaut sur règle) ✓ ; deux **rapports** exportables PDF/CSV — **Dépenses par projet** (toutes les charges, drill-down jusqu'à l'écriture, pour les déductions fiscales) et **Rendement par projet** (coût investi / revenus / résultat net / rendement %), avec agrégation des sous-projets et vue par exercice ou cumulée ✓
- **Clôture d'exercice & report à-nouveau** — les soldes du bilan se **reportent automatiquement** d'un exercice à l'autre, calculés en temps réel depuis l'origine (aucune écriture de clôture ni de report à passer à la main) ✓ ; lignes **« Résultat reporté »** et **« Résultat de l'exercice »** au bilan, section **« Capitaux propres »** dédiée (CO 959a) ✓ ; la clôture est un **verrou** (immutabilité + audit, CO art. 957-964) avec **réouverture possible** par un administrateur sur motif justifié et tracé ✓ ; **rôle explicite** par compte — Kesh ne devine jamais la fonction d'un compte d'après son numéro, le plan comptable reste librement renumérotable ✓ ; **bilan d'ouverture** pour reprendre une comptabilité existante (écran « Soldes de départ ») ✓
- **API externe à clé PAT** — clés d'accès *read* / *read-write* par entreprise pour intégrations IA & logiciels tiers (auth `Authorization: Bearer`, gestion via `/settings/api-keys`) ✓ ; **aucune clé n'atteint les fonctions d'administration**, quel que soit le rôle de qui l'a créée — révoquer une clé compromise suffit ✓ — voir [`docs/api-external.md`](docs/api-external.md)
- **Export/import d'installation** — sauvegarde complète `.keshbackup` (toutes les sociétés, utilisateurs et données système) via l'UI admin (`Administration → Sauvegarde complète` / `Restaurer / Importer`) pour migrer ou restaurer une installation sans accès SSH ✓ — réservé au rôle Admin
- **Récupération de mot de passe par email** — lien de réinitialisation self-service (valable 30 min, usage unique, anti-énumération), opt-in via `KESH_FEATURE_FORGOT_PASSWORD` + config SMTP ✓ — fallback break-glass admin conservé
- **Multilingue** — messages d'erreur API en FR/DE/IT/EN (langue choisie à l'onboarding ; sélecteur de langue dans l'interface à venir)
- **Multi-utilisateurs** — RBAC avec rôles, JWT + refresh tokens, isolation multi-tenant par `company_id`

## Pile technique

- **Backend** : Rust 1.96 (édition 2024), Axum, SQLx
- **Frontend** : SvelteKit 2 + Svelte 5, TypeScript, Tailwind CSS 4
- **Base de données** : MariaDB 10.11+ (parité prod NAS Synology Package Center DSM, compat ≥ 10.6 — cf. Story 10-1 D3)
- **Déploiement** : Docker Compose (web app uniquement)
- **Tests** : `cargo test`, Vitest, Playwright

## Démarrage rapide

### Prérequis

- Rust ≥ 1.96 (installé automatiquement via `rust-toolchain.toml`)
- Node.js ≥ 20
- Docker + Docker Compose

### Installation

```bash
# 1. Cloner le repo
git clone https://github.com/guycorbaz/kesh.git
cd kesh

# 2. Démarrer MariaDB + backend (mode dev complet)
docker compose -f docker-compose.dev.yml up -d

# 3. Configurer l'environnement
cp .env.example .env
# Adapter les valeurs dans .env

# 4. Frontend (hot reload)
cd frontend
npm install
npm run dev
```

L'application est accessible sur http://localhost:5173 (frontend dev) et http://localhost (API en mode Docker, port 80 ; en mode `cargo run` natif sur Linux non-root, lancer le backend avec `KESH_PORT=3000` ET le frontend avec `KESH_BACKEND_URL=http://localhost:3000 npm run dev` pour aligner le proxy vite).

### Image Docker (production)

Les images officielles sont publiées sur Docker Hub à chaque tag `v*.*.*` :

```bash
docker pull gcorbaz/kesh:latest
```

## Structure du projet

```
kesh/
├── crates/                  # Backend Rust (workspace multi-crates)
│   ├── kesh-core/           # Logique métier pure (types, validation)
│   ├── kesh-db/             # Persistance MariaDB, migrations
│   ├── kesh-api/            # Serveur HTTP Axum
│   ├── kesh-i18n/           # Internationalisation (Fluent)
│   ├── kesh-qrbill/         # Génération QR Bill 2.2
│   ├── kesh-payment/        # Fichiers pain.001
│   ├── kesh-import/         # Parseurs CAMT.053, CSV
│   ├── kesh-reconciliation/ # Rapprochement bancaire
│   ├── kesh-report/         # Bilan, résultat, balance
│   └── kesh-seed/           # Données d'amorçage
├── frontend/                # SvelteKit SPA
├── charts/                  # Plans comptables suisses
├── docs/                    # Documentation technique
└── .github/workflows/       # Pipelines CI/CD
```

## Documentation

- **Manuel utilisateur complet** (FR) : [`docs/manual/fr/user-manual.pdf`](docs/manual/fr/user-manual.pdf) — référence d'utilisation au quotidien (onboarding, comptabilité, facturation QR Bill, import bancaire, réconciliation, rapports, conformité).
- **Guide de démarrage rapide** (FR) : [`docs/user-guide/fr/getting-started.md`](docs/user-guide/fr/getting-started.md) — prise en main express pour un premier usage.
- **Manuel administrateur** (FR) : [`docs/manual/fr/admin-manual.pdf`](docs/manual/fr/admin-manual.pdf) — installation, configuration, sécurité, déploiement Docker, sauvegardes.
- **API externe** : [`docs/api-external.md`](docs/api-external.md) — authentification par clé API (PAT), points d'accès, exemples.

> Les manuels DE / IT / EN sont prévus pour une version ultérieure (le français reste la langue canonique de la documentation).

## Architecture

### Multi-tenant (Story 6.2)

Kesh supporte plusieurs sociétés par instance via un modèle multi-tenant :

- **JWT claims** : chaque token contient `user_id`, `role`, et **`company_id`**
- **Scoping** : toutes les requêtes filtrent par `company_id` du JWT (défense en profondeur contre IDOR)
- **Onboarding** : création de la company lors de l'inscription (contrat Story 6.1)
- **Foreign Key** : `users.company_id` NOT NULL, FK vers `companies.id`

Chaque user est assigné à exactement une company. Le `company_id` est inclus au JWT à la connexion (story 1.5) et utilisé pour scoper tous les accès aux ressources (comptes, contacts, factures, écritures comptables, etc.).

### Recherche full-text (Story 7.4)

Les recherches sur les colonnes texte longues utilisent un index `FULLTEXT` MariaDB avec `MATCH AGAINST IN BOOLEAN MODE` (10×+ speedup vs `LIKE '%query%'` au-delà de ~50k lignes) :

- **4 colonnes indexées** : `contacts.name`, `products.name`, `products.description`, `journal_entries.description`.
- **LIKE conservé** sur les colonnes structurées courtes (`email`, `invoice_number`, `payment_terms`).
- **UX prefix-search** préservée via auto-append `*` côté repository (`Mar*` matche `Marie`).
- **Régression v0.1 documentée** : perte du mid-word search (`argo` ne matche plus `Camargo`) — accepté pour v0.1, traçable via 3 régression detectors actifs.

Détails du pattern, limitations BOOLEAN MODE, runbook récupération échec migration : [docs/search-patterns.md](docs/search-patterns.md).

## Développement

### Commandes utiles

```bash
# Backend
cargo build --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all

# Frontend
cd frontend
npm run check          # svelte-check
npm run build          # build production
```

### Workflow Git

- Branche principale : `main`
- Les commits sur `main` déclenchent le pipeline CI (tests + build).
- Les tags `v*.*.*` déclenchent le pipeline Release (build et push Docker Hub).

## Tests

```bash
# Tests unitaires + intégration backend
DATABASE_URL='mysql://root:...@127.0.0.1:3306/kesh' \
  cargo test --workspace -- --test-threads=1

# Tests unitaires frontend
cd frontend && npm run test:unit

# Tests E2E Playwright
cd frontend && npm run test:e2e
```

> **Note** : les tests d'intégration SQLx créent des bases éphémères `_sqlx_test_*`. L'utilisateur DB doit avoir les droits `CREATE/DROP` sur `*.*` (en local, utiliser `root`).

## Feuille de route

Le projet suit une approche **BMAD** (Breakthrough Method of Agile AI-driven Development) avec une feuille de route structurée en epics :

| Version | Epics | Statut |
|---------|-------|--------|
| v0.1 | E1 Fondations & Authentification, E2 Onboarding & Configuration, E3 Plan comptable & Écritures, E4 Carnet d'adresses & Catalogue, E5 Facturation QR Bill, E6 Qualité & CI/CD, E7 Technical Debt Closure, E8 Import bancaire & Réconciliation, E9 Rapports & Exports, E9.5 Technical Debt Closure, E10 Déploiement & Opérations | ✅ Done |
| v0.1.1 (hotfix) | Logs fichier avec rotation, fix onboarding catch-22 (fresh install) | ✅ Done |
| v0.1.2 (hotfix) | Port 80 par défaut, onboarding self-service unifié (admin créé via UI au 1er boot, recovery break-glass via `.env`) | ✅ Done |
| v0.1.3 (hotfix) | Déblocage déploiements LAN HTTP-only (`KESH_COOKIE_SECURE=false` pour LAN privé `*.home.arpa` derrière Traefik sans HTTPS) | ✅ Done |
| v0.1.4 (hotfix) | CRUD `bank_accounts` post-onboarding + sidebar collapsible + restructuration UX (pages orphelines, widget solde homepage) | ✅ Done |
| v0.1.5 (hotfix) | Fix pages Facturer/Échéancier blanches en HTTP LAN (`crypto.randomUUID` hors contexte sécurisé) + scroll des listes déroulantes longues (plan comptable) | ✅ Done |
| v0.1.6 (hotfix) | Page détail d'une écriture comptable (`/journal-entries/{id}`) + fix bouton 404 « Voir l'écriture comptable » + UX facture (placement boutons ajout de ligne, libellé bouton impression) | ✅ Done |
| v0.1.7 (hotfix) | Aide + message d'erreur actionnable sur le champ QR-IBAN (compte bancaire) + fiabilisation de la suite de tests `fiscal_year` (dette technique) | ✅ Done |
| v0.1.8 (hotfix) | Numéro de version affiché corrigé : provient désormais du backend au runtime (champ `version` de `/health`) au lieu d'être codé en dur dans le frontend | ✅ Done |
| v0.2.0 | **E17 Infra & Souveraineté** — API externe PAT, export/import complet d'installation (`.keshbackup`), récupération de mot de passe par email, fix sécurité TOCTOU onboarding | ✅ Done |
| v0.3.0 | **E11/E18 TVA Suisse** — calcul + rapport par période, comptabilisation TVA due aux ventes, assistant d'achat (impôt préalable), décompte TVA (solde net AFC) et réconciliation rapport ↔ grand livre *(décompte officiel AFC / e-décompte ESTV hors périmètre)* | ✅ Done |
| v0.3.1 (hotfix) | Message actionnable lors de la suppression d'une écriture liée à une facture validée + garde-fou journaux (pas de déversement SQL+données quand `RUST_LOG=debug`) | ✅ Done |
| v0.3.2 | **E12 Avoirs (notes de crédit)** — annulation d'une facture validée par avoir lié, contre-passation comptable automatique (TVA comprise), PDF « Avoir », décompte TVA cohérent | ✅ Done |
| v0.4.0 | **E12 Factures fournisseurs & paiements** (#191/#194) — enregistrement de factures reçues + règlement binaire (virement bancaire / compte interne), génération **pain.001** (virement en deux temps), **import de factures depuis un dossier surveillé** avec décodage QR-facture, scan QR à la saisie ; **E19 Comptabilité analytique par projet** (#195) — dimension projet sur **tous les flux** (écritures manuelles, factures ventes & fournisseurs, réconciliation bancaire) + rapports **Dépenses** et **Rendement par projet** (rollup sous-projets, exercice/cumulé, PDF/CSV) ; base technique modernisée (**Rust 1.96**, dépendances à jour) | ✅ Done |
| v0.5.0 | **Adresses structurées (QR-facture type S)** (#213) — conformité SIX : adresses société & contacts en champs séparés (rue/n°/NPA/localité/pays), génération QR type S ; **contacts Personne/Entreprise** (prénom/nom séparés) + **personnes de contact** d'une entreprise (CRM informatif) | ✅ Done |
| v0.6.0 | **E20 Envoi de factures par e-mail** (#224) — modèles d'e-mail multilingues FR/DE/IT/EN éditables (Réglages, Admin), envoi de la facture PDF QR au contact (langue & civilité du contact, destinataire verrouillé, Reply-To société) | ✅ Done |
| v0.7.0 | **E21 Échéances & relances débiteurs** (#245, #231, #246) — conditions de paiement structurées sur le contact (échéance pré-calculée), correction des montants TTC (QR/PDF/e-mail/échéancier), rappels de paiement multi-niveaux configurables (frais, envoi manuel par e-mail via les modèles), historique de relance, balance âgée, suspension de relance, doc admin/user | ✅ Done |
| v0.8.0 | **E14 Clôture d'exercice** (#232) — **report à-nouveau automatique** : soldes de bilan cumulés en temps réel d'un exercice à l'autre (modèle virtuel Odoo/Flectra), lignes « Résultat reporté » + « Résultat de l'exercice » au bilan, exports CSV/PDF, clôture = verrou (immutabilité CO 957-964) ; **rôles de comptes explicites** (le rôle d'un compte ne se déduit plus de son numéro) + réactivation d'un compte archivé (#269) ; **réouverture d'un exercice clôturé** (Admin, motif obligatoire, audit, garde LIFO) ; **bilan d'ouverture** (écran « Soldes de départ » : reprise de comptabilité, écriture d'ouverture équilibrée datée au 1er jour du premier exercice) | ✅ Done |
| v0.9.0 | **E16 Facturation avancée** (#152, #144, #151) — **compte de produit par ligne de facture** : chaque ligne peut porter son propre compte de produit (repli sur le compte par défaut de la société), l'écriture comptable ventile le crédit sur les bons comptes, avoirs compris ; compte de produit par défaut sur la fiche produit ; **compléments du PDF de facture** (#151) — récapitulatif TVA par taux, coordonnées de contact de l'émetteur, et **numéro de client** du destinataire (saisi sur la fiche contact, unique par société, cherchable, avoirs compris) | ✅ Done |
| v0.10.0 | **E22 Technical Debt Closure** (#294, #295, #167, #251, #301) — **prévention des doublons à la saisie d'un contact** : pendant la frappe, Kesh signale les contacts au nom proche (casse, accents, traits d'union et invisibles indifférents) et avertit si l'IDE saisi appartient déjà à une fiche, y compris **archivée** — cas où l'enregistrement serait refusé sans que la fiche soit visible ; il **signale, il ne bloque jamais**. Également : **numéro de client canonique** (deux numéros identiques à l'œil sont le même numéro, quelle que soit la composition Unicode ou la collation), **durcissement des clés API** (un PAT n'atteint plus aucune route d'administration — révoquer une clé compromise suffit désormais), et **suite de tests 42× plus rapide** (squash du schéma de test, l'heure devenue la minute et demie) | ✅ Done |
| v0.11.0 | **E23 Dette i18n** (#316, #283) — les écrans réclamaient des libellés que **aucun** des quatre catalogues ne contenait : Kesh servait alors du **français correct** à un germanophone, sans erreur ni avertissement. **1600 messages écrits** — 358 libellés français neufs et 414 dans chacune des trois autres langues, les quatre catalogues portant désormais **les mêmes 1630 clés** —, et **six garde-fous** posés pour que le défaut ne puisse plus revenir en silence — le principal étant devenu **inconditionnel** : une clé sans traduction ne peut plus être tolérée, elle fait rougir le gate | ✅ Done |
| v0.11.1 (hotfix) | **Trois défauts qui ne disaient rien** — l'**écran d'import bancaire se vidait** dès qu'on y déposait un fichier CAMT.053, rendant la fonction inutilisable ; un **compte de facturation pouvait s'effacer tout seul** après être devenu non imputable, faisant échouer la facturation suivante sur une erreur qui ne nommait pas la cause ; le **filtre Statut des factures** rendait une liste vide trompeuse depuis la page 2. Plus un en-tête de colonne sans nom pour les lecteurs d'écran | ✅ Done |
| v0.6+ | E15 Justificatifs, Lettrage & Compléments (inc. journaux personnalisables) | 📋 Backlog |
| v0.5+ (prévu) | **Tableau de bord & Comptabilité personnelle** — widgets configurables sur la page d'accueil (évolution du patrimoine fortune & dettes mois par mois [#164], donut de répartition des dépenses par compte/sous-compte [#165], comparatif recettes/dépenses mensuel [#166]) ; **Budgets** (E13 [#196]) + comparatif budget validé vs réalité [#197] | 📋 Backlog |

Détails : [PRD complet](_bmad-output/planning-artifacts/prd.md).

## Contribuer

Les contributions sont les bienvenues. Merci d'ouvrir une issue avant tout changement significatif pour en discuter.

- Respecter les règles de qualité du code (`CLAUDE.md`)
- Ajouter des tests pour toute nouvelle logique métier
- `cargo fmt` + `cargo clippy` doivent passer sans warning

## Licence

Distribué sous licence [EUPL 1.2](https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12).
