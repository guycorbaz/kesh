# Story 43-1b : un règlement pour plusieurs factures — factures fournisseur (#823)

Status: done

*Fiche rédigée le 2026-11-17 au soir par l'agent de rédaction de l'Epic 43 (Opus 5.5), workflow `bmad-create-story`,
découpage de la 43-1 (validation P1, H3) ; **alignée dans la nuit** sur la remédiation de la 43-1a (validation P2 : H-B,
M-A, M-B, M-C ; exigences de la 43-2). Code lu à `8b50fd195`. Release **0.19.0**. **Part de la 43-1a à son commit T3**
(migration, entités, signatures) ; se fusionne après elle. **Coupable à la coupure** (C-43-18) : sans elle, la migration
reste entière et la colonne fournisseur vide, la 43-2 et la 43-3 retirent leurs commits fournisseur. `kesh-db` touché :
gate complet en boucle. Décisions : C-43-1 à C-43-26 (fichier d'épique), qui font foi ; la règle de différence est
tranchée (C-43-16, C-43-25). **Contrat figé de la 43-1a** : `kesh-gate-logs/e43/t3.md` (commit `241839568`), code sur
la branche `story/43-1a-reglement-groupe-modele`, registre `e0190-choix-autonomes-e431a.md` (C-e431a-5, -6, -8, -9
repris ci-dessous). Validation terminée (`kesh-gate-logs/e43/validation-fiches-contre-t3.md`, puis
`validation-fiches-p-ciblee-2.md` : 0 au-dessus de LOW).*

**Issues** : `refs #823` ; aucun mot-clé de fermeture dans un commit.

## Story

En tant que comptable qui paie plusieurs factures fournisseur (d'un ou de plusieurs fournisseurs) par un seul débit, je
veux qu'au rapprochement Kesh passe **un** paiement qui solde et lettre chacune, et qu'un dé-rapprochement le défasse.

## Ce qui se lit au code (`8b50fd195`)

- Acceptation unitaire : `accept_one_supplier_invoice` (`routes/reconciliation.rs:3312`), égalité exacte du montant
  (:3406), `supplier_invoices::lock_for_bank_settlement_in_tx` (`supplier_invoices.rs:2345` : facture `FOR UPDATE`, compte,
  lot `generated` en lecture verrouillante, garde du lot du compte), `pay_in_tx_as` (:1976 ; une écriture **par** facture,
  non réemployable telle quelle ; projet d'en-tête `inv.project_id` :2163 ; `UPDATE … settlement_journal_entry_id = ?`
  :2179 ; audit `supplier_invoice.paid` :2219), `purchase_payable_line` (:1891) ; fenêtre fournisseur
  `InvoiceDateWindow::for_supplier_transaction` (`repositories/reconciliation.rs:541`).
- Détachement : `cancel_in_tx` (:2530) remet les colonnes de règlement à `NULL` (:2639) et écrit
  `details.settlementJournalEntryId` ; annulation du règlement : `cancel_settlement_in_tx` (:2819, `UPDATE` :2929).
- Retour en brouillon d'une payée rapprochée refusé : `PaidReconciled` (:1194-1209).

## Critères d'acceptation

- **AC1** — `grouped_settlements::pay_supplier_invoices_in_tx(tx, company_id, bank_account_id, bank_transaction_id,
  supplier_invoice_ids, difference_account_id: Option<i64>, batch_size, acteur, aujourd_hui)` (signature figée par le
  contrat `kesh-gate-logs/e43/t3.md`, commit `241839568`), avec le **point de sauvegarde propre** de la 43-1a (D6) : bornes et doublon d'abord ; transaction
  (introuvable, non `pending`, puis sens et montant nul par la variante `WrongDirection`, puis devise par
  `CurrencyMismatch`) ; puis la méthode de **son** unitaire : `lock_for_bank_settlement_in_tx` facture par facture,
  **identifiants croissants** (réemployé, non recopié) ; refus (`GroupRefusal`, `GroupedDocument::SupplierInvoice`, numéro
  affichable) : introuvable, non `open`, lot `generated`, tenue par un lot du compte, compte bancaire, date (borne basse
  seule : `InvoiceDateWindow::for_supplier_transaction(&tx).check(…)`, `PaymentBeforeInvoice` → refus,
  `PaymentTooLate` ignoré), compte de dette égal à la banque, et ceux de la différence et de `create_in_tx` (AC2). **Aucun
  refus de projet** (C-43-20).
- **AC2** — Écriture (sans projet d'en-tête, forme D7 de la 43-1a) : crédit banque de |montant| ; par facture, débit de
  **son** compte de dette (`purchase_payable_line`) de son `total_amount`, avec le projet de la facture estampillé sans
  validation. **Différences — décision de Guy du 2026-11-17** (C-43-16, `kesh-gate-logs/e43/decision-difference.md`) :
  calculées par **la même** fonction pure `group_difference` que le bras client, selon la **décomposition définitive de
  C-43-25** (`decision-difference.md`, § « Décomposition de l'écart », dont le tableau fait foi) : `d = |montant| − Σ
  total_amount` (quatre décimales) ; **`e` = `d` arrondi au centime** (demi-centime en s'éloignant de zéro) ; **`r = d − e`**.
  Les `total_amount` fournisseur étant au centime, `r` est nul en pratique ; s'il ne l'est pas, il va au compte d'arrondi
  (au débit si `r > 0`), hors plafond. Écart `e` **signé, les deux sens admis** (plus payé ou moins payé) ; si `e ≠ 0`, une
  ligne de `|e|` sur le compte **choisi par l'utilisateur** (`difference_account_id`), du côté qui équilibre l'écriture (au
  débit si `e > 0`, au crédit sinon — symétrique du code client, `grouped_settlements.rs:759-775`, crédit si positif) ;
  arrondi automatique si `e = 0`. **Ordre des refus, celui du code** (`group_difference` puis `settle_client_inner`,
  `grouped_settlements.rs:306-322`, :664-716) : (1) `|e| >` plafond → `DifferenceTooLarge`, **d'abord, dans
  `group_difference`** ; (2) `r ≠ 0` → compte d'arrondi requis et contrôlé ; (3) `e ≠ 0` sans compte →
  `DifferenceAccountRequired` ; (4) compte égal à la banque ; (5) inexistant, archivé ou d'une autre société
  (`ACCOUNT_NOT_FOUND`) ; (6) non imputable ; (7) égal à un compte de dette des factures du groupe. Plus de refus
  `Underpayment`. **Ordre des lignes de l'écriture** (`line_order`, comme D2 de la 43-1a, relu par la vue, AC6 bis) :
  banque ; une dette par facture, **par identifiant croissant** ; arrondi si `r ≠ 0` ; écart si `e ≠ 0`. Ni projet ni lettrage sur ces lignes ; `enforce_postable = false` pour
  l'écriture (comptes de pièces et de réglages), le compte choisi étant contrôlé à part. Audit : `gap` (= `e`),
  `difference_account_id`, `rounding_difference` (= `r`) — jamais `bank_fees`. *(Formule précédente `e = |montant| − Σ
  centime(total)` remplacée par C-43-25 le 2026-11-18.)*
- **AC3** — Chaque facture : `status = 'paid'`, `settlement_type = 'bank_transfer'`, `settlement_bank_account_id`,
  `settlement_journal_entry_id`, **`settlement_journal_entry_line_id`**, `paid_at`, `version + 1`, `WHERE … AND version = ?
  AND status = 'open'` (version lue sous le verrou de `lock_for_bank_settlement_in_tx`, qui **est** la méthode de l'unitaire
  fournisseur), `rows_affected() == 1`. Transaction rapprochée comme en 43-1a AC5 (`AND version = ?`) ;
  `recheck_open_locked_in_tx` en dernière acquisition.
- **AC4** — Audit (C-43-9) : `reconciliation.accepted` (`proposal_type: "supplier_invoice_group"`, clés snake_case,
  `batch_size`) d'abord, puis `supplier_invoice.paid` par facture avec les détails de `pay_in_tx_as` et l'acteur ; pas de
  doublon.
- **AC5** — Lettrage de chaque facture (`sync_supplier_invoice_in_tx`) : une clé par facture (sa ligne d'achat, sa ligne
  de dette).
- **AC6** — Dé-rapprochement : le bras fournisseur de `grouped_settlements::cancel_in_tx` (ordre de verrous de
  l'unitaire : facture fournisseur **avant** l'écriture, comme `supplier_invoices::cancel_settlement_in_tx`) ; refus par
  facture (`supplier_settlement_cancel_blocker`) avant toute écriture ; délettrage, factures rouvertes (`open`, colonnes
  `settlement_*` **ligne comprise** et `paid_at` à `NULL`, `version + 1`), contre-passation, audit
  `supplier_invoice.settlement_cancelled` (ou l'action de l'unitaire) par facture ; la facture qui bloque est rendue à
  `entry_cancel_outlook` comme côté client. Et, repris du registre de la 43-1a :
  - le bras fournisseur **remplace** l'`DbError::Invariant` que `cancel_in_tx` rend aujourd'hui quand il ne trouve aucun
    membre client (`grouped_settlements.rs:1225-1230`, C-e431a-9) ; `classify` reconnaît déjà le groupe fournisseur (ligne
    renseignée) — test : un groupe fournisseur se dé-rapproche, sans `Invariant` ;
  - un groupe fournisseur n'est **jamais un retrait seul** (il ignore le rang 0, C-e431a-6) : `expect_unlink_only =
    Some(true)` → `DbError::ReconciliationCancelChanged { unlink_only: false }` (:1257-1258), rien d'écrit — test.
- **AC6 bis — Vue d'un groupe fournisseur** (contrat `t3.md`, § « Vue et dé-rapprochement » ; C-e431a-8) : aujourd'hui
  `view_members` (`grouped_settlements.rs:1070`) ne lit que les membres **clients**, et `difference_account_of`
  (:1109-1125) ne lit qu'`invoice_settlements` — sur un groupe fournisseur, `ReconciliationView::grouped_invoices` sort vide
  et `grouped_difference_account_id` serait faux ou `None`, **sans erreur**. La 43-1b les complète :
  - `view_members` gagne le bras fournisseur : membres lus sur `supplier_invoices.settlement_journal_entry_id = écriture`
    (`kind` `supplier_invoice`, `id`, numéro affichable, montant brut = `total_amount`), par identifiant croissant ;
  - `difference_account_of` gagne le même bras : ~~apports = `total_amount` des membres~~ *(remplacé au
    développement et aux revues P1 et P2, C-e431b-4, C-e431b-15)* apports = les montants des lignes de dette **nommées**
    — celles des membres et celles des factures détachées (trace du détachement, AC7), leur nombre recoupé avec les
    factures du paiement —, qui doivent occuper les positions 1 à n ; décomposition recalculée par
    `group_difference(|montant|, apports, Decimal::MAX)`, ligne d'écart relue **par position** dans l'ordre des lignes
    d'AC2 (banque, une dette par facture, arrondi si `r ≠ 0`, écart si `e ≠ 0`) ; `None` si `e = 0` ; toute autre
    forme, `Invariant`.
  La 43-2 (AC7, `groupedInvoices`, `differenceAccount` fournisseur) et la 43-3 (AC5, dialogue d'annulation d'un groupe
  fournisseur) en dépendent.
- **AC7** — **Les deux sites** qui remettent `settlement_journal_entry_id` à `NULL` — `supplier_invoices.rs:2639`
  (`cancel_in_tx`, détachement) et `:2929` (`cancel_settlement_in_tx`, inatteignable aujourd'hui pour un membre d'un groupe
  rapproché, mais gardé quand même) — emploient le fragment partagé `CLEAR_SETTLEMENT_SQL` de la 43-1a (AC1.3, T2), qui
  remet aussi la ligne à `NULL` (le `CHECK` rendrait sinon une 500, `4025`). Détachement d'une facture payée du groupe : `details` de l'audit gagne `settlementJournalEntryLineId`. Le dé-rapprochement ultérieur
  contre-passe l'écriture entière et ne rouvre que les factures encore attachées ; quand plus aucune ne l'est, l'écriture
  est classée `entry`. Faux positif bénin du gel « paiement détaché » (`find_modification_blocker`) écrit au doc-comment.
- **AC8** — Gestes unitaires refusés : retour en brouillon (`PaidReconciled`), annulation du règlement
  (`MATCHED_BANK_TRANSACTION`). Tests.

## Tests

Dans `crates/kesh-db/tests/grouped_settlements.rs` (module fournisseur) : groupe de deux fournisseurs ; facture d'un lot
`generated` refusée ; écart dans les deux sens accepté sur le compte choisi (plus payé au débit, moins payé au crédit),
au-delà du plafond refusé, sans compte refusé, compte interdit (dette, banque, non imputable, archivé) refusé ; projets par
ligne, **projet archivé
après coup accepté** ; date antérieure refusée ; un refus validé par le test laisse les tables inchangées (point de
sauvegarde) ; concurrence avec `pay_in_tx` depuis la fiche et avec `create_batch` ; dé-rapprochement ; détachement puis
dé-rapprochement (AC7) ; acteur clé d'API ; **ordre des refus** (plafond dépassé **sans** compte → `DifferenceTooLarge`,
non `DifferenceAccountRequired`) ; **écriture équilibrée** pour un cas où `r` et `e` sont non nuls ensemble (restes
fournisseur fabriqués à quatre décimales par la fixture, cas 4 ou 6 du tableau de la décision) ; **dé-rapprochement d'un
groupe fournisseur sans `Invariant`** et **`expect_unlink_only = Some(true)` refusé** (AC6) ; **vue d'un groupe
fournisseur** (AC6 bis) : liste des membres, compte d'écart relu pour `e > 0` et pour `e < 0`, sans écart → `None`.
Mutations (`kesh-gate-logs/e431b/mut/`) : lire les membres clients seulement dans `view_members` (le test de vue rougit) ; omettre la ligne au détachement
(AC7 rougit) ; retirer `AND version = ?` ; inverser le côté de la ligne d'écart (l'écriture ne s'équilibre plus, le
test de l'écart rougit) ; retirer le contrôle du compte choisi égal à la banque (le test du compte interdit rougit).

## Tâches

- [x] **T0** — Étape 0 : LOW de la validation ; signatures du T3 de la 43-1a relues (`kesh-gate-logs/e43/t3.md`).
- [x] **T1** — AC1 à AC5. — [x] **T2** — AC6 à AC8. — [x] **T3** — Gates, Dev Agent Record, prompt de revue P1.

## Note de version

- **Ajouté** — Un débit peut régler plusieurs factures fournisseur, d'un ou de plusieurs fournisseurs : un seul paiement,
  chaque facture soldée et lettrée ; un petit écart, dans un sens ou dans l'autre, dans la limite d'un plafond (5.00 par
  défaut), se comptabilise sur le compte que vous choisissez (refs #823).

## Dev Notes

Modules : `repositories/grouped_settlements` (paiement, bras du dé-rapprochement, vue : `view_members`,
`difference_account_of`), `repositories/supplier_invoices` (détachement), `reconciliation_cancel` (bras). Trois.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, 2026-11-17), en autonomie ; choix au registre `e0190-choix-autonomes-e431b.md`
(C-e431b-1 à 11).

### Completion Notes List

- **Étape 0** : LOW L-3 de `validation-fiches-p-ciblee-2.md` traité (en-tête : le chemin du worktree de la 43-1a
  remplacé par sa branche). Contrat `t3.md` relu, annotation de la revue P1 de la 43-1a comprise. L-1, L-2, L-4 portent
  sur la 43-7 et l'épique : hors de cette fiche.
- **AC1 à AC5** : `pay_supplier_invoices_in_tx` / `pay_supplier_inner` — point de sauvegarde, bornes, transaction (sens,
  devise), `lock_for_bank_settlement_in_tx` réemployée par identifiant croissant, borne basse de date, dette lue sur
  l'achat et contrôlée égale au `total_amount` (C-e431b-2), dette ≠ banque, même `group_difference` (plafond d'abord),
  compte d'arrondi, compte d'écart, écriture sans projet d'en-tête, factures `paid` avec leur ligne à la version lue
  sous verrou, transaction rapprochée, lettrage par facture, `recheck_open_locked_in_tx`, audit.
- **AC6, AC6 bis** : bras fournisseur de `view_members`, `difference_account_of` (bloc des dettes lu sur l'écriture,
  C-e431b-4 — *annoté le 2026-11-18 : cette forme devinée a été remplacée aux revues P1 et P2 par les lignes de dette
  nommées et leur nombre recoupé, C-e431b-4 récrit, C-e431b-15*), `group_cancel_blocker` (tête `supplier_settlement_cancel_head` extraite, C-e431b-5) et `cancel_in_tx`
  (`supplier_cancel_in_tx`) : l'`Invariant` de C-e431a-9 est remplacé ; `Some(true)` → `ReconciliationCancelChanged`.
- **AC7** : les deux sites emploient déjà `CLEAR_SETTLEMENT_SQL` (43-1a) ; l'audit du détachement gagne
  `settlementJournalEntryLineId` ; doc-comment du faux positif bénin sur `supplier_invoices::cancel_in_tx`.
- **AC8** : retour en brouillon refusé (`PaidReconciled`) ; annulation du règlement refusée sur le motif
  `MatchedBankTransaction` (variante `EntryNotReversable`, C-e431b-8).
- **Hors des modules annoncés** (deux gardes de tests que le gate a fait rougir) :
  `kesh-api/tests/bank_account_closed_inventory.rs` (écrivains classés, `ANNULATIONS` 6 → 7, `FENETRE` + 2, anti-muets
  41 → 45 sites, G 14 → 16, revérifications 9 → 11 — C-e431b-6) ; `kesh-db/tests/letterings_lexical.rs` (cinq
  écrivains du règlement fournisseur au lieu de trois ; `reverse_in_tx(` admis comme contre-passation).
- **Tests** : 12 dans `crates/kesh-db/tests/grouped_settlements_supplier.rs` (C-e431b-1). ⚠️ Écrits **après** le code
  (écart à « rouges d'abord », `kesh-gate-logs/e431b/rouges.log`) ; la campagne de mutations prouve qu'ils mordent.
- **Mutations** (`kesh-gate-logs/e431b/mut/campagne.log`, code commité avant) : 6 jouées, 5 tuées (M1, M2, M4, M5, M6),
  1 équivalente (M3 `AND version = ?`, facture tenue `FOR UPDATE` — C-e431b-11).
- **Gates du passage en revue**, au dernier commit de code `cb594a7e2`, bases remises à neuf :
  `scripts/test-fast.sh --ci` (voie 2) **5561 passés, 13 ignorés** (`kesh-gate-logs/e431b/test-fast-review-2.log`) ;
  `scripts/gate-ci.sh kesh_ci_e431b` (voie 1) **5568 passés, 0 échec, 17 ignorés** (`gate-ci-review.log`). Le premier
  `test-fast` (`test-fast-review.log`, `ad3acb75e`) rendait 1 échec, la garde lexicale
  `supplier_settlement_writers_sync_or_dissolve`, corrigée par `cb594a7e2`. Frontend non touché. **E2E non lancé** (à
  la clôture ; aucune route ni écran dans cette story).
- **Manuels** : aucun changement (ni route ni écran). `user-manual.tex:3253` cite encore #823 comme limite — à la 43-3.

#### Remédiation de la revue P1 (2026-11-18, Opus 5.5)

Revue P1 (Opus, trois lentilles) : 0 CRITICAL, 0 HIGH, 3 MEDIUM distincts (A MEDIUM-1 = B-2 = E1, convergent ; B-1 =
E2 ; B-3), une douzaine de LOW. Remédiation selon les décisions de l'orchestrateur, base : tête de la 43-1a fusionnée
(`f711dd2de`, tests et fiche seulement, plus une phrase de doc-comment).
- **Rouges d'abord** (`kesh-gate-logs/e431b/p1-rouges.log`, `rouges.log`) : 8 tests neufs, écrits avant le correctif ;
  2 rouges qui confirment les MEDIUM — la vue rendait la dette de la facture détachée comme compte d'écart (`Some(9)`
  au lieu de `None`), la fiche recevait 1213 entre deux factures du groupe ; 6 verts (couverture des LOW).
  *(Annoté à la revue P2, LOW-4 : recompté depuis la source, `git grep -c '^#\[sqlx::test'` sur
  `grouped_settlements_supplier.rs`, 12 à `366568c61`, 18 à `b3b2800be` : **six** tests neufs, dont **deux** rouges
  et **quatre** verts ; « 8 » et « 6 » étaient faux.)*
- **Vue** (C-e431b-4 récrit) : les lignes de dette = celles des membres + celles des factures détachées, nommées par la
  trace du détachement (`detached_debt_lines`) ; positions 1 à n exigées, sinon `Invariant`.
- **Verrous** (C-e431b-12) : toutes les factures verrouillées par identifiant croissant avant le compte
  (`payment_batches::lock_invoices_sql` réemployée) ; la fonction unitaire n'est pas scindée.
- **Règle commune** (C-e431b-13) : `difference_accounts`, `push_difference_lines`, `create_group_entry`, appelées par
  les deux bras ; le bras client garde son comportement (ses binaires rejoués : `p1-cible-2.log`). Dette restante
  au registre (issue proposée).
- **E3** (C-e431b-14) : `LockedForSettlement.bank_ledger_account_id`, lu sous verrou.
- **LOW** : tests de `TooMany`, `CurrencyMismatch`, `HeldByPaymentBatch` (l'autre facture nommée), brouillon, annulée,
  compte d'arrondi absent, compte clos, exercice clos ; refus **après** l'écriture (transaction changée) ; délettrage ;
  détachement de la première et d'une facture du milieu ; doc-comment de test corrigé (A LOW-2) ; *Note de version*
  sans « réglable » (A LOW-6).
- **Test de la 43-1a adapté** : `a_group_without_client_settlements_is_an_invariant` (revue P2 de la 43-1a) attendait
  le message « aucun règlement client », que la 43-1b remplace ; il attend désormais l'`Invariant` du bras fournisseur
  (sa facture fabriquée pointe la ligne de banque) — comportement inchangé : jamais une réponse.
- **Mutations** (`mut/campagne.log`) : M1, M2, M4b, M5b, M7 (transaction `AND version = ?`, A LOW-5), M8 (lecture
  de tête des factures), M9 (lignes détachées ignorées) tuées ; M3 équivalente ; M4, M5, M6 sans motif (code déplacé
  ou remplacé), remplacées par M4b, M5b, M9.
- **Gates** au commit de code `b3b2800be`, bases remises à neuf : `test-fast --ci` (voie 2) **5572 passés, 13 ignorés**
  (`test-fast-p1.log`) ; `gate-ci.sh kesh_ci_e431b` (voie 1) **5579 passés, 0 échec, 17 ignorés** (`gate-ci-p1.log`).
  Frontend non touché. E2E à la clôture.
- **Non traité** : B-4 (commentaire, fait), B-5 (plan `EXPLAIN` de `locked_supplier_members`) et B-6
  (`RaceDuringUpdate` inatteignable côté fournisseur) — issues proposées au rapport.

#### Remédiation de la revue P2 (2026-11-18, Opus 5.5)

Revue P2 (Sonnet 5.5, passe complète) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW ; les trois MEDIUM de la P1 levés.
Traités selon les décisions de l'orchestrateur :
- **LOW-1** (production, rouge d'abord, `kesh-gate-logs/e431b/p2-rouges.log`) : une trace de détachement manquante
  pour la dernière facture d'un groupe sans écart faisait rendre sa dette comme compte d'écart. La vue recoupe
  désormais le nombre de lignes de dette nommées avec les factures du paiement (`group_size_at_payment`, C-e431b-15) ;
  test `a_missing_detachment_trace_makes_the_view_an_invariant`. C-e431b-4 annoté, doc-comment récrit.
- **LOW-6** (production) : `detached_debt_lines` filtre `si.status = 'cancelled'` (C-e431b-16).
- **LOW-3** : test `the_bank_ledger_is_the_one_read_under_lock` et mutation M10.
- **LOW-7** : test `a_group_cancel_on_an_entry_without_settlement_is_an_invariant` ; le test de la 43-1a adapté
  renommé `a_malformed_supplier_group_is_an_invariant_for_the_view`, assertion sur la variante (C-e431b-18).
- **LOW-4, LOW-5** : décompte de la P1 annoté (six tests neufs, deux rouges, quatre verts) ; AC6 bis, la ligne de la
  vue (§ Completion Notes), le File List et le doc-comment public de `pay_supplier_invoices_in_tx` (verrouillage de
  toutes les factures avant le compte) mis à jour.
- **LOW-8** : la campagne distingue une mutation non compilée et garde la cause de chaque échec ; M11 (délettrage
  retiré du dé-rapprochement) et M12 (côté de l'écart inversé au bras **client**) jouées.
- **LOW-2** : inscrit au registre comme limite du test d'interblocage (C-e431b-17), non traité.
- **Tests**, recomptés depuis la source (`git grep -c '^#\[sqlx::test'`) : `grouped_settlements_supplier.rs` 18 à
  `b3b2800be`, **21** à `bd5d97820` (trois neufs) ; `grouped_settlements.rs` : un test renommé, aucun ajouté.
- **Mutations** (`mut/campagne.log`, causes jointes) : M9, M10, M11, M12 tuées — M10 par la ligne de banque au compte
  de l'instantané (6 au lieu de 9), M11 par l'achat resté lettré, M12 par l'`Invariant` de la balance (« débit=20.0300,
  crédit=20.0100 »), M9 par l'`Invariant` du recoupement.
- **Gates** au commit de code `bd5d97820`, bases remises à neuf : `test-fast --ci` (voie 2) **5575 passés, 13
  ignorés** (`test-fast-p2.log`) ; `gate-ci.sh kesh_ci_e431b` (voie 1) **5582 passés, 0 échec, 17 ignorés**
  (`gate-ci-p2.log`). Frontend non touché. E2E à la clôture.

### File List

- `crates/kesh-db/src/repositories/grouped_settlements.rs`, `supplier_invoices.rs` (modifiés)
- `crates/kesh-db/tests/grouped_settlements_supplier.rs` (neuf)
- `crates/kesh-db/tests/letterings_lexical.rs`, `crates/kesh-api/tests/bank_account_closed_inventory.rs` (modifiés)
- `crates/kesh-db/tests/grouped_settlements.rs` (modifié : le test de la 43-1a adapté puis renommé,
  `a_malformed_supplier_group_is_an_invariant_for_the_view`)
- `_bmad-output/implementation-artifacts/e0190-choix-autonomes-e431b.md`, `…-review-prompt-p1.md` (neufs)
- cette fiche, `sprint-status.yaml` (modifiés)

## Change Log

- 2026-11-17 — Fiche née du découpage de la 43-1.
- 2026-11-17 (nuit) — Alignée sur la remédiation de la 43-1a : projet sans validation, différences (proposition M-A), borne
  basse de date, point de sauvegarde, signature figée, audit snake_case.
- 2026-11-17 (nuit, P3 de la 43-1a) — Les deux sites de remise à `NULL` et le fragment partagé (M2) ; la fonction
  `group_difference` commune et ses signes (M4) ; `enforce_postable = false` (L2).
- 2026-11-18 — Décision de Guy du 2026-11-17 sur la différence : deux sens admis, compte choisi par l'utilisateur
  (`difference_account_id`), refus seulement au-delà du plafond et pour un compte interdit ; ordre sens puis devise.
- 2026-11-18 (P6 ciblée des 43-2 et 43-3, CRITICAL-1) — AC2 aligné sur la décomposition définitive de C-43-25 (`e` = `d`
  arrondi au centime, `r = d − e`), ordre des contrôles, clés d'audit `gap`, `difference_account_id`,
  `rounding_difference` ; l'ancienne formule `Σ centime(total)` marquée remplacée.
- 2026-11-18 (fiches contre le contrat `t3.md`) — HIGH-1 : AC6 bis, vue d'un groupe fournisseur (`view_members`,
  `difference_account_of`), avec ses tests ; MEDIUM-1 : ordre des refus du code (plafond d'abord, dans
  `group_difference`) ; MEDIUM-2 : AC6 reprend C-e431a-9 (l'`Invariant` remplacé) et C-e431a-6 (jamais un retrait seul) ;
  LOW-2 à LOW-5 : règle tranchée, `t3.md` existant, ordre des lignes de l'écriture, test d'équilibre `r` et `e` ensemble.
- 2026-11-17 (développement, Opus 5.5) — Validation terminée (contre `t3.md`, puis passe ciblée 2 : 0 au-dessus de
  LOW) ; étape 0 (L-3) ; AC1 à AC8 et AC6 bis ; 12 tests, 6 mutations (5 tuées, 1 équivalente) ; `test-fast --ci` et
  `gate-ci` verts à `cb594a7e2` ; passage en `review`. E2E à la clôture.
- 2026-11-18 (revue P1, Opus, B/E/A : 0/0/3/≈12 ; remédiation Opus 5.5) — 3 MEDIUM remédiés (vue après détachement,
  ordre des verrous, règle de la différence commune), LOW de tests et de rédaction ; `test-fast --ci` et `gate-ci`
  verts à `b3b2800be`. Revue P2 complète à suivre.
- 2026-11-18 (clôture, Opus 5.5) — **Boucle de revue close.** Tendance (CRITICAL/HIGH/MEDIUM/LOW) : **P1** complète,
  trois lentilles B, E, A, toutes Opus — 0/0/3/≈12 (MEDIUM : vue fausse après le détachement de la dernière facture,
  convergent sur les trois lentilles ; ordre des verrous compte avant facture ; règle de la différence en double) →
  **P2** complète, Sonnet — 0/0/0/8 → **P3** ciblée sur `bd5d97820`, Haiku — 0/0/0/4. Les défauts de production de la
  P1 et le LOW-1 de la P2 ont été reproduits par un test rouge avant correction (`p1-rouges.log`, `p2-rouges.log`).
  **Reclassements** : aucun MEDIUM reclassé en dette. Le reste de la duplication (B-3), le contrôle de plan (B-5) et
  `RaceDuringUpdate` inatteignable (B-6) sont ouverts en issues ; issues de la boucle : **#1255 à #1258** (#1258 : le
  coût par vue de `group_size_at_payment`, LOW-C de la P3). P2 LOW-2 inscrit au registre comme limite du test
  d'interblocage (C-e431b-17), non traité. P3 : LOW-A traité (`debt_lines_out_of_their_positions_make_the_view_an_invariant`,
  mutation M13 tuée), LOW-B traité (C-e431b-18 annoté), **LOW-D non traité** — l'assertion de
  `a_missing_detachment_trace_makes_the_view_an_invariant` porte sur la variante seule, sans épingler le message du
  recoupement (aucune autre cause d'`Invariant` n'y précède le recoupement, d'après la P3). Avant la clôture, tête
  de la branche d'épique fusionnée (`fcce75ed3`, clôture de la 43-1a ; conflit de `sprint-status.yaml` résolu en gardant
  les deux lignes). **Gates de clôture** au dernier commit de code `920471faa`, bases remises à neuf :
  `test-fast --ci` (voie 2) **5576 passés, 13 ignorés** (`test-fast-cloture.log`) ; `gate-ci.sh kesh_ci_e431b`
  (voie 1) **5583 passés, 0 échec, 17 ignorés** (`gate-ci-cloture.log`) ; frontend complet après `npm ci` —
  `check`, `lint-i18n-ownership`, `build` verts, Vitest **3948 tests, 246 fichiers** (`front-*.log`) ; **E2E complet**
  sur `kesh_e2e_e431b`, port 3477 (`e2e-cloture.log`, script `e2e-cloture.sh`) : **437 passés, 7 échecs, 18 ignorés**
  — les sept de la KF-029 (#97), un par un : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` ; aucun échec variable. Statut `done`.
- 2026-11-18 (reprise de `main` après l'Epic 37, branche `chore/43-reprise-main`, `6f152a5f8` à `69c4d1db7`) — Bras
  fournisseur : `foreign: None` sur la banque et chaque dette ; refus `DifferenceAccountInForeignCurrency` par la règle
  commune `difference_accounts`, test rouge (`grouped_settlements_supplier.rs`,
  `a_foreign_currency_difference_account_is_refused`, rouge `MultiCurrencyDisabled`) puis vert ; registre
  C-e431b-19 (détail commun : C-e431a-17 de la 43-1a). Gates au commit `69c4d1db7`, bases remises à neuf (journaux
  `kesh-gate-logs/e43r/`) : `test-fast --ci` 6013/13 ; `gate-ci` 6030/0/17 (au second tirage, le premier rouge sur
  `disque_sature_pendant_l_ecriture`, hors de l'Epic 43, vert rejoué seul) ; frontend vert (Vitest 4353) ; E2E 458
  passés, 18 ignorés, et les sept échecs de la KF-029 seuls : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
