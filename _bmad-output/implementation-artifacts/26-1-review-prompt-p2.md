# Prompt — revue de code P2, Story 26-1 (importer le CSV d'UBS tel quel — sens des débits, lignes à ignorer)

*Versionné le 2026-10-11. Protocole **complet** à trois lentilles (B, E, A), en **Opus**, contexte frais chacune,
lecture seule. Passe complète et non ciblée : la remédiation P1 touche plusieurs modules (`kesh-import` parseur et
erreurs, `kesh-api` routes et export, panneau Svelte, E2E, manuel).*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1`. **Le diff à revoir** :
`git diff 91426149 8639c786 -- . ':!_bmad-output'` (`91426149` = fin de la validation de la fiche ; `8639c786` =
dernier commit de code, remédiation P1 ; les commits suivants ne portent que la fiche, le registre et ce prompt).
⛔ **Attention particulière au diff de remédiation** `git diff f23ad162 8639c786 -- . ':!_bmad-output'` : sur les
stories précédentes, la sévérité s'est déplacée vers ce qu'on venait d'écrire (CLAUDE.md § « La passe ciblée ») —
cherche d'abord ce que la remédiation P1 a pu casser ou laisser à moitié.

Fiche : `_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md` — AC1 à AC20 (dont AC3-bis), Tâches, Dev
Notes, le **Dev Agent Record** et le **Change Log** (section « Revue de code P1 — bilan finding par finding »). Choix
C-26-1-1 à 39 au registre `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md` (neufs en P1 : 38, 39).
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/26-1-review-p1-{B,E,A}.md`. Journaux de la remédiation :
`26-1-mutations-p1-nouvelles.log`, `26-1-mutations-p1-rejeu.log`, `26-1-publish-dry-run.log`,
`26-1-p1b-gate-driver.log`, `26-1-p1b-gate-backend.log`, `26-1-p1b-front-*.log`, `26-1-p1b-e2e-full.log` (gate au dernier commit de code), et ceux du premier passage sur `a2b281fb` : `26-1-p1-*.log`,
`26-1-p1-e2e-rejeu-sidebar.log` (même dossier).
Règles : `CLAUDE.md` du dépôt — § « Migration breaking policy » (P1 à P8), § « Recompter ses propres comptes rendus »,
§ « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Une passe qui ne déclare pas ses
axes exercés ne compte pas comme passe ». Issues : #620 (la PR la fermera), #621 (référencée seulement). Hors
périmètre : la § « Import avec un profil existant » du manuel (Story 26-3), la devise forcée à CHF (préexistante,
écrite au manuel), le `-amount` de CAMT (`camt053/mod.rs`, recensé au Change Log).

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** Le relevé réel de Guy n'est jamais à lire : ne cherche pas, n'ouvre pas,
n'utilise pas de fichier hors du dépôt. N'écris dans ton rapport **aucune** donnée bancaire réelle ; les seules
valeurs citables sont celles de la fixture synthétique `crates/kesh-import/tests/fixtures/csv/ubs_synthetique.csv`.

**Ce que la remédiation P1 a changé** (à vérifier, pas à croire) :
- `error.rs` : `all_is_complete` compare `all()` (par `as_str`) à la liste des variants que serde passe à
  `deserialize_enum` (désérialiseur de capture dans le module de tests) ; doc-comment de `all()` récrit ; AC14-9 récrit.
- `parser.rs` : test `header_guard_runs_before_first_data_record` (E-1) ; deux cas ajoutés à
  `whole_debit_column_definition` (E-2) ; débit nul → `Decimal::ZERO` dans `parse_row` et assertion
  `!is_sign_negative()` dans `zeros_and_explicit_plus_are_accepted` (E-4, C-26-1-39).
- `tests/ubs_fichier_reel.rs` : répartition non stricte (E-6).
- `routes/bank_imports.rs` : doc-comment dupliqué retiré (F4) ; `exports/csv_tables.rs` : imports du module de tests
  regroupés (F5) ; `BankImportUpload.svelte` : `errorDetails = null` au site `BANK_ACCOUNT_REQUIRED` et aux deux branches
  `UNKNOWN_ERROR` (F6 et sa propagation, `8639c786`) ;
  `bank-csv-import.spec.ts` : numéro de ligne en texte exact dans sa cellule (E-7).
- Manuel : ligne « Encodage » au tableau « Le relevé CSV d'UBS » (F7), PDF régénéré.
- Fiche : phrase du Dev Agent Record sur l'arbre « vérifié propre » rectifiée (F2), cause écrite, douze mutations
  rejouées ; mutations neuves P1-N1 à N5.

⛔ **Axes communs aux trois lentilles** (chacune les déclare exercés ou non) :
1. **Le sens des montants, de bout en bout.** Formulaire → requête → DTO → validation → base →
   `db_profile_to_csv_profile` → parseur, à l'aller (POST, PUT) et au retour (GET, export). **Tous** les constructeurs
   de `CsvProfile` et tous les lecteurs de `bank_profiles` (`grep -rn "CsvProfile {" crates`,
   `grep -rn "bank_profiles" crates --include=*.rs`). La normalisation du débit nul (E-4) change-t-elle un montant
   écrit, une clé de doublon (`dedup_key_scalar`), un total, une période ?
2. **Le refus de signe** : non confirmable sur tous les chemins (preview, import avec et sans `confirmPartialImport`,
   nouvelle tentative forcée après `EncodingMismatch` dans les deux handlers) ; rien d'écrit ; `whole_debit_column`
   conforme à C-26-1-25 sur les cas limites, **dont les deux cas neufs** ; préséances d'AC3.
3. **Lignes à ignorer et garde** : plafond 20 partout, garde avant le premier enregistrement de données (le test neuf
   prouve-t-il l'ordre, sous `header_row_count` 1 **et** ≥ 2 ?), `headers()?.position()`, angle mort, numéros de ligne.
4. **Migration et version** (P1 à P8) : rien ne doit avoir bougé dans `crates/kesh-db/migrations/` depuis `f23ad162`
   (P8 ; vérifie `git diff f23ad162 8639c786 -- crates/kesh-db`) ; bump, dix crates à 0.13.1, audit P5, P6, P7,
   `sha384`, squash — relus ou déclarés non exercés.
5. **i18n** : clés neuves dans les quatre locales, replis égaux au fr-CH, arguments Fluent, recomptes de
   `i18n-keys.test.ts` et `i18n-libelle-en-dur.test.ts` (la remédiation ne doit pas les avoir déplacés).
6. ⛔ **LE MANUEL — `docs/manual/fr/user-manual.tex` ET son PDF.** Les sections « Profils banque », « Le relevé CSV
   d'UBS » (dont la ligne neuve « Encodage » : l'auto-détection lit-elle bien le fichier UTF-8 à BOM ? relire
   `crates/kesh-import/src/csv/` et le libellé du formulaire `BankProfileForm.svelte`), « Détection des doublons »,
   « Rejet partiel » disent-elles **vrai** sur le code ? Le débit nul (E-4) est-il décrit quelque part, et juste ?
   **Cherche une phrase du manuel, hors des sections réécrites, que la story ou sa remédiation rend fausse.** Contrôle
   le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/`) — l'apostrophe y est typographique, écrire `.` à sa place dans un motif ; le
   PDF est-il bien régénéré depuis le `.tex` de `8639c786` (la ligne « Encodage » s'y lit-elle) ?
7. **CHANGELOG** : une seule section `## [0.13.1] — Non publié` ; rien qui promette au-delà du code ; la remédiation
   P1 appelle-t-elle une ligne (débit nul) ?

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. La normalisation du débit nul (sous
  les deux réglages, `0.00`, `-0.00`, `+0`, avec et sans `DebitObservation::Zero`) ; le désérialiseur de capture de
  `error.rs` (que se passe-t-il si serde change de chemin, si `deserialize` n'appelle pas `deserialize_enum` — le test
  échoue-t-il bruyamment, ou passe-t-il à vide ?) ; le panneau (`errorDetails` à **chaque** site où `errorCode` est
  posé, après la remédiation) ; tout ce que la P1 a laissé.
- **E — Edge Case Hunter : bords, et les tests prouvent-ils ce qu'ils nomment.** Rejoue mentalement les mutations
  M1 à M28 **et** P1-N1 à N5 (journaux nommés plus haut) et cherche une **mutation plausible qui resterait verte**,
  en priorité sur le code de la remédiation : `all_is_complete` resterait-il vert si `all()` perdait l'ordre, ou
  doublait un variant ? Le test neuf de la garde resterait-il vert si la garde lisait `last_skipped` d'une autre
  façon ? Les deux cas neufs de `whole_debit_column_definition` distinguent-ils bien les deux mutations qu'ils
  visent, et seulement elles ? L'E2E resserré (`toHaveText('11')` sur la première cellule) : la cellule est-elle bien
  le numéro (lire `BankImportUpload.svelte`) ? Rejouabilité de la spec.
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC (1 à 20, et 3-bis) contre le code, et
  **AC14-9 récrit** contre la garde réelle ; AC19 (chaque mutation exécutée, résultat rapporté) ; la rectification F2
  dit-elle ce que les journaux montrent (compare `26-1-mutations-api.log` et `26-1-mutations-p1-rejeu.log`) ? Le bilan
  P1 finding par finding du Change Log : chaque finding des trois rapports P1 y est-il, avec un traitement juste (en
  particulier les LOW déclarés **acceptés** ou **réfutés** : B2, B3, E-5, E-8, E-9) ? **Décomptes recomptés depuis la
  source** (tests backend et Vitest exécutés, E2E, mutations) avec leur périmètre ; C-26-1-38 et 39 justifiés ?

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-1-review-p2-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée ; dis
pour chacun s'il vient de la **remédiation P1** (`f23ad162..8639c786`) ou du code d'origine. Pour tout finding
affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
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
