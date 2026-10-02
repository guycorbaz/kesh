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
  (`reports.types.ts:106-124`). ⚠️ **Trois gardes « rapport vide »** : `csv.rs:377`, `pdf.rs:1080`, et côté écran
  `isReportEmpty('vat', …)` (`frontend/src/lib/features/reports/reports.api.ts:177-183`, utilisé par
  `VatReportView.svelte:11`) — chacun teste « aucune vente et récupérable nul ».
- **Tests existants** : `kesh-report/tests/vat_report_reconciliation.rs`, `vat_report_recoverable.rs`,
  `kesh-api/tests/vat_report_e2e.rs`, ⚠️ l'E2E `frontend/tests/e2e/reports.spec.ts:234-237`
  (`getByText(/TVA due/i)`, `getByText(/Solde/i)` — sous-chaînes, mode strict : un libellé neuf qui les contient les fait
  échouer, et l'escompte daté du jour que crée `invoice-write-off.spec.ts` rend l'échec **dépendant de l'ordre** des
  specs), les fixtures de `pdf.rs` (`fixture_vat`, `:1996`, et `vat_report_pdf_recoverable_only_renders`, `:2042`) et du test unitaire
  `vat_report.rs` (`aggregate`, `:275`) — tous construisent un `VatReport` littéral.
- **Manuel** : `docs/manual/fr/user-manual.tex` § *Décompte TVA* (`:1649`) affirme déjà un calcul « à partir des
  écritures réellement comptabilisées » (faux depuis toujours, #390) et porte un `keshwarning` sur l'avoir.

## Acceptance Criteria

**AC 1 — Les diminutions.** `VatReport` gagne `write_off_rows: Vec<VatWriteOffRow { rate, base_ht, vat }>` (agrégées
**par taux** depuis `write_off_vat` des lignes `invoice_settlements` de type `write_off`, **de la société**, dont
`settled_on` tombe dans la période — tri par taux comme `rows`) et `total_vat_write_off`. Les montants se lisent en
`Decimal` depuis les chaînes du JSON, par une fonction placée **à côté de** `write_off_vat_json` (`kesh-db`,
`invoice_settlements.rs` — la forme ne s'écrit qu'à un endroit). Un JSON **de forme fausse** (pas un tableau, clé
absente, valeur non chaîne, décimal illisible — la syntaxe, elle, est garantie par `json_valid`) est une **erreur**, jamais
un zéro silencieux : variante neuve `ReportError::CorruptData(String)`, mappée en **500** dans `From<ReportError> for AppError`
(`kesh-api/src/errors.rs:931-975`, `match` exhaustif) **sur le précédent de `TrialBalanceUnbalanced`** (`:954-963`) :
`tracing::error!` puis `AppError::Internal(…)` (code `INTERNAL_ERROR`).

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

**AC 4 — Les rendus.** Partout (CSV, PDF, écran), la section et la « TVA due nette » n'apparaissent **que s'il y a des
soldes** dans la période : sans soldes, chaque rendu est inchangé. CSV : après les lignes par taux et le total de TVA
due, une section « Diminutions de contre-prestation (soldes) » — une ligne par taux (taux, base HT, TVA), puis « Total TVA des soldes » et « TVA due
nette » — et le solde existant calculé sur le net. PDF : la même section, dans le style des tableaux existants.
Le CSV porte une **ligne de titre** pour la section (première colonne seule), qui la distingue des lignes de vente de même
forme. `VatPdfLabels` (`pdf.rs:1004`) gagne trois libellés (titre de section, total des soldes, TVA due nette).
L'écran : la même section et la ligne « TVA due nette », **seulement s'il y a des soldes** (sans soldes, le rapport est
inchangé). Rapport sans vente mais
avec des soldes : **pas « vide »** — aux **trois** gardes (`csv.rs:377`, `pdf.rs:1080`, `isReportEmpty('vat', …)` dans
`reports.api.ts:177-183`), qui testent désormais aussi l'absence de soldes.

**AC 5 — L'API et les types.** Les champs (`writeOffRows`, `totalVatWriteOff`, `totalVatDueNet`) dans la réponse de
`GET /api/v1/reports/vat` et dans `reports.types.ts`. Les libellés de l'écran dans les 4 locales, `sitesTotal`
recompté.

**AC 6 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Décompte TVA* : les diminutions (escompte, perte) retranchées
par taux dans la période du solde ; les limites (AC 8) ; la phrase « à partir des écritures réellement comptabilisées »
corrigée au passage (elle est fausse : la TVA due vient des factures — #390). ⚠️ **Sites que le patch rend faux**
(validation P2) : `:1656` (« Solde net dû à l'AFC : TVA due − TVA récupérable » → sur la TVA due **nette**), `:1666`
(« Le contrôle porte sur les écritures des factures retenues » → et des **soldes**), `:2125` (même phrase au glossaire) ;
`README.md:44` (« solde net dû à l'AFC ») relu. PDF régénéré, contrôlé aplati.
CHANGELOG `[0.12.1]` : l'entrée d2a (« le rapport TVA ne retranche pas encore … ») mise à jour ; #384 fermée.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- `kesh-report` (base) : une facture à deux taux, un escompte soldé dans la période → `write_off_rows` par taux,
  `total_vat_due_net`, `vat_balance`, **delta nul** ; un solde **hors** période → ignoré ; un solde de frais bancaires
  (`[]`) → aucune ligne ; un solde d'une **autre société** → ignoré ; un solde **annulé** → disparaît ;
- le delta reste nul sans solde (non-régression) ;
- JSON illisible → erreur ;
- rendus CSV et PDF : la section et la TVA nette avec des soldes, leur absence sans soldes, et **chacun des deux gardes
  Rust** (`csv.rs:377`, `pdf.rs:1080`) : sans vente ni récupérable, mais avec un solde → rendu **non vide** ;
- Vitest de la vue (`VatReportView.test.ts`, à créer) : sans soldes, section et TVA nette **absentes** ; avec soldes,
  présentes ; rapport vide ; bandeau d'écart inchangé ; et `isReportEmpty('vat', …)` : **pas de vente, un solde → pas
  vide** ; un test d'API sur la forme de la réponse ;
- JSON de forme fausse inséré en SQL → `CorruptData` (500 à l'API) ;
- **E2E** : `reports.spec.ts:234-237` ancré (`exact: true` ou `data-testid`) ; `invoice-write-off.spec.ts` étendu —
  après le solde **et avant son annulation** (un solde annulé quitte le rapport, AC 8), le rapport TVA de la période
  montre la section des soldes (le seul test qui voit les trois champs
  traverser la frontière HTTP).

**AC 8 — Limites assumées** (écrites au manuel et dans le code) :
- un solde **annulé** disparaît du rapport de **sa** période (la ligne est retirée, `invoice_settlements_write.rs:767`) et
  la contre-passation, datée du jour, n'est jointe nulle part : **la reprise de TVA consécutive à l'annulation
  n'apparaît dans aucun décompte, et l'alerte d'écart reste muette** — si la période du solde était déjà déclarée, la
  reprise est **à reporter à la main**. Même comportement que l'avoir (`user-manual.tex:1670-1680`), lié à #390 ;
- n'apparaissent pas les parts sans TVA : la part à **0 %** (lignes exonérées) et toute part à taux positif dont la TVA
  s'arrondit à 0.00 (très petit escompte, `vat.rs:193-198`) ;
- le delta suppose le compte de TVA due **stable** (limite L-1 existante), et **distinct** du compte d'une nature : un
  compte de nature réglé sur le compte de TVA due ferait compter son débit dans `soldes`.

## Tasks / Subtasks

- [ ] **T1 — le calcul** (AC 1, AC 2, AC 3), `vat_report.rs`.
- [ ] **T2 — les rendus** CSV et PDF (AC 4).
- [ ] **T3 — l'écran et les types** (AC 4, AC 5).
- [ ] **T4 — i18n** (AC 5).
- [ ] **T5 — manuel, CHANGELOG** (AC 6).
- [ ] **T6 — tests** (AC 7), dont les fixtures `VatReport` existantes complétées et l'E2E `reports.spec.ts` ancré.
- [ ] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ Recalculer la ventilation depuis les lignes de facture : elle est **figée** sur la ligne de solde (un changement
  d'algorithme ne doit pas réécrire l'histoire).
- ⛔ Changer `total_vat_due` ou `rows` : ils restent la TVA facturée — le net est un champ à part.
- ⛔ Basculer vers le grand livre (#390) : hors périmètre.
- ⛔ Lire un JSON illisible comme zéro.

### Modules

`kesh-report` (calcul, rendus), `kesh-db` (lecture de la ventilation, à côté de son écriture), `kesh-api` (mapping de
`CorruptData`), `frontend`, `kesh-i18n` (+ `docs`, `CHANGELOG`) — cinq modules de code, au seuil.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-02** — Créée (Guy : « oui, enchaîne »).
- **2026-10-02** — Validation P3 ciblée (Sonnet) : 3 MED, 2 LOW. Retenus : l'E2E vérifie le rapport **avant**
  l'annulation (M1) ; un test par garde Rust « vide » (M2) ; section et TVA nette conditionnelles **partout** (L1) ;
  `CorruptData` sur le précédent `TrialBalanceUnbalanced` (L2). **M3 — le signal de découpage HIGH → HIGH (P1 → P2) a
  été écarté par un motif que la règle ne prévoit pas** (« défauts distincts, cinq modules ») : la règle demande le
  découpage, et l'arbitrage revient au Project Lead — signalé à Guy en P2 sans attendre sa réponse, **reposé
  explicitement le 2026-10-02**. Le plafond de sévérité redescend en P3 (HIGH → HIGH → MED).
- **2026-10-02** — Validation P2 (Opus) : 1 HIGH, 3 MED, 5 LOW, retenus. L'E2E `reports.spec.ts` cherche « TVA due » et
  « Solde » par sous-chaîne — ancré, « TVA due nette » affichée seulement avec des soldes, et un scénario E2E « solde
  puis rapport » (H1) ; la limite du solde annulé dit sa conséquence fiscale — la reprise n'apparaît nulle part, l'alerte
  reste muette (M1) ; `ReportError::CorruptData` → 500, `kesh-api` au périmètre, lecture du JSON dans `kesh-db` (M2, L4) ;
  trois phrases du manuel et le README (M3) ; références, limites (parts arrondies à zéro, compte de nature confondu
  avec la TVA), états du test de la vue, libellés PDF, ligne de titre CSV (L1–L5). ⚠️ Signal de découpage (HIGH → HIGH) :
  **non découpée** — défauts distincts, aucun recyclé, cinq modules de code.
- **2026-10-02** — Validation P1 (Sonnet) : 1 HIGH, 1 MED, 2 LOW, retenus. Le garde « rapport vide » de l'**écran**
  (`isReportEmpty`, `reports.api.ts`) manquait — les trois gardes sont cités, et un test de la vue est prévu (H1) ; la
  formule du delta se contredisait sur le signe — `soldes` en `SUM(debit) − SUM(credit)`, vérification chiffrée écrite
  (M1) ; plage des types et second site de fixture PDF (L1, L2).

[#384]: https://github.com/guycorbaz/kesh/issues/384
