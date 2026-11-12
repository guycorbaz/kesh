# Story 35-3 : un compte désigné aux réglages de facturation et retypé depuis ne reçoit plus d'écriture ; la validation le refuse en le nommant (#537)

Status: ready-for-dev

*Validation terminée (P1 à P4, 2026-11-12).*

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**, tête d'epic : à livrer avant la première facture client de Guy (au plus tard le
2026-11-30). **Aucune migration.** Boucle complète : règle comptable, `kesh-db` touché (gate complet même en cours de
boucle, `CLAUDE.md` § *Exception `kesh-db`*). Code lu sur **`release/0.17.6`** (`4e1c768a`). Registre du kickoff :
`e35-choix-autonomes-e350.md` (C-e350-2, C-e350-4, C-e350-7, C-e350-8) ; registre de la story :
`e35-choix-autonomes-e353.md`, à créer au premier choix. Validations P1, P2 et P3 remédiées (Change Log).*

## Story

En tant que titulaire qui valide une facture client ou une facture fournisseur, je veux que Kesh refuse d'écrire la
créance, la dette ou la TVA sur un compte dont le **type** ne convient plus à ce rôle, et me dise lequel et où le
corriger, afin qu'un compte débiteurs retypé en charge ne reçoive jamais, en silence, la créance de mes ventes.

⚠️ **Kesh tient des livres réels** (0.17.5 sur le NAS). Un compte désigné déjà mal typé laisse aujourd'hui la validation
réussir ; dès la 0.17.7, il la **bloquera**, à la première facture. La story livre donc aussi l'avertissement et la marche
à suivre (AC11, AC13).

## Ce qui se lit au code (`release/0.17.6`)

**Le type n'est contrôlé qu'à la désignation.** `update_invoice_settings` exige, champ par champ, `Asset` pour la
créance, `Revenue` pour le produit, `VAT_SETTING_ACCOUNT_TYPES` pour les trois comptes de TVA (TVA due `Liability`,
récupérable `Asset`, décompte `Liability`) et `Liability` pour les créanciers
(`crates/kesh-api/src/routes/company_invoice_settings.rs:345-414` ; `crates/kesh-db/src/repositories/company_invoice_settings.rs:1279-1283`).
Le type est revérifié à chaque enregistrement des réglages, même pour un compte inchangé (`validate_account_of`,
`:200-237`), avec un message technique : « Compte créance : type de compte incompatible (attendu Asset) »
(`:224-229`). Mais **rien ne relit les réglages quand un compte change de type**.

**Le retypage passe**, par trois chemins. `accounts::update` (`crates/kesh-db/src/repositories/accounts.rs:622`)
commence par `check_role_account_type(changes.role, changes.account_type)` (`:80-105`), qui ne juge que la paire
**demandée**, et rend `Ok` pour un compte sans rôle (`:84-86`) :
- un compte de rôle *Créances clients* retypé en **passif** : le rôle l'admet (`kesh-core/src/chart_of_accounts/mod.rs:161-166`,
  tout rôle de bilan accepte `Asset` ou `Liability`), le réglage exige un actif ;
- un compte retypé en **charge** dans la même requête qui lui retire son rôle (`PUT /accounts/{id}`, full-replace,
  `role: null` admis, `crates/kesh-api/src/routes/accounts.rs:303-307`) ;
- un compte **sans rôle**, désigné par son identifiant (la désignation ne dépend pas du rôle), retypé seul.
Un compte mouvementé exige `confirm_retype` (`accounts.rs:694-736`), un compte vierge non. La garde se fait **par
identifiant désigné**, jamais par numéro ni par rôle (Story 14-3a) ; les numéros `1100`, `2000`, `1171` ci-dessous sont
ceux des montages de test.

**La garde à l'usage existe, sans le type.** La validation d'une facture et celle d'une facture fournisseur verrouillent
les comptes désignés candidats, puis contrôlent ceux que l'écriture écrit :
- `lock_designated_accounts_in_tx` (`company_invoice_settings.rs:921-977`) lit `SELECT id, number, active, postable`
  (`:960-970`) ; son doc-comment écrit l'angle mort : « le **type** du compte (angle mort assumé, préexistant — un compte
  désigné retypé après sa désignation reste utilisé ; le compte d'arrondi, lui, contrôle son type) » (`:902-904`) ;
- `DesignatedAccountsSnapshot::check_written` (`:1010-1040`) **retourne** `DbError::InactiveOrInvalidAccounts` au
  premier compte absent ou archivé, et **accumule** les non imputables (`DbError::DesignatedAccountsNotPostable`, 400
  `ACCOUNT_NOT_POSTABLE`, `crates/kesh-api/src/errors.rs:4571-4587`) ;
- appelants : `invoices::validate_invoice` (`crates/kesh-db/src/repositories/invoices.rs:2194-2199` puis `:2386`) et
  `supplier_invoices::validate_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:1698-1702` puis `:1727`).
  Les rôles : `DesignatedRole::{Receivable, VatPayable, Payable, VatRecoverable}` (`company_invoice_settings.rs:737-782`),
  **`pub(in crate::repositories)`** : invisible de `kesh-api`.

**Inventaire des écritures qui lisent un compte désigné** (validation P1, V1-1 ; à recompter à l'étape 0 par
`git grep` des lectures de `default_*_account_id` hors route des réglages) :

| Site | Réglage lu | Type contrôlé aujourd'hui | Dans cette story |
|---|---|---|---|
| validation facture, `invoices.rs:2194` / `:2386` | créance, TVA due | non | **oui** |
| validation fournisseur, `supplier_invoices.rs:1698` / `:1727` | créanciers, TVA récupérable | non | **oui** |
| solde du reste d'un règlement, `invoice_settlements_write.rs:679-680` → `vat_payable_account_for_write` (`company_invoice_settings.rs:661-691`) | TVA due | non (« sans contrainte de type », `:663-664`) | **non** : C-e350-7, **#1044** |
| avoir, `credit_notes.rs:502`, `:518`, `:666` | TVA due (créance lue sur l'écriture de vente) | non | non (#525, P1 TVA reportés) |
| règlements client, rapprochement | créance lue sur l'écriture de vente | sans objet | — |
| règlement et annulation fournisseur, lots de paiement | créanciers lus sur l'écriture d'achat | sans objet | — |
| arrondi, natures du solde (`usable_designated_account`) | arrondi, escompte, frais, pertes | oui (charge ou produit) | — |

L'import (CAMT, factures importées) et le MCP n'écrivent pas d'écriture sur ces comptes.

## Décisions (C-e350-2)

- **D1 — La garde se met à l'usage**, dans `check_written`, pour les quatre rôles ; le retypage n'est pas refusé.
  Un avertissement au retypage d'un compte désigné est tracé par **#1047** (précédent exact à y nommer :
  l'archivage gardé de la 33-8, `accounts::archive_guarded`, `accounts.rs:859`, et
  `company_invoice_settings::designations_of_account`, `:62`).
- **D2 — Un refus nommé, typé et traduit**, compréhensible par le **Comptable** qui le rencontre (la validation lui est
  ouverte, les réglages non : `errors.rs:4561-4566`). Variante neuve de `DbError` (par exemple
  `DesignatedAccountsWrongType`), rendue en **400 `DESIGNATED_ACCOUNT_WRONG_TYPE`** (nom à confirmer au registre `e353`),
  `details.rejected[{accountId, accountNumber, setting, expectedType, actualType}]`, `setting` parmi `receivable`,
  `vatPayable`, `payable`, `vatRecoverable` ; `expectedType` et `actualType` en valeurs de l'API (`Asset`, `Liability`,
  `Revenue`, `Expense`). ⚠️ Pas `INACTIVE_OR_INVALID_ACCOUNTS`, anonyme : le compte vient des réglages, non de la requête.
  **Forme du message** (V1-2) — aucun sélecteur de pluriel, une phrase par compte refusé puis une phrase d'action,
  assemblées au serveur par `t_args` :
  - par compte, clé `error-designated-account-wrong-type` avec `{ $account }`, `{ $setting }`, `{ $actual }`,
    `{ $expected }`, par exemple : « Le compte 1100, désigné comme compte débiteurs dans Administration → Facturation,
    est un compte de passif ; ce réglage demande un compte d'actif. » ;
  - puis une fois, clé `error-designated-account-wrong-type-action` : « Rien n'a été enregistré. Un administrateur doit
    corriger le réglage concerné — désigner un compte du bon type, ou rendre au compte son type d'origine —, puis la validation
    pourra être relancée telle quelle. » ;
  - les libellés des quatre réglages (« compte débiteurs », « compte de TVA due », « compte créanciers », « compte de TVA
    récupérable ») et des types au singulier (« compte d'actif », « compte de passif », « compte de charge », « compte de
    produit ») sont des clés ×4 ; les `account-type-*` du catalogue (« Actif »…) ne se déclinent pas dans la phrase ;
  - **ordre et séparateur** (V2-10) : les comptes sont rangés **par numéro**, comme `details.rejected` de
    `ACCOUNT_NOT_POSTABLE` ; chaque phrase se termine par un point, et les phrases se joignent par **une espace** ; la
    phrase d'action vient en dernier. `details.rejected` suit le même ordre.
- **D3 — Ordre des refus, par accumulation** (V1-8) : `check_written` parcourt tous les rôles écrits, accumule les
  comptes de mauvais type comme il accumule les non imputables, puis choisit : **absent ou archivé**
  (`InactiveOrInvalidAccounts`, inchangé) > **type** (tous les comptes de mauvais type) > **non imputable**. Un retour
  anticipé sur le type masquerait un compte archivé visité plus tard dans l'ordre
  `Receivable < VatPayable < Payable < VatRecoverable`. Un compte à la fois retypé et non imputable n'est nommé que par le
  refus de type.
- **D4 — Une seule source des types attendus** (V1-5) : une **constante publique** en `kesh-db`, à côté de
  `VAT_SETTING_ACCOUNT_TYPES` (`:1279`), par exemple `CLAIM_SETTING_ACCOUNT_TYPES { receivable: Asset, payable: Liability }`
  (ou une constante qui porte les quatre). `check_written` la lit par un lecteur interne (`DesignatedRole::expected_type`,
  qui reste `pub(in crate::repositories)`) ; la route des réglages la lit à la place de `AccountType::Asset` (`:350`) et
  `AccountType::Liability` (`:407`). Précédent : C-31-5-19.
- **D5 — Le type se lit sous le même verrou**, dans la même requête : ajouter `account_type` **en dernière colonne**
  (`SELECT id, number, active, postable, account_type FROM …`, V2-8) de `:961` et à `LockedDesignatedAccount` (`:806-812`). Les motifs qui reconnaissent la requête restent vrais
  (`ACCESSEUR = ["FROM accounts FORCE INDEX (PRIMARY) WHERE company_id", "LOCK IN SHARE MODE"]`,
  `crates/kesh-db/tests/invoices_validate_vat.rs:1259-1262`, `supplier_invoices_repository.rs:2254-2257` ;
  `rejeu_interblocage_e2e.rs:1306`) ; aucun test ne cherche `number, active, postable`. Deux sondes cherchent `account_type, active FROM accounts` (relecture du
  type d'un compte lié : `bank_account_opening_balance_e2e.rs:1001`, `credit_card_e2e.rs:805`) : la place en dernière colonne
  garantit qu'elles ne reconnaissent pas l'accesseur. Le plan (`FORCE INDEX (PRIMARY)`)
  ne change pas. `LockedDesignatedAccount` est public et lu par l'avoir (`credit_notes.rs:518`) : un champ ajouté ne doit
  rien y changer.
- **D6 — Le message de la page des réglages** (V1-4, limite voisine courte, même route que D4) : « type de compte
  incompatible (attendu Asset) » (`:224-229`) dit le type **en mots** (« attendu : actif »), puisque c'est là que
  l'administrateur corrige. Le message reste en français en dur, comme les autres refus de cette route (#710).
  `validate_account_of` sert aussi l'arrondi et les natures (`resolve_designated_account`, `:259-266`) : leur message
  devient « attendu : charge ou produit ». Le début de phrase « type de compte incompatible » est **gardé** : le test
  `accounts_designated_archive_e2e.rs:1249-1254` l'asserte. La page contrôle les champs en série (`:345-414`) et ne nomme
  que **le premier** fautif : avec deux comptes retypés, il faut enregistrer deux fois (V2-7) — à dire au manuel.

## Critères d'acceptation

1. **AC1 — Créance retypée en passif, rôle gardé.** Montage `seed_accounting_company` (créance `1100`), facture
   brouillon. ⚠️ Le montage **ne pose aucun rôle** (`test_fixtures.rs:130-136`) : le test pose d'abord le rôle
   `Receivable` sur `1100` par `accounts::update`, vérifie qu'il est posé (V2-3) ; puis `accounts::update` retype `1100` en `Liability` **en gardant son rôle** (`confirm_retype = true` s'il est
   mouvementé) ; `validate_invoice` rend la variante de D2, nomme `1100`, `setting = receivable`,
   `expectedType = Asset`, `actualType = Liability` ; **rien n'est écrit** (facture en brouillon, aucune écriture,
   séquence de numérotation inchangée : le numéro, attribué à `invoices.rs:2329`, part avec le rollback). Après le
   retypage, le rôle `Receivable` est **toujours** posé : c'est le chemin que #537 décrit (`accepts_account_type` admet
   le passif pour ce rôle).
2. **AC2 — Créance retypée en charge, rôle retiré** dans la même requête : rôle `Receivable` posé d'abord et vérifié,
   puis un seul `accounts::update` (type `Expense`, rôle `None`) ; le rôle n'est plus posé ; même refus,
   `actualType = Expense`.
   **AC2-bis — Compte sans rôle** désigné comme créance, retypé seul en charge : même refus.
3. **AC3 — TVA due** retypée en actif, facture **avec** TVA : refus, `setting = vatPayable`. Facture **sans** TVA : la
   validation passe (la TVA due n'est pas écrite, `check_written` ne juge que les rôles écrits).
4. **AC4 — Facture fournisseur** : rôle `Payable` posé d'abord sur `2000` et vérifié ; créanciers retypés en actif → refus à la validation, `setting = payable` ;
   TVA récupérable (`1171`) retypée en passif, facture à TVA → refus, `setting = vatRecoverable` ; sans TVA, la
   validation passe. Rien n'est écrit.
5. **AC5 — Ordre** (D3) : un compte retypé **et** archivé rend `InactiveOrInvalidAccounts` ; **créance retypée et TVA due
   archivée** (l'archivé après le retypé dans l'ordre des rôles) rend aussi `InactiveOrInvalidAccounts` ; retypé **et**
   non imputable, le refus de type seul ; deux comptes retypés (créance et TVA due), un refus qui les nomme tous les deux.
6. **AC6 — Le chemin nominal ne bouge pas** : comptes au bon type, la validation passe et écrit la même écriture
   qu'avant (tests existants de `invoices_validate_vat.rs` et `supplier_invoices_repository.rs` verts sans retouche
   de leurs attendus), **à une exception près, nommée** (V2-2) : `one_account_for_two_roles_is_named_once`
   (`invoices_validate_vat.rs:1417-1443`) désigne l'actif `1100` à la fois créance et TVA due ; D3 faisant primer le
   type, il rend désormais le refus de type, `1100` nommé, `setting = vatPayable`, et son attendu change en ce sens.
   Dans un même flux, les deux rôles attendent des types opposés : le dédoublonnage C40 des non imputables désignés
   (`NonPostableAccounts::new`, doc de `check_written`, `company_invoice_settings.rs:1000-1004`) ne s'atteint plus que par
   un montage incohérent ; le doc-comment le dit. Étape 0 : recenser les montages qui désignent un compte d'un type
   contraire au réglage (`git grep -n "SET default_\(receivable\|payable\|vat_payable\|vat_recoverable\)_account_id" --
   'crates/*/tests/*'`, 38 sites sur `release/0.17.6`) et écrire le verdict de chacun au Change Log. Le recensement couvre
   aussi les deux autres formes (V3-4) : `INSERT INTO company_invoice_settings (… default_…)` (quatorze sites de
   `kesh-api/tests`, plus `test_fixtures.rs:157`, `:315`) et `UPDATE accounts SET account_type` (douze tests, dont
   `invoice_write_off.rs:1205`, `reconciliation_e2e.rs:6776`) : pour chacun, dire s'il valide une facture après le retypage.
7. **AC7 — HTTP.** `POST /api/v1/invoices/{id}/validate` et `POST /api/v1/supplier-invoices/{id}/validate` rendent
   400, le code de D2, les `details` de D2, et un message fr-CH qui contient le numéro du compte, le réglage en mots,
   « Administration → Facturation », le type attendu en mots et la phrase d'action. **Cas à deux comptes de types
   attendus différents** (créance retypée en passif, TVA due retypée en actif) : deux phrases justes, une phrase
   d'action, valable aux deux nombres (« le réglage concerné »). **Cas à deux comptes côté achat** (créanciers `2000`
   retypé en actif, TVA récupérable `1171` retypée en passif) : l'ordre se lit — par numéro `1171` puis `2000`, alors que
   l'ordre des rôles donnerait `2000` puis `1171` (V3-3). Tests dans `crates/kesh-api/tests/` ; ces tests montent l'application, donc le catalogue (les libellés fr-CH
   ne s'assertent qu'ici, jamais dans un test unitaire qui lirait l'état i18n global, cf. 35-2a, D10).
8. **AC8 — Les écrans.** Fiche facture : le dialogue de validation se **ferme** sur ce code, comme sur
   `ACCOUNT_NOT_POSTABLE` (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:449-456`), et le message s'affiche
   tel quel. Fiche facture fournisseur : le message s'affiche (`supplier-invoices/[id]/+page.svelte:499`, déjà
   `err.message`). Vitest de la fiche facture.
9. **AC9 — Une source** (D4) : test d'intégration par `PUT /api/v1/company/invoice-settings` (`validate_account_of`
   est privé, `:200`) : pour chacun des quatre champs, un compte du type que donne la constante passe et un compte d'un
   autre type est refusé — la route et `check_written` lisant la même constante, AC1–AC4 et ce test rougissent
   ensemble si l'une diverge. Pour un champ au moins, le message du refus est asserté **en mots** (« attendu : actif »,
   D6) : rouge sur le code de départ par l'assertion, et non par la seule compilation.
10. **AC10 — Quatre langues et G9.** Clés neuves dans `fr-CH`, `de-CH`, `it-CH`, `en-CH` (`parity_between_locales`),
    placées **à côté de `error-designated-account-not-postable`** (C-e350-8) ; aucun sélecteur de pluriel (D2), donc
    rien à `SELECTEURS_RESOLUS_COTE_SERVEUR`. Les replis Rust des clés neuves égalent leur valeur fr-CH et s'inscrivent à
    la table fermée **G9** (`crates/kesh-api/tests/textes_coherents.rs:734`, `les_replis_rust_suivent_le_catalogue`),
    dans le bloc `errors.rs` existant, en fin de bloc (C-e350-8). **Chacune** des dix clés (deux phrases, quatre libellés
    de réglage, quatre libellés de type) porte son littéral de repli, écrit avec l'apostrophe typographique du catalogue
    (`’`), et s'inscrit avec ses constantes de substitution (`{ $account }` → `{compte}`…) : un libellé tiré d'une table
    dynamique ou d'un `format!` de clé ne serait jamais comparé par G9 (`textes_coherents.rs:883-913`) (V3-9). Étape 0 : écrire au Change Log pourquoi
    `error-designated-account-not-postable` n'y figure pas (clé à sélecteur), pour confirmer que les clés neuves, elles,
    y entrent.
11. **AC11 — CHANGELOG et contrat.** Section `## [0.17.7] — Non publié` (règle d'union C-e350-8), *Corrigé*, refs #537,
    **en tête de l'entrée un avertissement** sur le précédent de #429 (`CHANGELOG.md:839`) : « ⚠️ Avant votre prochaine
    facture, ouvrez *Administration → Facturation* et cliquez *Enregistrer* sans rien changer : si un compte désigné n'a
    plus le bon type, la page le nomme ; corrigez-le, sinon la validation des factures sera refusée. » Pour
    l'intégrateur, le **changement de contrat** est écrit comme tel : une validation qui réussissait rend
    `400 DESIGNATED_ACCOUNT_WRONG_TYPE`. `docs/api-external.md` : une ligne neuve du tableau des erreurs **juste après
    `ACCOUNT_NOT_POSTABLE`** (`:1048`), qui dit sa place (après absent ou archivé, avant non imputable — la ligne de
    `ACCOUNT_NOT_POSTABLE` dit aujourd'hui que le refus des comptes désignés « vient après tous les autres » : à
    ajuster). Deux phrases voisines disent le contraire de D3 pour un compte désigné et sont corrigées (V2-4) : `:1047`
    (`INACTIVE_OR_INVALID_ACCOUNTS`, « d'un mauvais type » : hors comptes désignés des réglages) et la dernière phrase
    de `:1048` (« un compte à la fois … d'un mauvais type **et** non imputable rend `INACTIVE_OR_INVALID_ACCOUNTS` »). La
    ligne neuve dit l'ordre complet (absent ou archivé > type > imputabilité) et les deux routes ; les refus de la
    validation fournisseur (`:628`) s'étendent ; le guide n'a pas de liste des refus de la validation facture : la ligne
    neuve suffit.
12. **AC12 — Doc-comments.** Les trois passages qui disent le type non contrôlé sont **reformulés, non effacés** (V1-3) :
    `company_invoice_settings.rs:902-904`, `invoices.rs:2005-2010`, `supplier_invoices.rs:1521-1530`. Le type est
    désormais refusé à l'usage, **mais après le verrou** : un compte désigné retypé en charge et imputé par une ligne
    prend toujours ses deux verrous, et le cycle étroit de `supplier_invoices.rs:1521-1530` reste couvert par le rejeu
    des routes. Les listes d'ordre des refus deviennent fausses et sont mises à jour (V2-6) : le doc de `check_written`
    (« Refuse, dans cet ordre », `company_invoice_settings.rs:996-1004`), les commentaires d'appel `invoices.rs:2381-2385`
    et `supplier_invoices.rs:1722-1726`, et le doc de `validate_invoice` (`invoices.rs:2040-2051`). S'y ajoutent (V3-5) la
    requête recopiée au doc-comment (`company_invoice_settings.rs:842`) et « Pourquoi partagé suffit » (`:847-864`) : le
    verrou garde désormais aussi le **type** lu. Greper `angle mort`, `retypé`, `dans cet ordre`,
    `InactiveOrInvalidAccounts\` ; non imputable` et `number, active, postable` sur `crates/kesh-db/src`, et écrire au
    Change Log le verdict de chaque occurrence.
13. **AC13 — Manuels**, PDF régénérés et contrôlés aplatis. **Manuel d'administration** (`docs/manual/fr/admin-manual.tex`,
    V3-1), le lecteur que le message envoie corriger :
    - `:3816` (« Chacun doit être un compte actif du bon type et imputable … ») : un compte désigné **retypé** depuis fait
      refuser la validation (`400` et le code de D2, nom à confirmer au registre `e353`, message qui nomme le compte) ; `:3818` : la phrase « Après une mise à
      jour vers la v0.13.0, vérifiez … » reçoit son pendant pour la 0.17.7 : ouvrir *Facturation* et cliquer *Enregistrer*
      sans rien changer ; « les avoirs ne sont pas soumis à ce contrôle » reste ;
    - `:3822` (« voient leur type contrôlé à chaque enregistrement, même reconduits ») : la page ne nomme que le premier
      champ fautif ; recommencer jusqu'à ce que l'enregistrement passe.

    **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) :
    - un paragraphe voisin de « Un compte des réglages devenu non imputable » (`:1169`) : le refus, sa cause, et **la
      marche à suivre en deux voies** — désigner dans *Administration → Facturation* un compte du bon type, **ou** rendre
      au compte retypé son type (avec la confirmation demandée s'il porte des écritures) —, puis relancer la validation ;
      et le contrôle préventif d'AC11 (*Enregistrer* sans rien changer) ;
    - la validation fournisseur (`:1788-1796`), même chose pour les créanciers et la TVA récupérable ;
    - « Changer le type d'un compte qui porte des écritures » (`:452-461`) : un compte désigné aux réglages de facturation ne se retype pas sans
      reprendre la désignation — faute d'avertissement à l'écran (D1, **#1047**, V3-6), c'est le seul avertissement ;
    - la note *Rôles des comptes* (`:491`), qui décrit la garde à l'usage par la seule imputabilité : le type y entre ;
      relire la liste « Trois cas échappent encore » (`:493`) ;
    - « recommencez l'enregistrement jusqu'à ce qu'il passe » : la page ne nomme qu'un champ fautif à la fois (D6).
14. **AC14 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e353/captures/`) : le refus à la
    validation d'une facture client ; à celle d'une facture fournisseur ; le refus de la page des réglages (D6).

## Étape 0 : contrôle sur base jetable

Sur une base **jetable** (`kesh_e353`, peuplée par le seed ; **jamais** la base du NAS ni `~/kesh/`), jouer en lecture
seule la requête qui liste les désignations au mauvais type, après avoir retypé un compte désigné par `accounts::update`
(chemin réel), et garder la sortie au journal (`kesh-gate-logs/e353/etape0.log`) :

```sql
SELECT s.company_id, x.setting, a.number, a.account_type, x.expected
FROM company_invoice_settings s
JOIN (SELECT 'receivable' setting, 'Asset' expected UNION ALL SELECT 'vatPayable', 'Liability'
      UNION ALL SELECT 'payable', 'Liability' UNION ALL SELECT 'vatRecoverable', 'Asset') x
JOIN accounts a ON a.id = CASE x.setting
    WHEN 'receivable' THEN s.default_receivable_account_id
    WHEN 'vatPayable' THEN s.default_vat_payable_account_id
    WHEN 'payable' THEN s.default_payable_account_id
    ELSE s.default_vat_recoverable_account_id END
WHERE BINARY a.account_type <> BINARY x.expected;
```

(`account_type` est un `VARCHAR(20)` aux valeurs `Asset|Liability|Revenue|Expense`, `migrations/20260411000001_accounts.sql:11,20`.)
Le même retypage, à l'écran, doit faire refuser l'enregistrement de *Administration → Facturation* sans changement
(D6, AC11) : c'est le contrôle que l'avertissement propose à Guy ; le constater, capture à l'appui. S'il ne se vérifie
pas, s'arrêter et le dire avant d'écrire l'avertissement.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, garder le journal (`kesh-gate-logs/e353/rouges.log`). La variante de D2 et
la constante de D4 n'existent pas au départ : les rouges se prennent sur un **squelette compilable** commité d'abord —
variante, code, bras HTTP, clés et constante déclarés, `check_written` **inchangé** —, si bien que les tests rougissent
par leur attendu, non par la compilation (V3-2) :

- `crates/kesh-db/tests/invoices_validate_vat.rs`, module `garde_usage_comptes_reglage` (`:1236`) : AC1, AC2, AC2-bis,
  AC3, AC5, AC6. **Rouge attendu** : la validation **réussit** et écrit la créance sur le compte retypé — le test
  l'affirme en commentaire, avec l'identifiant de l'écriture produite au journal du rouge.
- `crates/kesh-db/tests/supplier_invoices_repository.rs`, même module (`:2250`) : AC4.
- **Le type se lit frais, sous verrou** (V2-1, patron `retypage_tenu`, `crates/kesh-api/tests/credit_card_e2e.rs:805-822`) :
  une transaction de test prend `1100` `FOR UPDATE` et le retype en `Liability` **sans commit** ; la validation, lancée
  en tâche, est vue en attente sur l'accesseur (`attendre_une_requete_en_cours`, motifs `ACCESSEUR`) ; la transaction
  commite ; la validation rend le refus de type. Rouge au départ (elle réussit), rouge sous M7 (une lecture séparée,
  non verrouillante, lit l'instantané REPEATABLE READ ouvert avant le retypage — `:937-951` — et voit encore un actif).
  Le test précédent (« la sonde `FOR UPDATE NOWAIT` échoue pendant l'attente ») est retiré : ce verrou existe depuis la
  15-5d, le test était vert au départ et ne tuait pas M7.
- `crates/kesh-api/tests/` (fichier existant de la validation, ou neuf) : AC7, AC9.
- Vitest de `invoices/[id]/+page.svelte` : AC8.
- Le retypage passe par `accounts::update`, **le chemin réel**, et non par un `UPDATE accounts SET account_type` posé
  en SQL : c'est le chemin que #537 décrit, et il confirme que le défaut est atteignable (sinon, le dire au Change Log
  avant de continuer). **Exception écrite** : le test du retypage tenu ci-dessus retype en SQL dans une transaction tenue
  ouverte, parce que `accounts::update` commite lui-même et ne peut pas être tenu ; il ne prouve pas l'atteignabilité,
  que AC1 et AC2 prouvent, mais la fraîcheur de la lecture.

**Mutations à jouer et consigner** : M1 le contrôle de type retiré de `check_written` ; M2 l'ordre D3 inversé (type
après non imputable) ; M2-bis un retour anticipé sur le type (AC5, cas archivé après retypé) ; M3 `Receivable` attendu
`Liability` dans la constante ; M4 un des quatre rôles absent du contrôle ; M5 la route qui revient à son
`AccountType::Asset` en dur pendant que la constante change (AC9 rougit) ; M6 le code retiré de la liste de fermeture du
dialogue (AC8) ; M7 le type lu par une requête séparée, hors de la requête verrouillante (test du verrou) ; M8 la liste
rangée par rôle et non par numéro (AC7, côté achat).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la branche de départ, recompter l'inventaire ; contrôle sur base jetable ;
      écrire au Change Log ce qui a bougé.
- [ ] Tests ci-dessus, rouges consignés.
- [ ] `kesh-db` : colonne lue sous verrou, contrôle par accumulation, variante d'erreur et son code, constante publique.
- [ ] `kesh-api` : réponse 400 nommée, clés ×4 et G9, route des réglages sur la constante et message en mots (D6).
- [ ] Écran de la fiche facture ; doc-comments ; documentation, deux manuels, CHANGELOG ; captures.
- [ ] L'avertissement au retypage d'un compte désigné est tracé par **#1047** (V3-6) : le rappeler au registre `e353`.
- [ ] Gates : `test-fast.sh --ci` **complet à chaque commit de code** (exception `kesh-db`), frontend complet, E2E
      complet au dernier commit de code, échecs nommés contre `docs/testing.md`.

## Dev Notes

### Hors périmètre

- **Le solde du reste** (TVA due, `invoice_settlements_write.rs:679-680`) : C-e350-7, **#1044**, qui reprendra la
  constante et le code de cette story.
- **L'avoir** (C35 de la 15-5d, #525 pour la TVA due de l'avoir, l'un des P1 de la TVA reportés).
- **Le compte de produit par défaut** et les comptes de produit des lignes : déjà rejugés au posting, type compris
  (`validate_line_revenue_accounts_in_tx`, `invoices.rs:2201-2238`).
- **Le compte d'arrondi** : contrôle déjà son type (`rounding_account_for_write`).
- **Le compte de décompte TVA** : lu par aucun flux d'écriture (`:905-907`).
- **Un avertissement au retypage** (D1) et **#939** (archivage sans avertissement d'autres désignateurs) : autres
  issues.

### Fichiers communs — règle d'union (C-e350-8)

- Avec la **35-2a** (et la 35-2b, qui part d'elle) : `crates/kesh-api/src/errors.rs` (ici un bras `DbError` neuf, voisin
  de `:4571` ; la 35-2a n'y touche pas, validation P4 de la 35-2a, V4-3),
  `messages.ftl` ×4 (ici à côté de `error-designated-account-not-postable`), `textes_coherents.rs` (G9 : ici en fin du
  bloc `errors.rs`), `user-manual.tex` (ici `:452-461`, `:491`, `:493`, `:1169`, `:1788-1796`), `CHANGELOG.md` (section `[0.17.7]`),
  `docs/api-external.md` (ici après `ACCOUNT_NOT_POSTABLE`), `frontend/src/routes/(app)/invoices/[id]/+page.svelte` et son
  dossier de tests (ici la liste de fermeture du dialogue, `:449-456` ; la 35-2a ne touche pas ce fichier, seulement
  `pdf-error.ts`, V4-9). `admin-manual.tex` (ici `:3816`, `:3818`, `:3822`) n'est touché que par cette story. La règle d'union est celle de
  C-e350-8, recopiée au fichier d'epic.
- Avec la **35-8** (#431, plus tard) : `repositories/company_invoice_settings.rs`. La 35-8 attend la fusion de celle-ci.

### Ce qui doit rester vrai

- Aucune lecture verrouillante neuve : le type se lit dans la requête qui verrouille déjà (D5), sans nouvel ordre de
  verrous ni nouveau cycle (doc-comment de `lock_designated_accounts_in_tx`, § *Pourquoi partagé suffit*).
- Un refus n'écrit rien et ne consomme pas de numéro.
- La page des réglages garde son contrôle : elle exempte un compte inchangé du contrôle *actif* et *imputable*, mais
  **pas du contrôle de type** (`validate_account_of`, `:200-237`) ; seul son message change (D6). Exception connue : le
  compte créanciers absent du corps est conservé sans contrôle (`:394-400`).
- Fixtures synthétiques seulement.

### Références

- Issue #537 et son commentaire (côté dette) ; classement `kesh-gate-logs/epic35-classement-bugs.md` § 7.3 ; validation
  P1 `kesh-gate-logs/e35-3-validation-p1.md`.
- Story 15-5d (#429, garde à l'usage, et son avertissement au CHANGELOG) ; Story 25-2-a (#382, `confirm_retype`) ;
  Story 31-5 (C-31-5-19, source unique des types de TVA) ; Story 33-8 (#486, compte désigné archivé).
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ;
  § *Kesh est en production*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (kickoff de l'Epic 35). Validation non lancée.
- 2026-11-12 — **Validation P1** (Sonnet) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 6 LOW. Prémisses relues au code de
  `release/0.17.6` avant chaque patch ; aucune réfutée. Appliqués :
  - V1-1 : inventaire des écritures sur compte désigné ajouté ; le solde du reste (`invoice_settlements_write.rs:679-680`,
    `vat_payable_account_for_write` lu à `company_invoice_settings.rs:661-691`, « sans contrainte de type ») est laissé
    hors périmètre, avec une issue (C-e350-7) ;
  - V1-2 : forme du message fixée (D2 : une phrase par compte, une phrase d'action, sans sélecteur) ; AC7 à deux comptes ;
  - V1-3 : AC12 vise `invoices.rs:2005-2010`, `supplier_invoices.rs:1521-1530`, `company_invoice_settings.rs:902-904`,
    reformulés et non effacés (l'argument de verrou reste celui du rejeu) ; la cible fausse `invoices.rs:2040-2051` retirée ;
  - V1-4 : avertissement au CHANGELOG (précédent `CHANGELOG.md:839`), contrôle préventif par *Enregistrer* sans changement,
    marche à suivre en deux voies au manuel, contrôle SQL sur base jetable à l'étape 0, changement de contrat écrit pour
    l'intégrateur, message du refus adressé au Comptable (`errors.rs:4561-4566`), message de la page des réglages en mots (D6) ;
  - V1-5 : D4 réduit à la constante publique (`DesignatedRole` est `pub(in crate::repositories)`, `:737`) ; AC9 par
    `PUT /company/invoice-settings` ;
  - V1-6 : troisième chemin, compte sans rôle (`accounts.rs:84-86`), AC2-bis ;
  - V1-7 : G9 à AC10 ; règle d'union des fichiers communs (C-e350-8) ;
  - V1-8 : D3 par accumulation, AC5 avec l'ordre inverse des rôles, M2-bis ;
  - V1-9 : test du verrou et M7 ;
  - V1-10 : `api-external.md` (place du refus), manuel `:452-461`, `:493`, `:1788-1796` ; précédent de la 33-8 nommé en D1 ;
  - V1-11 : M5 rendue vérifiable par AC9 ; pas de M8, le solde du reste étant hors périmètre.
  - Symptômes greppés dans la fiche et le fichier d'epic : « deux chemins », « méthode `DesignatedRole::expected_type` »,
    `invoices.rs:2040-2051`, « ne disent plus que le type est un angle mort » — aucune occurrence restante.
- 2026-11-12 — **Validation P2** (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 6 LOW (`kesh-gate-logs/e35-3-validation-p2.md`).
  Prémisses relues au code de `release/0.17.6` ; aucune réfutée (`test_fixtures.rs:130-136`, montage sans rôle ;
  `credit_card_e2e.rs:805-808`, patron `retypage_tenu` ; `invoices_validate_vat.rs:1422` ; sondes
  `bank_account_opening_balance_e2e.rs:1001`, `credit_card_e2e.rs:805` ; `accounts_designated_archive_e2e.rs:1253`). Appliqués :
  - V2-1 : le test du verrou, vert au départ et aveugle à M7, est remplacé par le patron `retypage_tenu` ; exception écrite à
    la règle « retypage par `accounts::update` » ;
  - V2-2 : AC6 nomme l'exception `one_account_for_two_roles_is_named_once` et son nouvel attendu (refus de type, `1100`,
    `setting = vatPayable`) ; C40 dit inatteignable hors montage incohérent ; recensement des 38 montages à l'étape 0 ;
  - V2-3 : AC1, AC2 et AC4 posent d'abord le rôle (`Receivable`, `Payable`) et le vérifient avant et après le retypage ;
  - V2-4 : `api-external.md:1047` et la dernière phrase de `:1048` corrigées ; « validation facture étendus » retiré ;
  - V2-5 : #1044 nommée (tableau, hors périmètre, registre C-e350-7 annoté) ; V2-6 : listes d'ordre des refus à AC12 ;
    V2-7 : D6 étendu (arrondi et natures, début de phrase gardé pour `:1253`, un champ à la fois) et AC9 asserte le message
    en mots ; V2-8 : `account_type` en dernière colonne ; V2-9 : manuel `:491` ; V2-10 : ordre par numéro, phrases jointes
    par une espace.
  - **Signal de découpage** (`CLAUDE.md`, § *Règle de splitting préventif*) : P1 5 MEDIUM → P2 4 MEDIUM, **sévérité en
    baisse** ; trois des quatre MEDIUM naissent de la remédiation P1 (le test du verrou, AC6 face à un test existant, le
    contrat écrit pour l'intégrateur) et le quatrième (V2-3, montage sans rôle) est d'origine. Le critère de sévérité n'est
    pas atteint, et le périmètre reste à cinq modules (`kesh-db` dépôt et erreurs, `kesh-api` erreurs et route,
    `kesh-i18n`, fiche facture) : **pas de découpage** ; la boucle continue. Constat à déclarer à Guy par l'orchestrateur.
  - Symptômes greppés (fiche, registre, fichier d'epic) : « issue à ouvrir », « issue demandée », « verts sans retouche »
    sans exception, le test de la sonde `NOWAIT`, `postable, account_type, active` — aucun reste hors des entrées du Change Log.
- 2026-11-12 — **Validation P3** (Sonnet) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 8 LOW (`kesh-gate-logs/e35-3-validation-p3.md`).
  **Série 5 → 4 → 1** : la boucle converge ; le MEDIUM est une omission d'origine, non un produit de la remédiation P2.
  Prémisses relues au code de `release/0.17.6` ; aucune réfutée (`admin-manual.tex:3816`, `:3822` ;
  `company_invoice_settings.rs:842`, `:850-852`). Appliqués :
  - V3-1 : le manuel d'administration entre à AC13 (`:3816`, `:3822`), au PDF et à la liste d'union ; le paragraphe « Pour
    les intégrations » de la section de version relève de la préparation de release (noté, non fait ici) ;
  - V3-2 : rouges pris sur un squelette compilable ; V3-3 : cas à deux comptes côté achat, M8 ; V3-4 : recensement des trois
    formes de montage ; V3-5 : `:842` et `:850-866` à AC12, grep de `number, active, postable` ; V3-6 : tâche d'issue,
    issue rédigée pour l'orchestrateur ; V3-7 : 35-2a/35-2b, en-tête, fiche facture à la liste d'union ; V3-8 : « le réglage
    concerné » ; V3-9 : dix littéraux de repli, apostrophe typographique.
  - Symptômes greppés (fiche, registre, fichier d'epic) : « Avec la **35-2** », « ce réglage — », « Validation P1 remédiée »
    à l'en-tête, `d'actif` dans les exemples de message (gardés : ce sont des exemples ; le littéral suit le catalogue,
    AC10) — aucun autre reste.
- 2026-11-12 — **Validation P4** (Haiku, passe ciblée sur `dd9d2517` et `7936e372`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW
  (`kesh-gate-logs/e35-3-validation-p4-ciblee.md`). Appliqués après lecture au code : V4-1 (`company_invoice_settings.rs` relu :
  « Pourquoi partagé suffit » commence à `:847` et finit à `:864`, `:866` ouvre la section suivante : AC12 cite `:847-864`) ;
  V4-2 (AC13 ne fige plus le nom du code avant le registre `e353`) ; V4-3 (`admin-manual.tex:3818`, « Après une mise à jour vers
  la v0.13.0 », relu, ajouté à AC13 et à la liste d'union) ; V4-4 (« 35-2 (fiche parente) »). Remédiation hors code de
  production, sur une passe sans rien au-dessus de LOW.
- 2026-11-12 — **Validation terminée.** Série des MEDIUM : P1 5 → P2 4 → P3 1 → P4 0 (CRITICAL/HIGH à 0 sur les quatre
  passes ; LOW : 6, 6, 8, 4). Modèles : P1 Sonnet, P2 Opus, P3 Sonnet (complètes), P4 Haiku (ciblée). Décisions : garde
  du type à l'usage dans `check_written` (C-e350-2), solde du reste hors périmètre (C-e350-7, #1044), refus nommé
  `DESIGNATED_ACCOUNT_WRONG_TYPE` (nom à confirmer, `e353`) sans sélecteur, ordre par accumulation, constante publique des
  types, avertissement au CHANGELOG et aux deux manuels pour des livres réels, avertissement au retypage tracé par #1047,
  règle d'union C-e350-8. **Signal de découpage non levé** : sévérité décroissante à chaque passe, cinq modules.

