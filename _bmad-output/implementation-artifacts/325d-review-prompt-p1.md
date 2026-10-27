# Prompt — revue de code P1 de la Story 32-5d (la clé MCP à la remise à zéro et à la réparation), trois lentilles, Sonnet

Tu relis le développement de la **partie d de la Story 32-5** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*la remise à zéro de démonstration conserve les clés du serveur MCP (`mcp_keys`) et vide ses réglages
(`mcp_server_settings` : serveur désactivé) ; la règle des principaux orphelins (#528) verrouille, révoque puis repointe
aussi les clés MCP orphelines ; ses quatre consommateurs écrivent les deux champs neufs ; les manuels le disent*.
Worktree `/home/gcorbaz/devel/kesh-32-5d`, branche `story/32-5d`, partie de `d993106c` (`main`, 32-5a et 32-5b
fusionnées). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-5d diff d993106c...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie d est en revue.** Les AC de la partie c (routes de gestion, audit, i18n, `api-external.md`) ne
sont pas livrés et ne sont pas des findings ; en revanche, un état intermédiaire qui ouvrirait un accès ou ferait mentir
un texte en est un (fiche, § *Le découpage*, dernier alinéa).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md` — *Ce que le code dit* (faits 8, 9, 17),
*Partie D* (AC-d1 à AC-d3), tâches T-d0 à T-d4, mutations Md1 à Md4, *Ce que les gardes ne voient pas*, *Fichiers
touchés (prévus)* pour la 32-5d, *À recontrôler à l'étape 0*, et le **Dev Agent Record** de la partie d (étape 0,
notes, décomptes) — puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-325.md`
(C-32-5-11, 12, 22, 23, 27, 43, et les entrées du développement de la 32-5d, C-32-5-45 à C-32-5-50). Le Dev Agent
Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux `/home/gcorbaz/devel/kesh-gate-logs/325d-*.log` (lecture seule). Aucun contrôle visible ne change : pas de
capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni ailleurs ; **aucune redirection** (`>`, `>>`, `tee`) : compare par
  des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé par erreur se
  **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **N'exécute rien qui écrit, compile ou exécute du code** : ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad, ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `npm`/`npx`/`node`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni
  `python3`, ni `sed -i`, ni `perl`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection),
  `cat`, `ls`, `find`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. `crates/kesh-db/src/repositories/companies.rs` : `lock_active_orphan_keys` et `repoint_orphans` interpolent un nom
   de table par `format!` — ne reçoivent-ils jamais que des littéraux du module ? Le `SELECT … FOR UPDATE` et l'`UPDATE`
   sont-ils ceux d'avant pour `api_keys` et `users` (mêmes colonnes, même prédicat `ORPHAN`, même ordre `ORDER BY
   t.id`) ? L'ordre des verrous est-il bien `users → api_keys → mcp_keys` à l'exécution ? `RevokedKey` et ses deux
   alias : un condensat (`key_hash`) peut-il fuir au détail d'audit ? `is_empty` : les cinq champs y sont-ils ?
   `mcp_keys::revoke_in_tx` sur une ligne verrouillée peut-il rendre `OptimisticLockConflict`, et que se passe-t-il
   alors (toute la remise à zéro, la réparation au démarrage, l'import) ?
2. `crates/kesh-db/src/backup.rs` : `RESET_PRESERVED_TABLES` reste-t-elle **triée** et sous-ensemble de
   `TABLES_TO_TRUNCATE` ? `reset_cleared_tables` rend-elle encore `mcp_server_settings` ? Les règles 1 à 4 du test de
   partition tiennent-elles avec `mcp_keys` conservée (règle 3 : `mcp_keys` ne référence-t-elle aucune table vidée ?
   règle 2 : pourquoi `mcp_keys` n'y entre-t-elle pas ?) ?
3. Les quatre consommateurs (`repair_installation_in_tx`, `kesh_seed::reset_demo`, `routes::onboarding` — choix de la
   langue sans société —, `auth::bootstrap`) : chacun écrit-il les deux champs, avec la forme stable (`[]`, `0`) ?
4. Les tests (`companies_repository.rs`, `onboarding_audit_e2e.rs`, `bootstrap.rs`, `support/installations_atteintes.rs`)
   : un test peut-il passer **à vide** — une clé MCP qui n'authentifierait pas **avant** pour une autre raison (le
   contrôle positif de montage existe-t-il ?), un `authentifier` refusé parce que le créateur n'est pas Administrateur
   ou pas de la même société (C-32-5-43) plutôt que parce que la clé est révoquée, un montage sous
   `FOREIGN_KEY_CHECKS = 0` sur une autre connexion du pool, un instantané `snapshot` qui ne lirait pas `mcp_keys` ?
   Les attentes à égalité JSON stricte couvrent-elles les deux champs neufs ?

## Lentille E — Edge cases

1. La règle des orphelins : clé MCP orpheline **seule** (aucun utilisateur ni clé d'API orphelin) ; clé MCP orpheline
   **déjà révoquée** ; clé MCP orpheline **expirée** mais non révoquée (est-elle « active » au sens du `SELECT` ?
   faut-il la révoquer ?) ; installation **sans société** (`target = None`) ; **deux** sociétés ; base **sans
   utilisateur** (étape 4 de `repair_installation_in_tx` : une clé MCP peut-elle exister sans créateur ?). Le créateur
   d'une clé MCP orpheline est-il rattaché à la même cible, si bien que `find_active_auth_by_key_hash`
   (`u.company_id = k.company_id`) reste cohérent après repointage ?
2. La remise à zéro : `mcp_server_settings` vidée — `get_or_defaults` rend-il bien `enabled = false` ensuite ? Une clé
   MCP créée pendant la démonstration **garde l'accès** aux livres de production : est-ce dit au manuel, et est-ce le
   comportement voulu (C-32-5-11) ? L'import d'une sauvegarde (`routes/admin.rs`, étape 5-quater) : le commentaire
   « aucun verrou neuf » est-il vrai pour `mcp_keys` (vidée puis réinsérée par la même transaction ?) ?
3. Concurrence (raisonnée, non testée — *Ce que les gardes ne voient pas*) : deux démarrages sur une base sans société ;
   une remise à zéro concurrente d'un import ; une révocation de clé MCP par une route (32-5c, à venir) pendant la
   règle. L'ordre `users → api_keys → mcp_keys` est-il le même partout où il est écrit
   (`companies.rs`, `kesh-seed/src/lib.rs`, `bootstrap.rs`, `routes/admin.rs`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`)
   ? Un site qui l'écrit a-t-il été oublié (`git grep -nF "api_keys" -- docs crates | grep -iF "verrou\|lock"`) ?
4. Les **mutations** Md1 à Md4 et md5 à md7 (`325d-mutations.log`, `325d-mut-*.log`) : chacune tuée par une
   **assertion nommée** ? Le motif muté est-il celui que décrit la fiche ? Une mutation plausible **non jouée**
   survivrait-elle — l'ordre des verrous inversé (`mcp_keys` avant `api_keys`), les clés MCP révoquées sans être
   ajoutées à `mcp_keys_revoked`, `mcp_keys_repointed` rendu par l'`UPDATE` des `api_keys`, `bootstrap.rs` qui
   journalise un mauvais champ, `mcp_server_settings` ajoutée à `RESET_PRESERVED_TABLES` ? Raisonne sur le code,
   n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-d1 à AC-d3, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-d1** : `RESET_PRESERVED_TABLES` et sa doc ; règle 4 (doc et message) ; « six » → « sept » ; le test
  d'`onboarding_audit_e2e.rs` (clé MCP active qui **authentifie** après `reset_demo`, `mcp_server_settings` vide) ;
  `mcp_keys` dans les **deux** listes d'instantanés du test de la remise à zéro.
- **AC-d2** : `OrphanPrincipals` (`mcp_keys_revoked: Vec<RevokedMcpKey>`, `mcp_keys_repointed`), `is_empty` ; les
  quatre consommateurs ; les tests (i) avec et sans cible, (ii) l'orphelin MCP seul (`installation.repaired` écrite, la
  clé au détail) ; montages d'orphelins (le commun **et** `make_orphans` d'`onboarding_audit_e2e.rs`) ; textes de
  l'ordre des verrous à **chaque** site du fait 9.
- **AC-d3 — le manuel, PDF compris** : relis `docs/manual/fr/admin-manual.tex` (réinitialisation `:1414`, défaut #528
  `:1416`, § *Réparation de l'installation* `:2467` et son détail `:2472`) et `docs/manual/fr/user-manual.tex`
  (`:190`, `:3204`, `:3214-3215`) **contre le code à HEAD** : chaque phrase est-elle vraie (conservation des clés MCP,
  serveur désactivé, révocation et rattachement des clés MCP orphelines, deux listes au détail) ? Une phrase
  annonce-t-elle ce qui n'existe pas encore (une page des clés MCP, un écran — 32-7) ? Une autre phrase des manuels
  est-elle devenue fausse par la partie d (`grep -nF "clés d'API" docs/manual/fr/*.tex`, `grep -nF "réinitialis"`,
  `grep -nF "installation.repaired"`, et § *Rollback* `:2859` « les clés révoquées restent révoquées ») ? Contrôle le
  **PDF aplati**, pas seulement le `.tex` :
  `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}serveur MCP.{80}"`, de même
  pour `user-manual.pdf` ; les PDF sont-ils régénérés depuis le `.tex` de HEAD (une phrase neuve du `.tex` absente du
  PDF est un finding) ? Le PDF de la brochure est-il inchangé (`git diff --stat d993106c...HEAD -- docs/manual`) ?
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md:339-340` dit-il le nouvel ordre ? L'absence d'entrée au CHANGELOG
  (C-32-5-49) tient-elle ?
- **Étape 0** : chaque point de *À recontrôler à l'étape 0* qui concerne la partie d traité au Dev Agent Record, ligne
  lue citée — et ces lignes, relues à `d993106c` (`git show d993106c:<f> | sed -n …`), sont-elles exactes ?
- **Décomptes** : tests ajoutés recomptés (`git show d993106c:<f>` contre `HEAD`,
  `grep -cE '#\[(sqlx::|tokio::)?test'`), périmètre déclaré ; chiffres du gate complet, du build du frontend et de
  l'E2E contre les journaux, au **dernier commit de code** (`git log --oneline d993106c..HEAD`) ; le binaire de l'E2E
  bâti après ce commit ; échecs E2E jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*.
- **Exception `kesh-db`** : la partie d touche `backup.rs`, `repositories/companies.rs` et leurs tests ; le gate
  complet a-t-il tourné au dernier commit de code, base remise à zéro juste avant (`CLAUDE.md`, *Test Locally First*,
  exception `kesh-db`) ?
- **Messages de commit** : `git log d993106c..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit ne rien rendre ; chaque commit porte `refs #723`.
- **Registre et statut** : clé `32-5d-cle-mcp-principal` de `sprint-status.yaml` à `review` ; registre complété par
  union (C-32-5-45 et suivantes), sans réécrire C-32-5-1 à 44.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325d-review-p1-<B|E|A>.md` (selon ta lentille) :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `B1…`/`E1…`/`A1…`, chacun avec sévérité, fichier:ligne, **preuve copiée** (sortie `grep -nF`,
  `sed -n` ou `git diff`), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
