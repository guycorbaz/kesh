# Story 15.5d : Les comptes de réglage contrôlés à l'usage — créance, TVA, créanciers, et le compte créanciers à l'écran

Status: ready-for-dev

<!-- Quatrième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P4 de la 15-5b
     (finding F4-3, choix C33 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b l'ancien AC20
     (garde à l'usage des comptes de réglage, choix C27 et C28, révision de la limite L2 de D-A0) et tout
     ce qui s'y rattache — la variante `DesignatedAccountsNotPostable`, sa clé i18n, ses tests, ses
     passages de manuel —, plus la remédiation des deux HIGH et du MEDIUM que cette règle a fait naître
     en P4 : le compte créanciers exposé à l'écran (C34, révise C25) et l'avoir exempté avec sa vraie
     raison (C35). Choix applicables : C6, C16, C19, C25 (révisé par C34), C27, C28 (texte révisé par
     C36), C29, C33, C34 (révisé par C45), C35, C36, C39 (révisé par C42), C40, C41, C42, C43 (révisé
     par C48), C44, C45, C46, C47, C48, C49, C50, C51, C52, C53, C65.
     Découpée à son tour après la validation P3 (choix C52, ligne de partage C53) : le réalignement
     des flux de règlement sur l'ordre canonique des verrous est sorti en 15-5e, qui passe avant ;
     la 15-5e a été réécrite depuis (choix C54) : elle rejoue sur interblocage toutes les routes qui
     écrivent au journal, et ne garde de l'ordre que l'avance des réglages de la saisie fournisseur ;
     puis découpée (choix C61) en 15-5e1 (socle, dont cette story dépend seule — C65) et 15-5e2.
     Historique complet au Change Log.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation (les 15-5b et
     15-5c portent le même) — il n'existe pas de statut « en validation » dans `sprint-status.yaml`. Le
     développement attend la clôture de la boucle de validation ET le merge des 15-5a, 15-5b et 15-5e1
     (finding R1-7 de la P1 ; C52 ; C65 : la 15-5e a été découpée en 15-5e1 et 15-5e2, cette story ne
     dépend que de la première). -->

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
**Et de la 15-5e1** (`15-5e1-socle-rejeu.md`, choix C52, réécrite selon C54, découpée par C61 —
« Socle du rejeu sur interblocage »), **et pas de la 15-5e2** (choix C65) : le **rejeu de la
validation d'une facture** et le patron des enveloppes ; l'avance des réglages de la saisie fournisseur
avant les comptes de charge ; le doc-comment canonique de `validate_invoice` (numérotation **(2 bis')**)
et le commentaire « 5 bis » réécrits. Cette story **suppose ce rejeu et cette avance en place** et y
place son verrou ; elle n'en redécrit que ce qui la concerne et **renvoie à la 15-5e1** pour le reste.
Le rejeu de la **saisie fournisseur** et de la **complétion d'une facture importée** vient avec la
**15-5e2** (`15-5e2-rejeu-des-autres-flux.md`), dans un ordre de merge libre : si cette story merge
avant elle, ces deux routes portent le verrou de l'accesseur sans être encore rejouées, et une victime
d'interblocage y rend `500` comme aujourd'hui jusqu'au merge de la 15-5e2 — une fenêtre de même
nature que l'état actuel, pas une régression de nature.
Ne pas commencer avant le merge de la 15-5e1. **Indépendante de la 15-5c** : elles touchent des
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
     de produit ou de charge d'une ligne — les tests le font : `supplier_invoices_repository.rs:49-55`
     désigne `2000` comme compte créanciers alors que `seed_accounting_company` l'a déjà désigné comme
     TVA due, finding R2-7). Concrètement : un type `DesignatedRole` (`Receivable`, `VatPayable`,
     `Payable`, `VatRecoverable`) et un ensemble ordonné de rôles (`BTreeSet<DesignatedRole>` ou
     newtype équivalent) rendus **avec** les lignes, dans une structure dont la forme est **fixée**
     (finding F3-6 de la P3, choix **C50**) :
     `pub(in crate::repositories) struct GeneratedLines { pub lines: Vec<NewJournalEntryLine>, pub roles: BTreeSet<DesignatedRole> }` ;
     le générateur de vente (et sa variante `_rounded`) rend `Result<GeneratedLines, DbError>`, celui
     d'achat `Result<(GeneratedLines, Decimal), DbError>` (le total TTC reste le second membre du
     tuple) ; les **19 tests** sont adaptés **mécaniquement** — ils lisent le champ `.lines` (p. ex.
     `generate_…(…).unwrap().lines`, ou `let (generated, ttc) = …; generated.lines` côté achat), sans
     autre changement de leurs assertions. `DesignatedRole` est un type **neuf** plutôt qu'`AccountRole`
     (`crates/kesh-db/src/entities/account.rs:90-102`), délibérément (finding F2-4, choix **C44**) :
     l'ensemble désigne des **champs des réglages** effectivement écrits, non le rôle que le plan
     attribue au compte (un compte désigné ne porte pas forcément ce rôle) ; ses **quatre** variantes
     se traduisent en champs par un `match` exhaustif, sans les neuf autres rôles d'`AccountRole` à
     écarter par un bras mort ; et `AccountRole` ne dérive pas `Ord`, qu'exige un `BTreeSet` —
     - côté vente, par `generate_invoice_journal_lines` (`invoices.rs:1784`) : `Receivable` toujours,
       `VatPayable` **dans la branche même** `if total_vat > Decimal::ZERO` (`:1851`) qui écrit les
       lignes de TVA due ; `generate_invoice_journal_lines_rounded` (`:1879`) le transmet ;
     - côté achat, par `generate_purchase_journal_lines` (`supplier_invoices.rs:105`, qui rend déjà un
       tuple `(lignes, total TTC)`) : `Payable` toujours, `VatRecoverable` dans la branche
       `if total_vat > Decimal::ZERO` (`:129`) ;
     - l'appelant traduit chaque rôle en l'identifiant qu'il a lui-même passé au générateur (créance,
       TVA due, créanciers, TVA récupérable des réglages) et appelle l'accesseur sur ces identifiants.
       Ventilation des 25 occurrences de
       `grep -rnE "generate_(invoice_journal_lines(_rounded)?|purchase_journal_lines)\(" crates | grep -v "///"`
       sur `ac1719b9` (findings R2-2 / F2-3, recompté) : **3 définitions** (`invoices.rs:1784`, `:1879`,
       `supplier_invoices.rs:105`), **1 transmission** (`invoices.rs:1886`, `_rounded` → générateur),
       **2 appels de production** (`invoices.rs:2247`, `supplier_invoices.rs:365`) — ceux que la garde
       adapte — et **19 tests** (14 dans `invoices.rs`, 4 dans `supplier_invoices.rs`, 1 dans
       `credit_notes.rs:933`, test de 16-1a qui compare la facture et sa contre-passation compte par
       compte). **L'avoir n'appelle pas ces générateurs** : il a le sien,
       `generate_credit_note_journal_lines` (`credit_notes.rs:187`, appelé à `:507-512`), que la story
       ne touche pas — c'est la forme exacte de son exemption (C35, AC3 ; révision de C39 consignée en
       **C42**) ;
   - un accesseur neuf de `company_invoice_settings`, **en deux temps** (finding F2-1 = R2-1, choix
     **C43** — voir « L'ordre des verrous » ci-dessous) :
     - **verrouiller** (p. ex. `lock_designated_accounts_in_tx(conn, company_id, ids: &[i64])`) :
       lit `id, number, active, postable` de **tous les comptes candidats du flux** — ceux que les
       réglages désignent pour les rôles que le flux **peut** écrire, qu'il les écrive ou non —, en
       **une seule requête** `WHERE company_id = ? AND id IN (…) ORDER BY id FOR UPDATE` — l'**ordre
       de verrouillage par identifiant** est fixé, pour que deux flux concurrents ne prennent jamais
       ces lignes dans des ordres opposés (finding R1-6) — et rend cet instantané, **sans refuser** ;
       un identifiant `None` des réglages n'y entre pas. **Un identifiant d'une autre société ne doit
       pas être verrouillé** (finding F3-5 de la P3, choix **C51**) : le dépôt a mesuré qu'un
       `FOR UPDATE` par clé primaire verrouille la ligne **avant** que le filtre `company_id` ne
       l'écarte (`opening_complement.rs:431-440`, revue de code P1 de la story du complément
       d'ouverture, B-F1). Les identifiants viennent des réglages de la société, que la 15-5b garde à
       la désignation : le cas ne naît que d'un `UPDATE` direct, d'où un **test** (AC7) plutôt qu'un
       filtrage d'office ; si ce test montre la ligne étrangère verrouillée, l'accesseur prend le
       patron `owned_account_ids` d'`opening_complement.rs` — les identifiants de la société
       d'abord, par une lecture non verrouillante (un compte ne change jamais de société), puis le
       `FOR UPDATE` sur eux seuls — et le Dev Agent Record le dit. Le **plan d'exécution** de la
       requête (`EXPLAIN`, accès par clé primaire sur `id IN (…)`, aucun `filesort` qui parcourrait
       — donc verrouillerait — d'autres lignes : piège mesuré à `opening_complement.rs:274-288`) est
       relevé au Dev Agent Record ;
     - **contrôler**, après la génération : sur l'instantané (les lignes sont verrouillées jusqu'au
       commit, il n'y a rien à relire), **seuls les comptes des rôles rendus par le générateur** (C39,
       inchangé). L'accesseur **refuse lui-même** (finding F3-4 de la P3, choix **C49**), dans cet
       ordre, écrit à son doc-comment :
       1. un compte d'un rôle écrit **absent de l'instantané** (inexistant, ou d'une autre société)
          **ou inactif** (archivé) → `InactiveOrInvalidAccounts`, le refus que `create_in_tx` rend
          aujourd'hui pour ce cas — même variante, même code, aucun contrat neuf ; la 15-5a interdit
          de nommer un tel compte, il n'est donc pas nommé ;
       2. sinon, les comptes **de la société, actifs et non imputables** →
          `DesignatedAccountsNotPostable`.
       Pourquoi l'accesseur et non `create_in_tx` : le contrôle de `create_in_tx`
       (`journal_entries.rs:96-99`) est une lecture **non verrouillante**, qui lit sous REPEATABLE
       READ l'instantané ouvert par la première lecture simple de la transaction — antérieure au
       verrou ; un compte archivé entre les deux y paraîtrait encore actif et l'écriture passerait.
       L'accesseur, lui, tient la ligne **fraîche** sous verrou (même raison que le filtre
       `active = TRUE` de `rounding_account_for_write`, `company_invoice_settings.rs:497-499`, et que
       la mise en garde d'`opening_complement.rs:431-434`). Le contrôle de `create_in_tx` reste en
       place, inchangé, pour les autres comptes de l'écriture ;
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
     - **le verrou** : les candidats sont la **créance** (`default_receivable_account_id`, déjà exigée
       à `:2008-2010`) et la **TVA due** (`default_vat_payable_account_id`, si désignée) ; ils sont
       verrouillés **après** le compte d'arrondi (`rounding_account_for_write`, `:2065`) et **avant**
       l'exercice (`fiscal_years::find_open_covering_date`, `:2172`) — c'est-à-dire à l'étape
       **(2 bis')** du doc-comment canonique, celle du compte d'arrondi, dans la numérotation que fixe
       l'AC5 de la 15-5e1 (alignée sur les étapes du code ; l'ancien « 1 bis. `accounts` » n'existe
       plus, « (1 bis) » étant le projet lu sans verrou) ;
     - **le contrôle** : **entre la génération des lignes** (`generate_invoice_journal_lines_rounded`,
       `:2247-2253`) **et `create_in_tx`** (`:2255`) — finding R4-3/F4-5 —, sur les rôles rendus par le
       générateur : la **créance** toujours ; la **TVA due** **seulement si le générateur l'a écrite**,
       c'est-à-dire si `total_vat > 0` (`invoices.rs:1849-1852`, calculé sur les montants **arrondis
       par ligne**) — **pas** « une ligne à taux > 0 » : une facture à taux positif dont la TVA arrondit
       à zéro n'écrit rien sur la TVA due et n'est pas bloquée (finding F4-4 ; cas existant
       `gen_lines_rate_rounds_to_zero`), même si son compte a été verrouillé ;
     - **l'ordre des refus** (réécrit en P2, **écrit dans le doc-comment « # Erreurs » de
       `validate_invoice`**) : le verrou ne refuse rien, il ne déplace donc aucun refus ; le refus de
       cette story vient **après** tous les refus antérieurs — total nul, montant minimum, compte
       d'arrondi, comptes de produit des lignes (2 ter, `ACCOUNT_NOT_POSTABLE` de la 15-5a),
       **exercice** (`FiscalYearInvalid`, bien que le verrou des comptes le précède désormais) — et
       après le `ConfigurationRequired("default_vat_payable_account_id")` du générateur (TVA due
       **absente**) ; il précède `create_in_tx`. « Le refus » désigne ici les deux refus de
       l'accesseur, dans leur ordre (C49 : compte absent ou inactif, puis non imputable) ;
   - **création d'une facture fournisseur** (`crates/kesh-db/src/repositories/supplier_invoices.rs`,
     `create_in_tx` — donc aussi la **complétion d'une facture importée**, qui l'appelle,
     `routes/imported_supplier_invoices.rs:243`) :
     - **les réglages** sont chargés après le projet et le fournisseur, **avant** les comptes de charge
       — avance posée par la **15-5e1** (son AC5), que cette story suppose ;
       l'exigence `ConfigurationRequired("default_payable_account_id")` reste à sa place (après
       l'exercice) : la priorité des refus ne bouge pas ;
     - **le verrou** : les candidats, **créanciers** (`default_payable_account_id`, valeur résolue
       par l'AC19 de la 15-5b) et **TVA récupérable** (`default_vat_recoverable_account_id`), sont
       verrouillés **après** les comptes de charge et **avant** l'exercice ;
     - **le contrôle** : **entre** `generate_purchase_journal_lines` (`:365-369`) **et**
       `journal_entries::create_in_tx` (`:372`), sur les rôles rendus : les **créanciers** toujours ;
       la **TVA récupérable** **seulement si le générateur l'a écrite** (TVA totale positive,
       `:129-131`). Le compte de **charge** non imputable est refusé plus tôt, par la 15-5a
       (`supplier_invoices.rs:337-346`) : son refus passe avant ; l'exercice aussi ;
   - **l'ordre des verrous** (finding **F2-1 HIGH = R2-1**, choix **C43** ; règle posée par la
     **15-5e1**, choix C54). La règle — **l'ordre réduit la fréquence des interblocages ; le rejeu des
     routes les rend invisibles** — est écrite au doc-comment de `validate_invoice`
     (`invoices.rs:1916-1929`) tel que la 15-5e1 l'aura réécrit. Cette story y place **son seul verrou
     neuf**, celui de l'accesseur : à la validation, **après** le compte d'arrondi et **avant**
     l'exercice ; à la saisie fournisseur, **après** les comptes de charge et **avant** l'exercice.
     **Pourquoi avant l'exercice** : une garde qui verrouillerait la TVA due **après** l'exercice
     croiserait **à coup sûr** le solde du reste, qui la verrouille avant (T1 valide la facture B :
     tient l'exercice, attend la TVA due ; T2 solde la facture A avec TVA : tient la TVA due, attend
     l'exercice), et, côté achat, un règlement fournisseur par compte interne égal au compte
     créanciers (compte, puis exercice, `supplier_invoices.rs:639-662`). **Pourquoi après
     l'arrondi** : la validation prend déjà l'arrondi en premier parmi ses comptes ; l'accesseur suit
     la convention. Ces places **réduisent la fréquence** des interblocages ; elles **ne les excluent
     pas** : l'insertion des lignes reprend la créance, la TVA due et les créanciers par la clé
     étrangère `fk_jel_account`, **après** l'exercice (finding F1-1 de la P1 de la 15-5e), et un
     compte interne de règlement peut être l'un de ces comptes (`invoice_settlements_write.rs:135-137`).
     Un interblocage restant est **rejoué** par les routes (15-5e1 pour la validation, 15-5e2 pour la
     saisie fournisseur et la complétion d'import) ; cette story n'affirme l'absence
     d'aucun cycle. D'où les deux temps de l'accesseur : **verrouiller tôt tous les candidats,
     contrôler tard les seuls rôles écrits**. Verrouiller un candidat que le générateur n'écrira pas
     (TVA due d'une facture sans TVA) coûte un verrou de ligne superflu, jamais un refus ;
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
     `ACCOUNT_NOT_POSTABLE` **ne doit pas** emprunter : son message dit déjà où agir. **Le dialogue se
     ferme** sur `ACCOUNT_NOT_POSTABLE` (finding F2-6, choix **C46**) : le code est ajouté à la liste
     des « erreurs non-retryables » (`:428-434`, aujourd'hui `FISCAL_YEAR_INVALID` et
     `CONFIGURATION_REQUIRED`) — qu'il vienne d'un compte de réglage (cette story) ou d'un compte de
     produit d'une ligne (15-5a), réessayer depuis le dialogue rend le même refus ; le message reste
     affiché par `notifyError` ;
   - **saisie d'une facture fournisseur** : `frontend/src/routes/(app)/supplier-invoices/+page.svelte:213-214`
     (`catch` à `:213`, `if (isApiError(err)) formError = err.message;` à `:214`). Les `catch` de `:103` (scan du QR,
     `supplier-invoices-scan-failed`) et de `:133` (chargement de la page) **ne sont pas** concernés :
     aucun des deux n'atteint `create_in_tx` ;
   - **complétion d'un import** : `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte:261`
     (`formError = completeErrorLabel(err)`), dont le `switch` replie sur `err.message` par son
     `default:` (`:300`) — `ACCOUNT_NOT_POSTABLE` ne doit **pas** y recevoir de branche qui en
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
   - un `<select>` **Compte créanciers (Passif)** dans la section *Comptes par défaut*
     (`:296-342`, titre `settings-invoicing-default-accounts-title`), **juste après la créance**
     (`:300-314`), son pendant côté achat — **pas** dans la section *Comptes TVA* (`:344-400`), où
     se trouve la TVA due : le compte créanciers n'est pas un compte de TVA (findings R2-6 / F2-5),
     `data-testid="settings-payable-account"`, clé neuve
     `settings-invoicing-payable-account` dans les quatre locales (« Compte créanciers (Passif) » ;
     DE/IT/EN sur le patron de `invoices-settings-vat-payable`, `messages.ftl:502` / `:508`) ;
   - ses options : `withCurrentAccount(liabilityAccounts, payableId, accounts)` — **même filtre que la
     TVA due** (`liabilityAccounts` : `active && postable && accountType === 'Liability'`, `:70-72` ; le
     serveur exige le type `Liability`, AC19 de la 15-5b) ; la **valeur courante est préservée** même si
     le compte est devenu non imputable (patron #271, `:80-87`) : elle reste affichée et sélectionnée,
     et l'exemption « inchangé » de l'AC11 de la 15-5b la laisse passer à l'enregistrement ;
   - la valeur est **lue** au chargement (`onMount`, `:123-145`) et à la relecture sur conflit de
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
6. **AC6 — Les contournements des E2E restent, et leur motif est corrigé** (findings R2-3 / F2-2,
   choix **C45**, qui révise C34). L'inventaire se fait **par commande**, non par liste :
   `grep -rn "defaultPayableAccountId == null" frontend/tests/e2e` rend **trois** sites sur `ac1719b9`
   — `payment-batches.spec.ts:59`, `inbox-import.spec.ts:93` et `supplier-invoices.spec.ts:86`, cette
   dernière exerçant le flux même que l'AC1 garde ; T0 refait la commande. Chacun repose le compte
   créanciers par un `PUT` d'API quand il vaut `null`. La prémisse des passes précédentes (« le seed
   E2E le désigne ») est **fausse** : les presets `post-onboarding` et `with-company` appellent
   `seed_accounting_company` (`crates/kesh-api/src/routes/test_endpoints.rs:184`), **pas**
   `insert_with_defaults`, et son `INSERT INTO company_invoice_settings`
   (`crates/kesh-db/src/test_fixtures.rs:156-170`) ne pose que créance, produit, TVA due et TVA
   récupérable — jamais `default_payable_account_id`. Les contournements complètent donc un seed
   **indépendamment** de #521 : ils **restent**. Le dev **réécrit leur commentaire** dans les trois
   specs : « le seed `with-company` (`seed_accounting_company`) ne désigne pas le compte créanciers ».
   Le seed n'est **pas** étendu (rayon d'impact : tous les tests `kesh-db` et `kesh-api` qui l'emploient,
   dont des assertions sur le contenu des réglages — hors de cette story, C45). La suite E2E complète du
   gate D7 (T7) reste le juge de non-régression de ces trois fichiers, jugée fichier par fichier contre
   `docs/testing.md` § « Les échecs attendus ». Le résultat est consigné au Dev Agent Record.

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
       15-5a (ligne) ; TVA due **absente** → `ConfigurationRequired` (finding F4-5) ; **aucun exercice
       ouvert** à la date de la facture **et** créance non imputable → `FiscalYearInvalid` (le verrou
       des comptes précède désormais l'exercice, leur refus non — finding F2-1, C43) ; côté achat,
       aucun exercice ouvert **et** créanciers non imputables → `FiscalYearInvalid` ;
     - **créanciers** non imputable → saisie de facture fournisseur refusée, rien d'écrit ;
     - **TVA récupérable** non imputable : facture fournisseur **avec** TVA → refusée ; **sans** TVA
       → acceptée ;
     - créance **et** TVA due non imputables, désignées sur **deux comptes distincts** (`1100` et
       `2000` de `seed_accounting_company`) → **un** refus nommant les deux, `count = 2`, pluriel
       (finding F1 de la P1) ;
     - **un même compte désigné pour deux rôles** (créance et TVA due posées sur le même identifiant
       par un `UPDATE company_invoice_settings` direct dans le montage, sur le patron de
       `supplier_invoices_repository.rs:49-55` — finding R2-7), rendu non imputable, facture avec TVA → **un** refus qui le nomme
       **une fois**, `count = 1`, message au **singulier** (choix C40 ; il prouve que le
       dédoublonnage de `NonPostableAccounts::new` est bien atteint par l'accesseur) ;
     - compte de réglage **archivé** → `InactiveOrInvalidAccounts`, désormais rendu par l'accesseur
       (C49 ; la variante neuve n'est pas émise). ⚠️ Ce test prouve le **résultat**, non **qui** le
       produit : sans la course d'archivage (fenêtre de quelques millisecondes entre l'instantané et
       le verrou), `create_in_tx` rendrait le même refus. Le refus de l'accesseur est donc vérifié
       **à la lecture** et consigné au Dev Agent Record ; sa mutation n'est pas attendue rouge, et
       c'est écrit ;
     - **un identifiant d'une autre société** (finding F3-5, choix C51) : un compte d'une seconde
       société, posé comme TVA due des réglages par un `UPDATE company_invoice_settings` direct ;
       une connexion ouvre une transaction et appelle l'accesseur de verrou sur les candidats (dont
       cet identifiant), **sans conclure** ; une seconde connexion verrouille la ligne étrangère par
       `SELECT id FROM accounts WHERE id = ? FOR UPDATE NOWAIT` → **réussit** (la ligne n'est pas
       verrouillée). S'il échoue, l'accesseur prend le patron `owned_account_ids` (AC1) et le test
       reste ; le plan (`EXPLAIN`) est relevé au Dev Agent Record ;
     - **avoir** (`crates/kesh-db/tests/credit_notes_repository.rs`) sur une facture dont la
       créance des réglages est devenue non imputable → émis (C35 :
       l'exemption est voulue et un test la fige) ;
     - **place du verrou de l'accesseur — trois tests**, qui figent cette **place** (findings
       F2-1 = R2-1, F3-2 ; choix C43, C53) — **pas** l'absence d'interblocage, que plus rien n'affirme
       (C54 ; le rejeu est à la route, 15-5e1 et 15-5e2). Patron commun : deux connexions de travail et une
       transaction **bloqueuse**, `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:560`,
       précédents `opening_complement_repository.rs:717`, `supplier_invoices_repository.rs:1442`) ;
       les motifs d'attente sont fixés par le dev et écrits au test (pour l'accesseur, p. ex.
       `FROM accounts WHERE company_id` et `ORDER BY id FOR UPDATE`) :
       1. **vente — solde du reste ↔ validation** — `crates/kesh-db/tests/invoice_write_off.rs`, dont
          le montage fournit ce que le test exige (finding R3-4 de la P3) : `company()` (`:35`)
          désigne le compte d'**escompte** (`default_discount_account_id`, sur `4000`) et le compte
          d'arrondi ; `validated_invoice()` (`:57`) pose une facture A **validée avec TVA** et un
          **reste dû** égal à son TTC (écriture de vente posée à la main, comme le fait ce fichier) ;
          la facture brouillon B, même société, est créée puis validée par le vrai chemin
          (`invoices::validate_invoice`, patron `invoices_validate_vat.rs:84`). La bloqueuse
          verrouille la ligne de l'exercice (`SELECT … FROM fiscal_years WHERE id = ? FOR UPDATE`) ;
          tâche 1 : **escompte** sur A (montant au centime : pas de compte d'arrondi) — vue en
          attente sur l'exercice (motifs `fiscal_years` et `FOR UPDATE`), elle tient donc la TVA
          due ; tâche 2 : **validation avec TVA** de B — vue en attente **sur le verrou de
          l'accesseur** — c'est le critère ; la bloqueuse annule ; chaque tâche finit **en succès ou
          en 1213** (reconnu par `kesh_db::retry::is_deadlock_error`), jamais par une autre erreur :
          au niveau du dépôt, sans rejeu, la reprise de la créance par `fk_jel_account` après
          l'exercice peut faire de l'une la victime, et l'issue n'est pas le critère. Sous l'ordre fautif (verrou après
          l'exercice), la tâche 2 attend l'exercice et non les comptes :
          `attendre_une_requete_en_cours` panique au bout de dix secondes — le test rougit sans
          dépendre du minutage ;
       2. **achat — règlement fournisseur par compte interne ↔ saisie fournisseur** (finding F3-2) —
          `crates/kesh-db/tests/supplier_invoices_repository.rs`, montage de ce fichier (compte
          créanciers désigné sur `2000`, `:49-55`) : une facture fournisseur A saisie avant ; la
          bloqueuse verrouille la ligne de l'exercice ; tâche 1 : **règlement de A par compte
          interne = le compte créanciers** — vue en attente sur l'exercice, elle tient donc ce
          compte ; tâche 2 : **saisie d'une facture fournisseur à TVA** B — vue en attente **sur le
          verrou de l'accesseur** — le critère ; la bloqueuse annule ; chaque tâche finit en succès
          ou en 1213, comme au test 1. Sous l'ordre fautif
          (verrou des créanciers et de la TVA récupérable après l'exercice), la tâche 2 attend
          l'exercice : le test rougit de même ;
       3. **vente — la validation prend l'arrondi avant les comptes désignés** (C53) —
          `crates/kesh-db/tests/invoice_amount_due_parity.rs` (montage `seeded_with_rounding`, `:72` :
          compte d'arrondi désigné) : une facture brouillon **avec TVA et arrondie** (total TTC non
          multiple de 0.05). La bloqueuse verrouille la ligne du **compte d'arrondi** ; tâche :
          **validation** — vue en attente sur le compte d'arrondi (motifs de la requête de
          `rounding_account_for_write`) ; **sonde** `SELECT id FROM accounts WHERE id = ? FOR UPDATE
          NOWAIT` sur la **TVA due**, puis sur la **créance** → **réussissent** (puis annulent). Sous
          l'ordre fautif (verrou de l'accesseur avant l'arrondi), la tâche tient déjà ces deux comptes
          et les sondes échouent aussitôt. La bloqueuse annule ; la validation réussit (une seule
          transaction de travail, rien avec quoi former un cycle) ;
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
   - **mutation** (objet précisé, finding R2-8) : pour chacun des quatre rôles, **ne pas l'insérer**
     dans l'ensemble rendu par le générateur → son test rougit ; **retirer l'appel du contrôle** dans
     chacun des deux flux → les tests de ce flux rougissent ; **une mutation d'ordre par flux**
     (finding F3-2) : dans `validate_invoice`, déplacer le verrou des comptes désignés après
     `find_open_covering_date` → le test de place 1 rougit ; dans
     `supplier_invoices::create_in_tx`, le même déplacement → le test 2 rougit ; dans
     `validate_invoice`, déplacer ce verrou **avant** le compte d'arrondi → le test 3 rougit ; poser
     le rôle `VatPayable` **hors** de la branche `total_vat > 0` du générateur → le test « TVA
     arrondie à zéro » rougit ; contourner `NonPostableAccounts::new` (liste construite sans
     dédoublonnage) → le test « même compte pour deux rôles » rougit ; consigné au Dev Agent Record,
     en touchant le fichier après restauration. Les mutations du rejeu des routes sont celles des
     **15-5e1** et **15-5e2** ;
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
     (`:409-415`) — **et le dialogue fermé** (C46). Mutations attrapées : `ACCOUNT_NOT_POSTABLE` ajouté
     à cette branche ; retiré de la liste de fermeture (`:428-434`). Les deux `catch`
     fournisseurs (`supplier-invoices/+page.svelte:213-214`, `import/+page.svelte:261`) n'ont pas de branche
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
   - Doc-comments de l'accesseur (ses deux temps, et pourquoi), de `validate_invoice` — « # Erreurs »
     (ordre des refus de l'AC1) **et « # Ordre des locks (canonique) »** (`invoices.rs:1916-1929`,
     tel que la 15-5e1 l'aura réécrit : l'étape **(2 bis')** de sa numérotation — celle du compte
     d'arrondi, AC5 de la 15-5e1 — gagne les comptes désignés, verrouillés après le compte d'arrondi
     et avant `fiscal_years`, par identifiant croissant ; finding R2-1 de la 15-5d, et R2-1 de la P2
     de la 15-5e) — et de
     `supplier_invoices::create_in_tx` (réglages → comptes de charge → comptes désignés → exercice) ;
     la limite **L2** de D-A0 est citée comme **révisée** pour ces quatre comptes
     (`14-3b-consommateurs-roles.md:188`).
   - Le commentaire « 5 bis » du solde du reste (`invoice_settlements_write.rs`), **réécrit en place
     par la 15-5e1** (son AC5) sans le verrou de la validation sur la TVA due, qui n'existait pas
     encore, **gagne une phrase** : la validation verrouille désormais la TVA due (et la créance),
     après le compte d'arrondi et **avant** l'exercice, comme le solde du reste ; un interblocage
     restant entre eux est rejoué par les deux routes (validation : 15-5e1 ; solde du reste : déjà
     rejoué, migré vers l'enveloppe par la 15-5e2). Aucune autre phrase de ce commentaire
     n'est touchée ; il ne prétend à l'absence d'aucun cycle (finding F3-3, C54).
   - `docs/api-external.md`, table du § 10 : la ligne `ACCOUNT_NOT_POSTABLE` (posée par la 15-5a,
     étendue par la 15-5b) gagne la **validation d'une facture** et la **saisie d'une facture
     fournisseur** quand un compte désigné dans les réglages n'est pas imputable — même `details`.
     Et la table du solde du reste (`docs/api-external.md:281`, « Compte de TVA due absent (escompte,
     perte) | `CONFIGURATION_REQUIRED` ») devient « Compte de TVA due **absent ou inutilisable**
     (archivé, non imputable) » : c'est ce que rend déjà `vat_payable_account_for_write`
     (`company_invoice_settings.rs:454-475`), et la divergence de code avec la validation (C28, AC3)
     doit se lire là où l'intégrateur lit (finding F2-7).
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

- [ ] **T0 — Refaire les relevés** sur `HEAD`, **après le merge de la 15-5e1** (donc des 15-5a et 15-5b) : numéros de ligne de
      `invoices.rs` (génération, `create_in_tx`, `:1849-1852`), `supplier_invoices.rs`, `credit_notes.rs`,
      des écrans et des manuels ; refaire le grep des lecteurs des quatre comptes
      (`grep -rnE "default_(receivable|payable|vat_payable|vat_recoverable)_account_id" crates/*/src`, hors
      tests et module des réglages) — **un lecteur qui écrit, absent de l'AC1 et de l'AC3, bloque la
      story** ; refaire le grep des appelants des générateurs (AC1, 25 occurrences sur `ac1719b9`, ventilées) et
      celui des contournements E2E (AC6, trois sites) ; **vérifier que la 15-5e1 est en place** — rejeu
      de la validation, réglages de la saisie fournisseur avant les comptes de charge, doc-comment
      canonique de `validate_invoice` et « 5 bis » réécrits ; **relever si la 15-5e2 est mergée** (rejeu
      de la saisie fournisseur et de la complétion d'import) et le consigner, sans en faire une
      condition (C65) — et
      relever les deux cases où l'accesseur s'insère (AC1, « L'ordre des verrous ») ; **un écart bloque
      la story** (la 15-5d ne réordonne aucun flux existant) ;
      **relever le texte exact que la 15-5b aura laissé** dans les encadrés `user-manual.tex:380` et
      `:390` et à `admin-manual.tex:2016-2025` (l'item que l'AC8 réécrit n'existe pas sur `67c31c95`),
      le copier au Dev Agent Record et, s'il ne contient aucun des motifs du grep de T6, **ajouter à ce
      grep** le motif qui le retrouve (finding R1-8 de la P1) ; relever le montage que la 15-5b aura
      écrit dans `company_invoice_settings_postable_e2e.rs` (AC7).
- [ ] **T1 — La garde** (AC1, AC3) : les rôles rendus par les deux générateurs (`DesignatedRole`,
      branches de `total_vat > 0`), rendus par `GeneratedLines` (C50), leurs appelants adaptés — dont
      les 19 tests, mécaniquement ; l'accesseur de `company_invoice_settings` en deux temps — verrou
      de tous les candidats (`ORDER BY id FOR UPDATE`, avant l'exercice, `EXPLAIN` relevé, C51),
      contrôle des rôles écrits (compte absent ou inactif → `InactiveOrInvalidAccounts`, puis variante
      construite par `NonPostableAccounts::new`, après la génération, C49) ; ses appels, aux places
      fixées à l'AC1 (après l'arrondi ou les comptes de charge, avant l'exercice) ; la variante `DesignatedAccountsNotPostable` (`error_code()` → `"ACCOUNT_NOT_POSTABLE"`,
      `match` exhaustif de `kesh-db/src/errors.rs`) et son bras dans `crates/kesh-api/src/errors.rs` (400,
      `t_args`, `details()`) ; doc-comments.
- [ ] **T2 — Le message** (AC2) : la clé dans les quatre `messages.ftl`, inscrite à
      `SELECTEURS_RESOLUS_COTE_SERVEUR` ; tests Rust par locale, singulier et pluriel.
- [ ] **T3 — Les écrans** (AC4, AC5) : lecture des trois `catch` de l'AC4, consignée ;
      `ACCOUNT_NOT_POSTABLE` ajouté aux codes qui ferment le dialogue de validation (C46) ; le `<select>`
      du compte créanciers, ses types, le mock de `settings-invoicing-page.test.ts`, sa clé i18n
      (quatre locales) ; borne `sitesTotal` de
      `frontend/src/lib/shared/i18n-keys.test.ts` relevée **délibérément**, ventilation recomptée
      (`sitesTotal`, `sitesNonResolus`, `relais`, `sitesGabarit`, `litterauxMin`, `clesDepuisTsMin`) et
      écrite au Dev Agent Record.
- [ ] **T4 — Les E2E** (AC6) : les trois contournements gardés, leur commentaire réécrit (le seed
      `seed_accounting_company` ne désigne pas le compte créanciers) ; non-régression jugée sur la
      **suite E2E complète** de T7.
- [ ] **T5 — Les tests** (AC7) : `kesh-db` (dont « même compte pour deux rôles » et « deux comptes
      distincts », ordre des refus avec l'exercice, identifiant d'une autre société, les **trois**
      tests de place du verrou — deux vers l'exercice, un vers l'arrondi), `kesh-api` (`company_invoice_settings_postable_e2e.rs`, `inbox_import_e2e.rs`),
      Vitest (réglages, écran de validation) ; mutations consignées.
- [ ] **T6 — Manuel, API, CHANGELOG** (AC8, AC9) ; PDF régénérés et contrôlés aplatis (ligatures
      normalisées) ; **grep du symptôme** (règle *Propagation post-patch*) :
      `grep -rnE "reprend les comptes de la facture|sous-compte imputable|inverse exact|n'est pas exposé|aucun autre chemin ne prend de verrou|aucun cycle n'est|#429" docs/manual/fr/*.tex docs/api-external.md crates frontend/src CHANGELOG.md`
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

**Risque d'interblocage — l'ordre des verrous** (réécrit en P2, findings F2-1 HIGH = R2-1 et F2-8,
choix **C43** ; **déplacé en 15-5e** au découpage, choix C52/C53 ; **règle révisée par C54**). La
règle est celle de la 15-5e1 (`15-5e1-socle-rejeu.md`) : **l'ordre des verrous réduit
la fréquence des interblocages ; le rejeu des routes les rend invisibles.** Aucun ordre « comptes
avant exercice » ne tient de bout en bout, l'insertion des lignes reprenant chaque compte écrit par la
clé étrangère `fk_jel_account` après l'exercice. Ce qui revient à **cette** story, et seulement à
elle :

| flux | ce que la 15-5d ajoute | sa place |
|---|---|---|
| validation d'une facture | **créance + TVA due** (`ORDER BY id`, accesseur) | après l'arrondi (`invoices.rs:2065`), avant l'exercice (`:2172`) |
| saisie / complétion fournisseur | **créanciers + TVA récupérable** (`ORDER BY id`, accesseur) | après les comptes de charge (`supplier_invoices.rs:300-350`), avant l'exercice (`:352-355`) — réglages déjà avancés par la 15-5e1 |

*(Lignes relevées sur `f8a569a7`, avant les 15-5a, 15-5b et 15-5e : T0 les refait.)*

Les interblocages dont **la place de l'accesseur réduit la fréquence** : (i) validation ↔ solde du
reste sur la TVA due (F2-1, test 1 de l'AC7 — le solde verrouille la TVA due avant l'exercice,
`invoice_settlements_write.rs:487-491`) ; (ii) saisie fournisseur ↔ règlement fournisseur sur le
compte créanciers (test 2) ; (iii) validation ↔ règlement client dont le compte interne est la
créance ou la TVA due (R3-1 = F3-1, test 3 pour la place après l'arrondi). **Aucun n'est exclu** :
chacun peut encore naître de la reprise des comptes par `fk_jel_account`, et c'est le rejeu des routes
(15-5e1, 15-5e2) qui le rend invisible. Les comptes de charge (type `Expense`) et les comptes désignés côté
achat (passif, actif) sont disjoints par type. **Entre les comptes d'une même requête**, l'ordre est
celui des identifiants (`ORDER BY id`, finding R1-6 de la P1). `accounts::create` (qui rend le parent
non imputable) ne prend aucun verrou sur les réglages ni sur les factures (vérifié en P4 de la 15-5b,
lentille F).

**L'inversion préexistante réglages / exercice** (finding F2-8, relevée en P2) : levée par la
**15-5e1** (son AC5), qui avance les réglages de la saisie fournisseur avant les comptes de charge.
Cette story en a besoin — pour verrouiller les candidats avant l'exercice, la saisie doit connaître les
réglages avant l'exercice — et la suppose en place (T0).

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
- Le **rejeu sur interblocage** de toutes les routes qui écrivent au journal, et ce qui reste de
  l'ordre des verrous des flux existants : les **15-5e1** et **15-5e2** (C52, réécrite selon C54,
  découpée par C61). Les inversions **exercice puis arrondi**, préexistantes — le lot de
  rapprochement et l'avoir d'une facture arrondie — y sont couvertes par le rejeu (**#536**, fermée
  par la 15-5e2).

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
- **C42** — révision de C39 : l'avoir n'appelle pas les générateurs de la garde ; il a le sien.
- **C43** — l'ordre des verrous : tous les candidats verrouillés avant l'exercice, contrôle des seuls
  rôles écrits après la génération ; côté achat, les réglages avancés avant les comptes de charge.
- **C44** — `DesignatedRole`, type neuf plutôt qu'`AccountRole`.
- **C45** — les contournements E2E restent (le seed ne désigne pas le compte créanciers) ; le seed
  n'est pas étendu (révise C34).
- **C46** — le dialogue de validation se ferme sur `ACCOUNT_NOT_POSTABLE`.
- **C47** — le signal de découpage D5 de la P2 (MEDIUM → HIGH, recyclé) ne la découpe pas ; « si la
  P3 recycle encore, découpage » (voir le Change Log de la P3).
- **C48** — « l'arrondi d'abord » : le règlement client par compte interne et le solde du reste
  verrouillent le compte d'arrondi avant leur autre compte (révise C43, qui les disait conformes) ;
  appliqué par la **15-5e** depuis le découpage (C52), puis **retiré** par sa réécriture (C54, C55).
- **C49** — l'accesseur refuse lui-même un compte absent ou inactif (`InactiveOrInvalidAccounts`),
  avant le non imputable.
- **C50** — la signature des générateurs : `GeneratedLines { lines, roles }`.
- **C51** — un identifiant d'une autre société : test de non-verrouillage, `owned_account_ids` si
  besoin, `EXPLAIN` relevé.
- **C52** — la clause de C47 joue : l'ordre des verrous des règlements sort en 15-5e, qui passe avant.
- **C53** — la ligne de partage : la 15-5e réordonne les flux existants et porte leurs tests ; la
  15-5d garde la place de son accesseur et les trois tests qui la figent.
- **C54** — la défense contre l'interblocage est le rejeu (15-5e réécrite) ; les verrous de cette
  story restent, ses affirmations d'absence de cycle sont retirées.
- **C65** — cette story dépend de la 15-5e1 seule ; le rejeu de la saisie fournisseur et de la
  complétion d'import vient avec la 15-5e2, ordre de merge libre.

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/{company_invoice_settings,invoices,supplier_invoices,invoice_settlements_write}.rs`
(le dernier pour une phrase du commentaire « 5 bis », AC9 — réécrit en place par la 15-5e1 ;
`invoices.rs`, `supplier_invoices.rs` et `credit_notes.rs` aussi pour les 19 tests des générateurs,
C50),
`crates/kesh-db/src/errors.rs` (variante), `crates/kesh-api/src/errors.rs` (bras),
`crates/kesh-i18n/locales/*/messages.ftl` (deux clés : `error-designated-account-not-postable`,
`settings-invoicing-payable-account`), `crates/kesh-i18n/src/loader.rs` (sélecteur),
`frontend/src/routes/(app)/settings/invoicing/{+page.svelte,settings-invoicing-page.test.ts}`,
`frontend/src/lib/features/invoices/invoices.types.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`
(borne), `frontend/tests/e2e/{payment-batches,inbox-import,supplier-invoices}.spec.ts` (AC6 :
commentaires des contournements), `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (AC4 :
fermeture du dialogue, C46) et, au besoin, les autres écrans de facture (AC4, si un `catch` est en
défaut), tests
(`crates/kesh-db/tests/{invoices_validate_vat,supplier_invoices_repository,invoice_write_off}.rs`,
`crates/kesh-db/tests/invoice_amount_due_parity.rs` (test 3 de place du verrou), et `crates/kesh-db/tests/credit_notes_repository.rs` pour le test de C35 ; `crates/kesh-api/tests/{company_invoice_settings_postable_e2e,inbox_import_e2e}.rs` ;
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
  `15-5b-gardes-surfaces-neuves.md`, `15-5e1-socle-rejeu.md` (le rejeu de la validation, l'avance des réglages, le doc-comment canonique —
  dépendance), `15-5e2-rejeu-des-autres-flux.md` (le rejeu des autres routes, dont la saisie
  fournisseur — sans dépendance), `15-5e-ordre-des-verrous-reglements.md` (index, `split`) ;
  `14-3b-consommateurs-roles.md` (D-A0, L2).
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
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-5d-validate-prompt-p2.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P1, **F**
  adversaire de périmètre complet ; rapports `target/gate-logs/15-5d-p2-{R,F}.md`). Bruts : R
  3 MEDIUM + 5 LOW (R2-1 à R2-8), F 1 HIGH + 1 MEDIUM + 6 LOW (F2-1 à F2-8), soit 16. Doublons
  inter-lentilles : F2-1 = R2-1 (retenu HIGH), F2-3 = R2-2 (retenu MEDIUM), F2-5 = R2-6 → **1 HIGH,
  3 MEDIUM, 9 LOW distincts** (recompté : 16 bruts − 3 fusions = 13). Trend : **P1 4 MEDIUM / 7 LOW
  → P2 1 HIGH / 3 MEDIUM / 9 LOW**.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | F2-1 = R2-1 | HIGH | la garde verrouille la TVA due **après** l'exercice, le solde du reste **avant** : cycle neuf (1213 → 500) ; Dev Notes et `invoice_settlements_write.rs:463-470` affirmaient l'inverse | AC1 : accesseur en deux temps — **tous les candidats verrouillés avant l'exercice** (`ORDER BY id`), contrôle des rôles écrits après la génération (C39 inchangé) ; côté achat, réglages avancés avant les comptes de charge ; ordre des refus réécrit (AC1) ; AC7 : test « exercice absent + compte non imputable », test de non-interblocage à deux connexions, mutation « verrou après l'exercice » ; AC9 : doc-comment « Ordre des locks » (`invoices.rs:1916-1925`) et commentaire « 5 bis » ; Dev Notes réécrites (table des quatre flux) (**C43**) |
  | R2-2 = F2-3 | MEDIUM | `credit_notes.rs:933`, cité comme l'avoir appelant du générateur, est un test ; l'avoir a son propre générateur | AC1 : ventilation des 25 occurrences (3 définitions, 1 transmission, 2 appels de production, 19 tests) ; `generate_credit_note_journal_lines` (`credit_notes.rs:187`) nommé ; révision de C39 (**C42**) |
  | R2-3 | MEDIUM | AC6 : le seed E2E (`seed_accounting_company`) ne désigne pas le compte créanciers ; le motif « contournement de #521 » est faux | AC6 réécrit : contournements gardés, commentaire corrigé, seed non étendu (**C45**, révise C34) |
  | F2-2 | MEDIUM | AC6 : un troisième contournement (`supplier-invoices.spec.ts:86`) | AC6, T0, T4 : inventaire par commande, trois sites ; fichier ajouté aux fichiers touchés |
  | R2-4 | LOW | C34 parle encore de « specs rejouées » | C45 (le registre ne réécrit pas C34) |
  | R2-5 | LOW | AC4 : `import/+page.svelte:267` / `:301` faux ; `:213` est le `catch` | `:261`, `:300`, `:213-214` (AC4, AC7) |
  | R2-6 = F2-5 | LOW | AC5 : « section des comptes par défaut, à côté de la TVA due » désigne deux sections | AC5 : section *Comptes par défaut* (`:296-342`), après la créance (`:300-314`) ; `:70-72`, `:123-145` |
  | R2-7 | LOW | `test_fixtures.rs:158-169` cité pour un `UPDATE` et un double rôle qu'il ne fait pas | AC1, AC7 : `supplier_invoices_repository.rs:49-55` |
  | R2-8 | LOW | mutation « retirer chacun des quatre contrôles » ambiguë sous C39 | AC7 : par rôle (ne pas l'insérer), par flux (retirer l'appel du contrôle), et le verrou déplacé |
  | F2-4 | LOW | `DesignatedRole` double quatre variantes d'`AccountRole` | AC1 : type neuf justifié (champs des réglages, `match` sans bras mort, `Ord`) (**C44**) |
  | F2-6 | LOW | le dialogue de validation reste ouvert sur `ACCOUNT_NOT_POSTABLE` | AC4 : le code ferme le dialogue (`:428-434`) ; AC7 : asserté, mutation ; T3 (**C46**) |
  | F2-7 | LOW | `docs/api-external.md:281` : « TVA due absente » alors que le solde du reste refuse aussi un compte inutilisable | AC9 : « absent ou inutilisable (archivé, non imputable) » |
  | F2-8 | LOW | inversion préexistante réglages / exercice entre validation et saisie fournisseur | **fermée** par la remédiation de F2-1 (réglages avancés côté achat) — Dev Notes ; **aucune KF** à ouvrir |

  **Signal de la règle de découpage** (amendement D5) : la sévérité **monte** (MEDIUM → HIGH). Le HIGH
  n'est pas un défaut d'origine neuf : il vient de la **place** de la garde, fixée par la remédiation
  de la P4 de la 15-5b (R4-3 / F4-5), et R2-2 naît de la remédiation P1 (texte de C39) — c'est du
  **recyclage** au sens de l'amendement. **Signal levé et déclaré** au Project Lead par
  l'orchestrateur, à qui revient l'arbitrage ; la remédiation ne propose pas de découpage : le défaut
  porte sur l'unique règle métier de la story (sa place dans la transaction), qu'aucune sous-story ne
  séparerait de son ordre de verrous. Décompte des modules inchangé (`invoice_settlements_write.rs`
  relève de `kesh-db`, l'écran de validation du `frontend`).
  **Décisions de l'orchestrateur** : ordre des verrous (C43, mécanisme imposé par l'orchestrateur) ;
  C42, C44, C45, C46 consignés pendant la remédiation.
  **Propagation post-patch** : `après ces verrous`, `check_designated_accounts_postable_in_tx`,
  `credit_notes.rs:933`, `ignore les rôles`, `ignorent les rôles`, `test_fixtures.rs:158-169`,
  `import/+page.svelte:267`, `` `:301` ``, `rejouées`, `retirés s'ils`, `le seed E2E le`, `à côté de la
  TVA due`, `:355-369`, `deux specs`, `aucun autre chemin ne prend` grepés sur les fiches 15-5a
  (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml`. Restent : la ventilation
  (qui nomme `credit_notes.rs:933` comme test), le commentaire de code cité pour être réécrit (AC9, et
  ajouté au grep de T6), les Change Logs (historiques), et dans le registre C34 et C39, non réécrits
  par règle et révisés par C45 et C42. La 15-5b (`:359`) dit que « la 15-5d décide de leur retrait
  (son AC6) » : toujours exact (elle décide de les garder), laissé tel quel — fiche close en
  validation.
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P3 suit** (un HIGH et des
  MEDIUM en P2), complète (**Sonnet**, rotation D6) : la remédiation change l'ordre des verrous de
  deux flux et le flux fournisseur.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-5d-validate-prompt-p3.md` ; deux
  lentilles **Sonnet** en contexte frais, rotation D6 : **R** chasseur de régressions de la
  remédiation P2 (commit `34b308c4`), **F** adversaire de périmètre complet ; rapports
  `target/gate-logs/15-5d-p3-{R,F}.md`). Bruts : R 1 MEDIUM + 3 LOW (R3-1 à R3-4), F 2 MEDIUM + 4 LOW
  (F3-1 à F3-6), soit 10. Doublon inter-lentilles : R3-1 = F3-1 → **2 MEDIUM, 7 LOW distincts**
  (recompté : 10 bruts − 1 fusion = 9 = 2 + 7). Trend : **P1 4 MEDIUM / 7 LOW → P2 1 HIGH / 3 MEDIUM /
  9 LOW → P3 2 MEDIUM / 7 LOW**.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | R3-1 = F3-1 | MEDIUM | le règlement client par compte interne verrouille le compte, **puis** l'arrondi, puis l'exercice ; la validation, l'arrondi puis la créance et la TVA due : cycle neuf si le compte interne est la créance ou la TVA due ; la table des flux omettait l'étape d'arrondi et C43 disait ce flux conforme | AC1 : règle « l'arrondi d'abord » — le règlement client classe le paiement avant l'étape (3) et prend l'arrondi avant le compte interne ; **le solde du reste aussi** (arrondi avant la nature), faute de quoi le réordonnancement ouvrait le cycle symétrique ; ordres des refus écrits ; AC7 : test 3 de non-interblocage (compte interne = TVA due, sonde `NOWAIT`), tests d'ordre des refus, mutation ; Dev Notes : table des flux réécrite (étape d'arrondi, rapprochement, avoir) ; AC9 : commentaire « 5 bis » réécrit et déplacé (**C48**, révise C43) |
  | F3-2 | MEDIUM | le test de non-interblocage ne couvre que la vente ; le déplacement du verrou côté achat ne rougit rien | AC7 : test 2 (règlement fournisseur par compte interne = compte créanciers ↔ saisie fournisseur à TVA) ; une mutation d'ordre **par flux** |
  | F3-3 | LOW | le lot de rapprochement prend l'exercice puis l'arrondi (préexistant) ; la fiche et le commentaire réécrit affirmaient l'absence de cycle | table des flux et AC1 : ligne « rapprochement », renvoi à **#536** (ouverte par l'orchestrateur) ; le commentaire réécrit ne prétend plus à l'absence de cycle (AC9) ; motif `aucun cycle n'est` ajouté au grep de T6 |
  | F3-4 | LOW | l'archivage était laissé au contrôle non verrouillant de `create_in_tx`, qui lit un instantané antérieur au verrou | AC1 : l'accesseur refuse lui-même un compte absent ou inactif (`InactiveOrInvalidAccounts`), puis le non imputable ; ordre écrit ; AC7 : limite du test « archivé » écrite (**C49**) |
  | F3-5 | LOW | `WHERE company_id = ? AND id IN (…) FOR UPDATE` : un identifiant étranger peut être verrouillé (mesuré à `opening_complement.rs:431-440`) | AC1 : `EXPLAIN` relevé, patron `owned_account_ids` si la mesure le demande ; AC7 : test d'un identifiant d'une autre société, sonde `NOWAIT` (**C51**) |
  | F3-6 | LOW | signature des générateurs non fixée, 19 tests en dépendent | AC1 : `GeneratedLines { lines, roles }`, vente `Result<GeneratedLines>`, achat `Result<(GeneratedLines, Decimal)>`, 19 tests adaptés mécaniquement par `.lines` ; T1 (**C50**) |
  | R3-2 | LOW | place du chargement des réglages côté achat non fixée | AC1 : après (0) projet et (1) fournisseur, avant (2) — ordre projet → réglages identique des deux côtés ; table |
  | R3-3 | LOW | en-tête des choix incomplet (C42, C45 seulement « révisé par » ; C47 absent) | en-tête et « Décisions consignées » complétés (C42 à C51) |
  | R3-4 | LOW | montage du test de non-interblocage sous-spécifié (compte d'escompte, facture validée avec TVA et reste dû) | AC7 : test 1 dans `invoice_write_off.rs`, montage `company()` (`:35`, escompte et arrondi désignés) et `validated_invoice()` (`:57`) ; facture B par le vrai chemin |

  **Relevé hors des rapports, pendant la remédiation** : le grep des sites qui verrouillent le compte
  d'arrondi (`rounding_account_for_write|write_off_account_for_write`) a trouvé un cinquième site,
  l'**avoir** d'une facture arrondie (`credit_notes.rs:370` exercice, puis `:525` arrondi) — même
  inversion que le rapprochement, préexistante, hors périmètre (l'avoir est exempté, C35). Écrit à la
  table et à l'AC1 comme **à tracer** ; **signalé à l'orchestrateur** (issue à ouvrir, ou #536 à
  étendre).
  **Signal de la règle de découpage** (amendement D5) : la sévérité **baisse** (HIGH → MEDIUM) : le
  signal « égale ou supérieure » n'est pas levé. Mais C47 a posé une condition propre — « si la P3
  recycle encore, découpage » —, et la réponse est **oui, en partie** : **F3-2** (MEDIUM) naît de la
  remédiation P2 (le changement d'ordre côté achat, introduit par C43, sans test qui le fige) ;
  **R3-1 = F3-1** (MEDIUM) ne naît **pas** d'elle — le cycle tient à la garde elle-même (verrouiller
  la créance ou la TVA due après le compte d'arrondi), il existait sous la place de la P1 comme sous
  celle de la P2 —, mais c'est la remédiation P2 qui en a **certifié l'absence** (C43, table, AC9) ;
  parmi les LOW, F3-3 et R3-2 portent sur du texte de la P2, F3-4 sur l'argument (a) de C43, R3-3 sur
  C47, R3-4 sur le test de la P2 ; F3-5 et F3-6 remontent à la P1. **Déclaré à l'orchestrateur, à
  qui revient l'arbitrage** au regard de C47 ; la remédiation ne propose pas de découpage (le défaut
  porte toujours sur l'ordre des verrous de l'unique règle métier). Décompte des modules inchangé
  (`invoice_settlements_write.rs`, désormais modifié pour son code, relève de `kesh-db`).
  **Décisions** : C48 (décision de l'orchestrateur, étendue au solde du reste pendant la
  remédiation), C49, C50, C51 (décisions de l'orchestrateur, consignées).
  **Propagation post-patch** : `aucun cycle connu`, `aucun cycle n'est`, `le suivent aussi`, `comme le
  règlement client`, `suit le chemin d'aujourd'hui`, `:195-197`, `quatre flux`, `test de
  non-interblocage rougit`, `ou le fichier des tests de solde`, `ConfigurationRequired` (pour les
  refus d'arrondi et de nature, qui sont `RoundingAccountNotConfigured` et
  `WriteOffAccountNotConfigured`) grepés sur les fiches 15-5 (mère), 15-5b, 15-5c, 15-5d, le registre
  et `sprint-status.yaml`. Restent : C43 (registre, « les règlements par compte interne le suivent
  aussi »), non réécrit par règle et révisé par C48 ; le commentaire de code cité pour être réécrit
  (AC9) ; les Change Logs (historiques). Les numéros de ligne cités par F (`:195-197` pour le
  classement du paiement) sont faux : `:175-176`, corrigé.
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P4 suit** (des MEDIUM en P3),
  complète (**Opus**, rotation D6) : la remédiation touche l'ordre de deux flux de plus (règlement
  client, solde du reste), donc du code de production d'un autre module de flux — la passe ciblée ne
  suffit pas.
- 2026-10-08 — **Découpage après la validation P3** (choix **C52** de l'orchestrateur, ligne de partage
  **C53**). La clause de C47 (« si la P3 recycle encore, découpage ») a joué : F3-2 naît de la
  remédiation P2, et la remédiation P3 a dû étendre le réordonnancement à deux flux de production de
  plus. Le réalignement des flux **existants** sur l'ordre canonique des verrous sort en
  **15-5e** (`15-5e-ordre-des-verrous-reglements.md`, `refs #429`), qui passe **avant** cette story.
  **Sorti de cette fiche** : la règle « l'arrondi d'abord » appliquée au règlement client par compte
  interne et au solde du reste (C48), avec leurs ordres de refus ; l'avance des réglages de la saisie
  fournisseur avant les comptes de charge (C43) ; la table des flux et les inversions préexistantes
  (#536, avoir) ; le test d'ordre des refus des flux réordonnés ; le test de non-interblocage 3
  (règlement client ↔ validation) ; les mutations de ces flux ; le commentaire de l'étape (3) de
  `settle_invoice` ; la réécriture et le déplacement du commentaire « 5 bis ». **Gardé ici** : la garde
  à l'usage (accesseur en deux temps, C39/C42/C49), `GeneratedLines` (C50), le test d'une autre
  société (C51), l'écran du compte créanciers, l'avoir exempté, le manuel, `docs/api-external.md`, le
  CHANGELOG, `closes #429` ; la **place** de l'accesseur (après l'arrondi ou les comptes de charge,
  avant l'exercice) et les tests qui la figent — 1 et 2, inchangés, et un **test 3 neuf** (sonde : la
  validation tient l'arrondi sans tenir la créance ni la TVA due), seul à rougir si l'accesseur passe
  avant l'arrondi ; la phrase que le commentaire « 5 bis » gagne (la validation verrouille désormais la
  TVA due avant l'exercice) et l'étape 1 bis du doc-comment canonique. Dépendance ajoutée : 15-5e ; T0
  vérifie que son ordre est en place. Les findings de la P3 relatifs à l'ordre (R3-1 = F3-1, F3-2,
  F3-3, R3-2) sont repris au Change Log de la 15-5e.
  **Propagation post-patch** : `settle_invoice`, `write_off_invoice`, `RoundingAccountNotConfigured`,
  `WriteOffAccountNotConfigured`, `test 3`, `trois flux`, `arrondi d'abord`, `C48`, `5 bis`,
  `1916-1925`, `réordonn`, `invoice_settlement.rs`, `f15e5fbf` grepés sur le corps de cette fiche
  (hors Change Log) ; les occurrences restantes sont des renvois à la 15-5e ou la place de
  l'accesseur. Décompte : **9 AC, 8 tâches T0–T7** (recompté). **Validation** : une passe ciblée sur
  le découpage (le corps a perdu un pan et gagné un test), après celle de la 15-5e.
- 2026-10-08 — **Alignement sur C54** (réécriture de la 15-5e : « Rejeu sur interblocage des flux
  d'écriture »). **Rien de la garde ni de ses verrous ne change** : accesseur en deux temps, places
  (après l'arrondi ou les comptes de charge, avant l'exercice), refus, tests de refus. Changé :
  (1) la dépendance à la 15-5e décrit ce qu'elle livre désormais — le rejeu des routes, l'avance des
  réglages de la saisie fournisseur (son AC5, ex-AC4), les commentaires réécrits ; « l'arrondi
  d'abord » du règlement client et du solde du reste n'existe plus ; (2) l'AC1 « l'ordre des
  verrous » et les Dev Notes ne prétendent plus fermer de cycle : les places **réduisent la
  fréquence**, la reprise des comptes par `fk_jel_account` après l'exercice (finding F1-1 de la P1
  de la 15-5e) laisse des cycles possibles, que le rejeu rend invisibles ; « Pourquoi après
  l'arrondi » ne s'appuie plus sur l'ancien AC2 de la 15-5e ; (3) les trois tests « de
  non-interblocage » deviennent des tests **de place du verrou** : le critère reste l'attente vue sur
  l'accesseur (ou les sondes du test 3) ; l'assertion « les deux tâches réussissent » des tests 1 et 2
  est remplacée par « succès ou 1213, jamais une autre erreur » — au niveau du dépôt, sans rejeu, une
  1213 y est possible ; le renvoi au test de concurrence 1 de la 15-5e (retiré) est supprimé ; (4) le
  commentaire « 5 bis » est réécrit **en place** par la 15-5e (son AC5) ; (5) T0 vérifie le rejeu des
  routes de validation, de saisie et de complétion ; (6) C54 ajouté aux décisions. **Propagation
  post-patch** : `non-interblocage`, `aucun cycle`, `cycle (iii) est fermé`, `les deux réussissent`,
  `les deux tâches réussissent`, `son AC4`, `son AC3`, `son AC2`, `son AC6`, `concurrence 1`,
  `que la 15-5e prévient` grepés sur le corps de la fiche (hors Change Log) : les occurrences
  restantes sont l'AC4 de la **15-5a** ou de la **15-5b** (pas de la 15-5e) et le rappel « que plus
  rien n'affirme ». Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). La passe ciblée sur le
  découpage, encore à lancer, porte aussi sur cet alignement.
- 2026-10-08 — **Alignement sur la P2 de la 15-5e** (finding R2-1 de cette passe) : la 15-5e fixe,
  à son AC5, la numérotation du doc-comment canonique de `validate_invoice` sur les étapes du code
  — (1) facture, (1 bis) projet lu sans verrou, (2) réglages, (2 bis') compte d'arrondi, (2 quater)
  matérialisation du compte de produit, (3) exercice, … L'étape « 1 bis. `accounts` » que les
  points de l'AC1 (validation, « le verrou ») et de l'AC9 étendaient n'existe plus : ils renvoient à
  l'étape **(2 bis')**. Aucun changement de fond (place des verrous inchangée : après l'arrondi,
  avant l'exercice). Propagation : `1 bis` grepé sur la fiche — seule la mention historique du
  Change Log reste. Décompte inchangé : 9 AC, 8 tâches T0–T7.
- 2026-10-08 — **Alignement sur le découpage de la 15-5e** (choix **C61**, **C65** ; finding F3-2 de
  la P3 de la 15-5e) : la 15-5e est découpée en **15-5e1** (socle : enveloppes, registre, rejeu de la
  validation, du règlement client et de son annulation ; avance des réglages de la saisie
  fournisseur, doc-comment canonique, « 5 bis ») et **15-5e2** (rollout : les autres routes, dont la
  saisie fournisseur et la complétion d'import ; Pattern 5, manuels). Cette story **dépend de la
  15-5e1 seule** : en-tête, § des dépendances, T0 (vérifie la 15-5e1, relève sans l'exiger la
  15-5e2), renvois de l'AC1, de l'AC7, de l'AC9, des Dev Notes et des References ventilés vers la
  15-5e1 (contenu dont elle dépend) ou les 15-5e1 / 15-5e2 (rejeu des routes) ; la fenêtre
  « saisie fournisseur verrouillée sans rejeu » si cette story merge avant la 15-5e2 est écrite.
  Aucun changement de fond. Propagation : `15-5e` grepé sur le corps (hors Change Log) — les
  occurrences restantes sont historiques (C48, C52, C53, C54, finding F1-1 « de la P1 de la 15-5e »,
  relevé « avant les 15-5a, 15-5b et 15-5e ») ou le nom de la fiche index. Décompte inchangé :
  9 AC, 8 tâches T0–T7.
