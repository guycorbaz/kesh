# Prompt — revue de code P4 ciblée de la Story 33-2, une lentille, sur la remédiation de la P3

Une seule lentille, contexte frais. La remédiation ne touche que des textes, la fiche et une expression d'un test Vitest : **aucune ligne de production**. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e332`, branche `story/33-2-import-camt`. Diff **aplati** : `git -C /home/gcorbaz/devel/kesh-e332 diff 0e7f3eaa HEAD`. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p4.md`, et rien d'autre. Langue : français. À lire pour juger : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p3.md` ; journaux en lecture seule : `/home/gcorbaz/devel/kesh-gate-logs/e332/manuels-4.log`, `p3-textes.log`, `p3-vitest.log`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`), pas même vers `/dev/null`. Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, `pkill`, `killall`. **Aucun `git`** qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## La lentille

1. **La rédaction unique** : « Un champ du relevé (une date, un montant, l'IBAN par exemple) écrit avec une section CDATA ou une lettre codée, que l'ancienne lecture ne savait pas lire, peut mener à ce refus » au manuel utilisateur (`.tex` et `user-manual.pdf` aplati) et au `CHANGELOG.md`. Dit-elle vrai contre `crates/kesh-import/src/camt053/mod.rs` et `legacy_0173_aliases` ? Ne promet-elle rien, n'énumère-t-elle rien de fermé ? Les deux textes se relisent-ils correctement, phrase d'avant et phrase d'après comprises (« demandez alors à la banque ») ?
2. **Rien de rassurant ne reste** : aucune phrase ne dit qu'un fichier n'y mène pas. « Un fichier qu'acceptait la 0.17.4 peut donc être refusé » est-il juste ?
3. **Propagation** : `champ obligatoire`, `n'y mène pas`, `peut y mener`, l'ancienne liste, dans `CHANGELOG.md`, `docs/` (`admin-manual.tex`, `api-external.md`), `crates/kesh-i18n/locales`, `README.md`, `website/`.
4. **La fiche** : le tableau des mutations (M4, M5, M25 contre `/home/gcorbaz/devel/kesh-gate-logs/e332/vmut.py`, à **lire**), l'annotation d'E1, le total de 43, l'entrée du Change Log ; une entrée passée réécrite ?
5. **Le test** : `FR_WHOLE` dans `BankImportUpload.test.ts` (« Ligne » au lieu d'« Index ») : le test V13 peut-il encore rougir comme il le doit ?

## Le rapport

Décompte par sévérité, recompté, en tête. Pour chaque constat : sévérité, lieu, preuve copiée, conséquence, remédiation. Ensuite **les axes réellement exercés et ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
