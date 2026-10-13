# Story 28.4 : Clore un compte bancaire, en lecture seule, historique conservé

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur
`5aad8b35` (kickoff de l'Epic 28), worktree `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4`. Validation P1
faite (Sonnet, lentilles R et F : 4 HIGH, 9 MEDIUM, 12 LOW, trois convergences) et remédiée ; validation P2 faite
(Opus, lentilles R et F : 1 HIGH, 9 MEDIUM, 9 LOW) et remédiée par **simplification de la conception** (C-28-4-21 :
la garde ne verrouille plus, aucun verrou ajouté à aucun chemin) ; validation P3 faite (Sonnet, lentilles R et F :
R 0/1/2/5, F 0/1/2/6 en CRIT/HIGH/MED/LOW, le HIGH et les deux MEDIUM convergents) et remédiée par une **règle de placement** de la
garde (elle ne déplace jamais la naissance de la vue cohérente, AC7 ; annotation P3 de C-28-4-21) ; validation P4 faite
(Opus, lentilles R et F : R 0/0/3/9, F 0/0/2/7) et remédiée — L1 tracée par **#651**, garde de W24 après le comptage
(C-28-4-24), M54 rendue tuable (T-a27, T-a49) ; validation P5 à lancer : `28-4-validate-prompt-p5.md` (Sonnet, lentilles R et F).

⛔ **Deux stories, une fiche, deux PR successives** (C-28-4-1, sur le précédent C-26-2-15) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`28-4a-cloture-serveur`** | migration, entité, lectures, gestes *clore* et *rouvrir*, refus serveur sur **tous** les chemins d'écriture, refus de l'archivé aux gestes qui créent (AC6-bis), registre statique des écrivains, messages serveur (4 locales), étiquettes d'audit ; la ligne `### Corrigé` du CHANGELOG et la note d'`api-external.md` de l'AC6-bis | AC1 à AC10, AC6-bis | **`refs #642`** — ⛔ jamais `closes` ; et **`refs #651`** (L1, AC7) |
| **`28-4b-cloture-ecran`** | écran *Comptes bancaires*, réglages de facturation, libellé de refus de la réconciliation, i18n de l'interface, manuels (PDF), `docs/api-external.md`, CHANGELOG, captures clair/sombre | AC11 à AC17 | **`closes #642`**, et **`refs #651`** à côté — ⛔ jamais `closes #651` : la story ne ferme pas L1 |

**Ordre** : la 28-4a se développe **après le merge de la 28-5, de la 28-6 et de la 28-7** (fichier d'epic, § *Ordre* :
elles touchent `routes/reconciliation.rs`), ou en parallèle avec rebase — la 28-4a n'y écrit que **quatre pré-vols**
(AC6, sites R1 à R4, plus R8 sur la route que la 28-5 introduit ; l'appariement R5 est retiré en validation P1,
C-28-4-14). La 28-4b part de `origin/main` **après** la fusion de la 28-4a.
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
   clos** des écrivains (Dev Notes § *Inventaire*) ; **deux bornes, écrites** (validation P4, R3 = F-M2, R9 = F-L7) :
   la promesse vaut **hors de la fenêtre de concurrence L1** — une écriture en cours au moment de la clôture peut
   encore aboutir (AC7, tracée par [#651](https://github.com/guycorbaz/kesh/issues/651)) — et **sauf l'annulation d'un
   lot généré** (`cancel_batch`, laissée sans garde pour réparer ce que L1 a laissé passer, C-28-4-23) ;
4. le compte clos n'est plus **proposé** là où l'on écrit (sélecteurs) et reste **lisible** là où l'on lit (écran de
   gestion avec filtre, export, sauvegarde, historique des imports, écritures, rapports) ;
5. les manuels, les messages, le CHANGELOG ;
6. **extension décidée par l'orchestrateur en validation P1** (F3 = R12, C-28-4-12 réécrit) : les gestes qui
   **créent** une ligne rattachée à un compte bancaire refusent aussi un compte **archivé** (AC6-bis) — défaut voisin,
   préexistant, que la garde de la clôture rencontre aux mêmes endroits. Borné aux créations ; les
   annulations n'en sont **pas** (table *Dehors*). C'est la seule extension, et elle ne s'élargit plus.

**Dehors**, et pourquoi :

| Exclu | Raison |
|---|---|
| Geler le **compte comptable lié** (1020…) : écritures manuelles, lettrage, contre-passation | Le grand livre n'est pas le compte bancaire : une écriture de correction peut être nécessaire après la clôture à la banque. Le compte comptable s'archive par son propre geste (*Plan comptable*), qui existe. L'écran **avertit** d'un solde non nul (C-28-4-6), le serveur ne refuse pas. |
| Contrôler la date de clôture contre les dates des transactions, ou l'interdire dans le futur | La date est **informative** ; c'est l'état qui garde (C-28-4-3). Un contrôle de date ouvrirait des questions de fuseau (UTC du serveur contre date locale suisse) sans rien protéger. |
| Clôture automatique (relevé CAMT portant un solde nul, inactivité) | #642 demande un geste ; une clôture automatique serait un appariement qui décide seul (§ *Un appariement automatique propose, il ne crée jamais*). |
| Masquer, dans l'historique des imports ou des règlements, les boutons *Supprimer l'import* / *Annuler le règlement* d'un compte clos | Le serveur refuse avec un message traduit (AC6) ; masquer exigerait de porter l'état du compte dans trois payloads de lecture. Angle mort assumé, écrit au manuel (AC14). |
| Refuser un compte **archivé** aux **annulations** (W9, W24, W25, W28, W29, W30) et aux `accept_one_*` | ⛔ **Interdit**, non seulement exclu. Un compte archivé peut porter des règlements par virement et des lots (l'archivage ne compte que `bank_transactions`, `bank_accounts.rs:744-752`), et il **ne se rouvre pas** (aucune route de restauration, `archive_for_company` doc l. 614-617) : refuser l'annulation d'un règlement de ce compte la rendrait **définitivement impossible**. `ensure_open_in_tx`, que les annulations appellent, ne lit donc **que** `closed_on`. Les `accept_one_*` n'en ont pas besoin : un archivé n'a aucune transaction (l'archivage la refuse), et le pré-vol R1 le rend déjà `404` (C-28-4-12). |
| Lever le refus `412 BANK_ACCOUNT_HAS_TRANSACTIONS` de l'archivage | Écarté par #642 lui-même (« le sens d'archivé deviendrait flou ») : la clôture **remplace** cet usage. Seul le message du refus change (il propose la clôture, AC9). |
| Manuels DE / IT / EN | `docs/manual/{de,en,it}/` ne portent qu'un `README.md` : rien à traduire. |
| Cohérence d'une sauvegarde prise pendant une clôture | Relève de #633 (instantané commun, Story 28-3). Angle mort écrit à la doc de `close_in_tx`. |

**Décompte des modules** (§ *Règle de splitting préventif*) — `kesh-db` (migration, entité, **sept** dépôts :
`bank_accounts`, `bank_imports`, `invoice_settlements`, `invoice_settlements_write`, `reconciliation_cancel`,
`payment_batches`, `supplier_invoices`), `kesh-api` (`routes/bank_accounts`, `routes/bank_imports`,
`routes/reconciliation`, `exports`), `kesh-i18n`, frontend (`features/bank-accounts`, `routes/settings/invoicing`,
`features/reconciliation`), `docs/manual` : **plus de cinq**. D'où le découpage en deux (C-28-4-1). ⚠️ **La 28-4a
dépasse elle-même cinq modules** : c'est une **dérogation** à la règle, et non l'exception qu'elle prévoit (cycles
Cargo, merges intermédiaires impossibles à tester) — § *Dérogation* des Dev Notes. Son pattern est **un** appel de
garde posé à des points d'étranglement du dépôt, mécanique, et l'inventaire clos n'a de sens que **complet** : le
livrer en deux fois mettrait sur `main` un état « clos » qui ne tient pas sur une partie des chemins. La 28-4b compte
cinq modules — **la borne** de la règle, non un en-deçà (`kesh-i18n`, `docs/manual` compris ; `api-external.md` et le
CHANGELOG s'y ajoutent comme documents).

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
  aucun site du dépôt ne l'active : `grep -rn "set_ignore_missing\|ignore_missing: true" crates/` est **vide** ;
  `grep -rn "ignore_missing" crates/` rend **deux** lignes, deux tests qui **recopient** la valeur du `MIGRATOR` sans
  l'activer (`kesh-db/tests/migrations_upgrade_path.rs:56`, `kesh-db/tests/common/mod.rs:62`) — une 0.13.2 refuse de démarrer sur une
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
  l. 44, 46, 63, 64, 84, 115, 118, 119, 123, 131, 138, 141, 195, 202, 205, 274 — la l. 141, « avec `total = 80` »,
  ajoutée en validation P1). Les autres sites résolvent par version
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
  `closedOn` (`"AAAA-MM-JJ"` ou `null`) ; la macro `columns!()` (`repositories/bank_accounts.rs:36-40`, source unique
  des colonnes du module, doc l. 30-35) le porte — ce module n'a pas de constante `COLUMNS` (celle de
  `bank_imports.rs:30` est une autre table ; P3, F6).
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
- **Lectures inchangées, et qui doivent le rester** (un test chacune, sur un compte clos — T-a36 à T-a38, mutation
  M34) : `GET /bank-imports` (T-a36) et `GET /bank-imports/{id}` (T-a37) (aucun filtre d'archivage aujourd'hui) ;
  `GET /reconciliation/proposals?bankAccountId=` d'un compte clos rend `200` (liste vide par l'invariant I1, T-a38) —
  la garde d'archivage l. 610 n'est **pas** étendue ;
  l'export global (`bank_accounts.csv` gagne une colonne `closed_on` après `archived`, `csv_tables.rs:626-660`, valeur
  `AAAA-MM-JJ` ou vide) ; la sauvegarde `.keshbackup` (export générique `kesh_db::backup::export_table`, rien à coder :
  un test d'aller-retour vérifie qu'un compte clos revient clos, à la même date).
- `count_other_active_for_company` (l. 760-775) **exclut** les comptes clos (`AND closed_on IS NULL`) : un compte clos
  ne peut pas devenir principal, il ne doit donc plus empêcher l'archivage du compte principal (AC#9/AC#10 de la
  v014-1). ⚠️ **Conséquence, écrite au manuel et au message** (F9, C-28-4-19) : quand tous les autres comptes sont clos,
  le principal s'archive, et la société reste **sans principal** (QR-facture sans IBAN) ; le remède est de *rouvrir* un
  compte clos puis de le *désigner* principal (`PUT`). Le message `bank-accounts-errors-cannot-archive-primary` cesse
  de dire « tant qu'un autre compte non-archivé existe » (AC9). `first_active_bank_account_linked_to` (l. 820)
  **garde** les comptes clos (C-28-4-7) : le compte comptable d'un compte clos reste « lié à une banque » pour le refus
  de désignation comme compte de créance (mutation M44).

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
  compte. Chemin **neuf** : son seul verrou de ligne est celui de sa propre ligne, pris en première instruction, sans
  autre verrou tenu ni demandé ensuite (`audit_log` n'a aucune clé étrangère) — il ne peut pas fermer un cycle (AC7).

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
BANK_ACCOUNT_CLOSED`**, lu sur la ligne qu'ils verrouillent `FOR UPDATE`. Place dans l'ordre des refus :

- `PUT` et `PATCH` : immédiatement **après** le `409` de version de chacun, sur la ligne qu'ils verrouillent déjà (pas
  de seconde lecture, C-28-4-9). Le court-circuit no-op du `PATCH` (KF-004) **ne** dispense **pas** du refus : un
  `PATCH` identique sur un compte clos rend `409`, non `200`.
- `DELETE` (archivage) — **réécrit en validation P2** (C-28-4-21, qui remplace C-28-4-16) : la route garde **sa
  séquence au sol** (`routes/bank_accounts.rs:823-916` : pré-vol hors transaction → sentinelle `companies` → décompte
  des transactions → principal → `archive_for_company`), et ne gagne que deux lectures de `closed_on` **sur des
  instructions qui existent déjà** : (1) le **pré-vol** non verrouillant (`find_by_id_for_company`, l. 839-845, à côté
  de `existing.archived`) refuse un compte clos en `409 BANK_ACCOUNT_CLOSED` — **avant** la sentinelle et le décompte,
  si bien qu'un compte clos qui porte des imports (le cas ordinaire : on clôt un compte qui a servi) rend `409` et non
  le `412 BANK_ACCOUNT_HAS_TRANSACTIONS` dont le message propose… la clôture ; (2) le `SELECT … FOR UPDATE` existant
  d'`archive_for_company` (`repositories/bank_accounts.rs:619-623`) lit aussi `closed_on` et refuse en `409
  BANK_ACCOUNT_CLOSED`, **après** son contrôle de version (comme `PUT` et `PATCH`) — c'est le refus d'une clôture
  commitée entre le pré-vol et ce verrou, sur une ligne que le chemin verrouillait déjà. **Aucun verrou ajouté, ordre
  inchangé.** Ordre des refus : `400` → `404` (pré-vol, archivé compris) → `409 BANK_ACCOUNT_CLOSED` (pré-vol) →
  sentinelle → `412 BANK_ACCOUNT_HAS_TRANSACTIONS` → `412 BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY` → `404` / `409
  OPTIMISTIC_LOCK_CONFLICT` → `409 BANK_ACCOUNT_CLOSED` (sous verrou, course seulement). La limite L5 de la route
  (`routes/bank_accounts.rs:853-856`, l'import ne prend pas la sentinelle) **reste telle quelle** : la story ne la
  ferme pas.

L'archivage d'un compte clos est donc refusé (il faut le rouvrir d'abord, C-28-4-10).

**AC6 — Tous les chemins d'écriture refusent un compte clos, côté serveur.**

Le refus est `409 BANK_ACCOUNT_CLOSED` (`details: { "bankAccountId" }`), sauf dans les lots, où il est par proposition
(§ *Pattern batch*). Il est posé aux **points d'étranglement du dépôt** — toute route, clé d'API comprise, qui y passe
est couverte — et doublé, là où la route lit déjà le compte en pré-vol, d'un refus précoce. ⛔ **Depuis la validation
P2 (C-28-4-21), aucune garde ne prend de verrou** : là où le chemin verrouille **déjà** la ligne du compte au sol (W8,
W27, W13-W15), `closed_on` se lit dans cette instruction ; partout ailleurs, la garde est une **lecture cohérente non
verrouillante** (`bank_accounts::ensure_open_in_tx`, ou `ensure_open_unarchived_in_tx` pour W22), dans la transaction de
l'écrivain — ni `FOR UPDATE`, ni `LOCK IN SHARE MODE` —, posée **après le dernier verrou que le chemin prend avant
sa première lecture ordinaire — ou, s'il n'en a aucune hors l'audit, avant sa première écriture** (règle de placement
de l'AC7, validation P3, clause complétée en validation P4, F-L2 : elle ne déplace jamais la naissance de la vue
cohérente). Critère et preuve : AC7. ⚠️ **Exception à « toute écriture refusée »** (C-28-4-23, R9 = F-L7) :
l'annulation d'un lot généré (`POST /payment-batches/{id}/cancel`, W21) reste permise sur un compte clos. Table
complète au § *Inventaire*
(sites W1 à W29, et W30 que la 28-5 introduit) ; résumé :

| Route | Garde | Refus |
|---|---|---|
| `POST /bank-imports/preview` | pré-vol route (R6, à côté de `archived`, `bank_imports.rs:965`) | `409` |
| `POST /bank-imports` (CAMT et CSV) | pré-vol (R7, l. 1114) **et** `bank_imports::create_with_transactions` (W22-W23) | `409` |
| `DELETE /bank-imports/{id}` | `bank_imports::delete_unreconciled_in_tx` (W24) : lecture non verrouillante **après le comptage de l'étape 3** (sur le résultat de l'étape 2, transactions de l'import verrouillées) **et avant le `DELETE` de l'étape 4** — ⛔ **déplacée en validation P3** (R2 = F2 : posée avant l'étape 2, elle ouvrait la vue avant le verrou, contre le doc-comment de la fonction, `bank_imports.rs:389-391`, « aucune lecture ordinaire avant le premier verrou », et l. 420-425, « aucune vue n'est ouverte avant les verrous »), **puis après le comptage en validation P4** (R1, C-28-4-24). **Ordre des refus** : `404` (import absent, étape 1) → `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` (étape 3, refus **du sol**) → `409 BANK_ACCOUNT_CLOSED` (garde) → suppression. Un import qui porte une transaction rapprochée rend donc le refus du sol, compte clos ou non — par I1, c'est **tout** import non vide d'un compte clos hors L1 ; le refus « compte clos » ne vaut que pour un import **sans** transaction rapprochée (vide, ou laissé par L1). Sans cet ordre, « rouvrez-le d'abord » envoyait l'utilisateur vers un geste qui ne menait qu'au refus suivant | `409` (l'un ou l'autre code) |
| `POST /reconciliation/accept` | pré-vol (R1, à côté de l. 1040) → **chaque** proposition en `failed[]`, code `BANK_ACCOUNT_CLOSED`, `200`. ⛔ **Aucune garde** dans `accept_one_*` (C-28-4-14) : l'invariant I1 suffit — la transaction acceptée reste `pending` dans l'**état commité** jusqu'au commit du lot (elle n'est verrouillée qu'à son `UPDATE`, `reconciliation.rs:2038`, après l'`INSERT invoice_settlements`), si bien que le décompte de la clôture la voit `pending` et refuse (validation P3, F3). L'appariement R5 est **retiré** | `200` + `failed[]` |
| `POST /reconciliation/reject` | pré-vol (R2, l. 3174) → chaque transaction en `failed[]` | `200` + `failed[]` |
| `POST /reconciliation/manual` | pré-vol (R3, l. 3564) | `409` |
| `POST /reconciliation/split` | pré-vol (R4, l. 4007) | `409` |
| `POST /reconciliation/transactions/{id}/cancel` | `reconciliation_cancel::cancel_in_tx` (W9), lecture non verrouillante sur `bt.bank_account_id`, **après l'étape (3)** (écriture et exercice verrouillés), avant les motifs de l'étape (4) — ⛔ **déplacée en validation P3** (R1 = F1 : avant (3), elle figeait la vue avant l'attente du verrou d'exercice, ce qu'interdit `reconciliation_cancel.rs:247-252` et `:296`) ; pour un rapprochement de facture, l'étape (6) appelle W25, qui relit le compte de son règlement (seconde lecture non verrouillante, dans la vue déjà née en (4)) | `409` |
| `POST /reconciliation/transactions/{id}/unreject` — **introduite par la 28-5, à garder au rebase** (absente de `5aad8b35`) | pré-vol R8 + garde non verrouillante dans sa transaction (W30), après le verrou de la ligne `bank_transactions` ; ⛔ R8 se place **avant** la classification « pas rejetée » de la route (la fiche 28-5 lit la transaction, puis classe) : sur un compte clos, c'est la clôture qui doit répondre. Si la 28-5 n'est pas mergée au moment du dev, la ligne reste à faire et se dit au Dev Agent Record — la 28-4a ne dépend pas de son code | `409` |
| `POST /invoices/{id}/settlements` (virement) | `settle_invoice` (W8) : `closed_on` lu **dans** le `SELECT … FOR UPDATE` du compte qui existe déjà (`invoice_settlements_write.rs:131-135`, étape (3)) — **et non** dans `invoice_settlements::create_in_tx`, que `accept_one_invoice` appelle après avoir pris l'exercice (C-28-4-14) | `409` |
| `POST /invoices/{id}/settlements/{sid}/cancel` | `invoice_settlements_write::cancel_settlement_in_tx` (W25), si `settlement_bank_account_id` est non nul — colonne **ajoutée** au `SELECT` verrouillé de l'étape (2) (l. 835-836, qui ne la lit pas aujourd'hui ; même ligne, verrou inchangé), puis garde non verrouillante sur ce compte, **après (2-bis)** (écriture et exercice verrouillés), avant les motifs de (3) — déplacée en validation P3 (R1 = F1 ; `invoice_settlements_write.rs:858-861`) | `409` |
| `POST /supplier-invoices/{id}/pay` (virement) | `supplier_invoices::pay_in_tx` (W27), dans le `SELECT … FOR UPDATE` existant (l. 753-756, étape (3), avant l'exercice de l'étape (4), l. 842-843 — « (5) » corrigé en validation P4, F-L5) | `409` |
| `POST /supplier-invoices/{id}/settlement/cancel` | `supplier_invoices::cancel_settlement_in_tx` (W29), si `inv.settlement_bank_account_id` est non nul (ligne verrouillée à l'étape (1)), garde non verrouillante **après (1-bis)** (l. 1290-1312, écriture de règlement et exercice verrouillés), avant les motifs de (2) (l. 1314) — déplacée en validation P3 (R1 = F1) | `409` |
| `POST /supplier-invoices/{id}/cancel` (facture **payée** par virement) — **ajoutée en validation P1** (F2 = R1) | `supplier_invoices::cancel_in_tx` (W28) annule aussi une facture payée et remet `settlement_bank_account_id` à `NULL` (l. 1131-1134) : garde non verrouillante si `inv.settlement_bank_account_id` est non nul, **après** le verrou facture (l. 1061) **et** celui de l'écriture d'achat et de son exercice (l. 1070-1080), avant les motifs de (2) (l. 1082) — déplacée en validation P3 (R1 = F1 ; `supplier_invoices.rs:1040-1045`, « les verrous d'abord, tous, avant toute lecture non verrouillante ») | `409` |
| `POST /payment-batches` | `payment_batches::create_batch` (W17-W19) : `closed_on` lu dans la lecture non verrouillante **existante** de l'étape (0) (l. 85-99, à côté de `archived`) — la relecture verrouillée de la P1 est **retirée** (C-28-4-21) | `409` |
| `POST /payment-batches/{id}/confirm` | transitif : chaque facture passe par `pay_in_tx` (W27) ; la lecture non verrouillante existante de l'étape (2) (l. 427-441) gagne le refus précoce à côté de `archived` | `409` |
| `PUT` / `PATCH` / `DELETE /bank-accounts/{id}` | AC5 (W13-W15) | `409` |

- `DbError::BankAccountClosed { bank_account_id: i64 }` (nouveau, `crates/kesh-db/src/errors.rs`) et
  `AppError::BankAccountClosed { bank_account_id }` (nouveau, que produisent les **pré-vols** de route) → `409
  BANK_ACCOUNT_CLOSED`, message traduit `bank-accounts-errors-closed`. ⚠️ **Où naît le `409`** (validation P4, R11 =
  F-L4) : les chemins de dépôt remontent l'erreur par `?`, `e.into()` ou `AppError::Database(e)` — `errors.rs:165`,
  `Database(#[from] DbError)` ; les routes de réconciliation par `ReconciliationError::Db(db) => AppError::Database(db)`
  —, si bien qu'ils produisent `AppError::Database(DbError::BankAccountClosed { … })`, **jamais**
  `AppError::BankAccountClosed`. Le `409` naît donc d'un **bras** `DbError::BankAccountClosed { bank_account_id }`
  **dans le `match` d'`AppError::Database(db_err)`** (`errors.rs:2905` sur `5aad8b35`, à côté de `DbError::NotFound`
  l. 2917 et `DbError::OptimisticLockConflict` l. 2922 ; le `match` est exhaustif, le compilateur l'imposera), et du
  bras `AppError::BankAccountClosed { bank_account_id }` des pré-vols ; ⛔ **les deux bras appellent une seule
  fonction de rendu** (`bank_account_closed_response(bank_account_id)`, même corps, même `details.bankAccountId`,
  DRY) — deux rendus écrits à la main dériveraient. T-a27 l'exerce par les deux voies (pré-vol et dépôt).
- **Un test HTTP par ligne de la table** (`bank_accounts_closed_e2e.rs`, AC10), sur un compte clos **après** avoir servi
  (import rapproché, règlement, lot confirmé) : le refus, **et l'absence d'écriture** (compte des lignes avant/après,
  aucune trace d'audit de l'opération refusée). Le même jeu rejoué sur le compte **rouvert** passe : c'est ce qui
  prouve que le refus vient de la clôture et non d'une autre garde.
- ⚠️ **Lignes dont l'état ne se construit pas par le flux HTTP** (R6) : la clôture refuse tant qu'une transaction est
  `pending` (I1, rejetées comprises) ou qu'un lot est `generated` (I2). Les lignes suivantes posent donc leur état
  **puis** `UPDATE bank_accounts SET closed_on = … WHERE id = …` **par SQL direct** (invariant cassé à dessein, écrit
  en tête du test), et rouvrent par la route : `DELETE /bank-imports/{id}` (un import non rapproché porte des
  `pending` ; le second cas de cette ligne, l'import **rapproché** d'un compte clos, se construit lui par l'API —
  import, rapprochement de toutes ses transactions, clôture —, validation P4, R1) ; `accept`, `reject`, `manual`, `split` (une transaction à rapprocher est `pending`) ; `unreject` (une
  rejetée est `pending`) ; `POST /payment-batches/{id}/confirm` (un lot `generated`). Toutes les autres lignes se
  construisent par l'API (import, règlement, paiement, lot créé puis confirmé, rapprochement à annuler).
- Les autres écrivains de l'inventaire sont **hors d'atteinte par invariant** (I1, I2, I3) ou **ne portent pas de compte
  bancaire** : chacun est classé, avec sa raison, au registre statique (AC8).

**AC6-bis — Les gestes qui créent refusent aussi un compte archivé (défaut voisin, intégré en validation P1).**

Décision de l'orchestrateur (F3 = R12) ; C-28-4-12 réécrit. Défaut **préexistant** (P2 : une opération acceptée sur un
compte retiré de l'usage) : `settle_invoice` (`invoice_settlements_write.rs:131-135`) et `pay_in_tx`
(`supplier_invoices.rs:753-756`) lisent le compte sans contrôler `archived` ; `create_batch` et l'import le contrôlent en
lecture **non** verrouillante seulement (étape (0) de `payment_batches.rs:85-99`, pré-vols R6/R7) — et c'est ce qu'ils
**continuent** de faire après la validation P2 (C-28-4-21) : la story ne ferme pas la course archivage / création
(L5 de la route), elle ferme le règlement vers un archivé.

| Site | Lecture verrouillée | Archivé → erreur **existante** du chemin |
|---|---|---|
| W8 `settle_invoice` | `SELECT journal_account_id, closed_on, archived FROM bank_accounts … FOR UPDATE` (l'instruction de l'étape (3), étendue) | `DbError::NotFound` — celle que rend aujourd'hui un compte inexistant ou d'une autre société (l. 141) |
| W27 `pay_in_tx` | même extension de l'instruction de l'étape (3) (l. 753-756) | `DbError::NotFound` (l. 763) |
| W17-W19 `create_batch` | **aucun changement** (validation P2) : l'étape (0) refuse déjà l'archivé, en lecture non verrouillante, au sol | `DbError::IllegalStateTransition("le compte bancaire source est archivé")` (l. 95-98), inchangé |
| W22-W23 `create_with_transactions` | la garde de tête (`ensure_open_unarchived_in_tx`, **non verrouillante**) lit aussi `archived` | `DbError::NotFound`, que la route d'import **devra apparier**, aux **deux** appels — `routes/bank_imports.rs:1312` (CAMT) **et** `:2199` (CSV, `create_csv`) — par `.map_err(\|e\| match e { DbError::NotFound => AppError::BankAccountNotFound, e => e.into() })` : au sol, les deux font `?`, et `DbError::NotFound` deviendrait `404 NOT_FOUND` « Ressource introuvable » (`errors.rs:165`, `2917-2921`), non le `404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` du pré-vol R7. L'appariement est sûr : au sol, `create_with_transactions` ne rend `NotFound` que par le `fetch_one` de relecture de l'entête (l. 133-139, `RowNotFound` → `DbError::NotFound` par `map_db_error`, `errors.rs:1455`), sur une ligne insérée dans la même transaction — **inatteignable** ; sa doc (l. 52-58) ne nomme que `ForeignKeyViolation`. Hors ce cas, seule la garde neuve le produit (phrase corrigée en validation P3, R5 = F4 : « ne rend jamais `NotFound` » était faux à la lettre) |

- **Ordre dans la lecture** : absent, autre société ou archivé → l'erreur existante du chemin ; puis clos → `409
  BANK_ACCOUNT_CLOSED` ; puis (W8, W27) `journal_account_id` nul → `ConfigurationRequired`. Un compte ne peut pas être
  à la fois clos et archivé par l'API (la clôture refuse l'archivé, l'archivage refuse le clos) ; l'ordre ne vaut que
  pour une base restaurée ou forcée.
- **Deux fonctions de garde, et non une** — renommées en validation P2 (elles ne verrouillent plus, C-28-4-21) :
  `bank_accounts::ensure_open_in_tx` (annulations W9, W24, W25, W28, W29, W30) lit `closed_on` **seul** ;
  `bank_accounts::ensure_open_unarchived_in_tx` (W22) lit `closed_on, archived` et rend `DbError::NotFound` pour
  l'archivé, que l'appelant traduit. Toutes deux par une lecture cohérente **non verrouillante**, SQL tenu dans une
  constante (`ENSURE_OPEN_SQL`, `ENSURE_OPEN_UNARCHIVED_SQL`) que T-a45 contrôle. W8, W27, W13-W15, W17 et W20
  n'appellent ni l'une ni l'autre : ils étendent une instruction existante. ⛔ **Interdit, écrit aux deux
  doc-comments** : faire lire `archived` à `ensure_open_in_tx`, ou appeler la variante « unarchived » depuis une
  annulation (table *Dehors* : un archivé ne se rouvre pas) ; et y ajouter un verrou (AC7).
- **Ce que cela ne ferme pas** : une course archivage / import ou archivage / création de lot reste ce qu'elle est au
  sol (L5 de la route) — les lectures d'`archived` de W17 et W22 sont non verrouillantes. Séquentiellement, W22 refuse
  l'archivé au dépôt (T-a46), derrière le pré-vol R7.
- **Aucune clé i18n, aucun code d'erreur neuf** : les erreurs existent, avec leurs messages. Le seul code neuf est
  l'appariement des deux appels de la route d'import (ci-dessus).
- `### Corrigé` du CHANGELOG (`## [0.13.3] — Non publié`, créé ou complété par **union**) — **dans la PR de la
  28-4a** (règle d'inclusion du `CLAUDE.md` : la doc va avec le code qui la motive) : « **Un compte bancaire archivé ne
  reçoit plus de règlement.** Un règlement de facture ou un paiement fournisseur par virement vers un compte archivé
  étaient acceptés ; ils sont refusés comme vers un compte inexistant. L'annulation d'un règlement ancien de ce compte
  reste possible. » (Réécrit en validation P2 : la mention « un lot de paiement ou un import créés pendant son
  archivage » est retirée — la course n'est plus fermée, C-28-4-21.) Et `docs/api-external.md`, § 7, une
  note « *Depuis la v0.13.3* » à la ligne des règlements (l. 464 et 514 ; à relire au rebase) : même contenu, codes
  rendus.
- Tests T-a42, T-a43, T-a46, T-a47 ; mutations M46 à M48 et M53 (M49 et M50 sans objet depuis la validation P2).

**AC7 — Concurrence : la story n'ajoute aucun verrou, ne déplace la naissance d'aucune vue, et nomme ce qu'elle laisse passer.**

*Réécrit en validation P2 (C-28-4-21, qui remplace C-28-4-16 et C-28-4-17 et rend sans objet la partie « ordre des
verrous » de C-28-4-14).* La P1 avait fait prendre à la garde un verrou exclusif sur la ligne du compte ; la P2 a
montré que ce verrou, combiné aux verrous partagés que les `INSERT` à clé étrangère posent sur `companies`, ouvrait
des cycles neufs avec les routes de gestion et avec `unreject` (28-5), et que chaque correctif en ouvrait un autre —
le recyclage que désigne l'amendement D5. La conception est donc simplifiée.

- **Critère décidable** : ⛔ *la story n'ajoute aucun verrou de ligne, explicite (`FOR UPDATE`, `LOCK IN SHARE MODE`,
  `FOR SHARE`) ou implicite par clé étrangère (aucun `INSERT` ni `UPDATE` de colonne à clé étrangère neuf), à aucun
  chemin qui existe sur `origin/main` (`5aad8b35`), et ne change l'ordre des verrous d'aucun de ces chemins* ;
  **et**, second volet ajouté en validation P3 (R1 = F1), ⛔ *aucun déplacement de la naissance de la vue : la première
  lecture non verrouillante de chaque chemin reste celle du sol — ou la garde, posée immédiatement avant elle, sans
  aucun verrou entre les deux ; sur un chemin qui n'en a aucune au sol, la garde suit le dernier verrou qu'il prend
  avant sa première écriture.* ⚠️ **Borne du second volet** (validation P4, R4) : les deux sous-`SELECT` sur `users`
  de l'`INSERT` d'`audit_log::insert_in_tx` (`audit_log.rs:94-95`) **ne comptent pas** comme « première lecture
  ordinaire » — c'est la dernière instruction de chaque chemin, aucune décision n'en dépend, et leur caractère
  verrouillant n'est pas établi ; sans cette exclusion, le volet ne serait pas décidable pour W24 et W30, dont c'est
  la seule lecture possible au sol. « Aucune », dans la table, se lit donc « aucune hors l'audit ». Le premier volet tient les cycles, le second la fraîcheur de ce que le chemin relit
  après ses attentes : sous `REPEATABLE READ`, une lecture ordinaire posée avant un verrou fige l'instantané **avant
  l'attente**, et le chemin relirait ensuite un exercice « ouvert » qu'une clôture concurrente vient de fermer —
  une erreur comptable. La preuve est la table *Position par chemin* (Dev Notes § *Concurrence*) : une ligne par
  chemin, les verrous au sol, ce que la story ajoute, et les trois réponses « verrou ajouté ? », « ordre changé ? »
  — **non** sur chaque ligne — et « vue déplacée ? » (première lecture non verrouillante au sol, puis avec la story)
  — **non** sur chaque ligne. La table se **recopie au Dev Agent Record** avec la ligne de code retenue (T6), et la revue de code la vérifie
  chemin par chemin contre le diff.
- **Règle de la garde** (doc-comment de `bank_accounts::ensure_open_in_tx`) : là où le chemin verrouille **déjà** la
  ligne du compte au sol, `closed_on` (et `archived`, AC6-bis) se lit **dans cette instruction**, élargie de colonnes
  (W8 `settle_invoice` l. 131-135, W27 `pay_in_tx` l. 753-756, W13 l. 413, W14 l. 548, W15 `archive_for_company`
  l. 619-623) ; là où le chemin lit déjà la ligne sans verrou (W17 étape (0), W20 étape (2), pré-vols R1-R4, R6, R7 et
  celui de l'archivage), `closed_on` s'ajoute à cette lecture ; **R8 est une lecture neuve** (validation P4, R6 : la
  route `unreject` de la 28-5 ne lit jamais `bank_accounts`, elle lit `bank_account_id` sur `bank_transactions`) —
  sur le pool, hors transaction, donc sans effet sur le critère ; ailleurs (W9, W22, W24, W25, W28, W29, W30), la garde
  est une **lecture cohérente non verrouillante** (`SELECT closed_on FROM bank_accounts WHERE company_id = ? AND
  id = ?`), dans la transaction de l'écrivain.
- ⛔ **Règle de placement** (validation P3, R1 = F1 ; annotation P3 de C-28-4-21 ; clause finale ajoutée en validation
  P4, F-L2, annotation P4) : *chaque garde se pose **après le dernier verrou que le chemin prend avant sa première
  lecture ordinaire — ou, si le chemin n'en a aucune hors l'audit, après le dernier verrou qu'il prend avant sa
  première écriture, et avant cette écriture***, de sorte que la story ne déplace jamais la naissance de la vue
  cohérente. Sa position ne règle aucun ordre de **verrous** (elle n'en prend pas) ; elle règle le
  moment où naît la **vue**, et c'est ce que le sol garde. Les doc-comments du sol qui l'imposent, à l'endroit même des
  gardes : `reconciliation_cancel.rs:247-252` et `:296` (W9 : « aucune lecture non verrouillante avant l'étape 3 ») ;
  `supplier_invoices.rs:1040-1045` (W28 : « les verrous d'abord, tous, avant toute lecture non verrouillante »,
  l. 1040) et `:1290-1294` (W29, une autre raison, de même effet : « la queue lit l'exercice SANS verrou, et le socle
  ne verrouille l'exercice de l'origine qu'APRÈS », d'où le verrou (1-bis) — attribution corrigée en validation P4,
  R10) ; `invoice_settlements_write.rs:858-861` (W25 : l'instantané « ne se fige qu'à sa
  première lecture non verrouillante, qui vient APRÈS » le verrou (2-bis)) ; `bank_imports.rs:389-391` et `:420-425`
  (W24 : « aucune lecture ordinaire avant le premier verrou », « aucune vue n'est ouverte avant les verrous »).
  Positions qui en découlent : W9 entre (3) et (4) ; W25 entre (2-bis) et (3) ; W28 après l'écriture d'achat et son
  exercice (l. 1070-1080), avant (2) ; W29 après (1-bis), avant (2) ; W24 **après le comptage de l'étape 3, avant le
  `DELETE` de l'étape 4** (validation P4, R1, C-28-4-24 : entre l'étape 2, dernier verrou avant la première écriture,
  et la garde, il n'y a que le comptage **en mémoire** — ni verrou, ni lecture —, la clause finale de la règle est donc
  tenue ; et le refus du sol, `HasReconciled`, passe avant elle) ; W30 après le verrou de sa ligne `bank_transactions`
  (le dernier du chemin) ; W22 **en tête de la fonction, avant l'`INSERT` d'entête** (sa position ne change pas la
  vue, déjà née dans la route, `routes/bank_imports.rs:1173` CAMT et `:1978` CSV ; le chemin ne prend aucun verrou
  explicite — « n'importe où » corrigé en validation P4, R5). La garde reste avant la première écriture du chemin —
  toutes ces positions le sont. ⚠️ La phrase de la P2 « sa position ne règle
  plus aucun ordre ; elle se place au plus tôt » est **retirée** : vraie des verrous, elle était fausse de la vue, et
  « au plus tôt » est exactement la position que le sol interdit.
- **Les deux chemins neufs** (clôture, réouverture) ne sont pas au sol : leur seul verrou est celui de **leur** ligne,
  en première instruction, sans autre verrou tenu ni demandé ensuite (lectures non verrouillantes, `UPDATE` de la même
  ligne, `audit_log` sans clé étrangère). Une transaction qui ne demande qu'un verrou, en tenant rien, ne ferme aucun
  cycle. Deux précisions, sans lesquelles l'argument n'est pas complet (validation P3, R7 = F7) : (a) l'`INSERT` de
  `audit_log::insert_in_tx` remplit `actor_label` et `company_id` par deux sous-`SELECT` sur `users`
  (`audit_log.rs:94-95`), dont le caractère non verrouillant n'est pas garanti — « un seul verrou » ne vaut donc qu'à
  `users` près ; inoffensif (personne ne tient `users` en exclusif en attendant `bank_accounts`), et identique à tous
  les chemins audités ; (b) la demande `FOR UPDATE` de la clôture, **en attente** derrière un verrou partagé de clé
  étrangère (import, lot, règlement en cours), fait attendre derrière elle toute demande partagée ultérieure sur la
  même ligne (file d'attente InnoDB) : elle ajoute des **arêtes d'attente** aux chemins existants, sans en tenir. La
  propriété est celle de tout `X` sur cette ligne au sol (W8, W27, `PUT`, `PATCH`, `DELETE`) — pas une classe
  neuve ; aucune paire de chemins qui en fasse un cycle n'a été trouvée (aucun écrivain qui tient un `S` sur le
  compte ne demande ensuite un verrou que tiendrait un second demandeur de `S`).
- **Ce qui sérialise encore, et qui est au sol** : un écrivain qui a **inséré** une ligne rattachée au compte (entête
  et transactions d'import, lot, règlement) tient, par la clé étrangère, un verrou partagé sur la ligne du compte
  jusqu'à son commit ; le `FOR UPDATE` de la clôture l'attend, puis ses décomptes voient ce qu'il a commité (T-a31).
  De même W8, W27, W13-W15 lisent le compte sous leur verrou existant, donc la valeur commitée la plus récente : une
  clôture commitée avant leur verrou les fait refuser, sans fenêtre.
- **L1 — angle mort assumé** (C-28-4-21 ; doc-comment d'`ensure_open_in_tx`, manuel utilisateur AC14) : une écriture
  dont la transaction a pris sa **vue cohérente** avant le commit d'une clôture peut encore aboutir sur le compte
  qu'on clôt — en REPEATABLE READ, sa lecture non verrouillante lit cet instantané antérieur. Chemins concernés : ceux
  dont la garde est non verrouillante (W9, W17, W22, W24, W25, W28, W29, W30). Conséquence : **une opération réelle de
  plus** sur un compte clos — un import (ses transactions `pending`, ce qui défait I1 pour ce compte), un lot
  `generated` (défait I2), une annulation de rapprochement ou de règlement (une contre-passation vraie), une
  suppression d'import, une annulation de rejet. **Aucune erreur comptable** : l'opération est vraie, et elle aurait
  été acceptée une seconde plus tôt. Remède à la main : rouvrir le compte, traiter l'opération (rapprocher ou supprimer
  l'import, confirmer ou annuler le lot — `cancel_batch`, W21, n'est pas gardé, C-28-4-23), clore à nouveau.
  **Classe P2 par la grille** (validation P3, R6 = F8 ; annotation P3 de C-28-4-21) : « comportement faux sans effet
  sur les comptes » — une écriture acceptée sur un compte que #642 demande de rendre en lecture seule. La « P3 au
  plus » de la P2 était un arbitrage, non une application de la grille. ⛔ **Dette assumée et motivée** : la fermer
  exige une garde verrouillante, c'est-à-dire la conception P1 et les cycles qu'elle a ouverts (C-28-4-21,
  *Écartées*) ; la fenêtre est étroite, l'opération qui passe est vraie, aucune erreur comptable n'en naît, et le
  remède est à la main de l'utilisateur. ⛔ **Tracée par [#651](https://github.com/guycorbaz/kesh/issues/651)**
  (`bug`, `P2`, ouverte par l'orchestrateur en validation P4 — R3 = F-M2 —, avec la piste du rejeu) : c'est l'issue,
  non la fiche, qui porte le défaut après le merge (Issue Tracking Rule) ; la 28-4a et la 28-4b la citent en
  `refs #651`, jamais en `closes`. Elle est écrite, avec son numéro, au doc-comment d'`ensure_open_in_tx`, au manuel
  utilisateur (AC14) et à la note *Depuis la v0.13.3* de `docs/api-external.md` (AC16, pour les intégrations par clé
  d'API, qui ne lisent pas le manuel). **Conséquence pour #642** : sa promesse « toute écriture qui le viserait est
  refusée côté serveur » vaut **hors de cette fenêtre** — l'AC14, le CHANGELOG (AC16) et le corps de la PR de la
  28-4b le disent. Fenêtre : une clôture commitée pendant qu'une requête d'écriture est **en cours** sur le
  même compte — pour une PME, deux onglets au même instant ; pour l'**import**, elle couvre aussi l'**analyse du
  fichier** (la vue naît à la recherche de doublons, `routes/bank_imports.rs:1173` CAMT et `:1978` CSV, avant
  l'analyse ; R6 c) : sur un gros relevé, « quelques instants » est un minimum, non une borne.
- **`accept_one_invoice`** (C-28-4-14) : **aucune garde**, donc aucun verrou pris par la story ; il lit le compte en
  non-verrouillant (`reconciliation.rs:1758-1760`), et son `INSERT invoice_settlements`
  (`settlement_bank_account_id`, `fk_invoice_settlements_settlement_bank`) pose un verrou **partagé implicite** sur la
  ligne du compte, après l'exercice — **au sol**, inchangé (F2 de la P2). L'invariant I1 le tient hors d'atteinte : la
  transaction qu'accepte le lot reste `pending` **dans l'état commité** jusqu'au commit du lot — elle n'est pas tenue
  par un verrou en amont (lue sans verrou, `reconciliation.rs:1563` ; verrouillée seulement par son `UPDATE`,
  l. 2038, après l'`INSERT invoice_settlements`) — ; le décompte de la clôture, qui lit l'état commité, la voit
  `pending` et refuse. *(Phrase corrigée en validation P3, F3 : la P2 attribuait l'invariant à un « verrou de ligne »
  qui n'est pas pris à ce moment.)* Et quand **L1 a défait I1** pour un compte (un import ou une annulation de
  rapprochement passés pendant la clôture), ce n'est plus l'invariant qui protège W1-W6 et `accept_one_invoice`, mais
  le **pré-vol** R1-R4, lecture fraîche en autocommit sur le pool, qui voit le compte clos et refuse (validation P4,
  R7 — le raisonnement même qui a fait reclasser W21, C-28-4-23).
- **Inversion `companies` → `bank_accounts`, préexistante et inchangée** : W8 et W27 tiennent le compte en exclusif
  puis insèrent des lignes à clé étrangère vers `companies` (verrou partagé), alors que `POST`, `PUT` et `DELETE
  /bank-accounts…` prennent la sentinelle `companies` puis le compte (ou, pour `PUT` / `POST` avec `isPrimary`, les
  comptes de la société par `flip_primary_off_for_company`). Ce cycle est **au sol** ; la story ne l'étend à aucun
  chemin (aucune garde verrouillante) et ne le traite pas. Les autres cycles préexistants que la P2 a nommés
  (`create_batch` / `confirm_batch`, W9 « écriture » / W8 sur `uq_invoice_settlements_entry`) sont au sol, inchangés.
- **Tests de concurrence** : **T-a31** pour le premier volet (clôture contre import qui a inséré : la clôture attend sur le verrou
  partagé de clé étrangère, puis rend `BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` ; motifs `["FROM bank_accounts",
  "archived = FALSE FOR UPDATE"]`, première instruction de `close_in_tx` ; `attendre_une_requete_en_cours`,
  `test_fixtures.rs:561`, qui filtre `INFO LIKE '%motif%'` et panique après 10 s) ; **T-a45**, statique : les SQL des
  deux fonctions de garde ne portent aucun verrou ; et, pour le second volet du critère (validation P3), **T-a48**
  (une clôture d'exercice concurrente, par virement, contre W25, W28 et W29 : rouge si la garde est avancée avant le
  verrou d'exercice — tue M37 à M39) **et** le test existant `a_concurrent_close_waits_for_the_unreconciliation`
  (`crates/kesh-db/tests/reconciliation_cancel.rs:128`), qui **change de sens** sans changer de texte : sa transaction
  porte un compte bancaire (`monter`, l. 29-115 ; compte inséré l. 79-86), la garde de W9 y passe donc toujours, et une garde avancée avant
  (3) le ferait rougir (tue M36). Les trois tests d'entrelacement existants des autres chemins
  (`invoice_settlement.rs:1266`, `supplier_invoices_repository.rs:1714` et `:2059`) règlent par **compte interne**
  (`settle_cash`, `invoice_settlement.rs:438` ; `paid_invoice`, `supplier_invoices_repository.rs:1186`) : la garde n'y passe pas, ils ne voient pas sa position — d'où T-a48. Pour W24 (validation P4, R1 et F-M1), la position se tient **deux fois** : séquentiellement, par le cas « import rapproché d'un compte clos » de T-a27 (toute garde posée avant le comptage rend `BANK_ACCOUNT_CLOSED` au lieu du refus du sol : tue M54 et M55) ; et, pour la seule fraîcheur de la vue, par **T-a49** (entrelacement sur le verrou de trou de l'étape 2, montage de F-M1 : tue M56). Retirés en validation P2, faute d'objet : T-a32 (un import attendait
  la garde d'une clôture), T-a39 (création de lot contre clôture), T-a40 (sérialisation exclusive), T-a41
  (non-interblocage selon la position de la garde), T-a44 (archivage contre import).
- Les routes neuves sont classées `SansEcritureAuJournal` au registre des routes (aucune enveloppe de rejeu) : elles
  ne peuvent pas être victimes d'un cycle (point précédent). Aucune route existante ne gagne ni ne perd d'enveloppe.

**AC8 — Le registre statique des écrivains : l'inventaire clos se tient tout seul.**

Test neuf `crates/kesh-api/tests/bank_account_closed_inventory.rs`, patron de `audit_route_registry.rs` : il lit les
sources **hors `#[cfg(test)] mod …`** et **hors commentaires** (`//`, `///`, `//!` — sept occurrences en commentaire
sur `5aad8b35`, dont `reconciliation.rs:1736, 2535, 2988, 3518, 3797, 4255` et `bank_accounts.rs:796` ; le total de 29
n'est vrai qu'à cette condition) de `crates/kesh-db/src/` et `crates/kesh-api/src/`, extrait toute instruction
`INSERT [IGNORE] INTO` / `REPLACE INTO` / `UPDATE` / `DELETE FROM` visant `bank_imports`, `bank_transactions`,
`payment_batches`, `payment_batch_items`, `invoice_settlements`, `supplier_invoices` ou `bank_accounts`, et la rattache à
sa fonction englobante. Chaque couple `(fichier, fonction)` doit figurer au registre, avec un statut :

- `Gardee("<fonction de garde ou instruction>")` ;
- `Invariant("I1" | "I2" | "I3", "<raison>")` ;
- `SansCompteBancaire("<raison>")` (l'écriture ne porte aucun compte bancaire, ou porte un compte neuf) ;
- `HorsPerimetre("<raison>")` (routes de test, et tout écrivain hors périmètre **que l'extraction voit**) — ⚠️
  précisé en validation P4 (R2) : la **restauration d'installation** n'y figure **pas**, parce que l'extraction ne la
  voit pas (nom de table non littéral, ci-dessous) — une entrée à son nom n'aurait aucun site et ferait rougir le
  diff inverse ; elle est classée hors périmètre **dans la liste des angles morts du doc-comment**, avec sa raison
  (I1 : elle remplace toute la base par un état qui le respectait). Le rejeu post-restauration est du SQL
  (`kesh-db/src/post_restore/*.sql`), hors de ce que l'extraction lit ;
- `SortieAutorisee("<raison>")` — ajouté en validation P3 (F8, C-28-4-23) : un geste qui ne fait que **sortir** de
  l'état qu'un invariant interdit sur un compte clos, laissé volontairement sans garde pour que l'utilisateur puisse
  réparer ce que L1 a laissé passer. Seul site : W21 (`cancel_batch`).

Diff ensembliste **dans les deux sens** (une écriture neuve non classée rougit ; une entrée sans site rougit), comme le
registre des routes. Sur `5aad8b35`, l'extraction rend **29 sites** (W1-W29, § *Inventaire*) ; **30** après le rebase sur la 28-5 (W30, `unreject`). ⛔ Le test **n'établit
pas** que la garde s'exécute — c'est l'affaire des tests de l'AC6 ; il ferme le mode d'échec « un écrivain ajouté
demain contourne la clôture sans que rien ne le signale ». Son doc-comment le dit, et dit ce qu'il ne voit pas :
écritures dont le **nom de table n'est pas littéral** — `format!` à nom variable, `QueryBuilder` dont la table est
poussée par `push(table)` (un `QueryBuilder::new("INSERT INTO bank_transactions …")` à littéral, comme W23,
`bank_imports.rs:110`, **est** vu, F8) —, SQL hors de ces deux crates. Sites connus de la première forme — **trois**,
énumérés au doc-comment (« le seul site connu » de la P1 était faux, validation P4, R2) : (1) et (2)
`kesh-db/src/backup.rs:496` (`format!("DELETE FROM \`{table}\`")`) et `:548` (`format!("INSERT INTO \`{table}\` …")`),
dans `restore_body` (l. 473), qu'appelle `restore_tables_in_tx` (l. 444), elle-même appelée par la restauration
d'installation, `kesh-api/src/routes/admin.rs:295` — **dans** une crate lue, invisibles parce que le nom de table est
variable ; hors périmètre (I1, ci-dessus) ; numéros de `5aad8b35`, **décalés à l. 581 et 633 par la 28-3**
(`c1667e8a`, sur `origin/main`), à relire au rebase (T0) ; (3) `kesh-seed/src/lib.rs:586` (`format!("DELETE FROM
\`{table}\`")`, remise à zéro de la démo, toutes tables), hors des deux crates lues (F5). Les trois sont des angles
morts du registre, classés hors périmètre au doc-comment, **non** des entrées du registre.

**Second inventaire clos, par appelant** (validation P2, F6 ; C-28-4-22) : la garde de W8 ne vit pas dans
`invoice_settlements::create_in_tx` mais chez un de ses appelants (C-28-4-14), si bien qu'un statut par couple
`(fichier, fonction)` du SQL ne peut pas dire vrai pour W8 — et qu'un quatrième appelant créé demain, avec
`SettlementKind::Choice(BankTransfer { … })` et sans garde, passerait sans rougir. Le même test extrait donc aussi
**tout appel** `invoice_settlements::create_in_tx(` hors tests et hors commentaires dans les deux crates, le rattache à
sa fonction englobante, et exige un statut **par appelant**, diff dans les deux sens. Sur `5aad8b35`, trois appelants
(`grep -rn "invoice_settlements::create_in_tx(" crates/kesh-db/src crates/kesh-api/src`) :

| Appelant | Statut |
|---|---|
| `kesh-db/src/repositories/invoice_settlements_write.rs::settle_invoice` (l. 283) | `Gardee("closed_on, archived lus dans le FOR UPDATE du compte, étape (3), l. 131-135")` |
| `kesh-api/src/routes/reconciliation.rs::accept_one_invoice` (l. 1986) | `Invariant("I1", "transaction pending dans l'état commité jusqu'au commit du lot ; sous L1 (#651), le pré-vol R1 refuse")` — raison corrigée en validation P4 (R7 : « tenue par le lot » contredisait l'AC7) |
| `kesh-db/src/repositories/invoice_settlements_write.rs::write_off_invoice` (l. 637) | `SansCompteBancaire("passage en perte : aucun compte bancaire")` |

Précédent et extracteur à réutiliser : `crates/kesh-db/tests/letterings_lexical.rs`,
`invoice_settlement_writers_stay_in_their_module_and_sync` (l. 577-627), qui **assert déjà** cette liste de trois
créateurs (`appels_et_suite`, l. 463) — le registre neuf y ajoute le statut « compte clos » ; il ne la remplace pas.
La ligne W8 du registre principal prend le statut `Gardee("par ses appelants : inventaire des appelants")`.

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

Le message existant `bank-accounts-errors-has-transactions` (l. 1295 en fr-CH, fallback `errors.rs:2581-2587`)
**garde** la voie de la 27-1 — supprimer les imports non rapprochés, puis archiver : elle sert encore, pour un compte
dont aucun import n'a été rapproché (C-27-1-12) — et **ajoute** la clôture (R3, C-28-4-19) : « Le compte bancaire
contient des transactions — archivage refusé pour préserver l'audit comptable. Les imports dont aucune transaction
n'est rapprochée peuvent être supprimés (Quotidien → Importer des relevés). Sinon, pour retirer le compte de l'usage,
clore le compte : il passe en lecture seule et garde son historique. » — la deuxième phrase reste **à l'octet**, ce que
le test de la 27-1 `bank_accounts_e2e.rs:2316-2320` assert (`msg.contains(…)`) : il reste vert sans retouche et gagne
une assertion sur « clore le compte ». Le message `bank-accounts-errors-cannot-archive-primary` (fallback
`errors.rs:2602-2605`) remplace « tant qu'un autre compte non-archivé existe » par « tant qu'un autre compte ni archivé
ni clos existe » (F9). Les deux, dans les quatre locales **et** dans le fallback Rust (grep de la valeur, § *Propagation
post-patch*). **G9** (`textes_coherents.rs:711`, table fermée : fr-CH et repli Rust égaux) vise **déjà**
`bank-accounts-errors-has-transactions` : sa ligne reste, son contenu suit ; les **cinq** clés neuves de l'AC9 et
`cannot-archive-primary` y **entrent** (le repli Rust de chacune existe, AC6/T3). Étiquettes d'audit :
`crates/kesh-api/src/audit_labels.rs` (l. 80-82) gagne `bank_account.closed` et `bank_account.reopened` ;
`audit_label_registry.rs` reste vert.

**AC10 — Les tests de la partie a, et les gardes qui changent de sens.**

Tests listés au § *Tests* (T-a1 à T-a49, dont cinq retirés en validation P2 : T-a32, T-a39, T-a40, T-a41, T-a44 ;
T-a48 ajouté en validation P3, T-a49 en validation P4) ; mutations M1 à M24, M33 à M56 et les trois `-bis` (M10-bis,
M14-bis, M20-bis) (§ *Mutations*) **toutes rouges**, ou déclarées équivalentes avec la raison, ou sans objet (M19
depuis la P1 ; M10, M35, M49, M50 depuis la P2 ; M36 à M39, déclarées sans objet en P2, **rétablies** en validation P3
avec un objet neuf — garde avancée avant le verrou d'exercice ; M54, équivalente en P3, **tuable** depuis la
validation P4). Gardes
existantes à faire suivre, **chacune nommée** : `audit_route_registry.rs` (deux routes neuves `("post",
"bank_accounts::close_bank_account", Traced, SansEcritureAuJournal)` et `reopen` ; les décomptes de son doc-comment —
**115** routes de `lib.rs`, **118** au registre sur `5aad8b35`, l. 26-31 — passent à 117 / 120, **et ses cinq
assertions** avec eux (liste complétée en validation P3, R4) : `LIB_ROUTES.len() == 115` (l. 645) → 117, `traced ==
109` (l. 648) → 111 **et son message** (l. 649-657, qui énumère les routes tracées : y ajouter la clôture et la
réouverture d'un compte bancaire, 28-4a, #642), le `118` de l. 667 → 120, `sans_ecriture, 91` (l. 990-991) → 93 et
son message (« 90 routes de lib.rs » → 92), `tout.len() == 118` (l. 997) → 120 ; tout à relire après le rebase sur
la 28-5 qui en ajoute) ; `audit_label_registry.rs` ; `test_schema_guard.rs` ;
`migrations_upgrade_path.rs` ; `textes_coherents.rs` **G9** (l. 711, AC9 : une ligne existante suit, six entrent) ;
`bank_accounts_e2e.rs:2316-2320` (le test de la 27-1 sur le message du `412`, qui reste vert, AC9) ;
`letterings_lexical.rs` (sa liste des trois créateurs de règlement reste vraie, AC8) ;
`reconciliation_cancel.rs:128` (`a_concurrent_close_waits_for_the_unreconciliation`, qui garde désormais **aussi** la
position de la garde de W9, AC7, M36 — texte inchangé ; ⚠️ s'il venait à être remonté sur un compte interne ou sans
compte bancaire, il cesserait de la garder, et cela s'écrirait) ; tests du CSV
(`exports_global_e2e.rs`, en-tête de `bank_accounts.csv`) ; `admin_backup_e2e.rs` (aller-retour) ; les tests existants de
l'archivage dont l'ordre des refus change pour un compte clos (AC5 — aucun aujourd'hui ne combine clos et décompte,
à confirmer par le gate). Le doc-comment du point (vi) d'`audit_route_registry.rs` (routes `SansEcritureAuJournal`
rejouées) **ne change pas** : aucune enveloppe n'est ajoutée (AC7). **Gate backend
complet** à chaque commit de la boucle (story `kesh-db` : migration et dépôts — **pas** de gate ciblé, même en boucle de
revue), E2E complet au dernier commit de code (il ne touche pas l'écran, mais `/companies/current` et `GET
/bank-accounts` changent de contenu).

### Partie b — `28-4b-cloture-ecran`

**AC11 — L'écran *Comptes bancaires*.**

`frontend/src/routes/(app)/bank-accounts/+page.svelte` et `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` :

- type `BankAccountSummary` (l. 15) et brut (`interface BankAccountSummaryRaw`, l. 62) : `closedOn: string | null` ; `parseBankAccount` (l. 80) le porte ;
  `listBankAccounts(includeArchived = false, includeClosed = false)` (l. 105 : la signature d'aujourd'hui est
  `listBankAccounts(includeArchived = false)`, un **booléen** ; un objet d'options casserait `npm run check` à chaque
  appelant qui passe `false` — R9, C-28-4-20) — **les appelants existants gardent leur sens et leur code**
  (`listBankAccounts(false)` / `listBankAccounts()` → ni archivés ni clos ; la page de gestion, l. 68, passe sa
  variable `includeArchived` puis la nouvelle `includeClosed`) ;
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
(`settings/invoicing/+page.svelte:148`) passent `includeClosed = true` (second paramètre), pour rester alignés sur le
serveur, qui compte un compte clos comme lié à son compte comptable (C-28-4-7). Preuve : un **E2E** qui clôt un compte
(non principal, sans transaction en attente) et vérifie son absence des **six** sélecteurs et de la tuile, puis sa
présence après réouverture.

Consommateurs **sans changement de code**, dont l'effet se dit (R9, R7, C-28-4-18) :
- la **tuile de l'accueil** (`(app)/+page.svelte:58`) : un compte clos en sort, avec son solde comptable et son écart,
  même non nuls (la clôture **avertit** d'un solde non nul, C-28-4-6, sans refuser). Le solde reste lisible sur l'écran
  *Comptes bancaires* (bascule des clos) et au plan comptable ; le manuel le dit (AC14, § *Solde comptable*) ;
- `lib/features/reconciliation/ReconciliationProposals.svelte:152` (`listBankAccounts()`) : un compte clos sort de la
  liste ; le repli `null` existant couvre une proposition qui le nommerait ;
- **Paramètres** (`routes/(app)/settings/+page.svelte:468`, liste de `company.bankAccounts` lue de `/companies/current`) :
  un compte clos n'y figure plus ; le texte d'aide `settings-bank-manage-hint` renvoie à l'écran *Comptes bancaires*
  (AC15). « Aucun compte bancaire configuré » ne s'affiche que si tous les comptes ouverts ont disparu — le principal ne
  se clôt pas, donc seulement sans principal (F9) : angle mort assumé, écrit au manuel.

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
  annulation, **annulation d'une facture fournisseur payée par ce compte** (W28), **création et confirmation** d'un lot
  de paiement — l'**annulation** d'un lot généré reste possible (C-28-4-23, validation P4, R9 = F-L7) —, modification,
  archivage) ; ce qui reste (historique des imports, écritures, rapports, export, sauvegarde ; le compte comptable lié
  reste ouvert) ; l'avertissement de solde ; la réouverture et sa trace ; l'angle mort des boutons *Supprimer
  l'import* / *Annuler le règlement* qui restent affichés et sont refusés ; **une opération en cours au moment de la
  clôture** — pour un import, **depuis l'envoi du fichier** (validation P4, R8 : la fenêtre couvre l'analyse du
  relevé) —, dans un autre onglet ou par une intégration, peut encore aboutir sur le compte (L1, AC7, #651) : le
  rouvrir pour la traiter, puis le clore à nouveau ; et donc, dit en toutes lettres, **le refus des écritures vaut hors
  de cette fenêtre** ; **une transaction rejetée compte comme non
  rapprochée** (elle est `pending`) : elle empêche la clôture tant qu'elle n'est pas rapprochée manuellement ou que son
  import n'est pas supprimé (R13 — la voie « annuler le rejet » de la 28-5, si elle est mergée, se nomme aussi) ; et,
  au paragraphe de l'archivage, qu'**un compte principal s'archive quand tous les autres sont clos**, laissant la société
  sans compte principal ni IBAN de QR-facture — rouvrir puis désigner un autre compte (F9).
- § *Supprimer un import* (l. 1724, phrase l. 1754-1757) : la voie « supprimer ses imports pour archiver » cède la place
  au renvoi à la clôture (`\S\ref{sec:clore-compte-bancaire}`) ; la phrase reste exacte sur l'archivage. **Et l'ordre
  des refus de W24** (validation P4, R1 = F-L3, C-28-4-24) : un import dont une transaction est rapprochée ne se
  supprime pas, **que son compte soit clos ou non** — c'est le refus existant (« annulez d'abord leur rapprochement »),
  et, sur un compte clos, annuler un rapprochement exige de le rouvrir d'abord ; le refus « compte clos » ne
  s'affiche que pour un import sans transaction rapprochée.
- § *Import bancaire* (l. 1696, « comptes bancaires non archivés ») → « ni archivés ni clos ». Toute autre mention
  greppée (`grep -n "non archiv\|archiv" docs/manual/fr/user-manual.tex`).
- § *Solde comptable, relevé et écart* (`\label{sec:solde-comptable}`, l. 238-262, R7) : « le chiffre affiché sous chaque
  compte bancaire » — un compte clos ne figure plus sur la tuile de l'accueil ; son solde se lit sur l'écran *Comptes
  bancaires* (bascule *Afficher les comptes clos*) et au plan comptable (C-28-4-18).
- Règlements (client et fournisseur) : un compte **archivé** n'est plus proposé et ne reçoit plus de règlement par
  l'API ; l'annulation d'un règlement ancien de ce compte reste possible (AC6-bis) — une phrase là où le manuel décrit
  le règlement par virement (`grep -n "virement" docs/manual/fr/user-manual.tex`).
- `docs/manual/fr/admin-manual.tex` : sous-section **« Passer à la 0.13.3 »** (à côté de `sec:maj-0-13-2`, l. 1972 ;
  créée ici ou complétée par **union** si une autre story de l'epic l'a ouverte) : une migration, non breaking, pas de
  bump de `kesh_version_min_required` ; retour à la 0.13.2 : **restaurer la sauvegarde prise avant la mise à jour** ;
  les intégrations par clé d'API voient disparaître les comptes clos de `GET /bank-accounts` sans `includeClosed=true`.
  **Et deux phrases existantes que la 0.13.3 rend fausses** : l'étape 1 de la *Procédure de mise à jour standard*
  (`admin-manual.tex:1943`, « Pour passer à la 0.13.0 ou à la 0.13.1, cette sauvegarde est le seul retour en arrière
  possible … La 0.13.2 n'a pas de migration ») gagne la 0.13.3 (retour : restaurer, R7) ; § *Rollback en cas d'échec*
  (l. 2234, phrase l. 2251 : « le retour à la version précédente n'est possible que si aucune autre migration de la
  release ne relève la version minimale exigée ») gagne : « une release qui ajoute une migration ferme le retour sans
  restauration, qu'elle relève ou non la version minimale » (F7, dit au conditionnel prudent de C-28-4-11).
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
  d'écriture, `failed[]` dans les lots de réconciliation, **et l'angle mort L1** (validation P3, R6 b : une écriture
  dont la transaction a commencé avant le commit d'une clôture peut encore aboutir sur le compte ; une intégration qui
  clôt un compte puis importe en parallèle doit le savoir ; remède : rouvrir, traiter, clore — AC7), **avec le lien
  vers [#651](https://github.com/guycorbaz/kesh/issues/651)** et la borne de la promesse (« refusée côté serveur » hors
  de cette fenêtre, validation P4) ; l'ordre des refus de `DELETE /bank-imports/{id}` (`409
  BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` avant `409 BANK_ACCOUNT_CLOSED`, C-28-4-24) ; § 10 (codes
  d'erreur) : les cinq codes neufs. La note ⁸
  (l. 233, archivage après suppression des imports) renvoie **aussi** à la clôture (la voie de la 27-1 reste, AC9).
  Les sous-sections qui décrivent les refus des routes gardées gagnent chacune `409 BANK_ACCOUNT_CLOSED` (ou
  `failed[]`), F10 : « Lister et annuler les règlements d'une facture » (l. 464), « Annuler le règlement d'une facture
  fournisseur » (l. 514), « Annuler une facture fournisseur » (l. 524, facture payée), « Accepter des propositions de
  rapprochement » (l. 534), « Annuler un rapprochement bancaire » (l. 554) — `grep -n "^### " docs/api-external.md`
  rejoué au rebase pour les autres routes de la table de l'AC6. (La note de l'archivé, AC6-bis, est écrite par la
  28-4a.)
- `CHANGELOG.md`, `## [0.13.3] — Non publié` (créé ou complété par **union**), rubrique **`### Ajouté`** : « **Clore un
  compte bancaire ([#642](https://github.com/guycorbaz/kesh/issues/642)).** … » — ce que fait la clôture, ses trois
  conditions, la réouverture, ce qui reste lisible ; **la borne du refus** (validation P4, R3 = F-M2) : « les écritures
  visant un compte clos sont refusées côté serveur ; une opération déjà en cours au moment de la clôture peut encore
  aboutir ([#651](https://github.com/guycorbaz/kesh/issues/651)) — rouvrir, traiter, clore à nouveau » ; et, dans le bloc *À faire en passant à la 0.13.3*, la ligne des
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
- [ ] **T2** — Entité, macro `columns!()`, `BankAccountListFilter`, appelants, `includeClosed`, `count_other_active_for_company`,
  CSV `closed_on` (AC2).
- [ ] **T3** — `DbError::BankAccountClosed`, `AppError` (cinq variantes neuves, `errors.rs` près de l. 2580-2610),
  fallbacks Rust (AC6, AC9) ; **bras `DbError::BankAccountClosed` dans le `match` d'`AppError::Database`**
  (`errors.rs:2905`) et bras `AppError::BankAccountClosed`, rendus par **une** fonction partagée (AC6, validation P4,
  R11 = F-L4).
- [ ] **T4** — `bank_accounts::ensure_open_in_tx`, `ensure_open_unarchived_in_tx` (lectures non verrouillantes, SQL en
  constantes), `close_in_tx`, `reopen_in_tx` + doc-comments (règle de la garde **et sa règle de placement** — après
  le dernier verrou que le chemin prend avant sa première lecture ordinaire, ou, sans lecture ordinaire hors l'audit,
  avant sa première écriture —, critère à deux volets « aucun verrou ajouté » et « aucun déplacement de la naissance
  de la vue », invariants I1-I3, angle mort L1, sa classe P2 **et son issue, #651**) ; handlers `close_bank_account`, `reopen_bank_account` ; routes `lib.rs` (AC3, AC4,
  AC7).
- [ ] **T5** — Refus du compte sur lui-même : `update_for_company`, `set_journal_account_id_for_company`,
  `archive_for_company` (`closed_on` lu dans leur `FOR UPDATE` existant) ; dans la route d'archivage, refus `409` dans
  le pré-vol existant, avant la sentinelle — **aucun verrou neuf** (AC5, C-28-4-21).
- [ ] **T6** — Gardes des points d'étranglement W8, W9, W17-W19, W20, W22-W25, W27, W28, W29 ; W30 (au rebase sur la
  28-5, R8 avant la classification) ; pré-vols R1-R4, R6-R8 (AC6) ; W24 **après le comptage de l'étape 3, avant le
  `DELETE`** (C-28-4-24). **Aucune** garde dans `accept_one_*` ni dans
  `invoice_settlements::create_in_tx` (C-28-4-14). **Aucun verrou ajouté** (AC7) : pour chaque site, **recopier au
  Dev Agent Record** la ligne de la table *Position par chemin* (§ *Concurrence*) avec la ligne de code retenue, et
  y répondre « verrou ajouté : non ; ordre changé : non ; vue déplacée : non » — la dernière réponse en nommant la
  première lecture non verrouillante du chemin au sol et la ligne de la garde (validation P3). À chaque site, un
  commentaire d'une ligne renvoie au doc-comment du sol qui impose la position (AC7, *Règle de placement*).
- [ ] **T6-bis** — Archivé aux gestes qui créent : W8, W27 (instruction étendue), W22 (`ensure_open_unarchived_in_tx`),
  appariement `DbError::NotFound` → `AppError::BankAccountNotFound` aux **deux** appels de la route d'import
  (`routes/bank_imports.rs:1312` CAMT, `:2199` CSV) ; `### Corrigé` du CHANGELOG et note d'`api-external.md` (AC6-bis).
- [ ] **T7** — Registre statique `bank_account_closed_inventory.rs`, écrivains **et** appelants de
  `invoice_settlements::create_in_tx` (AC8).
- [ ] **T8** — i18n serveur ×4, `audit_labels.rs` (AC9).
- [ ] **T9** — Tests T-a1 à T-a49 (cinq retirés), mutations M1-M24, M33-M56 et les trois `-bis`, gardes nommées à l'AC10 ; gate complet + E2E complet au dernier
  commit de code ; commit par étape ; Dev Agent Record (« clôture accessible par l'API seulement jusqu'à la 28-4b ») ;
  **corps de PR demandé à l'orchestrateur** : `refs #642` et `refs #651` (validation P4).

### 28-4b

- [ ] **T10** — Partir d'`origin/main` après la fusion de la 28-4a ; `cp -al` des `node_modules`.
- [ ] **T11** — `bank-accounts.api.ts`, écran, dialogues (AC11) ; réglages de facturation (AC12) ; libellé (AC13).
- [ ] **T12** — i18n UI ×4, `sitesTotal` recompté (AC15).
- [ ] **T13** — Manuels utilisateur et d'administration, PDF régénérés et contrôlés aplatis (AC14).
- [ ] **T14** — `api-external.md`, CHANGELOG, README / site si concernés (AC16) ; corps de PR demandé à l'orchestrateur :
  `closes #642` et, à côté, `refs #651` (validation P4).
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
| W1 | `kesh-api/src/routes/reconciliation.rs:2039` | `accept_one_invoice` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 ; W8 en aval **sans garde** (C-28-4-14) |
| W2 | `…/reconciliation.rs:2538` | `accept_one_split` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 |
| W3 | `…/reconciliation.rs:2990` | `accept_one_rule` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R1 |
| W4 | `…/reconciliation.rs:3356` | `reject_batch` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R2 |
| W5 | `…/reconciliation.rs:3801` | `post_manual_once` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R3 |
| W6 | `…/reconciliation.rs:4258` | `post_split_once` | `UPDATE bank_transactions` | Invariant I1 + pré-vol R4 |
| W7 | `kesh-api/src/routes/bank_accounts.rs:604` | `create_bank_account` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf, ouvert) |
| W8 | `kesh-db/src/repositories/invoice_settlements.rs:415` | `create_in_tx` | `INSERT INTO invoice_settlements` | **Gardée par ses appelants** (C-28-4-14) : `settle_invoice` lit `closed_on, archived` dans le `FOR UPDATE` existant du compte de son étape (3) (`invoice_settlements_write.rs:131-135`) ; `accept_one_invoice` (`reconciliation.rs:1986`) : Invariant I1 + pré-vol R1, **sans garde** ; `write_off_invoice` (l. 637) : sans compte bancaire. ⛔ Aucune garde **dans** `create_in_tx`. Statut par appelant : second inventaire de l'AC8 (F6 de la P2) |
| W9 | `kesh-db/src/repositories/reconciliation_cancel.rs:339` | `cancel_in_tx` | `UPDATE bank_transactions` | **Gardée** (`ensure_open_in_tx` sur `bt.bank_account_id`, lecture non verrouillante **après l'étape (3)**, avant (4) — déplacée en validation P3, AC7 *Règle de placement* ; pour un rapprochement de facture, l'étape (6) passe aussi par la garde de W25) |
| W10 | `kesh-db/src/repositories/bank_accounts.rs:49` | `create` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf) |
| W11 | `…/bank_accounts.rs:301` | `upsert_primary_in_tx` | `UPDATE bank_accounts` | Invariant I3 (vise le principal, jamais clos) |
| W12 | `…/bank_accounts.rs:331` | `upsert_primary_in_tx` | `INSERT INTO bank_accounts` | SansCompteBancaire (compte neuf) |
| W13 | `…/bank_accounts.rs:457` | `set_journal_account_id_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5 : `closed_on` dans le `FOR UPDATE` existant, l. 413) |
| W14 | `…/bank_accounts.rs:577` | `update_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5 : `closed_on` dans le `FOR UPDATE` existant, l. 548) |
| W15 | `…/bank_accounts.rs:640` | `archive_for_company` | `UPDATE bank_accounts` | **Gardée** (AC5 : `closed_on` dans le `FOR UPDATE` existant, l. 619-623, et dans le pré-vol de la route) |
| W16 | `…/bank_accounts.rs:702` | `flip_primary_off_for_company` | `UPDATE bank_accounts` | Invariant I3 (vise le principal) |
| W17 | `kesh-db/src/repositories/payment_batches.rs:137` | `create_batch` | `INSERT INTO payment_batches` | **Gardée** (`closed_on` ajouté à la lecture non verrouillante existante de l'étape (0), l. 85-99 ; la relecture verrouillée de la P1 est retirée, C-28-4-21) |
| W18 | `…/payment_batches.rs:152` | `create_batch` | `UPDATE payment_batches` | **Gardée** (même transaction que W17) |
| W19 | `…/payment_batches.rs:164` | `create_batch` | `INSERT INTO payment_batch_items` | **Gardée** (même transaction) |
| W20 | `…/payment_batches.rs:507` | `confirm_batch` | `UPDATE payment_batches` | **Gardée**, transitivement par W27 (+ refus précoce dans la lecture non verrouillante existante de l'étape (2)) ; et Invariant I2 |
| W21 | `…/payment_batches.rs:589` | `cancel_batch` | `UPDATE payment_batches` | `SortieAutorisee` (validation P3, F8, C-28-4-23) — classé « Invariant I2 » jusqu'à la P2, ce qui était inexact sous L1 : un lot `generated` peut exister sur un compte clos (L1), et `cancel_batch` est le geste, **volontairement non gardé**, qui l'en fait sortir |
| W22 | `kesh-db/src/repositories/bank_imports.rs:66` | `create_with_transactions` | `INSERT INTO bank_imports` | **Gardée** (`ensure_open_unarchived_in_tx`, lecture non verrouillante, première instruction de la fonction, avant l'`INSERT` d'entête ; AC6-bis) |
| W23 | `…/bank_imports.rs:110` | `create_with_transactions` | `INSERT INTO bank_transactions` | **Gardée** (même transaction) |
| W24 | `…/bank_imports.rs:494` | `delete_unreconciled_in_tx` | `DELETE FROM bank_imports` | **Gardée** (`ensure_open_in_tx`, lecture non verrouillante **après le comptage de l'étape 3, avant le `DELETE` de l'étape 4** — l'entête verrouillé à l'étape 1 donne `bank_account_id` ; déplacée en validation P3 après l'étape 2, R2 = F2, puis en validation P4 après le comptage, R1, C-28-4-24 : le refus du sol `HasReconciled` passe avant le refus « clos » ; les doc-comments « l'entête, première instruction » et « aucune vue n'est ouverte avant les verrous » restent vrais) |
| W25 | `kesh-db/src/repositories/invoice_settlements_write.rs:922` | `cancel_settlement_in_tx` | `DELETE FROM invoice_settlements` | **Gardée** si `settlement_bank_account_id` est non nul — colonne **à ajouter** au `SELECT` verrouillé de l'étape (2) (l. 835-836 ne lit que `journal_entry_id, amount, settled_on`, R8 ; même ligne, verrou inchangé) ; garde non verrouillante **après (2-bis)**, avant (3) (déplacée en validation P3) |
| W26 | `kesh-db/src/repositories/supplier_invoices.rs:524` | `create_in_tx` | `INSERT INTO supplier_invoices` | SansCompteBancaire — **confirmé en validation P1** (R1, F2) : la facture naît ouverte, la liste de colonnes de l'`INSERT` (l. 524-528) ne nomme aucun `settlement_*` |
| W27 | `…/supplier_invoices.rs:895` | `pay_in_tx` | `UPDATE supplier_invoices` | **Gardée** (dans le `FOR UPDATE` existant de l. 753-756, `closed_on, archived` ajoutés, AC6-bis) |
| W28 | `…/supplier_invoices.rs:1133` | `cancel_in_tx` | `UPDATE supplier_invoices` | **Gardée** si `inv.settlement_bank_account_id` est non nul (`ensure_open_in_tx`, non verrouillante, **après** le verrou facture l. 1061 **et** celui de l'écriture d'achat et de son exercice l. 1070-1080, avant les motifs l. 1082 — déplacée en validation P3) — ⛔ **reclassée en validation P1** (F2 = R1) : le classement d'origine, « SansCompteBancaire, n'annule qu'une facture non réglée », était **faux** ; la fonction annule une facture « ouverte ou payée » (doc l. 1052) et remet `settlement_bank_account_id` à `NULL` (l. 1131-1134), et `supplier_invoice_cancel_blocker` (l. 985-1030) ne refuse pas l'état `paid` |
| W29 | `…/supplier_invoices.rs:1369` | `cancel_settlement_in_tx` | `UPDATE supplier_invoices` | **Gardée** si `settlement_bank_account_id` (`ensure_open_in_tx`, non verrouillante, **après (1-bis)**, l. 1290-1312, avant les motifs l. 1314 — position fixée en validation P3) |
| W30 | *introduite par la 28-5, à garder au rebase* — handler de `POST /api/v1/reconciliation/transactions/{id}/unreject` (annuler le rejet, #526 ; fiche `28-5-transaction-rejetee-atteignable.md`, non mergée au `5aad8b35`) | à relire sur la branche rebasée | `UPDATE bank_transactions` (remise à NULL de `auto_match_rejected_at`) | **Refus `409 BANK_ACCOUNT_CLOSED`** (pré-vol R8 sur le `bank_account_id` de la transaction, et garde `ensure_open_in_tx`, non verrouillante, dans sa transaction, après le verrou de la ligne `bank_transactions`, sous le verrou nommé du compte que la 28-5 prend) ; en outre Invariant I1 (une rejetée est `pending`) |

Hors de l'extraction, à classer `HorsPerimetre` si le registre les voit : le rejeu post-restauration
(`kesh-db/src/post_restore/*.sql`, SQL et non Rust), les routes de test (`routes/test_endpoints.rs`). ⚠️ **La
restauration d'installation** (situation corrigée en validation P4, R2 ; la fiche la plaçait dans
`admin_backup/import.rs`, qui ne fait que l'**analyse** du fichier) écrit les sept tables depuis
`kesh-db/src/backup.rs`, `restore_body` (l. 473) — `DELETE` l. 496, `INSERT` l. 548, deux `format!` à nom de table
variable —, par `restore_tables_in_tx` (l. 444), qu'appelle `kesh-api/src/routes/admin.rs:295` ; l'extraction ne la
voit pas (nom non littéral) : angle mort du registre, classé hors périmètre au doc-comment (I1 : elle remplace toute
la base), **pas** une entrée du registre (AC8). Numéros de `5aad8b35` ; la 28-3 (`c1667e8a`) les porte à l. 557, 581
et 633 (`restore_tables_in_tx` l. 528). ⚠️ Les lignes de `repositories/invoices.rs` 4353-4862 sont dans `mod tests` (ouvert
l. 2984-2985) : exclues à juste titre. **Élargir avant de figer** (consigne de la passe : un détecteur part du
symptôme) : écritures par `format!` (`grep -rn 'format!("UPDATE\|format!("INSERT\|format!("DELETE' crates/` — il
rend, dans les deux crates lues, les deux sites de `backup.rs` ci-dessus),
`QueryBuilder`, `kesh-reconciliation`, `kesh-payment`, `kesh-seed` (aucun site sur `5aad8b35` pour `kesh-reconciliation`
et `kesh-payment` : `grep -rlE "INSERT INTO|UPDATE|DELETE FROM" crates/kesh-reconciliation/src crates/kesh-payment/src`
à rejouer ; ⚠️ **un site** pour `kesh-seed` — `kesh-seed/src/lib.rs:586`, `format!("DELETE FROM \`{table}\`")`, remise à
zéro de la démo sur toutes les tables, nom de table variable : `HorsPerimetre`, angle mort du registre, F5).
Écrivains **par colonne** (recompte de la lentille F, P1) : `settlement_bank_account_id` n'est écrit que par W8, W27,
W28, W29 — tous gardés désormais.

**Pré-vols de route** (refus précoce, à côté de la garde d'archivage existante — sites `if ….archived {`) :

| # | Site | Handler |
|---|---|---|
| R1 | `routes/reconciliation.rs:1040` | `post_accept` — produit le `failed[]` complet, sans verrou |
| R2 | `routes/reconciliation.rs:3174` | `post_reject` — idem |
| R3 | `routes/reconciliation.rs:3564` | `post_manual` — `409` |
| R4 | `routes/reconciliation.rs:4007` | `post_split` — `409` |
| R5 | ~~`routes/reconciliation.rs:2003-2009`~~ | **Retiré en validation P1** (C-28-4-14) : sans garde dans `create_in_tx`, `accept_one_invoice` ne reçoit jamais `DbError::BankAccountClosed` ; un appariement serait du code mort. Numéro conservé pour ne pas renuméroter |
| R6 | `routes/bank_imports.rs:965` | `preview` — `409` |
| R7 | `routes/bank_imports.rs:1114` | `create` — `409` |
| R8 | *introduit par la 28-5, à garder au rebase* — handler `unreject` | `409` (W30) |

`get_proposals` (l. 610) est une **lecture** : pas de refus (AC2).

### Les trois invariants, et leurs écrivains (`invariant-dans-le-temps`)

- **I1 — un compte clos ne porte aucune transaction `pending`.** Établi par la clôture (`412` sinon). Écrivains qui
  **produisent** une `pending` : `create_with_transactions` (W23, gardé) et `reconciliation_cancel::cancel_in_tx` (W9,
  gardé) — `grep -rn "status = 'pending'" crates/*/src` à rejouer. Ceux qui la font **disparaître** (W1-W6, W24) ne le
  cassent pas. La restauration d'installation (`kesh-db/src/backup.rs::restore_body`, appelée par
  `routes/admin.rs:295`) remplace toute la base par un état qui le respectait — sauf sauvegarde incohérente (#633,
  28-3) : angle mort écrit.
- **I2 — un compte clos ne porte aucun lot `generated`.** Établi par la clôture. Seul producteur : `create_batch`
  (W17, gardé). `confirm_batch` et `cancel_batch` en sortent.
- **I3 — un compte clos n'est jamais principal.** **Structurel** : `chk_bank_accounts_closed_not_primary` (AC1). La
  clôture refuse le principal ; `PUT` sur un compte clos est refusé (AC5) ; la réouverture ne touche pas `is_primary`.
  D'où W11 et W16 hors d'atteinte, et `find_primary` (QR-facture, `invoice_pdf_service.rs:222`) qui ne rend jamais un
  compte clos.

### Concurrence — la doctrine appliquée ici

*Réécrit en validation P2 (C-28-4-21). Les versions P0 et P1 de ce paragraphe — garde exclusive, règle « pièces →
compte → exercice », archivage verrouillé avant son décompte — sont dans l'historique Git (`32ee4e36`) ; elles ne
s'appliquent plus.*

- InnoDB, `REPEATABLE READ` ; `sqlx::begin()` émet `BEGIN` sans instantané : la vue cohérente naît à la **première
  lecture non verrouillante** de la transaction. Une lecture **verrouillante** (`FOR UPDATE`) lit au contraire la
  dernière version commitée. Trois conséquences, et c'est toute la doctrine de la story :
  - une garde **non verrouillante** lit l'instantané de sa transaction : une clôture commitée après la naissance de cet
    instantané ne se voit pas — c'est l'angle mort **L1** (AC7) ;
  - la clôture fait son `FOR UPDATE` **d'abord** ; ses décomptes, lus ensuite en non-verrouillant, voient tout ce qui
    a été commité avant qu'elle obtienne le verrou. Un écrivain qui a **inséré** une ligne rattachée au compte tient,
    par la clé étrangère, un verrou partagé sur la ligne du compte : la clôture l'attend (T-a31). C'est au sol ;
  - *(validation P3, R1 = F1)* une garde non verrouillante qui serait posée **avant** un verrou du chemin y ferait
    naître la vue **avant l'attente** de ce verrou : tout ce que le chemin relit ensuite sans verrou — l'exercice
    « ouvert » qu'une clôture d'exercice concurrente vient de fermer — viendrait de cet instantané ancien, et une
    annulation passerait dans un exercice clos (erreur comptable, P1). Le sol l'interdit en toutes lettres
    (`reconciliation_cancel.rs:247-252` et `:296`, `supplier_invoices.rs:1040-1045`, `invoice_settlements_write.rs:858-861`,
    `bank_imports.rs:389-391` et `:420-425`) ; d'où la **règle de placement** de l'AC7 : la garde se pose après le
    dernier verrou que le chemin prend avant sa première lecture ordinaire. Le verrou nommé des routes de
    réconciliation (`with_account_lock`, `GET_LOCK`) ne lit aucune table InnoDB : il n'ouvre aucune vue.
- **Critère** (AC7), à deux volets : aucun verrou ajouté et aucun ordre changé ; **aucun déplacement de la naissance de
  la vue** — chemin par chemin. **Position par chemin** — la preuve (sur `5aad8b35` ; à recopier au Dev Agent Record
  avec la ligne de code retenue). *Colonne « vue » ajoutée, lignes `PUT` et `accept_one_*` corrigées, lignes
  `upsert_primary_in_tx`, W21, W26 et `first_active_bank_account_linked_to` ajoutées en validation P3 (R1 = F1, R3 = F3,
  R8, F5).* « 1re lecture non verrouillante » = l'instruction où naît la vue au sol, puis avec la story ; « n/a » =
  la story n'ajoute aucune lecture dans la transaction.

  | Chemin | Verrous au sol, dans l'ordre | Ce que la story ajoute | Verrou ajouté ? | Ordre changé ? | 1re lecture non verrouillante : au sol → avec la story | Vue déplacée ? |
  |---|---|---|---|---|---|---|
  | W8 `settle_invoice` | facture X (l. 91) → [compte de créance lu sans verrou, étape (2), `sale_receivable_account`] → compte X (l. 131-135) → [compte interne seulement : `accounts` X, l. 170] → [écart d'arrondi seulement : `accounts` X, `rounding_account_for_write` l. 225 → `usable_designated_account … FOR UPDATE` — ajouté en validation P4, F-L1] → exercice X (étape (5), `find_open_covering_date … FOR UPDATE`, l. 244 → `fiscal_years.rs:740`) → `INSERT journal_entries` (clé étrangère → `companies` S) → `INSERT invoice_settlements` → `UPDATE invoices` → audit | `closed_on, archived` lus dans l'instruction du compte, élargie | non | non | étape (2), avant le compte et l'exercice (**au sol**, hors périmètre) → inchangée : la story ne lit rien hors de l'instruction verrouillée | non |
  | W27 `pay_in_tx` (seul ou par `confirm_batch`) | facture fournisseur X (l. 725) → [ligne de créancier lue sans verrou, étape (2), `purchase_payable_line`] → compte X (l. 753-756) → [`accounts` l. 798] → exercice X (l. 843) → `INSERT journal_entries` (l. 874) → `UPDATE supplier_invoices` (l. 895) → lettrage → audit | `closed_on, archived` dans l'instruction du compte, élargie | non | non | étape (2), au sol → inchangée | non |
  | W20 `confirm_batch` | lot X (l. 412) → compte lu **sans** verrou (étape (2), l. 427-441) → W27 facture par facture → `UPDATE payment_batches` (l. 507) → audit | `closed_on` lu dans la lecture existante de l'étape (2) | non | non | étape (2), après le seul verrou (1) → inchangée (même instruction) | non |
  | W17-W19 `create_batch` | compte lu sans verrou (étape (0), l. 85-99) → factures fournisseur X (l. 289) → `INSERT payment_batches` (l. 137 ; clés étrangères → compte S, `companies` S) → `UPDATE` (l. 152) → `INSERT payment_batch_items` (l. 164) → audit (l. 195) | `closed_on` lu dans l'étape (0) existante | non | non | étape (0), avant tout verrou (**au sol**) → inchangée (même instruction) | non |
  | W22-W23 `create_with_transactions` (routes l. 1312 CAMT, l. 2199 CSV) | [recherche de doublons sans verrou, dans la transaction de la route : `find_by_company_and_hash`, `routes/bank_imports.rs:1173` CAMT, `:1978` CSV] → `INSERT bank_imports` (l. 66 ; compte S, `companies` S, `users` S) → `INSERT bank_transactions` (l. 110) → relecture (l. 133-139) → audit | une lecture non verrouillante `closed_on, archived` en tête de la fonction | non | non | route l. 1173 / l. 1978, avant tout verrou → inchangée : la garde lit la vue déjà née (d'où la fenêtre de L1 pour l'import, AC7) | non |
  | W24 `delete_unreconciled_in_tx` | entête X (l. 456-458) → transactions de l'import X et trou (`LOCK_IMPORT_TRANSACTIONS_SQL`, l. 469) → [comptage en mémoire, étape 3, refus `HasReconciled`] → `DELETE` (cascade) → audit (route) | une lecture non verrouillante **après le comptage de l'étape 3**, avant le `DELETE` (validation P4, C-28-4-24) | non | non | aucune **hors l'audit** (« aucune vue n'est ouverte avant les verrous », l. 420-425) → la garde, après le dernier verrou avant la première écriture (étape 2), seul le comptage en mémoire entre les deux | non — ⚠️ **corrigé en P3** (R2 = F2) : entre les étapes 1 et 2, la garde ouvrait la vue avant le verrou de l'étape 2 ; **déplacée en P4** (R1) après le comptage, pour l'ordre des refus, sans effet sur la vue |
  | W9 `reconciliation_cancel::cancel_in_tx` | [route : verrou nommé du compte, `with_account_lock`, `reconciliation.rs:4530`, hors InnoDB] → transaction X (1, l. 286) → règlement ou trou de `uq_invoice_settlements_entry`, facture X (2, l. 295-305) → écriture et exercice X (3, l. 308-319) → [motifs, (4)] → `UPDATE bank_transactions` (5, l. 339) → (6) W25 imbriqué, ou `reverse_in_tx` (`companies` S) → audit | une lecture non verrouillante **entre (3) et (4)** ; en (6), celle de W25 (même compte pour un règlement issu d'un rapprochement, `reconciliation.rs:1998-2000`) | non | non | les motifs de (4), après (3) → la garde, juste avant (4), après (3) ; celle de W25 en (6) lit la vue née en (4) | non — ⚠️ **corrigé en P3** (R1 = F1) : entre (2) et (3), la garde faisait naître la vue avant le verrou d'exercice |
  | W25 `cancel_settlement_in_tx` (client) | facture X (1, l. 824) → règlement X (2, l. 835-836) → écriture et exercice X (2-bis, l. 866) → [motifs, (3)] → dissolution du lettrage → `reverse_owned_in_tx` (`companies` S) → `DELETE` règlement (l. 922) → `UPDATE invoices` → audit | `settlement_bank_account_id` ajouté au `SELECT` verrouillé de (2) (même ligne, même clé primaire) ; une lecture non verrouillante du compte **après (2-bis)**, avant (3) | non | non | les motifs de (3) → la garde, juste avant (3), après (2-bis) | non — ⚠️ **corrigé en P3** (R1 = F1) |
  | W28 `supplier_invoices::cancel_in_tx` | facture X (l. 1061) → écriture d'achat et exercice X (l. 1070-1080) → [motifs, (2), l. 1082-1093] → dissolution (l. 1106) → contre-passation (l. 1119, `companies` S) → `UPDATE` (l. 1133) → audit (l. 1160) | une lecture non verrouillante (facture payée par virement) **après l. 1080**, avant (2) | non | non | les motifs de (2) → la garde, juste avant (2), après le verrou d'exercice | non — ⚠️ **corrigé en P3** (R1 = F1) : avant l. 1070, elle précédait le verrou d'exercice |
  | W29 `supplier_invoices::cancel_settlement_in_tx` | facture X (l. 1282) → [si payée] écriture de règlement et exercice X (1-bis, l. 1290-1312) → [motifs, (2), l. 1314] → dissolution (l. 1340) → contre-passation (l. 1355, `companies` S) → `UPDATE` (l. 1369) → audit (l. 1386) | une lecture non verrouillante **après (1-bis)**, avant (2) | non | non | les motifs de (2) → la garde, juste avant (2), après (1-bis) | non — ⚠️ **corrigé en P3** (R1 = F1) : avant l. 1290, elle précédait le verrou d'exercice |
  | W30 `unreject` (28-5, à relire au rebase) | verrou nommé du compte (`with_account_lock`, hors InnoDB) → transaction X (`find_by_id_for_account_for_update`) → `UPDATE bank_transactions` (aucune colonne à clé étrangère) → audit | une lecture non verrouillante après la transaction ; pré-vol R8 hors transaction | non | non | aucune **hors l'audit** dans la transaction (code de `story/28-5`, `unreject_once`, relu par les lentilles de la P4) → la garde, après le dernier verrou (la ligne) — **à revérifier au rebase** sur le code mergé de la 28-5 | non |
  | W13 `PATCH` | (réglages S si lien, `claims_for_target` de la route) → compte X (l. 410-414) → `accounts` → `UPDATE` (l. 457) → audit | `closed_on` dans l'instruction du compte | non | non | n/a | non |
  | W14 `PUT` | `companies` X (sentinelle, route l. 719) → réglages S (`claims_for_target`, route l. 724) → **principal démis X** (`flip_primary_off_for_company`, route l. 730 → dépôt l. 688, seulement si `isPrimary`) → **compte cible X** (`update_for_company`, route l. 745 → dépôt l. 548) → `UPDATE` (l. 577) → audit | `closed_on` dans l'instruction du compte cible | non | non | n/a | non — ⚠️ **ordre corrigé en P3** (R3 = F3 : la P2 écrivait le compte avant le principal démis). Conséquence : sur un compte clos avec `isPrimary: true`, le principal est démis **avant** le refus `409 BANK_ACCOUNT_CLOSED` lu sous le verrou du compte cible ; le handler rend `Err` sans commit, la transaction est abandonnée et le principal le reste (T-a26) |
  | W15 archivage | pré-vol sans verrou (route l. 839-845, hors transaction) → `companies` X (l. 857) → décomptes sans verrou (l. 860-880) → compte X (`archive_for_company`, l. 619-623) → `UPDATE` (l. 640) → audit | `closed_on` lu dans le pré-vol et dans l'instruction d'`archive_for_company` | non | non | décompte de l. 860-861 (**au sol** ; « l. 862 » corrigé en validation P4, R10) → inchangée | non |
  | `POST /bank-accounts` (W7, W10, W12) | `companies` X (l. 561) → réglages S (l. 567) → principal démis X (l. 587) → `INSERT` (l. 604) → audit | rien | non | non | n/a | non |
  | `upsert_primary_in_tx` (W11, W12) | compte principal X (`… is_primary = TRUE AND archived = FALSE LIMIT 1 FOR UPDATE`, `bank_accounts.rs:285`) → `UPDATE` (l. 301) ou `INSERT` (l. 331) | rien (vise le principal, jamais clos : I3) | non | non | n/a | non |
  | `accept_one_*` (W1-W3, W8 par `accept_one_invoice`) | verrou nommé (`with_account_lock`, route l. 1157, hors InnoDB) → [transaction lue **sans** verrou, `find_pending_by_id_for_account`, l. 1563 ; facture sans verrou, l. 1598 (« Pas de `FOR UPDATE` sur la facture en amont », l. 2079) ; compte sans verrou, l. 1758-1760] → exercice X (`find_open_covering_date`, l. 1911) → `journal_entries::create_in_tx` (l. 1972) → `INSERT invoice_settlements` (l. 1986, compte S implicite) → `UPDATE bank_transactions` (l. 2038) → `UPDATE invoices` (l. 2093) | rien dans la transaction ; pré-vol R1 hors transaction (`find_by_id_for_company` sur le pool, l. 1032-1043) | non | non | n/a (au sol : l. 1563 — 1562 / 1596 corrigés en 1563 / 1598 en validation P4, R10 = F-L6) | non — ⚠️ **ligne corrigée en P3** (F3 : la P2 écrivait « transactions X → facture X → exercice X (l. 1964-1972) ») |
  | `reject`, `manual`, `split` (W4-W6) ; `preview`, `create` d'import | inchangés | `closed_on` lu sur la ligne que le pré-vol existant lit déjà (R2-R4, R6, R7 : `find_by_id_for_company`, pool, autocommit) | non | non | n/a (hors transaction) | non |
  | W21 `cancel_batch` | lot X (`payment_batches.rs:573`) → `UPDATE payment_batches` (l. 589) → audit (l. 619) | rien (`SortieAutorisee`, C-28-4-23) | non | non | n/a | non |
  | W26 `supplier_invoices::create_in_tx` | projets X (l. 433) → comptes S (l. 464) → exercice X (l. 477) → `journal_entries::create_in_tx` (l. 502) → `INSERT supplier_invoices` (l. 524 ; aucune colonne `settlement_*`) — verrous complétés en validation P4 (R10) | rien | non | non | n/a | non |
  | `first_active_bank_account_linked_to` (`bank_accounts.rs:814`, SQL l. 820, `LOCK IN SHARE MODE`) | lecture partagée des comptes liés au compte comptable | rien : garde les clos (C-28-4-7, M44), aucune colonne ajoutée | non | non | n/a | non |
  | **clôture / réouverture** (neuves) | — (absentes du sol) | compte X en première instruction → décomptes sans verrou → `UPDATE` de la même ligne → audit (sans clé étrangère) | chemin neuf : un verrou, rien d'autre | — | — → la clôture : son premier décompte, **après** son verrou (condition de T-a31, la route ne lisant rien dans la transaction avant) ; la réouverture : aucune | — |

  Aucune instruction neuve n'est un `INSERT` ou un `UPDATE` de colonne à clé étrangère : la story n'ajoute donc pas
  non plus de verrou implicite. Les cycles préexistants nommés par la P2 (inversion `companies` / compte de W8 et W27
  contre les routes de gestion ; `create_batch` / `confirm_batch` ; W9 « écriture » / W8 sur le trou de
  `uq_invoice_settlements_entry`) sont **au sol** et inchangés (AC7).
- **Import (W22), suppression d'import (W24), annulation de rapprochement (W9), annulation de rejet (W30)** : les
  cycles que la P1 avait analysés (W9 / W24, W30 / W24) naissaient du verrou exclusif de la garde ; sans lui, ces
  chemins gardent leur suite de verrous au sol, où aucun d'eux ne demande la ligne du compte.
- La clôture **ne prend pas** la sentinelle `companies` (contrairement à l'archivage, `routes/bank_accounts.rs:857`, et
  à la création et au remplacement, l. 561 et 719) : elle n'a besoin d'aucune autre ligne. Le refus du principal se lit
  sur sa propre ligne, verrouillée ; un `PUT` concurrent sur la même ligne est sérialisé par ce même verrou.

### Angles morts assumés (à écrire aux doc-comments)

- Le registre de l'AC8 voit les écritures à nom de table littéral, dans `kesh-db` et `kesh-api`, hors tests ; il ne
  voit pas les trois sites à nom variable — `kesh-db/src/backup.rs:496` et `:548` (restauration d'installation) et
  `kesh-seed/src/lib.rs:586` (remise à zéro de la démo) —, énumérés au doc-comment (AC8, validation P4, R2).
- Sauvegarde prise pendant une clôture : #633 / 28-3.
- Boutons *Supprimer l'import* / *Annuler le règlement* affichés sur l'historique d'un compte clos, refusés au clic
  (manuel, AC14).
- **L1** (AC7, C-28-4-21) : une écriture dont la vue cohérente précède le commit d'une clôture peut encore aboutir sur
  le compte clos (garde non verrouillante) — une opération réelle de plus, sans erreur comptable ; I1 ou I2 défait pour
  ce compte jusqu'au traitement ; remède : rouvrir, traiter, clore. **P2 par la grille, dette assumée et motivée**
  (validation P3) ; **tracée par #651** (validation P4) ; écrite aussi à `docs/api-external.md` (AC16). Numérotation propre à la fiche (L5 est celle du
  code de la route d'archivage).
- Les routes neuves n'ont pas d'enveloppe de rejeu : un seul verrou, en tenant rien, elles ne peuvent pas être victimes
  d'un cycle (AC7).
- L'inversion `companies` → `bank_accounts` de W8 / W27 contre les routes de gestion : **au sol**, inchangée, non
  traitée (AC7).
- Une transaction rejetée bloque la clôture comme une non rapprochée (R13, manuel).
- Le principal s'archive quand tous les autres comptes sont clos (F9, manuel et message).
- La date de clôture n'est contrôlée ni contre les transactions ni contre le futur (C-28-4-3).

### État actuel des fichiers modifiés — ce qui doit être préservé

- `routes/bank_accounts.rs` : l'archivage (l. 823-916) garde ses refus (pré-vol hors transaction pour le `404`,
  sentinelle, `412` transactions, `412` principal, `409` version) et leur ordre relatif ; **seul ajout** (AC5,
  C-28-4-21) : le refus `409 BANK_ACCOUNT_CLOSED` lu dans le pré-vol (avant la sentinelle) et dans le `FOR UPDATE`
  d'`archive_for_company` — aucun verrou neuf. Le `PATCH` garde son
  court-circuit no-op **pour un compte ouvert**.
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

La 28-4a touche `kesh-db` (une migration, l'entité, sept dépôts) et `kesh-api` (`routes/bank_accounts`,
`routes/bank_imports`, `routes/reconciliation`, `exports`, `errors`, `audit_labels`) et `kesh-i18n` : au-delà de cinq
modules. ⚠️ **C'est une dérogation à la règle de découpage, non l'exception qu'elle prévoit** (R14) : l'exception du
`CLAUDE.md` vise les cycles de dépendance Cargo et les merges intermédiaires impossibles à tester ; ce n'est pas le
cas ici, et la dérogation écarte aussi le « story-zero pilote » que la règle prescrit. Elle se motive autrement, et
c'est au Project Lead d'en juger. Justification : (1) **un seul pattern** — une lecture non verrouillante
`ensure_open_in_tx` à des points d'étranglement, ou une colonne lue dans une instruction existante — répété
mécaniquement, sans verrou ajouté ; (2) l'**inventaire clos** est l'unité de preuve : une
moitié de gardes sur `main` ferait mentir l'état « clos » sur l'autre moitié, et le registre de l'AC8 ne peut pas être
vert à moitié ; (3) les touches à `routes/reconciliation.rs` sont réduites à quelques lignes (pré-vols R1-R4, R8), pour limiter
le conflit avec la 28-5, la 28-6, la 28-7. **Risque accepté** : une revue de code plus longue ; la revue du rollout se
fait **fichier par fichier** (§ *Comment splitter*). Signal de non-convergence (sévérité qui recycle) → déclaré au
Project Lead, comme le veut D5.

### Fichiers communs avec les autres stories de l'epic

| Fichier | Stories |
|---|---|
| `crates/kesh-api/src/routes/reconciliation.rs` | 28-5, 28-6, 28-7 (sérialisées) — 28-4a : R1-R4, et R8 sur la route `unreject` de la 28-5 |
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
- T-a26 `PUT`, `PATCH` (non identique **et** identique, no-op), `DELETE` sur un clos → `409` (AC5) ; `DELETE` sur un
  clos **sans** transaction (sinon le `412` masquerait M42) **et** sur un clos **avec** transactions (`409
  BANK_ACCOUNT_CLOSED`, non `412` : le refus du pré-vol précède le décompte, AC5) ; et (validation P3, R3 = F3) `PUT`
  sur un compte clos avec `isPrimary: true`, un autre compte étant principal : `409 BANK_ACCOUNT_CLOSED`, et le
  principal **reste principal** (`is_primary` relu après la requête) — le démotage de `flip_primary_off_for_company`
  (route l. 730) précède le refus, il n'est défait que par l'abandon de la transaction ; aucune trace d'audit.
- T-a27 une ligne par route de la table de l'AC6 — **dix-huit** lignes : les **seize** routes de la table hors
  `PUT` / `PATCH` / `DELETE` (portés par T-a26), plus la seconde ligne W9 de la validation P2, ci-dessous, plus la
  seconde ligne W24 de la validation P4, ci-dessous (16 + 1 + 1 = 18 ; formulation reprise en validation P3, R8, et
  en validation P4), la ligne `unreject` comprise si la 28-5 est mergée : refus **et** absence d'écriture et de trace ; le même
  jeu sur le compte rouvert passe. Lignes construites par SQL direct : AC6, dernier point. ⚠️ **Ligne W9**
  (validation P2, R4) : elle annule un rapprochement de type **`Entry`** (manuel, ventilé ou par règle) — sur un
  rapprochement de facture, l'étape (6) de W9 passe par W25, dont la garde refuse avec le même `409`, et M9 survivrait ;
  une seconde ligne W9 sur un rapprochement **de facture** s'ajoute (refus par l'une ou l'autre garde). ⚠️ **Lignes
  W24 — l'ordre des refus** (validation P4, R1 = F-L3, C-28-4-24), deux cas : (a) import **non rapproché** d'un compte
  clos (transactions `pending`, clôture par SQL direct) → `409 BANK_ACCOUNT_CLOSED`, import intact ; (b) import
  **rapproché** d'un compte clos (construit par l'API : import, rapprochement de toutes ses transactions, clôture) →
  `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` (le refus **du sol**, `details.reconciledCount`), **non**
  `BANK_ACCOUNT_CLOSED` ; rejoué sur le compte rouvert, (b) rend le même refus — c'est ce qui prouve que la clôture ne
  le masque pas. Le cas (b) tue M54 et M55.
- T-a28 `accept` sur un clos : `200`, toutes les propositions en `failed[]` `BANK_ACCOUNT_CLOSED`, aucune ligne
  modifiée ; T-a29 `reject` idem.
- T-a30 (réécrit en validation P1, R5 retiré) : test de dépôt — `settle_invoice` par virement sur un compte clos rend
  `BankAccountClosed` et n'écrit rien (aucune ligne `invoice_settlements`, `journal_entries`, facture inchangée).
- T-a31 le test déterministe de l'AC7 (motifs écrits à l'AC7) : une transaction d'import a **inséré** son entête et
  ses transactions `pending` sans commiter — elle tient, par la clé étrangère, un verrou partagé sur la ligne du
  compte, au sol — ; une clôture lancée en parallèle **attend** sur sa première instruction, puis, au commit de
  l'import, rend `BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` (tue M6). Si la prémisse InnoDB (verrou partagé du parent à
  l'`INSERT` d'un enfant) était fausse, la sonde paniquerait au bout de 10 s : le test la vérifie aussi.
- ~~T-a32~~ — **retiré en validation P2** (C-28-4-21) : un import n'attend plus la garde d'une clôture (garde non
  verrouillante) ; ce qu'il ferait sous concurrence est l'angle mort L1.
- T-a33 registre statique (AC8) ; T-a34 son volet inverse (entrée sans site) par un cas construit.
- T-a35 `audit_label_registry.rs`, `audit_route_registry.rs`, `test_schema_guard.rs`, `migrations_upgrade_path.rs` verts.
- T-a36 `GET /bank-imports` liste l'import d'un compte clos (`200`) ; T-a37 `GET /bank-imports/{id}` d'un import de
  compte clos (`200`) ; T-a38 `GET /reconciliation/proposals?bankAccountId=<clos>` → `200`, liste vide (AC2).
- ~~T-a39~~ — **retiré en validation P2** : la relecture verrouillée de `create_batch` qu'il exerçait n'existe plus
  (M10 sans objet) ; la création de lot pendant une clôture relève de L1.
- ~~T-a40~~ — **retiré en validation P2** : la garde ne sérialise plus deux écrivains (M35 sans objet).
- ~~T-a41~~ — **retiré en validation P2** : une garde non verrouillante ne peut pas former de cycle, quelle que soit
  sa position (M36 à M39 sans objet). Le critère « aucun verrou ajouté » se tient par la table *Position par chemin*
  et par T-a45. *Annotation de la validation P3 (2026-10-13, R1 = F1)* : la raison vaut pour les **cycles**, non pour
  la **vue** — la position de la garde décide où naît l'instantané. T-a41 reste retiré (il portait sur les cycles) ;
  M36 à M39 sont **rétablies** avec cet objet neuf, tuées par `reconciliation_cancel.rs:128` (M36) et T-a48
  (M37 à M39).
- T-a42 `POST /invoices/{id}/settlements` par virement vers un compte **archivé** → l'erreur que la route rend
  aujourd'hui pour un compte inexistant (code relu au dev, écrit au Dev Agent Record), aucune écriture ; clé d'API
  `read-write` comprise (AC6-bis).
- T-a43 `POST /supplier-invoices/{id}/pay` par virement vers un compte archivé → idem (AC6-bis).
- ~~T-a44~~ — **retiré en validation P2** (C-28-4-21) : (i) l'import n'attend plus la garde d'un archivage (garde non
  verrouillante ; la course archivage / import reste la limite L5 de la route, au sol) ; (ii) l'archivage ne prend
  plus la ligne avant son décompte (M49 sans objet). La part séquentielle de (i) passe à T-a46.
- T-a45 statique (kesh-db, test unitaire) : les constantes `ENSURE_OPEN_SQL` et `ENSURE_OPEN_UNARCHIVED_SQL` ne
  contiennent ni `FOR UPDATE`, ni `LOCK IN SHARE MODE`, ni `FOR SHARE` (insensible à la casse) — garde-fou du critère
  de l'AC7 sur les deux fonctions neuves (tue M52). ⛔ Il ne prouve pas le critère sur les instructions élargies (W8,
  W27, W13-W15, W17, W20) : celles-là se vérifient au diff, par la table.
- T-a46 dépôt (séquentiel) : `bank_imports::create_with_transactions` appelé directement sur un compte **archivé** →
  `DbError::NotFound`, et sur un compte **clos** → `DbError::BankAccountClosed` ; dans les deux cas, aucune ligne
  `bank_imports` ni `bank_transactions` (tue M48, et M11 côté dépôt).
- T-a47 appariement de la route d'import (validation P2, R3 / F4) : les deux appels (`routes/bank_imports.rs:1312`
  CAMT, `:2199` CSV) passent par **un** auxiliaire nommé, `import_create_error(e: DbError) -> AppError`
  (`NotFound` → `AppError::BankAccountNotFound` ; tout autre → `e.into()`, donc `BankAccountClosed` → `409`) ; test
  unitaire de l'auxiliaire, **et** assertion textuelle sur la source hors tests, **aplatie avant la recherche**
  (blancs et sauts de ligne réduits à rien, comme `pdftotext … | tr -s`), parce que `rustfmt` peut rompre la chaîne en
  `.await` / `.map_err(import_create_error)?` sur deux lignes (validation P3, F9) : exactement deux appels
  `bank_imports::create_with_transactions(`, chacun suivi, dans la source aplatie, de
  `.await.map_err(import_create_error)` (tue M53 ; ni plus large — elle ne tuerait plus M53 —, ni sensible au
  formatage). Le cas réel —
  un archivage commité entre le pré-vol R7 et la garde — est une course que la story ne monte pas (L5).
- T-a48 (validation P3, R1 = F1) — **entrelacement par virement**, au dépôt, une clôture d'**exercice** concurrente,
  sur le patron de `a_concurrent_close_waits_for_the_unreconciliation` (`crates/kesh-db/tests/reconciliation_cancel.rs:128`) :
  (1) une transaction `closing` exécute `UPDATE fiscal_years SET status = 'Closed' WHERE id = <exercice du règlement>`
  sans commiter ; (2) l'annulation démarre (`tokio::spawn`) ; (3) `attendre_une_requete_en_cours(&pool,
  &["fiscal_years", "FOR UPDATE"], …)` la voit en attente du verrou d'exercice ; (4) `closing.commit()` ; (5) l'annulation
  doit rendre le motif `FiscalYearClosed` et n'avoir rien écrit. Trois cas, chacun réglé **par virement** vers un compte
  bancaire **ouvert** (sans quoi la garde ne passe pas, ce qui est précisément le défaut des trois tests existants,
  montés en espèces ou sur compte interne), dans un exercice autre que celui du jour (la contre-passation verrouille
  l'exercice du jour, qui ne doit pas servir de garde par accident) : (a) W25, `invoice_settlements_write::cancel_settlement`
  d'un règlement `SettlementChoice::BankTransfer` → `SettlementNotCancellable { FiscalYearClosed }`, règlement
  toujours là ; (b) W29, `supplier_invoices::cancel_settlement` d'une facture payée par virement →
  `SettlementNotCancellable { FiscalYearClosed }` ; (c) W28, `supplier_invoices::cancel` de cette même facture payée
  par virement, l'exercice clos étant celui de l'**achat** → `SupplierInvoiceNotCancellable { FiscalYearClosed }`.
  Avec la garde à sa place, la vue naît après l'attente et les motifs voient l'exercice clos ; avec la garde avancée
  avant le verrou d'exercice, elle naît avant, les motifs voient l'exercice ouvert et l'annulation passe : le test
  rougit (tue M37, M38, M39). W9 n'y figure pas : le test existant de `reconciliation_cancel.rs:128` le couvre déjà
  (M36, AC7). ⚠️ Raisonnement statique, non exécuté en validation : la prémisse (une garde avancée fait réussir
  l'annulation) se **vérifie au dev** par les mutations elles-mêmes — une mutation qui survit dirait que la prémisse
  est fausse, et cela s'écrirait au Dev Agent Record.
- T-a49 (validation P4, F-M1 ; C-28-4-24) — **entrelacement sur le verrou de trou de l'étape 2 de W24**, au dépôt,
  sur le patron de T-a31 et de `a_concurrent_close_waits_for_the_unreconciliation` : il tient la **fraîcheur de la
  garde elle-même** pendant l'attente de l'étape 2, ce que l'ordre des refus (T-a27, lignes W24) ne voit pas.
  Montage : comptes A et B ouverts, non principaux, de la même société ; un import N sur A **sans transaction**
  (`INSERT bank_imports` par SQL direct) ; (1) une connexion `autre` ouvre une transaction et y crée un import N+1
  sur **B**, **après** N, avec au moins une transaction (`create_with_transactions`), sans commiter — ses lignes sont
  les premières de `idx_bank_transactions_import` au-delà de N (index non scopé société,
  `20260504000001_bank_imports.sql:87`), et elle ne tient, par clé étrangère, qu'un verrou partagé sur **B** ; (2) la
  suppression de N démarre (`tokio::spawn`, sa propre transaction) : l'étape 1 verrouille l'entête N, l'étape 2
  (`FORCE INDEX (idx_bank_transactions_import)`, plage vide pour N) demande le verrou de la première ligne au-delà,
  non commitée, et **attend** ; (3) `attendre_une_requete_en_cours(&pool, &["idx_bank_transactions_import", "FOR
  UPDATE"], || handle.is_finished())` doit rendre `true` — s'il rend `false`, la suppression a fini sans attendre et
  le test **échoue en le disant** (« prémisse InnoDB fausse : l'étape 2 n'attend pas ») ; (4) la clôture de A
  (`close_in_tx`, troisième connexion) passe — A n'a ni `pending`, ni lot, n'est pas principal, et personne ne tient
  sa ligne — et **commite** ; (5) `autre.rollback()` ; (6) la suppression doit rendre
  `DbError::BankAccountClosed { bank_account_id: A }`, et l'import N être toujours là. Avec la garde à sa place, la
  vue naît après l'attente et voit A clos ; avec la lecture de la garde avancée avant l'étape 2 (M56, ou M54), elle
  naît avant, voit A ouvert, et N est supprimé : le test rougit. ⚠️ Raisonnement statique sur InnoDB (MariaDB 10.11),
  non exécuté en validation ; la prémisse — l'attente de l'étape 2 sur une ligne non commitée d'un autre import — est
  **contrôlée par le test lui-même** à l'étape (3). Si elle se révèle fausse au dev, T-a49 se retire, M56 devient
  « non tuée, assumée » avec la raison écrite à sa ligne, et cela s'écrit au Dev Agent Record.

**Partie b** (Vitest, Playwright) :

- T-b1 `listBankAccounts` construit les quatre URL ; T-b2 `parseBankAccount` porte `closedOn` ; T-b3/T-b4
  `closeBankAccount` / `reopenBankAccount` (méthode, corps).
- T-b5 écran : ligne close → badge et seule action *Rouvrir* ; T-b6 principal → *Clore* désactivé ; T-b7 dialogue :
  date préremplie, avertissement présent si solde ≠ 0, absent si 0 ou `null` ; T-b8 refus `412` affiché par le message
  serveur ; T-b9 bascule *Afficher les comptes clos*.
- T-b10 réglages de facturation : `listBankAccounts(…, true)` (second paramètre `includeClosed`).
- T-b11 `failed-proposal-label` : `BANK_ACCOUNT_CLOSED`.
- T-b12 E2E : clore → absent des six sélecteurs et de la tuile ; rouvrir → présent (AC12).
- T-b13 E2E : clore, voir le badge, rouvrir (parcours de l'écran, captures AC17).
- T-b14 `i18n-keys.test.ts`, `lint-i18n-ownership`.

### Mutations

Chacune doit faire rougir au moins un test nommé ; une mutation équivalente se déclare avec sa raison.

| # | Mutation | Test attendu rouge |
|---|---|---|
| M1 | `ensure_open_in_tx` ne contrôle plus `closed_on` | T-a27 (plusieurs lignes) |
| M2 | filtre de liste ignore `include_closed` (défaut inclut les clos) | T-a2-T-a7 |
| M3 | clôture ne refuse plus le principal | T-a19 (la contrainte rend alors une erreur SQL, non le `412`) |
| M4 | décompte *pending* retiré | T-a20 |
| M5 | décompte des lots retiré | T-a21 |
| M6 | décompte *pending* en lecture faite **avant** le `FOR UPDATE` | T-a31 |
| M7 | réouverture ne remet pas `closed_on` à NULL | T-a23 |
| M8 | garde W8 retirée (`closed_on` ôté de l'instruction de `settle_invoice`, l. 131-135 — site **unique** depuis la validation P1) | T-a27 (règlement), T-a30 |
| M9 | garde W9 retirée | T-a27 (annulation d'un rapprochement de type `Entry` — sur un rapprochement de facture, la garde de W25 refuse aussi : R4 de la P2) |
| M10 | ~~relecture verrouillée de W17 retirée~~ | **sans objet** depuis la validation P2 : la relecture n'existe plus (C-28-4-21) |
| M10-bis | refus « clos » de l'étape (0) de `create_batch` retiré | T-a27 (création de lot) — **n'est plus équivalente** depuis la validation P2 : l'étape (0) est désormais la seule garde de W17 |
| M11 | garde W22 retirée (import) | T-a27 (import, pré-vol R7 neutralisé aussi), T-a46 (compte clos) |
| M12 | garde W24 retirée | T-a27 (suppression d'import) |
| M13 | garde W25 retirée | T-a27 (annulation de règlement client) |
| M14 | garde W27 retirée | T-a27 (paiement fournisseur) — la ligne de confirmation est refusée d'abord par le refus précoce de l'étape (2) de `confirm_batch` |
| M14-bis | refus précoce de l'étape (2) de `confirm_batch` retiré | **équivalente** : chaque facture passe par W27, qui rend le même `409` ; la transaction du lot est abandonnée entière (`confirm_batch` n'écrit rien avant la boucle, à vérifier au dev — sinon, la mutation devient tuable par T-a27 et se nomme). ⚠️ Équivalente **pour un lot sain** seulement (validation P3, R8) : un lot dont la première facture n'est plus `open` rend, sans le refus précoce, `IllegalStateTransition` de `pay_in_tx` avant d'atteindre le compte, et `409 BANK_ACCOUNT_CLOSED` avec lui — aucun test ne monte ce cas, qui ne change que le code du refus |
| M15 | garde W29 retirée | T-a27 (annulation de règlement fournisseur) |
| M16 | pré-vol R1 retiré | T-a28 — raison corrigée en validation P3 (R8) : la ligne `accept` de T-a28 casse I1 **à dessein** par SQL direct (transaction `pending` sur un compte clos, AC6) et `accept_one_*` n'a **aucune** garde (C-28-4-14) ; sans R1, le lot **réussit** et écrit — T-a28 rougit sur l'écriture et sur l'absence de `failed[]` `BANK_ACCOUNT_CLOSED`, non parce qu'« I1 rend un autre code » |
| M17 | pré-vol R2 retiré | T-a29 |
| M18 | pré-vols R3/R4 retirés | T-a27 (manuel, ventilé) |
| M19 | ~~appariement R5 retiré~~ | **sans objet** depuis la validation P1 (R5 retiré, C-28-4-14) |
| M20 | pré-vol R6 retiré (aperçu) | T-a27 (aperçu) |
| M20-bis | pré-vol R7 retiré seul (import) | **équivalente** : la garde W22 rend le même `409` sans rien insérer ; R7 n'épargne que l'analyse du fichier |
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
| M33 | pré-vol R8 et garde W30 retirés (`unreject`, après le rebase sur la 28-5) | T-a27 (ligne `unreject`, construite par SQL direct) |
| M34 | `GET /reconciliation/proposals` refuse un compte clos (garde d'archivage l. 610 étendue) | T-a38 |
| M35 | ~~garde en `LOCK IN SHARE MODE` au lieu de `FOR UPDATE`~~ | **sans objet** depuis la validation P2 : la garde ne verrouille plus ; son inverse est M52 |
| M36 | garde W9 **avancée** avant le verrou d'exercice (entre (2) et (3)) — *rétablie en validation P3, objet neuf (R1 = F1) ; l'énoncé de la P1, « posée après (3) », visait un ordre de verrous, et la P2 l'avait déclarée sans objet* | `a_concurrent_close_waits_for_the_unreconciliation` (`crates/kesh-db/tests/reconciliation_cancel.rs:128`), **existant** : sa transaction porte un compte bancaire, la garde y passe ; avancée, elle fige la vue avant l'attente, les motifs voient l'exercice ouvert et la contre-passation réussit |
| M37 | garde W25 avancée avant (2-bis) — *rétablie en validation P3* | T-a48 (a) ; `invoice_settlement.rs:1266` ne la voit pas (règlement en espèces, `settle_cash`) |
| M38 | garde W29 avancée avant (1-bis) — *rétablie en validation P3* | T-a48 (b) ; `supplier_invoices_repository.rs:1714` ne la voit pas (`paid_invoice` règle sur compte interne) |
| M39 | garde W28 avancée avant le verrou de l'écriture d'achat et de son exercice (entre l. 1061 et l. 1070) — *rétablie en validation P3* | T-a48 (c) ; `supplier_invoices_repository.rs:2059` ne la voit pas (même montage) |
| M40 | garde W28 retirée | T-a27 (annulation d'une facture fournisseur payée) |
| M41 | AC5 : `set_journal_account_id_for_company` ne refuse plus | T-a26 (`PATCH` non identique) |
| M42 | AC5 : l'archivage ne refuse plus un compte clos (les deux lectures : pré-vol **et** `archive_for_company`) | T-a26 (`DELETE` sans transaction) |
| M43 | AC5 : refus « clos » du pré-vol de l'archivage replacé après le décompte | T-a26 (`DELETE` avec transactions : `412` au lieu de `409`) |
| M44 | `first_active_bank_account_linked_to` exclut les clos | T-a9 |
| M45 | colonne `closed_on` absente ou vide dans `bank_accounts.csv` | T-a10 |
| M46 | W8 n'écarte plus l'archivé | T-a42 |
| M47 | W27 n'écarte plus l'archivé | T-a43 |
| M48 | W22 n'écarte plus l'archivé | T-a46 (dépôt) — par la route, le pré-vol R7 refuse d'abord |
| M49 | ~~archivage : décompte lu avant le verrou de la ligne~~ | **sans objet** depuis la validation P2 : l'archivage ne prend plus la ligne avant son décompte (C-28-4-21 remplace C-28-4-16) |
| M50 | ~~relecture de W17 n'écarte plus l'archivé~~ | **sans objet** depuis la validation P2 (relecture retirée). *Avant la P2 :* **équivalente** pour la suite : l'étape (0) refuse l'archivé avec la même erreur ; seule une course archivage / création de lot la distingue, et aucun test déterministe n'est prescrit (coût d'un T-a39 de plus pour une fenêtre de quelques millisecondes) |
| M51 | AC5 : refus « clos » retiré de la seule lecture verrouillée d'`archive_for_company` (pré-vol gardé) | **équivalente** pour la suite : le pré-vol rend le même `409` ; seule une clôture commitée entre le pré-vol et le `FOR UPDATE` la distingue, et aucun test de course n'est prescrit (le refus sous verrou empêche un compte à la fois clos et archivé, sans conséquence comptable) |
| M52 | une fonction de garde (`ensure_open_in_tx` ou `ensure_open_unarchived_in_tx`) écrite en `FOR UPDATE` (ou `LOCK IN SHARE MODE`) | T-a45 |
| M53 | appariement `import_create_error` retiré à l'un des deux appels de la route d'import (rejouée pour CAMT, l. 1312, puis pour CSV, l. 2199) | T-a47 |
| M54 | garde W24 (lecture **et** refus) avancée avant l'étape 2 (entre les étapes 1 et 2) — *ajoutée en validation P3 (R2 = F2) ; reclassée en validation P4 (R1, F-M1)* | T-a27, ligne W24 (b) : posée avant le comptage, la garde rend `BANK_ACCOUNT_CLOSED` au lieu de `BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` ; et T-a49. ⚠️ *Déclarée « équivalente sous MariaDB 10.11 » en P3* : la raison ne regardait que l'effet de la vue sur l'étape 3 (lecture verrouillante) et oubliait la **fraîcheur de la garde elle-même** pendant l'attente de l'étape 2 (F-M1) ; elle était fausse, et la nouvelle position (après le comptage, C-28-4-24) rend la mutation tuable par un test séquentiel |
| M55 | garde W24 replacée **entre les étapes 2 et 3** (la position de la P3, avant le comptage) — *ajoutée en validation P4* | T-a27, ligne W24 (b) : un import rapproché d'un compte clos rend `BANK_ACCOUNT_CLOSED` au lieu du refus du sol |
| M56 | **lecture** de la garde W24 avancée avant l'étape 2, refus évalué à sa place (après le comptage) — la seule part de M54 qui ne touche que la vue — *ajoutée en validation P4 (F-M1)* | T-a49. *Repli écrit d'avance* : si la prémisse de T-a49 se révèle fausse au dev, M56 est **non tuée, assumée**, avec cette raison — « elle ne se distingue de la position prescrite que par la fraîcheur de la garde pendant l'attente de l'étape 2 ; sans attente observable, indiscernable » —, bornée à **MariaDB 10.11, CI et dev** (`mariadb:10.11` : `docker-compose.yml:4`, `docker-compose.dev.yml:56`, `ci.yml:50`, `release.yml:21`) ; la production admet « MariaDB 10.11+ » (`docker-compose.prod.yml:8`), où `innodb_snapshot_isolation` (≥ 11.6) ferait rendre `ER_CHECKREAD` à l'étape 2 (R12) |

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
| 2026-10-13 | Validation P1 (Sonnet, lentilles R et F) et sa remédiation (Opus 5.5) | **Bilan** : R — 0 CRITICAL, 1 HIGH, 6 MEDIUM, 7 LOW ; F — 0 CRITICAL, 3 HIGH, 3 MEDIUM, 5 LOW (rapports `kesh-gate-logs/28-4-validate-p1-{R,F}.md`, hors dépôt). Convergences : F2 = R1 (W28), F4 ≈ R2 (W24), F3 = R12 (archivé). **HIGH** — F1 : la règle « garde après les verrous métier » prenait le compte après l'exercice dans W9, W25, W29 (cycle avec `settle_invoice` / `pay_in_tx`) → règle « pièces → compte → exercice », table des positions par chemin, `accept_one_invoice` sans verrou (I1), garde de W8 dans l'instruction de `settle_invoice` et non dans `create_in_tx`, appariement R5 et mutation M19 sans objet (C-28-4-14). F2 = R1 : W28 annule une facture payée et détache `settlement_bank_account_id` → **gardé** (ligne de l'AC6, T-a27, M40, M39 ; C-28-4-15). F3 = R12 : archivé intégré, borné aux gestes qui créent (W8, W27, W17-W19, W22-W23), erreurs existantes, annulations interdites de lire `archived` → **AC6-bis**, deux fonctions de garde, `### Corrigé` et note d'`api-external.md` dans la 28-4a (C-28-4-12 réécrit). **MEDIUM** — F4 = R2 : W24 entre les étapes 1 et 2, démonstration d'absence de cycle avec l'import, cycle résiduel W9 / W24 rejoué des deux côtés. R3 : message du `412` qui garde la voie de la 27-1 (test `bank_accounts_e2e.rs:2316-2320` inchangé), G9 nommé, six clés y entrent. R4 : T-a36 à T-a38, M34. R5 : M8 (site unique), M10 (T-a39), M20 scindé, M10-bis / M14-bis / M20-bis / M50 déclarées équivalentes, mutations neuves d'ordre, de mode de verrou, `PATCH`, `DELETE`, C-28-4-7, CSV (M35 à M45). R6 : lignes de T-a27 construites par SQL direct nommées, R8 avant la classification de la 28-5. R7 : § *Solde comptable*, `admin-manual.tex:1943`, tuile de l'accueil (C-28-4-18). F5 : constat `ignore_missing` exact, `kesh-seed/src/lib.rs:586` `HorsPerimetre`. F6 : archivage — verrou de ligne et refus « clos » avant le décompte (C-28-4-16 ; ferme aussi L5 pour les imports, T-a44, M49). **LOW** — R8 (colonne de W25 à ajouter), R9 (deux booléens, consommateurs nommés ; C-28-4-20), R10 (commentaires exclus), R11 (sept dépôts, l. 141, assertions de `audit_route_registry.rs`), R13 (rejetées, manuel), R14 (dérogation dite comme telle), F7 (§ *Rollback*), F8 (`QueryBuilder`), F9 (principal archivable, message ; C-28-4-19), F10 (cinq sous-sections d'`api-external.md`), F11 (motifs). **Constat neuf de la remédiation** : inversion `companies` → `bank_accounts` par clé étrangère, préexistante pour W8 / W27, étendue par la garde — angle mort écrit (C-28-4-17). **Recomptes** (fiche après remédiation) : 18 AC (AC1-AC17 + AC6-bis) ; 44 T-a (T-a1-T-a44) + 14 T-b ; mutations M1-M50 + trois `-bis` = 53 numéros, dont M19 sans objet et quatre équivalentes déclarées (M10-bis, M14-bis, M20-bis, M50) ; 29 sites W1-W29 (+ W30), sites classés « Gardée » hors W30 : **16** (W8, W9, W13-W15, W17-W20, W22-W25, W27-W29 ; 15 avant la remédiation — le « 16 → 17 » de F2 compte un de trop), recompté par `grep -E "^\| W[0-9]+ \|"` sur la table ; choix C-28-4-1 à C-28-4-20. Validation P2 : `28-4-validate-prompt-p2.md` (Opus). |
| 2026-10-13 | Validation P2 (Opus 5.5, lentilles R et F) et sa remédiation (Opus 5.5) — **conception simplifiée** | **Trend** : P1 — 4 HIGH (R 0/1/6/7, F 0/3/3/5, en CRIT/HIGH/MED/LOW) ; P2 — R 0/0/4/2, F 0/1/5/7 (rapports `kesh-gate-logs/28-4-validate-p2-{R,F}.md`, hors dépôt), soit 1 HIGH, 9 MEDIUM, 9 LOW. **Signal D5 levé** : la sévérité ne baisse pas (HIGH → HIGH), et presque tous les défauts de la P2 **naissent de la remédiation P1** (garde exclusive, archivage verrouillé avant son décompte, analyse de cycles qui s'étend à chaque correctif) — c'est le **recyclage** que désigne l'amendement D5 de la règle de découpage, non une revue qui trouve des défauts d'origine neufs. **Décision de l'orchestrateur** : simplifier la conception plutôt que corriger la sentinelle (C-28-4-21, qui remplace C-28-4-16 et C-28-4-17, et C-28-4-14 en partie) — la garde est une lecture cohérente **non verrouillante** (`ensure_open_in_tx`, `ensure_open_unarchived_in_tx`), ou une colonne ajoutée à une instruction qui lit déjà le compte ; l'archivage garde sa séquence au sol (refus « clos » au pré-vol et dans le `FOR UPDATE` existant) ; critère décidable de l'AC7, « aucun verrou ajouté, explicite ou implicite par clé étrangère, aucun ordre changé, à aucun chemin de `origin/main` », prouvé par la table *Position par chemin* refondue ; angle mort **L1** (une écriture dont la vue précède le commit d'une clôture peut encore aboutir — sans erreur comptable, P3 au plus, manuel utilisateur, pas d'issue). Aucun chemin ne déroge au critère : `retry_app_on_deadlock` n'est ajouté nulle part. **Findings corrigés** (9) : R3 = F4 — l'appariement `DbError::NotFound` → `BankAccountNotFound` n'existait pas ; prescrit aux deux appels (`routes/bank_imports.rs:1312` CAMT, `:2199` CSV) par un auxiliaire `import_create_error`, « aucun code neuf » borné, T-a47, M53 ; R4 — T-a27 gagne une ligne W9 de type `Entry` (qui tue M9) et une ligne de facture, la table nomme W25 imbriqué en (6) ; F6 — second inventaire clos des appelants de `invoice_settlements::create_in_tx` (AC8, C-28-4-22, sur le précédent `letterings_lexical.rs:577-627`) ; F7 — W24 ne figure plus dans aucune liste de l'inversion `companies` ; F8 — l'index `uq_bank_imports_company_hash` (supprimé par `20260507000001`) n'est plus cité ; F10 — exercice de `settle_invoice` à l'étape (5) (l. 244, `fiscal_years.rs:740`), de `pay_in_tx` à l. 843 ; R5 — ligne W1 ; R6 — L5 à `routes/bank_accounts.rs:853-856`. **Sans objet après simplification** (10) : R1 et F5 (cycles des routes de gestion, partenaires sous-estimés, raison de l'écartée de C-28-4-17) — la garde ne prend plus le compte, l'archivage plus la ligne avant son décompte ; seule l'inversion **au sol** de W8 / W27 reste, nommée à l'AC7 comme préexistante ; R2 et F3 (T-a44 (ii), M49, « L5 se ferme ») — test et mutation retirés, L5 reste ouverte et dite ; **F1 (HIGH, `unreject` / W24)** — vérifié dans la fiche 28-5 (worktree `kesh-28-5`, `da25bf1b`, § *Séquence de la route*, l. 420-467, lecture seule) : W30 prend le verrou nommé du compte (hors InnoDB, que W24 ne prend pas), la ligne `bank_transactions` en `FOR UPDATE`, un `UPDATE` sans colonne à clé étrangère, un audit sans clé étrangère ; avec une garde non verrouillante, W30 ne demande plus la ligne du compte, W24 l'attend au plus sur la transaction, sans réciproque : **aucun cycle**. La phrase de la 28-5 qui annonce un `FOR UPDATE` du compte dans `unreject` (l. 462-465) devient fausse — à relire au rebase, non modifiée ici ; F2 — le cycle introduit avec `accept_one_invoice` naissait de la garde ; la phrase « aucun verrou du compte » est corrigée (« aucune garde ; un verrou partagé implicite par clé étrangère, au sol ») ; F9 — plus de montée de verrou possible (aucune garde verrouillante, l'archivage n'appelle plus de garde) ; F11 — la position de la garde de W25 imbriquée ne règle plus aucun ordre, la table la nomme (R4) ; F12 et F13 — cycles préexistants que la garde « élargissait » : elle ne les touche plus, nommés à l'AC7 comme au sol. **Recomptes** (fiche après remédiation, par `grep` sur la fiche) : 18 AC ; T-a1-T-a47 (47 numéros, dont 5 retirés : T-a32, T-a39, T-a40, T-a41, T-a44 ; **42 actifs**) + 14 T-b ; mutations M1-M53 + trois `-bis` = **56** numéros, dont **9 sans objet** (M10, M19, M35-M39, M49, M50), **3 équivalentes** déclarées (M14-bis, M20-bis, M51 ; M10-bis redevient tuable), **44 à tuer** ; 29 sites W1-W29 (+ W30), 16 « Gardée » hors W30 (inchangé) ; T-a27 à dix-sept lignes ; choix C-28-4-1 à C-28-4-22. Validation P3 : `28-4-validate-prompt-p3.md` (Sonnet, lentilles R et F, axe obligatoire : le critère de l'AC7 chemin par chemin). ⚠️ *Annotation datée du 2026-10-13 (validation P3, R1 = F1)* : les « sans objet » de cette entrée qui reposent sur « la position de la garde ne règle plus aucun ordre » — **F11** (W25 imbriquée), et la raison donnée à M36-M39 et au retrait de T-a41 — sont vrais des **verrous** et faux de la **vue** : la position de la garde décide où naît l'instantané, et celle que cette remédiation a laissée faisait naître la vue avant le verrou d'exercice. Voir l'entrée P3. Les chiffres de cette entrée restent ceux de sa date. |
| 2026-10-13 | Validation P3 (Sonnet 5.5, lentilles R et F) et sa remédiation (Opus 5.5) — **règle de placement** | **Trend** (CRIT/HIGH/MED/LOW) : P1 — R 0/1/6/7, F 0/3/3/5 ; P2 — R 0/0/4/2, F 0/1/5/7 ; **P3 — R 0/1/2/5, F 0/1/2/6** (rapports `kesh-gate-logs/28-4-validate-p3-{R,F}.md`, hors dépôt). Convergences : R1 = F1 (HIGH), R2 = F2, R3 = F3 (MEDIUM) ; parmi les LOW, R5 = F4, R6 ⊃ F8, R7 = F7, R8 ⊃ F5. **Signal D5 levé une seconde fois** : la sévérité ne baisse pas (HIGH → HIGH), et le HIGH **naît de la remédiation P2** — la simplification C-28-4-21 a rendu non verrouillante une garde laissée à sa position P1, si bien qu'une lecture ordinaire précède désormais le verrou d'exercice. C'est le recyclage que désigne D5 ; mais la correction est **locale** (le placement, site par site), elle ne change ni la règle métier ni la simplification, qui est maintenue sur le principe — déclaré à l'orchestrateur. À noter : la position d'**origine** (P0, « garde après les verrous métier », C-28-4-14) était la bonne pour la vue ; la P1 l'a avancée pour éviter des cycles qui ne naissaient que d'une garde verrouillante ; la P2 a retiré le verrou sans revenir sur la position. **HIGH R1 = F1** (confirmé par l'orchestrateur, `reconciliation_cancel.rs:246-250`) — garde de W9, W25, W28, W29 avant le verrou d'exercice : vue née avant l'attente, une annulation qui attend derrière une clôture d'exercice relirait l'exercice ouvert (erreur comptable). Correctifs : règle de placement à l'AC7 et en annotation datée de C-28-4-21 (*après le dernier verrou que le chemin prend avant sa première lecture ordinaire*), doc-comments du sol cités (`reconciliation_cancel.rs:247-252` et `:296`, `supplier_invoices.rs:1040-1045`, `invoice_settlements_write.rs:858-861`, `bank_imports.rs:389-391` et `:420-425`) ; critère de l'AC7 à **deux volets** (« aucun déplacement de la naissance de la vue ») ; table *Position par chemin* refaite avec la colonne « 1re lecture non verrouillante : au sol → avec la story » et « vue déplacée ? », **non** sur chaque ligne ; positions W9 entre (3) et (4), W25 entre (2-bis) et (3), W28 après `supplier_invoices.rs:1070-1080`, W29 après (1-bis) ; M36-M39 **rétablies** (objet inverse : garde avancée) — M36 tuée par le test **existant** `reconciliation_cancel.rs:128` (sa transaction porte un compte bancaire, la garde y passe), M37-M39 par **T-a48** (neuf : entrelacement par virement, W25, W29, W28), les trois tests existants de ces chemins réglant en espèces ou sur compte interne ; T-a41 reste retiré (il portait sur les cycles), annoté. **MEDIUM R2 = F2** — W24 après l'étape 2 ; M54 (garde avancée) **équivalente sous MariaDB 10.11**, raison écrite. **MEDIUM R3 = F3** — lignes `PUT` (principal démis l. 730 **avant** le compte cible) et `accept_one_*` (lectures sans verrou, exercice l. 1911) corrigées ; phrase de l'AC7 sur I1 (la transaction reste `pending` dans l'état commité, non « tenue par son verrou ») ; T-a26 gagne `PUT` sur un clos avec `isPrimary: true`. **LOW** — R4 (cinq assertions d'`audit_route_registry.rs`, l. 991 et 997, message de `traced`) ; R5 = F4 (`create_with_transactions` ne rend `NotFound` que par un `fetch_one` inatteignable) ; R6 = F8 (L1 **classée P2** par la grille, dette assumée et motivée, écrite aussi à `api-external.md` ; fenêtre de l'import qui couvre l'analyse du fichier ; W21 reclassé `SortieAutorisee`, cinquième statut du registre, C-28-4-23) ; F5 et R8 (lignes `upsert_primary_in_tx`, W21, W26, `first_active_bank_account_linked_to` ajoutées à la table ; T-a27 « 16 + 1 = 17 » ; raison de M16 corrigée ; M14-bis équivalente pour un lot sain seulement ; `BankAccountSummaryRaw` l. 62) ; F6 (macro `columns!()`, non `COLUMNS`) ; F7 = R7 (sous-`SELECT` d'`audit_log` sur `users`, file d'attente InnoDB) ; F9 (assertion de T-a47 sur la source aplatie). **Écarts à la consigne** : numéros de ligne cités relus au sol (`reconciliation_cancel.rs:247-252` / `:296` et `bank_imports.rs:389-391` au lieu de 248 / 298 / 390) ; volet « vue » formulé pour couvrir aussi la garde placée **juste avant** la première lecture du sol, sans verrou entre elles (qui devient alors la première lecture), et le chemin sans lecture ordinaire au sol (W24, W30). **Recomptes** (par `grep` sur la fiche, après remédiation) : 18 AC ; T-a1-T-a48 (48 numéros, 5 retirés, **43 actifs**) + 14 T-b ; mutations M1-M54 + trois `-bis` = **57** numéros, dont **5 sans objet** (M10, M19, M35, M49, M50), **4 équivalentes** (M14-bis, M20-bis, M51, M54), **48 à tuer** ; 30 sites W1-W30, 16 « Gardée » hors W30 (inchangé ; W21 passe d'`Invariant` à `SortieAutorisee`) ; T-a27 à dix-sept lignes ; choix C-28-4-1 à C-28-4-23 (un neuf, C-28-4-23 ; C-28-4-14, -15 et -21 annotés). Validation P4 : `28-4-validate-prompt-p4.md` (Opus, lentilles R et F, axe obligatoire : le double critère, verrous et naissance de la vue, chemin par chemin sur `origin/main`). |
| 2026-10-13 | Validation P4 (Opus 5.5, lentilles R et F) et sa remédiation (Opus 5.5) | **Trend** (CRIT/HIGH/MED/LOW) : P1 — R 0/1/6/7, F 0/3/3/5 ; P2 — R 0/0/4/2, F 0/1/5/7 ; P3 — R 0/1/2/5, F 0/1/2/6 ; **P4 — R 0/0/3/9, F 0/0/2/7** (rapports `kesh-gate-logs/28-4-validate-p4-{R,F}.md`, hors dépôt). **Le HIGH est à 0 depuis la P4** ; les MEDIUM sont **distincts** et ne recyclent pas un défaut de conception — sauf **F-M1**, né de la remédiation P3 (raison de M54) ; R3 = F-M2 est lui aussi daté de la P3 par les deux lentilles (reclassement P2 sans issue), mais c'est un défaut de **traçage**, réglé hors fiche par l'issue. Signal D5 : non levé (sévérité en baisse, HIGH → 0). Double critère de l'AC7 tenu sur tous les chemins par les deux lentilles (aucun verrou ajouté, aucun ordre changé, aucune vue déplacée). Convergences : R3 = F-M2, R1 ⊃ F-L3, R11 = F-L4, R9 = F-L7, R10 ⊃ F-L6 ; F-M1 recoupe R12. **MEDIUM** — **R3 = F-M2** (L1 P2 sans issue) : réglé par l'orchestrateur, qui a ouvert **#651** (`bug`, `P2`, piste du rejeu) ; #651 cité à l'AC7 (L1), à l'AC14, aux angles morts, à la note d'`api-external.md` (AC16), au CHANGELOG prescrit, au registre (annotation P4 de C-28-4-21), aux tâches T4, T9, T14 ; corps de PR : 28-4a `refs #642` + `refs #651`, 28-4b `closes #642` + `refs #651` ; la promesse de #642 (« refusée côté serveur ») bornée, au périmètre, à l'AC14 et au CHANGELOG, à « hors de la fenêtre de #651 ». **R1** (W24, impasse « rouvrez-le ») : garde **après le comptage de l'étape 3, avant le `DELETE`** (C-28-4-24, décision de l'orchestrateur) — ordre des refus `404` → `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` → `409 BANK_ACCOUNT_CLOSED`, écrit à l'AC6, testé (T-a27, deux lignes W24), dit au manuel (AC14) et à `api-external.md` ; la position vérifiée contre la règle de placement (W24 n'a aucune lecture ordinaire hors l'audit ; entre l'étape 2 et la garde, seul le comptage en mémoire). **R2** (restauration mal située, « le seul site connu ») : `kesh-db/src/backup.rs:496` et `:548` (`restore_body`, l. 473 ; `restore_tables_in_tx`, l. 444 ; appelée par `routes/admin.rs:295`) nommés à l'inventaire, à l'I1, à l'AC8 (trois sites énumérés) et aux angles morts ; le statut `HorsPerimetre` de l'AC8 ne la vise **pas** — l'extraction ne la voit pas, une entrée sans site ferait rougir le diff inverse : elle est hors périmètre **au doc-comment**. **F-M1** (M54 non équivalente) : avec la position neuve, M54 est tuée **séquentiellement** par T-a27 (ligne W24 b) ; sa part « vue seule » devient **M56**, tuée par **T-a49** (neuf, montage de F-M1 : import vide, import suivant non commité sur un autre compte, clôture commitée pendant l'attente de l'étape 2), dont l'étape (3) contrôle la prémisse InnoDB ; repli écrit d'avance (M56 « non tuée, assumée », raison exacte, bornée à MariaDB 10.11 CI et dev — R12) ; **M55** neuve (position P3, tuée par T-a27). **LOW** — R4 (sous-`SELECT` de l'audit exclus du second volet, AC7 et table), R5 (W22 « en tête de la fonction »), R6 (R8 lecture neuve), R7 (raison de l'appelant `accept_one_invoice` ; sous L1, le pré-vol protège), R8 (fenêtre de l'import à l'AC14), R9 = F-L7 (exception de C-28-4-23 au périmètre, à l'AC6, à l'AC14), R10 = F-L6 (1563 / 1598, l. 860-861, `parseBankAccount` l. 80, verrous de W26, citation de W29 réattribuée à `supplier_invoices.rs:1290-1294`), R11 = F-L4 (bras `DbError::BankAccountClosed` dans le `match` d'`AppError::Database`, `errors.rs:2905`, rendu partagé, AC6 et T3), R12 (borne MariaDB, à M56), F-L1 (verrou d'arrondi de W8, l. 225), F-L2 (clause « avant sa première écriture » à la règle de placement, aux trois lieux), F-L3 (couvert par R1), F-L5 (exercice de W27 à l'étape (4)). **Écarts à la consigne** : le refus du sol de W24 est un **`409`** `BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS` (`errors.rs`, `StatusCode::CONFLICT`), non le « `412` » des rapports et de la consigne — écrit tel quel ; M54 n'est pas déclarée « non tuée » : la position neuve la rend tuable par un test séquentiel, et T-a49 couvre la part que l'ordre des refus ne voit pas ; `origin/main` a reçu la 28-3 (`c1667e8a`), qui décale `backup.rs` (l. 528, 557, 581, 633) et touche `bank_imports.rs` — numéros de `5aad8b35` gardés, décalage écrit, à relire au rebase (T0). **Recomptes** (par `grep` sur la fiche, après remédiation) : 18 AC ; T-a1-T-a49 (49 numéros, 5 retirés, **44 actifs**) + 14 T-b ; mutations M1-M56 + trois `-bis` = **59** numéros, dont **5 sans objet** (M10, M19, M35, M49, M50), **3 équivalentes** (M14-bis, M20-bis, M51), **51 à tuer** ; 30 sites W1-W30, 16 « Gardée » hors W30 (inchangé) ; T-a27 à **dix-huit** lignes (16 + 1 + 1) ; choix C-28-4-1 à C-28-4-24 (un neuf, C-28-4-24 ; C-28-4-21 annoté). Validation P5 : `28-4-validate-prompt-p5.md` (Sonnet, lentilles R et F, double critère obligatoire, axe sur les régressions de cette remédiation). |
