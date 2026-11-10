# Story 34-2 : le solde d'ouverture se saisit avec le compte bancaire (#533)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (aucune colonne : l'écriture est une écriture
ordinaire, son lien au compte bancaire n'est pas stocké). Registre : `e34-choix-autonomes-e341.md`, C-e341-6 à C-e341-12. Suit la
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
  compte `Asset` ou `Liability` (`validate_journal_account_id`, `bank_accounts.rs:382-390`) ; un découvert, ou une carte de crédit (34-11a),
  porte un solde créditeur.
- **Le manuel** : `user-manual.tex:951-990` (*Reprise de comptabilité — soldes de départ*, *Compléter un compte oublié* `:973-…`) et
  `:2355-2371` (*Comptes bancaires*).

## Choix (détail au registre)

- **C-e341-6 — société sans écriture** : le mécanisme reçoit une **politique** (`ComplementScope::SingleBankLedger`) qui lève le seul refus
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
   réemploie (DRY : un seul `decide_date`, un seul `check_lines`, un seul calcul de contrepartie). `complement_status` et l'écran *Soldes de
   départ* ne changent pas de comportement : les tests de `tests/opening_complement_repository.rs` et `tests/opening_balances_e2e.rs` passent
   **sans modification**.
2. **AC-2 — création avec solde** : `POST /api/v1/bank-accounts` accepte `openingBalance: { amount, side: "debit" | "credit" }`, facultatif ;
   absent, nul ou vide → comportement d'avant, octet pour octet. Présent : exige `journalAccountId` (`400 VALIDATION_ERROR` sinon, avant toute
   écriture) ; contrôles de forme de `parse_complement_amount` (réemployé, non recopié). La réponse `201` est le compte, plus
   `openingEntry: { id, entryNumber, entryDate, fiscalYearId } | null`.
3. **AC-3 — l'écriture** : journal `OD`, libellé « Solde d'ouverture — UBS — Personnel » (C-e341-11), deux lignes : le compte lié au sens saisi,
   le compte de report à l'opposé, même montant ; date et exercice de `decide_date` ; `journal_entry.created` tracé ; `bank_account.created`
   porte `opening_entry_id`.
4. **AC-4 — société sans écriture** (le cas nominal) : premier exercice ouvert, aucune écriture → l'écriture est passée, datée du premier jour
   du premier exercice ; l'écran *Soldes de départ* rend ensuite `ALREADY_HAS_ENTRIES` pour la génération et propose le mode *Compléter* (le
   compte bancaire n'y figure plus : mouvementé). Sans aucun exercice → `400 OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR`, rien d'écrit.
5. **AC-5 — mêmes refus que *Compléter*, rien d'écrit** : compte lié déjà mouvementé (`ACCOUNT_MOVED`), compte de report absent ou non imputable,
   date verrouillée, aucun exercice ouvert : le refus du complément sort **tel quel** (code et message), et **ni le compte bancaire ni
   l'écriture** n'existent après (AC-9 de l'atomicité, compté en base : `bank_accounts` et `journal_entries` inchangés). L'ancien principal
   n'est pas démoté.
6. **AC-6 — compte existant** : `POST /api/v1/bank-accounts/{id}/opening-balance` `{ amount, side }`, Comptable+, clé `read-write` admise :
   `404` compte absent, d'une autre société ou archivé ; `409 BANK_ACCOUNT_CLOSED` clos ; `412 BANK_ACCOUNT_NOT_CONFIGURED` sans compte lié
   (code existant) ; puis les refus du complément. `201` avec l'écriture. Rejouée sur interblocage.
7. **AC-7 — l'écran** : au formulaire de création, quand un compte lié est choisi, *Solde d'ouverture* (montant) et *Avoir / Dette* (C-e341-7),
   avec une aide qui dit la date (« daté du premier jour du premier exercice s'il est ouvert, sinon d'aujourd'hui ; contrepartie : Bénéfice/perte
   reporté ») ; sans compte lié, les champs sont masqués et rien n'est envoyé. Succès : un avis nomme l'écriture (numéro, date). Refus : le motif
   du serveur dans `form-error`. Action de ligne *Saisir le solde d'ouverture* sur un compte ouvert, lié — même dialogue.
8. **AC-8 — garde du registre** : `create_bank_account` passe `Traced, Rejouee` ; la route neuve y est inscrite `Traced, Rejouee`
   (`audit_route_registry.rs`).
9. **AC-9 — manuel** : *Comptes bancaires* (le champ, le sens, la date, l'atomicité, l'action de ligne) et *Soldes de départ* (une écriture
   passée par un compte bancaire fait passer l'écran en mode *Compléter* ; le compte de la banque n'y figure plus). PDF régénéré, contrôlé aplati.

## Tâches

- [ ] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : `crates/kesh-db/tests/opening_complement_repository.rs` (portée
  `SingleBankLedger` sur société vide : passe ; portée `Screen` : `NoEntries` inchangé) ; `crates/kesh-api/tests/bank_accounts_e2e.rs` ou fichier
  neuf `bank_account_opening_balance_e2e.rs` : AC-2 à AC-6, dont l'atomicité comptée (AC-5) et un crédit sur un compte `Liability`.
- [ ] **T2 — dépôt** : `create_opening_complement_in_tx` + `ComplementScope` ; `create_opening_complement` devient une enveloppe (lecture
  `owned_account_ids` hors transaction **inchangée** pour l'écran). Pour le compte bancaire, `owned` est le seul compte lié, déjà contrôlé de la
  société par `validate_journal_account_id` (aucun verrou sur un compte d'autrui : la raison d'être d'`owned`, `:441-452`, reste tenue).
  ⚠️ L'ordre des verrous change dans `create_once` : `companies` en exclusif (sentinelle) précède le partagé que demande le complément — déjà tenu,
  sans effet ; puis `bank_accounts`, puis comptes et exercices. Le dire au doc-comment de `create_once` ; le rejeu couvre les cycles résiduels.
  ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [ ] **T3 — routes** : corps `openingBalance` (forme partagée avec la route neuve), réponse augmentée (`#[serde(flatten)]` du compte),
  `create_once` appelle le complément après l'`INSERT`, avant les traces ; route neuve dans `comptable_routes` (`lib.rs:~936-955`), bornée par
  `retry_app_on_deadlock`. Mapping des refus : celui d'`errors.rs:3414` (aucun nouveau code).
- [ ] **T4 — registre** (AC-8).
- [ ] **T5 — écran** : `bank-accounts.api.ts` (`createBankAccount` rend `{ …, openingEntry }`, `enterOpeningBalance(id, …)`), formulaire et
  action de ligne ; Vitest : champs masqués sans lien, présélection du sens par type, corps envoyé, avis de succès, refus affiché.
- [ ] **T6 — i18n** (quatre locales) : libellé de l'écriture, champs, sens, aide, avis, action. `lint-i18n-ownership`.
- [ ] **T7 — E2E** : spec neuve `bank-account-opening-balance.spec.ts` : créer un compte lié à un compte neuf (créé par l'API, jamais mouvementé)
  avec 1'234.50 *Avoir* → la liste montre le solde `1'234.50` dans la colonne *Solde* (la valeur traverse HTTP et le calcul de
  `list_by_company_with_balances`) ; second essai sur un compte mouvementé → motif affiché, aucun compte créé.
- [ ] **T8 — manuel** (AC-9), captures (formulaire avec solde, avis, refus) dans `kesh-gate-logs/e341/captures/34-2-*.png`.
- [ ] **T9 — CHANGELOG** : *Ajouté*, `refs #533`.

### Mutations nommées

- **M1** : `ComplementScope` ignoré (NoEntries toujours levé) → rouge « portée `Screen` : `NoEntries` inchangé ».
- **M2** : l'appel du complément déplacé **après** le `commit` de `create_once` → rouge AC-5 (compte bancaire présent après refus).
- **M3** : *Avoir* envoyé en crédit → rouge AC-3.
- **M4** : la contrepartie posée sur le compte lié lui-même (écriture nulle) → rouge AC-3 (deux comptes distincts).
- **M5** : `openingBalance` accepté sans `journalAccountId` → rouge AC-2.
- **M6** : la route de ligne sans contrôle « clos » → rouge AC-6.

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

`routes/(app)/bank-accounts/+page.svelte`, `bank-accounts.api.ts`, `routes/bank_accounts.rs` avec **34-1** (avant) et **34-11a** (après : une
carte se crée avec son solde d'ouverture, sens *Dette*). `opening_complement.rs` : aucune autre story de l'epic.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
