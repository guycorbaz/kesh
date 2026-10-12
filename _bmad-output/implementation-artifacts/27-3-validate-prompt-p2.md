# Prompt — validation P2 de la spec, Story 27-3 (changer de profil après l'aperçu)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était **Sonnet** ; P2 est
une passe **complète**, **Opus** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu` (base `a2743fd4` = v0.13.1 +
rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` (remédiation P1 : commit `c39effde` ;
`git show c39effde` pour le diff de la remédiation, `git show 19b51ea3:<fiche>` pour l'état relu en P1).
Issue #626 (`gh api repos/guycorbaz/kesh/issues/626` et `…/626/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md` (C-27-3-1 à 27 ; C-27-3-20 à 27 viennent de
la remédiation P1, C-27-3-19 est révisé par C-27-3-25) ; choix d'origine du contournement :
`epic-26-choix-autonomes.md` C-26-3-15.
Rapports P1 (à lire pour savoir ce qui a déjà été relevé, **pas** pour en reprendre les conclusions) :
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p1-R.md` et `27-3-validate-p1-F.md` ; décisions au Change Log de
la fiche, § « Validation P1 ».
Story voisine **non mergée**, à lire en lecture seule pour les recouvrements : `/home/gcorbaz/devel/kesh-27-1`
(branche `story/27-1-supprimer-import`, `git -C /home/gcorbaz/devel/kesh-27-1 diff origin/main --stat`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Test Locally First ; § Issue Tracking
Rule).

## Axes prioritaires (communs aux deux lentilles)

0. **Régressions introduites par la remédiation P1 — axe premier, à exercer en entier.** *La sévérité se déplace
   vers ce qu'on vient d'écrire* (CLAUDE.md, § passe ciblée). Relis `git show c39effde` ligne à ligne et, pour chaque
   ajout, demande : contredit-il une autre phrase de la fiche ou du registre ? est-il vrai au code de la base ?
   a-t-il un test qui rougirait ? En particulier :
   - **AC3 / C-27-3-20** : `reset()` qui éteint `isPreviewing` et `isConfirming` — `reset()` est appelée au **succès**
     de l'import (dans `handleConfirm`, avant son `finally`) et par le `$effect` du compte : un ordre d'exécution
     rallume-t-il un état ? l'avis effacé par `reset()` survit-il à la branche `409` (qui ne passe pas par `reset()`) ?
   - **C-27-3-21** : *Annuler* désactivé pendant la confirmation — reste-t-il un geste qui invalide une confirmation
     en vol (navigation, démontage du composant, dépôt par glisser sur une zone masquée) ? D11 « inatteignable » est-il
     vrai **après** la story (l'AC2 masque-t-elle vraiment tout ce qui rendrait *Annuler*) ?
   - **V10 / M6** : le test CAMT tue-t-il M6 tel que la mutation est écrite ? l'AC4 (présélection) ou l'AC1 (relance
     au changement de profil) le rendent-ils instable ?
   - **I11, I12, M16a, M16b** : I11 distingue-t-il vraiment M16a de l'implémentation prescrite (que rend le code de
     base quand le profil demandé est supprimé — `resolve_csv_profile`, `bank_imports.rs:716-724`) ? M16b est-il tué
     par I6 tel qu'I6 est écrit ? La variante déclarée « équivalente » l'est-elle ?
   - **AC16 / V12 / M17** : `input.value = ''` avant `handleFileSelect` — le `File` lu reste-t-il valide ? l'espion du
     setter de `value` par `Object.defineProperty` est-il réalisable sur un `HTMLInputElement` de jsdom monté par
     `@testing-library/svelte`, et l'élément espionné est-il celui que l'AC2 démonte aussitôt ? Le défaut D16 est-il
     réel à la base (raisonné, non exécuté en P1) ?
   - **AC17 / C-27-3-25** : l'inventaire H1-H11 est-il **clos** — refais-le depuis le code (`BankImportUpload.svelte`,
     `BankImportList.svelte`, `BankProfileSelector.svelte`, `routes/(app)/bank-import/+page.svelte` : tout nœud de
     texte, tout attribut affiché `title` / `aria-label` / `placeholder`, tout littéral affecté à `errorMessage` ou
     affiché) ; les non-résolus déclarés le sont-ils à juste titre ; les numéros de ligne cités sont-ils exacts ; les
     21 clés neuves existent-elles déjà ailleurs (`grep` des quatre `.ftl`, et de ceux de la 27-1) ; la réutilisation
     de `common-loading` et `error-unexpected` respecte-t-elle `lint-i18n-ownership` (`GLOBAL_NAMESPACES`) et la garde
     « un repli par clé » (`i18n-un-repli-par-cle.test.ts`) ; les traductions de/it/en sont-elles justes et cohérentes
     avec les clés voisines des catalogues ; la clé partagée `bank-import-labels-period` est-elle légitime ; V13 tient-
     il dans un seul fichier de test avec les mocks de V1-V12 ?
   - **AC11** : recompte `sitesTotal` (2032 → 2063, +31 ; 2072 à la fusion) **depuis la source** :
     `grep -o "i18nMsg(" <fichier> | wc -l` sur les trois fichiers, et le décompte des sites H1-H11 de la table.
   - **AC10 / T1** : l'entrée `("bank-import-errors-preview-stale", 1, &[])` a-t-elle la forme attendue par la
     `TABLE` de `les_replis_rust_suivent_le_catalogue` (`crates/kesh-api/tests/textes_coherents.rs:679-735`) ?
   - **AC8 / C-27-3-24** : l'asymétrie CSV / CAMT décrite est-elle l'ordre réel de `create` (`:1105-1117`) et de
     `create_csv` (`:1781-1809`) ? I10 étendu au CAMT est-il réalisable (fixture CAMT déjà importée + empreinte d'un
     autre fichier) ?
   - **E1 / E2** : motif vide (`pattern: ''` → `null`) compatible avec `createProfileViaUi` et `BankProfileForm.svelte` ;
     corps du `PUT` reconstruit conforme à `CreateOrUpdateBankProfileRequest` (`bank_profiles.rs:86-100`).
   Un défaut **introduit** par la remédiation se signale comme tel (« régression P1 ») ; il compte au même titre qu'un
   défaut d'origine.
1. **Inventaire clos des chemins vers la confirmation** (fiche, tableau D1-D17) : refais-le **depuis le code**, pas
   depuis la liste — `frontend/src/lib/features/bank-import/BankImportUpload.svelte` en entier (toute affectation de
   `preview`, `selectedFile`, `selectedAccountId`, `bankProfileId`, `isLoading`, tout appel à `previewBankImport` /
   `createBankImport`), `BankProfileSelector.svelte`, `routes/(app)/bank-import/+page.svelte`, et côté serveur toute
   lecture de `bank_profiles` ou de `bank_accounts` sur le chemin `preview` / `create` / `preview_csv` / `create_csv`
   (`crates/kesh-api/src/routes/bank_imports.rs`). Un chemin qui mène à une confirmation avec un mapping non vu et
   que le tableau omet est au moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Import dans Kesh* (l. 1690 et
   suivantes), § *Import avec un profil existant* (l. 1790-1808, dont l'encadré `keshwarning`), et les phrases
   « redéposez le fichier » (l. 1743, l. 1784) que la fiche déclare **inchangées** (C-27-3-14) et que l'AC16 rend
   désormais tenables par « parcourir » : le sont-elles au code prescrit ? Ce que l'AC12 prescrit d'écrire sera-t-il
   vrai, y compris avec *Annuler* désactivé pendant la confirmation ? Les libellés traduits de l'AC17 sont-ils cités
   par le manuel (une citation qui deviendrait fausse en de/it/en n'est pas un défaut — le manuel est en français —,
   mais une citation fr qui ne correspondrait plus à l'écran l'est) ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Grepe le **symptôme** (« avant de
   déposer », « redépos », « ne la recalcule », « Annuler, choisir ») sur tout le dépôt — manuels, i18n des quatre
   locales, messages serveur, `docs/`, `website/`, `README.md`. Une promesse que le code ne tiendrait pas est au
   moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche et du registre se recompte depuis la source — 18 AC (AC1-AC17 +
   AC1-bis), tests U1-U4 / I1-I12 / V1-V13 / E1-E2 (31), 20 mutations (M1-M15, M16a, M16b, M17-M19), D1-D17, 27 choix,
   24 clés neuves (3 + 21), sites par fichier (23 → 49, 1 → 5, 3 → 4), `sitesTotal` 2063 / 2072, 11 champs de
   `BankProfile` hors horodatages, les numéros de ligne cités (dont ceux, neufs, de l'AC17 et des D15-D17). Un total
   doit être cohérent avec sa propre ventilation, dans la fiche **et** dans le Change Log « Validation P1 ».

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #626 (constat, attendu, contournement) couverte par un AC testable, chaque AC (dont AC16,
  AC17) par une tâche, chaque AC par un test **qui rougirait** ; chaque mutation M1-M19 tuée par le test nommé
  (sinon, laquelle survit et pourquoi — les neuves M16a, M16b, M17, M18, M19 et la réaffectée M6 en particulier) ;
  chaque fait, chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (24 clés
  i18n dans les quatre locales, manuel, CHANGELOG, `docs/api-external.md`) confrontés à ce que le code prescrit
  fera réellement ; les gardes existantes qui changeront de sens ou rougiront (`frontend/src/lib/shared/i18n-keys.test.ts`,
  `i18n-un-repli-par-cle.test.ts`, `i18n-repli-divergent-actif.test.ts`, `i18n-libelle-en-dur.test.ts`,
  `lint-i18n-ownership`, la parité des quatre `.ftl` — `crates/kesh-i18n/src/loader.rs::parity_between_locales` —,
  le moissonneur des replis `i18nMsg`, `crates/kesh-api/tests/textes_coherents.rs`,
  `crates/kesh-api/tests/audit_route_registry.rs`, les E2E existants `bank-import*.spec.ts`,
  `bank-csv-import.spec.ts`) — la fiche les nomme-t-elle toutes ? Le tableau des fichiers communs avec la 27-1
  (dont les lignes neuves `textes_coherents.rs`, `loader.rs`, et les fichiers neufs de l'AC17) est-il exact et
  complet (compare au `diff --stat` de la 27-1, hunks relus) ?
- **F — Faisabilité et défense serveur (adversaire de la conception)** : l'empreinte (AC6-AC8, C-27-3-7 à 10) ferme-t-
  elle **D1 et D3** sans faux positif ? Cherche une entrée qui **change l'aperçu sans changer l'empreinte** (faux
  négatif : ce qui, dans `preview_csv` / `parse_csv_collect` / `select_statement_by_iban`, dépend d'autre chose que
  société, compte, IBAN, hash du fichier, format, profil) et une qui **change l'empreinte sans changer l'aperçu**
  (faux positif : un import légitime refusé — preview et création résolvent-elles le **même** profil, dans tous les
  cas d'auto-détection, de correspondances multiples, de profil explicite ; D17 est-il bien le seul cas côté API ?).
  La comparaison dans `create_csv` après `resolve_csv_profile` sur la transaction est-elle sans fenêtre ? La
  précédence prescrite (AC8, I10-I12) est-elle l'ordre réel des refus ? Côté écran : la machine d'état **révisée**
  (séquence non réactive, `isPreviewing` / `isConfirming` éteints par `reset()`, *Annuler* désactivé pendant la
  confirmation, instantané, branche `409` et avis hors du bloc `{#if preview}`, `input.value` vidé) est-elle
  réalisable en Svelte 5 sans boucle (`$effect` du compte, `bind:value`, `value` + `onChange` du sélecteur) ; une
  réponse d'aperçu tardive après un succès d'import ; la stabilité des tests V3, V8, V9, V10, V12, V13 (promesses
  retenues, ordre des résolutions, mock d'`i18nMsg` basculable) et des E2E E1/E2.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; marque « régression P1 » tout défaut
introduit par `c39effde`. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste
des axes exercés ET non exercés** (l'axe 0 sous-point par sous-point) — un « 0 finding » sans elle ne compte pas
comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt,
`gh` en écriture, toute requête SQL. Rien ne s'écrit dans `/home/gcorbaz/devel/kesh-27-1`. Rien de `~/kesh-test/`
(relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
