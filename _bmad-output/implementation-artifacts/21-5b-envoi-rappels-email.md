# Story 21.5b: Envoi de rappels par e-mail (backend)

Status: ready-for-dev

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
2. **`totalDue`** = `invoice_total_ttc(lines)` **+ Σ `fee_amount` des rappels non-annulés déjà envoyés** (`invoice_reminders::list_for_invoice` filtré `cancelled_at.is_none()`, OU une somme SQL dédiée `SUM(fee_amount) WHERE cancelled_at IS NULL`) **+ `fee_amount` du niveau en cours d'envoi** (`dunning_levels::find_by_level_number(level)`). **Calculé AVANT l'insertion du rappel courant** (sinon double comptage — le body est snapshoté dans `NewInvoiceReminder`). `reminderFee` = frais du niveau en cours.
3. **`daysOverdue`** = `(today_utc - due_date).num_days().max(0)` (clampé ≥ 0), `0` si `due_date` NULL. `today_utc = chrono::Utc::now().naive_utc().date()` (cohérent `is_invoice_overdue`/UTC_DATE).

### Preview

4. **`GET /api/v1/invoices/{id}/reminder-preview?level=N`** — **Comptable+** : rend **côté serveur** subject+body d'un rappel de niveau `N` pour la facture, via `email_templates::get_effective(company_id, InvoiceReminder, language, N)` + `render(template.subject/body, build_reminder_vars(...))`. DTO `ReminderPreviewResponse { to: Option<String>, language: Language, level: i16, subject: String, body: String }` (calqué `EmailPreviewResponse` + `level`). `to` = `locked_recipient(contact)` (NULL si contact sans e-mail). Scopé company (404 cross-tenant). **Ne consomme pas de slot rate-limit** (lecture). Niveau validé (`N >= 1`, existe en config → sinon 422 `DUNNING_LEVEL_NOT_FOUND`).

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
7. **Anti-double-envoi sous verrou (item 17)** : l'insertion du rappel se fait dans une **transaction** avec `SELECT … FOR UPDATE` sur la row `invoices` (`invoices::find_scoped_for_update_in_tx`, réutilisé de 21-5a) — **re-calcul du prochain niveau sous verrou** (`invoice_reminders::current_level_in_tx` + config) et **re-check** : si le `levelNumber` visé n'est plus cohérent (un envoi concurrent a avancé le cycle → le niveau visé dépasse désormais le prochain) → **409 `LEVEL_ALREADY_SENT`**. Pas de `UNIQUE(invoice_id, level_number)` (le ré-envoi volontaire est un besoin D18) — la protection est le re-check sous verrou. Re-vérifier aussi `status`/`paid_at`/`dunning_paused_at` sous le verrou (TOCTOU, leçon 21-5a).
8. **Ordre « SMTP d'abord ⇒ enregistrer » (item 16)** : `state.mailer.send_email(&email).await?` d'abord (échec → 500 `SMTP_SEND_FAILED`, **rien enregistré**) ; **puis** dans la tx : `invoice_reminders::insert_in_tx(channel=ReminderChannel::Email, sent_to=Some(to), level_number, fee_amount=level.fee_amount, sent_at=NOW, subject, body, actor_user_id)` + audit `invoice.reminder_sent` (`"channel":"email"`, `reminderId`, `levelNumber`) dans la MÊME tx. **Note** : contrairement à l'unitaire facture (qui envoie le body du payload sans re-render), le rappel envoie le subject+body **du payload** (édités depuis la preview) — cohérent Epic 20. Response 200 = `ReminderResponse` (le rappel créé, DTO 21-5a).
9. **Log INFO envoi réussi (item 21)** : `tracing::info!` après commit (facture ET rappel — appliquer aussi à l'envoi facture Epic 20 qui ne loggue rien aujourd'hui) : `invoice_id`, `level_number`, `channel`, destinataire (masqué/partiel si politique). Un envoi réussi doit laisser une trace dans le log fichier.

### Envoi par lot

10. **`POST /api/v1/dunning/reminders/send-batch`** — **Comptable+** : envoie le **prochain niveau** à une liste de factures. **Request DTO** : `{ invoiceIds: Vec<i64> }` (**PAS de `to`**, PAS de subject/body — le lot rend chaque template serveur, item 16). Réponse **HTTP 200** `{ accepted: Vec<AcceptedReminder>, failed: Vec<FailedProposal> }` (pattern batch CLAUDE.md).
11. **Cap dur + pré-check rate-limit (item 19)** — **AVANT tout SMTP** :
    - `invoiceIds.len() > 20` → **422** global (`RESULT_TOO_LARGE` ou `BATCH_TOO_LARGE`) — borne la durée HTTP (N envois SMTP séquentiels).
    - **pré-check de capacité** : si `invoiceIds.len() > slots restants` sur le limiteur send-email → **429 global** AVANT le 1er SMTP (pas de blocage mi-course qui gèlerait l'utilisateur 15 min). Nécessite une **nouvelle méthode `RateLimiter::remaining_slots(key) -> u32`** (`middleware/rate_limit.rs` : `max_attempts - recent_count`, actuellement privés — à exposer). Le lot **consomme 1 slot par e-mail effectivement envoyé** (pas de contournement du 20/15 min).
12. **Traitement per-facture (item 19)** : pour chaque `invoice_id` (dans l'ordre), en **transaction séparée post-SMTP** :
    - gardes → `FailedProposal { invoice_id, error_code, details? }` (JAMAIS d'`AppError` global — succès partiel = HTTP 200) : `INVOICE_NOT_FOUND` (absente **ou cross-tenant** — même code, pas de fuite d'existence), `INVOICE_ALREADY_PAID`, `DUNNING_PAUSED`, `CONTACT_EMAIL_MISSING`, `INVOICE_NOT_PDF_READY`/`INVOICE_NOT_VALIDATED`, `LEVEL_ALREADY_SENT` (re-check niveau sous verrou), `NO_NEXT_LEVEL` (facture au dernier niveau / dunning désactivé), `RATE_LIMITED` (si un slot manque en cours de lot malgré le pré-check), `SMTP_SEND_FAILED`.
    - **niveau = prochain uniquement** (le lot n'autorise pas le ré-envoi ni le saut — item 18). Le template du prochain niveau est rendu serveur (`get_effective` + `build_reminder_vars`).
    - succès → `AcceptedReminder { invoice_id, reminder_id, level_number }` + audit `invoice.reminder_sent` + log INFO.
    - **Garde-fou variant sum-type** (CLAUDE.md pattern batch) : pas d'`unreachable!()` ; un refactor incomplet → `AppError::Internal` (500 global), une validation métier per-facture → `FailedProposal`.
13. **Exceptions `AppError` globales autorisées** (CLAUDE.md) uniquement en amont du per-facture : 401 (auth), 403 (RBAC Comptable+), 400 (body invalide), 422 (cap 20), 429 (pré-check capacité), 500 (pool/panic). Toute erreur dépendante de la facture → `FailedProposal`.

### Tests

14. **Tests unitaires** `build_reminder_vars` : les 10 variables posées, `totalDue` = TTC + Σ frais non-annulés + frais niveau (pas de double comptage), `daysOverdue` clampé ≥ 0 + `due_date` NULL → 0, formatage suisse (`format_money`/`format_date`), `reminderLevel`/`reminderFee` corrects par niveau.
15. **Tests E2E envoi unitaire** (MockMailer, `KESH_TEST_MODE`) : preview rendue serveur (subject/body substitués, `to` verrouillé) ; envoi niveau prochain → e-mail capturé (`/_test/sent-emails`, PDF joint `facture-*.pdf`, destinataire = contacts.email) + `invoice_reminders` (channel='email', sent_to) + audit `invoice.reminder_sent` ; **gardes** : payée → 422 `INVOICE_ALREADY_PAID`, suspendue → 422 `DUNNING_PAUSED`, sans e-mail → 422 `CONTACT_EMAIL_MISSING`, SMTP non configuré → 412, contenu vide → 422 ; **niveau > prochain → refus** ; **ré-envoi niveau ≤ prochain autorisé** ; **SMTP down → 500 + rien enregistré** (MockMailer::failing) ; anti-IDOR 404 ; RBAC Consultation → 403.
16. **Tests E2E envoi lot** : lot de N factures dues → `{ accepted, failed }` 200 ; succès partiel (1 payée + 1 sans e-mail + 1 OK → 1 accepted, 2 failed avec les bons `error_code`) ; **cap 21 → 422** ; **pré-check capacité** (lot > slots restants → 429 global, aucun e-mail parti) ; niveau = prochain uniquement ; cross-tenant id → `INVOICE_NOT_FOUND` (pas de fuite) ; RBAC Consultation → 403.
17. **Test rate-limiter** `remaining_slots` : décrémente à chaque `check_and_record`, `max_attempts` au départ, 0 quand bloqué.

### Doc & gate

18. **Doc** : CHANGELOG `Ajouté` (envoi des rappels par e-mail — preview/unitaire/lot). Manuels = **21-8**. Pas de nouvelle migration (aucune table/colonne) → pas de ligne idempotence, pas de bump `min_required`.
19. **Gate local complet** (Test Locally First) : `cargo fmt`/`clippy`/`build`/`test --workspace --test-threads=1`. **E2E backend Rust** (MockMailer) suffisants ici (pas de frontend → 21-6). **Rappel gate** : la DB dev doit être migrée (`sqlx migrate run`) ; le gate workspace complet est le filet ultime (leçon 21-5a — un SELECT `Invoice` cross-crate).

## Tasks / Subtasks

- [ ] **T1 — `build_reminder_vars` + calcul `totalDue`/`daysOverdue`** (AC 1,2,3,14)
  - [ ] Builder calqué `build_invoice_vars` + 4 vars rappel.
  - [ ] Somme des frais non-annulés (`list_for_invoice` filtré, ou `SUM` SQL) + frais niveau courant, calculée avant insertion.
  - [ ] `daysOverdue` clampé UTC. Tests unitaires.
- [ ] **T2 — Rate-limiter `remaining_slots`** (AC 11,17)
  - [ ] Méthode `RateLimiter::remaining_slots(key) -> u32` (`middleware/rate_limit.rs`) + test.
- [ ] **T3 — Preview rappel** (AC 4)
  - [ ] `GET .../reminder-preview?level=N` + DTO `ReminderPreviewResponse` + validation niveau.
- [ ] **T4 — Envoi unitaire** (AC 5,6,7,8,9)
  - [ ] `POST .../reminders/send` : 8 gardes + éligibilité + choix niveau ≤ prochain + FOR UPDATE re-check (LEVEL_ALREADY_SENT) + SMTP-puis-enregistrer (insert_in_tx channel=Email + audit) + log INFO.
  - [ ] Log INFO ajouté aussi à l'envoi facture Epic 20 (item 21).
  - [ ] Tests E2E unitaire (AC 15).
- [ ] **T5 — Envoi lot** (AC 10,11,12,13)
  - [ ] `POST /api/v1/dunning/reminders/send-batch` : cap 20 (422) + pré-check capacité (429) + per-facture tx post-SMTP + FailedProposal + niveau prochain + audit + log.
  - [ ] Tests E2E lot (AC 16).
- [ ] **T6 — Montage RBAC + doc + gate** (AC 18,19)
  - [ ] 3 routes dans `comptable_routes` (avant le `route_layer` — anti-footgun). CHANGELOG. Gate complet vert.

## Dev Notes

### Ground-truth (2026-07-16, 2 agents Explore)

**Pipeline envoi Epic 20** (`routes/invoice_email.rs`) : `send_invoice_email:244` — gardes ordonnées auth/tenant(`get_company_for:250`) → **rate-limit 429** (`rate_limiter_send_email.check_and_record((company.id,user_id)):254`) → **SMTP 412** (`smtp_ready:271`) → facture 404 (`find_by_id_with_lines:279`) → contact (`load_active_contact:282`) → **`locked_recipient:73`** (contacts.email trim non-vide ; **pas de champ `to` au payload**, `SendInvoiceEmailRequest`=subject+body `:54`) → contenu vide 422 (`:287`) → PDF (`invoice_pdf_service::render:296`). **Inversion** : `state.mailer.send_email(&email).await?:317` PUIS enregistrement `:323`. Cas facture disparue #219 → trace best-effort + 409. **Preview** `preview_invoice_email:201` : rend serveur (`get_effective` + `build_invoice_vars` + `render:225`), DTO `EmailPreviewResponse { to: Option<String>, language, subject, body }:40`.

**Vars** : `build_invoice_vars:159` (privé, 6 vars, `amount`=TTC via `invoice_total_ttc:185`, `format_money`/`format_date` kesh-i18n), `salutation_line:107` (civilité×langue×type #12), `resolve_language:67`. `build_reminder_vars` (nouveau) réutilise ça + ajoute reminderLevel/Fee/totalDue/daysOverdue.

**Moteur** (`kesh-core/email_template_engine.rs`) : `render(template:&str, vars:&HashMap<String,String>) -> String:56` single-pass, jamais de re-scan des valeurs substituées, token inconnu laissé littéral. `validate_tokens:34` (au save seulement, PAS au send).

**Templates rappel** : `email_templates::get_effective(pool, company_id, EmailTemplateType::InvoiceReminder, language, N):106` — cascade override(type,langue,N)→override(type,langue,0)→`reminder_default(langue,N):71`→générique. `EffectiveEmailTemplate.level_number` = slot demandé. `allowed_variables()` InvoiceReminder = 10 vars **déjà déclarées 21-3** (`email_template.rs:63`). Défauts : niveaux 1/2/3 + générique ; `{reminderFee}` aux niveaux 2/3 ; `{reminderLevel}` **déclaré mais non utilisé** par les défauts (l'alimenter quand même).

**TTC** : `kesh_core::accounting::vat::invoice_total_ttc(lines: (Decimal line_total, Decimal vat_rate)) -> Decimal` (vat.rs:62, TVA arrondie par ligne). `{totalDue}` = TTC + Σ frais non-annulés + frais niveau (pas de helper somme existant — composer). `{amount}`=TTC facture.

**Rate-limiter** : `build_send_email_rate_limiter:lib.rs:145` (20/15min, clé `(company_id,user_id)`), `AppState.rate_limiter_send_email: Arc<RateLimiter<(i64,i64)>>:62`. `check_and_record(key):201` consomme (rejette sans consommer). `check_rate_limit(key):179` pré-check **binaire** (bloqué ou non), **PAS de compte de slots restants** → **ajouter `remaining_slots(key) -> u32`** (`max_attempts:privé - recent_count:134` — trivial à exposer).

**Mailer** : `state.mailer.send_email(&OutgoingEmail):mail/mod.rs:91`. `OutgoingEmail { to, subject, body, from_display_name, reply_to, attachment: Option<EmailAttachment{filename, content_type, bytes}> }:53`. MockMailer en `KESH_TEST_MODE` + `GET /api/v1/_test/sent-emails` (8 champs camelCase). `NoopMailer` renvoie Ok silencieux → d'où la garde 412 obligatoire. `smtp_ready:lib.rs:75` → 412 `SMTP_NOT_CONFIGURED` (`errors.rs:265`).

**PDF** : `invoice_pdf_service::render(pool, i18n, locale, company, invoice_id) -> RenderedInvoicePdf { bytes, filename_base }:68`. attachment = `facture-{filename_base}.pdf`. Contraintes : validated, ≤9 lignes (`MAX_LINES_PER_PDF`), contact, compte bancaire primary.

**Enregistrement** : PAS `mark_emailed` mais `invoice_reminders::insert_in_tx(&mut tx, &NewInvoiceReminder{ channel: ReminderChannel::Email, sent_to: Some(to), … }):repositories/invoice_reminders.rs:104` + audit `invoice.reminder_sent`. `ReminderChannel::Email` existe (`entities/invoice_reminder.rs:19`). Patron déjà en place pour le manuel (`routes/dunning_reminders.rs:299`, channel=Manual). `find_scoped_for_update_in_tx` + `current_level_in_tx` (21-5a) réutilisés.

### Patterns à réutiliser (ne PAS réinventer)

- **Envoi + gardes** : `send_invoice_email` (ordre 8 gardes + inversion SMTP). Ne pas changer l'ordre.
- **Batch `{accepted,failed}`** : `FailedProposal { business_id, error_code: String, details: Option<Value> }` (business id = `invoice_id`), HTTP 200 succès partiel, cross-tenant → même code que not-found (CLAUDE.md pattern batch + `reconciliation.rs::accept_batch`). Pas d'`unreachable!()` aux variants.
- **Éligibilité / niveau** : `dunning_eligibility` (liste), `invoice_reminders::current_level_in_tx`, `dunning_levels::find_by_level_number`/`list_all_by_company` (prochain niveau) — 21-5a.
- **Erreurs → HTTP** : variantes AppError 21-5a (`INVOICE_ALREADY_PAID`/`DUNNING_PAUSED`? — **`DUNNING_PAUSED` n'existe pas encore en AppError**, à ajouter ; `INVOICE_ALREADY_PAID` existe ; `DUNNING_LEVEL_NOT_FOUND` existe ; `LEVEL_ALREADY_SENT`/`NO_NEXT_LEVEL`/`BATCH_TOO_LARGE` à ajouter). Cross-tenant → 404 `NOT_FOUND`.
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

- **Nouveaux** : `build_reminder_vars` + handlers `reminder_preview`/`send_reminder`/`send_reminder_batch` (dans `routes/invoice_email.rs` ou `routes/dunning_reminders.rs` — à trancher validate ; cohérence : les data-endpoints rappels sont dans `dunning_reminders.rs`, le socle envoi dans `invoice_email.rs`). DTOs request/response. `RateLimiter::remaining_slots`. Variantes AppError (`DunningPaused`, `LevelAlreadySent`, `NoNextLevel`, `BatchTooLarge`). Tests e2e.
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

### Debug Log References

### Completion Notes List

### File List

## Change Log
