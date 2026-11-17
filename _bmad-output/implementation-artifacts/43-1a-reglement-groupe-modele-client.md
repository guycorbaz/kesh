# Story 43-1a : un règlement pour plusieurs factures — migration, lettrage par ligne, factures clients (#823)

Status: ready-for-dev

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), par le workflow
`bmad-create-story`, en découpant la 43-1 (`43-1-reglement-groupe-modele.md`, `split`) après sa validation P1
(`kesh-gate-logs/e43/validation-43-1-p1.md`, 0/3/7/8 : H1, H2, M1-M7, L1-L8 appliqués ici) et les décisions de Guy du
même jour. Code lu à `8b50fd195`. Release **0.19.0**. **Story de tête ; porte la seule migration de la release.**
**`kesh-db` touché : gate complet même en cours de boucle.** Décisions : fichier d'épique
`_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md`, **C-43-1 à C-43-20**, qui font foi.
**Jalons** : commit T3 mercredi 18 avant midi (la 43-1b et la 43-2 en partent) ; passage en `review` mercredi 18 à 18 h.
Validation : passe 2 à lancer sur cette fiche.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable qui reçoit un virement pour plusieurs factures clients (d'un ou de plusieurs clients), je veux que
Kesh enregistre **un** règlement qui solde chacune et la lettre, l'éventuelle petite différence sur le compte d'écart, et
qu'un dé-rapprochement le défasse en entier.

Cette story pose **la migration entière** (colonnes de la 43-1b et de la 43-5 comprises, C-43-4), le **lettrage par
ligne** (deux familles), le **règlement groupé client** et son **dé-rapprochement**. Fournisseurs : 43-1b ; API : 43-2 ;
écran : 43-3.

## Ce qui se lit au code (`8b50fd195`)

- `bank_transactions.matched_entry_id` (`20260504000001_bank_imports.sql:71`) ; `uq_invoice_settlements_entry`
  (`20260827000001_invoice_settlements.sql:65`) ; `supplier_invoices.settlement_journal_entry_id` sans unicité
  (`20260628000001_supplier_invoices.sql:47`).
- L'acceptation unitaire client vit **dans la route** (`accept_one_invoice`, `routes/reconciliation.rs:2375-3090`) : il n'y
  a pas de primitive de dépôt à réemployer (H3). Ses étapes, à reproduire **dans le même ordre** : rechargement de la
  transaction sous le verrou du compte, refus d'un débit (`refuse_non_credit_settlement`), facture, éligibilité, reste dû,
  compte de banque, compte de créance lu sur l'écriture de vente (`sale_receivable_account`), `refuse_if_claim_account`,
  trop-perçu, compte d'arrondi (`rounding_account_for_write`, `company_invoice_settings.rs:628`) et son refus de compte de
  créance, lignes (`settlement_journal_lines`, `invoice_settlements.rs:237`), écriture avec `project_id` d'en-tête
  (:2820), règlement, `UPDATE invoices … AND version = ?` (« le verrou qui interdit de régler deux fois », :2905-2965),
  `UPDATE bank_transactions … SET matched_entry_id`, lettrage, `reconciliation.accepted` puis `invoice.paid` /
  `invoice.partially_settled` avec `reconciliation_audit_id` (:3003-3075).
- Héritage du projet : `line.project_id.or(new.project_id)` (`journal_entries.rs:521`).
- Lettrage : `SYNC_SETTLEMENT_ENTRIES_SQL` (`letterings.rs:1477`), `SYNC_DOCUMENT_LINES_SQL` (:1497),
  `discover_invoice_document` (:1536), `discover_supplier_invoice_document` (:1901), `existing_document_group` (:1622).
- Dé-rapprochement : `classify` (`reconciliation_cancel.rs:158`, `fetch_optional` :166-175), `cancel_in_tx` (:581) —
  ordre : transaction, classement verrouillant, facture, écriture et exercice, `ensure_open_in_tx` (:631), motifs,
  `UPDATE … SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL` (:687), annulation appelée,
  `recheck_open_locked_in_tx` (:784), audit ; `entry_cancel_outlook` (:325), `get_view` (:435).
- `document_owners` (`journal_entries.rs:2253-2258`, doc-comment « seul `invoice_settlements.journal_entry_id` est
  `UNIQUE` » ; `MIN(id)` par écriture :2335-2338).
- `TABLES_TO_TRUNCATE` (`backup.rs:46`) : ordre enfant d'abord ; `invoice_settlements` :56, `journal_entry_lines` :57,
  `supplier_invoices` :61.
- P6 : `migrations_upgrade_path.rs:133-135` (`assert_eq!(total, 91)`) et `total - 57` (:228-231).
- Rattrapage `20261010000001` : **retiré** du rejeu (`RETIRED_BACKFILLS`, `post_restore.rs:213-216`).
- Manuel d'administration : `\subsubsection{Passer à la 0.18.0}` (`docs/manual/fr/admin-manual.tex:2540`).

## Décisions propres à la story

- **D1** — Module neuf `crates/kesh-db/src/repositories/grouped_settlements.rs`. Il **reproduit** les étapes de
  `accept_one_invoice` pour N factures, en appelant les fonctions de dépôt qu'elle appelle (aucune n'est recopiée) ; il ne
  modifie pas `accept_one_invoice`.
- **D2** — Lignes de l'écriture : banque d'abord, puis une ligne de créance par facture dans l'ordre des **identifiants
  croissants** (aussi l'ordre des verrous), puis l'écart. Attribution ligne → facture par cet ordre ; l'étape 0 vérifie que
  `create_in_tx` rend les lignes dans l'ordre fourni (ligne lue citée au Change Log), sinon appariement par position
  d'insertion (jamais par compte et montant : deux factures peuvent avoir le même reste).
- **D3** — Une écriture est **groupée** si au moins un règlement client ou fournisseur qui la pointe porte un identifiant
  de ligne.
- **D4** — Le dé-rapprochement groupé retire les propriétaires, **puis** contre-passe l'écriture devenue sans propriétaire
  (`journal_entries::reverse_in_tx`) : aucune autorité neuve.
- **D5** — **Audit (H1, C-43-9)** : le dépôt écrit `reconciliation.accepted` (avec l'acteur, `details` : `proposalType:
  "invoice_group"`, `journalEntryId`, `transactionAmount`, `invoices: [{ id, amount }]`, `difference`) **avant** les
  traces par facture, puis `invoice.paid` par facture avec les détails de l'unitaire (`paid_via: "reconciliation"`,
  `settlement_journal_entry_id`, `reconciliation_audit_id` = l'id de l'entrée précédente, avant/après). La route (43-2)
  n'écrit aucune trace. Aucune action neuve dans `audit_labels.rs`.

## Critères d'acceptation

**Migration (C-43-4, C-43-5)**

- **AC1** — `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql`, DDL seul, en instructions séparées :
  1. `invoice_settlements` : `ADD COLUMN journal_entry_line_id BIGINT NULL`, `fk_invoice_settlements_entry_line` →
     `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY uq_invoice_settlements_entry_line (journal_entry_line_id)`,
     `UNIQUE KEY uq_invoice_settlements_entry_invoice (journal_entry_id, invoice_id)` ;
  2. `DROP INDEX uq_invoice_settlements_entry` (la clé étrangère s'appuie sur la clé composée, colonne de tête — prouvé à
     T0 sur MariaDB 10.11) ;
  3. `supplier_invoices` : `ADD COLUMN settlement_journal_entry_line_id BIGINT NULL`, `fk_supplier_invoices_settlement_line`
     → `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY uq_supplier_invoices_settlement_line`, `CONSTRAINT
     chk_supplier_invoices_settlement_line CHECK (settlement_journal_entry_line_id IS NULL OR settlement_journal_entry_id
     IS NOT NULL)` ;
  4. `journal_entry_lines` : `ADD COLUMN bank_transaction_id BIGINT NULL`, `fk_jel_bank_transaction` →
     `bank_transactions(id) ON DELETE RESTRICT`, `INDEX idx_jel_bank_transaction (bank_transaction_id)` (43-5).
  En-tête : non breaking, avec les raisons de C-43-4 (refus de démarrer d'une 0.18.0, refus de toute sauvegarde 0.19.0,
  sauvegarde 0.18.0 importable ; un binaire 0.18.0 qui démarrerait classerait mal un groupe, d'où l'importance du refus de
  démarrer ; pas de relèvement de `kesh_version_min_required`) ; DDL non transactionnel en MariaDB : un échec au milieu
  laisse les premières instructions posées, rejeu à la main seulement (L4).
- **AC2** — `docs/migrations-idempotence-audit.md` : une ligne (`tracked-by-sqlx`, avec la réserve d'AC1), totaux à **92**
  aux deux sites, compteurs de partition recomptés depuis le tableau. P6 : `migrations_upgrade_path.rs` passe à
  `assert_eq!(total, 92)` (message mis à jour) et `total - 58` (la frontière reste à la 35ᵉ migration ; commentaires
  :137-156 et :228-231 ajustés) ; les autres sites résolvent par version (`pre_upgrade_backup/tests.rs`,
  `sauvegarde_avant_migration.rs`) — relevé au Change Log. Squash `test-schema` repris, `test_schema_guard.rs` vert ;
  `every_data_backfill_migration_is_triaged` vert sans entrée. `TABLES_TO_TRUNCATE` (`backup.rs:46`) : `supplier_invoices`
  placée avant `journal_entry_lines`, `journal_entry_lines` avant `bank_transactions` (ou le commentaire « FK-safe »
  amendé), `reset_partition_covers_the_canonical_list_in_order` relu (L3).

**Entités, lectures, export**

- **AC3** — `InvoiceSettlement.journal_entry_line_id`, `NewInvoiceSettlement.journal_entry_line_id` (et
  `invoice_settlements::create_in_tx`), `SupplierInvoice.settlement_journal_entry_line_id` ; toutes les listes de colonnes
  qui hydratent ces entités (`invoice_settlements.rs:18` `COLUMNS`, `supplier_invoices.rs:44`, `:56`, `list_all_by_company`
  `:3190-3210`, le littéral `kesh-reconciliation/src/matching.rs` près de :317 et :804) — inventaire par `git grep`, au
  Change Log. `JournalEntryLine` n'a pas besoin du champ dans cette story (lu par la 43-5). Export de souveraineté
  (`exports/csv_tables.rs` : `invoice_settlements` :1386, `supplier_invoices` :1232, `journal_entry_lines`) : colonnes
  ajoutées.

**Lettrage par ligne (deux familles)**

- **AC4** — `discover_invoice_document` et `discover_supplier_invoice_document` ne retiennent, d'une écriture de règlement
  dont le règlement porte un identifiant de ligne, **que cette ligne** ; sans identifiant, toutes les lignes du compte de
  tiers, comme avant. Verrous, index forcés et ordre inchangés. Les tests existants de `letterings` passent sans
  modification.

**Règlement groupé client**

- **AC5** — `grouped_settlements::settle_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction_id,
  invoice_ids, acteur, aujourd_hui) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError>`, dans la transaction et
  sous le verrou de compte de la route :
  - ordre des verrous de l'unitaire : transaction rechargée (`find_pending_by_id_for_account`), puis factures **par
    identifiant croissant** (`FOR UPDATE`), puis le compte bancaire ;
  - **refus** (`GroupRefusal`, rien d'écrit, une variante par code, `invoice_id: Option<i64>` quand une facture est en
    cause) : transaction introuvable ou non `pending` ; transaction qui n'est pas un crédit ; moins de 2 ou plus de 50
    factures, doublon ; facture introuvable ou d'une autre société ; non `validated` ; déjà payée ; reste dû nul ; sans
    écriture de vente ; écriture de vente sans ligne au débit ; compte bancaire non lié, clos, archivé ; compte de créance
    égal au compte de la banque ; projet non imputable (C-43-20) ; écart au-delà de `GROUP_DIFFERENCE_MAX` (C-43-16) ;
    écart sans compte d'arrondi (`ROUNDING_ACCOUNT_NOT_CONFIGURED`) ; compte d'arrondi égal à un compte de créance ou à la
    banque ; période verrouillée, exercice clos ou absent, comptes non imputables (traduits des `DbError` de
    `create_in_tx`). Tout autre `DbError` reste `Err` (erreur de transaction, 500 ou 409 rejoué) — dit au doc-comment. Un
    compte bancaire de type carte de crédit est accepté comme par l'acceptation unitaire (même angle mort que
    `invoice_settlements_write.rs` près de :190, à dire) ;
  - écriture datée comme l'unitaire (date de valeur, à défaut de comptabilisation), **sans** `project_id` d'en-tête :
    débit banque du montant ; par facture, crédit de **son** compte de créance de son **reste dû brut**, avec
    `project_id` = celui de la facture ; ligne d'écart `d = montant − Σ restes bruts` si `d ≠ 0` (C-43-16 : compte
    d'arrondi, sans projet, au crédit si `d > 0`) ;
  - une ligne `invoice_settlements` par facture (`amount` = reste brut, `settled_on`, `bank_transfer`,
    `settlement_bank_account_id`, `journal_entry_line_id`) ;
  - chaque facture : `UPDATE invoices SET paid_at = …, version = version + 1 … WHERE id = ? AND company_id = ? AND version
    = ? AND status = 'validated'`, la version étant celle **lue sous le verrou** de cette étape ; `rows_affected() != 1` →
    refus de la facture ;
  - la transaction : `SET matched_entry_id = ?, status = 'reconciled', version = version + 1 WHERE … AND status = 'pending'
    AND matched_entry_id IS NULL`, une ligne exactement, sinon `Invariant` ;
  - lettrage de chaque facture (`sync_invoice_in_tx`) ; audit de D5 ;
  - `recheck_open_locked_in_tx` du compte en dernière acquisition, avant l'audit (C-43-19) ;
  - `GroupedSettlement` rend `journal_entry_id`, l'id de l'audit `reconciliation.accepted`, et par facture `(invoice_id,
    settlement_id, line_id, amount)`, plus `difference`.
- **AC6** — Après AC5 : reste dû 0 et `paid_at` posé pour chaque facture ; une clé de lettrage **par** facture (sa ligne de
  vente et sa seule ligne de créance) ; ni la ligne de banque ni l'écart lettrés ; la ligne de créance de chaque facture
  porte son projet, banque et écart aucun.

**Dé-rapprochement groupé client (M1, L6)**

- **AC7** — `classify` reconnaît l'écriture groupée (D3) : lecture verrouillante de **toutes** ses lignes
  `invoice_settlements` (`fetch_all`, ordre `(journal_entry_id, invoice_id)`) et des factures fournisseur qui la pointent
  (préparé pour la 43-1b) ; variante `ReconciliationKind::GroupedSettlement` (code `"grouped_settlement"`). `cancel_in_tx`
  l'aiguille vers `grouped_settlements::cancel_in_tx`, **dans l'ordre de l'unitaire** : transaction, classement
  verrouillant, factures par identifiant croissant, écriture et exercice, `ensure_open_in_tx` ; puis :
  1. tous les refus, **avant toute écriture** : par facture (créditée, solde existant, exercice clos, lettrage figé par la
     période), et ceux de l'écriture (`settlement_entry_cancel_blocker`, motifs de propriété ignorés) ; le premier rendu ;
  2. `UPDATE bank_transactions SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL …` comme
     l'unitaire (audit `wasPreviouslyRejected` compris ; `unlinkOnly` toujours faux pour un groupe) ;
  3. délettrage de chaque pièce ;
  4. suppression des lignes `invoice_settlements` de l'écriture ;
  5. contre-passation (`reverse_in_tx`, datée du jour, acteur) ;
  6. chaque facture : `paid_at` projeté, `version + 1` (comme `cancel_settlement_in_tx`) ;
  7. `recheck_open_locked_in_tx` ; audit avec l'acteur, clé d'API comprise (`reconciliation.cancelled` et une
     `invoice.settlement_cancelled` par facture, comme l'unitaire).
  `entry_cancel_outlook` (motifs de la queue commune et de chaque facture) et `get_view` rendent la variante et la liste
  `(kind, id, numéro, montant)` des factures (structure lue par la 43-2).
- **AC8** — Gestes unitaires sur une facture du groupe : annulation de son règlement refusée (`MATCHED_BANK_TRANSACTION`) ;
  un avoir refusé (facture réglée) ; un nouveau règlement refusé (payée). Tests.

**Sites qui supposent « une écriture, un règlement » (M6, M7) — inventaire des sites non résolus**

- **AC9** — Résolus ici : `classify`, `entry_cancel_outlook`, `get_view` ; `document_owners` : doc-comment corrigé
  (`journal_entries.rs:2253-2258`), le motif nomme le plus petit règlement (inchangé) — test « une écriture groupée refuse
  la modification, la contre-passation directe et le délettrage manuel ». **Assumés, sûrs** (lecture au Change Log) :
  `ReversalAuthority::exempts` (refusé avant par `MatchedBankTransaction`), `reversal_blockers`,
  `reconciliation_link.rs` (`LINKABLE_SETTLEMENTS` :150, `other_side_predicate` :698-710, :811 — `EXISTS`, test
  `the_settlement_exclusion_matches_the_classification`), `letterings/open_items.rs:123`, `credit_notes.rs:383`,
  `kesh-report/src/vat_report.rs:339` (soldes `write_off` seulement), `imported_supplier_invoices.rs:425` (`LIMIT 1` : un
  seul document source montré pour une écriture groupée), `document_group_frozen_by_periods` (N groupes). Tout site
  nouveau trouvé par `git grep -n "invoice_settlements\|settlement_journal_entry_id"` s'ajoute à la liste, résolu ou
  assumé.

**Sauvegarde, test d'accord, montée**

- **AC10** — Aller-retour `.keshbackup` d'une base qui porte un groupe client : empreintes identiques, colonnes de ligne
  comprises ; un `.keshbackup` 0.18.0 (fixture sans les colonnes) s'importe, colonnes à `NULL`.
- **AC11** — Test d'accord `crates/kesh-db/tests/lettering_documents_backfill.rs` : fixture d'un groupe non lettré ; le
  SQL du rattrapage (retiré du rejeu) s'abstient, la synchronisation lettre — divergence écrite au doc-comment, sans
  prétendre à un rejeu (M3).
- **AC12** — Test de montée (`migrations_upgrade_path.rs` ou voisin, résolution **par version**) : base au schéma d'avant
  peuplée (règlements unitaires clients et fournisseurs), migration, données intactes, colonnes à `NULL`, puis un groupe se
  crée.

**Manuel d'administration (L8)**

- **AC13** — `docs/manual/fr/admin-manual.tex` (`.tex` seulement) : `\subsubsection{Passer à la 0.19.0}` sur le patron de
  « Passer à la 0.18.0 » (:2540) et des 0.17.5/0.17.6 : migration ; sauvegarde écrite par Kesh
  (`…-0.18.0-to-0.19.0.keshbackup` dans `backup/pre-upgrade/`), `KESH_SKIP_PRE_UPGRADE_BACKUP=0.19.0` ; la 0.18.0 refuse
  de démarrer (« was previously applied but is missing ») et refuse toute sauvegarde 0.19.0 (`IMPORT_SCHEMA_MISMATCH`) ;
  une sauvegarde 0.18.0 s'importe dans la 0.19.0 ; retour par la sauvegarde d'avant la mise à jour seulement, « tout ce
  qui a été saisi depuis est perdu », renvoi à la procédure de retour existante.

## Tests à écrire, et la preuve du rouge

`crates/kesh-db/tests/grouped_settlements.rs` (neuf, squash, `#[sqlx::test]`) ; rouges d'abord
(`kesh-gate-logs/e431a/rouges.log`) :

- groupes de 2 et 3 factures de **deux clients** ; restes à quatre décimales ; facture déjà réglée en partie (son reste) ;
  écarts `+0.30`, `−4.99`, `5.00` acceptés, `5.01` refusé ; écart sans compte d'arrondi refusé ; deux factures à projets
  différents (AC6) ;
- un test par variante de `GroupRefusal` ; après chaque refus, tables inchangées (comptes avant/après) ;
- concurrence : règlement unitaire (fiche facture) concurrent d'une facture du groupe ; deux groupes qui partagent une
  facture — un seul aboutit, l'autre refuse (`version`) ou est rejoué, aucun interblocage non rejoué ;
- dé-rapprochement : complet, refus sur exercice clos (rien d'écrit), acteur clé d'API dans chaque entrée ; entrelacement
  « avoir / annulation d'une facture du groupe » et dé-rapprochement ;
- AC8, AC9 (`document_owners`), AC10 à AC12 ;
- **mutations** (`kesh-gate-logs/e431a/mut/`) : retirer le filtre par ligne d'AC4 (AC6 rougit) ; inverser l'attribution
  (AC6 rougit sur deux restes différents) ; retirer `AND version = ?` (le test de concurrence rougit) ; omettre la
  suppression des règlements à l'étape 4 d'AC7 (reste dû faux) ; poser le projet à l'en-tête (AC6 rougit sur la banque).

## Tâches

- [ ] **T0** — Étape 0 : LOW de la validation P2 ; D2 (ordre des lignes de `create_in_tx`) ; appui de la clé étrangère après
  `DROP INDEX` et `CHECK` sur colonne neuve, sur MariaDB 10.11 ; lignes lues citées au Change Log.
- [ ] **T1** — Migration, audit d'idempotence, P6, squash, `TABLES_TO_TRUNCATE` (AC1, AC2). Commit.
- [ ] **T2** — Entités, listes de colonnes, export (AC3). Commit.
- [ ] **T3** — **Signatures publiques** de `grouped_settlements.rs` documentées : `GroupedSettlement`, `GroupRefusal`
  (toutes ses variantes, celles de la 43-1b comprises), `GroupedInvoiceRef`, `GROUP_DIFFERENCE_MAX`, `settle_invoices_in_tx`,
  `pay_supplier_invoices_in_tx` (corps : `unimplemented!` permis à ce commit seulement), `cancel_in_tx`. **Commit
  transmis à l'orchestrateur mercredi avant midi** : la 43-1b et la 43-2 en partent. Aucun `unimplemented!` ne reste au
  passage en `review`, sauf celui de `pay_supplier_invoices_in_tx`, propriété de la 43-1b (qui le remplace).
- [ ] **T4** — Lettrage par ligne (AC4).
- [ ] **T5** — Règlement groupé client (AC5, AC6).
- [ ] **T6** — Dé-rapprochement groupé client, gestes unitaires, inventaire (AC7 à AC9).
- [ ] **T7** — Sauvegarde, test d'accord, montée (AC10 à AC12).
- [ ] **T8** — Manuel d'administration (AC13).
- [ ] **T9** — Gates (`kesh-db` : complet en boucle) ; passage en `review` **mercredi 18 h** ; Dev Agent Record ; prompt de
  revue P1.

## Note de version

- **Avant la mise à jour** : faites une sauvegarde complète (base, `.keshbackup`, dossier des documents). La 0.19.0 porte
  une migration ; le retour à la 0.18.0 ne se fait que par cette sauvegarde ; une sauvegarde faite en 0.19.0 ne s'importe
  pas dans une 0.18.0. Kesh écrit aussi la sienne dans `backup/pre-upgrade/` avant de migrer.
- **Ajouté** — Un virement peut régler plusieurs factures clients, d'un ou de plusieurs clients : un seul règlement,
  chaque facture soldée et lettrée, une petite différence (5.00 au plus) passée au compte de différences d'arrondi ; le
  dé-rapprochement le défait en entier (refs #823).

## Dev Notes

### Modules touchés

`kesh-db/migrations` (+ squash, audit, `migrations_upgrade_path.rs`, `backup.rs`), `entities`,
`repositories/grouped_settlements` (neuf), `repositories/letterings`, `repositories/reconciliation_cancel` (+ listes de
colonnes de `invoice_settlements` et `supplier_invoices`, doc-comment de `journal_entries`), `kesh-api/exports/csv_tables`.
Cinq modules métier : au plafond de la règle de splitting, et tenue ainsi parce que la migration, la création et
l'annulation se testent ensemble (aucun commit fusionné ne doit produire un groupe qu'on ne sait pas défaire).

### Limites assumées

- Pas de règlement partiel dans un groupe ; l'écart est borné à 5.00 (Q4 à Guy).
- La ligne de banque et l'écart d'un groupe n'ont pas de compte analytique, à la différence d'un règlement unitaire dont
  toutes les lignes héritent du projet de la facture.
- Le motif `OwnedBySettlement` d'une écriture groupée nomme une seule facture (la plus petite) ; le gel est juste, le
  message incomplet — issue à ouvrir si la 43-3 ne le corrige pas à l'écran.

### Hors périmètre

Fournisseurs (43-1b), paiements existants (43-5, mais sa colonne est posée ici), API (43-2), écran (43-3), raccord Epic 42
(43-6).

### Ce qui doit rester vrai

Un règlement unitaire se crée, se lettre, s'annule et se dé-rapproche exactement comme en 0.18.0 : aucun test existant ne
change pour passer.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche née du découpage de la 43-1 (validation P1, H3), findings H1, H2, M1-M7, L1-L8 appliqués ; décisions
  de Guy du 2026-11-17 (tiers mixtes, écart, ordre de sortie).
