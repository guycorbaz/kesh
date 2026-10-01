# Story 25.4-d2a : Solder le reste — l'écriture et son annulation

Status: done

**Issues : [#384], [#490]** — ⛔ la PR porte `refs #384, refs #490` : la **25-4-d2b** (le bouton) fermera #490, la
**25-4-d2c** (le rapport TVA) fermera #384.

**Première des trois stories de la 25-4-d2.** Empilée sur la 25-4-d1 (branche `story/25-4-d2a-ecriture-de-solde`).

## Arbitrages (Project Lead)

Rendus le 2026-10-01 (25-4-d, cf. `25-4-d1-comptes-de-solde.md`) :

1. Quatre natures : **escompte accordé**, **frais bancaires**, **perte sur débiteur**, **reste d'arrondi** (#490).
2. L'utilisateur **choisit la nature** ; **aucun seuil**.
3. Un compte par nature, réglé dans les paramètres (posé par la 25-4-d1 ; le reste d'arrondi utilise le compte de
   différences d'arrondi).
4. **TVA corrigée au prorata des taux** de la facture pour l'escompte et la perte ; pas pour les frais ni l'arrondi.
5. Un bouton « Solder le reste » sur la fiche, **annulable par contre-passation**.
6. Toute facture **validée non soldée** peut être soldée, réglée en partie ou pas du tout.
7. Le rapport TVA retranche la TVA des soldes.

**Retenus par défaut le 2026-10-01** (Guy : « pousse puis continue », sans trancher ; révisables) :

- **Découpage de la d2 en trois** : d2a (cette story — l'écriture, l'annulation, côté serveur), d2b (le bouton, le
  dialogue, la liste, le manuel utilisateur), d2c (le rapport TVA). La d2 entière touchait six modules de code.
- **Le solde est une ligne de `invoice_settlements`**, de type `write_off` : le reste dû, l'échéancier, la balance
  âgée, le rapprochement, `paid_at` et l'annulation existante le suivent sans modification. Comme un règlement, il
  bloque l'avoir et la dévalidation tant qu'il n'est pas annulé.
- **Rapport TVA** (d2c) : retrancher la TVA des soldes, par taux, dans la période du solde ; #390 reste à part. D'où,
  ici, la **ventilation par taux figée** sur la ligne de solde.
- Le solde porte sur le **reste exact** (brut, quatre décimales) ; la créance tombe à zéro.

## Story

En tant que comptable,
je veux solder ce qui reste dû sur une facture en choisissant la nature de l'écart,
afin que la créance se ferme et que l'écart — et la TVA qu'il corrige — soit passé au bon compte.

## Les faits, vérifiés dans le code (inventaire du 2026-10-01)

- **`invoice_settlements`** (`20260827000001`, `20260828000001`) : `settlement_type VARCHAR(20)`,
  `settlement_bank_account_id`, `settlement_account_id` ; `chk_invoice_settlements_type` (`bank_transfer`,
  `internal_account`) et `chk_invoice_settlements_counterparty` (une référence par mode) ; `amount > 0` ;
  `UNIQUE (journal_entry_id)`. Le reste dû retranche **toute** ligne de la table (`INVOICE_AMOUNT_DUE_DERIVED_SQL`,
  `invoice_settlements.rs:119`), quel que soit son type.
- **`settle_invoice`** (`invoice_settlements_write.rs:45-323`) est le patron : verrou `FOR UPDATE` de la facture,
  statut `validated`, borne `settled_on >= date − 1 jour`, créance = **première ligne de débit** de l'écriture de vente,
  exercice ouvert couvrant la date, `journal_entries::create_in_tx` (verrou de période, exercice clos, comptes),
  `invoice_settlements::create_in_tx`, `paid_at` + **`version + 1`** (invariant écrit à `:251-262`), audit dans la
  transaction.
- **`cancel_settlement_in_tx`** (`:412-560`) contre-passe **toutes** les lignes de l'écriture, supprime la ligne de
  règlement, remet `paid_at` à `NULL` si un reste réapparaît, `version + 1`, audit. Il ne regarde pas le type.
- **`SettlementChoice`** est partagé avec le fournisseur (`entities/supplier_invoice.rs:104-110`) : ⛔ ne pas y ajouter
  de variante — le solde a son propre chemin.
- **Ventilation de TVA** : `vat_breakdown_by_rate` (`kesh-core/src/accounting/vat.rs:123-153`), TVA arrondie par ligne,
  taux > 0 seulement. **Aucune fonction de prorata.** L'écriture de vente crédite la TVA due par taux sur
  `default_vat_payable_account_id` ; l'avoir la débite de même (`credit_notes.rs:187-257`) — précédent direct.
- **Comptes de nature** : seuls `rounding_account_for_write` (`company_invoice_settings.rs:400-427`) relit un compte
  désigné au moment d'écrire (actif, imputable, charge ou produit, `FOR UPDATE`).
- **Lignes d'écriture** en `DECIMAL(19,4)` : une écriture peut créditer la créance au reste exact (la c3-b le fait).
- **#490** : un reste brut inférieur au demi-centime (`0.0040`, p. ex. une facture de `10.0040` réglée de `10.00`) est
  **insoldable** —
  tout paiement positif est un trop-perçu.

## Acceptance Criteria

**AC 1 — La migration** (`20261001000004_invoice_settlements_write_off.sql`).
- `ADD COLUMN write_off_nature VARCHAR(20) NULL`, `ADD COLUMN write_off_vat JSON NULL` (la ventilation figée, AC 4).
- `chk_invoice_settlements_type` recréée avec `'write_off'` ; `chk_invoice_settlements_counterparty` recréée avec une
  troisième branche : `write_off` ⇒ `settlement_account_id IS NOT NULL AND settlement_bank_account_id IS NULL`. Chaque
  `DROP CONSTRAINT` et chaque `ADD CONSTRAINT` en **instruction `ALTER TABLE` distincte**, comme le seul précédent du
  dépôt (`20260714000002_email_templates_reminder.sql`).
- Nouvelle `chk_invoice_settlements_write_off_nature` : `(settlement_type = 'write_off') = (write_off_nature IS NOT
  NULL)`, `(settlement_type = 'write_off') = (write_off_vat IS NOT NULL)` et `write_off_nature IN ('discount',
  'bank_fees', 'bad_debt', 'rounding')`. ⚠️ L'alias `JSON` de MariaDB ajoute un `CHECK (JSON_VALID(write_off_vat))`
  implicite : le squash régénéré doit le porter (`test_schema_guard` le contrôle).
- **DDL seul** (P7 sans objet) ; **non breaking** — confirmé en validation (P1, P2) : `settlement_type` est lu en
  `String` partout, `check_schema_compat` n'exige que les colonnes `NOT NULL` sans défaut. À écrire dans l'en-tête (un binaire antérieur lit
  `settlement_type` en chaîne, ignore les deux colonnes, contre-passe un solde comme un règlement ; `DROP CONSTRAINT`
  n'est pas une opération listée en P3) — la passe de revue le confirme. P5 (audit, 74), P6 (`migrations_upgrade_path.rs`
  73 → 74, 39 → 40), squash, `migrations.sha384`, export CSV des deux colonnes, sauvegarde antérieure → `NULL`.

**AC 2 — Le prorata** (`kesh-core/src/accounting/vat.rs`). `write_off_vat_shares(lines, total_ttc, amount) ->
Vec<VatRateShare { rate_percent, base_ht, vat_amount }>` : pour chaque taux > 0 de `vat_breakdown_by_rate(lines)`,
`vat_amount = Money::round_to_centimes(amount × vat_r / total_ttc)` et `base_ht = Money::round_to_centimes(amount × base_r / total_ttc)`, où
`Money::round_to_centimes` est la fonction existante (`money.rs:66`, MidpointAwayFromZero — pas de helper neuf), et
`total_ttc` est le TTC **figé** de la facture, calculé par `invoice_total_ttc_rounded` (`vat.rs:93-98`, lignes +
`rounding_amount` — ne pas le recalculer). ⚠️ `base_ht` est **informatif** (il servira au rapport TVA, d2c) : l'écriture
n'en dépend pas, et `A − Σ vat_amount` ne vaut pas `Σ base_ht` au centime près par construction — ne pas l'asserter. Note pour la d2c : `base_ht` ne couvre que les taux > 0 ; le chiffre 235 du décompte AFC
(diminutions de contre-prestation) porte toute la réduction, part à 0 % comprise — la d2c la déduira de `amount`. Tests : un taux, plusieurs taux, lignes à 0 %
(aucune part), arrondi figé (la part sans TVA ne porte rien), `amount == total_ttc` (la TVA corrigée égale la TVA
facturée), montant à quatre décimales, `amount` minuscule (parts nulles omises).

**AC 3 — Le compte au moment d'écrire.** `write_off_account_for_write(conn, company_id, nature)` généralise
`rounding_account_for_write` (**un seul code**, le compte d'arrondi en devient un cas) : la colonne de la nature
(`rounding` → `default_rounding_account_id`), actif, imputable, charge ou produit, `FOR UPDATE`. La colonne se choisit
par un `match` qui rend un **littéral SQL complet** — jamais interpolé (précédent : `role_check`,
`journal_entries.rs:1546-1560`). Un cœur commun rend `Option<i64>` ; chaque enveloppe pose **sa** erreur —
`rounding_account_for_write` garde `RoundingAccountNotConfigured`, sur lequel `routes/reconciliation.rs:1495` fait un
`match`. Absent ou invalide →
`DbError::WriteOffAccountNotConfigured { nature }` → **400 `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`**, message qui nomme la
nature et renvoie à *Paramètres → Facturation* (4 locales). `rounding_account_for_write` garde son erreur et ses
messages.

**AC 4 — L'écriture de solde.** `write_off_invoice(pool, user_id, company_id, invoice_id, nature, settled_on) ->
WriteOffOutcome { journal_entry_id, amount }`, sur le patron de `settle_invoice` :
1. verrou de la facture `FOR UPDATE`, statut `validated` (une facture annulée par avoir est `cancelled` : refusée),
   **`paid_at IS NULL`** — sinon `InvalidInput("invoiceAlreadyPaid")` : une facture réglée avant l'existence de
   `invoice_settlements` a `paid_at` posé **sans aucune ligne** (`invoices.rs:1490-1505`) et serait soldée pour tout son
   TTC ; l'avoir pose déjà ce garde (`credit_notes.rs:325`). *(`settle_invoice` ne l'a pas non plus : hors périmètre,
   à signaler.)* ; **`version` attendue** — la facture verrouillée doit porter la `version` du corps, sinon
   `OptimisticLockConflict` → 409 ; borne de date ;
2. reste brut `A = amount_due` ; `A <= 0` → refus `InvalidInput("nothingToWriteOff")` ;
3. nature `rounding` : seulement si `A < 0.05` (un reste d'arrondi ne dépasse pas l'unité de 5 centimes), sinon
   `InvalidInput("writeOffRoundingTooLarge")` ;
4. compte de la nature (AC 3) ; créance = première ligne de débit de l'écriture de vente ;
5. escompte et perte : parts de TVA (AC 2) ; garde `Σ vat_amount < A`, sinon `DbError::Invariant` (le débit du compte de
   la nature doit rester strictement positif — `chk_jel_debit_credit_exclusive`). Inatteignable par le prorata (P2 :
   une part n'est non nulle que pour `A ≳ 0.06`, la somme n'atteint `A` que pour `A ≲ 0.017`) : la garde vit dans une
   **fonction pure** qui bâtit les lignes de l'écriture à partir de `(A, parts)`, et c'est elle que le test exerce avec des
   parts injectées ; si une part est non nulle, compte `default_vat_payable_account_id`
   **courant** (comme l'avoir), absent → `ConfigurationRequired` ;
6. exercice ouvert couvrant `settled_on` ; écriture au journal **OD**, libellé « Solde facture {numéro} — {intitulé} », l'intitulé
   en **français figé** comme le libellé des règlements (`invoice_settlements_write.rs:219`) — « escompte accordé »,
   « frais bancaires », « perte sur débiteur », « reste d'arrondi » — par une fonction nommée de la nature,
   `project_id` de la facture, lignes : **débit** du compte de la nature `A − Σ TVA`, **débit** de la TVA due par taux,
   **crédit** de la créance `A` ;
7. ligne `invoice_settlements`, insérée par **le même** `invoice_settlements::create_in_tx` (`invoice_settlements.rs:246-281`) :
   `NewInvoiceSettlement` porte désormais un `kind: SettlementKind { Choice(SettlementChoice), WriteOff { nature,
   account_id, vat } }` qui fixe type, références, `write_off_nature` et `write_off_vat` — le second appelant
   (`routes/reconciliation.rs:1595`) passe `Choice` ; les deux colonnes rejoignent `COLUMNS` et l'entité
   `InvoiceSettlement` (`entities/invoice_settlement.rs:22-41`). La ligne : `settlement_type = 'write_off'`, `settlement_account_id` = compte de la nature,
   `amount = A`, `write_off_nature`, `write_off_vat` = les parts (JSON `[{ratePercent, baseHt, vatAmount}]`, `[]` sans
   TVA) ;
8. `paid_at = settled_on`, **`version + 1`**. ⚠️ **Invariant : tant qu'un solde existe, la facture est payée.** Le solde
   éteint toujours la totalité du reste, **et aucun autre règlement de la facture ne peut être annulé tant qu'il existe**
   (AC 6). C'est ce qui garde un solde hors du « déjà réglé » d'un rappel (un rappel est refusé sur une facture payée,
   `dunning_reminders.rs:292`, `dunning_eligibility.rs:87`) et hors du statut « partiellement payée » de l'export CSV
   (`routes/invoices.rs:1528`, `paid_at` lu d'abord) ;
9. audit `invoice.written_off` (nature, montant, TVA corrigée, écriture), libellé dans `audit_labels.rs` (4 locales).

**AC 5 — La route.** `POST /api/v1/invoices/{id}/write-off` `{ nature, settledOn, version }` — rôle Comptable+, **sans
montant** (le serveur solde le reste exact) ; la **`version`** remplace le garde que le montant donne au règlement
manuel (le refus du trop-perçu, `routes/invoices.rs:1170-1175`) : un écran périmé ou une tentative rejouée qui relit un
reste changé est refusé en **409** (patron `UnvalidateInvoiceRequest`, `routes/invoices.rs:885-886`). `nature` est reçue
en **`String`** et convertie à la main (une enum serde rendrait 422 — `into_parts`, `routes/invoices.rs:1189-1194`). Réponse `{ invoice, journalEntryId, amount }`. Le handler est **rejoué sur
interblocage** — sûr grâce à la `version` : une tentative rejouée qui trouve un reste changé est refusée en 409 — (`retry_with` / `is_deadlock_error`, patron `routes/reconciliation.rs:844-851`) : l'ordre des verrous
est celui de `settle_invoice` (facture → compte → exercice), qui peut former un cycle avec `accept_one_invoice` (#491 —
le règlement manuel, lui, reste non rejoué : hors périmètre). Nature inconnue → 400. Toutes les
erreurs de l'AC 4 mappées (clés i18n, 4 locales). Inscrite au registre d'audit des routes.

**AC 6 — La liste et l'annulation.** **Nouveau motif de refus d'annulation** dans `settlement_cancel_blocker_unlinking`
(la fonction qui sert la lecture **et** l'écriture, `invoice_settlements_write.rs:350-380`) : un règlement qui n'est pas
un solde **ne s'annule pas tant qu'un solde existe sur la facture** — « annulez d'abord le solde ». Sans lui, annuler un
règlement de 998 après un escompte de 2 rouvrirait la facture (`paid_at = NULL`) avec l'escompte toujours passé et
compté en « déjà réglé » (P2). Le motif couvre le dé-rapprochement, qui passe par `cancel_settlement_in_tx`
(`reconciliation_cancel.rs:352`). Le motif se place **juste après le rang 1** (`InvoiceCredited`), **avant** la queue commune de
`settlement_entry_cancel_blocker` : comme lui, c'est une propriété de la **facture**, non de l'écriture ciblée — placé
après, il ferait rouvrir un exercice clos pour rien (P3). Code, libellé serveur (4 locales). ⚠️ La liste existante
(`GET …/settlements`, `routes/invoices.rs:1351-1364`) renverra ce code **dès cette story** : le frontend le reçoit, et sa
table des motifs (`settlement-cancel-blocked.ts:31-62`, dont le `default` de `:58-62` rend le code tel quel) l'afficherait **brut**. D'où le geste minimal côté frontend :
ajouter le code à `InvoiceSettlementCancelCode` (`features/invoices/settlement-cancel.ts`) et son message (clé i18n,
4 locales) dans `invoiceSettlementCancelMessage` (partie propre aux factures client, non la queue commune
partagée avec le fournisseur) — un site `i18nMsg` de plus : `sitesTotal` (`i18n-keys.test.ts`) se recompte. Aucun autre
écran, qui reste à la d2b. `GET …/settlements` : `settlementType = "write_off"` et un champ
`writeOffNature` (`null` hors solde). L'annulation existante (`…/settlements/{id}/cancel`) contre-passe le solde,
**TVA comprise**, rouvre la facture (`paid_at = NULL`), `version + 1` — prouvé par test, sans code nouveau si
possible.

**AC 7 — Le CHANGELOG** `[0.12.1]` *Added* : le solde du reste par l'API (`POST …/write-off`), l'écran suit (d2b).
Le manuel utilisateur vient avec la d2b.

**AC 8 — Tests.** Chacun aurait échoué avant le patch :
- prorata (AC 2) ;
- une facture **non réglée** et une facture **réglée en partie** soldées, pour chaque nature : écriture équilibrée,
  comptes et montants, TVA par taux (escompte, perte), aucune TVA (frais, arrondi), reste dû nul, `paid_at`, `version`,
  audit ;
- **#490** : un reste de `0.0040` (facture de `10.0040`, règlement partiel de `10.00` inséré au montage) soldé en nature
  `rounding`, créance à zéro ;
- **le nouveau motif** : un règlement ne s'annule pas tant qu'un solde existe (manuel et dé-rapprochement), puis
  s'annule une fois le solde annulé ; il précède un exercice clos (rang) ; Vitest : le message s'affiche, pas le code ;
- **le sens inverse** : une proposition de rapprochement visant une facture déjà soldée est refusée (la facture sort des
  candidats, `reconciliation.rs:129-134` ; une proposition en vol est refusée par la `version`) ;
- refus : brouillon, facture annulée par avoir, facture soldée, **`paid_at` posé sans ligne de règlement**, `version`
  périmée (409), compte non configuré ou archivé, arrondi ≥ 0.05, date avant la facture, exercice
  clos, période verrouillée, compte TVA absent ;
- annulation : reste rétabli, TVA contre-passée, `paid_at = NULL` ; l'avoir et la dévalidation sont refusés tant que
  le solde existe ;
- API : rôle (Consultation refusée), nature inconnue, liste avec `writeOffNature` ;
- l'invariant de l'AC 4 point 8 : après un solde d'une facture réglée en partie, un rappel est refusé, l'export CSV
  donne « payée », et l'annulation du règlement antérieur est refusée ;
- sauvegarde antérieure sans les colonnes.

## Tasks / Subtasks

- [x] **T1 — migration** (AC 1), dont la liste de colonnes **écrite à la main** de `serialize_invoice_settlements_csv`
  (`exports/csv_tables.rs:1332-1368`) et son test d'en-tête.
- [x] **T2 — prorata** (AC 2).
- [x] **T3 — compte au moment d'écrire** (AC 3).
- [x] **T4 — écriture de solde** (AC 4).
- [x] **T5 — route, liste, motif d'annulation** (AC 5, AC 6), dont le code du motif côté frontend.
- [x] **T6 — CHANGELOG** (AC 7).
- [x] **T7 — tests** (AC 8).
- [x] **T8 — gates** : backend complet (migration), frontend (non touché : `check` et `test:unit` pour la forme),
  **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ Ajouter une variante à `SettlementChoice` : partagé avec le fournisseur, dont le CHECK n'admet que deux valeurs.
- ⛔ Créer une table pour la ventilation : une table neuve rend **toutes** les sauvegardes antérieures inimportables
  (l'inventaire des tables doit être identique) — d'où la colonne JSON.
- ⛔ Recopier `rounding_account_for_write` : le généraliser.
- ⛔ Un numéro de compte dans le code.
- ⛔ Toucher le frontend, **hormis** le code du nouveau motif et son message (AC 6) : la d2b. (Conséquences assumées
  jusqu'à la d2b : l'écran affiche un solde fait par l'API comme
  « Espèces ou autre compte », et la fiche compte le solde dans « Déjà réglé » — la d2b doit les distinguer.)

### Modules

`kesh-core` (prorata), `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` (le seul code de motif, AC 6) (+ `CHANGELOG`) —
cinq modules de code, au seuil sans le dépasser.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- **Écart à la fiche, assumé** : l'AC 6 ne citait que la table des motifs de la fiche facture. Le **dé-rapprochement**
  lit le même motif (`settlement_cancel_blocker_unlinking`, via `GET /reconciliation/transactions/{id}` et le refus au
  clic) : son écran l'aurait affiché brut aussi. Le code et son message ont donc été ajoutés aux **deux** tables
  (`features/invoices/settlement-cancel.ts` et `features/reconciliation/reconciliation-cancel.ts`), toujours dans le
  module `frontend` — `sitesTotal` 1775 → 1777.
- **Hors AC, par la règle de synchronisation des docs** : `docs/api-external.md` documente la route, le type
  `write_off`, `writeOffNature` et le motif `INVOICE_WRITTEN_OFF`.

### Completion Notes List

- **T1** — `20261001000004_invoice_settlements_write_off.sql` : `write_off_nature`, `write_off_vat JSON`, les deux CHECK
  recréés (une instruction par `DROP`/`ADD`), `chk_invoice_settlements_write_off_nature` (nature **et** ventilation ssi
  solde). Audit à 74 (66 `tracked-by-sqlx` + 8 `yes`, recomptés), `migrations_upgrade_path.rs` 73 → 74 et 39 → 40,
  frontière 34 (résidus grepés), squash régénéré — il porte le `CHECK (json_valid(...))` implicite —, sha384, export CSV.
- **T2** — `write_off_vat_shares` (`kesh-core/accounting/vat.rs`) et `VatRateShare`, sur `vat_breakdown_by_rate`, arrondis
  par `Money::round_to_centimes`, `total_ttc` = `invoice_total_ttc_rounded`.
- **T3** — `usable_designated_account` (cœur commun, `Option`) ; `rounding_account_for_write` garde son erreur ;
  `write_off_account_for_write` choisit la colonne par un `match` de littéraux SQL complets ;
  `DbError::WriteOffAccountNotConfigured { nature }` → 400 `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`, un message par nature.
- **T4** — `write_off_invoice` : gardes (statut, `paid_at`, `version`, date), reste exact, seuil `rounding` < 0.05, compte
  de la nature, créance, parts de TVA et compte de TVA due courant, exercice, écriture OD (« Solde facture … — intitulé »,
  français figé, `WriteOffNature::entry_label`), ligne par `create_in_tx` (`SettlementKind`), `paid_at`, `version + 1`,
  audit `invoice.written_off` (libellé, 4 locales). Fonction pure `write_off_journal_lines` avec la garde `Σ TVA < A`.
- **T5** — `POST /api/v1/invoices/{id}/write-off` (`nature` en `String`, `version`, rejeu `retry_with` sur 1213), inscrite
  au registre d'audit des routes (110 / 92 / 113) ; liste : `writeOffNature`. Motif `WriteOffExists`
  (`INVOICE_WRITTEN_OFF`) au **rang 1 bis**, refusé par l'annulation elle-même (donc aussi par le dé-rapprochement) ;
  messages serveur des deux familles (4 locales) et tables frontend (cf. Debug Log).
- **T6** — CHANGELOG `[0.12.1]` *Added* (#384, #490) ; `docs/api-external.md`.
- **T7** — périmètre `f6eff8d3` → arbre de travail, recompté aux deux bornes : **18 tests Rust neufs** (5 `vat.rs`
  19 → 24 ; 10 `invoice_write_off.rs` 0 → 10 ; 2 `invoice_echeancier_e2e.rs` 28 → 30 ; 1 `admin_full_import_e2e.rs`
  32 → 33), **3 Vitest neufs** (`settlement-cancel.test.ts` 0 → 2, `reconciliation-cancel.test.ts` 4 → 5) dont un cas
  ajouté à une table existante. **Quatre mutations tuées** et restaurées (fichier touché ensuite) : motif supprimé → 2
  rouges ; garde `paid_at` retirée → 1 ; TVA non corrigée → 2 ; contrôle de `version` retiré → 1.
- **Gates** — base remise à zéro ; `scripts/test-fast.sh` **vert, 2607/2607** ; frontend `check` (0 erreur, 27
  avertissements préexistants), `lint-i18n-ownership`, `test:unit` **856/856**, `build`.
- **E2E complet** — base `kesh_e2e` reconstruite, montage complet (`smtpConfigured:true`), run à 21:44 UTC : **232
  passés, 7 échecs, 19 ignorés** — exactement les sept **KF-029** de la liste nominative, aucune pollution.

### File List

- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `CHANGELOG.md`
- `crates/kesh-api/src/audit_labels.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/routes/invoices.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs`
- `crates/kesh-core/src/accounting/vat.rs`
- `crates/kesh-db/migrations/20261001000004_invoice_settlements_write_off.sql` (nouveau)
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/src/entities/invoice_settlement.rs`
- `crates/kesh-db/src/entities/mod.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/src/repositories/invoice_settlements.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/test-schema/0001_schema_squash.sql`
- `crates/kesh-db/tests/invoice_settlement.rs`
- `crates/kesh-db/tests/invoice_write_off.rs` (nouveau)
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `docs/api-external.md`
- `docs/migrations-idempotence-audit.md`
- `frontend/src/lib/features/invoices/settlement-cancel.ts`
- `frontend/src/lib/features/invoices/settlement-cancel.test.ts` (nouveau)
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts`
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.test.ts`
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`

## Change Log

- **2026-10-01** — Créée après l'inventaire ; d2 découpée en d2a / d2b / d2c ; recommandations retenues par défaut
  (Guy : « pousse puis continue »).
- **2026-10-01** — Validation P1 (Sonnet) : 2 HIGH, 4 MED, 2 LOW. **H1 reclassé LOW** après vérification : le solde
  apparaîtrait comme « déjà réglé » sur un rappel et « partiellement payée » à l'export — inatteignable, un solde éteint
  toujours le reste et pose `paid_at`, qu'un rappel refuse (`dunning_reminders.rs:292`) et que l'export lit d'abord ;
  l'invariant est désormais écrit (AC 4) et testé (AC 8), et le « Déjà réglé » de la fiche renvoyé à la d2b. Retenus :
  rejeu sur interblocage de la route (H2, #491) ; CHECK recréés en instructions distinctes (M1) ; garde `Σ TVA < A` (M2) ;
  intitulé de nature nommé, français figé (M3) ; liste CSV en tâche explicite (M4) ; `invoice_total_ttc_rounded` réutilisé
  et `base_ht` dit informatif (L1, L2).
- **2026-10-01** — Validation P2 (Opus) : 3 HIGH, 1 MED, 9 LOW, tous retenus. **Le reclassement du H1 de P1 est
  réfuté** : l'invariant « un solde éteint le reste » n'était vérifié qu'à l'instant du solde — annuler ensuite un
  règlement antérieur rouvrait la facture, escompte toujours passé. D'où un motif de refus d'annulation (AC 6).
  Retenus aussi : `version` dans la requête (H2 — sans montant, rien ne gardait d'un écran périmé ni d'un rejeu) ; refus
  d'une facture payée sans ligne de règlement (H3) ; `SettlementKind` dans `create_in_tx` (M1) ; et les LOW (garde de TVA
  en fonction pure, exemple #490 reconstruit, `nature` en `String`, `Money::round_to_centimes`, CHECK de `write_off_vat`,
  phrase cassée par le patch de P1, choix de colonne par littéral, note 235 pour la d2c, refus « avoir » au test).
  ⚠️ Signal de découpage (HIGH → HIGH) : **non découpée** — quatre modules, défauts distincts et d'origine ; signalé à Guy.
- **2026-10-01** — Validation P3 (Sonnet) : 3 MED, retenus. Le nouveau motif s'afficherait **brut** à l'écran dès cette
  story (la liste existante le renvoie) → code et message côté frontend, la story passe à cinq modules ; son **rang**
  précisé (après « avoir », avant la queue de l'écriture) ; test du **sens inverse** (rapprochement d'une facture soldée
  refusé). Sévérité : HIGH → HIGH → MED.
- **2026-10-01** — Validation P4 ciblée (Haiku, sur la seule remédiation de P3) : 2 MED **reclassés LOW** — deux plages
  de lignes imprécises, sens juste (`settlement-cancel-blocked.ts`, `reconciliation.rs`) — corrigées. Axe déclaré non
  répondu (le site `i18nMsg` du motif) repris par l'orchestrateur : `sitesTotal` à recompter, écrit à l'AC 6.
  **Boucle close** : P1 2H/4M/2L → P2 3H/1M/9L → P3 3M → P4 0 > LOW ; Sonnet → Opus → Sonnet → Haiku ; remédiation de
  P4 sur la fiche seule.
- **2026-10-01** — Implémentée (T1–T8). 18 tests Rust et 3 Vitest neufs, 4 mutations tuées. Gates : backend 2607/2607,
  frontend 856/856, E2E 232/7 (KF-029). Écart assumé : le motif traduit aussi dans la table du dé-rapprochement.
  Statut → review.
- **2026-10-02** — Revue de code P1 (Sonnet, lentilles A, B, C ; prompt versionné) : 1 CRIT, 6 MED, 4 LOW avant triage.
  **B1, reclassé HIGH** (de CRITICAL) : un reste exact à quatre décimales (10.0050) laissait sa fraction de centime au
  compte de la nature — corrigé, elle va au compte de différences d'arrondi, au débit ou au crédit, comme pour le
  règlement au centime (la créance reste créditée du reste exact, convention de la c3-b) ; la nature `rounding` la garde
  sur son compte. HIGH et non CRITICAL : aucun total n'était faux. ⚠️ **Mon propre patch de B1 cassait la nature
  `rounding`** (garde fausse pour 0.0040, ligne à zéro) : vu à la relecture avant tout test, réécrit. **A-M1 = B2** : le
  compte de TVA due est relu actif, imputable, `FOR UPDATE` (`vat_payable_account_for_write`) ; absent ou archivé →
  `ConfigurationRequired`. **A-M2** : commentaire du seuil de 0.05 aligné sur le code (« à partir de »). **B3** : test du
  dé-rapprochement refusé par le solde (lecture et geste, lien rétabli). **B4** : test par la base, plusieurs taux,
  ligne à 0 %, arrondi figé négatif, reste partiel. **C-M1** : en-tête de `audit_route_registry.rs` (109/112 → 110/113).
  Laissés LOW : A-L1 (pas de bouton, voulu — d2b), B5 (compte de nature égal au compte de TVA, mauvaise configuration
  sans perte d'exactitude), B6 (le backfill `20260828000001` ne filtre pas `write_off` — migration appliquée,
  immuable (P8), cas de configuration aberrant qui échouerait bruyamment sur le CHECK), C-L1 (type TS, conséquence
  assumée jusqu'à la d2b). **5 tests neufs** (périmètre `33b4f201` → remédiation : `invoice_write_off.rs` 10 → 15),
  mutation de B1 tuée. Gate complet, base remise à zéro : **2612/2612**.
- **2026-10-02** — Revue de code P2 ciblée (Opus, sur la remédiation `b8663cc1`) : 1 MED, 4 LOW. **M1 — régression de mon
  correctif de B1** : un reste inférieur au demi-centime (0.0040) soldé en escompte, frais ou perte s'arrondissait à
  0.00 au centime, laissant un débit de nature nul → `Invariant` → **500** ; c'est le cas même de #490, qui passait avant
  le correctif. Corrigé : quand l'arrondi vaut zéro, tout le reste va au compte d'arrondi, sans ligne de nature (tests
  par la base, trois natures, et sur la fonction pure ; mutation de l'ancienne garde tuée par les deux). **L1** : le
  correctif de B1 verrouillait le compte de TVA avant le compte d'arrondi, à l'inverse de la validation d'une facture
  arrondie — ordre rétabli (arrondi, puis TVA). **L3** : cas « compte de TVA actif mais non imputable » ajouté ;
  assertion du test de dé-rapprochement resserrée sur `SettlementNotCancellable` (le dé-rapprochement laisse le rang 1
  au geste du règlement). Laissés LOW : L2 (le test à taux mêlés ne distingue pas TTC figé et brut — couvert par
  `vat.rs`), L4 (un reste de 0.0050 en frais impute 0.01 de charge et crédite 0.0050 d'arrondi : équilibré, sans
  perte). Tests : `invoice_write_off.rs` 15 → 17. Gate complet, base remise à zéro : **2614/2614**.
- **2026-10-02** — Revue de code P3 ciblée (Sonnet, sur la remédiation `8a2537d9`) : 1 MED, 3 LOW. **M-1 reclassé LOW** :
  un reste sous le demi-centime soldé en « frais bancaires » donne une écriture libellée « … — frais bancaires » qui ne
  touche que le compte d'arrondi — le libellé désigne le **geste**, comme « Règlement facture » quand la troisième ligne
  est un écart d'arrondi ; montant inférieur au demi-centime, audit exact. **L-1** : le commentaire justifiant l'ordre
  des verrous invoquait une symétrie absente (aucun autre chemin ne verrouille le compte de TVA) — corrigé. **L-2** : la
  seconde branche de la garde est inatteignable (`total_vat == amount`, refusé plus haut) — commentaire corrigé, garde
  laissée par défense. **L-3** : quand TVA et arrondi manquent tous deux, l'erreur affichée a changé (arrondi d'abord) —
  sans conséquence, consigné. Remédiation **en commentaires seulement**, aucune ligne exécutable : la boucle se clôt
  (`CLAUDE.md`, passe ciblée). **Boucle close** : P1 1C/6M/4L (Sonnet ×3) → P2 1M/4L (Opus) → P3 0 > LOW (Sonnet).
  Gate complet au dernier commit, base remise à zéro : **2614/2614** ; frontend non touché par la boucle (856/856 à
  `33b4f201`) ; E2E au commit d'implémentation (232/7, KF-029), la boucle n'ayant modifié que le chemin du solde, qu'aucun
  écran n'appelle encore. Statut → done.

[#384]: https://github.com/guycorbaz/kesh/issues/384
[#490]: https://github.com/guycorbaz/kesh/issues/490
