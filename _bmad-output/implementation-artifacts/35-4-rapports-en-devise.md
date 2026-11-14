# Story 35-4 : rapports dans les deux devises

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, D2, C-35-3. Suit 35-3a.
**Aucune migration attendue** (colonnes de 35-1 et 35-3a ; aucun index nécessaire : les sommes passent par
`account_id`).*

## Story

En tant que comptable en multi-devises, je veux voir, pour chaque compte en devise, le solde et les mouvements en
devise à côté du CHF, dans le grand livre, la balance, le bilan et le compte de résultat, à l'écran et dans les
exports, afin de rapprocher mes comptes PayPal ou Wise de leurs relevés et de contrôler ma réévaluation.

## Critères d'acceptation

1. **AC-1 — Règle unique.** Le solde en devise d'un compte = Σ `amount_currency` au débit − Σ au crédit, signé comme le
   CHF par le type du compte (`opening.rs`).
   - **Côté d'une ligne** : celui de son montant CHF (`debit > 0` ⇒ débit, `credit > 0` ⇒ crédit) ; `amount_currency`
     n'a pas de signe (C-35-3). Les rapports lisent une **paire brute (débit, crédit) en devise** :
     `SUM(CASE WHEN jel.debit > 0 THEN jel.amount_currency ELSE 0 END)` et son symétrique, **sans condition de
     devise sur la ligne** — CR-4, T3 : la ligne ne porte plus de devise (35-3a AC-1) ; la **devise est celle du
     compte**, `a.currency`, et toutes les lignes d'un compte en devise sont dans sa devise (verrou de 35-1 AC-4 dès la
     première ligne, contrôle central de 35-3a AC-4). Aucune condition `jel.currency = a.currency` ne s'écrit nulle
     part (la colonne n'existe pas). Un compte CHF ne porte aucune ligne en devise (`amount_currency` `NULL`, ignoré
     par `SUM`) ; le solde en devise d'un compte CHF reste `NULL` par `a.currency IS NULL` (ci-dessous), ce qui
     neutralise aussi une ligne en devise posée hors du dépôt (limite de 35-3a AC-1). Une ligne `revaluation`
     (`amount_currency = 0`) n'y ajoute rien. Chaque somme en devise est enveloppée `COALESCE(…, 0)`, comme le CHF (un
     compte sans ligne ne rend pas `NULL` là où le compte est en devise).
   - **La devise gouverne l'affichage, pas la ligne** : au grand livre, `fetch_lines` lit `amount_currency` pour toute
     ligne, mais la ligne, le solde progressif, l'ouverture, la clôture et les totaux en devise ne sont remplis **que si
     la section est en devise** (`LedgerSection.currency` = `a.currency` non `NULL`) ; au journal, les champs en devise
     d'une ligne ne sont remplis que si `a.currency` **et** `jel.amount_currency` sont non `NULL` (une ligne
     `revaluation`, montant 0, s'y lit comme au grand livre, AC-2 : « Réévaluation au cours X », sans montant en devise
     affiché) ; à la balance, les champs en devise d'une ligne ne sont remplis que si `a.currency` est non `NULL` (AC-3) ;
     au bilan et au compte de résultat, par le `CASE WHEN a.currency IS NULL` ci-dessous. Sans cela, une ligne
     en devise posée hors du dépôt sur un compte CHF s'afficherait sans devise (test T3, AC-8).
   - `signed`, `opening_from` et `debit_sense` (`opening.rs`, :75 pour `opening_from`) s'appliquent **sans
     changement** à la paire en devise (ce sont des fonctions sur `Decimal`) : aucun `CASE` d'ouverture n'est recopié
     en SQL, aucune seconde règle. « Fonction partagée » = fragment SQL de la somme en devise + `opening_from` ; la
     balance lit déjà 18 paramètres positionnels (`trial_balance.rs:178-195`), 12 de plus avec les trois paires en
     devise : un test exerce l'ordre des `bind`.
   - **Où vit le fragment, et sur quelles requêtes il s'écrit** : les requêtes n'ont pas la même forme — celles du
     grand livre (`opening_balance`, `general_ledger.rs:338-342` ; `fetch_totals`, `:465-473`) ne joignent **pas**
     `accounts` (filtre `jel.account_id = ?`), et la balance lit les lignes par la sous-requête `x`
     (`trial_balance.rs:165-170`), qui ne projette que `account_id, debit, credit, entry_date, fiscal_year_id`. Le
     fragment est donc une fonction **paramétrée** par l'alias de la ligne (T3 : plus d'expression de devise à lui
     passer), et il vit dans `kesh-db` (`repositories/account_balances.rs`, AC-1 quater), seul crate visible à la fois de
     `balances_as_of` et de `kesh-report` ; `x` projette en plus `jel.amount_currency` (seulement) ; les deux
     requêtes du grand livre **ne joignent pas** `accounts` pour la somme : la devise de la section est celle que lit
     `select_accounts` (`a.currency`, AC-2), et la paire en devise se calcule sans elle.
   - **Bilan et compte de résultat : le signe est en SQL**, non en Rust — `AccountBalance` est un `sqlx::FromRow` dont
     `balance` vient d'un `sign_expr` par type (`balance_sheet.rs:288-292`, `income_statement.rs:62-66`), pour que
     `HAVING` le lise. Le solde en devise y suit **le même patron** : un `sign_expr` en devise construit par la même
     `match` sur le type, à partir du fragment de somme en devise, et enveloppé
     `CASE WHEN a.currency IS NULL THEN NULL ELSE … END AS balance_currency` — `NULL`, jamais `0`, pour un compte CHF
     (sinon le JSON ne serait plus identique, AC-6). Un test croise ce solde avec `signed` appliqué à la paire brute.
   - Les sommes CHF existantes **ne changent pas**. Un compte CHF a `currency = None` et aucun champ en devise.
   - **AC-1 bis — Compte de RÉSULTAT en devise, solde borné par exercice.** Il suit les **mêmes bornes** que le CHF :
     ouverture = Σ depuis le début de l'exercice contenant `from` (paire `since_fy` en devise passée à `opening_from`) ;
     solde progressif en devise remis à zéro à chaque rupture d'exercice, **dans le même `Cumul`** que le CHF (un état
     `running` par devise, lignes sautées par `offset` comprises, #775) ; `FiscalYearBreak` porte
     `closingBalanceCurrency` ; la clôture en devise d'un compte de résultat qui franchit un exercice est le solde de
     l'exercice final (sommes en devise de `fetch_totals` filtrées sur `fiscal_year_id = exercice_final`, #778), pas
     `ouverture + totaux`. Compte de bilan : chaîne continue, tous exercices. Balance : `before_*`, `since_fy_*`,
     `period_*` en devise. Compte de résultat (rapport) : `fiscal_year_id` ET dates, comme le CHF.
   - **AC-1 ter — Inclusion.** Un compte est listé si son solde CHF **ou** son solde en devise est non nul
     (`HAVING balance != 0 OR balance_currency != 0` au bilan et au compte de résultat ; ouverture CHF ou devise non
     nulle pour le filtre du grand livre — `generate`, `general_ledger.rs:687`
     (`line_count == 0 && opening.is_zero()`), non `select_accounts`, qui ne filtre rien — et pour la règle d'inclusion
     de la balance, `trial_balance.rs:209`, qui ne vise que les comptes archivés). Cas : conversion qui laisse un
     compte à CHF 0 et USD ≠ 0. `BalanceSheet::is_empty()` reste sur les listes.
   - **AC-1 quater — Fonction à date pour 35-5.** ⚠️ **Elle vit dans `kesh-db`**, non dans `kesh-report` :
     `fx_revaluation::post` (35-5) est une fonction de dépôt de `kesh-db`, et `kesh-db` ne dépend de `kesh-report` qu'en
     `[dev-dependencies]` (`crates/kesh-db/Cargo.toml:48`) — l'inverse (`kesh-report/Cargo.toml:9`) ferait un cycle.
     `kesh_db::repositories::account_balances::balances_as_of(conn: &mut MySqlConnection, company_id, as_of,
     account_ids: Option<&[i64]>)` → `Vec<(account_id, chf_net, Option<devise_net>)>`, en **sens débit brut**
     (Σ débit − Σ crédit, et la paire en devise de l'AC-1), **sans signe par type** — la grandeur même de la formule de
     35-5 (AC-1 de 35-5), et `opening.rs` n'est pas visible de `kesh-db`. Règles :
     - borne **inclusive** `je.entry_date <= as_of`, tous exercices, **sans borne basse ni `HAVING`** : une
       réévaluation déjà passée au même jour est comptée, ce qui rend le rejeu de 35-5 (résidu seul) exact — test : deux
       appels encadrant une réévaluation au `as_of` ;
     - `Some(&[])` rend un `Vec` vide **sans requête** (un `IN ()` est une erreur de syntaxe ; et, à la différence
       de `select_accounts`, `general_ledger.rs:366`, une liste vide ne vaut pas `None`) ;
     - `account_ids = None` : tous les comptes **en devise** de la société ; `Some(ids)` : exactement ces comptes
       (bornés à la société, KF-002 : un identifiant étranger à la société est **ignoré**, sans erreur — l'appelant
       compare le nombre de lignes rendues à celui demandé), **chacun rendu** même sans ligne (`LEFT JOIN`, `(0, Some(0))`) ; `devise_net` vaut
       `None` pour un compte CHF ;
     - les lignes `revaluation` comptent dans le CHF, pas dans la devise ;
     - lecture ordinaire (non verrouillante) dans la connexion fournie : c'est le verrou d'exercice de 35-5 qui la
       stabilise.
2. **AC-2 — Grand livre** (`general_ledger.rs` `generate` :644). Pour un compte en devise :
   - `RawAccount`/`select_accounts` (`general_ledger.rs:250`, `:359`) lisent `a.currency` (source de
     `LedgerSection.currency` et de la devise liée aux requêtes d'ouverture et de totaux) ;
   - `RawLine`/`fetch_lines` lisent `amount_currency`, `exchange_rate`, `rate_source` (la requête ne joint pas
     `accounts`, `general_ledger.rs:407-413` ; la devise d'une ligne est celle de sa section, T3) ; `LedgerLine` gagne le
     montant en devise (en débit/crédit comme le CHF), le cours (10 décimales stockées ; affichage par un formateur
     **neuf** `format_exchange_rate`, AC-5 — ⚠️ **pas** `format_rate` de `pdf.rs:1063`, qui rend un taux de TVA
     `"{rate:.2} %"`), le solde progressif en devise ; `LedgerSection` gagne `currency`, `openingCurrency`,
     `closingCurrency`, totaux en devise. `unnatural_balance` reste CHF.
   - Une ligne `revaluation` s'affiche « Réévaluation au cours X » sans montant en devise ; comptes CHF : inchangés.
     Le libellé suit le régime de chaque sortie, tel qu'il est : le **JSON** ne porte pas de libellé (il porte
     `rateSource` et le cours) ; l'**écran** le traduit (clé i18n, quatre locales) ; le **PDF** le prend d'un champ
     neuf de `SectionLabels` (défaut français, `pdf.rs:153`, comme `ledger_fy_break` — le handler construit
     `PdfContext::fr_ch_default`, `routes/reports.rs:940`) ; le **CSV** l'écrit en littéral français, comme ses
     autres libellés (`"Solde d'ouverture"`, `csv.rs:709`).
   - Les structs `LedgerLine`/`LedgerSection` sont construites à la main en `general_ledger.rs:726`, `:781`, `:824`
     (test), `:850` (test), `csv.rs:1325`, `:1333`, `pdf.rs:2451`, `:2474` (relevé du 2026-11-14 ;
     `tests/general_ledger.rs:135` n'en construit aucune, c'est un type de retour) : sites revus au Change Log.
3. **AC-3 — Balance** (`trial_balance.rs` :129). `TrialBalanceRow` gagne `currency`, `openingCurrency`,
   `totalDebitCurrency`, `totalCreditCurrency`, `balanceCurrency`, `closingCurrency` (compte en devise seulement).
   **Aucun total en devise** (devises mélangées) : `total_debit`, `total_credit`, `balanced`, `opening_balanced`,
   `retained_earnings` restent CHF.
4. **AC-4 — Bilan** (`balance_sheet.rs` :166) et **compte de résultat** (`income_statement.rs` :30) : `AccountBalance`
   gagne `currency` et `balanceCurrency` (`a.currency` dans `SELECT` et `GROUP BY`) ; mention « USD 1'233.57 » à côté
   du CHF ; totaux en CHF seulement.
5. **AC-5 — Exports.**
   - **CSV : règle de colonnes.** Les colonnes en devise n'apparaissent **que si le rapport contient au moins un
     compte en devise** ; sinon la sortie est **identique octet pour octet** à celle d'avant, y compris pour une société
     multi-devises sans compte en devise dans le rapport. Pour un rapport donné l'en-tête est stable ; lignes CHF :
     colonnes vides.
   - **PDF** : « mêmes lignes de texte » en mono-devise (horodatage : pas d'octet pour octet).
     - *Grand livre* : sous chaque mouvement en devise, une **seconde ligne en petit** (« USD 31.35 × 0.8808 · solde USD
       1'233.57 ») ; « À reporter » et « Report » portent aussi le report en devise (`carry` en devise), remis à zéro à
       la rupture d'exercice des comptes de résultat ; ouverture, clôture et rupture portent le solde en devise.
     - *Bilan et compte de résultat* : « USD 1'233.57 » entre le nom et le solde (`draw_account_row`, `pdf.rs:482`).
     - *Balance* : ses six colonnes occupent la largeur (`TB_COL_X`, `pdf.rs:718`) ; sous la ligne d'un compte en
       devise, une **seconde ligne en petit**, comme au grand livre : « USD ouverture … · débit … · crédit … ·
       clôture … ». Aucune ligne de total en devise.
     - Test de PDF multi-pages.
   - Les colonnes en devise sont portées par le JSON, le PDF et le CSV ; l'existant reste inégal (`lettering_code` JSON
     seul, C-15-1b-6).
   - **Affichage** : 2 décimales, somme exacte puis arrondi **explicite** `round_dp_with_strategy(2,
     MidpointAwayFromZero)` avant tout formatage (les formateurs existants ne font qu'un `{:.2}` : `pdf.rs:247`,
     `csv.rs:41`). Montants : `format_swiss_amount` (apostrophes) dans le **PDF** et à l'écran ; dans le **CSV**,
     `format_amount_iso` (`csv.rs:40`, point décimal sans séparateur de milliers, comme les colonnes CHF — un tableur
     ne relit pas `1'233.57`). Cours : `format_exchange_rate` (neuf, `pdf.rs`, `pub(crate)` pour que `csv.rs` l'appelle, partagé par PDF et CSV), qui rend le cours **par
     unité** sans `%`, zéros de queue retirés, au moins 4 décimales et au plus 10 (`0.8808`, `0.0057510000` →
     `0.005751`) ; testé (cours JPY < 0.01, cours à 10 décimales). `format_rate` (`pdf.rs:1063`, taux de TVA en
     `%` à 2 décimales) ne s'emploie pas : il rendrait `0.88 %`.
6. **AC-6 — JSON, écran et MCP.**
   - Les champs en devise portent `#[serde(skip_serializing_if = "Option::is_none")]` : le JSON d'un rapport sans
     compte en devise est **identique** à celui d'avant (test d'égalité sur un JSON enregistré avant).
   - Vues (`GeneralLedgerView`, `TrialBalanceView`, `BalanceSheetView`, `IncomeStatementView`, `JournalReportView`),
     types TS (`reports.types.ts`), `breakBefore`/`trailingBreaks` de `GeneralLedgerView.svelte` (règle dupliquée en
     TS), `isReportEmpty`, i18n quatre locales (`i18n-keys`, `i18n-libelle-en-dur` verts). Mono-devise : rien de
     visible ne change (test).
   - **MCP** : `kesh_grand_livre` (`LIMITE_MAX_MCP = 30`), `kesh_balance` (`LIMITE_MAX_MCP_BALANCE = 55`),
     `kesh_bilan`, `kesh_compte_de_resultat` : champs en devise **présents à `null`** hors compte en devise — la macro
     `sortie!` (`mcp/sortie.rs:97-114`) n'admet aucun attribut de champ et **n'est pas étendue** (choix de
     l'orchestrateur, 2026-11-14, aligné sur 35-1 AC-8 et 35-3a AC-8, registre C-35-9 ; le JSON des routes de rapports,
     lui, omet ces champs par `skip_serializing_if`, AC-6). Sorties à étendre : `LigneGrandLivre`, `Section`, `Rupture`
     (`grand_livre.rs`), `LigneDeSolde` (`rapports.rs:63`) et `LigneBalance` (`rapports.rs:104`, construite `:219`).
     Limites recalculées
     pour le pire cas avec champs en devise sous `MCP_ENVOI_OCTETS_MAX` (`services/envois_mcp.rs:55`, contrôlé par
     `mcp/sortie.rs:463`) : mesure par `tailles_mesurees` (`tests/mcp_outils_e2e.rs:1345`), dont la fixture
     `fixture_tailles` (`:788`) gagne des comptes et des lignes en devise (société en multi-devises) ; toute limite
     changée l'est aussi dans `bornes_attendues` (`:99-160`) et la table `mesurees` de `bornes_des_schemas`
     (`:182-195`). `kesh_bilan` et `kesh_compte_de_resultat` ne sont pas paginés : le seuil « environ 60 comptes » du
     commentaire de `mcp/outils/rapports.rs:6-7` est re-mesuré avec champs en devise et récrit ; code de
     devise typé (`Code`/`TexteNonFiable`), `mcp_gardes_lexicales` et `mcp_contrat` verts.
7. **AC-7 — Rapports inchangés ou étendus : inventaire clos.**
   - **Journal** (`journal_report.rs:92`) : le JSON et l'écran gagnent les champs en devise (omis si absents) ; le PDF
     et le CSV du journal restent **inchangés**.
   - **CHF seulement**, chacun testé (une écriture USD y est comptée au CHF de sa ligne) :
     accueil (`homepage_summary.rs:252`, `:295` : graphiques en CHF) ; rapports de projets (`project_report.rs:263`,
     `:291`, `:456`) ; TVA (`vat_report.rs:261`, `:337`, `:368` : base légale CHF) ; balance âgée
     (`aged_receivables.rs` : factures, hors périmètre D7).
   - **Hors rapports, CHF seulement, nommés pour clore l'inventaire** (sommes de `debit`/`credit` relevées par
     `grep -rln "SUM(jel.debit\|SUM(jel.credit\|SUM(l.debit\|SUM(debit"` hors tests ; le grep rend aussi
     `kesh-api/src/routes/bank_accounts.rs`, par deux commentaires seulement, `:244` et `:878`) : contrôles de clôture
     (`kesh-db/src/repositories/closing_checks.rs:536`, équilibre CHF par écriture) ; solde comptable d'un compte
     bancaire (`bank_accounts.rs:1871-1905`, compte lié jamais en devise, C-35-9) ; postes ouverts et lettrage
     (`letterings/open_items.rs:83`, `:91` : limite du lettrage, 35-3a AC-9) ; réconciliation
     (`reconciliation_link.rs:151`, `:801`, comptes en devise exclus, C-35-5) ; équilibre des écritures
     (`journal_entries.rs:522`, `:1684`, `:913`). Aucun n'est étendu ; un site neuf trouvé au développement s'ajoute
     ici, au Change Log.
8. **AC-8 — Tests.** Jeu à deux exercices : compte USD de bilan, compte USD de **résultat** (ventes), compte de
   **passif** en devise, compte archivé ; achats et ventes à des cours différents, conversion, réévaluation.
   - La ligne `revaluation` est posée **par le dépôt** (`NewJournalEntryLine.foreign`, source `revaluation`), jamais
     par l'API (refusée par 35-3a AC-6). Elle est exemptée de l'égalité CHF = round(montant × cours) par 35-3a AC-3
     (exemption `revaluation`, déjà écrite) et par `chk_jel_fx_chf` (35-3a AC-1).
   - Soldes en devise et CHF attendus calculés à la main ; grand livre sur une période franchissant la fin d'exercice
     (rupture et clôture en devise) avec `offset > 0` (#775) ; compte à CHF 0 et devise ≠ 0 listé aux quatre rapports ;
     **concordance** grand livre ↔ balance ↔ bilan ↔ compte de résultat **en devise** (appel réel des générateurs,
     comme les tests CHF de `general_ledger.rs`) ; contenu CSV et PDF ; MCP ; `balances_as_of` dans une transaction.
   - **T3 (CR-4)** : la devise affichée d'un mouvement, d'une section ou d'une ligne de rapport est celle du compte
     (`a.currency`) — test : un compte EUR à deux lignes (`afc` et `manual`) donne la paire en devise attendue sans
     aucune condition de devise sur la ligne ; ajout : une ligne en devise insérée **par SQL direct** sur un compte CHF
     (hors dépôt) n'apparaît dans aucun champ en devise du grand livre, de la balance, du bilan, du compte de résultat
     ni du journal ; contrôle au Change Log du développement que
     `grep -rnE "\b(jel|l)\.currency\b" crates/ --include=*.rs --include=*.sql` ne rend rien (motif échappé : un `.`
     nu rendrait des faux positifs sur `total_currency` et consorts).
   - Non-régression mono-devise : quatre CSV **octet pour octet** contre fixtures enregistrées **depuis le commit de
     base avant toute modification** (cas multi-devises sans compte en devise compris) ; JSON identique (AC-6).
9. **AC-9 — Manuel** : lecture des rapports en devise ; i18n ; PDF.

## Tâches

- [ ] `kesh-report` : paire en devise, `Cumul`, quatre générateurs + journal ; PDF, CSV, `format_exchange_rate`.
- [ ] `kesh-db` : `repositories/account_balances.rs`, `balances_as_of` (AC-1 quater) et ses tests (borne inclusive,
      compte sans ligne, `None`/`Some`, dans une transaction).
- [ ] Sites de construction à revoir (échec de compilation), relevés le 2026-11-14 : `AccountBalance` —
      `balance_sheet.rs:376` (test), `csv.rs` (7 : :846, :928, :937, :946, :1054, :1065, :1074), `pdf.rs` (7 : :1787,
      :1800, :1826, :1839, :2000, :2298, :2307), `benches/export.rs:45` ; `TrialBalanceRow` — `trial_balance.rs:213`,
      `:284`, `csv.rs:1213`, `pdf.rs:1861`, `benches/export.rs:102` ; `LedgerLine`/`LedgerSection` — AC-2 ; `FiscalYearBreak` (gagne `closingBalanceCurrency`, AC-1 bis) —
      `general_ledger.rs:591`, `:609`, `:842` (test), `csv.rs:1390`, `:1395`, `pdf.rs:2514`, `:2531`, `:2537` ;
      `JournalEntryLineRow` (AC-7 : champs en devise) — `journal_report.rs:177` (et les **deux** `SELECT` de
      `generate`, :100 et :124, qui lisent `a.currency` — la jointure `INNER JOIN accounts a` y est déjà, pour le
      numéro et le nom du compte — et `jel.amount_currency`, `jel.exchange_rate`, `jel.rate_source` ; T3),
      `csv.rs:1172`, `pdf.rs:1902`, `:2404`, `:2412`, `benches/export.rs:153`, `:161`. Consommateurs
      à étendre (sans échec de compilation) : `mcp/outils/rapports.rs:142` (`fn ligne`, `LigneDeSolde`), la
      construction de `LigneBalance` (`mcp/outils/rapports.rs:219`) et la correspondance du grand livre de
      `mcp/outils/grand_livre.rs` (`fn section`, :148) ; **aucune** extension de `sortie!` (champs en devise présents à `null`, AC-6, C-35-9) : les champs neufs des sorties MCP sont des `Option<…>` de types admis.
- [ ] `services/rapports.rs`, routes, MCP (limites).
- [ ] Frontend : cinq vues, types ; Vitest — `IncomeStatementView.test.ts` et `JournalReportView.test.ts` **n'existent
      pas** (relevé du 2026-11-14) : à créer, avec le cas « mono-devise : rien de visible ne change » (AC-6) ; E2E
      (grand livre d'un compte USD).
- [ ] Manuel, CHANGELOG.

## Notes de développement

- `crates/kesh-report/src/` : `general_ledger.rs` (sommes :338-340, :465-468 ; `Cumul`), `trial_balance.rs`
  (:151-161), `balance_sheet.rs` (:289-290, :340), `income_statement.rs` (:63-64), `opening.rs`, `pdf.rs`, `csv.rs`.
- `crates/kesh-api/src/services/rapports.rs` (partagé avec MCP) ; `routes/reports.rs` sérialise les structs
  `kesh-report` directement.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 10 MEDIUM, 3 LOW ; tous remédiés dans la fiche ; P1-11 (côté 35-3a) porté par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 5 LOW ; remédiés dans la fiche sauf 35-4-P2-1 (volet
  35-5 : `balances_as_of` dans `kesh-db`, en sens débit brut — Tâches de 35-5 à aligner).
- 2026-11-14 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche sauf <aucun
  inter-fiches>. Prémisses et lignes citées relues au code (sommes, `opening.rs:75`, binds 18 → +12, `:687`, `:209`,
  `kesh-db/Cargo.toml:48`, `kesh-report/Cargo.toml:9`, sites MCP, inventaire AC-7) : exactes.
- 2026-11-14 — validation P4 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 7 LOW ; remédiés dans la fiche sauf 35-4-P4-1
  (volet 35-3a : extension de `sortie!` aux attributs de champ, à porter par 35-3a AC-8). MEDIUM : P4-1 omission MCP
  impossible avec `sortie!` (`mcp/sortie.rs:97-114`) ; P4-2 fragment SQL partagé — forme des requêtes du grand livre
  et de la balance, lieu `kesh-db` ; P4-3 PDF de la balance non spécifié (`TB_COL_X`, `pdf.rs:718`) ; P4-4 format
  CSV des montants en devise (`format_amount_iso`, `csv.rs:40`) ; P4-8 `LigneBalance` MCP absente de l'inventaire.
  LOW : P4-5 arrondi explicite avant `{:.2}` ; P4-6 sites `LedgerLine`/`LedgerSection` ; P4-7 `JournalEntryLineRow`
  des benches et lignes des `SELECT` du journal ; P4-9 régime des libellés (JSON, PDF, CSV) ; P4-10 `Some(&[])` de
  `balances_as_of` ; P4-11 inventaire AC-7 (`routes/bank_accounts.rs`) ; P4-12 `RawAccount` lit `a.currency`.
- 2026-11-14 — validation P5 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche sauf <aucun
  inter-fiches>. MEDIUM P5-1 : la tâche « Sites de construction » renvoyait à une « extension de `sortie!` reprise de
  35-3a », contraire à l'AC-6 et au registre C-35-9 (champs à `null`, macro non étendue) ; l'entrée P4 ci-dessus
  (« extension de `sortie!` ») est historique. LOW P5-2 : visibilité de `format_exchange_rate` (`pub(crate)`). Lignes
  citées relues au code (`opening.rs:75`, sommes `general_ledger.rs:338`/`:465`, `:687`, `trial_balance.rs:165-195`/`:209`,
  `sign_expr` bilan et résultat, `Cargo.toml` des deux crates, `sortie.rs:97-114`/`:463`, `rapports.rs:63`/`:104`,
  `grand_livre.rs:37`/`:56`/`:61`, `mcp_outils_e2e.rs:99`/`:182`/`:788`/`:1345`, `pdf.rs:153`/`:718`/`:1063`,
  `csv.rs:40`, `journal_report.rs:92`/`:177`) : exactes ; accord avec 35-1, 35-3a et 35-5 (`balances_as_of`) vérifié.
- 2026-11-14 — validation P6 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; aucun inter-fiches. Lentille sur le
  seul diff `c0801b02..c08ccd9b`. Relu au code : `pub(crate)` suffit (`kesh-report/src/lib.rs:18`, `:25`, `csv` et
  `pdf` dans le même crate) ; `Option<T: ChampSur>` admis (`mcp/sortie.rs:85-86`), `Montant` (`:215`, `:228`) et
  `Code<E>` (`:302`) disponibles pour montants, cours et devise ; « présents à `null` », macro non étendue : accord
  avec l'AC-6 de la fiche, 35-3a AC-8 et C-35-9. LOW P6-1 : l'entrée P5 porte un gabarit non rempli
  (« sauf <aucun inter-fiches> ») — à lire « aucun inter-fiches » ; relevé daté laissé tel quel.
- 2026-11-14 — révision CR-4 (revue 2 de Beat T1–T6 ; Opus) : **T3** — la ligne ne stocke plus de devise (35-3a
  AC-1) : les rapports lisent la devise sur le compte (`a.currency`) ; la paire en devise se somme **sans** condition
  `jel.currency = a.currency` (retirée de l'AC-1) ; le fragment partagé n'est plus paramétré que par l'alias de la
  ligne ; `x` de la balance projette `jel.amount_currency` seul ; les requêtes du grand livre (`fetch_lines`,
  ouverture, totaux) ne joignent pas `accounts` pour la somme, la devise de section venant de `select_accounts` (AC-1,
  AC-2) ; les deux `SELECT` du journal lisent `a.currency` par leur jointure existante (Tâches) ; test et contrôle
  « aucune lecture de `jel.currency` » (AC-8). Prémisses relues au code : `journal_report.rs:100-135` (jointure
  `accounts a` présente), `general_ledger.rs:407-413` (`fetch_lines` sans `accounts`). T1, T2, T4, T5, T6 sans point
  de contact. AC touchés : AC-1, AC-2, AC-8, Tâches. Fiche à revalider (cycle CR-4).
- 2026-11-14 — validation CR-4 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW ; remédiés dans la fiche sauf <aucun
  inter-fiches>. MEDIUM CR4-P1-1 : affichage en devise non gouverné par la devise du compte (ligne en devise hors dépôt
  sur compte CHF, grand livre et journal). LOW : CR4-P1-2 `COALESCE` des sommes en devise ; CR4-P1-3 motif du `grep`
  de contrôle T3 (faux positifs) ; CR4-P1-4 test T3 non observable (ajout d'un cas par SQL direct). Relu au code :
  `journal_report.rs:100-135` (jointure `accounts a`), `general_ledger.rs:338-342`/`:407-413`/`:465-473` (aucune jointure
  `accounts`), `trial_balance.rs:165-195` (18 binds ; +12 pour six sommes), `balance_sheet.rs:288-300` et
  `income_statement.rs:62-83` (`sign_expr`, `HAVING balance != 0`, `INNER JOIN` lignes), `opening.rs:75`,
  `kesh-db/Cargo.toml:48`, `sortie.rs:97`, `rapports.rs:63`/`:104`/`:219`, `csv.rs:40`, `pdf.rs:153`/`:718`/`:1063` :
  exacts ; `grep jel.currency|l.currency` sur `crates/` : aucun reste aujourd'hui ; aucun reste de la colonne dans 35-1,
  35-2, 35-3a, 35-3b, 35-5 (accord de 35-3a AC-1/AC-2 et de 35-5 sur `balances_as_of`).
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW, remédiés dans la fiche ; aucun
  inter-fiches. CR4-P2-1 : la règle « la devise gouverne l'affichage » ne nommait que le grand livre et le journal, alors
  que le test T3 couvre aussi balance, bilan et compte de résultat (balance : AC-3 ; bilan, résultat : `CASE`) — nommés.
  CR4-P2-2 : la condition du journal (`amount_currency` non `NULL`) remplissait une ligne `revaluation` à 0 — alignée sur
  AC-2. Relu au code : `journal_report.rs:100-135` (`INNER JOIN accounts a`), `general_ledger.rs:338-342`, `:407-413`,
  `:465-473` (sans `accounts`, sommes CHF en `COALESCE`), `trial_balance.rs:165-170` (`x` sans `amount_currency`),
  `balance_sheet.rs:288-292`, `income_statement.rs:62-66` (`COALESCE` au CHF : « comme le CHF » exact) ; `grep -rnE
  "\b(jel|l)\.currency\b" crates/ --include=*.rs --include=*.sql` : vide, motif non échappé : vide aussi ; test T3 par
  SQL direct réalisable sous `chk_jel_fx_*` (35-3a AC-1 : trois colonnes cohérentes). Axes exercés : vérité au code des
  ajouts, cohérence interne (AC-1 ter, AC-2, AC-3, AC-7), points de contact 35-3a (contraintes, colonne absente).
  Non exercés : relecture complète de la fiche (hors périmètre), 35-5 (aucun ajout ne touche `balances_as_of`).
