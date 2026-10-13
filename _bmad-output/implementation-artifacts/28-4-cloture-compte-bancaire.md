# Story 28.4 : Clore un compte bancaire, en lecture seule, historique conservé

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur
`5aad8b35` (kickoff de l'Epic 28), worktree `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4`. Validation P1 à
lancer : `28-4-validate-prompt-p1.md` (Sonnet, lentilles R et F).

⛔ **Deux stories, une fiche, deux PR successives** (C-28-4-1, sur le précédent C-26-2-15) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`28-4a-cloture-serveur`** | migration, entité, lectures, gestes *clore* et *rouvrir*, refus serveur sur **tous** les chemins d'écriture, registre statique des écrivains, messages serveur (4 locales), étiquettes d'audit | AC1 à AC10 | **`refs #642`** — ⛔ jamais `closes` |
| **`28-4b-cloture-ecran`** | écran *Comptes bancaires*, réglages de facturation, libellé de refus de la réconciliation, i18n de l'interface, manuels (PDF), `docs/api-external.md`, CHANGELOG, captures clair/sombre | AC11 à AC17 | **`closes #642`** |

**Ordre** : la 28-4a se développe **après le merge de la 28-5, de la 28-6 et de la 28-7** (fichier d'epic, § *Ordre* :
elles touchent `routes/reconciliation.rs`), ou en parallèle avec rebase — la 28-4a n'y écrit que **quatre pré-vols
et un appariement d'erreur** (AC6, sites R1 à R5, plus R8 sur la route que la 28-5 introduit). La 28-4b part de `origin/main` **après** la fusion de la 28-4a.
La clé mère `28-4-cloture-compte-bancaire` passe au statut `split`. **Aucune release entre les deux PR** : entre elles,
la clôture n'existe que par l'API, sans écran, sans manuel ni CHANGELOG — à écrire au Dev Agent Record de la 28-4a.

## Story

**As a** indépendant, PME ou association dont un compte bancaire n'est plus utilisé ou a été clôturé par la banque,
**I want** marquer ce compte **clos** — en lecture seule —, avec une date de clôture facultative, et pouvoir le
rouvrir si je me suis trompé,
**so that** plus rien ne s'y enregistre (ni import, ni rapprochement, ni règlement, ni lot de paiement), tout en
gardant son historique entier consultable pendant les dix ans de conservation (CO 958f).

Issue : [#642](https://github.com/guycorbaz/kesh/issues/642) (CR, `enhancement` — sa fermeture ne fait pas baisser le
décompte des bugs). Demande du Project Lead (rétrospective de l'Epic 27, Q7, 2026-10-13) : « un compte devrait pouvoir
être archivé d'une manière ou d'une autre s'il n'est plus utilisé ou clos dans la banque : on pourrait, par exemple,
le marquer en lecture seule afin qu'il ne soit plus utilisé à l'avenir ». Ajoutée à l'epic (« ok, ajoute 642 »).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § 28-4. Cible : **v0.13.3**.

## Périmètre minimal — ce qui ferme #642, et ce qui en est exclu

**Leçon de l'Epic 28** : trois fiches ont dérivé en élargissant leur périmètre. Celui-ci est posé **avant** les AC, et
toute extension en cours de validation ou de revue se refuse ici, par écrit, ou se fait par CR.

**Dedans** (chaque ligne correspond à une phrase de #642) :

1. un état **clos**, distinct de l'archivage, porté par **une** colonne `bank_accounts.closed_on DATE NULL` (C-28-4-2) ;
2. deux gestes, **clore** et **rouvrir**, tracés à l'audit ;
3. le refus **côté serveur**, clés d'API comprises, de toute écriture qui viserait un compte clos — sur l'**inventaire
   clos** des écrivains (Dev Notes § *Inventaire*) ;
4. le compte clos n'est plus **proposé** là où l'on écrit (sélecteurs) et reste **lisible** là où l'on lit (écran de
   gestion avec filtre, export, sauvegarde, historique des imports, écritures, rapports) ;
5. les manuels, les messages, le CHANGELOG.

**Dehors**, et pourquoi :

| Exclu | Raison |
|---|---|
| Geler le **compte comptable lié** (1020…) : écritures manuelles, lettrage, contre-passation | Le grand livre n'est pas le compte bancaire : une écriture de correction peut être nécessaire après la clôture à la banque. Le compte comptable s'archive par son propre geste (*Plan comptable*), qui existe. L'écran **avertit** d'un solde non nul (C-28-4-6), le serveur ne refuse pas. |
| Contrôler la date de clôture contre les dates des transactions, ou l'interdire dans le futur | La date est **informative** ; c'est l'état qui garde (C-28-4-3). Un contrôle de date ouvrirait des questions de fuseau (UTC du serveur contre date locale suisse) sans rien protéger. |
| Clôture automatique (relevé CAMT portant un solde nul, inactivité) | #642 demande un geste ; une clôture automatique serait un appariement qui décide seul (§ *Un appariement automatique propose, il ne crée jamais*). |
| Masquer, dans l'historique des imports ou des règlements, les boutons *Supprimer l'import* / *Annuler le règlement* d'un compte clos | Le serveur refuse avec un message traduit (AC6) ; masquer exigerait de porter l'état du compte dans trois payloads de lecture. Angle mort assumé, écrit au manuel (AC14). |
| Réparer l'**archivage** des chemins qui ne le contrôlent pas (règlement de facture, `accept_one_*`) | Défaut voisin, préexistant, hors #642 : à ouvrir en issue par l'orchestrateur (rapport final). La garde de clôture ne lit que `closed_on` (C-28-4-12). |
| Lever le refus `412 BANK_ACCOUNT_HAS_TRANSACTIONS` de l'archivage | Écarté par #642 lui-même (« le sens d'archivé deviendrait flou ») : la clôture **remplace** cet usage. Seul le message du refus change (il propose la clôture, AC9). |
| Manuels DE / IT / EN | `docs/manual/{de,en,it}/` ne portent qu'un `README.md` : rien à traduire. |
| Cohérence d'une sauvegarde prise pendant une clôture | Relève de #633 (instantané commun, Story 28-3). Angle mort écrit à la doc de `close_in_tx`. |

**Décompte des modules** (§ *Règle de splitting préventif*) — `kesh-db` (migration, entité, six dépôts), `kesh-api`
(`routes/bank_accounts`, `routes/bank_imports`, `routes/reconciliation`, `exports`), `kesh-i18n`, frontend
(`features/bank-accounts`, `routes/settings/invoicing`, `features/reconciliation`), `docs/manual` : **plus de cinq**.
D'où le découpage en deux (C-28-4-1). La 28-4a reste au-dessus de cinq modules : **dérogation motivée** (§ *Dérogation*
des Dev Notes) — son pattern est **un** appel de garde posé à des points d'étranglement du dépôt, mécanique, et
l'inventaire clos n'a de sens que **complet** : le livrer en deux fois mettrait sur `main` un état « clos » qui ne
tient pas sur une partie des chemins. La 28-4b compte cinq modules.

## Critères d'acceptation

### Partie a — `28-4a-cloture-serveur`

**AC1 — La colonne, la contrainte, et les garde-fous de migration.**

- Migration neuve `crates/kesh-db/migrations/20261013000001_bank_accounts_closed_on.sql` — version **à relire au
  rebase** (aucune autre story de l'epic n'en prévoit, mais l'unicité se vérifie : `ls crates/kesh-db/migrations |
  grep 20261013`). Deux instructions, dans cet ordre :

  ```sql
  ALTER TABLE bank_accounts
      ADD COLUMN closed_on DATE NULL,
      ALGORITHM=INSTANT, LOCK=NONE;
  ALTER TABLE bank_accounts
      ADD CONSTRAINT chk_bank_accounts_closed_not_primary
          CHECK (closed_on IS NULL OR is_primary = FALSE);
  ```

  `closed_on IS NULL` = compte ouvert ; une date = compte clos (et sa date de clôture). En-tête en commentaires `--`
  seulement (`migrations_contain_no_block_comment`) : story, #642, non-breaking, pourquoi une seule colonne, pourquoi
  la contrainte (invariant I3, Dev Notes), aucune donnée écrite. Les commentaires évitent les mots-clés SQL
  d'écriture en tête de ligne.
- **Non-breaking, aucun bump `kesh_version_min_required`** (P1/P3 : `ADD COLUMN` nullable ; `ADD CONSTRAINT CHECK`
  n'est pas sur la liste P3). Le retour à une 0.13.2 est **de toute façon fermé** : le `MIGRATOR` de sqlx 0.8.6
  (`Cargo.lock`) a `ignore_missing: false` (`~/.cargo/registry/src/*/sqlx-core-0.8.6/src/migrate/migrator.rs:51`) et
  aucun site du dépôt ne l'active (`grep -rn "ignore_missing" crates/` : vide) — une 0.13.2 refuse de démarrer sur une
  base qui porte une migration qu'elle ne connaît pas (`VersionMissing`, l. 40). **Constat statique**, non exécuté :
  le manuel d'administration le dit au conditionnel prudent (« restaurer la sauvegarde prise avant la mise à jour »),
  sans affirmer le mécanisme (C-28-4-11 ; prémisse A-C8 de l'Epic 15, non versée au `CLAUDE.md` par cette story).
- **P5** : ligne à sa place chronologique **dans** le tableau de `docs/migrations-idempotence-audit.md`, verdict
  `tracked-by-sqlx`. Compteurs **recomptés depuis le tableau** : sur `5aad8b35`, en-tête `## Table d'audit (80
  migrations)` l. 17, `Total` l. 106 = 80, `yes` 10, `tracked-by-sqlx` 70, `no` 0 ; après la 28-4a : **81 / 81 / 10 /
  71 / 0**, la ventilation du `Total` gagnant `+ 1 Story 28-4a`. Contrôle :
  `ls crates/kesh-db/migrations/*.sql | wc -l` = `grep -c '^| \`20' docs/migrations-idempotence-audit.md` = 81.
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté site par site. Sur `5aad8b35`, un
  seul site positionnel : `crates/kesh-db/tests/migrations_upgrade_path.rs` (`total, 80` l. 118 ; `total - 46`
  l. 202 et l. 205), frontière **34**. La 28-4a le porte à **`total` 81, soustracteur 47** (81 − 47 = 34), ajoute à la
  généalogie (après la l. 195, « Story 26-2a : 45 → 46 … ») la ligne **« Story 28-4a : 46 → 47, frontière inchangée
  (81 - 47 = 34) »**, et réécrit le message d'assertion (« 80 précédentes + Story 28-4a : bank_accounts_closed_on »).
  Toutes les mentions de la valeur se greppent **par jeton** — `grep -nE "\b(80|46)\b"
  crates/kesh-db/tests/migrations_upgrade_path.rs` — et ne doivent plus rendre que des généalogies (sur `5aad8b35` :
  l. 44, 46, 63, 64, 84, 115, 118, 119, 123, 131, 138, 195, 202, 205, 274). Les autres sites résolvent par version
  (`migrations_before`) : rien à faire, **à confirmer par le gate**.
- **P7** : la migration n'écrit aucune donnée → ni `POST_RESTORE_BACKFILLS`, ni `EXEMPT_MIGRATIONS` ;
  `every_data_backfill_migration_is_triaged` vert **au gate**, pas supposé.
- **P8** : la migration ne s'édite plus une fois appliquée à une base persistante (`kesh_284`, `kesh_e2e_284`) ; toute
  précision va à la ligne d'audit ou aux Dev Notes.
- **Squash** régénéré par `scripts/regen-test-schema.sh` (jamais à la main), **après le dernier rebase** :
  `crates/kesh-db/tests/test_schema_guard.rs` vert. **Registre des checksums** `crates/kesh-db/migrations.sha384` :
  **une ligne ajoutée** (`20261013000001 <sha384>`), à la suite de `20261011000002`, jamais une ligne modifiée ;
  `grep -c '^20' crates/kesh-db/migrations.sha384` = 81.
- **Contrainte exercée** : un test SQL direct (`UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = <principal>`)
  échoue sur `chk_bank_accounts_closed_not_primary` ; même chose pour `UPDATE … SET is_primary = TRUE` sur un compte
  clos.

**AC2 — L'entité et les lectures.**

- `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) gagne `closed_on: Option<NaiveDate>`, sérialisé
  `closedOn` (`"AAAA-MM-JJ"` ou `null`) ; la liste `COLUMNS` (`repositories/bank_accounts.rs:38`) le porte.
- **Filtre de liste** : `list_by_company(pool, company_id, include_archived: bool)` (l. 172) devient
  `list_by_company(pool, company_id, filter: BankAccountListFilter)` avec
  `BankAccountListFilter { include_archived: bool, include_closed: bool }` (`Default` = les deux à `false`). Sémantique :
  par défaut, comptes **ni archivés ni clos** ; `include_closed` ajoute les clos non archivés ; `include_archived`
  ajoute les archivés **clos ou non**. Les quatre combinaisons sont testées. Appelants (sur `5aad8b35`) :
  `routes/companies.rs:352` → `Default` (le compte clos sort des sélecteurs d'import et de réconciliation, qui lisent
  `/companies/current`) ; `exports/global.rs:288` → les deux à `true` ; `list_by_company_with_balances` (l. 962-967)
  relaie le filtre ; tests de `bank_accounts_repository.rs` (l. 91, 134, 860, 867, 1620) et
  `bank_account_statement_gap.rs` (l. 154, 430) adaptés.
- `GET /api/v1/bank-accounts` (`routes/bank_accounts.rs:487`) : query `includeClosed` (défaut `false`) à côté de
  `includeArchived` (`ListBankAccountsQuery`, l. 117-122).
- **Lectures inchangées, et qui doivent le rester** (un test chacune, sur un compte clos) : `GET /bank-imports` et
  `GET /bank-imports/{id}` (aucun filtre d'archivage aujourd'hui) ; `GET /reconciliation/proposals?bankAccountId=`
  d'un compte clos rend `200` (liste vide par l'invariant I1) — la garde d'archivage l. 610 n'est **pas** étendue ;
  l'export global (`bank_accounts.csv` gagne une colonne `closed_on` après `archived`, `csv_tables.rs:626-660`, valeur
  `AAAA-MM-JJ` ou vide) ; la sauvegarde `.keshbackup` (export générique `kesh_db::backup::export_table`, rien à coder :
  un test d'aller-retour vérifie qu'un compte clos revient clos, à la même date).
- `count_other_active_for_company` (l. 760-775) **exclut** les comptes clos (`AND closed_on IS NULL`) : un compte clos
  ne peut pas devenir principal, il ne doit donc plus empêcher l'archivage du compte principal (AC#9/AC#10 de la
  v014-1). `first_active_bank_account_linked_to` (l. 820) **garde** les comptes clos (C-28-4-7) : le compte comptable
  d'un compte clos reste « lié à une banque » pour le refus de désignation comme compte de créance.

**AC3 — Clore.**

- `POST /api/v1/bank-accounts/{id}/close`, corps `{ "version": i32, "closedOn"?: "AAAA-MM-JJ" }` (extracteur de
  validation `impl_validation_extractor!`, 400 sur corps malformé). Routeur `comptable_routes` (`lib.rs:349`), à côté
  de `/api/v1/bank-accounts/{id}` (l. 687-692). Comptable+ ; clé `read-write` acceptée, clé `read` refusée `403
  API_KEY_READ_ONLY` par le middleware (C-28-4-8, précédent de l'archivage). Garde `assert_onboarding_complete`.
- `closedOn` absent → date du jour **UTC** (`chrono::Utc::now().date_naive()`, précédent `opening_balances.rs:598`) ;
  l'écran l'envoie toujours (AC11). Aucune borne (C-28-4-3).
- **Ordre des refus** : forme (`400` : `version < 1`, date invalide) → `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
  (`AppError::BankAccountNotFound` : inexistant, autre société, **archivé**) → **sous verrou** : `409
  OPTIMISTIC_LOCK_CONFLICT` (version) → `409 BANK_ACCOUNT_CLOSED` (déjà clos) → `412
  BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY` → `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` (`details.pendingCount`) → `412
  BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES` (`details.batchCount`, lots au statut `generated`) — refuser plutôt qu'avertir (C-28-4-4) ; le principal refusé, et rendu structurel par la contrainte de l'AC1 (C-28-4-5).
- Effet : `closed_on = <date>`, `version + 1`, **rien d'autre** — ni `archived`, ni `journal_account_id`, ni le compte
  comptable lié. Trace `bank_account.closed` dans la même transaction, `details` :
  `{ "bank_account_id", "closed_on", "before": { "closed_on": null, "version" }, "after": { "closed_on", "version" } }`.
- Séquence (dépôt, `bank_accounts::close_in_tx`) : `SELECT … FROM bank_accounts WHERE company_id = ? AND id = ? AND
  archived = FALSE FOR UPDATE` **première instruction** de la transaction, **sans** sentinelle `companies` ; puis les
  deux décomptes en **lecture non verrouillante** (Dev Notes § *Concurrence*) ; puis `UPDATE`. Réponse `200` avec le
  compte.

**AC4 — Rouvrir.**

- `POST /api/v1/bank-accounts/{id}/reopen`, corps `{ "version": i32 }`. Mêmes routeur, rôles, clés et garde
  d'onboarding. Ordre : forme → `404` (inexistant, autre société, archivé) → sous verrou `409
  OPTIMISTIC_LOCK_CONFLICT` → `409 BANK_ACCOUNT_NOT_CLOSED` (compte ouvert).
- Effet : `closed_on = NULL`, `version + 1`. Le compte **ne redevient pas principal** (il ne l'était pas, I3). Trace
  `bank_account.reopened`, `details` : `{ "bank_account_id", "before": { "closed_on", "version" }, "after": {
  "closed_on": null, "version" } }` — la date de clôture levée reste lisible à l'audit.

**AC5 — Le compte clos refuse ses propres modifications.**

`PUT /bank-accounts/{id}` (`update_for_company`, l. 548), `PATCH /bank-accounts/{id}` (`set_journal_account_id_for_company`,
l. 413) et `DELETE /bank-accounts/{id}` (`archive_for_company`, l. 622) refusent un compte clos en **`409
BANK_ACCOUNT_CLOSED`**, lu sur la ligne qu'ils verrouillent déjà `FOR UPDATE` (pas de seconde lecture). Place dans
l'ordre des refus : immédiatement **après** le `409` de version de chacun (C-28-4-9). Le court-circuit no-op du `PATCH`
(KF-004) **ne** dispense **pas** du refus : un `PATCH` identique sur un compte clos rend `409`, non `200`. L'archivage
d'un compte clos est donc refusé (il faut le rouvrir d'abord, C-28-4-10).

**AC6 — Tous les chemins d'écriture refusent un compte clos, côté serveur.**

Le refus est `409 BANK_ACCOUNT_CLOSED` (`details: { "bankAccountId" }`), sauf dans les lots, où il est par proposition
(§ *Pattern batch*). Il est posé aux **points d'étranglement du dépôt** — toute route, clé d'API comprise, qui y passe
est couverte — et doublé, là où la route lit déjà le compte en pré-vol, d'un refus précoce. Table complète au § *Inventaire*
(sites W1 à W29, et W30 que la 28-5 introduit) ; résumé :

| Route | Garde | Refus |
|---|---|---|
| `POST /bank-imports/preview` | pré-vol route (R6, à côté de `archived`, `bank_imports.rs:965`) | `409` |
| `POST /bank-imports` (CAMT et CSV) | pré-vol (R7, l. 1114) **et** `bank_imports::create_with_transactions` (W22-W23) | `409` |
| `DELETE /bank-imports/{id}` | `bank_imports::delete_unreconciled_in_tx` (W24), après les verrous des étapes 1-2 | `409` |
| `POST /reconciliation/accept` | pré-vol (R1, à côté de l. 1040) → **chaque** proposition en `failed[]`, code `BANK_ACCOUNT_CLOSED`, `200` ; et `invoice_settlements::create_in_tx` (W8) apparié au même code dans `accept_one_invoice` (R5, l. 2003-2009) | `200` + `failed[]` |
| `POST /reconciliation/reject` | pré-vol (R2, l. 3174) → chaque transaction en `failed[]` | `200` + `failed[]` |
| `POST /reconciliation/manual` | pré-vol (R3, l. 3564) | `409` |
| `POST /reconciliation/split` | pré-vol (R4, l. 4007) | `409` |
| `POST /reconciliation/transactions/{id}/cancel` | `reconciliation_cancel::cancel_in_tx` (W9), sur `bt.bank_account_id` | `409` |
| `POST /reconciliation/transactions/{id}/unreject` — **introduite par la 28-5, à garder au rebase** (absente de `5aad8b35`) | pré-vol R8 + garde dans sa transaction (W30) ; si la 28-5 n'est pas mergée au moment du dev, la ligne reste à faire et se dit au Dev Agent Record — la 28-4a ne dépend pas de son code | `409` |
| `POST /invoices/{id}/settlements` (virement) | `invoice_settlements::create_in_tx` (W8), quand le règlement porte un compte bancaire | `409` |
| `POST /invoices/{id}/settlements/{sid}/cancel` | `invoice_settlements_write::cancel_settlement_in_tx` (W25), si `settlement_bank_account_id` | `409` |
| `POST /supplier-invoices/{id}/pay` (virement) | `supplier_invoices::pay_in_tx` (W27), dans le `SELECT … FOR UPDATE` existant (l. 753-756) | `409` |
| `POST /supplier-invoices/{id}/settlement/cancel` | `supplier_invoices::cancel_settlement_in_tx` (W29), si `settlement_bank_account_id` | `409` |
| `POST /payment-batches` | `payment_batches::create_batch` (W17-W19) : refus précoce à l'étape (0) (l. 85-99, à côté de `archived`) **et** relecture verrouillée **après** les verrous `supplier_invoices` (l. 289) | `409` |
| `POST /payment-batches/{id}/confirm` | transitif : chaque facture passe par `pay_in_tx` (W27) ; l'étape (2) (l. 427-441) gagne le refus précoce à côté de `archived` | `409` |
| `PUT` / `PATCH` / `DELETE /bank-accounts/{id}` | AC5 (W13-W15) | `409` |

- `DbError::BankAccountClosed { bank_account_id: i64 }` (nouveau, `crates/kesh-db/src/errors.rs`) → `AppError::BankAccountClosed
  { bank_account_id }` → `409 BANK_ACCOUNT_CLOSED`, message traduit `bank-accounts-errors-closed`.
- **Un test HTTP par ligne de la table** (`bank_accounts_closed_e2e.rs`, AC10), sur un compte clos **après** avoir servi
  (import rapproché, règlement, lot confirmé) : le refus, **et l'absence d'écriture** (compte des lignes avant/après,
  aucune trace d'audit de l'opération refusée). Le même jeu rejoué sur le compte **rouvert** passe : c'est ce qui
  prouve que le refus vient de la clôture et non d'une autre garde.
- Les autres écrivains de l'inventaire sont **hors d'atteinte par invariant** (I1, I2, I3) ou **ne portent pas de compte
  bancaire** : chacun est classé, avec sa raison, au registre statique (AC8).

**AC7 — Concurrence : la clôture et un écrivain ne se croisent pas.**

- **Règle de la garde** (écrite au doc-comment de `bank_accounts::lock_open_for_write`) : lecture `SELECT closed_on FROM
  bank_accounts WHERE company_id = ? AND id = ? FOR UPDATE`, **dans la transaction de l'écriture**, **après** tous les
  verrous que le chemin prend déjà sur ses lignes métier (facture, facture fournisseur, transaction bancaire, import,
  lot) et **avant** le commit ; là où le chemin verrouille déjà la ligne `bank_accounts` (`pay_in_tx` l. 753, le
  règlement client l. 131-135), le contrôle se lit **dans cette même instruction**. Raison : l'ordre existant est
  « lignes métier → `bank_accounts` » (`settle_invoice` : facture puis compte ; `claims_for_target` : `companies` →
  réglages → `bank_accounts`) ; une garde posée **avant** les lignes métier créerait un cycle (Dev Notes § *Concurrence*).
- **La clôture** ne prend que le verrou exclusif de **sa** ligne, en première instruction, puis lit en non-verrouillant :
  aucun cycle possible avec un écrivain, qui ne lui dispute que cette ligne.
- **Deux tests déterministes** (kesh-db, `attendre_une_requete_en_cours`, `test_fixtures.rs:561`) :
  (i) une transaction d'import a pris la garde et inséré des transactions `pending` sans commiter ; une clôture lancée
  en parallèle **attend** (vue par `attendre_une_requete_en_cours`), puis, au commit de l'import, rend
  `BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` ; (ii) une clôture tient son verrou sans commiter ; un import lancé en
  parallèle attend sur la garde, puis, au commit de la clôture, rend `BankAccountClosed` et **n'a rien inséré**.
- Les routes neuves sont classées `SansEcritureAuJournal` au registre des routes (aucune enveloppe de rejeu) ; un `1213`
  éventuel remonte en `500` comme pour l'archivage — angle mort assumé, écrit au doc-comment.

**AC8 — Le registre statique des écrivains : l'inventaire clos se tient tout seul.**

Test neuf `crates/kesh-api/tests/bank_account_closed_inventory.rs`, patron de `audit_route_registry.rs` : il lit les
sources **hors `#[cfg(test)] mod …`** de `crates/kesh-db/src/` et `crates/kesh-api/src/`, extrait toute instruction
`INSERT [IGNORE] INTO` / `REPLACE INTO` / `UPDATE` / `DELETE FROM` visant `bank_imports`, `bank_transactions`,
`payment_batches`, `payment_batch_items`, `invoice_settlements`, `supplier_invoices` ou `bank_accounts`, et la rattache à
sa fonction englobante. Chaque couple `(fichier, fonction)` doit figurer au registre, avec un statut :

- `Gardee("<fonction de garde ou instruction>")` ;
- `Invariant("I1" | "I2" | "I3", "<raison>")` ;
- `SansCompteBancaire("<raison>")` (l'écriture ne porte aucun compte bancaire, ou porte un compte neuf) ;
- `HorsPerimetre("<raison>")` (restauration d'installation, rejeu post-restauration, routes de test).

Diff ensembliste **dans les deux sens** (une écriture neuve non classée rougit ; une entrée sans site rougit), comme le
registre des routes. Sur `5aad8b35`, l'extraction rend **29 sites** (W1-W29, § *Inventaire*) ; **30** après le rebase sur la 28-5 (W30, `unreject`). ⛔ Le test **n'établit
pas** que la garde s'exécute — c'est l'affaire des tests de l'AC6 ; il ferme le mode d'échec « un écrivain ajouté
demain contourne la clôture sans que rien ne le signale ». Son doc-comment le dit, et dit ce qu'il ne voit pas :
écritures construites par `format!` sans nom de table littéral, `QueryBuilder`, SQL hors de ces deux crates.

**AC9 — Messages serveur et audit, quatre locales.**

Clés neuves dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, bloc `bank-accounts-*` (l. 1280-1342
en fr-CH) : `bank-accounts-errors-closed`, `bank-accounts-errors-not-closed`, `bank-accounts-errors-cannot-close-primary`,
`bank-accounts-errors-has-pending-transactions`, `bank-accounts-errors-has-open-payment-batches` ; et
`audit-log-action-bank-account-closed` / `-reopened` (à côté de l. 2407). Textes français prescrits :

- *closed* : « Ce compte bancaire est clos : il est en lecture seule. Pour y enregistrer une opération, rouvrez-le
  d'abord (Administration → Comptes bancaires). »
- *not-closed* : « Ce compte bancaire n'est pas clos. »
- *cannot-close-primary* : « Le compte principal ne peut pas être clos : désignez d'abord un autre compte comme
  principal (c'est lui que porte la QR-facture). »
- *has-pending-transactions* : « Ce compte porte des transactions importées qui ne sont pas rapprochées. Rapprochez-les,
  ou supprimez l'import s'il ne contient aucune transaction rapprochée, avant de clore le compte. »
- *has-open-payment-batches* : « Un lot de paiement généré sur ce compte n'est ni confirmé ni annulé. Confirmez-le ou
  annulez-le avant de clore le compte. »
- audit : « Compte bancaire clos » / « Compte bancaire rouvert ».

Le message existant `bank-accounts-errors-has-transactions` (l. 1295 en fr-CH, fallback `errors.rs:2581-2587`) cesse de
proposer la suppression des imports comme voie d'archivage et propose la clôture : « Le compte bancaire contient des
transactions — archivage refusé pour préserver l'audit comptable. Pour le retirer de l'usage, clore le compte : il
passe en lecture seule et garde son historique. » — dans les quatre locales **et** dans le fallback Rust (grep de la
valeur, § *Propagation post-patch*). Étiquettes d'audit : `crates/kesh-api/src/audit_labels.rs` (l. 80-82) gagne
`bank_account.closed` et `bank_account.reopened` ; `audit_label_registry.rs` reste vert.

**AC10 — Les tests de la partie a, et les gardes qui changent de sens.**

Tests listés au § *Tests* (T-a1 à T-a35) ; mutations M1 à M24 et M33 (§ *Mutations*) **toutes rouges**, ou déclarées
équivalentes avec la raison. Gardes existantes à faire suivre, **chacune nommée** : `audit_route_registry.rs` (deux
routes neuves `("post", "bank_accounts::close_bank_account", Traced, SansEcritureAuJournal)` et `reopen` ; les décomptes
de son doc-comment — **115** routes de `lib.rs`, **118** au registre sur `5aad8b35`, l. 26-31 — passent à 117 / 120, à
relire après le rebase sur la 28-5 qui en ajoute) ; `audit_label_registry.rs` ; `test_schema_guard.rs` ;
`migrations_upgrade_path.rs` ; `textes_coherents.rs` (si une garde vise les messages modifiés) ; tests du CSV
(`exports_global_e2e.rs`, en-tête de `bank_accounts.csv`) ; `admin_backup_e2e.rs` (aller-retour). **Gate backend
complet** à chaque commit de la boucle (story `kesh-db` : migration et dépôts — **pas** de gate ciblé, même en boucle de
revue), E2E complet au dernier commit de code (il ne touche pas l'écran, mais `/companies/current` et `GET
/bank-accounts` changent de contenu).

### Partie b — `28-4b-cloture-ecran`

**AC11 — L'écran *Comptes bancaires*.**

`frontend/src/routes/(app)/bank-accounts/+page.svelte` et `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` :

- type `BankAccountSummary` (l. 15) et brut (l. 61) : `closedOn: string | null` ; `parseBankAccount` (l. 79) le porte ;
  `listBankAccounts(options?: { includeArchived?: boolean; includeClosed?: boolean })` (l. 105) — **les appelants
  existants gardent leur sens** (`listBankAccounts(false)` / `listBankAccounts()` → ni archivés ni clos) ;
  `closeBankAccount(id, version, closedOn)` et `reopenBankAccount(id, version)`.
- bouton bascule **« Afficher les comptes clos »** / « Masquer les comptes clos » (`data-testid="toggle-closed"`), à
  côté de celui des archivés (l. 325), défaut **masqué** (C-28-4-13) ;
- une ligne close : atténuée comme une archivée (l. 436), badge **« Clos le JJ.MM.AAAA »** (`closed-badge-{id}`, date
  au format suisse de `format.ts`), et **une seule action, « Rouvrir »** (`reopen-{id}`) — Lier, Modifier, Archiver et
  Clore masqués ;
- une ligne ouverte : action **« Clore »** (`close-{id}`) ; sur le compte **principal**, l'action est **désactivée**
  avec l'explication en `title` et en texte d'aide (« désignez d'abord un autre compte principal ») — le serveur
  refuse de toute façon (AC3) ;
- **dialogue de clôture** : texte qui dit ce que fait la clôture (lecture seule, historique conservé, réversible) ;
  champ **date de clôture** (`type="date"`, prérempli à la date **locale** du jour) ; si `currentBalance` est non nul et
  non `null`, **avertissement** : « Le solde comptable du compte lié est de {montant} : vérifiez-le avant de clore
  (écriture de virement ou de frais) » — avertir, **ne pas** bloquer (C-28-4-6) ; boutons *Annuler* (`outline`) et
  *Clore le compte* ;
- **dialogue de réouverture** : une phrase, *Annuler* / *Rouvrir* ;
- les refus serveur (`412` *pending*, lots ouverts, principal ; `409` version) s'affichent par le **message serveur** —
  par l'utilitaire de la 28-2 s'il est mergé (`[object Object]` interdit), sinon par le motif existant de la page
  (`isApiError(err) ? err.message : …`, l. 281) ;
- toasts de succès traduits ; après clôture, si les clos sont masqués, le toast le dit (« Compte clos. Affichez les
  comptes clos pour le retrouver. »).

**AC12 — Les sélecteurs ne proposent plus un compte clos.**

Aucun écran d'écriture ne change de code : leur source exclut désormais les clos par défaut (AC2) — import et
réconciliation via `/companies/current` (`bank-import/+page.svelte:31-32`, `reconciliation/+page.svelte:23-25`) ;
règlement de facture (`invoices/[id]/+page.svelte:567`, `invoices/due-dates/+page.svelte:105`), facture fournisseur
(`supplier-invoices/[id]/+page.svelte:87`), lots de paiement (`payment-batches/+page.svelte:51`), tuile de l'accueil
(`(app)/+page.svelte:58`) via `listBankAccounts()`. **Un seul appelant change** : les réglages de facturation
(`settings/invoicing/+page.svelte:148`) passent `includeClosed: true`, pour rester alignés sur le serveur, qui compte un
compte clos comme lié à son compte comptable (C-28-4-7). Preuve : un **E2E** qui clôt un compte (non principal, sans
transaction en attente) et vérifie son absence des **six** sélecteurs et de la tuile, puis sa présence après
réouverture.

**AC13 — Libellé du refus dans la réconciliation.**

`frontend/src/lib/features/reconciliation/failed-proposal-label.ts` (l. 139-144) gagne `case 'BANK_ACCOUNT_CLOSED':`
→ clé `reconciliation-failed-bank-account-closed` (quatre locales). Atteignable seulement par une course (compte clos
dans un autre onglet) ; un test Vitest du libellé.

**AC14 — Les manuels, PDF compris.**

- `docs/manual/fr/user-manual.tex`, § *Comptes bancaires* (`\label{sec:comptes-bancaires}`, l. 1642) : la phrase d'entrée
  (l. 1644, « Un compte qui porte des transactions importées ne s'archive pas ») renvoie à la clôture ; sous-section
  neuve **« Clore un compte bancaire »** (`\label{sec:clore-compte-bancaire}`) : quand clore (compte inutilisé, clôturé
  par la banque) ; les trois conditions (pas principal, aucune transaction non rapprochée, aucun lot généré) ; la date
  facultative ; ce qui devient impossible (import, rapprochement et son annulation, règlement par ce compte et son
  annulation, lot de paiement, modification, archivage) ; ce qui reste (historique des imports, écritures, rapports,
  export, sauvegarde ; le compte comptable lié reste ouvert) ; l'avertissement de solde ; la réouverture et sa trace ;
  l'angle mort des boutons *Supprimer l'import* / *Annuler le règlement* qui restent affichés et sont refusés.
- § *Supprimer un import* (l. 1724, phrase l. 1754-1757) : la voie « supprimer ses imports pour archiver » cède la place
  au renvoi à la clôture (`\S\ref{sec:clore-compte-bancaire}`) ; la phrase reste exacte sur l'archivage.
- § *Import bancaire* (l. 1696, « comptes bancaires non archivés ») → « ni archivés ni clos ». Toute autre mention
  greppée (`grep -n "non archiv\|archiv" docs/manual/fr/user-manual.tex`).
- `docs/manual/fr/admin-manual.tex` : sous-section **« Passer à la 0.13.3 »** (à côté de `sec:maj-0-13-2`, l. 1972 ;
  créée ici ou complétée par **union** si une autre story de l'epic l'a ouverte) : une migration, non breaking, pas de
  bump de `kesh_version_min_required` ; retour à la 0.13.2 : **restaurer la sauvegarde prise avant la mise à jour** ;
  les intégrations par clé d'API voient disparaître les comptes clos de `GET /bank-accounts` sans `includeClosed=true`.
- PDF régénérés (`make fr` dans `docs/manual/`, sous verrou), commités, **contrôlés aplatis** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "Clore un compte bancaire"`.

**AC15 — Interface en quatre locales.**

Clés UI neuves (fr-CH, de-CH, it-CH, en-CH) : `bank-accounts-actions-close`, `-actions-reopen`, `-actions-show-closed`,
`-actions-hide-closed`, `-labels-closed-badge` (avec `{ $date }`), `-close-dialog-title`, `-close-dialog-body`,
`-close-dialog-date`, `-close-dialog-balance-warning` (avec `{ $amount }`), `-close-dialog-confirm`,
`-reopen-dialog-body`, `-close-primary-hint`, `-toast-close-success`, `-toast-close-success-hidden`,
`-toast-reopen-success`, `reconciliation-failed-bank-account-closed` — **seize**, à recompter à l'écriture.
`settings-bank-manage-hint` (l. 1340, « ajouter, modifier ou archiver ») gagne « clore ». Gardes : `npm run
lint-i18n-ownership` ; `frontend/src/lib/shared/i18n-keys.test.ts` — `ATTENDU.sitesTotal` (2075 sur `5aad8b35`, l. 601)
**recompté** (`grep -o "i18nMsg(" <fichier>`), delta écrit avec son périmètre ; la 28-2 et la 28-5 le modifient aussi
(conflit de valeur, à recompter au rebase, jamais à additionner de confiance).

**AC16 — `docs/api-external.md`, CHANGELOG.**

- `docs/api-external.md`, § 7, ligne *Comptes bancaires* (l. 220) : `POST /bank-accounts/{id}/close`, `/reopen` ; note
  neuve « *Depuis la v0.13.3* » : `closedOn`, `includeClosed`, les refus `409 BANK_ACCOUNT_CLOSED` sur tous les chemins
  d'écriture, `failed[]` dans les lots de réconciliation ; § 10 (codes d'erreur) : les cinq codes neufs. La note ⁸
  (l. 233, archivage après suppression des imports) renvoie à la clôture.
- `CHANGELOG.md`, `## [0.13.3] — Non publié` (créé ou complété par **union**), rubrique **`### Ajouté`** : « **Clore un
  compte bancaire ([#642](https://github.com/guycorbaz/kesh/issues/642)).** … » — ce que fait la clôture, ses trois
  conditions, la réouverture, ce qui reste lisible ; et, dans le bloc *À faire en passant à la 0.13.3*, la ligne des
  intégrations par clé d'API (`includeClosed`).
- README, § *Fonctionnalités*, et `website/` : vérifier si une ligne décrit l'archivage des comptes bancaires ; la
  compléter si oui (sinon rien — pas de commit cosmétique).

**AC17 — Recette visuelle, et les tests de la partie b.**

- **Captures Playwright** hors dépôt (`/home/gcorbaz/devel/kesh-gate-logs/28-4b-captures/`), thème **clair et sombre** :
  liste avec un compte clos affiché (badge, action *Rouvrir*), dialogue de clôture **avec** l'avertissement de solde,
  action *Clore* désactivée sur le principal, dialogue de réouverture. Jointes au prompt de revue et à la PR (fichier
  d'epic, § *Recette visuelle*). ⛔ Aucune donnée réelle : fixtures synthétiques (IBAN de test).
- Tests T-b1 à T-b14 (§ *Tests*), mutations M25 à M32 (§ *Mutations*). E2E complet au dernier commit de code ;
  `bank-accounts-crud.spec.ts` (l. 117, 148) reste vert.

## Tasks / Subtasks

### 28-4a

- [ ] **T0** — Rebase sur `origin/main` après la 28-5, la 28-6 et la 28-7 (ou rebase continu) ; relire les numéros de
  ligne cités et les compteurs (routes, migrations, `sitesTotal`) **sur la branche rebasée**. Bases `kesh_284`,
  `kesh_e2e_284`, port 3037.
- [ ] **T1** — Migration, P5-P8, squash, `migrations.sha384`, `migrations_upgrade_path.rs` (AC1).
- [ ] **T2** — Entité, `COLUMNS`, `BankAccountListFilter`, appelants, `includeClosed`, `count_other_active_for_company`,
  CSV `closed_on` (AC2).
- [ ] **T3** — `DbError::BankAccountClosed`, `AppError` (cinq variantes neuves, `errors.rs` près de l. 2580-2610),
  fallbacks Rust (AC6, AC9).
- [ ] **T4** — `bank_accounts::lock_open_for_write`, `close_in_tx`, `reopen_in_tx` + doc-comments (règle de la garde,
  invariants I1-I3, angles morts) ; handlers `close_bank_account`, `reopen_bank_account` ; routes `lib.rs` (AC3, AC4, AC7).
- [ ] **T5** — Refus du compte sur lui-même : `update_for_company`, `set_journal_account_id_for_company`,
  `archive_for_company` (AC5).
- [ ] **T6** — Gardes des points d'étranglement W8, W9, W17-W19, W22-W25, W27, W29 ; W30 (au rebase sur la 28-5) ; pré-vols R1-R4, R6-R8 ; appariement R5
  (AC6). Pour chaque site, **noter au Dev Agent Record** la position retenue par rapport aux verrous déjà pris (AC7).
- [ ] **T7** — Registre statique `bank_account_closed_inventory.rs` (AC8).
- [ ] **T8** — i18n serveur ×4, `audit_labels.rs` (AC9).
- [ ] **T9** — Tests T-a1 à T-a35, mutations M1-M24, gardes nommées à l'AC10 ; gate complet + E2E complet au dernier
  commit de code ; commit par étape ; Dev Agent Record (« clôture accessible par l'API seulement jusqu'à la 28-4b »).

### 28-4b

- [ ] **T10** — Partir d'`origin/main` après la fusion de la 28-4a ; `cp -al` des `node_modules`.
- [ ] **T11** — `bank-accounts.api.ts`, écran, dialogues (AC11) ; réglages de facturation (AC12) ; libellé (AC13).
- [ ] **T12** — i18n UI ×4, `sitesTotal` recompté (AC15).
- [ ] **T13** — Manuels utilisateur et d'administration, PDF régénérés et contrôlés aplatis (AC14).
- [ ] **T14** — `api-external.md`, CHANGELOG, README / site si concernés (AC16).
- [ ] **T15** — Tests T-b1 à T-b14, mutations M25-M32, captures clair/sombre, E2E complet au dernier commit de code
  (AC17).

## Dev Notes

### Inventaire — tout ce qui écrit une ligne rattachée à un compte bancaire, au sol (`5aad8b35`)

**Tables qui référencent `bank_accounts`** (fermé : `grep -rn "REFERENCES bank_accounts" crates/kesh-db/migrations/`) :

```
crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:60:        FOREIGN KEY (settlement_bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
crates/kesh-db/migrations/20260628000002_payment_batches.sql:34:        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
crates/kesh-db/migrations/20260828000001_invoice_settlements_type.sql:36:        FOREIGN KEY (settlement_bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
crates/kesh-db/migrations/20260504000001_bank_imports.sql:38:        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
crates/kesh-db/migrations/20260504000001_bank_imports.sql:81:        FOREIGN KEY (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
```

soit `bank_imports`, `bank_transactions`, `payment_batches` (et ses `payment_batch_items`), `invoice_settlements`,
`supplier_invoices` — plus `bank_accounts` elle-même. `bank_profiles` et `reconciliation_rules` ne portent **aucun**
compte bancaire (`grep -n "bank_account" crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql` : vide).

**Écrivains SQL hors tests** — extraction (awk : instruction d'écriture sur l'une des sept tables, hors commentaire de
ligne, hors bloc `#[cfg(test)] mod …`, fonction englobante) sur `crates/` :

| # | Site | Fonction | Instruction | Classement |
|---|---|---|---|---|
| W1 | `kesh-api/src/routes/reconciliation.rs:2039` | `accept_one_invoice` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 ; et W8 en aval (R5) |
| W2 | `…/reconciliation.rs:2538` | `accept_one_split` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 |
| W3 | `…/reconciliation.rs:2990` | `accept_one_rule` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 |
| W4 | `…/reconciliation.rs:3356` | `reject_batch` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R2 |
| W5 | `…/reconciliation.rs:3801` | `post_manual_once` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R3 |
| W6 | `…/reconciliation.rs:4258` | `post_split_once` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R4 |
| W7 | `kesh-api/src/routes/bank_accounts.rs:604` | `create_bank_account` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf, ouvert) |
| W8 | `kesh-db/src/repositories/invoice_settlements.rs:415` | `create_in_tx` | `INSERT INTO invoice_settlements` | **Gardée** quand le règlement porte un compte bancaire (virement ; couvre `settle_invoice` et `accept_one_invoice`) |
| W9 | `kesh-db/src/repositories/reconciliation_cancel.rs:339` | `cancel_in_tx` | `UPDATE bank_transactions` | **Gardée** (`bt.bank_account_id`, après les verrous des étapes 1-3) |
| W10 | `kesh-db/src/repositories/bank_accounts.rs:49` | `create` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf) |
| W11 | `…/bank_accounts.rs:301` | `upsert_primary_in_tx` | `UPDATE bank_accounts` | Invariant I3 (vise le principal, jamais clos) |
| W12 | `…/bank_accounts.rs:331` | `upsert_primary_in_tx` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf) |
| W13 | `…/bank_accounts.rs:457` | `set_journal_account_id_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5) |
| W14 | `…/bank_accounts.rs:577` | `update_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5) |
| W15 | `…/bank_accounts.rs:640` | `archive_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5) |
| W16 | `…/bank_accounts.rs:702` | `flip_primary_off_for_company` | `UPDATE bank_accounts` | Invariant I3 (vise le principal) |
| W17 | `kesh-db/src/repositories/payment_batches.rs:137` | `create_batch` | `INSERT INTO payment_batches` | **Gardée** (après les verrous `supplier_invoices`) |
| W18 | `…/payment_batches.rs:152` | `create_batch` | `UPDATE payment_batches` | **Gardée** (même transaction que W17) |
| W19 | `…/payment_batches.rs:164` | `create_batch` | `INSERT INTO payment_batch_items` | **Gardée** (même transaction) |
| W20 | `…/payment_batches.rs:507` | `confirm_batch` | `UPDATE payment_batches` | **Gardée**, transitivement par W27 (+ refus précoce) ; et Invariant I2 |
| W21 | `…/payment_batches.rs:589` | `cancel_batch` | `UPDATE payment_batches` | Invariant I2 (un compte clos n'a aucun lot `generated`) |
| W22 | `kesh-db/src/repositories/bank_imports.rs:66` | `create_with_transactions` | `INSERT INTO bank_imports` | **Gardée** (première instruction de la fonction) |
| W23 | `…/bank_imports.rs:110` | `create_with_transactions` | `INSERT INTO bank_transactions` | **Gardée** (même transaction) |
| W24 | `…/bank_imports.rs:494` | `delete_unreconciled_in_tx` | `DELETE FROM bank_imports` | **Gardée** (après les étapes 1-2) |
| W25 | `kesh-db/src/repositories/invoice_settlements_write.rs:922` | `cancel_settlement_in_tx` | `DELETE FROM invoice_settlements` | **Gardée** si `settlement_bank_account_id` (lu l. 836) |
| W26 | `kesh-db/src/repositories/supplier_invoices.rs:524` | `create_in_tx` | `INSERT INTO supplier_invoices` | SansCompteBancaire — **à vérifier** : la facture naît ouverte, `settlement_bank_account_id` NULL |
| W27 | `…/supplier_invoices.rs:895` | `pay_in_tx` | `UPDATE supplier_invoices` | **Gardée** (dans le `FOR UPDATE` de l. 753-756) |
| W28 | `…/supplier_invoices.rs:1133` | `cancel_in_tx` | `UPDATE supplier_invoices` | SansCompteBancaire — **à vérifier** : n'annule qu'une facture non réglée (`supplier_invoice_cancel_blocker`, l. 985) |
| W29 | `…/supplier_invoices.rs:1369` | `cancel_settlement_in_tx` | `UPDATE supplier_invoices` | **Gardée** si `settlement_bank_account_id` |
| W30 | *introduite par la 28-5, à garder au rebase* — handler de `POST /api/v1/reconciliation/transactions/{id}/unreject` (annuler le rejet, #526 ; fiche `28-5-transaction-rejetee-atteignable.md`, non mergée au `5aad8b35`) | à relire sur la branche rebasée | `UPDATE bank_transactions` (remise à NULL de `auto_match_rejected_at`) | **Refus `409 BANK_ACCOUNT_CLOSED`** (pré-vol R8 sur le `bank_account_id` de la transaction, et garde `lock_open_for_write` dans sa transaction, après le verrou de la ligne `bank_transactions` et sous le verrou du compte qu'elle prend) ; en outre Invariant I1 (une rejetée est `pending`) |

Hors de l'extraction, à classer `HorsPerimetre` si le registre les voit : le rejeu post-restauration
(`kesh-db/src/post_restore/*.sql`, SQL et non Rust), l'import d'installation (`admin_backup/import.rs`), les routes de
test (`routes/test_endpoints.rs`). ⚠️ Les lignes de `repositories/invoices.rs` 4353-4862 sont dans `mod tests` (ouvert
l. 2984-2985) : exclues à juste titre. **Élargir avant de figer** (consigne de la passe : un détecteur part du
symptôme) : écritures par `format!` (`grep -rn 'format!("UPDATE\|format!("INSERT\|format!("DELETE' crates/`),
`QueryBuilder`, `kesh-reconciliation`, `kesh-payment`, `kesh-seed` (aucun site sur `5aad8b35` pour les trois crates :
`grep -rlE "INSERT INTO|UPDATE|DELETE FROM" crates/kesh-reconciliation/src crates/kesh-payment/src` à rejouer).

**Pré-vols de route** (refus précoce, à côté de la garde d'archivage existante — sites `if ….archived {`) :

| # | Site | Handler |
|---|---|---|
| R1 | `routes/reconciliation.rs:1040` | `post_accept` — produit le `failed[]` complet, sans verrou |
| R2 | `routes/reconciliation.rs:3174` | `post_reject` — idem |
| R3 | `routes/reconciliation.rs:3564` | `post_manual` — `409` |
| R4 | `routes/reconciliation.rs:4007` | `post_split` — `409` |
| R5 | `routes/reconciliation.rs:2003-2009` | `accept_one_invoice` — `DbError::BankAccountClosed` de W8 → `FailedProposal` `BANK_ACCOUNT_CLOSED` (aujourd'hui `DATABASE_ERROR`) |
| R6 | `routes/bank_imports.rs:965` | `preview` — `409` |
| R7 | `routes/bank_imports.rs:1114` | `create` — `409` |
| R8 | *introduit par la 28-5, à garder au rebase* — handler `unreject` | `409` (W30) |

`get_proposals` (l. 610) est une **lecture** : pas de refus (AC2).

### Les trois invariants, et leurs écrivains (`invariant-dans-le-temps`)

- **I1 — un compte clos ne porte aucune transaction `pending`.** Établi par la clôture (`412` sinon). Écrivains qui
  **produisent** une `pending` : `create_with_transactions` (W23, gardé) et `reconciliation_cancel::cancel_in_tx` (W9,
  gardé) — `grep -rn "status = 'pending'" crates/*/src` à rejouer. Ceux qui la font **disparaître** (W1-W6, W24) ne le
  cassent pas. La restauration d'installation remplace toute la base par un état qui le respectait — sauf sauvegarde
  incohérente (#633, 28-3) : angle mort écrit.
- **I2 — un compte clos ne porte aucun lot `generated`.** Établi par la clôture. Seul producteur : `create_batch`
  (W17, gardé). `confirm_batch` et `cancel_batch` en sortent.
- **I3 — un compte clos n'est jamais principal.** **Structurel** : `chk_bank_accounts_closed_not_primary` (AC1). La
  clôture refuse le principal ; `PUT` sur un compte clos est refusé (AC5) ; la réouverture ne touche pas `is_primary`.
  D'où W11 et W16 hors d'atteinte, et `find_primary` (QR-facture, `invoice_pdf_service.rs:222`) qui ne rend jamais un
  compte clos.

### Concurrence — la doctrine appliquée ici

- InnoDB, `REPEATABLE READ` ; `sqlx::begin()` émet `BEGIN` sans instantané : la vue cohérente naît à la **première
  lecture non verrouillante**. La clôture fait son `FOR UPDATE` **d'abord** ; ses décomptes, lus ensuite en
  non-verrouillant, voient donc tout écrivain qui a commité avant qu'elle obtienne le verrou. Un écrivain qui tenait la
  garde au moment de la demande de verrou la fait **attendre** jusqu'à son commit ; un écrivain qui arrive après attend
  la fin de la clôture, puis lit `closed_on` en lecture **verrouillante** (valeur commitée la plus récente) et refuse.
- **Pourquoi la garde en verrou exclusif, et non partagé** : plusieurs chemins verrouillent déjà la ligne
  `bank_accounts` en exclusif plus loin dans la même transaction (règlement client l. 131-135, `pay_in_tx`
  l. 753-756). Une garde partagée suivie d'un exclusif sur la même ligne est une **montée de verrou** : deux écrivains
  concurrents en partagé, chacun demandant l'exclusif, s'interbloquent. L'exclusif partout supprime ce cas ; il
  sérialise les écritures d'un même compte bancaire, ce qui est négligeable (une PME, un humain).
- **Pourquoi après les lignes métier** : `cancel_settlement_in_tx` verrouille la facture (l. 824) puis le règlement ;
  `settle_invoice` verrouille la facture puis le compte. Une annulation de rapprochement qui prendrait le compte
  **avant** la facture formerait un cycle avec un règlement de la même facture depuis l'écran. Idem pour
  `create_batch` : il verrouille les factures fournisseur (l. 289) — d'où la relecture verrouillée **après**, et un
  simple refus précoce non verrouillé à l'étape (0).
- La clôture **ne prend pas** la sentinelle `companies` (contrairement à l'archivage, `routes/bank_accounts.rs:857`, et à la création et au remplacement, l. 561 et 719) : elle
  n'a besoin d'aucune autre ligne, et la sentinelle l'exposerait aux chemins qui verrouillent `companies` après
  `bank_accounts`. Le refus du principal se lit sur sa propre ligne, verrouillée ; un `PUT` concurrent sur la même ligne
  est sérialisé par le même verrou.

### Angles morts assumés (à écrire aux doc-comments)

- Le registre de l'AC8 voit les écritures à nom de table littéral, dans `kesh-db` et `kesh-api`, hors tests.
- Sauvegarde prise pendant une clôture : #633 / 28-3.
- Boutons *Supprimer l'import* / *Annuler le règlement* affichés sur l'historique d'un compte clos, refusés au clic
  (manuel, AC14).
- Un `1213` sur les deux routes neuves remonte en `500` (aucune enveloppe de rejeu, comme l'archivage).
- La date de clôture n'est contrôlée ni contre les transactions ni contre le futur (C-28-4-3).

### État actuel des fichiers modifiés — ce qui doit être préservé

- `routes/bank_accounts.rs` : l'archivage (l. 823-916) garde ses refus et leur ordre (pré-vol hors transaction pour le
  `404`, sentinelle, puis `412` transactions, `412` principal) — le refus `409 BANK_ACCOUNT_CLOSED` s'y lit sur la ligne
  verrouillée par `archive_for_company`. Le `PATCH` garde son court-circuit no-op **pour un compte ouvert**.
- `routes/reconciliation.rs` : le pré-vol par lot de la 27-4 (l. 1043-1075, `classify_batch_transaction`) reste tel
  quel ; le refus de clôture vient **avant** lui (une requête sur un compte clos ne lit pas ses transactions).
- `payment_batches.rs` : les refus d'archivage (`IllegalStateTransition`, l. 95-98 et 436-441) restent ; la clôture a
  son propre code.
- `/companies/current` : la forme `BankAccountJson` (`settings.types.ts:42`) ne change pas — seul le contenu exclut
  les clos.

### Pièges connus du dépôt, applicables ici

- `textes_coherents.rs` G19 (URL des manuels au tag) ne concerne que la release.
- `KESH_ADMIN_BACKUP_DIR` vers un dossier inscriptible pour l'E2E (`docs/agents-orchestration.md` § 4).
- Mutations : restauration par `git checkout -- <fichier>` **puis `touch`** (`mutation-restauree-binaire-perime`).
- Base de gate remise à zéro avant tout gate complet (`CLAUDE.md` § *Un gate laisse la base piégée*).
- Le registre de sprint et le registre des choix se fusionnent par **union**.

### Dérogation à la règle de découpage — 28-4a (C-28-4-1)

La 28-4a touche `kesh-db` (une migration, l'entité, six dépôts) et `kesh-api` (`routes/bank_accounts`,
`routes/bank_imports`, `routes/reconciliation`, `exports`, `errors`, `audit_labels`) et `kesh-i18n` : au-delà de cinq
modules. Justification : (1) **un seul pattern** — un appel `lock_open_for_write` à des points d'étranglement, ou une
colonne lue dans un `FOR UPDATE` existant — répété mécaniquement ; (2) l'**inventaire clos** est l'unité de preuve : une
moitié de gardes sur `main` ferait mentir l'état « clos » sur l'autre moitié, et le registre de l'AC8 ne peut pas être
vert à moitié ; (3) les touches à `routes/reconciliation.rs` sont réduites à sept lignes environ (R1-R5, R8), pour limiter
le conflit avec la 28-5, la 28-6, la 28-7. **Risque accepté** : une revue de code plus longue ; la revue du rollout se
fait **fichier par fichier** (§ *Comment splitter*). Signal de non-convergence (sévérité qui recycle) → déclaré au
Project Lead, comme le veut D5.

### Fichiers communs avec les autres stories de l'epic

| Fichier | Stories |
|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs` | 28-5, 28-6, 28-7 (sérialisées) — 28-4a : R1-R5, et R8 sur la route `unreject` de la 28-5 |
| `frontend/src/lib/features/reconciliation/failed-proposal-label.ts` | 28-5, 28-6 probables — 28-4b : un `case` |
| `crates/kesh-api/src/errors.rs` | 28-6 (code neuf `FailedProposal`), 28-5 |
| `crates/kesh-api/tests/audit_route_registry.rs`, `lib.rs`, `audit_labels.rs` | 28-5 (route « annuler le rejet ») |
| `crates/kesh-i18n/locales/*/messages.ftl` | 28-2 (repli d'erreur), 28-5, 28-6 |
| `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) | 28-2, 28-5 |
| `frontend/src/routes/(app)/bank-accounts/+page.svelte` | 28-1 (variante `destructive` du bouton *Archiver*), 28-2 (`String(err)` l. 281) |
| `docs/manual/fr/user-manual.tex` / `.pdf` | 28-5 (rejet), 28-6 / 28-7 si texte |
| `docs/manual/fr/admin-manual.tex` / `.pdf` | release (§ *Passer à la 0.13.3*) |
| `docs/api-external.md` | 28-5, 28-6 |
| `CHANGELOG.md` `[0.13.3]` | toutes |
| `kesh-db/src/backup.rs`, `admin_backup/export.rs` | 28-3 (instantané) — 28-4a n'y écrit rien, seulement un test |
| `_bmad-output/implementation-artifacts/sprint-status.yaml`, `epic-28-choix-autonomes.md` | toutes (union) |

### Tests

**Partie a** (`kesh-db` en `#[sqlx::test(migrations = "./test-schema")]` ; `kesh-api` en E2E HTTP) :

- T-a1 contrainte : principal → clos refusé ; clos → principal refusé (SQL direct).
- T-a2 à T-a5 `list_by_company`, quatre combinaisons du filtre.
- T-a6 `GET /bank-accounts` défaut / `includeClosed=true` / avec `includeArchived=true`.
- T-a7 `/companies/current` exclut un compte clos.
- T-a8 `count_other_active_for_company` exclut un compte clos (archivage du principal permis quand l'autre compte est
  clos).
- T-a9 `first_active_bank_account_linked_to` voit un compte clos (refus de désignation maintenu).
- T-a10 export global : `bank_accounts.csv` porte `closed_on`, valeur et vide.
- T-a11 `.keshbackup` aller-retour : le compte revient clos, même date.
- T-a12 clore : `200`, `closedOn` explicite ; T-a13 sans `closedOn` → date du jour UTC ; T-a14 trace d'audit (action,
  `details`).
- T-a15 à T-a21 refus de la clôture, dans l'ordre : T-a15 `400` forme ; T-a16 `404` archivé et autre société ;
  T-a17 `409` version ; T-a18 `409` déjà clos ; T-a19 `412` principal, avec le test d'**ordre** (principal **et**
  *pending* → `412 …PRIMARY`) ; T-a20 `412` *pending* (`pendingCount`) ; T-a21 `412` lot `generated` (`batchCount`).
- T-a22 rôles : Consultation `403`, clé `read` `403 API_KEY_READ_ONLY`, clé `read-write` `200`.
- T-a23 rouvrir : `200`, trace avec `before.closed_on` ; T-a24 rouvrir un ouvert → `409 BANK_ACCOUNT_NOT_CLOSED` ;
  T-a25 le compte rouvert n'est pas principal.
- T-a26 `PUT`, `PATCH` (dont `PATCH` identique, no-op), `DELETE` sur un clos → `409` (AC5).
- T-a27 une ligne par route de la table de l'AC6 : refus **et** absence d'écriture et de trace ; le même jeu sur le
  compte rouvert passe.
- T-a28 `accept` sur un clos : `200`, toutes les propositions en `failed[]` `BANK_ACCOUNT_CLOSED`, aucune ligne
  modifiée ; T-a29 `reject` idem.
- T-a30 R5 : W8 appelé sur un compte clos rend `BankAccountClosed` (test de dépôt), et `accept_one_invoice` l'apparie
  au code canonique (test d'unité ou E2E avec invariant cassé par SQL direct).
- T-a31, T-a32 les deux tests déterministes de l'AC7.
- T-a33 registre statique (AC8) ; T-a34 son volet inverse (entrée sans site) par un cas construit.
- T-a35 `audit_label_registry.rs`, `audit_route_registry.rs`, `test_schema_guard.rs`, `migrations_upgrade_path.rs` verts.

**Partie b** (Vitest, Playwright) :

- T-b1 `listBankAccounts` construit les quatre URL ; T-b2 `parseBankAccount` porte `closedOn` ; T-b3/T-b4
  `closeBankAccount` / `reopenBankAccount` (méthode, corps).
- T-b5 écran : ligne close → badge et seule action *Rouvrir* ; T-b6 principal → *Clore* désactivé ; T-b7 dialogue :
  date préremplie, avertissement présent si solde ≠ 0, absent si 0 ou `null` ; T-b8 refus `412` affiché par le message
  serveur ; T-b9 bascule *Afficher les comptes clos*.
- T-b10 réglages de facturation : `includeClosed: true`.
- T-b11 `failed-proposal-label` : `BANK_ACCOUNT_CLOSED`.
- T-b12 E2E : clore → absent des six sélecteurs et de la tuile ; rouvrir → présent (AC12).
- T-b13 E2E : clore, voir le badge, rouvrir (parcours de l'écran, captures AC17).
- T-b14 `i18n-keys.test.ts`, `lint-i18n-ownership`.

### Mutations

Chacune doit faire rougir au moins un test nommé ; une mutation équivalente se déclare avec sa raison.

| # | Mutation | Test attendu rouge |
|---|---|---|
| M1 | `lock_open_for_write` ne contrôle plus `closed_on` | T-a27 (plusieurs lignes) |
| M2 | filtre de liste ignore `include_closed` (défaut inclut les clos) | T-a2-T-a7 |
| M3 | clôture ne refuse plus le principal | T-a19 (la contrainte rend alors une erreur SQL, non le `412`) |
| M4 | décompte *pending* retiré | T-a20 |
| M5 | décompte des lots retiré | T-a21 |
| M6 | décompte *pending* en lecture faite **avant** le `FOR UPDATE` | T-a31 |
| M7 | réouverture ne remet pas `closed_on` à NULL | T-a23 |
| M8 | garde W8 retirée | T-a27 (règlement), T-a30 |
| M9 | garde W9 retirée | T-a27 (annulation de rapprochement) |
| M10 | garde W17 retirée (création de lot) | T-a27 |
| M11 | garde W22 retirée (import) | T-a27 (import, pré-vol R7 neutralisé aussi), T-a32 |
| M12 | garde W24 retirée | T-a27 (suppression d'import) |
| M13 | garde W25 retirée | T-a27 (annulation de règlement client) |
| M14 | garde W27 retirée | T-a27 (paiement fournisseur, confirmation de lot) |
| M15 | garde W29 retirée | T-a27 (annulation de règlement fournisseur) |
| M16 | pré-vol R1 retiré | T-a28 (sans R1, I1 rend un autre code : `failed[]` ≠ `BANK_ACCOUNT_CLOSED`) |
| M17 | pré-vol R2 retiré | T-a29 |
| M18 | pré-vols R3/R4 retirés | T-a27 (manuel, ventilé) |
| M19 | appariement R5 retiré | T-a30 |
| M20 | pré-vols R6/R7 retirés | T-a27 (aperçu) |
| M21 | AC5 : `update_for_company` ne refuse plus | T-a26 |
| M22 | AC5 : court-circuit no-op du `PATCH` avant le refus | T-a26 (`PATCH` identique) |
| M23 | `count_other_active_for_company` compte les clos | T-a8 |
| M24 | une entrée retirée du registre statique | T-a33 |
| M25 | `listBankAccounts` n'envoie plus `includeClosed` | T-b1 |
| M26 | ligne close garde l'action *Clore* | T-b5 |
| M27 | *Clore* actif sur le principal | T-b6 |
| M28 | avertissement de solde affiché pour un solde nul | T-b7 |
| M29 | dialogue n'envoie pas `closedOn` | T-b3 / T-b13 |
| M30 | réglages de facturation sans `includeClosed` | T-b10 |
| M31 | `case 'BANK_ACCOUNT_CLOSED'` retiré | T-b11 |
| M32 | bascule des clos sans effet | T-b9 |
| M33 | pré-vol R8 et garde W30 retirés (`unreject`, après le rebase sur la 28-5) | T-a27 (ligne `unreject`) |

### Project Structure Notes

- Fichiers neufs : `crates/kesh-db/migrations/20261013000001_bank_accounts_closed_on.sql` ;
  `crates/kesh-api/tests/bank_account_closed_inventory.rs` ; `crates/kesh-api/tests/bank_accounts_closed_e2e.rs` (ou
  extension de `bank_accounts_e2e.rs`, au choix du dev, écrit au Dev Agent Record) ; tests de dépôt dans
  `crates/kesh-db/tests/bank_accounts_repository.rs` ; E2E `frontend/tests/e2e/bank-accounts-closed.spec.ts`.
- Squash `crates/kesh-db/test-schema/0001_schema_squash.sql` : régénéré, jamais édité.
- Aucun fichier sous `docs/known-failures.md` / `docs/change_request.md` (archivés).

### References

- Issue #642 ; `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § 28-4.
- `crates/kesh-api/src/routes/bank_accounts.rs` (archivage l. 823-916, `PATCH` l. 930) ; `crates/kesh-db/src/repositories/bank_accounts.rs`
  (doc l. 3-14, `list_by_company` l. 172, `count_other_active_for_company` l. 764, sentinelle l. 869).
- Précédent de colonne : Story 26-2 (`26-2-description-compte-bancaire.md`, AC1) ; précédent de garde par proposition :
  Story 8-5b (`BANK_ACCOUNT_NOT_CONFIGURED` en `failed[]`) ; précédent de registre statique : `audit_route_registry.rs` ;
  précédent de route fermée ou ouverte aux clés : Story 27-1 (C-27-1-3) et `docs/api-external.md` § 4.
- `CLAUDE.md` § *Migration breaking policy* (P1-P8), § *Pattern batch*, § *Inventorier les sites NON RÉSOLUS*,
  § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Détail |
|---|---|---|
| 2026-10-13 | Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) | Sur `5aad8b35`. Deux clés (28-4a serveur, 28-4b écran) ; 17 AC ; inventaire W1-W30 et R1-R8 (W30, R8 : route `unreject` de la 28-5, ajoutée sur message de l orchestrateur) ; invariants I1-I3 ; tests T-a1-T-a35, T-b1-T-b14 ; mutations M1-M33 ; choix C-28-4-1 à C-28-4-13. Validation P1 à lancer (`28-4-validate-prompt-p1.md`). |
