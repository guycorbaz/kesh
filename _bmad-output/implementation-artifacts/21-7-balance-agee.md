# Story 21.7: Balance âgée débiteurs (rapport)

Status: done

<!-- Créée 2026-07-19 par bmad-create-story. Cartographie ground-truth par 4 agents Explore (patron rapport kesh-report / route+CSV+RBAC / requête postes ouverts+TTC / frontend reports+liens croisés). Consomme 21-2 (helper TTC #246). Indépendante de 21-3..21-6. Décisions Guy 2026-07-19 : vue tous-rôles + export CSV Comptable+ (D24) ; ajout synchro URL ?tab= à /reports (onglets adressables → liens entrants directs). -->

## Story

En tant que **comptable d'une PME suisse**,
je veux **une balance âgée des créances clients — le total dû réparti par contact et par tranche d'ancienneté (Non échu, 1-30, 31-60, 61-90, 90+ jours de retard), exportable en CSV**,
afin de **piloter le recouvrement d'un coup d'œil, prioriser les relances sur les créances les plus anciennes, et réconcilier le total avec le compte débiteurs**.

## Contexte

Le cycle de suivi débiteurs (Epic 21) a livré les échéances (#245), les montants TTC canoniques (#246, story 21-2), le socle de rappels (21-3/21-4), l'envoi (21-5), la page Rappels et les intégrations fiche/dashboard (21-6a/b/c). **Il manque la vue de synthèse comptable : la balance âgée.** C'est la fonction de contrôle qui répartit l'encours débiteur par ancienneté et le réconcilie avec le grand-livre.

**21-7 est une story backend Rust + frontend** : un nouveau rapport `kesh-report::aged_receivables` (patron `balance_sheet.rs`), une route JSON + un export CSV, une vue frontend dans `/reports`, et les liens croisés de navigation. **Aucune nouvelle table, aucune migration, aucun bump `min_required`** — le rapport lit les factures existantes.

### Décisions figées

- **D-7a — Date de référence = aujourd'hui (UTC), v1.** Le paramètre « as-of » (date d'arrêté choisie) est **v2** (planning §E item 23). La `generate` prend un `as_of: NaiveDate` (= `Utc::now().naive_utc().date()` côté handler) — passé en paramètre (pas `UTC_DATE()` en dur) pour la **testabilité** (seeds à dates fixes).
- **D-7b — Vue tous-rôles, export CSV Comptable+ (Guy 2026-07-19, D24).** L'endpoint JSON de consultation (`GET /api/v1/reports/aged-receivables`) est monté dans `authenticated_routes` (Admin+Comptable+Consultation, comme les autres rapports, FR65). L'endpoint d'export CSV (`.../export`) est monté dans `comptable_routes` (garde `require_comptable_role` — Admin|Comptable). Précédent exact : l'export CSV de l'échéancier restreint Comptable+ (`lib.rs:601-603`, anti-exfiltration).
- **D-7c — Synchro URL `?tab=` sur /reports (Guy 2026-07-19).** La page `/reports` gagne une synchro URL minimale de l'onglet actif (`?tab=<id>`), lue au chargement (validée contre la liste des onglets, fallback défaut) et écrite au changement d'onglet (`replaceState`). Rend **tous** les onglets adressables/bookmarkables — c'est ce qui permet aux liens entrants (échéancier, Rappels) de cibler directement l'onglet balance âgée (21-6c avait reporté ce lien faute de synchro URL).
- **D-7d — Buckets fixes (planning §E).** « Non échu | 1-30 | 31-60 | 61-90 | 90+ jours de retard ». Bornes figées (pas de configuration). `days = as_of − due_date`. Voir AC 3 pour les frontières exactes.
- **D-7e — Factures suspendues INCLUSES (D10, invariant anti-dissimulation).** Le prédicat postes ouverts = `status = 'validated' AND paid_at IS NULL`. Il ne doit **PAS** ajouter `dunning_paused_at IS NULL` (une facture suspendue reste dans la balance âgée — elle ne sort que de la liste « à rappeler »).

### ⚠️ Piège n°1 — le montant bucketisé est le TTC dérivé, jamais `total_amount`

`invoices.total_amount` est le **HT** (Σ `line_total`). Le total dû (créance) est le **TTC**, dérivé des lignes par le helper canonique #246 (story 21-2). La balance âgée DOIT consommer les constantes SQL existantes — **jamais recalculer le `ROUND` soi-même** :

- **`kesh_db::repositories::invoices::INVOICE_TTC_DERIVED_JOIN_SQL`** (`crates/kesh-db/src/repositories/invoices.rs:154-163`, `pub const`) — forme agrégat (table dérivée `LEFT JOIN (… SUM(line_total + ROUND(line_total*vat_rate/100,2)) AS ttc …) lt ON lt.invoice_id = i.id`), **prérequis alias `i` sur `invoices`**. C'est la forme adaptée aux SUM multi-factures. `kesh-report` dépend déjà de `kesh-db` (cf. `balance_sheet.rs` utilise `kesh_db::errors::map_db_error`) → la constante `pub` est importable.
- Helper Rust de vérité : `kesh_core::accounting::vat::invoice_total_ttc` (`crates/kesh-core/src/accounting/vat.rs:62-71`). La parité SQL≡Rust est asservie par `crates/kesh-db/tests/invoice_ttc_parity.rs` — **la balance âgée réplique cette discipline** (AC 12).

### ⚠️ Piège n°2 — sérialisation camelCase des champs de bucket numérotés

Un champ nommé `days_1_30` sérialisé par `#[serde(rename_all = "camelCase")]` donne `days130` (ambigu). **Nuance (finding validate P1)** : avec les noms de champs effectivement retenus en AC 1 (`days_1_to_30`, `days_31_to_60`, `days_61_to_90`, `days_over_90` — avec `to`/`over`), `rename_all = "camelCase"` produit **déjà** automatiquement `days1To30`/`days31To60`/`days61To90`/`daysOver90` — les cibles voulues. Le `#[serde(rename = "…")]` explicite reste donc recommandé comme **renfort défensif** (explicite > implicite, protège d'un futur renommage) et pour figer le contrat vis-à-vis du miroir TS, mais ce n'est PAS un correctif obligatoire pour éviter un bug avec ces noms-là. Cible du JSON : `notDue`, `days1To30`, `days31To60`, `days61To90`, `daysOver90`, `total`.

### Hors scope (garde-fous)

- **Paramètre as-of (date d'arrêté)** → v2. v1 = aujourd'hui uniquement.
- **Export PDF** de la balance âgée → hors scope (le planning §E item 23 ne demande que le CSV). Ne créer que `render_aged_receivables_csv`, pas de `render_*_pdf`.
- **Paiement partiel** (L21-1) : la balance raisonne en tout-ou-rien (`paid_at`). Un acompte n'apparaît pas — hors scope.
- **Bornes de buckets configurables** → hors scope (bornes figées D-7d).
- **Drill-down vers une facture individuelle** : le lien par contact pointe vers la **liste filtrée** `/invoices?contactId={id}` (déjà supportée, aucun changement backend/liste), pas vers une facture.
- **Aucune nouvelle table** → **pas** de changement export global / backup / audit d'idempotence / compteurs (D26 ne s'applique qu'aux nouvelles tables) ; **pas de migration** ; **pas de bump `min_required`**.
- **Réglages #255/#256/#257** (dettes pré-existantes) : ne pas corriger ici.
- **RBAC configurable (#258)** : la story utilise le modèle 3-rôles actuel (gardes ordinales `require_comptable_role` / tous-rôles). Le passage à des permissions configurables est un epic dédié (#258, après Epic 21) ; les gardes posées ici (vue tous-rôles, export Comptable+) migreront alors vers des permissions comme le reste du code. Ne rien anticiper ici.

## Acceptance Criteria

### A. Rapport `kesh-report::aged_receivables` (backend)

1. **Nouveau module `crates/kesh-report/src/aged_receivables.rs`** avec les structs `Serialize` (montants `rust_decimal::Decimal`, jamais f64/string), **renames serde explicites** sur les buckets (piège n°2) :
   - `AgedBucket { not_due, days_1_to_30, days_31_to_60, days_61_to_90, days_over_90, total }` — chaque champ avec `#[serde(rename = "notDue"|"days1To30"|"days31To60"|"days61To90"|"daysOver90"|"total")]`.
   - `AgedReceivablesRow` : `contact_id: i64` (`#[serde(rename="contactId")]`), `contact_name: String` (`contactName`), + les 6 champs de `AgedBucket` (aplatis `#[serde(flatten)]` **ou** dupliqués — choisir la forme qui sérialise proprement ; documenter).
   - `AgedReceivables { as_of: NaiveDate (#[serde(rename="asOf")]), rows: Vec<AgedReceivablesRow>, totals: AgedBucket }`.

2. **`pub async fn generate(pool: &MySqlPool, company_id: i64, as_of: NaiveDate) -> Result<AgedReceivables, ReportError>`** (patron `balance_sheet::generate`). Une **seule requête agrégée** groupée par contact :
   - `FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id` + `INVOICE_TTC_DERIVED_JOIN_SQL` (alias `lt`).
   - `WHERE i.company_id = ? AND i.status = 'validated' AND i.paid_at IS NULL` (postes ouverts — **sans** filtre `dunning_paused_at`, D-7e).
   - Les 5 buckets via `SUM(CASE WHEN … THEN COALESCE(lt.ttc,0) ELSE 0 END)` + `total = SUM(COALESCE(lt.ttc,0))`.
   - `GROUP BY c.id, c.name ORDER BY c.name, c.id`. `HAVING total <> 0` (écarte une éventuelle facture legacy sans lignes → TTC 0, qui polluerait la liste sans montant).
   - **Scoping multi-tenant obligatoire** : `WHERE i.company_id = ?` + `.bind(company_id)` (jamais un `company_id` client). `as_of` **bindé** (pas `UTC_DATE()`).
   - Erreurs DB → `ReportError::Db` via `kesh_db::errors::map_db_error`.

3. **Frontières des buckets** (`days = as_of − due_date`, en jours) :
   - **Non échu** : `due_date IS NULL` **OU** `due_date >= as_of` (⇔ `days <= 0`). Une facture sans échéance est « non échue » (cohérent avec `is_invoice_overdue`/`days_overdue` qui la traitent comme non-en-retard).
   - **1-30** : `1 <= days <= 30`.
   - **31-60** : `31 <= days <= 60`.
   - **61-90** : `61 <= days <= 90`.
   - **90+** : `days >= 91` (strictement plus de 90 jours — pas de recouvrement de borne avec 61-90).
   - Implémentation SQL : `DATEDIFF(?, i.due_date)` (bind `as_of`) ; `NULL due_date` → `DATEDIFF` renvoie `NULL` → géré par la branche `due_date IS NULL OR …` du bucket « Non échu ».

4. **Totaux généraux (`totals`) sommés en Rust** (patron `balance_sheet` : `.iter().map(…).sum()`), pas en SQL — un `AgedBucket` où chaque colonne = Σ de la colonne sur toutes les `rows`, et `total` = Σ des `row.total`.

5. **Réexports** dans `crates/kesh-report/src/lib.rs` : `pub use aged_receivables::{generate as generate_aged_receivables, AgedReceivables, AgedReceivablesRow, AgedBucket};` (patron des autres rapports) + `pub use csv::render_aged_receivables_csv;`.

### B. Export CSV (backend, `kesh-report`)

6. **`pub fn render_aged_receivables_csv<W: Write>(report: &AgedReceivables, writer: W) -> Result<(), ReportError>`** dans `crates/kesh-report/src/csv.rs` (patron **exact** `render_vat_report_csv`) : BOM UTF-8 (`write_bom`), `WriterBuilder` `delimiter(b';')` + `Terminator::CRLF`, montants via le formateur ISO existant (`format_amount_iso`). **AUCUN paramètre `locale`** (⚠️ finding validate P1 : les `render_*_csv` existants n'ont pas de param locale — en-têtes **français en dur**, ne pas inventer une signature localisée divergente). En-tête littéral FR : `Contact;Non échu;1-30;31-60;61-90;90+;Total`. Une ligne par contact, puis une **ligne « Total général »** avec les `totals`. Rapport vide → en-tête seul (patron court-circuit `render_vat_report_csv`).

### C. Routes HTTP (backend, `kesh-api`)

7. **Handler JSON `get_aged_receivables`** dans `crates/kesh-api/src/routes/reports.rs` (structure calquée sur `get_balance_sheet`) : extracteurs `State<AppState>` + `Extension<CurrentUser>` (pas de `Query` — aucun paramètre en v1). Calcule `let as_of = Utc::now().naive_utc().date();`, appelle `generate_aged_receivables(&state.pool, current_user.company_id, as_of)`, renvoie `Json<AgedReceivables>`. Audit best-effort **DÉDIÉ** `emit_aged_receivables_audit(as_of, …)` (⚠️ NE PAS réutiliser `emit_report_audit` `:1057` — sa signature exige `fiscal_year_id/period_start/period_end`, la modifier casserait ses 4 callers Story 9-1).

8. **Handler export `export_aged_receivables`** (structure du corps calquée sur `export_balance_sheet`, MAIS **pas les helpers `ReportPeriod`**) : `State` + `Extension<CurrentUser>` + `Query<AgedExportQuery { format: Option<String> }>`.
   - **Validation `format` DÉDIÉE** (⚠️ finding validate P1) : NE PAS réutiliser `validate_format` (`reports.rs:124`) — elle **rejette `None`** (400) alors qu'on veut un défaut `csv`. Écrire une garde locale : `None` ou `"csv"` → OK ; toute autre valeur (dont `"pdf"`) → `AppError::Validation` 400.
   - **Réponse SANS les helpers `ReportPeriod`** (⚠️ finding validate P1) : `build_export_response_with_locale` (`reports.rs:980`) et `build_filename` (`:1007`) prennent un `&ReportPeriod` et composent `..._{periodStart}_{periodEnd}` — **incompatibles** avec un `as_of` unique. Suivre plutôt le **précédent « date unique » de `routes/exports.rs::export_global`** (`:104`, `:122`) : construire le filename via un helper de type `build_global_filename(company_name, as_of)` (nom `kesh-balance-agee-{company_slug}-{asOf}.csv`) puis la `Response` à la main avec `crate::util::build_content_disposition(&filename, locale_bcp47)` + `header::CONTENT_TYPE = "text/csv; charset=utf-8"`. Bufferiser le CSV via `render_aged_receivables_csv` dans un `Vec<u8>`.
   - **Audit export best-effort DÉDIÉ** : NE PAS appeler `emit_report_export_audit` (`reports.rs:1113`) — sa signature exige `fiscal_year_id/period_start/period_end` (« la modifier briserait les 4 callers Story 9-1 »). Écrire `emit_aged_receivables_export_audit(as_of: NaiveDate, …)` bespoke (details_json snake_case).

9. **Montage des routes** dans `crates/kesh-api/src/lib.rs` — **RBAC divergent (D-7b)** :
   - `GET /api/v1/reports/aged-receivables` → `get_aged_receivables` dans **`authenticated_routes`** (tous rôles).
   - `GET /api/v1/reports/aged-receivables/export` → `export_aged_receivables` dans **`comptable_routes`** (garde `require_comptable_role`, `lib.rs:275`/`:559-561`).
   - ⚠️ Garde IDOR (`lib.rs:772-774`) : les deux routes DOIVENT rester **dans** leur sous-routeur avant le `;` fermant (une route orpheline bypass l'auth → IDOR cross-tenant).

### D. Frontend — types & wrappers (`features/reports/`)

10. **`frontend/src/lib/features/reports/reports.types.ts`** gagne (miroir camelCase, montants `string`) : `AgedBucketDto { notDue; days1To30; days31To60; days61To90; daysOver90; total }` (tous `string`), `AgedReceivablesRowDto extends AgedBucketDto { contactId: number; contactName: string }`, `AgedReceivablesDto { asOf: string; rows: AgedReceivablesRowDto[]; totals: AgedBucketDto }`.
    - ⚠️ **NE PAS toucher `ReportType`** (`reports.types.ts:113`, finding validate P1) : `ReportType` est le domaine de `downloadReport`/`getReportExportUrl`/`buildExportFilename`/`TYPE_SLUGS_FALLBACK` qui présupposent tous une `ReportQuery` (`fiscalYearId`/`period*`) — y ajouter `'aged-receivables'` casserait la compilation TS (Record incomplet) et introduirait un type sans query cohérente.
    - Le type d'onglet **`TabId` est LOCAL** à `frontend/src/routes/(app)/reports/+page.svelte:50` (`type TabId = ReportType | 'project-expenses' | 'project-return'`) — c'est **là** qu'on ajoute `| 'aged-receivables'` (AC 13), PAS dans `reports.types.ts`.

11. **`frontend/src/lib/features/reports/reports.api.ts`** gagne des wrappers **dédiés** (ne PAS réutiliser `downloadReport`/`buildExportFilename`, liés à `ReportType`+`ReportQuery`, cf. AC 10) : `getAgedReceivables(): Promise<AgedReceivablesDto>` → `apiClient.get('/api/v1/reports/aged-receivables')` ; `downloadAgedReceivables(): Promise<void>` (patron `onExportCsv` de l'échéancier `due-dates/+page.svelte:251-282` : `apiClient.getBlob('/api/v1/reports/aged-receivables/export?format=csv')` → `res.blob()` → `<a download>` éphémère, filename **`balance-agee-${today}.csv`** avec `today = new Date().toISOString().slice(0,10)`). Tests vitest : chaque wrapper appelle le bon chemin/méthode.

### E. Frontend — vue & page

12. **`AgedReceivablesView.svelte`** (sous `features/reports/`, namespace **`reports-*`** uniquement — lint #30) : props `{ dto: AgedReceivablesDto }`. En-tête « Arrêté au {formatSwissDate(dto.asOf)} » + lien vers l'échéancier (`reports-aged-link-due-dates` → `/invoices/due-dates`). Tableau : colonnes Contact | Non échu | 1-30 | 31-60 | 61-90 | 90+ | Total (`reports-aged-col-*`), une ligne par `row` (montants `formatReportAmount`, `font-mono text-right`), **le nom du contact est un lien** vers `/invoices?contactId={row.contactId}` (drill-down — `?contactId=` déjà supporté par `/invoices`). `<tfoot>` = ligne « Total général » (`reports-aged-total-row`) avec `dto.totals`. Empty-state si `rows.length === 0` (`reports-aged-empty`). `data-testid` : `aged-receivables-table`, `aged-receivables-row`, `aged-receivables-total`.

13. **`frontend/src/routes/(app)/reports/+page.svelte`** :
    - **Type d'onglet** : ajouter `| 'aged-receivables'` au `TabId` **local** `:50` (PAS `ReportType`, cf. AC 10).
    - **Onglet** : ajouter `{ id: 'aged-receivables', labelKey: 'reports-aged-balance', fallback: 'Balance âgée' }` au tableau `tabs` (→ 8 onglets). Stockage DTO `let agedReceivables = $state<AgedReceivablesDto | null>(null)`. Rendu du panneau : `{:else if activeTab === 'aged-receivables' && agedReceivables} <AgedReceivablesView dto={agedReceivables} />`.
    - **Contrôles conditionnels — passer le bloc BINAIRE en TROIS branches** (⚠️ finding validate P1) : le template actuel est `{#if !isProjectTab} <ReportSelector … onExportPdf onExportCsv …/> {:else} <!-- contrôles projet --> {/if}` (`:429-508`). Le transformer en `{#if isProjectTab} … {:else if isAgedTab} … {:else} <ReportSelector …/> {/if}` avec `isAgedTab = $derived(activeTab === 'aged-receivables')`. **Le bloc `isAgedTab` ne branche NI `ReportSelector` NI les exports génériques `onExportPdf`/`onExportCsv`/`canExport`** (liés à `activeReportPeriod()` qui renvoie `null` pour cet onglet) — juste « Arrêté au {today} » + bouton **Générer** + le bouton Export CSV dédié (ci-dessous). `generate()` gagne `case 'aged-receivables': agedReceivables = await getAgedReceivables()` (dans le même `genSeq` race-guard).
    - **Export CSV dédié** : `reports/+page.svelte` **importe pour la 1re fois** `authState` depuis `$lib/app/stores/auth.svelte` (⚠️ finding validate P1 — aucun état d'auth aujourd'hui) et dérive `const canManage = $derived(authState.currentUser?.role === 'Admin' || authState.currentUser?.role === 'Comptable')` (patron `reminders/+page.svelte:39-40`). Le bouton Export CSV n'apparaît que si `canManage` (D-7b — inutile d'afficher un bouton qui prendra 403), appelle `downloadAgedReceivables`, guard `exporting`. Un Consultation voit le tableau mais pas le bouton.
    - **Synchro URL `?tab=` (D-7c)** — **lecture ONE-SHOT au montage, PAS un `$effect` réactif** (⚠️ finding validate P1, risque de boucle) : dans `onMount` (ou un effet gardé par un flag d'initialisation), lire `page.url.searchParams.get('tab')`, le valider contre les `id` connus du tableau `tabs` (invalide/absent → défaut `balance-sheet`), initialiser `activeTab`. **Ne jamais re-lire `page.url` en continu.** Sur `selectTab`, écrire `?tab=<id>` via `goto('?tab=…', { replaceState: true, keepFocus: true, noScroll: true })` (replaceState — ne pas empiler l'historique). Ne pas casser la navigation clavier tabs ni le `genSeq`. **Note** : un deep-link `?tab=project-expenses`/`project-return` ouvre l'onglet projet avec un sélecteur vide (pas de `selectedProjectId` dans l'URL) — pas de crash, UX dégradée acceptable, hors scénarios E2E (seul `?tab=aged-receivables` est testé).

### F. Liens croisés (frontend)

14. **Échéancier → balance âgée** (`invoices/due-dates/+page.svelte`, en-tête flex existant) : `<Button variant="outline" href="/reports?tab=aged-receivables" data-testid="due-dates-link-aged">` libellé `due-dates-link-aged` (« Voir la balance âgée »). Placé à côté du lien `due-dates-link-reminders` existant (21-6c).
15. **Rappels → balance âgée** (`invoices/reminders/+page.svelte`, en-tête flex existant) : lien analogue vers `/reports?tab=aged-receivables`, libellé `reminders-link-aged` (« Voir la balance âgée »). Complète le maillage échéancier ↔ Rappels ↔ balance âgée.
16. **Balance âgée → échéancier + drill-down par contact** : couverts par AC 12 (lien en-tête `reports-aged-link-due-dates` + lien contact → `/invoices?contactId=`).

### G. i18n

17. **Nouvelles clés dans les 4 FTL** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), traductions réelles FR/DE/IT/EN :
    - Rapport (`reports-aged-*`) : `reports-aged-balance` (label onglet), `reports-aged-title`, `reports-aged-as-of` (`{ $date }`), `reports-aged-empty`, colonnes `reports-aged-col-contact` / `-col-not-due` / `-col-1-30` / `-col-31-60` / `-col-61-90` / `-col-over-90` / `-col-total`, `reports-aged-total-row`, `reports-aged-link-due-dates`, `reports-filename-aged-receivables` (slug fichier, ex. `balance-agee`).
    - Liens routes : `due-dates-link-aged`, `reminders-link-aged`.
    - **Placement lint** : `AgedReceivablesView` sous `features/reports/` → clés `reports-*` uniquement (piège #30, respecté par construction). `due-dates-*` / `reminders-*` sont des routes (hors périmètre lint). Réutiliser `reports-export-csv-button` + le bouton Générer existant. ⚠️ **Note (finding validate P1)** : `+page.svelte:496` appelle `i18nMsg('reports-generate', 'Générer')` mais `reports-generate` est une **dead-key pré-existante** (absente des FTL, seul `reports-button-generate` existe) — hors scope 21-7, ne pas s'appuyer dessus ; utiliser le fallback ou la vraie clé `reports-button-generate`.

### H. Tests

18. **Tests d'intégration `kesh-report`** — nouveau `crates/kesh-report/tests/aged_receivables.rs`, `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` + `seed_accounting_company` + `invoices::create`/`validate_invoice`. Seeder plusieurs factures pour ≥ 2 contacts avec échéances couvrant **chaque bucket ET chaque frontière** (`due_date` = `as_of` [non échu], `as_of − 1` [1-30], `as_of − 30`, `as_of − 31`, `as_of − 60`, `as_of − 61`, `as_of − 90`, `as_of − 91` [90+], + une facture **sans `due_date`** [non échu], + une facture **suspendue** `dunning_paused_at` [incluse, D-7e], + une facture **payée** [exclue], + un **brouillon** [exclu]). Asserter :
    - Chaque facture tombe dans le bon bucket (aux frontières 30/31, 60/61, 90/91).
    - **Réconciliation par ligne** : pour chaque `row`, `not_due + days_1_to_30 + … + days_over_90 == total`.
    - **Réconciliation générale** : `Σ rows.total == totals.total` et chaque colonne des `totals` == Σ de la colonne.
    - **Parité TTC** : `totals.total ==` Σ `invoice_total_ttc` (helper Rust) sur les postes ouverts seedés (réplique de `invoice_ttc_parity.rs`) — au centime.
    - **Scoping** : une facture d'une **autre company** n'apparaît pas.
    - Empty-state : company sans poste ouvert → `rows` vide, `totals` à zéro.
19. **Tests E2E backend `kesh-api`** — étendre/créer `crates/kesh-api/tests/reports_e2e.rs` (ou `aged_receivables_e2e.rs`) : `GET /aged-receivables` **200 pour Consultation** (vue tous-rôles, D-7b) ; `GET /aged-receivables/export` **403 pour Consultation**, **200 `text/csv`** pour Comptable et Admin (D24) ; **scoping cross-tenant** — `company_id` vient du JWT (`CurrentUser`), une autre company n'est pas exposable via l'API ; le test seede des factures pour **deux companies distinctes** et vérifie que le rapport de la company A ne contient **QUE** ses contacts/montants (les postes de B absents des `rows`), **HTTP 200 avec `rows` scopées** (pas de 404) ; format invalide (`?format=pdf`) → 400.
20. **E2E frontend** — `frontend/tests/e2e/reports.spec.ts` : **mettre à jour l'assertion `toHaveCount(7)` → `toHaveCount(8)`** (nouvel onglet). Nouveau scénario : seeder des factures échues (helper `overdueDate` promu 21-6c) → `/reports` → onglet « Balance âgée » → Générer → le tableau affiche des lignes + le total général ; le lien d'un contact pointe vers `/invoices?contactId=` ; **deep-link `/reports?tab=aged-receivables`** ouvre directement l'onglet (D-7c) ; le bouton Export CSV est **visible pour admin**, **absent pour Consultation**. Lien croisé : depuis l'échéancier, `due-dates-link-aged` ouvre l'onglet balance âgée.
21. **vitest** — `AgedReceivablesView` rend lignes + total + empty-state ; wrappers `reports.api` (AC 11).

### I. Gate & documentation

22. **Gate local complet (Test Locally First) — story BACKEND, gate workspace complet obligatoire** :
    ```sh
    cargo fmt --all -- --check
    cargo build --workspace --all-targets
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace           # (ou scripts/test-fast.sh si MariaDB dev up)
    cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
    cd frontend && npm run test:e2e  # PAS dans la CI → critique (backend kesh_e2e + PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64)
    ```
    ⚠️ Aucune migration → pas de compteur migrations/backup/export à toucher, pas de bump `min_required`/Cargo. Le gate runtime (boot/import) n'est pas concerné (pas de schéma). `cd frontend` explicite (cwd errant après tâche de fond, leçon 21-6b) ; jamais de pipe sur le runner.
23. **CHANGELOG** `[Non publié]` → `Ajouté` : balance âgée débiteurs (répartition par contact et ancienneté, réconciliation TTC, export CSV Comptable+, navigation croisée). **Manuels → 21-8.** **README** : Epic 21 déjà 🚧, aucun changement de statut d'epic.

## Tasks / Subtasks

- [x] **T1 — Rapport `kesh-report::aged_receivables`** (AC: 1, 2, 3, 4, 5) — module + structs (renames serde explicites, piège n°2), `generate(pool, company_id, as_of)` (requête agrégée par contact, buckets `DATEDIFF`, TTC via `INVOICE_TTC_DERIVED_JOIN_SQL`, D-7e sans exclusion paused), totaux sommés en Rust, réexports `lib.rs`.
- [x] **T2 — Export CSV** (AC: 6) — `render_aged_receivables_csv` (patron `render_vat_report_csv`, BOM+`;`+CRLF), réexport.
- [x] **T3 — Routes HTTP + RBAC** (AC: 7, 8, 9) — `get_aged_receivables` (authenticated_routes) + `export_aged_receivables` (comptable_routes, D24), montage `lib.rs` (garde IDOR), audit best-effort, validation format.
- [x] **T4 — Frontend types & wrappers** (AC: 10, 11, 21) — DTO camelCase, `getAgedReceivables`/`downloadAgedReceivables`, vitest.
- [x] **T5 — Vue & page reports** (AC: 12, 13) — `AgedReceivablesView.svelte`, onglet (8e), contrôles conditionnels `isAgedTab`, branche `generate()` (genSeq), export CSV gate Comptable+, **synchro URL `?tab=` (D-7c)**.
- [x] **T6 — Liens croisés** (AC: 14, 15, 16) — échéancier→balance âgée, Rappels→balance âgée, balance âgée→échéancier + drill-down contact.
- [x] **T7 — i18n 4 FTL** (AC: 17).
- [x] **T8 — Tests** (AC: 18, 19, 20) — intégration kesh-report (buckets/frontières/réconciliation/parité/scoping), reports_e2e (RBAC vue vs export), E2E frontend (onglet 8, génération, deep-link `?tab=`, export gate, lien croisé).
- [x] **T9 — Gate workspace complet + CHANGELOG** (AC: 22, 23).

## Dev Notes

### Pièges, par ordre de coût

1. **TTC dérivé, jamais `total_amount` (piège n°1).** Consommer `INVOICE_TTC_DERIVED_JOIN_SQL` (`kesh-db`, `pub const`, alias `i`). Test de parité au helper Rust `invoice_total_ttc` obligatoire (AC 18). `total_amount` = HT.
2. **Sérialisation camelCase des buckets numérotés (piège n°2).** `#[serde(rename = "…")]` explicite sur chaque champ de `AgedBucket` — sinon `days_1_30` → `days130` (ambigu, casse le miroir TS).
3. **RBAC divergent (D-7b/D24).** Vue JSON = `authenticated_routes` (tous rôles) ; export CSV = `comptable_routes` (Comptable+). Ne pas monter l'export dans `authenticated_routes`. Garde IDOR : routes DANS le sous-routeur avant le `;`.
4. **Invariant D10 (D-7e).** Ne PAS ajouter `dunning_paused_at IS NULL` au WHERE postes ouverts. Une facture suspendue reste dans la balance âgée. Test dédié (AC 18).
5. **`due_date` nullable + frontières.** `NULL` → « Non échu ». Frontières exactes (30/31, 60/61, 90/91) testées (AC 18). `DATEDIFF(as_of, due_date)`, `as_of` bindé (testabilité), UTC partout.
6. **`ReportPeriod` inadapté.** Les rapports existants prennent une période `[start,end]` + `fiscal_year_id` ; la balance âgée prend un `as_of: NaiveDate` unique — **pas** de `fiscal_year_id`, **pas** de `ReportPeriod`. Signature `generate(pool, company_id, as_of)`.
7. **Synchro URL `?tab=` (D-7c).** Nouveau comportement transverse à /reports : **lecture ONE-SHOT au montage** (jamais un `$effect` qui re-lit `page.url` en continu → risque de boucle/double-écriture pendant un `genSeq` en vol, finding validate P1), valider la valeur contre les `id` connus (fallback défaut), écrire en `replaceState` (ne pas empiler l'historique), ne pas casser le `genSeq` ni la navigation clavier. Seul ajout « non-rapport » de la story.

### Leçon de review héritée (à appliquer dès le dev)

**Un patch de review vient AVEC son test** (21-5b/21-6c : convergence en 2 passes quand chaque patch porte son test). **Fix structurel > incrémental** sur un bug d'état. **Disclosure non sélective** : documenter toutes les déviations. **Ne jamais piper le runner** (masque l'exit code). **Gate workspace complet** pour une story backend (une régression fanout SQL n'est visible qu'au gate complet — leçon 21-5a/21-6a).

### Contrats backend (ground-truth, à ne pas re-deviner)

| Élément | Emplacement | Note |
|---|---|---|
| Patron rapport | `crates/kesh-report/src/balance_sheet.rs:61-85` | `generate(pool, company_id, period)` → structs camelCase, totaux sommés Rust, `Decimal` |
| TTC SQL agrégat | `crates/kesh-db/src/repositories/invoices.rs:154-163` | `INVOICE_TTC_DERIVED_JOIN_SQL` `pub const`, alias `i`, join `lt.ttc` |
| TTC helper Rust | `crates/kesh-core/src/accounting/vat.rs:62-71` | `invoice_total_ttc((line_total, vat_rate)…)` — parité SQL≡Rust (`invoice_ttc_parity.rs`) |
| Prédicat postes ouverts | `crates/kesh-db/src/repositories/invoices.rs:309-311` | `status='validated' AND paid_at IS NULL` |
| Invariant D10 (suspension incluse) | `invoice.rs:44-46`, `invoices.rs:124-131` (doc `PausedFilter`), `:326-333` (application, no-op par défaut) | balance âgée n'exclut PAS `dunning_paused_at` — le défaut `PausedFilter::All` est un no-op délibéré |
| Groupement/nom contact | `dunning_eligibility.rs:73-90` | `JOIN contacts c`, `c.name`, `ORDER BY c.name, c.id` |
| Handler rapport | `crates/kesh-api/src/routes/reports.rs:139-169` | `State`+`Extension<CurrentUser>`(+`Query`), `Json<T>`, audit best-effort |
| Handler export (structure) | `reports.rs:278-341` | calquer la structure, MAIS pas `build_export_response_with_locale`/`build_filename` (`:980`/`:1007`, exigent `&ReportPeriod`) |
| Précédent export « date unique » | `routes/exports.rs:104`, `:122` (`build_global_filename(company_name, export_date)` + `build_content_disposition`) | **le bon patron** pour un filename à date unique (`as_of`), sans `ReportPeriod` |
| `validate_format` (à NE PAS réutiliser) | `reports.rs:124` (test `:1270` `rejects_none`) | rejette `None` → 400 ; écrire une garde dédiée « défaut csv » |
| Audit (à NE PAS réutiliser) | `emit_report_audit:1057`, `emit_report_export_audit:1113` | exigent `fiscal_year_id/period_*` (4 callers Story 9-1) → fns bespoke `as_of` |
| Montage + RBAC | `crates/kesh-api/src/lib.rs:275`, `:559-561`, `:601-603`, `:755-817` | `comptable_routes` = `require_comptable_role` ; précédent échéancier CSV Comptable+ |
| CSV rapport | `crates/kesh-report/src/csv.rs:316-376` | `render_vat_report_csv` : BOM+`;`+CRLF, `format_amount_iso`, court-circuit vide |
| Format formatage | `crates/kesh-report/src/pdf.rs:160-189` | `format_swiss_amount` / `format_swiss_date` (si besoin) |
| `From<ReportError>` | `crates/kesh-api/src/errors.rs:748-798` | `CsvGeneration`→500 `CSV_GENERATION_FAILED` (auto) |
| Seed test | `crates/kesh-db/src/test_fixtures.rs:80` | `seed_accounting_company` → `SeededCompany { company_id, fiscal_year_id, accounts, … }` |
| Parité TTC (modèle) | `crates/kesh-db/tests/invoice_ttc_parity.rs:31-141` | test 4-voies à répliquer |

### Contrats frontend (ground-truth)

| Élément | Emplacement | Note |
|---|---|---|
| Page reports (onglets) | `frontend/src/routes/(app)/reports/+page.svelte` | 7 onglets ARIA, DTO par `$state`, `generate()` sur clic, `genSeq` race-guard, **aucune synchro URL** (à ajouter D-7c) ; contrôles binaires `{#if !isProjectTab}` `:429-508` (→ 3 branches) |
| `TabId` (LOCAL) | `reports/+page.svelte:50` | `type TabId = ReportType \| 'project-expenses' \| 'project-return'` — étendre ICI, PAS `reports.types.ts`. Ne pas toucher `ReportType` (`reports.types.ts:113`) |
| `canManage` / auth | `reminders/+page.svelte:39-40` (patron) | `import { authState } from '$lib/app/stores/auth.svelte'` + `authState.currentUser?.role === 'Admin'\|'Comptable'` — 1er usage d'auth dans reports |
| Export CSV client (patron) | `due-dates/+page.svelte:251-282` (`onExportCsv`) | `apiClient.getBlob` + `<a download>` + filename `balance-agee-${today}.csv` |
| RBAC page | `reports/+page.ts:2` | tous rôles (Admin+Comptable+Consultation) |
| Contrôles conditionnels | `+page.svelte` (`isProjectTab`) | patron pour `isAgedTab` |
| Vue rapport (patron) | `features/reports/BalanceSheetView.svelte` | thead/tbody/tfoot, `formatReportAmount`, `formatSwissDate`, `isReportEmpty` |
| Wrappers API | `features/reports/reports.api.ts` | `apiClient.get` + `buildQuery` ; export `downloadReport`/`triggerDownload` (`getBlob`+`<a download>`) |
| Formatage CHF | `reports.api.ts:185-191` (`formatReportAmount`) | montants nus (pas de préfixe CHF) `font-mono text-right` |
| Drill-down `?contactId=` | `invoices/+page.svelte:76-86`, `invoices.api.ts:27,87` | déjà supporté — `/invoices?contactId={id}` |
| Liens croisés (patron) | `due-dates/+page.svelte:302-304`, `reminders/+page.svelte:261-263` | `<Button variant="outline" href=… data-testid=…>` + clé i18n |
| lint i18n | `frontend/scripts/lint-i18n-ownership.js` | `features/reports/` → clés `reports-*` uniquement |
| E2E reports | `frontend/tests/e2e/reports.spec.ts` | `toHaveCount(7)` → **8** ; `reports-export-pdf.spec.ts` (download.saveAs) |

### Project Structure Notes

**Nouveaux fichiers** :
- `crates/kesh-report/src/aged_receivables.rs`
- `crates/kesh-report/tests/aged_receivables.rs`
- `frontend/src/lib/features/reports/AgedReceivablesView.svelte`
- (éventuel) `crates/kesh-api/tests/aged_receivables_e2e.rs`

**Modifiés** :
- `crates/kesh-report/src/lib.rs` (réexports) + `csv.rs` (+`render_aged_receivables_csv`)
- `crates/kesh-api/src/routes/reports.rs` (2 handlers) + `lib.rs` (2 routes) + `errors.rs` (si nouvel variant, sinon réutilise)
- `frontend/src/lib/features/reports/{reports.types.ts,reports.api.ts,reports.api.test.ts}`
- `frontend/src/routes/(app)/reports/+page.svelte` (onglet + contrôles + `?tab=` sync + export)
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` + `reminders/+page.svelte` (liens croisés)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/tests/e2e/reports.spec.ts` (tab count + scénario aged)
- `CHANGELOG.md`

**Décompte** : ~5 modules (kesh-report, kesh-api, frontend/features/reports, frontend/routes {due-dates,reminders}, kesh-i18n). Au seuil de la règle de splitting — story cohérente (un rapport bout-en-bout). Si `validate` boucle > 4 passes sans converger, splitter (21-7a backend rapport+route+CSV / 21-7b frontend vue+onglet+URL sync+liens). Pas de nouveau crate, pas de migration.

### References

- [Source: `epic-21-echeances-relances.md` §E items 23/25, D10, D23 (Non échu + réconciliation), D24 (CSV Comptable+), L21-1 (tout-ou-rien), L21-7 (libellés génériques)]
- [Source: 21-2 (helper `invoice_total_ttc` #246 + constantes SQL), 21-6c (drill-down `?contactId=`, liens croisés échéancier↔Rappels, `overdueDate` promu `api-fixtures.ts`)]
- [Source: cartographie ground-truth 4 agents Explore 2026-07-19 — patron `balance_sheet.rs`/`csv.rs`, `reports.rs`+`lib.rs` RBAC, prédicat postes ouverts+TTC, frontend `/reports`+`?contactId=`]
- [Source: `CLAUDE.md#Test Locally First`, `#Review Iteration Rule`, `#Issue Tracking Rule` ; décisions Guy 2026-07-19 (vue tous-rôles, `?tab=` URL sync)]

## Change Log — validate

### Pass 1 (Sonnet, 2 reviewers : véracité citations + cohérence/sécurité, 2026-07-19) — 1 CRITICAL + 3 HIGH + 4 MEDIUM + LOW → patchés

Auteur spec : Opus. Panel orthogonal Sonnet. Tous les findings > LOW re-vérifiés `grep`/`Read` sur le code réel avant patch.

- **C1 (CRITICAL) — citation D10 fausse.** `invoices.rs:913-921` = gestion `OptimisticLockConflict` (re-query rows==0), **pas** D10. **Patch** : vraies réfs `invoices.rs:124-131` (doc `PausedFilter`) + `:326-333` (application, no-op par défaut).
- **H1 (HIGH) — helpers export/audit incompatibles `as_of`.** `build_export_response_with_locale`/`build_filename` (`reports.rs:980`/`:1007`) exigent `&ReportPeriod` + filename `{periodStart}_{periodEnd}` ; `emit_report_audit`/`emit_report_export_audit` exigent `fiscal_year_id/period_*` (4 callers Story 9-1). **Patch AC 7/8** : précédent « date unique » `exports.rs::export_global` (`build_global_filename` + `build_content_disposition`) + fns audit **bespoke** `emit_aged_receivables_(export_)audit(as_of)`.
- **H2 (HIGH) — `TabId` mal localisé + `ReportType` intouchable.** `TabId` est LOCAL (`+page.svelte:50`), absent de `reports.types.ts` ; ajouter `'aged-receivables'` à `ReportType` casserait `TYPE_SLUGS_FALLBACK`/`downloadReport`. **Patch AC 10/13** : étendre le `TabId` local, ne pas toucher `ReportType`.
- **H3 (HIGH) — wrappers export réutilisés non type-checkables.** `buildExportFilename('aged-receivables',…)` refuse un type hors `ReportType` + exige `{start,end}`. **Patch AC 11** : wrappers dédiés (`getAgedReceivables`/`downloadAgedReceivables`, patron `onExportCsv` échéancier, filename `balance-agee-${today}.csv`).
- **M1 (MEDIUM) — CSV param locale inexistant.** Les `render_*_csv` n'ont pas de param locale (en-têtes FR en dur). **Patch AC 6** : signature sans locale, en-têtes FR littéraux.
- **M2 (MEDIUM) — `validate_format` rejette `None`.** Incompatible « défaut csv ». **Patch AC 8** : garde de format dédiée.
- **M3 (MEDIUM) — bloc de contrôle binaire.** `{#if !isProjectTab}…{:else}` → 3 branches, le bloc aged ne branche pas les exports génériques. **Patch AC 13**.
- **M4 (MEDIUM) — `canManage` sans source.** `reports/+page.svelte` n'importe aucun auth. **Patch AC 13** : import `authState` + `authState.currentUser?.role`.
- **LOW** : `?tab=` lecture one-shot (risque boucle `$effect`) → patché AC 13 + piège #7 ; deep-link `?tab=project-*` documenté acceptable ; piège n°2 serde nuancé (les noms `days_1_to_30` camelCasent déjà correctement — rename explicite = renfort défensif, pas correctif obligatoire) ; `reports-generate` dead-key pré-existante notée (vraie clé `reports-button-generate`).

**~35 citations file:line vérifiées : 1 fausse (C1), le reste exact** (constantes TTC, patron rapport, CSV, routes/RBAC, prédicat postes ouverts, contraintes DB `contacts.name NOT NULL`/`line_total >= 0`, drill-down `?contactId=`, lint i18n, `toHaveCount(7)`). Contradictions internes : aucune. Cohérence planning (items 23/25, D10/D23/D24, L21-1/L21-7) : confirmée.

### Pass 2 (Haiku ×2, contexte frais, 2026-07-19) — 0 CRITICAL/HIGH → **CONVERGÉ**

Panel Haiku orthogonal aux patches Opus. **36 citations re-vérifiées `grep`/`Read` — les 4 correctifs structurels (C1/H1/H2/H3) confirmés exacts et sans régression.** Confirmation clé : `Decimal` Rust → **`string` JSON** (serde projet) → `string` TS — le DTO frontend en `string` (AC 10) est correct (résout l'incertitude LOW du reviewer cohérence).

- **Ground-truth (Haiku)** : 0 finding. Toutes les citations exactes, dont les réfs D10 corrigées, `build_global_filename` (`exports.rs:122`), `TabId` local vs `ReportType`, `validate_format` rejette `None`, `kesh-report → kesh-db` dep.
- **Cohérence (Haiku)** : READY-FOR-DEV. 1 seul point actionnable → **M-P2 (reclassé LOW)** : AC 19 « cross-tenant → 404/absence » ambigu (le `company_id` vient du JWT → réponse **200 + `rows` scopées**, jamais 404). **Patché** (seed 2 companies, assert scoping en 200). Reclassement documenté : pur libellé de test, zéro impact code/design, aucune régression possible → traité comme LOW. Les autres LOW (piège serde nuancé, `HAVING total<>0`, contact sans nom = invariant schéma, dead-key `reports-generate`) sont documentés/acceptables.

### Trend & décision — validate

**Pass 1 (Sonnet ×2) : 1 CRITICAL + 3 HIGH + 4 MEDIUM → Pass 2 (Haiku ×2) : 0 CRITICAL/HIGH (1 clarification de test reclassée LOW, patchée).** Critère d'arrêt atteint (0 > LOW), budget 2/8. Rotation orthogonale Sonnet→Haiku, tous orthogonaux à l'auteur Opus. Les findings Pass 1 re-vérifiés `grep` avant patch ; Pass 2 a re-confirmé les 4 correctifs structurels sans régression. **Spec scellée, prête pour `bmad-dev-story 21-7`.**

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M) — `bmad-dev-story`, 2026-07-19/20.

### Debug Log References

Backend E2E lancé en `KESH_TEST_MODE` (port 8181, MockMailer, DB `kesh_e2e`) selon `docs/testing.md`. Tests d'intégration `#[sqlx::test]` contre MariaDB dev (root, DB éphémères par test).

### Completion Notes List

- **T1** — `crates/kesh-report/src/aged_receivables.rs` : `AgedBucket` (renames serde explicites `notDue`/`days1To30`/…), `AgedReceivablesRow` (`#[serde(flatten)]` bucket), `AgedReceivables { asOf, rows, totals }`. `generate(pool, company_id, as_of)` : requête agrégée `GROUP BY c.id, c.name` avec `SUM(CASE WHEN DATEDIFF(?, due_date) …)` par bucket, TTC via `INVOICE_TTC_DERIVED_JOIN_SQL` (alias `lt`), `WHERE status='validated' AND paid_at IS NULL` (D-7e, sans exclusion `dunning_paused_at`), `HAVING total <> 0`, `as_of` bindé. Totaux sommés en Rust. Réexports `lib.rs`.
- **T2** — `render_aged_receivables_csv` (patron `render_vat_report_csv`, BOM+`;`+CRLF, en-têtes FR en dur, sans param locale), ligne « Total général », court-circuit rapport vide.
- **T3** — `get_aged_receivables` (JSON, `authenticated_routes` tous rôles) + `export_aged_receivables` (CSV, `comptable_routes` D24) dans `routes/reports.rs`. Validation format **dédiée** (`validate_aged_export_format`, défaut csv, pdf→400). Filename **date-unique** (`kesh-balance-agee-{company}-{asOf}.csv`, patron `export_global`) + `build_content_disposition` direct (PAS `build_export_response_with_locale`/`ReportPeriod`). Audit **bespoke** `emit_aged_receivables_(export_)audit(as_of)`. Montage `lib.rs` (2 routes, garde IDOR respectée).
- **T4** — `reports.types.ts` : `AgedBucketDto`/`AgedReceivablesRowDto`/`AgedReceivablesDto` (montants `string`, `ReportType` NON touché). `reports.api.ts` : wrappers dédiés `getAgedReceivables`/`downloadAgedReceivables` (patron `onExportCsv` échéancier, filename `balance-agee-${today}.csv`). 2 tests vitest.
- **T5** — `AgedReceivablesView.svelte` (namespace `reports-*`, table + total + empty-state + drill-down `?contactId=` + lien échéancier). `reports/+page.svelte` : `TabId` **local** étendu, onglet 8, `agedReceivables` state, bloc de contrôle en **3 branches** (`isProjectTab`/`isAgedTab`/`else`, l'aged ne branche pas les exports génériques), branche `generate()` (genSeq), export CSV dédié gaté `canManage` (import `authState`), **synchro URL `?tab=` lecture one-shot `onMount` + write `replaceState` dans `selectTab`**.
- **T6** — liens croisés : échéancier→balance âgée (`due-dates-link-aged`), Rappels→balance âgée (`reminders-link-aged`), balance âgée→échéancier (`reports-aged-link-due-dates`) + drill-down par contact.
- **T7** — 15 clés `reports-aged-*` + `due-dates-link-aged` + `reminders-link-aged` × 4 FTL (FR/DE/IT/EN), parité validée par `cargo test -p kesh-i18n`. Réutilise `reports-button-generate` / `reports-export-csv-button`.
- **T8** — **Backend** : `crates/kesh-report/tests/aged_receivables.rs` (3 tests : buckets+frontières 30/31/60/61/90/91 + réconciliation ligne/générale + parité TTC helper Rust + D10 suspendue incluse + paid/draft exclus + null-due=Non échu ; scoping company_id ; empty). `crates/kesh-api/tests/reports_e2e.rs` (+5 : vue Consultation 200 ; export Consultation 403 / Comptable 200 text/csv ; format pdf 400 ; scoping 2 companies). **Frontend** : `reports.api.test.ts` (+2 vitest) ; `reports.spec.ts` (onglet 7→8 + 4 scénarios : génération+drill-down+export Admin, deep-link `?tab=`, lien croisé échéancier, export absent Consultation).
- **T9** — Gate (voir Change Log). CHANGELOG `[Non publié] → Ajouté`. README inchangé (Epic 21 déjà 🚧). Manuels → 21-8.

**Découverte hors scope (tracée)** : bug #259 (rappel manuel daté d'aujourd'hui rejeté `REMINDER_DATE_IN_FUTURE` avant midi UTC — `ManualReminderDialog` 21-6b, `T12:00:00` codé en dur). Fait échouer l'E2E `reminders.spec.ts › rappel manuel` avant midi. **Non lié à 21-7** (la balance âgée ne touche pas les rappels manuels) — non corrigé ici (discipline de périmètre), issue bug ouverte.

### File List

**Nouveaux**
- `crates/kesh-report/src/aged_receivables.rs`
- `crates/kesh-report/tests/aged_receivables.rs`
- `frontend/src/lib/features/reports/AgedReceivablesView.svelte`

**Modifiés**
- `crates/kesh-report/src/lib.rs`, `crates/kesh-report/src/csv.rs`
- `crates/kesh-api/src/routes/reports.rs`, `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/tests/reports_e2e.rs`
- `frontend/src/lib/features/reports/reports.types.ts`, `reports.api.ts`, `reports.api.test.ts`
- `frontend/src/routes/(app)/reports/+page.svelte`
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`, `frontend/src/routes/(app)/invoices/reminders/+page.svelte`
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`
- `frontend/tests/e2e/reports.spec.ts`
- `CHANGELOG.md`

### Change Log — dev

**bmad-dev-story (Opus 4.8, 2026-07-19/20)** — T1→T9. Backend Rust (`kesh-report` + `kesh-api`) + frontend Svelte. **Aucune migration, aucune nouvelle table, aucun bump `min_required`.**

Gate (« Test Locally First ») :

| Check | Résultat |
|---|---|
| `cargo fmt --all -- --check` | OK |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 warning |
| `cargo build --workspace --all-targets` | OK (via clippy all-targets) |
| `cargo test -p kesh-report` | 67 unit + intégration (dont aged 3/3) |
| `cargo test -p kesh-api --test reports_e2e` | 33/33 (dont aged 5/5) |
| `cargo test -p kesh-i18n` | 21/21 (parité FTL 4 locales) |
| `cargo nextest run --workspace` | **1877 passed, 0 failed**, 4 skipped (gate backend complet vert) |
| `npm run check` | 0 erreur |
| `npm run lint-i18n-ownership` | PASS |
| `npm run test:unit` | 408/408 (reports.api +2 aged) |
| `npm run build` | ✓ |
| `npm run test:e2e reports.spec` | 12/12 (dont 4 aged + onglet 8) + échéancier régression OK |

Note E2E : `reminders.spec.ts › rappel manuel` échoue avant midi UTC = **bug pré-existant #259** (time-of-day, hors scope 21-7). Fanout-regression : nul (changements additifs/isolés, aucune requête ou struct partagée modifiée).

## Change Log — code review

### Pass 1 (Sonnet, panel Blind Hunter + Edge Case Hunter + Acceptance Auditor, 2026-07-20) — 1 HIGH + 1 MEDIUM + LOW → patchés / reclassé

Auteur du code : Opus. Panel orthogonal Sonnet (3 couches). Blind Hunter et Edge Case Hunter : **0 CRITICAL/HIGH/MEDIUM** (RBAC réel vérifié, buckets SQL corrects aux frontières, réconciliation testée, scoping OK, avoir/cancelled écartés par CHECK/status, D10 inclus, pas d'injection, pas de dérive TZ, export non-stale). Acceptance Auditor : 1 HIGH + 1 MEDIUM + LOW.

- **AA-1 (HIGH) — vitest `AgedReceivablesView` absent (AC 21).** La spec exige un test de rendu (lignes + total + empty-state) ; seuls les 2 wrappers `reports.api` étaient testés. **Patch** : `frontend/src/lib/features/reports/AgedReceivablesView.test.ts` (3 tests — lignes+total, drill-down `?contactId=`, empty-state), patron `ProjectExpensesView.test.ts`.
- **AA-2 (MEDIUM) — `reports-aged-title` / `reports-filename-aged-receivables` listées AC 17 mais absentes des FTL.** **Reclassé déviation documentée** (pas de patch) : les deux clés sont **inutilisées par construction** → les ajouter créerait des **dead-keys** (anti-pattern, cf. #255). `reports-aged-title` : la vue n'a pas de titre de section (l'en-tête affiche « Arrêté au {date} » via `reports-aged-as-of`, cohérent AC 12) ; `reports-filename-aged-receivables` : le filename est **codé en dur** `balance-agee-${today}.csv` (décision validate H3, AC 11) → le mécanisme `reports-filename-*`/`resolve_type_slug` n'est pas emprunté. Reliquat de rédaction de la liste AC 17.
- **LOW patchés** : (a) test unitaire direct `render_aged_receivables_csv` dans `csv.rs` (BOM + en-têtes FR + ligne total + court-circuit vide) — Blind+Auditor ; (b) cas `due_date` **futur** → bucket Non échu (`aged_future_due_date_is_not_due`) — Edge ; (c) variante export **200 Admin** (`aged_receivables_export_ok_for_admin`) — Auditor AC 19 ; (d) `#[serde(rename = "total")]` explicite — Auditor.
- **Dismiss (LOW)** : `?tab=` non-réactif à la navigation historique (choix documenté one-shot anti-boucle) ; flash cosmétique du tab défaut avant `onMount` sur deep-link ; helper filename inline vs extrait (style/DRY).

Gate post-patch : csv unit 2/2, aged intégration 4/4 (+future-due), reports_e2e aged 6/6 (+admin), vitest reports 28/28 (dont `AgedReceivablesView` 3/3). fmt OK.

### Pass 2 (Haiku, même panel 3 couches, diff aplati mono-commit, 2026-07-20) — 0 finding actionnable → **CONVERGÉ**

Diff unique aplati fourni à Haiku (mitigation CLAUDE.md — évite la confusion d'indexation multi-commit). Panel Haiku orthogonal aux patches Sonnet/Opus.

- **Blind Hunter** : 0 finding > LOW (« merger »). SQL buckets/binds, RBAC, injection, format, `?tab=` one-shot, gate export, tests — tout re-vérifié grep OK.
- **Acceptance Auditor** : **22/22 AC PASS**, 0 finding. Les 7 correctifs P1 re-vérifiés présents ; la déviation AA-2 (clés `reports-aged-title`/`reports-filename-aged-receivables` non ajoutées) **re-confirmée correcte** — `grep -rn` = **0 référence** dans tout le code (les ajouter serait des dead-keys).
- **Edge Case Hunter** : 2 MEDIUM + 1 LOW.
  - **2 MEDIUM (race `as_of = Utc::now()`) → DISMISS.** Ground-truth exact (les 2 handlers appellent `Utc::now()` indépendamment), mais **non-actionnable** : c'est la sémantique correcte d'un rapport *point-in-time* « arrêté à aujourd'hui » — franchir minuit UTC entre deux appels reflète fidèlement le jour courant (comportement identique à `balance_sheet` / échéancier `UTC_DATE()` / réconciliation). Le paramètre `as_of` explicite (reproductibilité inter-appels) est **reporté v2 (D-7a)**. Le CSV porte l'`asOf` dans son nom → un export cross-minuit reste un snapshot cohérent en soi.
  - **1 LOW → patché.** Message par défaut « Sélectionnez un exercice et cliquez sur Générer » affiché sur l'onglet aged non-généré (trompeur — pas d'exercice à choisir). Branche `{:else if isAgedTab}` dédiée + clé `reports-aged-instruction-generate` × 4 FTL. Gate : check 0 err, lint-i18n PASS, kesh-i18n 21/21, build ✓.

### Trend & décision — code review

**Pass 1 (Sonnet ×3) : 1 HIGH + 1 MEDIUM + LOW → Pass 2 (Haiku ×3) : 0 actionnable (2 MEDIUM `as_of` dismissed sémantique inhérente, 1 LOW patché).** Critère d'arrêt atteint (0 > LOW actionnable), budget 2/8. Rotation orthogonale Sonnet→Haiku, tous orthogonaux à l'auteur Opus. Blind+Edge Pass 1 avaient déjà 0 CRITICAL/HIGH/MEDIUM ; le HIGH (vitest manquant) + MEDIUM (clés) de l'Auditor P1 corrigés/reclassés, re-confirmés par l'Auditor P2. **Story done.**
