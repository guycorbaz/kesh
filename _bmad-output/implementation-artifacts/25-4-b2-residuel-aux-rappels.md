# Story 25.4-b2 : Le résiduel aux rappels — le rappel réclame le reste dû

Status: ready-for-dev

**Issue : [#416]** — cette story en livre la partie **rappels** ; la partie agrégats est la 25-4-b1
(PR #475). ⛔ **La PR de b2 porte `closes #416`**, titre ET corps (§ *Issue Tracking Rule*).

**Mère : `25-4-propager-le-residuel.md`** (`split`) — **source des faits et des arbitrages**, en
particulier § *Arbitrages du 2026-09-26/27 (Q1)*. **Sœurs** : 25-4-a (mergée, #472), 25-4-b1 (`done`,
PR #475). ⚠️ Branche `story/25-4-b2-residuel-aux-rappels` **empilée** sur celle de b1 : intégrer
`main` par merge après celui de #475.

⚠️ **Ne pas contester les arbitrages en validation** : en contester la mise en œuvre.

## Story

En tant que gérant d'une PME,
je veux qu'un rappel réclame ce que le client **doit encore** — dans le texte, dans la QR-facture et
sur le PDF joint —,
afin qu'un client qui a déjà payé une partie ne soit ni relancé pour le montant entier, ni amené à
le payer une seconde fois en scannant la QR.

## Le défaut, vérifié dans le code

Depuis la 24-2, une facture se règle en plusieurs fois. Les rappels l'ignorent :

| Site | Ce qu'il réclame |
|---|---|
| Texte — `render_reminder`, `crates/kesh-api/src/routes/invoice_email.rs:313-360` | `total_due = ttc + other_fees + *level_fee` (`:334`) — **TTC**, ni règlements ni avoir ; doc-comment `:312` idem |
| QR du PDF joint — `invoice_pdf_service::render`, `crates/kesh-api/src/routes/invoice_pdf_service.rs:90` | `amount: Some(total_ttc)` (`:243`) : un client qui scanne la QR paie **à nouveau le montant entier** |
| PDF joint — rappel unitaire `invoice_email.rs:499-500`, lot `:1140-1143` | la **facture telle quelle** : titre « Facture », aucun bloc réglé / reste (`kesh-qrbill/src/pdf.rs:682-698` n'imprime que « Total TTC ») |
| Gabarits par défaut — `crates/kesh-db/src/entities/email_template_defaults.rs:71-215` | phrases de frais **inconditionnelles** : niveau 2 « Des frais de rappel de {reminderFee} ont été ajoutés » (`:87`), niveau 3 « (frais de rappel de {reminderFee} inclus) » (`:96-97`) — à frais nuls, « frais de 0.00 » |

Une facture de 1 081.— réglée de 900.— est relancée pour **1 081.— + frais**, avec une QR à
1 081.—. Aucun test ne combine règlement partiel et rappel (vérifié : aucun E2E ni test Rust).

### Ce que l'inventaire a établi, et qui fixe les choix

1. **Les frais de rappel ne sont PAS comptabilisés** — aucune écriture dans `invoice_email.rs`,
   `dunning_reminders.rs`, `dunning_levels.rs`, `invoice_reminders.rs`. C'est **documenté** :
   `admin-manual.tex:1200` (« affichés mais non comptabilisés … ne figurent pas sur la QR-facture »)
   et `dunning-cgv-hint` (`fr-CH/messages.ftl:1528`). Un virement « reste + frais » serait d'ailleurs
   **refusé** comme trop-perçu (`invoice_settlements_write.rs:165-171`,
   `routes/reconciliation.rs:1326-1345`). ⇒ **La QR porte le reste dû, sans les frais.** Le point
   ouvert de la mère (§ *Point ouvert … les frais de rappel*) est ainsi **clos par la règle
   existante** ; comptabiliser les frais n'est pas dans cette story.
2. **Le PDF d'un rappel est la facture elle-même** : trois appelants de `render`, tous
   `(pool, i18n, locale, company, invoice_id)`, aucun montant ni variante. Précédent de variante :
   l'**avoir**, qui surcharge `invoice-pdf-title` et `invoice-pdf-number` dans `i18n.entries`
   (`routes/credit_notes.rs:334-347`).
3. **Le message non structuré de la QR vaut `Facture {n}`** (`invoice_pdf_service.rs:246-250`), pas
   le numéro seul comme l'écrit la mère (`:115-117`). Il est émis même avec une QRR. **Il reste
   identique**, comme la référence (`build_qrr(company.id, invoice.id)`, `:218-229`) : c'est ce que
   lit le rapprochement.
4. **La QR refuse un montant ≤ 0** (`kesh-qrbill/src/validation.rs:22-29`, `:435-445`) →
   `InvoiceNotPdfReady` (400), `INVOICE_NOT_PDF_READY` en lot (`invoice_email.rs:944-946`). Un reste dû
   ≤ 0 sur une facture `validated` sans `paid_at` n'existe que par données héritées (25-4-b1, revue
   P1) — mais il ferait échouer le rappel sur un message trompeur.
5. **L'envoi unitaire ne recalcule pas le texte** : il envoie le sujet et le corps que le client a
   pu modifier depuis l'aperçu (`invoice_email.rs:486-487`). Corriger `render_reminder` corrige
   l'**aperçu** et le **lot** ; l'unitaire suit par l'aperçu.
6. **Aucun archivage** du PDF envoyé ([#387], ouverte). Le rappel pour le montant restant n'est pas
   la facture réémise (recadrage de Guy, mère `:105-109`) : #387 n'est pas touchée.
7. **Le moteur de gabarits ne connaît aucune condition** (`kesh-core/src/email_template_engine.rs`,
   substitution `{var}` en une passe). « À zéro, rien ne s'affiche » ne peut donc passer que par une
   **variable rendue vide** par le serveur.

## Acceptance Criteria

### Volet 1 — le texte du rappel

**AC 1** — `render_reminder` calcule `total_due = reste dû + frais des autres niveaux + frais du
niveau`, le reste dû venant d'`invoice_settlements::amount_due` (forme **scalaire** : une facture à
la fois). ⛔ Aucune réécriture de la formule. Le doc-comment `:312` suit.

**AC 2** — Une variable **`{feeNotice}`** rejoint la liste blanche du type `InvoiceReminder`
(`entities/email_template.rs:63-74`) : phrase localisée (4 locales) disant le montant des frais
**cumulés** inclus dans `{totalDue}`, rendue **vide** quand ce cumul est nul. Les gabarits par défaut
des 4 langues et de tous les niveaux n'écrivent plus aucune phrase de frais en dur : ils emploient
`{feeNotice}`. `{reminderFee}` reste disponible pour les gabarits personnalisés.

**AC 3** — À frais cumulés nuls, le texte rendu par un gabarit **par défaut** ne contient ni le mot
« frais » (et ses équivalents de/it/en), ni un montant de frais. ⚠️ Un gabarit **personnalisé** qui
écrit `{reminderFee}` en toutes lettres reste de la responsabilité de l'administrateur : le manuel
admin le dit.

**AC 4** — `{amount}` reste le **TTC** de la facture (« montant de la facture ») : il n'est pas
réinterprété.

### Volet 2 — le PDF joint au rappel

**AC 5** — Le PDF joint à un rappel (unitaire et lot) est un **rappel**, pas la facture : titre
localisé « Rappel » (4 locales), sur le patron de surcharge de l'avoir. Le téléchargement de la
facture (`GET …/pdf`) et l'envoi de facture (`POST …/send-email`) sont **inchangés**.

**AC 6** — Le PDF du rappel nomme **en toutes lettres le numéro de la facture d'origine** et, sous le
total TTC, porte : **déjà réglé** (et **avoir** s'il y en a un), puis **reste à payer** en gras. Les
lignes nulles ne s'affichent pas (même règle que les frais) ; une facture sans règlement ni avoir ne
montre que le total, qui **est** le reste à payer *(arbitrage Q2, Guy, 2026-09-27)*.

**AC 7** — La QR du PDF de rappel porte le **reste dû** — le TTC s'il n'y a aucun règlement.
⛔ La **référence** (QRR ou aucune) et le **message non structuré** (`Facture {n}`) sont **identiques**
à ceux du PDF de facture, octet pour octet. Les frais **ne** sont **pas** dans la QR.

**AC 8** — Les frais cumulés figurent sur le PDF du rappel par une ligne **« Frais de rappel »**
(4 locales) portant la mention qu'ils **ne sont pas compris dans le bulletin de versement** ;
**absente** quand les frais sont nuls *(arbitrage Q1, Guy, 2026-09-27)*.

**AC 9** — Reste dû ≤ 0 : le rappel est **refusé** avec un code dédié `REMINDER_NOTHING_DUE`
(unitaire : erreur HTTP ; lot : échec **par facture** dans la réponse, jamais d'erreur globale —
§ *Pattern batch*), au lieu d'un `INVOICE_NOT_PDF_READY` trompeur. L'aperçu le signale aussi. Le
frontend traduit le code (`frontend/src/lib/features/reminders/reminder-error-label.ts`).

### Volet 3 — tests, textes

**AC 10** — Tests qui auraient échoué avant le patch, chacun avec un cas **partiellement réglé** et
une TVA **non nulle** :
- `render_reminder` : `totalDue` = reste + frais (et non TTC + frais) — la formule elle-même, pas
  une valeur passée à la main comme `reminder_vars_ajoute_les_4_variables_rappel`
  (`invoice_email.rs:1497-1520`) ;
- `{feeNotice}` vide à frais nuls, non vide sinon ; gabarits par défaut sans « frais » à zéro ;
- construction des entrées QR du rappel, **sans base** (patron `invoice_pdf_service.rs:553-655`) :
  `qr.amount` = reste dû ; `qr.reference` et `unstructured_message` **égaux** à ceux de la facture ;
- PDF de rappel : génération `Ok`, et le bloc réglé / reste mesuré selon la doctrine du dépôt
  (§ *Comment tester un PDF*, `16-3a-coordonnees-emetteur-pdf.md:416-429`) ;
- `REMINDER_NOTHING_DUE` en unitaire et en lot ;
- un E2E Playwright : facture réglée en partie → aperçu du rappel montre le reste + frais, pas le TTC.

**AC 11** — Manuels : `user-manual.tex` § rappels (`:990-997`) dit ce que réclame un rappel et ce
que porte le PDF joint ; `admin-manual.tex:1200` n'écrit plus que la QR porte « le total TTC de la
facture d'origine », mais le **reste dû**, frais toujours exclus ; `{feeNotice}` documentée avec les
variables de gabarit. PDF régénérés et contrôlés **aplatis**. CHANGELOG `[0.12.1]` *Fixed*.

## Tasks / Subtasks

- [ ] **T1 — texte** (AC 1-4) : `render_reminder`, `{feeNotice}` (liste blanche, clés Fluent ×4,
  rendu serveur), gabarits par défaut ×4 langues × niveaux.
- [ ] **T2 — PDF de rappel** (AC 5-8) : variante de `render` (paramètre explicite, pas de booléen
  muet), champs optionnels dans `InvoicePdfData` sur le patron d'`origin_reference`, bloc sous le
  total dans `pdf.rs`, clés `I18N_KEYS`/`DEFAULT_EN` ajoutées **en fin** des deux tableaux, les deux
  appelants de rappel branchés.
- [ ] **T3 — refus du reste nul** (AC 9) : code, unitaire, lot, aperçu, libellé frontend ×4.
- [ ] **T4 — tests et mutations** (AC 10).
- [ ] **T5 — textes** (AC 11).
- [ ] **T6 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Mettre les frais dans la QR** : ils ne sont pas comptabilisés ; le virement serait refusé comme
  trop-perçu (§ inventaire, point 1).
- ⛔ **Toucher la référence ou le message de la QR** : le rapprochement les lit.
- ⛔ **Modifier le PDF de la facture** (téléchargement, envoi, renvoi) : seule la pièce jointe d'un
  **rappel** change.
- ⛔ **Réécrire le reste dû à la main** : `amount_due` existe ; il ne prend pas de `company_id` — le
  scoping est déjà fait par le chargement de la facture (`find_by_id_with_lines(pool, company.id, …)`).
- ⛔ **Un booléen `is_reminder` passé à `render`** : un paramètre qui dit ce qu'il porte (enum ou
  struct d'options), sans quoi l'appel `render(…, true)` ne se relit pas.
- ⚠️ **Géométrie du PDF** : le bloc ajoute de la hauteur ; il doit entrer dans la réserve
  `recap_reserve` et respecter les gardes `HeaderOverflow` / `TooManyLines` (`pdf.rs:527-529`,
  `:582-595`).
- ⚠️ **`I18N_KEYS.len() == DEFAULT_EN.len()`** (`types.rs:276-279`) ne garantit pas l'appariement :
  ajouter en fin des deux tableaux, et le test positionnel `pdf.rs:1835-1852` doit suivre.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-api/src/routes/invoice_email.rs:161-230, 313-404, 412-562, 1029-1190` | variables, `render_reminder`, aperçu, unitaire, lot |
| `crates/kesh-api/src/routes/invoice_pdf_service.rs:90-345, 553-655` | `render`, entrées QR, tests sans base |
| `crates/kesh-api/src/routes/credit_notes.rs:334-347` | patron de surcharge du titre |
| `crates/kesh-qrbill/src/types.rs:120-159, 216-315` ; `pdf.rs:388-712` | données PDF, clés, rendu |
| `crates/kesh-db/src/entities/email_template.rs:63-74` ; `email_template_defaults.rs:71-215` | liste blanche, gabarits par défaut |
| `crates/kesh-db/src/repositories/invoice_settlements.rs:173-218` | `amount_due`, `amount_settled` |
| `crates/kesh-db/src/repositories/invoice_reminders.rs:65-89` | frais cumulés |
| `docs/manual/fr/user-manual.tex:990-997` ; `admin-manual.tex:1183-1200` | textes |

### Gardes-fous du dépôt

- Aucune migration prévue. Si une devenait nécessaire : § *Migration breaking policy* (P5 à P8).
- `crates/kesh-db/` touché (entités) : gate complet en fin de boucle de revue.
- Modules touchés : `kesh-qrbill`, `kesh-api`, `kesh-db`, `kesh-i18n`, `frontend` — **cinq**, à la
  limite de la règle de découpage (> 5) : ne pas en ajouter sans le signaler.

## ✅ Arbitrages de Guy (2026-09-27)

- **Q1 — les frais sur le PDF du rappel** : *« oui, rajouter une ligne "Frais de rappel" »*. Une
  ligne « Frais de rappel : 20.— (non compris dans le bulletin de versement) », absente à zéro — la
  QR reste au reste dû (AC 7, AC 8).
- **Q2 — « déjà réglé » à zéro** : *« ok »*. Sans règlement ni avoir, le PDF ne montre que le total,
  qui est le reste à payer (AC 6).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-27** — Créée. Inventaire vérifié par deux explorations et contrôle direct des citations ;
  le point ouvert de la mère sur les frais est clos par la règle existante (non comptabilisés, hors
  QR) ; message de la QR rectifié (`Facture {n}`, pas le numéro seul) ; deux questions à Guy.
  Tranchées le jour même : ligne « Frais de rappel » sur le PDF (Q1), « déjà réglé » masqué à zéro
  (Q2).

[#387]: https://github.com/guycorbaz/kesh/issues/387
[#416]: https://github.com/guycorbaz/kesh/issues/416
