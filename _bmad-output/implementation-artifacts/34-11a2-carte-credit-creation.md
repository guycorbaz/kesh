# Story 34-11a2 : créer et modifier une carte de crédit — type, quatre derniers chiffres, lien à un passif (#972)

Status: done

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), à la remédiation de la validation P1 du groupe e341 (découpe de l'ancienne 34-11a,
arbitrage de l'orchestrateur), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`, numéros de ligne de `a75835db`.
Release **0.17.6**, Epic 34. **Aucune migration** : le schéma (IBAN facultatif, `account_kind`, `card_last4`, `CHECK`s) est posé par la
**34-11a1** (C-e341-29). Registre : `e34-choix-autonomes-e341.md`, C-e341-15, 16, 19, 21, 30 à 33, 39 à 43. Suit **34-11a1**. Commits en `refs #972` ;
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

- **C-e341-15 — quatre chiffres exactement, et garde de Luhn**, précisée à la validation P1 (3.8), **puis restreinte à la validation P2** (4.1,
  4.4 ; arbitrage de l'orchestrateur : la garde ne refuse **jamais** un IBAN, une référence QR ni un numéro de téléphone) : (a) elle s'applique au
  **nom** et à la **description**, pour **les deux types**, par une fonction de `kesh-core` appelée par `validate_bank_input` (création, `PUT`)
  **et** par `set_bank_account` (assistant) ; (b) pas aux quatre chiffres (sans objet). (c) **Suite candidate, définie par algorithme** (validation P3, 4.2 : la
  rédaction précédente contredisait l'exemple de l'AC-3) : une suite est une **plus longue** séquence de chiffres dans laquelle deux groupes consécutifs
  sont séparés par **exactement un** caractère séparateur (espace U+0020, espace insécable U+00A0 ou U+202F, tiret `-`) ; un double séparateur ou un
  séparateur de queue termine la suite. Elle est **entière** si le caractère **immédiatement avant son premier chiffre** et celui **immédiatement
  après son dernier chiffre** sont absents (début, fin du texte) ou ne sont **ni chiffre ni lettre** (une espace, un tiret isolé, une ponctuation y
  comptent). Cas de table : « Visa 4111 1111 1111 1111 perso » → suite `4111 1111 1111 1111`, bordée par deux espaces : **entière** (refus) ;
  « 4111 1111 1111 1111 » seul : refus ; « 4111111111111111perso » : le caractère après le dernier chiffre est `p`, une lettre : **pas
  entière** (accepté) ; « CH95 0076 2008 0636 0837 7 » : le caractère avant le premier chiffre est `H` : pas entière (accepté). (d) **Groupements admis, et eux seuls** : contigu ; 4-4-4-4 ; 4-4-4-4-3 (19 chiffres) ; **4-6-5** (American Express,
  `3782 822463 10005`) ; **4-6-4** (Diners) ; séparateur unique et constant dans la suite. Un IBAN imprimé (`CH95 0076 2008 0636 0837 7` : la suite
  commence par `95`, collée à une lettre, et se groupe 2-4-4-4-4-1), une QRR imprimée (groupes de cinq) ou un téléphone (`+41 79 123 45 67`) n'ont
  aucun de ces groupements. (e) **Longueur totale de 13 à 19 chiffres** : une QRR contiguë (27) n'en est pas une. (f) **Préfixe d'un réseau**
  (IIN) cohérent avec la longueur : Visa `4` (13, 16, 19) ; Mastercard `51`-`55`, `2221`-`2720` (16) ; American Express `34`, `37` (15) ; Diners
  `300`-`305`, `36`, `38` (14) ; Discover `6011`, `644`-`649`, `65` (16, 19) ; JCB `3528`-`3589` (16-19) ; Maestro `50`, `56`-`69` (13-19). Aucun
  préfixe ne commence par `0` : `0041791234564` (13 chiffres, Luhn vrai, vérifié) n'en est pas une. (g) Puis la somme de **Luhn**. (h) Message qui dit pourquoi (« ressemble à un numéro
  de carte complet ; Kesh ne garde que les quatre derniers chiffres »). (i) Un compte existant dont le nom porterait une telle suite serait refusé à
  son prochain `PUT` : probabilité faible, écrite au manuel. **Repli consigné** : si le dev constate qu'une de ces bornes ne se tient pas simplement
  (un faux positif sur une donnée usuelle que les tests négatifs ne ferment pas), la garde est **retirée du nom et de la description** et la story
  s'en tient à la décision de Guy, qui porte sur le champ du numéro (quatre chiffres) — retrait écrit au Change Log, choix C-e341-15 annoté.
  **Limites assumées** (validation P3, 4.3 ; la garde protège d'une **saisie accidentelle**, elle n'est pas un filtre de sécurité) : (1) un numéro suivi
  de sa date ou de son CVC séparés par une espace (« 4111 1111 1111 1111 12/28 ») forme la suite 4-4-4-4-2, groupement non admis : il passe ; (2) le
  point, la barre ou tout autre séparateur que ceux de (c) (« 4111.1111.1111.1111 ») découpent en groupes de quatre : il passe ; (3) Diners en
  16 chiffres (`3056 9300 0902 0004`) : le préfixe `300`-`305` n'est admis qu'à 14 : il passe ; (4) un numéro **précédé** de chiffres sous un séparateur
  unique (« Facture 12 4111 1111 1111 1111 », « 2026-11-10 4111 1111 1111 1111 ») forme une seule suite de groupement non admis : il passe (validation
  P4, F8). Écrites au registre et au manuel.
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
- **C-e341-19 — l'assistant reste bancaire** (IBAN obligatoire, principal) ; seule la garde de Luhn l'atteint. Son corps n'a pas
  `deny_unknown_fields` : un `accountKind` ou un `cardLast4` envoyé y est **ignoré** et un compte bancaire est créé — client interne, angle mort assumé
  (validation P5, F7), à la différence de C-e341-42 qui vise les routes publiques des comptes bancaires.
- **C-e341-39 — `NewBankAccount.iban` reste `String`** (validation P2, 3.2 de la 34-11a1) : `create_once` lie l'IBAN d'une carte (`NULL`) depuis
  l'entrée validée, comme la description (`routes/bank_accounts.rs:642`), sans toucher les 68 constructions de `NewBankAccount`.
- **C-e341-40 — le verrou du type** (validation P2, 4.2) : le retypage (`accounts::update`) prend la ligne `accounts` **`FOR UPDATE` en tête**
  (elle est lue aujourd'hui sans verrou, `repositories/accounts.rs:500-505`, ce que le code dit lui-même, `:546-549`), puis lit les comptes bancaires
  liés ; la création, le `PUT` et le `PATCH` d'un compte bancaire relisent **dans leur transaction** le type du compte lié `LOCK IN SHARE MODE`, après
  les réglages (ordre global `companies` → réglages → `accounts` → `bank_accounts`). Les deux gestes se sérialisent sur la ligne `accounts` ; aucun
  ne lit le type sur le pool seulement. La route `PUT /api/v1/accounts/{id}` est enveloppée dans `retry_app_on_deadlock` (absente aujourd'hui de
  `routes/accounts.rs`) ; son registre **reste `Traced, SansEcritureAuJournal`** (elle n'écrit pas au journal) et le point (vi) du doc-comment du
  registre (`audit_route_registry.rs:159`, « vingt-quatre routes `SansEcritureAuJournal` sont rejouées quand même ») passe à **vingt-cinq**, en
  nommant `accounts::update_account`, son enveloppe et son opération (validation P3, 4.4).
- **C-e341-41 — pas de carte au règlement client, refusé au serveur** (validation P2, 4.3) : le règlement d'une facture client par virement sur un
  compte de type carte est refusé `400 SETTLEMENT_ACCOUNT_IS_CREDIT_CARD` par la route de règlement ; l'acceptation d'une proposition « facture »
  par la réconciliation d'une transaction de carte reste possible (un remboursement reçu sur la carte peut solder une facture) : angle mort assumé,
  écrit au manuel.
- **C-e341-21 — affichage « Visa — Personnel •••• 1234 »** : la fonction de `format.ts` posée par la 34-11a1.
- **C-e341-33 — le filtre des comptes liables reçoit un paramètre** « passif seulement » (`linkableLedgerAccounts(accounts, excluded, { liabilityOnly })`,
  fonction de la 34-1, C-e341-5).

## Acceptance Criteria

1. **AC-1 — créer une carte** : `POST /bank-accounts` `accountKind: "credit_card"`, `bankName`, `description?`, `cardLast4?`, `journalAccountId`
   (passif) → `201`, `iban` et `qrIban` `null`, `isPrimary` faux. `accountKind` absent vaut `"bank"` : un corps 0.17.5 passe inchangé.
2. **AC-2 — refus à la création**, chacun sans écriture : IBAN ou QR-IBAN fourni ; `isPrimary: true` ; sans compte lié ; compte lié `Asset`
   (`400 CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`) ; `cardLast4` autre que quatre chiffres (`"12345"`, `"12a4"`, `"123"`) ; un numéro de carte de la forme
   C-e341-15 valide au sens de Luhn dans le nom ou la description (`400 CARD_NUMBER_NOT_ALLOWED`). IBAN, QR-IBAN ou `isPrimary` fournis pour une
   carte : `400 CREDIT_CARD_FIELD_NOT_ALLOWED` (`details.field`). **Compte bancaire** (validation P4, F3 et F4) : `POST` d'un compte `bank` (type absent,
   puis `"bank"` explicite) avec `iban` absent, puis `null` → `400`, rien d'écrit (refus de `validate_bank_input`, comme en 0.17.5, et non `500` du
   `CHECK` « `bank` ⇒ IBAN ») ; `cardLast4` fourni pour un compte `bank` → `400 BANK_ACCOUNT_FIELD_NOT_ALLOWED` (`details.field = "cardLast4"`, C-e341-42),
   rien d'écrit, et non `500` du `CHECK` de `card_last4`. **« Fourni »** (validation P5, F6) : `cardLast4` absent, `null` ou `""` (après `trim`) valent
   **absent**, pour les deux types — le formulaire qui sérialise le champ vide d'un compte bancaire passe, une carte sans chiffres est acceptée et, au
   `PUT`, efface ; cas au test pour `null` et `""`, aux deux types.
3. **AC-3 — Luhn, positifs et négatifs** (tests de table `kesh-core`) : **refusés** — numéros de test publics des réseaux : Visa 16 contigu et
   4-4-4-4 espacé, Mastercard 4-4-4-4 tiret, American Express 4-6-5 (`3782 822463 10005`) et contigu, Diners 4-6-4, et chacun entouré de texte
   (« Visa 4111 1111 1111 1111 perso »). **Acceptés**, en deux familles (validation P4, F5). (i) **Construits pour passer Luhn** — la table l'asserte pour
   chacun, sans quoi le test négatif ne prouve rien : un IBAN suisse contigu (`CH9500762008063608377`, synthétique, Luhn vrai sur ses 19 chiffres,
   vérifié) et sa forme imprimée ; la référence QR `210000000003139471430009313` (27 chiffres, synthétique, clé modulo 10 récursif juste et Luhn vrai,
   calculés), contiguë et imprimée par cinq ; « 41 1111 1111 1111 11 » (Visa 16, Luhn vrai, groupement 2-4-4-4-2 non admis : tue M2b) ; `0041791234564` (13 chiffres, Luhn vrai, vérifié) ; « 4111111111111111perso » (lettre collée) ; une suite de 16 chiffres valide au sens
   de Luhn mais sans préfixe de réseau (`0…`, construite par le dev). (ii) **Acceptés par la longueur ou le groupement**, sans exigence de Luhn :
   `+41 79 123 45 67`, `2026-11-10`, `20261110-0042`, un montant `1'234.50`. La garde atteint aussi l'assistant (`POST /onboarding/bank-account`, nom ou description).
   *(Écart, revue P2 A2 : à l'assistant, le **nom** seulement — la description de l'assistant vient de la 34-1, raccord d'intégration (3).)*
4. **AC-4 — modifier une carte** (C-e341-30) : `PUT` sans `accountKind` modifie nom, description, `cardLast4`, lien (passif) — remplacement complet
   (C-26-2-8) : **`cardLast4` absent vaut effacé**, comme la description ; `PUT` avec
   `accountKind: "bank"` → `400 BANK_ACCOUNT_KIND_IMMUTABLE` ; avec IBAN, QR-IBAN ou `isPrimary` → `400 CREDIT_CARD_FIELD_NOT_ALLOWED` ; avec un lien `Asset` ou sans lien → `400`. `PUT` sur un id inconnu ou archivé, `iban` absent → `404` (et non `400`), C-e341-43 ; `PUT` d'un compte `bank` avec `iban` absent ou `null` → `400`, et avec `cardLast4` →
   `400 BANK_ACCOUNT_FIELD_NOT_ALLOWED`, rien d'écrit (validation P4, F3, F4).
   `PATCH` qui délie une carte ou la relie à un `Asset` → `400`. Un compte bancaire garde exactement ses règles d'avant (tests existants de
   `bank_accounts_e2e.rs` verts sans modification).
5. **AC-5 — retypage** (C-e341-31) : retyper `Expense` le passif lié à une carte → `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT`, rien d'écrit ; retyper
   `Revenue` l'actif lié à un compte bancaire → même refus ; un compte lié à un compte bancaire **archivé** se retype ; un compte non lié se retype
   comme avant. **Concurrence** (C-e341-40) : un retypage et un `POST`/`PATCH` de carte sur le même compte, croisés, ne laissent jamais une carte liée à
   un non-passif (test à deux connexions, ou raisonnement écrit au doc-comment et dit tel quel). **Règlement client** (C-e341-41) :
   `400 SETTLEMENT_ACCOUNT_IS_CREDIT_CARD`, rien d'écrit ; un règlement fournisseur depuis une carte passe.
6. **AC-6 — audit** : `bank_account.created` porte `account_kind` et `card_last4_present` — **jamais** les chiffres eux-mêmes ; `bank_account.updated`
   idem dans ses instantanés.
7. **AC-7 — l'écran** : *Comptes bancaires* : choix *Compte bancaire / Carte de crédit* à la création (non modifiable ensuite, affiché en édition) ;
   pour une carte : IBAN, QR-IBAN et *principal* masqués, *Quatre derniers chiffres (facultatif)* avec l'aide « Kesh ne garde jamais le numéro
   complet », compte lié **obligatoire**, liste réduite aux passifs (C-e341-33). Le solde d'ouverture de la 34-2 s'y applique, sens *Dette*
   présélectionné. Sélecteurs (C-e341-32) : la carte est proposée au paiement fournisseur, pas au règlement client.
   *(Écarts, revue P2 A2 : liste des passifs par `admitsLedgerType`, non `linkableLedgerAccounts` — raccord (1) ; sens *Dette* du solde
   d'ouverture non livré — raccord (2). Titre « Nouveau compte bancaire » sur le formulaire d'une carte : #1037.)*
8. **AC-8 — codes d'erreur** : `CARD_NUMBER_NOT_ALLOWED`, `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`, `BANK_ACCOUNT_KIND_IMMUTABLE`,
   `ACCOUNT_LINKED_TO_BANK_ACCOUNT`, `CREDIT_CARD_FIELD_NOT_ALLOWED`, `BANK_ACCOUNT_FIELD_NOT_ALLOWED`, `SETTLEMENT_ACCOUNT_IS_CREDIT_CARD` : variantes `AppError`, `errors.rs`, messages aux quatre `messages.ftl`, libellés frontend, tableau d'erreurs
   de `docs/api-external.md`.
   *(Écart, revue P2 A2 : « libellés frontend » — les écrans affichent le message du serveur, déjà traduit ; aucun libellé propre par code,
   C-e34a2-14.)*
9. **AC-9 — manuel** : utilisateur, *Comptes bancaires* : le type carte, les quatre chiffres, le refus d'un numéro complet, le lien à un passif, ce
   que la carte ne fait pas (QR-facture, `pain.001`, CAMT), où elle est proposée ; la phrase `user-manual.tex:2369` (« l'IBAN reste affiché à la
   suite … Deux comptes peuvent porter la même description : l'IBAN les distingue ») récrite pour une carte (les quatre chiffres en tiennent lieu) ;
   *Plan comptable* : le retypage refusé d'un compte lié.
   `api-external.md` : corps de création et de `PUT`, refus. PDF régénéré, aplati.

## Tâches

- [x] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : AC-1 à AC-6, Vitest AC-7. *(Écart : tests écrits avec le code, journal
  réel `kesh-gate-logs/e34a2/rouges.log` ; la campagne de mutations en tient lieu — Dev Agent Record.)*
- [x] **T2 — `kesh-core`** : `looks_like_card_number(&str) -> bool` (bornes, groupements, longueur, préfixe, Luhn — C-e341-15), tests de table (AC-3).
- [x] **T3 — routes** : corps (`accountKind`, `cardLast4`), et **`iban: Option<String>` avec `#[serde(default)]` dans `CreateBankAccountBody` et
  `UpdateBankAccountBody`** (`routes/bank_accounts.rs:79`, `:99`, `String` aujourd'hui ; validation P3, 4.1) — un compte **bancaire** sans IBAN rend
  `400` par `validate_bank_input`, comme en 0.17.5 ; `validate_bank_input` par type — **au `PUT`, le type courant est lu avant** (validation P5, F2) :
  lecture du compte sur le pool, filtrée par société (`find_by_id_for_company`), avant `validate_bank_input` et `validate_journal_account_id` (le type
  est immuable, la lecture hors transaction est sûre) ; un id inconnu, d'une autre société ou archivé rend `404` **avant** tout refus de forme qui
  dépend du type ; au `POST`, le type est celui du corps (absent = `bank`) —, garde de Luhn, contrôles C-e341-16 et C-e341-30 (`POST`, `PUT`,
  `PATCH`), `set_bank_account` (Luhn seulement), audit (AC-6).
- [x] **T4 — dépôt** : `accounts::update` prend la ligne `accounts` `FOR UPDATE` en tête (C-e341-40 — elle ne l'est pas aujourd'hui), puis refuse le
  retypage (C-e341-31) après lecture des comptes bancaires liés non archivés ; création, `PUT`, `PATCH` relisent le type du compte lié `LOCK IN SHARE
  MODE` dans leur transaction, **avant** le complément de la 34-2 (un refus de type n'écrit rien) ; route du plan rejouée, point (vi) du registre
  (C-e341-40). **`PUT` d'une carte** (validation P3, 4.1) : `update_for_company` (`repositories/bank_accounts.rs:606-…`, `UPDATE … iban = ?` depuis
  `NewBankAccount.iban: String`, `:658-662`) lie l'IBAN en `Option` (C-e341-39 étendu au `PUT`) ; **porteur** (validation P4, F2) : `ValidatedBankInput.iban`
  devient `Option<String>` (définition `routes/bank_accounts.rs:251-257`, calcul `:322-339`) ; les deux constructions de
  `NewBankAccount` qui lisent `input.iban` (`:630`, `:808`) prennent l'IBAN du compte bancaire (une carte ne passe pas par elles pour son IBAN) et
  l'`INSERT` de `create_once` (`:642`, `bind(&new.iban)`) lie `input.iban` ; `BankAccountPutFields` (`:560-575`, `Copy, Default`) reçoit `iban: Option<&str>`
  et `card_last4: Option<&str>` (remplacement complet : `None` = effacé), que l'`UPDATE` écrit à la place de `new.iban` ; `NewBankAccount` est inchangé, et
  son `iban` n'est plus lu au `PUT` (doc-comment d'`update_for_company` mis à jour). ⚠️ **Les huit littéraux de test** (validation P5, F4) —
  `routes/bank_accounts.rs:818`, `crates/kesh-db/tests/bank_accounts_repository.rs:918`, `:1089`, `:1109`, `:1129`, `:1146`, `:1457`, `:1666`, `:1686` —
  portent `iban: Some(<l'IBAN du payload>)` ; `iban: None` ou `..Default::default()` écrirait `NULL` sur un compte `bank` (`500` du `CHECK`) : le
  `Default` de `BankAccountPutFields` est documenté (« `iban` vaut `NULL` : réservé aux cartes »). Le
  `PUT` **n'a pas** de court-circuit no-op (son `UPDATE` incrémente toujours `version`) : `is_no_op_change` (`repositories/bank_accounts.rs:252-256`),
  propre à l'assistant (`:347`), reste inchangé (validation P4, F1). Les chaînes d'attendus des extracteurs (`routes/bank_accounts.rs:204`, `:209` :
  « `iban` str ») deviennent « `iban?` str » et nomment `accountKind?`, `cardLast4?`. Test :
  `PUT` d'une carte → `iban` `NULL` en base, quatre chiffres écrits, effacés s'ils sont absents. ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
  *(Écart : « avant le complément de la 34-2 » non livré, la 34-2 étant absente de la base — raccord d'intégration (4).)*
- [x] **T5 — écran** (AC-7), `linkableLedgerAccounts` à paramètre, sélecteurs (C-e341-32), i18n quatre locales, `lint-i18n-ownership`.
  *(Écart : `linkableLedgerAccounts` n'existe pas dans la base ; `admitsLedgerType` en tient lieu — raccord d'intégration (1) ; le sens *Dette*
  du solde d'ouverture — raccord (2).)*
- [x] **T6 — codes** (AC-8).
- [x] **T7 — E2E** `bank-accounts-credit-card.spec.ts` : créer une carte liée à un passif créé par l'API, `•••• 1234` dans la liste et dans le
  sélecteur de l'import, CAMT absent pour elle, lot refusé, numéro complet refusé avec son motif ; modifier sa description.
- [x] **T8 — manuel** (AC-9), captures `kesh-gate-logs/e341/captures/34-11a2-*.png` (création carte, refus d'un numéro complet, liste), CHANGELOG
  *Ajouté*, `refs #972`.

### Mutations nommées

- **M1** : la garde de Luhn retirée de la description → rouge AC-2 ; **M1b** : retirée de l'assistant → rouge AC-3 (assistant).
- **M2** : la borne « lettre adjacente » retirée → rouge AC-3 sur « 4111111111111111perso » (préfixe Visa, 16 ; l'IBAN contigu ne la tue pas : son
  préfixe `95` n'est d'aucun réseau) ; **M2b** : groupements quelconques admis → rouge AC-3 sur « 41 1111 1111 1111 11 » (2-4-4-4-2, Visa 16, Luhn vrai) ;
  **M2c** : préfixe de réseau non contrôlé → rouge AC-3 (`0041…`, suite `0…`) ; **M2e** : 4-6-5 non reconnu → rouge AC-3 (Amex). **M2d** (longueur non
  bornée) est **équivalente** et retirée de la liste : (f) n'admet que des longueurs de 13 à 19, (e) est redondante avec (f) ; la QRR contiguë exerce
  (e) et (f) ensemble (validation P5, F1).
- **M3** : `PUT` sans `accountKind` lu comme `"bank"` → rouge AC-4.
- **M4** : `PATCH` qui délie une carte accepté → rouge AC-4.
- **M5** : le refus de retypage retiré → rouge AC-5 ; **M5b** : appliqué aussi aux comptes bancaires archivés → rouge AC-5.
- **M6** : `card_last4` écrit dans l'audit → rouge AC-6.
- **M7** : la carte proposée au règlement client → rouge Vitest AC-7 ; **M7b** : refus serveur retiré → rouge AC-5 (règlement client).
- **M10** : `validate_bank_input` accepte `None` pour un compte `bank` → rouge AC-2 et AC-4 (`400` attendu, `500` du `CHECK` sinon) ; **M10b** : `cardLast4`
  non refusé pour un compte `bank` → rouge AC-2 et AC-4.
- **M9** : `BankAccountPutFields` sans `card_last4` (non écrit au `PUT`) → rouge du test du `PUT` d'une carte ; **M9b** : l'IBAN du `PUT` lié depuis
  `NewBankAccount.iban` (chaîne vide) → rouge du même test (`CHECK` carte ⇒ IBAN nul).
- **M8** : le `FOR UPDATE` de tête du retypage retiré → rouge du test croisé (AC-5), s'il est déterministe ; sinon écrite non tuée.

*Ajoutées au développement et aux revues (journaux `kesh-gate-logs/e34a2/mut/M*.log` ; passe 1 dans `mut-p1/`) :*

- **M11**, **M11b** : `recheck_ledger_type_in_tx` retiré de la création, du `PUT` → rouges des tests croisés par les routes (revue P1, B2).
- **M12** : le `PATCH` d'une carte prépare son contrôle à la règle d'un compte bancaire → rouge `patch_d_une_carte` (revue P1, B1).
- **M13**, **M13b** : *Délier* rendu à une carte ; la liste du lien rendue à la règle d'un compte bancaire → rouges Vitest du formulaire de lien.
- **M14** : un actif choisi n'est plus retiré au passage à *Carte* → rouge Vitest de la page (revue P1, B9).
- **M15** : les cartes closes écartées du refus de retypage → rouge `retypage_carte_close_et_lien_partage` (revue P1, E7).
- **M16** : plus de masquage des IBAN → rouge `un_iban_valide_n_est_jamais_un_numero_de_carte` (revue P1, B7).
- **M17** : un numéro complet dans `cardLast4` n'est plus nommé → rouge `numero_complet_dans_les_quatre_chiffres` (revue P1, E5).
- **M18** : `maxlength={4}` rétabli → rouge Vitest « numéro complet collé » (revue P1, E5).
- **M19** : le pré-vol du `PATCH` ne filtre plus les comptes archivés → rouge `patch_d_une_carte_inchangee_archivee_ou_close` (revue P2, E4 (c)).
- **Non jouées, ordre non témoigné** (revue P2, E4 (a), (b)) : déplacer `recheck_ledger_type_in_tx` **après** `update_for_company`, ou
  `locked_ledger_type` après `set_journal_account_id_for_company`. Le refus et la sonde resteraient les mêmes (le rollback défait l'écriture) ;
  seul un cycle avec un écrivain concurrent de la ligne `bank_accounts` le montrerait, et le rejeu sur interblocage le masquerait. Aucun
  test sans toucher la production ne le distingue : ordre tenu par la revue et le doc-comment, mutations écrites non tuées.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, gate frontend complet, E2E complet au dernier commit de code.

## Manuels

Utilisateur (AC-9), `api-external.md`. **Prompt de revue : les nommer**, PDF aplatis.

## Ce qui ne se traite pas

- **PayPal, caisse** : l'énumération est prête, le type ne l'est pas — rien à tracer tant que Guy ne le demande pas.
- **Un relevé CAMT de carte** : issue demandée par la 34-11a1.

## Points hérités de la revue P1 de la 34-11a1 (renvoyés ici par l'orchestrateur, 2026-11-11)

Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e34a1-review-p1-{B,E}.md`. Non corrigés dans la 34-11a1 ; à traiter ou à écarter par écrit ici.

- **E5 = B7 (LOW)** — l'écran *Comptes bancaires* ouvre l'édition complète d'une carte (`routes/(app)/bank-accounts/+page.svelte`, `formIban = ba.iban ?? ''`) ;
  l'enregistrement finit en `400 CHECK_CONSTRAINT_VIOLATION` « Valeur invalide » (`chk_bank_accounts_card_shape`), ou en refus de l'IBAN vide : refus non
  nommé. Le `PUT` d'une carte de cette fiche (AC-4, T4) le règle ; vérifier que *Modifier* d'une carte n'envoie plus d'IBAN, ou nomme son refus.
- **B5 (LOW)** — l'invariant « le principal a un IBAN » repose sur le **type** (`chk_bank_accounts_card_shape` : `is_primary = FALSE` pour une carte), non
  sur `iban IS NULL`, contrairement au principe de C-e341-18. Remède proposé : `CHECK (is_primary = FALSE OR iban IS NOT NULL)` — mais la 34-11a2 ne porte
  pas de migration (C-e341-29) : l'écrire en dette (prochaine migration de `bank_accounts`), ou borner la promesse du doc-comment de `BankAccount::iban`.
- **E6 (LOW)** — `accept` ne filtre pas le **glisser-déposer** (`BankImportUpload.svelte`, `handleDrop` → `handleFileSelect`, sans contrôle d'extension) :
  un `.xml` déposé sur une carte part au serveur, qui rend `422 BANK_IMPORT_CAMT_NEEDS_IBAN`, affiché par `setError`. Correct ; aucun test front ne montre
  ce refus affiché après un dépôt.

*Ajoutés le 2026-11-12 par l'orchestrateur, revue P2 de la 34-11a1 (deux LOW de production) :*

- **E3 (LOW)** — la clé fr-CH `bank-import-labels-drop-zone-csv-only` et son repli Svelte emploient l'apostrophe typographique U+2019,
  alors que le fichier emploie l'apostrophe droite.
- **B3 (LOW)** — `frontend/src/lib/features/bank-accounts/format.ts:26` écrit l'espace insécable de `CARD_LAST4_SEPARATOR` en littéral
  invisible : l'écrire `' '`.

## Raccords d'intégration (C-e34a2-1 ; revue P1, A1) — porteur : **l'orchestrateur, à l'intégration**

La branche part de la 34-11a1 sans la 34-1 ni la 34-2. Quatre raccords restent à faire quand elles se rejoignent ; chacun a son test attendu.

1. **`linkableLedgerAccounts({ liabilityOnly })`** (34-1, C-e341-5, C-e341-33) : la fonction de la 34-1 reçoit le paramètre, ou appelle
   `admitsLedgerType(kind, accountType)` (`frontend/src/lib/features/bank-accounts/credit-card.ts`) ; les deux listes de
   `routes/(app)/bank-accounts/+page.svelte` passent par elle. Test : `bank-accounts-page-credit-card.test.ts` reste vert (passifs seuls).
2. **Le sens *Dette* présélectionné du solde d'ouverture d'une carte** (34-2, C-e341-7) : une carte liée à un passif ouvre le solde
   d'ouverture avec *Dette* ; le paragraphe *Solde d'ouverture* du manuel utilisateur s'écrit alors (§ *Une carte de crédit*, commentaire
   LaTeX en place). Test attendu : un Vitest « carte → *Dette* présélectionné ».
3. **La garde du numéro complet sur la description de l'assistant** (34-1) : `onboarding::set_bank_account` ajoute
   `refuse_card_number("description", …)` (commentaire au site) ; `docs/api-external.md` (note ¹¹ et tableau des erreurs), le manuel
   utilisateur (§ *Une carte de crédit*, « à l'onboarding, pour le nom de la banque ») **et son PDF**, et le CHANGELOG (« le nom de la banque à
   l'onboarding ») disent alors « nom ou description » à l'assistant. Test attendu : `garde_du_numero_a_l_assistant` étendu à la description.
4. **L'ordre de la relecture du type et du complément de la 34-2** : le complément (solde d'ouverture saisi à la création d'un compte) se
   place **après** `recheck_ledger_type_in_tx` dans `create_once` (un refus de type n'écrit rien). Test attendu : une carte refusée par la
   relecture n'écrit aucune écriture d'ouverture.

## Recoupements

`routes/bank_accounts.rs`, `bank-accounts/+page.svelte`, `format.ts` avec **34-1**, **34-2**, **34-11a1** ; `repositories/accounts.rs` : aucune autre
story de l'epic. La **34-11b** s'appuie sur celle-ci.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5) à la remédiation de la **validation P1 (Sonnet)** du groupe e341 : seconde moitié de l'ancienne 34-11a (3.1,
  découpe) ; y entrent **3.2** (`PUT`, `PATCH` d'une carte, C-e341-30, AC-4, M3, M4), **3.4** (retypage d'un compte lié, C-e341-31, AC-5, relu
  `repositories/accounts.rs:575-597`), **3.8** (Luhn : périmètre, forme reconnue, négatifs, C-e341-15 précisé, AC-3, M2), **3.9** (sélecteurs,
  C-e341-32), **1.5** (Luhn à l'assistant, relu `onboarding.rs:589-648`), et le paramètre « passif seulement » du filtre (axe 8, C-e341-33).
- 2026-11-10 : **validation P2 (Opus)** — pour cette fiche : 2 M (4.1, 4.2), 3 L (4.3 à 4.5), nés de la remédiation P1. Remédiation (Opus 5.5),
  tout appliqué. **4.1** : la garde de Luhn est **gardée et restreinte** (arbitrage de l'orchestrateur) : suite entière bornée, groupements des
  réseaux seuls, 13 à 19 chiffres, préfixe IIN cohérent, puis Luhn ; tests négatifs **construits pour passer Luhn** (IBAN CH, QRR, `0041…`) ; repli
  écrit (retrait du nom et de la description) si une borne ne tient pas ; M2 à M2e. **4.2** : prémisse de la T4 fausse (`accounts::update` ne verrouille
  pas, relu `repositories/accounts.rs:498-506`, `:544-549`) : `FOR UPDATE` en tête, relecture du type `LOCK IN SHARE MODE` côté lien, route du plan
  rejouée (C-e341-40), test croisé, M8. **4.3** : refus serveur au règlement client (C-e341-41), M7b. **4.4** : 4-6-5 et 4-6-4 reconnus. **4.5** :
  `cardLast4` absent effacé au `PUT`, code `CREDIT_CARD_FIELD_NOT_ALLOWED`, `user-manual.tex:2369` nommée. Et C-e341-39 (`NewBankAccount.iban`).
- 2026-11-10 : **validation P3 (Sonnet)** — pour cette fiche : 2 M (4.1, 4.2), 3 L (4.3 à 4.5). Remédiation (Opus 5.5), tout appliqué. **4.1** :
  `iban: Option<String>` dans les deux corps (relus `routes/bank_accounts.rs:73-110`), IBAN lié en `Option` et `card_last4` écrit par
  `update_for_company` (relus `repositories/bank_accounts.rs:560-575`, `:600-612`, `:658-662`), `is_no_op_change`, test, M9, M9b ; S11 de la 34-11a1
  renvoyé ici. **4.2** : la suite candidate est définie par algorithme (séparateur unique, bornes = caractère avant le premier et après le dernier
  chiffre), cas de table. **4.3** : limites assumées écrites (date ou CVC à la suite, séparateurs autres, Diners 16) ; les espaces insécables comptent
  comme séparateurs. **4.4** : la route du plan reste `SansEcritureAuJournal`, le point (vi) passe à vingt-cinq (relus `audit_route_registry.rs:155-162`,
  `:421`). **4.5** : `0041791234564`, Luhn vrai (vérifié par calcul), remplace `0041791234567`.
- 2026-11-10 : **validation P4 ciblée (Sonnet)** — pour cette fiche : 3 M (F1, F3, F4), 3 L (F2, F5, F8). Remédiation (Opus 5.5), tout appliqué.
  **F1** : `is_no_op_change` ne sert que l'assistant (relu `repositories/bank_accounts.rs:252`, `:347`) ; la clause est retirée de la T4, le `PUT` n'a pas
  de no-op. *(L'entrée « validation P3 » ci-dessus cite encore `is_no_op_change` pour le `PUT` : relevé daté, faux à sa date, corrigé ici et non réécrit.)*
  **F3** : refus d'un compte `bank` sans IBAN, `POST` et `PUT`, asserté (AC-2, AC-4), M10 ; chaînes d'attendus des extracteurs (relues
  `routes/bank_accounts.rs:200-212`). **F4** : `cardLast4` sur un compte `bank` → `400 BANK_ACCOUNT_FIELD_NOT_ALLOWED`, M10b. **F2** : porteur de l'IBAN
  `Option` nommé (`ValidatedBankInput.iban`, `BankAccountPutFields.iban`, relus `:333-340`). **F5** : deux familles d'exemples acceptés, QRR Luhn-vraie
  fournie (calculée). **F8** : limite (4), numéro précédé de chiffres. F6 et F7 (34-11b, 34-2, closes) : laissés à leurs développeurs, transmis par
  l'orchestrateur.
- 2026-11-10 : **validation P5 ciblée (Opus)** — pour cette fiche : 2 M (F1, F2), 5 L (F3 à F7). Remédiation (Opus 5.5), tout appliqué. **F1** : M2
  nommée sur « 4111111111111111perso », M2b tuée par « 41 1111 1111 1111 11 » (ajouté aux acceptés, Luhn vrai calculé), M2d déclarée équivalente et retirée.
  **F2** : au `PUT`, le type courant se lit sur le pool avant `validate_bank_input`, `404` avant tout refus dépendant du type (C-e341-43 ; relu
  `routes/bank_accounts.rs:728-742`). **F3** : lignes de `ValidatedBankInput` corrigées (relu `:251-257`), constructions `:630`, `:808` et `INSERT` `:642`
  nommées. **F4** : huit littéraux de test nommés (relu `git grep -nE 'BankAccountPutFields \{'`), `Default` documenté, doc-comment
  d'`update_for_company`. **F5** : la référence QR proposée en P4 avait une clé fausse (modulo 10 récursif : 7 attendu, 6 écrit) ; remplacée par
  `210000000003139471430009313`, clé juste et Luhn vrai, calculés. **F6** : `cardLast4` vide ou `null` vaut absent. **F7** : l'assistant ignore
  `accountKind` et `cardLast4`, écrit à C-e341-19.
- 2026-11-12 : **développement** (Opus 5.5, clé e34a2, worktree `/home/gcorbaz/devel/kesh-e34a2`, base `27ca0bd9`). Étape 0, LOW de la validation
  P6 relus au code : **F1** — `find_by_id_for_company` ne filtre pas `archived` (relu `repositories/bank_accounts.rs:136-150`) : le `PUT` filtre
  `.filter(|b| !b.archived)` avant `BankAccountNotFound` (`routes/bank_accounts.rs`, `update_bank_account`) ; **F2** — doc-comment de l'ordre des
  erreurs du `PUT` récrit (C-e341-43) ; **F3** — huit littéraux de test plus celui d'`update_once` : les huit portent `iban: Some(…)`, celui
  d'`update_once` passe `input.iban` ; **F4** — négatif Visa 19 en 4-4-4-4-3 (`4111 1111 1111 1111 110`) et Amex en 16 chiffres
  (`3782246310005001`, Luhn vrai, accepté) ajoutés à la table. Choix C-e34a2-1 à 9 au registre `e34-choix-autonomes-e34a2.md`. **Fusion de la
  34-11a1 `done`** (`580675bb`) par `git merge --no-ff` : commit `86da9022`, sans conflit ; deux témoins transitoires de la 34-11a1
  (`compte_sans_iban_aucune_route_ne_le_cree`, `compte_sans_iban_le_put_ignore_le_type`, `bank_accounts_e2e.rs`), qui disaient « la 34-11a2
  changera ce contrat », récrits au contrat de la carte (`81b56bfa`) : seule modification de `bank_accounts_e2e.rs`, ses autres tests verts
  sans modification (AC-4). Écart à la consigne Edit : C-e34a2-9 (B3, espace insécable, script Python d'une ligne).
- 2026-11-12 : **revue de code P1 (Opus, trois lentilles)** — 0 CRITICAL, 0 HIGH, 3 MEDIUM distincts (B1 = E1, B2 = E2, A1), ~27 LOW
  (`kesh-gate-logs/e34a2-review-p1-{B,E,A}.md`). **Remédiation** (Opus 5.5, commit de code `b27c3ea1`, registre C-e34a2-10 à 15) :
  **B1 = E1, B8 = E6** — le `PATCH` lit le type du compte sur le pool avant le pré-vol (un id inconnu ou archivé garde la règle d'un compte
  bancaire, l'ordre d'avant) ; une carte vers un actif (débiteurs, non imputable), une charge, un produit ou délier rend
  `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY` ; ordre écrit au doc-comment et à la note ¹¹ ; tests étendus. **B2 = E2, E8, A12** — trois tests
  croisés par les **routes** (création, `PUT`, `PATCH`) pendant un retypage tenu, attente constatée par la sonde `PROCESSLIST` ; le test croisé
  du dépôt sonde au lieu d'attendre 400 ms ; M11, M11b, M4 et M8 tuées par eux. **A1** — paragraphe *Solde d'ouverture* retiré du manuel
  (commentaire LaTeX), note ¹¹ : l'assistant ne garde que le nom ; section **Raccords d'intégration** (quatre raccords, porteur
  l'orchestrateur). **E5** — plus de `maxlength` sur les quatre chiffres ; refus à l'écran ; `CARD_NUMBER_NOT_ALLOWED`
  (`details.field = "cardLast4"`) côté serveur ; tests. **B7 = E4** — IBAN valides masqués avant la garde (`mask_ibans`, test sur un IBAN
  suisse dont les chiffres font un Visa 19, M16). **E3** — faux négatifs écrits (module, manuel, note ¹¹), portée « séparateur unique, même
  différent ». **B3** — CHANGELOG : ordre `404` du `PUT`. **B4 = E9** — commentaire KF-004 annoté. **B5 = A3** — témoins renommés
  (`compte_sans_iban_carte_refusee_sans_lien_ou_avec_iban`, `compte_sans_iban_le_put_refuse_un_autre_type`). **A4** — deux commentaires
  périmés. **A5** — décompte précisé (voir le Dev Agent Record, annotation). **A6** — cases T1, T4, T5 annotées. **A7** — captures 05 (lien
  d'une carte) et 06 (règlement client sans la carte). **A8, B9** — Vitest du formulaire de lien (M13, M13b) et du retrait d'un actif (M14).
  **A9** — Vitest du `409` sur l'écran du plan, C-e34a2-14. **A10 = E10** — manuel : « nom ou description », formulaire *Modifier* seul.
  **A11** — « avant tout refus de forme qui dépend du type », `version < 1` d'abord. **B10** — union discriminée des payloads. **E7** — carte
  close comptée, carte et compte bancaire sur le même passif, refus avant `AccountHasEntries` (tests, M15). **A2** — PDF régénéré dans la
  branche, écart à X-1 déclaré (C-e34a2-15). Issues ouvertes par l'orchestrateur : B5 de la 34-11a1 → **#1032**, angle mort de la
  réconciliation → **#1033**.
- 2026-11-12 : **revue de code P2 (Sonnet, trois lentilles, passe complète)** — `kesh-gate-logs/e34a2-review-p2-{B,E,A}.md` : **0 au-dessus de
  LOW** une fois réfuté le seul MEDIUM. **B1 réfuté** (interblocage retypage / règlement client « non rejoué ») : le règlement client est
  rejoué, `crates/kesh-api/tests/audit_route_registry.rs:513` : `("post", "invoices::settle_invoice_handler", Traced, Rejouee),` — et le
  retypage l'est aussi (`accounts::update`, C-e341-40). **Renvoyés en issues** : faux négatif de `mask_ibans` (deux lettres de pays devant un
  numéro de carte) → **#1035** ; nom non borné avant la garde → **#1036** ; titre « Nouveau compte bancaire » sur le formulaire d'une carte →
  **#1037**. **Remédiation, tests et documents seulement, aucune ligne de production** (commit de tests `175d1ef5`) : E3 (`PUT` d'une carte
  avec un numéro complet dans le nom, la description, les quatre chiffres ; `cardLast4 = "1234 5678"` ; `PATCH` vers le passif courant, d'une
  carte archivée, d'une carte close ; carte vers un passif non imputable ou le compte créanciers désigné — phrase au manuel) ; E4 : mutation
  (c) jouée (M19, tuée), (a) et (b) écrites non tuées, ordre non témoigné, avec la raison (liste des mutations nommées) ; A8 (`version < 1`
  avant le `404` au `PUT`) ; B4 (code et message d'un compte bancaire sans `iban`) ; A1 (M11 à M19 nommées) ; A2 (AC-3, AC-7, AC-8 annotées) ;
  A3 (raccord (3) : manuel, PDF, CHANGELOG) ; A5 (renvoi `mut-p1/`) ; A6 (code du script E2E précisé) ; A7 (captures 01 à 06 reprises : toast
  fermé, la 06 avec un compte bancaire ordinaire proposé et la carte absente). A4 (raccords au fichier d'epic) : l'orchestrateur, à
  l'intégration. **Tendance de la boucle** : P1 (Opus) 0 CRITICAL, 0 HIGH, 3 MEDIUM, ~27 LOW → P2 (Sonnet) 0 au-dessus de LOW (un MEDIUM
  réfuté). **Validation terminée**, sur une passe sans finding au-dessus de LOW, et une remédiation qui ne touche aucune ligne de production.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (claude-opus-5-5), en autonomie, 2026-11-12.

### Completion Notes List

- **Livré** : garde `kesh_core::card_number::looks_like_card_number` (C-e341-15 (c)-(g), table de tests, deux familles d'acceptés dont la
  famille (i) asserte Luhn) ; corps `accountKind`, `cardLast4`, `iban` facultatif ; `validate_bank_input` par type ; lien obligatoire à un
  passif (pré-vol sur le pool, relecture sous `LOCK IN SHARE MODE` dans la transaction de la création, du `PUT` et du `PATCH`) ; `PUT` qui lit
  le type courant avant toute forme (`404` d'abord) ; `BankAccountPutFields.iban`/`card_last4` ; retypage refusé (`accounts::update`
  `FOR UPDATE` en tête, `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT`, route rejouée, point (vi) du registre à vingt-cinq) ; règlement client refusé
  (`400 SETTLEMENT_ACCOUNT_IS_CREDIT_CARD`) ; garde à l'assistant (nom) ; traces `account_kind`/`card_last4_present` ; sept codes, quatre
  locales ; écran (type à la création, champs d'une carte, liste réduite aux passifs, payload sans IBAN, type figé en édition), formulaire de
  lien et dialogue de règlement ; manuels utilisateur et administrateur, `api-external.md`, CHANGELOG ; points hérités E5/B7, B5, E6, E3, B3.
- **Non livré ici, renvoyé à l'intégration** (C-e34a2-1) : le sens *Dette* présélectionné du solde d'ouverture d'une carte (34-2 absente de
  la base ; le manuel l'annonce) ; le paramètre `liabilityOnly` de `linkableLedgerAccounts` (34-1 absente : `admitsLedgerType` en tient lieu) ;
  la garde sur la **description** de l'assistant (34-1) ; l'ordre « relecture du type avant le complément de la 34-2 ».
- **Rouges d'abord : écart** — tests écrits avec le code ; la preuve qu'ils mordent est la campagne de mutations (`kesh-gate-logs/e34a2/rouges.log`).
- **Mutations** (`kesh-gate-logs/e34a2/mut/M*.log`, `mut/campagne.log` — *annotation, revue P2 A5 : les journaux de cette passe, écrasés
  par la passe 2, sont copiés dans `mut-p1/`*) : **18 jouées, 18 tuées** — M1, M1b, M2, M2b, M2c, M2e, M3, M4, M5, M5b,
  M6, M7, M7b, M8, M9, M9b, M10, M10b ; aucune erreur de compilation dans les journaux. M8 tuée par
  `retypage_croise_avec_la_creation_d_une_carte`. M2d déclarée équivalente par la fiche, non jouée.
- **Gates au dernier commit de code `9a3904f5`** (journaux `kesh-gate-logs/e34a2/`, joués au commit `167c8c4e` = `9a3904f5` + le PDF) :
  `scripts/test-fast.sh --ci` 5186 exécutés, 5186 passés, 12 ignorés (`gate-test-fast-2.log`) ; `cargo build --workspace --all-targets` vert
  (`build-2.log`) ; `scripts/gate-ci.sh kesh_ci_e34a2` 5193 passés, 0 échoué, 16 ignorés (`gate-ci-2.log`) ; frontend `npm run check` 0 erreur
  (29 avertissements, aucun dans un fichier touché), `lint-i18n-ownership` vert, Vitest 2900 passés (224 fichiers), `npm run build` vert
  (`front-*-2.log`) ; `cargo fmt` et `cargo clippy --workspace --all-targets -D warnings` verts. **E2E complet** (base `kesh_e2e_e34a2` recréée,
  port 3372, `e2e-r2/playwright.log`) : 386 passés, 18 ignorés, **7 échecs, les sept de la KF-029 (#97)** nommés un par un contre
  `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) ;
  `bank-accounts-credit-card.spec.ts` vert. Aucun backend vivant après (`e2e-r2/ss-apres.txt` vide).
- **Gate du premier passage** (`gates-1.txt`, commit `81b56bfa`) : un rouge, `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres`
  (« 2030 \emph{Carte de crédit} » au manuel, compte absent des plans livrés ; avant lui, « 1234 ») : exemple remplacé par un sous-compte de
  2100 *Dettes bancaires à court terme* (manuel, quatre locales, repli Svelte), quatre chiffres d'exemple « 2041 ».
- **Captures** (thème clair, `kesh-gate-logs/e34a2/captures/`) : `34-11a2-01-creation-carte.png`, `-02-refus-numero-complet.png`, `-03-liste.png`,
  `-04-modification-carte.png`, prises après l'E2E r2 sur la base jetable.
- **Décomptes** (de `27ca0bd9` à `9a3904f5`, fusion comprise ; recomptés par `grep -c`) : tests Rust neufs 23 (`card_number.rs` 5,
  `credit_card_repository.rs` 5, `credit_card_e2e.rs` 12, `bank_account.rs` 1), deux récrits (`bank_accounts_e2e.rs`) ; Vitest neufs 10
  (`credit-card.test.ts` 3, `bank-accounts-page-credit-card.test.ts` 5, `SettleInvoiceDialog.test.ts` 1, `BankImportUpload.test.ts` 1) ; une
  spec Playwright neuve. Compteurs des gardes : `sitesTotal` 2924 → 2942, candidates 86 → 87, `conforme` 72 → 73, inventaire `bankName` de la page 3 → 5.
- **Issues demandées** (une par problème) : (1) B5 — `CHECK (is_primary = FALSE OR iban IS NOT NULL)` à poser à la prochaine migration de
  `bank_accounts` (l'invariant tient aujourd'hui par le type) ; (2) la proposition « facture » de la réconciliation accepte une transaction de
  carte comme règlement client (angle mort assumé de C-e341-41, si Guy veut le fermer). *(Ouvertes : #1032, #1033.)*
- *(Annotation du 2026-11-12, revue P1 A5 : le décompte « 23 tests Rust neufs » vaut **hors fusion**, `580675bb..9a3904f5` ; de `27ca0bd9` à
  `9a3904f5`, fusion comprise, il y en a 24 — un test de la 34-11a1 entré par la fusion, `admin_full_import_e2e.rs`.)*

#### Remédiation de la revue P1 (2026-11-12)

- **Dernier commit de code : `b27c3ea1`.** Gates joués au commit `b16920ae` (= `b27c3ea1` + registre et PDF), journaux `kesh-gate-logs/e34a2/` :
  `scripts/test-fast.sh --ci` 5194 exécutés, 5194 passés, 12 ignorés (`gate-test-fast-3.log`) ; `cargo build --workspace --all-targets` vert
  (`build-3.log`) ; `scripts/gate-ci.sh kesh_ci_e34a2` **tué par `mem-guard` au plafond** (code 137, `gate-ci-3.log`, sans échec de test au
  moment de l'arrêt), **rejoué** : 5201 passés, 0 échoué, 16 ignorés (`gate-ci-4.log`) ; frontend `npm run check` 0 erreur (29 avertissements,
  aucun dans un fichier touché), `lint-i18n-ownership` vert, Vitest 2905 passés (225 fichiers), `npm run build` vert (`front-*-3.log`) ;
  `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` verts.
- **E2E complet** (`e2e-r3/playwright.log`, base recréée, port 3372) : 384 passés, 18 ignorés, **9 échecs** — les sept de la KF-029 (#97),
  nommés contre `docs/testing.md`, et **deux variables** (`reports.spec.ts:422`, `sidebar-navigation.spec.ts:397`), sans rapport avec la
  story ; **rejoués seuls** sur base recréée (`e2e-r4/`) : 3 passés, 0 échec — pollution d'état entre specs, comme le décrit `docs/testing.md`.
  `bank-accounts-credit-card.spec.ts` vert. Aucun backend vivant après (`ss-apres.txt` vides).
- **Mutations, passe 2** (`mut/campagne-2.log`, `mut/M*.log` ; ceux de la passe 1 copiés dans `mut-p1/`) : **28 jouées, 28 tuées** — Rust :
  M1, M1b, M2, M2b, M2c, M2e, M3, M4, M5, M5b, M6, M7b, M8, M9, M9b, M10, M10b, M11, M11b, M12, M15, M16, M17 ; frontend : M7, M13, M13b, M14,
  M18. Le `FAIL` de chaque journal nomme le test attendu (M4 par `patch_croise_avec_un_retypage` : le pré-vol couvre désormais les cas
  séquentiels ; M8 par la sonde du test croisé du dépôt ; M11 par `creation_croisee_avec_un_retypage`). Aucune erreur de compilation.
- **Tests neufs de la remédiation** (de `3c1510d8` à `b27c3ea1`, recomptés par `grep -c`) : Rust 8 (`card_number.rs` 5 → 6,
  `credit_card_repository.rs` 5 → 6, `credit_card_e2e.rs` 12 → 18) ; Vitest 5 (`bank-accounts-page-credit-card.test.ts` 5 → 7,
  `accounts-page.test.ts` 13 → 14, `BankAccountJournalLinkForm-credit-card.test.ts` 2, neuf) ; `patch_d_une_carte` étendu (quatre cibles de plus).
- **Captures** : 01 à 04 reprises, 05 (formulaire de lien d'une carte : passifs seuls, pas de *Délier*) et 06 (dialogue de règlement d'une
  facture client : la carte n'est pas proposée ; la base E2E n'avait aucun autre compte bancaire, la liste est vide), prises après l'E2E r3.
- **Écarts** : le PDF régénéré dans la branche (C-e34a2-15) ; aucun nouvel écart à la consigne Edit.
- *(Annotation, revue P2 A6 : « `e2e rc=0` » de `gates-3.txt` est le code de sortie du **script** `e2e-e34a2.sh` ; celui de Playwright est
  `playwright rc=1` dans `e2e-r3/playwright.log`, pour les neuf échecs nommés ci-dessus.)*

#### Remédiation de la revue P2 (2026-11-12)

- **Aucune ligne de production touchée.** Commits : `175d1ef5` (tests, une phrase du manuel), puis documents et PDF.
- **Tests neufs** (de `eed314a9` à `175d1ef5`, recomptés) : `credit_card_e2e.rs` 18 → 22 (`put_d_une_carte_et_numero_complet`,
  `patch_d_une_carte_inchangee_archivee_ou_close`, `carte_vers_un_passif_non_imputable_ou_creanciers`, `put_version_zero_avant_le_404`) ;
  `compte_bancaire_sans_iban_ou_avec_quatre_chiffres` étendu (message).
- **Gates**, voie base : ciblé `credit_card_e2e`, `credit_card_repository`, `bank_accounts_e2e` 76 passés sur 76 (`cible-5.log`) ;
  `scripts/test-fast.sh --ci` 5198 passés sur 5198, 12 ignorés (`gate-test-fast-5.log`) ; `scripts/gate-ci.sh kesh_ci_e34a2` 5205 passés,
  0 échoué, 16 ignorés (`gate-ci-5.log`) ; `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` verts. **Mutation M19** jouée,
  tuée par `patch_d_une_carte_inchangee_archivee_ou_close` (`mut/M19.log`). **E2E complet non rejoué** : seuls des tests Rust et des documents
  ont changé depuis l'E2E complet de `b27c3ea1` ; la spec de la carte a tourné seule (`e2e-r5/`, 1 passée) pour reprendre les captures.
  Vitest non rejoué : aucun fichier frontend touché.

### File List

- `crates/kesh-core/src/card_number.rs` (neuf), `crates/kesh-core/src/lib.rs`
- `crates/kesh-db/src/entities/bank_account.rs`, `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/accounts.rs`,
  `crates/kesh-db/src/repositories/bank_accounts.rs`, `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/tests/bank_accounts_repository.rs`, `crates/kesh-db/tests/credit_card_repository.rs` (neuf)
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/accounts.rs`, `crates/kesh-api/src/routes/bank_accounts.rs`,
  `crates/kesh-api/src/routes/onboarding.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`, `crates/kesh-api/tests/bank_accounts_e2e.rs`, `crates/kesh-api/tests/credit_card_e2e.rs` (neuf)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/src/lib/features/bank-accounts/credit-card.ts` (neuf), `credit-card.test.ts` (neuf), `bank-accounts.api.ts`, `format.ts`,
  `BankAccountJournalLinkForm.svelte`, `BankAccountJournalLinkForm-credit-card.test.ts` (neuf), `bank-name-inventory.test.ts`
- `frontend/src/routes/(app)/accounts/accounts-page.test.ts` ; `crates/kesh-db/tests/payment_batches_repository.rs` (commentaire)
- `frontend/src/lib/features/bank-import/BankImportUpload.svelte`, `BankImportUpload.test.ts`
- `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte`, `SettleInvoiceDialog.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`, `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`, `bank-accounts-page-credit-card.test.ts` (neuf)
- `frontend/tests/e2e/bank-accounts-credit-card.spec.ts` (neuf)
- `docs/manual/fr/user-manual.tex` et `.pdf`, `docs/manual/fr/admin-manual.tex` et `.pdf`, `docs/api-external.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e34a2.md` (neuf), cette fiche, son prompt de revue P1, `sprint-status.yaml`
