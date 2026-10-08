# Story 15.5a : Le refus « compte non imputable », sous son vrai nom

Status: done

<!-- Issue de la story 15-5, DÉCOUPÉE le 2026-10-08 après la passe de validation P1 (choix C7 de
     `epic-15-choix-autonomes.md`). Sous-story « zéro » du patron « story-zéro qui pose le patron +
     rollout » du CLAUDE.md : elle pose la variante d'erreur que la 15-5b déploiera sur les surfaces
     neuves. Choix applicables : C3 (forme du refus), C7 (découpage), C13 (ordre des causes ; sa clé de
     détail est révisée par C16), C16 (forme du détail), C17 (construction de la variante), C18 (pluriel
     Fluent), C19 (parenthèse du message), C20 (`exempt_ids` retiré), C21 (verrou hors périmètre), C22
     (ordre de la boucle fournisseur), C23 (frontière des commentaires), C24 (documentation des
     intégrateurs), C29 (accesseur unique du détail `rejected`), C31 (ordre lexicographique puis
     identifiant). Validations P2 et P3 faites (Change Log). -->

**Issues** : `refs #427`, `refs #429` — **ne ferme rien** (c'est la 15-5b qui ferme #427 et #429).
Elle traite le **commentaire de #429** : toutes les gardes de postabilité refusent aujourd'hui avec
`InactiveOrInvalidAccounts`, « Un ou plusieurs comptes sont archivés ou invalides », message que le
dépôt avait déjà jugé trompeur en 16-1a pour ce motif.

**Dépendances** : aucune. **La 15-5b dépend de cette story** (elle émet la variante posée ici).

## Story

En tant que **comptable ou intégrateur qui appelle Kesh par l'écran ou par clé d'API**,
je veux qu'**un refus pour compte non imputable le dise sous son vrai nom, en nommant le ou les
comptes**,
afin de **savoir quoi corriger** — choisir un sous-compte imputable — au lieu de chercher un compte
« archivé ou invalide » qui ne l'est pas.

### Pourquoi une story à part

La 15-5 d'origine mêlait ce changement de contrat (un code d'erreur neuf, visible des intégrations, et
la réécriture de tous les tests qui figeaient l'ancien) et le déploiement de gardes sur sept surfaces
neuves. La passe P1 a compté plus de cinq modules (C-4/M5) et la seule dérogation codifiée — les cycles
Cargo — ne s'appliquait pas. Cette moitié est le **socle** : la variante, son message, et les quatre
gardes **existantes** (saisie manuelle 14-3b, trois gardes de la 24-5) converties. Elle ne crée aucune
garde neuve.

## Acceptance Criteria

### La variante et son message

1. **AC1 — Une variante dédiée, un code, un détail structuré — garantis par construction.**
   `crates/kesh-db/src/errors.rs` porte :
   - `pub struct NonPostableAccount { pub account_id: i64, pub account_number: String }` — **jumelle**
     d'`ArchivedAccount` (`errors.rs:291`), dont le numéro est `Option` parce qu'un compte inconnu n'en
     a pas ; ici le compte est toujours de la société et actif, son numéro toujours connu (choix C16) ;
   - `pub struct NonPostableAccounts(Vec<NonPostableAccount>)` — **champ privé** ; seul constructeur
     `NonPostableAccounts::new(impl IntoIterator<Item = NonPostableAccount>) -> Self`, qui **trie par
     numéro dans l'ordre lexicographique de la chaîne** (`"1000" < "10000" < "1010" < "2000"` — le
     numéro est une `String`, et un tri numérique échouerait sur un numéro non numérique), **puis par
     identifiant** à numéro égal, et **dédoublonne par identifiant**, avec la précondition « non vide »
     vérifiée par `debug_assert!` ; accesseurs en lecture (`iter()`, `numbers()`) (choix C17, C31) ;
   - l'**accesseur unique du détail JSON** `NonPostableAccounts::details(&self) -> serde_json::Value`,
     qui rend `{ "rejected": [{ "accountId", "accountNumber" }] }` dans l'ordre de la liste —
     `serde_json` est déjà une dépendance de `kesh-db`. Le bras d'`errors.rs` de `kesh-api` **et** les
     `failed[].details` de la 15-5b l'appellent ; aucun autre site ne construit ce JSON (choix C29) ;
   - la variante `DbError::AccountsNotPostable(NonPostableAccounts)`, avec son attribut thiserror
     `#[error("Un ou plusieurs comptes ne sont pas imputables")]` (patron de
     `InactiveOrInvalidAccounts`, `errors.rs:349`), et un raccourci
     `DbError::accounts_not_postable(impl IntoIterator<Item = NonPostableAccount>)`.
   Parce que le champ est privé, **la variante ne peut être construite qu'avec une liste triée,
   dédoublonnée et non vide** — c'est le système de types qui le garantit, pas une consigne. Son
   `error_code()` rend `"ACCOUNT_NOT_POSTABLE"`. `crates/kesh-api/src/errors.rs` la mappe en
   **HTTP 400**, code `ACCOUNT_NOT_POSTABLE`, avec un corps
   `{ "error": { "code", "message", "details": { "rejected": [{ "accountId", "accountNumber" }] } } }`,
   dont `details` est **la valeur rendue par `NonPostableAccounts::details()`** —
   **exactement la forme du jumeau `ACCOUNT_ARCHIVED`** (bras `DbError::ReversalAccountsArchived`,
   `crates/kesh-api/src/errors.rs:2945-2972`). Le client envoie des identifiants : `accountId` lui dit
   quelle ligne corriger, `accountNumber` le dit à l'utilisateur. Les deux sont exposés sans risque : la
   variante n'est émise que pour un compte **de la société et actif** (anti-énumération KF-002
   préservée — un compte inconnu ou d'une autre société reste `InactiveOrInvalidAccounts`, ou `404` là
   où il l'est). ⚠️ La clé `rejected` est celle que la 15-5b réutilisera dans `failed[].details` — **une
   seule forme** pour un même refus (choix C16, qui révise la clé `accountNumbers` de C13).
2. **AC2 — Le message nomme la cause et les comptes, dans les quatre locales.** Clé neuve
   `error-account-not-postable` dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
   variables `$numbers` (numéros joints par « , ») et `$count` (nombre de comptes, passé comme
   **nombre** Fluent pour que le sélecteur de pluriel fonctionne), **sélecteur `[one]` / `*[other]`**
   (patron `contact-payment-terms-days-label`, `fr-CH/messages.ftl:440`) et **apostrophe
   typographique** `’` (patron `fr-CH/messages.ftl:975`). Texte FR **exact** :
   ```ftl
   error-account-not-postable = { $count ->
       [one] Le compte { $numbers } n’est pas imputable (compte de regroupement, de résultat ou de clôture) : choisissez un compte imputable.
      *[other] Les comptes { $numbers } ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : choisissez des comptes imputables.
   }
   ```
   La parenthèse nomme les **trois** sortes de comptes non imputables : regroupement (`postable =
   FALSE`, parent), **résultat** (rôle `CurrentYearResult`, 2979 — refusé par
   `test_create_manual_rejects_result_account`) et clôture (9000/9100/9200) (choix C19). Les trois autres
   locales suivent le vocabulaire déjà en place pour « non imputable »
   (`error-opening-complement-retained-earnings-not-postable`, ligne 975 de `fr-CH`, 925 de `de-CH`,
   `it-CH` et `en-CH`) : **DE** « bebuchbar », **EN** « postable », **IT** « registrabile ». Aucune ne dit
   « archivé » ni « invalide ». Le repli Rust du `t_args` reprend le texte FR au singulier/pluriel selon
   `count`.
   **Le sélecteur est inscrit au garde-fou** `SELECTEURS_RESOLUS_COTE_SERVEUR`
   (`crates/kesh-i18n/src/loader.rs:367`), dont le test (`:496-510`) refuse tout sélecteur non inscrit :
   le dictionnaire servi au frontend fige un sélecteur sur `*[other]`. L'inscription dit **où** la clé
   est résolue avec ses arguments — le bras `AccountsNotPostable` de `crates/kesh-api/src/errors.rs`, par
   `t_args` — et pourquoi c'est sûr : le frontend affiche `err.message`, déjà résolu par le serveur, et
   ne lit jamais cette clé dans son dictionnaire (choix C18).

### Les gardes existantes nomment la cause

3. **AC3 — Saisie manuelle** (`validate_lines_accounts_in_tx`,
   `crates/kesh-db/src/repositories/journal_entries.rs:84`). La requête **actuelle** (`:96-105`) filtre
   `active = TRUE` dans le `WHERE` et y ajoute `postable = TRUE` si `enforce_postable` — elle perd ainsi
   la raison de l'absence. La requête **réécrite** lit `id, number, active, postable` **sans** clause
   `active` ni `postable` dans le `WHERE` (seulement `company_id` et `id IN (…)`), et la décision se
   prend en Rust, dans cet ordre :
   1. un identifiant demandé **absent** du résultat (inconnu ou d'une autre société) **ou** un compte
      **archivé** → `InactiveOrInvalidAccounts` (inchangé) ;
   2. sinon, si `enforce_postable`, tout compte `postable = FALSE` →
      `DbError::accounts_not_postable(…)` nommant **tous** ces comptes (identifiant et numéro).
   Quand les deux défauts coexistent dans une même écriture, **le premier gagne** (patron « une seule
   raison, la plus bloquante d'abord » de `validate_line_revenue_accounts_in_tx`,
   `crates/kesh-db/src/repositories/invoices.rs:631-643`). Le chemin `enforce_postable = false` est
   **strictement inchangé** (même ensemble de comptes acceptés, même erreur). La fonction reste
   rollback-agnostique.
   **Le paramètre `exempt_ids` est retiré** (choix C20) : son seul appelant (`journal_entries.rs:340`)
   passe `&[]`, la modification d'une écriture n'existant plus depuis la 24-4b (la route rend
   `409 ENTRY_IS_POSTED`, `update_journal_entry`, `crates/kesh-api/src/routes/journal_entries.rs:596`). Le doc-comment le dit.
   **Le verrouillage ne change pas** : la requête reste sans `FOR UPDATE` — hors périmètre, cf. Dev
   Notes (choix C21).
4. **AC4 — Les trois gardes de la 24-5**, qui lisent aujourd'hui `active, postable` et rendent
   `InactiveOrInvalidAccounts` pour tout écart, distinguent désormais, **dans cet ordre** :
   - (a) compte **inconnu, d'une autre société, archivé**, ou — pour le compte de charge seulement —
     **d'un autre type que `Expense`** → `InactiveOrInvalidAccounts` (inchangé) ;
   - (b) sinon, compte **non imputable** → `DbError::accounts_not_postable([…])`.
   Un compte qui cumule (a) et (b) — p. ex. un compte d'actif non imputable proposé comme compte de
   charge — rend **(a)** (finding M4). Les trois sites, qui ajoutent `number` à leur `SELECT` (le
   verrou `FOR UPDATE` qu'ils posent déjà est conservé) :
   - `crates/kesh-db/src/repositories/invoice_settlements_write.rs:156-167` (règlement client, compte
     interne) ;
   - `crates/kesh-db/src/repositories/supplier_invoices.rs:336-347` (compte de charge, **dans la boucle
     sur les lignes**) — la boucle devient **deux passes** (choix C22) : d'abord la **forme** de toutes
     les lignes (quantité et prix strictement positifs, taux de TVA dans 0–100), dans l'ordre des
     lignes, refus immédiat — l'ordre des refus de forme entre eux est celui d'aujourd'hui ; puis les
     **comptes**, ligne par ligne : une ligne en défaut (a) rend `InactiveOrInvalidAccounts`
     immédiatement ; les comptes en défaut (b) sont **collectés** et, si aucune ligne n'est en (a), un
     seul `AccountsNotPostable` les nomme tous après la passe. **Seul changement d'ordre observable** :
     une ligne *i* en défaut de compte et une ligne *j > i* en défaut de forme rendent désormais le refus
     de **forme** (`IllegalStateTransition`) — testé (T6) ;
   - `crates/kesh-db/src/repositories/supplier_invoices.rs:639-650` (règlement fournisseur, compte
     interne).
   La **complétion d'une facture importée** (route
   `crates/kesh-api/src/routes/imported_supplier_invoices.rs:243`, qui appelle
   `supplier_invoices::create_in_tx`) hérite de la conversion sans changement de code.
5. **AC5 — L'écriture d'ouverture hérite du nom juste ; le complément ne bouge pas, et c'est prouvé.**
   `create_opening_entry` (`fn` `journal_entries.rs:562`, appel de `create_in_tx` `:634`,
   `enforce_postable = true`) rend désormais `ACCOUNT_NOT_POSTABLE` pour une ligne sur un compte non
   imputable de la société ; un compte inconnu, archivé ou d'une autre société garde
   `INACTIVE_OR_INVALID_ACCOUNTS` (test existant `post_cross_tenant_account_inactive_or_invalid`,
   `fn` `crates/kesh-api/tests/opening_balances_e2e.rs:705`, assertion `:770`, **inchangé et vert**).
   Le complément de soldes (`opening_complement.rs:641`) n'est **pas** concerné : son contrôle préalable
   (`check_lines`, `fn` `:646`, appel `:585`) rend `OPENING_COMPLEMENT_ACCOUNT_INVALID` avant
   `create_in_tx`. **Aucun test existant n'exerce la branche `|| !a.postable` de `check_lines`** (les
   tests de `opening_complement_repository.rs` visent un compte archivé, étranger, inexistant, ou le
   2970 de report) : la story en **ajoute un** (T6) — une ligne de complément sur un compte non
   imputable de la société → `(AccountInvalid, Some(id))` — et le mute.

### Ce que voit l'utilisateur, et ce qui ne change pas

6. **AC6 — Les écrans affichent le message du serveur.** `JournalEntryForm.svelte:178`
   (`frontend/src/lib/features/journal-entries/`) ajoute `case 'ACCOUNT_NOT_POSTABLE':` au groupe qui
   affiche `err.message`. Les autres écrans atteints par les gardes de l'AC4 et de l'AC5 — dialogue de
   règlement d'une facture client, saisie et règlement d'une facture fournisseur, **complétion d'une
   facture importée** (`frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`,
   `completeErrorLabel`, qui rabat un code inconnu sur `err.message`), soldes de départ — affichent
   déjà `err.message` pour un code qu'ils ne connaissent pas : **le dev le vérifie** écran par écran
   (lecture du `catch`) et le consigne au Dev Agent Record ; un écran qui afficherait un repli générique
   à la place est corrigé dans cette story.
7. **AC7 — Les tests qui figeaient l'ancien code pour un compte non imputable sont réécrits à dessein**,
   chacun avec un commentaire qui cite la 15-5a :
   - `crates/kesh-db/src/repositories/journal_entries.rs:3676` (`test_create_manual_rejects_non_postable_line`)
     et `:3755` (`test_create_manual_rejects_result_account`) → `AccountsNotPostable` ;
   - `crates/kesh-db/tests/invoice_settlement.rs:354` (`un_compte_non_imputable_est_refuse`) →
     `AccountsNotPostable` ; **`:311` (compte archivé) reste `InactiveOrInvalid`** ;
   - `crates/kesh-db/tests/supplier_invoices_repository.rs:424` (charge non imputable) et `:470`
     (règlement fournisseur, compte interne non imputable) → `AccountsNotPostable` ; **`:388` (mauvais
     type) et `:672` (autre société) restent `InactiveOrInvalidAccounts`** ;
   - `crates/kesh-api/tests/reports_e2e.rs:1953` → `ACCOUNT_NOT_POSTABLE` ; son commentaire, qui dit
     « aucun code d'erreur neuf n'est introduit par cette story » (AC 13 de la 24-5), est réécrit : le
     code neuf est **délibéré** (choix C3) ;
   - et toute autre occurrence que rend la commande de T0, triée : non imputable → nouveau code ;
     inconnu / archivé / autre société / mauvais type → inchangé.
   Les assertions par `format!("{err:?}").contains(…)` sont remplacées par un `matches!` sur la
   variante — un `contains("InactiveOrInvalid")` passe aussi pour un nom de variante voisin.
8. **AC8 — Non-régression.** Restent verts **sans modification** : `test_create_in_tx_auto_flow_allows_non_postable`
   (`journal_entries.rs:3687`, flux automatique sur compte de configuration devenu non imputable) ;
   `reverse_succeeds_when_an_account_became_non_postable` (`crates/kesh-api/tests/journal_entry_reversal_e2e.rs:1015`) ;
   `invoices_validate_vat.rs:301` (compte TVA **archivé** → `InactiveOrInvalidAccounts`) ; les refus de
   rôles absents de `insert_with_defaults_in_tx`, `kesh-seed/src/lib.rs:208` et `onboarding.rs:194,727`,
   qui ne passent par aucune des gardes modifiées. (Le test de l'ouverture cross-tenant est déjà nommé à
   l'AC5.)
9. **AC9 — Le contrat changé est écrit.**
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (à créer en tête si absente — c'est le motif
     exact qu'exige `scripts/prepare-release.sh:189`), rubrique **Modifié** : pour un compte non
     imputable, la saisie manuelle, l'écriture d'ouverture, le règlement par compte interne, la
     saisie / la complétion d'une facture fournisseur importée / le règlement d'une facture fournisseur
     répondent désormais `ACCOUNT_NOT_POSTABLE` (avec `details.rejected[{accountId, accountNumber}]`) au
     lieu de `INACTIVE_OR_INVALID_ACCOUNTS` — **changement visible d'une intégration par clé d'API** —,
     et le message nomme le ou les comptes.
   - **`docs/api-external.md`**, table des codes d'erreur (§ 10 *Gestion des erreurs*) : une ligne
     `400` `ACCOUNT_NOT_POSTABLE` — compte de la société, actif, mais non imputable (regroupement,
     résultat, clôture) ; `details.rejected[{accountId, accountNumber}]` ; rendu par les routes citées
     ci-dessus (choix C24). La 15-5b y ajoutera ses propres routes.
   - Le **manuel** ne cite ni l'ancien code ni l'ancien message pour ce motif (vérifié au `.tex` **et** au
     PDF aplati, cf. T7) : il n'y a rien à y changer dans cette story — le dire au Dev Agent Record,
     comme contrôle exercé et sans objet. `docs/manual/fr/admin-manual.tex` : sans objet, à écrire de
     même.

## Tasks / Subtasks

- [x] **T0 — Refaire l'inventaire des lecteurs de l'ancien code** sur `HEAD` et le comparer à l'AC7 :
  ```sh
  grep -rnE "InactiveOrInvalid[^A]|InactiveOrInvalidAccounts|INACTIVE_OR_INVALID_ACCOUNTS" \
    crates frontend/src frontend/tests --include=*.rs --include=*.ts --include=*.svelte
  ```
  (le motif `InactiveOrInvalid[^A]` attrape les `contains("InactiveOrInvalid")` que le motif plein
  rate — finding C-1). Trier **chaque** ligne (non imputable / autre motif / commentaire) au Dev Agent
  Record ; un site neuf non trié bloque la story.
- [x] **T1 — La variante** (AC1)
  - [x] `NonPostableAccount`, `NonPostableAccounts` (champ privé, `new` trieur/dédoublonneur,
        `debug_assert!` non vide), `DbError::AccountsNotPostable(NonPostableAccounts)` avec son attribut
        `#[error("…")]` et le raccourci `accounts_not_postable` dans `crates/kesh-db/src/errors.rs` ;
        doc-comment : pourquoi elle existe (commentaire de #429, précédent
        `RevenueAccountRejection::NotPostable`, choix C3, C16, C17) ; `error_code()` →
        `"ACCOUNT_NOT_POSTABLE"`. Compléter le `match` exhaustif de `errors.rs:~745`.
  - [x] Tests unitaires du constructeur : entrée désordonnée et dupliquée → triée par numéro **dans
        l'ordre lexicographique** (attendu fixé : `["1000", "10000", "1010", "2000"]` pour une entrée
        `["2000", "1010", "10000", "1000"]`), puis par identifiant à numéro égal, dédoublonnée par
        identifiant ; entrée vide → panique en `debug` (`#[should_panic]`, sous
        `#[cfg(debug_assertions)]`) ; `details()` rend exactement
        `{"rejected":[{"accountId":…,"accountNumber":…}, …]}` dans cet ordre (C29, C31).
  - [x] Bras dans `crates/kesh-api/src/errors.rs`, à côté de `InactiveOrInvalidAccounts` (`:3067`) :
        400, code, `t_args("error-account-not-postable", repli, args{numbers, count})` — `count`
        posé comme **nombre** Fluent (`FluentValue::from(usize)`), jamais comme chaîne —,
        `"details": accounts.details()` (C29 ; forme du bras `ReversalAccountsArchived`, `:2945-2972`).
- [x] **T2 — Le message** (AC2) : la clé dans les quatre `messages.ftl` ; l'inscription dans
      `SELECTEURS_RESOLUS_COTE_SERVEUR` (`loader.rs:367`) avec son commentaire « où » ; le test du
      garde-fou (`loader.rs:496-510`) vert ; `npm run lint-i18n-ownership` vert ; un test Rust par locale
      (patron des tests i18n de `kesh-i18n`, `bundle.format` **avec arguments**) vérifie que la clé se
      résout **au singulier et au pluriel** et contient le numéro passé.
- [x] **T3 — Saisie manuelle** (AC3) : réécrire `validate_lines_accounts_in_tx` (requête + décision en
      Rust ; retrait d'`exempt_ids` et de l'argument `&[]` de son appelant) et **son doc-comment**, que
      cette story possède seule (choix C23) : il annonce encore `Err(DbError::InactiveOrInvalidAccounts)`
      seul, un « facteur commun de `create_in_tx` et `update` » et un grandfather « à l'update » qui
      n'existent plus. Le nouveau doc-comment renvoie au doc-comment de `create_in_tx` pour la liste des
      flux qui passent `enforce_postable = false`, **sans la répéter** (ce paragraphe-là appartient à la
      15-5b).
- [x] **T4 — Les trois gardes de la 24-5** (AC4) : `SELECT active, postable, number[, account_type]`
      (verrou conservé) ; ordre (a) puis (b) ; deux passes dans la boucle des lignes de facture
      fournisseur. **Un commentaire neuf** au `match` de chaque site dit l'ordre (a)/(b). Les commentaires
      existants qui précèdent les `SELECT` décrivent ce qui est exigé (actif et imputable) et restent
      vrais : **ne pas les réécrire** — en particulier, le commentaire du compte interne de
      `invoice_settlements_write.rs` qui se termine par « restent ouverts et sont suivis par #427 »
      appartient à la 15-5b (choix C23).
- [x] **T5 — Ouverture et écrans** (AC5, AC6)
  - [x] `crates/kesh-api/src/routes/opening_balances.rs` : la doc de module (`:25-26`, « compte
        inexistant / archivé / non-postable / cross-tenant → `INACTIVE_OR_INVALID_ACCOUNTS` ») est
        réécrite (non imputable → `ACCOUNT_NOT_POSTABLE`) ; l'énumération d'exemples du mapping global
        (`:189`) gagne `ACCOUNT_NOT_POSTABLE`. **Verdict écrit** : `:332` et `:405` parlent d'ids
        **absents** (inexistant / autre société) et restent justes — aucun changement.
  - [x] `JournalEntryForm.svelte:178` : le `case`. Lecture des `catch` des autres écrans (AC6, dont
        `supplier-invoices/import/+page.svelte`), consignée.
- [x] **T6 — Tests** (AC3, AC4, AC5, AC7, AC8)
  - [x] Réécrire les tests de l'AC7 (liste fermée + ce que T0 aura trouvé).
  - [x] `kesh-db` — saisie manuelle : (i) un compte non imputable → `AccountsNotPostable` nommant
        `(id, n°)` ; (ii) **deux** comptes non imputables → les deux, triés par numéro ; (iii) un
        archivé **et** un non imputable dans la même écriture → `InactiveOrInvalidAccounts` (priorité) ;
        (iv) `enforce_postable = false` sur un non imputable → accepté (déjà couvert par
        `test_create_in_tx_auto_flow_allows_non_postable`, le citer).
  - [x] `kesh-db` — gardes 24-5 : pour chacune, non imputable → `AccountsNotPostable` ; archivé →
        `InactiveOrInvalidAccounts` ; et pour le compte de charge, **un compte d'actif non imputable**
        → `InactiveOrInvalidAccounts` (cumul (a)+(b), finding M4) ; deux lignes de charge non
        imputables → un seul refus nommant les deux ; **ordre des passes** (C22) : ligne 1 au compte non
        imputable + ligne 2 de quantité nulle → `IllegalStateTransition` ; ligne 1 au compte archivé +
        ligne 2 de prix négatif → `IllegalStateTransition`.
  - [x] `kesh-db` — complément de soldes (`crates/kesh-db/tests/opening_complement_repository.rs`, patron
        `refus_par_compte`, `:343`) : une ligne sur un compte de la société, actif, `postable = FALSE`
        → `(AccountInvalid, Some(id))`.
  - [x] `kesh-api` : `opening_balances_e2e.rs` — une ligne d'ouverture sur un compte non imputable →
        400 `ACCOUNT_NOT_POSTABLE` et `details.rejected == [{accountId, accountNumber}]` ;
        `reports_e2e.rs:1953` asserte aussi `details.rejected`.
  - [x] **Le pont `count`/`numbers` du bras API est testé de bout en bout** (finding P3 F-1) — dans
        `opening_balances_e2e.rs`, dont le montage appelle `init_error_i18n` (`:79`) : sans lui, `t_args`
        rend le repli Rust et le sélecteur Fluent n'est jamais exercé (c'est le cas de `reports_e2e.rs`,
        qui n'asserte que le code). Deux cas, locale `fr-CH` : **un** compte non imputable → le
        `message` commence par « Le compte », contient son numéro, ne contient ni « archiv » ni
        « invalide » ; **deux** comptes → « Les comptes », les deux numéros dans l'ordre de C31.
        **Mutation** : `count` passé en chaîne, puis retiré des arguments → le cas « un compte »
        rougit (le sélecteur retombe sur `*[other]`) ; consigner au Dev Agent Record.
  - [x] Le **compte de test ne diffère d'un compte accepté que par `postable`** (même société, actif,
        bon type), et l'assertion porte sur la **variante / le code**, jamais sur le seul statut 400.
  - [x] **Mutation** : pour chacune des quatre gardes, retirer la branche (b) une fois → le test
        négatif rougit (il retombe sur `InactiveOrInvalidAccounts` ou passe) ; idem pour la branche
        `|| !a.postable` de `check_lines` ; restaurer **et toucher le fichier** (mémoire « mutation
        restaurée, binaire périmé »). Consigner la liste au Dev Agent Record.
- [x] **T7 — Propagation et documentation** (AC9)
  - [x] `CHANGELOG.md` et `docs/api-external.md` (AC9).
  - [x] Manuel : `grep -n "archivés ou invalides\|INACTIVE_OR_INVALID" docs/manual/fr/*.tex` et
        `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "archivés ou invalides"`
        — attendu : aucune occurrence pour ce motif ; consigner le résultat.
  - [x] **Grep du symptôme** (règle *Propagation post-patch*) : les commentaires qui disent qu'un compte
        non imputable est refusé en `InactiveOrInvalidAccounts` / « archivés ou invalides » —
        `grep -rnE "non.?postable.*InactiveOrInvalid|InactiveOrInvalid.*non.?postable|INACTIVE_OR_INVALID_ACCOUNTS" crates`
        — sont réécrits, en particulier `journal_entries.rs:3647` (doc du test réécrit à l'AC7).
        **Verdict déjà établi en validation P2** : `crates/kesh-db/tests/invoices_line_revenue_account.rs:567`
        et `:708` (compte **archivé**) et `crates/kesh-db/src/repositories/accounts.rs:276` (id
        **absent**) ne portent pas ce motif — les relire, ne pas les réécrire sauf fait nouveau.
  - [x] **Commentaires qui décrivent la clause SQL que l'AC3 retire** (finding P3 F-2, attribution
        C23 : ils parlent de la garde de `validate_lines_accounts_in_tx`, que cette story réécrit) :
        `grep -rnE "clause .active|garde .active|validate_accounts.\]|active = TRUE.,? qui est inconditionnelle" crates`
        (six lignes sur `92770300`). **Verdict établi en
        validation P3, sur `92770300`** — l'affirmation de fond (« la garde `active` ne dépend pas
        d'`enforce_postable` ») **reste vraie** après l'AC3 (étape 1, en Rust) ; seule la désignation
        devient fausse :
        - `crates/kesh-db/src/errors.rs:607-608` (« la clause `active = TRUE` ») → **réécrire** : la
          garde `active`, désormais décidée en Rust ;
        - `crates/kesh-db/src/repositories/journal_entries.rs:217` et `:1722` (« la garde
          `active = TRUE` de [`validate_accounts`] ») → **réécrire** : le lien d'intra-doc vise une
          fonction qui n'existe pas (défaut antérieur) — le faire pointer sur
          [`validate_lines_accounts_in_tx`] et dire « la garde `active` » ;
        - `crates/kesh-api/tests/journal_entry_reversal_e2e.rs:415` (« la garde `active = TRUE`, qui est
          inconditionnelle ») → **réécrire** de même (« la garde `active` ») ;
        - `crates/kesh-db/src/errors.rs:484`, `crates/kesh-db/src/repositories/credit_notes.rs:423-424`
          et, hors du motif mais relu, `errors.rs:11`
          (« la garde `active` … inconditionnelle », sans désigner de clause) → **justes, inchangés** ;
        Tout site neuf rendu par la commande est trié de même au Dev Agent Record.
- [x] **T8 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) —
      **même en cours de boucle de revue** (la story touche des repositories `kesh-db`) ; gate frontend
      complet ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par
      fichier contre `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 9 AC, 9 tâches T0–T8.)*

## Dev Notes

### Le modèle, et ce qu'il n'est pas

- `validate_line_revenue_accounts_in_tx` (`invoices.rs:550-660`, 16-1a) est le modèle de la
  **priorité** : une seule raison, la plus bloquante d'abord. `RevenueAccountRejection::NotPostable`
  (`crates/kesh-db/src/errors.rs:5-47`) est le précédent d'un refus qui nomme la non-imputabilité ; on
  ne le réutilise pas (il est propre aux lignes de facture, choix C3).
- `ReversalAccountsArchived(Vec<ArchivedAccount>)` (`errors.rs:614`) est le modèle de la **forme** du
  détail (`details.rejected`) ; on ne réutilise pas `ArchivedAccount` (numéro optionnel, choix C16).
- **Ne pas changer `enforce_postable`** des appelants : cette story change le **nom** du refus, pas son
  **périmètre**. Aucun compte aujourd'hui accepté ne devient refusé, aucun refusé ne devient accepté.
- La décision en Rust (au lieu de la clause SQL) est nécessaire pour savoir **pourquoi** un compte
  manque — la requête actuelle le filtre et perd l'information.

### Ce qui doit être préservé

- Anti-énumération (KF-002) : un compte inconnu ou d'une autre société ne livre jamais son numéro ni son
  identifiant ; seul un compte **de la société, actif** est nommé. La construction de la variante n'a
  donc lieu qu'après le contrôle d'appartenance.
- `insert_with_defaults_in_tx`, `seed_demo`, `onboarding.rs:194,727` reconnaissent
  `InactiveOrInvalidAccounts` pour **leurs** refus (comptes de rôle absents) ; ils ne passent par aucune
  des gardes modifiées — le vérifier à T0, ne rien y changer.

### Hors périmètre, et écrit

- **Le verrouillage de `validate_lines_accounts_in_tx`** (choix C21). La saisie manuelle lit les comptes
  sans verrou, alors que les gardes de la 24-5 lisent `FOR UPDATE` : un compte archivé ou rendu non
  imputable entre le contrôle et l'insertion passe. Le défaut est antérieur (14-3b), étranger au *nom*
  du refus, et poser un verrou sur le chemin le plus fréquent changerait l'ordre d'acquisition des
  verrous face aux flux qui verrouillent déjà ces comptes — risque d'interblocage non mesuré. Signalé
  pour une issue de dette.
- Le message du complément de soldes (`OPENING_COMPLEMENT_ACCOUNT_INVALID`) pour un compte non
  imputable : il garde son code propre ; seul le test manquant est ajouté (AC5).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C3** — la forme du refus (variante dédiée, code `ACCOUNT_NOT_POSTABLE`, étendue aux gardes
  existantes).
- **C7** — le découpage : cette story est le socle ; la 15-5b déploie.
- **C13** — l'ordre des causes sur une garde à plusieurs critères (sa clé de détail est révisée par C16).
- **C16** — `details.rejected[{accountId, accountNumber}]`, forme du jumeau `ACCOUNT_ARCHIVED`, commune
  au 400 et aux `failed[]` de la 15-5b.
- **C17** — newtype à champ privé, constructeur trieur, précondition non vide.
- **C18** — sélecteur Fluent inscrit à `SELECTEURS_RESOLUS_COTE_SERVEUR`.
- **C19** — « de regroupement, de résultat ou de clôture ».
- **C20** — `exempt_ids` retiré.
- **C21** — verrou hors périmètre.
- **C22** — forme d'abord, comptes ensuite, dans la boucle fournisseur.
- **C23** — chaque commentaire réécrit par une seule story.
- **C24** — `docs/api-external.md`.
- **C29** — `NonPostableAccounts::details()`, seul constructeur du JSON `details.rejected` (bras API de
  cette story, `failed[]` et refus à l'usage de la 15-5b).
- **C31** — ordre lexicographique du numéro, puis identifiant.

### Fichiers touchés (prévision)

`crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/{journal_entries,invoice_settlements_write,supplier_invoices}.rs`,
`crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/opening_balances.rs` (commentaires),
`crates/kesh-i18n/locales/*/messages.ftl`, `crates/kesh-i18n/src/loader.rs`,
`frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`,
tests (`crates/kesh-db/tests/{invoice_settlement,supplier_invoices_repository,opening_complement_repository}.rs`,
`crates/kesh-api/tests/{reports_e2e,opening_balances_e2e}.rs`, `mod tests` de `journal_entries.rs`),
commentaires (`crates/kesh-db/src/errors.rs`, `journal_entries.rs`, `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`,
T7), `CHANGELOG.md`, `docs/api-external.md`. **Aucune migration** (P1–P8 sans objet). Modules de premier
niveau : `kesh-db`, `kesh-api` (erreurs), `kesh-i18n`, `frontend/journal-entries` — sous le seuil de la
règle de splitting.

### Tests — ce qui rendrait un test vert sans rien prouver

- `contains("InactiveOrInvalid")` sur le `Debug` : passe pour n'importe quelle variante dont le nom
  commence ainsi. Asserter la variante par `matches!`.
- Un compte de test non imputable **et** d'un autre type : le refus viendrait du type. Ne changer que
  `postable` — sauf dans le test de priorité, qui cumule exprès.
- Un message asserté en français seulement : le pluriel cassé dans une autre locale passerait. T2 teste
  les quatre.
- Un test du pluriel par `all_messages` (le dictionnaire du frontend) : il rendrait toujours `*[other]`.
  Le message de cette clé est formé côté serveur : tester par `format` avec arguments.

### References

- Issues : #427, #429 (et son commentaire), #375 (24-5).
- `crates/kesh-db/src/repositories/journal_entries.rs:65-133`, `:3640-3760`.
- `crates/kesh-db/src/repositories/invoices.rs:550-660`.
- `crates/kesh-db/src/errors.rs:291-295` (`ArchivedAccount`), `:349`, `:614`.
- `crates/kesh-api/src/errors.rs:52-60` (`t_args`), `:2945-2972`, `:3067-3074`, `:3120-3140`.
- `crates/kesh-i18n/src/loader.rs:361-367`, `:496-540`.
- `_bmad-output/implementation-artifacts/24-5-comptes-de-cloture.md`, `14-3b-consommateurs-roles.md`.
- Fiche source : `15-5-gardes-postabilite-serveur.md` (statut `split`, Change Log de la passe P1).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`, autonomie déléguée — consignes de l'Epic 15), worktree
`kesh-15-5a`, branche `story/15-5a-refus-non-imputable`.

### Debug Log References

- Journaux de gate (scratchpad de session, non versionnés) : `gate1.log` (test-fast), `e2e.log`
  (Playwright).
- Vitest : les 104 fichiers échouaient d'abord en « Cannot find module '/@fs/…/kesh/frontend/node_modules/…' »
  — le lien symbolique `frontend/node_modules` du worktree sort de la racine servie par Vite. Remplacé
  par une copie en liens durs (`cp -al`) du `node_modules` du dépôt principal : défaut du montage du
  worktree, pas du code ; rien de versionné n'a changé.

### Completion Notes List

**T0 — inventaire des lecteurs de l'ancien code** (commande de T0 sur `HEAD` = `38bbd272`, avant
toute modification ; chaque ligne triée) :

| Site | Tri | Sort |
|---|---|---|
| `kesh-db/src/repositories/journal_entries.rs:78`, `:130` | garde de la saisie manuelle (doc + refus) | réécrits (T3) |
| `journal_entries.rs:3647`, `:3676-3677`, `:3755` | tests non imputable | réécrits (AC7) |
| `kesh-db/src/repositories/invoice_settlements_write.rs:166` | garde 24-5, `_ =>` | converti (T4) |
| `kesh-db/src/repositories/supplier_invoices.rs:346`, `:649` | gardes 24-5, `_ =>` | converties (T4) |
| `kesh-db/tests/invoice_settlement.rs:311` | compte **archivé**, `contains` | reste `InactiveOrInvalid`, passé en `matches!` |
| `kesh-db/tests/invoice_settlement.rs:354` | non imputable | réécrit (AC7) |
| `kesh-db/tests/supplier_invoices_repository.rs:424`, `:470` | non imputable | réécrits (AC7) |
| `supplier_invoices_repository.rs:388` (type), `:672` (autre société) | autre motif | inchangés |
| `kesh-api/tests/reports_e2e.rs:1953` | non imputable (9000) | réécrit (AC7) |
| `kesh-api/src/routes/opening_balances.rs:26`, `:189` | doc de module / mapping | réécrits (T5) |
| `opening_balances.rs:332`, `:405` | ids **absents** | justes, inchangés (verdict T5) |
| `kesh-api/tests/opening_balances_e2e.rs:703`, `:770` | autre société | inchangés (AC5) |
| `kesh-api/src/errors.rs:3067-3069` | bras de l'ancienne variante | inchangé (le nouveau bras s'y ajoute) |
| `kesh-api/src/errors.rs:3124` | commentaire 16-1a (comptes de produit) | juste, inchangé |
| `kesh-api/src/routes/reconciliation.rs:3000` | compte bancaire **archivé** | juste, inchangé (surface de la 15-5b) |
| `kesh-api/src/routes/onboarding.rs:194`, `:713`, `:727`, `kesh-seed/src/lib.rs:208` | rôles absents de `insert_with_defaults_in_tx` | inchangés (AC8) |
| `kesh-api/tests/fiscal_years_e2e.rs:1025` | rôles absents (fixture) | inchangé |
| `kesh-db/src/repositories/company_invoice_settings.rs:597`, `:601`, `:667`, `:735`, `:792` ; `kesh-db/tests/company_invoice_settings_repository.rs:116-959` | réglages de facturation (comptes absents / inactifs) | inchangés — surface de la 15-5b |
| `kesh-db/src/repositories/invoices.rs:2086` | compte de produit archivé ou retypé | juste, inchangé |
| `kesh-db/src/repositories/credit_notes.rs:427` | compte archivé | juste, inchangé |
| `kesh-db/src/repositories/accounts.rs:276` ; `kesh-db/tests/opening_balances_repository.rs:304` | id absent | justes, inchangés |
| `kesh-db/tests/invoices_validate_vat.rs:5`, `:284`, `:301-302` | compte TVA archivé | inchangés (AC8) |
| `kesh-db/tests/invoices_line_revenue_account.rs:567`, `:708` | compte archivé | justes, inchangés (verdict P2) |
| `kesh-db/src/errors.rs:350`, `:467`, `:745` | définition / doc / `error_code` | inchangés |
| `frontend/…/JournalEntryForm.svelte:178` | groupe `err.message` | `case 'ACCOUNT_NOT_POSTABLE'` ajouté (AC6) |

Aucun site neuf hors de la liste de l'AC7.

**Ce qui a été implémenté**

- **AC1** — `NonPostableAccount`, `NonPostableAccounts` (champ privé ; `new` dédoublonne par
  identifiant puis trie par numéro lexicographique puis identifiant ; `debug_assert!` non vide ;
  `iter()`, `numbers()`, `len()`, `is_empty()` — choix **C-15-5a-4** ; `details()` seul constructeur du
  JSON), `DbError::AccountsNotPostable` avec `#[error]`, raccourci `DbError::accounts_not_postable`,
  `error_code()` → `ACCOUNT_NOT_POSTABLE`. Bras `kesh-api` : 400, `t_args` avec `numbers` et `count`
  **nombre** Fluent (`usize`), repli FR singulier/pluriel, `details: accounts.details()`. Le `match`
  exhaustif de `error_code()` est complété ; aucun autre `match` exhaustif sur `DbError` n'a exigé
  de bras (compilation du workspace).
- **AC2** — clé `error-account-not-postable` dans les quatre locales, sélecteur `[one]` / `*[other]`,
  texte FR exact de l'AC ; vocabulaire DE/IT/EN de la parenthèse : choix **C-15-5a-2**. Inscrite à
  `SELECTEURS_RESOLUS_COTE_SERVEUR` avec son « où ».
- **AC3** — `validate_lines_accounts_in_tx` : `SELECT id, number, active, postable` sans clause
  `active`/`postable` ; (1) absent ou archivé → `InactiveOrInvalidAccounts` ; (2) si
  `enforce_postable`, non imputables → `accounts_not_postable` (tous nommés). `exempt_ids` retiré, ainsi
  que l'argument `&[]` de l'appelant. Doc-comment réécrit (C23), sans répéter la liste des flux de
  `create_in_tx`. Pas de `FOR UPDATE` (C21).
- **AC4** — trois gardes : `number` ajouté au `SELECT` (verrou conservé), bras (b) explicite, `_ =>`
  (a) inchangé, commentaire d'ordre (a)/(b) neuf à chaque `match` ; commentaires existants non
  réécrits (C23). Boucle des lignes fournisseur en **deux passes** (C22).
- **AC5** — l'ouverture hérite par `create_in_tx` ; prouvé de bout en bout (tests ci-dessous). Le
  complément garde `OPENING_COMPLEMENT_ACCOUNT_INVALID` ; sa branche `|| !a.postable` est désormais
  testée et mutée.
- **AC6** — `case 'ACCOUNT_NOT_POSTABLE'` dans `JournalEntryForm.svelte`. **Lecture des `catch`**, écran
  par écran : règlement d'une facture client — `invoices/[id]/+page.svelte:582-588` et
  `invoices/due-dates/+page.svelte:265-270`, `markError = err.message` affiché par
  `SettleInvoiceDialog` (`errorMsg`) ; saisie d'une facture fournisseur —
  `supplier-invoices/+page.svelte:213-214`, `formError = err.message` ; règlement fournisseur —
  `supplier-invoices/[id]/+page.svelte:112-113`, `payError = err.message` ; complétion d'une facture
  importée — `supplier-invoices/import/+page.svelte:267-301`, `completeErrorLabel` rabat le `default`
  sur `err.message` ; soldes de départ — `settings/opening-balances/+page.svelte:208-213`,
  `submitError = err.message`. **Aucun écran n'affiche de repli générique** pour ce code : rien à
  corriger.
- **AC7** — les tests de la liste fermée sont réécrits avec un commentaire qui cite la 15-5a, en
  `matches!` / `match` sur la variante (plus aucun `contains("InactiveOrInvalid")` dans le dépôt).
  ⚠️ **`test_create_manual_rejects_result_account` passait à vide** sur la base de dev seedée (aucun
  compte `CurrentYearResult` — vérifié par requête) : il crée désormais un compte de résultat
  temporaire faute d'en trouver un (choix **C-15-5a-1**), et la mutation M1 le fait rougir.
- **AC8** — restés verts sans modification, au gate complet : `test_create_in_tx_auto_flow_allows_non_postable`
  (cas iv de T6), `reverse_succeeds_when_an_account_became_non_postable`, `invoices_validate_vat.rs`,
  les tests de `company_invoice_settings_repository.rs`, d'onboarding et de seed,
  `post_cross_tenant_account_inactive_or_invalid`.
- **AC9** — `CHANGELOG.md` : section `## [0.13.0] — Non publié` créée (motif exact de
  `prepare-release.sh:189`), rubrique **Modifié**. `docs/api-external.md` § 10 : lignes
  `ACCOUNT_NOT_POSTABLE` et `INACTIVE_OR_INVALID_ACCOUNTS` (choix **C-15-5a-3**). **Manuel** : contrôle
  exercé et sans objet — `grep -n "archivés ou invalides\|INACTIVE_OR_INVALID" docs/manual/fr/*.tex` :
  0 ligne ; `pdftotext … | tr '\n' ' ' | tr -s ' ' | grep -o "archivés ou invalides\|INACTIVE_OR_INVALID"`
  sur les trois PDF FR (`user-manual`, `admin-manual`, `marketing-brochure`) : 0 occurrence chacun.
  `user-manual.tex:390` décrit la protection sans citer de message : juste, inchangé.
  `admin-manual.tex` : sans objet. Aucun PDF régénéré (aucun `.tex` touché).

**T7 — grep du symptôme**
- `grep -rnE "non.?postable.*InactiveOrInvalid|InactiveOrInvalid.*non.?postable|INACTIVE_OR_INVALID_ACCOUNTS" crates`
  après patch : seuls restent des sites triés ci-dessus (absents, archivés, autre société), plus le
  doc-comment réécrit de `reports_e2e.rs` qui cite l'ancien code comme historique.
- `grep -rnE "clause .active|garde .active|validate_accounts.\]|active = TRUE.,? qui est inconditionnelle" crates` :
  réécrits `kesh-db/src/errors.rs` (doc de `ReversalAccountsArchived`, « la garde `active` … décidée en
  Rust »), `journal_entries.rs` (doc de `create_in_tx_inner` et de `archived_accounts_in_tx`, liens
  vers [`validate_lines_accounts_in_tx`]), `journal_entry_reversal_e2e.rs:415` ; justes et inchangés :
  `errors.rs:11`, `errors.rs:~599` (ex-`:484`), `credit_notes.rs:423-424`. Plus aucune occurrence de
  `[validate_accounts]`.

**Tests ajoutés** (périmètre : `origin/main` (`c9be146a`) → arbre de travail du commit de dev,
recomptés par `grep -cE '#\[(tokio::)?test|#\[sqlx::test'` aux deux bornes) : **+16** —
`kesh-db/src/errors.rs` 0 → 4 (tri lexicographique + dédoublonnage, `details()`, panique sur vide en
debug, `error_code`) ; `kesh-i18n/src/loader.rs` 16 → 17 (singulier/pluriel des quatre locales par
`format` avec arguments, texte FR exact) ; `journal_entries.rs` 38 → 40 (deux comptes nommés triés ;
archivé prioritaire) ; `supplier_invoices_repository.rs` 37 → 43 (charge archivée ; actif non
imputable comme charge → (a) ; deux lignes non imputables → un refus ; ordre des passes ×2 ;
règlement fournisseur archivé) ; `opening_complement_repository.rs` 23 → 24 (complément sur compte non
imputable → `(AccountInvalid, Some(id))`) ; `opening_balances_e2e.rs` 26 → 28 (singulier : code,
`details.rejected`, « Le compte », numéro, ni « archiv » ni « invalide » ; pluriel : « Les comptes »,
« 10000, 2100 »). Réécrits sans changement de nombre : `invoice_settlement.rs` (17),
`reports_e2e.rs` (38), plus deux tests de `journal_entries.rs` et deux de
`supplier_invoices_repository.rs`. Total nextest : 2754 = 2738 (v0.12.1) + 16.

**Mutations** (chacune appliquée seule, tests ciblés lancés, fichier restauré **puis touché**) :

| Mutant | Tests qui rougissent |
|---|---|
| M1 — saisie manuelle : branche (b) retirée | `test_create_manual_rejects_non_postable_line`, `…_names_every_non_postable_account_sorted`, `…_rejects_result_account` (3/3) |
| M2 — règlement client : bras (b) retiré | `un_compte_non_imputable_est_refuse` |
| M3 — compte de charge : bras (b) retiré | `create_with_non_postable_expense_account_is_rejected`, `create_with_two_non_postable_expense_lines_names_both` |
| M4 — règlement fournisseur : bras (b) retiré | `pay_with_non_postable_account_is_rejected` |
| M5 — `check_lines` : `\|\| !a.postable` retiré | `refus_compte_non_imputable` |
| M6 — `count` passé en chaîne | `post_non_postable_account_is_named_singular` (le pluriel reste vert, attendu) |
| M7 — `count` retiré des arguments | `post_non_postable_account_is_named_singular` |
| M8 — boucle fournisseur ramenée à une passe | `create_form_of_later_line_wins_over_archived_account` (son jumeau « non imputable » reste vert : en une passe, le compte non imputable est collecté et la forme de la ligne suivante refuse quand même — seul le cas archivé distingue les deux ordres, ce que dit l'AC4) |

**Gates — au dernier commit de code (ce commit)**
1. Base de dev remise à zéro (`restart mariadb` + `sqlx migrate run` + `seed-dev-db.sql`) avant le
   gate complet.
2. `scripts/test-fast.sh` : `cargo fmt --check` vert, `clippy --workspace --all-targets -D warnings` vert,
   **nextest 2754 exécutés, 2754 passés, 4 ignorés** (103,9 s).
3. Frontend : `npm run check` 0 erreur (27 avertissements préexistants, 5 fichiers, aucun touché) ;
   `npm run lint-i18n-ownership` PASS ; `npm run test:unit` **104 fichiers, 969 tests verts** ;
   `npm run build` vert.
4. **E2E complet** (montage du `CLAUDE.md` / `docs/testing.md` : backend du commit servant
   `frontend/build`, `kesh_e2e` neuve — vidée par la remise à zéro de MariaDB — et migrée,
   `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR`/`KESH_SMTP_*`, `KESH_TEST_MODE` des deux côtés ;
   `/health` : `smtpConfigured: true`) — lancé à 12:19 UTC : **239 passés, 9 échoués, 18 ignorés**
   (10,3 min). Jugés fichier par fichier :
   - `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
     `:77`, `:150` — **KF-029 (#97)**, les sept attendus ;
   - `onboarding-path-b.spec.ts:27` — hors liste : échec dans le **`beforeEach`** (`page.fill('#username')`
     en timeout), le même crochet que `:65` (KF-029) ; **rejoué seul : passe le crochet puis est
     ignoré** (`test.fixme`, KF-032) — pas une régression, la story ne touche ni le login ni
     l'onboarding ;
   - `sidebar-navigation.spec.ts:75` — **rejoué seul : vert** (1,4 s) ⇒ pollution d'état, non la
     KF-046.
   KF-045 (#421) ne s'est pas déclenchée (run après 12:00 UTC). Aucune spec touchant la saisie
   d'écriture, les soldes de départ ou les factures n'a échoué. **Verdict : aucune régression.**

**Reste / à signaler**
- Le verrou de `validate_lines_accounts_in_tx` (C21) : hors périmètre, issue de dette à ouvrir par
  l'orchestrateur.
- Le `frontend/node_modules` en lien symbolique des worktrees casse Vitest (cf. Debug Log).

### File List

- `CHANGELOG.md`
- `docs/api-external.md`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/tests/invoice_settlement.rs`
- `crates/kesh-db/tests/supplier_invoices_repository.rs`
- `crates/kesh-db/tests/opening_complement_repository.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/opening_balances.rs`
- `crates/kesh-api/tests/opening_balances_e2e.rs`
- `crates/kesh-api/tests/reports_e2e.rs`
- `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`
- `crates/kesh-i18n/src/loader.rs`
- `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`
- `_bmad-output/implementation-artifacts/15-5a-refus-non-imputable.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`

## Change Log

- 2026-10-08 — **Créée par découpage de la 15-5** après la passe de validation **P1** (trois lentilles
  Sonnet, contexte frais ; prompt `15-5-validate-prompt-p1.md`). Bilan de P1 sur la 15-5 entière :
  **0 CRITICAL, 0 HIGH** ; bruts par lentille A 5 MEDIUM / 6 LOW, B 2 / 2, C 4 / 4 ; **après fusion des
  doublons, 7 MEDIUM et 8 LOW distincts** (ventilation dans la fiche 15-5). Ce qui en revient à cette
  moitié, et comment :
  - **M1 = C-1** (MEDIUM, lentilles A et C) — le grep de T2 ratait `invoice_settlement.rs:354` ;
    `supplier_invoices_repository.rs:424` et `:470` n'étaient pas nommés → liste fermée à l'AC7, motif
    `InactiveOrInvalid[^A]` à T0.
  - **M4** (MEDIUM, A) — l'ordre « non imputable ET mauvais type » n'était pas tranché → AC4 (a) puis
    (b), test de cumul.
  - **M5 = C-4** (MEDIUM, A et C) — règle de splitting franchie → découpage (choix **C7**).
  - **C-3** (MEDIUM, C) — `## [Unreleased]` inexistant → `## [0.13.0] — Non publié`, rubriques
    françaises (AC9).
  - **L2/B4** (LOW, A et B) — détail structuré non fixé → `details.accountNumbers` (AC1, choix C13 ;
    **révisé en P2**, cf. ci-dessous).
  - **L3** (LOW, A) — pluriel et apostrophe → AC2.
  - **L4** (LOW, A) — messages des quatre locales non testés → T2.
  - **C-8 / L5** (LOW, A et C) — manuel : contrôle sans objet ici, écrit à l'AC9.
  Les autres findings (B1/M2, C-2/M3, B2/C-6, B3/C-5, C-7, L1, L6) portent sur les surfaces neuves : voir
  la 15-5b. Décisions de l'orchestrateur : C7 à C11 ; ajoutées pendant la remédiation : C12 à C14.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-5a-validate-prompt-p2.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P1, **F**
  adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×3 → P2 Opus ×2). **0 CRITICAL, 0 HIGH.**
  Bruts : R 3 MEDIUM + 7 LOW, F 2 MEDIUM + 6 LOW. Doublons inter-lentilles : F-3 = R-8, F-4 = R-7,
  F-5 = R-4, F-7 = R-5 → **5 MEDIUM et 9 LOW distincts**. Trend : P1 (15-5 entière) 7 MEDIUM / 8 LOW →
  P2 (15-5a seule) 5 MEDIUM / 9 LOW.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R-1 | MEDIUM | le sélecteur Fluent de l'AC2 aurait rougi `SELECTEURS_RESOLUS_COTE_SERVEUR` | AC2, T2, `loader.rs` aux fichiers (C18) | **né de la remédiation P1** (L3 a introduit le sélecteur) |
  | R-2 | MEDIUM | parenthèse « regroupement ou de clôture » omettant le compte de résultat (2979) | AC2, 4 locales + repli ; signalé à la 15-5b (C19) | **né de la remédiation P1** (texte exact écrit en P1, qui recopiait la parenthèse du message existant de `validate_account_of`) |
  | R-3 | MEDIUM | variante publique, promesse « construite seulement par le constructeur » invérifiable | AC1, T1 : newtype à champ privé (C17) | **né de la remédiation P1** (affirmation ajoutée avec L2/B4) |
  | F-1 | MEDIUM | `details.accountNumbers` s'écarte du jumeau `ACCOUNT_ARCHIVED` (`details.rejected[{accountId, accountNumber}]`) | AC1, AC9, T6 ; révise C13 (C16) | **né de la remédiation P1** (C13) |
  | F-2 | MEDIUM | AC5 : « vérifié par son test existant » faux — la branche non imputable de `check_lines` n'est exercée par aucun test | AC5, T6 (test + mutation) | **né de la remédiation P1** (la spec d'origine disait « message juste », la remédiation « vérifié ») |
  | R-4 = F-5 | LOW | ordre des refus dans la boucle fournisseur changé par la collecte | AC4, T6 : forme d'abord (C22) | — |
  | R-5 = F-7 | LOW | faits : `fr-CH:975` (925 ailleurs), `fn` `:562` / appel `:634`, `check_lines` `:646` / `:585`, `post_cross_tenant…` `:705` ; AC8 doublait l'AC5 (`:770`) | corrigés ; doublon retiré de l'AC8 | — |
  | R-6 | LOW | T5/T7 nommaient des commentaires qui ne portent pas le motif | verdicts écrits à T5 et T7 | — |
  | R-7 = F-4 | LOW | surface omise : complétion d'une facture importée et son écran | AC4, AC6, AC9 | — |
  | R-8 = F-3 | LOW | `exempt_ids` paramètre mort ; doc d'un `update` inexistant | AC3, T3 : retiré (C20) | — |
  | R-9 | LOW | frontière : deux commentaires réécrits par les deux stories | T3, T4 : attribution par contenu (C23) | — |
  | R-10 | LOW | en-tête n'annonçait pas C13 ; T1 sans l'attribut `#[error]` | en-tête, T1 | — |
  | F-6 | LOW | itérable vide → message sans numéro ; `#[error]` | AC1, T1 : `debug_assert!` (C17) | — |
  | F-8 | LOW | `validate_lines_accounts_in_tx` lit sans verrou | **hors périmètre, écrit** (Dev Notes, C21) ; issue suggérée à l'orchestrateur | — |

  **Signal de la règle de découpage** (sévérité MEDIUM → MEDIUM) : **constaté**. Selon l'amendement D5,
  il compte ce qui le compose : les **cinq** MEDIUM de P2 sont **nés de la remédiation P1** — aucun
  défaut d'origine de la conception. Ce n'est pas le même défaut qui revient sous une autre forme
  (aucun ne recycle un finding de P1), mais c'est le motif « la sévérité se déplace vers ce qu'on vient
  d'écrire » : déclaré au Project Lead par l'orchestrateur. Aucun découpage de la 15-5a n'est fait
  (quatre modules, sous le seuil ; le découpage de cette passe porte sur la 15-5b, choix C15).
  **Décisions de l'orchestrateur** : C15 à C24 pour cette fiche (C16 révise C13). **Propagation
  post-patch** : `accountNumbers`, `regroupement ou de clôture`, `exempt_ids`, `:634` / `:770` / `925`
  grepés sur les fiches 15-5, 15-5a, 15-5b, 15-5c et le registre.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-5a-validate-prompt-p3.md` ; deux
  lentilles **Sonnet** en contexte frais : **R** chasseur de régressions de la remédiation P2
  (`92770300`), **F** adversaire de périmètre complet ; rotation D6 : P1 Sonnet ×3 → P2 Opus ×2 → P3
  Sonnet ×2). **0 CRITICAL, 0 HIGH.** Bruts : R 0 MEDIUM + 3 LOW, F 1 MEDIUM + 2 LOW ; aucun doublon
  inter-lentilles → **1 MEDIUM et 5 LOW distincts**. Trend : P1 (15-5 entière) 7 MEDIUM / 8 LOW → P2
  5 MEDIUM / 9 LOW → **P3 1 MEDIUM / 5 LOW**.

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | F-1 | MEDIUM | le pont `count`/`numbers` du bras API n'était testé nulle part : T2 teste la clé par `bundle.format`, `reports_e2e.rs` n'appelle pas `init_error_i18n` et n'asserte que le code — `count` passé en chaîne retomberait sur `*[other]`, tests verts | T1 (`count` nombre Fluent), T6 : test de bout en bout dans `opening_balances_e2e.rs` (montage `init_error_i18n`, `:79`), 1 compte / 2 comptes, singulier / pluriel, mutants `count` chaîne et absent | **né de la remédiation** (sélecteur introduit en P1, inscrit en P2) |
  | R3-1 | LOW | AC4 : `imported_supplier_invoices.rs:243` désignait le dépôt homonyme | chemin complet `crates/kesh-api/src/routes/…` | — |
  | R3-2 | LOW | AC3 décrivait au présent la requête réécrite | AC3 distingue la requête actuelle (`:96-105`) et la requête réécrite | — |
  | R3-3 | LOW | registre : C13 dit encore `details.accountNumbers` sans renvoi | ligne de renvoi ajoutée à C13 (l'entrée n'est pas réécrite), vers C16 | — |
  | F-2 | LOW | commentaires « clause / garde `active = TRUE` » rendus inexacts par l'AC3 ; lien `[validate_accounts]` mort | T7 : grep et verdict site par site (réécrits : `kesh-db/src/errors.rs:607-608`, `journal_entries.rs:217`, `:1722`, `journal_entry_reversal_e2e.rs:415` ; justes : `errors.rs:11`, `:484`, `credit_notes.rs:423-424`) — attribution C23 | — |
  | F-3 | LOW | « trié par numéro » ambigu | AC1, T1 : lexicographique, puis identifiant, attendu fixé (C31) | — |

  **Ajouté par décision de l'orchestrateur** (findings R3-3 et F-4 de la P3 de la 15-5b) : l'accesseur
  unique `NonPostableAccounts::details()` (AC1, T1, choix **C29**), que la 15-5b appelle.
  **Signal de la règle de découpage** : sévérité **MEDIUM → MEDIUM** (P2 → P3), mais **un** MEDIUM,
  distinct de ceux de P2, sur la seule surface de test : convergence, pas recyclage — aucun découpage
  (amendement D5) ; déclaré au Project Lead par l'orchestrateur. **Décisions** : C29, C31.
  **Propagation post-patch** : `trié`, `triée par numéro`, `accountNumbers`, `validate_accounts`,
  `clause .active`, `details()` grepés sur les fiches 15-5, 15-5a, 15-5b, 15-5c et le registre.
  **Une passe P4 suit** (un MEDIUM en P3).
- 2026-10-08 — **Implémentation** (`bmad-dev-story`, Claude Opus 5.5) : T0 à T8. Variante
  `DbError::AccountsNotPostable` / code `ACCOUNT_NOT_POSTABLE` / `details.rejected[{accountId,
  accountNumber}]`, message à sélecteur dans les quatre locales, saisie manuelle réécrite (décision en
  Rust, `exempt_ids` retiré), trois gardes de la 24-5 converties (boucle fournisseur en deux passes),
  tests figés réécrits, +16 tests, huit mutations tuées, CHANGELOG `[0.13.0]`, `api-external.md`.
  Gate complet backend 2754/2754, frontend 969/969, E2E 239 passés / 9 échoués — tous jugés (sept
  KF-029, un crochet KF-029 sur un test `fixme`, une pollution). Choix C-15-5a-1 à C-15-5a-4. Statut →
  `review`.
- 2026-10-08 — **Revue de code, clôture** (`bmad-code-review`, Sonnet 5.5 en trois lentilles : Blind Hunter,
  Edge Case Hunter, Acceptance Auditor). Trend : P1 = 0 CRITICAL, 0 HIGH, 0 MEDIUM, 13 LOW (périmètre :
  la branche contre `main`) ; uniquement des LOW, la boucle s'arrête (CLAUDE.md § Review Iteration Rule).
  Sort des LOW :
  - **A-2 corrigé** (documentation seule) : `docs/api-external.md`, lignes `INACTIVE_OR_INVALID_ACCOUNTS`
    (mauvais type du compte de charge) et `ACCOUNT_NOT_POSTABLE` (un compte archivé ou invalide sur une
    autre ligne de la même écriture prime sur un compte non imputable).
  - **Acceptés et écrits** : B-1/E-4, résidu du compte 2979 en cas de panic d'un test ; B-2/E-1,
    non-vacuité de `NonPostableAccounts` seulement en `debug_assert!` (aucun appelant ne peut produire
    une liste vide) ; B-3, `case` redondant ; B-4, doc-comment non reformaté ; E-2, E-3 et A-3, tests HTTP
    absents sur des routes qui propagent l'erreur sans remappage ; A-1, test qui ne mord pas (son jumeau
    « archivé » prouve l'ordre) ; A-4, tutoiement italien « scegli » ; A-5, journaux de gate hors dépôt
    (chiffres déjà au Dev Agent Record : nextest 2754/2754, vitest 969, E2E 239 passés / 9 échoués).
  - Aucune ligne de code de production ni de test touchée par cette clôture. Choix C-15-5a-5. Statut →
    `done`.
