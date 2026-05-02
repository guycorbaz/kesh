---
epic: 8
title: "Import Bancaire & Réconciliation"
version: v0.1
status: backlog
sourceArtifact: _bmad-output/planning-artifacts/epics.md (legacy "Epic 7" section, lines 1016-1095)
relatedFRs:
  - FR42
  - FR43
  - FR44
  - FR45
  - FR46
  - FR47
  - FR48
  - FR49
  - FR50
  - FR51
  - FR52
  - FR53
relatedDecisions:
  - "Architecture decision #2 (multi-version parsers)"
  - "Architecture decision #9 (kesh-reconciliation dedicated module)"
crates:
  - kesh-import (publishable, zero internal deps)
  - kesh-reconciliation (kesh-core + kesh-db deps)
stories:
  - 8-1-import-camt053
  - 8-2-import-csv-multi-encodage-profils-banque
  - 8-3-detection-doublons-rejet-partiel
  - 8-4-reconciliation-matching-automatique
  - 8-5-reconciliation-manuelle-regles-affectation
---

# Epic 8 — Import Bancaire & Réconciliation

## Vue d'ensemble

**Objectif :** L'utilisateur peut importer ses relevés bancaires (CAMT.053 ISO 20022 + CSV multi-encodage) et réconcilier les transactions avec matching automatique et règles d'affectation.

**Périmètre v0.1 :** import + détection doublons + réconciliation auto/manuelle. **Hors v0.1** : génération pain.001 (Epic 12), TVA appliquée aux transactions importées (Epic 11).

**Provenance :** epic créé 2026-05-02 par migration de la section legacy « Epic 7 : Import Bancaire & Réconciliation » dans [`epics.md`](epics.md) (renumérotage suite à insertion Epic 6 « Qualité & CI/CD » et Epic 7 « Technical Debt Closure » — décisions rétro Epic 5 et Epic 6).

> ⚠️ **Drift `epics.md` non corrigé** : la section legacy y porte toujours le titre « Epic 7 », alors que sprint-status.yaml et ce fichier disent Epic 8. Tracé via [CR-009 #61](https://github.com/guycorbaz/kesh/issues/61) — à résoudre avant la création de `epic-9.md`.

**Dépendances downstream (épics qui consomment ce travail) :** Epic 9 (Rapports — bilan/résultat construits sur écritures issues de réconciliation), Epic 11 (TVA — appliquée aux transactions classifiées via règles d'affectation).

## Dépendances Epic 7 (Technical Debt Closure)

Le prep sprint Epic 8 (validé Guy 2026-05-01, ajusté 2026-05-02 — cf. [`epic-7-retro-2026-05-01.md`](../implementation-artifacts/epic-7-retro-2026-05-01.md)) adresse les fondations critiques **avant** Story 8-1 :

| Item | Issue | Impact Epic 8 |
|---|---|---|
| KF-020 SELECT FOR UPDATE pour update no-op | [#49](https://github.com/guycorbaz/kesh/issues/49) | **Bloquant Story 8-4** — la réconciliation crée/modifie des écritures, doit serializer pour fermer la race no-op résiduelle |
| KF-002-H-002 deadlock-retry middleware | [#43](https://github.com/guycorbaz/kesh/issues/43) | **Foundation Story 8-4** — multi-locks finalize (transaction + écriture + matching) sont des candidats deadlock typiques |
| Spike `kesh-import` crate design | (à créer prep) | **Bloquant Story 8-1** — décision « crate publiable indépendante, zéro dépendance sur `kesh-core`, types autonomes via `From`/`Into` » à valider sur un POC avant impl complète |
| Fixtures CAMT.053 SIX officielles | (à peupler prep) | **Bloquant Story 8-1** — `docs/six-references/` doit contenir les exemples SIX officiels pour les tests d'intégration multi-version |

**Cleanup E2E parallèle (KF-022/023/025) :** non bloquant, mais recommandé avant Story 8-1 pour éviter qu'Epic 8 hérite d'un baseline E2E fragile. Cf. issues [#54](https://github.com/guycorbaz/kesh/issues/54), [#55](https://github.com/guycorbaz/kesh/issues/55), [#57](https://github.com/guycorbaz/kesh/issues/57).

## Architecture (rappel)

Cf. [`architecture.md`](architecture.md) §11.5 et §17 pour les détails. Structure cible :

```
Fichier CAMT.053 → kesh-import (parse, multi-version)
                    ↓ From/Into
                  kesh-core (validation domaine)
                    ↓
                  kesh-reconciliation (matching, mutex per bank account)
                    ↓
                  kesh-db (persistance bank_imports + bank_transactions)
```

Routes API : `kesh-api/routes/bank_imports.rs`. Frontend : `features/bank-import/` + `features/reconciliation/`.

**Schémas DB anticipés** (à formaliser dans les migrations Story 8-1/8-2/8-5) :

- `bank_imports` (id, company_id, bank_account_id, file_hash, filename, imported_at)
- `bank_transactions` (id, import_id, bank_account_id, date, amount, reference, details, status `pending`/`reconciled`, matched_entry_id)
- `bank_profiles` (id, company_id, bank_name, column_mapping_json, date_format, encoding, created_at) — Story 8-2
- `import_rules` (id, company_id, match_pattern, account_id, priority, active, created_at) — Story 8-5

Toutes les tables scopent par `company_id` — pattern multi-tenant Story 6-2 / Story 7-1.

---

## Stories

### Story 8-1 : Import CAMT.053

**As a** utilisateur
**I want** importer mes relevés bancaires au format CAMT.053
**So that** les transactions apparaissent dans Kesh

**Critères d'acceptation :**

- **Given** fichier CAMT.053 valide, **When** import, **Then** toutes les transactions sont extraites avec date, montant, référence, détails
- **Given** fichier CAMT.053 avec sous-transactions (`TxDtls`), **When** import, **Then** les sous-transactions sont extraites individuellement (FR49)
- **Given** import, **When** lié à un compte bancaire, **Then** les transactions sont associées au bon compte (FR50)
- **Given** fichier, **When** glisser-déposer ou sélection, **Then** prévisualisation des transactions avant confirmation d'import
- **Given** parseur, **When** version du format détectée, **Then** le parseur sélectionne la version correspondante (multi-version — décision archi #2)
- **And** `kesh-import` est une crate publiable indépendante (zéro dépendance sur `kesh-core`) — à valider via spike prep
- **And** types autonomes dans `kesh-import`, conversion via `From`/`Into` vers `kesh-core`
- **And** tests d'intégration avec les fichiers de test SIX officiels (`docs/six-references/`)
- **And** schéma : tables `bank_imports` (id, company_id, bank_account_id, file_hash, filename, imported_at) et `bank_transactions` (id, import_id, bank_account_id, date, amount, reference, details, status `pending`/`reconciled`, matched_entry_id)
- **And** scoping multi-tenant `company_id` sur toutes les requêtes (pattern Story 6-2)

### Story 8-2 : Import CSV (multi-encodage & profils banque)

**As a** utilisateur
**I want** importer des relevés CSV de différentes banques
**So that** je puisse gérer toutes mes banques dans Kesh

**Critères d'acceptation :**

- **Given** fichier CSV UTF-8, **When** import, **Then** les transactions sont correctement parsées (FR42)
- **Given** fichier CSV ISO-8859-1, **When** import, **Then** détection automatique de l'encodage et parsage correct (FR52)
- **Given** format CSV inconnu, **When** import, **Then** l'utilisateur peut configurer un profil de format par banque (mapping colonnes) (FR53)
- **Given** profil banque configuré, **When** import suivant de la même banque, **Then** le profil est appliqué automatiquement
- **Given** rejet de lignes, **When** format de date non reconnu, **Then** listing détaillé des erreurs avec numéros de ligne (FR51)
- **And** schéma : table `bank_profiles` (id, company_id, bank_name, column_mapping_json, date_format, encoding, created_at)
- **And** scoping multi-tenant `company_id`

### Story 8-3 : Détection de doublons & rejet partiel

**As a** utilisateur
**I want** que Kesh détecte les doublons et gère les imports partiels
**So that** aucune transaction ne soit comptée deux fois

**Critères d'acceptation :**

- **Given** fichier déjà importé (même hash), **When** tentative de réimport, **Then** avertissement « fichier déjà importé » avec option de forcer (FR43)
- **Given** transactions qui chevauchent un import précédent, **When** import, **Then** les doublons sont détectés et signalés — aucune transaction en double
- **Given** fichier avec erreurs partielles, **When** import, **Then** les transactions valides sont importées, les invalides sont rejetées avec listing détaillé (FR51)
- **And** détection de doublons par combinaison : `date + montant + référence + bank_account_id`

### Story 8-4 : Réconciliation & matching automatique

**As a** utilisateur
**I want** que les transactions connues soient automatiquement proposées pour réconciliation
**So that** le travail de réconciliation soit minimal

**Critères d'acceptation :**

- **Given** transactions importées, **When** réconciliation, **Then** le système propose automatiquement des contreparties pour les transactions connues (factures en attente de paiement) (FR44)
- **Given** proposition de matching, **When** affichage, **Then** tableau avec transaction bancaire ↔ écriture comptable proposée + score de confiance
- **Given** propositions, **When** validation en lot, **Then** toutes les propositions acceptées sont réconciliées en une action
- **Given** transaction réconciliée, **When** vérification, **Then** l'écriture comptable correspondante est créée ou liée
- **And** le matching considère : montant exact, référence facture, nom client/fournisseur
- **And** **mutex par compte bancaire** pour éviter les imports concurrents (`kesh-reconciliation`) — fonde sur KF-020 #49 (SELECT FOR UPDATE) closed dans le prep sprint
- **And** deadlock-retry sur les multi-locks finalize — fonde sur KF-002-H-002 #43 closed dans le prep sprint
- **And** scoping multi-tenant `company_id`

### Story 8-5 : Réconciliation manuelle & règles d'affectation

**As a** utilisateur
**I want** réconcilier manuellement les transactions et créer des règles pour l'avenir
**So that** les prochains imports soient plus automatisés

**Critères d'acceptation :**

- **Given** transaction sans proposition, **When** création manuelle de contrepartie, **Then** l'utilisateur sélectionne le compte comptable et crée l'écriture (FR45)
- **Given** affectation manuelle effectuée, **When** feedback, **Then** le système propose de créer une règle d'affectation automatique (FR46)
- **Given** règles d'affectation, **When** gestion, **Then** CRUD des règles (description contient « X » → compte Y) (FR47)
- **Given** transaction agrégée (plusieurs sous-montants), **When** éclatement, **Then** l'utilisateur peut diviser en sous-lignes avec comptes différents (FR48)
- **And** les règles sont appliquées par priorité lors des imports suivants
- **And** schéma : table `import_rules` (id, company_id, match_pattern, account_id, priority, active, created_at)
- **And** scoping multi-tenant `company_id`

---

## Critères d'arrêt Epic 8

Epic considéré « done » quand :

- [ ] 5/5 stories avec status `done` dans `sprint-status.yaml`
- [ ] FR42-FR53 tous validés via tests E2E (au moins un par FR)
- [ ] Tests d'intégration `kesh-import` couvrant ≥ 2 versions CAMT.053 + ≥ 1 fichier SIX officiel par version
- [ ] `kesh-import` publiable (vérification : `cargo publish --dry-run` passe sans erreur, zéro dépendance interne)
- [ ] Pattern multi-tenant `company_id` validé via test IDOR cross-company sur chaque entité (`bank_imports`, `bank_transactions`, `bank_profiles`, `import_rules`)
- [ ] Aucun KF nouveau de sévérité > LOW non adressé ou non documenté en dette v0.2
- [ ] Rétrospective Epic 8 produite (status `done` dans `sprint-status.yaml`)

---

## Risques & questions ouvertes

Les éléments ci-dessous sont à clarifier ou à décider lors de la création de chaque story spec via `/bmad-create-story`. Si un risque devient un blocker, créer un GitHub Issue (template KF ou CR selon le type) et **ne pas modifier silencieusement les ACs**.

| # | Risque / question | Story impactée | À traiter |
|---|---|---|---|
| R1 | Détection doublons « date+montant+référence+compte » : tolérance dates (transaction valeur vs date booking ?), tolérance montants (centimes ?), gestion des références vides | 8-3 | Spec validate Story 8-3 |
| R2 | Statement balance reconciliation : FR42 ne mentionne pas de check « somme transactions == solde de clôture du fichier ». Anti-pattern silencieux possible si fichier corrompu/tronqué. | 8-1 | **CR-010 [#62](https://github.com/guycorbaz/kesh/issues/62) ouverte 2026-05-02** — à valider avant Story 8-1 spec |
| R3 | Gestion fichiers > 10 Mo : limite upload, streaming parser, timeout HTTP | 8-1, 8-2 | Spec validate Story 8-1 |
| R4 | Encodage CSV : « détection automatique UTF-8 / ISO-8859-1 ». Algorithme déterministe ? Heuristique ? Que faire si BOM absent et byte ambigu ? | 8-2 | Spec validate Story 8-2 |
| R5 | Score de confiance matching : seuil par défaut, configurabilité, exposé dans l'UI ? | 8-4 | Spec validate Story 8-4 |
| R6 | Règles d'affectation : conflit entre règles (ordre priorité strict ? règle wildcard finale ?). Pattern matching : substring vs regex ? | 8-5 | Spec validate Story 8-5 |
| R7 | Éclatement transaction agrégée (FR48) : balance débit/crédit doit rester équilibrée — comment empêcher un éclatement déséquilibré ? | 8-5 | Spec validate Story 8-5 |
| R8 | Spike `kesh-import` indépendance : si POC échoue (`kesh-core` types incontournables), Story 8-1 doit refondre l'archi → impact downstream | 8-1 | **Spike prep sprint** (avant Story 8-1) |

**Pattern à respecter :** chaque risque traité dans la spec validate de la story correspondante doit produire soit (a) une décision documentée dans le story file, soit (b) un GitHub Issue (CR si scope change, KF si bug/dette) référencé dans la spec.

---

## Références

- [`epics.md`](epics.md) — section legacy « Epic 7 : Import Bancaire & Réconciliation » (lines 1016-1095, antérieur au renumérotage 2026-04-20)
- [`architecture.md`](architecture.md) — §11.5 dépendances inter-crates, §17 cartographie FR → modules
- [`prd.md`](prd.md) — FR42-FR53 (banking imports + réconciliation), §UX-DR scenarios C-Réconciliation
- [`epic-7-retro-2026-05-01.md`](../implementation-artifacts/epic-7-retro-2026-05-01.md) — prep sprint scope (4 critical path items)
- Issues GitHub bloquantes : [#49](https://github.com/guycorbaz/kesh/issues/49), [#43](https://github.com/guycorbaz/kesh/issues/43)
- Issues GitHub cleanup parallèle : [#54](https://github.com/guycorbaz/kesh/issues/54), [#55](https://github.com/guycorbaz/kesh/issues/55), [#57](https://github.com/guycorbaz/kesh/issues/57)
- CRs ouverts pendant la création de cet epic : [#61 CR-009 epics.md drift](https://github.com/guycorbaz/kesh/issues/61), [#62 CR-010 FR42 balance check](https://github.com/guycorbaz/kesh/issues/62)
