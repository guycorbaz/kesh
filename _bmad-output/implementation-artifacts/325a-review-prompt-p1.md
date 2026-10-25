# Prompt — revue de code P1 de la Story 32-5a (les tables MCP et la sauvegarde), trois lentilles, Sonnet

Tu relis le développement de la **partie a de la Story 32-5** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) : *les
tables `mcp_keys` et `mcp_server_settings`, leurs entités et dépôts, l'inventaire de la sauvegarde, la fermeture de la
fenêtre de P7, l'export global, la limite de Q4 au manuel et au CHANGELOG, le refus de la sauvegarde 0.14.0*. Worktree
`/home/gcorbaz/devel/kesh-32-5`, branche `story/32-5`, partie de `deb3b250` (`main` fusionné, 32-4a comprise). Le diff
à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-5 diff deb3b250...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie a est en revue.** Les AC des parties b, c et d (AC-b1 et suivants) ne sont pas livrés et ne sont
pas des findings ; en revanche, un état intermédiaire qui ouvrirait un accès ou ferait mentir un texte en est un
(§ *Le découpage*, dernier alinéa).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` — *Ce que le code dit* (faits 7, 10, 11, 14 à 19),
*Partie A* (AC-a1 à AC-a15), tâches T-a0 à T-a7, mutations Ma1 à Ma7, *Ce que les gardes ne voient pas*, *Dev Notes*
(schéma), *À recontrôler à l'étape 0 du développement*, et le **Dev Agent Record** de la partie a — puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md` (C-32-5-1 à C-32-5-28, et les entrées du
développement, C-32-5-29 à C-32-5-36). Le Dev Agent Record déclare des gates, des mutations et un E2E : **ne les prends
pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/325a-*.log`
(lecture seule). Aucun contrôle visible ne change : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans le scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`) : compare par
  des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé par erreur se
  **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou du scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `python3`, ni `sed -i`, ni `perl`, ni `sha384sum` sur autre chose qu'un fichier du dépôt vers la sortie standard.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `sha384sum <fichier>`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Les règles de `CLAUDE.md` § *Migration breaking policy* à tenir (P5 à P8)

- **P5** — la ligne de `20261025000001` au tableau de `docs/migrations-idempotence-audit.md`, et les compteurs
  **recomptés depuis la source** : `ls crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`,
  les deux sites du total (en-tête de section, ligne `Total`) et les trois partitions, dont la somme égale le total.
- **P6** — `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` : chaque site résout par version, ou porte un
  garde-fou fail-loud ; dans `migrations_upgrade_path.rs`, **chaque** `\b(83|49|84|50)\b` trié (historique daté ou
  valeur vivante), frontière 34 inchangée.
- **P7** — la migration n'écrit aucune donnée (premier mot-clé de chaque statement, décommenté) ; le registre
  `POST_RESTORE_BACKFILLS` vidé, `20261010000001` passé à `RETIRED_BACKFILLS` **à contenu identique** (compare
  `git show deb3b250:crates/kesh-db/src/post_restore.rs`) et à `EXEMPT_MIGRATIONS` ; la justification commence par
  `Hors fenêtre` ; aucune exemption `Durable` ne porte « aucune version publiée » ; les compteurs 9 et 21 et leurs
  lignes de généalogie ; les justifications réécrites (`20261010000002`, `20261009000001`, `20261011000001`,
  `20261015000001`, `20260917000001`) **vraies** — et leur fondement inchangé.
- **P8** — `crates/kesh-db/migrations.sha384` : la ligne `20261025000001` est-elle **égale** à `sha384sum` du fichier
  à HEAD ? Le fichier de migration a-t-il changé **après** le commit qui inscrit la ligne
  (`git log --format=%h -- crates/kesh-db/migrations/20261025000001_mcp_keys_and_server_settings.sql`) ? Aucune
  migration antérieure n'est-elle modifiée (`git diff deb3b250...HEAD --stat -- crates/kesh-db/migrations/`) ? Le squash
  `test-schema/0001_schema_squash.sql` porte-t-il les deux tables telles que la migration finale (contraintes
  `chk_mcp_keys_name_nonempty` en `REGEXP`, `chk_mcp_keys_send_validation` en `utf8mb4_nopad_bin`) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. La migration : types, défauts, FK `RESTRICT`, index, `ENGINE=InnoDB` sur les deux tables ; les `CHECK` ferment-ils
   ce qu'ils prétendent (un nom d'espaces insécables ? un tableau JSON de nombres dans `scopes` ? `send_validation`
   `NULL` ?) ; l'en-tête dit-il vrai (non breaking, fenêtre refermée, sauvegardes antérieures refusées) ?
2. `entities/mcp_key.rs` : `McpScope` (ordre, chaînes, `FromStr` strict, `ALL`) ; `McpScopes` (invariant « non vide »
   tenable par tout chemin de construction ? `Decode` qui ignorerait une erreur ?) ; `SendValidation` ; `Debug` sans
   condensat, **aucun** `Serialize`/`Deserialize` sur `McpKey`, `NewMcpKey`, `McpKeyAuthRow` (et `McpScopes` ?).
3. `repositories/mcp_keys.rs` : chaque requête scopée par société (KF-002) ; `find_active_auth_by_key_hash` joint
   `users` **et** `companies`, filtre révoquée et expirée ; `revoke_in_tx` / `set_send_validation_in_tx` sous verrou
   optimiste ; `touch_last_used` ne réveille pas une clé révoquée ; `CAST(scopes AS CHAR)` à chaque lecture.
4. `repositories/mcp_server_settings.rs` : `get` n'écrit jamais ; `save_in_tx` à `version = 0` ne peut écraser une
   ligne (course entre lecture et `INSERT`) ; l'`UPDATE` a-t-il un `rows_affected` fiable (sqlx `CLIENT_FOUND_ROWS`) ;
   décodage strict des trois listes.
5. `backup.rs`, `exports/global.rs` : places dans l'ordre enfants → parents ; commentaires « onze » et « 41 »
   recomptés sur la liste.
6. Les tests : un test peut-il passer **à vide** — une assertion de message (`msg.contains("chk_…")`) satisfaite par une
   autre contrainte, un `raw_key_insert` dont le refus viendrait d'une autre colonne (longueur de `key_hash`), le test
   orphelin dont l'insertion sous `FOREIGN_KEY_CHECKS = 0` se ferait sur une autre connexion du pool, l'aller-retour
   dont l'instantané comparerait deux listes vides, `populate` dont les lignes ne seraient pas comptées ?
7. `admin_full_import_e2e.rs` : les quatre tests du lettrage convertis gardent-ils chacun une assertion sur une marque
   posée par le rejeu ? `full_import_replays_backfills_in_increasing_version_order` : cinq versions, ordre croissant ;
   `ac10_…_est_refusee` : 400, code, aucune entrée d'audit, comptes inchangés — et la fixture n'est pas modifiée
   (`git diff deb3b250...HEAD --stat -- crates/kesh-api/tests/fixtures`) ?

## Lentille E — Edge cases

1. La sauvegarde : une ligne `mcp_server_settings` aux listes non vides, une clé révoquée, `expires_at` /
   `last_used_at` / `revoked_at` à la milliseconde, un nom UTF-8 — que fait `decode_cell` d'une colonne `JSON`, et le
   rebind à l'import ? Un `.keshbackup` qui porterait `mcp_keys` avec une portée inconnue : l'import réussit-il, et que
   devient la lecture ensuite (erreur au décodage — 500 ?) ? Est-ce dit quelque part ?
2. La remise à zéro (`reset_demo`) vide les deux tables dans cette partie : le test le prouve-t-il pour les deux, et
   rien ne mène-t-il, avant la 32-5d, à une clé créée hors des tests (aucune route) ?
3. Le refus des sauvegardes antérieures : une sauvegarde de la **0.15.0** (format 2, sans les deux tables) est-elle
   refusée par le même contrôle — raisonne sur `parse_and_verify` ; un manifeste qui porterait `mcp_keys` **sans**
   `mcp_server_settings` ?
4. Les **mutations** Ma1 à Ma7 (dix jouées : Ma1a/b, Ma2, Ma3, Ma4a/b/c, Ma5, Ma6, Ma7) : chacune tuée par une
   **assertion nommée** (journaux `325a-mut-*.log`, `325a-mutations.log`) ? Le motif muté est-il celui que décrit la
   fiche ? Une mutation plausible **non jouée** survivrait-elle — `send_validation` défaut `exempt`, `CHECK` des listes
   retiré, `set_send_validation_in_tx` sans `company_id`, `list_by_company` qui ignore `include_revoked`,
   `mcp_server_settings::get` qui écrirait les défauts ? Raisonne sur le code, n'exécute rien.
5. P7 : `registry_entries_are_within_import_window` et `exemptions_claiming_out_of_window_really_are_out_of_window`
   recalculent-ils la fenêtre à `20261025000001` ? Les tests unitaires qui ne parcourent que le registre passent
   maintenant à vide (fiche, *Ce que les gardes ne voient pas*) : la machinerie reste-t-elle éprouvée par
   `post_restore_class_a.rs` et `replay_retired` ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-a1 à AC-a15, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-a1, AC-a2, AC-a3, AC-a12** : cf. P7 et P8 ci-dessus ; écarts au schéma de la fiche justifiés au registre
  (C-32-5-29).
- **AC-a6** : 41 au compte de `admin_full_export_e2e.rs`, tables nommées ; `EMPTY_BEFORE_RESET` inchangée.
- **AC-a9, AC-a10, AC-a14** : rejoue chaque `grep` de l'AC-a14 et recopie la sortie ; chaque reste est-il une
  exception nommée par la fiche ?
- **AC-a11** : cf. P5 et P6.
- **AC-a13 — le manuel, PDF compris** : relis `docs/manual/fr/admin-manual.tex` (§ *Ce que la sauvegarde emporte*,
  *Ce qui reste hors du fichier*, *Compatibilité*, l'avertissement qui le suit, *Reprises de données rejouées à
  l'import*, *Restaurer sur une nouvelle machine*, *Les pièces à l'import*, `:1505`) et `docs/manual/fr/user-manual.tex`
  (`:1914`, `:3027`) **contre le code** : chaque phrase est-elle vraie à HEAD (le motif journalisé « table … absente du
  manifeste », `errors.rs` ; le recalcul des numéros de client et la réparation #528, `routes/admin.rs` ; ce que garde
  une restauration du réglage de la sauvegarde automatique) ? Puis **les PDF aplatis** :
  `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` et de même `user-manual.pdf` — les phrases
  neuves y sont-elles (« mêmes tables », « au tag de cette version », « sans attendre une sauvegarde complète »,
  « clés du serveur MCP », « sauvegarde automatique ») et les anciennes absentes (« égale ou plus récente »,
  « 0.15.0 ou plus récente », « appliquera cette fois », « rejoue donc ces reprises ») ? Le PDF est-il celui du `.tex`
  à HEAD (date de commit des deux) ? Le grep de classe de l'AC-a13 (5), **corrigé par P8-1** (`égale ou`, `.tex` seul,
  deux §), et chaque occurrence restante justifiée. Le CHANGELOG `[0.16.0]` : `### Modifié` avant `### Corrigé`, même
  substance que le manuel.
- **AC-a15** : cf. lentille B, point 7.
- **Étape 0** : chaque point de *À recontrôler à l'étape 0 du développement* traité au Dev Agent Record, ligne lue citée.
- **Décomptes** : tests ajoutés recomptés (`git show deb3b250:<f>` contre `HEAD`, `grep -cE '#\[(sqlx::)?test'`),
  périmètre déclaré ; chiffres des gates et de l'E2E contre les journaux, au **dernier commit de code**
  (`git log --oneline deb3b250..HEAD`) ; échecs E2E jugés fichier par fichier contre `docs/testing.md`
  § *Les échecs attendus*.
- **Messages de commit** : `git log deb3b250..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-5a-tables-et-sauvegarde-mcp` de `sprint-status.yaml` à `review` ; registre complété
  par union (C-32-5-29 et suivantes), sans réécrire C-32-5-1 à 28 ; #747 et #748 cités.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325a-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
