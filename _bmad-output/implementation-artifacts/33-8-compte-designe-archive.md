# Story 33-8 : un compte désigné dans les paramètres de facturation ne s'archive plus sans avertissement (#486)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-16 à 18 et 29. Commits en `refs` ; à la PR : `closes #486`.*

## Story

En tant que Comptable qui range son plan comptable, je veux être averti, au moment d'archiver un compte, qu'il est désigné dans les paramètres de facturation, et par
quel réglage ; pour ne pas découvrir l'effet des mois plus tard, à la validation d'une facture ou à un règlement refusé.

## Ce que le code dit (prémisses établies)

- **Aucune garde** : `routes/accounts.rs:335-348` et `repositories/accounts.rs:681-756` ne vérifient que les sous-comptes actifs.
- ⚠️ **Prémisse corrigée — pas quatre réglages, dix.** L'issue en nomme quatre ; `entities/company_invoice_settings.rs:24-62` en porte **dix** : `default_receivable`, `default_revenue`, `default_vat_payable`,
  `default_vat_recoverable`, `default_vat_decompte`, `default_payable`, `default_rounding`, `default_discount`, `default_bank_fees`, `default_bad_debt`. Les libellés d'écran existent (`fr-CH/messages.ftl:680-704`, clés `settings-invoicing-*`) ; les trois de TVA sont `invoices-settings-vat-payable|recoverable|decompte` (`:583-585`, lus).
- **La seconde face, commentée dans #486, est toujours là** : `validate_account_of` refuse tout compte inactif (`routes/company_invoice_settings.rs:200-204`), sans égard pour « inchangé » (seul `require_postable` l'est, `:165`). Les comptes reconduits sont revalidés à chaque `PUT`, pour six champs. Un compte déjà archivé bloque donc l'enregistrement de **n'importe quel** réglage. Les quatre comptes d'écart, eux, sont déjà épargnés (`resolve_designated_account`, `:220-250`, dont le commentaire cite #486).
- **Le patron d'un refus qui se confirme** : `confirm_retype` et `DbError::AccountHasEntries` (`repositories/accounts.rs`, `errors.rs:3425-3450`, écran `accounts/+page.svelte:295`) : 409, ampleur dans `details`, second envoi explicite.
- **Droits** : l'archivage est au routeur Comptable (`lib.rs:525`), le réglage à l'Administrateur. Un **refus sec** enfermerait un Comptable, et un cas réel existe : un non-assujetti qui range ses comptes TVA, désignés d'office (C-e334-16).
- **14 appels** à `accounts::archive(` en `crates/`, presque tous des montages de tests : on les garde intacts (C-e334-17).
- **Écran** : dialogue à `accounts/+page.svelte:730-745`, `submitArchive` `:340-362`.

## Conception (C-e334-16, 17)

- **Avertir, puis laisser faire** : doctrine de #280 (« l'avertissement informe, il n'interdit pas »), et lettre de l'epic (« sans avertissement »).
- `repositories/company_invoice_settings.rs` : `designations_of_account(executor, company_id, account_id) -> Vec<&'static str>`, codes stables (`receivable`, `revenue`, `vatPayable`, `vatRecoverable`, `vatDecompte`, `payable`, `rounding`, `discount`, `bankFees`, `badDebt`) ; **un test d'inventaire compare la liste aux colonnes `default_%_account_id`** lues à `information_schema`, pour qu'un onzième réglage ne se taise pas.
- `accounts::archive_guarded(pool, id, version, user_id, company_id, confirm_designated)` porte le corps ; `archive` reste comme l'appelle le reste du dépôt, `confirm = true`, pour les montages. **Seule la route** passe `req.confirm_designated`. Lecture **dans** la transaction de l'archivage ; course avec un `PUT` de réglages concurrent : la même que `confirm_retype`, écrite comme limite.
- **Ordre des refus (C-e334-29)** : sous-comptes actifs (existant), **puis** désignation, puis version : la version se contrôle à l'`UPDATE` (`repositories/accounts.rs:703-707`), et la désignation se lit juste avant, sur l'état **courant**, non sur celui de l'écran. Un écran périmé sans confirmation reçoit donc le 409 de désignation, juste à cet instant ; s'il confirme, le 409 de version le fait recharger. Deux dialogues dans ce cas rare, plutôt qu'une lecture de version de plus. **Le refus de désignation ne consomme pas la version.**
- `DbError::AccountDesignatedInSettings { settings: Vec<&'static str> }`, **409 `ACCOUNT_DESIGNATED_IN_SETTINGS`**, `details: { settings }`, message traduit.
- `ArchiveAccountRequest.confirm_designated` : `#[serde(default)]` (`false`). Un appelant de l'API qui l'ignore reçoit le 409 : **changement de comportement à dire** dans `docs/api-external.md`.
- **Audit** : `account.archived`, snapshot inchangé, plus `"designated_settings": […]` **quand il n'est pas vide** (les tests existants restent exacts).
- **Seconde face** : `validate_account_of` ne refuse l'inactif que si la valeur **change** (paramètre renommé `changed`, qui commande inactif et imputable). Le type reste vérifié (#537 n'est pas élargie, ni aggravée). Motif : le manuel et l'écran disent qu'un compte désigné peut être archivé ; il ne doit pas verrouiller les réglages.
- **Écran** : à réception du 409, le dialogue reste ouvert, liste les réglages par leurs libellés existants, explique l'effet, propose **« Archiver quand même »**. ⚠️ **Le réglage est réservé à l'Administrateur, l'archivage ouvert au Comptable** : le lien vers *Paramètres → Facturation* ne s'affiche qu'à l'Administrateur (`authState.currentUser?.role`, déjà lu à `accounts/+page.svelte:414`) ; le Comptable lit à sa place « Un administrateur peut changer ce réglage ».

## Acceptance Criteria

1. **AC-1** : pour chacun des dix réglages (table), `PUT …/archive` sans confirmation → 409 `ACCOUNT_DESIGNATED_IN_SETTINGS`, `details.settings` exact ; **le compte reste actif, `version` inchangée, aucun audit**. Rouge avant.
2. **AC-2** : compte désigné par deux réglages → les deux codes.
3. **AC-3** : avec `confirmDesignated: true` → 200, compte archivé, audit portant `designated_settings`. Le réglage **reste** tel quel.
4. **AC-4** : compte non désigné → 200 sans confirmation, audit exactement inchangé. Sous-compte actif, désigné → refus existant d'abord. Version périmée, avec confirmation → 409 `OPTIMISTIC_LOCK_CONFLICT`. **Version périmée, sans confirmation, compte désigné → 409 `ACCOUNT_DESIGNATED_IN_SETTINGS`**, puis, sa confirmation envoyée à la même version, 409 `OPTIMISTIC_LOCK_CONFLICT` ; compte inchangé aux deux.
5. **AC-5** : **isolation** : un compte d'une autre société de même numéro, ou une désignation d'une autre société, ne déclenche rien (test à deux sociétés).
6. **AC-6 — inventaire** : la liste des dix codes égale les colonnes `default_%_account_id` de `company_invoice_settings`, recalculées en base.
7. **AC-7 — seconde face** : TVA due archivée par confirmation, puis `PUT /company/invoice-settings` qui ne change que le format de numérotation → 200. Passer **à** un compte archivé → toujours 400.
8. **AC-8 — écran** : les deux états (Vitest + `accounts.spec.ts`), libellés dans les quatre langues ; **Administrateur : le lien ; Comptable : aucun lien, le texte « Un administrateur… »**, et sa confirmation archive bien ; 409 de version toujours rendu par son message actuel.
9. **AC-9** : `reactivate` inchangé ; les 14 appels de `accounts::archive` passent sans modification.

## Tâches

- [ ] **T1 — rouges** : `tests/accounts_designated_archive_e2e.rs`, AC-1 à 7.
- [ ] **T2** : dépôt, `DbError`, `errors.rs`, route. ⚠️ `kesh-db` touché : gate complet en cours de boucle.
- [ ] **T3** : `validate_account_of` ; grep des tests qui épinglent « compte archivé » sur `company_invoice_settings` : les ajuster en connaissance de cause, et le dire au Change Log.
- [ ] **T4** : écran, types, API, Vitest, i18n quatre locales (`error-account-designated-in-settings`, `accounts-archive-designated-*`), `lint-i18n-ownership`.
- [ ] **T5** : E2E, captures thème clair (dialogue normal, avertissement) dans `kesh-gate-logs/e334/captures/`.
- [ ] **T6** : manuels, `docs/api-external.md`, CHANGELOG *Corrigé* (`refs #486`).

### Mutations nommées (P2)

- **M1** : ne lire que neuf colonnes (retirer `default_bad_debt`) → rouge AC-1, AC-6. **M2** : requête sans `company_id` → rouge AC-5.
- **M3** : `confirm_designated` ignoré → rouge AC-3. **M4** : refus après l'`UPDATE` → rouge AC-1 (`version`). **M5** : `changed` remis à `true` pour l'inactif → rouge AC-7.
- **M6** : `designated_settings` toujours écrit → rouge AC-4 (audit exact).
- **M7** : désignation lue après l'`UPDATE` ou version contrôlée avant elle → rouge AC-4 (version périmée sans confirmation). **M8** : lien affiché à tous les rôles → rouge AC-8.

## Gates

Gate complet, E2E complet au dernier commit de code. Aucune migration : la colonne existe, la requête lit seulement.

## Manuels

`user-manual.tex`, § `sec:plan-comptable-archivage` (`:404-410`) : l'avertissement, les réglages, « Archiver quand même » et ce que cela implique ; le comportement de refus à l'écriture. Paramètres de facturation : qu'un compte archivé y reste affiché. `docs/api-external.md` : le 409 et `confirmDesignated`. PDF régénérés, contrôlés aplatis.

## Issues voisines : rapprochées, non intégrées

- **#280** (factures validées référençant le compte) : même dialogue, autre lecture coûteuse : reste ouverte ; le dialogue laisse la place d'une seconde ligne.
- **#286** (article au compte devenu inutilisable), **#45** (validation serveur à la création) : autres gestes, autres écrans. **#537** (retypage d'un compte désigné) : le helper `designations_of_account` lui servira ; pas ici : changer le contrôle du retypage déplace une règle comptable.
- **À tracer** : les autres désignateurs sans avertissement (`products.revenue_account_id`, `bank_accounts.journal_account_id`, `reconciliation_rules.counterparty_account_id`, `settlement_account_id`) : **issue demandée** s'il n'en existe pas (grep au T1).

## Recoupements

`company_invoice_settings.rs` (route et entité) avec **33-1a**, qui y pose le format du numéro interne : autres champs, mais **le test d'inventaire ne vise que `default_%_account_id`**, et le corps reste en remplacement intégral : à vérifier à l'intégration. `accounts/+page.svelte` : aucune autre fiche.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5), remédiation (Opus 5.5). Aucun finding au-dessus de LOW. **L-R8-1** : ordre décidé (C-e334-29, lu : `repositories/accounts.rs:681-710`), AC-4 élargi, M7. **L-R8-2** : lien réservé à l'Administrateur, AC-8, M8. Point « libellés de TVA à vérifier » levé, lu.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
