# Story 2.6: Onboarding — Comptes facturation pré-remplis

**Status:** ready-for-dev

## Story

**As a** nouvel utilisateur Kesh,
**I want** que les comptes par défaut de facturation soient pré-sélectionnés à la fin de l'onboarding (avec comptes standard Suisse : 1100 Clients, 3000 Ventes),
**So that** je puisse valider ma première facture sans détour par une page de configuration.

## Contexte

Créée en backlog le 2026-04-14 par Guy Corbaz lors du démarrage de la Story 5.2. Les comptes par défaut de facturation (`default_receivable_account_id`, `default_revenue_account_id`) sont **obligatoires** pour valider une facture (Story 5.2), sinon réponse `400 CONFIGURATION_REQUIRED`. UX actuelle sous-optimale : l'utilisateur découvre la contrainte à la 1ère tentative de validation.

La Story 5.2 (Validation & numérotation) a créé la table `company_invoice_settings` avec ces champs. La Story 3.1 (Plan comptable) charge le plan standard pour le type d'organisation choisi à l'onboarding. **Cette story complète le cycle** : lors de la finalisation de l'onboarding, pré-remplir automatiquement les comptes par défaut si le plan comptable les contient.

### Dépendances

- **Story 3.1 (Plan comptable)** : ✅ done — charge `accounts` selon le type d'org (PME, Association, Indépendant)
- **Story 2.3 (Onboarding Chemin B)** : ✅ done — finit l'onboarding et persiste le `company_id`
- **Story 5.2 (Validation & numérotation)** : ✅ done — crée la table `company_invoice_settings`, avec FK et repos existant
- **FR4-FR5 (PRD)** : Onboarding + auto-installation du plan comptable

### Blocage levé

Sans cette story, la première validation facture est bloquée par `400 CONFIGURATION_REQUIRED` jusqu'à que l'utilisateur accède manuellement à `/settings/invoicing`. Cette story rend le path heureux entièrement autonome dans l'onboarding.

---

## Acceptance Criteria

### AC 1 : Pré-remplissage à la fin de l'onboarding

**Given** utilisateur en fin d'onboarding (Chemin A démo ou Chemin B production),
**When** étape finale validée et persist en base (company créée, plan comptable chargé),
**Then** les comptes par défaut sont automatiquement pré-sélectionnés :
- `default_receivable_account_id` = `id` de l'account `1100` ("Clients"), s'il existe
- `default_revenue_account_id` = `id` de l'account `3000` ("Ventes"), s'il existe
- Autres champs `company_invoice_settings` restent à leurs valeurs defaults (format `F-{YEAR}-{SEQ:04}`, journal `Ventes`, template `{YEAR}-{INVOICE_NUMBER}`)

### AC 2 : Comptes standards par plan comptable

**Given** un type d'organisation (PME, Association, Indépendant),
**When** plan comptable chargé en Story 3.1,
**Then** les 3 plans standards suisses DOIVENT contenir ces numéros de comptes :
- `1100` (Asset, "Clients" ou "Créances")
- `3000` (Revenue, "Ventes" ou "Produits")

**Et** documenter dans le code ou en commentaire que les comptes `1100` et `3000` sont les numéros standard suisses, garantis présents dans les plans fournis.

### AC 3 : Fallback si comptes manquants

**Given** plan comptable chargé dont les codes `1100` et/ou `3000` n'existent pas (plans alternatifs futurs),
**When** fin de l'onboarding,
**Then** afficher un écran optionnel de sélection des comptes par défaut :
- Libellé : "Configurer les comptes de facturation"
- Deux dropdowns : "Compte clients (débiteur)" et "Compte ventes (revenu)"
- Remplir automatiquement les comptes `1100` et `3000` si présents (ne pas forcer l'utilisateur à chercher)
- Bouton "Configurer" : persiste la sélection dans `company_invoice_settings`
- Bouton "Configurer plus tard" : saute l'écran, `company_invoice_settings` créée avec `NULL` pour les deux FKs (prise de config ultérieure)

### AC 4 : Demande de configuration explicite en cas de `NULL`

**Given** utilisateur avec `default_receivable_account_id = NULL` ou `default_revenue_account_id = NULL`,
**When** accès au formulaire de création de facture (`/invoices/create`),
**Then** afficher un banner orange (warning) : "Configuration incomplète — Configuration des comptes de facturation requise" avec lien vers `/settings/invoicing`

**Et** le bouton "Valider facture" doit être désactivé avec tooltip : "Configurez d'abord les comptes de facturation dans les paramètres".

### AC 5 : Test E2E — Chemin A (démo)

**Given** utilisateur lance l'app pour la 1ère fois,
**When** onboarding : sélection langue → mode Guidé/Expert → choix "Données de démo",
**Then** script seed charge un company avec plan comptable PME + pré-remplissage auto :
- `company_invoice_settings.default_receivable_account_id` est défini et non-NULL
- `company_invoice_settings.default_revenue_account_id` est défini et non-NULL
- Utilisateur accède à `/invoices/create`, peut créer une facture et cliquer "Valider" sans erreur `CONFIGURATION_REQUIRED`

### AC 6 : Test E2E — Chemin B (production)

**Given** utilisateur lance l'app pour la 1ère fois,
**When** onboarding : sélection langue → mode Guidé/Expert → choix "Production" → type "Indépendant" → coordonnées → compte bancaire → validation finale,
**Then** plan comptable "Indépendant" chargé et pré-remplissage auto appliqué :
- `company_invoice_settings` créée avec accounts `1100` et `3000` pré-sélectionnés
- Utilisateur accède immédiatement à `/invoices/create` et peut valider une facture sans détour config

### AC 7 : Tests unitaires — Logique pré-remplissage

- Test: `company_invoice_settings::insert_with_defaults` crée row avec `default_receivable_account_id` et `default_revenue_account_id` correctement mappés depuis les accounts du plan comptable
- Test: si accounts `1100` ou `3000` manquants, retour gracieux (NULL ou erreur attendue, à documenter)
- Test: l'ordre des migrations garantit que `accounts` est persisté avant `company_invoice_settings` (FK integrity)

---

## Spécifications Techniques

### Backend — Rust / kesh-db / kesh-api

#### T1 : Repository — Fonction de pré-remplissage

**Fichier:** `crates/kesh-db/src/repositories/company_invoice_settings.rs`

Ajouter fonction :

```rust
/// Crée company_invoice_settings avec pré-remplissage auto des comptes par défaut (1100, 3000)
/// Utilisée lors de la finalisation de l'onboarding (après chargement du plan comptable)
pub async fn insert_with_defaults(
    pool: &PgPool,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, RepositoryError> {
    // 1. Chercher accounts 1100 et 3000 pour ce company
    let receivable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = $1 AND number = $2 AND active = true LIMIT 1"
    )
    .bind(company_id)
    .bind("1100")
    .fetch_optional(pool)
    .await?
    .flatten();

    let revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = $1 AND number = $2 AND active = true LIMIT 1"
    )
    .bind(company_id)
    .bind("3000")
    .fetch_optional(pool)
    .await?
    .flatten();

    // 2. INSERT avec les valeurs par défaut
    sqlx::query_as::<_, CompanyInvoiceSettings>(
        r#"
        INSERT INTO company_invoice_settings 
            (company_id, invoice_number_format, default_receivable_account_id, 
             default_revenue_account_id, default_sales_journal, journal_entry_description_template)
        VALUES ($1, 'F-{YEAR}-{SEQ:04}', $2, $3, 'Ventes', '{YEAR}-{INVOICE_NUMBER}')
        RETURNING *
        "#
    )
    .bind(company_id)
    .bind(receivable)
    .bind(revenue)
    .fetch_one(pool)
    .await
    .map_err(RepositoryError::from)
}
```

**Appel:** Depuis le handler d'onboarding (voir T2).

#### T2 : Intégration dans le flow d'onboarding

**Fichiers concernés:**
- `crates/kesh-api/src/routes/onboarding.rs` — endpoint `POST /api/v1/onboarding/finalize` (ou similar)
- `crates/kesh-api/src/services/onboarding.rs` — service layer

**Logique:**

Lors de la finalisation de l'onboarding (après création du `company` et chargement du plan comptable) :

```rust
// Dans le service finalize_onboarding()
// 1. Créer le company (déjà existant)
// 2. Charger le plan comptable (déjà Story 3.1)
// 3. NOUVEAU : pré-remplir les comptes de facturation
let invoice_settings = company_invoice_settings::insert_with_defaults(
    &state.pool,
    company_id,
).await?;

// 4. Retourner confirmation avec les settings pré-remplis
Ok(Json(OnboardingFinalizeResponse {
    company_id,
    company_name,
    chart_loaded: true,
    invoice_settings: invoice_settings.into(),
}))
```

#### T3 : Gestion du fallback (AC 3)

Si la Story 2.6 doit supporter des plans comptables alternatifs (sans codes 1100/3000), ajouter un endpoint optionnel :

**Fichier:** `crates/kesh-api/src/routes/onboarding.rs`

```rust
/// Optionnel : step de sélection des comptes si 1100/3000 manquants
#[derive(Deserialize)]
pub struct SelectDefaultAccountsRequest {
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
}

pub async fn select_default_accounts(
    State(state): State<AppState>,
    Json(body): Json<SelectDefaultAccountsRequest>,
) -> Result<StatusCode, AppError> {
    let company_id = /* extract from JWT */;
    
    // Update company_invoice_settings
    company_invoice_settings::update(
        &state.pool,
        company_id,
        &CompanyInvoiceSettingsUpdate {
            invoice_number_format: /* existing */,
            default_receivable_account_id: body.default_receivable_account_id,
            default_revenue_account_id: body.default_revenue_account_id,
            default_sales_journal: Journal::Ventes,
            journal_entry_description_template: /* existing */,
        },
    ).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

**Route:** `POST /api/v1/onboarding/select-default-accounts` (si nécessaire).

### Frontend — Svelte/TypeScript

#### T4 : Écran optionnel de sélection des comptes (AC 3)

**Fichier:** `frontend/src/routes/(app)/onboarding/+page.svelte`

Ajouter un **step optionnel** après la finalisation si les comptes `1100` ou `3000` manquent:

```svelte
<!-- Fragment du flux onboarding -->
{#if onboardingStep === 'select-default-accounts'}
    <div class="onboarding-step">
        <h2>Configurer les comptes de facturation</h2>
        <p>Sélectionnez les comptes par défaut pour la facturation (optionnel)</p>
        
        <label>Compte clients (débiteur)
            <select bind:value={selectedReceivableId}>
                <option value={null}>Aucun (configurer plus tard)</option>
                {#each accounts.filter(a => a.type === 'Asset') as account}
                    <option value={account.id}>
                        {account.number} — {account.name}
                    </option>
                {/each}
            </select>
        </label>
        
        <label>Compte ventes (revenu)
            <select bind:value={selectedRevenueId}>
                <option value={null}>Aucun (configurer plus tard)</option>
                {#each accounts.filter(a => a.type === 'Revenue') as account}
                    <option value={account.id}>
                        {account.number} — {account.name}
                    </option>
                {/each}
            </select>
        </label>
        
        <button on:click={handleConfigureAccounts}>Configurer</button>
        <button on:click={handleSkipAccountConfiguration}>Configurer plus tard</button>
    </div>
{/if}
```

**Logique:**
- Si `company_invoice_settings.default_receivable_account_id !== NULL` et `default_revenue_account_id !== NULL` → skip ce step
- Si l'un est NULL → afficher ce step avec pré-remplissage si possibles
- Bouton "Configurer plus tard" → appel API optionnel, skip directement vers l'accueil

#### T5 : Banner d'avertissement sur page creation facture (AC 4)

**Fichier:** `frontend/src/routes/(app)/invoices/create/+page.svelte`

Au chargement, vérifier `invoiceSettings`:

```svelte
<script>
    import { page } from '$app/stores';
    import { toast } from '$lib/components/Toast.svelte';
    
    let invoiceSettings = $page.data.invoiceSettings;
    
    $: {
        if (!invoiceSettings?.defaultReceivableAccountId || 
            !invoiceSettings?.defaultRevenueAccountId) {
            showConfigWarning = true;
        }
    }
</script>

{#if showConfigWarning}
    <Banner type="warning" icon="AlertTriangle">
        Configuration incomplète — 
        <a href="/settings/invoicing">Configurez les comptes de facturation</a>
    </Banner>
    <button disabled title="Configurez d'abord les comptes de facturation">
        Valider facture
    </button>
{/if}
```

### Base de données — Migrations

**Fichier:** Aucune nouvelle migration — la table `company_invoice_settings` existe déjà (Story 5.2)

**Changements:** Aucun schéma — seule la logique de pré-remplissage lors de l'insertion (AC 1, T1).

---

## Scénarios de Seed et E2E

### Seed Data (kesh-seed)

**Fichier:** `crates/kesh-seed/src/main.rs`

Lors de la création d'une company en mode démo, appeler `company_invoice_settings::insert_with_defaults` :

```rust
// Dans seed_company()
let company = create_company(pool, "Demo Corp", org_type::Pme).await?;

// Charger le plan comptable (déjà fait)
load_chart_of_accounts(pool, company.id, org_type::Pme).await?;

// NOUVEAU: pré-remplir les comptes de facturation
let settings = company_invoice_settings::insert_with_defaults(pool, company.id).await?;
println!("Invoice settings auto-configured: {:?}", settings);
```

### Test E2E — Playwright

**Fichier:** `frontend/tests/e2e/onboarding.spec.ts`

Ajouter 2 tests:

#### Test 1 : Chemin A (démo)

```typescript
test('Onboarding Path A: Demo mode auto-configures invoice settings', async ({ page }) => {
    // 1. Visiter app → onboarding
    await page.goto('/');
    
    // 2. Sélectionner langue (ex: Français)
    await page.click('button[aria-label*="Français"]');
    
    // 3. Mode Guidé
    await page.click('button:has-text("Guidé")');
    
    // 4. Sélectionner "Données de démo"
    await page.click('button:has-text("Données de démo")');
    
    // 5. Attendre que l'onboarding se termine
    await page.waitForURL('/');
    
    // 6. Naviguer vers creation de facture
    await page.goto('/invoices/create');
    
    // 7. Vérifier que le formulaire est accessible (pas de banner d'avertissement)
    const warningBanner = page.locator('text=Configuration incomplète');
    await expect(warningBanner).not.toBeVisible();
    
    // 8. Vérifier que le bouton "Valider" est activé
    const validateBtn = page.locator('button:has-text("Valider")');
    await expect(validateBtn).toBeEnabled();
});
```

#### Test 2 : Chemin B (production)

```typescript
test('Onboarding Path B: Production mode auto-configures invoice settings', async ({ page }) => {
    // 1. Visiter app → onboarding
    await page.goto('/');
    
    // 2. Sélectionner langue
    await page.click('button[aria-label*="Français"]');
    
    // 3. Mode Expert
    await page.click('button:has-text("Expert")');
    
    // 4. Sélectionner "Production"
    await page.click('button:has-text("Production")');
    
    // 5. Type d'organisation: Indépendant
    await page.click('button:has-text("Indépendant")');
    
    // 6. Saisir coordonnées
    await page.fill('input[name="company_name"]', 'Mon Business SARL');
    await page.fill('input[name="address"]', '123 Rue des Alpes, 1200 Genève');
    await page.click('button:has-text("Suivant")');
    
    // 7. Saisir compte bancaire
    await page.fill('input[name="iban"]', 'CH9300762011623852957');
    await page.click('button:has-text("Terminer onboarding")');
    
    // 8. Attendre accueil
    await page.waitForURL('/');
    
    // 9. Naviguer vers factures et créer une facture test
    await page.goto('/invoices/create');
    await page.fill('input[name="client_name"]', 'ACME Corp');
    await page.fill('input[name="amount"]', '1000.00');
    
    // 10. Cliquer "Valider" — ne doit pas échouer avec CONFIGURATION_REQUIRED
    await page.click('button:has-text("Valider")');
    
    // 11. Vérifier que la facture est validée (redirection vers liste ou confirmation)
    await expect(page).toHaveURL(/\/(invoices|home)/);
});
```

---

## Developer Context (Contexte dev)

### Architecture & Patterns

**Pattern existant — Lazy creation & upsert** (story 5.2):
- `company_invoice_settings` est créée on-demand la 1ère fois qu'elle est accédée (GET endpoint avec `INSERT IGNORE`)
- Les migrations Story 5.2 créent la table avec une FK vers accounts (`default_receivable_account_id`, `default_revenue_account_id`)

**Ce qui change (story 2.6)**:
- Au lieu de créer avec `NULL` pour les account IDs, pré-remplir avec les codes `1100` et `3000` si existants dans le plan comptable
- Appeler avant que l'utilisateur ne crée une facture (au finish de l'onboarding)

**Leçons de Story 2.5 (Mode Guidé/Expert)**:
- Persistence double (localStorage + serveur) fonctionne bien
- Les mises à jour non-bloquantes (fire-and-forget) acceptables pour les paramètres utilisateur
- Tests E2E via Playwright couvrent les paths critiques

### Files Modified / Created

| File | Type | Note |
|------|------|------|
| `crates/kesh-db/src/repositories/company_invoice_settings.rs` | Modify | Ajouter `insert_with_defaults()` |
| `crates/kesh-api/src/routes/onboarding.rs` | Modify | Appeler pré-remplissage à la fin d'onboarding |
| `frontend/src/routes/(app)/onboarding/+page.svelte` | Modify | Step optionnel de sélection si fallback |
| `frontend/src/routes/(app)/invoices/create/+page.svelte` | Modify | Banner d'avertissement si config incomplète |
| `crates/kesh-seed/src/main.rs` | Modify | Appeler pré-remplissage en mode démo |
| `frontend/tests/e2e/onboarding.spec.ts` | Modify | Ajouter 2 tests E2E |

### Code Quality Standards

- **i18n:** Clés pour banner "Configuration incomplète" (FR, DE, IT, EN)
- **Testing:** Tests E2E couvrant les 2 paths (démo + production), tests unitaires pour la repo fonction
- **Documentation:** Commenter en code que les comptes `1100` et `3000` sont des standards suisses
- **Validation:** Les FKs en base garantissent l'intégrité ; côté frontend, afficher un message clair si configuration manquante

---

## Change Log

| Date       | Version | Description                                                | Auteur           |
|------------|---------|-------------------------------------------------------------|------------------|
| 2026-04-14 | 0.1     | Stub créé en backlog pendant démarrage Story 5.2            | Claude Opus 4.6  |
| 2026-04-21 | 1.0     | Spec complète issue de `bmad-create-story` — AC, tech spec, tests E2E | Claude Haiku 4.5 |

---

## Notes pour le dev

1. **Dépendance forte avec Story 3.1** : Le code assume que le plan comptable est chargé (accounts avec numéros 1100, 3000 existant). Cette story dépend de 3.1 être `done`.

2. **Optionalité du fallback** : L'AC 3 (fallback avec dropdown si comptes manquants) est optionnel pour cette itération si les 3 plans standards suisses fournis CONTIENNENT TOUJOURS 1100 et 3000. Documente cette assomption.

3. **Appel timing** : Appeler `insert_with_defaults()` **après** le chargement du plan comptable, pendant la finalisation de l'onboarding (pas avant, ou les accounts n'existent pas).

4. **i18n** : La clé d'avertissement "Configuration incomplète" doit être présente dans les 4 locales. Ajouter les clés:
   - `config-incomplete-title = Configuration incomplète`
   - `config-incomplete-link = Configurez les comptes de facturation`
   - `invoice-settings-required = Configurez d'abord les comptes de facturation`

5. **Rollback plan** : Si la pré-remplissage automatique cause des problèmes (ex: comptes 1100/3000 inexistants), révert le changement et utiliser le fallback (AC 3) ou forcer la configuration manuelle avant validation.
