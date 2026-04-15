# Story 5.4: Échéancier factures

Status: review

<!-- Validation optionnelle via validate-create-story avant dev-story. -->

## Story

As a **utilisateur (comptable ou indépendant)**,
I want **consulter un échéancier listant les factures validées, triées par date d'échéance, avec un indicateur de statut de paiement (payée / impayée / en retard) et pouvoir marquer manuellement une facture comme payée**,
so that **je puisse suivre mes créances clients, prioriser mes relances et anticiper ma trésorerie — en attendant la réconciliation bancaire automatique (Epic 6)**.

### Contexte

**Quatrième et dernière story de l'Epic 5 — Facturation QR Bill**. Créée en backlog le 2026-04-14 lors du démarrage de Story 5.2 (scope v0.6). Aucun FR explicite ne la couvre (la réconciliation automatique FR44 dépend d'Epic 6), mais elle est indispensable avant v1.0 : sans échéancier, l'utilisateur n'a aucune vue opérationnelle sur ses créances. Elle exploite la colonne `invoices.due_date` (ajoutée en 5.1, défaut backend `due_date = date` posé en 5.2) restée inutilisée jusqu'ici.

**Fondations déjà en place** (Stories 5.1, 5.2, 5.3 — NE PAS refaire) :

- **Table `invoices`** — migration `20260416000001_invoices.sql` + `20260417000001_invoice_validation.sql` + `20260417000002_invoice_validated_journal_entry_check.sql`. Colonnes pertinentes : `status VARCHAR(16) CHECK IN ('draft','validated','cancelled')`, `invoice_number VARCHAR(64) NULL`, `date DATE NOT NULL`, `due_date DATE NULL`, `total_amount DECIMAL(19,4)`, `contact_id BIGINT NOT NULL`, `journal_entry_id BIGINT NULL`, `version INT`. **Une facture validée a toujours `due_date` non-NULL** (défaut 5.2 `due_date = invoice.date` si non fourni) — vérifié côté handler `create_invoice`. Aucune colonne `paid_at` n'existe encore : à ajouter en 5.4.
- **Repository `invoices`** — `crates/kesh-db/src/repositories/invoices.rs`. `InvoiceListQuery`/`InvoiceListResult`/`InvoiceListItem` + `push_where_clauses` canonique (voir `repositories/invoices.rs:109-190`). `InvoiceSortBy` enum local. Patterns CRUD + audit atomique + rollback explicite. **À étendre** pour ajouter : filtres échéancier (`paid`, `overdue`, `due_before`), nouveaux sort_by (`DueDate`), nouveau handler `mark_as_paid` avec audit `invoice.paid` / `invoice.unpaid`.
- **Handler routes** — `crates/kesh-api/src/routes/invoices.rs`. `list_invoices_handler` accepte déjà des query params `status`, `contactId`, `dateFrom`, `dateTo`, `search`, tri (`sortBy`, `sortDirection`), pagination. `validate_invoice_handler` (5.2) et `invoice_pdf_handler` (5.3) sont les références de montage de nouveaux sous-endpoints (`POST /:id/validate`, `GET /:id/pdf`).
- **Frontend feature `invoices`** — `frontend/src/lib/features/invoices/` (api.ts, types.ts, helpers). Page liste `/invoices` + vue détail `/invoices/[id]` avec boutons « Valider » (5.2) et « Télécharger PDF » (5.3). **À étendre** : ajouter un onglet/page `/invoices/due-dates` (ou filtre dédié sur la liste existante ? — **décision : page dédiée**, voir Décisions de conception).
- **i18n FTL** — ~90 clés factures déjà présentes × 4 langues. Ajouter ~20 clés pour l'échéancier et le marquage de paiement.
- **Pattern `ListResponse<T>`** (`routes/mod.rs:25`), `notifySuccess/Error`, `i18nMsg`, `formatSwissAmount`, `formatSwissDate`, `onMount` pour URL init, cleanup debounce 300ms — **à réutiliser tels quels**.
- **Audit log** — wrapper `{before, after}` pour `update`, snapshot direct pour `create`/`delete`. `mark_as_paid` est sémantiquement un update ciblé (champ `paid_at`) → wrapper `{before, after}` obligatoire.
- **RBAC** : `authenticated_routes` pour GETs, `comptable_routes` pour le marquage paiement (cohérent avec validate/create/update).

### Scope verrouillé — ce qui DOIT être fait

1. **Migration `paid_at`** — ajouter `invoices.paid_at DATETIME(3) NULL` + index sur `(company_id, status, paid_at)` et sur `(company_id, status, due_date)` pour accélérer les requêtes de l'échéancier.
2. **Entité Rust** — ajouter `paid_at: Option<NaiveDateTime>` à `Invoice` et `InvoiceListItem`. Sérialisation `#[serde(rename_all = "camelCase")]` → `paidAt`.
3. **Repository** — 3 extensions :
   - `InvoiceListQuery` : ajouter `payment_status: Option<PaymentStatusFilter>` et `due_before: Option<NaiveDate>`.
   - `InvoiceSortBy::DueDate` variant.
   - `pub async fn mark_as_paid(pool, user_id, id, company_id, expected_version, paid_at: Option<NaiveDateTime>) -> Result<Invoice, DbError>` — set `paid_at = ?` (ou NULL pour dé-marquer) + version+1 + audit wrapper `{before, after}`. Refuse si `status != 'validated'` (`DbError::IllegalStateTransition`).
4. **Filtre dérivé `payment_status`** — non stocké en DB (dérivé). Enum API : `paid` (`paid_at IS NOT NULL`), `unpaid` (`paid_at IS NULL`), `overdue` (`paid_at IS NULL AND due_date < CURRENT_DATE`), `all` (pas de filtre). **Toujours combiné avec `status = 'validated'`** (une facture draft ou cancelled n'apparaît pas dans l'échéancier).
5. **API routes** — 2 endpoints :
   - `GET /api/v1/invoices/due-dates?paymentStatus=&dueBefore=&sortBy=&sortDirection=&limit=&offset=&search=&contactId=` (authenticated_routes) — identique à `GET /api/v1/invoices` sauf que `status` est forcé à `validated` côté handler (sécurité par défaut) et que le tri par défaut est `dueDate ASC`. Réponse inclut un champ dérivé `isOverdue: bool` par item (`paid_at IS NULL AND due_date < today`).
   - `POST /api/v1/invoices/:id/mark-paid` (comptable_routes) — body `{ paidAt?: string ISO date, version: number }`. Si `paidAt` omis → `paid_at = CURRENT_TIMESTAMP(3)`. Mapping erreurs : `IllegalStateTransition` → 409 (facture non validée), `OptimisticLockConflict` → 409, `NotFound` → 404.
   - `POST /api/v1/invoices/:id/unmark-paid` (comptable_routes) — body `{ version: number }`. Passe `paid_at` à NULL. Utile pour corriger une erreur de saisie. Mêmes mappings erreurs.
6. **Export CSV échéancier** — endpoint `GET /api/v1/invoices/due-dates/export.csv?paymentStatus=&dueBefore=...` (authenticated_routes). Réponse `text/csv; charset=utf-8`. Colonnes : `Numéro,Date,Date d'échéance,Client,Total,Statut paiement,Date paiement`. En-têtes i18n selon `companies.accounting_language` (même règle que description écriture comptable en 5.2). Formats suisses : apostrophe U+2019 séparateur milliers (1'234.56), dates dd.mm.yyyy. Limite `10_000` lignes max (au-delà → 400 `RESULT_TOO_LARGE` avec suggestion de raffiner les filtres). Pas de pagination dans le CSV — c'est un export complet.
7. **Page frontend `/invoices/due-dates`** — route SvelteKit dédiée. Tableau de factures validées avec colonnes : Numéro, Date facture, Date échéance, Client, Total, Statut paiement (badge), Paiement le (si payée). Filtres : `paymentStatus` (radio/segmented : Toutes / Impayées / En retard / Payées), `dueBefore` (datepicker), recherche texte (debounce 300ms), filtre contact (dropdown cherchable — réutiliser pattern `/invoices`). Tri cliquable sur Date, Date échéance, Client, Total. **Par défaut** : `paymentStatus=unpaid`, tri `dueDate ASC`. Lignes en retard surlignées (classe `row-overdue` → fond orange clair, cohérent avec design system tokens Story 1.9).
8. **Bouton « Marquer payée »** — sur chaque ligne de l'échéancier (si `unpaid`) et sur la page détail `/invoices/[id]` (vue lecture seule si `validated && !paid_at`). Dialog confirmation avec datepicker optionnel (valeur défaut = aujourd'hui) + champ commentaire (non persisté pour v0.1 — noté dans le dialog comme « fonctionnalité à venir » pour anticiper Epic 6). POST → reload.
9. **Bouton « Dé-marquer payée »** — sur la page détail uniquement (si `validated && paid_at IS NOT NULL`). Dialog confirmation « Cette facture sera à nouveau considérée comme impayée. Utile pour corriger une erreur. Continuer ? ». POST → reload. Pas sur la ligne échéancier (évite les mis-clics).
10. **Bouton « Exporter CSV »** en haut de la page échéancier — télécharge `/api/v1/invoices/due-dates/export.csv` avec les filtres courants passés en query string. Nom de fichier : `echeancier-{today}.csv` (ou `due-dates-{today}.csv` en EN).
11. **Entrée sidebar** — dans le groupe « Factures » (ou équivalent) : ajouter « Échéancier » pointant vers `/invoices/due-dates`. Réutiliser le pattern navGroups de `+layout.svelte`.
12. **i18n** — ~20 clés × 4 langues.
13. **Tests** — Rust DB (filtres, tri, `mark_as_paid`, concurrence version, CSV export), unit handlers, Vitest (formateurs spécifiques échéancier si nécessaire), Playwright (flow golden : créer facture → valider → voir dans échéancier → marquer payée → disparaît de « Impayées »).

### Scope volontairement HORS story — décisions tranchées

- **Réconciliation automatique** (matching avec transactions bancaires importées) → **Epic 6** (Import Bancaire). En 5.4, `paid_at` est **manuel uniquement**. Epic 6 posera `paid_at` automatiquement lors d'un match réussi.
- **Paiements partiels** — une facture est soit payée soit non-payée. Pas de montant partiel, pas de table `invoice_payments`. Reporté à Epic 10 (Avoirs & Paiements).
- **Relances automatiques** (emails, PDF de rappel) → hors v0.1, évalué post-MVP.
- **Statut « En contentieux » / « Abandonné »** → hors v0.1. Une facture reste « impayée » tant que non marquée payée. L'utilisateur gère le suivi hors Kesh pour v0.1.
- **Colonne `paid_amount DECIMAL`** — non introduite. Une facture est binaire payée/non-payée en v0.1. Introduire `paid_amount` nécessiterait de gérer les soldes restants, les devises, les trop-perçus — complexité qui appartient à Epic 10.
- **Écriture comptable à l'encaissement** (débit banque / crédit créance) → **Epic 6** (réconciliation). En 5.4, `mark_as_paid` ne crée PAS d'écriture comptable — c'est un simple marqueur de suivi. L'écriture d'encaissement sera générée lorsque la transaction bancaire est réconciliée (Epic 6). **Décision explicite** : ne pas anticiper cette écriture en 5.4 car elle requiert de choisir le compte banque (Asset) — information qui n'existe que via la réconciliation.
- **Notifications d'échéance proche** (ex. « 3 factures arrivent à échéance demain ») → hors v0.1.
- **Historique des changements de statut paiement** — couvert par l'audit log (`invoice.paid` / `invoice.unpaid` avec before/after). Pas de vue dédiée « historique » en 5.4.
- **Filtre multi-contact** — un seul `contactId` à la fois. Multi-sélection reportée.
- **Export PDF de l'échéancier** — CSV suffit pour v0.1 (transmission comptable). PDF en v0.2 si demandé.
- **Graphique trésorerie prévisionnelle** — nice-to-have, reporté (Epic 7 ou plus tard).

### Décisions de conception

- **`paid_at DATETIME(3) NULL`** (pas `DATE`) — horodate pour traçabilité fine. L'UI affiche uniquement la date (pas l'heure), mais le stockage fin facilite le tri et évite les collisions « plusieurs factures payées le même jour à la même seconde » lors d'imports batch futurs. Cohérent avec `created_at`/`updated_at`. **Fuseau horaire** : UTC naïf (`NaiveDateTime`) — convention projet (même que les autres horodates).

- **Pas de colonne `paid` BOOL dérivée / générée** — une colonne `paid_at IS NOT NULL` est binaire en sémantique. Une `paid BOOL` redondante introduirait un risque de désynchronisation. `payment_status` est dérivé **à la query** (SQL `CASE` ou WHERE appropriés).

- **Tri par défaut de l'échéancier = `due_date ASC`** (les plus urgentes en tête). Secondaire = `id ASC` (stabilité). Contrairement à la liste `/invoices` (tri par `date DESC`).

- **Route séparée `/invoices/due-dates` vs filtre sur `/invoices`** — page dédiée. **Raisons** : (a) cas d'usage distinct — l'échéancier est un outil opérationnel (relances, trésorerie), tandis que `/invoices` est un outil d'archive (consultation historique) ; (b) pré-sélection de filtres (statut validé + par défaut impayées + tri par échéance) qui seraient lourds à mémoriser par URL ; (c) plus tard on pourra enrichir avec des agrégats (« 12'450 CHF impayés, dont 3'200 CHF en retard ») qui n'ont pas leur place sur `/invoices`. L'entrée de sidebar distincte améliore la découvrabilité.

- **Agrégat résumé en haut de page** — afficher « X factures impayées, total CHF XX'XXX.XX, dont Y en retard (CHF ZZ'ZZZ.ZZ) ». Calculé côté backend et inclus dans la réponse du GET liste (champ `summary: { unpaidCount, unpaidTotal, overdueCount, overdueTotal }`). Un seul round-trip.
  - **Périmètre du summary** : calculé sur les factures filtrées par `contact_id`, `search`, `date_from`, `date_to` **mais le filtre `paymentStatus` est volontairement IGNORÉ**. Le summary affiche toujours les totaux d'impayées (unpaid + overdue), quel que soit le filtre `paymentStatus` actif côté UI. **Raison** : afficher un summary vide quand l'utilisateur bascule sur « Payées » n'aurait aucune valeur métier — le summary est un KPI opérationnel (créances en attente), pas un résumé du tableau affiché. Ce comportement est intentionnel et testé (T2.5 `test_due_dates_summary_ignores_payment_status_filter`).

- **`isOverdue` calculé côté backend** — le handler enrichit chaque `InvoiceListItem` avec `isOverdue: bool`. **Raison** : single source of truth pour « aujourd'hui » (évite les désynchros entre fuseau serveur et navigateur). Calcul : `status == 'validated' && paid_at IS NULL && due_date < CURRENT_DATE`. Le frontend utilise le flag directement pour le surlignage.

- **`mark_as_paid` avec version obligatoire** — verrou optimiste classique (comme `update`). Si le frontend a chargé une ancienne version, il reçoit 409 et doit recharger. **Raison** : éviter une race entre deux utilisateurs comptables qui marquent la même facture en même temps avec des dates différentes.

- **Refus `mark_as_paid` sur facture non validée** — `DbError::IllegalStateTransition` → 409. **Raison** : une facture draft n'a pas d'existence comptable → marquer payée n'a aucun sens. Une facture cancelled (future Epic 10) ne doit pas non plus être marquable.

- **Audit log `invoice.paid` / `invoice.unpaid`** — événements dédiés (pas `invoice.updated` générique). **Raison** : traçabilité métier spécifique. Snapshot wrapper `{before: {paidAt: null}, after: {paidAt: "2026-04-20T10:00:00"}}`. Le détail reste interrogeable via `audit_log::find_by_entity("invoice", id, ...)`.

- **Export CSV limite 10_000 lignes** — protection mémoire. Au-delà, erreur 400 `RESULT_TOO_LARGE` avec message i18n « Trop de résultats (> 10 000). Veuillez affiner vos filtres. ». Build en mémoire (`Vec<u8>`) — ~2 Mo max, pas de streaming nécessaire. Réponse `axum::body::Body::from(bytes)`.

- **Encodage CSV** — UTF-8 avec **BOM** (`0xEF 0xBB 0xBF`) en tête. **Raison** : Excel sous Windows interprète correctement les accents (é, à, etc.) avec BOM. Sans BOM, Excel ouvre en ISO-8859-1 par défaut et casse les caractères non-ASCII. Pattern compatible avec LibreOffice et Numbers. Séparateur **point-virgule** `;` (standard suisse/francophone Excel). Guillemets doubles `"` pour les champs contenant `;` ou `"` (avec échappement `""`). Sauts de ligne CRLF (standard CSV RFC 4180).

- **Index composites pour perf** — `(company_id, status, paid_at)` pour filtrer rapidement impayées ; `(company_id, status, due_date)` pour tri par échéance + `WHERE due_date < today` (overdue). Vérifier via `EXPLAIN` en test d'intégration qu'on tape bien l'index (optionnel mais utile — voir T6.3).

- **Contrainte de cohérence** — CHECK `paid_at IS NULL OR status = 'validated'` au niveau DB. **Raison** : empêche un bug applicatif d'introduire un état incohérent (facture draft avec paid_at set). Redondant avec la vérification handler/repository mais défense en profondeur.

- **UI : badge statut paiement** — composant réutilisable `<PaymentStatusBadge :status="paid|unpaid|overdue" />`. Couleurs depuis les design tokens (vert = payée, gris = impayée, orange = en retard). Accessibilité : `aria-label` explicite, pas juste couleur (contraste AA).

- **Datepicker `paidAt` dans le dialog « Marquer payée »** — input `type="date"` (natif, localisé via `lang` attr). Valeur défaut = aujourd'hui. Validation : pas dans le futur (`<= today`), pas antérieure à `invoice.date` (sinon payée avant d'avoir été émise — impossible). Validation côté backend aussi (règle identique) → 400 `INVALID_INPUT` avec message ciblé.

- **`unmark-paid` strictement sur page détail** — **pas** sur la ligne de tableau de l'échéancier. **Raison** : action rare, destructive (perte d'info), doit être délibérée. Cohérent avec le pattern « suppression = dialog confirmation depuis la vue détail ».

- **Pas de notification email au client** quand une facture passe `paid` — hors scope (pas d'infra email en v0.1 de toute façon).

- **Mono-tenant v0.1** — `get_company(&state)` pour résoudre `company_id`, comme `validate_invoice_handler` / `invoice_pdf_handler` (5.2/5.3). Pas de multi-tenancy en 5.4.

### Dette technique documentée — v0.2

- **BS1 — Fuseau horaire société pour `isOverdue` et filtre `Overdue`** (sévérité MEDIUM, reclassée dette technique au review pass 1 de la Story 5.4).
  - **Constat** : le calcul d'« overdue » utilise `UTC_DATE()` côté SQL et `NaiveDateTime` UTC naïf côté Rust (convention projet §66). Pour un utilisateur en CET/CEST, le basculement d'une facture en « en retard » se produit à minuit UTC (01:00 ou 02:00 heure locale), pas à minuit local. Le `summary.overdueCount` peut ainsi changer selon l'heure de la journée où la page est consultée, et une facture due le 2026-04-14 apparaît encore à jour pendant ~1-2h le 2026-04-15 matin heure suisse.
  - **Décision v0.1** : accepté comme trade-off. Kesh est mono-société v0.1 avec un utilisateur suisse unique ; l'impact est limité à une fenêtre de 1-2h par jour et ne produit jamais de faux positif critique (ni comptable, ni légal).
  - **Remédiation v0.2** : introduire `companies.timezone` (défaut `Europe/Zurich`), passer la « date de référence » depuis le handler avec conversion TZ-aware, et remplacer `UTC_DATE()` par un bind paramétrique. Propriétaire : Guy. Story de remédiation : à créer lors du scope v0.2 (tracée dans backlog Epic 13 ou Epic 9 selon priorité).
  - **Mitigation en 5.4** : comparaison `paid_at.date() < invoice.date - 1 jour` dans `mark_as_paid` (tolérance 1 jour, P2 review pass 1) + CHECK DB correspondante (P4) pour que les paiements saisis en tout début de journée heure suisse ne soient pas rejetés.

### Concurrence et ordre des locks

- `POST /:id/mark-paid` : simple UPDATE avec verrou optimiste (`version`). Pas de FOR UPDATE explicite — la logique est ponctuelle et ne touche qu'une seule ligne `invoices`. Pas d'interaction avec `journal_entries` (pas d'écriture comptable en 5.4).
- Deux `mark-paid` simultanés sur la même facture → un seul réussit (version+1), le second reçoit 409 `OPTIMISTIC_LOCK_CONFLICT`.
- Audit log atomique dans la même transaction (pattern existant). Sur échec audit → `tx.rollback()` explicite.

## Acceptance Criteria

1. **Given** une facture validée (`status = 'validated'`) avec `due_date` dépassée et `paid_at IS NULL`, **When** un utilisateur consulte `/invoices/due-dates`, **Then** la facture apparaît avec un badge « En retard » et la ligne est surlignée visuellement.
2. **Given** une facture draft ou cancelled, **When** l'utilisateur consulte l'échéancier, **Then** la facture n'apparaît pas (filtrage `status = 'validated'` strict côté backend).
3. **Given** l'échéancier chargé par défaut, **When** la page s'affiche, **Then** le filtre `paymentStatus=unpaid` est actif, le tri est `dueDate ASC`, et les factures en retard sont en tête.
4. **Given** un utilisateur comptable consulte une facture validée non payée, **When** il clique « Marquer payée » et confirme (date par défaut = aujourd'hui), **Then** la facture passe `paid_at = NOW()`, un audit log `invoice.paid` est créé, et la facture disparaît du filtre « Impayées » à l'actualisation.
5. **Given** une facture draft ou cancelled, **When** tentative d'appel `POST /:id/mark-paid`, **Then** 409 `ILLEGAL_STATE_TRANSITION` avec message i18n.
6. **Given** deux utilisateurs comptables tentent de marquer la même facture payée en parallèle, **When** les deux requêtes partent avec la même `version`, **Then** la première réussit, la seconde reçoit 409 `OPTIMISTIC_LOCK_CONFLICT`.
7. **Given** une facture payée par erreur, **When** l'utilisateur clique « Dé-marquer payée » sur la vue détail et confirme, **Then** `paid_at` redevient NULL, audit `invoice.unpaid` créé, facture réapparaît dans « Impayées ».
8. **Given** l'utilisateur saisit une date de paiement antérieure à `invoice.date - 1 jour`, **When** soumission, **Then** 400 `INVALID_INPUT` avec message i18n précis (« La date de paiement ne peut être antérieure à la date de facture »). **Note (amendée 2026-04-15, code review pass 3 B)** : la borne supérieure « pas dans le futur » du critère initial a été supprimée. `paid_at` représente la date d'exécution bancaire effective, qui peut légitimement être dans le futur (ordre de virement programmé, décalage week-end/jour férié réécrit par la banque).
9. **Given** un échéancier filtré sur « Impayées » avec 15 factures, **When** l'utilisateur clique « Exporter CSV », **Then** un fichier `echeancier-YYYY-MM-DD.csv` est téléchargé avec BOM UTF-8, séparateur `;`, encodage correct des accents, montants au format suisse (1'234.56), dates dd.mm.yyyy, une ligne d'en-tête traduite selon `companies.accounting_language`.
10. **Given** plus de 10'000 factures correspondant aux filtres, **When** tentative d'export, **Then** 400 `RESULT_TOO_LARGE` avec message invitant à raffiner les filtres.
11. **Given** la page échéancier, **When** affichage de l'entête, **Then** un résumé « X factures impayées, total CHF XX'XXX.XX, dont Y en retard (CHF ZZ'ZZZ.ZZ) » est affiché, calculé sur les factures filtrées par contact/search/dates mais **indépendamment du filtre `paymentStatus`** (le summary reflète toujours les impayées, voir Décisions de conception).
12. **Given** les 4 langues UI (fr-CH, de-CH, it-CH, en-CH), **When** navigation sur l'échéancier, **Then** tous les textes (titres, filtres, boutons, dialogs, badges, messages d'erreur, en-têtes CSV) sont traduits — aucune clé manquante (`npm run test:unit` i18n audit vert).
13. **Given** l'endpoint `GET /api/v1/invoices/due-dates`, **When** requête anonyme ou avec token invalide, **Then** 401 (authenticated_routes).
14. **Given** un utilisateur rôle « Admin » (non comptable), **When** tentative de POST mark-paid, **Then** 403 `FORBIDDEN` (pattern comptable_routes, à confirmer — vérifier si admin hérite ou non). Ajuster la couche `comptable_routes` si admin doit être autorisé (décision : **admin autorisé** car admin ⊇ comptable dans le projet).
15. **Given** une facture validée avec `paid_at` défini, **When** affichage sur la vue détail `/invoices/[id]`, **Then** la date de paiement est visible, un bouton « Dé-marquer payée » est présent, et le badge « Payée » est affiché.
16. **Given** un test Playwright `invoices_echeancier.spec.ts`, **When** exécution du flow golden (créer → valider → voir dans échéancier → marquer payée → vérifier disparition), **Then** toutes les assertions passent (création, validation, navigation, dialog, reload, filtrage).
17. **Given** `cargo test --workspace -- --test-threads=1` et `npm run check` + `npm run test:unit -- --run`, **When** exécution, **Then** tous les tests passent, aucune régression introduite sur Stories 5.1/5.2/5.3.
18. **Given** la migration `paid_at`, **When** rollback puis re-run, **Then** aucune perte de données (la colonne est additive, les CHECK sont additifs). Migration vérifiée idempotente implicite (sqlx migrate suit le tracking standard).
19. **Given** un audit log `invoice.paid`, **When** consultation via `audit_log::find_by_entity("invoice", id, ...)`, **Then** l'entrée contient le wrapper `{before: {paidAt: null, ...}, after: {paidAt: "...", ...}}` avec le snapshot complet de la facture.
20. **Given** les index composites créés, **When** requête de l'échéancier sur une base de test avec 10'000 factures, **Then** `EXPLAIN` montre utilisation des index `(company_id, status, paid_at)` et/ou `(company_id, status, due_date)` (test d'intégration optionnel mais recommandé).

## Tasks / Subtasks

### T1 — Migration + entité (AC: #1, #2, #15, #18, #20)

- [ ] T1.1 Créer `crates/kesh-db/migrations/20260419000001_invoice_paid_at.sql` :
  - `ALTER TABLE invoices ADD COLUMN paid_at DATETIME(3) NULL;`
  - `ALTER TABLE invoices ADD CONSTRAINT chk_invoices_paid_at_validated CHECK (paid_at IS NULL OR status = 'validated');` — **enforced uniquement sur MariaDB ≥ 10.2** (MariaDB 10.1 et antérieur parsent mais ignorent les CHECK). Le projet Kesh cible MariaDB 10.11+ (vérifier `docker-compose.dev.yml` et architecture — fait 2026-04-15 : compatible). Défense en profondeur double via le guard `status = 'validated'` dans `mark_as_paid` (T2.4) — la CHECK est un filet de sécurité, pas l'unique barrière.
  - `CREATE INDEX idx_invoices_payment_status ON invoices (company_id, status, paid_at);`
  - `CREATE INDEX idx_invoices_due_date ON invoices (company_id, status, due_date);`
  - Vérifier numéro de migration via `ls crates/kesh-db/migrations/` avant création — ajuster si hotfix intercurrente.
- [ ] T1.2 Étendre `crates/kesh-db/src/entities/invoice.rs` :
  - Ajouter `paid_at: Option<NaiveDateTime>` à la struct `Invoice` + `#[serde(rename_all = "camelCase")]` (déjà appliqué à la struct).
  - Ajouter `paid_at` à `InvoiceListItem`.
- [ ] T1.2bis Étendre `crates/kesh-db/src/repositories/invoices.rs` :
  - Mettre à jour la constante `FIND_INVOICE_SCOPED_SQL` (ligne ~36) : ajouter `paid_at` dans la liste des colonnes du `SELECT` (sinon `sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)` panique au runtime avec `ColumnNotFound`).
  - Mettre à jour `invoice_snapshot_json` (ligne ~50) : ajouter `"paidAt": inv.paid_at.map(|dt| dt.to_string())` (ou équivalent `serde_json::Value`) — indispensable pour AC#19 (le snapshot audit doit contenir `paidAt` pour que `{before, after}` soit utilisable).
  - **IMPORTANT** : il n'existe AUCUNE constante `INVOICE_COLUMNS` dans le repo (vérifié 2026-04-15) — ne pas en créer. La source de vérité SQL est `FIND_INVOICE_SCOPED_SQL` et les chaînes inline dans `list`/`insert_*`. Parcourir tout le fichier `invoices.rs` pour repérer chaque `SELECT ... FROM invoices` inline et y ajouter `paid_at` (y compris `list` via `QueryBuilder`, les 2 `UPDATE ... WHERE id = ? AND version = ?` pour le returning via `FIND_INVOICE_SCOPED_SQL` post-update, etc.).

### T2 — Repository (AC: #4, #5, #6, #7, #8, #11, #19)

- [ ] T2.1 Étendre `InvoiceListQuery` dans `repositories/invoices.rs` :
  - Ajouter `pub payment_status: Option<PaymentStatusFilter>` avec `pub enum PaymentStatusFilter { Paid, Unpaid, Overdue, All }` (Default = All).
  - Ajouter `pub due_before: Option<NaiveDate>` (filtre supplémentaire).
  - Ajouter variant `InvoiceSortBy::DueDate`.
- [ ] T2.2 Étendre `push_where_clauses` pour gérer `payment_status` :
  - `Paid` → `AND paid_at IS NOT NULL`.
  - `Unpaid` → `AND paid_at IS NULL`.
  - `Overdue` → `AND paid_at IS NULL AND due_date < UTC_DATE()`. Utiliser `UTC_DATE()` MariaDB natif (et **non** `CURDATE()` qui dépend de la TZ de la session SQL). **Convention projet : tout est en UTC naïf** (mémo `feedback_sqlx_mysql_gotchas` + convention `NaiveDateTime`). `UTC_DATE()` garantit la cohérence quelle que soit la TZ du serveur/du container MariaDB. Même règle appliquée dans `due_dates_summary` (T2.3) et dans le calcul `is_overdue` côté handler (T3.1).
  - `All` → pas de clause.
- [ ] T2.3 Ajouter `pub async fn due_dates_summary(pool, company_id, query: &InvoiceListQuery) -> Result<DueDatesSummary, DbError>` :
  - `DueDatesSummary { unpaid_count: i64, unpaid_total: Decimal, overdue_count: i64, overdue_total: Decimal }`.
  - 1 requête SQL `SELECT COUNT(*), COALESCE(SUM(total_amount), 0), SUM(CASE WHEN paid_at IS NULL AND due_date < UTC_DATE() THEN 1 ELSE 0 END), SUM(CASE WHEN paid_at IS NULL AND due_date < UTC_DATE() THEN total_amount ELSE 0 END) FROM invoices WHERE company_id = ? AND status = 'validated' AND paid_at IS NULL [+ filtres contact/search/date_from/date_to]`.
  - **Le `payment_status` du query est ignoré** pour le summary (toujours calculé sur les impayées) — c'est le sens métier du résumé.
- [ ] T2.4 Ajouter `pub async fn mark_as_paid(pool, user_id, id, company_id, expected_version, paid_at: Option<NaiveDateTime>) -> Result<Invoice, DbError>` :
  - Transaction : SELECT FOR UPDATE facture → vérifier `status = 'validated'` (sinon `IllegalStateTransition`) → UPDATE `paid_at`, `version+1` WHERE `version = expected_version` (rows=0 → `OptimisticLockConflict`) → audit wrapper `{before, after}` avec action `invoice.paid` (si `paid_at.is_some()`) ou `invoice.unpaid` (si `None`) → commit.
  - Rollback explicite si audit échoue.
- [ ] T2.5 Tests unitaires (`invoices::tests`) :
  - `test_mark_as_paid_nominal` + `test_mark_as_paid_rejects_draft` + `test_mark_as_paid_rejects_cancelled`.
  - `test_mark_as_paid_optimistic_lock`.
  - `test_unmark_paid` (passer `None`) + vérifier action audit = `invoice.unpaid`.
  - `test_list_filter_overdue` (fixture : 3 factures dont 1 passée + impayée, 1 passée + payée, 1 future + impayée → seul overdue=true retourne 1 item).
  - `test_due_dates_summary_computes_correct_totals`.
  - `test_mark_as_paid_concurrent_one_succeeds_other_409`. **Config pool obligatoire** : `PoolOptions::new().max_connections(4)` (au moins 2 — la 1re tx garde sa connexion jusqu'au COMMIT, le 2e `pool.begin()` doit pouvoir prendre une autre connexion ou `tokio::join!` deadlock). Pattern identique à Story 5.2 `test_concurrent_calls_serialize_via_for_update` — copier le helper `setup_pool_for_concurrency_test()` si déjà présent, sinon le créer. Compatible avec CI `--test-threads=1` (concurrence intra-test Tokio, pas inter-test). Voir `feedback_sqlx_mysql_gotchas` (mémoire projet) pour les pièges cross-binary SQLx.
  - `test_due_dates_summary_ignores_payment_status_filter` — fixture : 2 factures impayées + 1 payée, appeler `due_dates_summary` avec `payment_status = Some(Paid)` → asserter `unpaid_count == 2` (le filtre est bien ignoré côté summary).
  - `test_mark_as_paid_atomic_rollback_on_audit_failure`.

### T3 — API routes (AC: #4, #5, #6, #9, #10, #13, #14)

- [ ] T3.1 Étendre `crates/kesh-api/src/routes/invoices.rs` :
  - Handler `list_due_dates_handler` : parse query params (`paymentStatus`, `dueBefore`, + réutilise ceux de `list_invoices`) → force `status = 'validated'` → appelle `invoices::list` + `invoices::due_dates_summary` en parallèle (`tokio::join!` OK, read-only) → enrichit chaque item avec `is_overdue: bool` → renvoie `ListResponse<InvoiceListItemWithOverdue> + summary`.
  - DTO réponse : soit ré-emploi de `ListResponse<T>` avec méta dans un wrapper `{ items, total, offset, limit, summary: DueDatesSummary }`, soit wrapper dédié `DueDatesResponse`. **Décision** : wrapper dédié `DueDatesResponse { items: Vec<InvoiceListItemWithOverdue>, total: i64, offset: i64, limit: i64, summary: DueDatesSummary }`.
- [ ] T3.2 Handler `mark_invoice_paid_handler` :
  - `POST /api/v1/invoices/:id/mark-paid`. Body `{ paidAt?: String (ISO 8601 datetime), version: i32 }`.
  - Parse `paidAt` → `NaiveDateTime` (défaut `Utc::now().naive_utc()` si absent).
  - Validation : `paidAt <= NOW()` ET `paidAt.date() >= invoice.date` (**comparaison calendaire pure** : `paid_at` est `NaiveDateTime`, `invoice.date` est `NaiveDate` — convertir via `.date()` pour éviter qu'un paiement à 00:30 UTC soit rejeté comme « antérieur » à une facture du même jour). Validation DB dans la transaction pour atomicité (ou comparaison post-SELECT FOR UPDATE dans `mark_as_paid`).
  - Mapping erreurs : `IllegalStateTransition` → 409 `ILLEGAL_STATE_TRANSITION`, `OptimisticLockConflict` → 409, `NotFound` → 404, `InvalidInput` → 400.
- [ ] T3.3 Handler `unmark_invoice_paid_handler` :
  - `POST /api/v1/invoices/:id/unmark-paid`. Body `{ version: i32 }`.
  - Délègue à `mark_as_paid` avec `paid_at = None`.
- [ ] T3.4 Handler export CSV `export_due_dates_csv_handler` :
  - `GET /api/v1/invoices/due-dates/export.csv?...`. Même parsing que list_due_dates.
  - Vérifier `total > 10_000` → 400 `RESULT_TOO_LARGE` avec message i18n.
  - Charger toutes les lignes filtrées (via `invoices::list_all_for_export` — nouvelle fn helper qui ne pagine pas mais a la limite dure ≤ 10_000).
  - **Pas de streaming** : 10'000 lignes × ~200 octets par ligne = ~2 Mo max en mémoire, parfaitement tenable. Construire le CSV entier dans un `Vec<u8>` via `csv::Writer`, puis renvoyer `axum::body::Body::from(bytes)`. Streaming = complexité inutile pour le volume max autorisé.
  - Construire le CSV avec `csv::Writer`. **Ajouter `csv = "1"` dans `[dependencies]` de `crates/kesh-api/Cargo.toml`** — la crate est présente dans `Cargo.lock` comme dépendance transitive uniquement (vérifié 2026-04-15), elle n'est PAS dans les dépendances directes de `kesh-api`. Ne pas se fier au lock.
  - Config : `csv::WriterBuilder::new().delimiter(b';').terminator(csv::Terminator::CRLF).from_writer(buf)`. Séparateur `;`, quoting `"`, lineterminator CRLF. **Écrire le BOM `\u{FEFF}` manuellement dans le buffer AVANT de passer au Writer** — la crate `csv` ne gère pas le BOM nativement.
  - En-têtes i18n selon `companies.accounting_language` (clés `echeancier-csv-header-*`).
  - Réponse `Content-Type: text/csv; charset=utf-8` + `Content-Disposition: attachment; filename="echeancier-{YYYY-MM-DD}.csv"`.
- [ ] T3.5 Enregistrer routes dans `crates/kesh-api/src/lib.rs` :
  - `GET /invoices/due-dates` et `GET /invoices/due-dates/export.csv` dans `authenticated_routes`.
  - `POST /invoices/:id/mark-paid` et `/unmark-paid` dans `comptable_routes`.
  - **Pas de contrainte d'ordre de déclaration** : Axum 0.8 (router `matchit 0.8+`) donne automatiquement la priorité aux segments statiques (`due-dates`) sur les segments dynamiques (`{id}`), quel que soit l'ordre des `.route(...)`. Déclarer la route sur le bon sous-routeur (authenticated pour GET, comptable pour POST mark/unmark) suffit — inutile de forcer un ordre.
- [ ] T3.6 Tests d'intégration (`tests/invoice_echeancier_e2e.rs` ou équivalent) :
  - `test_list_due_dates_default_returns_only_unpaid_validated`.
  - `test_mark_paid_forbidden_for_draft_invoice`.
  - `test_mark_paid_rejects_paid_at_in_future`.
  - `test_mark_paid_rejects_paid_at_before_invoice_date`.
  - `test_export_csv_over_limit_returns_400`.
  - `test_export_csv_format_has_bom_and_swiss_amounts`.
  - `test_summary_matches_filtered_list`.

### T4 — Frontend : page échéancier + marquage (AC: #1, #3, #4, #7, #11, #12, #15, #16)

- [ ] T4.1 Étendre `frontend/src/lib/features/invoices/invoices.types.ts` :
  - Ajouter `paidAt: string | null` à `Invoice` et `InvoiceListItem`.
  - Ajouter `isOverdue: boolean` à `DueDateItem` (nouveau type).
  - Ajouter types `PaymentStatusFilter = 'all' | 'paid' | 'unpaid' | 'overdue'`.
  - Ajouter types `DueDatesResponse`, `DueDatesSummary`.
- [ ] T4.2 Étendre `invoices.api.ts` :
  - `listDueDates(query: DueDatesQuery): Promise<DueDatesResponse>`.
  - `markInvoicePaid(id: number, paidAt: string | undefined, version: number): Promise<Invoice>`.
  - `unmarkInvoicePaid(id: number, version: number): Promise<Invoice>`.
  - `exportDueDatesCsv(query: DueDatesQuery): Promise<Blob>` — **contrat retenu** : le backend renvoie le CSV complet en body (pas d'URL signée, pas de redirect). Côté frontend, utiliser le pattern canonique :
    ```ts
    const blob = await invoicesApi.exportDueDatesCsv(query);
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = `echeancier-${todayIso}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    ```
    Pas d'approche `<a href={backendUrl} download>` direct : nécessiterait de passer le JWT en query string (fuite dans les logs) ou d'utiliser des cookies (non utilisés par le projet — wrapper fetch).
- [ ] T4.3 Créer `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` :
  - Onglets/segmented filter `paymentStatus` (Toutes / Impayées / En retard / Payées).
  - Tableau avec colonnes listées au scope (#7).
  - Résumé agrégat en haut (summary).
  - Bouton « Marquer payée » par ligne (unpaid/overdue only).
  - Bouton « Exporter CSV » en haut.
  - onMount : parse URL params → default `paymentStatus=unpaid`, `sortBy=dueDate`, `sortDirection=asc`.
  - Debounce 300ms sur search.
  - Surlignage ligne overdue (classe CSS `row-overdue`).
- [ ] T4.4 Créer composant `<PaymentStatusBadge>` dans `frontend/src/lib/features/invoices/PaymentStatusBadge.svelte` :
  - Props : `status: 'paid' | 'unpaid' | 'overdue'`.
  - Réutilise tokens design system (couleurs, tailles).
- [ ] T4.5 Créer dialog « Marquer payée » dans `frontend/src/lib/features/invoices/MarkPaidDialog.svelte` :
  - Input date (type="date") valeur défaut = today.
  - Validation client : `paidAt <= today && paidAt >= invoice.date`.
  - Bouton Annuler / Confirmer.
  - Gestion 409 (modale reload) / 400 (toast erreur précis).
- [ ] T4.6 Modifier `frontend/src/routes/(app)/invoices/[id]/+page.svelte` :
  - Afficher `paidAt` si présent (format suisse).
  - Badge paiement.
  - Bouton « Marquer payée » si `validated && !paidAt`.
  - Bouton « Dé-marquer payée » si `validated && paidAt` (avec dialog confirmation dédié).
- [ ] T4.7 Modifier `frontend/src/routes/(app)/+layout.svelte` : ajouter entrée navGroup « Factures » → « Échéancier » → `/invoices/due-dates`.
- [ ] T4.8 Tests Vitest :
  - `invoices/due-dates.test.ts` si helpers spécifiques (ex. formateur d'URL query). Minimal — la majorité du test est en Playwright.

### T5 — i18n (AC: #12)

- [ ] T5.1 Ajouter ~20 clés × 4 langues (fr-CH, de-CH, it-CH, en-CH) dans `crates/kesh-i18n/locales/*/messages.ftl` :
  - Navigation : `nav-invoices-due-dates`.
  - Page : `due-dates-title`, `due-dates-filter-all`, `due-dates-filter-unpaid`, `due-dates-filter-overdue`, `due-dates-filter-paid`, `due-dates-summary` (avec args `{unpaidCount}, {unpaidTotal}, {overdueCount}, {overdueTotal}`), `due-dates-column-due-date`, `due-dates-column-payment-status`, `due-dates-column-paid-at`, `due-dates-export-button`, `due-dates-no-results`.
  - Statuts : `payment-status-paid`, `payment-status-unpaid`, `payment-status-overdue`.
  - Actions : `invoice-mark-paid-button`, `invoice-mark-paid-dialog-title`, `invoice-mark-paid-dialog-body`, `invoice-mark-paid-date-label`, `invoice-mark-paid-confirm`, `invoice-mark-paid-success`, `invoice-unmark-paid-button`, `invoice-unmark-paid-dialog-title`, `invoice-unmark-paid-dialog-body`, `invoice-unmark-paid-success`.
  - Erreurs : `invoice-error-paid-at-future`, `invoice-error-paid-at-before-invoice-date`, `invoice-error-mark-paid-not-validated`, `echeancier-export-error-too-large`.
  - En-têtes CSV (locale = `companies.accounting_language`, PAS locale UI) : `echeancier-csv-header-number`, `-date`, `-due-date`, `-contact`, `-total`, `-payment-status`, `-paid-at`.

### T6 — Tests (AC: #16, #17)

- [ ] T6.1 Tests DB `invoices::tests` (extensions listées en T2.5).
- [ ] T6.2 Tests d'intégration API (T3.6).
- [ ] T6.3 Test d'intégration perf (optionnel mais recommandé) : fixture 10k factures → `EXPLAIN` sur liste due-dates → asserter index utilisé. Si trop lent à monter en CI, marquer `#[ignore]` + documenter commande manuelle.
- [ ] T6.4 Test Playwright `frontend/tests/e2e/invoices_echeancier.spec.ts` :
  - Login comptable.
  - Créer 2 factures (une avec due_date passée, une avec due_date future) + valider les deux (réutiliser helpers 5.1/5.2).
  - Naviguer `/invoices/due-dates` → vérifier que les 2 apparaissent, celle passée surlignée « En retard ».
  - Cliquer « Marquer payée » sur la facture passée → confirmer → vérifier disparition du filtre « Impayées ».
  - Basculer filtre « Payées » → vérifier réapparition avec badge Payée.
  - Cliquer vue détail → bouton « Dé-marquer payée » → confirmer → retour à « Impayées ».
  - Bouton « Exporter CSV » → vérifier téléchargement (`page.on('download')`), extension `.csv`, taille > 0.
- [ ] T6.5 Audit i18n : script qui vérifie que les ~20 clés existent dans les 4 locales (souvent déjà posé en CI — vérifier).

### T7 — Validation finale

- [ ] T7.1 `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] T7.2 `cargo test --workspace -- --test-threads=1` (respecter contrainte SQLx cross-binary, mémoire `feedback_sqlx_mysql_gotchas`).
- [ ] T7.3 `npm run test:unit -- --run` full suite frontend.
- [ ] T7.4 `npm run check` (svelte-check 0 errors).
- [ ] T7.5 `npm run test:e2e` (Playwright — au moins `invoices_echeancier.spec.ts`).
- [ ] T7.6 Test manuel : créer facture, valider, vérifier sur échéancier, marquer payée, dé-marquer, exporter CSV (vérifier accents et montants dans Excel).
- [ ] T7.7 Mettre à jour sprint-status → `review`.

## Dev Notes

### Architecture — où va quoi

```
kesh-db/
├── migrations/20260419000001_invoice_paid_at.sql     # T1.1
└── src/
    ├── entities/invoice.rs                          # T1.2 (ajout paid_at)
    └── repositories/invoices.rs                     # T2.1-T2.4

kesh-api/
├── Cargo.toml                                       # ajout `csv` si absent
└── src/routes/invoices.rs                           # T3.1-T3.4

kesh-api/tests/invoice_echeancier_e2e.rs             # T3.6

frontend/src/lib/features/invoices/
├── invoices.api.ts                                  # T4.2
├── invoices.types.ts                                # T4.1
├── PaymentStatusBadge.svelte                        # T4.4
└── MarkPaidDialog.svelte                            # T4.5

frontend/src/routes/(app)/
├── invoices/due-dates/+page.svelte                  # T4.3
├── invoices/[id]/+page.svelte                       # T4.6 (extension)
└── +layout.svelte                                   # T4.7 (nav)

frontend/tests/e2e/invoices_echeancier.spec.ts       # T6.4

crates/kesh-i18n/locales/*/messages.ftl              # T5.1 (~20 clés × 4)
```

### Ce qui existe DÉJÀ — NE PAS refaire

- **Pattern repository CRUD + audit atomique + rollback** — `contacts.rs` / `products.rs` / `invoices.rs` (5.1/5.2). Stricte application.
- **`InvoiceListQuery`/`push_where_clauses`** — référence canonique pour extension (vs réécriture).
- **`DbError::{IllegalStateTransition, OptimisticLockConflict, NotFound, InvalidInput}`** — tous déjà mappés dans les handlers.
- **`get_company(&state)` mono-tenant** — pattern validé en 5.2/5.3.
- **`ListResponse<T>`** — `routes/mod.rs:25`. Sera **étendu** en `DueDatesResponse` wrapper (T3.1).
- **Crate `csv`** — vérifier présence dans `kesh-api/Cargo.toml` avant T3.4 (probablement absent — l'ajouter).
- **Design tokens + composants UI** — Story 1.9. Couleurs, espacements, badges → réutilisation stricte.
- **`formatSwissAmount`, `formatSwissDate`** — `frontend/src/lib/shared/utils/*`. Réutiliser, ne pas dupliquer.
- **Pattern `onMount` init URL + cleanup debounce** — Stories 4.1/4.2 canoniques.
- **Audit log `invoice.validated`** — wrapper précédent (5.2). Pattern identique pour `invoice.paid` / `invoice.unpaid`.

### Points de vigilance (prévention LLM)

1. **NE PAS créer d'écriture comptable dans `mark_as_paid`** — c'est un simple marqueur en 5.4. L'écriture d'encaissement est Epic 6. Toute tentative de générer une écriture ici nécessiterait de choisir un compte banque, ce qui n'est pas disponible en 5.4. **Décision explicite**, documentée dans « Scope hors story ».
2. **NE PAS oublier `is_overdue` dans la réponse liste** — calcul côté backend uniquement (single source of truth pour « today »). Le frontend ne doit PAS recalculer.
3. **NE PAS filtrer `status` côté frontend** — le handler `list_due_dates_handler` force `status = 'validated'` côté backend. Le frontend ne peut PAS demander des draft via cette route.
4. **Pas de contrainte d'ordre Axum** : Axum 0.8 / `matchit 0.8+` priorise automatiquement les segments statiques sur les dynamiques. `/invoices/due-dates` gagne sur `/invoices/{id}` sans condition d'ordre. Ne PAS introduire de sous-routeur séparé juste pour ça — cela complique le RBAC pour rien.
5. **CSV BOM UTF-8 obligatoire** — sans BOM, Excel Windows casse les accents. Test explicite qui vérifie les 3 premiers bytes `EF BB BF`.
6. **Séparateur CSV `;`** — standard suisse/francophone Excel. Pas `,` (anglo-saxon).
7. **En-têtes CSV locale = `companies.accounting_language`** — PAS la locale UI (pattern identique à description écriture comptable 5.2). Une entreprise FR génère un CSV FR même si l'utilisateur courant est EN.
8. **Validation `paidAt` temporelle** — refuser futur ET antérieur à `invoice.date`. Côté backend ET côté frontend (pour l'UX). Messages i18n distincts.
9. **`unmark-paid` uniquement depuis la vue détail** — pas de bouton sur la ligne de tableau (évite mis-clic destructif).
10. **Verrou optimiste `version` obligatoire** pour mark-paid/unmark-paid — même pattern que update. Le frontend doit passer la version courante.
11. **Limite export 10'000 lignes** — défense mémoire. Message d'erreur explicite (pas silent truncation).
12. **Audit actions dédiées `invoice.paid` / `invoice.unpaid`** — ne pas utiliser `invoice.updated` générique. Traçabilité métier.
13. **CHECK DB `paid_at IS NULL OR status = 'validated'`** — défense en profondeur. Redondant mais protège d'un bug applicatif.
14. **Migration number** — `ls crates/kesh-db/migrations/` avant création. Max observé au 2026-04-15 : `20260418000001_country_code.sql` → **placeholder `20260419000001`**.
15. **Route `GET /due-dates` + `GET /due-dates/summary` vs réponse unique** — décision retenue = **réponse unique** (summary inclus dans `DueDatesResponse`). 1 seul round-trip, cohérence des données. Ne PAS implémenter un endpoint summary séparé.
16. **Frontend : pré-sélection par défaut `paymentStatus=unpaid`** — si l'URL n'a pas de param. Pas « all » (sinon la page initiale est polluée par les payées).
17. **Surlignage overdue accessibilité** — couleur + icône ou texte (pas que couleur). Contraste AA. Test axe-core (pattern Story 1.11).
18. **`tokio::join!` pour list + summary** — read-only, safe. Pas besoin de transaction commune.
19. **NE PAS introduire `paid_amount` ou `remaining_amount`** — v0.1 binaire. Paiements partiels = Epic 10.
20. **Filtres `payment_status=overdue` + `dueBefore`** peuvent se combiner — sémantique : les 2 conditions s'additionnent (AND). Ex. `overdue + dueBefore=2026-03-01` = factures en retard ET dont l'échéance est avant le 1er mars. Cohérent.

### Previous Story Intelligence (Stories 5.1, 5.2, 5.3)

- **Story 5.1** (creation brouillon) : pattern repo, entités, audit snapshot direct pour create/delete + wrapper pour update. Champs `due_date` existe déjà, défaut pragmatique posé en 5.2 (`= invoice.date` si omis).
- **Story 5.2** (validation + numérotation) : `validate_invoice` atomique (tx unique), `create_in_tx` pour `journal_entries`, ordre des locks documenté. Handler `validate` renvoie `InvoiceResponse` enrichi. Tous les mappings d'erreurs `IllegalStateTransition → 409`, `ConfigurationRequired → 400`, `FiscalYearInvalid → 400` sont en place.
- **Story 5.3** (PDF QR Bill) : résolution locale via `state.config.locale` (pas de champ sur CurrentUser). Endpoint `GET /invoices/:id/pdf` en `authenticated_routes`. Pattern `get_company(&state)` mono-tenant. Validation `invoice.status = 'validated'` avant génération (409 sinon).
- **Épine dorsale audit** : action `invoice.paid` / `invoice.unpaid` s'inscrit dans la même convention. Le décodeur de `audit_log` côté UI doit savoir les afficher (si une vue d'audit existe — sinon no-op).

### Dépendances et ordre de livraison

- **Story 5.2 DOIT être done** avant démarrage 5.4 (sinon pas de `invoice_number` ni `validated` status) — **vérifié** au 2026-04-15 (5.2 done).
- **Story 5.3 recommandée done** (pas bloquant mais confort de test manuel) — **done** au 2026-04-15.
- **Epic 6** bénéficiera de 5.4 (la réconciliation automatique exploite `paid_at` manuel comme fallback si aucun matching trouvé — à confirmer au design Epic 6).

### Decisions logs / Questions

Aucune question bloquante au moment de la rédaction. Les décisions de conception sont tranchées. Si Guy souhaite valider explicitement avant dev :

- **Q1** : confirmer « admin autorisé à mark-paid » (AC #14) — sémantiquement cohérent avec le projet mais à valider.
- **Q2** : confirmer le choix d'un endpoint CSV séparé vs paramètre `?format=csv` sur le GET principal — décision retenue = **endpoint séparé** (clarté content-type, pas de branching dans le handler principal).
- **Q3** : confirmer le contenu de la colonne « Date d'échéance » dans le CSV si `due_date IS NULL` (théoriquement impossible après 5.2 mais défense) — afficher vide ou la `invoice.date` ? Retenu = **vide** (fidèle à la DB).

### References

- Source : `_bmad-output/planning-artifacts/epics.md` (Epic 5 — Facturation QR Bill, Stories 5.1-5.3).
- Source : `_bmad-output/implementation-artifacts/5-1-creation-factures-brouillon.md` (pattern CRUD + audit).
- Source : `_bmad-output/implementation-artifacts/5-2-validation-numerotation-factures.md` (pattern validate + ordre des locks + configuration facturation).
- Source : `_bmad-output/implementation-artifacts/5-3-generation-pdf-qr-bill.md` (pattern sous-endpoint GET `/:id/pdf`, i18n locale company).
- Source : `crates/kesh-db/src/repositories/invoices.rs` (InvoiceListQuery, push_where_clauses, patterns audit).
- Source : `crates/kesh-api/src/routes/invoices.rs` (mapping handlers existants).

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context) — création de la story en session autonome.

### Debug Log References

### Completion Notes List

- Story créée 2026-04-15 depuis le stub backlog du 2026-04-14.
- Colonne dérivée du stub : le stub mentionnait `value_date` mais la colonne réelle en DB est `due_date` (migration `20260416000001_invoices.sql` + défaut 5.2). Corrigé dans toute la story.
- Scope volontairement hors story : écriture comptable d'encaissement (Epic 6), paiements partiels (Epic 10), relances (post-MVP).
- Implémentation phased (a→d) 2026-04-15, Claude Opus 4.6. Voir Change Log 2.0 pour détails.
- Bug latents corrigés au passage (non bloquants 5.4 mais découverts) :
  1. `delete()` SELECT FOR UPDATE omettait `journal_entry_id` (Invoice struct a 13 champs, SELECT n'en listait que 12). Ajouté `journal_entry_id` + `paid_at`.
  2. Tests `test_update_rejects_non_draft` et `test_delete_rejects_non_draft` étaient cassés depuis la migration `20260417000002` (CHECK `chk_invoices_validated_has_je` violée par l'`UPDATE status='validated'` naïf). Remplacés par un helper `force_validate` qui crée un journal_entry stub. Pré-existant, non 5.4.
  3. `invoice_pdf.rs:104` — `map_err(|e| map_qrbill_error(e))` fixé en `map_err(map_qrbill_error)` (clippy `redundant_closure` exposé par cargo fmt).
- Pré-existant non résolu : 2 tests `kesh-api::config` échouent quand `DATABASE_URL` est set dans l'env (attendent `MissingVar`). Sans lien avec 5.4.
- Playwright spec `invoices_echeancier.spec.ts` livré mais non exécuté (requiert stack complète frontend+backend+DB avec seed fiscal_year + invoice_settings). À valider manuellement ou en CI.

### File List

**Créés :**

- `crates/kesh-db/migrations/20260419000001_invoice_paid_at.sql`
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs`
- `frontend/src/lib/features/invoices/PaymentStatusBadge.svelte`
- `frontend/src/lib/features/invoices/MarkPaidDialog.svelte`
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`
- `frontend/tests/e2e/invoices_echeancier.spec.ts`

**Modifiés :**

- `crates/kesh-db/src/entities/invoice.rs` — `paid_at` ajouté à `Invoice`
- `crates/kesh-db/src/errors.rs` — variant `DbError::InvalidInput(String)` + error_code
- `crates/kesh-db/src/repositories/invoices.rs` — `PaymentStatusFilter`, `InvoiceSortBy::DueDate`, `InvoiceListQuery.{payment_status, due_before}`, `due_dates_summary`, `mark_as_paid`, `list_for_export`, `push_where_clauses` étendu (UTC_DATE), `FIND_INVOICE_SCOPED_SQL` + `delete` SELECT étendus, `invoice_snapshot_json` avec `paidAt`, `InvoiceListItem.paid_at`, tests (8 nouveaux + 2 pré-existants réparés)
- `crates/kesh-api/Cargo.toml` — dep `csv = "1"`
- `crates/kesh-api/src/errors.rs` — mapping `DbError::InvalidInput` → 400
- `crates/kesh-api/src/lib.rs` — routes `/invoices/due-dates`, `/due-dates/export.csv`, `/:id/mark-paid`, `/:id/unmark-paid`
- `crates/kesh-api/src/routes/invoices.rs` — 4 handlers + DTOs + `paidAt` dans réponses
- `crates/kesh-api/src/routes/invoice_pdf.rs` — clippy fix (redundant_closure)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — ~40 clés × 4 langues
- `frontend/src/lib/features/invoices/invoices.types.ts` — types Due dates
- `frontend/src/lib/features/invoices/invoices.api.ts` — `listDueDates`, `markInvoicePaid`, `unmarkInvoicePaid`, `exportDueDatesCsv`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` — badge + boutons mark/unmark + dialog
- `frontend/src/routes/(app)/+layout.svelte` — entrée sidebar « Échéancier »

## Change Log

| Date       | Version | Description                                                                                                                                                                                                                                                                                                  | Auteur          |
| ---------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------|
| 2026-04-14 | 0.1     | Stub créé en backlog pendant démarrage Story 5.2 (scope v0.6)                                                                                                                                                                                                                                                | Claude Opus 4.6 |
| 2026-04-15 | 1.0     | Story contextualisée complète via `bmad-create-story` — scope verrouillé, 20 AC, 7 groupes de tasks, décisions de conception, ordre des locks, pièges LLM. Colonne réelle = `due_date` (pas `value_date`). Marquage manuel payée/impayée en v0.1 ; réconciliation automatique reportée Epic 6. Status → ready-for-dev. | Claude Opus 4.6 |
| 2026-04-15 | 1.1     | **Passe validate #1 (Claude Sonnet 4.6 orthogonal, contexte frais)** : 1 CRITICAL + 3 HIGH + 2 MEDIUM résolus. C1 : clarifié périmètre summary (`paymentStatus` ignoré, comportement intentionnel + test dédié). H1 : remplacé référence fantôme `INVOICE_COLUMNS` par `FIND_INVOICE_SCOPED_SQL` + `invoice_snapshot_json` réels (+ nouvelle tâche T1.2bis). H2 : ajouté warning `max_connections(4)` pour test concurrence. H3 : rendu l'ajout `csv = "1"` dans `kesh-api/Cargo.toml` obligatoire + config `WriterBuilder` + BOM manuel. M1 : corrigé l'avertissement erroné sur l'ordre des routes Axum (matchit 0.8+ priorise les segments statiques automatiquement). M2 : précisé conversion `paid_at.date() >= invoice.date` cross-type. | Claude Sonnet 4.6 |
| 2026-04-15 | 1.2     | **Passe validate #2 (Claude Haiku 4.5 orthogonal, contexte frais)** : 2 HIGH + 2 MEDIUM résolus (C1/C2 Haiku classés faux positifs — déjà couverts par T1.2 étendu). H-TZ : remplacé `CURDATE()` par `UTC_DATE()` partout + justification (convention projet UTC naïf). H-streaming : supprimé le streaming CSV (contradictoire avec limite dure 10k) au profit d'un `Vec<u8>` simple ~2 Mo max. M-MariaDB : noté que la CHECK est enforced à partir de 10.2 + projet cible 10.11+ (vérifié). M-CSV-contract : verrouillé le contrat API frontend `Promise<Blob>` + pattern `URL.createObjectURL` (pas de `<a href>` direct car JWT). Résidu : LOW uniquement (nit naming T1.2bis, DRY `DueDatesResponse` vs `ListResponse`) — critère d'arrêt atteint. | Claude Haiku 4.5 |
| 2026-04-16 | 2.2     | **Code review complète Story 5.4 — 4 groupes × 11 passes adversariales orthogonales (Opus/Sonnet/Haiku)**. **Groupe B (API backend) — 2 passes (Opus + Sonnet)** : 22 patches (B1 défaut backend `paymentStatus=Unpaid` AC#3 ; B2 export CSV gated Comptable+ ; B3 helper `is_invoice_overdue` centralisé ; B4 `today` capturé une fois par requête ; B7 assert Content-Disposition CSV ; B9 `csv_sanitize` étendu TAB+leading whitespace ; B10 test e2e `alreadyUnpaid` ; B11+B21 validations `dueBefore` vs `dateFrom`/`dateTo` ; B12 helper `validate_version` ≥ 0 ; B13 whitelist `InvalidInput` codes (anti-pollution clé FTL) ; B15 strip `payment_status` côté handler avant summary (rendre AC#11 explicite) ; B16 cap `MAX_OFFSET=1_000_000` ; B17 retrait clés FTL orphelines `invoice-error-paid-at-future` × 4 locales ; B19 `csv_sanitize` sur `payment_status` i18n CSV ; B20 cap `invoice_number` à 64 chars dans nom fichier PDF ; B22 `is_overdue` exposé sur `InvoiceListItemResponse` (cohérence liste standard ↔ échéancier, `DueDateItemResponse` aliasé) ; B23–B25 fixes commentaire test obsolète, typo « payed »→« paid », wording `validate_version`). Critère d'arrêt atteint pass 2 (Auditor PASS, zéro finding ≥ MEDIUM). **Groupe C (kesh-qrbill refondu) — 3 passes (Opus + Sonnet + Haiku)** : 22 patches sur le crate publishable Swiss QR Bill SIX 2.2. Pass 1 : `validate_qrr` `.unwrap()` → `map_err` ; `validate_iban` ASCII-check avant slice (anti-panic multi-byte) ; `data.currency` vs `invoice.currency` cross-check (sinon QR/PDF désync) ; `build_qrr(0,0)` → erreur (QRR tout-zéro rejeté banques) ; `format_amount_payload` réécriture défensive ; whitelist Annex C strict + ajouts pragmatiques U+2019 (apostrophe macOS), U+2014 (em-dash), U+20AC (€) ; validation pays ISO-3166-1 alpha-2 (table 249 codes) ; cross-check QR-IBAN ↔ Reference type (4 cas exhaustifs SIX §3.3) ; vecteur SIX officiel pour `compute_qrr_checksum` (`21000000000313947143000901` → check `7`) ; cap payload SIX 997 octets ; check `module_mm >= 0.4mm` ; suppression `eprintln!` (lib publiable) ; géométrie Swiss cross corrigée. Pass 2 : 8 patches dont 3 critiques régressions pass 1 — retrait `\n` du whitelist Annex C (séparateur payload SIX !), `format_amount_payload` `format!("{:.2}", ...)`, Swiss cross blanc 8×8 / rouge 7×7 (zone garde 0.5mm), payload bytes vs chars, `line2` non-vide pour type K, `is_iso_3166_alpha2` `debug_assert!` trié, line count strict `==32`. Pass 3 (Haiku) : Auditor PASS, 1 patch test verrouillage tri ISO. **Groupe D (Frontend Svelte 5 + i18n + Playwright) — 3 passes (Opus + Sonnet + Haiku)** : 17 patches. Pass 1 : régression frontend AC#8 amendé corrigée (`MarkPaidDialog` retire garde `paidAt > today` + attribut `max`) ; `paidAt` envoyé avec suffixe `Z` explicite (anti-naive datetime) ; `downloadPdf` whitelist `PDF_ERROR_KEYS` + `<a download>` hidden (au lieu de `window.open` popup-blocker) ; `toggleSort` branche morte « Asc:Asc » → defaults DESC montants/ASC dates ; `statusOf()` strict `=== true` ; `canExportCsv` derived sur `authState.role` (B2 cohérent) ; reset summary à zéro sur erreur load ; `FILTER_FALLBACK_FR` map (anti-leak raw enum) ; CSV export propage `sortBy/sortDirection` ; URL `paymentStatus` toujours écrit (D1:a share-friendly) ; `playwright.config.ts` `locale: 'fr-CH', timezoneId: 'Europe/Zurich'` (D4). Pass 2 : 5 patches (3 clés FTL `invoice-pdf-error-{not-found,generic,empty}` ajoutées 4 locales ; `INVOICE_TOO_MANY_LINES_FOR_PDF` remappé vers clé existante `error-invoice-too-many-lines-for-pdf` ; `isOverdue === true` strict aussi dans page `[id]` pour symétrie). Pass 3 (Haiku) : Auditor PASS, faux positifs Blind écartés. **Bilan global** : 80 patches sur 11 passes adversariales, 3 LLMs orthogonaux. Compilation `cargo check --tests -p kesh-{db,api,qrbill}` clean ; `svelte-check` 0 erreurs. Status → done après commit. | Claude Opus+Sonnet+Haiku |
| 2026-04-15 | 2.1     | **Code review Groupe A (DB layer) — 3 passes adversariales orthogonales**. Pass 1 (Opus, Blind+Edge+Auditor) : 7 HIGH + 10 MEDIUM ; 14 patches appliqués (CHECK paid_at alignée Rust↔SQL, CHECK élargie à `status IN ('validated','cancelled')` pour anticipation Epic 10, migration `country_code` UPDATE step 2 supprimé pour éviter perte de données, summary KPI durci sans filtres temporels initialement, `IF NOT EXISTS` partout, ORDER BY NULLs last déterministe, `paid_at` futur upper-bound 7j initialement, `paidAt` RFC3339, `journalEntryId` en audit snapshot, `list_for_export` retourne `(rows, truncated)`, `WHERE status='validated'` enforcé en export, 5 tests `escape_like`, test `test_mark_as_paid_atomic_rollback_on_audit_failure`). Pass 2 (Sonnet) : 1 HIGH + 4 MEDIUM dont **2 régressions de pass 1** corrigées : (a) restauration `date_from/date_to/due_before` dans summary (spec §74 explicite), (b) suppression de la borne supérieure `paid_at` (clarification domaine : `paid_at` = date d'exécution bancaire, futur légitime → AC#8 amendé). N3 ajouté : sémantique « `payment_status` non-`All` l'emporte sur `query.status` contradictoire » au lieu de zéro silencieux. N4 : assertion stricte `ForeignKeyViolation` sur test rollback. Pass 3 (Haiku) : Auditor PASS, Edge/Blind 1 MEDIUM réel (test conflit filtres manquant) → patché. **Critère d'arrêt atteint** (zéro finding ≥ MEDIUM hors décisions documentées). Dérogations spec actées : (1) AC#8 amendé `paid_at` peut être futur ; (2) CHECK DB élargie à `cancelled` vs spec §91 (anticipation Epic 10) ; (3) sémantique `payment_status` precedence (N3, non spécifié). | Claude Opus+Sonnet+Haiku |
| 2026-04-15 | 2.0     | **Implémentation complète (Claude Opus 4.6, phased run a/b/c/d)**. Phase (a) : migration `20260419000001_invoice_paid_at.sql` (colonne DATETIME(3) + CHECK + 2 index composites), entité `Invoice.paid_at`, `PaymentStatusFilter`, `InvoiceSortBy::DueDate`, `InvoiceListQuery.{payment_status, due_before}`, `due_dates_summary()` (une requête, `CAST AS SIGNED` pour les compteurs MariaDB), `mark_as_paid()` atomique (SELECT FOR UPDATE + check validated + CHECK calendaire `paid_at.date() >= invoice.date`), `list_for_export()`, variant `DbError::InvalidInput`. 8 nouveaux tests repo + 2 tests pré-existants réparés (bug latent : `test_update_rejects_non_draft` et `test_delete_rejects_non_draft` cassés par la CHECK `chk_invoices_validated_has_je` de 5.2). Bug latent corrigé aussi : `SELECT FOR UPDATE` de `delete()` et `FIND_INVOICE_SCOPED_SQL` omettaient `journal_entry_id` — ajouté. Phase (b) : 4 handlers (`list_due_dates`, `mark_invoice_paid`, `unmark_invoice_paid`, `export_due_dates_csv`), DTOs `DueDatesResponse`/`DueDateItemResponse` avec `isOverdue` backend, validation `paid_at <= now()` côté handler, CSV (BOM + `;` + CRLF + montants suisses + limite 10'000), 6 tests e2e intégration. Dep `csv = "1"` ajouté à `kesh-api/Cargo.toml`. Phase (c) : types TS, API client (`exportDueDatesCsv` via `apiClient.getBlob`), composants `PaymentStatusBadge` + `MarkPaidDialog`, page `/invoices/due-dates` (tabs paymentStatus, tri cliquable, summary, export, surlignage overdue, URL sync), intégration mark/unmark dans page détail `/invoices/[id]`, entrée sidebar. Phase (d) : spec Playwright `invoices_echeancier.spec.ts` (golden path + 401), audit i18n 156/156 clés présentes (~40 clés × 4 locales). **Validation** : `cargo fmt` clean, `cargo clippy -D warnings` clean (fix au passage d'un `redundant_closure` pré-existant dans 5.3), 24/24 tests `kesh-db::repositories::invoices` passent, 6/6 tests e2e `invoice_echeancier_e2e.rs` passent, `svelte-check` 0 errors, 181/181 vitest passent. Pré-existant : 2 tests `config` de `kesh-api` échouent quand `DATABASE_URL` est set dans l'env (non lié à 5.4). Status → review. | Claude Opus 4.6 |
