# Story 33-8 : un compte désigné dans les paramètres de facturation ne s'archive plus sans avertissement (#486)

Status: review

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-16 à 18 et 29. Commits en `refs` ; à la PR : `closes #486`.*

## Story

En tant que Comptable qui range son plan comptable, je veux être averti, au moment d'archiver un compte, qu'il est désigné dans les paramètres de facturation, et par
quel réglage ; pour ne pas découvrir l'effet des mois plus tard, à la validation d'une facture ou à un règlement refusé.

## Ce que le code dit (prémisses établies)

- **Aucune garde** : `routes/accounts.rs:335-348` et `repositories/accounts.rs:681-756` ne vérifient que les sous-comptes actifs.
- ⚠️ **Prémisse corrigée — pas quatre réglages, dix.** L'issue en nomme quatre ; `entities/company_invoice_settings.rs:24-62` en porte **dix** : `default_receivable`, `default_revenue`, `default_vat_payable`,
  `default_vat_recoverable`, `default_vat_decompte`, `default_payable`, `default_rounding`, `default_discount`, `default_bank_fees`, `default_bad_debt`. Les libellés d'écran existent (`fr-CH/messages.ftl:680-704`, clés `settings-invoicing-*`) ; les trois de TVA sont `invoices-settings-vat-payable|recoverable|decompte` (`:583-585`, lus).
- **La seconde face, commentée dans #486, est toujours là** : `validate_account_of` refuse tout compte inactif (`routes/company_invoice_settings.rs:200-204`), sans égard pour « inchangé » (seul `require_postable` l'est, `:165`). Les comptes reconduits sont revalidés à chaque `PUT`, pour six champs. Un compte déjà archivé bloque donc l'enregistrement de **n'importe quel** réglage. Les quatre comptes d'écart, eux, sont déjà épargnés (`resolve_designated_account`, `:220-250`, dont le commentaire cite #486).
- **Le patron d'un refus qui se confirme** : `confirm_retype` et `DbError::AccountHasEntries` (`repositories/accounts.rs`, `errors.rs:3425-3450`, écran `accounts/+page.svelte:295`) : 409, ampleur dans `details`, second envoi explicite.
- **Droits** : l'archivage est au routeur Comptable (`lib.rs:525`), le réglage à l'Administrateur. Un **refus sec** enfermerait un Comptable, et un cas réel existe : un non-assujetti qui range ses comptes TVA, désignés d'office (C-e334-16).
- **14 appels** à `accounts::archive(` en `crates/`, presque tous des montages de tests : on les garde intacts (C-e334-17).
- **Écran** : dialogue à `accounts/+page.svelte:730-745`, `submitArchive` `:340-362`.

## Conception (C-e334-16, 17)

- **Avertir, puis laisser faire** : doctrine de #280 (« l'avertissement informe, il n'interdit pas »), et lettre de l'epic (« sans avertissement »).
- `repositories/company_invoice_settings.rs` : `designations_of_account(executor, company_id, account_id) -> Vec<&'static str>`, codes stables (`receivable`, `revenue`, `vatPayable`, `vatRecoverable`, `vatDecompte`, `payable`, `rounding`, `discount`, `bankFees`, `badDebt`) ; **un test d'inventaire compare la liste aux colonnes `default_%_account_id`** lues à `information_schema`, pour qu'un onzième réglage ne se taise pas.
- `accounts::archive_guarded(pool, id, version, user_id, company_id, confirm_designated)` porte le corps ; `archive` reste comme l'appelle le reste du dépôt, `confirm = true`, pour les montages. **Seule la route** passe `req.confirm_designated`. Lecture **dans** la transaction de l'archivage ; course avec un `PUT` de réglages concurrent : la même que `confirm_retype`, écrite comme limite.
- **Ordre des refus (C-e334-29)** : sous-comptes actifs (existant), **puis** désignation, puis version : la version se contrôle à l'`UPDATE` (`repositories/accounts.rs:703-707`), et la désignation se lit juste avant, sur l'état **courant**, non sur celui de l'écran. Un écran périmé sans confirmation reçoit donc le 409 de désignation, juste à cet instant ; s'il confirme, le 409 de version le fait recharger. Deux dialogues dans ce cas rare, plutôt qu'une lecture de version de plus. **Le refus de désignation ne consomme pas la version.**
- `DbError::AccountDesignatedInSettings { settings: Vec<&'static str> }`, **409 `ACCOUNT_DESIGNATED_IN_SETTINGS`**, `details: { settings }`, message traduit.
- `ArchiveAccountRequest.confirm_designated` : `#[serde(default)]` (`false`). Un appelant de l'API qui l'ignore reçoit le 409 : **changement de comportement à dire** dans `docs/api-external.md`.
- **Audit** : `account.archived`, snapshot inchangé, plus `"designated_settings": […]` **quand il n'est pas vide** (les tests existants restent exacts).
- **Seconde face** : `validate_account_of` ne refuse l'inactif que si la valeur **change** (paramètre renommé `changed`, qui commande inactif et imputable). Le type reste vérifié (#537 n'est pas élargie, ni aggravée). Motif : le manuel et l'écran disent qu'un compte désigné peut être archivé ; il ne doit pas verrouiller les réglages.
- **Écran** : à réception du 409, le dialogue reste ouvert, liste les réglages par leurs libellés existants, explique l'effet, propose **« Archiver quand même »**. ⚠️ **Le réglage est réservé à l'Administrateur, l'archivage ouvert au Comptable** : le lien vers *Paramètres → Facturation* ne s'affiche qu'à l'Administrateur (`authState.currentUser?.role`, déjà lu à `accounts/+page.svelte:414`) ; le Comptable lit à sa place « Un administrateur peut changer ce réglage ».

## Acceptance Criteria

1. **AC-1** : pour chacun des dix réglages (table), `PUT …/archive` sans confirmation → 409 `ACCOUNT_DESIGNATED_IN_SETTINGS`, `details.settings` exact ; **le compte reste actif, `version` inchangée, aucun audit**. Rouge avant.
2. **AC-2** : compte désigné par deux réglages → les deux codes.
3. **AC-3** : avec `confirmDesignated: true` → 200, compte archivé, audit portant `designated_settings`. Le réglage **reste** tel quel. *(Annoté le 2026-11-07, revue P2, L7 : la clé s'écrit `designatedSettings` depuis la remédiation de la P1, comme le reste du JSON de l'audit ; vaut aussi pour la section d'en-tête et M6.)*
4. **AC-4** : compte non désigné → 200 sans confirmation, audit exactement inchangé. Sous-compte actif, désigné → refus existant d'abord. Version périmée, avec confirmation → 409 `OPTIMISTIC_LOCK_CONFLICT`. **Version périmée, sans confirmation, compte désigné → 409 `ACCOUNT_DESIGNATED_IN_SETTINGS`**, puis, sa confirmation envoyée à la même version, 409 `OPTIMISTIC_LOCK_CONFLICT` ; compte inchangé aux deux.
5. **AC-5** : **isolation** : un compte d'une autre société de même numéro, ou une désignation d'une autre société, ne déclenche rien (test à deux sociétés).
6. **AC-6 — inventaire** : la liste des dix codes égale les colonnes `default_%_account_id` de `company_invoice_settings`, recalculées en base.
7. **AC-7 — seconde face** : TVA due archivée par confirmation, puis `PUT /company/invoice-settings` qui ne change que le format de numérotation → 200. Passer **à** un compte archivé → toujours 400.
8. **AC-8 — écran** : les deux états (Vitest + `accounts.spec.ts`), libellés dans les quatre langues ; **Administrateur : le lien ; Comptable : aucun lien, le texte « Un administrateur… »**, et sa confirmation archive bien ; 409 de version toujours rendu par son message actuel.
9. **AC-9** : `reactivate` inchangé ; les 14 appels de `accounts::archive` passent sans modification.

## Tâches

- [x] **T1 — rouges** : `tests/accounts_designated_archive_e2e.rs`, AC-1 à 7.
- [x] **T2** : dépôt, `DbError`, `errors.rs`, route. ⚠️ `kesh-db` touché : gate complet en cours de boucle.
- [x] **T3** : `validate_account_of` ; grep des tests qui épinglent « compte archivé » sur `company_invoice_settings` : les ajuster en connaissance de cause, et le dire au Change Log.
- [x] **T4** : écran, types, API, Vitest, i18n quatre locales (`error-account-designated-in-settings`, `accounts-archive-designated-*`), `lint-i18n-ownership`.
- [x] **T5** : E2E, captures thème clair (dialogue normal, avertissement) dans `kesh-gate-logs/e335/captures/`.
- [x] **T6** : manuels, `docs/api-external.md`, CHANGELOG *Corrigé* (`refs #486`).

### Mutations nommées (P2)

- **M1** : ne lire que neuf colonnes (retirer `default_bad_debt`) → rouge AC-1, AC-6. **M2** : requête sans `company_id` → rouge AC-5.
- **M3** : `confirm_designated` ignoré → rouge AC-3. **M5** : `changed` remis à `true` pour l'inactif → rouge AC-7.
- **M6** : `designated_settings` toujours écrit → rouge AC-4 (audit exact).
- **M7** : désignation lue après l'`UPDATE` ou version contrôlée avant elle → rouge AC-4 (version périmée sans confirmation). **M8** : lien affiché à tous les rôles → rouge AC-8.

## Gates

Gate complet, E2E complet au dernier commit de code. Aucune migration : la colonne existe, la requête lit seulement.

## Manuels

`user-manual.tex`, § `sec:plan-comptable-archivage` (`:404-410`) : l'avertissement, les réglages, « Archiver quand même » et ce que cela implique ; le comportement de refus à l'écriture. Paramètres de facturation : qu'un compte archivé y reste affiché. `docs/api-external.md` : le 409 et `confirmDesignated`. PDF régénérés, contrôlés aplatis.

## Issues voisines : rapprochées, non intégrées

- **#280** (factures validées référençant le compte) : même dialogue, autre lecture coûteuse : reste ouverte ; le dialogue laisse la place d'une seconde ligne.
- **#286** (article au compte devenu inutilisable), **#45** (validation serveur à la création) : autres gestes, autres écrans. **#537** (retypage d'un compte désigné) : le helper `designations_of_account` lui servira ; pas ici : changer le contrôle du retypage déplace une règle comptable.
- **À tracer** : les autres désignateurs sans avertissement (`products.revenue_account_id`, `bank_accounts.journal_account_id`, `reconciliation_rules.counterparty_account_id`, `settlement_account_id`) : **issue demandée** s'il n'en existe pas (grep au T1).

## Recoupements

`company_invoice_settings.rs` (route et entité) avec **33-1a**, qui y pose le format du numéro interne : autres champs, mais **le test d'inventaire ne vise que `default_%_account_id`**, et le corps reste en remplacement intégral : à vérifier à l'intégration. `accounts/+page.svelte` : aucune autre fiche.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5), remédiation (Opus 5.5). Aucun finding au-dessus de LOW. **L-R8-1** : ordre décidé (C-e334-29, lu : `repositories/accounts.rs:681-710`), AC-4 élargi, M7. **L-R8-2** : lien réservé à l'Administrateur, AC-8, M8. Point « libellés de TVA à vérifier » levé, lu.
- 2026-11-06 : validation P2 (Opus 5.5), remédiation (Opus 5.5). Un LOW. **L-R8-1** : M4 retirée, doublon de M7 : le refus annule la transaction (patron lu : `repositories/accounts.rs:692-695`), donc l'AC-1 reste vert, et c'est l'AC-4 qui tue ce déplacement. Numéros des autres mutations conservés : sept mutations, M1 à M3, M5 à M8.

- 2026-11-07 : développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e335`, commits de code `32944fc0` et `d7dcb3c9`. **Étape 0** : la P3 ne rend aucun constat sur cette fiche ; aucun point « à établir ». **Grep de T1** (`gh issue list`, titres) : aucune issue ouverte sur les autres désignateurs sans avertissement, hors #286 pour l'article ; issue demandée. **Ajout hors fiche, C-e334-34** : lu, `settings/invoicing/+page.svelte` chargeait `fetchAccounts(false)`, si bien que `withCurrentAccount` ne résolvait pas un compte archivé : le champ s'affichait vide, et un geste l'effaçait (le défaut de #271). La fiche demandait qu'il « reste affiché » au manuel seulement. Deux gardes recomptées, que la fiche ne nommait pas : `i18n-keys.test.ts` (`sitesTotal` 2832 → 2849) et `i18n-libelle-en-dur.test.ts` (77 → 79, `conforme` 63 → 65) ; la seconde a rougi au gate de `32944fc0`, d'où `d7dcb3c9` et un rejeu de Vitest, `check` et l'E2E.

- 2026-11-07 : remédiation de la revue P1 (Opus 5.5), commit `0f2bba8b`. Passe 1 (trois lentilles) : 0 CRITICAL, 0 HIGH, des MEDIUM. Traités :
  - **A M-2, E-1, E-2, E-3, B M2** : le dialogue disait le même effet pour dix réglages, faux pour `vatDecompte`. Une phrase par réglage, quatre locales, `user-manual.tex`, chacune prouvée par un témoin avant l'archivage puis la variante d'erreur (`archived_*`). ⚠️ **Prémisse nuancée** : A et l'orchestrateur donnent `revenue` refusé « plus tôt » ; c'est vrai du compte **explicite** d'une ligne, non du **réglage** : lu `invoices.rs:645-648` (« les lignes sans compte ne sont PAS contrôlées ici ») et constaté par le test, le compte par défaut archivé n'est refusé qu'à la validation. La phrase de `user-manual.tex:412`, qui contredisait `:219` sur la TVA, prend l'escompte pour exemple.
  - **B M1 = A M-1** : `active_children_are_refused_first` sans confirmation, M9. **B M3** : le type, test et M10 ; différence écrite au doc-comment de `validate_account` et au manuel d'administration. **A L-2** : `details` illisible, plus d'enfermement (Vitest corrigé, il épinglait le défaut).
  - **LOW** : commentaires périmés (`company_invoice_settings.rs:229,232`, `idor_multi_tenant_e2e.rs:955`, `errors.rs:3423`, le bras déplacé) ; `designatedSettings` ; `role="alert"` ; promesse de la fiche au manuel d'administration (§ *Un compte désigné puis archivé*) ; A L-5, B L4 : la société se lit sur le compte, `archive_guarded` perd son paramètre ; A L-7 : l'inventaire par clé étrangère ; E-8 : confirmation explicite fausse, confirmation sur un compte non désigné, société sans réglages, mock remis à zéro hors du test ; L7 : capture Comptable.
  - **Tranchés** : la course (C-e334-37) et le réarchivage (C-e334-38).
  - **Réfuté** : B L6, « trois tests suivants tournent en Comptable » : lu `accounts-page.test.ts`, le `beforeEach` du fichier reconnecte l'Administrateur avant chaque test, et l'`afterEach` déconnecte.
  - **Écart de méthode** : aucun script sur le code hors des mutations.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/e335/rouges.log`) : rouge à la compilation (`DESIGNATED_ACCOUNT_COLUMNS` absente) ; puis, la constante seule posée, 12 tests, 5 passés, 7 rouges, dont `an_archived_designation_no_longer_locks_the_settings` (`400 Compte TVA due : compte archivé`), qui établit la seconde face. Les cinq verts d'avant le correctif décrivent ce qui ne doit pas changer (non désigné, sous-comptes, version avec confirmation, autre société) et l'inventaire ; M2, M6 et M1 montrent que trois d'entre eux mordent.
- **Mutations** (`e335/mut/synthese-33-8.log`), sept sur sept tuées. M1 : `each_of_the_ten_settings_stops_the_archive`, `two_settings_on_one_account_are_both_named`, `the_inventory_equals_the_real_columns`. M2 : `another_companys_designation_is_ignored`. M3 : les tests de confirmation. M5 : `an_archived_designation_no_longer_locks_the_settings`. M6 : `an_undesignated_account_archives_as_before`. M7 : `a_stale_version_without_confirmation_hits_the_designation_first`. M8 : le test Vitest du Comptable. M4 retirée à la validation.
- **Gates** (`e335/33-8/`). Au commit `32944fc0` : `test-fast.sh --ci` vert, 4825 exécutés, 4825 passés, 12 ignorés ; `cargo build --workspace --all-targets` vert ; `lint-i18n-ownership` vert ; `npm run build` vert ; Vitest rouge, 2 échecs de `i18n-libelle-en-dur.test.ts`. Au commit `d7dcb3c9`, qui ne touche que ce fichier de test : Vitest 2702 tests, 218 fichiers, vert ; `npm run check` 0 erreur ; **E2E complet** sur `kesh_e2e_e335` recréée : 378 tests, 353 passés, 18 ignorés, **7 échecs, tous de la KF-029** (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`), 16,2 min (`e2e-r33-8-2/`). `test-fast` et `build` n'ont pas été rejoués à `d7dcb3c9` : aucun fichier Rust n'y bouge.
- **Captures** (`e335/captures/33-8-*.png`) : dialogue normal, avertissement Administrateur, réglage archivé. Le Comptable est couvert en Vitest (AC-8), sans capture.
- **Manuels** : `user-manual.tex`, § `sec:plan-comptable-archivage` ; PDF régénéré, contrôlé aplati (« Archiver un compte désigné dans les paramètres », « Archiver quand même » lus). `docs/api-external.md` : le 409, `confirmDesignated`, l'ordre des refus. PDF de la brochure et du manuel d'administration, régénérés sans changement de source, rétablis.
- **AC-9** : les appels de `accounts::archive(` n'ont pas changé ; `archive` lit désormais le `company_id` du compte, puis délègue, confirmation posée. `reactivate` n'est pas touché.
- **Limite écrite** : la lecture de la désignation ne verrouille pas les réglages (doc-comment d'`archive_guarded`), comme `confirm_retype`. Autres désignateurs : issue demandée.

- **Remédiation P1, rouges** (`e335/rouges.log`) : Rust, 23 tests, 22 passés, 1 rouge (`designatedSettings`). Les preuves `archived_*` passaient avant le code, ce qu'elles doivent faire : elles établissent des refus existants, non un comportement neuf, et leur première écriture a corrigé trois prémisses (le `revenue`, le compte créanciers à désigner au montage, un numéro de compte en double). Vitest, 2 rouges sur 11 (`chaque réglage dit son propre effet`, `details illisible`).
- **Mutations P1** (`e335/mut/synthese-33-8-p2.log`), huit sur huit tuées : `M9-ordre`, `M10-type`, `M11-societe`, `M12-cle`, `V-enferme`, `V-effet`, `V-decompte`, `V-reglages`.
- **Gates à `0f2bba8b`** (`e335/33-8-p1/`) : `test-fast.sh --ci` 4836 passés, 12 ignorés ; build vert ; `check`, `lint-i18n-ownership`, `npm run build` verts ; Vitest 2703, 218 fichiers ; **E2E complet** 378 tests, 352 passés, 18 ignorés, 8 échecs : les 7 de la KF-029 et `sidebar-navigation.spec.ts:81` (KF-052, #424, liste de `docs/testing.md`).
- **Captures** : `e335/captures/33-8-p1-avertissement-comptable.png`, et celles du dialogue, de l'avertissement Administrateur et du réglage archivé, reprises.
- **Manuels** : `user-manual` et `admin-manual`, PDF contrôlés aplatis (« décompte TVA, lui », « Un compte désigné puis archivé » lus ; « non-assujetti peut vouloir » absent) ; brochure rétablie.

### File List

`crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/accounts.rs`, `crates/kesh-db/src/repositories/company_invoice_settings.rs`, `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/accounts.rs`, `crates/kesh-api/src/routes/company_invoice_settings.rs`, `crates/kesh-api/tests/accounts_designated_archive_e2e.rs` (neuf), `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`, `frontend/src/lib/features/accounts/accounts.types.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`, `frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/src/routes/(app)/accounts/accounts-page.test.ts`, `frontend/src/routes/(app)/settings/invoicing/+page.svelte`, `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`, `frontend/tests/e2e/accounts.spec.ts`, `docs/manual/fr/user-manual.{tex,pdf}`, `docs/api-external.md`, `CHANGELOG.md`, `_bmad-output/implementation-artifacts/33-8-compte-designe-archive-review-prompt-p1.md` (neuf).
