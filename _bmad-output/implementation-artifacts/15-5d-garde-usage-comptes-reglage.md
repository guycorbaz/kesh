# Story 15.5d : Les comptes de réglage contrôlés à l'usage — créance, TVA, créanciers, et le compte créanciers à l'écran

Status: ready-for-dev

<!-- Quatrième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P4 de la 15-5b
     (finding F4-3, choix C33 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b l'ancien AC20
     (garde à l'usage des comptes de réglage, choix C27 et C28, révision de la limite L2 de D-A0) et tout
     ce qui s'y rattache — la variante `DesignatedAccountsNotPostable`, sa clé i18n, ses tests, ses
     passages de manuel —, plus la remédiation des deux HIGH et du MEDIUM que cette règle a fait naître
     en P4 : le compte créanciers exposé à l'écran (C34, révise C25) et l'avoir exempté avec sa vraie
     raison (C35). Choix applicables : C6, C16, C19, C25 (révisé par C34), C27, C28 (texte révisé par
     C36), C29, C33, C34, C35, C36, C39, C40, C41. Historique complet au Change Log.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation (les 15-5b et
     15-5c portent le même) — il n'existe pas de statut « en validation » dans `sprint-status.yaml`. Le
     développement attend la clôture de la boucle de validation ET le merge des 15-5a et 15-5b
     (finding R1-7 de la P1). -->

**Issues** : **ferme #429** (P1 — les comptes désignés dans les réglages de facturation ne sont
re-validés nulle part). La PR porte `closes #429` (mot-clé **dans la PR**, le dépôt merge en squash).
La 15-5b, qui garde ces comptes **à la désignation**, ne fait que `refs #429` : la désignation seule
laisse un réglage devenu non imputable recevoir des écritures (finding P3 F-1 de la 15-5b), et c'est
l'usage — cette story — qui ferme le défaut. Voisines citées, **non traitées ici** : **#473** et
**#523** (l'avoir relit la créance et le compte d'arrondi dans les réglages du moment — 15-6a),
**#525** (l'avoir contre-passe la TVA sur la TVA due des réglages du moment — report TVA).

**Dépend de la 15-5a** (le newtype `NonPostableAccounts`, son constructeur `details()` — C29 —, le
code `ACCOUNT_NOT_POSTABLE`) **et de la 15-5b** (l'AC19 : `defaultPayableAccountId` absent du corps
est préservé, `Option<Option<i64>>` ; l'AC10 : les six champs refusent un compte non imputable à la
désignation ; la parenthèse « de regroupement, de résultat ou de clôture » de `validate_account_of`).
Ne pas commencer avant le merge de la 15-5b. **Indépendante de la 15-5c** : elles touchent des
passages distincts du manuel ; seule la borne `sitesTotal` de `i18n-keys.test.ts` est commune — celle
qui merge en second la relève sur l'état rebasé.

## Story

En tant que **comptable ou administrateur d'une société tenue dans Kesh**,
je veux que **la validation d'une facture et la saisie d'une facture fournisseur refusent d'écrire
sur un compte de réglage devenu non imputable, avec un message qui dit où le corriger — et que cet
endroit existe à l'écran pour chacun de ces comptes**,
afin qu'**aucune écriture automatique n'atterrisse sur un compte de regroupement, de résultat ou de
clôture**, sans pour autant bloquer un flux sans recours dans l'application.

### Pourquoi une story à part

La garde à l'usage est une **règle métier neuve** — la révision de la limite **L2** de la décision
D-A0 (14-3b) pour quatre comptes —, pas le report d'un patron : elle a sa variante d'erreur, sa clé
i18n à sélecteur, deux flux de `kesh-db` sans rapport avec le rapprochement, deux écrans à relire et
deux manuels. Elle avait été ajoutée à la 15-5b par la remédiation de sa validation P3 ; la passe P4 a
trouvé **deux HIGH et un MEDIUM nés d'elle** (le refus des créanciers renvoyait à un champ absent de
l'écran ; l'avoir écrivait sur deux de ces comptes sans garde et la fiche le disait « snapshot de la
facture »), et a relevé qu'elle n'avait rien d'un rollout (finding F4-3). Rien du reste de la 15-5b
n'en dépend. D'où le découpage (choix C33).

## Acceptance Criteria

### La garde à l'usage (#429)

1. **AC1 — Un compte de réglage devenu non imputable est refusé quand un flux veut y écrire**
   (choix C27). L'exemption « inchangé » de l'AC11 de la 15-5b laisse en place un réglage devenu non
   imputable (ajout d'un sous-compte, règle 14-3a) ; les flux #8 et #10 de l'inventaire (a) de la 15-5b
   y écrivent avec `enforce_postable = false`. Sur le patron du compte d'arrondi —
   `rounding_account_for_write` / `usable_designated_account`
   (`crates/kesh-db/src/repositories/company_invoice_settings.rs:400`, `:484-510`, requête `:499`),
   relu **au moment d'écrire**, dans la transaction, `FOR UPDATE` — :
   - **le mécanisme du prédicat — les rôles rendus par le générateur** (findings R1-2 / F2 de la P1,
     choix **C39**). Le générateur de lignes **expose l'ensemble des rôles de réglage qu'il a
     effectivement écrits**, et la garde ne contrôle **que les comptes de ces rôles** : aucun recalcul
     de la TVA hors du générateur (règle DRY), aucune inspection des `account_id` de `entry_lines`
     (ambiguë dès qu'un même compte joue deux rôles, ou qu'un compte de réglage coïncide avec un compte
     de produit ou de charge d'une ligne — les fixtures le font : `test_fixtures.rs:158-169` réutilise
     `2000` pour la TVA due). Concrètement : un type `DesignatedRole` (`Receivable`, `VatPayable`,
     `Payable`, `VatRecoverable`) et un ensemble ordonné de rôles (`BTreeSet<DesignatedRole>` ou
     newtype équivalent) rendus **avec** les lignes —
     - côté vente, par `generate_invoice_journal_lines` (`invoices.rs:1784`) : `Receivable` toujours,
       `VatPayable` **dans la branche même** `if total_vat > Decimal::ZERO` (`:1851`) qui écrit les
       lignes de TVA due ; `generate_invoice_journal_lines_rounded` (`:1879`) le transmet ;
     - côté achat, par `generate_purchase_journal_lines` (`supplier_invoices.rs:105`, qui rend déjà un
       tuple `(lignes, total TTC)`) : `Payable` toujours, `VatRecoverable` dans la branche
       `if total_vat > Decimal::ZERO` (`:129`) ;
     - l'appelant traduit chaque rôle en l'identifiant qu'il a lui-même passé au générateur (créance,
       TVA due, créanciers, TVA récupérable des réglages) et appelle l'accesseur sur ces identifiants.
       Les **autres appelants** — 25 occurrences de
       `grep -rnE "generate_(invoice_journal_lines(_rounded)?|purchase_journal_lines)\(" crates | grep -v "///"`
       sur `67c31c95`, définitions et tests compris, dont l'**avoir** (`credit_notes.rs:933`) —
       ignorent les rôles ; l'avoir les ignore **délibérément** (C35, AC3) ;
   - un accesseur neuf de `company_invoice_settings` (p. ex.
     `check_designated_accounts_postable_in_tx(conn, company_id, ids: &[i64])`) lit
     `id, number, active, postable` des comptes reçus, en **une seule requête**
     `WHERE company_id = ? AND id IN (…) ORDER BY id FOR UPDATE` — l'**ordre de verrouillage par
     identifiant** est fixé, pour que deux flux concurrents ne prennent jamais les mêmes lignes dans
     des ordres opposés (finding R1-6) —, et rend une erreur pour ceux qui sont **de la société, actifs
     et non imputables**. Un compte absent, d'une autre société ou **archivé** n'est **pas** refusé par
     lui : il suit le chemin d'aujourd'hui (`InactiveOrInvalidAccounts` dans `create_in_tx`), seul cas
     où la 15-5a interdit de nommer un compte ;
   - **un même compte désigné pour deux rôles est nommé une fois** (finding F1 de la P1, choix
     **C40**). Le dédoublonnage n'est **pas** réécrit dans l'accesseur : il est garanti par
     `NonPostableAccounts::new`, que la 15-5a définit comme trieur **et dédoublonneur par
     identifiant** (`15-5a-refus-non-imputable.md`, AC1, l. 48-53 : « qui **trie par numéro** […]
     **puis par identifiant** à numéro égal, et **dédoublonne par identifiant** » ; l. 62-63 : « la
     variante ne peut être construite qu'avec une liste triée, dédoublonnée et non vide »). L'accesseur
     construit la variante par ce seul constructeur ; `count` vaut donc le nombre de comptes
     **distincts**, et le message prend le singulier quand deux rôles pointent sur le même compte.
     Un test le fige (AC7) ;
   - **validation d'une facture** (`crates/kesh-db/src/repositories/invoices.rs`, `validate_invoice`) :
     la garde se place **entre la génération des lignes** (`generate_invoice_journal_lines_rounded`,
     `:2247-2253`) **et `create_in_tx`** (`:2255`) — finding R4-3/F4-5 —, sur les rôles rendus par le
     générateur : la **créance** (`default_receivable_account_id`) toujours ; la **TVA due**
     (`default_vat_payable_account_id`) **seulement si le générateur l'a écrite**, c'est-à-dire si
     `total_vat > 0` (`invoices.rs:1849-1852`, calculé sur les montants **arrondis par ligne**) — **pas**
     « une ligne à taux > 0 » : une facture à taux positif dont la TVA arrondit à zéro n'écrit rien sur
     la TVA due et n'est pas bloquée (finding F4-4 ; cas existant `gen_lines_rate_rounds_to_zero`).
     Conséquence de cette place, **écrite dans le doc-comment « # Erreurs » de `validate_invoice`** :
     tous les refus antérieurs gardent leur priorité — total nul, montant minimum, compte d'arrondi,
     comptes de produit des lignes (2 ter, `ACCOUNT_NOT_POSTABLE` de la 15-5a), exercice — et le
     `ConfigurationRequired("default_vat_payable_account_id")` du générateur (TVA due **absente**) passe
     avant ce refus (TVA due **non imputable**) ;
   - **création d'une facture fournisseur** (`crates/kesh-db/src/repositories/supplier_invoices.rs`,
     `create_in_tx` — donc aussi la **complétion d'une facture importée**, qui l'appelle,
     `routes/imported_supplier_invoices.rs:243`) : même place, **entre** `generate_purchase_journal_lines`
     (`:365-369`) **et** `journal_entries::create_in_tx` (`:372`), sur les rôles rendus : les
     **créanciers** (`default_payable_account_id`, valeur résolue par l'AC19 de la 15-5b) toujours ; la
     **TVA récupérable** (`default_vat_recoverable_account_id`) **seulement si le générateur l'a
     écrite** (TVA totale positive, `:129-131`). Le compte de **charge** non imputable est refusé plus
     tôt, par la 15-5a (`supplier_invoices.rs:337-346`) : son refus passe avant ;
   - un seul refus nomme **tous les comptes de réglage** en défaut du flux (pas ceux des lignes, refusés
     plus tôt) : variante neuve `DbError::DesignatedAccountsNotPostable(NonPostableAccounts)`
     (`crates/kesh-db/src/errors.rs`), même code **`ACCOUNT_NOT_POSTABLE`**, HTTP 400, `details` =
     `NonPostableAccounts::details()` (C29) — un seul contrat pour l'intégrateur (choix C28).
2. **AC2 — Le message dit où agir, et qui peut le faire** (choix C28, texte révisé par C36). Clé neuve
   `error-designated-account-not-postable`, quatre locales, sélecteur `[one]` / `*[other]` inscrit à
   `SELECTEURS_RESOLUS_COTE_SERVEUR` (`crates/kesh-i18n/src/loader.rs:367`, patron de l'AC2 de la
   15-5a), `count` passé comme nombre ; texte FR **exact** :
   ```ftl
   error-designated-account-not-postable = { $count ->
       [one] Le compte { $numbers }, désigné dans Paramètres → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable.
      *[other] Les comptes { $numbers }, désignés dans Paramètres → Facturation, ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à leur place des comptes imputables.
   }
   ```
   - « un compte imputable » et non « un sous-compte » : le remède « sous-compte » ne vaut que pour un
     compte de regroupement, non pour un compte de résultat ou de clôture (finding R4-5) ;
   - « un administrateur doit » : la validation d'une facture et la saisie fournisseur sont ouvertes au
     Comptable, la page des réglages est réservée à l'Admin (`crates/kesh-api/src/lib.rs:207-211`) — le
     message vaut pour les deux rôles sans branche d'écran (finding R4-5/F4-6, choix C36) ;
   - DE/IT/EN sur le vocabulaire de la 15-5a (« bebuchbar », « registrabile », « postable ») et le
     libellé du menu tel que chaque locale l'affiche ; test Rust par locale, au singulier et au pluriel.
3. **AC3 — Ce qui ne change pas, et c'est écrit** (doc-comment de l'accesseur et Dev Notes) :
   - `create_in_tx` garde `enforce_postable = false` — la garde est en amont, sur les seuls comptes de
     réglage, et le **compte bancaire** (compte de configuration lui aussi) reste utilisable devenu non
     imputable (D-A0, AC15 de la 15-5b, C6) ;
   - le **compte de produit par défaut** reste exempté (D3-bis de la 16-1a, `invoices.rs:574-579`) ;
   - le compte de **décompte TVA** n'est lu par aucun flux d'écriture : seul l'export CSV le lit
     (`crates/kesh-api/src/exports/csv_tables.rs:932`) ; le rapport TVA **ne le lit pas** — il lit la
     TVA due et la TVA récupérable, sans écrire (`crates/kesh-report/src/vat_report.rs:173-174`)
     (finding F3 de la P1) ;
   - le **solde du reste** garde son refus `ConfigurationRequired` pour une TVA due inutilisable
     (`vat_payable_account_for_write`, `company_invoice_settings.rs:454`) — divergence de code assumée
     et écrite (C28) ;
   - **l'avoir est exempté de la garde à l'usage, délibérément** (choix C35, findings R4-2/F4-2). Fait
     vérifié : l'avoir ne reprend de la facture que ses **comptes de produit** (snapshot des lignes,
     D5-bis, `crates/kesh-db/src/repositories/credit_notes.rs:503-505`) ; la **créance** et la **TVA
     due** sont **relues dans les réglages du moment** (`:360-364`, `:507-512`), puis postées par
     `create_in_tx(…, false)` (`:548`). Il n'est pas gardé ici parce que ces deux lectures sont
     elles-mêmes **le défaut** à corriger, et qu'une garde posée dessus serait défaite par leur
     correction : la **créance** sera lue **sur l'écriture de vente** par la 15-6a (#473, et #523 pour
     le compte d'arrondi), et la **TVA due** relève de #525 (report TVA). Garder aujourd'hui le compte
     des réglages bloquerait l'annulation d'une facture sur un compte que l'avoir **ne devrait pas
     lire** — l'argument de l'AC15 de la 15-5b (une pièce émise doit rester annulable) s'y ajoute.
4. **AC4 — Les écrans affichent le refus.** Trois `catch`, et trois seulement (findings R1-1 / F3 de
   la P1) :
   - **validation d'une facture** : `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `catch` de
     `confirmValidate`, `:402-415` — `validateError = err.message` pour un `ApiError`, avec une branche
     propre à `CONFIGURATION_REQUIRED` (`:409-411`, suffixe « Configurez les comptes par défaut ») que
     `ACCOUNT_NOT_POSTABLE` **ne doit pas** emprunter : son message dit déjà où agir ;
   - **saisie d'une facture fournisseur** : `frontend/src/routes/(app)/supplier-invoices/+page.svelte:213`
     (`if (isApiError(err)) formError = err.message;`). Les `catch` de `:103` (scan du QR,
     `supplier-invoices-scan-failed`) et de `:133` (chargement de la page) **ne sont pas** concernés :
     aucun des deux n'atteint `create_in_tx` ;
   - **complétion d'un import** : `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte:267`
     (`formError = completeErrorLabel(err)`), dont le `switch` replie sur `err.message` par son
     `default:` (`:301`) — `ACCOUNT_NOT_POSTABLE` ne doit **pas** y recevoir de branche qui en
     changerait le texte.
   Le dev vérifie chacun à la lecture, le consigne au Dev Agent Record, et corrige ici un écran qui
   afficherait un repli générique. Aucune branche par rôle n'est ajoutée : le message de l'AC2 vaut pour
   les deux. **Un test d'écran le fige** (AC7, finding R1-3 / F4 de la P1, choix **C41**).

### Le compte créanciers, à l'écran (finding R4-1/F4-1)

5. **AC5 — `defaultPayableAccountId` s'affiche et s'enregistre dans *Paramètres → Facturation***
   (choix C34, qui révise C25). Sans lui, le refus de l'AC1 pour les **créanciers** renvoie à un champ
   qu'aucun écran ne montre : la saisie de **toute** facture fournisseur et la complétion de **tout**
   import échouent, et le seul recours est un `PUT` à la main avec un cookie de session Admin
   (`grep -rn "defaultPayableAccountId" frontend/src | grep -v test` → aucune sortie sur `007c4eb1`).
   Dans `frontend/src/routes/(app)/settings/invoicing/+page.svelte` :
   - un `<select>` **Compte créanciers (Passif)** dans la section des comptes par défaut, à côté de la
     TVA due (`:355-369`), `data-testid="settings-payable-account"`, clé neuve
     `settings-invoicing-payable-account` dans les quatre locales (« Compte créanciers (Passif) » ;
     DE/IT/EN sur le patron de `invoices-settings-vat-payable`, `messages.ftl:502` / `:508`) ;
   - ses options : `withCurrentAccount(liabilityAccounts, payableId, accounts)` — **même filtre que la
     TVA due** (`liabilityAccounts` : `active && postable && accountType === 'Liability'`, `:68-70` ; le
     serveur exige le type `Liability`, AC19 de la 15-5b) ; la **valeur courante est préservée** même si
     le compte est devenu non imputable (patron #271, `:80-87`) : elle reste affichée et sélectionnée,
     et l'exemption « inchangé » de l'AC11 de la 15-5b la laisse passer à l'enregistrement ;
   - la valeur est **lue** au chargement (`onMount`, `:122-145`) et à la relecture sur conflit de
     version (`:199-214`), et **envoyée** par `updateInvoiceSettings` (`:174-189`) ;
   - les types `InvoiceSettingsResponse` et `UpdateInvoiceSettingsRequest`
     (`frontend/src/lib/features/invoices/invoices.types.ts:119-162`) gagnent
     `defaultPayableAccountId: number | null` — le serveur le rend déjà
     (`crates/kesh-api/src/routes/company_invoice_settings.rs:43`, `:70`) ;
   - le **mock** de `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
     (fonction `settings()`, `:60-75`, champs listés à partir de `:63` — `defaultVatPayableAccountId: null`
     à `:67`) gagne `defaultPayableAccountId: null` : sans lui, le champ devenu obligatoire du type casse
     `npm run check` (finding R1-5 de la P1).
   Les numéros de ligne de cet AC (et de l'AC8) sont relevés sur `67c31c95`, **avant** le merge de la
   15-5b, qui touche ces fichiers : T0 les refait (finding R1-7 de la P1).
   L'AC19 de la 15-5b (absent du corps → préservé) **reste** le filet pour un client qui ne l'envoie
   pas (onglet ouvert avant la mise à jour, intégration) ; choisir « — Sélectionner — » l'efface,
   comme pour les autres champs (`null`).
6. **AC6 — Les contournements des E2E sont retirés s'ils deviennent inutiles.**
   `frontend/tests/e2e/payment-batches.spec.ts:57-64` et `frontend/tests/e2e/inbox-import.spec.ts:91-97`
   reposent le compte créanciers par un `PUT` d'API quand il vaut `null` — contournement de #521. Après
   l'AC19 de la 15-5b, aucun enregistrement ne l'efface plus ; le dev **vérifie** que le seed E2E le
   désigne (`insert_with_defaults`, rôle `Payable`), puis **juge le retrait sur la suite E2E
   complète** — celle du gate D7 de T7, base fraîchement seedée —, **pas** sur les deux specs isolées
   (finding R1-4 de la P1) : une autre spec qui enregistre les réglages avant elles pourrait effacer le
   compte créanciers, et deux specs vertes seules ne le verraient pas. Suite complète sans échec neuf
   imputable à ces deux fichiers (jugée fichier par fichier contre `docs/testing.md` § « Les échecs
   attendus ») → le contournement est retiré ; sinon il reste, avec un commentaire qui dit pourquoi. Le
   résultat est consigné au Dev Agent Record.

### Ce qui doit être prouvé

7. **AC7 — Tests, un par compte, et le chemin de correction à l'écran.**
   - `kesh-db` (fichiers de tests de la validation de facture, p. ex.
     `crates/kesh-db/tests/invoices_validate_vat.rs`, et `crates/kesh-db/tests/supplier_invoices_repository.rs`) ;
     le compte de test ne diffère d'un compte accepté **que par `postable`** et est **désigné avant**
     d'être rendu non imputable (le chemin réel : l'exemption de l'AC11 de la 15-5b l'a laissé en
     place) :
     - **créance** non imputable → validation refusée, `DesignatedAccountsNotPostable` nommant
       `(id, n°)`, facture toujours brouillon, aucune écriture ;
     - **TVA due** non imputable : facture **avec** TVA → refusée ; facture **sans** TVA → validée ;
       facture **à taux positif dont la TVA arrondit à zéro** → validée (finding R4-3/F4-4) ;
     - **ordre** : facture dont une **ligne** et la **créance** sont non imputables → le refus de la
       15-5a (ligne) ; TVA due **absente** → `ConfigurationRequired` (finding F4-5) ;
     - **créanciers** non imputable → saisie de facture fournisseur refusée, rien d'écrit ;
     - **TVA récupérable** non imputable : facture fournisseur **avec** TVA → refusée ; **sans** TVA
       → acceptée ;
     - créance **et** TVA due non imputables, désignées sur **deux comptes distincts** (`1100` et
       `2000` de `seed_accounting_company`) → **un** refus nommant les deux, `count = 2`, pluriel
       (finding F1 de la P1) ;
     - **un même compte désigné pour deux rôles** (créance et TVA due posées sur le même identifiant
       par un `UPDATE company_invoice_settings` direct dans le montage, comme le fait
       `test_fixtures.rs:158-169`), rendu non imputable, facture avec TVA → **un** refus qui le nomme
       **une fois**, `count = 1`, message au **singulier** (choix C40 ; il prouve que le
       dédoublonnage de `NonPostableAccounts::new` est bien atteint par l'accesseur) ;
     - compte de réglage **archivé** → `InactiveOrInvalidAccounts`, inchangé (la variante n'est pas
       émise) ;
     - **avoir** (`crates/kesh-db/tests/credit_notes_repository.rs`) sur une facture dont la
       créance des réglages est devenue non imputable → émis (C35 :
       l'exemption est voulue et un test la fige) ;
   - `kesh-api`, deux fichiers (finding R1-3 de la P1) :
     - **validation d'une facture** — `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`,
       fichier **créé par la 15-5b** (absent de `67c31c95`), dont on reprend le montage tel que la
       15-5b l'écrit (T5 de la 15-5b : `create_seeded_company`, patron `idor_multi_tenant_e2e.rs:~751`),
       avec `init_error_i18n` ; route `POST /api/v1/invoices/{id}/validate` (`lib.rs:509`), jeton
       d'un utilisateur **Comptable** (le message doit valoir pour lui, C36) ; une facture brouillon à
       une ligne avec TVA, créée par `POST /api/v1/invoices`, à une date couverte par un exercice ouvert ;
       la créance désignée puis rendue non imputable (`UPDATE accounts SET postable = FALSE`) →
       400 `ACCOUNT_NOT_POSTABLE`, `details.rejected`, `message` contenant « Paramètres → Facturation »,
       le numéro, et « administrateur » ; facture toujours brouillon ;
     - **complétion d'une facture importée** — `crates/kesh-api/tests/inbox_import_e2e.rs`
       (existant), sur le montage de `complete_creates_invoice_and_marks_completed` (`:716` :
       `setup`, `seed_staging`, `complete_body`) : le compte créanciers des réglages rendu non
       imputable, `POST /api/v1/imported-supplier-invoices/{id}/complete` → 400
       `ACCOUNT_NOT_POSTABLE` ; le staging **reste** `to_complete` (`staging_status`) et
       `supplier_invoice_count` est inchangé (patron de `complete_closed_fiscal_year_keeps_to_complete`,
       `:792`) ;
   - **mutation** : retirer chacun des quatre contrôles une fois → son test rougit ; poser le rôle
     `VatPayable` **hors** de la branche `total_vat > 0` du générateur → le test « TVA arrondie à zéro »
     rougit ; contourner `NonPostableAccounts::new` (liste construite sans dédoublonnage) → le test
     « même compte pour deux rôles » rougit ; consigné au Dev Agent Record, en touchant le fichier après
     restauration ;
   - **frontend** (`frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`,
     existant) : le compte créanciers en place s'affiche, **y compris devenu non imputable** (valeur
     préservée) ; un changement est envoyé dans `updateInvoiceSettings` ; la relecture sur conflit le
     reprend. C'est la preuve du **chemin de correction** que le message de l'AC2 désigne ;
   - **frontend, écran de validation** (AC4, choix C41) — fichier neuf
     `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts`, patron de
     `invoice-write-off-page.test.ts` (mocks hoistés avant l'import du composant) : la validation
     rejetée par un `ApiError` 400 `ACCOUNT_NOT_POSTABLE` affiche **`err.message` tel quel**, pour un
     utilisateur **Comptable** comme pour un **Admin** — ni le suffixe « Configurez les comptes par
     défaut », ni « Demandez à votre administrateur » de la branche `CONFIGURATION_REQUIRED`
     (`:409-415`). Mutation attrapée : `ACCOUNT_NOT_POSTABLE` ajouté à cette branche. Les deux `catch`
     fournisseurs (`supplier-invoices/+page.svelte:213`, `import/+page.svelte:267`) n'ont pas de branche
     par code sur ce chemin : leur lecture est consignée au Dev Agent Record (AC4), sans test d'écran.
8. **AC8 — Le manuel dit l'usage, l'avoir, et le champ créanciers.** L'AC17 de la 15-5b a levé les
   réserves des encadrés pour la **désignation** ; cette story les complète pour l'**usage** :
   - `docs/manual/fr/user-manual.tex`, encadrés **`:380`** (§ *Rôles des comptes*) et **`:390`**
     (§ *Les comptes de clôture*) : l'item que la 15-5b y a écrit — un compte de réglage devenu non
     imputable **après** sa désignation reste utilisé — est **réécrit** : la validation d'une facture et
     la saisie d'une facture fournisseur le refusent désormais, avec un message qui renvoie à
     *Paramètres* → *Facturation* ; l'**avoir** fait exception (il relit la créance et la TVA due dans
     les réglages au moment de l'avoir, sans les contrôler) ; le **compte de produit par défaut** aussi
     (D3-bis). Aucune phrase ne dit que l'avoir « reprend les comptes de la facture d'origine » pour
     la créance et la TVA due : c'est **faux** (AC3) ;
   - § validation d'une facture (`user-manual.tex:847-849`, qui énumère ce qui empêche la validation :
     compte d'arrondi, montant minimum) : une phrase pour le refus de l'AC1 (finding F4-7) ;
   - § *Saisir une facture fournisseur* (`:1285-1293`) : le compte de dette fournisseur est le **compte
     créanciers** désigné dans *Paramètres* → *Facturation* ; s'il n'est pas imputable (ou la TVA
     récupérable, pour une facture qui en porte), la saisie est refusée avec un message qui y renvoie ;
   - § *Importer des factures depuis un dossier* (`user-manual.tex:1374-1397`, bouton *Compléter*, « Kesh
     vérifie que le total TTC correspond […] avant de créer la facture fournisseur définitive ») : une
     phrase dit que la complétion est refusée de la même façon — la complétion crée une facture
     fournisseur par le même chemin que la saisie (`imported_supplier_invoices.rs:243`) — et renvoie au
     paragraphe de la saisie (finding F5 de la P1) ;
   - `docs/manual/fr/admin-manual.tex`, § *Configuration des comptes TVA* (`:2016-2025`) : la phrase
     que la 15-5b y ajoute (refus à la désignation) est complétée — un compte TVA désigné **devenu**
     non imputable bloque la validation d'une facture portant de la TVA (ou la saisie d'une facture
     fournisseur) avec un message qui renvoie aux paramètres ; et un paragraphe voisin nomme le **compte
     créanciers** (`2000` dans les plans livrés), désormais réglable sur la même page, sur le même
     patron « actif et imputable » ;
   - le passage de l'avoir (`user-manual.tex:1155`, « l'inverse exact de l'écriture de la facture (la
     créance client, le produit et la TVA due …) ») n'est **pas** réécrit ici : il relève de #473 et
     #525 ; le dev le **signale** au Dev Agent Record sans le corriger ;
   - les PDF (`user-manual.pdf`, `admin-manual.pdf`) sont régénérés (`latexmk -xelatex` dans
     `docs/manual/fr/`), commités, et **contrôlés aplatis en normalisant les ligatures** :
     `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'`
     (le corps du PDF rend « ff » par la ligature `ﬀ` : un `grep` de « affiche » ou « différences » y
     rend un faux négatif — finding F-7 de la P2 de la 15-5c) ; les phrases nouvelles présentes, les
     phrases levées absentes.
9. **AC9 — Doc-comments, `docs/api-external.md`, CHANGELOG.**
   - Doc-comments de l'accesseur, de `validate_invoice` (« # Erreurs », ordre de l'AC1) et de
     `supplier_invoices::create_in_tx` ; la limite **L2** de D-A0 est citée comme **révisée** pour ces
     quatre comptes (`14-3b-consommateurs-roles.md:188`).
   - `docs/api-external.md`, table du § 10 : la ligne `ACCOUNT_NOT_POSTABLE` (posée par la 15-5a,
     étendue par la 15-5b) gagne la **validation d'une facture** et la **saisie d'une facture
     fournisseur** quand un compte désigné dans les réglages n'est pas imputable — même `details`.
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (créée par la 15-5a ; **la créer en tête si
     absente** — motif exact exigé par `scripts/prepare-release.sh:189`) :
     - rubrique **Corrigé** : la validation d'une facture et la saisie d'une facture fournisseur
       refusent un compte de réglage (créance, TVA due, créanciers, TVA récupérable) devenu non
       imputable (#429) ;
     - rubrique **Ajouté** : le compte créanciers se règle dans *Paramètres → Facturation* ;
     - **une ligne d'action** (finding F4-8 — précédent 16-1a-bis, où un « aucune action de votre
       part » a été pris en défaut) : « Après la mise à jour, vérifiez dans *Paramètres → Facturation*
       que les comptes désignés sont imputables : un compte qui a reçu des sous-comptes doit y être
       remplacé, faute de quoi la validation des factures (ou la saisie des factures fournisseurs) est
       refusée. »

## Tasks / Subtasks

- [ ] **T0 — Refaire les relevés** sur `HEAD`, **après le merge de la 15-5b** : numéros de ligne de
      `invoices.rs` (génération, `create_in_tx`, `:1849-1852`), `supplier_invoices.rs`, `credit_notes.rs`,
      des écrans et des manuels ; refaire le grep des lecteurs des quatre comptes
      (`grep -rnE "default_(receivable|payable|vat_payable|vat_recoverable)_account_id" crates/*/src`, hors
      tests et module des réglages) — **un lecteur qui écrit, absent de l'AC1 et de l'AC3, bloque la
      story** ; refaire le grep des appelants des générateurs (AC1, 25 occurrences sur `67c31c95`) ;
      **relever le texte exact que la 15-5b aura laissé** dans les encadrés `user-manual.tex:380` et
      `:390` et à `admin-manual.tex:2016-2025` (l'item que l'AC8 réécrit n'existe pas sur `67c31c95`),
      le copier au Dev Agent Record et, s'il ne contient aucun des motifs du grep de T6, **ajouter à ce
      grep** le motif qui le retrouve (finding R1-8 de la P1) ; relever le montage que la 15-5b aura
      écrit dans `company_invoice_settings_postable_e2e.rs` (AC7).
- [ ] **T1 — La garde** (AC1, AC3) : les rôles rendus par les deux générateurs (`DesignatedRole`,
      branches de `total_vat > 0`), leurs appelants adaptés ; l'accesseur de `company_invoice_settings`
      (`ORDER BY id FOR UPDATE`, variante construite par `NonPostableAccounts::new`) ; ses deux appels,
      à la place fixée ; la variante `DesignatedAccountsNotPostable` (`error_code()` → `"ACCOUNT_NOT_POSTABLE"`,
      `match` exhaustif de `kesh-db/src/errors.rs`) et son bras dans `crates/kesh-api/src/errors.rs` (400,
      `t_args`, `details()`) ; doc-comments.
- [ ] **T2 — Le message** (AC2) : la clé dans les quatre `messages.ftl`, inscrite à
      `SELECTEURS_RESOLUS_COTE_SERVEUR` ; tests Rust par locale, singulier et pluriel.
- [ ] **T3 — Les écrans** (AC4, AC5) : lecture des trois `catch` de l'AC4, consignée ; le `<select>`
      du compte créanciers, ses types, le mock de `settings-invoicing-page.test.ts`, sa clé i18n
      (quatre locales) ; borne `sitesTotal` de
      `frontend/src/lib/shared/i18n-keys.test.ts` relevée **délibérément**, ventilation recomptée
      (`sitesTotal`, `sitesNonResolus`, `relais`, `sitesGabarit`, `litterauxMin`, `clesDepuisTsMin`) et
      écrite au Dev Agent Record.
- [ ] **T4 — Les E2E** (AC6) : contournements retirés, jugés sur la **suite E2E complète** de T7 ;
      retrait confirmé ou contournement rétabli avec un commentaire.
- [ ] **T5 — Les tests** (AC7) : `kesh-db` (dont « même compte pour deux rôles » et « deux comptes
      distincts »), `kesh-api` (`company_invoice_settings_postable_e2e.rs`, `inbox_import_e2e.rs`),
      Vitest (réglages, écran de validation) ; mutations consignées.
- [ ] **T6 — Manuel, API, CHANGELOG** (AC8, AC9) ; PDF régénérés et contrôlés aplatis (ligatures
      normalisées) ; **grep du symptôme** (règle *Propagation post-patch*) :
      `grep -rnE "reprend les comptes de la facture|sous-compte imputable|inverse exact|n'est pas exposé|#429" docs/manual/fr/*.tex docs/api-external.md crates frontend/src CHANGELOG.md`
      — chaque occurrence est réécrite ou justifiée au Dev Agent Record.
- [ ] **T7 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) — **même
      en cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate frontend
      complet ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par
      fichier contre `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 9 AC, 8 tâches T0–T7.)*

## Dev Notes

### La garde à l'usage et la doctrine D-A0 (vérifié en validation P3 de la 15-5b)

D-A0 (14-3b) exempte les flux automatiques de la garde de `create_in_tx` (`enforce_postable = false`),
et sa limite **L2** dit en toutes lettres : « un compte de config (créance/produit/dette) devenu
non-postable après configuration reste posté par les flux automatiques […] Remédiation éventuelle :
re-vérifier `postable` à la résolution avec message dédié — amélioration future si un besoin se
manifeste » (`14-3b-consommateurs-roles.md:188`). L'AC1 **est** cette remédiation, pour quatre
comptes : elle ne touche pas `create_in_tx` ni son drapeau (D-A0 tenu), elle **révise L2** pour la
créance, la TVA due, les créanciers et la TVA récupérable — le besoin s'est manifesté (#429). Elle ne
contredit pas l'angle mort « compte bancaire **à l'usage** » (C6) : le compte bancaire n'est pas un
réglage de facturation, il sert à **tous** les rapprochements et règlements, et le bloquer à l'usage
changerait D-A0 elle-même — ce qui reste hors de cette story.

**Les plans livrés** (finding R4-4, prémisse corrigée) : `insert_with_defaults` ne désigne d'office que
les rôles `Receivable`, `DefaultRevenue` et `Payable` (`company_invoice_settings.rs` ~`:562-590`) — soit
**`1100` et `2000`** pour la créance et les créanciers ; **`2200` et `1171`** existent dans les trois
plans et sont **choisis par l'utilisateur** (`admin-manual.tex:2025` : comptes « ajoutés aux plans »,
non désignés). Les quatre sont des **feuilles** dans les trois plans
(`crates/kesh-core/assets/charts/*.json`, `parentNumber`, vérifié par script en P4) : aucune société
fraîchement créée n'est bloquée.

**Risque d'interblocage** : l'accesseur verrouille des lignes `accounts` `FOR UPDATE`, comme
`rounding_account_for_write` dans le même flux de validation (`invoices.rs:2065`) et comme la garde du
compte de charge (24-5) dans le flux fournisseur — `get_or_create_default_in_tx` verrouille déjà la
ligne des réglages. Le dev place la lecture **après** ces verrous existants (c'est le cas à la place
fixée par l'AC1) et le dit dans le doc-comment ; **entre les comptes d'une même requête**, l'ordre est
celui des identifiants (`ORDER BY id`, une seule requête, AC1 — finding R1-6 de la P1) ; `accounts::create` (qui rend le parent non imputable)
ne prend aucun verrou sur les réglages ni sur les factures (vérifié en P4, lentille F).

### Pourquoi le compte créanciers passe à l'écran ici, et pas dans la 15-5b

C25 l'avait écarté « sans nécessité pour fermer le défaut » (#521). C27 a créé cette nécessité : un refus
qui renvoie à un champ invisible est une **impasse**, pire que le défaut silencieux qu'il remplace —
avant la garde, la saisie postait (à tort) sur le compte de regroupement ; après, elle serait bloquée
sans recours à l'écran (findings R4-1/F4-1, HIGH). La garde et l'écran vont donc **ensemble**, dans la
même story (choix C34). La 15-5b garde l'AC19 (#521), qui reste nécessaire pour les clients qui
n'envoient pas le champ.

### Ce qui doit être préservé

- Le contrat d'erreur de la 15-5a : `ACCOUNT_NOT_POSTABLE` n'est émis que pour un compte **de la
  société, actif** ; `details` par `NonPostableAccounts::details()` seul (C29).
- L'exemption « inchangé » de l'AC11 de la 15-5b, à la désignation : ce n'est **pas** une tolérance à
  l'usage, et la garde de l'AC1 ne la contourne pas — un réglage inchangé reste enregistrable ; il est
  refusé quand un flux veut y écrire.

### Hors périmètre, et écrit

- **L'avoir** (C35) : créance → 15-6a (#473, #523) ; TVA due → #525. Le passage du manuel `:1155`
  (« l'inverse exact ») relève des mêmes issues.
- Le **compte bancaire à l'usage** (D-A0, C6) et le **compte de produit par défaut** (D3-bis).
- L'alignement du solde du reste (`ConfigurationRequired`) sur ce refus (C28).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C27** — garde **à l'usage** des comptes de réglage (créance, TVA due, créanciers, TVA
  récupérable) ; l'exemption « inchangé » ne vaut qu'à la désignation. Compatible avec D-A0.
- **C28** — forme du refus : variante `DesignatedAccountsNotPostable`, même code et même `details`,
  message propre qui dit où agir (texte révisé par **C36**).
- **C29** — `NonPostableAccounts::details()` (15-5a), seul constructeur du JSON `rejected`.
- **C33** — le découpage : la garde à l'usage sort de la 15-5b.
- **C34** — le compte créanciers exposé à l'écran (révise **C25**).
- **C35** — l'avoir exempté de la garde à l'usage, avec sa vraie raison.
- **C36** — le message : « un compte imputable », « un administrateur doit ».
- **C39** — le prédicat : les générateurs rendent les rôles de réglage effectivement écrits ; la garde
  ne contrôle que ceux-là.
- **C40** — le dédoublonnage : confié à `NonPostableAccounts::new` (15-5a), figé par un test.
- **C41** — l'AC4 prouvé par un test Vitest de l'écran de validation ; les deux `catch` fournisseurs
  par lecture consignée.

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/{company_invoice_settings,invoices,supplier_invoices}.rs`,
`crates/kesh-db/src/errors.rs` (variante), `crates/kesh-api/src/errors.rs` (bras),
`crates/kesh-i18n/locales/*/messages.ftl` (deux clés : `error-designated-account-not-postable`,
`settings-invoicing-payable-account`), `crates/kesh-i18n/src/loader.rs` (sélecteur),
`frontend/src/routes/(app)/settings/invoicing/{+page.svelte,settings-invoicing-page.test.ts}`,
`frontend/src/lib/features/invoices/invoices.types.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`
(borne), éventuellement `frontend/tests/e2e/{payment-batches,inbox-import}.spec.ts` (AC6) et les
écrans de facture (AC4, si un `catch` est en défaut), tests
(`crates/kesh-db/tests/{invoices_validate_vat,supplier_invoices_repository}.rs`, et `crates/kesh-db/tests/credit_notes_repository.rs`
pour le test de C35 ; `crates/kesh-api/tests/{company_invoice_settings_postable_e2e,inbox_import_e2e}.rs` ;
`frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts`, neuf),
`docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}`, `docs/api-external.md`, `CHANGELOG.md`.
**Aucune migration** (P1–P8 sans objet).

**Décompte des modules** (règle de splitting préventif ; recompté en P1, finding F6) : `kesh-db` (trois
dépôts, erreurs), `kesh-api` (erreurs, deux fichiers de tests), `kesh-i18n` (quatre locales, chargeur),
`frontend` (réglages, types, test de l'écran de validation, borne i18n), manuels (deux) — **cinq** en
comptant les specs Playwright avec le `frontend` et `docs/api-external.md` / `CHANGELOG.md` comme
compagnons de documentation ; **six ou sept** si l'on compte à part `frontend/tests/e2e` et les
documents de `docs/`. Le seuil (« plus de 5 modules ») est donc atteint ou franchi selon la convention
de compte. Le signal est **déclaré** au Change Log (amendement D5) ; il ne déclenche pas de découpage :
la story porte une **seule règle métier** (la garde à l'usage) et l'écran qui la rend praticable,
inséparables (C34), et elle est revue en **passes complètes**.

### Tests — ce qui rendrait un test vert sans rien prouver

- Un compte de test **désigné après** avoir été rendu non imputable : la désignation (AC10 de la 15-5b)
  le refuserait, et le test prouverait la garde de la 15-5b, pas celle-ci. Désigner, **puis** basculer
  `postable`.
- Asserter `400` sans le code ni `details.rejected` : un refus pour une autre raison passerait.
- Le test « TVA arrondie à zéro » doit utiliser un taux **positif** : un taux nul ne distingue pas les
  deux prédicats.
- Le test « même compte pour deux rôles » doit asserter `count = 1` **et** un seul élément dans
  `details.rejected` : un `400` seul passerait aussi avec un doublon. Et le test « deux comptes » doit
  utiliser deux comptes **distincts** : sur un compte commun, il prouverait le dédoublonnage, pas la
  réunion des deux rôles dans un seul refus.
- Le test Vitest du compte créanciers doit porter sur un compte **non imputable** : avec un compte
  imputable, l'affichage marcherait sans `withCurrentAccount`.

### References

- Issues : #429 (et son commentaire) ; #473, #523, #525 (avoir) ; #521 (15-5b).
- Fiches : `15-5-gardes-postabilite-serveur.md` (mère, `split`), `15-5a-refus-non-imputable.md`,
  `15-5b-gardes-surfaces-neuves.md` ; `14-3b-consommateurs-roles.md` (D-A0, L2).
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit NOMMER le
  manuel*, § *Test Locally First* (exception `kesh-db`), § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — **Créée à la passe de validation P4 de la 15-5b** (choix C33), sur le finding **F4-3**
  (MEDIUM, lentille F : l'ancien AC20 de la 15-5b est une règle métier neuve, pas un rollout, et les
  deux HIGH de la passe en sont nés). Historique de ce contenu avant sa sortie :
  - **Validation P3 de la 15-5b** (lentilles Sonnet R et F) : finding F-1 (MEDIUM) — un réglage
    « inchangé » devenu non imputable restait utilisé sans contrôle à l'usage, le défaut de #429
    subsistait → AC20 ajouté à la 15-5b, choix **C27** (garde à l'usage) et **C28** (forme du refus) ;
    révision de la limite L2 de D-A0 pour quatre comptes.
  - **Validation P4 de la 15-5b** (prompt versionné `15-5b-validate-prompt-p4.md` ; lentilles **Opus**
    R et F, contexte frais) — findings qui portaient sur ce contenu, et leur sort ici :

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R4-1 = F4-1 | HIGH | le refus des **créanciers** renvoie à un champ que l'écran n'a pas : impasse sans recours | **AC5** : le champ exposé à l'écran (C34, révise C25) ; AC6 (E2E) ; AC7 (test Vitest du chemin de correction) | **né de la remédiation P3** (C27 contre C25) |
  | R4-2 = F4-2 | MEDIUM | l'avoir écrit sur la créance et la TVA due **des réglages du moment**, sans garde ; la fiche le disait « snapshot de la facture » | **AC3** : exemption délibérée, avec la vraie raison (créance → 15-6a, #473, #523 ; TVA due → #525) ; test qui la fige (AC7) ; AC8 (manuel) (C35) | **d'origine** (l'inventaire #11 date de la création de la 15-5) ; texte de l'AC17 (iv) né de la remédiation P3 |
  | F4-3 | MEDIUM | l'AC20 n'est pas du rollout ; la 15-5b n'a plus de justification pour son ampleur | **cette story** (C33) | **né de la remédiation P3** |
  | R4-3 = F4-4 | LOW | « porte de la TVA » inconnaissable avant la génération ; prédicat à fixer | AC1 : `total_vat > 0`, garde entre génération et `create_in_tx` ; test « TVA arrondie à zéro » | — |
  | F4-5 | LOW | place de la garde dans l'ordre des refus non fixée | AC1 : ordre écrit au doc-comment ; « tous les comptes **de réglage** » ; tests d'ordre (AC7) | — |
  | R4-4 | LOW | « les plans livrés désignent d'office 1100, 2200, 2000 et 1171 » faux pour 2200 et 1171 | Dev Notes corrigées | — |
  | R4-5 = F4-6 | LOW | « désignez-y un sous-compte » ; message servi à un Comptable sans accès aux réglages | AC2 : « un compte imputable », « un administrateur doit » (C36) | — |
  | F4-7 | LOW | sections du manuel où la validation et la saisie fournisseur listent leurs refus non visées | AC8 : `:847-849` et `:1285-1293` | — |
  | F4-8 | LOW | CHANGELOG : changement qui peut bloquer après mise à jour, pas seulement « Corrigé » | AC9 : ligne d'action | — |

  Les autres findings de la P4 (R4-6, R4-7, F4-9) portent sur ce qui reste dans la 15-5b : voir son
  Change Log. **Décisions de l'orchestrateur** : C33 à C35 ; **C36** ajouté pendant la remédiation.
  **Propagation post-patch** : `AC20`, `DesignatedAccountsNotPostable`, `designated-account`, `C27`,
  `C28`, `sous-compte imputable`, `reprend les comptes`, `n'est pas exposé`, `closes #429` grepés sur les
  fiches 15-5, 15-5a (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml` ; les
  occurrences restantes sont historiques (Change Logs), des renvois à la 15-5d, ou — pour « sous-compte
  imputable » — l'énoncé de la story 15-5a (`15-5a-refus-non-imputable.md:27`), qui porte sur la saisie
  manuelle et que cette passe ne touche pas (fiche en développement sur une autre branche, signalé à
  l'orchestrateur).
  Décompte : **9 AC, 8 tâches T0–T7** (recompté). **Validation : à lancer** — une passe complète, la
  story portant une règle métier et un écran neuf.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-5d-validate-prompt-p1.md` ; deux
  lentilles **Sonnet** en contexte frais : **R** auditeur d'acceptation, **F** adversaire de périmètre
  complet ; rotation D6 : la P2 sera Opus). **0 CRITICAL, 0 HIGH.** Bruts : R 3 MEDIUM + 5 LOW
  (R1-1 à R1-8), F 1 MEDIUM + 5 LOW (F1 à F6). Doublons inter-lentilles : F2 = R1-2, F3 ≈ R1-1 (F3
  ajoute la phrase du rapport TVA de l'AC3, traitée avec), F4 ⊂ R1-3 → **4 MEDIUM et 7 LOW
  distincts** (recompté : 14 bruts − 3 fusions = 11). Trend : **P1 4 MEDIUM / 7 LOW** (première passe
  de cette fiche ; la P4 de la 15-5b, d'où elle est née, est à l'entrée précédente).

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | R1-1 ≈ F3 | MEDIUM | AC4 : `catch` `:103` (scan QR) et `:133` (chargement) hors sujet ; écran de complétion d'import sans chemin ; AC3 : le rapport TVA ne lit pas le compte de décompte | AC4 : trois `catch` nommés (`invoices/[id]/+page.svelte:402-415`, `supplier-invoices/+page.svelte:213`, `supplier-invoices/import/+page.svelte:267`, `default:` `:301`), `:103` et `:133` écartés ; AC3 : `vat_report.rs:173-174` lit TVA due et récupérable |
  | R1-2 = F2 | MEDIUM | AC1 : « identifiants effectivement présents dans `entry_lines` » sans mécanisme — `total_vat` local au générateur, ambigu quand un compte joue deux rôles | AC1 : les générateurs (`invoices.rs:1784`/`:1851`, `supplier_invoices.rs:105`/`:129`) **rendent les rôles** effectivement écrits ; la garde ne contrôle que ces comptes ; appelants (25 occurrences) adaptés, l'avoir ignore les rôles (**C39**) |
  | R1-3 ⊃ F4 | MEDIUM | AC7 : complétion d'import sans fichier ni montage ; cas e2e sans route ni montage ; AC4 sans test | AC7 : `inbox_import_e2e.rs` (montage `:716`, patron `:792`) ; `company_invoice_settings_postable_e2e.rs` (montage de la 15-5b, `POST /api/v1/invoices/{id}/validate`, Comptable) ; Vitest `invoice-validate-page.test.ts` neuf (**C41**) |
  | F1 | MEDIUM | un même compte pour deux rôles → « Les comptes 2000, 2000 » ? | **C40** : dédoublonnage par `NonPostableAccounts::new` (15-5a, AC1 : « trie […] et **dédoublonne par identifiant** ») ; test « même compte pour deux rôles → nommé une fois, singulier » ; deux comptes **distincts** (`1100`, `2000`) pour le test « créance et TVA due » ; mutation |
  | R1-4 | LOW | AC6 : retrait jugé sur deux specs isolées | AC6, T4 : jugé sur la suite E2E complète |
  | R1-5 | LOW | mock `settings-invoicing-page.test.ts:67` sans `defaultPayableAccountId` | AC5, T3 |
  | R1-6 | LOW | ordre de verrouillage entre comptes non fixé | AC1 : une requête, `ORDER BY id FOR UPDATE` ; Dev Notes |
  | R1-7 | LOW | statut `ready-for-dev` contre « Validation : à lancer » ; lignes relevées avant la 15-5b | en-tête : convention du registre écrite ; AC5 : lignes relevées sur `67c31c95`, T0 les refait |
  | R1-8 | LOW | l'item du manuel que l'AC8 réécrit n'existe pas encore | T0 : relevé du texte laissé par la 15-5b, grep de T6 complété au besoin |
  | F5 | LOW | § *Importer des factures depuis un dossier* (`user-manual.tex:1374-1397`) non visé | AC8 : une phrase, renvoi à la saisie |
  | F6 | LOW | décompte des modules imprécis | Dev Notes : décompte écrit, signal déclaré (ci-dessous) |

  **Signal de la règle de découpage** : sévérité sans objet (première passe). **Décompte des modules** :
  cinq selon la convention de la fiche, six ou sept en comptant à part `frontend/tests/e2e` et les
  documents de `docs/` — le seuil est atteint ou franchi. **Constaté et déclaré** (amendement D5) ; pas
  de découpage proposé : une seule règle métier, et l'écran qui la rend praticable (C34) ne s'en
  sépare pas. Déclaré au Project Lead par l'orchestrateur, à qui revient l'arbitrage.
  **Décisions de l'orchestrateur** : mécanisme des rôles (C39), dédoublonnage par la 15-5a (C40), test
  d'écran de l'AC4 (C41).
  **Propagation post-patch** : `:103`, `:133`, `effectivement présents`, `vat_report.rs:174`, `le
  rapport TVA le lit`, `deux specs`, `sans le contournement`, `total_vat > 0` grepés sur les fiches
  15-5a (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml` — les occurrences
  restantes sont historiques (Change Logs de la 15-5b et de cette fiche) ou exactes (le prédicat
  `total_vat > 0`, désormais porté par la branche du générateur).
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P2 suit** (des MEDIUM en P1),
  complète (Opus) : la remédiation change le mécanisme de la garde.
