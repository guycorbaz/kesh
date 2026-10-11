# Prompt — revue de code P1, Story 26-1 (importer le CSV d'UBS tel quel — sens des débits, lignes à ignorer)

*Versionné le 2026-10-11. Trois lentilles (Sonnet), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1`. **Le diff à revoir** :
`git diff 91426149 __CODE__ -- . ':!_bmad-output'` (`91426149` = fin de la validation de la fiche ; `__CODE__` = dernier
commit de code ; les commits suivants ne portent que la fiche, le registre et ce prompt). Fiche :
`_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md` — AC1 à AC20 (dont AC3-bis), Tâches, Dev Notes, et
le **Dev Agent Record** (choix C-26-1-28 à 35 au registre `epic-26-choix-autonomes.md`, mutations M1 à M28, décomptes).
Règles : `CLAUDE.md` du dépôt — § « Migration breaking policy » (P1 à P8), § « Recompter ses propres comptes rendus »,
§ « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Pattern batch » (hors sujet ici).
Issues : #620 (la PR la fermera), #621 (référencée seulement : la lecture de l'IBAN et des soldes du préambule reste à
faire). Hors périmètre : la § « Import avec un profil existant » du manuel (Story 26-3), la devise forcée à CHF
(préexistante, écrite au manuel).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** Le relevé réel de Guy n'est jamais à lire : ne cherche pas, n'ouvre pas,
n'utilise pas de fichier hors du dépôt. N'écris dans ton rapport **aucune** donnée bancaire réelle ; les seules
valeurs citables sont celles de la fixture synthétique `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`.

**Contexte** : story **pleine pile**. `kesh-import` : `DebitSign`, `CsvProfile.debit_sign`, `MAX_HEADER_ROW_COUNT`,
`parse_row` → `RowParse`/`DebitObservation`, `RowAccumulator` (compteur et vecteur propres au signe,
`whole_debit_column ⇔ d ≥ 2 et e = d`), garde de surcomptage `guard_header_skips_data`,
`CsvError::AmountSignMismatch`/`HeaderSkipsData`, `CsvLineErrorCode::AmountSignMismatch`/`as_str()`/`all()` ;
fixture et test `#[ignore]` `ubs_fichier_reel.rs`. `kesh-db` : migration `20261011000001` (colonne, deux CHECK, bump
`min_required` 0.13.1), squash, `migrations.sha384`, entité (`debit_sign_enum`), dépôt (COLUMNS/INSERT/UPDATE/audit),
`EXEMPT_MIGRATIONS`, tests de migration. `kesh-api` : DTO et réponse des profils, `db_profile_to_csv_profile`,
`map_csv_error` (deux bras), `AppError` (deux variantes, deux messages), `csv_line_error_to_payload`, export CSV.
Frontend : `BankProfileForm.svelte` (radios, max 20, aide), `BankImportUpload.svelte` (`errorDetails`, liste du refus,
message par ligne), `csv-line-error-label.ts`, deux gardes i18n recomptées, Vitest, E2E (`bank-csv-import.spec.ts`).
Textes : quatre `messages.ftl`, manuel utilisateur FR (`.tex` **et PDF**), `CHANGELOG.md`,
`docs/migrations-idempotence-audit.md`. Dix `Cargo.toml` à 0.13.1.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **Le sens des montants, de bout en bout.** Le réglage traverse-t-il formulaire → requête → DTO → validation → base →
   `db_profile_to_csv_profile` → parseur, à l'aller (POST, PUT) comme au retour (GET, export) ? Un site qui lirait le
   profil sans le réglage (cherche **tous** les constructeurs de `CsvProfile` et tous les lecteurs de `bank_profiles`,
   `grep -rn "CsvProfile {" crates` et `grep -rn "bank_profiles" crates --include=*.rs`) ?
2. **Le refus de signe** : non confirmable sur **tous** les chemins (preview, import avec et sans
   `confirmPartialImport`, nouvelle tentative forcée après `EncodingMismatch` dans les deux handlers) ; aucune
   transaction ni import écrits ; `whole_debit_column` conforme à la définition fermée (C-26-1-25) sur les cas limites ;
   préséances d'AC3 (avant la boucle, puis signe, puis autres).
3. **Lignes à ignorer et garde** : plafond 20 partout (validate, CHECK, formulaire, textes), garde placée avant le
   premier enregistrement de données, `headers()?.position()` pour une ligne, angle mort écrit et fixé, numéros de
   ligne (sémantique de la ligne vide).
4. **Migration et version** (P1 à P8) : bump `min_required` en dernière instruction **et** dix crates à 0.13.1 dans le
   même commit, audit P5 recompté (en-tête, `Total`, partitions), P6 (`total` 79 et `total - 45`, frontière 34), P7
   (`EXEMPT_MIGRATIONS`, compteur 18), ligne `sha384` égale au fichier, squash, aucun `;` ni `--` dans un littéral.
5. **i18n** : clés neuves dans les quatre locales, repli Svelte et repli Rust = texte fr-CH, arguments Fluent
   (`$count`, `$line`) présents partout, famille `bank-import-csv-errors-` dans `i18n-keys.test.ts` (recomptes avec leurs
   lignes d'historique : `sitesTotal` 2020 → 2026, `sitesGabarit` 10 → 11 ; `i18n-libelle-en-dur.test.ts` 61 → 62).
6. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET son PDF.** Les sections réécrites (« Profils banque »,
   « Le relevé CSV d'UBS », « Détection des doublons », « Rejet partiel ») disent-elles **vrai** sur ce que fait le code
   (relire `BankProfileForm.svelte`, `BankImportUpload.svelte`, `parser.rs`, `crates/kesh-core/src/bank_imports.rs`
   (`dedup_key_scalar`), les handlers de `routes/bank_imports.rs`) ? Chaque affirmation : chemin de menu, libellés,
   bornes, les trois formes de signe refusées, « au moins deux », extourne, colonne unique, garde datée, numéros de
   ligne, devise, profil UBS (valeurs du tableau contre la fixture), « 9 et non 10 », référence et doublons, case
   « Importer les lignes valides quand même ». **Cherche une phrase du manuel, hors des sections réécrites, que la story
   rend fausse** (ex. : une mention de « 0-5 », du rejet partiel ou du signe ailleurs dans le manuel, la FAQ, le
   glossaire, le manuel d'administration). Contrôle le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - |
   tr '\n' ' ' | tr -s ' '` vers `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire
   `.` à sa place dans un motif.
7. **CHANGELOG** : une seule section `## [0.13.1] — Non publié` ; rien qui promette au-delà du code ; changement
   d'API (graphie des codes, `debitSign` obligatoire, deux 422, colonne d'export) complet ; « À faire en passant à la
   0.13.1 » exact.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. `parse_row` (chaque branche, zéro,
  `+500.00`, `-0.00`, débit illisible, ligne courte), `RowAccumulator` (comptes `d`/`e`, plafonds, `push_general` pour
  les erreurs `csv`), garde (fichier vide avec `header_row_count = 1`, `headers()` en erreur, profil `header_row_count =
  0`), conversions d'erreur (bras manquant → 500 masqué ?), `debit_sign_enum` (aucun repli), DTO `deny_unknown_fields`,
  formulaire (état de `debitSign` quand `existing` change, décocher/recocher, envoi en colonne unique), panneau
  (`errorDetails` posé et remis à zéro à **chaque** site où `errorCode` l'est ; `details` mal formé), types TS contre
  les DTO Rust.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M28 du Dev Agent Record (dont M19, déclarée équivalente sous `is_sign_negative()`, et M26, qui modifie la
  migration) et cherche une **mutation plausible qui resterait verte** (ex. : `whole_debit_column` compté sur les
  crédits ; erreur de signe rangée aussi dans le vecteur général ; la garde lisant une autre colonne que `date` ;
  `db_profile_to_csv_profile` lisant le réglage mais le parseur l'ignorant sur la première ligne ; le formulaire
  initialisé à `positive` pour un profil `negative`). Tests qui passent à vide ? Le test AC9 (assertion de montage).
  L'E2E : scénarios indépendants (motifs distincts), « aucun import de plus » par l'API, `data-testid` seuls,
  rejouabilité (deuxième passage de la spec sur la même base).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 20, et 3-bis) contre le code :
  satisfait, partiel, absent. Les tests numérotés d'AC14 (1 à 10, 6-bis, 7-bis/ter/quater) existent-ils et prouvent-ils
  ce qu'ils nomment ? AC15 point par point (statuts, codes, `details`, graphie, PUT relu, rien d'écrit). AC16 (trois
  scénarios, scan axe sans exclusion). AC17 (rien d'imprimé qu'un nom d'assertion ou « conforme », aucun `assert_eq!`
  sur une valeur lue). AC18 et AC19 (chaque mutation exécutée, résultat rapporté). **Décomptes recomptés depuis la
  source** (migrations 79 ; audit 69/10/0 ; `EXEMPT_MIGRATIONS` 18 ; Vitest et sites i18n ; tests annoncés) et
  périmètre de chaque décompte. Les choix C-26-1-28 à 35 : justifiés, ou défaut ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-1-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour tout
finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les sept axes communs et ceux de ta lentille) — un « 0 finding » sans elle
ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et le texte aplati du PDF au même
endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo` (aucune sous-commande), `npm`, `npx`,
`node`, `docker`, `gh` en écriture, aucune requête SQL ni HTTP. Aucun fichier hors du dépôt (en particulier aucun relevé
bancaire réel). Autorisés : lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers
`/home/gcorbaz/devel/kesh-gate-logs/`.
