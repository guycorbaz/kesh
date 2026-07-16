# Story 21.5b: Envoi de rappels par e-mail (backend)

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **comptable/fiduciaire utilisant Kesh**,
I want **envoyer les rappels débiteurs par e-mail — un aperçu rendu serveur, un envoi unitaire (avec choix du niveau) et un envoi par lot des factures dues — avec le PDF de la facture joint, dans la langue et le ton du niveau, et une trace historisée**,
so that **je relance mes débiteurs sans ressaisir les courriers, avec l'escalade de ton par niveau et une preuve de chaque envoi (dossier de recouvrement)**.

## Contexte

Épic 21 « Échéances & relances débiteurs ». La couche **données & éligibilité** (`invoice_reminders`, éligibilité, endpoints liste/toggle/manuel/annulation) a été livrée par **21-5a** (done). Le **socle d'envoi e-mail** (SMTP, MockMailer, rendu templates, PDF joint, rate-limit, preview) vient de l'**Epic 20** (envoi de factures, done). Le **type `invoice_reminder`, la colonne `level_number` et les défauts Rust par niveau** viennent de **21-3** (done). Le **TTC canonique** (`invoice_total_ttc`) vient de **21-2a** (done).

Cette story **21-5b** assemble ces briques pour **envoyer les rappels** :
- `build_reminder_vars()` (6 variables de base + `reminderLevel`/`reminderFee`/`totalDue`/`daysOverdue`).
- **Preview** rendue serveur d'un rappel à un niveau donné.
- **Envoi unitaire** (choix du niveau ≤ prochain, gardes d'éligibilité, PDF joint, ordre « SMTP puis enregistrer », re-check du niveau sous verrou).
- **Envoi par lot** `{ accepted, failed }` (cap 20, pré-check capacité rate-limit, 1 slot/e-mail, tx per-facture post-SMTP, niveau = prochain uniquement).
- **Log INFO** sur envoi réussi.

**RBAC** : preview + envoi unitaire + envoi lot = **Comptable+** (item 22). L'écran de relance est **21-6** ; la balance âgée **21-7**.

## Acceptance Criteria

### Builder de variables

1. **`build_reminder_vars(invoice, lines, contact, company, language, level_number, reminder_fee, total_due, days_overdue) -> HashMap<String,String>`** (dans `routes/invoice_email.rs` ou un module voisin, à côté de `build_invoice_vars`) : pose les **6 variables de base** exactement comme `build_invoice_vars` (`salutation` via `salutation_line`, `contactName`, `companyName`, `invoiceNumber` avec fallback `#{id}`, **`amount` = TTC** via `invoice_total_ttc`, `dueDate` avec fallback `«—»`) **+ 4 variables rappel** : `reminderLevel` = `level_number.to_string()`, `reminderFee` = `format_money(reminder_fee)`, `totalDue` = `format_money(total_due)`, `daysOverdue` = entier `to_string()` (PAS `format_money`). Réutiliser `salutation_line` (`invoice_email.rs:107`), `resolve_language` (`:67`), `format_money`/`format_date` (kesh-i18n).
2. **`totalDue`** = `invoice_total_ttc(lines)` **+ Σ des frais des rappels non-annulés dé-dupliqués par niveau** **+ `fee_amount` du niveau en cours d'envoi** (`dunning_levels::find_by_level_number(level)`). **Dédoublonnage par `level_number` (H2)** : un même niveau peut avoir plusieurs lignes `invoice_reminders` (ré-émission D18) — ne compter le frais d'un niveau **qu'une fois**. Requête : `SELECT SUM(fee) FROM (SELECT level_number, MAX(fee_amount) fee FROM invoice_reminders WHERE company_id=? AND invoice_id=? AND cancelled_at IS NULL GROUP BY level_number) t` (ou équivalent en Rust : grouper par `level_number`, prendre le frais, sommer). **Attention** : si le niveau en cours d'envoi a déjà une ligne non-annulée (ré-émission), ne PAS additionner son frais deux fois (le `+ fee du niveau courant` s'annule si `level` figure déjà dans la somme dédupliquée). **Calculé AVANT l'insertion du rappel courant** (sinon double comptage — le body est snapshoté dans `NewInvoiceReminder`). `reminderFee` = frais du niveau en cours (`format_money(level.fee_amount)`).
3. **`daysOverdue`** = `(today_utc - due_date).num_days().max(0)` (clampé ≥ 0), `0` si `due_date` NULL. `today_utc = chrono::Utc::now().naive_utc().date()` (cohérent `is_invoice_overdue`/UTC_DATE).

### Preview

4. **`GET /api/v1/invoices/{id}/reminder-preview?level=N`** — **Comptable+** : rend **côté serveur** subject+body d'un rappel de niveau `N` pour la facture, via `email_templates::get_effective(company_id, InvoiceReminder, language, N)` + `render(template.subject/body, build_reminder_vars(...))`. DTO `ReminderPreviewResponse { to: Option<String>, language: Language, level: i16, subject: String, body: String }` (calqué `EmailPreviewResponse` + `level`). `to` = `locked_recipient(contact)` (NULL si contact sans e-mail). Scopé company (404 cross-tenant). **Ne consomme pas de slot rate-limit** (lecture). **`level` requis (M1)** : `#[serde(default)]` interdit ici — absence → 400 `VALIDATION_ERROR` (l'UI 21-6 fournit toujours le niveau) ; `N >= 1` et existe en config → sinon 422 `DUNNING_LEVEL_NOT_FOUND`.

### Envoi unitaire

5. **`POST /api/v1/invoices/{id}/reminders/send`** — **Comptable+** : envoie un rappel e-mail. **Request DTO** (figé, **PAS de champ `to`** — item 16) : `{ levelNumber: i16, subject: String, body: String }` (subject/body édités côté client depuis la preview). Ordre des gardes calqué `send_invoice_email` (item 16) :
   1. auth/tenant (`get_company_for`).
   2. **rate-limit 429** (`rate_limiter_send_email.check_and_record((company_id, user_id))`).
   3. **SMTP prêt 412** (`smtp_ready` → `AppError::SmtpNotConfigured`).
   4. facture 404 (`find_by_id_with_lines`, scopé company).
   5. **gardes d'éligibilité** (item 16, AC explicite aussi sur l'unitaire) : `status='validated'` sinon 422 ; `paid_at` non-NULL → 422 `INVOICE_ALREADY_PAID` ; `dunning_paused_at` non-NULL → 422 `DUNNING_PAUSED` ; `levelNumber` inexistant en config → 422 `DUNNING_LEVEL_NOT_FOUND`.
   6. contact actif (`load_active_contact`) + **destinataire verrouillé** `locked_recipient` (sinon 422 `CONTACT_EMAIL_MISSING`).
   7. contenu vide (subject/body trim) → 422 `INVOICE_EMAIL_EMPTY_CONTENT`.
   8. rendu PDF de la **facture d'origine** (`invoice_pdf_service::render`) → attachment `facture-{filename_base}.pdf`.
6. **Choix du niveau (item 18)** : à l'unitaire, `levelNumber` autorisé = **≤ prochain niveau** (ré-émettre un e-mail perdu d'un niveau déjà atteint, ou envoyer le prochain). `levelNumber > prochain` → refus (saut interdit → passe par le rappel manuel 21-5a). Le **prochain niveau** = plus petit `dunning_levels.level_number > current_level` (0 rappel → niveau 1).
7. **Anti-double-envoi + gardes SOUS VERROU AVANT le SMTP (item 17, corrigé C1/H1)** : les gardes globales AC 5 (auth, **rate-limit** étape 2, SMTP 412) s'appliquent **avant** ce verrou ; puis **toutes les vérifications métier qui peuvent rejeter la requête doivent avoir lieu AVANT l'appel SMTP** — car après le SMTP l'e-mail est irréversiblement parti et le fait DOIT être tracé (leçon Epic 20 `mark_emailed` `invoices.rs:1712-1734` : jamais de re-check métier post-SMTP). Séquence :
   - **(a) Verrou bref pré-SMTP** : ouvrir une tx, `invoices::find_scoped_for_update_in_tx` (`FOR UPDATE` sur la row `invoices`, 21-5a), re-vérifier `status='validated'`/`paid_at IS NULL`/`dunning_paused_at IS NULL` + calculer le niveau courant (`invoice_reminders::current_level_in_tx`) et le prochain niveau (config). **Rejets ici** (avant SMTP, aucun e-mail parti) : gardes d'éligibilité → 4xx ; **unitaire** `levelNumber > prochain` (tentative de saut, D18) → **409 `LEVEL_ALREADY_SENT`** ; niveau sans config → 422. **Puis COMMIT/relâcher le verrou** (ne PAS tenir le verrou pendant le SMTP — 2-5 s/message).
   - **(b) SMTP** : `state.mailer.send_email(&email).await?` (échec → 500 `SMTP_SEND_FAILED`, rien enregistré — l'e-mail n'est pas parti).
   - **(c) Enregistrement best-effort post-SMTP** : nouvelle tx, `invoice_reminders::insert_in_tx(channel=Email, sent_to=Some(to), …)` + audit `invoice.reminder_sent`. **Ne JAMAIS rejeter ici sur une condition métier** (l'e-mail est parti) : si la facture a disparu (#219) entre (a) et (c) → trace best-effort (calquer `EmailSentInvoiceGone` `invoice_email.rs:335-379`) + `AppError::ReminderSentButInvoiceGone` (409) plutôt que perte silencieuse. **Échec DB générique en (c)** (deadlock/pool, ≠ facture-disparue, L3-p3) : propager → 500 (calquer `invoice_email.rs:380 Err(e) => Err(e.into())`) — même exposition résiduelle « dual-write sans outbox » qu'Epic 20, assumée héritée (pas un rejet métier).
   - **Course résiduelle assumée (H1)** : un double-clic / envoi concurrent du **même** niveau reste possible (le verrou de (a) est relâché avant le SMTP). Ce n'est PAS bloqué en backend (le ré-envoi ≤ prochain est un besoin D18) → **risque documenté**, mitigé côté UI **21-6** (désactivation du bouton après clic). Le re-check de (a) protège contre le **saut** de niveau, pas contre la ré-émission concurrente.
8. **Ordre « SMTP d'abord ⇒ enregistrer » (item 16)** confirmé par la séquence AC 7 (b)→(c). Le rappel envoie le subject+body **du payload** (édités depuis la preview), pas de re-render au send (cohérent Epic 20). Response 200 = `ReminderResponse` (le rappel créé, DTO 21-5a) ; 409 `ReminderSentButInvoiceGone` si la facture a disparu post-SMTP.
9. **Log INFO envoi réussi (item 21)** : `tracing::info!` après commit (facture ET rappel — appliquer aussi à l'envoi facture Epic 20 qui ne loggue rien aujourd'hui) : `invoice_id`, `level_number`, `channel`, destinataire (masqué/partiel si politique). Un envoi réussi doit laisser une trace dans le log fichier.

### Envoi par lot

10. **`POST /api/v1/dunning/reminders/send-batch`** — **Comptable+** : envoie le **prochain niveau** à une liste de factures. **Request DTO** : `{ invoiceIds: Vec<i64> }` (**PAS de `to`**, PAS de subject/body — le lot rend chaque template serveur, item 16). Les `invoiceIds` sont **dé-dupliqués** en entrée (L2 — un doublon ne doit pas envoyer deux niveaux en un appel ; le cap 20 s'applique **après** dédup). Réponse **HTTP 200** `{ accepted: Vec<AcceptedReminder>, failed: Vec<FailedProposal> }` (pattern batch CLAUDE.md).
11. **Cap dur + pré-check rate-limit (item 19)** — **AVANT tout SMTP** :
    - `invoiceIds.len() > 20` → **422** global (`RESULT_TOO_LARGE` ou `BATCH_TOO_LARGE`) — borne la durée HTTP (N envois SMTP séquentiels).
    - **pré-check de capacité** : si `invoiceIds.len() > slots restants` sur le limiteur send-email → **429 global** AVANT le 1er SMTP (pas de blocage mi-course qui gèlerait l'utilisateur 15 min). Nécessite une **nouvelle méthode `RateLimiter::remaining_slots(key) -> u32`** (`middleware/rate_limit.rs` : `max_attempts - recent_count`, actuellement privés — à exposer). Le lot **consomme 1 slot par e-mail effectivement envoyé** (pas de contournement du 20/15 min).
12. **Traitement per-facture (item 19, même ordre pré-SMTP que l'unitaire — C1)** : pour chaque `invoice_id` (dans l'ordre, **liste dé-dupliquée** — L2), la séquence est : **(a)** gardes pré-SMTP sous verrou bref (`FOR UPDATE`, re-check `status`/`paid_at`/`dunning_paused_at` + calcul du **prochain niveau** — le lot n'envoie **que** le prochain niveau, jamais de ré-envoi ni saut, item 18) → échec = `FailedProposal { invoice_id, error_code, details? }` (JAMAIS d'`AppError` global — succès partiel = HTTP 200), commit/relâche le verrou ; **(b)** SMTP ; **(c)** enregistrement best-effort + audit. Codes `FailedProposal` per-facture : `INVOICE_NOT_FOUND` (absente **ou cross-tenant** — même code, pas de fuite d'existence), `INVOICE_ALREADY_PAID`, `DUNNING_PAUSED`, `CONTACT_EMAIL_MISSING`, `INVOICE_NOT_PDF_READY`/`INVOICE_NOT_VALIDATED`, `REMINDER_CONTENT_EMPTY` (M2 — override company vidé rend un subject/body vide), `NO_NEXT_LEVEL` (facture au dernier niveau / dunning désactivé), `RATE_LIMITED` (slot manquant en cours de lot malgré le pré-check), `SMTP_SEND_FAILED`. **`LEVEL_ALREADY_SENT` n'est PAS un cas du lot** (le lot ne cible que le prochain niveau calculé sous verrou juste avant l'envoi — pas de niveau fourni par le client à re-checker).
    - succès → `AcceptedReminder { invoice_id, reminder_id, level_number }` + audit `invoice.reminder_sent` + log INFO.
    - **Garde-fou variant sum-type** (CLAUDE.md pattern batch) : pas d'`unreachable!()` ; un refactor incomplet → `AppError::Internal` (500 global), une validation métier per-facture → `FailedProposal`.
13. **Exceptions `AppError` globales autorisées** (CLAUDE.md) uniquement en amont du per-facture : 401 (auth), 403 (RBAC Comptable+), 400 (body invalide), 422 (cap 20), 429 (pré-check capacité), 500 (pool/panic). Toute erreur dépendante de la facture → `FailedProposal`.

### Tests

14. **Tests unitaires** `build_reminder_vars` : les 10 variables posées, `totalDue` = TTC + Σ frais non-annulés + frais niveau, **dédoublonné par niveau (H2 — deux lignes même niveau ne comptent le frais qu'une fois)**, pas de double comptage du niveau courant, `daysOverdue` clampé ≥ 0 + `due_date` NULL → 0, formatage suisse (`format_money`/`format_date`), `reminderLevel`/`reminderFee` corrects par niveau.
15. **Tests E2E envoi unitaire** (MockMailer, `KESH_TEST_MODE`) : preview rendue serveur (subject/body substitués, `to` verrouillé) ; envoi niveau prochain → e-mail capturé (`/_test/sent-emails`, PDF joint `facture-*.pdf`, destinataire = contacts.email) + `invoice_reminders` (channel='email', sent_to) + audit `invoice.reminder_sent` ; **gardes** : payée → 422 `INVOICE_ALREADY_PAID`, suspendue → 422 `DUNNING_PAUSED`, sans e-mail → 422 `CONTACT_EMAIL_MISSING`, SMTP non configuré → 412, contenu vide → 422 ; **niveau > prochain → refus** ; **ré-envoi niveau ≤ prochain autorisé** ; **SMTP down → 500 + rien enregistré** (MockMailer::failing) ; anti-IDOR 404 ; RBAC Consultation → 403. **Gardes AVANT SMTP (C1)** : une facture payée/suspendue est rejetée **sans qu'aucun e-mail ne parte** (vérifier `/_test/sent-emails` vide) ; réciproquement, un test « facture supprimée post-SMTP » → e-mail parti + trace (409 `ReminderSentButInvoiceGone`, audit best-effort).
16. **Tests E2E envoi lot** : lot de N factures dues → `{ accepted, failed }` 200 ; succès partiel (1 payée + 1 sans e-mail + 1 OK → 1 accepted, 2 failed avec les bons `error_code`) ; **cap dur 20 : un lot de 21 → 422** ; **pré-check capacité** (lot > slots restants → 429 global, aucun e-mail parti) ; niveau = prochain uniquement ; cross-tenant id → `INVOICE_NOT_FOUND` (pas de fuite) ; RBAC Consultation → 403.
17. **Test rate-limiter** `remaining_slots` : décrémente à chaque `check_and_record`, `max_attempts` au départ, 0 quand bloqué.

### Doc & gate

18. **Doc** : CHANGELOG `Ajouté` (envoi des rappels par e-mail — preview/unitaire/lot). Manuels = **21-8**. Pas de nouvelle migration (aucune table/colonne) → pas de ligne idempotence, pas de bump `min_required`.
19. **Gate local complet** (Test Locally First) : `cargo fmt`/`clippy`/`build`/`test --workspace --test-threads=1`. **E2E backend Rust** (MockMailer) suffisants ici (pas de frontend → 21-6). **Rappel gate** : la DB dev doit être migrée (`sqlx migrate run`) ; le gate workspace complet est le filet ultime (leçon 21-5a — un SELECT `Invoice` cross-crate).

## Tasks / Subtasks

- [x] **T1 — `build_reminder_vars` + calcul `totalDue`/`daysOverdue`** (AC 1,2,3,14)
  - [x] `build_reminder_vars` (invoice_email.rs) réutilise `build_invoice_vars` + 4 vars rappel (pré-calculées par l'appelant, builder pur).
  - [x] `invoice_reminders::sum_fees_deduped_excluding(company, invoice, exclude_level)` : `SUM(MAX(fee) GROUP BY level_number) WHERE cancelled_at IS NULL AND level_number <> ?` (dédup H2 + exclut niveau courant). `daysOverdue` helper UTC clampé.
  - [x] Tests : unit `build_reminder_vars` (10 vars, formatage suisse) + `days_overdue` (clamp/None) 2/2 ; repo `sum_fees_deduped_excluding` (dédup MAX par niveau + exclut annulés/niveau) 4/4.
- [x] **T2 — Rate-limiter `remaining_slots`** (AC 11,17)
  - [x] `RateLimiter::remaining_slots(key) -> u32` (purge + `max_attempts.saturating_sub(recent_count)`, 0 si bloqué) + test (décrément + 0 bloqué) 1/1.
- [x] **T3 — Preview rappel** (AC 4)
  - [x] `GET .../reminder-preview?level=N` (`ReminderLevelQuery` sans serde default → 400 si absent) + `ReminderPreviewResponse` + `render_reminder` helper (get_effective + build_reminder_vars + render).
- [x] **T4 — Envoi unitaire** (AC 5,6,7,8,9) — `send_reminder` (invoice_email.rs)
  - [x] Gardes globales (rate-limit/SMTP) → verrou bref pré-SMTP (find_scoped_for_update_in_tx + status/paid/paused + niveau ≤ prochain → LEVEL_ALREADY_SENT) → relâche → contact/contenu/PDF → SMTP → enregistrement best-effort (`record_reminder_email` channel=Email + audit ; échec → `best_effort_reminder_audit` + `ReminderSentButInvoiceGone` 409). Log INFO.
  - [x] Tests E2E (AC 15) : preview niveau 2 (frais), unitaire happy (e-mail capturé PDF + reminder + audit), suspendue → 422 sans e-mail.
- [x] **T5 — Envoi lot** (AC 10,11,12,13) — `send_reminder_batch`
  - [x] Dédup ids + cap 20 (422 `BATCH_TOO_LARGE`) + pré-check `remaining_slots` (429) + SMTP 412 ; per-facture `send_one_batch_reminder` (gardes pré-SMTP verrou → niveau prochain (NO_NEXT_LEVEL) → contact/render/PDF → check_and_record (RATE_LIMITED) → SMTP → record best-effort). `{ accepted, failed }` HTTP 200.
  - [x] Tests E2E (AC 16) : succès partiel (1 accepted + 1 payée failed INVOICE_ALREADY_PAID, 1 seul e-mail) + cap 21 → 422.
- [x] **T6 — Montage RBAC + doc + gate** (AC 18,19)
  - [x] 3 routes dans `comptable_routes` (avant le `route_layer`). 4 variantes AppError (DunningPaused 422 / LevelAlreadySent 409 / ReminderSentButInvoiceGone 409 / BatchTooLarge 422). CHANGELOG. Gate complet (ci-dessous).

### Review Findings

Code review Pass 1 (2026-07-16, Sonnet — auteur = Opus, biais contourné). 3 couches : Blind Hunter, Edge Case Hunter, Acceptance Auditor. Tous les CRITICAL/HIGH affirmant une absence de code ont été vérifiés par grep ground-truth avant classement.

- [x] [Review][Decision] **Course TOCTOU — envois dupliqués du même niveau** (`blind`+`edge`) — **TRANCHÉ (Guy, 2026-07-16) : risque résiduel accepté étendu au lot.** La mitigation reste l'UI (21-6, anti-double-submit) ; l'invariant AC 12 « le lot n'envoie que le prochain niveau » est donc garanti **en séquentiel, best-effort en concurrent**. Une contrainte `UNIQUE(invoice_id, level_number)` est écartée définitivement — elle contredirait D18 qui autorise la ré-émission au même niveau. À revoir si un ordonnanceur automatique de lots arrive (deux lots concurrents deviendraient alors probables, alors qu'aujourd'hui les deux surfaces sont déclenchées manuellement). Détail original du finding — Le verrou `FOR UPDATE` est relâché (`tx.commit()`, `invoice_email.rs:475` unitaire / `:918` lot) **avant** le SMTP et **avant** l'INSERT du rappel (fait dans une tx neuve par `record_reminder_email:546`). Deux requêtes concurrentes sur la même facture lisent toutes deux `current=0`, calculent `next=1`, passent la garde, et envoient chacune un e-mail + insèrent une ligne `level_number=1`. Pas de contrainte `UNIQUE(invoice_id, level_number)` (migration `20260715000001_invoice_reminders.sql` : index non-uniques seulement) — et une telle contrainte serait **fausse**, D18 autorisant la ré-émission au même niveau. La Pass 1 validate (H1) a documenté ce risque comme **résiduel accepté, mitigé UI 21-6** — mais pour l'**unitaire** seulement. Sur le **lot**, AC 12 dit « le lot n'envoie que le prochain niveau, jamais de ré-envoi » : deux lots concurrents violeraient cet invariant, ce que H1 ne couvre pas explicitement. Décision requise : étendre le risque accepté au lot (documenter), ou fixer (clé d'idempotence / advisory lock).
- [x] [Review][Decision] **Aucune garde serveur « facture échue » avant de facturer des frais de rappel** (`edge`) — **TRANCHÉ (Guy, 2026-07-16) : statu quo, responsabilité de l'appelant.** Le filtre « factures dues » appartient à la couche éligibilité de 21-5a ; le client pioche dans la liste éligible. Permet aussi le rappel de courtoisie (facture due le jour même). **Catégorie C — décision design intentionnelle** (CLAUDE.md §"Tech debt management"), pas une dette : aucune story de remédiation à planifier. Risque assumé et connu : un appel API direct hors UI peut facturer des frais sur une facture non échue. Aucun changement de code — la liste close de gardes d'AC 5 reste la référence. Détail original du finding — `send_reminder:449-473` et `send_one_batch_reminder:894-917` ne vérifient que `status='validated'`, `paid_at`, `dunning_paused_at`. `days_overdue:204-206` **clampe** une échéance future à 0 pour l'affichage sans rejeter. Un Comptable+ peut donc envoyer un rappel « 0 jour de retard » avec frais sur une facture non encore due, et persister une ligne `invoice_reminders` facturable. La liste close de gardes d'AC 5 (item 16) n'inclut pas de garde d'échéance — l'éligibilité est le job de 21-5a et le client choisit les ids. Décision requise : garde serveur (= changement de scope → CR), ou statu quo assumé (le filtre « dues » reste la responsabilité de l'appelant).
- [x] [Review][Patch] **`FailedReminder` omet le champ obligatoire `details`** [`crates/kesh-api/src/routes/invoice_email.rs:816`] — Vérifié grep : la struct n'a que `invoice_id` + `error_code`, alors que le `FailedProposal` canonique Epic 8 (`reconciliation.rs:154-158`) porte `details: Option<serde_json::Value>`. Viole CLAUDE.md §"Pattern batch — FailedProposal per-proposal" (champ listé « obligatoire ») **et** AC 12, qui écrit la forme `FailedProposal { invoice_id, error_code, details? }`.
- [x] [Review][Patch] **Matrice de tests AC 15/16 largement absente** [`crates/kesh-api/tests/invoice_send_email_e2e.rs:1179+`] — Vérifié grep par zone : la zone rappels (≥1179) ne contient que `DUNNING_PAUSED` (1286, 1311) et un `INVOICE_ALREADY_PAID` (1364, côté **lot** seulement). Les occurrences de `CONTACT_EMAIL_MISSING` (479, 521) et `MockMailer::failing` (682, 687) sont **toutes** dans la zone Epic 20 préexistante — elles ne couvrent pas les rappels. Aucun 403/412/404 dans la zone rappels. Manquent, AC 15 : payée→422 unitaire, `CONTACT_EMAIL_MISSING`→422, SMTP non configuré→412, contenu vide→422, niveau>prochain→refus, ré-envoi ≤ prochain autorisé, **SMTP down→500 + rien enregistré** (`MockMailer::failing` — c'est précisément le comportement que le patch CRITICAL C1 devait garantir, et il est non testé), anti-IDOR→404, RBAC Consultation→403, facture supprimée post-SMTP→409. AC 16 : pré-check capacité→429 sans e-mail, cross-tenant→`INVOICE_NOT_FOUND`, RBAC→403 ; le succès partiel testé est 1 accepted/1 failed au lieu du 1 accepted/2 failed spécifié.
- [x] [Review][Patch] **`retry_after: 1` codé en dur sur le rejet rate-limit du lot** [`crates/kesh-api/src/routes/invoice_email.rs:856`] (`blind`+`edge`) — L'unitaire propage la vraie valeur (`retry_after: reject.retry_after_secs`, lignes 432 et 668) ; le lot renvoie toujours `1`. Or `remaining_slots` peut valoir 0 parce que la clé est **bloquée** jusqu'à `block_duration` (1800 s dans le test du rate-limiter). `AppError::RateLimited` mappe `retry_after` verbatim dans l'en-tête HTTP (`errors.rs:925-936`) : un client qui respecte l'en-tête réessaiera chaque seconde pendant 30 min.
- [x] [Review][Patch] **Rétrofit du log INFO sur l'envoi facture Epic 20 non fait** [`crates/kesh-api/src/routes/invoice_email.rs:647-790`] — Vérifié grep : `send_invoice_email` n'a que `tracing::warn!` (661) et `tracing::error!` (739, 774), aucun `tracing::info!` sur le chemin de succès. Les handlers rappels loguent bien (527, 999). AC 9 demande explicitement « appliquer **aussi** à l'envoi facture Epic 20 qui ne loggue rien aujourd'hui ».
- [x] [Review][Patch] **Erreurs DB génériques du lot encapsulées per-facture au lieu d'escalader en 500** [`crates/kesh-api/src/routes/invoice_email.rs:894,897,913,920,933,945`] — 6 sites `.map_err(|_| "DATABASE_ERROR".to_string())?` transforment un pool fermé / deadlock / connexion perdue en `FailedProposal { error_code: "DATABASE_ERROR" }` dans un HTTP 200. CLAUDE.md réserve explicitement le 500 global à « DB pool fermé, panic, IO catastrophique ». `DATABASE_ERROR` ne figure d'ailleurs pas parmi les codes énumérés par AC 12.
- [x] [Review][Patch] **Les échecs de rendu PDF du lot s'effondrent sur un code unique** [`crates/kesh-api/src/routes/invoice_email.rs`] — `invoice_pdf_service::render` échoue pour des causes distinctes (facture absente, non validée, trop de lignes, pas de compte bancaire) toutes mappées sur `"INVOICE_NOT_PDF_READY"`. AC 12 énumère `INVOICE_NOT_PDF_READY` **et** `INVOICE_NOT_VALIDATED` comme codes distincts. Sévérité LOW (le placement en `failed[]` et le HTTP 200 restent corrects).

**Écarté (1)** — *Slot rate-limit consommé avant les validations peu coûteuses* (`blind`, MEDIUM) : le Blind Hunter reproche à `send_reminder:426-442` d'appeler `check_and_record` (qui consomme un slot) avant les checks `smtp_ready` et `level_number < 1`. **Faux positif** : AC 5 **prescrit** cet ordre (« 1. auth/tenant, 2. rate-limit 429, 3. SMTP prêt 412, 4. facture 404 »), explicitement calqué sur `send_invoice_email` d'Epic 20 (item 16). Le code **suit** la spec scellée ; changer l'ordre serait la dévier. Le Blind Hunter, aveugle à la spec par construction, ne pouvait pas le savoir — c'est le comportement attendu de cette couche.

## Dev Notes

### Ground-truth (2026-07-16, 2 agents Explore)

**Pipeline envoi Epic 20** (`routes/invoice_email.rs`) : `send_invoice_email:244` — gardes ordonnées auth/tenant(`get_company_for:250`) → **rate-limit 429** (`rate_limiter_send_email.check_and_record((company.id,user_id)):254`) → **SMTP 412** (`smtp_ready:271`) → facture 404 (`find_by_id_with_lines:279`) → contact (`load_active_contact:282`) → **`locked_recipient:73`** (contacts.email trim non-vide ; **pas de champ `to` au payload**, `SendInvoiceEmailRequest`=subject+body `:54`) → contenu vide 422 (`:287`) → PDF (`invoice_pdf_service::render:296`). **Inversion** : `state.mailer.send_email(&email).await?:317` PUIS enregistrement `:323`. Cas facture disparue #219 → trace best-effort + 409. **Preview** `preview_invoice_email:201` : rend serveur (`get_effective` + `build_invoice_vars` + `render:225`), DTO `EmailPreviewResponse { to: Option<String>, language, subject, body }:40`.

**Vars** : `build_invoice_vars:159` (privé, 6 vars, `amount`=TTC via `invoice_total_ttc:185`, `format_money`/`format_date` kesh-i18n), `salutation_line:107` (civilité×langue×type #12), `resolve_language:67`. `build_reminder_vars` (nouveau) réutilise ça + ajoute reminderLevel/Fee/totalDue/daysOverdue.

**Moteur** (`kesh-core/email_template_engine.rs`) : `render(template:&str, vars:&HashMap<String,String>) -> String:56` single-pass, jamais de re-scan des valeurs substituées, token inconnu laissé littéral. `validate_tokens:34` (au save seulement, PAS au send).

**Templates rappel** : `email_templates::get_effective(pool, company_id, EmailTemplateType::InvoiceReminder, language, N):106` — cascade override(type,langue,N)→override(type,langue,0)→`reminder_default(langue,N):71`→générique. `EffectiveEmailTemplate.level_number` = slot demandé. `allowed_variables()` InvoiceReminder = 10 vars **déjà déclarées 21-3** (`email_template.rs:63`). Défauts : niveaux 1/2/3 + générique ; `{reminderFee}` aux niveaux 2/3 ; `{reminderLevel}` **déclaré mais non utilisé** par les défauts (l'alimenter quand même).

**TTC** : `kesh_core::accounting::vat::invoice_total_ttc(lines: (Decimal line_total, Decimal vat_rate)) -> Decimal` (vat.rs:62, TVA arrondie par ligne). `{totalDue}` = TTC + Σ frais non-annulés + frais niveau (pas de helper somme existant — composer). `{amount}`=TTC facture.

**Rate-limiter** : `build_send_email_rate_limiter:lib.rs:145` (20/15min, clé `(company_id,user_id)`), `AppState.rate_limiter_send_email: Arc<RateLimiter<(i64,i64)>>:62`. `check_and_record(key):201` consomme (rejette sans consommer). `check_rate_limit(key):179` pré-check **binaire** (bloqué ou non), **PAS de compte de slots restants** → **ajouter `remaining_slots(key) -> u32`** (`max_attempts:privé - recent_count:134` — trivial à exposer).

**Mailer** : `state.mailer.send_email(&OutgoingEmail):mail/mod.rs:91`. `OutgoingEmail { to, subject, body, from_display_name, reply_to, attachment: Option<EmailAttachment{filename, content_type, bytes}> }:54` (struct l.54, l.53 = commentaire). MockMailer en `KESH_TEST_MODE` + `GET /api/v1/_test/sent-emails` (8 champs camelCase). `NoopMailer` renvoie Ok silencieux → d'où la garde 412 obligatoire. `smtp_ready:lib.rs:75` → 412 `SMTP_NOT_CONFIGURED` (`errors.rs:265`).

**PDF** : `invoice_pdf_service::render(pool, i18n, locale, company, invoice_id) -> RenderedInvoicePdf { bytes, filename_base }:68`. attachment = `facture-{filename_base}.pdf`. Contraintes : validated, ≤9 lignes (`MAX_LINES_PER_PDF`), contact, compte bancaire primary.

**Enregistrement** : PAS `mark_emailed` mais `invoice_reminders::insert_in_tx(&mut tx, &NewInvoiceReminder{ channel: ReminderChannel::Email, sent_to: Some(to), … }):repositories/invoice_reminders.rs:104` + audit `invoice.reminder_sent`. `ReminderChannel::Email` existe (`entities/invoice_reminder.rs:19`). Patron déjà en place pour le manuel (`routes/dunning_reminders.rs:299`, channel=Manual). `find_scoped_for_update_in_tx` + `current_level_in_tx` (21-5a) réutilisés.

### Patterns à réutiliser (ne PAS réinventer)

- **Envoi + gardes** : `send_invoice_email` (ordre 8 gardes + inversion SMTP). Ne pas changer l'ordre.
- **Batch `{accepted,failed}`** : `FailedProposal { business_id, error_code: String, details: Option<Value> }` (business id = `invoice_id`), HTTP 200 succès partiel, cross-tenant → même code que not-found (CLAUDE.md pattern batch + `reconciliation.rs::accept_batch`). Pas d'`unreachable!()` aux variants.
- **Éligibilité / niveau** : `dunning_eligibility` (liste), `invoice_reminders::current_level_in_tx`, `dunning_levels::find_by_level_number`/`list_all_by_company` (prochain niveau) — 21-5a.
- **Erreurs → HTTP (M3, clarifié)** : distinguer **variantes `AppError`** (rejets HTTP unitaire/preview/garde-globale) des **`error_code` string de `FailedProposal`** (lot, jamais mappés HTTP — le lot est toujours 200) :
  - **Variantes `AppError` à AJOUTER** : `DunningPaused` (422, unitaire), `LevelAlreadySent` (409, unitaire — saut de niveau), `ReminderSentButInvoiceGone` (409, post-SMTP facture disparue), `BatchTooLarge` (422, garde globale lot). Existantes réutilisées : `InvoiceAlreadyPaid` (422), `DunningLevelNotFound` (422), `InvoiceNotValidated` (400), `ContactEmailMissing`, `InvoiceEmailEmptyContent` (422), `SmtpNotConfigured` (412), `RateLimited` (429).
  - **`error_code` string `FailedProposal` UNIQUEMENT (PAS de variante AppError — éviter le code mort)** : `NO_NEXT_LEVEL`, `REMINDER_CONTENT_EMPTY`, et les réutilisations `INVOICE_NOT_FOUND`/`INVOICE_ALREADY_PAID`/`DUNNING_PAUSED`/`CONTACT_EMAIL_MISSING`/`INVOICE_NOT_PDF_READY`/`RATE_LIMITED`/`SMTP_SEND_FAILED` (chaînes constantes dans le mapper `DbError`/erreur → `FailedProposal`). Cross-tenant → `INVOICE_NOT_FOUND`.
- **RBAC** : `require_comptable_role` (`lib.rs`), routes AVANT le `route_layer` (anti-footgun).

### Pièges identifiés

- **`totalDue` double comptage** : calculer AVANT l'insertion du rappel courant (le body est snapshoté). Filtrer `cancelled_at.is_none()`.
- **Pré-check lot AVANT SMTP** : ne jamais démarrer un lot qui va se bloquer à mi-course (429 après k envois gèlerait 15 min). Cap 20 borne aussi la durée HTTP.
- **`DUNNING_PAUSED` AppError manquant** : l'ajouter (422) — utilisé unitaire ET code `FailedProposal` lot.
- **`send-batch` niveau prochain uniquement** ; l'unitaire autorise ≤ prochain (D18). Le saut vers le haut = rappel manuel (21-5a).
- **Log INFO** : ajouter aussi à l'envoi facture Epic 20 (item 21 rétro-actif).
- **Leçon 21-5a** : propagation cross-crate — mais ici pas de nouveau champ d'entité. Le gate workspace complet reste le filet.

### Hors scope (garde-fous anti-creep)

- **AUCUN frontend** (écran relances, boutons envoi/lot, modale) → **21-6**.
- **AUCUNE balance âgée** → 21-7.
- **AUCUNE nouvelle table/colonne/migration** (réutilise `invoice_reminders` 21-5a).
- **Refactor page settings/email-templates multi-type** (item 20) = **déjà fait en 21-4** (frontend) — rien ici.
- Pas de HTML e-mail (text/plain, hérité Epic 20).

### Règle de splitting (CLAUDE.md)

Modules : `kesh-api/{routes/invoice_email ou dunning_reminders, middleware/rate_limit, errors, lib}` + réutilisation `kesh-db`/`kesh-core` sans modif. ~4 modules — sous le seuil. Si `validate` > 4 passes → envisager split preview+unitaire / lot. Suivre le plan par défaut.

### Project Structure Notes

- **Nouveaux** : `build_reminder_vars` + handlers `reminder_preview`/`send_reminder`/`send_reminder_batch` (dans `routes/invoice_email.rs` ou `routes/dunning_reminders.rs` — à trancher validate ; cohérence : les data-endpoints rappels sont dans `dunning_reminders.rs`, le socle envoi dans `invoice_email.rs`). DTOs request/response. `RateLimiter::remaining_slots`. Variantes AppError à ajouter (M3) : `DunningPaused` (422), `LevelAlreadySent` (409), `ReminderSentButInvoiceGone` (409), `BatchTooLarge` (422). **PAS** `NoNextLevel` (string `FailedProposal` seulement). Tests e2e.
- **Modifiés** : `lib.rs` (3 routes Comptable+), `errors.rs` (variantes + arms 422/409), `routes/invoice_email.rs` (log INFO envoi facture), `CHANGELOG.md`.

### References

- [Source: epic-21-echeances-relances.md#C items 14-19,21,22 ; #B item 13 (totalDue)]
- [Source: 21-5a-donnees-eligibilite-relances.md — invoice_reminders, FailedProposal, find_scoped_for_update_in_tx, éligibilité]
- [Source: routes/invoice_email.rs:107,159,201,244 — salutation_line/build_invoice_vars/preview/send + inversion SMTP]
- [Source: kesh-core/email_template_engine.rs:34,56 ; accounting/vat.rs:62 — render/validate/TTC]
- [Source: repositories/email_templates.rs:106 ; entities/email_template_defaults.rs:71 ; email_template.rs:63 — cascade/défauts/allowed_vars]
- [Source: middleware/rate_limit.rs:179,201 ; lib.rs:62,145 — limiteur send-email]
- [Source: mail/mod.rs:53,91 ; routes/invoice_pdf_service.rs:68 ; test_endpoints.rs:359 — mailer/PDF/MockMailer]
- [Source: repositories/invoice_reminders.rs:104 ; entities/invoice_reminder.rs:19 ; routes/dunning_reminders.rs:299 — insert channel Email + audit]
- [Source: CLAUDE.md — pattern batch FailedProposal, Test Locally First, splitting]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.8 (1M context) — session `01FL3zWi6FC6ZPvEjZnFfPmR` (source : trailers `Co-Authored-By` des 3 commits d'implémentation).

### Debug Log References

Aucun incident de debug notable pendant l'implémentation.

**Note de traçabilité** : la session de dev a **crashé après le dernier commit de code** (`bc32f6e1`, 2026-07-16 12:41) mais **avant** la clôture de la story. Le Dev Agent Record, le File List, cette entrée de Change Log et le passage du statut en `review` ont été reconstitués le 2026-07-16 dans une session ultérieure (Opus 4.8), à partir de faits vérifiés — diff des 3 commits, trailers Git, et exécution effective du gate — et non de la mémoire de la session crashée. Vérification d'intégrité post-crash : arbre de travail propre, `git fsck --connectivity-only` sans erreur, aucun fichier orphelin ni opération Git interrompue, aucun fichier modifié après le dernier commit. **Aucun code perdu.**

### Completion Notes List

- **T1** — `build_reminder_vars` construit les 10 variables (6 de base réutilisées d'Epic 20 via `build_invoice_vars` + `reminderLevel`/`reminderFee`/`totalDue`/`daysOverdue`), builder pur : les 4 vars rappel sont pré-calculées par l'appelant. `sum_fees_deduped_excluding` implémente la dédup H2 (`SUM(MAX(fee) GROUP BY level_number)`, exclut annulés + niveau courant). `days_overdue` clampé UTC.
- **T2** — `RateLimiter::remaining_slots` expose les slots restants (purge + `max_attempts.saturating_sub(recent_count)`, 0 si bloqué) pour le pré-check du lot.
- **T3** — `GET .../reminder-preview?level=N` : `ReminderLevelQuery` sans `serde(default)` → 400 si `level` absent (patch M1 de la Pass 1 validate).
- **T4** — `send_reminder` suit le flux scellé par le CRITICAL C1 : gardes globales → verrou bref pré-SMTP (re-check statut/paid/paused/niveau) → **relâche du verrou** → contact/contenu/PDF → SMTP → enregistrement **best-effort**. Jamais de rejet post-SMTP : facture disparue après envoi → `ReminderSentButInvoiceGone` 409 **avec trace d'audit**.
- **T5** — `send_reminder_batch` respecte le pattern batch CLAUDE.md : `{ accepted, failed }` en **HTTP 200**, erreurs per-facture encapsulées en `FailedProposal` (identifiant business `invoiceId`, jamais d'index positionnel). Les 4 `AppError` globales (cap 20 dépassé, rate-limit, SMTP indisponible, body invalide) restent conformes aux exceptions documentées.
- **T6** — 3 routes montées dans `comptable_routes` **avant** le `route_layer` (RBAC). 4 variantes `AppError` ajoutées. CHANGELOG mis à jour dans le commit `bc32f6e1`.
- **Risque résiduel documenté** (hérité, non introduit par 21-5b) : crash-process entre SMTP-OK et commit → e-mail parti sans trace. Exposition dual-write d'Epic 20, hors scope (remédiation = outbox). Cf. Pass 3 validate.
- **Aucune migration** dans cette story → pas de bump `kesh_version_min_required` ni d'entrée d'audit d'idempotence requis.

### File List

Hors artefacts BMAD — 8 fichiers, +1079/-2 lignes sur les 3 commits (`40edf89d`, `32a4233e`, `bc32f6e1`) :

- `crates/kesh-api/src/routes/invoice_email.rs` (+676) — `build_reminder_vars`, `days_overdue`, `render_reminder`, `send_reminder`, `send_reminder_batch`, `send_one_batch_reminder` + tests unitaires.
- `crates/kesh-api/src/errors.rs` (+46) — variantes `DunningPaused` 422 / `LevelAlreadySent` 409 / `ReminderSentButInvoiceGone` 409 / `BatchTooLarge` 422.
- `crates/kesh-api/src/middleware/rate_limit.rs` (+43) — `remaining_slots` + test.
- `crates/kesh-api/src/lib.rs` (+13) — montage des 3 routes dans `comptable_routes`.
- `crates/kesh-db/src/repositories/invoice_reminders.rs` (+26) — `sum_fees_deduped_excluding`.
- `crates/kesh-api/tests/invoice_send_email_e2e.rs` (+222) — 4 tests E2E (AC 15/16).
- `crates/kesh-db/tests/invoice_reminders_repository.rs` (+54) — test repo dédup.
- `CHANGELOG.md` (+1).

## Change Log

### Code review Pass 3 (2026-07-16, Opus 4.8, contexte frais) — 3 MEDIUM + 2 LOW réels, patchés ; 4 écartés

Passe de scellage 3 couches en **Opus** (rotation complète Sonnet → Haiku → Opus), justifiée par le revirement de conception de la Pass 2. Diff unique aplati (1945 lignes, patches Pass 2 inclus). **La passe n'est pas revenue propre** — elle a trouvé un défaut de classe C1 que les deux passes précédentes avaient manqué.

**MEDIUM — contenu non borné → e-mail parti sans trace, rejouable à l'infini** (Edge Case Hunter). `send_reminder` n'imposait **aucune borne de longueur** sur `subject`/`body`, écrits dans les colonnes `TEXT` (65 535 octets) d'`invoice_reminders` **après** le SMTP. Un corps de 70 000 caractères (extrait de CGV collé) : toutes les gardes passent → e-mail réellement délivré au débiteur → INSERT rejeté par MariaDB **1406** → 409 `REMINDER_SENT_BUT_INVOICE_GONE` (mensonger, la facture est intacte) → **aucune ligne `invoice_reminders`** → le niveau n'avance pas → l'opérateur réessaie → **le débiteur reçoit un e-mail de plus à chaque tentative, sans qu'aucune trace ne soit jamais écrite**. C'est exactement la classe de bug que le patch CRITICAL C1 devait éliminer (« toute vérification pouvant rejeter la requête doit être AVANT le SMTP ») : la condition est connaissable pré-SMTP mais n'était découverte que post-SMTP. Déterministe, donc hors du risque TOCTOU accepté. Aggravant : **21-5a a déjà la garde sur la même table** (`validate_note_len` + `REMINDER_NOTE_MAX = 5000`, `dunning_reminders.rs:136`) — 21-5b écrivait deux colonnes TEXT de plus sans l'équivalent. L'ECH a vérifié le mode strict sur la MariaDB du projet (`STRICT_TRANS_TABLES` → erreur 1406, pas de troncature). **Patch** : helper partagé `helpers::validate_text_len` (DRY — `validate_note_len` y délègue désormais), bornes `REMINDER_SUBJECT_MAX = 500` / `REMINDER_BODY_MAX = 10_000` appliquées **pré-SMTP** ; bornes en caractères choisies pour que le pire cas UTF-8 (4 o/car) reste sous 65 535. Équivalent per-facture côté lot (`REMINDER_CONTENT_TOO_LONG`).

**MEDIUM — lot > quota d'envoi → 429 perpétuel** (Blind Hunter). Le pré-check compare `ids.len()` à `remaining_slots`, mais si le lot dépasse `max_attempts`, **aucune attente ne peut le faire passer** : la fenêtre ne libère jamais plus de `max_attempts` slots. Avec une fenêtre vierge, `retry_after_secs` renvoie 0 → `.max(1)` → `Retry-After: 1` — précisément la pathologie que le patch Pass 1 n°3 prétendait avoir supprimée. **Patch** : `RateLimiter::max_attempts()` exposé ; lot > quota → **422 `BATCH_EXCEEDS_SEND_QUOTA`** (erreur client définitive) au lieu d'un 429 (« réessaie ») trompeur.

**MEDIUM — panne SMTP en cours de lot invisible** (Acceptance Auditor). `send_email(...).is_err()` **jetait l'erreur**. L'unitaire est loggué parce que l'`AppError` traverse `IntoResponse` (`errors.rs:1212`), mais le lot ne construit jamais cette `AppError` et `crates/kesh-api/src/mail/` ne loggue rien. Une panne SMTP produisait donc 20 `SMTP_SEND_FAILED` dans un HTTP 200 et **zéro ligne de log** — ce qui vidait de sa substance la compensation `infra()` de la Pass 2. **Patch** : erreur liée et journalisée en `error!` ; le code reste per-facture (AC 12 l'impose).

**LOW — `ReminderSentButInvoiceGone` diagnostiquait une disparition inexistante**. Le bras `Err(e)` couvrait **toute** panne d'enregistrement (deadlock, timeout, commit) en annonçant « la facture a disparu » — le commentaire du code l'admettait (« ou hoquet DB »). L'opérateur partait chercher une suppression qui n'avait pas eu lieu. Le lot, lui, est honnête (`RECORD_FAILED_EMAIL_SENT`). **Patch** : `NotFound` → `ReminderSentButInvoiceGone` (exact) ; toute autre cause → nouvelle variante **`ReminderSentButNotRecorded`** (409), pendant unitaire du code du lot.

**LOW — bras mort sur le lookup `level_fee`**. `next_reminder_level` sélectionne `next_level` **dans** `levels`, donc le `find` suivant ne peut pas échouer : le `ok_or_else` était inatteignable, et émettait un `NO_NEXT_LEVEL` au `details` incompatible avec le vrai (`levelNumber` vs `currentLevel`). **Patch** : routé vers `infra()` — un invariant rompu par un futur refactor serait un bug structurel, donc loggué (CLAUDE.md).

**Écartés (4)** :
- **`render_reminder` → `infra()` serait un abus** (Blind Hunter, LOW) — **faux positif réfuté par lecture du code** : `email_templates::get_effective` ne renvoie **jamais** `NotFound`, il retombe toujours sur `to_effective_default` (`email_templates.rs:126-129`), qui n'est même pas un `Result`. Les seuls `?` de `render_reminder` sont deux appels DB → `infra()` y est correct. Le BH, aveugle au codebase, a supposé un échec de résolution de template impossible.
- **Course TOCTOU** et **absence de garde « facture échue »** (Blind Hunter, MEDIUM ×2) — décisions déjà tranchées (Pass 1/2). La couche aveugle les re-trouve par construction.
- **Slot rate-limit consommé avant les validations** (Blind Hunter, LOW) — **3e remontée consécutive** (Sonnet P1, Haiku P2, Opus P3). Même réfutation : AC 5 prescrit cet ordre. Trois couches aveugles indépendantes convergent sur le même faux positif : c'est structurel à un reviewer sans spec, pas un signal. *À noter pour les futures passes : ce finding est attendu et n'a pas besoin d'être re-vérifié.*

**Écarts spec/code documentés (Acceptance Auditor, LOW — doc uniquement, aucun changement de code)** :
- **AC 8 dit 200, le code renvoie 201** sur `send_reminder`. Le code est défendable et cohérent : le sibling 21-5a qui crée la même ressource renvoie aussi 201 (`dunning_reminders.rs:337`). Même classe que les 3 écarts déjà documentés — la spec est imprécise.
- **AC 5 ordonne « facture 404 » avant « niveau inexistant 422 »** ; le code fait l'inverse. Sans impact ni oracle : le lookup de niveau est scopé à la config company et indépendant de la facture, donc un id cross-tenant donne le même 422 qu'un id possédé. Placer le check de config, moins coûteux, en premier est raisonnable.
- **`RECORD_FAILED_EMAIL_SENT`** n'était énuméré nulle part. Comportement correct, liste de la spec en dérive.

**Extension d'AC 12 (cumulée)** : `CONTACT_ARCHIVED`, `DATABASE_ERROR` (Pass 2) + `REMINDER_CONTENT_TOO_LONG`, `RECORD_FAILED_EMAIL_SENT` (Pass 3). Toutes additives — la facture est écartée du lot dans tous les cas, seul le code est plus précis.

**Tests ajoutés (2)** : `send_reminder_oversized_content_rejected_before_smtp` (corps > 10 000 et objet > 500 → 400 **sans qu'aucun e-mail parte**, + la borne haute pile-poil reste acceptée, donc pas de régression sur un contenu légitime) et `send_reminder_batch_over_send_quota_returns_422`.

**Trend** : P1 (3H/4M/1L) → P2 (1H) → **P3 (3M/2L)** → patchés. La Pass 3 a trouvé **plus** que la Pass 2 : le scellage était justifié, et Opus a vu ce que Sonnet et Haiku avaient tous deux manqué. Findings > LOW subsistants : aucun. **Une Pass 4 est requise** par la Review Iteration Rule (P3 a remonté des MEDIUM) — LLM : Sonnet (reprise du cycle), contexte frais.

### Code review Pass 2 (2026-07-16, Haiku 4.5, contexte frais) — 1 HIGH convergent (patch Pass 1 incomplet) + 1 décision de conception révisée

Passe adversariale 3 couches en **Haiku** (rotation Sonnet → Haiku ; l'auteur reste Opus). **Diff unique aplati** `5bdce277..HEAD` (1866 lignes) — mitigation CLAUDE.md contre le bug d'indexation multi-commit de Haiku, d'autant plus nécessaire ici que les patches Pass 1 re-touchent des hunks écrits par les commits d'implémentation. **Aucune hallucination de type « REGRESSION-P1 » n'a été observée** : la mitigation a tenu.

- **Acceptance Auditor : 0 finding.** 19/19 AC implémentés, les 7 patches Pass 1 vérifiés par grep, les 3 déviations documentées correctement non re-litigées.
- **Blind Hunter + Edge Case Hunter : convergence sur 1 HIGH réel** — et il visait le **patch Pass 1 lui-même**.

**Le finding (confirmé par grep)** : le patch Pass 1 n°5 avait corrigé 6 sites `DATABASE_ERROR` mais en laissait **2 identiques** : `load_active_contact` (`|_| CONTACT_EMAIL_MISSING`) et le bras `_` du rendu PDF (`INVOICE_NOT_PDF_READY`) avalaient toujours les erreurs d'infrastructure. Incohérence interne : 6 sites escaladaient, 2 non.

**Ce que la correction a révélé de plus profond** : la boucle faisait `Err(Fatal(e)) => return Err(e)`, ce qui **jetait `accepted`** — si la facture n°1 part réellement et que la n°2 tombe sur un pool mort, le client reçoit un 500 et n'apprend jamais que la n°1 a été relancée, alors que l'e-mail est parti. Or CLAUDE.md réserve les exceptions 500 aux erreurs qui invalident la requête **« en amont du traitement per-proposal »** — et un pool déjà mort est bien attrapé en amont, par les `?` qui précèdent la boucle (`get_company_for`, `list_all_by_company`). Le patch Pass 1 n°5 avait donc **sur-appliqué** la règle sur un cas qu'elle ne visait pas.

**Décisions (Guy)** :
- **Escalade en cours de lot → abandonnée** (revirement assumé sur le patch Pass 1 n°5). Les erreurs d'infra survenant dans la boucle restent per-facture (`DATABASE_ERROR`), ce qui préserve `accepted`. La variante `BatchItemError::Fatal` disparaît ; le type redevient une struct `{ error_code, details }`. **Compensation** : `BatchItemError::infra()` journalise en `tracing::error!` (facture, contexte, erreur) — la panne reste visible à l'exploitant, ce qui était la vraie inquiétude de l'Edge Case Hunter, sans sacrifier le rapport de lot.
- **Codes contact honnêtes** : `CONTACT_ARCHIVED` ajouté. `|_| CONTACT_EMAIL_MISSING` mentait — un contact archivé a le plus souvent un e-mail, et le message envoyait l'utilisateur corriger le mauvais problème.

**Correction apportée par Guy en cours de patch** (« on ne devrait pas pouvoir créer une facture pour un contact archivé ou inexistant ») — vérifiée, et juste à moitié :
- **`CONTACT_NOT_FOUND` abandonné** : le FK `invoices.contact_id … ON DELETE RESTRICT` (`20260416000001_invoices.sql:32`) interdit de supprimer un contact ayant des factures. Un `NotFound` ici signalerait une **incohérence de données**, pas un cas métier → routé vers `infra()` (loggué). C'était bien du code mort.
- **`CONTACT_ARCHIVED` conservé, car réellement atteignable** : la création de facture rejette bien un contact archivé (`invoices.rs:399`), **mais `contacts::archive` (`contacts.rs:524`) ne vérifie pas l'existence de factures ouvertes** — seulement existence, double-archivage et version. Le chemin « facture créée avec contact actif → contact archivé plus tard → rappel » est donc ouvert, et Epic 20 le teste déjà (`archived_contact_returns_400`).
- **Écarté (1)** — *slot rate-limit consommé avant les validations* : re-remonté par le Blind Hunter Haiku, comme en Pass 1 par le Blind Hunter Sonnet. Même réfutation : AC 5 prescrit cet ordre. Deux couches aveugles indépendantes convergent sur le même faux positif — c'est le comportement attendu d'un reviewer sans spec, pas un signal.

**Extension d'AC 12 assumée** : la liste des codes per-facture gagne `CONTACT_ARCHIVED` et `DATABASE_ERROR` (ce dernier redevenant légitime puisque l'infra ne s'escalade plus). Additif — aucun changement de comportement (la facture est écartée du lot dans tous les cas), seul le code est plus précis.

**Test ajouté** : `send_reminder_archived_contact_reports_archived_not_missing_email` — contact archivé après création, sur les 2 surfaces (unitaire 400 / lot `failed[]` 200), avec assertion explicite que le code n'est **pas** `CONTACT_EMAIL_MISSING`.

**Trend** : Pass 1 (3H/4M/1L) → Pass 2 (1H convergent, 0 CRITICAL, Acceptance Auditor 0 finding) → patché. Findings > LOW subsistants : aucun. LLM : Sonnet → Haiku. Une Pass 3 (Opus, contexte frais) est justifiée par le **revirement de conception** sur l'escalade du lot — un changement structurel mérite une passe de scellage, comme en validate Pass 3.

### Code review Pass 1 (2026-07-16, Sonnet 4.6) — 3 HIGH + 4 MEDIUM + 1 LOW → 2 décisions + 6 patches appliqués

Passe adversariale 3 couches (Blind Hunter / Edge Case Hunter / Acceptance Auditor), toutes en **Sonnet** — l'implémentation étant d'**Opus 4.8**, la Review Iteration Rule impose un modèle orthogonal. Diff unique aplati `5bdce277..bc32f6e1` (1213 lignes, artefacts BMAD exclus) plutôt que la séquence de 3 commits.

**Grep ground-truth** appliqué aux 3 HIGH affirmant une absence de code — il a tranché dans les deux sens :

- **Confirmé** `FailedReminder` sans `details` (struct à 2 champs vs `reconciliation::FailedProposal:154-158` à 3).
- **Confirmé** la matrice de tests AC 15/16 absente, par analyse **par zone de lignes** : `CONTACT_EMAIL_MISSING` (479, 521) et `MockMailer::failing` (682, 687) existent bien dans le fichier mais **uniquement dans la zone Epic 20 préexistante** ; la zone rappels (≥1179) n'avait que `DUNNING_PAUSED` et un `INVOICE_ALREADY_PAID` côté lot. Sans le découpage par zone, un simple `grep -c` aurait conclu à tort que les scénarios étaient couverts.
- **Confirmé** le rétrofit AC 9 manquant (`send_invoice_email` : seulement `warn`/`error`, aucun `info` sur succès).
- **Réfuté** le MEDIUM du Blind Hunter sur l'ordre des gardes (rate-limit avant validation) : AC 5 **prescrit** cet ordre mot pour mot, calqué sur Epic 20 (item 16). Le code suit la spec scellée. Faux positif structurel de la couche aveugle, pas une hallucination — écarté.

**Décisions (Guy)** :
- **TOCTOU envois dupliqués** → risque résiduel accepté **étendu au lot** (H1 de la validate ne couvrait que l'unitaire). Mitigation UI 21-6. `UNIQUE(invoice_id, level_number)` écarté définitivement : contredirait D18 (ré-émission au même niveau légitime). À revoir si un ordonnanceur automatique de lots arrive.
- **Garde « facture échue »** → statu quo, responsabilité de l'appelant (l'éligibilité est le job de 21-5a). **Catégorie C — décision design intentionnelle**, pas une dette.

**Patches appliqués (6)** :
1. `FailedReminder` gagne `details: Option<serde_json::Value>` — signature canonique Epic 8 respectée. Renseigné où c'est informatif (`NO_NEXT_LEVEL` → `currentLevel`, `RATE_LIMITED` → `retryAfterSecs`), `None` sinon.
2. **Matrice de tests AC 15/16** — 9 tests E2E ajoutés (détail ci-dessous).
3. `retry_after` du lot : nouvelle méthode `RateLimiter::retry_after_secs` (blocage restant, sinon sortie de fenêtre glissante) au lieu du `1` codé en dur qui faisait marteler un client obéissant pendant 30 min. + 2 tests unitaires.
4. Rétrofit AC 9 : `tracing::info!` sur le succès de `send_invoice_email` (Epic 20) ; `company_id` + `to` ajoutés aux 2 logs de rappel, qui n'avaient pas le destinataire exigé par AC 9.
5. Nouveau `BatchItemError { Failed { error_code, details } | Fatal(AppError) }` : les 6 sites `.map_err(|_| "DATABASE_ERROR")` escaladent désormais en `AppError::Internal` (500 global), conformément à CLAUDE.md qui réserve le 500 aux erreurs d'infrastructure. `DATABASE_ERROR` disparaît des codes per-facture (il ne figurait pas dans AC 12). L'échec d'enregistrement **post-SMTP** reste per-facture — escalader y perdrait les factures déjà acceptées du lot.
6. Rendu PDF du lot : `INVOICE_NOT_VALIDATED` / `INVOICE_NOT_FOUND` distingués de `INVOICE_NOT_PDF_READY` (AC 12 énumère les codes séparément).

**Tests ajoutés (11 : 9 E2E + 2 unitaires)** : `send_reminder_paid_invoice_rejected_before_smtp`, `send_reminder_contact_without_email_rejected`, `send_reminder_empty_content_returns_422`, `send_reminder_level_skip_rejected_but_resend_allowed` (D18 : saut 409 + ré-émission autorisée), `send_reminder_smtp_down_returns_500_and_records_nothing` (**la garantie du patch C1, jusqu'ici non testée**), `send_reminder_smtp_not_configured_returns_412` (+ garde lot), `send_reminder_cross_tenant_is_invisible` (404 unitaire / `INVOICE_NOT_FOUND` lot), `send_reminder_consultation_role_returns_403` (3 routes), `send_reminder_batch_capacity_precheck_returns_429` (+ assertion `Retry-After > 1` verrouillant le patch 3), `send_reminder_batch_partial_success` étendu au 1 accepted / 2 failed d'AC 16. Unitaires : `retry_after_secs_reflects_block_then_window`, `retry_after_secs_uses_window_when_not_blocked`.

**Écarts spec/code assumés, tranchés en faveur du code** (à corriger dans la spec, pas dans le code) :
- AC 15 annonce `CONTACT_EMAIL_MISSING` → **422** ; la variante `AppError::ContactEmailMissing` est **partagée avec Epic 20** et renvoie **400** (`errors.rs:1015`, attesté par `contact_without_email_returns_400`). Changer le code casserait Epic 20 pour un gain nul.
- AC 5 annonce `status != 'validated'` → **422** ; le code renvoie **400** (`InvoiceNotValidated`, même partage Epic 20).
- Le code d'erreur du contenu vide est `INVOICE_EMAIL_EMPTY_CONTENT` (variante Epic 20), pas `REMINDER_CONTENT_EMPTY` (qui reste, lui, un code per-facture du lot). Les tests attestent le comportement réel.

**Trend** : Pass 1 (3H/4M/1L) → patches appliqués. Findings > LOW subsistants : aucun. Relance Pass 2 (Haiku ou Opus, contexte frais) requise par la Review Iteration Rule.

### Dev-story (2026-07-16, Opus 4.8) — T1-T6 implémentées, gate backend complet vert

Implémentation des 6 tâches en 3 commits (`40edf89d` T1, `32a4233e` T2, `bc32f6e1` T3-T6), conforme à la spec scellée en Pass 3 validate. 8 fichiers hors artefacts BMAD, +1079/-2.

**Gate backend complet** (exécuté 2026-07-16 via `scripts/test-fast.sh --no-lint`, MariaDB dev up) :

| Check | Résultat |
|---|---|
| `cargo fmt --all -- --check` | ✅ exit 0 |
| `cargo check --workspace --all-targets` | ✅ exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ exit 0, 0 warning |
| `cargo nextest run` | ✅ **1845 passés / 1845**, 0 échec, 4 skipped, 2817 s |

Les 4 tests skipped sont des `#[ignore]` préexistants et sans lien avec 21-5b (pdfium absent de l'hôte, perf export, placeholder multi-devises Story 11, perf PDF).

**8 tests ajoutés, tous verts au gate** : 3 unitaires (`reminder_vars_ajoute_les_4_variables_rappel`, `days_overdue_clampe_et_gere_none`, `remaining_slots_decrements_and_zeroes_when_blocked`), 1 repo (`sum_fees_deduped_excludes_level_and_cancelled` — dédup H2), 4 E2E (`reminder_preview_renders_level`, `send_reminder_happy_path`, `send_reminder_paused_rejected_before_smtp`, `send_reminder_batch_partial_success`). AC 16 (cap dur 20 → 422 `BATCH_TOO_LARGE`) est couvert **dans** `send_reminder_batch_partial_success` (`invoice_send_email_e2e.rs:1371-1385`), pas dans une fonction séparée.

**Incident de session** : la session de dev a crashé après `bc32f6e1` et avant la clôture. Le gate annoncé par T6 (« Gate complet (ci-dessous) ») n'avait en réalité **jamais été tracé** — il a été (ré)exécuté intégralement lors de la reconstitution plutôt que présumé passé. Détail de la vérification d'intégrité post-crash en § Debug Log References.

**Statut** : `ready-for-dev` → `review`. Prochaine étape : `bmad-code-review` (Pass 1, LLM ≠ Opus pour contourner le biais d'auteur — cf. Review Iteration Rule).

### Validate Pass 1 (2026-07-16, Sonnet 4.6) — 1 CRITICAL + 2 HIGH + 3 MEDIUM + 2 LOW, patchés

Passe adversariale grep ground-truth (toutes les refs `fichier:ligne` confirmées exactes). Remédiés :
- **C1 (CRITICAL)** — re-check métier **post-SMTP** contredisait Epic 20 (`mark_emailed` `invoices.rs:1712` ne re-vérifie jamais après l'envoi, pour ne pas perdre la trace) → e-mail parti à un débiteur payé/suspendu **sans trace**. **Patch** : gardes + re-check niveau **AVANT le SMTP** sous verrou bref (relâché avant l'envoi), SMTP, puis enregistrement **best-effort** (jamais de rejet post-SMTP ; facture disparue → `ReminderSentButInvoiceGone` 409 + trace). AC 7-8 + AC 12 restructurés + test race.
- **H1** — `LEVEL_ALREADY_SENT` post-lock inatteignable (D18 autorise ≤ prochain → re-check passe toujours). **Patch** : le re-check pré-SMTP protège le **saut** de niveau ; la course double-clic même-niveau est un **risque résiduel documenté**, mitigé UI 21-6.
- **H2** — `totalDue` sommait les frais sans dédoublonner par niveau → ré-émission double-compte. **Patch** : `SUM(MAX(fee) GROUP BY level_number)` + test.
- **M1** — preview `level` requis (400 si absent). **M2** — code `REMINDER_CONTENT_EMPTY` ajouté aux `FailedProposal` du lot. **M3** — clarifié variantes `AppError` (DunningPaused/LevelAlreadySent/ReminderSentButInvoiceGone/BatchTooLarge) vs strings `FailedProposal` (NO_NEXT_LEVEL etc., pas de code mort).
- **L1** — ref `mail/mod.rs:53→54`. **L2** — dédup des `invoiceIds` du lot.

**Trend** : Pass 1 → 1C/2H/3M (> LOW) → patchés. Split NON recommandé. Relance Pass 2 (Haiku, contexte frais).

### Validate Pass 2 (2026-07-16, Haiku 4.5, contexte frais) — 0 CRITICAL/HIGH/MEDIUM, 2 LOW, patchés

Passe adversariale avec discipline grep ground-truth. **Les 7 patches Pass 1 (C1/H1/H2/M1-M3/L1-L2) confirmés corrects et cohérents** (grep-vérifiés AC par AC : verrou pré-SMTP → SMTP → best-effort, dédup `totalDue`, variantes AppError vs strings FailedProposal). Cohérence interne AC 5↔7↔8↔12 validée, complétude items epic 14-19/21/22 à 100%. 2 LOW cosmétiques remédiés :
- **L1-p2** — formulation « cap 21 » clarifiée en « cap dur 20 : lot de 21 → 422 ».
- **L2-p2** — AC 7 précise que les gardes globales (auth/rate-limit/SMTP) s'appliquent avant le verrou pré-SMTP.

**Trend** : Pass 1 (1C/2H/3M) → Pass 2 (0 > LOW). Relance Pass 3 (Opus, contexte frais) — passe de scellage justifiée par la restructuration architecturale du CRITICAL C1.

### Validate Pass 3 (2026-07-16, Opus 4.8, contexte frais) — CONVERGÉ (0 > LOW)

Passe de scellage. **Les 7 patches Pass 1/2 confirmés corrects par grep** (C1 flux verrou-pré-SMTP→SMTP→best-effort fidèle au précédent Epic 20 `invoice_email.rs:317-380` ; helpers 21-5a présents `find_scoped_for_update_in_tx:482`/`current_level_in_tx:67`/`insert_in_tx:104`/`ReminderChannel::Email` ; rate-limiter `remaining_slots` exposable `max_attempts:43`/`recent_count:134`). Vérification profonde C1 : le seul résidu « e-mail parti sans trace » est le crash-process entre SMTP-OK et commit — **exposition dual-write héritée d'Epic 20, non introduite par 21-5b, hors scope (outbox)**. Opus note que le fix H1 est **plus correct que l'epic item 19** (qui listait à tort `LEVEL_ALREADY_SENT` parmi les erreurs batch). 2 LOW cosmétiques : L3 (échec DB générique post-SMTP → 500, absorbé AC 7c) + L4 (redondance gardes AC 5/AC 7a, intentionnelle TOCTOU).

**Trend final** : P1 (1C/2H/3M) → P2 (0 > LOW) → **P3 (0 > LOW) CONVERGÉ**. LLM : Sonnet→Haiku→Opus (rotation complète). Split non déclenché (3 passes < 4, ~4 modules). **Spec scellée, prête pour `bmad-dev-story`.**
