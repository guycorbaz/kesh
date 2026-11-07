# Prompt — revue de code P2 ciblée de la Story 33-8 : la remédiation de la P1, une lentille, contexte frais

Tu relis la **remédiation de la revue P1 de la Story 33-8** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : **un compte désigné dans les paramètres de facturation ne s'archive plus sans avertissement (#486)**. Ce n'est pas la story entière : ce que la remédiation a écrit, et ce qu'elle a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque si la branche a changé. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff b4c4e8a2 0f2bba8b`**. Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p2.md`, et rien d'autre.

Lis d'abord : les trois rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p1-{B,E,A}.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-8-compte-designe-archive.md`. Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-8-p2.log`, `mut/33-8-p2-*.log` (`mut-33-8-p2.py` est à lire, jamais à exécuter), `33-8-p1/`, `e2e-r33-8-p1/`, `user-manual-33-8-p1.txt`, `admin-manual-33-8-p1.txt`. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-8-*.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`rebase`/`push`/`tag`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`,
  `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`** ; aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (dans ce shell, `grep` nu est une fonction),
  `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `head`, `tail`, `cut`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, les tubes entre ces commandes,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires

1. **Le texte du dialogue, réglage par réglage** : chaque phrase de `designatedSettingEffect`, aux quatre locales et au manuel, contre le code et contre son test (`archived_*`) : témoin avant, variante d'erreur exacte, cas qui n'est pas refusé. Une phrase qui promet plus que le test ? `revenue` (seule la validation : le brouillon ne contrôle pas le compte par défaut, nuance d'une prémisse de la P1) ; l'avoir, que le texte ne nomme pas (`user-manual.tex`, § sur les avoirs).
2. **`archive_guarded`** : la société lue sur le compte, le `NotFound`, les rollbacks ; `archive` qui délègue ; l'ordre sous-comptes, désignation, version, et M9.
3. **Le type** : `the_type_stays_checked_for_a_historical_field_only`, M10, et la différence écrite au doc-comment et au manuel d'administration, contre `validate_account` et `resolve_designated_account`.
4. **La sortie du dialogue** : `details` illisible (`archiveDesignated = []`, le message du serveur), `role="alert"`, rouvrir le dialogue.
5. **La clé `designatedSettings`** : le symptôme `designated_settings` sur tout le dépôt ; l'écran du journal d'audit l'affiche-t-il ?
6. **L'inventaire** par clé étrangère (`KEY_COLUMN_USAGE`) : passe-t-il sur `test-schema` comme sur les migrations ?
7. **Les manuels, `.tex` et PDF aplatis** : `user-manual.tex` § `sec:plan-comptable-archivage`, `admin-manual.tex` § *Un compte désigné puis archivé* ; l'exemple « escompte, désigné d'office » contre `admin-manual.tex` § *Comptes du solde du reste*, plan des associations compris.
8. **Les choix** C-e334-37 (course, limite assumée) et C-e334-38 (réarchivage) : les raisons tiennent-elles au code ?

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
