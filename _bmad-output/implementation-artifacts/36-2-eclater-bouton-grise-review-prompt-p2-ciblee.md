# Prompt — passe ciblée P2 de la Story 36-2 (#1150, #1151) : une lentille, sur le seul diff de la remédiation P1

Tu relis **la seule remédiation de la revue P1** de la Story 36-2 de Kesh (comptabilité suisse, SvelteKit 5 runes,
release 0.17.8) : la fenêtre *Éclater* de la réconciliation. La revue P1 (trois lentilles) n'a rien rendu au-dessus de
LOW (B 0/0/0/4, E 0/0/0/6, A 0/0/0/6) ; l'orchestrateur a fait corriger ces LOW dans la story (règle « Tarir la source »).
Ta passe ne relit **que ce correctif**, pour y chercher ce qu'il aurait cassé ou laissé à moitié.

Worktree `/home/gcorbaz/devel/kesh-36-2`, branche `story/36-2-eclater-grise`. Le diff, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-2 diff ee9f15714 46c26744f -- frontend crates docs`
(`ee9f15714` = dernier commit de code relu en P1 ; `2248951a1` et `46c26744f` = la remédiation, `46c26744f` = dernier
commit de code ; les commits suivants ne portent que la fiche, le registre, ce prompt et le registre de sprint).
Langue du rapport : français. Modèle : selon la rotation de l'orchestrateur (Haiku permis : passe ciblée). Contexte frais.

Lis : les rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/e36-2/review-p1-{B,E,A}.md` (ce que la remédiation devait
corriger) ; dans la fiche `_bmad-output/implementation-artifacts/36-2-eclater-bouton-grise.md`, la dernière entrée du
**Change Log** (2026-11-15) et, au **Dev Agent Record**, le tableau des mutations « Remédiation de la revue P1 » et les
gates de remédiation ; au registre `e36-choix-autonomes-36-2.md`, **C-36-2-20 à C-36-2-22**. Journaux (lecture seule)
sous `/home/gcorbaz/devel/kesh-gate-logs/e36-2/` : `r0-rem1.log`, `remise-a-neuf-rem1.log`, `test-fast-ci-rem1.log`,
`check-avertissements-rem1.log`, `front-complet-rem1.log`, `propagation-rem1.log`, `mut/campagne-rem1-*.log`,
`mut/M7-*.log`, `mut/M10.log` à `mut/M12.log`. Captures : `captures/r0-rem1-g0-largeur-sans-projet.png`,
`captures/r0-rem1-g390-telephone.png`, `captures/r0-rem1-f-somme-fausse.png`, `captures/r0-rem1-e-vide.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`** ; ni `pkill`, `killall`, `kill`. **Aucun `git`** qui écrit (`checkout`, `switch`, `restore`, `stash`…).
- **Rien qui compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`, `sh -c`, `jq`, `make`,
  `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, `claude`, `playwright`, `vitest`, `sed -i` ; ni
  **`scripts/prepare-release.sh`**, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/mut-36-2.py`,
  `mut/run-mut-36-2.sh`, `e2e-36-2.sh`, `gates-rem-36-2.sh`). **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq` ; pour un PDF, la seule
  forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (le PDF n'est pas régénéré dans la
  branche : à l'intégration).
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée ; tout chemin cité
  se vérifie (`ls` ou `git cat-file -e`).

## La lentille unique — *Regression Hunter* sur la remédiation

Axes obligatoires :

1. **Les largeurs** (`largeurs` dérivé, `sm:`, colonne du bouton sans largeur) : la classe calculée
   `` class={`${largeurs.compte} py-1`} `` est-elle vue par Tailwind (classes littérales présentes dans la source) ?
   Avec et sans *Projet*, la somme laisse-t-elle au bouton une place raisonnable (`r0-rem1.log` : `colonne_bouton:35` et
   `43`) ? Sous 640 px, aucune largeur ; le dialogue tient (`16+358`) ?
2. **L'écart non calculable** (`balanceDiff` devenu `string | null`) : chaque usage de `balanceDiff` gère-t-il `null` ?
   La branche `{:else if balanceDiff === null}` de l'indicateur ; dans `clientError`, la branche `balanceDiff === null`
   (atteinte seulement si toutes les lignes sont lues et `expectedBig` nul) rend la clé générique
   `reconciliation-split-error-imbalance` sans arguments — son repli égale-t-il la valeur fr-CH, et le contrat de l'API
   (`errors.rs`) et l'étiquette d'échec partiel restent-ils intacts ?
3. **Les textes** : `reconciliation-split-balance-incomplete` (quatre locales, `{ $expected }` seul) ;
   `reconciliation-split-error-imbalance-detail` récrit (quatre locales, deux variables, phrase distincte de
   l'indicateur) ; « sans signe ni espace » (quatre locales) ; replis égaux au fr-CH ; `split-i18n.test.ts`,
   `i18n-entrees-a-variables.test.ts`, `sitesTotal` 3046.
4. **Les tests neufs** : passent-ils pour la bonne raison ? (`V-f` crédit, montant illisible, nul ; suppression d'une
   ligne — l'appariement compte/montant après `removeLine` ; ` 70`, `70 `, `1 200.50`, seize chiffres ; `1’200.50` ;
   quinze chiffres). R0 (d) tape-t-il bien U+2019 (`cat -A` ou `od -c` sur la ligne) ? `griseReproduit` (A4) : la somme
   en centimes est-elle juste pour `1'200.50`, `1’200.50`, `70,00` ? R0 (g0) et (g390) : assertions qui peuvent rougir ?
5. **Les mutations** : M10, M11, M12, M7-ter (redéfinie), M7-quinquies tuées ; M7-bis, M7-bis-b déclarées équivalentes
   (C-36-2-20) — l'argument tient-il ? M7-quater : `w-px` bien retiré du code (`git grep -n "w-px" --
   frontend/src/lib/features/reconciliation`) ?
6. **Le manuel** (`docs/manual/fr/user-manual.tex`, § *Éclatement*) contre le code remédié : zéros au-delà admis
   (`60.0000`), « sans signe ni espace », message qui dit la somme à atteindre et l'écart actuel, « écart non
   calculable ».
7. **Les comptes rendus** : A1 (`closes` au Record), A2 (en-tête de la spec), A3 (pièces : `remise-a-neuf-rem1.log`,
   `check-avertissements-rem1.log` — y lire qu'aucun avertissement ne vise les fichiers de la story), A5 (verdict H2) ;
   chaque chiffre du Record remédié retrouvé à son journal et à son commit.

## Rapport

Un fichier : `/home/gcorbaz/devel/kesh-gate-logs/e36-2/review-p2-ciblee.md`. Findings numérotés (`P2-1`…), sévérité
(CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, preuve, remède ; **la liste des axes
exercés ET non exercés** ; le décompte final recompté depuis la liste.
