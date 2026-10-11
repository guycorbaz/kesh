# Prompt — validation P1 de la spec, Story 26-1 (import du CSV UBS tel quel)

*Versionné le 2026-10-10. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md` (commit `77486ace`).
Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` (décisions du Project Lead). Issues #620 et #621
(`gh api repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Registre : `epic-26-choix-autonomes.md` (C-26-1-1 à 13).
Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit NOMMER le manuel, § Inventorier les sites
NON RÉSOLUS, § Propagation post-patch, § Migration breaking policy P1–P8). Fiches voisines sur d'autres branches :
26-2 (description de compte bancaire, ajoute aussi une migration) et 26-3 (manuels, écran d'import ; laisse à la 26-1
la sous-section « Profils banque » du manuel utilisateur).

Axes prioritaires :

1. **Le parseur** (`crates/kesh-import/src/csv/parser.rs`) : `debit_sign`, `AMOUNT_SIGN_MISMATCH` (C-26-1-5 — change
   le comportement des profils existants : tests, seeds, fixtures, E2E qui s'appuieraient sur un débit ou un crédit
   négatif ? grep), plafond 20, lignes vides, garde `BANK_CSV_HEADER_SKIPS_DATA` (C-26-1-8 : une seule ligne ignorée,
   `reader.headers()`, BOM, numéros de ligne), mode colonne unique. Une transaction peut-elle encore disparaître ou
   changer de signe sans un mot ?
2. **Migration et version** (C-26-1-3, C-26-1-4) : colonne `debit_sign` + CHECK, recréation de
   `chk_bank_profiles_header_row_count` (syntaxe MariaDB exacte), squash de test `crates/kesh-db/test-schema/`,
   P2/P2-bis (bump `min_required` 0.13.1 et dix `Cargo.toml` : justifié ? tenable dans cette story ? interaction avec
   `scripts/prepare-release.sh` et les branches 26-2/26-3), P3, P5 (recompte), P6, P7 (`EXEMPT_MIGRATIONS` Durable,
   compteur recompté), P8 ; course de version de migration avec la 26-2.
3. **Chaîne complète du réglage** : entité, repository, `db_profile_to_csv_profile` (le site qui, s'il l'oublie,
   laisse tout vert pour rien), routes (`debitSign` obligatoire, C-26-1-9), export CSV des tables, `.keshbackup`
   (sauvegarde antérieure sans la colonne), formulaire `BankProfileForm.svelte`, i18n 4 locales,
   `docs/api-external.md`, semver de `kesh-import` (`cargo publish --dry-run` en CI).
4. **Données personnelles** : la fixture est-elle entièrement synthétique ? aucun IBAN, nom, solde ni montant réels de
   Guy dans la fiche. Le vrai fichier est `/home/gcorbaz/kesh-test/transactions.csv` : tu peux le lire pour comparer
   la STRUCTURE, ⛔ n'en recopie RIEN dans ton rapport (ni ligne, ni nom, ni IBAN, ni montant). La fixture
   reproduit-elle exactement la structure UBS (BOM, préambule, ligne vide, titres, point-virgule final, colonnes
   vides) ? Le test `#[ignore]` `KESH_UBS_REAL_CSV` prouve-t-il somme = solde final − solde initial ?
5. **Manuel** (sous-section « Profils banque » du manuel utilisateur, `.tex` et PDF aplati
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG `[0.13.1]`, recompte AC/tâches/tests/mutations, gates
   prescrits (E2E complet au dernier commit de code).

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase des deux issues et
  des décisions du Project Lead couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; faits
  cités revérifiés au code.
- **F — Full-scope adversary** : la conception elle-même ; ce que le parseur laisse passer en silence ; ce que le
  manuel affirmerait de faux ; tests existants qui changeront de sens ; chemins qui lisent ou écrivent un profil.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes
exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p1-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/
`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un script du dépôt, `gh` en écriture, SQL.
Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
