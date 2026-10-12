# Prompt — validation P3 de la spec, Story 27-3 (changer de profil après l'aperçu)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était **Sonnet**, P2 **Opus** ;
P3 est une passe **complète**, **Sonnet** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu` (base `a2743fd4` = v0.13.1 +
rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` (remédiation P2 : commit `3f5a3c15` ;
`git show 3f5a3c15` pour le diff de la remédiation ; `git show c39effde` pour celle de la P1 ;
`git show 19b51ea3:<fiche>` pour l'état d'origine).
Issue #626 (`gh api repos/guycorbaz/kesh/issues/626` et `…/626/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md` (C-27-3-1 à 34 ; C-27-3-28 à 34 viennent de
la remédiation P2 ; C-27-3-7, 21 et 23 y sont amendés) ; choix d'origine du contournement :
`epic-26-choix-autonomes.md` C-26-3-15.
Rapports antérieurs (à lire pour savoir ce qui a déjà été relevé, **pas** pour en reprendre les conclusions) :
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p1-{R,F}.md` et `27-3-validate-p2-{R,F}.md` ; décisions au Change
Log de la fiche, § « Validation P1 » et § « Validation P2 » (dont le constat du signal de découpage D5).
Story voisine **non mergée**, à lire en lecture seule pour les recouvrements : `/home/gcorbaz/devel/kesh-27-1`
(branche `story/27-1-supprimer-import`, `git -C /home/gcorbaz/devel/kesh-27-1 diff origin/main --stat`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif et son
amendement D5 ; § Test Locally First ; § Issue Tracking Rule).

## Axes prioritaires (communs aux deux lentilles)

0. **Régressions introduites par la remédiation P2 — axe premier, à exercer en entier.** *La sévérité se déplace
   vers ce qu'on vient d'écrire* (CLAUDE.md, § passe ciblée) : en P2, deux défauts marqués « régression P1 » étaient
   des lignes que la remédiation P1 avait **réécrites sans voir le défaut**. Relis `git show 3f5a3c15` ligne à ligne
   et, pour chaque ajout, demande : contredit-il une autre phrase de la fiche ou du registre ? est-il vrai au code de
   la base ? a-t-il un test qui rougirait, et la mutation nommée est-elle **tuée par le test nommé tel qu'il est
   écrit** ? En particulier :
   - **AC9 / AC10 / C-27-3-28 — avis neutre** : le nouveau texte (« Le profil, le fichier ou le compte a changé… »)
     est-il vrai dans **tous** les cas où l'écran le montre ? Le texte de recalcul sans « avec le profil choisi » reste-
     t-il juste pour l'AC1 ? Les traductions de/it/en sont-elles correctes et cohérentes avec
     `bank-import-errors-preview-stale` ? Le manuel prescrit (AC12) dit-il désormais vrai, sans promettre que l'écran
     nomme la cause ? Reste-t-il, où que ce soit (fiche, registre, manuel, CHANGELOG prescrit, `docs/api-external.md`
     prescrit), une phrase qui impute le `409` au seul profil ?
   - **I6-bis / M16b / C-27-3-29** : I6-bis est-il réalisable avec `build_multipart_with_text_fields`
     (`crates/kesh-api/tests/bank_imports_e2e.rs:677`) — un profil à motif, aucun `bankProfileId`, même nom de
     fichier — et rend-il `201` au code prescrit ? M16b telle que réécrite y rend-elle `409` ? Une **autre** lecture de
     M16b survit-elle ?
   - **V3 (ordre A, C, B) / V8 variante rejetée / M21 / M22 / C-27-3-30** : chaque mutation est-elle tuée par
     l'assertion nommée, avec des promesses retenues réalisables en Vitest ? M3 reste-t-elle tuée par V3 et V8 ?
   - **V6-bis / M20 / C-27-3-31** : les trois sites d'effacement de l'avis sont-ils cohérents avec le reste de T3 et
     de l'AC9 (l'avis doit **survivre** au recalcul `'stale'` qu'il annonce, y compris quand ce recalcul échoue —
     variante de V6, E2) ? (b) « reconfirmer » est-il atteignable après le recalcul (cases, `disabled`) ?
   - **AC3 / AC9 / C-27-3-33 — `isConfirming` éteint avant `void runPreview`** : un ordre d'exécution rallume-t-il
     un état, ou laisse-t-il une fenêtre où *Confirmer* est cliquable sur un aperçu retiré ? M24 est-elle tuée par
     V6 ?
   - **AC4 / C-27-3-32 — annotation conditionnée à `autoMatched`** : vrai au code (`BankProfileSelector.svelte:53-55`,
     `BankImportUpload.svelte:252`, `bank_imports.rs:1688-1693`) ? V1 et V4 tuent-ils M23 ? La présélection KF #70
     reste-t-elle intacte ? Le manuel (`user-manual.tex:1794-1796`) reste-t-il vrai ?
   - **D18 / D19 / C-27-3-34, D13 et D17 élargis** : vrais au code, et l'inventaire D1-D19 est-il désormais clos
     (axe 1) ?
   - **AC16 / V12 sans ordre** : la phrase de l'AC16 et celle de C-27-3-26 se contredisent-elles encore ?
   - **AC11 commentaires masqués**, **AC10 gardes nommées**, **AC13 ligne du § 10**, **T1 `normalize_iban`
     `pub(crate)`**, **tableau des fichiers communs** (`BankImportList.svelte`, `textes_coherents.rs`) : exacts au code
     et au `diff --stat` actuel de la 27-1 ?
   - **Change Log « Validation P2 »** : les décomptes (24 findings, 4 doublons, 20 distincts, 4 au-dessus de LOW, 16
     LOW) et le **constat D5** (deux « régressions P1 » qui figuraient déjà à `19b51ea3`) sont-ils vrais ? Vérifie-le
     contre `git show 19b51ea3:…`.
   Un défaut **introduit** par la remédiation P2 se signale comme tel (« régression P2 ») ; il compte au même titre
   qu'un défaut d'origine.
1. **Inventaire clos des chemins vers la confirmation** (fiche, tableau D1-D19) : refais-le **depuis le code**, pas
   depuis la liste — `frontend/src/lib/features/bank-import/BankImportUpload.svelte` en entier (toute affectation de
   `preview`, `selectedFile`, `selectedAccountId`, `bankProfileId`, `isLoading`, tout appel à `previewBankImport` /
   `createBankImport`), `BankProfileSelector.svelte`, `routes/(app)/bank-import/+page.svelte`, et côté serveur toute
   lecture de `bank_profiles` ou de `bank_accounts` sur le chemin `preview` / `create` / `preview_csv` / `create_csv`
   (`crates/kesh-api/src/routes/bank_imports.rs`). Un chemin qui mène à une confirmation avec un mapping non vu et
   que le tableau omet est au moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Import dans Kesh* (l. 1690 et
   suivantes), § *Import avec un profil existant* (l. 1790-1808, dont l'encadré `keshwarning`), les phrases
   « redéposez le fichier » (l. 1743, l. 1784) et « corriger le fichier source » (l. 1826). Ce que l'AC12 prescrit
   d'écrire sera-t-il vrai au code prescrit, avec l'avis neutre, *Annuler* désactivé pendant la confirmation et
   l'annotation conditionnée ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr
   '\n' ' ' | tr -s ' '` — et vérifie que les contrôles positifs et négatifs prescrits par l'AC12 sont
   **discriminants** (faux à la base, vrais après). Grepe le **symptôme** (« avant de déposer », « redépos », « ne la
   recalcule », « Annuler, choisir », « Le profil a été modifié », « avec le profil choisi ») sur tout le dépôt —
   manuels, i18n des quatre locales, messages serveur, `docs/`, `website/`, `README.md`. Une promesse que le code ne
   tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche et du registre se recompte depuis la source — 18 AC (AC1-AC17 +
   AC1-bis), tests U1-U4 / I1-I12 + I6-bis / V1-V13 + V6-bis / E1-E2 (33), 25 mutations (M1-M15, M16a, M16b,
   M17-M24), D1-D19, 34 choix, 24 clés neuves (3 + 21), sites par fichier (23 → 49, 1 → 5, 3 → 4, comptés commentaires
   masqués), `sitesTotal` 2063 / 2072, 11 champs de `BankProfile` hors horodatages, les numéros de ligne cités (dont
   les neufs : `:677`, `:815`, `:1574`, `:55-104`, `:1720`, `:1826`, `loader.rs:733`, `i18n-keys.test.ts:32`). Un
   total doit être cohérent avec sa propre ventilation, dans la fiche **et** dans les Change Logs P1 et P2.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #626 (constat, attendu, contournement) couverte par un AC testable, chaque AC (dont AC16,
  AC17) par une tâche, chaque AC par un test **qui rougirait** ; chaque mutation M1-M24 tuée par le test nommé
  (sinon, laquelle survit et pourquoi — les neuves M20-M24 et la réaffectée M16b en particulier) ; chaque fait,
  chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (24 clés i18n dans les
  quatre locales, manuel, CHANGELOG, `docs/api-external.md` § 7 et § 10) confrontés à ce que le code prescrit fera
  réellement ; les gardes existantes nommées à l'AC10 (et celles déclarées sans incidence) — le sont-elles à juste
  titre ? Le tableau des fichiers communs avec la 27-1 est-il exact et complet (compare au `diff --stat` actuel de la
  27-1, hunks relus) ?
- **F — Faisabilité et défense serveur (adversaire de la conception)** : l'empreinte (AC6-AC8, C-27-3-7 à 10) ferme-t-
  elle **D1, D3 et D18** sans faux positif ? Cherche une entrée qui **change l'aperçu sans changer l'empreinte**
  (faux négatif) et une qui **change l'empreinte sans changer l'aperçu** (faux positif : un import légitime refusé ;
  D17 élargi est-il bien le seul cas côté API ?). La précédence prescrite (AC8, I10-I12) est-elle l'ordre réel des
  refus ? Côté écran : la machine d'état **révisée deux fois** (séquence non réactive dont le `finally` est
  conditionné, réponses périmées en erreur ignorées, `isPreviewing` / `isConfirming` éteints par `reset()`,
  `isConfirming` éteint avant le recalcul `'stale'`, trois sites d'effacement de l'avis, *Annuler* désactivé pendant
  la confirmation, instantané, `autoMatchedId` conditionné, `input.value` vidé) est-elle réalisable en Svelte 5 sans
  boucle (`$effect` du compte, `bind:value`, `value` + `onChange` du sélecteur), et cohérente — un site d'effacement
  de l'avis qui l'efface au mauvais moment, une extinction qui laisse *Confirmer* actif sur un aperçu retiré ? La
  stabilité des tests V3, V6, V6-bis, V8, V9, V10, V12, V13 (promesses retenues, ordre des résolutions, mock
  d'`i18nMsg` basculable, plusieurs rendus dans un même test) et des E2E E1/E2.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; marque « régression P2 » tout défaut
introduit par `3f5a3c15` (et « régression P1 » ce qui vient de `c39effde`, **après** l'avoir vérifié contre
`19b51ea3`). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés
ET non exercés** (l'axe 0 sous-point par sous-point) — un « 0 finding » sans elle ne compte pas comme passe. Rapport
dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **`python3`** (et tout interpréteur :
`python`, `perl`, `ruby`), `docker`, `bash` sur un script du dépôt, `gh` en écriture, toute requête SQL. Rien ne
s'écrit dans `/home/gcorbaz/devel/kesh-27-1`. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite.
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `wc`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard (ou vers un fichier de `/home/gcorbaz/devel/kesh-gate-logs/`).
