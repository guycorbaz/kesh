# Prompt — revue de code P2 de la Story 32-6a, passe ciblée (regression hunter), Opus

Tu relis **une seule chose** : la remédiation de la revue de code P1 de la **Story 32-6a** de Kesh (comptabilité
suisse, Rust/Axum, MariaDB) — l'endpoint `/mcp` du serveur MCP et ses gardes, la story de sécurité de l'epic. Le motif
qui justifie cette passe : *la sévérité se déplace vers ce qu'on vient d'écrire.* Ce qu'il reste à relire n'est plus la
story, c'est le **commit de la remédiation**.

- Worktree : `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`.
- Le commit de code à relire, **seul** : `63d3f659` — `git -C /home/gcorbaz/devel/kesh-32-6 show 63d3f659`.
  Le code d'avant : `11ec6ca7`. Ne relis pas le reste de la story, sauf pour juger un effet du commit.
- Modèle : **Opus**. Contexte frais. Langue du rapport : français.

Lis d'abord : les trois rapports de la passe P1 (`/home/gcorbaz/devel/kesh-gate-logs/326a-review-p1-B.md`, `-E.md`,
`-A.md`) ; dans la fiche `_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`, l'entrée **« Revue de code
P1 »** du Change Log, le § *Ce que les gardes ne voient pas* (ajouts datés du 2026-10-25), l'AC-a28 amendé et le § *Le
manuel* (reports) ; au registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md`, **C-32-6-52 à
C-32-6-55** et l'annotation de C-32-6-42. Les gates et mutations déclarés se recoupent aux journaux
`/home/gcorbaz/devel/kesh-gate-logs/326a-p1-*` et `326a-mut-P1-*.log` (lecture seule) : ne les prends pas pour argent
comptant.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  sous `/tmp`, ni dans un scratchpad, ni dans `kesh-gate-logs`. **Aucune redirection** (`>`, `>>`, `tee`) : enchaîne
  par des pipes. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui compile ou exécute du code** : ni `python`, `perl`, `ruby`, `node`, `npm`, `npx`,
  `jq`, `bash -c`/`sh -c` d'un script ; ni `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
  aucun script du dépôt, de `kesh-gate-logs` ou d'un scratchpad ; ni `cargo` (aucune sous-commande), `rustc`, `make`,
  `docker`, `mysql`/`mariadb`, `sqlx`, `curl`, `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`.
- Toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée.
- Aucune donnée réelle ; ne lis rien sous `~/kesh-test/`.

## Les deux axes obligatoires

1. **Chaque motif greppé sur le dépôt réel.** Le commit touche les gardes `lecture_seule_jamais_defaite` (lignes de
   commentaire retirées avant les blocs, commentaires exécutables `/*!`/`/*M!` gardés et refusés dans le Rust),
   `un_seul_point_d_entree` (bras `kesh_api::mcp`), `mcp_sans_pool` (imports groupés), `un_seul_ecouteur` (`socket2`,
   `from_raw_fd`), et les listes blanches de `services_motif.rs` (`EXCEPTIONS_V1`) et de `mcp_gardes_lexicales.rs` qui
   suivent la ligne de câblage de `lib.rs`. Pour chacune : greppe son motif sur son périmètre, confronte à la liste
   blanche ; cherche une forme plausible que le changement **laisse passer** ou **fait attraper à tort**, et dis si elle
   est écrite. En particulier : l'exemption des `.sql` pour les commentaires exécutables est-elle juste (que lit le
   squash de test, et passe-t-il jamais sur la connexion du MCP) ?
2. **Chaque dépendance lue à la source.** `max_statement_time` (MariaDB 10.11 : unité, valeur `0`, ce qu'il interrompt,
   ce qu'il ne compte pas — attente de verrou de métadonnées, instruction en file), l'erreur 1969 ; `sqlx-core` 0.8.6 :
   `after_connect` qui exécute deux instructions et échoue sur la seconde (que devient la connexion ? lis
   `pool/inner.rs`), le `move` d'une `String` dans la fermeture d'`after_connect` (`pool/options.rs`) ; `HeaderMap::get_all`
   et `HeaderMap::get` (`http` 1.x) ; le format décimal `{}.{:03}` pour un délai inférieur à la seconde ou non entier.

## Ce qu'il faut chercher, dans le seul commit `63d3f659`

- **`crates/kesh-db/src/pool.rs`** : la borne posée sur **chaque** connexion ? Un délai de 50 ms (le registre de test
  `avec_delai(50 ms)`) donne-t-il `0.050` ? Un `Duration` de moins d'une milliseconde, non nul, déclenche-t-il le
  `assert!` (`as_millis() == 0`) — et un appel de production peut-il le faire ? Le test (g) passe-t-il à vide (la
  partie (2) dépend-elle de la (1) ; le témoin « 1 juste après l'abandon » peut-il être faux par course) ?
- **`crates/kesh-api/src/lib.rs`** : le délai est-il lu **avant** que le registre soit déplacé dans `Arc` ? Les tests
  `tests/common/lecture_seule.rs` (`DELAI_OUTIL`) changent-ils le comportement des suites de services qui l'emploient ?
- **`crates/kesh-api/src/services/serveur_mcp.rs`** et `mcp/outils/kesh_version.rs` : `@@session.max_statement_time`
  se décode-t-il bien en `f64` sur MariaDB 10.11 (type SQL) ; `EtatServeur` a perdu `Eq` — une conséquence ailleurs ?
  `kesh_version` rend-il maintenant une donnée qu'il ne devrait pas (AC-a28 amendé) ?
- **`crates/kesh-api/src/mcp/transport.rs`** : `unique` et ses quatre usages — un en-tête répété refusé à **chaque**
  site, et nulle part ailleurs ? L'ordre des contrôles est-il inchangé (le nouveau test `enveloppe_avant_la_cle` le
  prouve-t-il pour les quatre cas, et pour l'`Origin`) ? L'étape 9 rend-elle **exactement** le 401 de l'étape 7, et le
  403 reste-t-il possible pour une clé valide par un autre chemin ? Reste-t-il un refus après l'étape 7 qui dépende de
  la clé ?
- **`crates/kesh-api/src/mcp/reseau.rs`** : la coupe du sous-chemin de `public_base_url` — `https://h?x`, `https://h#y`,
  `https://[::1]:8443/app`, `https://h:/app`, un schéma en majuscules, une base vide ; peut-elle produire une origine
  propre **plus large** que voulu ?
- **Les tests ajoutés ou modifiés** (`mcp_transport_e2e.rs` : `origine`, `chemins_voisins`, `enveloppe_avant_la_cle`,
  `en_tetes_repetes_refuses`, `le_delai_du_registre_borne_l_instruction` ; `pool_lecture_seule.rs` (g) ;
  `mcp_gardes_lexicales.rs`) : chacun discrimine-t-il ce qu'il vise ? Les mutations `326a-mut-P1-*.log` sont-elles
  tuées par **l'assertion visée** (lis le message, pas le seul verdict) ?
- **Les comptes rendus** : l'entrée « Revue de code P1 » du Change Log dit-elle seulement ce qui a tourné ; ses décomptes
  se recomptent-ils (tests, mutations, périmètre) ; les annotations datées (A5, C-32-6-42, AC-a28) réécrivent-elles une
  entrée passée au lieu de l'annoter ?

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/326a-review-p2.md`, par l'outil
d'écriture de fichier, jamais par une redirection :

1. Les findings, chacun : identifiant (`H1`, `H2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne,
   preuve (sortie copiée), correction proposée, et s'il est **né du commit `63d3f659`** ou antérieur.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. Le décompte `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
