# Story 34-11a2 : créer et modifier une carte de crédit — type, quatre derniers chiffres, lien à un passif (#972)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), à la remédiation de la validation P1 du groupe e341 (découpe de l'ancienne 34-11a,
arbitrage de l'orchestrateur), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`, numéros de ligne de `a75835db`.
Release **0.17.6**, Epic 34. **Aucune migration** : le schéma (IBAN facultatif, `account_kind`, `card_last4`, `CHECK`s) est posé par la
**34-11a1** (C-e341-29). Registre : `e34-choix-autonomes-e341.md`, C-e341-15, 16, 19, 21, 30 à 33. Suit **34-11a1**. Commits en `refs #972` ;
à la PR : `refs #972` (la 34-11b ferme l'issue).*

## Story

En tant que titulaire d'une carte de crédit, je veux la déclarer dans *Comptes bancaires* comme une carte — un nom, une description, ses quatre
derniers chiffres, un compte de dette —, puis la modifier, pour y faire entrer ses relevés (34-11b), sans que Kesh ne garde jamais le numéro complet.

## Décisions de Guy (#972, fichier d'epic, 2026-11-10)

Une carte est tenue **comme un compte**, **sans IBAN**, **liée à un compte de dette** ; **seuls les quatre derniers chiffres** sont gardés, le
numéro complet n'est **jamais** enregistré ; aucun QR-IBAN, aucun `pain.001`. ⛔ La carte se déclare par son **type**, jamais parce que son compte
lié serait un 2xxx (14-3a) ; rien ici n'apparie (« un appariement automatique propose, il ne crée jamais »).

## Ce que le code dit (prémisses établies)

- **Après la 34-11a1** : le schéma admet `iban NULL`, `account_kind`, `card_last4`, et ses `CHECK`s (carte ⇒ ni IBAN ni QR-IBAN ni principal ;
  quatre chiffres ASCII) ; chaque site lecteur tolère ou refuse un compte sans IBAN. Rien ne crée encore de carte.
- **La création** : `CreateBankAccountBody` (`routes/bank_accounts.rs:73-87`) ; `validate_bank_input` (`:304-340`) contrôle banque, description
  (`normalize_description`, `:282-301`), IBAN (`:322-325`), QR-IBAN ; `validate_journal_account_id` (`:353-401`) admet `Asset | Liability`
  (`:382-390`) et lit sur le **pool**.
- **La modification** : `UpdateBankAccountBody` (`:94-110`, remplacement complet, `iban: String` obligatoire `:99`) ; `update_once` (`:773-…`) ;
  le `PATCH` du lien (`:1318-1420`, dépôt `set_journal_account_id_for_company`, `repositories/bank_accounts.rs:459-560`) accepte `NULL` (délier)
  sans condition.
- **L'assistant** (`routes/onboarding.rs:589-648`) n'appelle **pas** `validate_bank_input` : il refait ses contrôles (banque `:595-600`, IBAN
  `:603-604`, QR-IBAN `:607-613`). Une garde portée par `validate_bank_input` seule ne l'atteindrait pas (validation P1, 1.5 et 3.8).
- **Le retypage d'un compte du plan** : `accounts::update` (`repositories/accounts.rs:488-…`), `account_type` réécrit (`:591-597`), gardé seulement
  par la confirmation des écritures historiques (`:575-588`, `AccountHasEntries`) — **jamais** par un compte bancaire qui le désigne. Un passif lié
  à une carte, retypé `Expense` ou `Asset`, casserait « carte ⇒ passif » sans rien qui rougisse ; c'est **déjà vrai** du lien `Asset | Liability`
  d'un compte bancaire (validation P1, 3.4).
- **Les sélecteurs qui proposeraient une carte** (inventaire de la 34-11a1, S13) : règlement d'une facture client (`SettleInvoiceDialog.svelte:249`),
  paiement d'une facture fournisseur (`supplier-invoices/[id]/+page.svelte:888`), lots (`payment-batches/+page.svelte:162`, déjà filtré par 34-11a1),
  import (`BankImportUpload.svelte:372`, CAMT filtré par 34-11a1), relevé saisi (`ManualStatementForm.svelte:346`), réconciliation
  (`reconciliation/+page.svelte:68`), lien (`BankAccountJournalLinkForm.svelte:135`).
- **Le formulaire** : `routes/(app)/bank-accounts/+page.svelte:520-592` (création et édition), `:157-168` (comptes liables ; après la 34-1 :
  `linkableLedgerAccounts`).

## Choix (détail au registre)

- **C-e341-15 — quatre chiffres exactement, et garde de Luhn**, précisée à la validation P1 (3.8) : (a) la garde s'applique au **nom** et à la
  **description**, pour **les deux types**, par une fonction de `kesh-core` appelée par `validate_bank_input` (création, `PUT`) **et** par
  `set_bank_account` (assistant) ; (b) pas aux quatre chiffres (sans objet) ; (c) **forme reconnue** : 13 à 19 chiffres contigus, ou des groupes de
  quatre chiffres séparés par une espace ou un tiret (`1234 5678 9012 3456`, `1234-5678-9012-345`) — une date `2026-11-10` ou une référence
  `20261110-0042` n'est pas de cette forme ; puis la somme de Luhn ; (d) message qui dit pourquoi (« ressemble à un numéro de carte complet ; Kesh ne
  garde que les quatre derniers chiffres ») ; (e) un compte existant dont le nom porterait une telle suite serait refusé à son prochain `PUT` :
  probabilité faible, écrite au manuel.
- **C-e341-16 — une carte est toujours liée à un passif** : création et `PUT` exigent `journalAccountId` `Liability` ; `PATCH` qui délie une carte
  ou la relie à un non-passif : refusé.
- **C-e341-30 — `PUT` d'une carte** (validation P1, 3.2) : `accountKind` facultatif au `PUT` ; **absent → le type courant** (un client 0.17.5 qui
  remplace une carte ne la transforme pas en compte bancaire, et ne se voit pas réclamer un IBAN) ; présent et différent →
  `400 BANK_ACCOUNT_KIND_IMMUTABLE`. Pour une carte : `iban`, `qrIban` absents ou nuls, `isPrimary` faux ; sinon `400`.
- **C-e341-31 — retypage d'un compte lié** (validation P1, 3.4 ; limite voisine corrigée) : `accounts::update` refuse
  `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT` le changement de type d'un compte lié à un compte bancaire **non archivé** vers un type que ce lien n'admet
  plus : hors `Liability` pour une carte, hors `Asset | Liability` pour un compte bancaire ; le refus nomme le compte bancaire. Écarté : avertir
  seulement (l'invariant se casserait en silence).
- **C-e341-32 — qui propose une carte** (validation P1, 3.9) : le **paiement d'une facture fournisseur** la propose (D créanciers / C passif de la
  carte : juste) ; le **règlement d'une facture client** ne la propose pas (un client ne paie pas sur la carte du titulaire) ; relevé saisi, import
  CSV, réconciliation, lien : proposée.
- **C-e341-19 — l'assistant reste bancaire** (IBAN obligatoire, principal) ; seule la garde de Luhn l'atteint.
- **C-e341-21 — affichage « Visa — Personnel •••• 1234 »** : la fonction de `format.ts` posée par la 34-11a1.
- **C-e341-33 — le filtre des comptes liables reçoit un paramètre** « passif seulement » (`linkableLedgerAccounts(accounts, excluded, { liabilityOnly })`,
  fonction de la 34-1, C-e341-5).

## Acceptance Criteria

1. **AC-1 — créer une carte** : `POST /bank-accounts` `accountKind: "credit_card"`, `bankName`, `description?`, `cardLast4?`, `journalAccountId`
   (passif) → `201`, `iban` et `qrIban` `null`, `isPrimary` faux. `accountKind` absent vaut `"bank"` : un corps 0.17.5 passe inchangé.
2. **AC-2 — refus à la création**, chacun sans écriture : IBAN ou QR-IBAN fourni ; `isPrimary: true` ; sans compte lié ; compte lié `Asset`
   (`400 CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`) ; `cardLast4` autre que quatre chiffres (`"12345"`, `"12a4"`, `"123"`) ; un numéro de carte de la forme
   C-e341-15 (c) valide au sens de Luhn dans le nom ou la description (`400 CARD_NUMBER_NOT_ALLOWED`).
3. **AC-3 — Luhn, positifs et négatifs** (tests de table `kesh-core`) : numéros de test publics des réseaux (15 et 16 chiffres, contigus, espacés par
   quatre, tirets) refusés ; **acceptés** : `2026-11-10`, `20261110-0042`, un IBAN, un numéro de téléphone suisse, un montant `1'234.50`, une suite de
   16 chiffres qui ne passe pas Luhn. La garde atteint aussi l'assistant (`POST /onboarding/bank-account`, nom ou description).
4. **AC-4 — modifier une carte** (C-e341-30) : `PUT` sans `accountKind` modifie nom, description, `cardLast4`, lien (passif) ; `PUT` avec
   `accountKind: "bank"` → `400 BANK_ACCOUNT_KIND_IMMUTABLE` ; avec IBAN, QR-IBAN ou `isPrimary` → `400` ; avec un lien `Asset` ou sans lien → `400`.
   `PATCH` qui délie une carte ou la relie à un `Asset` → `400`. Un compte bancaire garde exactement ses règles d'avant (tests existants de
   `bank_accounts_e2e.rs` verts sans modification).
5. **AC-5 — retypage** (C-e341-31) : retyper `Expense` le passif lié à une carte → `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT`, rien d'écrit ; retyper
   `Revenue` l'actif lié à un compte bancaire → même refus ; un compte lié à un compte bancaire **archivé** se retype ; un compte non lié se retype
   comme avant.
6. **AC-6 — audit** : `bank_account.created` porte `account_kind` et `card_last4_present` — **jamais** les chiffres eux-mêmes ; `bank_account.updated`
   idem dans ses instantanés.
7. **AC-7 — l'écran** : *Comptes bancaires* : choix *Compte bancaire / Carte de crédit* à la création (non modifiable ensuite, affiché en édition) ;
   pour une carte : IBAN, QR-IBAN et *principal* masqués, *Quatre derniers chiffres (facultatif)* avec l'aide « Kesh ne garde jamais le numéro
   complet », compte lié **obligatoire**, liste réduite aux passifs (C-e341-33). Le solde d'ouverture de la 34-2 s'y applique, sens *Dette*
   présélectionné. Sélecteurs (C-e341-32) : la carte est proposée au paiement fournisseur, pas au règlement client.
8. **AC-8 — codes d'erreur** : `CARD_NUMBER_NOT_ALLOWED`, `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`, `BANK_ACCOUNT_KIND_IMMUTABLE`,
   `ACCOUNT_LINKED_TO_BANK_ACCOUNT` : variantes `AppError`, `errors.rs`, messages aux quatre `messages.ftl`, libellés frontend, tableau d'erreurs
   de `docs/api-external.md`.
9. **AC-9 — manuel** : utilisateur, *Comptes bancaires* : le type carte, les quatre chiffres, le refus d'un numéro complet, le lien à un passif, ce
   que la carte ne fait pas (QR-facture, `pain.001`, CAMT), où elle est proposée ; *Plan comptable* : le retypage refusé d'un compte lié.
   `api-external.md` : corps de création et de `PUT`, refus. PDF régénéré, aplati.

## Tâches

- [ ] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : AC-1 à AC-6, Vitest AC-7.
- [ ] **T2 — `kesh-core`** : `looks_like_card_number(&str) -> bool` (forme + Luhn), tests de table (AC-3).
- [ ] **T3 — routes** : corps (`accountKind`, `cardLast4`), `validate_bank_input` par type, garde de Luhn, contrôles C-e341-16 et C-e341-30 (`POST`,
  `PUT`, `PATCH`), `set_bank_account` (Luhn seulement), audit (AC-6).
- [ ] **T4 — dépôt** : `accounts::update` refuse le retypage (C-e341-31), sous le verrou de la ligne `accounts` déjà pris, lecture des comptes
  bancaires liés non archivés **après** ce verrou. ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [ ] **T5 — écran** (AC-7), `linkableLedgerAccounts` à paramètre, sélecteurs (C-e341-32), i18n quatre locales, `lint-i18n-ownership`.
- [ ] **T6 — codes** (AC-8).
- [ ] **T7 — E2E** `bank-accounts-credit-card.spec.ts` : créer une carte liée à un passif créé par l'API, `•••• 1234` dans la liste et dans le
  sélecteur de l'import, CAMT absent pour elle, lot refusé, numéro complet refusé avec son motif ; modifier sa description.
- [ ] **T8 — manuel** (AC-9), captures `kesh-gate-logs/e341/captures/34-11a2-*.png` (création carte, refus d'un numéro complet, liste), CHANGELOG
  *Ajouté*, `refs #972`.

### Mutations nommées

- **M1** : la garde de Luhn retirée de la description → rouge AC-2 ; **M1b** : retirée de l'assistant → rouge AC-3 (assistant).
- **M2** : la forme reconnue élargie à toute suite de 13 à 19 chiffres avec séparateurs quelconques → rouge AC-3 (`20261110-0042` ou date).
- **M3** : `PUT` sans `accountKind` lu comme `"bank"` → rouge AC-4.
- **M4** : `PATCH` qui délie une carte accepté → rouge AC-4.
- **M5** : le refus de retypage retiré → rouge AC-5 ; **M5b** : appliqué aussi aux comptes bancaires archivés → rouge AC-5.
- **M6** : `card_last4` écrit dans l'audit → rouge AC-6.
- **M7** : la carte proposée au règlement client → rouge Vitest AC-7.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, gate frontend complet, E2E complet au dernier commit de code.

## Manuels

Utilisateur (AC-9), `api-external.md`. **Prompt de revue : les nommer**, PDF aplatis.

## Ce qui ne se traite pas

- **PayPal, caisse** : l'énumération est prête, le type ne l'est pas — rien à tracer tant que Guy ne le demande pas.
- **Un relevé CAMT de carte** : issue demandée par la 34-11a1.

## Recoupements

`routes/bank_accounts.rs`, `bank-accounts/+page.svelte`, `format.ts` avec **34-1**, **34-2**, **34-11a1** ; `repositories/accounts.rs` : aucune autre
story de l'epic. La **34-11b** s'appuie sur celle-ci.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5) à la remédiation de la **validation P1 (Sonnet)** du groupe e341 : seconde moitié de l'ancienne 34-11a (3.1,
  découpe) ; y entrent **3.2** (`PUT`, `PATCH` d'une carte, C-e341-30, AC-4, M3, M4), **3.4** (retypage d'un compte lié, C-e341-31, AC-5, relu
  `repositories/accounts.rs:575-597`), **3.8** (Luhn : périmètre, forme reconnue, négatifs, C-e341-15 précisé, AC-3, M2), **3.9** (sélecteurs,
  C-e341-32), **1.5** (Luhn à l'assistant, relu `onboarding.rs:589-648`), et le paramètre « passif seulement » du filtre (axe 8, C-e341-33).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
