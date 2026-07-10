# Story 20.3b1: Envoi de facture par e-mail — backend

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

En tant que comptable d'une PME,
je veux envoyer une facture validée par e-mail à mon client (PDF QR-facture joint, message personnalisé dans sa langue),
afin de ne plus exporter le PDF puis passer par mon client mail à la main.

**Périmètre de cette sous-story : backend uniquement** (migrations, entités, mailer, config, endpoints, tests E2E backend). Le frontend (modale, bouton, fiche contact, flag UI) est la sous-story **20-3b2** ; les manuels + Playwright sont en **20-4**.

### Note de découpage (règle de splitting préventif CLAUDE.md)

La 20-3b du planning epic-20 touche kesh-db + kesh-api (mail, config, routes invoices/contacts/companies/health, middleware) + frontend (invoices, contacts, shared/flags) + kesh-i18n — au-delà du seuil de 5 modules. Split appliqué **avant** spec : **20-3b1 (backend, cette story) → 20-3b2 (frontend)**, calqué sur le découpage 20-1/20-2 (convergence 1 passe chacun). La 20-3b1 est autonome et testable en isolation (E2E backend via `MockMailer`).

## Acceptance Criteria

**Migrations & entités**

1. Migration `contacts` (décisions #11/#12 epic) : `ADD COLUMN language CHAR(2) NULL` + CHECK `(language IS NULL OR BINARY language IN (BINARY 'FR', BINARY 'DE', BINARY 'IT', BINARY 'EN'))` (calqué `chk_companies_instance_language`, `20260404000001_initial_schema.sql:18`) ; `ADD COLUMN salutation VARCHAR(10) NOT NULL DEFAULT 'Neutre'` + CHECK `salutation IN ('Monsieur','Madame','Neutre')`. **Sémantique `language NULL` = hérite de `companies.instance_language`** (résolution à l'envoi, pas de copie à la création). Non-breaking (ADD COLUMN) → pas de bump `kesh_version_min_required` ; entrée `docs/migrations-idempotence-audit.md` (`tracked-by-sqlx`).
2. Migration `invoices` (décision #16) : `ADD COLUMN emailed_at DATETIME(6) NULL` + `ADD COLUMN emailed_to VARCHAR(320) NULL` (pattern `paid_at`). Non-breaking ; entrée idempotence audit.
3. Migration `companies` : `ADD COLUMN email VARCHAR(320) NULL`. **Refinement assumé du planning** : la décision #2 epic exige `Reply-To` = e-mail de la société, or **aucun champ e-mail n'existe** sur `companies` ni `company_invoice_settings` (vérifié). Colonne nullable → `Reply-To` omis tant que non renseignée (saisie UI en 20-3b2). Non-breaking ; entrée idempotence audit.
4. Compteur `crates/kesh-db/tests/migrations_upgrade_path.rs:71-75` : **47 → 50** (+ commentaire chaîné lignes 60-70). `TABLES_TO_TRUNCATE` (backup.rs) et compteurs d'export **inchangés** (ils comptent des tables, pas des colonnes — vérifié).
5. Entités et repositories étendus :
   - `Contact` : `language: Option<Language>` (réutilise `entities::Language`, company.rs:73) + `salutation: Salutation` — **nouvel enum `Salutation { Monsieur, Madame, Neutre }`** dans `contact.rs`, pattern exact `ContactType` (as_str PascalCase, FromStr, sqlx Type/Encode/Decode via String, serde).
   - `NewContact` + `ContactUpdate` étendus ; repository `contacts.rs` : constantes `COLUMNS`/`FIND_BY_ID_SQL` (:26-34), INSERT (:195-218) + binds, UPDATE + binds, **`is_no_op_change` (:371-387)** et **`contact_snapshot_json` (:37-53)** étendus (sinon KF-004 et l'audit ignorent ces champs).
   - `Invoice` : `emailed_at: Option<NaiveDateTime>` + `emailed_to: Option<String>` ; constantes SQL (`FIND_INVOICE_SCOPED_SQL` et toute liste de colonnes invoices) étendues.
   - `Company` : `email: Option<String>` ; repository companies (colonnes/UPDATE) + DTO route update company (camelCase, champ optionnel `#[serde(default)]`).
   - Routes contacts : `CreateContactRequest`/`UpdateContactRequest` (contacts.rs:64-117) + `ContactResponse` (:125-183) étendus — `language: Option<Language>`, `salutation: Option<Salutation>` (`#[serde(default)]`, absent = inchangé/défaut Neutre à la création).
6. Repository `invoices::mark_emailed(pool, company_id, invoice_id, to: &str, subject: &str, user_id, actor_api_key_id: Option<i64>) -> Result<Invoice, DbError>` : `UPDATE invoices SET emailed_at = NOW(6), emailed_to = ? WHERE id = ? AND company_id = ?` + audit **`invoice.emailed`** (entity_type `"invoice"`, details `{ "to": …, "subject": … }`, constructeur `NewAuditLogEntry::for_actor` — leçon 20-1 AC#11) **dans la même transaction**. **Pas de verrou optimiste** (métadonnée d'envoi, pas d'état comptable ; renvoi autorisé et ré-audité, chaque envoi écrase `emailed_at`/`emailed_to`).

**Mailer (décisions #1/#2 epic)**

7. `crates/kesh-api/src/mail/mod.rs` : nouvelles structs
   ```rust
   pub struct EmailAttachment { pub filename: String, pub content_type: String, pub bytes: Vec<u8> }
   pub struct OutgoingEmail {
       pub to: String,
       pub subject: String,
       pub body: String,                        // text/plain (décision #9 epic — HTML hors scope)
       pub from_display_name: Option<String>,   // nom de la société
       pub reply_to: Option<String>,            // companies.email si renseigné
       pub attachment: Option<EmailAttachment>,
   }
   ```
   + méthode de trait `fn send_email<'a>(&'a self, email: &'a OutgoingEmail) -> MailFuture<'a>` (pattern objet-safe `MailFuture` existant, mod.rs:33-54). Implémentations : `SmtpMailer` (voir AC#8), `NoopMailer` (Ok(())), `MockMailer` (nouveau buffer `sent_emails() -> Vec<CapturedEmail>` capturant to/subject/body/from_display_name/reply_to/attachment filename+taille ; `failing()` s'applique aussi). **`CapturedMail` et `send_password_reset` strictement inchangés** (33 call-sites `new_for_tests` + recovery e2e intacts).
8. `SmtpMailer::send_email` (smtp.rs) : message via builder lettre (anti-injection CRLF typé, à préserver) —
   - `From` = `Mailbox::new(Some(display_name), self.from.email.clone())` quand `from_display_name` est fourni, sinon `self.from` tel quel (l'adresse reste toujours `KESH_SMTP_FROM` ; **seul le display-name est dynamique** — décision #2 + garde L20-1 : jamais de `From` fourni par l'appelant).
   - `Reply-To` : `reply_to.parse::<Mailbox>()` ; si parse échoue → `tracing::warn!` + **omission** (pas d'échec d'envoi pour un Reply-To invalide).
   - Corps : `MultiPart::mixed()` = `SinglePart` text/plain (corps) + `Attachment::new(filename).body(bytes, content_type)` quand attachment présent ; sinon single part text/plain. `ContentType::parse(...)` → `map_err` → `AppError::SmtpSendFailed` (pas d'`unwrap`/`expect`).
   - Factoriser un `build_outgoing_message(&self, email: &OutgoingEmail) -> Result<Message, AppError>` testable unitairement (pattern `build_message` existant smtp.rs:96-122).
   - **Aucun changement Cargo.toml** : feature lettre `builder` déjà active (Cargo.toml:66-70, vérifié — `MultiPart`/`Attachment` disponibles).

**Config, gate SMTP, health (décision #3 epic)**

9. `Config::smtp_configured(&self) -> bool` = `smtp_host`/`smtp_user`/`smtp_password`/`smtp_from` tous `Some` **et** `is_valid_email_simple(smtp_from)`. (`public_base_url` reste exigé uniquement par le gate recovery.) Le fail-fast `KESH_FEATURE_FORGOT_PASSWORD` (config.rs:1078-1127) est **inchangé**.
10. `main.rs` (:233-272) : construire `SmtpMailer` quand `config.smtp_configured()` (plus seulement quand `forgot_password_enabled`). Échec de build : si `forgot_password_enabled` → `exit(1)` (comportement actuel conservé) ; sinon → `tracing::error!` + fallback `NoopMailer` (dégradation gracieuse, pas de fail-fast pour l'envoi facture).
11. `GET /health` (health.rs:24-53) expose `smtpConfigured: state.config.smtp_configured()` dans **les deux branches** 200/503 (à côté de `forgotPasswordEnabled` — consommé par le frontend en 20-3b2).

**Rate-limit (décision #15 epic)**

12. Généraliser `RateLimiter` (middleware/rate_limit.rs:35) en `RateLimiter<K: Eq + Hash + Clone = IpAddr>` — **paramètre de type avec défaut → zéro changement des call-sites existants** (login + recovery restent `RateLimiter` nu). Nouvelle instance `AppState.rate_limiter_send_email: Arc<RateLimiter<(i64, i64)>>` (clé = `(company_id, user_id)`), fabrique `build_send_email_rate_limiter()` calquée `build_recovery_rate_limiter` (lib.rs:103-109) : **20 envois / fenêtre 15 min / blocage 15 min** (hardcodé, même limitation assumée que le recovery « L5 v0.2+ »). Défaut ajouté dans le **corps** de `AppState::new_for_tests` (signature figée, lib.rs:76-96). Dépassement → 429 (même mapping que le recovery).

**Endpoints (décisions #13/#15 epic)**

13. `GET /api/v1/invoices/{id}/email-preview` (nouveau module `crates/kesh-api/src/routes/invoice_email.rs`, enregistré dans `comptable_routes` à côté du bloc mark-paid, lib.rs:334-345) → 200 :
    ```json
    { "to": "client@example.ch" | null, "language": "FR", "subject": "…", "body": "…" }
    ```
    Facture scopée company (`find_by_id_with_lines` → 404 sinon) ; `to` = `contact.email` (null si absent — le frontend désactivera l'envoi) ; `language` = `contact.language` sinon `company.instance_language` ; subject/body = template `email_templates::get_effective(pool, company.id, InvoiceSend, language)` (jamais d'erreur — vérifié) rendu via `kesh_core::email_template_engine::render` avec les variables AC#14. Pas d'exigence de statut `validated` pour la preview (le POST la porte via le rendu PDF).
14. Variables de substitution (les 6 déclarées par `EmailTemplateType::InvoiceSend.allowed_variables()`, email_template.rs:42-53) — valeurs **pré-formatées suisses** :
    - `salutation` : matrice genre × langue × type de contact (cf. Dev Notes §Matrice) via helper pur `salutation_line(salutation, contact_type, last_name: Option<&str>, language) -> String` dans `invoice_email.rs`, testé unitairement.
    - `contactName` = `contact.name` ; `companyName` = `company.name`.
    - `invoiceNumber` = `invoice.invoice_number` sinon `#{id}` (même fallback que le PDF).
    - `amount` = `kesh_i18n::format_money(&invoice.total_amount)` (apostrophe U+2019).
    - `dueDate` = `kesh_i18n::format_date(&due_date)` sinon `"—"`.
15. `POST /api/v1/invoices/{id}/send-email` (Comptable+, même module) — body `{ "subject": "…", "body": "…" }` (camelCase, **pas de champ `to`** — décision #13 : destinataire verrouillé `contacts.email`). Séquence de gardes, dans cet ordre :
    1. `get_company_for` (auth/tenant).
    2. Rate-limit `check_and_record((company.id, current_user.user_id))` → 429.
    3. `!state.config.smtp_configured()` → **412 `SMTP_NOT_CONFIGURED`** (nouveau variant `AppError::SmtpNotConfigured` — garde impérative : sans elle, `NoopMailer` renverrait Ok et la facture serait marquée envoyée à tort).
    4. Facture scopée company → 404 (anti-IDOR).
    5. `contact.email` absent/vide → **400 `CONTACT_EMAIL_MISSING`** (nouveau variant).
    6. `subject`/`body` vides après trim → **422 `INVOICE_EMAIL_EMPTY_CONTENT`** (nouveau variant).
    7. Rendu PDF : `invoice_pdf_service::render(&state.pool, &state.i18n, locale_contact, &company, id)` — **signature 20-3a : `&Company`** ; `locale_contact` = `Locale::from(language.as_str())` (pattern invoices.rs:968) où `language` = contact sinon instance. Erreurs héritées inchangées (validated, ≤ 9 lignes, adresse, banque — L20-2).
    8. `mailer.send_email(OutgoingEmail { to, subject, body, from_display_name: Some(company.name), reply_to: company.email, attachment: Some(EmailAttachment { filename: format!("facture-{}.pdf", rendered.filename_base), content_type: "application/pdf", bytes }) })` — échec → **500 `SMTP_SEND_FAILED`** (mapping existant errors.rs:1043-1053), facture **non marquée**.
    9. Succès → `invoices::mark_emailed(...)` (AC#6) → **200 `InvoiceResponse`** à jour.
16. `InvoiceResponse` (invoices.rs:162-184) expose `emailed_at`/`emailed_to` (camelCase auto via serde) — la fiche facture 20-3b2 affichera « Envoyée le … à … ».
17. Nouveaux variants `AppError` + clés FTL **dans les 4 locales** (fr/de/it/en-CH) : `error-smtp-not-configured`, `error-contact-email-missing`, `error-invoice-email-empty-content` (codes JSON : `SMTP_NOT_CONFIGURED` 412, `CONTACT_EMAIL_MISSING` 400, `INVOICE_EMAIL_EMPTY_CONTENT` 422).

**Tests (E2E backend = preuve de bout en bout, MockMailer)**

18. Tests unitaires : matrice `salutation_line` (les 3 civilités × 4 langues × Personne/Entreprise, avec et sans `last_name`) ; construction des variables (fallbacks `invoiceNumber`/`dueDate`) ; `build_outgoing_message` (multipart : corps + attachment présents, display-name dans le From, Reply-To présent/omis-si-invalide) ; `smtp_configured()` (complet / partiel / from invalide).
19. Nouveau `crates/kesh-api/tests/invoice_send_email_e2e.rs` (squelette : `password_recovery_e2e.rs` pour l'injection MockMailer via AppState littéral + poignée `mock.sent_emails()`, `invoice_delete_e2e.rs` pour le seed facture validée) — cas minimum :
    - **Happy path** : 200 ; mock capture to = email contact, subject/body rendus (template défaut FR), attachment `facture-*.pdf` non vide ; `emailed_at`/`emailed_to` posés en DB ; audit `invoice.emailed` présent avec to+subject.
    - **Langue contact** : contact `language = 'DE'` → subject/body du template défaut DE + PDF rendu locale DE.
    - **Renvoi** : 2e POST → 200, `emailed_at` écrasé, 2 entrées d'audit.
    - **Contact sans email** → 400 `CONTACT_EMAIL_MISSING` ; **facture draft** → 400 `INVOICE_NOT_VALIDATED` ; **IDOR** (facture d'une autre company) → 404 ; **Consultation** → 403 ; **subject vide** → 422.
    - **SMTP down** (`MockMailer::failing()`) → 500 `SMTP_SEND_FAILED` **et** `emailed_at` reste NULL (non marquée).
    - **SMTP non configuré** (config sans vars SMTP) → 412 `SMTP_NOT_CONFIGURED`.
    - **Rate-limit** : 21e envoi dans la fenêtre → 429 (seuils injectables comme `recovery_max_attempts` dans password_recovery_e2e).
    - **Preview** : GET → 200, `to` null si contact sans email, subject/body rendus.
20. Non-régression : `password_recovery_e2e` **inchangé** (CapturedMail/recovery intacts), `invoice_pdf_e2e` inchangé, suite workspace verte (gate Test Locally First, kesh-db + kesh-api en série).

## Tasks / Subtasks

- [x] **T1 — Migrations + entités + repositories** (AC: #1-#6)
  - [x] T1.1 3 migrations (contacts / invoices / companies), en-têtes « non-breaking, pas de bump » + 3 entrées `docs/migrations-idempotence-audit.md`
  - [x] T1.2 Compteur `migrations_upgrade_path.rs` 47 → 50
  - [x] T1.3 Enum `Salutation` + extension `Contact`/`NewContact`/`ContactUpdate` + repo contacts (COLUMNS, INSERT/UPDATE, binds, `is_no_op_change`, `contact_snapshot_json`)
  - [x] T1.4 Extension `Invoice` (+ constantes SQL) + `invoices::mark_emailed` (+ audit `invoice.emailed` in-tx)
  - [x] T1.5 Extension `Company.email` + repo + DTO route update company ; DTOs + `ContactResponse` routes contacts
- [x] **T2 — Mailer** (AC: #7, #8)
  - [x] T2.1 `OutgoingEmail`/`EmailAttachment` + méthode trait `send_email` + `NoopMailer`
  - [x] T2.2 `SmtpMailer::send_email` + `build_outgoing_message` (MultiPart, display-name, Reply-To tolérant)
  - [x] T2.3 `MockMailer.sent_emails()` (buffer dédié, `failing()` couvert, `CapturedMail` intact)
- [x] **T3 — Config / gate / health** (AC: #9-#11)
  - [x] T3.1 `Config::smtp_configured()` + tests unit
  - [x] T3.2 `main.rs` construction mailer découplée + dégradation gracieuse
  - [x] T3.3 `/health` → `smtpConfigured` (2 branches)
- [x] **T4 — Rate-limiter générique + instance send-email** (AC: #12)
- [x] **T5 — Module `invoice_email.rs`** (AC: #13-#17)
  - [x] T5.1 `salutation_line` + construction des variables
  - [x] T5.2 Handler preview + handler send (séquence de gardes AC#15) + enregistrement `comptable_routes`
  - [x] T5.3 3 variants `AppError` + clés FTL ×4 + `InvoiceResponse.emailed_*`
- [x] **T6 — Tests** (AC: #18-#20)
  - [x] T6.1 Unit (matrice, vars, message, config)
  - [x] T6.2 `invoice_send_email_e2e.rs` (liste AC#19)
- [x] **T7 — Test Locally First & commit** (AC: #20)
  - [x] T7.1 `cargo fmt --check` + `build --workspace --all-targets` + `clippy -D warnings` + `cargo test --workspace` (kesh-db et kesh-api **en série** `--test-threads=1`) — 92 suites, 1778 tests, 0 échec
  - [x] T7.2 Commit sur `story/20-1-envoi-factures-email`

## Dev Notes

### Matrice `{salutation}` (décision #12 epic, figée ici)

Personne + civilité renseignée, **avec** `last_name` → formule + nom ; **sans** `last_name` → formule seule. Entreprise ou civilité `Neutre` → formule neutre (jamais de nom).

| | FR | DE | IT | EN |
|---|---|---|---|---|
| Monsieur (avec nom) | Cher Monsieur {nom} | Sehr geehrter Herr {nom} | Egregio Signor {nom} | Dear Mr {nom} |
| Monsieur (sans nom) | Cher Monsieur | Sehr geehrter Herr | Egregio Signore | Dear Sir |
| Madame (avec nom) | Chère Madame {nom} | Sehr geehrte Frau {nom} | Gentile Signora {nom} | Dear Ms {nom} |
| Madame (sans nom) | Chère Madame | Sehr geehrte Frau | Gentile Signora | Dear Madam |
| Neutre / Entreprise | Madame, Monsieur | Sehr geehrte Damen und Herren | Gentili Signore e Signori | Dear Sir or Madam |

Nota IT : « Egregio Signor » (tronqué) devant un nom, « Egregio Signore » seul.

### Ground-truth vérifié (cartographie 3 agents Explore, 2026-07-09)

**Mailer** : trait objet-safe via `MailFuture` (mod.rs:33-54), une seule méthode aujourd'hui. `SmtpMailer` (smtp.rs:31-141) : `from: Mailbox` parsé au boot, `build_message` factorisé testable, text/plain only. `MockMailer` (mod.rs:84-136) : buffer `Arc<Mutex<Vec<CapturedMail>>>`, poignée conservée par le test après `Arc::new(mailer)`. lettre 0.11 features `tokio1-rustls-tls, smtp-transport, builder` (Cargo.toml:66-70) → MultiPart/Attachment OK sans modif.

**Config** : gate SMTP réel = bloc `if forgot_password_enabled` config.rs:1078-1127 (fail-fast `IncompleteSmtpConfig`) ; `is_valid_email_simple` (routes/contacts.rs:197) rejette les display-names dans `KESH_SMTP_FROM` — c'est pour ça que le display-name doit être injecté **à l'envoi** (Mailbox::new), pas dans la var d'env. `smtp_password` privé (accès `smtp_password()`, config.rs:374). Pas de helper `smtp_configured()` aujourd'hui.

**Rate-limiter** : `RateLimiter` keyé `IpAddr` strict (rate_limit.rs:35-40), `check_and_record` atomique (:195), fabrique recovery lib.rs:103-109 (5/15min/30min hardcodé). Généralisation par paramètre de type avec défaut = zéro diff sur les call-sites existants.

**Socle 20-1** : `get_effective(pool, company_id, type, language) -> EffectiveEmailTemplate` **jamais d'erreur introuvable** (email_templates.rs:90-110). Variables `InvoiceSend` = `["salutation","contactName","invoiceNumber","amount","dueDate","companyName"]` (email_template.rs:42-53). Moteur `kesh_core::email_template_engine::render(template, &HashMap<String,String>) -> String` **infaillible** (token inconnu laissé littéral, single-pass). Défauts 4 langues en constantes Rust (`email_template_defaults.rs`). Formatage : `kesh_i18n::{format_money, format_date}` (formatting.rs — apostrophe U+2019, dd.mm.yyyy).

**Langue** : `Language` (company.rs:73-84, FR/DE/IT/EN) réutilisé tel quel pour `contacts.language`. **Pas de conversion typée Language→Locale** : pattern existant `Locale::from(language.as_str())` (invoices.rs:968, parsing permissif fallback FrCh).

**Contacts repo — pièges** : colonnes en dur dans `COLUMNS`/`FIND_BY_ID_SQL` (:26-34) + INSERT (:195-218) + UPDATE, **et** `is_no_op_change` (:371-387) + `contact_snapshot_json` (:37-53) — les 2 derniers oubliés = KF-004 et audit aveugles aux nouveaux champs.

**Invoices** : `paid_at` = pattern exact pour `emailed_at` (entities/invoice.rs:17-40). `find_by_id_with_lines(pool, company_id, id)` (invoices.rs repo :432). Audit dans les repos mutants, même tx, `for_actor` (leçon 20-1 Pass 1 AC#11).

**Routes** : `comptable_routes` lib.rs:208-453, `.route_layer(require_comptable_role)` :451-453 ; zone d'insertion à côté de mark-paid (lib.rs:334-345). Handler modèle `mark_invoice_paid_handler` (invoices.rs:827-852). `require_auth` injecte `CurrentUser` avant le check de rôle (ordre oignon lib.rs:703-711).

**Health** : `GET /health` public (health.rs:24-53) expose déjà `forgotPasswordEnabled` dans les 2 branches 200/503 → y ajouter `smtpConfigured` (consommation frontend en 20-3b2 via `feature-flags.svelte.ts`).

**E2E MockMailer** : `password_recovery_e2e.rs` = modèle exact (AppState littéral :127-141, `mailer: Arc::new(mock.clone())`, assertions `mock.sent()`). `invoice_delete_e2e.rs` = modèle seed (`seed_accounting_company`, `create_validated_invoice` :143-172, `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`).

**Compteurs figés** (leçon 20-1, famille « nouvelle migration → compteur figé ailleurs ») : SEUL `migrations_upgrade_path.rs:71-75` (47) est impacté par des ADD COLUMN. `TABLES_TO_TRUNCATE` (backup.rs:34-64) et exports (`admin_full_export_e2e.rs`) comptent des **tables** — non impactés (vérifié : aucune assertion ne fige le nombre de colonnes de contacts/invoices).

### Décisions de conception (refinements assumés vs planning)

- **`companies.email` (nouvelle colonne)** : le planning suppose un e-mail de société pour le `Reply-To` — inexistant en base. Nullable, `Reply-To` omis si absent. UI de saisie en 20-3b2.
- **Endpoint preview séparé** (`GET email-preview`) : les routes templates 20-1 sont **Admin-only**, or l'expéditeur est Comptable+ — la modale ne peut pas rendre le template côté client. Le serveur rend et renvoie subject/body pré-remplis, l'utilisateur édite, le POST envoie le texte final tel quel (le serveur ne re-rend PAS le template au send : ce que l'utilisateur a vu/édité est ce qui part).
- **Garde 412 avant tout envoi** : sans elle, `NoopMailer` (SMTP non configuré) retournerait Ok → facture marquée envoyée à tort. C'est la garde runtime qui matérialise la « dégradation gracieuse » côté API.
- **Pas de verrou optimiste sur send** : l'envoi ne modifie pas l'état comptable ; une course entre 2 envois = 2 renvois légitimes, tous deux audités.
- **Seuils rate-limit 20/15min/15min** : hardcodés (parité limitation recovery L5). Configurables = v0.2+.
- **Trait `send_email` générique** (pas `send_invoice`) : consommateurs futurs = rappels (#231) et récurrentes (#223) — décision #5 epic (types futurs), le mailer n'a pas à connaître le métier facture.

### Frontières de scope

- **AUCUN fichier frontend** (20-3b2) ; **aucun manuel/Playwright** (20-4).
- Recovery (`send_password_reset`, `CapturedMail`, FTL recovery) **intouché** — décision epic « Recovery laissé en paix ».
- `kesh-qrbill` et `invoice_pdf_service` **inchangés** (le service 20-3a est consommé tel quel, signature `&Company`).
- Pas de BCC/archivage, pas d'envoi groupé, pas de bounces (différé — L20-3 : « envoyée » = remise SMTP, pas accusé de réception).

### Testing standards summary

- E2E backend = preuve bout-en-bout de la story (MockMailer) ; `kesh-db` et `kesh-api` **en série** (`--test-threads=1`, empirique 20-1).
- Gate Test Locally First : 4 checks backend (story 100 % Rust).
- Non-régression sentinelles : `password_recovery_e2e` + `invoice_pdf_e2e` verts inchangés.

### References

- [Source: `_bmad-output/planning-artifacts/epic-20-envoi-factures-email.md`] — décisions #1-#3 (SMTP/From/gate), #11-#12 (langue/civilité), #13-#16 (envoi), limitations L20-1/2/3, contexte technique.
- [Source: cartographie 3 agents Explore 2026-07-09 — mail/config/rate-limit, kesh-db/socle 20-1/audit, routes/E2E] (références précises dans les Dev Notes ci-dessus).
- [Source: `_bmad-output/implementation-artifacts/20-1-templates-email-socle.md`] — socle consommé (get_effective, moteur, variables).
- [Source: `_bmad-output/implementation-artifacts/20-3a-service-pdf-facture.md`] — `invoice_pdf_service::render(pool, i18n, locale, company: &Company, invoice_id)` (signature post-review Pass 1).
- [Source: CLAUDE.md §Test Locally First, §Règle de splitting préventif, §Migration breaking policy P5]

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5) — run 2026-07-09 interrompu par un crash de session (implémentation T1-T6 quasi complète dans le working tree, non commitée), repris et complété le 2026-07-09 par un second run Fable 5.

### Debug Log References

- **Crash & reprise** : le premier run a été interrompu après avoir écrit l'essentiel de T1-T6 (48 fichiers modifiés + 5 nouveaux, tout compilait, clippy 0 warning). Reprise avec inventaire ground-truth (grep par AC) avant de cocher quoi que ce soit — aucun fichier tronqué détecté.
- **Réparation DB dev `kesh`** (pré-requis tests `--lib` kesh-db, cassée AVANT le crash, depuis le 2026-07-05) : migration `20260705000001_structured_addresses` marquée `success=0` alors que son DDL était entièrement appliqué (DDL MariaDB non transactionnel — crash antérieur entre DDL et bookkeeping) ; idem `20260705000002` (row absente puis `success=0` après une tentative `sqlx migrate run`) et `20260706000001` (DDL appliqué, row absente). Fix : `UPDATE _sqlx_migrations SET success=1` (checksums vérifiés SHA-384 contre les fichiers) + INSERT de la row manquante, puis `sqlx migrate run` a appliqué proprement `20260708000001` + les 3 migrations de la story. kesh-db `--lib` : 57 échecs (`Unknown column 'language'`) → 219/219 verts après réparation.
- Test e2e `company_email_endpoint_sets_reply_to` : premier rouge — `GET /companies/current` renvoie `{ company: {...}, bankAccounts: [...] }` (enveloppe), pas `CompanyJson` nu ; fix de l'assertion (`current["company"]["version"]`).
- **Régression attrapée par le gate workspace complet** : `idor_multi_tenant_e2e::idor_contacts_cross_company_returns_404` → 500 sur `GET /contacts/{id}` (contact de sa propre company). Cause : 3 requêtes `query_as` à liste de colonnes **inline** (pas les constantes) non étendues par le premier run → `ColumnNotFound` au décodage : `contacts::find_by_id_in_company` (contacts.rs:287, manquait `language, salutation`), `reconciliation.rs:177` SELECT contacts (idem), `credit_notes.rs:223` lock facture d'origine (manquait `emailed_at, emailed_to` — aurait cassé la création d'avoir). Fix DRY : `contacts::COLUMNS` et `invoices::FIND_INVOICE_SCOPED_SQL` passés `pub(crate)` et réutilisés aux 3 sites (une seule liste à maintenir, la classe de bug ne peut plus se reproduire sur ces chemins). Vérifié par balayage exhaustif de tous les `query_as::<_, Contact|Invoice|Company>` du workspace — aucun autre site à liste stale.

### Completion Notes List

- **T1-T6 conformes à la spec** ; gate T7 vert (fmt / build / clippy -D warnings / workspace série). Détail tests : kesh-api `--lib` 284 (dont matrice salutation ×4 langues, vars + fallbacks, `build_outgoing_message` ×4, `smtp_configured` ×2), kesh-db `--lib` 219, e2e `invoice_send_email_e2e` 16/16, sentinelles `password_recovery_e2e` 11/11 et `invoice_pdf_e2e` 14/14 inchangées vertes.
- **Déviation (garde 412)** : la garde utilise `state.smtp_ready` (bool AppState) et non `config.smtp_configured()` directement. `smtp_ready` = config complète **ET** mailer réellement construit (posé dans `main.rs` après le `match` de build). Plus fidèle que la spec : si `SmtpMailer::from_config` échoue (relay STARTTLS), le fallback est NoopMailer et `smtp_ready=false` → 412 au lieu d'un envoi fantôme marqué « envoyée ». `/health.smtpConfigured` expose le même bool (cohérence garde/flag). Défaut `false` dans `new_for_tests` ; les e2e l'injectent via AppState littéral.
- **Déviation (AC#5 « DTO route update company »)** : aucune route générique d'update company n'existe (seul l'onboarding `set_coordinates` touche les coordonnées). Créé un endpoint dédié minimal `PUT /api/v1/companies/current/email` (Admin-only, groupe admin_routes à côté de la config templates), validation `is_valid_email_simple` (clé FTL `error-company-email-invalid` ×4), verrou optimiste via `companies::update` réutilisé (`CompanyUpdate` étendu de `email`, `is_no_op_change` inclus — piège KF-004 évité), `CompanyJson` expose `email` + `version`. Nécessaire pour que la 20-3b2 puisse saisir l'e-mail société (Reply-To).
- **Complément e2e à la reprise** : + `send_uses_contact_language_for_pdf` (envoi contact DE → 200 + PDF joint non vide via `Locale::from("DE")` ; le texte du PDF n'est pas extractible — streams compressés, même limite qu'`invoice_pdf_e2e`) ; + 3 tests endpoint company email (Reply-To happy path + effacement, e-mail invalide 400, RBAC Comptable 403).
- **Rate-limiter** : généralisé `RateLimiter<K: Eq+Hash+Clone = IpAddr>` — zéro diff sur les call-sites login/recovery ; instance `(company_id, user_id)` 20/15min/15min ; seuils injectables en test via `RateLimiter::with_thresholds`.
- Périmètre respecté : 0 fichier frontend, recovery (`CapturedMail`/`send_password_reset`) intact, `invoice_pdf_service` consommé tel quel (signature `&Company` 20-3a).

### File List

**Nouveaux**

- `crates/kesh-db/migrations/20260709000001_contacts_language_salutation.sql`
- `crates/kesh-db/migrations/20260709000002_invoices_emailed.sql`
- `crates/kesh-db/migrations/20260709000003_companies_email.sql`
- `crates/kesh-api/src/routes/invoice_email.rs`
- `crates/kesh-api/tests/invoice_send_email_e2e.rs`

**Modifiés — kesh-db**

- `crates/kesh-db/src/entities/contact.rs` (enum `Salutation`, `Contact`/`NewContact`/`ContactUpdate` + `language`/`salutation`)
- `crates/kesh-db/src/entities/invoice.rs` (`emailed_at`/`emailed_to`)
- `crates/kesh-db/src/entities/company.rs` (`Company.email`, `CompanyUpdate.email`)
- `crates/kesh-db/src/repositories/contacts.rs` (COLUMNS/INSERT/UPDATE/binds, `is_no_op_change`, `contact_snapshot_json`)
- `crates/kesh-db/src/repositories/invoices.rs` (constantes SQL + `mark_emailed` + audit `invoice.emailed` in-tx)
- `crates/kesh-db/src/repositories/companies.rs` (colonnes/UPDATE + `is_no_op_change` email)
- `crates/kesh-db/src/repositories/reconciliation.rs` (liste de colonnes invoices ; SELECT contacts re-basé sur `contacts::COLUMNS`)
- `crates/kesh-db/src/repositories/credit_notes.rs` (lock facture re-basé sur `invoices::FIND_INVOICE_SCOPED_SQL`)
- `crates/kesh-db/tests/migrations_upgrade_path.rs` (compteur 47 → 50)
- `crates/kesh-db/tests/{companies_repository,credit_notes_repository,invoices_validate_vat,kf005_fulltext_index_e2e,payment_batches_repository,reconciliation_repository,supplier_invoices_repository}.rs` (littéraux étendus, mécanique)

**Modifiés — kesh-api**

- `crates/kesh-api/src/mail/mod.rs` (`OutgoingEmail`/`EmailAttachment`, trait `send_email`, `NoopMailer`, `MockMailer.sent_emails()`/`CapturedEmail`)
- `crates/kesh-api/src/mail/smtp.rs` (`build_outgoing_message` + `send_email` MultiPart/display-name/Reply-To tolérant + tests)
- `crates/kesh-api/src/config.rs` (`smtp_configured()` + tests)
- `crates/kesh-api/src/main.rs` (construction mailer découplée, `smtp_ready`, dégradation gracieuse)
- `crates/kesh-api/src/lib.rs` (AppState `rate_limiter_send_email`/`smtp_ready`, fabrique, routes email-preview/send-email/companies-email)
- `crates/kesh-api/src/errors.rs` (3 variants 412/400/422)
- `crates/kesh-api/src/middleware/rate_limit.rs` (`RateLimiter<K = IpAddr>` générique + `with_thresholds`)
- `crates/kesh-api/src/middleware/auth.rs` (test_state littéral étendu)
- `crates/kesh-api/src/routes/{mod,health,invoices,contacts,companies,onboarding}.rs` + `src/exports/metadata.rs`
- `crates/kesh-api/tests/{inbox_import,invoice_delete,invoice_echeancier,invoice_pdf,password_recovery,reconciliation,setup_admin,vat_report}_e2e.rs` (AppState littéraux étendus, mécanique)

**Modifiés — autres**

- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (4 clés d'erreur ×4 langues)
- `crates/kesh-reconciliation/src/matching.rs`, `crates/kesh-seed/src/lib.rs` (littéraux/colonnes, mécanique)
- `docs/migrations-idempotence-audit.md` (3 entrées + total 50)

## Senior Developer Review (AI)

### Pass 1 — 2026-07-10, Sonnet 5 × 3 reviewers (Blind Hunter + Edge Case Hunter + Acceptance Auditor), contexte frais, diff aplati `695475b7..e308e7e1`

**9 findings bruts → 1 HIGH + 4 MEDIUM + 4 LOW. Triage : 4 patchs + 1 réfuté ground-truth + 3 LOW traités (1 fix DRY, 1 dismiss documenté, 1 documenté) + AA 20/20 ACs conformes/déviations justifiées.**

- **[HIGH → patché] ECH-1 (+BH-3 même racine) — envoi SMTP et marquage non atomiques** : facture supprimée (#219) ou annulée par avoir entre l'envoi et `mark_emailed` → l'UPDATE (`AND status='validated'`) fait 0 row → 404 trompeur alors que l'e-mail est PARTI, aucune trace d'audit, retry client = doublon. Patch : (1) `mark_emailed` ne conditionne plus sur `status` (un flip `cancelled` mid-flight doit tracer l'envoi) ; (2) le handler mappe le NotFound post-envoi en **409 `EMAIL_SENT_INVOICE_GONE`** (nouveau variant + FTL ×4, message explicite « ne renvoyez pas ») + **trace d'audit best-effort** `invoice.emailed` avec `invoiceGone: true` (audit_log n'a pas de FK sur entity_id — vérifié). Tests : `mark_emailed_survives_cancelled_status` + `mark_emailed_deleted_invoice_returns_not_found` (la fenêtre exacte n'est pas simulable en E2E ; sémantique repo prouvée + mapping vérifié par lecture).
- **[MEDIUM → patché] ECH-2 — contact archivé non exclu** : `find_by_id` ne filtre pas `active` → un contact archivé recevait encore des factures. Patch : helper `load_active_contact` (preview + send) → **400 `CONTACT_ARCHIVED`** (nouveau variant + FTL ×4) + test e2e `archived_contact_returns_400` (preview + send + non-marquage).
- **[MEDIUM → patché] BH-2 — lookup contact non scopé company** : `contacts::find_by_id` (WHERE id seul) au lieu du variant scopé — non exploitable aujourd'hui (`invoice.contact_id` vient d'une facture scopée) mais défense en profondeur, même classe que les 3 sites corrigés au gate. Patch : `find_by_id_in_company` via `load_active_contact`.
- **[MEDIUM → réfuté ground-truth] BH-1 — injection CRLF via `subject`** : test empirique ajouté (`build_outgoing_message_subject_crlf_never_injects_header`) — lettre neutralise le CRLF (aucun en-tête `Bcc:` injecté dans le message formaté). Finding réfuté, test conservé en garde de régression permanente.
- **[MEDIUM → patché] AA-1 — AC#19 audit testé par comptage seulement** : ajout `last_audit_emailed_details` + assertions `details.to`/`details.subject` dans le happy path.
- **[LOW → fix DRY] BH-5** : résolution du destinataire dupliquée preview/send → helper `locked_recipient`.
- **[LOW → dismiss documenté] BH-4 — CHECK `salutation` sans `BINARY`** : vrai écart cosmétique avec `chk_contacts_language`, mais (1) tout write API passe par l'enum sqlx `Salutation` (casse exacte garantie), (2) la migration est déjà appliquée aux DB dev/e2e — la modifier casserait les checksums `_sqlx_migrations` (VersionChecksumMismatch au boot). Risque résiduel = INSERT SQL direct hors API, hors modèle de menace v0.1. Non corrigé.
- **[LOW → documenté] AA-2 — `ContactResponse.salutation: Salutation` non-Option** : la lettre de l'AC#5 (`Option<Salutation>`) visait les DTOs de requête (qui sont bien `Option` + `#[serde(default)]`) ; la colonne étant `NOT NULL DEFAULT 'Neutre'`, la réponse expose le type plein. Déviation assumée, documentée ici.

Gate post-patchs : voir Change Log.

## Change Log

- 2026-07-10 — `bmad-code-review` Pass 1 (Sonnet 5 × 3) : 9 findings bruts → 1 HIGH + 4 MEDIUM patchés/réfutés (détail section Senior Developer Review). 2 nouveaux variants `AppError` (409 `EMAIL_SENT_INVOICE_GONE`, 400 `CONTACT_ARCHIVED`) + FTL ×4, `mark_emailed` sans condition status, helpers `locked_recipient`/`load_active_contact`, 4 nouveaux tests (e2e 19/19). Pass 2 (LLM différent — Haiku) à suivre, conformément à la Review Iteration Rule.
- 2026-07-09/10 — `bmad-dev-story` : implémentation complète T1-T7. Run initial interrompu par un crash de session après l'écriture de T1-T6 (non commitée) ; reprise le jour même avec inventaire ground-truth, réparation de l'état `_sqlx_migrations` de la DB dev (cassé pré-existant, sans lien avec le crash), complément de tests e2e (langue DE à l'envoi + endpoint company email ×3). Le gate workspace complet a attrapé 1 régression du run crashé (3 `query_as` à listes de colonnes inline non étendues → 500 ; fix DRY par constantes `pub(crate)` réutilisées). Déviations documentées : garde 412 via `smtp_ready` (config + build mailer réussis), endpoint dédié `PUT /companies/current/email` (aucune route update company générique n'existait).
