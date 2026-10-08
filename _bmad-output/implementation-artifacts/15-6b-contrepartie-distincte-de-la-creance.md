# Story 15.6b : Un règlement ne vise pas le compte qu'il solde

## Status

ready-for-dev

<!-- Spécifiée le 2026-10-08 en autonomie (bmad-create-story), fille de la 15-6 découpée d'emblée
     (choix C-15-6-1). Choix propres : C-15-6-3, C-15-6-4 (précisé par C-15-6-14), C-15-6-10,
     C-15-6-11. Validation P1 (Sonnet, lentilles F et R) appliquée le 2026-10-08 : #524 sorti vers la
     15-6d (C-15-6-9), lots de paiement ajoutés (C-15-6-10), #492 rattaché à la 15-5c (C-15-6-12).
     Validation P2 (Opus, lentilles F et R) appliquée le 2026-10-08 : comptes d'écart comparés à la
     créance (C-15-6-18), issue d'un lot bloqué (C-15-6-19), écrans sans archivés (C-15-6-17),
     décompte des modules (C-15-6-20). Validation P3 (Sonnet, lentilles R et F ; remédiation Opus 5.5) appliquée le
     2026-10-08 : refus distinct pour un compte venu des réglages (`role`), tables de refus du
     guide, helper commun, sources des numéros (C-15-6-26). Validation P4 (Opus, lentilles R et F)
     appliquée le 2026-10-08 : rôle `rounding` du compte d'arrondi quel que soit le geste, angles morts
     de la validation et de l'avoir écrits (#537, #525), message de liste vide conditionné (C-15-6-30).
     Validation P5 (Sonnet, lentilles R et F) appliquée le 2026-10-08 : alignement sur la 15-5e
     réécrite par C54 (rejeu, pas de réordonnancement), trois clés plates, critère de T2 par nom et
     liste fermée (C-15-6-33). Validation P6 (Opus, lentilles R et F) appliquée le 2026-10-08 :
     0 au-dessus de LOW, 18 LOW appliqués — **validation close** (C-15-6-34). -->

**Issue** : `refs #474` (P1) — la fermeture vient avec la 15-6c, qui traite la configuration.
**Mère** : `15-6-creance-juste-avoir-reglement.md`. **Dépend de** : 15-6a (lecteur
`invoice_settlements::sale_receivable_account`, mêmes fichiers). **De préférence après** la 15-5c
(le code neuf apparaît dans `failed[]`, dont la **15-5c** pose les libellés traduits — #492,
`frontend/src/lib/features/reconciliation/failed-proposal-label.ts`).
**Après** les 15-5a à 15-5e (ordre de l'epic ; la **15-5e** passe avant la 15-5d), qui touchent les
mêmes fonctions : `settle_invoice` et `pay_in_tx` (15-5a AC4 : refus `ACCOUNT_NOT_POSTABLE` à
l'étape du compte interne), leurs **routes** et celle du règlement fournisseur et de la confirmation d'un lot (**15-5e** AC3 :
rejouées sur interblocage — aucun ordre de verrou ni de refus n'y change, C55), le commentaire « 5 bis »
de `write_off_invoice` (15-5e AC5, réécrit en place), `accept_one_invoice` (15-5b AC6 : commentaire de
classement), le manuel du rapprochement (15-5c) — **T0** rebase et refait les numéros de ligne (ceux de
cette fiche sont du commit `1920381e`). Cette fiche cite les étapes **par leur nom** (lecture de la
créance, compte interne, compte d'arrondi, trop-perçu…), les numéros d'étape pouvant changer aux
rebases.
**Voisin sorti** : la contrepartie égale au **compte de banque** (#524) est la **15-6d** (choix
C-15-6-9) — hors de cette fiche, **écran du rapprochement manuel compris** (`ManualMatchModal`).

## Story

En tant que **comptable**,
je veux que **Kesh refuse un règlement dont la contrepartie est le compte même qu'il doit solder**
(le compte débiteurs de la facture client, le compte créanciers de la facture fournisseur),
afin que **le reste dû ne baisse jamais sans mouvement au grand livre**.

## Le défaut, établi au code

- **Règlement client** (`crates/kesh-db/src/repositories/invoice_settlements_write.rs`,
  `settle_invoice`) : la créance est lue sur l'écriture de vente (`:100-115`), la contrepartie résolue
  selon le mode (`:118-170`) — virement : `bank_accounts.journal_account_id` ; compte interne : tout
  compte `active && postable` (`:159-167`). **Aucune comparaison** entre les deux. Choisir 1100 produit
  `D 1100 / C 1100` : l'écriture s'équilibre (`kesh-core/src/accounting/balance.rs` n'interdit pas un
  même compte au débit et au crédit), le règlement s'enregistre, le reste dû baisse, la facture peut
  passer « payée » — et le grand livre ne bouge pas.
- **Le virement aussi** : un compte bancaire lié au 1100 (la route l'accepte, `bank_accounts.rs:288-293`
  — type actif ou passif) donne la même écriture nulle.
- **Règlement fournisseur** (`crates/kesh-db/src/repositories/supplier_invoices.rs`, `pay_in_tx`) :
  symétrique — dette lue sur l'écriture d'achat (`:580-593`), contrepartie par virement (`:598-621`)
  ou compte interne (`:622-656`, `Some((true, true))` `:648`) ; rien n'interdit le 2000 :
  `D 2000 / C 2000`, facture « payée » sans paiement.
- **Lots de paiement** (`crates/kesh-db/src/repositories/payment_batches.rs`) : `confirm_batch`
  (`:304`) appelle `pay_in_tx` en `BankTransfer` pour chaque facture du lot (appel `:360`), dans une
  seule transaction (ouverte `:311`) — troisième appelant de `pay_in_tx`. `create_batch` (`:68`) ne contrôle du
  compte bancaire source que « existe, non archivé, journal lié non nul » (`:84-103`) : un compte
  bancaire lié au 2000 produit un lot **et son fichier pain.001**, que la seule garde de `pay_in_tx`
  rendrait ensuite **inconfirmable** — alors que le fichier est peut-être déjà déposé à la banque.
- **Encaissement par rapprochement** (`crates/kesh-api/src/routes/reconciliation.rs`,
  `accept_one_invoice` `:1175`) : compte de banque (`:1384-1412`) et créance (`:1425-1452`) lus, jamais
  comparés.
- **Écrans** : `SettleInvoiceDialog.svelte:105` offre tout compte `active && postable`, 1100 compris ;
  la fiche fournisseur (`supplier-invoices/[id]/+page.svelte:83-84`) offre de même le 2000, et les
  deux offrent tout compte bancaire lié.
- **Comptes d'écart du même geste** (finding F3 de la P2) : le compte d'arrondi du règlement
  (`settle_invoice`, compte d'arrondi du règlement, `invoice_settlements_write.rs:189` ; `accept_one_invoice` (c-bis),
  `reconciliation.rs:1487`) et les comptes du solde du reste (`write_off_invoice` : nature `:433`,
  reste d'arrondi, TVA due `:489`) sont lus dans les **réglages** — l'utilisateur ne les choisit pas
  au moment du geste. Les comptes d'arrondi et des natures sont revérifiés **charge ou produit** au
  moment d'écrire (`company_invoice_settings.rs:400-447`, `:484-507`, `usable_designated_account`).
  ⚠️ Le compte de **TVA due**, lui, ne l'est **par aucun type** : `vat_payable_account_for_write`
  (`company_invoice_settings.rs:454-477`) n'exige que « de la société, actif, imputable »
  (`FOR UPDATE`) — le type n'est contrôlé qu'à la **désignation** (route des réglages). Pour lui, la
  comparaison à la créance (AC3 bis) n'est donc pas un filet du changement de type : c'est la
  **seule** garde au moment d'écrire (finding F4 de la P3).
  Le contrôle de **type** ne suffit pas non plus pour les autres : le type d'un compte qui porte des écritures **se change**
  avec confirmation (`accounts.rs:497-512`, `confirm_retype`), le rôle `Receivable` se retirant dans le
  même geste. Le 1100 devenu `Expense` porte toujours la créance des ventes antérieures (lue sur la
  vente, 15-6a) et passe le contrôle de type : `write_off_invoice` écrirait alors `D 1100 / C 1100`.
  Ces comptes sont donc **comparés par identifiant** à la créance de la vente (AC3 bis, choix
  C-15-6-18, qui révise le « résolu par le type » de C-15-6-3).

**Ce que la story ne résout pas — angles morts écrits** (finding F2 de la P4 ; le « résolu par le
type » des rédactions précédentes était faux : l'argument du retypage confirmé, qui a motivé l'AC3 bis,
le défait aussi, et la créance lue dans les réglages n'a **aucun type contrôlé** à la validation,
`invoices.rs:2008-2010` — présence seulement, puis `active` par `create_in_tx`) :

- **Validation d'une facture — créance retypée** : le compte débiteurs désigné, retypé avec
  confirmation (`accounts.rs:497-512` ; le retypage ne touche pas les réglages) en `Revenue`, reste
  `default_receivable_account_id` ; une ligne de facture qui l'impute en produit donne à la validation
  `D 1100 / C 1100` (HT) ; de même le compte d'arrondi de la validation, s'il est la créance retypée en
  charge. Tracé par **#537** (ouverte par l'orchestrateur). Hors de cette story : la validation
  **crée** la créance, elle n'en solde aucune (classe de la story : « un règlement ne vise pas le
  compte qu'il solde »), et la garder ici ajouterait un sixième module (`invoices.rs`).
- **Avoir — TVA due et produit de repli** : l'avoir écrit la TVA due et le produit de repli
  (`default_revenue_account_id`, lignes sans compte, D-B2) **lus dans les réglages courants**
  (`credit_notes.rs:205`, `:511`) ; un 1100 retypé `Liability` puis désigné TVA due donne
  `D 1100 (TVA) / C 1100`. La **TVA due** est tracée par **#525** (TVA due de l'avoir lue dans les
  réglages du moment — angle mort de la 15-6a), dont c'est un cas ; le **produit de repli** est
  tracé lui aussi par **#525**, depuis le commentaire de l'orchestrateur du 2026-10-08 (« Même
  famille … compte de produit de repli … À traiter avec la TVA due de cette issue » — posté juste
  après la P5, qui l'avait écrit « non tracé » ; finding R6-2 de la P6). La créance de l'avoir est relue sur la
  vente (15-6a), ses autres comptes ne le sont pas. Hors de cette story pour la même raison (un septième module,
  `credit_notes.rs`).

## Acceptance Criteria

1. **AC1 — Une variante dédiée.** `crates/kesh-db/src/errors.rs` gagne
   `DbError::SettlementCounterpartyIsClaimAccount { account_id: i64, account_number: Option<String>, claim: ClaimSide, role: SettlementAccountRole, batch: Option<SettlementBatchContext> }`,
   l'énumération `ClaimSide { Receivable, Payable }` (`as_str()` → `"receivable"` / `"payable"`),
   l'énumération **`SettlementAccountRole`** — d'où vient le compte en cause, donc quel est le remède
   (finding R1 = F1 de la P3, choix C-15-6-26) :
   - `Counterparty` (`"counterparty"`) — la contrepartie que l'utilisateur **choisit** (compte interne,
     compte bancaire du virement) ou que porte le **compte bancaire** du geste (lot, rapprochement) ;
     remède : un autre compte, ou relier le compte bancaire à son propre compte de banque ;
   - `Rounding` (`"rounding"`) — le compte de différences d'arrondi des **réglages**
     (`default_rounding_account_id`), **quel que soit le geste** : règlement (compte d'arrondi du
     règlement), rapprochement (c-bis), et au solde du reste le compte du reste d'arrondi comme le
     compte de la nature
     `rounding` — `write_off_account_for_write(.., Rounding)` lit la **même colonne**
     (`company_invoice_settings.rs:438-441`) que `rounding_account_for_write` (`:408`) (findings R1 = F1
     de la P4, choix C-15-6-30) ;
   - `WriteOffNature` (`"write_off_nature"`) — le compte d'une nature de solde des **réglages**, pour
     les natures `discount`, `bank_fees` et `bad_debt` seulement (les trois champs de la section
     *Solde du reste* des réglages, `settings/invoicing/+page.svelte:29`) ;
   - `VatPayable` (`"vat_payable"`) — le compte de TVA due des **réglages** (solde du reste) ;
   pour les trois derniers, le remède est de désigner un autre compte dans *Paramètres → Facturation*.
   `claim: Payable` ne se combine qu'avec `Counterparty` (le règlement fournisseur n'écrit aucun
   compte d'écart) — et ce n'est pas un doc-comment qui le tient (finding F5 de la P5) : le helper de
   construction ne prend **pas** la paire `(claim, role)` mais un **sujet** qui rend la combinaison
   `Payable` + rôle désigné irreprésentable, p. ex.
   `ClaimSubject::{ Counterparty(ClaimSide), Designated(DesignatedRole) }` avec
   `DesignatedRole::{ Rounding, WriteOffNature, VatPayable }` (un compte désigné est toujours comparé à
   une **créance** : `claim = Receivable` s'en déduit) — `ClaimSubject` et `DesignatedRole` vivent
   **tous deux** dans `invoice_settlements.rs`, à côté du helper qui les consomme (finding R6-8 de la
   P6 : `DesignatedRole` n'avait ni fichier ni place au décompte) ; `SettlementAccountRole`, dont
   `DesignatedRole` est un sous-ensemble, reste dans `errors.rs` avec la variante ; la variante garde ses deux champs, remplis
   par le helper. Le doc-comment dit aussi que **le rôle suit la colonne de réglage lue, pas la
   route** — un même compte désigné sort sous le même `details.role` partout. Et
   `SettlementBatchContext { payment_batch_id: i64, supplier_invoice_id: i64, supplier_invoice_number: Option<String> }`
   (signatures indicatives). `batch` vaut `None` partout sauf à la **confirmation d'un lot** (AC6), où
   `confirm_batch` le remplit en interceptant le refus de `pay_in_tx`. `error_code()` →
   `"SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"` — **un** code pour les quatre rôles (le défaut est le
   même : une écriture `D X / C X`) ; `details.role` et le message distinguent le remède. Doc-comment :
   le motif (écriture nulle, reste dû qui baisse sans mouvement), #474.
   **Helper commun** (finding F3 de la P3, DRY ; choix C-15-6-26) — dans
   `crates/kesh-db/src/repositories/invoice_settlements.rs`, à côté du lecteur de la 15-6a, la
   **comparaison** est séparée de la **construction du refus** (signatures indicatives) :
   - `pub fn ensure_not_claim_account(account_id: i64, claim_account_id: i64) -> Result<(), ClaimAccountClash>`
     — **pure**, sans I/O ni transaction ; `ClaimAccountClash { account_id: i64 }` à l'égalité. C'est
     la seule partie que la **15-6d** emprunte (sa garde compare la contrepartie au compte de banque ;
     son refus reste le `VALIDATION_ERROR` du flux ventilé, C-15-6-28) ;
   - `pub async fn claim_account_refusal(conn: &mut sqlx::MySqlConnection, company_id: i64, clash: ClaimAccountClash, subject: ClaimSubject) -> DbError`
     — appelée **à l'échec seulement** : lit `accounts.number` (`SELECT number FROM accounts WHERE id = ? AND company_id = ?`,
     dans la transaction de l'appelant) et rend la variante, `batch: None` ; une erreur SQL de cette
     lecture est rendue telle quelle (`DbError` de `map_db_error`) ;
   - `pub fn claim_account_refusal_details(account_id: i64, account_number: Option<&str>, claim: ClaimSide, role: SettlementAccountRole, bank_account_id: Option<i64>) -> serde_json::Value`
     (`kesh-db` dépend déjà de `serde_json`) — construit les `details` pour les **trois**
     consommateurs : le mapping HTTP (AC2), le `FailedProposal` du rapprochement (AC5) et le
     `PaymentBatchFailedItem` de la création de lot (AC6), pour que leurs clés ne divergent pas. Les
     clés **de lot** (`paymentBatchId`, `supplierInvoiceId`, AC2 et AC6) sont **hors** de ce helper :
     le **mapping HTTP** les ajoute, et lui seul, quand la variante porte `batch: Some` — le seul
     consommateur qui en ait (finding F6-5 de la P6).
   Tous les sites des AC3, AC3 bis, AC4, AC5 et AC6 appellent `ensure_not_claim_account`, puis, sur
   `Err(clash)` seulement, `claim_account_refusal` ; la lecture du numéro **à
   l'échec seulement** remplace les « lectures neuves » écrites site par site.
2. **AC2 — Sa réponse HTTP.** `crates/kesh-api/src/errors.rs` : **400**, corps
   `{ "error": { "code": "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT", "message": …, "details": { "accountId": …, "accountNumber": …, "claim": "receivable" | "payable", "role": "counterparty" | "rounding" | "write_off_nature" | "vat_payable" } } }`
   (`details` construits par `claim_account_refusal_details`, AC1). Le patron de
   `CREDIT_NOTE_INVOICE_SETTLED` (`crates/kesh-api/src/errors.rs:2838-2845`) ne vaut **que pour la
   forme de `details`** : lui est un **409** et appelle `t(` ; ce refus-ci est un **400** et emploie
   `t_args` (`errors.rs:52`, privé au module, utilisable). Avec `batch`, `details` gagne
   `paymentBatchId` et `supplierInvoiceId`. Message par `t_args`, variable `$account` (le numéro ;
   repli `#<id>` s'il manque), **six** clés **plates** — la clé suit le **rôle**, pas seulement le
   côté :
   - `error-settlement-counterparty-is-receivable` — fr : « Le compte { $account } est le compte
     débiteurs de cette facture : un règlement doit faire sortir la créance vers un autre compte
     (banque, caisse, compensation…). »
   - `error-settlement-counterparty-is-payable` — fr : « Le compte { $account } est le compte
     créanciers de cette facture : un règlement doit éteindre la dette par un autre compte (banque,
     caisse, compensation…). »
   - `error-settlement-counterparty-is-payable-in-batch` (avec `batch`, variable `$invoice` = numéro
     de la facture fournisseur, repli `#<id>`) — fr : « Le compte bancaire de ce lot est lié au compte
     { $account }, le compte créanciers de la facture { $invoice } : le lot ne peut pas être confirmé.
     Reliez le compte bancaire à son propre compte de banque puis confirmez de nouveau, ou annulez le
     lot et réglez la facture depuis sa fiche. » — le remède de la confirmation n'est pas celui du
     règlement unitaire : l'utilisateur n'y choisit aucun compte (finding F4 de la P2).
   - **trois clés plates pour un compte désigné dans les réglages**, une par rôle — **aucun
     sélecteur Fluent** (finding R5-1 de la P5, choix C-15-6-33 ; même choix que la 15-5c, son AC4) :
     un sélecteur neuf rougirait `no_new_select_expression_reaches_the_frontend_dictionary`
     (`crates/kesh-i18n/src/loader.rs:482`, liste `SELECTEURS_RESOLUS_COTE_SERVEUR`, `:367`), et le
     dictionnaire du frontend le résoudrait sans argument, sur sa variante par défaut. Textes fr :
     - `error-settlement-rounding-account-is-receivable` (rôle `rounding`) — « Le compte
       { $account }, désigné dans les réglages comme compte de différences d'arrondi, est le compte
       débiteurs de cette facture : un administrateur doit désigner un autre compte dans
       Paramètres → Facturation. »
     - `error-settlement-write-off-account-is-receivable` (rôle `write_off_nature`) — « Le compte
       { $account }, désigné dans les réglages comme compte de cette nature de solde, est le compte
       débiteurs de cette facture : un administrateur doit désigner un autre compte dans
       Paramètres → Facturation. »
     - `error-settlement-vat-payable-account-is-receivable` (rôle `vat_payable`) — « Le compte
       { $account }, désigné dans les réglages comme compte de TVA due, est le compte débiteurs de
       cette facture : un administrateur doit désigner un autre compte dans Paramètres →
       Facturation. »
     « Un administrateur doit » et non « désignez » (finding F4 de la P4) :
     le solde du reste et le règlement sont ouverts au Comptable, les réglages à l'Administrateur seul
     (`crates/kesh-api/src/lib.rs:207-210`) — c'est la formulation que C36 a retenue pour la 15-5d,
     vraie aussi pour un client d'API. L'utilisateur n'a choisi **aucun** compte dans le dialogue de
     règlement ou de solde : le renvoyer vers « un autre compte (banque, caisse…) » serait un remède
     faux (findings R1 = F1 de la P3). **Repli** : chaque clé a son `default` de `t_args`
     (`crates/kesh-api/src/errors.rs:52`), la phrase fr complète — plus de `match` **de sélection**
     à construire en Rust (il n'existait que pour le sélecteur). ⚠️ `t_args` rend ce `default`
     **tel quel**, sans les `FluentArgs` (`errors.rs:58`, `None => default.to_string()`) : il est
     donc **construit par `format!`** avec le numéro (et `$invoice` pour la clé de lot), patron
     `errors.rs:88-92` (`let fallback = format!("Ligne {n}"); … t_args(…, &fallback, &args)`) — un
     `default` littéral « Le compte { $account } … » laisserait fuir le gabarit (finding R6-5 de la P6).
   Choix de la clé côté serveur, **un rôle, une clé** : `role = Counterparty` → `-is-receivable` /
   `-is-payable` (et `-is-payable-in-batch` avec `batch`) ; `Rounding` → `-rounding-account-is-receivable` ;
   `WriteOffNature` → `-write-off-account-is-receivable` ; `VatPayable` →
   `-vat-payable-account-is-receivable`.
   Dans les **quatre** locales (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`),
   placées près de `error-credit-note-blocked-settled` ; `npm run lint-i18n-ownership` vert.
3. **AC3 — Règlement client, deux modes.** Dans `settle_invoice`, **aussitôt après** la résolution
   de la contrepartie (compte bancaire du virement, ou compte interne) et donc **après** ses refus
   existants — 404 compte bancaire, configuration absente, `INACTIVE_OR_INVALID_ACCOUNTS`, et
   `ACCOUNT_NOT_POSTABLE` que la **15-5a** (son AC4, mergée avant) y insère pour le compte interne —,
   et **avant** le refus du trop-perçu :
   si `counterparty_account_id == receivable_account_id` → `SettlementCounterpartyIsClaimAccount { claim: Receivable, role: Counterparty, batch: None, … }`,
   par `ensure_not_claim_account` puis `claim_account_refusal` (AC1). La garde compare l'identifiant que **le lecteur de la 15-6a**
   a déjà rendu (choix C-15-6-11). Numéro : lu par le helper **à l'échec seulement**, dans les deux
   modes (le `number` que la 15-5a ajoute au `SELECT` du compte interne n'est pas réutilisé : un seul
   chemin vaut mieux que l'économie d'une lecture sur un refus). Rien n'est écrit : ni écriture,
   ni ligne `invoice_settlements`, ni `paid_at`, ni `version`, ni audit (la transaction est
   abandonnée). Ordre écrit dans le doc-comment de la fonction.
   **AC3 bis — Les comptes d'écart ne sont pas la créance** (choix C-15-6-18). Le même refus, par la
   même comparaison d'identifiants avec la créance de la vente (lecteur de la 15-6a), garde les comptes
   que le geste écrit **en face** de la créance. **Règle de placement, unique** (findings F1 et F6 de
   la P5, choix C-15-6-33 ; précisée par R6-4 = F6-6 de la P6) : **chaque comparaison suit la
   lecture de son compte et celle de la créance** — aussitôt la plus tardive des deux —, et donc les
   refus de configuration ou d'état de ce compte ; l'ordre des refus est celui des lectures du code — que la
   15-5e, réécrite par C54, **ne change pas** (C55 : ni l'arrondi avant le compte interne, ni avant la
   nature) ; T0 le relève :
   - `settle_invoice`, **compte d'arrondi du règlement** (rendu par
     `rounding_account_for_write(.., Payment)`, `invoice_settlements_write.rs:189` au commit
     `1920381e`) — `role: Rounding`. Ordre des refus, dans les deux modes : **contrepartie** (compte
     bancaire ou compte interne : leurs refus, puis la comparaison de l'AC3, `role: counterparty`) →
     **trop-perçu** → **compte d'arrondi** (`ROUNDING_ACCOUNT_NOT_CONFIGURED`, puis cette comparaison,
     `role: rounding` ; trop-perçu et écart d'arrondi sont exclusifs) ;
   - `write_off_invoice` — ordre des lectures **nature → reste d'arrondi → TVA due** (`:430-490` au
     commit `1920381e`, inchangé par la 15-5e) : le compte de la **nature** — `role: WriteOffNature`
     pour `discount`, `bank_fees`, `bad_debt`, `role: Rounding` pour la nature `rounding` (il **est**
     alors le compte d'arrondi, un seul compte, une seule comparaison ; le reste d'arrondi n'est pas lu
     séparément, `:471-472`) — ; le compte du **reste d'arrondi** (lu quand le reste n'est pas au
     centime) — `role: Rounding` (même colonne que le compte d'arrondi du règlement, AC1) — ; le compte
     de **TVA due** — `role: VatPayable`. La **nature** est lue **avant** la créance
     (`write_off_account_for_write` `:433`, puis la créance `:434-445`) : elle est comparée aussitôt
     la **créance** lue ; le reste d'arrondi et la TVA due, aussitôt leur propre lecture. Aucun refus
     métier ne sépare la nature de la créance (seul l'`Invariant` d'une vente malformée) : l'ordre des
     refus n'en change pas — et la lecture de la créance **ne s'avance pas** (C55 : aucun
     réordonnancement).
     **Ordre des refus quand plusieurs coïncident avec la créance** (finding F7 de la P4) : celui des
     lectures — **nature, puis reste d'arrondi, puis TVA due** ; le premier égal est nommé (un seul
     `account_id` / `role` par refus), ce que le doc-comment écrit, pour que le message et
     `details.role` soient déterministes. Et vis-à-vis des refus de configuration (finding F6 de la
     P5) : une **nature** égale à la créance est refusée **avant** la lecture du reste d'arrondi et de
     la TVA due, donc avant leurs `WRITE_OFF_ACCOUNT_NOT_CONFIGURED` / `CONFIGURATION_REQUIRED`. Le
     test 10 ter le fige.
   - `accept_one_invoice`, étape (c-bis) : le compte d'arrondi (`reconciliation.rs:1487`) →
     `FailedProposal` comme l'AC5, `details = { "accountId", "accountNumber", "claim": "receivable", "role": "rounding" }`
     — **sans** `bankAccountId`, qui n'a pas de sens pour un compte d'arrondi (finding F1 de la P3).
   `account_id` du refus = le compte d'écart fautif (qui **est** la créance) ; message : la clé plate
   du rôle (AC2 — `-rounding-account-`, `-write-off-account-`, `-vat-payable-account-is-receivable`),
   qui renvoie à *Paramètres → Facturation*. Ferme le chemin du changement de type confirmé (`accounts.rs:497-512`) ; « résolu par
   le type » ne vaut plus pour ces comptes. Le contrôle de type existant reste (il garde ce qu'il
   gardait) — et pour la TVA due, il n'y en a pas au moment d'écrire (§ *Le défaut*) : la comparaison
   est sa seule garde.
4. **AC4 — Règlement fournisseur, deux modes.** Même garde dans `pay_in_tx`, après l'étape (3) et ses
   refus existants (dont `ACCOUNT_NOT_POSTABLE` de la 15-5a — absent du code de `1920381e`, sa place
   se relit en T0 ; le bloc du compte interne est aujourd'hui `supplier_invoices.rs:622-656`) :
   `counterparty_account_id == payable_account_id` → `claim: Payable, role: Counterparty`, par
   `ensure_not_claim_account` puis `claim_account_refusal` (AC1). Mêmes garanties « rien
   d'écrit ». Le commentaire « jumeau exact » de `:623-641` mentionne la nouvelle garde. **DRY**
   (choix C-15-6-11) : la requête de l'étape (2) (`:582-593`, ligne de **crédit** de l'écriture
   d'achat, **et** le TTC) n'est pas celle du lecteur de la 15-6a (ligne de débit, sans montant) ; elle
   est extraite en lecteur sœur
   `supplier_invoices::purchase_payable_line(conn, company_id, purchase_entry_id) -> Result<Option<(i64, Decimal)>, DbError>`
   (signature indicative), appelé par `pay_in_tx` et par la création de lot (AC6) — deux appelants,
   une requête.
5. **AC5 — Rapprochement d'une facture.** Dans `accept_one_invoice`, après la lecture de la créance
   (étape (b), par le lecteur de la 15-6a) et **avant** le contrôle du trop-perçu (étape (c)) :
   `bank_ledger_account_id == receivable_account_id` →
   `FailedProposal { error_code: "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT", details: { "bankAccountId", "accountId", "accountNumber", "claim": "receivable", "role": "counterparty" } }`
   (refus rendu par `ensure_not_claim_account` / `claim_account_refusal`, converti par une seule fonction de
   `reconciliation.rs` qui appelle `claim_account_refusal_details`, AC1, pour cette étape et (c-bis))
   — **jamais** d'`AppError` global (§ *Pattern batch* du `CLAUDE.md`) ; HTTP 200, la proposition dans
   `failed[]`, les autres du lot traitées normalement. Les refus antérieurs à l'étape de la créance
   gardent **tous** la priorité (`RECONCILIATION_SCORE_TOO_LOW` de l'étape 7bis compris), notamment :
   `BANK_ACCOUNT_NOT_CONFIGURED` (compte bancaire sans journal), `INVOICE_SALE_ENTRY_MALFORMED`
   (étape (b)) ; le trop-perçu (`RECONCILIATION_OVERPAYMENT`) vient **après**. Erreur SQL de la
   lecture du numéro → `DATABASE_ERROR` comme les étapes voisines. Le commentaire de classement que la
   **15-5b** (son AC6) pose dans cette fonction — « reste tel quel … ses comptes ne viennent pas du
   client » — est **amendé** : le compte de banque vient de la configuration et peut être la créance ;
   il est désormais gardé ici (sinon le commentaire deviendrait trompeur).
6. **AC6 — Lots de paiement : refus à la création, garde à la confirmation** (choix C-15-6-10,
   précisé par C-15-6-19).
   - **Création** (`create_batch` → `validate_invoice_for_batch`, `payment_batches.rs:226-300`) : pour
     chaque facture, la dette lue sur l'écriture d'achat (lecteur sœur de l'AC4) est comparée au
     `journal_account_id` du compte bancaire source ; égalité → la facture va dans `failed[]` du lot
     (`PaymentBatchFailedItem`), `error_code = "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"`,
     `details = { "bankAccountId", "accountId", "accountNumber", "claim": "payable", "role": "counterparty" }`
     (par `claim_account_refusal_details`, AC1) — patron batch, pas d'erreur globale. **Source de
     `accountNumber`** (finding R7 de la P3) : `create_batch` ne lit du compte bancaire que
     `journal_account_id, archived` (`payment_batches.rs:83-90`) ; le numéro est lu par
     `claim_account_refusal` **à l'échec seulement** (une `SELECT number FROM accounts` dans la
     transaction du lot), puis le refus est converti en item. Une **erreur SQL** de cette lecture
     (finding R7 de la P4) n'est pas un refus de la facture : elle est **propagée**, échec infra de
     toute la création, comme les autres `DbError` de `validate_invoice_for_batch`
     (`payment_batches.rs:229`) — seul le refus métier devient un item. **Signature** :
     `validate_invoice_for_batch` ne lit aujourd'hui ni `purchase_journal_entry_id` (son `SELECT`,
     `:251-253`) ni le compte lié du compte bancaire (`create_batch` le lit `:84-103` sans le
     transmettre, appel `:114`) : elle gagne la colonne et un paramètre `bank_ledger_account_id: i64`. **Écriture d'achat sans ligne de crédit**
     (`purchase_payable_line` → `None` ; `pay_in_tx` en fait un `Invariant`, 500,
     `supplier_invoices.rs:593`) : à la création, c'est une donnée de **cette** facture — item
     `failed[]` `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED`,
     `details = { "reason": "no_credit_line_on_purchase_entry", "purchaseEntryId" }`, patron de
     `INVOICE_SALE_ENTRY_MALFORMED` du rapprochement (§ *Pattern batch* du `CLAUDE.md` : une erreur qui
     dépend de la proposition individuelle ne devient pas une erreur globale). Ces contrôles viennent
     **après** les refus existants de la facture (`SUPPLIER_INVOICE_NOT_FOUND`, `…_NOT_OPEN`,
     `NO_PAYMENT_COORDINATES`, `ALREADY_IN_GENERATED_BATCH`, IBAN). Si **aucune** facture n'est
     retenue, aucun lot n'est créé et **aucun fichier pain.001 n'est produit** (comportement existant
     `accepted.is_empty()`, `:120`). Le `fn fail` local (`:235-241`, `details: None`) reçoit une
     variante avec `details`, ou le contrôle construit l'item lui-même. **Ce refus à la création est la
     défense principale.**
   - **Confirmation** (`confirm_batch`, `:304`, boucle `pay_in_tx` `:359-370`) : la garde de
     `pay_in_tx` (AC4) **reste** — le compte bancaire a pu être relié entre la création et la
     confirmation. `confirm_batch` intercepte ce refus et le rend **avec son contexte** (`batch:
     Some(SettlementBatchContext { payment_batch_id, supplier_invoice_id, supplier_invoice_number })`).
     **Source de `supplier_invoice_number`** (finding F5 de la P3) : `pay_in_tx` le lit
     (`inv.supplier_invoice_number`) mais ne le rend pas, et l'item de lot n'a que
     `supplier_invoice_id` (`ITEM_COLS`, `payment_batches.rs:47-48`). Tranché : `confirm_batch`, en
     interceptant le refus, fait **une** `SELECT supplier_invoice_number FROM supplier_invoices WHERE id = ? AND company_id = ?`
     dans la même transaction (permise : le refus est une erreur métier, aucune erreur SQL n'a rendu
     la transaction inutilisable) — **pas** de champ de plus dans la variante, qui resterait vide
     partout ailleurs. Effet :
     la confirmation entière échoue en **400** `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`
     (transaction atomique), `details` portant `paymentBatchId` et `supplierInvoiceId`, message
     `error-settlement-counterparty-is-payable-in-batch` (AC2) qui dit les **deux issues** : relier le
     compte bancaire à son propre compte de banque puis confirmer de nouveau, ou annuler le lot
     (`cancel_batch`, `:431` — les factures redeviennent réglables à l'unité) et régler la facture
     depuis sa fiche. Le lot reste `generated`, aucun règlement écrit. Le doc-comment de `confirm_batch`
     dit les deux contrôles, pourquoi le second subsiste, et les deux issues.
   - **Écran** : `failedItemLabel` (`frontend/src/lib/features/payment-batches/payment-batch-helpers.ts:78`)
     reçoit les **deux** codes neufs, clés `payment-batches-failed-counterparty-is-claim-account` et
     `payment-batches-failed-purchase-entry-malformed` **écrites en toutes lettres** (règle de la 23-3
     rappelée dans le JSDoc), quatre locales. Textes **fr** de référence (finding F6 de la P3) :
     `payment-batches-failed-counterparty-is-claim-account` — « Le compte bancaire du lot est lié au
     compte créanciers de cette facture » ; `payment-batches-failed-purchase-entry-malformed` —
     « Écriture d'achat sans ligne de crédit : facture à vérifier ». Le libellé n'affiche pas le
     numéro du compte (le `failed[]` du lot ne montre que `#id — libellé`,
     `payment-batches/+page.svelte:200`) : le compte en cause est celui du compte bancaire choisi à
     l'écran, il n'y a rien d'autre à nommer. **Les « six » deviennent « huit » aux QUATRE sites**
     (finding R4 de la P3, jeton grepé) : `payment-batch-helpers.ts:59` (« Les six libellés sont
     RELEVÉS, pas inventés » — à amender : les deux neufs ne sont **pas** relevés du glossaire, ils sont
     écrits par cette story, et le JSDoc le dit), `payment-batch-helpers.ts:73` (« Les six clés »),
     `payment-batch-helpers.test.ts:59` (commentaire « Les six codes sont désormais assertés ») et
     `:61` (`it('mappe les six codes …')`). Le refus de confirmation
     s'affiche par son message traduit dans la fiche du lot
     (`frontend/src/routes/(app)/payment-batches/[id]/+page.svelte:40-50`, `confirmBatch` :
     `catch (err) { if (isApiError(err)) actionError = err.message; }`, `:45`) — **vérifié à la P2,
     aucun changement**.
   - **Écran de création d'un lot** (`frontend/src/routes/(app)/payment-batches/+page.svelte:53`, filtre
     `journalAccountId !== null`) : **non filtré, choix écrit** (C-15-6-19) — il ne charge pas le plan
     comptable, et le refus y est lisible par facture (`failed[]` libellé) ; voir § *Angles morts*.
7. **AC7 — Libellé dans le `failed[]` du rapprochement.** Si la **15-5c** est mergée (module
   `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`, #492 — vérifier par
   `test -f` sur ce chemin), le code neuf y reçoit son libellé, dans les quatre locales, **à deux
   cas selon `details.role`** (finding F1 de la P3) : `counterparty` — clé
   `reconciliation-failed-counterparty-is-claim-account`, fr « Le compte bancaire est lié au compte
   débiteurs de la facture : reliez-le à son propre compte de banque » ; `rounding` — clé
   `reconciliation-failed-rounding-account-is-claim-account`, fr « Le compte d'arrondi désigné est le
   compte débiteurs de la facture : un administrateur doit en désigner un autre dans Paramètres →
   Facturation » (C36, finding F4 de la P4 ; préfixes à
   ajuster à la convention que la 15-5c aura posée, sans changer le sens). Le test de la liste en
   dur du module (`failed-proposal-label.test.ts`) inclut les deux cas, et ses deux sites neufs entrent
   dans le relevé de la **borne exacte** `sitesTotal` (AC8, dernier point). Sinon : le signaler dans le
   Dev Agent Record et à l'orchestrateur, pour que la 15-5c l'ajoute à sa liste — **ne pas** poser un
   second mécanisme.
8. **AC8 — Les écrans ne proposent plus le compte soldé** (choix C-15-6-4, signature fixée par
   C-15-6-14, archivés tranchés par C-15-6-17). Trois fonctions pures dans
   `frontend/src/lib/features/accounts/account-options.ts`, partagées avec la 15-6c :
   - `accountIdsWithRole(accounts, role)` → `Set<number>` des comptes portant ce rôle ;
   - `withoutAccountIds(accounts, ids)` → les comptes dont l'id n'est pas dans `ids` ;
   - `bankAccountsNotLinkedTo(bankAccounts, ids)` → les comptes bancaires dont `journalAccountId`
     n'est pas dans `ids`.
   JSDoc : pourquoi le rôle ici (singleton par société, réglages dérivés), que la garde serveur,
   exacte, reste l'autorité, et que **les ids se calculent sur la liste que l'écran reçoit, avant le
   filtre `active && postable`** — sans quoi un compte bancaire lié à un 1100 devenu **non imputable**
   ne serait pas écarté. ⚠️ **Les comptes archivés n'y sont pas, et c'est voulu** (C-15-6-17) : les
   trois écrans chargent le plan **sans** les archivés (`invoices/[id]/+page.svelte:554` et
   `invoices/due-dates/+page.svelte:102` : `fetchAccounts(false)` ; `supplier-invoices/[id]/+page.svelte:80` :
   `fetchAccounts()`, défaut `false`). Un compte bancaire lié à un compte **archivé** reste donc
   proposé — mais il ne peut produire **aucune** écriture : la garde `active` de `create_in_tx`
   refuse tout compte archivé, et une créance archivée est refusée de même. Le serveur suffit ; le
   JSDoc le dit.
   - `SettleInvoiceDialog.svelte` : `selectableAccounts` (`:105`) et la liste des comptes bancaires
     (`:208`) passent par ces fonctions avec les ids du rôle `Receivable` calculés sur la prop
     `accounts` ; le compte bancaire **présélectionné** (`:89`, primaire ou premier) est choisi **dans
     la liste filtrée**. ⚠️ Le `$effect` de réinitialisation (`:82-91`) lit aujourd'hui `open`,
     `amountDue` et `bankAccounts` ; choisir la présélection dans la liste filtrée le ferait dépendre
     aussi de `accounts`, et une liste `accounts` arrivée **après** l'ouverture **effacerait la saisie
     en cours** (date, montant, mode). La présélection vit donc dans un **effet distinct**, qui ne
     touche ni la date ni le montant ni le mode, et ne remplace `bankAccountId` que si la valeur
     courante est **absente de la liste filtrée** (`null` compris) ; l'effet de réinitialisation garde
     ses seules dépendances (finding F6-3 de la P6 ; test 18). Si la liste filtrée est **vide** **et** qu'au moins un compte bancaire a été
     **écarté par le filtre** (`bankAccounts.length > filtered.length` — le seul compte bancaire est lié
     au compte débiteurs ; finding F3 de la P4), le dialogue l'**écrit** — clé
     `invoices-settle-no-eligible-bank-account`, fr « Aucun
     compte bancaire utilisable : le seul compte lié est le compte débiteurs de cette facture. Reliez
     un compte bancaire à son propre compte de banque, ou réglez par un compte interne. » (premier
     segment `invoices` : conforme à `lint-i18n-ownership` pour un fichier de
     `src/lib/features/invoices/`, sans entrée neuve dans `KNOWN_VIOLATIONS`), quatre locales — au
     lieu d'un menu vide silencieux. Une société **sans aucun** compte bancaire (installation neuve)
     a aussi une liste filtrée vide, mais rien n'a été écarté : le message, qui accuse le compte
     débiteurs, y serait faux — le dialogue garde alors son comportement actuel (aucun état vide
     n'existe aujourd'hui, `grep -n "length === 0" SettleInvoiceDialog.svelte` ne rend rien).
   - `supplier-invoices/[id]/+page.svelte:83-84` : idem avec `Payable` ; les ids se calculent sur
     `accts` **avant** le filtre `accts.filter((a) => a.active && a.postable)` de la ligne 84 ; liste
     vide **et** au moins un compte bancaire écarté par le filtre des comptes créanciers (la page filtre
     déjà `journalAccountId !== null`, `:83` : une société sans compte bancaire **lié** n'a rien
     d'écarté, et garde son comportement actuel) → même traitement, clé
     `supplier-invoices-pay-no-eligible-bank-account` (alignée sur ses
     voisines `supplier-invoices-pay-*`, `:96`, `:100`, `:322`), fr « Aucun compte bancaire
     utilisable : le seul compte lié est le compte créanciers de cette facture. Reliez un compte
     bancaire à son propre compte de banque, ou réglez par un compte interne. »
   - Les écrans qui alimentent le dialogue (`invoices/[id]/+page.svelte`, `invoices/due-dates/+page.svelte`)
     ne changent pas : le filtre vit dans le dialogue, et ils lui passent déjà la liste des comptes
     actifs non filtrée par `postable`.
   - **Borne exacte `sitesTotal`** (`frontend/src/lib/shared/i18n-keys.test.ts:456`, assertée `:742` ;
     **elle existe déjà** — 1868 au 2026-10-08 —, ce n'est pas la 15-5c qui la pose ; finding R5-2 de
     la P5) : elle est relevée pour **tous** les sites `i18nMsg` neufs de la story, comptés —
     **deux** dans `failedItemLabel` (AC6), **deux** pour les messages de liste vide (AC8 :
     `SettleInvoiceDialog.svelte` et `supplier-invoices/[id]/+page.svelte`), et **deux** de plus si la
     15-5c est mergée (libellé `failed[]` du rapprochement, AC7) : **+4** attendus, **+6** avec la
     15-5c. Le chiffre se **recompte** par le relevé du test sur l'état **rebasé** (l'ordre de merge des
     15-5c, 15-5d et 15-6b change la base), aux deux bornes, et la ventilation (`sitesNonResolus`,
     `relais`, `sitesGabarit`) se recompte avec lui ; le commentaire de la borne reçoit une ligne qui
     nomme la story et les sites, sur le patron des lignes existantes (`:441-455`).
9. **AC9 — Les refus serveur s'affichent** dans le dialogue client, dans le **dialogue de solde du
   reste** et sur la fiche fournisseur (message traduit de l'AC2, non le code). Vérifié par lecture
   (P1, lentille R ; P2 ; P3) : les `catch` qui lisent `ApiError.message` sont dans les **parents** du
   dialogue de règlement (`invoices/due-dates/+page.svelte:265-268`,
   `invoices/[id]/+page.svelte:582-585`), qui passent la prop `errorMsg` ; le **solde du reste**
   (AC3 bis, rôles `write_off_nature`, `rounding`, `vat_payable`) s'affiche par
   `invoices/[id]/+page.svelte:509` (`writeOffError = err.message;`, rendu `:1577`
   `errorMsg={writeOffError}`) — c'est **là** que les messages des comptes désignés (AC2, trois clés
   plates : « un administrateur doit désigner un autre compte dans Paramètres → Facturation ») sont
   lus, et, pour `rounding`, **aussi dans le dialogue de règlement** (compte d'arrondi du règlement,
   AC3 bis : `markError = err.message`, `invoices/[id]/+page.svelte:584`,
   `invoices/due-dates/+page.svelte:267`, prop `errorMsg={markError}` `:519` ; finding R6-10 de la
   P6) — et c'est pourquoi ils ne
   renvoient pas à un choix de compte que ce dialogue n'offre pas (finding R1 = F1 de la P3) ; ceux de la
   fiche fournisseur sont dans la page — **aucun changement de code**. Le test Vitest (§ *Tests*) ne fige que
   le **rendu** de la prop `errorMsg` par le dialogue, pas la lecture de `err.message` par les parents
   (montage trop lourd pour ce qu'il prouverait) : c'est dit, pas surestimé.
10. **AC10 — Documentation des intégrateurs** (`docs/api-external.md`). Le § 10 *Gestion des
    erreurs* (`:397-415`) **n'est pas touché** : il ne porte que des codes génériques
    (`UNAUTHENTICATED`, `API_KEY_*`, `VALIDATION_ERROR`, `NOT_FOUND`), et la convention du guide est la
    **table de refus par route** (`CREDIT_NOTE_INVOICE_SETTLED` n'y figure pas non plus). Les sites,
    tranchés (findings R2 = F2 de la P3, choix C-15-6-26) :
    - **Solde du reste** — table « Refus / Code / Statut » de `POST /api/v1/invoices/{id}/write-off`
      (`:274-283`) : une ligne « Un compte désigné pour la nature, le reste d'arrondi ou la TVA due est
      le compte débiteurs de la facture | `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` | `400`,
      `details.role` (`write_off_nature`, `rounding`, `vat_payable`) ».
    - **Règlement client** — `POST /api/v1/invoices/{id}/settlements` n'a ni section ni table (le
      guide ne le cite qu'en renvoi, `:257`, `:272`) : une phrase « Refus de `POST
      /api/v1/invoices/{id}/settlements` (enregistrer un règlement) » ajoutée sous le § *Lister et
      annuler les règlements d'une facture* (`:253`), **hors** de la table de ce § (`:259-268`), qui est
      celle de l'**annulation** — sans quoi un intégrateur la lirait comme un refus d'annulation (finding
      F6 de la P4) ; elle nomme la route, le code, `400` et les `details` (`claim`, `role` :
      `counterparty` ou `rounding`).
    - **Règlement fournisseur et lots** — `POST /supplier-invoices/{id}/pay` n'est cité qu'en renvoi
      (`:287`) et le guide n'a **aucune** section sur les lots (`grep -n "payment-batch"
      docs/api-external.md` ne rend **rien** ; la mention de `:287` dit « lot de paiement ») : une
      phrase « Refus de `POST /api/v1/supplier-invoices/{id}/pay` (régler une facture fournisseur) »
      sous le § *Annuler le règlement d'une facture fournisseur* (`:285`), à côté de ce renvoi, posée
      **avant** la ligne « Refus : » de l'**annulation** (`:291`, `SUPPLIER_INVOICE_NOT_PAID`…) et
      libellée par sa route, pour qu'un intégrateur ne la lise pas comme un refus d'annulation — même
      consigne que côté client (finding F7 de la P5). Elle nomme la route du paiement (`400`), puis les deux
      routes de lot (ouvertes aux clés : `comptable_routes`, `crates/kesh-api/src/lib.rs:496-505`) :
      `POST /api/v1/payment-batches` (item `failed[]`, avec `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED`)
      et `POST /api/v1/payment-batches/{id}/confirm` (`400`, `details.paymentBatchId` et
      `details.supplierInvoiceId`). Pas de section neuve.
    - **Rapprochement** — dans la **phrase** des refus par proposition du § *Accepter des propositions
      de rapprochement* (`:307`, qui cite en prose `RECONCILIATION_OVERPAYMENT` et
      `RECONCILIATION_INVOICE_NOT_ELIGIBLE` ; le § n'a pas de liste), avec `details.role`
      (`counterparty`, `rounding`).
11. **AC11 — Manuels** (`docs/manual/fr/user-manual.tex`, et `admin-manual.tex` en dernier tiret) :
    - § *Enregistrer et annuler un règlement* (`\label{sec:reglement-client}`, `:1045`) : « ou
      espèces et **tout autre compte** » (`:1048`, jumeau du « libre » fournisseur, présent au PDF
      aplati — finding F5 de la P4) devient « ou espèces et un autre compte de contrepartie » ; après « Kesh
      passe l'écriture *Débit contrepartie / Crédit débiteurs* » (`:1049`), une phrase : la contrepartie
      ne peut pas être le compte débiteurs de la facture — l'écriture serait nulle et le reste dû
      baisserait sans que rien ne bouge au grand livre ; Kesh ne le propose pas et le refuse ; de même un
      compte bancaire lié au compte débiteurs. **Deux cas, deux remèdes** (finding F1 de la P3) : si le
      compte en cause est celui que vous **choisissez** (compte interne, compte bancaire), choisissez-en
      un autre ou reliez le compte bancaire à son propre compte de banque ; si c'est un compte
      **désigné dans les réglages** — compte d'arrondi, compte d'une nature de solde, compte de TVA due
      (AC3 bis) —, un administrateur doit en désigner un autre dans *Paramètres → Facturation*. La
      seconde phrase est écrite
      au paragraphe **Solder le reste** du même § (`:1064-1072`, qui renvoie déjà à la section
      *Solde du reste* des paramètres).
    - § *Régler une facture fournisseur* (`:1295-1305`) : le même énoncé pour le compte créanciers ;
      et « choisissez un compte de contrepartie **libre** » (présent au PDF aplati) est **corrigé** —
      « libre » devient faux.
    - § *Paiements par fichier pain.001* (`\label{sec:paiements-fournisseurs}`, `:1346-1360`) : une
      facture dont le compte créanciers est le compte lié au compte bancaire du lot n'entre pas dans le
      lot (motif affiché) ; si le compte bancaire a été relié depuis, la confirmation est refusée — et
      le manuel dit les **deux issues** : relier de nouveau le compte bancaire à son propre compte de
      banque puis confirmer, ou annuler le lot et régler la facture depuis sa fiche (le fichier
      pain.001 a pu être exécuté par la banque : c'est alors la première).
    - § rapprochement (puce *Accepter*, `:1525`) : la proposition échoue avec son libellé si le compte
      bancaire est lié au compte débiteurs (renvoi à la configuration, que la 15-6c ferme), **ou** si
      le compte d'arrondi désigné est le compte débiteurs (renvoi à *Paramètres → Facturation*) — les
      deux cas du libellé de l'AC7. ⚠️ La
      **15-5c** réécrit cette section : écrire sur sa version si elle est mergée, sinon rebase attendu.
    - § balance âgée (`:1754`, finding F3 de la P5) : la liste des écritures qui écartent le total
      général du solde du compte clients cite « règlement dont la contrepartie est ce compte
      lui-même » comme un cas ordinaire — après la story, ce geste est **refusé**. La phrase devient
      « règlement **enregistré avant la version 0.13.0** dont la contrepartie était ce compte
      lui-même », ou le cas rejoint les « données antérieures » que la même phrase cite déjà. Le reste
      de la phrase (« changement du compte clients dans les réglages », que la 15-6a relit sans le
      réécrire ; « paiement d'une facture fournisseur compensé sur ce compte », que la story n'interdit
      pas — le compte soldé y est 2000, non 1100) **ne change pas**.
    - **Manuel d'administration** (`docs/manual/fr/admin-manual.tex`, § des réglages de facturation ;
      finding F6-1 de la P6 — l'AC n'y portait aucun verdict, et la 15-6c écrivait que la 15-6b n'y
      touche pas) : le remède des trois clés « compte désigné » renvoie l'**administrateur** à
      *Paramètres → Facturation*, et c'est ce manuel qu'il lit. Verdict, ligne par ligne :
      `:2027` (*Compte de différences d'arrondi* : « charge ou produit, actif et imputable ») —
      **inchangée**, elle reste vraie (l'exigence de type est celle de la désignation) ; `:2029`
      (*Comptes du solde du reste*) — **une phrase ajoutée** : un solde du reste est refusé, sans rien
      écrire, si le compte de la nature, le compte de différences d'arrondi ou le compte de TVA due
      est le **compte débiteurs de la facture** (cas d'un compte débiteurs changé de type depuis) ;
      désigner alors un autre compte ici ; `:2033` (paiement avec écart) — la liste « réglage vide,
      ou compte archivé, rendu non imputable ou changé de type depuis » gagne « **ou devenu le compte
      débiteurs de la facture** ». La phrase de frontière de la 15-6c (son AC9) est alignée dans la
      même remédiation.
    PDF régénérés (`make fr`) et contrôlés **aplatis** (`pdftotext … | tr '\n' ' ' | tr -s ' '`), une
    phrase neuve au moins par section — `:1754` et les deux phrases du manuel d'administration compris.
12. **AC12 — CHANGELOG** sous `## [0.13.0] — Non publié`, `### Corrigé`, renvoi à
    [#474](https://github.com/guycorbaz/kesh/issues/474). **Propriétaire de la section** : la première
    story de la version mergée la crée en tête des versions si elle est absente ; les suivantes y
    ajoutent leur entrée.

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `main` après le merge de la 15-6a et des 15-5a à 15-5e ;
  **refaire tous les numéros de ligne** de la fiche sur `HEAD` ; vérifier où la 15-5a a placé
  `ACCOUNT_NOT_POSTABLE` dans `settle_invoice` et `pay_in_tx` (AC3, AC4) ; **relever l'ordre des
  lectures** de `settle_invoice` et de `write_off_invoice` sur `HEAD` et vérifier que l'AC3 bis le suit
  — sinon l'AC3 bis suit le code et le doc-comment le dit ; constater que les routes du règlement, du
  solde, du paiement fournisseur et de la confirmation d'un lot sont rejouées par la 15-5e (un refus
  métier de cette story n'est pas un 1213 : il n'est jamais rejoué) ; relire
  le commentaire de la 15-5b dans `accept_one_invoice` (AC5), tester l'existence de
  `failed-proposal-label.ts` (AC7), relever `sitesTotal` (AC8). Les choix **C36, C52, C54, C55**
  que la fiche cite vivent au registre de la branche des 15-5 (dépôt principal) : absents de cette
  branche avant le rebase, ils y arrivent avec lui (finding R6-11 de la P6).
- [ ] **T1 — Variante, rôle, contexte de lot, code, mapping, i18n** (AC1, AC2).
- [ ] **T2 — Helper commun puis gardes serveur** : `ensure_not_claim_account`,
  `claim_account_refusal` et `claim_account_refusal_details` dans `invoice_settlements.rs` (AC1), **avant** les sites qui les
  appellent ; gardes (AC3, AC3 bis, AC4, AC5), lecteur sœur fournisseur (AC4). Aucun site ne compare
  ni ne construit le refus ni ne lit le numéro par lui-même. **Critère par le nom seul, sur une liste
  fermée** (findings R5-3 et F2 de la P5, choix C-15-6-33 — le critère de la P4, un `grep` mono-ligne
  sur un littéral à champs, rendait vert par construction le littéral que rustfmt coupe sur plusieurs
  lignes, et tout champ contenant un point) :
  `grep -rnF "SettlementCounterpartyIsClaimAccount" crates/*/src | grep -vE ':[0-9]+:\s*//'`
  (occurrences de **code**, commentaires et doc-comments exclus) ne rend des lignes **que** dans ces
  cinq fichiers, chacune pour une raison nommée :
  - `kesh-db/src/errors.rs` — la définition et le bras d'`error_code()` (et leurs tests unitaires) ;
  - `kesh-db/src/repositories/invoice_settlements.rs` — `claim_account_refusal`, **seule**
    construction de la variante (et ses tests) ;
  - `kesh-db/src/repositories/payment_batches.rs` — l'interception de `confirm_batch`, qui reconstruit
    le refus avec `batch: Some`, et la conversion en `PaymentBatchFailedItem` à la création (AC6) ;
  - `kesh-api/src/errors.rs` — le mapping HTTP (AC2) et ses tests ;
  - `kesh-api/src/routes/reconciliation.rs` — la conversion en `FailedProposal` (AC5, AC3 bis).
  Une ligne dans un **autre** fichier est un site non résolu : il est ramené au helper ou ajouté à la
  liste avec sa raison, au Dev Agent Record. Dans les cinq fichiers, chaque occurrence est **lue**
  (pas seulement comptée) : une construction hors de `claim_account_refusal` et de l'interception de
  `confirm_batch` est un défaut. Et `grep -rnF "SELECT number FROM accounts WHERE id = ? AND company_id = ?" crates/*/src`
  ne rend, pour cette story, que `claim_account_refusal` (chaîne écrite sur une ligne — une requête
  coupée par `\` y échapperait : le helper l'écrit d'un tenant, et le doc-comment le dit).
- [ ] **T3 — Lots de paiement** (AC6), serveur (création, écriture d'achat malformée, confirmation
  contextualisée) et libellés.
- [ ] **T4 — Libellé `failed[]` du rapprochement** (AC7) ou signalement.
- [ ] **T5 — Fonctions d'écran et écrans** (AC8, AC9).
- [ ] **T6 — Tests** (§ *Tests*) — chaque test de garde **rougit d'abord** sur le code non corrigé,
  **sauf** les tests de non-régression 3 et 9, verts avant comme après.
- [ ] **T7 — Documentation** : `docs/api-external.md` (AC10), manuel + PDF (AC11), CHANGELOG (AC12).
- [ ] **T8 — Inventaire fermé** (§ *Inventaire*) : rejouer les commandes ; tout site neuf est traité
  ou écrit en angle mort avec sa raison.
- [ ] **T9 — Gates** : backend complet (dépôts `kesh-db` touchés : pas de ciblage), frontend complet
  (`check`, `lint-i18n-ownership`, `test:unit` — dont la borne `sitesTotal` relevée, AC8 —, `build`),
  E2E complet au dernier commit de code (D7).
  Le verdict de lecture sur les E2E existants (§ *Tests*, « E2E existants ») est recopié au Dev Agent
  Record, puis confronté au run : un rouge sur une des specs nommées s'examine d'abord contre ce
  verdict.

## Inventaire — les sites qui écrivent `D X / C X` avec un compte choisi par le client

Règle « Inventorier les sites NON RÉSOLUS » : l'ensemble clos est celui des **écrivains d'une
écriture de règlement ou de rapprochement à deux comptes dont l'un est désigné par l'utilisateur ou par
la configuration**. Commandes de recompte :

```sh
grep -rn "journal_entries::create_in_tx" crates/kesh-db/src crates/kesh-api/src
grep -rn "pay_in_tx\|settle_invoice(" crates/kesh-db/src crates/kesh-api/src
grep -rn "rounding_account_for_write\|write_off_account_for_write\|vat_payable_account_for_write" crates/kesh-db/src crates/kesh-api/src
grep -rn "active && a.postable\|a.active && a.postable" frontend/src
grep -rn "listBankAccounts(" frontend/src
```

| site | compte soldé | contrepartie | verdict |
|---|---|---|---|
| `settle_invoice` (règlement client, deux modes) | créance (vente) | banque / compte interne | **AC3** |
| `settle_invoice`, compte d'arrondi du règlement | créance (vente) | compte d'arrondi | **AC3 bis** |
| `write_off_invoice` (solde du reste) | créance (vente) | nature, reste d'arrondi, TVA due | **AC3 bis** (le type ne suffit pas : changement de type confirmé) |
| `pay_in_tx` (règlement fournisseur, deux modes) | dette (achat) | banque / compte interne | **AC4** |
| `create_batch` / `confirm_batch` (lots) | dette (achat) | banque du lot | **AC6** |
| `accept_one_invoice` (rapprochement d'une facture) | créance (vente) | banque ; arrondi (c-bis) | **AC5**, **AC3 bis** |
| `post_manual`, `accept_one_rule` | compte de banque | compte choisi | **15-6d** (#524) |
| `accept_one_split`, `post_split` | compte de banque | comptes choisis | déjà gardés (`:1941-1950`, `:3462-3475`) |
| `credit_notes.rs:548` `create_credit_note` | créance (vente) | produit, TVA, arrondi | **15-6a** pour la créance et l'arrondi (lus sur la vente) ; TVA due lue dans les réglages courants : **angle mort, #525** ; produit de repli : **angle mort, #525** (commentaire de l'orchestrateur du 2026-10-08) (§ *Le défaut*) |
| validation de facture (`invoices.rs:2255`) | — (**crée** la créance) | produit `Revenue`, arrondi | créance des réglages sans type contrôlé : **angle mort, #537** (§ *Le défaut*) |
| validation d'achat (`supplier_invoices.rs:372`) | — (**crée** la dette) | charge | hors classe (aucun compte soldé) ; même forme possible que #537 côté dette (2000 retypé en charge) — à confronter en T8 à la garde à l'usage de la 15-5d, tracé par **#537** (commentaire du 2026-10-08, « Même forme côté dette ») |
| `opening_complement.rs:641` (soldes de départ) | — | lignes saisies, `enforce_postable = true` | hors classe : chaque ligne est choisie par l'utilisateur, aucune créance n'est soldée par le geste |
| saisie manuelle d'écriture (non rendue par les commandes : elle passe par `journal_entries::create`, ajoutée par raisonnement) | — | lignes libres | hors classe : l'utilisateur écrit chaque ligne |

Écrans rendus par les commandes frontend :

| écran | verdict |
|---|---|
| `SettleInvoiceDialog.svelte:105`, `supplier-invoices/[id]/+page.svelte:84` | **AC8** |
| `payment-batches/+page.svelte:53` (création de lot, comptes bancaires) | non filtré, **choix écrit** (C-15-6-19) : refus par facture lisible ; § *Angles morts* |
| `settings/invoicing/+page.svelte:65-77` (réglages) | **15-6c** (désignation) |
| `supplier-invoices/+page.svelte:129`, `supplier-invoices/import/+page.svelte:83` (charges d'une facture fournisseur) | hors classe : aucun compte soldé n'est choisi |
| `bank-accounts/+page.svelte`, `BankAccountJournalLinkForm.svelte` (lien des comptes bancaires) | **15-6c** |
| `ReconciliationProposals.svelte` / `ManualMatchModal.svelte` | **15-6d** |
| `routes/(app)/+page.svelte:58` (tuile des comptes bancaires du tableau de bord, rendue par `listBankAccounts(`) | hors classe : affichage seul, aucun règlement (finding F6-2 de la P6) |
| `features/accounts/account-validity.ts:20` (rendu par `active && a.postable`) | hors classe : doc-comment, aucun menu (finding F6-2 de la P6) |

## Décompte des modules — signal de découpage (D5)

Finding F5 de la P2 : la P1 a ajouté les lots sans recompter. Recompté le 2026-10-08 (et de nouveau à
la P5, finding R5-5), **au barème de la règle** (« modules métier de premier niveau ») : `kesh-db`
règlement client (`invoice_settlements_write.rs`, solde du reste compris), `kesh-db` helper commun
(`invoice_settlements.rs` : trois fonctions publiques et les types `ClaimAccountClash`,
`ClaimSubject`, `DesignatedRole` — les deux premiers omis jusqu'à la P4, le troisième jusqu'à la P6), `kesh-db` règlement fournisseur
(`supplier_invoices.rs`), `kesh-db` lots (`payment_batches.rs`), `kesh-api` rapprochement
(`routes/reconciliation.rs`), `frontend` factures (`features/invoices`), `frontend` factures
fournisseurs (`routes/supplier-invoices`), `frontend` lots (`features/payment-batches`), `frontend`
comptes (`features/accounts/account-options.ts`) — **neuf** (et non huit : le helper commun manquait
au décompte ; les clés plates de la P5 n'ajoutent pas `kesh-i18n/src/loader.rs`), plus la plomberie
d'erreur et d'i18n
(`kesh-db/errors`, `kesh-api/errors`, `kesh-i18n`) et, si la 15-5c est mergée, une ligne dans
`features/reconciliation`. **Au barème du geste** (patron de la 15-5a : l'écran suit le geste qu'il
sert) : règlement client, règlement fournisseur, lots, rapprochement, fonctions d'écran partagées —
**cinq**. **Signal déclaré au Project Lead** ; **pas de découpage** (choix C-15-6-20) : les gestes ne
sont pas indépendants — une variante, un `ClaimSide`, un lecteur sœur partagé par `pay_in_tx` et les
lots, trois fonctions d'écran —, et les lots ne font qu'appeler la garde de `pay_in_tx` (AC4) ; les
séparer laisserait une story sans sa garde ou dupliquerait la plomberie. Aucun défaut de la P2 ne
recycle un défaut de la P1.

**P3 (signal D5 levé de nouveau, déclaré au Project Lead)** : sévérité P2 → P3 MEDIUM → MEDIUM. Deux
des cinq MEDIUM distincts (R1 = F1 : remède faux du refus de l'AC3 bis ; R2 = F2 : table de refus du
solde oubliée) **naissent de la remédiation P2** — l'AC3 bis ajoutée sans son message ni sa
documentation : c'est le motif « recyclage » de l'amendement D5, mais **contenu** — une seule règle
(AC3 bis), un seul module de plus (aucun : le dialogue de solde est lu, pas modifié), corrigé ici par
un discriminant et une clé. Le troisième (F3, DRY) est d'origine. **Pas de découpage** (choix
C-15-6-26) ; la remédiation P3 change une règle métier (le message suit le rôle) : la passe suivante
doit être **complète**, pas ciblée.

**P4 (signal D5 levé une troisième fois, déclaré au Project Lead)** : MEDIUM → MEDIUM ; F1 **recycle**
le discriminant posé en P3 (compte d'arrondi du solde mal classé), contenu à une ligne de classement et
un test ; F2 et F3 sont d'origine. Aucun module de plus (F2 traité en angle mort, #537 et #525). **Pas
de découpage** (C-15-6-30) ; un troisième recyclage sur l'AC3 bis en P5 déciderait de la sortir en
story propre.

**P5 (signal D5 levé par la sévérité, déclaré au Project Lead)** : MEDIUM → MEDIUM (3 et 3, R + F ;
F2 = R5-3). **Le critère « troisième recyclage sur l'AC3 bis » n'est pas atteint** : le seul MEDIUM qui
touche l'AC3 bis (F1, son ordre des refus) **ne vient pas d'un patch** — il naît d'un **changement
extérieur**, la création de la 15-5e (C52), qui réordonnait alors `settle_invoice` et
`write_off_invoice`, après la P4 ; sa réécriture par C54 (pendant la remédiation) a retiré ce
réordonnancement, et l'AC3 bis garde l'ordre des lectures du code. Les deux autres recyclent des patches de passes antérieures
**hors** de l'AC3 bis : R5-1, le sélecteur Fluent de l'AC2 (posé en P3, réécrit en P4), remplacé par
trois clés plates ; F2 = R5-3, le critère de T2 réécrit en P4, remplacé par un critère par nom sur liste
fermée. F3 (manuel `:1754`) est d'origine. Aucun module de plus (le décompte passe de huit à neuf par
une omission corrigée, pas par un ajout). **Pas de découpage** (C-15-6-33).

**P6 (clôture — signal de dispersion déclaré, non découpé)** : 0 au-dessus de LOW ; le signal de
**sévérité** n'est pas levé. Le finding F6-7 constate en revanche que le **critère de dispersion** de
l'amendement D5 (« la dispersion dans plus de cinq modules ») est **rempli au barème de la règle** :
**neuf** modules métier de premier niveau, plus la plomberie d'erreur et d'i18n, et `admin-manual.tex`
(documentation, hors décompte) ajouté par F6-1. Le chiffre de **cinq** ne vaut qu'au « barème du
geste », que la règle ne prévoit pas : il n'est **plus** invoqué comme justification. **Déclaré au
Project Lead, non découpé** (choix C-15-6-34), pour une raison écrite qui ne dépend pas du barème : la
fiche est **validée** (six passes, sévérité retombée à 0 au-dessus de LOW), ses gestes partagent une
variante, un sujet typé, un lecteur sœur et trois fonctions d'écran, et un découpage à ce stade
rouvrirait la validation de deux fiches pour séparer ce que la conception lie — le risque que la règle
de découpage veut prévenir (non-convergence) ne s'est pas réalisé. L'arbitrage revient à Guy en revue
de fin d'epic.

## Tests

Backend, `#[sqlx::test(migrations = "./test-schema")]` :

1. `crates/kesh-db/tests/invoice_settlement.rs` — **compte interne = créance** : refus
   `SettlementCounterpartyIsClaimAccount { claim: Receivable, role: Counterparty, account_number: Some("1100"), batch: None, .. }` ;
   aucune écriture neuve, aucune ligne `invoice_settlements`, `version` et `paid_at` inchangés.
2. Même fichier — **virement, compte bancaire lié au 1100** (posé par `UPDATE bank_accounts SET
   journal_account_id = …` : le test ne doit pas dépendre de la 15-6c) : même refus.
3. Même fichier — un règlement ordinaire (banque 1020, puis compte interne caisse) reste accepté
   (non-régression de l'ordre et du montant). Le seed `seed_accounting_company` n'a pas de 1020
   (`test_fixtures.rs:129-133` : 1000, 1100, 2000, 3000, 4000) : le créer comme
   `invoice_settlement.rs:182-185`.
4. Même fichier — **montant excessif sur 1100** : le refus est celui de la contrepartie, **pas** le
   trop-perçu (ordre AC3 ; le test 1 n'a pas de montant excessif et ne le fige pas).
5. `crates/kesh-db/tests/supplier_invoices_repository.rs` — compte interne = 2000 : `claim: Payable`,
   `role: Counterparty`,
   facture toujours `open`, aucune écriture.
6. Même fichier — virement sur un compte bancaire lié au 2000 : même refus. ⚠️ La fixture existante
   (`:215`) lie un compte bancaire au **1100** pour payer des factures fournisseurs : elle reste
   valide (le compte soldé est 2000) ; ne pas la changer.
7. `crates/kesh-api/tests/reconciliation_e2e.rs` — **un lot de deux propositions, deux créances
   différentes** : la facture A validée avec la créance sur 1100 ; puis un compte d'actif `1101` créé
   et désigné comme débiteurs ; la facture B validée (créance sur 1101) ; le compte bancaire du lot lié
   au 1100 (SQL). `accept_batch` (un seul `bank_account_id` pour tout le lot, `:1013`) : HTTP 200, B
   acceptée, A dans `failed[]` avec le code et les **cinq** clés de `details` (`bankAccountId`,
   `accountId`, `accountNumber`, `claim`, `role = "counterparty"`) ; la transaction bancaire
   de A reste en attente.
8. Même fichier — proposition sur A **d'un montant supérieur au reste** : `failed[]` porte
   `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, pas `RECONCILIATION_OVERPAYMENT` (ordre AC5). La
   transaction bancaire porte la **référence de la facture** (patron du test existant de
   `RECONCILIATION_OVERPAYMENT`, `two_centimes_over_a_half_centime_invoice_is_an_overpayment`,
   `reconciliation_e2e.rs:4869`), pour que son **score reste positif** : l'étape 7bis
   (`RECONCILIATION_SCORE_TOO_LOW` si `score.total <= 0.0`, `reconciliation.rs:1344-1354`) passe
   **avant** les étapes (a) à (c), et un score nul rendrait ce refus-là avant comme après la story,
   sans rien figer de l'ordre AC5 (finding F6-4 de la P6).
9. `crates/kesh-db/tests/invoice_settlement.rs` — **ordre** : compte interne 1100 **archivé** →
   `INACTIVE_OR_INVALID_ACCOUNTS` (l'ancien refus prime).
10. `crates/kesh-db/tests/invoice_write_off.rs` — **solde du reste, nature = créance** (AC3 bis) :
    après validation, 1100 change de type par SQL (`account_type = 'Expense'`, `role = NULL` — le
    chemin `confirm_retype` sans la route) et est désigné compte d'escompte en SQL ; solder le reste en
    escompte → `SettlementCounterpartyIsClaimAccount { claim: Receivable, role: WriteOffNature, .. }`,
    rien d'écrit.
10 bis. Même fichier — **solde du reste, reste d'arrondi = créance** (findings R1 = F1 de la P4) :
    même changement de type, 1100 désigné compte **d'arrondi** (et non d'escompte, qui reste un compte
    de charge distinct) ; solder en **escompte** un reste à quatre décimales (montage de
    `a_four_decimal_remainder_sends_its_sub_centime_to_the_rounding_account`, `:898`) →
    `SettlementCounterpartyIsClaimAccount { role: Rounding, .. }` — **pas** `WriteOffNature` : le rôle
    suit la colonne de réglage lue, pas la nature du geste ; rien d'écrit.
10 ter. Même fichier — **ordre des refus du solde** (AC3 bis, finding F6 de la P5) : une facture
    **avec TVA** validée d'abord (la validation exige le réglage de TVA due, `test_fixtures.rs:150-151`) ;
    **après validation** — finding R6-9 de la P6 —, même changement de type, 1100 désigné compte
    d'**escompte**, **et** réglage de TVA due vidé (`default_vat_payable_account_id = NULL`) ; solder en
    escompte → `SettlementCounterpartyIsClaimAccount { role: WriteOffNature, .. }`, **pas**
    `ConfigurationRequired("default_vat_payable_account_id")` : la nature est comparée aussitôt la
    créance lue, avant la lecture de la TVA due ; rien d'écrit.
11. `crates/kesh-db/tests/invoice_settlement.rs` (finding R5-7 de la P5 : le fichier du règlement, non
    celui du solde) — **arrondi du règlement = créance** (AC3 bis). Ce fichier n'a pas de montage
    d'arrondi : il reprend celui de `crates/kesh-db/tests/invoice_amount_due_parity.rs`
    (`seeded_with_rounding`, `:72`, qui désigne le compte d'arrondi par `designate_rounding_account`)
    et une facture au **reste brut hors centime** (patron `raw_due_invoice`,
    `crates/kesh-api/tests/invoice_echeancier_e2e.rs:986`) —
    **arrondi du règlement = créance**
    (AC3 bis) : même changement de type, 1100 désigné compte d'arrondi ; un paiement au centime de
    cette facture (`SettlesWithRounding`) par **compte interne** (caisse 1000) → même refus,
    `role: Rounding`, rien d'écrit (le compte interne, valide, est contrôlé avant, AC3 bis).
12. `crates/kesh-api/tests/reconciliation_e2e.rs` — **arrondi (c-bis) = créance** (AC3 bis) : patron
    `half_centime_invoice_and_tx` (`:4705`) et `designate_rounding_account` (`:4681`), le compte
    d'arrondi désigné étant la créance retypée → `failed[]` `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`,
    HTTP 200, `details.role = "rounding"` et **pas** de `details.bankAccountId` (AC3 bis).
13. `crates/kesh-db/tests/payment_batches_repository.rs` — **création** : deux factures fournisseurs,
    A achetée avec la dette sur 2000, B après changement du réglage créanciers vers un `2001` ; compte
    bancaire source lié au 2000 : A dans `failed[]` avec le code et ses `details`, B dans le lot ; avec
    A seule, **aucun** lot créé. `details` = les cinq clés de l'AC6 (`role = "counterparty"`), et
    `accountNumber = "2000"` (lu à l'échec par le helper, AC6).
14. Même fichier — **création, écriture d'achat sans ligne de crédit** (en-tête d'écriture sans
    ligne, désigné par `UPDATE supplier_invoices SET purchase_journal_entry_id = …`) : la facture dans
    `failed[]` avec `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED` et ses `details`, les autres du lot
    retenues.
15. Même fichier — **confirmation, puis remède** : lot créé sur un compte bancaire lié au 1020 (créé
    comme au test 3), puis relié au 2000 en SQL ; `confirm_batch` → `SettlementCounterpartyIsClaimAccount
    { claim: Payable, role: Counterparty, batch: Some(..), .. }` portant l'id du lot, de la facture
    **et son numéro** (`supplier_invoice_number`, relu par `confirm_batch`, AC6), lot toujours
    `generated`, aucun règlement écrit ; puis relié de nouveau au 1020 → la confirmation **passe**.
    (La fixture `:100` lie au 1100 : inchangée, même raison qu'au test 6.)
16. Test HTTP `POST /api/v1/invoices/{id}/settlements` (fichier existant qui l'appelle,
    `crates/kesh-api/tests/invoice_echeancier_e2e.rs`) : 400, code, `details.claim = "receivable"`,
    message **fr**. ⚠️ Kesh n'a **pas** de négociation par `Accept-Language` : la langue des messages
    est **globale au processus** (`init_error_i18n`, `crates/kesh-api/src/errors.rs:23-26`), comme le
    dit `docs/api-external.md` § 10. Le test de parité de `kesh-i18n` (`parity_between_locales`,
    `crates/kesh-i18n/src/loader.rs:719`) ne contrôle que la **présence** des clés dans les quatre
    locales, **pas leur traduction** — une copie du français passerait : la traduction se relit en
    revue.
16 bis. Même fichier — test HTTP `POST /api/v1/invoices/{id}/write-off` (le fichier l'appelle déjà) :
    compte de la nature `discount` = la créance retypée (montage du test 10) → **400**,
    `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT`, `details.role = "write_off_nature"`, message **fr**
    contenant « nature de solde » et « Paramètres → Facturation » et **ne** contenant **pas** « banque,
    caisse » — il fige la clé plate `error-settlement-write-off-account-is-receivable` de l'AC2
    (findings R1 = F1 de la P3, R5-1 de la P5), que le test 16 ne voit pas.

Frontend (Vitest) :

17. `frontend/src/lib/features/accounts/account-options.test.ts` (existant) : les trois fonctions — le
    rôle donné écarté, les comptes bancaires liés écartés, les autres gardés, les comptes sans rôle
    intacts ; un compte bancaire lié à un compte de rôle `Receivable` **non imputable** est écarté (ids
    calculés avant le filtre `active && postable`).
18. `SettleInvoiceDialog.test.ts` : 1100 (rôle `Receivable`) absent du menu « Compte interne » ; un
    compte bancaire lié au 1100 absent du menu des virements, la présélection tombe sur un autre ;
    `accounts` fourni **après** l'ouverture et une saisie de montant : le montant saisi est **gardé**
    (AC8, effet distinct) ; seul
    compte bancaire lié au 1100 → le message de liste vide ; **aucun** compte bancaire → **pas** de
    message de liste vide (rien n'a été écarté, AC8 — finding F3 de la P4) ; la prop `errorMsg`
    s'affiche (AC9 — rendu seulement).
19. `supplier-invoices/[id]/supplier-settlement-page.test.ts` : 2000 absent, banque liée au 2000
    absente, liste vide écrite ; aucun compte bancaire lié → pas de message de liste vide.
20. `frontend/src/lib/features/payment-batches/payment-batch-helpers.test.ts` : les libellés des deux
    codes neufs (AC6), le test « mappe les six codes » devenu « huit ».

Soit **23 tests nommés** (19 backend — 1 à 16, 10 bis, 10 ter et 16 bis —, 4 fichiers Vitest étendus) : tous neufs,
sauf le test existant « mappe les six codes » de `payment-batch-helpers.test.ts:61`, **renommé et
étendu** à huit codes (test 20 ; ses six assertions existantes restent) — à recompter aux deux bornes
avant de les déclarer. Si la 15-5c est mergée, `failed-proposal-label.test.ts` reçoit en plus les
deux cas de l'AC7 (non compté : conditionnel).

**E2E existants — verdict de lecture** (finding F7 de la P3 ; à confronter au run de T9) :
- `frontend/tests/e2e/bank-account-journal-link.spec.ts:105-119` lie le premier compte bancaire
  (TEST_IBAN) au **1100**, compte débiteurs du préréglage (`test_fixtures.rs:129-147`). La 15-6b ne
  touche pas la liaison (c'est la 15-6c, C-15-6-15) : la spec reste verte ici. ⚠️ **Le filtre par
  rôle de l'AC8 est inerte en E2E** (finding R4 de la P4) : le seed (`seed_accounting_company`,
  appelé par `test_endpoints.rs:184`, `:197`) insère les comptes **sans rôle**
  (`test_fixtures.rs:129-139`, `INSERT INTO accounts (company_id, number, name, account_type)` ;
  aucun `SET role` ni `'Receivable'` dans le seed) : `accountIdsWithRole(…, 'Receivable')` y rend un
  ensemble vide, rien ne disparaît des menus, rien ne se décale. Seul le refus **serveur** (qui
  compare la créance lue sur la vente, sans rôle) s'appliquerait si une spec réglait une facture
  client par le compte bancaire lié au 1100 — aucune ne le fait.
- Les specs qui en dépendent n'en souffrent pas, par lecture : `invoices_echeancier.spec.ts:143-146`
  règle par **compte interne**, `selectOption({ index: 1 })` — le menu n'est pas filtré en E2E (seed
  sans rôle), l'index 1 reste le même compte qu'avant la story ; `invoices.spec.ts:318-321` fait de
  même (même menu, même index, même raison — finding R5-6 de la P5) ; `reminders.spec.ts:104-112` règle
  par API sur un compte d'actif `startsWith('10')` (liquidités, jamais 1100) ;
  `invoice-write-off.spec.ts:70-85` — **seule** spec qui traverse le chemin de l'AC3 bis — désigne
  l'escompte sur 4000 et règle par API sur 1000 : nature 4000, TVA due 2000 du seed
  (`test_fixtures.rs:164-166`), aucune n'est la créance 1100 ; `invoices_echeancier.spec.ts:227-228` et
  `invoices-settlement-cancel.spec.ts:103-104` règlent par API sur `cash` (1000) ;
  `reconciliation-cancel.spec.ts:60-83` relie le compte bancaire au 1000 avant d'accepter ;
  `reconciliation-amount-due.spec.ts` n'accepte rien ; `reconciliation.spec.ts` ne joue pas
  l'acceptation ; `payment-batches.spec.ts:51-81` et `supplier-invoices.spec.ts` paient des dettes
  du **2000** avec un compte bancaire lié à un compte liquide, ou un compte interne de l'actif
  (`supplier-invoices.spec.ts:77`, premier compte `Asset` actif — 1000, pas 2000).
- **Aucune rupture attendue** — par l'inertie du filtre en E2E, non par un décalage sans effet ; aucune
  spec n'est modifiée par cette story, et **aucun E2E n'exerce l'AC8** (angle mort écrit, § *Angles
  morts*). *(Le test « un compte d'actif désigné comme compte d'arrondi est refusé » de la P1 est
**retiré** : il figeait la garantie du « résolu par le type », que l'AC3 bis remplace.)*

## Dev Notes

### Ce qui doit être préservé

- L'**ordre** des refus existants (AC3–AC6) — la 15-5e n'en change aucun (son AC7) — et le **verrou** `FOR UPDATE`
  posé sur le compte bancaire ou le compte interne à la résolution de la contrepartie — la garde neuve compare des identifiants déjà lus, elle ne
  relit rien sauf le numéro du compte (à l'échec) et, à la confirmation d'un lot, le numéro de la
  facture fournisseur (`supplier_invoice_number`, AC6).
- Le **refus du trop-perçu** (du règlement ; étape (c) du rapprochement) vient **après** : un
  montant excessif sur 1100 rend le refus de contrepartie, pas `overpayment` — choix délibéré (la
  contrepartie est le défaut de fond) ; les tests 4 et 8 le figent.
- `enforce_postable = false` de ces flux : inchangé.
- Le pattern batch des lots (`PaymentBatchFailedItem`) : une facture refusée n'empêche pas les autres.

### Pièges

- **Ne pas** réutiliser `DbError::InvalidInput` : code générique `INVALID_INPUT`, sans `details`
  (choix C-15-6-3). **Ne pas** réutiliser `ACCOUNT_NOT_POSTABLE` (15-5a) : 1100 est imputable.
- La 15-5a insère `ACCOUNT_NOT_POSTABLE` à la lecture du compte interne de `settle_invoice` et de `pay_in_tx` ; la 15-5b
  pose dans `accept_one_invoice` un commentaire de classement que l'AC5 amende ; la **15-5c** pose les
  libellés de `failed[]` (#492) et réécrit le manuel du rapprochement — le rebase peut demander de
  fusionner la liste de codes : c'est l'AC7. T0 refait les numéros de ligne.
- `kesh-i18n` : un test vérifie la **présence** des clés dans les quatre locales (pas leur
  traduction) ; un autre (`no_new_select_expression_reaches_the_frontend_dictionary`, `loader.rs:482`)
  refuse tout **sélecteur** Fluent neuf — d'où les clés plates (AC2). Ajouter les clés neuves aux
  quatre fichiers dans le même commit — **six** serveur (AC2 : trois par côté et contexte, trois pour
  un compte désigné dans les réglages, une par rôle), **quatre** écran
  (deux de lots, AC6 ; deux de liste vide, AC8), **deux** de plus si la 15-5c est mergée (libellé
  `failed[]` du rapprochement à deux cas, AC7).
- **Un code, deux remèdes** (C-15-6-26) : ne pas créer un second code pour le compte désigné — le
  défaut est le même (`D X / C X`), un intégrateur le traite d'un seul bras ; c'est `details.role`
  qui porte la différence, et le message qui la dit à l'utilisateur.
- **Recompter** les tests ajoutés aux deux bornes avant de les déclarer (§ *Recompter ses propres
  comptes rendus*).

### Angles morts assumés

- Le filtre d'écran est par **rôle**, la garde serveur par **compte de l'écriture de vente** : si le
  réglage débiteurs a été déplacé hors du compte de rôle `Receivable`, l'écran peut proposer un compte
  que le serveur refusera (avec un message clair) — jamais l'inverse (écriture fausse). Choix C-15-6-4.
- **Comptes archivés** : absents des listes que reçoivent les écrans ; un compte bancaire lié à un
  compte archivé reste proposé, et le serveur refuse toute écriture sur un compte archivé (C-15-6-17).
- **Écran de création d'un lot** (`payment-batches/+page.svelte:53`) : il propose un compte bancaire
  lié au compte créanciers ; le refus arrive par facture, libellé, avant tout fichier (AC6). Le
  filtrer demanderait de charger le plan comptable sur cet écran (C-15-6-19).
- **Validation d'une facture et avoir** (finding F2 de la P4) : créance retypée lue à la validation
  (**#537**), TVA due de l'avoir lue dans les réglages courants (**#525**), produit de repli de
  l'avoir de même (**#525** aussi, commentaire de l'orchestrateur du 2026-10-08) ; validation d'achat,
  même forme côté dette (**#537**, commentaire du 2026-10-08) — § *Le défaut*, inventaire. Hors classe
  de la story (aucun ne solde un compte), et deux modules de plus.
- **Le filtre d'écran n'est exercé par aucun E2E** : le seed E2E n'a pas de rôle (§ *Tests*, « E2E
  existants ») ; l'AC8 n'est figé que par Vitest (tests 17-19).
- **Données antérieures** : des règlements `D 1100 / C 1100` ou `D 2000 / C 2000` déjà écrits par une
  instance v0.12 restent tels quels, sans détection. Acceptable : le produit ne tient pas encore de
  comptabilité réelle (`CLAUDE.md` § *Project Overview*) ; à réexaminer si une instance en a écrit.

### Références

- [Source: crates/kesh-db/src/repositories/invoice_settlements_write.rs:43-200, :356-540]
- [Source: crates/kesh-db/src/repositories/accounts.rs:497-515 (changement de type confirmé)]
- [Source: crates/kesh-db/src/repositories/supplier_invoices.rs:554-700]
- [Source: crates/kesh-db/src/repositories/payment_batches.rs:68-140, :226-300, :304-380, :431]
- [Source: crates/kesh-api/src/routes/reconciliation.rs:1013, :1175, :1378-1500]
- [Source: crates/kesh-db/src/errors.rs:515-532, :742-790 ; crates/kesh-api/src/errors.rs:23-26, :2823-2846]
- [Source: crates/kesh-db/src/repositories/company_invoice_settings.rs:397-447, :454-477 (TVA due, sans type), :484-507]
- [Source: frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:89, :105, :208 ;
  frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:80-84 ;
  frontend/src/routes/(app)/invoices/[id]/+page.svelte:509, :554, :582-585, :1577 ; frontend/src/routes/(app)/invoices/due-dates/+page.svelte:102, :265-268 ;
  frontend/src/lib/features/payment-batches/payment-batch-helpers.ts:78 ;
  frontend/src/routes/(app)/payment-batches/[id]/+page.svelte:40-50 ; frontend/scripts/lint-i18n-ownership.js:164-187]
- [Source: docs/manual/fr/user-manual.tex:1045-1052, :1064-1072, :1295-1305, :1346-1360, :1525 ;
  docs/api-external.md:253-283, :285-287, :307, :397-415]
- [Source: frontend/tests/e2e/bank-account-journal-link.spec.ts:105-119, invoices_echeancier.spec.ts:143-146 ;
  crates/kesh-db/src/test_fixtures.rs:129-139 (seed sans rôle) ; crates/kesh-api/src/lib.rs:207-210 (réglages, Admin)]
- [Source: crates/kesh-db/src/repositories/company_invoice_settings.rs:408, :438-441 (même colonne d'arrondi) ;
  crates/kesh-db/src/repositories/invoices.rs:2008-2010 ; crates/kesh-db/src/repositories/credit_notes.rs:205, :511 ;
  frontend/src/routes/(app)/settings/invoicing/+page.svelte:29 ; issues #525, #537 ; C36 (15-5d), C-15-6-30]
- [Source: issue #474 ; `epic-15-choix-autonomes.md` C-15-6-1, C-15-6-3, C-15-6-4, C-15-6-9 à C-15-6-12, C-15-6-14, C-15-6-17 à C-15-6-20, C-15-6-26, C-15-6-30, C-15-6-33, C-15-6-34 ; C36, C52, C54, C55 au registre de la branche des 15-5 (arrivent au rebase, T0)]
- [Source: docs/manual/fr/admin-manual.tex:2027, :2029, :2033 (AC11, verdict) ; frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:82-91 (effet de réinitialisation) ; crates/kesh-api/src/errors.rs:52-59, :88-92 (repli `t_args` formaté)]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie). Statut `ready-for-dev`.
- 2026-10-08 — **Validation P1** (Sonnet 4.6, lentilles F et R ; prompt
  `15-6b-validate-prompt-p1.md`). F : 2 HIGH, 5 MEDIUM, 2 LOW ; R : 2 HIGH, 5 MEDIUM, 2 LOW (R3 = F1,
  R4 = F2, R5 = F3). Appliqués :
  - **HIGH F1 (= R3 MEDIUM)** — #524 : sorti vers une sous-story **15-6d** (le finding F4 de découpage
    la rendait nécessaire), avec le même refus que le flux ventilé ; C-15-6-6 révisé par C-15-6-9 ;
    l'angle mort « voisin hors périmètre » est retiré de cette fiche.
  - **HIGH F2 (= R4 MEDIUM)** — `confirm_batch`, troisième appelant de `pay_in_tx` : refus **à la
    création du lot**, par facture, avant tout fichier pain.001, et garde maintenue à la confirmation
    (AC6, tests 11–12, manuel, api-external). Choix C-15-6-10.
  - **HIGH R1** — test 5 irréalisable (un seul compte bancaire par lot) : refait avec deux factures à
    créances différentes (test 7).
  - **HIGH R2** — test « fr et de par `Accept-Language` » irréalisable (locale globale,
    `init_error_i18n`) : test fr + parité des clés (test 13).
  - **MEDIUM F3 = R5** — #492 est en **15-5c**, pas en 15-5b : dépendance, AC7, pièges corrigés ;
    registre révisé (C-15-6-12).
  - **MEDIUM F4** — découpage (plus de cinq modules avec #524 et les lots) : 15-6d créée.
  - **MEDIUM F5** — ordre vis-à-vis de `BANK_ACCOUNT_NOT_CONFIGURED`, `INVOICE_SALE_ENTRY_MALFORMED`
    et `RECONCILIATION_OVERPAYMENT` écrit (AC5) et testé (test 8).
  - **MEDIUM F6** — `docs/api-external.md` (AC10).
  - **MEDIUM F7** — liste de comptes bancaires vide écrite à l'écran (AC8, tests 15–16).
  - **MEDIUM R6** — « le test 1 fige le trop-perçu » était faux : test 4 distinct.
  - **MEDIUM R7** — DRY : la garde client compare l'id du lecteur de la 15-6a ; côté fournisseur, la
    requête diffère (crédit + TTC) et devient un lecteur sœur partagé par `pay_in_tx` et la création
    de lot (C-15-6-11).
  - **LOW F8** — propriétaire unique de la section `[0.13.0]` (AC12) ; lignes de la fiche fournisseur
    corrigées (`:82-83`). **LOW F9** — T7 « grep de formes » remplacé par un inventaire fermé des
    sites (§ *Inventaire*, T8). **LOW R8** — décomptes refaits. **LOW R9** — ids calculés sur la liste
    complète (AC8, test 14) ; route du règlement client nommée ; AC9 « vérifié, aucun changement » +
    test ; lignes de la fiche fournisseur.
  - Signature des fonctions d'écran fixée pour être partagée avec la 15-6c (C-15-6-14, finding F2 de
    la 15-6c).
  - Décompte après passe : **12 AC, 9 tâches, 17 tests** (13 backend, 4 fichiers Vitest).
- 2026-10-08 — **Validation P2** (Opus 5.5, lentilles R et F ; prompt `15-6b-validate-prompt-p2.md`).
  R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 9 LOW ; F : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 10 LOW (R-1 = F1,
  R-2.1 = F9, R-6 = F7, R-7 = F10, R-9 ⊃ F12, R-10 = F13, R-11 = F8). Tous appliqués, décisions de
  l'orchestrateur comprises :
  - **MEDIUM R-1 = F1** — les écrans ne reçoivent pas les archivés : l'AC8 y **renonce** (un compte
    archivé ne peut recevoir aucune écriture, la garde serveur suffit) ; les ids se calculent avant le
    filtre `active && postable` (cas non imputable, test 17). Choix C-15-6-17.
  - **MEDIUM F2** — clés renommées pour `lint-i18n-ownership` (`invoices-settle-…`), sans allonger
    `KNOWN_VIOLATIONS` ; clé fournisseur alignée (`supplier-invoices-pay-…`). Choix C-15-6-17.
  - **MEDIUM F3** — « résolu par le type » ne tient pas (changement de type confirmé) : les comptes
    d'arrondi et de solde du reste sont **comparés par identifiant** à la créance (AC3 bis, tests
    10–12). Choix C-15-6-18, qui révise C-15-6-3.
  - **MEDIUM F4** — lot bloqué à la confirmation : message propre, `details` (lot, facture) et manuel
    disent les deux issues (relier de nouveau, ou annuler le lot) ; test 15 vérifie le remède. Le refus
    à la création reste la défense principale. Choix C-15-6-19.
  - **MEDIUM F5** — modules recomptés et écrits (§ *Décompte des modules*) : huit au barème des
    fichiers, cinq au barème du geste ; **signal D5 déclaré au Project Lead**, pas de découpage
    (gestes dépendants). Choix C-15-6-20.
  - **MEDIUM R-2** — `ACCOUNT_NOT_POSTABLE` (15-5a) dans l'ordre des refus (AC3, AC4) ; tâche T0 de
    rebase ; commentaire de la 15-5b dans `accept_one_invoice` amendé (AC5).
  - **MEDIUM R-3** — le filtre de `ManualMatchModal` relève de la 15-6d (en-tête, inventaire), qui le
    tranche sans conditionnel.
  - **LOW** — R-4 : `:83-84` ; R-5 : `:120`, `usable_designated_account` `:484-507` ; R-6 = F7 :
    fichier de la confirmation et « vérifié, aucun changement » ; R-7 = F10 : signature de
    `validate_invoice_for_batch`, écriture d'achat malformée → `failed[]` (test 14) ; R-8 : six → huit
    clés ; R-9 = F12 : décompte des clés, parité = présence ; R-10 = F13 : test du dialogue = rendu de
    la prop, dit ; R-11 = F8 : inventaire complété par ses propres commandes (backend et écrans) ;
    R-12 : 1020 créé, tests verts d'avance exclus du « rougit d'abord » ; F6 : écran de création de
    lot non filtré, choix écrit ; F11 : « contrepartie libre » corrigé au manuel ; F14 : lots dans la
    table du § 10 seulement, routes nommées ; F15 : données antérieures en angle mort.
  - **Tests renumérotés** (10–12 neufs pour l'AC3 bis, 14 neuf, l'ancien 10 retiré) : les numéros
    cités dans l'entrée P1 ci-dessus visent l'**ancienne** numérotation.
  - Décompte après passe : **13 AC (AC1–AC12 et AC3 bis), 10 tâches (T0–T9), 20 tests** (16 backend,
    4 fichiers Vitest).
- 2026-10-08 — **Validation P3** (lentilles R et F en **Sonnet** — prompt `15-6b-validate-prompt-p3.md` ;
  remédiation par Opus 5.5). R : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW ; F : 0 CRITICAL, 0 HIGH,
  3 MEDIUM, 4 LOW (F8 noté par la lentille, non compté comme défaut). R1 = F1, R2 = F2. Tous appliqués,
  décisions de l'orchestrateur comprises (choix C-15-6-26) :
  - **MEDIUM R1 = F1** — un compte venu des **réglages** (arrondi, nature, TVA due) n'appelle pas le
    remède d'une contrepartie choisie : discriminant `role` (`SettlementAccountRole`) dans la variante
    et en `details.role` ; quatrième clé serveur `error-settlement-designated-account-is-receivable`
    qui renvoie à *Paramètres → Facturation* (sélecteur Fluent sur `$role`) ; libellé `failed[]` du
    rapprochement à deux cas (AC7) ; manuel à deux cas (AC11) ; `details` de (c-bis) fixés sans
    `bankAccountId` ; dialogue de solde (`invoices/[id]/+page.svelte:509`) couvert par l'AC9 ; test 16
    bis (HTTP du solde) neuf. Un seul code conservé.
  - **MEDIUM R2 = F2** — `docs/api-external.md` : ligne à la table du solde (`:274-283`) ; une phrase
    « Refus » sous le § des règlements client (`:253`) et sous celui du règlement fournisseur (`:285`,
    lots compris) ; le § 10, générique, n'est plus visé (AC10).
  - **MEDIUM F3** — helper commun dans `invoice_settlements` : **comparaison** pure
    (`ensure_not_claim_account`, empruntée par la 15-6d) séparée de la **construction** du refus
    (`claim_account_refusal`, numéro lu à l'échec seulement) et des `details`
    (`claim_account_refusal_details`, trois consommateurs) ; T2 l'ordonne en premier.
  - **LOW** — F4 : `vat_payable_account_for_write` ne contrôle aucun type (`company_invoice_settings.rs:454-480`),
    la comparaison est sa seule garde (§ *Le défaut*, AC3 bis) ; F5 : `supplier_invoice_number` relu par
    `confirm_batch` (une `SELECT`), pas de champ de plus ; R7 : `accountNumber` de la création de lot lu
    à l'échec par le helper ; F6 : textes fr des quatre clés d'écran (et des deux libellés de l'AC7) ;
    F7 : verdict écrit sur les E2E existants, `bank-account-journal-link.spec` compris (§ *Tests*, T9) ;
    R3 : `:432` → `:433`, `:110` → `:114`, `:247-249` → `:251-253` ; R4 : « six » → « huit » aux quatre
    sites (jeton grepé) ; R5 : patron `CREDIT_NOTE_INVOICE_SETTLED` borné à la forme de `details`
    (409, `t(`) ; R6 : relève du registre — porté par C-15-6-26, qui dit C-15-6-3 révisé (pas de
    réécriture d'entrée).
  - Propagation : `grep` des jetons `six`, `trois** clés`, `:432`, `:247-249`, `lecture neuve`,
    « patron de CREDIT_NOTE » sur la fiche — résidus restants légitimes (Change Logs P1/P2, sites
    « six » nommés pour être corrigés).
  - Signal D5 : **levé** (MEDIUM → MEDIUM ; deux MEDIUM nés de la remédiation P2, contenus dans
    l'AC3 bis) ; déclaré au Project Lead, pas de découpage (§ *Décompte des modules*). La P4 doit être
    complète (règle métier touchée).
  - Trend : P1 4 HIGH / 10 MEDIUM / 4 LOW (F + R) → P2 0 HIGH / 8 MEDIUM / 19 LOW → P3 0 HIGH /
    5 MEDIUM / 9 LOW (dédoublonnés : 3 MEDIUM distincts).
  - Décompte après passe : **13 AC (AC1–AC12 et AC3 bis), 10 tâches (T0–T9), 21 tests** (17 backend
    dont 16 bis neuf, 4 fichiers Vitest ; 20 neufs, 1 existant renommé et étendu).
- 2026-10-08 — **Validation P4** (Opus 5.5, lentilles R et F ; prompt `15-6b-validate-prompt-p4.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW ; F : 0 CRITICAL, 0 HIGH, 3 MEDIUM,
  4 LOW (R1 = F1, R5 ⊂ F6). Tous appliqués, décisions de l'orchestrateur comprises (choix C-15-6-30) :
  - **MEDIUM R1 = F1** — le compte du reste d'arrondi (5 bis) et celui de la nature `rounding` lisent
    `default_rounding_account_id`, la colonne du compte d'arrondi du règlement : ils portent désormais
    `role: rounding` ; `write_off_nature` ne vaut que pour `discount`, `bank_fees`, `bad_debt` (les trois
    champs de la section *Solde du reste*). Le rôle suit la colonne lue, pas la route (doc-comment).
    AC1, AC3 bis, AC9, AC10, AC11 ; test **10 bis** neuf (escompte à reste fractionnaire, compte
    d'arrondi = créance → `rounding`).
  - **MEDIUM R2** — critère de T2 réécrit : il vise la **construction** de la variante (littéral avec
    champs) et la **lecture du numéro** (`grep -rn`, `-r` compris) ; les déstructurations attendues
    sont énumérées hors du critère.
  - **MEDIUM F2** — « résolu par le type » était faux pour la validation et l'avoir : remplacé par des
    angles morts écrits — créance retypée à la validation, **#537** (ouverte par l'orchestrateur) ; TVA
    due et produit de repli de l'avoir lus dans les réglages courants, **#525** (§ *Le défaut*,
    inventaire, § *Angles morts*). Pas de garde neuve : deux modules de plus, hors de la classe
    « règlement ». La validation d'achat (même forme côté dette) est **signalée à l'orchestrateur**.
  - **MEDIUM F3** — le message de liste vide n'apparaît que si la liste filtrée est vide **et** qu'au
    moins un compte bancaire a été écarté par le filtre ; une société sans compte bancaire (installation
    neuve) garde le comportement actuel (AC8, tests 18 et 19).
  - **LOW** — R3 : sélecteur Fluent écrit sur plusieurs lignes, repli Rust par rôle (AC2) ; R4 : verdict
    E2E corrigé — le filtre par rôle est **inerte** en E2E (seed sans rôle), aucun E2E n'exerce l'AC8
    (angle mort écrit) ; R5 : `grep "payment-batch"` ne rend rien ; R6 : Dev Notes — numéro de la
    facture relu à la confirmation d'un lot ; R7 : erreur SQL de la lecture du numéro à la création d'un
    lot propagée (échec infra) ; R8 : transaction de `confirm_batch` ouverte `:311`, montages des tests
    `:352`/`:898` (des tests, pas des helpers), `ACCOUNT_NOT_POSTABLE` fournisseur renvoyé à T0, borne
    `sitesTotal` de la 15-5c à recompter (AC7) ; F4 : « un administrateur doit désigner » (C36) aux
    messages, au libellé `failed[]` de l'AC7 et au manuel ; F5 : « espèces et tout autre compte »
    (`user-manual.tex:1047`) corrigé avec le « libre » ; F6 : sites de l'AC10 (phrase « Refus » hors de
    la table d'annulation, phrase et non liste au § du rapprochement) ; F7 : ordre des refus quand
    plusieurs comptes d'écart sont la créance (nature, reste d'arrondi, TVA due).
  - Propagation : jetons `write_off_nature`, `WriteOffNature`, `désignez`, `résolu par le type`,
    `tout autre compte`, `payment-batch` grepés sur la fiche — résidus restants légitimes (Change Logs
    P2/P3, citations du finding, test 16 bis dont la nature `discount` reste `write_off_nature`).
  - **Signal D5 — levé, déclaré au Project Lead, pas de découpage.** Sévérité F : P3 3 MEDIUM → P4
    3 MEDIUM (égale). **F1 est un recyclage** : le discriminant `role` posé par la remédiation P3 pour
    donner le bon remède classait mal le compte d'arrondi du solde — un défaut né du correctif
    précédent, la forme que l'amendement D5 retient comme motif de découpage. Il ne découpe pas ici,
    pour trois raisons écrites : (1) le recyclage est **contenu** au même discriminant — une ligne de
    classement et un test, aucun module touché de plus ; (2) les deux autres MEDIUM ne recyclent rien
    (F2 date de la spécification initiale, F3 de la P1) ; (3) F2 est traité en angle mort et non en
    garde, précisément pour ne pas ajouter les sixième et septième modules qui déclencheraient le
    critère de dispersion. Le recyclage de la P3 (deux MEDIUM nés de la remédiation P2) et celui-ci
    portent sur la même règle (AC3 bis) : si la P5 en trouve un troisième, le découpage de l'AC3 bis en
    story propre est la mesure indiquée.
  - Trend : P1 4 HIGH / 10 MEDIUM / 4 LOW (F + R) → P2 0 HIGH / 8 MEDIUM / 19 LOW → P3 0 HIGH /
    5 MEDIUM / 9 LOW → P4 0 HIGH / 5 MEDIUM / 10 LOW (dédoublonnés : 4 MEDIUM distincts).
  - Décompte après passe : **13 AC (AC1–AC12 et AC3 bis), 10 tâches (T0–T9), 22 tests** (18 backend
    dont 10 bis et 16 bis, 4 fichiers Vitest ; 21 neufs, 1 existant renommé et étendu).
- 2026-10-08 — **Validation P5** (Sonnet, lentilles R et F ; prompt `15-6b-validate-prompt-p5.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 5 LOW ; F : 0 CRITICAL, 0 HIGH, 3 MEDIUM,
  5 LOW (R5-3 = F2). Tous appliqués, décision structurante de l'orchestrateur comprise (choix
  **C-15-6-33**) — **une seule règle d'ordre des verrous pour l'epic, celle de la 15-5e** :
  - **MEDIUM F1** — la fiche ignorait la **15-5e** : dépendances « après 15-5a à 15-5e » (en-tête,
    T0) ; étapes citées **par leur nom**. La 15-5e, **réécrite par C54** pendant cette remédiation
    (correction de consigne de l'orchestrateur), ne réordonne plus `settle_invoice` ni
    `write_off_invoice` : elle **rejoue** leurs routes (et celles du paiement fournisseur et de la
    confirmation d'un lot) et réécrit le commentaire « 5 bis ». L'ordre des refus de l'AC3 bis reste
    celui des lectures du code — **nature, puis reste d'arrondi, puis TVA due** au solde ;
    **contrepartie, puis trop-perçu, puis compte d'arrondi** au règlement —, désormais énoncé comme
    règle (chaque comparaison suit sa lecture).
  - **MEDIUM R5-1** — le sélecteur Fluent `$role` aurait rougi
    `no_new_select_expression_reaches_the_frontend_dictionary` (`loader.rs:482`) : **trois clés
    plates**, une par rôle désigné (comme la 15-5c) ; le repli Rust par `match` disparaît ; six clés
    serveur au lieu de quatre.
  - **MEDIUM R5-2** — la borne `sitesTotal` **existe** (1868) : relevée pour **tous** les sites neufs,
    comptés (+4, +6 avec la 15-5c), ventilation recomptée sur l'état rebasé (AC8, T0, T9).
  - **MEDIUM R5-3 = F2** — le critère de T2 était muet sur un littéral que rustfmt coupe : critère par
    le **nom seul** (`grep -rnF`, commentaires exclus) sur une **liste fermée** de cinq fichiers, chaque
    occurrence lue.
  - **MEDIUM F3** — `user-manual.tex:1754` (balance âgée) citait le règlement sur le compte débiteurs
    comme un cas ordinaire : ajouté à l'AC11, reformulé en données antérieures, contrôlé au PDF aplati.
  - **LOW** — R5-4 : le **produit de repli** de l'avoir n'est **pas** couvert par #525 (vérifié par
    `gh api` sur le corps et les commentaires) : écrit « non tracé », **signalé à l'orchestrateur** ;
    R5-5 : modules recomptés, **neuf** (le helper commun `invoice_settlements.rs` manquait) ; R5-6 :
    verdict E2E complété (`invoices.spec.ts:318-321`, `reminders.spec.ts:104-112`,
    `invoice-write-off.spec.ts:70-85`) ; R5-7 : test 11 placé dans `invoice_settlement.rs`, montage
    d'arrondi nommé, `:180-183` → `:182-185` ; R5-8 : colonne « issues » de la fiche d'index complétée
    (angles morts #537, #525) ; F4 : `:1047` → `:1048` ; F5 : `Payable` + rôle désigné rendu
    irreprésentable par un sujet `ClaimSubject` au helper de construction ; F6 : chaque comparaison suit
    sa lecture, une nature égale à la créance précède le `CONFIGURATION_REQUIRED` de la TVA due — test
    **10 ter** neuf ; F7 : phrase « Refus » du paiement fournisseur posée avant la ligne de refus de
    l'annulation, libellée par sa route ; F8 : pour mémoire (registre non réécrit ; la fiche d'index dit
    les révisions de C-15-6-3).
  - Propagation : jetons `15-5a, 15-5b, 15-5c et 15-5d`, `designated-account-is-receivable`,
    `sélecteur`, `$role`, `(4bis)`, `(5 bis)`, `étape (3)`, `étape (4)`, `huit`, `:1047`, `180-183`,
    `#525` grepés sur la fiche — résidus restants légitimes (Change Logs P1-P4 ; étapes de `pay_in_tx`,
    que la 15-5e ne touche pas).
  - Signal D5 : § *Décompte des modules*, « P5 » — levé par la sévérité, **non** déclenché : F1 vient
    d'un changement extérieur (la 15-5e), non d'un patch ; les deux recyclages (R5-1, F2) sont hors de
    l'AC3 bis.
  - Trend : P1 4 HIGH / 10 MEDIUM / 4 LOW (F + R) → P2 0 HIGH / 8 MEDIUM / 19 LOW → P3 0 HIGH /
    5 MEDIUM / 9 LOW → P4 0 HIGH / 5 MEDIUM / 10 LOW → P5 0 HIGH / 6 MEDIUM / 10 LOW (dédoublonnés :
    5 MEDIUM distincts). La remédiation P5 change l'ordre des refus et les clés i18n : une **P6
    complète** (Opus, rotation D6) est indiquée.
  - Décompte après passe : **13 AC (AC1–AC12 et AC3 bis), 10 tâches (T0–T9), 23 tests** (19 backend
    dont 10 bis, 10 ter et 16 bis, 4 fichiers Vitest ; 22 neufs, 1 existant renommé et étendu).
- 2026-10-08 — **Validation P6 — clôture** (Opus 5.5, lentilles R et F ; prompt
  `15-6b-validate-prompt-p6.md` ; rapports `target/gate-logs/15-6b-p6-R.md`, `15-6b-p6-F.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 11 LOW ; F : 0 CRITICAL, 0 HIGH, 0 MEDIUM,
  7 LOW (R6-4 = F6-6 ; F6-7 est une observation). **0 au-dessus de LOW : la boucle s'arrête** (§ *Review
  Iteration Rule*). Tous les LOW appliqués (choix C-15-6-34) :
  - R6-1 : test 11 sans renvoi à la « concurrence 1 » de la 15-5e, retirée par C55 (le helper
    `raw_due_invoice` existe toujours, `invoice_echeancier_e2e.rs:986`).
  - R6-2, R6-3 : le **produit de repli** de l'avoir est tracé par **#525**, la **validation d'achat**
    par **#537** (commentaires de l'orchestrateur du 2026-10-08) — « non tracé » / « signalé »
    corrigés au § *Le défaut*, à l'inventaire et aux angles morts (le Change Log P5 reste historique).
  - R6-4 = F6-6 : la règle de placement de l'AC3 bis dit « suit la lecture de son compte **et** celle
    de la créance » — la nature est lue avant la créance (`:433`, puis `:434-445`) ; aucun
    réordonnancement.
  - R6-5 : repli de `t_args` construit par `format!` (patron `errors.rs:88-92`).
  - R6-6 : fiche d'index — « un message par rôle (six clés) ».
  - R6-7 : `:409` → `:408`, `:454-480` → `:454-477`, table de l'annulation `:262-271` → `:259-268`.
  - R6-8 : `DesignatedRole` placé dans `invoice_settlements.rs` avec `ClaimSubject`, et cité au décompte.
  - R6-9 : test 10 ter — TVA due vidée **après** validation.
  - R6-10 : AC9 — le message `rounding` arrive aussi par le dialogue de **règlement**.
  - R6-11 : C-15-6-30, C-15-6-33, C-15-6-34 aux Références ; C36, C52, C54, C55 arrivent au rebase (T0).
  - F6-1 : **manuel d'administration** — verdict ligne par ligne (`:2027` inchangée ; une phrase à
    `:2029` ; « ou devenu le compte débiteurs de la facture » à `:2033`) ; frontière de la 15-6c alignée.
  - F6-2 : deux lignes « hors classe » à la table des écrans (`routes/(app)/+page.svelte:58`,
    `account-validity.ts:20`).
  - F6-3 : présélection dans un **effet distinct**, qui n'efface pas la saisie en cours ; test 18 étendu.
  - F6-4 : test 8 — transaction à référence de facture, score positif (étape 7bis avant (a)-(c)) ; AC5 :
    tous les refus antérieurs gardent la priorité.
  - F6-5 : clés de lot ajoutées par le mapping HTTP seul, hors de `claim_account_refusal_details`.
  - F6-7 : décompte au barème de la règle (neuf modules) écrit et déclaré (§ *Décompte des modules*, P6).
  - Propagation : jetons `non tracé`, `signalé à l'orchestrateur`, `aucune issue`, `concurrence 1`,
    `aussitôt`, `:409`, `454-480`, `262-271`, `DesignatedRole`, `deux messages` grepés sur les fiches
    15-6 — résidus restants légitimes (Change Logs P4/P5, citations des findings) ; `:558` de la 15-6a
    corrigé dans le même commit.
  - **Signal D5 — dispersion déclarée au Project Lead, non découpée** : neuf modules métier au barème
    de la règle, plus de cinq ; le « barème du geste » n'est plus invoqué. Raison écrite (§ *Décompte
    des modules*, P6) : fiche validée et convergée, gestes liés par une variante, un sujet typé, un
    lecteur sœur et trois fonctions d'écran ; le risque visé par la règle (non-convergence) ne s'est
    pas réalisé. Arbitrage de Guy en revue de fin d'epic.
  - **Trend complet** : P1 (Sonnet) 4 HIGH / 10 MEDIUM / 4 LOW (F + R) → P2 (Opus) 0 HIGH / 8 MEDIUM /
    19 LOW → P3 (Sonnet) 0 HIGH / 5 MEDIUM / 9 LOW → P4 (Opus) 0 HIGH / 5 MEDIUM / 10 LOW → P5 (Sonnet)
    0 HIGH / 6 MEDIUM / 10 LOW → **P6 (Opus) 0 HIGH / 0 MEDIUM / 18 LOW** (17 distincts). Aucun
    CRITICAL en six passes. Remédiations : Opus 5.5 (P2 à P6). Reclassements : aucun MEDIUM+ reclassé
    en dette ; la validation et l'avoir (F2 de la P4) sont des angles morts **tracés** (#537, #525).
  - Décompte après passe : **13 AC (AC1–AC12 et AC3 bis), 10 tâches (T0–T9), 23 tests** (19 backend
    dont 10 bis, 10 ter et 16 bis, 4 fichiers Vitest ; 22 neufs, 1 existant renommé et étendu) —
    inchangé (le test 18 gagne un cas, sans test neuf). Statut : `ready-for-dev`.
