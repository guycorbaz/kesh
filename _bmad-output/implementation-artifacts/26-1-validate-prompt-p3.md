# Prompt — validation P3 de la spec, Story 26-1 (import du CSV UBS tel quel)

*Versionné le 2026-10-10. Deux lentilles (**Sonnet**), contexte frais chacune, en lecture seule. Rotation (D6, rétro
25) : P1 Sonnet, P2 Opus, P3 Sonnet. Passe **complète** et non ciblée : la remédiation P2 change encore une règle
métier (compteur propre aux erreurs de signe, deux messages selon la colonne Débit, extourne bloquante).*

Dépôt `/home/gcorbaz/devel/kesh-26-1`, branche `story/26-1` (base `fe162504`). Fiche :
`_bmad-output/implementation-artifacts/26-1-import-csv-ubs-natif.md`. **La remédiation P2 est le commit `b62a0857`**
(`git show b62a0857`), précédée de `deec289e` (retrait des chiffres du relevé réel) ; remédiation P1 `963c270c`, fiche
d'origine `77486ace`. Rapports antérieurs : `/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p{1,2}-{R,F}.md` — lis-les
**après** ta propre lecture de la fiche. Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`. Issues #620
et #621 (`gh api …/issues/N` et `…/N/comments`). Registre : `epic-26-choix-autonomes.md` (C-26-1-1 à 24 ; bandeaux
d'amendement sur C-26-1-5, 7, 8, 14, 15). Règles : `CLAUDE.md` (§ Review Iteration Rule, § Le prompt d'une passe doit
NOMMER le manuel, § Inventorier les sites NON RÉSOLUS, § Propagation post-patch, § Migration breaking policy P1–P8).
Fiches voisines : 26-2 (`/home/gcorbaz/devel/kesh-26-2`, migration `20261011000002`) et 26-3 (manuels, écran
d'import).

**Arbitrages de l'orchestrateur, non contestables ici** (contester leur *mise en œuvre*, pas leur principe) : refus de
signe bloquant et non confirmable, **y compris pour une extourne isolée** ; compteur propre au signe ; lignes du refus
listées à l'écran ; codes de ligne en `SCREAMING_SNAKE_CASE`, libellés par table TS littérale déclarée dans
`MOTIFS_DYNAMIQUES` ; le § « Rejet partiel » et la procédure « Import avec un profil existant » du manuel réécrits par
cette story ; horodatages 26-1 `…000001` / 26-2 `…000002` ; fixture synthétique. Décisions du Project Lead : sens des
montants = réglage explicite ; plafond relevé sans détection de la ligne de titres ; préambule hors périmètre.

⛔ **Données personnelles — règle durcie en P2** : aucun chiffre du relevé réel (somme, soldes, nombre de mouvements,
de sorties, d'entrées, de lignes), pas même agrégé, ne doit figurer dans la fiche, le registre, les prompts **ni ton
rapport**. Le vrai fichier est `/home/gcorbaz/kesh-test/transactions.csv` : tu peux le lire pour comparer la STRUCTURE
et pour vérifier, **sans rien afficher**, qu'aucune de ses valeurs n'apparaît dans les fichiers de la story (compare
dans un script qui n'imprime que `fichier:ligne: <catégorie>`, jamais la valeur).

## Axes — braqués sur `b62a0857`, puis sur ce que P1 et P2 n'ont pas exercé

1. **Compteur propre au signe (AC3, AC14-7-bis)** : la prescription est-elle implémentable sans ambiguïté dans
   `parse_csv_collect` (deux closures, plafond général et plafond du signe, `total_errors` général encore utilisé par
   `PartialFailure`) ? `truncated` et `total_errors` de `CsvError::AmountSignMismatch` sont-ils définis sans recouvrement
   avec ceux de `PartialFailure` ? `parse_csv` (wrapper) ?
2. **`whole_debit_column` (C-26-1-23)** : définition exacte (« toutes les valeurs non nulles de la colonne Débit en
   défaut, au moins une ») — et les cas limites : fichier sans débit, débits tous nuls, débit de mauvais signe sur une
   ligne qui échoue d'abord sur sa date, débit et crédit tous deux en défaut, profil `negative` dont tous les débits
   sont positifs. Le message `…-refused-profile` est-il vrai dans chacun ? Les tests AC14-7-ter et AC15 le fixent-ils ?
3. **Écran** : liste des lignes dans le panneau d'erreur (état `details` conservé, troncature, `data-testid`),
   absence de case de confirmation, table `CSV_LINE_ERROR_LABELS` et sa déclaration dans `i18n-keys.test.ts`
   (`MOTIFS_DYNAMIQUES`, `CARDINALITES`, relais, recompte d'`ATTENDU` — la prescription correspond-elle au mécanisme
   réel du test, l. ≈ 553-950 ?). Le panneau d'aperçu (`warnings.invalidLines`) passe-t-il aussi par la table ?
4. **Manuel** (`docs/manual/fr/user-manual.tex`, § Import bancaire, l. ≈ 1625-1715, et son PDF
   `docs/manual/fr/user-manual.pdf`, aplati `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : ce qu'AC18 fait écrire —
   Profils banque, sous-section UBS, Rejet partiel, Import avec un profil existant, extourne, colonne unique, devise,
   numéros, angle mort — est-il **vrai du code prescrit et de l'écran réel** (`BankImportUpload.svelte`,
   `BankProfileSelector.svelte`, `+layout.svelte`) ? Reste-t-il au chapitre une phrase que la story rend fausse
   (doublons, « rapport d'import ») ? Le CHANGELOG prescrit dit-il juste ?
5. **AC9 et `migrations_upgrade_path.rs`** : la fonction locale `index_of_migration` + la fonction locale
   `apply_migrations_up_to` (qui prend un **nombre** de migrations) : l'index rendu est-il le bon argument (nombre
   *avant* la migration) ? L'inventaire `\b(44|78)\b` est-il complet ?
6. **Propagation** : chaque symptôme corrigé en P2 (refus « si au moins une », `BANK_CSV_PARSE_PARTIAL_FAILURE`,
   `common/mod.rs`, `messageI18nKey` lu par l'écran, « aucun import n'apparaît », « physique », chiffres réels) a-t-il
   encore une copie dans la fiche, le registre ou les prompts ? Greper la **valeur**.
7. **Non exercés jusqu'ici** : syntaxe MariaDB de la migration (précédents `20260714000002`, `20261001000004`) ; import
   d'une sauvegarde 0.13.1 dans une 0.13.0 (manifeste) suivi au code ; rendu Svelte 5 du `fieldset` ; `setInputFiles`
   avec `buffer` ; `Decimal::from_str_exact("+500.00")` lu au source de `rust_decimal`.

## Lentilles

- **R — Auditeur d'acceptation** : `.claude/skills/bmad-create-story/checklist.md` ; chaque phrase des deux issues,
  des décisions du Project Lead **et des arbitrages de l'orchestrateur** couverte par un AC testable ; chaque AC par une
  tâche, chaque tâche par un test, chaque garde par une mutation (AC19, M1-M16) ; faits cités revérifiés au code ;
  décomptes de la fiche recomptés (21 AC, 22 mutations, bilan P1 de 20 lignes, bilan P2 de 20 lignes).
- **F — Full-scope adversary** : la conception remédiée elle-même ; ce que le parseur ou la route laisse encore passer
  en silence ; ce que le manuel affirmerait de faux ; tests existants qui changent de sens ; ce que la remédiation a
  **introduit**.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie copiée),
correction proposée, et pour chacun : **né de la remédiation P2 (`b62a0857`)**, **né de la remédiation P1 (`963c270c`)** ou **d'origine** (critère D5 de découpage).
Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non
exercés** — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-1-validate-p3-<R|F>.md` ; dernier message : le chemin, le bilan, une ligne par
MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh` et `scripts/regen-test-schema.sh`), `make`, `latexmk`,
`git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`, `bash` sur un
script du dépôt, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
