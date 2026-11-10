# Story 34-3 : « Écriture manuelle » sur tout compte imputable, sauf celui de la banque (#967)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e342.md`,
C-e342-1 à C-e342-5, C-e342-20, C-e342-22. Commits en `refs #967` ; à la PR : `closes #967`. **Précède 34-4, 34-11a et 34-11b** (fichiers communs et
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
    `reconciliation-labels-notice-supplier-invoice-in-batch` (« affectez la transaction à la main » ; catalogue fr `:1433`,
    de/en/it `:1367` ; repli `ReconciliationProposals.svelte:503`) ;
  - *(validation P1, F3-2)* trois autres messages renvoient au geste **sans** citer le libellé exact :
    `reconciliation-labels-more-capped` (« affectées manuellement », fr `:1441`, de/en/it `:1375`, repli
    `ReconciliationProposals.svelte:652`), `reconciliation-failed-supplier-invoice-in-batch` (« affectez la transaction à la
    main », fr `:1473`, autres `:1407`, repli `failed-proposal-label.ts:302`), `reconciliation-failed-supplier-payment-hold`
    (fr `:1475`, autres `:1409`, repli `failed-proposal-label.ts:322`) : 12 messages et 3 replis de plus ;
  - commentaire E2E `reconciliation-manual.spec.ts:54` ;
  - manuel utilisateur : **13** lignes nommant l'affectation manuelle (`/usr/bin/grep -c -iE 'Affecter manuellement|affectation manuelle'`),
    dont celles qui citent les classes 5, 6 et 7 : `:2903`, `:2957`, `:3004`, `:3947` ; les autres : `:2933`, `:2945`, la
    légende `:2951` et sa capture `reconciliation-manuelle.png` (encore « TODO capture »), `:3008` (« Comme l'affectation
    manuelle »), `:3010` (« le sélecteur de l'affectation manuelle ne propose pas ce compte », à étendre à *Éclater*),
    `:3034`, `:3546`, `:3949` ;
  - *(validation P1, F3-4)* le manuel donne **deux conduites différentes** selon le sens : un paiement de client se règle
    depuis la fiche de la facture puis se rattache (`:3947`) ; le paiement d'un fournisseur passe par la facture **saisie et
    validée, sans la payer depuis sa fiche**, puis *Accepter* sa proposition — ou, si elle est déjà réglée, le rattachement
    (`:3948`) ;
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
   un avis s'affiche sous le sélecteur (dans *Éclater* : sous la ligne concernée), sans rien bloquer. **Deux textes**, alignés
   sur le manuel (`user-manual.tex:3947`, `:3948`) :
   - `Receivable` : « Ce compte est celui des débiteurs. Une écriture manuelle ne règle aucune facture client : elle resterait
     due. Pour un paiement de client, enregistrez le règlement depuis la fiche de la facture, puis rattachez-y la
     transaction. » ;
   - `Payable` : « Ce compte est celui des créanciers. Une écriture manuelle ne règle aucune facture fournisseur : elle
     resterait à payer. Pour le paiement d'un fournisseur, saisissez et validez la facture sans la payer depuis sa fiche : elle
     vous sera proposée ici, à accepter. Si elle est déjà payée, rattachez la transaction à ce paiement. »
   Il disparaît si l'on choisit un autre compte. `data-testid="counterparty-claim-account-notice"`, `data-role`. Quatre
   locales. Vitest des deux rôles, texte attendu complet. Hors périmètre, écrit ici : les rôles de TVA et un compte 1100 ou
   2000 **sans rôle posé** n'avertissent pas (C-e342-1, rôle explicite).
5. **AC-5 — le serveur, inchangé** : aucune ligne exécutable de `crates/` ne change ; les deux commentaires cités se
   corrigent. Les tests serveur de refus listés ci-dessus passent sans modification.
6. **AC-6 — les citations du libellé** : les quatre messages `reconciliation-failed-rule-masked*`, l'avis du lot, et les
   trois messages qui renvoient au geste sans le citer (`reconciliation-labels-more-capped`,
   `reconciliation-failed-supplier-invoice-in-batch`, `reconciliation-failed-supplier-payment-hold`) nomment « Écriture
   manuelle » (ou « passée par une écriture manuelle ») dans les quatre locales, et leurs replis Svelte/TS sont identiques au
   catalogue fr. ⚠️ *(Validation P2, P2-3-3)* `i18n-repli-divergent-actif.test.ts:142` **ne contrôle pas** cet écart : il ne
   retient que les clés dont les replis divergent **entre plusieurs sites**, et chacun de ces replis n'a qu'un site. Le contrôle
   est donc un test neuf, dans `failed-proposal-label.test.ts` (patron de `:357`) ou un test qui compare repli et catalogue fr,
   sur la **liste fermée** des **huit** clés de cet AC (quatre `failed-rule-masked*`, l'avis du lot, trois de F3-2) et celle du
   bouton. *(Validation P3, P3-3-2)* `ecartsAuCatalogue` (`frontend/src/lib/shared/i18n-repli-catalogue.ts:82-104`) juge **par
   préfixe** et signale aussi les clés du préfixe qu'aucun site littéral ne demande : appelé tel quel sur les trois préfixes
   (`reconciliation-labels-`, `reconciliation-failed-`, `reconciliation-manual-` ; *P4-F4 : écrits en entier, `startsWith` ne
   retenant rien d'un préfixe tronqué*), il signalerait des voisines sans rapport. Le test **filtre** son résultat
   sur la liste fermée (ou compare relevé des replis et catalogue clé par clé), et asserte aussi que chacune des neuf clés est
   **présente** dans le relevé.
   **Garde de contrôle** *(P2-3-1 : la forme de la P1, avec `manuell`, captait « manuelle » et le libellé neuf lui-même, 197
   lignes)*, insensible à la casse, sur le **geste**, non sur le mot :
   `git grep -niE 'affect(er|ation|ée?s?|ez)[^.]{0,20}(manuel|à la main)|manuell zu(weisen|gewiesen)?\b|ordnen [^.]{0,30} manuell zu|manueller abgleich|assign(ed)?[^.]{0,30}manual|assegn[a-z]*[^.]{0,30}(manual|a mano)|riconciliazione manuale|manual reconciliation|réconciliation manuelle' -- crates/kesh-i18n frontend/src frontend/tests 'docs/manual/fr/*.tex'`.
   *(Validation P3, P3-3-1 : la forme de la P2 limitait « assign … manual » à 12 caractères et voulait « assegna » collé à
   « manual » / « a mano » ; elle manquait « otherwise assign the transaction manually » et « assegnare la transazione a mano »,
   trois messages par locale en anglais et en italien.)* Au commit de rédaction elle rend **97 lignes** (catalogues fr 12, de 12,
   en 12, it 12 ; `user-manual.tex` 18 ; `admin-manual.tex` 1 ; `frontend/src` 29 ; `frontend/tests` 1 ; rejouée et recomptée
   par `| cut -d: -f1 | sort | uniq -c` au commit `73c365af`).
   **Règle des commentaires** *(validation P4 ciblée, F1 : la P3 disait « ceux qui citent une story datée restent » tout en
   listant à réécrire des lignes qui en citent une)* : **la citation de story reste, la phrase qui nomme le geste se réécrit**
   (« Story 8-5a-base FR45 — réconciliation manuelle » devient « Story 8-5a-base FR45 — écriture manuelle (geste rebaptisé par
   la 34-3) », **sans l'ancien nom** ; en-têtes de/en/it : « Manuelle Buchung (Geste von der 34-3 umbenannt) », « Manual entry
   (renamed by 34-3) », « Registrazione manuale (rinominata dalla 34-3) » — *P5-1 : l'exemple de la P4 citait « réconciliation
   manuelle » et répondait encore à la garde ; ces quatre formes, passées à la garde, n'y répondent pas*). Elle s'applique à **tous** les
   commentaires que la garde rend :
   - qui citent la story 8-5a-base : `ReconciliationProposals.svelte:574`, `ReconciliationProposals.test.ts:260`,
     `ManualMatchModal.svelte:2`, `reconciliation.api.ts:67`, `reconciliation.types.ts:142`, et l'**en-tête de catalogue**
     `# Story 8-5a-base FR45 — Réconciliation manuelle.` (fr `messages.ftl:1507`, de/en/it `:1440`), qui coiffe le bouton et le
     titre renommés *(F3 : c'est le seul en-tête que la garde rend ; le libellé d'audit n'en a pas)* ;
   - qui n'en citent pas : `ReconciliationProposals.svelte:15`, `:115`, `:200`, `:298`, `ReconciliationProposals.test.ts:279`,
     `:553`, `:556` (donnée de test, réécrite avec son assertion), `:601`, `bank-accounts.api.ts:2`, et *(F2)*
     `failed-proposal-label.ts:298`, `:327`, `failed-proposal-label.test.ts:329`, `:341`, et *(P5-2)* le commentaire E2E
     `frontend/tests/e2e/reconciliation-manual.spec.ts:54`.
   **Survivants attendus après la story**, et eux seuls : le libellé d'audit `audit-log-action-reconciliation-manual-matched`
   (C-e342-3 ; fr `:3150`, de `:3077`, en `:3083`, it `:3078`) et `admin-manual.tex:2100` (« n'en est pas affectée ; ce manuel »,
   faux positif) — **cinq lignes**. Sortie et nombre résiduel copiés au Change Log. ⚠️ La garde est ligne à ligne : un libellé coupé
   par la ligne (`user-manual.tex:2902-2903`, « \emph{Affecter » / « manuellement} ») lui échappe ; il est listé à l'AC-8.
7. **AC-7 — E2E de bout en bout** (`reconciliation-manual-entry.spec.ts`, neuf) : relevé saisi par l'API, un débit d'un
   montant unique ; *Réconciliation* → *Écriture manuelle* sur la ligne → choix d'un compte **de classe 1 ou 2** autre que celui
   de la banque : dans ce harnais, **`2000 Capital CI`** du preset `with-company` (`POST /api/v1/_test/seed`,
   `crates/kesh-db/src/test_fixtures.rs:127-131` ; *P2-3-4 : c'est lui que sèment les E2E, non `scripts/seed-dev-db.sql`*) —
   **pas** `1000 Caisse CI`, que `wiredBankAccountViaApi` relie au compte bancaire (`helpers/api-fixtures.ts:229`) et que le
   dialogue n'offre donc pas, ni `1100 Banque CI`, compte débiteurs par défaut des réglages (`test_fixtures.rs:163-164`), dont
   l'emploi brouillerait le parcours. Le preset ne pose aucun `role` (`:135`) : l'avis AC-4 n'est **pas** exercé par cet E2E,
   seulement par Vitest. À confirmer à l'étape 0, jamais un numéro supposé → *Passer l'écriture* → la ligne quitte la liste ;
   l'écriture existe au journal, sur ce compte, au montant (lu par l'API) ; le dialogue ne propose pas le compte de la banque.
   En `finally` : rapprochement annulé, import supprimé (patron de `reconciliation-link-entry.spec.ts`). Il rougit si le filtre
   de classes revient (mutation M1).
8. **AC-8 — manuel utilisateur** : § *Réconciliation manuelle* (`user-manual.tex:2949-2961`) renommé et récrit : tout compte
   imputable sauf celui de la banque, exemples (intérêts reçus, virement vers un autre de vos comptes, apport privé), l'avis
   débiteurs/créanciers et ses deux conduites ; `:2860`, `:2874` (« affectez la transaction à la main », lot de paiement),
   `:2902-2903` (libellé coupé par la ligne), `:2933`, `:2936` (« acceptées, affectées manuellement, éclatées ou rattachées »,
   double de `reconciliation-labels-more-capped`), `:2941` (« utilisez \emph{Affecter manuellement} »), `:2942` (« acceptée,
   affectée manuellement, éclatée ou rattachée »), `:2945`, `:2951` (légende), `:2956`, `:3004`, `:3008`, `:3010` (étendu à
   *Éclater*), `:3034`, `:3546`, `:3947`, `:3949` alignés *(P2-3-2 : liste recomptée sur la sortie de la garde de l'AC-6, plus
   `:2902-2903` qu'elle ne voit pas)* ; le motif de `:3947` (« ils ne proposent que les comptes
   des classes 5, 6 et 7 ») devient « la facture resterait due ». Le virement entre deux de vos comptes garde sa consigne : le
   second côté se rattache ou se rejette (`:2991`, issue #823), il ne reçoit pas une seconde écriture.
   **PDF** : le `.tex` change dans la story ; le PDF de `user-manual` est touché par 34-3, 34-4, 34-9 et 34-10, et un PDF ne
   se fusionne pas : il se **régénère une fois à l'intégration**, puis se contrôle aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '` :
   « Écriture manuelle » présent, « classes 5, 6 et 7 » absent). La story peut le régénérer pour se relire, sans le commiter
   (le `CLAUDE.md` exige un PDF à jour au push et à la release : c'est l'intégration qui le porte).

## Tâches

- [ ] **T0 — étape 0** : relire chaque prémisse ci-dessus au commit de départ (fichier:ligne cités au Change Log) ; établir le
  compte de classe 1 ou 2 du plan semé utilisé par l'E2E.
- [ ] **T1 — rouge** : Vitest AC-2, AC-3 (dont « Éclater n'offre pas le compte de la banque »), AC-4, et l'E2E AC-7, écrits
  d'abord ; les journaliser rouges (`kesh-gate-logs/e342/rouges.log`). Mettre à jour les assertions existantes qui figent le
  filtre : `ManualMatchModal.test.ts:79`, `:234-262` (« aucun filtrage au-delà des classes 5/6/7 »),
  `TransactionSplitModal.test.ts:75`, `ReconciliationProposals.test.ts:260-285`, `failed-proposal-label.test.ts:341-357`.
  *(F3-3)* Les deux dialogues sont remplacés par `ModalSuccessStub.test.svelte` dans `ReconciliationProposals.test.ts:50-55` :
  ajouter au bloc 15-6d (`:600-660`, qui lit `data-bank-ledger-account-id` pour `manual-match-button`) un test
  « *Éclater* reçoit le compte de la banque » qui clique `split-button` et lit la prop du stub. *(P2-3-5)* L'en-tête de
  `ModalSuccessStub.test.svelte` (« `"undefined"` si le parent ne la passe pas — cas de l'éclatement ») devient faux : le
  corriger.
- [ ] **T2** : fonction pure de filtrage partagée ; `ManualMatchModal`, `TransactionSplitModal` (nouvelle prop
  `bankLedgerAccountId`), `ReconciliationProposals` (passage de la prop) ; avis AC-4 ; doc-comments d'en-tête des deux dialogues.
- [ ] **T3** : catalogues fr/de/en/it (bouton, titre, validation, deux avis, quatre messages `failed-rule-masked*`, avis du lot, trois messages de F3-2), replis ; `failed-proposal-label.ts`.
- [ ] **T4** : commentaires `reconciliation.rs:5320`, `manual.rs:31` (sans changement exécutable).
- [ ] **T5** : manuel utilisateur (`.tex` ; PDF à l'intégration, AC-8) ; CHANGELOG `## [0.17.6] — Non publié`, *Changé* (`refs #967`).
- [ ] **T6** : captures (thème clair, base jetable) : dialogue *Écriture manuelle* ouvert sur un compte de classe 2 ; avis
  débiteurs ; *Éclater* avec une ligne sur un compte de produits. Dans `kesh-gate-logs/e342/captures/`.

### Mutations nommées

- **M1** : rétablir `['5','6','7'].some(...)` dans la fonction partagée → rouge AC-2, AC-3, AC-7.
- **M2** : retirer `a.id !== bankLedgerAccountId` de la fonction partagée → rouge AC-2 et « Éclater n'offre pas le compte de la banque ».
- **M3** : ne plus passer `bankLedgerAccountId` à `TransactionSplitModal` → rouge « *Éclater* reçoit le compte de la banque »
  (`ReconciliationProposals.test.ts`, bloc 15-6d) ; le Vitest de `TransactionSplitModal.test.ts`, qui passe la prop
  lui-même, reste vert et ne compte pas.
- **M4** : avis conditionné à `Receivable` seul → rouge AC-4 (cas `Payable`).
- **M4-bis** : texte client affiché pour `Payable` → rouge AC-4 (texte attendu du cas `Payable`).
- **M5** : repli d'un message `failed-rule-masked` laissé à l'ancien libellé → rouge du test neuf repli/catalogue de l'AC-6
  (non de `i18n-repli-divergent-actif.test.ts`, qui reste vert sur ce cas, P2-3-3).

## Gates

Frontend surtout ; aucun fichier `kesh-db`. Gate ciblé entre les passes (`npm run check`, Vitest des fichiers touchés,
`cargo fmt`/`clippy` du workspace pour les deux commentaires), gate complet et **E2E complet au dernier commit de code**.

## Ce qui ne se traite pas

- **Les règles d'affectation** restent limitées aux comptes de charges et de produits (`rules/RuleFormModal.svelte:44-51`,
  `accountType === 'Expense' || 'Revenue'`) : une règle pour un virement récurrent vers un compte d'épargne ne se crée pas.
  Hors décision de Guy (qui vise les deux dialogues). **#973** (C-e342-4).
- *(F3-5)* L'avis AC-4 ne couvre ni les comptes de TVA (`VatRecoverable`, `VatPayable`, `VatSettlement`), ni un compte
  débiteurs ou créanciers dont le rôle n'est pas posé. **Issue demandée** pour les comptes de TVA (une écriture manuelle y
  fausse le décompte sans signal) ; le second cas découle du choix « rôle explicite » (Story 14-3a) et n'appelle pas d'issue.
  *(Annoté le 2026-11-10, revue de code P1, A1-2 : l'issue des comptes de TVA est ouverte, **#981**.)*
- Le second côté d'un virement entre deux comptes propres : #823, et le signal de la 34-4.

## Recoupements

`ReconciliationProposals.svelte` et le § *Réconciliation* du manuel avec **34-4** (avertissement, confirmation, et ses
textes qui nomment « Écriture manuelle ») et **34-11b** (carte de crédit). 34-3 passe la première ; si 34-4 se développe en
parallèle, ses textes nomment déjà « Écriture manuelle » (C-e342-5).

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : Validation P1 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p1-e342.md`, pour cette
  fiche 0 C, 0 H, 4 M, 1 LOW, tous retenus après lecture : **F3-1** (lus : `helpers/api-fixtures.ts:229`, `seed-dev-db.sql:73-75` :
  1000 est le compte de la banque du harnais) → AC-7 ; **F3-2** (lus : fr `messages.ftl:1441`, `:1473`, `:1475` ; manuel
  `:2933`, `:2951`, `:3008`, `:3010`, `:3034`, `:3546`) → inventaire et garde AC-6 élargis ; **F3-3** (lu :
  `ReconciliationProposals.test.ts:50-55`, les dialogues y sont des stubs) → test au parent, M3 réécrite ; **F3-4** (lu :
  `user-manual.tex:3948`) → deux textes à l'AC-4, M4-bis ; **F3-5** (lu : `accounts.types.ts:10-20`) → hors périmètre écrit,
  issue TVA demandée. Transversal **X-1** : PDF régénéré à l'intégration (AC-8). L'issue des règles d'affectation est ouverte : #973.
- 2026-11-10 : Validation P2 (Opus), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p2-e342.md`, pour cette
  fiche 0 C, 0 H, 3 M, 3 LOW, tous retenus après lecture :
  - **P2-3-1** MEDIUM, **né de la remédiation P1** (F3-2) : la garde de l'AC-6 captait « manuelle », donc le libellé neuf.
    Garde réécrite sur le geste ; lancée au commit de rédaction, 91 lignes (ventilation à l'AC-6) ; survivants attendus nommés ;
  - **P2-3-2** MEDIUM, **défaut d'origine que la P1 n'a pas comblé** : lus `user-manual.tex:2874`, `:2936`, `:2941`, `:2942`
    (et `:2860`, `:2956`, rendus par la garde neuve) ; liste de l'AC-8 recomptée sur la sortie de la garde, plus `:2902-2903`,
    coupé par la ligne ;
  - **P2-3-3** MEDIUM, **défaut d'origine** (AC-6 et M5 dès la création) : lu `i18n-repli-divergent-actif.test.ts:142`, la
    garde ne voit que les replis divergents entre sites ; test neuf repli/catalogue nommé, AC-6 et M5 corrigés ;
  - LOW : **P2-3-4** (lus : `test_fixtures.rs:127-135`, `:163-164`) → E2E sur `2000 Capital CI`, avis non exercé par l'E2E
    dit ; **P2-3-5** (lu : en-tête de `ModalSuccessStub.test.svelte`) → T1 ; **P2-3-6** (lus : fr `:1433`, de `:1367`) →
    citation.
  - CHANGELOG : *Changé* gardé (comportement d'un geste existant modifié ; admis par l'orchestrateur), cohérent avec la 34-10 ;
    la 34-4 en *Ajouté*, la 34-9 en *Corrigé* (C-e342-22).
- 2026-11-10 : Validation P3 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p3-e342.md`, pour cette
  fiche 0 C, 0 H, 1 M, 2 LOW, tous retenus :
  - **P3-3-1** MEDIUM, **défaut d'origine non vu par la P2** (motifs anglais et italien hérités de la P1) : garde élargie,
    **rejouée** au commit `73c365af` : 97 lignes, catalogues 12 / 12 / 12 / 12 (les six de plus sont les trois messages
    `-supplier-invoice-in-batch`, `-supplier-payment-hold`, `-notice-supplier-invoice-in-batch` en anglais et en italien) ;
  - **P3-3-2** LOW (lu : `i18n-repli-catalogue.ts:82-104`, filtre par préfixe) → test filtré sur la liste fermée, présence
    assertée ;
  - **P3-3-3** LOW (lus : `ReconciliationProposals.svelte:15`, `:115`, `:200`, `:298`, `:574`, `.test.ts:260`, `:279`, `:553`,
    `:556`, `:601`, `ManualMatchModal.svelte:2`) → commentaires du geste listés, à réécrire.
- 2026-11-10 : Validation P4 ciblée (Haiku), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p4-e342.md`, pour
  cette fiche 1 M, 3 LOW, tous retenus après lecture :
  - **F1** MEDIUM, **né de la remédiation P3** (P3-3-3) : règle unique « citation de story gardée, phrase du geste réécrite » ;
    les six lignes qui citent la 8-5a-base (dont l'en-tête de catalogue) et les autres commentaires classés explicitement ;
    survivants réduits au libellé d'audit (quatre locales) et au faux positif `admin-manual.tex:2100`, cinq lignes ;
  - **F2** (lus : `failed-proposal-label.ts:298`, `:327`, `.test.ts:329`, `:341`) → ajoutés à la liste ;
  - **F3** (lus : fr `messages.ftl:1507`, de/en/it `:1440`, et le libellé d'audit fr `:3150`, de `:3077`, en `:3083`, it `:3078`,
    sans en-tête) → « commentaire d'en-tête » du libellé d'audit retiré, en-tête réel classé ;
  - **F4** (lu : `i18n-repli-catalogue.ts:82-104`, `startsWith`) → préfixes écrits en entier.
- 2026-11-10 : Validation P5 ciblée (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p5-e342.md`, pour
  cette fiche 1 M, 2 LOW, tous retenus :
  - **P5-1** MEDIUM, **né de la remédiation P4** (F1) : l'exemple de réécriture gardait « réconciliation manuelle » ; remplacé
    par une forme sans l'ancien nom, et ses trois équivalents d'en-tête de/en/it. **Garde rejouée** sur les quatre formes
    (`echo … | grep -ciE '<motif de l'AC-6>'`) : 0 ligne chacune ; les cinq survivants annoncés restent justes ;
  - **P5-2** (lu : `reconciliation-manual.spec.ts:54`) → ajouté à la liste « n'en citent pas » ;
  - **P5-3** : *(annotation datée de l'entrée P4, non réécrite)* « les six lignes qui citent la 8-5a-base » se lit « cinq
    commentaires de code et l'en-tête de catalogue en quatre locales, soit neuf lignes ».

## Dev Agent Record

### Agent Model Used

Opus 5.5 (développement en autonomie, clé `e3403`, worktree `/home/gcorbaz/devel/kesh-e3403`, base `7f817792`).

### Completion Notes List

**Étape 0 (T0)** — prémisses relues au commit `7f817792` (`git show 7f817792:<fichier> | sed -n …`) :
- `ManualMatchModal.svelte:87-93` (filtre `['5', '6', '7']` + `a.id !== bankLedgerAccountId`) et `:168` (repli du titre
  « Réconciliation manuelle ») ; `TransactionSplitModal.svelte:74-76` (même filtre, sans exclusion du compte de la banque) ;
  `ReconciliationProposals.svelte:671-681` (`<TransactionSplitModal …>` sans `bankLedgerAccountId`) ;
  `AccountAutocomplete.svelte:200-207` (`a.active && a.postable`) : confirmés.
- **LOW P6-F2** (validation P6) : le repli `ManualMatchModal.svelte:168` est réécrit (« Écriture manuelle ») et tenu égal au
  catalogue par `ecriture-manuelle-i18n.test.ts`.
- **Compte de l'E2E** : `crates/kesh-db/src/test_fixtures.rs:131` `("2000", "Capital CI", "Liability")`, inséré sans `role`
  (`INSERT INTO accounts (company_id, number, name, account_type)`) ; `helpers/api-fixtures.ts:229` relie `1000` au compte
  bancaire. L'E2E prend donc `2000`.
- **Garde de l'AC-6 avant** : 97 lignes (catalogues 12 × 4, `user-manual.tex` 18, `admin-manual.tex` 1, `frontend/src` 29,
  `frontend/tests` 1), `kesh-gate-logs/e3403/garde-ac6-avant.log` — égale à la fiche.
- **Garde de l'AC-6 après** (depuis `9387e0c3`) : **5 lignes**, exactement les survivants annoncés —
  `audit-log-action-reconciliation-manual-matched` de `:3080`, en `:3086`, fr `:3153`, it `:3081` (numéros décalés par les
  clés d'avis ajoutées) et `admin-manual.tex:2100` ; `garde-ac6-apres.log`.

**Rouges (T1)** — `kesh-gate-logs/e3403/rouges.log` : Vitest du dossier `reconciliation`, 23 rouges sur 274 (6 fichiers),
dont le module `reconciliation-counterparties` absent. Vert d'avance, comme la fiche l'annonçait : « Éclater n'offre pas le
compte de la banque ». Rouge **préexistant** révélé : le repli de `reconciliation-labels-more-capped` (« rattachées à un
règlement ») divergeait du catalogue fr (« rattachées à une écriture existante ») — corrigé (C-e3403-6). ⚠️ **L'E2E AC-7
n'a pas été joué rouge au commit de départ** : le build lancé pour cela a été arrêté avant d'avoir pris le verrou, pour ne
pas construire un mélange de sources pendant l'écriture de l'implémentation ; son rouge est établi par M1 rejouée sur l'E2E.

**Mutations** (`kesh-gate-logs/e3403/mut/*.log`, `scripts/mut-e3403.py`, motif vérifié unique, restauration
`git checkout --` + `touch`) — Vitest du dossier `reconciliation` :
- M1 (filtre de classes rétabli dans `counterpartyAccounts`) : 11 rouges, dont AC-2 (deux dialogues et fonction pure),
  AC-3, et les avis (1100, 2000 plus proposés) ;
- M2 (exclusion du compte de la banque retirée) : 5 rouges, dont AC-2 et « Éclater n'offre pas le compte de la banque » ;
- M3 (prop non passée à *Éclater*) : 1 rouge, « Éclater reçoit le compte de la banque » (`ReconciliationProposals.test.ts`) ;
- M4 (`Receivable` seul) : 3 rouges (cas `Payable` du dialogue, de l'éclatement et de la fonction pure) ;
- M4-bis (texte client pour `Payable`) : 2 rouges (texte attendu du cas `Payable`) ;
- M5 (repli `failed-rule-masked-payment-batch` à l'ancien libellé) : 3 rouges — le test neuf **et** deux tests déjà
  présents de `failed-proposal-label.test.ts` (T-b28, « les replis des clés neuves égalent le catalogue fr-CH »). ⚠️ La
  fiche (P2-3-3) disait qu'aucun test existant ne voyait cet écart : faux pour les quatre `failed-rule-masked*` et les deux
  `supplier-*`, déjà comparées par T-b28 ; vrai pour l'avis du lot, `more-capped` et le bouton, que seul le test neuf couvre.
- M1 sur l'E2E : voir *Gates*.

**Gates au dernier commit de code** (`ca2f4a17` ; les commits suivants ne portent que `_bmad-output/`) :
- bases `kesh_e3403`, `kesh_e2e_e3403` recréées à neuf ;
- `scripts/test-fast.sh --ci` (au commit `b25b77b3`, code Rust et catalogues identiques à `ca2f4a17`) : fmt, clippy,
  **nextest 5138 exécutés, 5138 passés, 12 ignorés** — `gate-test-fast.log` ;
- `cargo build --workspace --all-targets` : vert — `gate-build-workspace.log` ;
- `scripts/gate-ci.sh kesh_ci_e3403` : **5145 passés, 0 échoué, 16 ignorés** — `gate-ci.log` ;
- frontend : `npm run check` 0 erreur (29 avertissements, tous dans des fichiers que la story ne touche pas),
  `lint-i18n-ownership` vert, `npm run build` vert ; `npm run test:unit` complet : **1er passage 2 rouges** (le compteur
  de sites de `i18n-keys.test.ts`, 2922 → 2924 pour les deux avis ; la garde `e2e-selecteurs-traduits` sur un
  `getByRole('heading', { name: 'Écriture manuelle' })` de l'E2E neuf), corrigés au commit `ca2f4a17` ; **2e passage :
  224 fichiers, 2919 tests, tous verts** — `gate-front-vitest.log`, `gate-front-vitest-2.log` ;
- **E2E complet** (`e2e-r1.log`, base vide, port 3343, runner avec `KESH_INBOX_DIR` et `KESH_DOCUMENTS_DIR`) :
  **386 passés, 8 échecs, 17 ignorés** ; `reconciliation-manual-entry.spec.ts` vert. Les huit, nommés contre
  `docs/testing.md` § *Les échecs attendus* : les sept de la KF-029 (#97 — `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et
  `backup-restore-new-install.spec.ts:140` (AC20). Ce huitième **n'est pas** la pollution d'état « variable » du tableau :
  rejoué seul sur base vide **avec** `KESH_E2E_CAPTURES_DIR` posé (je l'avais posé pour toute la suite), il rougit encore
  (`e2e-r2-backup-restore-seul.log` : `admin-restore-files-restored` introuvable) ; rejoué seul **sans** cette variable,
  ses trois tests passent (`e2e-r4-backup-restore-sans-captures.log`). C'est le mode « captures » de cette spec qui la fait
  rougir, sans rapport avec la story : **#994** (KF-058). (Effet de bord du même choix : toutes les specs qui capturent ont
  écrit leurs images dans `kesh-gate-logs/e3403/captures/` ; seules les `e3403-*` sont de la story.)
- **M1 sur l'E2E** (`mut/M1-e2e.log`, `mut/M1-e2e-run.log`) : build muté, base vide, spec seule → **rouge**, pour la bonne
  raison (`reconciliation-manual-entry.spec.ts:125`, l'option `2000` absente : `Expected: 1, Received: 0`) ; fichier
  restauré, frontend reconstruit ensuite (`gate-front-build-3.log`). Un premier essai (r3) avait échoué en `rc=127` sur un
  script E2E que j'avais cassé pendant le run : il ne compte pas.
- **Captures** (thème clair, base E2E jetable, `kesh-gate-logs/e3403/captures/`) : `e3403-1-ligne-bouton-ecriture-manuelle.png`,
  `e3403-2-dialogue-liste-toutes-classes.png` (1100, 1105, 2000, 3000, 4000 proposés ; 1000, compte de la banque, absent),
  `e3403-3-dialogue-compte-classe-2.png`, `e3403-4-dialogue-avis-debiteurs.png`, `e3403-5-eclater-produits-et-avis.png`
  (ligne 3000 et ligne 1105 avec son avis), et `e3403-ecriture-manuelle-classe-2.png` (prise par l'E2E). Le compte 1105
  « Débiteurs (capture) », au rôle `Receivable`, est créé par le script de capture (`scripts/captures-e3403.mjs`, hors dépôt).
- **Constat voisin, non traité** : dans *Éclater*, l'indicateur d'équilibre n'affiche que « Balance » (catalogue fr
  `reconciliation-split-balance-indicator = Balance`, même clé pour les deux états ; le repli détaillé n'est jamais
  affiché) — visible sur `e3403-5`. Préexistant ; corrigé ici, il aurait rouvert tous les gates : **#993**.

**Écarts à la consigne** : les tests neufs de `TransactionSplitModal.test.ts` ont été ajoutés par un `cat >>` (ajout en fin de
fichier, non un remplacement), le reste par l'outil Edit ; l'outil ne manquait de rien, c'était un raccourci. Mon script de
mutation, au premier essai de M1 sur l'E2E, a restauré **tous** les fichiers modifiés (`git diff --name-only`), dont cette
fiche non commitée : ce Dev Agent Record a été réécrit ; le script ne restaure plus que le fichier muté.

### Change Log (développement)

- 2026-11-10 : développement (Opus 5.5). Commits `612057f0` (rouges), `ecb9ffd3` (implémentation), `9387e0c3` (manuel,
  CHANGELOG), `b25b77b3` (types des tests), `ca2f4a17` (compteur i18n, sélecteur E2E), `f9480a34` (registre, prompt P1),
  et celui-ci (fiche, `Status: review`). Choix neufs : C-e3403-1 à C-e3403-7. Aucune migration, aucune ligne exécutable de
  `crates/` modifiée (deux commentaires).
- 2026-11-11 : **revue de code P1** (Opus, trois lentilles B, E, A ; rapports `kesh-gate-logs/e3403-review-p1-{B,E,A}.md`) :
  **0 C, 0 H, 0 M, 15 LOW** (B 6, E 5, A 4). Remédiation (Opus 5.5) au commit `584a1183` (code, tests, catalogues,
  manuel), et `15b41959` (fiche, registre) :
  - **corrigés** : B1-1 (`claimRoleOfId`, seule règle d'affichage de l'avis) ; B1-2 = E1-3 = A1-3 (Vitest : l'avis suit
    sa ligne après suppression, deux lignes débiteurs, `colspan` 4 et 5 contre l'en-tête) ; B1-3 = E1-4 = A1-1 (ancre
    positive de l'E2E avant l'absence du compte de la banque ; M2 à rejouer sur l'E2E) ; B1-6 (le test TVA prouve
    d'abord le choix) ; B1-4 (de « Die Rechnung bliebe ») ; A1-4 (en « This is the … account ») ; B1-5 (avis créanciers
    reformulé, quatre locales, repli, test, manuel — décision de l'orchestrateur, C-e3403-8) ; E1-1 (manuel : la liste
    ouverte sans frappe ne montre que les vingt premiers comptes) ; A1-2 (#981, #993, #994 cités ; `:222` annotée) ;
  - **issues demandées** : E1-1 (comportement : menu sans frappe limité aux vingt premiers comptes), E1-2 (avis
    indépendant du sens de la transaction, C-e3403-9), E1-5 (compte choisi sorti de la liste, champ vide : #271,
    fermée, ne vise que les `<select>` de configuration — non couvert) ;
  - symptômes grepés (anciens textes de l'avis, « Sie bliebe », « This account is the accounts ») : seul reste
    `user-manual.tex:3957`, la FAQ fournisseur, déjà sans ambiguïté (« saisissez, puis validez »), laissée telle quelle ;
  - gates de la remédiation **tels qu'ils ont tourné** : `cargo test -p kesh-i18n` 41 passés, 0 échec ;
    `textes_coherents` 23 passés ; `lint-i18n-ownership` vert ; Vitest du dossier `reconciliation` 16 fichiers,
    290 tests verts, `npm run check` 0 erreur (hors verrou, avant la séquence). **Différés par la pause des commandes
    lourdes** (`PAUSE-HEAVY`, priorité à mybibli, 2026-11-11) : `npm run check` sous verrou, Vitest complet, `npm run
    build`, M2 sur l'E2E, E2E complet — à jouer à la levée de la pause, au commit `584a1183`.
  - Suite : passe ciblée P2 (Haiku), prompt `34-3-ecriture-manuelle-review-prompt-p2-ciblee.md`.

### File List

- `frontend/src/lib/features/reconciliation/reconciliation-counterparties.ts` (neuf) et `.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/CounterpartyClaimNotice.svelte` (neuf)
- `frontend/src/lib/features/reconciliation/ecriture-manuelle-i18n.test.ts` (neuf)
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte`, `.test.ts`
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte`, `.test.ts`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `.test.ts`
- `frontend/src/lib/features/reconciliation/ModalSuccessStub.test.svelte`
- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, `.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts`, `reconciliation.types.ts` (commentaires)
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` (commentaire)
- `frontend/src/lib/shared/i18n-keys.test.ts` (compteur de sites 2922 → 2924)
- `frontend/tests/e2e/reconciliation-manual-entry.spec.ts` (neuf), `reconciliation-manual.spec.ts` (commentaire)
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`
- `crates/kesh-api/src/routes/reconciliation.rs`, `crates/kesh-reconciliation/src/manual.rs` (commentaires seuls)
- `docs/manual/fr/user-manual.tex` (PDF non régénéré : à l'intégration, AC-8)
- `CHANGELOG.md` (`## [0.17.6] — Non publié`, *Changé*)
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e3403.md` (neuf), `34-3-ecriture-manuelle-review-prompt-p1.md`
  (neuf), cette fiche, `sprint-status.yaml`
