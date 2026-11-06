# Prompt — revue de code P3 ciblée de la Story 33-2, une lentille, sur la remédiation de la P2

Une seule lentille, contexte frais, braquée sur **la remédiation de la revue P2** : des textes, le numéro de ligne du panneau des doublons, une mutation. Aucune ligne de Rust de production n'a changé. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e332`, branche `story/33-2-import-camt`. Diff **aplati** : `git -C /home/gcorbaz/devel/kesh-e332 diff 50db6151 HEAD`. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p3.md`, et rien d'autre. Langue : français. À lire ensuite pour juger : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p2.md`, le Change Log de `_bmad-output/implementation-artifacts/33-2-import-camt-refus.md` ; journaux en lecture seule sous `/home/gcorbaz/devel/kesh-gate-logs/e332/` : `mut/summary.log`, `mut/M43.log`, `vmut.py` (à **lire**, jamais à exécuter), `p2-*.log`, `manuels-3.log`.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, `pkill`, `killall`. **Aucun `git`** qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `claude`, `sed -i`, `playwright`, `vitest`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## La lentille

1. **La liste des causes** (`CHANGELOG.md`, `docs/manual/fr/user-manual.tex`, `user-manual.pdf` aplati) contre `crates/kesh-import/src/camt053/mod.rs` (`handle_stmt_text`, `apply_balance`, `emit_transactions`, `finalize_statement`) et `routes/bank_imports.rs` (`legacy_0173_aliases`). Un champ dont la perte fait échouer la relecture manque-t-il ? La phrase du CHANGELOG, « Un fichier qui n'écrit aucun de ces champs ainsi, dans aucun de ses relevés, se lit de la même façon dans les deux lectures et n'y mène pas », est-elle exacte, y compris pour un fichier dont les références ou les libellés portent des entités ? Une date de valeur est-elle à tort présentée comme cause ?
2. **Le panneau des doublons** : `{dup.newIndex + 1}` sous « Ligne » ; `apply_duplicate_lines_filter`, l'envoi et les tests lisent-ils encore `newIndex` tel quel ? Quatre locales. Le manuel utilisateur parle-t-il d'un index ?
3. **M43** : tuée par quoi, et le motif porte-t-il bien sur la production du rang ?
4. **La fiche** : AC-N3, tableau des mutations (M4, M5, M25 à M43 contre `vmut.py` et `mut/summary.log`), décomptes, entrée du Change Log, note L5 ; une entrée passée réécrite ?
5. **Propagation** : l'ancienne liste (« la date de comptabilisation, un montant ou l'IBAN »), « ni référence », « Index » du panneau des doublons subsistent-ils ailleurs ?

## Le rapport

Décompte par sévérité, recompté, en tête. Pour chaque constat : sévérité, lieu, preuve copiée, conséquence, remédiation. Ensuite **les axes réellement exercés et ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe.
