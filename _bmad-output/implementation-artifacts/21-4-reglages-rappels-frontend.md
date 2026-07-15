# Story 21.4: Réglages des rappels + templates multi-type/multi-niveau (frontend)

Status: ready-for-dev

<!-- Créée 2026-07-14 par bmad-create-story. Cartographie ground-truth par 3 agents Explore parallèles (vat-rates gabarit / email-templates refactor / index settings + conventions test-i18n). Story FRONTEND (Svelte 5) de l'Epic 21, consomme le socle backend 21-3 (routes dunning-levels + dunning-settings + email-templates avec level_number). Décisions figées : plan epic-21 D7 (exemple délais cumulés), D13 (hint CGV), D20 (refactor multi-type), section D (sélecteur niveau). Un petit ajout backend est inclus (paramètre `?level=` sur les routes email-templates — LOW-3 de 21-3). -->

## Story

En tant qu'**administrateur d'une PME suisse**,
je veux **configurer mes niveaux de rappel (délais + frais) et personnaliser les textes d'e-mail de rappel par niveau depuis l'interface**,
afin de **piloter le cycle de relance sans passer par l'API — avec un aperçu clair de quand chaque rappel partira**.

## Contexte

Le **socle backend est livré** (21-3) : tables `dunning_levels` + `company_dunning_settings`, routes REST, type e-mail `invoice_reminder` avec `level_number` et cascade. Cette story rend le tout **pilotable depuis l'UI**, en deux volets + un ajout transverse :

1. **Nouvelle page `settings/dunning`** — CRUD des niveaux de rappel (délai + frais) + réglage de la période de grâce (singleton), avec l'**exemple calculé des délais cumulés** (D7) et un **avertissement CGV** (D13). Calquée sur `settings/vat-rates` (CRUD collection) + `settings/invoicing` (singleton verrou optimiste).
2. **Refactor `settings/email-templates`** — aujourd'hui **mono-type** (`invoice_send` en dur), à rendre **multi-type** (`invoice_send` + `invoice_reminder`) et **multi-niveau** (sélecteur de niveau 0..N pour `invoice_reminder`). Le backend renvoie déjà **20 templates en config zéro-config** (3 niveaux : 4 langues × [1 invoice_send niv.0 + 4 invoice_reminder niv.0-3]) — **dynamique au-delà** (plus l'entreprise configure de niveaux, plus la liste grandit ; ne jamais hard-coder 20 dans un test — utiliser `maxLevel` dynamique).
3. **Carte « Rappels »** dans l'index `settings/+page.svelte` + clés i18n dans les 4 FTL.

**Ajout backend minimal** : les routes email-templates opèrent aujourd'hui au **niveau 0 en dur** (21-3, LOW-3) ; il faut leur ajouter un paramètre de niveau pour que le sélecteur puisse éditer les templates de niveau 1..N.

**Hors scope** (garde-fous) : liste des factures à rappeler / éligibilité / envoi (21-5a/21-5b) ; compteur dashboard + liens croisés dashboard (21-6) ; balance âgée (21-7).

## Acceptance Criteria

### A. Ajout backend — paramètre de niveau sur les routes email-templates

1. **`?level=<i16>` (défaut 0) sur les 3 handlers single-template** (`crates/kesh-api/src/routes/email_templates.rs`) : `get_email_template` (`:121`), `update_email_template` (`:139`), `restore_email_template_default` (`:183`) lisent un query param `level` (via `axum::extract::Query`, défaut 0 si absent — rétro-compat totale) et le passent à `get_effective`/`upsert_override`/`restore_default` (qui prennent déjà `level_number`) à la place du `0` codé en dur (21-3). **Note (L2 validate P2)** : `restore_email_template_default` est **idempotent** — `restore_default` n'a **pas** de paramètre `version` (contrairement à vat-rates deactivate) ; le handler DELETE prend uniquement `Path` + le query `?level=`, **aucun body**. Ne PAS y ajouter de verrou optimiste. **Validation** : `level >= 0` (sinon 400) ; pour `invoice_send`, tout `level != 0` → 400 (`invoice_send` n'a qu'un niveau 0 — cohérent avec la cascade). Le `list_email_templates` (`:104`) reste inchangé (renvoie déjà tous les niveaux via `max_reminder_level`).
2. **Tests backend** : étendre `crates/kesh-api/tests/email_templates_e2e.rs` — un PUT `?level=2` sur `invoice_reminder` crée un override niveau 2 (vérifiable via GET `?level=2` + présence dans la liste) ; GET `?level=2` sans override retombe sur le défaut Rust niveau 2 ; **DELETE `?level=2` (restore) supprime l'override niveau 2 et le GET `?level=2` retombe sur le défaut Rust niveau 2, PAS sur le générique niveau 0** (M5 validate P1) ; PUT `?level=1` sur `invoice_send` → 400. Gate backend (fmt/clippy/test) vert.

### B. Feature frontend `dunning` (nouvelle) — page réglages rappels

3. **`frontend/src/lib/features/dunning/dunning.types.ts`** (camelCase, calque `vat-rates.types.ts`) :
   - `DunningLevelResponse { id: number; levelNumber: number; delayDays: number; feeAmount: string; version: number }` (montant décimal en **string**, comme `VatRateResponse.rate`).
   - `CreateDunningLevelPayload { delayDays: number; feeAmount: string }` ; `UpdateDunningLevelPayload { delayDays: number; feeAmount: string; version: number }` ; `DeleteDunningLevelPayload { version: number }`.
   - `DunningSettingsResponse { gracePeriodDays: number; seededAt: string | null; version: number }` ; `UpdateDunningSettingsRequest { gracePeriodDays: number; version: number }`.
4. **`frontend/src/lib/features/dunning/dunning.api.ts`** (calque `vat-rates.api.ts`, via `apiClient` de `$lib/shared/utils/api-client`) :
   - `listDunningLevels(): Promise<DunningLevelResponse[]>` → `apiClient.get('/api/v1/dunning-levels')`.
   - `createDunningLevel(payload)` → `apiClient.post`, `updateDunningLevel(id, payload)` → `apiClient.put('/api/v1/dunning-levels/${id}', …)`, `deleteDunningLevel(id, version)` → `apiClient.delete('/api/v1/dunning-levels/${id}', { version })` (version dans le body du DELETE, comme vat-rates).
   - `getDunningSettings()` → `apiClient.get('/api/v1/company/dunning-settings')` ; `updateDunningSettings(req)` → `apiClient.put('/api/v1/company/dunning-settings', req)` (calque `invoices.api.ts:69-77`).
   - `frontend/src/lib/features/dunning/index.ts` : re-exports types + fonctions. **Pas de store** (le CRUD appelle directement — cohérent avec vat-rates ; un store n'est utile qu'en 21-5/21-6).
5. **`frontend/src/routes/(app)/settings/dunning/+page.ts`** : copie exacte du guard Admin de `vat-rates/+page.ts` (`ssr = false` + `if (browser && authState.currentUser?.role !== 'Admin') throw redirect(302, '/')`).
6. **`frontend/src/routes/(app)/settings/dunning/+page.svelte`** (calque `vat-rates/+page.svelte` + section grâce calquée `invoicing`) :
   - **Niveaux** : `onMount(load)` → `listDunningLevels()` ; liste triée par `levelNumber` ; machine à états `Mode` (`{kind:'none'|'create'|'edit';level}`) ; formulaire création (délai jours + frais) qui **append** (le backend calcule `level_number = MAX+1`) ; édition (délai + frais, avec `version`) ; suppression avec confirmation (`version`, le backend renumérote). Validation forme (M4 validate P2) : `delayDays` via `<input type="number" min="0" step="1">` ; `feeAmount` via `<input type="number" min="0" max="10000" step="0.01">` (point décimal, pas virgule — cohérent avec le patron `vat-rates` qui traite `fRate` comme string et laisse le **backend autoritaire** sur les bornes/scale). Le frontend fait une garde légère (champ non vide, parse numérique) et affiche l'erreur backend (400) telle quelle si la validation serveur échoue — pas de duplication stricte de la règle 0..10'000 côté client. Verrou optimiste : 409 `OPTIMISTIC_LOCK_CONFLICT` → toast + reload. Deux canaux d'erreur (`formError` inline + `toast`). `data-testid` : `dunning-level-row`, `dunning-level-new`, `dunning-form-delay/fee/submit/error`, `dunning-level-delete-confirm`.
   - **Période de grâce** (section séparée, calque `invoicing`) : `getDunningSettings()` au load, champ nombre `gracePeriodDays` (≥ 0), `save()` PUT avec `version`, 409 → reload GET (patron `invoicing/+page.svelte:113-132`). `data-testid` : `dunning-grace-input`, `dunning-grace-save`.
   - **Exemple calculé des délais cumulés (D7)** : sous la liste, afficher l'échéancier prévisionnel calculé côté client : niveau N part à **`échéance + grâce + Σ(délais 1..N)`** jours. Ex. grâce 5 + niveaux 10/10/10 → « 1er rappel proposé **15 j** après l'échéance, 2e à **25 j**, dernier à **35 j** ». `$derived` sur `levels` (valeurs **persistées**) + `gracePeriodDays`. `data-testid` : `dunning-example`. **Garde saisie (M3 validate P1)** : l'exemple se calcule sur les **niveaux persistés** (la liste chargée), PAS sur le formulaire de création/édition en cours — ainsi une saisie transitoire vide/non-numérique ne produit jamais `NaN j` ni crash. (Il se rafraîchit naturellement après un save réussi qui recharge la liste.) La grâce affichée peut être la valeur du champ si numérique, sinon retomber sur la dernière persistée.
   - **Avertissement CGV (D13)** : un encart d'information (pas une erreur) rappelant que **les frais de rappel ne sont exigibles qu'avec une base contractuelle (CGV)** et qu'ils **ne sont pas inclus dans le QR de la facture**. `data-testid` : `dunning-cgv-hint`.
   - **IDs DOM** : `$props.id()` (JAMAIS `crypto.randomUUID` — contrainte HTTP LAN NAS, cf. #145 ; référence `invoicing/+page.svelte:27-29`).
   - **RBAC UI** : protection par le `+page.ts` (redirect non-Admin) ; pas de masquage de bouton (cohérent vat-rates). Défense en profondeur backend déjà en place (`require_admin_role`).

### C. Refactor `email-templates` multi-type / multi-niveau

7. **Types** (`email-templates.types.ts`) : ajouter `levelNumber: number` à `EmailTemplateResponse` (`:18-30` — le backend l'expose déjà, le front l'ignore). Corriger le **commentaire obsolète `:19`** sur `templateType` (« v1 : `'invoice_send'` uniquement ») → mentionner `invoice_send` + `invoice_reminder` (M2 validate P1). Optionnel : unionner `templateType: 'invoice_send' | 'invoice_reminder'`. (Le commentaire d'en-tête « 4 combinaisons » à corriger est dans `api.ts:8-11`, couvert par AC 8 — pas dans ce fichier.)
8. **API client** (`email-templates.api.ts`) : `templateUrl(templateType, language, level)` (`:30-32`) ajoute `?level=${level}` (défaut 0) ; les 3 fonctions `getEmailTemplate`/`updateEmailTemplate`/`restoreEmailTemplateDefault` gagnent un paramètre `levelNumber` propagé à l'URL. `listEmailTemplates` inchangé. Corriger le commentaire d'en-tête (`:8-11`).
9. **Page** (`email-templates/+page.svelte`) — **refactor du modèle d'état vers une clé composite (type × niveau × langue)** :
   - Remplacer `templateType = $state('invoice_send')` (`:48`) par un **sélecteur de type** (`invoice_send` | `invoice_reminder`) + un **sélecteur de niveau** visible **uniquement pour `invoice_reminder`** (niveaux `0..maxLevel`, où `maxLevel` = plus grand `levelNumber` présent dans la liste chargée **pour `invoice_reminder`**). Supprimer la logique « type = premier élément » (`:90-94`).
   - **⚠️ Reset du niveau au changement de type (H1 validate P1)** : basculer `activeType` vers `invoice_send` DOIT forcer `activeLevel = 0` **avant tout rendu** — `invoice_send` n'existe qu'au niveau 0 (backend `email_templates.rs:156` : `InvoiceSend => vec![0]`). Sans reset, la clé `invoice_send:2:FR` est absente des maps → `activeDraft` devient `undefined` → `activeDraft.subject` lève une `TypeError` en pleine UI. Le sélecteur de niveau est de toute façon masqué pour `invoice_send`, mais l'état interne doit être cohérent.
   - **`templates` et `drafts` indexés par clé composite string plate** `` `${type}:${level}:${lang}` `` (L3 validate P1 : trancher pour la **clé string plate**, PAS un `Record` imbriqué — évite le risque `Cannot set properties of undefined` sur un bucket `type→level` non pré-initialisé, et itère trivialement sur les items reçus). PAS d'indexation par langue seule (`:33-47`), sinon collision : le backend renvoie plusieurs items par langue (types/niveaux différents). Construire ces maps depuis les items de `listEmailTemplates()`.
   - **INVARIANT ANTI-RÉGRESSION (bug 20-2)** : changer de **type**, de **niveau** OU de **langue** NE DOIT JAMAIS re-fetch ni écraser un brouillon en cours. La ré-hydratation (`syncDraftFromTemplate`) reste réservée aux moments load initial / save / reload-post-409 / restore (comme `selectLang` ne ré-hydrate pas, `:77-82`). Les `bind:value` pointent le brouillon de la combinaison active.
   - **`allowedVariables` suit la combinaison active** (`:49`) : `invoice_send` et `invoice_reminder` ont des jeux de variables différents (le backend les fournit par item) — l'afficher depuis l'item `(type, level, lang)` courant, pas un `$state` global figé.
   - **Titre/sélecteurs i18n** : remplacer le titre statique `email-templates-type-invoice_send` (`:209`) par les sélecteurs. Libellés : `email-templates-type-invoice_send` (« Envoi de facture »), `email-templates-type-invoice_reminder` (« Rappel de facture »), niveau `email-templates-level-generic` (« Générique », niveau 0) et `email-templates-level-n` (« Rappel {$n} » via interpolation `i18nMsg(key, fb, { n })`). **Le libellé « Mise en demeure » pour le dernier niveau est HORS SCOPE 21-4** (M4 validate P1) — il concerne les badges/liste/historique de rappels (section D de l'epic), donc **21-6**. En 21-4 (réglages), le sélecteur de niveau se contente de « Générique » / « Rappel N » (L21-7 : libellés i18n génériques, pas de label custom).
   - Préserver : verrou optimiste `version` par combinaison, restaurer défaut (DELETE `?level=`), validation tokens 422 (`EMAIL_TEMPLATE_UNKNOWN_VARIABLES`), garde Admin, `$props.id()`.
   - **Recommandation** : extraire l'éditeur (subject/body/variables/save/restore) en composant `lib/features/email-templates/TemplateEditor.svelte` pour ne pas faire exploser `+page.svelte` avec 3 dimensions — **si** extrait, toutes ses clés i18n doivent être `email-templates-*` (lint-ownership scanne `lib/features/`).

### D. Index settings + i18n

10. **Carte « Rappels »** dans `frontend/src/routes/(app)/settings/+page.svelte` : insérer une `<section>` (précédée d'un `<Separator />`) après la section email-templates (`~:297`, avant la fermeture `</div>` `:298`), calquée sur la carte vat-rates (`:275-284`) : titre `dunning-title` (« Rappels débiteurs »), `<Button href="/settings/dunning" data-testid="settings-dunning-manage-link">` libellé `settings-manage` (« Gérer »), description `settings-dunning-link` avec le **fallback FR** (M3 validate P2) : « Configurez les niveaux de rappel (délais et frais), la période de grâce, et personnalisez les textes d'e-mail de rappel par niveau. »
11. **Clés i18n dans les 4 FTL** `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (le frontend les charge via `GET /api/v1/i18n/messages` ; le fallback FR est inline dans `msg()/i18nMsg()`). **⚠️ Contrainte lint-ownership (M1 validate P2)** : toute clé référencée depuis un composant sous `lib/features/dunning/` DOIT être préfixée `dunning-` (le lint scanne `lib/features/`). **Jeu de clés à créer (exhaustif, ajuster les libellés au besoin)** :
    - **Page dunning** : `dunning-title` (« Rappels débiteurs »), `dunning-subtitle`, `dunning-levels-heading` (« Niveaux de rappel »), `dunning-col-level`, `dunning-col-delay`, `dunning-col-fee`, `dunning-delay-label` + `dunning-delay-help` (« Jours depuis l'étape précédente (échéance + grâce pour le 1er) »), `dunning-fee-label`, `dunning-level-new` (« Ajouter un niveau »), `dunning-form-submit`, `dunning-form-cancel`, `dunning-delete` + `dunning-delete-confirm-title`/`-confirm-body`/`-confirm-action`, `dunning-created`/`dunning-updated`/`dunning-deleted` (toasts), `dunning-form-error-delay`, `dunning-form-error-fee`, `dunning-load-error`, `dunning-conflict` (409).
    - **Grâce** : `dunning-grace-heading`, `dunning-grace-label`, `dunning-grace-help`, `dunning-grace-save`, `dunning-grace-saved`.
    - **Exemple D7** : `dunning-example-heading`, `dunning-example-line` (interpolé « {$level}. rappel proposé {$days} j après l'échéance » via `i18nMsg(key, fb, { level, days })`).
    - **Hint CGV** : `dunning-cgv-hint`.
    - **Index settings** : `settings-dunning-link` (description de la carte — namespace `settings-`, comme `settings-vat-rates-link`).
    - **email-templates** : `email-templates-type-invoice_reminder` (« Rappel de facture »), `email-templates-level-generic` (« Générique »), `email-templates-level-n` (« Rappel {$n} », interpolé).
    Traduire aux 4 langues au standard des clés existantes.

### E. Tests + gate

12. **Vitest (unit)** : `frontend/src/lib/features/dunning/dunning.api.test.ts` (calque `settings.api.test.ts` : `vi.mock('$lib/shared/utils/api-client')`, assert les URLs + `version` sur create/update/delete/settings). Étendre `email-templates.api.test.ts` pour le paramètre `?level=`.
13. **Playwright (e2e)** : `frontend/tests/e2e/dunning.spec.ts` (calque `homepage-settings.spec.ts` + `vat-rates.spec.ts`) : login Admin → `/settings/dunning` → voir les 3 niveaux seedés + la grâce + l'exemple calculé ; créer/éditer/supprimer un niveau ; éditer la grâce ; a11y axe. **Étendre `email-templates.spec.ts`** — **séquence anti-régression brouillons précise (M2 validate P2, calque `spec.ts:87-105` étendu aux dimensions type+niveau)** :
    1. Login Admin → `/settings/email-templates`, attendre le chargement.
    2. Sélectionner type `invoice_reminder`, niveau 2, langue FR ; saisir subject/body uniques (« brouillon-niv2-fr ») **sans enregistrer**.
    3. Basculer niveau 1 (même type/langue) → vérifier que le formulaire N'AFFICHE PAS « brouillon-niv2-fr » (brouillon du niveau 1, distinct).
    4. Revenir niveau 2 → vérifier que « brouillon-niv2-fr » est **toujours là** (pas re-fetch).
    5. Basculer langue DE (même type/niveau) → brouillon FR:2 absent ; revenir FR → brouillon FR:2 présent.
    6. Basculer type `invoice_send` → vérifier reset niveau + pas de crash (H1) ; revenir `invoice_reminder`/niveau 2 → brouillon FR:2 présent.
    7. Enregistrer → 200, puis re-vérifier l'affichage. (Le principe : changer type/niveau/langue ne re-fetch JAMAIS et ne perd JAMAIS un brouillon en cours.)
    Pré-requis run E2E : backend `KESH_TEST_MODE=true` contre `kesh_e2e` + `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (Ubuntu 26+).
14. **Gate complet** : backend (fmt/clippy/test) pour l'ajout A ; **frontend** `npm run check` + `npm run lint-i18n-ownership` + `npm run test:unit` + `npm run build` (les 4 checks CI, cf. CLAUDE.md §Frontend). E2E localement (touche des pages).
15. **CHANGELOG** `[Non publié]` : entrée `Ajouté` (réglages des rappels débiteurs + personnalisation des textes de rappel par niveau). **README « Feuille de route »** : vérifier que E21 reste cohérent (socle → UI ; pas de `(à venir)` à retirer tant que l'envoi 21-5 n'est pas là).

## Tasks / Subtasks

- [ ] **T1 — Backend `?level=`** (AC 1-2) : query param niveau sur get/put/restore email-templates + validation + tests e2e.
- [ ] **T2 — Feature `dunning` (types + api)** (AC 3-4) : `dunning.types.ts` + `dunning.api.ts` + `index.ts`.
- [ ] **T3 — Page `settings/dunning`** (AC 5-6) : `+page.ts` guard + `+page.svelte` (CRUD niveaux + grâce + exemple cumulés D7 + hint CGV D13).
- [ ] **T4 — Refactor email-templates** (AC 7-9) : types `levelNumber` + api `?level=` + page clé composite + sélecteurs type/niveau + invariant brouillons.
- [ ] **T5 — Index + i18n** (AC 10-11) : carte settings + clés `dunning-*`/level dans les 4 FTL.
- [ ] **T6 — Tests + gate** (AC 12-15) : vitest + Playwright + gate frontend + backend + CHANGELOG.

## Dev Notes

### Pièges identifiés (ground-truth 2026-07-14, 3 agents Explore)

- **Collision d'indexation email-templates** : `templates`/`drafts` sont indexés par langue seule aujourd'hui (`+page.svelte:33-47`, boucle `:87-89`). Le backend renvoie désormais **20 items** (type × langue × niveau) → l'indexation par langue en écrase la plupart. **Clé composite obligatoire** `(type, level, lang)`.
- **Invariant brouillons (bug 20-2)** : `selectLang` NE ré-hydrate PAS (`:77-82`, commentaire ligne 79) ; `syncDraftFromTemplate` (`:72-75`) réservé à load/save/reload/restore. À PRÉSERVER en étendant aux dimensions type + niveau. Test garde-fou `email-templates.spec.ts:87-105` à étendre.
- **`allowedVariables` par combinaison** : `invoice_send` ≠ `invoice_reminder` (jeux de variables différents, fournis par le backend par item) — ne pas garder un `$state` global (`:49`).
- **Pas d'API secure-context en HTTP LAN** : `$props.id()` pour les IDs DOM, jamais `crypto.randomUUID`/`crypto.subtle`/`navigator.clipboard` (page blanche sur NAS HTTP, #145). Référence `settings/invoicing/+page.svelte:27-29`.
- **i18n canonique** : importer `i18nMsg` depuis `$lib/shared/utils/i18n.svelte` (PAS depuis `onboarding.svelte`, même si `settings/+page.svelte:6` le fait encore). Interpolation `{$n}` via `i18nMsg(key, fb, { n })`.
- **lint-i18n-ownership** : scanne **uniquement** `src/lib/features/**` (pas `routes/`). Les clés dans un composant `lib/features/dunning/` DOIVENT être `dunning-*` (préfixe = nom du dossier). Les pages sous `routes/settings/*` ne sont pas scannées, mais garder la convention par cohérence.
- **Verrou optimiste** : code d'erreur canonique `OPTIMISTIC_LOCK_CONFLICT` (409) → reload GET (patron `invoicing/+page.svelte:113-132`). `version` dans le body du DELETE (comme vat-rates).
- **Backend `?level=` rétro-compat** : défaut 0 si absent → les appels existants (et 21-3) restent valides.

### Patterns à réutiliser (ne PAS réinventer)

- **Page CRUD collection** : `settings/vat-rates/+page.svelte` (machine à états `Mode`, `onMount(load)`, `formError`+`toast`, `version`) + `vat-rates/{api,types}.ts` + `+page.ts` guard Admin.
- **Singleton config verrou optimiste** : `settings/invoicing/+page.svelte` (GET-or-create, PUT + version, 409 reload) + `invoices.api.ts:69-77`.
- **Carte index** : `settings/+page.svelte:275-297` (section vat-rates/email-templates).
- **API** : `apiClient` (`$lib/shared/utils/api-client`) — `.get/.post/.put/.delete(url, body?)`. Erreurs via `isApiError(e)` + `e.code`/`e.message`/`e.status`.
- **Tests** : vitest `settings.api.test.ts` (mock apiClient) ; Playwright `homepage-settings.spec.ts` (login/seed/axe) + `vat-rates.spec.ts` (CRUD) ; fixtures `tests/e2e/helpers/{test-state.ts,api-fixtures.ts}`.
- **Query extractor Axum** (M6 validate P1 — le patron `ListVatRatesQuery` de `routes/vat.rs:40` est `history: Option<bool>` **sans** `#[serde(default)]`, donc PAS un précédent exact) : pour un défaut 0 sans `Option`, deux voies au choix — (a) `#[derive(Deserialize)] struct LevelQuery { #[serde(default)] level: i16 }` (absent → `i16::default() == 0`), OU (b) `level: Option<i16>` puis `.unwrap_or(0)` dans le handler (calque littéral de `ListVatRatesQuery`). La voie (b) est la plus proche du précédent existant ; la voie (a) est plus concise. Trancher à l'implémentation.

### Hors scope (garde-fous anti-creep)

- **AUCUNE liste à rappeler / éligibilité / envoi** → 21-5a/21-5b.
- **AUCUN compteur dashboard ni liens croisés dashboard** → 21-6.
- **AUCUNE balance âgée** → 21-7.
- Pas de store `dunning` (inutile pour le CRUD ; viendra si 21-5/21-6 en ont besoin).
- Pas de label custom par niveau (libellés i18n génériques « Rappel N », L21-7).

### Dérogation / watch-point règle de splitting (CLAUDE.md)

Cette story touche ~6 modules frontend/backend (`lib/features/dunning` neuf, `lib/features/email-templates` refactor, `routes/settings/{dunning,email-templates,index}`, i18n ×4, tests, + ajout backend routes email-templates). Deux volets à profils de risque différents : **B (page dunning)** = gabarit mécanique ; **C (refactor email-templates)** = plus délicat (état à 3 dimensions + invariant anti-régression). Le plan d'epic les a scopés en une story. **Décision** : suivre le plan, MAIS si `bmad-create-story validate` dépasse **4 passes** sans converger (critère 2), **splitter en 21-4a (page dunning : A backend `?level=` + B feature/page dunning + carte index) / 21-4b (refactor email-templates multi-type/niveau)** — les deux sont indépendantes (la page dunning ne dépend pas du refactor email). Précédent : 21-2 splittée après divergence.

### Project Structure Notes

- **Nouveaux** : `routes/(app)/settings/dunning/{+page.svelte,+page.ts}`, `lib/features/dunning/{dunning.types.ts,dunning.api.ts,index.ts,dunning.api.test.ts}`, `tests/e2e/dunning.spec.ts`.
- **Modifiés** : `routes/(app)/settings/email-templates/+page.svelte`, `lib/features/email-templates/{email-templates.api.ts,email-templates.types.ts,email-templates.api.test.ts}`, `routes/(app)/settings/+page.svelte`, `tests/e2e/email-templates.spec.ts`, `crates/kesh-api/src/routes/email_templates.rs`, `crates/kesh-api/tests/email_templates_e2e.rs`, `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, `CHANGELOG.md`.

### References

- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md — D7 (exemple délais cumulés), D13 (hint CGV), D20 (refactor multi-type), section D (option A+ / sélecteur niveau), L21-7 (libellés génériques)]
- [Source: 21-3-socle-config-rappels.md — socle backend consommé (routes, level_number, cascade) + LOW-3 (segment niveau en 21-4)]
- [Source: frontend/src/routes/(app)/settings/vat-rates/+page.svelte + vat-rates/{api,types}.ts + +page.ts — gabarit CRUD]
- [Source: frontend/src/routes/(app)/settings/invoicing/+page.svelte + invoices.api.ts:69-77 — patron singleton verrou optimiste]
- [Source: frontend/src/routes/(app)/settings/email-templates/+page.svelte (:33-47,48,72-94,209) + email-templates.{api,types}.ts — état à refactorer]
- [Source: frontend/src/routes/(app)/settings/+page.svelte:275-297 — carte index]
- [Source: frontend/scripts/lint-i18n-ownership.js — règle ownership ; docs/i18n-key-ownership-pattern.md]
- [Source: docs/optimistic-locking-patterns.md, docs/testing.md — patrons 409 + recette E2E]
- [Source: CLAUDE.md §Frontend (checks CI), §Pas d'API secure-context HTTP LAN]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Validate Pass 1 (2026-07-14, Sonnet 4.6) — 1 HIGH + 6 MEDIUM + 3 LOW, patchés

Passe adversariale grep ground-truth (~25 refs frontend/backend vérifiées, quasi toutes exactes ; formule D7 confirmée). Split 21-4a/21-4b **non** recommandé (findings localisés/mécaniques). Remédiés :
- **H1** — AC 9 ne resetait pas `activeLevel` au passage à `invoice_send` (qui n'existe qu'au niveau 0) → clé `invoice_send:2:FR` absente → `activeDraft.subject` `TypeError`. **Patch** : reset `activeLevel = 0` forcé avant rendu (AC 9).
- **M1/M2** — AC 7 : la ref « 4 combinaisons » est dans `api.ts:8` (couverte par AC 8), pas dans `types.ts` ; le vrai commentaire obsolète est `types.ts:19` (« v1 : invoice_send uniquement »). **Patch** : AC 7 recadrée.
- **M3** — AC 6 : garde de l'exemple calculé sur saisie vide/NaN non spécifiée. **Patch** : l'exemple se calcule sur les niveaux **persistés** (jamais le formulaire en cours) → pas de `NaN j`.
- **M4** — AC 9 : libellé « Mise en demeure » sous-spécifié → **différé à 21-6** (badges/liste/historique), retiré de 21-4.
- **M5** — AC 2 : pas de test DELETE `?level=`. **Patch** : test restore niveau 2 → retombe sur défaut Rust niveau 2 (pas générique 0).
- **M6** — Dev Notes : `ListVatRatesQuery` cité comme précédent `#[serde(default)]` mais c'est `Option<bool>` sans. **Patch** : 2 voies explicitées (`#[serde(default)] i16` ou `Option<i16>`+`unwrap_or(0)`).
- **L1** — « 20 templates » = valeur zéro-config, dynamique au-delà (reformulé, anti-hardcode test).
- **L2** — `maxLevel` filtré par `invoice_reminder` (AC 9).
- **L3** — clé composite **string plate** tranchée (pas Record imbriqué → évite `Cannot set properties of undefined`).

**Trend** : Pass 1 → 1 HIGH / 6 MEDIUM (> LOW) → patchés. Relance Pass 2 (LLM différent, contexte frais).

### Validate Pass 2 (2026-07-15, Haiku 4.5, contexte frais) — 0 HIGH, 4 MEDIUM + 2 LOW, patchés

Passe adversariale. **Les 6 patches Pass 1 confirmés sans régression** (H1 reset, L3 clé string, M3 exemple persisté, M5 test DELETE, M4 hors-scope, M6 2 voies Query). Findings (tous des **gaps de clarté de spec**, pas des bugs de logique) remédiés :
- **M1** — AC 11 : clés i18n non énumérées → **liste exhaustive** ajoutée (page/grâce/exemple/CGV/index/email-templates) + rappel contrainte lint-ownership (`dunning-*` sous `lib/features/dunning/`).
- **M2** — AC 13 : séquence E2E anti-régression brouillons imprécise → **7 étapes précises** (niveau 2→1→2, langue FR→DE→FR, type reset H1).
- **M3** — AC 10 : texte `settings-dunning-link` manquant → **fallback FR ajouté**.
- **M4** — AC 6 : format `feeAmount` imprécis → `<input type="number" step="0.01">` + backend autoritaire (patron vat-rates).
- **L2** — AC 1 : clarifié que `restore_email_template_default` est **idempotent** (grep confirmé : `restore_default` n'a PAS de `version`, pas de body — Haiku confondait avec vat-rates deactivate). Ne pas y ajouter de verrou optimiste.
- **L1** — extraction `TemplateEditor.svelte` reste une recommandation (dette cosmétique à revoir en rétro si `+page.svelte` > seuil LOC).

**Trend** : Pass 1 (1H/6M) → Pass 2 (0H/4M) → patchés. Relance Pass 3 (Opus, contexte frais). Split toujours non recommandé (findings de clarté, pas de complexité émergente).
