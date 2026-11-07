# Prompt — revue de code P5 ciblée de la Story 33-8 : la remédiation de la P4, une lentille, contexte frais

Tu relis la **remédiation de la revue P4 de la Story 33-8** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version 0.17.5, sans migration : l'archivage d'un compte désigné dans les paramètres de facturation (#486). La P4 a trouvé fausse la phrase à part du décompte TVA (« n'est employé par aucune écriture : son archivage n'en refuse aucune ») : le compte reçoit à la main le décompte et le paiement à l'AFC, que la garde `active` refuse. La remédiation retire l'exception entière : la règle générale s'affiche sans condition, pour les dix réglages. Ce n'est pas la story entière : ce que les commits de remédiation ont écrit, et ce qu'ils ont pu casser.

Worktree `/home/gcorbaz/devel/kesh-e335`, où plusieurs stories se succèdent : relis par `git show`/`git diff`, jamais le fichier sur disque. Bornes : **`git -C /home/gcorbaz/devel/kesh-e335 diff 62c08cbe 8ec3a46a`** (commits de code `eaf360bb` et `8ec3a46a`). Langue : français. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p5.md`, et rien d'autre.

Lis d'abord : le rapport `/home/gcorbaz/devel/kesh-gate-logs/e338-review-p4.md` ; le Change Log de `_bmad-output/implementation-artifacts/33-8-compte-designe-archive.md` (branche `story/33-8-compte-designe-archive`), entrée « remédiation de la revue P4 ». Journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e335/` : `mut/synthese-33-8-p5.log`, `mut/synthese-33-8-p5b.log`, `mut/33-8-p5*-*.log` et `.diff` (`mut-33-8-p5.sh` et `mut-33-8-p5b.sh` sont à lire, jamais à exécuter), `33-8-p4/`, `e2e-r33-8-p4/`, `user-manual-33-8-p4.txt`. **Manuel** : `docs/manual/fr/user-manual.tex`, § « Archiver un compte désigné dans les paramètres », et son **PDF** `docs/manual/fr/user-manual.pdf`, à contrôler aplati. **Captures** (thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/e335/captures/33-8-*.png`, dont `33-8-p4-decompte-seul.png`.

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

1. **La règle sans exception, contre le code** : la règle s'affiche-t-elle désormais dans tous les cas (décompte TVA seul, réuni à d'autres, `details` illisible ou vide), à la lecture du gabarit `accounts/+page.svelte` ? Est-elle vraie pour le décompte TVA (`journal_entries.rs`, garde `active` ; `create` manuel) ? Le dialogue, le manuel ou un test disent-ils encore, sous une autre forme, qu'un réglage ou un compte archivé ne refuse rien ?
2. **Le symptôme retiré partout** : clé `accounts-archive-effect-vat-decompte` et `data-testid="account-archive-designated-vat-decompte"` absents des quatre locales, du gabarit et du manuel `.tex` et PDF aplati ; les `.test.ts` qui s'y réfèrent n'en vérifient que l'absence ; `sitesTotal` de `i18n-keys.test.ts` recompté (2850 → 2849) ; CHANGELOG cohérent ; formules « employé par aucune écriture », « n'en refuse aucune » et leurs traductions (`git grep`, apostrophes droite et courbe).
3. **Les tests** : le test Rust renommé `archived_vat_decompte_refuses_a_manual_entry_but_no_automatic_flow` prouve-t-il ce que dit son nom (refus `InactiveOrInvalidAccounts` d'une saisie manuelle sur le compte archivé, passage après réactivation, aucun refus des flux automatiques) ? Le Vitest « le décompte TVA seul : la règle générale, sans autre phrase » ; le Vitest du compte créanciers, déplacé dans son `describe` (L2 de la P4). Chaque mutation de `synthese-33-8-p5.log` et `synthese-33-8-p5b.log` rougit-elle pour la bonne raison, et le motif de `V-masque` corrigé produit-il bien la mutation voulue (lire le `.diff`) ?
4. **La fiche** : Change Log de la remédiation P4 contre le diff, les journaux de mutation et de gate (chiffres recomptés) ; File List contre `git diff --name-only bdc3faa4 <tête de la branche>`.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Puis **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité, recompté, en tête.
