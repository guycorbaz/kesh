# Story 14.4 : Bilan d'ouverture — saisie des soldes de départ (migration)

## Status

ready-for-dev

## Story

**As a** comptable / fiduciaire / indépendant qui **migre sa comptabilité vers Kesh** depuis un autre logiciel,
**I want** un **écran dédié pour saisir les soldes de départ** de mes comptes de bilan (actifs, passifs, capitaux propres, dont le report à-nouveau accumulé), qui génère **une écriture d'ouverture équilibrée** datée au 1er jour de mon premier exercice,
**so that** mon **bilan soit juste dès le premier jour** dans Kesh (report à-nouveau d'ouverture matérialisé) sans devoir composer manuellement une écriture OD ligne à ligne, tout en respectant le modèle temps réel virtuel (14-1) et les rôles de comptes explicites (14-3).

## Contexte

### Ce qui existe (14-1, 14-3, et antérieur)

- **Modèle temps réel virtuel (14-1)** : aucun snapshot, aucune écriture de clôture. Le bilan est **calculé par accumulation** de toutes les écritures `entry_date ≤ date d'arrêté`, tous exercices confondus (`kesh-report/src/balance_sheet.rs:275-313`, `fetch_cumulative_section`, **aucun** filtre `fiscal_year_id`). **Conséquence directe pour 14-4** : une écriture OD datée `fy_start` du 1er exercice est **incluse nativement** dans les soldes de bilan — **aucune modification du moteur de bilan n'est nécessaire**.
- **14-1 anticipe explicitement 14-4** (`14-1:50`) : *« ❌ Bilan d'ouverture / soldes de départ éditables : Story 14-4 (dépend de 14-3). En v1 de 14-1, un migrant peut poser une écriture OD sur ses comptes de capitaux propres réels — mais l'écran dédié = 14-4. »* 14-4 **est** cet écran dédié.
- **Rôles de comptes (14-3a/b/c)** : le compte physique **RetainedEarnings** (rôle `RetainedEarnings`, numéro conventionnel 2970 dans les 3 plans) **reste `postable` VOLONTAIREMENT pour ce persona migrant** — documenté à 3 endroits, dont la migration `20260722000001_accounts_role_postable.sql:173-174` qui cite nommément *« persona de la Story 14-4 »* et `kesh-core/src/chart_of_accounts/mod.rs:308-309`. Seul `CurrentYearResult` (2979) est **non-postable** (c'est une ligne calculée, jamais postée).
- **Distinction D1 de 14-3c** : au bilan, le **compte physique RetainedEarnings** (report d'ouverture migrant, itemisé dans la section Capitaux propres) est **distinct** de la ligne CALCULÉE « Résultat reporté (calculé) » (cumul du P&L des exercices Kesh antérieurs). Pour un migrant frais, la ligne calculée = **0** (aucun exercice Kesh antérieur) et son report accumulé apparaît sur le compte physique 2970. **14-4 alimente exactement ce compte physique** — la distinction 14-3c fonctionne sans changement.
- **Création d'écriture** : `journal_entries::create(pool, fiscal_year_id, user_id, new)` (`journal_entries.rs:125-147`, saisie manuelle, `enforce_postable=true`) enveloppe `create_in_tx` : transaction atomique avec re-lock `fiscal_years FOR UPDATE`, garde `FiscalYearClosed` (`:201`), garde `DateOutsideFiscalYear` (`:211-213`), numérotation séquentielle, garde d'équilibre débit=crédit (`:289-306`), audit `journal_entry.created`. La validation métier amont (`accounting::validate`, équilibre / ≥2 lignes / libellé / montants ≥ 0) se fait au handler `create_journal_entry` (`routes/journal_entries.rs:397-508`).
- **Frontend saisie** : `JournalEntryForm.svelte` + `balance.ts` (arithmétique **big.js**, jamais `parseFloat` — CO 957-964) + `AccountAutocomplete.svelte` (filtre `a.active && a.postable`, `:32-34`). Types montants en **string décimale** (jamais `number`).

### Ce qui N'existe PAS encore (deltas à construire)

- ❌ **Aucun concept « soldes de départ » / « bilan d'ouverture » / « opening entry »** côté plan comptable/écritures (recherche exhaustive = 0). *(Ne PAS confondre avec `bank_imports.opening_balance` = solde de relevé bancaire CAMT.053/CSV, concept totalement distinct.)*
- ❌ **Aucun helper « exercice vierge »** : pas de `COUNT(*) FROM journal_entries WHERE fiscal_year_id=?` prêt (à construire).
- ❌ **Aucune requête « 1er exercice »** : `fiscal_years::list_by_company` trie `start_date DESC` (`:602-618`, le plus récent en tête) — pas de « premier exercice » (ASC) prêt (à construire).
- ❌ **Aucun flag « migration »** dans l'onboarding (`OnboardingState` = `stepCompleted, isDemo, uiMode, version`). **Hors scope 14-4** (décision D1 : écran dédié menu, PAS d'étape onboarding).

### Ce qui n'est PAS dans 14-4

- ❌ **Étape d'onboarding** dédiée (décision **D1** : écran menu découplé, utilisable post-onboarding).
- ❌ **Marqueur/colonne « écriture d'ouverture »** (décision **D2** : MVP sans marqueur — garde « exercice vierge » + correction via édition normale de l'écriture ; **PAS de migration**).
- ❌ **Auto-équilibrage** (décision **D3** : équilibre manuel + total en direct ; l'auto-solde sur un compte equity est une amélioration future **L2**).
- ❌ **Choix / création de la date du 1er exercice** : l'écriture est datée au `start_date` du **premier exercice existant** (tel que créé par l'onboarding = 1er janvier de l'année courante). Un migrant voulant un exercice décalé/antérieur le crée d'abord via Paramètres → Exercices (limitation **L1**).
- ❌ **Postings sur des comptes de résultat** (Revenue/Expense) : interdits (fausseraient le P&L de l'exercice courant) — l'écriture d'ouverture ne touche **que** des comptes de bilan (Asset/Liability). Garde serveur (défense en profondeur).
- ❌ **Snapshot / affectation du résultat assistée** : évolutions futures (14-1 L3, CO 958).

## Décisions de conception (tranchées par Guy 2026-07-24, avant dev)

### D1 — Écran dédié accessible depuis le menu (PAS d'étape onboarding)

Nouvel écran `frontend/src/routes/(app)/settings/opening-balances/+page.svelte` (ou équivalent sous la navigation « Comptabilité / Paramètres »), accessible **Comptable+** (miroir de la saisie d'écriture et du plan comptable). Découplé de l'onboarding (qui n'a aujourd'hui aucun mode « migration » et dont le flux ne doit pas être alourdi). Le migrant y accède quand il veut, tant que son 1er exercice est **vierge** (voir D2).

### D2 — MVP sans marqueur : garde « 1er exercice vierge » + correction via édition normale (PAS de migration)

L'écriture d'ouverture est une **écriture OD standard** (table `journal_entries`/`journal_entry_lines`), **sans aucune colonne ni marqueur dédié** :

- **Génération autorisée UNIQUEMENT si le premier exercice n'a AUCUNE écriture** (`COUNT(*) = 0`). C'est le garde-fou anti-doublon MVP : on ne peut pas générer deux écritures d'ouverture.
- Une fois générée (le 1er exercice contient alors 1 écriture), l'écran passe en **état verrouillé** : il pointe vers le journal pour toute **correction** (l'écriture d'ouverture est une OD normale, éditable/supprimable via l'éditeur d'écritures existant tant que l'exercice reste ouvert).
- Pour **re-saisir intégralement** les soldes, l'utilisateur supprime l'écriture d'ouverture via le journal, puis rouvre l'écran (le 1er exercice redevient vierge).
- **Aucune migration** → politique Migration breaking (P3/P5) **sans objet**. **Aucun** audit d'idempotence à modifier.

**Limitation assumée (L3)** : sans marqueur, l'écran ne peut pas « ré-éditer » une écriture d'ouverture existante en pré-remplissant la grille — il verrouille et délègue au journal. Le marqueur robuste (variant `Journal::Ouverture` ou colonne `is_opening`) est une amélioration future si le besoin d'un ré-édition assistée émerge.

### D3 — Équilibrage manuel + total en direct (réutilise `balance.ts`)

La grille affiche le **total débit / total crédit cumulés et le déséquilibre** en direct (réutilise `computeBalance` de `balance.ts`, arithmétique **big.js**). Le bouton **« Générer l'écriture d'ouverture » est désactivé tant que l'écriture n'est pas équilibrée** (débits = crédits, total > 0, tous montants valides), exactement comme `JournalEntryForm` (`canSubmit`). Le migrant pose lui-même son report à-nouveau accumulé sur le compte de rôle `RetainedEarnings` (ou `EquityCapital`/`EquityOther`) pour équilibrer l'actif migré. **Pas d'auto-solde** (amélioration future L2).

### D4 — Grille = comptes de bilan (Asset + Liability) actifs ET postables

La grille de saisie liste les comptes **actifs ET postables de type `Asset` ou `Liability`** (capitaux propres inclus : capital, réserves, report à-nouveau — ce sont des comptes de type `Liability` dans les 3 plans). Exclut :
- Les comptes **Revenue / Expense** (fausseraient le P&L de l'exercice courant — l'écriture d'ouverture ne touche que le bilan).
- Les comptes **non-postables** (regroupements, `CurrentYearResult` 2979) — cohérent avec `AccountAutocomplete` (`.filter(a => a.active && a.postable)`).

Conséquence : le flux réutilise `journal_entries::create` (saisie manuelle, `enforce_postable=true`) sans exemption — la grille n'offre déjà que des comptes postables. **Garde serveur défense-en-profondeur** : le handler rejette toute ligne dont le compte n'est pas `Asset`/`Liability` (`error-opening-balances-non-balance-account`) même si le client est contourné.

### D5 — Endpoint dédié, RBAC Comptable+, journal `OD`, description en langue de la company, garde atomique

- Nouvel endpoint **`POST /api/v1/opening-balances`** monté dans **`comptable_routes`** (`lib.rs:282-580`, scellé `require_comptable_role`) — même niveau que la création d'écriture. Consultation → 403 ; non-auth → 401.
- Le handler **force** `journal = OD` et `entry_date = start_date du 1er exercice` (jamais fournis par le client — anti-injection, miroir de la façon dont `fiscal_year_id`/`entry_number` sont calculés par le repo).
- **Description en langue COMPTABLE de la company (finding P1-H1)** : la description de l'écriture est un champ **persistant et immuable** (CO 957-964) → elle DOIT être rendue dans la langue comptable de la company, **PAS** la locale globale du serveur (`errors::t()` est mono-locale, figée au boot `main.rs:217` — inadaptée). Utiliser le pattern établi `let locale = Locale::from(company.accounting_language.as_str()); state.i18n.format(&locale, "opening-balances-entry-description", None)` (miroir exact `routes/invoices.rs:1153-1157` pour les descriptions d'écritures de facturation ; `company.accounting_language` = `crates/kesh-db/src/entities/company.rs:163`). DTO minimal : `OpeningBalancesRequest { lines: Vec<OpeningBalanceLine { account_id, debit, credit }> }` (montants **string décimale**, parse `Decimal::from_str` miroir `create_journal_entry`).
- **Garde atomique via fn repo dédiée (finding P1-C1)** : le wrapper `journal_entries::create` (`:125-147`) est **auto-contenu** (ouvre/commit sa propre tx, opaque) → **impossible** d'y injecter le re-check `count == 0` sous le lock. **NE PAS** le réutiliser tel quel. À la place, **nouvelle fn repo `journal_entries::create_opening_entry`** (miroir du pattern `invoices::validate_invoice:1245-1409` = own tx + `create_in_tx` direct) :
  1. `pool.begin()`.
  2. `SELECT id, status FROM fiscal_years WHERE id=? AND company_id=? FOR UPDATE` → `None`→`NotFound` ; `Closed`→`Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)` ; `Open`→continue. **Ce lock sérialise avec toute création d'écriture concurrente** (`create_in_tx` verrouille la même ligne `fiscal_years FOR UPDATE`, `:188-201`).
  3. `count_in_fiscal_year(&mut tx, company_id, fiscal_year_id)` **sous le lock** → si `> 0` → rollback + `Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)`.
  4. `create_in_tx(&mut tx, fiscal_year_id, user_id, new, /*enforce_postable*/ true)` (re-lock `fiscal_years` ré-entrant même tx ; valide comptes/équilibre/date ; insère ; audit `journal_entry.created`).
  5. `commit`.
  Le check status+count **sous** le `FOR UPDATE` ferme réellement la fenêtre de course (résout aussi la course « Closed concurrent » L5 : le code 14-4 distinct est émis, pas le générique `FiscalYearClosed`).
- Réutilise `accounting::validate` (équilibre / ≥2 lignes / montants ≥ 0 / débit⊕crédit — appelé au handler comme `create_journal_entry:456`) + `create_opening_entry`. Mapping d'erreurs : **promouvoir `map_core_error` (`routes/journal_entries.rs:171`) en `pub(crate)`** et le réutiliser (DRY — PAS de duplication des arms) pour `ENTRY_UNBALANCED`/`VALIDATION_ERROR` ; nouveau `map_opening_balances_error` (miroir `map_reopen_error` 14-2) pour les 2 clés namespacées → `AppError::IllegalState(t(...))` (déjà 409) / `AppError::Validation` ; `INACTIVE_OR_INVALID_ACCOUNTS` via `AppError::from`.

### D6 — État de l'écran piloté par un endpoint de statut

Nouvel endpoint **`GET /api/v1/opening-balances/status`** monté dans **`comptable_routes`** (`require_comptable_role` — **PAS** `authenticated_routes` qui est « tout rôle » `lib.rs:582-594` et laisserait passer Consultation ; finding P1-M2/L-a) retournant l'état permettant à l'écran de décider quoi afficher :

```jsonc
{
  "fiscalYear": { "id": 12, "name": "Exercice 2026", "startDate": "2026-01-01", "status": "Open" } | null,
  "canEnter": true,        // 1er exercice existe, Open, et vierge (0 écriture)
  "reason": "READY" | "NO_FISCAL_YEAR" | "FIRST_YEAR_CLOSED" | "ALREADY_HAS_ENTRIES"
}
```

`canEnter=false` + `reason` pilote un **état verrouillé** avec message explicite (pas de grille). Évite de laisser le POST échouer en aveugle et donne une UX claire au migrant.

## Acceptance Criteria

### A. Backend — endpoint `POST /api/v1/opening-balances` (génère l'écriture d'ouverture)

- **Given** un utilisateur **Comptable+**, un premier exercice **Open et vierge**, et un body `{ lines: [{ accountId, debit, credit }, …] }` **équilibré** (≥2 lignes non nulles, débits = crédits) sur des comptes de bilan postables, **When** `POST /api/v1/opening-balances`, **Then** **201** + `JournalEntryResponse` : une écriture `journal = OD`, `entry_date = start_date du 1er exercice`, description rendue via la clé `opening-balances-entry-description` **dans la langue comptable de la company** (`Locale::from(company.accounting_language)`, D5/P1-H1 — pas la locale serveur globale), rattachée au 1er exercice, avec audit `journal_entry.created`.
- **And** `entry_date` et `journal` sont **forcés serveur** (tout champ du client est ignoré/absent du DTO).
- **And** l'écriture apparaît immédiatement dans le **bilan** (via le calcul cumulatif `fetch_cumulative_section`, `entry_date ≤ as_of`) — **aucune modification de `balance_sheet.rs`**. Le report accumulé posé sur un compte de rôle `RetainedEarnings` s'itemise dans la section Capitaux propres (14-3c) et la ligne « Résultat reporté (calculé) » reste à 0 (aucun exercice Kesh antérieur).

### B. Backend — gardes 14-4

- **Given** aucun exercice pour la company, **When** POST, **Then** **400** `error-opening-balances-no-fiscal-year` (créer d'abord un exercice).
- **Given** le premier exercice est **Closed**, **When** POST, **Then** **400** `error-opening-balances-first-year-closed`.
- **Given** le premier exercice **contient déjà ≥1 écriture**, **When** POST, **Then** **409** avec **`error.code` = `ILLEGAL_STATE_TRANSITION`** (code machine **PARTAGÉ**, comme 14-2 L2) MAIS **`error.message`** = le texte **distinct localisé** `error-opening-balances-already-has-entries`. Pattern D7 de 14-2 : le repo émet `DbError::Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)`, `map_opening_balances_error` le mappe en `AppError::IllegalState(t(...))` (message déjà localisé) — **JAMAIS** le générique `DbError::IllegalStateTransition` (dont le `Display` log-only écraserait le message). ⚠️ **Les tests assertent le `error.message`** (contenu distinct), pas le `error.code` (partagé) — sinon le défaut repasse silencieusement (P1-H1-BH).
- **Given** une ligne cible un compte **Revenue/Expense** (ou inexistant/archivé/non-postable/autre company), **When** POST, **Then** **400** : `error-opening-balances-non-balance-account` (type ≠ Asset/Liability, garde 14-4 défense-en-profondeur — voir helper `accounts::find_types_by_ids_in_tx` AC-C) OU `INACTIVE_OR_INVALID_ACCOUNTS` (garde `journal_entries` existante pour archivé/non-postable/cross-tenant).
- **Given** un body **déséquilibré** OU `< 2` lignes non nulles OU montant négatif, **Then** **400** (`ENTRY_UNBALANCED` pour le déséquilibre ; `VALIDATION_ERROR` pour < 2 lignes [`EntryNeedsTwoLines`] et montant négatif [`EntryNegativeAmount`] — via `accounting::validate`, miroir `create_journal_entry` + `map_core_error`). **Les 3 branches sont testées séparément** (AC-H, P1-M2-AA).
- **And** la garde « 1er exercice vierge » (`ALREADY_HAS_ENTRIES`), la vérification du statut `Open`, et l'insertion s'exécutent **dans la même transaction sous `fiscal_years FOR UPDATE`** (fn repo dédiée `create_opening_entry`, D5/P1-C1) — le lock sérialise avec toute création d'écriture concurrente (qui prend le même `FOR UPDATE`), fermant réellement la fenêtre de course. **NE PAS** réutiliser le wrapper `create()` (auto-contenu, pas d'injection possible). Testé par un scénario de course concurrente (AC-H, P1-M3-BH).

### C. Backend — nouveaux helpers & fn repo

- `fiscal_years::find_first_by_company(pool, company_id) -> Result<Option<FiscalYear>, DbError>` : `WHERE company_id=? ORDER BY start_date ASC, id ASC LIMIT 1` (tie-break `id ASC` déterministe si deux exercices partageaient une même `start_date` — cas limite, finding P1-L-3).
- `journal_entries::count_in_fiscal_year(tx_or_pool, company_id, fiscal_year_id) -> Result<i64, DbError>` : `SELECT COUNT(*) FROM journal_entries WHERE company_id=? AND fiscal_year_id=?`. Prend un `&mut Transaction` (appelé **sous le lock** dans `create_opening_entry`) ; une surcharge pool-level sert le `GET /status`.
- `accounts::find_types_by_ids_in_tx(&mut tx, company_id, ids) -> Result<Vec<(i64, String)>, DbError>` (**helper manquant, finding P1-M1-ECH**) : `SELECT id, account_type FROM accounts WHERE company_id=? AND id IN (…)` (bind list, pattern `IN(...)` établi — PAS de N+1 via `find_by_id_in_company` en boucle). Sert la garde « comptes de bilan » (rejet Revenue/Expense) du handler.
- **fn repo dédiée `journal_entries::create_opening_entry(pool, company_id, fiscal_year_id, user_id, new) -> Result<JournalEntryWithLines, DbError>`** (D5/P1-C1) : own tx + `SELECT fiscal_years FOR UPDATE` (status : `Closed`→`Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)`) + `count_in_fiscal_year(&mut tx)` (`>0`→`Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)`) + `create_in_tx(&mut tx, …, true)` + commit. Constantes clés namespacées à côté de `FY_NAME_DUPLICATE_KEY`/`FY_REOPEN_*` (14-2). C'est le **fix structurel** de la course : vérification status+count **sous** le `FOR UPDATE` que prend aussi toute création d'écriture concurrente → sérialisation par construction (pas de garde séquentielle racy).

### D. Backend — endpoint `GET /api/v1/opening-balances/status` (D6)

- **Given** un Comptable+, **When** `GET /api/v1/opening-balances/status`, **Then** **200** + `{ fiscalYear, canEnter, reason }` (cf. D6) : `canEnter=true`/`reason=READY` ssi 1er exercice existe, `Open`, et `count=0` ; sinon `NO_FISCAL_YEAR` / `FIRST_YEAR_CLOSED` / `ALREADY_HAS_ENTRIES`.
- **And** scopé company (multi-tenant), Consultation autorisé en lecture ? → **non** : réservé Comptable+ (cohérent avec l'écran mutateur ; Consultation ne voit pas l'entrée de menu). Non-auth → 401.

### E. Frontend — écran « Soldes de départ »

- **Given** la page `settings/opening-balances`, **When** elle charge, **Then** elle appelle `GET /opening-balances/status` + `fetchAccounts(false)` (`Promise.allSettled`, tolérance de panne miroir `journal-entries/+page.svelte:98-102`).
- **And** si `canEnter=false` → **état verrouillé** : message explicite selon `reason` (`opening-balances-locked-no-fiscal-year` / `-first-year-closed` / `-already-has-entries`, ce dernier avec lien vers le journal pour correction). **Pas de grille.**
- **And** si `canEnter=true` → **grille** listant les comptes **actifs + postables de type Asset/Liability** (filtre client `a.active && a.postable && (a.accountType==='Asset' || a.accountType==='Liability')`), une ligne par compte (numéro + nom + badge rôle si présent), colonnes **Débit / Crédit** (inputs string décimale, mutuellement exclusifs par ligne, réutilise la validation `balance.ts` : `AMOUNT_RE`, `classifyLine`, `computeBalance`).
- **And** un **bandeau de total en direct** (total débit / total crédit / différence, `formatSwissAmount`) miroir de `JournalEntryForm` (`:426-469`) ; le bouton **« Générer l'écriture d'ouverture »** est **désactivé** tant que non équilibré (`!balance.isBalanced`) ou soumission en cours.
- **And** la soumission POST `{ lines }` (uniquement les lignes non vides) → succès : toast `opening-balances-success` **puis rechargement du statut** (`getOpeningBalancesStatus()`) → l'écran repasse en **état verrouillé `ALREADY_HAS_ENTRIES`** in-place (comportement **déterministe** unique, finding P1-M2-BH — pas de redirection). Cet état verrouillé propose un **lien vers le bilan** (`opening-balances-goto-balance-sheet`) et un **lien vers le journal** (`opening-balances-goto-journal`) pour visualiser/corriger. Échec serveur → message serveur affiché inline (gestion `err.code` : `ENTRY_UNBALANCED`, `ILLEGAL_STATE_TRANSITION` [message distinct], `VALIDATION_ERROR`, `INACTIVE_OR_INVALID_ACCOUNTS`, `opening-balances-non-balance-account`).
- **And** un utilisateur **Consultation** ne voit **pas** l'entrée de menu / est refusé (403 backend = filet).

### F. Frontend — types, API client, navigation

- `opening-balances.types.ts` : `OpeningBalanceLineRequest { accountId, debit, credit }`, `OpeningBalancesRequest { lines }`, `OpeningBalancesStatus { fiscalYear, canEnter, reason }`.
- `opening-balances.api.ts` : `getOpeningBalancesStatus()` → `GET`, `generateOpeningBalances(req)` → `POST`.
- Entrée de navigation (sidebar / menu Comptabilité ou Paramètres) vers l'écran, gatée Comptable+ (miroir du pattern RBAC local `canModify()` de `accounts/+page.svelte:354-357`).

### G. i18n (4 locales) — hardcode interdit

- Clés (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, convention `opening-balances-*` / `error-opening-balances-*`) : `opening-balances-title`, `-intro`, `-account`, `-debit`, `-credit`, `-total-debit`, `-total-credit`, `-diff`, `-generate`, `-generating`, `-success`, `-entry-description` (libellé serveur de l'écriture), `-locked-no-fiscal-year`, `-locked-first-year-closed`, `-locked-already-has-entries`, `-goto-journal`, `-goto-balance-sheet` (+ variables `{ $name }` / `{ $date }` où utile) ; erreurs serveur : `error-opening-balances-no-fiscal-year`, `error-opening-balances-first-year-closed`, `error-opening-balances-already-has-entries` (**atteinte via le mapper D7-like**), `error-opening-balances-non-balance-account`. **4 locales synchronisées.** (Réutiliser `journal-entry-form-balanced/-unbalanced` pour le bandeau si adéquat.)
- **Note consommation (finding P2-BH1)** : toutes ces clés vivent dans le **bundle partagé** `kesh-i18n/locales/*.ftl`, mais elles sont consommées à des endroits distincts — les clés **UI** (`opening-balances-*`) par le frontend (`i18nMsg`), les clés **erreur serveur** (`error-opening-balances-*`) par le backend (`t()` / `state.i18n.format`, insérées dans `error.message` que le frontend affiche tel quel, cf. AC-E), et `-entry-description` par le backend (langue company). Aucune clé morte : chaque clé a un consommateur (front OU serveur). Le frontend ne reformule PAS les `error-*` (il affiche `err.message`).

### H. Tests & gate

- **Repo** (`crates/kesh-db/tests/fiscal_years_repository.rs` + `.../journal_entries` ou fichier dédié) : `find_first_by_company` (plusieurs exercices → plus petit `start_date` ; **égalité `start_date` → tie-break `id ASC`**, P1-L-3 ; aucun → `None`) ; `count_in_fiscal_year` (0 / N) ; `find_types_by_ids_in_tx` (types corrects, filtre company) ; `create_opening_entry` (happy → OD + audit ; sur exercice avec écriture → `Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)`, aucun audit ; exercice Closed → `Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)` ; exercice inexistant → `NotFound`).
- **E2E** (`crates/kesh-api/tests/opening_balances_e2e.rs`, nouveau) — `POST` : happy path (201, écriture OD datée `fy_start`, équilibrée, **présente au bilan** via `GET /reports/balance-sheet` + description dans la langue comptable de la company) ; **déséquilibré → 400 `ENTRY_UNBALANCED`** ; **< 2 lignes non nulles → 400 `VALIDATION_ERROR`** (P1-M2-AA) ; **montant négatif → 400 `VALIDATION_ERROR`** (P1-M2-AA) ; ligne sur compte **Revenue → 400** `error-opening-balances-non-balance-account` (**assert `message`**) ; 1er exercice avec écriture préexistante → **409** (assert **`message`** distinct `already-has-entries`, **pas** le générique — P1-H1-BH) ; aucun exercice → 400 ; 1er exercice **Closed** → 400 (assert message distinct) ; Consultation → 403 ; non-auth → 401 ; PAT (si applicable au niveau comptable) ; cross-tenant (compte d'une autre company) → `INACTIVE_OR_INVALID_ACCOUNTS`. **`GET /status`** (P1-M1-AA) : les **4 valeurs `reason`** testées via le endpoint lui-même — `READY` (exercice Open vierge), `NO_FISCAL_YEAR` (aucun exercice), `FIRST_YEAR_CLOSED` (1er exercice clos), `ALREADY_HAS_ENTRIES` (après génération) — **+ RBAC `GET /status`** : Consultation → 403, non-auth → 401 (P1-L-4). **Course concurrente (P1-M3-BH)** : deux `create_opening_entry` (ou `create_opening_entry` vs `create_in_tx` normal) concurrents via `tokio::join!` — un seul réussit, l'autre `ALREADY_HAS_ENTRIES` ; état final = **une** écriture d'ouverture, pas de doublon (prouve la sérialisation par `FOR UPDATE`, miroir `reopen_close_concurrent_is_serialized` de 14-2). Chaque garde avec son test (`feedback_review_patch_needs_test`).
- **Frontend Vitest** (`opening-balances-page.test.ts`, nouveau) : grille visible seulement si `canEnter` ; **état verrouillé + message selon chacune des 4 `reason`** ; bouton Générer désactivé tant que non équilibré / actif une fois équilibré ; submit appelle `generateOpeningBalances` avec les lignes non vides ; **succès → rechargement statut → état verrouillé in-place** (M2-BH) ; Consultation ne voit pas la grille. (Réutiliser les tests `balance.ts` existants pour l'arithmétique.)
- **Playwright E2E** (`frontend/tests/e2e/opening-balances.spec.ts`, nouveau — **finding P1-M3-AA**, CLAUDE.md § E2E) : parcours migrant bout-en-bout navigateur — charger l'écran sur un 1er exercice vierge (statut `READY` → grille), saisir des soldes équilibrés (au moins un actif + une contrepartie capitaux propres), cliquer « Générer », vérifier le toast + l'état verrouillé `ALREADY_HAS_ENTRIES` + la présence des soldes au bilan (`/reports`). Conventions `data-testid` systématiques (miroir `journal-entries.spec.ts`, `accounts.spec.ts`). Pré-requis Playwright (MariaDB + seed + `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE`) — cf. Dev Notes.
- **Gate Test Locally First** complet vert (backend fmt/build/clippy/test + frontend check/lint-i18n/test/build ; Playwright si l'infra locale est disponible). **Pas de migration.**

### I. Doc-sync

- **CHANGELOG** `[Non publié]` : nouvel écran « Soldes de départ » pour migrer une comptabilité (génère l'écriture d'ouverture équilibrée datée au 1er jour du 1er exercice ; report à-nouveau accumulé posé sur le compte de rôle RetainedEarnings).
- **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : nouvelle sous-section « Reprise de comptabilité / soldes de départ » sous `\section{Exercices comptables}` (`:427`, à côté de Création/Clôture/Réouverture — **PAS** « Configuration des exercices » qui n'existe pas, finding P1-L-b) : procédure migrer → saisir les soldes de bilan → équilibrer via le report à-nouveau (compte de rôle RetainedEarnings) → générer.
- **README** (« Feuille de route » v0.8.0) : retirer *« bilan d'ouverture à suivre »* (feature livrée) — **même commit**.
- **Pas de migration** → **pas** d'entrée `docs/migrations-idempotence-audit.md`.

## Tasks / Subtasks

- [ ] **T1 — Helpers & fn repo** : `fiscal_years::find_first_by_company` (`ORDER BY start_date ASC, id ASC LIMIT 1`) ; `journal_entries::count_in_fiscal_year` (variante `&mut tx` + pool-level) ; `accounts::find_types_by_ids_in_tx` (bulk `IN(...)`, P1-M1-ECH) ; **fn `journal_entries::create_opening_entry`** (own tx + `SELECT fiscal_years FOR UPDATE` status guard + `count_in_fiscal_year(&mut tx)` guard + `create_in_tx(&mut tx, …, true)` + commit ; constantes `FY_OPENING_ALREADY_HAS_ENTRIES_KEY`/`FY_OPENING_FIRST_YEAR_CLOSED_KEY` naméspacées, P1-C1). Tests repo. — AC-C.
- [ ] **T2 — Endpoint statut** `GET /api/v1/opening-balances/status` : nouveau module `routes/opening_balances.rs` (déclaré `routes/mod.rs`), handler résolvant `find_first_by_company` + statut FY + `count_in_fiscal_year` → `{ fiscalYear, canEnter, reason }` (READY / NO_FISCAL_YEAR / FIRST_YEAR_CLOSED / ALREADY_HAS_ENTRIES). Monté `comptable_routes`. — AC-D/D6.
- [ ] **T3 — Endpoint génération** `POST /api/v1/opening-balances` (`routes/opening_balances.rs`) : DTO `OpeningBalancesRequest { lines }` ; parse `Decimal` (miroir `create_journal_entry`) ; résout 1er exercice via `find_first_by_company` (400 `no-fiscal-year` si absent ; pré-check statut Closed → 400 UX) ; **garde comptes de bilan** (`find_types_by_ids_in_tx` → rejet Revenue/Expense → `error-opening-balances-non-balance-account`) ; force `journal=OD` + `entry_date=fy_start` + **`description` via `Locale::from(company.accounting_language)` + `state.i18n.format`** (P1-H1, PAS `errors::t()`) ; `accounting::validate` (au handler) ; **`journal_entries::create_opening_entry`** (fn dédiée atomique — status+count sous `FOR UPDATE`, P1-C1 ; **PAS** `create()`). **Promouvoir `map_core_error` en `pub(crate)`** + le réutiliser (DRY) ; nouveau `map_opening_balances_error` (miroir `map_reopen_error` 14-2) : `FY_OPENING_ALREADY_HAS_ENTRIES_KEY` → `AppError::IllegalState(t("error-opening-balances-already-has-entries"))` (409 code partagé + message distinct) ; `FY_OPENING_FIRST_YEAR_CLOSED_KEY` → `AppError::Validation(t("error-opening-balances-first-year-closed"))` (400). Monté `comptable_routes`. — AC-A/B/D5.
- [ ] **T4 — Frontend types + API** : `opening-balances.types.ts` + `opening-balances.api.ts` (`getOpeningBalancesStatus`, `generateOpeningBalances`). — AC-F.
- [ ] **T5 — Écran + navigation** : page `settings/opening-balances/+page.svelte` (statut → grille OU verrou ; grille comptes Asset/Liability postables, colonnes Débit/Crédit, bandeau total `balance.ts`, bouton désactivé si non équilibré ; submit lignes non vides ; gestion `err.code`) + entrée de menu gatée Comptable+. — AC-E/F.
- [ ] **T6 — i18n** : clés `opening-balances-*` / `error-opening-balances-*` × 4 locales (AC-G), dont `-entry-description` (libellé serveur) et les 4 messages d'erreur.
- [ ] **T7 — Tests** : repo (T1, dont `create_opening_entry` + tie-break `find_first`) + e2e `opening_balances_e2e.rs` (happy path + toutes gardes AC-H : déséquilibré / <2 lignes / négatif / non-balance-account / already-has-entries / closed / no-fy / RBAC POST+GET / cross-tenant / **`GET /status` les 4 `reason`** / **course concurrente** ; **assertions de `message`** sur already-has-entries & non-balance-account) + Vitest `opening-balances-page.test.ts` (verrou 4 reasons/grille/équilibre/submit→verrou in-place) + **Playwright `opening-balances.spec.ts`** (parcours migrant bout-en-bout, P1-M3-AA). — AC-H.
- [ ] **T8 — Doc-sync** : CHANGELOG + user-manual (section reprise de compta) + README (retirer « bilan d'ouverture à suivre »). **Pas de migration.** — AC-I.

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-24 par 2 agents de cartographie)

**Backend — écritures & exercices**
- Création : `crates/kesh-db/src/repositories/journal_entries.rs` — `create` (`:125-147`, `enforce_postable=true`), `create_in_tx` (`:168-342`, params `tx, fiscal_year_id, user_id, new, enforce_postable`) ; gardes `FiscalYearClosed` (`:201`), `DateOutsideFiscalYear` (`:211-213`), équilibre débit=crédit (`:289-306`), postabilité `validate_lines_accounts_in_tx` (`:65-114`) ; audit `journal_entry.created`. `MAX(entry_number)+1` scopé `(company_id, fiscal_year_id)` (`:231-240`).
- `NewJournalEntry`/`NewJournalEntryLine`/`Journal` (Achats/Ventes/Banque/Caisse/**OD**) : `crates/kesh-db/src/entities/journal_entry.rs:33-41`, `:168-190`. `fiscal_year_id`/`entry_number` calculés par le repo.
- **Pattern écriture système équilibrée** (référence) : `invoices::validate_invoice` `crates/kesh-db/src/repositories/invoices.rs:1245-1409` (`create_in_tx(..., false)`).
- **Test qui matérialise déjà l'ouverture** : `crates/kesh-db/tests/report_aggregates.rs:1057-1140` (OD 1er janvier, débit actif / crédit `RetainedEarnings` 2970, migrant).
- Exercices : `fiscal_years::list_by_company` (**DESC**, `:602-618`) ; `find_by_id_in_company` (`:422`) ; `create_if_absent_in_tx` (`:228-289`) ; `FiscalYear`/`NewFiscalYear` (`crates/kesh-db/src/entities/fiscal_year.rs:73-92`). **`find_first_by_company` (ASC) à créer.**
- **`count_in_fiscal_year` à créer** (aucun `COUNT(*) FROM journal_entries WHERE fiscal_year_id=?` existant).

**Backend — rôles & bilan**
- `AccountRole` (10 valeurs, `EquityCapital/EquityOther/RetainedEarnings/CurrentYearResult` + `Receivable/…`) : `crates/kesh-db/src/entities/account.rs:89-181` ; `is_singleton` (`:157-169`) ; `postable` champ (`:240`). `accepts_account_type` (`kesh-core/src/chart_of_accounts/mod.rs:161-166` : `DefaultRevenue`→Revenue, sinon Asset|Liability). `is_postable` (`:314-319`, seul `CurrentYearResult` non-postable).
- **RetainedEarnings postable pour migrant** : `kesh-core/src/chart_of_accounts/mod.rs:308-309` + migration `20260722000001_accounts_role_postable.sql:173-174` (« persona Story 14-4 »). Numéros conventionnels 2970 (RetainedEarnings) / 2979 (CurrentYearResult) dans `crates/kesh-core/assets/charts/{pme,association,independant}.json`. **NE PAS hardcoder les numéros** — raisonner par rôle/type (principe Guy).
- Bilan : `crates/kesh-report/src/balance_sheet.rs` — `fetch_cumulative_section` (`:275-313`, `entry_date ≤ as_of`, **aucun filtre FY**), partition equity/dettes par rôle (`:120-135`, `:173-198`), `fetch_retained_earnings` (`:328-351`, ligne calculée `entry_date < fy_start`), distinction D1 (`:30-43`, `:82-89`). **Aucune modif nécessaire** pour 14-4.

**API & RBAC**
- `crates/kesh-api/src/routes/journal_entries.rs` : `create_journal_entry` (`:397-508`, pré-check `find_covering_date` `:440-454`, `accounting::validate` + `create`), DTOs (`:62-97`), `map_core_error` (`:171-196`), `MAX_DESCRIPTION_LEN=500` (`:373`), `MAX_LINES_PER_ENTRY=500` (`:370`). Tests mapping inline (`:642-751`).
- Montage : `crates/kesh-api/src/lib.rs` — `comptable_routes` (`:282-580`, `require_comptable_role` `:578-580`, merge `:854`) ; `authenticated_routes` (lecture, `:583-594`). Ajouter `pub mod opening_balances;` dans `routes/mod.rs` (à côté `:26`).
- Erreurs : `crates/kesh-api/src/errors.rs` — `ENTRY_UNBALANCED`/`NO_FISCAL_YEAR`/`FISCAL_YEAR_CLOSED`/`DATE_OUTSIDE_FISCAL_YEAR` (`:1004-1033`), `INACTIVE_OR_INVALID_ACCOUNTS` (`:2161-2163`). **Réutiliser `AppError::IllegalState(String)` (ajouté par 14-2, → 409 code partagé `ILLEGAL_STATE_TRANSITION` + message localisé)** pour `ALREADY_HAS_ENTRIES` — pattern `map_reopen_error` de 14-2 comme modèle du `map_opening_balances_error`.
- **Réutilisation atomique / DRY** : `journal_entries::create_in_tx` est **`pub`** (`:168`, réutilisable depuis un nouveau module route/repo) — la fn dédiée `create_opening_entry` l'appelle directement dans sa propre tx (miroir `invoices::validate_invoice:1245-1409`). `map_core_error` (`routes/journal_entries.rs:171`) est **privé** → le promouvoir `pub(crate)` pour le réutiliser sans dupliquer les arms (DRY, P1-L1-ECH).
- **Locale par company (P1-H1)** : pattern établi `Locale::from(company.accounting_language.as_str())` + `state.i18n.format(&locale, key, args)` — `crates/kesh-api/src/routes/invoices.rs:1153-1157` (descriptions d'écritures de facturation), `routes/exports.rs:55` (en-têtes CSV). `errors::t()` (`errors.rs:33-39`) est mono-locale (figée au boot `main.rs:217`) → **inadaptée** à un champ persistant multilingue. Champ `company.accounting_language: Language` (`crates/kesh-db/src/entities/company.rs:163`).

**Frontend**
- Saisie : `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte` (grille lignes, `canSubmit` `:139-147`, bandeau balance `:426-469`), `balance.ts` (**big.js**, `computeBalance` `:47-68`, `classifyLine` `:103-122`, `AMOUNT_RE` `:23`, `formatSwissAmount` `:93-101`), `form-helpers.ts` (`LineDraft` `:9-15`), `AccountAutocomplete.svelte` (filtre `active && postable` `:32-34`). Types : `journal-entries.types.ts` (montants string).
- Plan comptable : `frontend/src/routes/(app)/accounts/+page.svelte` (`canModify()` `:354-357`, badges rôle/postable), `accounts.api.ts` (`fetchAccounts(includeArchived)` `:10-14`), `accounts.types.ts` (`AccountResponse` : `role`, `postable`, `accountType`, `active` `:45-60`).
- Page hôte pattern (mode list/create inline, `Promise.allSettled`) : `frontend/src/routes/(app)/journal-entries/+page.svelte:98-123`.
- Rapports : `BalanceSheetView.svelte` (section equity par rôle, ligne calculée « (calculé) » `:34`, `:217-226`) — **aucune modif**. `reports.api.ts` `getBalanceSheet` (`:33-39`).
- i18n : `crates/kesh-i18n/locales/fr-CH/messages.ftl` (bloc `journal-entry-form-*` `:226-239`, `account-role-*` `:162-173`, `onboarding-*` `:39-102`), helper `i18nMsg` (`frontend/src/lib/shared/utils/i18n.svelte.ts:14-23`).

**Onboarding (contexte, hors scope écran)**
- Seed plan comptable : `crates/kesh-api/src/routes/onboarding.rs:372-389` (`count_by_company==0`). 1er exercice auto-créé **1er janvier année courante** : `onboarding.rs:750-773` (pas de date choisie). Pas de flag « migration » (`OnboardingState`).

**Tests**
- Pas de `journal_entries_e2e.rs` dédié (mapping inline `journal_entries.rs:642-751`). Fixtures via repo : `crates/kesh-api/tests/reports_e2e.rs:249-266`. Patterns e2e CRUD Comptable+ : `accounts_e2e.rs`, helper `tests/common/mod.rs:10`. E2E Playwright : `frontend/tests/e2e/journal-entries.spec.ts`, `accounts.spec.ts` (`data-testid` systématiques).

### Pièges, par ordre de coût

1. **Ne toucher QUE des comptes de bilan** (Asset/Liability) : une ligne sur Revenue/Expense datée `fy_start` fausse le P&L de l'exercice courant. Grille filtrée client + **garde serveur** (rejet type ≠ Asset/Liability). La ligne calculée « Résultat reporté » (`fetch_retained_earnings`, `entry_date < fy_start`) n'est **jamais** alimentée par l'écriture d'ouverture (datée = `fy_start`, pas `<`) — le report accumulé va sur le **compte physique** RetainedEarnings, pas la ligne calculée. C'est exactement la distinction D1 de 14-3c.
2. **Raisonner par RÔLE/TYPE, jamais par numéro** (principe Guy) : ne pas hardcoder 2970/2800/2979. La grille liste les comptes par type ; le migrant choisit ses comptes de capitaux propres (rôle affiché en badge).
3. **`entry_date = fy_start` forcé serveur** : doit tomber dans `[fy_start, fy_end]` du 1er exercice (garde `DateOutsideFiscalYear`) — `fy_start` exactement. Le 1er exercice doit être **Open** (garde `FiscalYearClosed`).
4. **Garde « exercice vierge » atomique (fix structurel, P1-C1)** : le check status+`count_in_fiscal_year` se fait **sous** `fiscal_years FOR UPDATE` dans la fn repo dédiée `create_opening_entry` (own tx + `create_in_tx` direct, miroir `validate_invoice`). **NE PAS** réutiliser `create()` (auto-contenu → aucune injection possible → course ouverte). `ALREADY_HAS_ENTRIES`/`FIRST_YEAR_CLOSED` = `Invariant(KEY)` naméspacés → mapper dédié (pattern D7 14-2, PAS le générique log-only).
5. **Équilibre = big.js côté client, `accounting::validate` côté serveur** : le serveur reste l'autorité (jamais `parseFloat`). Montants en **string décimale** de bout en bout.
6. **Description en langue de la company (P1-H1)** : `Locale::from(company.accounting_language)` + `state.i18n.format` (miroir `invoices.rs:1153`), **PAS** `errors::t()` (locale serveur globale figée au boot). Champ persistant immuable → langue correcte dès l'écriture.
7. **MVP sans marqueur (D2)** : autoriser la génération uniquement si `count==0` ; correction post-génération via l'éditeur d'écritures normal (l'écriture d'ouverture est une OD standard). Ne PAS ajouter de colonne/variant.
8. **`list_by_company` trie DESC** : ne pas prendre `[0]` pour le 1er exercice — créer `find_first_by_company` (ASC, tie-break `id ASC`).

### Limitations documentées (triage catégories)

- **L1 — date du 1er exercice non choisie dans 14-4 (catégorie C — décision design)** : l'écriture est datée au `start_date` du 1er exercice existant (onboarding = 1er janvier année courante). Un migrant voulant un exercice décalé le crée d'abord via Paramètres → Exercices. Pas une dette.
- **L2 — pas d'auto-équilibrage (catégorie C — décision design D3)** : équilibre manuel MVP. L'auto-solde de la différence sur un compte equity désigné est une amélioration future (friendly migrant) si le besoin émerge.
- **L3 — pas de ré-édition assistée de l'écriture d'ouverture (catégorie C — décision design D2)** : sans marqueur, l'écran verrouille après génération et délègue la correction au journal. Un marqueur robuste (variant `Journal::Ouverture` ou colonne `is_opening`) réactiverait une ré-édition assistée — amélioration future.
- **L4 — garde « exercice vierge » plus large que « pas deux écritures d'ouverture » (catégorie C — conséquence assumée de D2, finding P1-M1-BH)** : le critère `COUNT(*)==0` bloque l'écran si le 1er exercice contient **n'importe quelle** écriture (y compris une facture/banque saisie avant de découvrir l'écran), pas seulement une écriture d'ouverture. C'est **inhérent au MVP sans marqueur** (impossible de distinguer une écriture d'ouverture d'une écriture normale sans marqueur — cf. D2/L3). Un migrant dans ce cas saisit ses soldes via une écriture OD manuelle (mécanisme v1 de 14-1). L'écran verrouillé (`ALREADY_HAS_ENTRIES`) l'explique et pointe vers le journal. Levé si/quand le marqueur L3 est livré.
- **L5 — code d'erreur générique sur la course « Closed concurrent » évitée par construction (catégorie C — finding P1-L2-ECH)** : la vérification du statut `Open` étant faite **sous le `FOR UPDATE`** dans `create_opening_entry` (D5), une clôture concurrente est sérialisée → le code 14-4 distinct `FIRST_YEAR_CLOSED` est émis, **pas** le générique `FiscalYearClosed`. (Le pré-check statut du handler hors-lock reste un raccourci UX pour le cas non-concurrent ; l'autorité est la fn repo sous lock.) Pas une dette.

### References

- Conception : `14-1:50` (14-4 = écran dédié), `14-3c` (distinction D1 physique/calculé), décisions Guy 2026-07-24 (D1 écran menu, D2 sans marqueur, D3 équilibre manuel, D4 comptes bilan postables).
- Norme : CO art. 957-964 (immutabilité, arithmétique décimale exacte) ; reprise de comptabilité (bilan d'ouverture = position de clôture pré-Kesh reportée au 1er jour du 1er exercice).
- Conventions : CLAUDE.md § Test Locally First, § Review Iteration Rule, § Règle de commit, § Issue Tracking Rule, § Pattern batch (N/A ici — écriture unique).

## Change Log — create-story

Spec créée 2026-07-24 par cartographie ground-truth parallèle (2 agents Explore : backend écritures/exercices/rôles/bilan, frontend/API/onboarding). L'Epic 14 de `epics.md` est **périmé** — le scope 14-4 vient de la note léguée `14-1:50` + 14-3 + décisions Guy. 4 forks tranchés par Guy avant rédaction (options recommandées retenues) : **D1** écran dédié menu (pas d'étape onboarding) ; **D2** MVP sans marqueur (garde « exercice vierge » + correction via édition normale, **pas de migration**) ; **D3** équilibrage manuel + total en direct (réutilise `balance.ts`) ; **D4** grille = comptes de bilan (Asset+Liability) actifs et postables. Décisions dérivées : D5 (endpoint dédié `POST /opening-balances`, RBAC Comptable+, journal OD + date + description forcés serveur), D6 (endpoint `GET /status` pilotant l'état verrou/grille). Réutilise le variant `AppError::IllegalState` (14-2) pour le 409 `ALREADY_HAS_ENTRIES`. **Pas de migration** (P3/P5 sans objet). Aucune modification du moteur de bilan (l'écriture OD datée `fy_start` est incluse nativement). Prochaine étape : `bmad-create-story validate`.

## Change Log — validate

**Passe 1 (Sonnet ×3 — Blind Hunter / Edge Case Hunter / Acceptance Auditor, contexte frais, grep ground-truth)** : **1 CRITICAL, 2 HIGH, ~6 MEDIUM, ~6 LOW** (dédupliqués). ECH a vérifié ~40 ancres `fichier:ligne` — **toutes exactes**. Le CRITICAL a **convergé** sur BH + ECH (signal fort).

Findings & remédiation :
- **P1-C1 (CRITICAL, convergé BH+ECH)** — la garde anti-course `ALREADY_HAS_ENTRIES` était irréalisable via `journal_entries::create()` (wrapper auto-contenu, aucune injection possible du re-check sous le lock). **Fix structurel** : nouvelle fn repo dédiée `create_opening_entry` (own tx + `SELECT fiscal_years FOR UPDATE` status+count sous le lock + `create_in_tx` direct, miroir `validate_invoice`). Patché D5/AC-A/AC-B/AC-C/T1/T3, Piège 4, L5.
- **P1-H1-ECH (HIGH)** — la description était rendue via `errors::t()` (locale serveur globale) au lieu de la langue comptable de la company → violait le mandat multilingue sur un champ persistant immuable. **Fix** : `Locale::from(company.accounting_language)` + `state.i18n.format` (miroir `invoices.rs:1153`). Patché D5/AC-A/T3/Dev Notes/Piège 6.
- **P1-H1-BH (HIGH→clarif)** — ambiguïté code d'erreur : `code` = `ILLEGAL_STATE_TRANSITION` **partagé** (comme 14-2 L2), `message` = distinct localisé. **Les tests assertent le `message`.** Patché AC-B/AC-H.
- **P1-M1-ECH (MED)** — helper de type de compte manquant pour la garde « comptes de bilan » (`validate_lines_accounts_in_tx` privé + pas de select `account_type`). Ajout `accounts::find_types_by_ids_in_tx` (AC-C/T1).
- **P1-M2-BH (MED)** — comportement post-génération « OU » non tranché → **rechargement statut → verrou in-place** déterministe (AC-E).
- **P1-M3-BH (MED)** — atomicité non testée → **test de course concurrente** (AC-H/T7).
- **P1-M1-AA (MED)** — `GET /status` ne testait que 2 des 4 `reason` → **les 4 testées via le endpoint** + RBAC (AC-H).
- **P1-M2-AA (MED)** — branches AC-B (`<2 lignes`, négatif) sans test → **3 branches testées séparément** (AC-H).
- **P1-M3-AA (MED)** — aucun Playwright E2E (mandat CLAUDE.md) → **scénario Playwright `opening-balances.spec.ts`** (AC-H/T7).
- **P1-M1-BH (MED)** — garde `COUNT==0` plus large que l'objectif → **limitation L4** (conséquence assumée du MVP sans marqueur D2).
- **LOW** — D6 montage `comptable_routes` clarifié (P1-M2-ECH/L-a) ; `map_core_error` privé → `pub(crate)` DRY (P1-L1-ECH) ; tie-break `find_first_by_company id ASC` (P1-L-3) ; RBAC `GET /status` testé (P1-L-4) ; section manuel corrigée `\section{Exercices comptables}` (P1-L-b) ; numérotation Pièges dédupliquée (P1-L-c) ; course « Closed concurrent » émet le code distinct (P1-L2-ECH → L5).

Trend : **Passe 1 (Sonnet ×3) : 1 CRIT / 2 HIGH / 6 MED > LOW.** Tous patchés → **Passe 2 de confirmation (Haiku, modèle orthogonal)** requise par la Review Iteration Rule.
