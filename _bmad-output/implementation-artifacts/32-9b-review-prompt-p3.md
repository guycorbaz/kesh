# Prompt — revue de code P3 de la Story 32-9b, passe ciblée « regression hunter », Haiku

Tu relis **une seule remédiation** : celle de la revue de code P2 de la **partie b de la Story 32-9** de Kesh
(comptabilité suisse, Rust/Axum, MariaDB) — les outils MCP de lecture des contacts (`kesh_contacts`, `kesh_contact`) et
des pièces (`kesh_pieces`). C'est une **passe ciblée** (`CLAUDE.md`, § *La passe ciblée*) : une seule lentille,
*regression hunter*, braquée sur le **seul commit de la remédiation**. Ta question n'est pas « la story est-elle
bonne ? », mais **« ce commit a-t-il cassé, affaibli ou mal dit quelque chose ? »**.

Worktree `/home/gcorbaz/devel/kesh-32-9`, branche `story/32-9b`. Le commit à relire :
`git -C /home/gcorbaz/devel/kesh-32-9 show 3c527874` (« Story 32-9b : remédiation de la revue de code P2 »). Il touche
deux fichiers : `crates/kesh-api/tests/mcp_outils_e2e.rs` (test seul, aucun code de production) et
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-329.md` (C-32-9-64). Le commit qui le suit ne porte que
des comptes rendus. Langue du rapport : français. Modèle : **Haiku**. Contexte frais.

**Garde-fou Haiku** : donne-toi un seul diff, `git show 3c527874` (jamais deux commits aplatis), et pour tout finding
affirmant qu'un code « manque » ou « reste », copie la sortie de `grep -nF` ou de `sed -n` qui le prouve ; sans elle, le
finding ne compte pas.

Lis, pour le contexte : le rapport de la P2, `/home/gcorbaz/devel/kesh-gate-logs/329b-review-p2.md` (R1 à R3), et
l'entrée « Revue de code P2 (32-9b) » du Change Log de `_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md`.
Les gates et la mutation déclarés (`329b-mutation-p2-Mr1.out`, `329b-gate-complet-p2.log`, `329b-e2e-p2.log`) se
recoupent dans les journaux de `/home/gcorbaz/devel/kesh-gate-logs/` : ne les prends pas pour argent comptant.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, aucune
  redirection (`>`, `>>`, `2>`, `tee`). Un fichier créé par erreur se **déclare** ; il ne se supprime pas : **aucun
  `rm`**.
- **Aucun `git checkout` ni `git switch`, ni `git restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `merge`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`perl`/`ruby`/`node`/`npm`/`npx`,
  ni `bash -c`/`sh -c`, ni `jq`, ni `bc` ; ni **`scripts/prepare-release.sh`**, ni `scripts/regen-test-schema.sh`, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de `kesh-gate-logs`
  ou d'un scratchpad ; ni `make`, `latexmk`, `cargo` (aucune sous-commande), `rustc`, `docker`, `mysql`/`mariadb`,
  `sqlx`, `curl`, `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find` (sans `-exec`), `wc`, `sort`, `uniq`.
- **Données réelles** : aucune. Ne lis rien sous `~/kesh-test/`.

## La lentille — regression hunter, sur `3c527874` seulement

Pour **chaque** hunk, demande-toi ce qu'il a pu casser, affaiblir ou faire dire de faux. En particulier :

1. **La pose des dates** (`filtres_de_kesh_contacts_transmis`) — l'`UPDATE contacts SET created_at = ?, updated_at = ?`
   pose-t-il bien les deux colonnes (noms exacts dans `crates/kesh-db/migrations/`) ? `updated_at` porte-t-il un
   `ON UPDATE CURRENT_TIMESTAMP` que le même `UPDATE` neutralise (la valeur explicite l'emporte) ? Un `UPDATE`
   ultérieur du test (l'archivage de X vient **avant**) réécrit-il `updated_at` ? Les dates se rangent-elles dans
   l'ordre voulu : par nom X, C, Fo ; par création Fo, C, X ; par modification C, Fo, X — trois ordres distincts ?
2. **Les attendus des quatre cas neufs** — `{"sortDirection":"desc"}` attendu `[fo, c]` suppose que le tri par défaut
   est **par nom** : lis `services/carnet.rs`/`kesh-db` (valeur par défaut de `sort_by`, `TriContacts`) et
   `mcp/outils/contacts.rs` (transmission de `sort_direction` quand `sort_by` est absent). Le cas `updatedAt` est-il
   bien relié à la colonne `updated_at` (`as_sql_column`) ? L'égalité d'ordre peut-elle dépendre d'un départage
   instable (deux dates égales) ?
3. **L'assertion `actif`** — la lecture `includeArchived: true` rend-elle `[(x,false),(c,true),(fo,true)]` dans cet
   ordre par défaut (nom) ? Peut-elle passer à vide (liste vide, clé absente → `unwrap` qui panique, donc non) ? La
   mutation Mr1 (`actif: true` en dur) a-t-elle été tuée par **cette assertion** (journal `329b-mutation-p2-Mr1.out` :
   ligne de la panique) ? Une mutation voisine (`actif: !active`) survivrait-elle ? Le `git diff` après restauration
   de `contacts.rs` est-il vide (`git diff -- crates/kesh-api/src/mcp/outils/contacts.rs` : sortie vide) ?
4. **C-32-9-64** — l'ajout dit-il vrai : `maxLength` compte les points de code du texte brut (JSON Schema), l'outil
   compte `s.trim().chars().count()` (`mcp/outils/commun.rs`) ; l'exemple « 104 points de code » est-il exact (2 + 100
   + 2) ? « Même écart pour `kesh_factures` » : greppe `"maxLength"` dans `crates/kesh-api/src/mcp/outils/`. L'entrée
   a-t-elle réécrit une partie datée de la P1 (un texte historique ne se réécrit pas) ?
5. **Propagation** : greppe les symptômes (`updated_at`, `sortDirection`, `actif`) dans les tests `mcp_*.rs` : un autre
   test dépend-il de l'ordre ou des dates de `filtres_de_kesh_contacts_transmis` ? Un compte de cas ou de tests écrit
   ailleurs (fiche, Change Log, Dev Agent Record) devient-il faux ?

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/329b-review-p3.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne, la
   preuve (sortie de commande copiée), la correction proposée, et s'il est **né de la remédiation** (`3c527874`) ou
   **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** (les cinq ci-dessus) — sans cette liste, un
   « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
