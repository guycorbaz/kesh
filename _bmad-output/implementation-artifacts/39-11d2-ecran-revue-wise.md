# Story 39-11d2 : la revue et l'acceptation Wise à l'écran

Status: done

*Fiche rédigée le 2026-11-22, branche `story/39-11d2`, partie de `story/39-11d` (`440ae42c` à la rédaction ; la branche avance par sa revue P1, reprise par la condition préalable). Epic 39, phase 1
KISS. **Reprend l'AC-3 de la 39-11d**, reporté par son amendement A3 (la revue Wise, Story 39-11c, n'était pas dans la
branche). **Dépend de** : 39-11c (`done`, fusionnée dans `epic/39-wise` en `ab38871d`), 39-11d (page Wise sur la page
commune `ProviderReviewPage.svelte` de la 40-7d). C'est le seul écran où se rapprochent les transactions Wise importées :
sans lui, elles restent en attente, et le responsable de l'epic ne peut pas passer ses comptes 2025.*

## Story

En tant que **Comptable**, je veux, sur la page Wise, voir ce que Kesh propose pour chaque transaction importée — frais,
conversion, rattachement à l'autre côté, règle, paiement par carte groupé avec ses lignes —, comprendre en une phrase
pourquoi une ligne est « à examiner » ou en écart, et accepter les propositions choisies d'un geste, afin de tenir mes
comptes Wise sans passer par l'API.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-39-11d2-1` | La revue Wise montre les lignes d'un paiement par carte groupé (proposition ou « à examiner »). | AC-1, AC-2 |
| `FR-39-11d2-2` | Page Wise : exercice, propositions à cocher, leur genre et « ce qui sera fait » en clair. | AC-2 |
| `FR-39-11d2-3` | Chaque motif « à examiner » et chaque écart de la revue Wise s'explique à l'écran en une phrase, quatre locales. | AC-3 |
| `FR-39-11d2-4` | Acceptation par lot (*Pattern batch*, `{ accepted, failed }`), refus lisibles, revue relue. | AC-4 |
| `NFR-39-11d2-1` | Manuel utilisateur (`.tex` seul) et `docs/api-external.md` ; limites du rapprochement au manuel. | AC-5 |
| `NFR-39-11d2-2` | E2E de l'acceptation ; captures de chaque état touché (*Recette visuelle*). | AC-6 |

## Avant le code — base de la branche

**Condition préalable, hors de cette story** : `story/39-11d` (après sa revue) **fusionnée dans `epic/39-wise`**. Ce n'est
pas un conflit de fiche : c'est l'intégration de deux lignes de code — `story/39-11d` porte `epic/40-stripe` (page
commune, 40-7d), `epic/39-wise` porte la 39-14 et la 39-11c — qui ont modifié les mêmes fichiers depuis `1f03f180`
(dont `wise_imports.rs`, `provider_imports.rs`, `paypal_imports.rs`, `provider_accept.rs`,
`kesh-db/src/repositories/provider_review.rs`, `audit_route_registry.rs`, `lib.rs`, `wise_import_e2e.rs`,
`docs/api-external.md`, `user-manual.tex`). Elle se fait par l'orchestrateur, avec son gate complet (`kesh-db` touché),
sa revue du code d'intégration (diff combiné, interactions sans conflit de texte : la 39-14 a récrit `wise_imports.rs`,
la 39-11d l'a modifié) et ses compteurs recomptés (§ *Travailler à plusieurs développeurs*, `CLAUDE.md`). **Tant
qu'elle n'est pas faite, cette story attend.**

**Étape 0 de la story** (inconditionnelle) : `git merge epic/39-wise` dans `story/39-11d2` — `story/39-11d2` n'ajoute à
`story/39-11d` que des commits de fiche, la fusion n'apporte donc que l'intégration déjà jugée ; conflits éventuels
(fiche 39-11d, `sprint-status.yaml`) résolus en gardant les deux côtés. Puis `nextest` complet et frontend complet
**avant T1**, déclarés au Change Log. Relire au code, et citer au Change Log, la ligne de chaque nom repris ci-dessous
(`build_wise_review`, `card_group_blocker`, `reconciled_card_lines`, `remove_transactions`, `ReviewProposal`,
`PayoutProposal`/`StripeReviewProposal`, `findingDate`, `CODES_MOTEUR` de `provider-labels.test.ts`).

## Critères d'acceptation

1. **AC-1 — Les lignes d'un paiement groupé dans la revue** (seul changement serveur, **additif**, `kesh-api` seulement).
   La revue ne montre aujourd'hui d'un `card_group` que sa ligne principale (`transaction`) et des identifiants
   (`bankTransactionIds`, `details.bankTransactionIds`) : l'écran ne peut pas en montrer les lignes. Dans
   `routes/wise.rs`, sur le patron de la proposition `payout` de Stripe (type de **revue** sérialisé seul,
   `PayoutProposal`/`StripeReviewProposal`, `kesh-reconciliation/src/stripe.rs`), le type de revue vivant dans
   `routes/wise.rs` (la revue y est construite en `Review<WiseProposal>`, puis convertie par `map_proposals` à la fin ;
   `remove_transactions` garde son type) : la proposition `card_group` de la
   revue porte en plus `lines` (le `TransactionSummary` de chaque ligne, dans l'ordre de `bankTransactionIds`, principale
   en tête) et `merchantAmount` (montant du marchand en CHF, deux décimales, chaîne) ; les quatre motifs `card_group_*`
   portent `details.lines` (mêmes résumés) ; `card_group_statements_missing` porte en plus `details.missingCurrencies`
   (devises des soldes de `missingBankAccountIds`, que le serveur connaît : `BankAccountSummary` n'a pas de devise),
   posé **dans `card_group_blocker`** (son itérateur porte déjà `(id, devise)` ; paires triées ensemble, pour que
   `missingCurrencies[i]` reste la devise de `missingBankAccountIds[i]`), donc servi à la revue **et** dans
   `failed[].details` d'un `PROVIDER_PROPOSAL_STALE` de même raison. La conversion vers le type de revue se fait à la
   fin de `build_wise_review` (le `map_proposals` initial, `review.map_proposals(WiseProposal::Common)`, reste), d'après
   une table `id principale → (lignes, montant du marchand)` tenue pendant la boucle des groupes. Le **corps de l'acceptation ne change pas** (`WiseProposal`, champs
   d'affichage ignorés, déjà admis par `POST /wise/accept`) ; aucune migration ; `ReviewProposal` commune non touchée.
   Test HTTP ajouté à `wise_reconciliation_e2e.rs` (paiement groupé de `monter_partielle` : `lines` de trois résumés, la
   ligne CHF en tête, `merchantAmount` ; un `card_group_statements_missing` porte, à la revue,
   `details.lines` et `details.missingCurrencies` ; dans `failed[].details` d'une acceptation forgée,
   `missingCurrencies` sans `lines` — seuls les champs du blocker y passent) ; un corps
   d'acceptation qui reprend `lines` et `merchantAmount` est accepté.

2. **AC-2 — La page Wise avec sa revue** (`frontend/src/routes/(app)/reconciliation/wise/+page.svelte`, page commune
   **étendue, non recopiée**). `wise.api.ts` gagne `getWiseReview`, `acceptWiseProposals` (sur `getProviderReview`,
   `acceptProviderProposals`), passés à `api` : la page affiche l'exercice, les propositions, les écarts et « à
   examiner » (la page commune le fait dès que `getReview` est là). `wise.types.ts` : `WiseProposal` (`fee`,
   `conversion`, `link_entry`, `rule`, `card_group` avec `lines` et `merchantAmount`), `WiseAcceptProposal` (champs du
   corps seuls). `wise-review.ts`, sur le patron de `stripe-review.ts` : clé (`<type>-<transaction>`, la débitée d'une
   conversion, la principale d'un groupe), corps d'acceptation, `selectedInDateOrder`. Genre et « ce qui sera fait »
   (`wise-labels.ts`) :
   - `fee` : « Frais » — « Écriture au compte de frais <n° — nom> » ;
   - `conversion` : « Conversion » — « Au cours de Wise : 100.00 EUR → 109.00 USD » (montants de `transaction` et
     `otherTransaction`, en valeurs absolues), suivi de « une écriture » si un côté est en CHF, sinon « deux écritures par le compte de
     conversion » ;
   - `link_entry` : « Rattachement » — « Rattachée à l'écriture n° <n> déjà passée, sans nouvelle écriture » (`entry`) ;
   - `rule` : « Règle » — « Règle <libellé> → <n° — nom> » ;
   - `card_group` : « Paiement par carte » — « Une écriture : <merchantAmount> CHF au compte <n° — nom> (règle
     <libellé>), chaque solde crédité de sa part, la différence de change sur son compte » ; ses **lignes** affichées
     sous la proposition (date, montant et devise, identifiant Wise, « frais » par `lineLabel`) : la ligne de la
     proposition est la principale (`lines[0]`), les sous-lignes sont les suivantes (`lines[1..]`).
   Libellés des règles et des comptes lus au chargement (`onLoad`, best-effort) ; la lecture paginée des règles de la page
   PayPal (`allRules`) **sort dans `lib/features/provider/`** (non dans `rules.api.ts`, que `paypal-page.test.ts` et
   `rules-page.test.ts` simulent en entier) et sert aux deux pages (DRY). Page commune : quatre propriétés facultatives
   neuves — `proposalLines(p)` et `findingLines(f)` (rendent des
   `ProviderReviewTransaction[]`, la principale exclue), qui affichent les lignes en sous-lignes sous la proposition ou
   la ligne « à examiner » — chaque sous-ligne d'un « à examiner » avec son propre bouton « Rattacher à une écriture »
   (même modale, sur cette transaction ; la ligne du motif garde le sien, sur la principale). Identifiants de test des
   sous-lignes propres, jamais ceux d'une ligne (`<testId>-proposal-line-<id>`, `<testId>-to-review-line-<id>`,
   `<testId>-link-line-<id>`) ; `lineLabel(t)` (texte après une sous-ligne : « frais » pour une `FEE-CARD-…`) ;
   `toReviewExtra` (fragment sous le libellé d'un « à examiner », comme `gapExtra` pour un écart : liens « Voir
   l'écriture », lien vers la page Stripe) ; la page Wise lie `bankAccounts` (`bind:bankAccounts`) pour savoir si la
   société a un compte Stripe (`bankAccounts` est déjà `$bindable` : ce n'est pas une propriété neuve) ; `findingDate` lit aussi `details.date` (écart sans transaction ni écriture, `balance_gap`).
   Sans elles, PayPal et Stripe rendent ce qu'ils rendent (tests existants verts sans changement
   d'assertion, **sauf** le premier test de `provider-labels.test.ts`, qui fige `PROVIDER_COMMON_FAILED_CODES` et gagne
   `RATE_UNAVAILABLE`, AC-4 — le libellé neuf vaut aussi pour PayPal et Stripe). Rien n'est coché d'office ; « Tout cocher » ne coche que des propositions ; les propositions restent
   dans une seule table, triées par date, le genre en colonne. Le mode sans revue de la page commune (`hasReview`) reste,
   générique ; un test Vitest neuf, `lib/features/provider/ProviderReviewPage.test.ts`, le garde (page générique sans
   `getReview`, `listBankAccounts` et `listFiscalYears` simulés : ni exercice ni propositions). Le commentaire d'en-tête de la page Wise et celui de `ProviderReviewPage` (« jusqu'à la
   revue Wise ») sont récrits, et tout commentaire qui dit la revue Wise absente, trouvé en grepant le symptôme
   (`ProviderReviewPage.svelte`, « Wise, avant la 39-11c » ; `provider-review.types.ts` à relire) ; les tests qui affirment l'absence de revue sont **retournés**, non supprimés :
   `wise-page.test.ts` (ceux qui attendent ni exercice ni propositions) et `wise-import.spec.ts` (`wise-fiscal-year` à 0
   élément, en-tête « la page n'a que l'import »).

3. **AC-3 — Chaque motif en une phrase** (`wise-labels.ts`, clés `wise-` ; communs par `providerToReviewLabel`). Listes
   **fermées**, exportées (`WISE_GAP_CODES`, `WISE_TO_REVIEW_CODES`, `WISE_CARD_GROUP_UNSUPPORTED_REASONS`) et **comparées à
   la source Rust** par un test Vitest (constantes `GAP_*`, `REVIEW_*` de `kesh-reconciliation/src/wise.rs` et celles de la
   revue commune que Wise sert ; raisons de `judge_card_group`), sur le patron de `STRIPE_TO_REVIEW_CODES`. Un code
   inconnu s'affiche brut, jamais une ligne vide. Phrases (à reprendre telles quelles ou mieux dites, une phrase chacune) :
   - `fee_account_missing` : la phrase commune existante ;
   - `no_rule` : « Aucune règle ne reconnaît cette ligne : créez une règle d'affectation (filtre de type `CARD`,
     TRANSFER…), ou passez l'écriture à la main puis rattachez la ligne. » ;
   - `ambiguous_link` : « Plusieurs écritures déjà passées peuvent être l'autre côté de ce virement (n° …) : rattachez la
     ligne à la bonne. » (numéros tirés de `entries`) ;
   - `stripe_payout_unmatched` : « Versement Stripe sans l'écriture de son versement : importez le rapport Stripe et
     acceptez le versement, ou passez l'écriture à la main puis rattachez la ligne ; aucune règle ne s'y applique. »
     (lien vers la page Stripe seulement si la société a un compte Stripe) ;
   - `card_group_no_rule` : « Paiement par carte de { $amount } CHF : aucune règle ne lui donne un compte en CHF ;
     créez-en une. » ;
   - `card_group_unsupported` : « Les lignes de ce paiement par carte n'ont pas la forme attendue (<raison>) : traitez
     chaque ligne à la main. », la raison traduite par sa liste fermée (`not_a_debit`, `merchant_scale`,
     `exchange_missing`, `merchant_not_chf`, `exchange_currency_mismatch`, `exchange_amount_missing`,
     `card_line_missing`) ;
   - `card_group_statements_missing` : « Une ligne de ce paiement peut manquer : le relevé du solde <devises> ne couvre
     pas cette date ; importez-le, ou traitez les lignes à la main. » (devises de `details.missingCurrencies`, AC-1) ;
   - `card_group_partly_reconciled` : « Une part de ce paiement est déjà rapprochée : annulez ce rapprochement puis
     acceptez le paiement entier, ou finissez à la main. », suivi d'un lien « Voir l'écriture » par identifiant de
     `details.journalEntryIds` (des identifiants, non des numéros : aucun « n° » affiché) ;
   - écart `conversion_half_missing` : « L'autre moitié de cette conversion manque : importez le relevé de l'autre devise,
     ou elle est déjà rapprochée. » ;
   - écart `fee_without_origin` : « Frais sans la ligne dont il vient sur ce solde : importez le relevé complet de la
     période. » ;
   - écart `balance_gap` : la phrase de l'avertissement `gap` de l'import (`wiseWarningLabel`, **réutilisée**), avec la
     date et la devise de `details` (l'écart n'a pas de transaction : la colonne date lit `details.date`).
   Textes « proposer », jamais « comptabiliser automatiquement ». Clés aux quatre locales, chacune en toutes lettres ;
   clés neuves dans le bloc de l'Epic 39 des catalogues, préfixe `wise-` pour ce qui est propre à Wise, `provider-` pour
   le commun (`docs/i18n-key-ownership-pattern.md`, `lint-i18n-ownership`) ; `sitesTotal` de `i18n-keys.test.ts` et
   garde des libellés en dur **recomptés** depuis la source.

4. **AC-4 — Acceptation par lot.** « Accepter la sélection » envoie les corps dans l'ordre des dates, par paquets de 500
   (`chunkProposals`), compte figé ; réponse `{ accepted, failed }` (*Pattern batch*, HTTP 200) : compte rendu par la clé commune
   `provider-accept-report` (« n proposition(s) acceptée(s), m refusée(s). »), chaque refus par sa transaction et sa phrase (`wiseFailedLabel` : délégation à
   `providerFailedLabel`, puis `failedProposalLabel`) ; `PROVIDER_PROPOSAL_STALE` dont `details.reason` est
   `card_group_statements_missing` ou `card_group_partly_reconciled` dit la phrase du motif (AC-3) ; les autres raisons
   — liste fermée : `transaction_not_found`, `provider_settings_changed`, `transaction_not_pending`, `kind_changed`,
   `entry_changed`, `entry_linked`, `rule_no_longer_matches`, `conversion_pair_changed`, `card_group_changed` — la
   phrase commune « Rechargez la revue » (`withdrawal_foreign_currency`, inatteignable depuis Wise — `rejudge` refuse
   avant par `kind_changed` —, exclu par écrit). Un test Vitest **dédié** compare les littéraux `stale(…, "<raison>")`
   de `provider_accept.rs` (hors `#[cfg(test)]`) et de `routes/wise.rs` à cette liste **augmentée de l'exclusion**
   (dix raisons ; motif ancré sur `stale(<identifiant>, "<raison>")`, la définition `pub fn stale(…, reason: &str)`
   exclue, sans quoi il en compte onze ; le relevé de `CODES_MOTEUR` ne lit que les codes en majuscules) ; les deux raisons du blocker ne sont
   pas des littéraux de `stale` (`failed(primary, PROVIDER_PROPOSAL_STALE, …)`) et sont gardées par
   `WISE_TO_REVIEW_CODES` (AC-3). `RATE_UNAVAILABLE` reçoit un libellé **propre aux prestataires** dans `providerFailedLabel` (clé
   `provider-failed-rate-unavailable`, ajouté à `PROVIDER_COMMON_FAILED_CODES`) : « Aucun cours mensuel { $currency }
   retenu pour { $month } : récupérez ou saisissez ce cours dans les cours de change, puis acceptez de nouveau. » — le
   mois formaté depuis `details.month` (`AAAA-MM-01`) comme `monthLabel` de `lib/features/journal-entries/fx-refusals.ts`,
   exportée sous un nom propre (`fxMonthLabel` : `exchange-rates-grid.ts` exporte déjà un `monthLabel` d'une autre
   signature), jamais brut ; `details` absents ou vides tolérés (le premier test de `provider-labels.test.ts` appelle
   `providerFailedLabel(code)` sans eux) — ; le libellé générique (`fx-refusals.ts`, « gardez la ligne comme cours saisi ») ne convient pas à une proposition ;
   `FX_ACCOUNT_INVALID`, `COUNTERPARTY_CURRENCY_MISMATCH`, `PERIOD_LOCKED` par les libellés existants. Un
   `400` global (corps refusé) s'affiche par `errorMessageOf`. La revue se relit après l'acceptation. Vitest : corps d'un
   groupe (identifiants, règle, contrepartie ; sans `lines`), ordre, paquets, refus d'un groupe périmé.

5. **AC-5 — Manuel et API.** `docs/manual/fr/user-manual.tex` (`.tex` seul, PDF à la release ; DE, IT, EN « à traduire »
   selon la règle du dépôt), sous-section `sec:comptes-wise` : son paragraphe « Le rapprochement » (« n'est pas encore dans
   cette version ») **récrit**, ainsi que, hors de cette sous-section, la phrase des contrôles de clôture
   (« Transactions bancaires à rapprocher » : « la page Wise ne fait qu'importer », arrivée par la fusion préalable) ; ce que Kesh
   propose (frais, conversions au cours de Wise — une écriture si un côté est en francs, deux par le compte de conversion
   sinon —, rattachement à l'autre côté déjà passé, règles, paiement par carte converti en une écriture avec ses lignes) ;
   chaque motif « à examiner » et chaque écart en termes d'utilisateur ; le **cours mensuel retenu** de la société,
   condition de l'acceptation d'un frais ou d'une règle sur un solde en devise, d'une conversion entre deux devises
   étrangères et de la part en devise d'un paiement groupé (sinon le refus du cours mensuel manquant, AC-4, qui renvoie
   aux cours de change) ; l'acceptation par lot, rien d'office ; une ligne
   non proposée se traite à la main (écriture, puis *Rattacher à une écriture*) ; **les trois limites de la section
   « Limites du rapprochement à reprendre au manuel » de la fiche 39-11d** (continuité des relevés, solde en devise peu
   utilisé, part arrivée après coup), et l'annulation d'une ligne d'un paiement groupé qui remet **toutes** ses lignes en
   attente (amendement A-2 de la 39-11c). Gardes de `textes_coherents.rs` connues avant d'écrire (G2, G4-bis/G4-ter :
   aucun numéro de compte en dur, G18, G25). Contrôle du PDF local, aplati (`pdftotext … | tr '\n' ' ' | tr -s ' '`), non
   commité. `docs/api-external.md` : `lines` et `merchantAmount` du `card_group`, `details.lines` des motifs
   `card_group_*`, `details.missingCurrencies` de `card_group_statements_missing` (revue et `failed[].details`) ; la phrase « avec les mêmes `details` qu'à la revue » du refus
   `PROVIDER_PROPOSAL_STALE` (§ `POST /wise/accept`) remplacée par les champs réellement partagés
   (`missingBankAccountIds`, `missingCurrencies` ; `reconciledBankTransactionIds`, `journalEntryIds`) ; phrase « La revue et l'acceptation des transactions Wise ne sont pas encore servies » (§ *L'écran
   Wise*) remplacée par ce que l'écran appelle.

6. **AC-6 — E2E** `frontend/tests/e2e/wise-import.spec.ts` étendue (même société, même parcours, reseed en fin) : cours
   mensuels de 2020 posés par l'API (patron de `paypal-import.spec.ts`), compte de conversion et comptes de change de la
   société, règles par l'API (`counterparty_contains` filtrée `CARD`). Après l'import du zip : exercice 2020, revue à
   l'écran — une proposition de frais, une conversion, une règle ; « à examiner » `no_rule` et `stripe_payout_unmatched`
   avec leur phrase. Un **paiement par carte groupé** sur un **second compte Wise** (libellé distinct, soldes CHF et USD,
   ses deux comptes du plan CHF et USD créés par l'API, relevés CSV faits par la spec sur le patron de
   `releves_partielle` de `wise_reconciliation_e2e.rs` — en-tête repris de la première ligne d'une fixture lue par
   chemin —, identifiants et noms fictifs ; libellé qui trie **après** « Wise Fictif » pour que celui-ci reste le compte
   choisi à l'ouverture) : proposition `card_group`, sa ligne principale et ses deux sous-lignes affichées. Acceptation par
   l'écran, **une par compte Wise** (le corps n'en porte qu'un, et le compte rendu s'efface au changement de compte) :
   un frais et une conversion du premier → « 2 proposition(s) acceptée(s), 0 refusée(s). » ; le groupe du second →
   « 1 proposition(s) acceptée(s), 0 refusée(s). » ; propositions disparues à la relecture ; le groupe en **une**
   écriture (vérifiée par l'API, `GET /api/v1/reconciliation/transactions/{id}` de chaque ligne ou une
   relecture équivalente nommée au Dev Agent Record : trois transactions rapprochées sur la même écriture). Captures (thème clair,
   `KESH_CAPTURES_DIR`, hors dépôt) : revue, groupe et ses lignes, « à examiner », écarts (vides avec les fixtures :
   capture de l'état « Aucun écart. »), compte rendu de chaque acceptation ; jointes au Dev Agent Record et au prompt de revue.

## Tâches

- [x] T0 — Fusion d'`epic/39-wise`, conflits gardés des deux côtés, noms relus au code (*Avant le code*).
- [x] T1 — `lines`, `merchantAmount`, `details.lines`, `details.missingCurrencies` ; test HTTP (AC-1).
- [x] T2 — API, types, `wise-review.ts`, propriétés de la page commune, règles partagées (AC-2).
- [x] T3 — Libellés des genres, motifs, écarts et refus ; listes contre la source ; i18n (AC-3, AC-4).
- [x] T4 — Manuel, `docs/api-external.md` (AC-5).
- [x] T5 — E2E, captures (AC-6).
- [x] T6 — Signaler à l'orchestrateur l'issue à ouvrir (écran d'annulation d'une ligne groupée, *Hors périmètre*).

## Gates

Règle des stories de l'epic : `nextest` complet (changement de `kesh-api`), frontend complet (`check`,
`lint-i18n-ownership`, Vitest, `build`), E2E des specs `wise-import.spec.ts`, `paypal-import.spec.ts` et `stripe-import.spec.ts` au dernier
commit de code — au-delà de l'amendement A5 de la 39-11d (« la seule spec ajoutée »), parce que la page commune est
partagée ;
`gate-ci.sh` et E2E complet à la fin de l'epic. Le Dev Agent Record déclare exactement ce qui a tourné.

## Hors périmètre

- La vue d'annulation d'une transaction (`GET /reconciliation/transactions/{id}`, écran de la réconciliation) ne dit pas
  que les autres lignes d'un paiement groupé reviennent en attente (relevé par la 39-11c, « l'écran le dira avec
  39-11d ») : le manuel le dit (AC-5) ; l'écran d'annulation reste à faire, **à ouvrir en issue**.
- Choisir à l'écran une écriture parmi celles d'un `ambiguous_link` : la modale « Rattacher à une écriture » existante y
  suffit.

## Modules touchés (4)

`kesh-api` (`routes/wise.rs`, tests), `frontend` (page Wise, page commune, features `wise`, `provider`, règles, E2E),
`kesh-i18n` (libellés), `docs` (manuel `.tex`, `docs/api-external.md`).

## Note de version

*Ajouté* — Rapprochement Wise à l'écran : la page Wise propose, pour chaque transaction importée, les frais, les
conversions au cours de Wise, le rattachement à l'autre côté déjà passé, les règles et le paiement par carte converti
en une écriture, avec ses lignes ; chaque ligne « à examiner » ou en écart dit pourquoi ; les propositions choisies
s'acceptent d'un geste.

## Dev Agent Record

*Développée le 2026-11-22 (modèle Opus), worktree `kesh-wt39d2`, branche `story/39-11d2`. Journaux hors dépôt :
`kesh-gate-logs/s39-11d2/`.*

### Étape 0 — base et noms relus

- Fusion non squash d'`epic/39-wise` (`206d2a8b4`, 39-11d, 39-11c, 39-14, Epic 40 à jour) : `8d82118d6`. Un seul
  conflit, `sprint-status.yaml` (39-11c et 39-11d `done` d'`epic/39-wise`, 39-11d2 passée `in-progress`) ; aucun
  compteur touché par la fusion.
- Gates avant tout code, au commit de fusion (`cible-0.log`) : `cargo fmt --check` 0, clippy workspace
  `-D warnings` 0, nextest ciblé Wise (`binary(wise_import_e2e) | binary(wise_reconciliation_e2e) | test(/wise/)`)
  **64 passés**. Le frontend complet n'a pas été joué au commit de fusion (consigne de l'orchestrateur : fmt, clippy,
  nextest ciblé) ; il l'a été à la fin (ci-dessous).
- Noms relus au code, au commit de fusion : `build_wise_review` (`routes/wise.rs:136`), `reconciled_card_lines`
  (`:331`), `card_group_blocker` (`:393`, son itérateur porte `(id, devise)`), `remove_transactions` (`:445`, garde
  `Review<WiseProposal>`), `ReviewProposal` (`kesh-reconciliation/src/provider_review.rs:439`, non touchée),
  `PayoutProposal`/`StripeReviewProposal` (`kesh-reconciliation/src/stripe.rs:312`, `:328`), `findingDate`
  (`frontend/src/lib/features/provider/provider-review.ts:38`), `CODES_MOTEUR` (`provider-labels.test.ts:23`).
- Les trois LOW de la validation P4, recontrôlés : `fxMonthLabel` (exporté sous ce nom de `fx-refusals.ts`,
  l'homonyme de `exchange-rates-grid.ts` intact) ; `details` absents tolérés (test « détails absents ou vides » de
  `provider-labels.test.ts`) ; motif des raisons ancré sur `stale(<identifiant>, "<raison>")`, définition exclue
  (dix raisons relevées) ; paires `(id, devise)` triées ensemble dans `card_group_blocker`.
- Limites de la fiche 39-11d (« à reprendre par la 39-11d2 ») : les trois, au manuel (§ *Limites* de
  `sec:comptes-wise`), avec l'annulation d'une ligne groupée.

### Revue d'intégration de l'Epic 39 (consigne de l'orchestrateur)

- **M1** : manuel, paragraphe « Le rapprochement » récrit ; § Stripe « Versement sans réception » (compte Wise : la
  page Wise propose le rattachement) ; `api-external.md` au même mot (§ *L'écran Wise*, réception d'un versement
  Stripe sur Wise au § Stripe).
- **L1** : `routes/wise.rs`, `DESCRIPTION_MAX_CHARS = ENTRY_DESCRIPTION_MAX_CHARS` (une seule définition).
- **L2** : en-têtes récrits — page Wise, `wise-import.spec.ts`, `stripe_reconciliation_e2e.rs`.
- M2 non touché (traité sur `epic/40-stripe`).

### Ce qui a été fait

- **AC-1** (`crates/kesh-api/src/routes/wise.rs`) : type de revue `WiseReviewProposal` (`CardGroup(CardGroupReview)` |
  `Common(Proposal)`, sérialisé à plat), `CardGroupReview { proposal (aplati), lines, merchant_amount }` ; table
  `shown` (principale → lignes, montant) tenue dans la boucle des groupes, conversion finale `to_review_proposal`
  (le `map_proposals(WiseProposal::Common)` initial reste) ; `details.lines` sur les quatre motifs `card_group_*` ;
  `missingCurrencies` posé dans `card_group_blocker`, donc aussi à l'acceptation. Corps de l'acceptation inchangé.
- **AC-2** : `wise.api.ts` (`getWiseReview`, `acceptWiseProposals`), `wise.types.ts` (`WiseProposal`,
  `WiseAcceptProposal`), `wise-review.ts` (clé, transaction, corps, ordre, sous-lignes) ; page commune : propriétés
  `proposalLines`, `findingLines`, `lineLabel`, `toReviewExtra` et, ajoutée au développement, `toReviewLineCells`
  (les colonnes propres d'une sous-ligne « à examiner ») ; `findingDate` lit `details.date` ; `openLink` prend une
  transaction. La page Wise montre aussi, dans « À examiner », le **nom** et l'**identifiant Wise** de chaque ligne
  et sous-ligne (sans eux, les captures montraient des lignes qu'on ne pouvait pas retrouver). Lecture paginée des
  règles sortie dans `provider/provider-rules.ts` (`listAllRules`, `ruleLabelsById`), reprise par la page PayPal ;
  `accountNumberName` (provider-labels) remplace les deux copies privées de PayPal et Stripe (DRY). Sous-titre de la
  page Wise : « Puis acceptez les propositions de rapprochement : rien n'est écrit dans les livres sans votre
  acceptation » (quatre locales).
- **AC-3, AC-4** (`wise-labels.ts`) : listes fermées `WISE_GAP_CODES`, `WISE_TO_REVIEW_CODES`,
  `WISE_CARD_GROUP_UNSUPPORTED_REASONS`, `WISE_STALE_RELOAD_REASONS` (+ `WISE_STALE_EXCLUDED_REASONS`), comparées à la
  source Rust ; une phrase par motif et par écart ; `wiseFailedLabel` ; `RATE_UNAVAILABLE` propre aux prestataires
  (`provider-failed-rate-unavailable`, mois par `fxMonthLabel`), `PROVIDER_COMMON_FAILED_CODES` augmentée (seul test
  existant dont l'assertion change, comme le prévoit l'AC-2).
- **AC-5** : manuel utilisateur (`.tex` seul) et `docs/api-external.md`. PDF construit dans une copie hors dépôt
  (`pdf.log` : 3 passes, stable), aplati et grepé : les phrases neuves s'y trouvent, « pas encore dans cette version »
  n'y est plus. DE, IT, EN : manuels non traduits (règle du dépôt).
- **AC-6** : `wise-import.spec.ts`, tests « 5 » et « 6 » (numérotés à la suite des points 1 à 4 de l'en-tête ; il n'y
  a pas de test « 4 »), relecture des trois transactions du groupe par `GET /api/v1/reconciliation/transactions/{id}`
  (`status` `reconciled`, même `matchedEntryId` non nul) ; tests « sans revue » retournés (`wise-page.test.ts` : deux ;
  `wise-import.spec.ts` : `wise-fiscal-year` à un élément) ; le test L7 de la page commune sans revue déplacé dans
  `ProviderReviewPage.test.ts`.

### Tests ajoutés (recomptés, de `8d82118d6` à `HEAD`)

- Rust : `wise_reconciliation_e2e.rs` **23 → 25** (`revue_du_groupe_avec_ses_lignes`,
  `lignes_et_devises_d_un_groupe_a_examiner`).
- Vitest, `it(` : `wise-review.test.ts` 0 → 6, `wise-labels.test.ts` 9 → 23, `provider-labels.test.ts` 3 → 6,
  `ProviderReviewPage.test.ts` 0 → 2, `provider-review.test.ts` 0 → 1, `wise-page.test.ts` 8 → 12 (deux retournés).
- E2E : `wise-import.spec.ts` 3 → 5 tests.
- Revues P1 et P2 (de `6ce8fd5cc` à `HEAD`) : tests unitaires `routes::wise::tests` 0 → 3
  (`groupe_sans_lignes_d_affichage_est_une_erreur_interne`, `groupe_connu_rend_ses_lignes`,
  `groupe_absent_de_shown_rend_l_erreur_interne`) ; Vitest `wise-page.test.ts` 12 → 13.
- Compteurs recomptés par leur garde : `sitesTotal` 3782 → **3816** (34 sites `i18nMsg(` au diff hors tests, 0
  retiré) ; garde des libellés en dur 149 → **159** (`conforme` 128 → 137, `ecartee` 21 → 22, chaque candidate nommée).

### Sondes négatives (mutations hors dépôt, une à la fois, tuées seulement sur assertion nommée)

- Frontend (copie avec son vrai `node_modules`, témoin vert 114/114, `mut-temoin.log`) : F1 corps d'un groupe avec
  `lines` → `wise-review.test.ts` « le corps d'un groupe… » ; F2 `RATE_UNAVAILABLE` hors de `providerFailedLabel` →
  trois tests de `provider-labels.test.ts` ; F3 blocage d'un groupe dit « Rechargez » → `wise-labels.test.ts` « un
  groupe périmé par son blocage… » ; F4 sous-lignes non rendues → `wise-page.test.ts` « genres et ce qui sera fait… »
  (élément `wise-proposal-line-11` introuvable) ; F5 `findingDate` sans `details.date` → `provider-review.test.ts`.
  Journaux `mut-F*.log`.
- Rust (copie `git archive` de `40cc2b538`, cible cargo propre, supprimée ensuite ; témoin 25/25) : R1
  `missingCurrencies` retiré → `lignes_et_devises_d_un_groupe_a_examiner` panique (`:1202`) ; R2 groupe sans entrée
  dans `shown` → `revue_du_groupe_avec_ses_lignes` panique (`:1154`). Journaux `mutrs-*.log`.

### Gates (dernier commit de code `6ce8fd5cc`)

- `cargo fmt --check` 0 et clippy workspace `-D warnings` 0 (`cible-2.log`, à `40cc2b538` ; le seul changement Rust
  postérieur est une clé ajoutée aux catalogues `.ftl`, couverte par le nextest ci-dessous) ; nextest ciblé Wise 66
  passés (`cible-2.log`).
- **nextest complet** à `6ce8fd5cc` (`nextest-final2.log`) : **6660 exécutés, 6660 passés, 17 ignorés**.
- **Frontend complet** (`front-final3.*`, arbre de `6ce8fd5cc` avant son commit, même contenu) : `npm run check` 0
  (0 erreur), `lint-i18n-ownership` 0, Vitest **5152 passés (303 fichiers)**, `npm run build` 0.
- **E2E partiel** à `6ce8fd5cc` (`e2e-final2.log`, `e2e.head`) : `wise-import.spec.ts` (spec étendue),
  `paypal-import.spec.ts` et `stripe-import.spec.ts` (page commune partagée) — **16 passés** (5 + 7 + 4), 0 échec.
  Captures dans `kesh-gate-logs/s39-11d2/captures/` : `10-revue-premier-compte`, `11-a-examiner`,
  `12-ecarts-aucun`, `13-compte-rendu-premier-compte`, `14-paiement-groupe-et-ses-lignes`,
  `15-compte-rendu-paiement-groupe`, `16-page-wise-apres-acceptation` (00 à 07 : la partie import, rejouée ; les
  autres, PayPal et Stripe). `11-a-examiner` est prise sur l'élément : l'en-tête fixe de l'application recouvre sa
  première ligne ; la capture pleine page `10` la montre entière.
- Non joués (règle de la nuit) : `gate-ci.sh` et E2E complet, à la fin de l'epic.

### Hors périmètre, à ouvrir en issue (T6, signalé à l'orchestrateur)

- L'écran d'annulation d'une transaction (`GET /reconciliation/transactions/{id}`) ne dit pas que les autres lignes
  d'un paiement groupé reviennent en attente (le manuel le dit).
- Revue d'intégration L3 : aucun test de bout en bout ne joue la chaîne `POST /stripe/accept` (versement vers un
  compte Wise) → `GET /wise/review` propose `link_entry` → `POST /wise/accept` ; non ajouté ici.

### Fichiers

`crates/kesh-api/src/routes/wise.rs`, `crates/kesh-api/tests/wise_reconciliation_e2e.rs`,
`crates/kesh-api/tests/stripe_reconciliation_e2e.rs` (en-tête), `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`,
`docs/manual/fr/user-manual.tex`, `docs/api-external.md`, `frontend/src/lib/features/provider/{ProviderReviewPage.svelte,
ProviderReviewPage.test.ts, provider-labels.ts, provider-labels.test.ts, provider-review.ts, provider-review.test.ts,
provider-rules.ts}`, `frontend/src/lib/features/wise/{wise.api.ts, wise.types.ts, wise-labels.ts, wise-labels.test.ts,
wise-review.ts, wise-review.test.ts}`, `frontend/src/lib/features/{paypal/paypal-review.ts, stripe/stripe-labels.ts,
journal-entries/fx-refusals.ts}`, `frontend/src/routes/(app)/reconciliation/{wise/+page.svelte, wise/wise-page.test.ts,
paypal/+page.svelte}`, `frontend/src/lib/shared/{i18n-keys.test.ts, i18n-libelle-en-dur.test.ts}`,
`frontend/tests/e2e/wise-import.spec.ts`, `sprint-status.yaml`, cette fiche.

## Change Log

- 2026-11-22 — fiche rédigée (AC-3 de la 39-11d, amendement A3), non validée.
- 2026-11-22 — validation P1 (Opus, contexte frais, lecture seule) : 0 CRITICAL, 1 HIGH, 7 MEDIUM, 9 LOW, tous remédiés
  dans la fiche (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d2-P1.md`). HIGH : la base n'était pas un
  conflit de fiche mais l'intégration de `story/39-11d` (Epic 40 compris) et d'`epic/39-wise` (39-14, 39-11c) — posée en
  condition préalable, hors story, avec gate et revue d'intégration ; étape 0 inconditionnelle, gates avant T1. MEDIUM :
  acceptation E2E une par compte Wise (M1) ; phrase du manuel absente sans la fusion (M2) ; tests « sans revue » nommés,
  retournés (M3) ; `details.missingCurrencies` servi (M4) ; écritures d'une part rapprochée par identifiant, sans numéro
  (M5) ; sous-lignes sans la principale, identifiants de test propres (M6) ; cours mensuel au manuel et au refus (M7).
  LOW : `findingDate` et `details.date`, raisons de `PROVIDER_PROPOSAL_STALE` fermées, montants absolus, place du type
  de revue, `CODES_MOTEUR`, versement Stripe sans compte Stripe, T6, une table triée par date, état « Aucun écart ».
- 2026-11-22 — validation P2 (Sonnet, contexte frais, lecture seule) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 11 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d2-P2.md`). MEDIUM : `missingCurrencies` posé dans
  `card_group_blocker`, donc aussi à l'acceptation (M1) ; libellé propre de `RATE_UNAVAILABLE` (M2) ; propriétés
  `lineLabel` et `toReviewExtra`, `bind:bankAccounts` (M3) ; `api-external.md` suit l'AC-1 (M4) ; test dédié des raisons
  de `PROVIDER_PROPOSAL_STALE` (M5). LOW : gabarits Fluent, `allRules` dans `provider/`, E2E PayPal et Stripe au gate,
  test du mode sans revue, en-tête, clé du compte rendu, conversion finale, second compte de l'E2E, bloc i18n de l'epic.
  L9 (préfixe des identifiants de sous-ligne) laissé : `getByTestId` est exact. L11 (densité) : aucun seuil franchi.
- 2026-11-22 — validation P3 (Opus, contexte frais, lecture seule) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 8 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d2-P3.md`). MEDIUM : le test HTTP exigeait `details.lines`
  dans `failed[].details`, que seul le blocker alimente — `missingCurrencies` seul à l'acceptation, et phrase « les
  mêmes `details` qu'à la revue » d'`api-external.md` corrigée. LOW : nom doublé, `bankAccounts` déjà liable, mois du
  cours formaté, test de la liste commune qui change d'assertion, `stripe-import.spec.ts` nommée, place de la phrase
  des contrôles de clôture, commentaires périmés à greper, `ProviderReviewPage.test.ts`, liste des raisons augmentée
  de l'exclusion.
- 2026-11-22 — validation P4 ciblée sur la remédiation P3 (`0326e98c`, Sonnet, lecture seule) : 0 CRITICAL, 0 HIGH,
  0 MEDIUM, 3 LOW (rapport `kesh-gate-logs/epic-39-40-kiss/validation/39-11d2-P4.md`), remédiés dans la fiche seule :
  nom `fxMonthLabel` (homonyme), `details` absents tolérés, motif du test des raisons ancré, paires `(id, devise)` triées
  ensemble. **Boucle close** sur une passe sans rien au-dessus de LOW. Tendance : P1 1 HIGH, 7 MEDIUM, 9 LOW → P2 5 MEDIUM,
  11 LOW → P3 1 MEDIUM, 8 LOW → P4 0 au-dessus de LOW (3 LOW). Modèles : Opus (P1, P3), Sonnet (P2, P4 ciblée). Les LOW
  de la P4 ne sont relus par aucune passe : à recontrôler à l'étape 0 du développement.
- 2026-11-22 — développée (étape 0, T0 à T5) ; gates au Dev Agent Record ; passée en `review`.
- 2026-11-22 — revue de code P1 (Sonnet, contexte frais, lecture seule ; rapport `kesh-gate-logs/s39-11d2/revue-p1-rapport.md`) :
  0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW, remédiés (commit de remédiation P1). MEDIUM-1 : le bouton « Rattacher » d'une sous-ligne
  « à examiner » n'était testé nulle part sur SA transaction (mutation `openLink(line)` → `openLink(tx)` survivante) : test
  Vitest `wise-page.test.ts` qui clique `wise-link-line-31` et exige `getEntryCandidates(31)`, jamais 30 ; mutation rejouée sur
  une copie hors dépôt (`s39-11d2/mut-openlink-src`), témoin vert 13/13, mutation tuée sur l'assertion nommée « expected last
  `vi.fn()` call to have been called with [ 31 ] » (journal `p1fix-mut-openLink.log`). LOW-1 : `docs/api-external.md` § *L'écran Wise*
  dit que les corps d'acceptation ne portent que les champs du corps. LOW-2 : `to_review_proposal` ne replie plus en silence
  (`unwrap_or_default`) : `card_group_display` rend une erreur interne tracée (`tracing::error!`, `AppError::Internal`),
  propagée par `build_wise_review` (tests `groupe_sans_lignes_d_affichage_est_une_erreur_interne`, `groupe_connu_rend_ses_lignes`).
  LOW-3 : ligne vide finale de `provider-review.ts` retirée. Passe P2 ciblée sur ce commit : prompt `revue-p2-prompt.md`.
- 2026-11-22 — revue de code P2 ciblée sur la remédiation P1 (`5c172533b`, Opus, contexte frais, lecture seule ; prompt
  `revue-p2-prompt.md`, rapport `kesh-gate-logs/s39-11d2/revue-p2-rapport.md`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 3 LOW. MEDIUM-1
  et LOW-1 à LOW-3 de la P1 confirmés corrigés (mutations `openLink` et repli vide rejouées, tuées). LOW-A appliqué (test
  seul, `6c5c40f68`) : `groupe_absent_de_shown_rend_l_erreur_interne` exige `(_, Some(AppError::Internal(_)))` de
  `to_review_proposal` sur `shown` vide, et aucune erreur pour une proposition commune ; mutation « erreur ignorée »
  (`Err(e) => (…, Some(e))` → `Err(_e) => (…, None)`) sur une copie `git archive` hors dépôt, cible cargo propre supprimée
  ensuite : témoin 3/3 vert, mutant tué sur l'assertion nommée « erreur interne attendue, obtenu None » (`wise.rs:1010`),
  rc 101 (journal `kesh-gate-logs/s39-11d2/lowa-mut.log` ; ciblé `routes::wise::tests` 3 passés et `cargo fmt --check` 0,
  `lowa-cible.log`). LOW-B noté, non appliqué : le tuple `(proposition, Option<AppError>)` vient de `map_proposals`, qui ne
  sait pas échouer ; un pré-contrôle de `shown` ou un `try_map_proposals` serait plus simple, et le `tracing::error!` de
  `card_group_display` double celui d'`AppError::Internal` (il ajoute les identifiants) — forme, sans comportement faux.
  LOW-C hors périmètre (phrase sœur de Stripe, `docs/api-external.md`) : reprise par la story de solde de l'Epic 40.
  **Boucle close** sur une passe sans rien au-dessus de LOW. Tendance : P1 (Sonnet) 0/0/1/3 → P2 ciblée (Opus) 0/0/0/3.
  Clippy et nextest complet de ce dernier commit (test seul) au gate de l'intégration dans `epic/39-wise`.
