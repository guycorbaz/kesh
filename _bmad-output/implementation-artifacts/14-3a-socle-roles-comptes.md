# Story 14.3a : Socle des rôles de comptes — attribut explicite, chart-agnostique

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, association) ou fiduciaire qui adapte son plan comptable,
**I want** que chaque compte porte un **rôle explicite** (créances clients, TVA due, capital, résultat reporté…) que je peux consulter et modifier, ainsi qu'un indicateur **postable / non-postable**,
**So that** Kesh cesse de deviner la fonction d'un compte à partir de son **numéro** — mon plan comptable n'est pas légalement figé, je dois pouvoir le renuméroter ou le restructurer sans casser la comptabilité.

## Contexte

### Le principe (Guy, 2026-07-21)

> **Chaque compte a un RÔLE explicite. Le numéro ne sert JAMAIS à déduire le rôle.**

Le plan comptable suisse (norme Käfer / PME) est un **usage**, pas une obligation légale. Un utilisateur peut renuméroter ses comptes, en ajouter, en supprimer. Tout code qui écrit `WHERE number = '1100'` est un piège silencieux.

### État des lieux (cartographie 2026-07-22, 2 agents Explore + vérification directe)

- **14-1 a retiré** le hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` de `balance_sheet.rs` en rendant les fonds propres **entièrement virtuels**. Ce trou est fermé.
- **Le seul hardcode fonctionnel restant** en Rust de production est `company_invoice_settings.rs` : 6 lookups `WHERE number = '1100' / '3000' / '2000'` (`:275`, `:284`, `:296` + miroir `:396`, `:405`, `:416`) + un fail-fast d'onboarding `if receivable.is_none() || revenue.is_none()` (`:308-311`, `:428-430`). **Sa suppression est la Story 14-3b**, pas celle-ci.
- **Aucune notion de compte non-postable n'existe.** Les comptes titres (`1`, `10`, `28`) sont postables comme les autres ; la seule validation à la saisie est `active = TRUE` (`journal_entries.rs:152-156` create, `:702-706` update, 2 copies).
- **Aucune réactivation de compte archivé** (issue **#269**) : `accounts::update` rejette explicitement un compte archivé (`:186-192` `IllegalStateTransition`), aucune fonction `reactivate`, aucune route, aucun bouton. Le précédent existe pour les projets (`unarchive_project`, `POST /projects/{id}/unarchive`).

### Décision de découpage (Guy, 2026-07-22)

La story 14-3 initiale touchait 6 crates + le frontend → **règle de splitting préventif** (CLAUDE.md, seuil 5 modules) déclenchée. Split validé :

- **14-3a (cette story) — SOCLE.** Pose le modèle et l'UI d'administration : colonnes `role` + `postable`, enum `AccountRole`, entity/repo/API, 3 plans comptables annotés, page Plan comptable enrichie, **réactivation #269**, i18n. **Aucun consommateur du rôle.** Le comportement métier de l'application est **strictement inchangé**.
- **14-3b — CONSOMMATEURS** (story suivante, rollout mécanique) : garde de postabilité à la saisie d'écriture + filtres frontend (7 sites), présentation des fonds propres **par rôle** au bilan (backend + CSV + PDF + frontend), remplacement des 6 lookups par numéro de `company_invoice_settings` par des lookups par rôle.

Ce découpage suit le pattern « story-zero pose le pattern → rollout mécanique » de CLAUDE.md.

### Décision — vocabulaire des rôles (Guy, 2026-07-22 : périmètre « étendu »)

**10 rôles**, stockés en `VARCHAR(32)` PascalCase (même convention que `account_type`), `NULL` = aucun rôle :

| Rôle | Sens | Compte des plans livrés | Singleton ? |
|---|---|---|---|
| `Receivable` | Créances clients (débiteurs) | 1100 | ✅ |
| `DefaultRevenue` | Produit par défaut de facturation | 3000 | ✅ |
| `Payable` | Dettes fournisseurs (créanciers) | 2000 | ✅ |
| `VatRecoverable` | Impôt préalable (TVA récupérable) | 1171 | ✅ |
| `VatPayable` | TVA due | 2200 | ✅ |
| `VatSettlement` | Décompte TVA | 2206 | ✅ |
| `EquityCapital` | Capital (social / exploitant / association) | 2800 | ❌ multi |
| `EquityOther` | Autres fonds propres — réserves, fonds affectés/libres, prélèvements et apports privés | 2900 (PME), 2850 + 2860 (association, indépendant) | ❌ multi |
| `RetainedEarnings` | Bénéfice / perte reporté | 2970 | ✅ |
| `CurrentYearResult` | Résultat de l'exercice | 2979 | ✅ |

**Singleton** = au plus **un compte ACTIF** par société peut porter ce rôle (il alimente un champ de configuration unique ou une ligne unique du bilan). `EquityCapital` / `EquityOther` sont multi-valués : l'indépendant a 2800 capital + 2850 prélèvements + 2860 apports, l'association a 2850 fonds affectés + 2860 fonds libres.

**Pourquoi `EquityOther` et non `EquityReserve`** : les numéros 2850/2860 portent des sens **différents selon le plan** — « Fonds affectés / Fonds libres » en association (des fonds au sens propre), « Prélèvements privés / Apports privés » chez l'indépendant (des mouvements de capital de l'exploitant, de signes opposés). Les nommer « réserve » dans le modèle ferait afficher les prélèvements personnels d'un indépendant sous un intitulé « Réserves » en 14-3b — contresens pour un persona cible. Un intitulé **neutre** au niveau du modèle, la sémantique fine étant portée par le **nom du compte lui-même** (déjà multilingue dans les charts) et par l'i18n de la section. Corollaire : le backfill reste **indépendant du `org_type`** (un seul `WHERE number IN ('2900','2850','2860')`), ce qu'un découpage sémantique fin aurait rendu impossible.

**Homogénéité réelle des 3 plans** (vérifiée entrée par entrée) : les **9 autres** rôles utilisent **exactement les mêmes numéros** dans `pme.json`, `association.json` et `independant.json` (1100, 3000, 2000, 1171, 2200, 2206, 2800, 2970, 2979). Seul `EquityOther` diffère, et de façon **disjointe** : `2900` en PME (jamais 2850/2860), `2850`+`2860` en association et indépendant (jamais 2900). Un backfill unique `WHERE number IN ('2900','2850','2860')` couvre donc les 3 plans sans collision — mais **ne pas** en conclure qu'un rôle = un numéro partout.

### Décision — unicité STRUCTURELLE des rôles singleton (vérifiée empiriquement sur MariaDB)

Un contrôle applicatif « SELECT puis INSERT » laisse une fenêtre de course. Fix **structurel** : une colonne générée `VIRTUAL` + un index UNIQUE.

```sql
singleton_role VARCHAR(32) GENERATED ALWAYS AS (CASE WHEN active AND role IN (<les 8 singletons>) THEN role ELSE NULL END) VIRTUAL
CONSTRAINT uq_accounts_company_singleton_role UNIQUE (company_id, singleton_role)
```

`NULL` n'entre pas dans une contrainte UNIQUE MySQL/MariaDB → les comptes sans rôle et les rôles multi-valués sont libres.

**Ce n'est pas une invention : le repo possède déjà exactement ce pattern.** `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:54` — `active_uniq VARCHAR(255) GENERATED ALWAYS AS (IF(active, match_value, NULL)) VIRTUAL` + `CONSTRAINT uq_reconciliation_rules_match_active UNIQUE (company_id, match_type, active_uniq)` (`:68-69`), documenté `:20-28` comme le « Workaround Option A » au **UNIQUE partiel** que MariaDB n'a pas nativement, avec la même annulation sur `active`. **Suivre ce précédent** : même mécanique, `VIRTUAL` (pas `STORED`), même style de commentaire. Pré-requis déjà acté là-bas : MariaDB ≥ 10.6 pour un UNIQUE sur colonne `VIRTUAL` (le compose épingle 10.11+).

**Note DDL** : l'ajout d'une colonne `VIRTUAL` est instantané (pas de réécriture de lignes), mais l'`ADD CONSTRAINT UNIQUE` construit un index — `ALGORITHM=INSTANT` reste inapplicable à l'ensemble de l'`ALTER`. Sans conséquence pratique : ~84 comptes par société.

**Le `active AND` est indispensable, pas cosmétique.** Sans lui, un compte **archivé** squatte son rôle singleton à vie : l'utilisateur qui archive son compte 1100 d'origine et en crée un autre (le numéro n'étant pas modifiable après création, `entities/account.rs:96`) ne pourrait **jamais** donner `Receivable` au remplaçant — 409 permanent causé par un compte mort. Le code que cette story doit remplacer applique déjà ce filtre (`company_invoice_settings.rs:275` : `… AND active = true …`) ; l'oublier ici serait une régression sur le mécanisme censé le durcir.

**Conséquence assumée** : réactiver un compte archivé dont le rôle singleton a **entre-temps** été repris par un compte actif **échoue** (le rôle doit d'abord être libéré). C'est le comportement correct — il doit être exposé comme un 409 explicite, pas comme une erreur 500 (AC-E).

**Vérifié sur le conteneur `kesh-mariadb` (2026-07-22)**, colonne générée `active`-aware comprise :
- plusieurs `NULL` OK, plusieurs `EquityOther` sur la même société OK, même rôle singleton sur deux sociétés différentes OK ;
- doublon singleton actif sur la même société → **`ERROR 1062 Duplicate entry '1-Receivable'`** ;
- `UPDATE … SET active = FALSE` sur le porteur → `singleton_role` recalculé à `NULL`, le rôle est **libéré** ; un compte actif peut alors le prendre ;
- `UPDATE … SET active = TRUE` sur l'ancien porteur alors que le rôle est repris → **`ERROR 1062`** propre (pas de corruption) ;
- `ALTER TABLE ADD COLUMN … STORED + ADD CONSTRAINT UNIQUE` puis `UPDATE` de backfill dans le **même** script fonctionne sur une table existante peuplée ; `ALGORITHM=INSTANT` est refusé (`ERROR 1845`) comme attendu.

**Le backup/restore tolère nativement la colonne générée** — `backup.rs:88-96` (`non_generated_columns`) filtre déjà `EXTRA NOT LIKE '%GENERATED%'` et le doc-comment de `TableExport.column_names:73` dit explicitement « hors colonnes générées ». Rien à faire de ce côté (mais un test le prouve, cf. AC-G).

### Décision — backfill par numéro, UNE SEULE FOIS, en migration (Guy, 2026-07-22)

Les installations existantes (dont le NAS de Guy en v0.7.0) ont déjà un plan comptable **créé par Kesh à partir des plans livrés**. La migration affecte donc les rôles par numéro, `WHERE role IS NULL AND active = TRUE` (idempotent, et sans donner de rôle singleton à un compte déjà archivé).

**Le numéro sert une fois, dans une migration de données — jamais dans le code applicatif.** Le principe reste tenu : après la migration, aucune ligne de Rust ne déduit un rôle d'un numéro. L'utilisateur peut tout modifier depuis la page Plan comptable.

### Décision — postabilité : deux backfills, tous deux chart-agnostiques

`postable BOOLEAN NOT NULL DEFAULT TRUE`. Backfill à `FALSE` pour :

1. **Les comptes qui ont des enfants** (comptes titres / de regroupement) — `WHERE EXISTS (SELECT 1 FROM accounts c WHERE c.parent_id = a.id)`. **Purement structurel, aucun numéro.**
2. **Le compte de rôle `CurrentYearResult`** — en modèle temps réel virtuel (14-1), l'application **calcule** le résultat de l'exercice à chaque rendu ; y poster une écriture serait un double-comptage garanti. C'est le « durcissement de 14-1 » annoncé dans `balance_sheet.rs:36-38`.

**`RetainedEarnings` reste postable** — décision explicite : un utilisateur qui **migre** depuis un autre logiciel doit pouvoir poser son report à nouveau d'ouverture. Le rendre non-postable casserait le persona cible de la Story 14-4. L'utilisateur peut le passer non-postable lui-même après migration.

Les deux backfills mettent la même valeur (`FALSE`) sur des ensembles qui peuvent se recouper : leur **ordre relatif n'affecte pas le résultat final**. Il est néanmoins fixé (#1 puis #2) par convention, pour que le script SQL soit lisible et reproductible à l'identique.

### Décision — où vit `AccountRole` : duplication CONTRÔLÉE, imposée par l'orphan rule Rust

Le workspace impose la réponse, il n'y a **rien à décider en dev** :

- `kesh-db` dépend de `kesh-core` (`crates/kesh-db/Cargo.toml:8`), **jamais l'inverse** ;
- `sqlx` n'est une dépendance que de `kesh-db` (`crates/kesh-db/Cargo.toml:16`) — `kesh-core` ne la connaît pas.

Donc : un `AccountRole` **unique** défini dans `kesh-core` ne peut **pas** recevoir les impls `Type<MySql>` / `Encode` / `Decode` depuis `kesh-db` — ni le trait ni le type ne seraient locaux à `kesh-db` → **`error[E0117]` (orphan rule)**, vérifié par compilation. Et l'inverse (`AccountRole` dans `kesh-db`, utilisé par `ChartEntry`) créerait un **cycle Cargo**.

**Décision : deux enums, comme `AccountType`.**

- `kesh-core::chart_of_accounts::AccountRole` — `Deserialize` seul, pour `ChartEntry` et `validate_chart`.
- `kesh-db::entities::account::AccountRole` — impls sqlx manuelles, pour la persistance.
- Conversion au **seul** point de contact (`bulk_create_from_chart`) via `as_str()` / `FromStr`, exactement comme `AccountType` aujourd'hui.

**Corollaire — correction d'une idée reçue** : la duplication de `AccountType` entre `kesh-core:16-21` et `kesh-db:11-17` n'est **pas** une dette à résorber, c'est la **conséquence structurelle** de la même contrainte. Ne pas tenter de la « nettoyer » au passage. Un test unitaire de cohérence (les deux enums ont les mêmes variantes et le même `as_str()`) est le bon garde-fou, pas la fusion.

### Hors scope (garde-fous — tout ceci est 14-3b ou plus tard)

- ❌ **Appliquer** `postable` à la saisie d'écriture (`journal_entries.rs:152-156` / `:702-706`) → **14-3b**. Ici la colonne est posée, lisible et modifiable, mais **rien ne la lit** côté métier.
- ❌ **Filtrer** les sélecteurs de compte du frontend (7 sites dupliqués) → **14-3b**.
- ❌ **Présentation des fonds propres par rôle** au bilan (`balance_sheet.rs`, CSV, PDF, `BalanceSheetView`) → **14-3b**.
- ❌ **Remplacer** les 6 lookups `number = '1100'/'3000'/'2000'` de `company_invoice_settings.rs` → **14-3b**.
- ❌ **Rôles de trésorerie** (caisse/banque) : écartés du vocabulaire (option non retenue).
- ❌ **Bilan d'ouverture / soldes de départ** → **Story 14-4**.
- ❌ **Réouverture d'exercice** → **Story 14-2**.
- ❌ **Migration du plan comptable d'un org_type à un autre**, import de plan personnalisé : hors Epic 14.

### Limitations documentées (catégorie B — tracées, cf. CLAUDE.md § Tech debt management)

- **L1 — rôles singleton mono-valués.** Un seul compte actif par société peut porter `Receivable`, `DefaultRevenue`, `Payable`, `VatRecoverable`, `VatPayable`, `VatSettlement`, `RetainedEarnings`, `CurrentYearResult`. Une PME souhaitant séparer débiteurs CH / étrangers, ou ventiler la TVA due par taux, ne le peut pas via les rôles (elle garde bien sûr autant de comptes qu'elle veut — ils n'ont simplement pas de rôle). C'est **aligné sur l'existant** : `company_invoice_settings` n'a qu'un `default_receivable_account_id`, un `default_revenue_account_id`, etc. Lever la limitation supposerait de passer ces champs de configuration en listes — **hors v0.1**. Remédiation : à créer comme issue GitHub `enhancement` + `v0.2-milestone` au moment où un besoin utilisateur réel se manifeste, pas avant (YAGNI).
- **L2 — l'attribut `postable` n'est pas appliqué pendant la fenêtre 14-3a → 14-3b.** Assumé et **rendu visible à l'utilisateur** (AC-H), pas silencieux.

### Point d'attention légué à 14-3b (à ne pas perdre)

**(1) Invariant de lookup — un rôle singleton n'est unique QUE parmi les comptes actifs.** `archive()` ne remet **pas** `role` à `NULL` (décision explicite : la réactivation doit pouvoir restaurer l'état d'origine), et la colonne générée est `active`-aware. La table peut donc légitimement contenir, après un cycle archive → reprise, **deux** lignes `role = 'Receivable'` dont une archivée. → **Tout lookup par rôle DOIT porter `AND active = TRUE`**, exactement comme les 6 lookups par numéro qu'il remplace (`company_invoice_settings.rs:275`). Un `WHERE role = ?` nu peut ramener plusieurs lignes.

**(2) Collision de libellés au bilan.** 
Le compte de rôle `RetainedEarnings` (2970) reste postable et **son solde réel apparaît déjà** dans le détail des passifs du bilan (rendu 14-1). En 14-3b, la présentation par rôle mettra côte à côte :

- la ligne itemisée du compte, p. ex. « 2970 Bénéfice/perte reporté : 50 000 » (solde d'ouverture posé par un migrant), et
- la ligne **calculée** « Résultat reporté : 5 000 » (cumul du P&L Kesh antérieur à `fy_start`).

Deux nombres différents sous des libellés quasi identiques. **L'arithmétique est saine** — l'équation de 14-1 tient (vérifié par calcul : actifs 55 000 = passifs 50 000 + reporté calculé 5 000 + résultat 0), c'est un problème de **lisibilité**, pas de comptabilité. 14-3b devra soit distinguer nettement les libellés (« Solde d'ouverture / ajustements » vs « Résultat reporté (calculé par Kesh) »), soit fusionner les deux dans une présentation unique. À traiter là-bas, mentionné ici pour que l'information ne se perde pas.

## Acceptance Criteria

### A. Schéma — colonnes `role` et `postable`

- **Given** la table `accounts` existante, **When** la migration s'applique, **Then** elle gagne :
  - `role VARCHAR(32) NULL` avec `CONSTRAINT chk_accounts_role CHECK (role IS NULL OR BINARY role IN (BINARY '<les 10 rôles>'))` — liste fermée, `BINARY` pour neutraliser la collation, **calquée sur `chk_accounts_type`** (`20260411000001_accounts.sql:20`) ;
  - `postable BOOLEAN NOT NULL DEFAULT TRUE` ;
  - `singleton_role VARCHAR(32) GENERATED ALWAYS AS (CASE WHEN active AND role IN (<les 8 singletons>) THEN role ELSE NULL END) VIRTUAL` — `VIRTUAL` comme le précédent `active_uniq` (`20260513000001_reconciliation_rules.sql:54`), et le `active AND` est **obligatoire** (§Décision unicité) ;
  - `CONSTRAINT uq_accounts_company_singleton_role UNIQUE (company_id, singleton_role)`.
- **And** la migration est **non-breaking** au sens de la politique P1/P3 (`ADD COLUMN` nullable + `ADD COLUMN NOT NULL DEFAULT` + nouvelle contrainte) → **PAS de bump `kesh_version_min_required`**. Justification à écrire dans l'en-tête SQL : un binaire antérieur sélectionne des colonnes **explicites** (`accounts.rs:17` `COLUMNS`, `:19` `FIND_BY_ID_SQL`) et ne verra jamais `role`/`postable`/`singleton_role` ; ses `INSERT` laissent `role` à `NULL` (aucun conflit sur l'UNIQUE) et `postable` à son défaut.
- **And** `docs/migrations-idempotence-audit.md` reçoit sa ligne (verdict attendu `tracked-by-sqlx`) **et** la section « Statistiques » est incrémentée (`Total : 54 → 55`, `tracked-by-sqlx : 43 → 44`). L'oubli est un finding MEDIUM en code review (politique P5).
- **And** `ALGORITHM=INSTANT` n'est **pas** applicable à l'`ALTER` complet (l'`ADD CONSTRAINT UNIQUE` construit un index) — le documenter dans l'en-tête SQL plutôt que de le tenter. Volume concerné : ~84 comptes par société.
- **And** `crates/kesh-db/tests/migrations_upgrade_path.rs` est mis à jour : `:77` `assert_eq!(total, 54)` → **55**, et `:105` `let n_before_upgrade_window = total - 20;` → **`total - 21`**. Le second n'est pas cosmétique : laisser `- 20` déplacerait la **frontière des 23 migrations historiques** que le commentaire `:90-104` impose de garder fixe, et le test continuerait de passer en testant autre chose (échec silencieux).

### B. Backfill des installations existantes

- **Given** une base issue d'une v0.7.0 avec un plan comptable seedé, **When** la migration s'applique, **Then** les rôles sont affectés selon la table de correspondance du §Contexte, **`WHERE role IS NULL AND active = TRUE`** (idempotent, ne surcharge jamais un rôle déjà posé, et ne donne pas de rôle à un compte archivé).
- **And** `postable = FALSE` pour tout compte ayant au moins un enfant (`parent_id` pointant sur lui), **sans référence à un numéro**.
- **And** `postable = FALSE` pour le compte de rôle `CurrentYearResult`.
- **And** `RetainedEarnings` reste `postable = TRUE`.
- **And** le backfill est **multi-société** : il porte sur toutes les lignes de `accounts`, pas sur une société particulière.
- **And** le backfill est un **best effort assumé et communiqué**. C'est le seul heuristique disponible, mais il contredit localement le principe fondateur de la story (« mon plan n'est pas figé, je peux renuméroter ») : un utilisateur ayant réaffecté `1100` à autre chose et mis ses débiteurs en `1101` recevra `Receivable` sur le mauvais compte, **sans erreur ni trace**, avec un effet différé en 14-3b. → entrée CHANGELOG explicite (« les rôles ont été pré-affectés d'après les numéros du plan comptable standard ; vérifiez la colonne Rôle de la page Plan comptable si vous avez renuméroté vos comptes ») **et** même avertissement dans la section Plan comptable du manuel utilisateur (T14).
- **And** le backfill ne peut **pas** violer `uq_accounts_company_singleton_role`. L'invariant qui le garantit est : **chaque rôle singleton est mappé à exactement UN numéro** dans la table de correspondance (le seul rôle mappé à plusieurs numéros, `EquityOther`, est multi-valué), combiné à `uq_accounts_company_number` qui interdit deux comptes de même numéro dans une société. *(Ce n'est pas l'unicité du numéro seule qui protège : si un futur rôle singleton couvrait deux numéros, elle ne suffirait plus.)* **À prouver par un test** (AC-G).

### C. Modèle Rust

- **Given** l'entité `Account`, **When** on la lit, **Then** elle expose `role: Option<AccountRole>` et `postable: bool`.
- **And** `AccountRole` est un enum suivant **exactement** le pattern de `AccountType` (`entities/account.rs:11-66`) : `as_str()`, `FromStr` strict, impls **manuelles** `Type<MySql>` / `Encode` / `Decode` (délégation à `String`) — **pas** de `#[derive(sqlx::Type)]`, cohérence avec l'existant.
- **And** `is_singleton(&self) -> bool` existe sur **les DEUX** `AccountRole` — celui de `kesh-db` (validations repo/API) **et** celui de `kesh-core` (`validate_chart` y est **privé**, `chart_of_accounts/mod.rs:88`, et `kesh-core` ne peut pas atteindre `kesh-db`). La liste des 8 singletons vit donc à **trois** endroits : le `CASE WHEN` de la migration, `kesh-db::AccountRole::is_singleton()`, `kesh-core::AccountRole::is_singleton()`. Commentaire croisé dans les trois fichiers **et** test de cohérence les couvrant tous les trois (AC-G) — dont une comparaison à l'expression SQL réelle lue via `SELECT GENERATION_EXPRESSION FROM information_schema.COLUMNS WHERE TABLE_NAME='accounts' AND COLUMN_NAME='singleton_role'`. C'est le seul garde-fou qui ferme les trois sources ; deux tests Rust qui se comparent l'un à l'autre laisseraient la migration dériver seule.
- **And** `NewAccount` gagne `role: Option<AccountRole>` et `postable: bool` ; `AccountUpdate` gagne `role: Option<AccountRole>` et `postable: bool`.
- **And** les **deux** listes de colonnes du repository sont mises à jour : `accounts.rs:17` (`COLUMNS`) **et** `accounts.rs:19` (`FIND_BY_ID_SQL`, qui duplique la liste en littéral) — omettre la seconde casse `FromRow` au runtime, pas à la compilation.
- **And** `account_snapshot_json` (`accounts.rs:27-38`) inclut `role` et `postable` (sinon l'audit log ment sur les modifications de rôle).
- **And** `is_no_op_change` (`accounts.rs:164-166`) compare aussi `role` et `postable` — sans quoi un changement de rôle seul est silencieusement ignoré (KF-004 court-circuit).

### D. API comptes

- **Given** `GET /api/v1/accounts`, **When** je liste, **Then** chaque `AccountResponse` porte `role` (`null` si aucun) et `postable` — ajout de champs, **contrat rétro-compatible**.
- **And** `POST /api/v1/accounts` accepte `role` (optionnel, défaut `null`) et `postable` (optionnel, défaut `true`).
- **And** `PUT /api/v1/accounts/{id}` prend `role` et `postable` **obligatoires** (avec `version`). **Décision explicite** : `UpdateAccountRequest` est un *full-replace* — `name` et `accountType` y sont déjà requis sans défaut (`routes/accounts.rs:38-42`), omettre `name` produit déjà un 400. Rendre `role`/`postable` requis est donc **cohérent** et surtout **sûr** : avec un `Option` laxiste, un client qui corrige la casse d'un libellé effacerait silencieusement le rôle du compte (et avec `#[serde(default)]` sur `postable`, rendrait postable un compte qui ne l'était pas). Une donnée perdue en silence est pire qu'un 400 explicite. Le changement de contrat du **PUT** (le GET reste rétro-compatible) est à signaler dans le CHANGELOG.
- **And** tests E2E dédiés : `PUT` sans `role` → **400** ; `PUT` avec `role: null` → le rôle est **effacé** (intention explicite du client) ; `PUT` sans `postable` → **400**.
- **Given** un rôle singleton déjà porté par un autre compte **actif** de la société, **When** je l'affecte à un second compte, **Then** l'API répond **409** avec un code client **dédié** `ACCOUNT_ROLE_ALREADY_ASSIGNED` et un `details` **nommant le compte en conflit** (`{ accountId, accountNumber, accountName }`).
- **Précision de conception (ne pas se contenter du mapping générique)** : `map_db_error` (`crates/kesh-db/src/errors.rs:166`) convertit **déjà** tout `1062` en `DbError::UniqueConstraintViolation`, et `crates/kesh-api/src/errors.rs:2001` le rend en `409 RESOURCE_CONFLICT` avec un message **fixe** (« Ressource déjà existante ») — le détail MySQL est seulement **logué**, jamais renvoyé. Ce chemin générique **ne peut pas** satisfaire l'exigence « nommer le compte en conflit ». Il faut donc, avant l'`INSERT`/`UPDATE`, un `SELECT` du compte portant déjà `(company_id, <rôle>)` et un variant `AppError` dédié. **Précédent exact à copier** : `AppError::IdeAlreadyExists` (`crates/kesh-api/src/errors.rs:190-196`), dont le doc-comment dit explicitement « Code client dédié […] distinct du générique `RESOURCE_CONFLICT` […] pour UX précise côté form ».
- **And** la contrainte DB reste la **source de vérité** : le `SELECT` préalable sert l'ergonomie, pas la correction. Une course perdue retombe sur le `1062` → `RESOURCE_CONFLICT` générique (409 aussi, message moins précis). **Ne pas** remplacer la contrainte par le `SELECT`.
- **And** RBAC inchangé : lecture pour tout rôle authentifié, écriture `comptable_routes` (`lib.rs:276-284`).

### E. Réactivation d'un compte archivé (issue #269)

- **Given** un compte archivé (`active = FALSE`), **When** un Admin/Comptable le réactive via `PUT /api/v1/accounts/{id}/reactivate` (body `{ version }`) — **`PUT` et non `POST /unarchive`** comme les projets : on privilégie la symétrie **locale** avec `PUT /api/v1/accounts/{id}/archive` déjà en place (`routes/accounts.rs:195`) plutôt que la symétrie cross-feature ; `unarchive_project` reste le modèle pour la **logique**, pas pour le verbe —, **Then** `active` repasse à `TRUE`, `version` est incrémenté et une entrée d'audit `account.reactivated` est écrite **dans la même transaction** (pattern `archive`, `accounts.rs:270+`).
- **And** la réactivation est **refusée** (`IllegalStateTransition` → 409) si le **compte parent est archivé** — symétrique du garde-fou de `archive()` qui refuse d'archiver un compte ayant des enfants actifs (`accounts.rs:278-290`). Sans ce garde, on obtient un compte actif sous un parent inactif : incohérence d'arborescence.
- **And** réactiver un compte **déjà actif** est un no-op idempotent qui retourne l'entité inchangée **sans** bumper `version` **ni** écrire d'audit (cohérent avec le court-circuit no-op KF-004 de `update`).
- **And** verrouillage optimiste : `version` incorrect → `OPTIMISTIC_LOCK_CONFLICT` (409).
- **Given** un compte archivé portant un rôle **singleton** que, depuis son archivage, un **autre compte actif** a repris, **When** je tente de le réactiver, **Then** l'API répond **409 `ACCOUNT_ROLE_ALREADY_ASSIGNED`** nommant le compte porteur, avec un message indiquant qu'il faut d'abord libérer le rôle — **jamais un 500**. C'est la conséquence directe du `active AND` de la colonne générée : la réactivation fait repasser `singleton_role` de `NULL` à sa valeur et heurte l'UNIQUE (`ERROR 1062`, vérifié empiriquement). Le détecter par un `SELECT` préalable dans la même transaction, comme en AC-D.
- **And** côté frontend, une ligne archivée de la page Plan comptable affiche un bouton **« Réactiver »** (aujourd'hui `{#if canModify() && account.active}` masque **toute** action sur les lignes archivées, `+page.svelte:287`).

### F. Plans comptables & seed

- **Given** `ChartEntry` (`kesh-core/src/chart_of_accounts/mod.rs:38-46`), **When** un plan JSON est désérialisé, **Then** un champ optionnel `role` est accepté (`#[serde(default)] pub role: Option<AccountRole>`) — **non-breaking** pour un JSON sans rôle.
- **And** les 3 plans (`pme.json`, `association.json`, `independant.json`) portent `"role": "…"` sur les entrées de la table de correspondance, et **uniquement** sur celles-ci.
- **And** `validate_chart` (`mod.rs:88-114`) rejette un plan où **un rôle singleton apparaît deux fois** — la validation existe déjà pour l'unicité des numéros et l'existence des parents ; c'est le même esprit, et ça attrape une faute de frappe dans un JSON avant qu'elle n'atteigne la DB.
- **Given** une nouvelle société créée par l'onboarding, **When** `bulk_create_from_chart` (`accounts.rs:421`) insère le plan, **Then** les rôles du JSON sont persistés **et** `postable` est calculé : `FALSE` pour toute entrée qui est parent d'une autre entrée du plan **ou** portant le rôle `CurrentYearResult`, `TRUE` sinon. Un nouveau seed doit produire **exactement** l'état qu'aurait produit le backfill de la migration sur une base pré-existante (**invariant testé**, AC-G).
- **And** `AccountRole` existe en **deux** exemplaires, conformément à la §Décision « où vit `AccountRole` » : `kesh-core::chart_of_accounts::AccountRole` (`Deserialize` seul) et `kesh-db::entities::account::AccountRole` (impls sqlx manuelles), conversion via `as_str()` / `FromStr` au seul point de contact `bulk_create_from_chart`. **Ce n'est pas une négligence** : l'orphan rule Rust interdit toute autre solution (`error[E0117]`), et un `AccountRole` unique dans `kesh-db` créerait un cycle Cargo. Un **test unitaire de cohérence** vérifie que les deux enums exposent les mêmes variantes et le même `as_str()` — c'est lui, pas la fusion, qui protège contre la dérive.

### G. Tests

**Repository (`crates/kesh-db/src/repositories/accounts.rs`, tests inline — 11 tests existants, numéros `T100`/`T200` chart-agnostiques)**

- `create` avec rôle + `postable=false` → relu correctement (round-trip `Encode`/`Decode`).
- `update` change le rôle seul → `version` bumpé + audit `account.updated` dont le `details.before/after` contient l'ancien et le nouveau rôle.
- `update` no-op (mêmes name/type/role/postable) → **pas** de bump de version, **pas** d'audit.
- Rôle singleton en double sur la même société → erreur DB mappée (pas un panic).
- Même rôle singleton sur **deux sociétés** → accepté.
- Deux comptes `EquityOther` sur la même société → accepté.
- **Cycle « le rôle singleton se libère à l'archivage »** (le test le plus important d'AC-B/AC-E) : compte A actif porte `Receivable` → archiver A → affecter `Receivable` à B actif → **accepté** → réactiver A → **refusé** avec `ACCOUNT_ROLE_ALREADY_ASSIGNED` nommant B → retirer le rôle de B → réactiver A → **accepté**.
- `reactivate` : nominal, parent archivé → refus, déjà actif → no-op sans bump, mauvaise `version` → conflit, audit `account.reactivated` écrit.
- Backfill : un compte **archivé** portant le numéro cible ne reçoit **pas** le rôle (`AND active = TRUE`).

**Plans comptables (`kesh-core/src/chart_of_accounts/mod.rs`, 14 tests existants)**

- Les 3 plans se chargent avec leurs rôles ; chaque rôle singleton apparaît **exactement une fois** par plan.
- `validate_chart` rejette un plan avec un singleton dupliqué.
- **Cohérence des TROIS sources de la liste singleton** — le garde-fou central de la story : (a) les deux `AccountRole` exposent les mêmes variantes, le même `as_str()`, un `FromStr` réciproque **et le même `is_singleton()`** ; (b) l'ensemble des singletons Rust est **identique à celui encodé dans la migration**, vérifié en lisant `SELECT GENERATION_EXPRESSION FROM information_schema.COLUMNS WHERE TABLE_NAME = 'accounts' AND COLUMN_NAME = 'singleton_role'` sur la base de test. Sans (b), les deux enums peuvent dériver de concert avec le SQL sans que rien ne le signale.

**Migration (`crates/kesh-db/tests/`)**

- `migrations_fresh_install.rs` / `migrations_upgrade_path.rs` : ces suites comptent les lignes de `accounts` (`("accounts", 4)`, `migrations_upgrade_path.rs:~228`) — **vérifier que le compte reste juste** ; cette story n'ajoute aucun compte, seulement des colonnes, donc le compteur ne doit **pas** bouger. Ajouter l'assertion que les colonnes existent et que le backfill a bien tourné sur les comptes seedés par les migrations (`1171`, `2206` insérés par `20260614000001_vat_accounts_config.sql`).
- **Invariant « seed ≡ backfill »** : une base fraîche (seed via `bulk_create_from_chart`) et une base migrée (backfill SQL) produisent le **même** `(role, postable)` pour chaque numéro des 3 plans. Filet le plus important de la story — il couvre la divergence la plus probable : deux sources de vérité indépendantes (les annotations `"role"` des 3 JSON vs la liste de numéros du SQL de backfill).
  - **Montage obligatoire — le test par défaut ne teste RIEN.** Sous `#[sqlx::test]` (mode standard du repo), la base éphémère est créée vide, **toutes** les migrations tournent, **puis** le test insère : le backfill s'exécute donc toujours sur une table `accounts` vide et ne rencontre jamais un plan comptable. Il faut le montage de `migrations_upgrade_path.rs` : `#[sqlx::test(migrations = false)]` → `apply_migrations_up_to(total - 1)` → insérer les entrées du plan **en SQL brut** (`number`, `name`, `account_type`, `parent_id`, dans l'ordre de longueur de numéro comme `bulk_create_from_chart:434-437`) → `MIGRATOR.run()` (le backfill s'exécute alors sur des données réelles) → comparer par numéro avec ce que produit le seed. **Ne pas** appeler `bulk_create_from_chart` sur ce schéma pré-migration : il bindera `role`/`postable` qui n'existent pas encore → `ERROR 1054`. Extraire le calcul de `postable` du seed en **fonction pure** testable séparément, et comparer à elle. Répéter pour les 3 plans.
  - **Anti-pattern à proscrire** : recopier l'`UPDATE` du backfill dans le test — le test validerait alors une copie de lui-même.

**Backup / export (`crates/kesh-api/tests/`)**

- `serialize_accounts_csv` (`exports/csv_tables.rs:198-231`) : l'en-tête et les lignes gagnent `role` + `postable` (12 colonnes au lieu de 10). L'en-tête est écrit **à la main** — l'oubli est silencieux.
- La colonne générée `singleton_role` **n'apparaît pas** dans le manifeste de backup ni dans le CSV export → test explicite (le filtre `EXTRA NOT LIKE '%GENERATED%'` de `backup.rs:88-96` est censé s'en charger ; le prouver plutôt que le supposer).
- Les suites `admin_backup_e2e` / `admin_full_import_e2e` restent vertes : un export → import complet round-trip les nouvelles colonnes.

**Compilation — les ~29 littéraux `NewAccount { … }` du workspace (à ne PAS découvrir au gate)**

`NewAccount` gagne deux champs ; un littéral de struct Rust ne tolère pas de champ manquant (`#[serde(default)]` n'agit que sur la désérialisation JSON, jamais sur un littéral). **29 littéraux répartis sur 15 fichiers** (le 30e hit du grep est la définition `entities/account.rs:87`) cassent `cargo build --workspace --all-targets` — le **premier** des 4 checks obligatoires. Inventaire (`grep -rn "NewAccount {" crates/ --include=*.rs`) :

- `crates/kesh-db/src/entities/account.rs` (définition), `crates/kesh-db/src/repositories/accounts.rs`, `crates/kesh-api/src/routes/accounts.rs` — couverts par T4/T5/T7 ;
- **`crates/kesh-api/tests/`** : `api_keys_e2e.rs`, `bank_accounts_e2e.rs`, `exports_global_e2e.rs`, `reconciliation_e2e.rs`, `reconciliation_manual_e2e.rs`, `reconciliation_rules_e2e.rs`, `reconciliation_split_e2e.rs`, `reports_e2e.rs`, `reports_export_e2e.rs` ;
- **`crates/kesh-db/tests/`** : `bank_accounts_repository.rs`, `kf005_fulltext_index_e2e.rs`, `reconciliation_rules_repository.rs`, `report_aggregates.rs`.

→ ajouter `role: None, postable: true` partout. **Alternative recommandée** : donner à `NewAccount` un constructeur `NewAccount::new(company_id, number, name, account_type, parent_id)` (rôle `None`, `postable` `true`) et l'utiliser dans les tests — absorbe le même churn lors des Epics suivants. Décision laissée au dev, mais **l'inventaire ci-dessus n'est pas négociable**.

**API E2E (`crates/kesh-api/tests/`)**

- **Il n'existe aujourd'hui aucun test E2E dédié aux comptes** (seul `idor_multi_tenant_e2e.rs` touche `/api/v1/accounts`). Créer `accounts_e2e.rs` : list/create/update avec rôle, **409 `ACCOUNT_ROLE_ALREADY_ASSIGNED` dont le `details` nomme effectivement le compte en conflit** (pas seulement le code HTTP), cycle archive → reactivate, refus de réactivation sous parent archivé, refus de réactivation quand le rôle singleton a été repris entre-temps, RBAC (un Lecteur ne peut ni créer ni réactiver), IDOR (compte d'une autre société → 404).

**Frontend**

- Vitest sur la page ou sur un composant extrait : rendu de la colonne Rôle, sélecteur de rôle, bouton Réactiver visible **uniquement** sur les lignes archivées.
- Playwright `frontend/tests/e2e/accounts.spec.ts` (116 l. existantes) : étendre avec (a) affectation d'un rôle via le dialog Modifier, (b) cycle archiver → afficher les archivés → réactiver → le compte réapparaît dans la liste par défaut. Conserver la convention `data-testid` existante (`account-row-{number}-…`).

**Gate**

- Backend : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (ou `DATABASE_URL=…:3307 scripts/test-fast.sh` sur la DB tmpfs, ~3 min).
- Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- E2E Playwright (la story touche le frontend) : `npm run test:e2e` avec `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`.

### H. Interface — page Plan comptable

- **Given** la page `/accounts`, **When** je consulte la liste, **Then** une colonne **Rôle** affiche le libellé traduit du rôle (ou un tiret si aucun), et un indicateur signale les comptes **non-postables**.
- **And** les dialogs Créer et Modifier exposent un sélecteur **Rôle** (liste des 10 rôles + « Aucun ») et une case **Postable**.
- **And** l'indicateur « non-postable » porte une mention explicite de son caractère **indicatif** tant que 14-3b n'est pas livrée (tooltip / texte d'aide : « Indicatif — la saisie d'écriture ne le bloque pas encore »). **Rationale** : sans cette mention, l'UI promet une protection qui n'existe pas (le seul garde actif à la saisie reste `active = TRUE`, `journal_entries.rs:152-156`), et un utilisateur pourrait poster sur le compte de résultat en croyant en être empêché. Mieux vaut une limitation affichée qu'une fausse sécurité (limitation **L2**).
- **And** une erreur 409 `ACCOUNT_ROLE_ALREADY_ASSIGNED` affiche un message **nommant le compte déjà porteur du rôle** (numéro + nom, depuis le `details` de la réponse), pas un « Erreur inattendue » ni un « Ressource déjà existante » générique.
- **And** les lignes archivées affichent un bouton **Réactiver** (cf. AC-E).
- **And** une ligne **archivée qui porte encore un rôle** affiche ce rôle de façon visiblement inactive (grisé/barré + mention `account-role-archived-hint`), pour que l'utilisateur qui active le toggle « afficher les archivés » comprenne pourquoi **deux** comptes semblent porter le même rôle. Rappel : `update` refuse tout compte archivé (`accounts.rs:186-192`), l'utilisateur ne peut donc pas retirer le rôle du compte archivé — seulement agir sur le porteur actif ou réactiver.
- **And** la page est **entièrement internationalisée**. Aujourd'hui elle code le français en dur (`ACCOUNT_TYPES`/`TYPE_LABELS` `+page.svelte:18-25`, messages de validation `:117-121` / `:164-167`, messages d'erreur `:142` / `:188` / `:217`) **alors que les clés FTL existent déjà et ne sont utilisées par personne** (`accounts-title`, `accounts-add`, `accounts-edit`, `accounts-archive`, `account-field-*`, `account-type-*`, `account-archived-label` — `fr-CH/messages.ftl:146-159`). Migrer la page vers `i18nMsg` plutôt que d'y ajouter une nouvelle couche de français en dur. La page vit sous `src/routes/`, donc **hors périmètre** de `lint-i18n-ownership` (qui ne parcourt que `src/lib/features/**`, `scripts/lint-i18n-ownership.js`) : les clés singulier `account-*` existantes restent utilisables telles quelles.
- **And** nouvelles clés dans **les 4 locales** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`), alignées **clé pour clé** : `account-field-role`, `account-role-none`, `account-role-<slug>` × 10 (calqué sur `account-type-<slug>:155-158`), `account-field-postable`, `account-postable-no`, **`account-postable-hint`** (la mention « Indicatif » ci-dessus), **`account-role-archived-hint`**, `accounts-reactivate`, **`accounts-reactivate-aria`** (les `aria-label` de la page sont aujourd'hui du français en dur, `+page.svelte:288`/`:291`), `accounts-reactivated`, `accounts-role-conflict`. **Plus** les clés nécessaires à la migration i18n complète de la page : messages de validation (`+page.svelte:117-121`, `:164-167`) et messages d'erreur (`:142`, `:188`, `:217`) — en réutilisant les clés FTL existantes quand elles conviennent, en en créant sinon. Faire l'inventaire **avant** de coder : une clé oubliée = un littéral français de plus, soit exactement ce que cet AC corrige. Attention : `fr-CH` fait 1330 lignes contre 1238 pour les 3 autres — **vérifier l'écart avant d'insérer**, ne pas se fier au numéro de ligne d'une locale pour les autres.
- **And** Svelte 5 runes (`$state`, `$derived`, `$props`), `data-testid` sur chaque élément interactif, `aria-label` sur les boutons icône, gestion `OPTIMISTIC_LOCK_CONFLICT` et `RESOURCE_CONFLICT` explicite — conventions de `RuleFormModal.svelte`.

### I. Non-régression — le comportement métier ne change pas

- **Given** cette story livrée, **When** j'utilise Kesh normalement (saisie d'écritures, factures, rapports, réconciliation), **Then** **rien ne change** : `postable` n'est lu par aucun code métier, les fonds propres du bilan restent rendus comme en 14-1, `company_invoice_settings` cherche toujours ses comptes par numéro.
- **And** les 1900 tests backend et 418 tests frontend de la baseline 14-1 restent verts.

## Tasks / Subtasks

- [ ] **T1** `AccountRole` en **deux** exemplaires (`kesh-core` `Deserialize` seul + `kesh-db` avec impls sqlx), conversion via `as_str()`/`FromStr` dans `bulk_create_from_chart`, + test de cohérence des deux enums. **Décision déjà tranchée** (§Décision « où vit `AccountRole` », orphan rule) — ne pas la rouvrir — AC-C/F.
- [ ] **T2** Migration `2026MMDD000001_accounts_role_postable.sql` : `role` + CHECK, `postable`, `singleton_role` généré + UNIQUE, backfill rôles `WHERE role IS NULL`, backfill `postable` (enfants puis `CurrentYearResult`). En-tête commentée au format des migrations récentes (justification métier, bloc « Idempotence », mention non-breaking / pas de bump) — AC-A/B.
- [ ] **T3** `docs/migrations-idempotence-audit.md` : ligne du tableau + section « Statistiques » (54→55, tracked-by-sqlx 43→44) — AC-A.
- [ ] **T4** `entities/account.rs` : enum `AccountRole` (pattern `AccountType:11-66`) + `is_singleton()` + champs sur `Account` / `NewAccount` / `AccountUpdate` — AC-C.
- [ ] **T5** *(pense aussi aux 7 littéraux `AccountUpdate { … }` — `routes/accounts.rs:183` + `repositories/accounts.rs` ×6 — tous dans des fichiers de cette tâche, mais ils cassent à la compilation)* `repositories/accounts.rs` : `COLUMNS:17` **et** `FIND_BY_ID_SQL:19`, `account_snapshot_json:27-38`, `is_no_op_change:164-166`, `create:41`, `update:170` (UPDATE SQL + binds), `bulk_create:354`, `bulk_create_from_chart:421` (rôles du chart + calcul de `postable`) — AC-C/F.
- [ ] **T6** `repositories/accounts.rs` : `reactivate(pool, id, version, user_id)` — garde parent archivé, no-op si déjà actif, verrou optimiste, audit `account.reactivated`, calqué sur `archive:270` (garde `:278-290`) — AC-E.
- [ ] **T7** `routes/accounts.rs` : DTO (`AccountResponse`, `CreateAccountRequest`, `UpdateAccountRequest`) + `ReactivateAccountRequest` + handler + route `PUT /{id}/reactivate` dans `comptable_routes` (`lib.rs:282-284`). **+ variant `AppError::AccountRoleAlreadyAssigned` dédié** (modèle `IdeAlreadyExists`, `kesh-api/src/errors.rs:190-196`) avec `details: { accountId, accountNumber, accountName }`, alimenté par un `SELECT` préalable du porteur du rôle ; le `1062` reste le filet de sécurité — AC-D/E.
- [ ] **T8** `kesh-core/chart_of_accounts` : `AccountRole` (`Deserialize` seul) **+ son propre `is_singleton()`** (`validate_chart:88` est privé dans ce crate), `ChartEntry.role` (`#[serde(default)]`), validation singleton dans `validate_chart`, annotation `role` des 3 JSON — AC-F.
- [ ] **T9** `exports/csv_tables.rs:198-231` : 2 colonnes de plus dans l'en-tête **et** les lignes — AC-G.
- [ ] **T10** i18n × 4 locales (nouvelles clés) — AC-H.
- [ ] **T11** Frontend : `accounts.types.ts` (`AccountRole`, champs sur `AccountResponse`/`Create`/`Update`), `accounts.api.ts` (`reactivateAccount`), page `/accounts/+page.svelte` (colonne Rôle, sélecteurs dialogs, case Postable, bouton Réactiver, migration i18n complète) — AC-E/H.
- [ ] **T12** Tests : repo inline, chart, migrations (dont invariant « seed ≡ backfill »), `accounts_e2e.rs` (nouveau), CSV export, Vitest, Playwright `accounts.spec.ts` — AC-G.
- [ ] **T13** Mettre à jour les **29 littéraux `NewAccount { … }`** (15 fichiers, dont les 13 de test listés en AC-G) + les **7 littéraux `AccountUpdate { … }`** (`role: None, postable: true`, ou constructeur dédié). À faire **tôt**, pas au moment du gate — AC-G.
- [ ] **T14** Doc : `CHANGELOG.md` (entrée [Non publié]) + `README.md` si la feuille de route bouge + **manuel utilisateur FR** `docs/manual/fr/user-manual.tex` section Plan comptable (colonne Rôle, case Postable, bouton Réactiver = features **visibles utilisateur**, donc gate doc-sync CLAUDE.md « stories qui ajoutent/modifient des features visibles utilisateur ») + régénérer le PDF (`make fr` dans `docs/manual/`) si le `.tex` change. Conclure explicitement « aucun changement requis » après lecture si tel est le cas — ne pas sauter l'étape en silence. Puis gate complet backend + frontend + E2E — AC-G.

## Dev Notes

### Pièges, par ordre de coût

1. **`FIND_BY_ID_SQL` duplique la liste de colonnes** (`accounts.rs:19`) au lieu de réutiliser `COLUMNS:17`. Ajouter les champs à `Account` sans mettre à jour **les deux** produit une erreur `FromRow` **au runtime seulement** — la compilation passe. C'est le piège n°1 de cette story.
2. **`is_no_op_change`** (`accounts.rs:164-166`) ne compare aujourd'hui que `name` et `account_type`. Non étendu, un `PUT` qui ne change **que** le rôle est silencieusement ignoré et retourne 200 avec l'ancienne valeur. Bug utilisateur invisible.
3. **La liste des rôles singleton existe à TROIS endroits**, pas deux : (a) le `CASE WHEN active AND role IN (…)` de la colonne générée — SQL, figé par la migration ; (b) `kesh-db::AccountRole::is_singleton()` ; (c) `kesh-core::AccountRole::is_singleton()`, indispensable parce que `validate_chart` est **privé dans `kesh-core`** (`chart_of_accounts/mod.rs:88`) et que `kesh-core` ne peut pas atteindre `kesh-db`. Les trois **doivent** rester synchrones : commentaire croisé dans les trois fichiers + test de cohérence à trois branches, **dont la lecture de l'expression SQL réelle** (`information_schema.COLUMNS.GENERATION_EXPRESSION`). Une divergence donne soit un 409 inexpliqué, soit une unicité non appliquée.
4. **`chk_accounts_role` est une liste fermée** : ajouter un rôle plus tard = migration **sur une contrainte déjà en production** (potentiellement sur le NAS de Guy). C'est volontaire (cohérent avec `chk_accounts_type:20` et le `FromStr` strict de l'enum), mais c'est aussi pourquoi le vocabulaire des 10 rôles a été arrêté **avant** le dev, `EquityOther` compris. Ne pas « assouplir » en retirant le CHECK, ne pas ajouter de rôle en cours de route sans repasser par une décision explicite.
5. **`AccountRole` est volontairement dupliqué** `kesh-core` / `kesh-db` — orphan rule, cf. §Décision dédiée. Ne pas « corriger » cette duplication, ni celle de `AccountType` (`chart_of_accounts/mod.rs:16-21` vs `entities/account.rs:11-17`) qui a exactement la même cause. Le garde-fou est le test de cohérence, pas la fusion.
6. **La migration ne peut pas être `ALGORITHM=INSTANT`** (l'`ADD CONSTRAINT UNIQUE` construit un index ; `ERROR 1845` vérifié sur la variante `STORED`). Ne pas copier-coller le pattern de `20260531000001_bank_accounts_archived.sql:15-17` sans réfléchir : il échouerait. Et **ne pas réinventer** la colonne générée : copier `20260513000001_reconciliation_rules.sql:54` + son commentaire `:20-28`.
7. **Un seul backfill de rôles pour les 3 plans**, mais **pas** parce que « les 3 plans ont les mêmes numéros » : c'est vrai pour 9 rôles sur 10, faux pour `EquityOther` (2900 en PME, 2850/2860 ailleurs — ensembles disjoints). Un unique `WHERE number IN ('2900','2850','2860')` couvre les trois, précisément parce que `EquityOther` est multi-valué.
8. **Le `1062` générique ne suffit pas** pour AC-D/AC-H : `errors.rs:166` → `UniqueConstraintViolation` → `kesh-api/src/errors.rs:2001` → message **fixe** « Ressource déjà existante », le détail MySQL n'étant que logué. Il faut le variant dédié + le `SELECT` préalable. Croire que « le mapping existe déjà » fait rater l'AC.
9. **La page `/accounts` code le français en dur** alors que les clés FTL existent, inutilisées. Ne pas empiler une nouvelle couche de littéraux français — AC-H.

### Contrats backend (ground-truth vérifié par lecture directe, 2026-07-22)

- **Table** `crates/kesh-db/migrations/20260411000001_accounts.sql:6-23` — `number VARCHAR(10)`, `account_type VARCHAR(20)`, `parent_id` auto-réf RESTRICT, `active BOOLEAN DEFAULT TRUE`, `version INT DEFAULT 1`, `uq_accounts_company_number:19`, `chk_accounts_type:20` (modèle du CHECK `BINARY`). **Aucun ALTER ultérieur** : le schéma est celui d'origine. **Aucun index secondaire** hors UNIQUE et FK.
- **Entity** `crates/kesh-db/src/entities/account.rs` — `AccountType:11-17`, `as_str:19-28`, `FromStr:30-41`, `Type<MySql>:42-50`, `Encode:52-59`, `Decode:61-66`, `Account:69-82` (`#[serde(rename_all="camelCase")]` + `sqlx::FromRow`), `NewAccount:87-93`, `AccountUpdate:99-102` (commentaire `:96` « Le numéro n'est PAS modifiable après création »).
- **Repo** `crates/kesh-db/src/repositories/accounts.rs` — `COLUMNS:17`, `FIND_BY_ID_SQL:19`, `account_snapshot_json:27-38`, `create:41`, `find_by_id:98`, `find_by_id_in_company:108`, `count_by_company:127`, `list_by_company:138` (filtre `active = TRUE` si `include_archived=false`, `:153`), `is_no_op_change:164-166`, `update:170` (rejet compte archivé `:186-192`, no-op `:209-212`, `UPDATE … SET name = ?, account_type = ?, version = version + 1 WHERE id = ? AND version = ? AND active = TRUE` `:214-217`, audit `account.updated`), `archive:270` (garde `:278-290`) (garde enfants actifs `:278-290`, audit `account.archived`), `bulk_create:354`, `bulk_create_from_chart:421` (tri topologique par longueur de numéro puis numéro `:434-437`, résolution `parent_number → parent_id`, **pas d'audit** — contexte système), `delete_all_by_company:497`. Tests inline `:521-1100` (11 tests, helpers `test_pool:528`, `cleanup_test_accounts:553` purge `number LIKE 'T%'`).
- **Routes** `crates/kesh-api/src/routes/accounts.rs` — `ListAccountsQuery:23`, `CreateAccountRequest:30`, `UpdateAccountRequest:39`, `ArchiveAccountRequest:47`, `AccountResponse:53-64` + `From<Account>:66-81`, `list_accounts:89`, `create_account:108` (validation `:115-145` : trim, `number` ≤ 10, `name` ≤ 255, parent existant **et actif**), `update_account:164`, `archive_account:195`. Câblage `crates/kesh-api/src/lib.rs:276` (`comptable_routes`), `:278-280`, `:282-284`, `:566-568`, `:572`.
- **Chart** `crates/kesh-core/src/chart_of_accounts/mod.rs` — `AccountType:16-21` (dupliqué), `ChartEntry:38-46` (`#[serde(rename_all="camelCase")]`, `#[serde(rename="type")] account_type`, `parent_number: Option<String>`), `resolve_name:49-57` (fallback `fr` puis numéro), `include_str!:60-62`, `load_chart:71-85` (`"pme"|"association"|"independant"`), `validate_chart:88-114` (unicité numéros + parents existants), tests `:116-317` (14 tests). Plans : `crates/kesh-core/assets/charts/{pme,association,independant}.json` — 84 / 81 / 84 entrées, une entrée par ligne.
- **Seed / onboarding** — `crates/kesh-api/src/routes/onboarding.rs:196`, `:205`, `:577`, `:590`, `:706`, `:713-729` ; démo `crates/kesh-seed/src/lib.rs:142-155` puis `:189-211`.
- **Saisie d'écriture (à NE PAS toucher ici — 14-3b)** — `crates/kesh-db/src/repositories/journal_entries.rs:146-167` (create) et `:696-718` (update) : `SELECT id FROM accounts WHERE company_id = ? AND active = TRUE AND id IN (…)` puis comparaison de cardinalité → `DbError::InactiveOrInvalidAccounts`. Deux copies à factoriser en 14-3b.
- **Config facturation (à NE PAS toucher ici — 14-3b)** — `crates/kesh-db/src/repositories/company_invoice_settings.rs:275`/`:284`/`:296` (variante pool) et `:396`/`:405`/`:416` (variante tx, marquée `MIRROR`), fail-fast `:308-311` / `:428-430`.
- **Backup** `crates/kesh-db/src/backup.rs` — `TABLES_TO_TRUNCATE:58` (`accounts`, FK self-réf), `TableExport.column_names:80` (doc « hors colonnes générées » `:77`), `non_generated_columns:88-96` (`EXTRA NOT LIKE '%GENERATED%'`), `export_table:120`, `restore_tables_in_tx:391` (INSERT paramétrés depuis les colonnes du manifeste). → **la colonne générée est déjà gérée**, ne rien y modifier.
- **Export CSV global** `crates/kesh-api/src/exports/csv_tables.rs:198-231` — en-tête écrit **à la main** (10 colonnes) + `write_record` ligne à ligne. C'est **là** qu'il faut ajouter les 2 colonnes.
- **i18n** `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — format Fluent, clés kebab-case préfixées par domaine. Comptes : `accounts-title:146` → `account-archived-label:159` (dont `account-type-asset|liability|revenue|expense:155-158`), `nav-accounts:75`, `error-inactive-accounts:205`. `fr-CH` = 1330 lignes, les 3 autres = 1238.

### Contrats frontend (ground-truth vérifié, 2026-07-22)

- `frontend/src/lib/features/accounts/accounts.types.ts` — `AccountType:1`, `AccountResponse:3-15`, `CreateAccountRequest:17-22`, `UpdateAccountRequest:24-28`, `ArchiveAccountRequest:30-32`. Aucun composant dans ce dossier.
- `frontend/src/lib/features/accounts/accounts.api.ts` — `fetchAccounts:9`, `createAccount:15`, `updateAccount:19`, `archiveAccount:26`. **Pas de `reactivateAccount`**.
- `frontend/src/routes/(app)/accounts/+page.svelte` (423 l.) — page unique (liste + 3 dialogs). `ACCOUNT_TYPES:18`, `TYPE_LABELS:20-25` (français en dur), arborescence `treeAccounts:59-84` + indentation `:274`, toggle archivés `:252`, `createValidation:117-121`, mapping `RESOURCE_CONFLICT:142`, `editValidation:164-167`, `OPTIMISTIC_LOCK_CONFLICT:188`/`:217`, `canModify:231-234` (Admin ou Comptable), ligne archivée `:273`/`:284`, bouton archiver conditionné `{#if canModify() && account.active}` `:287`, dialogs `:311-345` (créer), `:369-389` (modifier, numéro `disabled`), `:406-423` (archiver).
- **Précédent réactivation** — `frontend/src/lib/features/projects/projects.api.ts:40` (`unarchiveProject`), `frontend/src/routes/(app)/settings/projects/+page.svelte:135` / `:273` / `:297`, clés `projects-unarchive` / `projects-unarchived`. Autre : `reconciliation-rules-actions-reactivate`.
- **i18n runtime** — `frontend/src/lib/shared/utils/i18n.svelte.ts` (`i18nMsg(key, fallback, args?)`, store runes, `GET /api/v1/i18n/messages`). Import **obligatoirement** depuis ce module. `frontend/scripts/lint-i18n-ownership.js:16` — ne parcourt que `src/lib/features/**`, namespaces globaux `error|tooltip|common|mode|shortcut|demo`.
- **Style de référence pour un formulaire** — `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte` : `interface Props` + `$props()`, `$state`, `$derived`, callbacks en props (`onSuccess`/`onCancel`), `version` dans chaque mutation, `isApiError` de `$lib/shared/utils/api-client`, toasts `svelte-sonner` sur la page / `errorMsg` local dans les modals.
- **E2E** `frontend/tests/e2e/accounts.spec.ts` (116 l.) — seed `with-company`, `loginAndGoToAccounts`, `data-testid` : `account-row-{number}`, `-number`, `-name`, `-type-badge`, `-edit-button`, `-archive-button`, `account-create-button`, `account-show-archived-toggle`, `account-{create,edit,archive}-dialog-{submit,cancel,confirm}`. Couvre titre `:33`, arborescence `:38`, badges `:46`, compteur `:54`, création `:61`, modification `:87`, toggle archivés `:110`. **Pas de test d'archivage effectif ni de réactivation.**

### Conventions de migration (d'après les 3 dernières)

- Nommage `YYYYMMDDHHMMSS_snake_case.sql` ; dernière du repo : `20260715000001_invoice_reminders.sql`.
- En-tête de commentaire long **obligatoire** : story + issue, justification métier de chaque colonne, décision FK/index, bloc explicite « Idempotence (docs/migrations-idempotence-audit.md) : … », mention non-breaking / bump `min_required`.
- Convention majoritaire : **pas de `IF NOT EXISTS`** (verdict `tracked-by-sqlx`, 43 des 54 migrations).
- Justifier explicitement l'absence d'index quand c'est un choix (YAGNI), modèle : `20260531000001_bank_accounts_archived.sql:7-13`.
- **Pas de bump `kesh_version_min_required`** ici (P2/P3) — seul bump du repo à ce jour : `20260714000002_email_templates_reminder.sql` (`'0.7.0'`).

### Leçons de review à appliquer dès le dev

- **Un patch = un test** (`feedback_review_patch_needs_test`) — chaque correction de review vient avec sa régression testée, sinon la remédiation devient la source des findings suivants.
- **Fix structurel > incrémental** — l'unicité des rôles singleton est garantie par le schéma (colonne générée + UNIQUE), pas par un `SELECT` applicatif. Même esprit que la garde `create_in_tx` de 14-1.
- **Grep ground-truth obligatoire** sur tout finding CRITICAL/HIGH affirmant l'absence d'un patch ou la présence d'un anti-pattern (`grep -nF`), en particulier avec un reviewer Haiku.

### Références

- **Story 14-1** `_bmad-output/implementation-artifacts/14-1-cloture-report-des-soldes.md` — modèle temps réel virtuel, retrait du hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS`, renvoi explicite du durcissement à 14-3 (`balance_sheet.rs:30-38`).
- **Story 14-3b** (suivante) — consommateurs des rôles.
- **Story 14-4** — bilan d'ouverture, dépend de cette story.
- **Issue #269** — réactiver un compte archivé.
- Epic 14 « Clôture d'Exercice » (`_bmad-output/planning-artifacts/epics.md:1316-1332`, ex-Epic 13, FR60/FR61/FR62).
- Politiques CLAUDE.md : `## Migration breaking policy` (P1-P5), `## Règle de splitting préventif`, `## Review Iteration Rule`, `## Test Locally First`.
- Cartographie : 2 agents Explore (backend + frontend) + vérification directe des ancres, 2026-07-22. Contrainte d'unicité validée empiriquement sur `kesh-mariadb`.

## Change Log — validate

### Pass 2 (Opus, contexte frais, 2026-07-22) — 0 CRITICAL, 2 HIGH + 6 MEDIUM + 4 LOW → tous patchés

- **HIGH — régression directe du patch CRITICAL de la Pass 1.** La décision « deux enums `AccountRole` » a été propagée dans le Contexte, AC-C, AC-F et T1, **mais pas** à `is_singleton()` : celui-ci n'était posé que sur l'enum `kesh-db`, alors que `validate_chart` est **privé dans `kesh-core`** (`chart_of_accounts/mod.rs:88`) et que `kesh-core` ne peut pas atteindre `kesh-db`. Le dev aurait recopié la liste des singletons → **3e source de vérité** non déclarée, non couverte par le test de cohérence. **Résolu** : `is_singleton()` sur les deux enums (T8), piège 3 réécrit « TROIS endroits », test de cohérence étendu **et** comparé à l'expression SQL réelle (`information_schema.COLUMNS.GENERATION_EXPRESSION`) — le seul garde-fou qui ferme les trois. *Illustration du carry-forward « la remédiation est la source des findings suivants ».*
- **HIGH** — angle mort de l'inventaire de blast radius : `crates/kesh-db/tests/migrations_upgrade_path.rs:77` porte `assert_eq!(total, 54)` → **rouge garanti** dès la création de la migration ; et `:105` `total - 20` doit devenir `total - 21`, faute de quoi la **frontière des 23 migrations historiques** (commentaire `:90-104`) glisse et le test **continue de passer en testant autre chose** (échec silencieux, pire qu'un rouge). **Résolu** : AC-A + T2.
- **MEDIUM** — l'invariant « seed ≡ backfill », présenté comme le filet principal, **n'était pas exécutable** : sous `#[sqlx::test]` la base est vide quand les migrations tournent, le backfill ne rencontre donc jamais de plan comptable ; et `bulk_create_from_chart` ne peut pas tourner sur un schéma pré-migration (`ERROR 1054`). **Résolu** : montage complet spécifié (`migrations = false` + `apply_migrations_up_to(total - 1)` + insertion SQL brute + `MIGRATOR.run()`), + anti-pattern « recopier l'UPDATE dans le test » explicitement proscrit.
- **MEDIUM** — sémantique du `PUT` non tranchée : avec un `Option` laxiste, corriger la casse d'un libellé **effaçait silencieusement le rôle**. **Résolu** : `role`/`postable` **requis** (cohérent avec `name`/`accountType` déjà requis — c'est un *full-replace*), perte de donnée remplacée par un 400 explicite, + 3 tests E2E + mention CHANGELOG.
- **MEDIUM** — `STORED` choisi sans justification alors que **le repo a déjà ce pattern en `VIRTUAL`** : `20260513000001_reconciliation_rules.sql:54` `active_uniq … GENERATED ALWAYS AS (IF(active, match_value, NULL)) VIRTUAL` + UNIQUE, documenté `:20-28` comme le « Workaround Option A » au UNIQUE partiel absent de MariaDB — mécanique identique à la nôtre, annulation sur `active` comprise. **Résolu** : aligné sur `VIRTUAL` + précédent cité.
- **MEDIUM** — l'invariant « un rôle singleton n'est unique que **parmi les actifs** » (corollaire du patch `active AND` de la Pass 1) n'était transmis ni à 14-3b ni à l'UI. **Résolu** : §Point d'attention légué (1) « tout lookup par rôle DOIT porter `AND active = TRUE` » + décision explicite « `archive()` ne remet pas `role` à `NULL` » + AC-H (affichage inactif du rôle sur une ligne archivée).
- **MEDIUM** — la liste de clés i18n ne couvrait pas ce qu'AC-H exige (mention « Indicatif », messages de validation/erreur à migrer, `aria-label` en français en dur `+page.svelte:288`/`:291`) → liste complétée + inventaire préalable imposé. **MEDIUM** — backfill par numéro faux et silencieux sur un plan renuméroté (contredit localement le principe fondateur) → qualifié **best effort** + avertissement CHANGELOG et manuel.
- **LOW** — pièges renumérotés (deux « 8 ») ; verbe `PUT /reactivate` vs `POST /unarchive` des projets justifié ; décompte `NewAccount` précisé (29 littéraux / 15 fichiers, le 30e hit étant la définition) ; 7 littéraux `AccountUpdate { … }` ajoutés à T13.
- **Vérifié sain par le reviewer (tests exécutés, non supposés)** : faisabilité SQL complète rejouée sur MariaDB 11.3.2 y compris le backfill `postable` auto-référentiel (**pas d'`ERROR 1093`**) ; coexistence archivé+actif de même rôle ; `backup.rs:88-96` attrape bien `STORED GENERATED` **et** `VIRTUAL GENERATED` ; **un backup pris AVANT la migration reste importable** (`check_schema_compat` n'exige que les colonnes `is_required()`, or `role` est nullable et `postable` a un défaut) ; `restore_body` fait `DELETE FROM accounts` avant INSERT, aucun scénario d'échec du restore trouvé ; compteurs d'audit 54/43 exacts ; `sample_account()` (`csv_tables.rs:944`) est le seul littéral `Account { … }` du workspace ; les 24 INSERT SQL bruts de `accounts` sont tous en colonnes explicites ; ancres backend/frontend re-sondées à l'aveugle, exactes.
- **Trend** : Pass 1 (1 CRIT + 4 HIGH + 6 MED) → Pass 2 (0 CRIT, 2 HIGH + 6 MED) → patchés. **Pass 3 requise**, LLM différent (Haiku), contexte frais, diff aplati.

### Pass 1 (Sonnet ×2, contexte frais : « ancres & faisabilité » + « métier comptable & AC », 2026-07-22) — 1 CRITICAL + 4 HIGH + 6 MEDIUM + 5 LOW → tous patchés

- **CRITICAL** (ancres) — « `AccountRole` partagé, pas dupliqué » est **impossible à compiler** : `kesh-db` dépend de `kesh-core` (`kesh-db/Cargo.toml:8`) et `sqlx` n'existe que dans `kesh-db` (`:16`) → impls `Type<MySql>`/`Encode`/`Decode` sur un type de `kesh-core` = **`error[E0117]`** (orphan rule, prouvé par compilation d'un mini-workspace) ; l'inverse = cycle Cargo. **Résolu** : §Décision dédiée — duplication **contrôlée** des deux enums + test de cohérence. Corollaire acté : la duplication existante de `AccountType` n'est **pas** une dette mais la même contrainte structurelle. Vérité-terrain reconfirmée par l'orchestrateur (`grep -n "kesh-core\|sqlx" crates/*/Cargo.toml`).
- **HIGH** (métier) — un compte **archivé** squattait son rôle singleton à vie : le remplaçant actif ne pouvait jamais le recevoir (409 permanent causé par un compte mort), régression sur `company_invoice_settings.rs:275` qui filtre déjà `AND active = true`. **Résolu** : `CASE WHEN active AND role IN (…)` dans la colonne générée + backfill `AND active = TRUE` + AC-E (réactivation refusée en 409 explicite si le rôle a été repris) + test de cycle complet en AC-G. Sémantique **vérifiée empiriquement** sur `kesh-mariadb` : archivage → rôle libéré, reprise par un actif → OK, réactivation du squatteur → `ERROR 1062` propre.
- **HIGH** (ancres) — blast radius de compilation incomplet : **29 littéraux `NewAccount { … }` sur 16 fichiers** (13 hors périmètre annoncé) auraient cassé `cargo build --workspace --all-targets`, le **premier** des 4 checks du gate. **Résolu** : inventaire exhaustif en AC-G + tâche T13 dédiée.
- **HIGH** (ancres) — AC-D et AC-H étaient **incompatibles** : le mapping générique `1062 → RESOURCE_CONFLICT` (`kesh-db/src/errors.rs:166` → `kesh-api/src/errors.rs:2001`) renvoie un message **fixe** et **jette** le détail MySQL, il ne peut donc pas « nommer le compte en conflit ». **Résolu** : variant dédié `AccountRoleAlreadyAssigned` + `SELECT` préalable + `details`, sur le modèle de `IdeAlreadyExists` (`kesh-api/src/errors.rs:190-196`, dont le doc-comment prévoit explicitement ce cas) ; la contrainte DB reste la source de vérité.
- **HIGH** (métier) — `EquityReserve` mélangeait des natures comptables opposées : 2850/2860 = « fonds affectés/libres » en association mais « prélèvements/apports privés » chez l'indépendant, dont la présentation sous un intitulé « Réserves » en 14-3b serait un contresens — sur une liste `CHECK` **fermée**, donc coûteuse à corriger après coup. **Résolu** : renommé `EquityOther` (neutre), sémantique fine portée par le nom du compte et l'i18n ; effet de bord bénéfique — le backfill reste indépendant de `org_type`.
- **MEDIUM** — affirmation fausse « les 3 plans utilisent les mêmes numéros pour les 10 rôles » (vrai pour 9, faux pour `EquityOther` : 2900 en PME xor 2850+2860 ailleurs) → §Homogénéité réelle + piège 7 réécrits ; indicateur « non-postable » affiché sans effet pendant la fenêtre 14-3a→14-3b → micro-copie explicite obligatoire (AC-H, limitation L2) ; risque de collision d'affichage entre le solde réel de 2970 et la ligne calculée « Résultat reporté » → §Point d'attention légué à 14-3b (arithmétique vérifiée saine, problème de lisibilité) ; manuel utilisateur LaTeX absent de la doc-sync → T14 ; limitation « singleton mono-valué » non tracée → **L1** catégorie B ; T1 « à décider en dev » → supprimé (tranché par la §Décision).
- **LOW** — ancres décalées corrigées après re-vérification directe (`accounts.rs` no-op :209-212, UPDATE :214-217, garde archive :278-290 ; `backup.rs` `column_names` :80 / `restore_tables_in_tx` :391 ; `accounts.api.ts` 15/19/26 ; 14 tests chart et non 12) ; garantie anti-collision du backfill reformulée sur le vrai invariant (un rôle singleton = un seul numéro) ; ordre des deux backfills `postable` clarifié.
- **Validé positivement par les deux reviewers** : faisabilité MariaDB de bout en bout (ALTER + colonne générée + UNIQUE + backfills dans le même script ; `ALGORITHM=INSTANT` refusé comme annoncé) ; non-breaking confirmé (**aucun `SELECT *` sur `accounts` dans tout le workspace**) → pas de bump `min_required` ; `Option<AccountRole>` en `FromRow` a un précédent exact (`OnboardingState.ui_mode`) ; compteurs de `docs/migrations-idempotence-audit.md` (54→55, 43→44) exacts ; `backup.rs` gère nativement la colonne générée ; ~95 % des ~150 ancres exactes au caractère près ; l'équation du bilan de 14-1 n'est **pas** cassée par un `RetainedEarnings` postable (calcul chiffré à l'appui) ; AC-E répond entièrement à #269 ; découpage 14-3a/14-3b conforme au pattern CLAUDE.md.
- **Trend** : Pass 1 (1 CRIT + 4 HIGH + 6 MED) → patchés. **Pass 2 requise** (CRITICAL et HIGH trouvés), LLM différent, contexte frais.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
