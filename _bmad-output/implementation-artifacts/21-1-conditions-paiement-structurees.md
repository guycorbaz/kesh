# Story 21.1: Conditions de paiement structurées (jours) sur le contact → échéance de facture calculée

Status: review

<!-- Spec créée le 2026-07-12 (bmad-create-story, Fable 5). Source : planning epic-21-echeances-relances.md §A (décisions D1-D4) + issue #245 + cartographie 2 agents Explore (backend + frontend). Ferme #245. -->

## Story

En tant que **comptable d'une PME**,
je veux **définir un délai de paiement en jours sur la fiche contact et voir l'échéance et les conditions de paiement de mes factures pré-remplies automatiquement**,
afin de **ne plus saisir manuellement ces informations à chaque facture et d'obtenir des échéances fiables pour le futur cycle de relance (#231)**.

## Acceptance Criteria

### Base de données & entité

1. **Migration** `crates/kesh-db/migrations/20260712000001_contacts_default_payment_terms_days.sql` : `ALTER TABLE contacts ADD COLUMN default_payment_terms_days INT NULL;` + `ADD CONSTRAINT chk_contacts_payment_terms_days CHECK (default_payment_terms_days IS NULL OR (default_payment_terms_days >= 0 AND default_payment_terms_days <= 365));` — calque exact de `20260709000001_contacts_language_salutation.sql` (ADD COLUMN puis ADD CONSTRAINT séparé, en-tête de commentaire « Non-breaking → pas de bump kesh_version_min_required, politique Story 10-2 P1 »).
2. **Ligne ajoutée à `docs/migrations-idempotence-audit.md`** (garde-fou P5 — sinon finding MEDIUM en code review) **et** mise à jour de la ligne de synthèse `**Total** : 50 migrations (…)` (ligne 67) → 51.
3. **Compteur de migrations** : `migrations_upgrade_path.rs` passe 50 → 51 (famille « nouvelle migration → compteur figé » — 3 régressions de cette famille à l'Epic 20). Aucune nouvelle table → `TABLES_TO_TRUNCATE` (backup.rs) et compteur export inchangés.
4. **Entité** (`crates/kesh-db/src/entities/contact.rs`) : `default_payment_terms_days: Option<i32>` ajouté à `Contact` (:172-204), `NewContact` (:209-235), `ContactUpdate` (:244-269).
5. **Repo** (`crates/kesh-db/src/repositories/contacts.rs`) — TOUS les sites à liste de champs étendus : `COLUMNS` (:28-31) **et** `FIND_BY_ID_SQL` (:33-36, liste dupliquée), INSERT + bindings de `create` (:189-264), SET + bindings de `update` (:396-515), `contact_snapshot_json` (:39-57, audit), **`is_no_op_change` (:374-392)** — sans quoi une modification isolée du champ serait silencieusement ignorée (pas de bump version, pas d'audit). Helpers de test `new_contact` (:639-661) et `contact_to_update` (:1466-1487) étendus.
6. **Balayage workspace `query_as::<_, Contact>`** : vérifier que TOUTE requête qui hydrate un `Contact` utilise `contacts::COLUMNS` ou est étendue (leçon 20-3b1 : 3 listes SQL inline stale → 500 `ColumnNotFound` ; `grep -rn "query_as::<_, Contact>" crates/` et inspection de chaque site — 10 sites recensés, tous via `COLUMNS`/`FIND_BY_ID_SQL` au 2026-07-12, re-vérifier au dev).
6-bis. **Balayage symétrique côté construction `NewContact {`** (struct literal sans `Default` ni spread — E0063 sinon) : `grep -rn "NewContact {" crates/ | grep -v "pub struct"` remonte 22 sites. Outre `routes/contacts.rs:475` (AC 7) et le helper `new_contact` de `repositories/contacts.rs` (AC 5), **20 sites dans 13 fichiers de tests sur 3 crates** doivent recevoir `default_payment_terms_days: None,` : `kesh-api/tests/{invoice_delete_e2e.rs:116, vat_report_e2e.rs:125, invoice_pdf_e2e.rs:124, invoice_send_email_e2e.rs:160, inbox_import_e2e.rs:166, invoice_echeancier_e2e.rs:120, reconciliation_e2e.rs:206}`, `kesh-report/tests/vat_report_reconciliation.rs:46`, `kesh-db/tests/{supplier_invoices_repository.rs:64+380, reconciliation_repository.rs:89, credit_notes_repository.rs:29, invoices_validate_vat.rs:27, payment_batches_repository.rs:60, kf005_fulltext_index_e2e.rs:90+473+501}`, `kesh-db/src/repositories/invoices.rs:1946+4147` (module de test). Le gate `cargo build --workspace --all-targets` est le filet.

### API contacts

7. `CreateContactRequest` (:65-97) et `UpdateContactRequest` (:99-131) acceptent `default_payment_terms_days: Option<i32>` (`#[serde(default)]`, camelCase `defaultPaymentTermsDays`). Sémantique PUT full-payload existante : omis ou `null` → efface (aucun tri-state, conforme au reste du form contact).
8. **Validation** dans `validate_common` (:347-354) : si `Some(d)` avec `d < 0 || d > 365` → `AppError::Validation(format!("Le délai de paiement doit être compris entre 0 et 365 jours"))` — **chaîne française codée en dur**, cohérent avec le pattern existant de `validate_common` (:302-352, aucun message de ce fichier n'est i18n — vérifié, zéro usage de `t`/`t_args` dans contacts.rs). Le CHECK SQL n'est que le filet.
9. `ContactResponse` (:139-168) expose `defaultPaymentTermsDays: Option<i32>` **et** `defaultPaymentTermsLabel: Option<String>` — libellé **généré côté serveur** dans la **langue du contact** (fallback `company.instance_language`), présent sur **tous** les endpoints qui renvoient un contact : list, get, create, update, **archive** (uniformité API). ⚠️ `ContactPicker` du formulaire facture consomme l'endpoint **list** — si le label manque sur list, le pré-remplissage frontend est mort. (Le i18n frontend ne connaît que la locale UI — confirmé `i18n.svelte.ts` — le libellé par langue de contact DOIT venir de l'API.)
9-bis. **État réel des 5 handlers** (vérifié — le `From<Contact>` actuel n'a pas accès à `Company`) : `create_contact` (:453-505) a déjà `get_company_for` (:458) ; `list_contacts` (:395-434) l'appelle mais **jette le résultat** (`let _ =`, :401) → le conserver dans une variable et remplacer `.map(ContactResponse::from)` (:428) par une closure ; `get_contact` (:438-449), `update_contact` (:509-562) et `archive_contact` (:566-579) n'appellent **pas** `get_company_for` → l'ajouter. Implémentation : helper `contact_response_with_label(contact, &company, &i18n) -> ContactResponse` appliqué aux 5 sites ; le `impl From<Contact>` peut rester pour les usages internes mais AUCUN handler ne doit renvoyer un contact sans label.
10. Le label est calculé par un helper `payment_terms_label(days: i32, locale: Locale, i18n: &I18nBundle) -> String` appelant **directement `bundle.format(&locale, key, Some(&args))`** (PAS `t_args`, qui lit la locale globale de la requête — piège identifié). Précédent d'args Fluent : `AppError::InvoiceTooManyLinesForPdf` (`errors.rs:1032-1046`, `FluentArgs` + `{ $count }`).
11. **Libellés figés** (clés `kesh-i18n/locales/*/messages.ftl` ×4) :
    - `contact-payment-terms-days-label` : FR `Payable à { $days } jours net` · DE `Zahlbar innert { $days } Tagen` (usage CH — PAS « innerhalb von ») · IT `Pagabile entro { $days } giorni` · EN `Payable within { $days } days`
    - `contact-payment-terms-immediate-label` (cas `days == 0`) : FR `Payable au comptant` · DE `Zahlbar sofort` · IT `Pagabile a vista` · EN `Due upon receipt`
12. La résolution de langue réutilise `resolve_language(contact, company)` (`invoice_email.rs:63-65`) promue `pub(crate)` (DRY — pas de duplication), ainsi que l'idiome `kesh_i18n::Locale::from(language.as_str())` utilisé en `invoice_email.rs:283` (c'est un idiome inline, PAS une fonction nommée réutilisable — le reproduire tel quel).

### Création / édition de facture (backend)

13. `ensure_contact_belongs_to_company` (`invoices.rs:374-388`) **retourne le `Contact`** au lieu de `()` (le contact y est déjà chargé — pas de 2e fetch) ; les 2 call-sites (:509, :544) adaptés.
14. `create_invoice` (:502-534) : (a) si `req.due_date` est `None` **et** que le contact a `default_payment_terms_days = Some(d)` → `due_date = date + Duration::days(d)` ; sinon comportement actuel (`unwrap_or(req.date)`). (b) si `req.payment_terms` est `None`/vide après normalisation **et** que le contact a des jours → `payment_terms = libellé auto` (langue contact, AC 10-12). Valeurs explicites du client : **jamais écrasées**. Le commentaire :519-521 (« Pas de calcul auto — décision Guy ») est remplacé (décision caduque, #245).
15. `update_invoice` (:536-570) : **aucun pré-remplissage** depuis le contact (on édite une facture existante) — défaut `unwrap_or(req.date)` conservé.
16. **Validation `due_date >= date`** à la création : sinon 400 `VALIDATION_ERROR`, message via `t("error-invoice-due-date-before-date", …)` + clés FTL ×4. *Refinement vs planning (qui disait 422)* : cohérence avec les validations facture existantes dans le handler (précédent `dateFrom <= dateTo`, `invoices.rs:454-460` → `AppError::Validation` 400).
17. **À l'update** : la validation ne s'applique que si la paire `(date, due_date)` **change** par rapport à la facture stockée (un fetch scopé pour comparer) — une facture legacy avec `due_date < date` reste éditable sur ses autres champs (planning D4).

### Frontend — fiche contact

18. Formulaire contact (`routes/(app)/contacts/+page.svelte`) : nouveau champ « Délai de paiement (jours) » — `<Input id="form-payment-terms-days" type="text" inputmode="numeric">` (aucun `type="number"` n'existe dans ce form ; le pattern maison est text + parse/validation), state `formPaymentTermsDays`, reset création `''` / édition `String(c.defaultPaymentTermsDays ?? '')`. Validation client dans `formValidation` (:263-278) : vide OU entier 0..365, sinon message bloquant (bouton disabled, pattern existant). Payload : `defaultPaymentTermsDays: formPaymentTermsDays === '' ? null : Number(formPaymentTermsDays)` (null explicite, convention du form).
19. **Préséance jours > texte** : quand le délai est renseigné, le champ texte libre `#form-payment-terms` est **désactivé** avec un hint i18n (« Libellé généré automatiquement depuis le délai ») ; quand le délai est vide, comportement actuel inchangé. Les clés i18n de cette page (`contact-form-payment-terms-days*`) sont libres du lint-ownership (page route, hors `src/lib/features/` — confirmé).
20. Types (`contacts.types.ts`) : `defaultPaymentTermsDays: number | null` + `defaultPaymentTermsLabel: string | null` sur `ContactResponse` (:33-60) ; `defaultPaymentTermsDays?: number | null` sur les 2 payloads (:62-97).

### Frontend — formulaire facture

21. `InvoiceForm.svelte` — `onContactSelect` (:234-242) étendu, mêmes gardes « seulement si vide » : si `!dueDate` et `c.defaultPaymentTermsDays != null` → `dueDate = addDaysIso(date, c.defaultPaymentTermsDays)` ; le pré-remplissage de `paymentTerms` (si vide) préfère `c.defaultPaymentTermsLabel ?? c.defaultPaymentTerms`. Aucun écrasement d'une saisie utilisateur.
22. Helper **`addDaysIso(iso: string, days: number): string`** dans `lib/features/invoices/invoice-helpers.ts` (fichier existant) — aucun helper d'addition de jours ni lib de dates n'existe (confirmé, pas de date-fns). Implémentation timezone-safe (composantes + `Date.UTC`, retour `YYYY-MM-DD`) + tests unitaires (fin de mois, année bissextile, +0 jour).
23. `validateClient()` (:278-303) : si `dueDate` non vide et `dueDate < date` (comparaison lexicographique valide sur ISO) → message bloquant « L'échéance doit être postérieure ou égale à la date de la facture ».

### Tests

24. **Repo kesh-db** (tests inline `contacts.rs`, série `--test-threads=1`) : create + find avec jours ; update du seul champ jours → version bumpée + audit (prouve l'extension `is_no_op_change`) ; no-op inchangé (KF-004) ; verrou optimiste intact.
25. **E2E kesh-api** : contact créé avec `defaultPaymentTermsDays: 30` → response contient days + label FR ; contact langue `DE` → label `Zahlbar innert 30 Tagen` ; bornes (`-1`, `366` → 400) ; facture créée sans `dueDate` ni `paymentTerms` sur contact 30j → `due_date = date + 30` et `payment_terms` = libellé (langue contact) ; facture avec valeurs explicites → inchangées ; contact sans jours → comportement actuel (`due_date = date`) ; `dueDate < date` → 400 ; update d'une facture legacy `due_date < date` sans toucher la paire → accepté.
26. **Frontend** : unit `addDaysIso` ; unit/`formValidation` si extractible (sinon couvert E2E).
27. **E2E Playwright** (backend contre `kesh_e2e`, cf. Dev Notes) : (a) contacts.spec — créer un contact avec délai 30, rouvrir → champ jours affiché, champ texte désactivé ; (b) invoices.spec — contact API avec 30j (étendre `createContactWithAddressViaApi` d'un param `paymentTermsDays?`) → `/invoices/new`, sélection contact → `#invoice-due-date` = date + 30 et `#invoice-payment-terms` = « Payable à 30 jours net » ; création OK.

### Doc & gate

28. `CHANGELOG.md` section `[Non publié]` : entrée `Added` (conditions de paiement structurées, échéance pré-calculée, refs #245). Manuels user/admin : **différés à 21-8** (story doc de l'epic). README : vérifier que la Feuille de route reflète l'epic 21 en cours (v0.7 🚧) — inclure dans le même commit si dérive constatée.
29. **Quality gate Test Locally First** complet et vert avant chaque commit de code (backend 4 checks + frontend 4 checks + E2E ciblés), sans jamais piper le runner (`> log 2>&1; echo EXIT=$?`).

## Tasks / Subtasks

- [x] **T1 — Migration + entité + repo** (AC 1-6-bis)
  - [x] Migration `20260712000001_contacts_default_payment_terms_days.sql` (+ audit idempotence P5 avec ligne Total 50→51, compteur test 50→51)
  - [x] `Contact`/`NewContact`/`ContactUpdate` + `COLUMNS` + `FIND_BY_ID_SQL` + INSERT/UPDATE bindings + `contact_snapshot_json` + `is_no_op_change` + helpers de test
  - [x] Balayage `query_as::<_, Contact>` workspace (lecture) + **balayage `NewContact {` — 20 sites de tests dans 13 fichiers, 3 crates (AC 6-bis)** (construction)
- [x] **T2 — API contacts + libellé localisé** (AC 7-12)
  - [x] Requests + validation 0..365 (message FR en dur, pattern validate_common) ; `ContactResponse` days + label sur les **5** handlers (AC 9-bis : ajouter `get_company_for` à get/update/archive, conserver celui de list dans une variable, closure au lieu de `.map(ContactResponse::from)`)
  - [x] Helper `contact_response_with_label` + `payment_terms_label` (bundle.format direct + FluentArgs) ; `resolve_language` promue pub(crate) ; clés FTL ×4 (`contact-payment-terms-days-label`, `contact-payment-terms-immediate-label`)
- [x] **T3 — Défauts facture + validation échéance** (AC 13-17)
  - [x] `ensure_contact_belongs_to_company` → retourne `Contact` ; défauts due_date + payment_terms dans `create_invoice` ; update inchangé
  - [x] Validation `due_date >= date` create (400 + FTL ×4) + règle « paire modifiée » à l'update
- [x] **T4 — Frontend contact** (AC 18-20)
- [x] **T5 — Frontend facture** (AC 21-23) : `addDaysIso` + prefill + validateClient
- [x] **T6 — Tests backend** (AC 24-25)
- [x] **T7 — Tests frontend + E2E Playwright** (AC 26-27) : étendre `api-fixtures.ts` (`paymentTermsDays?`)
- [x] **T8 — Doc-sync + gate final** (AC 28-29)

## Dev Notes

### Pièges identifiés (ground-truth 2026-07-12)

- **Deux `MAX_PAYMENT_TERMS_LEN` distincts** : contacts.rs `= 100`, invoices.rs `= 255`. Ne pas les confondre ; le libellé auto généré fait < 60 chars dans les 4 langues, aucun risque de dépassement.
- **`t`/`t_args` lisent la locale GLOBALE de la requête** (`errors.rs:33-48`) — pour le libellé en langue du **contact**, appeler `bundle.format(&locale_contact, key, Some(&args))` directement (AC 10). Piège classé bloquant : un label FR pour un contact DE serait un bug silencieux en dogfooding.
- **`is_no_op_change` + `FIND_BY_ID_SQL`** : deux listes de champs qui ne sont PAS `COLUMNS` — les trois doivent être étendues (AC 5).
- **PUT full-payload contacts** : un champ omis du payload est effacé (pas de tri-state). Le frontend envoie toujours `null` explicite — conserver cette convention pour le nouveau champ.
- **`ContactPicker` consomme `listContacts`** : le label doit être présent sur l'endpoint **list** (AC 9), pas seulement le détail.
- **Pré-remplissage non destructif** : garde « seulement si champ vide » (pattern existant `onContactSelect` :239-241) sur `dueDate` ET `paymentTerms`. En édition de facture, `initialInvoice` peuple déjà les champs → aucun écrasement possible.
- **Ordre des gardes create_invoice** : `get_company_for` → contact (retourne l'entité) → lignes/TVA → payment_terms → défauts → validation `due_date >= date` — la validation porte sur la valeur FINALE (calculée ou explicite). Une échéance calculée est toujours ≥ date (days ≥ 0), la validation ne peut rejeter que des valeurs explicites.
- **Comparaison ISO frontal** : `dueDate < date` en string est correct sur `YYYY-MM-DD` (inputs `type="date"`).

### Patterns à réutiliser (ne PAS réinventer)

- Migration : `20260709000001_contacts_language_salutation.sql` (même table, ADD COLUMN + CHECK nommé `chk_contacts_*`).
- Erreur i18n avec args : `AppError::InvoiceTooManyLinesForPdf` (`errors.rs:1032-1046`).
- Validation de dates dans le handler : `dateFrom <= dateTo` (`invoices.rs:454-460`) → `AppError::Validation` 400.
- Résolution langue contact : `invoice_email.rs:63-65` (+ mapping `Language → Locale` du même module).
- Tests repo contacts : `test_create_and_find`, `update_no_op_returns_unchanged_entity_no_audit` (:1491), `update_partial_change_bumps_version` (:1542).
- Seed facture e2e : `invoice_pdf_e2e.rs:188-197` (`NewInvoice` avec due_date/payment_terms).
- Sélecteurs E2E : ids en dur `#form-*` (contacts), `#invoice-due-date`/`#invoice-payment-terms` (facture), combobox `Rechercher un contact` (invoices.spec.ts:104-131).

### Environnement de test (rappels projet)

- `kesh-db` en **série** : `cargo test -p kesh-db -- --test-threads=1` (DATABASE_URL dev `mysql://kesh:kesh_dev@127.0.0.1:3306/kesh`).
- E2E Playwright : backend contre **`kesh_e2e`** (recette complète dans testing.md / story 20-4), `KESH_STATIC_DIR=frontend/build` → **`npm run build` obligatoire après tout fix frontend** avant re-run E2E ; `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`.
- Jamais `runner | grep/tail` : `> log 2>&1; echo EXIT=$?` puis inspecter le log.

### Hors scope (garde-fous anti-creep)

- **Aucune migration des contacts existants** (texte libre conservé en lecture, décision #245) ; pas de backfill.
- **Pas de recalcul d'échéance à l'édition de facture** (AC 15) ni au changement de contact d'une facture existante au-delà de la garde « si vide ».
- **PDF inchangé** (`payment_terms` imprimé tel quel, `pdf.rs:374-383`) ; la due_date n'est pas imprimée (état actuel conservé — l'ajout relèverait de #151).
- **Manuels user/admin → 21-8.**

### Project Structure Notes

- Backend : `crates/kesh-db/{migrations,src/entities/contact.rs,src/repositories/contacts.rs}`, `crates/kesh-api/src/routes/{contacts.rs,invoices.rs,invoice_email.rs(pub use)}`, `crates/kesh-i18n/locales/*/messages.ftl`.
- Frontend : `src/routes/(app)/contacts/+page.svelte`, `src/lib/features/contacts/contacts.types.ts`, `src/lib/components/invoices/InvoiceForm.svelte`, `src/lib/features/invoices/invoice-helpers.ts`, `tests/e2e/{contacts.spec.ts,invoices.spec.ts,helpers/api-fixtures.ts}`.
- ~5 modules touchés (kesh-db, kesh-api routes contacts+invoices, kesh-i18n, frontend contacts, frontend invoices) — dans la limite de la règle de splitting (story volontairement bornée : pas de rapport, pas de dunning).

### References

- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md#A (D1-D4)] — décisions figées, libellés 4 langues, préséance, validation.
- [Source: GitHub #245] — cadrage Guy 2026-07-11.
- [Source: cartographie Explore backend/frontend 2026-07-12] — tous les chemins:lignes cités ci-dessus.

## Dev Agent Record

### Agent Model Used

Fable 5 (claude-fable-5) — run unique 2026-07-12.

### Debug Log References

- Gate intermédiaire : `cargo check` E0063 sur 4 littéraux `ContactUpdate` des tests inline de contacts.rs (sites non listés par la spec, qui ne couvrait que les `NewContact {`) — corrigés par le même script d'insertion.
- **Découverte hors-spec (T1)** : `exports/csv_tables.rs` énumère les colonnes contacts **explicitement** (header + row) → colonne `default_payment_terms_days` ajoutée à l'export souveraineté (`serialize_contacts_csv`). La spec affirmait « aucune nouvelle table → export inchangé », vrai au niveau fichier mais pas au niveau colonne. + 1 littéral `Contact` dans `kesh-reconciliation/src/matching.rs` (helper de test) non listé.
- **Bug attrapé par les e2e (T6)** : Fluent entoure les variables interpolées de marques d'isolation BiDi U+2068/U+2069 (`Payable à \u{2068}30\u{2069} jours net`). Inacceptable : le libellé est copié dans `invoices.payment_terms` et imprimé sur le PDF (Helvetica WinAnsi, pas de glyphe). Fix : strip ciblé dans `payment_terms_label` (pas de `set_use_isolating(false)` global qui toucherait tous les messages).

### Completion Notes List

- T1-T3 backend : migration 51, entité+repo (COLUMNS/FIND_BY_ID_SQL/INSERT/UPDATE/snapshot/is_no_op_change), 19 sites `NewContact{}` + 4 `ContactUpdate{}` + 1 `Contact{}` (matching.rs) + 1 `Contact{}` (invoice_email.rs tests) étendus, export CSV contacts + colonne, API contacts (validation 0..365, days+label sur les 5 handlers via `contact_response_with_label`, `resolve_language` promue pub(crate)), clés FTL ×4 (labels + erreur due-date), `create_invoice` défauts due_date/payment_terms + validation, `update_invoice` règle « paire modifiée ».
- T4-T5 frontend : form contact (champ jours inputmode numeric, validation client 0..365, texte libre disabled si jours, hints i18n), types ×3, `addDaysIso` (UTC-safe) + prefill `onContactSelect` (gardes « si vide ») + `validateClient` due date.
- T6-T7 tests : repo kesh-db (roundtrip + no-op extension prouvée), 8 e2e HTTP `contact_payment_terms_e2e.rs` (labels FR/DE/comptant, bornes, défauts facture, langue contact, historique, validation, legacy), 5 unit vitest `addDaysIso`, 2 scénarios Playwright (contacts + invoices) + fixture `paymentTermsDays?`.
- T8 : CHANGELOG [Non publié] + README roadmap v0.7 🚧 (E21).

### File List

- crates/kesh-db/migrations/20260712000001_contacts_default_payment_terms_days.sql (nouveau)
- crates/kesh-db/src/entities/contact.rs
- crates/kesh-db/src/repositories/contacts.rs
- crates/kesh-db/tests/migrations_upgrade_path.rs
- crates/kesh-db/tests/{credit_notes,reconciliation,supplier_invoices,payment_batches}_repository.rs, invoices_validate_vat.rs, kf005_fulltext_index_e2e.rs
- crates/kesh-db/src/repositories/invoices.rs (2 littéraux tests)
- crates/kesh-api/src/routes/contacts.rs
- crates/kesh-api/src/routes/invoices.rs
- crates/kesh-api/src/routes/invoice_email.rs (resolve_language pub(crate) + littéral test)
- crates/kesh-api/src/exports/csv_tables.rs
- crates/kesh-api/tests/contact_payment_terms_e2e.rs (nouveau)
- crates/kesh-api/tests/{invoice_delete,vat_report,invoice_pdf,invoice_send_email,inbox_import,invoice_echeancier,reconciliation}_e2e.rs (littéraux)
- crates/kesh-report/tests/vat_report_reconciliation.rs (littéral)
- crates/kesh-reconciliation/src/matching.rs (littéral test)
- crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl
- docs/migrations-idempotence-audit.md
- frontend/src/routes/(app)/contacts/+page.svelte
- frontend/src/lib/features/contacts/contacts.types.ts
- frontend/src/lib/components/invoices/InvoiceForm.svelte
- frontend/src/lib/features/invoices/invoice-helpers.ts + invoice-helpers.test.ts
- frontend/tests/e2e/helpers/api-fixtures.ts, contacts.spec.ts, invoices.spec.ts
- CHANGELOG.md, README.md

## Change Log

### Validate Pass 1 (2026-07-12, Sonnet 4.6, contexte frais)

5 findings, tous patchés : **V1-1 CRITICAL** balayage `NewContact {` manquant (20 sites de construction en littéral dans 13 fichiers de tests, 3 crates → E0063 au build workspace) → AC 6-bis + T1 ; **V1-2 HIGH** AC9 : 3 des 5 handlers contacts n'ont pas `get_company_for` (get/update/archive), list jette le résultat, `archive_contact` était un angle mort → AC 9-bis (helper `contact_response_with_label` sur les 5 sites) ; **V1-3 MEDIUM** message de borne : chaîne FR en dur (pattern validate_common, zéro i18n dans contacts.rs) au lieu du « message i18n » ambigu ; **V1-4 LOW** ligne Total 50→51 de l'audit idempotence ; **V1-5 LOW** `Locale::from(as_str())` = idiome inline, pas une fonction. 30+ refs chemin:ligne vérifiées OK par la passe (listées dans le rapport). Trend > LOW : 3 → à confirmer Pass 2.

### Validate Pass 2 (2026-07-12, Haiku 4.5, contexte frais) — CONVERGÉ

**0 finding > LOW, 0 hallucination.** Re-vérification ground-truth des patches Pass 1 (22 sites `NewContact {` confirmés au compte exact, 5 handlers contacts aux lignes citées, idiome `Locale::from` confirmé `invoice_email.rs:283`), matrice de cohérence ACs↔Tasks↔planning D1-D4 complète, 8 pièges Dev Notes tous validés fondés. **Trend > LOW : 3 → 0 — critère d'arrêt CLAUDE.md atteint (budget 2/8 passes). Spec prête pour `bmad-dev-story 21-1`.** Modèles : Pass 1 Sonnet 4.6, Pass 2 Haiku 4.5 (auteur spec : Fable 5).

### Dev (2026-07-12, Fable 5, run unique) — COMPLETED, status review

Implémentation intégrale T1→T8, 2 déviations additives documentées (export CSV contacts + littéraux hors liste — cf. Debug Log), 1 bug attrapé par les e2e (marques BiDi Fluent U+2068/U+2069 strippées dans `payment_terms_label` — le libellé part sur le PDF Helvetica).
**Quality gate Test Locally First tout vert** : `cargo fmt --check` OK · `cargo build --workspace --all-targets` OK · `clippy -D warnings` 0 · **workspace série 93 suites / 1793 tests / 0 échec** (dont 8 nouveaux e2e `contact_payment_terms_e2e` + test repo no-op) · frontend `check` 0 erreur · `lint-i18n-ownership` PASS · **unit 382/382** (+5 `addDaysIso`) · `build` OK · **Playwright 18 passed / 2 skips pré-existants EXIT=0** (backend `kesh_e2e`, dont les 2 nouveaux scénarios #245). Prochaine : `bmad-code-review 21-1` (LLM ≠ Fable).

### Code review Pass 1 (2026-07-12, Sonnet 4.6 × 3 reviewers parallèles — Blind Hunter + Edge Case Hunter + Acceptance Auditor)

**21 findings bruts → 5 MEDIUM patchés + 1 LOW patché + 5 LOW documentés/dismiss. 0 CRITICAL/HIGH. AA : 29/29 ACs SATISFAITS** (2 déviations dev confirmées légitimes, Dev Record vérifié — 8 e2e comptés, strip BiDi grep-confirmé complet contre le source fluent-bundle).

Patchs appliqués :
- **BH-1 MEDIUM** — `req.date + Duration::days(n)` panique sur overflow chrono (date proche `NaiveDate::MAX` acceptée par serde, chemin de panic introduit par la story) → `checked_add_signed` + 400 Validation + test e2e `invoice_date_overflow_returns_400_not_panic` (+262142-10-01 + 365j).
- **ECH-1 MEDIUM** — `days=1` → « Payable à 1 jours net » ×4 langues (imprimé sur le PDF client) → **sélecteurs pluriel Fluent** `[one]/*[other]` sur les 4 clés + assertions e2e FR « 1 jour net » / DE « 1 Tag ».
- **BH-2/ECH-4 MEDIUM** — texte libre désactivé mais toujours soumis → résurrection d'un texte legacy à l'effacement du délai. Fix : payload envoie `defaultPaymentTerms: null` quand le délai est renseigné (le délai REMPLACE le texte) + assertion Playwright (texte legacy saisi → délai → save → reopen : champ vide).
- **ECH-2 MEDIUM** — libellé auto généré même quand `dueDate` explicite diverge (« Payable à 30 jours net » sur une échéance à 9 j) → le libellé n'est généré **que si l'échéance vient aussi du délai** (`due_from_contact_days`) + 2 tests e2e combinaisons mixtes.
- **AA-1 MEDIUM** — trou de couverture : le fix HIGH du validate (label sur update/archive) non protégé par CI → assertions e2e PUT update (45 j → « Payable à 45 jours net ») + archive.
- **AA-2 LOW** — assertion `audit_log` directe ajoutée au test repo (1 entrée `contact.updated` pour l'update isolé).

Dismiss/documentés (LOW) : ECH-3 (prefill non réactif au changement de `date` post-sélection — comportement assumé, `date` jamais vide dans le flux normal) ; ECH-5 (TOCTOU théorique entre le fetch « paire modifiée » et le verrou optimiste — aucune corruption possible, message 400 vs 409 dans une fenêtre négligeable) ; AA-3 (compte « 20 sites » de la prose AC 6-bis = 19 réels — erreur de texte de spec pré-dev, le code couvre les 22 sites réels) ; AA-4 (le libellé auto contourne le plafond 255 — invariant sûr < 60 chars documenté). +1 fix de flake test : ré-ouverture du ContactPicker avec valeur identique ne re-déclenche pas la recherche → `fill('')` d'abord.

**Gate re-vert intégral post-patchs** : fmt/clippy 0 · workspace série **93 suites / 1794 tests / 0 échec** · e2e story 9/9 · frontend check 0 / unit 382 / build OK · **Playwright 18 passed EXIT=0**. Trend > LOW : 5 → à confirmer Pass 2 (Haiku, diff aplati).
