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
2. À l'enregistrement, le système **poste atomiquement l'écriture d'achat** réutilisant la logique TVA Epic 18 : **une ligne `D compte de charge (HT)` par ligne de facture** (pas d'agrégation inter-lignes, même si deux lignes partagent le même compte — parité avec `buildPurchaseVatLines` appelé ligne par ligne) + **une seule ligne `D impôt préalable 1171` agrégée** (Σ TVA récupérable, DC2) + `C créanciers 2000 (TTC)`. **La ligne D 1171 est OMISE si Σ TVA = 0** (facture 100% exempte) — la contrainte `chk_jel_debit_credit_exclusive` interdit toute ligne à zéro [H2 P1]. L'écriture utilise `entry_date = invoice_date`. Elle est équilibrée par construction (`Σdébit = Σcrédit`). Statut initial = **`open`**.
3. La facture fournisseur référence l'écriture d'achat créée (`purchase_journal_entry_id`) et son TTC (`total_amount`).
4. Le compte de charge et le taux TVA par ligne sont libres (multi-lignes, multi-taux supportés) ; le montant de TVA par taux est calculé via le helper canonique `kesh-core::accounting::vat::line_vat_amount` (arrondi half-up 2 décimales, taux en pourcent).
5. Le compte **créanciers** (2000) est résolu via un **compte par défaut configuré** au niveau société (nouveau `company_invoice_settings.default_payable_account_id`). Ce compte est **auto-lié au compte 2000 existant** par un UPDATE de backfill à la migration (le compte 2000 « Créanciers » est déjà dans les charts d'origine — cf. `pme.json` — donc on le **lie**, on ne le **crée pas**). Si absent malgré tout, l'enregistrement échoue avec une erreur métier explicite (`ConfigurationRequired`), pas un 500.
6. Validation : refus si fournisseur n'est pas `is_supplier`, si une ligne a un montant HT ≤ 0, si un compte de charge n'existe pas / n'appartient pas à la société, si la date tombe hors d'un exercice ouvert.

**Règlement binaire (action « Payer »)**

7. Un Comptable+ peut **payer** une facture fournisseur au statut `open` via un choix binaire, avec une **date de règlement** (`paymentDate`, fournie par le client — c'est la date de valeur, pas la date de saisie) [H1 P1] :
   - **virement** : `{ type: "bank_transfer", bankAccountId, paymentDate }` → contrepartie = `bank_account.journal_account_id` du compte choisi ;
   - **compte interne** : `{ type: "internal_account", accountId, paymentDate }` → contrepartie = ce compte du plan comptable.
8. Le paiement **poste atomiquement l'écriture de règlement** `D créanciers 2000 (TTC) / C contrepartie (TTC)` avec `entry_date = paymentDate`, passe la facture au statut **`paid`**, horodate `paid_at = NOW(3)`, et enregistre le type/compte de règlement utilisés (`settlement_type`, `settlement_bank_account_id` ou `settlement_account_id`, `settlement_journal_entry_id`).
9. Refus de paiement (erreur métier, pas 500) si : facture pas `open` (déjà payée/annulée), virement avec `bank_account.journal_account_id` non configuré, virement/compte interne dont le compte cible n'appartient pas à la société ou est inactif, `paymentDate` hors exercice ouvert.
10. Les coordonnées de paiement (IBAN/QR-IBAN/référence) sont **purement informatives** en 12-2 et **n'influencent pas** le choix du mode ni l'écriture (orthogonalité). Elles sont stockées pour consommation par 12-3.

**CRUD / liste / cohérence**

11. Liste paginée des factures fournisseurs (scopée société, anti-IDOR), triée par date décroissante, montrant n°, fournisseur, date, échéance, TTC, statut.
12. Détail d'une facture fournisseur : en-tête + lignes + liens « voir l'écriture d'achat » et (si payée) « voir l'écriture de règlement ». Action « Payer » visible uniquement si `open`.
13. Annulation possible d'une facture `open` (statut `cancelled`) : contre-passe l'écriture d'achat par relecture de ses lignes + swap D↔C (montants positifs jamais négatifs, cf. DC7 FIGÉ). Une facture `paid` ne peut être annulée directement.
14. Toutes les mutations (enregistrement, paiement, annulation) écrivent un **audit log** (`supplier_invoice.created` / `.paid` / `.cancelled`).
15. **i18n** : libellés FR + clés FTL ajoutées (FR/DE/IT/EN) au moins pour le PDF / titres ; fallbacks FR dans le code pour le reste (pattern 12-1). Navigation : item sidebar « Factures fournisseurs ».
16. **Tests** : intégration repo (enregistrement + paiement virement + paiement compte interne + **annulation open→cancelled** + refus : config absente, déjà payée, **payer une facture `cancelled`**, **annuler une facture `paid`**, compte étranger), unitaires helper de génération d'écritures (dont cas Σ TVA = 0 → 1171 omise), E2E parcours saisie→paiement, quality gate vert (workspace serial + vitest + svelte-check + lint i18n + build).

## Points de décision (DC) — à figer au `validate`

- **DC1 — Compte créanciers par défaut** [FIGÉ — corrigé M1] : ajouter `company_invoice_settings.default_payable_account_id` (FK accounts RESTRICT) **+ backfill UPDATE** liant au compte 2000 existant. ⚠️ Le compte 2000 « Créanciers » est **déjà** dans les charts d'origine (`pme.json`/`independant.json`/`association.json`) — donc **PAS d'INSERT** (≠ 18-1a qui créait 1171/2206 nouveaux). Migration : `ADD COLUMN nullable + FK`, puis `UPDATE company_invoice_settings cis INNER JOIN accounts a ON a.company_id = cis.company_id AND a.number = '2000' SET cis.default_payable_account_id = a.id WHERE cis.default_payable_account_id IS NULL`. Auto-configure toutes les companies **existantes** onboardées. Pour les **nouvelles** companies (post-migration), l'auto-config passe par l'extension de `insert_with_defaults[_in_tx]` qui résout aussi 2000 (cf. touch-points T1 / N-M1 Option A) — même mécanisme que 1100/3000.
- **DC2 — Granularité TVA de l'écriture d'achat** [FIGÉ] : agrégation `D 1171` **unique** (somme de toutes les TVA récupérables, un seul compte d'impôt préalable existe, cf. Epic 18). **Ligne D 1171 OMISE si Σ TVA = 0** (sinon violation `chk_jel_debit_credit_exclusive`, cf. H2). Cohérent avec contraintes `chk_jel_*` (montants positifs, débit/crédit exclusif).
- **DC3 — Modèle de lignes** : multi-lignes (description + HT + taux + compte de charge par ligne) vs ligne unique simplifiée. *Proposition : multi-lignes (correction comptable réelle), table `supplier_invoice_lines` miroir de `credit_note_lines`.*
- **DC4 — Numérotation** : pas de séquence interne auto (≠ 12-1) — le n° est celui **du fournisseur** (texte libre optionnel). L'id interne suffit. *Proposition : pas de table de séquence.* À confirmer (ou n° interne de suivi `FF-{YEAR}-{SEQ}` ?).
- **DC5 — Règlement virement immédiat vs différé** : en 12-2, le virement poste l'écriture et marque `paid` immédiatement (le flux deux-temps pain.001 est 12-3). *Proposition : immédiat en 12-2 ; 12-3 introduira un statut intermédiaire `payment_pending` pour les lots pain.001.*
- **DC6 — Champ coordonnées de paiement** : un seul couple IBAN/QR-IBAN + référence sur l'en-tête (pas par ligne). Validation IBAN/QR-IBAN via `kesh-qrbill::validation` (réutilisé), mais **non bloquante** en 12-2 (informatif). *Proposition : stocker tel quel, valider format si fourni, ne pas exiger.*
- **DC7 — Annulation** [FIGÉ — précisé M8] : contre-passation **automatique** de l'écriture d'achat (cohérence avec 12-1). **Source des lignes = approche (B) : relire les `journal_entry_lines` de `purchase_journal_entry_id`** (`SELECT account_id, debit, credit FROM journal_entry_lines WHERE journal_entry_id = ?`) et créer des `NewJournalEntryLine` avec swap `debit ↔ credit`. *Plus robuste que recalculer via les settings (qui ont pu changer depuis la création).* Montants jamais négatifs (contrainte `chk_jel_*_nonneg`). Description : `« Annulation facture fournisseur {n°|id} »`. Annulation interdite si statut ≠ `open`.
- **DC8 — Emplacement du helper de génération d'écritures** [FIGÉ — CORRIGÉ O-C1 passe 3] : le helper `generate_purchase_journal_lines` vit dans **`kesh-db::repositories::supplier_invoices`** (exactement comme `generate_credit_note_journal_lines` dans `credit_notes.rs:139`), car son type de retour `Vec<NewJournalEntryLine>` et `DbError` sont définis dans `kesh-db`. ⚠️ **PAS dans `kesh-core`** : `kesh-core` ne dépend pas de `kesh-db` (dépendance inverse `kesh-db → kesh-core`) ; y référencer `NewJournalEntryLine`/`DbError` créerait une **dépendance circulaire Cargo → échec de compilation**. Seul `kesh_core::accounting::vat::line_vat_amount` (calcul `Decimal` pur) est appelé depuis kesh-core. La crate `kesh-payment` (vide) reste réservée au pain.001 (12-3).

## Tasks / Subtasks

- [ ] **T1 — Migration & schéma DB** (AC: 1,2,3,5,8,10,11,14 ; DC1,DC3,DC4,DC6)
  - [ ] Migration `supplier_invoices` (en-tête : company_id FK, contact_id FK, supplier_invoice_number VARCHAR NULL, status VARCHAR(16) CHECK('open','paid','cancelled') DEFAULT 'open', invoice_date DATE, due_date DATE NULL, total_amount DECIMAL(19,4) TTC, creditor_iban VARCHAR(34) NULL, creditor_qr_iban VARCHAR(34) NULL, payment_reference VARCHAR(64) NULL, **expected_payment_amount DECIMAL(19,4) NULL** [M1-P2 — colonne pour le « montant attendu » d'AC1, consommé par la réconciliation 12-3], purchase_journal_entry_id BIGINT FK, settlement_type VARCHAR(20) NULL CHECK('bank_transfer','internal_account'), settlement_bank_account_id BIGINT NULL FK, settlement_account_id BIGINT NULL FK, settlement_journal_entry_id BIGINT NULL FK, paid_at DATETIME(3) NULL, version, created_at, updated_at).
  - [ ] CHECK métier cohérence statut↔paiement : `status <> 'paid' OR (settlement_journal_entry_id IS NOT NULL AND paid_at IS NOT NULL AND settlement_type IS NOT NULL)`.
  - [ ] Migration `supplier_invoice_lines` (miroir `credit_note_lines` : position, description, quantity DECIMAL>0, **unit_price DECIMAL>0** [L1], vat_rate DECIMAL(5,2) 0-100, **line_total DECIMAL CHECK(line_total > 0)** HT [L1 — cohérent AC6], expense_account_id BIGINT FK, UNIQUE(supplier_invoice_id, position)).
  - [ ] Migration ajout `company_invoice_settings.default_payable_account_id BIGINT NULL` FK RESTRICT (DC1) — **ADD COLUMN seul, PAS d'INSERT 2000** (existe déjà) + **UPDATE backfill liant au compte 2000 existant** par company (cf. DC1, SQL exact). Idempotent (`WHERE default_payable_account_id IS NULL`).
  - [ ] **Extension `company_invoice_settings` — TOUS les touch-points** [M5/M6 + O-M1 passe 3] ; en oublier un = bug silencieux (pas un échec de compile) :
    - `repositories/company_invoice_settings.rs` : constante `COLUMNS`, `settings_snapshot_json`, les deux `get_or_create_default[_in_tx]`, **`is_no_op_change()`** (sinon un PUT ne modifiant que ce champ est silencieusement ignoré, version non bumpée), **l'`UPDATE … SET` SQL + le bind** de `update()` (sinon jamais persisté), **et les deux `insert_with_defaults[_in_tx]` (l.299/403)** [N-M1 passe 4, Option A] : étendre le SELECT+INSERT pour résoudre le compte 2000 (`SELECT id FROM accounts WHERE company_id = ? AND number = '2000' … FOR UPDATE`) et le binder, à l'image de 1100/3000 — sinon les **nouvelles** companies (onboarding post-migration, cf. `onboarding.rs:696`) auraient `default_payable_account_id = NULL` (incohérent avec l'auto-config créances/produits). Pattern MIRROR à garder synchronisé entre les 2 variantes.
    - `entities/company_invoice_settings.rs` : struct `CompanyInvoiceSettings` + struct `CompanyInvoiceSettingsUpdate`.
    - `routes/company_invoice_settings.rs` : DTO `UpdateInvoiceSettingsRequest`, DTO `InvoiceSettingsResponse` (+ son `From` impl), et dans le handler PUT un appel **`validate_account(..., AccountType::Liability, …)`** (le compte 2000 est de type **Liability**, confirmé dans les 3 chartes) pour valider existence/scope/type/actif.
    - **4 tests existants** `tests/company_invoice_settings_repository.rs` (~l.229/289/371/453) construisent `CompanyInvoiceSettingsUpdate { … }` → **à mettre à jour sinon CI rouge** (échec de compile).
  - [ ] **Idempotence** : ajouter les nouvelles migrations à `docs/migrations-idempotence-audit.md` (garde-fou P5 CLAUDE.md). Migrations **non-breaking** (ADD COLUMN / CREATE TABLE) → pas de bump `kesh_version_min_required` (P3).
  - [ ] **Backup** [L3-P2] : ajouter `supplier_invoice_lines` puis `supplier_invoices` (ordre enfants→parents) à `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs`) — round-trip `.keshbackup` (cf. Dev Notes §Backup/export). PAS d'ajout au global ZIP export.
- [ ] **T2 — Entités + repository (enregistrement)** (AC: 1-6 ; DC2,DC3,DC8)
  - [ ] Entités `SupplierInvoice`, `SupplierInvoiceLine`, `NewSupplierInvoice` (`crates/kesh-db/src/entities/supplier_invoice.rs`), enregistrer dans `entities/mod.rs`.
  - [ ] Helper `generate_purchase_journal_lines(lines: &[(Decimal /*HT*/, Decimal /*taux*/, i64 /*expense_account_id*/)], payable_account_id: i64, recoverable_account_id: Option<i64>) -> Result<Vec<NewJournalEntryLine>, DbError>` dans **`kesh-db::repositories::supplier_invoices`** (PAS kesh-core — cf. DC8/O-C1, dépendance circulaire) : **une ligne D par ligne de facture** (pas d'agrégation inter-lignes [M2]), **une ligne D 1171 unique agrégée Σ TVA OMISE si Σ=0** [H2], C 2000 (TTC). Retourne `Err(ConfigurationRequired("default_vat_recoverable_account_id"))` **seulement si** `recoverable_account_id.is_none() && Σ TVA > 0` [M7]. Appelle `kesh_core::accounting::vat::line_vat_amount` (seul morceau kesh-core). Équilibre garanti.
  - [ ] `supplier_invoices::create(pool, new, user_id)` single-step transactionnel (pattern `credit_notes::create_credit_note` qui **ouvre la tx en interne via `pool.begin()`** puis passe `&mut tx` à `create_in_tx` — la signature publique prend `pool`, l'atomicité est gérée dedans) : valide fournisseur (`is_supplier`)/lignes/comptes de charge company-scoped, **résout l'exercice via `fiscal_years::find_open_covering_date(&mut tx, company_id, invoice_date)?.ok_or(FiscalYearInvalid)?`** [M4], résout `default_payable_account_id` + `default_vat_recoverable_account_id`, **calcule `total_amount = Σ line_total (HT) + Σ TVA = la ligne C 2000 (TTC) du helper, sans réarrondir** [O-M2], génère lignes, poste l'écriture (`entry_date = invoice_date` [L5], `journal = Journal::Achats` [M3], description `« Facture fournisseur {n°|id} - {contact} »` [L4]) via `journal_entries::create_in_tx`, INSERT facture+lignes, audit `supplier_invoice.created`, commit.
- [ ] **T3 — Repository (règlement binaire + annulation)** (AC: 7,8,9,13,14 ; DC5,DC7)
  - [ ] `supplier_invoices::pay(pool, id, company_id, settlement_choice, payment_date, user_id)` [H1] : verrou facture `FOR UPDATE`, garde statut `open`, **résout l'exercice via `find_open_covering_date(&mut tx, company_id, payment_date)`** [M4], résout contrepartie selon `bank_transfer` (→ `bank_account.journal_account_id`, garde non-null + company-scoped) ou `internal_account` (→ account company-scoped actif), poste `D 2000 / C contrepartie` **pour le montant `supplier_invoice.total_amount` (TTC stocké, = exactement le `C 2000` de l'écriture d'achat → solde 2000 ramené à 0)** [O-M2] (`entry_date = payment_date`, `journal = Journal::Banque` si virement / `Journal::OD` si compte interne [M3], description `« Règlement fournisseur {n°|id} - {contact} »` [L4]) via `create_in_tx` (qui rejette en erreur métier 4xx si un compte cible est inactif/archivé — edge attendu [O-L1/O-L2]), UPDATE statut `paid` + champs règlement + `paid_at = NOW(3)`, audit `supplier_invoice.paid`.
  - [ ] `supplier_invoices::cancel(...)` (DC7) : contre-passation écriture d'achat **par relecture des `journal_entry_lines` de `purchase_journal_entry_id` + swap D↔C** [M8] (PAS de recalcul via settings). **`entry_date` = date du jour** [L1-P2], résolue via `find_open_covering_date(&mut tx, company_id, today)` ; si aucun exercice ouvert ne couvre aujourd'hui → `Err(FiscalYearInvalid)` (erreur métier). Statut `cancelled`, audit `supplier_invoice.cancelled`. Refus si statut ≠ `open`.
  - [ ] `supplier_invoices::{get,list}` scopés société, pagination (pattern credit_notes).
- [ ] **T4 — Routes API** (AC: 7,9,11,12,14,15)
  - [ ] `crates/kesh-api/src/routes/supplier_invoices.rs` : `POST /api/v1/supplier-invoices` (créer, Comptable+), `POST /api/v1/supplier-invoices/{id}/pay` (régler, Comptable+), `POST /api/v1/supplier-invoices/{id}/cancel` (Comptable+), `GET /api/v1/supplier-invoices` + `GET /{id}` (lecture, tous rôles auth). Structs request/response camelCase. Montage dans `lib.rs` (comptable_routes + authenticated_routes), `routes/mod.rs`.
  - [ ] FailedX pattern non requis (endpoints unitaires, pas batch) — erreurs métier en `AppError` typées (pas 500) conformément AC5/AC9.
- [ ] **T5 — Frontend feature + pages** (AC: 1,7,11,12,15)
  - [ ] Feature `frontend/src/lib/features/supplier-invoices/` (`supplier-invoices.api.ts`, `.types.ts`, helpers + tests) — pattern `credit-notes`.
  - [ ] Pages `frontend/src/routes/(app)/supplier-invoices/+page.svelte` (liste) + `[id]/+page.svelte` (détail + action Payer : sélecteur binaire virement[compte bancaire] / compte interne[compte plan comptable]).
  - [ ] Formulaire d'enregistrement (lignes : compte de charge + HT + taux TVA via `getVatRates()`, coordonnées paiement optionnelles). Réutilise le pattern de `VatPurchaseAssistant.svelte` / `vat-purchase.ts` (`buildPurchaseVatLines`, `lineVatAmount`) pour la prévisualisation de l'écriture.
  - [ ] Câblage sidebar `+layout.svelte` (groupe « Quotidien », item « Factures fournisseurs » **inséré après « Avoirs » (`nav-credit-notes`) et avant « Importer »** [L2], testid `nav-link-supplier-invoices`).
- [ ] **T6 — i18n + tests + doc** (AC: 15,16)
  - [ ] Clés FTL FR/DE/IT/EN (au moins titres) + fallbacks FR dans le code.
  - [ ] Tests intégration repo (`crates/kesh-db/tests/supplier_invoices_repository.rs`) : enregistrement + paiement virement + paiement compte interne + **annulation (open→cancelled)** + refus (config absente, déjà payée, **payer une `cancelled`**, **annuler une `paid`**, compte étranger, exercice clos). Unitaires helper `generate_purchase_journal_lines` (multi-taux, **cas Σ TVA = 0 → ligne 1171 omise**, multi-lignes même compte → lignes D distinctes). **Test « TTC achat == TTC règlement → solde compte 2000 = 0 après paiement »** [O-M2]. E2E (`frontend/tests/e2e/supplier-invoices.spec.ts`) : saisie → paiement.
  - [ ] Doc : CHANGELOG `[Non publié]` Added, README feuille de route, `docs/user-guide/fr/getting-started.md` (section achats fournisseurs), manuel utilisateur si pertinent.
  - [ ] **Quality gate** Test Locally First (backend 4 checks + frontend 4 checks + E2E), workspace **serial** (`-j1 --test-threads=1`) car migrations DB.

## Découpage / split candidate

**Story umbrella — CANDIDATE SPLIT 12-2a..f** (scope cross-stack > 5 modules : `kesh-db` migrations+entities+repo, `kesh-core` accounting, `kesh-api` routes, `frontend` feature+pages+sidebar, i18n, tests). Conformément à la **Règle de splitting préventif** (CLAUDE.md) et au pattern des épopées 12-1/17-3/17-4/18-1, le `validate` tranchera le split. Proposition :

- **12-2a** — Fondation DB (migrations supplier_invoices/_lines + `default_payable_account_id` + backfill UPDATE 2000 + **extension COMPLÈTE `company_invoice_settings` : COLUMNS, snapshot, `is_no_op_change`, UPDATE SQL+bind, entité, Update struct, DTO Request/Response+From, `validate_account(Liability)` au PUT, 4 tests existants** [M5/M6/O-M1] + audit idempotence) — story-zéro.
- **12-2b** — Backend enregistrement (entités + helper `generate_purchase_journal_lines` **dans kesh-db** [O-C1, PAS kesh-core] + `create` single-step + tests unitaires helper [Σ TVA=0, multi-lignes même compte] + tests repo `create()` seul). *(Le test « solde 2000 = 0 après paiement » [O-M2] exige `pay()` → va en 12-2c/12-2f, pas ici [N-L1].)*
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
- `company_invoice_settings::get_or_create_default_in_tx(tx, company_id)` — accès config lazy en transaction. ⚠️ Repo a une constante `COLUMNS` + `settings_snapshot_json` à étendre pour `default_payable_account_id` (M5/M6).
- **Choix du journal** [M3] : écriture d'achat → `Journal::Achats` ; règlement virement → `Journal::Banque` ; règlement compte interne → `Journal::OD` (généraliste comptes non-bancaires : caisse, carte, Twint). Enum `Journal` : `entities/journal_entry.rs:33` (Achats/Ventes/Banque/Caisse/OD).
- **`NewJournalEntry`** (`entities/journal_entry.rs:166`) exige `entry_date: NaiveDate` + `description: String` + `journal: Journal` — tous fournis par le caller (pas de défaut `now()`).
- **Résolution exercice** : `fiscal_years::find_open_covering_date(&mut tx, company_id, date)` (cf. `credit_notes.rs:283`) → `None` = `DbError::FiscalYearInvalid` (erreur métier AC6/AC9). Appeler AVANT `create_in_tx` et passer le `fy.id`.

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
- Génération des lignes d'écriture + persistance/transaction dans `kesh-db::repositories::supplier_invoices` (le helper retourne `Vec<NewJournalEntryLine>`/`DbError`, types kesh-db → ne peut PAS être en kesh-core, cf. O-C1) ; seul `vat::line_vat_amount` (Decimal pur) est appelé depuis `kesh-core` ; HTTP dans `kesh-api::routes::supplier_invoices`. Aligné exactement sur la séparation 12-1 (`generate_credit_note_journal_lines` est en kesh-db).
- Migrations non-breaking (CREATE TABLE / ADD COLUMN nullable) → conformes politique migration (P1-P5 CLAUDE.md), audit idempotence à compléter.

### Conventions projet à respecter

- **Test Locally First** (CLAUDE.md) avant tout push : backend `fmt/build/clippy/test`, frontend `check/lint-i18n/test:unit/build`, E2E si frontend touché. Tests DB en **serial** (`-j1 --test-threads=1`).
- **Issue Tracking** : story rattachée à #191 (ne pas fermer #191 — 12-3/12-4 restent). Commit `(refs #191)`.
- **Branche** : `story/12-2-factures-fournisseurs-reglement` (déjà créée). Commit après chaque étape BMAD.
- **Backup/export** [précisé M9] : ajouter `supplier_invoice_lines` **AVANT** `supplier_invoices` dans `TABLES_TO_TRUNCATE` (`crates/kesh-db/src/backup.rs`, ordre enfants→parents) — le manifeste `.keshbackup` est auto-généré depuis cette liste. ⚠️ **NE PAS** ajouter au global ZIP export (`kesh-api/src/exports/`) : 12-1 (credit_notes) n'y est PAS non plus — l'export métier ZIP est un sous-ensemble figé, le backup `.keshbackup` est le mécanisme complet. Vérifier la cohérence dans `kesh-api/src/admin_backup/` (round-trip export/import).
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

## Change Log (validate)

Boucle de validation adversariale (Review Iteration Rule, CLAUDE.md) — rotation LLM, contexte frais, patches appliqués entre passes, ground-truth grep obligatoire.

- **Passe 1 — Sonnet 4.6** : 16 findings (0 CRITICAL / 2 HIGH / 9 MEDIUM / 5 LOW), tous ground-truthés. HIGH : H1 `payment_date` manquant dans `/pay` (date de valeur ≠ saisie) ; H2 ligne D 1171 à zéro viole `chk_jel_debit_credit_exclusive` → omettre si Σ TVA=0. MEDIUM : M1 backfill 2000 = UPDATE (pas INSERT, compte existe déjà) ; M2 D charge par ligne (pas agrégé) ; M3 variants `Journal` (Achats/Banque/OD) ; M4 `find_open_covering_date` explicite ; M5 `COLUMNS`+snapshot repo settings ; M6 `CompanyInvoiceSettingsUpdate`+PUT ; M7 `recoverable_account_id: Option<i64>` ; M8 contre-passation cancel = relecture lignes+swap (approche B) ; M9 exports/ trompeur (backup seulement, pas global ZIP). LOW : L1 `line_total>0` CHECK ; L2 position sidebar ; L3 tests cancel ; L4 descriptions écritures ; L5 `entry_date=invoice_date`. 8 DC validés SAINS (DC1/DC2/DC7 précisés). **Split 12-2a..f validé** (COLUMNS settings → 12-2a). → 16 patches appliqués.
- **Passe 2 — Haiku 4.5** : 1 HIGH / 2 MEDIUM / 4 LOW. Garde-fou Haiku appliqué (ground-truth) : **H1 écarté** (« COLUMNS missing default_payable_account_id » — déjà instruit par la spec T1 patch M5, non-finding : la spec ne peut qu'instruire l'ajout, ce qu'elle fait) ; **M2 écarté** (TVA=0 branching déjà couvert H2/M7/DC2) ; **L4 écarté** (`require_comptable_role` confirmé existant grep). Réellement actionnables : **M1** (AC1 « montant attendu » sans colonne migration → ajout `expected_payment_amount`), **L1** (cancel `entry_date` figé = date du jour), **L3** (`TABLES_TO_TRUNCATE` ajouté en tâche T1). → 3 patches appliqués. Trend >LOW : passe 1 = 11 → passe 2 = 1 (M1) réel.
- **Passe 3 — Opus 4.8** (catch-architectural) : 1 CRITICAL / 0 HIGH / 2 MEDIUM / 2 LOW, tous ground-truthés — ratés par Sonnet+Haiku. **O-C1 CRITICAL** : helper `generate_purchase_journal_lines` placé à tort en `kesh-core` (DC8) → `NewJournalEntryLine`/`DbError` sont kesh-db, `kesh-core` ne dépend pas de kesh-db → **dépendance circulaire Cargo / compile-blocker** ; déplacé en `kesh-db` (comme `generate_credit_note_journal_lines`). **O-M1 MEDIUM** : extension `company_invoice_settings` sous-spécifiée → `is_no_op_change` (mutation silencieuse ignorée) + UPDATE SQL+bind + DTO Request/Response+From + `validate_account(Liability)` + 4 tests existants cassent ; touch-points complétés (T1 + split 12-2a). **O-M2 MEDIUM** : risque HT/TTC (template 12-1 stocke HT) → `total_amount` figé = Σ HT + Σ TVA (TTC), règlement = TTC stocké, + test « solde 2000 = 0 ». O-L1/O-L2 LOW : comptes inactifs au pay/cancel = edge 4xx attendu (documenté). Points SAINS confirmés : `Journal::OD` autorisé par `chk_journal_entries_journal`, backfill UPDATE-pas-INSERT, atomicité tx, IDOR scopé, FK RESTRICT. → 5 patches. Trend >LOW : p1=11 → p2=1 → p3=3.
- **Passe 4 — Sonnet 4.6** (convergence) : confirme passes 1-3 SAINES (O-C1/O-M1/O-M2/H1/H2 tous re-vérifiés ground-truth : `kesh-db → kesh-core` OK, touch-points settings exacts, HT/TTC cohérent AC3/AC8/T2/T3). 1 MEDIUM + 1 LOW nouveaux : **N-M1** (`insert_with_defaults[_in_tx]` ne résout pas 2000 → nouvelles companies post-migration `NULL`, incohérent 1100/3000 ; Option A appliquée : étendre les 2 variantes pour résoudre 2000) ; **N-L1** (test « solde 2000=0 » mal placé en 12-2b car exige `pay()` → déplacé 12-2c/f). → 2 patches. Trend >LOW : p1=11 → p2=1 → p3=3 → p4=1.
- **Passe 5 — Haiku 4.5** (confirmation convergence) : 1 finding HIGH **P5-1 réfuté par garde-fou Haiku**. P5-1 affirmait « N-M1 non appliqué — le code n'a jamais été patché » avec grep prouvant l'absence de `default_payable_account_id` dans `crates/`. **Faux-positif méta-spec↔code** (cas pathologique documenté `feedback_haiku_review_diff_combined`) : on valide une SPEC, le champ est absent du code car le dev-story n'est pas lancé — état NORMAL d'une spec pré-dev. Ground-truth confirme que la spec instruit bien tous les touch-points (insert_with_defaults inclus, l.80), vérifié aussi par Sonnet passe 4. **Dismiss.** → 0 patch.

### Verdict de convergence

**✅ CONVERGÉ — 0 finding > LOW réel** (passe 5). Cycle complet 5 passes **Sonnet→Haiku→Opus→Sonnet→Haiku**. Trend réel > LOW : **p1=11 → p2=1 → p3=3 → p4=1 → p5=0**. ~26 patches appliqués au total. 8 DC figés. **O-C1 (CRITICAL Opus, compile-blocker kesh-core/kesh-db)** = catch architectural majeur raté par Sonnet+Haiku. 2 faux-positifs Haiku réfutés grep (passe 2 H1, passe 5 P5-1). **Split 12-2a..f validé et ordonné** (12-2a fondation DB+settings débloque le reste, pas de cycle Cargo). Spec **ready-for-dev**.
