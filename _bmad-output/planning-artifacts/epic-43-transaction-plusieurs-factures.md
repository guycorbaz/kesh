# Epic 43 — Une transaction, plusieurs factures (#823)

*Kickoff du 2026-11-17 (agent de rédaction de l'Epic 43, Opus 5.5), worktree `kesh-43`, branche
`story/43-transaction-plusieurs-factures` partie de `epic/43-transaction-plusieurs-factures` (= `main` = v0.18.0,
`8b50fd195`). Révisé le 2026-11-17 au soir (décisions de Guy ; validation P1 de la 43-1), puis **dans la nuit** :
validation P2 de la 43-1a (`kesh-gate-logs/e43/validation-43-1a-p2.md`), validations P1 de la 43-2 et de la 43-3, décision
de Guy sur Q4 (plafond configurable), décisions de l'orchestrateur (42-2 et 42-3 hors 0.19.0 ; migration propre à la 43-5 ;
code du refus de réimputation). Release **0.19.0** (vendredi 2026-11-20, coupure jeudi 19 au soir), rang 1 ; chaque story
doit pouvoir **sortir seule** à la coupure.*

## Objet

Au rapprochement, on choisit une ligne du relevé, puis on y **ajoute des factures une à une** ; un total montre la
**différence** entre le montant du relevé et les factures imputées ; le geste s'enregistre (« comme bexio », Guy,
2026-11-17). Un **crédit** règle des factures **clients**, un **débit** des factures **fournisseur** — jamais les deux ; les
factures peuvent être de **tiers différents**. Kesh passe **un** règlement, solde et lettre **chaque** facture. Le virement
entre comptes propres est déjà traité (0.17.6, 34-11b).

## Décisions (C-43-k) — code lu à `8b50fd195`, et à `origin/epic/42-reimputation-analytique` pour l'Epic 42

**C-43-1 — Une écriture par geste** : une ligne de banque, une ligne de tiers par facture, et au plus deux lignes de
différence (arrondi, écart — C-43-16, C-43-25). La transaction pointe **cette** écriture (`bank_transactions.matched_entry_id`,
`20260504000001_bank_imports.sql:71`). Tout ce qui lit ce lien reste juste sans changement (`journal_entries.rs:2343-2344`,
`bank_accounts.rs:1884-1898`, `reconciliation_cancel.rs:203`). *Écartées* : N écritures et une **table** de liens (une table
neuve rend toute sauvegarde 0.18.0 inimportable, `admin_backup/import.rs:250-253`) ; des transactions « filles ».

**C-43-2 — Chaque facture sait quelle ligne est la sienne** : `invoice_settlements.journal_entry_line_id`,
`supplier_invoices.settlement_journal_entry_line_id`, facultatives, uniques ; `NULL` = règlement unitaire, règle d'avant.
Sans elles, la découverte du lettrage (`letterings.rs:1477`, `:1497`, `:1536`, `:1901`) mêlerait les factures.

**C-43-3 — L'unicité change de forme** : `uq_invoice_settlements_entry (journal_entry_id)`
(`20260827000001_invoice_settlements.sql:65`) → `uq_invoice_settlements_entry_invoice (journal_entry_id, invoice_id)` ;
côté fournisseur, `CHECK` (ligne ⇒ écriture).

**C-43-4 — Migration de la 43-1a, non breaking : `20261118000001_grouped_settlements.sql`** (DDL seul) — les deux colonnes
de C-43-2, la clé de C-43-3, et la colonne de réglage du plafond (C-43-24). *La colonne de la 43-5 n'y est plus* (C-43-17).
La 0.18.0 refuse de démarrer sur la base migrée et refuse **toute** sauvegarde 0.19.0 (colonnes inconnues,
`import.rs:613-620`) ; une sauvegarde 0.18.0 s'importe dans la 0.19.0 (colonnes facultatives ou à défaut,
`import.rs:622-628`). « Non breaking » au sens du paragraphe ⚠️ de P1 (sans relever `kesh_version_min_required`, sans
corrompre le schéma) : `DROP INDEX` n'est pas dans la liste de P3, et aucun chemin ne fait lire un groupe par une 0.18.0.
`kesh_version_min_required` reste **0.17.6**. **Retour à la 0.18.0 : par la sauvegarde d'avant la mise à jour seulement.**
Les colonnes de la 43-1b restent dans cette migration même si la 43-1b sort (symétriques, vides, sans effet).

**C-43-5 — Compteurs de migrations selon ce qui sort.** 91 en 0.18.0 ; **92** avec la 43-1a ; **93** si la 43-5 est publiée
(sa migration `20261118000002_bank_line_transaction.sql`). Chaque story qui ajoute une migration écrit le compte qu'elle
produit (P5 : en-tête et `Total` de `docs/migrations-idempotence-audit.md` ; P6 : `migrations_upgrade_path.rs`,
`assert_eq!(total, …)` et la fenêtre `total - N` qui garde sa frontière à la 35ᵉ) ; si la 43-5 sort, son commit part avec,
et le compte de la release redevient 92 sans rien réécrire. P7 : DDL seul, aucune entrée de rejeu ; le rattrapage de
lettrage `20261010000001` ne se rejoue plus (`RETIRED_BACKFILLS`, `post_restore.rs:213-216`), seul son test d'accord gagne
une fixture. **Une colonne à `DEFAULT`** (C-43-24) est remise à sa valeur par défaut à l'import d'une sauvegarde
antérieure (`is_required()` faux, P7) : ici c'est voulu — une sauvegarde 0.18.0 n'a pas de plafond, la 0.19.0 lui donne
celui par défaut.

**C-43-6 — Un règlement groupé ne se défait que par le dé-rapprochement de sa transaction, en entier**, dans l'ordre de
verrous du geste unitaire. Les gestes unitaires sur une facture du groupe butent sur les gardes existantes
(`MATCHED_BANK_TRANSACTION` ; `PaidReconciled`, `supplier_invoices.rs:1194-1209`) ; une écriture groupée est **possédée**,
donc jamais réécrite par la modification d'écriture (`PUT`) : c'est ce qui permet la clé étrangère vers une ligne (C-43-2,
43-1a AC2). Seule l'annulation d'une facture fournisseur payée détache (`supplier_invoices.rs:2639`).

**C-43-7 — Admises dans un groupe** : factures du sens de la transaction, d'un ou plusieurs tiers, **deux au moins**, 50 au
plus ; factures **impayées** (client `validated`, reste dû > 0 ; fournisseur `open`, hors lot `generated`) ; avec la 43-5,
paiements existants non rapprochés. Pas de score, pas de fenêtre de 30 jours (choix explicite) ; **la borne basse de date
de l'unitaire reste** (paiement antérieur à la facture au-delà de la tolérance → refus, `InvoiceDateWindow::check`,
`repositories/reconciliation.rs:565`, refus `PaymentBeforeInvoice` seul). Chaque facture passe les refus de l'unitaire,
énumérés dans `GroupRefusal` (*Pattern batch* : aucun ne sort en `AppError`).

**C-43-8 — API** : `invoice_group` (`invoiceIds`), `supplier_invoice_group` (`supplierInvoiceIds`) ; recherche par texte,
tous tiers (`GET /reconciliation/transactions/{id}/group-candidates`) ; `kind = "grouped_settlement"`. Contrat : 43-2.

**C-43-9 — Audit** : le dépôt écrit lui-même `reconciliation.accepted` (clés **snake_case** de l'unitaire, `batch_size`
compris), puis l'action de l'unitaire par facture (`invoice.paid` avec `reconciliation_audit_id` ; `supplier_invoice.paid`),
avec l'acteur. La route n'écrit rien. Aucune action neuve hors du réglage (C-43-24).

**C-43-10 — Le lot pain.001 confirmé** : hors de la 0.19.0 (C-43-17). **#823 se ferme avec le règlement groupé** de la
0.19.0 ; le cas des lots pain.001 va à l'épique « envoyée » de la 0.20.0 (décision de Guy du 2026-11-17).

**C-43-11 — Réimputation analytique (Epic 42)**. La 42-2 (fournisseur) et la 42-3 (écriture manuelle) **sortent de la
0.19.0** (décision de l'orchestrateur, 2026-11-17) : en 0.19.0, seul le geste client de la 42-1 croise un groupe. Il lit
toutes les écritures de règlement de la facture (`project_reassignment.rs:443-453`) et réimpute toutes leurs lignes
(`reassign_lines_in_tx`, :352-363) ; sur une écriture groupée, il réimputerait la banque, les différences et les autres
factures. **La 43-6 le fait refuser** : `409 PROJECT_REASSIGNMENT_GROUPED_SETTLEMENT`, `details { invoiceId,
journalEntryId }`, juste après le contrôle de version, dès qu'un règlement de la facture porte `journal_entry_line_id`.
La 42-2 reprendra en 0.20.0 le même prédicat par ligne (`settlement_journal_entry_line_id`), y compris pour une facture
**détachée** d'un groupe (`kesh-gate-logs/e42/validation-42-2-p2.md`, V2-1) : refusée ou limitée à sa ligne, jamais
l'écriture entière.

**C-43-12 — Découpage** : § *Stories*.

**C-43-13** — *Tombée* : des tiers différents sont admis ; aucune présélection automatique.

**C-43-14 — Interface « comme bexio »** : ligne du relevé → ajout de factures une à une par recherche → total et
différence → *Enregistrer*. Fiche 43-3.

**C-43-15 — Pas de mélange** clients / fournisseurs : le sens de la transaction choisit la famille.

**C-43-16 — La différence — décision de Guy du 2026-11-17** (`kesh-gate-logs/e43/decision-difference.md` ; elle remplace la
proposition « moins-payé au compte de frais bancaires, trop-perçu refusé »). « Le trop perçu ou le moins doit pouvoir être
comptabilisé à la main : un moins peut être une perte sur débiteur ou des frais bancaires : on ne sait pas a priori. Un trop
perçu peut être un arrondi vers le haut de la part du client. » Montants **bruts** (C-43-21). Pour chaque facture, l'apport
est son reste dû brut (client) ou son `total_amount` (fournisseur).
- **Décomposition `r` / `e` : voir C-43-25** (version définitive, `decision-difference.md`, § « Décomposition de l'écart ») :
  `d = |montant| − Σ apports bruts`, `e` = `d` arrondi au centime (compte choisi), `r = d − e` (compte d'arrondi, hors
  plafond, refus `ROUNDING_ACCOUNT_NOT_CONFIGURED`). *~~`r = Σ (centime(apport) − apport)`, `e = |montant| − Σ
  centime(apport)`~~ — formule **remplacée** par C-43-25 le 2026-11-18 : elle portait le même centime tantôt à l'arrondi,
  tantôt à la différence choisie, et supprimait l'arrondi automatique du cas 20.01 (validation P6 ciblée, CRITICAL-1).*
- **Écart** `e` (C-43-25), **signé** — moins-payé (`e < 0`) ou trop-perçu (`e > 0`), **les deux sens admis**, pour les
  clients comme pour les fournisseurs. Plus de refus de principe (`OVERPAYMENT`, `UNDERPAYMENT`).
- **Compte choisi par l'utilisateur** dans le geste (`differenceAccountId`) : **obligatoire** dès que `e ≠ 0`. L'écran le
  **préremplit**, s'ils sont configurés, avec le compte de frais bancaires désigné pour un moins-payé et le compte d'arrondi
  pour un trop-perçu ; modifiable. Contrôles : compte de la société, **imputable**, non archivé, ni un compte de tiers des
  factures (créance, dette) ni le compte de la banque de la transaction. Aucune restriction de classe en 0.19.0.
- **Plafond** : `|e| ≤` le réglage de la société (C-43-24) ; au-delà, `RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE`.
- **Écriture** : une ligne d'écart de `|e|` sur le compte choisi, du côté qui équilibre l'écriture commune, sans projet,
  non lettrée. **Audit** : le compte choisi et le montant dans les `details`.

**C-43-17 — Paiements déjà passés (43-5) — SORTIE de la 0.19.0** (décision de Guy du 2026-11-17) : Guy introduira en
0.20.0 un état « envoyée » — un lot pain.001 ne paiera plus les factures, qui ne seront payées qu'au rapprochement — et n'a
fait aucun paiement par pain.001. La 43-5 passe en `backlog` ; sa migration `20261118000002` n'est pas écrite ; le compte de
migrations de la release est **92**. Ce qui suit reste comme trace de la conception, pour l'épique « envoyée ». *Avant :*
sans table neuve : colonne `journal_entry_lines.bank_transaction_id`, dans une
**migration propre à la 43-5**, `20261118000002_bank_line_transaction.sql`, qui sort avec elle (validation P2 de la 43-1a,
M-D). Le reste du modèle est inchangé (fiche 43-5). **Dé-rapprochement — décision de Guy (Q5 = a, 2026-11-17)** : les
paiements rattachés ne perdent que leur lien `bank_transaction_id` — ni contre-passés, leurs factures restent payées ; dans
un geste mixte, les règlements **créés** par le geste (factures impayées) sont annulés et leur écriture contre-passée, les
paiements **rattachés** ne perdent que leur lien. Elle lève, pour cette colonne, l'invariant de
`journal_entries_modification.rs:153-171` (une colonne neuve de `journal_entry_lines` serait perdue par le `PUT`) : la 43-5
le justifie par la possession, comme la 43-1a pour les clés vers une ligne.

**C-43-18 — Ordre de sortie à la coupure** (le premier nommé sort le premier ; la 43-5 n'y figure plus, sortie de la 0.19.0,
C-43-17) : **43-7**
(le plafond reste à sa valeur par défaut, non modifiable) ; **43-1b** (avec les commits fournisseur de 43-2 et 43-3) ;
**43-3** (l'API seule reste utile à la clé d'API de Guy) ; **43-2**. La **43-6 ne sort jamais** si la 42-1 et la 43-1a sont
publiées ensemble. La **43-1a** n'est publiée qu'avec la 43-2 et la 43-6 ; sans elles, l'epic entier sort.

⛔ **Barrière de release du commit fournisseur de la 43-2** (décision de l'orchestrateur du 2026-11-18, revue P1 de la 43-2,
B1/E1/A4) : le commit D4 (`70f74f964`, type `supplier_invoice_group` et recherche d'un débit) ne sort **qu'avec la
43-1b** ; sinon `git revert 70f74f964` avant la PR de release, et le guide d'API corrigé à la main (l. 741, 743 et 756,
liste de la fiche 43-2, § *Ordre de retrait*). Contrôle à l'intégration : `pay_supplier_invoices_in_tx` n'est plus le corps
provisoire (`Err(DbError::Invariant)`) et les trois tests fournisseur de `reconciliation_group_e2e.rs` ont perdu leur
`#[ignore]`.

**C-43-19 — Verrous et versions** *(récrit, H-A ; précisé à la P3 de la 43-1a)* : **la méthode de l'unitaire, à deux
écarts près** — l'ordre des deux `UPDATE` (factures avant transaction, sans effet) et l'absence de score et de fenêtre haute
(C-43-7) —
(`routes/reconciliation.rs:2932-2950`) : aucun `FOR UPDATE` sur les factures client en amont ; chaque facture et son reste
dû lus sur l'**instantané** ; puis `UPDATE invoices … AND version = <version lue> AND status = 'validated'` par
identifiant croissant, `rows_affected() == 1` sinon refus `race_during_update` ; la transaction `UPDATE … AND version = ?`
comme l'unitaire (:2918-2925). Fournisseur : la méthode de son unitaire (`lock_for_bank_settlement_in_tx`).
`ensure_open_in_tx`, puis `recheck_open_locked_in_tx` en dernière acquisition (`reconciliation_cancel.rs:631`, `:784`). Le
dépôt pose **son propre point de sauvegarde** et y revient avant de rendre un refus (M-C).

**C-43-20 — Projet par ligne, estampillé sans validation** *(récrit, H-B)* : la ligne de tiers de chaque facture porte le
`project_id` de sa facture, **sans** `validate_taggable_in_tx`, comme les flux qui recopient le projet d'une pièce
(`journal_entries.rs:271-282`, `:309-316`, test `pay_succeeds_when_project_archived_after_tagging`) : une facture au projet
archivé après coup se règle en groupe comme seule. Aucun refus de projet.

**C-43-21 — Montants bruts de bout en bout** : la recherche rend `amountDueRaw` (quatre décimales) et `amountDue`
(centime) ; `difference` et `settledInvoices[].amount` sont au brut ; l'écran calcule en entiers de 1/10 000.

**C-43-22 — Sens, montant nul et devise** *(récrit à la P4 de la 43-2)* : le geste groupé refuse d'abord le sens
contraire et le montant nul par la garde unique de l'unitaire, `refuse_wrong_direction` (`RECONCILIATION_TRANSACTION_NOT_CREDIT`
/ `NOT_DEBIT`, `zero_amount_transaction`), puis une devise non CHF (`RECONCILIATION_CURRENCY_MISMATCH`) ; la recherche
rend une liste vide pour ces transactions ; l'écran suit `canLink` (`transaction_can_be_linked_to_entry`,
`repositories/reconciliation_link.rs:535`, en reste le miroir côté serveur pour le rattachement).

**C-43-23 — Ordre de retrait de la 43-3** (validation P1 de la 43-3, M5) : d'abord les commits fournisseur, puis les
captures non essentielles ; **jamais l'E2E** ; les traductions de/it/en ne se retirent pas (la garde `i18n-keys.test.ts`
exige les quatre locales). L'E2E ne tourne que sur la branche réunie, au plus tôt mercredi soir, définitivement jeudi.

**C-43-24 — Le plafond de la différence est un réglage de la société** (décision de Guy, Q4, 2026-11-17) : colonne
`company_invoice_settings.group_settlement_max_difference DECIMAL(19,4) NOT NULL DEFAULT 5.0000`, posée par la migration de
la 43-1a, sur la table qui porte déjà `default_rounding_account_id` et `default_bank_fees_account_id`
(`company_invoice_settings.rs:628-670`). Le geste lit la valeur **au moment du geste** ; ligne de réglages absente → la
constante `DEFAULT_GROUP_DIFFERENCE_MAX`, valeur par défaut seulement, citée une seule fois. Le réglage se modifie dans
*Administration → Facturation* (`frontend/src/routes/(app)/settings/invoicing/+page.svelte`,
`PUT /api/v1/company/invoice-settings`, Admin, `routes/company_invoice_settings.rs:325`) : ≥ 0, au centime, au plus
**100.00** ; tracé à l'audit. Porté par une petite story à part, **43-7**, coupable.

**C-43-25 — Décomposition de l'écart et arrondi automatique** (décision de l'orchestrateur, **version définitive**,
`kesh-gate-logs/e43/decision-difference.md`, § « Décomposition de l'écart », dont le tableau fait foi) :
`d = |montant| − Σ restes bruts` (quatre décimales) ; **`e` = `d` arrondi au centime**, demi-centime arrondi en s'éloignant
de zéro (`amount_due_to_centime`) ; **`r = d − e`** (`|r| ≤ 0.005`). `r` va au **compte d'arrondi** (aucune ligne si
`r = 0`), comme pour une facture seule ; `e` va au **compte choisi** (règle de Guy, C-43-16), et c'est lui seul que le plafond
borne. L'**arrondi automatique** est le cas `e = 0` (`|d| < 0.005`) : aucun compte demandé, ni l'écran ni l'API ne parlent de
différence. Champs : `roundingDifference = r`, `gap = e` (au centime, signé), `difference = d = r + e` ; `r` et `e` peuvent
être non nuls ensemble. Exemples : 2 × 10.0050 payés 20.01 → aucun écart ; payés 20.02 → `e` +0.01, compte demandé ;
3 × 10.0050 payés 30.06 → `e` +0.05 et `r` −0.0050. *(Remplace les formes précédentes du même jour :
`centime(n × 0.005)`, puis `|d| < 0.005` avec `gap = d` brut — et la formule `Σ centime(apport)` de C-43-16, barrée.)*
**Ordre des contrôles — celui du code de la 43-1a** (`t3.md`, `grouped_settlements.rs:306-322` et :664-716) : **le
plafond sur `|e|` d'abord** (`DifferenceTooLarge`, rendu dans `group_difference`), puis le compte d'arrondi (requis si
`r ≠ 0`, avec ses contrôles habituels), puis `DifferenceAccountRequired` (si `e ≠ 0` sans compte choisi), puis les
contrôles du compte choisi (banque, existence, imputable, compte de tiers des factures du groupe). *(Corrigé le
2026-11-18 : cet ordre plaçait le plafond en dernier, contre le code.)* Calcul porté par `group_difference` (43-1a,
développeur prévenu), contrat 43-2 D8 bis, écran 43-3 D3.

**C-43-26 — Libellés et préremplissage par famille** (décision de l'orchestrateur, 2026-11-18) : clients — crédit
inférieur aux factures « Moins-payé » (prérempli : compte de frais bancaires), crédit supérieur « Trop-perçu » (prérempli :
compte d'arrondi) ; fournisseurs — débit supérieur aux factures « Débit supérieur aux factures » (prérempli : frais
bancaires), débit inférieur « Débit inférieur aux factures » (prérempli : arrondi). On ne préremplit qu'un compte désigné
**utilisable** (actif, imputable) ; sinon le champ reste vide, pour ne pas reproduire #1000. Fiche 43-3, D2 bis.

## Registre des propositions en attente de Guy

| Réf. | Proposition | Écrite dans | Si Guy la change |
|---|---|---|---|
| ~~M-A (C-43-16)~~ | ~~moins-payé au compte de frais bancaires, trop-perçu refusé~~ — **remplacée** par la décision de Guy du 2026-11-17 : deux sens admis, compte choisi par l'utilisateur, refus au-delà du plafond seulement (C-43-16) | 43-1a (prévenue en développement), 43-1b (reportée), 43-2 et 43-3 (après leur passe 3) | — |

*Tranchées* (décisions de Guy du 2026-11-17) : Q5 = (a) (C-43-17, désormais sans objet en 0.19.0) ; Q4 = plafond
réglable (C-43-24) ; Q4 bis = compte choisi, deux sens (C-43-16) ; Q1 = #823 se ferme avec le règlement groupé (C-43-10) ;
la 43-5 sort de la 0.19.0 (C-43-17).

## Stories

| Story | Objet | Part de | Parallèle |
|---|---|---|---|
| **43-1a** | migration (sans la colonne de la 43-5), entités, lettrage par ligne (deux familles), règlement groupé **client** avec différences, dé-rapprochement client, lecture du plafond | `epic/43` | — |
| **43-1b** | règlement groupé **fournisseur**, dé-rapprochement, détachement | 43-1a au commit T3 | 43-2, 43-3, 43-7 |
| **43-2** | API, `docs/api-external.md` | 43-1a au commit T3 | 43-1b, 43-3, 43-7 |
| **43-3** | écran façon bexio, annulation, i18n, manuel utilisateur, E2E | `epic/43` (Vitest sur contrat) ; E2E sur la branche réunie | tout |
| ~~43-5~~ | paiements déjà passés — **`backlog`, sortie de la 0.19.0** (C-43-17), vers l'épique « envoyée » de la 0.20.0 | — | — |
| **43-6** | refus de la réimputation d'une facture réglée en groupe (42-1) | branche réunie 42-1 + 43-1a | en dernier |
| **43-7** | réglage du plafond (API, écran, audit) | 43-1a au commit T1 | tout |
| **43-4** | solde de la dette (plafond 2 issues) | fin d'epic | — |

**Jalons** : commit T3 de la 43-1a mercredi 18 avant midi ; 43-1a en revue mercredi 18 à 18 h ; 43-6 prête mercredi soir ;
43-1a, 43-2, 43-6 closes jeudi midi ; le reste jeudi soir.

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
| `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql`, `…/20261118000002_bank_line_transaction.sql` (43-5), `test-schema/0001_schema_squash.sql`, `docs/migrations-idempotence-audit.md`, `tests/migrations_upgrade_path.rs`, `tests/journal_entries_modification.rs` | 43-1a, 43-5 | multi-devise ; toute migration |
| `crates/kesh-db/src/entities/{invoice_settlement,supplier_invoice,journal_entry}.rs` et l'entité des réglages de facturation | 43-1a, 43-5, 43-7 | multi-devise |
| `crates/kesh-db/src/repositories/grouped_settlements.rs` (neuf) | 43-1a, 43-1b, 43-5 | — |
| `crates/kesh-db/src/repositories/reconciliation_group_candidates.rs` (neuf, recherche ; `kesh-db` : gate complet en boucle pour la 43-2) | 43-2 seule | — |
| `repositories/{letterings,reconciliation_cancel,invoice_settlements,supplier_invoices,journal_entries,company_invoice_settings}.rs` | 43-1a, 43-1b, 43-5, 43-7 | multi-devise |
| `repositories/{bank_accounts,invoices,settlement_cancellation,reconciliation_link}.rs` | 43-5 | — |
| `repositories/project_reassignment.rs` (branche Epic 42) | 43-6 | **Epic 42** |
| `crates/kesh-api/src/routes/{reconciliation,reconciliation_group,company_invoice_settings}.rs`, `lib.rs`, `audit_labels.rs`, `exports/csv_tables.rs` | 43-1a (export), 43-2, 43-7 | 42-1 |
| `frontend/src/lib/features/reconciliation/`, `frontend/src/routes/(app)/settings/invoicing/` | 43-3, 43-7 | — |
| catalogues i18n (quatre locales) | 43-3, 43-6, 43-7 | tous (bloc par epic) |
| `docs/manual/fr/admin-manual.tex` (43-1a, 43-7), `user-manual.tex` (43-3, 43-5, 43-6) — `.tex` seulement | | 42, 44 |
| `docs/api-external.md` | 43-2, 43-6, 43-7 | 42 |

## Pour les travaux parallèles (multi-devise)

1. Un règlement peut être **commun à plusieurs factures**, de tiers différents : **une** écriture — une ligne de banque, une ligne de tiers par facture (projet de sa facture), au plus une ligne d'arrondi et une ligne d'écart (moins-payé ou trop-perçu, sur un compte **choisi par l'utilisateur**, au plus le plafond) — pointée par **une** transaction (`matched_entry_id`).
2. `invoice_settlements` : N lignes pour le même `journal_entry_id` ; colonne neuve `journal_entry_line_id` (`NULL` = règlement unitaire) ; unicité `(journal_entry_id, invoice_id)`.
3. `supplier_invoices` : N factures pour le même `settlement_journal_entry_id` ; colonne neuve `settlement_journal_entry_line_id`.
4. `company_invoice_settings.group_settlement_max_difference` : plafond de l'écart admis (`|e|`, dans les deux sens), réglage de la société (5.00 par défaut ; deux décimales à
   l'écran et à l'API, quatre en colonne).
5. Montants **bruts** (quatre décimales) dans le calcul ; le centime pour l'affichage et pour l'écart.
6. **Le geste groupé n'accepte qu'une transaction en CHF** (refus `RECONCILIATION_CURRENCY_MISMATCH`, après la garde de sens `refuse_wrong_direction` ; écran : `canLink`) : c'est le point à élargir par le multi-devise, avec une règle de devise **par ligne de tiers** ; un écart de change ne se confond ni avec la ligne d'arrondi ni avec la ligne d'écart choisie.
7. Migration : `crates/kesh-db/migrations/20261118000001_grouped_settlements.sql` (92ᵉ, seule de la release ; la 43-5 et sa colonne `journal_entry_lines.bank_transaction_id` sont sorties de la 0.19.0) ; DDL seul, non breaking. Celle du multi-devise prend un horodatage postérieur et part de `main` après la 0.19.0.
8. Code : `repositories/grouped_settlements.rs` (neuf), `letterings.rs`, `reconciliation_cancel.rs`, `routes/reconciliation_group.rs` (neuf), `frontend/src/lib/features/reconciliation/`. Ne pas les modifier avant la publication de la 0.19.0 sans prévenir.
