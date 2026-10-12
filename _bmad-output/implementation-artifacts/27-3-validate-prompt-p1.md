# Prompt — validation P1 de la spec, Story 27-3 (changer de profil après l'aperçu)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-27-3`, branche `story/27-3-profil-apres-apercu` (base `a2743fd4` = v0.13.1 +
rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-3-profil-apres-apercu.md` (commit `19b51ea3`).
Issue #626 (`gh api repos/guycorbaz/kesh/issues/626` et `…/626/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-3.md` (C-27-3-1 à 19) ; choix d'origine du
contournement : `epic-26-choix-autonomes.md` C-26-3-15.
Story voisine **non mergée**, à lire en lecture seule pour les recouvrements : `/home/gcorbaz/devel/kesh-27-1`
(branche `story/27-1-supprimer-import`, `git -C /home/gcorbaz/devel/kesh-27-1 diff origin/main --stat`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Test Locally First ; § Issue Tracking
Rule).

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos des chemins vers la confirmation** (fiche, tableau D1-D14) : refais-le **depuis le code**, pas
   depuis la liste — `frontend/src/lib/features/bank-import/BankImportUpload.svelte` en entier (toute affectation de
   `preview`, `selectedFile`, `selectedAccountId`, `bankProfileId`, `isLoading`, tout appel à `previewBankImport` /
   `createBankImport`), `BankProfileSelector.svelte`, `routes/(app)/bank-import/+page.svelte`, et côté serveur toute
   lecture de `bank_profiles` ou de `bank_accounts` sur le chemin `preview` / `create` / `preview_csv` / `create_csv`
   (`crates/kesh-api/src/routes/bank_imports.rs`). Un chemin qui mène à une confirmation avec un mapping non vu et
   que le tableau omet est au moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Import dans Kesh* (l. 1690 et
   suivantes), § *Import avec un profil existant* (l. 1790-1808, dont l'encadré `keshwarning`), et les phrases
   « redéposez le fichier » (l. 1743, l. 1784) que la fiche déclare **inchangées** (C-27-3-14) : le sont-elles
   vraiment au code que la fiche prescrit ? Ce que l'AC12 prescrit d'écrire sera-t-il vrai ? Contrôle aussi le
   **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Grepe le **symptôme**
   (« avant de déposer », « redépos », « ne la recalcule », « Annuler, choisir ») sur tout le dépôt — manuels, i18n
   des quatre locales, messages serveur, `docs/`, `website/`, `README.md`. Une promesse que le code ne tiendrait pas
   est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 23 sites `i18nMsg(` dans
   `BankImportUpload.svelte`, `sitesTotal` 2032 et le delta +2 (`grep -o "i18nMsg(" <fichier>`), le nombre d'AC
   (16), de tests (U4 / I10 / V11 / E2), de mutations (16), de choix (19), de champs de `BankProfile` (11 hors
   horodatages), les numéros de ligne cités.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #626 (constat, attendu, contournement) couverte par un AC testable, chaque AC par une
  tâche, chaque AC par un test **qui rougirait** ; chaque mutation M1-M16 tuée par le test nommé (sinon, laquelle
  survit et pourquoi — M6 en particulier, dont la fiche admet un repli) ; chaque fait, chemin et numéro de ligne cité
  **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (trois clés i18n, manuel, CHANGELOG,
  `docs/api-external.md`) confrontés à ce que le code prescrit fera réellement ; les gardes existantes qui
  changeront de sens ou rougiront (`frontend/src/lib/shared/i18n-keys.test.ts`, `lint-i18n-ownership`, la parité des
  quatre `.ftl`, le moissonneur des replis `i18nMsg`, `crates/kesh-api/tests/textes_coherents.rs`,
  `crates/kesh-api/tests/audit_route_registry.rs`, les E2E existants `bank-import*.spec.ts`,
  `bank-csv-import.spec.ts`) — la fiche les nomme-t-elle toutes ? Le tableau des fichiers communs avec la 27-1 est-il
  exact et complet (compare au `diff --stat` de la 27-1) ?
- **F — Faisabilité et défense serveur (adversaire de la conception)** : l'empreinte (AC6-AC8, C-27-3-7 à 10) ferme-t-
  elle **D1 et D3** sans faux positif ? Cherche une entrée qui **change l'aperçu sans changer l'empreinte** (faux
  négatif : ce qui, dans `preview_csv` / `parse_csv_collect` / `select_statement_by_iban`, dépend d'autre chose que
  société, compte, IBAN, hash du fichier, format, profil — le nom du fichier ? le `content_type` ? l'encodage forcé
  par `confirmEncodingMismatch` ?), et une qui **change l'empreinte sans changer l'aperçu** (faux positif : un import
  légitime refusé — preview et création résolvent-elles le **même** profil, dans tous les cas d'auto-détection, de
  correspondances multiples `bank_csv_multiple_profile_matches`, de profil explicite ?). La comparaison dans
  `create_csv` après `resolve_csv_profile` sur la transaction est-elle sans fenêtre (REPEATABLE READ, le profil
  comparé est-il celui qui est parsé) ? La précédence prescrite (AC8, I10) est-elle cohérente avec l'ordre réel des
  refus de `create` et `create_csv` ? Côté écran : la machine d'état prescrite (séquence, `isPreviewing` /
  `isConfirming`, instantané, branche `409`) est-elle réalisable en Svelte 5 sans boucle (`$effect` du compte,
  `bind:value`, `value` + `onChange` du sélecteur) ; le cas `reset()` → `$effect` → … ; une réponse d'aperçu tardive
  après un succès d'import ; la désactivation des sélecteurs pendant la confirmation ; la stabilité des tests V3,
  V8, V10 (promesses retenues, ordre des résolutions) et des E2E E1/E2 (fixture dérivée en mémoire : l'année
  remplacée apparaît-elle ailleurs ? les doublons de lignes avec le test 26-1 sont-ils vraiment évités ?
  `PUT /api/v1/bank-profiles/{id}` exige-t-il un corps complet ?). La compatibilité des clés d'API (C-27-3-9) tient-
  elle contre § 4 et § 7 de `docs/api-external.md` ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-3-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, `docker`, `bash` sur un script du dépôt,
`gh` en écriture, toute requête SQL. Rien ne s'écrit dans `/home/gcorbaz/devel/kesh-27-1`. Rien de `~/kesh-test/`
(relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
