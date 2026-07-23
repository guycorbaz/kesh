# Story 14.3b : Consommateurs des rôles — garde de postabilité & lookups de facturation par rôle

## Status

done

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

## Décisions de conception (tranchées par Guy 2026-07-23, avant dev)

### D-A0 — La garde ne s'applique qu'à la SAISIE MANUELLE (flux automatiques exemptés)

**Le piège (finding CRITICAL validate P1).** `journal_entries::create_in_tx` est le point d'entrée **partagé** de TOUTES les écritures : la saisie manuelle (via `create` pool-level, appelée par le seul handler `routes/journal_entries.rs:494`) **et** les flux automatiques — validation de facture de vente (`invoices.rs:1390`), avoir (`credit_notes.rs:331`), facture fournisseur + paiement (`supplier_invoices.rs:359/648/784`), réconciliation manuelle/split/règle (`reconciliation.rs` ×5). Ces flux automatiques postent sur des comptes **résolus depuis la config** (`company_invoice_settings.default_receivable_account_id` etc., FK figée cachée). Si l'un de ces comptes devenait non-postable — via une action utilisateur **légitime et déjà possible en prod** (ajouter un sous-compte analytique sous 1100 → 14-3a bascule le parent `postable=FALSE`) — une garde `postable` posée dans `create_in_tx` **casserait toute validation de facture/avoir** en 400 opaque. Régression du cœur comptable.

**La règle.** La garde de postabilité est une contrainte de **saisie manuelle**, pas de flux système. `create_in_tx` reçoit un paramètre explicite `enforce_postable: bool` (ou un enum `PostingSource::Manual | System`) :
- **Saisie manuelle** (`create` pool-level → `create_in_tx(.., enforce_postable=true)`, et `update`) : garde active.
- **Flux automatiques** (les ~7 appelants directs de `create_in_tx` listés ci-dessus : invoices, credit_notes, supplier_invoices ×3, reconciliation ×5) : `enforce_postable=false` — ils postent sur des comptes de config approuvés, on leur fait confiance.

Corollaire : les sélecteurs de config de facturation filtrent déjà `postable` (D-C2), donc l'utilisateur ne configure normalement pas un compte de rôle non-postable. Un compte de config devenu non-postable *après coup* reste posté par le flux automatique (posting sur un compte parent n'est pas corrupteur, juste inhabituel) — **brèche mineure assumée**, tracée en Limitations (L2).

### D-A1 — La garde manuelle « grandfather » PAR COMPTE les comptes déjà référencés à l'UPDATE

**Le piège.** Un compte peut devenir non-postable **après** avoir porté des écritures manuelles (14-3a : créer un sous-compte sur un compte-feuille mouvementé le passe `postable=FALSE`). Si la garde manuelle rejetait **toute** ligne visant un compte non-postable **aussi à l'update**, l'utilisateur ne pourrait **plus jamais éditer** une écriture historique passée sur ce compte (même pour en corriger la date) → régression bloquante.

**La règle (grandfather PAR COMPTE — décision Guy).**
- **CREATE manuel** : refuser toute ligne visant un compte non-postable. Un compte de regroupement / de résultat ne reçoit jamais de **nouvelle** écriture manuelle.
- **UPDATE manuel** : n'exiger `postable = TRUE` que pour les `account_id` **absents de l'ensemble des comptes déjà référencés** par les lignes persistées de l'écriture (avant remplacement). Un compte déjà présent dans l'écriture est **exempté** (`postable = TRUE OR id IN (exempt_ids)`), quel que soit son `postable` courant.

**Contrainte technique acceptée (finding HIGH validate P1) : l'exemption est PAR COMPTE, pas par ligne.** `update` fait `DELETE FROM journal_entry_lines` puis ré-INSERT complet (`journal_entries.rs:759-763`) — il n'existe **aucune identité de ligne stable** avant/après. On ne peut donc distinguer « re-sauvegarde d'une ligne existante sur X » de « ajout d'une **nouvelle** ligne sur X déjà référencé ailleurs ». Conséquence : on peut, via une édition, **ajouter** des lignes sur un compte non-postable déjà présent dans l'écriture. **Brèche mineure assumée** (le compte est visible dans l'UI, l'action d'édition est explicite, le compte n'est de toute façon jamais un compte système comme le résultat calculé) — tracée en Limitations (L3). La fermer (comparaison multiset `(account_id, debit, credit)` avant/après) a été jugée disproportionnée (décision Guy, cohérent §simplicité CLAUDE.md).

### D-A2 — Message d'erreur : réutiliser `InactiveOrInvalidAccounts` (pas de nouveau variant)

Un compte non-postable qui atteint la garde est **rejeté avec le variant existant `DbError::InactiveOrInvalidAccounts`** (400 `INACTIVE_OR_INVALID_ACCOUNTS`). Rationale : (1) le frontend filtre déjà les comptes non-postables des sélecteurs (chantier A frontend), donc ce 400 n'est atteignable qu'en appel API direct ou client périmé — cas de bord ; (2) distinguer « non-postable » de « inactif » exigerait d'**identifier le compte fautif** (le code actuel compare des cardinalités, il ne sait pas *lequel* manque), soit une complexité qui n'apporte rien au 99 % des cas couverts par le filtre frontend. Un compte non-postable **est** « invalide pour poster ». *(Si un besoin UX précis émerge, un variant dédié est une amélioration future — YAGNI ici.)*

### D-C1 — Mapping numéro → rôle des lookups de facturation

`1100` → `Receivable`, `3000` → `DefaultRevenue`, `2000` → `Payable`. Ces trois rôles sont **singleton** : au plus un compte actif par société les porte (`uq_accounts_company_singleton_role`), donc le `ORDER BY id LIMIT 1` devient sémantiquement redondant mais est **conservé** (lock déterministe, cohérence avec `FOR UPDATE`). `AND active = true` est **déjà présent** sur les 6 lookups — il est **conservé** (un rôle singleton n'est unique que parmi les comptes actifs, cf. invariant légué 14-3a).

### D-C2 — Frontend : quels sélecteurs filtrent `postable`

- **Sélecteurs de SAISIE (filtrent `postable === true`)** — 8 sites, cf. cartographie : le composant central `AccountAutocomplete.svelte` (couvre journal, assistant TVA, match manuel, split) + les `<select>` natifs des factures fournisseurs (saisie, import, règlement) + `RuleFormModal` (config de règle, mais le compte choisi sera **effectivement posté**).
- **Sélecteurs de CONFIGURATION dont le compte est ensuite posté (filtrent aussi `postable`, à confirmer au validate)** : `settings/invoicing` (comptes créance/produit/TVA — ils portent un rôle et sont postés à la génération d'écriture) et le compte lié d'un compte bancaire (`bank-accounts`, `BankAccountJournalLinkForm` — posté à la réconciliation). Filtrer `postable` y est protecteur (évite un `InactiveOrInvalidAccounts` en aval).
- **EXCLU explicitement** : le sélecteur de **compte parent** de la page Plan comptable (`accounts/+page.svelte`) — il veut au contraire des comptes de **regroupement** (souvent non-postables). Ne **pas** y ajouter `postable`.

## Acceptance Criteria

### A. Garde de postabilité à la saisie d'écriture MANUELLE (backend)

- **Given** une écriture **manuelle** en création (handler `routes/journal_entries.rs:494` → `journal_entries::create` → `create_in_tx(.., enforce_postable=true)`) dont une ligne vise un compte `postable = FALSE`, **When** je la soumets, **Then** elle est **rejetée** en `DbError::InactiveOrInvalidAccounts` (400) — la validation SQL des comptes (`journal_entries.rs:152-155`, `WHERE company_id = ? AND active = TRUE AND id IN (…)`) gagne **`AND postable = TRUE`**, conditionnée par `enforce_postable`.
- **Given** un flux **automatique** (validation de facture, avoir, facture fournisseur, réconciliation — appelants directs de `create_in_tx`), **When** il poste une écriture, **Then** la garde `postable` **ne s'applique PAS** (`enforce_postable=false`) : ces flux postent sur des comptes de config approuvés (D-A0). **Aucune régression** des flux automatiques — à prouver par un test (une facture dont le compte produit est non-postable se valide quand même).
- **Given** une écriture manuelle **existante** dont une ligne vise un compte devenu `postable = FALSE` **après** création, **When** je l'édite (`update`) **sans toucher à ce compte**, **Then** la sauvegarde **réussit** (grandfather par compte, D-A1) — la garde ne s'applique **qu'aux `account_id` absents de l'ensemble des comptes déjà référencés** par les lignes persistées (`postable = TRUE OR id IN (exempt_ids)`).
- **And** **Given** la même écriture existante, **When** je lui ajoute une ligne vers un compte non-postable **jamais référencé** dans cette écriture, **Then** elle est **rejetée**. *(Ajouter une ligne vers un compte non-postable **déjà référencé** dans l'écriture est en revanche toléré — brèche par-compte assumée, L3.)*
- **And** la validation des comptes est **factorisée** entre `create_in_tx` et `update` (aujourd'hui dupliquée, `:146-168` et `:696-719`). Le helper est **rollback-agnostique** (il retourne `Err`, ne rollback pas ; `create_in_tx` délègue le rollback au caller, `update` rollback lui-même autour de l'appel) et prend `enforce_postable: bool` + `exempt_ids: &[i64]` (vide en création, comptes déjà référencés en update).
- **And** les comptes **non-postables restent parfaitement listables et lisibles** — la garde ne concerne QUE la saisie manuelle d'une ligne, jamais la lecture, le reporting, l'archivage, ni les flux automatiques.

### B. Garde de postabilité — filtres frontend

- **Given** un sélecteur de compte de **saisie**, **When** je l'ouvre, **Then** seuls les comptes `active === true && postable === true` sont proposés. Sites couverts :
  - `AccountAutocomplete.svelte` (filtre `:32` `accounts.filter((a) => a.active)` → `+ && a.postable`) — couvre écriture au journal, assistant TVA (charge + contrepartie), match manuel, split de transaction ;
  - `supplier-invoices/+page.svelte` (filtre `:128`), `supplier-invoices/import/+page.svelte` (`:102`), `supplier-invoices/[id]/+page.svelte` (`:66`) — `<select>` natifs, ajouter `&& a.postable` ;
  - `RuleFormModal.svelte` (le `$derived eligibleAccounts` est déclaré `:42`, le `.filter(` réel est `:43-47`) — ajouter `&& a.postable`.
- **And** les sélecteurs de **configuration dont le compte est posté** filtrent aussi `postable` (D-C2) : `settings/invoicing/+page.svelte` (dérivés `:47-55`), `bank-accounts/+page.svelte` (`:78`) et `BankAccountJournalLinkForm.svelte` (`:44`).
- **And** le sélecteur de **compte parent** (`accounts/+page.svelte:146`) n'est **PAS** filtré `postable` (il veut des regroupements).
- **And** aucune régression de l'exclusion `active` existante ni des filtres métier déjà en place (classes 5/6/7 en réconciliation, `accountType === 'Expense'` en factures fournisseurs, etc.) — le `postable` s'**ajoute**, il ne remplace pas.

### C. Lookups de facturation par rôle

- **Given** `company_invoice_settings::insert_with_defaults` et son miroir `insert_with_defaults_in_tx`, **When** ils résolvent les comptes par défaut, **Then** les 6 lookups `WHERE number = '1100'/'3000'/'2000'` (`:275/:284/:296` + `:396/:405/:416`) deviennent `WHERE role = ?` avec `.bind(AccountRole::Receivable/DefaultRevenue/Payable)`, **en conservant `AND active = true` et `FOR UPDATE`**.
- **And** le fail-fast d'onboarding (`if receivable.is_none() || revenue.is_none()` → `InactiveOrInvalidAccounts`, `:308-311` et `:427-430`) est **inchangé dans sa logique** : un plan sans compte portant le rôle `Receivable` **ou** `DefaultRevenue` actif fait échouer l'initialisation de la config de facturation. `Payable` reste optionnel (pas de fail-fast).
- **And** ce fail-fast n'est atteignable qu'à l'**onboarding** (les 2 seuls appelants créent une société) et les 3 charts livrés portent toujours ces rôles → probabilité réelle **faible**. Le message générique `InactiveOrInvalidAccounts` (ne nommant pas le rôle manquant) est **accepté tel quel pour v0.1** (limitation L4), plutôt que d'ajouter un message dédié « configurez un compte de rôle Créances clients » que 14-3a rend théoriquement atteignable (un utilisateur peut désormais retirer le rôle via la page Plan comptable). *(Améliorer le message = amélioration future si le cas se manifeste — YAGNI.)*
- **And** les **deux** fonctions miroir sont modifiées **symétriquement** (blocs `MIRROR`, dupliqués volontairement pour contourner la fragilité HRTB de SQLx 0.8 — ne pas « dé-dupliquer » au passage).
- **And** le re-check FK au `rows == 0` (JOIN par `id` sur `active = TRUE`, `:338-367` / `:452-475`) n'est **pas** impacté (il opère par id résolu, pas par numéro) — le relire pour confirmer, ne pas y toucher.
- **And** la **docstring** de `insert_with_defaults` (`:249-260`, qui documente encore « 1100: Receivables », « lookups with string literals '1100'/'3000' are safe ») est **mise à jour** pour refléter la résolution par rôle — sinon elle ment.
- **And** les **consommateurs du variant `InactiveOrInvalidAccounts`** ne sont pas cassés : la retry-loop de `kesh-seed/src/lib.rs` (~`:91`, `:189`) et le handler `onboarding.rs:720` matchent sur ce variant — il reste le variant de fail-fast, inchangé.

### D. Comportement métier préservé (non-régression)

- **And** un plan comptable **standard** (seedé par Kesh, rôles posés par le backfill 14-3a) : l'onboarding et la génération de facture continuent de viser les comptes 1100/3000/2000 exactement comme avant — parce qu'ils portent désormais les rôles correspondants. **Aucune différence observable pour l'utilisateur standard.**
- **And** **portée réelle du gain Chantier C (à ne pas surestimer, finding validate P1).** `insert_with_defaults`/`_in_tx` ne tournent qu'à l'**onboarding** (2 appelants : seed démo, finalize onboarding) ; après onboarding, `default_receivable_account_id` est une **FK figée** reconfigurable seulement à la main via `settings/invoicing` (mécanisme indépendant des rôles, préexistant). Le bénéfice de Chantier C est donc : **(a)** éliminer le **dernier hardcode par numéro** du code de production (robustesse de principe — un futur chart JSON à numérotation différente restera correct sans re-coder), **et** **(b)** cohérence avec le principe fondateur « aucune ligne de Rust ne déduit un rôle d'un numéro ». Ce n'est **pas** un gain observable dans un parcours de renumérotation post-onboarding (celui-ci passe par `settings/invoicing`). Le test « plan renuméroté » (AC-E) est donc un test **unitaire de la fonction de résolution**, pas un test de parcours utilisateur.
- **And** les flux automatiques (facture/avoir/réconciliation) ne subissent **aucune** régression de la garde de postabilité (D-A0, `enforce_postable=false`), à prouver par un test.
- **And** aucune écriture manuelle existante ne devient inéditable (grandfather par compte D-A1, à prouver par un test).

### E. Tests

- **Backend `journal_entries` (repo)** :
  - create manuel (`enforce_postable=true`) : ligne vers compte non-postable → `InactiveOrInvalidAccounts` ; ligne vers compte postable → OK.
  - **flux automatique (`enforce_postable=false`) : ligne vers compte non-postable → OK** (non-régression D-A0). Le plus simple : appeler `create_in_tx` directement avec le flag `false` et un compte non-postable, vérifier le succès.
  - update grandfather **par compte** : écriture manuelle sur compte X, X devient non-postable (créer un enfant sous X), éditer l'écriture sans toucher X → **OK** ; ajouter une ligne vers un compte non-postable Y **jamais référencé** → **rejet**. *(Documenter le test discriminant : ajouter une 2e ligne sur X déjà référencé est TOLÉRÉ — c'est la brèche L3, l'assertion doit refléter le comportement réel, pas l'inverse.)*
  - le compte de résultat (`CurrentYearResult`, non-postable) est refusé à la saisie manuelle.
- **Backend `company_invoice_settings`** :
  - plan standard : `insert_with_defaults` résout les mêmes ids qu'avant (par rôle).
  - plan renuméroté (débiteurs = `Receivable` sur un numéro ≠ 1100) : la résolution trouve le bon compte.
  - plan sans compte `Receivable` actif : fail-fast `InactiveOrInvalidAccounts` (miroir des deux fonctions).
  - la retry-loop de seed et le finalize onboarding restent verts (tests E2E existants `onboarding_e2e` / seed).
- **Frontend** : Vitest sur `AccountAutocomplete` — un compte `postable === false` n'apparaît pas dans les options ; un compte `active && postable` apparaît. (Les `<select>` natifs et la page sont couverts par Playwright si praticable, sinon documenté comme testé via le filtre unitaire.)
- **E2E Playwright** (si praticable sur le harnais local) : dans un sélecteur de saisie d'écriture, un compte non-postable n'est pas sélectionnable.

## Tasks / Subtasks

- [ ] **T1 — Garde postabilité backend, factorisée + manuelle uniquement (AC-A, D-A0)**
  - [ ] Extraire un helper `validate_lines_accounts_in_tx(tx, company_id, account_ids, enforce_postable, exempt_ids) -> Result<(), DbError>`, **sans rollback interne** (retourne `Err`, le caller décide). Le SQL applique `AND postable = TRUE` **conditionnellement** à `enforce_postable` (sinon la clause n'est pas ajoutée), et exempte `id IN (exempt_ids)`.
  - [ ] Ajouter le paramètre `enforce_postable: bool` à `create_in_tx` (signature). **Répercuter sur TOUS les appelants** : `create` pool-level (manuel) → `true` ; les ~7 appelants automatiques → `false` (`invoices.rs`, `credit_notes.rs`, `supplier_invoices.rs` ×3, `reconciliation.rs`/`manual.rs`/`split.rs` — grep `create_in_tx` pour la liste exacte, un oubli casserait un flux ou introduirait la régression D-A0).
  - [ ] `create_in_tx` : appeler le helper avec `exempt_ids = []`.
  - [ ] `update` (manuel, `enforce_postable=true` implicite) : **réordonnancement nécessaire** — le bloc de validation actuel (`:696-719`, « Étape 5 ») s'exécute AVANT le fetch des lignes existantes (`before_lines`, « Étape 6 », `:730-736`). Récupérer d'abord les `account_id` déjà référencés (`SELECT DISTINCT account_id FROM journal_entry_lines WHERE entry_id = ?` ou réutiliser `before_lines`), **puis** appeler le helper avec `exempt_ids` = ces comptes. Conserver le rollback pré-erreur du caller.
- [ ] **T2 — Filtres postable frontend (AC-B)**
  - [ ] `AccountAutocomplete.svelte:32` → `.filter((a) => a.active && a.postable)`.
  - [ ] `<select>` natifs saisie : `supplier-invoices/+page.svelte:128`, `import/+page.svelte:102`, `[id]/+page.svelte:66`, `RuleFormModal.svelte:42` → `&& a.postable`.
  - [ ] Config postés : `settings/invoicing/+page.svelte:47-55`, `bank-accounts/+page.svelte:78`, `BankAccountJournalLinkForm.svelte:44` → `&& a.postable`.
  - [ ] **Ne pas toucher** `accounts/+page.svelte:146` (sélecteur parent).
  - [ ] **Corriger la chaîne i18n `account-postable-hint`** (AC-B / finding validate P1) : aujourd'hui « Indicatif — la saisie d'écriture ne le bloque pas encore » (`fr-CH/messages.ftl:176`, utilisée 3× dans `accounts/+page.svelte` : badge `:424`, dialog Créer `:532`, dialog Modifier `:604`). Après 14-3b elle **ment** in-app. La reformuler dans les **4 locales** (fr/de/it/en-CH) pour refléter le blocage effectif à la saisie manuelle (ex. « La saisie d'écriture manuelle sur ce compte est bloquée »), en gardant l'alignement clé-pour-clé (`lint-i18n-ownership`).
- [ ] **T3 — Lookups facturation par rôle (AC-C)**
  - [ ] `insert_with_defaults` : 3 lookups `number` → `role = ?` (Receivable/DefaultRevenue/Payable), `active`/`FOR UPDATE`/`.flatten()` conservés.
  - [ ] `insert_with_defaults_in_tx` : idem, symétrique (bloc MIRROR).
  - [ ] Mettre à jour la docstring `:249-260`.
- [ ] **T4 — Tests (AC-E)** : repo `journal_entries` (create manuel refus / **create auto `enforce_postable=false` accepté** / update grandfather par compte / résultat), repo `company_invoice_settings` (standard/renuméroté/fail-fast), Vitest `AccountAutocomplete`, E2E si praticable.
- [ ] **T5 — Doc & gate** : CHANGELOG (postabilité **bloquante à la saisie manuelle** ; flux automatiques inchangés ; facturation résolue par rôle) ; **chaîne i18n `account-postable-hint` corrigée aux 4 locales** (cf. T2) ; manuel utilisateur — corriger la mention « indicatif » posée par 14-3a (le badge « Non postable » devient bloquant pour la saisie manuelle) ; gate complet (`scripts/test-fast.sh` ou la série CLAUDE.md) ; README feuille de route si nécessaire.

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-23)

**Chantier A — `crates/kesh-db/src/repositories/journal_entries.rs`** (0 occurrence de `postable` aujourd'hui) :
- **`create` (pool-level, `:55`)** = point d'entrée **manuel**, unique appelant `routes/journal_entries.rs:494`. Ouvre la tx et appelle `create_in_tx`. C'est ici que `enforce_postable=true` est passé.
- **`create_in_tx` (`:90`)** = point d'entrée **partagé**, appelé directement par les flux **automatiques**. Appelants à répercuter en `enforce_postable=false` (grep `create_in_tx` dans `crates/kesh-db/src/repositories/` — sous-ensemble métier : `invoices.rs`, `credit_notes.rs`, `supplier_invoices.rs`, `reconciliation.rs`/`manual.rs`/`split.rs` ; ignorer les fichiers hors écriture comme `api_keys.rs`/`users.rs` qui matchent un `create_in_tx` homonyme d'un AUTRE repository — vérifier que c'est bien `journal_entries::create_in_tx`).
- create : SQL de validation `:152-155` (`WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})`), erreur `:167` `InactiveOrInvalidAccounts`. Bloc `:146-168`. Rollback **délégué au caller**.
- **`update` (pool-level, `:584`)** = manuel, unique appelant `routes/journal_entries.rs:600`. **Il n'existe PAS de `update_in_tx`** (contrairement à create) : `update` ouvre sa propre tx et gère **tout** le rollback en interne (chaque `return Err` est précédé de `tx.rollback()`). SQL validation `:702-705`, erreur `:718`, bloc `:696-719`. **`before_lines` fetché à l'« Étape 6 » (`:730-736`), APRÈS le bloc de validation** → réordonner (cf. T1).
- Le helper factorisé doit être **rollback-agnostique** (retourne `Err`, ne rollback pas) : `create_in_tx` délègue au caller, `update` rollback autour de l'appel.
- **Patron SQL de lookup par rôle** déjà dans le repo : `accounts.rs` `find_singleton_role_holder` (`WHERE company_id = ? AND role = ? AND active = TRUE`, `.bind(role)`) ; `AccountRole` implémente `Encode<MySql>` donc `.bind(AccountRole::X)` fonctionne directement.

**Chantier C — `crates/kesh-db/src/repositories/company_invoice_settings.rs`** :
- `insert_with_defaults` (`:264-379`, pool-level) : lookups `:275` (1100), `:284` (3000), `:296` (2000, optionnel) ; fail-fast `:308-311` (avec rollback). Appelée par `kesh-seed/src/lib.rs:189` (retry-loop matchant `InactiveOrInvalidAccounts`).
- `insert_with_defaults_in_tx` (`:387-486`, tx-level, MIROIR) : lookups `:396`/`:405`/`:416` ; fail-fast `:427-430` (sans rollback, le caller possède la tx). Appelée par `onboarding.rs:720`.
- `AND active = true` **déjà présent** partout ; `FOR UPDATE` présent. Re-check FK `rows == 0` par id : `:338-367` / `:452-475` — ne pas toucher.

**Mapping d'erreur** : `kesh-db/errors.rs` variant `InactiveOrInvalidAccounts` (`:48-52`, code `:168`) → `kesh-api/errors.rs:2140-2147` = **400** `INACTIVE_OR_INVALID_ACCOUNTS`, message i18n `error-inactive-accounts`. Réutilisé, pas modifié (D-A2).

**Frontend** — cf. tableau de la section D-C2 / AC-B. Le composant central `AccountAutocomplete.svelte:32` couvre à lui seul 4 sites de saisie ; les autres sont des `<select>` natifs avec leur propre `.filter`.

### Invariant légué de 14-3a (à ne pas perdre)

Un rôle singleton n'est unique **que parmi les comptes actifs** (`archive()` ne remet pas `role` à `NULL`, la colonne générée est `active`-aware). **Tout lookup par rôle DOIT porter `AND active = TRUE`** — sinon un `WHERE role = ?` nu peut ramener deux lignes (une archivée, une active) après un cycle archive → reprise. Les 6 lookups de facturation l'ont déjà ; ne pas l'enlever en passant au rôle.

### Pièges, par ordre de coût

1. **Oublier `enforce_postable=false` sur un flux automatique (D-A0)** — le risque n°1, purement runtime : un appelant automatique de `create_in_tx` laissé à `true` casse la validation de facture/avoir/réconciliation dès qu'un compte de config est non-postable. Répercuter le flag sur TOUS les appelants et **tester** qu'un flux auto poste sur un compte non-postable sans erreur.
2. **Grandfather de l'update (D-A1)** — le rater rend inéditables les écritures manuelles historiques. Runtime, invisible en validate statique. Test « X devient non-postable, l'écriture reste éditable ».
3. **Réordonnancement de `update`** — `before_lines` est fetché APRÈS le bloc de validation (`:730-736` vs `:696-719`). Sans réordonner, `exempt_ids` est vide au moment de valider → grandfather cassé.
4. **Symétrie des deux fonctions miroir de `company_invoice_settings`** — modifier une seule fait diverger onboarding et seed. Les deux triplets changent ensemble.
5. **Rollback dans le helper factorisé** — `create_in_tx` délègue, `update` rollback. Le helper retourne `Err`, ne rollback pas.
6. **Ne pas dé-dupliquer les blocs MIRROR** de `company_invoice_settings` (dette assumée HRTB SQLx 0.8). Factoriser A (`journal_entries`), oui ; miroir C, non.
7. **`postable` sur le sélecteur parent** — l'y ajouter serait l'inverse du besoin. Exclu.
8. **Chaîne i18n `account-postable-hint`** — devient un mensonge in-app si non corrigée aux 4 locales (T2/T5).

### Hors scope (garde-fous)

- ❌ Présentation des fonds propres **par rôle** au bilan (`balance_sheet.rs`/CSV/PDF/`BalanceSheetView`) → **14-3c**.
- ❌ Toute **migration** ou nouvelle colonne — 14-3b ne touche pas le schéma.
- ❌ Nouveau variant d'erreur pour « non-postable » (D-A2 : réutilise l'existant).
- ❌ Dé-duplication des fonctions miroir de `company_invoice_settings` (dette assumée).
- ❌ Rôles de trésorerie caisse/banque (écartés en 14-3a).
- ❌ Appliquer la garde de postabilité aux **flux automatiques** (D-A0 : `enforce_postable=false`).
- ❌ Empêcher un compte de rôle de devenir non-postable (option écartée : toucherait `effective_postable` de 14-3a).

### Limitations documentées (catégorie B — tracées, cf. CLAUDE.md § Tech debt management)

- **L1 (héritée 14-3a)** — rôles singleton mono-valués : un seul compte actif par société pour `Receivable`/`DefaultRevenue`/`Payable`. Aligné sur `company_invoice_settings` (un seul `default_*_account_id`). Lever = passer la config en listes, hors v0.1.
- **L2 — flux automatiques exemptés de la garde de postabilité (D-A0).** Un compte de config (créance/produit/dette) devenu non-postable *après* configuration reste posté par les flux automatiques (facture/avoir/réconciliation). Non-corrupteur (posting sur un parent), et prévenu en amont par le filtre `postable` des sélecteurs de config (D-C2). Remédiation éventuelle : re-vérifier `postable` à la résolution avec message dédié — amélioration future si un besoin se manifeste.
- **L3 — grandfather par compte, pas par ligne (D-A1).** À l'édition d'une écriture manuelle, on peut ajouter des lignes sur un compte non-postable **déjà référencé** dans cette écriture (pas d'identité de ligne stable, `DELETE`+ré-INSERT). Brèche mineure (compte visible, action explicite, jamais un compte système). Fermer = comparaison multiset avant/après, jugée disproportionnée.
- **L4 — message d'onboarding générique.** Un plan sans compte de rôle `Receivable`/`DefaultRevenue` actif fait échouer l'init de config facturation avec `InactiveOrInvalidAccounts` (ne nomme pas le rôle manquant). Faible fréquence (onboarding uniquement, charts standards toujours annotés). Améliorer le message = amélioration future.
- **L5 — `<select>` de config natif : valeur non-postable persistée affichée vide (code-review pass 2 ECH-F1).** Corollaire frontend de L2 : les `<select>` natifs de config (`settings/invoicing`, `bank-accounts`, `RuleFormModal`) filtrent `postable` (D-C2) ; si un compte de config devient non-postable *après* configuration, le champ s'affiche **vide** et une interaction utilisateur peut le **nuller silencieusement** → facturation cassée en `ConfigurationRequired`. Précondition rare (même déclencheur que L2). `AccountAutocomplete` (saisie d'écriture) n'est pas concerné (résout le libellé via la liste complète). Remédiation = inclure la valeur courante dans les options même non-postable. **Reclassé dette v0.2 (décision Guy), tracé [issue #271](https://github.com/guycorbaz/kesh/issues/271)** — hors périmètre 14-3b pour ne pas élargir le chantier frontend.

### Doc-sync (T5)

- **CHANGELOG** : la postabilité, **indicative** en 14-3a (badge « Non postable »), devient **bloquante à la saisie manuelle** (les flux automatiques restent inchangés) ; la facturation résout ses comptes de config par rôle (élimination du dernier hardcode par numéro).
- **Chaîne i18n `account-postable-hint`** (runtime, 4 locales) : corriger la promesse « ne le bloque pas encore » → blocage effectif (cf. T2/T5). C'est le seul endroit où l'utilisateur **voit** l'affirmation obsolète en direct.
- **Manuel utilisateur** : corriger la mention « indicatif » posée en 14-3a → bloquant à la saisie manuelle.
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

## Change Log — validate

### Pass 1 (Sonnet ×2, contexte frais, 2026-07-23) — 1 CRITICAL + 2 HIGH + 3 MEDIUM → tous patchés

Deux lentilles parallèles : (a) ancres ground-truth & faisabilité, (b) métier comptable & complétude AC.
- **CRITICAL** — la garde postable partageait `create_in_tx` avec les flux automatiques (facture/avoir/réconciliation) → régression du moteur comptable si un compte de config devient non-postable. **Décision Guy** : garde MANUELLE uniquement (`enforce_postable`, D-A0).
- **HIGH** — `update_in_tx` inexistant (c'est `update`, rollback interne, `before_lines` fetché après la validation) → corrigé + réordonnancement prescrit (T1).
- **HIGH** — grandfather ambigu (`DELETE`+ré-INSERT, pas d'identité de ligne). **Décision Guy** : grandfather PAR COMPTE + brèche L3 assumée.
- **MEDIUM** — chaîne i18n `account-postable-hint` (« ne bloque pas encore ») devient un mensonge in-app → correction 4 locales (T2/T5).
- **MEDIUM** — gain Chantier C surestimé (résolution par rôle seulement à l'onboarding) → AC-D reformulé.
- **MEDIUM** — message onboarding générique → accepté v0.1 (L4).
- **LOW** — RuleFormModal `:42`→`:43`.

Contrôle positif : la grande majorité des ancres vérifiées exactes au caractère près, « dernier hardcode par numéro » confirmé par grep, point de passage unique `create_in_tx` confirmé pour tous les flux, `CurrentYearResult`/`RetainedEarnings` sains vis-à-vis de la garde.

### Pass 2 (Haiku 4.5, contexte frais, grep ground-truth, 2026-07-23) — **CONVERGÉ (0 > LOW)**

Vérification par grep des ancres introduites/modifiées par la passe 1 : 9 appelants réels de `journal_entries::create_in_tx` (aucun oublié dans T1), 6 lookups `:275/:284/:296/:396/:405/:416` avec `active=true`, réordonnancement `update` (`:696-719` avant `:730-736`) confirmé, `account-postable-hint` présente aux 4 locales + 3 sites. Cohérence interne AC↔décisions↔tasks vérifiée, hors-scope net. **0 finding > LOW.**

### Décision — validate

**Trend** : Pass 1 Sonnet ×2 (1 CRIT + 2 HIGH + 3 MED) → Pass 2 Haiku (0 > LOW). **CONVERGÉ en 2 passes**, LLM distincts, contexte frais, patchs appliqués avant la passe 2. Critère d'arrêt Review Iteration Rule atteint. 2 décisions de conception tranchées par Guy (D-A0 garde manuelle, D-A1 grandfather par compte). Spec **ready-for-dev**.

## Change Log — code review (post-dev)

Dev-story implémentée (chantiers A + C), puis 3 couches adversariales (Blind Hunter / Edge Case Hunter / Acceptance Auditor) par passe, cycle LLM **Opus → Sonnet → Haiku**, contexte frais à chaque passe, diff aplati `base..HEAD` pour la passe Haiku (garde-fou CLAUDE.md indexation multi-commit).

### Pass 1 (Opus ×3, 2026-07-23) — 2 MEDIUM
- **AA-F1 (MED)** `CHANGELOG.md` disait encore « la saisie n'est pas encore bloquée » (hérité 14-3a, faux depuis 14-3b) → corrigé.
- **AA-F2 (MED)** `docs/manual/fr/user-manual.tex` même mention obsolète → corrigée + PDF régénéré.
- Dismiss : BH grandfather-picker (le `$effect` d'`AccountAutocomplete` résout le libellé via la liste complète), BH sérialisation `postable` (champ obligatoire type-checked). Defer LOW : `<select>` config, asymétrie réconciliation D-A0, grandfather active/postable pré-existant.

### Pass 2 (Sonnet ×3, 2026-07-23) — 5 MEDIUM (4 patchés + 1 différé)
- **BH-F2 (MED)** formulation doc « nouvelle ligne » imprécise (contredite par le test L3) → « ligne vers un compte non-postable pas déjà utilisé par l'écriture » (CHANGELOG + manuel).
- **AA-F1 (MED)** test fail-fast AC-E absent sur le miroir `insert_with_defaults_in_tx` → test `insert_with_defaults_in_tx_fails_fast_when_no_receivable_role` ajouté.
- **AA-F2 (LOW/MED)** 3 commentaires/docstrings citant `1100/3000` en dur → mis à jour par rôle.
- **ECH-F2 (MED perf, décision Guy = corriger)** les 6 lookups `WHERE role = ? AND active = true` n'utilisaient pas l'index `uq_accounts_company_singleton_role` (scan au lieu de `const`) → `WHERE singleton_role = ?` (encode déjà `active`, réalise l'intention documentée de la migration 14-3a).
- **ECH-F1 (MED, décision Guy = dette v0.2)** `<select>` config natif affiche vide une valeur non-postable persistée → nullification silencieuse possible. **Différé** : [issue #271](https://github.com/guycorbaz/kesh/issues/271) + limitation **L5**.
- Dismiss : BH-F1 HIGH réfuté → LOW (post_manual/split `enforce_postable=false` est la décision D-A0, filtré côté frontend).

### Pass 3 (Haiku ×3, diff aplati, 2026-07-23) — **CONVERGÉ (0 > LOW)**
Blind Hunter = 0 (« READY FOR MERGE »), Edge Case Hunter = 1 LOW (fallback `loadError` d'`AccountAutocomplete` : saisie ID direct sans filtre `postable`, mais rejeté en aval par le backend `enforce_postable=true` — aucune corruption), Acceptance Auditor = 0 (100 % conforme AC-A à AC-E, D-A0/D-A1/D-A2/D-C1/D-C2). **0 finding > LOW.**

### Décision — code review

**Trend** : Pass 1 Opus (2 MED) → Pass 2 Sonnet (5 MED : 4 patchés + 1 différé #271) → Pass 3 Haiku (**0 > LOW**). **CONVERGÉ en 3 passes**, cycle Opus→Sonnet→Haiku, contexte frais, patchs appliqués avant chaque passe suivante. Critère d'arrêt Review Iteration Rule atteint. 2 décisions de scope tranchées par Guy (ECH-F2 corriger via `singleton_role`, ECH-F1 dette v0.2 #271). Gate : 1947/1947 backend + 426 frontend + gate final workspace vert.
