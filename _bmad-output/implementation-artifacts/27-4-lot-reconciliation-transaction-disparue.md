# Story 27.4 : Un lot de réconciliation qui contient une transaction disparue est traité proposition par proposition

## Status

**ready-for-dev** — fiche créée le 2026-10-12 en autonomie (`bmad-create-story`), sur `f9725c29` (merge de la
Story 27-1, #635), branche `story/27-4-lot-reconciliation-transaction-disparue`, worktree
`/home/gcorbaz/devel/kesh-27-4`. Choix consignés au registre `epic-27-choix-autonomes.md`, **C-27-4-1 à C-27-4-14**.
13 AC (AC1 à AC13). Aucune validation lancée ; prompt de la P1 : `27-4-validate-prompt-p1.md`. PR : **`closes #634`**.

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
(C-27-4-1, C-27-4-2, C-27-4-3).**

Une fonction **pure** (sans I/O) classe chaque identifiant du lot à partir de la ligne lue (ou de son absence) :

| La ligne lue pour `(company_id, bank_account_id, id)` | Résultat | `errorCode` | `details` |
|---|---|---|---|
| absente — supprimée, **d'une autre société**, ou **d'un autre compte bancaire** de la même société | refus | `BANK_TRANSACTION_NOT_FOUND` | `null` |
| `status = 'reconciled'` (quel que soit `auto_match_rejected_at`) | refus | `RECONCILIATION_ALREADY_RECONCILED` | `null` |
| `status = 'pending'` et `auto_match_rejected_at` non nul | refus | **`RECONCILIATION_ALREADY_REJECTED`** (code neuf) | `null` |
| `status = 'pending'` et `auto_match_rejected_at` nul | **éligible** | — | — |

- L'ordre des lignes est la **priorité** (une ligne rapprochée **et** marquée rejetée rend
  `RECONCILIATION_ALREADY_RECONCILED`).
- ⛔ **Anti-énumération** : les trois causes de la première ligne rendent **le même code, sans `details`**, par le
  **même chemin** (une seule requête scopée société **et** compte). Rien dans la réponse ne distingue une
  transaction supprimée d'une transaction d'une autre société ; le compte bancaire du corps reste contrôlé avant
  (`404 BANK_ACCOUNT_NOT_FOUND`, inchangé).
- Le `match` sur `BankTransactionStatus` est **exhaustif, sans `_`** : un statut ajouté plus tard doit faire échouer
  la compilation, pas tomber dans une branche par défaut.
- Tests unitaires de la fonction (module `#[cfg(test)]` de `reconciliation.rs`, à côté des tests existants l. 4445
  et suivantes) : les quatre lignes du tableau, plus « rapprochée et rejetée » ; codes écrits **en dur** dans les
  tests, jamais relus d'une constante de production.

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
  positif (`400`) ; compte bancaire inconnu, d'une autre société ou archivé (`404 BANK_ACCOUNT_NOT_FOUND`) ; verrou
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
`with_account_lock`. Test : le verrou nommé du compte est tenu par une connexion du test (montage de
`reconciliation_e2e.rs:1486-1511`, nom `reconcile:{base}:{company}:{account}`) ; un lot dont toutes les transactions
sont absentes répond `200` en **moins de 2 s** (`LOCK_TIMEOUT_SECS` = 5, l. 65) — sous la mutation qui retire le
court-circuit, il rendrait `409` au bout de 5 s.

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
  auto_match_rejected_at IS NULL` : un rejet concurrent de la même transaction, ou une ligne déplacée, ne la
  ré-horodate plus en silence (la trace d'audit la comptait deux fois).
- Si l'`UPDATE` ne touche aucune ligne, la ligne est **relue par une lecture verrouillante**
  (`SELECT … WHERE id = ? AND company_id = ? AND bank_account_id = ? FOR UPDATE` — lecture *courante*, jamais
  la vue cohérente de la transaction, qui pourrait dater d'avant la suppression) et **reclassée par la fonction de
  l'AC1** : supprimée → `BANK_TRANSACTION_NOT_FOUND`, rapprochée → `RECONCILIATION_ALREADY_RECONCILED`, rejetée →
  `RECONCILIATION_ALREADY_REJECTED`. Si la relecture la trouve **éligible** (état que l'`UPDATE` aurait dû
  toucher — non atteignable), `tracing::warn!` et repli sur le refus d'aujourd'hui
  (`RECONCILIATION_ALREADY_RECONCILED`, `details.reason = "race_during_update"`).
- La trace `reconciliation.rejected` est inchangée (identifiants **rejetés** seulement, `entity_id` = le plus
  petit).

**AC5 — Un lot de plus de 500 identifiants est refusé en `400`, avant toute lecture (C-27-4-9).**

Aujourd'hui, `find_pending_by_ids` rend `Err(DbError::Sqlx(Protocol))` au-delà de `MAX_IN_CLAUSE_IDS` = 500
(`repositories/reconciliation.rs:263`, `:293-298`), ce qui sort en **`500`**. Les deux routes refusent désormais un
lot de plus de 500 identifiants en `400 VALIDATION_ERROR` à l'étape 0 (validation du corps, avant toute requête —
même famille que « proposals vide »). La borne est **une seule constante** : `MAX_IN_CLAUSE_IDS` devient `pub` dans
`kesh-db` (renommée si utile, ex. `MAX_BATCH_IDS`) et la route la lit — pas de second littéral `500`. Tests : 500
identifiants passent le contrôle (aucun `400` de taille), 501 rendent `400` ; les deux routes.

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
   identifiant inexistant → `rejected` = l'éligible ; `failed` = les trois autres **dans l'ordre croissant**, codes
   du tableau de l'AC1.
5. **Rejet concurrent de la même transaction** (AC4, garde `auto_match_rejected_at IS NULL`) — montage du 2. ; au
   lieu de supprimer, le test pose `auto_match_rejected_at` par `UPDATE` direct avant de relâcher le verrou → la
   ligne sort en `RECONCILIATION_ALREADY_REJECTED` ; **une seule** trace `reconciliation.rejected` en base pour
   elle (aucune du lot sous test).
6. **Acceptation d'une transaction rejetée** — `POST /accept` sur une transaction rejetée → `200`,
   `failed[0].errorCode = "RECONCILIATION_ALREADY_REJECTED"` ; le manuel continue de dire vrai (« l'acceptation
   d'une proposition la refuse », `user-manual.tex:1920`).

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
  gardent `reconciliation-errors-already-reconciled`. *(Ce code reste émis par trois sites de l'acceptation —
  `:1934-1940`, `:2435-2440`, `:2887-2892` — que la story ne modifie pas côté serveur ; le libellé cesse
  d'affirmer « déjà réconciliée » quand la cause peut être une suppression.)*
- Les quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` portent les deux clés
  neuves et le texte récrit (FR l. 1187, autres l. 1124) ; les replis en dur du `.ts` sont **identiques au FR**.
  Proposition DE / IT / EN dans les Dev Notes, à confronter à `docs/i18n-glossaire.md`.
- Le doc-comment du module (l. 1-50) recompte : **27 littéraux** (`grep -ohE 'error_code: "[A-Z_]+"'
  crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`, 26 aujourd'hui) **+ 2** codes de
  `DbError::error_code()` = **29 codes** ; et nomme le cinquième code qui lit ses `details`
  (`RECONCILIATION_ALREADY_RECONCILED`, pour `race_during_update`).
- `failed-proposal-label.test.ts` : `EXPECTED_KEYS` gagne `RECONCILIATION_ALREADY_REJECTED`, le décompte passe de
  28 à **29** (l. 67) et le commentaire d'en-tête (l. 1-8) avec lui ; un test pour `race_during_update` (clé neuve
  lue), un pour une autre raison (clé générique), un pour `details` absent.
- `frontend/src/lib/shared/i18n-keys.test.ts:580` : `sitesTotal` **2041 → 2043** (deux appels `i18nMsg(` ajoutés ;
  recompter, ne pas ajuster) ; `npm run lint-i18n-ownership` vert.

**AC8 — L'écran : rien à restructurer, un comportement à prouver (C-27-4-12, C-27-4-14).**

`ReconciliationProposals.svelte` affiche déjà `failed[]` ligne par ligne (`:432-449`), avec la transaction relevée
**avant** le lot (`snapshotSelected`, `:67-73`) et recharge la liste après (`:229`, `:248`). Aucun changement de
structure n'est attendu ; si le développement en trouve un nécessaire, il le consigne (choix `C-27-4-k`).

- **Vitest** (`ReconciliationProposals.test.ts`) : un lot d'acceptation à deux lignes dont une sort en
  `BANK_TRANSACTION_NOT_FOUND` → compteur `1`, l'*Échec partiel* désigne la transaction supprimée par sa **date et
  son montant relevés** (pas `TX #…`) avec le libellé récrit, `reconciliation-error` **absent** ; le test existant
  `:450-474` (« TX #99 ») est mis à jour au texte récrit.
- **E2E Playwright** (spec neuf `frontend/tests/e2e/reconciliation-deleted-import.spec.ts`, ou ajout motivé à un
  spec existant) : deux imports CAMT par l'API (relevé réécrit à chaque exécution — montants et références uniques,
  patron de `reconciliation-cancel.spec.ts:86-110`), une **règle d'affectation** créée par l'API qui rend les deux
  transactions sélectionnables (patron de `reconciliation-rules.spec.ts`) ; la page *Réconciliation* est ouverte,
  les deux lignes cochées ; **puis** l'import A est supprimé par `DELETE /api/v1/bank-imports/{id}`
  (`authedApiContext`) ; clic *Rejeter* → `reconciliation-success` montre `1`, un `reconciliation-failed-item`
  porte `data-error-code="BANK_TRANSACTION_NOT_FOUND"` et `data-tx-id` = la transaction de A, son texte contient le
  libellé récrit ; `reconciliation-error` absent. ⚠️ C'est le seul test qui voit le `200` traverser la frontière HTTP
  jusqu'à l'écran (`CLAUDE.md`, § *E2E*).

**AC9 — Le manuel utilisateur dit vrai, PDF compris (C-27-4-13).**

`docs/manual/fr/user-manual.tex` :

- **l. 1756-1759** (§ *Supprimer un import*, puce « Une page *Réconciliation* ouverte avant la suppression ») :
  réécrite — la transaction supprimée apparaît dans les *Échecs partiels* (« Transaction introuvable … »), les
  autres propositions du lot sont traitées ; plus de « refusé **en entier** », plus de consigne de recharger avant
  de recommencer.
- **l. 1925** (§ *Un refus n'empêche pas les autres*) : la liste des motifs gagne « transaction supprimée avec son
  import, rapprochée ou rejetée entre-temps (dans un autre onglet, par exemple) ».
- l. 1920 (« l'acceptation d'une proposition la refuse ») : **reste vrai** — vérifier, ne pas toucher sans raison.
- PDF régénéré (`make user` dans `docs/manual/`, sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`) et
  **contrôlé aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` contient les phrases
  neuves et ne contient plus « refusé en entier » dans § *Supprimer un import* (la phrase homonyme de l'import CSV,
  l. 1786, est légitime et reste).

**AC10 — L'API publique et le CHANGELOG (C-27-4-10).**

- `docs/api-external.md`, après le paragraphe du succès partiel (l. 538) : une transaction **introuvable** (supprimée,
  d'un autre compte ou d'une autre société), **rapprochée** ou **rejetée** est refusée dans `failed[]` —
  `BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_ALREADY_REJECTED`, sans
  `details` — au lieu du `400 VALIDATION_ERROR` du lot entier, *(depuis la v0.13.2)*, pour `accept` comme pour
  `reject` ; le plafond de 500 identifiants (`400`, auparavant `500`) ; le paragraphe du rejet (l. 548) dit que
  `failed[]` est trié en entier. ⚠️ **Changement de contrat** à écrire comme tel : un client qui traitait le `400`
  reçoit désormais un `200` à lire ligne par ligne.
- `CHANGELOG.md`, section **`## [0.13.2] — Non publié`** (existante, l. 11) : sous un **`### Corrigé`** (à créer
  entre *Modifié* et le séparateur `---`, l. 21) — le lot de réconciliation qui contient une transaction disparue
  n'est plus refusé en entier ([#634](https://github.com/guycorbaz/kesh/issues/634)) ; sous *Modifié* — le contrat
  API (`200` + `failed[]`, code neuf, plafond `400`). Lignes **ajoutées**, sans réécrire celles de la 27-1 (ni
  celles des 27-2 / 27-3 si elles ont été mergées avant : fusion par union).

**AC11 — Le dépôt : une lecture par identifiants sans filtre de statut (C-27-4-10).**

- `find_pending_by_ids` (`repositories/reconciliation.rs:265-318`), qui n'a que les deux appelants de cette story
  (`grep -rn "find_pending_by_ids" crates` : `reconciliation.rs:950`, `:3049`, plus des commentaires de tests), est
  **remplacée** par une fonction de même forme **sans filtre de statut ni de rejet** — nom proposé
  `find_by_ids_for_account` — scopée `company_id` **et** `bank_account_id`, même plafond, même `HashMap`.
  Doc-comment : qui l'appelle, pourquoi aucun filtre (la classification est faite par l'appelant, AC1).
- Test de dépôt (`crates/kesh-db/tests/reconciliation_repository.rs`) : rend une ligne `pending`, une `reconciled`,
  une rejetée ; n'en rend aucune d'un autre compte ni d'une autre société ; liste vide → vide ; 501 → erreur.
- ⚠️ `kesh-db` touché : **gate complet même en cours de boucle** (`CLAUDE.md`, § *Pendant une boucle de revue*,
  exception `kesh-db`).

**AC12 — Les textes du dépôt qui décrivaient le défaut sont mis à jour.**

- `crates/kesh-db/src/repositories/bank_imports.rs:424-427` (angle mort « lot préparé avant la suppression … le lot
  entier rend `400 VALIDATION_ERROR` … Non corrigé ici (#634) ») : réécrit au comportement neuf.
- `crates/kesh-api/tests/reconciliation_e2e.rs:1734-1737` (« `failed[]` n'est pas asserté : le pré-vol refuse en
  `400` … ») : corrigé ; le test `post_reject_locks_the_batch_by_increasing_id` reste vert tel quel.
- `post_reject_skips_reconciled_transactions` (`:1843`, assertion `:1894`) et
  `post_reject_after_accept_returns_already_reconciled_failed` (`:1968`, assertion `:2035`) : passent à `200`,
  `failed[0].errorCode = "RECONCILIATION_ALREADY_RECONCILED"`, la ligne inchangée en base ; leurs commentaires de
  « déviation » sont retirés.
- Doc-comments de `post_accept` (étape 0ter, `:944`, et le commentaire « fail-fast 400 », `:969-973`), de
  `post_reject`, de `accept_batch` et du module (l. 1-16) : au comportement neuf.
- Recherche de propagation, **valeurs** et non formulations : `grep -rnE "appartiennent pas|find_pending_by_ids|fail-fast 400|en entier" crates frontend/src docs`
  — chaque site résiduel trié (légitime ou corrigé) et nommé au Change Log.

**AC13 — Mutations : chacune fait rougir au moins un test nommé.**

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | le pré-vol d'`accept` rétablit le `400` global sur absence | AC6-1 (accept), AC6-3 |
| M2 | idem pour `reject` | AC6-1 (reject), AC6-4 |
| M3 | une ligne `reconciled` classée éligible | AC1 unitaire ; AC6-4 |
| M4 | une ligne rejetée classée éligible | AC1 unitaire ; AC6-6 |
| M5 | priorité inversée (rejetée avant rapprochée) | AC1 unitaire « rapprochée et rejetée » |
| M6 | `details` d'un `BANK_TRANSACTION_NOT_FOUND` porte `bankAccountId` | AC6-3 (comparaison des deux corps) |
| M7 | court-circuit de l'AC3 retiré | AC3 (`200` en < 2 s sous verrou tenu) |
| M8 | `failed[]` d'`accept` non remis dans l'ordre reçu (refus du pré-vol ajoutés en tête ou en fin) | AC2 (a) |
| M9 | `failed[]` de `reject` non trié | AC6-4 (ordre croissant) |
| M10 | `AND auto_match_rejected_at IS NULL` retiré de l'`UPDATE` du rejet | AC6-5 |
| M11 | relecture de l'AC4 non verrouillante (`FOR UPDATE` retiré) | AC6-2 (rejet) — à confirmer au développement ; si équivalente sous le montage, la déclarer **équivalente** avec la raison |
| M12 | `batch_size` = taille du sous-lot éligible | AC2 (b) |
| M13 | plafond de l'AC5 relevé à 501 | test AC5 (501 → `400`) |
| M14 | libellé `race_during_update` non lu (clé générique) | Vitest AC7 |
| M15 | `RECONCILIATION_ALREADY_REJECTED` absent de `failedProposalLabel` | Vitest AC7 (`EXPECTED_KEYS`) |

Toute mutation restée verte est soit corrigée par un test, soit déclarée **équivalente** avec sa raison au Change
Log. ⚠️ Après chaque mutation restaurée, **toucher** le fichier (`touch`) : cargo garde sinon le binaire muté.

## Tasks / Subtasks

- [ ] **T0 — Montage** (AC —) : base `kesh_274` et `kesh_e2e_274` (migrations du worktree, seed
  `scripts/seed-dev-db.sql`), port `3034`, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-27-4/target`,
  `node_modules` par `cp -al` depuis le dépôt principal. Gates lourds sous `flock
  /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.
- [ ] **T1 — Dépôt** (AC11, AC5) : `find_by_ids_for_account` à la place de `find_pending_by_ids`, constante de
  plafond `pub` ; test de dépôt.
- [ ] **T2 — Classification** (AC1) : fonction pure + tests unitaires.
- [ ] **T3 — `post_accept`** (AC2, AC3, AC5) : pré-vol classant, sous-lot éligible, `batch_size` explicite,
  fusion de `failed[]` dans l'ordre reçu, court-circuit, plafond.
- [ ] **T4 — `post_reject`** (AC4, AC3, AC5) : pré-vol classant, `reject_batch` sans `tx_map`, garde de
  l'`UPDATE`, relecture verrouillante et reclassement, `failed[]` trié.
- [ ] **T5 — Tests d'intégration** (AC6, AC12) : six scénarios, trois tests existants réécrits, commentaire
  `:1734-1737`.
- [ ] **T6 — Frontend** (AC7, AC8) : libellés, quatre catalogues, `sitesTotal`, Vitest, E2E.
- [ ] **T7 — Documentation** (AC9, AC10, AC12) : manuel + PDF aplati, `api-external.md`, CHANGELOG,
  doc-comments ; grep de propagation.
- [ ] **T8 — Mutations et gates** (AC13) : campagne M1-M15 ; gate complet (`scripts/test-fast.sh`), Vitest
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
- **Verrou nommé global au serveur** (KF-038, #228) : son nom porte la base courante — reconstruire
  `reconcile:{DATABASE()}:{company}:{account}` dans les tests (`reconciliation_e2e.rs:1492-1503`).
- **Test muet** : un test d'ordre qui ne contient que des identifiants déjà triés ne prouve rien (M8, M9) — envoyer
  un lot **non trié**.
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
| `CHANGELOG.md` (`[0.13.2]`) | oui | oui | oui — union |
| `docs/manual/fr/user-manual.tex` + `.pdf` | oui | oui | oui — PDF à **régénérer** après fusion, jamais fusionné en binaire |
| `docs/api-external.md` | — | oui | oui |
| `crates/kesh-i18n/locales/*/messages.ftl` (×4) | oui | oui | oui |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | oui (2032 → 2034) | oui (2032 → 2062) | oui (2041 → 2043) — **conflit certain** : recompter après fusion |
| `crates/kesh-api/tests/reconciliation_e2e.rs` | oui (commentaire l. ~2253) | — | oui (autres régions) |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | oui | oui | oui |

`crates/kesh-api/src/routes/bank_imports.rs` et `errors.rs`, touchés par 27-2 et 27-3, ne le sont **pas** ici. La 27-2
a relevé que `reconciliation.rs` ne contrôle pas la devise sur certains chemins (C-27-2-20) : **hors périmètre**,
non touché.

### Project Structure Notes

- Fonction de classification : privée à `crates/kesh-api/src/routes/reconciliation.rs` (elle construit des
  `FailedProposal`, type de la route) ; pas de nouveau module.
- Dépôt : `crates/kesh-db/src/repositories/reconciliation.rs`.
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
