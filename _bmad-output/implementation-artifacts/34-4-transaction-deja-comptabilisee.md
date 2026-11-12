# Story 34-4 : la réconciliation signale une transaction qui semble déjà comptabilisée, et demande confirmation (#828)

Status: review

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** (lecture seule, champ additif). Registre :
`e34-choix-autonomes-e342.md`, C-e342-5 à C-e342-9, C-e342-20 à C-e342-23. Conception reprise de la fiche courte
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
   ⚠️ *(F4-3)* Chaque compte vaut **au plus 50** (`LINK_CANDIDATES_LIMIT`, `ENTRY_CANDIDATES_LIMIT`, `:76`, `:504`) et ne
   porte aucune fenêtre de dates (`entry_candidates_sql` n'en a pas) : une écriture non rapprochée d'un montant récurrent (un
   loyer saisi à la main) signalera chaque transaction suivante du même montant, et demandera une confirmation à *Accepter*.
   Bruit attendu, dit au manuel ; une fenêtre de dates est hors périmètre (**issue demandée**). Même bruit, et permanent, pour
   *(P2-4-4)* l'**écriture d'ouverture** que la 34-2 passe sur le compte du grand livre de la banque (mode *Compléter*,
   contrepartie *Bénéfice/perte reporté*) : jamais rapprochée ni contre-passée, elle est candidate pour toute transaction de son
   montant ; le manuel la nomme avec le loyer. L'issue « fenêtre de dates » la couvre.
   Test HTTP : pour un règlement client sur ce compte au montant exact, un règlement **inférieur de 5 centimes** (compté,
   tolérance), un règlement inférieur de **6 centimes** (non compté), une écriture saisie au journal, et aucun des deux, le
   champ égale la longueur des deux listes rendues par `GET …/link-candidates` et `GET …/entry-candidates` pour la même
   transaction *(P2-4-2 : les cas de la mutation M2)*.
   *(Annoté le 2026-11-12, revues de code P1 et P2, C-e344-9 et C-e344-12 : le champ est aussi `null` quand le compte
   bancaire est **clos** — les routes du dialogue le refusent en 409. Depuis le remède de l'AC-8, les deux premiers nombres ne
   sont plus obtenus en appelant les fonctions du dialogue, mais par `count_already_booked_for_page`, deux lectures par page
   sur les **mêmes textes SQL** ; ils les égalent, ce que vérifie
   `s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue`. `entry_candidates_sql` est devenu
   `entry_sql(EntrySqlMode)`.)*
2. **AC-2 — le second côté** : `matchedElsewhereCount` compte les écritures dont le mouvement sur le compte du grand livre de
   `t` égale `t.amount`, ni contre-passées ni contre-passations, **rapprochées** (`bank_transactions.matched_entry_id`) d'une
   transaction d'un **autre** compte bancaire **dont le compte du grand livre diffère** de celui de `t`. Une requête dans
   `reconciliation_link.rs`, construite à partir du même texte que `entry_candidates_sql` (un mode de plus du constructeur, non
   une copie) ; un test asserte que les deux textes ne diffèrent que par les prédicats voulus. Tests (ils n'assertent que
   `matchedElsewhereCount`, jamais l'absence de candidate : la 34-11b rend cette écriture **candidate**, F4-2) :
   - virement de A vers B, côté A rapproché par une écriture manuelle `D 1021 / C 1020` : la transaction de B porte `1` ;
   - **comptes partagés** : deux comptes bancaires sur le même compte du grand livre, l'un rapproché : l'autre porte `0` ;
   - écriture contre-passée : `0`.

   *(Annoté le 2026-11-12, revues de code P1 et P2, choix C-e344-11 qui amende C-e344-2. L'énoncé ci-dessus, « d'un **autre**
   compte bancaire **dont le compte du grand livre diffère** de celui de `t` », n'est **plus** la règle : il dépendait du lien
   actuel d'un compte bancaire à son grand livre, modifiable après coup (finding E1-1). **Règle en vigueur** :
   `matchedElsewhereCount` compte les écritures dont le mouvement `M` sur le compte du grand livre de `t` égale `t.amount`, ni
   contre-passées ni contre-passations, **rapprochées d'une transaction d'un autre compte bancaire, de sens contraire** — de
   montant `−M` (`bt.bank_account_id <> ? AND bt.amount = -m.movement`, constante `MATCHED_ELSEWHERE`, mode
   `EntrySqlMode::MatchedElsewhere`). Aucun lien de compte au grand livre n'y entre. Tests ajoutés :
   `s34_4_ac2_compte_relie_ailleurs_ne_fait_pas_rejeter` (E1-1), `s34_4_ac2_le_second_cote_d_un_debit` (E2-4),
   `s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas` (B2-1, M10). Le cas « comptes partagés » vaut toujours `0`, parce
   que les deux transactions vont dans le même sens.)*
3. **AC-3 — lecture seule** : `get_proposals` n'écrit rien et ne prend aucun verrou de plus ; un échec de lecture du champ
   fait échouer la route (500), comme les autres lectures de la boucle — jamais un champ silencieusement à zéro.
4. **AC-4 — aucun refus serveur** : `POST /reconciliation/accept`, `/manual`, `/split` ne changent pas ; leurs tests passent
   sans modification. Un client d'API reste libre (C-v017-3-7).

### Écran

5. **AC-5 — l'avertissement** : sur une ligne dont `settlementCount + entryCount > 0` ou `matchedElsewhereCount > 0`, un
   avertissement (`data-testid="already-booked-warning"`), sous la ligne, sans numéro d'issue. **Préfixe des clés** *(Validation
   P3, P3-4-1)* : dans cette fiche, `…-already-booked-…` se lit **`reconciliation-labels-already-booked-…`**, le préfixe des avis
   de ligne (`reconciliation-labels-notice-…`) ; `frontend/scripts/lint-i18n-ownership.js` admet une clé `reconciliation-*` dans
   `src/lib/features/reconciliation/`, où vit `ReconciliationProposals.svelte` :
   - (a) si `settlementCount + entryCount > 0` : **deux clés**, choisies par le Svelte selon le nombre —
     `…-already-booked-one` : « Cette transaction semble déjà comptabilisée : une écriture non rapprochée du compte de la
     banque lui correspond. Rattachez-la (*Rattacher à une écriture existante*) plutôt que de passer une écriture : elle serait
     comptée deux fois. » *(P2-4-3 : la clause « ou, pour un règlement de facture client, d'au plus 5 centimes de moins »
     est retirée — elle s'affichait aussi sur un débit, où `settlementCount` vaut toujours 0 ; le dialogue de rattachement montre
     les montants)* ;
     `…-already-booked-other` : la même au pluriel, « { $count } écritures non rapprochées […] lui correspondent ». ⛔ **Pas de
     sélecteur Fluent** (`{ $count -> … }`) *(F4-1)* : le client reçoit les messages **pré-résolus sans arguments**
     (`crates/kesh-i18n/src/loader.rs:130-165`, `all_messages`) et `i18nMsg` ne remplace que `{ $nom }`
     (`frontend/src/lib/shared/utils/i18n.svelte.ts:19-27`) : un sélecteur arriverait figé sur sa variante par défaut. Patron
     existant : `homepage-charts-credit-balance-note-one` / `-other` (`ExpensePieCard.svelte:170-178`). Un test lit le
     **catalogue réel** (fr `messages.ftl`), non le seul repli, et vérifie qu'aucune des clés neuves ne porte `->` ;
   - (b) si `matchedElsewhereCount > 0` (clé `…-already-booked-elsewhere`) : « Une écriture du même montant est déjà rapprochée d'une transaction d'un autre de
     vos comptes bancaires : c'est probablement l'autre côté d'un virement entre vos comptes. Ne passez pas d'écriture pour
     elle : rejetez-la. » ;
   - (c) les deux : (a) puis (b), dans cet ordre, la consigne « rejetez-la » ne valant que pour (b) — texte du cas combiné :
     (a), puis (clé `…-already-booked-combined`) « Si aucune de ces écritures ne lui correspond, elle est peut-être l'autre côté d'un virement entre vos
     comptes : une écriture du même montant est rapprochée sur un autre de vos comptes bancaires. » (C-e342-7) ;
   - ⚠️ **(b), (c) et leur condition d'affichage sont provisoires** *(F4-2, précisé à la validation P2, P2-4-1)* : la 34-11b
     (C-e341-22) rend l'écriture d'un virement **candidate** de l'autre côté ; `matchedElsewhereCount` devient alors un
     sous-ensemble de `entryCount`. Avec la condition ci-dessus, le cas (b) **seul** ne s'afficherait plus jamais, et c'est (c)
     qui s'afficherait — en disant « si aucune de ces écritures ne lui correspond » d'une écriture qui est justement celle à
     rattacher. **Ce que la 34-11b, seconde à s'intégrer, doit réécrire** — à transmettre à son agent, la fiche 34-11b n'étant
     pas de ce groupe :
     1. **Condition d'affichage**, avec `k = settlementCount + entryCount` et `m = matchedElsewhereCount` (`m ≤ entryCount`) :
        (a) si `k − m > 0` et `m = 0` ; **(b′)** si `m > 0` et `k − m = 0` ; **(c′)** si `m > 0` et `k − m > 0` ; rien si `k = 0`.
        La confirmation (AC-6) reste déclenchée par `k > 0`.
     2. **(b′)**, clé neuve `…-already-booked-transfer` : « Cette transaction est probablement l'autre côté d'un virement entre
        vos comptes : l'écriture de ce virement est déjà rapprochée d'une transaction d'un autre de vos comptes bancaires.
        Rattachez-la à cette écriture (*Rattacher à une écriture existante*) plutôt que de passer une écriture : le virement
        serait compté deux fois. »
     3. **(c′)**, clé neuve `…-already-booked-transfer-among` : le texte (a) (clé `-one` ou `-other`), puis « L'une d'elles est
        l'écriture d'un virement déjà rapprochée sur un autre de vos comptes bancaires : c'est probablement celle-ci qu'il faut
        rattacher. »
     4. Les clés (b) `…-already-booked-elsewhere` et (c) de cette story sont **retirées** (quatre locales, replis) ; le Vitest
        des cas (aucun, a, b′, c′) remplace celui de (aucun, a, b, c) ; une mutation « condition (b′) sur `m > 0` seul » rougit
        le cas (c′).
     5. Le manuel (`:2915-2921`, `:2991`), le texte que cette story y écrit (AC-9), et la ligne #823 du § *Ce qui ne se traite
        pas* ci-dessous.
     Si la 34-11b ne part pas dans la 0.17.6, les textes (b) et (c) de cette story restent justes tels quels ;
   - aucune phrase pour un compte nul (jamais « 0 écriture ») ; `alreadyBooked: null` : rien.
   Vitest des quatre cas (aucun, a, b, c), texte attendu complet.

   *(Annoté le 2026-11-12, revues de code P1 et P2, C-e344-3, C-e344-11 et C-e344-13 : les textes **en vigueur** ne sont plus
   ceux cités ci-dessus. Le bouton est nommé entre guillemets : « Rattacher à une écriture existante ». (b) : « Une transaction
   de sens contraire, sur un autre de vos comptes bancaires, est déjà rapprochée d'une écriture du même montant : c'est
   probablement l'autre côté d'un virement entre vos comptes. Ne passez pas d'écriture pour elle : rejetez-la. » ; (c), après
   (a) : « Si rien de ce que propose « Rattacher à une écriture existante » ne lui correspond, elle est peut-être l'autre côté
   d'un virement entre vos comptes : une transaction de sens contraire, sur un autre de vos comptes bancaires, est rapprochée
   d'une écriture du même montant. » La condition d'affichage ne change pas. Ce que la 34-11b doit réécrire (le point 1
   ci-dessus et sa prémisse « `m ≤ entryCount` ») ne tient plus tel quel : voir `kesh-gate-logs/e34-transmission-34-11b.md`,
   mis à jour le 2026-11-12.)*
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
   et **500 transactions en attente**, page demandée à `limit=500` (`MAX_PROPOSALS_LIMIT`, `reconciliation.rs:74`), trois
   mesures par palier, profil `--release`, chiffres au Change Log et dans
   `kesh-gate-logs/e342/`. **Mesuré avant et après** le champ neuf (le coût ajouté est la différence). « Mauvaise pente » :
   plus de **3 s** pour la page à 20 000, ou croissance plus que linéaire du coût ajouté entre 5 000 et 20 000. ⚠️ Sur la base
   de dev en tmpfs, les chiffres sont un **plancher** (le NAS lit sur disque) : le Change Log le dit avec eux.
   - Pente bonne : rien de plus.
   - Pente mauvaise : le remède nommé est appliqué **dans cette story** — une lecture par page (une `SUM` par écriture filtrée
     `HAVING movement IN (…)` sur les montants de la page, appariement en Rust), pour les trois comptes —, à condition qu'il
     rende **les mêmes nombres** que les fonctions du dialogue (test d'égalité sur un jeu de 50 transactions variées). S'il
     ne peut pas les rendre exactement, section *Performance debt* de cette fiche, chiffres, et **issue demandée**.
9. **AC-9 — textes** : manuel utilisateur (§ *Réconciliation* : l'avertissement, ses deux formes, la confirmation ; le
   paragraphe `:2915-2921` dit désormais que l'écran signale ces cas), `docs/api-external.md` (le champ additif `alreadyBooked`,
   « depuis la v0.17.6 », dans le § des propositions où figure `notice`, `:657`), quatre catalogues i18n ; le bruit d'un
   montant récurrent (AC-1) dit au manuel. Le `.tex` change dans la story ; le PDF de `user-manual`, touché par quatre stories
   de l'epic, se **régénère une fois à l'intégration** et s'y contrôle aplati (un PDF ne se fusionne pas).
10. **AC-10 — E2E** (`reconciliation-already-booked.spec.ts`, neuf) : un Comptable saisit par l'API une écriture
    `D charge / C banque` d'un montant unique, puis un relevé dont un débit l'égale (`createManualDebitStatementViaApi`) ;
    *Réconciliation* : la ligne porte l'avertissement (a) ; *Écriture manuelle* → la confirmation nomme la transaction →
    *Annuler* → aucun dialogue d'écriture ; *Rattacher à une écriture existante* → l'écriture → la ligne quitte la liste.
    En `finally` : rattachement annulé, écriture contre-passée, import supprimé. Le second côté (b) est couvert par le test
    HTTP AC-2 et le Vitest AC-5 ; un E2E à deux comptes bancaires n'est pas exigé (C-e342-8).

## Tâches

- [x] **T0 — étape 0** : relire chaque prémisse au commit de départ ; vérifier que 34-3 est fusionnée (libellés) ou, sinon,
  écrire les textes avec « Écriture manuelle » et le dire au Change Log.
- [x] **T1 — mesure d'avant** : le banc AC-8, sans le champ ; chiffres journalisés.
- [x] **T2 — rouge** : tests `kesh-db` (les trois comptes, comptes partagés, contre-passation, égalité des textes SQL), test
  HTTP AC-1, Vitest AC-5 et AC-6, E2E AC-10 ; rouges journalisés (`kesh-gate-logs/e342/rouges.log`).
- [x] **T3** : `reconciliation_link.rs` (mode `matched_elsewhere` du constructeur ; une fonction de comptage qui appelle les
  deux fonctions existantes) ; `get_proposals` (le champ, dans la boucle Pass 2 ou après, avec le compte du grand livre déjà lu) ;
  `kesh-db` touché : **gate complet même en cours de boucle**.
- [x] **T4 — mesure d'après**, et le remède AC-8 si la pente l'exige.
- [x] **T5** : `reconciliation.types.ts` (`alreadyBooked`), `ReconciliationProposals.svelte` (avertissement, dialogue de
  confirmation, gardes des trois gestes) ; i18n ; manuel (`.tex` ; PDF à l'intégration) ; `docs/api-external.md` ; CHANGELOG *Ajouté* (`refs #828`).
- [x] **T6** : captures (thème clair, base jetable) : ligne (a), ligne (b), cas (c), confirmation ouverte sur *Accepter (N)*.

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
| M8 | clé `-other` employée pour un compte de 1 *(jouée : `if (k === 1 && false)` et `k > 1` → `k >= 1` — la phrase `-one` n'est plus jamais choisie, `-other` prend le compte de 1 ; description précisée à la revue P2, B2-6)* | Vitest AC-5, cas (a) à une écriture |
| M10 (revue P2, B2-1) | prédicat `bt.bank_account_id <> ?` neutralisé (`? IS NOT NULL`) | `s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas` |
| M9 (revue P1, E1-3) | `manual` et `split` intervertis dans `runGesture` | Vitest AC-6, doublures distinctes (`data-modal`) |

*(Annoté à la remédiation P1, 2026-11-12 : depuis C-e344-11, M3 vise le prédicat « transaction de sens contraire »
(`bt.amount = -m.movement` → `TRUE`) qui remplace le prédicat « grand livre différent » ; elle rougit « comptes partagés » et
« compte relié ailleurs ».)*

## Gates

`kesh-db` touché : `scripts/test-fast.sh` à chaque patch, E2E complet au dernier commit de code. Banc AC-8 sous verrou
(`flock … scripts/mem-guard.sh cargo test --release … -- --ignored`), base à soi.

## Ce qui ne se traite pas

- **Le rattachement des deux côtés d'un virement** à une même écriture, et d'une transaction à plusieurs écritures : #823
  (pour les deux côtés d'un virement, la 34-11b le traite, et réécrit alors cette ligne, AC-5).
- Une fenêtre de dates pour le signal d'un montant récurrent (AC-1) : **issue demandée**.
- Un paiement passé sur un autre compte du grand livre, ou un ancien : #827 (aucun signal possible sans lire les autres
  comptes ; hors périmètre de #828).
- Une écriture **rapprochée** sur ce même compte bancaire (doublon d'import) n'est pas signalée ici : la détection des lignes
  en double se fait à l'import (`post_import_skips_duplicate_lines_by_default`, `crates/kesh-api/tests/bank_imports_e2e.rs:1067`).

## Recoupements

`ReconciliationProposals.svelte` et le § *Réconciliation* du manuel avec **34-3** (libellés, avant) et **34-11b** (le
prélèvement mensuel d'une carte est un virement entre deux comptes bancaires de grands livres distincts : exactement le cas
(b) ; ordre recommandé par les deux fiches : 34-4, puis 34-11b, qui reprend le mode `matched_elsewhere` dans le même
constructeur et réécrit les textes (b) et (c), AC-5). `reconciliation_link.rs` avec 34-11b (condition 4) ; avec 34-11a :
aucune ligne commune prévue.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5), sur la conception de la v017-3c.
- 2026-11-10 : Validation P1 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p1-e342.md`, pour cette
  fiche 0 C, 1 H, 1 M, 3 LOW, tous retenus après lecture : **F4-1** HIGH (lus : `i18n.svelte.ts:19-27`, `i18nMsg` ne remplace
  que `{ $nom }` ; `loader.rs:130-165`, messages pré-résolus sans arguments) → deux clés `-one`/`-other`, contrôle au
  catalogue réel, M8 ; **F4-2** (lu : `34-11b-carte-credit-releve.md`, C-e341-22 et § *Recoupements*) → textes (b)/(c)
  provisoires, réécrits par la 34-11b, tests AC-2 sur `matchedElsewhereCount` seul ; **F4-3** (lus : `reconciliation_link.rs:76`,
  `:504`) → plafond 50 et bruit d'un montant récurrent dits, fenêtre en issue demandée ; **F4-4** (lu : `api-external.md:631`
  est `accept`, `:657` porte `notice`) → `:657` ; **F4-5** → `limit=500`, chiffres tmpfs = plancher. Transversal **X-1** : PDF à
  l'intégration (AC-9).
- 2026-11-10 : Validation P2 (Opus), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p2-e342.md`, pour cette
  fiche 0 C, 0 H, 1 M, 3 LOW, tous retenus après lecture :
  - **P2-4-1** MEDIUM, **né de la remédiation P1** (F4-2 annonçait que la 34-11b réécrirait (b) et (c) ; lu :
    `34-11b-carte-credit-releve.md:92-95`, `:153-157`, elle ne réécrit que (b)) : la condition d'affichage et les textes (b′),
    (c′) que la 34-11b doit écrire sont posés à l'AC-5, à transmettre par l'orchestrateur (fiche 34-11b non modifiée ici) ;
    clés nommées (`-elsewhere`, `-combined`) pour qu'elle les retire ;
  - LOW : **P2-4-2** → règlements à −5 et −6 centimes à l'AC-1 ; **P2-4-3** (lu : `reconciliation_link.rs:83-85`, crédits
    seulement) → clause des 5 centimes retirée du texte (a) ; **P2-4-4** (lu : `34-2-solde-ouverture-compte-bancaire.md:17-19`)
    → écriture d'ouverture nommée au bruit attendu.
  - CHANGELOG : *Ajouté* (C-e342-22).
- 2026-11-10 : Validation P3 (Sonnet), remédiation (Opus 5.5). Rapport `kesh-gate-logs/e34-validate-p3-e342.md`, pour cette
  fiche 0 C, 0 H, 0 M, 1 LOW, retenu : **P3-4-1** (lu : `frontend/scripts/lint-i18n-ownership.js:1-20`, propriété par dossier
  de fonctionnalité) → préfixe `reconciliation-labels-already-booked-…` écrit à l'AC-5 ; il vaut aussi pour les clés que la
  34-11b ajoutera (`-transfer`, `-transfer-among`).
- 2026-11-11 : développement (Opus 5.5, clé `e344`, worktree `kesh-e344`, base `f12ad73e` = tête de la 34-3). Commits
  `ff7414d2` (code, manuel, catalogues, E2E), `c3bcbff8` (registre, prompt de revue), `dc4f72d6` (garde i18n), puis le commit
  de clôture du développement. **Le remède de l'AC-8 a été appliqué** (C-e344-9). Détail au Dev Agent Record.
  Banc (journaux `kesh-gate-logs/e344/banc-*.log`, `--release`, tmpfs = plancher) : environ 0,2 à 0,5 s avant le champ ;
  **mauvaise pente** avec trois lectures par transaction (≈ 18,7 s à 20 000 écritures, 500 transactions) ; ≈ 0,3 s après le
  remède — chiffres au § *Performance*. Les chiffres vont dans `kesh-gate-logs/e344/` (la clé de développement), non
  `e342/` comme l'écrit l'AC-8 (clé de la spécification).
- 2026-11-12 : revue de code P1 (Opus, trois lentilles ; rapports `kesh-gate-logs/e344-review-p1-{B,E,A}.md`) : 0 C, 0 H,
  **3 MEDIUM distincts**, ≈ 20 LOW. Remédiation (Opus 5.5), commit `e29a85ca` ; tous retenus après lecture :
  - **E1-1** MEDIUM (lu : `reconciliation_link.rs:654` à `46c9d755`, `NOT (ba.journal_account_id <=> ?)` ; rouge reproduit,
    `matchedElsewhereCount = 1` pour un compte relié au 1022 après coup) → **C-e344-11**, qui amende C-e344-2 et le texte de
    l'AC-2 (« dont le compte du grand livre diffère ») : la règle de l'arbitrage, prise à la lettre (« une ligne sur le grand
    livre du compte courant »), est déjà vraie de toute écriture examinée et ne corrige rien — c'est consigné ; le second
    côté se fonde désormais sur la transaction rapprochée **de sens contraire** d'un autre compte bancaire
    (`bt.amount = -m.movement`), sans lien actuel à un grand livre. Test `s34_4_ac2_compte_relie_ailleurs_ne_fait_pas_rejeter`
    (relié au 1022, puis délié) ; manuel, `api-external.md`, textes (b) et (c) dans les quatre locales suivent.
  - **B1-1 = E1-2 = A1-1** MEDIUM, **B1-2 = E1-4 = A1-3** : `api-external.md` et le doc-comment de `AlreadyBooked` décrivent la
    lecture par page (deux lectures, mêmes textes SQL, égalité testée) ; lien rustdoc corrigé.
  - **E1-3 = A1-2** MEDIUM : une doublure par modale (`ModalSuccessStub{Manual,Split,Link}.test.svelte`, `data-modal`) ; les
    tests de *Continuer* et des lignes non signalées assertent la modale ouverte ; **M9** (branches interverties) tuée.
  - LOW : **B1-3** → C-e344-12, compte clos `null` ; **B1-4 = A1-5** → manuel (« chaque liste s'arrête à 50, le nombre affiché
    les additionne ») ; **B1-5, A1-10** → C-e344-13, textes neutres en nombre ; **B1-6** → C-e344-16, `One(i64)` ;
    **B1-7** → E2E *Continuer* puis le dialogue d'écriture, en navigateur ; **B1-8** → C-e344-15, montant laissé à la forme
    de la ligne, **issue demandée** ; **E1-5** → autre société dans le test d'égalité ; **E1-6** → Vitest Échap ; **E1-7** →
    nettoyage de l'E2E robuste ; **E1-9** → le banc rapproche la moitié de l'historique (`banc-p1-apres.log`) ; **E1-10** →
    C-e344-14, ordre écrit au manuel ; **A1-4** → manuel ; **A1-6** → hash au Dev Agent Record, journaux de la remédiation
    préfixés du commit ; **A1-7** → chiffres ci-dessus ; **A1-8** → plafonds des règlements et du second côté exercés ;
    **A1-9** → déclaré au Dev Agent Record (le test unitaire d'égalité des textes n'a pas été joué au rouge : il ne compile pas
    contre la base). **E1-8** : déclaré, rien d'autre.
- 2026-11-12 : revue de code P2 (Sonnet, trois lentilles, passe complète ; rapports `kesh-gate-logs/e344-review-p2-{B,E,A}.md`) :
  0 C, 0 H, **2 MEDIUM**, ≈ 14 LOW. Les trois lentilles jugent C-e344-11 juste pour les cas de la fiche ; l'orchestrateur la
  retient. Remédiation (Opus 5.5), commit `cf585bff` ; tous retenus après lecture :
  - **A2-1 = E2-5 = B2-3** MEDIUM : AC-1 (compte clos, lecture par page), AC-2 (règle de sens contraire) et AC-5 (textes en
    vigueur) annotés, l'énoncé d'origine laissé lisible.
  - **E2-1 = B2-4** MEDIUM, **A2-2** : `kesh-gate-logs/e34-transmission-34-11b.md` mis à jour (hors dépôt) : « `m` ⊂
    `entryCount` » n'est plus garanti tant que la condition 4 de la 34-11b lit le lien actuel ; deux voies écrites (aligner la
    condition 4 sur le sens contraire, recommandée ; ou des conditions sans soustraction avec l'intersection) ; noms
    `entry_sql(EntrySqlMode)`, `NOT_MATCHED`, `MATCHED_ELSEWHERE`, `count_already_booked_for_page`. La fiche 34-11b n'est pas
    touchée.
  - LOW : **B2-1 = A2-4** → `s34_4_ac2_rapprochee_sur_le_meme_compte_ne_compte_pas` (forgé) et **M10**, tuée (C-e344-18) ;
    **E2-4** → `s34_4_ac2_le_second_cote_d_un_debit` ; **E2-2 = A2-3, E2-3, B2-2** → manuel (règle de sens, faux négatifs des
    frais, de l'arrondi et du virement en plusieurs écritures, doublon de deux transactions du même montant) ; **B2-5** →
    C-e344-17, *Continuer* relit la ligne dans la liste courante, la liste du dialogue n'est plus vidée à la fermeture ;
    **B2-6** → description de M8 ; **B2-7** → allemand sans « Sie würde » ambigu (« Die Bewegung würde sonst doppelt
    gezählt », « Lehnen Sie die Transaktion ab »). Note d'E sur *Rejeter* sans confirmation : issue demandée (P3).

## Performance

Banc `banc_cout_propositions_deja_comptabilisees` (`#[ignore]`, `--release`, base de dev en **tmpfs** : ces chiffres sont un
**plancher**, le NAS lit sur disque), 500 transactions en attente, `limit=500`, trois mesures par palier, en ms
(journaux `kesh-gate-logs/e344/banc-avant.log`, `banc-apres.log`, `banc-apres-remede.log`) :

| Palier (écritures) | avant le champ | trois lectures par transaction | après le remède (deux par page) |
|---|---|---|---|
| 1 000 | 392 · 432 · 385 | 2 780 · 2 200 · 1 681 | 340 · 427 · 284 |
| 5 000 | 253 · 360 · 285 | 4 877 · 5 019 · 5 025 | 364 · 457 · 424 |
| 20 000 | 537 · 181 · 245 | 18 809 · 18 812 · 18 605 | 365 · 338 · 279 |

La première écriture (les fonctions du dialogue appelées par transaction) dépassait le seuil de 3 s à 20 000 écritures :
« mauvaise pente ». Le coût ajouté croissait à peu près linéairement (≈ 4,7 s à 5 000, ≈ 18,4 s à 20 000), mais au-delà du
seuil. Le remède nommé par l'AC-8 rend **les mêmes nombres** (test `s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue`,
50 transactions variées) ; après lui, le coût ajouté se perd dans le bruit de la mesure. Aucune dette de performance.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, en autonomie).

### Completion Notes List

**Étape 0.** Le dernier rapport de validation qui porte sur la fiche (`e34-validate-p4-e342.md`) n'a aucun finding sur elle
(« P3-4-1 … Juste ») ; ceux de la P3 sont déjà remédiés. Prémisses relues au commit de départ :
- `find_link_candidates` / `candidates_sql` et `find_entry_candidates` / `entry_candidates_sql`
  (`crates/kesh-db/src/repositories/reconciliation_link.rs`, lus en entier) : conformes à la fiche, plafonds 50 (`:76`,
  `:504` au départ), crédits seulement pour les règlements (`transaction_can_be_linked`).
- `get_proposals` (`routes/reconciliation.rs:974`) : lit `ba_check.journal_account_id` ; `link_preflight` refuse un compte
  clos (409) que `get_proposals` ne refuse pas — sans effet ici, un compte clos n'a pas de transaction en attente.
- La 34-3 est la base : « Écriture manuelle » (`reconciliation-manual-button-label`), les textes la nomment.
- Manuel : les lignes citées par la fiche ont glissé (la phrase « l'écran ne le signale pas » est à `:2901` à la base ;
  « Restent sans écriture… » à `:2920-2925` ; le second côté à `:2970` et `:3000`).
- L'issue « fenêtre de dates » demandée par la fiche existe : **#982** ; le manuel et le CHANGELOG la citent.

**Rouges d'abord** (`kesh-gate-logs/e344/rouges.log`). Les tests neufs, joués contre les sources de la base (fichiers du serveur
et composant remis temporairement à `f12ad73e` par `git restore --source`, puis restaurés) : backend 6/6 rouges (« champ
alreadyBooked absent ») ; Vitest 9 rouges sur les 19 neufs (les 10 autres assertent une absence et ne peuvent rougir qu'à une
mutation). ⚠️ **L'E2E AC-10 n'a pas été joué au rouge** : il aurait fallu reconstruire le serveur et le frontend de la base
pour un seul spec ; il asserte le texte de l'avertissement servi par le vrai serveur, absent de la base.

**Le remède de l'AC-8** (C-e344-9) : voir § *Performance*. La fonction par transaction de la première écriture est remplacée
par `count_already_booked_for_page` ; les conditions SQL sont extraites en constantes partagées avec les requêtes du dialogue
(aucune copie) ; `find_matched_elsewhere` reste le mode `MatchedElsewhere` du constructeur `entry_sql`, référence du second
côté, comme l'AC-2 le demande. Le wrapper `entry_candidates_sql` est retiré (il n'avait plus d'appelant hors test).

**Mutations** (`kesh-gate-logs/e344/mut/M1.log` à `M8.log`, résumé `mut/resume.log`) : les huit de la fiche sont tuées, chacune
par le test que la fiche nomme. M1 et M2 sont jouées dans `count_already_booked_for_page` (le code qui sert la route depuis le
remède). M3 et M4 mutent les constantes partagées : le test d'égalité AC-8 reste vert sous elles (les deux côtés lisent la même
constante), ce sont les tests HTTP AC-2 qui rougissent. M6 est plus large que son intitulé (hors ligne signalée, elle lance
*Accepter* au lieu de *Rejeter*) : six rouges sans rapport s'ajoutent à celui du test nommé.

**Gates au dernier commit de code (`dc4f72d6`)** — ce qui a réellement tourné, journaux sous `kesh-gate-logs/e344/` :
- base `kesh_e344` remise à neuf ; `scripts/test-fast.sh --ci` : **5146 passés, 13 ignorés** (`gate-test-fast.log`, joué au
  commit `ff7414d2` — le commit `dc4f72d6` ne change qu'un test Vitest) ;
- `cargo build --workspace --all-targets` : vert (`gate-build-workspace.log`, relancé hors verrou sous `mem-guard`, consigne de
  l'orchestrateur) ;
- `scripts/gate-ci.sh kesh_ci_e344` : **5153 passés, 0 échoué, 17 ignorés** (`gate-ci.log`) ;
- frontend : `lint-i18n-ownership` vert ; `npm run check` 0 erreur, 29 avertissements, tous hors des fichiers de la story
  (`gate-front-check.log`) ; Vitest complet : **premier passage 1 rouge** (`i18n-keys.test.ts`, garde du nombre de sites
  littéraux 2924 → 2932, +8 : les quatre textes de `already-booked.ts` et les quatre du dialogue), corrigé au commit
  `dc4f72d6`, rejoué : **2943 passés, 225 fichiers** (`gate-front-vitest-2.log`) ; `npm run build` vert ;
- **E2E complet** sur `kesh_e2e_e344` recréée vide, au commit `dc4f72d6` : **388 passés, 7 échecs, 17 ignorés**
  (`e2e-complet.log`). Les sept échecs sont les sept de la KF-029 (#97) de `docs/testing.md` § « Les échecs attendus » :
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`. Le spec neuf
  `reconciliation-already-booked.spec.ts` passe. Port 3344 libre avant et après, backend arrêté par son PID.
- Tests ciblés entre les étapes (`verts-backend.log`, `remede-cible.log`) : binaire `reconciliation_link_entry_e2e` entier et
  tests du module, 42/42 au commit `ff7414d2`. ⚠️ La consigne « `kesh-db` : gate complet même en cours de boucle » n'a pas été
  tenue entre la première écriture et le remède (gates ciblés seulement) ; le gate complet a tourné sur le code final.

*(Annoté le 2026-11-12, revue P1 A1-6 : les journaux `gates.log` et `gates-suite.log` portent le commit **`c3bcbff8`**, et non
`ff7414d2` ni `dc4f72d6` comme écrit ci-dessus : test-fast, gate-ci et l'E2E ont tourné à `c3bcbff8`, dont le code est celui de
`ff7414d2` (`c3bcbff8` n'ajoute que des artefacts) ; `dc4f72d6` ne change qu'un test Vitest, rejoué seul. Revue P1 A1-9 : le
test unitaire `the_matched_elsewhere_query_differs_only_by_the_matched_predicate` et le test d'égalité AC-8 n'ont pas été joués
au rouge — ils ne compilent pas contre la base, le mode et la fonction n'y existant pas.)*

**Remédiation de la revue P1** (commit **`e29a85ca`**, 2026-11-12 ; journaux sous `kesh-gate-logs/e344/p1/`, chacun préfixé
du hash du commit joué) :
- **Rouges** (`rouges.log`, entrée de la remédiation ; `p1/rouges-backend.log`, `p1/rouges-vitest.log`) : contre le code de
  `46c9d755`, 2 rouges backend pour la bonne raison (E1-1 reproduit, `matchedElsewhereCount` 1 au lieu de 0 ; B1-3, champ non
  nul sur un compte clos) ; 3 rouges Vitest (textes reformulés, garde des replis). Les tests `data-modal` (E1-3) et Échap
  (E1-6), et le test d'égalité enrichi (E1-5, A1-8), passent sur l'ancien code : ils ne rougissent que sous mutation.
- **Mutations rejouées** au commit `e29a85ca` (`mut-p1/M1.log` à `M9.log`) : les neuf tuées. M1 et M2 : tests AC-1 et AC-8 ;
  M3 (redéfinie, sens contraire neutralisé) : `comptes_partages` et `compte_relie_ailleurs` ; M4 : contre-passée ; M5 : Vitest
  de *Écriture manuelle* et Échap ; M6 : le test nommé, plus six sans rapport (inchangé) ; M7, M8 : cas AC-5 ; **M9** : les
  quatre tests qui assertent `data-modal`.
- **Banc** avec la moitié de l'historique rapprochée (`banc-p1-apres.log`, commit `e29a85ca`) : 207 · 193 · 208 ms à 1 000,
  223 · 230 · 209 ms à 5 000, 240 · 253 · 266 ms à 20 000. Bonne pente.
- **Gate complet au commit `e29a85ca`**, arbre propre : base `kesh_e344` remise à neuf ; `test-fast --ci` **5147 passés,
  13 ignorés** ; `cargo build --workspace --all-targets` vert ; `gate-ci.sh kesh_ci_e344` **5154 passés, 0 échoué,
  17 ignorés** ; `lint-i18n-ownership` vert ; `npm run check` 0 erreur (29 avertissements préexistants) ; Vitest complet
  **2944 passés, 225 fichiers** ; `npm run build` vert ; **E2E complet** sur `kesh_e2e_e344` recréée vide : **388 passés,
  7 échecs, 17 ignorés**, les sept de la KF-029 (#97) déjà nommés ; le spec neuf passe, *Continuer* puis le dialogue d'écriture
  compris. Port 3344 libre avant et après.
- **Captures refaites** (textes changés) dans `kesh-gate-logs/e344/captures-p1/` : `e344-1-ligne-a-deja-comptabilisee.png`,
  `e344-2-3-lignes-b-et-c.png` (textes (c) et (b) neufs), `e344-4-confirmation-accepter.png` (titre et introduction neufs),
  `e344-ligne-signalee.png`, `e344-confirmation-ecriture-manuelle.png`.
- **Décomptes de la remédiation** (de `46c9d755` à `e29a85ca`) : `reconciliation_link_entry_e2e.rs` 39 → 40 attributs
  `sqlx::test` (+1 : `s34_4_ac2_compte_relie_ailleurs_ne_fait_pas_rejeter`) ; Vitest 2943 → 2944 (+1, Échap) ; trois
  doublures neuves ; aucune clé i18n ajoutée, quatre textes reformulés par locale.
- **Après le gate**, le grep de l'ancienne règle (`grand livre distinct`, `relié à un autre compte du grand livre`) a trouvé
  trois restes : la phrase du CHANGELOG, le commentaire TSDoc de `AlreadyBooked` (`reconciliation.types.ts`) et un commentaire
  du test d'égalité. Corrigés au commit de clôture de la remédiation : **commentaires et documentation seulement**, aucune
  ligne exécutable ; contrôlés par `cargo fmt --check`, `cargo build -p kesh-api --tests` et `npm run check` (0 erreur)
  — `p1/build-commentaires.log`, `p1/check-commentaires.log`. Le gate complet et l'E2E de `e29a85ca` n'ont pas été rejoués :
  le dernier commit **de code exécutable** reste `e29a85ca`.

**Remédiation de la revue P2** (commit **`cf585bff`**, 2026-11-12 ; journaux sous `kesh-gate-logs/e344/p2/`, préfixés du
hash) :
- **Rouges** (`rouges.log`, entrée de la remédiation P2 ; `p2/rouges-vitest.log`) : 1 rouge Vitest pour la bonne raison
  (B2-5 : la modale s'ouvrait sur une ligne sortie de la liste). Les deux tests Rust neufs (E2-4, B2-1) sont des témoins.
- **Tests ciblés** (`p2/verts-cible.log`) : binaire `reconciliation_link_entry_e2e` et tests du module, **45/45**.
- **Mutations rejouées** au commit `cf585bff` (`mut-p2/`) : **M5** (3 rouges, dont le test de B2-5), **M9** (4 rouges),
  **M3** (`compte_relie_ailleurs`, `comptes_partages`), **M10** neuve (`rapprochee_sur_le_meme_compte`) — toutes tuées. Non
  rejouées : M1, M2, M4 (fonction de comptage et constante non touchées), M6 à M8 (`already-booked.ts` et le bouton
  *Rejeter* non touchés) — leurs journaux restent ceux de `mut-p1/`.
- **Gate complet au commit `cf585bff`**, arbre propre : base remise à neuf ; `test-fast --ci` **5149 passés, 13 ignorés** ;
  build du workspace vert ; `gate-ci.sh kesh_ci_e344` **5156 passés, 0 échoué, 17 ignorés** ; `lint-i18n-ownership` vert ;
  `npm run check` 0 erreur (29 avertissements préexistants) ; Vitest complet **2945 passés, 225 fichiers** ; `npm run build`
  vert ; **E2E complet** sur `kesh_e2e_e344` recréée vide : **388 passés, 7 échecs, 17 ignorés**, les sept de la KF-029
  (#97) ; le spec neuf passe. Port 3344 libre avant et après.
- **Captures** : l'écran français ne change pas (seul le texte allemand change) ; le script les a reprises dans
  `kesh-gate-logs/e344/captures-p2/` (non comparées une à une à `captures-p1/`).
- **Décomptes** (de `4152d337` à `cf585bff`) : `reconciliation_link_entry_e2e.rs` 40 → 42 attributs `sqlx::test` ; Vitest
  2944 → 2945 (+1, B2-5) ; trois textes allemands reformulés ; aucune clé ajoutée.

**Décomptes** (recomptés à la source, de `f12ad73e` à `HEAD`) : `reconciliation_link_entry_e2e.rs` 31 → 39 attributs
`sqlx::test` (+7 tests `s34_4_…`, +1 banc ignoré) ; `reconciliation_link.rs` 4 → 5 `#[test]` ; Vitest +19 tests neufs (14 dans
`ReconciliationProposals.test.ts`, 5 dans `already-booked-i18n.test.ts`) ; huit clés i18n neuves par locale, quatre locales.

**Captures** (thème clair, base E2E jetable, script `kesh-gate-logs/e344/scripts/captures-e344.mjs`), dans
`kesh-gate-logs/e344/captures/` : `e344-1-ligne-a-deja-comptabilisee.png` (cas a, ligne à règle), `e344-2-3-lignes-b-et-c.png`
(cas c puis cas b, compte d'épargne), `e344-4-confirmation-accepter.png` (confirmation ouverte sur *Accepter (1)*), et, prises
par l'E2E, `e344-ligne-signalee.png`, `e344-confirmation-ecriture-manuelle.png`. Le dossier contient aussi les captures que
d'autres specs écrivent pendant la suite complète (sans rapport).

**Manuel** : le `.tex` change (paragraphe neuf `\label{sec:deja-comptabilisee}`, encadré « Dépense déjà réglée », « Restent sans
écriture… », § *Un virement entre deux de vos comptes*) ; le PDF se régénère à l'intégration (C-e342-20), non régénéré ici.

**Écarts** :
- *Consigne Edit* : `cargo fmt --all` a reformaté les fichiers Rust (C-e344-10) ; les scripts de mutation remplacent un motif
  vérifié unique (permis pour les mutations) ; les scripts de `kesh-gate-logs/` ont été corrigés par de petits scripts Python
  (hors dépôt). Tout le code de fond est écrit par l'outil Edit ; l'outil ne manquait de rien.
- *Restauration des mutations* : au premier passage (M5), `git diff --name-only` a aussi remis à `HEAD` le registre
  `e34-choix-autonomes-e344.md`, non commité (C-e344-9, C-e344-10) ; texte repris du diff conservé au journal `mut/M5.log`,
  puis commité. Le script refuse désormais de muter un arbre non propre. (Même faute que celle déclarée par la 34-3.)
- *Verrous* : en cours de story, l'orchestrateur a ouvert une voie frontend (`.gate-front.lock`) et sorti les compilations
  seules des verrous ; le gate a été arrêté puis repris dans ces voies (`gates.log`, `gates-suite.log`).

**Limites voisines, non traitées (issues demandées, une par problème)** : voir le rapport final de l'agent.

### File List

- `crates/kesh-db/src/repositories/reconciliation_link.rs` — mode `MatchedElsewhere`, constantes SQL partagées,
  `find_matched_elsewhere`, `AlreadyBookedCounts`, `count_already_booked_for_page`, tests unitaires.
- `crates/kesh-api/src/routes/reconciliation.rs` — champ `alreadyBooked` de `get_proposals`.
- `crates/kesh-api/tests/reconciliation_link_entry_e2e.rs` — tests `s34_4_…` (AC-1, AC-2, AC-3, AC-8), banc ignoré.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` — huit clés `reconciliation-labels-already-booked-…`.
- `frontend/src/lib/features/reconciliation/already-booked.ts` (neuf), `AlreadyBookedConfirmDialog.svelte` (neuf),
  `ReconciliationProposals.svelte`, `reconciliation.types.ts`.
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`, `already-booked-i18n.test.ts` (neuf), fixtures
  de `LinkSettlementModal.test.ts`, `ManualMatchModal.test.ts`, `TransactionSplitModal.test.ts`.
- `frontend/src/lib/shared/i18n-keys.test.ts` — garde des sites littéraux 2924 → 2932.
- `frontend/tests/e2e/reconciliation-already-booked.spec.ts` (neuf).
- `docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e344.md` (neuf), `sprint-status.yaml`, cette fiche, le prompt de
  revue `34-4-transaction-deja-comptabilisee-review-prompt-p1.md`.
