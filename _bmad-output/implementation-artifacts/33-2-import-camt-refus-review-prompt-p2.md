# Prompt — revue de code P2 ciblée de la Story 33-2, une lentille, sur la remédiation de la P1

Une seule lentille, contexte frais, braquée sur **la remédiation de la revue P1**, qui change du code de production. Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11) tient des livres réels : ce code protège des relevés déjà importés.

Worktree `/home/gcorbaz/devel/kesh-e332`, branche `story/33-2-import-camt`. Diff **aplati** à relire : `git -C /home/gcorbaz/devel/kesh-e332 diff cb572893 HEAD`. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p2.md`, et rien d'autre. Langue : français.

Contexte à lire après le diff, seulement pour juger : le **Change Log** (entrée de la remédiation P1) et le **Dev Agent Record** de `_bmad-output/implementation-artifacts/33-2-import-camt-refus.md`, C-e332-27 et 28 de `e33-choix-autonomes-e332.md`, les rapports `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p1-{B,E,A}.md`. Journaux en lecture seule sous `/home/gcorbaz/devel/kesh-gate-logs/e332/` : `mut/summary.log`, `mut/M*.log`, `vmut.py` (à **lire**, jamais à exécuter), `gate-*.log`, `front-*.log`, `e2e-*/`, `manuels*.log`. Ne prends pas le compte rendu pour argent comptant : recoupe-le.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, `pkill`, `killall`. **Aucun `git`** qui écrit (`checkout`, `switch`, `restore`, `stash`, `reset`, `rebase`, `push`, `tag`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep` (`grep` nu est une fonction dans ce shell), `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## La lentille : ce que cette remédiation a pu casser

La sévérité se déplace vers ce qu'on vient d'écrire. Cherche, dans ce diff seulement :

1. **Le contrôle du rang** (`StatementSelection.position`, `legacy_0173_aliases`) : les deux lectures rendent-elles toujours le même nombre de relevés, dans le même ordre, si bien que le rang les compare ? Un fichier ordinaire peut-il être refusé à tort, à plusieurs relevés, ou à relevés du même compte ? Sa place (après le nombre de lignes) laisse-t-elle `line_count` et `misaligned` justes ? Le relevé vide (C-e332-5) ?
2. **La clé d'alias coupée** (`LegacyLine::from_draft`) : une ligne ordinaire à identifiant long reçoit désormais un alias ; en mode strict, sans ligne stockée possible, cela change-t-il un résultat ? Le `*old_key != new_key` et le contrôle de date et de montant, qui lisent maintenant la clé coupée, disent-ils encore vrai ? La phrase « un cas reste sans signal » des manuels et du CHANGELOG décrit-elle exactement ce qui reste, ni plus ni moins ?
3. **Les tests neufs** : `camt_full_line_through_the_database_stays_recognised_922` et `camt_line_restored_from_a_backup_stays_recognised_922` prouvent-ils ce qu'ils disent ? L'import de sauvegarde repasse-t-il vraiment par la ligne restaurée ? Les mutations M26 à M42 meurent-elles par ces tests, ou par un autre chemin qui ne prouve rien ?
4. **Le panneau** (rang à partir de 1, date, montant, détails ; `preview.transactions[line.newIndex]`) : l'index est-il bien celui de ce tableau, en CAMT comme en CSV ? `sitesTotal` et la valeur `Ligne` aux quatre locales ?
5. **Les textes, `.tex` et PDF aplatis** (`docs/manual/fr/user-manual.pdf`, `admin-manual.pdf`), `docs/api-external.md`, `CHANGELOG.md` : ce que la remédiation y a changé contre le code. « levée », « réduite », « date de comptabilisation », « avant la version 0.17.4 », « depuis la version 0.17.4 », « dans le texte du document », et l'argument qui remplace « aucune banque connue ». Greper la valeur : reste-t-il l'ancienne forme quelque part ?
6. **La fiche** : les AC corrigés (R4, R9, S3, F3, F4) disent-ils ce que fait le code ? Une entrée de passe passée réécrite ?

## Le rapport

Décompte par sévérité (CRITICAL, HIGH, MEDIUM, LOW), recompté, en tête. Pour chaque constat : sévérité, lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite **la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
