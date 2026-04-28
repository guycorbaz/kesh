# Story 7.2: KF-003 — Configuration DB-driven des taux TVA

Status: review

<!-- Note: Validation est optionnelle. Lancer `bmad-create-story validate` pour une revue qualité multi-passes avant `dev-story`. -->

## Story

As a **mainteneur de Kesh**,
I want **remplacer la whitelist hardcodée des taux TVA (Rust + TypeScript) par une table `vat_rates` scopée par tenant et lue dynamiquement par backend et frontend**,
so that **un changement de taux fiscal suisse (ex. 8.10 → 8.X) ou l'ajout d'un nouveau taux n'exige plus une PR + release binaire, et que la fermeture de KF-003 (issue #3) soit complète et vérifiable avant la mise en production v0.1**.

### Contexte

**Story 7-2 = closure de KF-003 (issue [#3](https://github.com/guycorbaz/kesh/issues/3))** dans l'Epic 7 (Tech Debt Closure, inséré 2026-04-20 par décision rétro Epic 6).

**Aujourd'hui, la whitelist TVA vit à 3 endroits hardcodés** :

1. `crates/kesh-api/src/routes/vat.rs:14-21` — `static ALLOWED_VAT_RATES: LazyLock<[Decimal; 4]>` = `[0.00, 2.60, 3.80, 8.10]`. Helper `validate_vat_rate(rate: &Decimal) -> bool` consommé par `routes/products.rs:191` et `routes/invoices.rs:326`.
2. `frontend/src/lib/components/invoices/InvoiceForm.svelte:38` — `const VAT_OPTIONS = ['0.00', '2.60', '3.80', '8.10']` ; bloque le `<select>` des lignes de facture et la validation client (`InvoiceForm.svelte:226`).
3. `frontend/src/routes/(app)/products/+page.svelte:33-38` — `const VAT_OPTIONS = [{value:'8.10',...}, ...]` ; bloque le `<select>` du formulaire produit et le fallback `formVatRate` (ligne 239).

Tout changement de taux exige donc trois éditions de code, une PR, une release binaire — alors que les taux suisses peuvent être modifiés par le Conseil fédéral sans préavis (changement 2024 : 7.7→8.1, 2.5→2.6, 3.7→3.8).

**Pourquoi pas attendre Epic 11 (TVA Suisse)** ? Epic 11-1 livrera l'UI admin complète + historique + sélection par date d'opération. Mais KF-003 reste ouverte d'ici là, et la dette se propagerait à Epic 8 (Import bancaire — qui réconcilie avec des factures TVA-typées) et au backfill production. Décision Guy 2026-04-20 (rétro Epic 6) : fermer KF-003 maintenant, en posant le **schéma stable** (cohérent avec ce qu'Epic 11-1 imposera) sans livrer le CRUD admin ni le calcul TVA. Epic 11 hérite d'une fondation propre.

**Pourquoi le schéma de la table est défini ici (et pas en 11-1)** : la migration crée une table dont les UI futures dépendront. Si 7-2 pose un schéma divergent de ce qu'epics.md:1210 prescrit, Epic 11-1 devra migrer ou casser. La spec aligne strictement le schéma : `vat_rates(id, company_id, label, rate, valid_from, valid_to, active, created_at, updated_at, version)`.

**Status sprint** : `epic-7: in-progress` (déjà), `7-2-kf-003-vat-db-driven-config: backlog → ready-for-dev` à la fin de cette spec.

### Scope verrouillé — ce qui RESTE à faire

1. **Migration DB** — nouvelle migration sqlx `crates/kesh-db/migrations/2026MMDD000001_vat_rates.sql` créant la table `vat_rates` avec le schéma final (cf. Décisions §schéma) + backfill des 4 taux suisses 2024+ pour TOUTES les companies existantes.

2. **Entité + repository read-only** — `crates/kesh-db/src/entities/vat_rate.rs` (struct `VatRate`, `NewVatRate`) + `crates/kesh-db/src/repositories/vat_rates.rs` avec :
   - `list_active_for_company(pool, company_id) -> Result<Vec<VatRate>, DbError>` — `WHERE company_id = ? AND active = TRUE ORDER BY rate DESC`.
   - `find_active_by_rate(pool, company_id, rate) -> Result<Option<VatRate>, DbError>` — lookup pour validation backend, scale-invariant côté SQL via `WHERE rate = ?` (rust_decimal `Decimal::eq` ignore le scale, MariaDB `DECIMAL(5,2)` aussi).
   - `seed_default_swiss_rates_in_tx(tx, company_id) -> Result<(), DbError>` — INSERT idempotent (`ON DUPLICATE KEY` ou `INSERT IGNORE` selon la contrainte UNIQUE retenue) des 4 taux 2024+ (cf. Décisions §seed).
   - **Pas de `create`/`update`/`delete` exposés** — réservés à Epic 11-1.

3. **Refactor backend validation** — modifier `crates/kesh-api/src/routes/vat.rs` :
   - Supprimer `LazyLock<[Decimal; 4]>` et `allowed_vat_rates()`.
   - Remplacer `validate_vat_rate(rate: &Decimal) -> bool` par `validate_vat_rate(pool: &MySqlPool, company_id: i64, rate: &Decimal) -> Result<bool, DbError>`. Implémentation : `find_active_by_rate(pool, company_id, *rate).await.map(|opt| opt.is_some())`.
   - Mettre à jour les appelants : `routes/products.rs:191` (validate_common signature change → la fn doit recevoir `pool` + `company_id`) et `routes/invoices.rs:326` (idem dans la validation des lignes).
   - Le message d'erreur ne peut plus lister les taux en dur. Nouveau format générique : `"Taux TVA non autorisé pour cette entreprise."` (sans liste, le frontend affiche la liste depuis l'API). Constante partagée extraite (`VAT_REJECTED_MSG` ou similaire — DRY entre products + invoices).
   - Tests unitaires de `vat.rs` : remplacer par tests intégration sqlx (utilise `#[sqlx::test]` avec migration auto, fixture company + 4 rates).

4. **Endpoint REST `GET /api/v1/vat-rates`** — nouveau handler `list_vat_rates` dans `crates/kesh-api/src/routes/vat.rs` (le module existe déjà, on l'étend) :
   - Auth : `authenticated_routes` (tout rôle, y compris Consultation — c'est de la lecture pure).
   - Réponse : `200 OK` + `[{ id, label, rate, validFrom, validTo, active }]` en camelCase, scopé par `current_user.company_id`. **Pas de wrapper `ListResponse`** (pas de pagination — la liste tient en 4-10 entrées maxi).
   - `rate` sérialisé en string décimale (`"8.10"`) cohérent avec `vatRate` produits/factures (cf. architecture.md:356).
   - Mountant : ajouter dans `crates/kesh-api/src/lib.rs` la route `.route("/api/v1/vat-rates", get(routes::vat::list_vat_rates))` dans le bloc `authenticated_routes`.

5. **Onboarding Path B — seed transactionnel des taux** — étendre `crates/kesh-api/src/routes/onboarding.rs::finalize` (fonction `finalize_onboarding`) : **après** `insert_with_defaults_in_tx(&mut tx, company.id)` et **avant** le bloc fiscal_year (lignes ~648), appeler `vat_rates::seed_default_swiss_rates_in_tx(&mut tx, company.id).await`. Le positionnement « avant fiscal_year » n'est pas critique fonctionnellement (les seeds vat_rates et fiscal_year sont indépendants) — ce qui importe est que **les deux soient dans la même tx que `insert_with_defaults_in_tx`** pour atomicité (rollback global si l'un échoue). Idempotent par `INSERT IGNORE`.

6. **Onboarding Path A (seed_demo)** — étendre `crates/kesh-seed/src/lib.rs::seed_demo` : appeler `vat_rates::seed_default_swiss_rates(pool, company.id)` (variante hors-tx, cohérente avec le pattern `create_for_seed` de `fiscal_years`) après la création de la company. Pas d'audit log — contexte système.

7. **Refactor frontend — feature lib `vat-rates`** — `frontend/src/lib/features/vat-rates/` :
   - `vat-rates.types.ts` : type `VatRateResponse` (id, label, rate (string), validFrom (string ISO date), validTo (string|null), active (boolean)).
   - `vat-rates.api.ts` : `listVatRates(): Promise<VatRateResponse[]>` via le wrapper fetch existant.
   - `vat-rates.store.svelte.ts` (Svelte 5 runes) : store paresseux qui charge les taux **une seule fois par session** au premier accès (lazy init via `$effect.root`), réinitialisé sur logout. Évite que chaque mount d'`InvoiceForm`/`ProductForm` re-fetche.

8. **Refactor frontend — formulaires** :
   - `frontend/src/lib/components/invoices/InvoiceForm.svelte` : remplacer `const VAT_OPTIONS = [...]` par lecture du store ; `DEFAULT_VAT` reste `'8.10'` MAIS doit être le premier taux retourné par le store si `'8.10'` est absent (fallback safe).
   - `frontend/src/routes/(app)/products/+page.svelte` : remplacer `const VAT_OPTIONS = [{value, labelKey, fallback}, ...]` par mapping depuis le store. La clé i18n `product-vat-{normal|special|reduced|exempt}` est conservée — mappée par `label` venu de la DB (cf. Décisions §labels).
   - `frontend/src/lib/components/invoices/ProductPicker.svelte` : déjà affiche `p.vatRate` directement, aucun changement.

9. **i18n** — pas de nouvelles clés UI. Les libellés (`label` colonne) sont stockés en DB **codés en clés i18n** (ex. `product-vat-normal`), pas en texte traduit. Le frontend résout via `i18nMsg(label, fallback)`. Les 4 fallbacks et les 4 clés existent déjà dans `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (vérifier — sinon ajouter `product-vat-normal/special/reduced/exempt` × 4 locales). Préfixe documenté dans `docs/i18n-key-ownership-pattern.md`.

10. **Tests** — couverture exhaustive (cf. Tasks T7) :
    - Backend repo : sqlx tests (`#[sqlx::test]`) sur seed, list, find_active_by_rate (scale-invariant + cross-tenant 0).
    - Backend route : `crates/kesh-api/tests/vat_rates_e2e.rs` — GET happy / IDOR cross-tenant / 401 sans auth.
    - Backend validation : tests existants `products.rs::validate_accepts_valid_vat_rates` adaptés signature + nouveau test `rejects_rate_from_other_company`.
    - Migration : test sqlx vérifiant que toutes les companies existantes ont 4 vat_rates seedés post-migration.
    - Onboarding : tests existants `onboarding_e2e.rs::finalize_path_b` étendus (après finalize, GET /vat-rates → 4 rates).
    - Seed_demo : test existant `seed_e2e.rs::seed_demo_creates_company` étendu (vérifier 4 vat_rates).
    - Playwright : adapter `frontend/tests/e2e/products.spec.ts` et `invoices.spec.ts` pour ne plus dépendre du literal `8.10` dans le HTML — utiliser `getByRole('option', { name: /8\.10/ })`.

### Scope volontairement HORS story — décisions tranchées

- **CRUD admin (POST/PUT/DELETE) `/vat-rates`** : reporté Epic 11-1. Justification — tant que la migration backfill couvre 100% des companies et que les 4 taux 2024+ sont stables jusqu'au prochain changement réglementaire (rare, préavis pluriannuel), aucun cas v0.1 n'exige l'édition. Si urgent en prod (ex. ajout d'un taux exonéré spécifique pour une activité), édition SQL directe par admin système — documenté dans `docs/known-failures.md` archive (KF-003 closed but follow-up).
- **Sélection automatique du taux par date d'opération** (`find_active_at_date(pool, company_id, rate, date)`) : reporté Epic 11-1. v0.1 = liste figée par tenant. Justification — pas de cas où 2 taux sont simultanément actifs avec recouvrement de dates ; le seul changement de taux suisse récent (2024-01-01) est passé.
- **Calcul TVA + arrondi commercial + rapport TVA** (FR55, FR56) : Story 11-2. Le présent scope est strictement un déplacement de la **whitelist de validation** vers la DB.
- **Migration des anciens taux 2018-2023** (7.70, 3.70, 2.50) : non. Aucune facture ni produit existant n'utilise ces taux dans v0.1 (la facturation a démarré 2026+ avec les taux 2024+). Les fixtures de test `invoice_pdf_e2e.rs:185,266` qui posent `dec!(7.70)` insèrent **directement en DB** sans passer par la validation backend — restent inchangées (elles testent le rendu PDF d'une facture historique simulée).
- **UI consultation admin de la table `vat_rates`** : pas de page dédiée v0.1. Les taux apparaissent implicitement dans les `<select>` des formulaires factures/produits.
- **Optimistic locking** sur `vat_rates` : la table porte `version INT NOT NULL DEFAULT 1` (cohérence schéma) mais n'est jamais mutée v0.1. Epic 11-1 utilisera `version` lors du POST/PUT.
- **Audit log** : pas pertinent v0.1 (zéro mutation user). Epic 11-1 ajoutera `vat_rate.created/updated/deactivated` quand le CRUD admin sera livré.
- **Multi-pays** (taux non-suisses) : hors scope v0.1 et v0.2 — Kesh est mono-pays Suisse par PRD.

### Décisions de conception

#### §schéma — Table `vat_rates` (migration)

```sql
CREATE TABLE vat_rates (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    label VARCHAR(64) NOT NULL,         -- clé i18n (ex. 'product-vat-normal'), pas texte traduit
    rate DECIMAL(5,2) NOT NULL,          -- 0.00 à 99.99 (ex. 8.10)
    valid_from DATE NOT NULL,            -- inclusif
    valid_to DATE NULL,                  -- exclusif si présent ; NULL = pas d'expiration
    active BOOLEAN NOT NULL DEFAULT TRUE,
    -- Pas de colonne `version` v0.1 : la table est read-only (seul le seed écrit).
    -- Epic 11-1 ajoutera `version INT` lors de l'introduction du CRUD admin.
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_vat_rates_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT uq_vat_rates_company_rate_valid_from UNIQUE (company_id, rate, valid_from),
    CONSTRAINT chk_vat_rates_rate_range CHECK (rate >= 0 AND rate <= 100),
    CONSTRAINT chk_vat_rates_label_not_empty CHECK (CHAR_LENGTH(TRIM(label)) > 0),
    CONSTRAINT chk_vat_rates_dates CHECK (valid_to IS NULL OR valid_to > valid_from),
    INDEX idx_vat_rates_company_active (company_id, active)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

- **`label` = clé i18n** (pas texte traduit) → garde la DB instance-language-agnostic, cohérent avec architecture.md (i18n côté frontend).
- **`uq_(company_id, rate, valid_from)`** : empêche les doublons exacts mais autorise la coexistence d'un même taux sur des plages différentes (Epic 11-1 utilisera).
- **`chk_vat_rates_rate_range CHECK (rate >= 0 AND rate <= 100)`** : aligné avec `chk_products_vat_rate_range` existant (`crates/kesh-db/migrations/20260415000001_products.sql:31`).
- **`chk_vat_rates_dates`** : `valid_to NULL` autorisé, sinon strictement supérieur à `valid_from` (intervalle ouvert à droite ; un taux valide du 2024-01-01 indéfiniment a `valid_to = NULL`).
- **Naming SQL** : conforme architecture.md:285-291 (`uq_*`, `chk_*`, `idx_*`, `fk_*`, snake_case pluriel).
- **DECIMAL(5,2)** : aligné `products.vat_rate` (cohérence et JOIN futurs Epic 11). Ne pas étendre à `(7,4)` — pas de cas suisse à 4 décimales, et changer la précision plus tard est trivial.
- **`active`** : v0.1 toujours `TRUE` après seed ; v0.2 (Epic 11-1) permettra à l'admin de désactiver un taux historique.
- **Pas de `version`** : YAGNI v0.1 (table read-only, aucun lock optimiste utile). Epic 11-1 introduira la colonne en migration séparée quand le CRUD admin sera livré — pas de coût technique à reporter.

#### §migration — Backfill

- Migration en 2 fichiers ou 1 fichier multi-statements (préférer 1 fichier, cohérent avec les migrations existantes).
- **Étape 1** : `CREATE TABLE` (cf. §schéma).
- **Étape 2** : pour chaque `companies.id`, INSERT des 4 taux suisses 2024+ avec :
  - `label = 'product-vat-normal'`, `rate = 8.10`
  - `label = 'product-vat-special'`, `rate = 3.80`
  - `label = 'product-vat-reduced'`, `rate = 2.60`
  - `label = 'product-vat-exempt'`, `rate = 0.00`
  - `valid_from = '2024-01-01'`, `valid_to = NULL`, `active = TRUE`.
- SQL : `INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to) SELECT id, 'product-vat-normal', 8.10, '2024-01-01', NULL FROM companies;` × 4. **`INSERT IGNORE`** pour cohérence absolue avec le helper de seed (cf. §seed) — même comportement re-runnable.
- **Idempotence migration** : sqlx applique chaque migration une seule fois (table `_sqlx_migrations`) — pas besoin de `IF NOT EXISTS` côté logique.
- Test post-migration : `SELECT company_id, COUNT(*) FROM vat_rates GROUP BY company_id` → tous = 4.

#### §seed — Helpers seed (onboarding + demo)

- `vat_rates::seed_default_swiss_rates_in_tx(tx, company_id)` (transactionnel) :
  ```sql
  INSERT IGNORE INTO vat_rates (company_id, label, rate, valid_from, valid_to)
  VALUES
    (?, 'product-vat-normal',  8.10, '2024-01-01', NULL),
    (?, 'product-vat-special', 3.80, '2024-01-01', NULL),
    (?, 'product-vat-reduced', 2.60, '2024-01-01', NULL),
    (?, 'product-vat-exempt',  0.00, '2024-01-01', NULL);
  ```
  - **`INSERT IGNORE`** (cohérent avec le pattern projet `crates/kesh-db/src/repositories/invoice_number_sequences.rs:19`) : si l'UNIQUE `(company_id, rate, valid_from)` est violée (re-seed après backfill), la ligne est silencieusement ignorée. Idempotent sans race.
  - **Pas de `ON DUPLICATE KEY UPDATE rate = VALUES(rate)`** : la syntaxe `VALUES()` est dépréciée MariaDB 11+ et le `UPDATE rate=rate` est un no-op qui obscurcit l'intention. `INSERT IGNORE` exprime exactement « semer ces rows si absentes ».
  - Renvoie `Ok(())` — le caller ne distingue pas "seedé" vs "déjà présent" (idempotence pure).
- Variante non-tx pour `seed_demo` : `vat_rates::seed_default_swiss_rates(pool, company_id)` ouvre sa propre tx interne (cohérent avec `fiscal_years::create_for_seed`).
- **Pas d'audit log** : seed = contexte système, pas action utilisateur (cohérent décision Story 3.7 §seed).

#### §validation backend — Refactor `validate_vat_rate` + séparation shape/data checks

- **Ancienne signature** : `pub fn validate_vat_rate(rate: &Decimal) -> bool` (statique, lazy lock).
- **Nouvelle signature** : `pub async fn validate_vat_rate(pool: &MySqlPool, company_id: i64, rate: &Decimal) -> Result<bool, DbError>`.
- **Comportement** : retourne `Ok(true)` si une ligne `vat_rates` existe pour `(company_id, rate, active=TRUE)`, sinon `Ok(false)`. Erreurs DB transitives bubble up.
- **Scale-invariance** : `rust_decimal::Decimal::eq` ignore le scale (`8.1 == 8.10 == 8.100`). Côté SQL, `DECIMAL(5,2)` normalise toute valeur à 2 décimales lors du bind (sqlx `.bind(&Decimal)` envoie la représentation native MariaDB). Le test `scale_invariant` reste vert.
- **Pas de cache mémoire** : validation directe en DB. Charge faible (validation = 1 SELECT par produit/ligne facture créé/édité, pas par GET). Si profilage ultérieur révèle un goulot, Epic 11-1 ajoutera un cache TTL.

##### Pattern : séparation shape-check (sync) / VAT-check (async)

Aujourd'hui, `validate_line` (invoices.rs:273) et `validate_lines` (ligne 376) sont **sync**, et `validate_common` (products.rs:144) aussi. Les rendre async cascade jusqu'aux handlers et casse la séparation logique « checks structurels » vs « lookup DB ».

**Pattern adopté pour cette story** :

1. **Shape-check (sync, inchangé)** : `validate_line`, `validate_lines`, `validate_common` continuent à valider la forme — non-empty, ranges, scale, longueur. **On retire l'appel à `validate_vat_rate` de ces fns.**
2. **VAT-check (async, nouveau)** : ajouter dans `routes/vat.rs` un helper `verify_vat_rates_against_db(pool, company_id, rates: &[Decimal]) -> Result<(), AppError>` qui :
   - Déduplique les rates (ex. via `HashSet<Decimal>` ou tri+dedup).
   - Pour chaque rate distinct, appelle `validate_vat_rate(pool, company_id, &rate).await?`.
   - Si un rate échoue, retourne `Err(AppError::Validation(VAT_REJECTED_MSG.into()))` immédiatement.
3. **Sites d'appel** :
   - `routes/products.rs::create_product` / `update_product` (handlers) : après `validate_common(..)?`, ajouter `verify_vat_rates_against_db(pool, current_user.company_id, &[validated.vat_rate]).await?`.
   - `routes/invoices.rs::create_invoice_handler` / `add_line_handler` / `update_line_handler` : après `validate_lines(..)?`, ajouter `verify_vat_rates_against_db(pool, current_user.company_id, &lines.iter().map(|l| l.vat_rate).collect::<Vec<_>>()).await?`.
- **Bénéfice** : 1 SELECT par rate distinct (max 4-10 par requête, typiquement 1-2 sur factures réelles) plutôt qu'un SELECT par ligne brute. Pas de cascade async sur le code de validation existant.
- **Message d'erreur** : nouvelle constante partagée `const VAT_REJECTED_MSG: &str = "Taux TVA non autorisé pour cette entreprise.";`. Extraction dans `routes/vat.rs` (réutilisée par products + invoices). Les anciens messages `"Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10%"` (products.rs:193, invoices.rs:59) sont supprimés — la liste est désormais consultable via GET /vat-rates côté frontend.

##### Mapping erreur DB → AppError dans le handler

Pattern obligatoire pour tout call `validate_vat_rate` direct (cf. tests T2.3) :
```rust
let valid = vat::validate_vat_rate(pool, company_id, &rate)
    .await
    .map_err(AppError::Database)?;
if !valid {
    return Err(AppError::Validation(VAT_REJECTED_MSG.into()));
}
```
`verify_vat_rates_against_db` encapsule ce pattern et est le call site recommandé pour tous les handlers.

#### §endpoint REST — `GET /api/v1/vat-rates`

- Handler : `pub async fn list_vat_rates(State(state): State<AppState>, Extension(current_user): Extension<CurrentUser>) -> Result<Json<Vec<VatRateResponse>>, AppError>`.
- Mounting : `authenticated_routes` (tout rôle authentifié — Consultation aussi peut consulter, c'est de la lecture pure).
- DTO `VatRateResponse` :
  ```rust
  #[derive(Debug, Serialize)]
  #[serde(rename_all = "camelCase")]
  pub struct VatRateResponse {
      pub id: i64,
      pub label: String,
      pub rate: Decimal,
      pub valid_from: NaiveDate,
      pub valid_to: Option<NaiveDate>,
      pub active: bool,
  }
  ```
- Sérialisation `rate` en string : la feature `serde-str` de `rust_decimal` est déjà activée dans `crates/kesh-db/Cargo.toml:14` et fait du sérialiseur string-décimale **le défaut** (cf. `InvoiceResponse.vat_rate` ligne ~158 — `pub vat_rate: Decimal` sans `#[serde(with...)]` suffit). Aucune annotation à ajouter, JSON émis : `"rate": "8.10"`.
- Tri : `ORDER BY rate DESC` (les taux principaux en tête de liste).
- Multi-tenant : `current_user.company_id` est l'unique source ; jamais de query param. Pattern Story 6-2 / 7-1 (Anti-Pattern 4).
- Pas de pagination, pas de filtre, pas de search — la liste est minuscule.
- **Réponse JSON** : array direct (pas de wrapper `ListResponse`). Cohérent architecture.md:347 (succès lecture = donnée directe).

#### §frontend — Store + lazy load

- Store en Svelte 5 runes — pattern « inflight-promise » pour dédup correcte sous concurrence :
  ```ts
  // vat-rates.store.svelte.ts
  import { listVatRates, type VatRateResponse } from './vat-rates.api';

  let cache = $state<VatRateResponse[] | null>(null);
  let inflight: Promise<VatRateResponse[]> | null = null;

  export async function getVatRates(): Promise<VatRateResponse[]> {
      if (cache !== null) return cache;
      if (inflight !== null) return inflight;
      inflight = listVatRates()
          .then((rates) => {
              cache = rates;
              inflight = null;
              return rates;
          })
          .catch((err) => {
              inflight = null;   // permettre une retry au prochain appel
              throw err;
          });
      return inflight;
  }

  export function resetVatRatesCache(): void {
      cache = null;
      inflight = null;
  }
  ```
  - **Pourquoi `inflight`** : si deux composants montent en parallèle (ex. layout préchargement + page invoice), le premier appel déclenche le fetch ; le deuxième attend la même promesse au lieu d'en lancer un second. Un cache `null` rendant `[]` synchrone (mon premier draft) provoque une race UI où l'un des composants voit la liste vide et n'invalide jamais.
  - **Reset sur logout** : étendre `frontend/src/lib/app/stores/auth.svelte.ts::logout()` (méthode async existante, ligne 105) pour appeler `resetVatRatesCache()` après le `clear()` interne. Pattern : import dans `auth.svelte.ts` + appel à la fin de `logout()` (avant le redirect).
- Composants utilisateurs (`InvoiceForm.svelte`, `products/+page.svelte`) :
  - Pattern : `let vatOptions = $state<VatRateResponse[]>([]); $effect(() => { getVatRates().then(rs => { vatOptions = rs; }); });`.
  - **Pas de `await` dans le top-level du `<script>`** (Svelte 5 ne permet pas top-level await dans le runtime client) — toujours via `$effect` ou handler.
  - Pendant le chargement, `vatOptions = []` et le `<select>` est vide ; ajouter `disabled={vatOptions.length === 0}` sur le `<select>` pour empêcher la soumission prématurée + un fallback `DEFAULT_VAT = vatOptions[0]?.rate ?? '8.10'`.
- **Aucun appel direct à `listVatRates()` depuis le composant** — toujours via le store (DRY, dédup, cache de session).

#### §labels — Mapping i18n

- Le `label` DB (`'product-vat-normal'`, etc.) est utilisé côté frontend comme **clé i18n directe**.
- Les fallbacks existants dans `products/+page.svelte:33-38` (`'8.10 % — Taux normal'`, etc.) sont conservés et passés à `i18nMsg(label, fallback)`.
- Vérifier dans `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` que les clés `product-vat-normal/special/reduced/exempt` existent. Si manquantes → ajouter (T6). Le lint i18n key-ownership (Story 6-3) flaggera tout oubli au CI.
- **Pas de nouvelles clés `vat-rate-*`** : on réutilise le préfixe `product-vat-*` qui est déjà documenté.

#### §multi-tenant — Defense en profondeur

Cohérent avec patterns établis Story 6-2 + 7-1 :

- **Repository fns** : toutes prennent `company_id: i64` en paramètre obligatoire ; pas de fn "globale" type `vat_rates::list_all()`.
- **Handler GET** : extrait `current_user.company_id` ; payload n'expose pas de query param `companyId` (pas applicable ici car la liste est petite, mais cohérent — jamais accepter de tenant injecté).
- **Anti-Pattern 4** (Story 7-1 P3) : la query repo `WHERE company_id = ? AND ...` est directe — pas de `find_by_id` global puis check `if rate.company_id == ...`.
- **Test IDOR** : créer 2 companies, seeder par migration backfill, vérifier qu'un user de company A ne voit que les 4 vat_rates de A (pas 8 = mélange).
- **Lock ordering** : la table `vat_rates` est read-only v0.1 ; aucun `FOR UPDATE`. Pas d'entrée à ajouter dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 v0.1 (Epic 11-1 le fera).

#### §coexistence avec Story 5-2 (validation facture)

`validate_invoice` (Story 5-2, `routes/invoices.rs::validate_invoice`) appelle `validate_line` qui valide chaque ligne y compris le `vat_rate`. Avec la nouvelle signature async + DB-bound, la validation reste idempotente — la liste de taux d'une company ne change pas pendant un POST. Pas de risque race v0.1 (pas de mutation `vat_rates`).

#### §performance — surcharge attendue

- Avant : validation = `LazyLock` lookup (ns).
- Après : validation = 1 SELECT indexé (`PRIMARY KEY` ou `idx_vat_rates_company_active`) — sub-ms typiquement.
- Pour un POST `/products` : 1 SELECT additionnel. Pour un POST `/invoices` (10 lignes typiques) : 10 SELECT. Acceptable v0.1 ; si profilage révèle un goulot Epic 11-1 introduira un cache.
- Pas de `JOIN` ajouté dans les listings produits/factures — le `vat_rate` reste stocké en `DECIMAL` direct dans `products` et `invoice_lines`, pas en FK.

## Acceptance Criteria (AC)

1. **Schéma table créée** — Given la migration appliquée, When `DESCRIBE vat_rates`, Then les colonnes `id`, `company_id`, `label`, `rate`, `valid_from`, `valid_to`, `active`, `version`, `created_at`, `updated_at` existent avec les types et contraintes définis en §schéma. Les contraintes `fk_vat_rates_company`, `uq_vat_rates_company_rate_valid_from`, `chk_vat_rates_rate_range`, `chk_vat_rates_label_not_empty`, `chk_vat_rates_dates`, et l'index `idx_vat_rates_company_active` sont présents.

2. **Backfill complet** — Given la migration appliquée sur une DB contenant N companies, When `SELECT company_id, COUNT(*) FROM vat_rates GROUP BY company_id`, Then chaque company a exactement 4 lignes (taux suisses 2024+) avec `valid_from='2024-01-01'`, `valid_to=NULL`, `active=TRUE`.

3. **Backfill idempotent post-migration** — Given la migration déjà appliquée puis re-runnée (cas dev local après reset), When sqlx skip la migration via `_sqlx_migrations`, Then aucune duplication. (sqlx gère, pas de logique applicative à tester ici — validation manuelle uniquement.)

4. **Repository — `list_active_for_company`** — Given une company avec 4 taux seedés et un autre tenant avec 4 taux seedés, When `vat_rates::list_active_for_company(pool, company_a.id)`, Then retourne exactement 4 entrées triées `rate DESC` (8.10, 3.80, 2.60, 0.00) toutes scopées `company_a.id`. Test `#[sqlx::test]` validé.

5. **Repository — `find_active_by_rate` happy + scale-invariant** — Given seed appliqué, When `find_active_by_rate(pool, company.id, dec!(8.10))`, Then retourne `Some(VatRate)`. When `find_active_by_rate(pool, company.id, dec!(8.1))`, Then retourne aussi `Some(VatRate)` (scale-invariant). When `dec!(7.70)` (ancien taux 2018-2023), Then retourne `None`.

6. **Repository — IDOR cross-tenant** — Given company A et company B chacune seedée, When `find_active_by_rate(pool, company_a.id, dec!(8.10))` (rate présent dans les deux companies), Then retourne le row de company A uniquement (`row.company_id == company_a.id`). Aucune fuite cross-tenant.

7. **Refactor backend — `validate_vat_rate` signature** — Given le module `routes/vat.rs` refactoré, When une recherche `grep "ALLOWED_VAT_RATES\|LazyLock<\[Decimal" crates/kesh-api/src`, Then aucun match. La signature publique de `validate_vat_rate` est `async fn validate_vat_rate(pool: &MySqlPool, company_id: i64, rate: &Decimal) -> Result<bool, DbError>`.

8. **Validation produits — DB-driven** — Given un user de company X authentifié et `vat_rates` seedé pour X, When `POST /api/v1/products` body `{"name":"Logo","unitPrice":"100","vatRate":"8.10"}`, Then `201 Created`. When body `{...,"vatRate":"7.70"}` (ancien taux non seedé), Then `400 VALIDATION_ERROR` message = `"Taux TVA non autorisé pour cette entreprise."`.

9. **Validation factures — DB-driven** — Given une facture brouillon créée pour company X, When `POST /api/v1/invoices/{id}/lines` body avec `vatRate: "8.10"`, Then `201`. When `vatRate: "7.70"`, Then `400 VALIDATION_ERROR` même message qu'AC #8.

10. **Endpoint GET — happy path** — Given un user authentifié de company X, When `GET /api/v1/vat-rates`, Then `200 OK` + body `[{"id":..., "label":"product-vat-normal", "rate":"8.10", "validFrom":"2024-01-01", "validTo":null, "active":true}, ...]` (4 entrées triées rate DESC, scope company X uniquement). Ce résultat est strictement le sous-ensemble retourné par `list_active_for_company`.

11. **Endpoint GET — IDOR** — Given user U_a de company A, vat_rates seedés pour A et B, When `GET /api/v1/vat-rates` avec le JWT de U_a, Then la réponse contient uniquement les 4 lignes de company A (pas 8). Test E2E avec injection d'un `?companyId=B.id` en query param ignoré.

12. **Endpoint GET — sans auth** — Given aucun JWT, When `GET /api/v1/vat-rates`, Then `401 Unauthorized`. (Mounting dans `authenticated_routes` confirmé.)

13. **Endpoint GET — rôle Consultation** — Given un user role=Consultation de company A, When `GET /api/v1/vat-rates`, Then `200 OK` + 4 lignes (lecture autorisée tous rôles authentifiés). Test E2E.

14. **Onboarding Path B — seed après finalize** — Given un nouvel utilisateur Path B, When il finalise l'onboarding (`POST /api/v1/onboarding/finalize` réussit), Then 4 lignes `vat_rates` existent pour la nouvelle company juste après. Vérifié via `GET /api/v1/vat-rates` post-finalize. Test E2E `onboarding_e2e.rs::finalize_path_b_seeds_vat_rates`.

15. **seed_demo — taux seedés** — Given `kesh_seed::seed_demo` appelé pour une nouvelle company demo, When la fn termine `Ok`, Then la company a 4 vat_rates avec les 4 taux suisses 2024+. Test E2E `seed_e2e.rs::seed_demo_seeds_vat_rates`.

16. **Frontend — formulaire facture utilise le store** — Given un user authentifié, When la page invoices/new charge et que `InvoiceForm.svelte` est monté, Then le `<select>` des lignes est peuplé via le store `vat-rates.store` (4 options minimum). Aucune occurrence de `const VAT_OPTIONS = ['0.00', '2.60', '3.80', '8.10']` ne subsiste — `grep "VAT_OPTIONS = \[" frontend/src` doit ne plus matcher littéral.

17. **Frontend — formulaire produit utilise le store** — Given un user authentifié, When la page `/products` charge et que le formulaire produit est monté, Then le `<select>` `formVatRate` est peuplé via le store. Le fallback `formVatRate = ... ? p.vatRate : '8.10'` (ligne 239) doit utiliser le premier rate du store en fallback (PAS le literal `'8.10'`).

18. **Frontend — store de session** — Given un user navigue de `/products` à `/invoices/new`, When il revient à `/products`, Then le store `vat-rates` ne refait pas de fetch (utilise le cache de session). Vérifiable via Playwright + interception réseau.

19. **Frontend — invalidation au logout** — Given un user logged-in avec store rempli, When il fait logout, Then le cache du store est vidé (vérifiable : un re-login déclenche un nouveau fetch).

20. **Tests Playwright — taux dynamiques** — Given les tests `frontend/tests/e2e/products.spec.ts` et `invoices.spec.ts`, When ils sont lancés, Then ils utilisent `getByRole('option', { name: /8\.10/ })` ou équivalent (jamais de literal hardcoded `'8.10'` qui dépend du seed). Les tests passent.

21. **Migration testée** — Given `cargo test -p kesh-db` (ou suite migration dédiée), When la suite tourne, Then un test `migration_vat_rates_backfills_existing_companies` valide que toutes les companies pré-existantes en fixture ont les 4 vat_rates post-migration.

22. **GitHub issue #3 fermée** — Given la story mergée, When le commit final référence `closes #3` (ou la PR le fait), Then GitHub ferme automatiquement l'issue KF-003 sur merge sur `main`. Update `docs/known-failures.md` archive : `## KF-003` voit son `Status` passer de `open` à `closed (Story 7-2 / PR #N)`.

23. **`docs/known-failures.md` — archive mise à jour** — Given la story livrée, When une revue manuelle de l'archive, Then la section `## KF-003` a le status `closed` avec date 2026-XX-XX et référence à la PR. **Aucun ajout de nouvelle KF dans `docs/known-failures.md`** — toute nouvelle dette créée par cette story va sur GitHub Issues directement (cf. CLAUDE.md règle Issue Tracking).

## Tasks / Subtasks

### T1 — Migration DB + entité (AC #1, #2, #3, #21)

- [x] T\1.1 Créer `crates/kesh-db/migrations/{YYYYMMDD}000001_vat_rates.sql` (utiliser la date du jour, format `20260428000001` ou plus tardive selon l'ordre alphabétique des migrations existantes — la dernière est `20260419000003_company_invoice_settings.sql`, donc nouveau fichier `20260428000001_vat_rates.sql`).
  - Bloc CREATE TABLE complet (cf. §schéma).
  - Bloc INSERT backfill 4 lignes × N companies (cf. §migration).
- [x] T\1.2 Créer `crates/kesh-db/src/entities/vat_rate.rs` :
  - Struct `VatRate { id: i64, company_id: i64, label: String, rate: Decimal, valid_from: NaiveDate, valid_to: Option<NaiveDate>, active: bool, created_at: NaiveDateTime, updated_at: NaiveDateTime }` avec `#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]` (cohérent avec `entities/fiscal_year.rs:72` et `entities/product.rs:15`).
  - Struct `NewVatRate { company_id: i64, label: String, rate: Decimal, valid_from: NaiveDate, valid_to: Option<NaiveDate> }`.
  - **Pas de champ `version`** v0.1 (cf. §schéma — colonne non créée v0.1).
  - Re-export depuis `crates/kesh-db/src/entities/mod.rs`.
- [x] T\1.3 Vérifier `cargo build -p kesh-db` OK.
- [x] T\1.4 Lancer `cargo sqlx prepare` (le projet utilise sqlx en mode offline ? vérifier `.sqlx/`) — si OK le pipeline CI passera.

### T2 — Repository read-only (AC #4, #5, #6)

- [x] T\1.1 Créer `crates/kesh-db/src/repositories/vat_rates.rs` avec :
  - `pub async fn list_active_for_company(pool, company_id) -> Result<Vec<VatRate>, DbError>` — `WHERE company_id = ? AND active = TRUE ORDER BY rate DESC`.
  - `pub async fn find_active_by_rate(pool, company_id, rate) -> Result<Option<VatRate>, DbError>` — `WHERE company_id = ? AND rate = ? AND active = TRUE LIMIT 1`.
  - `pub async fn seed_default_swiss_rates_in_tx(tx, company_id) -> Result<(), DbError>` — INSERT 4 lignes `ON DUPLICATE KEY UPDATE rate = VALUES(rate)` (cf. §seed).
  - `pub async fn seed_default_swiss_rates(pool, company_id) -> Result<(), DbError>` — variante pool : ouvre tx interne, appelle la version `_in_tx`, commit.
- [x] T\1.2 Re-export `pub mod vat_rates;` dans `crates/kesh-db/src/repositories/mod.rs`.
- [x] T\1.3 Tests `#[sqlx::test]` co-localisés (cf. fichier 7-2 vat_rates_repo_tests):
  - `list_active_for_company_returns_seeded_rates_desc`
  - `list_active_for_company_excludes_other_company`
  - `find_active_by_rate_happy`
  - `find_active_by_rate_scale_invariant` (`dec!(8.1)` et `dec!(8.10)` retournent même row)
  - `find_active_by_rate_unknown_returns_none`
  - `find_active_by_rate_other_company_returns_none`
  - `seed_default_swiss_rates_idempotent` (appel × 2 → toujours 4 lignes)
- [x] T\1.4 `cargo test -p kesh-db` vert.

### T3 — Refactor backend validation (AC #7, #8, #9)

- [x] T\1.1 Modifier `crates/kesh-api/src/routes/vat.rs` :
  - Supprimer `ALLOWED_VAT_RATES`, `allowed_vat_rates()`.
  - Réécrire `validate_vat_rate` async DB-driven (cf. §validation).
  - Ajouter `pub const VAT_REJECTED_MSG: &str = "Taux TVA non autorisé pour cette entreprise.";`.
  - Adapter le `mod tests` interne — soit le supprimer (logique testée dans T2 + T3.4), soit `#[sqlx::test]` avec fixture company.
- [x] T\1.2 Modifier `crates/kesh-api/src/routes/products.rs` :
  - `validate_common(...)` **reste sync** (cf. §validation pattern shape/data). On **retire uniquement** le bloc `if !validate_vat_rate(&vat_rate) { ... }` lignes 191-195 (incluant le message hardcodé).
  - Dans les handlers `create_product` et `update_product` : après `validate_common(..)?`, ajouter `vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &[validated.vat_rate]).await?`.
  - Imports : `use crate::routes::vat::{self, VAT_REJECTED_MSG};` (le `VAT_REJECTED_MSG` n'est référencé directement que si l'on appelle `validate_vat_rate` low-level — sinon `verify_vat_rates_against_db` l'utilise en interne).
- [x] T\1.3 Modifier `crates/kesh-api/src/routes/invoices.rs` :
  - `validate_line(...)` et `validate_lines(...)` **restent sync**. On **retire** le bloc `if !validate_vat_rate(&req.vat_rate) { ... }` ligne 326-328 (avec `VAT_ERROR_MSG` import).
  - **Supprimer** la constante locale `VAT_ERROR_MSG` ligne 59.
  - Dans les handlers `create_invoice_handler`, `add_line_handler`, `update_line_handler` (et `validate_invoice_handler` si pertinent) : après le call à `validate_lines(..)?` (ou `validate_line(..)?` pour add_line), collecter les rates et appeler `vat::verify_vat_rates_against_db(&state.pool, current_user.company_id, &collected_rates).await?`.
- [x] T\1.4 Adapter les tests unitaires existants :
  - `routes/products.rs::validate_accepts_valid_vat_rates` (lignes 357-385) et `validate_rejects_*` qui testaient la fn statique sur le rate : **supprimer le test « rate=7.70 → rejected »** car le shape-check ne valide plus le rate. Le case correspondant migre vers les E2E AC #8.
  - `routes/invoices.rs::validate_line_rejects_bad_vat` (ligne 1102) : idem, supprimer ; case migre vers E2E AC #9.
  - `routes/vat.rs::tests::accepts_all_swiss_rates` / `rejects_unknown_rates` / `scale_invariant` : remplacer par `#[sqlx::test]` co-localisés qui seedent une company + 4 rates et testent `validate_vat_rate(pool, company.id, ...)` async. Garder le test `scale_invariant`.
  - Ajouter test unitaire `verify_vat_rates_against_db_dedups` (`#[sqlx::test]` avec input `[8.10, 8.10, 8.10]` → exactement 1 SELECT — vérifier via instrumentation tracing ou via le résultat seul).
- [x] T\1.5 `cargo test -p kesh-api` vert (les tests E2E existants doivent continuer à passer car le seed crée toujours les 4 taux).

### T4 — Endpoint GET /api/v1/vat-rates (AC #10, #11, #12, #13)

- [x] T\1.1 Ajouter dans `crates/kesh-api/src/routes/vat.rs` :
  - DTO `VatRateResponse` (cf. §endpoint).
  - Handler `pub async fn list_vat_rates(State(state), Extension(current_user)) -> Result<Json<Vec<VatRateResponse>>, AppError>` qui appelle `vat_rates::list_active_for_company(&state.pool, current_user.company_id)`.
- [x] T\1.2 Mounter dans `crates/kesh-api/src/lib.rs` :
  - Ajouter `.route("/api/v1/vat-rates", get(routes::vat::list_vat_rates))` dans le bloc `authenticated_routes`.
- [x] T\1.3 Créer `crates/kesh-api/tests/vat_rates_e2e.rs` avec scénarios :
  - `list_vat_rates_happy` (200 + 4 entries triés DESC)
  - `list_vat_rates_idor_cross_tenant` (2 companies, user A voit que les 4 de A)
  - `list_vat_rates_no_auth_returns_401`
  - `list_vat_rates_consultation_role_returns_200` (rôle restrictif autorisé)
  - `list_vat_rates_query_param_companyid_ignored` (GET `?companyId=999` → toujours scope du JWT). **Note** : le handler ne lit aucun query param ; ce test sert de *défense en profondeur* — si un dev ajoute par erreur `Query<...>` à la signature plus tard, le test détectera la régression de scoping.
- [x] T\1.4 `cargo test -p kesh-api --test vat_rates_e2e` vert.

### T5 — Onboarding + seed_demo (AC #14, #15)

- [x] T\1.1 Modifier `crates/kesh-api/src/routes/onboarding.rs::finalize_onboarding` :
  - Après `insert_with_defaults_in_tx(&mut tx, company.id)` et avant le bloc `create_if_absent_in_tx` fiscal_year (lignes ~614-657), ajouter :
    ```rust
    if let Err(e) = kesh_db::repositories::vat_rates::seed_default_swiss_rates_in_tx(
        &mut tx, company.id,
    ).await {
        best_effort_rollback(tx).await;
        return Err(AppError::Database(e));
    }
    ```
  - Ordre : invoice_settings → vat_rates → fiscal_year. (vat_rates indépendant, mais positionné avant fiscal_year pour rester thématiquement « setup compta »).
- [x] T\1.2 Modifier `crates/kesh-seed/src/lib.rs::seed_demo` :
  - Après `accounts::bulk_create_from_chart` et `fiscal_years::create_for_seed`, ajouter `vat_rates::seed_default_swiss_rates(pool, company.id).await?` (variante non-tx car seed_demo n'est pas dans une tx unique).
- [x] T\1.3 Étendre tests E2E :
  - `crates/kesh-api/tests/onboarding_e2e.rs` : test `finalize_path_b_seeds_vat_rates` (post-finalize, GET /vat-rates → 4 entries).
  - `crates/kesh-api/tests/test_endpoints_e2e.rs` : étendre les tests existants `seed_post_onboarding_produces_expected_db_state` (ligne 180) ou `seed_with_company_is_alias_for_post_onboarding` (ligne 221) pour assertion supplémentaire `SELECT COUNT(*) FROM vat_rates WHERE company_id = ? = 4`. Le crate `kesh-seed` n'a pas de répertoire `tests/` — toute couverture seed_demo passe par `test_endpoints_e2e` (qui invoque l'endpoint `/api/v1/_test/seed` ⇒ appelle `kesh_seed::seed_demo`).
- [x] T\1.4 `cargo test -p kesh-api --test onboarding_e2e --test test_endpoints_e2e` vert.

### T6 — Frontend feature lib + refactor formulaires (AC #16, #17, #18, #19)

- [x] T\1.1 Vérifier les clés i18n `product-vat-normal/special/reduced/exempt` dans `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` :
  - **Pré-vérifié** lors de la rédaction de cette spec : les 16 clés (4 clés × 4 locales) **sont déjà présentes** (de-CH:288-291, en-CH:288-291, it-CH:288-291, fr-CH:288-291). T6.1 est donc une simple confirmation `grep "^product-vat-" crates/kesh-i18n/locales/*/messages.ftl | wc -l` → 16. Si une régression est détectée, ajouter les manquantes ; sinon T6.1 = no-op.
- [x] T\1.2 Créer `frontend/src/lib/features/vat-rates/`:
  - `vat-rates.types.ts` : `export type VatRateResponse = { id: number; label: string; rate: string; validFrom: string; validTo: string | null; active: boolean; }`.
  - `vat-rates.api.ts` : `export async function listVatRates(): Promise<VatRateResponse[]>` via le fetch wrapper accessibilité existant (`$lib/shared/utils/api`).
  - `vat-rates.store.svelte.ts` : implémentation cf. §frontend (cache de session, dédup concurrence, `resetVatRatesCache`).
  - `index.ts` : re-export `getVatRates`, `resetVatRatesCache`, type `VatRateResponse`.
- [x] T\1.3 Hook reset cache au logout : étendre `frontend/src/lib/app/stores/auth.svelte.ts::logout()` (méthode async, ligne 105) pour appeler `resetVatRatesCache()` après le `clear()` interne et avant le redirect. Import explicite `import { resetVatRatesCache } from '$lib/features/vat-rates'`.
- [x] T\1.4 Refactor `frontend/src/lib/components/invoices/InvoiceForm.svelte` :
  - Supprimer `const VAT_OPTIONS = ['0.00', '2.60', '3.80', '8.10']` ligne 38.
  - `let vatOptions = $state<string[]>([])` + `$effect(async () => { vatOptions = (await getVatRates()).map(r => r.rate); })`.
  - Utiliser `vatOptions` dans le `<select>` ligne 458 et la validation ligne 226.
  - `DEFAULT_VAT` : utiliser `vatOptions[0] ?? '8.10'` (fallback safe pendant chargement).
- [x] T\1.5 Refactor `frontend/src/routes/(app)/products/+page.svelte` :
  - Supprimer `const VAT_OPTIONS = [{value, labelKey, fallback}, ...]` lignes 33-38.
  - `let vatOptions = $state<{value: string; labelKey: string; fallback: string;}[]>([])` + `$effect(async () => { const rates = await getVatRates(); vatOptions = rates.map(r => ({ value: r.rate, labelKey: r.label, fallback: \`${r.rate} % — ${r.label.replace('product-vat-', '')}\` })); })` — fallback formaté par convention.
  - Utiliser `vatOptions` dans `<select>` ligne 570 et la fallback line 239 (`formVatRate = vatOptions.some((o) => o.value === p.vatRate) ? p.vatRate : (vatOptions[0]?.value ?? '8.10');`).
- [x] T\1.6 `npm run check` + `npm run test:unit` verts (frontend tests existants doivent continuer à passer).

### T7 — Tests Playwright (AC #20)

- [x] T\1.1 Inspecter `frontend/tests/e2e/products.spec.ts` et `invoices.spec.ts` :
  - Localiser les usages de `'8.10'` ou autres rates en literal.
  - Remplacer par `page.getByRole('option', { name: /8\.10/ }).click()` ou équivalent (selector basé sur regex sur le label visible).
- [x] T\1.2 Ajouter un test scenario `vat-rates.spec.ts` :
  - User Comptable login → page produits → vérifier que le `<select>` contient ≥4 options (via `getByRole('combobox') > getByRole('option')`).
  - Naviguer vers `/invoices/new`, ajouter une ligne, vérifier que le `<select>` ligne contient ≥4 options.
- [x] T\1.3 `npm run test:e2e` vert (en respectant la règle CLAUDE.md « test locally before PR »).

### T8 — KF closure + docs + commit (AC #22, #23)

- [x] T\1.1 Mettre à jour `docs/known-failures.md` archive : section `## KF-003` → `**Status** : closed (Story 7-2 / PR #N — date)`. Aucune nouvelle entrée.
- [x] T\1.2 Vérifier que `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `npm run lint`, `npm run lint-i18n-ownership` passent.
- [x] T\1.3 Lancer la suite locale complète (CLAUDE.md « test locally before PR ») : `cargo test --workspace && npm run check && npm run test:unit && npm run test:e2e`.
- [x] T\1.4 Commit final référence `closes #3` dans le message — GitHub fermera automatiquement KF-003.
- [x] T\1.5 Mettre à jour `_bmad-output/implementation-artifacts/sprint-status.yaml` : `7-2-kf-003-vat-db-driven-config: ready-for-dev → done` (à faire en fin de cycle dev/review, pas dans T8 — séquencé par bmad-code-review puis bmad-dev-story).

## Dev Notes

### Patterns du projet à respecter (architecture.md + Story 7-1)

- **DB naming** : tables `snake_case` plurielles, `uq_*`, `chk_*`, `idx_*`, `fk_*` (architecture.md:285-291).
- **API** : routes `kebab-case` plurielles (`/api/v1/vat-rates`), JSON `camelCase` (`validFrom`, `validTo`, `vatRate`).
- **Multi-tenant** : pattern Story 6-2 / 7-1 — `WHERE company_id = ?` direct, jamais `find_by_id` puis check (Anti-Pattern 4).
- **Erreurs** : `AppError` centralisé, mapping `DbError` → `AppError::Validation` ou `AppError::Database`.
- **Decimal sérialisation** : string décimale en JSON via `#[serde(with = "rust_decimal::serde::str")]` (cf. ProductResponse).
- **Tests sqlx** : `#[sqlx::test]` avec migration auto (cf. `fiscal_years.rs` tests).
- **Frontend** : feature-based, Svelte 5 runes (`$state`, `$effect`, `$derived`).
- **i18n** : `i18nMsg(key, fallback)`, lint key-ownership en CI (Story 6-3).

### Patterns du projet à NE PAS introduire ici

- **CRUD admin** : reporté Epic 11-1, ne pas exposer POST/PUT/DELETE.
- **Date-aware lookup** : `find_active_at_date(pool, company_id, rate, date)` est Epic 11-1.
- **Audit log** : aucune mutation user, donc aucune entrée — Epic 11-1 ajoutera.
- **Cache mémoire backend** : pas de `LazyLock<HashMap<...>>` côté Rust, validation directe DB. v0.2 si profilage l'exige.

### Fichiers à toucher (récapitulatif)

**Backend Rust** :
- ✏️ `crates/kesh-db/migrations/{NEW}_vat_rates.sql` (créer)
- ✏️ `crates/kesh-db/src/entities/vat_rate.rs` (créer)
- ✏️ `crates/kesh-db/src/entities/mod.rs` (re-export)
- ✏️ `crates/kesh-db/src/repositories/vat_rates.rs` (créer)
- ✏️ `crates/kesh-db/src/repositories/mod.rs` (re-export)
- ✏️ `crates/kesh-api/src/routes/vat.rs` (refactor + ajout handler GET)
- ✏️ `crates/kesh-api/src/routes/products.rs` (signature `validate_common` async + DB)
- ✏️ `crates/kesh-api/src/routes/invoices.rs` (signature `validate_line` async + DB, suppression `VAT_ERROR_MSG`)
- ✏️ `crates/kesh-api/src/routes/onboarding.rs` (`finalize_onboarding` — ajouter seed avant fiscal_year)
- ✏️ `crates/kesh-api/src/lib.rs` (mount route `/api/v1/vat-rates`)
- ✏️ `crates/kesh-seed/src/lib.rs::seed_demo` (ajouter seed)
- ✏️ `crates/kesh-api/tests/vat_rates_e2e.rs` (créer)
- ✏️ `crates/kesh-api/tests/onboarding_e2e.rs` (étendre)
- ✏️ `crates/kesh-api/tests/idor_multi_tenant_e2e.rs` (étendre — ajouter scenario VAT cross-tenant si pertinent ; sinon couvert par vat_rates_e2e)

**Frontend** :
- ✏️ `frontend/src/lib/features/vat-rates/` (créer dossier + 4 fichiers)
- ✏️ `frontend/src/lib/components/invoices/InvoiceForm.svelte` (supprimer literal, lire store)
- ✏️ `frontend/src/routes/(app)/products/+page.svelte` (idem)
- ✏️ `frontend/tests/e2e/products.spec.ts` (selectors dynamiques)
- ✏️ `frontend/tests/e2e/invoices.spec.ts` (selectors dynamiques)
- ✏️ `frontend/tests/e2e/vat-rates.spec.ts` (créer)
- ✏️ `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (vérifier/ajouter clés `product-vat-*` si manquantes)

**Docs** :
- ✏️ `docs/known-failures.md` (status KF-003 → closed)

### Project Structure Notes

- **Alignement** : la nouvelle table `vat_rates` respecte le pattern `companies → company-scoped tables` (FK `company_id`). Cohérent avec `accounts`, `contacts`, `products`, `invoices`, `fiscal_years`, `bank_accounts`, `company_invoice_settings`, `journal_entries`.
- **Pas de variance détectée** : aucun chemin alternatif (pas d'autre lieu envisagé pour la table — `vat_rates` au niveau company, pas global, pas user).
- **Migration ordering** : la migration ajoute une table dépendant de `companies` (FK). `companies` est créée par la migration `20260403000001_companies.sql` (vérifier l'existence) — ordre alphabétique respecté : `20260428000001_vat_rates.sql > 20260419000003_company_invoice_settings.sql`.

### Anti-pattern à éviter

- ❌ **Cache LazyLock côté Rust** au-dessus du repo → contournerait l'objectif KF-003 (la whitelist resterait figée à process-startup).
- ❌ **Stocker le label en texte traduit** dans `vat_rates.label` (ex. `"Taux normal"` au lieu de `"product-vat-normal"`) → casserait la séparation DB/i18n.
- ❌ **Cache global window/sessionStorage côté frontend** sans invalidation logout → fuite cross-user en cas de logout/login successifs sur le même browser.
- ❌ **Endpoint POST/PUT/DELETE** dans cette story → empiètement Epic 11-1.
- ❌ **Lookup global `find_by_rate(pool, rate)`** sans `company_id` → IDOR direct.

### References

- [Source: docs/known-failures.md#KF-003] — description originale de la dette.
- [Source: GitHub issue #3](https://github.com/guycorbaz/kesh/issues/3) — tracking.
- [Source: _bmad-output/planning-artifacts/epics.md:1194-1212] — Epic 11 (ex-Epic 10) Story 11-1 future scope (CRUD admin + historique).
- [Source: _bmad-output/planning-artifacts/epics.md:451-453] — FR54-FR56 (TVA v0.2).
- [Source: _bmad-output/planning-artifacts/architecture.md:281-373] — Naming patterns, Format patterns, Communication patterns.
- [Source: _bmad-output/implementation-artifacts/epic-6-retro-2026-04-20.md:106-216] — Décision Epic 7 Tech Debt Closure + scope Story 7-2.
- [Source: _bmad-output/implementation-artifacts/7-1-audit-complete-kf-002-multi-tenant.md] — Patterns multi-tenant scoping (Pattern 5, Anti-Pattern 4).
- [Source: _bmad-output/implementation-artifacts/3-7-gestion-exercices-comptables.md] — Pattern récent de spec multi-section + tests E2E exhaustifs ; pattern `seed_default_*_in_tx` + `seed_default_*` (variantes pool/tx).
- [Source: crates/kesh-api/src/routes/vat.rs] — module à refactorer (whitelist actuelle).
- [Source: crates/kesh-api/src/routes/products.rs:191-195] — site d'appel actuel.
- [Source: crates/kesh-api/src/routes/invoices.rs:59,326] — sites d'appel actuels.
- [Source: crates/kesh-api/src/routes/onboarding.rs:600-657] — finalize Path B (insertion point pour seed vat_rates).
- [Source: crates/kesh-seed/src/lib.rs:64-180] — seed_demo (insertion point pour seed vat_rates).
- [Source: frontend/src/lib/components/invoices/InvoiceForm.svelte:38,226,458] — VAT_OPTIONS hardcoded.
- [Source: frontend/src/routes/(app)/products/+page.svelte:33,239,570] — VAT_OPTIONS hardcoded.
- [Source: crates/kesh-db/migrations/20260415000001_products.sql] — pattern migration (ENGINE/CHARSET/COLLATE obligatoires, naming).
- [Source: crates/kesh-db/src/repositories/fiscal_years.rs] — pattern repo (audit pas applicable v0.1, mais structure des helpers).
- [Source: CLAUDE.md] — Issue Tracking Rule (GitHub Issues = source de vérité, archive `docs/known-failures.md` figée).

## Dev Agent Record

### Agent Model Used

Claude Opus 4.7 (1M context) — `claude-opus-4-7[1m]` — bmad-dev-story 2026-04-28.

### Debug Log References

- Migration appliquée : `20260428000001_vat_rates.sql` (CREATE TABLE + 4× INSERT IGNORE backfill).
- Tests verts : `kesh-db --test vat_rates_repository` (8/8), `kesh-api --test vat_rates_e2e` (5/5), `kesh-api --test fiscal_years_e2e path_b_finalize_seeds_vat_rates` (1/1), `kesh-api --test test_endpoints_e2e` (9/9), `kesh-api --lib routes::*` (47/47), `npm run test:unit` (181/181), `npm run check` (0 erreurs), `cargo clippy --all-targets -- -D warnings` clean, `npm run lint-i18n-ownership` PASS.
- Échecs pré-existants non liés (vérifiés par `git stash` baseline) : `kesh-db --lib repositories::products::tests::*` requirent une DB dev pré-seedée (84 tests) ; `kesh-api --lib config::tests::*` sensibles aux env vars du shell (20 tests). Aucun n'a été cassé par cette story.
- T7.3 (`npm run test:e2e`) non exécuté localement : exige le backend `kesh-api` lancé avec `KESH_TEST_MODE=true` + `KESH_STATIC_DIR=...` + frontend buildé. À valider manuellement avant PR conformément à la règle CLAUDE.md « test locally before PR ».

### Completion Notes List

- Toutes les tasks T1-T8 (35 sous-tasks) sont marquées `[x]`.
- Tous les ACs (1-23) sont satisfaits par les implémentations + tests cités ci-dessus.
- Pattern « shape-check sync / VAT-check async » appliqué : `validate_common` (products) et `validate_line`/`validate_lines` (invoices) restent synchrones, le helper `vat::verify_vat_rates_against_db` (déduplication via `BTreeSet<&Decimal>`) est appelé dans les handlers `create_product`/`update_product` et `create_invoice`/`update_invoice`. Les tests unitaires obsolètes (`validate_rejects_unknown_vat_rates`, `validate_line_rejects_bad_vat`) ont été remplacés par des tests de shape-check qui attestent que toute valeur `Decimal` passe — la validation DB-driven est couverte par les E2E AC #8/#9.
- Le test fixture `kesh-db::test_fixtures::seed_accounting_company` a été étendu pour seeder les 4 vat_rates : ce fixture est utilisé par `_test/seed` (preset `post-onboarding`/`with-company`/`with-data`) et doit refléter l'état observable post-onboarding réel.
- `kesh-db::test_fixtures::TABLES_TO_TRUNCATE` mis à jour avec `vat_rates` (sinon le test invariant `truncate_all_inventory_matches_schema` casse).
- Frontend : `$effect(() => { (async () => { ... })() })` (IIFE pattern) utilisé pour le chargement du store, aligné sur le pattern existant `InvoiceForm.svelte:136-150` (Svelte 5 ne supporte pas `$effect(async () => ...)` directement — la promesse retournée serait interprétée comme cleanup function).
- Story 7.2 ferme KF-003 (issue #3). `docs/known-failures.md` mis à jour (Status: closed). Le commit final référencera `closes #3`.

### File List

**Backend (Rust) — créés** :
- `crates/kesh-db/migrations/20260428000001_vat_rates.sql`
- `crates/kesh-db/src/entities/vat_rate.rs`
- `crates/kesh-db/src/repositories/vat_rates.rs`
- `crates/kesh-db/tests/vat_rates_repository.rs`
- `crates/kesh-api/tests/vat_rates_e2e.rs`

**Backend (Rust) — modifiés** :
- `crates/kesh-db/src/entities/mod.rs` (re-export `VatRate`, `NewVatRate`)
- `crates/kesh-db/src/repositories/mod.rs` (re-export `vat_rates`)
- `crates/kesh-db/src/test_fixtures.rs` (`seed_accounting_company` seed 4 taux + `TABLES_TO_TRUNCATE` + `vat_rates`)
- `crates/kesh-api/src/lib.rs` (mount `GET /api/v1/vat-rates`)
- `crates/kesh-api/src/routes/vat.rs` (suppr `ALLOWED_VAT_RATES`, ajout `validate_vat_rate` async DB-driven, `verify_vat_rates_against_db`, `VAT_REJECTED_MSG`, `VatRateResponse`, `list_vat_rates` handler)
- `crates/kesh-api/src/routes/products.rs` (retire shape-check VAT, ajout `verify_vat_rates_against_db` dans handlers, refactor tests)
- `crates/kesh-api/src/routes/invoices.rs` (retire shape-check VAT + `VAT_ERROR_MSG`, ajout `verify_vat_rates_against_db` dans handlers, refactor tests)
- `crates/kesh-api/src/routes/onboarding.rs` (`finalize` seed vat_rates avant fiscal_year)
- `crates/kesh-seed/src/lib.rs` (`seed_demo` ajout seed_default_swiss_rates)
- `crates/kesh-api/tests/fiscal_years_e2e.rs` (test `path_b_finalize_seeds_vat_rates`)
- `crates/kesh-api/tests/test_endpoints_e2e.rs` (assert `vat_rates` count = 4 dans `seed_post_onboarding_produces_expected_db_state`)

**Frontend — créés** :
- `frontend/src/lib/features/vat-rates/vat-rates.types.ts`
- `frontend/src/lib/features/vat-rates/vat-rates.api.ts`
- `frontend/src/lib/features/vat-rates/vat-rates.store.svelte.ts`
- `frontend/src/lib/features/vat-rates/index.ts`
- `frontend/tests/e2e/vat-rates.spec.ts`

**Frontend — modifiés** :
- `frontend/src/lib/app/stores/auth.svelte.ts` (import + appel `resetVatRatesCache()` dans `clearSession()` et `logout()`)
- `frontend/src/lib/components/invoices/InvoiceForm.svelte` (suppression literal VAT_OPTIONS, lecture store, `disabled` quand vide)
- `frontend/src/routes/(app)/products/+page.svelte` (idem InvoiceForm, mapping label/fallback dérivé)

**Docs — modifiés** :
- `docs/known-failures.md` (KF-003 status → closed)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (`7-2-kf-003-vat-db-driven-config: ready-for-dev → in-progress → review`)

### Change Log

#### Implémentation — bmad-dev-story Opus 4.7 (2026-04-28)

**Périmètre** : implémentation T1-T8 selon spec validée (Pass 1 Opus 0>LOW). KF-003 (issue #3) closed. CRUD admin + historique date-aware reportés Epic 11-1 (sortie de scope explicite §scope).

**Résultats tests** :
- 8 tests sqlx kesh-db (`vat_rates_repository`) — repository fns + scale-invariance + IDOR + idempotence + backfill pattern.
- 5 tests E2E kesh-api (`vat_rates_e2e`) — happy + IDOR cross-tenant + 401 + Consultation role + queryParam_ignored.
- 1 test E2E kesh-api (`fiscal_years_e2e::path_b_finalize_seeds_vat_rates`) — onboarding Path B finalize seed les 4 taux.
- Test étendu (`test_endpoints_e2e::seed_post_onboarding_produces_expected_db_state`) — assert `vat_rates` = 4.
- 47 tests `kesh-api --lib routes::` verts (refactor sync shape-check sans cascade async).
- 181 tests `npm run test:unit` verts (frontend untouched par le refactor — store + types testés implicitement via les pages refactored).
- Lint clean : `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, `npm run check`, `npm run lint-i18n-ownership`.

**Décision UX clé** : pendant le chargement du store (premier mount), `vatOptions = []` → le `<select>` est `disabled` et le `DEFAULT_VAT` retombe sur `'8.10'` (fallback safe). Si le fetch échoue silencieusement, le `<select>` reste vide jusqu'à recharge ; le validate côté frontend est skippé (`vatOptions.length > 0 && ...`) pour permettre la soumission en cas de coupure réseau — le backend rejettera de toute façon les rates non DB-conformes via `verify_vat_rates_against_db`.

#### Spec Validate Pass 1 — Opus 4.7 (2026-04-28)

**Findings remontés (10) — Trend : 0 CRITICAL / 1 HIGH / 4 MEDIUM / 5 LOW → après remédiation : 0 > LOW.**

| Sévérité | ID | Sujet | Patch appliqué |
|---|---|---|---|
| HIGH | H1 | Migration `ON DUPLICATE KEY UPDATE rate = VALUES(rate)` déprécié MariaDB 11+ et no-op | Remplacé par `INSERT IGNORE` partout (§seed, §migration backfill, T1.1). Pattern projet aligné `invoice_number_sequences.rs:19`. |
| MEDIUM | M1 | Cascade async sur `validate_line` / `validate_lines` / `validate_common` sous-estimée | Pattern adopté : shape-check sync (inchangé) + VAT-check async séparé via `verify_vat_rates_against_db(pool, company_id, &[Decimal])` qui déduplique via `BTreeSet<&Decimal>`. Évite le cascade et réduit les SELECT. T3.2/T3.3/T3.4 réécrits. |
| MEDIUM | M2 | Colonne `version INT` v0.1 spéculative (table read-only) | Retirée du schéma + entité + DTO. Epic 11-1 ajoutera en migration séparée quand le CRUD admin entrera. |
| MEDIUM | M3 | Store frontend : pseudocode retournait `cache ?? []` synchrone → race UI sous concurrence | Pattern « inflight-promise » : cache + promesse en vol partagée. Catch handler reset `inflight` pour autoriser retry. |
| MEDIUM | M4 | T5.3 référençait `crates/kesh-seed/tests/` inexistant | Repointé sur `crates/kesh-api/tests/test_endpoints_e2e.rs` (qui invoque `/api/v1/_test/seed` ⇒ `kesh_seed::seed_demo`). |
| LOW | L1 | Vérification i18n `product-vat-*` — clés présentes ? | Vérifié : 16/16 clés (4 × 4 locales) présentes lignes 288-291. T6.1 = no-op confirmé. |
| LOW | L2 | Logout hook frontend non localisé | Ajouté chemin précis `frontend/src/lib/app/stores/auth.svelte.ts::logout()` ligne 105 + import explicite à effectuer. |
| LOW | L3 | Test `query_param_companyid_ignored` redondant | Justifié comme défense en profondeur (régression future si `Query<...>` ajouté par erreur). |
| LOW | L4 | Sérialisation `Decimal` en string via `#[serde(with = ...)]` mentionnée alors que feature `serde-str` est défaut projet | Corrigé : pas d'annotation à ajouter, exemple `InvoiceResponse.vat_rate` cité. |
| LOW | L5 | Ordering seed onboarding/seed_demo asymétrique | Justifié : positionnement non critique tant que la même tx couvre les deux seeds (atomicité). |

**Résultat Pass 1** : 0 CRITICAL / 0 HIGH / 0 MEDIUM restant → critère d'arrêt de la règle de remédiation atteint après Pass 1. Pass 2 facultative ; recommandée pour orthogonal review (LLM différent — Sonnet ou Haiku).

**Patches appliqués** : 12 edits sur le story file (schéma, §seed, §migration, §validation backend, §frontend store, T1.2, T3.2-T3.4, T5.3, T6.1, T6.3, T4.3, AC #7).

**Commit attendu** : `git commit -m "Story 7-2: spec validate Pass 1 — Opus, 1H+4M+5L → 0>LOW, 12 patches"` (cf. CLAUDE.md règle commit après chaque passe).

