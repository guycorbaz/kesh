# Story 25.4-c4-a : L'arrondi à 5 centimes, figé à la validation

Status: ready-for-dev

**Issue : [#494]** (CR) — ⛔ la PR porte `refs #494` : la c4-b, qui rend l'arrondi visible, la fermera.

**Première des deux stories de la 25-4-c4** (découpage : sept modules au total). Arbitrages de Guy
(2026-10-01, consignés dans #494) :

1. **Réglage par société**, activé par défaut : les factures **émises** portent l'arrondi ; les
   **paiements** peuvent ne pas être arrondis (la 25-4-c3-b continue de les traiter au centime).
2. **Les avoirs suivent la même règle** : un avoir total reprend exactement le total de la facture.
3. **La ligne d'arrondi ne porte pas de TVA** ; l'écart s'écrit **à la validation** sur le compte de
   différences d'arrondi (`default_rounding_account_id`, 25-4-c3-a1/a2).
4. Story à part, **avant la 25-4-d** (#384).

Exemples de Guy : total calculé 123.44 → arrondi **+0.01** → **123.45** ; 234.52 → **−0.02** → **234.50**.

**Empilée sur la c3-b** (PR #493) : branche `story/25-4-c4-arrondi-facture-5-centimes`.

**Découpage.**
- **c4-a (celle-ci)** : la règle, la migration, la validation, l'écriture de vente, **toutes les formes du
  TTC** et l'avoir.
- **c4-b** : l'arrondi **visible** et le réglage. Elle porte la ligne « Arrondi » du PDF et de la fiche
  facture, l'aperçu en brouillon, l'interrupteur de *Paramètres → Facturation*, l'API du réglage, les
  libellés et les manuels.

⚠️ **Entre les deux stories, le PDF montre un total TTC arrondi sous un sous-total et une TVA qui ne
s'additionnent plus**, faute de la ligne « Arrondi ». Ce décalage est accepté parce que les deux stories
sont empilées et partent dans la même release. Il est à écrire dans le CHANGELOG de la c4-b, pas ici.

## Story

En tant que comptable suisse,
je veux que le total d'une facture émise soit arrondi à 5 centimes et que l'écart soit comptabilisé,
afin que la facture, sa QR-facture et la créance portent le montant que le client paiera effectivement.

## Les faits, vérifiés dans le code (inventaire du 2026-10-01)

- **Le TTC n'est stocké nulle part** : `invoices.total_amount` et `credit_notes.total_amount` contiennent
  le **HT** (`invoices.rs:430-434` `compute_total`, `credit_notes.rs:537-551`).
- **Deux définitions du TTC**, tenues d'accord par des tests de parité :
  - **SQL** : la macro `line_ttc_sql!` (`invoices.rs:165-177`), les formes scalaire
    `INVOICE_TTC_SUBQUERY_SQL` (`:189`) et jointe `INVOICE_TTC_DERIVED_JOIN_SQL` (`:202`), et pour les avoirs
    `INVOICE_CREDITED_SUBQUERY_SQL` / `_DERIVED_JOIN_SQL` (`invoice_settlements.rs:60-81`) ;
  - **Rust** : `kesh_core::accounting::vat::invoice_total_ttc` (`vat.rs:63`).
  - **Tests de parité** : `crates/kesh-db/tests/invoice_ttc_parity.rs` (quatre voies, dont le débit de
    créance), `invoice_amount_due_parity.rs`, et `kesh-report/tests/aged_receivables.rs:232`.
- **Appelants Rust du TTC** :
  - `routes/invoices.rs:291` (`total_ttc` de `InvoiceResponse`) ;
  - `invoice_pdf_service.rs:348` (montant de la QR et total du PDF) ;
  - `invoice_email.rs:187` (`{amount}`) ;
  - `routes/credit_notes.rs:315` (PDF de l'avoir).
- **Une ligne d'arrondi négative est impossible dans `invoice_lines`** :
  `chk_invoice_lines_line_total_non_negative`, `unit_price >= 0`, `quantity > 0` (squash `:607-611` ; avoirs
  `:422-426`). D'où une **colonne d'en-tête**.
- **L'écriture de vente** est produite par `generate_invoice_journal_lines` (`invoices.rs:1735-1832`), dans
  cet ordre :
  - (0) débit créance `total_ht + total_vat` ;
  - crédits produit ;
  - crédits TVA.
- **Trois lecteurs prennent la créance comme « la première ligne au débit »** (`jel.debit > 0 ORDER BY jel.id
  LIMIT 1`) : `invoice_settlements_write.rs:99-110`, `routes/reconciliation.rs:1421-1426` (dont le commentaire
  dit « EXACTEMENT UNE ligne de débit »), et l'écriture d'avoir en miroir.
- **La validation** : `validate_invoice` (`invoices.rs:1836`) lit les réglages (`:1903`), refuse une pièce à
  zéro (`:1925`), produit l'écriture (`:2096-2103`), puis passe la facture en `validated` (`:2140`).
  **La dévalidation** : `unvalidate` (`:1441`, remise en brouillon `:1576`).
- **L'avoir est toujours total** (`create_credit_note`, `credit_notes.rs:261`) : il recopie les lignes
  (`:356-363`). Son écriture (`:187-259`) est le miroir de la vente, avec en (0) le crédit de la créance.
- **La sauvegarde** suit les colonnes par `INFORMATION_SCHEMA` (`backup.rs:112, :355`) ; l'import complet
  tolère une colonne absente (`admin_full_import_e2e.rs:2351`).
- **Le rapprochement compare au reste dû** : il suit sans changement dès que le reste dû inclut l'arrondi.

## Acceptance Criteria

**AC 1 — La règle, une fois.** `Money::round_to_5_centimes()` (`kesh-core/src/types/money.rs`, à côté de
`round_to_centimes`) arrondit au multiple de 0.05 le plus proche, l'équidistant **loin de zéro** :
- 123.44 → 123.45 ; 234.52 → 234.50 ; 123.425 → 123.45 ;
- 10.0050 → 10.00 ; 0.025 → 0.05 ; −0.025 → −0.05.

Le commentaire de `round_to_centimes` qui renvoie à un arrondi cash « fourni séparément » est mis à jour.
Un helper `invoice_rounding(ttc_brut, actif) -> Decimal` rend l'écart `arrondi − brut`, et `0` si le
réglage est inactif. Tests unitaires sur ces exemples et sur un TTC déjà multiple de 0.05.

**AC 2 — La migration** (non breaking au sens de P1). Elle porte :
- `invoices.rounding_amount DECIMAL(19,4) NOT NULL DEFAULT 0` ;
- `credit_notes.rounding_amount DECIMAL(19,4) NOT NULL DEFAULT 0` ;
- `company_invoice_settings.round_to_5_centimes BOOLEAN NOT NULL DEFAULT TRUE`.

Le défaut `0` décrit exactement les pièces antérieures, qui n'étaient pas arrondies. Aucun backfill ; P7 :
inscription à `EXEMPT_MIGRATIONS` si le détecteur l'exige (DDL seul, pas d'écriture de données).
Garde-fous du dépôt :
- **P5** : ligne d'audit et compteurs recomptés, 71 migrations ;
- **P6** : `migrations_upgrade_path.rs`, `assert_eq!(total, 70)` → 71 et fenêtre 36 → 37, valeurs grepées
  sur tout le fichier ;
- squash régénéré (`scripts/regen-test-schema.sh`), `migrations.sha384` mis à jour.

L'entité et la projection des réglages portent le booléen (lu par la validation). Son API et son écran
sont pour la **c4-b**.

**AC 3 — Figé à la validation.** `validate_invoice` calcule le TTC brut des lignes (même formule que
`invoice_total_ttc`), puis `rounding = invoice_rounding(brut, settings.round_to_5_centimes)`, et :
- **si `rounding != 0`** :
  - lit le compte d'arrondi par `company_invoice_settings::rounding_account_for_write`, dans la
    transaction et **sous le verrou de la facture** ;
  - absent ou invalide → `RoundingAccountNotConfigured` (400 `ROUNDING_ACCOUNT_NOT_CONFIGURED`), **rien
    d'écrit**, la facture reste brouillon ;
- **le total arrondi est nul** (pièce minuscule, par ex. TTC 0.02 → 0.00) → refus
  `InvalidInput("invoiceTotalZero")`, comme une pièce à zéro ;
- `invoices.rounding_amount` est posé dans l'`UPDATE` qui passe la facture en `validated` (`:2140`) ;
- `unvalidate` le remet à `0` (`:1576`).

Une facture **validée** garde son arrondi même si le réglage change ensuite.

**AC 4 — L'écriture de vente.** Le débit de créance vaut le **TTC arrondi** (brut + arrondi). L'arrondi
**positif** s'écrit en **crédit**, le **négatif** en **débit**, sur le compte d'arrondi, en ligne **finale**,
donc après la créance. ⛔ La créance reste la **première ligne au débit** : les trois lecteurs `LIMIT 1`
restent justes. Le commentaire de `reconciliation.rs:1421` (« EXACTEMENT UNE ligne de débit ») est corrigé
en « la créance est la PREMIÈRE ligne au débit », et le même invariant est écrit à côté du générateur.
L'écriture est équilibrée, sans ligne à zéro.

**AC 5 — Toutes les formes du TTC.**
- **SQL** : `INVOICE_TTC_SUBQUERY_SQL` et `INVOICE_TTC_DERIVED_JOIN_SQL` ajoutent `i.rounding_amount`, et
  `INVOICE_CREDITED_*` ajoutent `cn.rounding_amount`. Le reste dû, les agrégats (#416), le rapprochement et
  la balance âgée suivent sans modification.
- **Rust** : un helper (par ex. `invoice_total_ttc_rounded(lines, rounding)`) sert les quatre appelants
  (`routes/invoices.rs:291`, `invoice_pdf_service.rs:348`, `invoice_email.rs:187`, `credit_notes.rs:315`),
  qui lisent l'arrondi **figé** de la pièce.
- **Les tests de parité** sont étendus à une pièce arrondie (positive et négative), dans les quatre voies,
  débit de créance compris.
- ⛔ **Inventaire du symptôme avant de conclure** :
  `grep -rn "invoice_total_ttc\|INVOICE_TTC\|line_ttc_sql!\|INVOICE_CREDITED" crates/`. Chaque site est soit
  arrondi, soit écrit comme exclu avec sa raison (fournisseur, lignes du rapport TVA).

**AC 6 — L'avoir.**
- `create_credit_note` recopie `rounding_amount` de la facture : l'avoir total annule **exactement** la
  facture, et le reste dû d'une facture créditée tombe à `0`.
- L'écriture d'avoir porte l'arrondi en miroir (débit si l'arrondi de la facture est positif, crédit s'il
  est négatif), sur le compte d'arrondi lu **au moment d'écrire**. Absent ou invalide →
  `RoundingAccountNotConfigured`, rien d'écrit.
- La créance reste la première ligne de l'avoir au crédit.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- validation de 123.44 → `rounding_amount` +0.01, écriture (crédit 0.01 sur le compte d'arrondi), créance
  123.45, reste dû 123.45 ; idem 234.52 → −0.02 en débit ;
- réglage désactivé → `rounding_amount = 0`, écriture inchangée ;
- compte d'arrondi absent, puis archivé → validation refusée, facture restée brouillon ; TTC déjà rond →
  aucun compte exigé ;
- règlement de 123.45 → soldée, **deux lignes** (le chemin d'écart au centime n'est pas pris) ;
- avoir sur une facture arrondie → reste dû 0, écriture miroir ;
- dévalidation → `rounding_amount` revenu à 0 ;
- sauvegarde : export puis import d'une base à facture arrondie, `rounding_amount` restauré ;
- mutations : arrondi omis de la forme SQL ; ligne d'arrondi retirée de l'écriture (déséquilibre) ;
  arrondi non recopié dans l'avoir.

**AC 8 — Textes de la c4-a.** CHANGELOG `[0.12.1]` *Changed* (#494) : le total d'une facture émise est
arrondi à 5 centimes, ce qui change le montant de la QR-facture. La c4-b complétera l'entrée (ligne
visible, réglage). Le manuel n'est pas touché ici : la c4-b l'écrit avec l'écran.

## Tasks / Subtasks

- [ ] **T1 — la règle** (AC 1) : `round_to_5_centimes`, `invoice_rounding`, tests unitaires.
- [ ] **T2 — la migration** (AC 2) : trois colonnes, garde-fous P5/P6/P7, squash, sommes de contrôle.
- [ ] **T3 — la validation et la dévalidation** (AC 3).
- [ ] **T4 — l'écriture de vente** (AC 4).
- [ ] **T5 — les formes du TTC** (AC 5) : SQL, helper Rust, quatre appelants, parité, inventaire.
- [ ] **T6 — l'avoir** (AC 6).
- [ ] **T7 — tests et mutations** (AC 7) ; tests existants dont les montants changent (TTC non rond sous
  réglage par défaut) : les mettre à jour en **disant pourquoi**, jamais en désactivant le réglage pour les
  faire passer.
- [ ] **T8 — CHANGELOG** (AC 8).
- [ ] **T9 — gates** : backend complet (base remise à zéro ; migration ⇒ gate complet même en cours de
  boucle), frontend complet, **E2E complet** (des specs affirment des TTC précis).

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Une ligne d'arrondi dans `invoice_lines`** : les contraintes interdisent le négatif, et le rapport TVA
  la compterait.
- ⛔ **Calculer l'arrondi à la volée d'après le réglage courant** pour une pièce validée : le reste dû, le
  PDF et la QR divergeraient de l'écriture déjà passée dès que le réglage change.
- ⛔ **Placer la ligne d'arrondi au débit AVANT la créance** : les lecteurs `LIMIT 1` prendraient le compte
  d'arrondi pour la créance.
- ⛔ **Toucher les factures fournisseur** : leur `total_amount` est déjà TTC, et elles ne sont pas émises par
  la société.
- ⚠️ **Le chemin d'écart au centime de la c3-b** (`classify_payment`) reste en place : il sert aux factures
  émises sans arrondi (réglage désactivé, factures antérieures) et aux paiements non arrondis.

### Tests existants qui vont bouger

Le réglage est actif par défaut. Toute facture de test au TTC non multiple de 0.05 change donc de total
(par ex. 108.10 reste rond, mais 10.0050 et 9.2540 des tests de la c3-b deviennent 10.00 et 10.00). Les
tests de la c3-b **désactivent le réglage** dans leur montage : ils portent sur des factures émises sans
arrondi, et c'est précisément le cas qu'ils couvrent. Les autres sont relus un par un.

### Modules

`kesh-core`, `kesh-db`, `kesh-api` (+ CHANGELOG) — sous le seuil.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-01** — Créée après l'inventaire des sites du TTC. Arrondi à 5 centimes **figé à la validation**
  dans une colonne d'en-tête (les lignes ne peuvent être négatives), écart en ligne finale de l'écriture de
  vente sur le compte d'arrondi, toutes les formes du TTC alignées, avoir en miroir. Découpage c4-a (fond) /
  c4-b (visible et réglage).

[#494]: https://github.com/guycorbaz/kesh/issues/494
