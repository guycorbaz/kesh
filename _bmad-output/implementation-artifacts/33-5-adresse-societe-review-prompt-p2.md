# Prompt — revue de code P2 ciblée de la Story 33-5 : la remédiation de la P1, une lentille, contexte frais

Tu relis la **remédiation de la revue P1 de la Story 33-5** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : **corriger l'adresse de la société (#909)**. Ce n'est pas la story entière : ce que la remédiation a écrit, et ce qu'elle a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff 0c3d6eae c371ee8f`** (commits `e2191fbb`, `c371ee8f`). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e335-review-p2.md`, et rien d'autre.

Lis d'abord : les trois rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/e335-review-p1-{B,E,A}.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-5-adresse-societe.md`. Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `rouges.log`, `mut/synthese-33-5-p2.log`, `mut/33-5-p2-*.log` (`mut-33-5-p2.py` est à lire, jamais à exécuter), `33-5-p1/`, `33-5-p1b/`, `e2e-r33-5-p1b/`, `user-manual-33-5-p1.txt`, `admin-manual-33-5-p1.txt`.

⚠️ **Hors périmètre, décidé par l'orchestrateur** : la liste des caractères de `is_six_annex_c_char` n'est pas conforme à la directive SIX (§ 4.1.1) ; elle se corrige dans une story séparée, la 33-12. Ne la signale pas. Vérifie en revanche que cette remédiation n'épingle aucun caractère appelé à changer (`&`, `<`, NBSP, U+2019, U+2014…), ni dans les tests, ni dans les manuels, ni dans les textes, et que `validation.rs` ne gagne qu'une délégation.

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

1. **Le refus de caractère** : `validate_required_for_qr` applique-t-il la règle à ce que le PDF contrôle, sur les valeurs telles qu'elles s'écrivent (rognées) ? Un caractère accepté ici et refusé au PDF, ou l'inverse, dans ces quatre champs ? Le message (`escape_debug`, `U+`) ? `validate_required` reste-t-il inchangé pour l'assistant, et le trou restant est-il tracé ?
2. **L'audit** : la condition sur l'ancien texte ; une société ordinaire garde-t-elle exactement ses cinq composants (`the_audit_entry_carries_the_five_components_only`) ? Le no-op ?
3. **Le pays** : `address_country_known` contre le contrôle du PDF, vide, blancs, minuscules ; `to_ascii_uppercase` et ce qu'il change pour les contacts (`validate_optional_keeping`, comparaison à `stored_country`).
4. **Les preuves** : chaque mutation de la synthèse rougit-elle pour la bonne raison ? Reste-t-il un champ que rien ne prouve ?
5. **Les manuels, `.tex` et PDF aplatis, et le CHANGELOG** contre le code : `admin-manual.tex` § *Pays des adresses*, `user-manual.tex` § `sec:coordonnees-emetteur`, le symptôme (`#909`, « ne corrige pas son adresse ») sur tout le dépôt.
6. **La garde G18** (`textes_coherents.rs`) : l'exemption est-elle bornée au chemin de la route ?

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
