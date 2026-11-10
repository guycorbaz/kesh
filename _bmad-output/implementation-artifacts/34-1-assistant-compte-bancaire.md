# Story 34-1 : l'assistant de démarrage pose la description et le compte comptable du compte bancaire (#966)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e341.md`,
C-e341-1 à C-e341-5. Commits en `refs #966` ; à la PR : `closes #966`.*

## Story

En tant qu'Administrateur qui installe Kesh, je veux saisir dès l'assistant la description du compte bancaire et le compte
comptable auquel il est lié, pour ne pas découvrir à la première réconciliation que le compte n'est pas configuré
(`BANK_ACCOUNT_NOT_CONFIGURED`), et savoir que ces choix se modifient après coup.

## Décision de Guy (fichier d'epic, #966, 2026-11-10)

Le compte bancaire peut être lié, **de manière facultative**, à un compte du plan, **présélectionné sur 1020 s'il existe**. La
page et le manuel disent que ce choix est **modifiable après coup**. ⛔ **Aucun rôle ne se déduit du numéro du compte**
(Story 14-3a) : 1020 n'est qu'une présélection d'écran, le serveur n'en sait rien et ne l'écrit nulle part.

## Ce que le code dit (prémisses établies)

- **Le plan est chargé avant l'étape bancaire** : `routes/onboarding.rs:353-411` (`accounting_language_in_tx`, étape 4→5) charge le
  plan (`accounts::bulk_create_from_chart_in_tx`, `:384`) ; l'étape bancaire est l'étape 6→7 (`:583-646`). La question de #966
  (« vérifier que le plan est choisi avant ») est donc réglée : il l'est, et `GET /api/v1/accounts` est servi à tout rôle authentifié
  (`lib.rs:1012`), assistant compris (le 423 ne vaut que sans utilisateur, `middleware/auth.rs:106-108`).
- **Ce que l'assistant pose aujourd'hui** : `BankAccountRequest` (`onboarding.rs:575-581`) n'a que `bank_name`, `iban`, `qr_iban` ;
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
  `Payable` (`company_invoice_settings.rs:1132-1163`, `singleton_role`). À l'étape 6, `claim_accounts_in_share_mode`
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

1. **AC-1 — le contrat** : `POST /api/v1/onboarding/bank-account` accepte `description` (facultative, `null` ou absente) et
   `journalAccountId` (facultatif). Absents, le comportement est exactement celui d'avant (un corps de la 0.17.5 passe inchangé).
2. **AC-2 — mêmes contrôles qu'à la création** : description normalisée par **la même** fonction que `POST /bank-accounts`
   (`trim`, vide → absente, 60 caractères, caractères cachés refusés, même message) ; compte lié contrôlé par **la même** fonction
   (`404` inexistant ou inactif, `400 INVALID_ACCOUNT_TYPE` hors `Asset | Liability`, `400 ACCOUNT_NOT_POSTABLE`). Aucune copie :
   les deux fonctions passent `pub(crate)` et sont appelées (garde DRY : `git grep -n "fn normalize_description\|fn validate_journal_account_id" -- crates` rend une ligne chacune).
3. **AC-3 — ce qui est écrit** : le compte créé porte la description et le lien saisis ; sans saisie, `NULL`. Un rejeu identique
   (même banque, IBAN, QR-IBAN, description, lien) est un no-op (`Unchanged`, aucune trace) ; un rejeu qui ne change que le lien ou la
   description est un `Updated`.
4. **AC-4 — pas de lien vers un futur compte de créance** : un `journalAccountId` qui porte le rôle `Receivable` ou `Payable` est refusé
   en `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, le même code et le même rendu qu'à `POST /bank-accounts`, avant toute écriture, l'étape
   restant à 6. Les rôles sont lus dans la transaction, comme la finalisation les lira (`singleton_role`).
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

- [ ] **T1 — rouges d'abord** (journal `kesh-gate-logs/e341/rouges.log`) :
  - `crates/kesh-api/tests/onboarding_path_b_e2e.rs` (ou fichier neuf `onboarding_bank_account_e2e.rs`) : AC-1 (corps 0.17.5 inchangé),
    AC-3 (description et lien écrits ; no-op ; `Updated`), AC-4 (rôle `Receivable`, rôle `Payable`, étape restée à 6, aucune ligne
    `bank_accounts`), AC-2 (61 caractères, compte `Revenue`, compte non imputable), AC-5 (détails de l'audit).
  - Vitest de l'écran (fichier neuf `frontend/src/routes/onboarding/onboarding-bank-step.test.ts`, patron de
    `routes/(app)/bank-accounts/bank-accounts-page.test.ts:31-33` pour le simulacre de `fetchAccounts`) : présélection 1020 ; aucun 1020 →
    « Aucun » ; 1020 non imputable → « Aucun » ; compte de rôle `Receivable` absent de la liste ; échec du chargement → « Aucun » et envoi
    possible ; le corps envoyé porte `description` et `journalAccountId` (ou `null`).
- [ ] **T2 — dépôt** : `upsert_primary_in_tx` reçoit la description et le lien sans élargir `NewBankAccount` (C-e341-1) — forme
  retenue : un second paramètre `PrimaryDetails { description: Option<String>, journal_account_id: Option<i64> }`, l'enveloppe
  `upsert_primary` passant `PrimaryDetails::default()` (ses appelants de test restent inchangés) ; `INSERT` et `UPDATE` nomment les deux
  colonnes ; `is_no_op_change` les compare. ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [ ] **T3 — route** : `BankAccountRequest` + `#[serde(default)]` ; `normalize_description` et `validate_journal_account_id` en
  `pub(crate)` dans `routes/bank_accounts.rs`, appelées par `set_bank_account` avant la transaction (même ordre de refus qu'à la création :
  banque, description, IBAN, QR-IBAN, compte lié) ; dans la transaction, après `lock_company`, si un lien est demandé : comptes de rôle
  `Receivable`/`Payable` lus (une fonction du dépôt, `company_invoice_settings::claim_accounts_by_role_in_tx`, que `insert_with_defaults_in_tx`
  réemploie pour ses deux lectures — DRY), puis `refuse_if_ledger_is_claim_account`. Audit (AC-5).
- [ ] **T4 — écran** : `onboarding.api.ts` et `onboarding.svelte.ts` (signature à objet `{ bankName, iban, qrIban, description, journalAccountId }`) ;
  `+page.svelte` : chargement des comptes à l'entrée de l'étape 6, filtre (AC-6), présélection, deux champs, la phrase AC-7. Le filtre
  s'écrit **une fois** : extraire celui de `bank-accounts/+page.svelte:157-168` en fonction de `lib/features/bank-accounts/format.ts`
  (`linkableLedgerAccounts(accounts, excludedIds)`), appelée par les deux écrans ; l'assistant passe les identifiants des comptes de rôle
  `Receivable`/`Payable`.
- [ ] **T5 — i18n** : `onboarding-bank-editable-later` aux quatre locales ; libellés réutilisés (`bank-accounts-labels-description`,
  `bank-accounts-help-description`, `bank-accounts-labels-journal-account-id`, `bank-accounts-labels-not-configured`). `npm run lint-i18n-ownership`.
- [ ] **T6 — E2E** : spec neuve `frontend/tests/e2e/onboarding-bank-account.spec.ts`, état amené à l'étape 6 **par l'API** (patron
  `onboarding-motif.spec.ts`), écran joué par sélecteurs d'`id` : 1020 présélectionné, description saisie, *Enregistrer* ; puis
  `GET /api/v1/bank-accounts` rend la description et le `journalAccountId` du 1020. La valeur doit **traverser la frontière HTTP**.
- [ ] **T7 — manuel** (AC-9), capture de l'étape (thème clair, base jetable) dans `kesh-gate-logs/e341/captures/34-1-*.png`.
- [ ] **T8 — CHANGELOG** : `## [0.17.6] — Non publié`, *Ajouté*, `refs #966`.

### Mutations nommées

- **M1** : `is_no_op_change` sans la comparaison du lien → rouge AC-3 (`Updated` attendu).
- **M2** : l'`INSERT` d'`upsert_primary_in_tx` sans `journal_account_id` → rouge AC-3 et E2E.
- **M3** : la lecture des rôles retirée (AC-4) → rouge sur `Receivable` et `Payable`.
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

- **Le solde d'ouverture à l'assistant** : il n'y a encore aucun exercice à l'étape 6 (il naît à la finalisation, `onboarding.rs:930`,
  `create_if_absent_in_tx`) ; la 34-2 le pose au formulaire *Comptes bancaires*. Rien à tracer : la 34-2 le dit au manuel.
- **Le chemin A (démonstration)** : le seed crée ses comptes lui-même ; inchangé.
- **#44 (KF-002-CR-001)**, comptes 1100/3000 manquants à la finalisation : sans rapport, inchangé.

## Recoupements

`routes/(app)/bank-accounts/+page.svelte` et `lib/features/bank-accounts/format.ts` avec **34-2** (même formulaire, après cette story) et
**34-11a** (type de compte, IBAN facultatif) ; `routes/onboarding.rs` et `upsert_primary_in_tx` avec **34-11a** (l'IBAN y devient-il
facultatif ? non : l'assistant ne crée qu'un compte bancaire, C-e341-9). Ordre : 34-1, puis 34-2, puis 34-11a.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
