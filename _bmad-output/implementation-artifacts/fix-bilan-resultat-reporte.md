# Correctif : le bilan affiche « Résultat reporté (calculé) » à 0.00 à côté du compte de report

Status: review

*Fiche rédigée le 2026-11-18, branche `fix/bilan-resultat-reporte`, partie de `main` à `70be12a6` (squash de
l'Epic 37, PR #1247). Correctif hors epic, **non urgent** : il ne passe pas par `hotfix/` et va à la prochaine release
planifiée. **Aucune migration.** Relevé par Beat à la recette d'un exercice réel 2024
(`data-not-committed/2024-qesh/sorties-kesh/bilan-2024.pdf`).*

## Intention

Le bilan d'une société qui tient son premier exercice dans Kesh affichait, dans la section *Capitaux propres*, son
compte de report (rôle `RetainedEarnings`, ici « 2970 Bénéfice / perte reporté », -37'571.82) **et**, juste dessous,
la ligne calculée « Résultat reporté (calculé) » à **0.00**. À zéro, cette ligne n'apporte rien et trouble le lecteur :
deux « résultats reportés » côte à côte, dont l'un nul (l'ancien logiciel de Beat ne l'avait pas).

**Prémisse vérifiée au code** (`origin/main` `70be12a6`) :

- `BalanceSheet::retained_earnings` (`crates/kesh-report/src/balance_sheet.rs`) est le cumul des résultats des
  exercices **strictement antérieurs** tenus dans Kesh (P&L `entry_date < fy_start`) : nul au premier exercice, où le
  report est porté par le compte physique (soldes d'ouverture).
- La ligne était dessinée **sans condition** : au PDF (`render_balance_sheet_pdf`, boucle des deux lignes calculées),
  au CSV (`render_balance_sheet_csv`, `write_fx_record` inconditionnel) et à l'écran
  (`BalanceSheetView.svelte`, `<tr>` inconditionnelle). Le PDF de Beat le montre (`pdftotext` : « Résultat reporté
  (calculé) 0.00 » sous « 2970 Bénéfice / perte reporté -37'571.82 »).
- **Pourquoi elle était toujours affichée.** La Story 14-1 l'a créée comme report à-nouveau virtuel ; la décision D1 de
  la Story 14-3c (`14-3c-fonds-propres-par-role.md`, « Collision Résultat reporté : DISTINGUER nettement ») impose de
  ne **jamais fusionner** le compte physique `RetainedEarnings` et la ligne calculée, et de marquer celle-ci
  « (calculé) ». D1 n'exige pas d'afficher la ligne nulle : elle traite le cas où **les deux** valeurs coexistent. Le
  compte physique, lui, n'est listé que non nul (`HAVING balance != 0`). Aucun texte de 14-1 ni de 14-3c ne motive
  l'affichage d'un zéro ; le seul garde-fou lié est la garde « rapport vide » (`is_empty`, revue P1 de la 14-1), qui
  reste inchangée.

## Décision

1. **Écran et PDF** : la ligne calculée n'est affichée que si `retained_earnings` est **non nul** (positif : bénéfice
   reporté ; négatif : « Perte reportée » à l'écran). Non nulle, elle reste : elle signale un résultat d'exercices
   antérieurs qu'aucune écriture n'a encore reporté (et D1 s'applique toujours : deux lignes distinctes, jamais
   fusionnées). « Résultat de l'exercice » reste toujours affiché. Le total des capitaux propres et la ligne
   « Total passifs + Capitaux propres » sont inchangés (on omet un zéro) : l'équation affichée reste juste.
2. **Source unique** : `BalanceSheet::shows_retained_earnings_line()` (backend, PDF et CSV) ; pendant frontend
   `showsRetainedEarningsLine()` (`reports.api.ts`), tenu en parallèle comme `isReportEmpty` — même règle, une
   valeur illisible (`NaN`) restant affichée comme avant.
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
   tests (après `balance_sheet_pdf_shows_equity_when_assets_liabilities_net_to_zero`, l. 2451–2483), non à sa fin où
   l'autre branche ajoute les siens.

## Critères d'acceptation

1. **AC-1** — Bilan dont le résultat reporté calculé est nul : la ligne « Résultat reporté (calculé) » n'apparaît ni à
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

- [x] Prémisse vérifiée (code, PDF de Beat, 14-1 et 14-3c/D1)
- [x] `BalanceSheet::shows_retained_earnings_line` + test unitaire
- [x] PDF et CSV : ligne omise à zéro ; tests unitaires (absente à zéro, présente non nulle, positive et négative)
- [x] Écran : `showsRetainedEarningsLine` + `{#if}` ; Vitest (composant et helper)
- [x] Intégration HTTP : CSV sans la ligne à zéro (scénario soldes d'ouverture sur 2970), avec la ligne non nulle
      (deux exercices)
- [x] Manuel FR (`user-manual.tex`), `docs/api-external.md`
- [x] Gates ciblés, mutations, captures (ci-dessous)
- [ ] Gate complet (`test-fast.sh --ci`, E2E complet, `gate-ci.sh`) : **groupé, à venir** sur une branche d'intégration
      (décision de Beat du 2026-11-18)
- [ ] Revue de code P1 (prompt : `kesh-gate-logs/fix-bilan-resultat-reporte/revue-p1-prompt.md`)

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

## Notes de version

- **Fixed** — Le bilan n'affiche plus la ligne « Résultat reporté (calculé) » quand elle vaut zéro (premier exercice
  tenu dans Kesh, report porté par votre compte de report), à l'écran, au PDF et au CSV ; non nulle, elle reste. Les
  totaux ne changent pas, et l'API (`retainedEarnings`) comme le serveur MCP (`report`) donnent toujours la valeur,
  zéro compris.
