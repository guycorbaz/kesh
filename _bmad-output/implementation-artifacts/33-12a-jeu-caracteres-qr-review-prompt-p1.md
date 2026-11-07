# Prompt — revue de code P1 de la Story 33-12a, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-12a** de Kesh (comptabilité suisse, Rust/Axum, version **0.17.5**, sans migration) : **le jeu de caractères de la QR-facture est celui de SIX (#941)**. Kesh tient des livres réels, et des QR-factures en sortent.

Worktree `/home/gcorbaz/devel/kesh-e3312`, **où la 33-12b se développe à la suite** : relis par `git show`/`git diff` sur les références, jamais le fichier sur disque. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e3312 diff 65441c20 03618ed6`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e3312a-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **Le code garde ses lettres, l'imprimé en dérive** (C-e3312-1). Un écart imprimé/code qui ne soit pas l'un des deux écrits serait un défaut de conformité ; `texte_imprime` ne touche pas ce que WinAnsi porte.
2. **Tout texte d'adresse passe par `lignes_adresse`**, aux douze sites ; `build_payload` passe ses douze champs par `texte_code`. Un autre lecteur de ces champs ?
3. **Un jeu élargi ne casse rien** : `\n`, `\r`, les contrôles restent refusés ; ce qui passait passe encore ; `validate` contrôle le brut, le code émet le normalisé (longueurs, 997 octets).
4. **La remise au noir** (C-e3312-8, trouvée à la capture, hors de la fiche) : la partie paiement sortait blanc sur blanc depuis `8b1e479a`. Le correctif suffit-il, tout chemin de dessin compris (rappels, avoirs) ? La couleur restante ailleurs ?
5. **Le décodeur n'a pas changé** : il lit aussi les QR de tiers.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche `_bmad-output/implementation-artifacts/33-12a-jeu-caracteres-qr.md` en entier, dont le **Change Log** et le **Dev Agent Record**, le registre `e33-choix-autonomes-e3312.md`, `docs/six-references/ig-qr-bill-v2.4-en.pdf` (§ 3.6.2, § 4.1.1). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e3312/` : `rouges-a*.log`, `mut/a-resume.log`, `mut/a-*.log`, `gate-a/`, `sonde/`. Les scripts qu'on y trouve se lisent, ne s'exécutent pas. **Captures** : `captures/a-caracteres.png` (avant le correctif de couleur), `captures/a2-caracteres.png` ; regarde-les si ton outil lit les images.

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

1. **Le manuel (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex`, paragraphe *Les caractères d'une QR-facture* ; le PDF aplati du commit de documentation se relit dans `/home/gcorbaz/devel/kesh-gate-logs/e3312/user-manual-33-12a.txt`. Promet-il quelque chose d'absent ? Le CHANGELOG `[0.17.5]` aussi.
2. **Le jeu** : les plages du test contre le § 4.1.1 du PDF, relu ; ce que la fiche dit refusé.
3. **La normalisation** : ordre retrait puis rognage, vacuité, `is_invisible` contre ce que `texte_code` retire, message et `billing_information`.
4. **La table des 125 lettres** : chaque base, recomptée.
5. **Les autres sorties du texte d'adresse** : corps de la facture, rappels, avoirs, MCP, import : à la lettre de « le reste du PDF n'est pas dans cette story ».
6. **Les tests qui prouvent moins** : un test vert par construction, l'outil `textes_du_pdf`, `couleurs_des_textes`, les mutations déclarées (survivante ? équivalente abusive ?).

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne : champs d'invisibles, U+00AD en bordure, longueurs à 70/71 avec un U+00A0 de bordure, 997 octets, adresse sans débiteur, message `None`.
- **A (Acceptance Auditor)** : chaque AC de la fiche (1 à 11), chaque mutation (M1 à M21), chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
