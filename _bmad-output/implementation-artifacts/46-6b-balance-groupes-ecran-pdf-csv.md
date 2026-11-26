# Story 46-6b : balance par groupes — écran, PDF et CSV

Status: done

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D11 modifiée (Q6), D9 étendue (Q10) ; FR-46-6,
FR-46-9, FR-46-10 ; NFR-46-1, NFR-46-5. Release visée **0.21.0** (prévision). Fiche écrite le 2026-11-20 (remarque 2
de Guy), à valider en boucle. **Suit la 46-6a, la 46-2 (composant et contrôles) et la 46-3 (rendu PDF/CSV groupé,
après la 48-11).** Aucune migration. **Repassée en `drafted` le 2026-11-21, redevenue `ready-for-dev` le même jour** : sa phrase de manuel (AC-6)
s'appuie sur C-46-6a-3, choix neuf de la 46-6a dont la validation est en cours ; elle redevient `ready-for-dev` quand
la boucle de la 46-6a se clôt, après une passe sur ses propres lignes touchées.

## Story

En tant que comptable,
je veux voir, imprimer et exporter la balance par groupes, avec les mêmes contrôles qu'au bilan,
afin de contrôler mes comptes au niveau des groupes et de remettre ce que je vois.

## Modules touchés (règle de splitting préventif)

Cinq : `frontend` (vue de la balance, page des rapports), `kesh-report/{pdf,csv}` (balance), `kesh-api/routes/reports`
(paramètres de l'export de la balance), `kesh-i18n`, la documentation (`docs/manual/fr/user-manual.tex`,
`docs/api-external.md`). E2E dans `frontend/tests/e2e`.

## Critères d'acceptation

**Ce que la balance montre aujourd'hui** (relevé du 2026-11-20 à `origin/main`) : **quatre** colonnes de montants —
ouverture, débit, crédit, clôture — à l'écran (`TrialBalanceView.svelte`), au PDF (`render_trial_balance_pdf`) et au
CSV (`NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture`) ; le **solde net des mouvements**
(`balance`) en a été retiré à la 25-5-b (#385) et ne paraît **nulle part** hors du JSON et du MCP (46-6a). Les
extensions ci-dessous portent sur ces quatre colonnes ; aucune cinquième n'est créée.

1. **AC-1 — Écran.** L'onglet Balance gagne les contrôles de la 46-2 (hiérarchie montrée / masquée, niveau, tout
   ouvrir / tout fermer, position des sous-totaux, accordéons, règle d'ouverture, mémoire par visiteur **propre à la
   balance**) et réutilise le composant `GroupedReportRows.svelte` de la 46-2 — **aucune copie** : le composant se
   paramètre par un descripteur de **colonnes** (étiquette et accès au montant), que la 46-2 pose pour le bilan (une
   colonne) et que cette story étend à quatre colonnes et à leurs montants en devise. Hiérarchie masquée : l'écran
   d'aujourd'hui (`TrialBalanceView.svelte`), ligne calculée du résultat reporté et contrôle d'ouverture compris. Par
   défaut, comme au bilan : hiérarchie montrée, `all`, `header` (Q10, décision 5). En mode groupé : la ligne du
   résultat reporté (libellé « Perte reportée » si négatif) vient **après** les groupes, hors d'eux ; un groupe en
   devise unique porte la même seconde ligne en devise qu'un compte (`reports-fx-trial-balance`) ; un sous-total en
   pied s'intitule « Total <nom du groupe> ». Les tests axe de la page, à zéro violation, portent aussi l'onglet Balance par défaut (groupé). Les `data-testid` d'aujourd'hui (`tb-row`, `tb-fx-row-<numéro>`,
   `tb-retained`, `tb-opening-check`, `tb-movements-check`) sont conservés sur les lignes de compte.
2. **AC-2 — PDF et CSV fidèles à l'écran.** `GET /reports/trial-balance/export` (`routes/reports.rs:745`) accepte les
   quatre paramètres (validation de la 46-1), lus comme à la 46-3 AC-1 : second extracteur
   `Query<Vec<(String, String)>>` et `groupes_de(&paires)`, **`ExportQuery` inchangé** (revue P2 de la 46-1, L-6),
   paramètre répété refusé (`400 VALIDATION_ERROR`), ordre `format` → accès → groupes, appel de
   `rapports::balance_par_groupes` (46-6a) ; le test d'intégration qui affirme qu'un paramètre de groupes répété est
   **ignoré** par l'export de la balance (46-3 AC-8, 46-6a AC-9) **bascule** en refus, déclaré au Change Log ; PDF (`render_trial_balance_pdf`, `pdf.rs:1042`) et CSV
   (`render_trial_balance_csv`, `csv.rs:355`) rendent `groupedRows` par **deux fonctions neuves**
   (`render_trial_balance_pdf` et `_csv` à plat restent **inchangées** avec leurs appelants et tests, comme la 46-3 le
   fait pour le bilan), qui reçoivent `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` de la 46-6a et appellent **la fonction de ligne groupée
   de la 46-3**, étendue aux quatre colonnes ; la route choisit selon que `grouping` est présent — au PDF, le groupe en devise unique porte la seconde ligne en petit
   (`write_fx_line`) comme un compte ; en-tête du PDF comme la 46-3 (niveau, et « détail ajusté à la main » si `open` ou `closed` n'est pas vide) ; **au CSV**, deux colonnes
   `Niveau;TypeLigne` en **tête** (la balance n'a pas de colonne `Section`, que la 46-3 AC-5 suit pour le bilan et le
   compte de résultat : elles se placent là où se placerait cette colonne, avant `NumeroCompte`), mêmes valeurs
   (`groupe`, `compte`, `propre`, `soustotal`) et même règle de rendu ; les lignes `Total` et du résultat reporté
   restent ce qu'elles sont, colonnes neuves vides ; quand les colonnes en devise sont présentes (un compte en devise
   au rapport, `csv.rs:364`), une ligne `groupe` ou `soustotal` en devise unique les remplit comme un compte, les autres
   les laissent vides. **Rapport vide** : les rendus groupés reprennent la garde des rendus à plat (`tb.rows.is_empty()`,
   message « rapport vide » au PDF, en-tête seul au CSV), comme la 46-3 pour le bilan. Test de fidélité JSON ↔ PDF comme la 46-3 AC-4. Les boutons
   d'export de l'onglet passent les paramètres de la dernière requête affichée.
3. **AC-3 — Sans paramètre (NFR-46-1).** CSV octet pour octet, PDF aux mêmes lignes de texte, contre des fixtures
   écrites **au commit de base de cette story** (après la 46-3, qui ne touche pas la balance), avant toute modification.
4. **AC-4 — Arrondi.** Fonctions communes posées par la 48-11 seules (noms relevés à T0 : elle n'a pas encore de
   fiche) ; aucun `{:.2}` neuf dans `kesh-report`, tel que sa garde l'interdira.
5. **AC-5 — Totaux.** Total débit = total crédit et contrôle d'ouverture affichés comme aujourd'hui, à tout niveau.
6. **AC-6 — Textes, tests, manuel.** Libellés neufs en quatre locales (bloc `reports-groups-*`), `i18n-keys`
   (`sitesTotal` recompté) et `i18n-libelle-en-dur` verts ; Vitest de la vue (`TrialBalanceView.test.ts` adapté) ; les
   specs d'aujourd'hui qui lisent l'**écran** de la balance — `frontend/tests/e2e/reports.spec.ts` (`tb-row`…),
   `reports-currency.spec.ts` (`tb-fx-row-…`) — passent en « hiérarchie masquée » ou s'adaptent, **sans perdre
   d'assertion** (comme la 46-2 AC-1) ; `opening-balances.spec.ts` ne lit que l'API sans paramètre (`:227`) et ne
   change pas ; E2E neuf (niveau 1, ouverture manuelle, export CSV aux mêmes paramètres) ; captures (thème clair)
   au Dev Agent Record ; manuel utilisateur FR, § *Balance des comptes* (`user-manual.tex:3972`), `.tex` seul : il dit
   aussi qu'à la balance les groupes qui mêlent des comptes de natures contraires sont dissous dès que des comptes des
   deux natures y portent un montant non nul (plan PME : classes 6 à 8 ; un compte à zéro, un compte miroir par
   exemple, ne dissout rien : 46-6a AC-2, C-46-6a-3), que `2 Passifs` y reste un groupe, et qu'un compte de groupe sans solde ni mouvement n'y a pas
   de ligne propre (46-6a AC-3 (d) : un groupe dissous à zéro, `6`, `7`, `70`, `8`, `80` du plan PME, n'y paraît pas).

## Amendement du 2026-11-22 (T0, au code de `epic/46-rapports-groupes` `416547b14`)

La fiche est relue contre le code réel avant le développement ; sept écarts, tranchés ici.

1. **48-11.** Elle n'est pas dans `main` : la balance groupée arrondit par le **remplaçant provisoire posé par la
   46-3**, `kesh_core::types::format_two_decimals` (CSV) et `pdf::format_swiss_amount_arrondi` (PDF), comme le
   bilan groupé ; les rendus à plat gardent `format_swiss_amount` et `format_amount_iso` (AC-3). Aucun `{:.2}` neuf.
2. **« La fonction de ligne groupée de la 46-3, étendue aux quatre colonnes »** (AC-2). Au PDF, `LibelleGroupe`
   (le libellé d'une ligne groupée) lit la ligne par un trait neuf de `groupes.rs`, `LigneGroupeeAffichee`
   (genre, profondeur, numéro, nom, état, montants imprimés, devise), implémenté par `LigneRapport` (un montant)
   et `LigneBalanceGroupee` (quatre : ouverture, débit, crédit, clôture — le net n'est imprimé nulle part) ; la
   règle de rendu de la 46-1 y devient une méthode unique, `porte_montant`. Un `Gabarit` (abscisse et longueur
   du libellé) distingue le bilan (nom à 25 mm, 48 ou 34 caractères) de la balance (intitulé à 20 mm, 22
   caractères, `TB_NAME_MAX_CHARS`, un de moins par 2,5 mm de retrait). Au CSV, `cellules_groupees` sert le
   bilan, le compte de résultat et la balance.
3. **Colonnes en devise du CSV, et « CHF » des en-têtes du PDF, en mode groupé.** La fiche dit « un compte en
   devise au rapport » (`csv.rs:364`, règle à plat). Le rendu groupé suit la règle de la 46-3 pour le bilan : les
   colonnes paraissent si **une ligne groupée** porte une devise. La différence ne joue que pour un compte en
   devise caché sous un groupe fermé qui mêle les devises (aucune ligne en devise visible : pas de colonnes vides).
4. **`data-testid` de l'écran groupé** (AC-1). Une ligne de tableau n'en porte qu'un : en mode groupé, les lignes
   viennent du composant partagé et gardent ses identifiants (`grouped-<genre>-<numéro>`, `group-toggle-<numéro>`) ;
   `tb-row` reste celui des lignes à plat (hiérarchie masquée). Sont conservés en mode groupé : `tb-fx-row-<numéro>`
   (seconde ligne en devise d'un compte), `tb-retained`, `tb-opening-check`, `tb-movements-check`, `tb-col-opening`,
   `tb-col-closing`, et les liens des numéros vers le grand livre. La seconde ligne d'un groupe s'appelle
   `tb-group-fx-row-<genre>-<numéro>`. Les specs d'aujourd'hui (`reports.spec.ts:409`, `reports-currency.spec.ts:155`)
   ne lisent que ces identifiants conservés : elles passent **sans changement** en mode groupé (aucune assertion
   perdue) ; elles ne sont pas jouées cette nuit (règle des gates de la nuit du 22.11 : seules les specs ajoutées).
5. **Composant paramétré** (AC-1). `GroupedReportRows.svelte` devient générique sur le type de ligne
   (`GroupedRowBase`) ; `TRIAL_BALANCE_COLUMNS` (quatre colonnes) s'ajoute à `BALANCE_COLUMNS`, et une
   `currencyRow` facultative (un `snippet`) porte la seconde ligne en devise de la balance, à la place du solde en
   devise à côté du libellé (`AccountCurrencyBalance`) du bilan ; le badge « archivé » de la balance à plat est
   repris (`archivedBadge`).
6. **Info-bulle du niveau.** Celle de `GroupingControls` parle des « sections légales du bilan » ; la balance reçoit
   la sienne (`levelHint`, clé `reports-groups-level-hint-trial-balance`, quatre locales).
7. **Test axe** (AC-1) et **E2E** (AC-6, T5). Le contrôle axe de l'onglet Balance groupé est dans la spec neuve
   `reports-balance-groupes.spec.ts` (seules les specs ajoutées se jouent cette nuit) ; T5 suit la règle de la nuit :
   fmt, clippy, nextest complet, frontend complet, E2E de la seule spec ajoutée ; ni `gate-ci.sh` ni E2E complet
   (à la branche d'intégration).

## Tâches / sous-tâches

- [x] **T0** — Reprise de la branche d'epic (46-1, 46-2, 46-3, 46-4, 46-6a fusionnées ; la 48-11 n'est pas dans
      `main` : remplaçant de la 46-3, amendement point 1).
- [x] **T1** — fixtures sans paramètre (AC-3) ; [x] **T2** — route et deux rendus neufs PDF/CSV (AC 2, 4) ; [x] **T3** — écran (AC 1, 5) ;
      [x] **T4** — i18n, tests, captures, manuel (AC 6) ; [x] **T5** — gates de la nuit du 22.11 (amendement point 7).

## Dev Notes

- `frontend/src/lib/features/reports/TrialBalanceView.svelte` (138 lignes) : colonnes ouverture, débit, crédit,
  clôture, « Perte reportée » si négatif, devise par `formatCurrencyFigure`, seconde ligne en devise.
- `pdf.rs:1042` `render_trial_balance_pdf`, `csv.rs:355` `render_trial_balance_csv`.
- Ne rien recopier de la 46-2 ni de la 46-3 : étendre leurs fonctions et composant (DRY).

### References

Fiches 46-1, 46-2, 46-3, 46-6a.

## Note de version

- **Added** — La balance des comptes s'affiche, s'imprime et s'exporte **par groupes de comptes**, comme le bilan :
  même barre de présentation (hiérarchie, niveau de détail, tout ouvrir, tout fermer, sous-totaux sur le titre ou
  en pied, groupes ouverts ou fermés d'un clic), avec sa propre mémoire ; chaque groupe totalise ouverture, débit,
  crédit et clôture de ses comptes, et un groupe en devise unique a sa seconde ligne en devise. Les exports PDF et
  CSV reprennent la présentation affichée (CSV : colonnes `Niveau;TypeLigne` en tête) ; sans présentation par
  groupes, ils restent ceux d'avant. Par l'API, l'export sans paramètre est groupé comme l'écran, et l'export d'avant se
  demande par `detailLevel=flat` (story 46-8, annoté le 2026-11-24).
- **Changed** — La balance s'affiche désormais groupée par défaut (tous les comptes), et l'export PDF ou CSV lancé
  depuis l'écran suit la vue affichée : le CSV de la balance change de forme sans geste de l'utilisateur, avec les
  colonnes `Niveau;TypeLigne` en tête et les sous-totaux. Le CSV groupé de la balance ne s'additionne pas tel quel :
  le titre d'un groupe (en pied, sa ligne de sous-total) répète les montants de ses comptes, et un groupe fermé n'y porte que son
  sous-total ; la balance porte sa ligne `Total` ; pour additionner les montants dans un tableur, décocher
  « Hiérarchie des groupes » pour l'export à plat d'avant, qui porte lui aussi sa ligne de total (à écarter).
- **Changed (API)** — `GET /reports/trial-balance/export` lit `detailLevel`, `subtotalPosition`, `open` et
  `closed`, qu'il ignorait jusqu'ici, avec les refus de la balance (`400 VALIDATION_ERROR`, message qui nomme le
  paramètre) : un de ces paramètres répété ; une valeur invalide (`detailLevel` qui n'est ni un entier de 1 à 20 ni
  `all`, `subtotalPosition` autre que `header` ou `footer`, un élément de `open` ou `closed` qui n'est pas un
  entier, un même identifiant dans les deux listes, plus de 500 identifiants) ; `subtotalPosition`, `open` ou
  `closed` sans `detailLevel`. Un export qui passait avec ces paramètres invalides est désormais refusé.
  *(Annoté le 2026-11-24, story 46-8 : `subtotalPosition`, `open` ou `closed` sans `detailLevel` sont désormais
  acceptés à l'export, groupé par défaut au niveau `all` ; ils sont refusés avec `detailLevel=flat`.)*

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`), 2026-11-22.

### Debug Log References

Journaux et captures : `kesh-gate-logs/s46-6b/` (hors dépôt).

- T1 : `t1-ecrire.log`, `t1-relire.log` (fixtures écrites puis relues, 2/2, au commit `4fa332461`, avant tout changement de rendu).
- Gate ciblé : `cible-a.log` (403/405 : deux tests à basculer ou à corriger, repris ci-dessous).
- Gate backend complet (`scripts/test-fast.sh --ci`, base remise à zéro) au dernier commit de code `b9f722845` :
  `gate-backend-final3.log` — fmt et clippy verts, **nextest 6256 exécutés, 6256 passés, 14 ignorés**. Deux runs
  précédents rouges, corrigés : `gate-backend-final.log` (fmt d'un test), `gate-backend-final2.log` (garde
  `textes_coherents::la_forme_libre_nnnn_nom_est_juste_partout` : « 1025 Banque USD » d'un test, numéro absent des
  plans livrés — jeu d'essai renuméroté 1000/1020).
- Frontend complet à `b9f722845` (`front-final.console.log`) : `npm run check` 0 erreur (29 avertissements, 5 fichiers,
  aucun dans le code touché hormis deux `state_referenced_locally` d'avant dans `+page.svelte`),
  `lint-i18n-ownership` vert, **Vitest 4719/4719 (283 fichiers)**, `npm run build` vert.
- **E2E partiel (1 spec)** : `reports-balance-groupes.spec.ts`, la seule spec ajoutée (règle de la nuit du 22.11),
  1/1 vert (`e2e.log`), binaire construit à `f09b8a6b6` (les commits suivants ne touchent que des tests Rust et la
  fiche). E2E complet et `gate-ci.sh` à la branche d'intégration. Les specs `reports.spec.ts` et
  `reports-currency.spec.ts` ne sont pas jouées (amendement point 4 : aucun de leurs identifiants n'a changé).
- Mutations (`mut/*.log`, copie hors dépôt, cible propre supprimée, témoin vert d'abord, tuée seulement sur assertion
  nommée) : **5/5 tuées** — M1 `porte_montant` sans la condition de pied (tuée par
  `pdf::tests::balance_groupee_footer_et_groupe_ferme`) ; M2 CSV groupé sans arrondi (`format_amount_iso`, tuée par
  `csv::tests::balance_csv_groupee_header`) ; M3 seconde ligne en devise retirée des groupes (tuée par
  `balance_groupee_groupe_en_devise_unique`) ; M4 gabarit de la balance à 48 caractères (tuée par
  `balance_groupee_libelle_tronque_a_sa_colonne`) ; M5 (Vitest, copie avec son propre `node_modules`) vue groupée
  désactivée dans `TrialBalanceView.svelte` (six tests de « vue groupée » en `AssertionError`). Non muté : la route
  (sa bascule est couverte par `export_balance_lit_les_parametres_de_groupes` et la fidélité JSON ↔ PDF/CSV).
- Manuel : `user-manual.tex` compilé dans une copie (`latexmk -xelatex`, rc 0), PDF aplati et grepé : les cinq phrases
  neuves présentes (`manuel.txt`) ; copie supprimée, aucun PDF commité.

### Completion Notes List

- **Écran** : la balance a la barre de la 46-2 (sa propre mémoire `kesh-report-grouping:<société>:trial-balance`,
  défaut hiérarchie montrée, `all`, `header`) et le composant `GroupedReportRows.svelte`, rendu générique
  (`GroupedRowBase`), à quatre colonnes (`TRIAL_BALANCE_COLUMNS`) ; seconde ligne en devise d'un compte et d'un groupe
  en devise unique (`currencyRow`) ; badge « archivé » ; résultat reporté après les groupes ; en-tête, totaux et
  contrôles inchangés ; info-bulle du niveau propre à la balance. Page : `trial-balance` entre dans les mécanismes de
  la 46-2 (`isGroupedReport`, `displayedReport`, `acceptGrouped`, export aux paramètres affichés).
- **Export** : `GET /reports/trial-balance/export` lit les quatre paramètres (`groupes_de`, `ExportQuery` inchangé,
  ordre format → accès → groupes, `balance_par_groupes`) ; `render_grouped_trial_balance_pdf` et `_csv`, neufs,
  rendent `groupedRows` ; les rendus à plat sont inchangés (fixtures de T1). Le libellé des lignes groupées
  (`LibelleGroupe`, PDF) et les cellules (`cellules_groupees`, CSV) sont **partagés** avec le bilan par le trait
  `LigneGroupeeAffichee` (`groupes.rs`) : la règle de rendu de la 46-1 n'est plus écrite qu'une fois
  (`porte_montant`). Arrondi : remplaçant de la 46-3 ; aucun `{:.2}` neuf.
- **Bascule déclarée** : `reports_e2e::groupes_parametre_repete` et le test de la 46-3
  `export_balance_ignore_les_parametres_de_groupes` (remplacé par `export_balance_lit_les_parametres_de_groupes`)
  affirmaient que l'export de la balance **ignore** un paramètre répété : il le **refuse** désormais (`400
  VALIDATION_ERROR`) ; la balance entre dans `export_groupes_refus_du_contrat`.
- **Limite constatée, assumée** : au PDF, la colonne de l'intitulé de la balance (22 caractères, comme à plat) abrège
  les noms longs en retrait ; le manuel le dit. L'élargir déplacerait les colonnes de montant du rendu à plat (AC-3).
  *(Annoté le 2026-11-22, remédiation de la revue P1, MEDIUM-1 : elle abrégeait aussi les sous-totaux au point de les
  confondre — « Total Capitaux étrangers à court terme » et « … à long terme » en « Total Capitaux étra… » ; un
  sous-total, sans numéro, part désormais de la colonne du numéro, 30 caractères au retrait nul.)*
- **Tests ajoutés** (recomptés de `416547b14` à `b9f722845`, `#[test]`/`#[sqlx::test]` et `it(`) : `pdf.rs` 72 → 78
  (+6), `csv.rs` 36 → 42 (+6), `reports_export_e2e.rs` 24 → 26 (+3 neufs, −1 remplacé ; le test de fixtures de la
  46-3 factorisé avec le neuf), `reports_e2e.rs` 47 → 47 (un test basculé) ; Vitest `TrialBalanceView.test.ts` 9 → 15
  (+6), `reports-page-groupes.test.ts` 24 → 28 (+4), `reports.api.groupes.test.ts` 9 → 11 (+2) ; E2E : une spec neuve
  (un test, contrôle axe compris). Compteurs des gardes relevés : `sitesTotal` 3515 → 3517,
  `CANDIDATES_ATTENDUES` 131 → 132.
- **Captures** (thème clair, `kesh-gate-logs/s46-6b/captures/`) : `46-6b-1-defaut-tous-les-comptes.png`,
  `-2-sous-totaux-en-pied.png`, `-3-niveau-1.png`, `-4-groupe-ouvert-a-la-main.png`, `-7-hierarchie-masquee.png`, et
  les exports au niveau 1 avec un groupe ouvert à la main : `-5-export-niveau-1-ouvert.csv`,
  `-6-export-niveau-1-ouvert.pdf` (rendu `-6-export-pdf-1.png`).

### File List

- `crates/kesh-report/src/groupes.rs` — trait `LigneGroupeeAffichee`, implémenté par `LigneRapport` et `LigneBalanceGroupee`.
- `crates/kesh-report/src/pdf.rs` — `Gabarit`, `LibelleGroupe` sur le trait, `limite_nom_groupe_balance`,
  `render_grouped_trial_balance_pdf`, `rendre_balance_pdf`, `draw_tb_amounts`, `tb_fx_text`, `draw_tb_flat_rows`,
  `draw_tb_grouped_rows` ; tests.
- `crates/kesh-report/src/csv.rs` — `cellules_groupees`, `render_grouped_trial_balance_csv`, `rendre_balance_csv`,
  `write_tb_record` ; tests.
- `crates/kesh-report/src/essai_groupes.rs`, `crates/kesh-report/src/lib.rs`.
- `crates/kesh-api/src/routes/reports.rs` — `export_trial_balance`.
- `crates/kesh-api/tests/reports_export_e2e.rs`, `crates/kesh-api/tests/reports_e2e.rs`,
  `crates/kesh-api/tests/fixtures/rapports-groupes/export-balance-{2025,2026}.{csv,pdf.txt}` (neufs).
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — `reports-groups-level-hint-trial-balance`.
- `frontend/src/lib/features/reports/{GroupedReportRows,GroupingControls,TrialBalanceView}.svelte`,
  `reports.types.ts`, `reports.api.ts`, `reports-grouping.ts`, `TrialBalanceView.test.ts`,
  `reports.api.groupes.test.ts` ; `frontend/src/routes/(app)/reports/+page.svelte`, `reports-page-groupes.test.ts` ;
  `frontend/src/lib/shared/{i18n-keys,i18n-libelle-en-dur}.test.ts` ;
  `frontend/tests/e2e/reports-balance-groupes.spec.ts` (neuf).
- `docs/manual/fr/user-manual.tex`, `docs/api-external.md`.
- `_bmad-output/implementation-artifacts/46-6b-balance-groupes-ecran-pdf-csv.md`, `sprint-status.yaml`.

## Change Log

- 2026-11-20 — Fiche écrite (remarque 2 de Guy) ; découpée de la « 46-6 » de la revue 1 (règle de splitting
  préventif). Six critères. `drafted`.
- 2026-11-20 — **validation P1 (Sonnet 5.5) : 0 C, 1 H, 1 M, 1 L, remédiés.** H : « cinq colonnes » (ouverture, débit,
  crédit, solde, clôture) — l'écran, le PDF et le CSV n'en montrent que **quatre**, le solde net ayant été retiré à la
  25-5-b : critères, CSV et fixtures recadrés sur les quatre. M : placement de `Niveau;TypeLigne` (la balance n'a pas
  de `Section`), composant paramétré par colonnes, résultat reporté et seconde ligne en devise d'un groupe, specs
  d'aujourd'hui à adapter (`reports.spec.ts`, `reports-currency.spec.ts`, `opening-balances.spec.ts`,
  `TrialBalanceView.test.ts`), conséquence du plan PME écrite au manuel. L : 48-11 sans fiche (noms et garde à relever
  à T0). Axes exercés : `TrialBalanceView.svelte`, `csv.rs`, `pdf.rs`, `reports/+page.svelte`, 46-2/46-3 ; non
  exercés : PDF compilé du manuel, rendu réel.
- 2026-11-20 — **validation P2 (Claude Opus 5.5, contexte frais, au code de `origin/main` `d29aa557`) : 0 C, 0 H, 0 M,
  4 L, remédiés.** L : enveloppe `AvecGroupes<TrialBalance, Vec<LigneBalanceGroupee>>` ; colonnes en devise du CSV
  (`csv.rs:364`) pour un groupe en devise unique, et garde « rapport vide » (`tb.rows.is_empty()`) des rendus groupés,
  comme la 46-3 ; `opening-balances.spec.ts` ne lit que l'API sans paramètre (`:227`), il ne change pas ; manuel : les
  groupes à zéro sans ligne propre (46-6a AC-3 (d)). Rapport : `kesh-gate-logs/epic-46/validation/46-6b-P2.md`.
- 2026-11-21 — Propagation de la correction de la 46-6a (C-46-6a-3 : une ligne nulle ne compte pas pour la
  dissolution par nature) à l'AC-6 (phrase du manuel). Aucune autre modification.
- 2026-11-21 — Validation P1 de la nouvelle boucle des 46-3 et 46-6a (F11, `kesh-gate-logs/s46-specs/validation-p1-rapport.md`) :
  repassée en `drafted` (phrase de manuel de l'AC-6 adossée à C-46-6a-3, en validation) ; AC-2 complété (lecture par
  `groupes_de` et second extracteur, `ExportQuery` inchangé, paramètre répété refusé, ordre des contrôles, appel de
  `balance_par_groupes`, bascule du test « paramètre répété ignoré par l'export de la balance »).
- 2026-11-21 — Redevient `ready-for-dev` : la boucle de la 46-6a est close (P1 Sonnet, P2 Opus, 0/0/0/10), C-46-6a-3 confirmé ; ses lignes touchées ont été relues à la P2.
- 2026-11-22 — Développement (`bmad-dev-story`) : amendement daté (sept écarts au code réel), T1 à T5. Fixtures au
  commit de base (`4fa332461`), rendu (`f09b8a6b6`), corrections de tests (`1539e1e5d`, `b9f722845`). Gates au dernier
  commit de code `b9f722845` : nextest 6256/6256, Vitest 4719/4719, check, lint, build verts ; E2E partiel (1 spec)
  vert ; 5 mutations tuées. Bascule déclarée : l'export de la balance refuse un paramètre de groupes répété, qu'il
  ignorait (tests `reports_e2e::groupes_parametre_repete` et `reports_export_e2e`). Story en `review` ; prompt de la
  revue P1 : `kesh-gate-logs/s46-6b/revue-p1-prompt.md`.
- 2026-11-22 — **Revue de code P1 (passe complète, une lentille) : 0 C, 0 H, 1 M, 2 L, remédiés** (rapport
  `kesh-gate-logs/s46-6b/revue-p1-rapport.md`). **M-1** : au PDF groupé, la colonne de l'intitulé (22 caractères)
  abrégeait deux sous-totaux à l'identique (« Total Capitaux étrangers à court terme » et « … à long terme » en
  « Total Capitaux étra… ») ; une ligne **sans numéro** (sous-total, ou titre au numéro vide) part désormais de la
  colonne du numéro et dispose de 30 caractères au retrait nul (`Gabarit::x_sans_numero_mm`,
  `nom_max_sans_numero`, `TB_NAME_MAX_CHARS_SANS_NUMERO` ; `limite_nom_groupe_balance(depth, avec_numero)`) ; le
  bilan, le compte de résultat et la balance à plat ne bougent pas (fixtures `export-balance-*` inchangées). 30
  suffit : recalculé sur les trois plans livrés, quatre langues, aucun sous-total n'y est plus abrégé à l'identique
  d'un autre (à 22 : `20`/`24` en français et en italien) ; `nom_max` des lignes numérotées n'est donc pas élargi.
  **L-1** : note de version, tous les refus neufs de l'export. **L-2** : `balanceCurrency` remonté dans
  `GroupedRowBase`, le transtypage `as unknown as GroupedRow` retiré. Manuel (limites des sous-totaux : 30, 29, 27,
  26, 24, 22) et `docs/api-external.md` suivent. Tests (de `3fa6cded9` à `c44edcf7b`, `#[test]`) : `pdf.rs` 78 → 79
  (`balance_groupee_sous_totaux_longs_distincts`, neuf : textes extraits et abscisses), `balance_groupee_footer_et_groupe_ferme`
  (« Total Actifs circulants » entier), `balance_groupee_libelle_tronque_a_sa_colonne` et
  `les_colonnes_de_montant_ne_se_chevauchent_pas` complétés ; `export_balance_groupee_fidele_au_json` suit la limite
  par genre de ligne. Mutations (`mut-r1.console.log`, témoin vert) : **3/3 tuées** par
  `pdf::tests::balance_groupee_sous_totaux_longs_distincts` — départ sans numéro remis à l'intitulé, longueur remise
  à 22, numéro vide non filtré. Gates au dernier commit de code `c44edcf7b` : fmt, clippy verts, **nextest 6257
  exécutés, 6257 passés** (1 instable au premier essai, `mcp_transport_e2e::abandon_au_delai_le_permis_reste_pris`,
  sans rapport), 14 ignorés (`gate-backend-r1.log`) ; frontend : check 0 erreur, lint, **Vitest 4719/4719 (283
  fichiers)**, build verts ; **E2E partiel (1 spec)** `reports-balance-groupes.spec.ts` 1/1 (`e2e.log`) ; manuel
  compilé dans une copie (`manuel-r1.txt`, phrase neuve présente). Captures reprises par la spec au binaire de `c44edcf7b` (1 à 7 ; rendu `-6-export-pdf-1.png`
  refait) ; ajoutées : `46-6b-6b-sous-totaux-longs.pdf` (le jeu du test neuf, sous-totaux en pied) et son rendu
  `-6b-sous-totaux-longs-1.png` — « Total Capitaux étrangers à c… » et « … à l… », sans chevauchement des montants. Prompt de la P2 (complète, Opus) : `kesh-gate-logs/s46-6b/revue-p2-prompt.md`.
- 2026-11-22 — **Revue de code P2 (passe complète, une lentille, Claude Opus 5.5, contexte frais, au code de `d462c079f`) : 0 C, 0 H, 0 M, 3 L** (rapport `kesh-gate-logs/s46-6b/revue-p2-rapport.md`) ; tendance P1 Sonnet 5.5 0/0/1/2 → P2 Opus 5.5 0/0/0/3. **LOW-1** et **LOW-2** remédiés par des tests seuls, aucune ligne de production : `pdf::tests::bilan_groupe_sous_total_garde_depart_et_longueur_du_bilan` (neuf : abscisse et longueur du sous-total du bilan aux niveaux 1 à 3, 45, 47, 48 caractères) et `les_colonnes_de_montant_ne_se_chevauchent_pas` (la ligne sans numéro de la balance groupée, avec et sans numéro, aux niveaux 1 à 8, mesurée par `largeur_mm` sur des « E » gras à la limite, contre le début du plus long montant ; l'égalité par construction `2.5 * n` est retirée). `#[test]` de `pdf.rs` : 79 → 80 (de `c44edcf7b` à `HEAD`). Mutations jouées une à une sur une copie hors dépôt (cible distincte, témoin vert, `cargo test -p kesh-report --lib`) : A `x_sans_numero_mm: 25.0 → 0.0` tuée par `bilan_groupe_sous_total_garde_depart_et_longueur_du_bilan` (abscisse) ; B `nom_max_sans_numero: nom_max + 8` tuée par la même (longueur) ; en plus, `TB_NAME_MAX_CHARS_SANS_NUMERO + 12` tuée par `les_colonnes_de_montant_ne_se_chevauchent_pas`, `balance_groupee_libelle_tronque_a_sa_colonne` et `balance_groupee_sous_totaux_longs_distincts`. **LOW-3** (premier compte seul en haut de page quand une ligne a sa seconde ligne en devise) touche la production, antérieur à la P1 : à ouvrir en issue (bug, P4), consigné dans `kesh-gate-logs/epic-38-kiss/issues-a-ouvrir.md`. Boucle close : aucun finding au-dessus de LOW, remédiation sans production. Story `done`. Gate ciblé au dernier commit de test `d3523b2aa` (aucune ligne de production changée depuis `c44edcf7b`) : fmt et clippy workspace verts ; nextest `package(kesh-report) | binary(reports_export_e2e)` **318 exécutés, 318 passés, 1 ignoré** (`kesh-gate-logs/s46-6b/p2-final2.log`, base remise à zéro ; un premier essai, `p2-final.log`, a échoué en « Unknown database » sur la base de gate disparue, sans rapport).
- 2026-11-26 — **Suivis ouverts** (dette de la story, non réglée par la PR de l'epic) : #1642 (LOW-3 de la revue P2 : le premier compte d'un groupe seul en haut de la page suivante du PDF).
