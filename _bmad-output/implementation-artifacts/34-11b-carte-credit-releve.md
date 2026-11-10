# Story 34-11b : le relevé d'une carte de crédit, sa réconciliation et son prélèvement mensuel (#972)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration.** Registre : `e34-choix-autonomes-e341.md`,
C-e341-22 à C-e341-28. Suit **34-11a** (le compte carte) et **34-3** (« Écriture manuelle » sur tout compte imputable sauf celui de la
banque). Commits en `refs #972` ; à la PR : `closes #972`.*

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
  (inventaire de la 34-11a, C4). **Soldes signés librement** (`kesh-core/src/bank_imports/manual.rs:480-485`, « Règle 5 (soldes : signe libre) ») ;
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
- **C-e341-23 — l'annulation d'un côté ne défait que ce côté** : quand une **autre** transaction, d'un **autre** compte du grand livre, pointe la
  même écriture, l'annulation remet `matched_entry_id` à `NULL` pour la seule transaction annulée (statut `pending`, trace
  `reconciliation.cancelled` avec `unlink_only: true`) et **ne contre-passe rien** ; l'écriture reste celle de l'autre côté. L'annulation du dernier
  côté suit le chemin d'aujourd'hui. Écarté : refuser (impasse décrite ci-dessus).
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
   l'ordre inverse. La trace dit `unlink_only`. Le verrou : la transaction annulée `FOR UPDATE`, puis la ligne de l'écriture `FOR UPDATE` avant la
   lecture des autres liens (même discipline que `link_entry_in_tx`, `:303-330`) ; deux annulations simultanées des deux côtés ne laissent pas
   l'écriture contre-passée avec un lien vivant (test à deux connexions, ou raisonnement écrit au doc-comment s'il n'est pas déterministe — dit tel quel).
5. **AC-5 — le relevé saisi d'une carte** (E2E, chemin principal) : sur une carte (34-11a) avec solde d'ouverture *Dette* 1'000 (34-2), un relevé
   saisi à la main : ouverture `-1000.00`, deux achats en débit (120.00, 80.00), le paiement reçu en crédit (1000.00), clôture `-200.00` → accepté ;
   chaque achat passé par « Écriture manuelle » sur une charge ; le paiement rattaché à l'écriture de virement de la banque ; le solde de la carte
   dans la liste vaut `-200.00` et égale le solde du relevé (pas d'écart affiché).
6. **AC-6 — le relevé CSV d'une carte** : un fichier CSV synthétique à colonnes débit/crédit séparées s'importe sur la carte par un profil, et ses
   achats arrivent en débit (test d'intégration de la route d'import, fixture synthétique sous `crates/kesh-api/tests/fixtures/`).
7. **AC-7 — l'écran** : la boîte de rattachement (v017-3a) liste l'écriture de virement pour la transaction de la carte ; rien n'est rattaché sans
   clic (Vitest + E2E de l'AC-5).
8. **AC-8 — manuel pas à pas** : section neuve *Tenir une carte de crédit* (`user-manual.tex`, près de *Comptes bancaires*) : créer le compte de
   dette (par ex. un 2030 *Carte de crédit* sous le 20 — le numéro est un exemple, Kesh n'en déduit rien) ; déclarer la carte (34-11a) avec son
   solde d'ouverture *Dette* (34-2) ; chaque mois, saisir le relevé depuis le PDF (signes : la dette en négatif, les achats au débit, le paiement au
   crédit) ou l'importer en CSV (deux conventions) ; réconcilier les achats (« Écriture manuelle », règles) ; le prélèvement : côté banque
   « Écriture manuelle » sur le compte de la carte, côté carte *Rattacher à une écriture* ; annuler un côté ; ce que la carte ne fait pas. Un exemple
   chiffré **synthétique**. PDF régénéré, aplati.
9. **AC-9 — `api-external.md`** : la condition du rattachement et l'annulation d'un seul côté.

## Tâches

- [ ] **T1 — rouges d'abord** (`kesh-gate-logs/e341/rouges.log`) : AC-2 (rouge aujourd'hui : l'écriture n'est pas candidate), AC-3, AC-4, AC-6.
- [ ] **T2 — dépôt** : condition 4 (AC-1) ; `reconciliation_cancel::cancel_in_tx` : branche « autre côté » avant le classement, et la garde
  `settlement_cancellation.rs:135-157` qui ne compte plus comme bloquant un lien d'un **autre** compte du grand livre dans ce seul cas (ou appelée
  après le retrait du lien, au choix du dev, motivé au Change Log). ⚠️ `kesh-db` touché : **gate complet même en cours de boucle**.
- [ ] **T3 — trace** : `unlink_only` à `reconciliation.cancelled` ; l'écran d'audit l'affiche s'il rend les détails (sinon rien).
- [ ] **T4 — écran** : aucun changement attendu hors libellés ; si la boîte de rattachement filtre côté client un candidat déjà rapproché, l'aligner.
- [ ] **T5 — E2E** `credit-card-statement.spec.ts` (AC-5, AC-7), données créées par l'API sur la base E2E, montants synthétiques.
- [ ] **T6 — manuel** (AC-8), `api-external.md` (AC-9), captures `kesh-gate-logs/e341/captures/34-11b-*.png` (relevé saisi, rattachement du
  prélèvement, liste des comptes).
- [ ] **T7 — CHANGELOG** : *Ajouté* (tenir une carte), *Corrigé* (un virement entre deux comptes de Kesh se réconcilie des deux côtés), `refs #972`.

### Mutations nommées

- **M1** : condition 4 rendue à l'ancienne forme → rouge AC-2.
- **M2** : condition 4 sans le filtre du compte du grand livre (toute transaction d'un autre **compte bancaire** admise) → rouge AC-3 (comptes qui
  partagent le grand livre).
- **M3** : l'annulation contre-passe même quand l'autre côté reste lié → rouge AC-4.
- **M4** : l'annulation « un côté » retire aussi le lien de l'autre → rouge AC-4.
- **M5** : la lecture des autres liens déplacée avant le verrou de l'écriture → rouge du test à deux connexions s'il existe, sinon écrit non tuée.

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

## Recoupements

`reconciliation_link.rs`, `reconciliation_cancel.rs`, `settlement_cancellation.rs` : ⚠️ **34-4** (#828, fiche
`34-4-transaction-deja-comptabilisee.md`, AC-2) ajoute à `entry_candidates_sql` un mode `matched_elsewhere` qui compte les écritures
**rapprochées d'un compte au grand livre différent** — exactement celles que la condition 4 neuve rend **candidates**. Après les deux stories,
`entryCount` et `matchedElsewhereCount` compteraient la même écriture : la seconde à intégrer aligne les deux (le signal de la 34-4 devient
« candidate dont l'autre côté est déjà rapproché », sous-ensemble de `entryCount`, et son libellé le dit). Ordre recommandé : 34-4, puis 34-11b,
qui reprend ce mode dans le même constructeur ; **34-3** pour le dialogue « Écriture manuelle ». `user-manual.tex` avec
34-1, 34-2, 34-11a (sections voisines).

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
