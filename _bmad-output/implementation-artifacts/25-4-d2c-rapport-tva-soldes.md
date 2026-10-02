# Story 25.4-d2c : Solder le reste — le rapport TVA retranche la TVA des soldes

Status: ready-for-dev

**Issue : [#384]** — fermée par cette story (la PR porte `closes #384`, titre **et** corps). Dernière des trois stories
de la 25-4-d2. Empilée sur la 25-4-d2b (branche `story/25-4-d2c-rapport-tva-soldes`).

## Arbitrages (Project Lead)

- 2026-10-01 : le rapport TVA est étendu pour **retrancher la TVA des soldes** (escompte, perte sur débiteur) ;
- retenu par défaut le 2026-10-01 (Guy : « pousse puis continue ») : retrancher **par taux, dans la période du solde**,
  à partir de la ventilation figée sur la ligne de solde ; **#390** (dériver la TVA due du grand livre) reste à part ;
- 2026-10-02 : « oui, enchaîne ».

## Story

En tant que comptable,
je veux que le décompte TVA tienne compte des escomptes accordés et des pertes sur débiteurs,
afin que la TVA due déclarée soit celle des contre-prestations réellement obtenues.

## Les faits, vérifiés dans le code

- **Le calcul** (`crates/kesh-report/src/vat_report.rs:83-190`, `generate`) : TVA due tirée des **lignes de facture**
  validées de la période (`i.date BETWEEN`), agrégée par taux avec arrondi par ligne ; TVA récupérable lue au **grand
  livre** (`recoverable_balance`) ; `vat_balance = total_vat_due − total_vat_recoverable` ; `reconciliation_delta` =
  TVA due des documents − solde du compte de TVA due **limité aux écritures de vente** des factures validées de la
  période (`due_account_balance_sales_scope`, `INNER JOIN invoices i ON i.journal_entry_id = jel.entry_id`).
- **Aujourd'hui, un solde n'est vu nulle part** : ni dans la TVA due (documents), ni dans le delta (qui ne joint que
  l'écriture de vente). La TVA corrigée d'un escompte reste donc due au décompte.
- **La ventilation figée** (d2a) : `invoice_settlements.write_off_vat` = `[{ratePercent, baseHt, vatAmount}]` (chaînes
  décimales), `[]` pour les frais et l'arrondi ; `settled_on` = date du solde ; l'écriture débite la TVA due, une ligne
  par taux, sur le compte `default_vat_payable_account_id` **courant au moment du solde**.
- **Les rendus** : `render_vat_report_csv` (`kesh-report/src/csv.rs:363`), `render_vat_report_pdf` (`pdf.rs:1063`) ; la
  route `GET /api/v1/reports/vat` (`kesh-api/src/routes/reports.rs`) sérialise `VatReport` tel quel ; l'écran
  `frontend/src/lib/features/reports/VatReportView.svelte` (**sans test** aujourd'hui) et ses types
  (`reports.types.ts:106-124`). ⚠️ **Trois gardes « rapport vide »** : `csv.rs:377`, `pdf.rs:1078`, et côté écran
  `isReportEmpty('vat', …)` (`frontend/src/lib/features/reports/reports.api.ts:177-183`, utilisé par
  `VatReportView.svelte:11`) — chacun teste « aucune vente et récupérable nul ».
- **Tests existants** : `kesh-report/tests/vat_report_reconciliation.rs`, `vat_report_recoverable.rs`,
  `kesh-api/tests/vat_report_e2e.rs`, les fixtures de `pdf.rs` (`fixture_vat`, `:1996`, et `vat_report_pdf_recoverable_only_renders`, `:2042`) et du test unitaire
  `vat_report.rs` (`aggregate`, `:275`) — tous construisent un `VatReport` littéral.
- **Manuel** : `docs/manual/fr/user-manual.tex` § *Décompte TVA* (`:1631` env.) affirme déjà un calcul « à partir des
  écritures réellement comptabilisées » (faux depuis toujours, #390) et porte un `keshwarning` sur l'avoir.

## Acceptance Criteria

**AC 1 — Les diminutions.** `VatReport` gagne `write_off_rows: Vec<VatWriteOffRow { rate, base_ht, vat }>` (agrégées
**par taux** depuis `write_off_vat` des lignes `invoice_settlements` de type `write_off`, **de la société**, dont
`settled_on` tombe dans la période — tri par taux comme `rows`) et `total_vat_write_off`. Les montants se lisent en
`Decimal` depuis les chaînes du JSON ; un JSON illisible est une **erreur** (`ReportError`), jamais un zéro silencieux.

**AC 2 — Le solde net.** `total_vat_due` et `rows` restent la TVA **facturée** (inchangés) ; un champ
`total_vat_due_net = total_vat_due − total_vat_write_off` s'ajoute ; `vat_balance = total_vat_due_net −
total_vat_recoverable`. ⚠️ Le sens de `vat_balance` change : documenté au champ et au CHANGELOG.

**AC 3 — La réconciliation.** Le delta compare la TVA **nette** au grand livre **net**. `ventes` =
`due_account_balance_sales_scope` (inchangé, `SUM(credit) − SUM(debit)`) ; `soldes` = **`SUM(debit) − SUM(credit)`** — de
signe **opposé**, magnitude positive : l'écriture de solde ne fait que débiter ce compte — sur les lignes du compte de
TVA due des **écritures de solde** de la période (`INNER JOIN invoice_settlements s ON s.journal_entry_id =
jel.entry_id AND s.settlement_type = 'write_off'`, `s.company_id = ?`, `s.settled_on BETWEEN`) ;
**`delta = total_vat_due_net − (ventes − soldes)`**. Vérification : facture de 1000 à 8.1 %, escompte corrigeant 10.00 →
`71.00 − (81.00 − 10.00) = 0`. Un solde de la période sur une facture d'une période **antérieure** : la TVA facturée
manque des deux côtés, la correction est des deux côtés — delta nul. Sans solde, le delta est inchangé.

**AC 4 — Les rendus.** CSV : après les lignes par taux et le total de TVA due, une section « Diminutions de
contre-prestation (soldes) » — une ligne par taux (taux, base HT, TVA), puis « Total TVA des soldes » et « TVA due
nette » — et le solde existant calculé sur le net. PDF : la même section, dans le style des tableaux existants.
L'écran : la même section, visible seulement s'il y a des soldes, et la ligne « TVA due nette ». Rapport sans vente mais
avec des soldes : **pas « vide »** — aux **trois** gardes (`csv.rs:377`, `pdf.rs:1078`, `isReportEmpty('vat', …)` dans
`reports.api.ts:177-183`), qui testent désormais aussi l'absence de soldes.

**AC 5 — L'API et les types.** Les champs (`writeOffRows`, `totalVatWriteOff`, `totalVatDueNet`) dans la réponse de
`GET /api/v1/reports/vat` et dans `reports.types.ts`. Les libellés de l'écran dans les 4 locales, `sitesTotal`
recompté.

**AC 6 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Décompte TVA* : les diminutions (escompte, perte) retranchées
par taux dans la période du solde ; les limites (AC 8) ; la phrase « à partir des écritures réellement comptabilisées »
corrigée au passage (elle est fausse : la TVA due vient des factures — #390). PDF régénéré, contrôlé aplati.
CHANGELOG `[0.12.1]` : l'entrée d2a (« le rapport TVA ne retranche pas encore … ») mise à jour ; #384 fermée.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- `kesh-report` (base) : une facture à deux taux, un escompte soldé dans la période → `write_off_rows` par taux,
  `total_vat_due_net`, `vat_balance`, **delta nul** ; un solde **hors** période → ignoré ; un solde de frais bancaires
  (`[]`) → aucune ligne ; un solde d'une **autre société** → ignoré ; un solde **annulé** → disparaît ;
- le delta reste nul sans solde (non-régression) ;
- JSON illisible → erreur ;
- rendus CSV et PDF : la section et la TVA nette ;
- Vitest de la vue (`VatReportView.test.ts`, à créer) et de `isReportEmpty('vat', …)` : **pas de vente, un solde → pas
  vide** ; un test d'API sur la forme de la réponse.

**AC 8 — Limites assumées** (écrites au manuel et dans le code) :
- un solde **annulé** disparaît du rapport de **sa** période (la ligne est retirée, la contre-passation est datée du
  jour) : si la période était déjà déclarée, le rapport de cette période change après coup — même comportement que
  l'avoir, lié à #390 ;
- la part à **0 %** d'un solde (lignes exonérées) n'apparaît pas : elle ne porte pas de TVA ;
- le delta suppose le compte de TVA due **stable** (limite L-1 existante).

## Tasks / Subtasks

- [ ] **T1 — le calcul** (AC 1, AC 2, AC 3), `vat_report.rs`.
- [ ] **T2 — les rendus** CSV et PDF (AC 4).
- [ ] **T3 — l'écran et les types** (AC 4, AC 5).
- [ ] **T4 — i18n** (AC 5).
- [ ] **T5 — manuel, CHANGELOG** (AC 6).
- [ ] **T6 — tests** (AC 7), dont les fixtures `VatReport` existantes complétées.
- [ ] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ Recalculer la ventilation depuis les lignes de facture : elle est **figée** sur la ligne de solde (un changement
  d'algorithme ne doit pas réécrire l'histoire).
- ⛔ Changer `total_vat_due` ou `rows` : ils restent la TVA facturée — le net est un champ à part.
- ⛔ Basculer vers le grand livre (#390) : hors périmètre.
- ⛔ Lire un JSON illisible comme zéro.

### Modules

`kesh-report` (calcul, rendus), `kesh-api` (si la route ne sérialise pas `VatReport` tel quel — à vérifier), `frontend`,
`kesh-i18n`, `docs` (+ `CHANGELOG`) — cinq au plus.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-02** — Créée (Guy : « oui, enchaîne »).
- **2026-10-02** — Validation P1 (Sonnet) : 1 HIGH, 1 MED, 2 LOW, retenus. Le garde « rapport vide » de l'**écran**
  (`isReportEmpty`, `reports.api.ts`) manquait — les trois gardes sont cités, et un test de la vue est prévu (H1) ; la
  formule du delta se contredisait sur le signe — `soldes` en `SUM(debit) − SUM(credit)`, vérification chiffrée écrite
  (M1) ; plage des types et second site de fixture PDF (L1, L2).

[#384]: https://github.com/guycorbaz/kesh/issues/384
