# Story 27.4 : Un lot de réconciliation qui contient une transaction disparue est traité proposition par proposition

## Status

**in-progress** — fiche créée le 2026-10-12 en autonomie (`bmad-create-story`), sur `f9725c29` (merge de la
Story 27-1, #635), branche `story/27-4-lot-reconciliation-transaction-disparue`, worktree
`/home/gcorbaz/devel/kesh-27-4`. Choix consignés au registre `epic-27-choix-autonomes.md`, **C-27-4-1 à C-27-4-32**.
13 AC (AC1 à AC13). Validation **close** en trois passes (P1 Sonnet 4 MEDIUM → P2 Opus 5 MEDIUM → P3 Sonnet
0 MEDIUM), LOW de la P3 appliqués ; développement en cours.
PR : **`closes #634`**.

## Story

**As a** Comptable qui rapproche ses transactions bancaires,
**I want** qu'un lot d'acceptation ou de rejet qui contient une transaction disparue entre-temps — supprimée avec
son import, rapprochée ou rejetée ailleurs — traite les autres propositions et me dise, ligne par ligne, laquelle
a été refusée et pourquoi,
**so that** une page *Réconciliation* restée ouverte pendant la suppression d'un import ne fasse plus échouer tout
le lot sur un message qui accuse le mauvais compte bancaire.

Issue : [#634](https://github.com/guycorbaz/kesh/issues/634) (bug, **P2**), relevée en validation P2 de la Story 27-1
(lentille F, F2-1 ; lentille R, R2-5). Epic : `_bmad-output/planning-artifacts/epic-27-recette-v0-13-1.md`, section
« Amendement du 2026-10-12 » — la v0.13.2 ferme #624, #626 et #634 pour que le nombre de bugs baisse. Cible :
**v0.13.2**. La Story 27-1 (suppression d'un import non rapproché, `DELETE /api/v1/bank-imports/{id}`) est **mergée**
sur `main` : c'est elle qui rend le cas atteignable — avant elle, une transaction ne disparaissait jamais.

### Le défaut, au sol

`post_accept` (`crates/kesh-api/src/routes/reconciliation.rs:944-967`) et `post_reject` (`:3049-3064`) lisent les
transactions du lot **hors transaction et hors verrou** par `reconciliation_repo::find_pending_by_ids`
(`crates/kesh-db/src/repositories/reconciliation.rs:279-318`), qui ne rend que les lignes
`company_id = ? AND bank_account_id = ? AND status = 'pending' AND auto_match_rejected_at IS NULL`. Dès qu'un
identifiant manque, **tout le lot** est refusé :

```rust
return Err(AppError::Validation(format!(
    "bankTransactions [{:?}] n'appartiennent pas au bankAccountId={}",
    missing, body.bank_account_id
)));
```

— un `400 VALIDATION_ERROR` dont le message, non traduit, affirme que les transactions « n'appartiennent pas » au
compte, alors qu'elles ont pu être supprimées (#632), rapprochées ou rejetées ailleurs. Aucune proposition valide du
lot n'est traitée. L'écran (`ReconciliationProposals.svelte:230`, `:249`) affiche ce message brut dans
`reconciliation-error`.

Ce comportement contredit :

- le § *Pattern batch — FailedProposal per-proposal* du `CLAUDE.md` : une erreur **qui dépend de la proposition
  individuelle** (« FK manquant, race condition optimistic lock ») va dans `failed[]`, en `200` ;
- le manuel utilisateur, `docs/manual/fr/user-manual.tex:1925` : « **Un refus n'empêche pas les autres.** » ;
- et le manuel le **reconnaît** lui-même depuis la 27-1 (`user-manual.tex:1756-1759`) : « si elle coche encore une
  transaction supprimée, son lot est refusé **en entier** » — puce à réécrire.

Le préexistant est plus large que la suppression : une transaction **rapprochée** ou **rejetée** dans un autre
onglet entre le chargement de la liste et le clic produit le même `400` (tests
`post_reject_skips_reconciled_transactions`, `reconciliation_e2e.rs:1843`, et
`post_reject_after_accept_returns_already_reconciled_failed`, `:1968`, qui le figent en le documentant comme
« déviation comportement vs spec verbose »).

## Critères d'acceptation

**AC1 — Une seule classification par transaction, partagée par les deux lots et par la reprise sous verrou
(C-27-4-1, C-27-4-2, C-27-4-3, C-27-4-19).**

Une fonction **pure** (sans I/O) classe chaque identifiant du lot à partir de la ligne lue (ou de son absence) :

| La ligne lue pour `(company_id, bank_account_id, id)` | Résultat | `errorCode` | `details` |
|---|---|---|---|
| absente — supprimée, **d'une autre société**, ou **d'un autre compte bancaire** de la même société | refus | `BANK_TRANSACTION_NOT_FOUND` | `null` |
| **rapprochée** au sens de `kesh_db::repositories::bank_imports::is_reconciled(status, matched_entry_id)` — `status <> 'pending'` **ou** `matched_entry_id` non nul (quel que soit `auto_match_rejected_at`) | refus | `RECONCILIATION_ALREADY_RECONCILED` | `null` |
| non rapprochée et `auto_match_rejected_at` non nul | refus | **`RECONCILIATION_ALREADY_REJECTED`** (code neuf) | `null` |
| non rapprochée et `auto_match_rejected_at` nul | **éligible** | — | — |

- L'ordre des lignes est la **priorité** (une ligne rapprochée **et** marquée rejetée rend
  `RECONCILIATION_ALREADY_RECONCILED`).
- ⛔ **Anti-énumération** : les trois causes de la première ligne rendent **le même code, sans `details`**, par le
  **même chemin** (une seule requête scopée société **et** compte). Rien dans la réponse ne distingue une
  transaction supprimée d'une transaction d'une autre société ; le compte bancaire du corps reste contrôlé avant
  (`404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, inchangé — `errors.rs:2250`, asserté par `reconciliation_e2e.rs:2210`).
- ⛔ **Une seule définition de « rapprochée »** (C-27-4-19) : la fonction **appelle** `is_reconciled` de la Story 27-1
  (`bank_imports.rs:349`, avec `status.as_str()`) et n'écrit **aucun** second test de statut. Un statut ajouté plus
  tard est donc refusé (`status <> 'pending'` → rapprochée) : la classification **échoue fermée**, ce qui remplace la
  garde « `match` exhaustif » de la première rédaction. Le doc-comment d'`is_reconciled` (l. 346-347, « Écrite deux
  fois, et seulement deux ») est complété : toujours écrite deux fois, **appelée** par la suppression d'import et par
  la classification des lots de réconciliation.
- Chaque code est écrit **au site de construction**, sous la forme `error_code: "…".to_string()` — la forme que
  compte la méthode de l'AC7 (`grep -ohE 'error_code: "[A-Z_]+"'`). Une fonction qui rendrait un `&'static str`
  repris par `error_code: code.to_string()` cacherait `RECONCILIATION_ALREADY_REJECTED` à ce décompte (R2-8 de la
  validation P2).
- Tests unitaires de la fonction (module `#[cfg(test)]` de `reconciliation.rs`, à côté des tests existants l. 4445
  et suivantes) : les quatre lignes du tableau, plus « rapprochée et rejetée », plus **« `pending` avec
  `matched_entry_id` »** (→ `RECONCILIATION_ALREADY_RECONCILED`, état qu'aucun chemin vivant ne produit mais que la
  suppression d'import refuse aussi) ; codes écrits **en dur** dans les tests, jamais relus d'une constante de
  production.

**AC2 — `POST /api/v1/reconciliation/accept` : le pré-vol classe, il ne refuse plus le lot (C-27-4-5, C-27-4-6).**

- Le pré-vol lit les lignes du lot par une fonction de dépôt **sans filtre de statut** (AC11), les classe (AC1), et
  ne transmet au traitement sous verrou (`retry_with` → `accept_once` → `accept_batch`) que les propositions
  **éligibles**, dans l'ordre reçu.
- Réponse `200` : `accepted[]` inchangé ; `failed[]` réunit les refus du pré-vol et ceux du traitement sous verrou,
  **dans l'ordre des propositions reçues**.
- Le refus global `400` « bankTransactions […] n'appartiennent pas au bankAccountId » **disparaît** : le message ne
  subsiste nulle part (`grep -rnF "appartiennent pas au bankAccountId" crates frontend/src docs` vide, hors
  artefacts BMAD historiques).
- Restent **globaux**, inchangés : corps vide ou mal formé, identifiant non positif, doublon, `bankAccountId` non
  positif (`400`) ; compte bancaire inconnu, d'une autre société ou archivé (`404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
  — à ne pas confondre avec `BANK_ACCOUNT_NOT_FOUND`, code de `failed[]` des variantes, l. 1661, 2197, 2573) ; verrou
  du compte non obtenu (`409`) ; plus le plafond de l'AC5.
- Le champ `batch_size` de la trace `reconciliation.accepted` (`reconciliation.rs:2022`) garde son sens : la
  **taille du lot reçu** (`body.proposals.len()`), pas celle du sous-lot éligible — il faut donc la passer
  explicitement à `accept_batch`, qui la calcule aujourd'hui sur la tranche qu'il reçoit (`:1173`).
- Aucune trace d'audit pour un refus du pré-vol (comme pour tout refus de `failed[]` aujourd'hui).
- Le traitement sous verrou est **inchangé** : `accept_one_invoice` recharge la ligne
  (`find_pending_by_id_for_account`, `:1442-1472`) et rend `BANK_TRANSACTION_NOT_FOUND` /
  `RECONCILIATION_ALREADY_RECONCILED` ; `accept_one_split` et `accept_one_rule` rendent
  `RECONCILIATION_TRANSACTION_NOT_PENDING` (`:2300-2313`, `:2701-2714`) — non harmonisés (C-27-4-11).
- Tests propres à l'AC2 (en plus de l'AC6) : **(a) ordre** — un lot de trois propositions **non trié par
  identifiant**, `[absente (id haut), éligible refusée sous verrou (ex. facture non validée), absente (id bas)]` →
  `failed[]` dans cet ordre exact ; **(b) `batch_size`** — un lot de deux dont une absente : la trace
  `reconciliation.accepted` de l'autre porte `batch_size = 2`.

**AC3 — Un lot sans aucune proposition éligible ne prend ni transaction ni verrou (C-27-4-4).**

Si le pré-vol ne laisse **aucune** proposition éligible, `post_accept` et `post_reject` répondent `200`
(`{ accepted: [], failed: [...] }` / `{ rejected: [], failed: [...] }`) **sans** ouvrir de transaction ni appeler
`with_account_lock`. Test, **pour chacune des deux routes** (deux tests, ou un test qui appelle `/accept` puis
`/reject` sous le même verrou tenu) : le verrou nommé du compte est tenu par une connexion du test (montage de
`reconciliation_e2e.rs:1486-1511`, nom `reconcile:{base}:{company}:{account}`) ; un lot dont toutes les transactions
sont absentes répond `200` — sous la mutation qui retire le court-circuit **d'une seule** des deux routes (M7a,
M7b), celle-ci attendrait `LOCK_TIMEOUT_SECS` = 5 (l. 65) et rendrait `409` : l'assertion du statut suffit, aucune
borne de temps n'est posée (elle n'ajouterait rien et risquerait un faux rouge sur un poste chargé — F3-4 de la
validation P3).

**AC4 — `POST /api/v1/reconciliation/reject` : même classification ; reprise exacte d'une course sous verrou
(C-27-4-5, C-27-4-7).**

- Le pré-vol classe comme l'AC2 ; `reject_batch` ne reçoit plus que les identifiants **éligibles**, triés par
  identifiant croissant — l'ordre de verrouillage de la Story 27-1 (AC4-bis, `:3079-3086`) est **préservé** ; il ne
  reçoit plus le `tx_map` du pré-vol, dont il n'a plus l'usage (ses deux branches `BANK_TRANSACTION_NOT_FOUND` et
  `RECONCILIATION_ALREADY_RECONCILED`, `:3195-3213`, deviennent du code mort et **sont retirées**).
- Réponse `200` : `failed[]` réunit les refus du pré-vol et ceux du traitement sous verrou, **triés par identifiant
  croissant** — le contrat publié par la 27-1 (« `rejected[]` et `failed[]` sortent triés », `api-external.md:548`)
  vaut désormais pour tout le `failed[]`.
- L'`UPDATE` du rejet (`:3216-3224`) garde ses conditions et **ajoute** `AND bank_account_id = ? AND
  matched_entry_id IS NULL AND auto_match_rejected_at IS NULL` :
  - `auto_match_rejected_at IS NULL` : un rejet concurrent de la même transaction ne la ré-horodate plus en silence
    (la trace d'audit la comptait deux fois) — M10 ;
  - `bank_account_id = ?` (C-27-4-27) : la **restauration** d'une installation vide `bank_transactions` puis la
    recharge **avec ses identifiants** (`backup.rs`, `restore_tables_in_tx`), et elle n'est exclusive que des autres
    imports (`bank_imports.rs`, angle mort « restauration ») : un identifiant vu par le pré-vol sur le compte X peut,
    au moment de l'`UPDATE`, désigner une ligne du compte Y. La clause empêche de rejeter la transaction d'un autre
    compte — M17, **tuée** par l'AC6-5 (b) ;
  - `matched_entry_id IS NULL` (C-27-4-30) : la reprise sous verrou dit « rapprochée » comme le pré-vol, au sens
    d'`is_reconciled` (AC1) — M19, équivalente (état `pending` avec lien qu'aucun chemin vivant ne produit).

  `reject_batch` reçoit donc `bank_account_id` (sa signature, l. 3183-3190, ne le porte pas aujourd'hui) à la place
  du `tx_map`.
- Si l'`UPDATE` ne touche aucune ligne, la ligne est **relue par une lecture verrouillante** — fonction de dépôt
  neuve `find_by_id_for_account_for_update(tx, company_id, bank_account_id, id) -> Option<BankTransaction>`
  (`crates/kesh-db/src/repositories/reconciliation.rs`, testée à l'AC11 ; C-27-4-21), `SELECT … WHERE id = ? AND
  company_id = ? AND bank_account_id = ? FOR UPDATE` — lecture *courante*, jamais la vue cohérente de la
  transaction, qui pourrait dater d'avant la suppression (prouvé par l'AC6-8) — et **reclassée par la fonction de
  l'AC1** : supprimée → `BANK_TRANSACTION_NOT_FOUND`, rapprochée → `RECONCILIATION_ALREADY_RECONCILED`, rejetée →
  `RECONCILIATION_ALREADY_REJECTED`. Si la relecture la trouve **éligible** (état que l'`UPDATE` aurait dû
  toucher — non atteignable sous la lecture courante, M18 équivalente), `tracing::warn!` et repli sur le refus
  d'aujourd'hui (`RECONCILIATION_ALREADY_RECONCILED`, `details.reason = "race_during_update"`).
- La trace `reconciliation.rejected` est inchangée (identifiants **rejetés** seulement, `entity_id` = le plus
  petit).

**AC5 — Un lot de plus de 500 identifiants est refusé en `400`, avant toute lecture (C-27-4-9).**

Aujourd'hui, `find_pending_by_ids` rend `Err(DbError::Sqlx(Protocol))` au-delà de `MAX_IN_CLAUSE_IDS` = 500
(`repositories/reconciliation.rs:263`, `:293-298`), ce qui sort en **`500`**. Les deux routes refusent désormais un
lot de plus de 500 identifiants en `400 VALIDATION_ERROR` à l'étape 0 (validation du corps, avant toute requête —
même famille que « proposals vide »). La borne est **une seule constante** : `MAX_IN_CLAUSE_IDS` devient `pub` dans
`kesh-db`, **sous son nom** (C-27-4-20) — elle borne aussi `find_contacts_by_ids` (`repositories/reconciliation.rs:214`,
lecture des contacts de `GET /proposals`), qu'un renommage propre aux lots tromperait — et la route la lit : pas de
second littéral `500`. Tests : 500 identifiants passent le contrôle (aucun `400` de taille), 501 rendent `400` ; les
deux routes.

**AC6 — Le cas de l'issue, de bout en bout : un import supprimé entre la liste et le lot (tests d'intégration
Rust).**

Dans `crates/kesh-api/tests/reconciliation_e2e.rs` (aides `setup_company`, `seed_bank_transactions`, `spawn_app`
du fichier ; route `DELETE /api/v1/bank-imports/{id}` appelée avec le JWT du Comptable) :

1. **Suppression avant le lot** — deux imports A et B du même compte, une transaction chacun (A : `tA`, B : `tB`),
   chacune rapprochable d'une facture validée. `DELETE` de A (`204`), puis `POST /accept` avec `[tA, tB]` →
   `200`, `accepted` = `[tB]` (écriture créée, facture réglée), `failed` = `[{ tA, BANK_TRANSACTION_NOT_FOUND,
   details: null }]`. Même montage pour `POST /reject` → `rejected` = `[tB]`, `failed` = `[tA …]`.
2. **Suppression pendant le lot** (après le pré-vol, avant le verrou) — le test tient le verrou nommé du compte ;
   le lot part en tâche et se bloque dans `GET_LOCK` (attente observée par
   `kesh_db::test_fixtures::attendre_une_requete_en_cours`, `test_fixtures.rs:561`, motif `GET_LOCK`) ; le test
   supprime A (`DELETE`, `204`), puis relâche le verrou. **Acceptation** : `tA` sort en
   `BANK_TRANSACTION_NOT_FOUND` (recharge sous verrou, `:1442-1456`), `tB` est acceptée. **Rejet** : `tA` sort en
   `BANK_TRANSACTION_NOT_FOUND` par la reprise de l'AC4 (et non plus `RECONCILIATION_ALREADY_RECONCILED`), `tB` est
   rejetée.
3. **Autre compte, autre société** — `post_accept_returns_400_on_cross_account_proposal` (`:2222`) est **réécrit**
   (et renommé) : transaction d'un autre compte de la même société → `200`, `failed` =
   `[{ id, BANK_TRANSACTION_NOT_FOUND, details: null }]`, rien d'écrit (aucune écriture, facture non réglée). Un
   test jumeau avec une transaction d'une **autre société** rend **exactement** le même corps de réponse (comparé
   en JSON, identifiant mis à part) — l'assertion d'anti-énumération de l'AC1.
4. **Lot mixte des quatre classes** — un lot `reject` portant une éligible, une rapprochée, une rejetée et un
   identifiant inexistant, **envoyé dans un ordre non croissant** (ex. `[inexistant (id le plus haut), rejetée,
   éligible, rapprochée (id le plus bas)]`) → `rejected` = l'éligible ; `failed` = les trois autres **dans l'ordre
   croissant**, codes du tableau de l'AC1, et **`details: null` sur chacun** — c'est ce qui tue M3 au niveau de la
   route : sous M3, la rapprochée passe le pré-vol et ressort par le repli `race_during_update`, au même code mais
   avec `details` non nul (R2-7 de la validation P2).
5. **Rejet : la ligne change sous le lot sans disparaître** — deux cas, même montage (celui du 2. : le test tient le
   verrou nommé, attend `GET_LOCK`, agit par `UPDATE` direct, relâche), lot `reject [tA, tB]`.

   **(a) Rejet concurrent de la même transaction** (AC4, garde `auto_match_rejected_at IS NULL` ; C-27-4-16) — au
   lieu de supprimer, le test pose sur `tA`, par `UPDATE` direct (qui n'écrit
   aucune trace), une **valeur sentinelle** `auto_match_rejected_at = '2026-01-01 00:00:00.000'`, relit sa
   `version` (`v0`), puis relâche le verrou. Assertions : (a) `failed` = `[{ tA, RECONCILIATION_ALREADY_REJECTED,
   details: null }]` et `rejected` = `[tB]` — **sans `tA`** ; (b) en base, `tA` garde `auto_match_rejected_at` =
   la sentinelle et `version = v0` ; (c) **aucune** trace `reconciliation.rejected` ne liste `tA`, et l'unique trace
   du lot porte `details.bank_transaction_ids = [tB]`. Sous M10 (garde retirée), `tA` entre dans `rejected[]`, son
   horodatage et sa `version` changent, et la trace la cite : (a), (b) et (c) rougissent.

   **(b) La transaction passe sur un autre compte** (AC4, garde `bank_account_id = ?` ; C-27-4-27, tue M17) — le test
   crée un **second compte bancaire de la même société** ; pendant l'attente, il exécute `UPDATE bank_transactions
   SET bank_account_id = <second compte> WHERE id = tA` — ce que produit, au même identifiant, une restauration
   concurrente. Attendus : `failed` = `[{ tA, BANK_TRANSACTION_NOT_FOUND, details: null }]`, `rejected` = `[tB]` ; en
   base, `tA` garde `auto_match_rejected_at` nul ; aucune trace `reconciliation.rejected` ne la cite. Sous M17,
   l'`UPDATE` du lot touche la ligne du second compte : `tA` entre dans `rejected[]`.
6. **Acceptation d'une transaction rejetée** — `POST /accept` sur une transaction rejetée → `200`,
   `failed[0].errorCode = "RECONCILIATION_ALREADY_REJECTED"` ; le manuel continue de dire vrai (« l'acceptation
   d'une proposition la refuse », `user-manual.tex:1920` — cf. AC9 pour la course étroite assumée).
7. **Rejet : refus du pré-vol et refus sous verrou mêlés** (C-27-4-17, C-27-4-29, tue M9 dans ses deux ordres) —
   trois imports A, B, C du même compte, créés dans cet ordre, une transaction chacun : `tBas` (A) `< tMil` (B)
   `< tHaut` (C). `tBas` et `tHaut` sont rejetées **avant** le lot (`UPDATE` direct). Lot `reject` envoyé
   **`[tHaut, tMil, tBas]`** (non trié), montage du 2. : le test tient le verrou, attend `GET_LOCK`, supprime B
   (`204`), relâche. → `rejected` = `[]`, `failed` = `[{ tBas, RECONCILIATION_ALREADY_REJECTED }, { tMil,
   BANK_TRANSACTION_NOT_FOUND }, { tHaut, RECONCILIATION_ALREADY_REJECTED }]` **dans cet ordre** : le seul refus
   sous verrou (`tMil`) est **entre** les deux refus du pré-vol. Une concaténation « pré-vol puis verrou », chacun
   trié, rendrait `[tBas, tHaut, tMil]` ; « verrou puis pré-vol », `[tMil, tBas, tHaut]` : les deux rougissent.
8. **Rejet : la relecture de l'AC4 est une lecture courante** (C-27-4-15, tue M11) — deux transactions `tB < tA` du
   **même** import, éligibles. Le test ouvre une transaction SQL sur sa propre connexion et y exécute `DELETE FROM
   bank_transactions WHERE id = tA` **sans la valider** (verrou X tenu sur `tA`). Le lot `reject [tA, tB]` part en
   tâche : le pré-vol (lecture non verrouillante) voit encore `tA` éligible ; sous verrou, `tB` est rejetée — sa
   relecture `SELECT auto_match_rejected_at` **ouvre la vue cohérente** de la transaction du lot, qui voit `tA` —,
   puis l'`UPDATE` de `tA` se bloque sur le verrou X. Le test observe ce blocage **durablement** (C-27-4-26) :
   `attendre_une_requete_en_cours(&["UPDATE bank_transactions", "auto_match_rejected_at"])`, puis une pause de
   **500 ms**, puis le même appel une seconde fois — l'aide existante, appelée deux fois, sans mécanisme neuf. Le
   motif désigne **aussi** l'`UPDATE` de `tB`, de même texte (PROCESSLIST montre l'instruction, non les valeurs
   liées) : le premier appel peut le surprendre en vol, avant que la vue cohérente soit ouverte ; 500 ms plus tard,
   le seul `UPDATE` encore en cours est celui de `tA`, bloqué. Puis le test **valide** la suppression. L'`UPDATE` de `tA` touche 0 ligne ; la relecture `FOR UPDATE` la voit **absente** →
   `failed` = `[{ tA, BANK_TRANSACTION_NOT_FOUND, details: null }]`, `rejected` = `[tB]`. Sous M11 (relecture
   ordinaire), la vue cohérente montrerait `tA` éligible → repli `RECONCILIATION_ALREADY_RECONCILED` +
   `race_during_update` : le test rougit. ⚠️ Le montage de l'AC6-2 ne tue **pas** M11 : la suppression y est validée
   avant la première lecture ordinaire du lot, si bien que même une relecture ordinaire la voit (F1 de la
   validation P1).

   ⚠️ **Angle mort assumé, à écrire au doc-comment du test** (comme A2-3 de la 27-1, `reconciliation_e2e.rs:1739-1747`) :
   la sonde lit une requête **en cours**, non une requête **en attente de verrou** — le seul trait propre à
   l'attente (`information_schema.INNODB_TRX`, `trx_state = 'LOCK WAIT'`) exige le privilège `PROCESS`, que
   l'utilisateur de **dev** n'a pas (`kesh`, droits par base, `scripts/mariadb-init/01-dev-grants.sql` ;
   `test_fixtures.rs:547-550`) — la CI, elle, tourne en `root` ; le test doit passer sur le poste de dev, où il
   tourne d'abord. Sous M11, la mutation ne survit que si l'`UPDATE`
   de `tB`, par clé primaire, dure **plus de 500 ms**. Le code juste ne peut rendre aucun faux rouge : quel que soit
   l'`UPDATE` observé, la suppression est validée avant ou pendant le blocage de `tA`, et la lecture courante la
   voit. Si M11 survit à la campagne, rejouer le test avant de la déclarer survivante.

Chaque test vérifie aussi l'**état en base** (ligne non modifiée, aucune écriture, aucune trace d'audit
superflue), pas seulement le corps de la réponse.

**AC7 — Les libellés de l'écran disent ce qui s'est passé (C-27-4-8).**

Dans `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` :

- **`RECONCILIATION_ALREADY_REJECTED`** (neuf) → clé neuve `reconciliation-failed-already-rejected`, FR :
  « Cette transaction a déjà été rejetée : elle n’est plus proposée. »
- **`BANK_TRANSACTION_NOT_FOUND`** → même clé (`reconciliation-failed-bank-transaction-not-found`), **texte
  récrit** dans les quatre locales, FR : « Transaction introuvable : elle a été supprimée avec son import, ou
  n’appartient pas à ce compte bancaire. »
- **`RECONCILIATION_ALREADY_RECONCILED` avec `details.reason = "race_during_update"`** → clé neuve
  `reconciliation-failed-changed-during-batch`, FR : « La transaction a été modifiée pendant le traitement —
  rapprochée, rejetée ou supprimée par une autre opération. » Les autres formes (`details` nul ou autre raison)
  gardent `reconciliation-errors-already-reconciled`. *(Ce code, avec `details.reason = "race_during_update"`, reste
  émis par trois sites de l'acceptation — `:1934-1940`, `:2435-2440`, `:2887-2892` — que la story ne modifie pas
  côté serveur, et par le repli non atteignable du rejet (AC4) ; le libellé cesse d'affirmer « déjà réconciliée »
  quand la cause peut être une suppression. Le contrat publié le dit aussi (AC10). Pour une proposition **règle**
  ou **ventilée**, la cause la plus fréquente d'un refus sous verrou n'est d'ailleurs pas ce code mais
  `RECONCILIATION_TRANSACTION_NOT_PENDING` (« n’est plus en attente », libellé inchangé, C-27-4-11) : c'est pourquoi
  le manuel décrit la conduite, non la liste des motifs (AC9, C-27-4-28).)*
- Les quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` portent les deux clés
  neuves et le texte récrit (FR l. 1187, autres l. 1124) ; les replis en dur du `.ts` sont **identiques au FR**.
  Proposition DE / IT / EN dans les Dev Notes, à confronter à `docs/i18n-glossaire.md`.
- Le doc-comment du module (l. 1-50) recompte : **27 littéraux** (`grep -ohE 'error_code: "[A-Z_]+"'
  crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`, 26 aujourd'hui) **+ 2** codes de
  `DbError::error_code()` = **29 codes** ; et nomme le cinquième code qui lit ses `details`
  (`RECONCILIATION_ALREADY_RECONCILED`, pour `race_during_update`) — la phrase `failed-proposal-label.ts:25`,
  « **Quatre codes** seulement lisent leur `details` », devient « **Cinq codes** » (grep de la valeur :
  `grep -nF "Quatre codes" frontend/src/lib/features/reconciliation/`) ; le doc-comment de `failedProposalLabel`
  (`failed-proposal-label.ts:105-107`), qui **énumère** les quatre codes sans écrire « Quatre » — ce grep ne le voit
  pas —, nomme le cinquième.
- Filets déjà en place, qui tournent dans `npm run test:unit` et doivent rester verts : `i18n-repli-divergent-actif.test.ts`
  et `i18n-un-repli-par-cle.test.ts` (replis identiques au FR, un repli par clé).
- `failed-proposal-label.test.ts` : `EXPECTED_KEYS` gagne `RECONCILIATION_ALREADY_REJECTED`, le décompte passe de
  28 à **29** (l. 67), avec le **titre** du test (l. 66, « couvre les 28 codes relevés ») et le commentaire
  d'en-tête (l. 1-8) — grep de la valeur : `grep -nwE "2[68]" frontend/src/lib/features/reconciliation/failed-proposal-label*.ts` ; un test pour `race_during_update` (clé neuve
  lue), un pour une autre raison (clé générique), un pour `details` absent.
- `frontend/src/lib/shared/i18n-keys.test.ts:580` : `sitesTotal` **2041 → 2043** sur la base `f9725c29` (deux appels
  `i18nMsg(` ajoutés ; recompter, ne pas ajuster — la valeur finale dépend des branches mergées avant, cf.
  § *Fichiers communs*) ; `npm run lint-i18n-ownership` vert.

**AC8 — L'écran : rien à restructurer, un comportement à prouver (C-27-4-12, C-27-4-14).**

`ReconciliationProposals.svelte` affiche déjà `failed[]` ligne par ligne (`:432-449`), avec la transaction relevée
**avant** le lot (`snapshotSelected`, `:67-73`) et recharge la liste après (`:229`, `:248`). Aucun changement de
structure n'est attendu ; si le développement en trouve un nécessaire, il le consigne (choix `C-27-4-k`).

- **Vitest** (`ReconciliationProposals.test.ts`) : un lot d'acceptation à deux lignes dont une sort en
  `BANK_TRANSACTION_NOT_FOUND` → compteur `1`, l'*Échec partiel* désigne la transaction supprimée par sa **date et
  son montant relevés** (pas `TX #…`) avec le libellé récrit, `reconciliation-error` **absent** ; le test existant
  `:455-478` (« TX #99 », texte asserté l. 476) est mis à jour au texte récrit.
- **E2E Playwright** (spec neuf `frontend/tests/e2e/reconciliation-deleted-import.spec.ts`, ou ajout motivé à un
  spec existant) : deux imports CAMT par l'API (relevé réécrit à chaque exécution — montants et références uniques,
  patron de `reconciliation-cancel.spec.ts:86-110`), une **règle d'affectation** créée par l'API qui rend les deux
  transactions sélectionnables — `POST /api/v1/reconciliation/rules`, corps `CreateRuleRequest`
  (`crates/kesh-api/src/routes/reconciliation_rules.rs:99`, `matchType`, `matchValue`, `counterpartyAccountId`, …) ;
  la correspondance se lit dans `kesh-reconciliation/src/rules.rs:46` (`rule_matches`, `CounterpartyContains` sur
  `counterparty_name`) ; `reconciliation-rules.spec.ts` crée sa règle **par le formulaire** et n'est pas un patron
  d'appel d'API (R3-6 de la validation P3) — sa valeur de correspondance est **unique
  à l'exécution** (contrepartie réécrite dans le relevé, comme les montants), pour qu'elle ne rende sélectionnable
  aucune transaction d'un autre spec, et la règle comme l'import B sont **supprimés en `finally`** (C-27-4-24) ; la page *Réconciliation* est ouverte,
  les deux lignes cochées ; **puis** l'import A est supprimé par `DELETE /api/v1/bank-imports/{id}`
  (`authedApiContext`) ; clic *Rejeter* → `reconciliation-success` montre `1`, un `reconciliation-failed-item`
  porte `data-error-code="BANK_TRANSACTION_NOT_FOUND"` et `data-tx-id` = la transaction de A, son texte contient le
  libellé récrit ; `reconciliation-error` absent. ⚠️ C'est le seul test qui voit le `200` traverser la frontière HTTP
  jusqu'à l'écran (`CLAUDE.md`, § *E2E*).

**AC9 — Le manuel utilisateur dit vrai, PDF compris (C-27-4-13).**

`docs/manual/fr/user-manual.tex` :

- **l. 1756-1759** (§ *Supprimer un import*, puce « Une page *Réconciliation* ouverte avant la suppression ») :
  réécrite **par la conduite, non par une liste de libellés** (C-27-4-28) — la transaction supprimée est refusée et
  apparaît dans les *Échecs partiels* avec son motif ; les autres propositions du lot sont traitées ; la liste,
  rechargée après le lot (`ReconciliationProposals.svelte:229`, `:248`), ne la propose plus — il n'y a rien à
  refaire. Aucun libellé n'est cité : selon l'instant de la suppression et le type de la proposition, le motif est
  « introuvable », « n’est plus en attente » ou « modifiée pendant le traitement » (AC7, AC10), et une liste close
  serait fausse pour l'un d'eux. Plus de « refusé **en entier** », plus de consigne de recharger avant de recommencer.
- **l. 1925** (§ *Un refus n'empêche pas les autres*) : la liste des motifs gagne « transaction supprimée avec son
  import, rapprochée ou rejetée entre-temps (dans un autre onglet, par exemple) ».
- l. 1920 (« l'acceptation d'une proposition la refuse ») : **reste vrai pour une liste périmée** (le pré-vol refuse
  la transaction rejetée) ; la course étroite — un rejet validé pendant que le lot d'acceptation attend le verrou du
  compte, que la recharge sous verrou d'`accept_one_invoice` accepte encore — est un **choix de conception** déjà
  écrit (« je rétracte », doc-comment de `find_pending_by_id_for_account`) et n'est **pas** décrite au manuel
  (C-27-4-23). Ne pas toucher la phrase.
- PDF régénéré (`make user` dans `docs/manual/`, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`) et
  **contrôlé aplati et normalisé** — `pdftotext` rend les ligatures (« aﬀiche », U+FB00 à U+FB04) et l'apostrophe
  typographique (« n’est ») :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/’/'/g; s/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g; s/ﬃ/ffi/g; s/ﬄ/ffl/g"`.
  Attendus **chiffrés** : « refusé en entier » passe de **2 à 1** occurrence (reste la phrase homonyme de l'import
  CSV, l. 1786, légitime) ; les phrases neuves sont présentes (témoins choisis sans ligature ni apostrophe :
  « les autres propositions du lot sont traitées » pour la l. 1756, « rapprochée ou rejetée » pour la l. 1925 —
  rédiger les phrases pour qu'elles les contiennent mot pour mot ; le témoin évite « entre-temps », dont
  `pdftotext` retire le tiret quand LaTeX coupe la ligne sur « entre- », R3-1 de la validation P3).

**AC10 — L'API publique et le CHANGELOG (C-27-4-10).**

- `docs/api-external.md`, après le paragraphe du succès partiel (l. 538) (C-27-4-18) : une transaction
  **introuvable** (supprimée, d'un autre compte ou d'une autre société), **rapprochée** ou **rejetée** est refusée
  dans `failed[]` au lieu du `400 VALIDATION_ERROR` du lot entier, *(depuis la v0.13.2)*, pour `accept` comme pour
  `reject` — avec, **au pré-vol**, `BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`,
  `RECONCILIATION_ALREADY_REJECTED`, `details` nul. Le texte écrit aussi les codes qu'une transaction **changée
  pendant le traitement** rend, parce qu'ils ne sont pas les mêmes : sous verrou, une proposition `split` ou `rule`
  rend `RECONCILIATION_TRANSACTION_NOT_PENDING` (inchangé, C-27-4-11), et `RECONCILIATION_ALREADY_RECONCILED` peut
  porter **`details.reason = "race_during_update"`** quand la transaction est rapprochée, rejetée ou supprimée entre
  la relecture et l'écriture (acceptation ; repli du rejet). **Tous ces refus signifient la même conduite** : relire
  `GET /api/v1/reconciliation/proposals`. Puis : le plafond de 500 identifiants (`400`, auparavant `500`), en
  précisant que le **corps** reste borné par la limite par défaut d'Axum (2 Mo) — un lot `accept` de propositions
  ventilées, jusqu'à `MAX_SPLIT_LINES` = 50 lignes chacune, peut être refusé avant 500 propositions en
  `400 VALIDATION_ERROR` « requête invalide (corps non-parsable) » (`reconciliation.rs:542-550`) (F2-7) ; le
  paragraphe du rejet (l. 548) dit que `failed[]` est trié en entier. ⚠️ **Changement de contrat** à écrire comme
  tel : un client qui traitait le `400` reçoit désormais un `200` à lire ligne par ligne, et un client qui teste
  `details == null` sur `RECONCILIATION_ALREADY_RECONCILED` se trompe.
- `CHANGELOG.md`, section **`## [0.13.2] — Non publié`** (existante, l. 11) : sous un **`### Corrigé`** — à créer
  entre *Modifié* et le séparateur `---` (l. 21) **s'il n'existe pas encore à la fusion** : les 27-2 et 27-3 en
  créent chacune un, et la section ne doit en porter qu'un : le lot de réconciliation qui contient une transaction
  disparue n'est plus refusé en entier ([#634](https://github.com/guycorbaz/kesh/issues/634)) ; sous *Modifié* — le contrat
  API (`200` + `failed[]`, code neuf, `details.reason = "race_during_update"` possible sur
  `RECONCILIATION_ALREADY_RECONCILED`, plafond `400`). Lignes **ajoutées**, sans réécrire celles de la 27-1 (ni
  celles des 27-2 / 27-3 si elles ont été mergées avant : fusion par union).

**AC11 — Le dépôt : une lecture par identifiants sans filtre de statut (C-27-4-10).**

- `find_pending_by_ids` (`repositories/reconciliation.rs:265-318`), qui n'a que les deux appelants de cette story
  (`grep -rn "find_pending_by_ids" crates` : `reconciliation.rs:950`, `:3049`, plus des commentaires de tests), est
  **remplacée** par une fonction de même forme **sans filtre de statut ni de rejet** — nom proposé
  `find_by_ids_for_account` — scopée `company_id` **et** `bank_account_id`, même plafond, même `HashMap`.
  Doc-comment : qui l'appelle, pourquoi aucun filtre (la classification est faite par l'appelant, AC1).
- Test de dépôt (`crates/kesh-db/tests/reconciliation_repository.rs`) : rend une ligne `pending`, une `reconciled`,
  une rejetée ; n'en rend aucune d'un autre compte ni d'une autre société ; liste vide → vide ; 501 → erreur.
- **Relecture verrouillante de l'AC4** (C-27-4-21) : `find_by_id_for_account_for_update` dans le même fichier de
  dépôt (aucune fonction n'y fait aujourd'hui de `SELECT … FOR UPDATE` par identifiant), sur la transaction passée
  par l'appelant, même scope `company_id` **et** `bank_account_id`, sans filtre de statut. C'est la requête de
  `find_pending_by_id_for_account` (`:337-356`, même scope, aucun filtre de statut malgré son nom — « Naming
  caveat ») **plus** `FOR UPDATE` : le texte SQL est **partagé** — une **fonction privée** qui construit le texte
  (`format!` sur `BANK_TX_COLUMNS`), appelée par les deux lectures, la verrouillante y ajoutant ` FOR UPDATE` ; une
  constante n'est pas réalisable sans `const_format`, absente du workspace (F3-3 / R3-2 de la validation P3) —, pas
  recopié, pour que les deux lectures ne divergent pas (C-27-4-32, R2-9). Doc-comment : lecture
  **courante**, pourquoi (vue cohérente d'un lot déjà ouvert), seul appelant `reject_batch`. Test de dépôt : rend la
  ligne `pending`, `reconciled`, rejetée ; `None` pour un identifiant absent, d'un autre compte, d'une autre société.
  Le caractère **courant** de la lecture est prouvé côté route par l'AC6-8, pas ici.
- ⚠️ `kesh-db` touché : **gate complet même en cours de boucle** (`CLAUDE.md`, § *Pendant une boucle de revue*,
  exception `kesh-db`).

**AC12 — Les textes du dépôt qui décrivaient le défaut sont mis à jour.**

- `crates/kesh-db/src/repositories/bank_imports.rs:424-427` (angle mort « lot préparé avant la suppression … le lot
  entier rend `400 VALIDATION_ERROR` … Non corrigé ici (#634) ») : réécrit au comportement neuf.
- `crates/kesh-db/src/repositories/bank_imports.rs:346-347`, doc-comment d'`is_reconciled` : complété du second
  appelant (AC1, C-27-4-19) — la phrase « Écrite deux fois, et seulement deux » reste vraie. Son jumeau côté écran,
  `frontend/src/lib/features/bank-import/bank-import-deletion.ts:12-14` (« `is_reconciled`, appelée par
  `delete_unreconciled_in_tx` »), nomme lui aussi le second appelant.
- `crates/kesh-db/src/repositories/reconciliation.rs:18` (doc du module) et `:322` (doc de
  `find_pending_by_id_for_account`), qui citent `find_pending_by_ids` : au nom neuf.
- `crates/kesh-api/tests/reconciliation_e2e.rs:1734-1737` (« `failed[]` n'est pas asserté : le pré-vol refuse en
  `400` … ») : corrigé ; le test `post_reject_locks_the_batch_by_increasing_id` reste vert tel quel.
- Deux commentaires de test qui décrivent l'ancien `400` sans aucun des jetons du grep ci-dessous (F3-1 de la
  validation P3) : `reconciliation_e2e.rs:1195-1196` (« filtre `status='pending'` et exclurait la tx → 400 ») et
  `:4325-4326` (« contrôle préalable (400, vérifié au développement) ») — réécrits au comportement neuf ; et
  `:2106` (doc de `post_accept_filters_signed_amount`, « `find_pending_by_ids` ne filtre pas par sign », R3-8) — au
  nom neuf.
- `post_reject_skips_reconciled_transactions` (`:1843`, assertion `:1892`) et
  `post_reject_after_accept_returns_already_reconciled_failed` (`:1968`, assertion `:2035`) : passent à `200`,
  `failed[0].errorCode = "RECONCILIATION_ALREADY_RECONCILED"`, la ligne inchangée en base ; leurs commentaires de
  « déviation » sont retirés.
- Doc-comments de `post_accept` (étape 0ter, `:944`, et le commentaire « fail-fast 400 », `:969-973`), de
  `post_reject`, de `accept_batch` et du module (l. 1-16) : au comportement neuf.
- Recherche de propagation, **valeurs** et non formulations : `grep -rnE "appartiennent pas|find_pending_by_ids|fail-fast 400|en entier" crates frontend/src docs`,
  plus `grep -nE "→ 400|\(400|batch-level 400" crates/kesh-api/tests/reconciliation_e2e.rs`
  — chaque site résiduel trié (légitime ou corrigé) et nommé au Change Log.

**AC13 — Mutations : chacune fait rougir au moins un test nommé.**

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | le pré-vol d'`accept` rétablit le `400` global sur absence | AC6-1 (accept), AC6-3 |
| M2 | idem pour `reject` | AC6-1 (reject), AC6-4 |
| M3 | une ligne `reconciled` classée éligible | AC1 unitaire ; AC6-4 (assertion `details: null` : sous M3, la rapprochée ressort par le repli `race_during_update`) |
| M4 | une ligne rejetée classée éligible | AC1 unitaire ; AC6-6 |
| M5 | priorité inversée (rejetée avant rapprochée) | AC1 unitaire « rapprochée et rejetée » |
| M6 | `details` d'un `BANK_TRANSACTION_NOT_FOUND` porte `bankAccountId` | assertion `details: null` d'AC6-1 et d'AC6-3 — **pas** la comparaison des deux corps de l'AC6-3, qui resterait verte si la valeur venait du corps de la requête |
| M7a | court-circuit de l'AC3 retiré de `post_accept` seul | AC3, route `/accept` (`200` sous verrou tenu, au lieu de `409`) |
| M7b | court-circuit de l'AC3 retiré de `post_reject` seul | AC3, route `/reject` |
| M8 | `failed[]` d'`accept` non remis dans l'ordre reçu (refus du pré-vol ajoutés en tête ou en fin) | AC2 (a) |
| M9 | `failed[]` de `reject` non trié en entier (refus du pré-vol et refus sous verrou concaténés, chacun trié, **dans l'un ou l'autre ordre**) | **AC6-7** (refus sous verrou d'id milieu entre deux refus du pré-vol : les deux ordres de concaténation rougissent, C-27-4-29) ; AC6-4 (lot envoyé non trié) pour la variante « aucun tri » |
| M10 | `AND auto_match_rejected_at IS NULL` retiré de l'`UPDATE` du rejet | AC6-5 (a) (`tA` absente de `rejected[]`, sentinelle et `version` inchangées, aucune trace ne la cite) |
| M11 | relecture de l'AC4 non verrouillante (`FOR UPDATE` retiré) | **AC6-8** — l'AC6-2 ne la tue pas (C-27-4-15) ; observation durable, angle mort écrit (survit seulement si l'`UPDATE` de `tB` dure plus de 500 ms ; rejouer avant de conclure, C-27-4-26) |
| M12 | `batch_size` = taille du sous-lot éligible | AC2 (b) |
| M13 | plafond de l'AC5 relevé à 501 | test AC5 (501 → `400`) |
| M14 | libellé `race_during_update` non lu (clé générique) | Vitest AC7 |
| M15 | `RECONCILIATION_ALREADY_REJECTED` absent de `failedProposalLabel` | Vitest AC7 (`EXPECTED_KEYS`) |
| M16 | classification sur `status` seul (`is_reconciled` non appelé, `matched_entry_id` ignoré) | AC1 unitaire « `pending` avec `matched_entry_id` » |
| M17 | `AND bank_account_id = ?` retiré de l'`UPDATE` du rejet | **AC6-5 (b)** (`tA` passée sur un autre compte pendant l'attente du verrou : sous M17, elle entre dans `rejected[]`) — l'équivalence déclarée en P1 reposait sur un fait faux : la restauration réinsère les mêmes identifiants (C-27-4-27) |
| M18 | branche « relue éligible » (warn + repli `race_during_update`) remplacée par un autre refus | **équivalente, déclarée d'emblée** (C-27-4-22) : non atteignable sous la lecture courante (l'`UPDATE` à 0 ligne et la relecture `FOR UPDATE` lisent la même version validée de la ligne, sous le même verrou) ; elle ne le devient que sous M11 |
| M19 | `AND matched_entry_id IS NULL` retiré de l'`UPDATE` du rejet | **équivalente, déclarée d'emblée** (C-27-4-30) : la clause ne refuse que l'état `pending` avec lien, qu'aucun chemin vivant ne produit (AC1 ; les seuls `UPDATE` qui le posent sont dans des tests de `kesh-db`) et que le pré-vol refuse déjà |

Toute mutation restée verte est soit corrigée par un test, soit déclarée **équivalente** avec sa raison au Change
Log ; M18 et M19 le sont d'emblée. ⚠️ Après chaque mutation restaurée, **toucher** le fichier (`touch`) : cargo garde
sinon le binaire muté.

## Tasks / Subtasks

- [ ] **T0 — Montage** (AC —) : base `kesh_274` et `kesh_e2e_274` (migrations du worktree, seed
  `scripts/seed-dev-db.sql`), port `3034`, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-4/target`,
  `node_modules` par `cp -al` depuis le dépôt principal. Gates lourds sous `flock
  /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [ ] **T1 — Dépôt** (AC11, AC5) : `find_by_ids_for_account` à la place de `find_pending_by_ids`, constante de
  plafond `pub` sous son nom (C-27-4-20), `find_by_id_for_account_for_update` (AC4, C-27-4-21) ; tests de dépôt.
- [ ] **T2 — Classification** (AC1) : fonction pure appelant `is_reconciled` (C-27-4-19) + tests unitaires ;
  doc-comment d'`is_reconciled` complété.
- [ ] **T3 — `post_accept`** (AC2, AC3, AC5) : pré-vol classant, sous-lot éligible, `batch_size` explicite,
  fusion de `failed[]` dans l'ordre reçu, court-circuit, plafond — **avec leurs tests** : AC2 (a) ordre, AC2 (b)
  `batch_size`, AC3 route `/accept` (verrou tenu), AC5 route `/accept` (500 passe, 501 → `400`).
- [ ] **T4 — `post_reject`** (AC4, AC3, AC5) : pré-vol classant, `reject_batch` sans `tx_map` et avec
  `bank_account_id`, garde de l'`UPDATE`, relecture verrouillante (fonction de dépôt de T1) et reclassement,
  `failed[]` trié en entier — **avec leurs tests** : AC3 route `/reject`, AC5 route `/reject`.
- [ ] **T5 — Tests d'intégration** (AC6, AC12) : huit scénarios (AC6-1 à AC6-8, le 5. en deux cas (a) et (b)),
  trois tests existants réécrits,
  commentaire `:1734-1737`.
- [ ] **T6 — Frontend** (AC7, AC8) : libellés, quatre catalogues, `sitesTotal`, doc-comment « Cinq codes »,
  Vitest (filets `i18n-repli-divergent-actif` et `i18n-un-repli-par-cle` verts), E2E à valeur de règle unique et
  nettoyage en `finally`.
- [ ] **T7 — Documentation** (AC9, AC10, AC12) : manuel + PDF aplati et normalisé (« refusé en entier » 2 → 1),
  `api-external.md`, CHANGELOG, doc-comments ; grep de propagation.
- [ ] **T8 — Mutations et gates** (AC13) : campagne M1-M19 (M7a et M7b ; M18 et M19 déclarées équivalentes ; M11 rejouée
  avant d'être déclarée survivante) ; gate complet (`scripts/test-fast.sh`), Vitest
  complet, `npm run check`, `npm run lint-i18n-ownership`, `npm run build`, **E2E complet au dernier commit de
  code** ; la base remise à zéro avant le gate complet.

## Dev Notes

### Inventaire des lots de réconciliation et de leurs pré-vols, au sol

Commande : `grep -n "^pub async fn\|^async fn" crates/kesh-api/src/routes/reconciliation.rs` et
`grep -rln "failed: Vec<" crates/kesh-api/src/`.

| Route | Forme | Lecture de la transaction | Refus d'une transaction disparue aujourd'hui | Après la story |
|---|---|---|---|---|
| `POST /reconciliation/accept` (`post_accept`, l. 839) | **lot** `{accepted, failed}` | pré-vol `find_pending_by_ids` hors verrou (l. 950) ; recharge sous verrou par variante | **`400` global** au pré-vol ; sous verrou : `BANK_TRANSACTION_NOT_FOUND` / `RECONCILIATION_ALREADY_RECONCILED` (facture), `RECONCILIATION_TRANSACTION_NOT_PENDING` (ventilé, règle) | `failed[]` (AC2) |
| `POST /reconciliation/reject` (`post_reject`, l. 3010) | **lot** `{rejected, failed}` | pré-vol `find_pending_by_ids` hors verrou (l. 3049) ; **aucune** recharge sous verrou (`tx_map` du pré-vol, l. 3195) | **`400` global** au pré-vol ; sous verrou : `RECONCILIATION_ALREADY_RECONCILED` + `race_during_update` (l. 3226-3232), trompeur | `failed[]` (AC4), reprise exacte |
| `POST /reconciliation/manual` (l. 3364) | une transaction | `find_strictly_pending_by_id_for_account` avant et sous verrou | `404 RECONCILIATION_TRANSACTION_NOT_PENDING` ; `409` sur `UPDATE` (l. 3663) | **inchangé** — route unitaire, hors § *Pattern batch* |
| `POST /reconciliation/split` (l. 3776) | une transaction | idem (l. 3938, 4052) | idem (l. 4114) | **inchangé** |
| `GET /reconciliation/transactions/{id}`, `POST …/{id}/cancel` | une transaction | — | `404` | **inchangé** |
| `GET /reconciliation/proposals` | lecture | — | — | **inchangé** |
| Règles (`reconciliation_rules.rs`) | CRUD unitaire | ne lisent pas `bank_transactions` | — | hors sujet |

Autres lots du dépôt (`failed: Vec<` : `payment_batches.rs:121`, `invoice_email.rs:1073`, `inbox_import.rs:75`) :
aucun ne lit `bank_transactions` (`grep -n "bank_transaction" <fichier>` vide) ; leurs refus globaux relevés
(`invoice_email.rs:418`, `:488`) portent sur le corps (niveau de rappel), conformes au § *Pattern batch*. **Hors
périmètre**, non modifiés.

### Inventaire des sites NON RÉSOLUS après la story (ensemble clos)

Méthode : tout site de `reconciliation.rs` qui **rend un refus lié à l'état de la transaction** —
`grep -nE 'BANK_TRANSACTION_NOT_FOUND|RECONCILIATION_ALREADY_RECONCILED|RECONCILIATION_TRANSACTION_NOT_PENDING|race_during_update|find_pending_by_ids|find_pending_by_id_for_account|find_strictly_pending'`
— est soit résolu, soit écrit ici comme angle mort assumé.

| Site | Ce qui reste | Pourquoi assumé |
|---|---|---|
| `accept_one_invoice`, `UPDATE bank_transactions` à 0 ligne (`:1934-1940`) | `RECONCILIATION_ALREADY_RECONCILED` + `race_during_update` pour une suppression **entre** la recharge sous verrou et l'`UPDATE` | fenêtre de quelques millisecondes ; libellé honnête à l'écran (AC7) ; une reprise par relecture, comme au rejet, toucherait le flux d'acceptation de facture (lettrage, audit) — hors du gain de l'issue (C-27-4-8) |
| `accept_one_split`, idem (`:2435-2440`) | idem | idem |
| `accept_one_rule`, idem (`:2887-2892`) | idem | idem |
| `accept_one_split` / `accept_one_rule`, recharge sous verrou (`:2300-2313`, `:2701-2714`) | `RECONCILIATION_TRANSACTION_NOT_PENDING` réunit introuvable, rapprochée, autre compte, autre société | déjà par proposition, libellé existant juste (« n’est plus en attente ») ; harmoniser avec la variante facture changerait un code publié sans gain (C-27-4-11) |
| `accept_one_invoice`, recharge sous verrou sans filtre de rejet (`:1442-1472`) ; ventilé et règle, qui remettent `auto_match_rejected_at` à `NULL` | une transaction rejetée **entre** le pré-vol et le verrou est encore acceptée | préexistant, voulu par le doc-comment de `find_pending_by_id_for_account` (« j'ai rejeté trop vite, je rétracte ») ; le pré-vol couvre le cas courant (liste périmée) |
| Vue cohérente d'`accept_batch` : la recharge de la 2ᵉ proposition et des suivantes lit la vue ouverte par la 1ʳᵉ | une suppression validée pendant le lot n'est vue qu'à l'`UPDATE` (0 ligne → ligne 1 de ce tableau) | conséquence de REPEATABLE READ ; même issue, même libellé |
| `/manual`, `/split` (routes unitaires) | `404` / `409` | hors § *Pattern batch* : une requête, une transaction |
| `accept_one_invoice`, étape 4 sous verrou sur `status` seul (`:1467-1468`) ; `accept_one_split` et `accept_one_rule`, recharge par `find_strictly_pending_by_id_for_account` (`:2300`, `:2701`), qui filtre `status = 'pending'` sans regarder `matched_entry_id` | une transaction `pending` **avec** `matched_entry_id` passerait la recharge sous verrou (le pré-vol, lui, la refuse désormais par `is_reconciled`, AC1) | état qu'aucun chemin vivant ne produit (inventaire clos des écrivains, doc-comment d'`is_reconciled`) ; le pré-vol suffit à le refuser pour toute liste. Le site jumeau du **rejet** est, lui, résolu : son `UPDATE` porte `matched_entry_id IS NULL` (AC4, C-27-4-30) |
| Restauration d'une installation, ou remise à zéro de l'installation (`backup.rs:111-122`, `reset_cleared_tables`, appelée par `kesh-seed/src/lib.rs:584`), concurrente d'un lot (toutes routes) | la restauration vide et recharge `bank_transactions` **avec ses identifiants**, sans prendre le verrou nommé de réconciliation : un identifiant du lot peut désigner, sous verrou, une autre ligne ; la remise à zéro vide sans réinsérer, d'effet identique à une suppression d'import (déjà traité par la story) | préexistant, même angle mort que la suppression d'import (`bank_imports.rs`, « La restauration n'étant exclusive que des autres imports ») ; l'`UPDATE` du rejet est désormais scopé par compte (AC4, M17 tuée par l'AC6-5 (b)) ; les **trois `UPDATE` d'acceptation** (`:1919`, `:2417`, `:2869`) ne sont scopés que par `company_id` et `version` (leurs recharges sous verrou, lectures ordinaires, le sont par compte mais ne bloquent pas derrière une restauration) : non gardés (F3-5 de la validation P3) |
| `reject_batch`, repli « relue éligible » (AC4) | `RECONCILIATION_ALREADY_RECONCILED` + `race_during_update` | non atteignable sous la lecture courante (M18, équivalente) ; garde défensive, tracée par `warn!` |

**Triage de chaque angle mort (C-27-4-25)** — tous relèvent de la **catégorie C** (choix de conception), aucun d'un
défaut hors périmètre ; **aucune issue n'est ouverte**, et la fiche le dit pour que l'absence ne passe pas pour un
oubli :

- les trois `race_during_update` d'acceptation et la vue cohérente d'`accept_batch` : refus par proposition, rien
  d'écrit ; le code est documenté au contrat (AC10) comme « changée pendant le traitement — relire » et le libellé
  le dit (AC7). Ce n'est plus un code trompeur, c'est un code **générique** ;
- `RECONCILIATION_TRANSACTION_NOT_PENDING` des variantes ventilée et règle : code publié et juste, conduite écrite
  au contrat (AC10, C-27-4-11) ;
- la recharge sous verrou d'`accept_one_invoice` sans filtre de rejet : voulue (« je rétracte »), sans effet
  comptable — accepter est un geste légitime de l'utilisateur, le rejet n'est qu'un « ne plus proposer » ;
- `pending` avec lien sous verrou, repli éligible du rejet : états non atteignables ;
- restauration concurrente d'un lot : angle mort préexistant de la restauration, déjà écrit par la 27-1 ; la story
  ne l'aggrave pas et en ferme la conséquence la plus grave (rejeter la transaction d'un autre compte) ;
- `/manual`, `/split` : hors § *Pattern batch*.

Résolus par la story : pré-vol d'`accept` (`:944-967`), pré-vol de `reject` (`:3049-3064`), branches mortes de
`reject_batch` (`:3195-3213`), `UPDATE` du rejet à 0 ligne (`:3226-3232`), plafond `> 500` en `500`.

### État actuel des fichiers modifiés — ce qui doit être préservé

- **`post_accept` / `accept_once` / `accept_batch`** : rejeu `retry_with` sur 1213 **et**
  `ReconciliationTransactionAborted` (1305, point de sauvegarde perdu) — Story 25-4-c2, prédicat local, registre
  `RETRY_WITH_AUTORISE` ; **un seul verrou** pour le lot (H5) ; un point de sauvegarde par proposition. Le pré-vol
  reste **hors** de `retry_with` (calculé une fois) ; le sous-lot éligible est cloné dans la fermeture comme
  aujourd'hui (`body.proposals.clone()`, `:1008`).
- **`post_reject`** : tri croissant des identifiants avant le verrou (27-1, AC4-bis — contre le cycle avec
  `bank_imports::delete_unreconciled_in_tx`, qui verrouille l'entête puis les lignes par identifiant croissant) ;
  **pas** de rejeu (une 1213 rend `500`). La lecture verrouillante de l'AC4 prend un verrou X sur une ligne **déjà
  visée** par l'`UPDATE` qui vient d'échouer — même ordre, même ligne : pas de cycle neuf. Sur une ligne
  **absente**, `FOR UPDATE` pose un verrou d'intervalle, comme l'`UPDATE` à 0 ligne qui le précède déjà : rien de
  neuf (cf. l'angle mort « supremum » de `bank_imports.rs:408-411`).
- **`reject_batch`** : `rejected_at` relu en base après l'`UPDATE` (M4 Pass 1, horloge MariaDB) ; trace unique,
  `entity_id` = plus petit identifiant rejeté.
- **`ReconciliationProposals.svelte`** : relevé des transactions **avant** le lot (`snapshotSelected`), bilan
  hors des branches de la liste (C38 de la 15-5c), `clearBatchReport` à chaque nouveau bilan.

### Pièges connus du dépôt, applicables ici

- **Vue cohérente** (REPEATABLE READ) : une `SELECT` ordinaire sous la transaction du lot lit l'instantané ouvert à
  la première lecture ; pour savoir si une ligne a disparu **maintenant**, il faut une lecture **courante**
  (`FOR UPDATE`). C'est la règle de la 15-8a, citée par `delete_unreconciled_in_tx` (`bank_imports.rs:383-385`).
- **`innodb_snapshot_isolation`** (F2-6 de la validation P2) : la reprise de l'AC4 et le montage de l'AC6-8 reposent
  sur une lecture verrouillante (l'`UPDATE` de `tA`, puis le `FOR UPDATE`) qui lit une version validée
  **postérieure** à la vue déjà ouverte par la relecture de `tB`. C'est le comportement de **MariaDB 10.11**
  (`docker-compose.yml`, CI). Avec `innodb_snapshot_isolation` (actif par défaut depuis **11.6**), cette lecture
  rend `ER_CHECKREAD` (1020), que le `?` de `reject_batch` ferait sortir en `500` pour **tout le lot** — contraire
  au § *Pattern batch*. À revoir à la montée de version, comme le dit déjà `bank_imports.rs:413-418` pour la 27-1 ;
  l'AC6-8 est le test qui la signalera.
- **Motif `GET_LOCK` sans précédent** (R3-9 de la validation P3) : aucun test du dépôt n'observe encore un lot
  bloqué dans `with_account_lock` par `attendre_une_requete_en_cours(&["GET_LOCK"])` (`mutex.rs:90`, `SELECT
  GET_LOCK(?, ?)`). Vérifier le motif sur une exécution avant de bâtir l'AC3, l'AC6-2, l'AC6-5 et l'AC6-7 dessus ; à
  défaut, le panic de dix secondes de l'aide le signale au premier lancement.
- **Verrou nommé global au serveur** (KF-038, #228) : son nom porte la base courante — reconstruire
  `reconcile:{DATABASE()}:{company}:{account}` dans les tests (`reconciliation_e2e.rs:1492-1503`).
- **Test muet** : un test d'ordre qui ne contient que des identifiants déjà triés ne prouve rien (M8, M9) — envoyer
  un lot **non trié** (AC2 (a), AC6-4, AC6-7) ; un test de concurrence dont la suppression précède la première
  lecture ordinaire ne prouve pas la lecture courante (M11 : AC6-8, et non AC6-2).
- **Recompter, pas ajuster** : `sitesTotal`, le nombre de codes (29), le nombre de littéraux (27) — commandes
  copiées au Change Log.
- **Base piégée** : la remettre à zéro avant tout gate complet (`CLAUDE.md`, § *Un gate laisse la base piégée*) —
  ⚠️ sur ce poste, **ne pas** redémarrer `kesh-mariadb-dev` (tmpfs partagé par les autres worktrees) : recréer
  `kesh_274` / `kesh_e2e_274` seulement.

### Textes proposés (DE / IT / EN), à confronter au glossaire

| Clé | DE | IT | EN |
|---|---|---|---|
| `reconciliation-failed-bank-transaction-not-found` | Transaktion nicht gefunden: Sie wurde mit ihrem Import gelöscht oder gehört nicht zu diesem Bankkonto. | Transazione introvabile: è stata eliminata con la sua importazione o non appartiene a questo conto bancario. | Transaction not found: it was deleted with its import, or does not belong to this bank account. |
| `reconciliation-failed-already-rejected` | Diese Transaktion wurde bereits abgelehnt: Sie wird nicht mehr vorgeschlagen. | Questa transazione è già stata rifiutata: non viene più proposta. | This transaction has already been rejected: it is no longer proposed. |
| `reconciliation-failed-changed-during-batch` | Die Transaktion wurde während der Verarbeitung geändert – durch einen anderen Vorgang abgeglichen, abgelehnt oder gelöscht. | La transazione è stata modificata durante l’elaborazione — riconciliata, rifiutata o eliminata da un’altra operazione. | The transaction changed during processing — reconciled, rejected or deleted by another operation. |

### Découpage préventif — non requis

Modules touchés : `kesh-db/repositories/reconciliation`, `kesh-api/routes/reconciliation`,
`frontend/src/lib/features/reconciliation`, `kesh-i18n/locales`, documentation — **cinq au plus**, sous le seuil
de la règle de *splitting préventif* (C-27-4-13).

### Fichiers communs avec les branches non mergées 27-2 et 27-3 (lecture seule)

| Fichier | 27-2 | 27-3 | 27-4 |
|---|---|---|---|
| `CHANGELOG.md` (`[0.13.2]`) | oui (crée un `### Corrigé`) | oui (crée un `### Corrigé`) | oui — union, **un seul** `### Corrigé` (AC10) |
| `docs/manual/fr/user-manual.tex` + `.pdf` | oui | oui | oui — PDF à **régénérer** après fusion, jamais fusionné en binaire |
| `docs/api-external.md` | — | oui | oui |
| `crates/kesh-i18n/locales/*/messages.ftl` (×4) | oui | oui | oui |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | oui, **+2** (2041 → 2043 sur `f9725c29`) | oui, **+30** (2032 → 2062 sur `a2743fd4`, non rebasée sur la 27-1) | oui, **+2** (2041 → 2043 sur `f9725c29`) — conflit **possible**, non certain : deux branches écrivent la même ligne `2043` et Git peut fusionner sans conflit sur une valeur fausse. La valeur finale se **recalcule à chaque merge** (recompter, ne pas additionner de confiance) ; le Vitest, à égalité stricte, est le garde-fou |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | oui (un bloc à partir de la l. 2439 de `f9725c29`, dans `post_accept_emits_dual_audit_invoice_paid`) | — | oui (autres régions) |
| `crates/kesh-api/src/routes/reconciliation.rs` | oui (hunk de 3 lignes, commentaire de `get_proposals` vers la l. 626) | — | oui (`post_accept`, `accept_batch`, `post_reject`, `reject_batch`, tests unitaires) — régions disjointes |
| `_bmad-output/implementation-artifacts/epic-27-choix-autonomes.md` | oui | non (registre propre, `epic-27-choix-autonomes-27-3.md`) | oui — **union** (consignes des agents, 7) |
| `docs/i18n-glossaire.md` | oui | — | non (lecture seule, confrontation des traductions) |
| `crates/kesh-api/tests/textes_coherents.rs` | oui | oui | non |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | oui | oui | oui |

Relevé à la validation P2 (R2-5) sur les branches **actuelles** : `git diff origin/main...story/27-2-devise-import-csv
--stat` (base `f9725c29`, la 27-2 ayant été rebasée sur la 27-1) et `git diff origin/main...story/27-3-profil-apres-apercu
--stat` (base `a2743fd4`), plus `git diff … -- frontend/src/lib/shared/i18n-keys.test.ts | grep -E "^[-+]\s*sitesTotal"`
et `git diff … -- CHANGELOG.md | grep '^+###'`. À refaire à chaque rebase d'une de ces branches.
`crates/kesh-api/src/routes/bank_imports.rs` et `errors.rs`, touchés par 27-2 et 27-3, ne le sont **pas** ici. La
27-2 a relevé que `reconciliation.rs` ne contrôle pas la devise sur certains chemins (C-27-2-20) : **hors
périmètre**, non touché par la 27-4 — mais la 27-2 **modifie** ce fichier (commentaire ci-dessus) : fusion à refaire
sans conflit textuel attendu.

### Project Structure Notes

- Fonction de classification : privée à `crates/kesh-api/src/routes/reconciliation.rs` (elle construit des
  `FailedProposal`, type de la route) ; pas de nouveau module.
- Dépôt : `crates/kesh-db/src/repositories/reconciliation.rs` (`find_by_ids_for_account`,
  `find_by_id_for_account_for_update`, `MAX_IN_CLAUSE_IDS` rendue `pub`) ; `crates/kesh-db/src/repositories/bank_imports.rs`
  (doc-comment d'`is_reconciled` seulement).
- Aucune migration, aucune route neuve, aucun changement de rôle : un diff qui en porte une signale une dérive.

### References

- [Source: CLAUDE.md#Pattern batch — FailedProposal per-proposal]
- [Source: crates/kesh-api/src/routes/reconciliation.rs:839-1046, :1165-1225, :1430-1472, :3010-3290]
- [Source: crates/kesh-db/src/repositories/reconciliation.rs:259-357]
- [Source: crates/kesh-db/src/repositories/bank_imports.rs:365-440]
- [Source: frontend/src/lib/features/reconciliation/failed-proposal-label.ts, ReconciliationProposals.svelte]
- [Source: docs/manual/fr/user-manual.tex:1722-1760, :1906-1925]
- [Source: docs/api-external.md:536-548]
- [Source: _bmad-output/implementation-artifacts/27-1-supprimer-import-non-rapproche.md — AC3 (b), AC8, C-27-1-17]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-12 — fiche créée (`bmad-create-story`, autonomie, Opus), sur `f9725c29` ; choix C-27-4-1 à C-27-4-14.
- 2026-10-12 — **validation P1** (Sonnet, lentilles R et F, prompt `27-4-validate-prompt-p1.md`) : F = 0 CRITICAL,
  0 HIGH, 1 MEDIUM, 6 LOW ; R = 0 CRITICAL, 0 HIGH, 3 MEDIUM, 11 LOW ; doublons F2 = R1-5, F3 ⊂ R1-9 (a), F5 = R1-6,
  F1 ≈ R1-9 (d), F4 ⊂ R1-1. Remédiation (Opus), choix **C-27-4-15 à C-27-4-25** :
  F1 / R1-9 (d) → AC6-8, montage qui tue M11 (C-27-4-15) ; R1-1 / F4 → oracle d'AC6-5 à sentinelle (C-27-4-16) ;
  R1-2 → AC6-7, M9 rattachée, AC6-4 non trié (C-27-4-17) ; R1-3 / F6 → AC10 (`race_during_update`,
  `…_NOT_PENDING`, conduite unique) (C-27-4-18) ; F2 / R1-5 → `is_reconciled` réutilisée, M16 (C-27-4-19) ; F5 /
  R1-6 → nom gardé (C-27-4-20) ; R1-8 → fonction de dépôt `find_by_id_for_account_for_update` (C-27-4-21) ; R1-9
  → M7a/M7b, M6 rattachée à `details: null`, M17/M18 équivalentes (C-27-4-22) ; F7 / R1-10 → AC9 (C-27-4-23) ;
  R1-12 → E2E (C-27-4-24) ; R1-13 → triage catégorie C, aucune issue (C-27-4-25) ; R1-4, R1-7, R1-11, R1-14 sans
  choix (corrections de faits). Après remédiation : 13 AC, AC6 à huit scénarios, mutations M1-M18 (19 lignes, M7
  scindée), T0-T8, 25 choix. Prompt de la P2 : `27-4-validate-prompt-p2.md` (Opus).
- 2026-10-12 — **validation P2** (Opus, lentilles R et F, prompt `27-4-validate-prompt-p2.md`) : F = 0 CRITICAL,
  0 HIGH, 4 MEDIUM, 4 LOW ; R = 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW. Convergences : R2-1 = F2-5 (observateur de
  l'AC6-8, MEDIUM chez R, LOW chez F), R2-2 = F2-3 (M17), R2-3 = F2-1 + F2-8 (manuel, AC9). Soit **5 MEDIUM
  distincts** : R2-1/F2-5, R2-2/F2-3, R2-3/F2-1, F2-2, F2-4. Remédiation (Opus), choix **C-27-4-26 à C-27-4-32** :
  - R2-1 / F2-5 → AC6-8 : l'aide existante appelée deux fois à 500 ms d'écart, angle mort écrit comme A2-3 de la
    27-1, M11 rejouée avant d'être déclarée survivante (C-27-4-26) ;
  - R2-2 / F2-3 → raison de la garde `bank_account_id` réécrite (restauration concurrente, mêmes identifiants) ;
    M17 **tuée** par l'AC6-5 (b) ; ligne « restauration » aux sites non résolus (C-27-4-27) ;
  - R2-3 / F2-1 / F2-8 → AC9 : la puce du manuel décrit la conduite, aucun libellé ; témoin « les autres
    propositions du lot sont traitées » ; demi-phrase sur `…_NOT_PENDING` à l'AC7 (C-27-4-28) ;
  - F2-2 → AC6-7 **remplacé** par un cas à trois transactions qui tue M9 dans ses deux ordres (C-27-4-29) ;
  - F2-4 → `AND matched_entry_id IS NULL` dans l'`UPDATE` du rejet, M19 équivalente, site jumeau marqué résolu
    (C-27-4-30) ;
  - LOW : F2-6 (réserve `innodb_snapshot_isolation`, MariaDB ≥ 11.6, aux Pièges connus) ; F2-7 (limite de corps
    d'Axum à l'AC10) ; R2-4 (résidus nommés : `failed-proposal-label.ts:105-107`, titre du test l. 66,
    `bank-import-deletion.ts:12-14`, doc du dépôt l. 18 et l. 322, clause « `match` exhaustif » de C-27-4-1 marquée
    remplacée au registre) ; R2-5 (table des fichiers communs recalculée sur les branches actuelles : 27-2 sur
    `f9725c29`, `sitesTotal` +2 ; 27-3 sur `a2743fd4`, +30 ; conflit possible et non certain, Vitest garde-fou ; un
    seul `### Corrigé` au CHANGELOG ; registre en union) ; R2-6 (l. 3183-3190, 1467-1468, 450-476) ; R2-7
    (`details: null` à l'AC6-4, qui tue M3 au niveau route) ; R2-8 (codes écrits `error_code: "…"` au site, AC1) ;
    R2-9 (texte SQL partagé, C-27-4-32) ; R2-10 (sites split et règle ajoutés à la ligne « `pending` avec lien »).
  - **Constat D5** (§ *Règle de splitting préventif*, amendement du 2026-10-06) : P1 = 4 MEDIUM → P2 = 5 MEDIUM
    distincts, dont **4 nés de la remédiation P1** (montages ajoutés à l'AC6-7 et à l'AC6-8, équivalence de M17,
    puce du manuel) ; seul F2-4 est préexistant, rendu visible par C-27-4-19. Le signal est levé, et c'est un
    **recyclage** au sens du patch, non des défauts d'origine neufs. Décision de l'orchestrateur, dans l'autonomie
    déléguée par Guy le 2026-10-12 et sur le précédent de la 27-3 (« ne pas découper ») : **pas de découpage**,
    remédiation par **simplification** — aucune aide de test neuve, un cas remplacé plutôt qu'ajouté, une clause SQL,
    la conduite plutôt qu'une liste au manuel (C-27-4-31). Signal déclaré au Project Lead par le rapport de la
    remédiation.
  - Symptômes grepés après patch (valeurs) : `pas de confusion`, `M1-M18`, `M17 et M18`, `aucun chemin ne change`,
    `1464`, `3182-3189`, `450-474`, `2032 → 2034`, `conflit certain`, `deux motifs` — résidus restants : l'entrée P1 du
    Change Log, et au registre C-27-4-22 et C-27-4-23, historiques et marquées révisées dans leur titre.
  - Recompte après remédiation : 13 AC (`grep -cE '^\*\*AC[0-9]+ —'`) ; AC6 à huit scénarios, le 5. en deux cas ;
    mutations M1-M19, **20 lignes** (`grep -cE '^\| M[0-9]+[ab]? \|'`), M18 et M19 équivalentes d'emblée ; 9 tâches
    T0-T8 ; **32 choix** C-27-4 au registre (`grep -c '^### C-27-4-'`). Prompt de la P3 :
    `27-4-validate-prompt-p3.md` (Sonnet).
- 2026-10-12 — **validation P3** (Sonnet, lentilles R et F, prompt `27-4-validate-prompt-p3.md`) : F = 0 CRITICAL,
  0 HIGH, **0 MEDIUM**, 6 LOW ; R = 0 CRITICAL, 0 HIGH, **0 MEDIUM**, 10 LOW. Aucun recyclage au-dessus de LOW (R3-4,
  seul recyclage relevé, est un numéro de ligne). **La validation est close** (critère d'arrêt : uniquement des LOW).
  LOW appliqués (agent de dev, Opus) : F3-1 + R3-8 (trois commentaires de test ajoutés à l'AC12, grep élargi à
  `→ 400|\(400|batch-level 400`) ; F3-2 (`PROCESS` absent **en dev** seulement, CI en `root`, AC6-8 et C-27-4-26) ;
  F3-3 = R3-2 (texte SQL partagé par une **fonction** privée, AC11 et C-27-4-32) ; F3-4 (borne « moins de 2 s » de
  l'AC3 retirée : l'assertion `200` contre `409` suffit) ; F3-5 + R3-10 (ligne « restauration » : remise à zéro
  d'installation ajoutée, trois `UPDATE` d'acceptation non scopés par compte) ; F3-6 (`:1892`, `:1739-1747`, titre de
  C-27-4-7 marqué complété par C-27-4-30) ; R3-1 (témoin PDF « rapprochée ou rejetée », sans « entre-temps ») ; R3-3
  (`bank_imports.rs:413-418`) ; R3-4 (`:455-478`) ; R3-5 (T3 et T4 nomment les tests de l'AC2, de l'AC3 et de
  l'AC5) ; R3-6 (règle E2E par `POST /api/v1/reconciliation/rules`, `CreateRuleRequest`, route vérifiée
  `crates/kesh-api/src/lib.rs:695`) ; R3-7 (`sprint-status.yaml` à jour) ; R3-9 (motif `GET_LOCK` à vérifier sur
  une exécution, Pièges connus).
  - **Trend** : P1 4 MEDIUM (Sonnet) → P2 5 MEDIUM (Opus) → P3 **0 MEDIUM** (Sonnet). Modèles : Sonnet, Opus, Sonnet
    (passes complètes) ; remédiations Opus.
  - **D5** : signal levé en P2 (sévérité égale ou supérieure, recyclage au sens du patch) ; décision de
    l'orchestrateur : pas de découpage, simplification (C-27-4-31). La P3 confirme : aucun MEDIUM, aucun recyclage.
  - Symptômes grepés après patch (valeurs) : `2 s`, `< 2`, `1894`, `1738`, `une constante`, `400-405`, `450-476`,
    `rejetée entre-temps`, `utilisateur des tests` — résidus : l'entrée P2 de ce Change Log (historique) et la
    phrase du manuel prescrite à l'AC9 (« rapprochée ou rejetée entre-temps », qui contient le témoin).
  - Statut : `in-progress`.
