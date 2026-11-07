# Prompt — revue de code P1 de la Story 33-2, trois lentilles (B, E, A), Sonnet ou Opus

Tu relis la **Story 33-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), version **0.17.5**, sans
migration : **l'import CAMT.053 refuse le relevé qu'il ne sait pas relire « à la 0.17.3 » (#922, P1), et l'alias ne reconnaît plus
une ligne que si ses sept textes sont ceux de la lecture héritée (#915, réduit, non supprimé)**. Kesh tient des livres réels :
ce code protège des relevés déjà importés.

Worktree `/home/gcorbaz/devel/kesh-e332`, branche `story/33-2-import-camt`. Diff **aplati** à relire :
`git -C /home/gcorbaz/devel/kesh-e332 diff bdc3faa4 HEAD`. Passe **complète**, contexte frais, langue du rapport : français.
Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e332-review-p1-{B,E,A}.md`, un par lentille, et rien d'autre.

⚠️ **Les propriétés centrales.**
1. **La protection des livres.** Une ligne importée par la 0.17.3 ou avant, reprise par un autre fichier, reste reconnue. Les corps de
   `camt_import_of_0173_stays_recognised_after_the_819_fix` et `camt_import_of_0173_overlap_forced_imports_all_three_819` sont inchangés
   (`git diff` de ces deux fonctions vide) : c'est la preuve. Cherche ce qui ferait d'une ligne vraiment déjà importée une ligne
   « incertaine », donc réimportée : une colonne qui diffère entre ce qu'écrit l'INSERT et ce que recalcule `parse_legacy_0173`.
2. **Le refus est sans contournement et n'écrit rien** : les quatre causes, aux deux routes, avant `create_import_or_roll_back`.
3. **Aucun refus à tort** : un relevé ordinaire n'y mène pas (AC-N3). `parse_date` et la relecture héritée ne reçoivent aucune borne
   d'année (frontière avec la 33-4, C-e332-22).
4. **Le chemin de clé directe, le CSV et la saisie manuelle ne changent pas** (AC-S4).
5. **Le faux doublon est réduit, non supprimé** : tout texte qui dirait « supprimé » ou « levé » pour AC-F5 est un défaut.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant, seule) : la fiche
`_bmad-output/implementation-artifacts/33-2-import-camt-refus.md` en entier, dont le **Change Log** et le **Dev Agent Record** ; le registre
`e33-choix-autonomes-e332.md` (C-e332-1 à 7, 21, 22, 25 et suivants) ; `crates/kesh-api/src/routes/bank_imports.rs`,
`crates/kesh-core/src/bank_imports.rs`, `crates/kesh-import/src/camt053/mod.rs`, `crates/kesh-api/tests/bank_imports_e2e.rs`,
`frontend/src/lib/features/bank-import/BankImportUpload.svelte`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec
les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e332/` : `rouges.log`, `mut/summary.log`, `mut/M*.log`
(le script `vmut.py` est à **lire**, jamais à exécuter), `gate-*.log`, `front-*.log`, `e2e-*/`, `manuels*.log`.
**Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e332/captures/` : le refus, le panneau des lignes incertaines, un aperçu
à entités lues en entier ; regarde-les si ton outil lit les images.

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

1. **Les manuels (`.tex` et PDF aplati) contre le code** : `docs/manual/fr/user-manual.tex` (§ *Détection des doublons*, § *Import dans Kesh*, FAQ)
   et `docs/manual/fr/admin-manual.tex` (§ *Passer à la 0.17.4*, phrase datée) ; les **PDF** `docs/manual/fr/user-manual.pdf` et `admin-manual.pdf`,
   aplatis avant de greper. Les limites restantes y sont-elles toutes les deux, et justes ? Promet-on une protection absente ? Annonce-t-on un trou
   absent ? `docs/api-external.md` (paragraphes après l'*Ordre des refus*) et `CHANGELOG.md` : le rang du refus est-il celui du code ?
2. **Les causes de refus** : l'inventaire E1 à E4 de la fiche est-il clos ? Un cas de `Err` en lecture héritée, ou une divergence entre lectures,
   qu'il n'énumère pas ? Un relevé ordinaire qui y tomberait ?
3. **La comparaison des textes** : les sept colonnes contre l'INSERT réel (`new_txs`, `create_with_transactions`), `LineTexts::eq`, la coupe à la
   largeur (`cut_to_column_width`, frontières de caractère), `Some("")` contre `None`, `value_date`.
4. **L'ordre** : le refus à l'aperçu contre celui de la création (C-e332-21) ; les chemins qui écrivent avant le refus ; le verrou et la transaction.
5. **Les tests qui prouvent moins** : un test vert par construction, un montage qui n'exerce pas la branche voulue, le témoin AC-N3 et ses constantes,
   les mutations déclarées (une survivante ? une équivalente abusive ?).
6. **L'écran** : le panneau (`warning-uncertain-legacy-lines`), le singulier et le pluriel, les quatre locales, `uncertainLegacyLines` absent d'une
   réponse, les captures.

## Lentilles

- **B (Blind Hunter)** : le diff seul d'abord, sans fiche ni contexte du projet ; puis le code autour. Que casse ce diff ?
- **E (Edge Case Hunter)** : chaque branche et chaque borne. Cas à marcher : fichier à plusieurs relevés ; relevé sans ligne ; ligne présente en double
  en base ; textes de 255 et 256 caractères, IBAN de 34 et 35, `details` à 65 535 octets au milieu d'un caractère ; `reference` en CDATA ; fichier déjà importé.
- **A (Acceptance Auditor)** : chaque AC de la fiche (R1 à R9, N1 à N3, S1 à S5, F1 à F5, L1), T1 à T6, chaque mutation, chaque choix du registre : présent, testé, vrai.

## Le rapport

Pour chaque constat : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), preuve copiée, conséquence, remédiation proposée. Ensuite :
**la liste des axes réellement exercés et de ceux qui ne l'ont pas été** : un « 0 » sans cette liste ne compte pas comme passe. Le décompte par sévérité,
recompté, en tête.
