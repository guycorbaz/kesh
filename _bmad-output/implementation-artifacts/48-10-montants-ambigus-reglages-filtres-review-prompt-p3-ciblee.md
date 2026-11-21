# Prompt — revue P3 ciblée de la Story 48-10 (#1173, #1175, #1176, #1177) : une lentille, le seul commit `db9b3a4fd`

Tu relis la **remédiation des trois LOW de la passe P2 ciblée** de la Story 48-10 de Kesh (SvelteKit 5 runes), release
**0.20.0**. La P2 (Sonnet) a rendu 0/0/0/3 (`/home/gcorbaz/devel/kesh-gate-logs/e4810/review-p2-ciblee.md` : C1, C2, C3). Les
trois ont été corrigés en **un seul commit**, **`db9b3a4fd`**, qui ne doit toucher que des commentaires. Ta passe porte sur
**ce commit seul**, en diff aplati :

`git -C /home/gcorbaz/devel/kesh-48-10 diff db9b3a4fd~1 db9b3a4fd` (et `--stat`).

Une seule lentille : **exactitude des commentaires corrigés, et ce que le commit a pu casser**. Modèle : **Haiku** (la P2 était
Sonnet). Contexte frais. Langue du rapport : français.

Contexte à lire : le rapport P2 ; le registre `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4810.md`
(C-e4810-6, C-e4810-7). Journal (lecture seule) : `/home/gcorbaz/devel/kesh-gate-logs/e4810/remed-p2-vitest.log` (Vitest des
trois fichiers, 71/71).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`** ; ni `pkill`, `killall`, `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (`e2e-e4810.sh`, `mut/mutations-e4810*.py`) ; ni
  `make`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git show`, `git diff`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `wc`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Axes obligatoires

1. **Commentaires seulement** : le diff ne change-t-il **aucune ligne de code** (seules des lignes `//`, `*`, `/**`) ? Prouve-le par
   la sortie du diff.
2. **C1** — `frontend/src/lib/features/journal-entries/amount-entry-centimes.ts` : l'exception est-elle vraie au code ?
   Comparer à `JOURNAL_AMOUNT_RE` (`frontend/src/lib/features/journal-entries/balance.ts`) et aux tests
   (`amount-entry-centimes.test.ts`, « 10.01000 » illisible). Le serveur accepterait-il vraiment ces formes
   (`crates/kesh-api/src/routes/dunning_levels.rs`, `scale_within(fee, 2)` sans `normalize` : « 10.01000 » y serait-il refusé
   pour les frais ?) — la phrase dit-elle trop ?
3. **C2** — `frontend/src/routes/(app)/settings/dunning/+page.svelte`, doc-comment de `onFeeInput` : le choix (effacer un
   refus du serveur quelle que soit sa cause) est-il décrit comme le code le fait ? « un délai négatif ou non entier est déjà
   arrêté par l'écran » : vrai (`submitForm`, `Number.isInteger(delay) || delay < 0`) ?
4. **C3** — `frontend/src/routes/(app)/journal-entries/journal-entries-list-page.test.ts` : chaque commentaire est-il au-dessus
   du test qu'il décrit (« ␣1.250␣ » et « -5 ») ?
5. **Manuel** : le commit ne change aucun texte visible ; vérifie que `docs/manual/fr/admin-manual.tex` (item *Frais*, `:1348`)
   reste vrai au code (« deux décimales au plus », « Une saisie à exposant … ou précédée d'un signe n'est pas lue ») ; PDF non
   régénéré dans la story (intégration) — `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé comme ambigu'`
   (attendu 0), à nommer comme point d'intégration.

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e4810/review-p3-ciblee.md`. Findings numérotés (`D1`, `D2`…) avec sévérité
(CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes exercés ET non
exercés** ; le décompte final recompté depuis la liste. Un « 0 finding » sans cette liste ne compte pas.
