# Story 15.6c : La configuration ne prépare plus l'écriture nulle

## Status

review

<!-- Spécifiée le 2026-10-08 en autonomie (bmad-create-story), fille de la 15-6 découpée d'emblée
     (choix C-15-6-1). Choix propres : C-15-6-5 (révisé par C-15-6-13), C-15-6-14, C-15-6-15.
     Validation P1 (Sonnet, lentilles F et R) appliquée le 2026-10-08 : contrôles dans la transaction,
     sérialisation des deux gestes, écrans filtrés sur les réglages. Validation P2 (Opus, lentilles F
     et R) appliquée le 2026-10-08 : `LOCK IN SHARE MODE`, 409 avant le refus des réglages, code de la
     route des réglages interne, tests étendus sur ceux de la 15-5b (C-15-6-21, C-15-6-22).
     Validation P3 (Sonnet, lentilles R et F) appliquée le 2026-10-08 : test du `FOR UPDATE` de
     `before` (12 bis), attente par `attendre_une_requete_en_cours`, place du paragraphe au manuel
     d'administration, portée réelle des verrous, pas de verrou pour une dé-liaison (C-15-6-27).
     Validation P4 ciblée (Haiku) le 2026-10-08 : 0 finding, vérifié par l'orchestrateur —
     validation close. -->

**Issue** : `closes #474` (P1, avec la 15-6b) — mot-clé **dans la PR**. **Mère** :
`15-6-creance-juste-avoir-reglement.md`.

⛔ **Ne commence qu'après le merge de la 15-5b**, qui modifie les **mêmes fonctions** :
`validate_journal_account_id` (`crates/kesh-api/src/routes/bank_accounts.rs:260`, paramètre
`require_postable` ajouté), `create_bank_account` (`:384`), `update_bank_account` (`:498`),
`patch_bank_account_journal_link` (`:717`), le dépôt `crates/kesh-db/src/repositories/bank_accounts.rs`
(`set_journal_account_id_for_company` `:265`, `update_for_company` `:356`, contrôle « dans la
transaction » de son choix C10), le PUT des réglages (`company_invoice_settings.rs`, ses AC10, AC11 et
AC19) et l'écran des comptes bancaires (son AC13 : `withCurrentAccount` sur les deux `<select>`,
`BankAccountJournalLinkForm` alimenté par la liste complète). Les numéros de ligne ci-dessous sont ceux
du commit `1920381e` : **les relire après rebase**. La 15-5b crée aussi les deux fichiers de test que
cette fiche **étend** (`frontend/src/routes/(app)/bank-accounts/+page.test.ts`,
`crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`). **Après la 15-5d** (même epic,
mergée avant) : elle expose `defaultPayableAccountId` à l'écran des réglages (son AC5, choix C34) — le
compte créanciers y devient **corrigeable**, et son menu doit être filtré comme celui des débiteurs
(AC8). **Après** la 15-6b aussi : `ClaimSide`, les fonctions d'écran de `account-options.ts` (choix
C-15-6-14), `messages.ftl` et `errors.rs` communs ; la garde à l'usage de la 15-6b reste le filet des
données antérieures.

## Story

En tant qu'**administrateur**,
je veux que **Kesh refuse de lier un compte bancaire au compte débiteurs ou créanciers, et de désigner
comme compte débiteurs ou créanciers un compte lié à un compte bancaire**,
afin que **chaque encaissement ou paiement par ce compte bancaire ne produise pas une écriture nulle**
— et que l'erreur soit arrêtée à la configuration, une fois, plutôt qu'à chaque règlement.

## Le défaut, établi au code

- `validate_journal_account_id` (`bank_accounts.rs:260-294`) n'exige que : existe, de la société,
  actif, **actif ou passif** — donc 1100 (actif) et 2000 (passif) passent. Les trois routes l'appellent
  (création `:393`, remplacement `:512`, lien `:727`).
- Le PUT des réglages (`company_invoice_settings.rs:246`) valide le compte débiteurs (`:274-281`, actif)
  et créanciers (`:319-326`, passif) par type et état, sans regarder les comptes bancaires. Il lit la
  valeur en place **hors transaction** (`get_or_create_default(&state.pool, …)`, `:256`) ; la seule
  transaction est celle du dépôt, `company_invoice_settings::update` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:147`),
  qui relit `before` par un `SELECT` **non verrouillant** (`:163-169`).
- Une fois le lien posé, la 15-6b refuse chaque règlement et chaque rapprochement de facture sur ce
  compte bancaire — juste, mais tardif et répété.

## Acceptance Criteria

1. **AC1 — Deux refus dédiés** (choix C-15-6-5), patron de l'AC1–AC2 de la 15-6b :
   - `DbError::BankAccountLedgerIsClaimAccount { account_id: i64, account_number: Option<String>, claim: ClaimSide }`
     → code `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, **400**, `details = { "accountId", "accountNumber", "claim": "receivable" | "payable" }`.
   - `DbError::ClaimAccountLinkedToBankAccount { account_id: i64, account_number: Option<String>, claim: ClaimSide, bank_account_id: i64, bank_name: String }`
     → code `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`, **400**, `details = { "accountId", "accountNumber", "claim", "bankAccountId", "bankName" }`.
   `ClaimSide` est celui de la 15-6b. Mapping dans `crates/kesh-api/src/errors.rs` (`details` en
   camelCase). **Une clé par refus et par `claim`** (quatre clés), chacune nommant le bon compte —
   pas de « débiteurs (ou créanciers) » générique :
   - `error-bank-account-ledger-is-receivable` — fr : « Le compte { $account } est le compte
     débiteurs désigné dans Paramètres → Facturation : un compte bancaire doit être lié à son propre
     compte de banque. »
   - `error-bank-account-ledger-is-payable` — fr : même phrase, « compte créanciers ».
   - `error-claim-account-linked-to-bank-account-receivable` — fr : « Le compte { $account } est lié
     au compte bancaire { $bank } : il ne peut pas servir de compte débiteurs. »
   - `error-claim-account-linked-to-bank-account-payable` — fr : même phrase, « compte créanciers ».
   **Arguments des messages** (choix C-15-6-27, patron de l'AC2 de la 15-6b) : `$account` = le
   **numéro** du compte (`account_number`), repli `#<id>` quand il manque ; `$bank` = `bank_name`, tel
   que le rend `first_active_bank_account_linked_to` (AC2). **Source de `account_number`** : aucun des
   deux lecteurs ne le porte (`ClaimAccounts` n'a que des ids, le lecteur bancaire rend
   `(id, bank_name)`) ; la garde, **une fois le refus décidé**, le lit par
   `SELECT number FROM accounts WHERE id = ? AND company_id = ?`, dans la même transaction, **sans
   verrou** (le refus ne dépend plus de cette lecture) ; ligne absente → `None` → repli `#<id>`.
   Dans les **quatre** locales ; `npm run lint-i18n-ownership` vert. **Destinataires** :
   `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` sert l'écran **et les intégrateurs** (les routes
   `/api/v1/bank-accounts` sont ouvertes aux clés d'API, `comptable_routes`) ;
   `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` ne sert **que l'écran** et un client de **session** :
   `PUT /api/v1/company/invoice-settings` est une route d'administration **fermée aux clés**
   (`admin_routes`, couche `require_not_pat`, `crates/kesh-api/src/lib.rs:339-341` ; figé par
   `crates/kesh-api/tests/admin_pat_denied_e2e.rs:64`). ⚠️ Sur la route des réglages, ce refus est
   **typé et traduit**, alors que les autres refus de cette route sont des `AppError::Validation` en
   français en dur (limite écrite par la 15-5b, choix C3) : la différence est voulue — uniformité avec
   son symétrique bancaire, et des `details` qui nomment le compte bancaire à l'écran — et le
   doc-comment de la route la dit.
2. **AC2 — Deux lecteurs partagés, et un ordre de verrous unique** (choix C-15-6-13, syntaxe
   corrigée par C-15-6-21 ; DRY : chaque requête n'existe qu'une fois).
   - `company_invoice_settings::claim_accounts_in_share_mode(tx, company_id) -> Result<ClaimAccounts, DbError>`
     (`ClaimAccounts { receivable: Option<i64>, payable: Option<i64> }`, `Default` = deux `None`) :
     `SELECT default_receivable_account_id, default_payable_account_id FROM company_invoice_settings WHERE company_id = ? LOCK IN SHARE MODE` ;
     ligne absente → deux `None`.
   - `bank_accounts::first_active_bank_account_linked_to(tx, company_id, account_id) -> Result<Option<(i64, String)>, DbError>` :
     `SELECT id, bank_name FROM bank_accounts WHERE company_id = ? AND journal_account_id = ? AND archived = FALSE ORDER BY id LIMIT 1 LOCK IN SHARE MODE`
     — le **premier compte bancaire non archivé par `id`** (index `idx_bank_accounts_journal_account`,
     sur `(journal_account_id)` seul, migration `20260507200001`, l. 27). ⚠️ **Portée réelle du
     verrou** (choix C-15-6-27) : sous REPEATABLE READ, un balayage verrouillant sur un index
     secondaire non unique pose des verrous **next-key / d'intervalle** — y compris quand il ne trouve
     **aucune** ligne. Un `INSERT INTO bank_accounts` dont le `journal_account_id` tombe dans
     l'intervalle voisin (création par la route, `insert`/`insert_in_tx` du dépôt
     `crates/kesh-db/src/repositories/bank_accounts.rs:33`, `:210`, onboarding) peut donc **attendre**
     la fin du PUT des réglages. Pas de cycle (aucun flux ne tient `bank_accounts` puis réclame les
     réglages — vérifié par grep des lectures verrouillantes de `bank_accounts` à la validation P3) ;
     une **attente transitoire**, possible et **non mesurée**. La fiche ne dit plus « étroite ».
   - ⛔ **`FOR SHARE` est une erreur de syntaxe sur MariaDB 10.11** (version de la CI, du dev et de la
     production — `image: mariadb:10.11`) : le verrou partagé s'écrit **`LOCK IN SHARE MODE`**, comme
     partout dans le dépôt (`crates/kesh-db/src/repositories/opening_complement.rs:38-39`, `:502`,
     `:540`). Les requêtes ci-dessus se recopient telles quelles.
   - **Sérialisation des deux gestes** : la ligne des réglages est **toujours verrouillée la
     première**, puis les comptes bancaires. **Ordre global** : `companies` (verrou sentinelle, quand
     il est pris) → réglages → `bank_accounts` — cohérent avec le flux fournisseur (`companies` via
     `projects::validate_taggable_in_tx`, puis réglages `FOR UPDATE` par
     `get_or_create_default_in_tx`, `supplier_invoices.rs:283`, `:359`). Côté compte bancaire,
     `claim_accounts_in_share_mode` est le **premier verrou après la sentinelle** (création et
     remplacement : juste après `acquire_company_sentinel_lock`, lui-même une lecture verrouillante
     `SELECT id FROM companies … FOR UPDATE`, `crates/kesh-db/src/repositories/bank_accounts.rs:588`
     (fonction ; requête `:592`) ; **avant**
     `flip_primary_off_for_company` et avant le `FOR UPDATE` de la ligne ; lien, qui ne prend pas la
     sentinelle : première instruction de sa transaction). ⚠️ **Seulement si la cible est un compte**
     (choix C-15-6-27) : une dé-liaison (`journalAccountId: null`), une création ou un remplacement
     **sans** compte lié ne lisent pas les réglages et ne prennent **aucun** verrou S — le contrôle ne
     joue jamais sur `NULL` (AC4) ; la route passe alors `&ClaimAccounts::default()`. Cela réduit
     l'attente décrite ci-dessous (Dev Notes, *Latence*). Côté réglages,
     `company_invoice_settings::update` lit `before` en **`FOR UPDATE`** (au lieu du `SELECT` simple,
     `:163-169`), puis, après le contrôle de version (AC6), appelle
     `first_active_bank_account_linked_to`. Ainsi un lien et une désignation concurrents ne peuvent
     plus passer tous les deux : le second attend le premier et lit sa valeur validée, puis refuse.
     L'**angle mort « course »** de la spécification initiale est **supprimé**, pas déclaré. Le
     doc-comment des deux lecteurs dit l'ordre et pourquoi (un ordre inverse ouvrirait un
     interblocage entre les deux gestes).
   - **Note KF-004 réécrite** (`:176-180`, « race acceptée v0.1… mitigation future : SELECT FOR
     UPDATE ») : elle devient fausse et dit désormais ce qui est sérialisé — le PUT des réglages
     **contre les gestes bancaires** (lien, création, remplacement) — et ce qui **ne l'est pas** :
     `update` commence par `INSERT IGNORE` (`:157`), qui pose sur une ligne existante un verrou
     **partagé** ; le `FOR UPDATE` qui suit est une **promotion S → X**. Deux écrivains de la ligne qui
     font tous deux `INSERT IGNORE` puis `FOR UPDATE` — deux PUT simultanés, ou un PUT et
     `get_or_create_default_in_tx` (validation de facture, avoir, facture fournisseur) — peuvent
     s'interbloquer (1213, l'un des deux en 500). **Préexistant** (l'`UPDATE` actuel promeut déjà),
     non aggravé, hors périmètre : la note le dit au lieu de promettre l'inverse. *(Comportement
     d'InnoDB documenté, non exécuté à la validation.)*
3. **AC3 — Création d'un compte bancaire** : `journalAccountId` égal à `ClaimAccounts.receivable` ou
   `.payable` → `BankAccountLedgerIsClaimAccount`, contrôlé **dans la transaction** de la route
   (l'`INSERT` y est en ligne, `bank_accounts.rs:~400-445`), après les contrôles hors transaction
   (forme, 404, type, et la postabilité de la 15-5b). Rien n'est créé ; aucun compte principal n'est
   démoté.
4. **AC4 — Remplacement et lien** : même refus, **dans le dépôt**, sous le verrou que
   `update_for_company` et `set_journal_account_id_for_company` posent déjà ; les deux fonctions
   reçoivent un paramètre `claims: &ClaimAccounts` (lu par la route, AC2 ; `ClaimAccounts::default()`
   quand la cible est `NULL`, sans lecture ni verrou). Il ne joue **que si la
   valeur change** (`nouvelle valeur ≠ existing.journal_account_id`) et n'est pas `NULL` — exactement
   la place et l'exemption que la 15-5b donne à son contrôle de postabilité (son choix C10), et
   **après** lui : après le contrôle de version et, pour le lien seulement, après son court-circuit
   no-op (`set_journal_account_id_for_company`, `:302-306` ; `update_for_company`, `:356-411`, n'en a
   pas). Les **appels directs** au dépôt dans les tests passent `&ClaimAccounts::default()` (§ *Tests
   existants qui changent de sens*). Un PUT qui promeut le compte en principal
   et échoue sur ce refus n'a rien démoté. Un compte bancaire **déjà** lié au compte débiteurs (donnée
   antérieure) reste modifiable dans ses autres champs ; seul un nouveau lien vers un compte de
   créance est refusé.
5. **AC5 — Réglages de facturation, symétrique, dans la transaction** :
   `company_invoice_settings::update`, après la lecture verrouillée de `before` (AC2) : si
   `changes.default_receivable_account_id` (resp. `default_payable_account_id`, **valeur résolue**
   par l'AC19 de la 15-5b — absent du corps = valeur en place, donc inchangée) est non `NULL`,
   **diffère de `before`** et est lié à un compte bancaire non archivé →
   `ClaimAccountLinkedToBankAccount` nommant ce compte bancaire. Le compte créanciers est **envoyé
   par l'écran** depuis la 15-5d (son AC5) : son refus est corrigeable là où il est montré. Valeur inchangée : pas de contrôle
   (une configuration antérieure fautive ne bloque pas l'enregistrement des autres réglages — patron C4
   de la 15-5b). La comparaison se fait contre `before`, lu **sous verrou**, et non contre le `current`
   du handler, lu hors transaction.
6. **AC6 — Ordre des erreurs, écrit** dans le doc-comment de **chaque** fonction touchée :
   routes `create_bank_account`, `update_bank_account`, `patch_bank_account_journal_link`,
   `update_invoice_settings` ; dépôt `bank_accounts::update_for_company`,
   `bank_accounts::set_journal_account_id_for_company`, `company_invoice_settings::update` (sept ;
   « quatre » était un décompte sans liste, finding R3-3).
   - **Routes bancaires** (choix C10 de la 15-5b : l'ordre existant est conservé, le refus neuf vient
     en dernier) : forme → 404 / 400 type / 400 non imputable à la création (`validate_journal_account_id`,
     hors transaction) → 404 compte bancaire → 409 version → 400 `ACCOUNT_NOT_POSTABLE` (si changé) →
     **400 `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`** (si changé). La création n'a ni 404 compte
     bancaire ni 409.
   - **Route des réglages** (choix C-15-6-21, qui révise C-15-6-13) : forme et contrôles du handler
     (dont le 400 `VALIDATION_ERROR` non imputable de la 15-5b, qui ne dépendent que du corps et des
     comptes visés) → **409 version** → **400 `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`** →
     court-circuit « no-op ». **Le 409 d'abord** : l'exemption « inchangé » se calcule contre `before`,
     lu sous verrou ; quand la version ne concorde pas, `before` est un état que le client **n'a jamais
     vu** — un formulaire périmé qui renvoie l'ancien compte débiteurs, ou un compte créanciers résolu
     depuis un `current` périmé (AC19 de la 15-5b), serait refusé sur une valeur qu'il ne sait pas
     avoir changée. Le client doit voir l'état courant avant qu'on juge son changement : même ordre
     que les routes bancaires, dont le refus dépend lui aussi de l'état en base.
7. **AC7 — Écran des comptes bancaires** : il filtre sur les **identifiants désignés dans les
   réglages** — pas sur le rôle —, lus par `getInvoiceSettings()` (`frontend/src/lib/features/invoices/invoices.api.ts:99`,
   lisible par tout rôle), avec `withoutAccountIds` de la 15-6b (choix C-15-6-14).
   - `frontend/src/routes/(app)/bank-accounts/+page.svelte` : `linkableAccounts` (`:78-85`) écarte
     `defaultReceivableAccountId` et `defaultPayableAccountId` ; le filtre s'applique **avant**
     `withCurrentAccount` (15-5b AC13), si bien qu'un lien existant vers 1100 reste affiché.
   - `BankAccountJournalLinkForm.svelte` (`frontend/src/lib/features/bank-accounts/`) : après la 15-5b,
     il reçoit la liste **complète** et filtre lui-même — c'est son `eligibleAccounts` (`:44-55`) qui
     écarte les comptes désignés (prop neuve, ex. `claimAccountIds: ReadonlySet<number>`), **avant**
     `withCurrentAccount` (`:60`).
   - **Échec de `getInvoiceSettings()`** (appel neuf dans `reload()`, `+page.svelte:52-70`,
     `Promise.allSettled`) : **pas de filtre** — le menu reste celui de la 15-5b, et le refus serveur
     reste le filet. Pas d'erreur affichée pour ce seul appel.
   - **Second appelant** du formulaire : `frontend/src/lib/features/bank-accounts/BankAccountList.svelte`
     (`:8`, `:86`) n'est importé **nulle part** (`grep -rn "BankAccountList" frontend/src frontend/tests`
     ne rend que des types homonymes) — code mort. La prop `claimAccountIds` est **obligatoire**, et
     `BankAccountList.svelte` est **supprimé** (à revérifier en T0 ; s'il a retrouvé un importeur, il
     reçoit la prop). Choix C-15-6-22.
   - Le refus serveur s'affiche (message traduit) dans les deux formulaires — **vérifié** (lentille F,
     P2 : `err.message` lu dans les deux formulaires bancaires) ; aucun changement.
8. **AC8 — Écran des réglages** (`frontend/src/routes/(app)/settings/invoicing/+page.svelte`) : les
   menus du **compte débiteurs** et du **compte créanciers** (ce dernier posé par la 15-5d, son AC5 :
   `withCurrentAccount(liabilityAccounts, payableId, accounts)`) ne proposent plus les comptes liés à
   un compte bancaire non archivé (`listBankAccounts()`, qui exclut les archivés par défaut — c'est le
   bon ensemble : le serveur ne refuse que le lien à un compte bancaire **non archivé**) — par
   `withoutAccountIds`, **avant** `withCurrentAccount` (`:85-111`), et **seulement** pour ces deux
   menus (`assetAccounts`, `:65`, et `liabilityAccounts`, `:71`, servent aussi la TVA). Si la 15-5d
   n'est pas mergée au moment du développement, le menu créanciers n'existe pas : le signaler, sans le
   créer ici. **Échec de `listBankAccounts()`** : pas de filtre, le refus serveur reste le filet.
   L'affichage des refus est **vérifié** (lentille F, P1) : le `catch` lit `err.message`
   (`:195-196`) — aucun changement.
9. **AC9 — Manuel.** Le manuel n'a pas de section sur le lien d'un compte bancaire au grand livre
   (lentilles F et R, P1) : les destinations sont **fixées** ici.
   - `docs/manual/fr/user-manual.tex` § *Solde comptable, relevé et écart* (`:237-242`), après la
     mention du compte lié (`:239-240`) : une phrase — ce compte ne peut être ni le compte débiteurs ni
     le compte créanciers désignés dans *Paramètres → Facturation* ; Kesh ne les propose pas et refuse
     ce lien.
   - `docs/manual/fr/admin-manual.tex` : le manuel d'administration n'a **pas** de section
     *Paramètres → Facturation* propre ; ses paragraphes vivent dans la `\subsection{Loi sur la TVA
     (LTVA)}` (`:2013`), qui porte déjà la configuration des comptes TVA, l'arrondi, le solde du reste
     et le montant minimum (`:2017-2033`). **Place** (choix C-15-6-27) : le
     `\paragraph{Comptes débiteurs et créanciers.}` s'insère **après** le paragraphe sans en-tête
     « Quand un paiement --- règlement manuel ou rapprochement bancaire --- … » (`:2033`, qui poursuit
     le sujet du solde du reste), **avant** « Le format de décompte officiel AFC » (`:2035`). L'insérer
     juste après *Montant minimum* (`:2031`) ferait lire `:2033` comme sa suite — un non-sens. Il le dit
     en une incise (« sur la même page »). **Contenu, dans cet ordre** : (i) il **présente d'abord** les
     deux comptes, que le manuel d'administration ne décrit nulle part (`grep -n "compte débiteurs\|compte
     créanciers" docs/manual/fr/admin-manual.tex` vide au 2026-10-08, PDF aplati compris) — le compte
     débiteurs reçoit la créance de chaque facture validée et le compte créanciers la dette de chaque
     facture fournisseur, chacun soldé par les règlements ; (ii) puis la règle dans les deux sens — un
     compte lié à un compte bancaire ne peut pas être désigné, et réciproquement. Si la 15-5d a déjà
     présenté le compte créanciers (son AC8), ne pas le présenter deux fois : compléter.
   - **Frontière** : la **15-5b** édite le même bloc (`:2016-2025`), la **15-5d** aussi (le compte
     créanciers « désormais réglable sur la même page », son AC8), la **15-6a** une phrase de
     l'arrondi (`:2027`), la **15-6b** une phrase aux comptes du solde du reste (`:2029`) et une
     condition à la liste des refus du paiement avec écart (`:2033`) — le compte désigné devenu le
     compte débiteurs de la facture (son AC11, finding F6-1 de sa P6) ; la 15-6a (§ *Avoirs*) ne touche
     pas les passages ci-dessus. Écrire sur l'état rebasé — **relire le voisinage
     `:2017-2035` au rebase**, les numéros ayant bougé. PDF régénérés (`make fr`) et contrôlés
     **aplatis** (`pdftotext … - | tr '\n' ' ' | tr -s ' '`), sur un fragment de la phrase neuve.
10. **AC10 — Documentation des intégrateurs** (`docs/api-external.md`) : **un seul** code dans le
    tableau du § 10 *Gestion des erreurs* — `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (400, `details`,
    routes `POST /api/v1/bank-accounts`, `PUT /api/v1/bank-accounts/{id}`,
    `PATCH /api/v1/bank-accounts/{id}` (lien)). `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` **n'y
    figure pas** : il n'est émis que par une route fermée aux clés (AC1) ; le guide documente l'API par
    clé. **Les routes `/bank-accounts` ne sont présentées nulle part dans le guide**
    (`grep -n "bank-accounts" docs/api-external.md` vide au 2026-10-08) ; le § 7 les couvre seulement
    par « toute route `/api/v1/*` de l'UI est consommable ». **Tranché** (choix C-15-6-27) : une ligne
    **Comptes bancaires** s'ajoute au tableau du § 7 *Ressources disponibles* — lecture
    `GET /bank-accounts`, écriture `POST /bank-accounts`, `PUT /bank-accounts/{id}`,
    `PATCH /bank-accounts/{id}` (lien au grand livre), `DELETE /bank-accounts/{id}` (archivage) — pour
    que la ligne du § 10 ne renvoie pas à des routes que le guide tait ; aucune section neuve (documenter
    la ressource entière est hors périmètre).
11. **AC11 — CHANGELOG** sous `## [0.13.0] — Non publié`, `### Corrigé`, renvoi à #474 (une seule
    entrée pour 15-6b et 15-6c si elles partent dans la même release : fusionner plutôt que doubler).
    **Propriétaire de la section** : la première story de la version mergée la crée en tête des
    versions si elle est absente ; les suivantes y ajoutent leur entrée.

## Tasks / Subtasks

- [x] **T0 — Rebase sur `main` après les 15-5b, 15-5d et 15-6b** ; relire les numéros de ligne ;
  vérifier que `validate_journal_account_id`, le contrôle C10, `withCurrentAccount` sur les trois
  surfaces, le menu créanciers de la 15-5d et les fonctions de `account-options.ts` sont là où cette
  fiche les attend ; vérifier l'existence des deux fichiers de test de la 15-5b que cette fiche étend
  (`bank-accounts/+page.test.ts`, `company_invoice_settings_postable_e2e.rs`) et que
  `BankAccountList.svelte` n'a toujours aucun importeur. Rejouer l'inventaire des tests (§ *Tests
  existants qui changent de sens*), **appels directs au dépôt compris**.
- [x] **T1 — Variantes, codes, i18n** (AC1).
- [x] **T2 — Lecteurs et ordre des verrous** (AC2).
- [x] **T3 — Routes et dépôt bancaires** (AC3, AC4, AC6).
- [x] **T4 — Réglages** (AC5, AC6).
- [x] **T5 — Écrans** (AC7, AC8), suppression de `BankAccountList.svelte` (AC7).
- [x] **T6 — Tests** (§ *Tests*), chaque garde **rougit d'abord** — sauf les cas « accepté » des tests
  3, 4 et 6 (non-régression de l'exemption), verts d'avance ; le 12 bis rougit par mutation (retrait
  du `FOR UPDATE` de `before`). Adapter les tests Vitest existants (prop, mock).
- [x] **T7 — Documentation** : manuels + PDF (AC9), `docs/api-external.md` — ligne du § 7 et code du
  § 10 (AC10) —, CHANGELOG (AC11).
- [x] **T8 — Gates** : backend complet (dépôt `kesh-db` touché), frontend complet, E2E complet au
  dernier commit de code (`bank-accounts-crud.spec.ts`, `bank-account-journal-link.spec.ts`).

## Tests existants qui changent de sens — inventaire (fait à la remédiation P1)

Commandes à rejouer en T0 :
`grep -rn "journalAccountId\|journal_account_id" frontend/tests/e2e crates/kesh-api/tests crates/kesh-db/tests`,
puis, pour chaque lien posé **par la route** (un `UPDATE` SQL contourne la garde et n'est pas
concerné), vérifier si le compte visé est désigné dans les réglages de la fixture ; et
`grep -rnE "(set_journal_account_id_for_company|bank_accounts::update_for_company)\(" crates --include=*.rs`
pour les **appels directs** au dépôt, dont la signature change (AC4).

- **Fixture** : `seed_accounting_company` (`crates/kesh-db/src/test_fixtures.rs:80-172`, désignation
  des réglages `:157-172`, `.bind(accounts["1100"])` `:164`) — préréglage
  E2E `with-company` (`crates/kesh-api/src/routes/test_endpoints.rs:184`) — désigne **1100 « Banque
  CI »** comme compte débiteurs (et 2000 comme TVA due, 1000 comme impôt préalable ; aucun compte
  créanciers). Le seed de dev `scripts/seed-dev-db.sql:74` crée 1100 « Banque CI » mais **pas** de
  réglages : il n'est pas concerné.
- **`frontend/tests/e2e/bank-account-journal-link.spec.ts:105-119`** lie un compte bancaire au **1100**
  par l'écran et l'exige dans le menu : il **rougirait** (menu filtré, puis refus serveur). **Tranché**
  (choix C-15-6-15) : la spec lie au **1000 « Caisse CI »** (actif, imputable, non désigné comme
  créance), sans toucher à la fixture partagée ; son commentaire d'en-tête (`:6`) suit.
- `frontend/tests/e2e/reconciliation-cancel.spec.ts:62-83` lie au **1000** : inchangé.
  `frontend/tests/e2e/payment-batches.spec.ts:51-78` lie au premier compte d'actif actif de la liste
  (`liquid`) : à vérifier en T0 que ce n'est pas 1100 (ordre de `GET /api/v1/accounts`) ; sinon
  choisir `1000` explicitement.
- Les tests Rust qui lient par SQL (`supplier_invoices_repository.rs:215`, `payment_batches_repository.rs:100`
  au 1100, `reconciliation_e2e.rs:578`, `:4564`, `reconciliation_rules_e2e.rs:193`,
  `reconciliation_manual_e2e.rs:365`, `bank_account_statement_gap.rs:40`, `bank_accounts_e2e.rs:1081`)
  ne passent pas par la garde ; ceux qui lient par la route (`bank_accounts_e2e.rs`, `api_keys_e2e.rs:487`)
  visent des comptes créés par le test, sans réglages désignés — inchangés, à confirmer en T0.
- **Appels directs au dépôt** (troisième voie, omise en P1 — finding R2-4) : recompte du 2026-10-08,
  **14 sites de test au 2026-10-08, avant le merge de la 15-5b** (qui ajoute des tests à
  `bank_accounts_repository.rs` — T0 recompte, et ses appels neufs passent eux aussi
  `&ClaimAccounts::default()`) — `crates/kesh-db/tests/bank_accounts_repository.rs` ×12 (11 appels de
  `set_journal_account_id_for_company` : `:330`, `:383`, `:443`, `:479`, `:493`, `:532`, `:573`,
  `:587`, `:705`, `:720`, `:868` ; 1 de `update_for_company` : `:901`),
  `crates/kesh-api/tests/reconciliation_e2e.rs:2645` et `crates/kesh-api/tests/reconciliation_split_e2e.rs:225`
  (helper partagé, `journal_account_id` en paramètre). Ils ne compilent plus après l'AC4. **Tranché**
  (choix C-15-6-22) : ils passent `&ClaimAccounts::default()` — la garde n'est l'objet d'aucun d'eux ;
  avec deux `None`, elle ne joue pas, et **aucun test ne change de sens**. Seuls les tests neufs 5 et
  11 passent des `ClaimAccounts` remplis. *(Le prompt de remédiation annonçait « ×13 » pour le premier
  fichier ; la commande ci-dessus en rend 12.)*
- **Tests Vitest existants** (omis jusqu'à la P3 — finding F4 ; choix C-15-6-27) :
  - `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts` : ses **quatre**
    rendus (`props:` aux lignes `:85`, `:117`, `:139`, `:150`) ne passent pas la prop
    `claimAccountIds`, **obligatoire** (AC7) : `npm run check` les refuse. Ils reçoivent
    `claimAccountIds: new Set()` — aucun ne change de sens (ensemble vide = pas de filtre).
  - `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts` : il mocke
    `$lib/features/invoices/invoices.api` (`:27`) et `$lib/features/accounts/accounts.api` (`:32`),
    **pas** `$lib/features/bank-accounts/bank-accounts.api` ; la page appellera `listBankAccounts()`
    (AC8) — fonction réelle, appel réseau en jsdom. Le fichier gagne un `vi.mock` de ce module,
    `listBankAccounts` résolu à `[]` par défaut : les tests existants gardent leur sens (aucun compte
    lié = aucun filtre). Le test 15 surcharge ce mock.
  - Commande à rejouer en T0 :
    `grep -rn "BankAccountJournalLinkForm\|bank-accounts.api" frontend/src --include=*.test.ts`.

## Tests

Backend :

1. `crates/kesh-api/tests/bank_accounts_e2e.rs` — POST avec `journalAccountId` = compte débiteurs
   désigné → 400 `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`, `details.claim = "receivable"`, rien créé ;
   compte créanciers désigné → `"payable"`. Réglages posés en SQL dans le montage du test.
2. Même fichier — **PUT** et **PATCH** : mêmes refus, `version` inchangée.
3. Même fichier — **exemption** : un compte bancaire **déjà** lié au compte débiteurs (posé en SQL)
   accepte un PUT qui ne change que son nom, et refuse un PUT qui le relie au compte créanciers.
4. Même fichier — **ordre** : PUT avec une `version` périmée vers le compte débiteurs → 409 (la
   version précède le refus, AC6) ; réglage débiteurs `NULL` → le lien est accepté.
5. `crates/kesh-db/tests/bank_accounts_repository.rs` — `set_journal_account_id_for_company` avec des
   `ClaimAccounts` désignant la cible : refus sans écriture, `version` inchangée ; même cible avec
   `claims` vides : acceptée.
6. `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs` (**créé par la 15-5b**, étendu
   ici ; si son montage ne s'y prête pas, fichier neuf — à trancher en T0 et à écrire au Dev Agent
   Record) — compte débiteurs déplacé vers un compte lié à un compte bancaire → 400
   `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` avec `bankAccountId` et `bankName` ; compte bancaire
   **archivé** lié → accepté ; valeur **inchangée** alors qu'un lien fautif existe → accepté.
7. Même fichier — **deux comptes bancaires** non archivés liés au même compte : le refus nomme le
   **premier par `id`**.
8. Même fichier — `defaultPayableAccountId` **présent** vers un compte de passif lié → `claim:
   "payable"` (l'écran l'envoie depuis la 15-5d, AC5 ; un client de session hors écran aussi) ;
   **absent** → préservé, aucun contrôle (AC19 de la 15-5b).
9. Même fichier — **ordre** : `version` périmée **et** compte lié → **409** (le contrôle de version
   précède le refus, AC6).
10. Même fichier — message **fr** d'un des deux codes. ⚠️ Pas d'`Accept-Language` dans Kesh : la
    langue est globale au processus (`init_error_i18n`) ; le test de **parité des clés** de
    `kesh-i18n` (existant) contrôle la présence des clés dans les autres locales, pas leur traduction.
11. `crates/kesh-db/tests/bank_accounts_repository.rs` — **sérialisation, côté compte bancaire** (patron
    `credit_note_waits_for_a_concurrent_settlement`, `credit_notes_repository.rs:594`) : une
    transaction tenue à la main verrouille la ligne des réglages (`FOR UPDATE`) et y désigne le compte X
    sans valider ; une tâche (`tokio::spawn`) ouvre la transaction du lien (`claim_accounts_in_share_mode`
    puis `set_journal_account_id_for_company` vers X). L'attente se **prouve** par
    `kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["FROM company_invoice_settings", "LOCK IN SHARE MODE"], || tache.is_finished())`
    (`crates/kesh-db/src/test_fixtures.rs:560`, forme exacte du patron `:594-628`) — jamais par un délai
    fixe, qui passe à vide sur une machine chargée ; `panic!` si la tâche a fini sans attendre ; la
    transaction tenue n'est validée qu'une fois l'attente vue ; la tâche rend alors
    `BankAccountLedgerIsClaimAccount`. Le motif est couplé à la forme du verrou : le doc-comment du
    test le dit.
12. `crates/kesh-db/tests/company_invoice_settings_repository.rs` — **sérialisation, côté réglages,
    conflit de ligne** : une transaction tenue à la main relie un compte bancaire au compte X sans
    valider ; une tâche appelle `company_invoice_settings::update` avec X comme compte débiteurs ;
    attente prouvée par `attendre_une_requete_en_cours(&pool, &["FROM bank_accounts", "LOCK IN SHARE MODE"], || tache.is_finished())` ;
    après validation, elle rend `ClaimAccountLinkedToBankAccount`. ⚠️ Ce test **n'épingle pas** le
    `FOR UPDATE` de `before` : il passe aussi avec un `SELECT` simple (le PUT attend sur la ligne
    bancaire, pas sur les réglages). D'où le 12 bis.
12 bis. Même fichier — **sérialisation, côté réglages, sur le `FOR UPDATE` de `before`** (finding F1
    de la P3, choix C-15-6-27) : une transaction tenue à la main appelle `claim_accounts_in_share_mode`
    (verrou **S** sur la ligne des réglages) **sans encore** lier ; une tâche appelle
    `company_invoice_settings::update` désignant X comme compte débiteurs ; elle doit **attendre au
    `FOR UPDATE` des réglages** — prouvé par
    `attendre_une_requete_en_cours(&pool, &["FROM company_invoice_settings", "FOR UPDATE"], || tache.is_finished())` ;
    l'attente vue, la transaction tenue lie un compte bancaire à X (rien de ce que tient la tâche — un
    S sur les réglages, posé par `INSERT IGNORE` — ne la bloque) et valide ; la tâche rend
    `ClaimAccountLinkedToBankAccount`. **Rougit d'abord** en retirant le `FOR UPDATE` de `before` : la
    tâche ne s'arrête plus à la lecture de `before` mais à l'`UPDATE` (le motif n'est pas vu,
    l'assistant panique au bout de dix secondes), ou l'échange finit en interblocage 1213 (le lien de
    la transaction tenue attend le verrou d'intervalle posé par le lecteur bancaire de la tâche) — dans
    les deux cas, pas le refus attendu. C'est le seul test qui ferme la course que ce verrou ferme.

Frontend (Vitest) :

13. `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts` (existant) : les
    comptes désignés absents des options ; un lien existant vers l'un d'eux reste affiché.
14. `frontend/src/routes/(app)/bank-accounts/+page.test.ts` (**créé par la 15-5b**, étendu ici — pas
    de second fichier de rendu de la même page ; son mock de `getInvoiceSettings` s'ajoute aux siens) :
    les deux `<select>` n'offrent ni le compte débiteurs ni le compte créanciers désignés, et
    affichent un lien existant vers 1100 sans le perdre ; `getInvoiceSettings` en échec → menus non
    filtrés, page affichée.
15. `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts` (existant) : les
    menus du compte débiteurs et du compte créanciers (15-5d) n'offrent pas un compte lié à un compte
    bancaire, gardent la valeur en place, et les autres menus d'actif et de passif (TVA) sont intacts ;
    `listBankAccounts` en échec → pas de filtre.

E2E : `bank-account-journal-link.spec.ts` modifiée (inventaire ci-dessus).

Soit **16 tests nommés** (13 backend — 1 à 12 et 12 bis —, 3 fichiers Vitest) et **1 spec E2E
modifiée** ; s'y ajoutent les **adaptations** des tests Vitest existants (§ *Tests existants qui
changent de sens*, quatre rendus et un mock), qui ne sont pas des tests neufs.

## Dev Notes

### Angles morts assumés

- **Rôle sans réglage** : la garde suit les **réglages** (les comptes qu'utiliseront les ventes et
  achats futurs), pas le rôle ; l'écran aussi (AC7). Un compte de rôle `Receivable` qui ne serait plus
  le réglage débiteurs reste liable.
- **Données antérieures** : un lien fautif existant n'est pas défait (pas de migration de données — le
  produit ne tient pas encore de comptabilité réelle, `CLAUDE.md` § *Project Overview*) ; il est refusé
  à l'usage (15-6b) et signalé à la modification.
- **Import d'une sauvegarde** (`.keshbackup`) : il restaure tables et liens sans repasser par ces
  routes ; un couple fautif présent dans la sauvegarde revient tel quel. Filet : la garde à l'usage de
  la 15-6b.
- **Création des réglages à l'onboarding** (`company_invoice_settings::insert_with_defaults_in_tx`,
  désignation par rôle) : elle ne consulte pas les comptes bancaires. Un compte bancaire déjà lié au
  compte de rôle `Receivable` à ce moment n'est pas détecté ; même filet.

### Latence — ce que les verrous ajoutent, sans le présenter comme un défaut

- Tout geste bancaire **vers un compte** (création, remplacement, lien) prend un S sur la ligne des
  réglages ; il **attend** donc toute transaction qui tient cette ligne en X : PUT des réglages, mais
  aussi validation de facture, avoir et facture fournisseur (`get_or_create_default_in_tx`,
  `FOR UPDATE`, `invoices.rs:2006`, `credit_notes.rs:361`, `supplier_invoices.rs:359`) — le temps
  d'une transaction. Une dé-liaison n'attend pas (AC2).
- Le lecteur bancaire du PUT pose des verrous d'intervalle (AC2) : une création de compte bancaire
  concurrente peut attendre le PUT. Raisonné au code, **non mesuré**.

### Pièges

- **Respecter l'ordre et l'emplacement de la 15-5b** : sa garde de postabilité et celle-ci ne doivent
  pas se court-circuiter. Les deux vivent au même endroit — dans la transaction, après le contrôle de
  version et, pour le lien, après son no-op ; sous la condition « nouvelle valeur ≠
  `existing.journal_account_id` » ; la postabilité d'abord, puis le compte de créance.
- ⛔ **`FOR SHARE` n'existe pas en MariaDB 10.11** : `LOCK IN SHARE MODE` (AC2,
  `opening_complement.rs:38-39`). Le défaut a été écrit en P1 et attrapé en P2 ; un test de
  sérialisation (11, 12, 12 bis) le révélerait en 500 dès sa première exécution.
- **Attendre un verrou, c'est le prouver** : `attendre_une_requete_en_cours` (tests 11, 12, 12 bis),
  jamais un délai fixe.
- **Respecter l'ordre des verrous** (AC2) : réglages, puis comptes bancaires. Lire les réglages
  **après** le `FOR UPDATE` de la ligne bancaire réintroduirait un interblocage entre les deux gestes.
- `default_receivable_account_id` / `default_payable_account_id` peuvent être `NULL` : pas de refus
  dans ce cas.

### Références

- [Source: crates/kesh-api/src/routes/bank_accounts.rs:258-294, :384-600, :717-770]
- [Source: crates/kesh-db/src/repositories/bank_accounts.rs:237-420, :579-598 (verrou sentinelle)]
- [Source: crates/kesh-api/src/routes/company_invoice_settings.rs:246-400 ;
  crates/kesh-db/src/repositories/company_invoice_settings.rs:147-240]
- [Source: frontend/src/routes/(app)/bank-accounts/+page.svelte:77-85, :321-330, :437, :472-481 ;
  frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:44-60 ;
  frontend/src/routes/(app)/settings/invoicing/+page.svelte:65, :85-111, :195-196]
- [Source: crates/kesh-db/src/test_fixtures.rs:80-172 ; frontend/tests/e2e/bank-account-journal-link.spec.ts:105-119]
- [Source: crates/kesh-db/src/repositories/opening_complement.rs:38-39, :502, :540 ; crates/kesh-api/src/lib.rs:190-343, :339-341, :664-671 ;
  crates/kesh-api/tests/admin_pat_denied_e2e.rs:64 ; frontend/src/lib/features/bank-accounts/BankAccountList.svelte]
- [Source: docs/manual/fr/user-manual.tex:237-242 ; docs/manual/fr/admin-manual.tex:2013, :2017-2035 (insertion après :2033)]
- [Source: crates/kesh-db/src/test_fixtures.rs:560 (`attendre_une_requete_en_cours`) ; crates/kesh-db/tests/credit_notes_repository.rs:594-628 ;
  docs/api-external.md § 7, § 10 ; frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts:27-32]
- [Source: `15-5b-gardes-surfaces-neuves.md` AC10–AC13, AC19, choix C3, C4, C5, C10, C12, C25, C26 ;
  `15-5d-garde-usage-comptes-reglage.md` AC5, AC8, choix C34 (dépôt principal, branche
  `story/15-5-gardes-postabilite-serveur`)]
- [Source: issue #474 ; `epic-15-choix-autonomes.md` C-15-6-1, C-15-6-5, C-15-6-13 à C-15-6-15, C-15-6-21, C-15-6-22, C-15-6-27]

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, en autonomie), 2026-10-09. Worktree `kesh-15-6c`, cible cargo
propre, bases `kesh_156c` / `kesh_e2e_156c`, backend E2E sur le port 3019.

### Debug Log References

- **Gate backend, premier run** (sur `2707b69c`) : rouge à la compilation (`clippy::explicit_auto_deref`
  sur `&mut **tx`) — corrigé (`86f945c3`). **Deuxième run** : `rejeu_interblocage_e2e`
  `invoice_settings_update_is_replayed_when_it_is_the_deadlock_victim` rouge — son motif d'attente
  (`UPDATE company_invoice_settings`) ne voit plus la route, qui attend désormais au `FOR UPDATE` de
  `before` (même cycle, autre point d'attente). Motif corrigé (`673478b7`, choix C-15-6c-4).
- **Mutations jouées par script qui restaure depuis `HEAD`** : une première série frontend a été
  jouée alors que les écrans n'étaient pas encore commités — la restauration les a effacés et les
  résultats de F2 à F5 de cette série étaient faux. Écrans réappliqués, commités (`c780b4f3`), puis
  **les six mutations rejouées** sur l'arbre propre (résultats ci-dessous). Les mutations backend
  ont toutes été jouées après commit du code muté.
- **Vitest, premier run complet** : 3 rouges dans les gardes de décompte i18n
  (`i18n-keys.test.ts` `sitesTotal`, `i18n-libelle-en-dur.test.ts` candidates et ventilation),
  conséquence de la suppression de `BankAccountList.svelte` — recomptés depuis la source et
  ajustés avec leur motif (1922 → 1916 sites, 6 `i18nMsg(` dans le fichier supprimé ; 48 → 47
  candidates, `accountLabel`, `conforme` 41 → 40).
- Prettier n'a pas de configuration dans `frontend/` : lancé sur trois fichiers de test, il les a
  reformatés en entier. Annulé (`git checkout`), modifications rejouées sans lui.

### Completion Notes List

- **AC1** — `DbError::BankAccountLedgerIsClaimAccount` et `DbError::ClaimAccountLinkedToBankAccount`
  (`kesh-db/src/errors.rs`), codes 400, mapping unique `claim_configuration_response`
  (`kesh-api/src/errors.rs`), quatre clés dans les quatre locales. Le numéro du compte se lit par
  `accounts::number_in_company`, partagé avec le refus de la 15-6b (C-15-6c-1).
- **AC2** — `company_invoice_settings::claim_accounts_in_share_mode` (`ClaimAccounts`, `side_of`)
  et `bank_accounts::first_active_bank_account_linked_to`, en `LOCK IN SHARE MODE` ; `before` lu en
  `FOR UPDATE` ; note KF-004 réécrite (sérialisé / non sérialisé, rejeu de la route). Les gestes
  bancaires sans compte lié passent `ClaimAccounts::default()` sans lecture (`claims_for_target`).
- **AC3/AC4** — garde unique `bank_accounts::refuse_if_ledger_is_claim_account` : à la création dans
  la transaction de la route (premier verrou après la sentinelle, avant la démotion) ; au
  remplacement et au lien dans le dépôt, après la postabilité, sous l'exemption « inchangé ».
  21 appels directs au dépôt adaptés (`ClaimAccounts::default()`).
- **AC5/AC6** — contrôle dans `company_invoice_settings::update`, après le 409, avant le no-op,
  compte débiteurs puis créanciers ; ordre des erreurs écrit dans les sept doc-comments.
- **AC7/AC8** — page des comptes bancaires (réglages lus par `getInvoiceSettings`, en
  `Promise.allSettled`), `BankAccountJournalLinkForm` (prop obligatoire `claimAccountIds`), écran des
  réglages (`listBankAccounts`, échec rattrapé) : filtre avant `withCurrentAccount`, sur les seuls
  menus débiteurs et créanciers. `BankAccountList.svelte` supprimé (aucun importeur).
- **AC9** — manuel utilisateur (§ *Solde comptable, relevé et écart*) et paragraphe *Comptes
  débiteurs et créanciers.* du manuel d'administration, après « Quand un paiement… », avant « Le
  format de décompte officiel AFC » ; il complète la présentation du compte créanciers de la 15-5d
  et dit l'angle mort « données antérieures » (C-15-6c-3). PDF régénérés (`make fr`), contrôlés
  aplatis sur les phrases neuves ; la brochure régénérée sans changement de source n'est pas commitée.
- **AC10** — ligne *Comptes bancaires* au § 7 et `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` au § 10 du
  guide ; `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` n'y figure pas (route fermée aux clés).
- **AC11** — l'entrée #474 du CHANGELOG `[0.13.0]` est complétée, pas doublée.
- **Tests** (périmètre `f8b2accd..HEAD`, recomptés par `grep -cE '#\[(sqlx::test|tokio::test|test)'`
  et `grep -cE "^\s*it(\.each)?\("` aux deux bornes) : **15 tests backend neufs** — les 13 nommés de
  la fiche (1 à 12 et 12 bis ; le 2 couvre PUT et PATCH dans une fonction) et 2 unitaires de mapping
  (`configuration_claim_refusals_error_codes`, `claim_configuration_refusals_are_400_with_details`) ;
  **7 déclarations Vitest neuves, 9 cas** (formulaire 2, page 3 déclarations / 5 cas, réglages 2) ;
  1 test backend modifié (`rejeu_interblocage_e2e`, motif d'attente, C-15-6c-4) ; adaptations : quatre
  rendus du formulaire, un mock de `bank-accounts.api`, un de `invoices.api`, deux gardes de
  décompte i18n ; 1 spec E2E modifiée (lien au `1000`, absence du `1100` — C-15-6c-2).
- **Mutations jouées et constatées rouges** (15) — backend : M1 `before` sans `FOR UPDATE` → 12 bis
  seul rouge (le 12 reste vert, comme la fiche l'annonce) ; M2 lecteur des réglages sans verrou → 11 ;
  M3 garde du lien retirée → 5 et 11 ; M4 lecteur bancaire sans verrou → 12 ; M5 garde de création
  retirée → 1 ; M6 garde des réglages neutralisée → 6, 7, 8, 10 ; M7 exemption « inchangé » des
  réglages retirée → 6 et 8 ; M8 refus avant le 409 → 9 ; M9 exemption du remplacement bancaire
  retirée → 3. Frontend : F1 page sans filtre ; F2 formulaire sans filtre ; F3 filtre après
  `withCurrentAccount` (lien existant perdu) ; F4 menu débiteurs sans filtre ; F5 filtre étendu à la
  TVA ; F6 échec de `listBankAccounts` non rattrapé — chacune rouge sur le test qui la vise.
  Chaque fichier restauré par `git checkout` puis `touch`.
- **Gates au dernier commit de code `673478b7`** : bases `kesh_156c` / `kesh_e2e_156c` remises à zéro
  (DROP/CREATE, migrations, seed) avant chaque gate ; `scripts/test-fast.sh` (fmt + clippy + nextest)
  **3021/3021, 4 ignorés** ; frontend `npm run check` 0 erreur (27 avertissements, aucun dans les
  fichiers touchés), `lint-i18n-ownership` vert, `test:unit` **1127/1127** (au commit `86f945c3` ;
  aucun fichier frontend modifié depuis), `build` vert ; **E2E complet : 244 passés, 10 échoués,
  19 ignorés** — 7 KF-029, 2 KF-045 (`invoices.spec.ts:415`, `:439`, run à 08:10 UTC), et
  `invoice-minimum-amount.spec.ts:46` (page renvoyée au login, « session expirée ») : **rejoué seul,
  vert** — l'échec variable de pollution de `docs/testing.md`. `bank-account-journal-link.spec.ts` et
  `bank-accounts-crud.spec.ts` verts dans la suite et rejoués seuls. Montage : `KESH_TEST_MODE=true`
  des deux côtés, `KESH_COOKIE_SECURE=false`, SMTP factices (`smtpConfigured:true`), répertoires
  inbox, documents et sauvegarde sous `target/e2e/` du worktree. Backend arrêté par son PID.
- **À vérifier en revue** : (1) la portée next-key du lecteur bancaire (attente possible d'une
  création de compte bancaire pendant un PUT des réglages) reste **non mesurée** ; (2) le test de
  rejeu 15-5e1 dépend désormais du `FOR UPDATE` de `before` ; (3) l'écran des réglages filtre sur
  `journalAccountId` des comptes bancaires non archivés tels que les rend `GET /bank-accounts` —
  l'onboarding et l'import de sauvegarde restent des angles morts (Dev Notes).

### File List

38 chemins (`git diff --name-status f8b2accd..HEAD`) :

- Supprimé : `frontend/src/lib/features/bank-accounts/BankAccountList.svelte`.
- Backend : `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/{accounts,bank_accounts,company_invoice_settings,invoice_settlements}.rs`,
  `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/{bank_accounts,company_invoice_settings}.rs`,
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
- Tests backend : `crates/kesh-db/tests/{bank_accounts_repository,company_invoice_settings_repository}.rs`,
  `crates/kesh-api/tests/{bank_accounts_e2e,company_invoice_settings_postable_e2e,reconciliation_e2e,reconciliation_split_e2e,rejeu_interblocage_e2e}.rs`.
- Frontend : `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte`,
  `frontend/src/routes/(app)/bank-accounts/+page.svelte`, `frontend/src/routes/(app)/settings/invoicing/+page.svelte`.
- Tests frontend : `BankAccountJournalLinkForm.test.ts`, `bank-accounts-page.test.ts`,
  `settings-invoicing-page.test.ts`, `frontend/src/lib/shared/{i18n-keys,i18n-libelle-en-dur}.test.ts`,
  `frontend/tests/e2e/bank-account-journal-link.spec.ts`.
- Documentation : `CHANGELOG.md`, `docs/api-external.md`, `docs/manual/fr/{admin-manual,user-manual}.{tex,pdf}`.
- Artefacts : cette fiche, `sprint-status.yaml`, `epic-15-choix-autonomes.md` (C-15-6c-1 à 4).

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie). Statut `ready-for-dev`.
- 2026-10-08 — **Validation P1** (Sonnet 4.6, lentilles F et R ; prompt
  `15-6c-validate-prompt-p1.md`). F : 2 HIGH, 4 MEDIUM, 3 LOW ; R : 1 HIGH, 5 MEDIUM, 4 LOW (R3 = F5,
  R7 = F4). Appliqués :
  - **HIGH F1** — contrôle des réglages hors transaction : déplacé **dans** `company_invoice_settings::update`,
    contre `before` lu sous verrou ; ordre des erreurs écrit, route par route (AC5, AC6).
  - **HIGH F2** — course entre les deux gestes : **fermée** par `FOR SHARE` *(graphie fausse, corrigée en P2 : `LOCK IN SHARE MODE`)* / `FOR UPDATE` et un ordre
    de verrous unique (AC2, tests 11–12) ; l'angle mort est supprimé (C-15-6-13, qui révise
    C-15-6-5).
  - **HIGH R1** — AC6 périmé par l'AC13 de la 15-5b : l'écran vise désormais le filtre
    `eligibleAccounts` de `BankAccountJournalLinkForm`, avant `withCurrentAccount` (AC7).
  - **MEDIUM F3** — écran filtré sur les **ids désignés dans les réglages**, non sur le rôle (AC7).
  - **MEDIUM F4 (= R7 LOW)** — l'écran des réglages ne propose plus en débiteurs un compte lié (AC8) ;
    la lecture de `err.message` est vérifiée, la question retirée.
  - **MEDIUM F5 = R3** — destinations du manuel fixées (AC9).
  - **MEDIUM F6 = R6** — variantes `DbError` et fonctions de dépôt nommées, contrôle de création placé
    dans la transaction de la route, lecteurs uniques (AC1–AC3).
  - **MEDIUM R2** — signature des fonctions d'écran partagée avec la 15-6b (C-15-6-14).
  - **MEDIUM R4** — tests existants inventoriés au seed ; la spec E2E qui liait au 1100 lie au 1000
    (C-15-6-15).
  - **MEDIUM R5** — compte créanciers comparé à sa **valeur résolue** (AC19 de la 15-5b), test dédié
    (test 8).
  - **LOW F7** — propriétaire unique de la section `[0.13.0]` (AC11). **LOW F8** — angles morts
    « import de sauvegarde » et « création des réglages à l'onboarding » écrits. **LOW F9** — tests
    « réglage NULL » et « deux comptes bancaires, premier par id » (tests 4, 7) ; seed vérifié.
    **LOW R8** — une clé par `claim` nommant le bon compte (AC1). **LOW R9** — refus typé et traduit
    sur une route aux refus en français en dur : dit (AC1). **LOW R10** — fichier de test du PUT des
    réglages fixé (neuf) ; `docs/api-external.md` (AC10).
  - Décompte après passe : **11 AC, 9 tâches (T0–T8), 15 tests** (12 backend, 3 fichiers Vitest) +
    1 spec E2E modifiée.
- 2026-10-08 — **Validation P2** (Opus 5.5, lentilles R et F ; prompt `15-6c-validate-prompt-p2.md`).
  R : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 6 LOW ; F : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 4 LOW (R2-1 = F1,
  R2-2 = F4, R2-3 = F3, R2-8 = F5, R2-10 = F7). Tous appliqués, décisions de l'orchestrateur comprises :
  - **HIGH R2-1 = F1** — `FOR SHARE` (né de la remédiation P1) est une erreur de syntaxe sur MariaDB
    10.11 : **`LOCK IN SHARE MODE`** dans les deux lecteurs, lecteur renommé
    `claim_accounts_in_share_mode`, piège écrit ; valeur grepée sur les quatre fiches et le registre.
    Choix C-15-6-21.
  - **MEDIUM F2** — route des réglages : le **409 d'abord**, puis le refus (le client voit l'état
    courant avant qu'on juge son changement) ; AC6, test 9 (→ 409). Choix C-15-6-21, qui révise
    l'ordre de C-15-6-13.
  - **MEDIUM R2-3 = F3** — `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` n'est pas destiné aux intégrateurs
    (route fermée aux clés) : AC1 (destinataires), AC10 (un seul code au guide), test 8. Choix
    C-15-6-21.
  - **MEDIUM R2-2 = F4** — tests 6 et 14 **étendent** les fichiers créés par la 15-5b, pas de doublon ;
    T0 vérifie leur existence.
  - **MEDIUM R2-4** — appels directs au dépôt inventoriés (14 sites de test) et tranchés :
    `ClaimAccounts::default()`. Choix C-15-6-22.
  - **15-5d** (consigne de l'orchestrateur) : le compte créanciers devient corrigeable à l'écran ; son
    menu est filtré (AC8), son refus atteignable par l'écran (AC5, test 8, test 15).
  - **LOW** — R2-5 : références de fixture `:80-172` ; R2-6 : place exacte du contrôle (après la
    version, après le no-op du seul lien) ; R2-7 : « premier verrou après la sentinelle », ordre global
    `companies → réglages → bank_accounts` ; R2-8 = F5 : interblocage préexistant de l'`INSERT IGNORE`
    écrit dans la note KF-004 réécrite, non promis comme résolu ; R2-9 : frontière 15-5b/15-5d/15-6a
    au manuel d'administration, paragraphe à la fin du bloc LTVA qui porte déjà tous les réglages de
    facturation ; R2-10 = F7 : échec de `getInvoiceSettings()` / `listBankAccounts()` → pas de filtre ;
    F6 : `BankAccountList.svelte`, code mort, supprimé, prop obligatoire (C-15-6-22) ; F8 : voir
    ci-dessous.
  - **Signal de découpage (F8), déclaré au Project Lead** : la sévérité ne baisse pas (HIGH en P1,
    HIGH en P2) et le HIGH de la P2 **naît de la remédiation P1** — c'est le cas de recyclage que
    l'amendement D5 désigne comme déclencheur ; les modules dépassent cinq (`kesh-db` comptes bancaires
    et réglages, `kesh-api` deux routes, `frontend` deux écrans). **Non découpée** (choix C-15-6-22) :
    le défaut est une graphie de mot-clé, non une conception qui tourne en rond, et séparer les deux
    sens casserait l'ordre de verrous qui les tient ensemble. **Arbitrage de Guy attendu.**
  - Décompte après passe : **11 AC, 9 tâches (T0–T8), 15 tests** (12 backend, 3 fichiers Vitest) +
    1 spec E2E modifiée ; un fichier supprimé (`BankAccountList.svelte`).
- 2026-10-08 — **Validation P3** (Sonnet, lentilles R et F ; remédiation Opus 5.5 ; prompt
  `15-6c-validate-prompt-p3.md`). R : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 5 LOW ; F : 0 CRITICAL, 0 HIGH,
  2 MEDIUM, 6 LOW (R3-2 = F3, R3-6 = F6). Tous appliqués, décisions de l'orchestrateur comprises
  (choix C-15-6-27) :
  - **MEDIUM F1** — le `FOR UPDATE` de `before`, clé de la sérialisation côté réglages, n'était épinglé
    par aucun test (11 et 12 passent avec un `SELECT` simple) : **test 12 bis** neuf (transaction tenue
    en S par `claim_accounts_in_share_mode`, le PUT attend au `FOR UPDATE`), qui rougit par mutation ;
    le test 12 dit ce qu'il ne prouve pas.
  - **MEDIUM F2** — attente des tests 11, 12, 12 bis prouvée par `attendre_une_requete_en_cours`
    (`test_fixtures.rs:560`), motifs écrits ; plus de délai fixe.
  - **MEDIUM R3-1** — paragraphe du manuel d'administration placé **après** `:2033` (« Quand un
    paiement… »), avant `:2035` ; borne `:2017-2033` (AC9, Références). **LOW F5** — il présente
    d'abord les comptes débiteurs et créanciers, absents du manuel d'administration.
  - **LOW R3-2 = F3** — `$account` (numéro, repli `#<id>`), `$bank` (`bank_name`) ; numéro lu après
    décision du refus, sans verrou ; cas `None` (AC1).
  - **LOW F4** — tests Vitest existants inventoriés : quatre rendus de
    `BankAccountJournalLinkForm.test.ts` reçoivent `claimAccountIds: new Set()`, mock de
    `bank-accounts.api` ajouté à `settings-invoicing-page.test.ts` ; aucun ne change de sens.
  - **LOW R3-6 = F6** — portée réelle du verrou (next-key / intervalle, attente possible d'un `INSERT`
    concurrent, non mesurée) ; latence des gestes bancaires derrière les validations de pièces écrite
    (Dev Notes, *Latence*).
  - **LOW F7** — dé-liaison `null` (et création/remplacement sans compte) : ni lecture des réglages ni
    verrou S, `ClaimAccounts::default()` (AC2, AC4).
  - **LOW F8** — `docs/api-external.md` : ligne *Comptes bancaires* au § 7, code au § 10 (AC10, T7).
  - **LOW R3-3** — les sept fonctions de l'AC6 nommées. **LOW R3-4** — « 14 sites » daté, avant le
    merge de la 15-5b. **LOW R3-5** — chemin du dépôt préfixé pour la sentinelle (`:588`/`:592`).
  - **Trend** : P1 3 HIGH, 9 MEDIUM → P2 2 HIGH, 6 MEDIUM → P3 0 HIGH, 3 MEDIUM. **Signal de
    découpage (D5)** : non levé à cette passe — la sévérité baisse (HIGH → MEDIUM) et les trois MEDIUM
    sont distincts de ceux de la P2 ; aucun ne naît d'une remédiation (F1 et F2 visent des tests écrits
    dès la P1, R3-1 une borne du manuel écrite en P2 mais fausse dès son écriture, non recyclée). Le
    signal de la P2 (modules > 5) reste déclaré, l'arbitrage de Guy toujours attendu.
  - Décompte après passe : **11 AC, 9 tâches (T0–T8), 16 tests** (13 backend dont le 12 bis neuf,
    3 fichiers Vitest) + 1 spec E2E modifiée + adaptations de deux fichiers Vitest existants ; un
    fichier supprimé (`BankAccountList.svelte`).
- 2026-10-08 — **Validation P4 — passe ciblée, clôture de la validation** (Haiku 4.5, une lentille
  « chasseur de régressions » braquée sur la seule remédiation P3, diff aplati
  `git diff 4964eeb4 89e8987a` sur cette fiche ; prompt versionné `15-6c-validate-prompt-p4-ciblee.md`).
  **0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW.** Axes déclarés exercés : cohérence du commentaire d'en-tête,
  arguments des messages et source du numéro, portée du verrou (next-key), exemption de la dé-liaison,
  les sept fonctions de l'AC6 (recomptées), place du paragraphe du manuel d'administration, ligne du
  § 7 du guide, tâches T6/T7, inventaire daté et adaptations Vitest, test 12 bis (distinct du 12, motif
  d'attente, mutation), recompte des tests (16). Axes déclarés **non** exercés (hors d'une passe
  ciblée sur une fiche) : compilation, rougissement des tests, code des routes et du dépôt, rendu
  frontend, numéros de ligne après rebase, E2E — renvoyés à T0, T6 et T8.
  **« 0 » vérifié par l'orchestrateur, comme un finding** (règle « Une passe qui ne déclare pas ses axes
  ne compte pas ») : le test 12 bis existe dans la fiche ; `attendre_une_requete_en_cours` existe à
  `crates/kesh-db/src/test_fixtures.rs:560`. Haiku est employé ici en passe **ciblée**, conformément à
  la décision D6 (rétrospective de l'Epic 25) ; la remédiation relue ne touchait aucune ligne de code de
  production (fiche seule) : la boucle peut se clore.
  - **Trend complet** (F + R par passe) : P1 (Sonnet 4.6, lentilles F et R) 3 HIGH, 9 MEDIUM, 7 LOW →
    P2 (Opus 5.5, lentilles R et F) 2 HIGH, 6 MEDIUM, 10 LOW → P3 (Sonnet, lentilles R et F ;
    remédiation Opus 5.5) 0 HIGH, 3 MEDIUM, 11 LOW → P4 ciblée (Haiku 4.5) 0. Aucun CRITICAL sur
    l'ensemble. Modèles : Sonnet → Opus → Sonnet pour les passes complètes, Haiku pour la passe ciblée
    de fin de boucle (D6).
  - **Reclassements** : aucun finding MEDIUM+ reclassé en dette. **Signal de découpage** : levé en P2
    (modules > 5, HIGH né de la remédiation P1), non découpé (C-15-6-22), **arbitrage de Guy toujours
    attendu** ; non relevé en P3 ni en P4.
  - **Validation close** : 0 au-dessus de LOW. Décompte final : **11 AC, 9 tâches (T0–T8), 16 tests**
    (13 backend, 3 fichiers Vitest) + 1 spec E2E modifiée + adaptations de deux fichiers Vitest
    existants ; un fichier supprimé (`BankAccountList.svelte`).
- 2026-10-09 — **T0 — relecture contre `f8b2accd`** (`origin/main`, 15-5b, 15-5d, 15-6a, 15-6b mergées ;
  agent de développement Opus 5.5). **Aucun écart ne change une règle ni un AC** ; écarts de lieu et de
  décompte :
  - Numéros relocalisés par le texte : `validate_journal_account_id` `bank_accounts.rs:268`, création
    `:408` (sentinelle `:432`, `flip_primary_off` `:440`, `INSERT` en ligne `:457`), remplacement
    `:530`, lien `:756` ; dépôt : `set_journal_account_id_for_company` `:332` (no-op `:373`, garde de
    postabilité `:382`), `update_for_company` `:439`, `acquire_company_sentinel_lock` `:680` ;
    `company_invoice_settings::update` `:147`, `before` en `SELECT` simple `:163`, note KF-004 `:182` ;
    route des réglages `update_invoice_settings` `:272`.
  - **Le PUT des réglages est rejoué sur interblocage** depuis la 15-5e1 (`retry_on_deadlock`,
    choix C70) : l'interblocage `INSERT IGNORE` → `FOR UPDATE` que l'AC2 décrit (« l'un des deux en
    500 ») est désormais **rejoué**, pas rendu en 500. La note KF-004 réécrite le dit ; rien d'autre ne
    change.
  - Le fichier Vitest de la page bancaire créé par la 15-5b s'appelle
    `frontend/src/routes/(app)/bank-accounts/bank-accounts-page.test.ts` (et non `+page.test.ts`) :
    c'est lui que le test 14 étend. `company_invoice_settings_postable_e2e.rs` existe et son montage
    (`setup`, six comptes, Admin) se prête aux tests 6 à 10 : étendu, pas de fichier neuf.
  - **Appels directs au dépôt : 21 sites** (et non 14) — `bank_accounts_repository.rs` ×19 (la 15-5b a
    ajouté `:994`, `:1008`, `:1020` pour le lien et `:1057`, `:1073`, `:1089`, `:1102` pour le
    remplacement), `reconciliation_e2e.rs:2645`, `reconciliation_split_e2e.rs:225`. Tous passent
    `&ClaimAccounts::default()` (C-15-6-22), aucun ne change de sens.
  - `BankAccountList.svelte` : toujours **aucun importeur** → supprimé. Fonctions d'écran de la 15-6b
    présentes (`withoutAccountIds(accounts, ids: Set<number>)`). Menu créanciers de la 15-5d présent
    (`payableOptions`, `settings/invoicing/+page.svelte:94`).
  - Manuel d'administration : la 15-5d a déjà présenté le **compte créanciers** (§ *Compte créanciers.*,
    `:2154`) et nommé la créance client ; le paragraphe neuf **complète** (présente le compte débiteurs,
    renvoie au créanciers) et s'insère après « Quand un paiement… » (`:2162`), avant « Le format de
    décompte officiel AFC » (`:2164`). La frontière avec la 15-6b (`:2158`, `:2162`) est celle que
    l'AC9 décrit.
  - E2E : `payment-batches.spec.ts` lie au premier actif par numéro = `1000` (comptes triés par
    `number`) — non concerné ; aucune autre spec ne lie par la route au compte débiteurs ou créanciers.
- 2026-10-09 — **Développement** (Opus 5.5, en autonomie ; commits `bcc5828d`, `f48bcf87`,
  `c780b4f3`, `2707b69c`, `86f945c3`, `673478b7`). Toutes les tâches faites ; 15 tests backend et
  9 cas Vitest neufs ; 15 mutations rouges ; gates au dernier commit de code : backend 3021/3021,
  Vitest 1127/1127, E2E 244 / 10 (7 KF-029, 2 KF-045, 1 pollution rejouée seule verte). Écart à
  l'inventaire de la fiche : un test de la 15-5e1 couplé au point d'attente du PUT des réglages
  (C-15-6c-4). Choix C-15-6c-1 à C-15-6c-4. Statut `review`.
