# Story 15.5 : Gardes de postabilité côté serveur — réconciliation, règles, réglages de facturation, compte bancaire

Status: ready-for-dev

<!-- Spécification produite le 2026-10-08 en autonomie (relance de l'Epic 15, choix C1/C2).
     Choix tranchés pendant la spécification : C3 à C6 de
     `epic-15-choix-autonomes.md`. Validation : à lancer par l'orchestrateur, en contexte frais. -->

**Issues** : ferme **#427** (P1) et **#429** (P1, y compris son commentaire sur le message trompeur).
Voisin repris en partie : **#474** — seule la *postabilité* du compte comptable d'un compte bancaire
est traitée ici ; le fond de #474 (le compte débiteurs comme contrepartie de règlement, et comme
compte d'un compte bancaire) appartient à la **15-6** (choix C5).

## Story

En tant que **comptable ou administrateur d'une société tenue dans Kesh**,
je veux que **le serveur refuse un compte non imputable partout où le client désigne un compte qui
recevra une écriture** — et qu'il le dise sous son vrai nom,
afin qu'**aucune écriture ne puisse atterrir sur un compte de regroupement ou de clôture**, quel que
soit le client qui appelle (écran, clé d'API, intégration), et que le refus soit actionnable.

### Pourquoi maintenant

La doctrine posée par la Story 24-5 — *une garde serveur ne se déduit pas d'un filtre d'écran* — a
été appliquée à trois chemins (règlement client par compte interne, création et règlement de facture
fournisseur). Elle laisse **ouverts** les chemins de #427 et #429, que le manuel utilisateur avoue
en toutes lettres (`docs/manual/fr/user-manual.tex:380` et `:390`). Les écrans filtrent ; ces routes
sont atteignables par clé d'API (`actor_api_key_id` en atteste), et rien ne contrôle le compte côté
serveur. Conséquence concrète (#427) : un rapprochement routé sur **9000 Bilan d'ouverture** ou sur un
compte de regroupement fausse le résultat de l'exercice et le report à nouveau — le défaut même que
la 24-5 a fermé côté saisie manuelle.

## Acceptance Criteria

### Le refus, sous son vrai nom

1. **AC1 — Une variante dédiée.** `kesh-db` porte une variante `DbError::AccountsNotPostable(Vec<String>)`
   (numéros des comptes refusés, triés, dédupliqués), dont `error_code()` rend
   `"ACCOUNT_NOT_POSTABLE"`. `kesh-api` la mappe en **HTTP 400**, code `ACCOUNT_NOT_POSTABLE`, message
   traduit par une clé neuve `error-account-not-postable` (variable `$numbers`) dans les **quatre**
   locales — FR : « Le compte { $numbers } n'est pas imputable (compte de regroupement ou de
   clôture) : choisissez un compte imputable. » (formulation exacte au choix du dev, mais elle **nomme
   la cause** et **le ou les comptes**, et ne dit ni « archivé » ni « invalide »).
2. **AC2 — La saisie manuelle nomme la cause.** `validate_lines_accounts_in_tx`
   (`crates/kesh-db/src/repositories/journal_entries.rs:84`) distingue désormais les deux refus :
   compte **inconnu, d'une autre société ou archivé** → `InactiveOrInvalidAccounts` (inchangé) ;
   compte connu, actif, mais **non imputable et non exempté** → `AccountsNotPostable(numéros)`. Quand
   les deux défauts coexistent, le plus bloquant gagne : `InactiveOrInvalidAccounts` (patron
   « une seule raison, la plus bloquante d'abord » de `validate_line_revenue_accounts_in_tx`,
   `invoices.rs:631-643`). Le chemin `enforce_postable = false` est **strictement inchangé**.
3. **AC3 — Les trois gardes de la 24-5 nomment la cause.** Les gardes de
   `invoice_settlements_write.rs:156-167` (règlement client, compte interne) et de
   `supplier_invoices.rs:336-347` (compte de charge) et `:639-650` (règlement fournisseur, compte
   interne) rendent `AccountsNotPostable([numéro])` quand le compte est actif mais non imputable, et
   `InactiveOrInvalidAccounts` dans les autres cas (inconnu, archivé, mauvais type pour le compte de
   charge). Leur `SELECT` lit donc aussi `number`.

### Réconciliation (#427)

4. **AC4 — `POST /reconciliation/manual`** (`post_manual`, `reconciliation.rs:2942`) : un
   `counterpartyAccountId` actif, de la société, mais non imputable → **400 `ACCOUNT_NOT_POSTABLE`**,
   **rien n'est écrit** (aucune écriture, transaction bancaire toujours `pending`, aucun audit).
   Inconnu / cross-tenant / archivé → **404 `ACCOUNT_NOT_FOUND`**, inchangé (anti-énumération).
5. **AC5 — `POST /reconciliation/split`** (`post_split`, `:3362`) : si au moins une ligne vise un
   compte non imputable → **400 `ACCOUNT_NOT_POSTABLE`** nommant **tous** les comptes non imputables
   de la requête ; rien n'est écrit. Le refus 404 des comptes manquants reste prioritaire et inchangé.
6. **AC6 — `POST /reconciliation/accept`, proposition `split`** (`accept_one_split`, `:1829`) : une
   ligne sur un compte non imputable → la proposition tombe dans `failed[]` avec
   `errorCode = "ACCOUNT_NOT_POSTABLE"` et `details = { "accountIds": [...] }` ; **HTTP 200**, les
   autres propositions du lot sont traitées normalement (§ *Pattern batch — FailedProposal* du
   `CLAUDE.md`, aucune escalade en `AppError`).
7. **AC7 — `POST /reconciliation/accept`, proposition `rule`** (`accept_one_rule`, `:2207`) : si le
   compte de contrepartie de la règle est devenu non imputable, la proposition tombe dans `failed[]`
   avec `errorCode = "ACCOUNT_NOT_POSTABLE"` — **pas d'exemption** (choix C4 : une règle dont le
   compte a été scindé en sous-comptes est périmée ; poster sur le parent est exactement le défaut).
8. **AC8 — Une règle périmée n'est plus proposée.** `get_proposals` (`:449`) n'offre plus de
   candidat `rule` dont le compte de contrepartie est non imputable : l'ensemble
   `active_account_ids` (`:556`) passé à `kesh_reconciliation::first_matching_rule` ne retient que
   les comptes **actifs ET imputables**. (`accounts_info`, qui sert à l'affichage, peut rester sur les
   seuls comptes actifs.) La règle suivante qui correspond, s'il y en a une, est proposée à sa place.
9. **AC9 — `accept_one_invoice` reste tel quel**, et c'est écrit : ses comptes ne viennent pas du
   client (compte bancaire configuré, créance lue **sur l'écriture de vente** — `:1413-1450` —, compte
   d'arrondi des réglages déjà gardé `postable = TRUE` dans
   `company_invoice_settings.rs:469/499`). Un commentaire au site le dit, pour que la prochaine
   énumération ne le reclasse pas « oublié ».

### Règles de réconciliation (#427)

10. **AC10 — Création** (`POST /reconciliation/rules`, `reconciliation_rules.rs:248`) : compte de
    contrepartie non imputable → **400 `ACCOUNT_NOT_POSTABLE`**, aucune règle créée.
11. **AC11 — Modification** (`PATCH /reconciliation/rules/{id}`, `:345`) : un
    `counterpartyAccountId` **différent** du compte actuel de la règle et non imputable → 400
    `ACCOUNT_NOT_POSTABLE`. **Exemption** (choix C4) : un `counterpartyAccountId` **égal** au compte
    actuel n'est pas re-contrôlé sur `postable` (on peut renommer ou désactiver une règle dont le
    compte est devenu non imputable) ; il reste contrôlé sur existence/société/`active` comme
    aujourd'hui. Un PATCH sans `counterpartyAccountId` est inchangé.

### Réglages de facturation (#429)

12. **AC12 — Les six champs historiques** de `PUT /company/invoice-settings`
    (`company_invoice_settings.rs:246`) — `defaultReceivableAccountId`, `defaultRevenueAccountId`,
    `defaultVatPayableAccountId`, `defaultVatRecoverableAccountId`, `defaultVatDecompteAccountId`,
    `defaultPayableAccountId` — refusent un compte **non imputable** quand la valeur **change** par
    rapport au réglage en place : 400 `VALIDATION_ERROR`, message « {champ} : compte non imputable
    (compte de regroupement ou de clôture) » — **le message déjà émis** par `validate_account_of`
    (`:190-194`) pour les comptes désignés (choix C3 : les refus de cette route sont tous des
    `Validation` nommant le champ ; on n'en crée pas un second style).
13. **AC13 — Exemption « inchangé »** : une valeur **égale** à celle en place n'est pas re-contrôlée
    sur `postable` — patron exact de `resolve_designated_account` (`:200-229`) et de la décision D4
    des articles. Sans elle, un compte de réglage devenu non imputable après coup (ajout d'un
    sous-compte, règle 14-3a) bloquerait **tout** enregistrement des réglages, sur un champ que
    l'utilisateur n'a pas touché — le formulaire renvoie toutes les valeurs (`withCurrentAccount`,
    `settings/invoicing/+page.svelte:85-111`, issu de #271). Les contrôles existants (existence,
    société, `active`, type) restent **inconditionnels**, comme aujourd'hui.

### Compte comptable d'un compte bancaire (voisin de #474)

14. **AC14 — `validate_journal_account_id`** (`bank_accounts.rs:260`) refuse un compte non imputable
    → 400 `ACCOUNT_NOT_POSTABLE`, aux **trois** sites : création (`create_bank_account`, `:384`),
    remplacement (`update_bank_account`, `:498`), lien (`patch_bank_account_journal_link`, `:717`).
    **Exemption « inchangé »** pour les deux derniers : un `journalAccountId` égal à la valeur en place
    n'est pas re-contrôlé sur `postable`. Le compte débiteurs, lui, reste accepté ici : c'est la 15-6.

### Ce qui ne change pas, et qui doit être prouvé

15. **AC15 — Non-régression des flux automatiques** : `create_in_tx(…, false)` continue d'accepter
    un compte de **configuration** devenu non imputable (test existant
    `test_create_in_tx_auto_flow_allows_non_postable`, `journal_entries.rs:~3684`, toujours vert) ;
    la contre-passation (`reverse_in_tx_inner`, `:1656`) et l'annulation d'un rapprochement restent
    possibles sur un compte devenu non imputable (test existant
    `reverse_succeeds_when_an_account_became_non_postable`, `journal_entry_reversal_e2e.rs:1015`).
    Un compte bancaire dont le compte comptable est **devenu** non imputable continue de servir aux
    rapprochements (décision D-A0 de la 14-3b, choix C6).
16. **AC16 — Tests négatifs « compte non imputable refusé » sur CHAQUE surface** des AC 2 à 14, dont
    le compte de test ne diffère d'un compte accepté **que par `postable`** (même société, actif, bon
    type) et dont l'assertion porte sur le **code** (`ACCOUNT_NOT_POSTABLE` / le message du champ),
    pas sur le seul statut 400 — sans quoi un refus pour une autre raison passerait pour la garde.
    Chaque surface à exemption porte **aussi** son test positif « inchangé → accepté ». Et chaque test
    négatif est **muté** une fois (garde retirée → le test rougit), consigné au Dev Agent Record.
17. **AC17 — Le manuel dit vrai.** La réserve des deux paragraphes du manuel utilisateur
    (`user-manual.tex:380` et `:390`) est **levée** : le rapprochement bancaire (y compris ses règles)
    et les réglages de facturation refusent désormais un compte non imputable ; le compte d'un compte
    bancaire aussi. Le PDF est régénéré et **contrôlé aplati** (`pdftotext … | tr '\n' ' ' | tr -s ' '`).

## Inventaire des sites — l'ensemble clos des points où un compte reçoit une écriture

Règle *« Inventorier les sites NON RÉSOLUS »* du `CLAUDE.md`. Deux énumérations, faites au sol le
2026-10-08 sur `1920381e` ; le dev les **refait** au début de l'implémentation (les lignes bougent)
et la passe de revue les **refait** à son tour.

### (a) Tous les appels de `journal_entries::create_in_tx` / `create_in_tx_inner` hors tests

Commande (le motif multi-ligne rate les appels dont les arguments sont commentés ; un petit script
qui compte les parenthèses a été utilisé — le dev peut le refaire avec `grep -n -A12`) :

```sh
grep -rnE "journal_entries::create_in_tx\(|create_in_tx_inner\(|[^:_]create_in_tx\(" crates/*/src --include=*.rs
```

| # | site | 5ᵉ arg. | d'où vient le compte | verdict avant 15-5 | après 15-5 |
|---|---|---|---|---|---|
| 1 | `reconciliation.rs:1581` `accept_one_invoice` | `false` | banque (config), créance (écriture de vente), arrondi (réglage gardé) | aucun compte client | **inchangé**, commenté (AC9) |
| 2 | `reconciliation.rs:2083` `accept_one_split` | `false` | **client** (`splits[].counterpartyAccountId`) | **NON GARDÉ** (`active` seul, `:1951-1981`) | gardé (AC6) |
| 3 | `reconciliation.rs:2432` `accept_one_rule` | `false` | règle (choisie par le client) | **NON GARDÉ** (`active` seul, `:2289-2318`) | gardé (AC7, AC8) |
| 4 | `reconciliation.rs:3153` `post_manual` | `false` | **client** | **NON GARDÉ** (`Some(a) if a.active`, `:3028`) | gardé (AC4) |
| 5 | `reconciliation.rs:3619` `post_split` | `false` | **client** | **NON GARDÉ** (`Some(a) if a.active`, `:3488`) | gardé (AC5) |
| 6 | `invoice_settlements_write.rs:214` règlement client | `false` | client (compte interne) / banque (config) | gardé 24-5, **message trompeur** | message juste (AC3) ; banque : AC14 |
| 7 | `invoice_settlements_write.rs:497` solde du reste | `false` | réglages désignés | gardé (`company_invoice_settings.rs:353/499`, `postable = TRUE`) | inchangé |
| 8 | `supplier_invoices.rs:372` facture fournisseur | `false` | client (charge) + réglages (créanciers, TVA récup.) | charge gardée 24-5 (message trompeur) ; réglages **NON GARDÉS** (#429) | AC3 + AC12 |
| 9 | `supplier_invoices.rs:693` règlement fournisseur | `false` | client (compte interne) / banque | gardé 24-5, message trompeur | AC3 ; banque : AC14 |
| 10 | `invoices.rs:2255` validation de facture | `false` | lignes (gardées `validate_line_revenue_accounts_in_tx`) + réglages (créance, TVA due, produit par défaut) | lignes gardées ; **réglages NON GARDÉS** (#429) | AC12 |
| 11 | `credit_notes.rs:548` avoir | `false` | snapshot de la facture (D5-bis) | déjà validés à l'émission | inchangé (angle mort assumé, C6) |
| 12 | `journal_entries.rs:1656` contre-passation | `false` | écriture d'origine | exemption voulue | inchangé (AC15) |
| 13 | `journal_entries.rs:153` `create` (saisie manuelle) | **`true`** | client | gardé 14-3b, **message trompeur** | message juste (AC2) |
| 14 | `journal_entries.rs:634` `create_opening_entry` | **`true`** | client | gardé | message juste (effet d'AC2) |
| 15 | `opening_complement.rs:641` complément de soldes | **`true`** | client + compte de report | gardé | message juste (effet d'AC2) |
| — | `journal_entries.rs:~3700` | `false` | — | dans `mod tests` | — |

### (b) Toutes les routes qui reçoivent un identifiant de compte du client

Commande : `grep -rnE "pub [a-z_]*account_id" crates/kesh-api/src/routes/*.rs` (DTO désérialisés).

| route / champ | écrit une écriture ? | verdict |
|---|---|---|
| `journal_entries.rs` `lines[].accountId` | oui (#13) | gardé ; message AC2 |
| `opening_balances.rs` `accountId` | oui (#14) | gardé ; message AC2 |
| `reconciliation.rs` manual / split / accept | oui (#2–#5) | **AC4–AC8** |
| `reconciliation_rules.rs` `counterpartyAccountId` (POST, PATCH) | indirectement (#3) | **AC10–AC11** |
| `company_invoice_settings.rs` six champs historiques | indirectement (#8, #10) | **AC12–AC13** |
| `company_invoice_settings.rs` arrondi, escompte, frais, pertes | indirectement (#7) | déjà gardés (`require_postable`, « si changé ») |
| `bank_accounts.rs` `journalAccountId` (POST, PUT, PATCH) | indirectement (#1, #6, #9, et tous les rapprochements) | **AC14** |
| `invoices.rs` `lines[].revenueAccountId` | oui (#10) | gardé (16-1a, `NotPostable` hors défaut société) |
| `invoices.rs` règlement `accountId` / `bankAccountId` | oui (#6) | gardé 24-5 ; message AC3 |
| `supplier_invoices.rs` `expenseAccountId`, règlement `accountId` | oui (#8, #9) | gardé 24-5 ; message AC3 |
| `imported_supplier_invoices.rs` `expenseAccountId` | oui, via `supplier_invoices::create_in_tx` (#8) | gardé en aval |
| `products.rs` `defaultRevenueAccountId` | non (recopié sur la ligne de facture, gardée) | **hors périmètre, délibéré (D3, choix C6)** — le doc-comment doit être amendé (T8) |
| `credit_notes.rs` `revenueAccountId` | non (champ de **réponse**) | sans objet |
| `payment_batches.rs`, `bank_imports.rs` `bankAccountId` | non (compte bancaire, pas compte du plan) | sans objet |
| `reports.rs` `accountIds` | non (filtre de lecture) | sans objet |
| `accounts.rs` (rôle d'un compte) → `insert_with_defaults_in_tx` | indirectement, une fois, à la création des réglages | **angle mort assumé (C6)** : choix de l'application par rôle (D-A0) ; tout changement ultérieur passe par le PUT des réglages, gardé par AC12 |

## Tasks / Subtasks

- [ ] **T0 — Refaire les deux inventaires** ci-dessus sur `HEAD` (lignes à jour), et signaler au
  Change Log tout site absent de ces tables. Un site neuf non classé bloque la story.
- [ ] **T1 — La variante d'erreur** (AC1)
  - [ ] `DbError::AccountsNotPostable(Vec<String>)` dans `crates/kesh-db/src/errors.rs` (doc-comment :
        pourquoi elle existe — commentaire de #429, précédent `RevenueAccountRejection::NotPostable`) ;
        `error_code()` → `"ACCOUNT_NOT_POSTABLE"`.
  - [ ] Mapping dans `crates/kesh-api/src/errors.rs` à côté de `InactiveOrInvalidAccounts` (`:3067`) :
        400, `ACCOUNT_NOT_POSTABLE`, `t_args("error-account-not-postable", …)` avec `$numbers`
        (numéros joints par « , »).
  - [ ] Clé `error-account-not-postable` dans les **4** `messages.ftl` (`fr-CH`, `de-CH`, `it-CH`,
        `en-CH`) ; `npm run lint-i18n-ownership` vert.
- [ ] **T2 — Saisie manuelle et gardes 24-5** (AC2, AC3)
  - [ ] `validate_lines_accounts_in_tx` : lire `id, number, active, postable` (sans clause
        `postable` dans le `WHERE`), décider en Rust : manquant/archivé → `InactiveOrInvalidAccounts` ;
        sinon, si `enforce_postable`, non imputable hors `exempt_ids` → `AccountsNotPostable`. La
        fonction reste rollback-agnostique ; mettre à jour son doc-comment.
  - [ ] Les trois `SELECT active, postable …` de `invoice_settlements_write.rs` et
        `supplier_invoices.rs` : lire `number`, rendre la bonne variante.
  - [ ] Mettre à jour les tests qui figeaient l'ancien code pour un compte **non imputable** :
        `journal_entries.rs` (`test_create_manual_rejects_non_postable_line`,
        `test_create_manual_rejects_result_account`), `reports_e2e.rs:~1953` (l'assertion « aucun code
        neuf » de l'AC 13 de la 24-5 est **délibérément** remplacée — le dire dans son commentaire),
        et tout autre site trouvé par
        `grep -rn "InactiveOrInvalidAccounts\|INACTIVE_OR_INVALID_ACCOUNTS" crates/*/tests crates/*/src frontend`.
        Trier chaque occurrence : non imputable → nouveau code ; inconnu/archivé/cross-tenant → inchangé.
  - [ ] `JournalEntryForm.svelte:178` : ajouter `case 'ACCOUNT_NOT_POSTABLE':` au groupe qui affiche
        `err.message`.
- [ ] **T3 — Réconciliation** (AC4–AC9)
  - [ ] `post_manual` étape 3 : après `Some(a) if a.active`, `!a.postable` →
        `AppError::Database(DbError::AccountsNotPostable(vec![a.number]))` (ou variante `AppError`
        équivalente — même code et même message).
  - [ ] `post_split` étape 5 : collecter les non imputables **après** le contrôle des manquants
        (le 404 reste prioritaire) ; un seul refus nommant tous les comptes.
  - [ ] `accept_one_split` étape d : `SELECT active, postable` ; `FailedProposal
        { error_code: "ACCOUNT_NOT_POSTABLE", details: { "accountIds": [...] } }`.
  - [ ] `accept_one_rule` étape 5 : `SELECT active, postable, name` ; même `FailedProposal`.
  - [ ] `get_proposals` : `active_account_ids` = actifs **et** imputables (requête dédiée ou
        colonne ajoutée à `accounts_info_rows`).
  - [ ] Commentaire de classement à `accept_one_invoice` (AC9).
  - [ ] Mettre à jour la doc d'ordre de validation de `post_manual` (`:2932-2939`) et de `post_split`.
- [ ] **T4 — Règles** (AC10, AC11) : `validate_counterparty_account` prend un paramètre
      `current: Option<i64>` (création : `None`) ; `postable` contrôlé si `Some(id) != current`. Dans
      `patch`, la règle courante est déjà lue (`:369-372`) — la lire **avant** la validation du compte.
      Mettre à jour le doc-comment de module (`:13-17`).
- [ ] **T5 — Réglages de facturation** (AC12, AC13) : les six appels à `validate_account` passent
      `require_postable = (req.champ != current.champ)` ; `validate_account` reçoit ce paramètre (ou
      appeler directement `validate_account_of`). Réécrire le doc-comment de `validate_account_of`
      (`:150-158`, « Les champs historiques ne l'exigent pas » devient faux) et l'en-tête du module.
- [ ] **T6 — Compte bancaire** (AC14) : `validate_journal_account_id` reçoit `current: Option<i64>`
      (`None` à la création ; valeur en base pour PUT et PATCH, lue par
      `bank_accounts::find_by_id_for_company` **avant** la validation) ; non imputable et changé →
      `DbError::AccountsNotPostable`. Doc-comment à jour.
- [ ] **T7 — Tests** (AC15, AC16) — helper `set_account_not_postable` sur le patron de
      `products_revenue_account_e2e.rs:254` (UPDATE direct, `version + 1`).
  - [ ] `reconciliation_manual_e2e.rs` : non imputable → 400 `ACCOUNT_NOT_POSTABLE`, aucune écriture,
        transaction toujours `pending`.
  - [ ] `reconciliation_split_e2e.rs` : une ligne non imputable sur trois → 400, les comptes nommés,
        rien d'écrit.
  - [ ] `reconciliation_e2e.rs` (accept) : proposition split non imputable → 200, `failed[]` avec
        le code ; proposition valide du même lot → `accepted[]` ; proposition rule dont le compte est
        devenu non imputable → `failed[]` ; `get_proposals` ne propose plus cette règle.
  - [ ] `reconciliation_rules_e2e.rs` : POST non imputable → 400 ; PATCH vers non imputable → 400 ;
        PATCH en renvoyant le **même** compte devenu non imputable → 200 ; PATCH sans compte → 200.
  - [ ] Réglages de facturation — fichier neuf `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`
        (montage : `idor_multi_tenant_e2e.rs:751`, `create_seeded_company`) : pour **chacun** des six
        champs, changement vers un compte non imputable du bon type → 400 et message du champ ;
        PUT renvoyant un compte **déjà en place** devenu non imputable → 200.
  - [ ] `bank_accounts_e2e.rs` : POST, PUT, PATCH vers non imputable → 400 `ACCOUNT_NOT_POSTABLE` ;
        PUT et PATCH inchangés sur un compte devenu non imputable → 200.
  - [ ] `kesh-db` : un test par garde de l'AC2/AC3 asserte **la variante** (`AccountsNotPostable`
        pour non imputable, `InactiveOrInvalidAccounts` pour archivé), y compris la priorité quand
        les deux défauts coexistent.
  - [ ] AC15 : vérifier que les deux tests de non-régression nommés passent toujours (ne pas les
        réécrire).
  - [ ] Mutation : retirer chaque garde une fois, constater le rouge, restaurer **et toucher le
        fichier** (mémoire « mutation restaurée, binaire périmé »). Consigner la liste.
- [ ] **T8 — Propagation et documentation** (AC17)
  - [ ] `docs/manual/fr/user-manual.tex:380` et `:390` : lever la réserve (plus d'« exception » ni
        de « une intégration … pourrait les viser ») ; dire que le rapprochement (manuel, ventilation,
        règles) et les réglages de facturation refusent un compte non imputable, et qu'un compte
        bancaire ne peut être lié qu'à un compte imputable. Vérifier la section du rapprochement et
        celle des règles pour une phrase à ajuster.
  - [ ] `docs/manual/fr/admin-manual.tex` : sections *Paramètres → Facturation* (comptes par défaut)
        et *Comptes bancaires* (compte lié) — si elles énumèrent les critères d'un compte acceptable,
        y ajouter « imputable ».
  - [ ] Régénérer les PDF (`latexmk -xelatex` dans `docs/manual/fr/`), les commiter, contrôler le
        PDF aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "…"`.
  - [ ] `crates/kesh-api/src/routes/products.rs:303-307` : le doc-comment de D3 justifie l'exclusion
        de `postable` par « le code jumeau `company_invoice_settings::validate_account` ne le contrôle
        pas davantage » — **devenu faux**. Réécrire la justification (la garde vit sur la ligne de
        facture, `NotPostable` ; l'article ne poste rien), sans changer le comportement (choix C6).
        Idem pour le commentaire de `errors.rs:~1199`.
  - [ ] Doc-comments de `journal_entries.rs:70-77` et `:182-186` (liste des flux `false`) : y dire
        que les flux de réconciliation gardent désormais le compte **client** en amont.
  - [ ] **Grep du symptôme** avant de déclarer fini (règle *Propagation post-patch*) :
        `grep -rnE "#427|#429|ne vérifie pas cet indicateur|archivés ou invalides" crates docs frontend/src`
        — chaque commentaire qui annonce ces trous comme ouverts (p. ex. `invoice_settlements_write.rs:153-154`
        « Les trois flux de réconciliation, eux, restent ouverts et sont suivis par #427 ») est mis à jour.
  - [ ] `CHANGELOG.md` : section `## [Unreleased]`, rubrique *Fixed* (refus serveur des comptes non
        imputables sur le rapprochement, ses règles, les réglages de facturation et le compte d'un
        compte bancaire ; refus nommés « non imputable »), et *Changed* (le code `ACCOUNT_NOT_POSTABLE`
        remplace `INACTIVE_OR_INVALID_ACCOUNTS` pour ce motif — **visible des intégrations par clé
        d'API**).
- [ ] **T9 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate
      frontend complet ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé
      fichier par fichier contre `docs/testing.md` § « Les échecs attendus ».

## Dev Notes

### Le modèle à suivre, et ce qu'il n'est pas

- **`validate_lines_accounts_in_tx`** (14-3b, D-A0/D-A1) est le modèle de la garde : `postable`
  contrôlé quand le compte vient du client, exemption pour un compte **déjà référencé**. Depuis le gel
  des écritures, son paramètre `exempt_ids` n'est plus passé qu'à vide (`:340`) : l'exemption
  « déjà référencé » n'a plus d'appelant. Sur les surfaces de cette story, elle prend la forme
  **« contrôlé seulement si la valeur change »** — c'est celle de `resolve_designated_account`
  (`company_invoice_settings.rs:200`) et de la décision D4 des articles (`products.rs:~413`).
- **Ne PAS passer `enforce_postable = true`** à `create_in_tx` dans `post_manual` / `post_split` pour
  « faire simple » : les lignes contiennent aussi le **compte bancaire**, compte de configuration que
  D-A0 autorise à être devenu non imputable (ajout d'un sous-compte sous 1020). La garde se place sur
  le seul compte **client**, en amont. Même raison pour les flux `accept_*`.
- **Course résiduelle** : `post_manual` et `post_split` contrôlent le compte **hors** transaction
  (lecture sur le pool, comme le contrôle `active` actuel). Un compte rendu non imputable entre le
  contrôle et l'insertion passe — même dette LOW acceptée qu'en 19-3/19-4 et que pour `active`. Les
  flux `accept_*` lisent **dans** la transaction, sous le verrou de compte.

### Ce qui doit être préservé

- Anti-énumération (KF-002) : inconnu / cross-tenant / archivé restent en **404 `ACCOUNT_NOT_FOUND`**
  sur la réconciliation et les règles. `ACCOUNT_NOT_POSTABLE` n'est émis que pour un compte de la
  société, actif — il ne révèle rien.
- Le pattern batch : aucune erreur par proposition n'escalade en `AppError` dans `accept_one_*`.
- Les écrans filtrent déjà : `AccountAutocomplete.svelte:200-207` (`a.active && a.postable`) sert les
  deux modales de réconciliation ; `RuleFormModal.svelte:44-50`, `BankAccountJournalLinkForm.svelte:44-53`
  et `settings/invoicing/+page.svelte` filtrent `postable`. **Aucun changement d'écran n'est requis**
  — mais le dev le vérifie, pour ne pas laisser un écran proposer ce que le serveur refuse.
- `insert_with_defaults_in_tx` / `seed_demo` / `onboarding.rs:194,727` reconnaissent
  `InactiveOrInvalidAccounts` pour **leurs** refus (comptes de rôle absents) : ces chemins ne passent
  pas par les gardes modifiées, mais le vérifier (`kesh-seed/src/lib.rs:208`).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C3** — forme du refus : variante `kesh-db` dédiée + code `ACCOUNT_NOT_POSTABLE`, étendue à la
  saisie manuelle et aux gardes 24-5 ; les réglages de facturation gardent leur style `Validation`.
- **C4** — exemption « inchangé » pour réglages, compte bancaire, PATCH de règle ; aucune exemption
  pour les rapprochements ni pour l'acceptation par règle.
- **C5** — le compte d'un compte bancaire entre dans la story pour sa seule postabilité ; le reste de
  #474 va à la 15-6.
- **C6** — hors périmètre assumés : fiche article (D3), compte bancaire **à l'usage** (D-A0), rôles de
  comptes, avoir (D5-bis).

### Fichiers touchés (prévision)

`crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/{journal_entries,invoice_settlements_write,supplier_invoices}.rs`,
`crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/{reconciliation,reconciliation_rules,company_invoice_settings,bank_accounts,products}.rs`,
`crates/kesh-i18n/locales/*/messages.ftl`, `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`,
tests (`crates/kesh-api/tests/{reconciliation_manual_e2e,reconciliation_split_e2e,reconciliation_e2e,reconciliation_rules_e2e,bank_accounts_e2e,reports_e2e}.rs`,
nouveau `company_invoice_settings_postable_e2e.rs`, tests `kesh-db`), `docs/manual/fr/{user,admin}-manual.{tex,pdf}`,
`CHANGELOG.md`. **Aucune migration** (P1–P8 sans objet). Six modules de premier niveau touchés côté
code : la règle de splitting préventif (> 5 modules) est **à la limite** ; la story reste unique car
les gardes sont mécaniques et calquées sur un modèle existant — si la validation lève une
non-convergence réelle, découper en 15-5a (variante d'erreur + saisie manuelle + 24-5) et 15-5b
(surfaces neuves).

### Tests — ce qui rendrait un test vert sans rien prouver

- Asserter `400` sans le code : un refus `ACCOUNT_NOT_FOUND`/`VALIDATION_ERROR` pour une autre raison
  passerait. Asserter le code.
- Compte de test non imputable **et** d'un autre type, ou d'une autre société : le refus viendrait du
  type ou du tenant. Ne changer **que** `postable`.
- Fixtures des quatre `reconciliation_*_e2e.rs` : elles ne posent que `postable: true` (vérifié par
  #427) — le compte non imputable doit être créé ou basculé explicitement.

### References

- Issues : #427, #429 (et son commentaire), #474, #271 (exemption « valeur en place »), #375 (24-5).
- `crates/kesh-db/src/repositories/journal_entries.rs:65-133` (garde modèle), `:182-200`.
- `crates/kesh-db/src/repositories/invoices.rs:550-660` (`validate_line_revenue_accounts_in_tx`, priorité des raisons).
- `crates/kesh-db/src/errors.rs:5-47` (`RevenueAccountRejection`), `:346-350`, `:465-476`.
- `crates/kesh-api/src/errors.rs:90-130`, `:3067-3074`.
- `crates/kesh-api/src/routes/company_invoice_settings.rs:133-229`.
- `_bmad-output/implementation-artifacts/14-3b-consommateurs-roles.md` (D-A0, D-A1, L2).
- `_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md` (invariant I1 amendé, énumération des quinze sites, § passe 5).
- `CLAUDE.md` : § *Pattern batch*, § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit NOMMER le manuel*, § *Test Locally First* (exception `kesh-db`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Spécification : analyse de contexte complète, guide de développement produit (2026-10-08).

### File List

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie). Statut `ready-for-dev`.
  Choix C3–C6 consignés. Validation non lancée (à l'orchestrateur, contexte frais).
