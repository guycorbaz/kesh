# Story 15.5b : Gardes de postabilité côté serveur — rapprochement, règles, réglages de facturation, compte bancaire

Status: done

<!-- Issue de la story 15-5, DÉCOUPÉE le 2026-10-08 après la passe de validation P1 (choix C7 de
     `epic-15-choix-autonomes.md`). Sous-story de « rollout » : elle applique aux surfaces neuves la
     variante `DbError::AccountsNotPostable` posée par la 15-5a. Choix applicables : C3, C4, C5, C6,
     C7, C9, C10, C12 (corrigé par C26), C13 (clé révisée par C16), C14, C15 (découpage en 15-5c), C16,
     C19, C23, C24, C25 (révisé par C34), C26, C29 (accesseur unique du détail), C30 (création de règle
     gardée dans le dépôt), C33 (découpage en 15-5d). C8 et C11 sont passés à la 15-5c ; C27, C28 et la
     garde à l'usage des comptes de réglage (ex-AC20) à la 15-5d. Validations P2, P3 et P4 faites
     (Change Log). -->

**Issues** : **ferme #427** (P1) et **#521** (l'enregistrement des réglages de facturation efface le
compte créanciers — choix C25). La PR porte `closes #427 closes #521 refs #429` (mots-clés **dans la
PR**, le dépôt merge en squash). **#429 n'est PAS fermée ici** : cette story garde les comptes de
réglage **à la désignation** (AC10) ; la seule désignation laisse un réglage devenu non imputable
recevoir des écritures, et c'est la garde **à l'usage**, portée par la **15-5d** (choix C27, C33), qui
la ferme.
Voisin repris en partie : **#474** — seule la *postabilité* du compte comptable d'un compte bancaire
est traitée ici ; le reste de #474 appartient à la **15-6** (choix C5). **#492** (libellés des refus
par lot) et **#519** (manuel des règles d'affectation) sont passés à la **15-5c** (choix C15), avec
#481.

**Dépend de la 15-5a** (variante `DbError::AccountsNotPostable`, code `ACCOUNT_NOT_POSTABLE`, clé
`error-account-not-postable`, `details.rejected[{accountId, accountNumber}]`). Ne pas commencer avant
son merge. **La 15-5c et la 15-5d dépendent de cette story.**

## Story

En tant que **comptable ou administrateur d'une société tenue dans Kesh**,
je veux que **le serveur refuse un compte non imputable partout où le client désigne un compte qui
recevra une écriture** — rapprochement, règles de rapprochement, réglages de facturation, compte
comptable d'un compte bancaire —,
afin qu'**aucune écriture ne puisse atterrir sur un compte de regroupement, de résultat ou de clôture**, quel que
soit le client qui appelle (écran, clé d'API, intégration), et que le refus soit lisible à l'écran.

### Pourquoi maintenant

La doctrine de la Story 24-5 — *une garde serveur ne se déduit pas d'un filtre d'écran* — a été
appliquée à trois chemins. Elle laisse **ouverts** ceux de #427 et #429, que le manuel avoue en toutes
lettres (`docs/manual/fr/user-manual.tex:380` et `:390`). Les écrans filtrent ; ces routes sont
atteignables par un appel HTTP direct — par clé d'API pour le rapprochement, les règles et le compte
bancaire ; par une session Admin pour les réglages de facturation, route d'administration (enregistrée
parmi les routes Admin, `crates/kesh-api/src/lib.rs:207-211`) qu'une clé d'API ne peut pas atteindre —
le refus `API_KEY_ADMIN_FORBIDDEN` est posé par la couche `require_not_pat` (`lib.rs:337`) — et rien
ne contrôle le compte côté serveur. Un rapprochement routé sur
**9000 Bilan d'ouverture** fausse le résultat de l'exercice et le report à nouveau (#427).

### Une story de rollout

Elle applique un patron posé ailleurs (15-5a pour la variante ; `resolve_designated_account` et
14-3b pour la garde et l'exemption) à des surfaces nombreuses. Elle touche donc, **par nature**, plus de
cinq modules : `kesh-db` (dépôts `bank_accounts`, `reconciliation_rules`), quatre modules de routes
`kesh-api` (`reconciliation`, `reconciliation_rules`, `company_invoice_settings`, `bank_accounts`, plus
un doc-comment de `products`), deux modules `frontend` (`reconciliation`, `bank-accounts`), deux
encadrés du manuel et `docs/api-external.md`. C'est la forme que la § *Règle de splitting préventif*
prescrit pour la seconde moitié d'un découpage ; elle prévoit que **la sous-story de rollout est revue
fichier par fichier** — c'est le mode de revue demandé ici (choix C7).

⚠️ **Elle n'est pas strictement mécanique, et la validation P2 l'a dit** (finding F-3). Ce qui ne
l'était pas du tout — un module neuf de libellés traduits (#492) et la réécriture du manuel du
rapprochement (#519, #481) — est passé à la **15-5c** (choix C15). La garde **à l'usage** des comptes
de réglage, ajoutée en validation P3, est une règle métier neuve (révision de la limite L2 de D-A0) :
la validation P4 l'a relevé (finding F4-3) et elle est passée à la **15-5d** (choix C33). Ce qui reste
porte une sémantique propre et n'est pas un simple report du patron : le filtre de `get_proposals`
(AC5), le refus de la réactivation (AC8 b), l'écran du compte bancaire (AC13) et le compte créanciers
préservé (AC19, #521).
Ces points relèvent des passes de validation et de revue ordinaires, pas de la seule revue fichier par
fichier.

## Acceptance Criteria

### Rapprochement (#427)

1. **AC1 — `POST /reconciliation/manual`** (`post_manual`, `crates/kesh-api/src/routes/reconciliation.rs:2942`) :
   après le contrôle existant du compte de contrepartie (`Some(a) if a.active => a`, `:3028`), un compte
   **non imputable** → `AppError::Database(DbError::accounts_not_postable([NonPostableAccount { account_id: a.id, account_number: a.number }]))`, soit **400
   `ACCOUNT_NOT_POSTABLE`** ; **rien n'est écrit** (aucune écriture, transaction bancaire toujours
   `pending`, aucun audit). Inconnu / autre société / archivé → **404 `ACCOUNT_NOT_FOUND`**, inchangé
   (anti-énumération) et **prioritaire**.
2. **AC2 — `POST /reconciliation/split`** (`post_split`, `:3362`) : à l'étape de contrôle des comptes de
   contrepartie (`:3488`), les non imputables sont collectés **après** les manquants ; s'il y a des
   manquants → 404 `ACCOUNT_NOT_FOUND` **inchangé** (prioritaire) ; sinon, s'il y a des non imputables →
   400 `ACCOUNT_NOT_POSTABLE` nommant **tous** ceux de la requête ; rien n'est écrit.
3. **AC3 — `POST /reconciliation/accept`, proposition `split`** (`accept_one_split`, `:1829`, étape d
   `:1952-1981`) : le `SELECT` lit `id, active, postable, number` ; inconnu / archivé → `failed[]`
   `ACCOUNT_NOT_FOUND` **inchangé** (prioritaire) ; sinon, une ligne non imputable → `failed[]` avec
   `errorCode = "ACCOUNT_NOT_POSTABLE"` et `details = { "rejected": [{ "accountId", "accountNumber" }] }`
   (la forme du 400 de la 15-5a, triée, dédoublonnée — **la valeur rendue par
   `NonPostableAccounts::details()`**, posé par la 15-5a, choix C16 et C29 : aucun JSON construit à la
   main ici) ; **HTTP 200**, les autres propositions du lot sont traitées normalement (§ *Pattern
   batch — FailedProposal* du `CLAUDE.md`).
4. **AC4 — `POST /reconciliation/accept`, proposition `rule`** (`accept_one_rule`, `:2207`, étape 5
   `:2289-2318`) : le `SELECT` lit `active, postable, name, number` ; archivé → `ACCOUNT_NOT_FOUND`
   inchangé ; non imputable → `failed[]` `ACCOUNT_NOT_POSTABLE`, `details = { "rejected": [{ "accountId",
   "accountNumber" }] }` — par `NonPostableAccounts::details()` (C29).
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

7. **AC7 — Création** (`post_create`, `crates/kesh-api/src/routes/reconciliation_rules.rs:248`) : le
   contrôle de postabilité se fait **dans la transaction**, dans le dépôt
   (`reconciliation_rules::create_in_tx`, `crates/kesh-db/src/repositories/reconciliation_rules.rs:177`,
   seul appelant `routes/reconciliation_rules.rs:286`), comme celui du PATCH (AC8) et non par le seul
   pré-vol du handler (choix C30) : **après** la validation du projet par défaut (`:186-188`) et
   **avant** l'`INSERT`, lecture `SELECT number, postable, active FROM accounts WHERE id = ? AND
   company_id = ?` dans la transaction ; refus **seulement si `active && !postable`** →
   `DbError::accounts_not_postable([…])` → 400 `ACCOUNT_NOT_POSTABLE` (le handler propage par
   `map_create_or_update_db_err`, `:228-241`, qui rend `AppError::Database` pour tout ce qui n'est pas
   un doublon), aucune règle créée. Le pré-vol `validate_counterparty_account` (`:200`) est
   **inchangé** (404 existence / société / `active`). Ordre : validations de forme → 404 compte
   (handler) → refus du projet (dépôt) → **400 `ACCOUNT_NOT_POSTABLE`** → 409 doublon (à l'`INSERT`).
8. **AC8 — Modification** (`patch`, `:345`) — le contrôle de postabilité se fait **dans la
   transaction**, dans `reconciliation_rules::update_in_tx`
   (`crates/kesh-db/src/repositories/reconciliation_rules.rs:254`), sur la ligne `before` qu'elle lit déjà
   (`:262-264`), **juste après** la validation du projet par défaut (`:271-276`, même patron « seulement
   si ça change »). Le compte **cible** est `patch.counterparty_account_id.unwrap_or(before.counterparty_account_id)`.
   Il est contrôlé sur `postable` (lecture `SELECT number, postable, active FROM accounts WHERE id = ?
   AND company_id = ?` dans la transaction ; refus **seulement si `active && !postable`** — un compte
   archivé, non imputable ou non, garde le comportement actuel décrit sous *Hors périmètre*, et la
   variante n'est émise que pour un compte actif, comme l'exige la 15-5a) **si et seulement si** :
   - (a) le PATCH **change** le compte (`Some(id)` avec `id != before.counterparty_account_id`), **ou**
   - (b) le PATCH **réactive** la règle (`patch.active == Some(true)` et `before.active == false`) —
     **même si le compte est inchangé** (choix C9 : réactiver une règle dont le compte n'est plus
     imputable ressusciterait une règle qui ne s'appliquera jamais, ou pire).
   Non imputable → `DbError::accounts_not_postable([…])` → 400 `ACCOUNT_NOT_POSTABLE`. **Exemption**
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
   **réactivation** après désactivation est refusée (AC8 b). Le manuel le dira (15-5c, qui réécrit la
   section des règles). Aucune donnée n'est réécrite.

### Réglages de facturation (#429)

10. **AC10 — Les six champs historiques** de `PUT /company/invoice-settings`
    (`update_invoice_settings`, `crates/kesh-api/src/routes/company_invoice_settings.rs:246`, appels
    `:276-325`) — `defaultReceivableAccountId`, `defaultRevenueAccountId`, `defaultVatPayableAccountId`,
    `defaultVatRecoverableAccountId`, `defaultVatDecompteAccountId`, `defaultPayableAccountId` —
    refusent un compte **non imputable** quand la valeur **change** par rapport au réglage en place
    (`current`, lu `:256`) : 400 `VALIDATION_ERROR`, message
    « {champ} : compte non imputable (compte de regroupement, de résultat ou de clôture) » — **le message
    émis** par `validate_account_of` (`:190-194`) pour les comptes désignés, dont la parenthèse est
    **corrigée** par cette story : elle omettait le compte de résultat (2979), comme le message de la
    15-5a avant sa validation P2 (choix C19) ; toute assertion existante sur l'ancienne parenthèse est
    mise à jour (grep à T8). Choix C3 : les refus de cette route sont tous des `Validation` nommant le
    champ. **Limite écrite** : ce message est un `AppError::Validation` en **français en dur**, non
    traduit, comme tous les refus de cette route aujourd'hui — la traduction n'est pas l'objet de la
    story. Mise en œuvre : chaque appel passe à
    `validate_account_of(…, require_postable = (req.champ != current.champ), …)`, ou `validate_account`
    reçoit ce paramètre. Pour `defaultPayableAccountId`, la valeur comparée est celle que résout l'AC19
    (absent du corps → préservé, donc inchangé).
11. **AC11 — Exemption « inchangé »** : une valeur **égale** à celle en place n'est pas re-contrôlée sur
    `postable` — patron de `resolve_designated_account` (`:200-229`). Sans elle, un compte de réglage
    devenu non imputable après coup (ajout d'un sous-compte, règle 14-3a) bloquerait **tout**
    enregistrement des réglages : le formulaire renvoie **cinq** des six valeurs
    (`withCurrentAccount`, `frontend/src/routes/(app)/settings/invoicing/+page.svelte:85-111`, #271, et
    l'appel `updateInvoiceSettings`, `:174-189`) — **pas `defaultPayableAccountId`, qu'il n'envoie
    jamais** (#521) : ce sixième champ est préservé par son **absence** (AC19), et l'exemption par
    égalité ne joue pour lui que lorsqu'un client d'API le renvoie. Les contrôles existants (existence,
    société, `active`, type) restent **inconditionnels** sur toute valeur **présente** dans le corps.
    ⚠️ **L'exemption n'est pas une tolérance à l'usage** : elle évite seulement de bloquer
    l'enregistrement des réglages. Un réglage « inchangé » devenu non imputable sera refusé **quand un
    flux veut y écrire** — c'est l'usage qui refuse, pas la désignation (choix C27), et cette garde est
    portée par la **15-5d** (C33). Jusqu'à son merge, un tel réglage reste utilisé par les flux
    automatiques, comme aujourd'hui.

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
      qu'ils lisent (`existing.journal_account_id`) (choix C10). La lecture du compte est
      `SELECT number, postable, active` et le refus n'a lieu que si `active && !postable` (même règle
      qu'à l'AC8). Il s'applique **seulement si la valeur change** et n'est pas `NULL` : dans `set_journal_account_id_for_company`, **après** le court-circuit
      « no-op » (`:306`) ; dans `update_for_company`, après le contrôle de version (`:380-382`), sous la
      condition `new_journal_account_id != existing.journal_account_id`. Non imputable →
      `DbError::accounts_not_postable([…])` → 400 (les handlers propagent déjà `Err(e) =>
      AppError::Database(e)`, `:557` et `:751`).
    - **Ordre des erreurs conservé**, écrit dans les doc-comments : forme → 404 / 400 type du compte
      (`validate_journal_account_id`, hors transaction, inchangé) → 404 compte bancaire → 409 version →
      **400 `ACCOUNT_NOT_POSTABLE`**. Un PUT qui promeut le compte en principal et échoue sur ce refus
      **n'a rien démoté** (la transaction est abandonnée, `flip_primary_off_for_company` `:528-534`
      compris).
    - Le compte débiteurs, lui, reste accepté ici : c'est la 15-6 (choix C5).
13. **AC13 — L'écran garde le compte lié visible, sur ses trois surfaces.** Dans
    `frontend/src/routes/(app)/bank-accounts/+page.svelte` :
    - les deux `<select>` du compte lié (`:321-330`, `:472-481`) reçoivent leurs options de
      `withCurrentAccount(linkableAccounts, formJournalAccountId, accounts)`
      (`frontend/src/lib/features/accounts/account-options.ts:58`) (choix C12) ;
    - `BankAccountJournalLinkForm` (`:435-437`) reçoit la **liste complète** : `accounts={accounts}` au
      lieu de `accounts={linkableAccounts}`. Le composant appelle déjà
      `withCurrentAccount(eligibleAccounts, selectedAccountId, accounts)`
      (`BankAccountJournalLinkForm.svelte:60`) et filtre lui-même ses options (`:44-55`) ; mais la page
      lui passe aujourd'hui une liste **déjà filtrée** `active && postable` (`+page.svelte:78-85`), si
      bien que `withCurrentAccount` n'y retrouve jamais le compte devenu non imputable
      (`account-options.ts:69-71`) et que le champ s'affiche vide — le défaut #271 sur la troisième
      surface, celle du PATCH de l'AC12 (choix C26, qui corrige C12).
    **Fait vérifié au code** (Svelte 5.55,
    `node_modules/svelte/src/internal/client/dom/elements/bindings/select.js`) : quand la valeur liée
    n'est dans aucune option, `select_option` pose `selectedIndex = -1` — le champ s'affiche **vide** —
    mais **ne réécrit pas** la variable (« the model should be preserved unless explicitly changed ») ;
    le PUT renvoie donc l'identifiant en place et l'exemption de l'AC12 joue. Le défaut est d'affichage :
    l'utilisateur voit un champ vide pour un lien qui existe, et peut l'effacer sans le savoir — c'est
    exactement le cas de #271.

### Les refus de l'écran des règles, lisibles

14. **AC14 — Les refus de l'écran des règles s'affichent, au lieu de `[object Object]`** (choix C14).
    *(Le premier volet de l'ancien AC14 — un libellé traduit par `errorCode` pour les refus par lot,
    #492 — est passé à la 15-5c, choix C15.)*
    Le client d'API lève un `ApiError` **objet simple**, pas une instance d'`Error`
    (`frontend/src/lib/shared/types/api.ts:9-14`, `api-client.ts:208-233`) ; or
    `RuleFormModal.svelte:104`, `RulesList.svelte:64` et `:87` font
    `e instanceof Error ? e.message : String(e)` — le refus 400 de l'AC7 et de l'AC8 s'y afficherait
    « [object Object] ». Ces trois sites, ainsi que `ReconciliationProposals.svelte:68`, `:160`, `:178` et
    `frontend/src/routes/(app)/reconciliation/rules/+page.svelte:32` (même motif, même module), passent au
    patron de `ManualMatchModal.svelte:126-133` : `isApiError(e) ? e.message : (e instanceof Error ?
    e.message : String(e))`. Un test Vitest par composant le vérifie (un `ApiError` rejeté → son
    `message` affiché), **y compris la page** `reconciliation/rules/+page.svelte` (chargement en échec).

### Ce qui ne change pas, et qui doit être prouvé

15. **AC15 — Non-régression des flux automatiques** : `create_in_tx(…, false)` accepte toujours un compte
    de **configuration** devenu non imputable (`test_create_in_tx_auto_flow_allows_non_postable`,
    `crates/kesh-db/src/repositories/journal_entries.rs:3687`) — au niveau de `create_in_tx` : la garde
    des comptes de réglage à l'usage ne passe **pas** par ce drapeau ; elle est posée **en amont**, par
    la 15-5d, dans la validation d'une facture et la saisie d'une facture fournisseur — **l'avoir en
    est exempté** (il relit la créance et la TVA due dans les réglages du moment, défaut suivi par
    #473, #523 et #525 ; choix C35) ; la contre-passation (`reverse_in_tx_inner`,
    `fn` `:1518`, appel de `create_in_tx_inner` `:1656`) et l'annulation d'un rapprochement restent
    possibles sur un compte devenu non imputable (`reverse_succeeds_when_an_account_became_non_postable`,
    `crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1015`). Un compte bancaire dont le compte
    comptable est **devenu** non imputable continue de servir aux rapprochements (décision D-A0 de la
    14-3b, choix C6) : les contrôles `active` du compte de banque (`reconciliation.rs:1914-1938`, `:3011`,
    `:3454`) ne reçoivent **pas** de clause `postable`.
16. **AC16 — Tests négatifs « compte non imputable refusé » sur CHAQUE surface** des AC 1 à 12, dont
    le compte de test ne diffère d'un compte accepté **que par `postable`** (même société, actif, bon type)
    et dont l'assertion porte sur le **code** (`ACCOUNT_NOT_POSTABLE`, ou le message du champ pour les
    réglages) **et** `details.rejected` (`accountId` et `accountNumber`) là où il existe — pas sur le
    seul statut. Chaque surface à exemption porte **aussi** son test positif « inchangé → accepté ».
    Chaque surface à ordre écrit porte un test de **priorité** : AC1 (archivé → 404 avant tout), AC2 et
    AC3 (manquant **et** non imputable → `ACCOUNT_NOT_FOUND`), AC4 (archivé → `ACCOUNT_NOT_FOUND`), AC8
    (400 non imputable **avant** 409 de version, choix C10) et AC12. Chaque test négatif est **muté** une fois
    (garde retirée → le test rougit), consigné au Dev Agent Record.
17. **AC17 — Le manuel lève ses réserves** (`docs/manual/fr/user-manual.tex`) :
    - la réserve des deux encadrés **`:380`** (§ *Rôles des comptes*) et **`:390`** (§ *Les comptes de
      clôture*) est **levée pour ce que cette story ferme** : plus de « une intégration … pourrait les
      viser » ; ils disent que le rapprochement (manuel, ventilé, proposé, et ses règles), les réglages de
      facturation (**à la désignation**) et le compte comptable d'un compte bancaire
      refusent un compte non imputable — et ils nomment les trois sortes de comptes non imputables
      (regroupement, **résultat**, clôture, choix C19) ;
    - ils **ne promettent pas une protection absolue** (finding P3 F-2) : ils nomment, en termes
      d'utilisateur, les cas qui **échappent encore** — (i) le compte comptable d'un compte bancaire
      **déjà lié**, et son **usage** par le rapprochement et les règlements, quand il est devenu non
      imputable après coup (D-A0, C6) ; (ii) une règle **active** dont le compte est devenu non
      imputable reste listée « Active », mais n'est plus proposée (AC9) ; (iii) le compte de produit
      d'une **fiche article** (D3, C6) ; (iv) un compte **désigné dans les réglages de facturation**
      devenu non imputable **après** sa désignation, qui reste utilisé par les écritures automatiques
      (validation d'une facture, avoir, facture fournisseur) — la **15-5d** réécrit cet item quand elle
      pose la garde à l'usage (elle en excepte alors l'avoir et le compte de produit par défaut). ⚠️
      Ne **pas** écrire que l'avoir « reprend les comptes de la facture d'origine » : il n'en reprend
      que les comptes **de produit** ; la créance et la TVA due sont relues dans les réglages du moment
      (`credit_notes.rs:360-364`, `:507-512` ; findings R4-2/F4-2 de la P4) ;
    - `docs/manual/fr/admin-manual.tex`, § *Configuration des comptes TVA* (`:2016-2025`) : **à
      compléter** (finding P3 F-3) — ses voisins *Compte de différences d'arrondi* (`:2027`) et *Comptes
      du solde du reste* (`:2029`) disent « actif et **imputable** », lui non ; après l'AC10, un compte
      TVA non imputable est refusé **à la désignation**. Une phrase le dit, dans le registre de ses
      voisins (la 15-5d la complète pour l'usage). Le reste du manuel d'administration : contrôle sans
      objet, à refaire et écrire au Dev Agent Record ;
    - les PDF (`user-manual.pdf`, `admin-manual.pdf`) sont régénérés (`latexmk -xelatex` dans
      `docs/manual/fr/`), commités, et **contrôlés aplatis en normalisant les ligatures**
      (`pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'` — le
      corps du PDF rend « ff » par la ligature `ﬀ`, finding F-7 de la P2 de la 15-5c) : les phrases
      levées sont absentes, les nouvelles présentes.
    *(La réécriture du rapprochement lui-même — acceptation par lot, rapprochement manuel, éclatement,
    bouton Modifier, section des règles d'affectation, FAQ — est à la 15-5c, choix C15.)*
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
    - **Deux commentaires que cette story possède seule** (choix C23 — désignés par leur contenu, les
      numéros de ligne étant décalés par la 15-5a) : dans `crates/kesh-db/src/repositories/invoice_settlements_write.rs`,
      le commentaire du compte interne qui se termine par « Les trois flux de réconciliation, eux,
      restent ouverts et sont suivis par #427 » ; dans `journal_entries.rs`, le paragraphe
      `enforce_postable` du doc-comment de `create_in_tx` (« Les flux automatiques appelants directs
      (invoices, credit_notes, supplier_invoices, reconciliation) passent `false` … »). Dire que les flux
      de rapprochement gardent désormais le compte **client** en amont. Le doc-comment de
      `validate_lines_accounts_in_tx` appartient à la 15-5a et **n'est pas touché** ici.
    - **`docs/api-external.md`** (choix C24) : dans § *Accepter des propositions de rapprochement*, les
      codes de `failed[]` gagnent `ACCOUNT_NOT_POSTABLE` (`details.rejected[{accountId,
      accountNumber}]`), pour une proposition `split` ou `rule` ; une phrase dit que le rapprochement
      manuel (`POST /reconciliation/manual`) et ventilé (`POST /reconciliation/split`) refusent un compte
      de contrepartie non imputable en `400 ACCOUNT_NOT_POSTABLE`, après le `404 ACCOUNT_NOT_FOUND` ; la
      ligne `ACCOUNT_NOT_POSTABLE` de la table du § 10 (posée par la 15-5a) gagne ces routes, celles des
      règles et du compte bancaire (la validation d'une facture et la saisie d'une facture fournisseur y
      sont ajoutées par la 15-5d).
    - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (créée par la 15-5a ; **la créer en tête si
      absente** — motif exact exigé par `scripts/prepare-release.sh:189`), rubrique **Corrigé** : refus
      serveur des comptes non imputables sur le rapprochement (manuel, ventilé, propositions par règle et
      ventilées), les règles de rapprochement (création, changement de compte, réactivation), les
      réglages de facturation (à la désignation ; l'usage est à la 15-5d) et le compte comptable d'un
      compte bancaire ; une règle dont le compte n'est plus
      imputable n'est plus proposée ; enregistrer les réglages de facturation n'efface plus le compte
      créanciers (#521).

### Le compte créanciers préservé (#521)

19. **AC19 — `defaultPayableAccountId` absent du corps : préservé** (choix C25). Aujourd'hui l'écran
    *Paramètres → Facturation* ne l'envoie jamais (`settings/invoicing/+page.svelte:174-189`) et le
    serveur le déclare `Option<i64>` (`company_invoice_settings.rs:97`), persisté tel quel (`:389`) :
    **chaque enregistrement efface le compte créanciers**, et la facture fournisseur échoue ensuite en
    `ConfigurationRequired("default_payable_account_id")`
    (`crates/kesh-db/src/repositories/supplier_invoices.rs:361-362`). Correction,
    sur le patron du compte d'arrondi (25-4-c3-a1, `:98-103`) et de `credit_note_number_format` (#216) :
    - le champ devient `Option<Option<i64>>`, `#[serde(default, deserialize_with =
      "crate::helpers::double_option")]`, avec le doc-comment du patron ;
    - **absent** → la valeur en place (`current.default_payable_account_id`) est conservée, **sans
      contrôle** (patron `resolve_designated_account` : on ne refuse pas un enregistrement pour un champ
      que le client n'a pas envoyé) ; **`null`** → effacé ; **valeur** → validée comme aujourd'hui
      (existence, société, `active`, type `Liability`), puis sur `postable` si elle **change** (AC10,
      AC11) ;
    - la valeur résolue est celle que l'`UPDATE` persiste (`:389`).
    Le champ n'est pas exposé à l'écran **dans cette story** : il l'est par la **15-5d** (choix C34,
    qui révise C25), qui pose la garde à l'usage dont il devient le seul recours à l'écran. Les E2E qui
    le reposaient à la main (`payment-batches.spec.ts:57-64`, `inbox-import.spec.ts:91-97`) restent
    valides ici ; la 15-5d décide de leur retrait (son AC6).

### Les comptes de réglage, contrôlés à l'usage (#429) — passés à la 15-5d

L'ancien **AC20** (garde à l'usage de la créance, de la TVA due, des créanciers et de la TVA
récupérable ; variante `DesignatedAccountsNotPostable` ; clé `error-designated-account-not-postable` ;
choix C27 et C28) est sorti de cette story à la validation P4 (finding F4-3, choix **C33**) : c'est une
règle métier neuve — la révision de la limite L2 de D-A0 —, non un rollout, et les deux HIGH et un des
MEDIUM de la passe en sont nés. Il vit dans `15-5d-garde-usage-comptes-reglage.md`, avec l'exposition à
l'écran du compte créanciers (C34) et l'exemption écrite de l'avoir (C35). Les critères ne sont **pas
renumérotés** : cette story compte désormais **19 AC (AC1–AC19)**.

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

| # | site | 5ᵉ arg. | d'où vient le compte | avant | après 15-5a + 15-5b (+ 15-5d) |
|---|---|---|---|---|---|
| 1 | `reconciliation.rs:1581` `accept_one_invoice` | `false` | banque (config), créance (écriture de vente), arrondi (réglage gardé) | aucun compte client | **inchangé**, commenté (AC6) |
| 2 | `reconciliation.rs:2083` `accept_one_split` | `false` | **client** | **NON GARDÉ** (`active` seul, `:1952-1981`) | gardé (AC3) |
| 3 | `reconciliation.rs:2432` `accept_one_rule` | `false` | règle | **NON GARDÉ** (`active` seul, `:2289-2318`) | gardé (AC4, AC5) |
| 4 | `reconciliation.rs:3153` `post_manual` | `false` | **client** | **NON GARDÉ** (`:3028`) | gardé (AC1) |
| 5 | `reconciliation.rs:3619` `post_split` | `false` | **client** | **NON GARDÉ** (`:3488`) | gardé (AC2) |
| 6 | `invoice_settlements_write.rs:214` règlement client | `false` | client (compte interne) / banque (config) | gardé 24-5 | nom juste (15-5a) ; banque : AC12 |
| 7 | `invoice_settlements_write.rs:497` solde du reste | `false` | réglages désignés | gardé (`kesh-db/src/repositories/company_invoice_settings.rs:469` `vat_payable_account_for_write`, `:499` `usable_designated_account`, `postable = TRUE`) | inchangé |
| 8 | `supplier_invoices.rs:372` facture fournisseur | `false` | client (charge) + réglages (créanciers, TVA récup.) | charge gardée 24-5 ; réglages **NON GARDÉS** (#429) | 15-5a ; réglages : désignation (AC10) ; **usage : 15-5d** ; créanciers préservé (AC19) |
| 9 | `supplier_invoices.rs:693` règlement fournisseur | `false` | client (compte interne) / banque | gardé 24-5 | 15-5a ; banque : AC12 |
| 10 | `invoices.rs:2255` validation de facture | `false` | lignes (gardées) + réglages (créance, TVA due, produit par défaut) | **créance et TVA due NON GARDÉES** (#429) ; produit par défaut exempté (D3-bis) | créance, TVA due : désignation (AC10) ; **usage : 15-5d** ; produit par défaut : exemption D3-bis conservée, écrite (AC17) |
| 11 | `credit_notes.rs:548` avoir | `false` | produit : snapshot des lignes de la facture (D5-bis, `:503-505`) ; **créance et TVA due : réglages du moment** (`:360-364`, `:507-512`) | produit : validé à l'émission ; **créance et TVA due NON GARDÉES** | inchangé ici ; à l'usage, **exemption délibérée écrite par la 15-5d** (C35) : créance → 15-6a (#473, #523), TVA due → #525 |
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
| `company_invoice_settings.rs` six champs historiques | indirectement (#8, #10, #11) | désignation : **AC10–AC11**, et **AC19** pour `defaultPayableAccountId` (#521) ; **usage : 15-5d** pour créance, TVA due, créanciers, TVA récupérable (avoir exempté, C35) — **un réglage inchangé n'est pas « traité » par la seule désignation** (finding P3 F-1) ; produit par défaut : exemption D3-bis ; décompte TVA : lu par aucun flux d'écriture (l'export CSV, `csv_tables.rs:932`, et le rapport TVA, `crates/kesh-report/src/vat_report.rs:174`, le **lisent** sans écrire — finding F4-9) |
| `company_invoice_settings.rs` arrondi, escompte, frais, pertes | indirectement (#7) | déjà gardés (`resolve_designated_account`, « si changé ») |
| `bank_accounts.rs` `journalAccountId` (POST, PUT, PATCH) | indirectement (#1, #6, #9, tous les rapprochements) | **AC12–AC13** |
| `invoices.rs` `lines[].revenueAccountId` | oui (#10) | gardé (16-1a) |
| `invoices.rs` règlement `accountId` / `bankAccountId` | oui (#6) | gardé 24-5 |
| `supplier_invoices.rs` `expenseAccountId`, règlement `accountId` | oui (#8, #9) | gardé 24-5 |
| `imported_supplier_invoices.rs` `expenseAccountId` | oui, via `supplier_invoices::create_in_tx` (#8) | gardé en aval ; nom juste (15-5a) |
| `products.rs` `defaultRevenueAccountId` | non (recopié sur la ligne de facture, gardée) | **hors périmètre, délibéré (D3, C6)** — doc-comment amendé (AC18) |
| `credit_notes.rs` `revenueAccountId` | non (champ de **réponse**) | sans objet |
| `payment_batches.rs`, `bank_imports.rs` `bankAccountId` | non (compte bancaire, pas compte du plan) | sans objet |
| `reports.rs` `accountIds` | non (filtre de lecture) | sans objet |
| `accounts.rs` (rôle d'un compte) → `insert_with_defaults_in_tx` | indirectement, une fois, à la création des réglages | **angle mort assumé (C6)** ; tout changement ultérieur passe par le PUT des réglages (AC10) |

## Tasks / Subtasks

- [x] **T0 — Refaire les deux inventaires et TOUS les numéros de ligne de la fiche** sur `HEAD`,
      **après le merge de la 15-5a** (qui réécrit `journal_entries.rs`, `invoice_settlements_write.rs` et
      `supplier_invoices.rs` et décale les lignes citées par l'inventaire (a), l'AC15 et l'AC18 — choix
      C23) ; signaler au Change Log tout site absent de ces listes. Un site neuf non classé bloque la
      story.
- [x] **T1 — Rapprochement** (AC1–AC6)
  - [x] `post_manual` : refus après `Some(a) if a.active` ; doc d'ordre `:2932-2939`.
  - [x] `post_split` : collecte des non imputables après les manquants ; un seul refus.
  - [x] `accept_one_split` étape d : `SELECT id, active, postable, number` ; `FailedProposal` avec
        `details.rejected[{accountId, accountNumber}]`, rendu par `NonPostableAccounts::details()` (C29).
  - [x] `accept_one_rule` étape 5 : `SELECT active, postable, name, number` ; même `FailedProposal`.
  - [x] `get_proposals` : `postable` dans `accounts_info_rows`, filtre de `active_account_ids`,
        commentaire au site d'appel.
  - [x] Commentaire de classement à `accept_one_invoice` (AC6).
- [x] **T2 — Règles** (AC7–AC9)
  - [x] `reconciliation_rules::create_in_tx` (`kesh-db`) : contrôle après la validation du projet,
        avant l'`INSERT`, lecture `number, postable, active`, refus si `active && !postable` ; doc-comment
        (« # Erreurs ») et doc d'ordre de `post_create` (C30). `validate_counterparty_account` inchangé.
  - [x] `update_in_tx` (`kesh-db`) : contrôle (a)/(b) après la validation du projet, lecture
        `number, postable, active`, refus si `active && !postable` ; doc-comment (« # Erreurs ») et doc
        d'ordre de `patch`.
  - [x] Doc de module `reconciliation_rules.rs:13-17`.
- [x] **T3 — Réglages de facturation** (AC10, AC11, AC19)
  - [x] `require_postable = (req.champ != current.champ)` sur les six appels ; doc-comments `:150-158` et
        en-tête ; parenthèse du message de `validate_account_of` corrigée (C19).
  - [x] `default_payable_account_id` : `Option<Option<i64>>` + `double_option`, absent → préservé,
        `null` → effacé, valeur → validée (AC19) ; doc-comment du champ.
- [x] **T4 — Compte bancaire** (AC12, AC13)
  - [x] `validate_journal_account_id(…, require_postable)` ; `true` à la création.
  - [x] `update_for_company` et `set_journal_account_id_for_company` : contrôle « si changé » sous le
        verrou, lecture `number, postable, active`, refus si `active && !postable` ; doc-comments, ordre
        des erreurs.
  - [x] `bank-accounts/+page.svelte` : `withCurrentAccount` sur les deux `<select>` ;
        `accounts={accounts}` (liste complète) passé à `BankAccountJournalLinkForm` (C26) ; tests
        Vitest — fichier **neuf** `frontend/src/routes/(app)/bank-accounts/+page.test.ts` (aucun test de
        cette page n'existe) : un compte lié devenu non imputable reste affiché et sélectionné à
        l'ouverture du formulaire de modification **et** dans le formulaire de lien.
  - [x] E2E Playwright dont le montage lie un compte bancaire au premier compte d'actif actif **sans
        regarder `postable`** — sur un état où le plan PME a posé « 1 Actifs » (parent), le lien serait
        refusé en 400 après l'AC12 : `frontend/tests/e2e/payment-batches.spec.ts:53` (`liquid`) et
        `frontend/tests/e2e/supplier-invoices.spec.ts:77` (`internal`, même forme) gagnent `&& a.postable`.
- [x] **T5 — Les refus de l'écran des règles** (AC14, choix C14) : les sept `catch` nommés à l'AC14
      passent au patron `isApiError` ; un test Vitest par composant (`RuleFormModal.test.ts`, `RulesList`
      — test neuf —, `ReconciliationProposals.test.ts`, et la page `reconciliation/rules/+page.svelte` —
      test neuf).
- [x] **T6 — Tests backend** (AC15, AC16) — helper `set_account_not_postable` sur le patron de
      `crates/kesh-api/tests/products_revenue_account_e2e.rs:254` (UPDATE direct, `version + 1`).
  - [x] `reconciliation_manual_e2e.rs` : non imputable → 400 `ACCOUNT_NOT_POSTABLE` +
        `details.rejected`, aucune écriture, transaction toujours `pending`, aucun audit ; archivé
        → 404 (priorité).
  - [x] `reconciliation_split_e2e.rs` : une ligne non imputable sur trois → 400, `details.rejected` ; une ligne
        manquante **et** une non imputable → 404 (priorité) ; rien d'écrit.
  - [x] `reconciliation_e2e.rs` (accept, proposition `split`) : proposition non imputable → 200,
        `failed[]` avec le code et `details.rejected` ; proposition valide du même lot → `accepted[]` ;
        priorité : une ligne manquante **et** une non imputable → `failed[]` `ACCOUNT_NOT_FOUND`.
  - [x] `reconciliation_rules_e2e.rs` (accept, proposition `rule` — c'est là que vivent les tests
        d'acceptation par règle) : règle dont le compte est devenu non imputable → `failed[]`
        `ACCOUNT_NOT_POSTABLE` ; priorité : compte archivé → `failed[]` `ACCOUNT_NOT_FOUND` ;
        `get_proposals` ne propose plus cette règle et propose la suivante qui correspond.
  - [x] `reconciliation_rules_e2e.rs` : POST non imputable → 400 `ACCOUNT_NOT_POSTABLE` +
        `details.rejected`, aucune règle créée ; POST avec un projet par défaut archivé **et** un compte
        non imputable → refus du projet (ordre de l'AC7) ; PATCH vers un autre compte non
        imputable → 400 ; PATCH renvoyant le **même** compte devenu non imputable (règle active) → 200 ;
        PATCH `active:false` sur une telle règle → 200 ; PATCH `active:true` sur une telle règle
        désactivée → 400 `ACCOUNT_NOT_POSTABLE` ; PATCH `active:true` **avec** un nouveau compte imputable
        → 200 ; PATCH sans compte → 200 ; priorité : compte archivé → 404 avant tout ; PATCH vers un
        compte non imputable **avec une version périmée** → 400 `ACCOUNT_NOT_POSTABLE` (avant le 409,
        choix C10) ; PATCH `active:true` sur une règle désactivée dont le compte est archivé et non
        imputable → comportement actuel (accepté, *Hors périmètre*).
  - [x] Réglages — fichier neuf `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`
        (montage : `idor_multi_tenant_e2e.rs:~751`, `create_seeded_company`) : pour **chacun** des six
        champs, changement vers un compte non imputable du bon type → 400 et message du champ ; PUT
        renvoyant un compte **déjà en place** devenu non imputable → 200. **#521 (AC19)** : PUT **sans**
        `defaultPayableAccountId` → le compte créanciers en place est conservé (relu après) ; PUT avec
        `null` → effacé ; PUT avec un autre compte valide → remplacé ; mutation (champ remis en
        `Option<i64>`) → le premier test rougit.
  - [x] `bank_accounts_e2e.rs` : POST, PUT, PATCH vers non imputable → 400 `ACCOUNT_NOT_POSTABLE` ;
        PUT et PATCH inchangés sur un compte devenu non imputable → 200 ; **ordre** : compte bancaire
        inconnu + compte non imputable → 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (code actuel,
        `bank_accounts_e2e.rs:525`) ; version périmée + non imputable → 409 ; PUT `isPrimary:true` refusé
        pour non-imputabilité → l'ancien principal l'est toujours.
  - [x] `kesh-db` : `reconciliation_rules_repository.rs` et `bank_accounts_repository.rs` — la variante
        rendue par `create_in_tx` (règle), `update_in_tx`, `update_for_company`,
        `set_journal_account_id_for_company`.
  - [x] AC15 : les tests de non-régression nommés passent toujours (ne pas les réécrire).
  - [x] Mutation : retirer chaque garde une fois, constater le rouge, restaurer **et toucher le
        fichier**. Consigner la liste.
- [x] **T7 — Manuel** (AC17) : les deux encadrés (`:380`, `:390`), avec les cas qui échappent
      encore ; `admin-manual.tex` § *Configuration des comptes TVA* complété ; PDF régénérés, commités,
      contrôlés aplatis.
- [x] **T8 — Doc-comments, propagation, CHANGELOG** (AC18)
  - [x] Les sites de l'AC18, dont `docs/api-external.md`.
  - [x] **Grep du symptôme** avant de déclarer fini (règle *Propagation post-patch*) :
        `grep -rnE "#427|#429|#521|ne vérifie pas cet indicateur|restent ouverts|ne l'exigent pas|regroupement ou de clôture" crates docs/manual/fr/*.tex docs/api-external.md frontend/src`
        — chaque commentaire ou passage qui annonce ces trous comme ouverts est mis à jour ; chaque
        occurrence de l'ancienne parenthèse (message, test qui l'asserte, manuel) passe à « de
        regroupement, de résultat ou de clôture » (C19). *(Les motifs du manuel du rapprochement —
        `auto-validate`, « atomique », « par facture » — sont greppés par la 15-5c.)*
- [x] **T9 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) — **même en
      cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate frontend complet ;
      **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par fichier contre
      `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 19 AC, 10 tâches T0–T9.)*

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
- Les écrans filtrent déjà : `AccountAutocomplete.svelte:200-207`
  (`frontend/src/lib/features/journal-entries/`, `a.active && a.postable`) sert les deux modales de
  rapprochement, qui lui passent une liste **déjà réduite aux classes 5, 6 et 7** par préfixe du numéro
  (`ManualMatchModal.svelte:65-69`, `TransactionSplitModal.svelte:71-73`) — le compte proposé est donc
  de classe 5, 6 ou 7, actif et imputable ; `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte:43-50`,
  `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:44-60`,
  `frontend/src/routes/(app)/bank-accounts/+page.svelte:78-85` et
  `frontend/src/routes/(app)/settings/invoicing/+page.svelte` filtrent `postable`. Les modales manuelle
  et ventilée affichent `e.message` (`ManualMatchModal.svelte:126-132`) : le refus 400 s'y lit en clair
  sans changement. **Ce qui change à l'écran** : l'affichage des erreurs de l'écran des règles (AC14 —
  sans lui, la bascule « Réactiver » de `RulesList.svelte:55-67` afficherait « [object Object] » pour le
  refus de l'AC8 b) et les trois surfaces du compte lié d'un compte bancaire (AC13). La liste des refus
  par lot reste en code brut jusqu'à la 15-5c (#492) — un `ACCOUNT_NOT_POSTABLE` s'y lira donc d'abord
  en code, ce qui est l'état de tous les autres codes aujourd'hui.

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C4** — exemption « inchangé » pour réglages, compte bancaire, PATCH de règle ; aucune pour les
  rapprochements ni pour l'acceptation par règle.
- **C5** — le compte d'un compte bancaire, pour sa seule postabilité ; le reste de #474 à la 15-6.
- **C6** — hors périmètre assumés : fiche article (D3), compte bancaire **à l'usage** (D-A0), rôles de
  comptes, avoir (D5-bis).
- **C3** — la forme du refus (variante dédiée, posée par la 15-5a, employée ici).
- **C7** — le découpage ; cette story est le rollout.
- **C9** — une règle périmée n'est plus proposée ; sa réactivation est refusée ; pas de migration.
- **C10** — contrôle « inchangé » dans la transaction, sous le verrou ; ordre des erreurs conservé.
- **C12** — `withCurrentAccount` sur les `<select>` du compte bancaire ; **corrigé par C26**.
- **C13** — ordre des causes (sa clé de détail est révisée par C16).
- **C14** — les `catch` de l'écran des règles et de la liste des propositions lisent `ApiError`.
- **C15** — 15-5c : libellés de `failed[]` (#492, ex-C8) et manuel du rapprochement (#519, ex-C11, et
  #481) sortent de cette story.
- **C16** — `details.rejected[{accountId, accountNumber}]`, au 400 et dans `failed[].details`.
- **C19** — « de regroupement, de résultat ou de clôture », y compris dans `validate_account_of`.
- **C23** — frontière des commentaires avec la 15-5a ; T0 refait après son merge.
- **C24** — `docs/api-external.md`.
- **C25** — #521 : compte créanciers absent du corps → préservé ; **révisé par C34** : le champ est
  exposé à l'écran par la 15-5d.
- **C26** — `BankAccountJournalLinkForm` reçoit la liste complète.
- **C29** — `NonPostableAccounts::details()` (15-5a), seul constructeur du JSON `rejected`.
- **C30** — la création de règle est gardée dans le dépôt, comme le PATCH.
- **C33** — la garde à l'usage des comptes de réglage (C27, C28, ex-AC20) sort vers la **15-5d**.

### La garde à l'usage et la doctrine D-A0

Section passée à la **15-5d** avec l'ancien AC20 (choix C33) — y compris la prémisse corrigée sur les
plans livrés (finding R4-4 : seuls `1100` et `2000` sont désignés d'office ; `2200` et `1171` sont
choisis par l'utilisateur).

### Hors périmètre, et écrit

- La réactivation d'une règle dont le compte est **archivé** — qu'il soit imputable ou non : comportement
  actuel conservé (acceptée ; la règle n'est de toute façon pas proposée, AC5). Le contrôle neuf ne
  refuse qu'un compte **actif** non imputable (`active && !postable`), seul cas où la 15-5a autorise
  `ACCOUNT_NOT_POSTABLE` ; même règle pour les lectures des dépôts du compte bancaire (AC12).
- Le **compte de produit par défaut** à l'usage (exemption D3-bis de la 16-1a, délibérée) et le
  **compte bancaire** à l'usage (D-A0) : angles morts écrits au manuel (AC17).
- La garde **à l'usage** des comptes de réglage et l'exposition de `defaultPayableAccountId` à l'écran
  des réglages : **15-5d** (C33, C34). Le message français en dur
  des refus de `PUT /company/invoice-settings` (limite écrite à l'AC10).
- Le libellé traduit des refus par lot et le manuel du rapprochement : **15-5c** (C15).
- Le renommage du paramètre `active_account_ids` de `kesh-reconciliation` (crate hors périmètre).

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/{bank_accounts,reconciliation_rules}.rs`,
`crates/kesh-api/src/routes/{reconciliation,reconciliation_rules,company_invoice_settings,bank_accounts,products}.rs`,
`crates/kesh-api/src/errors.rs` (commentaire), `crates/kesh-db/src/repositories/{invoice_settlements_write,journal_entries}.rs`
(commentaires), `frontend/src/lib/features/reconciliation/{ReconciliationProposals.svelte,ReconciliationProposals.test.ts}`,
`frontend/src/lib/features/reconciliation/rules/{RuleFormModal.svelte,RuleFormModal.test.ts,RulesList.svelte,RulesList.test.ts}`,
`frontend/src/routes/(app)/reconciliation/rules/+page.svelte` (+ test),
`frontend/src/routes/(app)/bank-accounts/+page.svelte` (+ test neuf `+page.test.ts`),
`frontend/tests/e2e/{payment-batches,supplier-invoices}.spec.ts`, tests
(`crates/kesh-api/tests/{reconciliation_manual_e2e,reconciliation_split_e2e,reconciliation_e2e,reconciliation_rules_e2e,bank_accounts_e2e}.rs`,
nouveau `company_invoice_settings_postable_e2e.rs`, `crates/kesh-db/tests/{reconciliation_rules_repository,bank_accounts_repository}.rs`),
`docs/manual/fr/user-manual.{tex,pdf}` (deux encadrés), `docs/manual/fr/admin-manual.{tex,pdf}` (comptes
TVA), `docs/api-external.md`, `CHANGELOG.md`.
**Aucune migration** (P1–P8 sans objet). **Aucune** clé i18n neuve (celle de la garde à l'usage est à
la 15-5d) ; le message de l'AC10 reste en français en dur, celui d'`ACCOUNT_NOT_POSTABLE` à la saisie est
posé par la 15-5a.

### Tests — ce qui rendrait un test vert sans rien prouver

- Asserter `400` sans le code : un refus pour une autre raison passerait. Asserter le code.
- Compte de test non imputable **et** d'un autre type, ou d'une autre société : le refus viendrait du
  type ou du tenant. Ne changer **que** `postable`.
- Fixtures des quatre `reconciliation_*_e2e.rs` : elles ne posent que `postable: true` — le compte non
  imputable doit être créé ou basculé explicitement.
- Le test « PATCH inchangé → 200 » doit renvoyer le compte **dans le corps** : un PATCH sans
  `counterpartyAccountId` ne prouve pas l'exemption.
- Le test #521 doit **relire** les réglages après un PUT sans le champ : un test qui ne regarde que le
  statut 200 passe aussi quand le champ a été effacé.

### References

- Issues : #427, #429 (et son commentaire), #474, #521, #271, #375 (24-5) ; #492, #519, #481 → 15-5c.
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

Claude Opus 5.5 (`bmad-dev-story`, worktree `kesh-15-5b`, branche `story/15-5b-gardes-surfaces-neuves`).

### Debug Log References

- **T0 — inventaires refaits sur `33116a29`** (15-5a mergée dans la branche). (a) La commande rend
  **20 lignes**, les mêmes sites qu'au sol, décalés par la 15-5a : `reconciliation.rs:1581/2083/2432/3153/3619`,
  `invoice_settlements_write.rs:224/507`, `supplier_invoices.rs:394/724`, `invoices.rs:2255`,
  `credit_notes.rs:548`, `opening_complement.rs:641`, `journal_entries.rs:163` (create), `:204`
  (délégateur), `:644` (ouverture), `:1666` (contre-passation), `:3806` (`mod tests`), et les trois
  homonymes `users.rs:32`, `supplier_invoices.rs:222`, `imported_supplier_invoices.rs:34`. **Aucun site
  neuf.** (b) `grep -rnE "pub [a-z_]*account_id" crates/kesh-api/src/routes/*.rs` rend **63 lignes**,
  mêmes fichiers ; seul `companies.rs:322` n'était pas nommé — champ **de réponse**
  (`BankAccountJson`), sans objet. `onboarding.rs:531` (`upsert_primary`) ne pose aucun compte
  comptable. **Aucun site non classé.**
- Premier passage des tests d'acceptation ventilée en `400` : les lignes de proposition exigent une
  `description` ; ajoutée aux corps de test (pas de changement de code).
- Gate ciblé intermédiaire (neuf binaires touchés) : 207/207 verts.

### Completion Notes List

- **Rapprochement (AC1–AC6)** : `post_manual` (étape 3 bis) et `post_split` (étape 5 bis) refusent un
  compte de contrepartie actif non imputable en `400 ACCOUNT_NOT_POSTABLE`, après le 404 prioritaire ;
  `accept_one_split` (étape d) et `accept_one_rule` (étape 5) le rendent en `failed[]` par
  `non_postable_failed_proposal`, qui lit le code par `DbError::error_code` et le détail par
  `NonPostableAccounts::details()` (C29) ; `get_proposals` ne passe à `first_matching_rule` que les
  comptes actifs **et** imputables ; `accept_one_invoice` porte son commentaire de classement.
- **Règles (AC7–AC9)** : garde **dans la transaction** des dépôts `create_in_tx` (C30) et `update_in_tx`
  (changement de compte ou réactivation, C9 ; exemption « inchangé », C4 ; avant le 409, C10). Doc
  d'ordre de `post_create` et `patch`, doc de module.
- **Réglages (AC10, AC11, AC19)** : `validate_account` reçoit la valeur en place et exige `postable` si
  elle change ; parenthèse du message corrigée (C19) ; `defaultPayableAccountId` en
  `Option<Option<i64>>` + `double_option` — absent préservé sans contrôle, `null` effacé, valeur validée.
- **Compte bancaire (AC12, AC13)** : `validate_journal_account_id(…, require_postable)` vrai à la
  création ; `update_for_company` et `set_journal_account_id_for_company` contrôlent sous le verrou,
  si la valeur change et n'est pas `NULL`. Écran : `withCurrentAccount` sur les deux `<select>`, liste
  complète passée à `BankAccountJournalLinkForm` (C26). Deux E2E de montage gagnent `&& a.postable`.
- **Contrôle partagé** : `accounts::ensure_postable_if_active_in_tx` (C-15-5b-3), refus seulement si
  `active && !postable`.
- **AC14** : helper `errorMessageOf` (C-15-5b-1) aux sept `catch`.
- **AC15** : `test_create_in_tx_auto_flow_allows_non_postable` et
  `reverse_succeeds_when_an_account_became_non_postable` non modifiés, verts au gate complet.
- **AC17** : deux encadrés réécrits (`user-manual.tex`, § *Rôles des comptes* et § *Les comptes de
  clôture*), avec les quatre cas qui échappent encore ; § *Configuration des comptes TVA* complété
  (`admin-manual.tex`). Le reste du manuel d'administration : aucune autre mention du compte lié d'un
  compte bancaire ni des six champs — contrôle sans objet. PDF régénérés (`make admin user`) et
  contrôlés aplatis, ligatures normalisées : les phrases levées (« ne vérifie pas cet indicateur »,
  « pourrait donc les viser », « Deux chemins font exception », « mais leurs écrans n'offrent pas »)
  à 0 occurrence ; les nouvelles (« Quatre cas échappent encore », « déjà lié », « ne la propose
  plus », « au moment où vous le désignez », « Cette protection ne se limite pas à la saisie
  manuelle ») à 1 ; admin : « pas un compte de regroupement, de résultat ou de clôture » à 1.
- **AC18** : doc-comments `products.rs` (D3), `invoice_settlements_write.rs`, `journal_entries.rs`
  (`create_in_tx`), `errors.rs` (`details()` au présent) ; le commentaire D3 d'`errors.rs` ne reprend
  pas l'argument du jumeau — inchangé. `docs/api-external.md` : § *Accepter des propositions* et ligne
  `ACCOUNT_NOT_POSTABLE` du § 10. `CHANGELOG.md` : rubrique **Corrigé** de `[0.13.0] — Non publié`.
  **Grep du symptôme (T8)** sur `crates`, les manuels FR, `api-external.md`, `frontend/src` : plus aucun
  passage n'annonce #427/#429/#521 ouverts ; aucune occurrence de « regroupement ou de clôture ».
- **Mutations (AC16) — 20, toutes rouges, chacune restaurée puis le fichier touché** (script
  `scratchpad/mutate.py`) : M1 `post_manual` ; M2 `post_split` ; M2b ordre 404/400 de `post_split` ;
  M3 `accept_one_split` ; M3b ordre manquant/non imputable ; M4 `accept_one_rule` ; M4b archivé
  prioritaire ; M5 filtre de `get_proposals` ; M6 `create_in_tx` (règle, e2e + dépôt) ; M7a changement
  de compte (dont l'ordre 400 avant 409) ; M7b réactivation ; M7c exemption ; M7d condition `active` ;
  M8 désignation des réglages ; M8b exemption des réglages ; M9 #521 (`None => None`, C-15-5b-4 — deux
  tests rougissent) ; M10 création du compte bancaire ; M11a `set_journal_account_id_for_company`
  (e2e + dépôt) ; M11b `update_for_company` (dont « l'ancien principal le reste ») ; M11c exemption du
  PUT. Frontend : le helper sans sa branche `ApiError` → les **7** tests AC14 rougissent ; la page
  bancaire revenue à `linkableAccounts` sur les `<select>` → le test « formulaire de modification »
  rougit ; `accounts={linkableAccounts}` → le test « formulaire de lien » rougit. Aucun résidu de
  mutation (diff grepé).
- **Gates, au dernier commit de code** (périmètre : arbre de travail de la branche avant ce commit) :
  base de dev remise à zéro (restart tmpfs + migrations + seed) ; `scripts/test-fast.sh` **vert —
  fmt, clippy `-D warnings`, nextest 2784 exécutés / 2784 passés, 4 ignorés** (2754 au gate de la
  15-5a : +30, recompté par `grep -c 'sqlx::test'` aux deux bornes `HEAD`/arbre : manuel +2, ventilé
  +2, `reconciliation_e2e` +2, règles +10, compte bancaire +5, réglages +5 (fichier neuf), dépôt des
  règles +2, dépôt bancaire +2). Frontend : `npm run check` 0 erreur (27 avertissements, aucun neuf —
  ceux des fichiers touchés sont les `state_referenced_locally` préexistants de `RuleFormModal`) ;
  `lint-i18n-ownership` PASS ; `test:unit` **107 fichiers, 979 tests** (969 avant : +10) ; `build` OK.
- **E2E complet** (montage du `CLAUDE.md` + `KESH_INBOX_DIR`, `KESH_DOCUMENTS_DIR`, `KESH_SMTP_*`,
  `/health` `smtpConfigured:true` ; `kesh_e2e` recréée et migrée ; run à 13:34 UTC, donc pas de
  KF-045) : **235 passés, 12 échecs, 19 ignorés**. Jugement fichier par fichier contre
  `docs/testing.md` § « Les échecs attendus » : 7 KF-029 (`mode-expert:26/:41`,
  `onboarding-path-b:65/:92`, `onboarding:57/:77/:150`) ; `sidebar-navigation:75` — échoue **rejoué
  seul** → KF-046 (#424) ; `invoices:715`, `journal-entries:258/:321/:357` — tous en timeout sur
  `locator('#username')`, signature de la **KF-053** (#478), **verts rejoués seuls**. ⚠️ Quatre
  pollués, plus que les « 1 à 2 » habituels, tous sur la même signature réseau. **Verdict : aucune
  régression** ; les specs touchées (`payment-batches`, `supplier-invoices`) et celles du
  rapprochement passent.
- Backend E2E arrêté ; base de dev remise à zéro en fin de travail.
- **Choix consignés** : C-15-5b-1 à C-15-5b-4.

### File List

- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/15-5b-gardes-surfaces-neuves.md`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `crates/kesh-api/src/routes/bank_accounts.rs`
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/src/routes/products.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/src/routes/reconciliation_rules.rs`
- `crates/kesh-api/tests/bank_accounts_e2e.rs`
- `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs` (neuf)
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs`
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs`
- `crates/kesh-api/tests/reconciliation_split_e2e.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/accounts.rs`
- `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/src/repositories/reconciliation_rules.rs`
- `crates/kesh-db/tests/bank_accounts_repository.rs`
- `crates/kesh-db/tests/reconciliation_rules_repository.rs`
- `docs/api-external.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte`, `ReconciliationProposals.test.ts`
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte`, `RuleFormModal.test.ts`
- `frontend/src/lib/features/reconciliation/rules/RulesList.svelte`, `RulesList.test.ts` (neuf)
- `frontend/src/lib/shared/utils/api-client.ts`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`, `bank-accounts-page.test.ts` (neuf)
- `frontend/src/routes/(app)/reconciliation/rules/+page.svelte`, `rules-page.test.ts` (neuf)
- `frontend/tests/e2e/payment-batches.spec.ts`, `frontend/tests/e2e/supplier-invoices.spec.ts`

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
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-5b-validate-prompt-p2.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P1, **F**
  adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×3 → P2 Opus ×2). **0 CRITICAL, 0 HIGH.**
  Bruts : R 2 MEDIUM + 4 LOW, F 4 MEDIUM + 6 LOW. Doublons inter-lentilles : F-9 = R-6 e, F-10 = R-5 →
  **6 MEDIUM et 8 LOW distincts** (R-6, six sous-points a–f, compté pour un). Trend : P1 (15-5 entière)
  7 MEDIUM / 8 LOW → P2 (15-5b seule) 6 MEDIUM / 8 LOW.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R-1 | MEDIUM | AC13/C12 : `BankAccountJournalLinkForm` ne protège rien, la page lui passe une liste déjà filtrée | AC13, T4 : liste complète à `:437` (C26) | **né de la remédiation P1** (C12) |
  | R-2 | MEDIUM | AC17/T8 : « Rapprochement → Règles d'affectation » faux, le menu est sous *Administration* | chemin du manuel conservé ; motif retiré du grep de T8 ; la section passe à la 15-5c | **né de la remédiation P1** (C11) |
  | F-1 | MEDIUM | AC11 : prémisse « le formulaire renvoie toutes les valeurs » fausse — `defaultPayableAccountId` jamais envoyé, effacé à chaque enregistrement | issue **#521** créée par l'orchestrateur ; AC11 corrigé, **AC19** neuf, T3, T6 (C25) | **distinct, d'origine** (la prémisse figurait dans la spec du 2026-10-08) |
  | F-2 | MEDIUM | manuel faux sur les flux gardés : rapprochement manuel, éclatement, bouton *Modifier*, FAQ | **15-5c** (C15), qui ferme #481 | **distinct, d'origine** (défauts du manuel antérieurs à la story) |
  | F-3 | MEDIUM | la 15-5b n'est pas un rollout strictement mécanique | **découpage : 15-5c** (C15) ; ce qui reste de non mécanique est écrit (§ *Une story de rollout*) | **né de la remédiation P1** (la qualification « rollout » vient du découpage C7) |
  | F-4 | MEDIUM | `docs/api-external.md` (codes de `failed[]`, table du § 10) omis | AC18, T8 ; 15-5a AC9 (C24) | **distinct, d'origine** |
  | R-3 | LOW | tests de priorité manquants (accept split et rule, 400 avant 409) | AC16, T6 | — |
  | R-4 | LOW | AC8 : `SELECT number, postable` sans `active` contredisait le hors-périmètre et la 15-5a | AC8, AC12, T2, T4 : `active && !postable` | — |
  | R-5 = F-10 | LOW | frontière des commentaires avec la 15-5a ; lignes décalées | AC18 désigne par contenu ; T0 après merge de la 15-5a (C23) | — |
  | R-6 | LOW | a) en-tête sans C3 ; b) tests d'acceptation `rule` dans `reconciliation_rules_e2e.rs` ; c) d) libellés et type de `details` ; e) = F-9 archiver = désactiver ; f) test de `rules/+page.svelte` | a) b) f) ici ; c) d) e) **15-5c** | — |
  | F-5 | LOW | « atteignables par clé d'API » faux pour `PUT /company/invoice-settings` (route d'administration) | § *Pourquoi maintenant* | — |
  | F-6 | LOW | refus de l'AC10 en français en dur | limite écrite à l'AC10 | — |
  | F-7 | LOW | borne exacte `sitesTotal` de `i18n-keys.test.ts` | **15-5c** (seule à ajouter des sites i18n) | — |
  | F-8 | LOW | E2E qui lient un compte d'actif sans regarder `postable` | T4 : `&& a.postable` aux deux specs | — |

  **Signalé par la 15-5a** (son finding R-2, choix C19) : la parenthèse de `validate_account_of` omet le
  compte de résultat → corrigée à l'AC10, grep à T8.
  **Signal de la règle de découpage** (sévérité MEDIUM → MEDIUM) : **constaté**. Selon l'amendement
  D5 : trois MEDIUM (F-1, F-2, F-4) sont **distincts et d'origine** — la revue trouve des défauts neufs,
  elle ne tourne pas en rond ; trois (R-1, R-2, F-3) sont **nés de la remédiation P1**, sans qu'aucun ne
  recycle un finding de P1 sous une autre forme. Le découpage décidé par l'orchestrateur (15-5c, C15)
  répond à F-3 (dispersion et non-mécanicité), pas au signal de sévérité seul. Déclaré au Project Lead
  par l'orchestrateur.
  **Décisions de l'orchestrateur** : C15, C16, C19, C23 à C26 pour cette fiche. Les mentions de #492,
  C8 et C11 dans l'entrée de P1 ci-dessus sont **historiques** : ces objets sont à la 15-5c.
  **Propagation post-patch** : `accountNumbers`, `regroupement ou de clôture`, `#492`, `#519`, `AC14`,
  `AC17`, `linkableAccounts`, `defaultPayableAccountId` grepés sur les fiches 15-5, 15-5a, 15-5b, 15-5c
  et le registre.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-5b-validate-prompt-p3.md` ; deux
  lentilles **Sonnet** en contexte frais : **R** chasseur de régressions de la remédiation P2
  (`92770300`), **F** adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×3 → P2 Opus ×2 → P3
  Sonnet ×2). **0 CRITICAL, 0 HIGH.** Bruts : R 0 MEDIUM + 4 LOW, F 2 MEDIUM + 3 LOW. Doublon
  inter-lentilles : F-4 = R3-3 → **2 MEDIUM et 6 LOW distincts**. Trend : P1 (15-5 entière) 7 MEDIUM /
  8 LOW → P2 6 MEDIUM / 8 LOW → **P3 2 MEDIUM / 6 LOW**.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | F-1 | MEDIUM | un réglage « inchangé » devenu non imputable reste utilisé **sans contrôle à l'usage** (flux #8 et #10, `enforce_postable = false`) : le défaut de #429 subsistait, l'inventaire (b) le disait traité | **AC20 neuf** : garde à l'usage de la créance, de la TVA due, des créanciers et de la TVA récupérable, sur le patron du compte d'arrondi ; variante `DesignatedAccountsNotPostable`, code `ACCOUNT_NOT_POSTABLE`, message qui renvoie aux paramètres ; AC11 nuancé, inventaires (a) et (b) corrigés, T3, T6 (un test par compte, mutation), Dev Notes (compatibilité D-A0 vérifiée) — choix **C27**, **C28** | **distinct, d'origine** (l'exemption C4 et l'inventaire datent de la création de la 15-5) |
  | F-2 | MEDIUM | AC17 : les encadrés `:380` / `:390` levés promettaient une protection absolue | AC17 : ils nomment ce qui échappe encore (compte bancaire déjà lié et à l'usage, règle « Active » sur compte non imputable, fiche article et produit par défaut, avoir) | **distinct, d'origine** |
  | R3-1 | LOW | `API_KEY_ADMIN_FORBIDDEN` cité à `lib.rs:207-211` (enregistrement de la route) | § *Pourquoi maintenant* : route `:207-211`, refus posé par `require_not_pat` `:337` | — |
  | R3-2 | LOW | AC18 : la section `[0.13.0] — Non publié` n'existe pas encore | « créée par la 15-5a ; la créer en tête si absente » | — |
  | R3-3 = F-4 | LOW | `details.rejected` construit à deux endroits | accesseur unique `NonPostableAccounts::details()` posé par la 15-5a, appelé par AC3, AC4, AC20 (C29) | — |
  | R3-4 | LOW | AC19 sans chemin complet ; T4 citait un test de page inexistant | chemin `crates/kesh-db/…/supplier_invoices.rs:361-362` ; `bank-accounts/+page.test.ts` **neuf** | — |
  | F-3 | LOW | `admin-manual.tex:2016-2025` (comptes TVA) muet sur « imputable », contrairement à ses voisins | AC17, T7 : une phrase ajoutée ; PDF d'administration régénéré | — |
  | F-5 | LOW | POST de règle gardé par le seul pré-vol, hors transaction | AC7 : garde dans `reconciliation_rules::create_in_tx`, comme le PATCH (C30) ; T2, T6 | — |

  **Signal de la règle de découpage** (sévérité MEDIUM → MEDIUM, P2 → P3) : **constaté**. Les deux
  MEDIUM sont **distincts** de ceux de P2 et **d'origine** — aucun recyclage, aucun né de la
  remédiation P2 : la revue trouve des défauts neufs (amendement D5), pas de découpage. ⚠️ La
  remédiation **élargit** la story (AC20 : trois dépôts, une variante, une clé i18n) et touche une
  **règle métier** — révision de la limite L2 de D-A0 (14-3b) pour quatre comptes : **une passe P4
  complète suit**. Déclaré au Project Lead par l'orchestrateur. **Décisions** : C27 à C30.
  **Propagation post-patch** : `inchangé`, `NON GARDÉ`, `construit depuis`, `validate_counterparty_account`,
  `Aucune clé i18n neuve`, `sans objet`, `19 AC`, `ROUNDING`, `D3-bis`, `L2` grepés sur les fiches 15-5,
  15-5a, 15-5b, 15-5c et le registre ; le décompte passe à **20 AC, 10 tâches T0–T9** (recompté).
- 2026-10-08 — **Passe de validation P4** (prompt versionné `15-5b-validate-prompt-p4.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P3
  (`007c4eb1`), **F** adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×3 → P2 Opus ×2 → P3
  Sonnet ×2 → P4 Opus ×2). **0 CRITICAL, 1 HIGH.** Bruts : R 1 HIGH + 1 MEDIUM + 5 LOW, F 1 HIGH +
  2 MEDIUM + 6 LOW. Doublons inter-lentilles : R4-1 = F4-1, R4-2 = F4-2, R4-3 = F4-4, R4-5 = F4-6 →
  **1 HIGH, 2 MEDIUM et 9 LOW distincts**. Trend : P1 (15-5 entière) 7 MEDIUM / 8 LOW → P2 6 MEDIUM /
  8 LOW → P3 2 MEDIUM / 6 LOW → **P4 1 HIGH / 2 MEDIUM / 9 LOW**.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R4-1 = F4-1 | HIGH | AC20 × C25 : le refus des créanciers à l'usage renvoie à un champ que l'écran n'a pas — saisie fournisseur et complétion d'import bloquées sans recours à l'écran | **15-5d** : `defaultPayableAccountId` exposé dans *Paramètres → Facturation* (C34, révise C25), avec la garde ; ici, AC19 conservé et renvoi à la 15-5d | **né de la remédiation P3** (C27 contre C25) |
  | R4-2 = F4-2 | MEDIUM | l'avoir écrit sur la créance et la TVA due **des réglages du moment**, sans garde ; inventaire (a) #11 et AC17 (iv) le disaient « snapshot de la facture » / « reprend les comptes de la facture » | inventaire (a) #11 corrigé ; AC15 et AC17 (iv) réécrits sur le vrai ; exemption délibérée posée et écrite par la **15-5d** (C35 : créance → 15-6a, #473, #523 ; TVA due → #525) | **d'origine** (#11 date de la création de la 15-5) ; le texte de l'AC17 (iv) est né de la remédiation P3 |
  | F4-3 | MEDIUM | AC20 n'est pas du rollout ; la story n'a plus de justification pour son ampleur | **découpage : 15-5d** (C33) — AC20, C27/C28, variante, clé i18n, tests, passages de manuel ; `closes #429` passe à la 15-5d, cette story fait `refs #429` | **né de la remédiation P3** |
  | R4-3 = F4-4 | LOW | prédicat « porte de la TVA » inconnaissable avant la génération | 15-5d AC1 : `total_vat > 0`, garde entre génération et `create_in_tx` | — |
  | R4-4 | LOW | « les plans livrés désignent d'office 1100, 2200, 2000 et 1171 » faux pour 2200 et 1171 | 15-5d, Dev Notes | — |
  | R4-5 = F4-6 | LOW | « désignez-y un sous-compte » ; message servi au Comptable | 15-5d AC2 (C36) | — |
  | R4-6 | LOW | doc de module de `reconciliation_rules.rs` : `:13-17` (AC18) contre `:9-17` (T2) | T2 aligné sur `:13-17` | — |
  | R4-7 | LOW | inventaire (a) #7 : `company_invoice_settings.rs:353` n'est pas une garde du solde du reste | `:353` retiré ; `:469` et `:499` nommés par leur fonction | — |
  | F4-5 | LOW | place de la garde à l'usage dans l'ordre des refus | 15-5d AC1 | — |
  | F4-7 | LOW | manuel : sections de la validation et de la saisie fournisseur non visées | 15-5d AC8 | — |
  | F4-8 | LOW | CHANGELOG : ligne d'action pour un changement qui peut bloquer après mise à jour | 15-5d AC9 | — |
  | F4-9 | LOW | inventaire (b) : `vat_report.rs:174` lit les comptes TVA sans écrire | inventaire (b) : mentionné | — |

  **Signal de la règle de découpage** (sévérité MEDIUM → **HIGH**, P3 → P4) : **constaté, et suivi**.
  Selon l'amendement D5, le HIGH (R4-1/F4-1) et F4-3 sont **nés de la remédiation P3** — un défaut qui
  naît du correctif précédent, c'est le recyclage que l'amendement désigne comme déclencheur — et ils
  se concentrent sur la seule règle métier ajoutée en P3 : **découpage en 15-5d** (C33). F4-2 est
  d'origine. Ce qui reste ici est le rollout tel que la § *Une story de rollout* le décrit. Déclaré au
  Project Lead par l'orchestrateur. **Décisions de l'orchestrateur** : C33 à C35 ; C36 ajouté pendant la
  remédiation (15-5d). Les mentions de l'AC20, de C27 et de C28 dans les entrées P3 ci-dessus sont
  **historiques** : ces objets sont à la 15-5d.
  **Propagation post-patch** : `AC20`, `C27`, `C28`, `DesignatedAccountsNotPostable`,
  `designated-account`, `closes #429`, `n'est pas exposé`, `reprend les comptes`, `snapshot de la
  facture`, `:353`, `:9-17`, `20 AC`, `invoices_validate_vat` grepés sur les fiches 15-5, 15-5a (lecture
  seule, aucune occurrence de ces motifs), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml`. Décompte :
  **19 AC (AC1–AC19), 10 tâches T0–T9** (recompté). **Une passe P5 suit** (un HIGH et des MEDIUM en P4)
  — sur le périmètre réduit, elle peut être ciblée sur ce commit de remédiation.
- 2026-10-08 — **Passe de validation P5, ciblée** (prompt versionné `15-5b-validate-prompt-p5-ciblee.md` ;
  une lentille **Haiku**, contexte frais : chasseur de régressions braqué sur le seul commit de la
  remédiation P4, `67c31c95` ; passe ciblée de fin de boucle, décision D6). Rapport :
  `target/gate-logs/15-5b-p5-ciblee.md`. **Rapporté : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW.**
  - Le MEDIUM (« motif `20 AC` anachronique dans la propagation post-patch de la P4, à changer en
    `19 AC` ») est **réfuté par l'orchestrateur** : la ligne visée énumère les motifs **grepés** par la
    propagation post-patch ; `20 AC` y est la **valeur ancienne recherchée** — le décompte périmé que la
    remédiation P4 devait faire disparaître —, pas un décompte de la fiche. Le remplacer par `19 AC`
    rendrait la ligne fausse. Le décompte de la fiche, lui, est bien **19 AC, 10 tâches T0–T9**.
  - **Retenu : 0 au-dessus de LOW.** La remédiation P4 ne touche aucune ligne de code de production
    (fiche seule, aucun code écrit) : **boucle de validation close.**
  - Axes déclarés par la lentille : contradictions AC / tâches / Dev Notes / registre, fiches sœurs,
    numéros de ligne et chemins, décomptes, propagation post-patch, issues fermées et citées, découpage
    reflété ; aucun axe déclaré non exercé (passe ciblée, une lentille).

  **Trend complet de la fiche** (les deux premières lignes portent sur la 15-5 avant découpage) :

  | passe | modèle(s) | périmètre | bilan retenu |
  |---|---|---|---|
  | P1 | Sonnet ×3 | 15-5 entière | 0 C / 0 H / 7 MEDIUM / 8 LOW |
  | P2 | Opus ×2 | 15-5b | 0 C / 0 H / 6 MEDIUM / 8 LOW |
  | P3 | Sonnet ×2 | 15-5b | 0 C / 0 H / 2 MEDIUM / 6 LOW |
  | P4 | Opus ×2 | 15-5b | 0 C / 1 HIGH / 2 MEDIUM / 9 LOW → découpage 15-5d |
  | P5 ciblée | Haiku ×1 | commit `67c31c95` | 0 C / 0 H / 0 MEDIUM (1 rapporté, réfuté) / 0 LOW |

  Décompte inchangé : **19 AC (AC1–AC19), 10 tâches T0–T9** (recompté). Fiche prête pour le
  développement, après le merge de la 15-5a.
- 2026-10-08 — **Développement** (`bmad-dev-story`, Claude Opus 5.5). T0 refait : aucun site neuf. Les
  19 AC implémentés ; 30 tests backend et 10 tests Vitest neufs ; 20 mutations backend et 3 frontend,
  toutes rouges. Gate complet backend 2784/2784, frontend 979/979, E2E 235 passés / 12 échecs tous
  attendus (7 KF-029, KF-046, 4 KF-053 verts rejoués seuls). Choix C-15-5b-1 à C-15-5b-4. Statut
  `review`.
- 2026-10-08 — **Revue de code close** (`bmad-code-review`, Claude Sonnet 5.5, trois lentilles en contexte
  frais : Blind Hunter B, Edge Case Hunter E, Acceptance Auditor A ; diff `33116a29..c0aee2d3` ; prompt
  `15-5b-review-prompt-p1.md`, rapports `target/gate-logs/15-5b-review-p1-{B,E,A}.md`). Trend : **P1 : 0
  CRITICAL / 0 HIGH / 0 MEDIUM, 13 LOW** (B 6, E 4, A 3) — critère d'arrêt atteint dès la première passe
  (« uniquement des LOW »). Sort de chaque LOW :
  - **A-1 = E1** (`docs/api-external.md`, séparateur manquant avant `POST /supplier-invoices/{id}/pay`) :
    **corrigé** (virgule rétablie, une seule occurrence).
  - **B1 / E4** (codes bruts de `failed[]` affichés tels quels, `details.rejected` ignoré) : accepté,
    attribué à la **15-5c (#492)**.
  - **B2** (duplication du contrôle ; `errorMessageOf` non repris dans deux modales) : accepté, **dette
    LOW** — le corriger touche la production.
  - **B3 / A-3** (motif `e instanceof Error ? e.message : String(e)` encore présent, dont
    `settings/+page.svelte:236/:260`) : accepté, hors module, écarté par C-15-5b-1 ; suivi **#520**.
  - **B4 / E3** (contrôles de postabilité lus sans verrou, fenêtre étroite, création de compte bancaire
    hors transaction) : accepté, **même dette que #522**, consignée au doc-comment du helper.
  - **B5** (variable `active_account_ids` devenue « actifs et imputables ») et **B6 / C10** (400 avant 409,
    documenté) : acceptés, nit / comportement voulu.
  - **E2** (réactivation d'une règle dont le compte est archivé non refusée) : accepté, hors périmètre de
    l'AC8, comportement antérieur inchangé.
  - **A-2** (codes de `failed[]` dits en prose, non en liste) : accepté, conforme dans le fond.
  Axes exercés : chaque AC contre code et noms de tests, recompte des tests (+30 backend, +10 frontend,
  conformes), PDF aplatis, grep de résidus, inventaire du motif `String(e)`. Non exercés : exécution des
  tests et des mutations (revue en lecture seule). Choix C-15-5b-5. Statut `done`.
