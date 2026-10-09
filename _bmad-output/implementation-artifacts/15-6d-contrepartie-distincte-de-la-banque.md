# Story 15.6d : Un rapprochement ne prend pas le compte de la banque pour contrepartie

## Status

review

<!-- Créée le 2026-10-08 à la validation P1 de la 15-6b (finding F1 = R3, #524 ; finding F4 :
     découpage, plus de cinq modules), en autonomie. Choix propres : C-15-6-9 (qui révise C-15-6-6),
     C-15-6-23, C-15-6-28 (qui révise C-15-6-23 sur deux points, C-15-6-31). Validation P1 (Opus, lentilles F et R) appliquée le 2026-10-08 : égalité testée avant
     le 404 et la postabilité, règle sur le compte de banque plus proposée, câblage de l'écran testé.
     Validation P2 (Opus, lentilles R et F) appliquée le 2026-10-08 : contrôle du compte de banque actif
     ajouté au chemin par règle (même réponse sur les quatre chemins), gardes ventilées figées par deux
     témoins, angle mort de course retiré, garde de génération retirée (C-15-6-28). Validation P3
     (Sonnet, lentilles R et F) le 2026-10-08 : 10 LOW appliqués, validation close (C-15-6-31). -->

**Issue** : `closes #524` (P1) — mot-clé **dans la PR** (merge en squash). **Mère** :
`15-6-creance-juste-avoir-reglement.md`.
**Après la 15-5b** (rebase : elle modifie les mêmes fonctions — `ACCOUNT_NOT_POSTABLE` sur
`post_manual`, son AC1, et sur `accept_one_rule`, son AC4 ; l'ensemble passé à `first_matching_rule`
dans `get_proposals`, son AC5). **Aucune dépendance d'ordre des refus** : le refus de cette story
précède le sien (AC1, AC2, choix C-15-6-23). Son passage du manuel s'écrit sur la section du
rapprochement que la **15-5c** réécrit : après la 15-5c de préférence, sinon rebase attendu. La
**15-5c** modifie aussi les deux fichiers d'écran de l'AC4 — `ReconciliationProposals.svelte` (son
libellé de `failed[]`) et `ReconciliationProposals.test.ts` (trois tests ajoutés, sa T3) — : rebase de
forme attendu sur eux. Sa traduction des codes de `failed[]` porte l'AC6.
**Après la 15-6b** pour un seul emprunt : la **comparaison** de deux comptes de son helper commun
(`invoice_settlements`, `pub fn ensure_not_claim_account(account_id: i64, claim_account_id: i64) -> Result<(), ClaimAccountClash>`,
fonction **pure**, sans connexion ni `async`, telle que la fiche 15-6b la pose — T0 reprend le nom réel), choix
C-15-6-28 ; **pas** sa construction de refus (AC3). Autres fonctions de `reconciliation.rs` que les
15-6a/b/c ; filtre d'écran local, sans emprunt à `account-options.ts`.

## Story

En tant que **comptable qui rapproche ses relevés**,
je veux que **Kesh refuse un rapprochement manuel ou par règle dont la contrepartie est le compte même
de la banque**,
afin qu'**une transaction bancaire ne soit jamais marquée rapprochée par une écriture `D banque / C
banque`, nulle au grand livre**.

## Le défaut, établi au code

- Le rapprochement **ventilé** refuse déjà une contrepartie égale au compte comptable du compte
  bancaire, sur ses deux chemins :
  - `accept_one_split` (`crates/kesh-api/src/routes/reconciliation.rs:1829`, garde `:1941-1950`) :
    `FailedProposal { error_code: "VALIDATION_ERROR", details: { "reason": "counterparty_equals_bank_ledger" } }` ;
  - `post_split` (`:3362`, garde `:3462-3475`) : `AppError::Validation("splits[{idx}].counterpartyAccountId ne peut pas être le compte ledger banque")`,
    **400**, message en français en dur.
- Le rapprochement **manuel** (`post_manual`, `:2942` ; compte de banque `:2989`, contrepartie
  `:3028`) et l'acceptation **par règle** (`accept_one_rule`, `:2207` ; compte de banque `:2235`,
  contrepartie `:2287`) **ne comparent jamais** les deux comptes ;
  `build_journal_entry_for_counterparty` (`crates/kesh-reconciliation/src/manual.rs:67`) non plus.
- Une règle n'est attachée à aucun compte bancaire (l'entité `reconciliation_rule` ne porte pas de
  `bank_account_id`), et sa création par l'API ne restreint pas le type du compte : seule
  l'acceptation connaît le compte de banque en jeu.
- **Contrôle du compte de banque actif** : `post_manual` (étape 2 bis, `:3004-3017`, 412),
  `accept_one_split` (« Step c », `:1912-1938`, `failed[]` `BANK_ACCOUNT_NOT_CONFIGURED`) et
  `post_split` (« Step 4bis », `:3446-3460`, 412) le font **avant** leur garde d'égalité ;
  `accept_one_rule` **ne le fait pas** (étapes 1, 2, 4, 5, 6 — `:2217-2318`) : un compte de banque
  archivé y atteint l'écriture, dont la garde `active` (`journal_entries.rs:98`) refuse.
- **Écrans** : `ManualMatchModal.svelte:65-69` ne propose que les classes 5/6/7. Le serveur impose le
  **type** du compte de banque — actif ou passif (`validate_journal_account_id`,
  `crates/kesh-api/src/routes/bank_accounts.rs:260-294`) — mais **pas sa classe** : aucun des trois
  plans livrés (`crates/kesh-core/assets/charts/*.json`) n'a d'actif ou de passif en classe 5 à 7, et
  le commentaire de `post_split` (`:3465-3467`) le dit (« en UX normal pas de collision possible ») ;
  la collision suppose un compte d'actif ou de passif créé par l'utilisateur en classe 5 à 7.
  `RuleFormModal.svelte:44-50` ne propose **en création** que charges et produits ; en **édition**, le
  compte en place de la règle est réintroduit (`withCurrentAccount`, `:55-57`,
  `frontend/src/lib/features/accounts/account-options.ts:58`, #271), même s'il n'est pas éligible.

## Acceptance Criteria

1. **AC1 — Rapprochement manuel** (`post_manual`) : **juste après** l'étape 2 bis (compte de banque
   résolu et actif, `:2989-3017`), **avant** l'étape 3 (contrepartie : 404 `ACCOUNT_NOT_FOUND`, puis
   `ACCOUNT_NOT_POSTABLE` de la 15-5b) : `body.counterparty_account_id == bank_ledger_account_id` →
   **le même refus que `post_split`** (choix C-15-6-9) :
   `AppError::Validation(…)`, 400 `VALIDATION_ERROR` — même **code** ; le **texte** suit le champ en
   cause : `post_split` écrit aujourd'hui `splits[{idx}].counterpartyAccountId ne peut pas être le
   compte ledger banque` (`reconciliation.rs:3471`), `post_manual` écrira `counterpartyAccountId ne
   peut pas être le compte ledger banque`, par la fonction unique de l'AC3, qui prend le chemin du champ
   en paramètre (finding F3-2 de la P3). **Même place que le flux ventilé** (choix C-15-6-23) : `accept_one_split` et
   `post_split` comparent **après** le contrôle d'activité du compte de banque (« Step c »
   `:1912-1938`, « Step 4bis » `:3446-3460`) et **avant** la validation des comptes de contrepartie
   (`:1941-1950` puis « Step d » ; `:3462-3475` puis « Step 5 batch validation ») — si bien qu'un
   compte de banque devenu non imputable pris pour contrepartie rend partout **ce** refus, et non
   `ACCOUNT_NOT_POSTABLE` (qui inviterait à agir sur la mauvaise chose). Le 404 ne pourrait de toute façon pas précéder utilement : égale au
   compte de banque, la contrepartie existe et est active (étape 2 bis). **Avant l'étape 4**
   (`find_strictly_pending…`) aussi : une contrepartie égale au compte de banque sur une transaction
   **déjà rapprochée** rend donc **400 `VALIDATION_ERROR`**, pas 404
   `RECONCILIATION_TRANSACTION_NOT_PENDING` — choix écrit (le défaut de la requête prime l'état de la
   transaction), testé (test 3). Rien n'est écrit : aucune écriture, transaction bancaire toujours
   `pending`, aucun audit. **Ordre** écrit dans le doc-comment : forme (étape 0) → 404 compte bancaire
   / 412 `BANK_ACCOUNT_NOT_CONFIGURED` (étapes 1 à 2 bis, existantes) → **400 `VALIDATION_ERROR`** →
   404 `ACCOUNT_NOT_FOUND` → 400 `ACCOUNT_NOT_POSTABLE` (15-5b) → 404 transaction non en attente → la
   suite inchangée.
2. **AC2 — Acceptation par règle** (`accept_one_rule`).
   - **Contrôle du compte de banque actif, ajouté** (choix C-15-6-28) : juste après l'étape 1, la
     même étape que la « Step c » d'`accept_one_split` (`:1912-1938`) — `SELECT active FROM accounts
     WHERE id = ? AND company_id = ?` dans la transaction ; erreur SQL → `DATABASE_ERROR`
     (`details.message`) ; compte absent ou archivé → `FailedProposal { error_code:
     "BANK_ACCOUNT_NOT_CONFIGURED", details: { "bankAccountId" } }`, forme de l'étape 1 et
     d'`accept_one_split`. Sans lui, un même défaut — compte de banque archivé — rendrait sur ce seul
     chemin `VALIDATION_ERROR` (règle sur ce compte) ou le refus de la garde `active` de l'écriture
     (toute autre règle), quand les trois autres rendent `BANK_ACCOUNT_NOT_CONFIGURED`. **Changement
     délibéré** d'un refus existant : le code rendu aujourd'hui pour une règle ordinaire sur un compte
     de banque archivé est relevé en T0 et écrit au Change Log ; aucun test existant ne l'exerce
     (`grep -n "BANK_ACCOUNT_NOT_CONFIGURED" crates/kesh-api/tests/reconciliation_rules_e2e.rs` → `:1360`
     seul, compte bancaire **non lié**, étape 1, inchangé). **Atteignable depuis l'écran** (finding R1 de
     la P3) : `get_proposals` ne contrôle que l'archivage du **compte bancaire** (`ba_check.archived`,
     `reconciliation.rs:476`), jamais l'activité du compte comptable lié, et l'archivage d'un compte du
     plan n'est pas refusé quand un compte bancaire y est lié ; un compte de banque archivé **avant**
     l'ouverture de l'écran laisse donc proposer une règle ordinaire, dont l'acceptation tombe dans
     cette étape — ce que l'utilisateur d'écran voit (libellé de la 15-5c, AC6). Aucun contrôle n'est
     ajouté à `get_proposals` (périmètre) : angle mort écrit (§ *Angles morts*).
   - **Garde d'égalité** : **après** l'étape 4 (concordance de la
   contrepartie de la proposition avec celle de la règle, `RECONCILIATION_RULE_MISMATCH`) et **avant**
   l'étape 5 (contrepartie active, puis `ACCOUNT_NOT_POSTABLE` de la 15-5b, son AC4) : égalité de la
   contrepartie de la règle avec le compte de banque (lu à l'étape 1, dans la transaction) →
   `FailedProposal { error_code: "VALIDATION_ERROR", details: { "reason": "counterparty_equals_bank_ledger" } }`
   — exactement la forme de `accept_one_split`. HTTP 200, la proposition dans `failed[]`, les autres
   du lot traitées normalement (§ *Pattern batch* du `CLAUDE.md`) ; la transaction bancaire reste en
   attente, aucune écriture, aucun audit pour elle. **Ordre** : étape 1 (compte de banque :
   `DATABASE_ERROR`, `BANK_ACCOUNT_NOT_CONFIGURED`, `BANK_ACCOUNT_NOT_FOUND`) → **compte de banque
   actif** (`DATABASE_ERROR`, `BANK_ACCOUNT_NOT_CONFIGURED`) → étape 2
   (`RECONCILIATION_RULE_NOT_FOUND`) → étape 4 (`RECONCILIATION_RULE_MISMATCH`) → **`VALIDATION_ERROR`**
   → étape 5 (`ACCOUNT_NOT_FOUND`, `ACCOUNT_NOT_POSTABLE`) → étape 6. Écrit dans le doc-comment.
   **AC2 bis — Une règle sur le compte de banque n'est plus proposée.** `get_proposals` lit déjà le
   compte bancaire (`ba_check`, `reconciliation.rs:469-478`, qui porte `journal_account_id`) et passe
   à `first_matching_rule` (`:634-638` ; `kesh-reconciliation/src/rules.rs:85-95`, qui rend la
   **première** règle concordante) l'ensemble `active_account_ids` (`:556-557`). Le compte de banque en
   est **retiré** (s'il est configuré) : une règle dont la contrepartie est ce compte n'est plus
   proposée — l'acceptation la refuserait toujours — et **ne masque plus** une règle suivante valable
   pour la même transaction. Patron de l'AC5 de la 15-5b, qui retire du même ensemble les comptes non
   imputables : les deux retraits se composent, à écrire sur l'état rebasé. Le commentaire que la 15-5b
   pose au site d'appel pour dire ce que contient l'ensemble est **mis à jour** : « actifs, imputables,
   hors compte de la banque ». Le doc-comment de `first_matching_rule` (`kesh-reconciliation/src/rules.rs:72-75`,
   « non archivé ») reste tel quel : `kesh-reconciliation` est hors périmètre, comme pour la 15-5b — le
   site d'appel fait foi. La garde de l'AC2 reste le filet d'un client d'API ou d'une règle modifiée
   entre la proposition et l'acceptation.
3. **AC3 — Le choix de la forme est écrit.** Les deux refus reprennent ceux du flux ventilé
   **existant** — forme **et place** — pour qu'un même défaut rende la même réponse sur les **quatre**
   chemins (manuel, ventilé direct, ventilé par lot, règle) : compte de banque **actif** pris pour
   contrepartie → `VALIDATION_ERROR` ; devenu **non imputable** → `VALIDATION_ERROR`, pas
   `ACCOUNT_NOT_POSTABLE` ; **archivé** → `BANK_ACCOUNT_NOT_CONFIGURED` (412 sur les routes directes,
   `failed[]` sur le lot — sur la règle grâce à l'étape ajoutée par l'AC2). La **comparaison** emploie
   celle du helper commun de la 15-6b (en-tête) ; la **construction** du refus reste locale à
   `reconciliation.rs`, **une** fonction pour les deux `FailedProposal` (`accept_one_split`,
   `accept_one_rule`) et **une** pour les deux `AppError::Validation` (`post_split`, `post_manual`),
   les deux sites ventilés existants y étant ramenés. Signature de la seconde (indicative) :
   `fn counterparty_is_bank_ledger_error(field: &str) -> AppError`, qui rend
   `AppError::Validation(format!("{field} ne peut pas être le compte ledger banque"))` —
   `post_split` lui passe `splits[{idx}].counterpartyAccountId`, `post_manual` `counterpartyAccountId`.
   Les tests 1 et 8 n'assertent que le **code** `VALIDATION_ERROR`, pas le message (en dur, non
   traduit). **Limite assumée** (héritée
   de `post_split`, non introduite ici) : le message du 400 manuel est en français en dur, non
   traduit ; le code `VALIDATION_ERROR` et, pour le lot, `details.reason` sont les champs stables. Les
   doc-comments des deux gardes renvoient l'une à l'autre et au flux ventilé.
4. **AC4 — L'écran du rapprochement manuel ne propose pas le compte de la banque** (choix C-15-6-23).
   - **Résolution** dans `ReconciliationProposals.svelte` (qui reçoit `bankAccountId`, et dont la page
     n'a pas de test) : `listBankAccounts()` (`frontend/src/lib/features/bank-accounts/bank-accounts.api.ts:96`,
     comptes non archivés) **une fois, au montage**. **Pas de garde de génération** : la page remonte le
     composant à chaque changement de compte bancaire (`{#key selectedId}`,
     `frontend/src/routes/(app)/reconciliation/+page.svelte:70-72`), si bien qu'une réponse ne peut pas
     arriver pour un autre `bankAccountId` que celui du montage (choix C-15-6-28). Le doc-comment de
     l'appel le dit, et dit que le retrait du `{#key}` exigerait la garde. Le `journalAccountId` du
     compte sélectionné est passé à `ManualMatchModal` (prop neuve `bankLedgerAccountId: number | null`,
     `null` par défaut).
   - **Repli** : pendant le chargement, en cas d'échec de l'appel (erreur ignorée, sans message —
     comme le chargement des comptes du même composant, `:75-90`), si le compte bancaire est absent de
     la liste (archivé) ou non lié (`journalAccountId` nul) → `null` : **aucun filtrage**, la garde
     serveur (AC1) tranche.
   - **Filtre** dans `ManualMatchModal` : **local, une condition** dans `filteredAccounts` (`:65-69`),
     `a.id !== bankLedgerAccountId` — **sans conditionnel sur la 15-6b** et sans emprunt à
     `account-options.ts` (un retrait d'un seul id ne justifie pas la fonction d'ensemble ; rien ne le
     remplacera). Le filtre de classes 5/6/7 reste.
   - *(La page lit déjà `/api/v1/companies/current`, dont `bankAccounts` porte `journalAccountId`
     — `crates/kesh-api/src/routes/companies.rs:315-323` — mais son type local ne le déclare pas, et la
     page n'a aucun test : résoudre dans le composant garde le câblage testable dans son fichier de
     test existant.)*
   - **Écran de ventilation** (`TransactionSplitModal.svelte:71-72`, même filtre 5/6/7) : **non
     filtré**, angle mort écrit (§ *Angles morts*) — le serveur y refuse déjà.
5. **AC5 — L'écran des règles, figé.** `RuleFormModal.svelte:44-50` exclut le compte de banque par le
   type (charges et produits seulement) **en création**. En **édition**, `withCurrentAccount` (`:55-57`,
   #271) réintroduit le compte en place de la règle même non éligible : une règle créée par l'API sur le
   compte de banque, ouverte en édition, le propose et l'enregistre tel quel. **Aucun changement de
   code** : la réintroduction est voulue (#271 — sinon le champ s'afficherait vide sur une règle
   complète) ; une telle règle n'est plus proposée (AC2 bis) et elle est refusée à l'acceptation (AC2).
   Un commentaire dit les deux cas et renvoie à la garde serveur.
6. **AC6 — Libellé dans `failed[]`.** `VALIDATION_ERROR` fait partie des codes que la **15-5c**
   traduit (`failed-proposal-label.ts`, sa liste de 26 codes) : rien à ajouter si elle est mergée. Le
   libellé est **générique** — la 15-5c le relève sur un message existant ; au 2026-10-08,
   `error-validation` dit « Erreur de validation » (`crates/kesh-i18n/locales/fr-CH/messages.ftl:36`)
   — et `details.reason` n'est pas affiché : l'utilisateur n'apprend pas que la contrepartie est le
   compte de banque. **Assumé** : depuis l'AC2 bis, l'écran ne propose plus une telle règle ; ce
   libellé n'est atteint que par un client d'API ou une règle modifiée entre la proposition et
   l'acceptation. Le dev relit le libellé effectif de la 15-5c et l'écrit au Dev Agent Record. Si la
   15-5c n'est pas mergée : le signaler, sans second mécanisme.
7. **AC7 — Documentation des intégrateurs** (`docs/api-external.md`) : au § *Accepter des propositions
   de rapprochement* (`:303-307`), `VALIDATION_ERROR` avec `details.reason = "counterparty_equals_bank_ledger"`
   pour une proposition par règle (et par ventilation, déjà vrai et non documenté), et
   `BANK_ACCOUNT_NOT_CONFIGURED` (`details.bankAccountId`) pour une proposition par règle dont le compte
   de banque est archivé (AC2) ; une demi-phrase dit qu'une règle visant le compte de la banque n'est
   pas proposée (AC2 bis). **Rapprochement manuel et ventilation directe** : la **15-5b**, mergée
   avant, ajoute au même § une phrase disant que ces deux routes refusent une contrepartie non
   imputable en `400 ACCOUNT_NOT_POSTABLE`, « après le `404 ACCOUNT_NOT_FOUND` »
   (`15-5b-gardes-surfaces-neuves.md:323-330`, dépôt principal). Cette phrase est **complétée** : les
   deux routes refusent une contrepartie égale au compte de la banque en `400 VALIDATION_ERROR` (message
   seul, sans `details.reason`), **avant** le 404 et le non-imputable. T0 refait
   `grep -n "reconciliation/manual\|reconciliation/split" docs/api-external.md` sur l'état rebasé
   (vide au 2026-10-08, avant la 15-5b) ; s'il est encore vide, la 15-5b n'est pas mergée : le
   signaler, sans écrire sa phrase à sa place.
8. **AC8 — Manuel** (`docs/manual/fr/user-manual.tex`) : une phrase qui vaut pour les **trois**
   flux — rapprochement manuel (`:1536-1545`), éclatement (`:1546-1560`) et règles d'affectation
   (`:1561-1586`) — : la contrepartie d'un rapprochement ne peut pas être le compte comptable du compte
   bancaire lui-même (l'écriture serait nulle) ; Kesh **le refuse** dans les trois cas — seule
   affirmation sans réserve — et, **en règle générale**, ne le propose pas dans le rapprochement manuel
   ni ne propose une règle qui le vise (finding F3-4 de la P3 : la fiche connaît des cas où il est
   proposé — échec du chargement des comptes bancaires, compte non lié, plan atypique, écran de
   ventilation jamais filtré ; le manuel ne promet pas une protection partielle comme totale). *(La section *Éclatement* ne
   dit aujourd'hui rien de ce refus : ne pas écrire « comme pour un éclatement ».)* La section du
   rapprochement manuel est inexacte et la **15-5c** la réécrit : écrire **sur sa version**. PDF
   régénéré et contrôlé **aplati**.
9. **AC9 — CHANGELOG** sous `## [0.13.0] — Non publié`, `### Corrigé`, renvoi à
   [#524](https://github.com/guycorbaz/kesh/issues/524) ; et `### Modifié` (finding F3-5 de la P3) :
   l'acceptation d'une proposition par règle dont le compte de banque est archivé rend désormais
   `BANK_ACCOUNT_NOT_CONFIGURED` (`details.bankAccountId`) dans `failed[]` — changement délibéré d'un
   refus existant (AC2), visible d'un intégrateur ; le code d'avant, relevé en T0, y est nommé. La
   section `[0.13.0]` est absente au 2026-10-08 (`grep -n "^## \[" CHANGELOG.md` → premier `0.12.1`) :
   T0 le revérifie après rebase. **Propriétaire de la section** : la première
   story de la version mergée la crée en tête des versions si elle est absente ; les suivantes y
   ajoutent leur entrée.

## Tasks / Subtasks

- [x] **T0 — Rebase après la 15-5b et la 15-6b** (et la 15-5c si mergée — elle touche
  `ReconciliationProposals.svelte` et `ReconciliationProposals.test.ts`) ; relire les numéros de ligne
  de `post_manual`, `accept_one_rule`, `accept_one_split`, `post_split` et `get_proposals` ; vérifier
  où la 15-5b a placé `ACCOUNT_NOT_POSTABLE` (le refus de cette story le précède), comment elle a réduit
  `active_account_ids` (AC2 bis s'y compose), où elle a posé le helper `set_account_not_postable` (le
  réutiliser, ou le dupliquer dans chaque binaire de test qui l'emploie : tests 2, 5, 8, 9) ; reprendre
  le nom réel du helper de comparaison de la 15-6b ; relever le code que rend aujourd'hui une règle
  ordinaire sur un compte de banque archivé (AC2) ; refaire le grep de `docs/api-external.md` (AC7).
- [x] **T1 — Gardes serveur** (AC1, AC2 dont l'étape du compte de banque actif, AC3 et ses deux
  fonctions de refus) et proposition (AC2 bis, commentaire du site d'appel).
- [x] **T2 — Écrans** (AC4, AC5) — dont le mock de `bank-accounts.api` dans
  `ReconciliationProposals.test.ts` ; libellé (AC6) ou signalement.
- [x] **T3 — Tests** (§ *Tests*), chaque test d'une garde neuve **rougit d'abord** ; les témoins 8
  et 9 (gardes ventilées existantes) passent avant **et** après.
- [x] **T4 — Documentation** : `docs/api-external.md` (AC7), manuel + PDF (AC8), CHANGELOG (AC9).
- [x] **T5 — Gates** : backend complet, frontend complet, E2E complet au dernier commit de code (D7).

## Tests

Backend :

1. `crates/kesh-api/tests/reconciliation_manual_e2e.rs` — `POST /reconciliation/manual` avec
   `counterpartyAccountId` = compte lié du compte bancaire → 400 `VALIDATION_ERROR` ; **aucune
   écriture**, transaction bancaire toujours `pending`, **aucun audit** (patron
   `post_manual_emits_audit_log_pair`, `:1217`).
2. Même fichier — **ordre** : le compte de banque rendu **non imputable** (`postable = FALSE`, toujours
   actif — `set_account_not_postable` de la 15-5b, UPDATE direct, `version + 1`, sur le patron de
   `products_revenue_account_e2e.rs:254-260`) et pris pour contrepartie rend **ce refus**
   (`VALIDATION_ERROR`), **pas** `ACCOUNT_NOT_POSTABLE` (AC1, même place que le flux ventilé). *(Un
   compte de banque **archivé** n'atteint pas cette garde : l'étape 2 bis rend 412
   `BANK_ACCOUNT_NOT_CONFIGURED` avant — de même sur les trois autres chemins, test 7 pour la règle.)*
3. Même fichier — **transaction déjà rapprochée** et contrepartie = compte de banque → 400
   `VALIDATION_ERROR`, pas 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` (AC1).
4. `crates/kesh-api/tests/reconciliation_rules_e2e.rs` — une règle dont la contrepartie est le compte
   lié (créée par l'API — **avant** tout passage en non imputable : après la 15-5b, AC7, la création
   contrôle `postable` — ou en SQL). Depuis l'AC2 bis, `GET /reconciliation/proposals` ne rend plus
   cette proposition : le test **construit** le corps (`type: "rule"`, `bankTransactionId`, `ruleId`,
   `counterpartyAccountId`), patron `post_accept_rule` (`:1255`). `POST /reconciliation/accept` d'un
   lot de deux propositions, l'une par cette règle, l'autre valide → HTTP 200, l'une acceptée, l'autre
   dans `failed[]` avec `VALIDATION_ERROR` et `details.reason = "counterparty_equals_bank_ledger"` ; sa
   transaction reste en attente, **aucune écriture ni audit** pour elle.
5. Même fichier — **ordre côté règle** : la même règle, **créée d'abord**, puis le compte de banque
   rendu non imputable ; proposition construite comme au test 4 → `failed[]` `VALIDATION_ERROR` /
   `counterparty_equals_bank_ledger`, **pas** `ACCOUNT_NOT_POSTABLE` (AC2).
6. Même fichier (ou celui des propositions, à repérer en T0) — **proposition** (AC2 bis) : deux règles
   concordant avec la même transaction, la **première** sur le compte de banque, la seconde valable →
   `GET` des propositions : la proposition porte la **seconde** ; avec la première seule, aucune
   proposition par règle.
7. Même fichier — **compte de banque archivé, chemin par règle** (AC2, étape ajoutée) : le compte lié
   du compte bancaire archivé en SQL (`UPDATE accounts SET active = FALSE`) **après** la création des
   deux règles par l'API — la création refuse un compte archivé (`validate_counterparty_account`,
   `reconciliation_rules.rs:197-221`, 404 `ACCOUNT_NOT_FOUND`) : règles d'abord, archivage ensuite
   (findings R2 = F3-3 de la P3) ; deux propositions
   construites, l'une par une règle ordinaire (contrepartie 6510), l'autre par la règle sur le compte
   de banque → les deux dans `failed[]` en `BANK_ACCOUNT_NOT_CONFIGURED` avec
   `details.bankAccountId`, ni `VALIDATION_ERROR` ni le refus de l'écriture ; rien d'écrit. *(Rougit
   sur le code actuel.)*
8. `crates/kesh-api/tests/reconciliation_split_e2e.rs` — **témoin de l'ordre ventilé direct** :
   `POST /reconciliation/split`, une ligne dont la contrepartie est le compte de banque rendu non
   imputable → 400 `VALIDATION_ERROR`, pas `ACCOUNT_NOT_POSTABLE` ; aucune écriture. Fige l'ordre
   « égalité avant postabilité » sur lequel repose C-15-6-23 (garde `:3462-3475`, aucun test
   aujourd'hui : `grep -rnF "compte ledger banque" crates/kesh-api/tests` vide).
9. `crates/kesh-api/tests/reconciliation_e2e.rs` — **témoin de l'ordre ventilé par lot** : acceptation
   d'une proposition `type: "split"` (patron `:2579`) dont une ligne vise le compte de banque rendu non
   imputable → `failed[]` `VALIDATION_ERROR` / `details.reason = "counterparty_equals_bank_ledger"`, pas
   `ACCOUNT_NOT_POSTABLE` ; aucune écriture (garde `:1941-1950`, aucun test aujourd'hui :
   `grep -rnF "counterparty_equals_bank_ledger" crates/kesh-api/tests` vide).

Frontend (Vitest) :

10. `frontend/src/lib/features/reconciliation/ManualMatchModal.test.ts` (existant ; ses montages
    passent la prop neuve) : le compte passé en `bankLedgerAccountId` n'est pas proposé, même numéroté
    en classe 5/6/7 ; les autres le sont ; `null` → aucun filtrage.
11. `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` (existant) : **mock de
    `$lib/features/bank-accounts/bank-accounts.api` ajouté** (les tests existants ne mockent que
    `reconciliation.api`, `accounts.api` et l'i18n, `:20`, `:29` — sans lui, l'appel neuf partirait sur
    l'`apiClient` réel) ; la modale ouverte ne propose pas le `journalAccountId` du compte bancaire
    monté ; `listBankAccounts` en échec → compte non écarté, aucune erreur affichée.
12. `frontend/src/lib/features/reconciliation/rules/RuleFormModal.test.ts` (existant) : **en création**,
    un compte d'actif n'est pas proposé (fige l'AC5).

Soit **12 tests nommés** (9 backend dont 2 témoins, 3 fichiers Vitest).

## Dev Notes

### Ce qui doit être préservé

- L'ordre des refus existants de `post_manual` et `accept_one_rule` ; le refus neuf s'y **insère**
  avant la validation de la contrepartie, comme dans le flux ventilé — il ne déplace aucun refus
  existant, **hors** le changement délibéré de l'AC2 (compte de banque archivé sur le chemin par règle,
  désormais `BANK_ACCOUNT_NOT_CONFIGURED` ; finding R3 de la P3).
- La forme des refus du flux ventilé : ne pas les « améliorer » ici (un code dédié, une traduction) —
  ce serait diverger des deux chemins existants ; si un code dédié devient souhaitable, il vaudra pour
  les quatre chemins et sera une story à part. Le helper de la 15-6b n'est emprunté que pour la
  comparaison : sa construction de refus (variante nommée d'un compte de créance ou de dette) n'est pas
  celle-ci (AC3).
- **Aucun verrou à ajouter** pour l'égalité : sur les quatre chemins, la valeur comparée est **celle qui
  construit l'écriture** — `post_manual` lit `bank_ledger_account_id` (`:2989`) et passe la même
  variable à `build_journal_entry_for_counterparty` (`:3143-3147`) ; les chemins de lot font de même
  dans la transaction. Un relien concurrent du compte bancaire peut faire imputer l'**ancien** compte
  de banque (dette préexistante, hors #524), jamais produire une écriture `D x / C x`. Le
  `with_account_lock` des acceptations est un `GET_LOCK` consultatif
  (`crates/kesh-reconciliation/src/mutex.rs:66-90`) que les routes des comptes bancaires ne prennent
  pas : ce n'est pas lui qui rend l'égalité sûre.

### Angles morts assumés

- La **création** d'une règle sur un compte qui est, ou deviendra, le compte de banque d'un compte
  bancaire n'est pas refusée : une règle n'est pas attachée à un compte bancaire. Elle n'est plus
  **proposée** (AC2 bis) et elle est refusée à l'acceptation (AC2), seul moment où le couple existe.
- **Écran de ventilation** (`TransactionSplitModal.svelte:71-72`) : il propose encore le compte de
  banque s'il est numéroté en classe 5/6/7 (plan atypique) ; le serveur le refuse déjà (400 /
  `failed[]`, gardes `:1941-1950`, `:3462-3475`). Filtre d'écran non étendu (choix C-15-6-23).
- **Message du repli d'écran** : quand le filtre ne s'applique pas (échec de `listBankAccounts`,
  compte non lié, plan atypique), `ManualMatchModal.svelte:126-131` affiche `e.message` du 400 — en
  français en dur dans les quatre langues (limite héritée de `post_split`, AC3). Pas de clé i18n ici
  (§ *Ce qui doit être préservé*).
- **Édition d'une règle sur le compte de banque** (AC5) : l'écran la propose et l'enregistre ; elle
  n'est ni proposée ni acceptée.
- **Compte de banque archivé avant l'ouverture de l'écran** (AC2) : `get_proposals` ne contrôle pas
  l'activité du compte lié ; une règle ordinaire est proposée et son acceptation rend
  `BANK_ACCOUNT_NOT_CONFIGURED`. Pas de contrôle ajouté à la proposition.
- **Câblage de l'écran vu seulement sous mock** (finding F3-6 de la P3) : le test 11 mocke
  `listBankAccounts` ; `frontend/tests/e2e/reconciliation-manual.spec.ts` est un smoke sans fixtures ;
  le passage `journalAccountId` API → prop → filtre n'est jamais exercé de bout en bout. Risque faible
  (le parseur du champ est couvert, `bank-accounts.api.test.ts:37`, `:120`), assumé.

### Références

- [Source: crates/kesh-api/src/routes/reconciliation.rs:469-478, :556-557, :634-638, :1829, :1912-1950, :2207-2330, :2942-3060, :3143-3147, :3362, :3446-3475 ;
  crates/kesh-reconciliation/src/rules.rs:72-95 ; crates/kesh-reconciliation/src/mutex.rs:66-90 ;
  crates/kesh-api/src/routes/companies.rs:315-323 ; crates/kesh-api/src/routes/bank_accounts.rs:260-294]
- [Source: crates/kesh-reconciliation/src/manual.rs:67]
- [Source: frontend/src/lib/features/reconciliation/ManualMatchModal.svelte:65-69, :126-131, :157-163 ;
  frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:26-34, :75-90, :330-348 ;
  frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts:20, :29 ;
  frontend/src/routes/(app)/reconciliation/+page.svelte:70-72 ;
  frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte:71-72 ;
  frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte:44-57 ;
  frontend/src/lib/features/accounts/account-options.ts:58-74]
- [Source: crates/kesh-api/tests/reconciliation_rules_e2e.rs:1255, :1360 ; crates/kesh-api/tests/reconciliation_e2e.rs:2579 ;
  crates/kesh-api/tests/products_revenue_account_e2e.rs:254-260]
- [Source: docs/manual/fr/user-manual.tex:1536-1586 ; docs/api-external.md:303-307 ; crates/kesh-i18n/locales/fr-CH/messages.ftl:36]
- [Source: `15-5b-gardes-surfaces-neuves.md` AC1, AC4, AC5, AC7, `:323-330`, T6 ; `15-5c-rapprochement-libelles-et-manuel.md`
  AC1, T3 (dépôt principal) ; `15-6b-contrepartie-distincte-de-la-creance.md` (helper commun)]
- [Source: issue #524 ; `epic-15-choix-autonomes.md` C-15-6-6, C-15-6-9, C-15-6-23, C-15-6-28]

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), en autonomie (consignes de l'Epic 15).

### Debug Log References

**T0 — relevés au sol, 2026-10-09, sur `f2c5e419` (`origin/main`), avant tout code.**

- **Prérequis** : 15-5b, 15-5c et 15-6b mergées (sprint-status `done`) ; la **15-6c** (PR #586)
  ne l'est pas au relevé (`git fetch` : `origin/main` = `f2c5e419`) — rebase à refaire si elle
  arrive pendant le travail. Aucun rebase nécessaire au départ : la branche part de `origin/main`.
- **Numéros de ligne** (`crates/kesh-api/src/routes/reconciliation.rs`, 4357 lignes ; la fiche
  citait l'état d'avant la 15-5b) : `get_proposals` `:527` (`ba_check` `:547-556`, qui porte
  `journal_account_id` ; `active_account_ids` `:639-643`, réduit par la 15-5b aux comptes actifs
  **et** imputables, commentaire de site `:629-638` ; `first_matching_rule` `:724-728`) ;
  `accept_one_split` `:1997` (« Step c » `:2079-2105`, garde d'égalité `:2108-2118`, puis
  « Step d » `ACCOUNT_NOT_POSTABLE`) ; `accept_one_rule` `:2400` (étape 1 `:2411-2445`, étape 2
  `:2447`, étape 4 `:2466`, étape 5 `:2478-2525` avec `ACCOUNT_NOT_POSTABLE` de la 15-5b) ;
  `post_manual` `:3178` (étape 2 `:3225`, 2 bis `:3233-3249`, étape 3 `:3251-3265`, 3 bis
  `ACCOUNT_NOT_POSTABLE` `:3266-3280`, étape 4 `:3282`) ; `post_split` `:3577` (« Step 4bis »
  `:3662-3674`, garde `:3676-3690`, texte `:3686`).
- **Helper de la 15-6b** : `kesh_db::repositories::invoice_settlements::ensure_not_claim_account(account_id: i64, claim_account_id: i64) -> Result<(), ClaimAccountClash>`
  (`invoice_settlements.rs:694`), pure, sans `async` — nom et signature conformes à la fiche ;
  son doc-comment annonce déjà l'emprunt par la 15-6d.
- **`set_account_not_postable`** : la 15-5b l'a **dupliqué** dans chaque binaire de test — il
  existe déjà dans les quatre fichiers visés : `reconciliation_manual_e2e.rs:1325`,
  `reconciliation_rules_e2e.rs:2058`, `reconciliation_split_e2e.rs:1006`,
  `reconciliation_e2e.rs:5145` (nommé `set_account_not_postable_15_5b`). Réutilisé tel quel.
- **Code rendu aujourd'hui** pour une règle ordinaire sur un compte de banque archivé (AC2) : par
  lecture, l'acceptation passe les étapes 1 à 11 et `journal_entries::create_in_tx` rend
  `DbError::InactiveOrInvalidAccounts` (`journal_entries.rs:162-163`), que le repli générique
  d'`accept_one_rule` mappe en **`DATABASE_ERROR`** (`details.message`). **Mesuré** au rouge du
  test 7 (ci-dessous, T3) avant d'être écrit au CHANGELOG.
- **`docs/api-external.md`** : `grep -n "reconciliation/manual\|reconciliation/split"` rend la
  phrase de la 15-5b à `:399` — l'AC7 la complète. § *Accepter des propositions* `:385-397`.
- **CHANGELOG** : `## [0.13.0] — Non publié` présent (`:11`), avec `### Modifié` (`:17`) et
  `### Corrigé` (`:37`).
- **Libellé de la 15-5c (AC6)** : `VALIDATION_ERROR` → clé `error-validation`, « Erreur de
  validation » (`failed-proposal-label.ts:259-260`, fr-CH `messages.ftl:42`). Son doc-comment
  (`:44-47`) compte « six raisons sur sept sites » : la garde de la règle ajoute un **huitième**
  site — à mettre à jour (propagation).
- **Manuel** : sections réécrites par la 15-5c — *Réconciliation manuelle* `:1687`, *Éclatement*
  `:1701`, *Règles d'affectation* `:1715` (`user-manual.tex`).
- **Occupation du tmpfs MariaDB** (lecture seule, `df -h /var/lib/mysql` dans
  `kesh-mariadb-dev`) avant tout gate : **1,3 Go / 4,0 Go (31 %)**.

**Écarts avec la fiche, ventilés :**

1. **Test 11 — la modale est une doublure.** Depuis la revue P1 de la 15-5c,
   `ReconciliationProposals.test.ts` remplace `ManualMatchModal` par `ModalSuccessStub.test.svelte`
   (`:36-41`) : « la modale ouverte ne propose pas le `journalAccountId` » n'y est pas observable.
   Le test 11 asserte donc la **valeur de la prop** `bankLedgerAccountId` reçue par la doublure
   (étendue pour l'exposer), et le test 10 asserte le filtre dans la vraie modale ; les deux
   ensemble couvrent le câblage. Choix C-15-6d-1.
2. **Numéros de ligne** : tous décalés par les 15-5b/c, 15-6b, 15-5e2 (ci-dessus) ; aucun
   changement de fond — l'ordre décrit par l'AC1 et l'AC2 est celui du code.
3. **`set_account_not_postable`** : la fiche hésitait (réutiliser ou dupliquer) ; il est déjà
   dans chaque binaire — rien à ajouter.
4. **AC6** : la 15-5c est mergée ; rien à ajouter au mécanisme, seul le doc-comment de décompte
   (écart de propagation) change.
5. **AC7** : la phrase de la 15-5b existe (`:399`) — complétée, pas créée.

**Mesuré au rouge (T3, avant toute ligne de code de production)** — sur `f2c5e419` :

- test 1 : **200** — l'écriture `D 1020 / C 1020` était bien passée (le défaut de #524 reproduit).
  Premier essai sans `description` : `400 CHECK_CONSTRAINT_VIOLATION`
  (`chk_journal_entries_description_nonempty`) — le test aurait passé **sans** la garde ; une
  description est désormais envoyée, commentaire au test ;
- test 2 : `ACCOUNT_NOT_POSTABLE` ; test 3 : `404` ; test 4 : la proposition sur le compte de
  banque **acceptée** (`failed[]` vide) ; test 5 : `ACCOUNT_NOT_POSTABLE` ; test 6 : la règle
  proposée était celle du compte de banque ;
- test 7 — **code rendu aujourd'hui** (AC2, AC9) : règle ordinaire → **`DATABASE_ERROR`**,
  `details.message` = « Un ou plusieurs comptes sont archivés ou invalides » (garde `active` de
  l'écriture) ; règle sur le compte de banque → **`ACCOUNT_NOT_FOUND`** (`missingAccountIds`).
  Écrit au CHANGELOG (`### Modifié`) et au guide ;
- témoins 8 et 9 : **verts avant** (et après), comme exigé.

**Rebase pendant le travail** : la **15-6c** (PR #586) a été mergée (`803f3e15`) après le commit de
la documentation ; branche rebasée sur `803f3e15`. Conflits résolus **par union** : registre des
choix (C-15-6c-1..6 de `main` + C-15-6d-1), sprint-status (ligne 15-6c de `main`, ligne 15-6d de la
branche ; en-tête `(44)`), CHANGELOG (entrée #474 réécrite par la 15-6c conservée, entrée #524
ajoutée). `user-manual.pdf` **régénéré** (`make -B user`) sur le `.tex` fusionné, contrôlé aplati.
La 15-6c ne touche pas `reconciliation.rs` ni de migration ; les gates ci-dessous sont sur l'état
rebasé.

### Completion Notes List

- **Serveur** (`reconciliation.rs`) : trois fonctions neuves — `is_bank_ledger` (comparaison par
  `invoice_settlements::ensure_not_claim_account`, helper de la 15-6b), `counterparty_is_bank_ledger_failed_proposal`
  (lot) et `counterparty_is_bank_ledger_error(field)` (routes directes) ; les deux sites ventilés
  existants y sont ramenés (`accept_one_split`, `post_split` — texte inchangé). `post_manual` :
  étape 2 ter, après 2 bis, avant le 404 / `ACCOUNT_NOT_POSTABLE` / l'état de la transaction (AC1).
  `accept_one_rule` : étape 1 bis (compte de banque actif → `BANK_ACCOUNT_NOT_CONFIGURED`,
  C-15-6-28) et étape 4 bis (égalité, après `RULE_MISMATCH`, avant l'étape 5) ; ordre écrit dans
  les doc-comments des deux handlers et de `post_split` (AC2, AC3). `get_proposals` : le compte de
  la banque (`ba_check.journal_account_id`) retiré de `active_account_ids`, commentaire du site mis
  à jour (AC2 bis).
- **Écrans** : `ReconciliationProposals.svelte` résout `journalAccountId` par `listBankAccounts()`
  une fois au montage, sans garde de génération (doc-comment : le `{#key}` de la page) ;
  `ManualMatchModal.svelte` reçoit `bankLedgerAccountId` (`null` par défaut) et l'écarte par une
  condition locale (AC4). `RuleFormModal.svelte` : commentaire seul (AC5). `TransactionSplitModal`
  non filtré (angle mort de la fiche).
- **AC6** : la 15-5c est mergée ; libellé effectif `error-validation` = « Erreur de validation »
  (fr-CH), générique, `details.reason` non affiché — assumé. Doc-comment de
  `failed-proposal-label.ts` recompté : six raisons sur **huit** sites. **Aucune clé i18n** ajoutée
  ni modifiée (AC3, « Ce qui doit être préservé ») : les quatre locales sont inchangées.
- **Docs** : `docs/api-external.md` (AC7 — refus par lot, archivé, règle non proposée, phrase de la
  15-5b complétée) ; manuel FR (AC8, paragraphe *La contrepartie n'est jamais le compte de la
  banque*, après *Éclatement* ; refus sans réserve, filtrage « en règle générale ») et PDF
  régénéré (`make -B user`), contrôlé aplati ; CHANGELOG `[0.13.0]` `### Corrigé` et
  `### Modifié` (AC9). Manuels DE/IT/EN : vides (README seul), rien à traduire.
- **Tests** : 9 backend (`reconciliation_manual_e2e` 3, `reconciliation_rules_e2e` 4,
  `reconciliation_split_e2e` 1, `reconciliation_e2e` 1 — recomptés par
  `git diff origin/main HEAD | grep -c '^+#\[sqlx::test'`) et 7 Vitest (test 10 : 2 ; test 11 : 4,
  dont un `it.each` à 2 cas ; test 12 : 1). Périmètre : `origin/main` (`803f3e15`) → `HEAD`.
- **Mutations** (rejouées, toutes rouges ; journal `kesh-gate-logs/156d-mutations-backend.log`) :
  - M1 garde de `post_manual` neutralisée → tests 1, 2, 3 rouges ;
  - M2 garde d'égalité de la règle neutralisée → tests 4, 5 ;
  - M3 contrôle du compte de banque actif de la règle neutralisé → test 7 ;
  - M4 retrait du compte de banque de `active_account_ids` neutralisé → test 6 ;
  - M5 garde de `post_manual` déplacée après `ACCOUNT_NOT_POSTABLE` → test 2 ;
  - M6 garde de `post_manual` déplacée après l'étape 4 → tests 2, 3 ;
  - M7 garde de la règle déplacée après l'étape 5 → test 5 ;
  - M8 (témoin) garde de `post_split` neutralisée → test 8 ;
  - M9 (témoin) garde d'`accept_one_split` neutralisée → test 9 ;
  - M10 comparaison inversée dans `is_bank_ledger` → tests 1 à 5 ;
  - frontend F1 filtre de la modale retiré → test 10 ; F2 prop non passée → test 11 (4 cas) ;
    F3 résolution sans l'identifiant monté (`list[0]`) → test 11 (2 cas) ; F4 repli d'échec non
    nul → test 11 (cas d'échec).
  Restauration par `git checkout --` du fichier puis `touch` (binaire cargo).
- **Gates** (état rebasé sur `803f3e15`, code final = commit `04510336`) : voir le Change Log.

### File List

- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs`
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs`
- `crates/kesh-api/tests/reconciliation_split_e2e.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte`
- `frontend/src/lib/features/reconciliation/ManualMatchModal.test.ts`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/ModalSuccessStub.test.svelte`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.test.ts`
- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`
- `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/15-6d-contrepartie-distincte-de-la-banque.md`,
  `sprint-status.yaml`, `epic-15-choix-autonomes.md`

## Change Log

- 2026-10-08 — Création à la validation P1 de la 15-6b (findings F1 = R3 et F4), en autonomie :
  #524 sort de la 15-6b, qui dépassait cinq modules avec lui. Choix C-15-6-9. **9 AC, 6 tâches
  (T0–T5), 5 tests.** Statut `ready-for-dev`. Validation : à lancer.
- 2026-10-08 — **Validation P1** (Opus 5.5, lentilles R et F ; prompt `15-6d-validate-prompt-p1.md`).
  R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW ; F : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 4 LOW (R-1 = F3,
  R-2 ⊃ F5, R-3 ≈ F4, R-5 = F7). Tous appliqués, décisions de l'orchestrateur comprises :
  - **MEDIUM F1** — l'égalité avec le compte de banque est testée **avant** le 404 et
    `ACCOUNT_NOT_POSTABLE`, comme le flux ventilé : même réponse sur tous les chemins, plus de
    dépendance d'ordre envers la 15-5b ; AC1, AC2, AC3, test 2 inversé. Choix C-15-6-23, qui révise
    l'ordre de C-15-6-9.
  - **MEDIUM F2** — `get_proposals` ne propose plus une règle sur le compte de banque, qui masquait une
    règle suivante (AC2 bis, test 6).
  - **MEDIUM R-1 = F3** — câblage `ReconciliationProposals` → `ManualMatchModal` spécifié (effet
    dépendant de `bankAccountId`, garde de génération) et testé (test 8), mock de `bank-accounts.api`
    dans le fichier existant, repli écrit (chargement, échec, compte non lié ou archivé → aucun
    filtrage).
  - **MEDIUM R-3** — filtre local d'une condition, sans conditionnel sur la 15-6b (AC4).
  - **MEDIUM R-2 (⊃ F5)** — « aucune écriture / aucun audit » asserté (tests 1, 4) ; ordre côté règle
    testé (test 5) ; transaction déjà rapprochée → 400, écrit et testé (AC1, test 3).
  - **LOW** — R-4 : phrase du manuel pour les trois flux, ligne des règles, « comme pour un
    éclatement » retiré ; R-5 = F7 : guide sans section manuel/ventilation → « rien à ajouter » ;
    R-6 : libellé générique dit et assumé (AC6) ; F4 : écran de ventilation non filtré, angle mort
    écrit ; F6 : course résiduelle de `post_manual` écrite.
  - Signal de découpage : sans objet (lentille F : `kesh-api` rapprochement, `frontend`
    rapprochement, docs — sous le seuil ; aucun défaut recyclé, première passe).
  - Décompte après passe : **10 AC (AC1–AC9 et AC2 bis), 6 tâches (T0–T5), 9 tests** (6 backend,
    3 fichiers Vitest).
- 2026-10-08 — **Validation P2** (Opus 5.5, lentilles R et F ; prompt `15-6d-validate-prompt-p2.md`).
  R : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW ; F : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW (R-1 = F-4,
  R-2 = F-3, R-4 = F-7). Tous appliqués, décisions de l'orchestrateur comprises :
  - **MEDIUM R-1 (= F-4 LOW)** — `accept_one_rule` reçoit le contrôle du compte de banque actif (étape
    de la « Step c » d'`accept_one_split`), **avant** la garde d'égalité : même réponse sur les quatre
    chemins, y compris compte de banque archivé (AC2, AC3) ; AC1 rectifié (le flux ventilé compare
    **après** ce contrôle, avant la validation des contreparties) ; test 7 neuf. La voie « angle mort »
    proposée par F-4 est écartée. Choix C-15-6-28.
  - **MEDIUM F-2** — les deux gardes ventilées, sur lesquelles repose l'ordre de C-15-6-23, sont figées
    par deux témoins (tests 8 et 9, `post_split` et `accept_one_split`).
  - **MEDIUM F-1** — AC7 et T0 : le grep de `docs/api-external.md` est refait sur l'état rebasé ; la
    phrase que la 15-5b ajoute sur les routes manuelle et ventilée est complétée par le refus de cette
    story, qui précède le 404. *(Corrige la ligne R-5 = F7 de la P1 : « rien à ajouter » n'était vrai
    qu'avant la 15-5b.)*
  - **LOW** — F-3 = R-2 : angle mort « course résiduelle » retiré ; la valeur comparée est celle qui
    construit l'écriture, aucun verrou à ajouter (Dev Notes) ; F-6 : garde de génération et sous-cas
    « changement de `bankAccountId` » retirés, la page remonte le composant (`{#key}`) ; R-3 :
    `withCurrentAccount` en édition dit tel quel (AC5, test 12 « en création », angle mort) ; R-4 =
    F-7 : helper `set_account_not_postable` décrit sans sous-compte, emplacement repéré en T0 ; R-5 :
    tests 4 et 5 construisent leur proposition, règle créée avant le passage en non imputable,
    parenthèse sur la création retirée ; R-6 : la 15-5c citée en en-tête et en T0 pour les deux
    fichiers d'écran ; F-5 : prémisse corrigée (type imposé, classe non) ; F-8 : commentaire du site
    d'appel de la 15-5b mis à jour, doc-comment de `rules.rs` laissé (choix écrit), demi-phrase du
    guide sur la règle non proposée ; F-9 : message du repli d'écran en français, angle mort écrit.
  - **Helper commun** (décision de l'orchestrateur, finding F3 de la P3 de la 15-6b) : la 15-6d emprunte
    la **comparaison** du helper que la 15-6b pose dans `invoice_settlements` (nom indicatif
    `ensure_not_claim_account`) ; la construction du refus reste celle du flux ventilé, ramenée à deux
    fonctions locales (AC3). Ordre : après la 15-6b. Choix C-15-6-28.
  - Signal de découpage (amendement D5) : **non levé**. Sévérité P1 → P2 : MEDIUM → MEDIUM, mais les
    défauts sont **distincts** de ceux de la P1 et ne viennent pas de sa remédiation, sauf F-1, qui
    prolonge une affirmation de la P1 sans la recycler ; modules : `kesh-api` (rapprochement), frontend
    (rapprochement, règles), docs, tests `kesh-api` — sous le seuil. Déclaré au Project Lead par ce
    Change Log.
  - **Renumérotation** : les tests Vitest 7, 8, 9 de la P1 deviennent 10, 11, 12 (les numéros cités par
    l'entrée P1 ci-dessus sont ceux d'alors).
  - Décompte après passe : **10 AC (AC1–AC9 et AC2 bis), 6 tâches (T0–T5), 12 tests** (9 backend dont
    2 témoins, 3 fichiers Vitest).
- 2026-10-08 — **Validation P3 — clôture de la validation** (Sonnet, lentilles R et F ; prompt
  `15-6d-validate-prompt-p3.md` ; remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW ; F :
  0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW (R2 = F3-3). Axes déclarés exercés par les deux lentilles
  (numéros de ligne revérifiés au code, inventaire des appelants du constructeur d'écriture et de
  `active_account_ids`, ordre des contrôles sur les quatre chemins, verrous, tests existants, frontend,
  i18n, manuel `.tex` et PDF aplati, guide, CHANGELOG, règle de découpage, registre) et non exercés
  (aucune exécution ; fiches 15-5b/15-5c non rebasées). Tous les LOW appliqués (choix C-15-6-31) :
  - R1 : la nouvelle étape de l'AC2 est **atteignable depuis l'écran** (compte lié archivé avant
    l'ouverture ; `get_proposals` ne le contrôle pas) — phrase corrigée, angle mort écrit, aucun
    contrôle ajouté ;
  - R2 = F3-3 : test 7 — règles créées d'abord, compte lié archivé ensuite ;
  - R3 : Dev Notes — « ne déplace aucun refus » sauf le changement délibéré de l'AC2 ;
  - R4 : bandeau — C-15-6-28 parmi les choix propres ;
  - F3-1 : le registre ne se réécrit pas ; C-15-6-31 dit que C-15-6-28 **révise** C-15-6-23 (garde de
    génération, course résiduelle) ;
  - F3-2 : texte du refus selon le champ, signature de la fonction de refus (AC1, AC3) ; tests 1 et 8
    sur le code seul ;
  - F3-4 : le manuel n'affirme sans réserve que le refus (AC8) ;
  - F3-5 : `### Modifié` au CHANGELOG pour le changement délibéré de l'AC2 ; section `[0.13.0]` à
    revérifier en T0 (AC9) ;
  - F3-6 : câblage de l'écran vu seulement sous mock, angle mort écrit.
  - Propagation : jetons `Atteignable seulement`, `ne déplace aucun refus`, `Kesh ne le propose pas`,
    `ne peut pas être le compte ledger banque` grepés sur la fiche — aucun résidu hors citations.
  - **Trend complet** (F + R) : P1 (Opus 5.5, lentilles R et F) 0 HIGH, 6 MEDIUM, 7 LOW → P2 (Opus 5.5,
    lentilles R et F) 0 HIGH, 3 MEDIUM, 12 LOW → P3 (Sonnet, lentilles R et F) 0 HIGH, 0 MEDIUM,
    10 LOW. Aucun CRITICAL ni HIGH sur l'ensemble. Reclassements : aucun. Signal de découpage : jamais
    levé (sous le seuil de cinq modules, défauts distincts d'une passe à l'autre).
  - **Validation close** : 0 au-dessus de LOW. Décompte final : **10 AC (AC1–AC9 et AC2 bis),
    6 tâches (T0–T5), 12 tests** (9 backend dont 2 témoins, 3 fichiers Vitest) — inchangé par la P3.
- 2026-10-09 — **Développement** (Claude Opus 5.5, `bmad-dev-story`, en autonomie). T0 relevé sur
  `f2c5e419` avant tout code (Debug Log : numéros de ligne, helper de la 15-6b conforme,
  `set_account_not_postable` déjà présent dans les quatre binaires, code d'avant mesuré au rouge —
  `DATABASE_ERROR` / `ACCOUNT_NOT_FOUND` —, écarts ventilés). Choix **C-15-6d-1** (test 11 sur la
  prop reçue par la doublure de la modale, la 15-5c ayant remplacé la modale par une doublure).
  Rouge d'abord : les 7 tests des gardes neuves (1 à 7) rouges sur le code d'avant, les témoins 8
  et 9 verts avant et après ; 5 des 7 cas Vitest rouges avant (les cas `null` et le test 12, qui
  figent un comportement existant, verts). 10 mutations backend et 4 frontend, toutes rouges
  (Completion Notes). Rebasée sur `803f3e15` (15-6c, PR #586) après le commit de la documentation :
  registre, sprint-status et CHANGELOG par union, PDF régénéré.
  **Gates, sur l'état rebasé, au dernier commit de code (`04510336`, inchangé par les commits de
  documentation qui suivent)** :
  - backend complet (`scripts/test-fast.sh --ci` : fmt, clippy `-D warnings`, nextest, base
    `kesh_156d` remise à zéro avant) : **3082 / 3082**, 4 ignorés (= 3073 de `main` + 9) ;
  - frontend complet (`check` 0 erreur / 27 avertissements préexistants, `lint-i18n-ownership`,
    `test:unit`, `build`) : **Vitest 1156 / 1156** (= 1149 de `main` + 7) ;
  - E2E complet (port 3016, base `kesh_e2e_156d`) : **246 passés, 19 ignorés, 8 échoués** — les
    7 KF-029 attendus (`mode-expert:26`, `:41`, `onboarding-path-b:65`, `:92`, `onboarding:57`,
    `:77`, `:150`) et `sidebar-navigation:75` (KF-052, pollution connue), **rejoué seul : vert**.
    Les 10 specs `reconciliation*` vertes.
  - tmpfs MariaDB : 1,3 Go / 4,0 Go (31 %) avant les gates, 2,0 Go (50 %) après — le relevé couvre
    toutes les bases du conteneur, d'autres agents compris.
  Décompte : 10 AC satisfaits, 6 tâches cochées, **12 tests nommés** (9 backend, 3 fichiers Vitest
  — 7 cas). Statut **review** ; revue de code non lancée.

