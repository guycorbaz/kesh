# Prompt — revue de code P1 de la Story 33-12b, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-12b** de Kesh (comptabilité suisse, Rust/Axum, version **0.17.5**, sans migration) : **le reçu et la partie paiement impriment l'adresse entière et la référence (#941)**. Elle suit la 33-12a, relue à part. Kesh tient des livres réels.

Worktree `/home/gcorbaz/devel/kesh-e3312` : relis par `git show`/`git diff` sur les références, jamais le fichier sur disque. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e3312 diff 4093a974 00d7c26f`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e3312b-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Rien n'est perdu à la coupe** : `couper_ligne`, chaque mot, chaque caractère, multioctets compris ; le chemin rapide sous `max`, le chemin découpé au-delà.
2. **`lignes_adresse` est le seul chemin**, aux quatre blocs, et compose comme l'exige l'AC1 ; le pays comme l'exige l'AC2, `LI` compris, `CH` jamais.
3. **Les `y` suivent le nombre réel de lignes**, aux deux colonnes ; le cas ordinaire ne descend pas sous le point de dépôt ; les limites écrites à AC3 et AC4 sont justes, recalculées.
4. **La référence au reçu**, à sa place, partagée sans changer la partie paiement.
5. **Le type K** : rien n'y est composé, et le motif écrit est le bon.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche `_bmad-output/implementation-artifacts/33-12b-adresse-imprimee-qr.md` en entier, dont le **Change Log** et le **Dev Agent Record**, le registre `e33-choix-autonomes-e3312.md` (C-e3312-3 à 8), `docs/six-references/ig-qr-bill-v2.4-en.pdf` (§ 3.6.2). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e3312/` : `rouges-b.log`, `mut/b-resume.log`, `mut/b-*.log`, `mut/mut-b.py` (à **lire**, jamais à exécuter), `gate-b/`, `gate-b2/`. **Captures** (`captures/b-1-suisse-courte.png`, `b-2-rue-longue.png`, `b-3-debiteur-etranger.png`, `b-4-creancier-li.png`, `b-5-sans-numero.png`) ; regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/`, `~/kesh-test/`, `/home/gcorbaz/travail/`, ni le NAS. Fixtures synthétiques seulement.

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
  `sed -n`, `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex`, paragraphes *Le reçu et la partie paiement portent l'adresse entière* et *Ce que cette coupe ne garantit pas* ; le PDF aplati se relit dans `/home/gcorbaz/devel/kesh-gate-logs/e3312/user-manual-33-12b.txt`. Le CHANGELOG `[0.17.5]` aussi.
2. **La calibration** : 27 et 46, recalculés ; le débiteur de la partie paiement ; une adresse réelle ordinaire dans chaque colonne.
3. **Les bornes verticales** : les chiffres d'AC4, recalculés depuis les pas de `pdf.rs`.
4. **Ce qui n'imprime rien** : un champ vide, un champ d'invisibles, une rue seule, un numéro seul, un pays vide ou inconnu.
5. **Les tests qui prouvent moins** : un test vert par construction, les `y` exacts, les mutations déclarées (survivante ? équivalente abusive ?) et leur répartition contre la fiche.

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne : `max` et `max + 1`, mot de `max` caractères, espaces doubles, espace de tête, `Ĳ` qui allonge après le repli, NPA à espace interne, adresse sans débiteur, QR sans référence.
- **A (Acceptance Auditor)** : chaque AC de la fiche (1 à 7), chaque mutation (M1 à M15, et les ajoutées), chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
