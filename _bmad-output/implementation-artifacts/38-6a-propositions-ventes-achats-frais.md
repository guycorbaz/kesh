# Story 38-6a : Propositions de prestataire — ventes, achats, frais

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision de Beat (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

- **Epic** : 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`, § *38-6a*, E13, E15, C-38-8) ; décisions de Beat B3, B5,
  B6, B17 (`epics-38-40-cadrage.md`), qui priment.
- **Dépend de** : **38-3a** (colonnes de transaction de prestataire, C-38-4), **38-4** (critères et filtres des règles, C-38-6),
  **38-5** (TVA par défaut, C-38-7), **38-2** (réglages de compte prestataire, C-38-3), **38-6c1** (signal « déjà
  comptabilisée » en devise, C-38-14) et **38-6c2** (paires d'écritures comptées, B28) — voir *Écart 1* —, et l'Epic 37 (37-1 `fx_accounts_for`, 37-2
  `find_company_rate_in_tx`, 37-3a ligne en devise et contrôle central).
- **Prépare** : 38-6b (mêmes modules, variante `provider` élargie), 38-9b (écran des propositions multi-lignes).
- **Migration** : aucune (C-38-0 : une story qui en découvre une s'arrête et le dit).

## Story

En tant que **Comptable** d'une société qui encaisse et paie par PayPal,
je veux que **chaque vente, achat ou frais téléchargé reçoive une proposition d'écriture complète** — net au compte du
prestataire, brut hors TVA au compte de la règle, TVA au compte de TVA, frais au compte de frais de la devise, au cours
du mois — et l'**accepter en lot**,
afin que **mes comptes PayPal se tiennent sans saisie, sans qu'aucune écriture ne soit passée sans mon accord**.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-6a-1` | Une vente ou un achat reçoit une proposition : compte prestataire au net, contrepartie de la règle au brut hors TVA, TVA du compte (E13), frais au compte de frais de la devise. | AC-2, AC-3, AC-4 |
| `FR-38-6a-2` | Un mouvement de frais seul se propose au compte de frais de la devise. | AC-5 |
| `FR-38-6a-3` | Les propositions s'acceptent en lot (`POST /reconciliation/accept`, variante `provider`). | AC-7, AC-8 |
| `FR-38-6a-4` | Une transaction déjà comptabilisée porte `alreadyBooked` et n'est acceptée qu'avec `confirmAlreadyBooked: true`. | AC-9 |
| `FR-38-6a-5` | Le cours est le cours retenu du mois ; cours manquant → `RATE_UNAVAILABLE`. | AC-6 |
| `FR-38-6a-6` *(ajoutée)* | Une proposition qui ne peut pas s'accepter reste visible, avec le motif (`blockedBy`), au lieu de disparaître. | AC-1, AC-6 |
| `FR-38-6a-7` *(ajoutée)* | Sur un compte prestataire, l'acceptation par les anciennes variantes `rule` et `split` (au net, sans frais ni TVA) reste refusée (garde de 38-3a) ; une facture client ou fournisseur n'y est proposée et acceptée que si `fee_amount = 0` (D-P2-2) ; une transaction qui correspond à une facture mais porte des frais n'a **aucune proposition** : elle va à la file « à examiner », motif `invoice_with_fee` (D-P2-5). | AC-1, AC-8 |
| `NFR-38-6a-1` | *Pattern batch* : toute erreur par proposition en `failed[]`, codes constants ; aucune écriture d'office. | AC-7, AC-8 |
| `NFR-38-6a-2` | Revalidation sous verrou à l'acceptation. | AC-8 |
| `NFR-38-6a-3` | Code neuf hors de `reconciliation.rs`, constructeur pur testé sans base. | AC-2, AC-12 |
| `NFR-38-6a-4` *(ajoutée)* | Équilibre en devise principale exact : l'arrondi des lignes en devise se solde par une ligne d'arrondi bornée, jamais en silence. | AC-3 |
| `NFR-38-6a-5` *(ajoutée)* | Comptes non prestataires inchangés (non-régression de `get_proposals` et des quatre variantes existantes). | AC-11 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15)

- **`get_proposals`** (`crates/kesh-api/src/routes/reconciliation.rs:999`) lit les transactions `pending` non rejetées
  (`kesh-db/src/repositories/reconciliation.rs:964-988`, `auto_match_rejected_at IS NULL`), les règles actives
  (`:1049-1050`), le signal 34-4 par page (`:1069-1095`), puis **saute toute transaction hors CHF** (`:1101-1104`, commentaire
  « mono-CHF v0.1 ») ; la règle n'est essayée qu'en CHF (`:1432-1440`) ; `order_candidates` est le **dernier** geste sur
  `candidates` (`:1472`). `MAX_PROPOSALS_LIMIT` = 500 (`:74`).
- **Le candidat** a un discriminant `CandidateType { Invoice, Rule, SupplierInvoice }` (`:187-192`) et des champs `Option`
  par type (`:196-232`) ; `order_candidates` est exhaustif **sans bras `_`** (doc `:264-266`) : un variant neuf ne compile
  pas sans y être classé.
- **`ProposalNotice`** (`:153-165`) porte des champs **non optionnels** propres aux factures fournisseur
  (`supplier_invoice_id: i64`, `payment_batch_id: i64`) : il ne sert pas tel quel à un autre motif (choix C-38-6a-4).
- **Le lot** : `AcceptProposalInput` est un enum `#[serde(tag = "type")]` à quatre variantes (`:855-890`), contrôlées en
  surface par `post_accept` (`:1593-1700`) ; pré-vol de classement (`classify_batch_transaction`, `:1527`), **un** verrou de
  compte pour le lot (`with_account_lock`, `accept_once` `:1822-1853`), un point de sauvegarde par proposition
  (`accept_batch`, `:1964-2030`), répartition par `accept_one` (`:2054-2136`), `retry_with` sur 1213 et 1305 (`:1788-1815`).
  L'extracteur rend 400 en citant les valeurs de `type` admises (`AcceptBodyExtractor`, `:947-979`, message en dur `:961`).
- **`accept_one_rule`** (`:3946-4450`) est le précédent à suivre pas à pas : compte du grand livre de la banque et sa
  garde `BANK_ACCOUNT_NOT_CONFIGURED` (`:3956-4019`), règle relue active (`:4022-4040`), contrepartie ≠ banque
  (`:4055-4066`), contrepartie active et imputable (`:4068-4112`), transaction relue **strictement** `pending` sous verrou
  (`:4114-4138`), garde CHF `RECONCILIATION_CURRENCY_MISMATCH` (`:4140-4151`), rejeu de `rule_matches` (`:4153-4160`), rejeu
  du masquage par facture (`load_rule_mask`, `:4174-4187`), montant non nul (`:4188-4196`), exercice ouvert couvrant la
  date (`fiscal_years::find_open_covering_date`, `:4200-4222`), projet par défaut revalidé (`:4224-4254`),
  `journal_entries::create_in_tx(…, false)` (`:4260`) avec `PeriodLocked` et `LaterFiscalYearClosed` mappés (`:4266-4302`),
  `UPDATE bank_transactions … AND version = ?` (`:4307-4333`), `increment_applied_count_in_tx` (`:4335-4350`), audits
  `reconciliation_rule.applied` et `reconciliation.accepted` (`:4351-4422`), réponse à `invoice_id: 0` sentinelle (`:4428`).
- **Le constructeur existant** `build_journal_entry_for_counterparty` est à **deux lignes**, pur, et « signature stable
  contractée » (`crates/kesh-reconciliation/src/manual.rs:1-12`, `:68-75`) ; `build_split_journal_entry` fait N+1 lignes
  d'un même côté (`split.rs:1-20`). Ni l'un ni l'autre ne porte de ligne en devise : `NewJournalEntryLine` n'a que
  `account_id`, `debit`, `credit`, `project_id` (`kesh-db/src/entities/journal_entry.rs:203-211`) — 37-3a lui ajoute
  `foreign: Option<ForeignAmount>` (37-3a AC-2).
- **`kesh-reconciliation` ne dépend pas de `kesh-core`** (`crates/kesh-reconciliation/Cargo.toml:7-15` : `kesh-db` seul) ;
  `kesh-db` en dépend (`crates/kesh-db/Cargo.toml:8`). L'arrondi canonique est `Money::round_to_centimes`
  (`kesh-core/src/types/money.rs:66`) ; `line_vat_amount` (`kesh-core/src/accounting/vat.rs:40`).
- **Le signal « déjà comptabilisée »** : `count_already_booked_for_page` (`kesh-db/src/repositories/reconciliation_link.rs:959`)
  ne compte que les transactions que `transaction_can_be_linked_to_entry` admet — **CHF seulement** (`:535-537`) ; ailleurs
  `alreadyBooked` est `null` (`reconciliation.rs:118-124`). Le serveur ne refuse rien (`docs/api-external.md:704`).
- **Comptes de TVA** : `company_invoice_settings.default_vat_payable_account_id`, `default_vat_recoverable_account_id`
  (`migrations/20260614000001_vat_accounts_config.sql:53-56`) ; 38-5 ajoute `default_vat_acquisition_account_id` (C-38-7).
- **Les libellés de refus** : `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:1-80`, clés écrites en
  toutes lettres (`:30-31`), décompte des codes au doc-comment (`:8-15`) ; clés `reconciliation-failed-*`
  (`crates/kesh-i18n/locales/fr-CH/messages.ftl:1470-1489`), quatre locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`).
- **Registre des routes** : `post_accept` y est `Traced, Rejouee` (`crates/kesh-api/tests/audit_route_registry.rs:549`) ;
  aucune route neuve ici.
- **L'état que laisse 38-3a** (dépendance ; passe P2) : sur un compte prestataire, `get_proposals` ne propose **ni
  facture, ni règle, ni ventilation** (`TxDocuments::Nothing` et pas de règle, à côté du saut hors CHF), et `post_accept`
  refuse ces variantes par proposition en `FailedProposal` **`PROVIDER_PROPOSAL_REQUIRED`** (38-3a AC-6, code unique
  de C-38-8, D-P2-1 ; motif : une règle ou une ventilation au **net** fausse B3). 38-6a part de cet état, non du code
  d'aujourd'hui : elle lève cette garde pour la variante `provider` (point de contact de 38-3a) ; `rule` et `split`
  restent refusés ; elle lève aussi `TxDocuments::Nothing` pour lire les factures des **ventes et achats** d'un compte
  prestataire **CHF** (point de contact de l'AC-6 de 38-3a) ; les factures, selon D-P2-2 et D-P2-5 (C-38-6a-7) :
  admises seulement si `fee_amount = 0` ; avec des frais, la transaction qui correspond à une facture n'a aucune
  proposition (file, `invoice_with_fee`).
- **Audit** : `reconciliation.accepted` et `reconciliation_rule.applied` existent (`crates/kesh-api/src/audit_labels.rs:184`,
  `:192`) : aucune action neuve.

## Écarts à l'epic

1. **Dépendance à 38-6c1 et 38-6c2.** La table de l'epic faisait dépendre 38-6a de 38-3, 38-4, 38-5 seulement (corrigée à la P1) ; or `alreadyBooked` est
   `null` hors CHF tant que `transaction_can_be_linked_to_entry` n'admet que le CHF (`reconciliation_link.rs:535-537`), et
   FR-38-6a-4 vise surtout les transactions en devise de 2024 (B17). L'extension en devise est celle de **38-6c1** (AC-1,
   AC-3 ; E21 (1), C-38-14). Depuis B28, **38-6c2** compte aussi une **paire** d'écritures (vente au brut, frais à part)
   comme « déjà comptabilisée » (champ `pairCount`, **38-6c2 AC-3**) : c'est voulu (E15), et l'AC-9 le
   teste. 38-6a **dépend donc de 38-6c2** (qui suit 38-6c1, D-P2-3 : « 38-6c2 se développe avant 38-6a »).
2. **« A32 sinon »** (E15, FR-38-6a-5) : A32 qualifie une ligne reçue en `{ amount, chf }`. PayPal ne rend aucun CHF pour un
   mouvement en devise : la proposition n'a que le montant en devise, et le seul cours possible est le **cours retenu du
   mois** (`find_company_rate_in_tx`, kind `month`). Sans lui, `RATE_UNAVAILABLE` ; A32 n'a pas d'objet ici. La
   provenance de la ligne suit la qualification de 37-3a (C-37-9 complété : `afc` si égal au cours AFC du mois, sinon
   `manual`).
3. **Noms des colonnes de TVA** : l'epic écrit `default_vat_payable/recoverable` ; au code ce sont
   `default_vat_payable_account_id` et `default_vat_recoverable_account_id` (`20260614000001_vat_accounts_config.sql:53-56`).

## Choix

- **C-38-6a-1 — Où vit le code (NFR-38-6a-3).** (a) Constructeur **pur** `build_provider_entry` dans un module neuf
  `crates/kesh-reconciliation/src/provider.rs`, sans I/O, réexporté par `lib.rs` ; `kesh-reconciliation` gagne la
  dépendance `kesh-core = { path = "../kesh-core" }` (déjà dans l'arbre par `kesh-db`, aucune dépendance externe neuve).
  `manual.rs` et `split.rs` **inchangés** (signatures contractées). (b) Lecture et acceptation dans un module neuf
  `crates/kesh-api/src/routes/reconciliation_provider.rs` (`provider_candidate_for`, `accept_one_provider`). (c)
  `reconciliation.rs` ne reçoit que le **branchement** : un appel dans `get_proposals`, un variant de
  `AcceptProposalInput` et son bras dans `post_accept` et `accept_one`, le message de l'extracteur, le bras
  d'`order_candidates`.
- **C-38-6a-2 — Qu'est-ce qu'un compte prestataire.** Un compte bancaire qui porte une ligne `payment_provider_accounts`
  (E11, 38-2), lue par le dépôt de 38-2 (`payment_provider_accounts::find_by_bank_account_in_tx`, 38-2 AC-2 ;
  `fee_account_id` `NULL` = à désigner, refusé ici en `FEE_ACCOUNT_INVALID`, `details: { accountId: null, reason: "missing" }` — forme de 38-2 AC-5, dont les motifs
  `not_found`, `archived`, `not_postable`, `not_expense`, `currency_mismatch` sont repris tels quels ; `missing` est le seul motif de 38-6a) ; jamais déduit d'`account_kind` (reste `'bank'`, B16) ni de la devise. Sur un tel
  compte, `get_proposals` emprunte le chemin prestataire ; sur tout autre, **rien ne change** (saut hors CHF compris).
- **C-38-6a-3 — Le candidat `provider`.** `CandidateType::Provider` (`"provider"`), rang **1** dans `order_candidates` (celui
  de la règle, qu'il remplace sur un compte prestataire) ; au plus **un** par transaction. Champs additifs du candidat
  (`Option`, `None` hors `provider`) : `movementKind`, `providerLines` (aperçu), `rate`, `rateMonth`, `vat`
  (`{ treatment, vatRateId, ratePercent }`, 38-5), `rateSource` (`afc` | `manual`, provenance du cours retenu du mois,
  37-3a ; `null` en CHF), `feeAccountId`, `blockedBy` (`{ code, details }`). **Le DTO de transaction des propositions**
  (celui qui porte aujourd'hui date, montant, contrepartie) gagne, additifs et `null` hors compte prestataire,
  `invoiceRef` et `eventCode` (colonnes `invoice_ref`, `event_code` de 38-3a) : 38-9b les affiche (sa Q-38-9b-3 close
  par ce choix). Les champs de règle existants
  (`ruleId`, `ruleLabel`, `ruleMatchType`, `counterpartyAccountId`, `counterpartyAccountName`) sont remplis quand une règle
  s'applique. `score` = `{ total: 1.0, … 0.0 }`, convention de la règle (`reconciliation.rs:1462`).
- **C-38-6a-4 — Une proposition bloquée reste visible (FR-38-6a-6).** Quand la règle et les comptes sont connus mais que
  l'écriture ne peut pas se construire (cours, compte de TVA, compte de frais, exercice mono-devise, compte de la règle
  dans une autre devise), le candidat porte `blockedBy` avec le **même code et les mêmes `details`** que le refus
  d'acceptation (une seule fonction les construit, DRY) et `providerLines: null`. `ProposalNotice` n'est pas réutilisé
  (champs non optionnels, prémisse ci-dessus).
- **C-38-6a-5 — Montants du prestataire.** Le constructeur exige `amount = gross_amount + fee_amount` (montants signés ;
  38-8 AC-4 : `amount = gross_amount + fee_amount`, net du même signe que le brut ou nul, `|frais| ≤ |brut|` ; pour un
  genre `fee`, `fee_amount = 0` et `amount = gross_amount` ; le signe de `fee_amount` se constate sur les réponses de
  Beat, 38-8) ; sinon aucun
  candidat, `blockedBy` `PROVIDER_AMOUNTS_INCONSISTENT` (`details: { amount, grossAmount, feeAmount }`). Un `gross_amount`
  ou `fee_amount` `NULL` sur une transaction de prestataire est le même refus.
- **C-38-6a-6 — Ligne d'arrondi (NFR-38-6a-4).** Chaque ligne en devise vaut `round(montant × cours, 2)`
  (`Money::round_to_centimes`, C-37-4) ; les lignes de TVA sont en CHF (`round(TVA en devise × cours, 2)`, A13, S5). Si
  Σ débit ≠ Σ crédit en CHF, une **ligne d'arrondi** CHF solde l'écart sur les comptes de différence de change du compte
  du grand livre du prestataire (`fx_accounts_for`, 37-1 ; gain au crédit si les débits l'emportent, perte au débit
  sinon — la règle de 37-3b AC-2 bis). Bornée à `0.005 × n`, **n = le nombre de lignes dont le CHF est arrondi d'un montant en devise** — lignes en devise,
  **mais aussi** contrepartie CHF et TVA, calculées par `round(x × cours, 2)` (passe P1 : avec n = lignes en devise
  seules, une vente EUR à contrepartie CHF, quatre arrondis, pouvait s'écarter de 0.02 pour une borne de 0.01 et rendre
  un 500 sur une transaction valide) ; au-delà, `DbError::Invariant`
  (défaut du constructeur, 500, jamais une écriture). Test : vente EUR, contrepartie CHF, TVA et frais, cas construit
  où les quatre arrondis vont dans le même sens (écart de 0.02 = 0.005 × 4 : ligne d'arrondi de 0.02) ; écart simulé
  au-delà de `0.005 × n` : erreur du constructeur, aucune écriture. Visible à l'aperçu (D6 de 37-3b : jamais silencieuse).
- **C-38-6a-7 — Factures en CHF** (D-P2-2, Q-38-6a-1 option (b)). La transaction porte le **net** — une facture de
  CHF 100 payée par PayPal (frais −3.20) arrive à +96.80 : la régler au montant de la transaction laisse 3.20 ouverts
  et aucun frais au compte de frais (B3). D'où, sur un compte prestataire **en CHF** :
  - **`fee_amount = 0`** (net = brut, règlement exact) : les factures client ou fournisseur candidates sont **lues et
    proposées** comme sur un compte ordinaire, et **masquent** le candidat `provider` par la même règle
    (`rule_mask_for`, `has_strong_invoice_match` ; rejeu `load_rule_mask` à l'acceptation) : un paiement PayPal d'une
    facture saisie dans Kesh ne se passe pas en produit (faux rattachement muet, `CLAUDE.md`) ; leur acceptation
    (`invoice`, `supplier_invoice`) passe par les chemins existants ;
  - **`fee_amount ≠ 0`** (D-P2-5, qui remplace « la proposition est le candidat `provider` ») : si la transaction
    (genre `sale` ou `purchase`) **correspond à une facture** de Kesh, elle ne reçoit **aucune proposition** — ni
    facture, ni candidat `provider`, qui passerait en produit le paiement d'une facture existante (doublon) — et va à
    la file « à examiner » avec le motif **`invoice_with_fee`** (calculé, jamais stocké : `review_reason` reste
    nul ; la file et son motif sont de 38-6b, C-38-6b-4). « Correspond à une facture » = les **mêmes fonctions de
    dépôt** que sur un compte ordinaire rendent au moins une facture, quel que soit son rang, **appelées au brut**
    (`gross_amount`, le montant que le client a payé ou que l'on a payé ; jamais au net `amount`, que les lectures
    filtrent au montant — `find_unpaid_invoices_for_window` borne `amount_due` autour du montant reçu,
    `kesh-db/src/repositories/reconciliation.rs:657-663`, `find_open_supplier_invoices_for_window` exige
    `total_amount = ?`, `:825` — et
    qui ne retrouverait donc jamais une facture de CHF 100 payée à CHF 96.80) : crédit →
    `find_unpaid_invoices_for_window` et `find_invoices_slightly_overpaid_for_window` (fenêtre
    `InvoiceDateWindow::for_transaction`) ; débit → `find_open_supplier_invoices_for_window` (fenêtre
    `for_supplier_transaction`) et `find_open_supplier_invoice_in_account_batch`, au brut en valeur absolue. Une
    **seule** fonction `pub(crate)` (`invoice_match_for_provider_tx`, module neuf de C-38-6a-1), appelée par la
    proposition, par l'acceptation (rejeu sous verrou) et par la file de 38-6b (jamais recopiée) ; ses lectures sont
    **par transaction** (une à deux par vente ou achat à `fee_amount ≠ 0` d'un compte prestataire CHF), comme celles
    des factures CHF d'un compte ordinaire, et relèvent de la même exception au compteur de requêtes (AC-1) ; à
    `fee_amount = 0`, brut = net, et les lectures de `TxDocuments` suffisent (la lecture de 38-3a,
    `TxDocuments::Nothing`, est levée pour les ventes et achats d'un compte prestataire CHF). Une transaction qui ne
    correspond à aucune facture reste servie par le candidat `provider`. Une facture sans rapport dont le montant et la
    fenêtre coïncident envoie la vente à la file : c'est le sens sûr (rien d'écrit, motif visible ; « examiné » ou
    écriture manuelle, 38-6b C-38-6b-6), jamais un faux rattachement muet. Une acceptation `invoice` ou `supplier_invoice` est refusée par la garde de 38-3a
    (`PROVIDER_PROPOSAL_REQUIRED`) ; une acceptation `provider` → `PROVIDER_TX_UNDER_REVIEW` `{ reason:
    "invoice_with_fee" }` (AC-8 (4)). Le règlement d'une facture **au brut** avec ligne de frais n'est pas traité ici :
    **issue à ouvrir avec la PR** (C-38-13). Le cas se passe à la main (écriture manuelle 34-3, ou rattachement),
    écrit au manuel par 38-9b.
  En devise, aucune facture (les factures de Kesh sont en CHF, C-37-8) ; `rule` et `split` restent refusés partout.
- **C-38-6a-8 — Règle et devise.** 38-4 (AC sur la proposition) pose la règle : `first_matching_rule` reçoit
  `eligible_accounts: &HashMap<i64, Option<String>>` (comptes actifs, imputables, hors compte de la banque, avec leur
  devise) et saute une règle dont le compte n'admet pas la transaction (`rule_account_admits` : compte en X ⇔
  transaction en X ; compte CHF : toute devise) ; la suivante s'applique. 38-6a l'appelle tel quel, la carte bâtie sans
  le compte du grand livre du prestataire (patron `reconciliation.rs:1255-1259`) ; l'admission d'une contrepartie
  **CHF** pour une transaction en devise, que 38-4 laisse à 38-6a, est **retenue** (une ligne CHF à côté des lignes en
  devise, C-37-3 complété). À l'acceptation, `rule_account_admits` est rejoué (`RULE_ACCOUNT_CURRENCY_MISMATCH`,
  `details` de 38-4 : `{ ruleId, accountId, accountCurrency, transactionCurrency }`).
- **C-38-6a-9 — Ce que porte le corps d'acceptation.** `{ type: "provider", bankTransactionId, ruleId?, counterpartyAccountId?,
  confirmAlreadyBooked? }` : le serveur **recalcule** toute l'écriture sous verrou ; le client ne transmet aucune ligne.
  `ruleId` et `counterpartyAccountId` sont exigés ensemble pour une vente ou un achat, absents pour un frais seul. Un
  cours changé entre l'affichage et l'acceptation (cours saisi par un Administrateur, 37-2) donne l'écriture au cours du
  moment : écrit au manuel (38-9b) et à `api-external.md`.
- **C-38-6a-10 — Date.** `entry_date = value_date ?? booking_date` (patron `accept_one_rule`, `:4198`) ; le cours est celui
  du mois de `entry_date`. Le passage de 38-3b pose `booking_date` en Europe/Zurich.
- **C-38-6a-11 — Genres traités ici.** `sale` (montant > 0, avec règle), `purchase` (montant < 0, avec règle), `fee`
  (frais seul). Tout autre genre (`conversion`, `refund`, `withdrawal`, `deposit`, `dispute`, `other`, valeur inconnue
  `Other`) : **aucun candidat** dans cette story (38-6b les traite) ; à l'acceptation, `PROVIDER_MOVEMENT_KIND_UNSUPPORTED`.
  Une transaction qui porte `review_reason` n'a jamais de candidat ; acceptée par l'API : `PROVIDER_TX_UNDER_REVIEW`.
  Il en va de même d'une vente ou d'un achat qui correspond à une facture et porte des frais (`invoice_with_fee`,
  C-38-6a-7, motif calculé).

## Critères d'acceptation

1. **AC-1 — Le chemin prestataire de `get_proposals`.** Pour un compte bancaire prestataire (C-38-6a-2), la route
   `GET /api/v1/reconciliation/proposals?bankAccountId&limit` (forme inchangée, rôles inchangés : tout rôle qui lit
   aujourd'hui, clés d'API admises comme aujourd'hui) :
   - **lève le saut hors CHF** (`reconciliation.rs:1101-1104`) pour ce seul compte ; tout autre compte garde le saut
     (test : compte non prestataire avec une transaction EUR posée par SQL → toujours sans candidat) ;
   - lit, en plus des lectures existantes, **par page** (jamais par transaction) : la ligne de réglages du compte, le
     compte du grand livre du prestataire et sa devise, les comptes de TVA de `company_invoice_settings`, les cours
     retenus des mois présents dans la page (`find_company_rate_in_tx`, une lecture par couple devise × mois distinct),
     les genres et taux de TVA des comptes de règle concernés (38-5) ; nombre de requêtes de la route borné et testé
     (compteur ou `EXPLAIN` : il ne croît pas avec le nombre de transactions, hormis les lectures de factures CHF, par
     transaction comme sur un compte ordinaire — celles d'aujourd'hui et celles d'`invoice_match_for_provider_tx`,
     C-38-6a-7) ;
   - pour chaque transaction `pending` non rejetée de genre `sale`, `purchase` ou `fee` (C-38-6a-11) **sans**
     `review_reason` **et sans** le cas `invoice_with_fee` (C-38-6a-7 : facture correspondante et `fee_amount ≠ 0`),
     construit au plus un candidat `provider` (C-38-6a-3) par `build_provider_entry` ;
   - pose `blockedBy` (C-38-6a-4) au lieu de lignes quand l'écriture ne se construit pas : `RATE_UNAVAILABLE`
     (`{ currency, month }`), `VAT_ACCOUNT_NOT_CONFIGURED` (`{ designation: payable|recoverable|acquisition }`), `VAT_KIND_UNKNOWN`,
     `VAT_RATE_UNAVAILABLE` (38-5), `VAT_KIND_MISMATCH`,
     `FEE_ACCOUNT_INVALID` (`{ accountId, reason }`, 38-2 AC-5), `MULTI_CURRENCY_DISABLED` (`{ fiscalYearId }`, exercice de la date
     en mono-devise, 37-1 C-37-10), `RULE_ACCOUNT_CURRENCY_MISMATCH` (ne survient pas à la proposition grâce à
     C-38-6a-8 ; présent pour l'acceptation), `PROVIDER_AMOUNTS_INCONSISTENT` (C-38-6a-5), `BANK_ACCOUNT_NOT_CONFIGURED`
     (compte du grand livre absent ou inactif) ;
   - le champ `alreadyBooked` des propositions du compte est celui de 34-4, **étendu en devise par 38-6c1** et aux
     paires par **38-6c2** (`pairCount`) ; il est non nul dès qu'un de ses nombres l'est, `pairCount` compris (Écart 1) ;
   - **Tests** (passe P1 : seul le refus à l'acceptation avait un test par code) : un test HTTP de `GET …/proposals` par
     motif de `blockedBy` (cours absent, compte de TVA non désigné, genre inconnu posé par SQL, taux indisponible,
     genre de TVA incohérent avec le sens, compte de frais absent, exercice mono-devise, montants incohérents, compte du
     grand livre absent) : `providerLines: null`, code et `details` égaux à ceux du refus d'acceptation du même cas ;
   - ordre des candidates : `order_candidates` inchangé hors du bras neuf (rang 1) ; factures selon C-38-6a-7 ;
     tests : compte prestataire CHF, `fee_amount = 0`, une facture CHF à indice est proposée et masque le candidat
     `provider` ; même facture (CHF 100.00), vente à brut 100.00, frais −3.20, net 96.80 → **aucun candidat** (ni
     facture, ni `provider` : D-P2-5), la transaction reste `pending` — mutation « lecture au net » → rouge ; achat à
     frais dont le brut égale une facture fournisseur ouverte → de même ; transaction à frais sans facture
     correspondante → candidat `provider` rendu ;
     facture réglée ou annulée entre-temps → le candidat `provider` revient (la file se calcule). Le test de la file
     (`GET …/review`, motif `invoice_with_fee`) est celui de 38-6b (AC-6), qui dépend de cette story.
2. **AC-2 — Le constructeur pur (FR-38-6a-1, NFR-38-6a-3).** `kesh_reconciliation::provider::build_provider_entry(input:
   &ProviderEntryInput) -> Result<ProviderEntry, ProviderEntryError>`, sans I/O. `ProviderEntryInput` porte : la transaction
   (montant net signé, brut, frais, devise, genre, date), le compte du grand livre du prestataire et sa devise, la
   contrepartie (compte, devise ; absente pour un frais), le compte de frais (devise), la TVA résolue par 38-5
   (`DefaultVatResolution::None` ou `Resolved { treatment: VatTreatment, rate_percent, vat_rate_id }`, `default_vat_at`
   de 38-5 AC-5, et les comptes de TVA désignés), le cours (absent pour CHF), la paire de comptes de différence (gain,
   perte), la description. `ProviderEntry` porte le `NewJournalEntry` (journal `Banque`, lignes avec `foreign` de 37-3a
   pour les comptes en devise) et l'aperçu. La ventilation base/TVA est **celle de 38-5** (`split_default_vat(treatment,
   |G|, t) -> { base, vat }`, `base + vat = |G|` exactement pour `SaleIncluded`/`PurchaseIncluded`) : rien n'est
   recalculé ici. Lignes, pour une **vente** (net N > 0, brut G, frais F ≤ 0, `N = G + F`) :
   - débit compte prestataire : `N` (en devise du solde) ;
   - débit compte de frais : `|F|` si `F ≠ 0` (en devise du solde) ;
   - crédit contrepartie : `base` (`SaleIncluded`) ou `G` (sans TVA) ; en devise si la contrepartie est en devise, sinon
     en CHF (`round(base × cours, 2)`) ;
   - crédit TVA due (CHF) : `round(vat × cours, 2)` si `vat ≠ 0` (38-5 NFR-38-5-2, A13) ;
   - ligne d'arrondi si nécessaire (C-38-6a-6).
   Pour un **achat** (N < 0) : symétrique — crédit compte prestataire `|N|`, débit contrepartie `base`
   (`PurchaseIncluded`, débit impôt préalable `round(vat × cours, 2)`), débit frais `|F|` si `F ≠ 0` ; pour
   `Acquisition` (E13) : débit contrepartie `|G|` entier, puis **deux lignes CHF égales** `round(vat × cours, 2)` —
   débit impôt préalable, crédit TVA due sur acquisitions (`default_vat_acquisition_account_id`, 38-5, B18).
   `SaleIncluded` sur un achat, ou `PurchaseIncluded`/`Acquisition` sur une vente : refus `VAT_KIND_MISMATCH`
   (`{ accountId, vatTreatment, movementKind }`, code neuf, ajouté à C-38-8 par cette fiche) — jamais une TVA du mauvais
   côté. `VatKindUnknown` → `VAT_KIND_UNKNOWN`, `VatRateUnavailable` → `VAT_RATE_UNAVAILABLE` (codes de 38-5, repris
   tels quels). Montant nul → `ProviderEntryError::ZeroAmount` (le handler le rend
   `VALIDATION_ERROR`, `reason: zero_amount_transaction`, comme `:4188-4196`).
   **Tests unitaires** (sans base, table de vecteurs) : vente CHF avec et sans frais, avec TVA 8.1 et sans ; vente EUR
   avec contrepartie EUR, puis CHF ; achat avec impôt préalable ; achat avec acquisitions (deux lignes égales) ; frais
   seul ; arrondi soldé par la ligne d'arrondi (cas construit où trois arrondis divergent d'un centime) ; écart au-delà de
   la borne → erreur ; `amount ≠ gross + fee` → erreur ; chaque écriture produite passe `accounting::validate` (kesh-core)
   et, en devise, le contrôle de 37-3a (pas plus d'une devise étrangère, C-37-3 complété).
3. **AC-3 — Équilibre et devise.** Toute écriture construite a Σ débit = Σ crédit en CHF au centime ; ses lignes en
   devise portent toutes la devise du solde (une seule devise étrangère, `FX_MULTIPLE_CURRENCIES` impossible par
   construction : une contrepartie d'une autre devise est écartée par C-38-6a-8 et refusée par AC-8) ; les lignes de TVA
   et d'arrondi sont en CHF (comptes de TVA en CHF, S5). Test d'intégration : l'écriture passée par l'acceptation est
   relue (`GET /journal-entries/{id}`) avec `foreign { currency, amount, rate, rateSource }` sur les lignes en devise.
4. **AC-4 — Vente et achat de bout en bout (FR-38-6a-1).** Test d'intégration HTTP (Bearer, base `#[sqlx::test]`,
   transactions de prestataire posées par SQL sur les colonnes de 38-3a, sans réseau ni passage) : compte prestataire EUR lié à un compte du plan
   EUR, compte de frais EUR (38-2), règle `invoice_prefix` « LG » + filtre devise EUR vers un sous-compte de ventes EUR
   (38-4), genre `sale` 8.1 % sur ce compte (38-5), cours retenu de mars (37-2) : la proposition rend les quatre lignes
   attendues (montants en devise et CHF au centime, chiffres écrits dans le test) ; l'acceptation crée **une** écriture,
   la transaction passe `reconciled` avec `matched_entry_id`, `applied_count` de la règle +1. Achat USD avec acquisitions :
   deux lignes égales préalable/dû. Solde du compte du plan en devise après acceptation = Σ des nets.
5. **AC-5 — Frais seul (FR-38-6a-2).** Une transaction `movement_kind = fee` (38-8 : `amount = gross_amount`,
   `fee_amount = 0`) se propose sans règle et sans TVA : compte de frais de la devise contre compte prestataire, sens selon le signe ; `ruleId` et
   `counterpartyAccountId` absents ; acceptation sans eux. Un remboursement de frais (montant > 0) se propose au crédit
   du compte de frais.
6. **AC-6 — Cours (FR-38-6a-5).** Le cours est `find_company_rate_in_tx(conn, company_id, devise, RateKind::Month, 1er du
   mois de entry_date)` (37-2), lu dans la transaction à l'acceptation, sur le pool à la proposition ; `None` →
   `RATE_UNAVAILABLE` `{ currency, month: "AAAA-MM" }` (proposition : `blockedBy` ; lot : `failed[]`). Jamais de repli à
   1.0 (C-37-1). Transaction en CHF : aucun cours, `rate: null`. Tests : cours absent, cours saisi (provenance `manual`),
   cours AFC (`afc`).
7. **AC-7 — La variante `provider` du lot (FR-38-6a-3, NFR-38-6a-1).** `POST /api/v1/reconciliation/accept` (rôle
   Comptable+, clés d'API `read-write` admises comme aujourd'hui, route inchangée au registre : `Traced, Rejouee`, delta
   **0**) accepte `AcceptProposalInput::Provider { bank_transaction_id, rule_id: Option<i64>, counterparty_account_id:
   Option<i64>, confirm_already_booked: bool (défaut false) }` (`#[serde(rename = "provider", rename_all = "camelCase")]`).
   Pré-validation de surface dans `post_accept` (400 `VALIDATION_ERROR`, tout le lot, comme les variantes voisines) :
   identifiants > 0 quand présents, `ruleId` et `counterpartyAccountId` présents ensemble ou absents ensemble. Le message
   de `AcceptBodyExtractor` (`:961`) cite `"provider"` parmi les valeurs de `type`. Réponse : `AcceptedProposal`
   inchangé (`invoiceId: 0`, `supplierInvoiceId: null`, `journalEntryId`, `score` conventionnel). Toute erreur propre à
   la proposition est une `FailedProposal` (*Pattern batch*), identifiée par `bankTransactionId`, code constant écrit au
   site de construction.
8. **AC-8 — Revalidation sous verrou (NFR-38-6a-2, FR-38-6a-7).** `accept_one_provider`, sous le verrou de compte et le
   point de sauvegarde du lot, dans cet ordre (une seule raison rendue, la première) :
   1. le compte bancaire porte toujours ses réglages de prestataire, sinon `PROVIDER_ACCOUNT_REQUIRED` (code neuf) ;
   2. compte du grand livre du prestataire configuré et actif (`BANK_ACCOUNT_NOT_CONFIGURED`, `:3956-4019`) ;
   3. transaction relue strictement `pending` (`RECONCILIATION_TRANSACTION_NOT_PENDING`) ;
   4. `review_reason` posé → `PROVIDER_TX_UNDER_REVIEW` (`{ reason }` = la valeur stockée, forme de 38-6b) ; vente ou
      achat à `fee_amount ≠ 0` qui correspond à une facture (fonction de C-38-6a-7, relue **sous verrou**) →
      `PROVIDER_TX_UNDER_REVIEW` (`{ reason: "invoice_with_fee" }`, motif calculé) ; genre hors `sale|purchase|fee` →
      `PROVIDER_MOVEMENT_KIND_UNSUPPORTED` (`{ movementKind }`) ; montants incohérents → `PROVIDER_AMOUNTS_INCONSISTENT` ;
   5. pour `sale`/`purchase` : règle relue active (`RECONCILIATION_RULE_NOT_FOUND`), contrepartie égale à celle de la
      règle (`RECONCILIATION_RULE_MISMATCH`), contrepartie ≠ compte du prestataire (`VALIDATION_ERROR`,
      `counterparty_equals_bank_ledger`, constructeur existant), active et imputable (`ACCOUNT_NOT_FOUND`,
      `ACCOUNT_NOT_POSTABLE`), en CHF ou dans la devise de la transaction (`RULE_ACCOUNT_CURRENCY_MISMATCH`
      `{ ruleId, accountId, accountCurrency, transactionCurrency }`, comme C-38-6a-8 et 38-4), `rule_matches` rejoué (`RECONCILIATION_RULE_NO_LONGER_MATCHES`),
      masquage rejoué en CHF (`RECONCILIATION_RULE_MASKED`, `load_rule_mask`) ; `ruleId` absent pour une vente ou un
      achat → `VALIDATION_ERROR` (`reason: rule_required`) ; présent pour un frais → `VALIDATION_ERROR`
      (`reason: rule_not_expected`) ;
   6. **verrou des comptes désignés pris en tête de cette étape** — un seul appel à `lock_designated_accounts_in_tx`
      avec le compte de frais et **les trois** comptes de TVA des réglages (due, préalable, acquisitions), tous les
      candidats quel que soit le traitement de TVA, patron de la validation d'une facture (« tous les candidats, même
      la TVA due d'une facture sans TVA », `invoices.rs:1978-1982`) : le verrou ne dépend donc pas de la TVA résolue en
      (7), et l'ordre des refus reste (6) puis (7) — ; puis compte de frais revalidé **sur cet instantané** (de la
      société, actif, imputable, de charge, dans la devise du solde : `FEE_ACCOUNT_INVALID` `{ accountId, reason }`,
      motifs de 38-2 AC-5) ;
   7. TVA : `default_vat_at(conn, company_id, contrepartie, entry_date)` (38-5 AC-5, dans la transaction) —
      `VAT_KIND_UNKNOWN`, `VAT_RATE_UNAVAILABLE` ; compte de TVA requis présent, actif, imputable
      (`VAT_ACCOUNT_NOT_CONFIGURED` `{ designation }`) ; cohérence traitement/sens (`VAT_KIND_MISMATCH`). **Verrou des
      comptes désignés** (passe P1) : les comptes de TVA des réglages (dont celui d'acquisitions) et le compte de frais
      sont des comptes que l'écriture automatique **écrit sans contrôle d'imputabilité** par le dépôt
      (`create_in_tx(…, false)`) : ils se lisent par le **seul helper de verrou** de la 15-6a,
      `company_invoice_settings::lock_designated_accounts_in_tx` (`LOCK IN SHARE MODE`, `ORDER BY id`, avant
      `create_in_tx`) — l'appel unique pris en tête de (6), dont le compte de TVA requis se lit ici sur l'instantané —,
      et non par un `SELECT` nu comme la contrepartie de `accept_one_rule` (`:4073`, qui n'écrit pas
      de compte de réglage) ; test : archivage du compte d'acquisitions ou de TVA concurrent à l'acceptation → refus
      nommé, rien d'écrit (patron des tests de la 15-6a) ; le manuel (`user-manual.tex:491`, comptes désignés contrôlés
      « au moment où une écriture automatique veut l'utiliser ») est complété par 38-9b ;
   8. `alreadyBooked` relu pour **cette** transaction par les fonctions du dialogue (en devise : 38-6c1 ; **paires
      comprises** : B28, 38-6c2) : un nombre non nul — `pairCount` compris — et `confirmAlreadyBooked` faux →
      `TRANSACTION_ALREADY_BOOKED` (`details` = l'objet `alreadyBooked` de la proposition : `{ settlementCount,
      entryCount, matchedElsewhereCount, pairCount }`) — écart assumé à 34-4 AC-4, E15 ;
   9. exercice ouvert couvrant `entry_date` (`RECONCILIATION_FISCAL_YEAR_CLOSED`) ; exercice mono-devise pour une
      transaction en devise → `MULTI_CURRENCY_DISABLED` ;
   10. cours (AC-6) ; comptes de différence résolus et revalidés (`fx_accounts_for` puis `validate_fx_accounts_in_tx`,
       `FX_ACCOUNT_INVALID`) **seulement si** une ligne d'arrondi est requise ;
   11. `build_provider_entry` ; `journal_entries::create_in_tx(…, false)` — refus du dépôt mappés comme `accept_one_rule`
       (`PERIOD_LOCKED`, `LATER_FISCAL_YEAR_CLOSED`) **et** ceux de 37-3a (`ACCOUNT_REQUIRES_CURRENCY`,
       `FX_MULTIPLE_CURRENCIES`, `FX_RATE_NOT_AFC`, `MULTI_CURRENCY_DISABLED` : leur code, jamais `DATABASE_ERROR`) ;
   12. `UPDATE bank_transactions … AND version = ?` (`RECONCILIATION_ALREADY_RECONCILED`, `race_during_update`) ;
       `increment_applied_count_in_tx` si règle ; audits (AC-10).
   Sur un compte prestataire, les variantes `rule` et `split` restent refusées par proposition par la garde de 38-3a
   (`PROVIDER_PROPOSAL_REQUIRED`, 38-3a AC-6, code unique de C-38-8, D-P2-1 ; **aucun code neuf** ici) ; `invoice` et
   `supplier_invoice` (C-38-6a-7, D-P2-2) : admises si la transaction relue sous verrou a `fee_amount = 0` (chemins
   existants, inchangés), refusées par la même garde sinon (`PROVIDER_PROPOSAL_REQUIRED`). Tests : un test par refus,
   chacun « rien d'écrit » (aucune écriture, transaction `pending`, `applied_count` inchangé), dont `invoice` avec
   `fee_amount ≠ 0` ; `provider` sur une vente à `fee_amount ≠ 0` dont le **brut** correspond à une facture → `PROVIDER_TX_UNDER_REVIEW`
   `{ reason: "invoice_with_fee" }` (D-P2-5) ; la même vente sans facture correspondante → acceptée ; `invoice` avec
   `fee_amount = 0` → facture réglée ; lot mixte (une acceptée, une refusée) → `200`,
   `accepted[1]`, `failed[1]`, dans l'ordre reçu.
9. **AC-9 — Déjà comptabilisée (FR-38-6a-4, B17).** Fixture : une écriture EUR qui meut le compte du plan EUR du
   prestataire du net de la transaction (comme une écriture 2024 passée par l'API). `GET …/proposals` rend
   `alreadyBooked.entryCount ≥ 1` sur la transaction **et** le candidat `provider` ; `accept` sans
   `confirmAlreadyBooked` → `failed[]` `TRANSACTION_ALREADY_BOOKED`, rien d'écrit ; avec `true` → acceptée (l'écriture
   neuve existe, la précédente reste). Transaction sans rien aux livres : `confirmAlreadyBooked` absent → acceptée.
   **Paire (B28, passe P2)** — le cas des ventes de 2024 passées au brut avec les frais en écriture à part : fixture
   d'une vente EUR au brut (+100.00 EUR sur le compte du plan du prestataire) et d'une écriture de frais (−3.20 EUR),
   à deux jours d'écart, transaction +96.80 EUR sans écriture seule candidate → `alreadyBooked.pairCount` = 1,
   `entryCount` = 0, `alreadyBooked` non nul **par la seule paire** (38-6c2) sur la proposition ; `accept` sans
   `confirmAlreadyBooked` → `TRANSACTION_ALREADY_BOOKED`, rien d'écrit, `details` égal à l'`alreadyBooked` de la
   proposition, `pairCount` compris ; transaction déjà rattachée à ce groupe (38-6c2) → plus `pending`, aucune
   proposition.
10. **AC-10 — Audit.** `reconciliation.accepted` (existant) avec `details` : `type: "provider"`, `bank_transaction_id`,
    `movement_kind`, `rule_id` (ou `null`), `counterparty_account_id` (ou `null`), `fee_account_id`, `vat_kind` (ou
    `null`), `rate` (ou `null`), `currency`, `amount`, `gross_amount`, `fee_amount`, `journal_entry_id`, `entry_date`,
    `already_booked_confirmed` (bool) ; `reconciliation_rule.applied` (existant, mêmes champs qu'aujourd'hui) quand une
    règle s'applique ; la clé d'API est nommée (`NewAuditLogEntry::for_actor`). Aucune action neuve à
    `audit_labels.rs` (prémisse). Test : relecture des deux entrées.
11. **AC-11 — Non-régression (NFR-38-6a-5).** Tests existants de réconciliation (`reconciliation_e2e.rs` et voisins) verts
    **sans modification** ; un compte non prestataire avec les mêmes données CHF donne exactement la réponse d'avant
    (candidats, ordre, `alreadyBooked`) ; le test d'égalité 34-4 (`s34_4_ac8_…`) vert.
12. **AC-12 — Textes, i18n, documentation.** Codes neufs posés dans `failed[]` ou `blockedBy` par cette story :
    `RATE_UNAVAILABLE`, `VAT_ACCOUNT_NOT_CONFIGURED`, `VAT_KIND_MISMATCH`, `VAT_KIND_UNKNOWN` et `VAT_RATE_UNAVAILABLE`
    (38-5), `RULE_ACCOUNT_CURRENCY_MISMATCH` (38-4 en crée la clé ; libellé repris),
    `FEE_ACCOUNT_INVALID`, `PROVIDER_TX_UNDER_REVIEW`, `PROVIDER_MOVEMENT_KIND_UNSUPPORTED`, `PROVIDER_AMOUNTS_INCONSISTENT`,
    `PROVIDER_ACCOUNT_REQUIRED`, `TRANSACTION_ALREADY_BOOKED`, `MULTI_CURRENCY_DISABLED`
    (plus ceux de 37-3a remontés par le dépôt). Chacun : libellé dans `failed-proposal-label.ts` (clé
    `reconciliation-failed-*` écrite en toutes lettres, quatre locales, `details` lus pour `RATE_UNAVAILABLE` — devise et
    mois —, `VAT_ACCOUNT_NOT_CONFIGURED` — désignation —, `TRANSACTION_ALREADY_BOOKED` — nombres,
    `PROVIDER_TX_UNDER_REVIEW` — `reason` : la clé `reconciliation-review-reason-invoice-with-fee` est **créée ici**,
    aux quatre locales, puisque 38-6a se développe avant 38-6b, qui la reprend (38-6b AC-10) et pose celles des autres
    motifs ; textes : FR « Facture de Kesh avec frais : à régler à la main », DE « Rechnung in Kesh mit Gebühren: von
    Hand zu begleichen », IT « Fattura di Kesh con commissioni: da regolare a mano », EN « Kesh invoice with fees:
    settle by hand » (test `i18n-keys.test.ts` : la clé existe aux quatre locales) ; un `reason` sans clé encore — motifs stockés de 38-3a — garde le libellé générique du code) ; décompte des codes du
    doc-comment recompté au `grep` (`grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation*.rs | sort -u`,
    le fichier neuf compris) ; `reconciliation.types.ts` : `CandidateType` + `'provider'`, champs neufs (`rateSource`
    compris), `blockedBy`, `invoiceRef` et `eventCode` du DTO de transaction (C-38-6a-3 ; test HTTP : relus sur une
    transaction de prestataire, `null` sur un compte non prestataire) ;
    `i18n-keys.test.ts` vert. `docs/api-external.md` § *Accepter des propositions* : variante `provider`, son corps, ses
    refus, la ligne d'arrondi, le recalcul au moment de l'acceptation (C-38-6a-9), `alreadyBooked` exigeant la
    confirmation sur un compte prestataire. Manuel utilisateur : **38-9b** (écran) ; cette story n'ajoute pas de contrôle
    visible (aucune capture). **MCP** : aucun outil (E23) — les écritures passées se lisent par `kesh_ecritures`
    (`mcp/outils/ecritures.rs`), aucune donnée de prestataire n'est exposée.

## Tâches

- [ ] T1 — `kesh-reconciliation` : dépendance `kesh-core` ; module `provider.rs` (`ProviderEntryInput`, `ProviderEntry`,
      `ProviderEntryError`, `build_provider_entry`), doc-comments, tests de vecteurs (AC-2, AC-3).
- [ ] T2 — `kesh-api/src/routes/reconciliation_provider.rs` : lecture par page des réglages, cours, TVA, comptes de
      frais ; `provider_candidate_for` ; fonction unique de construction des refus (`blockedBy` et `FailedProposal`)
      (AC-1, C-38-6a-4).
- [ ] T3 — `reconciliation.rs` : branchement dans `get_proposals` (saut hors CHF levé pour un compte prestataire,
      filtre de devise des comptes de règle, masquage CHF) ; `CandidateType::Provider` et son bras d'`order_candidates` ;
      champs additifs du candidat (AC-1, AC-3).
- [ ] T4 — Variante `Provider` : `AcceptProposalInput`, `bank_transaction_id()`, pré-validation de `post_accept`,
      message de l'extracteur, bras d'`accept_one` → `accept_one_provider` ; la garde de 38-3a
      (`PROVIDER_PROPOSAL_REQUIRED`) laissée sur `rule` et `split`, levée pour les factures si `fee_amount = 0`
      (D-P2-2, C-38-6a-7) ; lecture des factures CHF dans `get_proposals` pour tout compte prestataire CHF, et
      `invoice_match_for_provider_tx` (lectures au **brut** ; D-P2-5 : aucune proposition si frais et facture
      correspondante) (AC-1, AC-7,
      AC-8).
- [ ] T5 — `accept_one_provider` : étapes 1 à 12 de l'AC-8, audits de l'AC-10 ; mapping des refus du dépôt (37-3a).
- [ ] T6 — Tests d'intégration HTTP (AC-4, AC-5, AC-6, AC-8, AC-9, AC-10, AC-11), compteur de requêtes (AC-1).
- [ ] T7 — Frontend : types, libellés et i18n (AC-12) ; `api-external.md`.
- [ ] T8 — Gate complet (`scripts/test-fast.sh`, frontend, E2E au dernier commit de code : la suite existante, aucune spec
      neuve ici — 38-9b porte l'E2E).

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Ajouté** : les propositions de rattachement des ventes, achats et frais des prestataires de paiement (PayPal) :
  Kesh propose, l'utilisateur accepte ; rien n'est écrit d'office. Les cas avec frais se passent encore à la main
  (écriture manuelle ou rattachement).

**Compteurs** (`LIB_ROUTES`, codes d'erreur du doc-comment, registres, `Cargo.lock`) : recomptés depuis la source après
chaque reprise de `main`.

## Dev Notes

- **Pourquoi un module à part** : `reconciliation.rs` fait 7 439 lignes ; le patron des propositions par règle s'y lit
  mais ne s'y recopie pas. Ce qui se partage avec `accept_one_rule` (mapping `PeriodLocked`, `LaterFiscalYearClosed`,
  `non_postable_failed_proposal`, `counterparty_is_bank_ledger_failed_proposal`, `load_rule_mask`) devient `pub(crate)`
  et s'appelle, ne se recopie pas (DRY).
- **Verrous** : ordre de `accept_one_rule` (compte bancaire nommé, transaction, exercice `FOR UPDATE` par
  `find_open_covering_date`, puis `create_in_tx` qui prend `companies` en partage, 37-3a AC-4), **plus** les comptes
  désignés en partage **avant** l'exercice (AC-8 (7) avant (9)), comme la validation d'une facture
  (`kesh-db/src/repositories/invoices.rs:1980-1988` : comptes désignés, puis `fiscal_years`) — passe P2. Un **seul**
  appel à `lock_designated_accounts_in_tx`, en tête de l'AC-8 (6), avec le compte de frais et les trois comptes de TVA
  des réglages (tous les candidats, la TVA n'étant résolue qu'en (7)) : les contrôles de
  l'AC-8 (6) et (7) se font sur cet instantané, non sur une lecture nue qui le précéderait. La lecture du cours et
  des comptes de différence se fait dans la transaction ; cycle éventuel avec la bascule de 37-1 défendu par le rejeu
  existant de `post_accept` (`retry_with`, 1213 et 1305) — à écrire au Change Log.
- **Signe des frais** : à confirmer avec 38-3a/38-8 (C-38-4 ne fixe pas le signe de `fee_amount`) ; le constructeur
  l'écrit en invariant (C-38-6a-5) et le teste.
- **Performance** : la page vaut au plus 500 transactions ; aucune lecture par transaction hormis celles des factures CHF
  (celles d'un compte ordinaire, et celles d'`invoice_match_for_provider_tx` au brut, C-38-6a-7). Mesure écrite au Change Log (page de 500 transactions de prestataire, 3 devises).
- **Pourquoi `invoice_with_fee` n'est pas stocké** (D-P2-5) : la facture peut être saisie, réglée ou annulée après le
  téléchargement ; un motif écrit au passage serait périmé (même raison que la file de 38-6b). 38-6a ne livre pas la
  file : elle ne livre que la fonction et l'absence de proposition ; 38-6b lit la même fonction.
- **Piège** : `order_candidates` est exhaustif ; `CandidateType` sert aussi au front (`reconciliation.types.ts`) — le
  variant ajouté sans le bras TS fait échouer `npm run check`.
- **Piège** : `alreadyBooked` exige la version en devise de 38-6c1 et `pairCount` de 38-6c2 ; tant qu'elles ne sont
  pas fusionnées, AC-9 ne peut pas passer (dépendance, Écart 1).
- **Références** : epic E11, E13, E14, E15, C-38-3, C-38-4, C-38-6, C-38-7, C-38-8, C-38-14 ; 37-3a AC-2 (`ForeignAmount`),
  AC-4 (qualification) ; 37-3b AC-2 bis (ligne de différence, gain au crédit / perte au débit) ; 34-4 (signal) ; 15-5b,
  15-6d (gardes de la règle) ; 30-1b (masquage).

## Questions ouvertes

- ~~**Q-38-6a-1 — Factures CHF sur un compte prestataire**~~ *(close, D-P2-2 : option (b) ; C-38-6a-7, AC-1, AC-8 ;
  règlement au brut avec frais à tracer en issue, C-38-13)*. 38-3a les exclut ; la rédaction de 38-6a les réadmettait sans dire comment un
  règlement au **net** respecte B3. Options : (a) **garder l'exclusion de 38-3a** — la facture masque seulement le
  candidat `provider` (aucun produit compté deux fois) ; l'utilisateur passe le paiement par une écriture manuelle
  (34-3) ou le rattachement ; (b) **réadmettre les factures quand `fee_amount = 0`** (net = brut, règlement exact), et
  garder l'exclusion sinon ; (c) un règlement **au brut** avec ligne de frais (variante neuve, hors périmètre de cette
  fiche). Recommandation de la passe : (b), qui ne fausse rien et couvre le paiement sans frais (virement PayPal
  entre particuliers) ; le cas avec frais reste à la main, écrit au manuel par 38-9b.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — validation P5 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW ; remédié. P5-1 (LOW, inter-fiches)
  AC-12 : textes des quatre locales de la clé `reconciliation-review-reason-invoice-with-fee` ajoutés. Vérifié au code
  (lignes lues) : `reconciliation.rs:646-663` (`amount_due BETWEEN`), `:813-832` (`total_amount = ?`), `:872` (lot de
  paiement par `amount`), `for_transaction`/`for_supplier_transaction` (`:504`, `:520`), `invoices.rs:1972-1990`,
  `company_invoice_settings.rs:921-946` ; colonnes `default_vat_payable/recoverable_account_id` + acquisitions (38-5) =
  trois comptes de TVA ; `gross_amount` (38-3a). Verrou en tête de (6), lectures au brut par transaction et clé créée ici :
  cohérents entre C-38-6a-7, AC-1, AC-8, AC-12, T4, Dev Notes et 38-6b AC-6/AC-10. Axes non exercés : le reste de la fiche.
- 2026-11-15 — validation P4 ciblée (Opus) : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 1 LOW ; tous remédiés dans la fiche.
  P4-1 (HIGH) « correspond à une facture » = la lecture d'un compte ordinaire, qui filtre au **montant** de la
  transaction (`kesh-db/src/repositories/reconciliation.rs:657-663`, `:825`, lus) : au net, une facture de CHF 100
  payée 96.80 n'est jamais retrouvée, `invoice_with_fee` ne se lève jamais et le candidat `provider` passe le doublon
  que D-P2-5 devait empêcher → lectures **au brut**, fonctions de dépôt nommées, test à mutation « au net » (C-38-6a-7,
  AC-1, AC-8, T4). P4-2 (MEDIUM) « lue par page » contredisait « même lecture » (fonctions par transaction) et
  l'exception du compteur de l'AC-1 → par transaction, dans l'exception (C-38-6a-7, AC-1, Dev Notes). P4-3 (MEDIUM)
  AC-12 reprenait la clé `reconciliation-review-reason-invoice-with-fee` de 38-6b, développée **après** 38-6a → créée
  ici. P4-4 (MEDIUM) AC-8 (6) se lisait sur le verrou de (7), qui ne peut se prendre qu'après la TVA résolue en (7) :
  ordre des refus circulaire → verrou en tête de (6) sur tous les candidats (patron `invoices.rs:1978-1982`, lu),
  (7) et Dev Notes alignés. P4-5 (LOW, point inter-fiches de 38-3a) : levée de `TxDocuments::Nothing` pour les ventes
  et achats d'un compte prestataire CHF écrite à la prémisse. Lignes lues : `routes/reconciliation.rs:335-345`,
  `:399-545`, `:1050-1215` ; `company_invoice_settings.rs:921-946`. Inter-fiches signalés : 38-6b (lecture au brut,
  clé reprise), 38-9b (clé posée par 38-6a).
- 2026-11-15 — validation P3 (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 2 LOW ; remédiés dans la fiche sauf inter-fiches
  (38-6b C-38-6b-4/AC-6, 38-9b, epic C-38-8, appliqués ou proposés par l'orchestrateur) : P3-1 (MEDIUM) D-P2-5 — une vente
  à frais qui correspond à une facture recevait le candidat `provider` (doublon du produit) → aucune proposition, motif
  calculé `invoice_with_fee`, fonction unique `invoice_match_for_provider_tx` (proposition, acceptation sous verrou,
  file de 38-6b), `PROVIDER_TX_UNDER_REVIEW` à l'acceptation `provider` (FR-38-6a-7, C-38-6a-7, C-38-6a-11, AC-1, AC-8
  (4), AC-12, T4) ; P3-2 (LOW) renvoi « AC-14 de 38-6c » → 38-6c2 AC-3 (Écart 1) ; P3-3 (LOW) AC-8 (6) : contrôle du
  compte de frais sur l'instantané du verrou de (7) (Dev Notes, ordre `lock_designated_accounts_in_tx` avant
  `fiscal_years`, relu à `invoices.rs:1978-1988` : exact). Prémisses de P2 relues au code : exactes. Renvois à
  38-6c1/38-6c2 relus : AC-1 et AC-3 de 38-6c1 (règle en devise, lecture par page), AC-3 de 38-6c2 (`pairCount`) :
  exacts. Taille : cinq modules, inchangée.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : D-P2-1 — code unique `PROVIDER_PROPOSAL_REQUIRED` (38-3a),
  `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL` retiré (prémisse, AC-8, T4) ; D-P2-2 — Q-38-6a-1 close, option (b) :
  factures proposées et acceptées sur un compte prestataire CHF seulement si `fee_amount = 0`, sinon candidat
  `provider` et refus `PROVIDER_PROPOSAL_REQUIRED` ; issue « règlement au brut avec frais » (C-38-13) (FR-38-6a-7,
  C-38-6a-7, AC-1, AC-8) ; `alreadyBooked` non nul dès que `pairCount` l'est, `TRANSACTION_ALREADY_BOOKED` porte
  `pairCount` (AC-1, AC-8 (8), AC-9) ; D-P2-3 — dépendance à 38-6c1 (devise) et 38-6c2 (paires), renvois à 38-6c
  répartis. Numéros d'AC cités de 38-6c : ceux d'avant découpage, à relire contre 38-6c1/38-6c2.
- 2026-11-15 — validation P2 (Opus) : 0 CRITICAL, 1 HIGH, 1 MEDIUM, 1 LOW ; remédiés dans la fiche sauf P2-1 en partie
  (Q-38-6a-1 à l'orchestrateur ; inter-fiches : C-38-8 de l'epic, 38-6c) : P2-1 (HIGH) la fiche ignorait la garde de
  38-3a AC-6 dont elle dépend — code neuf `PROVIDER_PROPOSAL_REQUIRED` doublant `PROVIDER_TX_REQUIRES_PROVIDER_PROPOSAL`,
  `split` réadmis au net, factures CHF réadmises et réglées au net contre B3 → prémisse ajoutée, code retiré (FR-38-6a-7,
  AC-8, AC-12, T4), `split` refusé, factures en Q-38-6a-1 (C-38-6a-7, AC-1) ; P2-2 (MEDIUM) B28 : la paire d'écritures
  (vente au brut, frais à part), cas premier de 2024, comptée « déjà comptabilisée » par 38-6c AC-14, absente de
  l'AC-8 (8) et de l'AC-9 → `details` = l'objet `alreadyBooked`, test de la paire, Écart 1 ; P2-3 (LOW) `details` de
  `PROVIDER_TX_UNDER_REVIEW` (forme de 38-6b) ; ordre des verrous des comptes désignés avant l'exercice et lecture
  unique par le helper (Dev Notes). Prémisses relues au code (`reconciliation.rs`, `reconciliation_link.rs`,
  `company_invoice_settings.rs:921`, `manual.rs`, `journal_entry.rs:203-211`, Cargo, migrations, `api-external.md:704`,
  `failed-proposal-label.ts`, `messages.ftl`) : exactes. Taille inchangée (cinq modules).
- 2026-11-15 — alignement inter-fiches P1 (Opus) : C-38-6a-3 et AC-12 — le DTO de transaction des propositions porte
  `invoiceRef` et `eventCode`, le candidat `rateSource` (liste inter-fiches, point 38-6a) ; découpage D-P1-1 : renvois à
  38-3 portés sur 38-3a (colonnes, dépendance) ou 38-3b (passage, `booking_date` à Europe/Zurich) ; AC-4 : transactions
  de prestataire posées par SQL sur les colonnes de 38-3a (le dépôt qui les insère est le passage de 38-3b, dont 38-6a
  ne dépend pas) ; Écart 1 : la table de l'epic est dite corrigée.
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW ; remédiés dans la fiche sauf les
  inter-fiches P1-5 et P1-6 : P1-1 (MEDIUM) borne de la ligne d'arrondi `0.005 × n` comptée sur les seules lignes en
  devise, trop étroite quand la contrepartie CHF et la TVA sont arrondies aussi (C-38-6a-6) ; P1-2 (MEDIUM)
  `FEE_ACCOUNT_INVALID` : `details` `{ feeAccountId, reason }` ici contre `{ accountId, reason }` à 38-2 AC-5 (C-38-6a-2,
  AC-1, AC-8 (6)) ; P1-3 (MEDIUM) comptes de TVA, d'acquisitions et de frais lus sans le helper de verrou des comptes
  désignés (AC-8 (7)) ; P1-4 (LOW) `details` de `RULE_ACCOUNT_CURRENCY_MISMATCH` sans `accountId` à l'AC-8 (5) ; P1-5
  (LOW, inter-fiches) table de l'epic : 38-6a dépend aussi de 38-6c, codes neufs de C-38-8, totaux FR/NFR ; P1-6 (LOW,
  inter-fiches) 38-2 AC-5 : motif `missing` ; P1-7 (LOW) tests de `blockedBy` absents (AC-1). Taille : cinq modules
  (`kesh-reconciliation`, `kesh-api`, `kesh-db`, `kesh-i18n`, frontend) — limite, pas de découpage proposé.
- 2026-11-15 — rédaction (Opus) : 12 AC, 7 FR (5 de l'epic + 2 ajoutées), 5 NFR (3 + 2) ; aucune migration ; écarts :
  dépendance à 38-6c, « A32 sinon » sans objet, noms des colonnes de TVA ; codes neufs ajoutés à C-38-8 :
  `VAT_KIND_MISMATCH`, `PROVIDER_MOVEMENT_KIND_UNSUPPORTED`, `PROVIDER_AMOUNTS_INCONSISTENT`, `PROVIDER_ACCOUNT_REQUIRED`,
  `PROVIDER_PROPOSAL_REQUIRED` ; alignée le même jour sur les fiches sœurs écrites en parallèle : 38-2
  (`find_by_bank_account_in_tx`), 38-4 (`rule_account_admits`, `eligible_accounts`), 38-5 (`default_vat_at`,
  `split_default_vat`, `VAT_KIND_UNKNOWN`, `VAT_RATE_UNAVAILABLE`), 38-8 (montants d'un frais seul, signe des frais).
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : section *Note de version* ajoutée ;
  compteurs « recomptés après chaque reprise de `main` ». Aucun PDF ni CHANGELOG commité dans cette fiche.
