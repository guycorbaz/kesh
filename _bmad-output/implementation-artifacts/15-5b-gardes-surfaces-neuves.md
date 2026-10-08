# Story 15.5b : Gardes de postabilité côté serveur — rapprochement, règles, réglages de facturation, compte bancaire

Status: ready-for-dev

<!-- Issue de la story 15-5, DÉCOUPÉE le 2026-10-08 après la passe de validation P1 (choix C7 de
     `epic-15-choix-autonomes.md`). Sous-story de « rollout » : elle applique aux surfaces neuves la
     variante `DbError::AccountsNotPostable` posée par la 15-5a. Choix applicables : C4, C5, C6, C7,
     C8, C9, C10, C11, C12, C13, C14. Validation P2 : à lancer. -->

**Issues** : **ferme #427** (P1), **#429** (P1), **#492** (codes bruts des refus par lot du
rapprochement, P3 — choix C8) et **#519** (manuel des règles d'affectation — choix C11). La PR porte
`closes #427 closes #429 closes #492 closes #519` (mots-clés **dans la PR**, le dépôt merge en squash).
Voisin repris en partie : **#474** — seule la *postabilité* du compte comptable d'un compte bancaire
est traitée ici ; le reste de #474 appartient à la **15-6** (choix C5).

**Dépend de la 15-5a** (variante `DbError::AccountsNotPostable`, code `ACCOUNT_NOT_POSTABLE`, clé
`error-account-not-postable`, `details.accountNumbers`). Ne pas commencer avant son merge.

## Story

En tant que **comptable ou administrateur d'une société tenue dans Kesh**,
je veux que **le serveur refuse un compte non imputable partout où le client désigne un compte qui
recevra une écriture** — rapprochement, règles de rapprochement, réglages de facturation, compte
comptable d'un compte bancaire —,
afin qu'**aucune écriture ne puisse atterrir sur un compte de regroupement ou de clôture**, quel que
soit le client qui appelle (écran, clé d'API, intégration), et que le refus soit lisible à l'écran.

### Pourquoi maintenant

La doctrine de la Story 24-5 — *une garde serveur ne se déduit pas d'un filtre d'écran* — a été
appliquée à trois chemins. Elle laisse **ouverts** ceux de #427 et #429, que le manuel avoue en toutes
lettres (`docs/manual/fr/user-manual.tex:380` et `:390`). Les écrans filtrent ; ces routes sont
atteignables par clé d'API, et rien ne contrôle le compte côté serveur. Un rapprochement routé sur
**9000 Bilan d'ouverture** fausse le résultat de l'exercice et le report à nouveau (#427).

### Une story de rollout

Elle applique un patron posé ailleurs (15-5a pour la variante ; `resolve_designated_account` et
14-3b pour la garde et l'exemption) à des surfaces nombreuses. Elle touche donc, **par nature**, plus de
cinq modules : `kesh-db` (dépôts `bank_accounts`, `reconciliation_rules`), quatre modules de routes
`kesh-api` (`reconciliation`, `reconciliation_rules`, `company_invoice_settings`, `bank_accounts`, plus
un doc-comment de `products`), `kesh-i18n`, deux modules `frontend` (`reconciliation`, `bank-accounts`)
et le manuel. C'est la forme que la § *Règle de splitting préventif* prescrit pour la seconde moitié
d'un découpage ; elle prévoit que **la sous-story de rollout est revue fichier par fichier** — c'est le
mode de revue demandé ici (choix C7).

## Acceptance Criteria

### Rapprochement (#427)

1. **AC1 — `POST /reconciliation/manual`** (`post_manual`, `crates/kesh-api/src/routes/reconciliation.rs:2942`) :
   après le contrôle existant du compte de contrepartie (`Some(a) if a.active => a`, `:3028`), un compte
   **non imputable** → `AppError::Database(DbError::accounts_not_postable([a.number]))`, soit **400
   `ACCOUNT_NOT_POSTABLE`** ; **rien n'est écrit** (aucune écriture, transaction bancaire toujours
   `pending`, aucun audit). Inconnu / autre société / archivé → **404 `ACCOUNT_NOT_FOUND`**, inchangé
   (anti-énumération) et **prioritaire**.
2. **AC2 — `POST /reconciliation/split`** (`post_split`, `:3362`) : à l'étape de contrôle des comptes de
   contrepartie (`:3488`), les non imputables sont collectés **après** les manquants ; s'il y a des
   manquants → 404 `ACCOUNT_NOT_FOUND` **inchangé** (prioritaire) ; sinon, s'il y a des non imputables →
   400 `ACCOUNT_NOT_POSTABLE` nommant **tous** ceux de la requête ; rien n'est écrit.
3. **AC3 — `POST /reconciliation/accept`, proposition `split`** (`accept_one_split`, `:1829`, étape d
   `:1952-1981`) : le `SELECT` lit `active, postable, number` ; inconnu / archivé → `failed[]`
   `ACCOUNT_NOT_FOUND` **inchangé** (prioritaire) ; sinon, une ligne non imputable → `failed[]` avec
   `errorCode = "ACCOUNT_NOT_POSTABLE"` et `details = { "accountNumbers": [...] }` (triés,
   dédupliqués) ; **HTTP 200**, les autres propositions du lot sont traitées normalement (§ *Pattern
   batch — FailedProposal* du `CLAUDE.md`).
4. **AC4 — `POST /reconciliation/accept`, proposition `rule`** (`accept_one_rule`, `:2207`, étape 5
   `:2289-2318`) : le `SELECT` lit `active, postable, name, number` ; archivé → `ACCOUNT_NOT_FOUND`
   inchangé ; non imputable → `failed[]` `ACCOUNT_NOT_POSTABLE`, `details = { "accountNumbers": [n°] }`.
   **Pas d'exemption** (choix C4 : une règle dont le compte a été scindé en sous-comptes est périmée).
5. **AC5 — Une règle périmée n'est plus proposée.** `get_proposals` (`:449`) : la requête
   `accounts_info_rows` (`:545-551`) lit aussi `postable`, et l'ensemble `active_account_ids` (`:556`)
   passé à `kesh_reconciliation::first_matching_rule` ne retient que les comptes **actifs ET
   imputables** (`accounts_info`, qui sert à l'affichage, garde tous les actifs). La règle suivante qui
   correspond, s'il y en a une, est proposée à sa place. Le nom du paramètre de `first_matching_rule`
   (`crates/kesh-reconciliation/src/rules.rs:85-95`) n'est **pas** changé ; un commentaire au site
   d'appel dit ce que l'ensemble contient.
6. **AC6 — `accept_one_invoice` reste tel quel, et c'est écrit** : ses comptes ne viennent pas du client
   (compte bancaire configuré ; créance lue **sur l'écriture de vente**, `:1413-1450` ; compte d'arrondi
   des réglages, déjà gardé `postable = TRUE` dans
   `crates/kesh-db/src/repositories/company_invoice_settings.rs:469` et `:499`). Un commentaire au site
   le dit, pour que la prochaine énumération ne le reclasse pas « oublié ».

### Règles de rapprochement (#427)

7. **AC7 — Création** (`post_create`, `crates/kesh-api/src/routes/reconciliation_rules.rs:248`) :
   `validate_counterparty_account` (`:200`) rend désormais le compte lu ; `post_create` refuse un compte
   **non imputable** → 400 `ACCOUNT_NOT_POSTABLE`, aucune règle créée. Ordre : validations de forme →
   404 compte (inchangé) → 400 non imputable.
8. **AC8 — Modification** (`patch`, `:345`) — le contrôle de postabilité se fait **dans la
   transaction**, dans `reconciliation_rules::update_in_tx`
   (`crates/kesh-db/src/repositories/reconciliation_rules.rs:254`), sur la ligne `before` qu'elle lit déjà
   (`:262-264`), **juste après** la validation du projet par défaut (`:271-276`, même patron « seulement
   si ça change »). Le compte **cible** est `patch.counterparty_account_id.unwrap_or(before.counterparty_account_id)`.
   Il est contrôlé sur `postable` (lecture `SELECT number, postable FROM accounts WHERE id = ? AND
   company_id = ?` dans la transaction) **si et seulement si** :
   - (a) le PATCH **change** le compte (`Some(id)` avec `id != before.counterparty_account_id`), **ou**
   - (b) le PATCH **réactive** la règle (`patch.active == Some(true)` et `before.active == false`) —
     **même si le compte est inchangé** (choix C9 : réactiver une règle dont le compte n'est plus
     imputable ressusciterait une règle qui ne s'appliquera jamais, ou pire).
   Non imputable → `DbError::accounts_not_postable([n°])` → 400 `ACCOUNT_NOT_POSTABLE`. **Exemption**
   (choix C4) : hors (a) et (b), le compte n'est pas re-contrôlé — on peut renommer, reprioriser ou
   **désactiver** une règle dont le compte est devenu non imputable, y compris quand le formulaire
   renvoie le même `counterpartyAccountId` (`RuleFormModal.svelte:84-90` le renvoie toujours). Le
   contrôle d'existence / société / `active` du handler (`:366-368`) est **inchangé**. **Ordre des
   erreurs** (choix C10), écrit dans le doc-comment de `patch` : validations de forme → 404 compte
   (handler) → 404 règle (handler, `:372-374`) → 404 règle (`update_in_tx`) → refus du projet → **400
   `ACCOUNT_NOT_POSTABLE`** → 409 conflit de version (au `UPDATE`) / 409 doublon.
9. **AC9 — Les règles existantes sur un compte non imputable restent en base, sans migration.** Elles
   ne sont plus proposées (AC5) et leur acceptation est refusée (AC4) ; une règle **active** sur un tel
   compte reste listée « Active » dans l'écran des règles et s'édite (AC8, exemption) ; sa
   **réactivation** après désactivation est refusée (AC8 b). Le manuel le dit (AC17). Aucune donnée
   n'est réécrite.

### Réglages de facturation (#429)

10. **AC10 — Les six champs historiques** de `PUT /company/invoice-settings`
    (`update_invoice_settings`, `crates/kesh-api/src/routes/company_invoice_settings.rs:246`, appels
    `:276-325`) — `defaultReceivableAccountId`, `defaultRevenueAccountId`, `defaultVatPayableAccountId`,
    `defaultVatRecoverableAccountId`, `defaultVatDecompteAccountId`, `defaultPayableAccountId` —
    refusent un compte **non imputable** quand la valeur **change** par rapport au réglage en place
    (`current`, lu `:256`) : 400 `VALIDATION_ERROR`, message
    « {champ} : compte non imputable (compte de regroupement ou de clôture) » — **le message déjà émis**
    par `validate_account_of` (`:190-194`) pour les comptes désignés (choix C3 : les refus de cette route
    sont tous des `Validation` nommant le champ). Mise en œuvre : chaque appel passe à
    `validate_account_of(…, require_postable = (req.champ != current.champ), …)`, ou `validate_account`
    reçoit ce paramètre.
11. **AC11 — Exemption « inchangé »** : une valeur **égale** à celle en place n'est pas re-contrôlée sur
    `postable` — patron de `resolve_designated_account` (`:200-229`). Sans elle, un compte de réglage
    devenu non imputable après coup (ajout d'un sous-compte, règle 14-3a) bloquerait **tout**
    enregistrement des réglages, le formulaire renvoyant toutes les valeurs (`withCurrentAccount`,
    `frontend/src/routes/(app)/settings/invoicing/+page.svelte:85-111`, #271). Les contrôles existants
    (existence, société, `active`, type) restent **inconditionnels**.

### Compte comptable d'un compte bancaire (voisin de #474)

12. **AC12 — Les trois routes du compte bancaire** (`crates/kesh-api/src/routes/bank_accounts.rs`) :
    - **Création** (`create_bank_account`, `fn` `:384`, appel de `validate_journal_account_id` `:393`) :
      `validate_journal_account_id` (`fn` `:260`) reçoit un paramètre `require_postable: bool`, vrai ici ;
      non imputable → 400 `ACCOUNT_NOT_POSTABLE` après les contrôles existants (404, type).
    - **Remplacement** (`update_bank_account`, `fn` `:498`, appel `:512`) et **lien**
      (`patch_bank_account_journal_link`, `fn` `:717`, appel `:727`) : l'appel garde
      `require_postable = false` — le contrôle de postabilité se fait **dans la transaction**, dans le
      dépôt, sous le verrou `FOR UPDATE` que `update_for_company`
      (`crates/kesh-db/src/repositories/bank_accounts.rs:356`) et `set_journal_account_id_for_company`
      (`:265`, `SELECT … FOR UPDATE` `:277-286`) posent déjà sur la ligne, et sur la valeur en place
      qu'ils lisent (`existing.journal_account_id`) (choix C10). Il s'applique **seulement si la valeur
      change** et n'est pas `NULL` : dans `set_journal_account_id_for_company`, **après** le court-circuit
      « no-op » (`:306`) ; dans `update_for_company`, après le contrôle de version (`:380-382`), sous la
      condition `new_journal_account_id != existing.journal_account_id`. Non imputable →
      `DbError::accounts_not_postable([n°])` → 400 (les handlers propagent déjà `Err(e) =>
      AppError::Database(e)`, `:557` et `:751`).
    - **Ordre des erreurs conservé**, écrit dans les doc-comments : forme → 404 / 400 type du compte
      (`validate_journal_account_id`, hors transaction, inchangé) → 404 compte bancaire → 409 version →
      **400 `ACCOUNT_NOT_POSTABLE`**. Un PUT qui promeut le compte en principal et échoue sur ce refus
      **n'a rien démoté** (la transaction est abandonnée, `flip_primary_off_for_company` `:528-534`
      compris).
    - Le compte débiteurs, lui, reste accepté ici : c'est la 15-6 (choix C5).
13. **AC13 — L'écran garde le compte lié visible.** Dans `frontend/src/routes/(app)/bank-accounts/+page.svelte`,
    les deux `<select>` du compte lié (`:321-330`, `:472-481`) reçoivent leurs options de
    `withCurrentAccount(linkableAccounts, formJournalAccountId, accounts)`
    (`frontend/src/lib/features/accounts/account-options.ts:58`), comme
    `BankAccountJournalLinkForm.svelte:60` (choix C12). **Fait vérifié au code** (Svelte 5.55,
    `node_modules/svelte/src/internal/client/dom/elements/bindings/select.js`) : quand la valeur liée
    n'est dans aucune option, `select_option` pose `selectedIndex = -1` — le champ s'affiche **vide** —
    mais **ne réécrit pas** la variable (« the model should be preserved unless explicitly changed ») ;
    le PUT renvoie donc l'identifiant en place et l'exemption de l'AC12 joue. Le défaut est d'affichage :
    l'utilisateur voit un champ vide pour un lien qui existe, et peut l'effacer sans le savoir — c'est
    exactement le cas de #271.

### Les refus, lisibles (#492)

14. **AC14 — Un libellé traduit par `errorCode`** pour **tous** les codes que `failed[]` peut porter,
    affichés par `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:362`
    (aujourd'hui `TX #{f.bankTransactionId} — {f.errorCode}`, choix C8). Nouveau module
    `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, sur le patron de
    `failedItemLabel` (`frontend/src/lib/features/payment-batches/payment-batch-helpers.ts:53-80`) :
    `switch` sur le code, **clés écrites en toutes lettres** (jamais construites par gabarit — elles
    doivent être vues par `i18n-keys.test.ts`), repli **avec le code brut** pour un code inconnu. Pour
    `ACCOUNT_NOT_POSTABLE`, le libellé nomme les comptes (`details.accountNumbers`). Les codes, relevés
    sur `1920381e` par `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u`
    (25), plus `ACCOUNT_NOT_POSTABLE` : `ACCOUNT_NOT_FOUND`, `ACCOUNT_NOT_POSTABLE`,
    `BANK_ACCOUNT_NOT_CONFIGURED`, `BANK_ACCOUNT_NOT_FOUND`, `BANK_TRANSACTION_NOT_FOUND`,
    `DATABASE_ERROR`, `FISCAL_YEAR_INVALID`, `INTERNAL_ERROR`, `INVOICE_NOT_FOUND`,
    `INVOICE_SALE_ENTRY_MALFORMED`, `PERIOD_LOCKED`, `PROJECT_ARCHIVED`, `PROJECT_NOT_FOUND`,
    `RECONCILIATION_ALREADY_RECONCILED`, `RECONCILIATION_CURRENCY_MISMATCH`,
    `RECONCILIATION_FISCAL_YEAR_CLOSED`, `RECONCILIATION_INVOICE_NOT_ELIGIBLE`,
    `RECONCILIATION_OVERPAYMENT`, `RECONCILIATION_RULE_MISMATCH`, `RECONCILIATION_RULE_NO_LONGER_MATCHES`,
    `RECONCILIATION_RULE_NOT_FOUND`, `RECONCILIATION_SCORE_TOO_LOW`, `RECONCILIATION_SPLIT_IMBALANCE`,
    `RECONCILIATION_TRANSACTION_NOT_PENDING`, `ROUNDING_ACCOUNT_NOT_CONFIGURED`, `VALIDATION_ERROR`
    (**26**). Le dev **refait** la commande et ajoute tout code apparu entre-temps ; un test vérifie que
    chaque code de la liste a un libellé distinct du repli. Les libellés, dans les **quatre** locales,
    sont **relevés** sur les messages existants quand il y en a (p. ex. `ROUNDING_ACCOUNT_NOT_CONFIGURED`
    reprend `error-rounding-account-not-configured`, qui dit où agir) — pas inventés. La ligne affiche
    `TX #<id> — <libellé>` ; le code brut reste lisible dans un `title` ou entre parenthèses, pour le
    support.

    **Et les refus de l'écran des règles s'affichent, au lieu de `[object Object]`** (choix C14). Le
    client d'API lève un `ApiError` **objet simple**, pas une instance d'`Error`
    (`frontend/src/lib/shared/types/api.ts:9-14`, `api-client.ts:208-233`) ; or
    `RuleFormModal.svelte:104`, `RulesList.svelte:64` et `:87` font
    `e instanceof Error ? e.message : String(e)` — le refus 400 de l'AC7 et de l'AC8 s'y afficherait
    « [object Object] ». Ces trois sites, ainsi que `ReconciliationProposals.svelte:68`, `:160`, `:178` et
    `frontend/src/routes/(app)/reconciliation/rules/+page.svelte:32` (même motif, même module), passent au
    patron de `ManualMatchModal.svelte:126-133` : `isApiError(e) ? e.message : (e instanceof Error ?
    e.message : String(e))`. Un test Vitest par composant le vérifie (un `ApiError` rejeté → son
    `message` affiché).

### Ce qui ne change pas, et qui doit être prouvé

15. **AC15 — Non-régression des flux automatiques** : `create_in_tx(…, false)` accepte toujours un compte
    de **configuration** devenu non imputable (`test_create_in_tx_auto_flow_allows_non_postable`,
    `crates/kesh-db/src/repositories/journal_entries.rs:3687`) ; la contre-passation (`reverse_in_tx_inner`,
    `fn` `:1518`, appel de `create_in_tx_inner` `:1656`) et l'annulation d'un rapprochement restent
    possibles sur un compte devenu non imputable (`reverse_succeeds_when_an_account_became_non_postable`,
    `crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1015`). Un compte bancaire dont le compte
    comptable est **devenu** non imputable continue de servir aux rapprochements (décision D-A0 de la
    14-3b, choix C6) : les contrôles `active` du compte de banque (`reconciliation.rs:1914-1938`, `:3011`,
    `:3454`) ne reçoivent **pas** de clause `postable`.
16. **AC16 — Tests négatifs « compte non imputable refusé » sur CHAQUE surface** des AC 1 à 12, dont le
    compte de test ne diffère d'un compte accepté **que par `postable`** (même société, actif, bon type)
    et dont l'assertion porte sur le **code** (`ACCOUNT_NOT_POSTABLE`, ou le message du champ pour les
    réglages) **et** `details.accountNumbers` là où il existe — pas sur le seul statut. Chaque surface à
    exemption porte **aussi** son test positif « inchangé → accepté ». Chaque surface à ordre écrit
    (AC2, AC3, AC8, AC12) porte un test de **priorité**. Chaque test négatif est **muté** une fois
    (garde retirée → le test rougit), consigné au Dev Agent Record.
17. **AC17 — Le manuel dit vrai** (`docs/manual/fr/user-manual.tex`) :
    - la réserve des deux encadrés **`:380`** (§ *Rôles des comptes*) et **`:390`** (§ *Les comptes de
      clôture*) est **levée** : plus d'« exception », plus de « une intégration … pourrait les viser » ;
      ils disent que le rapprochement (manuel, ventilé, proposé, et ses règles), les réglages de
      facturation et le compte comptable d'un compte bancaire refusent un compte non imputable ;
    - § *Acceptation par lot* (**`:1530-1532`**) : le texte actuel dit l'opération « atomique : soit toutes
      … soit aucune » et renvoie à « la doc CLAUDE.md projet » — **faux et hors de propos pour un
      utilisateur** (succès partiel, refus listés). Le réécrire : les propositions acceptées le sont, les
      refusées sont listées avec leur motif en clair (AC14) ;
    - § *Règles d'affectation automatique* (**`:1561-1586`**) : **réécrite sur le comportement réel**
      (choix C11, #519) — accès *Rapprochement* → *Règles d'affectation* (`/reconciliation/rules`,
      `frontend/src/routes/(app)/+layout.svelte:123`), une règle = libellé, **type de correspondance**
      (`counterparty_contains`, `counterparty_exact`, `reference_contains`, `iban_exact`,
      `crates/kesh-db/src/entities/reconciliation_rule.rs`) et valeur, **compte de contrepartie**
      (charge ou produit, **imputable**), priorité, projet par défaut ; elle produit une **proposition**
      dans l'écran de rapprochement, que l'utilisateur accepte — aucune écriture n'est créée à l'import,
      il n'existe ni brouillon ni option *auto-validate rules* ; désactiver / réactiver / archiver ; et
      le cas de l'AC9 (une règle dont le compte est devenu non imputable n'est plus proposée, se modifie,
      mais ne se réactive pas sans changer de compte). Le dev décrit **ce qu'il voit** dans
      `RuleFormModal.svelte`, `RulesList.svelte` et `get_proposals`, pas ce que dit cette liste ;
    - `docs/manual/fr/admin-manual.tex` : contrôle **sans objet** à ce jour (aucun passage n'énumère les
      critères des comptes par défaut historiques ni du compte lié) — le refaire et l'écrire au Dev Agent
      Record ;
    - le PDF est régénéré (`latexmk -xelatex` dans `docs/manual/fr/`), commité, et **contrôlé aplati**
      (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) : les phrases levées sont absentes, les nouvelles
      présentes.
18. **AC18 — Les doc-comments et le CHANGELOG suivent.**
    - `crates/kesh-api/src/routes/products.rs:303-308` : la décision D3 justifie l'exclusion de `postable`
      par « le code jumeau `company_invoice_settings::validate_account` ne le contrôle pas davantage » —
      **devenu faux**. Réécrire la justification (la garde vit sur la ligne de facture, `NotPostable` ;
      l'article ne poste rien ; exemption « inchangé » de D4) sans changer le comportement (choix C6).
      Idem pour le commentaire de `crates/kesh-api/src/errors.rs:~1196-1205` s'il reprend l'argument.
    - `company_invoice_settings.rs:150-158` (« Les champs historiques ne l'exigent pas ») et l'en-tête du
      module ; `reconciliation_rules.rs:13-17` (doc de module : le pré-vol ne contrôle que l'existence
      et `active`) ; doc d'ordre de validation de `post_manual` (`reconciliation.rs:2932-2939`) et de
      `post_split` ; doc-comments de `validate_journal_account_id`, `update_for_company`,
      `set_journal_account_id_for_company`, `update_in_tx`.
    - `crates/kesh-db/src/repositories/invoice_settlements_write.rs:153-155` (« Les trois flux de
      réconciliation, eux, restent ouverts et sont suivis par #427 ») et `journal_entries.rs:70-77`,
      `:182-186` (liste des flux `false`) : dire que les flux de rapprochement gardent désormais le compte
      **client** en amont.
    - `CHANGELOG.md`, section `## [0.13.0] — Non publié`, rubrique **Corrigé** : refus serveur des comptes
      non imputables sur le rapprochement (manuel, ventilé, propositions par règle et ventilées), les
      règles de rapprochement (création, changement de compte, réactivation), les réglages de
      facturation et le compte comptable d'un compte bancaire ; une règle dont le compte n'est plus
      imputable n'est plus proposée ; les refus d'une acceptation par lot s'affichent en clair (#492).
      Même rubrique (le CHANGELOG n'a pas de rubrique *Documentation*) : le manuel des règles
      d'affectation décrivait un écran inexistant (#519).

## Inventaire des sites — l'ensemble clos des points où un compte reçoit une écriture

Règle *« Inventorier les sites NON RÉSOLUS »* du `CLAUDE.md`. Faite au sol le 2026-10-08 sur `1920381e` ;
le dev la **refait** (T0) et la passe de revue la refait à son tour.

### (a) Tous les appels de `journal_entries::create_in_tx` / `create_in_tx_inner` hors tests

```sh
grep -rnE "journal_entries::create_in_tx\(|create_in_tx_inner\(|[^:_]create_in_tx\(" crates/*/src --include=*.rs \
  | grep -v "fn create_in_tx"
```

Sur `1920381e`, la commande rend **20 lignes** : **trois homonymes sans rapport** — `users.rs:32`,
`supplier_invoices.rs:222`, `imported_supplier_invoices.rs:34` (des `create_in_tx` d'autres entités),
à écarter — et **17 lignes** classées ci-dessous : 15 sites, le délégateur `journal_entries.rs:194` et
l'appel de `mod tests` `journal_entries.rs:3700`.

| # | site | 5ᵉ arg. | d'où vient le compte | avant | après 15-5a + 15-5b |
|---|---|---|---|---|---|
| 1 | `reconciliation.rs:1581` `accept_one_invoice` | `false` | banque (config), créance (écriture de vente), arrondi (réglage gardé) | aucun compte client | **inchangé**, commenté (AC6) |
| 2 | `reconciliation.rs:2083` `accept_one_split` | `false` | **client** | **NON GARDÉ** (`active` seul, `:1952-1981`) | gardé (AC3) |
| 3 | `reconciliation.rs:2432` `accept_one_rule` | `false` | règle | **NON GARDÉ** (`active` seul, `:2289-2318`) | gardé (AC4, AC5) |
| 4 | `reconciliation.rs:3153` `post_manual` | `false` | **client** | **NON GARDÉ** (`:3028`) | gardé (AC1) |
| 5 | `reconciliation.rs:3619` `post_split` | `false` | **client** | **NON GARDÉ** (`:3488`) | gardé (AC2) |
| 6 | `invoice_settlements_write.rs:214` règlement client | `false` | client (compte interne) / banque (config) | gardé 24-5 | nom juste (15-5a) ; banque : AC12 |
| 7 | `invoice_settlements_write.rs:497` solde du reste | `false` | réglages désignés | gardé (`kesh-db/src/repositories/company_invoice_settings.rs:353`, `:469`, `:499`, `postable = TRUE`) | inchangé |
| 8 | `supplier_invoices.rs:372` facture fournisseur | `false` | client (charge) + réglages (créanciers, TVA récup.) | charge gardée 24-5 ; réglages **NON GARDÉS** (#429) | 15-5a + AC10 |
| 9 | `supplier_invoices.rs:693` règlement fournisseur | `false` | client (compte interne) / banque | gardé 24-5 | 15-5a ; banque : AC12 |
| 10 | `invoices.rs:2255` validation de facture | `false` | lignes (gardées) + réglages (créance, TVA due, produit par défaut) | **réglages NON GARDÉS** (#429) | AC10 |
| 11 | `credit_notes.rs:548` avoir | `false` | snapshot de la facture (D5-bis) | validés à l'émission | inchangé (angle mort assumé, C6) |
| 12 | `journal_entries.rs:1656` contre-passation (appel dans `reverse_in_tx_inner`, `fn` `:1518`) | `false` | écriture d'origine | exemption voulue | inchangé (AC15) |
| 13 | `journal_entries.rs:153` `create` (saisie manuelle) | **`true`** | client | gardé 14-3b | nom juste (15-5a) |
| 14 | `journal_entries.rs:634` `create_opening_entry` | **`true`** | client | gardé | nom juste (15-5a) |
| 15 | `opening_complement.rs:641` complément | **`true`** | client + report | gardé en amont (`check_lines`) | inchangé |
| — | `journal_entries.rs:194` | — | délégation `create_in_tx` → `create_in_tx_inner` | — | — |
| — | `journal_entries.rs:3700` | `false` | — | `mod tests` | — |

### (b) Toutes les routes qui reçoivent un identifiant de compte du client

Commande : `grep -rnE "pub [a-z_]*account_id" crates/kesh-api/src/routes/*.rs` (DTO désérialisés ; 63
lignes sur `1920381e`, dont des champs de réponse — trier).

| route / champ | écrit une écriture ? | verdict |
|---|---|---|
| `journal_entries.rs` `lines[].accountId` | oui (#13) | gardé ; nom juste (15-5a) |
| `opening_balances.rs` `accountId` | oui (#14, #15) | gardé ; nom juste (15-5a) |
| `reconciliation.rs` manual / split / accept | oui (#2–#5) | **AC1–AC5** |
| `reconciliation_rules.rs` `counterpartyAccountId` (POST, PATCH) + `active` (PATCH) | indirectement (#3) | **AC7–AC9** |
| `company_invoice_settings.rs` six champs historiques | indirectement (#8, #10) | **AC10–AC11** |
| `company_invoice_settings.rs` arrondi, escompte, frais, pertes | indirectement (#7) | déjà gardés (`resolve_designated_account`, « si changé ») |
| `bank_accounts.rs` `journalAccountId` (POST, PUT, PATCH) | indirectement (#1, #6, #9, tous les rapprochements) | **AC12–AC13** |
| `invoices.rs` `lines[].revenueAccountId` | oui (#10) | gardé (16-1a) |
| `invoices.rs` règlement `accountId` / `bankAccountId` | oui (#6) | gardé 24-5 |
| `supplier_invoices.rs` `expenseAccountId`, règlement `accountId` | oui (#8, #9) | gardé 24-5 |
| `imported_supplier_invoices.rs` `expenseAccountId` | oui, via `supplier_invoices::create_in_tx` (#8) | gardé en aval |
| `products.rs` `defaultRevenueAccountId` | non (recopié sur la ligne de facture, gardée) | **hors périmètre, délibéré (D3, C6)** — doc-comment amendé (AC18) |
| `credit_notes.rs` `revenueAccountId` | non (champ de **réponse**) | sans objet |
| `payment_batches.rs`, `bank_imports.rs` `bankAccountId` | non (compte bancaire, pas compte du plan) | sans objet |
| `reports.rs` `accountIds` | non (filtre de lecture) | sans objet |
| `accounts.rs` (rôle d'un compte) → `insert_with_defaults_in_tx` | indirectement, une fois, à la création des réglages | **angle mort assumé (C6)** ; tout changement ultérieur passe par le PUT des réglages (AC10) |

## Tasks / Subtasks

- [ ] **T0 — Refaire les deux inventaires** sur `HEAD` (lignes à jour) et la liste des codes de l'AC14 ;
      signaler au Change Log tout site ou code absent de ces listes. Un site neuf non classé bloque la
      story.
- [ ] **T1 — Rapprochement** (AC1–AC6)
  - [ ] `post_manual` : refus après `Some(a) if a.active` ; doc d'ordre `:2932-2939`.
  - [ ] `post_split` : collecte des non imputables après les manquants ; un seul refus.
  - [ ] `accept_one_split` étape d : `SELECT active, postable, number` ; `FailedProposal` avec
        `details.accountNumbers`.
  - [ ] `accept_one_rule` étape 5 : `SELECT active, postable, name, number` ; même `FailedProposal`.
  - [ ] `get_proposals` : `postable` dans `accounts_info_rows`, filtre de `active_account_ids`,
        commentaire au site d'appel.
  - [ ] Commentaire de classement à `accept_one_invoice` (AC6).
- [ ] **T2 — Règles** (AC7–AC9)
  - [ ] `validate_counterparty_account` rend `Account` ; `post_create` refuse `!postable`.
  - [ ] `update_in_tx` (`kesh-db`) : contrôle (a)/(b) après la validation du projet ; doc-comment
        (« # Erreurs ») et doc d'ordre de `patch`.
  - [ ] Doc de module `reconciliation_rules.rs:9-17`.
- [ ] **T3 — Réglages de facturation** (AC10, AC11) : `require_postable = (req.champ != current.champ)`
      sur les six appels ; doc-comments `:150-158` et en-tête.
- [ ] **T4 — Compte bancaire** (AC12, AC13)
  - [ ] `validate_journal_account_id(…, require_postable)` ; `true` à la création.
  - [ ] `update_for_company` et `set_journal_account_id_for_company` : contrôle « si changé » sous le
        verrou ; doc-comments, ordre des erreurs.
  - [ ] `bank-accounts/+page.svelte` : `withCurrentAccount` sur les deux `<select>` ; test Vitest :
        un compte lié devenu non imputable reste affiché et sélectionné à l'ouverture du formulaire.
- [ ] **T5 — Les refus par lot** (AC14)
  - [ ] `failed-proposal-label.ts` + test unitaire (chaque code de la liste → libellé ≠ repli ;
        `ACCOUNT_NOT_POSTABLE` nomme les numéros ; code inconnu → repli avec le code).
  - [ ] Clés dans les **quatre** `messages.ftl` (préfixe `reconciliation-failed-`), `lint-i18n-ownership`
        et `i18n-keys.test.ts` verts.
  - [ ] `ReconciliationProposals.svelte:362` : afficher le libellé ; mettre à jour
        `ReconciliationProposals.test.ts`.
  - [ ] Second volet (C14) : les sept `catch` nommés à l'AC14 passent au patron `isApiError` ; un test
        Vitest par composant (`RuleFormModal.test.ts`, `RulesList` — test neuf —, `ReconciliationProposals.test.ts`).
- [ ] **T6 — Tests backend** (AC15, AC16) — helper `set_account_not_postable` sur le patron de
      `crates/kesh-api/tests/products_revenue_account_e2e.rs:254` (UPDATE direct, `version + 1`).
  - [ ] `reconciliation_manual_e2e.rs` : non imputable → 400 `ACCOUNT_NOT_POSTABLE` +
        `details.accountNumbers`, aucune écriture, transaction toujours `pending`, aucun audit ; archivé
        → 404 (priorité).
  - [ ] `reconciliation_split_e2e.rs` : une ligne non imputable sur trois → 400, numéros ; une ligne
        manquante **et** une non imputable → 404 (priorité) ; rien d'écrit.
  - [ ] `reconciliation_e2e.rs` (accept) : proposition split non imputable → 200, `failed[]` avec le code
        et `details.accountNumbers` ; proposition valide du même lot → `accepted[]` ; proposition rule
        dont le compte est devenu non imputable → `failed[]` ; `get_proposals` ne propose plus cette règle
        et propose la suivante qui correspond.
  - [ ] `reconciliation_rules_e2e.rs` : POST non imputable → 400 ; PATCH vers un autre compte non
        imputable → 400 ; PATCH renvoyant le **même** compte devenu non imputable (règle active) → 200 ;
        PATCH `active:false` sur une telle règle → 200 ; PATCH `active:true` sur une telle règle
        désactivée → 400 `ACCOUNT_NOT_POSTABLE` ; PATCH `active:true` **avec** un nouveau compte imputable
        → 200 ; PATCH sans compte → 200 ; priorité : compte archivé → 404 avant tout.
  - [ ] Réglages — fichier neuf `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`
        (montage : `idor_multi_tenant_e2e.rs:~751`, `create_seeded_company`) : pour **chacun** des six
        champs, changement vers un compte non imputable du bon type → 400 et message du champ ; PUT
        renvoyant un compte **déjà en place** devenu non imputable → 200.
  - [ ] `bank_accounts_e2e.rs` : POST, PUT, PATCH vers non imputable → 400 `ACCOUNT_NOT_POSTABLE` ;
        PUT et PATCH inchangés sur un compte devenu non imputable → 200 ; **ordre** : compte bancaire
        inconnu + compte non imputable → 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (code actuel,
        `bank_accounts_e2e.rs:525`) ; version périmée + non imputable → 409 ; PUT `isPrimary:true` refusé
        pour non-imputabilité → l'ancien principal l'est toujours.
  - [ ] `kesh-db` : `reconciliation_rules_repository.rs` et `bank_accounts_repository.rs` — la variante
        rendue par `update_in_tx`, `update_for_company`, `set_journal_account_id_for_company`.
  - [ ] AC15 : les tests de non-régression nommés passent toujours (ne pas les réécrire).
  - [ ] Mutation : retirer chaque garde une fois, constater le rouge, restaurer **et toucher le
        fichier**. Consigner la liste.
- [ ] **T7 — Manuel** (AC17) : les quatre passages, PDF régénéré, commité, contrôlé aplati.
- [ ] **T8 — Doc-comments, propagation, CHANGELOG** (AC18)
  - [ ] Les sites de l'AC18.
  - [ ] **Grep du symptôme** avant de déclarer fini (règle *Propagation post-patch*) :
        `grep -rnE "#427|#429|#492|ne vérifie pas cet indicateur|restent ouverts|ne l'exigent pas|auto-validate|Administration.*Règles d'affectation" crates docs/manual/fr/*.tex frontend/src`
        — chaque commentaire ou passage qui annonce ces trous comme ouverts est mis à jour.
- [ ] **T9 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) — **même en
      cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate frontend complet ;
      **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par fichier contre
      `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 18 AC, 10 tâches T0–T9.)*

## Dev Notes

### Le modèle à suivre, et ce qu'il n'est pas

- **Exemption « contrôlé seulement si la valeur change »** : `resolve_designated_account`
  (`company_invoice_settings.rs:209`) et le « grandfathering » du projet par défaut dans
  `reconciliation_rules::update_in_tx` (`:266-276`). C'est ce second modèle que suivent les gardes
  placées dans les dépôts (règle, compte bancaire) : contrôle **dans la transaction**, sur la ligne lue
  par le dépôt — ce qui conserve l'ordre des erreurs existant (choix C10) et évite une lecture
  périmée hors transaction (finding B2).
- **Ne PAS passer `enforce_postable = true`** à `create_in_tx` dans `post_manual` / `post_split` / `accept_*`
  pour « faire simple » : les lignes contiennent aussi le **compte bancaire**, compte de configuration
  que D-A0 autorise à être devenu non imputable. La garde se place sur le seul compte **client**, en
  amont.
- **Course résiduelle** : `post_manual` et `post_split` contrôlent le compte **hors** transaction (lecture
  sur le pool, comme le contrôle `active` actuel) ; la lecture de compte des dépôts `bank_accounts` et
  `reconciliation_rules` se fait dans la transaction mais sans verrou sur la ligne `accounts`. Un compte
  rendu non imputable entre le contrôle et l'insertion passe — même dette LOW acceptée qu'en 19-3/19-4 et
  que pour `active`. Les flux `accept_*` lisent dans la transaction, sous le verrou de compte bancaire.

### Ce qui doit être préservé

- Anti-énumération (KF-002) : inconnu / autre société / archivé restent en **404 `ACCOUNT_NOT_FOUND`**
  sur le rapprochement et les règles, et **prioritaires**. `ACCOUNT_NOT_POSTABLE` n'est émis que pour un
  compte de la société, actif.
- Le pattern batch : aucune erreur par proposition n'escalade en `AppError` dans `accept_one_*`.
- Les écrans filtrent déjà : `AccountAutocomplete.svelte:198-205`
  (`frontend/src/lib/features/journal-entries/`, `a.active && a.postable`) sert les deux modales de
  rapprochement ; `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte:43-50`,
  `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:44-60`,
  `frontend/src/routes/(app)/bank-accounts/+page.svelte:78-85` et
  `frontend/src/routes/(app)/settings/invoicing/+page.svelte` filtrent `postable`. Les modales manuelle
  et ventilée affichent `e.message` (`ManualMatchModal.svelte:126-132`) : le refus 400 s'y lit en clair
  sans changement. **Ce qui change à l'écran** : la liste des refus par lot (AC14), l'affichage des
  erreurs de l'écran des règles (AC14, second volet — sans lui, la bascule « Réactiver » de
  `RulesList.svelte:55-67` afficherait « [object Object] » pour le refus de l'AC8 b) et les `<select>`
  du compte bancaire (AC13).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C4** — exemption « inchangé » pour réglages, compte bancaire, PATCH de règle ; aucune pour les
  rapprochements ni pour l'acceptation par règle.
- **C5** — le compte d'un compte bancaire, pour sa seule postabilité ; le reste de #474 à la 15-6.
- **C6** — hors périmètre assumés : fiche article (D3), compte bancaire **à l'usage** (D-A0), rôles de
  comptes, avoir (D5-bis).
- **C7** — le découpage ; cette story est le rollout.
- **C8** — libellés traduits pour **tous** les codes de `failed[]` ; ferme #492.
- **C9** — une règle périmée n'est plus proposée ; sa réactivation est refusée ; pas de migration.
- **C10** — contrôle « inchangé » dans la transaction, sous le verrou ; ordre des erreurs conservé.
- **C11** — réécriture du § *Règles d'affectation automatique* ; ferme #519.
- **C12** — `withCurrentAccount` sur les `<select>` du compte bancaire.
- **C13** — clé `accountNumbers` commune ; ordre des causes.
- **C14** — les `catch` de l'écran des règles et de la liste des propositions lisent `ApiError`.

### Hors périmètre, et écrit

- La réactivation d'une règle dont le compte est **archivé** (pas seulement non imputable) : comportement
  actuel conservé (acceptée ; la règle n'est de toute façon pas proposée, AC5). Le contrôle neuf ne
  refuse que `postable = FALSE`.
- Le renommage du paramètre `active_account_ids` de `kesh-reconciliation` (crate hors périmètre).

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/{bank_accounts,reconciliation_rules}.rs`,
`crates/kesh-api/src/routes/{reconciliation,reconciliation_rules,company_invoice_settings,bank_accounts,products}.rs`,
`crates/kesh-api/src/errors.rs` (commentaire), `crates/kesh-db/src/repositories/{invoice_settlements_write,journal_entries}.rs`
(commentaires), `crates/kesh-i18n/locales/*/messages.ftl`,
`frontend/src/lib/features/reconciliation/{failed-proposal-label.ts,failed-proposal-label.test.ts,ReconciliationProposals.svelte,ReconciliationProposals.test.ts}`,
`frontend/src/lib/features/reconciliation/rules/{RuleFormModal.svelte,RuleFormModal.test.ts,RulesList.svelte,RulesList.test.ts}`,
`frontend/src/routes/(app)/reconciliation/rules/+page.svelte`,
`frontend/src/routes/(app)/bank-accounts/+page.svelte` (+ test), tests
(`crates/kesh-api/tests/{reconciliation_manual_e2e,reconciliation_split_e2e,reconciliation_e2e,reconciliation_rules_e2e,bank_accounts_e2e}.rs`,
nouveau `company_invoice_settings_postable_e2e.rs`, `crates/kesh-db/tests/{reconciliation_rules_repository,bank_accounts_repository}.rs`),
`docs/manual/fr/user-manual.{tex,pdf}`, `CHANGELOG.md`. **Aucune migration** (P1–P8 sans objet).

### Tests — ce qui rendrait un test vert sans rien prouver

- Asserter `400` sans le code : un refus pour une autre raison passerait. Asserter le code.
- Compte de test non imputable **et** d'un autre type, ou d'une autre société : le refus viendrait du
  type ou du tenant. Ne changer **que** `postable`.
- Fixtures des quatre `reconciliation_*_e2e.rs` : elles ne posent que `postable: true` — le compte non
  imputable doit être créé ou basculé explicitement.
- Le test « PATCH inchangé → 200 » doit renvoyer le compte **dans le corps** : un PATCH sans
  `counterpartyAccountId` ne prouve pas l'exemption.
- Le test de libellés doit lister les codes **en dur** (la liste de l'AC14) : un test qui itère sur la
  table de libellés elle-même est vert par construction.

### References

- Issues : #427, #429 (et son commentaire), #474, #492, #519, #271, #375 (24-5).
- `crates/kesh-api/src/routes/reconciliation.rs:155-170` (`FailedProposal`), `:449-640`, `:1829-1990`,
  `:2207-2320`, `:2942-3160`, `:3362-3620`.
- `crates/kesh-db/src/repositories/reconciliation_rules.rs:243-330`, `bank_accounts.rs:237-410`.
- `crates/kesh-api/src/routes/company_invoice_settings.rs:133-330`, `bank_accounts.rs:257-295`.
- `_bmad-output/implementation-artifacts/14-3b-consommateurs-roles.md` (D-A0, D-A1),
  `24-5-comptes-de-cloture.md`.
- Fiche source : `15-5-gardes-postabilite-serveur.md` (statut `split`) ; socle : `15-5a-refus-non-imputable.md`.
- `CLAUDE.md` : § *Pattern batch*, § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit
  NOMMER le manuel*, § *Test Locally First* (exception `kesh-db`), § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée par découpage de la 15-5** après la passe de validation **P1** (trois lentilles
  Sonnet, contexte frais ; prompt `15-5-validate-prompt-p1.md`). Bilan de P1 sur la 15-5 entière :
  **0 CRITICAL, 0 HIGH** ; bruts par lentille A 5 MEDIUM / 6 LOW, B 2 / 2, C 4 / 4 ; **7 MEDIUM et 8 LOW
  distincts** après fusion (ventilation dans la fiche 15-5). Ce qui en revient à cette moitié, et
  comment :
  - **M2 = B1** (MEDIUM, A et B) — « Aucun changement d'écran n'est requis » était faux : `failed[]`
    s'affiche en code brut → AC14, libellés pour tous les codes, ferme #492 (choix **C8**).
  - **M3 ≈ C-2** (MEDIUM, A et C) — un PATCH `active:true` ressuscitait une règle périmée ; ce que voit
    l'utilisateur n'était pas dit → AC8 (b), AC9, manuel (choix **C9**).
  - **B2 + C-6** (MEDIUM B, LOW C) — lire la valeur en place avant la validation changeait l'ordre des
    erreurs, hors transaction → contrôle dans la transaction, sous le verrou, ordre écrit et testé
    (choix **C10**).
  - **M5 = C-4** (MEDIUM, A et C) — splitting → découpage (choix **C7**).
  - **B3 + C-5** (LOW) — numéros de ligne : `fn` et appels distingués (`:384`/`:393`, `:498`/`:512`,
    `:717`/`:727`) ; dépôt `kesh-db/.../company_invoice_settings.rs` qualifié ; `:1656` = appel dans
    `reverse_in_tx_inner` (`fn` `:1518`).
  - **L1** (LOW, A) — chemins frontend complets.
  - **C-7** (LOW, C) — comportement du `<select>` vérifié au code de Svelte 5.55 et écrit (AC13) ;
    correction d'affichage par `withCurrentAccount` (choix **C12**).
  - **L5 + C-8** (LOW) — manuel : § *Règles d'affectation* inexistant à l'écran → réécrit (AC17, choix
    **C11**, #519) ; `admin-manual.tex` sans objet, écrit.
  - **L6** (LOW, A) — inventaire (a) « ±1 » : recompté (20 lignes rendues, 3 homonymes, 17 classées) ;
    le délégateur `journal_entries.rs:194` et les trois homonymes sont désormais nommés.
  - **L2/B4** (LOW) — `details.accountNumbers` aussi dans `failed[]` (choix **C13**).
  - **Trouvé pendant la remédiation** (axe manuel) : § *Acceptation par lot* (`user-manual.tex:1532`)
    dit l'opération « atomique » et renvoie au `CLAUDE.md` — faux, réécrit (AC17).
  - **Trouvé pendant la remédiation** (axe écrans) : `RuleFormModal.svelte:104`, `RulesList.svelte:64`
    et `:87` affichent `String(e)` d'un `ApiError` objet simple — « [object Object] » — : le refus que
    cette story ajoute y serait illisible → second volet de l'AC14 (choix **C14**). Le même motif existe
    hors du module (`reports/+page.svelte:210`, `settings/+page.svelte:236`, `:260`) : signalé à
    l'orchestrateur pour une issue, hors périmètre.
  Décisions de l'orchestrateur : C7 à C11 ; ajoutées pendant la remédiation : C12, C13, C14.
