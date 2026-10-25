# Prompt — revue de code P3 de la Story 32-6a, passe ciblée (regression hunter), Sonnet

Tu relis **une seule chose** : la remédiation de la revue de code P2 de la **Story 32-6a** de Kesh (comptabilité
suisse, Rust/Axum, MariaDB) — l'endpoint `/mcp` du serveur MCP et ses gardes, la story de sécurité de l'epic. Le motif
qui justifie cette passe : *la sévérité se déplace vers ce qu'on vient d'écrire.* Ce qu'il reste à relire n'est plus la
story, c'est le **commit de la remédiation**.

- Worktree : `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`.
- Le commit à relire, **seul** : `dd4b8398` — `git -C /home/gcorbaz/devel/kesh-32-6 show dd4b8398`. L'état d'avant :
  `4d99cdcd`. Ne relis pas le reste de la story, sauf pour juger un effet du commit.
- Modèle : **Sonnet**. Contexte frais. Une seule lentille : **regression hunter**. Langue du rapport : français.

Lis d'abord : le rapport de la passe P2 (`/home/gcorbaz/devel/kesh-gate-logs/326a-review-p2.md`) ; dans la fiche
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`, l'entrée **« Revue de code P2 »** du Change Log, le §
*Ce que les gardes ne voient pas* (correction datée du 2026-10-25, revue P2, H1), les annotations datées de l'AC-a10, de
N26 et de l'ordre des contrôles des Dev Notes ; au registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md`,
**C-32-6-56 et C-32-6-57** et l'annotation de C-32-6-54. Les gates et mutations déclarés se recoupent aux journaux
`/home/gcorbaz/devel/kesh-gate-logs/326a-p2-*` et `326a-mut-P2-*.log` (lecture seule) : ne les prends pas pour argent
comptant.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  sous `/tmp`, ni dans un scratchpad, ni dans `kesh-gate-logs`. **Aucune redirection** (`>`, `>>`, `tee`) : enchaîne
  par des pipes. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`**, aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`,
  `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui compile ou exécute du code** : ni `python`, `perl`, `ruby`, `node`, `npm`, `npx`,
  `jq`, `bash -c`/`sh -c` d'un script ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
  `scripts/mem-guard.sh`, `scripts/install-hooks.sh`, aucun script du dépôt, de `kesh-gate-logs` ou d'un scratchpad ;
  ni `cargo` (aucune sous-commande), `rustc`, `make`, `latexmk`, `docker`, `mysql`/`mariadb`, `sqlx`, `curl`,
  `sed -i`, `kill`, `pkill`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `pdftotext … -` (sortie sur la sortie standard).
- **Toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée** dans le
  rapport.
- Aucune donnée réelle ; ne lis rien sous `~/kesh-test/`.

## Les axes obligatoires

1. **Le changement de construction (H1, C-32-6-56) lu de bout en bout.** `repartiteur::appeler` lance l'outil dans un
   `tokio::spawn` qui possède l'outil (`Arc<dyn McpTool>`), une copie de `ContexteAcces` et de `LectureOutils`. Lis
   `tokio` à la source (`task::spawn`, `JoinHandle` : son `Drop` détache-t-il la tâche ou l'annule-t-il ?
   `JoinError` : panic et annulation), et `tokio::time::timeout` sur un `JoinHandle`. Cherche : un état que la tâche
   devrait posséder et qu'elle emprunte encore ; une propriété de la politique (`autoriser` avant toute exécution,
   `appliquer` seul chemin de sortie d'une donnée) que la tâche contourne ; une donnée d'outil qui partirait après un
   abandon ; un panic d'outil qui fuirait un texte libre (E5) ; un changement de comportement pour `delai_depasse`,
   `erreur_interne_texte_fixe`, `cle_requise_refusee_sans_execution`, `e1_rien_ne_part_sous_cle_requise`. Juge
   l'option écartée (garde `Drop` + `KILL QUERY`) au registre : la raison écrite tient-elle au code ?
2. **Les deux tests neufs discriminent-ils ?** `abandon_au_delai_le_permis_reste_pris` et
   `abandon_par_deconnexion_le_permis_reste_pris` (`crates/kesh-api/tests/mcp_transport_e2e.rs`) avec l'outil
   `LecteurRetarde` (`tests/common/mcp.rs`) : le témoin (trois lectures en cours côté serveur) peut-il passer à vide ;
   la sonde (`kesh_version`, 300 ms) peut-elle être bloquée pour une **autre** raison que le permis (le pool ordinaire
   de la `garde`, l'authentification, le verrou `LOCK TABLES` lui-même — `kesh_version` lit-il `accounts` ?) ; les
   marges de temps (attente 600 ms, délai 1 s, sonde 300 ms) tiennent-elles sur une station chargée ; la base du test
   est-elle bien la seule comptée (`DB = DATABASE()`) ; le verrou est-il toujours levé, même quand une assertion
   échoue (et sinon, l'effet sur les autres tests) ? La mutation P2-Mq1 (`326a-mut-P2-Mq1-sans-tache.log`) est-elle
   tuée par **l'assertion visée** — lis le message, pas le seul verdict ?
3. **Chaque motif greppé sur le dépôt réel.** La garde `lecture_seule_jamais_defaite` lit désormais
   `max_statement_time` (C-32-6-57) : greppe le motif sur `crates/`, confronte à la liste blanche (cinq exceptions) ;
   cherche une forme plausible que le changement laisse passer (`max_statement_time` composé, `@@max_statement_time`,
   `SET STATEMENT … FOR` sur plusieurs lignes) ou fait attraper à tort. La garde
   `contexte_mcp_construit_par_le_repartiteur_seul` suit-elle la nouvelle ligne sans perdre ce qu'elle prouvait ? Les
   doc-comments ajoutés (H4, H5, H6) disent-ils vrai — rejoue les `grep` qu'ils citent ?
4. **Le texte corrigé (H1, H2) et sa propagation.** Greppe sur tout le dépôt — code, fiche, registre, `docs/`,
   manuels `.tex` **et PDF** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) — la **valeur** des symptômes corrigés :
   « quelques millisecondes » à propos de la borne, et « 403 » à propos de l'étape 9. Le manuel n'est pas touché par
   le commit : vérifie qu'aucun manuel ne décrive déjà l'étape 9, le délai d'un outil ou l'abandon d'une requête MCP
   (sinon, il est faux à présent) ; dis-le avec la sortie de `grep`.
5. **Les comptes rendus.** L'entrée « Revue de code P2 » du Change Log dit-elle seulement ce qui a tourné ; ses
   décomptes se recomptent-ils (tests neufs et leur périmètre, mutations, gates aux journaux `326a-p2-*`) ; les
   annotations datées réécrivent-elles une entrée passée au lieu de l'annoter (l'entrée « Revue de code P1 » doit être
   restée telle quelle) ?

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/326a-review-p3.md`, par l'outil
d'écriture de fichier, jamais par une redirection :

1. Les findings, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et ligne,
   preuve (sortie `grep -nF` ou `sed -n` copiée), correction proposée, et s'il est **né du commit `dd4b8398`** ou
   antérieur.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. Le décompte `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
