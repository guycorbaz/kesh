# Story 25.4-b2 : Le résiduel aux rappels — le rappel réclame le reste dû

Status: review

**Issue : [#416]** — cette story en livre la partie **rappels** ; la partie agrégats est la 25-4-b1
(PR #475). ⛔ **La PR de b2 porte `closes #416`**, titre ET corps (§ *Issue Tracking Rule*).

**Mère : `25-4-propager-le-residuel.md`** (`split`) — **source des faits et des arbitrages**, en
particulier § *Arbitrages du 2026-09-26/27 (Q1)*. **Sœurs** : 25-4-a (mergée, #472), 25-4-b1 (`done`,
PR #475). ⚠️ Branche `story/25-4-b2-residuel-aux-rappels` **empilée** sur celle de b1 : intégrer
`main` par merge après celui de #475.

⚠️ **Ne pas contester les arbitrages en validation** : en contester la mise en œuvre.

**Noms de fichier nus** — plusieurs existent dans deux crates ; dans cette fiche, sauf chemin
explicite : `pdf.rs` et `types.rs` = `crates/kesh-qrbill/src/` (pas `kesh-report`, pas
`kesh-import`) ; `dunning_eligibility.rs` = `crates/kesh-db/src/repositories/` (pas le fichier de
tests homonyme) ; `dunning_levels.rs` = **les deux** fichiers de ce nom, route et dépôt. *(Inventaire
fait en validation P5 : `find crates -name <nom>` sur chaque nom nu de la fiche.)*

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
2. **Le PDF d'un rappel est la facture elle-même** : quatre appelants de `render` (`invoice_email.rs:500`, `:722`, `:1141`, `invoice_pdf.rs:37`), tous
   `(pool, i18n, locale, company, invoice_id)`, aucun montant ni variante. Précédent de variante :
   l'**avoir**, qui surcharge `invoice-pdf-title` et `invoice-pdf-number` dans `i18n.entries`
   (`routes/credit_notes.rs:334-347`) — ⚠️ dans la langue de l'**installation**, qui ne convient pas
   à un rappel (AC 5).
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
(`entities/email_template.rs:63-74`) : phrase localisée (4 langues, **en Rust indexé par
`Language`**, sur le patron de `salutation_line`, `invoice_email.rs:109-150` — `build_reminder_vars`
est pur et n'a pas le bundle Fluent) disant le montant des frais
**cumulés** inclus dans `{totalDue}`, rendue **vide** quand ce cumul est nul. Les gabarits par défaut
des 4 langues et de tous les niveaux n'écrivent plus aucune phrase de frais en dur : ils emploient
`{feeNotice}`. `{reminderFee}` reste disponible pour les gabarits personnalisés. ⚠️ **Test qui rougira** :
`reminder_vars_ajoute_les_4_variables_rappel` (`invoice_email.rs:1494`) exige que les clés rendues
égalent la liste blanche (`:1507-1514`) — `build_reminder_vars` doit donc insérer `feeNotice`, et le
test est mis à jour, pas contourné.

**AC 3** — À frais cumulés nuls, le texte rendu par un gabarit **par défaut** ne contient ni le mot
« frais » (et ses équivalents de/it/en), ni un montant de frais. ⚠️ Un gabarit **personnalisé** qui
écrit `{reminderFee}` en toutes lettres reste de la responsabilité de l'administrateur : le manuel
admin le dit.

**AC 4** — `{amount}` reste le **TTC** de la facture (« montant de la facture ») : il n'est pas
réinterprété.

### Volet 2 — le PDF joint au rappel

**AC 5** — Le PDF joint à un rappel (unitaire et lot) est un **rappel**, pas la facture ; la pièce
jointe se nomme `rappel-{n}.pdf` et non plus `facture-{n}.pdf` (`invoice_email.rs:509`, `:1164` ; `:733`, l'envoi de facture, reste `facture-`) —
⚠️ l'E2E `dunning-roundtrip.spec.ts:81-82` (`^facture-.*\.pdf$`) suit. Titre
localisé « Rappel » (4 locales), sur le patron de surcharge de l'avoir. ⛔ **Dans la locale déjà
résolue** que `render` reçoit (langue du contact, `resolve_language`) — **pas** `state.config.locale`
comme l'avoir (`crates/kesh-api/src/routes/credit_notes.rs:335-346`), qui produirait un titre dans la langue de l'installation
sur un PDF dans celle du contact. Le téléchargement de la
facture (`GET …/pdf`) et l'envoi de facture (`POST …/send-email`) sont **inchangés**.

**AC 6** — Le PDF du rappel nomme **en toutes lettres le numéro de la facture d'origine** et, sous le
total TTC, porte : **déjà réglé**, puis **reste à payer** en gras. ⛔ **Pas de ligne « avoir »** : une facture
qui porte un avoir est `cancelled` (`crates/kesh-db/src/repositories/credit_notes.rs:586`), un avoir est refusé sur une
facture réglée même en partie (même fichier, `:325`), et le rendu comme l'éligibilité exigent `validated`
(`invoice_pdf_service.rs:102`, `dunning_eligibility.rs:86`) — une facture créditée ne reçoit jamais
de rappel ; la ligne serait du code mort. Si #471 lève un jour le refus, elle y reviendra. Les
lignes nulles ne s'affichent pas (même règle que les frais) ; une facture sans règlement ne
montre que le total, qui **est** le reste à payer *(arbitrage Q2, Guy, 2026-09-27)*.

**AC 7** — La QR du PDF de rappel porte le **reste dû** — le TTC s'il n'y a aucun règlement.
⛔ **Un seul arrondi** : `amount_due` (DECIMAL(19,4)) est arrondi **une fois**, à 2 décimales,
`MidpointAwayFromZero` (la stratégie de `pdf.rs:682-684` et `generator.rs:38-39`), et **cette même
valeur** nourrit le « reste à payer » imprimé, la QR et `{totalDue}`. Le refus de l'AC 9 porte sur la
valeur **arrondie** (≤ 0.00) : un reste brut de 0.004 est refusé, pas envoyé à une QR invalide.
⚠️ **Limite connue, préexistante, non corrigée ici** : les montants ont jusqu'à 4 décimales
(`routes/limits.rs:28`), et un reste brut de 10.0050 donne une QR à 10.01 ; le trop-perçu se juge sur
la valeur **brute** (`invoice_settlements_write.rs:167`, `routes/reconciliation.rs:1336`) : le paiement
exact de la QR serait refusé. La QR de **facture** a le même défaut (`generator.rs:38-39`). Tracé par
**[#476]**, à rattacher à la 25-4-c (#420, comparaison du rapprochement).
⛔ La **référence** (QRR ou aucune) et le **message non structuré** (`Facture {n}`) sont **identiques**
à ceux du PDF de facture, octet pour octet. Les frais **ne** sont **pas** dans la QR.

**AC 8** — Les frais cumulés figurent sur le PDF du rappel par une ligne **« Frais de rappel »**
(4 locales) portant la mention qu'ils **ne sont pas compris dans le bulletin de versement** ;
**absente** quand les frais sont nuls *(arbitrage Q1, Guy, 2026-09-27)*. ⚠️ **Largeur** : la colonne
des libellés du récapitulatif n'a que 50 mm (`col_unit` → `col_tot`, `pdf.rs:532-534`), soit ~23
caractères à 9 pt (calibrage `IDENTITY_MAX_CHARS`, `:202`) ; la mention en compte ~58 en français,
plus en allemand. Le libellé court (« Frais de rappel ») reste dans la colonne ; la **mention** part
sur une ligne à elle depuis `col_desc`, comptée dans la réserve. Un test borne sa longueur dans les
4 locales, sur le patron de `IDENTITY_MAX_CHARS` — les gardes ne surveillent que l'ordonnée.

**AC 9** — Reste dû ≤ 0 : le rappel est **refusé** avec un code dédié `REMINDER_NOTHING_DUE`
(unitaire : erreur HTTP ; lot : échec **par facture** dans la réponse, jamais d'erreur globale —
§ *Pattern batch*), au lieu d'un `INVOICE_NOT_PDF_READY` trompeur. L'aperçu le signale aussi. Le
frontend traduit le code (`frontend/src/lib/features/reminders/reminder-error-label.ts`).
⛔ **En lot, le refus doit être CLASSÉ** : aujourd'hui `send_one_batch_reminder` range **toute**
erreur de `render_reminder` en panne (`.map_err(|e| BatchItemError::infra("render reminder", …))`,
`invoice_email.rs:1118`), qui rend `DATABASE_ERROR` et journalise en `error!`. Le refus y sortirait
donc en fausse alerte d'infrastructure. Il sort en `BatchItemError::failed("REMINDER_NOTHING_DUE")`
(`:885`), sur le patron de `classify_render_error` (`:937`) et de son test
`classify_render_error_ne_deguise_pas_un_refus_en_panne` (`:1557`) — un test symétrique l'établit.
⛔ **Et sur le second site** : le lot recalcule le reste dans le rendu PDF (`:1140-1143`,
`classify_render_error`) ; un règlement enregistré entre les deux appels y ferait tomber le refus
dans le bras final `other => BatchItemError::infra("render pdf", …)` (`:947`). Le variant du refus
rejoint le `match` de `classify_render_error` **et** le tableau `metier` de son test (`:1557`) —
le doc-comment `:922-936` l'impose.
**Où vit le refus** : dans `render_reminder` (aperçu, lot) **et** dans la variante PDF — l'envoi
unitaire n'appelle pas `render_reminder` (le texte vient de l'aperçu, `:486-487`), seul son rendu PDF
(`:499-500`) recalcule le reste. Un variant `AppError::ReminderNothingDue` et sa clé
`error-reminder-nothing-due` (4 locales), sur le patron de `DunningPaused` (`crates/kesh-api/src/errors.rs:1312`) :
l'aperçu et l'unitaire affichent `err.message` ; `reminder-error-label.ts` ne sert qu'au compte-rendu
du lot (`ReminderBatchReport.svelte:30`).
L'éligibilité (`dunning_eligibility.rs:85-89`) **n'est pas** modifiée : l'état est hérité et rare
(25-4-b1, revue P1), et un refus nommé à l'envoi le rend visible, là où une exclusion en amont le
ferait disparaître de la liste sans dire pourquoi.

### Volet 3 — tests, textes

**AC 10** — Tests qui auraient échoué avant le patch, chacun avec un cas **partiellement réglé** et
une TVA **non nulle** :
- `render_reminder` : `totalDue` = reste + frais (et non TTC + frais) — la formule elle-même, pas
  une valeur passée à la main comme `reminder_vars_ajoute_les_4_variables_rappel`
  (`invoice_email.rs:1494-1520`) ;
- `{feeNotice}` vide à frais nuls, non vide sinon ; gabarits par défaut sans « frais » à zéro ;
- construction des entrées QR du rappel, **sans base** (patron `invoice_pdf_service.rs:553-655`) :
  `qr.amount` = reste dû ; `qr.reference` et `unstructured_message` **égaux** à ceux de la facture ;
- PDF de rappel : génération `Ok`, et le bloc réglé / reste mesuré selon la doctrine du dépôt
  (§ *Comment tester un PDF*, `16-3a-coordonnees-emetteur-pdf.md:416-429`) ;
- `REMINDER_NOTHING_DUE` en unitaire et en lot — en lot, **classé** en échec par facture et non en
  `DATABASE_ERROR` ; un reste brut de 0.004 est refusé ;
- le rendu du PDF de rappel au **nombre maximal de lignes** avec le bloc complet (trois lignes, dont
  la mention des frais — AC 8) ;
- un E2E Playwright : facture réglée en partie → aperçu du rappel montre le reste + frais, pas le TTC.

**AC 11** — Manuels : `user-manual.tex` § rappels (`:990-997`) dit ce que réclame un rappel et ce
que porte le PDF joint ; `admin-manual.tex:1200` n'écrit plus que la QR porte « le total TTC de la
facture d'origine », mais le **reste dû**, frais toujours exclus. ⚠️ **Les variables de rappel ne
sont documentées NULLE PART** : les deux manuels ne listent que les six variables de la facture
(`admin-manual.tex:1163-1167`, `user-manual.tex:906-909`). Les deux sites gagnent la liste des
variables propres au rappel — `{reminderLevel}`, `{reminderFee}` (frais du niveau), `{totalDue}`
(**reste dû + frais cumulés**), `{daysOverdue}`, `{feeNotice}` (vide sans frais) — et disent que
`{amount}` reste le TTC de la facture. Le rappel des CGV `dunning-cgv-hint` (`fr-CH/messages.ftl:1528`,
3 autres locales, repli `settings/dunning/+page.svelte:263`) ne dit plus « le QR de la facture
jointe » mais celui du **rappel joint**. ⚠️ Le manuel dit aussi, sans le corriger, qu'un client qui
paie `{totalDue}` (reste + frais) sera refusé en trop-perçu : conséquence des frais non comptabilisés,
tracée par **[#401]**. Et il dit la **limite du papier** : le PDF « Rappel » ne part qu'avec un rappel
**envoyé par e-mail** ; pour une sommation envoyée hors de Kesh (`user-manual.tex:1003`) ou un contact
sans e-mail, le PDF de la **facture** porte toujours le TTC complet dans sa QR — ne pas le joindre tel
quel à une facture partiellement réglée (**[#477]**). PDF régénérés et contrôlés **aplatis**. CHANGELOG `[0.12.1]` *Fixed*.

## Tasks / Subtasks

- [x] **T1 — texte** (AC 1-4) : `render_reminder`, `{feeNotice}` (liste blanche, phrase Rust par `Language` ×4,
  rendu serveur), gabarits par défaut ×4 langues × niveaux.
- [x] **T2 — PDF de rappel** (AC 5-8) : variante de `render` (paramètre explicite, pas de booléen
  muet, qui **porte les montants déjà calculés** — la construction des entrées QR reste testable sans
  base), champs optionnels dans `InvoicePdfData` sur le patron d'`origin_reference`, bloc sous le
  total dans `pdf.rs`, clés `I18N_KEYS`/`DEFAULT_EN` ajoutées **en fin** des deux tableaux, les deux
  appelants de rappel branchés.
- [x] **T3 — refus du reste nul** (AC 9) : variant `AppError`, clé `error-*` ×4, `render_reminder`
  et variante PDF, **deux** sites classés en lot, libellé du compte-rendu ×4.
- [x] **T4 — tests et mutations** (AC 10).
- [x] **T5 — textes** (AC 11).
- [x] **T6 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

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
- ⚠️ **Géométrie du PDF** : le bloc (jusqu'à trois lignes : déjà réglé, reste, frais)
  ajoute de la hauteur ; il **entre dans `recap_reserve`** (calculée avant la boucle des lignes,
  `pdf.rs:579-586`) et respecte les gardes `HeaderOverflow` / `TooManyLines` (`:527-529`,
  `:582-595`). ⛔ **Ne pas copier le bloc `payment_terms`** (`:700-708`), le plus proche en apparence :
  il n'est pas réservé et se contente de **clamper** à `content_floor + 5.0`, d'où un tassement
  silencieux au lieu d'un refus. Un test à nombre de lignes maximal **avec** le bloc complet le prouve.
- ⚠️ **`I18N_KEYS.len() == DEFAULT_EN.len()`** (`types.rs:276-279`) ne garantit pas l'appariement :
  ajouter en fin des deux tableaux, et le test positionnel `pdf.rs:1835-1852` doit suivre.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-api/src/routes/invoice_email.rs:161-230, 313-404, 412-562, 1029-1190` | variables, `render_reminder`, aperçu, unitaire, lot |
| `crates/kesh-api/src/routes/invoice_pdf_service.rs:90-345, 553-655` | `render`, entrées QR, tests sans base |
| `crates/kesh-api/src/routes/credit_notes.rs:334-347` | patron de surcharge du titre — ⚠️ **sauf sa locale** (`state.config.locale`), cf. AC 5 |
| `crates/kesh-qrbill/src/types.rs:120-159, 216-315` ; `pdf.rs:388-712` | données PDF, clés, rendu |
| `crates/kesh-db/src/entities/email_template.rs:63-74` ; `email_template_defaults.rs:71-215` | liste blanche, gabarits par défaut |
| `crates/kesh-db/src/repositories/invoice_settlements.rs:187-220` | `amount_due`, `amount_settled` — suffisants : aucune ligne « avoir » (AC 6) |
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
- **Signal de découpage P2 → P3 (MED → MED)** : *« ne découpe pas »* (Guy, 2026-09-27).
- **M5 — le rappel papier** : *« les 1er et 2ème rappels n'ont pas besoin d'être en recommandé »*,
  puis *« le troisième rappel, la sommation, sera envoyé par un autre moyen, pas par kesh (pour
  l'instant) »* (Guy, 2026-09-27). ⇒ **Hors périmètre** : l'impression du PDF de rappel (sommation,
  contact sans e-mail) est tracée par **[#477]** ; le manuel dit la limite (AC 11).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Clippy `too_many_arguments` sur `build_qrbill_inputs` (8 arguments avec `PdfDocument`) : `allow`
  explicite, comme `build_reminder_vars`.
- Débordement LaTeX de 6 pt sur le paragraphe des variables (`user-manual.tex`) : `sloppypar`. Celui de
  `admin-manual.tex:1163` (60 pt) est **préexistant** — premier élément de la liste, non touché.
- Le signe moins typographique (U+2212) devant « déjà réglé » n'existe pas dans l'encodage WinAnsi des
  polices intégrées : tiret ASCII.

### Completion Notes List

- **Deux écarts de mise en œuvre, qui servent l'intention mieux que la lettre** :
  - **Le titre « Rappel » ne passe pas par une surcharge de `i18n.entries`** (patron de l'avoir) mais
    par un champ `reminder: Option<ReminderPdf>` d'`InvoicePdfData` : `draw_invoice_section` choisit
    `invoice-pdf-reminder-title`, résolu par `build_i18n` dans la locale **passée à `render_document`**
    (celle du contact). Le piège de locale de l'AC 5 disparaît **par construction**.
  - **Les montants se calculent une fois** : `render_reminder` rend `(sujet, corps, ReminderAmounts)`,
    et le lot passe **ces mêmes montants** au PDF — le texte et la QR ne peuvent pas diverger, même si
    un règlement arrive entre les deux. Le variant reste classé dans `classify_render_error` (AC 9,
    second site), en défense : `build_qrbill_inputs` refait `reminder_amount_due` sur les montants reçus.
- **Où vit le refus** : `reminder_amount_due` (arrondi `MidpointAwayFromZero`, puis refus ≤ 0.00) —
  appelé par `reminder_amounts` (aperçu, lot, et unitaire **avant le SMTP**) et par
  `build_qrbill_inputs`. `AppError::ReminderNothingDue` (422, `REMINDER_NOTHING_DUE`).
- **`{feeNotice}`** : Rust indexé par `Language` (patron `salutation_line`), précédé d'une espace,
  vide à frais cumulés nuls ; les 16 bras des gabarits par défaut l'emploient, plus aucune phrase de
  frais en dur. `{totalDue}` passe de « montant total dû » à « montant dû ».
- **Mutation exécutée** : retirer la hauteur du bloc de `recap_reserve` fait rougir
  `reminder_block_is_reserved_in_the_capacity_guard` (« la réserve ne sert à rien ») — tuée,
  restaurée par `sed` (qui date le fichier du présent, cf. l'incident de mutation de la b1). Les
  autres tests échoueraient **par construction** sur le code d'avant (montants 128.10 / 972.90,
  `facture-…pdf`, `INVOICE_NOT_PDF_READY`) ; non mutés un à un.
- **Tests ajoutés**, recomptés (`#[test]`/`#[sqlx::test]`/`it(` ajoutés dans le diff) : `kesh-db` 1,
  `kesh-qrbill` 4, `kesh-api` 5 unitaires (dont `reminder_vars_ajoute_les_4_variables_rappel`
  **renommé** et étendu, non compté) + 3 d'intégration, `kesh-i18n` 1 — **14** backend ; frontend 1
  Vitest ; Playwright 1 test ajouté (`reminders.spec.ts`), 1 assertion changée
  (`dunning-roundtrip.spec.ts`).
- **Sept clés i18n × 4 locales** : cinq `invoice-pdf-*`, `error-reminder-nothing-due`,
  `reminders-error-nothing-due` ; `dunning-cgv-hint` ×4 et son repli Svelte rectifiés.

### Gates

- Backend : base remise à zéro, `scripts/test-fast.sh` **2508 / 2508** (2494 + 14).
- Frontend : `check` 0 erreur (27 avertissements, préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **835 / 835** (834 + 1), build OK.
- E2E : ciblé `reminders.spec.ts` + `dunning-roundtrip.spec.ts` **10 / 10**. Complet, deux runs sur
  `kesh_e2e` reconstruite :
  - run 1 : **214 / 20 / 19** — les 7 KF-029, `product-revenue-account:133` (pollution répertoriée) et
    **12 hors liste**, tous `page.fill('#username')` sur un `/login` en « Erreur 500 » : **12 / 12 verts
    rejoués seuls**. Second passage du symptôme vu en b1 ⇒ **KF-053 ouverte ([#478])**, ajoutée à
    `docs/testing.md` § « Les échecs attendus » ;
  - run 2 : **224 / 10 / 19** — les 7 KF-029, `sidebar-navigation:75` (**KF-046, #424** : rouge rejoué
    seul, déterministe), `bank-accounts-crud:112` et `setup:75` (**verts rejoués seuls** : pollution).
    Conforme à la baseline.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-qrbill/src/types.rs`, `lib.rs` | `ReminderPdf`, champ `reminder`, 5 clés `I18N_KEYS`/`DEFAULT_EN` |
| `crates/kesh-qrbill/src/pdf.rs` | titre, bloc sous le total, réserve, `REMINDER_NOTE_MAX_CHARS`, 4 tests |
| `crates/kesh-qrbill/tests/golden_test.rs` | `reminder: None` |
| `crates/kesh-api/src/routes/invoice_pdf_service.rs` | `PdfDocument`, `ReminderAmounts`, `reminder_amount_due`, `reminder_amounts`, `render_document`, 3 tests |
| `crates/kesh-api/src/routes/invoice_email.rs` | `fee_notice`, `render_reminder`, unitaire, lot, classement, pièce jointe `rappel-`, 2 tests |
| `crates/kesh-api/src/routes/credit_notes.rs` | `reminder: None` |
| `crates/kesh-api/src/errors.rs` | `ReminderNothingDue` |
| `crates/kesh-api/tests/invoice_send_email_e2e.rs` | 3 tests d'intégration |
| `crates/kesh-db/src/entities/email_template.rs` | `feeNotice` en liste blanche |
| `crates/kesh-db/src/entities/email_template_defaults.rs` | 16 bras, 1 test |
| `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`, `src/loader.rs` | 7 clés, `dunning-cgv-hint`, 1 test |
| `frontend/src/lib/features/reminders/reminder-error-label.ts` / `.test.ts` | `REMINDER_NOTHING_DUE` |
| `frontend/src/routes/(app)/settings/dunning/+page.svelte` | repli `dunning-cgv-hint` |
| `frontend/tests/e2e/reminders.spec.ts`, `dunning-roundtrip.spec.ts` | reste dû ; `rappel-…pdf` |
| `docs/manual/fr/user-manual.tex` / `.pdf`, `admin-manual.tex` / `.pdf` | ce que réclame un rappel, variables, QR, #401, #477 |
| `CHANGELOG.md` | *Fixed* |

## Change Log

- **2026-09-27** — **Dev** : T1-T5 faits (§ *Completion Notes*) ; deux écarts de mise en œuvre écrits
  (titre par champ plutôt que par surcharge de locale ; montants calculés une fois et partagés).
  Gates verts (§ *Gates*) ; **KF-053 (#478) ouverte** pour le symptôme E2E revenu. Story en `review`.
- **2026-09-27** — **M5 arbitré** (Guy) : la sommation part hors de Kesh pour l'instant ; l'impression du
  PDF de rappel sort du périmètre, tracée par **#477** (ouverte) ; l'AC 11 ajoute la limite au manuel.
  Changement de texte seul, issu d'un arbitrage et non d'un finding : pas de passe supplémentaire.
- **2026-09-27** — **Validation P5 ciblée** (Haiku, `d06b5c6c`, prompt `25-4-b2-validate-prompt-p5.md`)
  — 2 findings rendus MEDIUM, **reclassés LOW** par l'orchestrateur : `pdf.rs` et `dunning_levels.rs`
  nus et homonymes, mais sans numéro de ligne, et le contexte désigne le bon fichier (`pdf.rs` est
  qualifié `kesh-qrbill` dans « Où regarder » ; « aucune écriture » vaut pour les deux
  `dunning_levels.rs`). Traités **comme classe** : inventaire de tous les noms nus de la fiche et de
  leurs homonymes (`pdf.rs`, `types.rs`, `dunning_eligibility.rs`, `dunning_levels.rs` ; `errors.rs`
  et `credit_notes.rs` ne restent nus que dans le Change Log), et une convention en tête de fiche.
  ⛔ **Boucle close en 5 passes** : `1H/3M/2L → 0 (+1M orchestrateur) → 5M/5L → 1M/2L → 0 >LOW`,
  rotation Sonnet → Haiku → Opus → Sonnet → Haiku, deux passes ciblées ; toutes les corrections sur la
  fiche. ⚠️ **M5 (le rappel papier) reste en attente d'arbitrage** : s'il ajoute du périmètre, une
  passe de plus sera due.
- **2026-09-27** — **Validation P4 ciblée** (Sonnet, `6197bffd`, prompt `25-4-b2-validate-prompt-p4.md`)
  — **1 MEDIUM, 2 LOW**, tous de référence : `credit_notes.rs:586`/`:325` sans chemin, alors que deux
  fichiers portent ce nom (le contenu est dans `kesh-db/src/repositories/`) ; `invoice_email.rs:510`/
  `:1161` décalés (`:509`/`:1164`) ; `[#401]` défini mais jamais invoqué. Symptôme grepé (nom de
  fichier nu) : **deux sites de plus** que la passe, `credit_notes.rs:335-346` et `errors.rs:1312`,
  eux aussi présents dans deux crates — qualifiés. Cinq axes exercés, commandes citées ; les
  corrections de P3 jugées justes et sans contradiction.
- **2026-09-27** — **Validation P3** (Opus, protocole complet, prompt `25-4-b2-validate-prompt-p3.md`)
  — **5 MEDIUM, 5 LOW**, tous sur la fiche ; les citations clés vérifiées par `grep -nF`. Corrigés :
  M1 la ligne « avoir » et `amount_credited` retirées — une facture créditée est `cancelled` et ne
  reçoit jamais de rappel ; M2 la largeur de la mention des frais (50 mm de colonne, ~58 caractères) ;
  M3 le refus classé aussi dans `classify_render_error` (second site du lot) ; M4 l'arrondi loin de
  zéro fait refuser le paiement exact de la QR — limite préexistante écrite dans l'AC 7, tracée par
  **#476** (ouverte) ; L1 où vit le refus, variant et clé ; L2 `{feeNotice}` en Rust par `Language`,
  test qui rougira nommé ; L3 quatre appelants de `render`, pas trois ; L4 nom de la pièce jointe et
  `dunning-cgv-hint` ; L5 la conséquence de `{totalDue}` = reste + frais, citée avec **#401**.
  ⚠️ **Deux points remontés à Guy** : M5 (le rappel papier n'a aucun PDF juste) et le **signal de
  découpage** — P2 (1M, orchestrateur) → P3 (5M) : sévérité égale (§ *Règle de splitting
  préventif*).
- **2026-09-27** — **Validation P2** (Haiku, prompt `25-4-b2-validate-prompt-p2.md`) — **0 finding**
  rendu, sans aucune commande citée ; l'axe 10 lisait de travers `amount_credited` (« LOW accepté »,
  alors que la fiche prescrit de la créer). ⛔ **Repris par l'orchestrateur** (§ *un 0 finding se
  vérifie comme un finding*) : forme de `BatchItemError::failed` conforme (`invoice_email.rs:885`) ;
  aucun autre site ne dit `totalDue` hors des six fichiers connus ; mais **1 MEDIUM** sur l'axe 8
  qu'elle déclarait exercé — l'AC 11 renvoyait à une documentation des variables de rappel qui
  **n'existe pas** (les manuels ne listent que celles de la facture). AC 11 rectifié.
- **2026-09-27** — **Validation P1** (Sonnet, prompt `25-4-b2-validate-prompt-p1.md`) — **1 HIGH,
  3 MEDIUM, 2 LOW**, tous des défauts de la fiche. HIGH : en lot, le refus du reste nul serait sorti
  en `DATABASE_ERROR` — `render_reminder` y est classé en panne (`invoice_email.rs:1118`) ; AC 9
  exige désormais la classification. MEDIUM : la locale du titre (le précédent de l'avoir prend celle
  de l'installation) ; la géométrie (le bloc le plus proche, `payment_terms`, clampe au lieu de
  réserver) ; l'arrondi (un seul, partagé par le texte, le PDF et la QR). LOW : l'éligibilité reste
  inchangée, motif écrit ; l'avoir n'a pas de fonction scalaire. Neuf axes exercés, références toutes
  exactes ; périmètre recompté à cinq modules.
- **2026-09-27** — Créée. Inventaire vérifié par deux explorations et contrôle direct des citations ;
  le point ouvert de la mère sur les frais est clos par la règle existante (non comptabilisés, hors
  QR) ; message de la QR rectifié (`Facture {n}`, pas le numéro seul) ; deux questions à Guy.
  Tranchées le jour même : ligne « Frais de rappel » sur le PDF (Q1), « déjà réglé » masqué à zéro
  (Q2).

[#387]: https://github.com/guycorbaz/kesh/issues/387
[#416]: https://github.com/guycorbaz/kesh/issues/416
[#476]: https://github.com/guycorbaz/kesh/issues/476
[#401]: https://github.com/guycorbaz/kesh/issues/401
[#477]: https://github.com/guycorbaz/kesh/issues/477
[#478]: https://github.com/guycorbaz/kesh/issues/478
