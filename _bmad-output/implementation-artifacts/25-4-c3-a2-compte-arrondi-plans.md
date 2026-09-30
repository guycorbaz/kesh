# Story 25.4-c3-a2 : Le compte de différences d'arrondi dans les plans livrés

Status: ready-for-dev

**Issue : [#476]** — ⛔ la PR porte `refs #476` (la **25-4-c3-b** la fermera).

**Née du découpage de la 25-4-c3.** Arbitrages de Guy (2026-09-30) : le compte se désigne par un
**réglage** (livré par la **25-4-c3-a1**, PR #487) ; les plans livrés proposent **« 6940 Différences
d'arrondi »**, un compte de **charge**, que l'onboarding **désigne d'office** pour une société neuve ; une
société existante le crée dans le plan et le choisit dans les paramètres (rien n'est écrit chez elle).

**Empilée sur la 25-4-c3-a1** (PR #487, non mergée) : branche `story/25-4-c3-a2-compte-arrondi-plans`.

## Story

En tant que personne qui crée sa société dans Kesh,
je veux que le plan comptable proposé contienne déjà un compte de différences d'arrondi, désigné dans les
paramètres,
afin de ne rien avoir à configurer pour que les écarts d'un demi-centime soient passés en écriture.

## Ce qui existe, vérifié dans le code

- **Les plans** : `crates/kesh-core/assets/charts/{pme,independant,association}.json` (84, 84, 81
  entrées ; tableau JSON), noms en `fr`, `de`, `it`, `en`. Le groupe `6` (« Autres charges
  d'exploitation », `Expense`, `pme.json:62`) porte `6900` (charges financières, `Expense`, `:75`) et
  `6950` (produits financiers, `Revenue`, `:76`) ; **aucun `6940`** dans les trois plans.
- **`ChartEntry`** (`crates/kesh-core/src/chart_of_accounts/mod.rs:187-206`) : `number`, `name`, `type`,
  `parentNumber`, `role` (facultatif), `postable` (facultatif, Story 24-5 — le patron d'un attribut de plan
  ajouté sans casser les plans existants : `#[serde(default)]`). `load_chart` (`:231`) appelle
  `validate_chart` (`:248`, doublons, parents, singletons de rôle, rôle ↔ type).
- **La création des comptes** : `accounts::bulk_create_from_chart`
  (`crates/kesh-db/src/repositories/accounts.rs:935`), appelée à l'**étape 4** de l'onboarding
  (`crates/kesh-api/src/routes/onboarding.rs:379-388`, garde : seulement si la société n'a aucun compte)
  et par le jeu de démonstration (`crates/kesh-seed/src/lib.rs:148-159`).
- **Le remplissage des réglages** : `company_invoice_settings::insert_with_defaults` (pool, appelé par le
  seed, `kesh-seed/src/lib.rs:194`) et son miroir `insert_with_defaults_in_tx` (appelé à la
  **finalisation** de l'onboarding, `onboarding.rs:721`, dans une **autre requête** que l'étape 4 : le plan
  n'est plus en mémoire). Ils posent créance, produit et créanciers **par rôle** ; le compte d'arrondi, lui,
  n'a pas de rôle (arbitrage) et reste `NULL` depuis la c3-a1.
- ⛔ **Aucun numéro de compte dans le code applicatif** (`14-3a`, migration `20260722000001:3-8`) : le
  numéro `6940` vit dans les **données** du plan, jamais dans le Rust.

## Acceptance Criteria

**AC 1 — Le compte dans les trois plans.** Une entrée `6940`, type `Expense`, parent `6`, noms : fr
« Différences d'arrondi », de « Rundungsdifferenzen », it « Differenze di arrotondamento », en « Rounding
differences ». Imputable (aucun `postable: false`).

**AC 2 — Le marqueur de plan.** Un attribut facultatif de `ChartEntry` — par exemple
`"roundingDifference": true` — désigne **l'**entrée qui sera le compte de différences d'arrondi.
`#[serde(default)]` : un plan sans l'attribut reste valide (patron de `postable`, 24-5). `validate_chart`
refuse : **plus d'une** entrée marquée ; une entrée marquée d'un autre type que `Expense` ou `Revenue` ;
une entrée marquée **non imputable** (au sens de `is_postable`). Une fonction de `kesh-core` rend le numéro
de l'entrée marquée d'un plan (`Option`), pour que personne ne recopie la recherche.

**AC 3 — La désignation à la création.** `insert_with_defaults` **et** `insert_with_defaults_in_tx`
(miroirs, modifiés ensemble) posent `default_rounding_account_id` sur le compte de la société dont le numéro
est celui de l'entrée marquée du plan de sa forme juridique (`companies.org_type` → `load_chart`), **s'il
existe, est actif, imputable, de charge ou de produit** — sinon ils le laissent `NULL`, **sans échouer** :
le compte d'arrondi est facultatif (contrairement à la créance et au produit, qui font échouer
l'onboarding). ⚠️ Le compte a pu être renuméroté ou supprimé entre l'étape 4 et la finalisation : `NULL`,
jamais une erreur. Aucune ligne existante n'est modifiée : `INSERT IGNORE` reste la règle (une société qui
a déjà ses réglages n'est pas touchée).

**AC 4 — Les sociétés existantes.** **Aucune migration, aucune donnée écrite** chez elles (arbitrage) :
elles créent le compte dans le plan et le choisissent dans les paramètres (c3-a1).

**AC 5 — Tests.**
- `kesh-core` : chacun des trois plans porte **exactement une** entrée marquée, `6940`, `Expense`, quatre
  langues ; `validate_chart` refuse deux marqueurs, un marqueur sur un actif, un marqueur non imputable ; un
  plan sans marqueur se charge ;
- `kesh-db` : après `bulk_create_from_chart` + `insert_with_defaults`, le réglage pointe sur le `6940` de
  la société, pour **chaque** plan ; renuméroté ou archivé avant l'appel → `NULL`, sans erreur ; une société
  dont les réglages existent déjà n'est pas modifiée ;
- `kesh-api` : l'onboarding de bout en bout (chemin « production », étape 4 puis finalisation) aboutit à un
  réglage désigné ;
- les tests qui comptent les entrées des plans ou comparent le plan semé à un état attendu
  (`accounts_role_backfill.rs`, `closing_accounts_backfill.rs`, `chart_of_accounts/mod.rs`, seed) sont
  relus et mis à jour **sans affaiblir leur assertion**.

**AC 6 — Textes.** `admin-manual.tex`, paragraphe *Compte de différences d'arrondi* (c3-a1) : les plans
livrés proposent `6940 Différences d'arrondi`, désigné d'office à la création d'une société ; une société
existante le crée et le choisit. La phrase « n'est encore lu par aucune écriture » **reste** (la c3-b la
retirera). PDF régénéré, contrôlé aplati. CHANGELOG `[0.12.1]` : compléter l'entrée *Added* de la c3-a1.
Manuel utilisateur : le relire là où il décrit le contenu des plans livrés — ne rien écrire s'il n'en
dresse pas la liste.

## Tasks / Subtasks

- [ ] **T1 — plans et marqueur** (AC 1, 2) : trois JSON, `ChartEntry`, `validate_chart`, fonction de numéro.
- [ ] **T2 — désignation** (AC 3, 4) : les deux `insert_with_defaults*`.
- [ ] **T3 — tests** (AC 5).
- [ ] **T4 — textes** (AC 6).
- [ ] **T5 — gates** : backend complet (base remise à zéro ; ⛔ dépôt `kesh-db` touché ⇒ gate complet même
  en cours de boucle), frontend (inchangé en principe : `check` et `test:unit`), **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Un rôle de compte** (arbitrage) ; ⛔ **le numéro `6940` dans le Rust** : il est dans les données du
  plan ; le code cherche l'entrée **marquée**.
- ⛔ **Faire échouer l'onboarding** quand le compte d'arrondi manque : il est facultatif.
- ⛔ **Une migration qui crée le compte chez les sociétés existantes** (arbitrage).
- ⚠️ **Les deux `insert_with_defaults*` sont miroirs** (commentaire `MIRROR`) : les modifier ensemble, et
  factoriser la recherche du compte d'arrondi plutôt que la recopier (règle DRY).
- ⚠️ Le jeu E2E `with-company` (`POST /_test/seed`) n'utilise **pas** les plans livrés (3000 et 4000
  seulement) : il n'est pas concerné, ne pas l'aligner.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-core/assets/charts/{pme,independant,association}.json` | les plans |
| `crates/kesh-core/src/chart_of_accounts/mod.rs:187-300` | `ChartEntry`, `load_chart`, `validate_chart`, `is_postable` |
| `crates/kesh-db/src/repositories/company_invoice_settings.rs:277-500` | les deux `insert_with_defaults*` |
| `crates/kesh-api/src/routes/onboarding.rs:370-390, 680-740` | étape 4 et finalisation |
| `crates/kesh-seed/src/lib.rs:140-200` | le jeu de démonstration |
| `crates/kesh-db/tests/company_invoice_settings_repository.rs` | tests des `insert_with_defaults*` |
| `docs/manual/fr/admin-manual.tex` (*Compte de différences d'arrondi*) | manuel |

### Gardes-fous du dépôt

- Modules : `kesh-core`, `kesh-db`, `kesh-api` (tests), `kesh-seed` (si touché), `docs` — sous le seuil.
- Aucune migration.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-30** — Créée au découpage de la 25-4-c3, selon les arbitrages de Guy : `6940 Différences
  d'arrondi` (charge) dans les trois plans, désigné d'office à la création d'une société par un **marqueur de
  plan** (pas de numéro dans le code, pas de rôle) ; rien chez les sociétés existantes.

[#476]: https://github.com/guycorbaz/kesh/issues/476
