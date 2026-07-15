# Story 21.5a: Données & éligibilité relances (backend)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **comptable/fiduciaire utilisant Kesh**,
I want **que le système identifie automatiquement les factures à relancer (échues, non payées, non suspendues), à quel niveau de rappel, et me permette de suspendre/reprendre une facture, d'enregistrer un rappel papier manuel et d'annuler un rappel envoyé par erreur — le tout tracé**,
so that **je pilote le recouvrement débiteur sans calcul manuel des échéances et en gardant une preuve auditée de chaque relance (dossier pré-contentieux)**.

## Contexte

Épic 21 « Échéances & relances débiteurs ». Le socle de configuration (niveaux de rappel `dunning_levels`, période de grâce `company_dunning_settings`, templates par niveau) a été livré par **21-3** (done). L'UI de réglages par **21-4** (done). Cette story **21-5a** pose la couche **données + éligibilité + endpoints backend** de gestion des rappels, **sans envoi e-mail** (l'envoi — `build_reminder_vars`, preview, unitaire, lot — est **21-5b**, qui consomme cette story).

Périmètre exact (plan d'epic, items 8-13, 16 partiel, 19 pattern, 22, 26) :
- Table **`invoice_reminders`** append-only (historique/snapshots des rappels).
- Colonnes **`invoices.dunning_paused_at`** + **`dunning_paused_note`** (suspension par facture).
- **Requête d'éligibilité SQL pure** (niveau courant = MAX level_number des rappels non-annulés ; prochain niveau ; date de déclenchement grâce+délai).
- **Endpoints** : liste à rappeler (groupée par contact, badge sans-email, état terminal), toggle suspension, rappel manuel, annulation d'un rappel (Admin).
- **Audits** de chaque action.
- **Export souveraineté + backup admin** : `invoice_reminders` ajoutée (les 2 autres tables l'ont été en 21-3).

**Prérequis consommés** : 21-2 (helper TTC `invoice_total_ttc` #246) pour `{totalDue}` — mais le calcul des frais cumulés / `{totalDue}` appartient surtout à **21-5b** (`build_reminder_vars`). En 21-5a, l'éligibilité et la liste n'ont **pas** besoin du TTC ; les snapshots `fee_amount` sont posés à l'enregistrement d'un rappel. À arbitrer en validate : si la liste doit afficher un montant, réutiliser le helper TTC existant, sinon différer à 21-5b/21-6.

## Acceptance Criteria

### Base de données & migrations

1. **Migration `invoice_reminders`** (nouveau fichier `crates/kesh-db/migrations/AAAAMMJJ000001_invoice_reminders.sql`) crée la table append-only avec, au minimum :
   - `id BIGINT PK AUTO_INCREMENT`, `company_id BIGINT NOT NULL`, `invoice_id BIGINT NOT NULL`.
   - `level_number SMALLINT NOT NULL` (niveau du rappel ; CHECK `>= 1`), `fee_amount DECIMAL(7,2) NOT NULL` (**snapshot** du frais du niveau au moment de l'envoi ; **aligné exactement sur `dunning_levels.fee_amount DECIMAL(7,2)` + CHECK `>= 0 AND <= 10000`** — `20260714000001_dunning_config.sql:31`).
   - `sent_at DATETIME(6) NOT NULL`, `channel VARCHAR(16) NOT NULL` (CHECK `IN ('email','manual')`), `sent_to VARCHAR(320) NULL` (destinataire e-mail réel ; NULL si `channel='manual'`).
   - `subject TEXT NOT NULL`, `body TEXT NOT NULL` (**snapshots** du texte réclamé — preuve pré-contentieuse ; pour un rappel **manuel** : `subject` = `« Rappel manuel — niveau {N} »`, `body` = copie de `note` ou chaîne vide — cf. AC 12/M3).
   - `note TEXT NULL` (note du rappel manuel), `actor_user_id BIGINT NULL` (acteur), `cancelled_at DATETIME(6) NULL` (annulation soft — exclue du MAX niveau).
   - **FK `invoice_id` → `invoices(id)` ON DELETE CASCADE** (aligné sur la suppression définitive #219 ; l'audit_log sans FK reste la trace résiduelle, comme `invoices.emailed_at`), **FK `company_id` → `companies(id)`** (ON DELETE RESTRICT, cohérent invoices).
   - Index `(company_id, invoice_id)` + `(invoice_id)` pour le calcul du MAX niveau.
2. **Analyse breaking (P1-P5 CLAUDE.md)** : `CREATE TABLE` d'une nouvelle entité = **non-breaking** (un binaire antérieur l'ignore) → **PAS** de bump `kesh_version_min_required`. La migration ajoutant `dunning_paused_at`/`dunning_paused_note` est un `ADD COLUMN` nullable → non-breaking également. La migration idempotence est ajoutée à `docs/migrations-idempotence-audit.md` (verdict + justification) — **finding MEDIUM en code-review si oublié (P5)**.
3. **Migration colonnes suspension** (peut être dans le même fichier ou un fichier séparé `AAAAMMJJ000002_invoices_dunning_paused.sql`) : `ALTER TABLE invoices ADD COLUMN dunning_paused_at DATETIME(6) NULL, ADD COLUMN dunning_paused_note VARCHAR(500) NULL;` (pattern `emailed_at`/`emailed_to` `20260709000002_invoices_emailed.sql:10-12`). Nullable → non-breaking. **PIÈGE OBLIGATOIRE — checklist fermée (ground-truth grep-vérifié Pass 3, cause n°1 d'échec)** : ces 2 colonnes DOIVENT être ajoutées au struct FromRow `entities/invoice.rs` (`pub struct Invoice`, après `emailed_to:38`) **ET** aux **3 SELECT qui désérialisent vers `struct Invoice`** dans `repositories/invoices.rs` (repérables par la présence de `emailed_at, emailed_to` dans la liste de colonnes — grep `emailed_to` → lignes 46/909/1707) :
   1. **`FIND_INVOICE_SCOPED_SQL`** (const, `:44-48`, `emailed_to` à `:46`) — couvre à elle seule les 11 `query_as::<_, Invoice>` (451/484/764/852/865/1177/1350/1442/1509/1604/3897).
   2. **`delete`** — `SELECT … FOR UPDATE` **inliné** (`:907-911`, `emailed_to` à `:909`, pas via la const).
   3. **`list_all_by_company`** — **inliné** (`:1705-1709`, `emailed_to` à `:1707`, pas via la const).
   **NE PAS toucher** `list_by_company_paginated` (`:507`, retourne `InvoiceListResult`/`Vec<InvoiceListItem>`) ni `list_for_export` (`:1652`, `Vec<InvoiceListItem>`) : ils désérialisent vers **`struct InvoiceListItem`** (`:187`, 15 champs, **sans** `emailed_*`/`dunning_*`) et n'ont PAS besoin des nouvelles colonnes. **Ne PAS ajouter `dunning_paused_*` à `InvoiceListItem`** (casserait les autres requêtes `InvoiceListItem` non mises à jour). Exposer l'état de suspension dans la **liste factures** (badge/filtre D10) est **différé à 21-6**. Une colonne au struct `Invoice` sans le SELECT correspondant → `ColumnNotFound` au runtime ; l'inverse (colonne en trop dans un SELECT `InvoiceListItem`) est ignoré par sqlx mais inutile. **Revérifier les numéros de ligne au dev** (le fichier bouge).

### Entité & repository `invoice_reminders`

4. **Entité** `crates/kesh-db/src/entities/invoice_reminder.rs` : struct `InvoiceReminder` (champs miroir DB, `fee_amount: rust_decimal::Decimal` avec `#[serde(with = "rust_decimal::serde::str")]` — Decimal (dé)sérialisé en **STRING**, jamais number), `channel` typé (enum `ReminderChannel { Email, Manual }` avec `as_str`/`FromStr` → valeurs DB **minuscules** `'email'`/`'manual'` cohérentes avec le CHECK AC 1 ; **`#[serde(rename_all="lowercase")]`** pour la sérialisation JSON, L5-p2). Déclaré dans `entities/mod.rs`.
5. **Repository** `crates/kesh-db/src/repositories/invoice_reminders.rs` :
   - `insert_in_tx(tx, NewInvoiceReminder) -> Result<InvoiceReminder>` : append d'une ligne (rappel manuel ici ; l'envoi e-mail réutilisera le même insert en 21-5b), **dans une transaction** avec l'audit `invoice.reminder_sent`.
   - `list_for_invoice(pool, company_id, invoice_id) -> Vec<InvoiceReminder>` : historique d'une facture (non-annulés + annulés distinguables), scopé company (anti-IDOR — cross-tenant → vide/404, jamais 403).
   - `current_level(pool/tx, company_id, invoice_id) -> i16` : **MAX(`level_number`) des rappels `cancelled_at IS NULL`** (0 si aucun) — PAS un COUNT (permet ré-envoi et rappel manuel à niveau choisi sans fausser la cadence).
   - `cancel_in_tx(tx, company_id, reminder_id, actor) -> Result<()>` : pose `cancelled_at = NOW(6)` (soft, préserve l'append-only) + audit `invoice.reminder_cancelled`. Scopé company.
6. **Bornes & validation** : `fee_amount` snapshot borné `0..=10000.00` scale 2 (réutiliser la constante/validation de `dunning_levels` — ne PAS dupliquer) ; `level_number >= 1` pour un rappel enregistré ; `channel` ∈ {email, manual}.

### Éligibilité SQL

7. **Requête d'éligibilité** (SQL pur, dans `repositories/invoice_reminders.rs` ou un module dédié `dunning_eligibility.rs`) : pour une company, retourne les factures **candidates au rappel** avec leur niveau courant, prochain niveau, date de déclenchement et statut terminal. Prédicats de base :
   - **GARDE « dunning activé » (C1 — D7)** : si la company a **0 niveau configuré** (`dunning_levels::count_for_company(company_id) == 0`), la liste est **vide** — aucune facture candidate NI terminale. Une table `dunning_levels` vidée volontairement = dunning **désactivé** (discriminant `seeded_at` non-NULL, pas de résurrection par le seed lazy) ; il ne faut donc PAS afficher les factures échues en « dernier niveau atteint » (ce serait l'inverse de la sémantique D7 — un état terminal signale une fin de cycle *après envois*, pas un dunning jamais activé). Le seed lazy de l'AC 7 ne s'applique qu'à une company **jamais seedée** (`seeded_at IS NULL`) → 3 niveaux posés ; une company seedée-puis-vidée reste à 0 niveau.
   - `invoices.status = 'validated' AND invoices.paid_at IS NULL AND invoices.dunning_paused_at IS NULL`.
   - `due_date` non-NULL (**COALESCE**/garde : une facture sans `due_date` n'est pas éligible — pas de rappel sur échéance inconnue ; à confirmer validate).
   - **niveau courant** = `COALESCE(MAX(r.level_number) FILTER cancelled_at IS NULL, 0)` (LEFT JOIN `invoice_reminders`).
   - **prochain niveau** = plus petit `dunning_levels.level_number` **strictement supérieur** au niveau courant.
   - **échéance de déclenchement** : niveau 1 ⇒ `today >= due_date + grace_period_days + delay_days(niveau 1)` ; niveau N>1 ⇒ `today >= sent_at(dernier rappel non-annulé) + delay_days(niveau N)`.
   - `grace_period_days` lu depuis `company_dunning_settings` (via `get_or_create`/seed 21-3 — l'éligibilité déclenche le **seed lazy** si nécessaire, cf. 21-3 « au premier accès config OU à la première évaluation d'éligibilité »).
8. **État terminal (fin de cycle visible — HIGH produit, item 9)** : **pour une company avec ≥ 1 niveau configuré** (la garde C1 de l'AC 7 s'applique d'abord), si le niveau courant = MAX(`level_number`) de la config (aucun niveau supérieur à envoyer) **et qu'au moins un rappel a été envoyé** (niveau courant ≥ 1), la facture **reste dans la liste** en état terminal `« Dernier niveau atteint — poursuite à envisager »` avec la date du dernier rappel — **jamais de sortie silencieuse**. Le DTO expose un champ discriminant (ex. `terminal: bool` + `last_reminder_at`). **Distinguer** terminal (fin de cycle après envois) de « dunning désactivé » (0 niveau → absente, C1).
9. **`unmark-paid` post-rappel** (item 28) : une facture dé-payée redevient immédiatement éligible au niveau suivant (sent_at ancien) — comportement acceptable, **documenté** (pas de logique spéciale à coder).

### Endpoints & RBAC

10. **Liste à rappeler** `GET /api/v1/dunning/reminders` (nom **figé** — cohérent avec le préfixe `dunning_*`/`/api/v1/dunning-levels` de 21-3/21-4) — **RBAC Comptable+** : retourne les factures éligibles **groupées par contact**. **Forme DTO figée (M2)** : réponse **imbriquée** `{ groups: Vec<ContactGroup> }` avec `ContactGroup { contact_id, contact_name, has_email: bool, invoices: Vec<ReminderCandidate> }` et `ReminderCandidate { invoice_id, invoice_number, due_date, current_level, next_level (nullable si terminal), terminal: bool, last_reminder_at (nullable) }`. **`last_reminder_at` = `MAX(sent_at)` des rappels non-annulés (`cancelled_at IS NULL`), NULL si aucun rappel (L3-p2).** `has_email` porté au niveau **contact** (une seule adresse par contact). Scopé company (`current_user.company_id`). DTOs `#[serde(rename_all="camelCase")]`, **sans `company_id`**. Réutiliser le scoping + l'agrégat SQL de `due_dates_summary` (pas la forme plate de `list_due_dates`). Ce contrat est consommé par 21-6 — le figer évite une divergence backend/tests E2E aval.
11. **Toggle suspension** — **deux routes distinctes figées (M1)** `PUT /api/v1/invoices/{id}/dunning-pause` (pause) + `PUT /api/v1/invoices/{id}/dunning-resume` (reprise) — plus simples à auditer/tester qu'un toggle unique à body discriminant, cohérent avec le couple `mark/unmark` existant — **RBAC Comptable+** : la reprise (`dunning-resume`) pose **`dunning_paused_at = NULL` ET `dunning_paused_note = NULL`** (M4 — le champ n'est pas append-only ; sans reset, une note de pause n°1 persisterait faussement lors d'une pause n°2 sans note ; l'audit_log garde la trace exacte). La pause pose `dunning_paused_at = UTC_TIMESTAMP(6)` + `dunning_paused_note` optionnelle. **Calquer exactement `mark_as_paid`** (`repositories/invoices.rs:1430-1556`) : `SELECT … FOR UPDATE` scopé company (`FIND_INVOICE_SCOPED_SQL`), verrou optimiste (`version = ?` dans le WHERE de l'UPDATE + `rows==0 → OptimisticLockConflict`), audit **dans la même tx**. Le DTO request porte `version` (validé `>= 0` → 400 explicite, cf. `MarkPaidRequest`/`validate_version` `invoices.rs:756-780`). **Garde anti-transition** (transposer `alreadyUnpaid` `invoices.rs:1468`) : reprise d'une facture **non suspendue** → `InvalidInput` 4xx (évite un faux 409 + audit spurieux). **Pause ET reprise auditées** (`invoice.dunning_paused` / `invoice.dunning_resumed`, acteur + facture, via `NewAuditLogEntry::from_current_user` ou `::user`). **Invariant anti-dissimulation (sécurité, item 10)** : une facture suspendue **reste dans la balance âgée et l'échéancier** — elle ne sort **que** de la liste « à rappeler » (l'AC 7 filtre `dunning_paused_at IS NULL` ; l'échéancier `list_due_dates` ne filtre PAS dessus). Scopé company (cross-tenant → 404, jamais 403). **Request DTO** (figé) : pause `{ version: i32, note?: String }`, reprise `{ version: i32 }`. **Codes** : version obsolète → **409 `OPTIMISTIC_LOCK_CONFLICT`** ; version négative → **400 `VALIDATION_ERROR`** ; reprise d'une facture non suspendue → **422 `INVOICE_NOT_PAUSED`** ; cross-tenant/absente → **404 `NOT_FOUND`**.
12. **Rappel manuel** `POST /api/v1/invoices/{id}/reminders/manual` — **RBAC Comptable+** : enregistre une ligne `invoice_reminders` `channel='manual'`, `sent_to = NULL`, niveau + date + note fournis, `fee_amount` = snapshot du frais du niveau visé (lookup `dunning_levels` config courante). Le cycle avance sans e-mail Kesh. **Gardes d'éligibilité** : facture `status='validated'` et `paid_at IS NULL` → sinon 4xx explicites. **Niveau accepté (H2 / D18)** : `level_number >= 1` **ET** existant dans `dunning_levels` de la company (nécessaire au lookup `fee_amount`) — **AUCUNE borne supérieure liée au niveau courant**. Le rappel manuel est explicitement le mécanisme produit du **saut de niveau vers le haut** (ex. mise en demeure niveau 3 directement, sans repasser par 1-2) ; contrairement à l'envoi e-mail unitaire (21-5b) borné à `≤ prochain`. Audit `invoice.reminder_sent` (channel manual). **`subject`/`body`** (M3) : `subject` = libellé auto `« Rappel manuel — niveau {N} »`, `body` = copie de `note` (ou chaîne vide si absente) — préserve la valeur probatoire du snapshot.
   - **Request DTO** (figé) : `{ levelNumber: i16, sentAt: DateTime, note?: String }` (camelCase).
   - **Garde `sent_at` non-futur (M2-p2)** : `sent_at <= UTC_TIMESTAMP(6)` sinon **422 `REMINDER_DATE_IN_FUTURE`** (`UTC_TIMESTAMP` et non `NOW()` pour rester cohérent avec la discipline UTC de l'éligibilité, H1/L2-p3). Rationale : un `sent_at` futur ferait monter le `current_level` mais reporterait le calcul du prochain niveau (`UTC_DATE() >= DATE(sent_at) + delay`) → **gèle silencieusement le cycle** jusqu'à cette date future.
   - **Codes d'erreur explicites (M1-p2)** : facture absente/cross-tenant → **404 `NOT_FOUND`** ; `status != 'validated'` → **422 `INVOICE_NOT_VALIDATED`** ; `paid_at` non-NULL → **422 `INVOICE_ALREADY_PAID`** ; `level_number` absent de `dunning_levels` → **422 `DUNNING_LEVEL_NOT_FOUND`** ; `level_number < 1` ou `sent_at` futur → **422** (`VALIDATION_ERROR`/`REMINDER_DATE_IN_FUTURE`). *(Note : une facture **suspendue** n'empêche PAS l'enregistrement d'un rappel manuel — le rappel manuel est une trace historique ; la suspension ne retire que de la liste auto « à rappeler ».)*
13. **Annulation d'un rappel** `POST /api/v1/invoices/{id}/reminders/{reminder_id}/cancel` (ou DELETE soft) — **RBAC Admin** (envoi accidentel = correction sensible) : pose `cancelled_at` soft, exclut du MAX niveau, audité `invoice.reminder_cancelled`. Scopé company.
14. **Historique** `GET /api/v1/invoices/{id}/reminders` — **RBAC tous rôles authentifiés (lecture)** : liste des rappels d'une facture (annulés distinguables). Alimente la fiche facture (21-6). **Response DTO** (figé, camelCase, sans `company_id`) : `Vec<ReminderResponse { id, levelNumber, feeAmount (string), sentAt, channel, sentTo (nullable), subject, body, note (nullable), cancelledAt (nullable) }>`, trié `sent_at DESC`. Scopé company (cross-tenant → liste vide / 404).
15. **Montage RBAC anti-footgun** (item 22, AC de test) : les mutations Comptable+ dans le bloc `require_comptable_role` (`lib.rs`), l'annulation Admin dans `admin_routes` (`require_admin_role`), la lecture historique dans le routeur authentifié simple. **Test explicite** qu'une route n'est pas déclarée **après** le `;` de fermeture du `route_layer` (bypass auth silencieux — piège Axum 0.8). Verrou optimiste/`FOR UPDATE` sur la row `invoices` là où un update concurrent est possible (toggle suspension, insert rappel).

### Export souveraineté & backup

16. **`invoice_reminders` ajoutée à l'export global** (3 endroits, calqués 21-3) : (a) `serialize_invoice_reminders_csv` dans `exports/csv_tables.rs` (précédent `serialize_dunning_levels_csv:664-696`) + import entité `InvoiceReminder` (`:27-31`) ; (b) imports serializer+repo dans `exports/global.rs:25-41` ; (c) query repo dans `build_global_export` (`global.rs:172-180`) + macro `push_csv!("invoice_reminders.csv", …)` (`global.rs:257-266`). **ET `admin_backup`** : ajouter `"invoice_reminders"` à `TABLES_TO_TRUNCATE` (`backup.rs:34-71`) **au bon rang FK** — enfant de `invoices` (CASCADE) → **avant `invoices`** dans la liste (ordre enfants→parents FK-safe ; le commentaire `backup.rs:64` réserve déjà la place pour 21-5a). L'énumération backup est dynamique (aucun compteur en dur côté export/restore). Le test auto-vérifié `backup_inventory_matches_schema` (`backup.rs:581`) rougit en **égalité stricte** si la table est créée sans être ajoutée à la constante (filet). **`invoice_reminders` = pièce du dossier de recouvrement** (preuve de mise en demeure) — absence de l'export = perte juridique (item 26).
17. **Compteurs de tests figés bumpés** — valeurs EXACTES (ground-truth 2026-07-15, **à revalider au dev** avant commit) :
    - `crates/kesh-api/src/exports/global.rs:278` `18 → 19` ; `:279` `18 → 19` ; `csv_count: 18 → 19` (~`:294`, grep `csv_count`) ; `:193` capacité `19 → 20` ; doc-comments en-tête `:3-15` (« 18 tables » → 19).
    - `crates/kesh-api/tests/admin_full_export_e2e.rs:274` `data_count 36 → 37` (compteur principal backup).
    - `crates/kesh-api/tests/exports_global_e2e.rs:619` `19 → 20` ; `:695` `18 → 19` ; **set `expected` `:622-645` ajouter `"invoice_reminders.csv"`** (égalité stricte `:646`) ; map rowCounts `:777-797` ajouter `("invoice_reminders.csv", 0)`. ⚠️ **NE PAS TOUCHER `:685` `19 => c == 'Z'`** (index de caractère dans `exportDate`, PAS un compteur de tables).
    - `crates/kesh-db/tests/migrations_fresh_install.rs:30-56` ajouter `invoice_reminders` à la liste `expected` (sémantique subset, non bloquant mais bonne pratique — `company_dunning_settings` y figure `:40`, `dunning_levels` `:43`).
    - **Compteur migrations** (`migrations_upgrade_path.rs`) : `ls crates/kesh-db/migrations/*.sql | wc -l` (53 post-21-3) + `grep -nF 'assert_eq!' migrations_upgrade_path.rs` → incrémenter du nombre de fichiers migration ajoutés (1 ou 2). **Vérifier la valeur exacte au dev.**
    - `docs/migrations-idempotence-audit.md` : ligne après `:84` (verdict `tracked-by-sqlx`, justif calquée `dunning_config:83`) + stats `:67` total `53 → 54` (ou +2), `:69` tracked-by-sqlx `42 → 43`.

### Tests

18. **Tests repo `invoice_reminders`** (`#[sqlx::test]`) : insert + `current_level` (MAX non-annulés, ignore annulés), `cancel_in_tx` (soft, exclut du MAX), scoping company (cross-tenant vide), snapshots préservés.
19. **Tests éligibilité SQL** : facture non-échue → absente ; échue niveau 1 dû → présente niveau 1 ; après rappel niveau 1, niveau 2 dû à `sent_at + delay` ; suspendue → absente de la liste mais **présente échéancier/balance âgée** (invariant item 10) ; payée → absente ; dernier niveau atteint (≥ 1 niveau configuré, tous envoyés) → **présente en terminal** (item 9) ; **`dunning_levels` vide (company seedée-puis-vidée, dunning désactivé D7) → facture absente (NI candidate NI terminale, C1)** ; company jamais seedée → seed lazy pose 3 niveaux à la 1re évaluation puis éligibilité normale ; sans `due_date` → absente ; **rappel `cancelled_at` non-NULL exclu du MAX niveau** (le niveau courant retombe).
20. **Tests E2E routes** (`crates/kesh-api/tests/`) : liste groupée par contact (forme imbriquée `groups[]`) + badge `has_email` ; toggle suspension (pause+reprise audités, `dunning_paused_note` remis à NULL à la reprise, facture suspendue absente de la liste ; reprise d'une facture non-suspendue → 4xx) ; rappel manuel (ligne créée, cycle avance) ; **rappel manuel niveau > next_level (saut, D18/H2) → accepté, `current_level` avance directement au niveau saisi** ; annulation Admin (soft, exclue du MAX → `current_level` retombe) ; **RBAC** : Comptable OK / Consultation 403 sur les mutations, Admin requis pour l'annulation (403 pour Comptable) ; **anti-IDOR** cross-tenant → 404 (jamais 403) ; gardes d'éligibilité (payée → 422 `INVOICE_ALREADY_PAID` ; niveau inexistant → 422 `DUNNING_LEVEL_NOT_FOUND` ; **`sent_at` futur → 422 `REMINDER_DATE_IN_FUTURE`**).
21. **Tests export/backup** : `invoice_reminders` présente dans le zip export + le backup + round-trip import ; compteurs à jour.

### Doc & gate

22. **Doc** : `docs/migrations-idempotence-audit.md` (ligne migration `invoice_reminders` + colonnes suspension, P5). Manuel admin/user = **21-8** (pas ici). CHANGELOG entrée section `Added` (backend rappels — données/éligibilité).
23. **Gate local complet** (Test Locally First) : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (+ `--test-threads=1` si kesh-db touché — **il l'est**). E2E backend Rust suffisants ici (pas de frontend → 21-6).

## Tasks / Subtasks

- [x] **T1 — Migrations** (AC 1,2,3)
  - [x] `invoice_reminders.sql` (table append-only, FK CASCADE invoice, CHECK channel, index) + colonnes suspension dans le même fichier.
  - [x] Colonnes `invoices.dunning_paused_at` + `dunning_paused_note` (ADD COLUMN nullable).
  - [x] Ligne `docs/migrations-idempotence-audit.md` (P5) + stats 53→54, tracked-by-sqlx 42→43.
  - [x] Aucun bump `min_required` (CREATE TABLE + ADD COLUMN nullable = non-breaking). Compteurs migrations tests 53→54 + `invoice_reminders` dans fresh_install. `touch lib.rs` (MIGRATOR macro cache). Tests migrations 3/3 + 8/8 verts.
- [x] **T2 — Entité + repository `invoice_reminders`** (AC 4,5,6)
  - [x] Entité `invoice_reminder.rs` (enum `ReminderChannel` serde lowercase + FromStr, Decimal), `mod.rs`. Piège AC 3 : `dunning_paused_at`/`note` propagés dans `Invoice` + 3 SELECT (`FIND_INVOICE_SCOPED_SQL` + delete inline + list_all_by_company) + littéral test `invoice_email.rs:510`.
  - [x] Repo : `insert_in_tx`, `list_for_invoice`, `current_level(_in_tx)` (MAX non-annulés), `cancel_in_tx` (soft, idempotent, scopé). Audit délégué au caller (route) dans la même tx.
  - [x] Bornes frais via CHECK DB (aligné `dunning_levels`, pas de duplication).
  - [x] Tests repo `#[sqlx::test]` 3/3 (insert/current_level MAX, cancel soft exclut du MAX + idempotent, scoping company).
- [x] **T3 — Éligibilité** (AC 7,8,9) — module `repositories/dunning_eligibility.rs`
  - [x] `list_reminder_candidates(pool, company_id)` : SQL récupère les candidates (validated/impayée/non-suspendue/due_date non-NULL + niveau courant MAX non-annulé + dernier rappel + contact), Rust calcule niveau suivant + terminal + déclenchement (today UTC via chrono, cohérent UTC_DATE).
  - [x] Seed lazy `ensure_seeded_in_tx` déclenché à la 1re évaluation (tx courte).
  - [x] Garde C1 : `list_all_by_company.is_empty()` → liste vide. État terminal (`terminal`/`next_level=None`/`last_reminder_at`).
  - [x] Tests éligibilité 6/6 (niveau 1/2 dû, non-échue/suspendue/payée absentes, terminal, dunning désactivé C1 + seed lazy).
- [x] **T4 — Endpoints + RBAC** (AC 10-15) — module `routes/dunning_reminders.rs`
  - [x] `GET /api/v1/dunning/reminders` liste groupée par contact + `hasEmail` (Comptable+).
  - [x] `PUT .../dunning-pause` + `.../dunning-resume` (repo `set_dunning_pause` : FOR UPDATE + verrou optimiste + audit `invoice.dunning_paused/resumed`, resume nulle la note ; garde `notPaused`) (Comptable+).
  - [x] `POST .../reminders/manual` + gardes (validée/payée/niveau/sent_at futur) + audit `invoice.reminder_sent` (Comptable+).
  - [x] `POST .../reminders/{reminderId}/cancel` soft + audit `invoice.reminder_cancelled` (Admin).
  - [x] `GET .../reminders` historique (tous rôles).
  - [x] Montage RBAC lib.rs (Admin/Comptable/authenticated, méthodes disjointes). Nouvelles variantes AppError 422 (InvoiceAlreadyPaid/DunningLevelNotFound/ReminderDateInFuture/InvoiceNotPaused) ; `INVOICE_NOT_VALIDATED` réutilise la variante canonique existante (**400**, Story 5.3) — écart mineur vs spec 422, documenté.
  - [x] Tests E2E 5/5 (liste groupée + has_email, pause/resume + reset note + 422, rappel manuel saut niveau + gardes, annulation Admin-only + soft, RBAC Consultation 403 + IDOR 404).
- [ ] **T5 — Export souveraineté + backup** (AC 16,17,21)
  - [ ] Ajouter `invoice_reminders` à export global + `TABLES_TO_TRUNCATE` backup.
  - [ ] Bumper TOUS les compteurs figés (valeurs vérifiées au dev).
  - [ ] Tests export/backup round-trip (AC 21).
- [ ] **T6 — Doc + gate** (AC 22,23)
  - [ ] CHANGELOG `Added`. Gate local complet vert.

## Dev Notes

### Ground-truth consolidé (2026-07-15, 4 agents Explore)

**Schéma `invoices`** (`migrations/20260416000001_invoices.sql:15-39`) : `status VARCHAR(16) DEFAULT 'draft'` CHECK `IN ('draft','validated','cancelled')` (**pas de statut 'paid'** — le paiement est `paid_at DATETIME(3)` via `20260419000001_invoice_paid_at.sql:20`, index `(company_id,status,paid_at)` + `(company_id,status,due_date)`), `due_date DATE NULL`, `company_id`/`contact_id` FK `ON DELETE RESTRICT`. `emailed_at DATETIME(6)/emailed_to VARCHAR(320)` ajoutés par `20260709000002_invoices_emailed.sql:10-12` (**précédent exact** pour `dunning_paused_at`/note et pour le champ snapshot `sent_to`). Entité `entities/invoice.rs` : `status:String`, `due_date:Option<NaiveDate>`, `paid_at/emailed_at:Option<NaiveDateTime>`, `emailed_to:Option<String>`. Convention migration OBLIGATOIRE : `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci`, pas d'`IF NOT EXISTS` (tracked-by-sqlx).

**FK CASCADE — précédent unique** : `invoice_lines.invoice_id ON DELETE CASCADE` (`20260416000001:53`) est le **seul** enfant CASCADE de `invoices` ; `credit_notes.invoice_id` est RESTRICT. `invoice_reminders.invoice_id → CASCADE` suit `invoice_lines`. La suppression #219 (`repositories/invoices.rs:899-1019 fn delete`, `SELECT … FOR UPDATE` + gardes `paid_at`/`credit_notes`) purge alors les rappels ; l'audit_log **sans FK** (`entity_id` = pointeur logique, `20260413000001_audit_log.sql:18`) reste la trace résiduelle — exactement le modèle `emailed`/`invoice.emailed`.

**`emailed` = précédent de l'ordre « action réussie ⇒ enregistrer »** (`routes/invoice_email.rs:316-332`) : SMTP d'abord (`send_email().await?`), et SEULEMENT après succès `mark_emailed` (`repositories/invoices.rs:1568-1638` : `UPDATE emailed_at/to` + audit `for_actor` **dans une même tx**). Pas de table dédiée pour l'e-mail Epic 20 (le renvoi écrase) — c'est précisément la limite que `invoice_reminders` **append-only** dépasse (une ligne/envoi, snapshots, `cancelled_at` soft). En 21-5a le rappel **manuel** n'a pas de SMTP : simple `insert_in_tx` + audit dans la tx.

**`contacts.email`** (`20260414000001_contacts.sql:12` `VARCHAR(320) NULL`) : « sans e-mail » = NULL **ou** trimmé vide. Calquer le helper `locked_recipient` (`invoice_email.rs:73-80` : `email.as_deref().map(str::trim).filter(|e| !e.is_empty())`). Badge `has_email` de l'AC 10 = ce prédicat.

**`FailedProposal`** (`routes/reconciliation.rs:154-158`) : `{ bank_transaction_id: i64, error_code: String, details: Option<serde_json::Value> }` + retour `AcceptResponse { accepted, failed }`. Pattern pour le **lot 21-5b** (business id = `invoice_id`) — **pas nécessaire en 21-5a** (endpoints unitaires). Cité pour figer les `error_code` canoniques des gardes d'éligibilité (`INVOICE_ALREADY_PAID`, `DUNNING_PAUSED`, `INVOICE_NOT_FOUND`…).

**Audit** (`entities/audit_log.rs:136 fn user`, `:158 fn for_actor` ; pont kesh-api `audit.rs:24 NewAuditLogEntry::from_current_user(&CurrentUser, action, entity_type, entity_id, details)`) : dans les handlers avec `&CurrentUser` → `from_current_user` ; dans un repo threadé → `for_actor(user_id, api_key_id, …)`. Toujours `audit_log::insert_in_tx(&mut tx, …)` **dans la même tx** que la mutation. Modèle toggle+audit : `mark_as_paid` (`invoices.rs:1516-1537`, action `invoice.paid`/`invoice.unpaid`). Actions à créer : `invoice.reminder_sent`, `invoice.reminder_cancelled`, `invoice.dunning_paused`, `invoice.dunning_resumed`. `audit_log` **n'a pas** de `company_id` (le scope tenant vient de l'entité auditée).

**RBAC montage** (`lib.rs`) : 3 blocs `Router::new()....route_layer(...);` mergés dans `protected` sous `require_auth` (oignon `lib.rs:178-179`). Admin `:182-266` (`require_admin_role:264`), Comptable+ `:270-525` (`require_comptable_role:523`), tous-rôles-auth `:528-783` (pas de layer RBAC). **Pattern same-path méthodes disjointes** : DELETE invoice = Admin (`:230-233`) / PUT invoice = Comptable+ (`:324-327`) → à calquer (toggle+manuel Comptable+, annulation Admin). **Piège anti-footgun** (`:731-733,750,757,768,777`) : toute `.route(...)` AVANT le `.route_layer(...);` du bloc ; une route après le `;` échappe à `require_auth` → IDOR cross-tenant → **AC 15 teste ce point**. `CurrentUser { user_id, role, company_id, api_key_id }` via `Extension` (`middleware/auth.rs:37-45`), `company_id` du JWT jamais du client. Enum `Role { Admin>Comptable>Consultation }` (`entities/user.rs:18-27`).

**Précédent liste + export CSV restreint** : `list_due_dates_handler` (`routes/invoices.rs:872-911`, tous rôles, `tokio::join!` liste+summary, scope `current_user.company_id`, force `status='validated'`) + `export_due_dates_csv_handler` **Comptable+** (`lib.rs:396-399`, précédent anti-exfiltration **G2 B2**). La liste due-dates est **plate** — l'AC 10 « groupée par contact » est un **nouveau DTO** (réutiliser le scoping + l'agrégat SQL de `due_dates_summary` `invoices.rs:587-608`, pas la forme).

**Éligibilité SQL — délais depuis l'étape précédente** : `delay_days` est « depuis l'étape précédente » (entité `dunning_level.rs`), et les `level_number` sont **contigus** (garanti D5). Le calcul n'a donc **PAS besoin de somme cumulée** : seul le *prochain* niveau (courant+1) est évalué, à partir de la dernière date d'envoi.
- **Niveau 1 dû** (aucun rappel encore) : `UTC_DATE() >= due_date + INTERVAL grace_period_days DAY + INTERVAL delay_days(1) DAY`.
- **Niveau N>1 dû** : `UTC_DATE() >= DATE(sent_at du dernier rappel non-annulé) + INTERVAL delay_days(N) DAY` (N = courant+1).
- **`UTC_DATE()` et non `CURDATE()`** (H1) : cohérent avec le précédent `due_dates_summary` (`repositories/invoices.rs:601` utilise `UTC_DATE()`) — évite une divergence d'un jour autour de minuit si le serveur MariaDB n'est pas en UTC (l'échéancier/balance âgée 21-7 et l'éligibilité aux rappels doivent s'accorder).
- `grace_period_days` de `company_dunning_settings` (get-or-create/seed). Niveau courant = `COALESCE(MAX(r.level_number) WHERE r.cancelled_at IS NULL, 0)`. Prochain = plus petit `dunning_levels.level_number > courant`. Terminal si aucun `> courant` **et** ≥ 1 niveau configuré (sinon C1 → absente).
- *(Une window `SUM(delay_days) OVER (ORDER BY level_number)` serait utile seulement pour projeter les dates de TOUS les niveaux futurs depuis `due_date` en une passe — non requis ici, ne pas sur-complexifier la requête.)*

**Socle 21-3 réutilisable** : `dunning_levels` `fee_amount DECIMAL(7,2)` CHECK 0..10000, `version DEFAULT 0`, `count_for_company`/`max_level_in_tx`/`list_all_by_company`/`delete_and_renumber` (`repositories/dunning_levels.rs`). `company_dunning_settings` grâce défaut 5, `ensure_seeded_in_tx` (`company_dunning_settings.rs:168` : sentinel lock + seed 3 niveaux `[(10,"0.00"),(10,"20.00"),(10,"40.00")]`, ne réécrit jamais la grâce, no-op si `seeded_at` non-NULL). **L'AC 7 déclenche `ensure_seeded_in_tx` à la 1re évaluation d'éligibilité** (cohérent 21-3 « au premier accès config OU à la 1re évaluation »). `FEE_MAX=10000` + `validate_fee`/`scale_within` (`routes/dunning_levels.rs:22,76` + `routes/limits.rs:31`). Decimal : feature `serde-str` → JSON en **string** ; bind/FromRow natif DECIMAL ; parse via `"20.00".parse::<Decimal>()`.

### Patterns à réutiliser (ne PAS réinventer) — hérités 21-3

- **Sentinel lock** : `bank_accounts::acquire_company_sentinel_lock(tx, company_id)` (`repositories/bank_accounts.rs:588`, `SELECT id FROM companies WHERE id=? FOR UPDATE`) — si un invariant cross-row est en jeu. Pour un simple update de row invoice, préférer `SELECT ... FOR UPDATE` sur la row `invoices` ciblée.
- **Erreurs → HTTP** : `DbError::NotFound → 404`, `OptimisticLockConflict → 409` (`errors.rs`), cross-tenant → **404** (anti-énumération KF-002, jamais 403).
- **RBAC** : `middleware::rbac::require_admin_role`/`require_comptable_role` (`middleware/rbac.rs:31-40`), enum `Role` (`Consultation<Comptable<Admin`, `entities/user.rs:20-24`).
- **Decimal** : `rust_decimal::Decimal` + `#[serde(with="rust_decimal::serde::str")]` (STRING) — cohérent `dunning_levels.fee_amount` (21-3).
- **Seed lazy dunning settings** : réutiliser `company_dunning_settings` repo (21-3) — l'éligibilité déclenche le seed comme la page config.
- **Backup auto-vérifié** : `backup_inventory_matches_schema` (`backup.rs:581`) — filet contre l'oubli de table.
- **Export global** (non auto-vérifié — à ne PAS oublier) : `exports/global.rs` (push_csv + compteurs) — voir où 21-3 a ajouté `dunning_levels`/`company_dunning_settings`.

### Pièges de nommage & de commentaires (LOW passe 1)

- **Collision de nom** (L5) : la nouvelle struct `entities::invoice_reminder::InvoiceReminder` partage son identifiant avec le variant d'enum `EmailTemplateType::InvoiceReminder` (`email_template.rs:30`). Pas de collision de compilation (chemins distincts), MAIS éviter un `use EmailTemplateType::*` non qualifié dans un fichier qui manipule aussi la struct — qualifier explicitement.
- **Commentaire backup à rafraîchir** (L6) : `backup.rs:64` réserve la place « invoice_reminders → 21-5a » et sous-entend une future FK `invoice_reminders → dunning_levels` — **ne PAS créer cette FK** (les `fee_amount`/`level_number` sont des snapshots découplés, append-only, cf. AC 1). Mettre à jour ce commentaire à l'implémentation.

### Compteurs à bumper

Valeurs exactes listées en **AC 17** (ground-truth 4 agents). Rappel du réflexe : **revalider chaque valeur au dev** par `grep -nF` avant de l'incrémenter (le codebase bouge). `admin_backup_e2e.rs`/`admin_full_import_e2e.rs` n'ont **pas** de compteur dur (bornes `>=` + itération dynamique de `TABLES_TO_TRUNCATE`).

### Hors scope (garde-fous anti-creep)

- **AUCUN envoi e-mail**, `build_reminder_vars`, preview, envoi unitaire, **lot `{accepted,failed}`** → **21-5b**.
- **AUCUN frontend** (page Rappels, badges UI, compteur dashboard, historique fiche) → **21-6**.
- **AUCUNE balance âgée** (`aged_receivables`) → **21-7**.
- **AUCUN calcul `{totalDue}`/frais cumulés** dans un e-mail → 21-5b (mais le **snapshot `fee_amount`** à l'insert d'un rappel manuel est bien 21-5a).
- Pas de label de niveau custom (i18n frontend « Rappel N »/« Mise en demeure ») → 21-6.

### Règle de splitting (CLAUDE.md)

Story déjà **pré-splittée** de 21-5b (recommandation critique architecture — 3 findings HIGH tombaient dans l'ex-21-4 monolithique). Modules touchés : `kesh-db/{entities,repositories,migrations,backup}`, `kesh-api/{routes,exports,lib}` — ~5 modules, à la limite. Si `validate` dépasse 4 passes sans converger → envisager split 21-5a-i (table+repo+éligibilité) / 21-5a-ii (endpoints+RBAC+export). Suivre le plan par défaut.

### Project Structure Notes

- **Nouveaux fichiers** : `migrations/AAAAMMJJ000001_invoice_reminders.sql` (+ éventuellement `..._invoices_dunning_paused.sql`), `entities/invoice_reminder.rs`, `repositories/invoice_reminders.rs` (+ éligibilité, ou `dunning_eligibility.rs`), `routes/dunning_reminders.rs` (ou étendre un routes existant), tests repo + e2e.
- **Fichiers modifiés** : `entities/mod.rs`, `repositories/mod.rs`, `backup.rs` (TABLES_TO_TRUNCATE), `exports/global.rs` (+ `csv_tables.rs` ?), `lib.rs` (montage routes), `docs/migrations-idempotence-audit.md`, compteurs tests (`migrations_*`, `admin_*_e2e`, `exports_global_e2e`), `CHANGELOG.md`.

### References

- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md#B — items 8,9,10,11,12,13,16,19,22,26 ; #F item 28]
- [Source: _bmad-output/implementation-artifacts/21-3-socle-config-rappels.md#Dev Notes — patterns, sentinel lock, RBAC, audit, compteurs]
- [Source: crates/kesh-db/migrations/20260416000001_invoices.sql — schéma invoices (status CHECK, due_date, FK)]
- [Source: crates/kesh-db/migrations/20260709000002_invoices_emailed.sql — précédent ADD COLUMN emailed_at/to]
- [Source: crates/kesh-db/migrations/20260419000001_invoice_paid_at.sql — paid_at]
- [Source: crates/kesh-api/src/routes/reconciliation.rs:154 — FailedProposal]
- [Source: crates/kesh-db/src/entities/audit_log.rs:136,158 — for_actor/user]
- [Source: crates/kesh-api/src/lib.rs:182,264,523 — montage RBAC admin/comptable + piège Axum 0.8 route_layer]
- [Source: crates/kesh-db/src/repositories/{dunning_levels.rs,company_dunning_settings.rs} — socle 21-3 (bornes frais, grâce, seed lazy)]
- [Source: crates/kesh-db/src/backup.rs:579 — backup_inventory_matches_schema]
- [Source: CLAUDE.md — Migration breaking policy P1-P5, Pattern batch FailedProposal, Issue Tracking, Test Locally First]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

### Validate Pass 1 (2026-07-15, Sonnet 4.6) — 1 CRITICAL + 3 HIGH + 5 MEDIUM + 6 LOW, patchés

Passe adversariale grep ground-truth (toutes les refs `fichier:ligne` confrontées au code ; compteurs export/backup revérifiés — 18→19, 36→37 exacts). Remédiés :
- **C1 (CRITICAL)** — `dunning_levels` vide (dunning désactivé volontairement, D7, `seeded_at` non-NULL → seed lazy no-op) faisait apparaître **toute facture échue en état terminal** « poursuite à envisager ». **Patch** : garde AC 7 `count_for_company == 0 → liste vide` (ni candidate ni terminale) ; AC 8 distingue terminal (≥1 niveau, tous envoyés) de désactivé ; AC 19 + test dédié.
- **H1** — formules éligibilité en `CURDATE()` (fuseau serveur) incohérent avec le précédent `due_dates_summary` `invoices.rs:601` en `UTC_DATE()` → divergence d'un jour possible. **Patch** : `UTC_DATE()` dans les 2 formules.
- **H2** — AC 12 « niveau cohérent » sous-spécifié, contredisait D18 (le rappel **manuel** est le mécanisme du **saut de niveau vers le haut**). **Patch** : niveau accepté = `>=1` + existant en config, **sans borne supérieure** ; test saut de niveau (AC 20).
- **H3** — AC 3 sous-énumérait les sites SELECT à propager (« cause n°1 d'échec »). **Patch** : checklist fermée des **5 emplacements** `repositories/invoices.rs` (2 inlinés hors constante).
- **M1** — noms d'endpoints non figés → `GET /api/v1/dunning/reminders` + `PUT .../dunning-pause` / `.../dunning-resume` figés.
- **M2** — forme DTO liste non spécifiée → **imbriquée** `{ groups: Vec<ContactGroup{invoices[]}> }` figée (contrat consommé par 21-6).
- **M3** — subject/body du rappel manuel non tranché → `subject`=« Rappel manuel — niveau {N} », `body`=copie `note`.
- **M4** — `dunning-resume` ne nettoyait pas `dunning_paused_note` → reset à NULL avec `dunning_paused_at`.
- **M5** — Dev Notes mentionnaient une window `SUM() OVER` inutile (les formules simples suffisent) → clarifié « alternative non retenue ».
- **L1-L4** — refs de lignes corrigées (`backup.rs:581`, `csv_count` ~`:294`, `reconciliation.rs:154`, `migrations_fresh_install:40/43`). **L5** (collision nom `InvoiceReminder` struct vs enum) + **L6** (commentaire `backup.rs:64` pas de FK) documentés en Dev Notes.

**Trend** : Pass 1 → 1C/3H/5M (> LOW) → patchés. Points validés sans réserve : `mark_as_paid`, `ensure_seeded_in_tx`, montage RBAC, FK CASCADE `invoice_lines`, `locked_recipient`, politique breaking. **Split NON recommandé** (findings localisés, pas de complexité émergente). Relance Pass 2 (LLM différent, contexte frais).

### Validate Pass 2 (2026-07-15, Haiku 4.5, contexte frais) — 0 CRITICAL/HIGH, 2 MEDIUM + 4 LOW, patchés

Passe adversariale avec discipline grep ground-truth. **Les patches Pass 1 (C1/H1/H2/H3/M1-M5) confirmés corrects** (grep-vérifiés AC par AC). Nouveaux findings (robustesse / complétude de contrat, aucun structurel) remédiés :
- **M1-p2** — codes d'erreur HTTP non figés sur les gardes d'éligibilité. **Patch** : codes explicites en AC 11 (`OPTIMISTIC_LOCK_CONFLICT` 409, `INVOICE_NOT_PAUSED` 422, `VALIDATION_ERROR` 400) et AC 12 (`INVOICE_ALREADY_PAID`/`INVOICE_NOT_VALIDATED`/`DUNNING_LEVEL_NOT_FOUND` 422, `NOT_FOUND` 404).
- **M2-p2** — `sent_at` d'un rappel manuel non validé : une **date future gèlerait silencieusement le cycle** (`next niveau = UTC_DATE() >= sent_at+delay`). **Patch** : garde `sent_at <= NOW(6)` → 422 `REMINDER_DATE_IN_FUTURE` (AC 12 + test AC 20).
- **L3-p2** — `last_reminder_at` non défini → `MAX(sent_at)` des rappels non-annulés (AC 10).
- **L4-p2** — DTOs request/response non figés → request rappel manuel `{ levelNumber, sentAt, note? }`, pause `{ version, note? }`/reprise `{ version }`, response historique `Vec<ReminderResponse{…}>` `sent_at DESC` (AC 11/12/14).
- **L5-p2** — `ReminderChannel` : `#[serde(rename_all="lowercase")]` cohérent DB minuscule (AC 4).
- **L6-p2** — collision de nom `InvoiceReminder` (déjà documenté Pass 1, pas une régression).

Vérifications positives (Haiku, Read confirmé) : invariant anti-dissimulation (`due_dates_summary` ne filtre PAS `dunning_paused_at`), snapshots découplés sans FK, scope 21-5a isolé de 21-5b/21-6/21-7.

**Trend** : Pass 1 (1C/3H/5M) → Pass 2 (0C/0H/2M) → patchés. Relance Pass 3 (Opus, contexte frais). Split toujours non recommandé.

### Validate Pass 3 (2026-07-15, Opus 4.8, contexte frais) — 0 CRITICAL/HIGH, 1 MEDIUM + 2 LOW, patchés

Passe de scellage. **Tous les patches Pass 1/2 confirmés corrects** (grep ground-truth AC par AC : codes d'erreur AC 11/12 ↔ tests AC 20 cohérents, logique SQL éligibilité correcte sur tous les cas, invariant anti-dissimulation `due_dates_summary` sans filtre `dunning_paused_at`, complétude items epic 8-13/16/19/22/26/28 tous couverts ou différés). Défaut résiduel orthogonal :
- **M1-p3** — la checklist AC 3 (« 5 sites SELECT ») **sur-incluait 2 sites** (`list_by_company_paginated:507`, `list_for_export:1652`) qui désérialisent vers `struct InvoiceListItem` (sans `emailed_*`/`dunning_*`), **pas** vers `Invoice`. **Patch** (grep-vérifié : `emailed_to` dans les SELECT uniquement lignes 46/909/1707) : AC 3 réécrite avec les **3 vrais sites** `Invoice` (const `FIND_INVOICE_SCOPED_SQL` + inline `delete:907` + inline `list_all_by_company:1705`) + garde explicite « NE PAS ajouter `dunning_paused_*` à `InvoiceListItem` » (casserait les autres requêtes) + exposition liste factures différée 21-6.
- **L1-p3** — fragment éditorial redondant (reliquat de merge de patch M4) dans AC 11 → nettoyé.
- **L2-p3** — garde `sent_at` en `NOW(6)` (heure locale) incohérente avec la discipline UTC (H1) → `UTC_TIMESTAMP(6)` (AC 12) + pause `dunning_paused_at = UTC_TIMESTAMP(6)` (AC 11).

**Trend** : Pass 1 (1C/3H/5M) → Pass 2 (0C/0H/2M) → Pass 3 (0C/0H/1M) → patchés. Relance Pass 4 (Sonnet, contexte frais) pour sceller. Split non recommandé (finding unique localisé).

### Validate Pass 4 (2026-07-15, Sonnet 4.6, contexte frais) — CONVERGÉ (0 > LOW)

Passe de scellage final. **M1-p3 confirmé correct par grep** (`emailed_to` aux 3 sites 46/909/1707 ; `list_by_company_paginated`/`list_for_export` → `InvoiceListItem` sans champs emailed/dunning). Balayage exhaustif des refs factuelles (compteurs export `global.rs:278/279/294`, `admin_full_export:274` 36, `exports_global:619/695`, migrations 53/42, `invoice.rs` emailed:35/38, FK CASCADE `invoice_lines`, `backup.rs:64`, `mark_as_paid:1430`, `locked_recipient`, `due_dates_summary` `UTC_DATE():601`, `FailedProposal:154-158`, codes `errors.rs`) — **toutes exactes**. Précédent `PUT` pour toggle d'état confirmé (`disable_user`/`archive_*`). Aucune incohérence AC↔AC / AC↔Dev Notes / AC↔tests. Aucun défaut structurel résiduel.

**Trend final** : P1 (1C/3H/5M) → P2 (0C/0H/2M) → P3 (0C/0H/1M) → **P4 (0 > LOW) CONVERGÉ**. LLM : Sonnet→Haiku→Opus→Sonnet (rotation complète). **Split NON déclenché** : convergence atteinte AU seuil de 4 passes (critère « > 4 passes » non franchi), story ~5 modules tenable. **Spec scellée, prête pour `bmad-dev-story`.**
