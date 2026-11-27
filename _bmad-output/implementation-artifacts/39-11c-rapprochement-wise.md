# Story 39-11c : rapprochement des lignes Wise — frais, conversions, autre côté déjà passé (virements, versements Stripe), règles, écarts

Status: done

*Fiche rédigée le 2026-11-20, branche `epic/39-wise`, numéros de ligne relevés au code de `origin/main` (`fd04099f`),
revérifiés à la validation P1. Epic 39, **phase 1 KISS** (G5 à G10, § *Ce que la phase 1 attend de l'Epic 38*).
**Dépend de** : 39-11b, **38-11c** (rapprochement PayPal : rattachement en devise, routes de revue et d'acceptation,
écritures de frais, de règle et de conversion ; fiche `drafted` lue dans le worktree de l'Epic 38 le 2026-11-20).
**Suivie par** : 39-11d.*

## Story

En tant que **Comptable**, je veux qu'après l'import des relevés Wise Kesh me **propose** l'écriture ou le
rattachement de chaque ligne — frais au compte de frais, conversions au cours de Wise, rattachement à l'écriture déjà
passée de l'autre côté (virement depuis ma banque, versement Stripe), achats et paiements par mes règles — et me liste
ce qu'il ne sait pas rapprocher, afin de tenir mon compte Wise sans saisie et sans rien compter deux fois.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11c-1` | Une ligne de frais (`FEE-…`) se propose au compte de frais du compte Wise ; sans compte de frais, elle reste « à examiner ». | AC-2 |
| `FR-39-11c-2` | Une conversion avec un côté en CHF se propose en **une** écriture (ligne CHF au montant de Wise, ligne en devise), ses deux transactions rattachées ; entre deux devises étrangères, en deux écritures par le compte de conversion, au même CHF. | AC-3 |
| `FR-39-11c-3` | Une ligne dont l'autre côté est déjà passé (virement entre ses comptes, versement Stripe de l'Epic 40) se propose au rattachement à cette écriture. | AC-4 |
| `FR-39-11c-4` | Un versement Stripe n'est jamais écrit ni affecté par une règle ; sans écriture de versement, il reste « à examiner ». | AC-4 |
| `FR-39-11c-5` | Les autres lignes passent par les règles d'affectation (filtres de 38-11b et 38-11c) ; le reste « à examiner ». | AC-5 |
| `FR-39-11c-6` | Les écarts sont listés avec leur motif. | AC-6 |
| `NFR-39-11c-1` | Aucune écriture d'office ; acceptation en lot (*Pattern batch*, `FailedProposal` par proposition). | AC-1, AC-7 |
| `NFR-39-11c-2` | Aucun code TVA ; aucune migration ; moteur de 38-11c réutilisé, non recopié. | AC-1, AC-8 |

## Critères d'acceptation

1. **AC-1 — Où vivent les propositions.** 38-11c2 **ne passe pas** par le rapprochement CHF existant (`get_proposals`
   `crates/kesh-api/src/routes/reconciliation.rs:1091`, saut hors CHF ; `accept_one_rule` `:4191`, refus hors CHF
   `:4385-4396` : 38-11c1 AC-1 les laisse CHF) : elle a ses routes `GET /api/v1/paypal/review` et `POST
   /api/v1/paypal/accept`, une revue pure et un moteur d'acceptation (verrous des comptes touchés par identifiant
   croissant, point de sauvegarde par proposition, rejugement sous verrou, écritures `fee`, `rule`, `conversion`,
   rattachement par `link_entry_in_tx`, `mark_reconciled_in_tx`, `fx_failed_proposal`). Wise suit **le même
   chemin** : `GET /api/v1/wise/review?account=<libellé>&fiscalYearId=<id>` (Comptable et plus, clés d'API admises en
   lecture, lecture seule) et `POST /api/v1/wise/accept` (Comptable et plus, clés `read-write`), **gestionnaires
   minces** sur le moteur de 38-11c2 **paramétré par le prestataire et par la clé de rattachement** (`provider_review.rs`, `provider_accept.rs` ; classement et revue propres à Wise, AC-2 à AC-6 ;
   écritures et acceptation communes) — le moteur est servi générique par 38-11c2 (AC-5, AC-7) : cette story n'a plus à le généraliser (G10 ne vaut que si le développement de 38-11c2 s'en écarte, T0). Réponses de la forme de 38-11c (`proposals`, `gaps`,
   `toReview`, `counts` ; `{ accepted, failed }`), *Pattern batch* (`CLAUDE.md`, `FailedProposal {
   bank_transaction_id, error_code, details }`, `routes/reconciliation.rs:576`, HTTP 200 ; 400 global seulement pour
   un corps mal formé, vide, de plus de `MAX_IN_CLAUSE_IDS` propositions ou portant deux fois la même transaction) ;
   proposition périmée → `failed[]` sous `PROVIDER_PROPOSAL_STALE` (code générique servi par 38-11c2, Q-39-10). `POST /api/v1/wise/accept` à `LIB_ROUTES` (`audit_route_registry.rs:440`, clé `wise::accept_proposals`) en
   **`Traced, Rejouee`**, comme `paypal/accept` (38-11c AC-7) : le handler appelle l'enveloppe `retry_app_on_deadlock`
   (`crates/kesh-api/src/retry.rs:40`), **jamais `retry_with`** (`RETRY_WITH_AUTORISE = ["post_accept"]`, `audit_route_registry.rs:1303`). ⚠️ Le
   handler **ne s'appelle pas `post_accept`** : l'exemption désigne une fonction par son **seul nom** (angle mort (d),
   `audit_route_registry.rs:376-378`), si bien qu'une `fn post_accept` de `routes/wise.rs` pourrait appeler `retry_with`
   sans que le volet (c bis) (`no_route_calls_retry_with_except_post_accept`, `:1707`) la refuse ; nom au choix de la
   reprise s'il est aligné sur celui de PayPal, jamais `post_accept`. Et un interblocage (`is_app_deadlock`) ou un échec de `ROLLBACK TO SAVEPOINT` **remonte** en
   `AppError` au lieu d'entrer dans `failed[]` — le moteur de 38-11c le fait une fois, la route Wise en hérite, et un test
   la prend en défaut sur le modèle de `rejeu_interblocage_e2e.rs` ; compte de la partition et phrase du manuel
   d'administration recomptés comme en 39-11b AC-3 ; trace par proposition
   acceptée (`provider.proposal_accepted`, trace générique servie par 38-11c2), sous l'acteur, inscrite à
   `audit_acteur_garde.rs`. La revue porte sur les transactions **pending** des comptes bancaires `provider = 'wise'`
   du libellé, datées dans l'exercice ; refus 404 compte Wise introuvable (39-11b), 404 `FISCAL_YEAR_NOT_FOUND`.
2. **AC-2 — Classement et frais** (fonction pure neuve `crates/kesh-reconciliation/src/wise.rs`, `classify(tx) ->
   WiseKind`, sur `provider_type` — le préfixe d'identifiant posé par 39-11b —, le signe et `counterparty_name`) :
   `Fee` (préfixe commençant par `FEE-`) ; `Conversion` (préfixe `BALANCE` ou `CONVERSION_ORDER`) ; `StripePayout`
   (montant > 0 et `counterparty_name` contenant « stripe », sans casse) ; `Other`. Chaque ligne n'a qu'un genre, dans
   cet ordre. **Frais** : proposition `fee` de 38-11c (débit compte de frais `fee_account_id`, crédit compte du plan
   du compte Wise, montant `-amount` en devise, ligne en devise au cours mensuel de la société, libellé « Frais Wise
   <devise> · <identifiant> ») ; compte de frais absent → « à examiner », motif `fee_account_missing`.
3. **AC-3 — Conversions** (décision (1) de l'orchestrateur, règle commune aux trois prestataires, Q-39-12 tranchée).
   Paire = deux transactions **pending** de comptes bancaires `wise` **du même libellé**, même `transaction_id`, genre
   `Conversion`, l'une débit, l'autre crédit, devises différentes (les relevés 2024 : chaque conversion est dans deux
   fichiers, frais à part dans la devise source). Proposition `conversion { debitTransactionId, creditTransactionId }`,
   exécutée par le **plan et le constructeur de conversion de 38-11c AC-8** — `plan_conversion` (`kesh-reconciliation/src/conversion.rs`) et
  `journal_entry_conversions::create_provider_conversion_in_tx` (le même code pour PayPal, Wise et Stripe ; relire leurs noms au T0) :
   - **un côté en CHF** → **une** écriture du journal *Banque* : ligne CHF au montant de la transaction CHF, ligne du
     compte en devise au montant en devise avec `foreign = { amount, rate = rate_from_chf(amount, CHF) }`
     (`crates/kesh-core/src/accounting/fx.rs:266`, provenance `manual`) — le CHF de Wise, jamais un cours de l'AFC ;
     écriture équilibrée sans différence de change ; `rate_from_chf` en échec → `failed[]` ; à l'acceptation, **les
     deux** transactions passent `reconciled` sur cette écriture (`mark_reconciled_in_tx` de 38-11c, une fois chacune).
   - **deux devises étrangères** (EUR↔USD) → deux écritures par le compte de conversion de la société
     (`companies.fx_transit_account_id`, `crates/kesh-db/src/entities/company.rs:205`), par `create_conversion_in_tx`
     (`crates/kesh-db/src/repositories/journal_entry_conversions.rs:261`, aujourd'hui privée : appelée par le constructeur de
     38-11c, `complete_difference = false`) : CHF = montant source au cours mensuel retenu de la société
     (`find_company_rate_in_tx`, `repositories/exchange_rates.rs:365`), cible au même CHF par `rate_from_chf` ; cours
     absent → `failed[]` `RATE_UNAVAILABLE` ; compte de conversion absent → `failed[]` `FX_ACCOUNT_INVALID` ; chaque
     transaction rattachée à son écriture.
   - ⚠️ **L'autre côté d'une conversion à une écriture.** Une écriture qui porte deux transactions de comptes
     différents doit être reconnue « virement entre deux comptes propres » par l'annulation, sinon annuler un côté
     contre-passerait l'écriture et laisserait l'autre rapprochée d'une écriture annulée. Le prédicat unique
     `other_side_predicate` (`crates/kesh-db/src/repositories/reconciliation_link.rs:698-709`, `bt.amount =
     -(movement)`, de même devise seulement) gagne **une** alternative : transaction d'un autre compte bancaire, de
     **devise différente** de `t` et de **signe opposé** au mouvement (`SIGN(bt.amount) = -SIGN(mouvement)`), exclusions des
     règlements inchangées ; la condition 3 (mouvement de l'écriture sur le grand livre de `t`) borne toujours le rattachement.
     Toutes ses dérivées en héritent (`linkable_side` `:739`, `transfer_other_side` `:773`, `find_matched_elsewhere` `:896`,
     décompte de page) : annuler un côté ne retire que son lien (34-11b). La devise de `t` devient un paramètre de plus,
     **commun à toute la page** (devise du compte du plan, comme `t.bank_account_id`). **Inerte pour l'existant** : tant qu'aucun compte
     bancaire n'est en devise (seuls les comptes prestataires peuvent l'être, C-37-9), toute transaction liée est en CHF
     (condition 1) et la seconde branche ne s'active jamais — tests existants verts sans changement d'assertion.
     ⚠️ **Texte unique, porté par la 38-11c1 (AC-1b) pour les trois prestataires** (décision de l'orchestrateur du 2026-11-20 ;
     40-7c ne l'écrit qu'en dernier recours, si ni 38-11c1 ni 39-11c ne l'ont écrit, sans condition de devise isolée, kesh-wt40 fiche 40-7c AC-1) : `(bt.bank_account_id
     <> ? AND ((bt.currency = ? AND bt.amount = -(M)) OR (bt.currency <> ? AND SIGN(bt.amount) = -SIGN(M))) AND
     exclusions)` — première branche : même devise et montant opposé exact (virement, versement Stripe) ; seconde : les
     deux côtés d'une conversion. Cette story le **réutilise** et en teste les cas Wise ; elle ne l'écrit qu'à défaut (T0 :
     38-11c1 développée sans AC-1b), auquel cas l'epic qui arrive le second reprend `main`, relit le texte et garde les tests de
     l'autre ; le test d'égalité de la 34-4 couvre un compte en devise et une paire de devises différentes.
     **Le plafond « une transaction liée au plus »** de `linkable_side` (doc `:730-737`) reste inerte : chaque branche
     impose `SIGN(bt.amount) = -SIGN(M)` ; deux transactions liées à l'écriture sont de signes opposés, une troisième ne
     peut l'être des deux (démonstration à écrire au Change Log de développement) ; un test de comportement (troisième
     transaction d'un compte qui partage le grand livre, refusée) le garde quand même. Porté par **38-11c1** (AC-1b), avant le
     constructeur de 38-11c2 — **c'est 38-11c2 qui livre la première conversion à une écriture (PayPal)** : sans l'alternative,
     annuler un côté d'une conversion PayPal contre-passe l'écriture et laisse l'autre côté rapproché d'une écriture annulée ;
     si 38-11c1 est développée sans l'AC-1b, cette story le porte (dans la forme ci-dessus). Changement de `kesh-db` : gate complet en boucle.
   - Moitié manquante (l'autre devise pas encore importée, ou déjà rapprochée) → écart `conversion_half_missing`.
4. **AC-4 — Autre côté déjà passé : virements entre ses comptes, versements Stripe** (G7, G8 ; décision (4) de
   l'orchestrateur). Pour une ligne `StripePayout`, ou `Other` de préfixe `TRANSFER` (virement, dépôt : seuls genres qui peuvent être l'autre côté
   d'un virement entre ses comptes ; un achat par carte ne l'est jamais et n'ouvre pas la lecture) : `find_matched_elsewhere`
   (`reconciliation_link.rs:896` : les écritures rattachables à `t` **déjà rapprochées** d'une transaction d'un autre
   compte bancaire, l'autre côté d'un virement, 34-4 et 34-11b), en devise par la levée de 38-11c1 AC-1 →
   **exactement une** candidate à 5 jours au plus — champ `EntryCandidate::days_apart` (`reconciliation_link.rs:552`,
   `ABS(DATEDIFF(e.entry_date, date de paiement de t))`) que la fonction rend **triée sans filtrer** (« Trie, ne filtre
   pas ») : le seuil de 5 jours s'applique en Rust sur la liste rendue (50 au plus, `ENTRY_CANDIDATES_LIMIT`), les
   candidates plus lointaines ne comptent ni pour « une » ni pour « plusieurs » → proposition `link_entry { bankTransactionId,
   journalEntryId }`, exécutée par `link_entry_in_tx` (`:1156`) comme le `link_entry` de 38-11c2 (une seule fonction) ;
   coût **mesuré** au développement (une lecture par transaction concernée, sur 20 000 écritures et 300 transactions `TRANSFER`, au Change Log) ;
   au-delà d'une seconde, lecture par page sur le patron de `count_already_booked_for_page` (`:959`) ; plusieurs → « à examiner », motif `ambiguous_link` (candidates nommées, choix à l'écran par `POST …/link-entry`,
   `lib.rs:1046`). Cas couverts sans code propre : le virement passé depuis la banque (relevé CAMT rapproché) et
   l'écriture du versement de l'Epic 40 (rapprochée de la transaction du compte Stripe, C-40-K5) — la transaction Wise
   en devient l'autre côté, jamais une seconde écriture. **Cette story seule** propose le rattachement de la
   réception Wise à l'écriture du versement : 40-7c ne propose aucune réception (depuis sa P2, elle liste l'écart
   `payout_without_receipt` tant que la réception n'est pas rattachée, et l'écart disparaît quand ce rattachement est
   accepté). Sans candidate : `StripePayout` →
   « à examiner », motif `stripe_payout_unmatched` (**jamais** de règle ni d'écriture de vente, écrit au manuel par
   39-11d) ; `Other` → AC-5.
5. **AC-5 — Règles et reste.** `first_matching_rule` (`crates/kesh-reconciliation/src/rules.rs:85-95`, `rule_matches`
   `:46-70`) avec les **filtres facultatifs** de 38-11b et 38-11c (`provider_type`, `counterparty_email`, `direction`,
   décision (3) : aucun type de critère neuf) → proposition `rule` de 38-11c (contrepartie CHF, ligne Wise en devise au
   cours mensuel, **sans code TVA**, `increment_applied_count_in_tx`). Les critères existants suffisent à Wise :
   `counterparty_contains` / `counterparty_exact` (marchand, bénéficiaire, payeur), `iban_exact` (IBAN du
   bénéficiaire), `reference_contains`, avec le filtre `provider_type` (`CARD`, `TRANSFER`…) et le sens. Un virement
   vers sa propre banque dont l'autre côté n'est pas encore passé s'affecte **par une règle** (IBAN ou nom → compte du
   plan de la banque) ; l'autre côté s'y rattachera à son import (34-11b) — Q-39-15 tranchée ainsi. Aucune règle →
   « à examiner » (motif `no_rule`).
6. **AC-6 — Écarts** (dans `gaps` de la revue, calculés, rien de stocké) : `conversion_half_missing` ;
   `fee_without_origin` (ligne `FEE-x` sans transaction `x` sur le même compte bancaire) ; `balance_gap` (pour chaque
   import des comptes du libellé dont la période tombe dans l'exercice et dont `opening_balance` n'est pas `NULL`, **la règle
   de 39-11b AC-5, telle quelle** : aucun autre relevé du compte ne couvre son **premier jour** (`overlapping_statements(…,
   period_from, period_from)`, hors lui-même) et `previous_statement_closing` (`crates/kesh-db/src/repositories/bank_imports.rs:358`,
   relevé à solde dont `period_to` < `period_from`) en rend un dont la clôture diffère de l'ouverture — l'import juge à son instant, la revue relit cette règle sur
   l'état présent des relevés du compte (un relevé importé ou saisi depuis peut changer la réponse de l'import, rien n'est stocké) ; les doublons ne se relisent pas, ils ne sont pas stockés). Chaque écart et chaque « à examiner » : transaction, date, montant, devise, motif.
7. **AC-7 — Acceptation.** Corps `{ account, proposals: [ {type, …} ] }` (`fee`, `conversion`, `link_entry`, `rule`) ;
   chaque proposition **rejuge** son classement sous verrou (transaction encore `pending`, même genre, même paire, même
   candidate, règle qui correspond toujours), sinon `failed[]` périmée ; refus du contrôle central des lignes en devise
   → `failed[]` par `fx_failed_proposal` (`routes/reconciliation.rs:763`) ; refus du rattachement (`LinkRefusal`) →
   `failed[]` sous leurs codes existants. Réponse `{ accepted: [{ type, bankTransactionIds, journalEntryIds }], failed
   }`.
8. **AC-8 — Aucune migration, i18n.** Genres et motifs calculés ; libellés des motifs et codes neufs aux quatre
   locales (`crates/kesh-i18n/locales/*`), `crates/kesh-api/src/errors.rs`.

## Amendement du 21.11

Décisions du responsable de l'epic du 21.11 (signature des epics), intégrées avant le développement ; elles
l'emportent sur les AC ci-dessus là où elles diffèrent.

- **A-1 — Paiement par carte converti : une écriture groupée** (décision (b)). Le CSV importé est celui « avec frais
  indiqués séparément », un fichier par solde. Un paiement par carte `CARD-<id>` est **converti** quand au moins une de
  ses lignes porte le segment de conversion écrit par l'import (`DE→CHF <montant> au cours <cours>`, 39-11b) ; ses
  lignes `CARD-<id>` (une par solde débité) et `FEE-CARD-<id>` **en attente**, sur tous les soldes du libellé, sont
  regroupées avant toute proposition et sortent du classement ligne à ligne (ni frais, ni règle). Proposition
  `card_group { bankTransactionIds, ruleId, counterpartyAccountId }` : la règle est la première qui correspond à la
  ligne principale (celle en CHF, à défaut la `CARD-<id>` de plus petit identifiant), de contrepartie **en CHF**.
  Écriture unique du journal *Banque* : **la charge au montant du marchand** (somme des lignes en CHF et des montants
  CHF des segments de conversion, frais exclus), **chaque solde Wise crédité de sa part** (ses lignes `CARD-` et
  `FEE-CARD-`, au cours mensuel retenu pour un solde en devise), **la différence de change sur sa ligne** (compte de
  perte ou de gain de change de la société ; absent → `failed[]` `FX_ACCOUNT_INVALID`). Toutes les transactions du
  groupe passent `reconciled` sur cette écriture ; la règle compte une application. Refus de forme (une ligne
  positive, un segment vers une autre devise que le CHF, un segment absent sur une ligne en devise) → « à examiner »,
  motif `card_group_unsupported` ; aucune règle → `card_group_no_rule`. Le marchand en devise étrangère (segment vers
  une autre devise que le CHF) n'est pas un groupe : sa ligne suit les règles, ses frais le compte de frais (jamais vu
  en partielle). Rejugement à l'acceptation : l'ensemble des lignes en attente du paiement, relu sous les verrous, est
  celui de la proposition (sinon périmée, `card_group_changed`).
- **A-2 — Annuler une ligne d'un paiement groupé** : l'écriture est contre-passée (aucune ligne n'est « l'autre
  côté » d'une autre : toutes au débit) et **toutes** les transactions encore rattachées à l'écriture redeviennent en
  attente avec elle, dans le même geste (`reconciliation_cancel::cancel_in_tx`, étape (5)), au lieu de rester
  rapprochées d'une écriture contre-passée. Inerte pour l'existant : hors groupe, une écriture qui n'est pas un retrait
  seul n'a qu'une transaction rattachée. Changement de `kesh-db` : gate complet.
- **A-3 — Ce qu'on n'accepte pas se traite à la main par l'API** (décision (c)) : une proposition non acceptée reste
  en attente et reparaît à la revue suivante ; une ligne se traite à la main par une écriture
  (`POST /api/v1/journal-entries`, ligne en devise comprise) puis son rattachement
  (`POST /api/v1/reconciliation/transactions/{id}/link-entry`), une écriture par ligne. Un test HTTP le joue sur un paiement
  groupé. L'écran viendra plus tard.
- **A-4 — Contrepartie d'une règle** (décision (d)) : en CHF ou dans la devise de la transaction, déjà dans le moteur
  (`matching_rule`, `COUNTERPARTY_CURRENCY_MISMATCH`) ; en CHF seulement pour un paiement groupé.
- **A-5 — Classement précisé** (constaté sur les fixtures de 39-11a) : un dépôt porte aussi le préfixe `BALANCE`
  (« Argent reçu de Stripe », type de détail `DEPOSIT`) ; `Conversion` exige donc, en plus du préfixe `BALANCE` ou
  `CONVERSION_ORDER`, le segment de conversion ; le versement Stripe se juge **avant** la conversion. Ordre : frais,
  versement Stripe, conversion, virement (`TRANSFER`), autre. Le virement est un genre propre (`transfer`) : la revue
  commune le laisse à Wise, qui lit l'autre côté déjà passé puis, à défaut, les règles (AC-4, AC-5).
- La correction du lecteur pour un même `CARD-<id>` débité puis recrédité (`DuplicateId`) est la story 39-11a2 ; les
  tests de cette story n'en ont pas besoin.

## Tâches

- [x] T0 — Lire 38-11c telle que développée ; noter chaque fonction réutilisée ou généralisée (G10) ; constater si le
  constructeur de conversion à une écriture et le prédicat étendu y sont.
- [x] T1 — `wise.rs` : `classify`, revue pure (AC-2, AC-4 à AC-6), tests purs.
- [x] T2 — Conversions : paire, constructeur commun, prédicat de l'autre côté s'il manque (AC-3).
- [x] T3 — Routes minces de revue et d'acceptation sur le moteur de 38-11c ; registre, trace (AC-1, AC-7).
- [x] T4 — Codes, i18n (AC-8) ; non-régression.

## Stratégie de tests

- **Purs** (`wise.rs`) : chaque genre sur des transactions construites ; préséance (une ligne de frais ou un versement
  Stripe n'est jamais une règle) ; paire de conversion (même ID, devises, sens, même libellé ; moitié manquante) ;
  candidate unique, aucune, plusieurs, à plus de 5 jours ; écarts.
- **HTTP** (`crates/kesh-api/tests/wise_reconciliation_e2e.rs`) sur les fixtures de 39-11a importées par 39-11b, cours
  mensuels de 2020 posés : frais acceptés au bon CHF ; conversion CHF→EUR → **une** écriture, CHF = montant de Wise,
  deux transactions rapprochées ; annulation d'un côté → seul son lien retiré, l'écriture reste ; second côté
  rattachable à nouveau ; troisième transaction refusée ; EUR→USD → deux écritures, même CHF,
  compte de conversion soldé ; virement vers l'IBAN fictif de la société après import et rapprochement d'un relevé CAMT
  de ce compte → `link_entry` accepté ; versement Stripe avec une écriture fictive de versement rapprochée d'une
  transaction d'un compte bancaire `stripe` posé en base → `link_entry` accepté, sans → `stripe_payout_unmatched` ;
  paiement par règle `counterparty_contains` filtrée `CARD` (jamais de lecture `find_matched_elsewhere` pour une ligne `CARD`) ; virement par règle `iban_exact` ; écarts listés ;
  proposition périmée ; interblocage injecté dans une proposition → rejeu, aucune ligne dans `failed[]` ; import chevauchant un autre
  → aucun `balance_gap` à la revue, import discontinu → un seul, **même quand un import postérieur chevauche sa fin** ;
  candidate unique à 5 jours parmi d'autres plus lointaines → `link_entry` ; handler d'acceptation qui n'est pas nommé
  `post_accept` (relu à la revue de code : le volet (c bis) ne le garde pas par le nom) ; clé `read` → 403 au `POST`. Somme des écritures par compte Wise = variation du solde Wise.
  Tests PayPal de 38-11c et de l'annulation 34-11b verts sans changement d'assertion.
- Gate complet en boucle (prédicat de `kesh-db`).

## Modules touchés (4)

`kesh-reconciliation` (`wise.rs`), `kesh-api` (routes `wise` minces, erreurs), `kesh-db` (prédicat de l'autre côté,
seulement si 38-11c ne le porte pas), `kesh-i18n`.

## Note de version

*Ajouté* — Les lignes d'un relevé Wise se rapprochent sur proposition : frais au compte de frais, conversions au cours
de Wise (une écriture quand un côté est en francs), rattachement à l'écriture déjà passée de l'autre côté (virement
depuis votre banque, versement Stripe), le reste par vos règles ; ce qui ne se rapproche pas est listé avec son motif.

## Notes de développement

- Ancienne 39-4 (classement des types Wise du relevé JSON, `superseded`) : la reconnaissance se fait ici par le
  préfixe de l'identifiant, langue de l'export indifférente (les descriptions du relevé sont traduites par Wise).
- La reconnaissance d'un virement interne par IBAN ou nom de la société (première rédaction) est remplacée par
  l'autre côté déjà passé, puis la règle : aucune heuristique propre, aucun motif `own_transfer_target_unknown`.
- ⚠️ **Taille** : la plus lourde des quatre ; tenable en une passe puisque 38-11c2 livre un moteur et un plan de conversion génériques
  (T0 le constate) ; sinon la généralisation lui revient et la story se découpe (11c1 : classement, revue, routes ; 11c2 : conversions et prédicat),
  à trancher à l'ouverture du développement, pas en cours de route.
- ⚠️ **Risque principal** : l'alternative du prédicat de l'autre côté (AC-3) touche un invariant de 34-11b qu'aucune
  contrainte ne défend ; ses tests (annulation d'un côté, troisième transaction refusée) sont la condition de sa clôture.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 3 HIGH, 4 MEDIUM, 2 LOW, remédiés dans la
  fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11c-rapprochement-wise-P1.md`). HIGH : propositions
  placées dans `GET /reconciliation/proposals` et `POST /reconciliation/accept`, que 38-11c laisse CHF (elle a ses
  routes) → routes `wise` minces sur le moteur de 38-11c ; conversion EUR↔USD « au cours AFC du mois » et repli à deux
  écritures pour un côté CHF → décision (1) appliquée ; annulation d'une conversion à une écriture qui contre-passait l'écriture en laissant l'autre côté rapproché
  (prédicat de l'autre côté étendu, testé). MEDIUM : virements internes par IBAN et nom (heuristique, `normalize_iban` privée,
  `rules.rs:112`) → `find_matched_elsewhere` puis règle ; versement Stripe par `find_entry_candidates` à ± 5 jours sans
  l'autre côté → décision (4), même chemin ; écarts dans une route à part → `gaps` de la revue ; `balance_gap` sans
  définition → règle de 39-11b. LOW : registre et trace d'audit ; motifs réduits (`own_transfer_target_unknown`,
  `fx_transit_missing` retirés, ce dernier rendu par `FX_ACCOUNT_INVALID` à l'acceptation).
- 2026-11-20 — validation P2 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 1 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/39-11c-rapprochement-wise-P2.md`). Remédiation P1 relue au code d'`origin/main`
  (`fd04099f`), à 38-11c et 40-7c lues dans leurs worktrees. MEDIUM : (1) `balance_gap` définie par un chevauchement que la
  revue ne peut pas lire (doublons non stockés) et qui, relue à la revue, signale à tort l'import qui en chevauche un autre →
  règle de 39-11b AC-5 (`overlapping_statements`) ; (2) `POST /wise/accept` sans statut `Rejouee` ni enveloppe de rejeu au
  registre, contre 38-11c AC-7 et 40-7c ; (3) l'alternative du prédicat de l'autre côté et le `AND bt.currency = …` de 40-7c AC-1
  réécrivent le même texte en sens contraires → un texte à deux branches, signal à 38 (38-11c livre la première conversion à une
  écriture sans l'alternative) et à 40 ; (4) `find_matched_elsewhere` lu pour toute ligne `Other` (une lecture lourde par
  transaction) → seulement `StripePayout` et `TRANSFER`, coût mesuré. LOW : constructeur de conversion nommé (`plan_conversion`,
  `create_provider_conversion_in_tx`) au lieu de `create_conversion_in_tx` « rendue publique ».
- 2026-11-20 — validation P3 ciblée (Claude Opus 5.5, contexte frais, commit `02ed58c6` seul) : 0 CRITICAL, 0 HIGH, 3 MEDIUM,
  1 LOW, remédiés (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P3-ciblee.md` ; rectifié le même jour : 2 MEDIUM,
  3 LOW, voir ci-dessous). MEDIUM : (1) `balance_gap`
  jugé sur le chevauchement de toute la période, dont la réponse à la revue diffère de celle de l'import (un relevé
  postérieur chevauchant la fin masque l'écart du début) → premier jour seul, comme 39-11b AC-5 ; (2) « le test refuse
  tout autre appel » à `retry_with` est faux pour un handler nommé `post_accept` (exemption par le seul nom, angle mort (d)
  de `audit_route_registry.rs:376-378`) → clé `wise::accept_proposals`, jamais `post_accept` ; (3) `days_apart` cité sans dire
  que c'est le champ d'`EntryCandidate` (`reconciliation_link.rs:552`), qui trie sans filtrer → seuil de 5 jours appliqué en
  Rust, nommé — **reclassé LOW** au rectificatif (le champ existe, la citation était juste, seule la nature du filtre manquait).
  LOW : phrase périmée « 40-7c AC-1 ajoute `AND bt.currency = …` » (40-7c écrit le texte à deux branches depuis `c8924cfd`)
  → texte porté par 38-11c, réutilisé ici, écrit seulement à défaut. Vérifié sans finding : enveloppe `retry_app_on_deadlock`
  (`retry.rs:40`), prédicat à deux branches (existant CHF inerte, plafond inerte : deux liens de signes opposés, une
  troisième ne peut être l'autre côté des deux ; annulation d'un côté), `find_matched_elsewhere` limité à `StripePayout` et
  `TRANSFER` (les dépôts Wise portent aussi le préfixe `TRANSFER`, relevés 2024 en agrégats).
  Rectificatif P3 (même passe) : LOW de plus — AC-4 disait « 40-7c propose le même rattachement depuis l'autre côté, le
  premier accepté rend l'autre périmé », faux depuis la P2 de la 40-7c (aucune réception proposée, écart
  `payout_without_receipt`) → la 39-11c seule propose le rattachement de la réception Wise à l'écriture du versement.
- 2026-11-20 — Validation P5 ciblée (Claude Sonnet 5.5, passe ciblée, contexte frais, rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11-P5-ciblee.md`) : 0 C/H/M, 2 LOW remédiés — G4, 39-11b AC-5 et 39-11c AC-6 disent la même chose (l'import juge à son instant, la revue relit la même règle sur l'état présent) ; 39-11c AC-3 : 40-7c n'écrit le texte qu'en dernier recours.
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : `PROVIDER_PROPOSAL_STALE` et `provider.proposal_accepted` servis (replis `WISE_PROPOSAL_STALE` et `wise.proposal_accepted` retirés) ; `link_sale` → `link_entry` (38-11c2) ; moteur servi générique par 38-11c2 (plus de généralisation G10 par défaut) ; rattachement en devise = 38-11c1 AC-1, prédicat de l'autre côté = 38-11c1 AC-1b (repli seulement si l'Epic 38 arrive après). Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.
- 2026-11-22 — développement (Claude Opus 5.5), branche `story/39-11c` partie d'`epic/39-wise` (`3a9861f0`) ;
  amendement du 21.11 écrit avant le code (`07f04c4e`). Voir le Dev Agent Record.
- 2026-11-22 — remédiation de la revue de code P1 (1 MEDIUM, 9 LOW) : garde `card_group_statements_missing` (M1), tests
  du gain de change et de la règle forgée (L1), échelle du montant CHF (L2), prédicats d'annulation alignés (L5), segment lu
  à sa place (L7), fixture sans raison sociale réelle (L4). Voir le Dev Agent Record ; passe P2 à lancer.
- 2026-11-22 — revue de code P2 (passe complète, une lentille ; 0 C, 0 H, 2 MEDIUM, 5 LOW, rapport
  `kesh-gate-logs/s39-11c/revue-p2-rapport.md`), remédiée (`5c62870f`) : part déjà rapprochée d'un paiement groupé
  (M1-P2, `card_group_partly_reconciled`), couverture d'une date par la continuité des relevés (M2-P2,
  `bank_imports::date_coverage`), solde en devise exigé quand ses relevés encadrent la date sans se suivre (écart au remède
  « chaque solde », limite écrite), L-a à L-d. Trend : P1 1 MEDIUM → P2 2 MEDIUM (défauts de la garde ajoutée en P1) ;
  passe P3 ciblée à lancer (`kesh-gate-logs/s39-11c/revue-p3-prompt.md`).
- 2026-11-22 — revue de code P3 (passe complète, une lentille ; 0 C, 0 H, 1 MEDIUM, 4 LOW, rapport
  `kesh-gate-logs/s39-11c/revue-p3-rapport.md`), remédiée (`3fa3c97f`) : M1-P3 traité par la documentation et un test,
  non par le code (le remède de code retiré : il contredisait l'amendement A-3, et la prémisse « même charge écrite deux
  fois » est fausse au chiffre) ; L1 corrigé (frais passé seul sans effet sur le groupe) ; L2 tests de `Gap` (relevé sans
  solde d'ouverture) et `NoPrevious` ; L3, L4 limites écrites, notées pour le manuel (fiche de la 39-11d). Trend : P1 1
  MEDIUM → P2 2 MEDIUM → P3 1 MEDIUM (une promesse de la documentation) ; passe P4 ciblée à lancer
  (`kesh-gate-logs/s39-11c/revue-p4-prompt.md`).
- 2026-11-26 — correctif demandé par le mainteneur sur la PR de l'Epic 39 : le frais `FEE-CARD-…` d'un paiement groupé
  allait, avec le change, au compte de perte de change ; il va désormais au compte de frais de son solde (contrôlé comme
  un frais isolé), la perte ou le gain ne portant que le change. Un solde à frais sans compte de frais met le groupe « à
  examiner » (`fee_account_missing`, le motif commun) et son acceptation est refusée (`PROVIDER_PROPOSAL_STALE`, même
  raison, `details.bankAccountIds`), rien n'est écrit. Vérifié sans changement : un même compte de frais peut servir à
  PayPal, Stripe et Wise s'il est en CHF (aucune unicité, CHECK ni garde ne l'interdit ; un compte de frais en devise
  ne sert que les soldes de sa devise ; test ajouté). Revue P1 ciblée (Sonnet) : 0/0/1/4, MEDIUM `sitesTotal` 3855 → 3856
  et LOW-1 (partage en CHF) corrigés ; LOW-2 à LOW-4 (texte statique, frais arrondi à zéro, couverture) laissés.

## Dev Agent Record

### T0 — ce que 38-11c2 livre et ce que la story réutilise

Moteur commun servi tel quel (aucune généralisation G10) : `load_review_in_tx`, `prepare_accept`, `accept_once`,
`PreparedAccept::finish`, `accept_common` (`fee`, `rule`, `link_entry`, `conversion`), `rejudge`, `check_counterparty`,
`month_rate`, `mark_reconciled`, `entry_refusal`, `db_refusal`, `stale` (`crates/kesh-api/src/routes/provider_accept.rs`) ;
`ProviderSpec` (`linkable`, `ruleable`, `conversion_unpaired` en `Placement::Gap`), `ProviderKind::Specific`,
`Review::map_proposals`, `matching_rule` (`crates/kesh-reconciliation/src/provider_review.rs`). Le plan et le
constructeur de conversion (`plan_conversion`, `create_provider_conversion_in_tx`) et le prédicat de l'autre côté à deux
branches (`other_side_predicate`, 38-11c1 AC-1b) **y sont** : cette story ne les touche pas (T2 réduit à la paire par
le même identifiant, `wise::conversion_match`). Deux retouches du moteur : `record_rule_applied` extrait d'`accept_rule`
(compteur et trace `reconciliation_rule.applied`, appelé aussi par le paiement groupé, DRY) et `failed` rendu public.
Champ `known_chf` de `ProviderEntry` : non employé (le paiement groupé écrit sa propre écriture à plusieurs lignes).

### Ce qui a été écrit

- `crates/kesh-reconciliation/src/wise.rs` (neuf, pur) : `classify` (ordre de l'amendement A-5), `exchange_of`
  (segment de conversion de 39-11b, forme exacte seulement), `SPEC`, `WiseProposal` (`card_group` + genres communs,
  désérialisation `untagged`), `card_groups` / `judge_card_group` / `shares_by_account` (A-1), `fees_without_origin`,
  `chf_rule_accounts`.
- `crates/kesh-api/src/routes/wise.rs` (neuf) : `GET /api/v1/wise/review` (revue commune, puis paiements groupés,
  autre côté déjà passé filtré à 5 jours en Rust, règles des virements, écarts `fee_without_origin` et `balance_gap`
  par `bank_imports::continuity_gap`) ; `POST /api/v1/wise/accept` (`accept_proposals`, enveloppe
  `retry_app_on_deadlock("wise::accept")`, jamais `retry_with`, nom ≠ `post_accept`) ; `impl EngineProposal for
  WiseProposal` ; `accept_card_group` (rejugement de l'ensemble des lignes, de la règle, contrepartie CHF, écriture
  charge / parts / différence de change, `FX_ACCOUNT_INVALID` sans compte de différence).
- `crates/kesh-db/src/repositories/provider_review.rs` : `pending_by_transaction_ids`, `opening_statements_in_period`,
  `fx_difference_accounts`.
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs` + `settlement_cancellation.rs` (A-2) : prédicat unique
  `PROVIDER_GROUP_SIBLING` (même `provider` et `provider_account`) ; à l'annulation d'une écriture contre-passée, les
  autres lignes du même paiement de prestataire redeviennent en attente (étape 5 bis, comptes ouverts contrôlés, une
  trace `reconciliation.cancelled` chacune, `groupedWith`) ; le rang 3 de la vue et du geste ne les compte plus
  comme blocage. Toute autre transaction rattachée reste refusée `MATCHED_BANK_TRANSACTION` (test existant
  `another_transaction_on_the_same_entry_still_refuses` vert sans changement — il avait rougi sur une première forme
  qui remettait en attente **toute** autre transaction : corrigée avant commit).
- Routes (`lib.rs`), registre (`audit_route_registry.rs` : `wise::accept_proposals` `Traced, Rejouee` ; totaux 153→154
  tracées, 161→162 routes, 31→32 rejouées, 165→166), inventaire des comptes clos (`bank_account_closed_inventory.rs` :
  44→45 sites, raison de `cancel_in_tx` complétée), manuel d'administration (« 154 des 162 routes », « 154 + 4 + 4 =
  162 », `.tex` seul), `docs/api-external.md` (section « Le rapprochement d'un compte Wise »).
- **AC-8 i18n** : aucun code d'erreur HTTP neuf (les refus réutilisent `PROVIDER_PROPOSAL_STALE`, `FX_ACCOUNT_INVALID`,
  `COUNTERPARTY_CURRENCY_MISMATCH`…) ; les motifs de la revue ne sont traduits nulle part (aucun écran, comme ceux de
  PayPal) : aucune clé ajoutée. Aucune migration.

### Tests (recomptés à `c07265c1`, périmètre `3a9861f0..HEAD`)

- 13 tests purs neufs (`#[test]` de `crates/kesh-reconciliation/src/wise.rs`).
- 9 tests HTTP neufs (`#[sqlx::test]` de `crates/kesh-api/tests/wise_reconciliation_e2e.rs`) : revue et acceptation
  du tout (12 propositions, soldes EUR 160.00 et USD 317.01 égaux à la variation Wise) ; conversion à une écriture,
  annulation d'un côté puis rattachement à nouveau ; paiement partiel groupé (écriture 97.95 / 86.60 / 13.89 USD à
  0.90 = 12.50 / perte 1.15 *(Annoté le 2026-11-26 : depuis le correctif de la #1595, la perte de change est de 1.11 et le frais de 0.04 va au compte de frais.)*), annulation d'une ligne (les trois en attente, vue non bloquée), proposition reparue ;
  traitement à la main par l'API (décision (c)) ; péremption, contrepartie, `FX_ACCOUNT_INVALID`,
  `card_group_no_rule` ; écarts (`conversion_half_missing` ×2, `fee_without_origin`, un `balance_gap` malgré un relevé
  postérieur chevauchant la fin, aucun pour le chevauchant) ; droits (clé `read` 403) et refus globaux ; interblocage
  injecté dans un paiement groupé rejoué (`wise::accept`), rien dans `failed[]` ; coût.
- **Mesure (AC-4)** : 300 virements en attente, 20 000 écritures sur le compte du plan du solde CHF, une lecture
  `find_matched_elsewhere` par virement : test en **3,86 s** au gate complet (3,91 s au ciblé), montage compris ;
  pas de lecture par page.
- Assertions existantes inchangées (PayPal 38-11c2, annulation 34-11b, `reconciliation_e2e`) ; seuls les compteurs des
  registres ont bougé.
- **Sondes par mutation** (`kesh-gate-logs/s39-11c/mutation.sh`, worktree restauré, `git status` vide après) : M1
  (étape 5 bis inerte) → `partielle_en_une_ecriture_puis_annulation` rouge ; M2 (lignes groupées laissées dans la
  revue commune) → même test rouge ; M3 (seuil des 5 jours à 100) → `revue_puis_acceptation_du_tout` rouge ; M4
  (ensemble relu non comparé) → **vert à la première sonde** : le test proposait le groupe sans la ligne de carte USD,
  que le jugement refusait déjà ; renforcé (`c07265c1`, groupe sans sa ligne de frais), M4 rouge, vert sans mutation
  (`mutation-m4.log`) ; M5 (conversion sans segment) → `chaque_genre_et_sa_preseance` rouge ; M6 (différence de change
  nulle) → `partielle_en_une_ecriture_puis_annulation` rouge. Une première passe des sondes (`mutation-invalide.log`)
  ne valait rien : deux fichiers `wise.rs` copiés sous le même nom, restauration croisée (fichier remis par
  `git checkout`, script corrigé).

### Gates (journaux `kesh-gate-logs/s39-11c/`)

- Ciblé `cible-1.sh` (filtre : kesh-reconciliation, libs kesh-api et kesh-db, binaires Wise, PayPal, registres,
  annulation, rattachement, règlements groupés) : 1900 / 1900 à `2870da3b` (`nextest-cible-2.log` → `cible-1.log`) ;
  deux passages précédents rouges, corrigés (compteurs des registres, nom de colonne `details_json`, ordre des
  identifiants du test, forme de l'étape 5 bis).
- **Backend complet** au dernier commit de code `c07265c1` : `scripts/test-fast.sh --ci` (fmt, clippy workspace
  `-D warnings`, nextest) **6488 exécutés, 6488 passés, 16 ignorés** (`gate-backend.log`).
- **Frontend** (aucun fichier du frontend touché) : `npm run check` 0 erreur (29 avertissements préexistants),
  `lint-i18n-ownership` vert, Vitest **4829 tests, 280 fichiers**, `build` vert, à `2870da3b` (les commits suivants ne
  touchent qu'un test Rust).
- **E2E** : aucun — la story n'ajoute aucune spec (décision du responsable pour cette nuit) ; **E2E complet et
  `gate-ci.sh` à la fin de l'epic**.

### Points relevés hors périmètre

- `docs/api-external.md` ne décrivait pas la revue et l'acceptation **PayPal** (38-11c2) : la section Wise les
  rappelle sans les documenter ; à reprendre par 38-11d ou à la fin de l'epic.
- La vue d'annulation (`GET /reconciliation/transactions/{id}`) ne dit pas que les autres lignes d'un paiement groupé
  reviendront en attente : l'écran le dira avec 39-11d.
- Un paiement partiel dont un seul relevé est importé (la ligne CHF sans la ligne convertie) passe par une règle au
  montant de la ligne : la somme reste juste quand l'autre relevé arrive (le groupe de la ligne convertie seule porte le
  reste du montant du marchand), mais en deux écritures.
- Fusion avec l'Epic 40 : `ProviderSpec` inchangé ; aucun conflit attendu hors des compteurs des registres (à
  recompter).

### Remédiation de la revue de code P1 (2026-11-22, rapport `kesh-gate-logs/s39-11c/revue-p1-rapport.md`)

0 CRITICAL, 0 HIGH, 1 MEDIUM, 9 LOW à la revue ; périmètre des décomptes ci-dessous : `b56fd531..HEAD`.

- **M1 (corrigé)** : une partielle dont un seul solde est importé ne donne plus deux écritures. Garde
  `missing_statement_balances` (`routes/wise.rs`) : pour chaque solde **en CHF** du compte Wise absent du groupe, un relevé
  importé doit couvrir la date du paiement (`bank_imports::overlapping_statements`) ; sinon revue « à examiner »
  `card_group_statements_missing` (`details.missingBankAccountIds`), et acceptation refusée `PROVIDER_PROPOSAL_STALE`
  (`details.reason = card_group_statements_missing`, mêmes identifiants). **Écart au remède proposé, à confirmer** : la
  garde ne porte que sur les soldes en CHF (une partielle débite d'abord le solde du marchand, en CHF), non sur chaque
  solde absent : le montage de test et l'usage réel importent CHF et USD sans EUR, et exiger un relevé EUR aurait mis
  toute partielle « à examiner ». Limite écrite (`docs/api-external.md`, entrée d'issue) : une partielle sur deux soldes
  en devise dont l'un manque, et un frais `FEE-CARD` importé plus tard par un second fichier du **même** solde que la carte,
  ne se décèlent pas. Aucune clé i18n : aucun écran n'affiche encore ces motifs (39-11d).
- **Sondes par mutation de la garde M1** (copie hors dépôt, `scratchpad/s39-11c-p1fix/sonde.sh`, journaux `m0` à `mC`) :
  copie non mutée verte (`partielle_dont_un_seul_solde_est_importe` 1/1) ; A (garde de la revue retirée) → rouge,
  `assertion left == right` à `wise_reconciliation_e2e.rs:1044` (nombre de propositions) ; B (garde de l'acceptation
  retirée) → rouge à `:1060` (refus de la proposition forgée) ; C (relevé jamais requis) → rouge à `:1044`. ⚠️ Deux pièges
  de la méthode, payés : (1) une première série de sondes ne compilait pas (`frontend/tests/e2e/fixtures` exclu de la copie) :
  aucune « tuée » comptée ; (2) la copie partageait la cible cargo de la worktree : cargo prend pour à jour des artefacts
  compilés depuis un autre chemin (empreintes indépendantes du chemin) — la copie mutée C a servi de binaire à la copie
  « non mutée » suivante, puis au premier gate de la worktree (test rouge à tort) ; remède : `touch` des sources avant chaque
  passe, et pour la worktree avant son gate. **Une copie de sonde prend sa propre cible cargo.**
- **Gate de la remédiation** (journaux `kesh-gate-logs/s39-11c/`) : ciblé (`wise_reconciliation_e2e`, `wise_*`,
  `reconciliation_e2e`, `settlement_*`, `textes_coherents`, `audit_route_registry`) 293 / 293 avant les sondes ; **backend
  complet** `scripts/test-fast.sh --ci` (fmt, clippy workspace `-D warnings`, nextest) **6491 exécutés, 6491 passés,
  16 ignorés** (`gate-backend.log`, après `touch` des sources, cf. plus haut ; 6488 + 3 tests HTTP neufs). Frontend : aucune
  clé i18n ajoutée, aucun fichier du frontend touché — non rejoué. Aucun E2E (aucune spec neuve).
- **L1** : trois tests HTTP — `partielle_dont_un_seul_solde_est_importe` (USD seul : « à examiner », forgée refusée, rien
  d'écrit ; le relevé CHF arrive : une proposition, **une** écriture), `partielle_gain_de_change` (cours 0.80 : 13.89 USD =
  11.11 CHF, gain 0.24 au crédit du compte de gain), `regle_forgee_sur_un_versement_stripe` (`kind_changed`). L'interblocage
  de deux annulations : test trop lourd, **entrée d'issue**.
- **L2** : `judge_card_group` refuse un montant CHF d'échelle > 2 (`merchant_scale`).
- **L3** : commentaire de `ProviderEntry.known_chf` corrigé (aucun appelant). La constante `DESCRIPTION_MAX_CHARS` de
  `routes/wise.rs` double celle de la 40-7c : **à unifier à la fusion** avec `ENTRY_DESCRIPTION_MAX_CHARS`.
- **L4** : « Stripe Payments Europe » remplacé par « Stripe Fictif SA » dans les deux ajouts de la story (test pur de
  `wise.rs`, relevé du test `ecarts`) ; les fixtures d'import des stories précédentes ne sont pas touchées.
- **L5** : `settlement_entry_cancel_blocker` aligné sur `status = 'reconciled'` (`NOT (bt.status = 'reconciled' AND …)`).
  Message de la vue et de la réponse sur le retour en attente des autres lignes : laissé à la 39-11d (écran). Interblocage
  des deux annulations et compte des autres lignes : **issue**.
- **L7** : `exchange_of` lit le **dernier** élément conforme (`rsplit`) : une description conforme, avant le segment,
  ne l'emporte plus.
- **L8, L9** : laissés (moteur partagé avec la 40-7c ; perf) — entrées d'issue. **L6** : échec fermé, conforme à A-5, laissé.
- Hors changement : la conversion **totale** (crédit du solde en devise au cours du mois, différence de change) reste
  celle de l'amendement A-1 ; question au responsable de l'epic.

### Remédiation de la revue de code P2 (2026-11-22, rapport `kesh-gate-logs/s39-11c/revue-p2-rapport.md`)

0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW à la revue. Commit de code `5c62870f` ; périmètre des décomptes : `be75a3a8..5c62870f`.

- **M1-P2 (corrigé)** : la garde du paiement groupé devient `card_group_blocker` (`routes/wise.rs`), **une** fonction pour
  la revue et l'acceptation, qui rend le motif et ses détails (mêmes des deux côtés : `toReview[].code`, ou
  `PROVIDER_PROPOSAL_STALE` avec `details.reason`). Motif 1, jugé d'abord : une ligne `CARD-<id>` ou `FEE-CARD-<id>` du
  même paiement **déjà rapprochée**, sur un solde du compte Wise → `card_group_partly_reconciled`
  (`details.reconciledBankTransactionIds`, `details.journalEntryIds`) ; lecture `provider_review::reconciled_by_transaction_ids`,
  factorisée avec `pending_by_transaction_ids` (`select_by_transaction_ids`, SQL de ce dernier inchangé). Élargi au **même**
  solde que le groupe (le remède disait « un autre solde ») : une ligne du paiement rapprochée seule l'est toujours à tort,
  quel que soit son solde. Sonde `p2_sonde_chf_d_abord` devenue le test `partielle_chf_d_abord_deja_rapprochee`.
- **M2-P2 (corrigé)** : motif 2, `card_group_statements_missing`, jugé par `bank_imports::date_coverage` (kesh-db, neuf,
  enum `DateCoverage`) : `Covered` si un relevé contient la date, ou si le premier relevé qui la suit prolonge sans écart le
  relevé à solde qui la précède (même règle de clôture et d'ouverture que `continuity_gap`) ; `Gap` (relevés de part et
  d'autre qui ne se suivent pas, ou relevé suivant sans solde d'ouverture), `NoPrevious`, `NothingAfter`. Tests
  `totale_apres_la_derniere_ligne_chf` (« à examiner », puis proposée et acceptée en une écriture une fois le relevé CHF
  suivant importé, continu) et `totale_releve_chf_suivant_discontinu` (reste « à examiner », acceptation forgée refusée).
- **Soldes en devise (recommandation de la revue, faite en partie)** : exigé du solde CHF sauf `Covered` (inchangé dans
  son principe) ; d'un solde en devise **seulement** sur `Gap`. Exiger tout solde qui a un relevé (la recommandation) aurait
  mis « à examiner » tout paiement postérieur au dernier relevé d'un solde **dormant** — un solde qui n'a plus bougé n'a
  aucun fichier à importer (rapport P2, M2) — ou antérieur au premier relevé d'un solde ouvert plus tard : jugé trop risqué
  pour des livres tenus ; **limite écrite** (L-a) : une partielle sur deux soldes en devise dont l'un n'a rien d'importé
  autour de la date se propose sans lui (écriture de la seule part présente) ; sa part arrivée ensuite tombe sous le motif 1,
  jamais une seconde écriture. Test `partielle_deux_devises_dont_une_absente` (la limite, puis le cas fermé par `Gap`).
- **L-a** : `docs/api-external.md` dit la règle de couverture, le motif neuf, les limites, et « jamais une seconde écriture »
  hors la limite du frais `FEE-CARD` (qui se propose seul, en écriture de frais) ; la phrase « jamais deux écritures »
  retirée. **L-b** : test pur `montant_du_marchand_au_centime` (zéros de queue admis, centime fractionnaire refusé, sur la
  ligne CHF et dans le segment). **L-c** : test pur `segment_le_dernier_element_conforme`. **L-d** : test HTTP
  `annulation_bloquee_par_une_ligne_liee_non_rapprochee` (ligne sœur forgée en attente avec son écriture :
  `MATCHED_BANK_TRANSACTION` à la vue, `cancelBlockedDocumentId` nommé, geste refusé). **L-e** : rien à faire.
- **Tests** (recomptés aux deux bornes) : `#[sqlx::test]` de `wise_reconciliation_e2e.rs` 12 → 17 ; `#[test]` de
  `kesh-reconciliation/src/wise.rs` 13 → 15. Aucune clé i18n (aucun écran n'affiche ces motifs, 39-11d) ; le manuel
  utilisateur ne décrit pas encore le rapprochement Wise (grep `card_group`, `paiement group` : aucun site) ; aucune migration.
- **Sondes par mutation** (copie `git archive` hors dépôt, cible propre `kesh-targets/s39-11c-mut`, base
  `kesh_s3911c_mut`, supprimées ; journal `kesh-gate-logs/s39-11c/mutation-p2fix.log`) : témoin non muté 9/9 vert avant et
  après ; chaque mutation tuée sur assertion nommée — A (motif 1 inerte) → `partielle_chf_d_abord_deja_rapprochee`
  `:1144` ; B (continuité jamais retenue) → `totale_apres_la_derniere_ligne_chf` `:1229` ; C (`Gap` pris pour couvert) →
  `totale_releve_chf_suivant_discontinu` `:1278` et `partielle_deux_devises_dont_une_absente` `:1354` ; D (`Gap` d'un solde
  en devise non exigé) → `partielle_deux_devises_dont_une_absente` `:1354` ; E (prédicat d'annulation sans
  `status = 'reconciled'`) → `annulation_bloquee_…` `:1396` ; F (échelle > 3) → `montant_du_marchand_au_centime`
  `wise.rs:783` ; G (`split` au lieu de `rsplit`) → `segment_le_dernier_element_conforme` `wise.rs:796`.
- **Gates** : ciblé (`cible-1.sh` : fmt, clippy workspace `-D warnings`, nextest des binaires touchés) **1910 / 1910**,
  8 ignorés (`cible-p2fix.log`) ; **backend complet** au commit de code `5c62870f` : `scripts/test-fast.sh --ci` (fmt, clippy
  workspace `-D warnings`, nextest) **6498 exécutés, 6498 passés, 16 ignorés** (`gate-backend.log` ; 6491 + 7 tests neufs),
  lancé avec `KESH_NO_MEMGUARD=1` sous `heavy.sh` : le bus systemd utilisateur de la station refusait la connexion
  (`mem-guard` ne pouvait pas créer de portée ; premier passage arrêté là, rc 1, aucun test joué). Frontend non touché, non rejoué. Aucun E2E (aucune spec
  neuve) ; E2E complet et `gate-ci.sh` à la fin de l'epic.

### Remédiation de la revue de code P3 (2026-11-22, rapport `kesh-gate-logs/s39-11c/revue-p3-rapport.md`)

0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW à la revue. Commit de code `3fa3c97f` ; périmètre des décomptes : `08c5f6c5..3fa3c97f`.

- **M1-P3 (documentation et test, remède de code retiré)** : le remède demandé — juger aussi les propositions `rule` et
  `fee` d'une ligne `CARD-`/`FEE-CARD-` contre ses sœurs rapprochées — a été écrit (contrôle commun à la revue et à
  l'acceptation), puis retiré : il faisait rougir `partielle_traitee_a_la_main_par_l_api`, le test de l'amendement A-3
  (une ligne d'un paiement groupé se traite à la main, « une écriture par ligne », puis le reste suit la revue : la ligne
  CHF par règle). Prémisse vérifiée au chiffre : il n'y a **pas** de seconde écriture de la même charge. Le groupe passé
  sans la ligne CHF a pour marchand la seule part USD (11.35) ; la ligne CHF par règle, sa seule part (86.60) ; les deux
  écritures font la charge (97.95), les crédits CHF (86.60) et USD (12.50, 13.89 USD) et la perte de change (1.15 *(Annoté le 2026-11-26 : depuis le correctif de la #1595, la perte de change est de 1.11 et le frais de 0.04 va au compte de frais.)*) de
  l'écriture unique de `partielle_en_une_ecriture_puis_annulation`. La sonde `p3_sonde_usd_d_abord_puis_chf` devient le
  test `partielle_usd_d_abord_puis_ligne_chf`, qui l'affirme sur le mouvement des quatre comptes. La phrase « jamais une
  seconde écriture » de `docs/api-external.md` devient vraie autrement : deux écritures possibles, **aucun montant compté
  deux fois** ; la doc de `card_group_blocker` dit pourquoi le motif 1 ne vaut que pour un groupe. Décision soumise à
  l'orchestrateur (voir le rapport de remédiation).
- **L1 (corrigé)** : motif 1 réduit aux lignes de carte (`reconciled_card_lines`, `routes/wise.rs`) : un frais
  `FEE-CARD-<id>` passé seul ne bloque plus le groupe, proposé sans lui (son écriture crédite le solde USD de la seule
  carte, 13.85) ; un frais importé après le groupe se propose seul (`fee`). Tests `frais_d_abord_puis_la_carte`,
  `frais_apres_le_groupe_passe_seul`.
- **L2 (tests)** : `totale_releve_chf_suivant_sans_solde_d_ouverture` (relevé suivant à `opening_balance` NULL, `Gap`),
  `totale_sans_releve_chf_anterieur` (`NoPrevious`, solde CHF exigé), `partielle_solde_en_devise_ouvert_apres`
  (`NoPrevious`, solde en devise non exigé). M5 (retrait de `period_to < date`) : mutant équivalent, laissé.
- **L3, L4 (écrits)** : `docs/api-external.md` et doc de `card_group_blocker` — continuité tenue pour preuve (lignes qui
  s'annulent, solde à zéro, deux relevés du même jour), solde en devise épars qui met tout paiement « à examiner » ;
  section « Limites du rapprochement à reprendre au manuel » de la fiche de la 39-11d.
- **Tests** (recomptés aux deux bornes) : `#[sqlx::test]` de `wise_reconciliation_e2e.rs` 17 → 23. Aucune clé i18n,
  aucun écran, aucune migration.
- **Sondes par mutation** (copie `git archive` dans `kesh-gate-logs/s39-11c/p3fix/src`, cible propre
  `kesh-targets/s39-11c-mut`, base `kesh_s3911c_mut`, une mutation par invocation de `heavy.sh`, `CARGO_BUILD_JOBS=4` ;
  copie, cible et base supprimées ; journal `kesh-gate-logs/s39-11c/p3fix/mutations.log`) : témoin 7/7 vert avant et
  après ; tuées sur assertion nommée — C (frais compté au motif 1) → `frais_d_abord_puis_la_carte` `:1456` ; M7
  (ouverture absente prise pour couverte) → `totale_releve_chf_suivant_sans_solde_d_ouverture` `:1564` ; N1 (`NoPrevious`
  pris pour couvert) → `totale_sans_releve_chf_anterieur` `:1602` ; N2 (`NoPrevious` d'un solde en devise exigé) →
  `partielle_solde_en_devise_ouvert_apres` `:1636`.
- **Gates** : ciblé (`p3fix-cible.sh` : fmt, clippy workspace `-D warnings`, nextest `wise_reconciliation_e2e` et
  `kesh-reconciliation wise::`) **38 / 38** (`p3fix-cible.log`) ; **backend complet** au commit de code `3fa3c97f` : `scripts/test-fast.sh --ci` (fmt, clippy workspace `-D warnings`,
  nextest) **6504 exécutés, 6504 passés, 16 ignorés** (`p3fix-gate-backend.log` ; 6498 + 6 tests neufs), `KESH_NO_MEMGUARD=1`
  sous `heavy.sh`. Frontend non touché, non rejoué ; aucun E2E (aucune spec neuve) ; E2E complet et `gate-ci.sh` à la fin
  de l'epic.

- **Revue de code P4 (ciblée, une lentille sur `3fa3c97f`)** : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 2 LOW, remédiés sans code de
  production (rapport `kesh-gate-logs/s39-11c/revue-p4-rapport.md`). L1-P4 : la phrase de `docs/api-external.md` bornée au
  cas de la ligne CHF seule ; frais à part au compte de frais, un centime d'arrondi possible ; même nuance à la fiche 39-11d.
  L2-P4 : commentaire du test `frais_apres_le_groupe_passe_seul` corrigé. **Tendance** P1 -> P4 : revue P1 à P3 avec
  remédiations, P4 sans rien au-dessus de LOW, boucle close. Modèles : rotation Sonnet/Opus, P4 ciblée. Décision M1-P3
  retenue : deux écritures sans double comptage (décision A-3). Gate : `cargo test -p kesh-api --test textes_coherents`, fmt.

### Notes de version

*Ajouté* — Les lignes d'un relevé Wise se rapprochent sur proposition (`GET /api/v1/wise/review`,
`POST /api/v1/wise/accept`) : frais au compte de frais, conversions au cours de Wise, rattachement à l'écriture déjà
passée de l'autre côté (virement, versement Stripe), le reste par vos règles ; un paiement par carte converti, pris sur
plusieurs soldes, devient une seule écriture au montant du marchand, son frais au compte de frais du solde (jamais à la
perte de change ; sans compte de frais, le paiement va « à examiner » et son acceptation est refusée) ; ce qui ne se
rapproche pas est listé avec son motif.
