# Prompt — revue de code P2 de la Story 32-5c, passe ciblée (regression hunter), Sonnet

Tu relis **la seule remédiation de la revue de code P1** de la partie c de la Story 32-5 de Kesh (comptabilité suisse,
Rust/Axum, MariaDB) : le commit **`277b7b88`** du worktree `/home/gcorbaz/devel/kesh-32-5c` (branche `story/32-5c`).
Le diff à relire : `git -C /home/gcorbaz/devel/kesh-32-5c show 277b7b88`. Son parent, `7200ac0d`, est l'état revu en P1
(rapports `/home/gcorbaz/devel/kesh-gate-logs/325c-review-p1-{B,E,A}.md`, lecture seule). Langue du rapport : français.
Modèle : **Sonnet**. Contexte frais. **Une seule lentille** : chasser ce que **ce commit** a cassé ou laissé faux. Les
findings LOW de la P1 tracés sur #738 (B2 = E4, B3 = E5, B5 = E2, B6, B7 = E6, B8, E7, E8, A3) ne sont pas à refaire.

Ce que le commit prétend faire :

1. **B1 = A1 (MEDIUM)** — `docs/api-external.md:35` réduit à : une clé `kesh_mcp_…` n'est pas une clé d'API, `/api/v1`
   la refuse en 401 ; plus rien sur la création ni sur « une clé d'API n'ouvre pas le serveur MCP ».
2. **B4 = E3 = A2** — `routes/mcp_keys.rs` : `cite()` et `SCOPE_CITED_MAX_CHARS = 64` bornent la portée inconnue
   recopiée au message de refus (coupure sur une frontière de caractère, `…` ajouté) ; test
   `an_unknown_scope_is_cited_within_bounds`.
3. **E1 = B9** — `routes/mcp_keys.rs` : `EXPIRY_MAX_YEAR = 9999`, une expiration dont l'année **UTC** dépasse 9999 est
   refusée en 400 traduit (clé neuve `admin-mcp-keys-error-expiry-out-of-range` dans les quatre catalogues, G9 de
   `textes_coherents.rs` étendue) ; test `c3_an_expiry_beyond_year_9999_is_refused_not_a_500`
   (`mcp_admin_routes_e2e.rs`). Constaté au développement : sans la garde, la clé était créée en **201 avec
   `expiresAt: null`** (permanente), non en 500.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**
  (ni `kesh-gate-logs`, ni scratchpad, ni `/tmp`) ; **aucune redirection** (`>`, `>>`, `tee`). Un fichier créé par
  erreur se **déclare** ; il ne se supprime pas.
- **Aucun `git checkout`, `git switch`, `git restore`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui compile ou exécute du code** : ni `python`/`python3`, `perl`, `ruby`, `node`/`npm`/`npx`,
  `jq`, `bash -c`/`sh -c` ; ni `cargo` (aucune sous-commande), `rustc`, `make`, `latexmk`, `docker`, `mysql`/`mariadb`,
  `sqlx`, `curl`, `sed -i` ; ni **`scripts/prepare-release.sh`** ni aucun script de `scripts/` ou de `kesh-gate-logs`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée.

## Axes

1. **`docs/api-external.md`** : la phrase de `:35` est-elle vraie à `277b7b88` (un test la porte-t-il ? lequel, et sur
   quelles routes) ? Une autre phrase du fichier la contredit-elle (`grep -niF "serveur MCP" docs/api-external.md`,
   `grep -nF "kesh_mcp" docs/api-external.md`) ? Le paragraphe dit-il encore plus que l'AC-c7 ?
2. **`cite()`** : bornes exactes (63, 64, 65 caractères, chaîne vide, caractères combinants, émojis multi-points de
   code) ; le message produit par `t_args` en contexte i18n (le paramètre `{ $scope }` reçoit-il la forme citée ?) ; le
   repli `format!` égale-t-il toujours la valeur fr-CH (G9 : la variable s'appelle-t-elle encore `portee`) ? Le test
   peut-il passer à vide ?
3. **L'expiration** : `exp.year()` est-il l'année **UTC** (`DateTime<Utc>`) ? L'ordre des deux contrôles (passé, puis
   an 9999) ; la borne exacte (`9999-12-31T23:59:59.999Z` accepté ? `DATETIME(3)` arrondit-il la milliseconde ?) ; la
   date de **création** n'est pas en cause. Le test e2e : ses deux entrées (`9999-12-31T23:59:59-05:00`,
   `9999-12-31T20:00:00-12:00`) dépassent-elles bien l'an 9999 en UTC ? Son témoin `9999-12-31T23:59:59Z` prouve-t-il la
   borne ? Le message comparé est-il celui du catalogue fr-CH ? Le doc-comment du test dit qu'une année à cinq chiffres
   est refusée par l'extracteur : est-ce établi par le dépôt ou seulement constaté ? La clé neuve est-elle présente,
   avec le même nom, dans les **quatre** catalogues, et la garde `les_comptes_cites_en_exemple_existent_dans_les_plans_livres`
   la tolère-t-elle par construction ou par hasard (forme `9999-12-31`) ?
4. **Propagation** : le même défaut d'expiration vit-il ailleurs dans le diff de la 32-5c (`git diff fa5b3468 277b7b88`)
   ou dans les doc-comments, la fiche, le registre (`grep -rnF "9999" crates/kesh-api/src/routes/mcp_keys.rs`,
   `grep -nF "expiry" crates/kesh-i18n/locales/*/messages.ftl`) ? `routes/api_keys.rs` porte le même défaut (hors
   périmètre, à tracer par l'orchestrateur) : le constat est-il exact (`sed -n 136,146p crates/kesh-api/src/routes/api_keys.rs`) ?
5. **Comptes rendus** : le Change Log « Revue de code P1 » de la fiche
   (`_bmad-output/implementation-artifacts/32-5-cles-et-reglages-mcp.md`) dit-il vrai — décomptes de tests, gates
   (`/home/gcorbaz/devel/kesh-gate-logs/325c-p1-*.log`), mutations (`325c-mutations-p1.log`, `325c-mut-Mp*.log`) ?
   Les entrées passées du Change Log ont-elles été réécrites (`git show 277b7b88 --stat`, puis `git diff 7200ac0d HEAD
   -- _bmad-output`) ? Messages de commit : `git log 7200ac0d..HEAD --format=%B | grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit ne rien rendre.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/325c-review-p2.md` :

- bilan en tête : CRITICAL / HIGH / MEDIUM / LOW ;
- findings numérotés `R1…`, chacun avec sévérité, fichier:ligne, **preuve copiée**, correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne compte
  pas comme passe ;
- rien d'autre n'est écrit nulle part.
