# Prompt — revue de code P2 de la Story 32-5b : passe ciblée, regression hunter, Opus

Tu relis **une seule chose** : la remédiation de la revue de code P1 de la partie b de la Story 32-5 de Kesh
(comptabilité suisse, Rust/Axum, MariaDB), qui porte sur l'authentification par clé MCP. Worktree
`/home/gcorbaz/devel/kesh-32-5b`, branche `story/32-5b`. Le commit de code de la remédiation est **`e2557b47`** ; le
diff à relire, **aplati**, est celui du code entre le dernier commit de la passe P1 et lui :

```
git -C /home/gcorbaz/devel/kesh-32-5b diff bdb1588c e2557b47 -- crates
```

(`bdb1588c` et `08390e61` ne portent que des artefacts de `_bmad-output/` ; le code de la partie b avant la remédiation
est `3afa49e8`.) Langue du rapport : français. Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe
antérieure, sauf pour savoir ce que la remédiation prétendait corriger.

**Le motif que tu chasses** : *la sévérité se déplace vers ce qu'on vient d'écrire*. Ta question n'est pas « la story
est-elle juste ? », mais « **cette remédiation a-t-elle introduit un défaut, affaibli une garde ou fait passer un test à
vide ?** ».

Ce que la remédiation prétend faire (rapports `/home/gcorbaz/devel/kesh-gate-logs/325b-review-p1-{B,E,A}.md`, lecture
seule ; registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md`, C-32-5-43 et C-32-5-44) :

- **E1 (MEDIUM)** — `company_id` et `user_id` valaient tous deux 1 dans les montages : société leurre dans `livres()`
  (`crates/kesh-api/tests/cles_mcp_e2e.rs`), second administrateur créateur dans
  `full_import_restores_mcp_keys_that_authenticate` (`crates/kesh-api/tests/admin_full_import_e2e.rs`), avec un
  `assert_ne!` de montage ;
- **B1 = E3** — `AND u.company_id = k.company_id` ajouté à la jointure de `find_active_auth_by_key_hash`
  (`crates/kesh-db/src/repositories/mcp_keys.rs`), et un test neuf
  (`auth_lookup_ignores_a_key_whose_creator_is_in_another_company`, `crates/kesh-db/tests/mcp_keys_repository.rs`) ;
- **C-32-5-44** — le montage de `auth_lookup_ignores_an_orphan_key` change : le créateur est orphelin de la même société
  absente que la clé, pour que la jointure à `companies` reste seule à refuser ;
- **B3 = E4** — `revoked_at` relu après l'import, et une clé révoquée après l'export, redevenue active ;
- **E6** — trois jetons hostiles portant le préfixe (`jetons_hostiles_avec_le_prefixe`) ;
- **E2** — **aucun test ajouté** : le test `touch_last_used_sets_it_but_never_on_a_revoked_key` existait déjà ; seule
  sa mutation a été jouée ;
- **A3** — la mutation « toute erreur de base devient un 401 » a été jouée.

Mutations de la remédiation, déclarées tuées (`/home/gcorbaz/devel/kesh-gate-logs/325b-p1r-mutations.log`) : mx1
(`company_id` et `user_id` intervertis), mx2 (condition de société du créateur retirée), mx3 (`revoked_at IS NULL`
retiré de `touch_last_used`), mx4 (erreur de base changée en 401), mx5 (`JOIN companies` retiré). Gates :
`325b-p1r-cible.log`, `325b-p1r-gate-complet.log` et l'E2E `325b-p1r-e2e.log`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`). Un fichier créé par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git restore`), aucun `git` qui écrit (`commit`,
  `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/regen-test-schema.sh`, ni aucun script de
  `kesh-gate-logs` ou d'un scratchpad, ni `make`, ni `cargo` (aucune sous-commande), ni `rustc`, ni `npm`/`npx`/`node`,
  ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `python3`, ni `sed -i`, ni `perl`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`,
  `find`, `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune. Ne lis rien sous `~/kesh-test/`.

## Les axes (une seule lentille : regression hunter)

1. **La condition de société du créateur** (`u.company_id = k.company_id`) :
   - casse-t-elle un appelant ou un test existant qui monterait une clé MCP dont le créateur est d'une autre société
     (`git grep -n "mcp_keys" e2557b47 -- crates` ; `admin_backup_e2e.rs`, `seed_mcp`, `mcp_keys_repository.rs`) ?
   - Contredit-elle un texte (doc-comment du module, de `McpKeyAuthRow`, de `authentifier`, la fiche, le manuel
     d'administration) qui décrirait la recherche sans elle ?
   - Est-elle compatible avec ce que la **32-5d** fera (règle des principaux orphelins : `companies.rs`, révocation puis
     repointage des clés, rattachement des utilisateurs) — une clé repointée vers la société cible garde-t-elle un
     créateur rattaché à la même société, ou devient-elle introuvable sans le dire ?
2. **Le montage orphelin réécrit** : l'`INSERT INTO users` sous `FOREIGN_KEY_CHECKS = 0` se fait-il sur la **même
   connexion** que la clé et la remise à 1 ? `last_insert_id()` sur la bonne requête ? Le test prouve-t-il encore ce
   qu'il prouvait (la jointure à `companies`), sans passer à vide ? La mutation mx5 rougit-elle **ce** test et pour la
   bonne raison ?
3. **E1 — les identifiants distincts** : la société leurre décale-t-elle bien `company_id` et non `user_id` (ordre des
   `AUTO_INCREMENT`) ? L'`assert_ne!` de montage peut-il être vrai tout en laissant une égalité ailleurs (un autre test
   du fichier qui ne passe pas par `livres()`) ? Dans `admin_full_import_e2e.rs`, le JWT d'import reste celui de
   `seed_admin` : l'import réussit-il pour la bonne raison, et le créateur `createur_mcp` est-il bien dans la
   sauvegarde (créé avant l'export) ?
4. **B3 = E4** : `mcp_key_revoked` (`CAST(revoked_at IS NOT NULL AS SIGNED)`, `fetch_optional`) distingue-t-il bien les
   trois états (absente, révoquée, active) ? K3 est-elle révoquée **après** l'export et **avant** l'import ? La
   version passée à `revoke_in_tx` (1) est-elle la bonne ?
5. **E6** : les trois jetons hostiles peuvent-ils rendre autre chose qu'un 401 pour une raison que le test ne voit pas
   (une erreur de base sur un paramètre de 100 000 caractères — `max_allowed_packet` —, un `\u{0}`) ? Si l'un rendait
   une erreur de base, le test rougirait-il ? Le message d'échec ne copie-t-il pas un jeton de 100 000 caractères au
   journal ?
6. **Ce qui n'a pas bougé et aurait dû** : les décomptes (tests ajoutés, mutations) et les déclarations du Dev Agent
   Record et du Change Log de la fiche (`32-5-cles-et-reglages-mcp.md`, entrée « Revue de code P1 ») sont-ils vrais
   à `e2557b47` et contre les journaux ? Le symptôme de chaque finding a-t-il été grepé ailleurs (un autre montage à
   identifiants égaux : `git grep -n "seed_admin\|fn livres" e2557b47 -- crates/kesh-api/tests` ; une autre jointure
   `users` sans condition de société dans `mcp_keys.rs`) ?

## Rapport

Écris ton rapport dans **`/home/gcorbaz/devel/kesh-gate-logs/325b-review-p2.md`** :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `R1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`, `sed -n` ou
  `git diff`), et, s'il vient de la remédiation, le hunk qui l'introduit ; correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
