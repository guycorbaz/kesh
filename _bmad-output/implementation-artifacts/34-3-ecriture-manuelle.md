# Story 34-3 : « Écriture manuelle » sur tout compte imputable, sauf celui de la banque (#967)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e342.md`,
C-e342-1 à C-e342-5. Commits en `refs #967` ; à la PR : `closes #967`. **Précède 34-4, 34-11a et 34-11b** (fichiers communs et
textes, § *Recoupements*).*

## Story

En tant que Comptable qui rapproche ses relevés, je veux passer une transaction sans facture sur n'importe quel compte
imputable — intérêts reçus, virement vers un autre de mes comptes, apport ou prélèvement privé, achat de marchandise —, par
un bouton dont le nom dit qu'il passe une écriture, pour ne plus être limité aux classes 5, 6 et 7 ni chercher le geste.

## Décision de Guy (2026-11-10, fichier d'epic)

« Écriture manuelle » remplace « Affecter manuellement ». Tout compte imputable est proposé, **sauf celui de la banque**. Le
filtre sur les classes 5, 6 et 7 est retiré dans les deux dialogues (*Écriture manuelle* et *Éclater*). La case à cocher par
compte est écartée. Aucun rôle ne se déduit du numéro d'un compte (Story 14-3a).

## Ce que le code dit (prémisses établies)

- **Le filtre est purement client, en deux copies** :
  - `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte:87-93` : `['5', '6', '7'].some((c) => a.number.startsWith(c)) && a.id !== bankLedgerAccountId` ;
  - `TransactionSplitModal.svelte:74-76` : le même filtre de classes, **sans** l'exclusion du compte de la banque.
- ⚠️ **Prémisse ajoutée — Éclater propose le compte de la banque dès que le filtre tombe.** `ReconciliationProposals.svelte:671-681`
  ne passe pas `bankLedgerAccountId` à `TransactionSplitModal` (seul `ManualMatchModal` le reçoit, `:658-669`). Aujourd'hui le
  filtre de classes l'écarte par accident (un compte de banque est en classe 1). Retirer le filtre sans rien d'autre ferait
  proposer le compte de la banque, que le serveur refuse ensuite en 400. Traité ici (C-e342-2).
- **Le reste du filtrage est déjà juste** : `AccountAutocomplete.svelte:200-207` ne garde que `a.active && a.postable` ; la
  liste vient de `fetchAccounts()` sans archivés (`ReconciliationProposals.svelte:184-197`).
- **Le serveur ne filtre pas par classe** et refuse ce qu'il faut :
  - `post_manual` (`crates/kesh-api/src/routes/reconciliation.rs:4791`) : compte de la banque refusé en 400
    `counterparty_equals_bank_ledger` (`:4881-4883`), compte archivé ou d'une autre société en 404 (`:4885-4900`), non
    imputable en 400 `ACCOUNT_NOT_POSTABLE` (`:4907-4914`) ;
  - `post_split` (`:5210`) : mêmes refus par ligne (`:5326-5332`, `:5342-5375`) ;
  - lot `accept` (`split`, `rule`) : `:3568-3633`, `:4000-4035`.
  Aucun code serveur ne change de comportement. Deux commentaires deviennent faux et se corrigent : `reconciliation.rs:5320`
  (« le frontend filtre client-side classes 5/6/7 ») et `crates/kesh-reconciliation/src/manual.rs:31` (« typiquement classe
  5/6/7 — […] le frontend filtre côté UX »).
- ⚠️ **Conséquence du retrait : le compte débiteurs et le compte créanciers deviennent proposés.** Le manuel le dit aujourd'hui
  impossible : « L'affectation manuelle ne règle pas une facture : le compte débiteurs n'est pas proposé » (`docs/manual/fr/user-manual.tex:2961`).
  Une écriture manuelle sur 1100 laisserait la facture due, sans lettrage, et rien ne le signalerait : le faux rattachement muet
  du `CLAUDE.md`. Les rôles `Receivable` et `Payable` existent (`frontend/src/lib/features/accounts/accounts.types.ts:10-20`,
  champ `role` de `AccountResponse`, `:53`). D'où un avis non bloquant (C-e342-1) : signaler, jamais bloquer.
- **Le libellé et ses citations** (recompté, `git grep`) :
  - clé `reconciliation-manual-button-label` : fr `messages.ftl:1508`, de/en/it `:1441` (« Manuell zuweisen », « Assign
    manually », « Assegna manualmente ») ; titre `reconciliation-manual-modal-title` (fr `:1509`, autres `:1442`) ; bouton de
    validation `reconciliation-manual-submit` = « Affecter » (fr `:1514`) ;
  - le libellé est **cité** dans quatre messages par locale : `reconciliation-failed-rule-masked-payment-batch`,
    `-invoice-candidate`, `-overpaid`, `reconciliation-failed-rule-masked` (fr `:1476-1479`, de/en/it `:1410-1413`), et dans
    leurs replis de `failed-proposal-label.ts:327-352` ; le test `failed-proposal-label.test.ts:357` asserte
    `N’utilisez « Affecter manuellement » que si` ;
  - repli du bouton `ReconciliationProposals.svelte:585`, commentaires `:15`, `:298`, `:574` ; avis du lot de paiement
    `reconciliation-labels-notice-supplier-invoice-in-batch` (« affectez la transaction à la main », `:503`) ;
  - manuel utilisateur : **13** lignes nommant l'affectation manuelle (`/usr/bin/grep -c -iE 'Affecter manuellement|affectation manuelle'`),
    dont celles qui citent les classes 5, 6 et 7 : `:2903`, `:2957`, `:3004`, `:3947` ;
  - `README.md:42` (« réconciliation manuelle ✓ ») et `CHANGELOG.md:1492` : historiques, inchangés.
- **Les E2E ne ciblent pas le libellé** : `reconciliation-manual.spec.ts` et `reconciliation-split.spec.ts` lisent les
  `data-testid` (`manual-match-button`, `split-button`) et ne montent aucune transaction en attente. Aucun E2E ne passe
  aujourd'hui une écriture manuelle de bout en bout. Les aides existent : `createManualDebitStatementViaApi`
  (`frontend/tests/e2e/helpers/api-fixtures.ts:331`) pose un relevé saisi d'un débit.
- **Tests serveur de refus, inchangés et verts** : `post_manual_rejects_bank_ledger_as_counterparty`
  (`crates/kesh-api/tests/reconciliation_manual_e2e.rs:1534`), `post_manual_bank_ledger_counterparty_precedes_not_postable`
  (`:1548`), `post_manual_archived_non_postable_counterparty_is_404_first` (`:1408`),
  `split_bank_ledger_counterparty_precedes_not_postable` (`reconciliation_split_e2e.rs:1109`).

## Acceptance Criteria

1. **AC-1 — le nom** : le bouton de chaque ligne s'intitule « Écriture manuelle » (fr), « Manuelle Buchung » (de), « Manual
   entry » (en), « Registrazione manuale » (it). Le titre du dialogue dit « Écriture manuelle » (et ses équivalents), son
   bouton de validation « Passer l'écriture » (« Buchung erfassen », « Post entry », « Registra ») (C-e342-3).
2. **AC-2 — Écriture manuelle, la liste** : le sélecteur propose tout compte **actif et imputable** de la société, de toute
   classe, **sauf** le compte du grand livre du compte bancaire. Un compte non imputable, archivé, ou le compte de la banque
   n'y figure pas. Vitest : un plan de test portant un compte de chaque classe 1 à 9, un compte non imputable, un archivé et
   le compte de la banque ; la liste rendue est exactement l'attendu (ensemble, non cardinal).
3. **AC-3 — Éclater, la même liste** : même règle pour chaque ligne d'*Éclater*, compte de la banque compris (il est
   aujourd'hui écarté par accident). `TransactionSplitModal` reçoit `bankLedgerAccountId` du parent, comme `ManualMatchModal`.
   Le filtre s'écrit **une fois** (DRY), dans une fonction pure partagée par les deux dialogues
   (`reconciliation-counterparties.ts`, nom à confirmer) ; repli `null` du compte de la banque inchangé (aucun filtrage, le
   serveur tranche).
4. **AC-4 — l'avis débiteurs / créanciers** (C-e342-1) : quand le compte choisi porte le rôle `Receivable` ou `Payable`,
   un avis s'affiche sous le sélecteur (dans *Éclater* : sous la ligne concernée), sans rien bloquer : « Ce compte est celui
   des débiteurs (ou : des créanciers). Une écriture manuelle ne règle aucune facture : la facture resterait due. Pour un
   paiement de facture, enregistrez le règlement depuis sa fiche, puis rattachez-y la transaction. » Il disparaît si l'on
   choisit un autre compte. `data-testid="counterparty-claim-account-notice"`. Quatre locales.
5. **AC-5 — le serveur, inchangé** : aucune ligne exécutable de `crates/` ne change ; les deux commentaires cités se
   corrigent. Les tests serveur de refus listés ci-dessus passent sans modification.
6. **AC-6 — les citations du libellé** : les quatre messages `reconciliation-failed-rule-masked*` et l'avis du lot citent
   « Écriture manuelle » dans les quatre locales, et leurs replis Svelte/TS sont identiques au catalogue fr (la garde
   `i18n-repli-divergent-actif.test.ts` le contrôle). `/usr/bin/grep -rn 'Affecter manuellement\|Manuell zuweisen\|Assign manually\|Assegna manualmente'`
   sur `crates/kesh-i18n frontend/src docs/manual/fr/*.tex` ne rend plus rien, hors historiques datés.
7. **AC-7 — E2E de bout en bout** (`reconciliation-manual-entry.spec.ts`, neuf) : relevé saisi par l'API, un débit d'un
   montant unique ; *Réconciliation* → *Écriture manuelle* sur la ligne → choix d'un compte **de classe 1 ou 2** (établi à
   l'étape 0 dans le plan semé, p. ex. la caisse ; jamais un numéro supposé) → *Passer l'écriture* → la ligne quitte la liste ;
   l'écriture existe au journal, sur ce compte, au montant (lu par l'API) ; le dialogue ne propose pas le compte de la banque.
   En `finally` : rapprochement annulé, import supprimé (patron de `reconciliation-link-entry.spec.ts`). Il rougit si le filtre
   de classes revient (mutation M1).
8. **AC-8 — manuel utilisateur** : § *Réconciliation manuelle* (`user-manual.tex:2949-2961`) renommé et récrit : tout compte
   imputable sauf celui de la banque, exemples (intérêts reçus, virement vers un autre de vos comptes, apport privé), l'avis
   débiteurs/créanciers, et la conduite pour un paiement de facture ; `:2903`, `:3004`, `:3947`, `:3949`, `:2945` alignés ;
   le motif de `:3947` (« ils ne proposent que les comptes des classes 5, 6 et 7 ») devient « la facture resterait due ».
   PDF régénéré, contrôlé aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`) : « Écriture manuelle » présent, « classes 5, 6
   et 7 » absent. Le virement entre deux de vos comptes garde sa consigne : le second côté se rattache ou se rejette
   (`:2991`, issue #823), il ne reçoit pas une seconde écriture.

## Tâches

- [ ] **T0 — étape 0** : relire chaque prémisse ci-dessus au commit de départ (fichier:ligne cités au Change Log) ; établir le
  compte de classe 1 ou 2 du plan semé utilisé par l'E2E.
- [ ] **T1 — rouge** : Vitest AC-2, AC-3 (dont « Éclater n'offre pas le compte de la banque »), AC-4, et l'E2E AC-7, écrits
  d'abord ; les journaliser rouges (`kesh-gate-logs/e342/rouges.log`). Mettre à jour les assertions existantes qui figent le
  filtre : `ManualMatchModal.test.ts:79`, `:234-262` (« aucun filtrage au-delà des classes 5/6/7 »),
  `TransactionSplitModal.test.ts:75`, `ReconciliationProposals.test.ts:260-285`, `failed-proposal-label.test.ts:341-357`.
- [ ] **T2** : fonction pure de filtrage partagée ; `ManualMatchModal`, `TransactionSplitModal` (nouvelle prop
  `bankLedgerAccountId`), `ReconciliationProposals` (passage de la prop) ; avis AC-4 ; doc-comments d'en-tête des deux dialogues.
- [ ] **T3** : catalogues fr/de/en/it (bouton, titre, validation, avis, quatre messages, avis du lot), replis ; `failed-proposal-label.ts`.
- [ ] **T4** : commentaires `reconciliation.rs:5320`, `manual.rs:31` (sans changement exécutable).
- [ ] **T5** : manuel utilisateur, PDF ; CHANGELOG `## [0.17.6] — Non publié`, *Changé* (`refs #967`).
- [ ] **T6** : captures (thème clair, base jetable) : dialogue *Écriture manuelle* ouvert sur un compte de classe 2 ; avis
  débiteurs ; *Éclater* avec une ligne sur un compte de produits. Dans `kesh-gate-logs/e342/captures/`.

### Mutations nommées

- **M1** : rétablir `['5','6','7'].some(...)` dans la fonction partagée → rouge AC-2, AC-3, AC-7.
- **M2** : retirer `a.id !== bankLedgerAccountId` de la fonction partagée → rouge AC-2 et « Éclater n'offre pas le compte de la banque ».
- **M3** : ne plus passer `bankLedgerAccountId` à `TransactionSplitModal` → rouge AC-3.
- **M4** : avis conditionné à `Receivable` seul → rouge AC-4 (cas `Payable`).
- **M5** : repli d'un message `failed-rule-masked` laissé à l'ancien libellé → rouge de la garde des replis.

## Gates

Frontend surtout ; aucun fichier `kesh-db`. Gate ciblé entre les passes (`npm run check`, Vitest des fichiers touchés,
`cargo fmt`/`clippy` du workspace pour les deux commentaires), gate complet et **E2E complet au dernier commit de code**.

## Ce qui ne se traite pas

- **Les règles d'affectation** restent limitées aux comptes de charges et de produits (`rules/RuleFormModal.svelte:44-51`,
  `accountType === 'Expense' || 'Revenue'`) : une règle pour un virement récurrent vers un compte d'épargne ne se crée pas.
  Hors décision de Guy (qui vise les deux dialogues). **Issue demandée** (C-e342-4).
- Le second côté d'un virement entre deux comptes propres : #823, et le signal de la 34-4.

## Recoupements

`ReconciliationProposals.svelte` et le § *Réconciliation* du manuel avec **34-4** (avertissement, confirmation, et ses
textes qui nomment « Écriture manuelle ») et **34-11b** (carte de crédit). 34-3 passe la première ; si 34-4 se développe en
parallèle, ses textes nomment déjà « Écriture manuelle » (C-e342-5).

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
