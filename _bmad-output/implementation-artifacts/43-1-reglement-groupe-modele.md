# Story 43-1 : un règlement pour plusieurs factures — le modèle, la migration et le dépôt (#823)

Status: split

> **2026-11-17 au soir — découpée** (validation P1 `kesh-gate-logs/e43/validation-43-1-p1.md`, H3 ; décisions de Guy du
> même jour) en **43-1a** (`43-1a-reglement-groupe-modele-client.md`), **43-1b** (`43-1b-reglement-groupe-fournisseur.md`)
> et **43-6** (`43-6-raccord-reimputation.md`, ex-T-R). Cette fiche reste lisible ; en cas d'écart, les fiches nées du
> découpage et le fichier d'épique révisé (C-43-1 à C-43-20) font foi. Les findings H1, H2, M1 à M7 et L1 à L8 y sont
> appliqués.

*Fiche rédigée le 2026-11-17 par l'agent de rédaction de l'Epic 43 (Opus 5.5), worktree `kesh-43`, branche
`story/43-transaction-plusieurs-factures` partie de `epic/43-transaction-plusieurs-factures` (= `main` = v0.18.0,
`8b50fd195`), par le workflow `bmad-create-story`. Release **0.19.0** (coupure jeudi 2026-11-19 au soir). **Porte la seule
migration de la release.** **`kesh-db` touché : gate complet même en cours de boucle.** Clé de base `e43` (bases
`kesh_e431`, `kesh_e2e_e431`, `kesh_ci_e431` : à confirmer par le prompt de développement). Décisions de l'epic :
`_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md`, **C-43-1 à C-43-13** ; elles font foi.
Validation : à lancer.*

**Issues** : `refs #823` dans chaque commit ; aucun mot-clé de fermeture dans un commit. La fermeture de #823 se décide à
la PR de release (question Q1 de l'epic).

## Story

En tant que comptable qui reçoit un virement pour trois factures (ou qui paie trois factures d'un fournisseur en un
virement), je veux que Kesh enregistre **un** règlement qui solde chacune et la lettre, et qu'un dé-rapprochement le
défasse en entier — pour que mon grand livre de banque montre le mouvement du relevé, et mes comptes de tiers des factures
soldées une à une.

Cette story pose le **modèle** et les **fonctions du dépôt** ; l'API est la 43-2, l'écran la 43-3.

## Ce qui se lit au code (`8b50fd195`)

- Une transaction pointe une écriture : `bank_transactions.matched_entry_id` (`20260504000001_bank_imports.sql:71`) ; huit
  chemins la posent (`reconciliation_cancel.rs:11-30`).
- `invoice_settlements` : `UNIQUE KEY uq_invoice_settlements_entry (journal_entry_id)`
  (`20260827000001_invoice_settlements.sql:65`) — la contrainte écrit elle-même que « le règlement groupé … n'est pas
  couvert ; la contrainte le rend IMPOSSIBLE plutôt que silencieusement faux » (:42-44).
- `supplier_invoices.settlement_journal_entry_id` sans unicité (`20260628000001_supplier_invoices.sql:47`).
- Lettrage de pièce : la découverte lit toutes les lignes du compte de tiers des écritures de règlement
  (`letterings.rs:1477` `SYNC_SETTLEMENT_ENTRIES_SQL`, :1497 `SYNC_DOCUMENT_LINES_SQL`, :1536 `discover_invoice_document`,
  :1901 `discover_supplier_invoice_document`).
- Dé-rapprochement : `reconciliation_cancel.rs` `classify` (:158) lit **un** règlement client (`fetch_optional`, :166-175)
  puis **une** facture fournisseur (`SUPPLIER_SETTLEMENT_CLASSIFY_SQL`, :117) ; `cancel_in_tx` (:581).
- Les lignes d'un règlement naissent sans compte analytique (`invoice_settlements.rs:246-255`,
  `settlement_journal_lines`).
- Les sites qui vident les colonnes de règlement fournisseur : `supplier_invoices.rs:2639` (annulation d'une facture payée,
  détachement) et :2929 (annulation du règlement) ; celui qui les pose : :2179 (`pay_in_tx_as`).
- Le rattrapage rejoué à chaque import : `20261010000001_lettering_documents_backfill.sql` (classe A,
  `post_restore.rs`), tenu d'accord avec la synchronisation par `crates/kesh-db/tests/lettering_documents_backfill.rs`.
- Branche `origin/epic/42-reimputation-analytique` : `project_reassignment.rs:443-453` lit les écritures de règlement
  d'une facture ; `reassign_lines_in_tx` (:352-363) réimpute toutes leurs lignes.

## Décisions propres à la story

- **D1** — Le code du règlement groupé vit dans **un module neuf**, `crates/kesh-db/src/repositories/grouped_settlements.rs`,
  qui **compose** les primitives existantes (lecture du compte de tiers, création d'écriture, lettrage, contre-passation,
  audit) sans en réécrire aucune. Les modules existants ne gagnent que la lecture par ligne et l'aiguillage.
- **D2** — Les lignes de l'écriture groupée suivent l'ordre des factures par **identifiant croissant** (aussi l'ordre des
  verrous) ; la ligne de banque en premier. L'attribution ligne → facture se fait par cet ordre, après `create_in_tx` ;
  le développement vérifie à l'étape 0 que `create_in_tx` rend les lignes dans l'ordre fourni (ligne lue citée au Change
  Log), sinon il apparie par une clé sans ambiguïté (compte et montant ne suffisent pas : deux factures peuvent avoir le
  même reste dû).
- **D3** — Une écriture groupée est **reconnue** à ceci : au moins un règlement (client ou fournisseur) qui la pointe porte
  un identifiant de ligne. Un règlement unitaire n'en porte pas (rien ne change pour lui).
- **D4** — Le dé-rapprochement groupé (C-43-6) contre-passe l'écriture **après** avoir retiré ses propriétaires, par
  `journal_entries::reverse_in_tx` — aucune autorité neuve, `journal_entries.rs` n'est pas touché.

## Critères d'acceptation

**Migration**

- **AC1** — `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql`, DDL seul, en cet ordre :
  1. `invoice_settlements` : `ADD COLUMN journal_entry_line_id BIGINT NULL`, clé étrangère
     `fk_invoice_settlements_entry_line` → `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY
     uq_invoice_settlements_entry_line (journal_entry_line_id)`, `UNIQUE KEY uq_invoice_settlements_entry_invoice
     (journal_entry_id, invoice_id)` ;
  2. puis, instruction à part, `DROP INDEX uq_invoice_settlements_entry` (la clé étrangère `fk_invoice_settlements_entry`
     s'appuie alors sur la nouvelle clé composée, colonne de tête) ;
  3. `supplier_invoices` : `ADD COLUMN settlement_journal_entry_line_id BIGINT NULL`, clé étrangère
     `fk_supplier_invoices_settlement_line` → `journal_entry_lines(id) ON DELETE RESTRICT`, `UNIQUE KEY
     uq_supplier_invoices_settlement_line (settlement_journal_entry_line_id)`, `CONSTRAINT
     chk_supplier_invoices_settlement_line CHECK (settlement_journal_entry_line_id IS NULL OR settlement_journal_entry_id IS
     NOT NULL)`.
  En-tête : non breaking (C-43-4, avec ses raisons), DDL seul (P7), pas de bump. Appliquée sur MariaDB 10.11 à une base
  **peuplée** par la 0.18.0 (règlements clients et fournisseurs), sans erreur ; le test de montée le prouve (AC10).
- **AC2** — `docs/migrations-idempotence-audit.md` : une ligne (verdict `tracked-by-sqlx`, justification), les deux totaux
  à **92** et les trois compteurs de partition recomptés depuis le tableau (`ls crates/kesh-db/migrations/*.sql | wc -l`,
  `grep -c '^| \`20' docs/migrations-idempotence-audit.md`). Inventaire P6 (`grep -rn "migrations.len()\|apply_migrations_up_to"
  crates/`) : chaque site positionnel garde son sens ou son garde-fou, relevé au Change Log. Le squash
  `crates/kesh-db/test-schema/0001_schema_squash.sql` reprend le schéma ; `crates/kesh-db/tests/test_schema_guard.rs` vert.
  `every_data_backfill_migration_is_triaged` vert sans entrée nouvelle.

**Entités et lectures**

- **AC3** — `InvoiceSettlement.journal_entry_line_id: Option<i64>` et `SupplierInvoice.settlement_journal_entry_line_id:
  Option<i64>` ; toutes les listes de colonnes qui hydratent ces entités les portent (`invoice_settlements.rs` `COLUMNS`,
  `supplier_invoices.rs` :44 et :56, et tout autre site trouvé par `git grep -n "settlement_journal_entry_id"` — inventaire
  au Change Log). L'export de souveraineté (`crates/kesh-api/src/exports/csv_tables.rs`, `serialize_invoice_settlements_csv`
  :1386 et l'export `supplier_invoices` :1232) ajoute la colonne ; le `.keshbackup` la porte (aller-retour d'AC9).

**Règlement groupé — clients**

- **AC4** — `grouped_settlements::settle_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction, invoice_ids,
  acteur, aujourd_hui) -> Result<Result<GroupedSettlement, GroupRefusal>, DbError>` :
  - verrouille les factures par identifiant croissant, puis le compte bancaire, dans l'ordre de l'acceptation unitaire ;
  - refuse (`GroupRefusal`, sans rien écrire) : moins de 2 ou plus de 50 factures, doublon, facture introuvable ou d'une
    autre société, non `validated`, déjà payée, sans écriture de vente, tiers différents (`contact_id`), transaction qui
    n'est pas un crédit, compte de créance d'une facture égal au compte de la banque (`refuse_if_claim_account`), compte
    bancaire non lié, clos ou archivé, somme `Σ amount_due_to_centime(reste dû)` ≠ montant de la transaction (le refus
    porte les deux sommes) ; le refus nomme la facture en cause quand il y en a une ;
  - passe **une** écriture datée comme l'acceptation unitaire (date de valeur, à défaut de comptabilisation) : débit de la
    banque du montant de la transaction ; crédit, par facture, de **son** compte de créance (lu sur son écriture de vente,
    `sale_receivable_account`) de son **reste dû brut** ; si la somme des restes bruts diffère du montant (fractions de
    centime), une ligne d'écart au compte de différences d'arrondi, comme `settlement_journal_lines` ;
  - insère une ligne `invoice_settlements` par facture (`amount` = reste brut, `settled_on`, `settlement_type =
    'bank_transfer'`, `settlement_bank_account_id`, **`journal_entry_line_id` = sa ligne de créance**) ;
  - projette `paid_at` et `version + 1` sur chaque facture (même règle que l'acceptation unitaire) ;
  - pose `matched_entry_id`, `status = 'reconciled'`, `version + 1` sur la transaction (`AND status = 'pending' AND
    matched_entry_id IS NULL`, une ligne exactement, sinon `Invariant`) ;
  - synchronise le lettrage de **chaque** facture (`sync_invoice_in_tx`) ;
  - écrit l'audit de C-43-9 (`invoice.settled` par facture, avec l'acteur) ; `reconciliation.accepted` est écrit par la
    route (43-2), qui reçoit de quoi le détailler dans `GroupedSettlement`.
- **AC5** — Après AC4, pour chaque facture : reste dû 0, `paid_at` posé, sa ligne de vente et **sa seule** ligne de créance
  de l'écriture groupée lettrées ensemble (une clé par facture, N clés distinctes) ; la ligne de banque n'est pas lettrée.

**Règlement groupé — fournisseurs**

- **AC6** — `grouped_settlements::pay_supplier_invoices_in_tx(...)`, même forme : verrous par
  `supplier_invoices::lock_for_bank_settlement_in_tx` facture par facture en ordre croissant (réemployé, non recopié) ;
  refus : bornes, doublon, introuvable, non `open` (`draft`, `paid`, `cancelled`), dans un lot `generated`, tenue par un
  lot du compte, tiers différents, transaction qui n'est pas un débit, compte bancaire, `Σ total_amount` ≠ |montant| ;
  écriture : crédit de la banque de |montant| ; débit, par facture, de **son** compte de dette (lu sur son écriture
  d'achat, comme `purchase_payable_line`) de son `total_amount` ; chaque facture passe `paid` avec `settlement_type =
  'bank_transfer'`, `settlement_bank_account_id`, `settlement_journal_entry_id` = l'écriture,
  **`settlement_journal_entry_line_id` = sa ligne de dette**, `paid_at`, `version + 1` ; transaction rapprochée ; lettrage
  de chaque facture (`sync_supplier_invoice_in_tx`) ; audit `supplier_invoice.paid` par facture, avec l'acteur.
- **AC7** — Pendant de l'AC5 pour les fournisseurs (une clé par facture, sa ligne d'achat et sa ligne de dette).

**Lettrage par ligne**

- **AC8** — La découverte d'une pièce (`discover_invoice_document`, `discover_supplier_invoice_document`) ne retient, d'une
  écriture de règlement dont le règlement porte un identifiant de ligne, **que cette ligne** ; sans identifiant, toutes les
  lignes du compte de tiers, comme avant. Verrous et ordre de lecture inchangés (`FOR UPDATE`, index forcés). Un règlement
  unitaire se lettre exactement comme en 0.18.0 : les tests existants de `letterings` restent verts sans modification.

**Dé-rapprochement**

- **AC9** — `reconciliation_cancel::classify` reconnaît l'écriture groupée (D3) et rend une variante neuve
  `ReconciliationKind::GroupedSettlement` (code `"grouped_settlement"`) ; `cancel_in_tx` l'aiguille vers
  `grouped_settlements::cancel_in_tx`, qui, dans la transaction du geste, après les verrous du geste unitaire (transaction,
  écriture et exercice, puis factures par identifiant croissant) :
  1. évalue, **avant toute écriture**, les refus de chaque facture (rangs du geste unitaire : facture créditée, solde
     existant, exercice clos, lettrage figé par la période, compte bancaire clos) et ceux de l'écriture
     (`settlement_entry_cancel_blocker`, motifs de propriété ignorés) ; le premier refus est rendu, rien n'est écrit ;
  2. remet `matched_entry_id` à `NULL`, `status = 'pending'` ;
  3. délettre chaque pièce (`dissolve_invoice_document_group_in_tx` / `dissolve_supplier_invoice_document_group_in_tx`) ;
  4. supprime les lignes `invoice_settlements` de l'écriture ; rouvre les factures fournisseur qui la pointent (statut
     `open`, colonnes `settlement_*`, **ligne comprise**, et `paid_at` à `NULL`, `version + 1`) ;
  5. contre-passe l'écriture (`journal_entries::reverse_in_tx`, datée du jour, acteur du geste) ;
  6. projette `paid_at` des factures clients ;
  7. écrit l'audit (C-43-9) avec l'acteur, clé d'API comprise.
  `entry_cancel_outlook` et `get_view` rendent la variante, et la liste `(id, numéro, montant)` des factures du groupe
  (nouvelle structure, lue par la 43-2).
- **AC10** — Les gestes unitaires sur une facture d'un groupe restent refusés par la garde existante : annulation du
  règlement client (`MATCHED_BANK_TRANSACTION`), retour en brouillon d'une facture fournisseur payée (`PaidReconciled`),
  annulation du règlement fournisseur. L'**annulation** d'une facture fournisseur payée du groupe la détache (colonnes
  `settlement_*` et **ligne** à `NULL` — le `CHECK` d'AC1 rougit sinon) ; le dé-rapprochement ultérieur contre-passe
  l'écriture entière et rouvre les seules factures encore attachées ; quand plus aucune ne l'est, l'écriture est classée
  `entry` et défaite comme aujourd'hui.

**Sauvegarde, rattrapage, montée**

- **AC11** — Un `.keshbackup` d'une base qui porte un groupe client et un groupe fournisseur se réimporte à empreintes
  identiques, colonnes de ligne comprises ; un `.keshbackup` 0.18.0 (fixture sans les colonnes) s'importe dans la 0.19.0
  (colonnes à `NULL`).
- **AC12** — Le rattrapage `20261010000001` rejoué sur une base qui porte une écriture groupée **non lettrée** (compte non
  lettrable puis rendu lettrable, ou lettrage retiré par la fixture) **s'abstient** pour ces factures (C-43-5) ; le test
  d'accord `lettering_documents_backfill.rs` gagne cette fixture et la divergence assumée (le rattrapage ne lettre pas, la
  synchronisation le fait) est écrite dans son doc-comment.
- **AC13** — Test de montée (`crates/kesh-db/tests/migrations_upgrade_path.rs` ou voisin, résolution **par version**) :
  base peuplée au schéma d'avant (règlements clients unitaires, factures fournisseur payées), migration appliquée, données
  intactes, colonnes de ligne à `NULL`, puis un groupe se crée.

**Raccord Epic 42 (C-43-11)**

- **AC-R** — *Joué sur la branche qui porte 42-1 (et 42-2 si elle est close) et 43-1, avant la coupure.* Sur une écriture
  de règlement commune, « Changer le compte analytique » d'une facture ne réimpute que **sa** ligne de tiers (identifiant
  porté par son règlement) : ni la ligne de banque, ni celles des autres factures ; l'exposition aux périodes closes et
  l'audit restent ceux du geste. Un règlement unitaire est réimputé comme avant (toute l'écriture). Test : un groupe de deux
  factures clients à comptes analytiques différents, réimputation de l'une ; les lignes de l'autre et la ligne de banque
  inchangées. Même test côté fournisseur si 42-2 est fusionnée.

## Tests à écrire, et la preuve du rouge

Fichier neuf `crates/kesh-db/tests/grouped_settlements.rs` (sur le squash, `#[sqlx::test]`), plus les ajouts nommés aux AC.
Chaque test d'AC4 à AC10 s'écrit **avant** le code et rougit d'abord (journal `kesh-gate-logs/e431/rouges.log`) :

- client : groupe de 2 et de 3 factures (restes entiers, puis restes à 4 décimales avec ligne d'écart) ; facture déjà
  réglée en partie (son reste, non son TTC) ; chaque refus d'AC4 (un test par code) ; rien d'écrit après un refus
  (comptes des tables avant/après) ;
- fournisseur : groupe de 2 ; facture d'un lot `generated` refusée ; tiers différents ; montant faux ;
- lettrage : AC5, AC7 ; un règlement unitaire ensuite sur une autre facture se lettre comme avant ;
- dé-rapprochement : client, fournisseur ; avec facture fournisseur annulée entre-temps (AC10) ; refus sur exercice clos
  (rien d'écrit) ; acteur clé d'API dans chaque entrée d'audit ;
- concurrence : deux groupes qui partagent une facture, lancés ensemble — un seul aboutit, l'autre refuse ou se rejoue ;
  aucun interblocage non rejoué (ordre croissant des verrous) ;
- **mutations** (journal `kesh-gate-logs/e431/mut/`) : retirer le filtre par ligne d'AC8 (AC5 rougit) ; inverser l'ordre
  d'attribution d'AC4 sur deux restes différents (AC5 rougit) ; omettre la remise à `NULL` de la ligne au détachement
  (AC10 rougit sur le `CHECK`) ; omettre la suppression des règlements à l'étape 4 d'AC9 (la contre-passation ou le reste
  dû rougit).

## Tâches

- [ ] **T0** — Étape 0 : relire les LOW de la dernière validation ; vérifier D2 (ordre des lignes de `create_in_tx`) et
  l'appui de la clé étrangère après `DROP INDEX` sur MariaDB 10.11 ; lignes lues citées au Change Log.
- [ ] **T1** — Migration, audit d'idempotence, P6, squash (AC1, AC2). Commit.
- [ ] **T2** — Entités, listes de colonnes, export CSV (AC3). Commit.
- [ ] **T3** — **Signatures publiques** de `grouped_settlements.rs` (types `GroupedSettlement`, `GroupRefusal`,
  `GroupedInvoiceRef`, fonctions d'AC4, AC6, AC9) documentées, corps complets ou non : **commit poussé à l'orchestrateur
  pour que la 43-2 parte**. Rien n'y doit rester `todo!()` au passage en `review`.
- [ ] **T4** — Lettrage par ligne (AC8), tests rouges puis verts.
- [ ] **T5** — Règlement groupé client (AC4, AC5).
- [ ] **T6** — Règlement groupé fournisseur (AC6, AC7).
- [ ] **T7** — Dé-rapprochement groupé et gestes unitaires (AC9, AC10).
- [ ] **T8** — Sauvegarde, rattrapage, montée (AC11 à AC13).
- [ ] **T-R** — Raccord Epic 42 (AC-R), sur la branche réunie que désigne l'orchestrateur.
- [ ] **T9** — Manuel d'administration (`docs/manual/fr/admin-manual.tex`, `.tex` seulement) : la 0.19.0 porte une
  migration ; retour à la 0.18.0 par la sauvegarde d'avant mise à jour seulement ; une sauvegarde 0.19.0 ne s'importe pas
  dans une 0.18.0.
- [ ] **T10** — Gates (`e35-dev-commun.md` § *Quels gates, quand*, `kesh-db` : complet en boucle), Dev Agent Record,
  prompt de revue P1.

## Note de version

- **Avant la mise à jour** : faites une sauvegarde complète (base, `.keshbackup` et dossier des documents). La 0.19.0
  porte une migration ; le retour à la 0.18.0 ne se fait que par cette sauvegarde. Kesh écrit aussi la sienne dans
  `backup/pre-upgrade/` avant de migrer.
- **Ajouté** — Un virement peut régler plusieurs factures d'un même client, ou un débit plusieurs factures d'un même
  fournisseur : un seul règlement, chaque facture soldée et lettrée ; le dé-rapprochement le défait en entier (refs #823).

## Dev Notes

### Modules touchés

`kesh-db/migrations` (+ squash, audit), `kesh-db/entities`, `repositories/grouped_settlements` (neuf),
`repositories/letterings`, `repositories/reconciliation_cancel` (+ listes de colonnes de `invoice_settlements`,
`supplier_invoices`), `kesh-api/exports/csv_tables`. Cinq modules métier et la migration : la story est **au bord** de la
règle de splitting ; elle reste une parce que le modèle, la création et l'annulation se testent ensemble (une écriture
groupée qu'on ne sait pas défaire ne doit exister à aucun commit fusionné — patron de la 30-1a).

### Limites assumées

- Le lot pain.001 confirmé reste hors de portée (C-43-10).
- Pas de règlement partiel dans un groupe : somme exacte.
- La ligne de banque d'un groupe n'a pas de compte analytique (comme toute ligne de règlement à sa création) ; après une
  réimputation (AC-R), elle n'en reçoit pas, à la différence d'un règlement unitaire. À écrire au manuel utilisateur (43-3).

### Hors périmètre

Acceptation par l'API, lecture des candidates, écran : 43-2, 43-3.

### Ce qui doit rester vrai

Un règlement unitaire (client, fournisseur, saisi depuis la fiche ou par rapprochement) se crée, se lettre, s'annule et se
dé-rapproche exactement comme en 0.18.0 : aucun test existant n'est modifié pour passer.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche rédigée (kickoff de l'Epic 43, C-43-1 à C-43-13).
