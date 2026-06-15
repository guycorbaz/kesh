---
status: ready-for-dev
epic: 18
story: 18-1a
type: story-zero
parent: 18-1
issue: 180
created: 2026-06-14
stepsCompleted: []
---

# Story 18-1a (story-zéro) — Comptes TVA dans le plan comptable + config

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md)
> (validate 5 passes, DC1-DC9 figés). **Story-zéro** : pose les comptes TVA et les champs de configuration
> que 18-1b (ventes), 18-1c (achats), 18-1d (rapport) consomment. **Aucune comptabilisation ici** — juste
> la fondation (comptes + colonnes settings + UI config + migration).

## User Story

**En tant qu'**administrateur d'une PME suisse,
**je veux** disposer dans mon plan comptable des comptes TVA corrects (impôt préalable / TVA récupérable,
TVA due, compte de décompte) et pouvoir les désigner comme comptes TVA par défaut,
**afin de** préparer la comptabilisation de la TVA (18-1b/c) et le décompte AFC.

## Contexte ground-truth (vérifié `main` @ `e08bd21`)

- **Plans** `crates/kesh-core/assets/charts/{pme,independant,association}.json` : ont `1170` (Impôt
  anticipé / Verrechnungssteuer, Asset, parent `10`) + `2200` (TVA due, Liability, parent `20`). `2201`
  (Impôt anticipé dû) **uniquement dans pme**. **`1171` et `2206` sont LIBRES dans les 3 plans** (vérifié
  grep, 0 collision).
- **Chargement** `chart_of_accounts/mod.rs:71-85` (`load_chart`) + résolution `parentNumber → parent_id`
  via `bulk_create_from_chart` (`accounts.rs:413-459`, tri topologique).
- **Table** `company_invoice_settings` créée dans `crates/kesh-db/migrations/20260417000001_invoice_validation.sql:35-51`
  (PK `company_id`, FK vers `accounts`, lazy `INSERT IGNORE`, verrou optimiste `version`). Champs comptes
  actuels : `default_receivable_account_id`, `default_revenue_account_id` (nullable, FK `fk_cis_*`).
- **Entité** `crates/kesh-db/src/entities/company_invoice_settings.rs` (`CompanyInvoiceSettings` +
  `CompanyInvoiceSettingsUpdate`).
- **Repo** `crates/kesh-db/src/repositories/company_invoice_settings.rs` ; **route**
  `crates/kesh-api/src/routes/company_invoice_settings.rs` (`PUT /company/invoice-settings`, remplacement
  intégral + OL).
- **Frontend** `frontend/src/routes/(app)/settings/invoicing/+page.svelte` (config facturation existante,
  sélecteurs de comptes) + `settings/vat-rates/+page.svelte` (taux TVA, Story 11-1).
- **Migrations** : 34 actuelles ; audit `docs/migrations-idempotence-audit.md` à compléter (politique P5).

## Décisions figées (héritées umbrella DC1, DC8)

- **Numéros & libellés des nouveaux comptes** (FIGÉ ici — confirmer non-collision au dev) :
  - **`1171` Impôt préalable** (TVA récupérable sur achats), type **Asset**, parent `10`.
    Libellés : FR « Impôt préalable », DE « Vorsteuer », IT « Imposta precedente », EN « Input VAT ».
  - **`2206` Décompte TVA** (solde net dû à l'AFC), type **Liability**, parent `20`.
    Libellés : FR « Décompte TVA », DE « MWST-Abrechnung », IT « Rendiconto IVA », EN « VAT settlement ».
  - **NE PAS renommer** `1170` ni `2201` (impôt anticipé / Verrechnungssteuer — sémantique distincte).
- **Migration non-breaking** (DC8) : `ADD COLUMN nullable` + FK + INSERT idempotent → **pas** de bump
  `kesh_version_min_required` (anciens binaires ignorent les nouvelles colonnes/comptes). Ligne audit
  idempotence obligatoire (P5).

## Acceptance Criteria

- **AC1** — Les 3 plans `.json` contiennent `1171` (Impôt préalable, Asset, parent `10`) et `2206`
  (Décompte TVA, Liability, parent `20`) avec libellés FR/DE/IT/EN ci-dessus. Une nouvelle installation
  (seed via `bulk_create_from_chart`) crée donc ces comptes automatiquement.
- **AC2** — Nouvelle migration `crates/kesh-db/migrations/<ts>_vat_accounts_config.sql` :
  - `ALTER TABLE company_invoice_settings ADD COLUMN default_vat_payable_account_id BIGINT NULL`
    + `default_vat_recoverable_account_id BIGINT NULL` + `default_vat_decompte_account_id BIGINT NULL`,
    chacun avec FK `REFERENCES accounts(id) ON DELETE RESTRICT` (pattern `fk_cis_*`).
  - **Migration data idempotente par company** : pour chaque company existante, **INSERT** `1171` et
    `2206` s'ils n'existent pas déjà (`WHERE NOT EXISTS … number=`), avec `parent_id` résolu par
    sous-requête corrélée (`SELECT id FROM accounts WHERE company_id=c.id AND number='10'|'20'`) ;
    `parent_id = NULL` toléré si le parent n'existe pas (plan custom). Libellé inséré dans la **locale
    comptable de la company** (cohérent `bulk_create_from_chart`, voir comment celui-ci choisit la
    langue) — sinon FR par défaut, documenté.
  - Migration **non-breaking** (pas de bump min_required) + **ligne ajoutée à
    `docs/migrations-idempotence-audit.md`** (verdict `tracked-by-sqlx`, +1 au total 34→35).
- **AC3** — La migration data **ne touche aucun compte existant** (pas d'UPDATE de `1170`/`2200`/`2201`).
  Idempotente : rejouable sans doublon (garantie par `uq_accounts_company_number` + `NOT EXISTS`).
- **AC4** — Entité `CompanyInvoiceSettings` + `CompanyInvoiceSettingsUpdate` étendues des 3 champs TVA
  (`Option<i64>`, camelCase `defaultVatPayableAccountId` / `…RecoverableAccountId` / `…DecompteAccountId`).
- **AC5** — Repo `company_invoice_settings` (SELECT/UPDATE) + route `PUT /company/invoice-settings`
  gèrent les 3 nouveaux champs (remplacement intégral, verrou optimiste inchangé). Validation : si fourni,
  l'`account_id` doit appartenir à la company (anti-IDOR, comme receivable/revenue) et idéalement être du
  bon type (payable/decompte = Liability, recoverable = Asset) — au minimum exister dans la company.
- **AC6** — Frontend `settings/invoicing/+page.svelte` : 3 sélecteurs de compte supplémentaires (TVA due,
  TVA récupérable, Décompte TVA), pré-remplis si configurés, alimentés par la liste des comptes de la
  company (même source que receivable/revenue). HTTP-LAN-safe (cf. [[feedback_no_secure_context_apis_http_lan]],
  `$props.id()` pour IDs DOM, pas de bug #143/#145).
- **AC7** — i18n FR/DE/IT/EN des nouveaux libellés UI (labels champs + hints) + lint-i18n-ownership vert.
- **AC8** — Tests : (a) test repo/route round-trip des 3 champs ; (b) test migration data (company
  existante sans `1171`/`2206` → comptes créés après migration, idempotent au re-run) ; (c) test
  anti-IDOR sur les nouveaux account_id ; (d) non-régression validation facture (receivable/revenue
  inchangés). `password_reset_tokens`/schéma : si un test compte les migrations (`34→35`) ou les comptes
  seedés, le mettre à jour (fail-loud attendu).

## Tasks (T-A1..T-A7)

- **T-A1** — Ajouter `1171` + `2206` aux 3 `.json` (libellés 4 langues). Vérifier non-collision + parents
  `10`/`20` présents dans chaque plan.
- **T-A2** — Migration SQL : ADD COLUMN ×3 + FK ×3 + INSERT data idempotent/company (parent sous-requête).
- **T-A3** — Étendre entité `CompanyInvoiceSettings` + `CompanyInvoiceSettingsUpdate` (3 champs).
- **T-A4** — Repo SELECT/UPDATE + route `PUT` : porter les 3 champs + validation appartenance company.
- **T-A5** — Frontend `settings/invoicing` : 3 sélecteurs + binding + appel PUT.
- **T-A6** — i18n ×4 + ligne `docs/migrations-idempotence-audit.md` (total 34→35).
- **T-A7** — Tests (AC8) + quality gate « Test Locally First » (fmt/clippy/build/test backend + check/lint/test:unit/build frontend).

## Hors-scope (→ stories suivantes)

- Comptabilisation TVA aux ventes (18-1b), helper achats (18-1c), remplissage `VatReport` récupérable
  (18-1d), réconciliation (18-1e). 18-1a **ne poste aucune écriture** et ne lit pas le rapport.
- Rendre les comptes TVA **obligatoires** à la validation : NON en 18-1a (nullable). 18-1b décidera si
  `default_vat_payable_account_id` NULL bloque (probable `CONFIGURATION_REQUIRED`, à spécifier en 18-1b).

## Risques

- **Locale des libellés à la migration data** : `bulk_create_from_chart` choisit la langue selon la
  company (locale comptable). La migration SQL pure ne connaît pas cette logique → vérifier comment
  obtenir la locale (colonne sur `companies` ?) ou insérer FR par défaut + documenter (les libellés sont
  éditables via le CRUD plan comptable de toute façon). **À trancher au dev T-A2** (lire le ground-truth
  de `bulk_create_from_chart` pour la sélection de langue).
- **FK `ON DELETE RESTRICT`** : cohérent avec `fk_cis_*` existants — un compte désigné ne peut être
  supprimé tant que référencé. OK.

## Prochaine étape

`bmad-dev-story 18-1a` (Opus recommandé — migration data + cross-crate entité/repo/route/frontend).

## Review Findings

### Pass 1 (Sonnet — Blind Hunter + Edge Case Hunter + Acceptance Auditor)

- [x] [Review][Patch] AC8(c) anti-IDOR : test cross-company manquant — le test livré (`update_vat_account_foreign_id_rejected_by_fk`) couvre le garde FK DB (id inexistant) mais PAS l'IDOR (compte valide d'une autre company). Patch : `idor_invoice_settings_vat_account_cross_company_rejected` ajouté dans `crates/kesh-api/tests/idor_multi_tenant_e2e.rs` (PUT settings avec compte Liability de company B → 400 ; contrôle positif compte propre → 200). Vert.
- [x] [Review][Patch] Indentation `<section>` TVA — `<h2>` + enfants à 4 tabs au lieu de 3 (bloc sur-indenté d'un niveau) [`frontend/.../settings/invoicing/+page.svelte:242`]. Dédenté d'un tab. `npm run check` 0 err.
- [x] [Review][Defer] Archive d'un compte TVA configuré laisse FK CIS obsolète → bloque la sauvegarde settings [`crates/kesh-db/src/repositories/accounts.rs`] — **pré-existant** (`default_receivable`/`revenue` ont le même gap, pas une régression 18-1a) ; 18-1a multiplie l'exposition de 2 à 5 colonnes FK. → Issue GitHub recommandée.
- [x] [Review][Defer] `admin_backup_e2e` ne vérifie pas l'intégrité FK des 3 nouvelles colonnes VAT après restore — narrow (ne se déclenche qu'avec TVA configurée + round-trip). → 18-1f (tests).
- [x] [Review][Defer] Pas de contrainte `vat_payable ≠ vat_decompte` (même compte Liability acceptable pour les deux) — concern design consommé en 18-1b. → 18-1b.
- Dismiss (réfutés grep ground-truth) : `assetAccounts` non défini (FAUX — défini ligne 47 filtre `Asset`) ; DRY SELECT/COLUMNS + tests inline (hygiène LOW correctement gérée dans le diff).

### Pass 2 (Haiku — diff aplati main vs HEAD, 0 hallucination)

- [x] [Review][Patch] M1 : le test anti-IDOR assertait seulement le statut 400 sans épingler la cause (un 400 incident pourrait masquer le garde). Mitigé par le contrôle positif (même payload, seul l'id change), mais durci : assert `error.code == "VALIDATION_ERROR"` ajouté [`crates/kesh-api/tests/idor_multi_tenant_e2e.rs`]. Vert.
- Blind Hunter : C1/H1 auto-dismiss par l'agent (colonnes bien SELECT, `null`→`Option::None` correct). Edge Case Hunter : 0 unhandled. Acceptance Auditor : AC1-AC8 tous MET, DC1/DC8 honorés, 0 finding.

### Pass 3 (Opus — diff aplati ; catch-architectural)

- [x] [Review][Patch] **HIGH** — backfill migration injecte 2 comptes TVA orphelins dans une company **stub** (bootstrap, non-onboardée, 0 compte), ce qui casse le garde de seed onboarding `if existing == 0` → seed du plan COMPLET sauté → company privée de 1000/2000/3000… Régression de données ciblant le scénario d'upgrade prod (historique stub #120). **4 affirmations vérifiées ground-truth** : `insert_stub_company` 0 compte + `20260528000001` précède `20260614000001` + backfill sans filtre `is_stub` + garde `count == 0`. **Fix** : `WHERE c.is_stub = FALSE AND …` sur les 2 INSERT du backfill (la stub recevra `1171`/`2206` via `bulk_create_from_chart` à l'onboarding, le chart JSON les incluant). Test `migration_backfill_skips_stub_company` ajouté (stub→0, onboardée→2). Copie inline du backfill dans le test idempotence alignée. Vert.
- Autres : MEDIUM backfill colonnes manquantes (réfuté — INSERT couvre les NOT NULL, MIGRATOR vert) ; archive FK dangling (INFO, pré-existant, déjà defer Pass 1) ; cross-check libellés chart JSON ↔ migration backfill sur 4 langues = ✅ identiques ; LOW drift copie SQL test (nit hérité). Tous ≤ LOW.
