# Story 21.6a: Exposition de la suspension (backend + liste factures)

Status: review

<!-- Créée 2026-07-17 par bmad-create-story. Cartographie ground-truth par 4 agents Explore parallèles (routes/invoices.rs / repositories+entities / frontend liste factures / tests+conventions). Issue de la règle de splitting préventif appliquée à 21-6 le 2026-07-16 (> 5 modules + trou backend D10). Consomme le socle 21-5a (colonnes + endpoints pause/resume). Indépendante de 21-6b. Décisions Guy 2026-07-17 : filtre tri-état, i18n des nouvelles chaînes seulement (issue #255 pour le reste), badge sur la liste factures seulement. -->

## Story

En tant que **comptable d'une PME suisse**,
je veux **voir quelles factures ont leurs rappels suspendus, et pouvoir filtrer la liste des factures là-dessus**,
afin de **retrouver une facture que j'ai suspendue — aujourd'hui elle disparaît de la liste à rappeler sans que rien ne la signale nulle part, donc je ne peux plus jamais la réactiver**.

## Contexte

**Cette story ferme un défaut fonctionnel réel, pas un confort d'affichage.** La story 21-5a a livré la suspension (colonnes `invoices.dunning_paused_at` / `dunning_paused_note`, endpoints `PUT /dunning-pause` et `/dunning-resume`, exclusion SQL de la liste à rappeler) — mais **aucune surface de lecture n'expose l'état de suspension** :

- `dunning_eligibility.rs:88` exclut la facture suspendue de la liste à rappeler (`AND i.dunning_paused_at IS NULL`) ;
- aucun `GET` ne renvoie `dunning_paused_at` — vérifié par grep exhaustif sur `crates/kesh-api/src/` (11 occurrences, toutes classées) : le DTO de **réponse d'écriture** `DunningPauseResponse` (`dunning_reminders.rs:97-98`) et ses 4 sites de construction (`:215-216`, `:242-243`), des **gardes d'envoi** (`invoice_email.rs:466`, `:1022`), une fixture de test (`invoice_email.rs:1338-1339`), un commentaire (`dunning_reminders.rs:132`). **Aucune surface de lecture** ;
- `ListInvoicesQuery` (`routes/invoices.rs:107-128`) n'a aucun filtre `paused` (9 champs : search / status / contactId / dateFrom / dateTo / sortBy / sortDirection / limit / offset).

**Conséquence** : on suspend une facture, elle sort de la liste à rappeler, et **la seule façon de savoir qu'elle est suspendue est de la re-suspendre** (muter et lire la réponse). La décision **D10** du plan d'epic exige pourtant « badge « suspendu » + filtre en liste factures » et pose un **invariant anti-dissimulation** : une facture suspendue reste dans la balance âgée et l'échéancier, elle ne sort **que** de la liste « à rappeler ».

**Ce que 21-6a livre** : l'exposition en lecture (détail + liste), le filtre `paused`, le badge dans la liste.

**Prérequis** : 21-5a (done) — colonnes, entité, endpoints. **Aucune migration** dans cette story : les colonnes existent depuis `20260715000001_invoice_reminders.sql:62-63`. **Donc aucun bump `kesh_version_min_required`, aucune ligne à ajouter à `docs/migrations-idempotence-audit.md`** (54 migrations, compteur `migrations_upgrade_path.rs:76-79` inchangé).

### Décisions figées (Guy, 2026-07-17)

- **D-a1 — Filtre tri-état, défaut no-op.** `?paused=all|paused|not-paused`, défaut `all` (ne filtre rien). Calqué sur `PaymentStatusParam` (`routes/invoices.rs:695-713`), le précédent le plus proche : DTO route kebab-case → `From` → enum repo → `match` SQL. Permet aussi bien « voir les suspendues » que « masquer les suspendues ».
- **D-a2 — i18n des nouvelles chaînes seulement.** Le badge et le filtre passent par `i18nMsg` dans les 4 langues (namespace `invoice-paused-*`, vérifié libre : aucune occurrence de `paused`/`suspend` dans les 4 FTL). Le reste de la page reste en FR codé en dur. **Le câblage i18n de la page liste est l'issue #255** (les clés `invoice-col-*` / `invoice-filter-*` / `invoice-status-*` existent déjà, traduites, mais sont mortes) — **NE PAS l'entreprendre ici**.
- **D-a3 — Badge sur la liste factures seulement.** Périmètre littéral de D10. Le champ transite mécaniquement par l'alias `DueDateItemResponse` et sera donc présent sur le wire de `GET /invoices/due-dates`, mais **l'échéancier n'est pas retouché** (il l'est par 21-6c pour les liens croisés).

### Hors scope (garde-fous)

- **Toggle suspension côté UI** (bouton pause/reprise sur la fiche facture) → **21-6c**.
- **Page Rappels**, envoi, lot, modale → **21-6b**.
- **Historique des rappels sur la fiche facture**, compteur dashboard, liens croisés → **21-6c**.
- **Câblage i18n de la page liste** → **issue #255**.
- **Manuels utilisateur/admin** → **21-8** (la story E2E + doc de l'epic).
- **`isOverdue` manquant du type TS `InvoiceListItemResponse`** : le backend l'envoie déjà (`routes/invoices.rs:260`) mais `invoices.types.ts:86-103` ne le déclare pas — le champ arrive sur le wire et est ignoré. **Écart constaté, NON corrigé ici** (n'élargit pas le diff sans nécessité).

## Acceptance Criteria

### A. Backend — exposition sur le détail facture

1. **`InvoiceResponse`** (`crates/kesh-api/src/routes/invoices.rs:160-191`) gagne deux champs, insérés **après `emailed_to` (`:180`)** pour refléter l'ordre de l'entité : `dunning_paused_at: Option<NaiveDateTime>` et `dunning_paused_note: Option<String>`. Le `#[serde(rename_all = "camelCase")]` du struct (`:161`) suffit — **aucun attribut serde par champ** (le struct n'en a aucun). Wire : `dunningPausedAt` / `dunningPausedNote`.

2. **`InvoiceResponse::from_parts`** (`:206-239`, littéral `Self { … }` `:216-237`) pose les deux champs depuis l'`Invoice`. **Point de passage unique** : il n'existe **aucun `From<Invoice>`**, et les **7** handlers qui renvoient un `InvoiceResponse` passent tous par ce helper — `get_invoice:521`, `create_invoice:599`, `update_invoice:655`, `validate_invoice_handler:683`, `mark_invoice_paid_handler:944`, `unmark_invoice_paid_handler:966`, **plus `routes/invoice_email.rs:818`** (envoi de facture, hors `invoices.rs` — grep `-rn "InvoiceResponse::from_parts" crates/`) → **une seule modification de construction, aucun call-site à toucher**. *(Les `from_parts` de `supplier_invoices.rs` / `imported_supplier_invoices.rs` portent sur `SupplierInvoiceResponse` — type distinct, hors scope.)*

3. **Aucune requête supplémentaire** : `FIND_INVOICE_SCOPED_SQL` (`crates/kesh-db/src/repositories/invoices.rs:44-48`) projette **déjà** `dunning_paused_at, dunning_paused_note` (`:46`), et l'entité `Invoice` les porte déjà (`entities/invoice.rs:46` et `:49`). `from_parts` a les valeurs en main.

### B. Backend — exposition sur la liste (piège n°1)

4. **`InvoiceListItem`** (`crates/kesh-db/src/repositories/invoices.rs:187-205`, 15 champs) gagne `dunning_paused_at: Option<NaiveDateTime>` et `dunning_paused_note: Option<String>`. C'est **le struct de la liste**, distinct de `Invoice` — 21-5a avait explicitement refusé de l'étendre et différé l'exposition à cette story.

5. **Checklist FERMÉE des SELECT qui désérialisent vers `InvoiceListItem`** — **exactement 2 sites**, tous deux des **littéraux SQL dupliqués, non factorisés en constante** (contrairement à `FIND_INVOICE_SCOPED_SQL`) :
   - **`list_by_company_paginated`** — SELECT `crates/kesh-db/src/repositories/invoices.rs:539-545`
   - **`list_for_export`** — SELECT `crates/kesh-db/src/repositories/invoices.rs:1806-1810`

   Ajouter `i.dunning_paused_at, i.dunning_paused_note` **aux deux**. **L'échec est au runtime (`ColumnNotFound` / `1054 Unknown column`), pas à la compilation** — c'est exactement la classe de régression qui a coûté 56 tests rouges et un 500 sur la réconciliation à la story 21-5a (un 4e site `Invoice` planqué derrière `#[sqlx(flatten)]` dans `reconciliation.rs`, manqué par les 4 reviewers, attrapé seulement par le gate workspace complet).

   **NE PAS toucher les 4 sites qui mappent `Invoice`** — ils projettent déjà les colonnes et sont alignés : `FIND_INVOICE_SCOPED_SQL:44-48` (14 usages + `credit_notes.rs:224`), `delete` inline `:924-929`, `list_all_by_company` inline `:1838-1845`, `reconciliation.rs:46-48` (`INVOICE_COLUMNS`, consommée par `UnpaidInvoiceCandidate` + `#[sqlx(flatten)]`).

   **NE PAS toucher** `dunning_eligibility.rs:73-90` (mappe `CandidateRow`, pas `Invoice`).

6. **`InvoiceListItemResponse`** (`crates/kesh-api/src/routes/invoices.rs:243-264`) et son **`From<InvoiceListItem>`** (`:266-289`) gagnent les deux champs (`#[serde(rename_all = "camelCase")]` `:242`).

7. **Propagation assumée à l'échéancier** : `pub type DueDateItemResponse = InvoiceListItemResponse;` (`:744`) → `GET /api/v1/invoices/due-dates` renverra `dunningPausedAt`/`dunningPausedNote` sur le wire. **C'est voulu** (l'invariant D10 exige que la facture suspendue reste visible dans l'échéancier ; l'exposer y est cohérent). **Aucun affichage n'est ajouté à la page échéancier** (D-a3).

### C. Backend — filtre `paused`

8. **Enum repo `PausedFilter`** dans `crates/kesh-db/src/repositories/invoices.rs`, placé à côté de `PaymentStatusFilter` (`:115-122`) et calqué dessus (mêmes derives, `#[default]` sur la variante neutre) :
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
   pub enum PausedFilter {
       #[default]
       All,
       Paused,
       NotPaused,
   }
   ```

9. **`InvoiceListQuery`** (`:167-181`) gagne `paused: Option<PausedFilter>`. Le `#[derive(Default)]` porte l'ajout pour les call-sites en `..Default::default()`, mais les **constructions littérales** de `kesh-api` sont **compile-forcées** (E0063) et doivent être mises à jour :
   - `list_invoices` — `routes/invoices.rs:484-496` → `paused: params.paused.map(Into::into)`
   - `build_due_dates_query` — `routes/invoices.rs:846-865` → **`paused: None`** (voir AC 11)
   - `routes/invoices.rs:882-885` utilise `..query.clone()` → **immunisée**, ne pas y toucher.

10. **`push_where_clauses`** (`:215-324`) gagne un `match` sur `query.paused.unwrap_or_default()`, placé après le bloc `payment_status` (`:275-295`) et calqué dessus — **SQL littéral, aucun `push_bind`** (pas de valeur utilisateur) :
    ```rust
    match query.paused.unwrap_or_default() {
        PausedFilter::All => {}
        PausedFilter::Paused => {
            qb.push(" AND i.dunning_paused_at IS NOT NULL");
        }
        PausedFilter::NotPaused => {
            qb.push(" AND i.dunning_paused_at IS NULL");
        }
    }
    ```
    **Le filtre s'applique automatiquement au `COUNT` et au `SELECT`** : `push_where_clauses` est appelé deux fois dans `list_by_company_paginated` (COUNT `:532`, SELECT `:545`) → `total` et `items` partagent le même prédicat, par construction.

11. **INVARIANT D10 — le défaut ne doit rien filtrer.** Périmètre réel de `push_where_clauses`, vérifié par grep — **3 call-sites, et eux seuls** : `list_by_company_paginated` (COUNT `:532` + SELECT `:546`) et `list_for_export` (`:1812`).

    ⚠️ **`due_dates_summary` (`:604-676`) ne l'appelle PAS** : il construit sa propre clause `WHERE` inline (`:623`) et ne lit **ni `payment_status` ni `paused`**. Le **résumé** de l'échéancier est donc structurellement insensible au filtre, quelle que soit son implémentation. **Ne PAS y ajouter de `match paused`** — hors scope, et sans objet (D-a3).

    **Mais l'échéancier reste concerné par ses items** : `list_due_dates_handler` (`:872-895`) appelle **`list_by_company_paginated`** — en parallèle de `due_dates_summary` via `tokio::join!` — pour produire la liste elle-même. Le filtre `paused` s'y applique donc bel et bien. **`build_due_dates_query` (`:846-865`) DOIT poser explicitement `paused: None`** (→ `All` → no-op) : à défaut, les factures suspendues disparaîtraient de l'échéancier, **soit exactement le bug que cette story ferme, réintroduit par sa correction**. Verrouillé par le test de l'**AC 24(f)**.

    *(Précédent à imiter — le handler strippe déjà explicitement `payment_status: None` pour la query du résumé (`:878-882`), avec un commentaire de review : « rend l'invariant visible côté handler au lieu de dépendre uniquement du contrat repo ». Même esprit ici.)*

12. **DTO route `ListInvoicesQuery`** (`routes/invoices.rs:107-128`) gagne `#[serde(default)] pub paused: Option<PausedParam>` (tous les champs existants sont `Option<_>` + `#[serde(default)]`).

13. **DTO `PausedParam` + `impl From<PausedParam> for PausedFilter`**, calqué `PaymentStatusParam` (`:695-713`), avec `#[serde(rename_all = "kebab-case")]` → wire **`all` | `paused` | `not-paused`**. Une valeur inconnue est rejetée par serde → **400**. *(Note : trois conventions de casse cohabitent déjà dans ce fichier — `InvoiceSortBy` est PascalCase sur le wire, `PaymentStatusParam` kebab-case. On suit `PaymentStatusParam`, précédent d'un enum de **filtre**.)*

14. **Aucune route nouvelle** : `paused` est un query param de `GET /api/v1/invoices`, déjà monté (`lib.rs:599`, `authenticated_routes`). **L'AC anti-footgun routeur est donc sans objet ici** — aucune modification de `build_router` n'est attendue. Si le dev croit devoir ajouter une route, c'est le signe d'une dérive de périmètre.

15. **Aucune validation handler supplémentaire** : contrairement à `limit`/`offset`/`search`/`status`/`date_from..date_to` (validés manuellement dans `list_invoices:441-482`), `paused` est entièrement validé par serde via l'enum. Ne pas ajouter de branche de validation.

### D. Frontend — types et API

16. **`frontend/src/lib/features/invoices/invoices.types.ts`** :
    - `InvoiceResponse` (`:26-56`) et `InvoiceListItemResponse` (`:86-103`) gagnent `dunningPausedAt: string | null` et `dunningPausedNote: string | null`.
    - `ListInvoicesQuery` (`:170-180`) gagne `paused?: 'all' | 'paused' | 'not-paused'`.

17. **`buildQueryString`** (`frontend/src/lib/features/invoices/invoices.api.ts:23-36`) sérialise `paused`, **omis si absent ou `'all'`** (cohérent avec la convention `syncUrl` « on n'écrit que le non-défaut »).

### E. Frontend — badge

18. **Composant `frontend/src/lib/features/invoices/DunningPausedBadge.svelte`**, calqué **`PaymentStatusBadge.svelte`** (le seul badge factures ; **il n'existe pas de `Badge.svelte` générique ni de système de variants**) :
    - `$props()` typé : `pausedAt: string | null`, `note?: string | null`.
    - Libellé : `i18nMsg('invoice-paused-badge', 'Suspendu')`.
    - **a11y** : `aria-label={label}` — l'état ne doit pas reposer sur la seule couleur (patron `PaymentStatusBadge.svelte:33`).
    - La **note** de suspension, si présente, est rendue en `title` (infobulle) — c'est la seule surface qui l'expose en v1 ; le toggle UI viendra en 21-6c.
    - Style : `color-mix(in srgb, var(--color-text-muted) 20%, transparent)` — **teinte neutre volontaire**, à distinguer de `overdue` (`--color-warning`) : une suspension est une décision délibérée, pas une alerte.

19. **`lint-i18n-ownership` — piège n°4, gate bloquant.** Le composant vit sous `features/invoices/` (**pluriel**) et consomme une clé `invoice-paused-badge` (**singulier**) → `keyBelongsToFeature()` (`frontend/scripts/lint-i18n-ownership.js:149-158`) lève une **violation** et `process.exit(1)`. C'est le piège connu **#30**, déjà subi par `MarkPaidDialog.svelte` et `SendEmailDialog.svelte`. **Action requise** : ajouter `DunningPausedBadge.svelte` à `KNOWN_VIOLATIONS` (`lint-i18n-ownership.js:22-114`) avec un commentaire `refs #30`, cohérent avec le précédent. **`npm run lint-i18n-ownership` DOIT être vert au gate.**

20. **Rendu dans la liste** : le badge s'affiche dans la **colonne Statut** (`frontend/src/routes/(app)/invoices/+page.svelte:360`), à côté du texte de statut existant, **conditionné à `inv.dunningPausedAt !== null`**. *(La page n'a aujourd'hui aucun badge — le statut est du texte brut via `statusLabel():230-239`. Ce badge est le premier.)*

### F. Frontend — filtre et URL

21. **Contrôle de filtre** : `<select id="invoice-paused-filter">` ajouté à côté du filtre statut (`+page.svelte:265-277`), en **`bind:value`** (le filtre statut est le seul contrôle de la page en `bind:value` — on suit ce patron, pas celui des `oninput` manuels). Trois options, libellés via `i18nMsg` : `invoice-paused-filter-all` / `-paused` / `-not-paused`. Toute sélection remet **`offset = 0`** via un `onchange` dédié.

    ⚠️ **Rectificatif (code review P1, AA-2)** : la parenthèse « convention de tous les filtres de la page » était **fausse**. Vérifié (`grep -n "offset = 0"` → 6 sites) : seuls les filtres **texte / date / contact** remettent l'offset à zéro ; le `<select>` du **statut ne le fait pas**. La remise à zéro reste **exigée ici** — elle est correcte (sans elle, changer de filtre depuis la page 3 laisse un offset survivant à un jeu de résultats rétréci → liste vide trompeuse) — mais elle est un **écart volontaire au précédent `statusFilter`**, pas un alignement sur lui. Le défaut symétrique de `statusFilter` est **pré-existant, hors scope** → issue **#257**.

22. **Synchro URL — piège n°3.** La page a une synchro URL **bidirectionnelle** :
    - **`initFromUrl()`** (`:52-84`) lit `paused` et le **valide contre une whitelist** `VALID_PAUSED` à ajouter à côté de `VALID_STATUS` / `VALID_SORT_BY` / `VALID_SORT_DIR` (`:48-50`). Valeur invalide → défaut `all` (patron existant). `mounted = true` doit rester **en dernier** (`:83`).
    - **`syncUrl()`** (`:93-125`) écrit `paused` **seulement s'il diffère de `'all'`** (convention « on n'écrit que le non-défaut » : `if (sortBy !== 'Date')`, `if (limit !== 20)`…).
    ⚠️ **`syncUrl` construit les params dans DEUX branches** (`:98-113` la branche `dateRangeError` qui omet les dates, et la branche nominale). **Le param `paused` doit être ajouté aux DEUX** — l'oublier dans la branche `dateRangeError` produit une perte de filtre silencieuse dès qu'une plage de dates est invalide.

23. **Bouton « Réinitialiser »** (`:313-326`) : **ne pas le toucher**. Il ne reset aujourd'hui ni `search` ni `statusFilter` (seulement contact + dates) ; y ajouter `paused` créerait une incohérence de comportement. Statu quo assumé.

### G. Tests

24. **Backend — étendre `crates/kesh-api/tests/dunning_reminders_e2e.rs`** (et non créer un fichier : il n'existe **aucun `invoices_e2e.rs`**, les tests factures sont éclatés par sujet, et ce fichier porte déjà les helpers de suspension). Patron : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`, fichier auto-porteur (`spawn_app`, `forge_jwt`, seeds locaux `create_company:109`, `create_contact:151`, `create_fiscal_year:163`, `validated_invoice:174`, `invoice_version:206`). Suspendre une facture : lire `version` via `invoice_version()` puis `PUT /api/v1/invoices/{id}/dunning-pause` avec `json!({ "version": v0, "note": "…" })` (patron `:295-303`).

    Scénarios **obligatoires** :
    - **(a)** `GET /api/v1/invoices/{id}` d'une facture suspendue → `dunningPausedAt` non-null **et** `dunningPausedNote` = la note ; d'une facture non suspendue → les deux `null`.
    - **(b)** `GET /api/v1/invoices` → les items portent `dunningPausedAt` (non-null pour la suspendue).
    - **(c)** `GET /api/v1/invoices?paused=paused` → **seule** la facture suspendue.
    - **(d)** `GET /api/v1/invoices?paused=not-paused` → la facture suspendue **absente**.
    - **(e)** `GET /api/v1/invoices` **sans param** → les deux factures présentes (défaut no-op, AC 11).
    - **(f)** **INVARIANT D10 (test anti-régression clé)** : `GET /api/v1/invoices/due-dates` retourne **toujours** la facture suspendue — le partage de `push_where_clauses` ne doit pas la filtrer.
    - **(g)** `total` de la réponse paginée cohérent avec `items.len()` sous filtre `paused=paused` (le COUNT partage le prédicat).
    - **(h)** `?paused=bogus` → **400**.

25. **Frontend — vitest** : étendre `frontend/src/lib/features/invoices/invoices.api.test.ts` (patron : `vi.mock('$lib/shared/utils/api-client')` puis `expect(apiClient.get).toHaveBeenCalledWith('/api/v1/…')`) — `buildQueryString` inclut `paused=paused`, et **l'omet** quand la valeur vaut `'all'` ou est absente.

26. **E2E Playwright** — étendre `frontend/tests/e2e/invoices.spec.ts`. **Ajouter des `data-testid`** (`invoice-paused-badge`, `invoice-paused-filter`) : la page liste n'en a qu'un aujourd'hui (`invoice-create-button:248`) et le spec sélectionne par rôle/texte, mais le patron `dunning.spec.ts` (21-4) est **full `getByTestId`** — on suit le plus récent. Scénario : créer un contact + une facture validée via les helpers API (`helpers/api-fixtures.ts` : `createContactWithAddressViaApi`, `createAndValidateInvoiceViaApi`) → suspendre via l'API → la liste affiche le badge → `paused=not-paused` la masque → `paused=paused` la montre seule.

27. **axe** : le test a11y existant (`invoices.spec.ts:88-99`) n'exerce que l'**empty state** — son commentaire (`:88-91`) demande explicitement de l'étendre aux badges une fois le seed en place. L'étendre à la liste **peuplée**. ⚠️ **Leçon 21-4** : un axe full-page sur une liste peuplée peut révéler des dettes a11y **pré-existantes** hors périmètre (la 21-4 a ainsi découvert le contraste des chips → issue #253). Si c'est le cas : **scoper** via `.include()` sur le conteneur du tableau (patron `reports.spec.ts:156,173`, `email-templates.spec.ts:201`) et documenter — **ne corriger aucune dette a11y pré-existante dans cette story**.

28. **Aucun test de forme de réponse à craindre** : vérifié — **zéro snapshot dans tout le repo** (`insta` / `assert_json_snapshot` / `toMatchSnapshot` → 0 hit), aucun `deny_unknown_fields` sur un DTO invoice, aucune assertion de compte de clés JSON sur une réponse facture (les assertions sont champ par champ). **Ajouter deux champs ne casse aucun test existant.** Les compteurs figés du repo sont tous au niveau **table/migration** (`exports_global_e2e.rs:619,696`, `migrations_upgrade_path.rs:76-79`) — cette story n'ajoute ni table ni migration, ils restent inchangés.

### H. Gate & documentation

29. **Gate local complet** (règle « Test Locally First », CLAUDE.md) :
    ```sh
    cargo fmt --all -- --check
    cargo build --workspace --all-targets
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace -- --test-threads=1     # kesh-db touché → serial
    cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
    cd frontend && npm run test:e2e                # PAS dans la CI → d'autant plus critique
    ```
    ⚠️ **Le gate workspace COMPLET est obligatoire, pas les suites ciblées.** C'est la leçon frontale de 21-5a : la régression cross-crate (`reconciliation.rs`) n'était visible que là — ni les 4 reviewers ni les tests par tâche ne l'ont vue. Cette story touche la même classe de code (un struct `FromRow` + ses SELECT).
    ⚠️ La **DB dev doit être migrée** (`sqlx migrate run`) avant les tests d'intégration `#[tokio::test]` du repo `invoices`, qui tapent la DB persistante — piège rencontré en 21-5a (faux échecs `1054 Unknown column`).

30. **CHANGELOG** : entrée sous `[Non publié]` → `Added` (badge et filtre « suspendu » dans la liste des factures ; état de suspension exposé sur l'API facture). **Manuels utilisateur/admin → 21-8** (story doc + E2E de l'epic). **README** : aucun changement (Epic 21 déjà 🚧 En cours, aucune feature de la section « Fonctionnalités » ne change d'état).

## Tasks / Subtasks

- [x] **T1 — Exposition détail facture** (AC: 1, 2, 3)
  - [x] Ajouter les 2 champs à `InvoiceResponse`. **Micro-déviation assumée** : placés après `project_id` (et non « après `emailed_to` » comme le disait l'AC 1) — l'ordre de l'entité `Invoice` est `emailed_to, project_id, dunning_paused_at, dunning_paused_note`, donc c'est ce placement qui sert réellement l'intention déclarée de l'AC (« refléter l'ordre de l'entité »). Sans effet fonctionnel (ordre des clés JSON seulement).
  - [x] Les poser dans le littéral de `from_parts`. Confirmé : **aucun** des 7 call-sites ne bouge.
- [x] **T2 — Exposition liste + fanout SELECT** (AC: 4, 5, 6, 7) — **piège n°1**
  - [x] Ajouter les 2 champs à `InvoiceListItem` + doc-comment d'avertissement sur le fanout (le prochain qui ajoute un champ doit voir le piège depuis le struct).
  - [x] Ajouter les 2 colonnes aux **2** SELECT. **Les deux listes de colonnes étant des littéraux strictement identiques, le remplacement a été fait en une seule opération globale** (`replace_all`) après vérification qu'exactement 2 occurrences existent dans le workspace (`grep -cF` → 2, lignes 554 et 1820) → l'oubli d'un site est structurellement impossible.
  - [x] Ajouter les 2 champs à `InvoiceListItemResponse` + son `From`.
  - [x] **Re-grep de contrôle** : `grep -rn "InvoiceListItem" crates/ --include=*.rs` → 2 sites de désérialisation (`:564`, `:1820`), aucun 3e. `SupplierInvoiceListItem` = type distinct.
- [x] **T3 — Filtre `paused` backend** (AC: 8-15)
  - [x] `PausedFilter` posé à côté de `PaymentStatusFilter`, doc-comment expliquant *pourquoi* `All` est un no-op délibéré.
  - [x] Champ `paused` sur `InvoiceListQuery` + les 2 constructions littérales. **`cargo build --workspace --all-targets` exit 0** → aucune 3e construction littérale manquée (la `:928` est bien immunisée par `..query.clone()`).
  - [x] `match` dans `push_where_clauses` (SQL littéral, aucun `push_bind`).
  - [x] `PausedParam` + `From` (kebab-case) + champ sur `ListInvoicesQuery`.
- [x] **T4 — Tests backend** (AC: 24) — 4 tests, scénarios (a)…(h), dont **(f) l'invariant D10**. `dunning_reminders_e2e` **10/10 vert** (6 pré-existants + 4 nouveaux).
- [x] **T5 — Frontend types, API, badge, filtre** (AC: 16-23)
  - [x] Types (`InvoiceResponse`, `InvoiceListItemResponse`, `ListInvoicesQuery`, nouveau `PausedFilter`) + `buildQueryString` (omet `all` et l'absence).
  - [x] `DunningPausedBadge.svelte` + entrée `KNOWN_VIOLATIONS` (**piège n°4**) → `npm run lint-i18n-ownership` **PASS**.
  - [x] Clés i18n `invoice-paused-*` (5 clés × **4 FTL**) : badge, libellé du filtre, 3 options.
  - [x] Rendu colonne Statut (conditionné à `dunningPausedAt !== null`) + `<select>` filtre en `bind:value`.
  - [x] `VALID_PAUSED` + `initFromUrl` + `syncUrl` **dans les 2 branches** (**piège n°3**) + `pausedFilter` ajouté aux dépendances de l'`$effect`.
- [x] **T6 — Tests frontend + E2E + axe + doc** (AC: 25, 26, 27, 30) — vitest **394** (+5), E2E **14/14**, CHANGELOG. **L'axe sur liste peuplée a attrapé un défaut réel de cette story** (badge à 3.69:1, cf. Dev Agent Record) + une dette a11y pré-existante → **issue #256**, test scopé.
- [x] **T7 — Gate complet** (AC: 29) — `fmt` OK, `clippy` **0**, `cargo test --workspace -- --test-threads=1` **exit 0 : 101 suites / 1872 tests / 0 échec**. Frontend `check` 0 erreur / `lint-i18n` PASS / `test:unit` 394 / `build` OK. E2E **14/14**.

## Dev Notes

### Les 4 pièges, par ordre de coût

1. **Fanout SELECT `InvoiceListItem` (runtime, pas compile-time)** — ajouter un champ au struct `FromRow` sans l'ajouter aux **2** SELECT littéraux → `ColumnNotFound` à l'exécution. Les deux SELECT sont des **copies dupliquées**, non factorisées en constante. 21-5a a payé cette exacte erreur : 56 tests rouges, 500 sur la réconciliation, invisible pour 4 reviewers, attrapé par le seul gate workspace complet. **Le compilateur ne vous aidera pas ici.**
2. **Défaut du filtre qui casse l'invariant D10** — `push_where_clauses` est partagé par la liste, son COUNT et l'export CSV (**3 call-sites — pas `due_dates_summary`, qui a son `WHERE` propre**). L'échéancier reste néanmoins concerné : **ses items passent par `list_by_company_paginated`**. Si le défaut n'était pas un no-op, ou si `build_due_dates_query` omettait `paused: None`, les factures suspendues disparaîtraient de l'échéancier — le bug même que la story ferme, réintroduit par sa correction. Test **AC 24(f)** obligatoire.
3. **`syncUrl` a deux branches** (`+page.svelte:98-113` et nominale) qui dupliquent la construction des params. Oublier `paused` dans la branche `dateRangeError` = perte de filtre silencieuse.
4. **`lint-i18n-ownership` bloque le gate** sur `invoice-*` dans `features/invoices/` (piège #30). L'entrée `KNOWN_VIOLATIONS` n'est pas optionnelle.

### Leçon de review héritée (21-5b — à appliquer dès le dev)

**Un patch de review vient AVEC son test.** La 21-5b a convergé en **5 passes** parce que chaque passe trouvait des défauts *introduits par la remédiation de la passe précédente*, et le compte ne baissait pas. La boucle ne s'est refermée qu'à partir du moment où chaque patch était livré avec son test (P4 : 4 tests ; P5 : plus rien). Corollaire ici : tout correctif appliqué en code-review sur cette story doit être accompagné du test qui le verrouille.

### Conventions vérifiées (ground-truth)

**Wire / casse** — trois conventions cohabitent dans `routes/invoices.rs` : `InvoiceSortBy` et `SortDirection` sont **PascalCase** sur le wire (aucun `rename_all` sur les enums, `repositories/invoices.rs:89-97` et `kesh-core/src/listing/mod.rs:17-23`), `PaymentStatusParam` est **kebab-case** (`:695-702`). On suit `PaymentStatusParam` : c'est le précédent d'un enum de **filtre**, la même nature que `paused`.

**Audit** — **rien à ajouter** : la suspension est déjà auditée par 21-5a (`invoice.dunning_paused` / `invoice.dunning_resumed`, `repositories/invoices.rs:1654-1673`). Cette story est en **lecture seule** côté mutation ; elle ne pose aucune entrée d'audit. *(Note d'observation : `set_dunning_pause` utilise `NewAuditLogEntry::user(...)` directement, kesh-db ne pouvant pas dépendre de kesh-api — l'attribution PAT n'est donc pas propagée sur ce chemin. Constat, hors scope.)*

**RBAC** — `GET /api/v1/invoices` est monté dans `authenticated_routes` (`lib.rs:599`) : lecture tous rôles authentifiés, scoping tenant par `current_user.company_id` (`routes/invoices.rs:499`) + check défensif `get_company_for` (`:439`). Le filtre `paused` n'introduit **aucune donnée nouvelle** au-delà de ce que le rôle voit déjà — pas d'élévation de surface.

**Seeds de test** — `validated_invoice` (`dunning_reminders_e2e.rs:174`) insère en listant explicitement ses colonnes : **les colonnes ajoutées héritent du `DEFAULT NULL`**, aucun helper ne casse. Le seul helper partagé du crate (`tests/common/mod.rs`, 1 fonction) ne crée ni contact ni facture → **ne pas le toucher**.

### Constat hors scope (à signaler en revue, ne pas corriger ici)

`serialize_invoices_csv` (`crates/kesh-api/src/exports/csv_tables.rs:394-436`) a une liste de **15 colonnes explicites** qui omet `dunning_paused_at`/`dunning_paused_note` — **mais aussi `emailed_at`/`emailed_to`**. C'est donc une **lacune pré-existante de l'export de souveraineté**, pas une régression de cette story, et aucun test n'assert son compte de colonnes. À trancher en rétrospective d'epic (catégorie A ou B), pas ici.

### Project Structure Notes

**Backend** (`crates/`) — 2 fichiers :
- `kesh-api/src/routes/invoices.rs` — `InvoiceResponse`, `from_parts`, `InvoiceListItemResponse` + `From`, `ListInvoicesQuery`, `PausedParam` + `From`, construction de `InvoiceListQuery` (×2).
- `kesh-db/src/repositories/invoices.rs` — `InvoiceListItem`, 2 SELECT, `PausedFilter`, `InvoiceListQuery`, `push_where_clauses`.

**Frontend** (`frontend/`) — 4 fichiers + 4 FTL :
- `src/lib/features/invoices/invoices.types.ts`, `invoices.api.ts`, `DunningPausedBadge.svelte` (nouveau)
- `src/routes/(app)/invoices/+page.svelte`
- `scripts/lint-i18n-ownership.js` (entrée `KNOWN_VIOLATIONS`)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`

**Tests** — `crates/kesh-api/tests/dunning_reminders_e2e.rs`, `frontend/src/lib/features/invoices/invoices.api.test.ts`, `frontend/tests/e2e/invoices.spec.ts`.

**Décompte modules** : 2 backend + 2 frontend (feature `invoices` + page) = **4 modules de premier niveau — sous le seuil de 5** de la règle de splitting préventif. Aucune migration, aucun nouveau crate, aucune route.

### References

- [Source: `_bmad-output/planning-artifacts/epic-21-echeances-relances.md#Décisions figées` — D10 (suspension, invariant anti-dissimulation, badge + filtre), item 22 (RBAC), L21-8]
- [Source: `_bmad-output/planning-artifacts/epic-21-echeances-relances.md#Découpage en stories` — 21-6a, encadré de splitting du 2026-07-16]
- [Source: `_bmad-output/implementation-artifacts/21-5a-donnees-eligibilite-relances.md` — colonnes, endpoints pause/resume, différé explicite de l'exposition liste vers 21-6, régression `reconciliation.rs` (Change Log)]
- [Source: `_bmad-output/implementation-artifacts/21-5b-envoi-rappels-email.md` — leçon « un patch de review vient avec son test » (5 passes)]
- [Source: `CLAUDE.md#Test Locally First`, `#Review Iteration Rule`, `#Migration breaking policy` (P5 sans objet — pas de migration), `#Issue Tracking Rule`]
- [Source: GitHub #231 (rappels débiteurs — cette story en ferme la surface « suspension visible »), #255 (dette i18n page liste, ouverte par cette cartographie), #253 (dette a11y contraste, Epic 20), #30 (piège ownership i18n `invoice-*` / `invoices/`)]

## Change Log — validate

### Pass 1 (Sonnet, 2026-07-17) — 0 CRITICAL / 1 HIGH / 1 MEDIUM / 1 LOW → patchés

Auteur de la spec : Opus. Reviewer orthogonal : Sonnet. Verdict **GO-ajusté**. Les 2 findings > LOW ont été **re-vérifiés ground-truth par l'orchestrateur** (`grep -n`) avant patch, conformément à la discipline projet — tous deux **confirmés réels**.

- **H1 — l'invariant D10 était juste, sa justification était fausse.** L'AC 11 affirmait que `push_where_clauses` est partagé par `list_by_company_paginated`, `due_dates_summary` **et** `list_for_export`. Grep : **3 call-sites seulement** (`:532`, `:546`, `:1812`) — `due_dates_summary` (`:604-676`) construit sa **propre** clause `WHERE` inline (`:623`) et ne lit ni `payment_status` ni `paused`. L'invariant tenait donc, mais par un mécanisme différent de celui décrit — une fausse prémisse d'architecture partagée sur laquelle un mainteneur aurait pu s'appuyer.
  **Vérification complémentaire de l'orchestrateur** (au-delà du finding) : `list_due_dates_handler` (`:872-895`) appelle `list_by_company_paginated` **et** `due_dates_summary` en `tokio::join!` → **les items de l'échéancier passent bien par `push_where_clauses`**. L'exigence « `build_due_dates_query` pose `paused: None` » reste donc **load-bearing**, et le test AC 24(f) garde tout son sens. **Patch** : AC 11 et Dev Notes piège n°2 réécrits avec le périmètre exact, la distinction résumé/items, la garde « ne pas ajouter de `match paused` dans `due_dates_summary` », et le précédent du strip explicite de `payment_status` (`:878-882`).
- **M1 — checklist « fermée » incomplète** : l'AC 2 annonçait « les 6 handlers » passant par `from_parts` ; il y en a **7** — le site `routes/invoice_email.rs:818` (envoi de facture, hors `invoices.rs`) manquait. Sans impact d'implémentation (`from_parts` reste le point de passage unique, aucun call-site à toucher), mais une checklist présentée comme exhaustive et fausse sape la confiance dans le reste du document. **Patch** : AC 2 corrigée à 7 sites + note distinguant les `from_parts` de `SupplierInvoiceResponse` (type distinct).
- **L1 — comptage inexact** : « les 8 occurrences » de `dunning_paused` dans `kesh-api/src` → **11** en réalité. **Patch** : les 11 classées nommément dans le Contexte.
- **Défaut interne trouvé à l'application des patches** (non remonté par le reviewer) : l'AC 11 renvoyait au « test de l'AC 19(f) » — numérotation périmée d'une version antérieure de la spec, la matrice de tests est l'**AC 24**. Corrigé.

Vérifications positives (grep/Read, Sonnet) : checklist des **2** SELECT `InvoiceListItem` **fermée et exacte** ; les 4 sites `Invoice` déjà alignés (dont `reconciliation.rs` post-régression 21-5a) ; littéraux `InvoiceListQuery` hors `kesh-api` tous protégés par `..Default::default()` ; namespace `invoice-paused-*` libre dans les 4 FTL ; comportement de `lint-i18n-ownership.js` et piège #30 confirmés ; compteurs figés migrations/export inchangés ; absence de snapshot / `deny_unknown_fields` confirmée.

### Pass 2 (Haiku, 2026-07-17) — 0 finding → **CONVERGÉ**

Contexte frais, prémunie explicitement contre le mode d'échec documenté (« auditer la spec comme une implémentation » — 6 erreurs de catégorie sur 7 findings lors d'un précédent validate). **Le garde-fou a tenu : 0 erreur de catégorie, 0 hallucination.** 73 outils, sampling agressif des chemins:lignes. Verdict **GO**.

Absorption des patches P1 confirmée par le reviewer (AC 11 réécrite, AC 2 à 7 sites, renvoi AC 24(f) corrigé). Vérifications indépendantes concordantes : `due_dates_summary:604-676` a bien sa clause `WHERE` propre ; `list_due_dates_handler:872-895` appelle `list_by_company_paginated` via `tokio::join!` ; les 2 branches de `syncUrl` ; namespace `invoice-paused-*` libre dans les 4 FTL ; piège #30 confirmé.

**Contre-vérification de l'orchestrateur sur le piège n°1** (le point où la story se joue, re-vérifié malgré le GO) — `grep -rn "InvoiceListItem" crates/ --include=*.rs` : **exactement 2 sites de désérialisation** (`repositories/invoices.rs:564` `list_by_company_paginated`, `:1820` `list_for_export`), plus la définition (`:187`), le champ `InvoiceListResult.items` (`:209`) et le `From` API (`routes/invoices.rs:267`). `SupplierInvoiceListItem` est un **type distinct** (`repositories/supplier_invoices.rs:47`), hors scope. **Checklist fermée de l'AC 5 confirmée exacte.**

### Trend & décision

**Passe 1 (Sonnet) : 1 HIGH + 1 MEDIUM + 1 LOW → Passe 2 (Haiku) : 0.** Critère d'arrêt de la règle de remédiation atteint (0 finding > LOW), budget 2/8. Rotation orthogonale à l'auteur (Opus) sur les deux passes. Les 2 findings > LOW de P1 ont été re-vérifiés ground-truth par l'orchestrateur avant patch — aucun n'était un faux-positif. **Spec scellée, prête pour `bmad-dev-story 21-6a`.**

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M context) — run unique, T1 → T7, aucune HALT.

### Debug Log References

- Gate workspace complet (serial) : **exit 0**, 101 suites / 1872 tests / 0 échec / 8 ignorés (`#[ignore]` pré-existants).
- Suites de la classe de risque (fanout `InvoiceListItem` / `push_where_clauses`) toutes vertes : `reconciliation_e2e` 25, `reconciliation_repository` 9, `exports_global_e2e` 20, `admin_full_export_e2e` 7, `invoice_echeancier_e2e` 9, `dunning_reminders_e2e` 10, `invoice_send_email_e2e` 39, `migrations_upgrade_path` 8 (54 migrations, compteur inchangé comme prévu).
- E2E Playwright `invoices.spec.ts` : 14/14 (backend 0.7.0 contre `kesh_e2e`, build statique).

### Completion Notes List

**Les 4 pièges de la spec ont tous été rencontrés ; aucun n'a mordu.**

1. **Fanout SELECT `InvoiceListItem` (piège n°1) — neutralisé par construction.** Les 2 SELECT étant des littéraux **strictement identiques**, j'ai vérifié par `grep -cF` qu'il existe exactement 2 occurrences dans le workspace, puis fait un **remplacement global unique**. L'oubli d'un site devenait alors impossible, au lieu de dépendre de ma vigilance sur une checklist. `exports_global_e2e` 20/20 prouve à l'exécution que le SELECT de `list_for_export` (le site le plus facile à oublier) est correct. Un doc-comment d'avertissement est posé **sur le struct** : le prochain qui y ajoute un champ verra le piège sans avoir lu cette story.
2. **Invariant D10 (piège n°2) — tenu et verrouillé.** `build_due_dates_query` pose `paused: None` avec un commentaire qui nomme l'invariant. Le test `paused_invoice_stays_visible_in_due_dates` mord réellement : il asserte que la facture suspendue sort de la liste à rappeler **et reste** dans l'échéancier.
3. **`syncUrl` à 2 branches (piège n°3)** — `paused` ajouté aux deux, avec commentaire dans la branche `dateRangeError` (la plus facile à oublier).
4. **`lint-i18n-ownership` (piège n°4)** — entrée `KNOWN_VIOLATIONS` ajoutée, gate PASS.

**Défaut réel trouvé par le test axe — dans le code de cette story (a11y).** Le badge affichait un contraste de **3.69:1** (mesuré par axe-core), sous le minimum AA de 4.5:1. **Cause racine : le gabarit copié.** `PaymentStatusBadge` colore le texte avec **la même variable** que la teinte de fond (`--color-text-muted` sur une teinte à 15-20 % du même token) ; sur une variante neutre cela donne du gris moyen sur du gris clair. Corrigé en passant le texte à `--color-text` → **11.4:1**, avec un avertissement dans le composant pour empêcher une future « harmonisation » avec le gabarit sans re-mesure. **Le gabarit garde le défaut** (variante `.unpaid`) — pré-existant, non corrigé ici (règle de la story), documenté dans #256.

**Dette a11y pré-existante révélée → issue #256.** Le test axe sur liste **peuplée** (le test historique n'exerçait que l'empty state, et son commentaire demandait justement de l'étendre) remonte `button-name` **critical** : les 3 boutons d'action de chaque ligne sont icône seule, `aria-hidden`, sans nom accessible → un lecteur d'écran annonce trois boutons anonymes. Vérifié **hors du diff 21-6a** (`git diff` + `git show HEAD`). Non corrigé (règle de la story) ; test scopé par `disableRules(['button-name'])` avec commentaire « retirer à la fermeture de #256 ».

**Micro-déviation AC 1 (assumée, sans effet fonctionnel).** L'AC demandait d'insérer les champs « après `emailed_to` **pour refléter l'ordre de l'entité** » — mais l'entité place `project_id` entre les deux. Les champs sont donc placés **après `project_id`**, ce qui sert réellement l'intention déclarée. Impacte seulement l'ordre des clés JSON.

**Déviations AC 18 — non déclarées initialement, remontées en code review P1 (AA-1).** Le reviewer a eu raison sur le fond : j'avais documenté scrupuleusement deux déviations et passé celles-ci sous silence, ce qui est une **inconsistance de discipline** — la disclosure ne doit pas être sélective. Les deux sont maintenues, désormais justifiées et **verrouillées par des tests** :

- **Pas de prop `pausedAt`.** L'AC 18 la prescrivait (`pausedAt: string | null`), mais l'AC 20 fait conditionner le rendu **par l'appelant** (`{#if inv.dunningPausedAt !== null}`). Les deux ensemble donneraient une prop **jamais lue** : soit du code mort, soit une double condition divergente. Le composant ne prend donc que `note`. Les deux ACs étaient mutuellement redondantes ; c'est l'AC 20 (comportement observable) qui prime sur l'AC 18 (liste de props).
- **`aria-label` porte le libellé ET la note**, là où l'AC 18 disait `aria-label={label}` et « le `title` est la seule surface » qui expose la note. **Écart délibéré, meilleur pour l'utilisateur** : l'annonce de `title` par les lecteurs d'écran est notoirement peu fiable, et `aria-label` gagne sur `title` pour le nom accessible. Avec la lettre de l'AC, un utilisateur non-voyant n'aurait **jamais** accès au motif de la suspension — l'infobulle est une surface *visuelle*. La rédaction « seule surface » visait à ne pas multiplier les affichages, pas à priver les lecteurs d'écran. Test E2E ajouté sur `aria-label` (P1).

**AC 21 non tenue → corrigée en P1 (AA-2).** L'AC exigeait `offset = 0` à chaque sélection ; **je ne l'avais pas implémenté**. Effet réel : changer de filtre depuis la page 3 laisse un offset survivant à un jeu de résultats rétréci → **liste vide trompeuse**. Corrigé (`onchange`) + test E2E. La justification de l'AC (« convention de tous les filtres ») était en outre **factuellement fausse** — le `<select>` du statut ne remet pas l'offset (6 sites `offset = 0`, tous sur texte/date/contact) → AC rectifiée, et le défaut symétrique **pré-existant** de `statusFilter` part en issue **#257** plutôt que d'être corrigé ici (même discipline que #255/#256).

**Constat hors scope confirmé** (déjà signalé par la spec, à trancher en rétro) : `serialize_invoices_csv` omet `dunning_paused_*` **et** `emailed_at`/`emailed_to` de l'export de souveraineté. Pré-existant, non touché.

**Aucune migration** → aucun bump `min_required`, `migrations-idempotence-audit.md` inchangé, compteurs migrations/export inchangés (vérifié : `migrations_upgrade_path` 8/8, `exports_global_e2e` 20/20).

### File List

**Backend**
- `crates/kesh-api/src/routes/invoices.rs` (M) — `InvoiceResponse` + `from_parts`, `InvoiceListItemResponse` + `From`, `ListInvoicesQuery`, `PausedParam` + `From`, 2 constructions de `InvoiceListQuery`.
- `crates/kesh-db/src/repositories/invoices.rs` (M) — `InvoiceListItem` (+ doc fanout), 2 SELECT, `PausedFilter`, `InvoiceListQuery`, `push_where_clauses`.
- `crates/kesh-api/tests/dunning_reminders_e2e.rs` (M) — 4 tests + 2 helpers (`pause_invoice`, `item_ids`).

**i18n**
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` (M)
- `crates/kesh-i18n/locales/de-CH/messages.ftl` (M)
- `crates/kesh-i18n/locales/it-CH/messages.ftl` (M)
- `crates/kesh-i18n/locales/en-CH/messages.ftl` (M)

**Frontend**
- `frontend/src/lib/features/invoices/DunningPausedBadge.svelte` (**NEW**)
- `frontend/src/lib/features/invoices/invoices.types.ts` (M)
- `frontend/src/lib/features/invoices/invoices.api.ts` (M)
- `frontend/src/lib/features/invoices/invoices.api.test.ts` (M) — 5 tests
- `frontend/src/routes/(app)/invoices/+page.svelte` (M)
- `frontend/scripts/lint-i18n-ownership.js` (M) — `KNOWN_VIOLATIONS`
- `frontend/tests/e2e/invoices.spec.ts` (M) — 4 tests + helper `pauseInvoiceViaApi`

**Doc / suivi**
- `CHANGELOG.md` (M) — `[Non publié]` → `Ajouté`
- `_bmad-output/implementation-artifacts/21-6a-exposition-suspension.md` (M)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (M)
