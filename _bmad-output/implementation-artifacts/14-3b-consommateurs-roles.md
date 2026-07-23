# Story 14.3b : Consommateurs des rôles — garde de postabilité & lookups de facturation par rôle

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, association) ou fiduciaire,
**I want** que Kesh **empêche** de passer une écriture sur un compte non-postable (compte de regroupement ou compte de résultat calculé) et qu'il **résolve les comptes de facturation par leur rôle** plutôt que par un numéro codé en dur,
**So that** mon plan comptable reste réellement le mien — je peux renuméroter, et la comptabilité continue de viser les bons comptes sans jamais poster sur un compte de titre.

## Contexte

### Ce que 14-3a a livré (socle) et ce qui reste

La Story **14-3a (done)** a posé sur `accounts` les colonnes `role` (`AccountRole`, 10 rôles, 8 singleton) et `postable` (bool), leur unicité structurelle, l'API, la page Plan comptable, la réactivation (#269). **Aucun code métier ne lit encore ces colonnes** — c'était explicitement le socle. 14-3b câble les **consommateurs mécaniques** :

- **Chantier A — garde de postabilité** à la saisie d'écriture (backend `journal_entries.rs`) + filtre `postable` des sélecteurs de compte du frontend.
- **Chantier C — lookups de facturation par rôle** : remplacer les 6 `WHERE number = '1100'/'3000'/'2000'` de `company_invoice_settings.rs` par des lookups `WHERE role = ?` — le **dernier hardcode fonctionnel** par numéro du repo.

### Ce qui n'est PAS dans 14-3b

Le **chantier B** initialement prévu ici (présentation des fonds propres **par rôle** au bilan : `balance_sheet.rs` + CSV + PDF + `BalanceSheetView`) a été sorti dans la **Story 14-3c** (décision Guy, 2026-07-23). Motif : c'est de la **conception** (nouvelles structures, sections, mise en page, résolution de la collision de libellés léguée par 14-1), pas du rollout mécanique — le mélanger avec A/C diluerait le mental-model adversarial (règle de splitting préventif CLAUDE.md). 14-3b est donc **purement mécanique** : pas de migration, pas de nouveau modèle, pas de nouvelle colonne.

### Découpage (rappel)

`14-3` → `14-3a` (socle, done) + `14-3b` (A+C, cette story) + `14-3c` (B, backlog). Chaque sous-story de rollout est revue **file-by-file** plutôt qu'en passes adversariales globales (pattern « story-zero pose le pattern → rollout mécanique », CLAUDE.md).

## Décisions de conception (actées avant dev — à confirmer au validate)

### D-A1 — La garde de postabilité « grandfather » les comptes déjà référencés à l'UPDATE

**Le piège.** Un compte peut devenir non-postable **après** avoir porté des écritures : en 14-3a (patch D2), créer un sous-compte sur un compte-feuille mouvementé le fait passer `postable = FALSE` (il devient un compte de regroupement). Si la garde rejetait **toute** ligne visant un compte non-postable **aussi à l'update**, l'utilisateur ne pourrait **plus jamais éditer** une écriture historique passée sur ce compte (même pour en corriger la date) → régression bloquante en production.

**La règle.**
- **CREATE** : refuser toute ligne visant un compte non-postable. Un compte de regroupement / de résultat ne doit jamais recevoir de **nouvelle** écriture.
- **UPDATE** : n'exiger `postable = TRUE` que pour les comptes **nouvellement introduits** par rapport aux lignes **déjà persistées** de l'écriture. Un compte déjà présent dans l'écriture avant modification est **toléré** (grandfather), quel que soit son `postable` courant. On peut donc toujours re-sauvegarder une écriture existante ; on ne peut pas y **ajouter** une ligne vers un compte non-postable.

C'est le miroir exact de la logique déjà en place pour `active` — sauf qu'`active` n'a pas ce problème (une écriture sur un compte archivé est déjà un cas géré), tandis que `postable` peut basculer sous les pieds d'une écriture valide.

### D-A2 — Message d'erreur : réutiliser `InactiveOrInvalidAccounts` (pas de nouveau variant)

Un compte non-postable qui atteint la garde est **rejeté avec le variant existant `DbError::InactiveOrInvalidAccounts`** (400 `INACTIVE_OR_INVALID_ACCOUNTS`). Rationale : (1) le frontend filtre déjà les comptes non-postables des sélecteurs (chantier A frontend), donc ce 400 n'est atteignable qu'en appel API direct ou client périmé — cas de bord ; (2) distinguer « non-postable » de « inactif » exigerait d'**identifier le compte fautif** (le code actuel compare des cardinalités, il ne sait pas *lequel* manque), soit une complexité qui n'apporte rien au 99 % des cas couverts par le filtre frontend. Un compte non-postable **est** « invalide pour poster ». *(Si un besoin UX précis émerge, un variant dédié est une amélioration future — YAGNI ici.)*

### D-C1 — Mapping numéro → rôle des lookups de facturation

`1100` → `Receivable`, `3000` → `DefaultRevenue`, `2000` → `Payable`. Ces trois rôles sont **singleton** : au plus un compte actif par société les porte (`uq_accounts_company_singleton_role`), donc le `ORDER BY id LIMIT 1` devient sémantiquement redondant mais est **conservé** (lock déterministe, cohérence avec `FOR UPDATE`). `AND active = true` est **déjà présent** sur les 6 lookups — il est **conservé** (un rôle singleton n'est unique que parmi les comptes actifs, cf. invariant légué 14-3a).

### D-C2 — Frontend : quels sélecteurs filtrent `postable`

- **Sélecteurs de SAISIE (filtrent `postable === true`)** — 8 sites, cf. cartographie : le composant central `AccountAutocomplete.svelte` (couvre journal, assistant TVA, match manuel, split) + les `<select>` natifs des factures fournisseurs (saisie, import, règlement) + `RuleFormModal` (config de règle, mais le compte choisi sera **effectivement posté**).
- **Sélecteurs de CONFIGURATION dont le compte est ensuite posté (filtrent aussi `postable`, à confirmer au validate)** : `settings/invoicing` (comptes créance/produit/TVA — ils portent un rôle et sont postés à la génération d'écriture) et le compte lié d'un compte bancaire (`bank-accounts`, `BankAccountJournalLinkForm` — posté à la réconciliation). Filtrer `postable` y est protecteur (évite un `InactiveOrInvalidAccounts` en aval).
- **EXCLU explicitement** : le sélecteur de **compte parent** de la page Plan comptable (`accounts/+page.svelte`) — il veut au contraire des comptes de **regroupement** (souvent non-postables). Ne **pas** y ajouter `postable`.

## Acceptance Criteria

### A. Garde de postabilité à la saisie d'écriture (backend)

- **Given** une écriture en **création** (`journal_entries::create` / `create_in_tx`) dont une ligne vise un compte `postable = FALSE`, **When** je la soumets, **Then** elle est **rejetée** en `DbError::InactiveOrInvalidAccounts` (400) — la validation SQL des comptes (`journal_entries.rs:152-155`, aujourd'hui `WHERE company_id = ? AND active = TRUE AND id IN (…)`) gagne **`AND postable = TRUE`**.
- **Given** une écriture **existante** dont une ligne vise un compte devenu `postable = FALSE` **après** création, **When** je l'édite (`update` / `update_in_tx`) **sans changer cette ligne**, **Then** la sauvegarde **réussit** (grandfather D-A1) — la garde `postable` de l'update (`journal_entries.rs:702-705`) ne s'applique **qu'aux `account_id` absents des lignes déjà persistées** de l'écriture.
- **And** **Given** la même écriture existante, **When** je lui **ajoute** une ligne (ou change une ligne) vers un compte `postable = FALSE`, **Then** elle est **rejetée** en `InactiveOrInvalidAccounts`.
- **And** la logique de validation des comptes est **factorisée** entre create et update (elle est aujourd'hui dupliquée quasi à l'identique, `:146-168` et `:696-719`). Le helper prend en compte la sémantique de rollback différente (create délègue le rollback au caller, update le fait lui-même — le helper **ne rollback pas**, il retourne `Err`, le caller décide) et l'ensemble des `account_id` à **exempter** (les comptes déjà présents, vide en création).
- **And** les comptes **non-postables restent parfaitement listables et lisibles** — la garde ne concerne QUE la saisie d'une ligne d'écriture, jamais la lecture, le reporting, ou l'archivage.

### B. Garde de postabilité — filtres frontend

- **Given** un sélecteur de compte de **saisie**, **When** je l'ouvre, **Then** seuls les comptes `active === true && postable === true` sont proposés. Sites couverts :
  - `AccountAutocomplete.svelte` (filtre `:32` `accounts.filter((a) => a.active)` → `+ && a.postable`) — couvre écriture au journal, assistant TVA (charge + contrepartie), match manuel, split de transaction ;
  - `supplier-invoices/+page.svelte` (filtre `:128`), `supplier-invoices/import/+page.svelte` (`:102`), `supplier-invoices/[id]/+page.svelte` (`:66`) — `<select>` natifs, ajouter `&& a.postable` ;
  - `RuleFormModal.svelte` (filtre `:42`) — ajouter `&& a.postable`.
- **And** les sélecteurs de **configuration dont le compte est posté** filtrent aussi `postable` (D-C2) : `settings/invoicing/+page.svelte` (dérivés `:47-55`), `bank-accounts/+page.svelte` (`:78`) et `BankAccountJournalLinkForm.svelte` (`:44`).
- **And** le sélecteur de **compte parent** (`accounts/+page.svelte:146`) n'est **PAS** filtré `postable` (il veut des regroupements).
- **And** aucune régression de l'exclusion `active` existante ni des filtres métier déjà en place (classes 5/6/7 en réconciliation, `accountType === 'Expense'` en factures fournisseurs, etc.) — le `postable` s'**ajoute**, il ne remplace pas.

### C. Lookups de facturation par rôle

- **Given** `company_invoice_settings::insert_with_defaults` et son miroir `insert_with_defaults_in_tx`, **When** ils résolvent les comptes par défaut, **Then** les 6 lookups `WHERE number = '1100'/'3000'/'2000'` (`:275/:284/:296` + `:396/:405/:416`) deviennent `WHERE role = ?` avec `.bind(AccountRole::Receivable/DefaultRevenue/Payable)`, **en conservant `AND active = true` et `FOR UPDATE`**.
- **And** le fail-fast d'onboarding (`if receivable.is_none() || revenue.is_none()` → `InactiveOrInvalidAccounts`, `:308-311` et `:427-430`) est **inchangé dans sa logique** : un plan sans compte portant le rôle `Receivable` **ou** `DefaultRevenue` actif fait échouer l'initialisation de la config de facturation. `Payable` reste optionnel (pas de fail-fast).
- **And** les **deux** fonctions miroir sont modifiées **symétriquement** (blocs `MIRROR`, dupliqués volontairement pour contourner la fragilité HRTB de SQLx 0.8 — ne pas « dé-dupliquer » au passage).
- **And** le re-check FK au `rows == 0` (JOIN par `id` sur `active = TRUE`, `:338-367` / `:452-475`) n'est **pas** impacté (il opère par id résolu, pas par numéro) — le relire pour confirmer, ne pas y toucher.
- **And** la **docstring** de `insert_with_defaults` (`:249-260`, qui documente encore « 1100: Receivables », « lookups with string literals '1100'/'3000' are safe ») est **mise à jour** pour refléter la résolution par rôle — sinon elle ment.
- **And** les **consommateurs du variant `InactiveOrInvalidAccounts`** ne sont pas cassés : la retry-loop de `kesh-seed/src/lib.rs` (~`:91`, `:189`) et le handler `onboarding.rs:720` matchent sur ce variant — il reste le variant de fail-fast, inchangé.

### D. Comportement métier préservé (non-régression)

- **And** un plan comptable **standard** (seedé par Kesh, rôles posés par le backfill 14-3a) : l'onboarding et la génération de facture continuent de viser les comptes 1100/3000/2000 exactement comme avant — parce qu'ils portent désormais les rôles correspondants. **Aucune différence observable** pour un utilisateur qui n'a pas renuméroté.
- **And** un plan **renuméroté** (ex. débiteurs en 1101 portant le rôle `Receivable`) : la facturation vise désormais le **bon** compte — c'est le gain de la story, à prouver par un test.
- **And** aucune écriture existante ne devient inéditable (grandfather D-A1, à prouver par un test).

### E. Tests

- **Backend `journal_entries` (repo)** :
  - create : ligne vers compte non-postable → `InactiveOrInvalidAccounts` ; ligne vers compte postable → OK.
  - update grandfather : écriture sur compte X, X devient non-postable (créer un enfant), éditer l'écriture sans changer la ligne → **OK** ; ajouter une ligne vers un compte non-postable → **rejet**.
  - le compte de résultat (`CurrentYearResult`, non-postable) est refusé à la saisie.
- **Backend `company_invoice_settings`** :
  - plan standard : `insert_with_defaults` résout les mêmes ids qu'avant (par rôle).
  - plan renuméroté (débiteurs = `Receivable` sur un numéro ≠ 1100) : la résolution trouve le bon compte.
  - plan sans compte `Receivable` actif : fail-fast `InactiveOrInvalidAccounts` (miroir des deux fonctions).
  - la retry-loop de seed et le finalize onboarding restent verts (tests E2E existants `onboarding_e2e` / seed).
- **Frontend** : Vitest sur `AccountAutocomplete` — un compte `postable === false` n'apparaît pas dans les options ; un compte `active && postable` apparaît. (Les `<select>` natifs et la page sont couverts par Playwright si praticable, sinon documenté comme testé via le filtre unitaire.)
- **E2E Playwright** (si praticable sur le harnais local) : dans un sélecteur de saisie d'écriture, un compte non-postable n'est pas sélectionnable.

## Tasks / Subtasks

- [ ] **T1 — Garde postabilité backend, factorisée (AC-A)**
  - [ ] Extraire un helper de validation des comptes dans `journal_entries.rs` (ex. `validate_lines_accounts_in_tx(tx, company_id, account_ids, exempt_ids) -> Result<(), DbError>`), sans rollback interne (le caller décide).
  - [ ] `create_in_tx` : appeler le helper avec `exempt_ids = []` et `AND postable = TRUE`.
  - [ ] `update_in_tx` : calculer `exempt_ids` = `account_id` des lignes **déjà persistées** de l'écriture (récupérées avant remplacement), appeler le helper, conserver le rollback pré-erreur du caller.
- [ ] **T2 — Filtres postable frontend (AC-B)**
  - [ ] `AccountAutocomplete.svelte:32` → `.filter((a) => a.active && a.postable)`.
  - [ ] `<select>` natifs saisie : `supplier-invoices/+page.svelte:128`, `import/+page.svelte:102`, `[id]/+page.svelte:66`, `RuleFormModal.svelte:42` → `&& a.postable`.
  - [ ] Config postés : `settings/invoicing/+page.svelte:47-55`, `bank-accounts/+page.svelte:78`, `BankAccountJournalLinkForm.svelte:44` → `&& a.postable`.
  - [ ] **Ne pas toucher** `accounts/+page.svelte:146` (sélecteur parent).
- [ ] **T3 — Lookups facturation par rôle (AC-C)**
  - [ ] `insert_with_defaults` : 3 lookups `number` → `role = ?` (Receivable/DefaultRevenue/Payable), `active`/`FOR UPDATE`/`.flatten()` conservés.
  - [ ] `insert_with_defaults_in_tx` : idem, symétrique (bloc MIRROR).
  - [ ] Mettre à jour la docstring `:249-260`.
- [ ] **T4 — Tests (AC-E)** : repo `journal_entries` (create/update grandfather/résultat), repo `company_invoice_settings` (standard/renuméroté/fail-fast), Vitest `AccountAutocomplete`, E2E si praticable.
- [ ] **T5 — Doc & gate** : CHANGELOG (garde de postabilité effective + facturation par rôle) ; manuel utilisateur si un comportement visible change (le badge « Non postable » de 14-3a devient **bloquant** — mettre à jour la mention « indicatif » de 14-3a) ; gate complet (`scripts/test-fast.sh` ou la série CLAUDE.md) ; README feuille de route si nécessaire.

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-23)

**Chantier A — `crates/kesh-db/src/repositories/journal_entries.rs`** (0 occurrence de `postable` aujourd'hui) :
- create : SQL `:152-155` (`WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})`), erreur `:167` `InactiveOrInvalidAccounts`. Bloc `:146-168`. Le rollback est **délégué au caller** (pas de `tx.rollback()` avant le `return Err`).
- update : SQL `:702-705`, erreur `:718`. Bloc `:696-719`. Ici le `return Err` est **précédé** de `tx.rollback().await.map_err(map_db_error)?;`.
- Différences entre les deux blocs : source des lignes (`new.lines` vs `updated.lines`), binder company (`new.company_id` vs `company_id`), déréférencement (`&mut **tx` vs `&mut *tx`), rollback. Le helper doit rester **rollback-agnostique**.
- **Patron SQL de lookup par rôle** déjà dans le repo : `accounts.rs` `find_singleton_role_holder` (`WHERE company_id = ? AND role = ? AND active = TRUE`, `.bind(role)`), et l'enum `AccountRole` implémente `Encode<MySql>` donc `.bind(AccountRole::X)` fonctionne directement.

**Chantier C — `crates/kesh-db/src/repositories/company_invoice_settings.rs`** :
- `insert_with_defaults` (`:264-379`, pool-level) : lookups `:275` (1100), `:284` (3000), `:296` (2000, optionnel) ; fail-fast `:308-311` (avec rollback). Appelée par `kesh-seed/src/lib.rs:189` (retry-loop matchant `InactiveOrInvalidAccounts`).
- `insert_with_defaults_in_tx` (`:387-486`, tx-level, MIROIR) : lookups `:396`/`:405`/`:416` ; fail-fast `:427-430` (sans rollback, le caller possède la tx). Appelée par `onboarding.rs:720`.
- `AND active = true` **déjà présent** partout ; `FOR UPDATE` présent. Re-check FK `rows == 0` par id : `:338-367` / `:452-475` — ne pas toucher.

**Mapping d'erreur** : `kesh-db/errors.rs` variant `InactiveOrInvalidAccounts` (`:48-52`, code `:168`) → `kesh-api/errors.rs:2140-2147` = **400** `INACTIVE_OR_INVALID_ACCOUNTS`, message i18n `error-inactive-accounts`. Réutilisé, pas modifié (D-A2).

**Frontend** — cf. tableau de la section D-C2 / AC-B. Le composant central `AccountAutocomplete.svelte:32` couvre à lui seul 4 sites de saisie ; les autres sont des `<select>` natifs avec leur propre `.filter`.

### Invariant légué de 14-3a (à ne pas perdre)

Un rôle singleton n'est unique **que parmi les comptes actifs** (`archive()` ne remet pas `role` à `NULL`, la colonne générée est `active`-aware). **Tout lookup par rôle DOIT porter `AND active = TRUE`** — sinon un `WHERE role = ?` nu peut ramener deux lignes (une archivée, une active) après un cycle archive → reprise. Les 6 lookups de facturation l'ont déjà ; ne pas l'enlever en passant au rôle.

### Pièges, par ordre de coût

1. **Grandfather de l'update (D-A1)** — le rater rend inéditables les écritures historiques sur un compte devenu non-postable. C'est le risque n°1, purement runtime (invisible en validate statique). Le test « X devient non-postable après une écriture, l'écriture reste éditable » est le garde-fou.
2. **Symétrie des deux fonctions miroir de `company_invoice_settings`** — modifier une seule fait diverger onboarding et seed. Les deux triplets de lookups changent ensemble.
3. **Rollback dans le helper factorisé** — create délègue, update rollback. Un helper qui rollback lui-même casserait l'un des deux chemins. Le helper retourne `Err`, ne rollback pas.
4. **Ne pas dé-dupliquer les blocs MIRROR** de `company_invoice_settings` (dette assumée, contrainte HRTB SQLx 0.8 documentée sur place). Facteur A (`journal_entries`), oui ; facteur C (miroir invoice-settings), non.
5. **`postable` sur le sélecteur parent** — l'y ajouter serait l'inverse du besoin (on veut des regroupements). Explicitement exclu.

### Hors scope (garde-fous)

- ❌ Présentation des fonds propres **par rôle** au bilan (`balance_sheet.rs`/CSV/PDF/`BalanceSheetView`) → **14-3c**.
- ❌ Toute **migration** ou nouvelle colonne — 14-3b ne touche pas le schéma.
- ❌ Nouveau variant d'erreur pour « non-postable » (D-A2 : réutilise l'existant).
- ❌ Dé-duplication des fonctions miroir de `company_invoice_settings` (dette assumée).
- ❌ Rôles de trésorerie caisse/banque (écartés en 14-3a).

### Limitations documentées (catégorie B)

- **L1 (héritée 14-3a)** — rôles singleton mono-valués : un seul compte actif par société pour `Receivable`/`DefaultRevenue`/`Payable`. Aligné sur `company_invoice_settings` (un seul `default_*_account_id`). Lever = passer la config en listes, hors v0.1.

### Doc-sync (T5)

- **CHANGELOG** : la postabilité, **indicative** en 14-3a (badge « Non postable »), devient **bloquante** à la saisie ; la facturation résout ses comptes par rôle (le plan peut être renuméroté).
- **Manuel utilisateur** : corriger la mention « indicatif — la saisie ne le bloque pas encore » posée en 14-3a → désormais bloquant.
- **README** feuille de route : marquer l'avancement 14-3 si pertinent.

### References

- Story socle : [Source: _bmad-output/implementation-artifacts/14-3a-socle-roles-comptes.md] (enum, invariant `active`, note léguée §138)
- Garde active existante : [Source: crates/kesh-db/src/repositories/journal_entries.rs#152-168, #696-719]
- Lookups par numéro : [Source: crates/kesh-db/src/repositories/company_invoice_settings.rs#264-486]
- Patron lookup par rôle : [Source: crates/kesh-db/src/repositories/accounts.rs#find_singleton_role_holder]
- Enum `AccountRole` : [Source: crates/kesh-db/src/entities/account.rs#89-181]
- Sélecteurs frontend : [Source: cartographie 2 agents Explore 2026-07-23 — tableau récapitulatif AC-B]
- Conventions : [Source: CLAUDE.md § Test Locally First, § Review Iteration Rule, § Règle de commit]

## Dev Agent Record

### Agent Model Used

(à remplir par dev-story)

### Debug Log References

### Completion Notes List

### File List
