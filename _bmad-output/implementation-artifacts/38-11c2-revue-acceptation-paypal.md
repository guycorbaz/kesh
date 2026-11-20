# Story 38-11c2 : revue et acceptation PayPal — moteur commun aux prestataires, ventes rattachées, frais, règles, conversions, retraits, écarts

Status: drafted

*Fiche rédigée le 2026-11-20 sous la clé 38-11c (branche `epic/38-paypal`, numéros de ligne relevés après la fusion
de `origin/main` `43ceb541`, revérifiés sur `origin/main` `fd04099f`), **découpée à la validation P3** (décision de
l'orchestrateur, proposée à la P1) : les AC-1 à AC-3 sont partis en **38-11c1** ; cette fiche garde les **AC-4 à
AC-11, sans changement de numéro** — une mention « 38-11c AC-4 » à « AC-11 » dans une autre fiche (38-11d, 38-12a,
Epics 39 et 40) la désigne. Epic 38, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 38-11b
(comptes PayPal, transactions importées, `with_account_locks`), **38-11c1** (rattachement en devise, autre côté à deux
branches, filtres de règle). **Suivie par** : 38-11d (écrans) ; **réutilisée** par 39-11c (Wise) et 40-7c (Stripe),
gestionnaires minces sur le moteur de cette story. Reprend, réduits au strict besoin, l'ancienne 38-6a (ventes, achats,
frais), 38-6b (conversions, retraits, « à examiner »).*

## Story

En tant que **Comptable** dont les ventes viennent des boutiques — écritures déjà passées au débit du compte PayPal de
la devise, avec « TXN <id> » ou « transaction <id> » dans le libellé —, je veux que Kesh me **propose** de rattacher
chaque encaissement PayPal à l'écriture de vente de même identifiant, même montant brut et même devise, d'écrire les
**frais**, les achats par règle, les conversions et les retraits, que j'accepte en lot, et me **liste les écarts**
(encaissement sans vente, vente sans encaissement, montant ou devise différents), afin de tenir mes comptes PayPal 2025
sans saisie, et de trouver les erreurs des boutiques.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-11c2-1` | Revue d'un compte PayPal sur un exercice : propositions (rattachement à une écriture, frais, règle, conversion, retrait), écarts (trois genres), « à examiner ». Kesh propose, rien n'est écrit à la lecture ; **tout rattachement est une proposition acceptée** par l'utilisateur. | AC-4 à AC-6 |
| `FR-38-11c2-2` | Acceptation en lot des propositions choisies ; un refus par proposition (*Pattern batch*). | AC-7 à AC-9 |
| `FR-38-11c2-3` | Conversion, règle commune aux prestataires : un côté en CHF → **une** écriture (le CHF réel) ; deux devises étrangères → **deux** écritures au **même CHF** par le compte de conversion. Retrait : virement vers le compte choisi, depuis un solde CHF. | AC-8 |
| `NFR-38-11c2-1` | Aucun code TVA (Q4 de Beat) ; aucune écriture de vente (le produit vient des boutiques). | AC-6, AC-8 |
| `NFR-38-11c2-2` | Classement, décisions de la revue et plan de conversion en fonctions **pures**, testées sans base. | AC-4, AC-5, AC-8 |
| `NFR-38-11c2-3` | **Un seul moteur** de revue et d'acceptation, **paramétré par le prestataire** et par la **clé de rattachement** : Wise (39-11c) et Stripe (40-7c) y branchent des gestionnaires minces, sans seconde logique de lot, de verrou, de rejugement ni d'écriture. Aucun nom de prestataire dans le code commun ; codes d'erreur `PROVIDER_*`. | AC-5, AC-7, AC-8 |

## Critères d'acceptation

*(AC-1 à AC-3 : 38-11c1.)*

4. **AC-4 — Classement PayPal** (fonction pure neuve, `crates/kesh-reconciliation/src/paypal.rs`). `classify(tx) ->
   ProviderKind` sur `provider_type`, le signe et le suffixe `:FEE` de `transaction_id` (constante de 38-11b) :
   - `Fee` — suffixe `:FEE`, **quel que soit le signe** (frais positifs d'un remboursement compris) ;
   - `SaleReceipt` — montant > 0 et type ∈ {`Website Payment`, `Subscription Payment`, `Express Checkout Payment`,
     `Mobile Payment`, `Shopping Cart Payment Received`, `Payment Received`} ;
   - `Conversion` — type ∈ {`General Currency Conversion`, `User Initiated Currency Conversion`} ;
   - `Withdrawal` — montant < 0 et type ∈ {`General Withdrawal`, `Withdraw Funds to Bank Account`, `User Initiated
     Withdrawal`} ;
   - `Other` — le reste (achats, remboursements, litiges, retenues isolées…).
   Listes fermées, comparées sans casse, chacune en constante documentée ; un type inconnu tombe en `Other` (jamais
   d'erreur). `txn_ids_in(text) -> Vec<String>` : identifiants qui suivent `TXN` ou `transaction` (sans casse,
   séparés par espace, `:`, `#` ou `-`), `[0-9A-Z]{10,20}` en mot entier, rendus en majuscules, sans doublon.
   `ProviderKind` est **commun** (module de l'AC-5) ; un prestataire peut ajouter ses genres propres en amont de la
   revue commune (Wise : versement Stripe, 39-11c AC-2 ; Stripe : versement, 40-7c AC-4).
5. **AC-5 — Revue pure, commune** (`crates/kesh-reconciliation/src/provider_review.rs`, neuf, **sans nom de
   prestataire** ; `paypal.rs` l'appelle). `build_review(input, spec) -> Review`, sans I/O, où `spec` est le
   **paramétrage du prestataire** : `classify` (AC-4), la **clé de rattachement** de la transaction (`transaction_id`
   pour PayPal et Wise — l'identifiant Wise y est rangé par 39-11b —, `reference` pour Stripe, 40-7b AC-5) et la
   **fonction d'extraction des identifiants** d'un libellé (`txn_ids_in` pour PayPal, `stripe_ids_in` pour Stripe,
   40-7c AC-2). Entrée : les transactions **pending** des comptes bancaires du compte prestataire (`provider`,
   `provider_account`) dont la date tombe dans l'exercice ; les clés de toutes les transactions (pending et
   rapprochées) des comptes bancaires **liés aux mêmes comptes du plan** (ceux du compte prestataire et tout autre
   compte bancaire qui partagerait l'un de ces comptes du plan — sinon une vente encaissée par l'autre compte
   paraîtrait sans encaissement) ; les **écritures de vente** de l'exercice — écritures non contre-passées ayant une
   ligne au **débit** d'un de ces comptes du plan et un identifiant extrait non vide —, chacune avec son mouvement en
   devise par compte et son état de rattachement (une transaction la pointe ou non) ; les règles actives
   (`first_matching_rule`, `rules.rs:85-95`, comptes actifs) ; les réglages (compte de frais par compte bancaire,
   devise de chaque compte du plan, compte du plan du compte bancaire principal de la société). Sortie, chaque élément
   avec la transaction, sa date, son montant, sa devise, son type et son nom :
   - **Encaissement** (`SaleReceipt`) : les écritures de vente dont un identifiant égale la clé → exactement une, sur
     le compte du plan **du même compte bancaire**, mouvement en devise = montant, non rattachée → proposition
     **`link_entry { bankTransactionId, journalEntryId }`** — la **proposition de rattachement générique**, celle de
     39-11c AC-4 (autre côté déjà passé) et de 40-7c AC-2 (charge → vente) ; aucune → écart `receipt_without_sale` ;
     une, mais sur un autre compte (devise différente) ou de mouvement différent, ou déjà rattachée → écart
     `amount_or_currency_mismatch` (`details` : écriture, montant et devise attendus et trouvés) ; plusieurs → « à
     examiner » `ambiguous_sale` (écritures nommées).
   - **Frais** : compte de frais réglé → proposition `fee { bankTransactionId, feeAccountId }` ; sinon « à examiner »
     `fee_account_missing`.
   - **Conversion** (appariement propre à PayPal, dans `paypal.rs`) : appariée à l'unique autre transaction
     `Conversion` pending du **même compte PayPal**, de signe opposé, d'une **autre devise**, de même `end_to_end_id`
     (le `Reference Txn ID`) non vide → proposition `conversion { debitTransactionId, creditTransactionId }` (une seule
     par paire) ; sinon « à examiner » `conversion_unpaired` (pas d'appariement par la date : les paires de 2024
     portent toutes leur référence).
   - **Retrait** (propre à PayPal) : contrepartie préremplie = compte du plan du compte bancaire **principal** de la
     société (s'il en a un ; sinon `null`, à choisir à l'écran, la proposition étant incomplète tant qu'il manque) →
     proposition `withdrawal { bankTransactionId, counterpartyAccountId }` **si le compte PayPal est en CHF** ; retrait
     d'un solde en devise → « à examiner » `withdrawal_foreign_currency` (le CHF reçu par la banque diffère du cours de
     Kesh : une écriture au cours mensuel ferait compter la banque deux fois quand son relevé CAMT se rapproche à son
     tour).
   - **Autre** : `first_matching_rule` → proposition `rule { bankTransactionId, ruleId, counterpartyAccountId }` ;
     sinon « à examiner » `no_rule`.
   - **Vente sans encaissement** : écriture de vente non rattachée dont **aucun** identifiant n'est la clé d'une
     transaction (pending ou rapprochée) de ces comptes bancaires → écart `sale_without_receipt` (écriture, numéro,
     date, identifiant, montant et devise).
   Ordre de chaque liste : date puis identifiant. Les écritures de vente se lisent par **une** requête bornée à
   l'exercice et à ces comptes du plan (la recherche de l'identifiant se fait en Rust par la fonction d'extraction,
   jamais par un `LIKE`) ; coût mesuré au développement sur 600 transactions et 600 écritures (au Change Log). Limite
   écrite (manuel, 38-11d) : une vente et son encaissement de part et d'autre du 1er janvier (dates de l'export, sans
   fuseau) paraissent en écarts.
6. **AC-6 — Route de revue** `GET /api/v1/paypal/review?account=<libellé>&fiscalYearId=<id>` (Comptable et plus,
   clés d'API admises en lecture), lecture seule, une transaction de lecture ; gestionnaire `paypal::review` (module
   `crates/kesh-api/src/routes/paypal.rs`), mince : lectures communes (module de l'AC-7), `build_review` avec le
   paramétrage PayPal. Refus : 404 `PROVIDER_ACCOUNT_NOT_FOUND` (38-11b), 404 `FISCAL_YEAR_NOT_FOUND`. Réponse
   `{ account, fiscalYearId, proposals: [...], gaps: [...], toReview: [...], counts: {…} }`, champs en camelCase,
   montants en chaînes décimales (convention de l'API) — la forme que reprennent `GET /api/v1/wise/review` et
   `GET /api/v1/stripe/review`. **Aucun code TVA** sur aucune proposition (Q4).
7. **AC-7 — Moteur d'acceptation commun et route PayPal** `POST /api/v1/paypal/accept` (Comptable et plus, clés
   d'API `read-write` admises), corps `{ account, proposals: [ {type, …} ] }` (formes de l'AC-5 ; `withdrawal` exige
   `counterpartyAccountId`). Le moteur vit dans un module **commun** `crates/kesh-api/src/routes/provider_accept.rs`
   (aucun nom de prestataire, aucune route propre), appelé par le gestionnaire mince **`paypal::accept_proposals`**
   — ⛔ **jamais nommé `post_accept`** : l'exemption `RETRY_WITH_AUTORISE = ["post_accept"]`
   (`crates/kesh-api/tests/audit_route_registry.rs:1303`) désigne une fonction par son **seul nom** (angle mort (d),
   `:376-378`), si bien qu'une `fn post_accept` de `routes/paypal.rs` pourrait appeler `retry_with` sans que le volet
   (c bis) (`no_route_calls_retry_with_except_post_accept`, `:1707`) la refuse. Inscrite à `LIB_ROUTES` (`:440`)
   sous `("post", "paypal::accept_proposals", Traced, Rejouee)` ; Wise et Stripe y ajoutent `wise::accept_proposals`
   et `stripe::accept_proposals` (39-11c AC-1, 40-7c AC-6). Le gestionnaire appelle l'enveloppe
   **`retry_app_on_deadlock`** (`crates/kesh-api/src/retry.rs:40`), opération `paypal::accept`, et **non**
   `retry_with`, que `post_accept` garde seul.
   - ⚠️ **Piège du patron par proposition** : `post_accept` rejoue aussi le `ROLLBACK TO SAVEPOINT` qui échoue (1305)
     quand un 1213 levé *dans* une proposition a été absorbé en `FailedProposal` alors qu'InnoDB a annulé toute la
     transaction (`accept_once`, `routes/reconciliation.rs:1949`, `TransactionAborted`) ; `retry_app_on_deadlock` ne
     rejoue que le 1213 (`is_app_deadlock`). Le moteur teste donc **toute** erreur de base d'une proposition, après
     déballage (erreurs de l'écriture, `ConversionError`, `DbError` de `link_entry_in_tx`), par
     `kesh_db::retry::is_deadlock_error` **avant** tout `ROLLBACK TO SAVEPOINT`, et la remonte aussitôt en
     `AppError::Database` : l'enveloppe rejoue toute l'opération, verrous repris ; **jamais** un interblocage dans
     `failed[]`. Un échec de `ROLLBACK TO SAVEPOINT` pour une autre cause remonte en erreur (500, non rejouée). Test
     sur le modèle de `rejeu_interblocage_e2e.rs` : une proposition qui interbloque → rejeu, aucune ligne dans
     `failed[]`.
   - **Verrous** : les comptes bancaires à verrouiller se lisent **avant** les verrous, sur le pool, depuis les
     transactions des propositions (ceux d'une autre société ou inconnus → `failed[]` `NOT_FOUND`), plus les comptes
     **supplémentaires** que le prestataire déclare (Stripe : le compte de versement, 40-7c AC-4 d) ; puis une
     transaction de base, `with_account_locks` (38-11b) **sur cette transaction** (patron d'`accept_once`, qui prend
     `with_account_lock` sur la sienne), identifiants croissants, et un point de sauvegarde par proposition (patron de
     `post_accept`, `:1688`). Le moteur écrit des transactions de **plusieurs** comptes bancaires (conversion) : il tient
     le verrou nommé de **chacun** ; la doc de `link_entry_in_tx` (« l'absence de cycle », qui dit que les gestes qui
     créent une écriture puis écrivent une transaction « n'écrivent aucune transaction d'un autre compte ») est
     complétée en ce sens.
   - **Rejugement** : chaque proposition rejuge son classement sous verrou (transaction encore pending, même genre,
     même écriture, même paire, règle qui correspond toujours, paramètres du prestataire inchangés) ; sinon `failed[]`
     **`PROVIDER_PROPOSAL_STALE`** (code générique, Q-39-10 de l'Epic 39 ; servi tel quel à Wise et Stripe). Un
     prestataire fournit le rejugement de ses propositions propres (Stripe : `payout`).
   - *Pattern batch* (`CLAUDE.md`, *Pattern batch — FailedProposal per-proposal*) : 400 global seulement pour un corps
     mal formé, vide, de plus de `MAX_IN_CLAUSE_IDS` (`repositories/reconciliation.rs:1121`) propositions ou portant
     deux fois la même transaction (**les deux transactions d'une conversion comptent**, `debitTransactionId` et
     `creditTransactionId` : une transaction ne figure qu'une fois dans tout le corps) ; tout le reste → `failed[]`
     (`FailedProposal`, `routes/reconciliation.rs:576` : identifiant de transaction — la transaction **débitée** pour
     une conversion —, `errorCode` constant, `details`), HTTP 200 ; les erreurs de base d'une proposition qui ne sont
     pas un interblocage (refus d'écriture : exercice clos, période verrouillée, `OptimisticLockConflict`,
     `BankAccountClosed` de `link_entry_in_tx`) → `failed[]` sous leur code, comme `accept_one_rule`.
   - Compte de `the_registry_partition_is_what_the_story_declares` (`:1082`) et phrase du manuel d'administration
     (`admin-manual.tex`, `le_manuel_dit_le_compte_du_registre`, `:1194`, `.tex` seul) relevés de un, comme en 38-11b
     AC-4, **recomptés** depuis la source.
8. **AC-8 — Écritures, communes.** Toutes au journal Banque, à la date de la transaction, par **une** fonction
   d'écriture du moteur (`write_provider_entry`, `provider_accept.rs`) sur `journal_entries::create_in_tx`
   (`repositories/journal_entries.rs:381`, `enforce_postable = false` comme les autres flux de rapprochement) :
   **deux lignes**, compte du plan du compte prestataire et contrepartie, sens selon le **signe** de la transaction,
   contrôle de la contrepartie **paramétré par le genre** et fait avant (frais : actif, imputable, de type charge,
   38-11b AC-2 ; règle : celui d'`accept_one_rule` ; retrait : actif, imputable, de type actif, CHF ; versement Stripe :
   40-7c AC-4 d) ; libellé fourni par le prestataire, tronqué UTF-8 à la borne existante — PayPal : « PayPal
   <compte> · <type> · TXN <id> », « Frais PayPal <devise> · TXN <id> ». Ligne du compte prestataire **en devise** si
   son compte du plan en a une : `foreign = { amount = |montant| }` au **cours mensuel retenu** par la société pour la
   devise et le mois de la date (le contrôle central des lignes en devise le lit, `journal_entry_fx.rs:174`,
   `find_company_rate_in_tx`, `repositories/exchange_rates.rs:365`, `RateKind::Month`), CHF = `round(amount × rate,
   2)` ; cours absent → `failed[]` `RATE_UNAVAILABLE` (code de 37-2, `details: { currency, month }`). La contrepartie
   porte le même CHF (et, si elle est de la même devise — compte de frais en EUR, compte de versement de 40-7c —, le
   même montant en devise et le même cours). Les refus du contrôle central des lignes en devise deviennent `failed[]`
   par `fx_failed_proposal` (`routes/reconciliation.rs:763`).
   - `link_entry` → `reconciliation_link::link_entry_in_tx` (`:1156`, devise du compte du plan passée, 38-11c1 AC-1),
     aucune écriture ; ses refus (`LinkRefusal`) → `failed[]` sous leurs codes existants.
   - `fee` → frais négatif : débit compte de frais / crédit compte prestataire ; frais positif (remboursement) :
     l'inverse.
   - `rule` → débit ou crédit de la contrepartie de la règle selon le signe ; `increment_applied_count_in_tx` et la
     trace `reconciliation_rule.applied`, comme `accept_one_rule` (`:4597-4645`).
   - `withdrawal` (compte PayPal CHF seulement, AC-5) → débit contrepartie choisie / crédit compte PayPal. Le mouvement
     de la banque, importé par CAMT, se rattache **à la même écriture** par le rattachement existant : c'est l'autre
     côté d'un virement (34-11b ; première branche du prédicat, 38-11c1 AC-1b).
   - `conversion` → **règle commune aux prestataires** (décision de l'orchestrateur du 2026-11-20), en code
     **partagé** : plan pur `plan_conversion` dans `crates/kesh-reconciliation/src/conversion.rs` (entrée : côté source
     et côté cible, chacun compte du plan, devise et montant ; cours mensuel du côté source ; sortie : une ou deux
     écritures à passer, ou un refus), écriture par une fonction de dépôt neuve
     `journal_entry_conversions::create_provider_conversion_in_tx` (`repositories/journal_entry_conversions.rs`) :
     - **un côté en CHF** → **une** écriture : ligne CHF au montant réel du côté CHF, ligne en devise au montant de
       l'autre côté, cours `rate_from_chf(montant en devise, CHF)` (`crates/kesh-core/src/accounting/fx.rs:266`),
       provenance `manual` — le CHF réel fait foi, aucun cours de Kesh ; écriture équilibrée sans différence de
       change ; les **deux** transactions la pointent. Annuler un côté ne retire que son lien (autre côté de seconde
       branche, 38-11c1 AC-1b) ; annuler le second contre-passe l'écriture ;
     - **deux devises étrangères** → **deux** écritures dans la même transaction, par le compte de conversion de la
       société (`companies.fx_transit_account_id`, `entities/company.rs:205`, 37-1) : écriture 1, débit conversion /
       crédit compte source (montant source au cours mensuel retenu, CHF `X`) ; écriture 2, débit compte cible
       (montant cible, cours `rate_from_chf(montant cible, X)`, provenance `manual`) / crédit conversion `X` — **même
       CHF des deux côtés** (Q3 de Beat ; une écriture ne porte qu'une devise étrangère, `FX_MULTIPLE_CURRENCIES`),
       par `create_conversion_in_tx` (`:261`, privée : appelée par la fonction neuve du même module, signature
       inchangée), `complete_difference = false` ; chaque transaction pointe l'écriture de **son** compte (débitée →
       écriture 1, créditée → écriture 2) ; compte de conversion absent → `failed[]` `FX_ACCOUNT_INVALID` (code de
       37-3b) ; cours source absent → `RATE_UNAVAILABLE`. Une telle conversion se défait en annulant **chacune** de ses
       deux transactions ; entre les deux, le compte de conversion n'est pas soldé (limite écrite au manuel, 38-11d).
     - Dans les deux formes, `rate_from_chf` en erreur → `failed[]` `FX_RATE_INVALID` (code existant, `errors.rs:591`).
     Tests purs du plan (CHF→EUR, EUR→CHF, EUR→USD, montant cible nul refusé) et HTTP des deux formes.
     `plan_conversion` et `create_provider_conversion_in_tx` ne nomment aucun prestataire : 39-11c AC-3 les appelle
     tels quels.
   - Après chaque écriture : la transaction (les deux pour une conversion) passe `reconciled` avec `matched_entry_id`
     par **une** fonction de dépôt neuve `mark_reconciled_in_tx` (contrôle de version, `rows_affected = 1`, sinon
     `RECONCILIATION_ALREADY_RECONCILED`), extraite de la mise à jour d'`accept_one_rule` (`:4569-4595`), qui
     l'appelle désormais (DRY ; tests d'`accept_one_rule` verts). ⚠️ **Inventaire des écrivains d'un compte bancaire**
     (`crates/kesh-api/tests/bank_account_closed_inventory.rs`, clé `(fichier, fonction)`, diff dans les deux sens) :
     l'`UPDATE bank_transactions` quitte `accept_one_rule` — son entrée du `REGISTRE` (`Invariant("I1", …)`) rougit
     « sans site » si l'`UPDATE` ne s'y trouve plus — et naît dans `mark_reconciled_in_tx` (`kesh-db/src/repositories/
     reconciliation.rs`), à classer avec sa raison (`Invariant`, transaction `pending` sous le verrou du compte,
     pré-vol des routes qui l'appellent) ; les fonctions d'écriture neuves (si elles portent un littéral d'écriture
     sur l'une des sept tables) s'y classent aussi. Tâche T5 et gate complet l'incluent.
   - Trace **`provider.proposal_accepted`** par proposition acceptée (`details` : prestataire, genre, transactions,
     écritures, règle), sous l'acteur (utilisateur et clé d'API, `NewAuditLogEntry::for_actor`) ; contrôlée par les
     gardes d'audit (`audit_route_registry.rs`, `audit_acteur_garde.rs`). Wise et Stripe l'émettent telle quelle.
   - Annulation : une transaction rapprochée par l'acceptation se dé-rapproche par le geste existant
     (`reconciliation_cancel`, genre `Entry` : contre-passation de son écriture, ou retrait du seul lien quand
     l'écriture a un autre côté, 38-11c1 AC-1b).
9. **AC-9 — Réponse et i18n.** `{ accepted: [{ type, bankTransactionIds, journalEntryIds }], failed: [...] }`. Code
   neuf : `PROVIDER_PROPOSAL_STALE` (`FX_RATE_INVALID`, `RATE_UNAVAILABLE`, `FX_ACCOUNT_INVALID` existent) ;
   libellés des genres d'écart et d'« à examiner » (`withdrawal_foreign_currency` compris, servis par l'écran,
   38-11d) ; messages aux quatre locales ; `crates/kesh-api/src/errors.rs`.
10. **AC-10 — Ce que la story ne fait pas** (phase 2, au fichier d'epic) : aucune écriture de vente, de
    remboursement ni de litige ; aucun rattachement automatique (tout passe par l'acceptation) ; aucun groupe
    d'écritures ; aucun code TVA ; aucune règle à compte en devise ; aucune boutique par préfixe ni code
    d'événement ; aucun retrait depuis un solde en devise. Une facture fournisseur ne se paie pas depuis un compte
    PayPal (limite de la phase 1, 38-11b AC-3 : les achats passent par les règles ; remède : compte interne ou
    écriture à la main, puis rattachement, 38-11c1 AC-1). Un remboursement ou une retenue isolée va « à examiner » :
    l'utilisateur passe l'écriture à la main (écriture en devise, 37-3b) puis la rattache (38-11c1 AC-1).
11. **AC-11 — Non-régression.** Tests existants du rapprochement, des règles et du rattachement verts sans
    changement d'assertion (seuls les littéraux de construction gagnent leurs champs ; nombre au Change Log).

## Tâches

- [ ] T3 — `provider_review.rs` (types communs, `build_review` paramétré) ; `paypal.rs` : `classify`, `txn_ids_in`,
  paramétrage, appariement des conversions, retrait ; `conversion.rs` : `plan_conversion` ; tests purs (AC-4, AC-5,
  AC-8).
- [ ] T4 — Lectures communes de la revue (transactions, clés, écritures de vente, mouvements) et route `GET` (AC-6).
- [ ] T5 — `mark_reconciled_in_tx` extraite ; `create_provider_conversion_in_tx` ; moteur `provider_accept.rs`
  (verrous, points de sauvegarde, interblocage remonté, rejugement, `write_provider_entry`) ; route `POST`
  `paypal::accept_proposals` (rejeu, registre, phrase du manuel d'administration), traces (AC-7, AC-8).
- [ ] T6 — Codes, i18n (AC-9) ; non-régression (AC-11).

*(T1 et T2 de l'ancienne 38-11c : 38-11c1.)*

## Stratégie de tests

- **Purs** (`provider_review.rs`, `paypal.rs`, `conversion.rs`) : chaque genre et chaque type de la liste ;
  `txn_ids_in` (« TXN 0TEST… », « transaction: 0test… », identifiant dans une phrase, deux identifiants, faux positif
  « TXN » seul) ; `build_review` sur des jeux construits à la main : vente concordante, sans vente, sur l'autre devise,
  montant différent, déjà rattachée, deux ventes, vente sans encaissement, vente encaissée par un autre compte bancaire
  du même compte du plan, clé `reference` au lieu de `transaction_id` (paramétrage), conversion appariée par
  référence, conversion orpheline, frais négatifs et positifs, avec et sans compte, retrait CHF et en devise, règle
  filtrée par type, par e-mail, par sens, règle non filtrée, sans règle ; plan de conversion (une et deux écritures).
- **HTTP** (`crates/kesh-api/tests/paypal_reconciliation_e2e.rs`) : fixture de 38-11a importée par 38-11b, écritures
  de vente posées par `POST /journal-entries` (comme le script des boutiques, R1), cours mensuels posés ; revue →
  listes attendues, rien d'écrit ; acceptation du tout → transactions rapprochées, frais écrits au bon CHF, conversion
  EUR→USD en deux écritures de même CHF (compte de conversion soldé), conversion CHF→EUR en une écriture au CHF réel
  pointée par les deux transactions, puis annulation d'un côté (seul son lien retiré) ; soldes des comptes PayPal en
  devise = somme des `Net` ; rattachement refusé pour une écriture rattachée entre la revue et l'acceptation
  (`PROVIDER_PROPOSAL_STALE`) ; proposition acceptée deux fois → périmée, rien d'écrit ; cours absent →
  `RATE_UNAVAILABLE` dans `failed[]`, le reste accepté ; interblocage injecté → rejeu, aucune ligne dans `failed[]` ;
  clé `read` → 403 au `POST` ; second passage de la revue sans les propositions acceptées.
- Gate **complet** (exception `kesh-db` : dépôts `reconciliation`, `journal_entry_conversions`).

## Modules touchés (4)

`kesh-reconciliation` (`provider_review.rs`, `paypal.rs`, `conversion.rs`), `kesh-db` (`reconciliation` pour
`mark_reconciled_in_tx`, `journal_entry_conversions`, lectures de la revue), `kesh-api` (routes `paypal` et
`provider_accept` neuves, `reconciliation` pour l'extraction, erreurs), `kesh-i18n`. Plus une phrase du manuel
d'administration (`.tex`).

## Note de version

*Ajouté* — Rapprochement PayPal : chaque encaissement se rattache, à votre acceptation, à l'écriture de vente de la
boutique (même TXN, même brut, même devise) ; les frais, les achats par règle, les conversions (une écriture quand un
côté est en CHF, deux au même CHF entre deux devises étrangères) et les retraits d'un solde CHF s'écrivent sur
proposition ; liste des écarts et des lignes à examiner.

## Notes de développement

- La conversion PayPal 2024 passait déjà par un compte de transit (9901), même CHF des deux côtés : c'est la forme à
  deux devises étrangères de l'AC-8 ; un côté CHF donne désormais une seule écriture au CHF réel.
- Le cours : 2024 employait un cours moyen du mois ; Kesh prend le cours mensuel retenu par la société (table
  modifiable, D9 de l'Epic 37).
- **Ce que servent les Epics 39 et 40** (contrats relus à la validation P3 dans `kesh-wt39` `db236659` et `kesh-wt40`
  `bd641ea9`) : moteur commun (`provider_accept.rs`), revue commune (`provider_review.rs`, `build_review` paramétré
  par clé et extraction), proposition générique `link_entry`, écriture commune `write_provider_entry` (frais, règle,
  versement de même devise), `plan_conversion` / `create_provider_conversion_in_tx`, `mark_reconciled_in_tx`,
  `PROVIDER_PROPOSAL_STALE`, trace `provider.proposal_accepted`, gestionnaires `<prestataire>::accept_proposals`.
  ⚠️ 40-7c AC-2 et AC-6 nomment la proposition de rattachement `link_sale` : le nom servi est **`link_entry`** (celui
  de 39-11c AC-4), écart signalé à l'Epic 40 au rapport de la P3.

## Change Log

- 2026-11-20 — fiche rédigée sous la clé 38-11c (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 3 HIGH, 6 MEDIUM, 4 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-38-kiss/validation/38-11c-rapprochement-paypal-P1.md`). Principaux : critères
  neufs de l'`ENUM` remplacés par des filtres (décision de l'orchestrateur) ; la lecture par page
  `count_already_booked_for_page` comptait les transactions en devise contre des mouvements CHF ; origine de la devise
  du compte du plan (pré-vol) ; retrait d'un solde en devise qui comptait la banque deux fois → « à examiner » ;
  règle commune des conversions (une écriture si un côté est CHF, deux sinon) en code partagé ; enveloppe de rejeu,
  registre des routes et phrase du manuel d'administration ; frais positifs ; ventes encaissées par un compte bancaire
  partageant le compte du plan ; appariement des conversions par la date retiré ; `api-external.md` laissé à 38-11d ;
  lignes `:4604-4640` et `:4573-4600` corrigées (`:4597-4645`, `:4569-4595`).
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 1 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-38-kiss/validation/38-11c-rapprochement-paypal-P2.md`). Principaux : la fiche
  disait la route d'acceptation rejouée « comme `post_accept` » alors que `post_accept` est la seule à garder `retry_with`
  (le test refuse tout autre appel) et que le patron par proposition absorbe un 1213 en `failed[]` puis échoue au
  `ROLLBACK TO SAVEPOINT` : enveloppe `retry_app_on_deadlock`, interblocage jamais rangé en `failed[]` ; l'extraction de
  `mark_reconciled_in_tx` déplace un écrivain de l'inventaire `bank_account_closed_inventory` ; les deux transactions
  d'une conversion comptent dans le doublon du corps ; `link_preflight` partagé avec la 29-8 (quatre appelants).
- 2026-11-20 — validation P3 (Claude Opus 5.5, contexte frais ; rapport `kesh-gate-logs/epic-38-kiss/validation/38-11-P3.md`) :
  **découpée** (AC-1 à AC-3 → 38-11c1, `git mv` vers cette clé ; numérotation gardée) ; moteur de revue et
  d'acceptation **commun, paramétré par le prestataire et la clé de rattachement** (`provider_review.rs`,
  `provider_accept.rs`), proposition générique `link_entry` (remplace `link_sale`), écriture commune
  `write_provider_entry`, comptes supplémentaires à verrouiller, codes `PROVIDER_*` (`PROVIDER_PROPOSAL_STALE`,
  `PROVIDER_ACCOUNT_NOT_FOUND`), trace `provider.proposal_accepted` ; gestionnaire `paypal::accept_proposals`, jamais
  `post_accept` ; interblocage testé **avant** le `ROLLBACK TO SAVEPOINT` (l'enveloppe ne rejoue pas un 1305) ;
  conversion à une écriture pointée par les deux transactions, annulation par l'autre côté de seconde branche
  (38-11c1 AC-1b) ; doc « absence de cycle » de `link_entry_in_tx` complétée (plusieurs comptes écrits sous leurs
  verrous).
