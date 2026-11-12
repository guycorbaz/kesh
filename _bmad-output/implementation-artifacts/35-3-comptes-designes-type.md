# Story 35-3 : un compte désigné aux réglages de facturation et retypé depuis ne reçoit plus d'écriture ; la validation le refuse en le nommant (#537)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**, tête d'epic : à livrer avant la première facture client de Guy (au plus tard le
2026-11-30). **Aucune migration.** Boucle complète : règle comptable, `kesh-db` touché (gate complet même en cours de
boucle, `CLAUDE.md` § *Exception `kesh-db`*). Code lu sur **`release/0.17.6`** (`4e1c768a`). Registre du kickoff :
`e35-choix-autonomes-e350.md` (C-e350-2, C-e350-4) ; registre de la story : `e35-choix-autonomes-e353.md`, à créer au
premier choix. Validation à lancer par l'orchestrateur.*

## Story

En tant que titulaire qui valide une facture client ou une facture fournisseur, je veux que Kesh refuse d'écrire la
créance, la dette ou la TVA sur un compte dont le **type** ne convient plus à ce rôle, et me dise lequel et où le
corriger, afin qu'un compte débiteurs retypé en charge ne reçoive jamais, en silence, la créance de mes ventes.

## Ce qui se lit au code (`release/0.17.6`)

**Le type n'est contrôlé qu'à la désignation.** `update_invoice_settings` exige, champ par champ, `Asset` pour la
créance, `Revenue` pour le produit, `VAT_SETTING_ACCOUNT_TYPES` pour les trois comptes de TVA (TVA due `Liability`,
récupérable `Asset`, décompte `Liability`) et `Liability` pour les créanciers
(`crates/kesh-api/src/routes/company_invoice_settings.rs:345-414` ; `crates/kesh-db/src/repositories/company_invoice_settings.rs:1279-1283`).
Le type est revérifié à chaque enregistrement des réglages (`validate_account_of`, `:200-237`), mais **rien ne relit
les réglages quand un compte change de type**.

**Le retypage passe.** `accounts::update` (`crates/kesh-db/src/repositories/accounts.rs:622`) commence par
`check_role_account_type(changes.role, changes.account_type)` (`:80-105`), qui ne juge que la paire **demandée**. Deux
chemins mènent au défaut :
- un compte de rôle *Créances clients* retypé en **passif** : le rôle l'admet (`kesh-core/src/chart_of_accounts/mod.rs:161-166`,
  tout rôle de bilan accepte `Asset` ou `Liability`), le réglage exige un actif ;
- un compte retypé en **charge** dans la même requête qui lui retire son rôle.
Un compte mouvementé exige `confirm_retype` (`accounts.rs:694-736`), un compte vierge non.

**La garde à l'usage existe, sans le type.** La validation d'une facture et celle d'une facture fournisseur verrouillent
les comptes désignés candidats, puis contrôlent ceux que l'écriture écrit :
- `lock_designated_accounts_in_tx` (`company_invoice_settings.rs:921-977`) lit `SELECT id, number, active, postable`
  (`:960-970`) ; son doc-comment écrit l'angle mort : « le **type** du compte (angle mort assumé, préexistant — un compte
  désigné retypé après sa désignation reste utilisé ; le compte d'arrondi, lui, contrôle son type) » (`:902-904`) ;
- `DesignatedAccountsSnapshot::check_written` (`:1010-1040`) refuse un compte absent ou archivé
  (`DbError::InactiveOrInvalidAccounts`), puis les non imputables (`DbError::DesignatedAccountsNotPostable`, 400
  `ACCOUNT_NOT_POSTABLE`, `crates/kesh-api/src/errors.rs:4571-4587`) ;
- appelants : `invoices::validate_invoice` (`crates/kesh-db/src/repositories/invoices.rs:2194-2199` puis `:2386`) et
  `supplier_invoices::validate_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:1698-1702` puis `:1727`).
  Les rôles : `DesignatedRole::{Receivable, VatPayable, Payable, VatRecoverable}` (`company_invoice_settings.rs:737-782`).

**L'avoir n'est pas concerné** : il emploie le même verrou pour ses propres contrôles et lit la créance sur l'écriture
de vente (`company_invoice_settings.rs:908-913`, C35 de la 15-5d) — une facture émise doit rester annulable.

## Décisions (C-e350-2)

- **D1 — La garde se met à l'usage**, dans `check_written`, pour les quatre rôles ; le retypage n'est pas refusé. Un
  avertissement au retypage d'un compte désigné serait un plus : s'il paraît utile, la story le **demande en issue** au
  rapport, sans l'écrire.
- **D2 — Un refus nommé, typé et traduit.** Une variante neuve de `DbError` (par exemple
  `DesignatedAccountsWrongType`), rendue en **400 `DESIGNATED_ACCOUNT_WRONG_TYPE`** (nom à confirmer au registre
  `e353`) avec `details.rejected[{accountId, accountNumber, setting, expectedType, actualType}]`, `setting` parmi
  `receivable`, `vatPayable`, `payable`, `vatRecoverable`. Le message dit le compte, son rôle aux réglages, le type
  attendu, et **où agir et qui le peut** : *Administration → Facturation*, un administrateur — comme
  `error-designated-account-not-postable`. ⚠️ Pas `INACTIVE_OR_INVALID_ACCOUNTS`, anonyme : le compte vient des réglages,
  non de la requête, et l'utilisateur ne peut pas deviner lequel.
- **D3 — Ordre des refus** : absent ou archivé (`InactiveOrInvalidAccounts`, inchangé, prime) → **type** → non
  imputable. Un compte à la fois retypé et non imputable est nommé par le refus de type.
- **D4 — Une seule source des types attendus.** Les types de la créance et des créanciers sont aujourd'hui écrits en
  dur dans la route (`AccountType::Asset`, `:350` ; `AccountType::Liability`, `:407`). La story les écrit **une fois** en
  `kesh-db` (à côté de `VAT_SETTING_ACCOUNT_TYPES`, ou par une méthode `DesignatedRole::expected_type`) et la route les
  lit, pour que garde et désignation ne divergent jamais (précédent : C-31-5-19).
- **D5 — Le type se lit sous le même verrou**, dans la même requête : ajouter `account_type` à la liste des colonnes
  de `:961` et à `LockedDesignatedAccount` (`:806-812`). ⚠️ Les tests qui reconnaissent la requête à ses motifs
  (`ACCESSEUR = ["FROM accounts FORCE INDEX (PRIMARY) WHERE company_id", "LOCK IN SHARE MODE"]`,
  `crates/kesh-db/tests/invoices_validate_vat.rs:1259-1262`, `supplier_invoices_repository.rs:2254-2257`) ne doivent
  pas cesser de la reconnaître ; le plan (`FORCE INDEX (PRIMARY)`) ne change pas. `LockedDesignatedAccount` est public
  et lu par l'avoir (`credit_notes.rs:518`) : un champ ajouté ne doit rien y changer.

## Critères d'acceptation

1. **AC1 — Créance retypée en passif, rôle gardé.** Montage `seed_accounting_company` (créance `1100`), facture
   brouillon ; `accounts::update` retype `1100` en `Liability` **en gardant son rôle** (`confirm_retype = true` s'il est
   mouvementé) ; `validate_invoice` rend la variante de D2, nomme `1100`, `setting = receivable`,
   `expectedType = Asset`, `actualType = Liability` ; **rien n'est écrit** (facture en brouillon, aucune écriture,
   numéro non consommé : la séquence de numérotation est inchangée).
2. **AC2 — Créance retypée en charge, rôle retiré** dans la même requête : même refus, `actualType = Expense`.
3. **AC3 — TVA due** retypée en actif, facture **avec** TVA : refus, `setting = vatPayable`. Facture **sans** TVA : la
   validation passe (la TVA due n'est pas écrite, `check_written` ne juge que les rôles écrits).
4. **AC4 — Facture fournisseur** : créanciers (`2000`) retypés en actif → refus à la validation, `setting = payable` ;
   TVA récupérable (`1171`) retypée en passif, facture à TVA → refus, `setting = vatRecoverable` ; sans TVA, la
   validation passe. Rien n'est écrit.
5. **AC5 — Ordre** (D3) : un compte retypé **et** archivé rend `InactiveOrInvalidAccounts` ; retypé **et** non
   imputable, le refus de type ; deux comptes retypés (créance et TVA due), un refus qui les nomme tous les deux.
6. **AC6 — Le chemin nominal ne bouge pas** : comptes au bon type, la validation passe et écrit la même écriture
   qu'avant (tests existants de `invoices_validate_vat.rs` et `supplier_invoices_repository.rs` verts sans retouche
   de leurs attendus).
7. **AC7 — HTTP.** `POST /api/v1/invoices/{id}/validate` et `POST /api/v1/supplier-invoices/{id}/validate` rendent
   400, le code de D2, un message fr-CH qui contient le numéro du compte, « Administration → Facturation » et le type
   attendu en mots (« actif », « passif »), et les `details` de D2. Tests dans `crates/kesh-api/tests/`.
8. **AC8 — Les écrans.** Fiche facture : le dialogue de validation se **ferme** sur ce code, comme sur
   `ACCOUNT_NOT_POSTABLE` (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:449-456`), et le message s'affiche
   tel quel. Fiche facture fournisseur : le message s'affiche (`supplier-invoices/[id]/+page.svelte:499`, déjà
   `err.message`). Vitest de la fiche facture.
9. **AC9 — Une source** (D4) : la route des réglages lit les types attendus de `kesh-db` ; un test épingle que
   `validate_account_of` et `check_written` exigent le même type pour chacun des quatre rôles.
10. **AC10 — Quatre langues.** Clés neuves dans `fr-CH`, `de-CH`, `it-CH`, `en-CH` (`parity_between_locales`) ;
    libellés des types repris de `account-type-*` (`messages.ftl:211-214` en fr-CH) ou d'une clé propre ; si un
    sélecteur de pluriel nomme plusieurs comptes, sa clé s'inscrit à `SELECTEURS_RESOLUS_COTE_SERVEUR` (kesh-i18n), et
    le nombre se passe en NOMBRE Fluent (précédent : `account_not_postable_response`, `errors.rs:1553-1579`).
11. **AC11 — Documentation.** `docs/api-external.md` : ligne du tableau des erreurs (`:1048` voisine) et refus de la
    validation fournisseur (`:628`). Manuel utilisateur : un paragraphe voisin de « Un compte des réglages devenu non
    imputable » (`docs/manual/fr/user-manual.tex:1169`), qui dit le refus, sa cause (retypage) et la marche à suivre ;
    la même chose pour la facture fournisseur là où le manuel décrit sa validation (à trouver par grep). Le doc-comment
    de `lock_designated_accounts_in_tx` (`:902-904`) et celui de `validate_invoice` (`invoices.rs:2040-2051`) ne disent
    plus que le type est un angle mort. PDF régénéré, contrôlé aplati. CHANGELOG `## [0.17.7] — Non publié`,
    *Corrigé*, refs #537.
12. **AC12 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e353/captures/`) : le refus à la
    validation d'une facture client ; à celle d'une facture fournisseur.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, garder le journal (`kesh-gate-logs/e353/rouges.log`) :

- `crates/kesh-db/tests/invoices_validate_vat.rs`, module `garde_usage_comptes_reglage` (`:1236`) : AC1, AC2, AC3, AC5,
  AC6. **Rouge attendu** : la validation **réussit** et écrit la créance sur le compte retypé — le test l'affirme en
  commentaire, avec l'identifiant de l'écriture produite au journal du rouge.
- `crates/kesh-db/tests/supplier_invoices_repository.rs`, même module (`:2250`) : AC4.
- `crates/kesh-api/tests/` (fichier existant de la validation, ou neuf) : AC7.
- Vitest de `invoices/[id]/+page.svelte` : AC8.
- Le test de AC9.
- Le retypage passe par `accounts::update`, **le chemin réel**, et non par un `UPDATE accounts SET account_type` posé
  en SQL : c'est le chemin que #537 décrit, et il confirme que le défaut est atteignable (sinon, le dire au Change Log
  avant de continuer).

**Mutations à jouer et consigner** : M1 le contrôle de type retiré de `check_written` ; M2 l'ordre D3 inversé (type
après non imputable) ; M3 `Receivable` attendu `Liability` ; M4 un des quatre rôles absent du contrôle ; M5 la route
qui revient à son `AccountType::Asset` en dur, la source de D4 changée (AC9 rougit) ; M6 le code retiré de la liste de
fermeture du dialogue (AC8).

## Tâches

- [ ] Étape 0 : relire les lignes citées sur la branche de départ ; écrire au Change Log celles qui ont bougé.
- [ ] Tests ci-dessus, rouges consignés.
- [ ] `kesh-db` : colonne lue sous verrou, contrôle et ordre, variante d'erreur et son code, source unique des types.
- [ ] `kesh-api` : réponse 400 nommée, clés ×4, route des réglages sur la source unique.
- [ ] Écran de la fiche facture ; documentation, manuel, CHANGELOG ; captures.
- [ ] Gates : `test-fast.sh --ci` **complet à chaque commit de code** (exception `kesh-db`), frontend complet, E2E
      complet au dernier commit de code, échecs nommés contre `docs/testing.md`.

## Dev Notes

### Hors périmètre

- **L'avoir** (C35 de la 15-5d, #525 pour la TVA due de l'avoir, l'un des P1 de la TVA reportés).
- **Le compte de produit par défaut** et les comptes de produit des lignes : déjà rejugés au posting, type compris
  (`validate_line_revenue_accounts_in_tx`, `invoices.rs:2201-2238`).
- **Le compte d'arrondi** : contrôle déjà son type (`rounding_account_for_write`).
- **Le compte de décompte TVA** : lu par aucun flux d'écriture (`:905-907`).
- **Un avertissement au retypage** (D1) et **#939** (archivage sans avertissement d'autres désignateurs) : autres
  issues.

### Fichiers communs

- Avec la **35-2** (C-e350-4) : `crates/kesh-api/src/errors.rs` (ici un bras `DbError` neuf, voisin de `:4571` ; la 35-2
  touche `InvoiceNotPdfReady` et `build_response`), les quatre `messages.ftl` (clés distinctes), `user-manual.tex` (ici
  vers l. 1169 ; la 35-2, l. 1251).
- Avec la **35-8** (#431, plus tard) : `repositories/company_invoice_settings.rs`. La 35-8 attend la fusion de celle-ci.

### Ce qui doit rester vrai

- Aucune lecture verrouillante neuve : le type se lit dans la requête qui verrouille déjà (D5), sans nouvel ordre de
  verrous ni nouveau cycle (doc-comment de `lock_designated_accounts_in_tx`, § *Pourquoi partagé suffit*).
- Un refus n'écrit rien et ne consomme pas de numéro.
- La page des réglages ne change pas. Elle exempte un compte inchangé du contrôle *actif* et *imputable*, mais **pas
  du contrôle de type** (`validate_account_of`, `:200-237`) : une fois un compte désigné retypé, aucun enregistrement
  des réglages ne passe tant que le champ fautif n'est pas corrigé — à vérifier à l'étape 0 sur l'écran
  (`settings/invoicing`), et à écrire au manuel avec la marche à suivre : désigner un compte du bon type, puis relancer
  la validation. (Exception connue : le compte créanciers absent du corps est conservé sans contrôle, `:394-400`.)
- Fixtures synthétiques seulement.

### Références

- Issue #537 et son commentaire (côté dette) ; classement `kesh-gate-logs/epic35-classement-bugs.md` § 7.3.
- Story 15-5d (#429, garde à l'usage) ; Story 25-2-a (#382, `confirm_retype`) ; Story 31-5 (C-31-5-19, source unique
  des types de TVA) ; Story 33-8 (#486, compte désigné archivé).
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (kickoff de l'Epic 35). Validation non lancée.
