# Story v0190-nom-societe : le nom de la société se modifie dans *Paramètres → Organisation* (#1097)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 (Sonnet 5.5), worktree `kesh-v0190c`, branche `story/v0190-nom-societe` à créer depuis
`epic/v0190-correction` (= `main` = v0.18.0, `8b50fd195`), par le workflow `bmad-create-story`. Issue de la scission de
`v0190-correction` (règle de splitting : six modules ; décision de l'orchestrateur du 2026-11-17). Release **0.19.0** si elle est close
le jeudi 2026-11-19 au soir, sinon **0.20.0** (accord de Guy du 2026-11-17). **Aucune migration** : `companies.name` existe
(`VARCHAR(255) NOT NULL`, `crates/kesh-db/migrations/20260404000001_initial_schema.sql`, ligne `6:    name VARCHAR(255) NOT NULL,`),
`CompanyUpdate` porte déjà `name` et `companies::update_in_tx` l'écrit déjà (`crates/kesh-db/src/repositories/companies.rs`,
`SET name = ?, first_name = ?, …`) ; si le développement en découvre une, il s'arrête et le dit. Voie de développement : voie 2, clé `v0190c`.*

## Story

En tant qu'administrateur dont le nom saisi à l'assistant est refusé par la QR-facture, je veux corriger le nom de la société dans
*Paramètres → Organisation*, afin de ne pas rester sans remède devant un PDF qui se refuse.

## Ce qui se lit au code (`8b50fd195`)

- Écran : `frontend/src/routes/(app)/settings/+page.svelte:640` `<dd>{data.company.name}</dd>` (section *Organisation*, sans édition). Modèle : le bloc e-mail — état
  `:230-236` (`emailEditing`, `emailBase`, `emailVersion`, `emailConflict`), `startEmailEdit`, `saveEmail` (appelle `updateCompanyEmail`, puis `adoptCompany`, et sur
  `OPTIMISTIC_LOCK_CONFLICT` `reloadAfterConflict`, patron de la 34-6) ; `const EMAIL_FIELDS = ['email'] as const;` `:62`, `isAdmin` `:229`. API :
  `frontend/src/lib/features/settings/settings.api.ts:39` `updateCompanyAddress` (modèle), types `settings.types.ts:58-62` `UpdateCompanyAddressRequest`.
- Serveur : `crates/kesh-api/src/routes/companies.rs` — `update_company_email` (`:112`), `update_company_contact_details`, `update_company_address` (`:344`)
  reconstruisent chacune un `CompanyUpdate` complet avec `name: company.name.clone(),` ; aucune n'écrit `name`. Patron de la trace : `NewAuditLogEntry::user(… "company.updated", "company", …)`,
  `details_json` `{before, after}` limité aux champs touchés, écrite seulement si `updated.version != company.version` (no-op KF-004). Routes : `crates/kesh-api/src/lib.rs:458-473`,
  bloc `admin_routes` (`require_not_pat` : session seulement).
- Contraintes de la QR-facture, publiques dans `kesh-qrbill` : `validation.rs:10` `pub const NAME_MAX: usize = 70;` (non ré-exportée à la racine : `lib.rs:30` n'exporte que
  `champ_vide_pour_qr, first_qr_refused_char, is_iso_3166_alpha2, validate` ; `pub mod validation;` `lib.rs:18` l'expose : `kesh_qrbill::validation::NAME_MAX`) ; `validation.rs:244`
  `pub fn champ_vide_pour_qr` ; `:213` `pub fn first_qr_refused_char`. Le PDF les applique au nom : `validation.rs:317-321`, et `invoice_pdf_service.rs:281` `name: company.name.clone(),`, `:419` `creditor_name: company.name.clone(),`.
- Modèle de validation : `address_input.rs` `validate_required_for_qr` (message qui nomme le champ et le caractère, français littéral, sans clé). L'assistant ne contrôle que « non vide » :
  `crates/kesh-api/src/routes/onboarding.rs:535` `if name.is_empty() {` (et recompose `name` en « prénom nom » pour une personne physique, `:533`) — **issue #1226** (l'assistant n'applique pas les contrôles QR au nom), hors de cette story.
- **Les trois refus du PDF disent que le nom « ne se modifie pas encore depuis l'interface »** — `invoice_pdf_service.rs:624`, `:704`, `:737` (replis) et les mêmes phrases aux quatre `messages.ftl` :
  `invoice-pdf-error-charset-company-name`, `invoice-pdf-error-empty-company-name`, `invoice-pdf-error-too-long-company-name` (fr-CH `:999`, `:1011`, `:1018` ; de-CH `:937`, `:949`, `:956` ; it-CH et en-CH idem).
  Deux gardes les **imposent** : `invoice_pdf_service.rs:1737` `if r.cle.ends_with("-company-name") {` / `!texte.contains("Organi")` et `crates/kesh-api/tests/invoice_pdf_e2e.rs:698`
  (`assert!(!m.contains("organisation"), "{m}");`). La story les **inverse**.
- Manuel : `docs/manual/fr/user-manual.tex`, § *Vos coordonnées sur la facture* (`:1221`), dernière phrase du paragraphe « Corriger votre adresse » : « Le nom et l'IDE ne se modifient pas ici. » ;
  `docs/manual/fr/admin-manual.tex:2667`. `docs/api-external.md` : aucune route `PUT /companies/current/*` n'y figure et elles refusent les clés d'API : **inchangé**.
- Gardes qui énumèrent les routes d'administration : `crates/kesh-api/tests/admin_pat_denied_e2e.rs:83-85`, `crates/kesh-api/tests/textes_coherents.rs:1355` (occurrence assumée du chemin dans `admin-manual.tex`, liste fermée `MULTI_SOCIETE_ASSUME`).

## Décisions

- **D1 — une route dédiée `PUT /api/v1/companies/current/name`** (C-v0190c-4), sur le patron d'`update_company_email` : `UpdateCompanyNameRequest { name: String, version: i32 }`, même routeur Admin-only
  (`lib.rs`, après la route `address`), rend `CompanyJson`. **Option écartée** : étendre `address` ou `contact-details` (blocs sans rapport, et le full-replace efface par omission).
- **D2 — les contrôles** (C-v0190c-5), dans une fonction unique de `companies.rs` (`validate_company_name_for_qr`, DRY, testée à part), appelée **avant** la lecture de la société : `trim` ; **vide** au sens de la
  QR-facture (`champ_vide_pour_qr`, non `trim().is_empty()`) → refus ; **plus de 70 caractères** (`chars().count() > kesh_qrbill::validation::NAME_MAX`) → refus qui dit le maximum et le nombre saisi ; **caractère que la
  QR-facture n'admet pas** (`first_qr_refused_char`) → refus qui nomme le caractère et son point de code (`escape_debug`, `U+{:04X}`, comme `address_input.rs`). `400 VALIDATION_ERROR`, français littéral comme la route
  d'adresse (pas de clé serveur neuve).
- **D3 — limite : `first_name` / `last_name` ne bougent pas** (C-v0190c-6, acceptée par l'orchestrateur le 2026-11-17). Pour une personne physique, l'assistant recompose `name` en « prénom nom » ; l'écran ne montre que `name` et
  le PDF n'emploie que `name` (`invoice_pdf_service.rs:281`, `:419`). La route reporte `first_name` / `last_name` à l'identique (full-replace) : le nom corrigé peut différer de « prénom nom ». Limite écrite, **tracée par l'issue #1227**.
- **D4 — audit et verrou** (C-v0190c-7). Transaction + `companies::update_in_tx` + `audit_log::insert_in_tx` comme les trois routes jumelles : verrou optimiste (`409 OPTIMISTIC_LOCK_CONFLICT`), no-op sans `version` ni audit,
  `company.updated` avec `{"before": {"name": …}, "after": {"name": …}}` — aucune nouvelle action (`audit_labels.rs:106`). Les PDF déjà figés ne changent pas.
- **D5 — l'écran** (C-v0190c-8). Le nom se lit comme aujourd'hui, avec le bouton **Modifier** (`isAdmin`) ; l'édition est le bloc e-mail transposé (`nameEditing`, `nameValue`, `nameBase`, `nameVersion`, `nameConflict`,
  `NAME_FIELDS = ['name'] as const`), mêmes `adoptCompany`, `reloadAfterConflict`, `CompanyConflictNotice` (testid `settings-company-name-conflict`, `field="name"`), `maxlength={70}`. Vide → refus local
  (`settings-company-name-required`) ; longueur et jeu de caractères : le message du serveur s'affiche tel quel. **Option écartée** : recopier le jeu de caractères dans le frontend.
- **D6 — les trois refus du PDF renvoient à l'écran** (C-v0190c-9) : les trois replis Rust et les douze phrases (3 clés × 4 locales) perdent « ne se modifie pas encore depuis l'interface » et disent
  « Corrigez-le dans Paramètres → Organisation » (tournure de `invoice-pdf-error-too-long-company-street`, `invoice_pdf_service.rs:739-743`, `messages.ftl` fr-CH `:1019`). Les deux gardes s'inversent (le texte **contient** « Organi »).
  `invoice_pdf_service.rs:1751` `assert_eq!(cles.len(), 42, "{cles:?}");` ne change pas : aucune clé ajoutée ni retirée.
- **D7 — le manuel** (C-v0190c-10) : `user-manual.tex` — remplacer « Le nom et l'IDE ne se modifient pas ici. » par une phrase qui dit que le **nom** se corrige là (bouton *Modifier* en face de « Nom », administrateurs,
  70 caractères au plus, jeu de caractères de la QR-facture, un PDF figé ne change pas) et que seul l'**IDE** ne s'y modifie pas ; `admin-manual.tex:2667` : `PUT /api/v1/companies/current/name` à côté de la route d'adresse
  (**lire d'abord la garde `MULTI_SOCIETE_ASSUME`, `textes_coherents.rs:1355`**). Pas de PDF commité.

## Critères d'acceptation

1. **AC1 — la route** (`crates/kesh-api/tests/company_address_e2e.rs`, section « Story v0190-nom-societe », harnais existant de ce fichier, un `put_name` à côté de `put_address`) :
   a. `{"name":"Nom corrigé SA","version":v}` → `200`, nom relu égal, `version` + 1, `first_name`/`last_name`/adresse/e-mail/téléphone/site inchangés ;
   b. rogné ; vide, blanc, U+00AD seul → `400`, rien d'écrit ; 70 caractères acceptés, 71 refusés (message : `70` et `71`) ; caractère hors jeu (« 张 », U+0A, U+2028) refusé, message qui le nomme avec son point de code ; `&`, `é`, U+2019 acceptés ;
   c. version périmée → `409` sans écriture ; un nom invalide à version périmée est un `400` (patron `an_invalid_address_at_a_stale_version_is_a_400`) ;
   d. nom identique → no-op : `version` inchangée, aucune entrée d'audit ;
   e. audit : une entrée `company.updated`, `details_json` = `{"before":{"name":…},"after":{"name":…}}` sans autre clé, acteur `user` (`common::audit_actor`) ;
   f. bout en bout : un nom de 71 caractères posé en base fait refuser le PDF d'une facture validée (message renvoyant à Paramètres → Organisation) ; après correction par la route, le PDF se produit (patron `correcting_the_country_restores_the_qr_invoice`, `company_address_e2e.rs:570`) ;
   g. Comptable → `403` ; clé d'API refusée (`admin_pat_denied_e2e.rs` : ajouter `("PUT", "/api/v1/companies/current/name")` à la liste `:83-85`).
2. **AC2 — les contrôles, à part** : tests unitaires de `validate_company_name_for_qr` (`mod tests` de `companies.rs`) : 0, 1, 70, 71 caractères, U+00AD seul, le premier caractère refusé d'une chaîne qui en porte deux.
3. **AC3 — les refus du PDF** : trois messages (replis et catalogues, quatre locales) renvoyant à *Paramètres → Organisation* ; deux gardes inversées ; `cles.len() == 42` ; `cargo test -p kesh-api --test textes_coherents` vert.
4. **AC4 — l'écran** (`settings-page.test.ts`, patron des blocs e-mail et adresse) : *Modifier* du nom réservé à l'administrateur ; l'enregistrement appelle `updateCompanyName({ name, version })`, affiche le nom reçu et le succès ; vide → refus local sans appel ; 400 → message affiché, saisie conservée ; `OPTIMISTIC_LOCK_CONFLICT` → même chemin que l'e-mail. `settings.api.test.ts` : l'appel `PUT` (URL, corps).
5. **AC5 — E2E** (`frontend/tests/e2e/company-address.spec.ts`, ou `company-name.spec.ts`) : Modifier le nom, enregistrer, recharger : nom corrigé affiché ; 71 caractères refusés avec le message du serveur ; **le nom est rendu tel qu'on l'a trouvé** (société partagée, comme `restoreAddress`). Captures (clair seulement) : bouton, édition, refus.
6. **AC6 — compteurs et gardes** : `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts:1040`) recompté ; `npm run lint-i18n-ownership` vert ; clés `settings-company-name-saved`, `-required`, `-conflict`, `-conflict-reload-failed` des quatre catalogues rangées à côté de `settings-company-email-saved` (fr-CH `:2504`), même ordre ; `manuel.rs` et `textes_coherents` verts.
7. **AC7 — le manuel** : D7, `.tex` seulement, contrôle par `git grep`.

## Tâches

1. `companies.rs` : requête, `validate_company_name_for_qr`, `update_company_name` ; route dans `lib.rs` ; tests AC1, AC2 ; `admin_pat_denied_e2e.rs`.
2. `invoice_pdf_service.rs` (trois replis) et quatre `messages.ftl` ; inversion des deux gardes (AC3).
3. `settings.api.ts`, `settings.types.ts`, `+page.svelte` (bloc nom), clés i18n, tests AC4, E2E AC5, `sitesTotal`.
4. Manuels (D7) ; la section `## Note de version` ci-dessous est assemblée par l'orchestrateur (la story n'écrit pas `CHANGELOG.md`).

## Gates

Ceux de `e35-dev-commun.md` : gate ciblé en cours de développement (`npm run check`, Vitest de `settings` ; `cargo nextest run -E 'binary(company_address_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e) | binary(admin_pat_denied_e2e)'`, `fmt`, `clippy` workspace) ; `test-fast --ci` et `gate-ci` au passage en `review` ; `gate-ci` et E2E complet à la clôture, au dernier commit de code. `kesh-db` non touché.

## Recette visuelle (captures, thème clair, base de démonstration, `kesh-gate-logs/v0190c/`)

*Organisation* avec le bouton *Modifier* en face du nom ; édition ; refus de 71 caractères ; refus du PDF d'une facture (message renvoyant à *Paramètres → Organisation*).

## Dev Notes

- Taille : une route, une page, quatre catalogues, ~6 fichiers de test, deux manuels ; cinq modules (kesh-api routes, kesh-api PDF, kesh-i18n, frontend settings, manuels).
- Issues liées (ouvertes) : **#1226** (l'assistant n'applique pas les contrôles de la QR-facture au nom), **#1227** (`first_name`/`last_name` sans écran, limite D3).
- Aucune donnée réelle dans les fixtures (« Démo SA »).

## Note de version

**Corrigé**
- Le **nom de la société** se corrige dans *Paramètres → Organisation* (bouton *Modifier*, administrateurs) : jusqu'à 70 caractères, jeu de caractères de la QR-facture ; le refus du PDF d'une facture dont le nom est vide, trop long ou refusé renvoie désormais à cet écran. Une route d'administration est ajoutée, `PUT /api/v1/companies/current/name` (session seulement), sans changement pour les clés d'API (refs #1097).
