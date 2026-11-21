# Correctif : le bilan affiche « Résultat reporté (calculé) » à 0.00 à côté du compte de report

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/bilan-resultat-reporte`, partie de `main` à `70be12a6` (squash de
l'Epic 37, PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` et va à la prochaine release
planifiée. **Aucune migration.** Relevé par le responsable de l'epic à la recette d'un exercice réel 2024
(`data-not-committed/2024-qesh/sorties-kesh/bilan-2024.pdf`).*

## Intention

Le bilan d'une société qui tient son premier exercice dans Kesh affichait, dans la section *Capitaux propres*, son
compte de report (rôle `RetainedEarnings`, ici « 2970 Bénéfice / perte reporté », -37'571.82) **et**, juste dessous,
la ligne calculée « Résultat reporté (calculé) » à **0.00**. À zéro, cette ligne n'apporte rien et trouble le lecteur :
deux « résultats reportés » côte à côte, dont l'un nul (l'ancien logiciel du contributeur ne l'avait pas).

**Prémisse vérifiée au code** (`origin/main` `70be12a6`) :

- `BalanceSheet::retained_earnings` (`crates/kesh-report/src/balance_sheet.rs`) est le cumul des résultats des
  exercices **strictement antérieurs** tenus dans Kesh (P&L `entry_date < fy_start`) : nul au premier exercice, où le
  report est porté par le compte physique (soldes d'ouverture).
- La ligne était dessinée **sans condition** : au PDF (`render_balance_sheet_pdf`, boucle des deux lignes calculées),
  au CSV (`render_balance_sheet_csv`, `write_fx_record` inconditionnel) et à l'écran
  (`BalanceSheetView.svelte`, `<tr>` inconditionnelle). Le PDF du responsable de l'epic le montre (`pdftotext` : « Résultat reporté
  (calculé) 0.00 » sous « 2970 Bénéfice / perte reporté -37'571.82 »).
- **Pourquoi elle était toujours affichée.** La Story 14-1 l'a créée comme report à-nouveau virtuel ; la décision D1 de
  la Story 14-3c (`14-3c-fonds-propres-par-role.md`, « Collision Résultat reporté : DISTINGUER nettement ») impose de
  ne **jamais fusionner** le compte physique `RetainedEarnings` et la ligne calculée, et de marquer celle-ci
  « (calculé) ». D1 n'exige pas d'afficher la ligne nulle : elle traite le cas où **les deux** valeurs coexistent. Le
  compte physique, lui, n'est listé que non nul (`HAVING balance != 0`). La Story 14-1, elle, demandait à la lettre
  d'afficher la ligne sans condition (`14-1-cloture-report-des-soldes.md` l. 102 : « Frontend `BalanceSheetView.svelte` :
  afficher « Résultat reporté » + « Résultat de l'exercice » ») ; mais son AC-D (l. 79) veut qu'au **premier
  exercice** le bilan « dégénère exactement au comportement mono-exercice actuel (régression zéro) » — et avant la
  14-1 il n'y avait aucune ligne de report : omettre la ligne nulle **sert** l'AC-D plutôt qu'elle ne la contredit.
  Aucun texte ne motive l'affichage d'un zéro ; le seul garde-fou lié est la garde « rapport vide » (`is_empty`,
  revue P1 de la 14-1), qui reste inchangée. *(Paragraphe corrigé à la revue P1, LOW-2 : il disait « aucun texte de
  14-1 ni de 14-3c ne motive l'affichage d'un zéro », inexact à la lettre pour la l. 102 de la 14-1.)*

## Décision

1. **Écran et PDF** : la ligne calculée n'est affichée que si `retained_earnings` est **non nul** (positif : bénéfice
   reporté ; négatif : « Perte reportée » à l'écran), **jugé arrondi au centime comme il s'affiche** (l'équidistant
   loin de zéro, `round_for_display` / `roundToCents` ; revue P1, LOW-1) : un résidu sous le demi-centime
   (`0.004`, `-0.004`) omet la ligne, qui n'affiche donc jamais « 0.00 » ni « -0.00 », et `0.005` s'affiche « 0.01 »
   — la valeur écrite est arrondie de même. Non nulle, elle reste : elle signale un résultat d'exercices
   antérieurs qu'aucune écriture n'a encore reporté (et D1 s'applique toujours : deux lignes distinctes, jamais
   fusionnées). « Résultat de l'exercice » reste toujours affiché. Le total des capitaux propres et la ligne
   « Total passifs + Capitaux propres » sont inchangés (on omet un zéro) : l'équation affichée reste juste.
2. **Source unique** : `BalanceSheet::shows_retained_earnings_line()` (backend, PDF et CSV) ; pendant frontend
   `showsRetainedEarningsLine()` (`reports.api.ts`), tenu en parallèle comme `isReportEmpty` — même règle (même arrondi :
   `roundToCents`, extrait de `formatCurrencyFigure` qui l'emploie désormais), une valeur illisible restant
   affichée comme avant.
3. **JSON et MCP : inchangés.** `GET /reports/balance-sheet` rend toujours `retainedEarnings`, zéro compris, et
   `kesh_bilan` toujours `report` : c'est la donnée, non sa présentation ; un champ stable évite de casser un client.
4. **CSV : la ligne nulle est omise, comme au PDF.** Le CSV du bilan est la *présentation* du bilan en tableau (mêmes
   sections, mêmes libellés FR-CH que le PDF ; le manuel dit « cette présentation se retrouve à l'identique dans les
   exports PDF et CSV ») ; ses lignes varient déjà avec les données (comptes à solde non nul, sections Actifs/Passifs
   omises vides, CSV réduit à l'en-tête pour un bilan vide, colonnes de devise conditionnelles) : aucun lecteur ne
   pouvait compter sur une ligne à position fixe. Garder la ligne au CSV seul aurait recréé l'écart que la Story 14-3c
   a fermé entre PDF et CSV. Les totaux du CSV sont inchangés, et le JSON reste la source stable pour un programme.
   Écrit à `docs/api-external.md` (§ 7, « Le bilan et son résultat reporté calculé »).
5. **Hors du bilan : la balance des comptes garde sa ligne à zéro** (écran, PDF, CSV) : elle entre dans le contrôle
   d'ouverture de la balance (somme ramenée au sens débit, « résultat reporté compris ») ; le défaut relevé ne vise que
   le bilan. Le manuel le dit.
6. **PDF, conflit avec `fix/pdf-montants-alignes`** (non fusionnée, réécrit l'alignement des montants de
   `pdf.rs`) : changement limité à la **tête** de la boucle des lignes calculées de `render_balance_sheet_pdf`
   (`pdf.rs` l. 710–721 après correctif : le tableau devient `[Option<…>; 2]` aplati), le corps de la boucle (où
   l'autre branche remplace `use_text` par `write_right`) n'est pas touché ; test ajouté **au milieu** du module de
   tests (après `balance_sheet_pdf_shows_equity_when_assets_liabilities_net_to_zero`, l. 2451–2498 après la revue P1), non à sa fin où
   l'autre branche ajoute les siens.

## Critères d'acceptation

1. **AC-1** — Bilan dont le résultat reporté calculé est nul **une fois arrondi au centime** : la ligne « Résultat reporté (calculé) » n'apparaît ni à
   l'écran, ni au PDF, ni au CSV ; « Résultat de l'exercice », les comptes de fonds propres et les totaux sont
   inchangés.
2. **AC-2** — Résultat reporté calculé non nul, positif ou négatif : la ligne apparaît avec son montant, à l'écran
   (« Perte reportée » si négatif), au PDF et au CSV, distincte du compte physique de report (D1).
3. **AC-3** — `GET /reports/balance-sheet` rend `retainedEarnings` dans les deux cas (zéro compris) ; `kesh_bilan`
   rend `report`.
4. **AC-4** — Les fixtures de non-régression mono-devise (`tests/fixtures/mono_devise/`, « mêmes lignes de texte »)
   sont inchangées : leur résultat reporté vaut 814.45, non nul.
5. **AC-5** — Manuel FR et `docs/api-external.md` décrivent la règle.

## Tâches

- [x] Prémisse vérifiée (code, PDF du responsable de l'epic, 14-1 et 14-3c/D1)
- [x] `BalanceSheet::shows_retained_earnings_line` + test unitaire
- [x] PDF et CSV : ligne omise à zéro ; tests unitaires (absente à zéro, présente non nulle, positive et négative)
- [x] Écran : `showsRetainedEarningsLine` + `{#if}` ; Vitest (composant et helper)
- [x] Intégration HTTP : CSV sans la ligne à zéro (scénario soldes d'ouverture sur 2970), avec la ligne non nulle
      (deux exercices)
- [x] Manuel FR (`user-manual.tex`), `docs/api-external.md`
- [x] Gates ciblés, mutations, captures (ci-dessous)
- [x] Gate complet (`test-fast.sh --ci`, E2E complet, `gate-ci.sh`) : **groupé**, vert au commit `5fc6fb1b` de la branche
      d'intégration `integ/fixes-0.19.0` (décision du responsable de l'epic du 2026-11-18 ; chiffres au Change Log)
- [x] Revue de code P1 (Sonnet) : 0 au-dessus de LOW, 3 LOW corrigés dans la story
- [x] Passe ciblée P2 sur la remédiation, 0 au-dessus de LOW (prompt : `kesh-gate-logs/fix-bilan-resultat-reporte/revue-p2-ciblee-prompt.md`)

## Change Log

- **2026-11-18 — développement.** Fichiers : `crates/kesh-report/src/{balance_sheet,pdf,csv}.rs`,
  `frontend/src/lib/features/reports/{reports.api.ts,BalanceSheetView.svelte}`, tests (`balance_sheet.rs`, `pdf.rs`,
  `csv.rs`, `opening_balances_e2e.rs`, `reports_e2e.rs`, `BalanceSheetView.test.ts`, `reports.api.test.ts`),
  `docs/manual/fr/user-manual.tex`, `docs/api-external.md`. Choix CSV et JSON : § Décision, points 3 et 4.
  **Textes de référence** (« mêmes lignes de texte », `rapports_mono_devise`) : aucun à mettre à jour, leur report
  vaut 814.45 (AC-4) ; un seul test existant change d'attente : `csv.rs` `balance_sheet_csv_equity_section_by_role`
  affirmait la présence de « Résultat reporté (calculé) » avec un report **nul** à côté du compte 2970 — exactement le
  cas du défaut ; il affirme désormais son absence, et le même bilan avec un report de 300.00 la retrouve, à sa place.

- **2026-11-18 — tests, recomptés aux deux bornes (`70be12a6` → branche, `grep -c '#\[test\]\|#\[sqlx::test'`).**
  Rust : **2 tests neufs** — `balance_sheet::tests::shows_retained_earnings_line_only_when_nonzero` (9 → 10),
  `pdf::tests::balance_sheet_pdf_omits_the_calculated_retained_line_only_when_zero` (41 → 42) ; **3 étendus** —
  `csv::tests::balance_sheet_csv_equity_section_by_role` (attente retournée à zéro, cas non nul ajouté),
  `opening_balances_e2e::post_happy_path_od_at_fy_start_appears_in_balance_sheet` (export CSV sans la ligne) et
  `reports_e2e::balance_sheet_next_year_reflects_prior_closing_balances` (export CSV avec la ligne à 4000.00).
  Vitest : `BalanceSheetView.test.ts` 6 → 9 déclarations `it` (dont un `it.each` de trois cas : 5 cas neufs),
  `reports.api.test.ts` 26 → 27.
- **2026-11-18 — sondes par mutation** (`kesh-gate-logs/fix-bilan-resultat-reporte/mutation-*.log`, source rétablie et
  vérifiée après chaque) : `shows_retained_earnings_line` forcé à `true` → 4 échecs (helper, CSV par rôle, PDF,
  `opening_balances_e2e` CSV) ; forcé à `false` → 5 échecs (helper, CSV par rôle, CSV
  `balance_sheet_csv_shows_equity_when_assets_liabilities_net_to_zero`, PDF, `reports_e2e` CSV non nul) ;
  `showsRetainedEarningsLine` forcé à `true` → 4 échecs Vitest (helper et les trois cas zéro du composant).
- **2026-11-18 — gate ciblé, au dernier commit de code** (`f1259fbc`, journaux
  `kesh-gate-logs/fix-bilan-resultat-reporte/`) : `cargo fmt --all -- --check` vert ; `cargo clippy --workspace
  --all-targets -- -D warnings` vert (`clippy.log`) ; `cargo nextest run --profile ci` sur `package(kesh-report)` et,
  de `kesh-api`, `reports_e2e`, `reports_export_e2e`, `opening_balances_e2e`, `fx_recette_api_e2e`,
  `dates_bornes_e2e`, `textes_coherents`, `mcp_outils_e2e` : **431 exécutés, 431 passés, 1 ignoré**
  (`nextest-cible.log`, base `kesh_fix41` migrée et seedée) ; frontend : `npm run check` 0 erreur (29 avertissements,
  aucun dans les fichiers touchés), `lint-i18n-ownership` vert, `test:unit` **4359 passés (271 fichiers)**,
  `npm run build` vert ; **E2E ciblé** (port 3381, base `kesh_e2e_fix41`, `LANG=fr_CH.UTF-8`) :
  `reports.spec.ts`, `reports-export-pdf.spec.ts`, `reports-currency.spec.ts`, `fiscal-year-closing-journey.spec.ts`,
  `opening-balances.spec.ts`, `bank-account-opening-balance.spec.ts` — **22 passés, 1 échec attendu**
  (`opening-balances.spec.ts:156`, de la liste des échecs connus : délai sur un clic du complément, sans rapport)
  (`e2e-cible.log`). **Gate complet groupé à venir** : ni `test-fast.sh --ci`, ni E2E complet, ni `gate-ci.sh` n'ont
  tourné sur cette branche.
- **2026-11-18 — captures avant/après** (`kesh-gate-logs/fix-bilan-resultat-reporte/captures/{avant,apres}/`, thème
  clair ; `captures.sh`/`captures.mjs`) : société `with-company`, soldes d'ouverture avec perte reportée -13'000 sur le
  compte 2970 (rôle `RetainedEarnings`), une vente de 1'200 dans l'exercice. « avant » = backend et frontend de
  `70be12a6` ; « après » = la branche. `01-report-nul-*` : l'écran et le PDF avant portent « Résultat reporté
  (calculé) 0.00 », après non ; totaux identiques (8'200.00 ; 11'200.00) ; le JSON garde `retainedEarnings`
  (`0.0000`) et les mêmes champs. `02-report-non-nul-*` (exercice antérieur avec une vente de 750) : ligne à 750.00
  avant et après, CSV identique octet pour octet.

- **2026-11-18 — revue de code P1 (Sonnet, passe complète) : 0 au-dessus de LOW, 3 LOW**
  (`kesh-gate-logs/fix-bilan-resultat-reporte/revue-p1.md`), **corrigés dans la story** (décision de
  l'orchestrateur), commit `69d20e34` :
  - **LOW-1** — un report non nul sous le centime (`0.004`, `-0.004`) gardait la ligne, affichée « 0.00 » voire
    « -0.00 ». La règle juge désormais la valeur arrondie au centime, l'équidistant loin de zéro :
    `round_for_display` (`pdf.rs`, `pub(crate)`, importé dans `balance_sheet.rs` — `pdf.rs` ne change qu'à la tête
    de boucle, où la valeur écrite passe par lui aussi) ; au CSV, `format_amount_iso(round_for_display(…))` ; à
    l'écran, `roundToCents` extrait de `formatCurrencyFigure` (qui l'emploie, comportement inchangé) et la ligne
    affichée par `formatCurrencyFigure`. Le nom `roundToCents` évite le suffixe `Display`, que la garde des libellés
    en dur (`i18n-libelle-en-dur.test.ts`) prend pour un libellé : nommée `roundForDisplay`, elle comptait une
    candidate de plus (120 pour 119) et rougissait deux tests — relevé au premier Vitest, renommée.
    Tests : `shows_retained_earnings_line_only_when_nonzero` (0.004, -0.004, 0.0049 omis ; 0.005, -0.005
    montrés), le test PDF (0.005 → « 0.01 », -0.005 → « -0.01 », 0.004 et -0.004 absents, aucun « -0.00 »), le test
    CSV par rôle (mêmes cas), Vitest du composant (`it.each` étendu à 0.004 et -0.004, cas 0.005 → « 0.01 ») et du
    helper. Sonde : règle remise à `!is_zero()` → 3 échecs (`mutation-p1fix-regle.log`), source rétablie.
  - **LOW-2** — § Intention corrigée : la 14-1 (l. 102) demandait l'affichage sans condition ; son AC-D (l. 79)
    justifie l'omission au premier exercice. Le paragraphe d'origine est annoté, non réécrit en silence.
  - **LOW-3** — `data-testid="balance-sheet-equity"` sur la table des capitaux propres ; le test la repère par lui
    et vérifie le libellé complet « Bénéfice de l'exercice ».
  Propagation : manuel FR et `docs/api-external.md` (§ 7) disent la règle d'arrondi ; grep de
  `round_for_display`, `roundToCents`, `formatCurrencyFigure`, `showsRetainedEarningsLine` sans autre site.
  **Gate ciblé au commit `69d20e34`**, chaque commande lancée par `heavy.sh` quand aucun autre `heavy.sh` ne
  tournait (`gates-p1fix.sh`) : `cargo fmt --check` vert ; `clippy --workspace --all-targets -D warnings` vert
  (`clippy-p1fix.log`) ; nextest `--profile ci` sur `package(kesh-report)` + `reports_e2e`, `reports_export_e2e`,
  `opening_balances_e2e` : **303 passés, 1 ignoré** (`nextest-p1fix.log`) ; Vitest **4362 passés, 271 fichiers**
  (`front-vitest-p1fix-3.log` ; `-p1fix.log` et `-p1fix-2.log` : les deux rouges de la garde des libellés, avant le
  renommage) ; `npm run check` 0 erreur (`front-check-p1fix-2.log`). Pas d'E2E ni de build relancés. **Gate complet
  groupé à venir.**
  Tendance de la boucle : P1 (Sonnet, complète) 0 au-dessus de LOW, 3 LOW → corrigés ; la remédiation touche la
  production, d'où une **passe P2 ciblée** (`revue-p2-ciblee-prompt.md`).
- 2026-11-18 — passe P2 ciblée (Sonnet, lecture seule) sur `69d20e34` et `ba8b56ac` : **0 au-dessus de LOW**, 2 LOW — LOW-1 `showsRetainedEarningsLine('')` rend `true` (le DTO porte toujours une chaîne décimale, sans effet) ; LOW-2 `balance_sheet.rs` importe `pdf::round_for_display` (dépendance du modèle vers le rendu, à déplacer après la fusion de `fix/pdf-montants-alignes`, tracé). **Boucle close.** Trend : P1 Sonnet 0 > LOW (3 LOW, corrigés) → P2 ciblée Sonnet 0 > LOW. Gate complet groupé à venir, avant tout push.
- 2026-11-18 — **Reprise de `main` après la v0.19.0, puis gate complet groupé.** `origin/main` (`773df80d`, tag
  `v0.19.0`) repris par fusion (`f13fc6d8`), sans rebase ; conflits résolus en gardant les deux intentions (`sprint-status.yaml` : les deux blocs).
  Gate ciblé après la reprise : `fmt`, `clippy` du workspace, nextest de `kesh-report`, `opening_balances_e2e` et `reports_e2e` **283 passés, 1 ignoré**, Vitest `src/lib/features/reports` **126 tests (10 fichiers)**, garde i18n verte (`kesh-gate-logs/fix-bilan-resultat-reporte/reprise-v0190-*.log`). Puis `origin/main` `37c2f875` (documentation seule : `CLAUDE.md`, bilan de la
  0.19.0, `sprint-status.yaml`) repris de même. **Gate complet groupé** sur la branche jetable `integ/fixes-0.19.0`
  (`773df80d` + les quatre correctifs fusionnés dans l'ordre PDF, monnaies retirées, résultat reporté, parent du
  compte ; seul conflit, `sprint-status.yaml`), **au commit `5fc6fb1b`**, base `kesh_integ` remise à zéro :
  `scripts/test-fast.sh --ci` **6165 passés, 14 ignorés** ; frontend `check` 0 erreur (29 avertissements
  préexistants), `lint-i18n-ownership`, `test:unit` **4627 tests (278 fichiers)**, `build` ; **E2E complet**
  (`LANG=fr_CH.UTF-8`, port 3382) **478 passés, 10 échecs, 18 ignorés** — les dix attendus, jugés un à un : sept KF-029
  (#97 : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`,
  `:150`), `backup-restore-new-install.spec.ts:262`, `:381`, `opening-balances.spec.ts:156` (`docs/testing.md`
  inchangé par la 0.19.0) ; `scripts/gate-ci.sh` (copie `git archive`) **6184 passés, 0 échoué, 18 ignorés**.
  Journaux : `kesh-gate-logs/integ-fixes-0.19.0/` (`gate-backend.log`, `front-*.log`, `e2e.log`, `gate-ci.log`) ; un
  premier essai, arrêté par le disque plein de la station, est gardé dans `avorte-disque-plein/`. Le commit de
  documentation `37c2f875` n'a pas été rejoué au gate (aucun code).

## Notes de version

- **Fixed** — Le bilan n'affiche plus la ligne « Résultat reporté (calculé) » quand elle vaut zéro au centime (premier exercice
  tenu dans Kesh, report porté par votre compte de report), à l'écran, au PDF et au CSV ; non nulle, elle reste. Les
  totaux ne changent pas, et l'API (`retainedEarnings`) comme le serveur MCP (`report`) donnent toujours la valeur,
  zéro compris.
