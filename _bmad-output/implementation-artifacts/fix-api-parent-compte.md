# Correctif : le parent d'un compte se change après sa création (`PUT /accounts/{id}` accepte `parentId`)

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/api-parent-compte`, partie de `main` à `70be12a6` (squash de l'Epic 37,
PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` et va à la prochaine release planifiée.
**Aucune migration** : la colonne `accounts.parent_id` et sa clé étrangère existent depuis l'origine ; seul le chemin
d'écriture manquait. Constaté par Beat à la recette d'un exercice réel 2024
(`data-not-committed/2024-qesh/controle/defauts-kesh.md`, § 7, « Le parent d'un compte ne se change pas — P4 »).*

## Intention

`PUT /api/v1/accounts/{id}` n'acceptait pas `parentId` (`UpdateAccountRequest` : `name`, `accountType`, `role`,
`postable`, `currency`, comptes de change, `version`) : le parent ne se posait qu'à la création. Les comptes du plan
livré par l'assistant, et ceux qui portent déjà des écritures, ne pouvaient donc pas être rangés après coup sous des
comptes de groupe reprenant la structure d'un ancien bilan.

**Prémisse vérifiée au code** (`70be12a6`) : `crates/kesh-api/src/routes/accounts.rs`, `UpdateAccountRequest` sans
`parent_id` (serde ignore le champ inconnu, le `PUT` rendait 200 sans rien changer) ; `kesh_db::entities::AccountUpdate`
sans parent ; l'`UPDATE` de `accounts::update` n'écrit pas `parent_id` ; commentaire de `accounts::create` :
« `parent_id` étant immuable après création (cf. `AccountUpdate`) ».

## Critères d'acceptation

1. **AC-1 — Trois états.** `PUT /accounts/{id}` accepte `parentId` : absent = inchangé (un client qui ne connaît pas le
   champ ne détache rien), `null` = racine, un identifiant = le nouveau parent. Renvoyer le parent en place est un no-op
   (ni version, ni audit).
2. **AC-2 — Contrôles de la création.** Parent introuvable ou d'une autre société → `400 VALIDATION_ERROR` « parent
   account not found » ; archivé → `400` « parent account is archived » (mêmes réponses que `POST /accounts`, par la
   même fonction) ; compte de différence de change (de la société ou propre) → `409 ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` ;
   compte de bilan en devise qui porte des écritures → `409 ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES` ; l'un des comptes
   propres résultants du compte → `400 FX_ACCOUNT_INVALID` `notPostable`. **Plus strict que la création** (revue
   P1-2, décision de Beat) : tout autre parent qui porte des écritures → `409 ACCOUNT_PARENT_HAS_ENTRIES`. La création n'exige aucune compatibilité de
   type entre parent et enfant : la modification non plus.
3. **AC-3 — Cycle.** Le compte lui-même, ou l'un de ses sous-comptes à tout niveau → `409 ACCOUNT_PARENT_CYCLE`
   (message traduit dans les quatre locales, qui nomme le parent demandé). Rien n'est écrit.
4. **AC-4 — Verrous.** Sous l'ordre du dépôt (`accounts::update`, « Verrous ») : le parent `FOR UPDATE` après le
   compte, puis ses ancêtres en lecture verrouillante, de sorte qu'un déplacement concurrent qui fermerait le cycle par
   un ancêtre est vu ; la route reste rejouée sur interblocage (`retry_app_on_deadlock("accounts::update")`).
5. **AC-5 — Effets.** Le nouveau parent devient non imputable (règle 14-3a D2, comme à la création, sans bump de
   version) ; l'ancien garde son état. Le compte déplacé garde type, postabilité, écritures et solde. Audit
   `account.updated`, `parentId` dans `before` et `after`.
6. **AC-6 — Écran.** *Modifier le compte* propose le champ **Compte parent (optionnel)**, comme la création, sans le
   compte ni ses sous-comptes, ni les comptes qui portent des écritures (parent actuel excepté) ; le champ ne part que
   s'il change ; un refus s'affiche dans le dialogue. Un E2E Playwright le traverse (revue P1-3).
7. **AC-7 — Rien ne casse ailleurs** (vérifié au code) : rapports, export, sauvegarde, MCP.
8. **AC-8 — Documentation.** `docs/api-external.md` (changement additif, code d'erreur au tableau),
   `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (ordre des verrous), manuel utilisateur FR (`.tex` seulement).

## Ce que le changement de parent touche ailleurs (AC-7)

- **Rapports** (`kesh-report`, routes de rapports) : aucun ne lit `accounts.parent_id` (`grep`) — le bilan et le
  compte de résultat regroupent par type et numéro. Inchangés.
- **Écran du plan comptable** : l'indentation se calcule depuis `parentId` à chaque chargement ; l'ordre reste celui
  des numéros (comme avant pour un enfant créé sous un parent de numéro sans rapport).
- **Export global** (`accounts.csv`, colonne `parent_id`) et **sauvegarde** (`backup.rs`, `FOREIGN_KEY_CHECKS = 0`
  pour l'auto-référence) : ils lisent la valeur courante ; aucun ordre de restauration ne dépend de la hiérarchie.
- **MCP `kesh_plan_comptable`** : rend `parent_id` tel quel, en lecture.
- **Réévaluation de fin d'exercice (Story 37-5)** : sa garde « un compte en devise mouvementé ne reçoit pas de
  sous-compte » reposait sur l'absence de reparentage (fiche 37-5, revue P4) ; elle est rejouée au changement de
  parent (`refuse_foreign_parent_with_entries_in_tx`).
- **Le parent mouvementé** (revue P1-2) : refusé au `PUT` ; la création l'accepte encore (écart tracé, A-1 de
  `kesh-gate-logs/issues-a-ouvrir-fixes.md`).
- **Postabilité** : `effective_postable` (compte à sous-comptes actifs non imputable) est inchangé ; un ancien parent
  sans plus de sous-compte actif redevient imputable par un `PUT` (comme après l'archivage de son dernier enfant).

## Tâches

- [x] `ParentChange` (trois états, `resolve`) et champ `AccountUpdate::parent` (`kesh-db`, entités).
- [x] `accounts::update` : no-op, `guard_parent_change_in_tx` (contrôles, cycle, verrous), `parent_id` écrit, nouveau
  parent non imputable ; doc-comments.
- [x] `DbError::AccountParentCycle` (409 `ACCOUNT_PARENT_CYCLE`) et `AccountParentNotFound` (400, message de la
  création) ; clé `error-account-parent-cycle` dans les quatre locales.
- [x] Route : `parentId` à trois états ; `check_parent_in_company`, partagé avec la création (DRY).
- [x] Écran : champ parent du dialogue de modification ; `parentCandidates`, `parentFieldForUpdate` (et options de la
  création par `parentCandidates`).
- [x] Tests : dépôt, API, Vitest ; mutations.
- [x] Docs : `api-external.md`, `MULTI-TENANT-SCOPING-PATTERNS.md`, manuel FR.

## Notes de développement

- **Deux variantes d'erreur, non une.** Le cycle a son code (`ACCOUNT_PARENT_CYCLE`), parce qu'aucun code existant
  ne dit à l'utilisateur quoi corriger. Le parent introuvable relu **sous verrou** dans la transaction rend la réponse
  exacte de la création (`400 VALIDATION_ERROR`, « parent account not found ») par `AccountParentNotFound` : la route
  l'a déjà refusé avant, la variante ne sert qu'au dépôt appelé seul. Un parent archivé **entre** la route et la
  transaction rend `409 ACCOUNT_PARENT_ARCHIVED` (variante existante, message traduit) : cas de course seulement.
- **Ancien parent.** Pas de retour automatique à « imputable » : ni l'archivage du dernier enfant ni aucun autre
  chemin ne le fait, et rendre imputable un compte de groupe sans geste de l'utilisateur ouvrirait la saisie sur un
  compte que l'utilisateur a pu fermer lui-même.
- **Aucune compatibilité de type exigée** : la création n'en exige pas (`create_account` ne contrôle que l'existence,
  la société et l'état actif) ; la consigne disait « si la création l'exige ».
- **Ordre des verrous.** Le compte `FOR UPDATE`, puis le parent `FOR UPDATE`, puis les ancêtres en partage : deux
  lignes de `accounts` sont prises dans l'ordre du geste, non par identifiant. Deux déplacements croisés (A sous B,
  B sous A) peuvent s'interbloquer ; InnoDB en annule un, la route le rejoue, et le rejeu voit le cycle (raisonné, non
  provoqué par un test). La course par un ancêtre est, elle, testée.
- **Écran.** Le champ ne part que s'il change (`parentFieldForUpdate`) : un écran ouvert avant un déplacement fait par
  un autre ne remet pas l'ancien parent en renvoyant le formulaire (et le contrôle de version refuserait de toute
  façon).

## Change Log

- 2026-11-18 — Correctif (`36d2706a`) et tests (`cc1f2d71`), base `70be12a6`.
- 2026-11-18 — Gate ciblé au commit `cc1f2d71` (`binary(account_parent_change)` et les tests `put_*` d'`accounts_e2e`) :
  7 passés. Mutations (`kesh-gate-logs/fix-api-parent-compte/mutations.log`) : **M1** `parent_id` non écrit → trois
  tests rouges (`ranger_puis_detacher_un_compte_mouvemente`, `un_compte_ne_se_range_pas_sous_lui_meme`,
  `put_range_un_compte_mouvemente_sous_un_groupe`) ; **M2** remontée des ancêtres en lecture simple → seul
  `course_un_cycle_ferme_par_un_ancetre_concurrent_est_vu` rouge. **M3** (cycle non refusé), **M4** (nouveau parent
  laissé imputable), **M5** (contrôles de change retirés) : **non concluantes** — les sept tests y échouent tous en
  30 s à l'ouverture de leur base (`sqlx-core testing/mod.rs:226`) : le conteneur `kesh-mariadb-dev2` s'est arrêté en
  cours de campagne (constaté : `container … is not running`), et le témoin sans mutation échoue de même. À rejouer.
- 2026-11-18 — Mutations M3 à M5 rejouées après le redémarrage du conteneur (`mutations-2.log`, sous `heavy.sh` et
  `mem-guard`) : témoin sans mutation 7/7 vert ; **M3** (cycle non refusé) → `un_compte_ne_se_range_pas_sous_lui_meme`,
  `course_un_cycle_ferme_par_un_ancetre_concurrent_est_vu`, `put_parent_refuse_ce_que_la_creation_refuse_et_le_cycle`
  rouges ; **M4** (nouveau parent laissé imputable) → `ranger_puis_detacher_un_compte_mouvemente`,
  `put_range_un_compte_mouvemente_sous_un_groupe` rouges ; **M5** (contrôles de change retirés) →
  `parent_de_change_ou_en_devise_mouvemente_refuse` rouge.
- 2026-11-18 — Écran (`f01e853d`) : champ parent du dialogue de modification, `parentCandidates`,
  `parentFieldForUpdate` ; commentaire d'`accounts::create` (« `parent_id` immuable ») corrigé ; `sitesTotal` de la
  garde i18n relevé par la garde, 3388 → 3392 (`front-test-unit-1-rouge-sitesTotal.log`). Dernier commit de code.

### Revue de code P1 (Opus 5.5, trois lentilles en un rapport, `kesh-gate-logs/fix-api-parent-compte/revue-p1.md`)

- Trend : **P1 : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW.** Chaque prémisse vérifiée au code avant remédiation.
- **P1-1** (MEDIUM, vérifié : `grep` — la doc d'`update` coiffait `guard_parent_change_in_tx`, `update` sans doc) :
  le garde déplacé avant la doc d'`update`.
- **P1-2** (MEDIUM, décision de Beat) : refus `409 ACCOUNT_PARENT_HAS_ENTRIES` d'un parent qui porte des écritures
  (`DbError::AccountParentHasEntries`, `has_journal_line_locked` sous le `FOR UPDATE` du parent, après le refus propre
  à la devise) ; clé `error-account-parent-has-entries` (quatre locales) ; `api-external.md`, manuel ; écran :
  `parentCandidates` écarte en modification les comptes `currencyLocked` (vérifié au code : `currency_locked_account_ids`
  = au moins une ligne d'écriture, sans condition de devise), sauf le parent actuel ; le refus s'affiche dans le
  dialogue (`editError = err.message`). Création inchangée : issue A-1.
- **P1-3** (MEDIUM) : cas Playwright « ranger un compte sous un groupe, puis le remettre à la racine »
  (`accounts.spec.ts`).
- **P1-4** (LOW, vérifié : `fx_designations_of_in_tx` lit les désignations en base, compte lui-même compris) :
  `fx_designations_excluding_in_tx` écarte celles du compte déplacé, jugées sur l'état après le `PUT` (étape 5) ;
  `fx_designations_of_in_tx` en devient l'enveloppe (`exclude_holder = 0`). Test
  `sous_son_ancien_compte_propre_retire_dans_le_meme_put` (accepté seul détenteur ; refusé si d'autres désignent).
- **P1-5, P1-8** (LOW) : issues A-2, A-3 à ouvrir. **P1-6** (LOW) : manuel aligné sur le code. **P1-7** (LOW) :
  commentaire replié.
- Commits : `4241e6b7` (code et tests), `2be85cf4` (docs), `3e1bfbb0` (sélecteur E2E « Aucun » par `data-testid`,
  relevé par la garde KF-043 `e2e-selecteurs-traduits.test.ts` — **dernier commit de code**).
- Gate ciblé de la remédiation (sous `heavy.sh` et `mem-guard`) : fmt et clippy verts ; nextest ciblé, mêmes 31
  binaires qu'au gate précédent, au commit `4241e6b7` (le suivant ne touche pas le Rust) : **2027 passés, 6 ignorés**
  (`cible-nextest.log`) ; frontend à `3e1bfbb0` : `check` 0 erreur, `lint-i18n-ownership`, `test:unit` **4359 tests
  (271 fichiers)**, `build` ; E2E ciblé `accounts.spec.ts` + `smoke.spec.ts` : **14 passés**, dont le cas neuf
  (`e2e-cible.log`). Premier passage Vitest rouge sur la garde KF-043 (« Aucun » visé par libellé :
  `front-test-unit-r1.log`), corrigé par `3e1bfbb0`. Captures non reprises : le dialogue ne change que par le filtre
  des candidats et un `data-testid`. **Gate complet groupé à venir.**
- La remédiation touche la production : passe P2 ciblée requise (`revue-p2-ciblee-prompt.md`).
- 2026-11-18 — passe P2 ciblée (Sonnet, lecture seule) sur `4241e6b7`..`3e1bfbb0` : **0 au-dessus de LOW**, 4 LOW. P2-4 (phrase soudée de `docs/api-external.md`) corrigée par l'orchestrateur ; P2-1 (formulation du refus quand le parent est déjà non imputable), P2-2 (comptes E2E non nettoyés), P2-3 (nom de variable d'un test) tracés en issues. **Boucle close.** Trend : P1 Opus 3 MEDIUM + 5 LOW → P2 ciblée Sonnet 0 > LOW. Gate complet groupé à venir, avant tout push.
- 2026-11-18 — **Reprise de `main` après la v0.19.0, puis gate complet groupé.** `origin/main` (`773df80d`, tag
  `v0.19.0`) repris par fusion (`f47532f4`), sans rebase ; conflits résolus en gardant les deux intentions (`sprint-status.yaml` : les deux blocs ; `docs/api-external.md` : le paragraphe du correctif et celui de `groupSettlementMaxDifference` gardés, les lignes `ACCOUNT_PARENT_*` ajoutées à celles de `main`, « Depuis la v0.19.0 » de `main` ; la mention « release qui suit la v0.18.0 » du correctif, devenue fausse, portée à « la v0.19.0 » ; `i18n-keys.test.ts` : `sitesTotal` **3499**, relevé par la garde, non additionné — 3495 de la 0.19.0 + 4 du correctif).
  Gate ciblé après la reprise : `fmt`, `clippy` du workspace, nextest des bibliothèques `kesh-db` et `kesh-api` et de neuf binaires (`account_currency_repository`, `account_parent_change`, `credit_card_repository`, `invoice_write_off`, `invoices_validate_vat`, `letterings`, `supplier_invoices_repository`, `accounts_e2e`, `journal_entry_reversal_e2e`) **1539 passés, 5 ignorés**, Vitest `src/lib/features/accounts` et garde i18n **76 tests (6 fichiers)** (`kesh-gate-logs/fix-api-parent-compte/reprise-v0190-*.log`). Puis `origin/main` `37c2f875` (documentation seule : `CLAUDE.md`, bilan de la
  0.19.0, `sprint-status.yaml`) repris de même. **Gate complet groupé** sur la branche jetable `integ/fixes-0.19.0`
  (`773df80d` + les quatre correctifs fusionnés dans l'ordre PDF, monnaies retirées, résultat reporté, parent du
  compte ; seul conflit, `sprint-status.yaml`), **au commit `5fc6fb1b`**, base `kesh_integ` remise à zéro :
  `scripts/test-fast.sh --ci` **6165 passés, 14 ignorés** ; frontend `check` 0 erreur (29 avertissements
  préexistants), `lint-i18n-ownership`, `test:unit` **4627 tests (278 fichiers)**, `build` ; **E2E complet**
  (`LANG=fr_CH.UTF-8`, port 3382) **478 passés, 10 échecs, 18 ignorés** — les dix attendus, jugés un à un : sept KF-029
  (#97 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`), `backup-restore-new-install.spec.ts:262`, `:381`, `opening-balances.spec.ts:156` (`docs/testing.md`
  inchangé par la 0.19.0) ; `scripts/gate-ci.sh` (copie `git archive`) **6184 passés, 0 échoué, 18 ignorés**.
  Journaux : `kesh-gate-logs/integ-fixes-0.19.0/` (`gate-backend.log`, `front-*.log`, `e2e.log`, `gate-ci.log`) ; un
  premier essai, arrêté par le disque plein de la station, est gardé dans `avorte-disque-plein/`. Le commit de
  documentation `37c2f875` n'a pas été rejoué au gate (aucun code).

## Notes de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Fixed** — Plan comptable : le **compte parent** d'un compte se change après sa création, par *Modifier le compte*
  et par `PUT /api/v1/accounts/{id}` (`parentId` : absent = inchangé, `null` = racine) — les comptes du plan livré, ou
  qui portent déjà des écritures, se rangent sous des comptes de groupe. Mêmes contrôles qu'à la création, et un compte
  ne se range ni sous lui-même ni sous l'un de ses sous-comptes (`409 ACCOUNT_PARENT_CYCLE`), ni sous un compte qui
  porte des écritures (`409 ACCOUNT_PARENT_HAS_ENTRIES`). Changement de contrat additif.

## Dev Agent Record

### Agent Model Used

Opus 5.5.

### Gates

**Gate complet groupé (2026-11-18)** : vert au commit `5fc6fb1b` de `integ/fixes-0.19.0` — chiffres au Change Log,
journaux dans `kesh-gate-logs/integ-fixes-0.19.0/`.

Journaux dans `kesh-gate-logs/fix-api-parent-compte/`. **Gate ciblé vert ; gate complet groupé à venir** (décision de
Beat du 2026-11-18 : `scripts/test-fast.sh --ci`, l'E2E complet et `scripts/gate-ci.sh` se joueront une seule fois, sur
une branche d'intégration réunissant plusieurs correctifs). Au dernier commit de code `f01e853d` :

- `cargo fmt --check` et `cargo clippy --workspace --all-targets -D warnings` : verts (`gate-cible.console.log`,
  `cible-clippy.log`).
- `cargo nextest run --profile ci` ciblé, base `kesh_fix44` remise à zéro : **2025 passés, 6 ignorés, 31 binaires**
  (`cible-nextest.log`) — bibliothèques de `kesh-db` (tests d'`accounts.rs`) et de `kesh-api` (tests d'`errors.rs`),
  `kesh-i18n`, et les binaires qui appellent `accounts::update` ou `PUT /accounts/{id}`, ou gardent schéma de test,
  textes et registres : `account_parent_change`, `accounts_e2e`, `account_currency_e2e`,
  `account_currency_repository`, `accounts_designated_archive_e2e`, `bank_accounts_e2e`, `credit_card_e2e`,
  `credit_card_repository`, `designated_account_type_e2e`, `exchange_rates_api_e2e`, `idor_multi_tenant_e2e`,
  `invoice_write_off`, `invoices_validate_vat`, `jour_suisse_nuit_e2e`, `journal_entry_reversal_e2e`, `letterings`,
  `letterings_e2e`, `open_items_e2e`, `reconciliation_e2e`, `rejeu_interblocage_e2e`, `reports_e2e`,
  `supplier_invoice_draft_e2e`, `supplier_invoices_repository`, `test_schema_guard`, `textes_coherents`,
  `audit_label_registry`, `audit_route_registry`, `i18n_e2e`.
- Frontend : `check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership`, `test:unit` **4358 tests
  (271 fichiers)**, `build`.
- E2E ciblé (port 3380, base `kesh_e2e_fix44` neuve) : `accounts.spec.ts`, `multi-currency.spec.ts`, `smoke.spec.ts` —
  **17 passés** (`e2e-cible.log`).
- Captures (thème clair) : `captures/avant/01` (dialogue sans champ parent, frontend de `70be12a6`),
  `captures/apres/01` à `05` (champ, liste ouverte, parent choisi, 1000 indenté sous le groupe 10 devenu
  « Non postable », groupe 10 sans lui-même ni son sous-compte 1000 dans ses options).

### File List

- `crates/kesh-db/src/entities/account.rs`, `crates/kesh-db/src/entities/mod.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/accounts.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/accounts.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `crates/kesh-db/tests/account_parent_change.rs` (neuf)
- `crates/kesh-api/tests/accounts_e2e.rs`
- `crates/kesh-db/tests/{account_currency_repository,credit_card_repository,invoice_write_off,invoices_validate_vat,letterings,supplier_invoices_repository}.rs`,
  `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` (champ `parent` des littéraux `AccountUpdate`)
- `frontend/src/lib/features/accounts/account-options.ts`, `account-options.test.ts`, `accounts.types.ts`
- `frontend/src/routes/(app)/accounts/+page.svelte`
- `docs/api-external.md`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/fix-api-parent-compte.md`, `sprint-status.yaml`
