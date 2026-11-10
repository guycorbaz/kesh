# Story 34-4 : la réconciliation signale une transaction qui semble déjà comptabilisée, et demande confirmation (#828)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (lecture seule, champ additif). Registre :
`e34-choix-autonomes-e342.md`, C-e342-5 à C-e342-9. Conception reprise de la fiche courte
`v017-3c-avertissements-propositions.md` (présente sur `main` depuis la release 0.17.0, `910e5d5a` ; statut `backlog`, jamais
développée) et de ses choix C-v017-3-7, -22, -23, -25 (`v017-3-choix-autonomes.md`). Commits en `refs #828` ; à la PR :
`closes #828`, `refs #823`, `refs #671`. **Après 34-3** (§ *Recoupements*).*

## Story

En tant que Comptable qui rapproche ses relevés, je veux que la page de réconciliation me **signale** une transaction en
attente qui ressemble à un mouvement déjà comptabilisé, et me **demande confirmation** avant que je l'accepte, que j'en passe
une écriture manuelle ou que je l'éclate, afin de ne pas compter deux fois un mouvement déjà aux livres.

## Origine et ce qui est déjà tranché

- **Précision de Guy sur #671 (2026-10-23)**, citée par la v017-3c : « Kesh doit afficher un avertissement et demander à
  l'utilisateur quoi faire : la garder en attente, la rejeter, la rattacher à l'écriture existante, etc. — jamais l'affecter
  d'office. »
- **#828, « Attendu »** : un avertissement par ligne ; « une confirmation avant *Accepter*, *Affecter manuellement* et
  *Éclater* pour une ligne signalée ; **aucun refus serveur** » ; une mesure du coût. Cela répond à la question ex-Q2 de la
  v017-3c (« une ligne signalée ne propose-t-elle plus rien ? ») : non, elle demande confirmation (C-e342-6).
- La règle du `CLAUDE.md` « un appariement automatique propose, il ne crée jamais » est tenue : rien n'est rattaché, rejeté ni
  refusé d'office ; l'avertissement renvoie au bouton *Rattacher à une écriture existante* de la ligne.

## Ce que le code dit (prémisses établies)

- **Les deux listes du dialogue existent, et seront comptées par elles-mêmes** (C-v017-3-7 : aucune seconde règle) :
  - `find_link_candidates` (`crates/kesh-db/src/repositories/reconciliation_link.rs:220-240`) : règlements de factures client
    par virement sur **ce** compte bancaire, non rapprochés, au montant de la transaction ou d'au plus 5 centimes de moins
    (`candidates_sql`, `:156-180`) ; **crédits seulement** (`transaction_can_be_linked`, `:83-85`) ; 50 au plus (`:76`) ;
  - `find_entry_candidates` (`:640-654`) : écritures dont le mouvement sur le compte du grand livre égale `t.amount`, non
    rapprochées, ni contre-passées ni contre-passations, hors règlement par virement de ce compte (`entry_candidates_sql`,
    `:583-606`) ; crédit ou débit non nul en CHF (`:513-515`) ; 50 au plus (`:504`).
  Les deux sont **disjointes** par construction (le `NOT EXISTS` de `:601-604` écarte de la seconde ce que compte la première).
- **Les deux routes du dialogue** les appellent après un pré-vol (`get_link_candidates`, `routes/reconciliation.rs:6171` ;
  `get_entry_candidates`, `:6433-6475`, qui résout le compte du grand livre par `link_preflight`).
- **`GET /reconciliation/proposals`** (`get_proposals`, `routes/reconciliation.rs:974`) fait déjà **N requêtes par
  transaction** (« Architecture 4-pass », `:966-972` ; boucle `:1034-1146`) ; il lit le compte du grand livre
  (`ba_check.journal_account_id`, `:1193`) ; limite 100 par défaut, 500 au plus (`:68`, `:74`). La réponse
  (`ReconciliationProposal`, `:108-117`) porte déjà un champ additif `notice` (Story 30-1b) : le patron du champ neuf.
- **Le second côté d'un virement** : deux comptes bancaires peuvent partager un compte du grand livre (aucune contrainte
  `UNIQUE` sur `bank_accounts.journal_account_id`, `20260507200001_bank_account_journal_link.sql:21-23` ; la lecture `sharing`,
  `kesh-db/src/repositories/bank_accounts.rs:1789`, `:1853`). D'où le prédicat « compte du grand livre **distinct** » de la
  v017-3c (finding R1 = F6 de la P2 de la v017-3a) : sans lui, une transaction non comptabilisée d'un compte qui partage son
  grand livre serait dite « l'autre côté d'un virement ».
- **Le manuel** dit déjà la conduite pour le second côté — « rejetez-la (issue #823) » (`docs/manual/fr/user-manual.tex:2915-2921`,
  `:2991`) — et pour une dépense réglée par un autre chemin (`:2895-2905`) ; il ne dit pas que l'écran le signale, puisqu'il
  ne le signale pas.
- **L'écran** : chaque ligne porte *Écriture manuelle* (34-3), *Éclater* et, si `canLink(p)` (`ReconciliationProposals.svelte:254-256`),
  *Rattacher à une écriture existante* (`:615-629`) ; *Accepter (N)* traite la sélection (`onAccept`, `:288-330`). Aucun
  dialogue de confirmation n'existe dans ce composant ; `$lib/components/ui/dialog` est celui des autres dialogues de la page.
- **Rien ne mesure aujourd'hui le coût de la page.** Précédent d'un banc ignoré : `export_global_zip_large_dataset_perf`
  (`crates/kesh-api/tests/exports_global_e2e.rs:1011`), `#[ignore]`, joué à la main.

## Acceptance Criteria

### Serveur

1. **AC-1 — le champ** : chaque `ReconciliationProposal` porte `alreadyBooked`, **additif** :
   `{ settlementCount: number, entryCount: number, matchedElsewhereCount: number } | null`. `null` quand la transaction ne peut
   rien avoir à rattacher (devise autre que CHF, montant nul) ou que le compte bancaire n'a pas de compte du grand livre ;
   sinon les trois nombres, zéro compris. `settlementCount = find_link_candidates(…).len()` et
   `entryCount = find_entry_candidates(…).len()` : **les mêmes fonctions** que le dialogue, mêmes paramètres (tolérance
   `AMOUNT_TOLERANCE_HUNDREDTHS`), si bien qu'un compte positif garantit une liste non vide à l'ouverture du dialogue.
   Test HTTP : pour un règlement client sur ce compte, une écriture saisie au journal, et aucun des deux, le champ égale la
   longueur des deux listes rendues par `GET …/link-candidates` et `GET …/entry-candidates` pour la même transaction.
2. **AC-2 — le second côté** : `matchedElsewhereCount` compte les écritures dont le mouvement sur le compte du grand livre de
   `t` égale `t.amount`, ni contre-passées ni contre-passations, **rapprochées** (`bank_transactions.matched_entry_id`) d'une
   transaction d'un **autre** compte bancaire **dont le compte du grand livre diffère** de celui de `t`. Une requête dans
   `reconciliation_link.rs`, construite à partir du même texte que `entry_candidates_sql` (un mode de plus du constructeur, non
   une copie) ; un test asserte que les deux textes ne diffèrent que par les prédicats voulus. Tests :
   - virement de A vers B, côté A rapproché par une écriture manuelle `D 1021 / C 1020` : la transaction de B porte `1` ;
   - **comptes partagés** : deux comptes bancaires sur le même compte du grand livre, l'un rapproché : l'autre porte `0` ;
   - écriture contre-passée : `0`.
3. **AC-3 — lecture seule** : `get_proposals` n'écrit rien et ne prend aucun verrou de plus ; un échec de lecture du champ
   fait échouer la route (500), comme les autres lectures de la boucle — jamais un champ silencieusement à zéro.
4. **AC-4 — aucun refus serveur** : `POST /reconciliation/accept`, `/manual`, `/split` ne changent pas ; leurs tests passent
   sans modification. Un client d'API reste libre (C-v017-3-7).

### Écran

5. **AC-5 — l'avertissement** : sur une ligne dont `settlementCount + entryCount > 0` ou `matchedElsewhereCount > 0`, un
   avertissement (`data-testid="already-booked-warning"`), sous la ligne, sans numéro d'issue :
   - (a) si `settlementCount + entryCount > 0` : « Cette transaction semble déjà comptabilisée : { $count } écriture(s) non
     rapprochée(s) du compte de la banque lui correspond(ent) — au même montant, ou, pour un règlement de facture client,
     d'au plus 5 centimes de moins. Rattachez-la (*Rattacher à une écriture existante*) plutôt que de passer une écriture :
     elle serait comptée deux fois. » (forme Fluent au pluriel par `{ $count ->}`, non « (s) », dans les quatre locales) ;
   - (b) si `matchedElsewhereCount > 0` : « Une écriture du même montant est déjà rapprochée d'une transaction d'un autre de
     vos comptes bancaires : c'est probablement l'autre côté d'un virement entre vos comptes. Ne passez pas d'écriture pour
     elle : rejetez-la. » ;
   - (c) les deux : (a) puis (b), dans cet ordre, la consigne « rejetez-la » ne valant que pour (b) — texte du cas combiné :
     (a), puis « Si aucune de ces écritures ne lui correspond, elle est peut-être l'autre côté d'un virement entre vos
     comptes : une écriture du même montant est rapprochée sur un autre de vos comptes bancaires. » (C-e342-7) ;
   - aucune phrase pour un compte nul (jamais « 0 écriture ») ; `alreadyBooked: null` : rien.
   Vitest des quatre cas (aucun, a, b, c), texte attendu complet.
6. **AC-6 — la confirmation** : *Accepter (N)*, *Écriture manuelle* et *Éclater* ouvrent d'abord un dialogue
   (`data-testid="already-booked-confirm"`) quand la sélection (ou la ligne) contient au moins une ligne signalée. Il nomme
   chaque transaction signalée (date suisse, montant, contrepartie) et propose *Continuer quand même* et *Annuler*. *Annuler*
   ne fait rien. *Continuer quand même* reprend le geste tel quel (lot entier pour *Accepter* ; ouverture du dialogue pour les
   deux autres). *Rejeter* et *Rattacher à une écriture existante* ne demandent rien (C-e342-8). Vitest : chacun des trois gestes,
   avec et sans ligne signalée ; *Annuler* n'appelle aucune API.
7. **AC-7 — après coup** : la liste est relue après chaque geste (comportement actuel) ; une ligne rattachée disparaît, et
   l'avertissement de l'autre côté d'un virement apparaît à la relecture suivante.

### Coût

8. **AC-8 — la mesure précède le remède** (C-v017-3-23, C-e342-9) : un banc `#[ignore = "banc de coût (AC-8) : …"]` dans
   `crates/kesh-api/tests/`, sur un historique synthétique de **1 000, 5 000 et 20 000 écritures** sur le compte du grand livre
   et **500 transactions en attente**, trois mesures par palier, profil `--release`, chiffres au Change Log et dans
   `kesh-gate-logs/e342/`. **Mesuré avant et après** le champ neuf (le coût ajouté est la différence). « Mauvaise pente » :
   plus de **3 s** pour la page à 20 000, ou croissance plus que linéaire du coût ajouté entre 5 000 et 20 000.
   - Pente bonne : rien de plus.
   - Pente mauvaise : le remède nommé est appliqué **dans cette story** — une lecture par page (une `SUM` par écriture filtrée
     `HAVING movement IN (…)` sur les montants de la page, appariement en Rust), pour les trois comptes —, à condition qu'il
     rende **les mêmes nombres** que les fonctions du dialogue (test d'égalité sur un jeu de 50 transactions variées). S'il
     ne peut pas les rendre exactement, section *Performance debt* de cette fiche, chiffres, et **issue demandée**.
9. **AC-9 — textes** : manuel utilisateur (§ *Réconciliation* : l'avertissement, ses deux formes, la confirmation ; le
   paragraphe `:2915-2921` dit désormais que l'écran signale ces cas), `docs/api-external.md` (le champ additif `alreadyBooked`,
   « depuis la v0.17.6 », dans le § des propositions, `:631`), quatre catalogues i18n, PDF régénéré et contrôlé aplati.
10. **AC-10 — E2E** (`reconciliation-already-booked.spec.ts`, neuf) : un Comptable saisit par l'API une écriture
    `D charge / C banque` d'un montant unique, puis un relevé dont un débit l'égale (`createManualDebitStatementViaApi`) ;
    *Réconciliation* : la ligne porte l'avertissement (a) ; *Écriture manuelle* → la confirmation nomme la transaction →
    *Annuler* → aucun dialogue d'écriture ; *Rattacher à une écriture existante* → l'écriture → la ligne quitte la liste.
    En `finally` : rattachement annulé, écriture contre-passée, import supprimé. Le second côté (b) est couvert par le test
    HTTP AC-2 et le Vitest AC-5 ; un E2E à deux comptes bancaires n'est pas exigé (C-e342-8).

## Tâches

- [ ] **T0 — étape 0** : relire chaque prémisse au commit de départ ; vérifier que 34-3 est fusionnée (libellés) ou, sinon,
  écrire les textes avec « Écriture manuelle » et le dire au Change Log.
- [ ] **T1 — mesure d'avant** : le banc AC-8, sans le champ ; chiffres journalisés.
- [ ] **T2 — rouge** : tests `kesh-db` (les trois comptes, comptes partagés, contre-passation, égalité des textes SQL), test
  HTTP AC-1, Vitest AC-5 et AC-6, E2E AC-10 ; rouges journalisés (`kesh-gate-logs/e342/rouges.log`).
- [ ] **T3** : `reconciliation_link.rs` (mode `matched_elsewhere` du constructeur ; une fonction de comptage qui appelle les
  deux fonctions existantes) ; `get_proposals` (le champ, dans la boucle Pass 2 ou après, avec le compte du grand livre déjà lu) ;
  `kesh-db` touché : **gate complet même en cours de boucle**.
- [ ] **T4 — mesure d'après**, et le remède AC-8 si la pente l'exige.
- [ ] **T5** : `reconciliation.types.ts` (`alreadyBooked`), `ReconciliationProposals.svelte` (avertissement, dialogue de
  confirmation, gardes des trois gestes) ; i18n ; manuel, PDF ; `docs/api-external.md` ; CHANGELOG *Ajouté* (`refs #828`).
- [ ] **T6** : captures (thème clair, base jetable) : ligne (a), ligne (b), cas (c), confirmation ouverte sur *Accepter (N)*.

### Mutations nommées

| # | Mutation | Doit rougir |
|---|---|---|
| M1 (ex-M13) | `entryCount` toujours 0 | test HTTP AC-1 |
| M2 | `settlementCount` lu sans la tolérance (montant exact) | test HTTP AC-1, cas d'un règlement 5 centimes en dessous |
| M3 (ex-M21) | prédicat « compte du grand livre différent » retiré | test « comptes partagés » |
| M4 | contre-passation non écartée dans `matched_elsewhere` | test de la contre-passation |
| M5 (ex-M15) | confirmation sautée pour *Écriture manuelle* | Vitest AC-6 |
| M6 | confirmation demandée aussi pour *Rejeter* | Vitest AC-6 |
| M7 | phrase (a) affichée à compte nul | Vitest AC-5, cas (b) seul |

## Gates

`kesh-db` touché : `scripts/test-fast.sh` à chaque patch, E2E complet au dernier commit de code. Banc AC-8 sous verrou
(`flock … scripts/mem-guard.sh cargo test --release … -- --ignored`), base à soi.

## Ce qui ne se traite pas

- **Le rattachement des deux côtés d'un virement** à une même écriture, et d'une transaction à plusieurs écritures : #823.
- Un paiement passé sur un autre compte du grand livre, ou un ancien : #827 (aucun signal possible sans lire les autres
  comptes ; hors périmètre de #828).
- Une écriture **rapprochée** sur ce même compte bancaire (doublon d'import) n'est pas signalée ici : la détection des lignes
  en double se fait à l'import (`post_import_skips_duplicate_lines_by_default`, `crates/kesh-api/tests/bank_imports_e2e.rs:1067`).

## Recoupements

`ReconciliationProposals.svelte` et le § *Réconciliation* du manuel avec **34-3** (libellés, avant) et **34-11b** (le
prélèvement mensuel d'une carte est un virement entre deux comptes bancaires de grands livres distincts : exactement le cas
(b) ; la 34-11b s'appuie sur ce signal). `reconciliation_link.rs` avec 34-11a si elle touche les comptes bancaires sans IBAN :
aucune ligne commune prévue.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5), sur la conception de la v017-3c.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
