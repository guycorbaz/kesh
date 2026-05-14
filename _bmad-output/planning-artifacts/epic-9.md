---
epic: 9
title: "Rapports & Exports"
version: v0.1
status: backlog
sourceArtifact: _bmad-output/planning-artifacts/epics.md (legacy "Epic 8" section, lines 1104-1137)
relatedFRs:
  - FR65
  - FR66
  - FR67
  - FR68
relatedDecisions:
  - "Architecture decision #12 (kesh-report crate dédiée pour PDF tabulaire — séparée de kesh-qrbill)"
  - "Architecture decision #13 (kesh-i18n crate transversale — formatage suisse montants/dates)"
crates:
  - kesh-report (interne, deps : kesh-core + kesh-db + kesh-i18n)
stories:
  - 9-1-rapports-comptables-bilan-resultat-balance-journaux
  - 9-2-export-pdf-csv
---

# Epic 9 — Rapports & Exports

## Vue d'ensemble

**Objectif :** L'utilisateur peut générer ses 4 rapports comptables réglementaires (bilan, compte de résultat, balance des comptes, journaux) et les exporter en PDF (formats suisses) et en CSV (par rapport ou par table globale).

**Périmètre v0.1 :** génération des 4 rapports + export PDF/CSV par rapport + export CSV global par table (souveraineté des données). **Hors v0.1** : rapports TVA (Epic 11), comparatif budget vs réalisé (Epic 13), rapport de clôture / report des soldes (Epic 14), modèles documents personnalisables FR81 (Epic 15).

**Provenance :** epic créé 2026-05-14 par migration de la section legacy « Epic 8 : Rapports & Exports » dans [`epics.md`](epics.md) (renumérotage suite à insertion Epic 6 « Qualité & CI/CD », Epic 7 « Technical Debt Closure » et clôture Epic 8 « Import Bancaire & Réconciliation » — décisions rétro Epic 5, Epic 6 et Epic 7).

> ⚠️ **Drift `epics.md` non corrigé** : la section legacy y porte toujours le titre « Epic 8 », alors que `sprint-status.yaml` et ce fichier disent Epic 9. Même cause que [CR-009 #61](https://github.com/guycorbaz/kesh/issues/61) (drift Epic 7 → 8 jamais résolu) — à grouper dans une rectification globale `epics.md` pré-Epic 10. Ne pas modifier silencieusement `epics.md` sans CR explicite.

**Dépendances downstream (épics qui consomment ce travail) :**
- **Epic 11 (TVA Suisse)** — le rapport TVA réutilise les agrégations comptables de `kesh-report` (notamment compte de résultat et balance par compte).
- **Epic 13 (Budgets)** — le comparatif budget vs réalisé s'appuie sur le compte de résultat généré ici.
- **Epic 14 (Clôture d'exercice)** — la clôture produit un bilan de fermeture et un bilan d'ouverture, reposant sur les générateurs `balance_sheet.rs` et `income_statement.rs` livrés ici.

## Dépendances Epic 8 (Import Bancaire & Réconciliation)

**Acquises (aucun blocker) :**

| Item | Origine | Disponibilité Epic 9 |
|---|---|---|
| `journal_entries` table avec `fiscal_year_id` + lignes débit/crédit | Epic 3 | ✅ disponible |
| `bank_transactions.matched_entry_id` | Epic 8 (8-4) | ✅ disponible — réconciliation crée les écritures source des rapports |
| `reconciliation_rules.applied_count` + audit `reconciliation_rule.applied` | Epic 8 (8-5b) | ✅ disponible — traçabilité audit pour l'audit trail Swiss CO 958f |
| FULLTEXT search index sur écritures | Epic 7 (7-4 KF-005) | ✅ disponible — base pour la recherche FR69 si re-scopée Epic 9 |
| Pattern multi-tenant `company_id` | Epic 7 (7-1) + KF-002 | ✅ pattern stable, à appliquer sur toutes les requêtes `kesh-report` |
| Crate `kesh-i18n` (formatage suisse montants/dates) | Epic 2 (2-1) | ✅ disponible — utilisée par PDF et CSV |
| Pattern `accept_one_X` strict / `FailedProposal` per-proposal | Epic 8 (8-4 → 8-5b) | ⏸️ pas directement réutilisé (rapports = lecture seule, pas de batch écriture) mais à garder en tête si export par lot futur |

**Aucune dette architecturale d'Epic 8 ne bloque Epic 9** (verdict rétro Epic 8 §🔍 Readiness).

## KFs ouvertes héritées Epic 8 — impact Epic 9

| # | KF | Origine | Impact Epic 9 | Action |
|---|---|---|---|---|
| [#26](https://github.com/guycorbaz/kesh/issues/26) / [#76](https://github.com/guycorbaz/kesh/issues/76) | Multi-candidates UI v0.2 | 8-4 | Aucun (UI réconciliation, pas rapports) | Reporté v0.2 |
| [#70](https://github.com/guycorbaz/kesh/issues/70) | Frontend wiring `bankProfileId` + `confirmEncodingMismatch` | 8-2 | Aucun (UI import bancaire) | Reporté v0.2 ou bug-fix story dédiée |
| [#54](https://github.com/guycorbaz/kesh/issues/54) | KF-022 E2E helpers 401 | Epic 7 | **À surveiller** — Story 9-1 ajoute des routes `/api/v1/reports/*`, si E2E Playwright touche ces routes, helpers 401 partiels peuvent flaker | Re-évaluer au moment de la spec validate 9-1 |
| [#57](https://github.com/guycorbaz/kesh/issues/57) | KF-025 E2E timing | Epic 7 | **À surveiller** — génération PDF + assertions sur fichiers téléchargés sont sensibles aux timeouts Playwright | Re-évaluer si E2E flake reprend post-9-1 ou 9-2 |

## Architecture (rappel)

Cf. [`architecture.md`](architecture.md) §3 (decisions #12, #13), §11 (workspace Cargo), §17 (cartographie FR → modules). Structure cible :

```
journal_entries + lines (kesh-db)        Bilan/Résultat/Balance/Journaux
        ↓                                       ↓ data
   kesh-core (validation, calculs)       kesh-report
        ↓                                       ↓ format
   kesh-report (agrégation, rendu)       kesh-i18n (montants suisses, dates dd.mm.yyyy)
        ↓                                       ↓ output
   kesh-api/routes/reports.rs            PDF (printpdf ou équiv.) / CSV
        ↓
   features/reports/ (Svelte)
```

**Crate `kesh-report` (interne, déjà tranchée — décision archi #12) :**

```
crates/kesh-report/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── balance_sheet.rs        # Bilan (actifs/passifs par classe de compte)
    ├── income_statement.rs     # Compte de résultat (charges/produits)
    ├── trial_balance.rs        # Balance des comptes (soldes débit/crédit)
    ├── journal_report.rs       # Journaux (Achats, Ventes, Banque, Caisse, OD)
    ├── pdf.rs                  # Génération PDF tabulaire
    └── csv.rs                  # Export CSV
```

**Routes API anticipées :** `kesh-api/routes/reports.rs` — endpoints `GET /api/v1/reports/{type}?period_start&period_end&format=pdf|csv|json` où `{type}` ∈ `{balance-sheet, income-statement, trial-balance, journals}`. Plus `GET /api/v1/exports/global.zip` pour l'export CSV global par table.

**Frontend :** `frontend/src/features/reports/` — page de sélection rapport + filtre période + bouton génération + téléchargement.

**Toutes les requêtes scopent par `company_id`** — pattern multi-tenant Story 7-1 / KF-002 inviolable.

---

## Stories

### Story 9-1 : Rapports comptables (bilan, résultat, balance, journaux)

**As a** utilisateur
**I want** générer mes 4 rapports comptables (bilan, compte de résultat, balance des comptes, journaux)
**So that** je puisse vérifier ma situation financière et préparer ma clôture d'exercice

**Critères d'acceptation :**

- **Given** données comptables (écritures validées dans l'exercice), **When** génération du bilan, **Then** actifs et passifs affichés par classe de compte, totaux calculés, équation bilan vérifiée (`somme actifs = somme passifs + capitaux propres`) (FR65)
- **Given** données comptables, **When** génération du compte de résultat, **Then** charges et produits affichés par classe de compte, résultat net (bénéfice/perte) calculé
- **Given** données comptables, **When** génération de la balance des comptes, **Then** tous les comptes du plan comptable avec soldes débit/crédit affichés, totaux débit = totaux crédit
- **Given** données comptables, **When** génération des journaux, **Then** écritures listées par journal (Achats, Ventes, Banque, Caisse, OD) avec totaux par journal et total général
- **Given** un rapport, **When** filtrage par période (date de début / date de fin), **Then** seules les écritures dont la date est dans la période sont incluses
- **Given** plusieurs exercices ouverts, **When** sélection d'un exercice, **Then** les rapports portent sur les écritures de cet exercice uniquement
- **And** `kesh-report` génère les données (4 modules : `balance_sheet`, `income_statement`, `trial_balance`, `journal_report`)
- **And** `kesh-i18n` formate les montants (apostrophe séparateur de milliers — `1'234.56`) et dates (`dd.mm.yyyy`) selon locale active
- **And** scoping multi-tenant `company_id` sur toutes les requêtes (pattern Story 7-1)
- **And** audit trail Swiss CO Art. 958f : chaque génération de rapport peut être journalisée (à confirmer au spec validate — voir R2)

### Story 9-2 : Export PDF & CSV

**As a** utilisateur
**I want** exporter mes rapports en PDF et CSV
**So that** je puisse les partager avec mon fiduciaire, les archiver ou les importer ailleurs

**Critères d'acceptation :**

- **Given** un rapport généré, **When** export PDF, **Then** le PDF respecte les formats suisses : apostrophe séparateur de milliers (`1'234.56`), dates `dd.mm.yyyy` (FR67)
- **Given** un rapport, **When** export CSV, **Then** fichier CSV avec séparateur point-virgule, encodage UTF-8 BOM (compat Excel CH/DE)
- **Given** menu export, **When** export global par table, **Then** l'utilisateur peut exporter l'ensemble des données (comptes, écritures, contacts, factures, transactions bancaires) en CSV (FR68)
- **Given** export global, **When** génération, **Then** ZIP contenant un CSV par table + fichier `metadata.json` (version Kesh, date export, locale, périmètre exercice)
- **Given** export, **When** bouton dans le menu principal, **Then** accessible directement (pas caché dans les paramètres) — souveraineté des données
- **And** génération PDF d'un rapport `< 3 secondes` sur dataset de référence (~1000 écritures)
- **And** les messages d'erreur disent ce qui s'est passé ET ce que l'utilisateur peut faire (UX-DR38) — par exemple : « Aucune écriture dans la période sélectionnée. Modifiez les dates ou choisissez un autre exercice. »
- **And** scoping multi-tenant `company_id` sur l'export global (impossible d'exfiltrer les données d'une autre company)
- **And** choix de la librairie PDF documenté dans la story (`printpdf` candidate par défaut, à valider — voir R3)

---

## Critères d'arrêt Epic 9

Epic considéré « done » quand :

- [ ] 2/2 stories avec status `done` dans `sprint-status.yaml`
- [ ] FR65-FR68 tous validés via tests E2E (au moins un par FR, sur dataset seedé)
- [ ] Crate `kesh-report` créée avec les 4 générateurs + pdf + csv (cf. structure §Architecture)
- [ ] Tests d'intégration sur jeu de données de référence : bilan vérifié manuellement (au moins un cas), équation bilan respectée, balance débit/crédit alignée
- [ ] Pattern multi-tenant `company_id` validé via test IDOR cross-company sur chaque endpoint `/api/v1/reports/*` et sur l'export global
- [ ] Performance : génération PDF d'un rapport < 3 s sur dataset référence (~1000 écritures), export ZIP global < 10 s
- [ ] Aucun KF nouveau de sévérité > LOW non adressé ou non documenté en dette v0.2
- [ ] Rétrospective Epic 9 produite (status `done` dans `sprint-status.yaml`)

---

## Risques & questions ouvertes

Les éléments ci-dessous sont à clarifier ou à décider lors de la création de chaque story spec via `/bmad-create-story`. Si un risque devient un blocker, créer un GitHub Issue (template KF ou CR selon le type) et **ne pas modifier silencieusement les ACs**.

| # | Risque / question | Story impactée | À traiter |
|---|---|---|---|
| R1 | **Performance sur grandes datasets** : balance et journaux peuvent atteindre 10k+ lignes par exercice. Indexation queries (`fiscal_year_id`, `account_id`, `entry_date`) suffisante ou faut-il agrégation pré-calculée ? Pagination CSV pour très gros exports ? | 9-1, 9-2 | Spec validate Story 9-1 + benchmark sur dataset seed |
| R2 | **Conformité Swiss CO Art. 957a + 958f** : formats légaux pour bilan/résultat (classes de compte obligatoires, totaux à afficher), audit trail des générations de rapports. **Recherche réglementaire requise** (action item #3 retro Epic 8). Sans cela, risque d'écart entre Kesh et exigences fiduciaire. | 9-1 | **Recherche pré-spec** — action item #3 retro Epic 8 (owner Guy). Décision : faire la recherche avant `bmad-create-story 9-1` ou intégrer dans la spec validate Pass 1 ? |
| R3 | **Choix librairie PDF** : `printpdf` (pure Rust, candidat par défaut), `weasyprint` (Python via subprocess), `tectonic` (TeX, lourd), `genpdf` (déprécié), ou autre ? `kesh-qrbill` utilise déjà une approche pixel-perfect SIX (probablement `printpdf` ou `pdfium`) — à clarifier pour cohérence et réutilisation potentielle. | 9-2 | Spec validate Story 9-2 — vérifier `Cargo.toml` `kesh-qrbill` et reproduire le choix si compatible |
| R4 | **I18n des labels rapports** (FR/DE/IT/EN) : 4 rapports × ~30 labels chacun = ~120 clés Fluent à ajouter. Owner traductions (Guy ? AI generated ? validation native speaker ?). Cohérence terminologique avec plan comptable Sterchi (Epic 3). | 9-1 | Spec validate Story 9-1 + coordination avec lint-i18n-ownership Story 6-3 |
| R5 | **Scope FR69 (recherche écritures par montant/libellé/numéro/date)** : actuellement mappé `kesh-report/` per `architecture.md` §17 line 641, mais sémantiquement plus proche d'une feature Epic 3 (saisie écritures). Story 7-4 (KF-005 fulltext index) a déjà posé les indexes. Faut-il une Story 9-3 dédiée ou couvrir dans la liste écritures Epic 3 (créer 3-8 brownfield) ? | (à créer) | Décision pré-spec — propose : laisser FR69 hors Epic 9 et tracker via CR si re-scopé. |
| R6 | **Scope FR70 (rapport personnalisable / drill-down)** : non mentionné explicitement dans `epics.md` legacy mais cité dans le mapping `architecture.md` §17. Périmètre v0.1 ou v0.2 ? | (à clarifier) | Décision pré-spec — propose : reporter v0.2 (Epic 15) sauf demande explicite. |
| R7 | **Audit trail Swiss CO Art. 958f sur générations de rapports** : chaque génération doit-elle créer une ligne d'audit ? Si oui, quel niveau de détail (utilisateur, type rapport, période, hash du PDF) ? | 9-1, 9-2 | Spec validate Story 9-1 — décision conjointe avec R2 |
| R8 | **Réutilisation PDF format CO 957a pour Story 14-1 (clôture d'exercice)** : la clôture produira un bilan de fermeture + bilan d'ouverture qui doivent réutiliser les générateurs Epic 9. Garder l'API publique de `kesh-report::balance_sheet` et `income_statement` stable et documentée (helper public pattern Story 8-5a-base). | 9-1 | Spec validate Story 9-1 — exposer les types `BalanceSheet`, `IncomeStatement` dans `lib.rs` avec doc `///` complète |

**Pattern à respecter :** chaque risque traité dans la spec validate de la story correspondante doit produire soit (a) une décision documentée dans le story file, soit (b) un GitHub Issue (CR si scope change, KF si bug/dette) référencé dans la spec.

---

## Action items retro Epic 8 → suivi Epic 9

Cf. [`epic-8-retro-2026-05-14.md`](../implementation-artifacts/epic-8-retro-2026-05-14.md) §✅ Action items.

### CRITICAL PATH

| # | Action | Status au démarrage Epic 9 |
|---|---|---|
| 1 | Créer `epic-9.md` | ✅ **Done** — ce fichier |
| 2 | Décision crate `kesh-reports` indépendante vs module | ✅ **Already decided** — architecture decision #12 (`kesh-report` crate interne dédiée). Pas d'action additionnelle. |
| 3 | Recherche réglementaire Swiss CO Art. 957a (formats balance/bilan/résultat) | 🔴 **À faire avant spec validate 9-1** — owner Guy. Risque R2 ci-dessus. |
| 4 | README sync : Epic 8 🚧 En cours → ✅ Done | ✅ **Done** — vérifié 2026-05-14 ligne 169 README.md |

### PROCESS (CLAUDE.md)

| # | Action | Status |
|---|---|---|
| 5 | Codifier "grep ground-truth 3× minimum sur patches HIGH validate/code-review" | ⏸️ **Hors scope Epic 9** — à traiter en commit chore CLAUDE.md séparé |
| 6 | Codifier "AcceptedProposal pattern strict batch — pas d'AppError global escalation" | ⏸️ **Hors scope Epic 9** — à traiter en commit chore CLAUDE.md séparé. Pattern réutilisable Epic 12 (pain.001) plutôt qu'Epic 9. |

### CLEANUP PARALLEL

| # | Action | Status |
|---|---|---|
| 7 | Fixer KF [#70](https://github.com/guycorbaz/kesh/issues/70) frontend wiring 8-2 OU décider deferral v0.2 | ⏸️ Indépendant Epic 9 — peut tourner en parallèle |
| 8 | Fixer KF [#76](https://github.com/guycorbaz/kesh/issues/76) multi-candidates UI 8-4 OU décider deferral v0.2 | ⏸️ Indépendant Epic 9 |
| 9 | Re-évaluer KF [#54](https://github.com/guycorbaz/kesh/issues/54) + [#57](https://github.com/guycorbaz/kesh/issues/57) (E2E timing) | ⏸️ À re-checker au moment de la spec validate 9-1 (impact direct si E2E touche `/api/v1/reports/*`) |

---

## Références

- [`epics.md`](epics.md) — section legacy « Epic 8 : Rapports & Exports » (lines 1104-1137, antérieure au renumérotage 2026-05-14)
- [`architecture.md`](architecture.md) — décisions #12 (`kesh-report` crate dédiée) et #13 (`kesh-i18n` transversale), §11 workspace Cargo, §17 cartographie FR65-FR70 → `kesh-report/` + `features/reports/`
- [`prd.md`](prd.md) — FR65-FR68 (rapports + export), §189 mention CO art. 957-964 (intégrité écritures, conservation 10 ans), §UX-DR38 (messages d'erreur actionnables)
- [`epic-8-retro-2026-05-14.md`](../implementation-artifacts/epic-8-retro-2026-05-14.md) — action items 1-9 et preview Epic 9 (§🚀 Epic 9 preview & dependencies)
- Issues GitHub à surveiller pendant Epic 9 : [#54](https://github.com/guycorbaz/kesh/issues/54), [#57](https://github.com/guycorbaz/kesh/issues/57)
- KFs minor reportées v0.2 (sans impact Epic 9) : [#26](https://github.com/guycorbaz/kesh/issues/26), [#70](https://github.com/guycorbaz/kesh/issues/70), [#76](https://github.com/guycorbaz/kesh/issues/76)
