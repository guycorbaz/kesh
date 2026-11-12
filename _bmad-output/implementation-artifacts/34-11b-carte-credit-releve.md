# Story 34-11b : le relevé d'une carte de crédit, sa réconciliation et son prélèvement mensuel (#972)

Status: done

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e341.md`,
C-e341-22 à C-e341-28, C-e341-35 à C-e341-37. Suit **34-11a1** et **34-11a2** (le compte carte), **34-3** (« Écriture manuelle » sur tout compte
imputable sauf celui de la banque) et **34-4** (ordre 34-3 → 34-4 → 34-11b). Commits en `refs #972`, `refs #823` ; à la PR : `closes #972`,
`refs #823` (C-e341-37 : #823 n'est résolue qu'en partie).*

## Story

En tant que titulaire d'une carte de crédit tenue comme un compte, je veux faire entrer son relevé mensuel (saisi depuis le PDF, ou importé en
CSV), réconcilier chaque achat, et passer le prélèvement mensuel **une seule fois** comme un virement de la banque vers la carte, pour que les
achats tombent à leur date et que la dette de la carte figure juste au bilan.

## Décisions de Guy (#972, fichier d'epic, 2026-11-10)

- Relevés « **pdf principalement, mais je dois pouvoir avoir des csv** » : deux chemins, **sans lecture automatique d'un PDF** — la **saisie
  manuelle du relevé** (#816) à partir du PDF, chemin principal, et l'**import CSV** par un profil.
- Le **prélèvement mensuel est un virement entre les deux comptes** : écriture côté banque, rattachement côté carte à cette écriture existante
  (#671). À terme Kesh **propose** l'appariement — ⛔ il ne le fait jamais seul (« un appariement automatique propose, il ne crée jamais »).
- Le PDF du relevé se joint comme pièce **s'il existe déjà un moyen de le faire**.
- **Manuel utilisateur pas à pas.**

## Ce que le code dit (prémisses établies)

- **La saisie manuelle n'a pas besoin d'IBAN** : `routes/bank_statements_manual.rs:1-28` (module), aucune lecture de `bank_accounts.iban`
  (inventaire de la 34-11a1, C4). **Soldes signés librement** (`kesh-core/src/bank_imports/manual.rs:480-485`, « Règle 5 (soldes : signe libre) ») ;
  lignes à montant positif et sens `credit`/`debit` (`:520-531`) ; contrôle `ouverture + crédits − débits = clôture` (`:536-543`). Une dette de
  carte s'écrit donc en solde **négatif**, un achat en **débit**, un remboursement ou le paiement reçu en **crédit**.
- **Le signe tient jusqu'au bilan** : le solde d'un compte lié est `SUM(debit) − SUM(credit)` de son compte du grand livre
  (`repositories/bank_accounts.rs:1755-1790`, `BALANCES_SQL`), négatif pour un passif endetté ; le contrôle K3 compare ce solde au solde
  d'arrivée du relevé (`ledger_at`, `:1758-1764`) : une dette de 1'234.50 saisie `-1234.50` concorde.
- **Le profil CSV garde le signe d'une colonne unique** (`kesh-import/src/csv/profile.rs:18-21` : « en colonne unique, le signe du fichier est
  gardé tel quel ») ; en débit/crédit séparés, `debit_sign` règle la colonne Débit. **Aucun réglage n'inverse une colonne unique** : un émetteur
  qui écrit ses achats en positif dans une seule colonne s'importerait à l'envers. L'ajouter demande une colonne et un bump (`debit_sign` a coûté
  une migration breaking, `20261011000001_bank_profiles_debit_sign.sql:1-6`) : hors de cette story, **issue demandée** (C-e341-25).
- **Le rattachement à une écriture existante** (v017-3a, #671) : `reconciliation_link::link_entry_in_tx` (`reconciliation_link.rs:753`), candidats
  `find_entry_candidates` (`:640`), règle unique `entry_candidates_sql` (`:583-606`), conditions 1 à 6 (`:481-497`).
- ⛔ **LE BLOCAGE, établi au code** : la **condition 4** — « `e` n'est rapprochée d'aucune transaction » —, écrite
  `NOT EXISTS (SELECT 1 FROM bank_transactions bt WHERE bt.matched_entry_id = e.id)` (`:596`). Le virement banque → carte est **une** écriture,
  `D <passif de la carte> / C <compte de la banque>`, déjà rapprochée de la transaction **de la banque** quand on veut y rattacher celle **de la
  carte**. Elle est donc **refusée**, et le geste que demande #972 est impossible aujourd'hui. Symétrique si la carte est rapprochée d'abord.
- **L'invariant que la condition 4 tient** : « une écriture, une transaction » (`reconciliation_link.rs:23-28`, `:303-330`), qu'aucune contrainte
  `UNIQUE` ne défend. Ses consommateurs, cherchés au symptôme (`git grep -n "matched_entry_id = ?" -- crates/*/src`, 12 lignes) :
  - `reconciliation_cancel.rs` : l'annulation d'un rapprochement contre-passe une « écriture propre » (`ReconciliationKind::Entry`, `:611-614`), et
    avant cela refuse si **une autre** transaction pointe la même écriture (`settlement_cancellation.rs:135-157`, `MatchedBankTransaction`).
    Avec deux transactions sur une écriture, **chacune refuserait l'annulation de l'autre** : impasse (C-e341-23).
  - `journal_entries::reverse_in_tx` et la garde de modification (`MATCHED_BANK_TRANSACTION`, `journal_entries.rs:2141`, `:2416`, `:2570`) :
    refusent tant qu'une transaction pointe l'écriture — juste, inchangé.
  - `repositories/bank_accounts.rs:1772-1786` (`BALANCES_SQL`) : filtre `bt.bank_account_id = ba.id` — par compte, sain.
  - `invoices.rs:1622`, `supplier_invoices.rs:1311`, `:1438` : écritures de **règlement** (une seule ligne de banque par construction : jamais
    candidates de deux comptes) — sains, justifié en AC-3.
  - les six `SET matched_entry_id = ?` de `routes/reconciliation.rs` (`:2826`, `:3389`, `:3769`, `:4248`, `:5077`, `:5541`) lient une écriture
    **qu'ils viennent de créer** — sains.
- **Le côté banque** : la transaction de débit se comptabilise par « Écriture manuelle » sur le compte de dette de la carte — possible depuis la
  **34-3**, qui ouvre le dialogue à tout compte imputable sauf celui de la banque (classe 2 comprise).
- **Les achats** se comptabilisent par « Écriture manuelle » ou par une règle de rapprochement (`routes/reconciliation_rules.rs`) : rien à écrire,
  la règle lit la transaction, non l'IBAN du compte.
- **Joindre le PDF** : aucun moyen n'existe pour un relevé ni pour une écriture (`git grep -n "source-document" -- crates/kesh-api/src/lib.rs` : les
  seules routes sont celles des factures fournisseur, `lib.rs:696`, `:710`, `:1112`). Rien n'est joint ; **issue demandée** (C-e341-26).

## Choix (détail au registre)

- **C-e341-22 — la condition 4 devient « au plus une transaction par compte du grand livre »** : `e` n'est rapprochée d'aucune transaction **d'un
  compte bancaire lié au même compte du grand livre** que celui de `t`. Une écriture de virement, qui meut deux comptes du grand livre, accepte
  donc une transaction de chaque côté, et pas plus ; deux comptes bancaires qui **partagent** un compte du grand livre restent tenus à une seule
  (le double comptage que la condition empêchait). La condition 3 (`M = t.amount` sur le compte du grand livre de `t`) est inchangée et garantit
  que chaque côté se rattache à **son** mouvement. Écartés : un statut « virement interne » sans écriture côté carte (la carte resterait sans
  réconciliation) ; deux écritures (double comptage du transit, à solder à la main).
  **Précisé à la validation P1 (4.4)** — le lien d'un compte bancaire au grand livre se modifie (`PATCH`, `NULL` compris), la condition ne peut donc
  pas se fier au seul lien **actuel** de l'autre côté. Forme retenue : `e` est rattachable à `t` si **aucune** transaction déjà liée à `e` n'est
  (i) du **même compte bancaire** que `t`, ni (ii) d'un compte bancaire dont le lien est `NULL` ou **égal** au grand livre de `t`, ni (iii) d'un
  compte bancaire dont le grand livre actuel ne porte **aucune** ligne de `e` (un compte relié ailleurs depuis : son lien ne prouve plus quel
  mouvement il a rapproché) — **au plus deux** transactions par écriture en tout. Une seule requête, la même pour la liste et le geste.
- **C-e341-23 — l'annulation d'un côté ne défait que ce côté** : quand une **autre** transaction, d'un **autre** compte du grand livre, pointe la
  même écriture, l'annulation remet `matched_entry_id` à `NULL` pour la seule transaction annulée (statut `pending`, trace
  `reconciliation.cancelled` avec `unlinkOnly: true`) et **ne contre-passe rien** ; l'écriture reste celle de l'autre côté. L'annulation du dernier
  côté suit le chemin d'aujourd'hui. Écarté : refuser (impasse décrite ci-dessus). **Précisé à la validation P1** : (4.6) le retrait d'un seul lien
  n'écrit rien au journal, il n'est donc **pas** refusé par un exercice clos ni par un lettrage en période close (`reconciliation_cancel.rs:545-557`
  ne s'applique qu'au dernier côté) ; le compte clos, lui, refuse toujours (`ensure_open_in_tx`, `:541`) ; (4.7) la trace écrit `unlinkOnly: true` et
  `reversalJournalEntryId: null`, en camelCase comme ses autres détails (`:671-676`) ; (4.2) la réponse `CancelReconciliationResponse` porte
  `reversalJournalEntryId: Option<i64>` (`routes/reconciliation.rs:5771`), `null` pour ce cas — additif en JSON, rupture de type côté TypeScript
  (`reconciliation.types.ts:199`), traitée.
- **C-e341-35 — une seule définition de « l'autre côté légitime »** (validation P1, 4.1, HIGH ; **récrite à la validation P2**, 5.1 et 5.2).
  Une autre transaction liée à `e` est « l'autre côté légitime » de `t` si elle ne tombe sous aucun des cas (i), (ii), (iii) de C-e341-22. La
  définition est **une** fonction de `kesh-db`, et elle **décide en tête**, avant les rangs 1 à 5 de `settlement_entry_cancel_blocker`
  (`settlement_cancellation.rs:100-170` : rang 2 `FiscalYearClosed` `:108-113`, rang 2 bis lettrage figé `:118-127`, rang 3 `:133-157`, puis
  rangs 4 et 5) : **si un autre côté légitime existe, l'annulation de `t` est `unlinkOnly`, et seuls s'appliquent le rang 0 (transaction non
  rapprochée) et le compte clos** — aucun rang 1 à 5, puisqu'aucune écriture n'est touchée. Sinon, la chaîne d'aujourd'hui, inchangée.
  **Elle a besoin de `t`** : la transaction concernée — liée (annulation) ou **prospective** (candidate) — passe en argument
  de `entry_cancel_blocker` et de la garde de règlement **en remplaçant** le paramètre existant `unlinking: Option<i64>` (`settlement_cancellation.rs:82-86`,
  exemption étroite du rang 3) par `concerned: Option<&BankTransaction>` (validation P3, 5.3) : l'exemption étroite s'applique quand `concerned` est
  liée à l'écriture (son identifiant est exclu, comme aujourd'hui), la décision « autre côté » lit son compte bancaire ; un seul paramètre, que M7 et
  M7c désignent. **`None` garde son sens strict, inchangé** : les
  écrans de facture qui l'appellent ainsi (`invoice_settlements_write.rs:795`, `settlement_cancel_blocker_unlinking(…, None)` ; dévalidation
  fournisseur, `supplier_invoices.rs:2462`) contre-passent l'écriture et **doivent** garder le rang 3 strict. Appelants : `cancel_blocker`
  (`reconciliation_cancel.rs:250-262`, qui sert `cancel_in_tx` **et** `get_view`, `:410`) passe `Some(&bt)` ; les candidates
  (`routes/reconciliation.rs:6468-6475`) passent `Some(&t)`, la transaction dont on liste les candidates ; le commentaire
  `reconciliation_cancel.rs:276-279` (« le rang 3 ne peut pas sortir ») est récrit.
- **C-e341-36 — les textes devenus faux sont corrigés ici** (validation P1, 4.3 ; ordre 34-3 → 34-4 → 34-11b) : le manuel (`user-manual.tex:2915-2921`,
  `:2991`, et ce que la 34-3, AC-8, y aura écrit sur le second côté d'un virement), le message du cas (b) de la 34-4 (« rejetez-la » →
  « rattachez-la à cette écriture »), ses quatre locales et ses tests. Les fiches 34-3 et 34-4 (autre groupe) ne sont **pas** modifiées : la
  34-11b corrige ce que leur code et leur manuel auront produit.
- **C-e341-37 — #823 en partie** : son objet est triple (lot `pain.001` de plusieurs factures, paiement groupé, virement entre comptes propres) ;
  la 34-11b ne résout que le troisième : `refs #823` à la PR, et un commentaire à demander sur #823 (« le virement est résolu par la 34-11b ;
  restent le lot et le paiement groupé »).
- **C-e341-24 — aucun appariement d'office** : la liste des candidats du rattachement montre l'écriture de virement (même montant, jours d'écart)
  et l'utilisateur la choisit. Aucun rattachement automatique, aucune proposition nouvelle dans cette story.
- **C-e341-25 — CSV** : les profils existants servent tels quels ; le manuel dit les deux conventions qui marchent (débit/crédit séparés, ou colonne
  unique où les achats sont négatifs). L'inversion d'une colonne unique : issue.
- **C-e341-26 — le PDF** : non joint (aucun moyen n'existe) ; le manuel conseille de noter la référence du relevé dans *Numéro de relevé*.
  Deux issues : joindre une pièce à un relevé saisi ; lire un relevé PDF (hors epic, sur décision de Guy).
- **C-e341-27 — pas de règle par compte** : une règle de rapprochement s'applique à tous les comptes ; rien à changer pour la carte.
- **C-e341-28 — la portée de la condition 4 neuve** vaut pour **tout** rattachement à une écriture, pas seulement pour une carte : le virement
  entre deux comptes bancaires ordinaires (épargne → courant) en bénéficie aussi. Le manuel le dit.

## Acceptance Criteria

1. **AC-1 — la règle** : `entry_candidates_sql` porte la condition 4 de C-e341-22 ; **un seul texte** (liste et geste), doc-comments du module et
   de l'invariant (`:23-28`, `:303-330`, `:481-497`) récrits : « au plus une transaction par compte du grand livre ».
2. **AC-2 — le virement** (test d'intégration, deux comptes : banque liée à un actif, carte liée à un passif) : la transaction de débit de la banque
   (−500) est passée par « Écriture manuelle » sur le passif de la carte ; la transaction de crédit de la carte (+500) liste cette écriture parmi
   ses candidats et s'y rattache ; les deux transactions sont `reconciled`, **une** écriture au journal, soldes : banque −500, carte +500 (dette
   réduite). Même chose dans l'ordre inverse (carte d'abord).
3. **AC-3 — ce qui reste refusé** : une troisième transaction (même compte du grand livre que l'une des deux) ne peut pas s'y rattacher ; deux
   comptes bancaires qui partagent un compte du grand livre ne s'y rattachent pas deux fois ; les écritures de règlement restent rattachables une
   fois (tests existants de `reconciliation_link_entry_e2e.rs` et `reconciliation_link_e2e.rs` **inchangés et verts**).
4. **AC-4 — l'annulation** (C-e341-23) : annuler le côté carte d'un virement rattaché des deux côtés → carte `pending`, banque toujours
   `reconciled`, **aucune** contre-passation, écriture intacte ; puis annuler le côté banque → chemin d'aujourd'hui (contre-passation). Et dans
   l'ordre inverse. La trace dit `unlinkOnly`. Le verrou : la transaction annulée `FOR UPDATE`, puis la ligne de l'écriture `FOR UPDATE` avant la
   lecture des autres liens (même discipline que `link_entry_in_tx`, `:303-330`) ; deux annulations simultanées des deux côtés ne laissent pas
   l'écriture contre-passée avec un lien vivant (test à deux connexions, ou raisonnement écrit au doc-comment s'il n'est pas déterministe — dit tel quel).
   **Forme de la lecture des autres liens** (validation P2, 5.3) : après le `FOR UPDATE` de l'écriture, elle est **ordinaire**, jamais verrouillante :
   une lecture `FOR UPDATE` de la ligne `bank_transactions` de l'autre côté attendrait l'annulation concurrente, qui la tient en exclusif (étape 1 de
   `cancel_in_tx`) et attend l'écriture — un interblocage à chaque double annulation. Écrit au doc-comment.
   **Exercice clos** (C-e341-23, 4.6) : l'écriture du virement dans un exercice clos ; annuler le côté carte → accepté, `unlinkOnly` ; annuler ensuite
   le dernier côté → refusé `FiscalYearClosed` comme aujourd'hui. **Réponse** : `reversalJournalEntryId: null` pour un retrait seul.
4b. **AC-4b — ce que l'écran voit** (C-e341-35, 4.1) : tests HTTP — `GET …/transactions/{id}` (vue d'annulation) d'un côté d'un virement rattaché
   des deux côtés rend `cancellable: true` et dit « seul le lien est retiré » ; `GET …/entry-candidates` de la transaction de la carte rend l'écriture
   de virement **sans** `cancelBlockedBy`. Vitest : `LinkSettlementModal.svelte` n'exige pas d'accusé pour elle (`:154`, `:159`) ;
   `CancelReconciliationDialog.svelte` affiche la confirmation (`:73`, `:155`) et un message de succès distinct (« le lien est retiré ; l'écriture
   reste celle de l'autre côté »). Un cas encore bloqué (comptes qui partagent le grand livre) garde son motif. **Exercice clos** (validation P2,
   5.1) : l'écriture du virement dans un exercice clos ; la vue d'annulation du côté carte rend `cancellable: true` et `unlinkOnly: true` (et non
   `FiscalYearClosed`) ; la candidate de la transaction de la carte rend `cancelBlockedBy: null`. **Le champ** (5.4) : la vue d'annulation porte
   `unlinkOnly: boolean` à côté de `cancellable` (Rust, `reconciliation.types.ts`, `api-external.md`), et c'est lui qui choisit le texte de la
   confirmation. **Non-régression** (5.2) : une facture client dont l'écriture de règlement est rapprochée reste **non annulable** depuis sa fiche
   (rang 3 strict sous `None`), et la dévalidation fournisseur de même.
4c. **AC-4c — les comptes reliés** (C-e341-22, 4.4) : compte bancaire A lié à L, une transaction de A rattachée à `e` ; A délié (`NULL`) → une
   transaction d'un compte B lié à L ne se rattache **pas** à `e` ; A relié à L2 (sans ligne dans `e`) → idem.
5. **AC-5 — le relevé saisi d'une carte** (E2E, chemin principal) : sur une carte (34-11a2) avec solde d'ouverture *Dette* 1'000 (34-2), un relevé
   saisi à la main : ouverture `-1000.00`, deux achats en débit (120.00, 80.00), le paiement reçu en crédit (1000.00), clôture `-200.00` → accepté ;
   chaque achat passé par « Écriture manuelle » sur une charge ; le paiement rattaché à l'écriture de virement de la banque ; le solde de la carte
   dans la liste vaut `-200.00` et égale le solde du relevé (pas d'écart affiché) — le relevé a `periodTo` **postérieur ou égal** à la date des quatre
   écritures (`ledger_at` se borne à `period_to`, `bank_accounts.rs:1755-1764`). ⚠️ La base E2E porte déjà des écritures : la portée
   `SingleBankLedger` de la 34-2 (société **sans** écriture) n'y est pas exercée ; elle l'est par les tests de la 34-2 — la dépendance n'est pas
   couverte de bout en bout par cet E2E (validation P1, 4.8).
6. **AC-6 — le relevé CSV d'une carte** : un fichier CSV synthétique à colonnes débit/crédit séparées s'importe sur la carte par un profil, et ses
   achats arrivent en débit (test d'intégration de la route d'import, fixture synthétique sous `crates/kesh-api/tests/fixtures/`).
7. **AC-7 — l'écran** : la boîte de rattachement (v017-3a) liste l'écriture de virement pour la transaction de la carte ; rien n'est rattaché sans
   clic (Vitest + E2E de l'AC-5).
8. **AC-8 — manuel pas à pas** : section neuve *Tenir une carte de crédit* (`user-manual.tex`, près de *Comptes bancaires*) : créer le compte de
   dette (par ex. un 2030 *Carte de crédit* sous le 20 — le numéro est un exemple, Kesh n'en déduit rien) ; déclarer la carte (34-11a2) avec son
   solde d'ouverture *Dette* (34-2) ; chaque mois, saisir le relevé depuis le PDF (signes : la dette en négatif, les achats au débit, le paiement au
   crédit) ou l'importer en CSV (deux conventions) ; réconcilier les achats (« Écriture manuelle », règles) ; le prélèvement : côté banque
   « Écriture manuelle » sur le compte de la carte, côté carte *Rattacher à une écriture* ; annuler un côté ; ce que la carte ne fait pas. Un exemple
   chiffré **synthétique**. PDF régénéré, aplati.
9. **AC-9 — `api-external.md`** : la condition du rattachement, l'annulation d'un seul côté, `reversalJournalEntryId` nullable (`:691`), le détail
   `unlinkOnly` de la trace.
10. **AC-10 — les textes du second côté** (C-e341-36 ; **transmission de la 34-4**, `kesh-gate-logs/e34-transmission-34-11b.md`, validation P2 du
   groupe e342, commit `73c365af`). Après la 34-11b, `matchedElsewhereCount` est un sous-ensemble de `entryCount` (même constructeur de requête, aucune
   copie). **Définition de `m`** (validation P3, 5.2 c) : `m` = le nombre d'écritures rendues par `find_entry_candidates` pour `t` dont **au moins une
   transaction est déjà liée** — rien d'autre (la définition de la 34-4, « autre compte dont le grand livre diffère », compterait un compte relié
   ailleurs que le cas (iii) refuse, et `k − m` pourrait devenir négatif). Les **clés** portent le préfixe de la 34-4, `reconciliation-labels-already-booked-…`
   (5.2 a) ; les tests de la 34-4 qui assertent que le texte de `matchedElsewhereCount` ne diffère de celui des candidates que par ses prédicats, et ceux de
   `matchedElsewhereCount` seul, sont **adaptés** à la nouvelle définition (5.2 b). Avec `k = settlementCount + entryCount` et `m` ainsi défini, l'avis de
   la 34-4 devient :
   - (a) si `k − m > 0` et `m = 0` : le texte (a) de la 34-4, inchangé ;
   - (b′) si `m > 0` et `k − m = 0` : clé neuve `…-already-booked-transfer` — « Cette transaction est probablement l'autre côté d'un virement entre vos
     comptes : l'écriture de ce virement est déjà rapprochée d'une transaction d'un autre de vos comptes bancaires. Rattachez-la à cette écriture
     (*Rattacher à une écriture existante*) plutôt que de passer une écriture : le virement serait compté deux fois. » ;
   - (c′) si `m > 0` et `k − m > 0` : clé neuve `…-already-booked-transfer-among` — le texte (a), clé `-one` ou `-other`, puis « L'une d'elles est
     l'écriture d'un virement déjà rapprochée sur un autre de vos comptes bancaires : c'est probablement celle-ci qu'il faut rattacher. » ;
   - rien si `k = 0` ; la confirmation reste déclenchée par `k > 0`.
   Les clés `…-already-booked-elsewhere` (b) et `…-already-booked-combined` (c) sont **retirées** des quatre locales et de leurs replis. Vitest : quatre
   cas — aucun signal, (a), (b′), (c′) ; mutation **M11** : la condition (b′) testée sur `m > 0` seul → rouge du cas (c′).
   Le manuel : `user-manual.tex:2915-2921` et `:2991` récrits (« le second côté d'un virement entre deux de vos comptes se rattache à l'écriture du
   virement » ; #823 reste citée pour le lot de plusieurs factures et le paiement groupé) ; **`:2913-2914`** aussi (validation P2, 5.1 : « un paiement
   d'un exercice clôturé se rattache aussi ; le dialogue dit alors que ce rattachement ne pourra pas être annulé tant que l'exercice reste clos » — faux
   pour le second côté d'un virement, dont le lien se retire même dans un exercice clos) ; ce que la 34-4 y aura écrit, aligné, et ce que la 34-3 (AC-8)
   y aura ajouté. La ligne « #823 » du § *Ce qui ne se traite pas* de la 34-4 n'est pas dans le dépôt (fiche d'un autre groupe, non modifiée) : son
   contenu, s'il passe au manuel ou au CHANGELOG, est aligné. Grep de la valeur après coup : `rejetez-la`, `already-booked-elsewhere`,
   `already-booked-combined`, `#823`, `second côté` dans `docs/manual/`, `crates/kesh-i18n/`, `frontend/src/`, `CHANGELOG.md`. PDF régénéré, aplati.

## Tâches

- [x] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : AC-2 (rouge aujourd'hui : l'écriture n'est pas candidate), AC-3, AC-4, AC-6.
- [x] **T2 — dépôt** : condition 4 (AC-1) ; la fonction unique de C-e341-35, **en tête** de la chaîne des rangs ; la signature de
  `entry_cancel_blocker` et de la garde de règlement reçoit la transaction concernée (`None` strict inchangé) ; `cancel_in_tx` prend la branche
  `unlinkOnly` quand elle le dit (seuls rang 0 et compte clos) ; `ReconciliationCancellation` porte `reversal_journal_entry_id: Option<i64>`.
  ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [x] **T3 — trace** : `unlinkOnly` et `reversalJournalEntryId: null` à `reconciliation.cancelled` ; vérifier le vocabulaire d'audit
  (`audit_log::vocabulary`) et l'écran d'audit, et les aligner s'ils lisent ces détails.
- [x] **T4 — écran** (AC-4b) : `reconciliation.types.ts:199` (`reversalJournalEntryId: number | null`), `CancelReconciliationDialog.svelte` (confirmation
  et message de succès du retrait seul), `LinkSettlementModal.svelte` (aucun faux obstacle), i18n quatre locales.
- [x] **T4b — textes du second côté** (AC-10), après la fusion de 34-3 et 34-4 dans la branche.
- [x] **T5 — E2E** `credit-card-statement.spec.ts` (AC-5, AC-7), données créées par l'API sur la base E2E, montants synthétiques.
- [x] **T6 — manuel** (AC-8), `api-external.md` (AC-9), captures `kesh-gate-logs/e341/captures/34-11b-*.png` (relevé saisi, rattachement du
  prélèvement, liste des comptes).
- [x] **T7 — CHANGELOG** : *Ajouté* (tenir une carte), *Corrigé* (un virement entre deux comptes de Kesh se réconcilie des deux côtés), `refs #972`.

### Mutations nommées

- **M1** : condition 4 rendue à l'ancienne forme → rouge AC-2.
- **M2** : condition 4 sans le filtre du compte du grand livre (toute transaction d'un autre **compte bancaire** admise) → rouge AC-3 (comptes qui
  partagent le grand livre).
- **M3** : l'annulation contre-passe même quand l'autre côté reste lié → rouge AC-4.
- **M4** : l'annulation « un côté » retire aussi le lien de l'autre → rouge AC-4.
- **M5** : la lecture des autres liens déplacée avant le verrou de l'écriture → rouge du test à deux connexions s'il existe, sinon écrit non tuée.
- **M6** : `get_view` garde l'ancienne garde (rang 3 sans exception) → rouge AC-4b (`cancellable: false`).
- **M7** : les candidates appellent encore avec `None` → rouge AC-4b (`cancelBlockedBy` non nul).
- **M7b** : la décision « autre côté » placée au rang 3 et non en tête → rouge AC-4b (exercice clos : `FiscalYearClosed` rendu).
- **M7c** : `None` traité comme « autre côté possible » → rouge de la non-régression (facture client annulable depuis sa fiche).
- **M8** : la condition (ii) sans le cas `NULL` → rouge AC-4c (compte délié) ; **M9** : sans (iii) → rouge AC-4c (compte relié).
- **M10** : le retrait seul refusé par l'exercice clos → rouge AC-4 (exercice clos).
- **M11** : la condition (b′) testée sur `m > 0` seul → rouge du cas (c′) (Vitest de l'AC-10).

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db` ; gate frontend ; E2E complet au dernier commit de code (les specs `reconciliation-*` sont le
rayon d'impact).

## Manuels

Utilisateur (AC-8), `api-external.md` (AC-9). **Prompt de revue : nommer le manuel** ; son exemple chiffré se rejoue sur une base jetable.

## Ce qui ne se traite pas — issues à demander (une par problème)

1. **Profil CSV : inverser le signe d'une colonne de montant unique** (relevés de carte qui écrivent les achats en positif) — demande une colonne et
   un bump (C-e341-25).
2. **Joindre le PDF d'un relevé saisi à la main** (aucun moyen pour un relevé ni pour une écriture) (C-e341-26).
3. **Lire un relevé de carte en PDF** — hors epic par décision de Guy ; tracé pour qu'il le place (C-e341-26).
4. **Proposer l'appariement des deux côtés d'un virement** (même montant, dates proches) dans les propositions de réconciliation — « à terme » dans
   #972 ; la liste des candidats en tient lieu aujourd'hui (C-e341-24).

*Ouvertes par l'orchestrateur : #977 (1), #978 (2), #979 (3), #980 (4) ; s'y ajoutent #1034 (l'écriture rapprochée de ses deux côtés n'est
plus signalée, C-e34b-2) et #1038 (deux comptes qui partagent un grand livre, indiscernables pour le conseil (b′), revue P1 B1-2).*

## Écarts à la fiche (lire le registre `e34-choix-autonomes-e34b.md`)

*Ajouté à la remédiation P2 (A2-5).* AC-1 (« au plus une transaction par compte du grand livre »), AC-4c et les mutations **M2, M8, M9** dans
leur forme ci-dessus ne décrivent plus la règle livrée : lire **C-e34b-1** (règle de sens contraire de la 34-4 : au plus une transaction par
côté, de comptes et de sens opposés ; jamais une écriture de règlement). **M8 et M9 sont couvertes par M2** : sous cette règle, le compte délié
(cas `NULL`) et le compte relié ailleurs sont refusés par le contrôle du **sens**, que M2 retire ; il n'y a pas de site distinct à muter. AC-4b
(non-régression de la dévalidation fournisseur) est tenue par une **seconde défense** (la garde de suppression de l'écriture d'achat refuse une
écriture rapprochée) : M7c, qui retire le rang 3 sous `None`, ne la fait pas rougir — voir le Dev Agent Record.

## Renvois à l'intégration (la 34-2 et la régénération du PDF sont dans la release)

*Ajouté à la remédiation P2 (A2-6) ; l'intégration de l'Epic 34 les reprend.*

1. **Le pas à pas *Dette*** : `user-manual.tex`, § *Tenir une carte de crédit, pas à pas*, étape 2, renvoie au paragraphe « Solde d'ouverture »
   de la carte (§ *Une carte de crédit*), que la 34-11a2 a laissé en commentaire jusqu'à la 34-2 — à rétablir, et à relire contre l'écran de
   la 34-2 (sens *Dette*).
2. **L'E2E** `credit-card-statement.spec.ts` pose la dette d'ouverture par une écriture (C-e34b-6) : à repasser par la 34-2 si elle le permet.
3. **Le PDF du manuel utilisateur**, à régénérer : y greper (aplati) `rejetez-la` (absent attendu), `Tenir une carte` et `#823` (présent pour le
   lot de plusieurs factures seulement). **Non régénéré par la story**, sur consigne de l'orchestrateur : l'intégration le fait à l'étape
   finale — y greper aussi `celle de son virement` (le pluriel de (b′), ajouté au `.tex` à la clôture, C3-6).
4. **Les LOW de production de la passe P3**, tracés par l'orchestrateur, non traités ici : C3-2 → refs #1041 (une attente déclarée dans un JSON
   illisible est ignorée sans rien dire), C3-3 → refs #1042 (limite de taille du corps lu par `…/cancel`, `413` natif non documenté), C3-5 →
   refs #1043 (test de non-dérive textuel ; bras client non comparé).

## Recoupements

`reconciliation_link.rs`, `reconciliation_cancel.rs`, `settlement_cancellation.rs` : ⚠️ **34-4** (#828, fiche
`34-4-transaction-deja-comptabilisee.md`, AC-2) ajoute à `entry_candidates_sql` un mode `matched_elsewhere` qui compte les écritures
**rapprochées d'un compte au grand livre différent** — exactement celles que la condition 4 neuve rend **candidates**. Après les deux stories,
`entryCount` et `matchedElsewhereCount` compteraient la même écriture : la seconde à intégrer aligne les deux (le signal de la 34-4 devient
« candidate dont l'autre côté est déjà rapproché », sous-ensemble de `entryCount`, et son libellé le dit). Ordre recommandé : 34-4, puis 34-11b,
qui reprend ce mode dans le même constructeur (AC-10) ; **34-3** pour le dialogue « Écriture manuelle » et son AC-8 (texte du second côté, corrigé
ici). `user-manual.tex` avec 34-1, 34-2, 34-11a1, 34-11a2 (sections voisines).

**Inventaire des lecteurs de `matched_entry_id`, élargi** (validation P1, 4.5 ; `git grep -n "matched_entry_id" -- 'crates/*/src/*' 'crates/*/src/**'`,
au-delà de la forme `= ?`) : `journal_entries.rs:2339-2340` (`GROUP BY matched_entry_id`, `MIN(id)` : le motif `MATCHED_BANK_TRANSACTION` d'une
contre-passation ne nomme qu'**une** transaction — juste, une suffit à refuser ; écrit au doc-comment) ; `reconciliation_link.rs:174` (condition 3
du rattachement à un règlement : une écriture de règlement n'a qu'une ligne de banque, inchangé) ; `bank_accounts.rs:1772`, `:1784` (par compte) ;
`exports/csv_tables.rs:750-774` (colonne exportée par transaction, sain) ; `routes/reconciliation.rs:4627` (relu au dev, à classer au Dev Agent Record).

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).
- 2026-11-10 : **validation P1 (Sonnet)** — pour cette fiche : 1 H (4.1), 3 M (4.2 à 4.4), 4 L (4.5 à 4.8). Remédiation (Opus 5.5), tout appliqué.
  **4.1** : définition unique de l'empêchement par l'autre côté (C-e341-35), appliquée à `cancel_blocker`/`get_view`, aux candidates et à la garde
  de règlement ; AC-4b, T4 récrit, M6, M7 (relus `reconciliation_cancel.rs:250-285`, `routes/reconciliation.rs:6462-6478`,
  `LinkSettlementModal.svelte:154-159`, `CancelReconciliationDialog.svelte:73`). **4.2** : `reversalJournalEntryId` nullable, TS, message, doc
  (relu `routes/reconciliation.rs:5766-5775`). **4.3** : #823 lue (`gh api`) — objet triple, la 34-11b résout le virement : `refs #823` et non
  `closes` (C-e341-37) ; correction des textes du manuel et de la 34-4 (b) produits avant elle (C-e341-36, AC-10), sur arbitrage de
  l'orchestrateur, sans toucher les fiches de l'autre groupe (relus `user-manual.tex:2912-2922`, `:2991`). **4.4** : condition 4 indépendante
  d'un lien modifiable (C-e341-22 précisé, AC-4c, M8, M9). **4.5** : inventaire élargi (Recoupements). **4.6** : retrait seul non refusé par
  l'exercice clos (relu `reconciliation_cancel.rs:540-557`), M10. **4.7** : `unlinkOnly` en camelCase (relu `:671-676`). **4.8** : `periodTo` fixé,
  la portée `SingleBankLedger` dite non exercée par cet E2E. Références à « 34-11a » portées à 34-11a1 / 34-11a2.
- 2026-11-10 : **validation P2 (Opus)** — pour cette fiche : 2 M (5.1, 5.2), 2 L (5.3, 5.4), nés de la remédiation P1 ; plus la transmission de
  la 34-4. Remédiation (Opus 5.5), tout appliqué. **5.1** : la décision « autre côté légitime » passe **en tête**, avant les rangs 1 à 5 (relus
  `settlement_cancellation.rs:100-157`) ; tests de la vue et des candidates en exercice clos (AC-4b) ; `user-manual.tex:2913-2914` ajoutée à l'AC-10 ;
  M7b. **5.2** : la transaction concernée passe en argument, `None` garde son sens strict (relus `invoice_settlements_write.rs:793-797`,
  `supplier_invoices.rs:2460-2464`) ; l'alternative « au choix du dev » retirée de la T2 ; non-régression facture client ; M7, M7c. **5.3** : lecture
  ordinaire des autres liens après le verrou de l'écriture (AC-4). **5.4** : champ `unlinkOnly` de la vue (AC-4b). **Transmission de la 34-4**
  (`kesh-gate-logs/e34-transmission-34-11b.md`) : conditions (a), (b′), (c′), clés neuves et retirées, Vitest à quatre cas, M11, textes (AC-10). Les
  fiches 34-3 et 34-4 ne sont pas touchées.
- 2026-11-10 : **validation P3 (Sonnet)** — pour cette fiche : 3 L (5.1 à 5.3). Remédiation (Opus 5.5), tout appliqué. **5.1** : M11 ajoutée à la
  liste. **5.2** : préfixe `reconciliation-labels-already-booked-…`, tests de la 34-4 à adapter, `m` défini en une phrase (relu
  `34-4-transaction-deja-comptabilisee.md:97-113`). **5.3** : `unlinking: Option<i64>` remplacé par `concerned: Option<&BankTransaction>`, un seul
  paramètre (relu `settlement_cancellation.rs:80-90`).

- 2026-11-12 : **développement (Opus 5.5)**, worktree `kesh-e34b`, base `3c1510d8` + fusion de la 34-4 close (`a8dce4a9`). Étape 0 : LOW F6 de la
  validation P4 (les deux `…_unlinking`) traité par C-e34b-3 — relu `reconciliation_cancel.rs:287-318` (avant) et
  `invoice_settlements_write.rs:806` (`settlement_cancel_blocker_unlinking(conn, company_id, settlement_id, None)`) ; inventaire des lecteurs :
  `journal_entries.rs:2339-2340` (doc-comment ajouté, un motif suffit), `routes/reconciliation.rs:4688` (`matched_entry_id IS NULL` du rejet,
  garde d'une transaction en attente : sain). Noms de la fiche alignés sur ceux de la 34-4 (`entry_sql(EntrySqlMode)`, `MATCHED_ELSEWHERE` →
  `HAS_LINK`, `NOT_MATCHED` → `linkable_side()`, `count_already_booked_for_page`). Écarts consignés au registre `e34-choix-autonomes-e34b.md`
  (C-e34b-1 à 8).
- 2026-11-12 : **revue de code P1** (trois lentilles, rapports `kesh-gate-logs/e34b-review-p1-{B,E,A}.md`) : 0 CRITICAL, 0 HIGH, 2 MEDIUM distincts
  (B1-1 = A1-1, E1-1 + E1-2), une vingtaine de LOW. **Remédiation (Opus 5.5)**, commits `1b83c157` (code, tests, docs) et `25a7d4dd`
  (`sitesTotal`, prompt P2) :
  - **B1-1** : (c′) compte à part les `k − m` non rapprochées et les `m` écritures de virement, singulier/pluriel (clés `-transfer-among-one`,
    `-other` ; `-transfer-among` retirée), quatre locales, replis, manuel ; Vitest récrit (C-e34b-11).
  - **E1-1, E1-2** : `expectUnlinkOnly` dans le corps de `POST …/cancel`, `409 RECONCILIATION_CANCEL_CHANGED`, rien d'écrit ; l'écran l'envoie
    toujours et relit la vue ; tests HTTP des deux sens, Vitest, mutations M12 et F4 (C-e34b-9). `api-external.md` suit.
  - **E1-3** : `other_side_predicate` exclut les écritures de règlement (client ; fournisseur payée) ; test `s34_11b_e1_3_…`, M13 (C-e34b-8 récrit).
  - **B1-2** : la limite des comptes qui partagent un grand livre écrite au manuel ; **issue demandée**.
  - **B1-3, E1-4, A1-11** : M2b déclarée équivalente (doc-comment de `linkable_side`) ; le cas encore bloqué testé
    (`s34_11b_a1_11_…`, état forgé).
  - **B1-4** : `CancelOutlook` — une seule lecture de l'autre côté par candidate, par vue et par geste (C-e34b-10).
  - **B1-5, A1-10** : « Aujourd'hui… ; si l'autre côté est annulé d'abord… » à la fenêtre de rattachement (quatre locales) et au manuel.
  - **B1-6, A1-5** : `kind: entry` (`api-external.md`) et la « mention » d'audit du manuel récrits au texte exact.
  - **A1-2** : C-e34b-1 corrigé ; tests `s34_11b_a1_2_relie_ailleurs_l_autre_mouvement_admis` (admis) et
    `s34_11b_a1_2_virement_eclate_sur_trois_comptes_refuse` (refusé) ; manuel (« Ce qui ne se rattache pas »).
  - **A1-3, E1-5** : doc-comments d'`entryCount` / `matchedElsewhereCount` (TS et Rust).
  - **A1-6, B1-8** : l'E2E lit les deux soldes comparés et exige `statementGap: null` ; il tient son prélèvement sur un compte bancaire **propre**
    (IBAN synthétique réservé, actif propre ; C-e34b-12) ; `s34_11b_ac4c` asserte aussi avant tout relien.
  - **A1-7** : assertion HTTP de `unlinkOnly` aux candidates ; mutations front F1 à F3 jouées.
  - **A1-8** : M7c décrite telle qu'elle est jouée, et rejouée avec `supplier_invoice_draft_e2e`.
  - **A1-4, A1-9, B1-7 — renvoyés à l'intégration** : le pas à pas *Dette* (la 34-2 est dans la release : à vérifier contre elle) et le PDF du
    manuel (régénéré à la release ; y greper `rejetez-la`, `Tenir une carte`, `#823`). Rien d'autre n'est changé pour eux.
- 2026-11-12 : **fusion de la 34-11a2** (`2dbcc102`), gates rejoués au commit de fusion (§ *Gates*).
- 2026-11-12 : **revue de code P2** (Sonnet, trois lentilles, rapports `kesh-gate-logs/e34b-review-p2-{B,E,A}.md`) : 0 CRITICAL, 0 HIGH,
  2 MEDIUM (B2-1, B2-2), LOW B2-3/E2-4, B2-4, E2-2, E2-3, A2-2 à A2-7. **Remédiation (Opus 5.5)**, commit de code `a9c7b967`, **rétabli à
  l'identique par `225a90de`** (incident ci-dessous) ; dernier commit de code : `225a90de` :
  - **B2-1** : le corps de `POST …/cancel` reste facultatif pour tout client — lu brut (`axum::body::Bytes`), `parse_cancel_expectation` : vide,
    `null`, `{}`, champ absent ou `null`, corps non JSON → aucun contrôle ; booléen → contrôle ; autre type → `400 VALIDATION_ERROR` ; test
    `s34_11b_b2_1_le_corps_de_l_annulation_reste_facultatif` (six formes acceptées, mauvais type refusé), mutation M14 (C-e34b-13).
  - **B2-2** : bras **règlement client** de l'exclusion testé (`s34_11b_b2_2_un_reglement_client_n_a_pas_d_autre_cote`) ; M13 scindée en M13a
    (fournisseur payée) et M13b (règlement client).
  - **B2-3, E2-4** : les deux sous-requêtes d'`other_side_predicate` bornées par `company_id = bt.company_id` (aucun paramètre lié ajouté) ; test
    unitaire `the_settlement_exclusion_matches_the_classification` (C-e34b-15).
  - **E2-3** : (b′) au pluriel (`-transfer-other`, quatre locales, repli fr-CH, Vitest, `sitesTotal` 2958), mutation F5 (C-e34b-14).
  - **B2-4** : `docs/api-external.md` — tableau des refus de `…/cancel`, corps facultatif. **E2-2** : liens rustdoc.
  - **A2-2 à A2-7** : fiche (§ *Écarts à la fiche*, § *Renvois à l'intégration*, *Annotations de la remédiation P2*).
  - **B1-2** de la P1 : issue **#1038** (comptes qui partagent un grand livre). Hors périmètre, déjà ouvertes : #977 (signe du CSV), #978 (PDF
    joint), #979 (lecture du PDF), #980 (appariement), #1034.
  - ⚠️ **Incident** : le script de rouges (`rouges-rem2.sh`) avait remis à l'index les fichiers de `8781e150` ; le commit suivant, `8abf9b50`
    (prompt P3), les a emportés et a **retiré le code de `a9c7b967`**. Rétabli à l'identique par `225a90de` (`git diff a9c7b967 225a90de` ne rend
    que le prompt) ; les mutations du jeu 4 jouées avant le rétablissement, sur l'ancien code, ont été **invalidées et rejouées** sur
    `225a90de`. Le prompt P3 le dit.
  - Correction d'un compte rendu antérieur : le commit du Change Log de la remédiation P1 est **`418efd46`** (et non « ff0b2a0b », écrit par
    erreur dans un rapport à l'orchestrateur, hors dépôt).
- 2026-11-12 : **passe ciblée P3** (Opus, une lentille, sur la seule remédiation P2 `a9c7b967`/`225a90de` ; prompt
  `34-11b-carte-credit-releve-review-prompt-p3-ciblee.md`, rapport `kesh-gate-logs/e34b-review-p3-ciblee.md`, six axes déclarés exercés) :
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW**. **Clôture (Opus 5.5)**, sans une ligne de code de production :
  - **C3-1** : renvois C-e34b-14 et C-e34b-15 permutés dans l'entrée P2 ci-dessus (corrigés : la borne par société est C-e34b-15, (b′) au
    pluriel C-e34b-14 ; relu `e34-choix-autonomes-e34b.md:154`, `:159`).
  - **C3-4** : `docs/api-external.md`, la ligne `400 VALIDATION_ERROR` remontée en tête du tableau des refus de `…/cancel`, « contrôlé avant tout
    autre refus, `404` compris » (`parse_cancel_expectation(&body)?` parle en premier dans le handler).
  - **C3-6** : `user-manual.tex`, (b′) au pluriel décrit (« N écritures de virement sont déjà rapprochées… », rattacher à celle de son
    virement). **PDF non régénéré** : l'intégration le fait à l'étape finale (§ *Renvois à l'intégration*, 3).
  - **C3-2, C3-3, C3-5** (production) : tracés par l'orchestrateur — refs #1041, refs #1042, refs #1043 (§ *Renvois à l'intégration*, 4).
  - Pas de gate relancé : commit sans code (documentation, fiche, registre de sprint).
- **Synthèse de la boucle de revue de code** (trend au-dessus de LOW) : **P1** (Opus, trois lentilles) 0 C / 0 H / **2 M** (B1-1, E1-1+E1-2) et
  une vingtaine de LOW → **P2** (Sonnet, trois lentilles) 0 C / 0 H / **2 M** (B2-1, né de la remédiation E1-1 ; B2-2, couverture du bras client
  de la remédiation E1-3) → **P3** (Opus, passe ciblée, une lentille) **0 au-dessus de LOW**, 6 LOW. Boucle close sur une passe sans MEDIUM.
  Décisions : M2b reclassée **équivalente** (le plafond ≤ 1 est une défense inerte, P1) ; le corps de `…/cancel` reste facultatif pour tout client
  plutôt qu'un contrat strict (C-e34b-13) ; les LOW de production de P3 tracés et non traités (consigne de l'orchestrateur : boucle close) ;
  B1-2 tracé en #1038 ; le pas à pas *Dette*, l'E2E de la dette d'ouverture et le PDF renvoyés à l'intégration. Signal de découpage : non levé
  (aucune passe n'a recyclé un défaut d'une passe antérieure sous une autre forme au-delà de B2-1, né de la remédiation et corrigé en P2).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement en autonomie, clé e34b).

### Completion Notes List

- **Fusion** de `story/34-4-transaction-deja-comptabilisee` (`0276a3ed`) en `a8dce4a9` : conflits `CHANGELOG.md` (sections unies) et
  `i18n-keys.test.ts` (`sitesTotal` 2942 + 2 + 8 = **2952**, recompté par la garde) ; gate ciblé après fusion :
  `reconciliation_link_entry_e2e` 40/40 (2 ignorés), Vitest `i18n-keys` 11/11.
- **La règle** (AC-1, C-e34b-1) : condition 4 = `linkable_side()` — aucune transaction liée autre que l'autre côté (autre compte bancaire,
  montant opposé à `M`, `other_side_predicate`), et une au plus ; `transfer_other_side` en est la lecture pour une transaction donnée.
  `MatchedElsewhere` = liste + `HAS_LINK` (C-e34b-2) ; `count_already_booked_for_page` lit les mêmes textes.
- **L'annulation** (AC-4, AC-4b) : `entry_cancel_blocker(…, &BankTransaction)` décide l'autre côté en tête (C-e34b-3) ; `get_view` porte
  `unlink_only` ; `cancel_in_tx` étape 3 bis : lecture **ordinaire** après le verrou de l'écriture, retrait du seul lien, `recheck_open_locked_in_tx`,
  trace `unlinkOnly: true`, `reversalJournalEntryId: null` ; le chemin ordinaire écrit `unlinkOnly: false`.
  `ReconciliationCancellation.reversal_journal_entry_id: Option<i64>`, réponse HTTP nullable.
- **Écrans** : `already-booked.ts` (b′)/(c′), clés `-transfer`, `-transfer-among` (quatre locales ; `-elsewhere`, `-combined` retirées) ;
  `CancelReconciliationDialog` (confirmation du retrait seul) ; `reconciliationCancelDoneMessage` + toast du détail d'import (C-e34b-5) ;
  `LinkSettlementModal` (texte du retrait seul, sans case, C-e34b-4). `sitesTotal` 2952 → **2956** (+4, recompté par la garde).
- **Tests ajoutés** (périmètre `a8dce4a9` → `HEAD`, recomptés par `grep -cE '#\[(sqlx::)?test'` / `^\s*it(\.each)?\(` aux deux bornes) :
  `reconciliation_link_entry_e2e.rs` 42 → 50 (+8 `s34_11b_*`), `bank_imports_e2e.rs` 89 → 90 (+1, AC-6), `reconciliation_link.rs` (unitaires)
  5 → 6 (+1 ; un récrit) ; Vitest `ReconciliationProposals.test.ts` 67 → 68 (deux cas (b), (c) remplacés par trois), `CancelReconciliationDialog.test.ts`
  9 → 10, `reconciliation-cancel.test.ts` 6 → 7, `LinkSettlementModal.test.ts` 28 → 29 ; E2E neuf `credit-card-statement.spec.ts`.
- **Rouges** (`kesh-gate-logs/e34b/rouges.log`, tests neufs contre le code `2fafcb49`, sources de la story retirées puis rendues) : **7 rouges**
  (AC-2 ×2, AC-3, AC-4 ×3 dont la course, AC-4 exercice clos) ; **2 verts dès le départ, attendus** : `s34_11b_ac4c_comptes_relies` (l'ancienne
  condition 4 refusait déjà tout lien : il garde la règle neuve, tué par M2) et `s34_11b_ac6_releve_csv_d_une_carte` (le CSV d'une carte est
  ouvert depuis la 34-11a1 : la fiche le croyait rouge). Vitest : 8 rouges contre l'`already-booked.ts` et les catalogues d'avant
  (`rouges-front.log`).
- **Mutations** (`kesh-gate-logs/e34b/mut/`, script `scripts/mut-e34b.py`, une par prise de verrou, restaurées par `git checkout` + `touch`), toutes
  **tuées** : M1 (11 rouges), M2 = M8 = M9 (sens non contrôlé : 5, dont `s34_11b_ac4c_comptes_relies`, `s34_11b_ac3`, deux tests de la 34-4),
  Macc (compte non contrôlé : `s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas` et le test du texte), M3 (4), M4 (4), **M5** (la course
  `s34_11b_ac4_deux_annulations_simultanees`, et elle seule), M6 = M7 (3), M7b (`s34_11b_ac4_exercice_clos`), M7c (rejouée comme « rang 3 sauté
  sous `None` », C-e34b-3 : `reconciliation_e2e::cancelling_a_reconciled_settlement_is_refused`), M10 (`s34_11b_ac4_exercice_clos`), M11 (Vitest
  (c′) seul). **M2b** (plafond « un seul lien » levé) n'est tué que par le test du texte : **équivalente** dans les états atteignables — deux liens
  d'autres comptes et de sens opposé exigeraient un second rattachement que la condition refuse déjà ; le plafond est une défense.
- **Gates au dernier commit de code** : voir *Gates* ci-dessous.
- **Captures** (thème clair, E2E) : `kesh-gate-logs/e34b/captures/34-11b-*.png`.
- **Manuel** : `user-manual.tex` § *Tenir une carte de crédit, pas à pas* (neuf, `sec:tenir-carte`), textes du second côté récrits (encadré « Dépense
  déjà réglée », *Une transaction qui semble déjà comptabilisée*, *Écriture manuelle*, *Rattacher …*, *Annuler un rapprochement*). **PDF non
  régénéré** (consigne de l'epic : à l'intégration). `api-external.md`, `CHANGELOG.md` (*Ajouté*, *Corrigé* ; l'entrée de la 34-4 alignée).
- **Écarts à la consigne Edit** : `python3 -I` sur la fiche (cocher sept tâches, artefact `_bmad-output`) ; `prettier --write` sur la spec E2E
  neuve (`daf8459c`), puis la mise en forme rendue au style du dépôt par une réécriture complète (Write) de ce fichier neuf (`d7bc0fd6`) ; un
  `sed -i` sur un script de `kesh-gate-logs` (pas du code du dépôt), qui a laissé `gates-e34b.sh.bak` (non supprimé : `rm` interdit).
- **Issues à demander** (fiche, § *Ce qui ne se traite pas*, une par problème) : inversion d'une colonne de montant unique d'un profil CSV ;
  joindre un PDF à un relevé saisi ; lire un relevé de carte en PDF ; proposer l'appariement des deux côtés d'un virement. Plus : signaler
  l'écriture rapprochée de ses deux côtés vue d'une troisième transaction (C-e34b-2, non signalée).

### File List

- `crates/kesh-db/src/repositories/reconciliation_link.rs`, `reconciliation_cancel.rs`, `journal_entries.rs` (doc-comment) ;
  `crates/kesh-db/tests/reconciliation_cancel.rs`
- `crates/kesh-api/src/routes/reconciliation.rs` ; `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs`, `bank_imports_e2e.rs`,
  `fixtures/releve-carte-synthetique.csv`
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`
- `frontend/src/lib/features/reconciliation/already-booked.ts`, `already-booked-i18n.test.ts`, `CancelReconciliationDialog.svelte` (+ test),
  `LinkSettlementModal.svelte` (+ test), `reconciliation-cancel.ts` (+ test), `ReconciliationProposals.test.ts`, `reconciliation.types.ts` ;
  `frontend/src/lib/shared/i18n-keys.test.ts` ; `frontend/src/routes/(app)/bank-import/[id]/+page.svelte` ;
  `frontend/tests/e2e/credit-card-statement.spec.ts`
- `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e34b.md`, `34-11b-carte-credit-releve-review-prompt-p1.md`, cette fiche

### Gates

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e34b/`, tels qu'ils ont tourné.

- **Premier passage** (`gates-1.txt`, commit `7ab06dce`) : `test-fast.sh --ci` **rouge** — 5207 exécutés, 5204 passés, **3 échecs réels** :
  `bank_account_closed_inventory` ×2 (un second `UPDATE bank_transactions` dans `cancel_in_tx` ; revérification plus la dernière acquisition)
  et `textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres` (compte 2101 du manuel). Corrigés par `52dc7ba1` : un
  seul chemin d'annulation (étapes 4 et 6 sautées pour le retrait seul ; `UPDATE`, revérification et trace communs) ; exemple sans numéro.
  Les mutations M3, M4, M5, M10 ont été **rejouées** sur `52dc7ba1` (2e jeu, `mut/resume.log`, premiers journaux gardés en `M*-jeu1.log`) :
  toutes tuées, par les mêmes tests.
- **Au dernier commit de code `52dc7ba1`** (`gates-2.txt`) :
  - `scripts/test-fast.sh --ci` : premier lancement **tué au plafond mem-guard** (8 Go, code 137, à la compilation de `kesh-api`) ; relancé
    (`gate-test-fast-2b.log`, base remise à neuf) : **5207 exécutés, 5207 passés, 13 ignorés** ;
  - `cargo build --workspace --all-targets` : vert ;
  - `scripts/gate-ci.sh kesh_ci_e34b` : **5214 passés, 0 échec, 17 ignorés** ;
  - E2E complet (port 3382, `kesh_e2e_e34b` recréée vide, `e2e-r2/`) : **390 passés, 7 échecs, 17 ignorés** — les sept de la KF-029 (#97),
    nommés un par un contre `docs/testing.md:394-400` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150`) ; `credit-card-statement.spec.ts` vert ; aucun port tenu après (`ss-apres.txt` vide).
- **Frontend** (sources inchangées depuis `63b540b6`, hors la spec E2E) : `npm run check` 0 erreur (29 avertissements préexistants),
  `lint-i18n-ownership` vert, `test:unit` **227 fichiers, 2968 tests**, `build` vert (`front-*-2.log`).
- ⚠️ `KESH_E2E_CAPTURES_DIR` pointait sur `e34b/captures/` pour toute la suite : les captures des autres specs s'y sont ajoutées ; celles de la
  story sont les cinq `34-11b-*.png`.

#### Remédiation P1 et fusion de la 34-11a2

- **Rouges** (`rouges-rem1.log`, tests neufs contre le code de `96f8ee70`) : backend 3 rouges (`s34_11b_e1_1_…`, `e1_2_…`, `e1_3_…`) ;
  3 verts dès le départ, attendus — `a1_2` ×2 et `a1_11` documentent la règle (cas admis, cas refusé, cas bloqué), ils ne corrigent rien ;
  Vitest 9 rouges (catalogues des clés `-transfer-among-*`, (c′), confirmation et relecture du dialogue, `expectUnlinkOnly`).
- **Mutations, 3e jeu** au code de la remédiation (`mut/resume-j3.log`, journaux `mut/*-j3.log`) : **toutes tuées** — M1 (14), M2 (7), Macc (2),
  M3 (6), M4 (5), M5 (1, la course), M6 (7), M7b (1, `s34_11b_ac4_exercice_clos` ; un premier jet ne compilait pas, rejoué), M7c (1,
  `reconciliation_e2e::cancelling_a_reconciled_settlement_is_refused` ; avec `supplier_invoice_draft_e2e` dans le lot, **la dévalidation
  fournisseur ne rougit pas** : son refus `MATCHED_BANK_TRANSACTION` vient de la garde de suppression de l'écriture d'achat, pas de la queue
  commune — seconde défense, pas un trou), M10 (1), M12 (2, les deux sens), M13 (1), F1 (2), F2 (1), F3 (1), F4 (1), M11 (1).
  **M2b** non rejouée : déclarée **équivalente** (doc-comment de `linkable_side`).
- **Au commit de fusion `2dbcc102`** (`gates-4.txt`, `front-4.txt`) : `test-fast --ci` **5225/5225**, 13 ignorés ; `cargo build` vert ;
  `gate-ci.sh kesh_ci_e34b` **5232 passés, 0 échec, 17 ignorés** ; frontend : `check` 0 erreur, `lint-i18n-ownership` vert, `test:unit`
  **228 fichiers, 2974 tests**, `build` vert ; **E2E** (`e2e-r4/`) **389 passés, 8 échecs, 17 ignorés** — les sept de la KF-029
  (`docs/testing.md:394-400`) et **un de pollution** : `backup-restore-new-install.spec.ts:140` (élément `admin-restore-files-restored`
  absent au bout de 60 s), **vert rejoué seul** (`e2e-r5/` : 3/3), dans la plage « 7 à 9 » de `docs/testing.md` ; `credit-card-statement.spec.ts`
  et `bank-accounts-credit-card.spec.ts` verts. Aucun port tenu après.
- Un passage intermédiaire (`gates-3.txt`, `25a7d4dd`) avait rendu `test-fast` 5213/5213, `build`, `check`, `lint` verts ; interrompu pour la
  fusion.
- **Écarts à la consigne Edit** (remédiation) : un `sed -i` lancé par erreur sur `crates/kesh-api/src/routes/reconciliation.rs` (une ligne de
  doc-comment, juste) — sa sauvegarde `.unused`, sortie du dépôt, est sous `kesh-gate-logs/e34b/reconciliation.rs.sed-backup-a-ignorer` ;
  `python3 -I` pour dériver un script de gates dans `kesh-gate-logs`.

#### Annotations de la remédiation P2 (2026-11-12)

- **A2-2 — décomptes recomptés au HEAD `8781e150`** (périmètre `a8dce4a9` → `8781e150`, les entrées ci-dessus restent datées) :
  `reconciliation_link_entry_e2e.rs` 42 → **56** (+14 : 8 du développement, 6 de la remédiation P1), `CancelReconciliationDialog.test.ts` 9 → **11**,
  `sitesTotal` **2957**. Les autres lignes tiennent. Les chiffres de la remédiation P2 sont plus bas.
- **A2-3** : le huitième échec E2E au commit de fusion, `backup-restore-new-install.spec.ts:140`, est la pollution tracée en **#931** ; vert
  rejoué seul. `e2e rc=0` dans `gates-4.txt` est le code du **script enveloppe**, non de Playwright (`playwright rc=1` à la fin de
  `e2e-r4/playwright.log`, ses 8 échecs étant nommés ci-dessus).
- **A2-4** : voir § *Écarts à la fiche* — la dévalidation fournisseur est gardée par une seconde défense ; M7c ne la vise pas.
- **A2-7 — T3, vérification consignée** : le vocabulaire d'audit (`crates/kesh-api/src/audit_labels.rs:184`, `reconciliation.cancelled`) et
  l'écran du journal d'audit ne lisent pas les détails : le libellé reste « Rapprochement annulé » pour les deux chemins, `unlinkOnly` et
  `reversalJournalEntryId` ne se lisent que dans le détail JSON (le manuel le dit). Rien à aligner ; un libellé distinct serait une demande à
  Guy.

#### Remédiation P2 — rouges, mutations, gates (2026-11-12)

- **Rouges** (`rouges-rem2.log`, tests neufs contre le code de `8781e150`) : backend **1 rouge**, `s34_11b_b2_1_…` (corps facultatif) ;
  `s34_11b_b2_2_…` et `s34_11b_e1_3_…` **verts dès le départ, attendus** — l'exclusion des deux bras existait depuis la P1 ; c'est la
  mutation qui prouve qu'ils la tiennent (M13a, M13b ci-dessous). Vitest **6 rouges** (catalogues des quatre locales et replis : clé
  `-transfer-other` ; (b′) au pluriel).
- **Mutations, jeu 4**, au code rétabli `225a90de` (`mut/resume-j4.log`, journaux `mut/*-j3.log` — le lanceur suffixe `-j3`) : **toutes
  tuées** — M1 (14), M2 (7), Macc (2), **M13a** (1, `s34_11b_e1_3_…`), **M13b** (1, `s34_11b_b2_2_…`), **M14** (1, `s34_11b_b2_1_…`), **F5**
  (1, Vitest (b′)). Les mutations du jeu 4 jouées avant le rétablissement (sur le code de `8abf9b50`) sont invalides et ne sont pas comptées.
- **Gates** au commit `aa8d7fc9` (code identique à `225a90de` : `git diff --stat 225a90de aa8d7fc9` ne nomme que le prompt P3) — `gates-6.txt`,
  `front-6.txt` :
  - `scripts/test-fast.sh --ci` : **5228 exécutés, 5228 passés, 13 ignorés** (`gate-test-fast-6.log`) ;
  - `cargo build --workspace --all-targets` : vert ;
  - `scripts/gate-ci.sh kesh_ci_e34b` : **5235 passés, 0 échec, 17 ignorés** (somme des `test result` de `gate-ci-6.log`) ;
  - frontend : `check` 0 erreur, `lint-i18n-ownership` vert, `test:unit` **228 fichiers, 2974 tests**, `build` vert ;
  - **E2E** (`e2e-r6/`) : **390 passés, 7 échecs, 17 ignorés** (`playwright rc=1`) — les sept de la KF-029 (#97), nommés contre
    `docs/testing.md:394-400` : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ;
    `credit-card-statement.spec.ts:128` et `bank-accounts-credit-card.spec.ts:33` verts ; aucun port tenu après (3382 libre).
- **Écarts à la consigne Edit** (remédiation P2) : aucun fichier de code écrit autrement que par Edit ; le script de chaîne `chaine-rem2b.sh`
  a été dérivé par `sed` dans `kesh-gate-logs` (hors dépôt). L'incident `8abf9b50` est au Change Log.
