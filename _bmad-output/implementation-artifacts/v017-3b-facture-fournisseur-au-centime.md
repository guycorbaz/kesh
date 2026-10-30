# Story v017-3b : une facture fournisseur naît au centime ; celle qui ne l'est pas est signalée, non modifiée

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0173`, branche
`story/v017-3-rapprochement-attente` partie de `c5101b07` (tête de la candidate `v0.16.1-rc1`) ; la story se fusionne
dans la release **0.17.0**. Sous-story de `v017-3-rapprochement-attente`, découpée avec **v017-3a** (#671) — motif :
C-v017-3-1. Registre : `v017-3-choix-autonomes.md` (C-v017-3-1, C-v017-3-13 à C-v017-3-17, C-v017-3-18 à C-v017-3-20 ajoutés à la
validation P1, C-v017-3-24 à la validation P2 — **simplification** : la matrice de conduite est retirée). Clé courte : `v0173b`.
Issue : **#687** (P3, bug), en `refs` seulement dans les commits. Les numéros de ligne cités sont ceux de `c5101b07`.*

**Aucune migration. Aucune version bumpée. Aucune donnée existante réécrite** (contrainte du Project Lead du
2026-10-30, `kesh-gate-logs/prompts/v017-donnees-a-garder.md`).

## Story

En tant que Comptable qui saisit les factures de ses fournisseurs, je veux que Kesh **refuse** une ligne dont le montant
n'est pas au centime, et me **signale** une facture déjà enregistrée dont le total ne l'est pas, avec la marche à
suivre, afin que le règlement de chaque facture écrive au compte de la banque un montant qu'un relevé peut porter — et
donc que la réconciliation puisse la proposer ou la rattacher —, sans que Kesh ne modifie jamais d'office une facture
existante.

## Origine

- **#687** (P3) : « Une facture fournisseur peut porter un TTC à plus de deux décimales : son règlement écrit une ligne
  de banque sous le centime, que la réconciliation ne propose jamais. » Pistes : arrondir ou refuser à la création
  (« change le HT/TVA saisis et doit couvrir la complétion d'une pièce importée ») ; régler avec une ligne d'arrondi.
- **Contrainte du Project Lead du 2026-10-30** : « Une facture fournisseur existante avec un TTC sous le centime n'est
  pas modifiée d'office ; dis comment l'utilisateur la traite. »
- Voisinage : **#686 / Story 30-1b** (la proposition fournisseur exige l'égalité exacte du TTC et du débit,
  `repositories/reconciliation.rs:811-834`, prédicat `si.total_amount = ?` en `:823`) ; **#476** (côté client, l'écart sous le centime se traite au règlement) ;
  **v017-3a** (le rattachement à une écriture exige un montant égal : même conséquence).

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Les prémisses de #687 — confirmées

| Prémisse | Au code | Verdict |
|---|---|---|
| `line_total = quantité × prix` n'est pas arrondi | `let line_total = line.quantity * line.unit_price;` aux **deux** passes de `create_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:400`, `:426`) — l'issue citait `:389`, `:415`, décalés depuis | **Exacte** |
| Aucune limite d'échelle à la création | `CreateSupplierInvoiceLineRequest` reçoit `quantity`, `unit_price` en `Decimal` (`routes/supplier_invoices.rs:278-285`) ; `to_new_supplier_invoice` (`:468-…`) ne contrôle aucune échelle ; `grep -n "scale" crates/kesh-api/src/routes/supplier_invoices.rs crates/kesh-api/src/routes/imported_supplier_invoices.rs` : rien. Le formulaire borne à quatre décimales (`frontend/src/lib/features/journal-entries/balance.ts:23`, `AMOUNT_RE`), le serveur non | **Exacte** |
| Le TTC (`DECIMAL(19,4)`) peut porter des fractions de centime | `total_amount DECIMAL(19,4)` (`20260628000001_supplier_invoices.sql:35`) ; `total_ttc = Σ line_total + Σ TVA` (`supplier_invoices.rs:120-178`), la TVA arrondie au centime par ligne (`kesh-core/src/accounting/vat.rs:40-44`) | **Exacte** |
| *(Validation P1, R7.)* Pourquoi la règle porte sur **chaque ligne**, non sur le seul TTC | Deux lignes `0.005 + 0.005` donnent un TTC au centime ; mais `generate_purchase_journal_lines` écrit une ligne de débit de charge **par ligne** à `line_total` (`supplier_invoices.rs:140-145`, `debit: *line_total` en `:142` ; *citation `:146-151` corrigée à la validation P2, R6*) : une ligne sous le centime écrit au journal une ligne sous le centime, même quand le total est au centime. *La première rédaction disait « le TTC est sous le centime exactement quand une ligne l'est » : inexact, deux lignes sous le centime peuvent se compenser.* | **Règle par ligne retenue** |
| Le règlement depuis la fiche écrit une ligne de banque au même montant | `pay_in_tx_as` lit le TTC sur la ligne créanciers de l'écriture d'achat (`purchase_payable_line`, `:671-688`) et écrit `D créanciers TTC / C banque TTC` (`:919-952`) | **Exacte** |
| La proposition ne propose jamais une telle facture | `find_open_supplier_invoices_for_window` : `si.total_amount = ?` en égalité exacte (`repositories/reconciliation.rs:811-834`) ; test T-b9 `a_supplier_ttc_under_the_centime_is_masking_not_candidate` (`kesh-db/tests/reconciliation_repository.rs:2299-…`) | **Exacte** |
| Le côté client l'évite | quantité et prix bornés à `MAX_DECIMAL_SCALE` (`routes/invoices.rs:483`, `:502` ; `routes/limits.rs:28`), `line_total` arrondi à quatre décimales (`repositories/invoices.rs:458`), écart sous le centime traité au règlement | **Exacte** |

### Une conséquence que l'issue ne nommait pas

- **Le fichier pain.001 arrondit chaque montant, pas la somme de contrôle** : `InstdAmt` = `round_dp(2)` de chaque TTC
  (`kesh-payment/src/pain001/mod.rs:124-126`), `CtrlSum` = arrondi de la somme **non arrondie** (`:148-150`). Avec des
  TTC sous le centime, les deux peuvent différer, et la banque rejette le fichier. Hors périmètre (C-v017-3-16) : **#822**
  (ouverte, P3). La 3b ferme la source pour les factures nouvelles.
- La quantité ou le prix à plus de quatre décimales sont **arrondis en silence** par MariaDB à l'insertion
  (`quantity DECIMAL(19,4)`, `unit_price DECIMAL(19,4)`, `20260628000001_supplier_invoices.sql:84-85`), alors que
  `line_total` est calculé en Rust sur les valeurs non arrondies : la ligne enregistrée peut ne plus égaler `quantité ×
  prix`. La borne à quatre décimales (AC-2) le ferme.

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Refuser une ligne dont `quantité × prix` n'est pas au centime, aux deux routes de création, par une fonction pure unique de `kesh-core` appelée par `create_in_tx` ; borner quantité et prix à quatre décimales | C-v017-3-13 **(Q1)** |
| D2 | L'existant : aucune migration, aucun `UPDATE`, paiement **non** refusé ; la fiche avertit, le manuel décrit les voies (D8) | C-v017-3-14 **(Q2)** |
| D3 | Complétion d'une pièce importée : la garde de ligne avant `AMOUNT_MISMATCH` | C-v017-3-15 |
| D4 | pain.001 (`CtrlSum`) : hors périmètre, #822 | C-v017-3-16 |
| D5 | Les montants sous le centime s'affichent aux quatre décimales (`formatExactAmount`), un seul `hasSubCentime` | C-v017-3-18 |
| D6 | Le formulaire garde son patron (refus au clic, message de cause) ; le bouton n'est pas désactivé | C-v017-3-19 |
| D8 | *(Validation P2.)* **Simplification** : la matrice de conduite par état est retirée ; un avertissement **unique et statique** sur la fiche, quel que soit l'état ; le manuel décrit les voies et leurs limites dans une section à `\label` | C-v017-3-24 |
| D9 | *(Validation P2, R2.)* La passe de forme des lignes (positivité, puis centime, puis TVA) est **une** fonction, appelée par la saisie et par la complétion, dans le même ordre | C-v017-3-15 (amendé) |
| D7 | **Dépendance à la v017-3a** : la 3b est livrée dans la même release ; la voie 7 du manuel (AC-16) la suppose | C-v017-3-20 |

## Critères d'acceptation

### La règle (`kesh-core`)

- **AC-1** — Une fonction pure, **seule définition** de la règle : `kesh_core::accounting::purchase_line_total(quantity:
  Decimal, unit_price: Decimal) -> Result<Decimal, LineNotInCentimes>` (module `accounting/purchase.rs` nouveau, ou
  `vat.rs` — à nommer au Dev Agent Record), qui rend `quantity × unit_price` si ce produit, **après `normalize()`**, a
  au plus deux décimales, et `Err(LineNotInCentimes { line_total })` sinon. Tests : `1 × 12.50` admis ; `3 × 33.3333`
  (`99.9999`) refusé ; `2 × 0.125` (`0.25`) admis ; `1.5 × 3.333` (`4.9995`) refusé ; `4 × 2.5000` (`10.0000`, échelle 4
  avant `normalize`) admis ; `0.1 × 0.1` (`0.01`) admis ; `0.3 × 0.1` (`0.03`) admis.

### La création (`kesh-db`, `kesh-api`)

- **AC-2** — `to_new_supplier_invoice` (`routes/supplier_invoices.rs:468`, **commune** à la saisie et à la complétion
  d'une pièce importée — AC13 de la 30-2b) refuse une quantité ou un prix à plus de `MAX_DECIMAL_SCALE` décimales
  **après `normalize()`** (`scale_within`, `routes/limits.rs:31` — qui, lui, ne normalise pas : c'est un écart voulu avec
  la facture client, à dire au doc-comment) : `400 VALIDATION_ERROR`, message qui nomme la ligne (rang 1-based) et le
  champ, comme la facture client (`routes/invoices.rs:483-510`, à reprendre sans recopier si une fonction s'en dégage).
  *(Validation P1, R9.)* **Ordre dans la conversion** : les bornes d'échelle des lignes viennent **avant**
  `validate_payment_details` (`:473-490`) — un défaut de saisie de ligne se dit avant un défaut de coordonnées ; ligne
  par ligne, la quantité avant le prix. *(Validation P2, R10.)* Les bornes d'échelle se contrôlent **avant** le `match`
  sur la source de paiement (`:473-490`), donc avant `QR_FIELDS_FROM_DOCUMENT` aussi bien que `validate_payment_details`.
- **AC-3** — `create_in_tx` appelle `purchase_line_total` dans sa **première passe** (la passe de forme, `:399-414`,
  avant les réglages, les comptes et l'exercice — « un refus de forme ne prend ainsi aucun verrou », `:415-419` ;
  ⚠️ l'étape (0) du projet analytique, `:370-372`, la précède et verrouille déjà : le dire au doc-comment, ne pas la
  déplacer), *(validation P1, R9)* **dans la boucle, ligne par ligne, après** le contrôle de positivité de la quantité,
  du prix et du produit (un produit nul ou négatif ne sort jamais en « pas au centime ») et avant les bornes de TVA, et
  rend une variante neuve `DbError::SupplierLineNotInCentimes { line: usize (1-based), line_total: Decimal }`,
  `line_total` **normalisé** (`4.9995`, jamais `4.999500`) ; la seconde passe (`:424-…`) relit `line_total` depuis le
  résultat de la première (aucun second produit). Côté API : `AppError` neuve, `400`, code
  `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, `details { line, lineTotal }`, message fr-CH « La ligne {line} donne un
  montant de {lineTotal}, qui n'est pas au centime. Saisissez la ligne au montant facturé par le fournisseur (par
  exemple quantité 1 et le montant de la ligne). » ; repli d'`errors.rs` égal au catalogue fr-CH ; quatre locales.
  *(Validation P1, F10.)* La garde **G9** de `crates/kesh-api/tests/textes_coherents.rs` (`:710-760`, table fermée
  fichier → clés et constantes de réécriture des variables Fluent) reçoit l'entrée de cette clé **et** la réécriture de
  ses deux variables (`{ $line }` → `{line}`, `{ $lineTotal }` → `{line_total}` ou la forme que la table emploie), faute
  de quoi G9 ne la contrôle pas.
  *(Validation P1, R9.)* **Une seule conversion** `LineNotInCentimes` (+ rang) → `AppError` (`From` ou fonction
  unique), employée par la création **et** par la complétion (AC-6) : aucune seconde construction du code d'erreur.
  *(Validation P2, R2, D9.)* La passe de forme (`:399-414` et la garde de centime) **s'extrait** en une fonction pure de
  `kesh-db` (`purchase_lines_form(&lines) -> Result<Vec<Decimal>, DbError>`, sans I/O : pour chaque ligne, dans cet
  ordre, positivité de la quantité, du prix et du produit, puis `purchase_line_total`, puis bornes de TVA ; rend les
  `line_total`). `create_in_tx` l'appelle à la place de sa boucle ; la complétion (AC-6) l'appelle aussi. **Un seul
  ordre, deux chemins.**
- **AC-4** — Invariant : une facture créée a un `total_amount` au centime. Test `#[sqlx::test]` sur quelques
  combinaisons (taux 0, 2.6, 8.1 ; plusieurs lignes) : `total_amount.normalize().scale() <= 2`, et la ligne créanciers
  de l'écriture d'achat l'égale. *(Validation P1, R3.)* **Et le refus au niveau du dépôt**, dans
  `crates/kesh-db/tests/supplier_invoices_repository.rs` : `supplier_invoices::create` avec une ligne `3 × 33.3333` →
  `DbError::SupplierLineNotInCentimes { line: 1, line_total: 99.9999 }` ; deux lignes dont la seconde est fautive →
  `line: 2` ; deux lignes `1 × 0.005` chacune (somme au centime, chaque ligne sous le centime — R7) → refus `line: 1` ;
  dans chaque cas, ni facture, ni écriture d'achat, ni trace d'audit en base. *(Validation P2, R1.)* Cas **négatif** :
  une ligne `-1 × 33.3333` (produit négatif **et** sous le centime) → le refus de **positivité** (`IllegalStateTransition`,
  message « strictement positifs »), **non** `SupplierLineNotInCentimes` ; de même `1 × -0.005`. Ce cas tue M10.
- **AC-5** — Tests HTTP (`POST /supplier-invoices`) : une ligne `3 × 33.3333` → `400
  SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, `details.line = 1`, `details.lineTotal = "99.9999"` ; *(validation P1, R5)*
  une quantité `1.00001` avec un prix `10` (produit `10.0001`, **pas** au centime : sans la borne de quantité, le refus
  serait celui du centime, non `VALIDATION_ERROR`) → `400 VALIDATION_ERROR` qui nomme la ligne et le champ `quantity` *(validation P2, R8 : `AppError::Validation` ne porte
  qu'un message, `errors.rs:153` — le test asserte le code et un fragment du message, « la quantité », comme
  `routes/invoices.rs:485`)* ;
  une quantité `100000` avec un prix `0.00001` (produit `1.00000`, **au** centime : seule la borne de prix l'arrête — sans
  elle, MariaDB stockerait le prix à `0.0000`, `20260628000001_supplier_invoices.sql:85`) → `400 VALIDATION_ERROR`, champ
  `unitPrice` (fragment « le prix unitaire », `routes/invoices.rs:504`) ; **rien d'écrit** (ni facture, ni écriture d'achat, ni trace), vérifié en base ; une saisie nominale au
  centime inchangée. *(Validation P2, R1.)* Et `-1 × 33.3333` → le refus de positivité, non le code du centime.
- **AC-6** — Complétion d'une pièce importée (`routes/imported_supplier_invoices.rs`) : ordre des refus — conversion
  (AC-2) → **garde de ligne** (la même fonction, avant le calcul de `Σ TTC`, `:251-275`) → `AMOUNT_MISMATCH` →
  `create_in_tx`. Le calcul de `Σ TTC` de la complétion lit `line_total` depuis `purchase_line_total` (aucun second
  produit). Le test (b) de `inbox_import_e2e.rs` (`:1375`, `3 × 33.3333 = 99.9999`) attend désormais
  `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, et un cas neuf à lignes au centime dont la somme diffère du QR garde
  `AMOUNT_MISMATCH`. La pièce reste à compléter après le refus (aucune transition).
  *(Validation P2, R2 — remplace l'ordre ci-dessus.)* Ordre : conversion (AC-2) → **`purchase_lines_form`** (positivité,
  centime, TVA : la même fonction et le même ordre que la saisie, D9) → `AMOUNT_MISMATCH` (`Σ TTC` calculé sur les
  `line_total` qu'elle rend) → `create_in_tx`. Un cas HTTP de complétion `-1 × 33.3333` rend **le même** refus que
  `POST /supplier-invoices` pour la même ligne. *(R12.)* Le « cas neuf » à lignes au centime est inutile : le cas (a) du
  même test (`inbox_import_e2e.rs:1365`, `1 × 99.00` contre un QR de `100.00`, `:1373` `AMOUNT_MISMATCH`) le garde déjà.
- **AC-7** — Les tests et fixtures qui créent des factures fournisseur **par les routes ou par `create_in_tx`** avec une
  ligne sous le centime se recensent (`grep -rnE "quantity|unit_price|unitPrice" crates/*/tests frontend/tests/e2e
  crates/kesh-seed` puis tri à la main) : chacun est adapté, ou réécrit pour insérer l'état **existant** par SQL s'il
  teste précisément un TTC sous le centime (ex. T-b9, qui insère déjà directement, `insert_supplier_invoice`). Liste et
  sort de chacun au Dev Agent Record. Le seed de démonstration (`kesh-seed`) n'en crée aucun après la story.
  *(Validation P1, axe R : le recensement de la lentille n'a trouvé qu'un site, le test (b) de `inbox_import_e2e.rs`
  `:1375-1400` ; `kesh-seed/src/pieges.rs:585` est au centime. À reconfirmer, non à supposer.)*

### L'existant (contrainte du 2026-10-30)

- **AC-8** — Aucun fichier sous `crates/kesh-db/migrations/` n'est ajouté ni modifié ; aucun `UPDATE`/`INSERT`/`DELETE`
  nouveau dans le code de production (`git diff c5101b07 -- crates/*/src | grep -nE '^\+.*\b(UPDATE|INSERT|DELETE)\b'`
  vide, tri au Dev Agent Record ; les montages de test de l'AC-9, sous `crates/*/tests/`, n'en relèvent pas).
- **AC-9** — Une facture **existante** au TTC sous le centime, comme une base 0.16.x, reste lisible (`GET
  /supplier-invoices/{id}`, liste), **payable** depuis la fiche (le paiement n'est pas refusé — C-v017-3-14), annulable
  ouverte, et son règlement annulable ; aucune de ces lectures ne la modifie (état relu à l'identique avant le geste).
  Tests HTTP. *(Validation P1, F6.)* **Le montage de la fixture**, dans un module d'aide **de test** (`crates/*/tests/`,
  jamais `src/`), en deux temps, sur la base éphémère du test seulement : (1) créer la facture au centime par
  `supplier_invoices::create` (une ligne `1 × 100.00`, taux 0) ; (2) amener **ensemble**, par SQL de test, la ligne de
  facture (`line_total`, `unit_price` à `100.0040`), `supplier_invoices.total_amount` et la ligne créanciers de
  l'écriture d'achat **et** la ligne de charge (`credit` et `debit` à `100.0040`, l'écriture restant équilibrée) — l'état
  exact qu'une 0.16.x produisait pour `1 × 100.004`. Pour l'état « payée », payer ensuite par `pay_in_tx` (le règlement
  écrit alors lui-même la ligne de banque à `100.0040`, lettrage compris). Le même montage sert à l'AC-11, à l'AC-19 et,
  joué dans `kesh_v0173b`, aux captures de l'AC-14 — jamais dans une base de Guy.
- **AC-10** — *(Réécrit à la validation P2, C-v017-3-24 : la matrice de conduite par état, ses conditions sur
  `cancellable`, `settlementCancellable`, `lastConfirmedBatch`, `settlementType` et `canManage`, et leurs tests sont
  **retirés**.)* La fiche de la facture (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`, total `:386`)
  affiche, quand `totalAmount` n'est pas au centime, **quel que soit l'état de la facture, annulée comprise** et quel que
  soit le rôle, un avertissement **unique et statique** (`data-testid="supplier-invoice-subcentime-warning"`) : « Le total
  de cette facture ({amount}) n'est pas au centime ; son règlement ne pourra pas être rapproché automatiquement d'un
  relevé bancaire. Voir le manuel, § « Une facture fournisseur au total sous le centime ». » Il ne promet aucun bouton
  et ne prescrit aucun geste. Le texte ne cite aucun numéro d'issue.
  `{amount}` **et** la cellule TTC de la fiche (`:386`) s'affichent aux quatre décimales quand le montant porte une
  fraction de centime, par `formatExactAmount` (`frontend/src/lib/features/invoices/invoice-helpers.ts:82-89`, réutilisé —
  déplacé vers un module partagé si l'import entre domaines l'exige, et le dire) ; la liste
  (`supplier-invoices/+page.svelte:93`) prend le même format. **Un seul** `hasSubCentime` : les deux homonymes
  (`invoices/invoice-helpers.ts:73`, `supplier-invoices/supplier-invoice-form.ts:63`) se réduisent à un ; *(validation P2,
  F10)* son contrat est celui du second — il accepte `null` et rend `false` sur `null` ou une valeur illisible
  (`try/catch`) —, sans quoi le rendu de `SupplierInvoiceForm.svelte:331` lèverait. Vitest : `100.0040` ouverte, payée
  et annulée → l'avertissement, avec « 100.0040 » (jamais « 100.00 ») ; `100.00` → aucun.
- **AC-11** — *(Réécrit à la validation P2, C-v017-3-24.)* Ce que le manuel affirme se **prouve** (§ AC-16) : un test HTTP
  joue la voie nominale sur la fixture de l'AC-9 payée depuis la fiche par virement — annulation du règlement (`POST
  /supplier-invoices/{id}/settlement/cancel`), puis de la facture (`POST /supplier-invoices/{id}/cancel`), ressaisie au
  centime, règlement — et constate que la ligne de banque du nouveau règlement est au centime. Les autres phrases du
  manuel qui décrivent un comportement s'établissent à l'étape 0, ligne lue citée au Change Log (§ *À recontrôler*), et
  s'écrivent au conditionnel si l'étape 0 ne peut pas les établir.
- **AC-19** — *(Validation P1, R4 ; contrainte « données à garder », § 2.)* **Restauration** : le test d'import
  d'installation (`crates/kesh-api/tests/admin_full_import_e2e.rs`, qui crée déjà des factures fournisseur, `:1731`)
  gagne une facture montée comme à l'AC-9 (`100.0040`, ouverte, et une payée) ; après export `.keshbackup` puis import
  d'installation, chacune se relit **à l'identique** : `total_amount`, ligne de facture, lignes de l'écriture d'achat et
  de règlement, à quatre décimales. La restauration n'appelle pas `create_in_tx` (`backup.rs:729`, `INSERT` générique)
  et ne doit pas l'appeler : le test le prouve.

### L'écran de saisie (`frontend`)

- **AC-12** — `SupplierInvoiceForm.svelte` : une ligne dont `quantité × prix` n'est pas au centime affiche sous elle
  « Le montant de la ligne ({lineTotal}) n'est pas au centime. Saisissez la ligne au montant facturé par le
  fournisseur. » (`data-testid="supplier-invoice-line-subcentime"`) — *(F11)* le remède dit comme au refus serveur.
  *(Validation P1, F1, C-v017-3-19.)* Le formulaire **garde son patron** : le bouton *Enregistrer* n'est désactivé que
  pendant l'enregistrement (`disabled={saving}`, `:579`) ; `linesInvalid` est lu au clic (`submit()`, `:203`). Au clic,
  une ligne sous le centime fait afficher dans `supplier-invoice-form-error` un message **dédié** (clé nouvelle :
  « Une ligne n'est pas au centime : corrigez-la avant d'enregistrer. »), non le message générique
  `supplier-invoices-err-lines` ; rien n'est envoyé. Le contrôle est une fonction pure de `supplier-invoice-form.ts`
  (`big.js`, jamais `parseFloat`), miroir de `purchase_line_total`, avec ses tests Vitest des mêmes cas que l'AC-1 ;
  *(F11)* elle rend « pas de refus » pour une saisie vide ou illisible (`try/catch`, comme `amountMismatch`), accepte la
  virgule décimale comme `AMOUNT_RE` (remplacée par un point avant `new Big`), et affiche `lineTotal` par `toFixed()`
  (jamais en notation exponentielle). Un refus serveur `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` s'affiche traduit, la
  ligne désignée : `SubmitError.field` (`supplier-invoice-form.ts:156`, `'iban' | 'reference' | null`) s'étend d'une
  désignation de ligne, et `describeSubmitError` (`:165` — *citation `:186` corrigée à la validation P2, R6 = F9*) gagne
  le cas.
  *(Validation P2, F5 = R7, C-v017-3-19 amendé.)* Le contrôle de centime est un **prédicat distinct**,
  `lineNotInCentimes(line)` (ou `firstLineNotInCentimes(lines)`), **hors** de `linesInvalid`. Ordre dans `submit()` : d'abord
  la date et `linesInvalid` (message générique `supplier-invoices-err-lines`), **puis** le prédicat de centime (message
  dédié) ; une ligne à la fois structurellement invalide et sous le centime reçoit donc le message générique. *(F11.)* Le
  miroir lit une quantité **vide** comme `1`, comme l'envoi (`SupplierInvoiceForm.svelte:218`, `quantity: l.quantity ||
  '1'`).
- **AC-13** — i18n : clés nouvelles dans les **quatre** catalogues ; `npm run lint-i18n-ownership` vert ;
  *(F10)* la constante `CLES_RELEVEES` de `frontend/src/lib/shared/i18n-un-repli-par-cle.test.ts` (`:180`, 344 à
  `c5101b07`) se **recompte** depuis le FTL fr-CH, avec son commentaire daté. *(Validation P2, F8 — corrige la phrase
  précédente.)* La garde lit les **sources** du front (« Celle-ci lit les SOURCES, pas les catalogues »,
  `i18n-un-repli-par-cle.test.ts:21`) : `CLES_RELEVEES` se recompte depuis les appels `i18nMsg` littéraux aux préfixes
  de `PREFIXES`, non depuis le FTL. Clés nouvelles, nommées : `supplier-invoices-errors-line-not-in-centimes` (refus
  serveur, repli d'`errors.rs`), `supplier-invoices-line-not-in-centimes` (message sous la ligne),
  `supplier-invoices-err-line-not-in-centimes` (message au clic), `supplier-invoices-subcentime-total-warning`
  (avertissement de la fiche) — noms définitifs au Dev Agent Record si le lint de propriété impose un autre préfixe. L'avis existant
  `supplier-invoices-subcentime` (« … impossible à atteindre par des lignes centime-exactes … », `messages.ftl` fr-CH
  `:2545`) devient vrai : le relire, ne pas le dupliquer.
- **AC-14** — Recette visuelle (thème clair, base de démonstration `kesh_v0173b`, montage de l'AC-9, jamais une base de
  Guy) — *réécrite à la validation P2, C-v017-3-24* : le formulaire avec une ligne refusée (message de ligne **et**
  message au clic) ; le refus serveur affiché ; la complétion d'une pièce avec QR et une ligne refusée ; la fiche d'une
  facture sous le centime **ouverte** et **annulée**, avec l'avertissement ; une fiche au centime, sans. Captures dans
  `/home/gcorbaz/devel/kesh-gate-logs/v0173b/captures/`, listées au Dev Agent Record.
- **AC-15** — E2E : *(F7)* la spec qui porte déjà les parcours du formulaire — `frontend/tests/e2e/supplier-documents.spec.ts`
  (ou `inbox-import.spec.ts`), avec l'assistant `fillFirstInvoiceLine` — gagne un test : une ligne `3 × 33.3333`
  affiche le message de ligne *(validation P2, F16 : `fillFirstInvoiceLine`, `helpers/inbox.ts:228-238`, ne remplit pas la
  quantité — remplir `supplier-invoice-line-qty` à part)* ; *(F1)* le clic sur *Enregistrer* affiche `supplier-invoice-form-error` avec le message
  dédié et n'enregistre rien ; corrigée en `1 × 99.99`, l'enregistrement passe.

### Les textes

- **AC-16** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré) — *réécrit à la validation P2,
  C-v017-3-24* :
  - § *Saisir une facture fournisseur* (`:1571-…`) — une ligne se saisit au centime (quantité et prix à quatre
    décimales au plus, montant de la ligne au centime), et pourquoi ;
  - § *Créer la facture d'un document* (`:1846-1851`) — la liste des refus gagne « une ligne qui n'est pas au centime » ;
  - dans § *Régler une facture fournisseur* (`sec:regler-facture-fournisseur`, `:1630`), une sous-section **« Une facture
    fournisseur au total sous le centime »** avec `\label{sec:facture-fournisseur-sous-centime}`, cible de
    l'avertissement de la fiche et de **tous** les renvois (F12). Elle décrit **en prose**, sans algorithme d'écran, les
    voies et leurs limites, chaque affirmation de comportement établie à l'étape 0 :
    1. pourquoi : une facture enregistrée avant la 0.17.0 a pu garder un total sous le centime ; Kesh ne la modifie
       pas ; son règlement par virement ne se rapproche pas automatiquement d'un relevé, et, ouverte, elle peut retenir
       la proposition d'une règle d'affectation pour un débit à 5 centimes près (L1, R11) ;
    2. **ouverte** : l'annuler, puis la ressaisir au montant facturé ;
    3. **payée** : annuler le règlement, **puis** la facture, dans cet ordre (R3) — annuler d'abord la facture laisse le
       règlement au grand livre, détaché ; si c'est arrivé, l'écriture de règlement détachée se contre-passe depuis le
       journal (*à établir à l'étape 0*, `[id]/+page.svelte:620-628`) ; une facture dont l'achat est dans un exercice
       clôturé mais le règlement dans un exercice ouvert : ne pas annuler le règlement tant que la facture ne peut pas
       l'être (F1) ;
    4. **exercice clôturé** : l'annulation suppose la **réouverture** de l'exercice par un Administrateur (R4,
       `routes/fiscal_years.rs:11`) ; si on ne la veut pas, la facture reste telle quelle — ses livres sont justes, seul
       le rapprochement automatique de son règlement est perdu ;
    5. **lot pain.001** : un lot en cours dont le fichier n'a pas été transmis (ou que la banque a refusé, #822) s'annule
       avant la facture ; un lot **exécuté** par la banque se confirme, puis la facture suit la voie « payée » ; si la
       banque a passé le lot en **un seul** débit, la transaction ne se rattache à aucune facture seule (#823) (F2, F4) ;
    6. **compte interne** : un règlement par compte interne (caisse, carte) n'a pas de débit au relevé ; rien n'est à
       rapprocher ; la correction n'est utile que pour un grand livre au centime (F3) ;
    7. **après ressaisie** : la nouvelle facture se règle par la réconciliation du débit déjà au relevé, ou depuis sa
       fiche puis par le **rattachement** de la transaction à son écriture (v017-3a, D7) ;
    8. **document joint** : détacher le document de l'ancienne facture le remet parmi les documents à traiter ; la
       nouvelle facture se crée depuis lui (F7, `routes/imported_supplier_invoices.rs:629-637`, à reconfirmer) ;
  - § *Un débit : les factures fournisseur* (`sec:matching-fournisseur`, `:2304`), § *Paiements fournisseurs*
    (`sec:paiements-fournisseurs`, `:1737`) et le diagnostic (`:3329`) — **renvoi** à
    `\ref{sec:facture-fournisseur-sous-centime}` (« un total à plus de deux décimales n'est jamais proposé » complété : cela
    ne se produit plus que pour une facture enregistrée avant la 0.17.0) ;
  - et tout ce que rend `grep -n "deux décimales\|sous-centime\|sous le centime\|centime-exact" docs/manual/fr/user-manual.tex`,
    trié. Le contrôle au PDF aplati (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) se fait sur des **fragments courts**, aux
    **deux** apostrophes (`'` et `’`).
  Les passages communs à la v017-3a se rédigent en **renvoi** à son texte et se réconcilient à la réunion des branches
  (§ *Dépendance à la v017-3a*).
- **AC-17** — `docs/api-external.md` : le code `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` au tableau des codes, la borne
  à quatre décimales de `POST /supplier-invoices` et de la complétion, « *(Depuis la v0.17.0.)* ». ⚠️ C'est un
  **changement de contrat** pour un client d'API qui envoyait des lignes sous le centime : le dire ainsi. *(F4.)* La
  phrase de `:602` (« un total à plus de deux décimales n'est jamais proposé, [#687] ») est complétée comme le manuel
  (ne se produit plus que pour une facture enregistrée avant la 0.17.0). Les doc-comments qui présentent #687 comme un
  angle mort ouvert — `crates/kesh-api/src/routes/reconciliation.rs:344`, `crates/kesh-db/src/repositories/reconciliation.rs:805`,
  `crates/kesh-db/tests/reconciliation_repository.rs:2302` — restent vrais pour l'existant : les **compléter**, non les
  effacer. `CHANGELOG.md:311` (v0.14.0, historique publié) ne se touche pas. *(Validation P2, F13.)* Et `:729-731`
  (refus de la complétion : « … `AMOUNT_MISMATCH` (le total TTC des lignes doit égaler exactement le montant du QR) ») :
  le nouveau code s'y ajoute, **avant** `AMOUNT_MISMATCH`, avec le refus de positivité.
- **AC-18** — `CHANGELOG.md` : *(F9)* la section `## [0.17.0]` n'existe pas à `c5101b07` (elle est créée par la
  v016-2). Dans cette branche, l'entrée s'écrit sous un en-tête **provisoire** `## [0.17.0] — Non publié` placé en tête
  du journal, section **Corrigé**, une entrée (`refs` [#687] par un lien, sans mot-clé de fermeture) : le refus, et ce
  qu'il advient des factures existantes (non modifiées, signalées). L'orchestrateur fusionne les en-têtes à la réunion.
  Aucune version touchée.

## Tâches

1. **Étape 0** — recontrôler le § *À recontrôler*, le noter au Change Log, ligne lue citée ; établir le cas du lot
   (AC-11). Bases `kesh_v0173b`, `kesh_e2e_v0173b`.
2. **Tests rouges d'abord**, commités rouges.
3. `kesh-core` (AC-1) ; `kesh-db` (AC-3, AC-4) ; `kesh-api` (AC-2, AC-3, AC-5, AC-6, AC-9, AC-11, AC-19) ; recensement AC-7.
4. `frontend` (AC-10, AC-12, AC-13) ; Vitest ; E2E (AC-15) ; captures (AC-14).
5. Textes (AC-16 à AC-18), PDF régénéré sous verrou et `mem-guard` ; **greper la valeur** des phrases corrigées ;
   compléter les trois doc-comments de l'AC-17.
6. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `purchase_line_total` : `normalize()` retiré avant le contrôle d'échelle | AC-1 (`4 × 2.5000` admis) |
| M2 | `purchase_line_total` : seuil `<= 4` au lieu de `<= 2` | AC-1, AC-5 |
| M3 | `create_in_tx` : l'appel retiré | AC-4 (refus au dépôt, *corrigé à la validation P1, R3 : la première rédaction comptait sur l'invariant, que M3 ne fait pas rougir*), AC-5 |
| M4 | `create_in_tx` : la garde déplacée **après** les verrous de la seconde passe | revue (aucun test ne voit un verrou) — **survivante assumée** si non tuée, le dire |
| M5 | complétion : garde de ligne après `AMOUNT_MISMATCH` | AC-6 (code attendu) |
| M6 | `to_new_supplier_invoice` : borne de quantité retirée | AC-5 (`1.00001 × 10`) |
| M6bis | `to_new_supplier_invoice` : borne de prix retirée | AC-5 (`100000 × 0.00001`) |
| M10 | `create_in_tx` : garde de centime placée **avant** la positivité | AC-4 (un produit négatif doit sortir en refus de positivité) — *validation P2, R1 : tuée par le cas `-1 × 33.3333`* |
| M11 | affichage : `formatSupplierInvoiceTotal` au lieu de `formatExactAmount` dans l'avertissement | Vitest AC-10 (« 100.0040 ») |
| M12 | formulaire : message générique `supplier-invoices-err-lines` au lieu du message dédié | Vitest AC-12 |
| M7 | fiche : avertissement affiché aussi pour une facture annulée | Vitest AC-10 — *validation P2, C-v017-3-24 : sens inversé* : avertissement **masqué** pour une facture annulée → Vitest AC-10 (annulée) |
| M8 | formulaire : contrôle de ligne retiré de `linesInvalid` | Vitest AC-12 — *validation P2, F5 = R7 : reformulée* : prédicat de centime retiré de `submit()` → Vitest AC-12 (rien n'est envoyé) |
| M9 | `DbError` → `AppError` : `400` remplacé par `409` | AC-5 |
| M13 | complétion : `purchase_lines_form` remplacé par le seul `purchase_line_total` | AC-6 (`-1 × 33.3333` rendrait le code du centime) |
| M14 | fiche : `formatExactAmount` remplacé par le format à deux décimales dans la cellule TTC | Vitest AC-10 |

## Gates

`kesh-db` est touché : **gate complet même en cours de boucle**. Au dernier commit de code : `scripts/test-fast.sh --ci`
sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, `cargo build --workspace --all-targets`, Vitest complet,
`npm run check`, `npm run lint-i18n-ownership`, `npm run build`, suite E2E complète selon `docs/testing.md` à la
lettre. Ne déclarer que ce qui a tourné.

## Ce que les gardes ne voient pas

- **L1 — Le grand livre existant** : une facture enregistrée avant la 0.17.0 garde son total ; tant que l'utilisateur ne
  la traite pas, son règlement n'est ni proposé ni rattachable (v017-3a), et un lot pain.001 qui la contient peut
  porter une somme de contrôle fausse (#822, C-v017-3-16). *(Validation P2, R11.)* Ouverte, elle **masque** aussi la
  règle d'affectation de tout débit à 5 centimes près (`repositories/reconciliation.rs:934`, `BETWEEN ? AND ?` ; T-b9).
- **L2 — Une facture réelle sous le centime** : si un fournisseur facture réellement un montant sous le centime (cas non
  rencontré en Suisse pour un montant payable), la saisie exige de la ramener au centime — le montant payable l'est
  toujours.
- **L3 — L'API externe** : un client qui envoyait des lignes sous le centime reçoit désormais un `400` (AC-17).

## À recontrôler à l'étape 0 du développement

- `supplier_invoices.rs:337-560` (les deux passes, l'ordre des verrous, `generate_purchase_journal_lines` `:120-178`).
- `routes/supplier_invoices.rs:278-300`, `:468-…` ; `routes/imported_supplier_invoices.rs:240-290`.
- `routes/invoices.rs:483-510` et `routes/limits.rs:28-36` (la forme des refus d'échelle côté client).
- L'annulation du règlement d'une facture payée par un lot confirmé : *établie à la validation P1* (AC-11,
  `supplier_invoices.rs:1547-1551`) ; reconfirmer la ligne.
- *(Validation P2.)* Les affirmations du § `sec:facture-fournisseur-sous-centime` : contre-passation de l'écriture de
  règlement détachée (`[id]/+page.svelte:620-628`, `supplier_invoices.rs:1274`) ; motifs distincts de l'annulation de
  l'achat et du règlement (`user-manual.tex:1690-1694`, `:1724-1730`) ; réouverture d'exercice (`routes/fiscal_years.rs:11`,
  LIFO) ; détachement d'un document (`routes/imported_supplier_invoices.rs:629-637`).
- *(R11.)* Qu'un document joint à une facture annulée peut être joint à une autre (`POST
  /supplier-invoices/{id}/source-document`, `lib.rs:671`) — fonde la phrase de l'AC-10.
- `[id]/+page.svelte:187-190`, `:592-610`, `:637-650` (messages et motifs d'annulation) ; `SupplierInvoiceForm.svelte:203`,
  `:579` ; `invoices/invoice-helpers.ts:73-89`.
- `inbox_import_e2e.rs:1360-1400` (le test (b)).
- Les mappings `DbError` → `AppError` (où ajouter la variante ; exhaustivité des `match`).

## Dépendance à la v017-3a (D7, C-v017-3-20)

La 3b est livrée dans **la même release** que la 3a. *(Récrit à la validation P2, C-v017-3-24 : l'avertissement ne
prescrit plus de geste.)* La voie 7 du manuel (AC-16 : régler depuis la fiche puis **rattacher** la transaction à son
écriture) suppose le rattachement d'un débit à une écriture, que seule la 3a livre : à `c5101b07`, le manuel dit le
contraire (`user-manual.tex:1634-1639`, « Ne payez pas depuis la fiche une facture dont le débit est (ou sera) au
relevé » ; `:2409`, « un débit […] ne se rattache pas »). **Si la 3a ne devait pas être livrée**, cette voie se retire du
manuel.

**Sections du manuel communes aux deux fiches**, à réconcilier à la réunion des branches (le PDF se régénère après) :

| Section (`user-manual.tex`, `c5101b07`) | v017-3a | v017-3b |
|---|---|---|
| `sec:regler-facture-fournisseur`, `:1630-1639` (« Ne payez pas depuis la fiche… ») | récrit (le débit se rattache désormais) | porte la sous-section `sec:facture-fournisseur-sous-centime` |
| `sec:matching-fournisseur`, `:2304`, et l'encadré « Dépense déjà réglée par un autre chemin », `:2324` | récrit (rattachement, avertissement) | complète « un total à plus de deux décimales n'est jamais proposé » |
| `sec:rattacher-reglement`, `:2384-2412` (« Ce qui ne se rattache pas ») | récrit | sa ligne « règlement sous le centime » renvoie à `sec:facture-fournisseur-sous-centime` |
| Diagnostic d'un débit sans candidate, `:3325-3340` | récrit (`:3337-3338`) | renvoi à `sec:facture-fournisseur-sous-centime` (`:3329`) |

## Collision avec la story parallèle v016-2

Aucun fichier métier commun. Fusion mécanique : `errors.rs` (variante et code), les quatre catalogues i18n,
`docs/api-external.md` (tableau des codes), `CHANGELOG.md` (`## [0.17.0]` : la v016-2 crée la section, la 3b écrit sous un
en-tête provisoire — AC-18), `user-manual.tex` (sections distinctes) et
son PDF (à régénérer après fusion).

## Issues à ouvrir (demandées par la validation P1)

1. **« Facture fournisseur : ni plafond de quantité, de prix ou de total de ligne, ni multiplication contrôlée »** (R6,
   P3). Constat : `grep -nE "MAX_|limits" crates/kesh-api/src/routes/supplier_invoices.rs crates/kesh-api/src/routes/imported_supplier_invoices.rs`
   ne rend que `PAYMENT_REFERENCE_MAX_CHARS` ; la facture client applique `MAX_QUANTITY`, `MAX_UNIT_PRICE`,
   `MAX_LINE_TOTAL` (`routes/invoices.rs:45`). `line.quantity * line.unit_price` (`supplier_invoices.rs:400`, `:426`)
   panique sur dépassement de `Decimal` (aucun `checked_mul`), et un produit très grand voit son échelle réduite par
   `rust_decimal`. Défaut préexistant, hors de #687 ; `purchase_line_total` (AC-1) sera le lieu naturel du
   `checked_mul`.
2. **« Facture fournisseur : le taux de TVA est arrondi en silence par sa colonne »** (R12, P3). Constat : `vat_rate
   DECIMAL(5,2)` (`20260628000001_supplier_invoices.sql:86`) alors que `line_vat_amount` est calculée sur le taux reçu
   (un taux `8.125` donne une TVA de ligne différente de celle du taux stocké `8.13`) ; aucune borne d'échelle sur
   `vat_rate` côté fournisseur (`grep -n scale` des deux routes : rien).

*(Validation P2, R9 = F15.)* Ouvertes depuis : la première est **#825**, la seconde **#826**. `purchase_line_total` ne fait
**pas** le `checked_mul` dans cette story : il reste à #825.

## LOW non pris

Aucun : les LOW R7, R8, R9, R10, R11, F7 à F13 sont appliqués (R11 sous condition établie à l'étape 0) ; R6 et R12 vont
aux issues ci-dessus ; F8 est appliqué (#822 citée, Q3 retirée).

## LOW non pris à la validation P2

Aucun : R6 à R12 et F7 à F16 sont appliqués ou rendus sans objet par la simplification (F7 et F16 en partie : la phrase
du document passe au manuel, la capture d'un Lecteur et de l'état « lot confirmé » disparaissent avec la matrice).
Les MEDIUM F1 à F6, R3 et R4 sont **absorbés** par la simplification : leurs constats deviennent des phrases du manuel
(AC-16, voies 3 à 6), non des branches d'écran.

## Questions au Project Lead

- **Q1 (C-v017-3-13)** — Refuser une ligne dont le montant n'est pas au centime, plutôt que de l'arrondir en silence ou
  de régler avec une ligne d'arrondi : d'accord ?
- **Q2 (C-v017-3-14, C-v017-3-24)** — Une facture existante sous le centime n'est ni modifiée ni bloquée : sa fiche porte
  un avertissement statique qui renvoie au manuel, où les voies (annuler, ressaisir, et leurs limites) sont décrites. Voulez-vous en plus une liste de ces factures (un filtre), ou l'avertissement
  suffit-il ?

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), découpée de `v017-3-rapprochement-attente` avec la v017-3a ; registre
  C-v017-3-1, 13 à 17 ; prompt de validation P1 (`v017-3b-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q3 à
  Guy, puis validation P1 (Sonnet, lentilles R et F).
- 2026-10-30 — **Validation P1** (Sonnet, lentilles R et F) : R 0/0/5/7, F 0/0/6/7 — 0 CRITICAL, 0 HIGH, 11 MEDIUM,
  14 LOW. Recoupements : R2 = F2, R1 ≈ F5, R10 = F3. Chaque prémisse vérifiée au code avant application ; **aucun
  finding rejeté**. Remédiation : matrice des états et phrase « si un bouton manque » (R1, F5) ; lot confirmé établi
  (`supplier_invoices.rs:1547-1551`) ; `formatExactAmount` et un seul `hasSubCentime` (R2, F2, C-v017-3-18) ; refus au
  dépôt testé, M3 corrigée (R3) ; AC-19 restauration d'une sauvegarde portant un TTC sous le centime (R4) ; borne de
  prix testée, M6bis (R5) ; prémisse du TTC corrigée et règle par ligne motivée (R7) ; citation `:811-834` (R8) ;
  ordres de refus et conversion unique (R9) ; dépendance à la v017-3a et sections communes du manuel (R10, F3, D7,
  C-v017-3-20) ; document d'une pièce importée (R11) ; patron du formulaire gardé, message dédié au clic (F1,
  C-v017-3-19) ; `api-external.md:602` et trois doc-comments (F4) ; montage de la fixture (F6) ; spec E2E (F7) ; #822
  citée, Q3 retirée (F8) ; en-tête CHANGELOG provisoire (F9) ; G9 et `CLES_RELEVEES` (F10) ; détails du miroir (F11) ;
  captures (F12) ; trois sites du manuel et contrôle aux deux apostrophes (F13) ; mutations M10 à M12 ajoutées. Issues
  à ouvrir : R6, R12. Prochaine étape : validation P2 (`v017-3b-validate-prompt-p2.md`, Opus, lentilles R et F).
- 2026-10-30 — **Validation P2 et simplification** (Opus, lentilles R et F) : R 0/0/5/7, F 0/0/6/10 — 0 CRITICAL,
  0 HIGH, 11 MEDIUM, 17 LOW. La sévérité stagne (11 → 11 MEDIUM) et la plupart des défauts portent sur la matrice de
  conduite par état, certains nés d'elle (F1 à F6, R3, R4) : recyclage, déclaré à Guy par l'orchestrateur. **Décision
  C-v017-3-24 (orchestrateur) : la matrice est retirée** — un avertissement unique et statique sur la fiche, quel que soit
  l'état, annulée comprise ; une section du manuel à `\label` (`sec:facture-fournisseur-sous-centime`) décrit en prose
  les voies et leurs limites (R3, R4, F1 à F4, F7 y deviennent des phrases). Prémisses vérifiées au code ; aucun finding
  rejeté. MEDIUM restants : cas négatif `-1 × 33.3333` qui tue M10 (R1) ; passe de forme extraite en une fonction, même
  ordre à la saisie et à la complétion, M13 (R2, D9) ; registre annoté (R5, F14). F5 (message au clic) tranché : prédicat
  distinct, lu après `linesInvalid`, ordre écrit, M8 reformulée. LOW appliqués : citations `:140-145` et `:165` (R6, F9) ;
  assertion de `VALIDATION_ERROR` par fragment (R8) ; #825 et #826 citées, `checked_mul` laissé à #825 (R9, F15) ; bornes
  avant le `match` (R10) ; masquage de la règle en L1 (R11) ; cas (a) suffisant (R12) ; contrat de `hasSubCentime` (F10) ;
  quantité vide lue `1` (F11) ; label nommé (F12) ; `api-external.md:729-731` (F13) ; quantité de l'E2E (F16) ; clés
  nommées et méthode de recompte de `CLES_RELEVEES` corrigée (F8). Prochaine étape : validation P3 complète
  (`v017-3b-validate-prompt-p3.md`, Sonnet, lentilles R et F).
