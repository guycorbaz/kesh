# Story 12.1: Avoirs (notes de crédit)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->
<!-- Story UMBRELLA — candidate au split (touche 6 modules : kesh-db, kesh-core, kesh-api, kesh-qrbill, frontend, i18n). Proposition de split en §"Tasks / Subtasks". Décision de split à confirmer au `validate` (règle de splitting CLAUDE.md : >5 modules → split ; pattern Epic 17/18). -->

## Story

As a **utilisateur (indépendant / PME / fiduciaire)**,
I want **annuler une facture validée en créant un avoir (note de crédit) qui lui est lié et génère automatiquement l'écriture de contre-passation**,
so that **ma comptabilité reste intègre (aucune suppression de document, traçabilité complète) et le solde du client revienne à zéro**.

## Contexte & motivation

- Réalise **FR36** (annuler une facture validée *uniquement* par création d'un avoir) + **FR37** (séquence de numérotation séparée des avoirs) + **FR38** (PDF dans un nouvel onglet).
- Referme la boucle ouverte par le dogfooding v0.3 : le bug **#184** (suppression d'une écriture liée à une facture validée bloquée par FK `fk_invoices_journal_entry`) et le CR **#186** (annuler une facture comptabilisée / extourne). L'avoir EST le mécanisme métier correct d'annulation — l'utilisateur ne supprime jamais l'écriture, il crée un avoir qui la contre-passe.
- **Impact Epic 18 (TVA)** : depuis Epic 18, la validation d'une facture comptabilise la TVA due (ligne(s) `Crédit 2200 TVA due` par taux). L'avoir doit donc **contre-passer aussi les lignes de TVA**, sinon le décompte TVA serait faussé.

## Acceptance Criteria

1. **Given** une facture au statut `validated`, **When** l'utilisateur crée un avoir lié, **Then** l'avoir référence la facture d'origine (`credit_notes.invoice_id`) et reprend ses lignes (snapshot description / quantité / prix / `vat_rate`). *(FR36)*
2. **Given** une facture au statut `draft`, **When** l'utilisateur tente de créer un avoir, **Then** l'opération est refusée (un avoir ne peut viser qu'une facture validée). 
3. **Given** une facture déjà entièrement créditée (un avoir émis existe pour elle), **When** l'utilisateur tente de créer un second avoir, **Then** l'opération est refusée (modèle v0.2 : un avoir total par facture — cf. DC7).
4. **Given** un avoir, **When** il est émis (`issued`), **Then** un numéro séquentiel est attribué depuis une **séquence dédiée et indépendante** des factures, au format `AV-{YEAR}-{SEQ:04}` (ex. `AV-2026-0001`), sans trou. *(FR37)*
5. **Given** un avoir émis, **When** la comptabilisation s'exécute, **Then** une écriture de **contre-passation** est générée automatiquement, inversant exactement l'écriture de validation de la facture : `Crédit 1100 (Débiteurs, TTC)` / `Débit 3000 (Produits, HT)` / `Débit 2200 (TVA due, une ligne par taux > 0)`. *(swap débit↔crédit — PAS de montant négatif)*
6. **Given** un avoir émis avec TVA multi-taux, **When** la contre-passation est générée, **Then** il y a une ligne de TVA contre-passée par taux distinct (cohérent avec la comptabilisation de la facture, BTreeMap taux ASC), chaque montant calculé via `line_vat_amount` sur le `vat_rate` figé de la ligne.
7. **Given** la facture d'origine + son avoir émis, **When** on vérifie le solde du compte Débiteurs (1100) pour ce client, **Then** facture + avoir = 0 (le solde revient à zéro). *(scénario PRD Marc)*
8. **Given** un avoir émis sur une facture, **When** l'opération réussit, **Then** la facture d'origine passe au statut `cancelled` (réalise FR36 « annuler une facture validée ») — sa propre écriture de validation reste intacte (intangibilité ; l'annulation est portée par la contre-passation). *(DC6)*
9. **Given** un avoir émis, **When** l'utilisateur télécharge son PDF, **Then** un PDF « similaire à la facture » est généré avec la mention **« Avoir »**, le numéro d'avoir, et une **référence à la facture d'origine**, **sans section QR Bill** (le paiement irait dans l'autre sens). *(FR38 : ouverture nouvel onglet/téléchargement)*
10. **Given** la contre-passation, **When** elle est comptabilisée, **Then** elle tombe dans un exercice comptable **ouvert** couvrant la date de l'avoir (sinon refus avec erreur métier claire), et l'écriture générée est équilibrée (`SUM(debit) == SUM(credit)`).
11. **Given** toute l'opération (création/émission avoir + contre-passation + bascule statut facture), **When** elle s'exécute, **Then** elle est **atomique** (une seule transaction sqlx, `SELECT … FOR UPDATE` sur la facture et la config, rollback complet en cas d'échec) et journalisée dans l'`audit_log` (`credit_note.created` / `credit_note.issued`).
12. **Given** l'accès multi-tenant, **When** un avoir est lu/créé, **Then** toutes les requêtes sont scopées par `company_id` (anti-IDOR), pattern identique aux factures.
13. **Given** l'UI, **When** une facture est `validated`, **Then** un bouton « Créer un avoir » est proposé sur la page détail facture ; une section « Avoirs » est accessible (liste + détail) ; le bouton « Voir l'écriture comptable » de l'avoir pointe vers l'écriture de contre-passation.
14. **Given** l'i18n, **When** des libellés sont ajoutés, **Then** ils sont traduits dans les **4 locales** (fr/de/it/en) et respectent l'ownership (`credit-note-*` pour la feature frontend `credit-notes`).
15. **Given** la qualité, **When** la story est livrée, **Then** tests d'intégration repo (`#[sqlx::test]`) couvrant contre-passation mono/multi-taux + équilibre + solde→0 + refus (draft / double avoir / exercice fermé), tests unitaires du helper de contre-passation et des helpers frontend, et au moins un E2E (création avoir depuis facture validée → PDF). `cargo fmt`/`clippy`/`test` + `npm run check`/`test:unit`/`lint-i18n-ownership` verts.

## Décisions de conception (DC — figées, à valider au `validate`)

- **DC1 — Tables dédiées** : créer `credit_notes` + `credit_note_lines` (miroir de `invoices`/`invoice_lines`), **pas** de réutilisation de `invoices` avec un flag de type. Rationale : séquence de numérotation séparée (FR37), sémantique de statut distincte, séparation propre. *(Source: agent schéma + invoices.sql)*
- **DC2 — Contre-passation via NOUVEAU helper** `generate_credit_note_journal_lines` : swap débit↔crédit, montants **positifs**. **NE PAS** réutiliser `generate_invoice_journal_lines` (documenté hors-scope avoirs, `invoices.rs:927-932`) et **NE PAS** utiliser de montants négatifs (interdits par `chk_jel_debit_nonneg` / `chk_jel_credit_nonneg` / `chk_jel_debit_credit_exclusive`). Réutiliser `kesh_core::accounting::vat::line_vat_amount` (fonctionne, montant positif) + agrégation par taux via `BTreeMap` (taux ASC, cohérent facture).
- **DC3 — Avoir TOTAL uniquement (v0.2)** : l'avoir reprend **toutes** les lignes de la facture (snapshot), réversion intégrale → solde 0. L'avoir **partiel** (créditer une partie) est **hors scope** (déféré, story future). Rationale : correspond au scénario PRD (« solde revient à zéro ») et garde la story tenable.
- **DC4 — Séquence séparée** : nouvelle table `credit_note_number_sequences` (DDL identique à `invoice_number_sequences`, scope `(company_id, fiscal_year_id)`, no-gap via `SELECT … FOR UPDATE` + `INSERT IGNORE` paresseux). Format configurable via nouvelle colonne `company_invoice_settings.credit_note_number_format VARCHAR(64) NOT NULL DEFAULT 'AV-{YEAR}-{SEQ:04}'`. Réutiliser `kesh_core::invoice_format::render` (placeholders `{YEAR}`/`{FY}`/`{SEQ:NN}`) tel quel.
- **DC5 — Modèle de statut `draft` → `issued` → `cancelled`** (miroir facture `draft`→`validated`) : l'avoir est créé en `draft` (revue possible), puis l'émission (`issued`) attribue le numéro + génère la contre-passation + bascule la facture en `cancelled` (DC6), le tout atomiquement. *(À arbitrer au `validate` : single-step « create+issue » acceptable si jugé plus simple — défaut figé = two-step miroir facture.)*
- **DC6 — La facture d'origine passe `cancelled` à l'émission de l'avoir** : réalise FR36. L'écriture de validation de la facture **reste intacte** (intangibilité CO 958f) ; l'annulation comptable est portée par l'écriture de contre-passation. Le statut `cancelled` existe déjà dans le CHECK de `invoices` (jamais utilisé jusqu'ici). Garde-fou `chk_invoices_paid_at_validated` autorise `paid_at` en statut `cancelled` (pas de régression).
- **DC7 — Un avoir total par facture** : `credit_notes.invoice_id` FK → `invoices(id)` `ON DELETE RESTRICT` + contrainte d'unicité applicative/DB empêchant un 2ᵉ avoir émis sur la même facture (AC3). 
- **DC8 — PDF sans QR Bill** : étendre `kesh-qrbill` pour rendre la section paiement conditionnelle (param `include_qr_bill: bool` sur `generate_qr_bill_pdf_with_date`, ou fonction dédiée `generate_credit_note_pdf`). Pour l'avoir : titre « Avoir » (clé i18n), `N° d'avoir`, ligne « Réf. facture d'origine : F-…», **omettre** `draw_separator`/`draw_receipt`/`draw_payment_part` (`pdf.rs:96-98`). Nouvelle route `GET /api/v1/credit-notes/{id}/pdf` calquée sur `invoice_pdf.rs`.
- **DC9 — Snapshot contact** : `credit_notes.contact_id` copié depuis la facture (FK `contacts` RESTRICT). 
- **DC10 — Exercice de la contre-passation** : posté à la **date de l'avoir** ; exiger un exercice ouvert via `fiscal_years::find_open_covering_date` (la date d'avoir peut différer de celle de la facture).
- **DC11 — Migration non-breaking** : `CREATE TABLE credit_notes` + `credit_note_lines` + `credit_note_number_sequences` + `ALTER TABLE company_invoice_settings ADD COLUMN credit_note_number_format … DEFAULT …` → **toutes non-breaking** (anciens binaires les ignorent) → **pas de bump** `kesh_version_min_required` (P1/P2). **Obligation P5** : ajouter les lignes correspondantes à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`).

## Tasks / Subtasks (proposition de split umbrella → sous-stories)

> Si split confirmé au `validate` : découper en **12-1a → 12-1f** (série a→f, pattern Epic 17/18). Sinon, exécuter dans cet ordre comme une story unique.

- [ ] **12-1a — Fondation DB + entités** (AC: 1,4,12 / DC1,DC4,DC7,DC11)
  - [ ] Migration `CREATE TABLE credit_notes` (miroir `invoices` : `id, company_id, contact_id, invoice_id FK RESTRICT, credit_note_number VARCHAR(64) NULL, status VARCHAR(16) DEFAULT 'draft' CHECK IN ('draft','issued','cancelled'), date DATE, total_amount DECIMAL(19,4), journal_entry_id BIGINT NULL FK RESTRICT, version INT DEFAULT 1, timestamps`) + CHECK `status<>'issued' OR (credit_note_number IS NOT NULL AND journal_entry_id IS NOT NULL)` + UNIQUE `(company_id, credit_note_number)` + index ; FK `invoice_id` + contrainte d'unicité « un avoir émis par facture » (DC7).
  - [ ] Migration `CREATE TABLE credit_note_lines` (miroir `invoice_lines` : `vat_rate DECIMAL(5,2)` figé, mêmes CHECK).
  - [ ] Migration `CREATE TABLE credit_note_number_sequences` (miroir `invoice_number_sequences`).
  - [ ] Migration `ALTER TABLE company_invoice_settings ADD COLUMN credit_note_number_format VARCHAR(64) NOT NULL DEFAULT 'AV-{YEAR}-{SEQ:04}'` + CHECK non-vide.
  - [ ] Entités Rust `CreditNote` / `CreditNoteLine` (miroir `entities/invoice.rs`).
  - [ ] Ligne(s) dans `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`) — **P5**.
- [ ] **12-1b — Repository + numérotation** (AC: 1,2,3,4,11,12 / DC4,DC7,DC9)
  - [ ] `repositories/credit_notes.rs` : `FIND_CREDIT_NOTE_SCOPED_SQL` (scopé `company_id`), `create` (draft, snapshot lignes facture), `get`/`list` paginé, transactions + verrou optimiste + `SELECT … FOR UPDATE` (miroir `invoices.rs`).
  - [ ] `repositories/credit_note_number_sequences.rs` : `next_number_for` (pattern atomique no-gap).
  - [ ] Audit log `credit_note.created`.
- [ ] **12-1c — Émission + contre-passation comptable** (AC: 5,6,7,8,10,11 / DC2,DC5,DC6,DC10) — **cœur métier**
  - [ ] Helper `generate_credit_note_journal_lines` (swap débit↔crédit, positifs, `line_vat_amount`, BTreeMap taux). Tests unitaires dédiés.
  - [ ] `issue_credit_note` (miroir `validate_invoice`) : tx unique → `FOR UPDATE` avoir+facture+settings, exercice ouvert (date avoir), `next_number_for`, render numéro, génère écriture contre-passation via le helper, `create_in_tx` (équilibre vérifié), `UPDATE credit_notes status='issued', number, journal_entry_id`, `UPDATE invoices status='cancelled'` (DC6), audit `credit_note.issued`.
- [ ] **12-1d — Routes API + PDF avoir** (AC: 9,13 / DC8)
  - [ ] Routes `GET/POST /api/v1/credit-notes`, `GET /api/v1/credit-notes/{id}`, `POST /api/v1/credit-notes/{id}/issue`, `GET /api/v1/credit-notes/{id}/pdf` (câblées `lib.rs`).
  - [ ] PDF : param `include_qr_bill` dans `kesh-qrbill` (ou `generate_credit_note_pdf`), override i18n titre/numéro + réf facture d'origine, omettre QR Bill.
- [ ] **12-1e — Frontend** (AC: 9,13,14 / DC8)
  - [ ] Feature `frontend/src/lib/features/credit-notes/` (`credit-notes.types.ts`, `credit-notes.api.ts`, `credit-note-helpers.ts` + `.test.ts`), miroir `invoices`.
  - [ ] Pages `routes/(app)/credit-notes/` (liste, `new?invoiceId=`, `[id]` détail avec PDF + lien écriture).
  - [ ] Bouton « Créer un avoir » sur `invoices/[id]/+page.svelte` (bloc `validated`), item nav « Avoirs » dans `+layout.svelte` (groupe `quotidien`).
  - [ ] i18n 4 locales, clés `credit-note-*` (ownership lint).
- [ ] **12-1f — Tests d'intégration + E2E + doc** (AC: 15)
  - [ ] `crates/kesh-db/tests/credit_notes_*.rs` (`#[sqlx::test]`, fixture `seed_accounting_company`) : contre-passation mono/multi-taux, équilibre, solde 1100 → 0, refus (facture draft / 2ᵉ avoir / exercice fermé / comptes TVA non configurés).
  - [ ] E2E Playwright `credit-notes.spec.ts` : créer avoir depuis facture validée → vérifier statut facture `cancelled` + PDF.
  - [ ] Sync doc : `user-manual.tex` §Avoirs (remplacer la note « pas d'assistant d'avoir dédié »), `CHANGELOG` `[Non publié]`, README §Fonctionnalités (retirer *(à venir)* de pain.001 ? non — pain.001 reste 12-2 ; ajouter avoirs), website si claim.

## Dev Notes

### Architecture de comptabilisation — la contre-passation (cœur)

La facture validée génère (helper `generate_invoice_journal_lines`, `crates/kesh-db/src/repositories/invoices.rs:933-995`) :

```
[0] Débit  receivable (1100) = total_ht + total_vat   (créance TTC)
[1] Crédit revenue    (3000) = total_ht               (produit HT)
[2..] Crédit vat_payable (2200) = vat_by_rate[r]      (TVA due, 1 ligne par taux > 0, BTreeMap ASC)
```

L'avoir DOIT produire l'**inverse exact** (swap débit↔crédit, montants positifs) :

```
[0] Crédit receivable (1100) = total_ht + total_vat   (annule la créance)
[1] Débit  revenue    (3000) = total_ht               (annule le produit)
[2..] Débit vat_payable (2200) = vat_by_rate[r]       (annule la TVA due, 1 ligne par taux)
```

- Calcul TVA par ligne : `kesh_core::accounting::vat::line_vat_amount(line.line_total, line.vat_rate)` (`crates/kesh-core/src/accounting/vat.rs:39-43`, arrondi `MidpointAwayFromZero` commercial AFC). Agréger `vat_by_rate: BTreeMap<Decimal, Decimal>` puis `total_vat = Σ valeurs` (somme des arrondis, **ne pas réarrondir**).
- **Anti-pattern interdit** : montants négatifs (DB `chk_jel_debit_nonneg`/`chk_jel_credit_nonneg`/`chk_jel_debit_credit_exclusive` les bannissent) et réutilisation de `generate_invoice_journal_lines` (docstring `invoices.rs:927-932` l'interdit explicitement).
- Équilibre vérifié par `journal_entries::create_in_tx` (`SUM(debit)==SUM(credit)`, rollback sinon) — l'inverse d'une écriture équilibrée l'est par construction.
- Lien : `credit_notes.journal_entry_id` → l'écriture de contre-passation (NOUVELLE `JournalEntry`, propre `entry_number`).

### Fichiers à créer / modifier (cités, fichier:ligne)

**Backend — créer** : `crates/kesh-db/migrations/2026MMDD0000NN_credit_notes.sql` (+ lines + sequences + alter settings) ; `crates/kesh-db/src/entities/credit_note.rs` ; `crates/kesh-db/src/repositories/credit_notes.rs` + `credit_note_number_sequences.rs` ; `crates/kesh-api/src/routes/credit_notes.rs` + `credit_note_pdf.rs`.
**Backend — modifier** : `crates/kesh-db/src/entities/mod.rs` + `repositories/mod.rs` (exports) ; `crates/kesh-db/src/entities/company_invoice_settings.rs` (+ champ `credit_note_number_format`) + son repo + ses tests ; `crates/kesh-api/src/lib.rs:375` (routes) ; `crates/kesh-qrbill/src/pdf.rs:70-101` (param `include_qr_bill`) + `src/types.rs` (clés i18n) ; `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` ; `docs/migrations-idempotence-audit.md`.

**Frontend — créer** : `src/lib/features/credit-notes/{credit-notes.types.ts,credit-notes.api.ts,credit-note-helpers.ts,credit-note-helpers.test.ts,credit-notes.api.test.ts}` ; `src/routes/(app)/credit-notes/{+page.svelte,new/+page.svelte,[id]/+page.svelte}` ; `tests/e2e/credit-notes.spec.ts`.
**Frontend — modifier** : `src/routes/(app)/+layout.svelte:57-65` (nav) ; `src/routes/(app)/invoices/[id]/+page.svelte:291-323` (bouton « Créer un avoir » dans bloc `validated`).

### Patterns à respecter (réutilisation, anti-réinvention)

- **Repository** (miroir `crates/kesh-db/src/repositories/invoices.rs`) : constante `FIND_*_SCOPED_SQL` avec `AND company_id = ?` (anti-IDOR) ; verrou optimiste `version` (`AND version = ?`, `version = version + 1`, re-query si `rows_affected==0` pour distinguer NotFound vs OptimisticLockConflict) ; transactions (`pool.begin()` + commit/rollback explicites OU async-block) ; `SELECT … FOR UPDATE` sur toute mutation ; audit log atomique en fin de tx (`audit_log::insert_in_tx`) ; snapshot JSON camelCase.
- **Numérotation** (miroir `repositories/invoice_number_sequences.rs:30-97`) : `SELECT next_number … FOR UPDATE` → `INSERT IGNORE` si absent → re-SELECT FOR UPDATE → `UPDATE next_number+1` → retourne la valeur lue. No-gap garanti par rollback.
- **Émission** (miroir `validate_invoice`, `invoices.rs:1019-1232`) : ordre des locks canonique = facture/avoir `FOR UPDATE` → settings `get_or_create_default_in_tx` (`FOR UPDATE`) → exercice ouvert → séquence → render → écriture → updates → audit.
- **PDF** (miroir `crates/kesh-api/src/routes/invoice_pdf.rs` + `kesh-qrbill/src/pdf.rs`) : réutiliser `InvoicePdfData`/`InvoiceLinePdf`/`QrBillI18n`/`build_i18n`/`split_address`/`sanitize_filename` ; conditionner `draw_separator`/`draw_receipt`/`draw_payment_part` (`pdf.rs:96-98`) ; override map i18n `invoice-pdf-title`→« Avoir » avant `QrBillI18n::new` ; nouvelles clés FTL `credit-note-pdf-*`.
- **Frontend** (miroir feature `invoices`) : `apiClient.get/post/getBlob` (`src/lib/shared/utils/api-client.ts`) ; montants en `string` décimale + `big.js` (jamais `number`) ; erreurs via `isApiError`/`notifyError` ; `i18nMsg(key, fallback, args?)` ; **ownership lint** : feature `credit-notes` → clés préfixées `credit-note-` (`frontend/scripts/lint-i18n-ownership.js`, check `key.startsWith('credit-notes-')` — **vérifier le préfixe exact attendu** : feature multi-segment `credit-notes` ⇒ préfixe `credit-notes-` ; les routes `(app)/` ne sont pas scannées) ; `data-testid` pour E2E ; PDF via `<a download>` invisible (anti popup-blocker, `invoices/[id]/+page.svelte:235-241`).

### Schéma DDL de référence (à mirrorer)

`invoices`/`invoice_lines` : `crates/kesh-db/migrations/20260416000001_invoices.sql:15-61`. `journal_entry_id` + `invoice_number_sequences` + `company_invoice_settings` : `20260417000001_invoice_validation.sql:21-57`. CHECK `validated_has_je` : `20260417000002_*`. Comptes TVA settings (Epic 18) : `20260614000001_vat_accounts_config.sql:53-104`. Entités : `crates/kesh-db/src/entities/invoice.rs:17-89`, `company_invoice_settings.rs:19-36`, `journal_entry.rs:128-151`.

### Tests

- **Repo** (`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fixture `kesh_db::test_fixtures::seed_accounting_company` → comptes 1000/1100/2000/3000/4000 + settings + FY 2020-2030 ; **étendre** la fixture pour configurer `default_vat_payable_account_id` = 2200 si besoin) : modèle `crates/kesh-db/tests/invoices_validate_vat.rs`. Cas : mono-taux (annule 1081 → solde 0), multi-taux (lignes TVA inversées par taux), équilibre, refus facture `draft`, refus 2ᵉ avoir, refus exercice fermé, refus comptes TVA non configurés (si TVA>0).
- **Unitaire** helper contre-passation (`kesh-core` ou `kesh-db`) : exemples chiffrés inverses de `vat.rs`/`invoices_validate_vat.rs`.
- **E2E** `tests/e2e/credit-notes.spec.ts` (modèle `invoices.spec.ts`, `seedTestState('with-data')`, login, `authedApiContext` pour setup) : créer facture → valider → créer avoir → vérifier facture `cancelled` + PDF `%PDF`.

### Project Structure Notes

- Numérotation epics : **sprint-status est autoritaire** (E12 = Avoirs & Paiements). `epics.md` porte encore l'ancienne numérotation (« Epic 11 : Avoirs & Paiements ») — dérive connue (action item rétro Epic 11 #2 non faite). Cette story suit la numérotation sprint-status (12-1).
- Aucune dépendance bloquante : E12 s'appuie sur Epic 5 (factures) + Epic 18 (comptabilisation TVA), tous deux livrés.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic 11 : Avoirs & Paiements (Story 11.1)] — AC d'origine.
- [Source: _bmad-output/planning-artifacts/prd.md:114,424-426] — FR36/FR37/FR38 + scénario Marc.
- [Source: crates/kesh-db/src/repositories/invoices.rs:933-995] — `generate_invoice_journal_lines` (à inverser, NE PAS réutiliser).
- [Source: crates/kesh-db/src/repositories/invoices.rs:1019-1232] — `validate_invoice` (modèle d'émission).
- [Source: crates/kesh-core/src/accounting/vat.rs:39-43] — `line_vat_amount` (réutiliser).
- [Source: crates/kesh-db/src/repositories/invoice_number_sequences.rs:30-97] — séquence no-gap (à mirrorer).
- [Source: crates/kesh-qrbill/src/pdf.rs:70-101] — conditionner QR Bill ; [crates/kesh-api/src/routes/invoice_pdf.rs:44-127] — handler PDF.
- [Source: frontend/src/routes/(app)/invoices/[id]/+page.svelte:276-324] — boutons conditionnels au statut ; [frontend/scripts/lint-i18n-ownership.js] — ownership i18n.
- [Source: CLAUDE.md#Migration breaking policy (P1-P5)] — non-breaking + audit idempotence ; [CLAUDE.md#Issue Tracking Rule] — #184/#186.

## Dev Agent Record

### Agent Model Used

(à compléter par dev-story)

### Debug Log References

### Completion Notes List

### File List
