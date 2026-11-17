# Story 43-2 — registre des choix (développement, 2026-11-17)

Registre de la story 43-2 (clé `e432`). Les décisions de l'épique (C-43-*) et le contrat de la 43-1a
(`kesh-gate-logs/e43/t3.md`, commit `241839568`) font foi ; ce registre consigne les choix tranchés par l'agent de
développement. Chaque choix est réversible sans migration.

**C-e432-1 — Plafond de lecture de la recherche : une ligne de plus.** `find_group_candidates` lit
`GROUP_CANDIDATES_SCAN_LIMIT + 1` (501) lignes et garde les 500 premières ; `scan_limit_reached` vaut vrai seulement si
une 501ᵉ existait. Autre option : vrai dès 500 lignes lues (« plafond atteint », lecture littérale de la fiche), qui
annoncerait `hasMore` à tort pour exactement 500 factures. Réversible : une constante.

**C-e432-2 — `limit` et `exclude`.** `limit` suit `get_proposals` : absent ou `≤ 0` → 50, au-delà de 100 → 100, sans
refus. `exclude` mal formé (non entier, `≤ 0`, vide entre deux virgules) ou de plus de 50 identifiants → `400
VALIDATION_ERROR` : un client qui croit retirer une facture et ne la retire pas se tromperait en silence. Autre option :
ignorer les valeurs invalides. Réversible.

**C-e432-3 — Facture fournisseur à TTC nul non offerte.** La recherche filtre `total_amount > 0`. La liste est ainsi plus
étroite que le règlement (que la 43-1b écrira) sur ce point, écrit au module ; le principe « n'offrir jamais ce que le
`POST` refuserait » n'est pas atteint par un filtre plus étroit.

**C-e432-4 — `kind` ajouté à `ReconciliationCancellation` (kesh-db).** La réponse de `POST …/cancel` doit porter `kind`
pour **tous** les rapprochements (fiche, M5). Le dépôt ne le rendait pas, et le déduire dans la route est faux pour un
retrait seul (`invoice_id` et `supplier_invoice_id` y valent `None` quel que soit le genre). Champ additif `kind:
ReconciliationKind`, posé aux deux constructeurs de `reconciliation_cancel::cancel_in_tx`. Fichier de la 43-1a : deux
lignes, à résoudre en gardant les deux côtés à la reprise de sa branche. Autre option : relire la vue avant le geste
(deux lectures qui peuvent se contredire, la leçon de la 25-3-a-2).

**C-e432-5 — La date du jour jusqu'au geste groupé.** `settle_invoices_in_tx` reçoit `aujourd_hui` (contrat T3). Elle est
lue une fois dans `post_accept` (`state.horloge.aujourd_hui()`, avant les rejeux) et passée à `accept_once`,
`accept_batch` et `accept_one` (patron de `post_cancel_reconciliation`) ; jamais une horloge lue dans la route
(`date_du_jour_garde`). Les trois fonctions passent à huit paramètres (`#[allow(clippy::too_many_arguments)]`, usage du
dépôt).

**C-e432-6 — `count` dans `too_few` et `too_many`.** `details` porte, en plus de `reason`, le nombre de factures reçu
(additif ; la variante le porte).

**C-e432-7 — Refus « compte de tiers » par le helper partagé.** `ClaimAccount` se traduit par
`invoice_settlements::claim_account_refusal_details` — le helper qu'appelle `claim_account_failed_proposal` —, plus la
pièce (`invoiceId` / `supplierInvoiceId`, `invoiceNumber`) quand la variante en porte une. Autre option : reconstruire un
`DbError::SettlementCounterpartyIsClaimAccount` pour appeler `claim_account_failed_proposal` (détour sans gain).

**C-e432-8 — Message du lettrage (L6 de la P3) laissé tel quel.** `lettering_error_to_failed_proposal` journalise
« encaissement : lettrage de la facture impossible » pour un `Invariant` d'un groupe aussi ; sans effet sur le contrat, et
le paramétrer toucherait les trois appelants de l'unitaire pour un journal interne.

**C-e432-9 — Refus de date partagé.** `payment_before_invoice_details(tolerance_days)` est extraite de
`date_refusal_failed_proposal` (même `details` pour l'unitaire et le groupe, DRY) ; le groupe y ajoute la pièce.

**C-e432-10 — Refus sans détails quand la variante n'en porte pas.** `BANK_ACCOUNT_NOT_CONFIGURED`, `…_NOT_FOUND`,
`…_CLOSED`, `ROUNDING_ACCOUNT_NOT_CONFIGURED`, `BANK_TRANSACTION_NOT_FOUND`, `RECONCILIATION_ALREADY_RECONCILED` :
`details: null` pour un groupe (la variante de `GroupRefusal` ne porte rien ; la fiche dit « — »). L'unitaire porte
`bankAccountId`, ou `amountDue` et `transactionAmount` pour l'arrondi : différence écrite ici, non dans le contrat public.

**C-e432-11 — `gap` de la vue, en cas d'erreur de `group_difference`.** Avec `Decimal::MAX` pour plafond, la fonction ne
peut refuser ; si elle le faisait, `gap` vaudrait `null` plutôt qu'une erreur de la vue.
