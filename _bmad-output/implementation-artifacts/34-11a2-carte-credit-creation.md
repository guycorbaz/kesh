# Story 34-11a2 : créer et modifier une carte de crédit — type, quatre derniers chiffres, lien à un passif (#972)

Status: ready-for-dev

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
8. **AC-8 — codes d'erreur** : `CARD_NUMBER_NOT_ALLOWED`, `CREDIT_CARD_LEDGER_MUST_BE_LIABILITY`, `BANK_ACCOUNT_KIND_IMMUTABLE`,
   `ACCOUNT_LINKED_TO_BANK_ACCOUNT`, `CREDIT_CARD_FIELD_NOT_ALLOWED`, `BANK_ACCOUNT_FIELD_NOT_ALLOWED`, `SETTLEMENT_ACCOUNT_IS_CREDIT_CARD` : variantes `AppError`, `errors.rs`, messages aux quatre `messages.ftl`, libellés frontend, tableau d'erreurs
   de `docs/api-external.md`.
9. **AC-9 — manuel** : utilisateur, *Comptes bancaires* : le type carte, les quatre chiffres, le refus d'un numéro complet, le lien à un passif, ce
   que la carte ne fait pas (QR-facture, `pain.001`, CAMT), où elle est proposée ; la phrase `user-manual.tex:2369` (« l'IBAN reste affiché à la
   suite … Deux comptes peuvent porter la même description : l'IBAN les distingue ») récrite pour une carte (les quatre chiffres en tiennent lieu) ;
   *Plan comptable* : le retypage refusé d'un compte lié.
   `api-external.md` : corps de création et de `PUT`, refus. PDF régénéré, aplati.

## Tâches

- [ ] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : AC-1 à AC-6, Vitest AC-7.
- [ ] **T2 — `kesh-core`** : `looks_like_card_number(&str) -> bool` (bornes, groupements, longueur, préfixe, Luhn — C-e341-15), tests de table (AC-3).
- [ ] **T3 — routes** : corps (`accountKind`, `cardLast4`), et **`iban: Option<String>` avec `#[serde(default)]` dans `CreateBankAccountBody` et
  `UpdateBankAccountBody`** (`routes/bank_accounts.rs:79`, `:99`, `String` aujourd'hui ; validation P3, 4.1) — un compte **bancaire** sans IBAN rend
  `400` par `validate_bank_input`, comme en 0.17.5 ; `validate_bank_input` par type — **au `PUT`, le type courant est lu avant** (validation P5, F2) :
  lecture du compte sur le pool, filtrée par société (`find_by_id_for_company`), avant `validate_bank_input` et `validate_journal_account_id` (le type
  est immuable, la lecture hors transaction est sûre) ; un id inconnu, d'une autre société ou archivé rend `404` **avant** tout refus de forme qui
  dépend du type ; au `POST`, le type est celui du corps (absent = `bank`) —, garde de Luhn, contrôles C-e341-16 et C-e341-30 (`POST`, `PUT`,
  `PATCH`), `set_bank_account` (Luhn seulement), audit (AC-6).
- [ ] **T4 — dépôt** : `accounts::update` prend la ligne `accounts` `FOR UPDATE` en tête (C-e341-40 — elle ne l'est pas aujourd'hui), puis refuse le
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
- [ ] **T5 — écran** (AC-7), `linkableLedgerAccounts` à paramètre, sélecteurs (C-e341-32), i18n quatre locales, `lint-i18n-ownership`.
- [ ] **T6 — codes** (AC-8).
- [ ] **T7 — E2E** `bank-accounts-credit-card.spec.ts` : créer une carte liée à un passif créé par l'API, `•••• 1234` dans la liste et dans le
  sélecteur de l'import, CAMT absent pour elle, lot refusé, numéro complet refusé avec son motif ; modifier sa description.
- [ ] **T8 — manuel** (AC-9), captures `kesh-gate-logs/e341/captures/34-11a2-*.png` (création carte, refus d'un numéro complet, liste), CHANGELOG
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

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
