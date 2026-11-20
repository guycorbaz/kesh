# Story 38-11c : rapprochement PayPal — ventes rattachées par TXN, frais, règles, conversions, retraits, écarts

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/38-paypal`, numéros de ligne relevés après la fusion de `origin/main`
(commit de fusion `43ceb541`). Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11b
(colonnes, comptes PayPal, transactions importées). **Suivie par** : 38-11d (écrans). Reprend, réduits au strict
besoin, l'ancienne 38-4 (critères e-mail exact et sens), 38-6a (ventes, achats, frais), 38-6b (conversions,
retraits, « à examiner ») et 38-6c1 (rattachement en devise).*

## Story

En tant que **Comptable** dont les ventes viennent des boutiques — écritures déjà passées au débit du compte PayPal de
la devise, avec « TXN <id> » ou « transaction <id> » dans le libellé —, je veux que Kesh **rattache** chaque
encaissement PayPal à l'écriture de vente de même identifiant, même montant brut et même devise, n'écrive que les
**frais**, propose les achats par règle, les conversions et les retraits, et me **liste les écarts** (encaissement
sans vente, vente sans encaissement, montant ou devise différents), afin de tenir mes comptes PayPal 2025 sans
saisie, et de trouver les erreurs des boutiques.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11c-1` | Le rattachement à une écriture existante (v017-3a) vaut aussi pour une transaction **en devise** sur un compte bancaire lié à un compte du plan de cette devise, le mouvement se comptant en devise. | AC-1 |
| `FR-38-11c-2` | Critères neufs des règles d'affectation : **type PayPal** (exact) et **e-mail exact** ; filtre de **sens** facultatif sur toute règle. | AC-2, AC-3 |
| `FR-38-11c-3` | Revue d'un compte PayPal sur un exercice : propositions (rattachement de vente, frais, règle, conversion, retrait), écarts (trois genres), « à examiner ». Kesh propose, rien n'est écrit à la lecture. | AC-4 à AC-6 |
| `FR-38-11c-4` | Acceptation en lot des propositions choisies ; un refus par proposition (*Pattern batch*). | AC-7 à AC-9 |
| `FR-38-11c-5` | Conversion : les deux moitiés au **même montant CHF** ; retrait : virement vers le compte choisi. | AC-8 |
| `NFR-38-11c-1` | Aucun code TVA (Q4 de Beat) ; aucune écriture de vente (le produit vient des boutiques). | AC-6, AC-8 |
| `NFR-38-11c-2` | Classement et rapprochement en fonctions **pures**, testées sans base. | AC-4, AC-5 |

## Critères d'acceptation

1. **AC-1 — Rattachement en devise** (`crates/kesh-db/src/repositories/reconciliation_link.rs`).
   - La condition 1 (`transaction_can_be_linked_to_entry`, `:535-537`, « montant non nul **et** CHF ») devient :
     montant non nul **et** `t.currency` = devise du compte du plan **actuel** du compte bancaire de `t` (`CHF` quand
     ce compte n'a pas de devise). Signature : la devise du compte du plan en second argument ; les cinq appelants
     (`:879`, `:905`, `:978`, `:1174` et le test `:1313`) la passent — un compte CHF garde exactement le
     comportement d'aujourd'hui (tests existants verts, test `:1313-1321` gardé et complété : `EUR` sur compte EUR
     vrai, `EUR` sur compte CHF faux, `USD` sur compte EUR faux).
   - Le mouvement de la condition 3 se compte **dans la devise du compte** quand elle n'est pas CHF : une seconde
     constante à côté d'`ENTRY_MOVEMENTS` (`:800-804`), `SUM(CASE WHEN l.debit > 0 THEN l.amount_currency ELSE
     -l.amount_currency END)` sur les lignes du compte (une ligne de réévaluation, `amount_currency = 0`, n'y pèse
     rien) ; `entry_sql` (`:629`) reçoit le genre de mouvement, **un seul texte** pour la liste, le geste et le
     décompte (tests `the_restricted_entry_query_is_the_list_query_plus_the_entry`,
     `the_matched_elsewhere_query_is_the_list_query_plus_the_link` étendus aux deux genres).
   - Les routes `GET …/entry-candidates` (`crates/kesh-api/src/routes/reconciliation.rs:6872`) et `POST …/link-entry`
     (`:6983`) en héritent sans changement de forme ; leurs docs « hors CHF » (`:6859-6860`, `:6603`) suivent (grep du
     symptôme « hors CHF » dans les deux fichiers et `docs/api-external.md`, au Change Log).
   - ⚠️ Le rattachement des **règlements** de la 29-8 (`get_link_candidates` `:6606`, `post_link_transaction`
     `:6697`, `transaction_can_be_linked` du dépôt `:90`) **reste CHF** : les ventes des boutiques sont des
     écritures, non des règlements de facture. `get_proposals` (`:1091`, saut hors CHF `:1189-1196`) et
     `accept_one_rule` (`:4191`, refus hors CHF `:4385-4395`) restent CHF : le rapprochement PayPal a ses propres
     routes (AC-4, AC-7).
2. **AC-2 — Critères neufs** (`crates/kesh-reconciliation/src/rules.rs`, entité `ReconciliationMatchType`,
   `crates/kesh-db/src/entities/reconciliation_rule.rs:32`). Deux variantes, valeurs ENUM posées par la migration de
   38-11b : `ProviderType` (`provider_type`) — égalité, après `trim` et sans casse, avec
   `BankTransaction::provider_type` ; `CounterpartyEmailExact` (`counterparty_email_exact`) — égalité, tout blanc
   retiré et minuscules, avec `BankTransaction::counterparty_email`. Une transaction sans ces champs (CAMT, CSV,
   saisie) ne les satisfait jamais. `rule_matches` (`rules.rs:46-70`) gagne les deux bras ; les huit tests existants
   restent verts sans changement d'assertion. Valeur stockée : la forme normalisée pour l'e-mail (un `@`, un
   caractère au moins de chaque côté, sinon 400 `VALIDATION_ERROR`), telle que saisie pour le type.
3. **AC-3 — Sens.** Colonne `direction` (38-11b) : `credit` → la règle ne vaut que pour un montant > 0, `debit` → < 0,
   `NULL` → les deux ; appliqué dans `rule_matches` **après** le critère, pour tout type (donc aussi au rapprochement
   CHF existant, où toutes les règles d'aujourd'hui ont `NULL` : comportement inchangé). Routes
   `crates/kesh-api/src/routes/reconciliation_rules.rs` (`post_create` `:259`, `patch` `:364`, `list` `:314`,
   `detail` `:342`) : `direction` dans le corps et la réponse (`null` admis ; autre valeur → 400) ; `matchType` admet
   les deux valeurs neuves ; doublon (`uq_reconciliation_rules_match_active`) → 409
   `RECONCILIATION_RULE_DUPLICATE` existant, quel que soit le sens (limite KISS de 38-11b AC-1, testée). Le compte de
   contrepartie d'une règle reste un compte **CHF** (garde de 37-1 inchangée) : un achat PayPal en devise s'écrit
   contre un compte de charge CHF, au cours (AC-8).
4. **AC-4 — Classement** (fonction pure neuve, `crates/kesh-reconciliation/src/paypal.rs`). `classify(tx) ->
   PaypalKind` sur `provider_type`, le signe et le suffixe `:FEE` de `transaction_id` (constante de 38-11b) :
   - `Fee` — suffixe `:FEE` ;
   - `SaleReceipt` — montant > 0 et type ∈ {`Website Payment`, `Subscription Payment`, `Express Checkout Payment`,
     `Mobile Payment`, `Shopping Cart Payment Received`, `Payment Received`} ;
   - `Conversion` — type ∈ {`General Currency Conversion`, `User Initiated Currency Conversion`} ;
   - `Withdrawal` — montant < 0 et type ∈ {`General Withdrawal`, `Withdraw Funds to Bank Account`, `User Initiated
     Withdrawal`} ;
   - `Other` — le reste (achats, remboursements, litiges, retenues isolées…).
   Listes fermées, comparées sans casse, chacune en constante documentée ; un type inconnu tombe en `Other` (jamais
   d'erreur). `txn_ids_in(text) -> Vec<String>` : identifiants qui suivent `TXN` ou `transaction` (sans casse,
   séparés par espace, `:`, `#` ou `-`), `[0-9A-Z]{10,20}` en mot entier, rendus en majuscules, sans doublon.
5. **AC-5 — Rapprochement pur.** `build_review(input) -> Review` (même module), sans I/O, sur : les transactions
   **pending** des comptes bancaires du compte PayPal dont la date tombe dans l'exercice ; les identifiants de toutes
   leurs transactions (pending et rapprochées) ; les **écritures de vente** de l'exercice — écritures non
   contre-passées ayant une ligne au **débit** d'un compte du plan d'un de ces comptes bancaires et un identifiant
   `txn_ids_in(description)` non vide —, chacune avec son mouvement en devise par compte et son état de rattachement
   (une transaction la pointe ou non) ; les règles actives (`first_matching_rule`, `rules.rs:85-95`, comptes actifs) ;
   les réglages (compte de frais par compte bancaire). Sortie, chaque élément avec la transaction, sa date, son
   montant, sa devise, son type et son nom :
   - **Encaissement** (`SaleReceipt`) : les écritures de vente dont un identifiant égale `transaction_id` →
     exactement une, sur le compte du plan **du même compte bancaire**, mouvement en devise = montant, non
     rattachée → proposition `link_sale { bankTransactionId, journalEntryId }` ; aucune → écart
     `receipt_without_sale` ; une, mais sur un autre compte (devise différente) ou de mouvement différent, ou déjà
     rattachée → écart `amount_or_currency_mismatch` (`details` : écriture, montant et devise attendus et trouvés) ;
     plusieurs → « à examiner » `ambiguous_sale` (écritures nommées).
   - **Frais** : compte de frais réglé → proposition `fee { bankTransactionId, feeAccountId }` ; sinon « à examiner »
     `fee_account_missing`.
   - **Conversion** : appariée à l'unique autre transaction `Conversion` pending du **même compte PayPal**, de signe
     opposé, d'une **autre devise**, de même `end_to_end_id` (le `Reference Txn ID`) non vide, ou à défaut de même
     date et de types identiques quand la paire est la seule du jour → proposition `conversion {
     debitTransactionId, creditTransactionId }` (une seule par paire) ; sinon « à examiner » `conversion_unpaired`.
   - **Retrait** : proposition `withdrawal { bankTransactionId, counterpartyAccountId }`, contrepartie préremplie =
     compte du plan du compte bancaire **principal** de la société (s'il en a un ; sinon `null`, à choisir à l'écran,
     et la proposition est incomplète tant qu'il manque).
   - **Autre** : `first_matching_rule` → proposition `rule { bankTransactionId, ruleId, counterpartyAccountId }` ;
     sinon « à examiner » `no_rule`.
   - **Vente sans encaissement** : écriture de vente non rattachée dont **aucun** identifiant n'est celui d'une
     transaction (pending ou rapprochée) des comptes du compte PayPal → écart `sale_without_receipt` (écriture,
     numéro, date, identifiant, montant et devise).
   Ordre de chaque liste : date puis identifiant. Les écritures de vente se lisent par une requête bornée à
   l'exercice et aux comptes du plan du compte PayPal (la recherche de l'identifiant se fait en Rust par
   `txn_ids_in`, jamais par un `LIKE` seul) ; coût mesuré au développement sur 600 transactions et 600 écritures (au
   Change Log).
6. **AC-6 — Route de revue** `GET /api/v1/paypal/review?account=<libellé>&fiscalYearId=<id>` (Comptable et plus,
   clés d'API admises en lecture), lecture seule, une transaction de lecture. Refus : 404 `PAYPAL_ACCOUNT_NOT_FOUND`
   (38-11b), 404 `FISCAL_YEAR_NOT_FOUND`. Réponse `{ account, fiscalYearId, proposals: [...], gaps: [...],
   toReview: [...], counts: {…} }`, champs en camelCase, montants en chaînes décimales (convention de l'API).
   **Aucun code TVA** sur aucune proposition (Q4).
7. **AC-7 — Route d'acceptation** `POST /api/v1/paypal/accept` (Comptable et plus, clés d'API `read-write`
   admises), corps `{ account, proposals: [ {type, …} ] }` (formes de l'AC-5 ; `withdrawal` exige
   `counterpartyAccountId`). *Pattern batch* (`CLAUDE.md`, *Pattern batch — FailedProposal per-proposal*) : 400
   global seulement pour un corps mal formé, vide, de plus de `MAX_IN_CLAUSE_IDS` (`repositories/reconciliation.rs:1121`)
   propositions ou portant deux fois la même transaction ; tout le reste → `failed[]` (`FailedProposal`,
   `routes/reconciliation.rs:576` : identifiant de transaction — la transaction **débitée** pour une conversion —,
   `errorCode` constant, `details`), HTTP 200. Verrous : les comptes bancaires touchés, par identifiant croissant
   (`with_account_lock`), puis une transaction de base avec un point de sauvegarde par proposition (patron de
   `post_accept`, `:1688`). Chaque proposition **rejuge** le classement de l'AC-5 sous verrou (transaction encore
   pending, même genre, même écriture, règle qui correspond toujours) ; sinon `failed[]`
   `PAYPAL_PROPOSAL_STALE`.
8. **AC-8 — Écritures.** Toutes au journal Banque, à la date de la transaction, par `journal_entries::create_in_tx`
   (`repositories/journal_entries.rs:381`, `enforce_postable = false` comme les autres flux de rapprochement, compte
   de contrepartie contrôlé avant, comme `accept_one_rule`) ; libellé : « PayPal <compte> · <type> · TXN <id> »,
   « Frais PayPal <devise> · TXN <id> », tronqué UTF-8 à la borne existante. Ligne du compte PayPal **en devise** si
   son compte du plan en a une : `foreign = { amount = |montant|, rate = cours mensuel de la société pour la devise
   et le mois de la date }` (`exchange_rates::find_company_rate_in_tx`, `repositories/exchange_rates.rs:365`,
   `RateKind::Month`), CHF = `round(amount × rate, 2)` ; cours absent → `failed[]` `RATE_UNAVAILABLE` (code de 37-2,
   `details: { currency, month }`). La contrepartie porte le même CHF (et, si elle est de la même devise — compte de
   frais en EUR —, le même montant en devise et le même cours). Les refus du contrôle central des lignes en devise
   deviennent `failed[]` par `fx_failed_proposal` (`routes/reconciliation.rs:763`).
   - `link_sale` → `reconciliation_link::link_entry_in_tx` (`:1156`), aucune écriture ; ses refus (`LinkRefusal`) →
     `failed[]` sous leurs codes existants.
   - `fee` → débit compte de frais / crédit compte PayPal (frais négatif).
   - `rule` → débit ou crédit de la contrepartie de la règle selon le signe ; `increment_applied_count_in_tx` et la
     trace `reconciliation_rule.applied`, comme `accept_one_rule` (`:4604-4640`).
   - `withdrawal` → débit contrepartie choisie (compte du plan, actif, imputable, CHF) / crédit compte PayPal. Le
     mouvement de la banque, importé par CAMT, se rattachera **à la même écriture** par le rattachement existant :
     c'est « l'autre côté d'un virement » de la 34-11b (condition 4, `reconciliation_link.rs:500-512`) ; un retrait
     depuis un solde en devise vers un compte CHF n'a pas d'autre côté au même montant : la banque se rattache par
     son propre rapprochement (limite au manuel).
   - `conversion` → **deux écritures** dans la même transaction, par le compte de conversion de la société
     (`companies.fx_transit_account_id`, 37-1) : écriture 1, débit conversion / crédit PayPal source (devise source,
     cours mensuel, CHF `X`) ; écriture 2, débit PayPal cible (montant cible, cours `rate_from_chf(montant cible,
     X)`, `crates/kesh-core/src/accounting/fx.rs:266`, provenance `manual`) / crédit conversion `X` — **même CHF des
     deux côtés** (Q3 de Beat ; « une écriture » est impossible : une écriture ne porte qu'une devise étrangère,
     `FX_MULTIPLE_CURRENCIES`). Par `create_conversion_in_tx` (`repositories/journal_entry_conversions.rs:261`, passé
     `pub`, signature inchangée), `complete_difference = false` ; compte de conversion absent → `failed[]`
     `FX_ACCOUNT_INVALID` (code de 37-3b). Un côté CHF : l'écriture CHF de ce côté porte des lignes CHF.
   - Après chaque écriture : la transaction (les deux pour une conversion) passe `reconciled` avec `matched_entry_id`
     par **une** fonction de dépôt neuve `mark_reconciled_in_tx` (contrôle de version, `rows_affected = 1`, sinon
     `RECONCILIATION_ALREADY_RECONCILED`), extraite de la mise à jour d'`accept_one_rule` (`:4573-4600`), qui
     l'appelle désormais (DRY ; tests d'`accept_one_rule` verts).
   - Trace `paypal.proposal_accepted` par proposition acceptée (`details` : genre, transactions, écritures, règle),
     sous l'acteur (utilisateur et clé d'API, `NewAuditLogEntry::for_actor`) ; inscrite aux gardes d'audit
     (`audit_route_registry.rs`, `audit_acteur_garde.rs`).
9. **AC-9 — Réponse et i18n.** `{ accepted: [{ type, bankTransactionIds, journalEntryIds }], failed: [...] }`. Codes
   neufs : `PAYPAL_PROPOSAL_STALE` ; libellés des genres d'écart et d'« à examiner » (servis par l'écran, 38-11d) ;
   messages aux quatre locales ; `crates/kesh-api/src/errors.rs`.
10. **AC-10 — Ce que la story ne fait pas** (phase 2, au fichier d'epic) : aucune écriture de vente, de
    remboursement ni de litige ; aucun rattachement automatique (tout passe par l'acceptation) ; aucun groupe
    d'écritures ; aucun code TVA ; aucune règle à compte en devise ; aucune boutique par préfixe ni code
    d'événement. Un remboursement ou une retenue isolée va « à examiner » : l'utilisateur passe l'écriture à la main
    (écriture en devise, 37-3b) puis la rattache (AC-1).
11. **AC-11 — Non-régression.** Tests existants du rapprochement, des règles et du rattachement verts sans
    changement d'assertion (seuls les littéraux de construction gagnent leurs champs ; nombre au Change Log).

## Tâches

- [ ] T1 — Rattachement en devise : condition 1, mouvement en devise, appelants, docs (AC-1).
- [ ] T2 — Critères neufs et sens : entité, `rule_matches`, dépôt et routes des règles (AC-2, AC-3).
- [ ] T3 — `paypal.rs` : `classify`, `txn_ids_in`, `build_review`, tests purs (AC-4, AC-5).
- [ ] T4 — Lectures de la revue (transactions, écritures de vente, mouvements) et route `GET` (AC-6).
- [ ] T5 — `mark_reconciled_in_tx` extraite ; `create_conversion_in_tx` publique ; route `POST`, écritures, traces
  (AC-7, AC-8).
- [ ] T6 — Codes, i18n (AC-9) ; non-régression (AC-11).

## Stratégie de tests

- **Purs** (`paypal.rs`, `rules.rs`) : chaque genre et chaque type de la liste ; `txn_ids_in` (« TXN 0TEST… »,
  « transaction: 0test… », identifiant dans une phrase, deux identifiants, faux positif « TXN » seul) ;
  `build_review` sur des jeux construits à la main : vente concordante, sans vente, sur l'autre devise, montant
  différent, déjà rattachée, deux ventes, vente sans encaissement, conversion appariée par référence et par date,
  conversion orpheline, frais avec et sans compte, règle par type, par e-mail, par sens, sans règle.
- **HTTP** (`crates/kesh-api/tests/paypal_reconciliation_e2e.rs`) : fixture de 38-11a importée par 38-11b, écritures
  de vente posées par `POST /journal-entries` (comme le script des boutiques, R1), cours mensuels posés ; revue →
  listes attendues ; acceptation du tout → transactions rapprochées, frais écrits au bon CHF, conversion en deux
  écritures de même CHF (compte de conversion soldé), soldes des comptes PayPal en devise = somme des `Net` ;
  rattachement refusé pour une écriture rattachée entre la revue et l'acceptation (`PAYPAL_PROPOSAL_STALE`) ; cours
  absent → `RATE_UNAVAILABLE` dans `failed[]`, le reste accepté ; clé `read` → 403 au `POST`. Rattachement en devise
  par les routes existantes `entry-candidates` et `link-entry` (EUR sur compte EUR ; CHF inchangé).
- Gate **complet** (exception `kesh-db` : le dépôt `reconciliation_link` change).

## Modules touchés (4)

`kesh-reconciliation` (règles, `paypal.rs`), `kesh-db` (`reconciliation_link`, `reconciliation_rules`,
`reconciliation`, `journal_entry_conversions`, entités), `kesh-api` (routes `paypal` neuves, `reconciliation_rules`,
`reconciliation` pour l'extraction, erreurs), `kesh-i18n`.

## Note de version

*Ajouté* — Rapprochement PayPal : chaque encaissement se rattache à l'écriture de vente de la boutique (même TXN,
même brut, même devise) ; seuls les frais s'écrivent ; achats par règle (critères neufs « type PayPal » et « e-mail
exact », sens facultatif), conversions au même CHF des deux côtés, retraits ; liste des écarts et des lignes à
examiner. *Changé* — Une transaction en devise se rattache à une écriture existante sur un compte bancaire en devise.

## Notes de développement

- ⚠️ Le prompt d'origine nommait `get_link_candidates` / `post_link_transaction` : ce sont les routes des
  **règlements de facture** (29-8). Les ventes des boutiques sont des écritures : la fonction juste est le
  rattachement à une écriture existante de la v017-3a (`find_entry_candidates` `:870`, `link_entry_in_tx` `:1156`).
- La conversion PayPal 2024 passait déjà par un compte de transit (9901), même CHF des deux côtés : c'est la forme de
  l'AC-8.
- Le cours : 2024 employait un cours moyen du mois ; Kesh prend le cours mensuel retenu par la société (table
  modifiable, D9 de l'Epic 37).

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
