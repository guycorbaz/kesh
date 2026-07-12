# Story 21.1: Conditions de paiement structurées (jours) sur le contact → échéance de facture calculée

Status: ready-for-dev

<!-- Spec créée le 2026-07-12 (bmad-create-story, Fable 5). Source : planning epic-21-echeances-relances.md §A (décisions D1-D4) + issue #245 + cartographie 2 agents Explore (backend + frontend). Ferme #245. -->

## Story

En tant que **comptable d'une PME**,
je veux **définir un délai de paiement en jours sur la fiche contact et voir l'échéance et les conditions de paiement de mes factures pré-remplies automatiquement**,
afin de **ne plus saisir manuellement ces informations à chaque facture et d'obtenir des échéances fiables pour le futur cycle de relance (#231)**.

## Acceptance Criteria

### Base de données & entité

1. **Migration** `crates/kesh-db/migrations/20260712000001_contacts_default_payment_terms_days.sql` : `ALTER TABLE contacts ADD COLUMN default_payment_terms_days INT NULL;` + `ADD CONSTRAINT chk_contacts_payment_terms_days CHECK (default_payment_terms_days IS NULL OR (default_payment_terms_days >= 0 AND default_payment_terms_days <= 365));` — calque exact de `20260709000001_contacts_language_salutation.sql` (ADD COLUMN puis ADD CONSTRAINT séparé, en-tête de commentaire « Non-breaking → pas de bump kesh_version_min_required, politique Story 10-2 P1 »).
2. **Ligne ajoutée à `docs/migrations-idempotence-audit.md`** (garde-fou P5 — sinon finding MEDIUM en code review).
3. **Compteur de migrations** : `migrations_upgrade_path.rs` passe 50 → 51 (famille « nouvelle migration → compteur figé » — 3 régressions de cette famille à l'Epic 20). Aucune nouvelle table → `TABLES_TO_TRUNCATE` (backup.rs) et compteur export inchangés.
4. **Entité** (`crates/kesh-db/src/entities/contact.rs`) : `default_payment_terms_days: Option<i32>` ajouté à `Contact` (:172-204), `NewContact` (:209-235), `ContactUpdate` (:244-269).
5. **Repo** (`crates/kesh-db/src/repositories/contacts.rs`) — TOUS les sites à liste de champs étendus : `COLUMNS` (:28-31) **et** `FIND_BY_ID_SQL` (:33-36, liste dupliquée), INSERT + bindings de `create` (:189-264), SET + bindings de `update` (:396-515), `contact_snapshot_json` (:39-57, audit), **`is_no_op_change` (:374-392)** — sans quoi une modification isolée du champ serait silencieusement ignorée (pas de bump version, pas d'audit). Helpers de test `new_contact` (:639-661) et `contact_to_update` (:1466-1487) étendus.
6. **Balayage workspace `query_as::<_, Contact>`** : vérifier que TOUTE requête qui hydrate un `Contact` utilise `contacts::COLUMNS` ou est étendue (leçon 20-3b1 : 3 listes SQL inline stale → 500 `ColumnNotFound` ; `grep -rn "query_as::<_, Contact>" crates/` et inspection de chaque site).

### API contacts

7. `CreateContactRequest` (:65-97) et `UpdateContactRequest` (:99-131) acceptent `default_payment_terms_days: Option<i32>` (`#[serde(default)]`, camelCase `defaultPaymentTermsDays`). Sémantique PUT full-payload existante : omis ou `null` → efface (aucun tri-state, conforme au reste du form contact).
8. **Validation** dans `validate_common` (:347-354) : si `Some(d)` avec `d < 0 || d > 365` → `AppError::Validation` (400, message i18n avec la borne). Le CHECK SQL n'est que le filet.
9. `ContactResponse` (:139-168) expose `defaultPaymentTermsDays: Option<i32>` **et** `defaultPaymentTermsLabel: Option<String>` — libellé **généré côté serveur** dans la **langue du contact** (fallback `company.instance_language`), présent sur **tous** les endpoints qui renvoient un contact : list, get, create, update. ⚠️ `ContactPicker` du formulaire facture consomme l'endpoint **list** — si le label manque sur list, le pré-remplissage frontend est mort. (Le i18n frontend ne connaît que la locale UI — confirmé `i18n.svelte.ts` — le libellé par langue de contact DOIT venir de l'API.)
10. Le label est calculé par un helper `payment_terms_label(days: i32, locale: Locale, i18n: &I18nBundle) -> String` appelant **directement `bundle.format(&locale, key, Some(&args))`** (PAS `t_args`, qui lit la locale globale de la requête — piège identifié). Précédent d'args Fluent : `AppError::InvoiceTooManyLinesForPdf` (`errors.rs:1032-1046`, `FluentArgs` + `{ $count }`).
11. **Libellés figés** (clés `kesh-i18n/locales/*/messages.ftl` ×4) :
    - `contact-payment-terms-days-label` : FR `Payable à { $days } jours net` · DE `Zahlbar innert { $days } Tagen` (usage CH — PAS « innerhalb von ») · IT `Pagabile entro { $days } giorni` · EN `Payable within { $days } days`
    - `contact-payment-terms-immediate-label` (cas `days == 0`) : FR `Payable au comptant` · DE `Zahlbar sofort` · IT `Pagabile a vista` · EN `Due upon receipt`
12. La résolution de langue réutilise `resolve_language(contact, company)` (`invoice_email.rs:63-65`) promue `pub(crate)` (DRY — pas de duplication), ainsi que le mapping `Language → Locale` existant du même module.

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

- [ ] **T1 — Migration + entité + repo** (AC 1-6)
  - [ ] Migration `20260712000001_contacts_default_payment_terms_days.sql` (+ audit idempotence P5, compteur 50→51)
  - [ ] `Contact`/`NewContact`/`ContactUpdate` + `COLUMNS` + `FIND_BY_ID_SQL` + INSERT/UPDATE bindings + `contact_snapshot_json` + `is_no_op_change` + helpers de test
  - [ ] Balayage `query_as::<_, Contact>` workspace
- [ ] **T2 — API contacts + libellé localisé** (AC 7-12)
  - [ ] Requests + validation 0..365 ; `ContactResponse` days + label sur list/get/create/update
  - [ ] Helper `payment_terms_label` (bundle.format direct + FluentArgs) ; `resolve_language` promue pub(crate) ; clés FTL ×4 (`contact-payment-terms-days-label`, `contact-payment-terms-immediate-label`)
- [ ] **T3 — Défauts facture + validation échéance** (AC 13-17)
  - [ ] `ensure_contact_belongs_to_company` → retourne `Contact` ; défauts due_date + payment_terms dans `create_invoice` ; update inchangé
  - [ ] Validation `due_date >= date` create (400 + FTL ×4) + règle « paire modifiée » à l'update
- [ ] **T4 — Frontend contact** (AC 18-20)
- [ ] **T5 — Frontend facture** (AC 21-23) : `addDaysIso` + prefill + validateClient
- [ ] **T6 — Tests backend** (AC 24-25)
- [ ] **T7 — Tests frontend + E2E Playwright** (AC 26-27) : étendre `api-fixtures.ts` (`paymentTermsDays?`)
- [ ] **T8 — Doc-sync + gate final** (AC 28-29)

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

### Debug Log References

### Completion Notes List

### File List

## Change Log
