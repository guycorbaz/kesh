# Prompt — revue de code P1, Story 27-2 (la devise d'un relevé CSV est lue, et le contrôle de devise joue)

*Versionné le 2026-10-12. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-27-2`, branche `story/27-2-devise-import-csv`. **Le diff à revoir** :
`git diff 80699a89 d6502211 -- . ':(exclude)_bmad-output'` (`80699a89` = validation close de la fiche ; `d6502211` =
dernier commit de code ; les commits suivants ne portent que la fiche, le registre, `sprint-status.yaml` et ce
prompt). ⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`, que le shell peut interpréter. Fiche :
`_bmad-output/implementation-artifacts/27-2-devise-import-csv.md` — AC1 à AC19 (dont AC1-bis, AC8-bis, AC12-bis),
inventaire des sites non résolus (a)-(i), Tâches, Dev Notes (ordre des refus, points de vigilance), et le **Dev Agent
Record** (choix C-27-2-1 à 31 au registre `epic-27-choix-autonomes-27-2.md`, mutations M1 à M25 et leurs variantes,
rouge d'AC19 avant correction, décomptes, gates). Règles : `CLAUDE.md` du dépôt — § « Recompter ses propres comptes
rendus », § « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Une passe qui ne
déclare pas ses axes exercés ne compte pas », § « Migration breaking policy » (aucune migration attendue, AC1-bis ;
lecture de P1 assumée par C-27-2-1). Issue : #624 (la PR la fermera, texte prescrit au § *Pour la PR* de l'AC12).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre **aucun** fichier hors du dépôt — en particulier rien sous
`~/kesh-test/`. Les fixtures (`crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`,
`crates/kesh-import/tests/fixtures/camt053/v04_amtdtls_foreign_currency.xml`) sont synthétiques. Le test
`crates/kesh-import/tests/ubs_fichier_reel.rs` (ignoré) se relit, il ne s'exécute pas.

**Contexte** : story **pleine pile**. `kesh-import` : `ColumnMapping.currency` (`deny_unknown_fields` figé par un test
et par un doc-comment), `DEFAULT_CSV_CURRENCY`, `MAX_REPORTED_CURRENCIES`, `CsvError::MixedCurrencies`,
`CsvLineErrorCode::InvalidCurrency`, lecture de la devise dans `parse_row` (après le montant, erreur qui garde
l'observation de débit), étape 7.c de `parse_csv_collect_inner` (après le refus de signe), `normalize_currency` ;
CAMT.053 : ancrage de la devise sur `path.last()` (`TxDtls` / `Ntry`) à l'ouverture d'un `Amt` ; fixture et tests
AC19 ; catalogue `tests/fixtures/README.md` ; `ubs_fichier_reel.rs` (colonne 4, bras `MixedCurrencies`). `kesh-core` :
textes « v0.1 » du refus. `kesh-api` : helper `unsupported_currency` (CAMT aperçu et import, CSV aperçu et import —
dans les deux bras, avant `confirm_partial_import`), information `bank_csv_currency_assumed`, bras explicite de
`map_csv_error`, `AppError::BankCsvMixedCurrencies` et sa réponse, repli de `BANK_IMPORT_UNSUPPORTED_CURRENCY`,
doc-module de `routes/bank_profiles.rs` ; garde G9 (`textes_coherents.rs`) ; tests AC14 (`bank_imports_e2e.rs`) et AC7
(`bank_profiles_e2e.rs`) ; parenthèse de `reconciliation_e2e.rs:2253`. `kesh-i18n` : cinq clés neuves et une modifiée ×
quatre catalogues. Frontend : `BankProfileForm.svelte` (champ, `OWNED_COLUMN_MAPPING_KEYS`, `keepUnownedKeys`),
`bank-profile.types.ts`, `csv-line-error-label.ts`, repli de `BankImportUpload.svelte:604`, Vitest (v1)-(v9),
`i18n-keys.test.ts` (familles, `sitesTotal` 2032 → 2034), E2E `bank-csv-import.spec.ts`. Textes : manuels utilisateur
et d'administration FR (`.tex` **et PDF**), `docs/i18n-glossaire.md`, `CHANGELOG.md` (`## [0.13.2] — Non publié`).

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **Le chemin de la devise, de bout en bout.** Inventaire des sites non résolus refait au `HEAD` : `grep -rnF '"CHF"'`
   sur `crates/kesh-import/src`, `crates/kesh-core/src/bank_imports.rs`, `crates/kesh-api/src/routes/bank_imports.rs`,
   `routes/bank_profiles.rs`, et les lecteurs `\.currency\b` — un site qui fixe ou suppose encore une devise sur le
   chemin CSV, hors `DEFAULT_CSV_CURRENCY` ? `db_profile_to_csv_profile` passe-t-il bien la clé ? Une supposition
   silencieuse ajoutée ailleurs (une cellule vide traitée comme CHF) ?
2. **L'ordre des refus** (Dev Notes, *Points de vigilance*) : fichier déjà importé → profil → erreurs fatales du
   parseur → refus de signe → mélange de devises → devise non supportée → rejet partiel → zéro ligne valide. Le
   contrôle est-il appelé **dans chacun** des deux bras de `create_csv`, **avant** `confirm_partial_import` ? Sur le
   relevé du second essai `EncodingMismatch` (aperçu) et de la reprise `confirm_encoding_mismatch` (import) ? Rien
   d'écrit sur un refus (transaction abandonnée) ?
3. **Le parseur** : forme de la devise (trois lettres **ASCII**, `s.len()` en octets), casse, erreur qui garde `debit`
   dans les **deux** bras (vide et illisible), invariant « chaque transaction valide porte la devise du relevé »,
   plafond des devises rapportées (10 premières du tri), refus de signe toujours prioritaire, zéro ligne valide.
4. **CAMT.053 (AC19)** : l'ancrage sur `path.last()` correspond-il exactement aux deux ancrages du montant
   (`handle_ntry_text` `Ntry/Amt`, `handle_txdtls_text` `TxDtls/Amt`) ? Un `Amt` de `<Bal>`, de `<Ntry>` sous un
   `<TxDtls>` ouvert, de `<NtryDtls><Btch>`, de `<AmtDtls><TxAmt>` ? Les replis `:603` / `:618` intacts ? L'angle mort
   assumé (`MissingRequiredField(currency)`) bien le seul effet de bord ?
5. **Le formulaire** : `OWNED_COLUMN_MAPPING_KEYS` seule source de vérité ; `keepUnownedKeys` typée sans `any` ni cast ;
   les deux sens du changement de mode ; un champ vidé → clé omise du JSON ; libellé et aide reliés (`for`,
   `aria-describedby`) ; un avertissement `svelte-check` neuf (le même motif que les quinze voisins) acceptable ?
6. **i18n** : cinq clés neuves × 4, une modifiée × 4, replis Svelte et Rust = texte fr-CH (G9 et
   `i18n-repli-divergent-actif.test.ts`), arguments Fluent (`{ $currencies }`) dans les quatre langues, terminologie du
   glossaire (`docs/i18n-glossaire.md`, ligne *devise*), apostrophe droite des textes neufs (C-27-2-21),
   `sitesTotal` recompté aux deux bornes, familles `bank-import-info-` (3) et `bank-import-csv-errors-` (7). La liste
   fermée des « v0.1 » gardés (AC10) : `grep -rnF "v0.1" crates --include=*.rs | grep -i "devise\|currency\|chf"` au
   `HEAD` égale-t-elle la sortie recopiée au Dev Agent Record ?
7. ⛔ **LES MANUELS — `docs/manual/fr/user-manual.tex`, `admin-manual.tex` ET leurs PDF.** La note *Profils banque*
   (colonne Devise lue ou supposée), la liste du formulaire, le tableau et les puces du *Relevé CSV d'UBS*, *Import dans
   Kesh* (« une devise non supportée bloque l'import »), *Rejet partiel* (exception non confirmable), et au manuel
   d'administration la section *Passer à la 0.13.2* (ajouter la colonne, transactions déjà importées, retour à la
   0.13.1) et la réserve du *Rollback en cas d'échec* disent-ils **vrai** sur le code (relire
   `BankProfileForm.svelte`, `BankImportUpload.svelte` (bouton *Confirmer* grisé), `routes/bank_imports.rs`,
   `csv/parser.rs`, `routes/reconciliation.rs` (gardes de devise `:631`, `:780`, `:2728`), la suppression d'import de la
   27-1 telle que la fiche la suppose) ? **Cherche une phrase des manuels, de `docs/api-external.md`, du README ou du
   site, hors des sections touchées, que la story rend fausse** (ex. : une phrase qui dit tout import CSV en CHF ; une
   énumération des refus non confirmables ; la liste des colonnes d'un profil ; une mention du préambule UBS).
   Contrôle les **PDF aplatis** (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire `.` à sa place dans un motif.
8. **CHANGELOG** : une seule section `## [0.13.2] — Non publié` ; patron de la 0.13.1 (bloc « À faire » numéroté,
   `### Modifié`, `### Corrigé`) ; aucun renvoi de traçabilité publié ; rien qui promette au-delà du code ; le remède
   des transactions déjà importées et le retour à la 0.13.1 conformes au code de la 0.13.1 (`git show v0.13.1:…`).

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Le parseur (une ligne trop courte pour
  la colonne Devise, colonne Devise égale à une colonne optionnelle, BOM, CRLF, une devise en minuscules mêlée à la
  même en majuscules), `MixedCurrencies` et son `Display`, la réponse `422` (`t_args`, repli `format!` dont la variable
  s'appelle `currencies`), `unsupported_currency` (variantes de `CoreError`, comportement CAMT inchangé), l'aperçu
  (devise annoncée à zéro ligne valide), le formulaire (état initial d'un profil existant, `counterparty` en création,
  `undefined` dans le spread), le CAMT (multi-`Stmt`, namespace préfixé, `Ccy` absent de l'`Amt` direct).
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M25 du Dev Agent Record (variantes M9a-e, M13a-b, M17a-b, M24a-b ; mutants équivalents déclarés à l'AC17) et
  cherche une **mutation plausible qui resterait verte** (ex. : le contrôle de devise appelé sur le relevé du
  **premier** essai `EncodingMismatch` ; l'information `bank_csv_currency_assumed` poussée sur `create_csv` ;
  `normalize_currency` sans `trim` ; la dérivation des fixtures qui toucherait le préambule ; l'E2E dont la liste
  d'import serait vide). Tests qui passent à vide ? Assertions seulement négatives ? Le helper `derive_ubs` saute-t-il
  bien 9 lignes **non vides** ? L'E2E : indépendance du test voisin (empreintes distinctes), rejouabilité sur la même
  base, `data-testid` seuls.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 19, et 1-bis, 8-bis, 12-bis) contre
  le code : satisfait, partiel, absent. Les tests nommés (AC1 (1)-(5), AC13-1 à 12, AC14-1 à 8, AC19 (1), (1-bis),
  (2), Vitest (v1)-(v9), E2E AC15-1 à 3) existent-ils et prouvent-ils ce qu'ils nomment ? AC1-bis (`git diff --stat
  a2743fd4 -- crates/kesh-db/` vide, aucun `Cargo.toml`). AC17 (chaque mutation exécutée, résultat rapporté, journal
  `kesh-gate-logs/27-2-mutations.log`, sorties `27-2-mut/`). AC19 : le rouge avant correction est-il consigné
  (`kesh-gate-logs/27-2-ac19-rouge.log`) ? **Décomptes recomptés depuis la source** (clés i18n, `sitesTotal`, tests
  ajoutés par fichier, nextest / Vitest / E2E déclarés) et périmètre de chaque décompte. Les choix C-27-2-29 et
  suivants : justifiés, ou défaut ? Les écarts déclarés au Dev Agent Record : réels, et complets ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/27-2-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les huit axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati des PDF au
même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, `docker`, `gh` en écriture, aucune requête SQL ni HTTP. ⛔ **Aucun interpréteur** : ni `python3` (même `-I`,
même en lecture seule), ni `python`, `perl`, `ruby`, `node`, `deno`, `bash -c` d'un script, ni `awk` utilisé comme
programme au-delà d'un filtre d'une ligne. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
