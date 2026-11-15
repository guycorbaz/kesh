# Story 40-3 : versements Stripe — date d'arrivée, écriture datée de l'arrivée, rattachement à la ligne UBS, échecs

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/40-stripe`, numéros de ligne relevés au commit `a2b1da9c`. Epic 40
(`planning-artifacts/epic-40-stripe.md`) : G1, G10, G11, G12 (créance au 31 décembre), G15, G18 ; contrats C-40-0,
**C-40-2 (a, b)**, **C-40-5**, C-40-6 ; décisions de Beat B13, B17, B20. **Dépend de** 40-2 (genre `withdrawal`,
`value_date` = date d'arrivée, lien d'un échec de versement), 40-1 (`ProviderMovement.value_date`, sa validation), et du
socle : 38-3b (passage, correspondance des colonnes), 38-6a (C-38-6a-10 : date de l'écriture), **38-6b** (C-38-6b-3 :
retrait, voies a et b ; file, AC-6), **38-6c1** (rattachement en devise et en lot), 34-11b (virement entre ses comptes,
livrée). **Suivie de** 40-4 (versements en route) et 40-6 (écrans). **Aucune migration** (`bank_transactions.value_date`
existe).*

*Modules : `kesh-api` (passage, propositions et file), `kesh-db` (écriture de `value_date`, suggestion du compte de
l'autre côté, file), `kesh-reconciliation` (date passée au constructeur du virement) — trois (≤ 5).*

## Story

En tant que comptable d'une société qui encaisse par Stripe, je veux qu'un versement Stripe se propose comme un
**virement** de « Débiteurs Stripe » vers ma banque **daté du jour où il arrive en banque**, que la ligne du relevé UBS
se rattache à la même écriture, que mes versements de 2024 déjà passés se rattachent en lot, et qu'un versement échoué
ou annulé attende à la file « à examiner » relié au versement d'origine, afin que la créance sur Stripe au 31 décembre
soit juste et que rien ne soit compté deux fois ni écrit sans moi.

## Ce que le code dit (prémisses vérifiées au commit `a2b1da9c`)

- `bank_transactions.value_date DATE NULL` existe (`crates/kesh-db/migrations/20260504000001_bank_imports.sql:61`),
  porté par l'entité (`crates/kesh-db/src/entities/bank_transaction.rs:87`) et écrit à l'import
  (`crates/kesh-db/src/repositories/bank_imports.rs:152, 161`).
- **Une seule « date de paiement »** : `kesh_db::repositories::reconciliation::payment_date(tx) = tx.value_date
  .unwrap_or(tx.booking_date)` (`crates/kesh-db/src/repositories/reconciliation.rs:277`) ; l'acceptation par règle date
  l'écriture ainsi (`crates/kesh-api/src/routes/reconciliation.rs:4198`), comme le partage en lot (`accept_one_split`, `:3748`), l'acceptation manuelle
  (`post_manual_once`, `:5084`) et le partage unitaire (`post_split_once`, `:5566`). 38-6a en fait la règle de toute proposition `provider` (C-38-6a-10 : `entry_date =
  value_date ?? booking_date`, cours du mois de `entry_date`).
- Les écritures candidates d'un rattachement (`GET …/entry-candidates`, 34-11b) **ne filtrent aucune date** ; elles se
  trient par proximité à la date de paiement, « liée en paramètre » (`crates/kesh-db/src/repositories/reconciliation_link.rs:626`,
  `ENTRY_CANDIDATES_LIMIT = 50`, `:526`) ; `docs/api-external.md:704` : « aucun ne regarde de date ».
- Socle à venir (Epic 38 non développé) : le passage de 38-3b écrit `value_date NULL` (38-3b AC-4, correspondance) ; la
  suggestion du compte de l'autre côté d'un retrait (38-6b C-38-6b-3 b) part de `booking_date`.

## Exigences

- `FR-40-3-1` — Un versement porte sa date d'arrivée (`value_date`) ; le mouvement garde sa date de sortie → AC-1.
- `FR-40-3-2` — Sa proposition crée l'écriture banque ↔ Débiteurs Stripe datée de l'arrivée, ou rattache l'écriture déjà
  passée côté banque → AC-2, AC-3, AC-5.
- `FR-40-3-3` — La ligne du relevé UBS se rattache à la même écriture (34-11b) → AC-4.
- `FR-40-3-4` — Un versement échoué ou annulé va à la file « à examiner », relié au versement d'origine → AC-7.
- `FR-40-3-5` *(ajoutée)* — Les versements de 2024 déjà passés par l'API se rattachent en lot à leurs écritures (B17) →
  AC-6.
- `NFR-40-3-1` — *Pattern batch* ; aucune écriture d'office → AC-2, AC-6.
- `NFR-40-3-2` — Retraits sans `value_date` (PayPal, Wise) inchangés : leurs tests restent verts sans retouche → AC-8.
- `NFR-40-3-3` — Aucune migration (`value_date` existe) → AC-9.

## Écarts à l'epic

1. **C-40-2 (b) est déjà à moitié dans le socle.** « Date de l'écriture créée = `value_date` si posée » est la règle
   générale de 38-6a (C-38-6a-10) et du code (`payment_date`, ci-dessus) : 40-3 ne la réécrit pas, il **vérifie** que le
   virement de 38-6b (voie b) la lit par la même fonction, et l'impose s'il ne le fait pas (AC-2). Seule la fenêtre de
   suggestion (C-38-6b-3) change réellement (AC-3).
2. **`relatedBankTransactionId` de la file** (38-6b AC-6) : la fiche 38-6b ne le rend que pour une moitié de conversion
   (`conversion_pair_under_review`). FR-40-3-4 (« relié au versement d'origine ») l'exige pour un échec de versement :
   champ rempli, **additif**, pour toute transaction de la file dont `related_provider_tx_id` désigne une transaction du
   même compte (AC-7).
3. **`value_date` posée par le classement** (40-2 AC-2, FR-40-2-7), et non « par le connecteur » de cette story : 40-3
   l'écrit en base et s'en sert.

## Critères d'acceptation

1. **AC-1 — `value_date` écrite par le passage (C-40-2 a, FR-40-3-1).** La correspondance `ProviderMovement` →
   `bank_transactions` du passage (38-3b AC-4, une seule fonction) écrit `value_date = movement.value_date` (au lieu de
   `NULL`) ; `booking_date` reste la date Zurich de `booked_at` (sortie du solde Stripe), `booked_at` l'instant `created` ;
   la complétude (38-3b AC-7) lit `booked_at`, **inchangée**. Empreinte `file_hash` de l'en-tête (38-3b AC-4,
   « enregistrements normalisés ») : `value_date` y entre **seulement si elle est posée** — l'empreinte d'un en-tête PayPal
   est inchangée (test : même empreinte avant et après pour une page PayPal fixée). Un mouvement connu n'est jamais
   réécrit (E6) : une `value_date` qui changerait chez Stripe (versement replanifié) ne met pas la ligne à jour
   (`tracing::warn!` sans montant, compté `known`, patron « statut tardif » de 38-3b). Tests : versement de la fixture
   (`created` 1794710640, `arrival_date` 1794787200) → `booking_date` = date Zurich de `created`, `value_date` = date UTC
   d'`arrival_date` ; vente → `value_date NULL`.
2. **AC-2 — Écriture du versement datée de l'arrivée (C-40-2 b, G10, FR-40-3-2).** Proposition `provider` d'un
   `withdrawal` (38-6b C-38-6b-3), **voies (a) et (b) inchangées** sauf la date :
   - voie (b) « créer » : l'écriture `D <compte du grand livre du compte cible> / C <compte du plan du compte Stripe>`
     (frais éventuels au compte de frais) est datée de **`payment_date(tx)`** — la fonction de
     `repositories/reconciliation.rs:277`, appelée une fois, passée à `build_provider_transfer` (`kesh-reconciliation`,
     38-6b AC-9) comme `entry_date` explicite, jamais recalculée dans le constructeur ; même date à l'aperçu
     (`providerLines`) et à l'acceptation ; cours (compte en devise) du mois de cette date ; exercice ouvert et verrou de
     période contrôlés **sur cette date** (refus existants de 38-6a AC-8 (9) : `RECONCILIATION_FISCAL_YEAR_CLOSED`,
     verrou de période ; en `failed[]`, *Pattern batch*). Si 38-6b lit déjà `payment_date`, rien n'est changé et le test
     le fige (Écart 1).
   - voie (a) « rattacher » : aucune date lue (`link_entry_in_tx`, 38-6c1 AC-11) ; une écriture côté banque datée d'un
     autre jour se rattache (test).
   - **Créance au 31 décembre** (G10, G12) : test — versement CHF −500.00 sorti le 31.12.2026 à 15 h (Zurich), arrivée
     le 4.1.2027, exercices 2026 et 2027 ouverts : écriture `D 1020 / C 1101` datée du **4.1.2027** ; solde de 1101 au
     31.12.2026 = 500.00 de plus qu'au lendemain de l'arrivée ; même versement, exercice 2027 absent → `failed[]`
     `RECONCILIATION_FISCAL_YEAR_CLOSED` (code de 38-6a AC-8 (9), posé aux sites d'acceptation,
     `routes/reconciliation.rs:3771`, `:4207` ; le test fige ce code, aucune alternative), rien d'écrit.
   - Audit : `reconciliation.accepted` existant (`type: provider`, `movement_kind: withdrawal`) ; aucun champ neuf.
3. **AC-3 — Suggestion du compte de l'autre côté (C-40-5, C-38-6b-3 amendé).** La lecture qui suggère
   `transferBankAccountId` (38-6b AC-1, une requête par page) part de **`payment_date`** de la transaction du
   prestataire, calculée en SQL `COALESCE(value_date, booking_date)` dans **une** expression nommée du module (la lecture
   porte sur une page de transactions : la date ne peut pas être liée en paramètre comme le fait `reconciliation_link.rs:626`,
   dont le commentaire interdit au contraire de réécrire ce `COALESCE` — c'est donc la **première** copie SQL de
   `payment_date`, et un test de concordance la tient : sur les cas `value_date` nulle et posée, l'expression SQL rend la
   même date que `payment_date`, `repositories/reconciliation.rs:277`) : retrait → transactions candidates datées de `d` à `d + 5 jours`,
   dépôt → de `d − 5` à `d` ; autres conditions inchangées (un seul compte non archivé, non clos, lié au grand livre,
   transaction `pending` de montant opposé, même devise). PayPal (`value_date NULL`) : `d = booking_date`, inchangé.
   Tests : versement sorti le J, arrivée J + 3, ligne UBS à J + 3 → suggérée ; à J + 9 → non suggérée
   (`transferBankAccountId: null`, à choisir) ; ligne UBS à J + 1 (avant l'arrivée prévue) → **non suggérée** (risque dit
   au manuel par 40-6 : le compte se choisit alors à la main, le rattachement ne regarde pas de date).
4. **AC-4 — La ligne UBS se rattache à la même écriture (FR-40-3-3, 34-11b).** Test d'intégration HTTP, deux comptes :
   Stripe CHF (compte `bank` sans IBAN lié à **1101**, frais 6914 ; ligne `payment_provider_accounts`, 38-2) et UBS (lié
   à 1020). Versement −500.00 CHF sorti le J, arrivée J + 2 ; ligne UBS +500.00 du J + 2 (relevé CAMT ou saisie, 34-11b).
   - Voie (b) : `GET /api/v1/reconciliation/proposals?bankAccountId=<Stripe>` → candidat `transferBankAccountId = UBS`,
     lignes datées J + 2 ; `POST /api/v1/reconciliation/accept` → `200 { accepted: [1], failed: [] }`, une écriture
     `D 1020 500.00 / C 1101 500.00` du J + 2, transaction Stripe `reconciled` ; puis `GET …/transactions/<UBS>/entry-
     candidates` liste cette écriture (`matchedElsewhereCount = 1`) et `POST …/link-entry` (`crates/kesh-api/src/lib.rs:931`) la rattache : **une**
     écriture, **deux** transactions rapprochées (Stripe et UBS), 1020 augmenté et 1101 diminué de 500.00 une seule
     fois (aucun doublon d'écriture).
   - Voie (a), ordre inverse : la ligne UBS passée d'abord par écriture manuelle `D 1020 / C 1101` (34-3) ; la proposition
     Stripe porte `linkJournalEntryId` ; l'acceptation rattache sans écriture.
   - Aucune acceptation sans geste : `GET …/proposals` n'écrit rien (compteur d'écritures inchangé), *un appariement
     propose, il ne crée jamais*.
5. **AC-5 — Versements dans d'autres devises (Q-40-3).** Test : Stripe EUR (lié à 1102, compte du plan en EUR) et un
   compte bancaire en EUR (UBS EUR, compte du plan en EUR) → voie (b) `D <UBS EUR> / C 1102` en EUR, cours du mois de
   `value_date` pour la colonne CHF ; compte cible en CHF → `blockedBy` / `failed[]`
   `PROVIDER_TRANSFER_CURRENCY_MISMATCH` (38-6b, inchangé) ; compte cible **prestataire** (solde Wise EUR, ou tout compte
   prestataire en EUR) : la voie (b) crée `D <compte Wise EUR> / C 1102`, puis le dépôt de l'autre prestataire s'y
   rattache par **sa** voie (a) (38-6b, `deposit`) — test avec un `FakeConnector` qui rend un `deposit` +X EUR sur le second
   compte prestataire, daté dans la fenêtre `[d − 5, d]` du dépôt.
6. **AC-6 — Versements de 2024 rattachés en lot (FR-40-3-5, B17, 38-6c1).** Les versements Stripe de 2024 passés par
   l'API (écritures `D 1020 / C 1101` existantes, datées par Beat — date de sortie ou d'arrivée, Q-40-4) : la proposition
   d'un versement téléchargé porte `alreadyBooked` (34-4, `entryCount ≥ 1`) ; **ce n'est pas la voie (a)** de C-38-6b-3 : celle-ci
   exige `matchedElsewhereCount = entryCount = 1` (côté banque déjà rapproché), ce que des écritures de 2024 passées par
   l'API, non rapprochées, ne remplissent pas — la proposition n'y porte donc pas `linkJournalEntryId`. Le rattachement
   des versements 2024 passe par **`GET /api/v1/reconciliation/link-suggestions`** (38-6c1 AC-4 : genre `unique` ou
   `ambiguous`, `version` par élément), puis le lot `POST /api/v1/reconciliation/link-entries` de 38-6c1 (corps
   `{ bankAccountId, items: [{ bankTransactionId, journalEntryId, version }] }`, *Pattern batch* : `{ accepted, failed }`,
   `FailedProposal` par élément, codes de C-38-14) qui rattache plusieurs versements d'un coup. Test : trois versements
   2024 téléchargés, trois écritures existantes dont l'une datée de la sortie et deux de l'arrivée → les suggestions les
   rendent `unique`, le lot les rattache tous, aucune écriture créée ; un lot qui porte pour l'un des trois un
   `journalEntryId` inexistant → ce seul élément en `failed[]` `ENTRY_NOT_LINKABLE`, les deux autres acceptés ;
   écriture déjà rattachée à un autre versement → `ENTRY_ALREADY_LINKED`. **Garde contre le doublon** : accepter en
   `provider` (voie b) un versement 2024 dont `alreadyBooked` est non nul, sans `confirmAlreadyBooked`, →
   `failed[]` `TRANSACTION_ALREADY_BOOKED`, rien d'écrit (38-6a AC-8 (8)).
7. **AC-7 — Versement échoué ou annulé (FR-40-3-4, G11).** Une transaction `payout_failure` / `payout_cancel` classée
   `other` / `unknown_event` avec `related_provider_tx_id` = le versement d'origine (40-2 AC-5) :
   - elle est à la file (`GET /api/v1/reconciliation/review`, 38-6b AC-6 : `reason: "unknown_event"`,
     `reasonSource: "stored"`) et n'a aucun candidat (`GET …/proposals`) ; `POST …/accept` en `provider` →
     `failed[]` `PROVIDER_TX_UNDER_REVIEW` ;
   - **`relatedBankTransactionId`** de l'élément de la file (Écart 2, additif) = l'`id` de la transaction du même compte
     dont `provider_tx_id` = `related_provider_tx_id`, sinon `null` ; lu par la requête de page de la file (une lecture
     par page, non par élément ; test du compteur de 38-6b AC-1 tenu) ; `docs/api-external.md` le dit ;
   - le versement d'origine, s'il est rapproché, **reste** rapproché : défaire est un geste humain (annulation 34-11b,
     `POST …/transactions/{id}/cancel`, inchangée) ; l'échec se traite alors à la main (écriture manuelle 34-3, puis
     « examiné », `…/review-done`) ;
   - Tests : versement rapproché puis échec téléchargé → échec à la file, `relatedBankTransactionId` = versement,
     versement toujours `reconciled` ; échec dont l'origine est hors de Kesh (antérieure à `fetch_from`) →
     `relatedBankTransactionId: null`.
8. **AC-8 — Non-régression (NFR-40-3-2, G18).** Retraits et dépôts PayPal de 38-6b (AC-4 de 38-6b, `value_date NULL`) :
   tests verts **sans retouche de leurs attentes** (date d'écriture = `booking_date`, fenêtre de suggestion inchangée) ;
   acceptations par règle, manuelle, de partage : inchangées (même `payment_date`) ; empreintes `file_hash` PayPal
   inchangées (AC-1).
9. **AC-9 — Ce que la story ne touche pas, documentation.** Aucune migration (C-40-0 ; P1–P8 sans objet ;
   `value_date` existe, `bank_imports.sql:61`) ; aucune route neuve (C-40-6 : `LIB_ROUTES` inchangé) ; aucun code d'erreur
   neuf ; aucune action d'audit neuve ; aucun outil MCP (G17 ; les écritures se lisent par `kesh_ecritures`) ; i18n :
   aucun texte neuf. `docs/api-external.md` : date de l'écriture d'un retrait de prestataire = date de valeur ; fenêtre de
   suggestion ; `relatedBankTransactionId` de la file. Manuel utilisateur et écrans (dates de sortie et d'arrivée à la
   proposition, créance au 31 décembre, ligne UBS avant la date prévue) : **40-6**, qui reçoit ces faits ; aucun contrôle
   visible ici, aucune capture. CHANGELOG : une ligne de l'entrée non datée de l'epic.

## Tâches

- [ ] T0 — Prérequis : 38-3b, 38-6a, 38-6b, 38-6c1, 40-1 et 40-2 fusionnées ; sinon arrêt.
- [ ] T1 — Passage : `value_date` dans la correspondance et dans l'empreinte (si posée) ; tests (AC-1).
- [ ] T2 — Voie (b) datée par `payment_date`, passée au constructeur ; tests de date, d'exercice, de cours (AC-2).
- [ ] T3 — Suggestion à partir de `COALESCE(value_date, booking_date)` ; tests de fenêtre (AC-3).
- [ ] T4 — Tests d'intégration des deux voies et du second côté UBS ; devises ; lot 2024 (AC-4, AC-5, AC-6).
- [ ] T5 — File : `relatedBankTransactionId` additif ; tests (AC-7).
- [ ] T6 — `docs/api-external.md`, CHANGELOG ; non-régression PayPal (AC-8, AC-9).
- [ ] T7 — Gate complet (dépôts touchés : ciblage interdit) sur base remise à zéro, `scripts/gate-ci.sh`, E2E complet au
      dernier commit de code.

## Dev Notes

- **Pourquoi la date d'arrivée** (G10) : dater l'écriture de la sortie gonflerait 1020 au 31 décembre d'un montant que le
  relevé UBS n'a pas (contrôle K3 faux) ; datée de l'arrivée, le montant en route reste dans « Débiteurs Stripe », la
  créance de B13. 40-4 en tire la ligne « versements en route ».
- **`arrival_date` est une prévision** (doc : « *Date that you can expect the payout to arrive* ») : la ligne UBS peut
  arriver un autre jour ; seule la suggestion en dépend (AC-3), jamais le rattachement.
- **Une seule date de paiement** : ne pas écrire un second `COALESCE` dans le constructeur ou la route ; la règle vit dans
  `payment_date` (Rust) et dans une seule expression SQL (AC-3, gardée par un test de concordance ; le commentaire de
  `reconciliation_link.rs:626` n'en est pas le précédent : il interdit la réécriture, la date y étant liée en paramètre).
- **Débit du compte bancaire par Stripe** (solde négatif) : à la file par 40-2 (Q-40-2-1), hors de cette story.
- **Points de contact** : 38-6b C-38-6b-3, AC-1, AC-4, AC-6, AC-9 ; 38-6a C-38-6a-10, AC-8 ; 38-6c1 AC-4, AC-5, AC-11 ;
  34-11b (second côté) ; 40-4 (versements en route : `booked_at < as_of ≤ value_date`) ; 40-6 (écrans, manuel).
- **Splitting** : trois modules (≤ 5).

### References

- `planning-artifacts/epic-40-stripe.md` § G10–G12, C-40-2, C-40-5, 40-3.
- `implementation-artifacts/38-6b-conversions-remboursements-litiges.md` C-38-6b-3, C-38-6b-4, AC-1, AC-4, AC-6 ;
  `38-6a-…` C-38-6a-10 ; `38-6c1-…` ; `40-2-classement-des-types-stripe.md` AC-2, AC-5.
- <https://docs.stripe.com/api/payouts/object> (`arrival_date`, `balance_transaction`, `failure_balance_transaction`).

## Questions ouvertes

- **Q-40-3** (Beat) : vers quels comptes partent les versements EUR et USD (UBS en devise, Wise) ? L'AC-5 couvre les deux.
- **Q-40-4** (Beat) : date des écritures de versement 2024 — sans effet sur le rattachement (AC-6), mais deux conventions
  dans les mêmes livres (2024 à la sortie, 2025 à l'arrivée) seraient à dire au manuel.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — rédaction (Opus). 9 AC ; 5 FR (dont 1 ajoutée), 3 NFR ; aucune migration ; trois écarts à l'epic (date de
  l'écriture déjà dans le socle, `relatedBankTransactionId` additif de la file, `value_date` posée par 40-2).
- 2026-11-15 — validation P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW ; remédiés dans la fiche, aucun inter-fiches.
  MEDIUM 40-3-P1-1 (AC-6) : la proposition ne porte pas `linkJournalEntryId` pour des écritures 2024 non rapprochées
  (C-38-6b-3 a exige `matchedElsewhereCount = entryCount = 1`) → rattachement par `link-suggestions` puis `link-entries`
  (38-6c1 AC-4, AC-5), garde `TRANSACTION_ALREADY_BOOKED` contre le doublon, test du « quatrième versement » réécrit.
  MEDIUM 40-3-P1-2 (AC-3, Dev Notes) : `reconciliation_link.rs:626` cité en précédent d'un `COALESCE` SQL alors qu'il
  l'interdit → première copie assumée, test de concordance avec `payment_date`. LOW : code d'exercice absent figé (AC-2) ;
  formulation « solde de 1101 revenu… » (AC-4) ; T0 sans 38-3b ni 40-1. Lignes lues et justes : `reconciliation.rs:277`,
  `routes/reconciliation.rs:3748, 4198, 5084, 5566`, `bank_imports.sql:61`, `bank_transaction.rs:87`, `bank_imports.rs:152-161`,
  `lib.rs:931`, `api-external.md:704`. Axes exercés : prémisses, contradictions B*/G*/C-40-*/38-*/40-2, sites non
  résolus (migration, sauvegarde, audit, RBAC, i18n, MCP, manuels), comptabilité, sécurité, exigences, tests, taille.
  Non exercés : réponses Stripe réelles (aucune d'échec de versement), exécution de code.
