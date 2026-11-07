# Prompt — revue de code P4 ciblée de la Story 33-8 : la remédiation de la P3, une lentille, contexte frais

Tu relis la **remédiation de la revue P3 de la Story 33-8** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : l'archivage d'un compte désigné dans les paramètres de facturation (#486). Le texte du dialogue qui dit ce que l'archivage entraîne a été faux trois fois, chaque fois sur un geste qu'il promettait de débloquer autrement (`revenue`, les règlements, puis l'avoir et l'annulation). La remédiation le réduit à une règle qui ne dit d'autre remède que la réactivation. Ce n'est pas la story entière : ce que le commit de remédiation a écrit, et ce qu'il a pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff 35841367 5ee57d69`** (un commit de code, `5ee57d69`). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p4.md`, et rien d'autre.

Lis d'abord : le rapport `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p3.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-8-compte-designe-archive.md` (branche `story/33-8-compte-designe-archive`). Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `mut/synthese-33-8-p4.log`, `mut/33-8-p4-*.log` (`mut-33-8-p4.sh` est à lire, jamais à exécuter), `33-8-p3/`, `e2e-r33-8-p3/`, `user-manual-33-8-p3.txt`. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-8-*.png`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

⚠️ **Hors périmètre, déjà tracé** : les autres désignateurs de comptes (#939) ; le réarchivage, la course avec le `PUT` des réglages et le focus de l'avertissement (#954).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, y compris `2>/dev/null`). Un fichier créé par erreur se **déclare**.
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

1. **La règle, contre le code** : « Tant que ce compte est archivé, toute écriture qui l'emploie est refusée, y compris sur une pièce déjà validée. Pour lever le blocage, réactivez-le. » Est-elle vraie pour chacun des dix réglages, sans exception de geste (règlement, solde, avoir, annulation d'une facture fournisseur, contre-passation, rapprochement) ? Lis `journal_entries.rs:116-117`, `:165-166` (garde `active` inconditionnelle), et cherche un chemin d'écriture qui emploie un compte archivé **sans** passer par cette garde, ou qu'une réactivation ne débloquerait pas. La phrase promet-elle quoi que ce soit d'autre que la réactivation ?
2. **L'exception `vatDecompte`** : sa phrase seule quand il est le seul réglage listé, la règle seule sans lui, les deux ensemble sinon, la règle quand `details` est illisible ; ces quatre cas, à la lecture du gabarit (`accounts/+page.svelte`) et des Vitest.
3. **Le même texte partout** : quatre locales (justesse de `de`, `it`, `en`), repli Svelte égal au catalogue `fr-CH`, manuel `.tex` et PDF aplati, CHANGELOG. Plus aucune promesse sur « un autre compte désigné » ni sur « les pièces à venir » (`git grep` sur ces formules, code, tests, manuels, CHANGELOG, `docs/api-external.md`).
4. **Les tests** : `…until_reactivated` gardent leur séquence et leur message ne promet rien de plus ; le Vitest du compte créanciers envoyé à chaque enregistrement ; chaque mutation de `synthese-33-8-p4.log` rougit-elle pour la bonne raison ?
5. **La fiche** : File List complète contre le diff de la branche depuis `bdc3faa4` ; Change Log.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
