# Prompt — validation P2 de la spec, Story 26-1 (import du CSV UBS tel quel)

*Versionné le 2026-10-10. Deux lentilles (**Opus**), contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
la P1 était Sonnet ; passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1` (base `fe162504`, = v0.13.0 + kickoff de l'Epic 26).
Fiche : `_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md`. **La remédiation de la P1 est le commit
`963c270c`** (`git show 963c270c`) ; la fiche d'origine est `77486ace`. Rapports de la P1 :
`/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p1-{R,F}.md` — lis-les **après** ta propre lecture de la fiche, pour
ne pas hériter de leur cadrage. Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` (décisions du
Project Lead). Issues #620 et #621 (`gh api repos/guycorbaz/kesh/issues/N` et `.../N/comments`). Registre :
`epic-26-choix-autonomes.md` (C-26-1-1 à 22 ; C-26-1-5 et C-26-1-7 amendés par C-26-1-15 et C-26-1-16). Règles :
`CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit NOMMER le manuel, § Inventorier les sites NON
RÉSOLUS, § Propagation post-patch, § Migration breaking policy P1–P8). Fiches voisines sur d'autres branches : 26-2
(`/home/gcorbaz/devel/kesh-26-2`, migration `20261011000002`) et 26-3 (manuels, écran d'import).

**Arbitrages de l'orchestrateur, non contestables ici** (contester leur *mise en œuvre*, pas leur principe) :
`AMOUNT_SIGN_MISMATCH` bloquante et non confirmable ; codes de ligne en `SCREAMING_SNAKE_CASE` et message traduit à
l'écran ; horodatages 26-1 `…000001` / 26-2 `…000002` ; fixture synthétique (le critère du fichier réel tenu par le test
`#[ignore]` `KESH_UBS_REAL_CSV`, en agrégats). Décisions du Project Lead : sens des montants = réglage explicite ; plafond
relevé sans détection de la ligne de titres ; préambule hors périmètre.

## Axes — braqués sur la remédiation `963c270c`, puis sur ce que la P1 n'a pas exercé

*Motif (CLAUDE.md, passe ciblée) : la sévérité se déplace vers ce qu'on vient d'écrire. La remédiation a ajouté une
variante d'erreur bloquante, un AC (AC3-bis), un changement de graphie d'API, des statuts HTTP réécrits et une
sémantique des numéros de ligne : c'est là qu'il faut chercher d'abord.*

1. **L'erreur de signe bloquante (AC3, C-26-1-15)** : `HardFailure(CsvError::AmountSignMismatch { … })` est-elle
   cohérente avec `ParseCsvOutcome`, `parse_csv` (wrapper), le plafond `MAX_CSV_LINE_ERRORS`, la sentinelle
   `empty_valid_sentinel_date`, et **chaque** chemin de `routes/bank_imports.rs` qui consomme le résultat (preview,
   create, les chemins de nouvelle tentative `forced = csv_profile.clone()` l. ≈ 1644 et 1796, `EncodingMismatch` +
   confirmation) ? Un chemin peut-il encore offrir la confirmation ou écrire des lignes ? Le frontend dérive-t-il la
   case de confirmation d'un champ que ce 422 porterait aussi (`details.lines`) ? Que devient un fichier qui a à la fois
   une erreur de signe, une `INVALID_DATE` et un `EncodingMismatch` — quel refus l'emporte, et est-ce écrit ?
2. **AC3-bis — graphie des codes et message à l'écran** : inventaire **clos** des sites qui émettent, lisent ou
   comparent un code de ligne (Rust, TS, Svelte, tests, i18n, `docs/`) — pas une liste des sites qui marchent
   (§ Inventorier les sites NON RÉSOLUS). Le changement de graphie casse-t-il un test, un snapshot, une fixture E2E ?
   Les clés `bank-import-csv-errors-*` lues **dynamiquement** arrivent-elles au frontend (mécanisme de chargement des
   messages, `lint-i18n-ownership`) ? Le CHANGELOG (`### Modifié`) dit-il juste ?
3. **Statuts HTTP réécrits (AC10, AC15, C-26-1-21)** : revérifier au code chaque statut et chaque code annoncé
   (`errors.rs`, extracteur `Json` d'axum 0.8, tests `bank_profiles_e2e.rs`). Le formulaire affiche-t-il quelque chose
   d'utile sur un 422 texte brut ?
4. **Numéros de ligne (AC4, AC14-4/5/6, C-26-1-16)** : la sémantique écrite est-elle exacte au source (`csv` 1.4.0
   `src/reader.rs` l. 1323-1326, 1626, 1646-1650 ; csv-core 0.1.13 l. 94, 665, 724), y compris pour `headers()` après
   une ligne vide, pour `HeaderSkipsData { line }`, et pour la fixture **sans LF final** ? Les attendus 9, 11, 1 des
   tests sont-ils ceux que le code rendra ?
5. **Migration et compteurs** (AC6, AC7, AC9) : `migrations.sha384`, `total` 78 → 79 et `total - 44` → `total - 45`,
   texte de `common/mod.rs`, test AC9 dans `migrations_upgrade_path.rs`, `EXEMPT_MIGRATIONS` 17 → 18, squash,
   P2-bis. **Ce que la P1 n'a pas exercé** : la syntaxe MariaDB exacte de `DROP CONSTRAINT` + `ADD CONSTRAINT` du même
   nom (documentation MariaDB, forme de l'existant dans `crates/kesh-db/migrations/`), et l'interaction du bump
   `min_required` 0.13.1 avec `check_downgrade_protection`, l'import d'une sauvegarde (manifeste) et
   `scripts/prepare-release.sh` (lecture seule : ne **pas** l'exécuter).
6. **Propagation** : chaque symptôme corrigé en P1 (ligne 10 → 9, « erreur de ligne » → refus global, 400 → 422,
   `BANK_CSV_PROFILE_VALIDATION`, 79 → 78/79, `INVALIDDATE`) a-t-il encore une copie dans la fiche, le registre
   (C-26-1-5, C-26-1-7, C-26-1-8, C-26-1-9 non amendés ?), le manuel prescrit ou le CHANGELOG prescrit ? Greper la
   **valeur**.
7. **Manuel** (`docs/manual/fr/user-manual.tex` § Import bancaire → Profils banque, l. ≈ 1665-1700, et son PDF
   `docs/manual/fr/user-manual.pdf`, aplati `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : ce que AC18 fait écrire
   est-il vrai du code prescrit (refus global, numéros, colonne unique non contrôlée, devise CHF, angle mort de la
   garde) ? La sous-section n'existe pas encore : vérifier la **prescription**, et que rien dans le `.tex` actuel ne
   contredit la fiche hors des lignes laissées à la 26-3.
8. **Non exercés par la P1, à couvrir** : rendu et accessibilité du formulaire (Svelte 5, `fieldset`/`legend`, état du
   choix quand on bascule de mode) ; ordonnancement réel de la suite E2E (`playwright.config.ts`, `retries`, remise à
   zéro de `kesh_e2e`, `setInputFiles` avec `buffer`) ; semver de `kesh-import` (variants publics ajoutés à `CsvError`
   et `CsvLineErrorCode`, champ public à `CsvProfile` — `cargo publish --dry-run` en CI ne le voit pas ; lecture
   seule, pas de `cargo`).
9. **Données personnelles** : aucune valeur réelle de Guy dans la fiche, le registre, ce prompt ; AC17 n'imprime rien
   de personnel, **même en échec**. Le vrai fichier est `/home/gcorbaz/kesh-test/transactions.csv` : tu peux le lire
   pour comparer la STRUCTURE, ⛔ n'en recopie RIEN dans ton rapport (ni ligne, ni nom, ni IBAN, ni montant, ni solde).

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase des deux issues,
  des décisions du Project Lead **et des arbitrages de l'orchestrateur** couverte par un AC testable ; chaque AC par une
  tâche, chaque tâche par un test, chaque garde par une mutation (AC19, M1-M16) ; faits cités revérifiés au code ;
  décomptes de la fiche recomptés (21 AC, 16 mutations, bilan P1 de 20 lignes).
- **F — Full-scope adversary** : la conception remédiée elle-même ; ce que le parseur ou la route laisse encore passer
  en silence ; ce que le manuel affirmerait de faux ; tests existants qui changent de sens ; ce que la remédiation a
  **introduit**.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée, et pour chacun : **né de la remédiation `963c270c`** ou **d'origine** (critère D5 de découpage).
Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non
exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p2-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et `scripts/regen-test-schema.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un
script du dépôt, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
