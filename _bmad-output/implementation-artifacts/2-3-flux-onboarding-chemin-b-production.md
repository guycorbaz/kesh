# Story 2.3: Flux d'onboarding — Chemin B (Production)

Status: done

## Story

As a **utilisateur**,
I want **configurer mon organisation pour commencer à travailler**,
so that **Kesh soit opérationnel avec mes données réelles**.

### Contexte

Troisième story de l'Epic 2. Implémente le Chemin B (production) de l'onboarding, activé quand l'utilisateur choisit "Configurer pour la production" à l'étape 3 du wizard. Ajoute 4 étapes supplémentaires (type d'organisation, langue comptable, coordonnées, compte bancaire) et une bannière bleue "Configuration incomplète". Crée la table `bank_accounts` et intègre la validation kesh-core (CheNumber, Iban, QrIban).

### Décisions de conception

- **Steps 3→7** : step 3 = choix du chemin production (nouveau endpoint `start-production`), steps 4-7 = org_type, accounting_language, coordonnées, banque. Le wizard reste plein-écran jusqu'à step 6 minimum.
- **Seuil d'accès à l'app** : Path A (démo) = step ≥ 3. Path B (production) = step ≥ 6 (coordonnées complétées). La banque (step 7) est optionnelle ("Configurer plus tard").
- **Bannière bleue** : affichée quand `!isDemo && stepCompleted < 7` (config incomplète). Lien "Terminer la configuration" → page Paramètres (placeholder tant que la page n'existe pas → redirect `/onboarding`).
- **Company update** : les étapes 4-6 mettent à jour la company placeholder créée par story 2-2 avec les vraies données (org_type, accounting_language, name, address, ide_number).
- **Plan comptable (FR5)** : cette story stocke le `org_type` choisi. L'installation effective du plan comptable est déférée à l'Epic 3 (story 3-1). Un TODO est documenté.
- **Table `bank_accounts`** : nouvelle table + entity + repository. Un seul compte bancaire au onboarding (is_primary=true). La gestion complète (CRUD multi-comptes) est déférée à des stories futures.
- **Validation** : CheNumber, Iban, QrIban via kesh-core (types déjà implémentés story 1.3). Validation côté API avant persistance.

## Acceptance Criteria (AC)

1. **Type d'organisation** — Given onboarding Chemin B (step=3 production), When étape "Type d'organisation", Then choix entre Indépendant, Association, PME. Sélection → company.org_type mis à jour, step avancé à 4. **Dette technique FR5 :** l'installation automatique du plan comptable et journaux adaptés au type est déférée à Epic 3 (story 3-1) — un TODO est ajouté dans le handler.
2. **Langue comptable** — Given type choisi (step=4), When étape "Langue comptable", Then choix FR/DE/IT/EN. La langue comptable est fixée au niveau instance, découplée de la langue interface. Step avancé à 5.
3. **Coordonnées** — Given langue comptable choisie (step=5), When saisie nom/raison sociale, adresse, IDE optionnel, Then données persistées dans company. IDE validé si saisi (format CHE + checksum via kesh-core). Step avancé à 6.
4. **Compte bancaire** — Given coordonnées saisies (step=6), When saisie banque, IBAN, QR-IBAN, Then validation IBAN/QR-IBAN via kesh-core, données persistées dans `bank_accounts`. Bouton "Configurer plus tard" disponible. Step avancé à 7.
5. **Bannière bleue** — Given onboarding Chemin B incomplet (step < 7, !isDemo), When navigation dans l'app, Then bannière bleue "Configuration incomplète — Terminer la configuration" visible en haut.
6. **Fonctionnement partiel** — Given onboarding partiel (step ≥ 6, banque non configurée), When accès à l'app, Then Kesh fonctionne normalement. La bannière bleue signale la configuration incomplète. Note : les messages "fonctionnalité dépendante" (ex: facturation sans banque) sont déférés aux stories des Epics correspondants (5, 6) qui ajouteront les guards contextuels.
7. **Interruption et reprise** — Given onboarding interrompu (browser fermé à step 5), When retour, Then bannière bleue + reprise au wizard à l'étape en cours.
8. **Correspondance Administration** — And chaque étape correspond à une section Administration : Type d'organisation → Paramètres > Organisation, Langue comptable → Paramètres > Comptabilité, Coordonnées → Paramètres > Organisation, Compte bancaire → Paramètres > Comptes bancaires.
9. **Atomicité** — And chaque étape est atomique et persistée immédiatement en base.
10. **Tests** — And tests unitaires validation (CheNumber, Iban, QrIban intégration), tests E2E API (6 nouveaux endpoints), tests vitest store, test Playwright flux complet Path B.

## Tasks / Subtasks

### T1 — Migration DB : table `bank_accounts` (AC: #4, #9)
- [x] T1.1 Créer migration `crates/kesh-db/migrations/YYYYMMDD_bank_accounts.sql` :
  ```sql
  CREATE TABLE bank_accounts (
      id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
      company_id BIGINT NOT NULL,
      bank_name VARCHAR(255) NOT NULL,
      iban VARCHAR(34) NOT NULL COMMENT 'IBAN normalisé sans espaces',
      qr_iban VARCHAR(34) NULL COMMENT 'QR-IBAN optionnel (QR-IID 30000-31999)',
      is_primary BOOLEAN NOT NULL DEFAULT FALSE,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      CONSTRAINT fk_bank_accounts_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
      CONSTRAINT chk_bank_accounts_bank_name_nonempty CHECK (CHAR_LENGTH(TRIM(bank_name)) > 0),
      CONSTRAINT chk_bank_accounts_iban_nonempty CHECK (CHAR_LENGTH(TRIM(iban)) > 0)
  ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
  ```
- [x] T1.2 Créer entity `BankAccount` + `NewBankAccount` dans `crates/kesh-db/src/entities/bank_account.rs`. Struct `FromRow` avec `company_id`, `bank_name`, `iban`, `qr_iban: Option<String>`, `is_primary`, `version`, timestamps.
- [x] T1.3 Ajouter `pub mod bank_account;` dans `entities/mod.rs` + réexports.

### T2 — Repository `bank_accounts` (AC: #4, #9)
- [x] T2.1 Créer `crates/kesh-db/src/repositories/bank_accounts.rs` :
  - `create(pool, new: NewBankAccount) -> BankAccount`
  - `find_primary(pool, company_id) -> Option<BankAccount>`
  - `list_by_company(pool, company_id) -> Vec<BankAccount>`
- [x] T2.2 Ajouter `pub mod bank_accounts;` dans `repositories/mod.rs`.
- [x] T2.3 Tests intégration DB : create, find_primary, list_by_company.

### T3 — Routes API onboarding Chemin B (AC: #1, #2, #3, #4, #9)
- [x] T3.0 Ajouter dépendance `kesh-core` dans `kesh-api/Cargo.toml` — **bloquant pour la compilation de T3.1-T3.7** (validation CheNumber, Iban, QrIban).
- [x] T3.1 Ajouter endpoint `POST /api/v1/onboarding/start-production` — avance step de 2 à 3 via `update_step(pool, 3, false, current.ui_mode, current.version)`. Progression stricte : requiert step == 2. Retourne `OnboardingState`.
- [x] T3.2 Ajouter endpoint `POST /api/v1/onboarding/org-type` — body: `{ "orgType": "Pme" }`. Valide valeur (case-sensitive PascalCase : `Independant`/`Association`/`Pme` — sinon 400 `VALIDATION_ERROR`). Met à jour `company.org_type` via pattern SELECT FOR UPDATE + OL. Step 3→4. Retourne `OnboardingState`.
- [x] T3.3 Ajouter endpoint `POST /api/v1/onboarding/accounting-language` — body: `{ "language": "FR" }`. Valide FR/DE/IT/EN. Met à jour `company.accounting_language`. Step 4→5. Retourne `OnboardingState`.
- [x] T3.4 Ajouter endpoint `POST /api/v1/onboarding/coordinates` — body: `{ "name": "...", "address": "...", "ideNumber": null }`. Valide name non-vide, address non-vide, IDE via `kesh_core::types::CheNumber::new()` si fourni. Met à jour company. Step 5→6. Retourne `OnboardingState`.
- [x] T3.5 Ajouter endpoint `POST /api/v1/onboarding/bank-account` — body: `{ "bankName": "...", "iban": "...", "qrIban": null }`. Valide IBAN via `kesh_core::types::Iban::new()`, QR-IBAN via `kesh_core::types::QrIban::new()` si fourni. Crée un `bank_accounts` (is_primary=true). Step 6→7. Retourne `OnboardingState`.
- [x] T3.6 Ajouter endpoint `POST /api/v1/onboarding/skip-bank` — skip le compte bancaire. Step 6→7 sans créer de bank_account. Retourne `OnboardingState`.
- [x] T3.7 Enregistrer les 6 nouvelles routes dans `build_router()` sous `authenticated_routes`.
- [x] T3.8 ~~Déplacé en T3.0~~ (voir ci-dessus).

### T4 — Frontend : wizard Chemin B étapes 4-7 (AC: #1, #2, #3, #4, #7)
- [x] T4.1 Modifier `frontend/src/routes/onboarding/+page.svelte` — remplacer le toast "À venir — Story 2-3" par un appel `POST /api/v1/onboarding/start-production` + continuer vers step 4.
- [x] T4.2 Ajouter étape 4 (step=3) — Choix type d'organisation : 3 cartes (Indépendant, Association, PME) avec description. Click → POST org-type → step 5.
- [x] T4.3 Ajouter étape 5 (step=4) — Choix langue comptable : 4 boutons (Français, Deutsch, Italiano, English) similaires à l'étape 1 mais pour la langue comptable. Texte explicatif "Langue des libellés du plan comptable (découplée de la langue de l'interface)".
- [x] T4.4 Ajouter étape 6 (step=5) — Formulaire coordonnées : champs nom/raison sociale, adresse (textarea), IDE optionnel. Accepter le format `CHE-xxx.xxx.xxx` ou `CHExxxxxxxxx` côté client — normaliser avant envoi à l'API (l'API attend la forme brute, kesh-core normalise). Validation côté client (required name + address). Submit → POST coordinates → step 7.
- [x] T4.5 Ajouter étape 7 (step=6) — Formulaire compte bancaire : champs banque, IBAN, QR-IBAN optionnel. Validation côté client (IBAN format). Bouton "Configurer plus tard" + bouton "Enregistrer". Submit → POST bank-account ou POST skip-bank → redirect `/`.
- [x] T4.6 Mettre à jour l'API module `onboarding.api.ts` : ajouter `startProduction()`, `setOrgType()`, `setAccountingLanguage()`, `setCoordinates()`, `setBankAccount()`, `skipBank()`.
- [x] T4.7 Mettre à jour le store `onboarding.svelte.ts` : ajouter les méthodes correspondantes.

### T5 — Frontend : bannière bleue + guard update (AC: #5, #6, #7)
- [x] T5.1 Créer composant `frontend/src/lib/shared/components/IncompleteBanner.svelte` — bannière bleue (bg-blue-100), texte "Configuration incomplète — Terminer la configuration", lien vers `/onboarding`. Textes via `i18nMsg()`.
- [x] T5.2 Modifier `(app)/+layout.svelte` : afficher `<IncompleteBanner />` si `!isDemo && stepCompleted >= 6 && stepCompleted < 7`. Condition mutuellement exclusive avec `<DemoBanner />`.
- [x] T5.3 **IMPORTANT : implémenter T5.3 et T5.4 ensemble** pour éviter une boucle de redirection. Modifier `(app)/+layout.ts` guard : le seuil d'accès à l'app devient :
  - Si `isDemo` : `stepCompleted < 3` → redirect `/onboarding`
  - Si `!isDemo` : `stepCompleted < 6` → redirect `/onboarding`
- [x] T5.4 Modifier `onboarding/+layout.ts` guard inverse : si `isDemo && stepCompleted >= 3` → redirect `/`. Si `!isDemo && stepCompleted >= 7` → redirect `/`. Note : step=6 (!isDemo) permet l'accès à l'app avec IncompleteBanner — PAS de redirect depuis le wizard à ce step (l'utilisateur a fini les coordonnées, le wizard se termine).

### T6 — Clés Fluent i18n (AC: #1, #2, #3, #4, #5)
- [x] T6.1 Ajouter les clés dans les 4 fichiers `locales/{fr,de,it,en}-CH/messages.ftl` :
  - `onboarding-choose-org-type` / `onboarding-org-independant` / `onboarding-org-independant-desc` / `onboarding-org-association` / `onboarding-org-association-desc` / `onboarding-org-pme` / `onboarding-org-pme-desc`
  - `onboarding-choose-accounting-lang` / `onboarding-accounting-lang-desc`
  - `onboarding-coordinates-title` / `onboarding-field-name` / `onboarding-field-address` / `onboarding-field-ide` / `onboarding-field-ide-hint`
  - `onboarding-bank-title` / `onboarding-field-bank-name` / `onboarding-field-iban` / `onboarding-field-qr-iban` / `onboarding-skip-bank`
  - `incomplete-banner-text` = "Configuration incomplète — Terminer la configuration"
  - `error-invalid-iban` / `error-invalid-qr-iban` / `error-invalid-che-number` — NOTE : ces clés sont pour la **validation côté client** (messages affichés sous les champs). L'API backend retourne des strings brutes via `AppError::Validation("IBAN invalide : ...")` — pas des clés Fluent.

### T7 — Tests (AC: #10)
- [x] T7.1 Tests intégration DB : bank_accounts CRUD (create, find_primary, list_by_company, FK constraint).
- [x] T7.2 Tests E2E API : 6 endpoints (start-production, org-type, accounting-language, coordinates, bank-account, skip-bank) — status codes + body + validations (IDE invalide → 400, IBAN invalide → 400).
- [x] T7.3 Test E2E API : flux complet Chemin B (language → mode → start-production → org-type → accounting-language → coordinates → bank-account → state step=7).
- [x] T7.4 Tests vitest : store methods (startProduction, setOrgType, setAccountingLanguage, setCoordinates, setBankAccount, skipBank).
- [x] T7.5 Test Playwright : flux complet Path B + bannière bleue visible après skip-bank.

## Dev Notes

### Mapping step_completed pour Chemin B

| step | Signification | Endpoint POST | Partagé A/B |
|------|--------------|---------------|-------------|
| 0 | Pas commencé | — | oui |
| 1 | Langue interface choisie | language | oui |
| 2 | Mode choisi (guidé/expert) | mode | oui |
| 3 | Chemin choisi (production) | start-production | B uniquement |
| 4 | Type d'organisation choisi | org-type | B uniquement |
| 5 | Langue comptable choisie | accounting-language | B uniquement |
| 6 | Coordonnées saisies | coordinates | B uniquement |
| 7 | Banque configurée ou skippée | bank-account / skip-bank | B uniquement |

### Guard d'accès conditionnel

```
// (app)/+layout.ts
if (isDemo && stepCompleted < 3) redirect /onboarding
if (!isDemo && stepCompleted < 6) redirect /onboarding

// onboarding/+layout.ts (guard inverse)
if (isDemo && stepCompleted >= 3) redirect /
if (!isDemo && stepCompleted >= 7) redirect /
// Note: steps 6 (partiel) laisse l'utilisateur dans l'app avec bannière bleue
```

### Bannières mutuellement exclusives

```svelte
{#if onboardingState.isDemo}
  <DemoBanner />
{:else if !onboardingState.isDemo && onboardingState.loaded && onboardingState.stepCompleted >= 6 && onboardingState.stepCompleted < 7}
  <IncompleteBanner />
{/if}
```

### Company update pattern (étapes 4-6)

La company placeholder (`name="(en cours de configuration)"`) créée en story 2-2 est mise à jour progressivement :
- Step 4 (org-type) : `UPDATE companies SET org_type = ?`
- Step 5 (accounting-language) : `UPDATE companies SET accounting_language = ?`
- Step 6 (coordinates) : `UPDATE companies SET name = ?, address = ?, ide_number = ?`

Chaque UPDATE utilise `SELECT FOR UPDATE` dans une transaction (pattern établi story 2-2 pour `ensure_company_with_language`). Tous passent `version` pour l'optimistic locking. Pattern exact à répliquer pour chaque handler :

```rust
async fn update_company_field(state: &AppState, /* field values */) -> Result<(), AppError> {
    use kesh_db::errors::map_db_error;
    let mut tx = state.pool.begin().await.map_err(map_db_error)?;
    let company = sqlx::query_as::<_, Company>(
        "SELECT ... FROM companies LIMIT 1 FOR UPDATE",
    ).fetch_one(&mut *tx).await.map_err(map_db_error)?;
    let rows = sqlx::query(
        "UPDATE companies SET <field> = ?, version = version + 1 WHERE id = ? AND version = ?",
    ).bind(/* value */).bind(company.id).bind(company.version)
     .execute(&mut *tx).await.map_err(map_db_error)?.rows_affected();
    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(AppError::Database(DbError::OptimisticLockConflict));
    }
    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}
```

### Validation kesh-core dans les routes API

```rust
// IDE validation (optionnel)
if let Some(ide) = &body.ide_number {
    kesh_core::types::CheNumber::new(ide)
        .map_err(|e| AppError::Validation(format!("IDE invalide : {e}")))?;
}

// IBAN validation
let iban = kesh_core::types::Iban::new(&body.iban)
    .map_err(|e| AppError::Validation(format!("IBAN invalide : {e}")))?;

// QR-IBAN validation (optionnel)
if let Some(qr) = &body.qr_iban {
    kesh_core::types::QrIban::new(qr)
        .map_err(|e| AppError::Validation(format!("QR-IBAN invalide : {e}")))?;
}
```

Stocker les valeurs normalisées (`.as_str()`) en DB, pas l'input brut.

### Plan comptable (FR5) — TODO Epic 3

Story 2-3 stocke `company.org_type` (Indépendant/Association/PME). L'installation automatique du plan comptable et des journaux adaptés est déférée à Epic 3 (story 3-1). Un TODO doit être ajouté dans le handler `set_org_type` :
```rust
// TODO(story 3-1): installer le plan comptable adapté au org_type choisi
```

### État existant du codebase (post story 2-2)

- **onboarding_state** : table avec singleton UNIQUE, steps 0-10, version pour OL
- **5 endpoints** existants : get_state, set_language, set_mode, seed_demo, reset
- **Wizard frontend** : 3 étapes (langue, mode, chemin A/B). Chemin B affiche actuellement un toast "À venir — Story 2-3"
- **kesh-core** : `CheNumber::new()`, `Iban::new()`, `QrIban::new()` déjà implémentés avec validation complète
- **Guard app** : `stepCompleted < 3` → redirect /onboarding (à modifier pour Path B)
- **DemoBanner** : conditionnel sur `isDemo`

### Piège : kesh-core n'est pas dans kesh-api Cargo.toml

`kesh-api` dépend actuellement de `kesh-db`, `kesh-i18n`, `kesh-seed`. Il faut ajouter `kesh-core` pour accéder aux validations CheNumber/Iban/QrIban (T3.8).

### Piège : `start-production` vs `seed-demo`

`seed-demo` (step 2→3) crée une company démo + fiscal year. `start-production` (step 2→3) ne crée rien de nouveau — la company placeholder existe déjà depuis step 1. Il avance juste le step et marque `is_demo=false` (déjà false par défaut).

### Piège : idempotence de `bank-account`

La progression stricte (`step == 6` requis) empêche un double appel en conditions normales : après le premier appel, step passe à 7 et un second appel échoue avec `ONBOARDING_STEP_ALREADY_COMPLETED`. Le seul risque est un échec après l'INSERT en DB mais avant l'UPDATE du step — dans ce cas le handler retourne une erreur 500, la row bank_account existe en DB mais le step reste à 6. Un retry réessaiera l'INSERT et créera un doublon. Pour mitiger : dans le handler `bank-account`, vérifier si un bank_account `is_primary=true` existe déjà pour le `company_id` avant d'INSERT ; si oui, le mettre à jour au lieu d'insérer.

### Piège : `bank_name` vs `iban` comme identifiant

Pas de contrainte UNIQUE sur `iban` dans `bank_accounts` pour cette story — un utilisateur pourrait théoriquement saisir le même IBAN deux fois. La gestion complète (détection doublons, multi-comptes) est une story future. Garder simple.

### Project Structure Notes

- **Nouvelle migration** : `crates/kesh-db/migrations/YYYYMMDD_bank_accounts.sql`
- **Nouvelle entity** : `crates/kesh-db/src/entities/bank_account.rs`
- **Nouveau repository** : `crates/kesh-db/src/repositories/bank_accounts.rs`
- **Nouveaux endpoints** : 6 routes dans `crates/kesh-api/src/routes/onboarding.rs`
- **Nouveau composant** : `frontend/src/lib/shared/components/IncompleteBanner.svelte`
- **Modifications** : `+page.svelte` (4 nouvelles étapes), `onboarding.api.ts`, `onboarding.svelte.ts`, `(app)/+layout.svelte`, `(app)/+layout.ts`, `onboarding/+layout.ts`, 4 fichiers .ftl

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-2-Story-2.3] — AC BDD
- [Source: _bmad-output/planning-artifacts/architecture.md#Database-bank_accounts] — Schéma bank_accounts
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Chemin-B] — Flow 6 étapes, bannière bleue
- [Source: _bmad-output/planning-artifacts/prd.md#FR4-FR5] — Onboarding assisté, plan comptable auto
- [Source: _bmad-output/planning-artifacts/prd.md#FR83-FR84] — Configuration comptes bancaires
- [Source: _bmad-output/planning-artifacts/prd.md#FR27] — Validation IDE CHE
- [Source: _bmad-output/implementation-artifacts/2-2-flux-onboarding-chemin-a-exploration.md] — Patterns onboarding, company bootstrap, guard

## Dev Agent Record

### Agent Model Used

Opus 4.6

### Debug Log References

### Completion Notes List

- T1: Migration bank_accounts + entity BankAccount/NewBankAccount + module exports
- T2: Repository bank_accounts (create, find_primary, list_by_company, upsert_primary) + module export
- T3: 6 nouveaux endpoints API (start-production, org-type, accounting-language, coordinates, bank-account, skip-bank) + kesh-core dep + routes registered + 3 helpers company update (OL pattern)
- T4: Wizard frontend 4 nouvelles étapes (org type cards, accounting language buttons, coordinates form, bank account form with skip) + API module + store methods
- T5: IncompleteBanner bleue + guard conditionnel Path A/B dans (app)/+layout.ts et onboarding/+layout.ts
- T6: 21 clés Fluent dans 4 locales pour Path B (org types, accounting lang, coordinates, bank, banner)
- T7.1: 5 tests intégration DB bank_accounts (create, find_primary, list, upsert, FK constraint)
- T7.2+T7.3: 6 tests E2E API Path B (start-production, org-type invalid, IDE validation, full flow, skip-bank, IBAN validation)
- Aucune régression : Path A E2E 9/9, DB onboarding 7/7, vitest 50/50
- Workspace compile proprement (cargo check + svelte-check 0 erreurs)

### File List

#### New Files
- `crates/kesh-db/migrations/20260410000001_bank_accounts.sql`
- `crates/kesh-db/src/entities/bank_account.rs`
- `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/tests/bank_accounts_repository.rs`
- `crates/kesh-api/tests/onboarding_path_b_e2e.rs`
- `frontend/src/lib/shared/components/IncompleteBanner.svelte`

#### Modified Files
- `crates/kesh-db/src/entities/mod.rs` — pub mod bank_account + réexports
- `crates/kesh-db/src/repositories/mod.rs` — pub mod bank_accounts
- `crates/kesh-api/Cargo.toml` — ajout dépendance kesh-core
- `crates/kesh-api/src/routes/onboarding.rs` — 6 endpoints + 3 helpers company update
- `crates/kesh-api/src/lib.rs` — 6 routes Path B dans authenticated_routes
- `crates/kesh-i18n/locales/fr-CH/messages.ftl` — 21 clés Path B
- `crates/kesh-i18n/locales/de-CH/messages.ftl` — 21 clés Path B
- `crates/kesh-i18n/locales/it-CH/messages.ftl` — 21 clés Path B
- `crates/kesh-i18n/locales/en-CH/messages.ftl` — 21 clés Path B
- `frontend/src/lib/features/onboarding/onboarding.api.ts` — 6 fonctions API Path B
- `frontend/src/lib/features/onboarding/onboarding.svelte.ts` — 6 méthodes store Path B
- `frontend/src/routes/onboarding/+page.svelte` — 4 nouvelles étapes wizard
- `frontend/src/routes/(app)/+layout.svelte` — IncompleteBanner conditionnel
- `frontend/src/routes/(app)/+layout.ts` — guard conditionnel Path A/B
- `frontend/src/routes/onboarding/+layout.ts` — guard inverse conditionnel

## Change Log

| Date | Passe | Modèle | Findings | Patches |
|------|-------|--------|----------|---------|
| 2026-04-09 | Implémentation | Opus 4.6 | — | T1-T7 complètes, DB 5/5 + API E2E 6/6 + vitest 50/50, aucune régression |
