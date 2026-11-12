# Story 34-2 : le solde d'ouverture se saisit avec le compte bancaire (#533)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (aucune colonne : l'écriture est une écriture
ordinaire, son lien au compte bancaire n'est pas stocké). Registre : `e34-choix-autonomes-e341.md`, C-e341-6 à C-e341-12, C-e341-38. Suit la
**34-1** (même formulaire). Commits en `refs #533` ; à la PR : `closes #533`.*

## Story

En tant que Comptable qui ouvre un compte bancaire dans Kesh, je veux saisir son solde d'ouverture dans le formulaire du compte, dès qu'un
compte du plan y est lié, pour que le compte comptable porte son solde sans passer par *Soldes de départ*.

## Ce que #533 demande, et ce qui est tranché ici

#533 : « un champ **solde d'ouverture**, actif dès qu'un compte du plan comptable est lié. À l'enregistrement, Kesh passe **une écriture
d'ouverture pour ce seul compte**, contrepartie sur le compte *Bénéfice/perte reporté*, datée du premier jour du premier exercice — c'est-à-dire
**le mécanisme du mode Compléter** (mêmes conditions : compte jamais mouvementé, exercice ouvert, jour non verrouillé), réutilisé et non
dupliqué. » Quatre points y sont « à trancher » ; ils le sont ici (C-e341-6 à 9).

## Ce que le code dit (prémisses établies)

- **Le mécanisme existe, et il est entier** : `kesh_db::repositories::opening_complement` (Story 25-7). `create_opening_complement`
  (`opening_complement.rs:433-478`) ouvre **sa propre** transaction (`:457`) et appelle `create_in_open_tx` (`:503-626`, privée), qui :
  verrouille `companies` en partagé (`:513-519`), les comptes `FOR UPDATE` (`:530-548`), le compte de report en partagé (`:551-557`),
  les exercices (`:560-570`) ; lit « jamais mouvementé » et « la société a des écritures » (`:573-587`) ; refuse dans l'ordre de
  `OpeningComplementRefusal` (`:590-595`, `check_lines` `:629-670`) ; calcule la contrepartie sur le compte de rôle `RetainedEarnings`
  (`:597-620` : au crédit si les débits l'emportent, au débit sinon) ; valide puis `journal_entries::create_in_tx` (`:626`), qui numérote et
  trace `journal_entry.created`.
- **La date** : `decide_date` (`:224-258`) — (a) premier jour du premier exercice s'il est ouvert et hors période verrouillée ; (b) sinon
  aujourd'hui, dans l'exercice ouvert qui le couvre ; (c) sinon refus `NO_OPEN_FISCAL_YEAR` ou `DATE_LOCKED`.
- ⚠️ **Le refus qui fait tout l'enjeu : `NoEntries`** (`:65-66`, `:590-592`). Le complément exige une société qui **a déjà des écritures** :
  « il n'y a rien à compléter, la génération de l'ouverture s'applique ». Or le cas de Guy — installation neuve du 2026-11-10, premier compte
  bancaire — est **précisément** celui d'une société sans écriture. Réutilisé tel quel, le mécanisme refuserait le cas nominal (C-e341-6).
- **La génération générale** (`POST /api/v1/opening-balances`, `routes/opening_balances.rs:345-~480`) refuse une société qui a des écritures
  (`ALREADY_HAS_ENTRIES`, 409, `:194-213`). Une écriture d'ouverture passée par le formulaire bancaire fait donc basculer l'écran *Soldes de
  départ* en mode *Compléter* : c'est le comportement voulu, et le manuel doit le dire (AC-9).
- **La forme de la route du complément** : `parse_complement_amount` (`opening_balances.rs:509-527`) borne le montant (positif, quatre décimales,
  `DECIMAL(19,4)`), le libellé vient de la langue comptable (`:592-596`, clé `opening-balances-complement-description`), l'appel est rejoué sur
  interblocage (`:600`). Les refus deviennent `OPENING_COMPLEMENT_*` par `errors.rs:3414-3423` (`opening_complement_refusal_response`).
- **La création du compte bancaire** : `create_bank_account` (`routes/bank_accounts.rs:541-574`) contrôle hors transaction, puis
  `create_once` (`:580-700`) dans `retry_app_on_deadlock` : sentinelle `companies` (`:595`, `FOR UPDATE`), compte de créance, démotion du
  principal, `INSERT` (`:637-650`), relecture, traces. Elle rend `Json<BankAccount>` en `201`.
- **Le registre des routes** classe `bank_accounts::create_bank_account` en `Traced, SansEcritureAuJournal`
  (`tests/audit_route_registry.rs:540`) : après cette story elle **écrit au journal** ; elle est déjà rejouée, la classe change (T4).
- **L'écran** : formulaire de création `routes/(app)/bank-accounts/+page.svelte:520-592`, `submitCreate` `:294-317`, `formJournalAccountId`
  `:79` ; liste des comptes liables `:157-168` (après la 34-1 : `linkableLedgerAccounts`).
- **Le signe** : le complément reçoit débit **ou** crédit (`ComplementLine`, `opening_complement.rs:125-130`). Un compte bancaire est lié à un
  compte `Asset` ou `Liability` (`validate_journal_account_id`, `bank_accounts.rs:382-390`) ; un découvert, ou une carte de crédit (34-11a2),
  porte un solde créditeur.
- **Le manuel** : `user-manual.tex:951-990` (*Reprise de comptabilité — soldes de départ*, *Compléter un compte oublié* `:973-…`) et
  `:2355-2371` (*Comptes bancaires*).

## Choix (détail au registre)

- **C-e341-6 — société sans écriture** (gardé à la validation P1, 2.2, sur arbitrage de l'orchestrateur ; la conséquence est **dite à l'écran et
  au manuel**, AC-7 et AC-9) : le mécanisme reçoit une **politique** (`ComplementScope::SingleBankLedger`) qui lève le seul refus
  `NoEntries` **quand un premier exercice existe** ; sans exercice, refus `NO_OPEN_FISCAL_YEAR` (message existant). Le complément de l'écran
  *Soldes de départ* garde `ComplementScope::Screen`, inchangé. Écartés : refuser et renvoyer à *Soldes de départ* (le cas nominal échouerait) ;
  passer par la génération générale (deux mécanismes, contraire à « réutilisé, non dupliqué »).
- **C-e341-7 — le signe** : un montant positif et un sens, *Avoir* ou *Dette (découvert)*, présélectionné *Avoir* pour un compte lié `Asset`,
  *Dette* pour un `Liability`. *Avoir* → débit, *Dette* → crédit, quel que soit le type : c'est le solde lu sur le relevé. Écarté : un montant
  signé (le signe d'un relevé de carte se lit à l'envers).
- **C-e341-8 — compte déjà mouvementé** : le champ reste visible, et le refus `OPENING_COMPLEMENT_ACCOUNT_MOVED` du serveur s'affiche ; la
  création du compte bancaire est **annulée avec lui** (atomicité). L'écran ne devine pas « jamais mouvementé ».
- **C-e341-9 — atomicité** : à la création, compte bancaire et écriture dans **la même transaction** ; un refus de l'écriture n'écrit rien.
- **C-e341-10 — un compte existant** : action de ligne *Saisir le solde d'ouverture*, `POST /api/v1/bank-accounts/{id}/opening-balance`,
  même mécanisme ; elle couvre la « liaison » de #533 (un compte lié après coup, par *Modifier* ou *Lier*), sans greffer une écriture sur le
  `PUT` (remplacement complet, rejouable) ni sur le `PATCH`.
- **C-e341-11 — le libellé** : « Solde d'ouverture — {compte bancaire} » (nom d'affichage, `BankAccount::display_name`), clé neuve
  `bank-accounts-opening-entry-description`, dans la langue comptable.
- **C-e341-12 — pas à l'assistant** : aucun exercice n'existe à l'étape 6 (`onboarding.rs:930`, à la finalisation).

## Acceptance Criteria

1. **AC-1 — un seul mécanisme** : `opening_complement` expose une forme dans la transaction de l'appelant,
   `create_opening_complement_in_tx(tx, company_id, user_id, lines, owned, description, today, scope)`, que `create_opening_complement`
   réemploie (DRY : un seul `decide_date`, un seul `check_lines`, un seul calcul de contrepartie). L'enveloppe `create_opening_complement` **garde
   sa signature actuelle** et fixe `ComplementScope::Screen` en elle-même : ses appelants par position (`tests/opening_complement_repository.rs`,
   `opening_balances_e2e.rs`) ne changent pas. `complement_status` et l'écran *Soldes de
   départ* ne changent pas de comportement : les tests de `tests/opening_complement_repository.rs` et `tests/opening_balances_e2e.rs` passent
   **sans modification**.
2. **AC-2 — création avec solde** : `POST /api/v1/bank-accounts` accepte `openingBalance: { amount, side: "debit" | "credit" }`, facultatif ;
   absent, nul ou vide → comportement d'avant, octet pour octet (`openingEntry` porte
   `#[serde(skip_serializing_if = "Option::is_none")]` : absent de la réponse quand rien n'est passé). Présent : exige `journalAccountId` (`400 VALIDATION_ERROR` sinon, avant toute
   écriture) ; contrôles de forme de `parse_complement_amount` (réemployé, non recopié). La réponse `201` est le compte, plus
   `openingEntry: { id, entryNumber, entryDate, fiscalYearId }` quand une écriture est passée (champ additif, écrit à `api-external.md`).
3. **AC-3 — l'écriture** : journal `OD`, libellé « Solde d'ouverture — UBS — Personnel » (C-e341-11), deux lignes : le compte lié au sens saisi,
   le compte de report à l'opposé, même montant ; date et exercice de `decide_date` ; `journal_entry.created` tracé ; `bank_account.created`
   porte `opening_entry_id`.
4. **AC-4 — société sans écriture** (le cas nominal) : premier exercice ouvert, aucune écriture → l'écriture est passée, datée du premier jour
   du premier exercice ; l'écran *Soldes de départ* rend ensuite `ALREADY_HAS_ENTRIES` pour la génération et propose le mode *Compléter* (le
   compte bancaire n'y figure plus : mouvementé). Sans aucun exercice → `409 OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR` (statut du code, `errors.rs:1557-1561`), rien d'écrit ; son
   message (« Aucun exercice ouvert ne couvre la date du jour ») est gardé : inexact au mot près pour « aucun exercice », il dit le remède (un
   exercice ouvert), et une installation passée par l'assistant a toujours son premier exercice.
5. **AC-5 — mêmes refus que *Compléter*, rien d'écrit** : compte lié déjà mouvementé (`ACCOUNT_MOVED`), compte de report absent ou non imputable,
   date verrouillée, aucun exercice ouvert : le refus du complément sort **tel quel** (code et message), et **ni le compte bancaire ni
   l'écriture** n'existent après (AC-9 de l'atomicité, compté en base : `bank_accounts` et `journal_entries` inchangés). L'ancien principal
   n'est pas démoté.
6. **AC-6 — compte existant** : `POST /api/v1/bank-accounts/{id}/opening-balance` `{ amount, side }`, Comptable+, clé `read-write` admise :
   `404` compte absent, d'une autre société ou archivé ; `409 BANK_ACCOUNT_CLOSED` clos ; `412 BANK_ACCOUNT_NOT_CONFIGURED` sans compte lié
   (code existant) ; puis les refus du complément. `201` avec l'écriture. Rejouée sur interblocage. **Ordre des verrous** (validation P2, 2.5),
   celui du `PUT` : sentinelle `companies` en exclusif (`acquire_company_sentinel_lock`), puis la ligne `bank_accounts` `LOCK IN SHARE MODE`
   (archivé, clos, lié lus sous ce verrou), puis le complément (son partagé sur `companies` est déjà couvert par l'exclusif ; comptes, exercices).
7. **AC-7 — l'écran** : au formulaire de création, quand un compte lié est choisi, *Solde d'ouverture* (montant) et *Avoir / Dette* (C-e341-7),
   avec une aide qui dit la date (« daté du premier jour du premier exercice s'il est ouvert, sinon d'aujourd'hui ; contrepartie : Bénéfice/perte
   reporté ») **et la conséquence de C-e341-6** (« Après ce solde, les soldes de départ des autres comptes se saisissent un à un dans *Soldes de
   départ*, mode *Compléter* : l'ouverture générale n'est plus proposée ») ; sans compte lié, les champs sont masqués et rien n'est envoyé. Succès : un avis nomme l'écriture (numéro, date). Refus : le motif
   du serveur dans `form-error`. Action de ligne *Saisir le solde d'ouverture* sur un compte ouvert, lié — même dialogue.
8. **AC-8 — garde du registre** : `create_bank_account` passe `Traced, Rejouee` ; la route neuve y est inscrite `Traced, Rejouee`
   (`audit_route_registry.rs`), et les totaux du registre suivent (`assert_eq!(traced + exempt + no_matter, LIB_ROUTES.len())`, `:988`, et le
   compte des routes tracées codé en dur du même fichier, recomptés) ; la garde qui exige l'appel de l'enveloppe de rejeu dans le corps du handler
   est satisfaite par les deux (`create_bank_account` l'appelle déjà, `bank_accounts.rs:563`).
9. **AC-9 — manuel** : *Comptes bancaires* (le champ, le sens, la date, l'atomicité, l'action de ligne) et *Soldes de départ* (une écriture
   passée par un compte bancaire fait passer l'écran en mode *Compléter* ; le compte de la banque n'y figure plus) ; la phrase de l'AC-7, **avant**
   le geste, dans les deux sections : qui veut saisir toute son ouverture d'un coup passe d'abord par *Soldes de départ*. PDF régénéré, contrôlé
   aplati.
10. **AC-10 — l'instantané** (validation P1, 2.1 ; **récrit à la validation P2**, 2.1 et 2.2, C-e341-38) : dans `create_once`, le complément est
   appelé **juste après le refus du compte de créance, avant la démotion de l'ancien principal et avant l'`INSERT`**. Raison : la démotion
   (`flip_primary_off_for_company`) relit l'ancien principal par `FIND_BY_ID_SQL`, **sans verrou** (`repositories/bank_accounts.rs:1408-1412`, `:58`) —
   une lecture ordinaire qui ouvrirait l'instantané avant le complément, et `create_in_open_tx` lit « jamais mouvementé » après ses verrous
   (`opening_complement.rs:573-587`) : un instantané ouvert plus tôt ne verrait pas une écriture commitée entre-temps. Avant le complément ne restent
   que des lectures **verrouillantes** : sentinelle `companies` `FOR UPDATE` (`:1617`), réglages `LOCK IN SHARE MODE` ; le numéro lu par
   `refuse_if_ledger_is_claim_account` ne l'est que sur le chemin du refus. Le complément n'a pas besoin de la ligne `bank_accounts` ; son libellé se
   forme de la banque et de la description saisies (`bank_account_display_name`). Un refus du complément ne démote rien et n'insère rien (AC-5).
   **Concurrence** : deux créations de la même société sont **sérialisées** par la sentinelle (`routes/bank_accounts.rs:595`) — aucun interblocage
   entre elles. Le cas à prouver est une **écriture de journal** (`POST /api/v1/journal-entries` sur le compte lié, qui ne prend pas la sentinelle)
   commitée pendant la création : la création la voit et rend `OPENING_COMPLEMENT_ACCOUNT_MOVED`, sans compte créé. Montage : le compte du plan est
   créé d'abord (jamais mouvementé), puis les deux requêtes se croisent ; test à deux connexions si un point de synchronisation est atteignable sans
   crochet de production, sinon le raisonnement écrit au doc-comment de `create_once` et dit tel quel au Dev Agent Record.

## Tâches

- [x] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : `crates/kesh-db/tests/opening_complement_repository.rs` (portée
  `SingleBankLedger` sur société vide : passe ; portée `Screen` : `NoEntries` inchangé) ; `crates/kesh-api/tests/bank_accounts_e2e.rs` ou fichier
  neuf `bank_account_opening_balance_e2e.rs` : AC-2 à AC-6, dont l'atomicité comptée (AC-5) et un crédit sur un compte `Liability`.
- [x] **T2 — dépôt** : `create_opening_complement_in_tx` + `ComplementScope` ; `create_opening_complement` devient une enveloppe de même signature (lecture
  `owned_account_ids` hors transaction **inchangée** pour l'écran). **Deux branches distinctes** pour `SingleBankLedger` (validation P1, 2.3 : le code
  actuel les confond dans un seul `filter`, `opening_complement.rs:590-592`) : `first = None` → `NoOpenFiscalYear` ; premier exercice présent →
  le test « la société a des écritures » est sauté. Pour le compte bancaire, `owned` est le seul compte lié, déjà contrôlé de la
  société par `validate_journal_account_id` (aucun verrou sur un compte d'autrui : la raison d'être d'`owned`, `:441-452`, reste tenue).
  ⚠️ **Deux ordres de verrous, écrits aux doc-comments** (validation P3, 2.1 — l'ordre écrit avant la P2 ne vaut plus) : dans `create_once`,
  `companies` (X, sentinelle) → réglages (S) → contrôle du type du compte lié (S, 34-11a2) → comptes et exercices (complément) → `bank_accounts`
  (démotion, `INSERT`) ; dans la route de ligne (AC-6), `companies` (X) → `bank_accounts` (S) → comptes et exercices. La sentinelle sérialise les
  deux gestes entre eux ; l'inversion `bank_accounts` / comptes ne joue que contre un geste sans sentinelle (un rapprochement, qui tient
  `bank_accounts` en partagé puis écrit), et les deux routes sont rejouées sur interblocage. **Le contrôle de type précède le complément**
  (validation P3, 2.3) : un refus de type ne passe aucune écriture ; la promotion S → X de la ligne `accounts` par le `FOR UPDATE` du complément peut
  interbloquer contre un geste qui tient S (une écriture de journal par sa clé étrangère) : rejouée, sans perte.
  ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [x] **T3 — routes** : corps `openingBalance` (forme partagée avec la route neuve), réponse augmentée (`#[serde(flatten)]` du compte),
  `create_once` appelle le complément **après le refus du compte de créance, avant la démotion et l'`INSERT`** (AC-10), puis démotion, `INSERT`,
  relecture, traces ; `parse_complement_amount`
  (`opening_balances.rs:509`, privée) passe `pub(crate)` ; route neuve dans `comptable_routes` (`lib.rs:~936-955`), bornée par
  `retry_app_on_deadlock`. Mapping des refus : celui d'`errors.rs:3414` (aucun nouveau code).
- [x] **T4 — registre** (AC-8).
- [x] **T5 — écran** : `bank-accounts.api.ts` (`createBankAccount` rend `{ …, openingEntry }`, `enterOpeningBalance(id, …)`), formulaire et
  action de ligne ; Vitest : champs masqués sans lien, présélection du sens par type, corps envoyé, avis de succès, refus affiché.
- [x] **T6 — i18n** (quatre locales) : libellé de l'écriture, champs, sens, aide, avis, action. `lint-i18n-ownership`.
- [x] **T7 — E2E** : spec neuve `bank-account-opening-balance.spec.ts` : créer un compte lié à un compte neuf (créé par l'API, jamais mouvementé)
  avec 1'234.50 *Avoir* → la liste montre le solde `1'234.50` dans la colonne *Solde* (la valeur traverse HTTP et le calcul de
  `list_by_company_with_balances`) ; second essai sur un compte mouvementé → motif affiché, aucun compte créé.
- [x] **T8 — manuel** (AC-9), captures (formulaire avec solde, avis, refus) dans `kesh-gate-logs/e341/captures/34-2-*.png`.
- [x] **T9 — CHANGELOG** : *Ajouté*, `refs #533`.

### Mutations nommées

- **M1** : `ComplementScope` ignoré (NoEntries toujours levé) → rouge « portée `Screen` : `NoEntries` inchangé ».
- **M2** : l'appel du complément déplacé **après** le `commit` de `create_once` → rouge AC-5 (compte bancaire présent après refus).
- **M3** : *Avoir* envoyé en crédit → rouge AC-3.
- **M4** : la contrepartie posée sur le compte lié lui-même (écriture nulle) → rouge AC-3 (deux comptes distincts).
- **M5** : `openingBalance` accepté sans `journalAccountId` → rouge AC-2.
- **M6** : la route de ligne sans contrôle « clos » → rouge AC-6.
- **M7** : `SingleBankLedger` qui ne distingue pas « aucun exercice » de « aucune écriture » → rouge AC-4 (sans exercice : refus attendu).
- **M8** : le complément replacé après la démotion et l'`INSERT` (une lecture ordinaire avant lui) → rouge du test « écriture de journal croisée »
  (AC-10) s'il est déterministe ; sinon écrite non tuée au Dev Agent Record. ⚠️ **Montage obligatoire** (validation P3, 2.2) : une création
  `isPrimary: true` **au-dessus d'un principal existant** — la relecture de la démotion (`repositories/bank_accounts.rs:1408-1412`) est la seule lecture
  ordinaire qui précède l'`INSERT`, et elle n'a lieu que dans ce cas ; hors de lui, M8 est un mutant équivalent. ⚠️ Un test de deux **créations** concurrentes ne la tue pas (la
  sentinelle les sérialise : test muet, validation P2, 2.2).
- **M9** : un refus du complément qui démote quand même l'ancien principal → rouge AC-5.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db` ; gate frontend complet ; E2E complet au dernier commit de code.

## Manuels

`user-manual.tex` (AC-9) ; `docs/api-external.md` (corps et réponse de `POST /bank-accounts`, route neuve). **Prompt de revue : nommer le manuel
et `api-external.md`**, PDF aplati.

## Ce qui ne se traite pas

- **Modifier une écriture d'ouverture** déjà passée : #532 (ouverte), inchangé ; le manuel renvoie à la contre-passation.
- **Un solde d'ouverture en devise** : les comptes bancaires de Kesh sont en CHF ; rien à faire.
- **Le compte de report choisi** : c'est le compte de rôle `RetainedEarnings`, comme *Compléter* ; aucun choix offert.

## Recoupements

`routes/(app)/bank-accounts/+page.svelte`, `bank-accounts.api.ts`, `routes/bank_accounts.rs` avec **34-1** (avant) et **34-11a2** (après : une
carte se crée avec son solde d'ouverture, sens *Dette*). `opening_complement.rs` : aucune autre story de l'epic.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet)** — pour cette fiche : 2 M (2.1, 2.2), 3 L (2.3 à 2.5). Remédiation (Opus 5.5), tout appliqué.
  **2.1** : AC-10 neuf (le complément avant toute lecture ordinaire ; interblocage par la clé étrangère, le perdant reçoit `ACCOUNT_MOVED`), T3, M8
  (relus `bank_accounts.rs:595-653`, `opening_complement.rs:503-587`). **2.2** : C-e341-6 gardé (arbitrage de l'orchestrateur), la phrase de la
  conséquence exigée à l'écran (AC-7) et au manuel (AC-9). **2.3** : deux branches en T2, M7 (relu `opening_complement.rs:590-592`). **2.4** : totaux
  du registre (relu `audit_route_registry.rs:988`), `parse_complement_amount` `pub(crate)`, garde de l'enveloppe de rejeu. **2.5** : l'enveloppe
  garde sa signature (AC-1). Références à « 34-11a » portées à 34-11a2.
- 2026-11-10 : **validation P2 (Opus)** — pour cette fiche : 2 M (2.1, 2.2), 3 L (2.3 à 2.5), nés de la remédiation P1. Remédiation (Opus 5.5),
  tout appliqué. **2.1** : prémisse de l'AC-10 fausse (la démotion relit sans verrou, relu `repositories/bank_accounts.rs:1404-1414`, `:58`) : le
  complément passe avant la démotion et l'`INSERT` (C-e341-38), AC-10 et T3 récrits, M9. **2.2** : la sentinelle sérialise deux créations (relu
  `bank_accounts.rs:1617`, `routes/bank_accounts.rs:595`) : phrase de l'interblocage retirée, le test croise une écriture de journal, M8 récrite.
  **2.3** : 409 et non 400 (relu `errors.rs:1557-1561`), message gardé (motif écrit). **2.4** : `skip_serializing_if`. **2.5** : ordre des verrous de la
  route de ligne. Au passage : « PDF régénéré, contrôlé aplati », déplacé par la remédiation P1 à la fin de l'AC-10, rendu à l'AC-9.
- 2026-11-10 : **validation P3 (Sonnet)** — pour cette fiche : 3 L (2.1 à 2.3). Remédiation (Opus 5.5), tout appliqué. **2.1** : la T2 dit les deux
  ordres de verrous d'après la P2 et l'inversion couverte par le rejeu. **2.2** : le montage de M8 exige une création principale au-dessus d'un
  principal existant (relu `repositories/bank_accounts.rs:1404-1414`). **2.3** : le contrôle de type de la 34-11a2 précède le complément, promotion
  S → X rejouée (T2).
- 2026-11-12 : **développement** (Opus 5.5, `bmad-dev-story`, worktree `kesh-e342`, base `133debc2`, clé de travail e342 — port 3342,
  bases `kesh_e342`, `kesh_e2e_e342`, `kesh_ci_e342`, journaux `kesh-gate-logs/e342/` au lieu des `e341/` de la fiche ; registre
  `e34-choix-autonomes-e341.md`, celui que la fiche cite, C-e341-44 à 50 ajoutés). **Étape 0** : LOW F7 de la validation P4
  (`e34-validate-p4-e341.md`, l'inversion de verrous joue aussi contre le retypage) traité au doc-comment de `create_once`, ligne lue
  `routes/bank_accounts.rs:819` (« … le retypage d'un compte du plan ») ; « test à deux connexions si un point de synchronisation est
  atteignable » (AC-10) établi : **déterministe**, par `attendre_une_requete_en_cours` sur le `FOR UPDATE` du complément, la ligne croisée
  étant ajoutée à une écriture **existante** (une création d'écriture prend le partagé de clé étrangère de `companies` et serait sérialisée
  par la sentinelle : montage muet) ; M8 tuée. Le contrôle de type de la 34-11a2 (T2) n'existe pas encore : il viendra avant l'appel du
  complément. **Écarts à la fiche** : (1) AC-4 « l'écran *Soldes de départ* rend `ALREADY_HAS_ENTRIES` pour la génération » — le
  `POST /opening-balances` rend `409 ILLEGAL_STATE_TRANSITION`, et c'est `GET /opening-balances/status` qui rend
  `reason: ALREADY_HAS_ENTRIES` (lu `routes/opening_balances.rs:185-196`) ; le test et `api-external.md` disent les deux ; (2) M2 (« le complément
  après le commit ») jouée sous une forme équivalente : refus avalé (`.ok()`), le compte se crée sans écriture ; (3) M9 jouée « commit même sur
  refus » : **équivalente** — le complément précède la démotion, un refus sort avant toute écriture (`mut/resume.txt`) ; (4) la création ne
  trace pas d'action `bank_account.*` à la route de ligne (C-e341-46) ; (5) le libellé du champ change au dialogue de ligne (C-e341-50).
  **Consigne Edit** : deux blocs de tests ajoutés en fin de fichier par `cat >>` (`bank-accounts.api.test.ts`, `format.test.ts`) au lieu de
  l'outil Edit — par commodité, l'outil ne manquait de rien ; `cargo fmt --all` appliqué (outil de formatage, non un remplacement de texte) ;
  mutations par le script `mut-run-e342.sh` (remplacements vérifiés uniques, permis). Tout le reste du code par l'outil Edit.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code), en autonomie, 2026-11-11 → 2026-11-12.

### Completion Notes List

- **Dépôt** (`opening_complement.rs`) : `ComplementScope { Screen, SingleBankLedger }` ; `create_opening_complement_in_tx` est la seule
  implémentation (l'ancienne `create_in_open_tx` privée, renommée et rendue publique) ; `create_opening_complement` garde sa signature et passe
  `Screen`. `SingleBankLedger` : sans exercice → `NoOpenFiscalYear` ; avec un premier exercice, « la société a des écritures » n'est pas lue.
  `repositories/bank_accounts.rs` : `find_by_id_locked_shared_in_tx` (lecture `LOCK IN SHARE MODE`). `opening_complement_repository.rs` et
  `opening_balances_e2e.rs` **inchangés** (`git diff --stat 133debc2 HEAD` vide sur eux).
- **Routes** : `POST /bank-accounts` accepte `openingBalance { amount, side }` (C-e341-45) et rend `openingEntry` (`CreatedBankAccountResponse`,
  `#[serde(flatten)]`, `skip_serializing_if`) ; le complément est appelé dans `create_once` après le refus du compte de créance, **avant** la
  démotion et l'`INSERT` (AC-10) ; `bank_account.created` porte `opening_entry_id` quand une écriture est passée, la trace d'avant sinon.
  `POST /bank-accounts/{id}/opening-balance` (`enter_opening_balance`, rejouée `bank_accounts::opening_balance`) : sentinelle →
  ligne en partagé → 404 / 409 clos / 412 sans lien → complément. `parse_complement_amount` et `invalid_amount` passent `pub(crate)`.
  Libellé « Solde d'ouverture — {nom} » dans la langue comptable, marques BiDi retirées.
- **Registre** (`audit_route_registry.rs`) : création `Traced, Rejouee` ; route neuve `Traced, Rejouee` ; 145 → 146 routes de `lib.rs`,
  138 → 139 tracées, 148 → 149 au registre, 25 → 27 rejouées, 120 → 119 sans écriture, point (vi) à vingt-trois. Manuel d'administration :
  « 139 des 146 routes », « 139 + 4 + 3 = 146 ».
- **Écran** : champs du solde (montant, sens présélectionné par le type du compte lié, aide, conséquence de C-e341-6 **avant** le geste) dans
  le formulaire de création dès qu'un compte est lié ; action de ligne *Saisir le solde d'ouverture* sur un compte ouvert et lié, même snippet ;
  avis qui nomme l'écriture ; refus dans le formulaire ou le dialogue. Garde `i18n-keys.test.ts` : `sitesTotal` 2928 → 2942 (+14, recompté
  par la garde).
- **i18n** : 15 clés × 4 locales (`bank-accounts-opening-*`, `bank-accounts-toast-*-opening*`, `bank-accounts-actions-opening-balance`,
  `error-bank-account-opening-invalid-amount`). Traductions de/it/en proposées par l'agent.
- **Manuels** : `user-manual.tex` (§ *Comptes bancaires* : puce, § *Le solde d'ouverture d'un compte* ; § *Soldes de départ* : avertissement
  avant l'énumération), `admin-manual.tex` (compte du registre), PDF FR régénérés (`make fr`, brochure rendue à son état), contrôlés aplatis :
  « Le solde d’ouverture d’un compte » 2 (titre et table), « Saisir le solde d’ouverture » 1, « Saisissez l’ouverture ici avant tout solde
  d’ouverture de compte bancaire » 1, « 139 des 146 routes » et « 139 + 4 + 3 = 146 » présents. `docs/api-external.md` : note ¹¹, tableau,
  paragraphe des interblocages. CHANGELOG `[0.17.6]` *Ajouté*, `refs #533`.
- **Tests neufs** (périmètre `133debc2..HEAD`, recomptés) : 5 `#[sqlx::test]` kesh-db (`opening_complement_bank_scope.rs`), 12 kesh-api
  (`bank_account_opening_balance_e2e.rs`), 15 Vitest (page 10, API 3, format 2), 2 E2E Playwright (`bank-account-opening-balance.spec.ts`).
- **Rouges d'abord** (`kesh-gate-logs/e342/rouges.log`) : Vitest 8 rouges / 2 verts attendus sur l'écran d'avant ; HTTP 11 rouges / 1 vert
  attendu sur le code de production d'avant (`git stash` des cinq fichiers) ; kesh-db : rouge de compilation.
- **Mutations** (`mut/M*.log`, `mut/resume.txt`) : M1 à M8 tuées ; M9 équivalente (expliquée). M8 tuée par
  `ecriture_croisee_pendant_la_creation_est_vue`, déterministe.
- **Gates au dernier commit de code `8db6fb7b`** (HEAD `76bcdf53`, PDF seuls en plus) : base `kesh_e342` recréée ;
  `scripts/test-fast.sh --ci` **5169 passés, 12 ignorés**, rc 0 ; `cargo build --workspace --all-targets` rc 0 ; `scripts/gate-ci.sh kesh_ci_e342`
  **5176 passés, 0 échoué, 16 ignorés**, rc 0 ; `cargo fmt --check` et `cargo clippy -p kesh-db -p kesh-api --all-targets -D warnings` verts
  (le clippy workspace tourne dans `test-fast.sh`). Frontend : `npm run check` 0 erreur (29 avertissements, aucun dans les fichiers de la
  story), `lint-i18n-ownership` PASS, `test:unit` **2911 passés (224 fichiers)**, `npm run build` rc 0.
- **E2E complet** sur `kesh_e2e_e342` recréée vide (`e2e-r1.log`) : **384 passés, 11 échoués, 18 ignorés**. Sept échecs de la KF-029 (#97,
  liste de `docs/testing.md`) ; **quatre hors liste**, tous en dépassement de 30 s dans le `beforeEach` qui appelle `seedTestState` :
  `fiscal-year-closing-journey.spec.ts:169`, `:320`, `fiscal-years.spec.ts:47`, `:53` — plus que les « 0 à 2 » de pollution que prévoit
  `docs/testing.md`. Rejoués seuls avec la spec neuve (`e2e-r2.log`) : **13 passés**. Station chargée (autres agents en gate) ; pas de
  régression établie, mais le dépassement de la borne est à noter. Les deux tests neufs passent dans les deux runs.
- **Captures** (thème clair, `kesh-gate-logs/e342/captures/`) : `34-2-01-formulaire-avec-solde.png`, `-02-avis-de-succes-et-solde.png`,
  `-03-refus-compte-mouvemente.png`, `-04-dialogue-de-ligne-dette.png`, `-05-dialogue-de-ligne-succes.png`.
- **Aucune migration.** #1002 à #1006 (étape bancaire de l'assistant) non touchées : la story ne passe pas par l'assistant (C-e341-12).

### File List

- `crates/kesh-db/src/repositories/opening_complement.rs`, `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/tests/opening_complement_bank_scope.rs` (neuf)
- `crates/kesh-api/src/routes/bank_accounts.rs`, `crates/kesh-api/src/routes/opening_balances.rs`, `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/tests/bank_account_opening_balance_e2e.rs` (neuf), `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`, `format.ts`, `bank-accounts.api.test.ts`, `format.test.ts`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`, `bank-accounts-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/bank-account-opening-balance.spec.ts` (neuf)
- `docs/manual/fr/user-manual.tex`, `user-manual.pdf`, `admin-manual.tex`, `admin-manual.pdf`, `docs/api-external.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e341.md`, `34-2-solde-ouverture-compte-bancaire-review-prompt-p1.md` (neuf),
  la fiche, `sprint-status.yaml`
