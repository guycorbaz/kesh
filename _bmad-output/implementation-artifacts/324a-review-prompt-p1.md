# Prompt — revue de code P1 de la Story 32-4a (rollout des lectures : écritures, postes ouverts, contacts), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-4a** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) : *la couche de
services — rollout des lectures : écritures comptables, postes ouverts, contacts*. Worktree
`/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a`, partie de la fiche validée `39939799` (3 passes de validation,
terminée). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-4a diff 39939799...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md` — *Ce que le code dit* (faits 1 à 9),
*Le rollout*, AC1 à AC17, tâches T0 à T7, *Dev Notes* (*Hors périmètre*), et le **Dev Agent Record** — et le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md` (C-32-4a-1 à C-32-4a-12, plus les entrées du
développement, C-32-4a-13 et suivantes). Le motif que ce rollout applique : `crates/kesh-api/src/services/mod.rs`
(règles R1 à R8). Le Dev Agent Record déclare des gates, des rouges et des mutations : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/324a-*.log`, `324a-mut-*.diff`,
`324a-mutations-campagne.log` et `324a-ac13-tri.txt` (lecture seule). La story n'ajoute aucun contrôle visible : pas de
capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`), pas même vers
  un fichier « jetable » : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement.
  Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris),
  ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni
  `sed -i`, ni `perl`. Aucune commande `gh` qui écrit ; `gh issue view 723|738|742` est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-api/src/services/ecritures.rs` : `lister` — l'ordre bornes → analyse de `dateFrom`, `dateTo`,
   `amountMin`, `amountMax` → montants négatifs → `amountMin > amountMax` → `dateFrom > dateTo` → `accountId <= 0` →
   description rognée est-il **exactement** celui qu'a retiré `list_journal_entries` (compare
   `git show 39939799:crates/kesh-api/src/routes/journal_entries.rs`, l. 345-487) ? Les **neuf** messages sont-ils
   identiques au caractère près (les aides `analyser_date`/`analyser_montant` reproduisent-elles `"{nom} invalide
   ({e})"`) ? La conversion `CoreJournal → DbJournal` est-elle la même ? `detail` : société, `find_by_id` hors
   transaction, **une** transaction pour les quatre lecteurs, `rollback` ; la garde lexicale déplacée
   (`get_journal_entry_reads_in_one_transaction`, `mod tests`) — compare-la jeton par jeton à l'ancienne
   (`git show 39939799:crates/kesh-api/src/routes/journal_entries.rs`, l. 1005-1080) : un motif a-t-il été **relâché**
   ? peut-elle passer à vide (corps mal délimité, `fin` au mauvais endroit) ?
2. `services/postes_ouverts.rs` : `parse_as_of` identique à celle qu'a retirée `routes/letterings.rs` (l. 504-527) ;
   ordre `exiger` → `asOf` → bornes → société → connexion → `open_items` ; `LIMITE_MAX` partagé avec
   `get_lettering_proposals` (une seule constante ?).
3. `services/carnet.rs` : `lister` lit la société **d'abord**, `detail` le contact **avant** la société ; défauts du
   tri (`unwrap_or_default()`, `SortDirection::Asc`) ; `find_by_id_in_company` (et non `find_by_id`) ; bornes 20 / 100.
4. Les cinq handlers minces (`routes/{journal_entries,letterings,contacts}.rs`) : statuts, corps (`ListResponse`,
   `JournalEntryDetailResponse` champ par champ, `OpenItemsResponse`, `contact_response_with_label`), ordre des
   erreurs identiques ? Aucun ne lit plus `state.pool` ? Les défauts serde (`default_limit`,
   `default_open_items_limit`) rendent-ils les constantes du service ? Imports morts, `get_company_for` encore
   employé là où il faut ? Le `mod tests` de `contacts.rs` inchangé, celui de `journal_entries.rs` seulement privé de
   la garde (compare par `sed -n '/^#\[cfg(test)\]/,$p'` en pipe sur `git show` des deux bornes) ?
5. `services/acces.rs` : `Portee::ContactsRead` ; un `match` exhaustif sur `Portee` ailleurs dans `src/` qui aurait
   dû gagner un bras ?
6. **Chaque `.pool()` des trois fichiers neufs** (R8 : la revue les lit, aucune garde ne le prouve) : n'emploie-t-on
   `lecture.pool()` qu'à lire ? aucun autre chemin vers le pool ?
7. `crates/kesh-api/tests/services_rollout.rs` : un test peut-il passer **à vide** (un `Ok` attendu qui serait aussi
   celui d'un contrôle absent, une table vide, un témoin qui ne témoigne de rien, une fixture qui n'atteint pas la
   branche — `Cas::Inexistant` qui n'appellerait pas l'entité voulue) ? `common/lecture_seule.rs` : identique à l'aide
   retirée de `services_motif.rs` ?
8. `crates/kesh-api/tests/services_motif.rs` : la garde étendue (ii), (iii), (viii) — faux vert possible (chemin faux,
   détecteur qui ne trouve rien, `corps_des_handlers` avec un préfixe vide, `sans_commentaires` qui coupe à un `//`
   dans une chaîne, compte du fichier contre somme des corps) ? La dix-neuvième ligne de `TABLE_DES_TRACES` : est-elle
   jouée au service **et** par la route ? Le bras `ContactsRead` d'`autre_portee` (un `panic!`) est-il atteignable ?

## Lentille E — Edge cases

1. Demandes limites au service : `limit` à `i64::MIN`/`i64::MAX`, `offset` à `i64::MIN` ; `dateFrom == dateTo` ;
   `amountMin == amountMax` ; `amountMin = "-0"` ; `description` de blancs Unicode ; `accountId` négatif **et** date
   invalide (lequel parle ?) ; `asOf = ""` ; `asOf = "+10000-01-01"` (400) ; `account_id = 0` aux postes ouverts.
2. Canal MCP : portées vides ; chacune des deux autres seule ; les trois ; un contexte MCP d'une **autre** société sur
   chacune des cinq lectures (AC11 (b)) — y a-t-il une lecture qui filtrerait par une autre société que celle du
   contexte (`company.id` lu contre `acces.company_id()`) ?
3. Lecture seule : quelles branches de la fermeture les fixtures n'atteignent-elles **pas** (pièce source présente,
   motifs de blocage présents, compte non lettrable, contact avec conditions de paiement) ? Le Dev Agent Record les
   nomme-t-il ? La mesure de l'AC11 (c) (1792 en autocommit **et** dans `begin()`) est-elle figée par un test qui
   rougirait si MariaDB acceptait le verrou ?
4. Les **mutations** de l'AC14 (M1a à M18, 35 dont M8 équivalente) : chaque mutation déclarée tuée l'est-elle par une
   **assertion nommée** (journaux `324a-mut-*.log`, diffs `324a-mut-*.diff`, `324a-mutations-campagne.log`) ? Le diff
   de la mutation est-il celui que décrit la fiche ? M8 est-elle vraiment équivalente
   (`kesh-db/src/repositories/journal_entries.rs:935-936`) ? Une mutation plausible **non jouée** survivrait-elle —
   p. ex. `ecritures::lister` qui lirait la société, `carnet::lister` sur `acces.user_id()` au lieu de
   `company_id()`, le défaut de tri inversé (`SortDirection::Desc`), `default_limit` du DTO ramené à 500 (le DTO passe
   `Some(params.limit)` : le défaut du service est-il alors jamais atteint par la route ?), `parse_as_of` sans la
   plage 1000-9999 ? Raisonne sur le code, n'exécute rien.
5. Le comportement **par la route** est-il tenu là où seul le service est testé : un `limit` absent sur
   `GET /api/v1/journal-entries` passe par le défaut serde du DTO, non par celui du service — les deux valent-ils
   bien la même constante ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC1 à AC17, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve :

- **AC1** : trois modules `pub mod` ; `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'`
  → exactement les deux lignes d'`acces.rs` ; huit fichiers ; aucun `forbid` redéclaré ; doc-comments ; doc-comment
  de `services/mod.rs` (services du rollout, exigences, inventaire, deux lectures non migrées).
- **AC2** : `exiger` en première instruction de chacun des cinq services ; constantes `EXIGENCE_*` aux valeurs du
  tableau du *Rollout*.
- **AC3-AC6** : chaque service contre son AC (ordre, défauts, plafonds, erreurs `AppError::Database(DbError::NotFound)`,
  types rendus).
- **AC7** : `grep -nE "\b(MAX_LIMIT|DEFAULT_LIMIT|MAX_LIST_LIMIT|DEFAULT_LIST_LIMIT|OPEN_ITEMS_DEFAULT_LIMIT)\b" crates/kesh-api/src/routes/{journal_entries,letterings,contacts}.rs`
  et `grep -nE 'clamp\(|\.max\(0\)' …` (deux lignes, dans `get_lettering_proposals`) ; recopiés au Dev Agent Record.
- **AC8** : `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests`
  → exactement `common/mod.rs` et `services_motif.rs` ; liste fermée des modifications de `services_motif.rs` ; les
  fichiers et comptes réellement exécutés recopiés au Dev Agent Record, contre `324a-cible-ac15.log` et le journal du
  gate complet.
- **AC9-AC11** : chaque cas présent dans `services_rollout.rs`, assertion nommée ; les « sans objet » dits au Dev Agent
  Record ; la mesure de l'AC11 (c) recopiée.
- **AC12** : (ii) six handlers et témoins ; (iii) douze couples, 7 / 5 / 5, compte du fichier égal à la somme des
  corps ; (vi) huit fichiers ; (viii) ; (i), (v-1), (v-2), (vii) inchangées ; limites écrites.
- **AC13** : inventaire au doc-comment et au Dev Agent Record, contre `324a-ac13-tri.txt` ; refais le tri sur un
  fichier au moins (`awk` en pipe, `#[cfg(test)]` tronqué, commentaires écartés).
- **AC14** : mutations (cf. lentille E, point 4).
- **AC15** : `kesh-db` non touché (`git diff 39939799...HEAD --stat -- crates/kesh-db` → vide) ; chiffres du Dev Agent
  Record (gate ciblé, gate complet, quatre gates frontend, E2E et liste nominative des échecs) contre les journaux ;
  gate et E2E au **dernier commit de code** (révision des journaux contre `git log --oneline 39939799..HEAD`) ; échecs
  E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **AC16** : **le manuel et l'API externe** — `docs/api-external.md` l. 213, 216-218, 405, 426-433, 441 contre le
  code déplacé (défauts, bornes, ordre des refus des postes ouverts) ; `docs/manual/fr/*.tex` **et** les PDF aplatis
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `admin-manual.pdf`) : aucun texte ne
  décrit autrement les bornes ou l'accès aux écritures, postes ouverts et contacts ; note de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md` ; doc-comments de module des trois fichiers de routes.
- **AC17** : #738 A5, P2-2, B4 (forme coupée) — chacun tenu par une garde ou une ligne, et une mutation.
- **Décomptes** : tests ajoutés recomptés (`git show 39939799:<f>` contre `HEAD`, `grep -cE '#\[(sqlx::|tokio::)?test'`),
  périmètre déclaré.
- **Messages de commit** : `git log 39939799..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-4a-services-ecritures-contacts` de `sprint-status.yaml` à `review` ; registre
  complété par union (C-32-4a-13 et suivantes), sans réécrire C-32-4a-1 à 12.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/324a-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
