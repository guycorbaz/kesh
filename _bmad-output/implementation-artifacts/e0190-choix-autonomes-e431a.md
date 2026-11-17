# Story 43-1a — registre des choix (développement, 2026-11-17)

Registre de la story 43-1a (clé `e431a`). Les décisions de l'épique (C-43-*) font foi ; ce registre consigne les décisions
reçues pendant le développement et les choix tranchés par l'agent.

## Décisions reçues

**C-e431a-1 — Règle de la différence : décision de Guy du 2026-11-17** (remplace D4 bis de la fiche et la proposition M-A
du registre de l'épique). Source : `kesh-gate-logs/e43/decision-difference.md`, transmise par l'orchestrateur.
- L'écart au centime `gap = e = |montant| − Σ centime(apport)` est admis **dans les deux sens** (moins-payé, trop-perçu),
  pour les deux familles ; seul `|gap| >` plafond est refusé (`DifferenceTooLarge`).
- Le compte de l'écart est **choisi par l'utilisateur** : paramètre `difference_account_id: Option<i64>` de
  `settle_invoices_in_tx` et de `pay_supplier_invoices_in_tx`, obligatoire dès que `gap ≠ 0`, ignoré sinon. Contrôles :
  de la société et actif (`DifferenceAccountInvalid`), imputable (`DifferenceAccountNotPostable`), pas le compte de banque
  (`DifferenceAccountIsBankLedger`), pas un compte de tiers (`ClaimAccount { role: Counterparty }`). Pas de restriction de
  classe.
- Les fractions de centime (`r`) restent au compte d'arrondi, hors plafond.
- Écriture client : `gap < 0` au débit du compte choisi, `gap > 0` au crédit. Aucun projet.
- `group_difference(amount, gross, max)` perd son paramètre de famille (la règle est la même).
- Réversibilité : la fonction pure et ses tests portent toute la règle.

**C-e431a-2 — Ajustements du contrat de T3 décidés par l'orchestrateur** (validations P2 et P3 des 43-2 et 43-3) :
1. `UnsupportedTransaction` → `CurrencyMismatch { currency }` (devise seule) ; `WrongDirection { amount }` pour le sens
   contraire et le montant nul ; ordre de l'étape 3 : sens et montant nul **puis** devise.
2. Bornes et doublon dans le dépôt, après le `SAVEPOINT`, seule définition.
3. `ClaimAccount { claim, role, account_id, account_number, piece }` avec les rôles réels de `SettlementAccountRole`
   (`Counterparty`, `Rounding`) — ni `receivable` ni `bank_fees` comme rôle (annule le point 4 du premier message).
4. `Decomposition { difference, rounding_difference, gap }` — `gap` signé remplace `bank_fees`.
5. Variantes supprimées : `Overpayment`, `Underpayment`, `BankFeesAccountNotConfigured`.
6. Erreurs du lettrage : `Err(DbError)`, jamais un refus (D6). Vérifié : `sync_invoice_in_tx` et
   `dissolve_invoice_document_group_in_tx` remontent par `?`.
7. `ROLLBACK TO SAVEPOINT` en échec (1305) → `Err(DbError::Sqlx)` ; `is_savepoint_lost` public et testé.
8. `GroupedSettlement.difference_account_id` ; la vue le relit sur la ligne d'écart
   (`ReconciliationView::grouped_difference_account_id`).

**C-e431a-12 — Ajustements de la passe 4 des fiches 43-2 et 43-3** (orchestrateur, quatrième message) :
1. Ordre des contrôles du compte d'écart, celui de la contrepartie choisie : banque (`DifferenceAccountIsBankLedger`),
   manquant — inexistant, archivé ou d'une autre société — (`DifferenceAccountInvalid`, route `ACCOUNT_NOT_FOUND` avec
   `missingAccountIds`), non imputable (`DifferenceAccountNotPostable { account_id, account_number }`), compte de tiers
   (`ClaimAccount { role: Counterparty }`). La variante `DifferenceAccountNotUsable` des étapes intermédiaires n'existe
   plus.
2. `ClaimAccount` porte `bank_account_id: Option<i64>` : renseigné quand le compte de créance d'une facture (`piece`)
   est le compte de banque (`account_id` = ce compte) ; vide quand le compte choisi pour l'écart est une créance
   (`account_id` = le compte choisi).
3. **Arrondi automatique** dans `group_difference` : `|e| ≤ centime(n × 0.005)` (n factures, `auto_rounding_tolerance`)
   → l'écart part au compte d'arrondi avec les fractions (`rounding_difference = difference`, `gap = 0`), sans compte
   choisi ni plafond. Exemple : deux restes de 10.0050 payés 20.01 → ni arrondi ni écart (les deux s'annulent).
4. Commentaire SQL de la colonne de plafond corrigé dans la migration (non publiée, P8 délibéré) : checksum mis à jour
   dans `migrations.sha384`, squash régénéré, bases `kesh_e431a`, `kesh_e2e_e431a`, `kesh_ci_e431a` reconstruites.
- Conséquence propre à la vue (C-e431a-8 amendé) : le compte d'écart se relit en recalculant la décomposition par
  `group_difference` (montant de la ligne de banque, montants bruts des règlements, plafond sans objet), puisque
  l'arrondi automatique peut poser une ligne d'arrondi sans fraction de centime.

## Choix de l'agent

**C-e431a-3 — `TABLES_TO_TRUNCATE`** : `imported_supplier_invoices` et `supplier_invoices` déplacées ensemble avant
`journal_entry_lines` (L5 de la fiche, première option). Elles étaient déjà après `projects`, que `supplier_invoices`
référence : l'ordre « FK-safe » annoncé y redevient vrai. Sans effet à l'exécution (`FOREIGN_KEY_CHECKS = 0`). Autre
option : amender le commentaire. Réversible.

**C-e431a-4 — Lettrage par ligne (AC4)** : la ligne du règlement est lue **dans la même requête verrouillante** que
l'écriture (`SYNC_SETTLEMENT_ENTRIES_SQL`, `SYNC_SUPPLIER_INVOICE_SQL`) ; le filtre `keep_owned_lines` ne garde, d'une
écriture dont la pièce porte sa ligne, que cette ligne. Verrous, index forcés et ordre inchangés. Réversible.

**C-e431a-5 — Le dé-rapprochement d'un groupe refuse tous les rangs avant toute écriture** : par facture,
`settlement_cancel_blocker_unlinking` (rangs 1, 1 bis, puis la queue 2 à 5 de l'écriture, exemptée pour la transaction
défaite) ; le premier motif rendu en `ReconciliationNotCancellable { blocker }`. L'unitaire laisse les rangs 3 à 5 à la
contre-passation ; pour un groupe, la contre-passation vient après la suppression des règlements, et « tous les refus
avant toute écriture » (AC7.1) l'exige. Autre option : laisser `reverse_in_tx` refuser (rollback de l'appelant).
Réversible.

**C-e431a-6 — Un groupe ignore le rang 0 (autre côté d'un virement)** : `entry_cancel_outlook` classe d'abord ; un groupe
n'est jamais un retrait seul (`unlink_only = false`), `expect_unlink_only = Some(true)` → `ReconciliationCancelChanged`.
Une écriture de règlement groupé n'est pas l'écriture d'un virement entre comptes propres.

**C-e431a-7 — Compte d'arrondi comparé aux seules créances**, comme l'unitaire (c-bis) — non à la banque.

**C-e431a-8 — Compte d'écart relu par la vue** : ordre `line_order` (banque, une ligne par facture, arrondi si `r ≠ 0`,
écart) ; `r` recalculé depuis les montants bruts des règlements, ce qui sépare les deux lignes même quand l'utilisateur
choisit le compte d'arrondi pour l'écart. Autre option : relire l'audit `reconciliation.accepted` (rejetée : l'audit
n'est pas une source de lecture fonctionnelle).

**C-e431a-9 — Classement d'un groupe fournisseur avant la 43-1b** : `classify` reconnaît un groupe fournisseur (ligne
renseignée) ; `grouped_settlements::cancel_in_tx` rend `DbError::Invariant` s'il ne trouve aucun règlement client — la
43-1b y ajoute la branche fournisseur. Inatteignable tant que rien n'écrit `settlement_journal_entry_line_id`.

**C-e431a-10 — Libellé de l'écriture** : « Encaissement groupé de N factures : n1, n2, … », borné à 500 caractères
(colonne `description`), coupé avec « ... ».

**C-e431a-11 — Registre des checksums** : la migration est inscrite à `migrations.sha384` dès T1 (le garde
`published_migrations_keep_their_checksums` l'exige). Toute modification avant publication change la ligne avec le
fichier (P8 : délibérée, bases de dev à reconstruire).
