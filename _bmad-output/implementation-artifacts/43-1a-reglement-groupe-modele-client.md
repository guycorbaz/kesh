# Story 43-1a : un règlement pour plusieurs factures — migration, lettrage par ligne, factures clients (#823)

Status: done

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), par le workflow
`bmad-create-story`, en découpant la 43-1 (`43-1-reglement-groupe-modele.md`, `split`) après sa validation P1 ; **remédiée
dans la nuit** après sa validation P2 (`kesh-gate-logs/e43/validation-43-1a-p2.md`, 0/3/4/8 : H-A, H-B, H-C, M-A à M-D,
L1-L8 appliqués selon les décisions de l'orchestrateur), les validations P1 de la 43-2 et de la 43-3 (exigences sur la
43-1a) et la décision de Guy sur Q4 (plafond configurable). Code lu à `8b50fd195`. Release **0.19.0**. **Story de tête ;
porte la migration de la release.** **`kesh-db` touché : gate complet même en cours de boucle.** Décisions : fichier
d'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md`, **C-43-1 à C-43-26**, qui font foi ;
la règle de la différence est **tranchée** (décision de Guy du 2026-11-17, décomposition définitive de l'orchestrateur :
`kesh-gate-logs/e43/decision-difference.md` ; registre de la story `e0190-choix-autonomes-e431a.md`, C-e431a-1, -2, -12,
-14) — D4 bis ci-dessous. **Jalons** : commit T3
mercredi 18 avant midi (43-1b et 43-2 en partent ; la 43-7 part de T1) ; passage en `review` mercredi 18 à 18 h sur le
périmètre du repli (Dev Notes). Validation : passe 3 (0/0/6/10) remédiée ; passe 4 ciblée à lancer.*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable qui reçoit un virement pour plusieurs factures clients (d'un ou de plusieurs clients), je veux que
Kesh enregistre **un** règlement qui solde chacune et la lettre, l'éventuelle petite différence passée au bon compte, et
qu'un dé-rapprochement le défasse en entier.

Cette story pose **la migration de la release** (colonnes de la 43-1b et réglage de la 43-7 compris ; la colonne de la
43-5 a sa propre migration), le **lettrage par ligne** (deux familles), le **règlement groupé client** et son
**dé-rapprochement**. Fournisseurs : 43-1b ; API : 43-2 ; écran : 43-3 ; réglage : 43-7.

## Ce qui se lit au code (`8b50fd195`)

- `bank_transactions.matched_entry_id` (`20260504000001_bank_imports.sql:71`) ; `uq_invoice_settlements_entry`
  (`20260827000001_invoice_settlements.sql:65`) ; `supplier_invoices.settlement_journal_entry_id` sans unicité
  (`20260628000001_supplier_invoices.sql:47`) ; `company_invoice_settings` porte `default_rounding_account_id` et
  `default_bank_fees_account_id` (`company_invoice_settings.rs:628-670`).
- L'acceptation unitaire client vit **dans la route** (`accept_one_invoice`, `routes/reconciliation.rs:2375-3090`), sans
  primitive de dépôt. Ses étapes, à reproduire dans le même ordre : rechargement **non verrouillant** de la transaction
  (`find_pending_by_id_for_account`, `repositories/reconciliation.rs:1219-1229`) ; refus d'un débit ; facture lue **sans
  verrou** (`find_invoice_by_id_for_company`) ; éligibilité ; fenêtre de dates (`InvoiceDateWindow::check`,
  `repositories/reconciliation.rs:565`, et `date_refusal_failed_proposal`, `routes/reconciliation.rs:2316`) ; reste dû sur
  l'instantané (`amount_due`) ; compte de banque ; compte de créance lu sur l'écriture de vente ; `refuse_if_claim_account` ;
  trop-perçu (`classify_payment`, `invoice_settlements.rs:179`, « Trop-perçu : refusé », :157) ; compte d'arrondi ; lignes ;
  écriture ; règlement ; **`UPDATE invoices … AND version = ?`** — « le verrou qui interdit de régler deux fois », et
  « ⛔ **Pas de `FOR UPDATE` sur la facture en amont** : il lirait `version` à jour … Il désarmerait le verrou »
  (:2932-2950) ; `UPDATE bank_transactions … AND version = ?` (:2905, refus `race_during_update`, :2918-2925) ; lettrage ;
  `reconciliation.accepted` puis `invoice.paid` (:3003-3075, clés snake_case, `batch_size` :1706-1710).
- Projets : `create_in_tx_inner` (`journal_entries.rs:286`) ne valide que les projets **par ligne**
  (`enforce_project_taggable`, :309-316) ; la contre-passation recopie sans valider (:271-282) ; test
  `pay_succeeds_when_project_archived_after_tagging` (`tests/supplier_invoices_repository.rs:1189-1191`).
- Lettrage : `letterings.rs:1477` `SYNC_SETTLEMENT_ENTRIES_SQL`, :1497 `SYNC_DOCUMENT_LINES_SQL`, :1536
  `discover_invoice_document`, :1901 `discover_supplier_invoice_document`, :1622 `existing_document_group`.
- Dé-rapprochement : `classify` (`reconciliation_cancel.rs:158`, `fetch_optional` :166-175), `cancel_in_tx` (:581),
  `ensure_open_in_tx` (:631), `expect_unlink_only` (:651-662, `RECONCILIATION_CANCEL_CHANGED`), `UPDATE … SET
  matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL` (:687), `recheck_open_locked_in_tx` (:784),
  `entry_cancel_outlook` (:325), `get_view` (:435) ; `ReconciliationKind` est `Copy` (:78-104).
- **Garde-fou de la modification d'écriture** : `crates/kesh-db/tests/journal_entries_modification.rs:144-195` — (1 bis)
  aucune clé vers une **ligne** (le `PUT` supprime et réinsère les lignes, `journal_entries.rs:1649-1655`, :1672) ; liste
  exacte des colonnes de `journal_entry_lines` (:153-171) ; toute colonne nommée `…journal_entry…` doit être une clé
  inventoriée vers `journal_entries` ou déclarée (:173-195).
- `document_owners` (`journal_entries.rs:2253-2258`, doc-comment « seul `invoice_settlements.journal_entry_id` est
  `UNIQUE` » ; `MIN(id)` :2335-2338).
- `TABLES_TO_TRUNCATE` (`backup.rs:46`) : `invoice_settlements` :56, `journal_entry_lines` :57, `supplier_invoices` :61 (avec
  `imported_supplier_invoices` juste avant).
- P6 : `migrations_upgrade_path.rs:133-135` (`assert_eq!(total, 91)`) et `total - 57` (:228-231).
- Rattrapage `20261010000001` **retiré** du rejeu (`RETIRED_BACKFILLS`, `post_restore.rs:213-216`).
- Manuel d'administration : `\subsubsection{Passer à la 0.18.0}` (`docs/manual/fr/admin-manual.tex:2540`).

## Décisions propres à la story

- **D1** — Module neuf `crates/kesh-db/src/repositories/grouped_settlements.rs`. Il **reproduit** pour N factures les
  étapes de `accept_one_invoice`, en appelant les fonctions de dépôt qu'elle appelle (aucune n'est recopiée) ; il ne modifie
  pas `accept_one_invoice`.
- **D2** — Lignes de l'écriture : banque, puis une ligne de créance par facture par **identifiant croissant**, puis arrondi,
  puis frais. Attribution ligne → facture par cet ordre ; l'étape 0 vérifie que `create_in_tx` rend les lignes dans l'ordre
  fourni (ligne lue citée), sinon appariement par position d'insertion (jamais par compte et montant).
- **D3** — Une écriture est **groupée** si au moins un règlement client ou fournisseur qui la pointe porte un identifiant
  de ligne.
- **D4** — Le dé-rapprochement groupé retire les propriétaires, **puis** contre-passe l'écriture devenue sans propriétaire
  (`journal_entries::reverse_in_tx`) : aucune autorité neuve.
- **D4 bis — La différence : une seule fonction pure** (M4 ; règle de Guy du 2026-11-17, décomposition définitive de
  `kesh-gate-logs/e43/decision-difference.md`, § « Décomposition de l'écart », dont le tableau des six cas fait foi ;
  registre C-e431a-14). `grouped_settlements::group_difference(montant, apports_bruts, plafond) -> Result<Decomposition,
  GroupRefusal>`, **pure**, appelée par les deux bras (client et fournisseur), la même pour les deux familles :
  - `d = |montant| − Σ apports bruts` (quatre décimales ; apport = reste dû brut côté client, `total_amount` côté
    fournisseur) ; `e = centime(d)`, demi-centime arrondi en s'éloignant de zéro (`amount_due_to_centime`) ;
    `r = d − e` (toujours `|r| ≤ 0.005`) ;
  - `Decomposition { difference: d, rounding_difference: r, gap: e }` — rien d'autre pour l'écart ; `d = r + e` ;
  - `r ≠ 0` → ligne au **compte d'arrondi** (`rounding_account_for_write`, contrôlé comme dans l'unitaire), jamais
    soumise au plafond ;
  - `e ≠ 0` (moins-payé ou trop-perçu, **les deux sens admis**) → ligne au **compte que l'utilisateur choisit**
    (`difference_account_id`, obligatoire) ; seul `|e| >` plafond est refusé (`DifferenceTooLarge`) ;
  - l'**arrondi automatique** est le cas `e = 0` (`|d| < 0.005`) : aucun compte choisi.

  | Cas | d | e | r |
  |---|---|---|---|
  | 2 × 10.0050 payés 20.01 | 0 | 0 | 0 |
  | 2 × 10.0050 payés 20.02 | +0.0100 | +0.01 | 0 |
  | 500.00 + 300.00 payés 799.99 | −0.0100 | −0.01 | 0 |
  | 3 × 10.0050 payés 30.06 | +0.0450 | +0.05 | −0.0050 |
  | 10.0049 + 10.0000 payés 20.00 | −0.0049 | 0 | −0.0049 |
  | 10.0050 + 10.0000 payés 20.00 | −0.0050 | −0.01 | +0.0050 |
- **D5 — Audit** (C-43-9) : le dépôt écrit `reconciliation.accepted` **avant** les traces par facture, avec l'acteur et les
  clés **snake_case** de l'unitaire : `bank_transaction_id`, `journal_entry_id`, `paid_amount`, `batch_size`, plus
  `proposal_type: "invoice_group"`, `difference` (= `d`), `rounding_difference` (= `r`), `gap` (= `e`),
  `difference_account_id` (le compte choisi, nul sans écart) et `invoices:
  [{ invoice_id, settled_amount }]` ; puis `invoice.paid` par facture avec les détails de l'unitaire (`paid_via:
  "reconciliation"`, `settlement_journal_entry_id`, `reconciliation_audit_id`, avant/après). La route n'écrit aucune trace.
  Au dé-rapprochement, `reconciliation.cancelled` porte `kind: "grouped_settlement"`, `invoice_id` et `settlement_id` nuls,
  et `invoices: [{ invoice_id, settlement_id, number }]` (L4).
- **D6 — Point de sauvegarde propre** (M-C) : `settle_invoices_in_tx` ouvre `SAVEPOINT grouped_settlement` (nom constant)
  en tête et, avant de rendre **tout** `Ok(Err(refus))`, fait `ROLLBACK TO SAVEPOINT grouped_settlement` : un refus ne
  laisse rien dans la transaction de l'appelant, que celui-ci valide ou non. Il s'emboîte dans celui de `accept_batch`
  (doc-comment). Un `Err(DbError)` laisse l'appelant revenir à **son** point de sauvegarde. Le doc-comment dit aussi (L3) :
  un `Err(DbError::Sqlx)` (1213, 1305) n'est **jamais** traduit en refus — il remonte, et la route rejoue (`accept_batch`,
  `is_savepoint_lost`, `routes/reconciliation.rs:1990-2046`) ; un `ROLLBACK TO SAVEPOINT grouped_settlement` qui échoue
  (1305, interblocage absorbé plus bas) remonte en `Err`, jamais suivi du refus ; les erreurs du lettrage restent des `Err`,
  comme dans l'unitaire (:2191-2218). Un `RELEASE` au succès est facultatif.
- **D7 — Projet estampillé sans validation** (H-B, C-43-20, M3, L2) : `create_in_tx` gagne une forme **`pub(crate)`**
  `create_copying_projects_in_tx(tx, fiscal_year_id, acteur, new, enforce_postable)` qui passe `enforce_project_taggable =
  false` ; seul appelant : `grouped_settlements` (même crate ; le test H-B passe par `settle_invoices_in_tx`). Doc-comment :
  « les projets viennent d'une pièce de la société, jamais d'une saisie ». Le groupe l'appelle avec **`enforce_postable =
  false`**, comme l'unitaire (`create_in_tx(…, false)`, :2820-2840) : les comptes sont ceux des réglages et de l'écriture de
  vente, tous de la société. Le test `pay_succeeds_when_project_archived_after_tagging` reste vert.

## Critères d'acceptation

**Migration (C-43-4, C-43-5, C-43-24)**

- **AC1** — `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql`, DDL seul, en instructions séparées :
  1. `invoice_settlements` : `ADD COLUMN journal_entry_line_id BIGINT NULL`, `fk_invoice_settlements_entry_line` →
     `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY uq_invoice_settlements_entry_line (journal_entry_line_id)`,
     `UNIQUE KEY uq_invoice_settlements_entry_invoice (journal_entry_id, invoice_id)` ;
  2. `DROP INDEX uq_invoice_settlements_entry` (la clé étrangère s'appuie sur la clé composée, colonne de tête — prouvé à
     T0 sur MariaDB 10.11) ;
  3. `supplier_invoices` : `ADD COLUMN settlement_journal_entry_line_id BIGINT NULL`, `fk_supplier_invoices_settlement_line`
     → `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY uq_supplier_invoices_settlement_line`, `CONSTRAINT
     chk_supplier_invoices_settlement_line CHECK (settlement_journal_entry_line_id IS NULL OR settlement_journal_entry_id
     IS NOT NULL)` ; **les deux sites qui remettent `settlement_journal_entry_id` à `NULL`** —
     `supplier_invoices.rs:2639` (`cancel_in_tx`, détachement) et `:2929` (`cancel_settlement_in_tx`) — remettent aussi la
     ligne à `NULL`, par **un seul fragment SQL partagé** (constante `CLEAR_SETTLEMENT_SQL`, `… settlement_journal_entry_id
     = NULL, settlement_journal_entry_line_id = NULL …`) que les deux `UPDATE` emploient (M2) ; test de la `CHECK` : poser
     la ligne sans l'écriture est refusé par la base ;
  4. `company_invoice_settings` : `ADD COLUMN group_settlement_max_difference DECIMAL(19,4) NOT NULL DEFAULT 5.0000`, et
     `CONSTRAINT chk_company_invoice_settings_group_max CHECK (group_settlement_max_difference >= 0 AND
     group_settlement_max_difference <= 100)`.
  **Aucune colonne sur `journal_entry_lines`** (C-43-17 : migration propre à la 43-5). En-tête : non breaking au sens du
  paragraphe ⚠️ de P1 (citer : « sans relever `kesh_version_min_required` et sans corrompre le schéma ») ; `DROP INDEX`
  hors de la liste de P3 ; la 0.18.0 refuse de démarrer et refuse toute sauvegarde 0.19.0, si bien qu'aucun chemin ne lui
  fait lire un groupe (son `classify` supposerait l'unicité retirée) ; pas de relèvement de `kesh_version_min_required`
  (L6) ; la colonne à `DEFAULT` est remise à 5.0000 à l'import d'une sauvegarde 0.18.0, et c'est voulu (P7, C-43-5) ; DDL
  non transactionnel en MariaDB : un échec au milieu laisse les premières instructions posées et la migration non inscrite,
  et le rejeu échoue en 1060 (reprise manuelle : AC13). L'ordre « clé composée avant `DROP INDEX` » est celui de la 30-2a
  (erreur 1553 sinon, `docs/migrations-idempotence-audit.md:104`) ; les `CHECK` sur colonne neuve ont leur précédent
  (`20261031000001`) (L8).
- **AC2** — Registres et garde-fous :
  - `docs/migrations-idempotence-audit.md` : une ligne (`tracked-by-sqlx`, réserve d'AC1), totaux à **92** aux deux sites
    (en-tête :17, `Total` :117, dont la ventilation gagne « + 1 Story 43-1a »), compteurs de partition recomptés depuis le
    tableau ; P6 : `migrations_upgrade_path.rs` à `assert_eq!(total, 92)` et `total - 58` (frontière inchangée) — le
    `grep -nE "\b(57|91)\b"` du fichier fait foi, relancé après la correction (L6), **les lignes d'historique exclues**
    (« Story … : a → b », « + … = 91 », :97, :131, :159-160, :221 : relevés datés, non réécrits ; une ligne
    « Story 43-1a : 57 → 58 » et « + grouped_settlements … = 92 » s'y ajoutent) (P4 LOW-3) ; si la 43-5 est publiée, **elle** passe à 93 et
    `total - 59` (C-43-5) ; les autres sites résolvent par version (`pre_upgrade_backup/tests.rs`,
    `sauvegarde_avant_migration.rs`), relevé au Change Log ;
  - squash `test-schema` repris, `test_schema_guard.rs` vert ; `every_data_backfill_migration_is_triaged` vert sans entrée ;
  - **`journal_entries_modification.rs` (H-C)** : les deux clés neuves vers une **ligne** contredisent l'assertion (1 bis)
    (:144-151), et les deux colonnes `…journal_entry_line_id` l'inventaire de :173-195. L'invariant est levé **par décision
    écrite** dans le test et au Change Log : « une écriture dont une ligne est référencée par un règlement est **possédée**
    (`document_owners` : `OwnedBySettlement`, `OwnedBySupplierInvoice`, `MatchedBankTransaction`), donc jamais réécrite par
    le `PUT` (`modification_guard`) ». Les assertions deviennent : clés vers une ligne **inventoriées** — `cles_vers` rend des
    couples `(table, colonne)` (:119-130) : exactement `(invoice_settlements, journal_entry_line_id)` et `(supplier_invoices,
    settlement_journal_entry_line_id)` ; ces deux colonnes déclarées au point (3) (:173-195, motif `%journal_entry%`) (L7).
    La levée tient parce que le `PUT` s'arrête à `modification_guard` (`journal_entries.rs:1574`), la suppression à
    `modification_guard` sous `enforce_ownership` (:1930 ; les deux appelants à `false`, `supplier_invoices.rs:1421-1423` et
    `invoices.rs:1708`, visent un achat ou une vente qu'aucun règlement ne possède), la contre-passation directe à
    `reversal_blockers`. La liste des colonnes de `journal_entry_lines` (:153-171) ne change pas (43-5 seulement). Test « `PUT` d'une
    écriture groupée refusé, aucune ligne réécrite, clés vers les lignes intactes » ;
  - `TABLES_TO_TRUNCATE` (`backup.rs:46`, L5) : `supplier_invoices` **et** `imported_supplier_invoices` déplacées ensemble
    avant `journal_entry_lines`, ou le commentaire « FK-safe » amendé (sans effet à l'exécution, `FOREIGN_KEY_CHECKS = 0`) ;
    `reset_partition_covers_the_canonical_list_in_order` relu.

**Entités, lectures, export**

- **AC3** — `InvoiceSettlement.journal_entry_line_id`, `NewInvoiceSettlement.journal_entry_line_id` (et
  `invoice_settlements::create_in_tx`), `SupplierInvoice.settlement_journal_entry_line_id`, le champ
  `group_settlement_max_difference` de l'entité des réglages de facturation ; toutes les listes de colonnes qui les
  hydratent (`invoice_settlements.rs:18` `COLUMNS`, `supplier_invoices.rs:44`, `:56`, `list_all_by_company` :3190-3210, le
  littéral `kesh-reconciliation/src/matching.rs` près de :317 et :804, les lectures de `company_invoice_settings`) —
  inventaire par `git grep`, au Change Log. Export de souveraineté (`exports/csv_tables.rs` : `invoice_settlements` :1386,
  `supplier_invoices` :1232, réglages de facturation) : colonnes ajoutées. **`journal_entry_lines` n'est pas touchée** (L1).
- **AC3 bis** — Lecture du plafond : `company_invoice_settings::group_settlement_max_difference(conn, company_id)` rend la
  colonne, ou `DEFAULT_GROUP_DIFFERENCE_MAX` (5.0000, **seule** occurrence de la valeur dans le code Rust) si la société
  n'a pas de ligne de réglages. Un test compare la constante au `DEFAULT` de la colonne
  (`information_schema.COLUMNS.COLUMN_DEFAULT`) : les deux sources ne dérivent pas (L9). **Livré à T1** avec le champ
  d'entité (M5), pour que la 43-7 parte de T1.

**Lettrage par ligne (deux familles)**

- **AC4** — `discover_invoice_document` et `discover_supplier_invoice_document` ne retiennent, d'une écriture de règlement
  dont le règlement porte un identifiant de ligne, **que cette ligne** ; sans identifiant, toutes les lignes du compte de
  tiers, comme avant. Verrous, index forcés et ordre inchangés. Les tests existants de `letterings` passent sans
  modification.

**Règlement groupé client**

- **AC5** — `grouped_settlements::settle_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction_id, invoice_ids,
  difference_account_id, batch_size, acteur, aujourd_hui) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError>`, dans la transaction et
  sous le verrou de compte de la route, **par la méthode de l'unitaire, à deux écarts près** (C-43-19, H-A, L1) : lecture
  sans verrou, version et reste dû de l'instantané, `UPDATE … AND version = <lue>` par identifiant croissant,
  `rows_affected()` contrôlé, `paid_at` posé seulement sur un reste dû tombé à zéro — comme l'unitaire. **Écarts** : (1)
  l'ordre des deux `UPDATE` — factures (étape 9) **avant** la transaction (étape 10), là où l'unitaire met à jour la
  transaction d'abord (`routes/reconciliation.rs:2905-2925` puis :2950-2965), pour qu'un refus d'une facture tombe avant
  toute écriture sur la transaction ; sans effet sur le résultat (même transaction, verrou de compte, point de sauvegarde
  D6) ; (2) ni score ni fenêtre haute de 30 jours (C-43-7) :
  1. `SAVEPOINT grouped_settlement` (D6) ;
  2. **avant toute lecture** : bornes (moins de 2, plus de 50), doublon → refus ;
  3. transaction rechargée (`find_pending_by_id_for_account`, non verrouillante) : introuvable (`TransactionNotFound`),
     non `pending` (`TransactionNotPending`), puis **le sens et le montant nul** (`WrongDirection { amount }`), puis **la
     devise** non CHF (`CurrencyMismatch { currency }`, `transaction_can_be_linked_to_entry`, C-43-22) → refus ;
  4. chaque facture, par identifiant croissant, lue **sans verrou** sur l'instantané (`find_invoice_by_id_for_company`),
     avec sa **version** et son **reste dû** (`amount_due`) : introuvable ou d'une autre société, non `validated`, déjà
     payée, reste dû nul, sans écriture de vente, écriture de vente sans ligne au débit → refus ; **date** :
     `InvoiceDateWindow::for_transaction(&tx).check(invoice_date, invoice_date)` — l'ancre réelle n'est **pas** lue
     (`check` rend `PaymentBeforeInvoice` avant `PaymentTooLate`, `reconciliation.rs:565-575`, et la borne haute est
     ignorée) ; seul `PaymentBeforeInvoice` est un refus (`PaymentBeforeInvoiceDate`, M-B, L5) ;
  5. compte bancaire (introuvable ou archivé `BankAccountNotFound`, non lié `BankAccountNotConfigured`, clos
     `BankAccountClosed`), `ensure_open_in_tx` ; compte de créance de chaque facture (`sale_receivable_account`) ; compte
     de créance égal à la banque → refus `ClaimAccount { role: Counterparty, bank_account_id: Some(..), piece }` ;
  6. **la différence (D4 bis, `group_difference`)** : `|e| >` plafond (AC3 bis) → `DifferenceTooLarge` ; puis, **dans cet
     ordre**, `r ≠ 0` → compte d'arrondi (`rounding_account_for_write` ; absent → `RoundingAccountNotConfigured` ; égal à
     un compte de créance → `ClaimAccount { role: Rounding }`) — contrôlé **avant** le compte d'écart ; `e ≠ 0` → compte
     choisi `difference_account_id` : absent → `DifferenceAccountRequired { gap }`, puis la convention de la contrepartie
     choisie : compte de banque → `DifferenceAccountIsBankLedger`, inexistant, archivé ou d'une autre société →
     `DifferenceAccountInvalid` (route : `ACCOUNT_NOT_FOUND`), non imputable → `DifferenceAccountNotPostable
     { account_id, account_number }`, compte de créance d'une facture → `ClaimAccount { role: Counterparty,
     bank_account_id: None }` ;
  7. écriture datée comme l'unitaire, **sans** projet d'en-tête, par la forme de D7 : débit banque du montant ; par
     facture, crédit de **son** compte de créance de son **reste brut**, `project_id` = celui de la facture (estampillé sans
     validation, H-B) ; arrondi : `r` au crédit si `r > 0`, au débit sinon ; écart : `e` au crédit du compte choisi si
     `e > 0` (trop-perçu), au débit sinon (moins-payé) ; ni l'un ni l'autre n'a de projet ; `enforce_postable = false` (D7, L2 : aucun refus « compte non imputable ») ; les refus de
     `create_in_tx` (période verrouillée, exercice clos ou absent) → refus ;
  8. une ligne `invoice_settlements` par facture (`amount` = reste brut, `settled_on`, `bank_transfer`,
     `settlement_bank_account_id`, `journal_entry_line_id`) ;
  9. chaque facture, par identifiant croissant : relire son reste dû après l'insertion des règlements (`amount_due`) ;
     `> 0` → `DbError::Invariant` (défense contre un défaut du calcul des restes, L4 ; l'unitaire ne pose `paid_at` que si
     `fully_settled`) ; puis `UPDATE invoices SET paid_at = …, version = version + 1 … WHERE id = ? AND company_id = ? AND
     version = <version lue à l'étape 4> AND status = 'validated'` ; `rows_affected() != 1` → refus `race_during_update`
     (la facture est nommée) ;
  10. la transaction : `UPDATE bank_transactions SET matched_entry_id = ?, status = 'reconciled', version = version + 1
      WHERE … AND status = 'pending' AND matched_entry_id IS NULL AND version = ?` ; `rows_affected() != 1` → refus
      `TransactionNotPending` (L2) ;
  11. lettrage de chaque facture (`sync_invoice_in_tx`) ; `recheck_open_locked_in_tx` ; audit de D5.
  **`GroupRefusal`** — la liste finale, celle de `kesh-gate-logs/e43/t3.md` : `TooFew`, `TooMany`, `Duplicate`,
  `TransactionNotFound`, `TransactionNotPending`, `WrongDirection { amount }`, `CurrencyMismatch { currency }`,
  `InvoiceNotFound`, `InvoiceNotEligible { piece, reason }` (`IneligibleReason` : `NotValidated`, `AlreadyPaid`,
  `JournalEntryNotSet`, `NothingDue`, `SupplierNotOpen`, `SupplierInPaymentBatch`, `RaceDuringUpdate`),
  `HeldByPaymentBatch` (43-1b), `SaleEntryMalformed`, `BankAccountNotConfigured`, `BankAccountNotFound`,
  `BankAccountClosed`, `ClaimAccount { claim, role: SettlementAccountRole, account_id, account_number, bank_account_id,
  piece }`, `PaymentBeforeInvoiceDate`, `DifferenceTooLarge { transaction_amount, sum_applied, gap, max_difference }`,
  `DifferenceAccountRequired { gap }`, `DifferenceAccountIsBankLedger`, `DifferenceAccountInvalid`,
  `DifferenceAccountNotPostable { account_id, account_number }`, `RoundingAccountNotConfigured`, `PeriodLocked`,
  `LaterFiscalYearClosed`, `FiscalYearInvalid`. Une pièce en cause est un `GroupedPiece { document: GroupedDocument,
  number }` (numéro affichable) ; aucune variante de projet (H-B). Plus de `Overpayment`, `Underpayment` ni
  `BankFeesAccountNotConfigured`. Tout autre `DbError` reste `Err` (doc-comment). **`GroupedSettlement`** :
  `journal_entry_id`, `reconciliation_audit_id`, la `Decomposition { difference, rounding_difference, gap }` de D4 bis
  (une seule définition pour les deux familles, la 43-1b et la 43-2 y renvoient, M4), `difference_account_id`, et par
  facture un `GroupedInvoiceRef { kind, id, number, settlement_id: Option<i64>, line_id, amount }` (montant brut ;
  `settlement_id` absent côté fournisseur).
- **AC6** — Après AC5 : reste dû 0 et `paid_at` posé pour chaque facture ; une clé de lettrage **par** facture (sa ligne de
  vente et sa seule ligne de créance) ; ni banque, ni arrondi, ni écart lettrés ; la ligne de créance de chaque facture
  porte son projet, les autres aucun.

**Dé-rapprochement groupé client**

- **AC7** — `classify` reconnaît l'écriture groupée (D3) : lecture verrouillante de **toutes** ses lignes
  `invoice_settlements` (`fetch_all`, ordre `(journal_entry_id, invoice_id)`) et des factures fournisseur qui la pointent
  (pour la 43-1b) ; variante `ReconciliationKind::GroupedSettlement { journal_entry_id }` (code `"grouped_settlement"`),
  **sans `Vec`** : le type reste `Copy`, la liste se lit à part. `cancel_in_tx` l'aiguille vers
  `grouped_settlements::cancel_in_tx`, **dans l'ordre de l'unitaire** : transaction, classement verrouillant, factures par
  identifiant croissant, écriture et exercice, `ensure_open_in_tx` ; puis :
  1. tous les refus, **avant toute écriture** : par facture (créditée, solde existant, exercice clos, lettrage figé par la
     période) et ceux de l'écriture (`settlement_entry_cancel_blocker`, motifs de propriété ignorés) ; le premier rendu ;
     `expect_unlink_only = Some(true)` → `409 RECONCILIATION_CANCEL_CHANGED` (un groupe n'est jamais un retrait seul, L4) ;
  2. `UPDATE bank_transactions SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL …` comme
     l'unitaire (`wasPreviouslyRejected` à l'audit) ;
  3. délettrage de chaque pièce ;
  4. suppression des lignes `invoice_settlements` de l'écriture ;
  5. contre-passation (`reverse_in_tx`, datée du jour, acteur) ;
  6. chaque facture : `paid_at` projeté, `version + 1` (comme `cancel_settlement_in_tx`) ;
  7. `recheck_open_locked_in_tx` ; audit (D5) avec l'acteur, clé d'API comprise, et une `invoice.settlement_cancelled` par
     facture.
  `entry_cancel_outlook` rend aussi, pour un groupe, **la facture du premier motif par facture** (`kind`, `id`, numéro) ;
  `get_view` rend la variante et la liste `(kind, id, numéro, montant brut)` des factures — lues par la 43-2
  (`groupedInvoices`, `cancelBlockedInvoice`).
- **AC8** — Gestes unitaires sur une facture du groupe : annulation de son règlement refusée (`MATCHED_BANK_TRANSACTION`) ;
  avoir refusé (facture réglée) ; nouveau règlement refusé (payée). Tests.

**Inventaire des sites qui supposent « une écriture, un règlement » (M6, M7, L8)**

- **AC9** — Résolus ici : `classify`, `entry_cancel_outlook`, `get_view` ; `document_owners` (doc-comment corrigé,
  `journal_entries.rs:2253-2258`, le motif nomme le plus petit règlement) ; `journal_entries_modification.rs` (AC2). Test :
  une écriture groupée refuse la modification, la contre-passation directe et le délettrage manuel. **Assumés, sûrs** (au
  Change Log, ligne lue) : `ReversalAuthority::exempts` ; `reversal_blockers` ; `reconciliation_link.rs` (:150, :698-710,
  :811, test `the_settlement_exclusion_matches_the_classification`) ; `letterings/open_items.rs:123` ;
  `credit_notes.rs:383` ; `kesh-report/src/vat_report.rs:339` (soldes `write_off` seulement) ;
  `imported_supplier_invoices.rs:425` (`LIMIT 1`) ; `document_group_frozen_by_periods` ; et, lus **par facture ou par
  règlement** (L8) : `invoice_settlements_write.rs` (dont `cancel_settlement_in_tx` :931-1131, refusé avant par le rang 3),
  `invoice_settlements.rs`, `routes/invoices.rs`, `services/factures.rs`, `invoice_pdf_service.rs`, `invoices.rs`,
  `payment_batches.rs`, `aged_receivables.rs`, `repositories/reconciliation.rs`, `matching.rs`,
  `mcp/outils/factures_fournisseur.rs:226`, `exports/global.rs`, le rejeu `post_restore/20260828000001…sql` (hors fenêtre,
  `EXEMPT_MIGRATIONS`, `post_restore.rs:425-431`). Tout site nouveau trouvé par `git grep -n
  "invoice_settlements\|settlement_journal_entry_id" -- 'crates/*/src/*'` s'ajoute, résolu ou assumé.

**Sauvegarde, test d'accord, montée**

- **AC10** — Aller-retour `.keshbackup` d'une base qui porte un groupe client : empreintes identiques, colonnes de ligne et
  plafond compris ; un `.keshbackup` 0.18.0 (fixture sans les colonnes) s'importe, colonnes de ligne à `NULL`, plafond à
  5.0000.
- **AC11** — Test d'accord `crates/kesh-db/tests/lettering_documents_backfill.rs` : fixture d'un groupe non lettré ; le SQL
  du rattrapage (retiré du rejeu) s'abstient, la synchronisation lettre — divergence écrite au doc-comment.
- **AC12** — Test de montée (résolution **par version**) : base au schéma d'avant peuplée (règlements unitaires clients et
  fournisseurs, réglages de facturation), migration, données intactes, colonnes à `NULL`, plafond à 5.0000, puis un groupe
  se crée. À annoncer pour la recette de montée du tag (L6) : empreintes changées sur `invoice_settlements`,
  `supplier_invoices`, `company_invoice_settings`.

**Manuel d'administration**

- **AC13** — `docs/manual/fr/admin-manual.tex` (`.tex` seulement) : `\subsubsection{Passer à la 0.19.0}` sur le patron de
  « Passer à la 0.18.0 » (:2540) : migration ; sauvegarde écrite par Kesh (`…-0.18.0-to-0.19.0.keshbackup` dans
  `backup/pre-upgrade/`), `KESH_SKIP_PRE_UPGRADE_BACKUP=0.19.0` ; la 0.18.0 refuse de démarrer (« was previously applied but
  is missing ») et refuse toute sauvegarde 0.19.0 (`IMPORT_SCHEMA_MISMATCH`) ; une sauvegarde 0.18.0 s'importe dans la
  0.19.0 (plafond remis à sa valeur par défaut) ; retour par la sauvegarde d'avant la mise à jour seulement, « tout ce qui a
  été saisi depuis est perdu » ; renvoi à la procédure de retour existante ; si la migration échoue au milieu (Kesh ne
  démarre pas, erreur 1060 au rejeu), restaurer la sauvegarde `pre-upgrade/` plutôt que reprendre à la main (L8).

## Tests à écrire, et la preuve du rouge

`crates/kesh-db/tests/grouped_settlements.rs` (neuf, squash, `#[sqlx::test]`) ; rouges d'abord
(`kesh-gate-logs/e431a/rouges.log`) :

- groupes de 2 et 3 factures de **deux clients** ; **les six cas du tableau de D4 bis** (`d`, `e`, `r`), en test pur, et
  en écritures pour les restes à quatre décimales (lignes d'arrondi et d'écart, équilibre, compte d'écart relu par la
  vue) ; facture déjà réglée en partie (son reste) ; écart de −0.40 et −5.00 acceptés au plafond par défaut, −5.01
  refusé ; **trop-perçu** accepté au compte choisi (crédit) ; plafond réglé à 0 : seul l'arrondi automatique (`e = 0`)
  passe ; écart sans compte choisi refusé (`DifferenceAccountRequired`) ; compte choisi = banque, inexistant, non
  imputable, créance → chacun son refus, dans l'ordre de AC5.6 ; deux factures à projets différents (AC6) ;
  **facture au projet archivé après sa validation, réglée en groupe, ligne de créance portant ce projet** (H-B) ; paiement
  daté de plus d'un jour avant une facture refusé (M-B) ;
- un test par variante de `GroupRefusal` ; après chaque refus, **la transaction est validée** par le test puis les tables
  comparées (inchangées) : preuve de D6 (M-C) ;
- **concurrence (H-A, M1)** — protocole en trois temps, qui fait naître l'instantané avant la course (MariaDB, `REPEATABLE
  READ` : l'instantané naît à la première lecture cohérente, non au `BEGIN`) :
  1. `let mut tx = pool.begin()` puis **une lecture cohérente** dans `tx` (`invoice_settlements::amount_due(&mut *tx,
     facture)`) : l'instantané est fixé ;
  2. sur **une autre connexion**, un `settle_invoice` **partiel** de la même facture, **validé** ;
  3. `settle_invoices_in_tx(&mut tx, …)` → `Ok(Err(race_during_update))` nommant la facture ; `ROLLBACK TO SAVEPOINT` fait,
     tables inchangées, reste dû jamais négatif.
  Les deux mutations « `FOR UPDATE` sur la facture à l'étape 4 » et « sans `AND version = ?` » **doivent rougir** (le groupe
  réglerait alors un reste périmé) ; sans l'étape 1, elles survivent — vérifié au rouge et consigné. **Deux groupes qui
  partagent une facture**, même protocole : la transaction du groupe 2 lit (étape 1), le groupe 1 est commité sur une autre
  connexion (étape 2), puis le groupe 2 refuse la facture (`race_during_update`) — issue déterministe ; un `1213` éventuel
  n'est pas asserté, il se rejoue dans le test par `kesh_db::retry::retry_on_deadlock` ;
- dé-rapprochement : complet, refus sur exercice clos (rien d'écrit), `expect_unlink_only = Some(true)` refusé, acteur clé
  d'API dans chaque entrée ; entrelacement « avoir sur une facture du groupe » et dé-rapprochement ;
- AC2 (`PUT` d'une écriture groupée refusé ; `journal_entries_modification.rs`), AC8, AC9, AC10 à AC12 ;
- `group_difference` (D4 bis) : tests unitaires purs — les six cas du tableau, les deux sens, `|montant|` pour une
  transaction au débit, le plafond, l'identité `d = r + e`, un centime jamais absorbé (cinquante factures) ; audit :
  `gap`, `difference_account_id`, `rounding_difference` ; `CHECK` de la colonne de ligne fournisseur (AC1.3) ; égalité
  constante / `DEFAULT` (AC3 bis) ;
- **mutations** (`kesh-gate-logs/e431a/mut/`), **réduites à quatre** (M6) : (1) `FOR UPDATE` sur la facture à l'étape 4
  et (2) sans `AND version = ?` — le test de concurrence rougit (H-A) ; (3) le détachement sans remettre la ligne à `NULL`
  — le test de la `CHECK` rougit (M2) ; (4) le plafond ne borne plus qu'un sens de `e` (`e > plafond` au lieu de
  `|e| > plafond`) — le refus d'un moins-reçu au-delà du plafond rougit (la mutation d'origine, « inverser le sens admis
  de `e` », est sans objet depuis que les deux sens sont admis). Les autres (filtre par ligne, `ROLLBACK TO SAVEPOINT`, projet validé) sont souhaitables si le temps le
  permet, en second lot. *Pourquoi ces quatre et non celles de la P3* (P4 LOW-2) : (3) et (4) gardent les deux
  décisions **neuves** de la P3 (M2, M4), que rien d'autre n'exerce ; le filtre par ligne est déjà exercé par AC6 (une
  clé de lettrage par facture : sans le filtre, les deux lignes de créance d'un même compte tombent dans un seul groupe
  et le test rougit), et `ROLLBACK TO SAVEPOINT` par la preuve de D6 (tables comparées après un refus validé).

## Tâches

- [x] **T0** — Étape 0 : LOW de la passe 3 ; D2 (ordre des lignes de `create_in_tx`) ; appui de la clé étrangère après
  `DROP INDEX` et `CHECK` sur colonne neuve, sur MariaDB 10.11 ; décision écrite levant l'invariant (1 bis) de
  `journal_entries_modification.rs` (AC2), ligne lue citée ; réponse de Guy sur M-A relue (registre de l'épique).
- [x] **T1** — Migration, audit d'idempotence, P6, squash, `journal_entries_modification.rs`, `TABLES_TO_TRUNCATE` (AC1,
  AC2), **et le plafond** : champ de l'entité des réglages de facturation, ses listes de colonnes, lecteur
  `group_settlement_max_difference` et `DEFAULT_GROUP_DIFFERENCE_MAX` (AC3 bis, M5). Commit — la 43-7 peut partir.
- [x] **T2** — Entités et listes de colonnes des règlements, fragment `CLEAR_SETTLEMENT_SQL` (AC1.3), export (AC3). Commit.
- [x] **T3** — **Signatures publiques** documentées : `GroupedSettlement`, `GroupedInvoiceRef`, `GroupedDocument`,
  `GroupRefusal` (toutes ses variantes, celles de la 43-1b comprises), `Decomposition` (champs publics),
  `GroupFamily`, **`group_difference` en `pub`** (pure, sans base : la 43-2 peut l'employer pour ses tests de contrat ;
  P4 LOW-4), `DEFAULT_GROUP_DIFFERENCE_MAX`,
  `settle_invoices_in_tx`, **`pay_supplier_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction_id,
  supplier_invoice_ids, batch_size, acteur, aujourd_hui)`** (corps `unimplemented!` permis à ce commit seulement),
  `cancel_in_tx`, la structure de la vue (factures, facture qui bloque). **Commit transmis à l'orchestrateur mercredi avant
  midi** : la 43-1b et la 43-2 en partent. Au passage en `review`, seul le `unimplemented!` de
  `pay_supplier_invoices_in_tx`, propriété de la 43-1b, peut rester.
- [x] **T4** — Lettrage par ligne (AC4). — [x] **T5** — Règlement groupé client, D6, D7 (AC5, AC6).
- [x] **T6** — Dé-rapprochement, gestes unitaires, inventaire (AC7 à AC9).
- [x] **T7** — Sauvegarde, test d'accord, montée (AC10 à AC12). — [x] **T8** — Manuel d'administration (AC13).
- [x] **T9** — Gates (`kesh-db` : complet en boucle) ; passage en `review` **mercredi 18 h** ; Dev Agent Record ; prompt de
  revue P1.

## Note de version

- **Avant la mise à jour** : faites une sauvegarde complète (base, `.keshbackup`, dossier des documents). La 0.19.0 porte
  une migration ; le retour à la 0.18.0 ne se fait que par cette sauvegarde ; une sauvegarde faite en 0.19.0 ne s'importe
  pas dans une 0.18.0. Kesh écrit aussi la sienne dans `backup/pre-upgrade/` avant de migrer.
- **Ajouté** — Un virement peut régler plusieurs factures clients, d'un ou de plusieurs clients : un seul règlement, chaque
  facture soldée et lettrée ; un écart — moins reçu ou trop-perçu, 5.00 au plus par défaut, réglable — passe au compte
  que vous choisissez, les fractions de centime au compte de différences d'arrondi ; le dé-rapprochement le défait en
  entier (refs #823).

## Dev Notes

### Modules touchés

`kesh-db/migrations` (+ squash, audit, `migrations_upgrade_path.rs`, `journal_entries_modification.rs`, `backup.rs`),
`entities`, `repositories/grouped_settlements` (neuf), `repositories/letterings`, `repositories/reconciliation_cancel` (+
listes de colonnes de `invoice_settlements`, `supplier_invoices`, `company_invoice_settings` ; forme de `create_in_tx` et
doc-comment de `journal_entries`), `kesh-api/exports/csv_tables`, `kesh-reconciliation/matching.rs` (littéral d'AC3).

### Dérogation règle de splitting

**Le décompte dépasse la borne** (`CLAUDE.md`, § *Règle de splitting préventif*, M6) : **trois crates** (`kesh-db`,
`kesh-api`, `kesh-reconciliation`) et **huit modules de dépôt** (`grouped_settlements`, `letterings`,
`reconciliation_cancel`, `journal_entries`, `company_invoice_settings`, `invoice_settlements`, `supplier_invoices`,
`backup`), plus la migration et le manuel.
- **Justification** : la migration, la création et l'annulation d'un groupe se testent ensemble ; aucun commit fusionné ne
  doit produire une écriture groupée qu'on ne sait pas défaire, ni une clé vers une ligne sans la levée écrite de
  l'invariant de la modification d'écriture. Hors `grouped_settlements`, `letterings` et `reconciliation_cancel`, les
  modules ne reçoivent que des colonnes, une fonction de lecture, un fragment SQL, une forme `pub(crate)`, l'ordre
  d'une liste de vidage, des assertions de garde-fou levées par décision écrite ou un doc-comment. Le travail que les
  autres stories consomment (T1 à T3 : schéma, entités, plafond, signatures) est isolé dans des commits à part, déjà livrés
  à midi.
- **Risque accepté** : une revue plus large qu'une story à cinq modules, et un passage en revue tendu ; il est borné par le
  repli ci-dessous.

### Repli des jalons (M6)

- **Mercredi midi** : commit T3 (T0 à T3), transmis à l'orchestrateur ; la 43-1b, la 43-2 et la 43-7 en partent.
- **Mercredi 18 h** : passage en `review` sur **T0 à T6 (AC1 à AC9)** ; revue P1 sur ce périmètre.
- **Jeudi matin** : second lot **AC10 à AC13** (sauvegarde, test d'accord, montée, manuel d'administration), relu par
  une **passe ciblée** sur ce seul lot.
- **Mutations** : les quatre de la section *Tests* seulement ; les autres en second lot si le temps le permet.
- Ce qui ne se réduit pas : AC7 (dé-rapprochement), H-A avec son protocole (M1), H-C (AC2).
- Conséquence déjà tenue par C-43-18 : sans la 43-2 et la 43-6, la 43-1a ne sort pas.

### Limites assumées

- Pas de règlement partiel dans un groupe : chaque facture est soldée ; l'écart, dans les deux sens, va au compte choisi
  dans la limite du plafond (décision de Guy du 2026-11-17).
- La ligne de banque et les différences d'un groupe n'ont pas de compte analytique, à la différence d'un règlement
  unitaire dont toutes les lignes héritent du projet de la facture.
- Le motif `OwnedBySettlement` d'une écriture groupée nomme une seule facture (la plus petite) ; le gel est juste, le
  message incomplet.

### Hors périmètre et dépendances de publication (L7)

Fournisseurs (43-1b), paiements existants (43-5, sa propre migration), API (43-2), écran (43-3), réglage du plafond
(43-7), refus de la réimputation (43-6). **La 43-1a ne se publie qu'avec la 43-2** (sans API, rien ne l'appelle) **et avec
la 43-6** (la 42-1 publiée réimputerait sinon une écriture groupée) — C-43-18.

### Ce qui doit rester vrai

Un règlement unitaire se crée, se lettre, s'annule et se dé-rapproche exactement comme en 0.18.0 : aucun test existant ne
change pour passer, hors des trois assertions de `journal_entries_modification.rs` levées par décision écrite (AC2).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement de la 43-1a, clé `e431a`, voie 1), le 2026-11-17.

### Completion Notes List

- **Périmètre livré** : T0 à T8 — AC1 à AC13. Le second lot de la fiche (AC10 à AC13) a été écrit le même jour, pendant
  l'attente du verrou de la voie 1 ; il entre donc dans cette revue P1 au lieu d'une passe ciblée jeudi.
- **Contrat** : la règle de la différence a changé quatre fois pendant le développement, sur décisions de Guy et de
  l'orchestrateur (registre `e0190-choix-autonomes-e431a.md`, C-e431a-1, -2, -12, -13, -14) ; la forme définitive est
  D4 bis. Commits de contrat : `3f6a6b22f` (T3), `2bd2117d0`, `a679cedcd`, `241839568` (définitive), fiche alignée
  `c31159276` ; signalés dans `kesh-gate-logs/e43/t3.md`.
- **T0** : MariaDB 10.11.19 — `DROP INDEX` seul en 1553, migration entière acceptée, `CHECK` éprouvées
  (`kesh-gate-logs/e431a/t0-mariadb.log`) ; D2 confirmé au code (`journal_entries.rs:513-530`, `:551-555`).
- **Rouges** : ⚠️ les tests de T5/T6 ont été écrits **après** le code ; la preuve du rouge passe par les mutations.
  Premier passage des tests neufs (`t5-cible-1.log`, code au commit `c31159276`) : 222 tests, 217 passés, 5 rouges, tous
  des défauts des tests (somme de versions rendue en `DECIMAL`, nom de colonne `actor_api_key_id`, fichier à inscrire au
  garde du squash) — `kesh-gate-logs/e431a/rouges.log`. Après correction (`69e75f5fc`) : `t5-cible-2.log`, 29 tests du
  rayon d'impact, 29 passés (dont les deux d'AC10 côté `kesh-api`).
- **Mutations** (`kesh-gate-logs/e431a/mut/`, script `mutations.py` et `campagne.sh`, code commité avant, restauration
  par `git restore --worktree` puis `touch`) — les quatre tuées (`campagne.log`, `mutation-N.log`) : (1) version lue par
  `FOR UPDATE` → `a_concurrent_partial_settlement_is_refused_by_the_version` et `two_groups_sharing_an_invoice_settle_it_once`
  rouges ; (2) sans `AND version = ?` → les mêmes ; (3) fragment `CLEAR_SETTLEMENT_SQL` sans la ligne →
  `a_settlement_line_without_its_entry_is_refused` ; (4) plafond sur un seul sens de `e` → trois rouges (deux tests purs,
  `the_gap_is_admitted_both_ways_up_to_the_ceiling`). La mutation 4 remplace « inverser le sens admis de `e` », sans objet
  depuis la décision de Guy.
- **Gates du passage en `review`** : voir *Gates* ci-dessous (journaux `test-fast-review.log`, `gate-ci-review.log`).
  Frontend non touché ; **E2E non lancé** (à la clôture, consigne de la release).
- **AC9 — inventaire** (`git grep -c "invoice_settlements\|settlement_journal_entry_id" -- 'crates/*/src/*'`, 39 fichiers) :
  résolus ici : `classify`, `entry_cancel_outlook`, `get_view`, `cancel_in_tx` (`reconciliation_cancel.rs`),
  `document_owners` (doc-comment, `journal_entries.rs`), lettrage (`letterings.rs`, AC4), export (`csv_tables.rs`).
  Assumés sûrs, comme la fiche les nomme (lecture par facture ou par règlement, ou garde existante qui refuse avant) ;
  sites **nouveaux** relevés, tous de la documentation ou des types : `kesh-api/src/errors.rs:5393` (détails du refus de
  compte de tiers), `lib.rs:1097` (route de liste des règlements, par facture), `routes/supplier_invoices.rs:94,243`
  (champ de réponse), `services/mod.rs:158` (doc), `entities/invoice.rs:36`, `kesh-db/src/errors.rs` (doc),
  `accounts.rs:372` (doc), `settlement_cancellation.rs:8,74` (doc), `kesh-report/src/errors.rs:27` (doc).
- **Écarts à la consigne Edit** : les tests de T5/T6 (et AC10, AC11, AC12) ont été rédigés par l'outil Write dans des
  brouillons (`kesh-gate-logs/e431a/draft-*.rs`), puis **ajoutés en fin de fichier par `cat >>`** — pour ne pas modifier
  un fichier de test pendant qu'un gate pouvait le compiler. Aucun `sed -i` ni script de remplacement sur le code ; un
  `replace_all` de l'outil Edit (`adjustment` → `gap` dans `grouped_settlements.rs`).
- **AC11** est dans `grouped_settlements.rs` (où vivent les montages du groupe), non dans
  `lettering_documents_backfill.rs` comme l'écrivait la fiche ; AC12 aussi, d'où l'inscription du fichier à
  `ALLOWED_REAL_MIGRATOR_FILES` (`test_schema_guard.rs`).
- **P8** : la migration a changé d'octets une fois avant toute publication (commentaire de la colonne de plafond,
  C-e431a-12.4) — checksum, squash et bases `kesh_e431a`, `kesh_e2e_e431a`, `kesh_ci_e431a` reconstruites.
- **Limites** (au rapport, à ouvrir en issues si l'orchestrateur le juge utile) : la vue (`get_view`) ne lit que les
  membres **clients** d'un groupe ; le dé-rapprochement d'un groupe fournisseur rend `Invariant` jusqu'à la 43-1b ; le
  motif `OwnedBySettlement` d'une écriture groupée ne nomme qu'une facture.

### Gates

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e431a/`, chacun avec son commit. Bases remises à neuf avant chaque
gate complet (`reset-base.sh`).

| Gate | Commit | Résultat | Journal |
|---|---|---|---|
| ciblé T1 | `3ac36d922` | 97/97 | `t1-cible.log` |
| ciblé T5/T6, premier passage | `c31159276` | 222 : 217 passés, 5 rouges (défauts des tests) | `t5-cible-1.log` |
| ciblé, rayon d'impact | `69e75f5fc` | 29/29 | `t5-cible-2.log` |
| mutations (4) | `69e75f5fc` | 4 tuées | `mut/campagne.log` |
| `test-fast --ci`, 1ᵉʳ | `69e75f5fc` | 5534 : 5522 passés, 12 rouges (registres et montages de tests) | `test-fast-review.log` |
| `test-fast --ci`, 2ᵉ | `130b0e60b` | 5534 : 5530 passés, 4 rouges (inventaires lexicaux, montée rc1/0.16) | `test-fast-review-2.log` |
| `test-fast --ci`, 3ᵉ | `bc02e360f` | 5534 : 5533 passés, 1 rouge (registre du compte clos) | `test-fast-review-3.log` |
| **`test-fast --ci`** (fmt, clippy, nextest) | **`69eb2e1ee`** | **5534 passés, 13 ignorés** | `test-fast-review-4.log` |
| **`gate-ci.sh kesh_ci_e431a`** | **`69eb2e1ee`** | **5541 passés, 0 échec, 17 ignorés** (recompté : somme des `test result:` du journal) | `gate-ci-review.log` |
| revue P1 — entrelacement, rouge | `16172c632` (code `ad77f3e21`) | rouge : `Ok(… reversal Some(4))`, contre-passé en exercice clos | `p1-rouge-entrelacement.log` |
| revue P1 — rayon d'impact | avant `58d949434` | 93 : 92 passés, 1 rouge de montage (CHECK `chk_invoices_validated_has_je`) ; entrelacement vert | `p1-vert-entrelacement.log` |
| revue P1 — `binary(grouped_settlements)` | `58d949434` | 29/29 | `p1-cible-2.log` |
| revue P1 — fmt, clippy, `nextest --profile ci` (`test-fast` sans son garde des droits, refusé par un droit `kesh_ci_e437` d'un autre agent) | `58d949434` | **5549 passés, 13 ignorés** | `test-fast-p1-sans-garde.log` |
| revue P1 — **`gate-ci.sh kesh_ci_e431a`** | `58d949434` | **5556 passés, 0 échec, 17 ignorés** | `gate-ci-p1.log` |
| revue P2 — ciblé `binary(grouped_settlements) \| binary(reconciliation_cancel)` | `5602c0d21` (avant commit) | 43/43 | `p2-cible-1.log` |
| revue P2 — mutation 5 (ancien ordre têtes/queue) | `5602c0d21` | tuée : `every_invoice_head_comes_before_the_entry_ranks` rouge (`FiscalYearClosed`), 33 autres passés | `mut/mutation-p2-ordre.log` |
| revue P2 — **`gate-ci.sh kesh_ci_e431a`** | `5602c0d21` | **5561 passés, 0 échec, 17 ignorés** | `gate-ci-p2.log` |
| frontend | — | non touché, non lancé | — |
| **clôture — `test-fast --ci`** (garde des droits conforme) | `f711dd2de` | **5554 passés, 13 ignorés** ; 1 flaky hors périmètre (`kesh-api::mcp_transport_e2e abandon_au_delai_le_permis_reste_pris`, passé au 2ᵉ essai) | `test-fast-cloture.log` |
| **clôture — `gate-ci.sh kesh_ci_e431a`** | `f711dd2de` | **5561 passés, 0 échec, 17 ignorés** | `gate-ci-cloture.log` |
| **clôture — frontend complet** (`check`, `lint-i18n-ownership`, `test:unit`, `build`, après `npm ci`) | `f711dd2de` | vert ; Vitest **3948 passés (246 fichiers)** | `frontend-cloture.log` |
| **clôture — E2E complet** (recette de `docs/testing.md`, `kesh_e2e_e431a` recréée, port 3143) | `f711dd2de` | **437 passés, 7 échecs, 18 ignorés** — les sept de la KF-029 (#97) : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` ; aucun autre | `e2e-complet-cloture.log` |
| E2E (passage en `review`) | — | non lancé : à la clôture (consigne de la release) | — |

Les rouges intermédiaires ne venaient d'aucun défaut du code de production : registres et décomptes des inventaires
(`bank_account_closed_inventory.rs`, `letterings_lexical.rs`), montages de tests qui exercent le code courant sur un
schéma d'avant (`lettering_documents_backfill.rs`, `migrations_upgrade_path.rs`, `company_invoice_settings_repository.rs`)
— sauf un : le `DELETE` des règlements d'un groupe vivait hors de son module (inventaire des écrivains, AC8 de la
15-1a2-i), déplacé dans `invoice_settlements_write::delete_grouped_settlements_in_tx` (`bc02e360f`).

### File List

- `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (neuf), `crates/kesh-db/migrations.sha384`,
  `crates/kesh-db/test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md`
- `crates/kesh-db/src/repositories/grouped_settlements.rs` (neuf), `repositories/mod.rs`,
  `repositories/reconciliation_cancel.rs`, `repositories/letterings.rs`, `repositories/journal_entries.rs`,
  `repositories/invoice_settlements.rs`, `repositories/invoice_settlements_write.rs`, `repositories/supplier_invoices.rs`,
  `repositories/company_invoice_settings.rs`, `src/backup.rs`
- `crates/kesh-db/src/entities/{invoice_settlement,supplier_invoice,company_invoice_settings}.rs`
- `crates/kesh-reconciliation/src/matching.rs`, `crates/kesh-api/src/exports/csv_tables.rs`,
  `crates/kesh-api/src/routes/reconciliation.rs`
- Tests : `crates/kesh-db/tests/grouped_settlements.rs` (neuf), `journal_entries_modification.rs`,
  `migrations_upgrade_path.rs`, `test_schema_guard.rs`, `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `docs/manual/fr/admin-manual.tex` (§ *Passer à la 0.19.0*, `.tex` seulement)
- `_bmad-output/implementation-artifacts/43-1a-reglement-groupe-modele-client.md`,
  `e0190-choix-autonomes-e431a.md` (neuf), `43-1a-reglement-groupe-modele-client-review-prompt-p1.md` (neuf)

## Change Log

- 2026-11-17 — Fiche née du découpage de la 43-1 (validation P1, H3), findings H1, H2, M1-M7, L1-L8 appliqués ; décisions
  de Guy du 2026-11-17 (tiers mixtes, écart, ordre de sortie).
- 2026-11-17 (nuit) — Remédiée après la validation P2 (0/3/4/8) : H-A (méthode de l'unitaire, aucun `FOR UPDATE` en amont,
  test du règlement partiel intercalé), H-B (projet estampillé sans validation, D7, refus de projet retiré), H-C (invariant
  de `journal_entries_modification.rs` levé par décision écrite, test du `PUT`), M-A (proposition de l'orchestrateur en
  attente de Guy : frais bancaires, trop-perçu refusé, arrondi séparé), M-B (borne basse de date), M-C (point de sauvegarde
  propre, D6), M-D (colonne de la 43-5 sortie de la migration ; 92/93) ; L1 à L8 ; exigences des validations P1 de la 43-2
  et de la 43-3 (brut, clés d'audit snake_case et `batch_size`, signature de `pay_supplier_invoices_in_tx`,
  `GroupedInvoiceRef` fournisseur, `GroupedDocument`, facture qui bloque, `Copy`) ; Q4 (plafond réglable, colonne et
  lecture ici, réglage en 43-7).
- 2026-11-17 (nuit, P3) — Remédiée après la validation P3 (0/0/6/10) : M1 (protocole du test de concurrence en trois
  temps, test des deux groupes déterministe), M2 (les deux sites `supplier_invoices.rs:2639` et `:2929`, fragment partagé,
  test de la `CHECK`), M3 (`create_copying_projects_in_tx` en `pub(crate)`), M4 (une définition des signes, fonction pure
  `group_difference`), M5 (plafond livré à T1), M6 (dérogation écrite, repli des jalons, mutations réduites à quatre) ; L1
  (« à deux écarts près », nommés), L2 (`enforce_postable = false`), L3 (doc-comment du point de sauvegarde), L4 (reste dû
  vérifié à zéro avant `paid_at`), L5 (ancre non lue), L6 (sites du compteur), L7 (couples `(table, colonne)`), L8
  (reprise au manuel), L9 (constante contre `DEFAULT`), L10 (défaut sans réponse de Guy). Passe 4 ciblée à lancer.
- 2026-11-17 (développement, étape 0) — **Clôture de la validation** : P1 (43-1, Opus) → découpage ; P2 0/3/4/8 ; P3
  0/0/6/10 ; P4 ciblée (Haiku) 0/0/1/6, MEDIUM dans la fiche 43-2 (corrigé `7d8859aa3`) ; P5 ciblée (Haiku) sur ce seul
  commit 0/0/0/4 pour la 43-1a (son MEDIUM porte sur la fiche 43-3, hors du commit, laissé à l'orchestrateur). LOW
  traités : P4 LOW-1 (dérogation, liste des modules), LOW-2 (pourquoi ces quatre mutations), LOW-3 (le grep fait foi,
  historiques exclus — lignes lues `migrations_upgrade_path.rs:97`, `:131`, `:159-160`, `:221`), LOW-4 (`Decomposition`,
  `GroupFamily`, `group_difference` en `pub` nommés à T3) ; P5 LOW-1 (« apports bruts », ligne 235). Non traités ici :
  P4 LOW-5 et LOW-6 (arbitrage et calendrier, au rapport — LOW-5 tranché depuis par Guy, C-e431a-1), P5 LOW-2 et LOW-3
  (fiche 43-2, pas la mienne), LOW-4 (trailer d'un commit passé). **T0** : sur MariaDB 10.11.19 (base jetable
  `kesh_e431a_t0`, `kesh-gate-logs/e431a/t0-mariadb.log`) — `DROP INDEX uq_invoice_settlements_entry` seul : ERROR 1553 ;
  migration entière (clé composée d'abord) : acceptée, `fk_invoice_settlements_entry` gardée ; `CHECK` de la ligne
  fournisseur : 4025 sans écriture, acceptée avec ; `CHECK` du plafond : 100.0001 et −0.0001 refusés, 100 et défaut 5.0000
  acceptés. **D2** : `create_in_tx_inner` insère `line_order = idx + 1` et relit `ORDER BY line_order`
  (`journal_entries.rs:513-530`, `:551-555`) — attribution par position confirmée.
- 2026-11-17 (développement, T3) — **Contrat modifié** par la décision de Guy (règle de la différence) et par les
  ajustements de l'orchestrateur (registre `e0190-choix-autonomes-e431a.md`, C-e431a-1 et C-e431a-2) : `gap` au lieu de
  `bank_fees`, `difference_account_id` en paramètre et dans le résultat, refus du compte d'écart (`Required`,
  `IsBankLedger`, `NotPostable`, `Invalid`), `ClaimAccount` aux rôles réels, `CurrencyMismatch`, `WrongDirection`,
  `Overpayment`, `Underpayment` et `BankFeesAccountNotConfigured` retirés. Les AC5 (étape 6) et la section *Tests*
  (frais, trop-perçu) se lisent sous cette règle.
- 2026-11-17 (développement) — **Fiche alignée sur le code** (validation ciblée `kesh-gate-logs/e43/validation-43-23-p6-
  ciblee.md`, CRITICAL-1 et HIGH-1) : en-tête (C-43-1 à C-43-26, différence tranchée), D4 bis réécrite sur la
  décomposition définitive (`e = centime(d)`, `r = d − e`, tableau des six cas, C-e431a-14), D5 (clés `gap`,
  `difference_account_id`, `rounding_difference`), AC5 (signature avec `difference_account_id`, ordre de l'étape 3,
  étape 6 et ordre des contrôles, écriture de l'écart dans les deux sens, liste finale de `GroupRefusal`), AC6, section
  *Tests* (six cas, mutation 4 redéfinie), note de version, limites.
- 2026-11-17 (développement, T1, `3ac36d922`) — Migration `20261118000001_grouped_settlements` (quatre `ALTER`), ligne
  d'audit d'idempotence (`tracked-by-sqlx`), totaux à 92 aux deux sites, squash repris, `TABLES_TO_TRUNCATE`, plafond
  (entité, listes de colonnes, `group_settlement_max_difference`, `DEFAULT_GROUP_DIFFERENCE_MAX`). **Levée de
  l'invariant (1 bis)** de `journal_entries_modification.rs` (AC2, H-C), décision écrite dans le test
  (`journal_entries_modification.rs:85-90`, puis `:175`) et reprise ici : « une écriture dont une ligne est référencée
  par un règlement est **possédée** (`document_owners` : `OwnedBySettlement`, `OwnedBySupplierInvoice`,
  `MatchedBankTransaction`), donc jamais réécrite par le `PUT` (`modification_guard`) » ; les clés vers une ligne sont
  inventoriées — exactement `(invoice_settlements, journal_entry_line_id)` et `(supplier_invoices,
  settlement_journal_entry_line_id)`. **Sites P6** (`git grep -ln "migrations.len()\|apply_migrations_up_to" -- crates`,
  onze fichiers) : `migrations_upgrade_path.rs` positionnel assumé, `assert_eq!(total, 92)` (`:135`) et `total - 58`
  (`:230`, frontière inchangée à 34, ligne historique ajoutée `:223`) ; **résolus par version**, sans changement :
  `kesh-api/src/pre_upgrade_backup/tests.rs:182` et `:506` (`migrations_before(PREMIERE_EN_ATTENTE, "mcp_prompts")`),
  `kesh-api/tests/sauvegarde_avant_migration.rs:28-30` (`migrations_before(20261027000001, "mcp_prompts")`), et le test
  neuf d'AC12 (`grouped_settlements.rs`, `migrations_before(GROUPED_VERSION, "grouped_settlements")`) ; les fichiers de
  rattrapage (`*_backfill.rs`, `journal_entry_number_sequences_bootstrap.rs`) visent une version passée, par version.
- 2026-11-17 (développement, T2, `3f6a6b22f`) — Entités et listes de colonnes, fragment `CLEAR_SETTLEMENT_SQL`, export.
  **Inventaire des listes de colonnes (AC3)**, par `git grep -c` sur `crates/*/src/*` : `journal_entry_line_id` dans 9
  fichiers (`csv_tables.rs`, `routes/reconciliation.rs`, `backup.rs`, `entities/invoice_settlement.rs`,
  `grouped_settlements.rs`, `invoice_settlements.rs` — `COLUMNS` et `insert` —, `invoice_settlements_write.rs`,
  `letterings.rs`, `reconciliation_cancel.rs`) ; `settlement_journal_entry_line_id` dans 8 (`csv_tables.rs`, `backup.rs`,
  `entities/supplier_invoice.rs`, `grouped_settlements.rs`, `letterings.rs`, `reconciliation_cancel.rs`,
  `supplier_invoices.rs` — `FIND_SCOPED_SQL`, `SUPPLIER_INVOICE_COLUMNS_SI`, `list_all_by_company` —, `matching.rs`) ;
  `group_settlement_max_difference` dans 4. Les autres fichiers qui nomment `settlement_bank_account_id` (13) ne
  portent pas de liste de colonnes à étendre : `mcp/outils/factures_fournisseur.rs:224-226` (champs de l'entité),
  `routes/supplier_invoices.rs` (réponse), `post_restore.rs` et son rejeu `20260828000001` (hors fenêtre),
  `reconciliation_link.rs` (prédicats `WHERE`).
- 2026-11-17 (développement, T3) — voir l'entrée « T3 » ci-dessus ; commits `3f6a6b22f`, `2bd2117d0`, `a679cedcd`,
  `241839568`.
- 2026-11-17 (développement, T4, `3f6a6b22f`) — Lettrage par ligne : `SYNC_SETTLEMENT_ENTRIES_SQL` et
  `SYNC_SUPPLIER_INVOICE_SQL` lisent la ligne du règlement ; `keep_owned_lines` ne garde que la ligne de la pièce.
  ⚠️ Écart à AC4 (« les tests existants de `letterings` passent sans modification ») et à *Ce qui doit rester vrai* :
  `letterings_lexical.rs` a changé (écrivains 2 → 3, créateurs, motif `{CLEAR_SETTLEMENT_SQL}` reconnu,
  `bc02e360f`) — un registre, aucun garde-fou affaibli (revue P1, A LOW-6).
- 2026-11-17 (développement, T5, `3f6a6b22f`, `2bd2117d0`, `a679cedcd`, `241839568`) — Règlement groupé client, D6
  (point de sauvegarde), D7 (projets), différence décomposée (C-e431a-14).
- 2026-11-17 (développement, T6, `3f6a6b22f`, `bc02e360f`) — Dé-rapprochement, gestes unitaires. **AC9, sites assumés,
  ligne lue** : `ReversalAuthority::exempts` (`journal_entries.rs:2662`) et `reversal_blockers` (`:2472`) — par écriture,
  le groupe est possédé, rien ne change ; `reconciliation_link.rs:150` (`LINKABLE_SETTLEMENTS`, par règlement),
  `:698-710` (`other_side_predicate` : `NOT EXISTS` sur `s2.journal_entry_id`, vrai dès un règlement) et `:811`
  (`NOT EXISTS`, même lecture) ; `letterings/open_items.rs:121-125` (`EXISTS` par facture) ; `credit_notes.rs:381-386`
  (premier règlement de la facture, `LIMIT 1 FOR UPDATE` — l'avoir est refusé, test
  `a_grouped_invoice_refuses_a_credit_note_and_a_manual_unlettering`) ; `kesh-report/src/vat_report.rs:337-341` (soldes
  `write_off` seulement, jamais groupés) ; `imported_supplier_invoices.rs:423-427` (`ORDER BY si.id LIMIT 1`, une
  facture nommée) ; `document_group_frozen_by_periods` (`letterings.rs:760`, par groupe) ;
  `mcp/outils/factures_fournisseur.rs:224-226` (champs d'une facture) ; rejeu `20260828000001`
  (`post_restore.rs:425-431`, `EXEMPT_MIGRATIONS`, hors fenêtre). Les sites lus par facture ou par règlement (L8) et
  les sites nouveaux sont au Dev Agent Record.
- 2026-11-17 (développement, T7, `6713a90d5`, `2e437efaa`, `40744cc78`) — AC10 (aller-retour, sauvegarde 0.18.0), AC11
  et AC12 — tous deux dans `grouped_settlements.rs`, non dans `lettering_documents_backfill.rs` que nommait AC11 (revue
  P1, A LOW-8), d'où l'inscription du fichier à `ALLOWED_REAL_MIGRATOR_FILES`.
- 2026-11-17 (développement, T8, `2bd2117d0`) — Manuel d'administration, § *Passer à la 0.19.0* (`.tex` seulement).
- 2026-11-17 (développement, T9, `69eb2e1ee`, `ad77f3e21`) — Gates du passage en `review` (*Gates*), Dev Agent Record,
  prompt de revue P1. ⚠️ *Ce qui doit rester vrai* disait « aucun test existant ne change pour passer, hors des trois
  assertions de `journal_entries_modification.rs` » : ont aussi changé des **registres et montages** —
  `letterings_lexical.rs`, `bank_account_closed_inventory.rs` (41 sites, 4 créateurs, 14 gardes, 9 revérifications),
  `lettering_documents_backfill.rs`, `migrations_upgrade_path.rs` (`draft_migration_preserves_a_populated_base`),
  `company_invoice_settings_repository.rs` — aucun garde-fou affaibli (revue P1, A LOW-6).
- 2026-11-17 (revue P2, 0/0/1/7, `5602c0d21`) — Tests seuls, plus un doc-comment ; aucune autre ligne de production.
  MEDIUM-1 : `every_invoice_head_comes_before_the_entry_ranks` épingle l'ordre de C-e431a-15 (`b` créditée et exercice
  clos → `InvoiceCredited` nommant `b`), mutation de l'ancien ordre tuée. LOW-1 :
  `a_successful_group_releases_its_savepoint` (`ROLLBACK TO` après succès → 1305). LOW-2 :
  `a_missing_gap_line_is_an_invariant_for_the_view` et `a_group_without_client_settlements_is_an_invariant` ; le bras
  groupé d'`entry_cancel_chain` reste inatteignable, commentaire au code (`reconciliation_cancel.rs`, retour anticipé).
  LOW-3 : constat 4025 par le numéro MariaDB (`numero_mariadb`). LOW-4 : libellé coupé à 500 caractères sur des
  numéros accentués (`len() > 500`). LOW-5 : `step_four_refuses_in_the_order_of_ac5_4`. LOW-7 : doc-comment du point
  de sauvegarde après un refus. LOW-6 : au registre (C-e431a-16), pour la 43-2, non traité.
- 2026-11-18 (revue P3 ciblée sur `5602c0d21`, 0/0/0/3 ; `f711dd2de`) — LOW-1 : le test de l'étape 4 renommé
  `step_four_refuses_before_the_date_when_nothing_is_due_or_the_sale_is_malformed`, doc-comment ramené à ce qu'il
  épingle. **Non traités** (décision de l'orchestrateur) : LOW-2 — l'assertion 4025 ne contrôle que le numéro, non le
  nom de la contrainte `chk_invoices_validated_has_je` ; LOW-3 — le downcast MySQL de `numero_mariadb` (test) reprend
  celui de `is_savepoint_lost` (production), à regrouper si un troisième site apparaît.
- 2026-11-18 — **Clôture de la revue de code** : P1 complète en trois lentilles, Opus (B 1 HIGH / 0 MEDIUM / 6 LOW ;
  E 0/0/2/5 ; A 0/0/4/8 — ensemble 1 HIGH, 6 MEDIUM, 19 LOW) → P2 complète, contexte frais (0/0/1/7, le HIGH fermé,
  aucune régression) → P3 ciblée sur la remédiation de la P2 (0/0/0/3). Modèles de la P2 et de la P3 : choisis par
  l'orchestrateur, non déclarés dans leurs rapports. Reclassements : B-2, E-3, E-7 et P2 LOW-6 laissés aux 43-1b et
  43-2 (registre C-e431a-16), P3 LOW-2 et LOW-3 non traités ; aucun MEDIUM+ reclassé en dette. Rouges constatés : le
  test d'entrelacement de B-1 rouge sur le code d'avant puis vert ; mutation 5 (ancien ordre têtes/queue) tuée. Gates de
  clôture au dernier commit de code `f711dd2de` (tableau *Gates*) : `test-fast --ci` 5554/13, `gate-ci` 5561/0/17,
  frontend vert (Vitest 3948), E2E 437 passés et les sept échecs de la KF-029 seuls. **Statut `done`.**
