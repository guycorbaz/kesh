# Story 48-11 : Les montants à deux décimales sont arrondis au centime, non tronqués — exports CSV, rapports CSV et PDF (#1245 ; #1430)

Status: ready-for-dev

*Fiche rédigée le 2026-11-20 (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-48-11`, branche `story/48-11-arrondi-au-centime`, partie de
`epic/48-corrections-0-20-0` à **`08c47a6cc`** (qui porte la PR #1327 de Beat — alignement à droite des PDF de rapports — et la PR #1346 —
`round_for_display`). **Tout le code cité est lu à `08c47a6cc`.** Release **0.20.0** (coupure le jeudi 2026-11-26 au soir), story
**prioritaire** de l'Epic 48 (plan `release-0.20.0.md`, ligne 2 ; fichier d'épique § *48-11*). **Aucune migration** : aucun montant stocké ne
change ; seuls changent des affichages et des fichiers produits, au demi-centime près. Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e4811/`.
Clé courte des bases : `e4811` (`kesh_e4811`, `kesh_ci_e4811`, `kesh_e2e_e4811`).*

## Story

En tant que comptable qui tient ses livres dans Kesh et remet à son fiduciaire les rapports PDF et CSV et l'export global, je veux que
chaque montant à deux décimales y soit **arrondi au centime**, le demi-centime s'éloignant de zéro, comme à l'écran, afin qu'un montant
de 10.0050 ne s'imprime plus 10.00 dans un fichier et 10.01 à l'écran.

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
| `kesh-report/src/pdf.rs:368-373` `format_exchange_rate` | `format!("{r:.prec$}", prec = scale as usize)` après `round_dp_with_strategy(10, …MidpointAwayFromZero)` et `prec = scale.max(4)` | cours de change | **assumé** : la précision n'est jamais inférieure à l'échelle de la valeur (elle **complète**, ne coupe pas) |
| `kesh-report/src/pdf.rs:3337` | `"{nom} : « {} » finit à {bord:.3} mm, …"` | un `f32` de géométrie, dans `mod tests` (`:2161`) | **assumé** : pas un `Decimal` |
| `kesh-report/src/csv.rs:50-51` `format_amount_iso` (62 appels hors tests) | `format!("{amount:.2}")` | tous les montants CHF des CSV de rapports | **corrigé** (D3) |
| `kesh-report/src/csv.rs:58-59` `format_currency_amount_iso` | `format_amount_iso(round_for_display(amount))` | montants en devise | **sain** ; simplifié par D3 |
| `kesh-report/src/csv.rs:745-747` `format_rendement` (2 appels) | `Some(p) => format!("{p:.2}%"),` | rendement d'un projet | **corrigé** (D4) |
| `kesh-report/src/balance_sheet.rs:69`, `:142` | `use crate::pdf::round_for_display;` / `!round_for_display(self.retained_earnings).is_zero()` | le modèle du bilan importe une fonction du module de rendu (**#1430**) | **corrigé** (D2) |
| `kesh-api/src/exports/csv_tables.rs:108-109` `fmt_decimal` (29 appels, plus 5 de `fmt_opt_decimal` `:113`) | `format!("{d:.2}")` | export global : débit, crédit, prix, quantité, taux, totaux, frais, soldes de relevé, réglages de facturation | **corrigé** (D3, D5) |
| `kesh-api/src/exports/csv_tables.rs:119-120` `fmt_opt_decimal_scale` (3 appels) | `d.map(\|d\| format!("{d:.scale$}"))` | `amount_currency` à 4, `exchange_rate` à 10 (`:430-431`) | **assumé** : précision égale à l'échelle de la colonne (`DECIMAL(19,4)`, `DECIMAL(20,10)`) : complète, ne coupe pas |
| `kesh-api/src/exports/csv_tables.rs:880-881` `fmt_rate` | `format!("{d:.10}")` | cours `DECIMAL(20,10)` | **assumé** : idem |
| `kesh-api/src/routes/bank_statements_manual.rs:551-555` `amount_string` (14 appels : réponses, détails d'audit, `BankStatementUnbalanced`) | `format!("{:.2}", d)` | sommes d'un relevé saisi (bornées à deux décimales, `kesh-core/src/bank_imports/manual.rs:43` `MANUAL_AMOUNT_MAX_DECIMALS: u32 = 2`), mais aussi le solde de clôture du relevé **précédent**, lu en base (`opening_gap`, `:524-547`), qui peut venir d'un CAMT | **corrigé** (D3) |
| `kesh-api/src/routes/company_invoice_settings.rs:45-49` `group_max_for_display` | `"{:.2}",` sur `value.round_dp_with_strategy(2, …MidpointAwayFromZero)` | plafond de l'écart d'un règlement groupé | **sain** ; **délègue** à la fonction commune (D3, DRY), tests existants `:599-600` en filet |
| `kesh-api/src/routes/reconciliation_group.rs:640-646` `fixed` | `round_dp_with_strategy(dp, MidpointAwayFromZero)` puis `rescale(dp)` | candidates d'un règlement groupé | **sain** (pas de précision de format) ; hors périmètre (D3) |
| `kesh-api/src/errors.rs:1566` `chf_text` | `n.rescale(2)` si `scale < 2`, sinon valeur entière | refus `FX_AMOUNT_MISMATCH` : **sans arrondi voulu** (`:1561-1564`) | **sain** |
| `kesh-qrbill/src/generator.rs:110-115` `format_amount_payload` | `format!("{:.2}", rounded)` après `MidpointAwayFromZero` (`:114`) | montant de la QR-facture | **sain**, **non touché** (factures émises, D7) |
| `kesh-qrbill/src/pdf.rs:1665-1698` `format_signed_ch`, `format_ch` | `round_dp_with_strategy(decimals, …MidpointAwayFromZero)` (`:1676`) | montants du PDF des factures et avoirs | **sain**, **non touché** (D7) |
| `kesh-qrbill/src/pdf.rs` (13 occurrences : `:84`, et `:4299` à `:5400` dans `mod tests` `:1901`) | `{:.3}mm`, `({:.2}, {:.2})` | `f32` de géométrie (module du QR, positions) | **assumé** : pas un `Decimal` |
| `kesh-payment/src/pain001/mod.rs:124-126` `amt` | `format!("{:.2}", d.round_dp(2))` | montants du pain.001 | **sain** pour la troncature (arrondi avant) ; **arrondi au pair** : hors périmètre, voir #822 et la question Q2 |
| `kesh-i18n/src/formatting.rs:16-22` `format_money` | `format!("{:.2}", abs)` après `MidpointAwayFromZero` (`:17`) | formatage i18n | **sain** ; `kesh-i18n` ne dépend d'aucun crate interne : non migré (D3) |
| `kesh-core/src/exchange_rates/parse/tests.rs:335`, `:339` | `"{kurs:.20}"` | message d'un test | **assumé** : test, pas un montant |
| `kesh-report/src/project_report.rs:437` `rendement_pct` | `.round_dp(2)` | rendement **au pair** (12.345 → 12.34) | hors symptôme (arrondi d'un autre mode, non troncature) : **issue à ouvrir** (I1) |
| `kesh-db/src/repositories/invoices.rs:459-461` `compute_line_total` | `(qty * unit_price).round_dp(4)` | **total de ligne stocké, au pair**, alors que l'écran l'estime au demi supérieur (`invoice-helpers.ts:31` `.round(4, ROUND_HALF_UP)`) | hors symptôme, **valeur stockée d'une facture** : **issue à ouvrir** (I2), jamais dans cette story |
| `kesh-core/src/accounting/vat.rs:148` | `rate_percent.round_dp(2)` | taux `DECIMAL(5,2)` : sans effet | sain |

**Le défaut de #1245 se trouve à cinq sites** (`format_swiss_amount`, `format_amount_iso`, `fmt_decimal`, `amount_string`, et les deux
formateurs de taux et rendement, `{:.2}` sur un `Decimal` qui ne tronquent rien aujourd'hui mais tronqueraient à la première valeur à
trois décimales). **Le PDF des factures (`kesh-qrbill`) n'a pas le défaut** : il arrondit déjà (`format_ch`, `:1676`) ; rien n'y est à
proposer.

**Ce qui sort à tort aujourd'hui, concrètement** : un rapport (journal, grand livre, balance, bilan, compte de résultat, TVA, échéancier,
projets) en PDF ou CSV dont une ligne ou un total porte une fraction de centime ; l'export global (débit/crédit d'une écriture saisie à
quatre décimales, total de ligne d'une facture à quantité ou prix fractionnaire, réglage `group_settlement_max_difference` posé à
`5.0060` — que le manuel d'administration décrit, `admin-manual.tex:3915`) ; un résidu négatif sous le demi-centime imprimé « -0.00 ».

### #1430 — le bilan importe sa règle d'arrondi du module de rendu

`kesh-report/src/balance_sheet.rs:69` `use crate::pdf::round_for_display;` et `:142` `!round_for_display(self.retained_earnings).is_zero()` :
le modèle dépend du module de présentation. La fonction commune de cette story (dans `kesh-core`) **absorbe** `round_for_display` :
`balance_sheet.rs` l'importe de `kesh_core`, `pdf.rs` et `csv.rs` aussi, et le CSV du bilan arrondit comme sa ligne affichée — les deux
griefs de #1430. **#1430 se ferme avec la story** (mot-clé de fermeture dans le corps de la PR d'intégration, jamais dans un commit).

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
    la valeur arrondie a une échelle ≤ 2, la précision ne fait plus que compléter.
  `kesh-report/src/pdf.rs`, `csv.rs` et `balance_sheet.rs` importent les deux de `kesh_core::types`. Ferme **#1430** (dépendance à rebours
  retirée, une seule règle d'arrondi dans un même export). **Option écartée** : un module d'arrondi dans `kesh-report` (laisse `kesh-api`
  sans la fonction, et `kesh-api` en a besoin) ; **option écartée** : une méthode sur `Money` (les sites manipulent des `Decimal` nus ;
  les envelopper ajoute du bruit sans garantie de plus).
- **D3 — les sites corrigés délèguent, sans se dupliquer** :
  - `format_amount_iso` (`csv.rs:50`) = `format_two_decimals(amount)` ; `format_currency_amount_iso` (`:58`) = `format_amount_iso(amount)`
    (le second arrondi devient redondant) ;
  - `format_swiss_amount` (`pdf.rs:316`) part de `format_two_decimals(amount)` au lieu de `format!("{amount:.2}")` ; le commentaire `:317`
    devient vrai ; `format_currency_amount` (`:355`) n'appelle plus `round_for_display` lui-même ;
  - `fmt_decimal` (`csv_tables.rs:108`) = `format_two_decimals(d)` ; l'en-tête `:17` et le doc-comment `:107` se réécrivent ;
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
- **D5 — l'export global arrondit au centime, comme le kickoff le décide** (fichier d'épique, § *48-11* : `csv_tables.rs:109`, « à
  corriger »). Les colonnes à quatre décimales (`quantity`, `unit_price`, `line_total`, `debit`, `credit`, `total_amount`) sortent donc
  **arrondies** à deux, au lieu d'être **tronquées**. **Option écartée, soumise à Guy (Q1)** : écrire chaque colonne **à son échelle**
  (`DECIMAL(19,4)` → quatre décimales), comme la Story 37-3a l'a fait pour `amount_currency` et `exchange_rate` (`csv_tables.rs:430-431`) :
  sans perte, mais elle change la forme de toutes les colonnes de l'export et s'écarte du remède décidé au kickoff ; une quantité de
  0.125 sort 0.13 sous D5, 0.1250 sous l'option écartée. La fiche ne tranche pas ce désaccord : elle suit le kickoff et le déclare.
- **D6 — la garde** (`crates/kesh-api/tests/arrondi_centime_garde.rs`, nouveau ; patron `date_du_jour_garde.rs` : listes closes à égalité
  stricte par fichier, ce fichier exclu nommément) :
  - **Famille 1 — précision de format** : sur `crates/*/src/**/*.rs`, lignes `//` retirées, compte des occurrences de
    `\{[A-Za-z_0-9.]*:[^{}"]*\.([0-9]+|[A-Za-z_][A-Za-z_0-9]*\$|\*)\}` (précision fixe, nommée `x$` ou passée en argument `.*`, avec ou
    sans largeur ni alignement ; la chaîne de format peut être seule sur sa ligne). Liste close **après** la story, une justification
    écrite par fichier, **23 occurrences dans 8 fichiers** : `kesh-core/src/types/money.rs` 1 (la fonction commune) ;
    `kesh-api/src/exports/csv_tables.rs` 2 (`.scale$`, `.10` : échelle de colonne) ; `kesh-core/src/exchange_rates/parse/tests.rs` 2
    (test) ; `kesh-i18n/src/formatting.rs` 1 (arrondi `:17`) ; `kesh-payment/src/pain001/mod.rs` 1 (arrondi au pair, #822) ;
    `kesh-qrbill/src/generator.rs` 1 (arrondi `:114`) ; `kesh-qrbill/src/pdf.rs` 13 (`f32`) ; `kesh-report/src/pdf.rs` 2 (`.prec$` de
    `format_exchange_rate`, `f32` du test `:3337`). Tout autre fichier doit compter **0** ; un compte qui diffère, dans un sens ou dans
    l'autre, rougit et nomme le fichier.
  - **Famille 2 — troncature explicite** : `\.trunc\(\)`, `trunc_with_scale\(`, `RoundingStrategy::(ToZero|RoundDown)` ; liste close :
    `kesh-core/src/bank_imports/manual.rs` 1, `kesh-api/src/routes/mcp_server_settings.rs` 1 (ni l'un ni l'autre n'affiche un montant).
  - **Sonde positive** (la garde n'est pas vraie par construction) : la fonction de comptage, appliquée à un texte écrit dans le test
    — `format!("{d:.2}")`, `format!("{:.2}", d)`, `format!("{d:>10.2}")`, `format!("{d:.scale$}")`, `format!("{:.*}", 2, d)`, une chaîne
    `"{:.2}",` seule sur sa ligne, et une ligne `// {:.2}` —, rend **6** ; la famille 2 sur `d.trunc()` et `RoundingStrategy::ToZero`
    rend 2 ; et la garde **voit au moins un fichier** (le balayage n'est pas vide : `sources().len()` > 100, et `money.rs` y figure). Les
    comptes attendus sont **écrits à la main** dans le test, jamais dérivés du même extracteur.
  - **Angles morts assumés** (écrits dans le doc-comment de la garde) : un `Decimal` écrit par `{}` ou `to_string()` sort à son échelle
    (quatre décimales) sans rien couper — c'est le symptôme de #813, non celui-ci ; une précision construite à l'exécution hors d'un
    littéral (aucun site) ; un découpage maison au point (R3 : aucun site sur un montant) ; le frontend (big.js arrondit, R5) ; l'arrondi
    au pair `round_dp(` (autre mode, non une troncature : I1, I2, #822).
  **Option écartée** : une garde par liste blanche de **fonctions** (« `{:.2}` interdit hors de `format_two_decimals` ») — elle ne voit pas
  une précision nommée ni un `f32`, et doit savoir lire le Rust ; la liste close par fichier est le patron éprouvé du dépôt.
- **D7 — factures réelles** : rien dans `kesh-qrbill`, ni dans `invoice_pdf_service.rs`, ni dans les écrans de facture. Le PDF d'une
  facture ou d'un avoir, archivé ou régénéré, reste identique octet pour octet à ce que la 0.19.0 rend (il arrondit déjà).
- **D8 — le manuel** : `admin-manual.tex:3915`, la phrase « l'export CSV des réglages la \textbf{tronque} … (\texttt{5.00}, défaut connu,
  issue~\#1245) ; l'écran et l'API l'\textbf{arrondissent} (\texttt{5.01}) » devient « l'écran, l'API et, depuis la 0.20.0, l'export CSV
  des réglages l'\textbf{arrondissent} (\texttt{5.01}) ». `.tex` seulement. Aucune autre langue ne porte la section. **Option écartée** :
  une phrase neuve aux quatre manuels sur l'arrondi des rapports (le format visible ne change pas — toujours deux décimales — ; seul un
  centime faux devient juste).
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
   (`:2352-2381`, et `:2687` « ni 0.00 ni -0.00 ») restent verts **sans modification**.
4. **AC4 — export global** (`kesh-api/src/exports/csv_tables.rs`, test unitaire ; `crates/kesh-api/tests/exports_global_e2e.rs`, test de
   route) : `fmt_decimal(dec!(10.0050)) == "10.01"` ; par la route `GET` de l'export global, une écriture dont une ligne porte
   `10.0050` au débit (posée par l'API ou par SQL dans la base de test) sort `10.01` dans le CSV des lignes d'écriture, et un réglage
   `group_settlement_max_difference` posé à `5.0060` par SQL sort `5.01` dans le CSV des réglages de facturation (le cas du manuel,
   `admin-manual.tex:3915`) ; `amount_currency` et `exchange_rate` sortent **inchangés** (quatre et dix décimales).
5. **AC5 — rapports par la route** (`crates/kesh-api/tests/reports_export_e2e.rs`) : une écriture à `10.0050` (débit/crédit) dans un
   exercice ; `GET …/trial-balance/export?…&format=csv` porte `10.01` ; la même en `format=pdf`, texte extrait, porte `10.01` (ou, si
   l'extraction n'est pas disponible côté `kesh-api`, l'AC3 en tient lieu et le Dev Agent Record le dit).
6. **AC6 — relevé saisi** (`bank_statements_manual.rs`, test unitaire du module) : `amount_string(dec!(10.0050)) == "10.01"`,
   `amount_string(dec!(-0)) == "0.00"`, `amount_string(dec!(50)) == "50.00"` (non-régression des réponses actuelles).
7. **AC7 — le plafond de l'écart** : `group_max_for_display` délègue à `format_two_decimals` ; ses tests existants
   (`company_invoice_settings.rs:599-600`, `5.0050` → `5.01`, `5.0150` → `5.02`) restent verts sans modification.
8. **AC8 — #1430** : `git grep -n 'crate::pdf::round_for_display\|fn round_for_display' -- crates/` ne rend que la définition dans
   `kesh-core/src/types/money.rs` ; `balance_sheet.rs` importe de `kesh_core` ; les tests du bilan (`pdf.rs:2670-2690` et ceux de
   `shows_retained_earnings_line`) restent verts.
9. **AC9 — la garde** (D6) : `arrondi_centime_garde` est verte avec la liste close de 23 occurrences dans 8 fichiers (famille 1) et de
   2 (famille 2) ; la sonde positive rend 6 et 2 ; le balayage voit plus de 100 fichiers.
10. **AC10 — factures réelles** (contrainte de Guy du 2026-11-18) : `git diff origin/epic/48-corrections-0-20-0...HEAD --stat --
    crates/kesh-qrbill crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-db 'frontend/src/routes/(app)/invoices'
    frontend/src/lib/components/invoices frontend/src/lib/features/invoices` **vide** (sortie copiée au Dev Agent Record) ; les tests de
    `kesh-qrbill` (dont ceux des PDF de facture et d'avoir) passent sans modification ; aucune facture, aucun avoir, aucune écriture,
    aucun montant stocké ne change (aucune migration, aucun repository touché).
11. **AC11 — manuel** (D8) : `sed -n '3915p' docs/manual/fr/admin-manual.tex | grep -c 'tronque'` rend 0 après la story (1 avant) ;
    `git grep -n '1245' -- docs/manual` vide ; PDF non commité.
12. **AC12 — propagation** (`CLAUDE.md`, § *Propagation post-patch*) : après le correctif, `git grep -nE ':\.2\}' -- crates/*/src/*`
    (sortie copiée, chaque ligne justifiée par la liste de la garde) ; `git grep -n 'format_amount_iso\|cohérent .kesh-report::csv'`
    sur les doc-comments : plus aucune phrase qui dit `{:.2}` pour un montant.

## Tests à écrire, et la preuve du rouge

Ordre : **le test d'abord, rouge**, puis le correctif, vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4811/`. Mutations par l'outil
**Edit** (jamais `git stash`) ; après restauration, `touch` du fichier (mémoire « mutation restaurée, binaire périmé »).

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC2 (`format_amount_iso`, rendu de la balance CSV, rendement) | le correctif de `csv.rs` | **rouge d'abord** : écrits avant le correctif, ils échouent sur `10.00` / `12.34%` ; `rouge-report.log` |
| AC3 (`format_swiss_amount`, rendu PDF, `format_rate`, rendement) | le correctif de `pdf.rs` | rouge d'abord (`"10.00"`, `"-0.00"`, `"8.12 %"`) ; `rouge-report.log` |
| AC4 (`fmt_decimal`, route de l'export global) | le correctif de `csv_tables.rs` | rouge d'abord (`10.00`, `5.00`) ; `rouge-api.log` |
| AC5 (route des rapports) | le correctif de `csv.rs` / `pdf.rs` | rouge d'abord ; `rouge-api.log` |
| AC6 (`amount_string(10.0050)`) | le correctif de `bank_statements_manual.rs` | rouge d'abord ; `rouge-api.log` ; les cas `-0` et `50` **ne rougissent pas** : filets |
| AC1 (fonction commune, neuve) | — (elle n'existe pas avant) | **M1** : stratégie remplacée par `ToZero` dans `round_for_display` → AC1, AC2, AC3, AC4 rouges (`mut-m1.log`) ; **M2** : retrait du `if r.is_zero()` → les cas `-0.0040` rouges (`mut-m2.log`) ; **M3** : `MidpointNearestEven` → `10.0050` rouge (`mut-m3.log`) |
| AC7 (plafond de l'écart) | **ne rougit pas** : sortie inchangée | filet déclaré (tests existants) |
| AC8 (#1430) | — (déplacement) | le `git grep` de l'AC8, sortie copiée |
| AC9 (garde) | un `{:.2}` réintroduit | **M4** : `let _ = format!("{:.2}", amount);` ajouté dans `kesh-report/src/csv.rs` → garde rouge, message qui nomme le fichier et dit 1 au lieu de 0 (`mut-m4.log`) ; **M5** : retrait d'un `{:.2}` légitime d'un fichier listé (par exemple la ligne de `money.rs` réécrite en `format!("{}", …)` le temps de la mutation) → rouge « 0 au lieu de 1 » (`mut-m5.log`) ; **M6** : `amount.trunc_with_scale(2)` ajouté dans `pdf.rs` → famille 2 rouge (`mut-m6.log`) ; la sonde positive rend 6 et 2 dans le même binaire |
| AC10, AC11, AC12 | — | commandes et sorties copiées |

Numérotation : **M1 à M6, uniques, un journal chacune.**

## Tâches

1. **Étape 0** : relire à la tête de la branche les lignes citées (`pdf.rs:316-318`, `:349-352`, `:1394-1395`, `:1666-1668` ;
   `csv.rs:50-59`, `:745-747` ; `csv_tables.rs:17`, `:107-121`, `:880-881` ; `bank_statements_manual.rs:551-555` ;
   `company_invoice_settings.rs:45-49` ; `balance_sheet.rs:69`, `:142`) ; en copier trois au Change Log ; relancer la simulation de la
   garde (`garde-simulation-avant.txt`) et noter tout écart (une fusion de `main` peut avoir ajouté un site : il entre au tableau, corrigé
   ou assumé, avant d'écrire la liste close).
2. Tests rouges (AC2 à AC6), journaux `rouge-*.log`.
3. `kesh-core` : `round_for_display`, `format_two_decimals`, réexport ; tests AC1 ; mutations M1 à M3.
4. `kesh-report` : `pdf.rs` (retrait de `round_for_display`, `format_swiss_amount`, `format_currency_amount`, `format_rate`,
   `format_rendement`), `csv.rs` (`format_amount_iso`, `format_currency_amount_iso`, `format_rendement`, en-tête `:7`), `balance_sheet.rs`
   (import, #1430) ; tests AC2, AC3, AC8.
5. `kesh-api` : `csv_tables.rs` (`fmt_decimal`, en-tête `:17`, doc `:107`), `bank_statements_manual.rs` (`amount_string`),
   `company_invoice_settings.rs` (`group_max_for_display`) ; tests AC4 à AC7.
6. Garde `crates/kesh-api/tests/arrondi_centime_garde.rs` (D6) ; mutations M4 à M6.
7. Manuel (D8), `.tex` seulement ; propagation (AC12) ; contrôle AC10 ; Dev Agent Record. **Ne pas écrire `CHANGELOG.md` ni
   `sprint-status.yaml`.**

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

**Corrigé**

- Rapports (PDF et CSV) et export global : les montants à deux décimales sont **arrondis au centime**, le demi-centime s'éloignant de
  zéro, comme à l'écran ; ils étaient jusqu'ici **tronqués** — un montant de 10.0050 sortait 10.00 au lieu de 10.01, −10.0050 sortait
  −10.00 au lieu de −10.01, et un résidu négatif sous le demi-centime s'imprimait « -0.00 ». Seuls les montants qui portent une fraction de
  centime changent ; aucune donnée enregistrée n'est modifiée, et le PDF des factures et des avoirs, qui arrondissait déjà, ne change
  pas (refs #1245).
- Export global : le plafond de l'écart d'un règlement groupé sort arrondi comme à l'écran (5.0060 → 5.01, non plus 5.00) (refs #1245).
- Interne : la règle d'arrondi d'affichage des rapports est commune au bilan, aux rapports PDF et CSV et à l'export global (refs #1430).

## Dev Notes

- **Modules** (règle de décompte de la 48-15 : une **crate** backend, un **domaine** frontend, ou **les manuels** ; les tests suivent leur
  module) : (1) `kesh-core` (`types/money.rs`, `types/mod.rs`), (2) `kesh-report` (`pdf.rs`, `csv.rs`, `balance_sheet.rs`), (3) `kesh-api`
  (`exports/csv_tables.rs`, `routes/bank_statements_manual.rs`, `routes/company_invoice_settings.rs`, tests `exports_global_e2e.rs`,
  `reports_export_e2e.rs`, `arrondi_centime_garde.rs`), (4) manuels (`admin-manual.tex`, une phrase). **Quatre modules : sous la borne
  de cinq, pas de découpage.** Au grain du fichier de production : neuf fichiers, un même geste (déléguer à la fonction commune).
- **Dépendances** : part de `epic/48` à `08c47a6cc`, après la fusion de #1327 (décision de Guy du 2026-11-18) et de #1346. Ne bloque ni
  n'est bloquée par l'Epic 49 (aucun fichier commun : l'Epic 49 touche `kesh-qrbill`, que cette story ne touche pas).
- **Fichiers communs** : `crates/kesh-report/src/pdf.rs` — PR #1327 et #1346 de Beat, **fusionnées** ; l'Epic 46 de Beat (rapports par
  groupes de comptes) touchera `kesh-report` : si sa branche est fusionnée dans `main` avant l'intégration, le second arrivé résout, et
  tout nouveau `{:.2}` de l'Epic 46 rougira la garde — c'est voulu ; **le dire à Beat** (canal technique) dès la fusion de la story.
  `admin-manual.tex` : toutes les stories ; PDF jamais commité.
- **Ce qui doit rester vrai** : deux décimales exactement dans chaque montant de rapport et d'export (Excel relit une colonne
  homogène, `csv.rs:48-49`) ; point décimal sans séparateur au CSV, apostrophe au PDF ; `amount_currency` à quatre et `exchange_rate` à
  dix décimales dans l'export global ; un cours de change à quatre à dix décimales (`format_exchange_rate`) ; le PDF des factures inchangé.
- **Limite connue, inchangée** : chaque ligne et chaque total s'arrondissent séparément ; la somme des lignes affichées d'un rapport peut
  différer d'un centime du total affiché quand plusieurs lignes portent des fractions de centime (c'était déjà vrai, et pire, avec la
  troncature). Le total reste juste ; c'est la présentation de valeurs exactes.
- **Production** : aucun montant stocké ne change ; un rapport ou un export **régénéré** après la mise à jour peut différer d'un centime
  de celui que la 0.19.0 avait produit, sur les seules valeurs à fraction de centime. Rien à faire sur le NAS.
- **Issues à ouvrir (demandées à l'orchestrateur)** :
  - **I1** — `kesh-report/src/project_report.rs:437` : le rendement d'un projet s'arrondit **au pair** (`round_dp(2)`, 12.345 → 12.34),
    non au demi loin de zéro comme le reste de l'affichage (P4).
  - **I2** — `kesh-db/src/repositories/invoices.rs:460` : le total de ligne d'une facture se calcule **au pair** à quatre décimales
    (`round_dp(4)`), alors que l'écran l'estime au demi supérieur (`invoice-helpers.ts:31`) ; divergence possible au cinquième chiffre
    (quantité × prix à huit décimales) entre l'estimation de l'écran et la valeur enregistrée. **Valeur stockée d'une facture : à traiter
    sans toucher aux factures émises** (P3).
  - (Selon la réponse de Guy à Q1) **I3** — export global : les colonnes à quatre décimales sortent à deux.
  - `kesh-payment/src/pain001/mod.rs:125` (`round_dp(2)`, au pair) : voisin de **#822** (CtrlSum) ; à rattacher à #822 ou à ouvrir à part,
    au choix de l'orchestrateur (Q2).
- Références : issue #1245 (et son commentaire du 2026-11-19 renvoyant à #813), #1430, #813, #822 ; fichier d'épique
  `epic-48-corrections-0-20-0.md` § *48-11* ; plan `release-0.20.0.md` ; `48-15-troncatures-silencieuses.md` (forme, décompte des
  modules) ; garde modèle `crates/kesh-api/tests/date_du_jour_garde.rs` ; journaux d'inventaire `kesh-gate-logs/e4811/inv-*.txt`,
  `garde-simulation-*.txt`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-11-20 — création de la fiche** (Opus 5.5, `bmad-create-story`, code lu à `08c47a6cc`). Inventaire par le symptôme en cinq
  recherches (R1 à R5, sorties dans `kesh-gate-logs/e4811/`) : **30 occurrences de précision de format dans 10 fichiers** (R1), 21
  fonctions `Decimal → String` (R2), 9 découpages au point (R3), troncatures et arrondis explicites (R4), 78 lignes du frontend (R5).
  Classement : **corrigés** — `kesh-report` 5 sites (`format_swiss_amount`, `format_amount_iso`, `format_rate`, deux `format_rendement`)
  et `balance_sheet.rs` (#1430), `kesh-api` 3 sites (`fmt_decimal`, `amount_string`, `group_max_for_display` par DRY) ; **assumés** — 23
  occurrences restantes dans 8 fichiers, chacune justifiée (fonction commune, échelle de colonne, arrondi préalable, `f32`, tests) ;
  frontend sain ; `kesh-qrbill` sain et non touché. Mode : `MidpointAwayFromZero`, zéro sans signe (`round_for_display`, `pdf.rs:349-352`).
  #1430 se ferme avec la story. Quatre modules (crate, manuels). Issues demandées : I1, I2 (I3 selon Q1). Questions à Guy : Q1 (échelle de
  l'export global), Q2 (pain.001).
