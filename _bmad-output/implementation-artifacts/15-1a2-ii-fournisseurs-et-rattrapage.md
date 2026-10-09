# Story 15.1a2-ii : Le lettrage des pièces fournisseurs, et le rattrapage des données existantes

## Status

ready-for-dev *(découpée de la 15-1a2 le 2026-10-09 à la remédiation de sa validation P1 — C-15-1a2-1 ;
**validation P2 (Opus, contexte frais) à mener avant tout développement**)*

## Story

**As a** indépendant, PME ou fiduciaire qui paie ses fournisseurs dans Kesh, et qui met à jour une
installation où des factures sont déjà soldées,
**I want** que l'achat et le paiement d'une facture fournisseur payée soient lettrés entre eux, et que
les pièces soldées et les contre-passations **d'avant la mise à jour** le soient aussi,
**so that** le grand livre dise, dès la mise à jour et après toute restauration, ce que disent les
pièces.

Seconde des deux sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). ⛔ **Suppose la
15-1a2-i mergée** : elle réutilise sa synchronisation (P3 : découverte verrouillante, groupe existant,
cible, abstention), son évaluation sans verrou des périodes (`lines_in_open_period`, P7), son extension
d'audit (`DocumentRef`, `*_inner`) et sa fixture partagée ; et son test d'accord (AC6) compare le
rattrapage à **cette** synchronisation. Ordre : 15-1a-i → 15-1a-ii → 15-1a2-i → **15-1a2-ii** → 15-1b →
15-1c.

**Numérotation conservée** de la 15-1a2 (P2, P3, P4, P6 ; AC6, AC7, AC8–AC12) ; numéro neuf : **AC16**.
Les éléments partagés avec la 15-1a2-i (AC8, AC9, AC10, AC12, P3, P4) portent ici leur **part ii**.

## Le modèle réel — relevé sur `056997b0`

*(Lignes sur `056997b0` ; **citer et re-greper par le nom de fonction**.)*

**Compte fournisseurs `B`** = celui de **la première ligne au crédit de l'écriture d'achat** (lecture
`supplier_invoices::purchase_payable_line`, `supplier_invoices.rs:660`, déjà faite par `pay_in_tx`).
Achat : **un** crédit sur `B` = TTC ; règlement : **un** débit sur `B` = TTC, **toujours complet** (pas
de paiement partiel fournisseur, une seule colonne de règlement). Mode de règlement : virement
(`bank_transfer`) ou **compte interne** (`internal_account`, `pay_in_tx` ≈ `:776`).

**Écrivains de `supplier_invoices.settlement_journal_entry_id`** — trois `UPDATE` :

| fonction | ligne | effet |
|---|---|---|
| `pay_in_tx` (`:713`) — appelée par `pay` (`:680`) **et** par `payment_batches::confirm_batch` (`payment_batches.rs:400`, appel `:456`) | `UPDATE … status = 'paid' …` `:895` | pose le règlement |
| `cancel_in_tx` (`:1032`) | `UPDATE … status = 'cancelled' …` `:1090` | contre-passe l'achat ; une facture **payée** voit son règlement **détaché** (colonnes à `NULL`), **non** contre-passé |
| `cancel_settlement_in_tx` (`:1229`) | `UPDATE … status = 'open' …` `:1299` | contre-passe le règlement |

Un règlement **détaché** n'est plus possédé (`reversal_blockers` lit `settlement_journal_entry_id`,
désormais `NULL`) : il est **lettrable à la main** (15-1a R5). L'achat d'une facture **annulée**, lui,
reste possédé (`purchase_journal_entry_id` intact, motif `OwnedBySupplierInvoice`) : sa paire `reversal`
**ne se dissout pas à la main** (C106, test
`supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`).

## Décisions

### P2 — Le groupe `document` d'une facture fournisseur

Pour une facture fournisseur **payée** (`status = 'paid'`, `settlement_journal_entry_id` non nul) :
l'**ancre** est la ligne d'achat sur `B` ; `C(S)` = l'ancre et la ligne **sur `B`** du règlement. Même
règle que P1 (15-1a2-i) : `|C| ≥ 2`, somme nulle, `B` lettrable, au moins une ligne en période ouverte.

### P3 (part ii) — La synchronisation d'une facture fournisseur

`sync_supplier_invoice_in_tx(tx, company_id, supplier_invoice_id, held_open_fiscal_year_id, actor)` et
`dissolve_supplier_invoice_document_group_in_tx(…)`, dans `letterings.rs`, **même algorithme** que leurs
jumelles client (15-1a2-i P3, étapes 1 à 6) — factorisé : la découverte change, le reste est commun
(⛔ pas de seconde copie de l'algorithme ; une fonction privée prend la découverte en paramètre).
Compte `B` non lettrable → `SyncOutcome::AccountNotLetterable`, **aucune erreur** : un paiement
n'échoue **jamais** à cause du lettrage (C-15-1a2-3). `documentType = "supplier_invoice"`,
`documentNumber = supplier_invoice_number`.

### P4 (part ii) — Où la synchronisation est appelée

| geste (`056997b0`) | appel | place | exercice tenu (`FOR UPDATE`) | acteur |
|---|---|---|---|---|
| `pay_in_tx` (`:713`) — **couvre** `pay` et `confirm_batch` | `sync_supplier_invoice_in_tx` | après l'`UPDATE … status = 'paid'` (`:895`) | `fy` de `find_open_covering_date(payment_date)` (`:843`) — l'exercice du règlement | `Actor { user_id, api_key_id: None }` |
| `cancel_settlement_in_tx` (`:1229`) | `dissolve_supplier_invoice_document_group_in_tx` | après les refus de l'étape (2), **avant** `reverse_owned_in_tx` (étape (3), `:1285`) | l'exercice du règlement, verrouillé à l'étape (1-bis) (`:1255` — valeur gardée) | idem |
| `cancel_in_tx` (`:1032`), facture **payée** | `dissolve_supplier_invoice_document_group_in_tx` | après les refus de l'étape (2), **avant** `reverse_owned_in_tx` (étape (3), `:1076`) | l'exercice de l'achat, verrouillé à l'étape (1) (`:1050` — valeur gardée) | idem |

Pour une facture **ouverte** annulée, la dissolution est un no-op (aucun groupe). Après la dissolution,
la contre-passation lettre ce qui est libre (15-1a-ii R6) : règlement ↔ miroir (annulation du
règlement), achat ↔ miroir (annulation de la facture). Le **règlement détaché** reste **ouvert**,
lettrable à la main. Un règlement par **compte interne** lettrable (ex. `1000`, actif) voit aussi sa
ligne de contrepartie appariée à son miroir par R6 : AC7 l'asserte (finding F-10).

Périodes closes : la règle de la 15-1a2-i P7 s'applique telle quelle (abstention à la création comme
à la dissolution ; un groupe gardé après l'annulation d'un paiement de période verrouillée laisse le
miroir ouvert).

### P6 — Rattrapage des données existantes : DEUX migrations, triées différemment (C-15-1a2-4)

Les pièces soldées et les contre-passations passées **avant** la 15-1a2 (et, pour les contre-passations,
avant la 15-1a-ii) doivent être lettrées. ⛔ Des migrations qui écrivent des données → triage **P7**.
⛔ **Une migration appliquée ne se modifie plus** (P8) : `20261009000001` (15-1a-i, mergée) n'est pas
touchée.

⚠️ **Pourquoi deux fichiers et non un** (heurt avec la consigne « une migration », signalé) : le
registre de rejeu exige qu'un extrait porte **tous** les statements d'écriture de sa migration
(`extract_carries_every_write_statement_of_its_source_migration`, `post_restore.rs`), et une migration
ne peut être **à la fois** au registre et exemptée (`exemptions_are_real_and_disjoint_from_registry`).
Rejouer les groupes `document` sans rejouer les paires `reversal` libres impose donc deux fichiers.

**M1 — `<AAAAMMJJ>000001_lettering_documents_backfill.sql`** (version postérieure à `20261009000001`,
datée au développement) — **registre `POST_RESTORE_BACKFILLS`, classe A** :

1. les groupes `document` **client** (15-1a2-i P1) ;
2. les groupes `document` **fournisseur** (P2) ;
3. les paires `reversal` dont l'**origine est l'écriture d'achat** d'une facture fournisseur
   (`supplier_invoices.purchase_journal_entry_id`) — factures annulées avant la 15-1a-ii.

**Pourquoi la classe A tient pour M1** (`CLAUDE.md` P7 : « ne pas réutiliser le critère "un `NULL` n'est
l'expression d'aucun choix" ») : aucune des lignes visées ne peut porter un `NULL` **choisi**. Un groupe
`document` ne se dissout pas à la main (`LETTERING_IS_DOCUMENT`) et se **recalcule** depuis
`invoice_settlements`, `credit_notes` et `supplier_invoices` : un groupe dissous à bon droit (règlement
annulé) ne se reforme pas, la pièce ayant changé. Une paire `reversal` dont une ligne est celle d'une
pièce ne se dissout pas à la main non plus (C106). Chaque statement est en outre gardé par
`lettering_key IS NULL` sur **toutes** les lignes du groupe candidat : un rejeu sur une base à jour est
un no-op strict. `sql` = la migration entière (backfill pur, aucun DDL, patron `20260729000001`).
`registry_entries_are_within_import_window` : aucune table applicative créée depuis `20260917000001`.

**M2 — `<AAAAMMJJ>000002_lettering_reversal_pairs_backfill.sql`** — **EXEMPTÉE du rejeu**
(`EXEMPT_MIGRATIONS`, `ExemptionBasis::Durable`) : les autres paires `reversal` (contre-passations
d'écritures manuelles, de rapprochements hors facture, de règlements annulés). **Justification**, qui
ne commence **pas** par « Hors fenêtre » (la migration est dans la fenêtre) : « Rejeu exclu à dessein :
un groupe `reversal` sans ligne de pièce se délettre à la main (`DELETE /letterings`) ; un `NULL` peut y
être un choix de l'utilisateur, qu'un rejeu à chaque import réécrirait en silence. Coût assumé : une
sauvegarde d'avant la 15-1a2 importée laisse ces paires ouvertes, lettrables à la main. »
`EXEMPT_MIGRATIONS.len()` : **16 → 17** (`every_exemption_declares_a_coherent_basis`, commentaire du
compteur à compléter).

**Règles communes du SQL** (M1 et M2) — la **même règle** que la synchronisation, sans quoi AC6 rougit :

- **compte lettrable** : `accounts.account_type IN ('Asset', 'Liability')` **et**
  `NOT EXISTS (SELECT 1 FROM bank_accounts b WHERE b.journal_account_id = a.id)` — le prédicat de
  `letterings::letterable_account`, recopié (C-15-1a2-3) ;
- **au moins une ligne du groupe en période ouverte** : exercice `Open`, aucun exercice postérieur
  `Closed` de la société, `entry_date > COALESCE(companies.books_locked_through, '0001-01-01')` —
  abstention sinon (C-15-1a2-2 ; Reçu points 3 et 10, recommandation du socle suivie) ;
- `|groupe| ≥ 2`, somme nulle, **toutes** les lignes `lettering_key IS NULL` ;
- clé = `MIN(jel.id)` du groupe, origine `document` (M1 1-2) ou `reversal` (M1 3, M2) ;
- paires `reversal` appariées **par position** (Reçu point 7) : rang dans `ORDER BY line_order` de
  l'origine contre rang dans `ORDER BY line_order` du miroir (`journal_entries.reverses_entry_id`), même
  compte, montants croisés — la forme de 15-1a-ii R6 ;
- ⛔ **forme des statements** : `UPDATE journal_entry_lines jel JOIN (SELECT … ) g ON g.line_id = jel.id
  SET …` — table dérivée, **pas** de `WITH` en tête : le détecteur P7 (`writes_data`) classe un statement
  sur son **premier mot-clé**, et un `WITH … UPDATE` lui échapperait ; pas de commentaire `/* */`
  (`migrations_contain_no_block_comment`), pas de littéral piège (`registry_sql_has_no_literal_hazard`) ;
  la faisabilité MariaDB 10.11 d'un `UPDATE` joint à une dérivée de la même table est vérifiée en T0.
- **Pas de relèvement** de `kesh_version_min_required` : la 15-1a-i l'a porté à `0.13.0` (C101) et les
  crates sont en `0.13.0` ; motif exact (C115) : tout binaire publié depuis la v0.10.0 annulerait un
  règlement sans dissoudre son groupe. **Non-breaking** (P1 : `UPDATE` de colonnes nullables).
- **Pas d'audit** pour le rattrapage (une migration n'a pas d'acteur) ; le manuel le dit (AC12).

⛔ **Deux implémentations de la même règle** (SQL de M1/M2, Rust de P3) : la duplication que le
`CLAUDE.md` interdit, acceptée **parce que P8 fige la migration** — et **tenue par le test d'accord**
AC6, sans lequel elle ne serait pas acceptable.

**Ce que M1/M2 laissent, et pourquoi** : les pièces historiques entièrement en période close restent
ouvertes, non lettrables à la main (15-1a2-i P7 point 1) ; une facture hors exercice ouvert reste comme
elle était lue au grand livre avant la 15-1b.

## Critères d'acceptation

**AC6** — **Accord rattrapage ↔ synchronisation**, sur la fixture partagée (15-1a2-i T5) **étendue**
(paiements fournisseurs directs et par lot, compte interne, annulations de paiement et de facture
payée, contre-passation d'écriture manuelle, pièces entièrement sous la borne et sur exercice clos,
compte non lettrable) : (a) on relève les marques posées par les gestes vivants ; (b) on les efface en
SQL brut ; (c) on exécute M1 puis M2 ; (d) les marques sont **identiques** à (a), à **une** différence
nommée près — le groupe gardé après l'annulation d'un règlement de période verrouillée (15-1a2-i P7
point 2), que le rattrapage ne recrée pas (il calcule l'état présent) : la ligne du règlement annulé y
est appariée à son miroir si une des deux est en période ouverte, ouverte sinon ; (e) l'appel de
`sync_invoice_in_tx` / `sync_supplier_invoice_in_tx` sur **chaque** pièce rend `Unchanged`,
`AbstainedClosedPeriods` ou `AccountNotLetterable` — **jamais** une écriture —, et ne produit **aucune**
entrée d'audit.

**AC7** — Fournisseur : paiement (direct **et** par lot pain.001 confirmé, `confirm_batch`) → achat et
règlement lettrés `document` ; annulation du règlement → dissous, règlement ↔ miroir lettrés, achat
ouvert ; annulation d'une facture payée → dissous, achat ↔ miroir lettrés, **règlement détaché ouvert**
et lettrable à la main (`POST /letterings` l'accepte avec une ligne de même compte qui le solde) ;
paiement par **compte interne** lettrable puis annulation → la ligne de contrepartie et son miroir sont
aussi lettrés `reversal` ; compte `B` **non lettrable** → paiement réussi, aucun groupe, aucune erreur.

**AC8 (part ii)** — Test lexical (même fichier et mêmes outils que la part i) : chacun des trois
`UPDATE supplier_invoices … settlement_journal_entry_id` de production est dans une fonction dont le
corps appelle `sync_supplier_invoice_in_tx(` (après l'`UPDATE`, pour `pay_in_tx`) ou
`dissolve_supplier_invoice_document_group_in_tx(` (avant `reverse_owned_in_tx(`, pour les deux
annulations). Un site neuf rougit **en se nommant**.

**AC9 (part ii)** — `lettering_invariants` étendu : une ligne `document` d'une facture fournisseur est
sur son achat ou son règlement en vigueur ; ces lignes ne sont ni `manual` ni `reversal`, **sauf**
l'achat d'une facture annulée (`reversal`) et le règlement détaché (libre ou `manual`).

**AC10 (part ii)** — Audit : `documentType = "supplier_invoice"`, `documentId`, `documentNumber` ;
acteur : l'auteur du geste, `api_key_id: None` (écart nommé, comme la part i).

**AC11** — Outillage des migrations (finding R10 = F-13) :
- **P7** : M1 au registre (classe A, `include_str!` de la migration entière, commentaire de classe sur
  le patron des entrées existantes) ; M2 à `EXEMPT_MIGRATIONS` (`Durable`, justification ci-dessus),
  compteur `16 → 17` ; les cinq tests qui ne parcouraient que le registre (extraction verbatim,
  couverture des écritures, sentinelle B, absence de DDL, littéraux) **reprennent matière** — le
  commentaire « VIDE depuis la Story 25-2-c » de `POST_RESTORE_BACKFILLS` est réécrit ;
- **P5** : deux lignes au tableau de `docs/migrations-idempotence-audit.md`, **à leur place
  chronologique**, verdict `yes` (gardes `lettering_key IS NULL`) ; compteurs **recomptés depuis le
  tableau** : en-tête `## Table d'audit (78 migrations)` et ligne `Total` (`… + 2 Story 15-1a2-ii`),
  `yes` **10**, `tracked-by-sqlx` **68**, `no` 0 — et `ls crates/kesh-db/migrations/*.sql | wc -l` = 78 =
  `grep -c '^| \`20' docs/migrations-idempotence-audit.md` ;
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspection de chaque site —
  `migrations_upgrade_path.rs` porte `assert_eq!(total, 76)` et la frontière `total - 42` (figée à 34) :
  `total` passe à **78**, l'écart à **44** ; les montages par version (`common::migrations_before`) ne
  bougent pas ;
- **squash** `crates/kesh-db/test-schema/` régénéré **pour le suivi `_sqlx_migrations`** — aucune
  différence de structure attendue (`test_schema_guard.rs` compare structure, `_kesh_version` et suivi) ;
- `crates/kesh-db/migrations.sha384` : deux lignes ;
- le nouveau fichier de test à fenêtre (`lettering_documents_backfill.rs`, vrai `MIGRATOR`) est inscrit
  à `ALLOWED_REAL_MIGRATOR_FILES` (`test_schema_guard.rs`) ;
- **P8** : `20261009000001` inchangée (`git diff` nul sur le fichier).

**AC12 (part ii)** — Documentation, par la valeur :
- `CHANGELOG.md` : « une facture fournisseur payée est lettrée avec son achat ; à la mise à jour, les
  pièces déjà soldées et les contre-passations déjà passées sont lettrées (sauf en période close) — sans
  entrée au journal d'audit ».
- `docs/api-external.md` : `document` couvre aussi les factures fournisseurs ; le message
  `LETTERING_IS_DOCUMENT` (réécrit par la part i) reste juste.
- `docs/manual/fr/user-manual.tex` : § des factures fournisseurs (paiement, `sec:annuler-facture-fournisseur`)
  — lettrage, règlement détaché lettrable à la main ; glossaire *Lettrage* élargi aux factures
  fournisseurs ; une note sur le rattrapage (période close, pas d'audit).
- `docs/manual/fr/admin-manual.tex` § « Reprises de données rejouées à l'import » (`:1723`) : le lettrage
  des pièces est rejoué ; les paires de contre-passation libres **ne le sont pas** (motif, coût).
- `make fr`, trois PDF, contrôle aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`).
- `README.md` « Feuille de route » (ligne v0.13.0) : vérifier qu'elle ne contredit pas l'état livré.

**AC16** — **Rejeu après restauration** (finding R3 ; `admin_full_import_e2e.rs`) : (a) une sauvegarde
**sans** lettrage (marques à `NULL` dans l'archive) importée → les groupes `document` et les paires
`reversal` d'achats annulés sont posés, `backfills_replayed` du détail d'audit nomme M1
`REPLAYED_UNCONDITIONAL` ; les autres paires `reversal` restent **ouvertes** ; (b) une paire `reversal`
**libre** délettrée à la main (`DELETE /letterings`), sauvegardée puis réimportée → **reste délettrée** ;
(c) une base à jour réimportée → rejeu no-op (`rows_affected = 0` informatif, marques inchangées).

## Tasks

- [ ] **T0** — Relevés : faisabilité et `EXPLAIN` sur MariaDB 10.11 de la forme `UPDATE … JOIN (dérivée)`
      de M1/M2 sur une base de gate peuplée ; re-greper les ancres de P4 ; routes `Rejouee` :
      `POST /supplier-invoices/{id}/pay`, `…/cancel`, `…/settlement/cancel`, `POST /payment-batches/{id}/confirm`.
- [ ] **T1** (P3 part ii) — `sync_supplier_invoice_in_tx`, `dissolve_supplier_invoice_document_group_in_tx`,
      sur l'algorithme factorisé de la part i.
- [ ] **T2** (P4 part ii) — Les trois appels, exercices tenus gardés.
- [ ] **T3** (P6, AC11) — M1, M2, registre, exemption, squash, sha384, audit d'idempotence, P6, compteurs.
      Les en-têtes qui parlent de « la migration de rattrapage de la 15-1a2 » au singulier —
      `letterings.rs:15` et `letterings_lexical.rs:15` — nomment les deux fichiers.
- [ ] **T4** (AC8 part ii) — Test lexical fournisseur.
- [ ] **T5** — Tests (liste ci-dessous) ; extension de la fixture partagée.
- [ ] **T6** (AC12 part ii) — CHANGELOG, `api-external.md`, manuels FR (utilisateur, administrateur) +
      `make fr` + PDF aplati, README.

**Tests prévus** :
- `crates/kesh-db/tests/lettering_documents.rs` (fichier de la part i) — 6 neufs :
  `supplier_payment_letters_purchase_and_payment` (AC7), `batch_confirm_letters` (AC7),
  `supplier_settlement_cancel_dissolves_and_pairs` (AC7), `cancel_paid_supplier_invoice_detaches_an_open_payment` (AC7),
  `internal_account_payment_pairs_its_counterpart_on_cancel` (AC7), `payable_not_letterable_is_skipped` (AC7) ;
  plus `supplier_audit_details_carry_the_invoice` (AC10) — **7** ;
- `crates/kesh-db/tests/lettering_documents_backfill.rs` (neuf, vrai `MIGRATOR`, montage par version) — 3 :
  `backfill_matches_live_sync` (AC6), `backfill_abstains_on_closed_and_locked_history` (AC6),
  `backfill_pairs_reversals_by_position` (AC6) ;
- `crates/kesh-db/tests/letterings.rs` — `lettering_invariants` **étendu** (AC9 part ii) ;
- `crates/kesh-db/tests/letterings_lexical.rs` — 1 neuf : `supplier_settlement_writers_sync_or_dissolve` (AC8 part ii) ;
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — 3 neufs : `full_import_replays_document_lettering` (AC16 a),
  `full_import_keeps_a_dissolved_free_reversal_pair` (AC16 b), `full_import_of_an_up_to_date_base_is_a_noop` (AC16 c) ;
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — `upgrade_path_preserves_data` **modifié** (total 78,
  écart 44 ; AC11 P6).

*(Recompte depuis cette liste : 7 + 3 + 1 + 3 = **14 fonctions de test neuves**, **2 tests modifiés**
— `lettering_invariants`, `upgrade_path_preserves_data` —, plus les tests unitaires de `post_restore.rs`
qui reprennent matière sans être écrits.)*

## Dev Notes

- **Gate `kesh-db` complet** (migrations, `post_restore.rs`, repositories : ciblage interdit) ; base
  remise à zéro avant ; **démarrage réel** contre une base persistante (E2E) — seul révélateur d'un
  défaut de checksum (P8).
- **Verrous** : ceux de la part i ; l'ordre facture fournisseur `FOR UPDATE` → exercice → lignes est
  celui des trois gestes. `confirm_batch` règle N factures dans une transaction : N synchronisations,
  chacune après son `UPDATE` ; interblocage → rejeu de la route (`Rejouee`).
- **Tests existants touchés** (F-10, non exécutés) : `supplier_invoices_repository.rs:2511`
  (`supplier_invoice_cancel_letters_a_pair_that_cannot_be_dissolved_by_hand`, facture **non payée**) —
  inchangé ; `payment_batches_repository.rs` — les paiements de lot portent désormais des marques ;
  `admin_full_import_e2e.rs:1556` (liste attendue tirée de `POST_RESTORE_BACKFILLS`) — suit le registre ;
  `post_restore_class_a.rs` (rejoue registre + retirés sur base à jour) — M1 doit y être no-op.
- **Règle de découpage** : (1) le lettrage (`letterings.rs`), (2) les factures fournisseurs
  (`supplier_invoices.rs` ; `payment_batches.rs` **inchangé**, couvert par `pay_in_tx`), (3) les
  migrations (M1, M2, squash, sha384, audit), (4) le rejeu (`post_restore.rs`), (5) la documentation.
  **Cinq**, au seuil.
- **Dépendances** : la 15-1a2-i **mergée** (dure).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Fiche **née** de cette remédiation : découpage de la 15-1a2 (finding F-9 ; C-15-1a2-1). Findings de la
passe : **R** 3 HIGH, 7 MEDIUM, 4 LOW ; **F** 2 HIGH, 7 MEDIUM, 3 LOW — bilan par finding dans l'index.
Portés **ici** : R3/F-1 (classe du rattrapage : deux migrations, M1 classe A, M2 exemptée — P6, AC16),
R5/F-2 part ii (compte non lettrable : `B` et SQL — P3, P6, AC7), R2/F-6 part ii (abstention au rattrapage
— P6, AC6), R4 (Reçus 1, 3, 7, 10, 13 intégrés à P6), R10/F-13 (outillage : registre, exemption,
squash pour le suivi, `ALLOWED_REAL_MIGRATOR_FILES`, compteurs P5 76 → **78**, P6 — AC11), R11 part ii
(lexical fournisseur — AC8), F-10 part ii (`confirm_batch`, compte interne, tests existants — AC7, Dev
Notes), R9/F-7 part ii (manuels, rattrapage — AC12). Choix consignés : C-15-1a2-1 à -4 (registre).
**8 critères** (AC6, AC7, AC8–AC10 parts ii, AC11, AC12 part ii, AC16), **7 tâches** (T0–T6), **14 tests neufs + 2 modifiés** —
recomptés depuis ce fichier. Prochaine passe : P2, Opus, complète.
