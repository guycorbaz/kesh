# Story 48-11 : Les montants à deux décimales sont arrondis au centime, non tronqués — rapports CSV et PDF ; l'export global écrit chaque colonne à son échelle (#1245 ; #1430)

Status: done

*Fiche rédigée le 2026-11-20 (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-48-11`, branche `story/48-11-arrondi-au-centime`, partie de
`epic/48-corrections-0-20-0` à **`08c47a6cc`** (qui porte la PR #1327 de Beat — alignement à droite des PDF de rapports — et la PR #1346 —
`round_for_display`). **Tout le code cité est lu à `08c47a6cc`.** Release **0.20.0** (coupure le jeudi 2026-11-26 au soir), story
**prioritaire** de l'Epic 48 (plan `release-0.20.0.md`, ligne 2 ; fichier d'épique § *48-11*). **Aucune migration** : aucun montant stocké ne
change ; seuls changent des fichiers produits — les rapports au demi-centime près, l'export global dans sa **forme** (chaque colonne à
l'échelle du schéma, décision de Guy du 2026-11-20, D5). Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e4811/`.
Clé courte des bases : `e4811` (`kesh_e4811`, `kesh_ci_e4811`, `kesh_e2e_e4811`).*

## Story

En tant que comptable qui tient ses livres dans Kesh et remet à son fiduciaire les rapports PDF et CSV et l'export global, je veux que
chaque montant à deux décimales des rapports y soit **arrondi au centime**, le demi-centime s'éloignant de zéro, comme à l'écran, et que
l'export global rende chaque valeur **sans perte**, à l'échelle où Kesh la garde, afin qu'un montant de 10.0050 ne s'imprime plus 10.00
dans un rapport quand l'écran dit 10.01, ni 10.00 dans l'export quand la base dit 10.0050.

## Ce qui se lit au code (`08c47a6cc`)

### Le défaut : `{:.2}` coupe les chiffres, il n'arrondit pas

`rust_decimal` **1.43.0** (`Cargo.lock`) : `impl fmt::Display for Decimal` passe la précision demandée à `to_str_internal`
(`rust_decimal-1.43.0/src/decimal.rs:2581` : `let (rep, additional) = crate::str::to_str_internal(self, false, f.precision());`), qui
recopie les `whole_len + prec` premiers chiffres de la mantisse (`src/str.rs:55` : `for i in 0..whole_len + prec {`) **sans arrondi**.
D'où : `format!("{:.2}", dec!(10.0050))` → `10.00` (attendu `10.01`), `5.0060` → `5.00`, `-10.0050` → `-10.00` (attendu `-10.01`), et
`-0.0040` → **`-0.00`** (le signe est rendu par `pad_integral(self.is_sign_positive(), …)`, `decimal.rs:2586`).

Les montants de Kesh sont stockés à **quatre décimales** (`DECIMAL(19,4)` : `debit`, `credit`, `quantity`, `unit_price`, `line_total`,
`total_amount`, `amount`) et s'y saisissent : le manuel utilisateur admet « quatre décimales au plus » pour le montant d'une écriture
(`docs/manual/fr/user-manual.tex:591`) et le prix unitaire (`:1441`). Une fraction de centime atteint donc les rapports et les exports.

### Le mode d'arrondi d'affichage de Kesh : l'équidistant loin de zéro

- `crates/kesh-report/src/pdf.rs:349-352` (PR #1346, Story 37-4) :
  ```rust
  pub(crate) fn round_for_display(amount: Decimal) -> Decimal {
      let r = amount.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
      if r.is_zero() { Decimal::ZERO } else { r }
  }
  ```
  — deux décimales, **`MidpointAwayFromZero`**, et un zéro rendu **sans signe** (`-0.0040` → `0`, pas « -0.00 »).
- Le même mode partout où Kesh arrondit déjà avant d'afficher : `kesh-core/src/types/money.rs:66-69` (`Money::round_to_centimes`,
  `.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)`), `kesh-api/src/routes/company_invoice_settings.rs:48`
  (`value.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero)`, « `5.0050` s'affiche `5.01` », `:41`),
  `kesh-api/src/routes/reconciliation_group.rs:641` (`fixed`), `kesh-qrbill/src/pdf.rs:1676` (`format_ch`), `kesh-qrbill/src/generator.rs:114`,
  `kesh-i18n/src/formatting.rs:17` (`format_money`).
- À l'écran, les rapports passent par big.js : `frontend/src/lib/features/reports/reports.api.ts:280-282` (`formatReportAmount` →
  `formatSwissAmount(new Big(v))`) puis `frontend/src/lib/features/journal-entries/balance.ts:247` (`const fixed = big.toFixed(decimals);`),
  `toFixed` de big.js arrondissant au demi « loin de zéro » (mode par défaut `Big.RM = 1`, `roundHalfUp` ; cité par
  `invoice-helpers.ts:60-61`). **Aujourd'hui l'écran d'un rapport dit 10.01 et son PDF 10.00.**
- **À ne pas confondre** : `Decimal::round_dp(n)` arrondit **au pair** (`rust_decimal-1.43.0/src/decimal.rs:1654` :
  `self.round_dp_with_strategy(dp, RoundingStrategy::MidpointNearestEven)`) ; `Decimal::rescale(n)` arrondit **loin de zéro** (exemple de
  la doc, `decimal.rs:1033-1037` : `1.45` → `rescale(1)` → `"1.5"`).

**Exemple au demi-centime (la règle de la story)** : 10.0050 → **10.01** ; 10.0049 → 10.00 ; 5.0060 → 5.01 ; −10.0050 → **−10.01** ;
−0.0040 → **0.00** (sans signe) ; 1234.5 → 1234.50 (CSV) / 1'234.50 (PDF).

### Inventaire par le symptôme

Le symptôme : un `Decimal` écrit à **précision fixe** sans arrondi préalable. Quatre recherches, sur **tous les crates** (`crates/*/src`,
tests compris pour qu'aucune forme n'échappe) et le **frontend** ; sorties copiées dans les journaux.

**R1 — toute précision de format** (`{x:.2}`, `{:.2}`, `{x:>10.2}`, `{x:.scale$}`) :

```
$ grep -rnE '\{[A-Za-z_0-9.]*:[^{}"]*\.[0-9]+\}' crates --include=*.rs | grep -v '/tests/'   # inv-precision-large.txt, 30 lignes
$ grep -rnE ':\.[a-z_0-9]+\$\}|:\.\*\}' crates --include=*.rs | grep -v '/tests/'             # inv-precision-variable.txt
```

Décompte **par occurrence**, hors lignes `//` (simulation de la garde, `garde-simulation-avant.txt`) : **30 occurrences dans 10 fichiers** :

```
3 crates/kesh-api/src/exports/csv_tables.rs
1 crates/kesh-api/src/routes/bank_statements_manual.rs
1 crates/kesh-api/src/routes/company_invoice_settings.rs
2 crates/kesh-core/src/exchange_rates/parse/tests.rs
1 crates/kesh-i18n/src/formatting.rs
1 crates/kesh-payment/src/pain001/mod.rs
1 crates/kesh-qrbill/src/generator.rs
13 crates/kesh-qrbill/src/pdf.rs
2 crates/kesh-report/src/csv.rs
5 crates/kesh-report/src/pdf.rs
total 30
```

**R2 — toute fonction `Decimal → String`** (pour voir un formatage maison que R1 ne verrait pas) :
`grep -rnE 'fn [a-z_0-9]+\([^)]*: (Option<)?(&)?Decimal[^)]*\)\s*->\s*(String|Cow|Option<String>)' crates --include=*.rs | grep -v '/tests/'`
→ 21 fonctions (`inv-fonctions-decimal-vers-string.txt`), toutes classées ci-dessous.

**R3 — découpage maison au point décimal** : `grep -rnE "(split|find|rfind|split_once)\('\.'\)" crates --include=*.rs | grep -v '/tests/'`
→ 9 sites (`inv-decoupage-maison.txt`) : `kesh-report/src/pdf.rs:325` (dans `format_swiss_amount`, après le `{:.2}` : traité avec lui),
`kesh-qrbill/src/pdf.rs:1678` et `kesh-i18n/src/formatting.rs:23` (après un arrondi `MidpointAwayFromZero`), `kesh-report/src/pdf.rs:3312`
(test), `kesh-core/src/exchange_rates/parse.rs:672`, `kesh-core/src/bank_imports/manual.rs:761`, `kesh-import/src/csv/parser.rs:831` (lecture
d'une saisie, non formatage), `kesh-api/src/admin_backup/{import,export}.rs` (noms de pièces). **Aucun ne tronque un montant.**

**R4 — troncature et arrondi explicites** : `round_dp(`, `round_dp_with_strategy`, `RoundingStrategy::`, `.trunc(`, `trunc_with_scale`,
`.rescale(`, `set_scale(` (`inv-arrondis.txt`). Troncatures : `kesh-core/src/bank_imports/manual.rs:800` (`n.abs().trunc() >= limit`, borne
de la partie entière d'une saisie) et `kesh-api/src/routes/mcp_server_settings.rs:126` (`reseau.trunc()`, une adresse IP) — **ni l'une ni
l'autre n'affiche un montant**. Les `rescale(2)` (`errors.rs`, `reconciliation.rs`, `reconciliation_link.rs`, `mcp/sortie.rs:229`,
`fx_revaluation.rs:174`) arrondissent loin de zéro ou complètent des zéros : sains. Les `round_dp(` (arrondi au pair) sont classés en fin
de tableau.

**R5 — frontend** : `grep -rnE 'toFixed\(|maximumFractionDigits|Intl\.NumberFormat|Math\.round\(|Math\.trunc\(|Math\.floor\(' frontend/src
--include=*.ts --include=*.svelte | grep -v '\.test\.ts'` → 78 lignes (`inv-frontend.txt`). Les montants y passent par `Big.toFixed`
(arrondi au demi loin de zéro) : `formatSwissAmount` (`balance.ts:244-257`), `formatReportAmount`, `formatAmount` (`open-items-labels.ts:24`),
`formatExactAmount`/`formatInvoiceTotal` (`invoice-helpers.ts:92`, `:102`), `formatForeignAmount` (`fx-display.ts:26`),
`formatCreditNoteTotal`, `formatSupplierInvoiceTotal`, `formatBatchTotal` ; `formatChfBalance` (`bank-accounts/format.ts:72`) arrondit au
centime avant `Intl` (`toCentimeNumber`) ; `formatChfExact` (`:369`) passe la chaîne entière à `Intl`. `ProjectReturnView.svelte:17`
(`Number(pct).toFixed(2)`) reçoit un rendement déjà arrondi à deux décimales par le serveur (`project_report.rs:437`). **Le frontend
n'a pas le défaut.**

#### Les sites, classés (ensemble clos : les 30 occurrences de R1 et les 21 fonctions de R2)

| Site (`08c47a6cc`) | Ligne lue | Ce qu'il formate | Classement |
|---|---|---|---|
| `kesh-report/src/pdf.rs:316-318` `format_swiss_amount` (31 appels hors tests, dont `ledger_amount` `:1819` et `format_currency_amount`) | `let raw = format!("{amount:.2}");` (sous le commentaire faux « gère signe et arrondi », `:317`) | tous les montants CHF des PDF de rapports | **corrigé** (D3) |
| `kesh-report/src/pdf.rs:1394-1395` `format_rate` (2 appels) | `format!("{rate:.2} %")` | taux de TVA du rapport TVA (`DECIMAL(5,2)` : ne tronque rien aujourd'hui) | **corrigé** (D4) |
| `kesh-report/src/pdf.rs:1666-1668` `format_rendement` (1 appel) | `Some(p) => format!("{p:.2}%"),` | rendement d'un projet (déjà arrondi au pair, `project_report.rs:437`) | **corrigé** (D4) |
| `kesh-report/src/pdf.rs:355-359` `format_currency_amount` (5 appels) | `format_swiss_amount(round_for_display(amount))` | montants en devise (Story 37-4) | **sain** ; simplifié par D3 |
| `kesh-report/src/pdf.rs:899`, `:1116` ; `csv.rs:225` — appels **directs** de `round_for_display` | `round_for_display(bs.retained_earnings),` ; `format_swiss_amount(round_for_display(v.unwrap_or_default()))` ; `&format_amount_iso(round_for_display(bs.retained_earnings)),` | ligne « Résultat reporté (calculé) » du bilan (PDF, CSV), montants en devise de la balance PDF | **sains**, redondants après D3 : **retirés** (D3) |
| `kesh-report/src/pdf.rs:368-373` `format_exchange_rate` | `format!("{r:.prec$}", prec = scale as usize)` après `round_dp_with_strategy(10, …MidpointAwayFromZero)` et `prec = scale.max(4)` | cours de change | **assumé** : la précision n'est jamais inférieure à l'échelle de la valeur (elle **complète**, ne coupe pas) |
| `kesh-report/src/pdf.rs:3337` | `"{nom} : « {} » finit à {bord:.3} mm, …"` | un `f32` de géométrie, dans `mod tests` (`:2161`) | **assumé** : pas un `Decimal` |
| `kesh-report/src/csv.rs:50-51` `format_amount_iso` (62 appels hors tests) | `format!("{amount:.2}")` | tous les montants CHF des CSV de rapports | **corrigé** (D3) |
| `kesh-report/src/csv.rs:58-59` `format_currency_amount_iso` | `format_amount_iso(round_for_display(amount))` | montants en devise | **sain** ; simplifié par D3 |
| `kesh-report/src/csv.rs:745-747` `format_rendement` (2 appels) | `Some(p) => format!("{p:.2}%"),` | rendement d'un projet | **corrigé** (D4) |
| `kesh-report/src/balance_sheet.rs:69`, `:142` | `use crate::pdf::round_for_display;` / `!round_for_display(self.retained_earnings).is_zero()` | le modèle du bilan importe une fonction du module de rendu (**#1430**) | **corrigé** (D2) |
| `kesh-api/src/exports/csv_tables.rs:108-109` `fmt_decimal` (29 appels, plus 5 de `fmt_opt_decimal` `:113` : 34 sites, classés par colonne en D5) | `format!("{d:.2}")` | export global : débit, crédit, prix, quantité, taux, totaux, frais, soldes de relevé, réglages de facturation | **corrigé** (D5 : chaque colonne à l'échelle du schéma, sans arrondi) |
| `kesh-api/src/exports/csv_tables.rs:119-120` `fmt_opt_decimal_scale` (2 appels, `:430-431`) | `d.map(\|d\| format!("{d:.scale$}"))` | `amount_currency` à 4, `exchange_rate` à 10 | **sain** ; devient le formateur unique de l'export (D5) : précision égale à l'échelle de la colonne, elle complète et ne coupe pas |
| `kesh-api/src/exports/csv_tables.rs:880-881` `fmt_rate` (`fmt_opt_rate` `:886`) | `format!("{d:.10}")` | cours `DECIMAL(20,10)` (`exchange_rates.rate`, `acknowledged_rate`) | **sain** ; délègue au formateur unique (D5, DRY) |
| `kesh-api/src/routes/bank_statements_manual.rs:551-555` `amount_string` (14 appels : réponses, détails d'audit, `BankStatementUnbalanced`) | `format!("{:.2}", d)` | sommes d'un relevé saisi (bornées à deux décimales, `kesh-core/src/bank_imports/manual.rs:43` `MANUAL_AMOUNT_MAX_DECIMALS: u32 = 2`), et le solde de clôture du relevé **précédent**, lu en base (`opening_gap`, `:523-547`, par `bank_imports::previous_statement_closing`, `kesh-db/src/repositories/bank_imports.rs:368`, colonne `closing_balance` `DECIMAL(18,2)`) : **ne tronque rien aujourd'hui** — aucune valeur reçue n'a plus de deux décimales | **délégation défensive**, comme D4 : passe par la fonction commune pour la garde et pour la première valeur à trois décimales ; l'AC6 (`10.0050`) teste la délégation (D3) |
| `kesh-api/src/routes/company_invoice_settings.rs:45-49` `group_max_for_display` | `"{:.2}",` sur `value.round_dp_with_strategy(2, …MidpointAwayFromZero)` | plafond de l'écart d'un règlement groupé | **sain** ; **délègue** à la fonction commune (D3, DRY), tests existants `:599-600` en filet |
| `kesh-api/src/routes/reconciliation_group.rs:640-646` `fixed` | `round_dp_with_strategy(dp, MidpointAwayFromZero)` puis `rescale(dp)` | candidates d'un règlement groupé | **sain** (pas de précision de format) ; hors périmètre (D3) |
| `kesh-api/src/errors.rs:1566` `chf_text` | `n.rescale(2)` si `scale < 2`, sinon valeur entière | refus `FX_AMOUNT_MISMATCH` : **sans arrondi voulu** (`:1561-1564`) | **sain** |
| `kesh-qrbill/src/generator.rs:110-115` `format_amount_payload` | `format!("{:.2}", rounded)` après `MidpointAwayFromZero` (`:114`) | montant de la QR-facture | **sain**, **non touché** (factures émises, D7) |
| `kesh-qrbill/src/pdf.rs:1665-1698` `format_signed_ch`, `format_ch` | `round_dp_with_strategy(decimals, …MidpointAwayFromZero)` (`:1676`) | montants du PDF des factures et avoirs | **sain**, **non touché** (D7) |
| `kesh-qrbill/src/pdf.rs` (13 occurrences : `:84`, et `:4299` à `:5400` dans `mod tests` `:1901`) | `{:.3}mm`, `({:.2}, {:.2})` | `f32` de géométrie (module du QR, positions) | **assumé** : pas un `Decimal` |
| `kesh-payment/src/pain001/mod.rs:124-126` `amt` | `format!("{:.2}", d.round_dp(2))` | montants du pain.001 | **sain** pour la troncature (arrondi avant) ; **arrondi au pair** : hors périmètre, issue **#1441** (décision de Guy du 2026-11-20, Q2) |
| `kesh-i18n/src/formatting.rs:16-22` `format_money` | `format!("{:.2}", abs)` après `MidpointAwayFromZero` (`:17`) | formatage i18n | **sain** ; `kesh-i18n` ne dépend d'aucun crate interne : non migré (D3) |
| `kesh-core/src/exchange_rates/parse/tests.rs:335`, `:339` | `"{kurs:.20}"` | message d'un test | **assumé** : test, pas un montant |
| `kesh-report/src/project_report.rs:437` `rendement_pct` | `.round_dp(2)` | rendement **au pair** (12.345 → 12.34) | hors symptôme (arrondi d'un autre mode, non troncature) : issue **#1442** (I1) |
| `kesh-db/src/repositories/invoices.rs:459-461` `compute_line_total` | `(qty * unit_price).round_dp(4)` | **total de ligne stocké, au pair**, alors que l'écran l'estime au demi supérieur (`invoice-helpers.ts:31` `.round(4, ROUND_HALF_UP)`) | hors symptôme, **valeur stockée d'une facture** : issue **#1443** (I2), jamais dans cette story |
| `kesh-core/src/accounting/vat.rs:148` | `rate_percent.round_dp(2)` | taux `DECIMAL(5,2)` : sans effet | sain |

**Le défaut de #1245 se trouve à cinq sites** (`format_swiss_amount`, `format_amount_iso`, `fmt_decimal`, `amount_string`, et les deux
formateurs de taux et rendement, `{:.2}` sur un `Decimal` qui ne tronquent rien aujourd'hui mais tronqueraient à la première valeur à
trois décimales). **Le PDF des factures (`kesh-qrbill`) n'a pas le défaut** : il arrondit déjà (`format_ch`, `:1676`) ; rien n'y est à
proposer. Le remède diffère selon le site : les rapports et `amount_string` **arrondissent** au centime (D3, D4) ; l'export global
**cesse de couper** et écrit chaque colonne à son échelle (D5, décision de Guy du 2026-11-20, Q1).

**Ce qui sort à tort aujourd'hui, concrètement** : un rapport (journal, grand livre, balance, bilan, compte de résultat, TVA, échéancier,
projets) en PDF ou CSV dont une ligne ou un total porte une fraction de centime ; l'export global (débit/crédit d'une écriture saisie à
quatre décimales, total de ligne d'une facture à quantité ou prix fractionnaire, réglage `group_settlement_max_difference` posé à
`5.0060` — que le manuel d'administration décrit, `admin-manual.tex:3915`) ; un résidu négatif sous le demi-centime imprimé « -0.00 ».

### #1430 — le bilan importe sa règle d'arrondi du module de rendu

`kesh-report/src/balance_sheet.rs:69` `use crate::pdf::round_for_display;` et `:142` `!round_for_display(self.retained_earnings).is_zero()` :
le modèle dépend du module de présentation. La fonction commune de cette story (dans `kesh-core`) **absorbe** `round_for_display` :
`balance_sheet.rs` l'importe de `kesh_core`, `pdf.rs` et `csv.rs` aussi, et le CSV du bilan arrondit comme sa ligne affichée — les deux
griefs de #1430. **#1430 se ferme avec la story** (mot-clé de fermeture dans le corps de la PR d'intégration, jamais dans un commit) :
**décision de Guy du 2026-11-20**, l'issue est rattachée au milestone **v0.20.0** (elle était à « Plus tard »).

`csv.rs:25` (`use crate::pdf::{format_exchange_rate, round_for_display};`) garde après la story l'import de `format_exchange_rate` depuis
`pdf.rs` : une dépendance du CSV au module PDF **subsiste**, entre deux modules de présentation ; elle n'est pas celle de #1430 (le
**modèle** du bilan qui dépend du rendu) et reste hors périmètre.

### Dépendances

`kesh-report` et `kesh-api` dépendent de `kesh-core` (`crates/kesh-report/Cargo.toml`, `crates/kesh-api/Cargo.toml`) ; `kesh-i18n` et
`kesh-payment` d'aucun crate interne. Le module `money` de `kesh-core` est privé, réexporté par `kesh-core/src/types/mod.rs:13`
(`pub use money::Money;`).

### Manuels

`git grep -nE '#1245|\\#1245|tronqu' -- 'docs/manual/*/*.tex'` : un seul site, `docs/manual/fr/admin-manual.tex:3915`, section « Écart admis
pour un règlement de plusieurs factures » (présente en **français seulement** : `git grep -n 'ecart-admis' -- 'docs/manual/*/*.tex'` ne rend
que `fr/admin-manual.tex`) : « l'export CSV des réglages la \textbf{tronque} à deux décimales (\texttt{5.00}, défaut connu, issue~\#1245) ;
l'écran et l'API l'\textbf{arrondissent} (\texttt{5.01}) ». Après la story, la phrase devient fausse : elle se réécrit (D8). Aucun autre
manuel ne décrit la précision des exports (`git grep -nE 'deux décimales|two decimal|zwei Dezimal|due decimali' -- 'docs/manual/*/*.tex'`
filtré sur `csv|export|rapport|report|pdf` : seuls `admin-manual.tex:3915` et `user-manual.tex:1512`, ce dernier sur la saisie du seuil).

## Décisions

- **D1 — le mode d'arrondi** : deux décimales, **`MidpointAwayFromZero`**, zéro rendu sans signe — celui de `round_for_display`
  (`pdf.rs:349-352`), de `Money::round_to_centimes`, du reste dû et de l'écran (big.js). Exemple : 10.0050 → 10.01, −10.0050 → −10.01,
  −0.0040 → 0.00. **Option écartée** : `round_dp(2)` (au pair, 10.0050 → 10.00 — l'écran et le reste dû diraient autre chose).
- **D2 — la fonction commune vit dans `kesh-core`**, `crates/kesh-core/src/types/money.rs`, réexportée par `types/mod.rs` :
  - `pub fn round_for_display(amount: Decimal) -> Decimal` — **déplacée** de `kesh-report/src/pdf.rs:349` (corps et doc-comment
    identiques ; la mention « les formateurs ne font qu'un `{:.2}` » se réécrit) ;
  - `pub fn format_two_decimals(amount: Decimal) -> String` — `round_for_display` puis **exactement deux décimales**, point décimal, sans
    séparateur, signe `-` seulement si le montant arrondi est non nul. **Seul `{:.2}` admis sur un `Decimal` de montant** dans le dépôt :
    la valeur arrondie a une échelle ≤ 2, la précision ne fait plus que compléter (l'autre précision admise sur un montant est celle de
    l'export global, égale à l'échelle de la colonne, D5).
  `kesh-report/src/pdf.rs`, `csv.rs` et `balance_sheet.rs` importent de `kesh_core::types` ce qu'ils emploient (`pdf.rs` et `csv.rs` :
  `format_two_decimals` ; `balance_sheet.rs` : `round_for_display`). Ferme **#1430** (dépendance à rebours retirée, une seule règle
  d'arrondi dans un même rapport). **Option écartée** : un module d'arrondi dans `kesh-report` (laisse `kesh-api`
  sans la fonction, et `kesh-api` en a besoin) ; **option écartée** : une méthode sur `Money` (les sites manipulent des `Decimal` nus ;
  les envelopper ajoute du bruit sans garantie de plus).
- **D3 — les sites corrigés délèguent, sans se dupliquer** :
  - `format_amount_iso` (`csv.rs:50`) = `format_two_decimals(amount)` ; `format_currency_amount_iso` (`:58`) = `format_amount_iso(amount)`
    (le second arrondi devient redondant) ;
  - `format_swiss_amount` (`pdf.rs:316`) part de `format_two_decimals(amount)` au lieu de `format!("{amount:.2}")` ; le commentaire `:317`
    devient vrai ; `format_currency_amount` (`:355`) n'appelle plus `round_for_display` lui-même ;
  - **les appels directs de `round_for_display` dans un formatage se retirent** : `pdf.rs:358` et `csv.rs:59` (ci-dessus),
    `pdf.rs:899` (`bs.retained_earnings` passe tel quel : la boucle le formate par `format_swiss_amount`, qui arrondit), `pdf.rs:1116`
    (`format_swiss_amount(v.unwrap_or_default())`), `csv.rs:225` (`format_amount_iso(bs.retained_earnings)`). Après la story,
    `round_for_display` ne s'appelle plus hors de `kesh-core` qu'en **un** site de production, `balance_sheet.rs:142` (un prédicat, non un
    formatage), et dans le test `pdf.rs:3102-3107` : `pdf.rs` ne l'important plus en production, le module de tests reçoit **une** ligne
    `use kesh_core::types::round_for_display;` (sinon le `use` de production serait inutilisé et `clippy -D warnings` rougirait) ; ses
    assertions ne changent pas. Le lien intra-doc de `balance_sheet.rs:126` vise `kesh_core::types::round_for_display` ;
  - `fmt_decimal` (`csv_tables.rs:108`) **ne délègue pas** à la fonction commune : l'export global n'arrondit pas (D5) ;
  - `amount_string` (`bank_statements_manual.rs:551`) = `format_two_decimals(d)` (le traitement du zéro négatif, `:552-554`, est porté
    par la fonction commune) ;
  - `group_max_for_display` (`company_invoice_settings.rs:45`) = `format_two_decimals(value)` (aucun changement de sortie ; DRY).
  **Non migrés, et pourquoi** : `kesh-qrbill` (factures émises, D7), `kesh-i18n::format_money` et `kesh-payment::amt` (crates sans
  dépendance à `kesh-core` ; les y lier pour une ligne saine n'en vaut pas le coût), `reconciliation_group::fixed` (précision variable
  `dp`, déjà juste ; sa fusion avec la fonction commune est un DRY possible, non un défaut — hors périmètre).
- **D4 — taux et rendements** : `format_rate` (`pdf.rs:1394`) et les deux `format_rendement` (`pdf.rs:1666`, `csv.rs:745`) passent par
  `format_two_decimals` (suivi de ` %` ou `%`, formes inchangées). Ils ne tronquent rien aujourd'hui (taux `DECIMAL(5,2)`, rendement
  arrondi en amont) ; les y faire passer laisse **un seul** `{:.2}` sur un `Decimal` dans `kesh-report`, ce qui rend la garde simple.
  **Option écartée** : les laisser, nommés à la garde (une exception de plus par fichier, et la première valeur à trois décimales
  tronquerait en silence). Le **mode au pair** du rendement (`project_report.rs:437`) reste : issue I1.
- **D5 — l'export global écrit chaque colonne à l'échelle de sa colonne dans le schéma, sans arrondi** (**décision de Guy du
  2026-11-20, Q1**, qui remplace le remède du kickoff — « arrondir au centime », fichier d'épique § *48-11*). C'est la règle que la
  Story 37-3a a posée pour `amount_currency` et `exchange_rate` (`csv_tables.rs:430-431`), étendue à toutes les colonnes `DECIMAL` de
  l'export. Une valeur lue en base a au plus l'échelle de sa colonne : une précision **égale** à cette échelle complète par des zéros et
  ne coupe jamais. Exemples : un débit de `10.0050` sort `10.0050` (non `10.01`, ni `10.00`) ; une quantité de `0.125` sort `0.1250` ;
  `150.25` sort `150.2500` ; un taux de TVA `8.10` sort `8.10`.
  - **Un seul formateur dans `csv_tables.rs`** : `fmt_decimal` et `fmt_opt_decimal` disparaissent ; `fmt_decimal_scale(d: Decimal,
    scale: usize) -> String` (`format!("{d:.scale$}")`) et `fmt_opt_decimal_scale` (`:119`, qui l'emploie) servent les 38 colonnes ;
    `fmt_rate` (`:880`) devient `fmt_decimal_scale(d, DEC_20_10)` (DRY). L'échelle s'écrit à chaque site par une **constante nommée
    d'après le type SQL** — `DEC_19_4 = 4`, `DEC_18_2 = 2`, `DEC_7_2 = 2`, `DEC_5_2 = 2`, `DEC_20_10 = 10` —, si bien qu'un relecteur
    confronte le site au schéma sans calcul. **Option écartée** : deux formateurs à échelle figée (`fmt_dec4`, `fmt_dec2`) — les
    cours à dix décimales en demanderaient un troisième, et l'échelle n'y serait plus lisible au type SQL.
  - **Classement des 34 sites de `fmt_decimal`/`fmt_opt_decimal`** (lignes à `08c47a6cc` ; type lu au schéma,
    `crates/kesh-db/test-schema/0001_schema_squash.sql`, que `test_schema_guard.rs` tient égal aux migrations ; extraction dans
    `kesh-gate-logs/e4811/q1-echelles-colonnes.txt`) — **ensemble clos** :

    | Échelle | Constante | Sites (`csv_tables.rs`) | Colonnes | Nombre |
    |---|---|---|---|---|
    | 4 | `DEC_19_4` | `:419`, `:420` | `journal_entry_lines.debit`, `.credit` | 2 |
    | 4 | `DEC_19_4` | `:537` | `products.unit_price` | 1 |
    | 4 | `DEC_19_4` | `:597`, `:598` | `invoices.total_amount`, `.rounding_amount` | 2 |
    | 4 | `DEC_19_4` | `:656`, `:657`, `:659` | `invoice_lines.quantity`, `.unit_price`, `.line_total` | 3 |
    | 4 | `DEC_19_4` | `:1103` (opt.), `:1107` | `company_invoice_settings.minimum_invoice_amount`, `.group_settlement_max_difference` | 2 |
    | 4 | `DEC_19_4` | `:1259`, `:1260` | `credit_notes.total_amount`, `.rounding_amount` | 2 |
    | 4 | `DEC_19_4` | `:1297`, `:1298`, `:1300` | `credit_note_lines.quantity`, `.unit_price`, `.line_total` | 3 |
    | 4 | `DEC_19_4` | `:1355`, `:1359` (opt.) | `supplier_invoices.total_amount`, `.expected_payment_amount` | 2 |
    | 4 | `DEC_19_4` | `:1405`, `:1406`, `:1408` | `supplier_invoice_lines.quantity`, `.unit_price`, `.line_total` | 3 |
    | 4 | `DEC_19_4` | `:1447` | `payment_batches.total_amount` | 1 |
    | 4 | `DEC_19_4` | `:1484` | `payment_batch_items.amount` | 1 |
    | 4 | `DEC_19_4` | `:1523` | `invoice_settlements.amount` | 1 |
    | 4 | `DEC_19_4` | `:1755` (opt.) | `imported_supplier_invoices.amount` | 1 |
    | 2 | `DEC_5_2` | `:538`, `:658`, `:1299`, `:1407`, `:863` | `products.vat_rate`, `invoice_lines.vat_rate`, `credit_note_lines.vat_rate`, `supplier_invoice_lines.vat_rate`, `vat_rates.rate` | 5 |
    | 2 | `DEC_7_2` | `:962`, `:1035` | `dunning_levels.fee_amount`, `invoice_reminders.fee_amount` | 2 |
    | 2 | `DEC_18_2` | `:764` (opt.), `:765` (opt.), `:814` | `bank_imports.opening_balance`, `.closing_balance`, `bank_transactions.amount` | 3 |

    **24 sites à quatre décimales, 10 à deux : 34.** S'y ajoutent les quatre colonnes déjà à leur échelle — `amount_currency`
    (`DEC_19_4`, `:430`), `journal_entry_lines.exchange_rate` (`DEC_20_10`, `:431`), `exchange_rates.rate` et `.acknowledged_rate`
    (`DEC_20_10`, par `fmt_rate`, `:923`, `:926`) — : **38 colonnes `DECIMAL` exportées, 25 à quatre décimales, 10 à deux, 3 à dix**,
    soit exactement les 38 colonnes `DECIMAL` des tables de `TABLES_EXPORTEES` au schéma (la seule autre table à colonnes `DECIMAL`,
    `_kesh_fetched_exchange_rates`, est une table système hors export). Ce décompte est celui que le test de pleine échelle (AC13)
    écrit à la main.
  - **Ce qui change pour qui lit l'export** : la **forme** des 25 colonnes à quatre décimales (`150.25` → `150.2500`) ; aucune valeur
    enregistrée ne change, et une fraction de centime que l'export coupait jusqu'ici y paraît désormais entière (`10.0050`) ; un tableur relit le même nombre ; un script qui compare des chaînes (`== "150.25"`) ou lit à deux décimales fixes est à
    adapter. Les colonnes à deux décimales ne changent pas. Note de version (section *Modifié*) et manuels (D8) le disent.
  - **Option écartée** (le remède du kickoff) : arrondir l'export global au centime par la fonction commune — l'export de
    souveraineté perdrait la fraction de centime que la base garde ; c'est ce que Guy a refusé le 2026-11-20.
- **D6 — la garde** (`crates/kesh-api/tests/arrondi_centime_garde.rs`, nouveau ; patron `date_du_jour_garde.rs` : listes closes à égalité
  stricte par fichier, ce fichier exclu nommément) :
  - **Famille 1 — précision de format** : sur `crates/*/src/**/*.rs`, lignes `//` retirées, compte des occurrences de
    `\{[A-Za-z_0-9.]*:[^{}"]*\.([0-9]+|[A-Za-z_][A-Za-z_0-9]*\$|\*)\}` (précision fixe, nommée `x$` ou passée en argument `.*`, avec ou
    sans largeur ni alignement ; la chaîne de format peut être seule sur sa ligne). Liste close **après** la story, une justification
    écrite par fichier, **22 occurrences dans 8 fichiers** (recompté le 2026-11-20 sur `crates/*/src/**/*.rs`, 334 fichiers, lignes `//`
    retirées : 30 occurrences / 10 fichiers aujourd'hui ; après la story −3 `kesh-report/src/pdf.rs`, −2 `kesh-report/src/csv.rs`, −2
    `csv_tables.rs` (`{d:.2}` et `{d:.10}` disparaissent, `{d:.scale$}` reste), −1 `bank_statements_manual.rs`, −1
    `company_invoice_settings.rs`, +1 `money.rs` = 22) : `kesh-core/src/types/money.rs` 1 (la fonction commune) ;
    `kesh-api/src/exports/csv_tables.rs` 1 (`.scale$` de `fmt_decimal_scale` : précision égale à l'échelle de la colonne, constante
    `DEC_*` — complète, ne coupe pas ; **prouvé par le test de pleine échelle, AC13**, sans quoi cette justification serait vraie par
    construction) ; `kesh-core/src/exchange_rates/parse/tests.rs` 2 (test) ; `kesh-i18n/src/formatting.rs` 1 (arrondi `:17`) ;
    `kesh-payment/src/pain001/mod.rs` 1 (arrondi au pair, **#1441**) ; `kesh-qrbill/src/generator.rs` 1 (arrondi `:114`) ;
    `kesh-qrbill/src/pdf.rs` 13 (`f32`) ; `kesh-report/src/pdf.rs` 2 (`.prec$` de `format_exchange_rate`, `f32` du test `:3337`). Tout
    autre fichier doit compter **0** ; un compte qui diffère, dans un sens ou dans l'autre, rougit et nomme le fichier.
  - **Famille 2 — troncature explicite** : `\.trunc\(\)`, `trunc_with_scale\(`, `RoundingStrategy::(ToZero|RoundDown)` ; liste close :
    `kesh-core/src/bank_imports/manual.rs` 1, `kesh-api/src/routes/mcp_server_settings.rs` 1 (ni l'un ni l'autre n'affiche un montant).
  - **Sonde positive** (la garde n'est pas vraie par construction) : la fonction de comptage, appliquée à un texte écrit dans le test
    — `format!("{d:.2}")`, `format!("{:.2}", d)`, `format!("{d:>10.2}")`, `format!("{d:.scale$}")`, `format!("{:.*}", 2, d)`, `format!("{:.1$}", d, 2)`, une chaîne
    `"{:.2}",` seule sur sa ligne, et une ligne `// {:.2}` —, rend **7** ; la famille 2 sur `d.trunc()` et `RoundingStrategy::ToZero`
    rend 2 ; et la garde **voit au moins un fichier** (le balayage n'est pas vide : `sources().len()` > 100, et `money.rs` y figure). Les
    comptes attendus sont **écrits à la main** dans le test, jamais dérivés du même extracteur.
  - **Angles morts assumés** (écrits dans le doc-comment de la garde) : un `Decimal` écrit par `{}` ou `to_string()` sort à son échelle
    (quatre décimales) sans rien couper — c'est le symptôme de #813, non celui-ci ; une précision construite à l'exécution hors d'un
    littéral (aucun site) ; un découpage maison au point (R3 : aucun site sur un montant) ; le frontend (big.js arrondit, R5) ; l'arrondi
    au pair `round_dp(` (autre mode, non une troncature : #1442, #1443, #1441) ; les **stratégies d'arrondi que la famille 2 ne nomme
    pas** — `RoundingStrategy::MidpointTowardZero` et `RoundHalfDown` (le demi vers zéro : `10.0050` → `10.00`, le symptôme même de
    #1245), `ToNegativeInfinity` (vers le bas), `.floor()`, `.to_i64()` après une multiplication par 100 — : **aucun site** aujourd'hui
    (`grep` sur `crates/*/src`, lignes non commentées, le 2026-11-20 : vide), une forme future n'est pas vue ; la famille 2 n'est pas
    élargie pour ne pas confondre une troncature et un arrondi d'un autre mode, que la relecture juge ; enfin la **liste close compte
    par fichier** : un échange à compte égal dans un même fichier (un `{:.3}` de `f32` retiré de `kesh-qrbill/src/pdf.rs`, un `{:.2}` de
    `Decimal` ajouté au même fichier) laisse la garde verte — la relecture du diff de tout fichier listé le voit, la garde non.
  **Option écartée** : une garde par liste blanche de **fonctions** (« `{:.2}` interdit hors de `format_two_decimals` ») — elle ne voit pas
  une précision nommée ni un `f32`, et doit savoir lire le Rust ; la liste close par fichier est le patron éprouvé du dépôt.
- **D7 — factures réelles** : rien dans `kesh-qrbill`, ni dans `invoice_pdf_service.rs`, ni dans les écrans de facture. Le PDF d'une
  facture ou d'un avoir, archivé ou régénéré, reste identique octet pour octet à ce que la 0.19.0 rend (il arrondit déjà).
- **D8 — les manuels** (`.tex` seulement, PDF non commité ; seul le français existe — `docs/manual/{de,en,it}/` ne portent qu'un
  `README.md`) :
  - `admin-manual.tex:3915`, la phrase « l'export CSV des réglages la \textbf{tronque} à deux décimales (\texttt{5.00}, défaut connu,
    issue~\#1245) ; l'écran et l'API l'\textbf{arrondissent} (\texttt{5.01}) » devient « l'export CSV des réglages l'écrit tel quel, à
    quatre décimales (\texttt{5.0060}), depuis la 0.20.0 ; l'écran et l'API l'\textbf{arrondissent} (\texttt{5.01}) ».
  - `user-manual.tex`, § *Contenu du ZIP* (`\subsection{Contenu du ZIP}`, après la liste des tables) : une phrase — « Chaque nombre
    s'y écrit à l'échelle où Kesh le garde, sans arrondi : quatre décimales pour les montants, prix, quantités et totaux
    (\texttt{150.2500}), deux pour les taux de TVA, les frais de rappel et les montants et soldes des relevés, dix pour les cours de
    change ; point décimal, sans séparateur des milliers. » Aucune phrase du manuel ne décrit aujourd'hui la précision de l'export
    (A8 de la validation P1) : sans elle, la forme nouvelle ne serait écrite nulle part qu'au journal des modifications.
  - `admin-manual.tex`, section `\subsubsection{Passer à la 0.20.0}\label{sec:maj-0-20-0}` — **ouverte par la story** si elle n'existe
    pas à son étape 0 (précédent : la Story 36-9 a ouvert « Passer à la 0.18.0 » pour son seul paragraphe, commentaire LaTeX à l'appui,
    `admin-manual.tex:2574-2577`), placée avant `sec:maj-0-19-0` ; sinon le paragraphe s'ajoute à la section existante —, paragraphe
    `\paragraph{Pour les intégrations.}` : « \textbf{L'export global change de forme} : les montants, prix, quantités et totaux s'y
    écrivent à quatre décimales (\texttt{150.25} devient \texttt{150.2500}), à l'échelle où Kesh les garde ; aucune valeur enregistrée ne change (une fraction
    de centime que l'export coupait jusqu'ici y paraît désormais entière, \texttt{10.0050}), un
    tableur relit le même nombre, mais un script qui compare des chaînes ou lit deux décimales fixes est à adapter. Les taux, les frais
    de rappel et les montants des relevés restent à deux décimales. » L'introduction de la section (sauvegarde, migrations) et les
    paragraphes des autres stories s'écrivent à l'intégration de la release. **La section sera ouverte trois fois** : elle l'est déjà,
    chacune avec son paragraphe « Pour les intégrations », sur `origin/epic/49-types-de-lignes-facture` (49-1a) et sur
    `story/48-12-reponses-perdues-fournisseurs` (`admin-manual.tex:2550`) ; la story écrit sous le titre le même commentaire LaTeX que
    la 48-12 (« section ouverte par la story pour son seul paragraphe ; les Stories 48-12 et 49-1a ouvrent la même section sur leur
    branche : à l'intégration, garder les paragraphes sous un seul titre »), et **le second arrivé fusionne les paragraphes** sous un
    seul `\subsubsection`.
  **Option écartée** : une phrase neuve sur l'arrondi des **rapports** (leur format visible ne change pas — toujours deux décimales — ;
  seul un centime faux devient juste).
- **D9 — pas d'E2E Playwright, des tests de route** : le défaut se voit à la frontière HTTP (fichiers téléchargés), non dans un contrôle
  de l'écran. Les tests `sqlx::test` de route lisent les **octets que le navigateur télécharge** ; un Playwright n'y ajouterait que le
  clic sur un lien de téléchargement, déjà couvert par les specs existantes. **Aucune recette visuelle** : aucun contrôle visible n'est
  ajouté ni modifié (règle du `CLAUDE.md`) ; le contenu d'un PDF n'est pas un contrôle.

## Critères d'acceptation

1. **AC1 — la fonction commune** (`kesh-core/src/types/money.rs`, tests unitaires) : `format_two_decimals` rend `10.0050` → `"10.01"`,
   `5.0060` → `"5.01"`, `10.0049` → `"10.00"`, `-10.0050` → `"-10.01"`, `-0.0040` → `"0.00"`, `Decimal::ZERO` et un zéro négatif → `"0.00"`,
   `1234.5` → `"1234.50"`, `12.345` (échelle 3) → `"12.35"`, `1000000` → `"1000000.00"` ; `round_for_display` rend les valeurs
   correspondantes et `dec!(-0.0040)` → un zéro **sans signe** (`!is_sign_negative()`).
2. **AC2 — CSV des rapports** (`kesh-report/src/csv.rs`, tests) : `format_amount_iso(dec!(10.0050)) == "10.01"`, `(5.0060) == "5.01"`,
   `(-0.0040) == "0.00"` ; **et un rendu** : `render_trial_balance_csv` d'une balance dont une ligne porte `10.0050` au débit écrit
   `10.01`, non `10.00` ; `format_rendement(Some(dec!(12.345)))` (`csv.rs`) == `"12.35%"`.
3. **AC3 — PDF des rapports** (`kesh-report/src/pdf.rs`, tests, avec `pdf_text`) : `format_swiss_amount(dec!(1234.5678)) == "1'234.57"`,
   `(dec!(10.0050)) == "10.01"`, `(dec!(-0.0040)) == "0.00"` ; un rendu `render_trial_balance_pdf` d'une ligne à `10.0050` montre
   `10.01` ; `format_rate(dec!(8.125)) == "8.13 %"` ; `format_rendement(Some(dec!(12.345))) == "12.35%"` ; les tests existants
   (`:2352-2381`, `:2687` « ni 0.00 ni -0.00 », et `:3100-3109` `le_montant_en_devise_s_arrondit_avant_formatage`) restent verts avec
   leurs **assertions inchangées** — seule modification admise : la ligne `use kesh_core::types::round_for_display;` ajoutée au module de
   tests (D3).
4. **AC4 — export global** (`kesh-api/src/exports/csv_tables.rs`, tests unitaires ; `crates/kesh-api/tests/exports_global_e2e.rs`, test
   de route) : `fmt_decimal_scale(dec!(10.0050), DEC_19_4) == "10.0050"`, `fmt_decimal_scale(dec!(150.25), DEC_19_4) == "150.2500"`,
   `fmt_decimal_scale(dec!(8.10), DEC_5_2) == "8.10"` ; par la route `GET` de l'export global, une écriture dont une ligne porte
   `10.0050` au débit (posée par SQL dans la base de test) sort `10.0050` dans `journal_entry_lines.csv`, et un réglage
   `group_settlement_max_difference` posé à `5.0060` par SQL sort `5.0060` dans `company_invoice_settings.csv` (le cas du manuel,
   `admin-manual.tex:3915`) ; `amount_currency` et `exchange_rate` sortent **inchangés** (quatre et dix décimales) ; un taux de TVA et
   un solde de relevé restent à deux décimales. Les tests unitaires qui figent les deux décimales **se réécrivent** :
   `serialize_journal_entry_lines_csv_decimal_two_decimals` (`csv_tables.rs:2455`, attendu `;150.2500;0.0000;;;;;;\r\n`, renommé
   `serialize_journal_entry_lines_csv_decimal_column_scale`), et les attendus de `:2485` (`1125.8300;0.0000`), `:2496`, `:2518`,
   `:2519` (`150.2500;0.0000`). `exports_global_e2e.rs:2045` (`"7777.77"` cherché par `contains`) reste vert sans modification
   (`7777.7700` le contient) : à dire au Dev Agent Record, non à corriger.
5. **AC5 — rapports par la route** (`crates/kesh-api/tests/reports_export_e2e.rs`) : une écriture à `1234.0050` (débit/crédit) dans un
   exercice ; `GET …/trial-balance/export?…&format=csv` porte `1234.01` ; la même en `format=pdf` porte `1'234.01`. **Le texte se lit
   côté `kesh-api`, c'est établi** : `printpdf` 0.7.0 ne compresse qu'hors `debug_assertions`
   (`printpdf-0.7.0/src/pdf_document.rs:822-831`, `optimize` vide sous `debug_assertions`), et `[profile.test]` hérite de
   `[profile.dev]` (`Cargo.toml:46-47`) ; la police des rapports est intégrée (`kesh-report/src/pdf.rs:440`,
   `BuiltinFont::Helvetica`), donc le texte passe par `write_text` : un octet WinAnsi par caractère
   (`pdf_layer.rs:461`, `lopdf::Document::encode_text(Some("WinAnsiEncoding"), &text)`), écrit en `Tj` hexadécimal (`:467`) — non
   par `write_codepoints` (`:370-385`, deux octets par glyphe, polices externes) ; `write_right` (`pdf.rs:519-525`) écrit le montant en
   un seul `Tj`. Le test cherche, dans le corps
   passé en minuscules, l'hexadécimal Latin-1 de `1'234.01` (`31273233342e3031`) et l'absence de celui de `1'234.00`
   (`31273233342e3030`), sans dupliquer `pdf_text` (privé aux tests de `kesh-report`). Pas d'autre voie admise.
6. **AC6 — relevé saisi** (`bank_statements_manual.rs`, test unitaire du module) : `amount_string(dec!(10.0050)) == "10.01"`,
   `amount_string(-Decimal::ZERO) == "0.00"` et `amount_string(-Decimal::new(0, 2)) == "0.00"` (zéros **négatifs construits** : `dec!(-0)` est un zéro positif, le macro efface le signe), `amount_string(dec!(50)) == "50.00"`, et le **chemin réel** : un solde `-0` saisi sur un relevé manuel sort du parseur (`manual.rs:790`, `-value`) en vrai zéro négatif et s'écrit `"0.00"`.
7. **AC7 — le plafond de l'écart** : `group_max_for_display` délègue à `format_two_decimals` ; ses tests existants
   (`company_invoice_settings.rs:599-600`, `5.0050` → `5.01`, `5.0150` → `5.02`) restent verts sans modification.
8. **AC8 — #1430** : `git grep -nw 'round_for_display' -- crates/` (toutes formes, imports entre accolades compris) — sortie copiée
   au Dev Agent Record — ne rend que : **toute ligne** de `kesh-core/src/types/money.rs` (la définition, son doc-comment, ses tests, et
   l'appel dans `format_two_decimals`, D2) ; le réexport de
   `kesh-core/src/types/mod.rs` ; des `use kesh_core::…round_for_display` ; `balance_sheet.rs:126` (lien intra-doc vers
   `kesh_core::types::round_for_display`) et `:142` (le prédicat) ; le test `pdf.rs:3100-3109` et son `use`. Aucun appel dans
   `kesh-report/src/pdf.rs` ni `csv.rs` hors de ce test (les cinq appels de formatage, `pdf.rs:358`, `:899`, `:1116`, `csv.rs:59`,
   `:225`, sont retirés, D3). Et `grep -n 'crate::pdf' crates/kesh-report/src/balance_sheet.rs` est **vide** ; `csv.rs:25` n'importe
   plus de `pdf.rs` que `format_exchange_rate`. Les tests du bilan (`pdf.rs:2670-2690` et ceux de `shows_retained_earnings_line`)
   restent verts.
9. **AC9 — la garde** (D6) : `arrondi_centime_garde` est verte avec la liste close de 22 occurrences dans 8 fichiers (famille 1) et de
   2 (famille 2) ; la sonde positive rend 7 et 2 ; le balayage voit plus de 100 fichiers.
10. **AC10 — factures réelles** (contrainte de Guy du 2026-11-18) : `git diff origin/epic/48-corrections-0-20-0...HEAD --stat --
    crates/kesh-qrbill crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-db 'frontend/src/routes/(app)/invoices'
    frontend/src/lib/components/invoices frontend/src/lib/features/invoices` **vide** (sortie copiée au Dev Agent Record) ; les tests de
    `kesh-qrbill` (dont ceux des PDF de facture et d'avoir) passent sans modification ; aucune facture, aucun avoir, aucune écriture,
    aucun montant stocké ne change (aucune migration, aucun repository touché).
11. **AC11 — manuels** (D8) : `git grep -n 'tronque' -- docs/manual/fr/admin-manual.tex` ne rend plus la phrase de
    `sec:ecart-admis-reglement-groupe` (elle porte `5.0060`) ; `git grep -n '1245' -- docs/manual` vide ; `user-manual.tex` § *Contenu du
    ZIP* porte la phrase de l'échelle ; `admin-manual.tex` porte `sec:maj-0-20-0` avec son paragraphe « Pour les intégrations » ; les
    quatre langues : seul `fr` existe (constaté, sortie de `ls docs/manual/*/` copiée) ; PDF non commités.
12. **AC12 — propagation** (`CLAUDE.md`, § *Propagation post-patch*) : après le correctif,
    `git grep -nE '\{[A-Za-z_0-9.]*:[^{}"]*\.([0-9]+|[A-Za-z_][A-Za-z_0-9]*\$|\*)\}' -- 'crates/*/src/*'` — le motif **de la garde**,
    qui voit `{d:>10.2}`, `.scale$` et `.*` — (sortie copiée ; 32 lignes à `08c47a6cc`, dont 7 de commentaires) : chaque ligne non
    commentée appartient à un fichier de la liste close de D6 ; chaque ligne de commentaire est relue — aucune ne dit plus `{:.2}` ni
    « deux décimales » pour un montant de l'export ou d'un rapport. Sites de commentaire à réécrire, recensés à `08c47a6cc` :
    `csv_tables.rs:17` (« Decimal toujours formaté `{:.2}` » → « chaque `Decimal` à l'échelle de sa colonne, constantes `DEC_*` ;
    les rapports (`kesh-report::csv`) arrondissent, l'export non »), `:107`, `:112`, `:429`, `:878` (doc-comments de formateurs qui
    disparaissent ou changent), `:1778` et `:2459` (tests) ; `kesh-report/src/csv.rs:7`, `:48`, `:55` ; `pdf.rs:317`, `:344-345` (la
    fonction déménage). `errors.rs:1563` et `kesh-qrbill/src/generator.rs:107`, `:111` restent vrais.
13. **AC13 — l'export à pleine échelle, colonne par colonne** (`exports_global_e2e.rs`, test `sqlx::test` neuf,
    `export_global_chaque_colonne_decimal_a_l_echelle_du_schema`) — ce qui empêche D5 et la justification de `csv_tables.rs` dans la
    garde d'être vraies par construction :
    - l'**ensemble des colonnes** se lit dans `information_schema.COLUMNS` (`TABLE_SCHEMA = DATABASE()`, `DATA_TYPE = 'decimal'`,
      `TABLE_NAME` dans `TABLES_EXPORTEES`), avec `NUMERIC_SCALE` : source **différente** du code testé ; le test assert qu'il en
      trouve **38** (25 à l'échelle 4, 10 à 2, 3 à 10), nombres **écrits à la main** — une colonne `DECIMAL` ajoutée par une migration
      future rougit le test, qui la nomme ;
    - **le principe vérifié** : pour chaque colonne, *la cellule du CSV vaut la valeur posée en base, écrite à `NUMERIC_SCALE`* (même
      échelle, mêmes chiffres, zéros de queue compris). La comparaison se fait par **`(table, colonne, clé de la ligne)`** — la clé
      est `id`, sauf `company_invoice_settings`, dont la clé est `company_id` —, la ligne étant **choisie par colonne** : une table peut
      porter plusieurs lignes nommées quand ses contraintes interdisent de poser toutes ses colonnes sur une seule ;
    - **valeur par défaut** : `1.` suivi des `s` premiers chiffres de `1234567891` (`1.12`, `1.1234`, `1.1234567891`), posée par
      `UPDATE … WHERE <clé> = ?` sur une ligne de la société. Une valeur à zéros de queue (`1.2600`, `0.0000`) distingue aussi bien
      `{:.2}` de `{:.4}` et s'admet là où une contrainte l'impose. **Toute dérogation à la valeur par défaut nomme sa contrainte**
      dans le test ;
    - **les dérogations, ensemble clos** (les contraintes `CHECK` des 19 tables relues au schéma à `08c47a6cc` et dans la migration
      `20261118000002_invoice_line_types.sql` de l'Epic 49 ; aucun déclencheur au schéma ; toutes les autres contraintes sur une
      colonne `DECIMAL` sont des bornes — `> 0`, `>= 0`, `<= 100`, `<= 10000`, `<= 1000000000` — que les valeurs par défaut
      respectent) :
      - **L1, L2, L3 : trois lignes distinctes, sans devise au départ ; L3 se pose par un seul `UPDATE` des cinq colonnes**
        (`amount_currency`, `exchange_rate`, `rate_source`, `debit`, `credit` : `chk_jel_fx_all_or_none` et `chk_jel_fx_chf` se
        vérifient ligne par ligne, trois `UPDATE` successifs échoueraient au premier ; L1 et L2 restent sans devise, sinon `fx_chf`
        refuse `1.1234`) ;
      - `journal_entry_lines` — `chk_jel_debit_credit_exclusive` (`debit = 0 and credit > 0 or debit > 0 and credit = 0`),
        `chk_jel_fx_all_or_none`, `chk_jel_fx_chf` (`debit + credit = round(amount_currency * exchange_rate, 2)` sur une ligne en
        devise hors réévaluation) : **trois lignes nommées**, prises parmi les lignes des écritures de `seed_with_full_data` —
        **L1** débit seul (`debit = 1.1234`, `credit = 0`, sans devise : cellules `1.1234` et `0.0000`) ; **L2** crédit seul
        (`debit = 0`, `credit = 1.1234`, sans devise) ; **L3** en devise (`amount_currency = 1.1234`, `exchange_rate = 1.1234567891`,
        `rate_source = 'manual'`, `debit = 1.26` = `round(1.1234 × 1.1234567891, 2)`, `credit = 0` : cellules `1.1234`,
        `1.1234567891`, `1.2600`, `0.0000`). `debit` et `credit` se vérifient sur L1 et L2, `amount_currency` et `exchange_rate` sur
        L3 (et `1.2600` sur L3 en filet) ;
      - `exchange_rates` — `chk_exchange_rates_ack_pair` et `chk_exchange_rates_ack_afc` : la ligne porte `source = 'afc'` et un
        `acknowledged_at` ; `rate = 1.1234567891`, `acknowledged_rate = 0.9876543210` (deux valeurs distinctes, pour qu'une colonne
        lue à la place de l'autre rougisse) ; `chk_exchange_rates_currency` (pas `CHF`) et `chk_exchange_rates_month_day` (`kind =
        'day'`, ou le 1er du mois pour `month`) ;
      - `invoice_lines`, `credit_note_lines` — `chk_*_unpriced_neutral` de l'Epic 49 (49-1a : une ligne `comment` ou `heading` a
        `quantity`, `unit_price`, `vat_rate` et `line_total` à 0) : l'`UPDATE` vise **par son `id`** la ligne `priced` de la fixture,
        jamais « les lignes de la facture » ; la contrainte n'existe pas encore sur `epic/48`, elle existera après la fusion de
        l'Epic 49 ;
    - **les fixtures** : le test neuf **monte son propre jeu**, sans toucher aux fixtures des tests existants (leurs attendus de
      comptes de lignes, `:982-996`, en dépendent) : une société seule, `seed_with_full_data` pour les écritures, puis une fonction
      d'aide qui insère les tables restantes pour cette société. Les `INSERT` existants ne sont **pas** dans `seed_with_full_data`
      (qui n'insère que `contacts`, `products`, `bank_accounts` et trois écritures) mais dans le corps de tests distincts
      (`invoices`/`invoice_lines`, `bank_imports`/`bank_transactions`, `vat_rates`, et neuf tables dans
      `export_global_zip_onze_tables_neuves_sortent_et_sont_scopees`, `:1825`) : ils se réécrivent dans la fonction d'aide. 14 des 19
      tables ont un `INSERT` quelque part dans le fichier, `company_invoice_settings` vient du préréglage et `journal_entry_lines` de
      `post_entry` (16) ; **trois sont à écrire**, aucune n'existant dans `exports_global_e2e.rs` (`:982`
      `("invoice_reminders.csv", 0)`, `:983-984` `dunning_levels` = 0) — un **niveau de relance** (`dunning_levels` : `level_number = 1`,
      `delay_days`, `fee_amount = 1.12`) ; un **rappel** (`invoice_reminders` : sur une facture de la fixture, `level_number = 1`,
      `channel = 'manual'`, `sent_at`, `subject`, `body`, `fee_amount = 1.12`) ; un **cours retenu** (`exchange_rates`, ci-dessus). La
      fonction d'aide est neuve ; l'extraction des `INSERT` existants n'est pas admise si elle ajoute des lignes à un fixture partagé ;
    - dans `<table>.csv`, l'en-tête porte le nom de la colonne ; un échec nomme `table.colonne`, la clé de la ligne, l'attendu et le lu.

## Tests à écrire, et la preuve du rouge

Ordre : **le test d'abord, rouge**, puis le correctif, vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4811/`. Mutations par l'outil
**Edit** (jamais `git stash`) ; après restauration, `touch` du fichier (mémoire « mutation restaurée, binaire périmé »).

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC2 (`format_amount_iso`, rendu de la balance CSV, rendement) | le correctif de `csv.rs` | **rouge d'abord** : écrits avant le correctif, ils échouent sur `10.00` / `12.34%` ; `rouge-report.log` |
| AC3 (`format_swiss_amount`, rendu PDF, `format_rate`, rendement) | le correctif de `pdf.rs` | rouge d'abord (`"10.00"`, `"-0.00"`, `"8.12 %"`) ; `rouge-report.log` |
| AC4 (`fmt_decimal_scale`, route de l'export global) | le correctif de `csv_tables.rs` | rouge d'abord : la route rend `10.00` et `5.00` là où `10.0050` et `5.0060` sont attendus ; les tests unitaires réécrits (`150.2500`) rougissent contre `{d:.2}` ; `rouge-api.log` |
| AC5 (route des rapports) | le correctif de `csv.rs` / `pdf.rs` | rouge d'abord (`1234.00`, hexadécimal de `1'234.00`) ; `rouge-api.log` |
| AC13 (pleine échelle) | le correctif de `csv_tables.rs` | rouge d'abord : les 24 colonnes à quatre décimales de `fmt_decimal` sortent `1.12` au lieu de `1.1234` (les 14 autres sont déjà vertes : filets) ; `rouge-api.log` ; **M7** : une seule constante fausse (`DEC_18_2` au lieu de `DEC_19_4` pour `invoice_lines.line_total`) → rouge par le `debug_assert!` de `fmt_decimal_scale` (« échelle 4 de 1.1234 supérieure à celle de sa colonne (2) », `csv_tables.rs:130`) : la requête de l'export échoue avant de livrer un CSV et le test rougit sur la réponse, non sur un message nommant la colonne (`mut-m7.log`, rejouée au HEAD de clôture) ; **M8** : `38` écrit à la main remplacé par `37` → rouge sur le décompte (`mut-m8.log`) |
| AC6 (`amount_string(10.0050)`) | le correctif de `bank_statements_manual.rs` | rouge d'abord ; `rouge-api.log` ; les cas `-0` et `50` **ne rougissent pas** : filets |
| AC1 (fonction commune, neuve) | — (elle n'existe pas avant) | **M1** : stratégie remplacée par `ToZero` dans `round_for_display` → AC1, AC2, AC3, AC4 rouges (`mut-m1.log`) ; **M2** : retrait du `if r.is_zero()` → les cas `-0.0040` rouges (`mut-m2.log`) ; **M3** : `MidpointNearestEven` → `10.0050` rouge (`mut-m3.log`) |
| AC7 (plafond de l'écart) | **ne rougit pas** : sortie inchangée | filet déclaré (tests existants) |
| AC8 (#1430) | — (déplacement) | le `git grep` de l'AC8, sortie copiée |
| AC9 (garde) | un `{:.2}` réintroduit | **M4** : `let _ = format!("{:.2}", amount);` ajouté dans `kesh-report/src/csv.rs` → garde rouge, message qui nomme le fichier et dit 1 au lieu de 0 (`mut-m4.log`) ; **M5** : retrait d'un `{:.2}` légitime d'un fichier listé (par exemple la ligne de `money.rs` réécrite en `format!("{}", …)` le temps de la mutation) → rouge « 0 au lieu de 1 » (`mut-m5.log`) ; **M6** : `amount.trunc_with_scale(2)` ajouté dans `pdf.rs` → famille 2 rouge (`mut-m6.log`) ; la sonde positive rend 7 et 2 dans le même binaire |
| AC10, AC11, AC12 | — | commandes et sorties copiées |

Numérotation : **M1 à M8, uniques, un journal chacune.**

## Tâches

1. **Étape 0** : relire à la tête de la branche les lignes citées (`pdf.rs:316-318`, `:349-352`, `:1394-1395`, `:1666-1668` ;
   `csv.rs:50-59`, `:745-747` ; `csv_tables.rs:17`, `:107-121`, `:880-881` ; `bank_statements_manual.rs:551-555` ;
   `company_invoice_settings.rs:45-49` ; `balance_sheet.rs:69`, `:142`) ; en copier trois au Change Log ; relancer la simulation de la
   garde (`garde-simulation-avant.txt`) et noter tout écart (une fusion de `main` peut avoir ajouté un site : il entre au tableau, corrigé
   ou assumé, avant d'écrire la liste close).
   Relire aussi le schéma (`q1-echelles-colonnes.txt` régénéré : 38 colonnes, 25/10/3) — une colonne `DECIMAL` ajoutée par une fusion
   entre au tableau de D5 avant tout code.
2. Tests rouges (AC2 à AC6, AC13), journaux `rouge-*.log`.
3. `kesh-core` : `round_for_display`, `format_two_decimals`, réexport ; tests AC1 ; mutations M1 à M3.
4. `kesh-report` : `pdf.rs` (retrait de `round_for_display` et de ses appels de formatage `:358`, `:899`, `:1116` ; `format_swiss_amount`,
   `format_currency_amount`, `format_rate`, `format_rendement` ; `use` du module de tests), `csv.rs` (`format_amount_iso`,
   `format_currency_amount_iso`, `format_rendement`, appel `:225`, import `:25`, en-tête `:7`, doc `:48`, `:55`), `balance_sheet.rs`
   (import, lien `:126`, #1430) ; tests AC2, AC3, AC8.
5. `kesh-api` : `csv_tables.rs` (D5 : `fmt_decimal_scale`, constantes `DEC_*`, les 34 sites et `:430-431`, `fmt_rate` ; commentaires
   de l'AC12 ; tests unitaires réécrits), `bank_statements_manual.rs` (`amount_string`), `company_invoice_settings.rs`
   (`group_max_for_display`) ; tests AC4 à AC7, AC13 ; mutations M7, M8.
6. Garde `crates/kesh-api/tests/arrondi_centime_garde.rs` (D6) ; mutations M4 à M6.
7. Manuels (D8 : trois sites), `.tex` seulement ; propagation (AC12) ; contrôle AC10 ; Dev Agent Record. **Ne pas écrire
   `CHANGELOG.md` ni `sprint-status.yaml`.**

## Gates

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ;
  `cargo nextest run -E 'package(kesh-core) & kind(lib) | package(kesh-report) | binary(exports_global_e2e) | binary(reports_export_e2e) |
  binary(arrondi_centime_garde) | (package(kesh-api) & kind(lib)) | package(kesh-qrbill)'` (`kesh-qrbill` pour l'AC10), sur la base
  `kesh_e4811`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh` (base `kesh_ci_e4811`). Frontend : aucun fichier touché ;
  `npm run check` et `test:unit` seulement si un fichier du frontend change (il ne le doit pas).
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*,
  `KESH_COOKIE_SECURE=false`, base `kesh_e2e_e4811` ; « vert » = les seuls échecs attendus, jugés fichier par fichier).
- `kesh-db` non touché (pas de migration, pas de repository, pas de `post_restore.rs`).
- Une commande lourde à la fois, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`.

## Note de version

*En tête de la note* : **l'export global change de forme** — tout outil qui le lit (tableur, import maison, script) y trouve
désormais les montants, prix, quantités et totaux à **quatre décimales** (`150.2500`).

**Modifié**

- Export global : chaque nombre s'écrit à l'échelle où Kesh le garde, **sans arrondi ni coupe** — quatre décimales pour les montants,
  prix, quantités et totaux, deux pour les taux de TVA, les frais de rappel et les montants et soldes des relevés, dix pour les cours
  de change. Les colonnes à quatre décimales sortaient jusqu'ici à deux décimales **tronquées** (10.0050 sortait 10.00 ; le plafond de
  l'écart d'un règlement groupé posé à 5.0060 sortait 5.00). Aucune valeur enregistrée ne change ; pour les valeurs sans fraction de
  centime seule la forme change (150.25 devient 150.2500) ; une fraction de centime que l'export coupait jusqu'ici y paraît désormais
  entière. Un tableur relit le même nombre, un script qui compare des chaînes ou lit deux décimales fixes est à adapter (refs #1245).

**Corrigé**

- Rapports (PDF et CSV) : les montants à deux décimales sont **arrondis au centime**, le demi-centime s'éloignant de zéro, comme à
  l'écran ; ils étaient jusqu'ici **tronqués** — un montant de 10.0050 sortait 10.00 au lieu de 10.01, −10.0050 sortait −10.00 au lieu
  de −10.01, et un résidu négatif sous le demi-centime s'imprimait « -0.00 ». Seuls les montants qui portent une fraction de centime
  changent ; aucune donnée enregistrée n'est modifiée, et le PDF des factures et des avoirs, qui arrondissait déjà, ne change pas
  (refs #1245).
- Interne : la règle d'arrondi d'affichage est commune au bilan et aux rapports PDF et CSV (refs #1430).

## Dev Notes

- **Calendrier (V3-4)** : si le temps manque, l'AC13 est le seul morceau déplaçable (il ne touche pas la production) ; le déplacer
  est une décision du Project Lead, à lui dire, non à prendre.
- **Modules** (règle de décompte de la 48-15 : une **crate** backend, un **domaine** frontend, ou **les manuels** ; les tests suivent leur
  module) : (1) `kesh-core` (`types/money.rs`, `types/mod.rs`), (2) `kesh-report` (`pdf.rs`, `csv.rs`, `balance_sheet.rs`), (3) `kesh-api`
  (`exports/csv_tables.rs`, `routes/bank_statements_manual.rs`, `routes/company_invoice_settings.rs`, tests `exports_global_e2e.rs`,
  `reports_export_e2e.rs`, `arrondi_centime_garde.rs`), (4) manuels (`admin-manual.tex`, deux sites ; `user-manual.tex`, une phrase).
  **Quatre modules : sous la borne de cinq, pas de découpage**, y compris sous la décision Q1 (elle change le geste dans
  `csv_tables.rs`, non le nombre de modules). Au grain du fichier de production : neuf fichiers ; deux gestes — déléguer à la fonction
  commune (rapports, relevé, plafond), écrire à l'échelle de la colonne (export global).
- **Charge révisée** (après Q1) : la décision ajoute la réécriture des 36 appels de `csv_tables.rs` (34 + `:430-431`) avec leur
  constante, cinq attendus de tests unitaires, le test de pleine échelle AC13 (19 tables à colonnes `DECIMAL`, dont **trois fixtures
  à écrire** — niveau de relance, rappel, cours retenu — et trois lignes d'écriture nommées pour les contraintes de
  `journal_entry_lines` : la part la plus lourde) et deux phrases de manuel de plus ; elle retire la délégation de `fmt_decimal`.
  Estimation : **une journée et demie** de développement, boucle de revue non comprise (la P2 a montré l'AC13 plus lourd qu'écrit) ;
  la story reste tenable avant la coupure du 2026-11-26.
- **Dépendances** : part de `epic/48` à `08c47a6cc`, après la fusion de #1327 (décision de Guy du 2026-11-18) et de #1346. Ne bloque ni
  n'est bloquée par l'Epic 49, mais **partage des fichiers avec lui** :
  - `crates/kesh-api/src/exports/csv_tables.rs` et `crates/kesh-api/tests/exports_global_e2e.rs` — la 49-1a (`ac5471e07`, sur
    `origin/epic/49-types-de-lignes-facture`) ajoute `il.line_type` deux lignes sous `fmt_decimal(il.line_total)` (`:659`, site de D5)
    et sous `:1300`, et le test `export_global_zip_line_type_is_the_last_column`. Conflit de texte improbable, **interaction réelle** :
    le second arrivé garde les deux intentions (colonne `line_type` en fin de ligne, échelle `DEC_19_4` des colonnes voisines) ;
  - la migration de la 49-1a (`20261118000002_invoice_line_types.sql`) ajoute `chk_invoice_lines_unpriced_neutral` et
    `chk_credit_note_lines_unpriced_neutral` : l'AC13 vise les lignes `priced` par leur `id` ; elle n'ajoute aucune colonne `DECIMAL`
    (le décompte 38 tient après fusion) ;
  - `admin-manual.tex`, `sec:maj-0-20-0` (D8) ;
  - et la garde **fige des comptes dans deux fichiers de l'Epic 49** : 13 occurrences dans `crates/kesh-qrbill/src/pdf.rs` (des `f32` de
  géométrie, `:84` et `:4299` à `:5400`) et 1 dans `crates/kesh-qrbill/src/generator.rs`. **`kesh-qrbill/src/pdf.rs` est donc un fichier
  commun avec l'Epic 49** au sens de la garde : tout `{:.N}` que l'Epic 49 y ajoute — même sur un `f32`, même légitime — rougit
  `arrondi_centime_garde` chez le second arrivé, qui met à jour le compte attendu et sa justification (ou corrige le site s'il formate un
  `Decimal` de montant). C'est voulu ; **le dire à l'Epic 49** dès la fusion de la story, comme à l'Epic 46.
- **Fichiers communs** : `crates/kesh-report/src/pdf.rs` — PR #1327 et #1346 de Beat, **fusionnées** ; l'Epic 46 de Beat (rapports par
  groupes de comptes) touchera `kesh-report` : si sa branche est fusionnée dans `main` avant l'intégration, le second arrivé résout, et
  tout nouveau `{:.2}` de l'Epic 46 rougira la garde — c'est voulu ; **le dire à Beat** (canal technique) dès la fusion de la story.
  `admin-manual.tex` : toutes les stories ; PDF jamais commité.
- **Ce qui doit rester vrai** : deux décimales exactement dans chaque montant de **rapport** (Excel relit une colonne homogène,
  `csv.rs:48-49`) ; dans l'**export global**, qui fait exception depuis Q1, chaque colonne à **une seule** échelle, celle du schéma —
  la colonne reste homogène, à quatre, deux ou dix décimales ; point décimal sans séparateur au CSV, apostrophe au PDF ;
  `amount_currency` à quatre et `exchange_rate` à dix décimales dans l'export global ; un cours de change à quatre à dix décimales
  (`format_exchange_rate`) ; le PDF des factures inchangé.
- **Limite connue, inchangée** : chaque ligne et chaque total s'arrondissent séparément ; la somme des lignes affichées d'un rapport peut
  différer d'un centime du total affiché quand plusieurs lignes portent des fractions de centime (c'était déjà vrai, et pire, avec la
  troncature). Le total reste juste ; c'est la présentation de valeurs exactes.
- **Production** : aucun montant stocké ne change. Un **rapport** régénéré après la mise à jour peut différer d'un centime de celui
  que la 0.19.0 avait produit, sur les seules valeurs à fraction de centime. L'**export global** régénéré change de **forme**, non de
  valeur : ses colonnes à quatre décimales s'écrivent `150.2500` au lieu de `150.25`, et une valeur à fraction de centime y paraît
  entière (`10.0050`) au lieu d'être coupée (`10.00`). Rien à faire sur le NAS ; un export de la 0.19.0 déjà remis à un tiers reste
  juste au centime tronqué près.
- **Issues liées** (décisions de Guy du 2026-11-20, Q2 — ouvertes, hors story) :
  - **#1442** (I1) — `kesh-report/src/project_report.rs:437` : le rendement d'un projet s'arrondit **au pair** (`round_dp(2)`,
    12.345 → 12.34), non au demi loin de zéro comme le reste de l'affichage.
  - **#1443** (I2) — `kesh-db/src/repositories/invoices.rs:460` : le total de ligne d'une facture se calcule **au pair** à quatre
    décimales (`round_dp(4)`), alors que l'écran l'estime au demi supérieur (`invoice-helpers.ts:31`) ; divergence possible au
    cinquième chiffre entre l'estimation de l'écran et la valeur enregistrée. **Valeur stockée d'une facture : à traiter sans toucher
    aux factures émises.**
  - **#1441** — `kesh-payment/src/pain001/mod.rs:125` (`round_dp(2)`) : les montants du pain.001 s'arrondissent au pair.
  - L'ancienne I3 (« l'export global sort à deux décimales ») **tombe** : D5 la résout.
- **#1245 et le corps de la PR d'intégration** : le remède écrit à #1245 (« une fonction commune, puis formater », « un test par
  site ») ne s'applique plus à l'export global, qui n'arrondit pas (Q1) ; le corps de la PR le dit. La fermeture de #1245 reste juste :
  rapports PDF et CSV arrondis, export global sans troncature.
- **#1430** : rattachée au milestone **v0.20.0** par décision de Guy du 2026-11-20 (elle était à « Plus tard ») ; la story la ferme, par
  le mot-clé de fermeture du corps de la PR d'intégration seulement, jamais dans un commit.
- Références : issue #1245 (et son commentaire du 2026-11-19 renvoyant à #813), #1430, #813, #822, #1441, #1442, #1443 ; fichier d'épique
  `epic-48-corrections-0-20-0.md` § *48-11* ; plan `release-0.20.0.md` ; `48-15-troncatures-silencieuses.md` (forme, décompte des
  modules) ; garde modèle `crates/kesh-api/tests/date_du_jour_garde.rs` ; journaux d'inventaire `kesh-gate-logs/e4811/inv-*.txt`,
  `garde-simulation-*.txt`.

## Dev Agent Record

### Agent Model Used

Sonnet 5.5 (`bmad-dev-story`, reprise du 2026-11-23 après le crash du 2026-11-20 : le premier agent n'a laissé que des tests rouges et
les manuels FR, commités par l'orchestrateur en `15ae0368a`, jamais exécutés).

### Debug Log References

Journaux hors dépôt sous `/home/gcorbaz/devel/kesh-gate-logs/e4811/` :

- **Rouges** (`dev/rouge-report.log`, `dev/rouge-api.log`, sur `15ae0368a`, `kesh_e4811`) : `kesh-report` 234 tests, **6 rouges**
  (`csv::tests::{format_amount_iso_arrondit_au_centime, format_rendement_arrondit, la_balance_csv_arrondit_la_fraction_de_centime}`,
  `pdf::tests::{format_rate_et_rendement_arrondissent, format_swiss_amount_arrondit_au_centime, la_balance_pdf_arrondit_la_fraction_de_centime}`) ;
  `kesh-api` 49 tests, **5 rouges** (`amount_string_arrondit_au_centime`, `export_global_ne_coupe_plus_la_fraction_de_centime` : lu `10.00`
  pour `10.0050`, `export_global_chaque_colonne_decimal_a_l_echelle_du_schema` : `group_settlement_max_difference` lu `1.12` pour `1.1234`,
  `export_trial_balance_{csv,pdf}_arrondit_la_fraction_de_centime`). Les tests rouges se compilaient sans le correctif : la fonction commune
  n'était pas appelée, aucun symbole ne manquait.
- **Vert ciblé** (`dev/vert-1.log`, commit `9a6c80f81`) : `kesh-core` lib, `kesh-report`, `exports_global_e2e`, `reports_export_e2e`,
  `arrondi_centime_garde`, `kesh-api` lib, `kesh-qrbill` : **1563 passés, 0 échec, 8 ignorés**.
- **Mutations** (`mut/muter.py`, un journal `mut/mut-m<n>.log`, code commité avant, restauration par `git restore --worktree` puis `touch`) :
  voir le tableau ci-dessous.
- **Gate au passage en `review`**, commit de code `e704d397a` (`dev/gate-commit.txt`) : bases remises à neuf (`dev/gate-bases.log`) ;
  `scripts/test-fast.sh --ci` (fmt, clippy `-D warnings`, nextest) **vert : 6222 passés, 14 ignorés, 0 échec**
  (`dev/gate-test-fast-ci.log`) ; `scripts/gate-ci.sh kesh_ci_e4811` **code 0** (`dev/gate-ci.log`, `dev/gate-ci.console` ; somme des lignes
  `test result:` : 259 binaires, 6241 passés, 0 échec, 18 ignorés) ; `dev/gate-exit.txt` porte les deux codes. **Frontend : aucun fichier
  touché** (`git diff --stat 15ae0368a~1 HEAD -- frontend` vide), donc ni `check` ni `test:unit` ; **E2E Playwright non lancé** (D9, à la clôture).
- **AC8** : `dev/ac8.txt` (`git grep -nw round_for_display -- crates/`, régénéré au HEAD de clôture, 18 lignes) ne rend que `money.rs` (définition, doc, tests, appel de
  `format_two_decimals`), le réexport de `types/mod.rs`, `balance_sheet.rs:70` (`use kesh_core::types::round_for_display`), `:126` (lien),
  `:142` (prédicat), le test de `pdf.rs:3090-3095` et son `use` (`pdf.rs:2153`), et un commentaire de la garde ;
  `git grep -n 'crate::pdf' crates/kesh-report/src/balance_sheet.rs` est **vide** ; `csv.rs` n'importe plus de `pdf.rs` que `format_exchange_rate`.
- **AC10** : `dev/ac10.txt` (`git diff origin/epic/48-corrections-0-20-0...HEAD --stat -- crates/kesh-qrbill … crates/kesh-db …`) **vide** ;
  les tests de `kesh-qrbill` sont dans le vert ciblé et dans `test-fast --ci`, sans modification.
- **AC12** : `dev/ac12.txt` (`git grep -nE '<motif PRECISION de la garde>' -- 'crates/*/src/*'`, régénéré au HEAD de clôture, 22 lignes) — chaque ligne non commentée appartient à un fichier de la liste close (22 occurrences, 8 fichiers : `money.rs` 1,
  `csv_tables.rs:135` `{d:.scale$}`, `parse/tests.rs` 2, `kesh-i18n` 1, `pain001` 1, `generator.rs:115`, `kesh-qrbill/src/pdf.rs` 13,
  `kesh-report/src/pdf.rs:361` et `:3325`) ; les lignes de commentaire restantes (`errors.rs:1563`, `money.rs:110`, `:113`, `generator.rs:107`,
  `:111`) sont vraies. `git grep -nE '1245|tronqu' -- docs/manual` rend **dix lignes** (douze occurrences : `admin-manual.tex` 725, 2697, 2707, 3889 ; `user-manual.tex` 2875, 3074, 3385, 3629, 4064, 4065), **toutes sans rapport** avec l'arrondi (troncature d'un mot de passe, d'un nom CAMT, d'un écran de 500 lignes) et aucune ne cite #1245 (recompté à la remédiation de la revue P1, A-1).
- **AC11** : `docs/manual/fr/admin-manual.tex` porte `sec:maj-0-20-0` (`:2550`) ; `ls docs/manual/*/` : seul `fr` a des `.tex` ; PDF non commités.
- **Mots-clés de fermeture** : `git log origin/epic/48-corrections-0-20-0..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide.

### Completion Notes List

- **Étape 0** (lecture à la tête de la branche) : les sites à corriger se sont trouvés tels que la fiche les décrit — `pdf.rs:318`
  `let raw = format!("{amount:.2}");`, `csv.rs:51` `format!("{amount:.2}")`, `csv_tables.rs:109` `format!("{d:.2}")` ; le nombre de colonnes
  `DECIMAL` (38 : 25/10/3) et la garde (22 occurrences dans 8 fichiers) ne sont **pas** recomptés à part : le test de pleine échelle
  (`information_schema`) et `arrondi_centime_garde` les établissent, verts avec ces nombres écrits à la main. Non refait : la simulation
  `garde-simulation-avant.txt` (remplacée par la garde exécutable). Les LOW du dernier rapport de validation étaient déjà traités dans la fiche.
- **Code** : `kesh-core/src/types/money.rs` (`round_for_display` déplacée, `format_two_decimals` neuve, réexportées par `types/mod.rs`) ;
  `kesh-report` (`pdf.rs`, `csv.rs`, `balance_sheet.rs`) ; `kesh-api` (`csv_tables.rs` : `fmt_decimal` et `fmt_opt_decimal` supprimées,
  `fmt_decimal_scale` et cinq constantes `DEC_*`, 34 sites + `:430-431` + `fmt_rate`, commentaires de l'AC12, tests unitaires réécrits ;
  `bank_statements_manual.rs::amount_string` et `company_invoice_settings.rs::group_max_for_display` délèguent) ; garde neuve
  `crates/kesh-api/tests/arrondi_centime_garde.rs` (famille 1 : 22/8 ; famille 2 : 2 ; sondes positives 7 et 2 ; balayage > 100 fichiers
  dont `money.rs`). Manuels FR : les trois sites de D8, posés par le premier agent et relus ici contre le code.
- **Mutations** (tous journaux sous `mut/`) :

  | N° | Mutation | Résultat |
  |---|---|---|
  | M1 | `ToZero` dans `round_for_display` (`mut-m1.log`) | **14 rouges** : `kesh-core` 2 (AC1), `kesh-report` 10 (4 `csv`, 5 `pdf`, 1 `balance_sheet`), route des rapports 2 (AC5 CSV et PDF). **`exports_global_e2e` ne rougit pas** : l'export global n'arrondit pas (D5) — la fiche le prévoyait à tort |
  | M2 | retrait de `if r.is_zero()` | **a survécu d'abord** (`mut-m2-survivant.log` : 594 passés) : `round_dp_with_strategy` rend déjà un zéro positif pour `-0.0040` (aucun test de la fiche ne distingue). Tué après ajout de zéros négatifs **construits** (`-Decimal::new(0, 2)`, `-Decimal::ZERO`) en `e704d397a` : 2 rouges (`mut-m2.log`) ; la garde est **réelle pour un seul chemin** : le solde `-0` saisi sur un relevé manuel (`manual.rs:790`, `-value`, puis `bank_statements_manual.rs:354` et `:358`) ; elle est sans objet pour une valeur lue en base (le décodage passe par `from_parts`, qui efface le signe d'un zéro) et pour les rapports. Les deux cas `dec!(-0)` de `e704d397a` étaient des zéros **positifs** (revue P1, E-1) ; remplacés par `-Decimal::ZERO` et par le solde `-0` d'un relevé saisi, **M2b** rougit (`mut-m2b.log`, voir le Change Log) |
  | M3 | `MidpointNearestEven` | 2 rouges (`mut-m3.log`) |
  | M4 | `format!("{:.2}", amount)` ajouté dans `csv.rs` | garde famille 1 rouge : `kesh-report/src/csv.rs : 1 au lieu de 0` (`mut-m4.log`) |
  | M5 | `{:.2}` retiré de `money.rs` | garde famille 1 rouge : `money.rs : 0 au lieu de 1` (`mut-m5.log`) |
  | M6 | `trunc_with_scale(2)` ajouté dans `pdf.rs` | garde famille 2 rouge : `kesh-report/src/pdf.rs : 1 au lieu de 0` (`mut-m6.log`) |
  | M7 | `DEC_18_2` pour `invoice_lines.line_total` | AC13 rouge par le `debug_assert!` de `fmt_decimal_scale` (« échelle 4 de 1.1234 supérieure à celle de sa colonne (2) », `csv_tables.rs:130`) : la requête de l'export échoue, le test rougit sur la réponse ; le message « attendu 1.1234, lu 1.12 » ne s'exerce plus tant que l'assertion est en place (`mut-m7.log`, rejouée au HEAD de clôture) |
  | M8 | `38` écrit à la main remplacé par `37` | AC13 rouge sur le décompte, liste des 38 colonnes imprimée (`mut-m8.log`) |

- **Écarts à la fiche** : (1) M1 ne rougit pas la route de l'export global (voir ci-dessus) ; (2) M2 a exigé deux cas de test de plus ;
  (3) `exports_global_e2e.rs:2045` (`"7777.77"`, `contains`) est resté vert sans modification, comme annoncé à l'AC4 ; (4) la garde ne balaie
  que `crates/*/src` (comme dit à D6), pas `tests/` ; (5) les tests unitaires réécrits de `csv_tables.rs` (`150.2500`, `fmt_decimal_scale_ecrit_a_l_echelle_de_la_colonne`) n'ont **pas** été rougis avant le correctif (`rouge-api.log` ne porte aucun test de `exports::csv_tables::tests`) : la route de l'export (AC4) et l'AC13 ont rougi, M7 tue une constante fausse (revue P1, A-3).
- **Aucune migration, aucun repository, aucun fichier du frontend ni de `kesh-qrbill`.** Le PDF des factures et des avoirs ne change pas
  (AC10).
- **Fichiers communs** (à dire) : `kesh-report/src/pdf.rs` (Beat, Epic 46) et `kesh-qrbill` : la garde fige 13 occurrences dans
  `kesh-qrbill/src/pdf.rs` et 1 dans `generator.rs` — à dire à l'Epic 49 et à Beat dès la fusion.
- **Tension de calendrier** : la voie base et Rust a été saturée le 2026-11-23 (un E2E complet de plus d'une heure, la 48-12 derrière) ;
  `alerte.md` l'écrit. Aucune conséquence pour la story elle-même.

### File List

`crates/kesh-core/src/types/{money.rs,mod.rs}` ; `crates/kesh-report/src/{pdf.rs,csv.rs,balance_sheet.rs}` ;
`crates/kesh-api/src/exports/csv_tables.rs` ; `crates/kesh-api/src/routes/{bank_statements_manual.rs,company_invoice_settings.rs}` ;
`crates/kesh-api/tests/{arrondi_centime_garde.rs (neuf),exports_global_e2e.rs,reports_export_e2e.rs}` ;
`docs/manual/fr/{admin-manual.tex,user-manual.tex}` ; cette fiche et `48-11-arrondi-au-centime-review-prompt-p1.md`.

## Change Log

- **2026-11-24 — revue de code close, gates et E2E de clôture verts, `Status: done`** (Sonnet 5.5). **Boucle de revue de code** : P1 Opus
  (trois lentilles B, E, A) **0 CRITICAL / 0 HIGH / 1 MEDIUM / 10 LOW**, remédiée à `2797e0447` ; P2 Sonnet (trois lentilles, contexte frais,
  modèle différent) **0 / 0 / 0 / 4 LOW** (`review-p2.md`). Tendance : 1 MEDIUM → 0 ; aucune passe au-dessus de LOW au dernier tour, la
  remédiation P1 (code de production : `fmt_decimal_scale`, `debug_assert!`) ayant été relue en passe complète par la P2, non ciblée ; boucle
  close sur une passe sans finding au-dessus de LOW. Aucun reclassement, aucun finding réfuté. **Sort des quatre LOW de la P2** (prémisse relue
  avant chacun, aucun code de production touché, donc pas de passe de plus) :
  - **E-1 — corrigé.** Prémisse relue : `mut-m4` à `mut-m8` étaient antérieurs à la remédiation (20:33 à 20:52 contre 22:07). **M4 à M8
    rejouées au HEAD de clôture** par `mut/muter2.py`, une à une sous le verrou de la voie base (en-tête `== mutation`, avant/après, `EXIT=100`,
    `== restaure` ; `git status --porcelain` vide après chacune, `muter2-m*-run.log`) : M4 (`csv.rs` : 1 au lieu de 0), M5 (`money.rs` : 0 au lieu
    de 1), M6 (`pdf.rs` : 1 au lieu de 0) rouges sur la garde (`mut-m4.log` à `mut-m6.log`, 5 tests dont 1 rouge) ; M7 rouge (1 test) **par le
    `debug_assert!`** (`csv_tables.rs:130`, « échelle 4 de 1.1234 supérieure à celle de sa colonne (2) ») : la requête de l'export échoue et le
    test rougit sur la réponse (`exports_global_e2e.rs:2235`, `unwrap`), non sur « attendu 1.1234, lu 1.12 » ; M8 rouge sur le décompte
    (`exports_global_e2e.rs:2617`). Fiche (tableau des preuves AC13, tableau des mutations) réécrite pour M7. M1, M2b, M3 : journaux de la
    remédiation P1 conservés (M1 et M2b rejouées au code `2797e0447`, qui est aussi celui de production du HEAD).
  - **A-1 — corrigé.** `assert_eq!(compte(PRECISION, &sonde), 7)` lu à `arrondi_centime_garde.rs:220` ; trois « 6 » devenus « 7 » (AC9, tableau
    des preuves, Dev Agent Record) et la liste de la sonde complétée de `{:.1$}` ; greppé par la valeur sur tout le worktree : plus aucun « sondes
    positives 6 » ni « rend 6 ».
  - **A-2 — corrigé.** `README.md` (ligne v0.20.0) : « les montants des rapports (PDF et CSV) sont arrondis au centime, non tronqués, et
    l'export global écrit chaque nombre à l'échelle où Kesh le garde » ; reste de la ligne inchangé.
  - **A-3 — corrigé.** `dev/ac8.txt` (18 lignes) et `dev/ac12.txt` (22 lignes) régénérés au HEAD par les mêmes commandes (anciens :
    `ac8.avant.txt`, `ac12.avant.txt`) ; la fiche cite `csv_tables.rs:135`, `money.rs:110` et `:113`. Les 22 occurrences et les 8 fichiers de la
    liste close sont inchangés.
  **Gates de clôture** au HEAD `5d8ded148` (`dev/gate-c-commit.txt` ; arbre propre, `dev/gate-c-status.txt` vide ; seul le code de
  `2797e0447` et le test de `65c6d0e30` diffèrent du gate de la remédiation, et ce dernier est maintenant gaté) : bases remises à neuf
  (`gate-c-bases.log`) ; `scripts/test-fast.sh --ci` **vert, 6224 passés (1 lent), 14 ignorés, 0 échec** (`gate-c-test-fast-ci.log`) ; base
  `kesh_e4811` remise à neuf ; `scripts/gate-ci.sh kesh_ci_e4811` **code 0**, **259 binaires, 6243 passés, 0 échec, 18 ignorés**
  (`gate-c-ci.log`, `gate-c-ci.console` ; `gate-c-exit.txt`). **Frontend : non rejoué**, `git diff --stat 15ae0368a~1 HEAD -- frontend` vide
  (le seul `npm ci` + `npm run build` du worktree a produit `frontend/build` pour l'E2E : `dev/front-build.log`). **E2E complet** au même
  commit (`dev/e2e-cloture.sh`, `dev/e2e-cloture.log` ; `kesh-api` construit au commit, base `kesh_e2e_e4811` recréée, port 3481,
  `KESH_COOKIE_SECURE=false`, `KESH_TEST_MODE=true` des deux côtés) : **491 passés, 7 échecs, 18 ignorés, 22,2 min**. Les sept échecs sont
  **exactement** ceux de la KF-029 (#97), jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`
  et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77` et `:150` ; **zéro échec de pollution**. Aucun autre
  rouge. Mots-clés de fermeture : `git log 08c47a6cc..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide (P2).
  Modèles : validations P1 Sonnet, P2 Opus, P3 Sonnet ; revues de code P1 Opus, P2 Sonnet.

- **2026-11-24 — revue P1 remédiée** (Sonnet 5.5). Revue P1 : Opus, trois lentilles B, E, A exercées l'une après l'autre par un seul agent,
  **0 CRITICAL / 0 HIGH / 1 MEDIUM / 10 LOW** (B : 0/0/0/3 ; E : 0/0/1/2 ; A : 0/0/0/5). Code de la remédiation `2797e0447` (gates et mutations
  à ce code). **Un dernier commit de test suit le gate** : il retire du test `amount_string_ecrit_un_zero_negatif_sans_signe` la ligne
  `amount_string(dec!(-0))` que le premier Edit n'avait pas ôtée (cas qui n'est pas un zéro négatif) — une assertion en moins, `cargo fmt
  --check` vert, **ni gate ni test relancés** : à rejouer à la passe suivante (gate ciblé). Sort de chaque finding :
  - **E-1 (MEDIUM) — corrigé.** Prémisse relue au code : `dec!(-0)` est un zéro positif ; `manual.rs:790` (`-value`) fabrique un vrai zéro
    négatif, qui atteint `amount_string` brut (`bank_statements_manual.rs:354`, `:358`). Cas `dec!(-0)` retirés de `money.rs` et du test de
    `amount_string` (grep `dec!(-0)` : plus un seul site d'assertion, seuls des commentaires l'expliquent) ; tests neufs
    `amount_string_ecrit_un_zero_negatif_sans_signe` (`-Decimal::ZERO`, `-Decimal::new(0, 2)`) et `un_solde_saisi_moins_zero_s_ecrit_zero_zero`
    (relevé manuel saisi `-0`, **prémisse assertée** : `is_sign_negative()` sur la valeur rendue par `parse_manual_statement`, puis `"0.00"`).
    **M2b** (retrait de `if r.is_zero()`) : **4 rouges** (`mut-m2b.log` : `money.rs` 2, `bank_statements_manual.rs` 2, dont le chemin réel),
    restauration `git restore --worktree` puis `touch`, `if r.is_zero()` relu à `money.rs:102`. Phrase M2 du Dev Agent Record réécrite
    (la source est le solde `-0` saisi, non une valeur lue) ; AC6 de la fiche réécrit.
  - **B-1 (LOW) — corrigé** : le doc-comment de `format_two_decimals` dit « le seul endroit **des rapports et de `kesh-api`** » et renvoie à la liste
    close de la garde (prémisse vérifiée : `formatting.rs:22`, `pain001/mod.rs:125`, `generator.rs:115` écrivent `{:.2}` après arrondi).
  - **B-2 (LOW) — écrit, non corrigé** : les deux tests du rendement (`csv.rs`, `pdf.rs`) prouvent le formateur seul ; leur doc-comment dit que
    `project_report.rs:437` arrondit déjà au pair (`round_dp(2)`, `12.345` s'affiche `12.34%`), renvoi à #1442 (déjà ouverte). Le rendement
    affiché ne passe pas à « loin de zéro » avant #1442.
  - **B-3 (LOW) — corrigé** : `debug_assert!(d.scale() as usize <= scale)` dans `fmt_decimal_scale` (production, hors build release) ; l'AC13
    tient les 38 colonnes, le gate vert montre qu'aucune valeur lue n'a une échelle supérieure à sa colonne.
  - **E-2 (LOW) — corrigé** : motif `PRECISION` de la garde élargi à `\.([0-9]+\$?|…)`, sonde positive à **sept** formes (`{:.1$}`), doc de la
    garde mise à jour ; aucun site dans `crates/*/src` (liste close inchangée : 22 / 8).
  - **E-3 (LOW) — écrit** : la borne du tampon (32 caractères ; `DECIMAL(19,4)` 20, `DECIMAL(20,10)` 21) est au doc-comment de `fmt_decimal_scale`.
  - **A-1 (LOW) — corrigé** : décompte recompté (`git grep -nE '1245|tronqu' -- docs/manual` : dix lignes, douze occurrences, toutes sans rapport),
    écrit au Dev Agent Record (AC12).
  - **A-2 (LOW) — corrigé** : M1 **rejouée** par `mut/muter2.py m1` (`mut-m1.log` porte maintenant `== mutation`, avant/après, **713 tests, 14
    rouges**, `EXIT=100`, `== restaure`) ; M2b par le même script ; l'ancien journal de M1 (par `run.sh`, 620 tests, 14 rouges) est remplacé.
  - **A-3 (LOW) — déclaré** : écart (5) ajouté aux « Écarts à la fiche » (tests unitaires de `csv_tables.rs` non rougis d'abord).
  - **A-4 (LOW) — corrigé** : `admin-manual.tex:2556` : « Les taux de TVA, … ; les cours de change gardent leurs dix décimales ».
  - **A-5 (LOW) — corrigé** : `docs/api-external.md` porte un paragraphe « L'export global change de forme » *(depuis la v0.20.0)*, avec ⚠️.
  - **Propagation** : greps `dec!(-0)` (2 sites de test traités), `seul\*\* endroit` (1), `{:.1\$}` (aucun site), `tronque à deux décimales` (PDF seul, attendu).
  - **Gates** (commit `072006c2a`, code = `2797e0447`) : bases remises à neuf (`dev/gate-r-bases.log`) ; ciblé `dev/vert-remed.log` **987 passés** ;
    `scripts/test-fast.sh --ci` **6224 passés (1 lent), 14 ignorés, 0 échec** (`dev/gate-r-test-fast-ci.log`) ; `scripts/gate-ci.sh kesh_ci_e4811`
    **code 0** (`dev/gate-r-ci.log` : 259 binaires, **6243 passés, 0 échec, 18 ignorés**, recompté par les lignes `test result:`) ; `dev/gate-r-exit.txt`.
    Frontend non touché ; pas d'E2E (clôture). **Remédiation de code de production** : `fmt_decimal_scale` (`csv_tables.rs`, `debug_assert!`) et
    commentaires de `money.rs` : une **passe ciblée** de ce seul commit est due (prompt P2 : passe complète, qui l'inclut).
  - Outillage : `muter2.py` a été lancé tel quel, les tests de remédiation (`bank_statements_manual.rs`) ont été posés par un script Python
    (écart à la consigne « Edit seulement », sans effet sur le code : relu au diff `2797e0447`).
  - Prompt de la revue P2 : `48-11-arrondi-au-centime-review-prompt-p2.md`.

- **2026-11-23 — développement terminé, passage en `review`** (Sonnet 5.5, reprise après crash). Rouges constatés (6 + 5) ; code `9a6c80f81` ;
  tests de zéros négatifs `e704d397a` (M2 avait survécu) ; M1 à M8 tuées ; `test-fast --ci` 6222 verts et `gate-ci` code 0 au commit `e704d397a` ;
  E2E Playwright non lancé (D9, à la clôture). `Status: review`. Prompt de la revue P1 : `48-11-arrondi-au-centime-review-prompt-p1.md`.

- **2026-11-20 — création de la fiche** (Opus 5.5, `bmad-create-story`, code lu à `08c47a6cc`). Inventaire par le symptôme en cinq
  recherches (R1 à R5, sorties dans `kesh-gate-logs/e4811/`) : **30 occurrences de précision de format dans 10 fichiers** (R1), 21
  fonctions `Decimal → String` (R2), 9 découpages au point (R3), troncatures et arrondis explicites (R4), 78 lignes du frontend (R5).
  Classement : **corrigés** — `kesh-report` 5 sites (`format_swiss_amount`, `format_amount_iso`, `format_rate`, deux `format_rendement`)
  et `balance_sheet.rs` (#1430), `kesh-api` 3 sites (`fmt_decimal`, `amount_string`, `group_max_for_display` par DRY) ; **assumés** — 23
  occurrences restantes dans 8 fichiers, chacune justifiée (fonction commune, échelle de colonne, arrondi préalable, `f32`, tests) ;
  frontend sain ; `kesh-qrbill` sain et non touché. Mode : `MidpointAwayFromZero`, zéro sans signe (`round_for_display`, `pdf.rs:349-352`).
  #1430 se ferme avec la story. Quatre modules (crate, manuels). Issues demandées : I1, I2 (I3 selon Q1). Questions à Guy : Q1 (échelle de
  l'export global), Q2 (pain.001).
- **2026-11-20 — validation P1 remédiée** (Opus 5.5 ; passe P1 Sonnet 5.5 sur `292af6883` : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW ;
  rapport `kesh-gate-logs/e4811/validation-p1.md`). Prémisse de chaque finding relue au code avant remède.
  - **Décisions de Guy du 2026-11-20 intégrées.** **Q1** : l'export global écrit chaque colonne à l'échelle du schéma, sans arrondi —
    D5 réécrite (formateur unique `fmt_decimal_scale`, constantes `DEC_*` nommées d'après le type SQL, classement fermé des 34 sites :
    **24 à quatre décimales, 10 à deux** ; avec `:430-431` et `fmt_rate`, **38 colonnes `DECIMAL` exportées, 25 à 4, 10 à 2, 3 à 10**,
    recomptées depuis `test-schema/0001_schema_squash.sql` — 40 colonnes `DECIMAL`, moins les 2 de `_kesh_fetched_exchange_rates`, seule
    table hors `TABLES_EXPORTEES` —, journal `q1-echelles-colonnes.txt`) ; AC4 réécrit ; AC13 neuf (test de pleine échelle,
    l'ensemble des colonnes lu dans `information_schema`, valeurs dérivées de `NUMERIC_SCALE`, décomptes écrits à la main) ; les cinq
    attendus figés à deux décimales de `csv_tables.rs` (`:2461`, `:2485`, `:2496`, `:2518`, `:2519`) nommés ; D8 étendue (phrase
    `admin-manual.tex:3915`, phrase de `user-manual.tex` § *Contenu du ZIP*, paragraphe « Pour les intégrations » d'une section
    `sec:maj-0-20-0`) ; Note de version (export sous **Modifié**, mention en tête) ; *Production*, *Ce qui doit rester vrai* ; I3
    tombe. **Q2** et issues : pain.001 → **#1441**, I1 → **#1442**, I2 → **#1443**. **#1430** rattachée à v0.20.0 (décision de Guy,
    remplace V1-6).
  - **V1-1 (MEDIUM) → corrigé** : AC8 par `git grep -nw 'round_for_display'` (toutes formes, l'import entre accolades de `csv.rs:25`
    compris), liste fermée des lignes admises, `grep 'crate::pdf' balance_sheet.rs` vide ; sort écrit des appels directs (D3 : `pdf.rs:358`,
    `:899`, `:1116`, `csv.rs:59`, `:225` retirés ; reste `balance_sheet.rs:142`) ; la dépendance `csv.rs` → `pdf.rs` par
    `format_exchange_rate` dite, hors #1430.
  - **V1-2 (LOW) → corrigé** : le test `pdf.rs:3100-3109` reçoit un `use` (AC3 le dit), le lien `balance_sheet.rs:126` vise `kesh_core`.
  - **V1-3 (LOW) → corrigé** : stratégies non couvertes (`MidpointTowardZero`, `RoundHalfDown`, `ToNegativeInfinity`, `.floor()`,
    `.to_i64()`) et échange à compte égal dans un fichier écrits aux angles morts de D6 (grep des formes, lignes non commentées : vide).
  - **V1-4 (LOW) → corrigé** : `kesh-qrbill/src/pdf.rs` déclaré fichier commun avec l'Epic 49 au sens de la garde (13 occurrences
    figées, plus 1 dans `generator.rs`), conséquence et consigne écrites aux Dev Notes.
  - **V1-5 (LOW) → corrigé** : AC5 sans échappatoire (le texte du PDF se lit côté `kesh-api` : `printpdf-0.7.0/src/pdf_document.rs:822-831`
    ne compresse qu'hors `debug_assertions`, lu) ; AC12 au motif de la garde (32 lignes à `08c47a6cc`, 25 non commentées), sites de
    commentaire à réécrire recensés.
  - **V1-6 (LOW) → résolu par décision** : #1430 au milestone v0.20.0 (vérifié par `gh issue view 1430` : `v0.20.0`).
  - **Recomptes** (périmètre : `crates/*/src/**/*.rs`, 334 fichiers, lignes `//` retirées, code à `08c47a6cc`) : famille 1, **30 / 10**
    avant (inchangé) et **22 / 8** après la story (était 23 : `csv_tables.rs` passe de 2 à 1 sous Q1) ; famille 2, 2 (inchangé) ;
    formes de V1-3, 0. Correction d'un décompte faux à sa date : `fmt_opt_decimal_scale` a **2** appels (`:430-431`), non 3 (ligne du
    tableau corrigée).
  - **Charge révisée** : une journée à une journée et demie de développement (Dev Notes).
- **2026-11-20 — validation P2 remédiée** (Opus 5.5 ; passe P2 Opus 5.5 sur `549e68c21`, contexte frais : 0 CRITICAL, 0 HIGH,
  1 MEDIUM, 5 LOW ; rapport `kesh-gate-logs/e4811/validation-p2.md`). Tendance : **P1 0/0/1/5 (Sonnet) → P2 0/0/1/5 (Opus)** ; le
  MEDIUM de la P2 et trois de ses LOW sont nés de la remédiation P1 (AC13, AC5, AC8), deux LOW sont préexistants. Prémisses relues
  au schéma et au code avant remède.
  - **V2-1 (MEDIUM) → corrigé** : AC13 réécrit — principe « la cellule vaut la valeur posée écrite à `NUMERIC_SCALE` », comparaison
    par `(table, colonne, clé)` (clé `company_id` pour `company_invoice_settings`, `id` ailleurs) ; trois lignes nommées pour
    `journal_entry_lines` (L1 débit seul, L2 crédit seul, L3 en devise à `debit = 1.26 = round(1.1234 × 1.1234567891, 2)`, cellule
    `1.2600`) sous `chk_jel_debit_credit_exclusive`, `chk_jel_fx_all_or_none`, `chk_jel_fx_chf` ; `exchange_rates` à `source = 'afc'` et
    `acknowledged_at` (`chk_exchange_rates_ack_pair`, `_ack_afc`), deux cours distincts ; toute dérogation nomme sa contrainte. Revue
    des contraintes `CHECK` des 19 tables (schéma à `08c47a6cc`, 86 contraintes `CHECK` lues ; aucun déclencheur) et de la migration de la
    49-1a : hors les trois cas nommés, toutes les contraintes sur une colonne `DECIMAL` sont des bornes que les valeurs par défaut
    respectent ; `chk_*_unpriced_neutral` (49-1a) → `UPDATE` des lignes `priced` par `id`.
  - **V2-2 (LOW) → corrigé** : les trois fixtures à écrire nommées (`dunning_levels`, `invoice_reminders`, `exchange_rates` ; décompte
    des `INSERT` du fichier relu : 22 tables en ont, dont 14 des 19 ; avec le préréglage et `post_entry`, 16) ; charge révisée à une journée et demie.
  - **V2-3 (LOW) → corrigé** : `csv_tables.rs` (`ac5471e07` : `line_type` sous `:659` et `:1300`), `exports_global_e2e.rs`, la contrainte
    de la 49-1a et `sec:maj-0-20-0` (déjà ouverte sur `epic/49` et sur la 48-12) déclarés communs ; D8 : même commentaire LaTeX que la
    48-12, le second arrivé fusionne les paragraphes.
  - **V2-4 (LOW) → corrigé** : AC5 cite `write_text` (`pdf_layer.rs:461`, `:467`), la police intégrée (`pdf.rs:440`) et `write_right`.
  - **V2-5 (LOW) → corrigé** : `amount_string` reclassé en délégation défensive (solde précédent `DECIMAL(18,2)`, `bank_imports.rs:368`).
  - **V2-6 (LOW) → corrigé** : la liste fermée d'AC8 admet toute ligne de `money.rs`, dont l'appel dans `format_two_decimals`.
  - Décomptes inchangés par cette remédiation (garde 22 / 8, famille 2 : 2 ; 38 colonnes, 25/10/3 ; 34 sites, 24/10).
- **2026-11-20 — validation P3 remédiée ; validation terminée** (Opus 5.5 ; passe P3 Sonnet 5.5 sur `97415d694`, contexte frais :
  0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW ; rapport `kesh-gate-logs/e4811/validation-p3.md`). Tendance : **P1 0/0/1/5 (Sonnet) → P2
  0/0/1/5 (Opus) → P3 0/0/0/4 (Sonnet)** ; aucun LOW ne recycle un défaut antérieur ; aucune passe de plus. Modèles alternés. Prémisses
  relues dans le rapport (schéma, fixtures du fichier de test).
  - **V3-1 (LOW) → corrigé** : « aucune valeur enregistrée ne change » (Note de version, D5, paragraphe du manuel d'administration de D8) ; la
    fraction de centime coupée jusqu'ici paraît entière ; la forme seule change pour les valeurs sans fraction de centime.
  - **V3-2 (LOW) → corrigé** : AC13 — le test neuf monte son propre jeu, sans toucher aux fixtures existantes ; les `INSERT` existants sont
    situés (corps de tests distincts, non `seed_with_full_data`).
  - **V3-3 (LOW) → corrigé** : AC13 — L1, L2, L3 distinctes, sans devise au départ ; L3 posée par un seul `UPDATE` des cinq colonnes.
  - **V3-4 (LOW) → déclaré** : l'AC13 est le seul morceau déplaçable ; à dire au Project Lead si le temps manque (Dev Notes).
