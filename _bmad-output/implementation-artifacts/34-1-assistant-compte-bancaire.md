# Story 34-1 : l'assistant de démarrage pose la description et le compte comptable du compte bancaire (#966)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e341.md`,
C-e341-1 à C-e341-5, C-e341-33, C-e341-34. Commits en `refs #966` ; à la PR : `closes #966`.*

## Story

En tant qu'Administrateur qui installe Kesh, je veux saisir dès l'assistant la description du compte bancaire et le compte
comptable auquel il est lié, pour ne pas découvrir à la première réconciliation que le compte n'est pas configuré
(`BANK_ACCOUNT_NOT_CONFIGURED`), et savoir que ces choix se modifient après coup.

## Décision de Guy (fichier d'epic, #966, 2026-11-10)

Le compte bancaire peut être lié, **de manière facultative**, à un compte du plan, **présélectionné sur 1020 s'il existe**. La
page et le manuel disent que ce choix est **modifiable après coup**. ⛔ **Aucun rôle ne se déduit du numéro du compte**
(Story 14-3a) : 1020 n'est qu'une présélection d'écran, le serveur n'en sait rien et ne l'écrit nulle part.

## Ce que le code dit (prémisses établies)

- **Le plan est chargé avant l'étape bancaire** : `routes/onboarding.rs:432-490` (`accounting_language_in_tx`, étape 4→5) charge le
  plan (`accounts::bulk_create_from_chart_in_tx`, `:463`) ; l'étape bancaire est l'étape 6→7 (`set_bank_account`, `:589-648`). La question de #966
  (« vérifier que le plan est choisi avant ») est donc réglée : il l'est, et `GET /api/v1/accounts` est servi à tout rôle authentifié
  (`lib.rs:1012`), assistant compris (le 423 ne vaut que sans utilisateur, `middleware/auth.rs:106-108`).
- **Ce que l'assistant pose aujourd'hui** : `BankAccountRequest` (`onboarding.rs:577-581`) n'a que `bank_name`, `iban`, `qr_iban` ;
  `upsert_primary_in_tx` (`repositories/bank_accounts.rs:328-447`) n'écrit ni `description` ni `journal_account_id` (l'`INSERT` de `:382`
  et l'`UPDATE` de `:352` ne les nomment pas), et son court-circuit no-op (`is_no_op_change`, `:252-256`) ne compare que banque,
  IBAN, QR-IBAN. Le compte de l'assistant naît donc **toujours** sans lien : `journal_account_id = NULL`.
- **L'audit l'annonce déjà** : `audit_bank_account_upsert` (`onboarding.rs:652-715`) trace `journal_account_id` à la création et dans
  les instantanés `before`/`after` ; elle ne trace pas la description.
- **`NewBankAccount`** (`entities/bank_account.rs:78-85`) n'a ni description ni lien ; il est construit **68 fois** dans 33 fichiers
  (`git grep -n "NewBankAccount {" -- crates | wc -l`), dont une trentaine de tests. L'élargir ferait toucher tous ces sites (C-e341-1).
- **Les contrôles existent côté création** (`routes/bank_accounts.rs`) : `normalize_description` (`:282-301`, borne
  `BANK_ACCOUNT_DESCRIPTION_MAX_CHARS` = 60, `entities/bank_account.rs:93`), `validate_journal_account_id` (`:353-401` : existe, actif,
  `Asset | Liability`, imputable à la création), et le refus du compte de créance (`refuse_if_ledger_is_claim_account`,
  `repositories/bank_accounts.rs:1587-1602`). Les deux premiers sont des `fn` privées du module de route.
- ⚠️ **Le compte de créance n'est pas encore désigné à l'étape 6** : les réglages de facturation naissent à la finalisation
  (`finalize_inner`, `onboarding.rs:853-884`, `insert_with_defaults_in_tx`), qui y pose les comptes de **rôle** `Receivable` et
  `Payable` (`company_invoice_settings.rs:1132-1163`, `singleton_role`, trois lectures `FOR UPDATE` : `Receivable`, `DefaultRevenue`, `Payable`). À l'étape 6, `claim_accounts_in_share_mode`
  (`company_invoice_settings.rs:230-247`) rend donc `ClaimAccounts::default()` (aucune ligne), et un lien vers le compte de rôle
  `Receivable` passerait — puis la finalisation le désignerait compte de créance : un compte bancaire lié au compte débiteurs,
  l'état que la Story 15-6c interdit. La finalisation n'en contrôle rien (`INSERT IGNORE`). **C'est un trou que cette story ouvrirait** :
  AC-4 le ferme (C-e341-3).
- **L'écran** : `frontend/src/routes/onboarding/+page.svelte:146-149` (état), `:189-212` (`submitBankAccount`), `:392-430` (formulaire :
  banque, IBAN, QR-IBAN) ; l'appel `setBankAccount(bankName, iban, qrIban)` (`lib/features/onboarding/onboarding.api.ts:63-73`,
  `onboarding.svelte.ts:134-140`).
- **Le filtre des comptes liables** existe à l'écran *Comptes bancaires* (`routes/(app)/bank-accounts/+page.svelte:157-168`) :
  `active && postable && (Asset | Liability)`, moins les comptes de créance désignés. Le type rend `role` (`accounts.types.ts:53-54`).
- **1020 existe dans les trois plans livrés** (`kesh-core/assets/charts/{pme,association,independant}.json`, ligne 6). La présélection
  se lit sur le **numéro** dans la liste déjà filtrée, côté écran seulement.
- **Le manuel ment après cette story** : `user-manual.tex:214` (« nom de la banque, IBAN et QR-IBAN … les autres champs d'un compte, dont
  sa description, se règlent ensuite ») et `:2371` (« La description ne se saisit pas à l'onboarding »).
- **E2E** : `onboarding-path-b.spec.ts` est rouge sur l'étape bancaire depuis avril (KF-029, #97 : `:65`, `:92`) ; on ne s'y appuie pas.
  `onboarding-motif.spec.ts:22-48` amène l'état à une étape **par l'API** puis joue l'écran : patron à suivre.

## Acceptance Criteria

1. **AC-1 — le contrat** : `POST /api/v1/onboarding/bank-account` accepte `description` et `journalAccountId`, chacun facultatif.
   **Absent = inchangé, `null` = effacé** (C-e341-34) : un corps de la 0.17.5 (champs absents) crée le compte sans description ni lien, et, sur un
   compte principal existant, **laisse** description et lien tels qu'ils sont — l'`UPDATE` d'aujourd'hui ne les touche pas. L'écran envoie toujours
   les deux champs (`null` pour « Aucun »).
2. **AC-2 — mêmes contrôles qu'à la création** : description normalisée par **la même** fonction que `POST /bank-accounts`
   (`trim`, vide → absente, 60 caractères, caractères cachés refusés, même message) ; compte lié contrôlé par **la même** fonction
   (`404` inexistant ou inactif, `400 INVALID_ACCOUNT_TYPE` hors `Asset | Liability`, `400 ACCOUNT_NOT_POSTABLE`). Aucune copie :
   les deux fonctions passent `pub(crate)` et sont appelées (garde DRY : `git grep -n "fn normalize_description\|fn validate_journal_account_id" -- crates` rend une ligne chacune).
3. **AC-3 — ce qui est écrit** : le compte créé porte la description et le lien saisis ; sans saisie, `NULL`. Un rejeu identique
   (même banque, IBAN, QR-IBAN, description, lien) est un no-op (`Unchanged`, aucune trace) ; un rejeu qui ne change que le lien ou la
   description est un `Updated`. **Test du rejeu sans les champs** : compte principal existant portant une description et un lien, corps
   0.17.5 (champs absents, même banque et IBAN) → `Unchanged`, description et lien intacts ; même corps avec `"journalAccountId": null` →
   `Updated`, lien effacé.
4. **AC-4 — pas de lien vers un futur compte de créance** : un `journalAccountId` qui porte le rôle `Receivable` ou `Payable` est refusé
   en `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, le même code et le même rendu qu'à `POST /bank-accounts`, avant toute écriture, l'étape
   restant à 6. Les rôles sont lus dans la transaction, comme la finalisation les lira (`singleton_role`). **Le test prouve qu'on lit le rôle,
   non le numéro** (14-3a) : un compte de rôle `Receivable` **renuméroté** (pas 1100) est refusé ; un compte numéroté 1100 **sans** rôle passe.
   **Ordre des refus** : les contrôles de forme et `validate_journal_account_id` lisent sur le **pool**, avant la transaction (comme à la création) :
   un 400 de forme précède le 409 d'étape ; le refus du compte de créance vient après `lock_state_at_step` et `lock_company`.
5. **AC-5 — l'audit** : `bank_account.created` porte `description` et `journal_account_id` saisis (forme de `routes/bank_accounts.rs:666-673`) ;
   les instantanés de `bank_account.updated` portent `description` en plus. Jamais l'IBAN en clair.
6. **AC-6 — l'écran** : sous le QR-IBAN, deux champs : *Description (optionnelle)* (même limite, même aide que la page *Comptes bancaires*)
   et *Compte comptable lié* : liste des comptes actifs, imputables, `Asset | Liability`, **sans** les comptes de rôle `Receivable` et
   `Payable` ; une option « Aucun ». **Présélection** : le compte numéroté `1020` s'il figure dans cette liste, « Aucun » sinon. Si la liste
   ne se charge pas, le champ vaut « Aucun » et l'étape reste utilisable.
7. **AC-7 — « modifiable plus tard »** : sous les champs, une phrase dit que la description et le compte lié se modifient ensuite dans
   *Administration → Comptes bancaires* — quatre locales, repli français dans l'écran.
8. **AC-8 — refus lisibles** : un refus du serveur (description trop longue, compte non imputable, compte de créance) s'affiche par
   `errorMessageOf`, motif du serveur compris, comme les refus actuels de l'étape (`+page.svelte:200-204`).
9. **AC-9 — manuel** : `user-manual.tex:214` et `:2371` récrits (la description et le compte lié se saisissent à l'assistant, 1020 proposé
   s'il existe, modifiables ensuite) ; PDF régénéré et contrôlé aplati.

## Tâches

- [x] **T1 — rouges d'abord** (journal `kesh-gate-logs/e341/rouges.log`) :
  - `crates/kesh-api/tests/onboarding_path_b_e2e.rs` (ou fichier neuf `onboarding_bank_account_e2e.rs`) : AC-1 (corps 0.17.5 inchangé),
    AC-3 (description et lien écrits ; no-op ; `Updated`), AC-4 (rôle `Receivable`, rôle `Payable`, étape restée à 6, aucune ligne
    `bank_accounts`), AC-2 (61 caractères, compte `Revenue`, compte non imputable), AC-5 (détails de l'audit).
  - Vitest de l'écran (fichier neuf `frontend/src/routes/onboarding/onboarding-bank-step.test.ts`, patron de
    `routes/(app)/bank-accounts/bank-accounts-page.test.ts:31-33` pour le simulacre de `fetchAccounts`) : présélection 1020 ; aucun 1020 →
    « Aucun » ; 1020 non imputable → « Aucun » ; compte de rôle `Receivable` absent de la liste ; échec du chargement → « Aucun » et envoi
    possible ; le corps envoyé porte `description` et `journalAccountId` (ou `null`).
- [x] **T2 — dépôt** : `upsert_primary_in_tx` reçoit la description et le lien sans élargir `NewBankAccount` (C-e341-1) — forme
  retenue : un second paramètre `PrimaryDetails { description: Option<Option<String>>, journal_account_id: Option<Option<i64>> }` — extérieur
  `None` = absent, donc **inchangé** (C-e341-34) —, l'enveloppe `upsert_primary` passant `PrimaryDetails::default()` (ses appelants de test restent
  inchangés, et ne touchent plus ces colonnes) ; l'`INSERT` écrit la valeur ou `NULL` ; l'`UPDATE` ne nomme une colonne que si elle est présente ;
  `is_no_op_change` ne compare que les champs présents. La route lit la double option par `#[serde(default, deserialize_with = …)]` (forme à
  choisir au dev, un test de désérialisation pour absent, `null`, valeur). `BankAccountPutFields` (`repositories/bank_accounts.rs:560-575`) n'est
  **pas** réemployé : il porte la sémantique du `PUT` (remplacement complet, `None` = effacé), contraire à celle-ci (absent = inchangé). Le
  doc-comment de `BankAccount::description` (`entities/bank_account.rs:34-36`, « écrite **seulement** par la création de la route et par le `PUT` »)
  est mis à jour : l'assistant l'écrit aussi. ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [x] **T3 — route** : `BankAccountRequest` + `#[serde(default)]` ; `normalize_description` et `validate_journal_account_id` en
  `pub(crate)` dans `routes/bank_accounts.rs`, appelées par `set_bank_account` avant la transaction (même ordre de refus qu'à la création :
  banque, description, IBAN, QR-IBAN, compte lié) ; dans la transaction, après `lock_company`, si un lien est demandé : comptes de rôle
  `Receivable`/`Payable` lus par **une** fonction du dépôt, `accounts::find_singleton_role_in_tx(tx, company_id, role, lock)`, que
  `insert_with_defaults_in_tx` réemploie pour ses **trois** lectures (`lock = ForUpdate`, inchangé) et que l'assistant appelle deux fois
  **sans verrou** (`lock = None`) : l'étape est sérialisée par `lock_state_at_step` (`FOR UPDATE` sur `onboarding_state`), aucune finalisation ne
  peut courir en même temps, et la finalisation reverrouille ces lignes ; puis `refuse_if_ledger_is_claim_account`. Audit (AC-5).
- [x] **T4 — écran** : `onboarding.api.ts` et `onboarding.svelte.ts` (signature à objet `{ bankName, iban, qrIban, description, journalAccountId }`) ;
  `+page.svelte` : chargement des comptes à l'entrée de l'étape 6, filtre (AC-6), présélection, deux champs, la phrase AC-7. Le filtre
  s'écrit **une fois** : extraire celui de `bank-accounts/+page.svelte:157-168` en fonction de `lib/features/bank-accounts/format.ts`
  (`linkableLedgerAccounts(accounts, excludedIds, options)`, `options.liabilityOnly` servant la 34-11a2, C-e341-33), appelée par les deux écrans ;
  l'assistant passe les identifiants des comptes de rôle `Receivable`/`Payable`.
- [x] **T5 — i18n** : `onboarding-bank-editable-later` aux quatre locales ; libellés réutilisés (`bank-accounts-labels-description`,
  `bank-accounts-help-description`, `bank-accounts-labels-journal-account-id`, `bank-accounts-labels-not-configured`). `npm run lint-i18n-ownership`.
- [x] **T6 — E2E** : spec neuve `frontend/tests/e2e/onboarding-bank-account.spec.ts`, état amené à l'étape 6 **par l'API** (patron
  `onboarding-motif.spec.ts`), écran joué par sélecteurs d'`id` : 1020 présélectionné, description saisie, *Enregistrer* ; puis
  `GET /api/v1/bank-accounts` rend la description et le `journalAccountId` du 1020. La valeur doit **traverser la frontière HTTP**.
- [x] **T7 — manuel** (AC-9), capture de l'étape (thème clair, base jetable) dans `kesh-gate-logs/e341/captures/34-1-*.png`.
- [x] **T8 — CHANGELOG** : `## [0.17.6] — Non publié`, *Ajouté*, `refs #966`.

### Mutations nommées

- **M1** : `is_no_op_change` sans la comparaison du lien → rouge AC-3 (`Updated` attendu).
- **M2** : l'`INSERT` d'`upsert_primary_in_tx` sans `journal_account_id` → rouge AC-3 et E2E.
- **M3** : la lecture des rôles retirée (AC-4) → rouge sur `Receivable` et `Payable` ; **M3b** : rôle remplacé par le numéro `1100`/`2000` → rouge
  « compte renuméroté ».
- **M7** : l'`UPDATE` remet description et lien à `NULL` quand ils sont absents → rouge AC-3 (rejeu sans les champs).
- **M4** : présélection sur le premier compte de la liste au lieu de `1020` → rouge Vitest « aucun 1020 → Aucun ».
- **M5** : filtre sans `postable` → rouge Vitest « 1020 non imputable ».
- **M6** : `journalAccountId` omis du corps envoyé → rouge Vitest et E2E.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db` ; gate frontend complet ; E2E complet au dernier commit de code ; les échecs se nomment
contre `docs/testing.md` § « Les échecs attendus » (KF-029 comprise : `onboarding-path-b` n'est pas un signal pour cette story).

## Manuels

`user-manual.tex` (AC-9) ; aucun autre. **Prompt de revue : nommer le manuel** et exiger le contrôle du PDF aplati
(`pdftotext user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o "modifi[a-z]* [a-z ]*Comptes bancaires"`).

## Ce qui ne se traite pas

- ⚠️ **Angle mort, issue demandée** (validation P2, 1.2) : la garde de l'AC-4 ne vaut qu'à l'étape 6. Entre l'étape 6 et la finalisation,
  `PUT /api/v1/accounts/{id}` (`lib.rs:525-528`, sans garde d'onboarding) peut poser le rôle `Receivable` ou `Payable` sur le compte qui vient
  d'être lié ; la finalisation le désigne alors compte de créance (`INSERT IGNORE`, `company_invoice_settings.rs:1178-1185`) sans contrôle. Chemin par
  l'API seulement, l'écran n'y mène pas. Issue : « Finalisation de l'assistant : refuser de désigner compte de créance un compte lié à un compte
  bancaire ».

- **Le solde d'ouverture à l'assistant** : il n'y a encore aucun exercice à l'étape 6 (il naît à la finalisation, `onboarding.rs:930`,
  `create_if_absent_in_tx`) ; la 34-2 le pose au formulaire *Comptes bancaires*. Rien à tracer : la 34-2 le dit au manuel.
- **Le chemin A (démonstration)** : le seed crée ses comptes lui-même ; inchangé.
- **#44 (KF-002-CR-001)**, comptes 1100/3000 manquants à la finalisation : sans rapport, inchangé.

## Recoupements

`routes/(app)/bank-accounts/+page.svelte` et `lib/features/bank-accounts/format.ts` avec **34-2** (même formulaire, après cette story) et
**34-11a1** (IBAN facultatif, tolérance) et **34-11a2** (type carte, filtre « passif seulement ») ; `routes/onboarding.rs` et `upsert_primary_in_tx`
avec **34-11a1** (l'IBAN y devient-il facultatif ? non : l'assistant ne crée qu'un compte bancaire, C-e341-19) et **34-11a2** (la garde de Luhn
atteint l'assistant). Ordre : 34-1, puis 34-2, puis 34-11a1, 34-11a2.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet)** — pour cette fiche : 1 M (1.1), 4 L (1.2 à 1.5). Remédiation (Opus 5.5), tout appliqué. **1.1** : la
  sémantique d'absence est tranchée « absent = inchangé, `null` = effacé » (C-e341-34, relu `repositories/bank_accounts.rs:351-357`) : AC-1, AC-3
  (test du rejeu sans les champs), T2, M7. **1.2** : numéros de ligne corrigés (relus : `onboarding.rs:432`, `:463`, `:577`, `:589`). **1.3** : le
  renvoi va à C-e341-19. **1.4** : trois lectures de rôle et non deux, `FOR UPDATE` (relu `company_invoice_settings.rs:1132-1163`) ; fonction
  unique à verrou paramétré, sans verrou à l'assistant (motif écrit en T3) ; ordre des refus écrit à l'AC-4. **1.5** : test du compte renuméroté
  et du 1100 sans rôle (AC-4, M3b) ; la garde de Luhn à l'assistant relève de la 34-11a2. Axe 8 : `linkableLedgerAccounts` reçoit
  `liabilityOnly` (C-e341-33). Références à « 34-11a » portées à 34-11a1 / 34-11a2.
- 2026-11-10 : **validation P2 (Opus)** — pour cette fiche : 3 L (1.1 à 1.3). Remédiation (Opus 5.5), tout appliqué. **1.1** : pourquoi
  `BankAccountPutFields` n'est pas réemployé, et le doc-comment de l'entité à mettre à jour (relus `repositories/bank_accounts.rs:560-575`,
  `entities/bank_account.rs:34-36`). **1.2** : angle mort écrit, issue demandée (relu `lib.rs:525-528`). **1.3** : en-tête complété (C-e341-33, 34).

- 2026-11-10 : **développement** (Opus 5.5, clé `e3401`, worktree `/home/gcorbaz/devel/kesh-e3401`, base `219bd553`). Étape 0 :
  les LOW des validations P1 et P2 sur cette fiche étaient tous appliqués (la P3 n'a rien relevé sur elle) ; prémisses relues au
  code : `onboarding.rs:577-581` (`BankAccountRequest` à trois champs), `repositories/bank_accounts.rs:252-256` (`is_no_op_change`
  à trois champs), `:382` (`INSERT` sans description ni lien), `company_invoice_settings.rs:1132-1163` (trois lectures
  `singleton_role … FOR UPDATE`). **Écarts à la fiche** : (1) le refus d'étape est un **400** `ONBOARDING_STEP_ALREADY_COMPLETED`
  (`errors.rs:1804-1806`), non le « 409 d'étape » de l'AC-4 — le test distingue les refus par leur code ; (2) l'option « Aucun » de
  l'AC-6 est l'option « Non configuré » de la page *Comptes bancaires*, clé réutilisée (T5, C-e3401-3) ; (3) une liste de comptes
  en échec affiche son motif sous le champ (garde `onboarding-catch-guard.test.ts`, C-e3401-2), avec une clé neuve
  `onboarding-bank-ledger-load-error` ; (4) le formulaire *Lier* passe lui aussi par `linkableLedgerAccounts` et perd son filtre
  sur le numéro (C-e3401-1) ; (5) M7 n'est exercée que par un corps 0.17.5 qui **change** la banque (le cas « même banque »
  s'arrête au no-op) : test ajouté, `bank_step_update_without_new_fields_keeps_them`. Choix : `e34-choix-autonomes-e3401.md`,
  C-e3401-1 à 7.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/e3401/rouges.log`) : `onboarding_bank_account_e2e`, 9 rouges sur 10 sur la route de
  `219bd553` (le dixième, « description faite d'espaces → absente », était vert avant la story : un corps sans description écrivait
  déjà `NULL`) ; Vitest, 12 rouges sur 12 (`onboarding-bank-step.test.ts` 10, `onboarding.api.test.ts` 2).
- **Tests neufs** (de `219bd553` à la tête) : 11 `#[sqlx::test]` dans `onboarding_bank_account_e2e.rs`, 1 `#[test]` de
  désérialisation dans `routes/onboarding.rs` ; 12 Vitest ; 1 spec E2E (`onboarding-bank-account.spec.ts`). Tests adaptés :
  `onboarding_audit_e2e.rs` (clés des instantanés + `description`), `bank_accounts_repository.rs` (six appels
  d'`upsert_primary_in_tx` avec `PrimaryDetails`), `BankAccountJournalLinkForm.test.ts` (le `5000` d'actif proposé),
  `onboarding-motif.test.ts` (simulacre de `fetchAccounts`), et trois compteurs de garde (`bank-name-inventory`,
  `sitesTotal` 2922 → 2928, relevé `onboarding-`/… 407 → 409).
- **Mutations** (`kesh-gate-logs/e3401/mut/`, script `mut-e3401.py`, restauration `git checkout --` + `touch`) : **8 sur 8
  tuées**. M1 → `bank_step_replays` ; M2 → 5 tests (création, pool d'une connexion, rejeux, 1100 sans rôle, mise à jour sans
  champs) ; M3 → refus des rôles et compte renuméroté ; M3b (rôle remplacé par les numéros 1100/2000) → compte renuméroté et 1100
  sans rôle ; M7 (UPDATE qui nomme toujours les deux colonnes) → `bank_step_update_without_new_fields_keeps_them` et
  `bank_step_replays` (premier essai écrit faux, ne compilait pas : `M7-essai1-ne-compile-pas.log`, rejoué) ; M4 → 7 Vitest dont
  « sans 1020 » ; M5 → « 1020 non imputable » et la liste ; M6 → les deux tests de l'API. Aucune équivalente.
- **Gates** (relevés dans `kesh-gate-logs/e3401/`), base `kesh_e3401` recréée à neuf :
  - `scripts/test-fast.sh --ci` au commit `b732d3ba` : fmt, clippy, nextest **5150 passés, 12 ignorés** (`gate-test-fast.log`) ;
    un premier passage au commit `166826c5` avait échoué à la compilation des tests de `kesh-db` (`bank_accounts_repository.rs`
    appelait `upsert_primary_in_tx` à deux arguments — le gate ciblé ne compilait pas ce binaire), corrigé par `b732d3ba` ;
  - `cargo build --workspace --all-targets` à `b732d3ba` : vert ;
  - `scripts/gate-ci.sh kesh_ci_e3401` à `b732d3ba` : **5157 passés, 0 échec, 16 ignorés** (`gate-ci.log`) ;
  - frontend à `166826c5` : `npm run check` 0 erreur (29 avertissements, aucun dans les fichiers de la story),
    `lint-i18n-ownership` vert, `npm run build` vert ; `test:unit` 2 rouges (les deux compteurs de garde), corrigés par
    `c9025b3a` (tests seuls) et **rejoué en entier à `c9025b3a`** : **2893 passés, 224 fichiers** (`front-test-unit-2.log`) ;
  - **E2E complet** (`e2e-r1.*`, base `kesh_e2e_e3401` recréée, port 3341, recette entière, `KESH_INBOX_DIR` et
    `KESH_DOCUMENTS_DIR` passés aussi au runner) : **386 passés, 7 échecs, 18 ignorés** ; les sept sont ceux de la KF-029 (#97)
    de `docs/testing.md` § « Les échecs attendus » (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150`) ; la spec neuve passe. Le binaire de l'E2E est celui de `c9025b3a` (aucun code de
    production après `b732d3ba`).
  - Le dernier commit de code de production est `b732d3ba` ; après lui, seuls des tests frontend (`c9025b3a`), le PDF et les
    comptes rendus.
- **Manuel** : `user-manual.tex` (Chemin B, § *Comptes bancaires*, note API) ; PDF régénéré (`make user`, 3 passes) et contrôlé
  aplati : « présélectionné s’il existe » ×2, « se modifient ensuite » ×1, « laisse inchangés » ×1, « ne se saisit pas à
  l’onboarding » ×0. ⚠️ Le motif de contrôle de la fiche (`modifi[a-z]* [a-z ]*Comptes bancaires`) ne peut pas correspondre à
  « se modifient ensuite dans Administration → Comptes bancaires » (majuscule, flèche).
- **Captures** (thème clair, base jetable `kesh_e2e_e3401`) : `kesh-gate-logs/e3401/captures/34-1-01-etape-1020-preselectionne.png`,
  `34-1-02-etape-saisie.png`, `34-1-03-liste-en-echec.png`. Sur la troisième (requête des comptes interrompue), le motif affiché
  est celui du client (« Impossible de contacter le serveur… »), avec la bannière de base indisponible du client ; le repli
  « … peut être lié plus tard » ne s'affiche que pour une erreur sans message.
- **Consigne Edit** : tout le code a été écrit par l'outil Edit/Write, à deux exceptions déclarées : `cargo fmt --all`
  (formateur officiel, après les Edit) ; les mutations, appliquées et restaurées par `mut-e3401.py` (permis). Une correction du
  script de mutation (hors dépôt) l'a été par un court `python3`.

### File List

- `crates/kesh-api/src/routes/onboarding.rs` — corps, contrôles, refus des rôles, audit, test de désérialisation.
- `crates/kesh-api/src/routes/bank_accounts.rs` — `normalize_description`, `validate_journal_account_id` en `pub(crate)`.
- `crates/kesh-db/src/repositories/accounts.rs` — `RoleLookupLock`, `find_singleton_role_in_tx`.
- `crates/kesh-db/src/repositories/company_invoice_settings.rs` — la finalisation la réemploie.
- `crates/kesh-db/src/repositories/bank_accounts.rs` — `PrimaryDetails`, `upsert_primary_in_tx`, `is_no_op_change`.
- `crates/kesh-db/src/entities/bank_account.rs` — doc-comment de `description`.
- `crates/kesh-api/tests/onboarding_bank_account_e2e.rs` (neuf), `onboarding_audit_e2e.rs`, `crates/kesh-db/tests/bank_accounts_repository.rs`.
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` — deux clés.
- `frontend/src/routes/onboarding/+page.svelte`, `frontend/src/lib/features/onboarding/onboarding.api.ts`, `onboarding.svelte.ts`,
  `frontend/src/lib/features/bank-accounts/format.ts`, `BankAccountJournalLinkForm.svelte`,
  `frontend/src/routes/(app)/bank-accounts/+page.svelte`.
- Tests frontend : `onboarding-bank-step.test.ts` (neuf), `onboarding.api.test.ts` (neuf), `onboarding-motif.test.ts`,
  `BankAccountJournalLinkForm.test.ts`, `bank-name-inventory.test.ts`, `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`.
- `frontend/tests/e2e/onboarding-bank-account.spec.ts` (neuf).
- `docs/manual/fr/user-manual.tex`, `user-manual.pdf` ; `CHANGELOG.md` (0.17.6, *Ajouté*).
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e3401.md` (neuf), `34-1-assistant-compte-bancaire-review-prompt-p1.md`
  (neuf), `sprint-status.yaml`.
