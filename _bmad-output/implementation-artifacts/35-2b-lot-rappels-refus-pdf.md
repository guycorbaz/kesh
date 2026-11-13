# Story 35-2b : le rapport d'un lot de rappels dit pourquoi le PDF d'une facture est refusé (#955)

Status: review

*Fiche née le 2026-11-12 du découpage de la 35-2 à la validation P2 (C-e350-9), par l'agent de kickoff de l'Epic 35
(Opus 5.5), worktree `kesh-e35`. Elle **part de la 35-2a fusionnée**, dont elle reprend le message localisé (la 35-2a garde
`AppError::InvoiceNotPdfReady(String)`, D3 ; aucun contrat `details` lisible par machine, C-e350-11). Release **0.17.7** ; Guy n'emploie pas encore les rappels (première facture client avant le
2026-11-30) : elle suit la 35-2a sans échéance propre. **Aucune migration.** Lignes lues sur **`release/0.17.6`**
(`4e1c768a`), identiques sur **`main`** (`4d39f377`, la 0.17.6 publiée ; `git diff --stat 4e1c768a 4d39f377 -- crates/kesh-api
frontend/src` vide, validation P1). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -9, -11, -46) ;
`e35-choix-autonomes-e352b.md` pour la story, à créer au premier choix.*

## Story

En tant que titulaire qui envoie un lot de rappels, quand une facture du lot est écartée parce que son PDF est refusé, je
veux lire dans le rapport **ce qu'il faut corriger**, comme sur la fiche de la facture, et non « Facture non imprimable en
PDF » seul.

## Ce qui se lit au code (`release/0.17.6`, identique sur `main`)

- La boucle du lot rend chaque PDF (`crates/kesh-api/src/routes/invoice_email.rs:1260-1269`) et classe l'échec par
  `classify_render_error` (`:1023-1036`) : `InvoiceNotPdfReady`, `InvoiceTooManyLinesForPdf` et
  `InvoicePdfHeaderOverflow` deviennent tous `BatchItemError::failed("INVOICE_NOT_PDF_READY")`, **sans `details`**, alors
  que `BatchItemError::failed_with(code, details)` existe (`:964-990`) et que `FailedReminder` porte un champ `details`
  (`:944-948`). Le bras final rend `DATABASE_ERROR` (règle du doc-comment `:1010-1022`) ; test
  `classify_render_error_ne_deguise_pas_un_refus_en_panne` (`:1749`) ; `BatchItemError` n'expose en test que `code()`
  (`:985-989`). Le commentaire du bras (`:1027-1029`) nomme les quatre causes.
- Le chemin du lot est **unique** : `git grep -n 'render_document('` rend quatre appels (`invoice_email.rs:560`, `:1260`,
  `issued_invoice_pdf.rs:151`, `:256`), un seul dans le lot ; `render_reminder` (`:362-405`) ne produit pas
  `InvoiceNotPdfReady`, et `classify_reminder_render_error` (`:1062`) ne change pas.
- Les deux autres refus du même code ont, sur la route unitaire, une phrase qui dit quoi corriger, construite **dans le bras
  de `IntoResponse`** : `InvoiceTooManyLinesForPdf(n)` (`crates/kesh-api/src/errors.rs:2144-2158`, `t_args(
  "error-invoice-too-many-lines-for-pdf", …)`, argument `count`) et `InvoicePdfHeaderOverflow` (`:2159-2168`,
  `t("error-invoice-pdf-header-overflow", …)`).
- Le montage : `Contact::structured_address` (`crates/kesh-db/src/entities/contact.rs:15-24`) ne rend `None` que si la rue,
  le numéro, le NPA et la localité sont **tous** vides ; un contact sans e-mail est refusé avant le rendu
  (`CONTACT_EMAIL_MISSING`, `invoice_email.rs:1222-1223`) ; `seed_contact` (`invoice_send_email_e2e.rs:163-200`) pose une
  adresse complète ; la 35-2a ajoute `seed_sendable_ve_vey` au même fichier ; `spawn_app` charge le catalogue (`:102`).
- Aucune spec E2E ne lit aujourd'hui le contenu d'une ligne du rapport (`git grep 'reminder-batch-failed-row' --
  frontend/tests` vide ; `dunning-roundtrip.spec.ts:80-91` ne vérifie que la visibilité du rapport). Type du rapport :
  `details: Record<string, unknown> | null` (`frontend/src/lib/features/reminders/reminders.types.ts:109`). Ailleurs,
  `details.message` existe avec un sens **technique** non traduit (`reconciliation.rs:804`, `:2167`).
- L'écran : `ReminderBatchReport.svelte:30` affiche `reminderErrorLabel(f.errorCode)` seul, « Facture non imprimable en
  PDF » (`reminder-error-label.ts:20`). Le composant n'a pas de test.
- Le manuel (`user-manual.tex:1251`) dit que le lot affiche ce libellé seul (issue #955) ; la 35-2a réécrit la phrase du
  téléchargement, celle-ci la complète.

## Décisions

- **D1 — Le message, pour les trois refus du code** (C-e350-46, voie (a) de la validation P1). `classify_render_error` rend
  `BatchItemError::failed_with("INVOICE_NOT_PDF_READY", json!({ "message": msg }))` pour :
  - `AppError::InvoiceNotPdfReady(msg)` — la phrase de la 35-2a ;
  - `AppError::InvoiceTooManyLinesForPdf(n)` et `AppError::InvoicePdfHeaderOverflow` — la phrase **de la route unitaire**,
    par deux fonctions `pub(crate)` extraites d'`errors.rs` (`message_trop_de_lignes_pdf(n: usize) -> String`,
    `message_entete_pdf_deborde() -> String`), appelées par les bras de `IntoResponse` (`:2144-2168`) **et** par
    `classify_render_error` : une seule source pour chaque phrase (DRY). Les bras de `IntoResponse` gardent leur statut et
    leur code (`INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`) ; seul le lot les rend sous
    `INVOICE_NOT_PDF_READY`, comme aujourd'hui.
  `details.message` est une phrase **destinée à l'utilisateur**, localisée dans la langue du serveur, non un contrat lisible
  par machine (C-e350-11). Le motif « ou » de `:1030-1032` est scindé en trois bras ; **une seule** occurrence de
  `InvoiceNotPdfReady(` reste hors tests (le bras `InvoiceNotPdfReady(msg) =>`), sans quoi la garde AC14 de la 35-2a
  (`textes_coherents.rs`) rougit. Le commentaire `:1027-1029` suit.
- **D2 — L'écran**, par une fonction pure à côté de `reminderErrorLabel` (`reminder-error-label.ts`),
  `reminderFailureText(failed) → string` :
  1. le message n'est lu **que pour `INVOICE_NOT_PDF_READY`** — le `details.message` technique d'un autre code ne s'affiche
     pas ;
  2. seulement si `typeof details.message === 'string'` et non vide après `trim()` (règle de la 35-2a, D1) ;
  3. rendu en texte (`{…}`), jamais `{@html}` ;
  4. séparateur : « : » — la ligne lit `#12 — Facture non imprimable en PDF : La localité du client …` (le patron
     `#{id} — {libellé}` de `ReminderBatchReport.svelte:30` garde son tiret, sans en ajouter un second). **Limite
     assumée** (validation P2, V2-4) : le séparateur « : » (espace avant, typographie française) est écrit en dur dans les
     quatre locales, et la ligne mêle la langue de l'interface (le libellé) et celle du serveur (le message, comme sur la
     route unitaire) ; en de-CH, it-CH et en-CH l'espace avant le deux-points est fautive. Sans effet sur les comptes ; une
     clé de gabarit coûterait quatre locales pour un ordre déjà mêlé de langues. Écrite au Change Log du développement ;
  5. **aucune clé i18n neuve** : `sitesTotal` d'`i18n-keys.test.ts` inchangé, recompté.

## Critères d'acceptation

1. **AC1 — Par HTTP** (`crates/kesh-api/tests/invoice_send_email_e2e.rs`, sur le modèle de
   `send_reminder_batch_partial_success`, `:2019`) : un lot de deux factures, dans **une** société. Montage :
   `seed_sendable` (il rend `(admin_id, company_id, invoice_id)` — `seed_sendable_ve_vey` de la 35-2a jette `admin_id`),
   puis `UPDATE contacts SET address_city = 'Ve–vey'` sur le contact de la première facture ; un **second** contact, distinct,
   par `seed_contact` (e-mail gardé, sinon `CONTACT_EMAIL_MISSING` passe avant le rendu), sa facture par
   `seed_validated_invoice`, puis ses quatre colonnes `address_street`, `address_building`, `address_postal_code`,
   `address_city` à `NULL` en SQL ; `seed_dunning(&pool, company_id)` (comme `:2044`). (Faire rendre `admin_id` à
   `seed_sendable_ve_vey` est permis : fonction de la 35-2a, fichier commun, à dire au Change Log.) Deux `failed[]` en
   `INVOICE_NOT_PDF_READY`, chacun avec `details.message` : le premier contient « localité du client » et « U+2013 » ; le
   second est la phrase de `invoice-pdf-error-client-address-required`, lue par `bundle.format(fr-CH, …)`, marques
   U+2068/U+2069 retirées, non par un littéral recopié. `message` est le seul champ de `details` à ce jour, asserté sur
   l'objet entier : `assert_eq!(details.as_object().unwrap().keys().collect::<Vec<_>>(), ["message"])`. Rouge sur le code de
   départ (`details` à `null`).
2. **AC2 — Pas de fausse panne** (test `:1749`, par un accesseur `#[cfg(test)] fn details(&self)` de `BatchItemError`, à
   côté de `code()`) : les trois refus restent `INVOICE_NOT_PDF_READY`, jamais `DATABASE_ERROR`, chacun avec
   `details.message` non vide (frontière : la 35-2a ne change ni la variante ni ce test, D3 de la 35-2a).
3. **AC3 — Les deux autres causes** (même test) : `InvoiceTooManyLinesForPdf(42)` donne `details.message` égal à
   `message_trop_de_lignes_pdf(42)`, `InvoicePdfHeaderOverflow` à `message_entete_pdf_deborde()` ; et la réponse de la route
   unitaire garde ses codes et son texte : le test existant `errors.rs:5726` reste vert, et un test asserte que son
   `message` est la même chaîne que la fonction extraite (une seule source).
4. **AC4 — Écran** (Vitest, `reminder-error-label.test.ts` pour `reminderFailureText`, sans monter le composant ; un test
   du composant pour le branchement) : message présent → « Facture non imprimable en PDF : *message* » ; absent, vide,
   blanc, non-chaîne → le libellé seul ; présent sur un **autre** code (`DATABASE_ERROR` avec `details.message`) → non
   affiché ; un message portant `<b>` s'affiche en texte. Mock local d'`i18nMsg`, sans état i18n global.
5. **AC5 — E2E** (sur le modèle de `dunning-roundtrip.spec.ts:56-91`) : `createContactWithAddressViaApi(page, nom, email,
   undefined, undefined, 'Ve–vey')` — l'**e-mail fourni** (3e paramètre ; sans lui la ligne devient `CONTACT_EMAIL_MISSING`
   et le contact n'est pas proposé au lot), `city` en **6e** paramètre (la spec `invoice-qr-caracteres.spec.ts` de la 35-2a
   l'appelle sans e-mail) ; `ensurePrimaryBankAccountViaApi` ; visite de `/settings/dunning` (niveaux semés au premier
   GET) ; `createAndValidateInvoiceViaApi(page, contactId, overdueDate())` ; case `reminder-batch-checkbox`,
   `reminder-batch-send` ; assertion : `reminder-batch-failed-row` contient « localité du client » et « U+2013 » — la valeur
   traverse la frontière HTTP jusqu'à l'écran. Rouge au départ : la ligne vaut « Facture non imprimable en PDF » seul.
6. **AC6 — Documentation** :
   - `docs/api-external.md` : dans la ligne d'`INVOICE_NOT_PDF_READY` écrite par la 35-2a, la clause « Le lot
     `POST /dunning/reminders/send-batch` rend ce code par facture, sans ce message » est **remplacée** : au lot,
     `failed[].details = { message }` *(depuis la v0.17.7)*, phrase localisée dans la langue du serveur, destinée à
     l'utilisateur, à ne pas lire par machine, seul champ à ce jour ; « `details` absent » ne vaut que pour la réponse `400`
     de la route unitaire ; le `details.message` d'autres lots (réconciliation) est un texte technique, de même nom et
     d'autre nature ;
   - manuel `user-manual.tex:1251` : la phrase de la 35-2a « le lot de rappels affiche encore … (issue \#955) » réécrite ;
     `:1527` (« un compte-rendu liste … les échecs par facture ») dit que le motif est lisible ; PDF régénéré, contrôlé aplati
     (absence de « (#955) » à cet endroit ; présence d'une phrase neuve, choisie sans ligature `ff`/`fi`/`fl`) ;
   - CHANGELOG `[0.17.7]`, *Corrigé*, refs #955 (règle C-e350-8 : la 35-2b **complète** le point #955 écrit par la 35-2a,
     sans en ajouter un second) : pour un client d'API, `details` passe de `null` à un objet pour ce code dans le lot.
   La PR porte `closes #955` (la 35-2a, `refs #955`).
7. **AC7 — Capture** : la ligne du rapport d'un lot (thème clair, base jetable, `kesh-gate-logs/e352b/captures/`).

**Mutations**, chacune avec le test qui la tue : M1 `json!({ "message": msg })` remplacé par `json!({ "message": "" })` (AC1,
AC2) ; M2 `details.message` non lu par `reminderFailureText` (AC4, AC5) ; M3 le bras `InvoiceNotPdfReady` rendu à `failed`
sans `details` (AC1, AC2) ; M4 un champ ajouté à `details` (`"code": …`) (AC1, « seul champ ») ; M5 le bras
`InvoiceTooManyLinesForPdf` rendu à `failed` sans `details` (AC3) ; M5bis le bras `InvoicePdfHeaderOverflow` rendu à
`failed` sans `details` (AC3) ; M6 le bras d'`IntoResponse` qui recopie la phrase au lieu d'appeler la fonction extraite,
avec un mot changé (AC3, même chaîne) ; M7 le filtre de code retiré de `reminderFailureText` (AC4, autre code).

**Angle mort écrit** (validation P2, V2-5 c) : aux tests unitaires, `t`/`t_args` rendent le **repli** (aucun bundle dans le
binaire `lib`, garde AC13 de la 35-2a) ; AC2 et AC3 ne prouvent donc, pour les deux phrases extraites, que le repli. Au lot,
sous catalogue réel, seul `InvoiceNotPdfReady` est exercé (AC1) ; la route unitaire garde son test HTTP sous catalogue
(`invoice_pdf_e2e.rs:485`). Si le montage de `invoice_pdf_e2e.rs:485-544` (en-tête qui déborde) se réemploie sans frais pour
un lot, l'ajouter ; sinon, l'écrire tel quel au Change Log du développement. G9 reste vert : la clé
`error-invoice-pdf-header-overflow` garde un seul site dans `errors.rs`, dans la fonction extraite.

## Tâches

- [x] Étape 0 : partir de la 35-2a fusionnée ; relire les lignes citées ; ajouter l'accesseur `details()` de test et mettre à
      jour le commentaire `:1027-1029`.
- [x] Tests, rouges consignés ; `errors.rs` (deux fonctions extraites) ; serveur ; `reminderFailureText` et composant ; E2E ;
      documentation ; capture.
- [x] Gates : `test-fast.sh --ci` et `scripts/gate-ci.sh`, frontend complet au dernier commit de code ; E2E complet à la
      clôture de la story (décision de Guy du 2026-11-13).

## Dev Notes

- Fichiers : `invoice_email.rs`, `errors.rs`, `reminder-error-label.ts` (+ test), `ReminderBatchReport.svelte` (+ test), une
  spec E2E, `api-external.md`, `user-manual.tex`, `CHANGELOG.md`.
- **Union (C-e350-8)** :
  - avec la **35-2a**, qui la précède : `invoice_email.rs`, `invoice_send_email_e2e.rs`, `api-external.md`, `user-manual.tex`,
    `CHANGELOG.md` ; la **garde AC14** de la 35-2a (`textes_coherents.rs`, « exactement deux occurrences de
    `InvoiceNotPdfReady(` » hors tests) : la 35-2b garde une seule occurrence au bras de `classify_render_error` (D1) ; le
    gate qui la porte est `textes_coherents.rs` ;
  - avec la **35-3** : `CHANGELOG.md`, `docs/api-external.md`, `user-manual.tex` (C-e350-8, points 1, 5, 6), et
    `errors.rs` — la 35-3 y ajoute un bras `DbError` après `DesignatedAccountsNotPostable` (`:4571`), la 35-2b ne touche que
    les bras `:2144-2168` et ajoute ses deux fonctions à côté : hunks distincts, aucun reformatage hors des passages
    touchés ; avec la **35-1**, de même (`errors.rs`, bras `NumberFormatUnrenderable`) ; écrit au fichier d'epic (règle
    d'union, ligne `errors.rs`) et au registre (C-e350-8, annotation de la validation P2) ;
  - avec la **35-5b** : `invoice_email.rs` — la 35-5b touche `:206`, `:381` (`render_reminder`), `:904` ; la 35-2b `:1023-1036`
    et le test `:1749` : hunks distincts.
- Fixtures synthétiques seulement. Pattern batch du `CLAUDE.md` : aucune erreur per-facture n'escalade.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement, en autonomie), worktree `/home/gcorbaz/devel/kesh-e352b`, le 2026-11-13.

### Debug Log References

Journaux hors dépôt, sous `/home/gcorbaz/devel/kesh-gate-logs/e352b/` : `rouges.log` (Rust, Vitest, E2E),
`rouges-front.log`, `e2e-rouge.log`, `cible-1.log`, `clippy.log`, `front-check.log`, `front-lint-i18n-ownership.log`,
`front-test-unit.log`, `front-build.log`, `make-user.log`, `user-manual-plat.txt`, `mut/` (`campagne.log`, `M1` à `M7`),
`front-test-unit.log` (premier Vitest complet, rouge sur la garde des libellés en dur), `front-test:unit.log` et les autres
`front-<script>.log` (frontend complet au dernier commit de code), `test-fast-ci.log`, `gate-ci.log` ; scripts `e2e-e352b.sh`, `mut-e352b.py`, `mut-rust-e352b.sh`,
`mut-campagne-e352b.sh` ; captures `captures/`.

### Completion Notes List

**Étape 0** (lignes relues sur la branche, `506237558` = la 35-2a terminée `9f8ef604` + `epic-35/base3`) : toutes les lignes
que la fiche lisait sur `release/0.17.6` sont **au même endroit** sur la branche — `invoice_email.rs:964-990`
(`struct BatchItemError`, `fn failed_with` `:979`, `fn code` `:988`), `:1023-1036` (`fn classify_render_error`, motif « ou »
`:1030-1032` : `AppError::InvoiceNotPdfReady(_)` / `| AppError::InvoiceTooManyLinesForPdf(_)` / `| AppError::InvoicePdfHeaderOverflow
=> BatchItemError::failed("INVOICE_NOT_PDF_READY")`), `:1260` (`render_document(` du lot), `:1749`
(`fn classify_render_error_ne_deguise_pas_un_refus_en_panne`) ; `errors.rs:2144-2168` (les deux bras), `:5726`
(`invoice_too_many_lines_maps_to_400`) ; `git grep -n 'render_document('` : les quatre appels cités ; `contact.rs:15-24`
(`structured_address`) ; `invoice_send_email_e2e.rs:163` (`seed_contact`), `:249` (`seed_sendable`), `:1192` (`seed_dunning`),
`:2019` (`send_reminder_batch_partial_success`), `:2348` (`seed_sendable_ve_vey`, qui rendait `(company_id, invoice_id)`).
La garde AC14 de la 35-2a (`textes_coherents.rs:1155-1184`) compte `InvoiceNotPdfReady(` hors `errors.rs` et hors modules de
test, **deux sites** attendus (`invoice_email.rs`, `invoice_pdf_service.rs`) : la 35-2b en garde un seul au bras
`InvoiceNotPdfReady(msg) =>`. Le manuel `:1251` porte la phrase de la 35-2a (« le lot de rappels affiche encore … (issue
\#955) »), `api-external.md:1059` la clause « sans ce message », le CHANGELOG la section `[0.17.7]` ouverte par la 35-2a.
**Aucun écart** de ligne avec la fiche. Les LOW de la dernière validation (P2, V2-1 à V2-5) étaient déjà appliqués à la fiche ;
rien n'en restait au code. Point « à établir » : l'angle mort de l'en-tête qui déborde — le montage se réemploie (C-e352b-3).

**Écart de placement** : les deux fonctions extraites sont posées juste avant `impl IntoResponse for AppError` (`errors.rs`,
après `opening_complement_refusal_response`), non entre les bras `:2144-2168` — Rust ne permet pas de fonction dans un `match`. Les
hunks restent distincts de ceux de la 35-3 (`:4571`) et de la 35-1.

**Rouges** (`rouges.log`, code de production intact, tests seuls) : `classify_render_error_ne_deguise_pas_un_refus_en_panne`
(« `InvoiceNotPdfReady("adresse incomplète")` : details absent au lot ») ; `send_reminder_batch_refus_pdf_porte_le_message`
(« details n'est pas un objet : `{"details":null,"errorCode":"INVOICE_NOT_PDF_READY","invoiceId":1}` ») ; Vitest : 6 rouges sur
11 (les quatre de `reminderFailureText`, absente ; deux du composant, la ligne ne montrant que le libellé) — le test « sans
message » du composant, vert au départ, est la non-régression ; **E2E** (binaire et build frontend du code de départ, spec
seule, `e2e-rouge.log`) : « Received string: "#1 — Facture non imprimable en PDF" ». AC3 (égalité avec les fonctions
extraites) n'a pas de rouge propre : il ne compile qu'avec elles ; M5, M5bis et M6 le prouvent.

**Mutations** (`mut/`, jouées sur `d765f9172` (Rust) et sur l'arbre de travail du même commit (frontend), chacune restaurée
par `git checkout --` puis `touch`, « restauré : 0 écart » au journal ; huit sur huit tuées) :

| | mutation | tuée par |
|---|---|---|
| M1 | `json!({ "message": message })` → `json!({ "message": "" })` | `classify_render_error_ne_deguise_pas_un_refus_en_panne` (« details.message vide »), `send_reminder_batch_refus_pdf_porte_le_message`, `send_reminder_batch_entete_qui_deborde_porte_le_message` (`mut/M1.log`) |
| M2 | `const message = failed.details?.message` → `undefined` | Vitest : 4 rouges, deux de `reminderFailureText`, deux du composant (`mut/M2.log`) ; l'E2E n'est pas rejoué pour elle : `e2e-rouge.log` montre le même affichage, libellé seul |
| M3 | bras `InvoiceNotPdfReady` → `failed` sans `details` | `classify_render_error_…` (« details absent au lot »), `send_reminder_batch_refus_pdf_porte_le_message` (`mut/M3.log`) |
| M4 | `"code": …` ajouté à `details` | les trois tests de M1 (« message est le seul champ de details ») (`mut/M4.log`) |
| M5 | bras `InvoiceTooManyLinesForPdf` → `failed` | `classify_render_error_…` seul (`mut/M5.log`) — aucun test HTTP au lot pour cette cause (angle mort ci-dessous) |
| M5bis | bras `InvoicePdfHeaderOverflow` → `failed` | `classify_render_error_…`, `send_reminder_batch_entete_qui_deborde_porte_le_message` (`mut/M5bis.log`) |
| M6 | bras d'`IntoResponse` qui recopie la phrase, « scindez » → « divisez » | `refus_du_pdf_route_unitaire_et_lot_meme_phrase` (`mut/M6.log`) |
| M7 | filtre de code retiré de `reminderFailureText` | Vitest « n'affiche pas le message d'un autre code » (`mut/M7.log`) |

**Captures** (AC7) : `captures/35-2b-rapport-lot-refus-pdf.png` (l'écran des rappels après un lot de deux factures, une envoyée,
une écartée) et `35-2b-rapport-lot-refus-pdf-detail.png` (le rapport seul), thème clair, base jetable `kesh_e2e_e352b`.

**Limite assumée** (D2.4, écrite ici comme la fiche le demande) : le séparateur « : » (espace avant) est écrit en dur dans les
quatre langues, et la ligne mêle la langue de l'interface (le libellé) et celle de l'installation (la phrase, comme sur la
route unitaire) ; en de-CH, it-CH et en-CH l'espace avant le deux-points est fautive. Sans effet sur les comptes.

**Angle mort** (V2-5 c) : réduit par C-e352b-3 — l'en-tête qui déborde est prouvé au lot sous catalogue réel
(`send_reminder_batch_entete_qui_deborde_porte_le_message`). Reste `InvoiceTooManyLinesForPdf` : sa phrase n'est prouvée au lot
que par le repli (AC2, AC3, binaire `lib` sans catalogue) ; la route unitaire garde son test HTTP. Un montage HTTP de
`trop de lignes` pour un rappel demanderait de connaître le plafond du document de rappel : non tenté.

**Garde des libellés en dur** (`i18n-libelle-en-dur.test.ts`) : `reminderFailureText` porte le suffixe `Text` ; la garde l'a
relevée au Vitest complet (91 candidates pour 90). Déclaration **nommée** au commentaire, puis compteurs 90 → 91 et `conforme`
76 → 77 — la seule déclaration neuve à suffixe surveillé du diff (`git diff 50623755 -- frontend/src ':!*.test.ts'`).
L'inventaire est **par exécution** (la garde relit toutes les sources) et sa sonde positive est ce rouge même : elle a vu la
déclaration neuve avant tout ajustement.

**Gates au dernier commit de code `24f69d217`** (bases remises à neuf ; chaque chiffre avec son journal) :

- `scripts/test-fast.sh --ci` (fmt, clippy, nextest, base `kesh_e352b` recréée) : **5334 passés, 13 ignorés**, rc 0
  (`kesh-gate-logs/e352b/test-fast-ci.log`, commit `24f69d217`).
- `scripts/gate-ci.sh kesh_ci_e352b` : GATECI (`kesh-gate-logs/e352b/gate-ci.log`, `gate-ci-console.log`, commit
  `24f69d217`).
- Frontend complet : `npm run check` 0 erreur, 29 avertissements (`front-check.log`) ; `lint-i18n-ownership` PASS
  (`front-lint-i18n-ownership.log`) ; Vitest **3195 passés, 233 fichiers** (`front-test:unit.log`) ; `npm run build` rc 0
  (`front-build.log`) — tous au commit `24f69d217`.
- Gate ciblé en cours de développement (`cible-1.log`, arbre de `d765f9172` avant commit) : 120/120 ; `clippy -D warnings`
  workspace rc 0 (`clippy.log`), `cargo fmt --check` rc 0.
- Manuel : `make user` rc 0, 3 passes (`make-user.log`) ; PDF aplati (`user-manual-plat.txt`) : « lot de rappels la reprend »
  1 occurrence, « le motif dit ce qu’il y a à » présent (apostrophe typographique, coupure de page), « (issue #955) » 0.
- **E2E complet : non lancé — à la clôture de la story.** La spec neuve n'a tourné qu'en rouge, sur le code de départ.

**Écarts aux consignes** : les mutations s'appliquent par un script Python (`mut-e352b.py`, motif vérifié unique), permis et
déclaré ; tout le code et les tests sont écrits par l'outil Edit (ou Write pour les fichiers neufs). Aucun autre écart.

### File List

- `crates/kesh-api/src/errors.rs` — `message_trop_de_lignes_pdf`, `message_entete_pdf_deborde` ; les deux bras les appellent ;
  test `refus_du_pdf_route_unitaire_et_lot_meme_phrase`.
- `crates/kesh-api/src/routes/invoice_email.rs` — `classify_render_error` (trois bras), `refus_pdf_au_lot`, accesseur de test
  `details()` ; test `classify_render_error_ne_deguise_pas_un_refus_en_panne` étendu (AC2, AC3).
- `crates/kesh-api/tests/invoice_send_email_e2e.rs` — `seed_sendable_ve_vey` rend `admin_id` (C-e352b-5) ; `phrase_fr` ;
  `send_reminder_batch_refus_pdf_porte_le_message` (AC1), `send_reminder_batch_entete_qui_deborde_porte_le_message`
  (C-e352b-3).
- `frontend/src/lib/features/reminders/reminder-error-label.ts` — `reminderFailureText`.
- `frontend/src/lib/features/reminders/reminder-error-label.test.ts` — quatre tests (AC4).
- `frontend/src/lib/features/reminders/ReminderBatchReport.svelte` — la ligne passe par `reminderFailureText`.
- `frontend/src/lib/features/reminders/ReminderBatchReport.test.ts` — neuf, trois tests (AC4, branchement).
- `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts` — 90 → 91 candidates, `conforme` 76 → 77, déclaration nommée.
- `frontend/tests/e2e/reminder-batch-refus-pdf.spec.ts` — neuf (AC5, C-e352b-4).
- `docs/api-external.md` — ligne `INVOICE_NOT_PDF_READY` (AC6).
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf` — `:1251`, `:1527` (AC6).
- `CHANGELOG.md` — point #955 de `[0.17.7]` complété (AC6).
- `_bmad-output/implementation-artifacts/e35-choix-autonomes-e352b.md` — neuf, C-e352b-1 à -5.
- `_bmad-output/implementation-artifacts/35-2b-lot-rappels-refus-pdf-review-prompt-p1.md` — neuf.
- `_bmad-output/implementation-artifacts/35-2b-lot-rappels-refus-pdf.md`, `sprint-status.yaml` — statut `review`.

## Change Log

- 2026-11-12 — Née du découpage de la 35-2 à la validation P2 (C-e350-9). Validation à lancer.
- 2026-11-12 — Réajustée au recadrage de la 35-2a (validation P3, C-e350-11) : `details` du lot réduit à `message`,
  phrase destinée à l'utilisateur ; aucune `reason` ni `field` ; frontière avec la 35-2a écrite (AC2) ; `closes #955` ici.
- 2026-11-12 — **Validation P1** (Opus) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW (`kesh-gate-logs/e35-2b-validation-p1.md`).
  Remédiée par l'agent de rédaction des fiches (Opus 5.5). Prémisses relues sur `main` (`4d39f377`) :
  `invoice_email.rs:1008-1036` (le motif « ou » `:1030-1032`), `errors.rs:2144-2168` (les deux phrases de la route
  unitaire, construites dans `IntoResponse`), `ReminderBatchReport.svelte:25-34`, `reminders.types.ts:105-110`, la garde AC14
  de la fiche 35-2a (`:239`). Toutes confirmées.
  - V1-1 (MEDIUM) : **voie (a)** retenue (C-e350-46) — `details.message` aussi pour `InvoiceTooManyLinesForPdf` et
    `InvoicePdfHeaderOverflow`, par deux fonctions extraites d'`errors.rs`, communes à `IntoResponse` et au lot (D1) ;
    simple : deux fonctions sans état, les bras gardent statut et code. AC3 inversé.
  - V1-2 (MEDIUM) : M1, reste du contrat retiré (« refus sans champ »), remplacée ; mutations M1 à M7 jouables, chacune avec
    son test.
  - V1-3 (MEDIUM) : AC5, un E2E qui voit `details.message` dans la ligne du rapport.
  - LOW : V1-4 (D2 : règle d'affichage, fonction pure, filtre de code, texte, séparateur « : », aucune clé neuve) ; V1-5
    (AC1 : montage du contact sans adresse, `seed_sendable_ve_vey`, phrase lue au catalogue) ; V1-6 (AC6 : la ligne d'API,
    le manuel `:1251` et `:1527`, le CHANGELOG) ; V1-7 (union : 35-3, 35-1, 35-5b, garde AC14 de la 35-2a) ; V1-8 (AC2,
    AC3 : accesseur `details()` de test, les deux causes) ; V1-9 (référence `main`, statut tenu par le registre de sprint).
  - Symptômes greppés (fiche, fichier d'epic, registre) : « refus sans champ », « restent sans `details` »,
    « sans ce message » — remplacés dans la fiche ; le fichier d'epic (« Le rapport d'un lot de rappels porte la phrase du
    refus ») reste juste.
- 2026-11-13 — **Validation P2** (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 5 LOW (`kesh-gate-logs/e35-2b-validation-p2.md`).
  Appliqués : V2-1 (AC1 : montage par `seed_sendable`, second contact distinct, `seed_dunning`) ; V2-2 (AC5 : e-mail en 3e
  paramètre, `city` en 6e, `/settings/dunning`, `overdueDate()`, rouge écrit) ; V2-3 (union : `errors.rs` nommé pour la
  35-2b au fichier d'epic — ligne de la story et règle d'union — et par annotation datée de C-e350-8, point 4 ; G9 vert, un
  seul site de `error-invoice-pdf-header-overflow`) ; V2-4 (séparateur « : » et mélange de langues **assumés par écrit**,
  sans clé neuve, D2.4 — la lecture du code ne montre rien qui gêne une locale au-delà de la typographie) ; V2-5 (M5bis,
  assertion de l'objet entier en AC1, angle mort du repli écrit).
- 2026-11-13 — **Validation terminée.** Série des findings au-dessus de LOW : P1 3 MEDIUM → P2 0 (CRITICAL et HIGH : 0).
  Modèles : Opus (P1), Sonnet (P2). La boucle se clôt sur une passe sans finding au-dessus de LOW ; la remédiation de P2 ne
  touche que la rédaction. Choix retenu : **C-e350-46** (voie (a) : le message pour les trois causes, par une construction
  commune). Statut : `ready-for-dev`.
- 2026-11-13 — **Développement** (Opus 5.5, agent `e352b`). Étape 0 : aucun écart de ligne entre `release/0.17.6` et la branche
  (`506237558`), lignes citées au Dev Agent Record ; écart de placement des deux fonctions d'`errors.rs` (avant
  `impl IntoResponse`). Rouges consignés (Rust 2, Vitest 6, E2E 1 sur le code de départ). D1 et D2 livrés ; AC1 à AC7 ; un
  test HTTP de plus pour l'en-tête qui déborde (C-e352b-3). Mutations M1 à M7 et M5bis : huit tuées sur huit. Garde des
  libellés en dur 90 → 91 (déclaration nommée). Choix C-e352b-1 à -5. Limite D2.4 et angle mort (`InvoiceTooManyLinesForPdf`
  au lot prouvé par le repli seul) écrits. Commits : `d765f9172` (implémentation), `24f69d217` (garde, dernier commit de
  code). Gates au dernier commit de code : au Dev Agent Record. **E2E complet à la clôture de la story** (décision de Guy du
  2026-11-13). Statut : `review`.
