# Epic 43 — Une transaction, plusieurs factures (#823)

*Kickoff du 2026-11-17 (agent de rédaction de l'Epic 43, Opus 5.5), worktree `kesh-43`, branche
`story/43-transaction-plusieurs-factures` partie de `epic/43-transaction-plusieurs-factures` (= `main` = v0.18.0,
`8b50fd195`). **Révisé le 2026-11-17 au soir** : décisions de Guy du même jour et validation P1 de la 43-1
(`kesh-gate-logs/e43/validation-43-1-p1.md`, 0/3/7/8). Release **0.19.0** (vendredi 2026-11-20, coupure jeudi 19 au soir),
rang 1 ; chaque story doit pouvoir **sortir seule** à la coupure.*

## Objet

Au rapprochement, on choisit une ligne du relevé, puis on y **ajoute des factures une à une** ; un total montre la
**différence** entre le montant du relevé et les factures imputées ; le geste s'enregistre (« comme bexio », Guy,
2026-11-17). Un **crédit** règle des factures **clients**, un **débit** des factures **fournisseur** — jamais les deux ; les
factures peuvent être de **tiers différents**. Kesh passe **un** règlement, solde et lettre **chaque** facture. Le virement
entre comptes propres est déjà traité (0.17.6, 34-11b).

## Décisions (C-43-k) — code lu à `8b50fd195`, et à `origin/epic/42-reimputation-analytique` pour l'Epic 42

**C-43-1 — Une écriture par geste : une ligne de banque, une ligne de tiers par facture impayée, une ligne d'écart s'il y
en a une.** La transaction pointe **cette** écriture (`bank_transactions.matched_entry_id`,
`20260504000001_bank_imports.sql:71`). Tout ce qui lit ce lien reste juste sans changement (`journal_entries.rs:2343-2344`,
`bank_accounts.rs:1884-1898`, `reconciliation_cancel.rs:203`). *Écartées* : N écritures et une **table** de liens (une table
neuve rend toute sauvegarde 0.18.0 inimportable, `admin_backup/import.rs:250-253`) ; des transactions « filles ».

**C-43-2 — Chaque facture sait quelle ligne est la sienne** : `invoice_settlements.journal_entry_line_id`,
`supplier_invoices.settlement_journal_entry_line_id`, facultatives, uniques ; `NULL` = règlement unitaire, règle d'avant.
Sans elles, la découverte du lettrage (`letterings.rs:1477`, `:1497`, `:1536`, `:1901`) mêlerait les factures.

**C-43-3 — L'unicité change de forme** : `uq_invoice_settlements_entry (journal_entry_id)`
(`20260827000001_invoice_settlements.sql:65`) → `uq_invoice_settlements_entry_invoice (journal_entry_id, invoice_id)` ;
côté fournisseur, `CHECK` (ligne ⇒ écriture).

**C-43-4 — Une seule migration, non breaking : `20261118000001_grouped_settlements.sql`** (DDL seul) — les deux colonnes
de C-43-2, la clé de C-43-3, et la colonne de C-43-17. La 0.18.0 refuse de démarrer sur la base migrée et refuse **toute**
sauvegarde 0.19.0, même sans groupe (colonnes exportées, `import.rs:613-620`) ; une sauvegarde 0.18.0 s'importe dans la
0.19.0 (colonnes facultatives, `import.rs:622-628`). Un binaire 0.18.0 qui démarrerait classerait mal un groupe (son
`classify` suppose l'unicité retirée) : c'est le refus de démarrer qui protège, et `kesh_version_min_required` reste
**0.17.6**. **Retour à la 0.18.0 : par la sauvegarde d'avant la mise à jour seulement.** La migration est entière dans la
43-1a, **même si la 43-1b ou la 43-5 sortent** : leurs colonnes restent alors à `NULL`, sans effet.

**C-43-5 — DDL seul** : P7 sans entrée (`every_data_backfill_migration_is_triaged`) ; P5 (totaux à 92) ; P6
(`migrations_upgrade_path.rs:133-135` : 91 → 92, et `total - 57` → `total - 58`, :228-231) ; squash `test-schema`.
*Corrigé (M3)* : le rattrapage de lettrage `20261010000001` **ne se rejoue plus** à l'import — il est dans
`RETIRED_BACKFILLS` (`post_restore.rs:213-216`) ; reste le test d'accord de sa matière, qui gagne une fixture de groupe.

**C-43-6 — Un règlement groupé ne se défait que par le dé-rapprochement de sa transaction, en entier**, dans l'ordre de
verrous du geste unitaire (M1). Les gestes unitaires sur une facture du groupe butent sur les gardes existantes
(`MATCHED_BANK_TRANSACTION` ; `PaidReconciled`, `supplier_invoices.rs:1194-1209`) ; seule l'annulation d'une facture
fournisseur payée détache (`supplier_invoices.rs:2639`), comme aujourd'hui.

**C-43-7 — Admises dans un groupe** : factures du sens de la transaction, d'un ou plusieurs tiers, **deux au moins**, 50 au
plus. Story de tête : factures **impayées** (client `validated`, reste dû > 0 ; fournisseur `open`, hors lot `generated`) ;
43-5 : en plus, les factures **payées dont le paiement n'est pas rapproché** (C-43-17). Ni score ni fenêtre de 30 jours
(choix explicite). Chaque facture passe les refus de l'acceptation unitaire, énumérés dans `GroupRefusal` (*Pattern
batch* : aucun ne sort en `AppError`).

**C-43-8 — API** : `invoice_group` (`invoiceIds`), `supplier_invoice_group` (`supplierInvoiceIds`) ; recherche des
candidates **par texte**, tous tiers (`GET /reconciliation/group-candidates?bankTransactionId=…&q=…`) ; `kind =
"grouped_settlement"`. Contrat : fiche 43-2.

**C-43-9 — Audit (H1)** : le dépôt écrit **lui-même** `reconciliation.accepted` en premier, puis, par facture, l'action de
l'unitaire — `invoice.paid` (avec `reconciliation_audit_id`, `paid_via`, `settlement_journal_entry_id`, avant/après) côté
client, `supplier_invoice.paid` côté fournisseur — avec l'**acteur** (clé d'API comprise). La route n'écrit rien. Aucune
action neuve.

**C-43-10 — Le lot pain.001 confirmé** n'est plus hors périmètre : c'est la **43-5** (C-43-17), coupable.

**C-43-11 — Réimputation analytique (Epic 42) sur une écriture commune** *(récrit, H2)*. Un règlement unitaire porte le
projet de sa facture sur **toutes** ses lignes (en-tête `project_id: invoice.project_id`, `routes/reconciliation.rs:2820` ;
`supplier_invoices.rs:2163` ; héritage `line.project_id.or(new.project_id)`, `journal_entries.rs:521`). L'écriture groupée
porte le projet de chaque facture sur **sa** ligne de tiers ; banque et écart : sans projet (C-43-20). Le geste de la 42-1
(`project_reassignment.rs:443-453`, `reassign_lines_in_tx` :352-363, par écriture) réimputerait toute l'écriture : sur une
écriture commune, il ne doit réimputer que la ligne de la facture. Porté par la **43-6** (raccord), en dernier ; **repli
approuvé par Guy** si le temps manque : le geste **refuse** une facture réglée en groupe (garde courte, prête mercredi).

**C-43-12 — Découpage** (H3 retenu, adapté) : § *Stories*.

**C-43-13** — *Tombée* (Guy, 2026-11-17) : des tiers différents sont admis. Le faux rattachement reste évité par le choix
explicite, facture par facture, sans présélection automatique.

**C-43-14 — Interface « comme bexio »** : ligne du relevé → ajout de factures une à une par recherche → total et
différence → *Enregistrer*. Fiche 43-3.

**C-43-15 — Pas de mélange** clients / fournisseurs : le sens de la transaction choisit la famille.

**C-43-16 — La différence** : `d = montant − Σ apports`. Apport : reste dû brut (client), `total_amount` (fournisseur),
montant du paiement existant (43-5). `d = 0` : pas de ligne d'écart. `0 < |d| ≤ 5.00` : une ligne sur le **compte de
différences d'arrondi** de l'unitaire (`company_invoice_settings::rounding_account_for_write`,
`company_invoice_settings.rs:628`), au crédit si `d > 0`, au débit sinon, sans projet ; refus de l'unitaire réemployés
(`ROUNDING_ACCOUNT_NOT_CONFIGURED`, compte d'arrondi égal au compte de tiers ou de banque). `|d| > 5.00` : refus
`RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`. *Pourquoi 5.00* : l'unitaire n'a pas de compte d'écart, seulement l'arrondi
sous le centime et une tolérance de 0.05 pour la proposition (`AMOUNT_TOLERANCE_HUNDREDTHS`, `routes/reconciliation.rs:86`) ;
Guy attend « de petites différences » (frais bancaires) ; 5.00 les couvre et l'écran l'affiche. **Valeur à confirmer par
Guy (Q4)** ; constante unique `GROUP_DIFFERENCE_MAX` dans `grouped_settlements.rs`.

**C-43-17 — Factures payées non rapprochées (43-5), sans table neuve** : colonne facultative
`journal_entry_lines.bank_transaction_id` (clé étrangère `ON DELETE RESTRICT`, index), posée sur la **ligne de banque**
d'une écriture de paiement **existante** pour nommer la transaction qui la rapproche. La transaction garde **un**
`matched_entry_id` : l'écriture groupée si le geste en passe une (factures impayées ou écart), sinon le plus petit
identifiant des écritures existantes ; **toutes** les écritures existantes du geste portent la colonne sur leur ligne de
banque, celle que pointe `matched_entry_id` comprise — c'est ce qui permet au dé-rapprochement de les reconnaître (une
écriture existante pointée sans la colonne reste un rattachement unitaire, défait comme aujourd'hui). Un « paiement
existant » est l'écriture d'un règlement client par virement ou d'une facture fournisseur payée, sur le compte de la
banque de la transaction, que rien ne rapproche encore ; son apport est le montant de sa ligne de banque. Le
dé-rapprochement **retire ces liens sans contre-passer** ces paiements (le geste ne les a pas créés ; contre-passer les
trente paiements d'un lot serait destructeur), à la différence du rattachement unitaire à une écriture (v017-3a), qui les
défait : écrit au manuel, question Q5. La 43-5 met à jour les lecteurs qui disent « cette écriture est rapprochée »
(inventaire dans sa fiche).

**C-43-18 — Ordre de sortie à la coupure** (le premier nommé sort le premier) : **43-5** ; puis la **43-6 complète**,
remplacée par son **repli**, qui, lui, **ne sort pas** si 42-1 et 43-1a sont publiées ensemble ; puis **43-1b** (avec les
tâches fournisseur de 43-2 et 43-3, livrées en commits séparés, retirables) ; puis **43-3** (l'API seule reste utile à la
clé d'API de Guy) ; puis **43-2**. La **43-1a** n'est publiée qu'avec la 43-2 ; sans elle, l'epic entier sort, migration
comprise.

**C-43-19 — Verrous et versions (M1, M2)** : l'ordre de l'unitaire partout ; `AND version = ?` et `rows_affected() == 1`
sur chaque facture et sur la transaction ; `ensure_open_in_tx` puis `recheck_open_locked_in_tx` du compte bancaire en
dernière acquisition (`reconciliation_cancel.rs:631`, `:784`).

**C-43-20 — Projet par ligne (H2)** : la ligne de tiers de chaque facture porte le `project_id` de sa facture, l'en-tête
aucun ; un projet non imputable (archivé, inexistant) est un refus de **la** facture (`GroupRefusal::ProjectNotTaggable`).

## Stories

| Story | Objet | Part de | Parallèle |
|---|---|---|---|
| **43-1a** | migration entière, entités, export, lettrage par ligne (deux familles), règlement groupé **client** avec écart, dé-rapprochement groupé client, `document_owners`, manuel d'administration | `epic/43` | — |
| **43-1b** | règlement groupé **fournisseur**, dé-rapprochement fournisseur, détachement | 43-1a au commit T3 | 43-2, 43-3 |
| **43-2** | API (deux types, recherche, vue), `docs/api-external.md` | 43-1a au commit T3 | 43-1b, 43-3 |
| **43-3** | écran façon bexio, annulation, i18n, manuel utilisateur, E2E | `epic/43` (Vitest sur contrat) ; E2E sur la branche réunie | tout |
| **43-5** | factures payées non rapprochées (lot pain.001 confirmé, « Payer » à la main) | 43-1a (+ 43-1b pour le fournisseur) | après 43-1b |
| **43-6** | raccord Epic 42 (ou son repli) | branche réunie 42-1 + 43-1a | en dernier |
| **43-4** | solde de la dette (plafond 2 issues) | fin d'epic | — |

Fiches : `43-1a-reglement-groupe-modele-client.md`, `43-1b-reglement-groupe-fournisseur.md`, `43-2-reglement-groupe-api.md`,
`43-3-reglement-groupe-ecran.md`, `43-5-paiements-non-rapproches.md`, `43-6-raccord-reimputation.md`. La fiche
`43-1-reglement-groupe-modele.md` est `split` (validée en P1, puis découpée).

**Jalons** : commit T3 de la 43-1a (migration et signatures) **mercredi 18 avant midi** ; 43-1a **en revue mercredi 18 à
18 h** (développement fini, `test-fast --ci` et `gate-ci` verts) ; repli de la 43-6 prêt mercredi soir ; 43-1a et 43-2
closes jeudi midi ; le reste jeudi soir.

## Questions à Guy

- **Q4** — Plafond de l'écart : 5.00 CHF (C-43-16) ?
- **Q5** — Dé-rapprocher un geste qui a rattaché des paiements existants : retirer le lien seulement (C-43-17) ?
- **Q1** — #823 se ferme-t-elle à la release si la 43-5 sort ? Proposé : `closes` si la 43-5 est publiée, `refs` sinon.

## Décomptes au kickoff

| Mesure | Prompt d'appel | Recompté le 2026-11-17 |
|---|---|---|
| Issues ouvertes | 458 | **461** |
| Bugs | 236 | **237** |
| P1 | 14 | **14** |

Plafond de la story de solde (43-4) : **2 issues**.

## Fichiers communs (déclarés au kickoff)

| Fichier ou module | Stories | Croise |
|---|---|---|
| `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (neuf), `test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md`, `tests/migrations_upgrade_path.rs` | 43-1a | multi-devise ; toute migration |
| `crates/kesh-db/src/entities/{invoice_settlement,supplier_invoice,journal_entry}.rs` | 43-1a | multi-devise |
| `crates/kesh-db/src/repositories/grouped_settlements.rs` (neuf) | 43-1a, 43-1b, 43-5 | — |
| `repositories/{letterings,reconciliation_cancel,invoice_settlements,supplier_invoices,journal_entries}.rs` | 43-1a, 43-1b, 43-5 | multi-devise ; 42-2 |
| `repositories/{bank_accounts,invoices,settlement_cancellation,reconciliation_link}.rs` | 43-5 | — |
| `repositories/project_reassignment.rs` (branche Epic 42) | 43-6 | **Epic 42** |
| `crates/kesh-api/src/routes/reconciliation.rs`, `routes/reconciliation_group.rs` (neuf), `lib.rs`, `audit_labels.rs`, `exports/csv_tables.rs` | 43-1a (export), 43-2 | 42-2, 42-3 |
| `frontend/src/lib/features/reconciliation/` | 43-3 | — |
| catalogues i18n (quatre locales) | 43-2, 43-3 | tous (bloc par epic) |
| `docs/manual/fr/admin-manual.tex` (43-1a), `user-manual.tex` (43-3, 43-5) — `.tex` seulement | | 42, 44 |
| `docs/api-external.md` | 43-2 | 42 |

## Pour les travaux parallèles (multi-devise)

1. Un règlement peut être **commun à plusieurs factures**, de tiers différents : **une** écriture — une ligne de banque, une ligne de tiers par facture (portant le projet de sa facture), une ligne d'écart éventuelle (compte d'arrondi, au plus 5.00) — pointée par **une** transaction (`matched_entry_id`).
2. `invoice_settlements` : N lignes pour le même `journal_entry_id` ; colonne neuve `journal_entry_line_id` (`NULL` = règlement unitaire) ; unicité `(journal_entry_id, invoice_id)` au lieu de `(journal_entry_id)`.
3. `supplier_invoices` : N factures pour le même `settlement_journal_entry_id` ; colonne neuve `settlement_journal_entry_line_id`.
4. `journal_entry_lines` : colonne neuve `bank_transaction_id` — la ligne de banque d'un paiement **existant** nomme la transaction qui la rapproche, quand un geste groupé rattache des paiements déjà passés (43-5, peut sortir de la 0.19.0 : la colonne reste alors vide).
5. Toute règle de devise (montant en devise, cours, écart de change) s'applique **par ligne de tiers** ; l'écart de change ne se confond pas avec la ligne d'écart du groupe.
6. Migration : `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (la 92ᵉ), DDL seul, non breaking ; celle du multi-devise prend un horodatage postérieur et part de `main` après la 0.19.0.
7. Code : `repositories/grouped_settlements.rs` (neuf), `letterings.rs`, `reconciliation_cancel.rs`, `routes/reconciliation_group.rs` (neuf), `frontend/src/lib/features/reconciliation/`. Ne pas les modifier avant la publication de la 0.19.0 sans prévenir.
