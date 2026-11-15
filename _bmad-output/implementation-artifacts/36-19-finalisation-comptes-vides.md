# Story 36-19 : la finalisation de l'assistant complète les comptes vides des réglages de facturation (#1200)

Status: review

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-19`, branche
`story/36-19-finalisation-comptes-vides`, par le workflow `bmad-create-story`. Release **0.18.0**, **sans migration**.
Code lu à **`fed34a9c7`** (tête de `origin/epic/36-0.18.0`, sur laquelle la branche a été avancée en avance rapide).
Registres : `e36-choix-autonomes-e360.md` (C-e360-3, C-e360-11) ; `e36-choix-autonomes-36-19.md` pour la story
(C-36-19-1 à 8). Bases `kesh_e3619`, `kesh_e2e_e3619` (voie 2), `kesh_ci_e3619` (voie 1). Validation P1 remédiée.*

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
    vierge** — `version = 1` **et** les dix colonnes de compte vides. Les **deux** moitiés comptent : une ligne de
    `version` 2 aux dix comptes vides (un `PUT` qui n'a changé qu'un texte : `update` incrémente dès qu'un champ change,
    `:394`, `:410`) est **enregistrée**, non vierge (T-D2b, M9). Une ligne enregistrée (`version` ≥ 2) garde ses
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
- **D4 — Atomique, idempotent, audité, `version` incrémentée** (C-36-19-3, C-36-19-7). Dans la transaction de la
  finalisation, **un seul ordre, pour une ligne neuve comme pour une ligne existante** (V1-3) :
  1. les comptes candidats d'abord (rôles et comptes du plan, `FOR UPDATE`, comme aujourd'hui : `onboarding_state →
     companies → accounts`) ;
  2. `INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)` **nu** — les colonnes de texte y prennent les
     valeurs que pose l'`INSERT` d'aujourd'hui (squash `:341-345`) ; `rows_affected == 1` dit « ligne neuve » ;
  3. la ligne relue en `FOR UPDATE` ;
  4. les contrôles de D3, D5 et D6 — **après** l'insertion et le verrou de la ligne, donc `company_invoice_settings`
     avant `bank_accounts`, l'ordre du geste bancaire (`claim_accounts_in_share_mode`, `:231`) ; un refus est annulé
     par le `rollback` de la route, la ligne neuve avec lui (T-D4 vérifie la ligne absente après `rollback`) ;
  5. **un seul** `UPDATE … SET col = COALESCE(col, ?) …` **si et seulement si** au moins une colonne change, avec
     `version = version + 1` **seulement si la ligne existait** (une ligne insérée par la même transaction n'a été lue
     par personne : elle garde `version` 1, comme aujourd'hui) ; sinon aucune écriture. Rejouée (interblocage,
  `retry_app_on_deadlock`, `onboarding.rs:894`) ou appelée deux fois, elle ne change rien de plus. Un complément écrit
  **une** entrée d'audit `company_invoice_settings.updated` (action existante, que l'écran du journal d'audit connaît)
  avec `{ "trigger": "onboarding", "before": …, "after": … }` au format de `settings_snapshot_json`
  (`company_invoice_settings.rs:95`, à rendre `pub`), par `NewAuditLogEntry::from_current_user` comme
  `company_invoice_settings.created` (`onboarding.rs:1149-1153`), à la même place dans la séquence. *Écartées* : une
  action neuve `…completed` (libellés à créer dans quatre langues) ; pas de `version` (un écran ouvert avant la
  finalisation renverrait la `version` 1 et **effacerait** les comptes complétés au premier *Enregistrer* — avec la
  `version` incrémentée, il reçoit un 409 et recharge).
- **D5 — Le motif dit ce qui manque, et comment le défaire** (C-36-19-4, C-36-19-8). Le refus reste **`400
  VALIDATION_ERROR`** (la 36-17 le reconnaît au code). Son message, construit par la route en français (comme les autres
  refus de la route ; limite #710 déjà écrite au manuel, `user-manual.tex:213`), nomme chaque réglage **obligatoire** en
  défaut, sa cause **et son remède**. Gabarit exact : « La configuration ne peut pas se terminer. {une phrase par cause,
  séparées par une espace, le débiteurs avant le produit} Réessayez ensuite. » Phrases (libellés des rôles :
  `crates/kesh-i18n/locales/fr-CH/messages.ftl:219` « `account-role-receivable = Créances clients` », `:220`
  « `account-role-default-revenue = Produit par défaut` ») :
  - réglage vide, aucun rôle : « Compte débiteurs : le réglage est vide et aucun compte actif du plan comptable ne porte
    le rôle « Créances clients » — désignez-le dans Administration → Facturation, ou attribuez ce rôle à un compte dans
    le plan comptable. » (produit : « Compte de produit par défaut : … « Produit par défaut » — … », même remède) ;
  - réglage désigné mais archivé : « Compte débiteurs : le compte désigné ({numéro}) est archivé — désignez-en un autre
    dans Administration → Facturation, ou réactivez-le dans le plan comptable. » (produit : de même) ;
  - réglage désigné sur un compte d'une **autre société** (revue P1, B6/E3, inatteignable par l'API) : « Compte
    débiteurs : le compte désigné n'est pas un compte de cette société — désignez-en un autre dans Administration →
    Facturation. » ;
  - compte de rôle non imputable (D6, V2-2) : « Compte débiteurs : le compte {numéro}, qui porte le rôle « Créances
    clients », n'est pas imputable — désignez un compte imputable dans Administration → Facturation, ou, dans le plan
    comptable, retirez d'abord ce rôle au compte {numéro}, puis attribuez-le à l'un de ses sous-comptes. » (produit : « Compte de produit par défaut : … « Produit par
    défaut » … », même remède) ;
  - compte de rôle lié à un compte bancaire (D6, débiteurs seulement) : « Compte débiteurs : le compte {numéro}, qui
    porte le rôle « Créances clients », est le compte du compte bancaire « {libellé} » — déliez ce compte bancaire dans
    Administration → Comptes bancaires (s'il est clos, rouvrez-le d'abord), ou désignez un autre compte débiteurs dans
    Administration → Facturation. » (la parenthèse : revue P1, E4 — la garde retient les comptes clos, dont le lien ne
    se change pas sans réouverture)
  Forme : une variante neuve de `DbError` (par exemple `DefaultInvoiceAccountsUnavailable(Vec<…>)` ; chaque élément :
  le réglage, la cause, le numéro du compte et, pour D6, le libellé du compte bancaire), avec son code dans
  `error_code` (`crates/kesh-db/src/errors.rs:1906`) : **`"VALIDATION_ERROR"`**, égal au code du rendu (V2-4). **Bras de rendu dans `crates/kesh-api/src/errors.rs`** (V1-6 ; match exhaustif ; aucune
  route ne l'atteint, la finalisation convertissant elle-même le refus) au patron de `InactiveOrInvalidAccounts`
  (`errors.rs:4692-4699`, « `"error-inactive-accounts",` » / « `"Un ou plusieurs comptes sont archivés ou invalides.",` ») :
  `400 VALIDATION_ERROR`, **clé neuve** `error-default-invoice-accounts-unavailable` dans les **quatre** locales, à côté
  de `error-inactive-accounts` (`messages.ftl:333` en `fr-CH`, `de-CH`, `it-CH`, `en-CH`), valeur `fr-CH` « Les comptes
  par défaut de facturation ne sont pas utilisables : terminez leur désignation dans Administration → Facturation. »,
  **repli égal à la valeur `fr-CH`** ; `parity_between_locales` vert. La variante est rendue **par une fonction neuve** que seule la finalisation
  appelle ; `insert_with_defaults_in_tx` en devient l'enveloppe et la **ramène** à `InactiveOrInvalidAccounts`, si bien
  que la démonstration (`kesh-seed/src/lib.rs:417`, `pieges.rs:399`, et la boucle de rejeu `lib.rs:329`), le refus de
  `seed_demo` (`onboarding.rs:269`) et les tests existants (`company_invoice_settings_repository.rs:118`, `:172`,
  `:916`, `:948`, `:1368`) gardent leur contrat **et kesh-seed n'est pas touché**. *Écartées* : remplacer le variant
  partout (cinq tests, la démonstration et sa boucle de rejeu à reprendre) ; un diagnostic relu par la route après le
  refus (deux jugements du même état à tenir égaux) ; un code neuf (la 36-17 ne le reconnaîtrait pas).
- **D6 — Un candidat invalide n'est jamais désigné ; seuls les deux obligatoires font refuser** (C-36-19-5 ; arbitrage
  de l'orchestrateur sur V1-2, choix prudent ; partie « finalisation » de #987, `refs #987`). Quand la finalisation tire
  un compte du **rôle** ou du plan — ligne neuve, ou complément —, elle écarte un candidat invalide :
  - pour le débiteurs et le créanciers, le compte du grand livre d'un compte bancaire non archivé (garde existante
    `first_active_bank_account_linked_to`) ;
  - pour les **trois comptes de rôle** (débiteurs, produit, créanciers), un compte **non imputable** — filtre neuf
    (validation P2, V2-2, arbitrage de l'orchestrateur). Les comptes de rôle ne sont aujourd'hui filtrés que sur
    `active` : la colonne générée ne lit que lui (squash `:86`, « `` `singleton_role` varchar(32) GENERATED ALWAYS AS
    (case when `active` <> 0 and `role` in ('Receivable','DefaultRevenue','Payable', `` … »), et créer un sous-compte
    rend son parent non imputable sans toucher à son rôle (`accounts.rs:205` « `UPDATE accounts SET postable = FALSE
    WHERE id = ? AND company_id = ?` ») : un 1100 scindé aux étapes 6 ou 7 serait désigné. Le filtre se pose **dans la
    fonction neuve** (lecture de `postable` du candidat par une requête **verrouillante**, `SELECT … FOR UPDATE` sur la
    ligne que la recherche du rôle a déjà verrouillée : une lecture simple lirait, sous `REPEATABLE READ`, le cliché de
    la première lecture non verrouillante de la transaction, et un sous-compte créé entre ce cliché et le verrou
    laisserait le candidat lu imputable — validation P3, V3-1 ; T-D4s, M14), **non** dans
    `find_singleton_role_in_tx` (`accounts.rs:419`), que partage la garde de l'étape 6 (`refuse_future_claim_account`,
    `onboarding.rs:740`) ;
  - les quatre comptes marqués par le plan le sont déjà à la lecture : `designated_account_id` (`:558`) exige « `AND
    postable = TRUE AND account_type IN (?, ?)` » (`:568`).
  - **Débiteurs et produit** (obligatoires) : un candidat invalide fait **refuser** la finalisation (D5) — elle ne peut
    aboutir sans eux. Seul le débiteurs peut être lié à un compte bancaire ; l'un ou l'autre peut être non imputable.
  - **Créanciers, arrondi, escompte, frais bancaires, pertes** (facultatifs) : un candidat invalide laisse le réglage
    **vide, sans refuser la finalisation** ; l'utilisateur le désigne plus tard dans *Administration → Facturation*. Un
    créanciers lié à un compte bancaire n'est donc jamais désigné d'office, et ne bloque pas l'assistant d'une personne
    qui n'a aucune facture fournisseur à tenir ; la validation d'une facture fournisseur sans créanciers le dit déjà
    (`supplier_invoices.rs:1725-1726`).
  Une valeur **déjà désignée** n'est pas recontrôlée (le `PUT` l'a contrôlée ; la garde à l'usage de la 15-6b reste le
  filet). *Écartées* : refuser la finalisation pour un créanciers lié (lettre de #987, « refusent » : bloquerait
  l'assistant pour un réglage facultatif) ; refuser avec `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` (la 36-17 ne
  reconnaît que `VALIDATION_ERROR`). La partie « toute désignation de rôle » de #987 (`PUT /api/v1/accounts/{id}`) reste
  à #987.
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
   **AC2 bis** — ligne de `version` 2 aux **dix comptes vides** (un `PUT` qui n'a changé que le format de numéro) :
   débiteurs et produit complétés ; créanciers, arrondi, escompte, frais bancaires, pertes **restés vides**.
3. **AC3 — Le rôle n'est cherché que pour un vide.** Ligne enregistrée avec débiteurs et produit désignés et actifs,
   **aucun** compte ne portant le rôle `Receivable` : la finalisation aboutit.
4. **AC4 — Le motif dit ce qui manque, et comment le défaire.** (a) débiteurs vide, rôle absent ; (b) produit désigné
   archivé ; (c) compte de rôle débiteurs lié à un compte bancaire ; (e) débiteurs **et** produit vides, aucun des deux
   rôles : `400 VALIDATION_ERROR`, message au gabarit de D5 — une phrase par réglage en défaut, débiteurs d'abord, séparées
   par une espace, chacune avec sa cause et son remède (pour (c) : « déliez ce compte bancaire dans Administration →
   Comptes bancaires ») ; il ne contient plus « 1100, 3000 manquants » ; aucune écriture (ligne — absente si elle était
   neuve —, `version`, audit, étape inchangés).
   (f) compte de rôle débiteurs **non imputable** (un sous-compte créé sous lui) : même refus, phrase « n'est pas
   imputable » de D5.
   **AC4 bis (d)** — compte de rôle **créanciers** lié à un compte bancaire, ligne vierge ou neuve : la finalisation
   **aboutit** (`200`, étape 8), débiteurs et produit posés, **créanciers vide** (D6). **(g)** — compte de rôle
   créanciers non imputable : de même, créanciers vide.
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
   *Amendé à la remédiation de la revue P1 (B1/E5, B3, arbitrage de l'orchestrateur)* : `crates/kesh-seed/src/lib.rs`
   change par **trois commentaires seulement** (séquence de verrous de `final_body`, et les deux commentaires de la
   désignation de la TVA, qui disaient « ligne neuve seulement ») ; aucune ligne de code de `kesh-seed` ne change, le
   correctif étant porté par l'enveloppe (`kesh-db`). Le diff de `frontend/src` reste vide.
8. **AC8 — E2E.** `frontend/tests/e2e/onboarding-finalize-reglages.spec.ts` (neuf) : preset `fresh`, assistant jusqu'à
   l'étape du compte bancaire, `/journal-entries` ouvert dans le navigateur (attente de la réponse de
   `GET /api/v1/company/invoice-settings`), retour à `/onboarding`, *Configurer plus tard* ; puis, **par l'API**
   (`authedApiContext`) : `stepCompleted = 8` (`expect.poll`) et débiteurs / produit égaux aux comptes de rôle. Aucune
   assertion sur l'écran d'après la finalisation (que la 36-17 change).
9. **AC9 — Manuel.** `docs/manual/fr/user-manual.tex`, § *Onboarding*, paragraphe `:217` : une phrase dit qu'à la
   finalisation Kesh désigne aussi les comptes par défaut de facturation que le plan fournit, même si un écran a été
   ouvert avant, sans remplacer un compte déjà choisi, et que, s'il refuse, le message nomme le réglage en défaut ; et
   qu'un compte **facultatif** inutilisable (un créanciers lié à un compte bancaire, un compte non imputable) n'est pas
   désigné : le réglage reste vide, à désigner dans *Administration → Facturation* (V2-6). PDF
   régénéré à l'intégration (contrôle aplati, forme E6, de cette phrase).
10. **AC10 — Périmètre.** Aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` = 91) ; au plus cinq modules
    (Dev Notes).
11. **AC11 — Rendu de la variante.** Le bras de `kesh-api/src/errors.rs` rend `400 VALIDATION_ERROR` avec la clé
    `error-default-invoice-accounts-unavailable`, présente dans les quatre locales, repli égal à la valeur `fr-CH` ;
    `parity_between_locales` (`crates/kesh-i18n/src/loader.rs`) et `npm run lint-i18n-ownership` verts. **Preuves**
    (V2-4) : la clé est **inscrite à la table G9** (`crates/kesh-api/tests/textes_coherents.rs:745`, « `const TABLE:
    [(&str, Cles); 9] = [` », entrée du fichier `crates/kesh-api/src/errors.rs`, un site, sans variable), ce qui compare
    le repli au catalogue ; un **test unitaire du bras** au patron de `kesh-api/src/errors.rs:5669-5672` (« `assert_eq!(err.error_code(),
    code, "repli de error_code()");` » puis statut, `body["error"]["code"]` et message égal à `texte(clé)`) — tue M12.

## Tests à écrire, et la preuve du rouge

Les rouges se journalisent dans `/home/gcorbaz/devel/kesh-gate-logs/e36-19/rouges.log` (tests écrits et lancés **avant**
le correctif, ou correctif mis de côté par `git stash` **après** le commit du code — consigne E5) ; les mutations dans
`/home/gcorbaz/devel/kesh-gate-logs/e36-19/mut/`, script versionné à côté, restauration par `git restore --worktree --
<fichier>` puis `touch <fichier>`.

**Dépôt** (`crates/kesh-db/tests/company_invoice_settings_repository.rs`, helpers `company_with_pme_chart` `:853`,
`company_with_chart` `:1043`) :

- **T-D1** ligne vierge (`get_or_create_default`) puis `insert_with_defaults_in_tx` : rend « posés = true » (« inséré =
  false » jusqu'à la remédiation de la revue P1, B1/E5 : le booléen de l'enveloppe dit désormais « comptes posés sur une
  ligne qui n'en avait aucun », ce sur quoi la démonstration désigne la TVA),
  sept comptes égaux à ceux d'une seconde société PME finalisée à neuf, TVA vide, `version` 2 ; **la seconde société
  (ligne neuve) a `version` = 1** (C-36-19-7 ; tue M10) (AC1). **Rouge** :
  `InactiveOrInvalidAccounts` (`:1305`).
- **T-D2** ligne enregistrée (`update`) : débiteurs choisi gardé, produit complété, arrondi vide gardé (AC2). **Rouge** :
  `InactiveOrInvalidAccounts`.
- **T-D2b** ligne créée par `get_or_create_default`, puis `update` qui ne change qu'`invoice_number_format`
  (`version` 2, dix comptes vides) : débiteurs et produit complétés, les cinq facultatifs restés vides (AC2 bis).
  **Rouge** : `InactiveOrInvalidAccounts` ; tue M9.
- **T-D3** débiteurs et produit désignés, rôle `Receivable` retiré (`UPDATE accounts SET role = NULL …`) : succès,
  rien d'écrit (AC3). **Rouge** : `:1240`.
- **T-D4** les refus de AC4 (a), (b), (c), (e) par la fonction neuve : la variante porte, dans l'ordre débiteurs puis
  produit, chaque réglage en défaut et sa cause ; ligne et `version` inchangées après `rollback` (ligne **absente** si
  elle était neuve) ; l'enveloppe rend `InactiveOrInvalidAccounts` pour chacun. **Rouge à l'exécution** (V1-7) : le
  développement commite d'abord un **squelette** — la variante déclarée, et la fonction neuve qui délègue au code
  d'aujourd'hui et rend donc `InactiveOrInvalidAccounts` —, lance T-D4 (rouge : variante attendue, autre variante reçue),
  journalise dans `rouges.log`, puis écrit le correctif.
- **T-D4d** compte de rôle créanciers lié à un compte bancaire (ligne vierge, puis ligne neuve) : succès, créanciers
  vide, débiteurs et produit posés (AC4 bis). **Rouge** : sur la ligne vierge, `InactiveOrInvalidAccounts` ; tue M6b.
- **T-D4f** ligne neuve, sous-compte créé sous le compte de rôle débiteurs (`accounts::create` avec `parent_id`, qui rend
  le parent non imputable, `accounts.rs:204-209`), puis de même sous le compte de rôle **produit** (V3-2) : refus, cause
  « non imputable » (AC4 f). Il lit un champ de la variante : il s'écrit **après le squelette**, avec T-D4 (V3-3).
  **Rouge à l'exécution** : sous le squelette, `InactiveOrInvalidAccounts` n'est pas rendu — le code d'aujourd'hui
  **réussit**, le parent non imputable désigné ; tue M11 (débiteurs et produit).
- **T-D4s** (V3-1) ligne neuve ; la transaction de la fonction ouvre d'abord une lecture **non verrouillante** des
  comptes (cliché `REPEATABLE READ`), puis, sur le pool, un sous-compte est créé et commité sous le compte de rôle
  débiteurs, puis la fonction est appelée dans la transaction : refus, cause « non imputable ». Tue M14 (lecture de
  `postable` rendue non verrouillante : le cliché le dit imputable).
- **T-D4g** même geste sous le compte de rôle créanciers : succès, créanciers vide (AC4 bis g). **Rouge** : aujourd'hui,
  créanciers désigné sur le parent non imputable ; tue M11.
- **T-D5** appel répété sur ligne complétée : « inchangé », `version` égale, aucun `UPDATE` (`updated_at` égal) (AC5).

**HTTP** (`crates/kesh-api/tests/onboarding_path_b_e2e.rs`, à côté de `full_path_b_flow` `:353`, mêmes helpers
`spawn_app` `:45`, `login` `:82`, `advance_to_step_2` `:99`) :

- **T-H1** étapes 2 à 7, `GET /api/v1/company/invoice-settings` (comptes `null`), `POST /onboarding/finalize` → `200`,
  étape 8, réglages relus complétés, séquence d'audit avec `company_invoice_settings.updated` à la place de `…created`
  (AC1, AC5). **Rouge** : `400` « 1100, 3000 manquants ».
- **T-H2** même chemin, rôle `Receivable` retiré avant la finalisation : `400 VALIDATION_ERROR`, message exact de D5
  (AC4a). **Rouge** : ancien message.
- **T-H2e** même chemin, rôles `Receivable` **et** `DefaultRevenue` retirés : message exact à **deux** phrases, débiteurs
  puis produit, séparées par une espace (AC4e). **Rouge** : ancien message ; tue M5b.
- **T-H3** même chemin, `POST /api/v1/bank-accounts` lié au compte de rôle débiteurs à l'étape 7 : `400`, message de D6
  avec son remède « déliez ce compte bancaire dans Administration → Comptes bancaires » (AC4c). **Rouge** : sans le
  correctif, `400` avec l'ancien message ; avec un correctif sans D6, **`200`** et un compte débiteurs lié à un compte
  bancaire (c'est M6a).
- **T-H3d** même chemin, compte bancaire lié au compte de rôle **créanciers** : `200`, étape 8, créanciers vide (AC4 bis).
  **Rouge** : `400` (ancien message) ; tue M6b.
- **T-H4** après T-H1, `PUT` en `version: 1` → `409` (AC6).

**Ajoutés à la remédiation de la revue P1** :

- **T-D4 (h)** deux causes **différentes** (débiteurs désigné archivé, produit vide sans rôle), débiteurs d'abord (E6) ;
  **T-D4 (i)** compte désigné d'une autre société → `DesignatedOutsideCompany` (B6/E3) ; **T-D4 (e)** contrôle aussi
  l'enveloppe (A7).
- **T-D4a** `finalize_reads_the_designated_account_under_lock` : cliché pris, compte désigné archivé et validé sur le
  pool, refus « archivé » dans la même transaction (E2 ; tue M15).
- **Motif mot pour mot** `finalize_refusal_message_names_each_cause_word_for_word` (`onboarding.rs`, test unitaire) : les
  cinq causes et un refus à deux causes différentes (B5/E6 ; tue M16).
- **T-H5** `seed_demo_after_a_settings_get_designates_the_vat_accounts` (`onboarding_audit_e2e.rs`) : `GET` des réglages,
  puis démonstration → trois comptes de TVA posés, `invoice_settings_created = true` (B1/E5 ; tue M18, qui rétablit la
  ligne d'avant la remédiation).
- **T-H6** `finalize_completion_by_an_api_key_is_attributed_to_the_key` (`onboarding_audit_e2e.rs`) :
  `company_invoice_settings.updated` attribuée à la clé (E7 ; tue M17).

**E2E** : AC8. **Rouge** : `stepCompleted` reste 7 (la finalisation rend 400) — journal du run rouge dans `rouges.log`.

**Mutations** (chacune doit faire rougir au moins un test nommé) :

| # | Mutation | Attendu rouge |
|---|---|---|
| M1 | retirer l'`UPDATE` de complément | T-D1, T-H1 |
| M2 | `COALESCE(col, ?)` → `?` sur le débiteurs | T-D2 |
| M3 | supprimer la condition « ligne vierge » des facultatifs | T-D2, tests existants `:1140`, `:1260` |
| M4 | retirer `version = version + 1` | T-D1, T-H4 |
| M5 | ramener le message à l'ancien texte / ne plus distinguer les causes | T-D4, T-H2 |
| M5b | ne garder que la première cause (ou inverser l'ordre) | T-D4 (e), T-H2e |
| M6a | retirer la garde du compte lié pour le débiteurs (D6) | T-H3 |
| M6b | retirer la garde du compte lié pour le créanciers (le candidat lié désigné) | T-D4d, T-H3d |
| M7 | rétablir le refus anticipé pour un réglage désigné (`:1240`) | T-D3 |
| M8 | ne pas écrire l'audit du complément | T-H1 |
| M9 | « vierge » ramenée aux seuls dix comptes vides (moitié `version = 1` retirée) | T-D2b |
| M10 | incrémenter `version` aussi sur une ligne neuve | T-D1 (assertion `version` 1 de la seconde société) |
| M11 | retirer le filtre `postable` des candidats de rôle (D6), joué sur **chacun** des trois comptes (V3-2) | débiteurs et produit : T-D4f ; créanciers : T-D4g |
| M12 | bras de rendu : autre clé ou autre code que ceux de D5 | test du bras (AC11) |
| M13 | rendre l'`UPDATE` de complément inconditionnel (V3-2) | T-D5 |
| M14 | lecture de `postable` du candidat rendue non verrouillante (V3-1) | T-D4s |
| M15 | lecture du compte **désigné** sans `LOCK IN SHARE MODE` (revue P1, E2) | T-D4a |
| M16 | phrase « archivé » amputée de « dans le plan comptable » (revue P1, E6) | test du motif mot pour mot |
| M17 | `…updated` écrit par `NewAuditLogEntry::user` au lieu de `from_current_user` (revue P1, E7) | T-H6 |
| M18 | l'enveloppe rend `inserted()` au lieu de « posés sur une ligne vierge » (revue P1, B1/E5) | T-H5 |

## Tâches

- [x] **T0 — Étape 0** : relire les lignes citées à la tête de la branche (elles peuvent bouger à la fusion de la
  36-7a) ; reprendre la sortie des deux commandes d'inventaire ; citer au Change Log ce qui a bougé.
- [x] **T1 — Bases** (faites en **voie 1**, `kesh-mariadb-dev:3306`, sur attribution du prompt d'appel — écart déclaré au
  Change Log ; script `kesh-gate-logs/e36-19/bases-a-neuf.sh`) (voie 2 ; la station a redémarré, bases vides) : `kesh_e3619` et `kesh_e2e_e3619` créées sur
  `kesh-mariadb-dev2` (`docker exec kesh-mariadb-dev2 mariadb -uroot -pkesh_dev_root …`), migrées (`sqlx migrate run`,
  `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3619`), semées (`scripts/seed-dev-db.sql`).
- [x] **T2 — Rouges** : T-D1, T-D2, T-D2b, T-D3, T-D4d, T-D4g, T-H1, T-H2, T-H2e, T-H3, T-H3d et l'E2E écrits et
  lancés **avant** le correctif (`rouges.log`) ; T-D4, T-D4f et T-D4s après le squelette (ci-dessous, V3-3), T-D5
  (M13), T-H4 (M4) et le test du bras (M12, AC11) se prouvent par mutation. **Le squelette** porte la variante, son `error_code()` (`kesh-db/src/errors.rs:1906`)
  et son bras de rendu (`kesh-api/src/errors.rs`, match exhaustif sans `_`, dernier bras `DbError::Sqlx(e)` `:5022`),
  sans quoi il ne compile pas.
- [x] **T3 — Dépôt** (`company_invoice_settings.rs`) : fonction neuve (insertion ou complément, D2, D3, D4, D6), rendu
  de l'issue (inséré / complété avec `before` / inchangé) ; `insert_with_defaults_in_tx` en enveloppe (D5) ;
  `settings_snapshot_json` `pub` ; doc-comments de `:1-17`, `:1114-1164`, `:1186-1196` mis à jour, et celui de
  `find_singleton_role_in_tx` (`accounts.rs:405-418`, « Source unique de cette lecture ») : la finalisation y ajoute un
  filtre `postable` (V3-4) ; variante de `DbError` et son code (`errors.rs`, `error_code` `:1906`).
- [x] **T4 — Route** (`onboarding.rs`) : appel de la fonction neuve (`:999`), message de D5 (`:1005-1010`), audit du
  complément dans `audit_finalize_seed` (`:1139`) ; doc-comments `:854-867`, `:985-997`, `:907-912` (« `INSERT
  IGNORE` … idempotent ») et l'angle mort `:737-739` (fermé côté finalisation, renvoi à #987), et `:715-740` (« Les
  rôles sont lus ici **comme la finalisation les lira** » : la finalisation écarte en plus un candidat non imputable,
  V3-4) ; bras de rendu de la
  variante dans `crates/kesh-api/src/errors.rs` (`400 VALIDATION_ERROR`, clé de D5) ; la clé
  `error-default-invoice-accounts-unavailable` dans les quatre `messages.ftl`, à côté de `error-inactive-accounts`
  (`:333`).
- [x] **T5 — Tests** T-D4 (squelette d'abord), T-D5, T-H4, test du bras et entrée G9 (AC11) ; mutations M1 à M14.
- [x] **T6 — Documentation** : `user-manual.tex:217` (AC9) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md:333` (ligne de
  `finalize` : ajouter la ligne des réglages en `FOR UPDATE` et `bank_accounts` en partagé, **dans l'ordre de la 15-6c**,
  réglages avant `bank_accounts`). Cet ordre contredit le bloc « Global Lock Order » du même document
  (`:316-318` : « `5. bank_accounts` » avant « `6. company_invoice_settings` ») ; écart **préexistant**, non réconcilié
  dans la story : une issue est demandée à l'orchestrateur (Dev Notes).
- [x] **T7 — Gates** (Dev Notes, *Gates*) et Dev Agent Record.

## Note de version

*Pour l'orchestrateur, qui l'assemble dans `## [0.18.0] — Non publié` (C-e360-11) ; cette branche n'écrit pas
`CHANGELOG.md`.*

**Corrigé**

- Assistant de configuration : la mise en place n'est plus refusée (« 1100, 3000 manquants ») quand vous avez ouvert le
  journal, les comptes bancaires, une facture ou les réglages de facturation avant de la terminer. Kesh désigne alors les
  comptes par défaut de facturation que votre plan comptable fournit, sans remplacer un compte que vous avez déjà choisi ;
  s'il doit refuser, le message dit quel réglage manque et pourquoi. Une installation déjà configurée n'est pas touchée.
  (refs #1200)
- Assistant de configuration : la mise en place ne désigne plus comme compte débiteurs ou créanciers un compte lié à un
  compte bancaire. Pour le compte débiteurs, elle refuse et dit comment faire (délier le compte bancaire, ou choisir un
  autre compte) ; le compte créanciers reste alors vide, à désigner dans Administration → Facturation. De même, elle
  ne désigne plus un compte devenu non imputable (un compte sous lequel vous avez créé un sous-compte) : pour le compte
  débiteurs ou le compte de produit par défaut, elle refuse et dit comment faire (désigner un compte imputable, ou
  retirer d'abord le rôle à ce compte puis l'attribuer à un sous-compte) ; le compte créanciers reste alors vide.
  (refs #987)

## Dev Notes

### Hors périmètre, et les issues

- **#987, partie « désignation de rôle »** : `PUT /api/v1/accounts/{id}` qui pose le rôle `Receivable` ou `Payable` sur
  un compte lié à un compte bancaire — reste à #987, que la PR porte en `refs`, non en `closes`. À l'orchestrateur de
  dire s'il faut la scinder.
- **Message en français seulement** (D5) : limite des refus de la route, déjà écrite (`user-manual.tex:213`, #710). Le
  bras de `errors.rs`, lui, est traduit (clé de D5), mais aucune route ne l'atteint.
- **Créanciers laissé vide quand son compte de rôle est lié à un compte bancaire** (D6, arbitrage de l'orchestrateur) :
  écart assumé à la lettre de #987 (« refusent »), à dire sur l'issue.
- **Refus de `seed_demo`** (`onboarding.rs:269-275`, « `"Comptes par défaut introuvables (1100, 3000). \` », même
  formule que l'ancien refus) : **hors périmètre**, inchangé. L'enveloppe lui rend toujours `InactiveOrInvalidAccounts`
  pour un refus ; le texte n'y est faux que pour un plan sans rôles, ce que la démonstration ne produit pas. Aucune
  issue demandée. *Corrigé à la remédiation de la revue P1 (B1/E5)* : cette fiche disait ici qu'aucune ligne vide ne
  précède la démonstration. **C'est faux** : `GET /api/v1/company/invoice-settings` n'a aucune garde d'étape (le
  middleware ne bloque que l'absence d'utilisateur), si bien qu'un appel direct ou une clé d'API crée la ligne vierge
  avant l'étape 2 — l'interface seule l'empêche (`(app)/+layout.ts:42`). Avant la story, la démonstration refusait alors ;
  après le correctif, elle complétait la ligne **sans** désigner sa TVA (le booléen de l'enveloppe disait « non
  insérée »). Arbitrage de l'orchestrateur, choix prudent : l'enveloppe rend « posés » pour une ligne vierge complétée,
  et la démonstration désigne sa TVA comme sur une base neuve (T-H5, M18) ; `kesh-seed` ne change que par ses
  commentaires (AC7 amendé). Écart restant, assumé : la ligne de la démonstration est alors en `version` 2 (complément),
  au lieu de 1.
- **Issue demandée à l'orchestrateur** (V2-3) — « `MULTI-TENANT-SCOPING-PATTERNS.md` : l'ordre global des verrous place
  `bank_accounts` avant `company_invoice_settings`, contraire au code de la 15-6c », étiquettes `documentation`, `P3`.
  Lignes : `docs/MULTI-TENANT-SCOPING-PATTERNS.md:316-318` (« `5. bank_accounts` », « `6. company_invoice_settings` ») ;
  `crates/kesh-api/src/routes/bank_accounts.rs:418-419` (« ordre `companies` → réglages → `accounts` → ») ;
  `crates/kesh-db/src/repositories/company_invoice_settings.rs:231` (`claim_accounts_in_share_mode`, réglages lus avant
  les comptes bancaires).
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
  connue » de la 36-17 n'est pas gardée. **La phrase de remède de son écran d'échec** (« Si les comptes 1100 et 3000
  figurent dans votre plan comptable, ce refus vient d'un écran de Kesh ouvert avant la fin de la configuration… ») :
  **décision d'intégration, prise par l'orchestrateur** (V1-5) ; la fiche 36-17 la dit « gardée, sauf décision
  contraire » (sa ligne 549). Si la 36-19 est dans la même release, la phrase se retire — une fois la 36-19 fusionnée,
  ce refus ne peut plus venir d'un écran ouvert avant —, et ce retrait coûte, côté 36-17 (liste de sa fiche, sans rien
  lui prescrire ici) : le test d'AC4 (`onboarding-end.test.ts`), les mutations M15 et M16, la capture d'AC12 et la ligne
  de T8. Dans le même cas, la **limite connue** des deux manuels écrite par la 36-17 (`user-manual.tex`, § *Onboarding*,
  « terminer la configuration avant d'utiliser Kesh … fait refuser la mise en place » ; `admin-manual.tex`, après
  `:1461`, la même limite en une phrase) **disparaît** ; la bannière « Configuration incomplète » peut garder son
  invitation à terminer la configuration. Ces phrases ne sont pas dans la base de cette branche
  (`git grep -n "1100, 3000\|pré-remplir les comptes" -- docs/manual/fr/*.tex` : vide) ; la 36-19 n'a donc rien à y
  retirer elle-même.

### Fichiers et modules

Modules (règle du `CLAUDE.md`) : **cinq** — `kesh-db` (`repositories/company_invoice_settings.rs`, `errors.rs`, tests),
`kesh-api` (`routes/onboarding.rs`, `errors.rs`, tests), `kesh-i18n` (une clé dans les quatre `messages.ftl`, V1-6), le
frontend pour **un spec E2E seulement**, les documents (`docs/manual/fr/user-manual.tex`,
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`). `kesh-seed` n'est pas touché (D5). À la limite, pas au-delà : pas de
découpage.

Fichiers communs avec d'autres stories de la 0.18.0 : `user-manual.tex` (36-17, 36-15, 36-18 possibles : paragraphes
distincts) ; `crates/kesh-db/src/errors.rs` et `crates/kesh-api/src/errors.rs` (toute story qui ajoute une variante :
union) ; `messages.ftl` ×4 (une clé `error-*` à côté de `:333`, jamais en fin de fichier) ;
`crates/kesh-api/tests/textes_coherents.rs` (table G9, `:745` : toute story qui ajoute un repli ; union ; la 36-19
ajoute une clé à l'entrée existante de `crates/kesh-api/src/errors.rs`, si bien que le `9` de la table ne bouge pas,
V3-4) ; `onboarding_path_b_e2e.rs`
(36-17 n'y touche pas). `CHANGELOG.md` : non (C-e360-11).

### Ce qui doit rester vrai

- Une ligne neuve se crée avec les mêmes valeurs qu'avant (mêmes sept comptes hors candidat écarté par D6, colonnes de
  texte, `version` 1, `…created`, séquence de `full_path_b_flow`), par l'ordre unique de D4.
- Aucune valeur non vide n'est réécrite ; aucune écriture à l'étape 8 ; aucune écriture sur refus.
- Ordre des verrous de la finalisation, **ligne neuve comprise** (D4, étapes 1 à 4) : `onboarding_state → companies →
  accounts → company_invoice_settings → bank_accounts (partagé)` ; rejeu sur interblocage inchangé (`onboarding.rs:894`). Un `PUT` concurrent prend
  `company_invoice_settings` d'abord : l'interblocage possible est rejoué des deux côtés (`company_invoice_settings.rs:384-393`).
- **Interblocage neuf, borné** (validation P2, V2-3) : avec le `FOR UPDATE` de la ligne existante (D4, étape 3), la
  finalisation tient les comptes de rôle en exclusif et attend la ligne des réglages en exclusif ; le `PATCH` de liaison
  d'un compte bancaire, qui ne prend pas la sentinelle `companies` (`routes/bank_accounts.rs:2085-2093`), tient les
  réglages en partagé et attend le compte du grand livre en partagé (ordre « `companies` → réglages → `accounts` →
  `bank_accounts` », `routes/bank_accounts.rs:418-419`). Même cycle avec la validation d'une facture sur une ligne
  enregistrée (réglages en exclusif, `get_or_create_default_in_tx` `:161-168`, puis comptes). Le cycle est **rejoué des
  deux côtés** (`routes/bank_accounts.rs:2061` `retry_app_on_deadlock("bank_accounts::patch", …)` ; `onboarding.rs:894`),
  et une tentative annulée n'écrit rien : **rien de faux**. Aujourd'hui, sur une ligne existante, la finalisation ne
  prend que le partagé de l'`INSERT IGNORE` (`:1250`), compatible : le cycle est neuf, et accepté. *Écartée* : prendre
  la ligne des réglages avant les comptes candidats (ordre de la 15-6c, mais change l'ordre `onboarding_state →
  companies → accounts` que la démonstration partage, `MULTI-TENANT-SCOPING-PATTERNS.md:341`).
- La démonstration et les jeux de pièges gardent leur comportement (`insert_with_defaults_in_tx` en enveloppe) ; ni l'un
  ni l'autre ne crée de compte bancaire avant les réglages (`/usr/bin/grep -n "bank" crates/kesh-seed/src/pieges.rs` :
  aucune ligne avant `:399` ; `lib.rs` n'en crée pas).

### Gates (consignes `e35-dev-commun.md` § *Quels gates, quand* ; voie 2)

*Annoté à la remédiation de la revue P1 (A8)* : le développement et la remédiation ont tourné en **voie 1**
(`kesh-mariadb-dev:3306`, verrous `/tmp/station-heavy.lock` puis `.gate.lock`), sur attribution du prompt d'appel ; ce qui
suit, rédigé pour la voie 2, est gardé tel quel (cf. Change Log, écart d'environnement).

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

Opus 5.5 (`claude-opus-5-5`), agent de développement de l'Epic 36, en autonomie, worktree `kesh-36-19`.

### Debug Log References

Journaux hors dépôt, `/home/gcorbaz/devel/kesh-gate-logs/e36-19/` :

- `rouges.log` (synthèse) ; `rouges-avant-correctif.log` (code `6c4f2229e` + tests de `6fb456e85`) ;
  `rouges-squelette.log` (`57948b58d`) ; `e2e-spec-rouge-avant.log` (binaire d'avant le correctif, sauvé
  `bin/kesh-api-avant-6fb456e85`) ;
- `cible-apres-correctif.log` (`0723be024`) ; `e2e-spec-vert-apres.log` (binaire `bin/kesh-api-0723be024`) ;
- `clippy-1.log` à `clippy-4.log` ; `front-build-1.log`, `front-check-1.log`, `front-lint-i18n-1.log` ;
- `mut/M*.log`, `mut/bilan.txt`, `mut/campagne.log`, `mut/campagne-2.log` (scripts `mut/muter.py`, `mut/jouer.sh`,
  `mut/campagne.sh`) ;
- passage en `review` : `test-fast-ci-92266e805.log`, `gate-ci-92266e805.log` (+ `.sortie`), `bases-a-neuf-92266e805.log`.

### Completion Notes List

- **Commits** (`git log --oneline 6c4f2229e..HEAD`) : `6c4f2229e` étape 0 (fiche) ; `6fb456e85` rouges d'avant correctif ;
  `57948b58d` squelette (variante, code, bras, clé ×4, fonction qui délègue ; tests T-D4, T-D4f, T-D4s, T-D5, bras, G9 ;
  manuel et `MULTI-TENANT-SCOPING-PATTERNS.md` dans le même commit) ; `0723be024` correctif ; `92266e805` test du bras au
  catalogue chargé (M12 survivante) ; puis la fiche (ce commit). **Dernier commit de code : `92266e805`.**
- **Rouges** (`rouges.log`) — avant correctif, 10 rouges sur 50 tests des deux binaires, chacun pour sa raison :
  T-D1, T-D2, T-D2b, T-D3, T-D4d (`InactiveOrInvalidAccounts`), T-D4g (créanciers désigné sur le parent non imputable,
  `left: Some(40)`), T-H1 et T-H3d (`400` au lieu de `200`), T-H2/T-H2e et T-H3 (ancien message « 1100, 3000
  manquants »). Sous le squelette, T-D4 (`InactiveOrInvalidAccounts` au lieu de la variante), T-D4f et T-D4s (`Ok`,
  parent non imputable désigné), T-D5 (le complément refusé) rouges ; le test du bras, G9 et `parity_between_locales`
  verts. E2E (spec seule) contre le binaire d'avant : `Expected: 8 / Received: 7`, 1 échec ; après : 1 réussi.
- **Mutations** (`mut/bilan.txt`), toutes restaurées par `git restore --worktree` puis `touch`, une seule prise des
  verrous pour la campagne (`campagne.sh`) : M1, M2, M3, M4, M5 (route), M5r (dépôt, causes confondues), M5b (route,
  une phrase), M5br (dépôt, une cause), M6a, M6b, M7, M8, M9, M10, M11 (filtre retiré des trois), M11p (produit
  seul), M11t (débiteurs et créanciers), M12c (code du bras), M13, M14 **tuées** par les tests que nomme la table ;
  **M12** (clé du bras) **a survécu** au test unitaire du bras — aucun catalogue n'y est chargé, le repli égal au
  texte de la clé passe — : test neuf `default_invoice_accounts_unavailable_reads_its_catalogue_key`
  (`onboarding_path_b_e2e.rs`, catalogue chargé par `spawn_app`), qui la tue (M12k). Écarts : M4 rejouée sur le dépôt
  seul (T-D1, T-D5), non sur T-H4 (épargne une compilation de `kesh-api`) ; M6a tuée par T-D4 (c) au dépôt et non par
  T-H3 (même raison) ; M11 jouée en trois formes, la fonction de rejet étant commune aux trois comptes. *Complété à la
  remédiation de la revue P1 (A6)* : **M1** n'a été jouée que sur le dépôt (tuée par T-D1 et quinze autres), non sur
  T-H1 ; **M6b** que sur le dépôt (tuée par T-D4d), non sur T-H3d — même raison. Ces deux écarts n'étaient pas déclarés.
- **Choix** : C-36-19-9 (relecture verrouillante de `postable`), C-36-19-10 (compte désigné contrôlé en partagé après la
  ligne), C-36-19-11 (forme de l'issue) au registre.
- **Écarts** : voie 1 au lieu de la voie 2 de la fiche (prompt d'appel) ; `clippy::large_enum_variant` → `before` en
  `Box` ; un seul test HTTP porte T-H1 et T-H4 (même montage), un seul T-H2 et T-H2e. **Consigne Edit** : code, tests et
  fiche par l'outil Edit ; `cargo fmt` ; les scripts de `kesh-gate-logs` (dont deux retouches de `jouer.sh` et
  `campagne.sh` par un `python3` de remplacement, hors dépôt) ; aucun `sed -i`.
- **AC7** : `git diff 6c4f2229e HEAD -- <les quatre fichiers de tests existants>` n'a aucune ligne retirée (ajouts en fin
  de fichier seulement) ; `git diff --stat 6c4f2229e HEAD -- crates/kesh-seed frontend/src` vide. **AC10** :
  `ls crates/kesh-db/migrations/*.sql | wc -l` = 91.
- **Décomptes** (périmètre `6c4f2229e..92266e805`) : `#[sqlx::test]` de `company_invoice_settings_repository.rs` 31 → 41
  (+10) ; de `onboarding_path_b_e2e.rs` 9 → 14 (+5) ; un `#[tokio::test]` neuf (`errors.rs`) ; une spec E2E neuve.
- **Ce qui n'a pas tourné** : l'E2E **complet** et le `gate-ci` de clôture (à la clôture, consignes de l'Epic 35) ; le PDF
  du manuel n'est pas régénéré (AC9 : à l'intégration). Aucune capture : aucun contrôle visible ne change.
- **Issues à ouvrir** (à l'orchestrateur) : #1201 existe déjà pour l'ordre des verrous du document. Aucune neuve.

### File List

- `crates/kesh-db/src/errors.rs` — variante `DefaultInvoiceAccountsUnavailable`, types `DefaultInvoiceSetting`,
  `DefaultInvoiceAccountCause`, `UnavailableDefaultInvoiceAccount`, `error_code()`.
- `crates/kesh-db/src/repositories/company_invoice_settings.rs` — `complete_default_accounts_in_tx`,
  `DefaultAccountsOutcome`, `is_blank`, `role_candidate_rejection`, `mandatory_account` ; enveloppe
  `insert_with_defaults_in_tx` ; `settings_snapshot_json` public ; doc-comments.
- `crates/kesh-db/src/repositories/accounts.rs` — doc-comments de `RoleLookupLock` et `find_singleton_role_in_tx`.
- `crates/kesh-api/src/routes/onboarding.rs` — appel, motif (`finalize_refusal_message`, `finalize_refusal_phrase`),
  audit `…updated`, doc-comments.
- `crates/kesh-api/src/errors.rs` — bras de rendu, test du bras.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — `error-default-invoice-accounts-unavailable`.
- `crates/kesh-api/tests/textes_coherents.rs` — entrée G9.
- `crates/kesh-db/tests/company_invoice_settings_repository.rs`, `crates/kesh-api/tests/onboarding_path_b_e2e.rs` — tests.
- `frontend/tests/e2e/onboarding-finalize-reglages.spec.ts` — spec neuve.
- `docs/manual/fr/user-manual.tex` (`:217`), `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (`:333`).
- `_bmad-output/implementation-artifacts/36-19-finalisation-comptes-vides.md`, `…-review-prompt-p1.md`,
  `e36-choix-autonomes-36-19.md`, `sprint-status.yaml`.

## Change Log

- 2026-11-15 — Fiche rédigée (Epic 36, 0.18.0). Branche avancée en avance rapide de `9172e9fe8` à `fed34a9c7`
  (`origin/epic/36-0.18.0`, commit « 36-19 ajoutée à la 0.18.0 »), code lu à cette tête. Inventaires par le symptôme
  exécutés (sorties copiées). #987 trouvée en suivant l'angle mort `onboarding.rs:737-739` ; sa partie « finalisation »
  entre dans D6 (`refs #987`). Registre `e36-choix-autonomes-36-19.md` (C-36-19-1 à 6). Validation non lancée.
- 2026-11-15 — **Validation P1** (`kesh-gate-logs/e36-19/validation-p1.md`) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW.
  Remédiée par l'agent de rédaction des fiches (Opus 5.5), outil Edit. Prémisses relues à `fed34a9c7` :
  `company_invoice_settings.rs:394`, `:410` (V1-1) ; `:231`, `:258`, `bank_accounts.rs:1676` (V1-2, V1-3) ;
  `errors.rs:4692-4699` et `messages.ftl:333` ×4 (V1-6) ; `onboarding.rs:269-275` (V1-6) ; fiche 36-17 `:549` (V1-5).
  Toutes confirmées, aucune réfutée.
  - V1-1 (MEDIUM) : T-D2b, AC2 bis, M9 ; D2 dit que les deux moitiés de « vierge » comptent.
  - V1-2 (MEDIUM), **arbitrage de l'orchestrateur** : un facultatif au candidat invalide reste vide sans refuser ; seuls
    les deux obligatoires font refuser — D6 récrit, AC4 bis, T-D4d, T-H3d, M6a/M6b, note de version, hors-périmètre
    (écart à la lettre de #987) ; C-36-19-5 récrit.
  - V1-3 (LOW) : ordre unique fixé en D4 (insertion nue, `FOR UPDATE`, contrôles, `UPDATE`), `version` incrémentée
    seulement sur une ligne existante (M10) ; C-36-19-7.
  - V1-4 (LOW) : un remède par cause dans le gabarit de D5 (compte lié : délier ou choisir un autre compte).
  - V1-5 (LOW) : la phrase de remède de la 36-17 devient une décision d'intégration, conséquences de la 36-17 listées.
  - V1-6 (LOW) : bras de rendu spécifié (clé neuve, quatre locales, repli égal à `fr-CH`), AC11 ; `kesh-i18n` devient le
    cinquième module ; `seed_demo` (`onboarding.rs:271`) au hors-périmètre avec sa raison ; C-36-19-8.
  - V1-7 (LOW) : squelette commité d'abord pour un rouge d'exécution de T-D4 ; cas à deux causes (AC4 e, T-H2e, M5b).
- 2026-11-15 — **Validation P2** (`kesh-gate-logs/e36-19/validation-p2.md`) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW.
  Remédiée par l'agent de rédaction des fiches (Opus 5.5), outil Edit. Prémisses relues à `fed34a9c7` : squash `:86`
  (`singleton_role` ne lit que `active`), `accounts.rs:204-209` et `:419-440`, `routes/bank_accounts.rs:418-419`,
  `:2061`, `:2085-2093`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md:312-319`, `textes_coherents.rs:723-760`,
  `kesh-api/src/errors.rs:5022`, `:5665-5675`. Toutes confirmées, aucune réfutée.
  - V2-1 (MEDIUM) : T-D1 porte l'assertion `version` 1 de la seconde société ; M10 tuée par T-D1 seul.
  - V2-2 (MEDIUM), **arbitrage de l'orchestrateur** : filtre `postable` sur les candidats de rôle, dans la fonction
    neuve (non dans `find_singleton_role_in_tx`, partagée par l'étape 6) ; obligatoire → refus, phrase « n'est pas
    imputable » de D5 ; facultatif → vide. D6 corrigée (la phrase « déjà écarté à la lecture » était fausse pour les
    comptes de rôle), AC4 f, AC4 bis g, T-D4f, T-D4g, M11.
  - V2-3 (LOW) : interblocage neuf avec le `PATCH` de liaison (et la validation d'une facture) écrit, borné par les deux
    rejeux ; T6 suit l'ordre de la 15-6c ; l'écart du bloc « Global Lock Order » n'est pas réconcilié : issue demandée.
  - V2-4 (LOW) : `error_code()` = `VALIDATION_ERROR` ; AC11 prouvé par la table G9 et un test du bras (M12).
  - V2-5 (LOW) : C-36-19-3 aligné sur C-36-19-7 ; T2 nomme tous les rouges d'avant correctif ; le squelette porte la
    variante, son code et son bras.
  - V2-6 (LOW) : AC9 dit qu'un facultatif inutilisable reste vide, à désigner dans *Facturation*.
- 2026-11-15 — **Validation P3, ciblée** (`kesh-gate-logs/e36-19/validation-p3-ciblee.md`, Sonnet, une lentille sur
  `d6a9c1b6c`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW. **Validation terminée** : trend P1 0/0/2/5 (Sonnet) → P2 0/0/2/4
  (Opus) → P3 ciblée 0/0/0/4 (Sonnet) ; dernière passe sans rien au-dessus de LOW, aucune remédiation de la P2 dans la
  production (la fiche seule). Les quatre LOW traités à l'étape 0 du développement (agent de développement, Opus 5.5,
  outil Edit), lignes relues à la tête `d6a9c1b6c` (code identique à `fed34a9c7` :
  `git diff --stat fed34a9c7 HEAD -- crates frontend docs` vide) :
  - V3-1 : `company_invoice_settings.rs:1458-1459` (`lock_usable_role_account`) « `SELECT id, account_type, postable FROM
    accounts WHERE company_id = ? AND singleton_role = ? FOR UPDATE` » — le patron d'une lecture de `postable` sous
    verrou existe ; `accounts.rs:430` « `"SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ? ORDER BY id
    LIMIT 1 FOR UPDATE"` » ne lit pas `postable`. D6 dit la lecture verrouillante ; T-D4s et M14 ajoutés. **Correctif de
    production**, couvert par T-D4s.
  - V3-2 : la table des mutations (M1 à M12) ne nommait pas T-D5 ; M13 ajoutée (→ T-D5) ; T-D4f étendu au produit, M11
    jouée sur chacun des trois comptes de rôle.
  - V3-3 : T-D4f déplacé après le squelette (il lit la cause de la variante), T2 récrit.
  - V3-4 : `accounts.rs:415-418` « Source unique de cette lecture : la finalisation de l'assistant
    (`insert_with_defaults_in_tx`, trois rôles, [`RoleLookupLock::ForUpdate`]) et l'étape bancaire » et
    `onboarding.rs:724-725` « Les rôles sont lus ici **comme la finalisation les lira** » ajoutés à T3 et T4 ; note de
    version complétée du refus « non imputable » ; remède de D5 « retirez d'abord ce rôle au compte {numéro}, puis
    attribuez-le à l'un de ses sous-comptes » (`accounts.rs:691` « `return Err(role_conflict(role, holder));` » : un
    rôle singleton porté refuse sa seconde attribution) ; formule G9 : la clé s'ajoute à l'entrée existante de
    `errors.rs`, le `9` ne bouge pas (`textes_coherents.rs:745` « `const TABLE: [(&str, Cles); 9] = [` »).
  - T0 : inventaires rejoués à la tête : 17 lignes (producteurs) et 10 lignes (`getInvoiceSettings`), égales aux
    sorties copiées.
  - **Écart d'environnement** : la fiche dit voie 2 ; le prompt d'appel attribue la **voie 1** (`kesh-mariadb-dev:3306`,
    verrous `/tmp/station-heavy.lock` et `.gate.lock`) : le développement suit le prompt.
- 2026-11-15 — **Développement** (Opus 5.5, outil Edit) : rouges d'abord (`6fb456e85`), squelette (`57948b58d`),
  correctif (`0723be024`), test du bras au catalogue chargé (`92266e805`, mutation M12 survivante). Détail au Dev Agent
  Record. Gates au dernier commit de code `92266e805`, bases remises à neuf (`bases-a-neuf-92266e805.log`) :
  `scripts/test-fast.sh --ci` vert, **5468 réussis, 13 ignorés**, aucun rejeu (`test-fast-ci-92266e805.log`) ;
  `scripts/gate-ci.sh kesh_ci_e3619` vert, code 0, **5475 réussis, 0 échec, 17 ignorés** sur 228 binaires
  (`gate-ci-92266e805.log`, sortie `gate-ci-92266e805.sortie`) ; frontend : `npm run check` 0 erreur (29 avertissements préexistants,
  `front-check-1.log`), `npm run lint-i18n-ownership` vert (`front-lint-i18n-1.log`), `npm run build` vert
  (`front-build-1.log`) — ces trois à `0723be024`, aucun fichier de `frontend/` touché depuis. **Non lancés** : E2E
  complet et `gate-ci` de clôture (à la clôture) ; la spec neuve jouée seule (rouge puis verte). Statut `review`.
- 2026-11-15 — **Revue de code P1** (Opus, trois lentilles ; `kesh-gate-logs/e36-19/review-p1-{B,E,A}.md`) : B 0/0/1/5,
  E 0/0/0/7, A 0/0/0/8 (B1 = E5, E1 = B4, E3 = B6, B5 couvert par E6 : 17 findings distincts). **Remédiée** par l'agent
  de développement (Opus 5.5, outil Edit), commit de code `e19dd034d`, prémisses relues au code ; **aucune réfutée**.
  - **B1 = E5 (MEDIUM)**, arbitrage de l'orchestrateur, choix prudent (C-36-19-12) : prémisse confirmée,
    `kesh-seed/src/lib.rs:418` « `if invoice_settings_created {` » sautait la TVA d'une ligne vierge complétée.
    L'enveloppe rend « posés » (`posed_on_empty_row`) pour une ligne vierge complétée ; T-H5 ; M18 tuée ; doc-comments de
    l'enveloppe et de `designate_vat_accounts_in_tx` ; fiche (Dev Notes, refus de `seed_demo`) corrigée ; T-D1 (mon test)
    passe de « inséré = false » à « posés = true ». AC7 amendé : `kesh-seed` change par trois commentaires, aucun code.
  - **E2** : T-D4a (cliché, archivage validé, refus « archivé ») ; M15 tuée ; doc-comment de `mandatory_account`.
  - **E6 / B5** : test unitaire du motif mot pour mot (cinq causes, refus à deux causes différentes) ; T-D4 (h) au dépôt
    (deux causes différentes) ; M16 tuée.
  - **E7** : T-H6 (finalisation par une clé d'API sur une ligne vierge, `…updated` en `api_key`) ; M17 tuée.
  - **E3 = B6** : cause `DesignatedOutsideCompany` (C-36-19-13), phrase sans « réactivez-le » ; T-D4 (i).
  - **E4** : remède « (s'il est clos, rouvrez-le d'abord) », la garde retenant les comptes clos
    (`bank_accounts.rs:1676` « `AND archived = FALSE` » ; route `POST /api/v1/bank-accounts/{id}/reopen`, `lib.rs:966`).
  - **E1 = B4** : doc-comment de `complete_default_accounts_in_tx` — promotion S → X nommée, primitives nommées juste
    (`retry_on_deadlock` pour le `PUT`, la validation, l'avoir, la facture fournisseur ; `retry_app_on_deadlock` pour la
    finalisation et le `PATCH`) ; la finalisation ajoutée à la liste du commentaire d'`update`.
  - **B2** : commentaires de `finalize` (`INSERT IGNORE remains idempotent`, « trois `FOR UPDATE` », `insert_with_defaults()`).
  - **B3** : séquence de `seed_demo` à `MULTI-TENANT-SCOPING-PATTERNS.md:341` et au doc de `final_body`.
  - **A1** : le commit `57948b58d` (squelette) portait aussi le manuel, `MULTI-TENANT-SCOPING-PATTERNS.md` et l'entrée G9,
    sans que son message les nomme, et ces documents décrivaient un comportement codé seulement à `0723be024`. Constaté,
    l'historique n'est pas réécrit.
  - **A2, A3** : `user-manual.tex:217` (« comptes d'arrondi, d'escompte, de frais bancaires et de pertes » ; « modifié
    et enregistré … un *Enregistrer* sans rien changer ne compte pas »). **A4** : `admin-manual.tex:3848` (exceptions
    de D2 et D6). PDF à l'intégration. **A5** : `MULTI-TENANT-SCOPING-PATTERNS.md:333` renvoie à #1201. **A6** : M1
    et M6b déclarées au Dev Agent Record. **A7** : T-D4 (e) contrôle l'enveloppe. **A8** : en-tête, T1 et *Gates*
    annotés (voie 1).
  - Mutations M15 à M18 (`mut/campagne-r1.log`, `mut/M15.log` à `M18.log`), une prise des verrous, toutes tuées et
    restaurées. Gate de remédiation : `scripts/test-fast.sh --ci` au commit `e19dd034d`, bases remises à neuf :
    vert, **5472 réussis, 13 ignorés**, un rejeu réussi au second essai —
    `kesh-api::mcp_transport_e2e abandon_au_delai_le_permis_reste_pris` (`FLAKY 2/2`), test d'un délai du serveur MCP
    sans rapport avec la story, station chargée (`test-fast-ci-e19dd034d-r1.log`, dont la première ligne porte le
    commit). Ni `gate-ci` ni E2E (consignes : à la clôture).
