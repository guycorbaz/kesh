# Story 12.2: Factures fournisseurs & règlement (binaire)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- SPEC UMBRELLA — candidate split 12-2a..f (cf. §"Découpage / split candidate"). Patron : épopées 12-1 / 17-3 / 17-4 / 18-1. -->

## Story

As a comptable PME utilisant Kesh,
I want enregistrer les factures reçues de mes fournisseurs puis les régler en un clic (virement bancaire ou compte interne),
so that je peux suivre ce que je dois (« ouvert / payé »), comptabiliser automatiquement l'achat et le règlement, et préparer l'export pain.001 (12-3) sans ressaisie.

## Contexte & source

- Issue GitHub **#191** « Factures fournisseurs & paiements » (réalignée 2026-06-27 sur le modèle binaire).
- Mémoire de design figée : `project_epic_12_supplier_invoices_design.md` (décisions produit Guy 2026-06-27).
- Suite de l'épopée 12-1 (Avoirs, mergée v0.3.2) dont elle **réutilise le pattern d'implémentation** (entité + migration + repo + routes + feature frontend), et de l'Epic 18 (Comptabilisation TVA & Achats) dont elle **réutilise la logique de comptabilisation d'achat TVA**.
- Cible : **v0.4** (épopée « Paiements » d'Epic 12). 12-3 (pain.001) et 12-4 (scan QR) sont **hors scope** de cette story.

## Modèle métier figé (décisions produit Guy)

**Règlement = choix BINAIRE — PAS de table de modes configurables.** Payer une facture fournisseur solde toujours **Créanciers (2000)** ; l'écriture est `D 2000 / C <contrepartie>`, où la contrepartie dépend du choix au moment de payer :

1. **Virement bancaire** → l'utilisateur choisit un **compte bancaire source** de la société (`bank_accounts`, Epic 8). Contrepartie = `bank_account.journal_account_id` (C 102x). L'IBAN de ce compte servira de **débiteur pain.001** (12-3). En 12-2, le règlement est **immédiat** (écriture postée, statut payé) ; le flux deux-temps d'export est ajouté en 12-3.
2. **Compte interne** → l'utilisateur choisit **librement un compte du plan comptable** (1000 caisse, compte carte de crédit, compte Twint…). Contrepartie = ce compte. Règlement **immédiat**, **jamais** de pain.001.

« Espèces / carte / Twint » ne sont PAS des modes pré-définis : ce sont juste des comptes internes choisis dans l'option (2).

**Orthogonalité QR** : les coordonnées de paiement (IBAN/QR-IBAN + référence + montant) sont une **donnée de la facture**, capturée à la saisie, **orthogonale** au mode de règlement. Une facture QR peut être réglée par n'importe quel mode (dont compte interne « espèces » : `D 2000 / C 1000`, sans pain.001). Ces coordonnées ne sont consommées que par le mode virement (12-3).

## Acceptance Criteria

**Enregistrement (saisie manuelle, chemin A)**

1. Un Comptable+ peut **enregistrer une facture fournisseur** en saisissant : fournisseur (contact `is_supplier = true`), n° de facture fournisseur (texte libre, optionnel), date de facture, échéance (optionnelle), et une ou plusieurs lignes (description, montant HT, taux TVA, compte de charge). Optionnellement : coordonnées de paiement (IBAN **ou** QR-IBAN, référence, montant attendu).
2. À l'enregistrement, le système **poste atomiquement l'écriture d'achat** réutilisant la logique TVA Epic 18 : pour chaque ligne `D compte de charge (HT)` + `D impôt préalable 1171 (TVA récupérable)` agrégé + `C créanciers 2000 (TTC)`. L'écriture est équilibrée par construction (`Σdébit = Σcrédit`). Statut initial = **`open`**.
3. La facture fournisseur référence l'écriture d'achat créée (`purchase_journal_entry_id`) et son TTC (`total_amount`).
4. Le compte de charge et le taux TVA par ligne sont libres (multi-lignes, multi-taux supportés) ; le montant de TVA par taux est calculé via le helper canonique `kesh-core::accounting::vat::line_vat_amount` (arrondi half-up 2 décimales, taux en pourcent).
5. Le compte **créanciers** (2000) est résolu via un **compte par défaut configuré** au niveau société (nouveau `company_invoice_settings.default_payable_account_id`). Si absent, l'enregistrement échoue avec une erreur métier explicite (config requise), pas un 500.
6. Validation : refus si fournisseur n'est pas `is_supplier`, si une ligne a un montant HT ≤ 0, si un compte de charge n'existe pas / n'appartient pas à la société, si la date tombe hors d'un exercice ouvert.

**Règlement binaire (action « Payer »)**

7. Un Comptable+ peut **payer** une facture fournisseur au statut `open` via un choix binaire :
   - **virement** : `{ type: "bank_transfer", bankAccountId }` → contrepartie = `bank_account.journal_account_id` du compte choisi ;
   - **compte interne** : `{ type: "internal_account", accountId }` → contrepartie = ce compte du plan comptable.
8. Le paiement **poste atomiquement l'écriture de règlement** `D créanciers 2000 (TTC) / C contrepartie (TTC)`, passe la facture au statut **`paid`**, horodate `paid_at`, et enregistre le type/compte de règlement utilisés (`settlement_type`, `settlement_bank_account_id` ou `settlement_account_id`, `settlement_journal_entry_id`).
9. Refus de paiement (erreur métier, pas 500) si : facture pas `open` (déjà payée/annulée), virement avec `bank_account.journal_account_id` non configuré, virement/compte interne dont le compte cible n'appartient pas à la société ou est inactif, date de règlement hors exercice ouvert.
10. Les coordonnées de paiement (IBAN/QR-IBAN/référence) sont **purement informatives** en 12-2 et **n'influencent pas** le choix du mode ni l'écriture (orthogonalité). Elles sont stockées pour consommation par 12-3.

**CRUD / liste / cohérence**

11. Liste paginée des factures fournisseurs (scopée société, anti-IDOR), triée par date décroissante, montrant n°, fournisseur, date, échéance, TTC, statut.
12. Détail d'une facture fournisseur : en-tête + lignes + liens « voir l'écriture d'achat » et (si payée) « voir l'écriture de règlement ». Action « Payer » visible uniquement si `open`.
13. Annulation possible d'une facture `open` (statut `cancelled`) : contre-passe l'écriture d'achat (réutilise le helper de contre-passation, montants positifs jamais négatifs) — **DC à trancher** (cf. DC7). Une facture `paid` ne peut être annulée directement.
14. Toutes les mutations (enregistrement, paiement, annulation) écrivent un **audit log** (`supplier_invoice.created` / `.paid` / `.cancelled`).
15. **i18n** : libellés FR + clés FTL ajoutées (FR/DE/IT/EN) au moins pour le PDF / titres ; fallbacks FR dans le code pour le reste (pattern 12-1). Navigation : item sidebar « Factures fournisseurs ».
16. **Tests** : intégration repo (enregistrement + paiement binaire 2 branches + refus), unitaires helper de génération d'écritures, E2E parcours saisie→paiement, quality gate vert (workspace serial + vitest + svelte-check + lint i18n + build).

## Points de décision (DC) — à figer au `validate`

- **DC1 — Compte créanciers par défaut** : ajouter `company_invoice_settings.default_payable_account_id` (FK accounts, créanciers 2000) **+ backfill migration** (créer/lier 2000 si plan comptable présent, idempotent, pattern story 18-1a pour 1171/2206). *Proposition : OUI, c'est le pendant achat de `default_receivable_account_id`.*
- **DC2 — Granularité TVA de l'écriture d'achat** : agrégation `D 1171` unique (somme de toutes les TVA récupérables) ou une ligne 1171 par taux ? *Proposition : 1171 unique agrégé (un seul compte d'impôt préalable existe, cf. Epic 18). Cohérent avec contrainte `chk_jel_*` (montants positifs, débit/crédit exclusif).*
- **DC3 — Modèle de lignes** : multi-lignes (description + HT + taux + compte de charge par ligne) vs ligne unique simplifiée. *Proposition : multi-lignes (correction comptable réelle), table `supplier_invoice_lines` miroir de `credit_note_lines`.*
- **DC4 — Numérotation** : pas de séquence interne auto (≠ 12-1) — le n° est celui **du fournisseur** (texte libre optionnel). L'id interne suffit. *Proposition : pas de table de séquence.* À confirmer (ou n° interne de suivi `FF-{YEAR}-{SEQ}` ?).
- **DC5 — Règlement virement immédiat vs différé** : en 12-2, le virement poste l'écriture et marque `paid` immédiatement (le flux deux-temps pain.001 est 12-3). *Proposition : immédiat en 12-2 ; 12-3 introduira un statut intermédiaire `payment_pending` pour les lots pain.001.*
- **DC6 — Champ coordonnées de paiement** : un seul couple IBAN/QR-IBAN + référence sur l'en-tête (pas par ligne). Validation IBAN/QR-IBAN via `kesh-qrbill::validation` (réutilisé), mais **non bloquante** en 12-2 (informatif). *Proposition : stocker tel quel, valider format si fourni, ne pas exiger.*
- **DC7 — Annulation** : une facture `open` annulée doit-elle contre-passer l'écriture d'achat (comme 12-1 annule la facture vente) ou simplement marquer `cancelled` en laissant l'écriture (puis OD manuelle) ? *Proposition : contre-passation automatique (cohérence avec 12-1), helper dédié réutilisant le swap débit/crédit. Montants jamais négatifs (contrainte `chk_jel_*_nonneg`).*
- **DC8 — Réutilisation crate `kesh-payment`** : la crate `kesh-payment` est vide. Y poser le helper de génération d'écritures d'achat/règlement, ou rester dans `kesh-db`/`kesh-core` ? *Proposition : logique de calcul pure (génération de lignes) dans `kesh-core::accounting` (à côté de `vat.rs`/`balance.rs`), persistance dans `kesh-db::repositories::supplier_invoices` (pattern 12-1). Réserver `kesh-payment` pour le pain.001 (12-3).*

## Tasks / Subtasks

- [ ] **T1 — Migration & schéma DB** (AC: 1,2,3,5,8,10,11,14 ; DC1,DC3,DC4,DC6)
  - [ ] Migration `supplier_invoices` (en-tête : company_id FK, contact_id FK, supplier_invoice_number VARCHAR NULL, status VARCHAR(16) CHECK('open','paid','cancelled') DEFAULT 'open', invoice_date DATE, due_date DATE NULL, total_amount DECIMAL(19,4) TTC, creditor_iban VARCHAR(34) NULL, creditor_qr_iban VARCHAR(34) NULL, payment_reference VARCHAR(64) NULL, purchase_journal_entry_id BIGINT FK, settlement_type VARCHAR(20) NULL CHECK('bank_transfer','internal_account'), settlement_bank_account_id BIGINT NULL FK, settlement_account_id BIGINT NULL FK, settlement_journal_entry_id BIGINT NULL FK, paid_at DATETIME(3) NULL, version, created_at, updated_at).
  - [ ] CHECK métier cohérence statut↔paiement : `status <> 'paid' OR (settlement_journal_entry_id IS NOT NULL AND paid_at IS NOT NULL AND settlement_type IS NOT NULL)`.
  - [ ] Migration `supplier_invoice_lines` (miroir `credit_note_lines` : position, description, quantity DECIMAL>0, unit_price DECIMAL>=0, vat_rate DECIMAL(5,2) 0-100, line_total DECIMAL>=0 HT, expense_account_id BIGINT FK, UNIQUE(supplier_invoice_id, position)).
  - [ ] Migration ajout `company_invoice_settings.default_payable_account_id BIGINT NULL` FK RESTRICT (DC1) + **backfill idempotent** compte 2000 créanciers (pattern `20260614000001_vat_accounts_config.sql`).
  - [ ] **Idempotence** : ajouter les nouvelles migrations à `docs/migrations-idempotence-audit.md` (garde-fou P5 CLAUDE.md). Migrations **non-breaking** (ADD COLUMN / CREATE TABLE) → pas de bump `kesh_version_min_required` (P3).
- [ ] **T2 — Entités + repository (enregistrement)** (AC: 1-6 ; DC2,DC3,DC8)
  - [ ] Entités `SupplierInvoice`, `SupplierInvoiceLine`, `NewSupplierInvoice` (`crates/kesh-db/src/entities/supplier_invoice.rs`), enregistrer dans `entities/mod.rs`.
  - [ ] Helper pur `generate_purchase_journal_lines(lines, payable_account_id, recoverable_account_id)` dans `kesh-core::accounting` → `Vec<NewJournalEntryLine>` : D charge par compte (HT agrégé), D 1171 (Σ TVA), C 2000 (TTC). Réutilise `vat::line_vat_amount`. Équilibre garanti.
  - [ ] `supplier_invoices::create(pool, new, user_id)` single-step transactionnel (pattern `credit_notes::create_credit_note`) : valide fournisseur/lignes/exercice, résout `default_payable_account_id` + `default_vat_recoverable_account_id`, génère lignes, poste l'écriture via `journal_entries::create_in_tx`, INSERT facture+lignes, audit `supplier_invoice.created`.
- [ ] **T3 — Repository (règlement binaire + annulation)** (AC: 7,8,9,13,14 ; DC5,DC7)
  - [ ] `supplier_invoices::pay(pool, id, company_id, settlement_choice, user_id)` : verrou facture `FOR UPDATE`, garde statut `open`, résout contrepartie selon `bank_transfer` (→ `bank_account.journal_account_id`, garde non-null + company-scoped) ou `internal_account` (→ account company-scoped actif), poste `D 2000 / C contrepartie` via `create_in_tx`, UPDATE statut `paid` + champs règlement + `paid_at`, audit `supplier_invoice.paid`.
  - [ ] `supplier_invoices::cancel(...)` (DC7) : contre-passation écriture d'achat, statut `cancelled`, audit `supplier_invoice.cancelled`.
  - [ ] `supplier_invoices::{get,list}` scopés société, pagination (pattern credit_notes).
- [ ] **T4 — Routes API** (AC: 7,9,11,12,14,15)
  - [ ] `crates/kesh-api/src/routes/supplier_invoices.rs` : `POST /api/v1/supplier-invoices` (créer, Comptable+), `POST /api/v1/supplier-invoices/{id}/pay` (régler, Comptable+), `POST /api/v1/supplier-invoices/{id}/cancel` (Comptable+), `GET /api/v1/supplier-invoices` + `GET /{id}` (lecture, tous rôles auth). Structs request/response camelCase. Montage dans `lib.rs` (comptable_routes + authenticated_routes), `routes/mod.rs`.
  - [ ] FailedX pattern non requis (endpoints unitaires, pas batch) — erreurs métier en `AppError` typées (pas 500) conformément AC5/AC9.
- [ ] **T5 — Frontend feature + pages** (AC: 1,7,11,12,15)
  - [ ] Feature `frontend/src/lib/features/supplier-invoices/` (`supplier-invoices.api.ts`, `.types.ts`, helpers + tests) — pattern `credit-notes`.
  - [ ] Pages `frontend/src/routes/(app)/supplier-invoices/+page.svelte` (liste) + `[id]/+page.svelte` (détail + action Payer : sélecteur binaire virement[compte bancaire] / compte interne[compte plan comptable]).
  - [ ] Formulaire d'enregistrement (lignes : compte de charge + HT + taux TVA via `getVatRates()`, coordonnées paiement optionnelles). Réutilise le pattern de `VatPurchaseAssistant.svelte` / `vat-purchase.ts` (`buildPurchaseVatLines`, `lineVatAmount`) pour la prévisualisation de l'écriture.
  - [ ] Câblage sidebar `+layout.svelte` (groupe « Quotidien », item « Factures fournisseurs », testid `nav-link-supplier-invoices`).
- [ ] **T6 — i18n + tests + doc** (AC: 15,16)
  - [ ] Clés FTL FR/DE/IT/EN (au moins titres) + fallbacks FR dans le code.
  - [ ] Tests intégration repo (`crates/kesh-db/tests/supplier_invoices_repository.rs`) : enregistrement + paiement virement + paiement compte interne + refus (config absente, déjà payée, compte étranger). Unitaires helper `generate_purchase_journal_lines` (multi-taux). E2E (`frontend/tests/e2e/supplier-invoices.spec.ts`) : saisie → paiement.
  - [ ] Doc : CHANGELOG `[Non publié]` Added, README feuille de route, `docs/user-guide/fr/getting-started.md` (section achats fournisseurs), manuel utilisateur si pertinent.
  - [ ] **Quality gate** Test Locally First (backend 4 checks + frontend 4 checks + E2E), workspace **serial** (`-j1 --test-threads=1`) car migrations DB.

## Découpage / split candidate

**Story umbrella — CANDIDATE SPLIT 12-2a..f** (scope cross-stack > 5 modules : `kesh-db` migrations+entities+repo, `kesh-core` accounting, `kesh-api` routes, `frontend` feature+pages+sidebar, i18n, tests). Conformément à la **Règle de splitting préventif** (CLAUDE.md) et au pattern des épopées 12-1/17-3/17-4/18-1, le `validate` tranchera le split. Proposition :

- **12-2a** — Fondation DB (migrations supplier_invoices/_lines + `default_payable_account_id` + backfill 2000 + audit idempotence) — story-zéro.
- **12-2b** — Backend enregistrement (entités + helper `generate_purchase_journal_lines` kesh-core + `create` single-step + tests repo).
- **12-2c** — Backend règlement binaire + annulation (`pay` / `cancel` + tests 2 branches).
- **12-2d** — Routes API (POST create/pay/cancel + GET list/detail + montage lib.rs).
- **12-2e** — Frontend (feature + pages liste/détail + formulaire + sélecteur binaire + sidebar).
- **12-2f** — i18n + E2E + doc-sync + quality gate final.

## Dev Notes

### Réutilisation Epic 18 (comptabilisation achat TVA)

- `kesh-core/src/accounting/vat.rs:39` — `pub fn line_vat_amount(base_ht: Decimal, rate_percent: Decimal) -> Decimal` (half-up 2 déc., taux en **pourcent**). **Helper canonique à réutiliser** pour le calcul de TVA par ligne.
- `frontend/src/lib/features/journal-entries/vat-purchase.ts:36,73` — `lineVatAmount()` / `buildPurchaseVatLines()` (3 lignes si TVA>0, 2 si exempt : D charge / D 1171 / C contrepartie TTC). **Parité frontend** à réutiliser pour la prévisualisation.
- `frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte` — composant assistant achat (charge/HT/taux/contrepartie) — modèle pour le formulaire d'enregistrement.
- `kesh-core/src/accounting/balance.rs:144` — `validate(JournalEntryDraft) -> Result<BalancedEntry, CoreError>` (équilibre partie double).
- `kesh-db/src/repositories/journal_entries.rs:55,90` — `create()` / `create_in_tx()` (lock exercice `FOR UPDATE`, vérif comptes actifs+company-owned, numéro séquentiel, re-validation balance, audit log). **Poster l'écriture via `create_in_tx` dans la même transaction.**
- Contraintes DB partie double (`migrations/20260412000001_journal_entries.sql:46-48`) : `chk_jel_debit_credit_exclusive`, `chk_jel_debit_nonneg`, `chk_jel_credit_nonneg`. **→ jamais de montants négatifs ; contre-passation par swap débit↔crédit** (cf. 12-1 DC7).
- Settings TVA (`migrations/20260614000001_vat_accounts_config.sql`, `entities/company_invoice_settings.rs:26-30`) : `default_vat_recoverable_account_id` (1171, impôt préalable, **Asset**) — utilisé pour le D 1171 de l'écriture d'achat. ⚠️ NE PAS confondre 1170 « impôt anticipé / withholding » ≠ 1171 « impôt préalable / TVA récupérable » (DC1 Epic 18).
- `company_invoice_settings::get_or_create_default_in_tx(tx, company_id)` — accès config lazy en transaction.

### Réutilisation pattern 12-1 (Avoirs — template d'implémentation)

- Migration : `migrations/20260627000001_credit_notes.sql` — modèle de table en-tête + lignes + (séquence). Cohérence statut↔écriture via CHECK (`status <> 'issued' OR (number IS NOT NULL AND journal_entry_id IS NOT NULL)`) → adapter pour `paid`.
- `kesh-db/src/repositories/credit_notes.rs:198` — `create_credit_note()` single-step transactionnel (10 étapes atomiques). **Modèle direct** pour `supplier_invoices::create()` et `::pay()`.
- `kesh-db/src/repositories/credit_notes.rs:139` — `generate_credit_note_journal_lines()` (swap débit↔crédit, montants positifs, ordre par taux). **Modèle** pour `generate_purchase_journal_lines()` et la contre-passation d'annulation (DC7).
- `kesh-db/src/repositories/credit_note_number_sequences.rs` — `next_number_for(tx, company_id, fiscal_year_id)` (no-gap `FOR UPDATE` lazy-insert) — **probablement non requis** (DC4 : pas de séquence interne).
- Routes : `kesh-api/src/routes/credit_notes.rs` + montage `lib.rs:230` (comptable_routes) / `lib.rs:386` (authenticated_routes). RBAC `require_comptable_role` pour mutations.
- Frontend : `frontend/src/lib/features/credit-notes/{credit-notes.api.ts,.types.ts,credit-note-helpers.ts}` + `routes/(app)/credit-notes/{+page.svelte,[id]/+page.svelte}` + sidebar `+layout.svelte:63`.
- i18n : leçon 12-1 — **ajouter les clés FTL dès l'implémentation** (12-1 n'a posé que 2 clés PDF, le reste en fallback FR ; on fait mieux ici).

### État existant à respecter (UPDATE files)

- `crates/kesh-db/src/entities/contact.rs:72-88` — `Contact { is_client, is_supplier, ide_number, default_payment_terms }`. **Un fournisseur se crée déjà** via CRUD contacts existant en posant `is_supplier=true`. AC1 doit filtrer/valider `is_supplier`.
- `crates/kesh-db/src/entities/bank_account.rs:31` — `journal_account_id: Option<i64>` (lien compte grand livre). **Contrepartie du virement = ce champ ; garde non-null obligatoire (AC9).**
- `crates/kesh-db/src/entities/company_invoice_settings.rs:20-38` — config 1-1 par société (PK company_id). Ajouter `default_payable_account_id` (DC1). Lazy-create via `get_or_create_default`.
- `crates/kesh-qrbill/src/validation.rs:35,67` — `validate_iban()` / `validate_qr_iban()` (mod-97, CH/LI, QR-IID 30000-31999). Réutiliser si AC1 valide les coordonnées (DC6, non bloquant).
- `crates/kesh-db/src/entities/invoice.rs:22,30` — machine à états facture **vente** (`draft/validated/cancelled` + `paid_at`). ⚠️ **NE PAS** réutiliser/altérer la table `invoices` : les factures fournisseurs sont une **entité distincte** (`supplier_invoices`). `mark_as_paid` des ventes ne poste PAS d'écriture (réconciliation Epic 6) — à l'inverse, le règlement fournisseur **poste** l'écriture (différence assumée).
- `kesh-payment` crate = **vide** ; ne pas y mettre la logique 12-2 (réservée pain.001 / 12-3, DC8).

### Project Structure Notes

- Nouvelle entité distincte `supplier_invoices` (≠ `invoices` ventes) — pas de colonne `invoice_type` ajoutée à `invoices` (la suggestion d'archi du rapport d'exploration est écartée : entité dédiée plus propre, isole le RBAC, la numérotation et le flux paiement).
- Logique de calcul pure (génération lignes) dans `kesh-core::accounting` (testable sans DB) ; persistance/transaction dans `kesh-db::repositories::supplier_invoices` ; HTTP dans `kesh-api::routes::supplier_invoices`. Aligné sur la séparation Epic 18 / 12-1.
- Migrations non-breaking (CREATE TABLE / ADD COLUMN nullable) → conformes politique migration (P1-P5 CLAUDE.md), audit idempotence à compléter.

### Conventions projet à respecter

- **Test Locally First** (CLAUDE.md) avant tout push : backend `fmt/build/clippy/test`, frontend `check/lint-i18n/test:unit/build`, E2E si frontend touché. Tests DB en **serial** (`-j1 --test-threads=1`).
- **Issue Tracking** : story rattachée à #191 (ne pas fermer #191 — 12-3/12-4 restent). Commit `(refs #191)`.
- **Branche** : `story/12-2-factures-fournisseurs-reglement` (déjà créée). Commit après chaque étape BMAD.
- **Backup/export** : nouvelles tables `supplier_invoices` / `supplier_invoice_lines` → **maj `TABLES_TO_TRUNCATE` (backup/import) + manifeste export** (leçon 12-1, cf. `project_12_1_avoirs_pr.md`). Vérifier `crates/kesh-db/src/backup.rs` + `kesh-api/src/admin_backup/` + `exports/`.
- **Pattern batch FailedProposal** : non applicable (endpoints unitaires).

### References

- [Source: GitHub issue #191] — spec produit réalignée binaire 2026-06-27.
- [Source: memory project_epic_12_supplier_invoices_design.md] — décisions Guy figées.
- [Source: crates/kesh-core/src/accounting/vat.rs:39] — `line_vat_amount`.
- [Source: crates/kesh-db/src/repositories/journal_entries.rs:90] — `create_in_tx`.
- [Source: crates/kesh-db/src/repositories/credit_notes.rs:139,198] — `generate_credit_note_journal_lines`, `create_credit_note`.
- [Source: crates/kesh-db/migrations/20260614000001_vat_accounts_config.sql] — pattern backfill compte + settings TVA.
- [Source: crates/kesh-db/src/entities/{contact.rs:72,bank_account.rs:31,company_invoice_settings.rs:20}] — entités existantes à respecter.
- [Source: crates/kesh-qrbill/src/validation.rs:35,67] — validation IBAN/QR-IBAN.

## Dev Agent Record

### Agent Model Used

(à remplir au dev-story — Opus 4.8 recommandé : scope cross-stack + comptable)

### Debug Log References

### Completion Notes List

### File List
