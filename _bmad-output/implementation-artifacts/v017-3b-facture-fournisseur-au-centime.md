# Story v017-3b : une facture fournisseur naît au centime ; celle qui ne l'est pas est signalée, non modifiée

Status: ready-for-dev

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0173`, branche
`story/v017-3-rapprochement-attente` partie de `c5101b07` (tête de la candidate `v0.16.1-rc1`) ; la story se fusionne
dans la release **0.17.0**. Sous-story de `v017-3-rapprochement-attente`, découpée avec **v017-3a** (#671) — motif :
C-v017-3-1. Registre : `v017-3-choix-autonomes.md` (C-v017-3-1, C-v017-3-13 à C-v017-3-17). Clé courte : `v0173b`.
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
  `repositories/reconciliation.rs:796-812`) ; **#476** (côté client, l'écart sous le centime se traite au règlement) ;
  **v017-3a** (le rattachement à une écriture exige un montant égal : même conséquence).

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Les prémisses de #687 — confirmées

| Prémisse | Au code | Verdict |
|---|---|---|
| `line_total = quantité × prix` n'est pas arrondi | `let line_total = line.quantity * line.unit_price;` aux **deux** passes de `create_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:400`, `:426`) — l'issue citait `:389`, `:415`, décalés depuis | **Exacte** |
| Aucune limite d'échelle à la création | `CreateSupplierInvoiceLineRequest` reçoit `quantity`, `unit_price` en `Decimal` (`routes/supplier_invoices.rs:278-285`) ; `to_new_supplier_invoice` (`:468-…`) ne contrôle aucune échelle ; `grep -n "scale" crates/kesh-api/src/routes/supplier_invoices.rs crates/kesh-api/src/routes/imported_supplier_invoices.rs` : rien. Le formulaire borne à quatre décimales (`frontend/src/lib/features/journal-entries/balance.ts:23`, `AMOUNT_RE`), le serveur non | **Exacte** |
| Le TTC (`DECIMAL(19,4)`) peut porter des fractions de centime | `total_amount DECIMAL(19,4)` (`20260628000001_supplier_invoices.sql:35`) ; `total_ttc = Σ line_total + Σ TVA` (`supplier_invoices.rs:120-178`), la TVA arrondie au centime par ligne (`kesh-core/src/accounting/vat.rs:40-44`) — le TTC est sous le centime **exactement** quand une ligne l'est | **Exacte** |
| Le règlement depuis la fiche écrit une ligne de banque au même montant | `pay_in_tx_as` lit le TTC sur la ligne créanciers de l'écriture d'achat (`purchase_payable_line`, `:671-688`) et écrit `D créanciers TTC / C banque TTC` (`:919-952`) | **Exacte** |
| La proposition ne propose jamais une telle facture | `find_open_supplier_invoices_for_window` : `si.total_amount = ?` en égalité exacte (`repositories/reconciliation.rs:811-834`) ; test T-b9 `a_supplier_ttc_under_the_centime_is_masking_not_candidate` (`kesh-db/tests/reconciliation_repository.rs:2299-…`) | **Exacte** |
| Le côté client l'évite | quantité et prix bornés à `MAX_DECIMAL_SCALE` (`routes/invoices.rs:483`, `:502` ; `routes/limits.rs:28`), `line_total` arrondi à quatre décimales (`repositories/invoices.rs:458`), écart sous le centime traité au règlement | **Exacte** |

### Une conséquence que l'issue ne nommait pas

- **Le fichier pain.001 arrondit chaque montant, pas la somme de contrôle** : `InstdAmt` = `round_dp(2)` de chaque TTC
  (`kesh-payment/src/pain001/mod.rs:124-126`), `CtrlSum` = arrondi de la somme **non arrondie** (`:148-150`). Avec des
  TTC sous le centime, les deux peuvent différer, et la banque rejette le fichier. Hors périmètre (C-v017-3-16) : issue
  demandée. La 3b ferme la source pour les factures nouvelles.
- La quantité ou le prix à plus de quatre décimales sont **arrondis en silence** par MariaDB à l'insertion
  (`quantity DECIMAL(19,4)`, `unit_price DECIMAL(19,4)`, `20260628000001_supplier_invoices.sql:84-85`), alors que
  `line_total` est calculé en Rust sur les valeurs non arrondies : la ligne enregistrée peut ne plus égaler `quantité ×
  prix`. La borne à quatre décimales (AC-2) le ferme.

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Refuser une ligne dont `quantité × prix` n'est pas au centime, aux deux routes de création, par une fonction pure unique de `kesh-core` appelée par `create_in_tx` ; borner quantité et prix à quatre décimales | C-v017-3-13 **(Q1)** |
| D2 | L'existant : aucune migration, aucun `UPDATE`, paiement **non** refusé ; la fiche avertit et dit la conduite | C-v017-3-14 **(Q2)** |
| D3 | Complétion d'une pièce importée : la garde de ligne avant `AMOUNT_MISMATCH` | C-v017-3-15 |
| D4 | pain.001 (`CtrlSum`) : issue demandée | C-v017-3-16 |

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
  **après `normalize()`** (`scale_within`, `routes/limits.rs:31`) : `400 VALIDATION_ERROR`, message qui nomme la ligne
  (rang 1-based) et le champ, comme la facture client (`routes/invoices.rs:483-510`, à reprendre sans recopier si une
  fonction s'en dégage).
- **AC-3** — `create_in_tx` appelle `purchase_line_total` dans sa **première passe** (la passe de forme, `:399-414`,
  avant les réglages, les comptes et l'exercice — « un refus de forme ne prend ainsi aucun verrou », `:415-419` ;
  ⚠️ l'étape (0) du projet analytique, `:370-372`, la précède et verrouille déjà : le dire au doc-comment, ne pas la
  déplacer) et rend une variante neuve
  `DbError::SupplierLineNotInCentimes { line: usize (1-based), line_total: Decimal }` ; la seconde passe (`:424-…`)
  relit `line_total` depuis le résultat de la première (aucun second produit). Côté API : `AppError` neuve, `400`, code
  `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, `details { line, lineTotal }`, message fr-CH « La ligne {line} donne un
  montant de {lineTotal}, qui n'est pas au centime. Saisissez la ligne au montant facturé par le fournisseur (par
  exemple quantité 1 et le montant de la ligne). » ; repli d'`errors.rs` égal au catalogue fr-CH (garde G9) ; quatre
  locales.
- **AC-4** — Invariant : une facture créée a un `total_amount` au centime. Test `#[sqlx::test]` sur quelques
  combinaisons (taux 0, 2.6, 8.1 ; plusieurs lignes) : `total_amount.normalize().scale() <= 2`, et la ligne créanciers
  de l'écriture d'achat l'égale.
- **AC-5** — Tests HTTP (`POST /supplier-invoices`) : une ligne `3 × 33.3333` → `400
  SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, `details.line = 1`, `details.lineTotal = "99.9999"` ; une quantité `1.00001` →
  `400 VALIDATION_ERROR` ; **rien d'écrit** (ni facture, ni écriture d'achat, ni trace), vérifié en base ; une saisie
  nominale au centime inchangée.
- **AC-6** — Complétion d'une pièce importée (`routes/imported_supplier_invoices.rs`) : ordre des refus — conversion
  (AC-2) → **garde de ligne** (la même fonction, avant le calcul de `Σ TTC`, `:251-275`) → `AMOUNT_MISMATCH` →
  `create_in_tx`. Le calcul de `Σ TTC` de la complétion lit `line_total` depuis `purchase_line_total` (aucun second
  produit). Le test (b) de `inbox_import_e2e.rs` (`:1375`, `3 × 33.3333 = 99.9999`) attend désormais
  `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES`, et un cas neuf à lignes au centime dont la somme diffère du QR garde
  `AMOUNT_MISMATCH`. La pièce reste à compléter après le refus (aucune transition).
- **AC-7** — Les tests et fixtures qui créent des factures fournisseur **par les routes ou par `create_in_tx`** avec une
  ligne sous le centime se recensent (`grep -rnE "quantity|unit_price|unitPrice" crates/*/tests frontend/tests/e2e
  crates/kesh-seed` puis tri à la main) : chacun est adapté, ou réécrit pour insérer l'état **existant** par SQL s'il
  teste précisément un TTC sous le centime (ex. T-b9, qui insère déjà directement, `insert_supplier_invoice`). Liste et
  sort de chacun au Dev Agent Record. Le seed de démonstration (`kesh-seed`) n'en crée aucun après la story.

### L'existant (contrainte du 2026-10-30)

- **AC-8** — Aucun fichier sous `crates/kesh-db/migrations/` n'est ajouté ni modifié ; aucun `UPDATE`/`INSERT`/`DELETE`
  nouveau (`git diff c5101b07 -- crates/*/src | grep -nE '^\+.*\b(UPDATE|INSERT|DELETE)\b'` vide, tri au Dev Agent
  Record).
- **AC-9** — Une facture **existante** au TTC sous le centime (fixture insérée par SQL, `100.0040`, comme une base
  0.16.x) reste lisible (`GET /supplier-invoices/{id}`, liste), **payable** depuis la fiche (le paiement n'est pas
  refusé — C-v017-3-14), annulable ouverte, et son règlement annulable ; aucune de ces lectures ne la modifie (état
  relu à l'identique avant le geste). Tests HTTP.
- **AC-10** — La fiche de la facture (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`, total `:386`)
  affiche, quand `totalAmount` n'est pas au centime (helper **existant** `hasSubCentime`,
  `supplier-invoice-form.ts:63-71`, réutilisé), un avertissement (`data-testid="supplier-invoice-subcentime-warning"`) :
  « Le total de cette facture ({amount}) n'est pas au centime : son règlement par virement ne pourra jamais être
  rapproché d'un relevé bancaire. » suivi de la conduite selon l'état — **ouverte** : « Annulez-la, puis ressaisissez-la
  au montant facturé. » ; **payée** : « Annulez le règlement, puis la facture ; ressaisissez-la au montant facturé,
  puis réglez-la par la réconciliation, ou depuis sa fiche avant de rattacher la transaction à son règlement. » ;
  **annulée** : rien. Le texte ne cite aucun numéro d'issue. Vitest des trois états et du cas au centime (aucun
  avertissement).
- **AC-11** — La conduite est **exécutable** : un test HTTP la joue de bout en bout sur la fixture de l'AC-9 payée
  depuis la fiche par virement — annulation du règlement (`POST /supplier-invoices/{id}/settlement/cancel`), annulation
  de la facture (`POST /supplier-invoices/{id}/cancel`), ressaisie au centime, règlement — et constate que la ligne de
  banque du nouveau règlement est au centime. ⚠️ **À établir à l'étape 0** : que l'annulation du règlement d'une facture
  payée **par un lot pain.001 confirmé** est admise ; si elle ne l'est pas, le manuel et l'avertissement disent la
  conduite réelle pour ce cas (et le Dev Agent Record le motive) — ne pas l'inventer.

### L'écran de saisie (`frontend`)

- **AC-12** — `SupplierInvoiceForm.svelte` : une ligne dont `quantité × prix` n'est pas au centime affiche sous elle
  « Le montant de la ligne ({lineTotal}) n'est pas au centime. » (`data-testid="supplier-invoice-line-subcentime"`), et
  `linesInvalid` (`supplier-invoice-form.ts:73-83`) la tient pour invalide (*Enregistrer* inactif). Le contrôle est une
  fonction pure de `supplier-invoice-form.ts` (`big.js`, jamais `parseFloat`), miroir de `purchase_line_total`, avec
  ses tests Vitest des mêmes cas que l'AC-1. Un refus serveur `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` s'affiche traduit,
  la ligne désignée (le patron de traduction des erreurs du formulaire, `supplier-invoice-form.ts:186`, à étendre).
- **AC-13** — i18n : clés nouvelles dans les **quatre** catalogues ; `npm run lint-i18n-ownership` vert. L'avis
  existant `supplier-invoices-subcentime` (« … impossible à atteindre par des lignes centime-exactes … »,
  `messages.ftl` fr-CH `:2545`) devient vrai : le relire, ne pas le dupliquer.
- **AC-14** — Recette visuelle (thème clair, base de démonstration ; la facture existante sous le centime s'y prépare
  par SQL dans `kesh_v0173b`, jamais dans une base de Guy) : le formulaire avec une ligne refusée ; la fiche d'une
  facture ouverte et d'une facture payée avec l'avertissement. Captures dans
  `/home/gcorbaz/devel/kesh-gate-logs/v0173b/captures/`, listées au Dev Agent Record.
- **AC-15** — E2E : `frontend/tests/e2e/supplier-invoices.spec.ts` (ou la spec existante de saisie fournisseur, à
  citer) gagne un test : une ligne `3 × 33.3333` affiche le message et laisse *Enregistrer* inactif ; corrigée en
  `1 × 99.99`, l'enregistrement passe.

### Les textes

- **AC-16** — Manuel utilisateur FR (`docs/manual/fr/user-manual.tex`, PDF régénéré) : § *Saisir une facture
  fournisseur* (`:1571-…`) — une ligne se saisit au centime (quantité et prix à quatre décimales au plus, montant de la
  ligne au centime), et pourquoi ; § *Régler une facture fournisseur* (`sec:regler-facture-fournisseur`, `:1630`) — une
  facture enregistrée avant la 0.17.0 dont le total n'est pas au centime : l'avertissement et la conduite (AC-10,
  AC-11) ; § *Un débit : les factures fournisseur* (`sec:matching-fournisseur`, `:2304`) — la phrase « un total à plus
  de deux décimales n'est jamais proposé » complétée (cela ne se produit plus que pour une facture enregistrée avant la
  0.17.0, renvoi à la conduite). Et tout ce que rend `grep -n "deux décimales\|sous-centime\|sous le centime\|centime-exact" docs/manual/fr/user-manual.tex`, trié.
- **AC-17** — `docs/api-external.md` : le code `SUPPLIER_INVOICE_LINE_NOT_IN_CENTIMES` au tableau des codes, la borne
  à quatre décimales de `POST /supplier-invoices` et de la complétion, « *(Depuis la v0.17.0.)* ». ⚠️ C'est un
  **changement de contrat** pour un client d'API qui envoyait des lignes sous le centime : le dire ainsi.
- **AC-18** — `CHANGELOG.md` : sous `## [0.17.0]`, section **Corrigé**, une entrée (`refs` [#687] par un lien, sans
  mot-clé de fermeture) : le refus, et ce qu'il advient des factures existantes (non modifiées, signalées). Aucune
  version touchée.

## Tâches

1. **Étape 0** — recontrôler le § *À recontrôler*, le noter au Change Log, ligne lue citée ; établir le cas du lot
   (AC-11). Bases `kesh_v0173b`, `kesh_e2e_v0173b`.
2. **Tests rouges d'abord**, commités rouges.
3. `kesh-core` (AC-1) ; `kesh-db` (AC-3, AC-4) ; `kesh-api` (AC-2, AC-3, AC-5, AC-6, AC-9, AC-11) ; recensement AC-7.
4. `frontend` (AC-10, AC-12, AC-13) ; Vitest ; E2E (AC-15) ; captures (AC-14).
5. Textes (AC-16 à AC-18), PDF régénéré sous verrou et `mem-guard` ; **greper la valeur** des phrases corrigées.
6. Gates, mutations, Dev Agent Record.

## Tests et mutations prévus

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `purchase_line_total` : `normalize()` retiré avant le contrôle d'échelle | AC-1 (`4 × 2.5000` admis) |
| M2 | `purchase_line_total` : seuil `<= 4` au lieu de `<= 2` | AC-1, AC-5 |
| M3 | `create_in_tx` : l'appel retiré | AC-4, AC-5 |
| M4 | `create_in_tx` : la garde déplacée **après** les verrous de la seconde passe | revue (aucun test ne voit un verrou) — **survivante assumée** si non tuée, le dire |
| M5 | complétion : garde de ligne après `AMOUNT_MISMATCH` | AC-6 (code attendu) |
| M6 | `to_new_supplier_invoice` : borne de quantité retirée | AC-5 (`1.00001`) |
| M7 | fiche : avertissement affiché aussi pour une facture annulée | Vitest AC-10 |
| M8 | formulaire : contrôle de ligne retiré de `linesInvalid` | Vitest AC-12 |
| M9 | `DbError` → `AppError` : `400` remplacé par `409` | AC-5 |

## Gates

`kesh-db` est touché : **gate complet même en cours de boucle**. Au dernier commit de code : `scripts/test-fast.sh --ci`
sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock`, `cargo build --workspace --all-targets`, Vitest complet,
`npm run check`, `npm run lint-i18n-ownership`, `npm run build`, suite E2E complète selon `docs/testing.md` à la
lettre. Ne déclarer que ce qui a tourné.

## Ce que les gardes ne voient pas

- **L1 — Le grand livre existant** : une facture enregistrée avant la 0.17.0 garde son total ; tant que l'utilisateur ne
  la traite pas, son règlement n'est ni proposé ni rattachable (v017-3a), et un lot pain.001 qui la contient peut
  porter une somme de contrôle fausse (issue demandée, C-v017-3-16).
- **L2 — Une facture réelle sous le centime** : si un fournisseur facture réellement un montant sous le centime (cas non
  rencontré en Suisse pour un montant payable), la saisie exige de la ramener au centime — le montant payable l'est
  toujours.
- **L3 — L'API externe** : un client qui envoyait des lignes sous le centime reçoit désormais un `400` (AC-17).

## À recontrôler à l'étape 0 du développement

- `supplier_invoices.rs:337-560` (les deux passes, l'ordre des verrous, `generate_purchase_journal_lines` `:120-178`).
- `routes/supplier_invoices.rs:278-300`, `:468-…` ; `routes/imported_supplier_invoices.rs:240-290`.
- `routes/invoices.rs:483-510` et `routes/limits.rs:28-36` (la forme des refus d'échelle côté client).
- L'annulation du règlement d'une facture payée par un lot confirmé (`supplier_settlement_cancel_blocker`,
  `supplier_invoices.rs:1480-…`) — AC-11.
- `inbox_import_e2e.rs:1360-1400` (le test (b)).
- Les mappings `DbError` → `AppError` (où ajouter la variante ; exhaustivité des `match`).

## Collision avec la story parallèle v016-2

Aucun fichier métier commun. Fusion mécanique : `errors.rs` (variante et code), les quatre catalogues i18n,
`docs/api-external.md` (tableau des codes), `CHANGELOG.md` (`## [0.17.0]`), `user-manual.tex` (sections distinctes) et
son PDF (à régénérer après fusion).

## Questions au Project Lead

- **Q1 (C-v017-3-13)** — Refuser une ligne dont le montant n'est pas au centime, plutôt que de l'arrondir en silence ou
  de régler avec une ligne d'arrondi : d'accord ?
- **Q2 (C-v017-3-14)** — Une facture existante sous le centime n'est ni modifiée ni bloquée : sa fiche l'avertit et dit
  la conduite (annuler, ressaisir). Voulez-vous en plus une liste de ces factures (un filtre), ou l'avertissement
  suffit-il ?
- **Q3 (C-v017-3-16)** — Ouvrir l'issue de la somme de contrôle du pain.001 (elle fera monter le décompte) ?

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), découpée de `v017-3-rapprochement-attente` avec la v017-3a ; registre
  C-v017-3-1, 13 à 17 ; prompt de validation P1 (`v017-3b-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q3 à
  Guy, puis validation P1 (Sonnet, lentilles R et F).
