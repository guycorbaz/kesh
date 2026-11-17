# Epic 43 — Une transaction, plusieurs factures (#823)

*Kickoff du 2026-11-17, rédigé par l'agent de rédaction de la fiche (Opus 5.5), worktree `kesh-43`, branche
`story/43-transaction-plusieurs-factures` partie de `epic/43-transaction-plusieurs-factures` (= `main` = v0.18.0,
`8b50fd195`). Release **0.19.0**, publiée le vendredi 2026-11-20, coupure le jeudi 2026-11-19 au soir. Rang 1 de la
release : si le temps manque, ce sont d'autres stories qui sortent (décision de Guy du 2026-11-17).*

## Objet

Un mouvement du relevé bancaire règle **plusieurs factures** :

- **fournisseurs** : un débit paie plusieurs factures fournisseur validées (ouvertes) du même fournisseur ;
- **clients** : un crédit encaisse plusieurs factures clients du même client.

À la réconciliation, l'utilisateur choisit un tiers, puis plusieurs factures ouvertes dont la **somme des restes dus égale
le montant** de la transaction ; Kesh passe **un** règlement et lettre **chaque** facture (Guy, commentaire de #823 du
2026-11-13). Le virement entre comptes propres est déjà traité (0.17.6, 34-11b) : hors de l'epic.

## Décisions (C-43-k) — code lu à `8b50fd195`

**C-43-1 — Une seule écriture de règlement par transaction : une ligne de banque, une ligne de tiers par facture.**
La transaction continue de pointer **une** écriture (`bank_transactions.matched_entry_id`,
`20260504000001_bank_imports.sql:71`). Tout ce qui lit ce lien reste juste sans changement : le motif
`MATCHED_BANK_TRANSACTION` (`journal_entries.rs:2343-2344`), l'écart de relevé (`bank_accounts.rs:1884-1898`), la
lecture du dé-rapprochement (`reconciliation_cancel.rs:203`), la vue de la transaction, l'export. Le relevé montre un
mouvement, le grand livre de la banque une ligne.
*Écartées* : (a) N écritures et une **table** de liens transaction → écritures — une table neuve rend toute sauvegarde
0.18.0 inimportable dans la 0.19.0 (`admin_backup/import.rs:250-253` : chaque table de l'inventaire doit figurer au
manifeste), et une quinzaine de lectures de `matched_entry_id` seraient à doubler ; (b) des transactions « filles » —
chaque somme du relevé compterait deux fois.

**C-43-2 — Chaque facture sait quelle ligne de l'écriture est la sienne.** Deux colonnes facultatives :
`invoice_settlements.journal_entry_line_id` et `supplier_invoices.settlement_journal_entry_line_id`, clé étrangère vers
`journal_entry_lines(id)`, uniques. `NULL` garde la règle d'avant : toutes les lignes du compte de tiers de l'écriture
(un règlement unitaire). *Pourquoi* : la découverte du lettrage de pièce lit **toutes** les lignes du compte de tiers de
chaque écriture de règlement (`letterings.rs`, `SYNC_SETTLEMENT_ENTRIES_SQL` :1477, `SYNC_DOCUMENT_LINES_SQL` :1497,
`discover_invoice_document` :1536 ; de même `discover_supplier_invoice_document`, :1901) ; sur une écriture commune, elle mêlerait les factures, et
`existing_document_group` (:1622) verrait la clé d'une autre facture.

**C-43-3 — La contrainte d'unicité change de forme, elle ne disparaît pas.** `uq_invoice_settlements_entry
(journal_entry_id)` (`20260827000001_invoice_settlements.sql:65`) est remplacée par `uq_invoice_settlements_entry_invoice
(journal_entry_id, invoice_id)` — un règlement par facture et par écriture ; elle porte aussi la clé étrangère
`fk_invoice_settlements_entry` (colonne de tête). Côté fournisseur, `settlement_journal_entry_id` n'a jamais été unique
(`20260628000001_supplier_invoices.sql:47`) ; s'ajoute `chk_supplier_invoices_settlement_line` (ligne `NULL` dès que
l'écriture l'est).

**C-43-4 — Migration non breaking : `20261118000001_grouped_settlements.sql`.** `ADD COLUMN` facultatifs, index et clés
ajoutés, l'ancien index unique retiré (`DROP INDEX` n'est pas dans la liste de P3). Aucun binaire 0.18.0 ne peut lire
l'état nouveau : il refuse de démarrer sur une base qui porte une migration inconnue, et il refuse un `.keshbackup` 0.19.0
au contrôle des colonnes (`import.rs:613-620`, (c1) `unknownColumns`). Une sauvegarde 0.18.0 reste importable dans la
0.19.0 (colonnes nouvelles facultatives, (c2) `import.rs:622-628`). `kesh_version_min_required` reste à **0.17.6** ; aucun bump Cargo
dans les stories (la release passe à 0.19.0). **Retour à la 0.18.0** : uniquement par la sauvegarde d'avant la mise à jour
(base, `.keshbackup`, dossier des documents) — Kesh écrit la sienne dans `backup/pre-upgrade/` avant de migrer. À dire en
tête des notes de version et au manuel d'administration.

**C-43-5 — DDL seul.** Aucun `UPDATE`/`INSERT` : rien à trier au registre de rejeu (P7) ; ligne d'audit d'idempotence
(P5, totaux recomptés : 92) ; inventaire P6 ; squash `crates/kesh-db/test-schema/0001_schema_squash.sql` ; P8 sans objet.
Le rattrapage de lettrage rejoué à chaque import (`20261010000001_lettering_documents_backfill.sql`, classe A) **s'abstient**
sur une écriture commune : pour la facture `I`, la somme de son groupe vaut `−Σ(restes dus des autres factures) ≠ 0`, et
les lignes y sont déjà lettrées (`t.deja > 0`). À prouver par une fixture du test d'accord.

**C-43-6 — Un règlement groupé ne se défait que par le dé-rapprochement de sa transaction, en entier.** Il naît rapproché,
dans la même transaction SQL. Tout geste unitaire bute donc sur la garde existante : annulation d'un règlement client
(rang 3, `MATCHED_BANK_TRANSACTION`, refusé par la contre-passation), retour en brouillon d'une facture fournisseur payée
(`PaidReconciled`, `supplier_invoices.rs:1194-1209`). Seule l'**annulation d'une facture fournisseur payée**
(`cancel_in_tx`, :2530) passe, comme aujourd'hui : elle détache le règlement (colonnes `settlement_*`, ligne comprise, à
`NULL`) ; au dé-rapprochement, l'écriture entière est contre-passée — c'est déjà le sort d'un règlement détaché
(`reconciliation_cancel.rs:39-50`). Ordre du dé-rapprochement groupé : lien d'abord (`matched_entry_id` à `NULL`),
délettrage de chaque pièce, retrait des règlements (lignes `invoice_settlements` supprimées, factures fournisseur rouvertes),
puis contre-passation de l'écriture **sans propriétaire** (`journal_entries::reverse_in_tx`, aucune autorité neuve :
`journal_entries.rs` n'est pas touché), projections de `paid_at`, audit.

**C-43-7 — Règles d'acceptation.** Même tiers pour toutes les factures ; 2 à 50 factures distinctes ; somme **exacte**
au centime (client : `Σ amount_due_to_centime(reste dû) = montant`, chaque règlement écrit au reste brut, l'écart de
fractions de centime sur le compte d'arrondi, comme `settlement_journal_lines`, `invoice_settlements.rs:237` ;
fournisseur : `Σ total_amount = |montant|`, comme l'acceptation unitaire, `routes/reconciliation.rs:3406`) ; sens (un
crédit pour des factures clients, un débit pour des factures fournisseur) ; chaque facture passe les contrôles
d'éligibilité de l'acceptation unitaire (statut, compte de créance qui n'est pas la banque, facture fournisseur hors lot
`generated`, compte bancaire ouvert, période). **Ni score ni fenêtre de 30 jours** : le choix est explicite, comme le
rattachement à une écriture (v017-3a). *Pattern batch* : une proposition par transaction, refus dans `failed[]`,
`details.invoiceId` pour la facture en cause.

**C-43-8 — API.** Deux types de proposition dans `POST /reconciliation/accept` : `invoice_group`
(`invoiceIds`) et `supplier_invoice_group` (`supplierInvoiceIds`) ; une lecture des candidates
`GET /reconciliation/group-candidates?bankTransactionId=…&contactId=…` ; la vue et l'annulation d'un rapprochement
rendent `kind = "grouped_settlement"` et la liste des factures. Contrat dans la fiche 43-2.

**C-43-9 — Audit.** `reconciliation.accepted` une fois (`proposalType`, liste des factures et montants), puis
`invoice.settled` par facture client ou `supplier_invoice.paid` par facture fournisseur ; au dé-rapprochement,
`reconciliation.cancelled` puis une entrée d'annulation par facture. Acteur : l'utilisateur, ou la clé d'API (`Acteur`).

**C-43-10 — Hors périmètre : le lot pain.001 *confirmé*.** Une facture d'un lot `generated` est refusée au règlement
bancaire (`InGeneratedBatch`, `supplier_invoices.rs:2388-2402`) ; la confirmation d'un lot passe **une écriture par
facture** (`payment_batches.rs:574-590`). Le débit unique d'un lot confirmé reste donc sans rattachement possible (il
faudrait rattacher une transaction à plusieurs écritures déjà passées, ou confirmer le lot en une écriture). Question Q1 à
Guy ; issue à ouvrir si le cas reste.

**C-43-11 — Réimputation analytique (Epic 42) sur une écriture commune : la ligne de la facture seulement.** Le geste de
la 42-1 lit toutes les écritures de règlement de la facture (`project_reassignment.rs:443-453`, branche
`origin/epic/42-reimputation-analytique`) et réimpute **toutes leurs lignes** (`reassign_lines_in_tx`, :352-363). Sur une
écriture commune, il réimputerait la ligne de banque et celles des autres factures, sans le dire. Règle : sur une écriture
dont un règlement porte un identifiant de ligne, le geste ne réimpute que **la** ligne de tiers de la facture — jamais la
ligne de banque (laissée sans compte analytique, comme toute ligne de règlement à sa création,
`invoice_settlements.rs:246-255`) ni celles des autres factures. *Ce que cela impose* : 42-1 (faite) et 42-2 lisent leurs
écritures de règlement par **une seule** fonction de `project_reassignment.rs`, et 42-2 (fournisseur) en fait autant ;
le **raccord** (tâche T-R de 43-1) se joue dès que les deux branches sont réunies, avant la coupure, avec son test.

**C-43-12 — Découpage en trois stories et une story de solde** (règle de splitting : plus de cinq modules) :

| Story | Objet | Dépend de | Parallèle avec |
|---|---|---|---|
| 43-1 | modèle, migration, dépôt (règlement groupé client et fournisseur, lettrage, dé-rapprochement), raccord Epic 42 | — | 43-3 dès le départ |
| 43-2 | API : acceptation groupée, candidates, vue et annulation, audit, `docs/api-external.md` | signatures de 43-1 (commit T3) | 43-3 |
| 43-3 | écran « Régler plusieurs factures », dialogue d'annulation, i18n, manuel utilisateur, E2E | contrat de 43-2 (Vitest sur maquettes) ; E2E après 43-2 | 43-1, 43-2 |
| 43-4 | solde de la dette de l'epic | fin d'epic | — |

**C-43-13 — Même tiers, même pour les clients.** Le faux rattachement est muet (`CLAUDE.md`, § *Un appariement
automatique propose…*) : un groupe mêlant deux tiers est refusé. Question Q2 à Guy si un paiement pour plusieurs tiers
est un cas réel.

## Questions à Guy

- **Q1** — Le débit d'un **lot pain.001 confirmé** (N écritures déjà passées) : à traiter dans une prochaine release
  (rattacher une transaction à plusieurs écritures, ou confirmer un lot en une écriture) ? #823 se ferme-t-elle avec la
  0.19.0 (cas « factures ouvertes ») avec une issue neuve pour le lot confirmé, ou reste-t-elle ouverte ?
- **Q2** — Un même virement pour plusieurs tiers (deux sociétés d'un groupe) : à refuser, comme proposé ?

## Décomptes au kickoff

| Mesure | Au 2026-11-17 (prompt d'appel) | Recompté le 2026-11-17 par l'agent |
|---|---|---|
| Issues ouvertes | 458 | **461** (`gh issue list --state open --limit 700 --json number --jq length`) |
| Bugs (`bug` ou `known-failure`) | 236 | **237** (commande de § *Priorités des défauts*) |
| P1 | 14 | **14** |

Au tag v0.18.0 : à reprendre du fichier de release (non recompté ici). **Plafond de la story de solde (43-4) : 2 issues.**
Objectif de la règle de release (moins d'issues qu'au tag précédent) : objectif, non barrière.

## Fichiers communs (déclarés au kickoff)

| Fichier ou module | Stories | Croise |
|---|---|---|
| `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (neuf) | 43-1 | multi-devise (Beat) : horodatage postérieur |
| `crates/kesh-db/test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md` | 43-1 | toute migration (compteurs recomptés) |
| `crates/kesh-db/src/entities/{invoice_settlement,supplier_invoice}.rs` | 43-1 | multi-devise |
| `crates/kesh-db/src/repositories/grouped_settlements.rs` (neuf) | 43-1 | — |
| `crates/kesh-db/src/repositories/{letterings,reconciliation_cancel,invoice_settlements,supplier_invoices}.rs` | 43-1 | multi-devise ; 42-2 (`supplier_invoices.rs`) |
| `crates/kesh-db/src/repositories/project_reassignment.rs` (branche Epic 42) | 43-1 (raccord T-R) | **Epic 42** |
| `crates/kesh-api/src/routes/reconciliation.rs`, `routes/reconciliation_group.rs` (neuf), `audit_labels.rs`, `errors.rs`, `exports/csv_tables.rs` | 43-1 (export), 43-2 | 42-2, 42-3 (`audit_labels.rs`, `errors.rs`) |
| `frontend/src/lib/features/reconciliation/` | 43-3 | — |
| `frontend/src/lib/components/invoices/ContactPicker.svelte` | 43-3 l'**emploie** sans le modifier | correction #1162 le modifie |
| catalogues i18n (quatre locales) | 43-2 (erreurs), 43-3 | tous : bloc par epic |
| `docs/manual/fr/user-manual.tex`, `admin-manual.tex` (`.tex` seulement) | 43-1 (admin), 43-3 (utilisateur) | 42, 44 |
| `docs/api-external.md` | 43-2 | 42 |

## Pour les travaux parallèles (multi-devise)

1. Un règlement peut désormais être **commun à plusieurs factures** : **une** écriture (une ligne de banque, une ligne de tiers par facture), pointée par **une** transaction (`matched_entry_id` inchangé).
2. `invoice_settlements` : N lignes peuvent porter le même `journal_entry_id` ; nouvelle colonne `journal_entry_line_id` (la ligne de tiers de la facture ; `NULL` = règlement unitaire d'avant) ; unicité `(journal_entry_id, invoice_id)` au lieu de `(journal_entry_id)`.
3. `supplier_invoices` : N factures peuvent porter le même `settlement_journal_entry_id` ; nouvelle colonne `settlement_journal_entry_line_id`.
4. Toute règle de devise sur un règlement (montant en devise, cours, écart de change) doit s'appliquer **par ligne de tiers**, et la ligne de banque porter la somme.
5. Migration : `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (la 92ᵉ), DDL seul, non breaking ; une migration du multi-devise prend un horodatage postérieur et part de `main` après la 0.19.0.
6. Code neuf : `crates/kesh-db/src/repositories/grouped_settlements.rs` ; touchés : `letterings.rs` (découverte par ligne), `reconciliation_cancel.rs` (dé-rapprochement groupé), `routes/reconciliation.rs` et `routes/reconciliation_group.rs`, `frontend/src/lib/features/reconciliation/`.
7. Recommandation : ne pas modifier ces fichiers avant la publication de la 0.19.0 ; prévenir avant sinon.
