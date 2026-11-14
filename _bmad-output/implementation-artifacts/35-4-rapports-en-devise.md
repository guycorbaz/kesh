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
     `SUM(CASE WHEN jel.debit > 0 THEN jel.amount_currency ELSE 0 END)` et son symétrique, filtrée
     `jel.currency = a.currency`. Une ligne `revaluation` (`amount_currency = 0`) n'y ajoute rien.
   - `signed`, `opening_from` et `debit_sense` (`opening.rs`, :75 pour `opening_from`) s'appliquent **sans
     changement** à la paire en devise (ce sont des fonctions sur `Decimal`) : aucun `CASE` d'ouverture n'est recopié
     en SQL, aucune seconde règle. « Fonction partagée » = fragment SQL de la somme en devise + `opening_from` ; la
     balance lit déjà 19 paramètres positionnels : un test exerce l'ordre des `bind`.
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
     nulle pour `select_accounts` du grand livre et la règle d'inclusion de la balance). Cas : conversion qui laisse un
     compte à CHF 0 et USD ≠ 0. `BalanceSheet::is_empty()` reste sur les listes.
   - **AC-1 quater — Fonction à date pour 35-5.** `balances_as_of(executor, company_id, as_of, account_ids)` →
     `Vec<(account_id, chf_signé, Option<devise_signé>)>`, sur un exécuteur générique (`impl sqlx::Executor` ou
     `&mut MySqlConnection`) pour que 35-5 lise dans sa transaction ; tous exercices, **sans borne basse ni `HAVING`**
     (un compte à solde nul reste rendu) ; les lignes `revaluation` comptent dans le CHF, pas dans la devise.
2. **AC-2 — Grand livre** (`general_ledger.rs` `generate` :644). Pour un compte en devise :
   - `RawLine`/`fetch_lines` lisent `currency`, `amount_currency`, `exchange_rate`, `rate_source` ; `LedgerLine` gagne le
     montant en devise (en débit/crédit comme le CHF), le cours (10 décimales stockées ; affichage comme `format_rate`
     de `pdf.rs`), le solde progressif en devise ; `LedgerSection` gagne `currency`, `openingCurrency`,
     `closingCurrency`, totaux en devise. `unnatural_balance` reste CHF.
   - Une ligne `revaluation` s'affiche « Réévaluation au cours X » sans montant en devise (libellé i18n, quatre
     locales, repris par JSON, PDF, CSV et TS) ; comptes CHF : inchangés.
   - Les structs `LedgerLine`/`LedgerSection` sont construites à la main dans `general_ledger.rs` (tests), `csv.rs`,
     `pdf.rs`, `tests/general_ledger.rs` : sites revus au Change Log.
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
     - Test de PDF multi-pages.
   - Les colonnes en devise sont portées par le JSON, le PDF et le CSV ; l'existant reste inégal (`lettering_code` JSON
     seul, C-15-1b-6).
   - **Affichage** : 2 décimales, somme exacte puis arrondi `MidpointAwayFromZero` ; cours via `format_rate` ;
     `format_swiss_amount` pour les montants.
6. **AC-6 — JSON, écran et MCP.**
   - Les champs en devise portent `#[serde(skip_serializing_if = "Option::is_none")]` : le JSON d'un rapport sans
     compte en devise est **identique** à celui d'avant (test d'égalité sur un JSON enregistré avant).
   - Vues (`GeneralLedgerView`, `TrialBalanceView`, `BalanceSheetView`, `IncomeStatementView`, `JournalReportView`),
     types TS (`reports.types.ts`), `breakBefore`/`trailingBreaks` de `GeneralLedgerView.svelte` (règle dupliquée en
     TS), `isReportEmpty`, i18n quatre locales (`i18n-keys`, `i18n-libelle-en-dur` verts). Mono-devise : rien de
     visible ne change (test).
   - **MCP** : `kesh_grand_livre` (`LIMITE_MAX_MCP = 30`), `kesh_balance` (`LIMITE_MAX_MCP_BALANCE = 55`),
     `kesh_bilan`, `kesh_compte_de_resultat` : champs en devise **omis** hors compte en devise ; limites recalculées
     pour le pire cas avec champs en devise sous `MCP_ENVOI_OCTETS_MAX` (`tests/mcp_outils_e2e.rs:104-150`) ; code de
     devise typé (`Code`/`TexteNonFiable`), `mcp_gardes_lexicales` et `mcp_contrat` verts.
7. **AC-7 — Rapports inchangés ou étendus : inventaire clos.**
   - **Journal** (`journal_report.rs:92`) : le JSON et l'écran gagnent les champs en devise (omis si absents) ; le PDF
     et le CSV du journal restent **inchangés**.
   - **CHF seulement**, chacun testé (une écriture USD y est comptée au CHF de sa ligne) :
     accueil (`homepage_summary.rs:252`, `:295` : graphiques en CHF) ; rapports de projets (`project_report.rs:263`,
     `:291`, `:456`) ; TVA (`vat_report.rs:261`, `:337`, `:368` : base légale CHF) ; balance âgée
     (`aged_receivables.rs` : factures, hors périmètre D7).
8. **AC-8 — Tests.** Jeu à deux exercices : compte USD de bilan, compte USD de **résultat** (ventes), compte de
   **passif** en devise, compte archivé ; achats et ventes à des cours différents, conversion, réévaluation.
   - La ligne `revaluation` est posée **par le dépôt** (`NewJournalEntryLine.foreign`, source `revaluation`), jamais
     par l'API (refusée par 35-3a AC-6). La contradiction avec 35-3a AC-3 (CHF = round(0 × cours)) est corrigée côté
     35-3a (exemption `revaluation`), portée par l'orchestrateur.
   - Soldes en devise et CHF attendus calculés à la main ; grand livre sur une période franchissant la fin d'exercice
     (rupture et clôture en devise) avec `offset > 0` (#775) ; compte à CHF 0 et devise ≠ 0 listé aux quatre rapports ;
     **concordance** grand livre ↔ balance ↔ bilan ↔ compte de résultat **en devise** (appel réel des générateurs,
     comme les tests CHF de `general_ledger.rs`) ; contenu CSV et PDF ; MCP ; `balances_as_of` dans une transaction.
   - Non-régression mono-devise : quatre CSV **octet pour octet** contre fixtures enregistrées **depuis le commit de
     base avant toute modification** (cas multi-devises sans compte en devise compris) ; JSON identique (AC-6).
9. **AC-9 — Manuel** : lecture des rapports en devise ; i18n ; PDF.

## Tâches

- [ ] `kesh-report` : paire en devise, `Cumul`, `balances_as_of`, quatre générateurs + journal ; PDF, CSV.
- [ ] Sites de construction à revoir (échec de compilation) : `AccountBalance` (`csv.rs` ≈6 tests, `pdf.rs` ≈8,
      `benches/export.rs:45`, `mcp/outils/rapports.rs:142`) ; `LedgerLine`, `LedgerSection`, `TrialBalanceRow`.
- [ ] `services/rapports.rs`, routes, MCP (limites).
- [ ] Frontend : cinq vues, types ; Vitest ; E2E (grand livre d'un compte USD).
- [ ] Manuel, CHANGELOG.

## Notes de développement

- `crates/kesh-report/src/` : `general_ledger.rs` (sommes :338-340, :465-468 ; `Cumul`), `trial_balance.rs`
  (:151-161), `balance_sheet.rs` (:289-290, :340), `income_statement.rs` (:63-64), `opening.rs`, `pdf.rs`, `csv.rs`.
- `crates/kesh-api/src/services/rapports.rs` (partagé avec MCP) ; `routes/reports.rs` sérialise les structs
  `kesh-report` directement.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Sonnet) : 0 CRITICAL, 3 HIGH, 10 MEDIUM, 3 LOW ; tous remédiés dans la fiche ; P1-11 (côté 35-3a) porté par l'orchestrateur.
