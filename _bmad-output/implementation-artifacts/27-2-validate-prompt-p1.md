# Prompt — validation P1 de la spec, Story 27-2 (la devise d'un relevé CSV est lue)

*Versionné le 2026-10-12. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-27-2`, branche `story/27-2-devise-import-csv` (base `a2743fd4` = `origin/main`,
v0.13.1 + rétrospective de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/27-2-devise-import-csv.md` (commit `1d6fc3a7`).
Issue #624 (`gh api repos/guycorbaz/kesh/issues/624` et `…/624/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-27-choix-autonomes-27-2.md` (C-27-2-1 à 10). Précédent direct :
`26-1-import-csv-ubs-natif.md` (même parseur, même écran, mêmes gardes i18n).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy, P1 à P8 ;
§ Issue Tracking Rule).

⚠️ La fiche **décide qu'aucune migration ni aucun bump de `kesh_version_min_required` n'est nécessaire** (C-27-2-1),
à l'inverse du précédent `debit_sign` de la 26-1, en s'appuyant sur le `deny_unknown_fields` de `ColumnMapping` dans
les binaires publiés. C'est la décision la plus chère à se tromper : vérifie-la toi-même avant d'en tirer quoi que ce
soit.

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos des sites non résolus** (fiche, § *Inventaire*) : refais les `grep` copiés, et **élargis-les**
   (un détecteur part du symptôme, pas d'une liste) — `CHF` sous toutes graphies (`"CHF"`, `'CHF'`, `CHF` dans les
   `.svelte`/`.ts`, les `.ftl`, les manuels), tout lecteur ou écrivain de `currency` / `Currency` / `ccy` sur le
   chemin CSV (parseur, `kesh-core`, routes `bank_imports` et `bank_profiles`, `kesh-db` `bank_transactions`,
   `bank_imports`, `bank_profiles`, export global, `.keshbackup`, `post_restore.rs`, `kesh-seed`,
   `test_endpoints.rs`, frontend `bank-import/`). Un site qui **fixe ou suppose** une devise sans figurer à
   l'inventaire est au moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Import CSV multi-encodage*
   (l. 1721-1807 : *Profils banque*, la `keshnote` l. 1753 « La colonne Devise n'est pas lue », *Le relevé CSV
   d'UBS* et son tableau l. 1761-1777), § *Import dans Kesh* (l. 1690-1719, dont « Une devise non supportée bloque
   l'import » l. 1717), § *Rejet partiel* (l. 1824-1828), § *Réconciliation automatique* (l. 1845, « en CHF »). Ce que
   le manuel dit **aujourd'hui** est-il vrai au code ? Ce que l'AC11 prescrit d'écrire sera-t-il vrai au code que la
   fiche prescrit ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '`. Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 19 AC, tâches T0-T8, mutations M1-M17,
   les « 29 littéraux `ColumnMapping { … }` » (`grep -rn "ColumnMapping {" crates`), les numéros de ligne cités
   (`sed -n`), `sitesTotal: 2032` (`i18n-keys.test.ts:575`), les comptes de familles dynamiques (`:628`, `:680`).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #624 (« colonne `Devise` facultative », « refus si une ligne n'est pas en CHF », « à défaut,
  documenter ») couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** — et
  chaque mutation M1-M17 par un test **qui la tue réellement** (lis le test prescrit, pas son nom) ; chaque fait,
  chemin et numéro de ligne cité par la fiche ou le registre **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits
  (quatre locales, replis Svelte, manuel, CHANGELOG) confrontés à ce que le code prescrit fera réellement ; les gardes
  existantes qui changeront de sens (`error.rs` `all_is_complete`, `i18n-keys.test.ts` et ses familles dynamiques,
  `i18n-un-repli-par-cle.test.ts` / `MOTIFS_DYNAMIQUES`, `lint-i18n-ownership`, `textes_coherents.rs`,
  `test_schema_guard.rs`, `exports/csv_tables.rs` et son test d'en-têtes, `csv-line-error-label.test.ts`) — la fiche
  les nomme-t-elle toutes ? La fixture `ubs_synthetique.csv` porte-t-elle bien `CHF` en colonne 4 sur les onze lignes,
  et la substitution de `;CHF;` prescrite (C-27-2-9) ne touche-t-elle que ce qu'elle doit ?
- **F — Faisabilité et chemins de la décision (adversaire de la conception)** :
  - **Le retour arrière** (C-27-2-1) : un profil enregistré par la 0.13.2 **avec** `currency`, relu par une 0.13.1 et
    une 0.13.0 — `git show v0.13.1:crates/kesh-import/src/csv/profile.rs`, `git show v0.13.1:crates/kesh-db/src/entities/bank_profile.rs`,
    `git show v0.13.1:crates/kesh-api/src/routes/bank_profiles.rs`, `…/bank_imports.rs` (`resolve_csv_profile`,
    `list_available_profiles`, `db_profile_to_csv_profile`). Y a-t-il **un** chemin de la 0.13.1 qui lise ce JSON
    **sans** `ColumnMapping` typé, ou qui l'ignore, et importerait alors en CHF en silence ? L'import d'une
    sauvegarde d'installation 0.13.2 dans une 0.13.1 (manifeste, `min_required`) ? Un profil **sans** `currency`
    est-il vraiment sérialisé à l'identique (et `CAST(column_mapping AS CHAR)` de MariaDB, l'ordre des clés, la
    comparaison `existing.column_mapping_json != new_profile.column_mapping_json` de `repositories/bank_profiles.rs:264`) ?
    Le raisonnement « bruyant, donc hors P1 » tient-il contre le texte de la § Migration breaking policy ? Le front
    0.13.1 qui renverrait un `PUT` sans la clé ?
  - **L'ordre des refus** (fiche, *Points de vigilance*) : est-il réalisable tel quel dans `parse_csv_collect_inner`
    et `create_csv` (étapes, `RowAccumulator`, branche `PartialFailure`, reprise `EncodingMismatch` à l'aperçu et à
    l'import) ? Le contrôle de devise **avant** `confirm_partial_import` : y a-t-il un cas où il refuse à tort (zéro
    ligne valide → `DEFAULT_CSV_CURRENCY`) ou laisse passer (mélange parmi les lignes **invalides** seulement) ?
  - **Le mélange calculé sur les seules lignes valides** (AC3) : un fichier dont toutes les lignes `EUR` sont
    invalides pour une autre raison, confirmé en partiel, importe-t-il des CHF exacts — ou cache-t-il quelque chose ?
  - **CAMT non touché** (C-27-2-3, C-27-2-10) : l'hypothèse sur `camt053/mod.rs:168-178` (devise de transaction prise
    sur tout `<Amt Ccy>` sous `<TxDtls>`, montant pris sur `TxDtls/Amt` seul, `:427`) est-elle exacte au code ? Si oui,
    son classement « hors périmètre, issue proposée » est-il défendable pour une story P1 « devise » ?
  - **`kesh-import` publiable** : zéro dépendance interne préservée ? `DEFAULT_CSV_CURRENCY` dans un crate censé ignorer
    le périmètre CHF (doc de `validate_currency_supported_v0_1`, `kesh-core/src/bank_imports.rs:456-461`) — contradiction
    ou supposition honnête ?
  - **L'écran** : un `<input type="number">` vidé, en Svelte 5 avec `bind:value`, rend-il `null`, `undefined` ou
    `''` — et `currencyIdx ?? undefined` envoie-t-il alors une clé absente, `null` (refusé par serde ? accepté en
    `None` ?) ou `""` (422) ? Patron identique pour `reference` / `details` : un défaut **existant** révélé ici est un
    finding à part entière.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/27-2-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo` (y compris `cargo publish --dry-run` et `cargo test`), `npm`, `npx`,
`node`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, toute requête SQL. Rien de `~/kesh-test/` (relevé
réel de Guy) ne se lit ni ne se cite — et le test `crates/kesh-import/tests/ubs_fichier_reel.rs` ne se lance pas.
Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la
sortie standard.
