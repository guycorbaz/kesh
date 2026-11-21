# Story 37-4 : rapports dans les deux devises

Status: done

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/37-multi-devises`. Epic 37, D2, C-37-3. Suit 37-3a.
**Aucune migration attendue** (colonnes de 37-1 et 37-3a ; aucun index nécessaire : les sommes passent par
`account_id`).*

## Story

En tant que comptable en multi-devises, je veux voir, pour chaque compte en devise, le solde et les mouvements en
devise à côté du montant en devise principale (aujourd'hui CHF), dans le grand livre, la balance, le bilan et le compte de résultat, à l'écran et dans les
exports, afin de rapprocher mes comptes PayPal ou Wise de leurs relevés et de contrôler ma réévaluation.

## Exigences

*(B26, 2026-11-15 : tirées des critères d'acceptation ci-dessous, sans en créer ni en modifier.)*

**Fonctionnelles**

- FR-37-4-1 — Le solde en devise d'un compte est Σ `amount_currency` au débit moins au crédit (côté de la ligne = celui de son CHF), signé par le type du compte comme le CHF (`opening.rs`, sans seconde règle), la devise étant celle du compte (`a.currency`), jamais d'une ligne : un compte CHF n'a aucun champ en devise, même si une ligne en devise y a été posée hors du dépôt ; les sommes CHF ne changent pas. Un compte de résultat en devise suit les bornes d'exercice du CHF (ouverture depuis le début de l'exercice, solde progressif remis à zéro à la rupture, clôture = solde de l'exercice final) ; un compte est listé si son solde CHF **ou** son solde en devise est non nul (AC-1, AC-1 bis, AC-1 ter). Pour 37-5, `kesh_db::repositories::account_balances::balances_as_of` rend, par compte, le débit net brut CHF et en devise sans signe par type, à borne **inclusive** `<= as_of`, tous exercices, réévaluation du jour comprise ; `None` = tous les comptes en devise de la société, `Some(ids)` = ces comptes bornés à la société (identifiant étranger ignoré, KF-002), chacun rendu même sans ligne, `Some(&[])` vide sans requête ; lecture non verrouillante dans la connexion fournie (AC-1 quater).
- FR-37-4-2 — Le grand livre montre, pour un compte en devise, montant, cours, solde progressif, ouverture, clôture (rupture d'exercice comprise) et totaux en devise ; une ligne `revaluation` se lit « Réévaluation au cours X » sans montant en devise ; `unnatural_balance` reste calculé en devise principale (AC-1 bis, AC-2).
- FR-37-4-3 — La balance porte les colonnes en devise d'un compte en devise (AC-3).
- FR-37-4-4 — Le bilan et le compte de résultat portent la devise et le solde en devise (AC-4).
- FR-37-4-5 — Les exports CSV et PDF du grand livre, de la balance, du bilan et du compte de résultat portent les champs en devise (colonnes CSV seulement si le rapport contient un compte en devise) ; montants arrondis explicitement à 2 décimales, `format_swiss_amount` au PDF, `format_amount_iso` au CSV, cours par `format_exchange_rate` (neuf, par unité, 4 à 10 décimales) ; au PDF, le grand livre met une seconde ligne en petit sous chaque mouvement en devise et reporte aussi le report en devise (« À reporter », « Report »), le bilan et le compte de résultat mettent la devise et son solde entre le nom et le solde, la balance met une seconde ligne en petit sous chaque compte en devise, sans ligne de total en devise (AC-5).
- FR-37-4-6 — JSON, vues et outils MCP (`kesh_grand_livre`, `kesh_balance`, `kesh_bilan`, `kesh_compte_de_resultat`) portent les champs en devise (AC-6).
- FR-37-4-7 — Le journal gagne les champs en devise au JSON et à l'écran seulement, son PDF et son CSV restent inchangés ; accueil, projets, TVA et balance âgée restent en CHF, de même que les sites hors rapports de l'inventaire clos (contrôles de clôture, compte bancaire, postes ouverts et lettrage avec l'outil MCP `kesh_postes_ouverts`, réconciliation, équilibre des écritures) (AC-7).

**Non fonctionnelles**

- NFR-37-4-1 — CSV identique octet pour octet à l'actuel s'il n'y a aucun compte en devise dans le rapport ; PDF aux mêmes lignes de texte (AC-5).
- NFR-37-4-2 — JSON d'un rapport sans compte en devise identique à l'actuel ; rien ne change à l'écran en mono-devise (AC-6).
- NFR-37-4-3 — Aucun total de rapport en devise étrangère (devises mêlées) : les totaux de la balance, du bilan et du compte de résultat, PDF compris, restent en devise principale ; seuls les totaux d'une section du grand livre se donnent en devise (AC-2, AC-3, AC-4, AC-5).
- NFR-37-4-4 — Champs MCP présents à `null` hors compte en devise, dans les limites de taille re-mesurées ; code de devise typé, `mcp_gardes_lexicales` et `mcp_contrat` verts (AC-6).
- NFR-37-4-5 — Quatre locales, `i18n-keys` et `i18n-libelle-en-dur` verts (AC-6).
- NFR-37-4-6 — Concordance grand livre, balance, bilan et compte de résultat en devise sur un jeu à deux exercices, `offset > 0` compris ; une ligne en devise posée par SQL sur un compte CHF n'apparaît dans aucun champ en devise ; CSV mono-devise comparés à des fixtures du commit de base ; `balances_as_of` testée dans une transaction ; `grep -rnE "\b(jel|l)\.currency\b"` sur `crates/` ne rend rien, contrôle consigné au Change Log du développement (AC-8).
- NFR-37-4-7 — Manuel à jour (AC-9) : `.tex` seulement, PDF du manuel non commité (régénéré à la release, rendu contrôlable en local).

## Critères d'acceptation

1. **AC-1 — Règle unique.** Le solde en devise d'un compte = Σ `amount_currency` au débit − Σ au crédit, signé comme le
   CHF par le type du compte (`opening.rs`).
   - **Côté d'une ligne** : celui de son montant CHF (`debit > 0` ⇒ débit, `credit > 0` ⇒ crédit) ; `amount_currency`
     n'a pas de signe (C-37-3). Les rapports lisent une **paire brute (débit, crédit) en devise** :
     `SUM(CASE WHEN jel.debit > 0 THEN jel.amount_currency ELSE 0 END)` et son symétrique, **sans condition de
     devise sur la ligne** — CR-4, T3 : la ligne ne porte plus de devise (37-3a AC-1) ; la **devise est celle du
     compte**, `a.currency`, et toutes les lignes d'un compte en devise sont dans sa devise (verrou de 37-1 AC-4 dès la
     première ligne, contrôle central de 37-3a AC-4). Aucune condition `jel.currency = a.currency` ne s'écrit nulle
     part (la colonne n'existe pas). Un compte CHF ne porte aucune ligne en devise (`amount_currency` `NULL`, ignoré
     par `SUM`) ; le solde en devise d'un compte CHF reste `NULL` par `a.currency IS NULL` (ci-dessous), ce qui
     neutralise aussi une ligne en devise posée hors du dépôt (limite de 37-3a AC-1). Une ligne `revaluation`
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
   - Les sommes existantes en devise principale (CHF) **ne changent pas**. Un compte CHF a `currency = None` et aucun champ en devise.
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
   - **AC-1 quater — Fonction à date pour 37-5.** ⚠️ **Elle vit dans `kesh-db`**, non dans `kesh-report` :
     `fx_revaluation::post` (37-5) est une fonction de dépôt de `kesh-db`, et `kesh-db` ne dépend de `kesh-report` qu'en
     `[dev-dependencies]` (`crates/kesh-db/Cargo.toml:48`) — l'inverse (`kesh-report/Cargo.toml:9`) ferait un cycle.
     `kesh_db::repositories::account_balances::balances_as_of(conn: &mut MySqlConnection, company_id, as_of,
     account_ids: Option<&[i64]>)` → `Vec<(account_id, chf_net, Option<devise_net>)>`, en **sens débit brut**
     (Σ débit − Σ crédit, et la paire en devise de l'AC-1), **sans signe par type** — la grandeur même de la formule de
     37-5 (AC-1 de 37-5), et `opening.rs` n'est pas visible de `kesh-db`. Règles :
     - borne **inclusive** `je.entry_date <= as_of`, tous exercices, **sans borne basse ni `HAVING`** : une
       réévaluation déjà passée au même jour est comptée, ce qui rend le rejeu de 37-5 (résidu seul) exact — test : deux
       appels encadrant une réévaluation au `as_of` ;
     - `Some(&[])` rend un `Vec` vide **sans requête** (un `IN ()` est une erreur de syntaxe ; et, à la différence
       de `select_accounts`, `general_ledger.rs:366`, une liste vide ne vaut pas `None`) ;
     - `account_ids = None` : tous les comptes **en devise** de la société ; `Some(ids)` : exactement ces comptes
       (bornés à la société, KF-002 : un identifiant étranger à la société est **ignoré**, sans erreur — l'appelant
       compare le nombre de lignes rendues à celui demandé), **chacun rendu** même sans ligne (`LEFT JOIN`, `(0, Some(0))`) ; `devise_net` vaut
       `None` pour un compte CHF ;
     - les lignes `revaluation` comptent dans le CHF, pas dans la devise ;
     - lecture ordinaire (non verrouillante) dans la connexion fournie : c'est le verrou d'exercice de 37-5 qui la
       stabilise.
2. **AC-2 — Grand livre** (`general_ledger.rs` `generate` :644). Pour un compte en devise :
   - `RawAccount`/`select_accounts` (`general_ledger.rs:250`, `:359`) lisent `a.currency` (source de
     `LedgerSection.currency` et de la devise liée aux requêtes d'ouverture et de totaux) ;
   - `RawLine`/`fetch_lines` lisent `amount_currency`, `exchange_rate`, `rate_source` (la requête ne joint pas
     `accounts`, `general_ledger.rs:407-413` ; la devise d'une ligne est celle de sa section, T3) ; `LedgerLine` gagne le
     montant en devise (en débit/crédit comme le CHF), le cours (10 décimales stockées ; affichage par un formateur
     **neuf** `format_exchange_rate`, AC-5 — ⚠️ **pas** `format_rate` de `pdf.rs:1063`, qui rend un taux de TVA
     `"{rate:.2} %"`), le solde progressif en devise ; `LedgerSection` gagne `currency`, `openingCurrency`,
     `closingCurrency`, totaux en devise. `unnatural_balance` reste calculé sur le solde en devise principale (U6).
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
   **Aucun total en devise étrangère** (devises mélangées) : `total_debit`, `total_credit`, `balanced`, `opening_balanced`,
   `retained_earnings` restent en **devise principale** (aujourd'hui CHF ; A29, U6).
4. **AC-4 — Bilan** (`balance_sheet.rs` :166) et **compte de résultat** (`income_statement.rs` :30) : `AccountBalance`
   gagne `currency` et `balanceCurrency` (`a.currency` dans `SELECT` et `GROUP BY`) ; mention « USD 1'233.57 » à côté
   du solde en devise principale ; **totaux en devise principale** seulement (A29, U6).
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
     l'orchestrateur, 2026-11-14, aligné sur 37-1 AC-8 et 37-3a AC-8, registre C-37-9 ; le JSON des routes de rapports,
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
     bancaire (`bank_accounts.rs:1871-1905`, compte lié jamais en devise, C-37-9) ; postes ouverts et lettrage
     (`letterings/open_items.rs:83`, `:91` : limite du lettrage, 37-3a AC-9), **et l'outil MCP `kesh_postes_ouverts`**
     qui les rend (V13, CR-6 : `mcp/outils/postes_ouverts.rs:210`, par `services/postes_ouverts.rs:98` →
     `letterings::open_items` — site atteint par cet appel, non par le `grep` ci-dessus, qui ne relève que des
     `SUM` ; sortie `Poste` et `PostesOuverts`, `:125-161`, en CHF seulement — un compte en devise lettrable y est
     atteignable ; aucun champ en devise ajouté, renvoi à la limite de 37-3a AC-9 ; la devise du compte se lit par
     `kesh_plan_comptable` (`devise`, 37-1 AC-8), et la description de l'outil, qui dit « solde », ne le dit pas en
     devise : un agent lit des CHF ; test : un compte de débiteurs USD rend ses postes au CHF de leurs lignes) ; réconciliation
     (`reconciliation_link.rs:151`, `:801`, comptes en devise exclus, C-37-5) ; équilibre des écritures
     (`journal_entries.rs:522`, `:1684`, `:913`). Aucun n'est étendu ; un site neuf trouvé au développement s'ajoute
     ici, au Change Log.
8. **AC-8 — Tests.** Jeu à deux exercices : compte USD de bilan, compte USD de **résultat** (ventes), compte de
   **passif** en devise, compte archivé ; achats et ventes à des cours différents, conversion, réévaluation.
   - La ligne `revaluation` est posée **par le dépôt** (`NewJournalEntryLine.foreign`, source `revaluation`), jamais
     par l'API (refusée par 37-3a AC-6). Elle est exemptée de l'égalité CHF = round(montant × cours) par 37-3a AC-3
     (exemption `revaluation`, déjà écrite) et par `chk_jel_fx_chf` (37-3a AC-1).
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
9. **AC-9 — Manuel** : lecture des rapports en devise ; i18n (compteurs `sitesTotal` recomptés depuis la source après chaque reprise de `main`) ; `.tex` seulement, PDF du manuel non commité (régénéré à la release).

## Tâches

- [x] `kesh-report` : paire en devise, `Cumul`, quatre générateurs + journal ; PDF, CSV, `format_exchange_rate`.
- [x] `kesh-db` : `repositories/account_balances.rs`, `balances_as_of` (AC-1 quater) et ses tests (borne inclusive,
      compte sans ligne, `None`/`Some`, dans une transaction).
- [x] Sites de construction à revoir (échec de compilation), relevés le 2026-11-14 : `AccountBalance` —
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
      `mcp/outils/grand_livre.rs` (`fn section`, :148) ; **aucune** extension de `sortie!` (champs en devise présents à `null`, AC-6, C-37-9) : les champs neufs des sorties MCP sont des `Option<…>` de types admis.
- [x] `services/rapports.rs`, routes, MCP (limites).
- [x] CR-6 (V13, AC-7) : test `mcp_outils_e2e.rs` — `kesh_postes_ouverts` sur un compte de débiteurs USD lettrable
      (lignes en devise) rend `solde`, `total_ouvert` et les `debit`/`credit` des postes au CHF des lignes, sans champ
      en devise (`reste_du` à `null` : aucune facture en devise) ; site inscrit à l'inventaire « CHF seulement » du
      Change Log du développement.
- [x] Frontend : cinq vues, types ; Vitest — `IncomeStatementView.test.ts` et `JournalReportView.test.ts` **n'existent
      pas** (relevé du 2026-11-14) : à créer, avec le cas « mono-devise : rien de visible ne change » (AC-6) ; E2E
      (grand livre d'un compte USD).
- [x] Manuel (`.tex` seulement), section *Note de version* de la fiche (pas d'écriture au `CHANGELOG.md`).

### Review Findings

*(Revue de code P1, 2026-11-16, Opus : Blind Hunter, Edge Case Hunter, Acceptance Auditor.)*

- [x] [Review][Patch] 37-4-CR-P1-1 MEDIUM — ordre des `bind` de la balance non exercé sur le segment « depuis le début de l'exercice » (toutes les périodes testées commençaient au début de l'exercice) ; commentaire qui annonçait un test absent [`trial_balance.rs:228`, `:396`] — test `balance_en_devise_sur_une_periode_en_cours_d_exercice`, commentaires récrits.
- [x] [Review][Patch] 37-4-CR-P1-2 MEDIUM — aucune capture du compte de résultat ni du journal en devise [`frontend/tests/e2e/reports-currency.spec.ts`] — vente USD sur un compte de produits USD, `37-4-resultat-usd.png` et `37-4-journal-usd.png`.
- [x] [Review][Patch] 37-4-CR-P1-3 MEDIUM — `docs/api-external.md` ne disait rien des champs en devise des rapports [`docs/api-external.md`] — paragraphe « Les rapports en devise ».
- [x] [Review][Patch] 37-4-CR-P1-4 MEDIUM — `balances_as_of(Some(ids))` : un identifiant répété faussait la comparaison du nombre de lignes rendues (KF-002) [`account_balances.rs`] — identifiants dédoublonnés, règle écrite, test `un_identifiant_repete_n_est_rendu_qu_une_fois`.
- [x] [Review][Patch] 37-4-CR-P1-5 LOW — résidu négatif sous le demi-centime affiché « -0.00 » [`pdf.rs` `round_for_display`, `reports.api.ts` `formatCurrencyAmount`] — zéro sans signe, testé en Rust et en Vitest.
- [x] [Review][Patch] 37-4-CR-P1-6 LOW — au PDF du bilan et du compte de résultat, un nom long recouvrait le solde en devise [`pdf.rs` `draw_account_row`] — nom d'un compte en devise tronqué (`ACCOUNT_NAME_MAX_CHARS_FX`), test.
- [x] [Review][Patch] 37-4-CR-P1-7 LOW — PDF du grand livre : la place réservée en tête de section ignorait la seconde ligne en devise [`pdf.rs:1712`] — 5 lignes pour une section en devise.
- [x] [Review][Patch] 37-4-CR-P1-8 LOW — test T3 vide pour le compte de résultat (ligne SQL posée sur un compte d'actif) [`tests/rapports_en_devise.rs`] — ligne en devise posée par SQL sur le compte de produits CHF 3000, compte de résultat 2025 contrôlé.
- [x] [Review][Patch] 37-4-CR-P1-9 LOW — `kesh_compte_de_resultat` non testé en devise par le MCP [`tests/mcp_outils_e2e.rs`] — ajouté à `rapports_en_devise_par_le_mcp`.
- [x] [Review][Patch] 37-4-CR-P1-10 LOW — décompte « 56 comptes » pour un bilan de 57 [`tests/mcp_outils_e2e.rs`] — corrigé.
- [x] [Review][Patch] 37-4-CR-P1-11 LOW — commentaire « À recompter après la fusion de la suite de la 37-3a » périmé [`i18n-keys.test.ts`] — récrit (3191, relevé de la garde).
- [x] [Review][Patch] 37-4-CR-P1-12 LOW — `somme_en_devise` recopie `alias` et `condition` dans le SQL sans le dire [`account_balances.rs`] — invariant écrit (littéraux du code seulement).
- [x] [Review][Patch] 37-4-CR-P1-13 LOW — lignes de l'inventaire AC-7 périmées après la fusion de la 37-3a : sites d'équilibre `kesh-db/src/repositories/journal_entries.rs` aujourd'hui `:698`, `:1097`, `:1907` — relevé au Change Log (AC non réécrit).
- [ ] [Review][Defer] 37-4-CR-P1-14 LOW — tests « CHF seulement » faibles (`rapports_chf_seulement.rs` : balance âgée sans facture, projets et accueil sur une ligne sans montant en devise, `contains("\"91.00")`) — non traité, à tracer.
- [ ] [Review][Defer] 37-4-CR-P1-15 LOW — « CHF » écrit en dur dans les descriptions MCP (`grand_livre.rs`, `rapports.rs`) et le manuel d'administration, là où U6/A29 disent « devise principale » — vrai aujourd'hui ; non traité, à tracer.
- [x] [Review][Patch] 37-4-CR-P2-1 MEDIUM — PDF du grand livre et de la balance : la seconde ligne en devise était à 4,2 mm sous sa ligne et 3,0 mm au-dessus de la suivante, collée à la suivante (relevé au rendu, balance et grand livre d'un compte USD) [`pdf.rs`] — `write_fx_line`, remontée `FX_RAISE_MM` (avance totale inchangée) ; test `la_seconde_ligne_en_devise_est_plus_pres_de_son_mouvement_que_du_suivant` (rouge à remontée 0).
- [x] [Review][Patch] 37-4-CR-P2-2 MEDIUM — balance à l'écran : les montants en devise passaient par `formatReportAmount` (pas d'arrondi, « -0.00 » possible, Blind Hunter M2 et Edge F3) [`TrialBalanceView.svelte`] — `formatCurrencyFigure` (partagé avec `formatCurrencyAmount`), test Vitest.
- [x] [Review][Patch] 37-4-CR-P2-3 LOW — nom tronqué à 38 caractères : trop large en capitales (85 mm à 2,5 mm = 34) [`pdf.rs`] — 34.
- [x] [Review][Patch] 37-4-CR-P2-4 LOW — `projets_en_chf` cherchait « "91.00 » dans le texte [`rapports_chf_seulement.rs`] — montants lus en nombre, 91 présent, 100 absent.
- [ ] [Review][Defer] 37-4-CR-P2-5 LOW — `balance_agee_inchangee` sans facture (vide), TVA sur l'impôt préalable seul (AC-7), croisement `sign_exprs` ↔ `signed` sans type `Expense` en devise (AC-1) — non traités, à tracer (avec CR-P1-14).
- [ ] [Review][Defer] 37-4-CR-P2-6 LOW — « CHF » en dur pour « devise principale » : descriptions MCP (tailles mesurées), manuels, `docs/api-external.md` — vrai aujourd'hui, non traité, à tracer (avec CR-P1-15).
- [ ] [Review][Defer] 37-4-CR-P2-7 LOW — `somme_en_devise` en `&str` public ; 4 SUM de plus par compte CHF ; colonnes CSV variables selon le rapport (choix de contrat) ; `colspan` du grand livre si « description » masquée ; `balances_as_of(None)` rend les comptes en devise archivés (non écrit) ; plus de 65 532 identifiants ; ligne sans cours sur compte en devise (« USD 0.00 × ») — non traités, à tracer.



## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release ; la fiche n'écrit jamais au `CHANGELOG.md`.)*

- **Added** — grand livre, balance, bilan et compte de résultat en devise (JSON, écran, exports CSV et PDF) ; le journal gagne les champs en devise au JSON et à l'écran seulement ; champs MCP en devise.
- **Inchangé** — s'il n'y a aucun compte en devise, CSV aux mêmes colonnes qu'avant et PDF aux mêmes lignes de texte ; accueil, projets, TVA et balance âgée restent en CHF. *(Corrigé à la revue d'intégration de l'équipe de Guy, 2026-11-17 — cette ligne disait « CSV identique octet pour octet » : vrai à la livraison de la 37-4, faux depuis l'échappement des cellules texte de la 37-5, qui peut changer une cellule d'un CSV de rapport même sans compte en devise ; Note de version de la 37-5.)*
- **Compteurs** : recomptés depuis la source après chaque reprise de `main` (`sitesTotal`, catalogues i18n).

## Notes de développement

- `crates/kesh-report/src/` : `general_ledger.rs` (sommes :338-340, :465-468 ; `Cumul`), `trial_balance.rs`
  (:151-161), `balance_sheet.rs` (:289-290, :340), `income_statement.rs` (:63-64), `opening.rs`, `pdf.rs`, `csv.rs`.
- `crates/kesh-api/src/services/rapports.rs` (partagé avec MCP) ; `routes/reports.rs` sérialise les structs
  `kesh-report` directement.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 10 MEDIUM, 3 LOW ; tous remédiés dans la fiche ; P1-11 (côté 37-3a) porté par l'orchestrateur.
- 2026-11-14 — validation P2 (Opus) : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 5 LOW ; remédiés dans la fiche sauf 37-4-P2-1 (volet
  37-5 : `balances_as_of` dans `kesh-db`, en sens débit brut — Tâches de 37-5 à aligner).
- 2026-11-14 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche sauf <aucun
  inter-fiches>. Prémisses et lignes citées relues au code (sommes, `opening.rs:75`, binds 18 → +12, `:687`, `:209`,
  `kesh-db/Cargo.toml:48`, `kesh-report/Cargo.toml:9`, sites MCP, inventaire AC-7) : exactes.
- 2026-11-14 — validation P4 (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 7 LOW ; remédiés dans la fiche sauf 37-4-P4-1
  (volet 37-3a : extension de `sortie!` aux attributs de champ, à porter par 37-3a AC-8). MEDIUM : P4-1 omission MCP
  impossible avec `sortie!` (`mcp/sortie.rs:97-114`) ; P4-2 fragment SQL partagé — forme des requêtes du grand livre
  et de la balance, lieu `kesh-db` ; P4-3 PDF de la balance non spécifié (`TB_COL_X`, `pdf.rs:718`) ; P4-4 format
  CSV des montants en devise (`format_amount_iso`, `csv.rs:40`) ; P4-8 `LigneBalance` MCP absente de l'inventaire.
  LOW : P4-5 arrondi explicite avant `{:.2}` ; P4-6 sites `LedgerLine`/`LedgerSection` ; P4-7 `JournalEntryLineRow`
  des benches et lignes des `SELECT` du journal ; P4-9 régime des libellés (JSON, PDF, CSV) ; P4-10 `Some(&[])` de
  `balances_as_of` ; P4-11 inventaire AC-7 (`routes/bank_accounts.rs`) ; P4-12 `RawAccount` lit `a.currency`.
- 2026-11-14 — validation P5 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche sauf <aucun
  inter-fiches>. MEDIUM P5-1 : la tâche « Sites de construction » renvoyait à une « extension de `sortie!` reprise de
  37-3a », contraire à l'AC-6 et au registre C-37-9 (champs à `null`, macro non étendue) ; l'entrée P4 ci-dessus
  (« extension de `sortie!` ») est historique. LOW P5-2 : visibilité de `format_exchange_rate` (`pub(crate)`). Lignes
  citées relues au code (`opening.rs:75`, sommes `general_ledger.rs:338`/`:465`, `:687`, `trial_balance.rs:165-195`/`:209`,
  `sign_expr` bilan et résultat, `Cargo.toml` des deux crates, `sortie.rs:97-114`/`:463`, `rapports.rs:63`/`:104`,
  `grand_livre.rs:37`/`:56`/`:61`, `mcp_outils_e2e.rs:99`/`:182`/`:788`/`:1345`, `pdf.rs:153`/`:718`/`:1063`,
  `csv.rs:40`, `journal_report.rs:92`/`:177`) : exactes ; accord avec 37-1, 37-3a et 37-5 (`balances_as_of`) vérifié.
- 2026-11-14 — validation P6 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; aucun inter-fiches. Lentille sur le
  seul diff `c0801b02..c08ccd9b`. Relu au code : `pub(crate)` suffit (`kesh-report/src/lib.rs:18`, `:25`, `csv` et
  `pdf` dans le même crate) ; `Option<T: ChampSur>` admis (`mcp/sortie.rs:85-86`), `Montant` (`:215`, `:228`) et
  `Code<E>` (`:302`) disponibles pour montants, cours et devise ; « présents à `null` », macro non étendue : accord
  avec l'AC-6 de la fiche, 37-3a AC-8 et C-37-9. LOW P6-1 : l'entrée P5 porte un gabarit non rempli
  (« sauf <aucun inter-fiches> ») — à lire « aucun inter-fiches » ; relevé daté laissé tel quel.
- 2026-11-14 — révision CR-4 (revue 2 du responsable de l'epic T1–T6 ; Opus) : **T3** — la ligne ne stocke plus de devise (37-3a
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
  exacts ; `grep jel.currency|l.currency` sur `crates/` : aucun reste aujourd'hui ; aucun reste de la colonne dans 37-1,
  37-2, 37-3a, 37-3b, 37-5 (accord de 37-3a AC-1/AC-2 et de 37-5 sur `balances_as_of`).
- 2026-11-14 — validation CR-4 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW, remédiés dans la fiche ; aucun
  inter-fiches. CR4-P2-1 : la règle « la devise gouverne l'affichage » ne nommait que le grand livre et le journal, alors
  que le test T3 couvre aussi balance, bilan et compte de résultat (balance : AC-3 ; bilan, résultat : `CASE`) — nommés.
  CR4-P2-2 : la condition du journal (`amount_currency` non `NULL`) remplissait une ligne `revaluation` à 0 — alignée sur
  AC-2. Relu au code : `journal_report.rs:100-135` (`INNER JOIN accounts a`), `general_ledger.rs:338-342`, `:407-413`,
  `:465-473` (sans `accounts`, sommes CHF en `COALESCE`), `trial_balance.rs:165-170` (`x` sans `amount_currency`),
  `balance_sheet.rs:288-292`, `income_statement.rs:62-66` (`COALESCE` au CHF : « comme le CHF » exact) ; `grep -rnE
  "\b(jel|l)\.currency\b" crates/ --include=*.rs --include=*.sql` : vide, motif non échappé : vide aussi ; test T3 par
  SQL direct réalisable sous `chk_jel_fx_*` (37-3a AC-1 : trois colonnes cohérentes). Axes exercés : vérité au code des
  ajouts, cohérence interne (AC-1 ter, AC-2, AC-3, AC-7), points de contact 37-3a (contraintes, colonne absente).
  Non exercés : relecture complète de la fiche (hors périmètre), 37-5 (aucun ajout ne touche `balances_as_of`).
- 2026-11-14 — validation CR-5 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche sauf <aucun inter-fiches>. MEDIUM CR5-P1-1 : A29/U6 non appliqués (AC-3 et AC-4 disaient « restent CHF » et « totaux en CHF seulement » — réécrits « devise principale (aujourd'hui CHF) »). LOW CR5-P1-2 : « sommes CHF existantes » (AC-1) alignée ; les noms de champs `chf_net`, `total_debit` ne changent pas (code). Points de contact U1 : aucun — la fiche ne lit ni `exchange_rates` ni `_kesh_fetched_exchange_rates` (les rapports lisent `amount_currency` et `exchange_rate` de la ligne) ; aucun reste de `republished_*` ; A24/A28 sans objet ici ; la fiche ne crée ni table ni migration (pas de fenêtre d'import, rejeu, `TABLES_TO_TRUNCATE`). U6 : « CHF » restant légitime pour TVA, projets, accueil (base légale, AC-7). Relu au code : `opening.rs:75`, `general_ledger.rs:687`, `trial_balance.rs:165-170`/`:209` : exacts. Non exercés : GET de la source journalière (aucun rapport n'y touche), 37-5 (inchangée).
- 2026-11-14 — validation CR-5 P2 ciblée (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié dans la fiche, aucun inter-fiches. LOW CR5-P2-1 : propagation de U6 incomplète — le récit (« à côté du CHF ») et AC-2 (« `unnatural_balance` reste CHF ») gardaient la formule réécrite en AC-3/AC-4 ; alignés « devise principale ». Relu au code : `trial_balance.rs:69-83` (`total_debit`, `total_credit`, `balanced`, `retained_earnings`, `opening_balanced`) et `general_ledger.rs:137`, `:797` (`unnatural_balance` booléen tiré du solde CHF) : exacts. Cohérence : A29, U6 (epic l. 294, 321) appliqués ; « devise principale » définie par l'epic (l. 17) ; 37-5 (`chf_net`, `balances_as_of`) non touchée. Non exercés : relecture complète de la fiche (hors périmètre) ; AC-7 « CHF seulement » laissé tel quel (description du calcul, non du vocabulaire débit/crédit).
- 2026-11-14 — révision CR-6 (couverture API et MCP, Opus) : **V13** — AC-7 : l'outil MCP `kesh_postes_ouverts`
  (`mcp/outils/postes_ouverts.rs:210`, service `services/postes_ouverts.rs:98` → `letterings::open_items`) ajouté aux
  sites « CHF seulement » — un compte en devise lettrable y est atteignable —, renvoi à la limite du lettrage de 37-3a
  AC-9 ; test et tâche. Prémisses relues au code : `postes_ouverts.rs:125-161` (sorties `Poste`, `PostesOuverts`, en
  `Montant` seulement), `services/postes_ouverts.rs:98`, `letterings/open_items.rs:83-91`. AC touchés : AC-7, Tâches.
  Fiche à revalider (cycle CR-6).
- 2026-11-14 — validation CR-6 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW ; remédiés dans la fiche sauf <aucun inter-fiches>. LOW 37-4-CR6-P1-1 : l'inventaire AC-7 disait « sommes relevées par grep » et y rangeait un outil MCP sans `SUM` — précisé (atteint par `letterings::open_items`). LOW -P1-2 : l'agent MCP lit des CHF sans l'avoir dit (`solde`) — renvoi à `kesh_plan_comptable.devise` écrit ; test : `reste_du` à `null`. Relu au code : `mcp/outils/postes_ouverts.rs:210` (`nom`), `:125-161` (`Poste`, `PostesOuverts`, `Montant` seulement, aucun champ en devise), `services/postes_ouverts.rs:98` (`letterings::open_items`), `open_items.rs:83-91` (`SUM(jel.debit - jel.credit)`), `:346-355` (`require_viewable_account` : un compte de bilan en devise est atteignable) ; 37-3a AC-9 (limite du lettrage) cohérent ; epic l. 363 (V13). Non exercés : relecture complète hors diff CR-6, compilation, rendu, 37-1 AC-8 (`devise` de `kesh_plan_comptable`, citée, non relue).
- 2026-11-15 — exigences FR/NFR ajoutées (B26 ; orchestrateur, Sonnet) : 7 FR, 7 NFR.
- 2026-11-15 — validation ciblée des exigences (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW ; tous remédiés dans la section Exigences, sans exigence ajoutée (7 FR, 7 NFR, total de l'epic inchangé). MEDIUM 37-4-B26-1 : FR-37-4-1 omettait AC-1 bis (bornes d'exercice d'un compte de résultat, rupture) et AC-1 ter (compte listé à CHF 0 et devise ≠ 0) — ajoutés, avec la neutralisation d'une ligne en devise sur un compte CHF. MEDIUM -B26-2 : AC-1 quater (`balances_as_of`, contrat de 37-5 : borne inclusive, KF-002, `Some(&[])`, sans signe) sans aucune exigence — porté par FR-37-4-1. MEDIUM -B26-3 : FR-37-4-7 « le journal est étendu », lu avec FR-37-4-5, étendait le PDF et le CSV du journal, que l'AC-7 garde inchangés ; inventaire hors rapports (dont l'outil MCP `kesh_postes_ouverts`, V13) absent — précisé. LOW -B26-4 : FR-37-4-5 sans les quatre rapports visés ni les règles d'affichage (arrondi, formateurs, `format_exchange_rate`). LOW -B26-5 : FR-37-4-2 sans ouverture/clôture en devise, ligne `revaluation`, `unnatural_balance` en CHF. LOW -B26-6 : NFR-37-4-3 « aucun total en devise » contredisait à la lettre les totaux de section du grand livre (AC-2) — borné aux totaux de rapport. LOW -B26-7 : NFR-37-4-4 et -6 sans `mcp_gardes_lexicales`/`mcp_contrat`, test T3 (ligne SQL sur compte CHF), fixtures du commit de base. Axes exercés : chaque exigence contre l'AC entier cité (AC-1 à AC-9) ; points majeurs non couverts (contrat d'API, gardes, MCP ; l'AC ne porte ni audit, ni RBAC, ni sauvegarde) ; récapitulatif de l'epic. Non exercés : les AC eux-mêmes (hors périmètre), le code.
- 2026-11-15 — validation ciblée P2 des exigences (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW, remédiés dans la section Exigences, sans exigence ajoutée (7 FR, 7 NFR). LOW 37-4-B26-P2-1 : FR-37-4-5 sans la disposition du PDF de l'AC-5 (seconde ligne en petit du grand livre et de la balance, report en devise, devise entre nom et solde au bilan et au compte de résultat, aucune ligne de total) — ajoutée. LOW -B26-P2-2 : NFR-37-4-6 sans `balances_as_of` dans une transaction ni contrôle `grep` de T3 (AC-8) — ajoutés. Les 14 exigences relues contre l'AC entier cité : justes, aucune ne contredit un AC ni n'invente. Axes exercés : vérité de chaque exigence (AC-1 à AC-9 lus en entier), points majeurs sans exigence (contrat d'API, gardes, MCP, bornes), non-reprise de l'erreur MEDIUM de P1 (journal PDF/CSV inchangés, totaux de section), récapitulatif de l'epic. Non exercés : les AC eux-mêmes, le code.
- 2026-11-16 — validation CR-7 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW ; fiche non modifiée. Grep des résidus du mode par exercice (`currency-mode`, `force`, `MULTI_CURRENCY_IN_USE`, `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`, `laterFiscalYear`, `monoCurrencyFiscalYear`, désactivation, archivage forcé) : aucun. Les « mono-devise » de la fiche (NFR-2, AC-5, AC-6, non-régression) visent une société sans compte en devise, juste sous CR-7 ; la devise gouvernée par `a.currency` et le verrou de 37-1 AC-4 restent exacts ; aucun verrou de `companies` n'y est invoqué (D-CR7-1 sans effet). Axes exercés : résidus, contradiction avec 37-1 (A34, `multiCurrency` des exercices), 37-3a et 37-5, tests. Non exercés : code.
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : PDF du manuel non commité, `.tex` seulement (2 remplacements) ; CHANGELOG remplacé par la section *Note de version* (1 remplacement, section ajoutée) ; compteurs « recomptés depuis la source après chaque reprise de `main` » (1 ajout, plus la Note de version). Fond des AC inchangé.
- 2026-11-16 — **étape 0 du développement** (orchestrateur, relevé `<station>/kesh-gate-logs/etape0/37-4.md`,
  contre `6e0638e6`, 37-1 + 37-2) reportée ici : ≈ 95 prémisses relues, les lignes de `kesh-report` citées par la fiche
  presque toutes exactes. Écarts qui changeaient le travail : colonnes de ligne en devise absentes à 37-2 (« attend
  37-3a ») ; `accounts.currency` lu par aucune requête à étendre (`RawAccount` `general_ledger.rs:250`, `select_accounts`
  `:359`, `SELECT` du journal) ; lignes qui ont bougé — `sortie.rs:463` → `:493`, table `mesurees` `:181-200`,
  `Montant` `:218`, `Code<E>` `:269`, `homepage_summary.rs` sommes `:261-263`/`:304`, `require_viewable_account`
  `open_items.rs:295`, sites d'équilibre `kesh-db/.../journal_entries.rs:529`/`:930`/`:1689`, solde bancaire
  `kesh-db/src/repositories/bank_accounts.rs:1871-1905` (non `routes/`), commentaires `routes/bank_accounts.rs:245`/`:899`.
  Prémisses « attend 37-3a » recontrôlées au développement contre `946aa86a` : `journal_entry_lines.amount_currency`,
  `exchange_rate`, `rate_source` et `chk_jel_fx_*` présents (migration `20261114000003`), aucune colonne de devise sur
  la ligne ; `NewJournalEntryLine.foreign` (`entities/journal_entry.rs`), source `revaluation` posée par le dépôt
  (`journal_entry_fx.rs:169`, `:219`) ; `chk_jel_fx_amount` admet `revaluation` à 0 ; `JournalEntryLine::foreign()`
  rend `None` sur un compte sans devise. Exact ; aucun écart à la fiche.
- 2026-11-16 — **développement** (worktree `kesh-s37-4`, branche partie de `946aa86a`). Fixtures mono-devise enregistrées
  au commit de base avant toute modification (`88955a09`) ; implémentation `5f469154`, `38d2d3ed`, `494a6e9d`, `5680f714`.
  **Fusions** (sur instruction de l'orchestrateur) : `epic/37-multi-devises` (37-2 terminée, `65b3877f`) → `3d5b185d`,
  conflit unique sur `sitesTotal` de `i18n-keys.test.ts`, les deux intentions gardées, **3191 relevé par la garde**
  après la fusion (3185 de la revue de 37-2 + 6 de la 37-4), non additionné ; gate ciblé `-p kesh-db -p kesh-report`
  1908/1908, Vitest `reports`, `exchange-rates`, `shared` 602/602. Puis `story/37-3a-ecritures-en-devise-modele`
  jusqu'à `e89291e6` → merge sans conflit ; `sitesTotal` relevé de nouveau par la garde : 3191 ; migrations 94 = lignes
  du tableau d'audit 94 (aucune migration de la 37-4). Contrôle T3 (AC-8) :
  `grep -rnE "\b(jel|l)\.currency\b" crates/ --include=*.rs --include=*.sql` → **vide** (des variables Rust `l`
  nommées comme l'alias de ligne ont été renommées pour que le contrôle reste littéral). Inventaire AC-7 « CHF
  seulement » : aucun site neuf ; l'outil MCP `kesh_postes_ouverts` testé en CHF sur un compte de débiteurs USD (CR-6).
  Bornes MCP re-mesurées (AC-6) : `kesh_grand_livre` 30 / 15 → **26 / 14**, `kesh_balance` 55 / 30 → **42 / 23**
  (champs présents à `null` en CHF et remplis en devise, `tailles_mesurees` et `tailles_mesurees_en_devise`) ; bilan de
  57 comptes en devise : 20 156 o, seuil « environ 60 » conservé. Choix au registre `e37-choix-autonomes-37-4.md`
  (C-37-4-1 à C-37-4-9).
- 2026-11-16 — revue de code P1 (Opus ; Blind Hunter, Edge Case Hunter, Acceptance Auditor) : 0 CRITICAL, 0 HIGH,
  4 MEDIUM, 11 LOW retenus (dont 2 LOW non traités, à tracer : CR-P1-14, CR-P1-15), 9 rejetés ; 13 corrigés (section
  *Review Findings*). Rejetés : « cours MCP arrondi à 2 décimales » (HIGH du Blind Hunter, réfuté au code : `Montant`
  normalise et garde l'échelle, `mcp/sortie.rs:225-233`) ; arrondi des montants MCP (donnée, non affichage) ; libellés
  `fx_*` du PDF en français (AC-2 : `fr_ch_default`) ; réévaluation vide au CSV, 0 au JSON (AC-2) ; enregistrement des
  fixtures par variable (fixtures inchangées depuis `88955a09`, vérifié) ; débordement de la seconde ligne de la balance
  (largeur suffisante) ; `assert!` au lieu d'`unreachable!` (même effet, C-37-4-6) ; `formatRate(undefined)` (rend
  `''`) ; `tailles_mesurees_en_devise` séparé de `fixture_tailles` (forme, non fond). Notés hors périmètre : une ligne
  à `amount_currency` `NULL` sur un compte en devise se lit USD 0 au grand livre et CHF au journal — exclue par le
  dépôt de la 37-3a (`ACCOUNT_REQUIRES_CURRENCY`), non par le schéma ; `balances_as_of(None)` rend aussi les comptes
  de résultat en devise, cumul de tous les exercices (contrat de l'AC-1 quater) — à filtrer par la 37-5 si sa
  réévaluation ne vise que le bilan. Inventaire AC-7 relu après la fusion de la 37-3a : aucun site neuf ; sites
  d'équilibre à `journal_entries.rs:698`, `:1097`, `:1907`. Gates (journaux `kesh-gate-logs/s37-4/review-p1/`) : fmt,
  clippy workspace verts ; gate ciblé `package(kesh-report) | binary(account_balances) |
  test(rapports_en_devise_par_le_mcp) | binary(textes_coherents)` 239/239 ; **gate complet** (patch dans `kesh-db`)
  `cargo nextest run --profile ci` sur `kesh_s37_4` remise à zéro (base recréée, migrée, seedée) : 5755 passés,
  13 ignorés — `scripts/test-fast.sh --ci` lui-même arrêté par le garde des droits (`GRANT … kesh_s37_3a` posé hors
  du fichier d'init, signalé à l'orchestrateur), nextest lancé seul avec le même profil ; `npm run check` 0 erreur,
  `lint-i18n-ownership` vert, Vitest 3909/3909 (252 fichiers), `npm run build` vert ; E2E : `reports-currency.spec.ts`
  seule, 1/1 (suite complète non rejouée).

- 2026-11-16 — revue de code P2 (Sonnet ; Blind Hunter, Edge Case Hunter, Acceptance Auditor) : 0 CRITICAL, 0 HIGH,
  2 MEDIUM, 5 LOW retenus (3 non traités, à tracer : CR-P2-5 à CR-P2-7), 15 rejetés ou déjà connus ; 4 corrigés.
  Rejetés : libellés `fx_*` du PDF en français (AC-2, i18n du PDF déférée) ; bornes MCP 26/14 et 42/23 (re-mesurées,
  AC-6, écrites au Dev Agent Record) ; chevauchement de la rupture d'exercice avec le solde en devise (réfuté au rendu :
  libellé à ~80 mm, solde à 111 mm) ; E2E complet non rejoué (dû à la clôture, § D7). Rendu PDF relu : grand livre,
  balance, bilan et compte de résultat d'un jeu USD/EUR (grand livre et balance corrigés). Gate ciblé (aucun fichier de
  `kesh-db` touché) : `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings` vert, `cargo nextest run -p
  kesh-report` 207 passés, 1 ignoré ; `npm run check` 0 erreur, `lint-i18n-ownership` vert ; Vitest 3910/3910 (252 fichiers), `npm run build` vert. Décomptes périmés du Dev Agent Record (« 5752 », « 28 tests Rust ») : relevés à
  `5680f714`, non réécrits ; E2E complet et gate complet à la clôture.
- 2026-11-16 — clôture (agent de fin de story, base `kesh-mariadb-dev2` port 3307, bases `kesh_s37_4` / `kesh_e2e_s37_4`
  recréées ; journaux `kesh-gate-logs/s37-4/final3/`). Fusion de `epic/37-multi-devises` (37-3a terminée, `8c44bc6b`),
  faite sans conflit de texte : recomptée depuis la source — migrations **94** (`ls crates/kesh-db/migrations/*.sql`),
  égales à l'en-tête, à la ligne `Total` et aux lignes du tableau de `docs/migrations-idempotence-audit.md` ;
  `sitesTotal` de `i18n-keys.test.ts` = **3191**, tenu par la garde, Vitest vert ; registre des routes (garde du gate)
  vert ; bornes MCP des rapports : `fixture_tailles` pose la société et l'exercice multi-devises et un compte USD de la
  37-3a, `pagination_transmise` et `tailles_mesurees` verts dans le gate. Gate backend complet `scripts/test-fast.sh
  --ci` : **5781 passés, 13 ignorés** (833 s) ; Vitest **3910 / 252 fichiers** ; `npm run build` vert. E2E complet au
  dernier commit de code (`8c44bc6b`), `LANG=fr_CH.UTF-8` : **429 passés, 13 échecs, 18 ignorés** (47,6 min).
  Verdict : sept de la KF-029 ; `backup-restore-new-install.spec.ts:262` et `:381` et `opening-balances.spec.ts:156`,
  préexistants à `19bf12fb` (établis, non rejoués) ; `invoices-settle-unknown-outcome.spec.ts:117`,
  `invoices.spec.ts:338`, `journal-entries.spec.ts:486` rejoués seuls sur le même montage : **3/3 passés**
  (`e2e-rejeu.log`), donc échecs d'état partagé de la suite complète, sans rapport avec les rapports. Trois échecs qui
  changent d'identité, un de plus que les « zéro à deux » de `docs/testing.md` : relevé, non traité.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement), 2026-11-16.

### Completion Notes

- **AC-1 à AC-1 quater** : `kesh_db::repositories::account_balances` — `somme_en_devise(alias, côté, condition)`
  (fragment unique, sans condition de devise sur la ligne, `COALESCE`) et `balances_as_of` (borne inclusive, tous
  exercices, `None` = comptes en devise de la société, `Some(ids)` bornés à la société, `Some(&[])` sans requête,
  lecture dans la connexion fournie, sens débit brut). Ouvertures par `opening_from` sur la paire en devise ;
  solde progressif en devise dans le même `Cumul` (ruptures, lignes sautées par `offset`) ; clôture en devise d'un
  compte de résultat = exercice final (#778) ; inclusion CHF **ou** devise (grand livre, balance, bilan, résultat).
  Signe en SQL au bilan et au résultat par `sign_exprs` partagée, `NULL` pour un compte CHF.
- **AC-2 à AC-4** : champs en devise des structs (`skip_serializing_if`), devise lue sur le compte ; réévaluation à 0
  des deux côtés au JSON, « Réévaluation au cours X » à l'écran, au PDF et au CSV ; `unnatural_balance` en CHF ; aucun
  total en devise hors section du grand livre.
- **AC-5** : CSV à colonnes en devise seulement si le rapport contient un compte en devise (C-37-4-1), montants
  arrondis explicitement (`round_for_display`), cours par `format_exchange_rate` ; PDF : seconde ligne en petit (grand
  livre, balance), report en devise sur « À reporter » / « Report », devise et solde au bilan et au résultat.
- **AC-6** : JSON identique hors compte en devise (fixtures du commit de base, octet pour octet) ; cinq vues,
  composant `AccountCurrencyBalance` partagé, six clés i18n `reports-fx-*` dans les quatre locales ; MCP : champs
  présents à `null`, bornes re-mesurées.
- **AC-7** : journal étendu au JSON et à l'écran seulement (fixtures PDF et CSV du journal inchangées) ; accueil,
  projets, TVA (impôt préalable) et balance âgée testés en CHF ; `kesh_postes_ouverts` en CHF (CR-6).
- **AC-8** : jeu à deux exercices calculé à la main (bilan, résultat, passif, archivé, réévaluation, conversion),
  concordance des quatre rapports en devise, `offset > 0`, T3 (paire `afc` + `manual`, ligne SQL sur un compte CHF
  invisible des cinq rapports), contenus CSV et PDF, PDF multi-pages.
- **AC-9** : manuel utilisateur (§ *Les rapports d'une société multi-devises*) et tableau des outils MCP du manuel
  d'administration ; `.tex` seulement.
- **Écart de procédure** : la base `946aa86a` ne compilait pas `kesh-api` ; le MCP a été écrit puis compilé après la
  fusion de la 37-3a demandée par l'orchestrateur.

### Tests ajoutés (recomptés depuis la source, de `946aa86a` à la tête)

- Rust : `kesh-db/tests/account_balances.rs` 5 (+1 doctest de `somme_en_devise`) ; `kesh-report/tests/
  rapports_mono_devise.rs` 2, `rapports_en_devise.rs` 9, `rapports_chf_seulement.rs` 4 ; `kesh-report/src/pdf.rs`
  35 → 39 ; `trial_balance.rs` 3 → 4 ; `kesh-api/tests/mcp_outils_e2e.rs` 30 (pointe de la 37-3a) → 33. Total 28.
- Vitest : `GeneralLedgerView` 18 → 21, `TrialBalanceView` 6 → 8, `BalanceSheetView` 4 → 6, `IncomeStatementView` 4
  (neuf), `JournalReportView` 3 (neuf). Total 14.
- E2E : `reports-currency.spec.ts`, 1 (neuf).

### Gates réellement exécutés (journaux `<station>/kesh-gate-logs/s37-4/`)

- Au dernier commit de code `5680f714` : `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest profil `ci`
  sur `kesh_s37_4`) — **5752 passés, 13 ignorés** (`gate-backend-1.log`) ; frontend `npm run check` 0 erreur (29
  avertissements, aucun dans un fichier touché), `lint-i18n-ownership` PASS, `npm run test:unit` **3907 / 252
  fichiers** (`front-test-unit.log`), `npm run build` vert (`front-build.log`).
- E2E complet au même commit (`LANG=fr_CH.UTF-8`, port 3374, base `kesh_e2e_s37_4`, recette de `docs/testing.md`) :
  **432 passés, 10 échecs, 18 ignorés** (`e2e-1.log`, 44,7 min). Les dix, fichier par fichier : les sept de la KF-029
  (`mode-expert.spec.ts:26`, `:41` ; `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`) ;
  `backup-restore-new-install.spec.ts:262` et `:381`, rouges aussi aux E2E finals de la 37-2 (`s37-2/final2/e2e.log`)
  et de la 37-3a (`s37-3a/e2e-run.log`) sans rapport avec les rapports ; `opening-balances.spec.ts:156`, rejouée seule
  (`e2e-rejeu.log`, avec `reports.spec.ts` et `reports-currency.spec.ts` : 17 passés, elle seule rouge, délai du clic),
  déjà relevée par la 37-1 (toast d'ouverture qui recouvre le bouton, `issues-a-ouvrir-epic-37.md` n° 4). La spec neuve
  `reports-currency.spec.ts` passe aux deux exécutions.
- Clôture, au dernier commit de code `8c44bc6b` : voir l'entrée « clôture » du Change Log — gate backend 5781 passés
  (13 ignorés), Vitest 3910, build vert, E2E **429 passés, 13 échecs, 18 ignorés** (`final3/e2e.log`), dont trois
  rejoués seuls et passés (`final3/e2e-rejeu.log`).
- Captures (thème clair) prises par la spec E2E : `captures/37-4-grand-livre-usd.png`, `37-4-balance-usd.png`,
  `37-4-bilan-usd.png`. Le compte de résultat et le journal en devise ne sont couverts que par Vitest (pas de capture).
  *(Revue de code P1 : `review-p1/captures/37-4-resultat-usd.png` et `37-4-journal-usd.png` ajoutées, mêmes
  captures que ci-dessus reprises dans ce dossier.)*

### File List

- `crates/kesh-db/src/repositories/account_balances.rs` (neuf), `crates/kesh-db/src/repositories/mod.rs`,
  `crates/kesh-db/tests/account_balances.rs` (neuf)
- `crates/kesh-report/src/{general_ledger,trial_balance,balance_sheet,income_statement,journal_report,csv,pdf}.rs`,
  `crates/kesh-report/benches/export.rs`
- `crates/kesh-report/tests/{rapports_mono_devise,rapports_en_devise,rapports_chf_seulement}.rs` (neufs),
  `crates/kesh-report/tests/fixtures/mono_devise/{chf,multi_sans_devise}/*` (30 fichiers, neufs)
- `crates/kesh-api/src/mcp/outils/{grand_livre,rapports}.rs`, `crates/kesh-api/tests/mcp_outils_e2e.rs`,
  `crates/kesh-api/tests/common/mcp_pieges.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/reports/{GeneralLedgerView,TrialBalanceView,BalanceSheetView,IncomeStatementView,
  JournalReportView}.svelte`, `AccountCurrencyBalance.svelte` (neuf), `reports.api.ts`, `reports.types.ts`, tests
  `*.test.ts` (dont deux neufs), `frontend/src/lib/shared/i18n-keys.test.ts`,
  `frontend/tests/e2e/reports-currency.spec.ts` (neuf)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`
- `_bmad-output/implementation-artifacts/e37-choix-autonomes-37-4.md` (neuf), `sprint-status.yaml`, cette fiche
- 2026-11-16 — revue de code P3 ciblée (Haiku ; entrée écrite par l'orchestrateur, la passe n'ayant rien remédié) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW tracé (troncature des noms en devise au nombre de caractères et non à la largeur : un nom de 34 capitales larges touche le solde) ; la seconde ligne remontée ne chevauche rien, les fixtures mono-devise sont inchangées, le test de la remontée rougit avec `FX_RAISE_MM = 0` (exécuté). **Boucle de revue close** : P1 (Opus) 4 MEDIUM, 9 LOW corrigés → P2 (Sonnet) 2 MEDIUM, 2 LOW corrigés → P3 ciblée (Haiku) 0 au-dessus de LOW. Clôture (fusion des correctifs de la 37-3a, gates et E2E complet) après la fin de la 37-3a.
