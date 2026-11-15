# Story 36-19 : la finalisation de l'assistant complète les comptes vides des réglages de facturation (#1200)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-19`, branche
`story/36-19-finalisation-comptes-vides`, par le workflow `bmad-create-story`. Release **0.18.0**, **sans migration**.
Code lu à **`fed34a9c7`** (tête de `origin/epic/36-0.18.0`, sur laquelle la branche a été avancée en avance rapide).
Registres : `e36-choix-autonomes-e360.md` (C-e360-3, C-e360-11) ; `e36-choix-autonomes-36-19.md` pour la story
(C-36-19-1 à 6). Bases `kesh_e3619`, `kesh_e2e_e3619` (voie 2), `kesh_ci_e3619` (voie 1). Validation non lancée.*

## Story

En tant que personne qui installe Kesh et qui, avant la fin de l'assistant de configuration, ouvre le journal, les
comptes bancaires, une facture ou les réglages de facturation, je veux que la finalisation de l'assistant désigne quand
même les comptes par défaut de facturation que le plan comptable fournit — sans toucher à ceux que j'ai déjà choisis —,
et, si elle doit refuser, qu'elle me dise ce qui manque réellement, afin de ne pas rester bloqué sur un refus qui
prétend que les comptes 1100 et 3000 n'existent pas.

## Ce qui se lit au code (`fed34a9c7`)

### Le défaut

- **La ligne vide.** `get_or_create_default` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:122`) crée
  la ligne des réglages sans aucun compte : `:129` « `sqlx::query("INSERT IGNORE INTO company_invoice_settings
  (company_id) VALUES (?)")` ». Les colonnes de texte y prennent leur valeur par défaut, **la même que celle que pose la
  finalisation** (schéma, `crates/kesh-db/test-schema/0001_schema_squash.sql:341` « `` `invoice_number_format`
  varchar(64) NOT NULL DEFAULT 'F-{YEAR}-{SEQ:04}' `` », `:344` « `DEFAULT 'Ventes'` », `:345`
  « `DEFAULT '{YEAR}-{INVOICE_NUMBER}'` ») ; `version` naît à 1 (`:346` « `` `version` int(11) NOT NULL DEFAULT 1 `` »).
  La seule différence entre cette ligne et celle de la finalisation, ce sont donc les **comptes**.
- **La finalisation ne la complète pas.** `insert_with_defaults_in_tx` (`company_invoice_settings.rs:1197`) :
  - lit les trois comptes de rôle (`:1209-1234`, `find_singleton_role_in_tx`, `RoleLookupLock::ForUpdate`) ;
  - refuse si le débiteurs ou le produit manque **avant même de regarder la ligne** : `:1240` « `if receivable.is_none()
    || revenue.is_none() {` » → `InactiveOrInvalidAccounts` ;
  - lit les quatre comptes marqués par le plan (`:1245` « `let designated = chart_designated_accounts(tx,
    company_id).await?;` ») ;
  - écrit par `INSERT IGNORE` (`:1250-1256`, sept comptes : débiteurs, produit, créanciers, arrondi, escompte, frais
    bancaires, pertes) — **sans effet** sur une ligne existante (`:1271` « `let inserted = rows == 1;` ») ;
  - sur une ligne existante (`:1280` « `if rows == 0 {` »), relit la ligne **jointe** aux comptes débiteurs et produit
    actifs (`:1294` « `JOIN accounts ar ON ar.id = cis.default_receivable_account_id AND ar.active = TRUE` », `:1295`
    même jointure pour le produit) : une ligne aux comptes vides ne se joint pas, et `:1305` « `None =>
    Err(DbError::InactiveOrInvalidAccounts),` ».
- **Le motif faux.** La route traduit ce refus en `AppError::Validation` (`crates/kesh-api/src/routes/onboarding.rs:1005-1010`),
  `:1008` « `"Impossible de pré-remplir les comptes de facturation (1100, 3000 manquants du plan comptable). \` » —
  rendu `400 VALIDATION_ERROR`. Les comptes existent ; le message ment.
- **Les comptes que la finalisation pose sur une ligne neuve** (réponse à la question du prompt) : débiteurs et produit
  (obligatoires, par rôle), créanciers (facultatif, par rôle `Payable`, `:1225-1234`, « `// OPTIONNEL (non fail-fast)` »), arrondi, escompte, frais
  bancaires, pertes (facultatifs, marqués par le plan, `chart_designated_accounts`, `:500`). Elle **ne pose pas**
  les trois comptes de TVA : `onboarding_path_b_e2e.rs` (`full_path_b_flow`, `:465`) le vérifie — « `assert_eq!(vat_payable,
  None, "TVA due non désignée en production");` » ; seule la démonstration les désigne (`designate_vat_accounts_in_tx`,
  `company_invoice_settings.rs:1480`, appelée par `crates/kesh-seed/src/lib.rs:421`).
- **Précédent d'un complément sans écrasement** : `designate_vat_accounts_in_tx`, `:1487` « `default_vat_payable_account_id
  = COALESCE(default_vat_payable_account_id, ?), \` » — mais il n'incrémente pas `version` (`:1476-1478`, ligne neuve de
  la même transaction) ; notre complément porte sur une ligne que d'autres ont lue.
- **Ce qui incrémente `version`** : seul `update` (`:410` « `default_bad_debt_account_id = ?, version = version + 1 \` »),
  c'est-à-dire un `PUT /api/v1/company/invoice-settings` qui change quelque chose (court-circuit no-op `:394` « `if is_no_op_change(&before, &changes) {` »).
  Commande : `/usr/bin/grep -n "version = version + 1\|UPDATE company_invoice_settings" crates/kesh-db/src/repositories/company_invoice_settings.rs`
  → `:400`, `:410`, `:1486` (le `:1486` est `designate_vat_accounts_in_tx`, sans `version`).
- **Ce qui protège déjà l'installation finalisée** : `finalize_inner` sort avant toute écriture si l'étape 8 est
  atteinte — `onboarding.rs:966` « `if onboarding.step_completed == 8 {` », `:968` « `return
  response_with_stub(pool, onboarding).await;` ».
- **Le motif est reconnu par la 36-17 au code, non au texte** (fiche 36-17, lecture seule,
  `/home/gcorbaz/devel/kesh-36-17/_bmad-output/implementation-artifacts/36-17-assistant-fin-configuration.md`, § *Une
  limite connue…*, « reconnu au code, non au texte ») : garder `400 VALIDATION_ERROR` garde les deux stories
  indépendantes.

### Inventaire par le symptôme : ce qui crée la ligne vide avant la finalisation

Symptôme : une ligne `company_invoice_settings` **existe** avant la finalisation **et ses comptes sont vides**. Commande,
à la tête du worktree :

```
$ git grep -n "INSERT IGNORE INTO company_invoice_settings\|get_or_create_default(\|get_or_create_default_in_tx(" -- 'crates/*/src/*' | /usr/bin/grep -v "^\S*:[0-9]*:\s*//"
crates/kesh-api/src/exports/global.rs:335:    let cis_row = company_invoice_settings::get_or_create_default(pool, company_id)
crates/kesh-api/src/exports/global.rs:344:    let cds_row = company_dunning_settings::get_or_create_default(pool, company_id)
crates/kesh-api/src/routes/company_invoice_settings.rs:300:    let settings = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;
crates/kesh-api/src/routes/company_invoice_settings.rs:336:    let current = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;
crates/kesh-db/src/repositories/company_dunning_settings.rs:36:pub async fn get_or_create_default(
crates/kesh-db/src/repositories/company_dunning_settings.rs:41:    let settings = get_or_create_default_in_tx(&mut tx, company_id).await?;
crates/kesh-db/src/repositories/company_dunning_settings.rs:48:pub async fn get_or_create_default_in_tx(
crates/kesh-db/src/repositories/company_dunning_settings.rs:173:    let settings = get_or_create_default_in_tx(tx, company_id).await?;
crates/kesh-db/src/repositories/company_invoice_settings.rs:122:pub async fn get_or_create_default(
crates/kesh-db/src/repositories/company_invoice_settings.rs:129:    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
crates/kesh-db/src/repositories/company_invoice_settings.rs:156:pub async fn get_or_create_default_in_tx(
crates/kesh-db/src/repositories/company_invoice_settings.rs:161:    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
crates/kesh-db/src/repositories/company_invoice_settings.rs:333:    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
crates/kesh-db/src/repositories/company_invoice_settings.rs:1250:        "INSERT IGNORE INTO company_invoice_settings \
crates/kesh-db/src/repositories/credit_notes.rs:434:            company_invoice_settings::get_or_create_default_in_tx(&mut tx, company_id).await?;
crates/kesh-db/src/repositories/invoices.rs:2156:            company_invoice_settings::get_or_create_default_in_tx(&mut tx, company_id).await?;
crates/kesh-db/src/repositories/supplier_invoices.rs:1651:        company_invoice_settings::get_or_create_default_in_tx(&mut *tx, company_id).await?;
```

Tri, site par site (les `company_dunning_settings` sont une autre table, traitée plus bas) :

| Site | Crée une ligne vide **durable** avant la finalisation ? | Raison (ligne lue) |
|---|---|---|
| `routes/company_invoice_settings.rs:300` (`GET`) | **oui** | tout rôle authentifié ; le frontend l'appelle par `getInvoiceSettings()` (liste ci-dessous) |
| `routes/company_invoice_settings.rs:336` (`PUT`) puis `update` `:333` | **oui**, et la ligne porte alors les choix de l'utilisateur (`version` ≥ 2 si un champ change) | `:336` « `let current = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;` » |
| `exports/global.rs:335` (export global) | **oui** | lecture sur le pool, commitée par `get_or_create_default` |
| `invoices.rs:2156` (validation d'une facture) | non | ligne vide → `:2158` « `let receivable_account_id = settings.default_receivable_account_id.ok_or_else(|| {` » → `ConfigurationRequired`, transaction annulée |
| `credit_notes.rs:434` (avoir) | non | `:435-437` exige le produit (`ConfigurationRequired`), et un avoir exige une facture validée, donc une ligne déjà désignée |
| `supplier_invoices.rs:1651` (validation fournisseur) | non | `:1725-1726` « `.ok_or_else(|| DbError::ConfigurationRequired("default_payable_account_id".into()))?;` », transaction annulée |

Côté frontend, l'application est ouverte dès l'étape 6 (`frontend/src/routes/(app)/+layout.ts:42` « `if (!isDemo && step
< 6) {` ») et ces écrans appellent le `GET` :

```
$ git grep -n "getInvoiceSettings" -- frontend/src ':!*.test.ts' | /usr/bin/grep -v "export\|import\|//"
frontend/src/lib/components/invoices/InvoiceForm.svelte:408:				const settings = await getInvoiceSettings();
frontend/src/lib/components/invoices/InvoiceForm.svelte:620:			const freshSettings = await getInvoiceSettings();
frontend/src/routes/(app)/bank-accounts/+page.svelte:163:			getInvoiceSettings(),
frontend/src/routes/(app)/invoices/[id]/+page.svelte:30:		getInvoiceSettings,
frontend/src/routes/(app)/invoices/[id]/+page.svelte:289:			invoiceSettings = await getInvoiceSettings();
frontend/src/routes/(app)/journal-entries/+page.svelte:160:		void getInvoiceSettings()
frontend/src/routes/(app)/journal-entries/[id]/+page.svelte:306:				getInvoiceSettings(),
frontend/src/routes/(app)/settings/invoicing/+page.svelte:11:		getInvoiceSettings,
frontend/src/routes/(app)/settings/invoicing/+page.svelte:193:			const [s, a] = await Promise.all([getInvoiceSettings(), fetchAccounts(true)]);
frontend/src/routes/(app)/settings/invoicing/+page.svelte:293:						const fresh = await getInvoiceSettings();
```

(`:30` et `:11` sont des lignes d'import à plusieurs lignes, non des appels.)

**Pourquoi la correction ne dépend pas de cet inventaire** : elle porte sur le **consommateur** (la finalisation), non
sur les producteurs. Quel que soit l'écran ou la route qui a créé la ligne, elle arrive à la finalisation dans l'un de
trois états — **absente** ; **vierge** (`version = 1`, les dix comptes vides : seul `get_or_create_default` ou un
`INSERT IGNORE` nu la produit) ; **enregistrée** (`version` ≥ 2 : un `PUT` a changé quelque chose). La correction traite
les trois ; un producteur neuf ou oublié ne rouvre rien. La sonde positive est le test HTTP (T-H1), qui crée la ligne par
le vrai `GET` et rougit sans le correctif.

**Les autres réglages créés « par défaut » avant la finalisation**, que celle-ci ignore ensuite :

```
$ git grep -n "INSERT IGNORE" -- 'crates/*/src/*' | /usr/bin/grep -v "^\S*:\s*//"
```
(sortie : `post_restore.rs:1642` et `:1652-1657` — un test du détecteur P7, texte de test ; `company_dunning_settings.rs:52`,
`:77` ; `company_invoice_settings.rs:129`, `:161`, `:333`, `:1250` ; `credit_note_number_sequences.rs:36`,
`invoice_number_sequences.rs:54`, `journal_entry_number_sequences.rs:79` ; `vat_rates.rs:309` ; `test_fixtures.rs:180`,
`:334`.)

| Réglage | Verdict | Raison |
|---|---|---|
| `company_dunning_settings` (`:52`, `:77`) | hors défaut | la table n'a **aucune colonne de compte** (`/usr/bin/grep -n "CREATE TABLE \`company_dunning_settings\`" -A15 …squash.sql \| /usr/bin/grep -i account` : vide) et la finalisation ne la crée pas : une ligne paresseuse y est l'état normal |
| `vat_rates` (`vat_rates.rs:309`) | hors défaut | un `INSERT IGNORE` **par taux** (clé `uq_vat_rates_company_rate_valid_from`, squash `:1332`) : un taux que l'utilisateur a saisi avant est gardé, les autres sont semés — c'est le comportement voulu, non une ligne vide |
| `fiscal_years::create_if_absent_in_tx` (`fiscal_years.rs:456`) | hors défaut | un exercice créé par l'utilisateur avant la finalisation est le sien ; aucune valeur vide à compléter |
| séquences de numéros (`*_number_sequences.rs`) | hors défaut | créées à l'usage, sans lien avec la finalisation |

### Le risque que la correction ouvrirait (#987)

Aux étapes 6 et 7, *Administration → Comptes bancaires* lit les réglages (`bank-accounts/+page.svelte:163`) — ce qui crée
la ligne vierge — puis `POST /bank-accounts` ne refuse de lier un compte bancaire qu'aux comptes de créance **désignés**
(`claim_accounts_in_share_mode`, `company_invoice_settings.rs:231`), vides à ce moment-là : lier le compte bancaire au
compte 1100 passe. Aujourd'hui, la finalisation refuse ensuite (la ligne existe). **Compléter naïvement la ligne
désignerait 1100 comme compte débiteurs alors qu'il est le compte d'un compte bancaire**, l'état que la Story 15-6c
interdit. La route le savait déjà pour l'étape 6 : `onboarding.rs:737-739` « `/// ⚠️ Angle mort (validation P2 de la
fiche, issue demandée) : entre l'étape 6` … `/// compte qui vient d'être lié ; la finalisation ne le contrôle pas.` » —
c'est **#987** (`bug`, P3, ouverte) : « Attendu : la finalisation, et toute désignation de rôle, refusent un compte lié
à un compte bancaire ». La garde existe dans le dépôt : `refuse_if_linked_to_bank_account`
(`company_invoice_settings.rs:258`), qui lit `bank_accounts::first_active_bank_account_linked_to`
(`bank_accounts.rs:1676`, `LOCK IN SHARE MODE`). D6 l'emploie.

## Décisions

- **D1 — Corriger la finalisation, non les producteurs.** La finalisation complète la ligne qu'elle trouve. *Écartées* :
  empêcher `get_or_create_default` de créer la ligne avant la finalisation (le `GET` deviendrait une lecture d'une ligne
  absente : six producteurs, frontend compris, à changer, et le `PUT` d'avant la finalisation resterait) ; supprimer la
  ligne vide à la finalisation puis l'insérer (perdrait les choix d'un `PUT`).
- **D2 — Ce qui se complète, et quand** (C-36-19-1). Une valeur **non vide** n'est **jamais** remplacée (`COALESCE`).
  - **Débiteurs et produit** (obligatoires) : complétés par leur compte de rôle **dès qu'ils sont vides**, quel que soit
    l'état de la ligne — la finalisation ne peut aboutir sans eux, et un réglage vide n'est pas une désignation.
  - **Créanciers, arrondi, escompte, frais bancaires, pertes** (facultatifs) : complétés **seulement si la ligne est
    vierge** — `version = 1` **et** les dix colonnes de compte vides. Une ligne enregistrée (`version` ≥ 2) garde ses
    facultatifs vides : l'utilisateur a enregistré l'écran avec eux, et `insert_with_defaults_does_not_touch_existing_settings`
    (`crates/kesh-db/tests/company_invoice_settings_repository.rs:1140`, « un second appel ne réécrit pas le réglage que
    l'utilisateur a vidé ») et `…_does_not_touch_existing_write_off_settings` (`:1260`) restent verts **sans être
    modifiés** (leur ligne a ses comptes : elle n'est pas vierge).
  - Les trois comptes de **TVA** ne sont jamais posés (comme pour une ligne neuve, `full_path_b_flow`).
  *Écartées* : compléter tout vide toujours (réécrit un réglage vidé et casse les deux tests cités) ; ne compléter que les
  deux obligatoires (une ligne vierge resterait sans créanciers ni comptes d'écart, ce que la fiche 36-17 appelle
  « l'effet résiduel ») ; un marqueur « ligne créée par défaut » (exigerait une migration, C-e360-3).
- **D3 — Le rôle n'est cherché que pour un réglage vide** (C-36-19-2). Un débiteurs ou un produit **désigné et actif**
  suffit, même si aucun compte ne porte plus le rôle : le refus anticipé `:1240` ne s'applique qu'aux réglages vides.
  Un réglage obligatoire désigné mais **archivé** n'est pas remplacé : refus (D5). *Écartée* : garder le refus anticipé
  (refuse une ligne enregistrée valable — le même symptôme : refus alors que rien ne manque).
- **D4 — Atomique, idempotent, audité, `version` incrémentée** (C-36-19-3). Dans la transaction de la finalisation,
  après les comptes (ordre des verrous inchangé jusque-là : `onboarding_state → companies → accounts`), la ligne existante
  se lit en `FOR UPDATE`, puis **un seul** `UPDATE … SET col = COALESCE(col, ?) …, version = version + 1 WHERE company_id
  = ?` **si et seulement si** au moins une colonne change ; sinon aucune écriture. Rejouée (interblocage,
  `retry_app_on_deadlock`, `onboarding.rs:894`) ou appelée deux fois, elle ne change rien de plus. Un complément écrit
  **une** entrée d'audit `company_invoice_settings.updated` (action existante, que l'écran du journal d'audit connaît)
  avec `{ "trigger": "onboarding", "before": …, "after": … }` au format de `settings_snapshot_json`
  (`company_invoice_settings.rs:95`, à rendre `pub`), par `NewAuditLogEntry::from_current_user` comme
  `company_invoice_settings.created` (`onboarding.rs:1149-1153`), à la même place dans la séquence. *Écartées* : une
  action neuve `…completed` (libellés à créer dans quatre langues) ; pas de `version` (un écran ouvert avant la
  finalisation renverrait la `version` 1 et **effacerait** les comptes complétés au premier *Enregistrer* — avec la
  `version` incrémentée, il reçoit un 409 et recharge).
- **D5 — Le motif dit ce qui manque** (C-36-19-4). Le refus reste **`400 VALIDATION_ERROR`** (la 36-17 le reconnaît au
  code) ; son message nomme chaque réglage obligatoire en défaut et sa cause, en français (comme les autres refus de la
  route ; quatre langues : hors périmètre, limite #710 déjà écrite au manuel, `user-manual.tex:213`). Gabarit exact :
  « La configuration ne peut pas se terminer — {causes, séparées par « ; »}. Corrigez-le dans Administration →
  Facturation ou dans le plan comptable, puis réessayez. » Causes :
  - réglage vide et aucun rôle : « compte débiteurs : le réglage est vide et aucun compte actif du plan comptable ne
    porte le rôle « Créances clients » » ; « compte de produit par défaut : le réglage est vide et aucun compte actif du
    plan comptable ne porte le rôle « Produit par défaut » » (libellés des rôles :
    `crates/kesh-i18n/locales/fr-CH/messages.ftl:219` « `account-role-receivable = Créances clients` », `:220`
    « `account-role-default-revenue = Produit par défaut` ») ;
  - réglage désigné mais archivé : « compte débiteurs : le compte désigné ({numéro}) est archivé » (produit : de même) ;
  - compte de rôle lié à un compte bancaire (D6) : « compte débiteurs : le compte {numéro}, qui porte le rôle « Créances
    clients », est le compte du compte bancaire « {libellé} » » (créanciers : « compte créanciers … « Dettes
    fournisseurs » … »).
  Forme : une variante neuve de `DbError` (par exemple `DefaultInvoiceAccountsUnavailable(Vec<…>)`, avec son code
  dans `error_code` et un bras dans le rendu de `kesh-api`), rendue **par une fonction neuve** que seule la finalisation
  appelle ; `insert_with_defaults_in_tx` en devient l'enveloppe et la **ramène** à `InactiveOrInvalidAccounts`, si bien
  que la démonstration (`kesh-seed/src/lib.rs:417`, `pieges.rs:399`, et la boucle de rejeu `lib.rs:329`), le refus de
  `seed_demo` (`onboarding.rs:269`) et les tests existants (`company_invoice_settings_repository.rs:118`, `:172`,
  `:916`, `:948`, `:1368`) gardent leur contrat **et kesh-seed n'est pas touché**. *Écartées* : remplacer le variant
  partout (cinq tests, la démonstration et sa boucle de rejeu à reprendre) ; un diagnostic relu par la route après le
  refus (deux jugements du même état à tenir égaux) ; un code neuf (la 36-17 ne le reconnaîtrait pas).
- **D6 — Un compte de rôle lié à un compte bancaire n'est pas désigné** (C-36-19-5 ; partie « finalisation » de #987,
  `refs #987`). Quand la finalisation tire du **rôle** le compte débiteurs ou le compte créanciers — ligne neuve, ou
  complément —, elle refuse (D5) si ce compte est le compte du grand livre d'un compte bancaire non archivé, par la garde
  existante (`first_active_bank_account_linked_to`). Une valeur **déjà désignée** n'est pas recontrôlée (le `PUT` l'a
  contrôlée ; la garde à l'usage de la 15-6b reste le filet). *Écartées* : laisser le créancier vide en silence (#987
  demande le refus) ; refuser avec `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (la 36-17 ne reconnaît que
  `VALIDATION_ERROR`). La partie « toute désignation de rôle » de #987 (`PUT /api/v1/accounts/{id}`) reste à #987.
- **D7 — Aucune migration, aucune écriture sur une installation finalisée** (C-e360-3). Le correctif ne touche que la
  transaction de la finalisation, qui sort avant toute écriture à l'étape 8 (`onboarding.rs:966-968`). Aucune ligne
  existante n'est réécrite par une mise à jour ; l'installation de Guy (finalisée) n'est pas concernée.

## Critères d'acceptation

1. **AC1 — Ligne vierge complétée.** Ligne créée par `GET /api/v1/company/invoice-settings` à l'étape 6 ou 7, puis
   finalisation : `200`, `stepCompleted = 8` ; les sept comptes (débiteurs, produit, créanciers, arrondi, escompte, frais
   bancaires, pertes) **égaux** à ceux d'une société de même plan finalisée sans `GET` préalable ; les trois comptes de
   TVA vides ; `version` = 2.
2. **AC2 — Une désignation n'est jamais remplacée.** Ligne enregistrée par un `PUT` (débiteurs = un autre compte d'actif
   imputable, arrondi vidé) avant la finalisation : le débiteurs reste celui du `PUT`, le produit est complété, l'arrondi
   (facultatif, ligne enregistrée) reste vide ; aucune autre colonne ne change.
3. **AC3 — Le rôle n'est cherché que pour un vide.** Ligne enregistrée avec débiteurs et produit désignés et actifs,
   **aucun** compte ne portant le rôle `Receivable` : la finalisation aboutit.
4. **AC4 — Le motif dit ce qui manque.** (a) débiteurs vide, rôle absent ; (b) produit désigné archivé ; (c) compte de
   rôle débiteurs lié à un compte bancaire : `400 VALIDATION_ERROR`, message au gabarit de D5 nommant le seul réglage en
   défaut et sa cause ; il ne contient plus « 1100, 3000 manquants » ; aucune écriture (ligne, `version`, audit, étape
   inchangés).
5. **AC5 — Idempotent et audité.** Une seconde finalisation (étape 8) ne change rien ; un appel répété de la fonction du
   dépôt sur une ligne complète rend « inchangé », sans `UPDATE` ni `version` changée. Le journal d'audit porte **une**
   `company_invoice_settings.updated` (`trigger` = `onboarding`, `before` aux comptes vides, `after` aux comptes posés) à
   la place de `company_invoice_settings.created` ; une ligne neuve garde `…created` (séquence de `full_path_b_flow`
   inchangée).
6. **AC6 — Écran ouvert avant : 409, pas d'effacement.** Après le complément, un `PUT` portant `version: 1` rend `409`,
   et les comptes complétés restent.
7. **AC7 — Contrats gardés.** Tous les tests existants de `company_invoice_settings_repository.rs`,
   `onboarding_path_b_e2e.rs`, `onboarding_audit_e2e.rs` et de la démonstration passent **sans modification** de leurs
   assertions ; `git diff --stat origin/epic/36-0.18.0...HEAD -- crates/kesh-seed frontend/src` est vide.
8. **AC8 — E2E.** `frontend/tests/e2e/onboarding-finalize-reglages.spec.ts` (neuf) : preset `fresh`, assistant jusqu'à
   l'étape du compte bancaire, `/journal-entries` ouvert dans le navigateur (attente de la réponse de
   `GET /api/v1/company/invoice-settings`), retour à `/onboarding`, *Configurer plus tard* ; puis, **par l'API**
   (`authedApiContext`) : `stepCompleted = 8` (`expect.poll`) et débiteurs / produit égaux aux comptes de rôle. Aucune
   assertion sur l'écran d'après la finalisation (que la 36-17 change).
9. **AC9 — Manuel.** `docs/manual/fr/user-manual.tex`, § *Onboarding*, paragraphe `:217` : une phrase dit qu'à la
   finalisation Kesh désigne aussi les comptes par défaut de facturation que le plan fournit, même si un écran a été
   ouvert avant, sans remplacer un compte déjà choisi, et que, s'il refuse, le message nomme le réglage en défaut. PDF
   régénéré à l'intégration (contrôle aplati, forme E6, de cette phrase).
10. **AC10 — Périmètre.** Aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 91) ; au plus quatre modules
    (Dev Notes).

## Tests à écrire, et la preuve du rouge

Les rouges se journalisent dans `/home/gcorbaz/devel/kesh-gate-logs/e36-19/rouges.log` (tests écrits et lancés **avant**
le correctif, ou correctif mis de côté par `git stash` **après** le commit du code — consigne E5) ; les mutations dans
`/home/gcorbaz/devel/kesh-gate-logs/e36-19/mut/`, script versionné à côté, restauration par `git restore --worktree --
<fichier>` puis `touch <fichier>`.

**Dépôt** (`crates/kesh-db/tests/company_invoice_settings_repository.rs`, helpers `company_with_pme_chart` `:853`,
`company_with_chart` `:1043`) :

- **T-D1** ligne vierge (`get_or_create_default`) puis `insert_with_defaults_in_tx` : rend « inséré = false »,
  sept comptes égaux à ceux d'une seconde société PME finalisée à neuf, TVA vide, `version` 2 (AC1). **Rouge** :
  `InactiveOrInvalidAccounts` (`:1305`).
- **T-D2** ligne enregistrée (`update`) : débiteurs choisi gardé, produit complété, arrondi vide gardé (AC2). **Rouge** :
  `InactiveOrInvalidAccounts`.
- **T-D3** débiteurs et produit désignés, rôle `Receivable` retiré (`UPDATE accounts SET role = NULL …`) : succès,
  rien d'écrit (AC3). **Rouge** : `:1240`.
- **T-D4** les trois refus de AC4 par la fonction neuve, chacun nommant son réglage et sa cause ; ligne et `version`
  inchangées après `rollback` ; l'enveloppe rend `InactiveOrInvalidAccounts` pour les trois. **Rouge** : la variante
  n'existe pas (compilation) — le rouge se prouve alors par mutation (M5).
- **T-D5** appel répété sur ligne complétée : « inchangé », `version` égale, aucun `UPDATE` (`updated_at` égal) (AC5).

**HTTP** (`crates/kesh-api/tests/onboarding_path_b_e2e.rs`, à côté de `full_path_b_flow` `:353`, mêmes helpers
`spawn_app` `:45`, `login` `:82`, `advance_to_step_2` `:99`) :

- **T-H1** étapes 2 à 7, `GET /api/v1/company/invoice-settings` (comptes `null`), `POST /onboarding/finalize` → `200`,
  étape 8, réglages relus complétés, séquence d'audit avec `company_invoice_settings.updated` à la place de `…created`
  (AC1, AC5). **Rouge** : `400` « 1100, 3000 manquants ».
- **T-H2** même chemin, rôle `Receivable` retiré avant la finalisation : `400 VALIDATION_ERROR`, message de D5 (AC4a).
  **Rouge** : ancien message.
- **T-H3** même chemin, `POST /api/v1/bank-accounts` lié au compte de rôle débiteurs à l'étape 7 : `400`, message de D6
  (AC4c). **Rouge** : sans le correctif, `400` avec l'ancien message ; avec un correctif sans D6, **`200`** et un compte
  débiteurs lié à un compte bancaire (c'est M6).
- **T-H4** après T-H1, `PUT` en `version: 1` → `409` (AC6).

**E2E** : AC8. **Rouge** : `stepCompleted` reste 7 (la finalisation rend 400) — journal du run rouge dans `rouges.log`.

**Mutations** (chacune doit faire rougir au moins un test nommé) :

| # | Mutation | Attendu rouge |
|---|---|---|
| M1 | retirer l'`UPDATE` de complément | T-D1, T-H1 |
| M2 | `COALESCE(col, ?)` → `?` sur le débiteurs | T-D2 |
| M3 | supprimer la condition « ligne vierge » des facultatifs | T-D2, tests existants `:1140`, `:1260` |
| M4 | retirer `version = version + 1` | T-D1, T-H4 |
| M5 | ramener le message à l'ancien texte / ne plus distinguer les causes | T-D4, T-H2 |
| M6 | retirer la garde du compte lié (D6) | T-H3 |
| M7 | rétablir le refus anticipé pour un réglage désigné (`:1240`) | T-D3 |
| M8 | ne pas écrire l'audit du complément | T-H1 |

## Tâches

- [ ] **T0 — Étape 0** : relire les lignes citées à la tête de la branche (elles peuvent bouger à la fusion de la
  36-7a) ; reprendre la sortie des deux commandes d'inventaire ; citer au Change Log ce qui a bougé.
- [ ] **T1 — Bases** (voie 2 ; la station a redémarré, bases vides) : `kesh_e3619` et `kesh_e2e_e3619` créées sur
  `kesh-mariadb-dev2` (`docker exec kesh-mariadb-dev2 mariadb -uroot -pkesh_dev_root …`), migrées (`sqlx migrate run`,
  `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3619`), semées (`scripts/seed-dev-db.sql`).
- [ ] **T2 — Rouges** : T-D1 à T-D3, T-H1 à T-H3, E2E écrits et lancés avant le correctif (`rouges.log`).
- [ ] **T3 — Dépôt** (`company_invoice_settings.rs`) : fonction neuve (insertion ou complément, D2, D3, D4, D6), rendu
  de l'issue (inséré / complété avec `before` / inchangé) ; `insert_with_defaults_in_tx` en enveloppe (D5) ;
  `settings_snapshot_json` `pub` ; doc-comments de `:1-17`, `:1114-1164`, `:1186-1196` mis à jour ; variante de
  `DbError` et son code (`errors.rs`, `error_code` `:1906`).
- [ ] **T4 — Route** (`onboarding.rs`) : appel de la fonction neuve (`:999`), message de D5 (`:1005-1010`), audit du
  complément dans `audit_finalize_seed` (`:1139`) ; doc-comments `:854-867`, `:985-997`, `:907-912` (« `INSERT
  IGNORE` … idempotent ») et l'angle mort `:737-739` (fermé côté finalisation, renvoi à #987) ; bras de rendu de la
  variante dans `crates/kesh-api/src/errors.rs` (`400 VALIDATION_ERROR`).
- [ ] **T5 — Tests** T-D4, T-D5, T-H4 ; mutations M1 à M8.
- [ ] **T6 — Documentation** : `user-manual.tex:217` (AC9) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md:333` (ordre des
  verrous de `finalize` : ajouter la ligne des réglages en `FOR UPDATE` et `bank_accounts` en partagé, s'ils y
  entrent).
- [ ] **T7 — Gates** (Dev Notes, *Gates*) et Dev Agent Record.

## Note de version

*Pour l'orchestrateur, qui l'assemble dans `## [0.18.0] — Non publié` (C-e360-11) ; cette branche n'écrit pas
`CHANGELOG.md`.*

**Corrigé**

- Assistant de configuration : la mise en place n'est plus refusée (« 1100, 3000 manquants ») quand vous avez ouvert le
  journal, les comptes bancaires, une facture ou les réglages de facturation avant de la terminer. Kesh désigne alors les
  comptes par défaut de facturation que votre plan comptable fournit, sans remplacer un compte que vous avez déjà choisi ;
  s'il doit refuser, le message dit quel réglage manque et pourquoi. Une installation déjà configurée n'est pas touchée.
  (refs #1200)
- Assistant de configuration : la mise en place refuse de désigner comme compte débiteurs ou créanciers un compte lié à
  un compte bancaire. (refs #987)

## Dev Notes

### Hors périmètre, et les issues

- **#987, partie « désignation de rôle »** : `PUT /api/v1/accounts/{id}` qui pose le rôle `Receivable` ou `Payable` sur
  un compte lié à un compte bancaire — reste à #987, que la PR porte en `refs`, non en `closes`. À l'orchestrateur de
  dire s'il faut la scinder.
- **Message en français seulement** (D5) : limite des refus de la route, déjà écrite (`user-manual.tex:213`, #710).
- **Facultatifs vides d'une ligne enregistrée** (D2) : restent à désigner dans *Administration → Facturation* ; la
  validation d'une facture fournisseur sans créanciers le dit déjà (`supplier_invoices.rs:1725-1726`).
- **Un réglage facultatif désigné puis archivé** n'est pas contrôlé par la finalisation (inchangé : il l'est à l'usage).
- Aucune issue neuve à ouvrir à la rédaction.

### Indépendance avec la 36-17 (#1187), et ce que l'intégration relit

- La 36-19 ne touche **aucun** fichier de `frontend/src` ; la 36-17 ne touche aucun fichier de `crates/` hors locales
  (AC11 de sa fiche). Le refus garde `400 VALIDATION_ERROR`, que la 36-17 reconnaît au code. Les E2E sont dans des
  fichiers distincts (le spec de la 36-19 est neuf).
- **Fichier commun : `docs/manual/fr/user-manual.tex`, § *Onboarding*** — la 36-19 modifie le paragraphe `:217`, la
  36-17 ajoute du texte après `:217` (sa fiche, AC10) : union à l'intégration, PDF régénéré après.
- **À l'intégration, si la 36-19 est fusionnée** (fiche 36-17, Note de version et Dev Notes) : la note « Limite
  connue » de la 36-17 n'est pas gardée ; la phrase de remède de son écran d'échec (« Si les comptes 1100 et 3000
  figurent dans votre plan comptable, ce refus vient d'un écran de Kesh ouvert avant la fin de la configuration… »)
  devient fausse et se retire ; la **limite connue** des deux manuels écrite par la 36-17 (`user-manual.tex`, § *Onboarding*,
  « terminer la configuration avant d'utiliser Kesh … fait refuser la mise en place » ; `admin-manual.tex`, après
  `:1461`, la même limite en une phrase) **disparaît** ; la bannière « Configuration incomplète » peut garder son
  invitation à terminer la configuration. Ces phrases ne sont pas dans la base de cette branche
  (`git grep -n "1100, 3000\|pré-remplir les comptes" -- docs/manual/fr/*.tex` : vide) ; la 36-19 n'a donc rien à y
  retirer elle-même.

### Fichiers et modules

Modules (règle du `CLAUDE.md`) : **quatre** — `kesh-db` (`repositories/company_invoice_settings.rs`, `errors.rs`, tests),
`kesh-api` (`routes/onboarding.rs`, `errors.rs`, tests), le frontend pour **un spec E2E seulement**, les documents
(`docs/manual/fr/user-manual.tex`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`). `kesh-seed` n'est pas touché (D5). Pas de
découpage.

Fichiers communs avec d'autres stories de la 0.18.0 : `user-manual.tex` (36-17, 36-15, 36-18 possibles : paragraphes
distincts) ; `crates/kesh-db/src/errors.rs` (toute story qui ajoute une variante : union) ; `onboarding_path_b_e2e.rs`
(36-17 n'y touche pas). `CHANGELOG.md` : non (C-e360-11).

### Ce qui doit rester vrai

- Une ligne neuve se crée exactement comme avant (mêmes sept comptes, `…created`, séquence de `full_path_b_flow`).
- Aucune valeur non vide n'est réécrite ; aucune écriture à l'étape 8 ; aucune écriture sur refus.
- Ordre des verrous de la finalisation : `onboarding_state → companies → accounts → company_invoice_settings →
  bank_accounts (partagé)` ; rejeu sur interblocage inchangé (`onboarding.rs:894`). Un `PUT` concurrent prend
  `company_invoice_settings` d'abord : l'interblocage possible est rejoué des deux côtés (`company_invoice_settings.rs:384-393`).
- La démonstration et les jeux de pièges gardent leur comportement (`insert_with_defaults_in_tx` en enveloppe) ; ni l'un
  ni l'autre ne crée de compte bancaire avant les réglages (`/usr/bin/grep -n "bank" crates/kesh-seed/src/pieges.rs` :
  aucune ligne avant `:399` ; `lib.rs` n'en crée pas).

### Gates (consignes `e35-dev-commun.md` § *Quels gates, quand* ; voie 2)

- Voie 2 : `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock scripts/mem-guard.sh …`, avec `TMPDIR=/tmp/kesh-voie2`
  et `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3619` exportés ; une commande lourde à la fois (E7).
- **`kesh-db` est touché** : gate **complet** (`scripts/test-fast.sh --ci`, bases remises à neuf) en cours de
  développement et à chaque remédiation, pas de gate ciblé seul.
- Passage en `review` : `scripts/test-fast.sh --ci` (voie 2) puis `scripts/gate-ci.sh kesh_ci_e3619` **en voie 1**
  (`flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh …`) ;
  frontend : `npm run check` et `npm run lint-i18n-ownership` (un spec E2E ajouté, aucun fichier de `src`). Pas d'E2E.
- Clôture : `gate-ci.sh` **et** E2E complet sur `kesh_e2e_e3619` recréée (voie 2, port 3307), au dernier commit de code ;
  échecs nommés contre `docs/testing.md` § « Les échecs attendus ».
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e36-19/`, chacun cité avec son commit.

### Références

- Issue #1200 et son commentaire (déclencheur : tout appel à `getInvoiceSettings()`) ; #987 ; #1187 (36-17).
- Fiche 36-17 (lecture seule) et ses validations `/home/gcorbaz/devel/kesh-gate-logs/e36-17/validation-p1.md`,
  `validation-p2.md`.
- Fichier d'epic, § *Phase 2 — décisions de Guy du 2026-11-15*, entrée 36-19.
- Stories 15-6c (comptes de créance et comptes bancaires), 15-7a1/15-7a2 (inséré, audit de la finalisation), 34-1
  (garde de l'étape 6), 25-4-c3-a2 / 25-4-d1 (comptes marqués par le plan), 31-5 (TVA non désignée en production).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche rédigée (Epic 36, 0.18.0). Branche avancée en avance rapide de `9172e9fe8` à `fed34a9c7`
  (`origin/epic/36-0.18.0`, commit « 36-19 ajoutée à la 0.18.0 »), code lu à cette tête. Inventaires par le symptôme
  exécutés (sorties copiées). #987 trouvée en suivant l'angle mort `onboarding.rs:737-739` ; sa partie « finalisation »
  entre dans D6 (`refs #987`). Registre `e36-choix-autonomes-36-19.md` (C-36-19-1 à 6). Validation non lancée.
