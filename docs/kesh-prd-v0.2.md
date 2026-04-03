# Kesh — Product Requirements Document (PRD)
**Version:** 0.2-draft  
**Date:** 2026-03-18  
**Statut:** Draft — Agent PM  
**Méthode:** BMAD (Breakthrough Method of Agile AI-driven Development)

---

## 1. Vision produit

**Kesh** est un logiciel de comptabilité personnelle et de petite structure (indépendants, TPE) développé en Rust, conçu pour les réalités suisses : plan comptable helvétique, standards bancaires SIX, facturation QR Bill. Il est simple, rapide, multilingue et fonctionne aussi bien en web app (auto-hébergée) qu'en application desktop standalone.

**Proposition de valeur :**  
> *"La comptabilité suisse, sans complexité inutile — dans ta langue, sur ta machine ou ton serveur."*

---

## 2. Utilisateurs cibles

| Persona | Mode préféré |
|---|---|
| **Particulier organisé** | Desktop (Tauri) — données locales |
| **Indépendant / freelance** | Desktop ou web auto-hébergé |
| **Petite association** | Web app sur serveur partagé |
| **Fiduciaire légère** | Web app multi-dossiers sur VPS |

---

## 3. Modes de déploiement

### Mode A — Web App
- Backend Rust (Axum) déployé sur un serveur Linux
- Frontend Svelte servi statiquement
- MariaDB sur le même serveur ou distante
- Accès via navigateur (Chrome, Firefox, Safari)
- Multi-utilisateurs possible à terme

### Mode B — Desktop (Tauri)
- Binaire Tauri cross-platform (macOS, Linux, Windows)
- Embarque le frontend Svelte compilé
- Lance le backend Axum en processus enfant local
- Connexion à une MariaDB locale (installée séparément ou via Docker)
- Option : assistant premier lancement pour configurer la BDD

### Même codebase
- Le backend Rust et le frontend Svelte sont identiques dans les deux modes
- Seul le packaging et la gestion de la connexion BDD diffèrent
- Configuration via fichier `.kesh.toml` (chemin BDD, port, langue, etc.)

---

## 4. Périmètre MVP

### 4.1 Plan comptable
- Chargement d'un plan comptable depuis fichier (JSON/YAML/CSV)
- Plan comptable suisse standard inclus (PME, association, perso)
- Support multi-plan (changer de référentiel par dossier/société)
- Numérotation par classe (1xxx actifs, 2xxx passifs, etc.)
- Ajout / modification / archivage de comptes

### 4.2 Saisie comptable
- Écritures en partie double (débit / crédit)
- Journaux : Achats, Ventes, Banque, Caisse, OD
- Lettrage automatique et manuel
- Import d'écritures depuis CSV normalisé

### 4.3 Import relevés bancaires
- Formats supportés :
  - **CAMT.053** (ISO 20022 — standard SIX suisse)
  - **MT940** (SWIFT legacy)
  - **CSV générique** (UBS, PostFinance, Raiffeisen, Neon, Revolut)
- Règles d'affectation automatique (apprentissage par règles)
- Interface de validation / correction avant intégration

### 4.4 Fichiers de paiement
- Génération **pain.001.001.03** (virements SEPA/SIX)
- Support QR-IBAN et IBAN classique
- Batch de paiements depuis les écritures ouvertes
- Export XML prêt pour import e-banking

### 4.5 Facturation QR Bill
- Génération de factures avec **Swiss QR Code** (standard SIX)
- Champs : QR-IBAN, montant, monnaie (CHF / EUR), référence QR
- Export PDF (A4, avec coupon QR en bas de page)
- Modèles de facture personnalisables
- Numérotation automatique des factures

### 4.6 Rapports
- Bilan (actif / passif)
- Compte de résultat (pertes & profits)
- Balance des comptes
- Journal général / journaux spécialisés
- Export : PDF, CSV, XLSX

---

## 5. Exigences non-fonctionnelles

| Critère | Exigence |
|---|---|
| **Langues** | FR, DE, IT, RM (Rumantsch Grischun), EN |
| **Plateforme desktop** | macOS, Linux, Windows |
| **Plateforme web** | Tout navigateur moderne (ES2020+) |
| **Base de données** | MariaDB 10.6+ |
| **Performance** | Import 10k lignes < 5s, pages < 300ms |
| **Offline desktop** | Fonctionnel sans connexion internet |
| **Confidentialité** | Aucune télémétrie par défaut |
| **Licence** | Open source (MPL-2.0 ou EUPL à définir) |

---

## 6. Architecture technique

```
kesh/
├── crates/
│   ├── kesh-core/        # moteur comptable pur (logique métier, pas d'I/O)
│   ├── kesh-db/          # couche MariaDB (sqlx + migrations)
│   ├── kesh-import/      # parseurs CAMT, MT940, CSV
│   ├── kesh-payment/     # génération pain.001 (XML ISO 20022)
│   ├── kesh-qrbill/      # Swiss QR Bill (génération + validation)
│   ├── kesh-i18n/        # traductions (Fluent .ftl)
│   ├── kesh-report/      # moteur de rapports + export PDF/XLSX
│   └── kesh-api/         # serveur Axum (REST + WebSocket)
├── frontend/             # Svelte + TypeScript (SPA)
│   ├── src/
│   └── vite.config.ts
├── src-tauri/            # shell Tauri (démarre kesh-api en sidecar)
│   ├── src/
│   └── tauri.conf.json
└── kesh.toml             # configuration (BDD, port, langue...)
```

### Flux de données (desktop)
```
[Tauri Shell]
    └─ lance ──► [kesh-api (Axum, port local aléatoire)]
                      └─ requêtes SQL ──► [MariaDB locale]
[WebView Tauri]
    └─ HTTP/WS ──► [kesh-api]
```

### Flux de données (web)
```
[Navigateur]
    └─ HTTPS ──► [kesh-api (Axum, reverse proxy nginx)]
                      └─ requêtes SQL ──► [MariaDB serveur]
```

---

## 7. Stack Rust

| Crate | Usage |
|---|---|
| `axum` | Serveur HTTP/WebSocket |
| `sqlx` | Accès MariaDB async + migrations |
| `serde` / `serde_json` | Sérialisation |
| `tauri` | Shell desktop cross-platform |
| `rust_decimal` | Arithmétique monétaire exacte (jamais f64) |
| `fast_qr` | Génération QR Code |
| `printpdf` | Génération PDF |
| `roxmltree` | Parsing XML (CAMT, pain.001) |
| `fluent-bundle` | Internationalisation |
| `tokio` | Runtime async |
| `tower-http` | Middleware CORS, compression, auth |
| `jsonwebtoken` | Auth JWT (mode web) |

---

## 8. Base de données MariaDB

### Schéma principal (ébauche)
```sql
-- Sociétés / dossiers comptables
CREATE TABLE companies (id, name, locale, fiscal_year_start, chart_of_accounts_id);

-- Plan comptable
CREATE TABLE accounts (id, company_id, number, name_fr, name_de, name_it, name_rm, name_en, class, type, parent_id, archived);

-- Écritures
CREATE TABLE journal_entries (id, company_id, date, journal, reference, description, locked);
CREATE TABLE journal_lines (id, entry_id, account_id, debit, credit, memo);

-- Factures
CREATE TABLE invoices (id, company_id, number, date, due_date, customer, qr_iban, amount, currency, status);

-- Paiements
CREATE TABLE payments (id, company_id, pain001_batch_id, iban, amount, currency, reference, status);

-- Règles d'import
CREATE TABLE import_rules (id, company_id, pattern, account_id, priority);
```

### Migrations
- Gérées via `sqlx migrate` (fichiers versionnés dans `crates/kesh-db/migrations/`)
- Compatible MariaDB 10.6+

---

## 9. Internationalisation (i18n)

- Toute chaîne UI externalisée dès le jour 1
- Fichiers de traduction : format **Fluent** (`.ftl`) par locale
- Locales MVP : `fr-CH`, `de-CH`, `it-CH`, `rm-CH`, `en-CH`
- Formats régionaux : dates (`dd.mm.yyyy`), montants (`CHF 1'234.56`)
- Langue détectée depuis OS (desktop) ou navigateur (web), modifiable par l'utilisateur
- Backend expose les traductions via API (le frontend n'embarque pas les `.ftl`)

---

## 10. Standards suisses à implémenter

| Standard | Organisme | Usage |
|---|---|---|
| Swiss QR Bill 2.2 | SIX | Facturation |
| pain.001.001.03 | SIX / ISO 20022 | Paiements |
| CAMT.053.001.04 | SIX / ISO 20022 | Relevés bancaires |
| QR-IBAN | SIX | Identification compte |
| Plan comptable PME | FIDAG / EXPERTsuisse | Comptabilité |
| TVA suisse | AFC (ESTV) | Taux 8.1% / 3.8% / 2.6% |

---

## 11. Sécurité

- Auth JWT pour le mode web (sessions courtes + refresh token)
- Mode desktop : pas d'auth obligatoire (mono-utilisateur local)
- Connexion MariaDB via credentials dans `kesh.toml` (non commité)
- TLS obligatoire en mode web (nginx upstream)
- Pas de données envoyées à l'extérieur

---

## 12. Hors périmètre MVP

- Synchronisation cloud
- Application mobile
- Module TVA déclaratif (envoi électronique à l'AFC)
- Multi-utilisateurs avec droits granulaires
- Open Banking API (PSD2, bLink SIX)
- Module salaires / RH

---

## 13. Critères d'acceptation MVP

- [ ] Créer un dossier comptable avec plan comptable suisse chargé
- [ ] Saisir 50 écritures manuelles et obtenir un bilan correct
- [ ] Importer un relevé CAMT.053 PostFinance et valider les écritures
- [ ] Générer un fichier pain.001 avec 3 paiements valides
- [ ] Produire une facture PDF avec QR Code scannable par une banque suisse
- [ ] Afficher l'interface en FR, DE, IT sans erreur de traduction
- [ ] Exporter un bilan en PDF
- [ ] Fonctionne en mode desktop (Tauri + MariaDB locale)
- [ ] Fonctionne en mode web (navigateur + MariaDB serveur)

---

## 14. Prochaines étapes BMAD

1. ✅ **PRD v0.1** — périmètre initial
2. ✅ **PRD v0.2** — architecture web+desktop, MariaDB
3. ⬜ **Architecture** — ADR, diagrammes détaillés, schéma BDD complet
4. ⬜ **User Stories** — découpage en sprints
5. ⬜ **Sprint 1** — scaffold Cargo workspace + kesh-db migrations + kesh-api hello world

---

*Document généré dans le cadre du workflow BMAD — à valider par le product owner avant de passer à l'agent Architect.*
