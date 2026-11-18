# Story 48-13 : Avant la vraie facture — client écrasé par un chargement tardif, adresse tronquée en silence, nom de société non contrôlé à l'assistant (#1233, #1243, #1226)

Status: review

*Fiche rédigée le 2026-11-18 (Sonnet 5.5), worktree `kesh-48-13`, branche `story/48-13-avant-la-vraie-facture` partie de
`epic/48-corrections-0-20-0` (= `main` = v0.19.0, `773df80da`), par le workflow `bmad-create-story`. Release **0.20.0** (sortie le
vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Story **prioritaire** : elle porte sur le chemin de la première facture client
QR de fin novembre, et **se fusionne avant** les stories de l'Epic 49 qui touchent `InvoiceForm.svelte` (arbitrage de Guy du 2026-11-18).
**Aucune migration** : aucune colonne ne change (`companies.name` `VARCHAR(255)`, adresse en cinq colonnes, inchangées) ; si le
développement en découvre une, il s'arrête et le dit. `#1234` (`billing_information`) est **retirée** de la story et de la 0.20.0 (décision de
Guy du 2026-11-18) : `kesh-qrbill` n'est pas touché.*

*Révisée le 2026-11-18 après les validations P1 et P2 (Opus puis Sonnet) : la story avait été élargie aux troncatures silencieuses des contacts, du fournisseur, de l'article et de l'e-mail
de la société (décision de Guy du 2026-11-18, « diminuer le nombre de bugs »). La validation P2 (V2-2) a relevé qu'elle dépassait la règle de splitting et retardait le correctif qui débloque l'Epic 49 ;
**sur décision de l'orchestrateur, la story est coupée en deux, le périmètre est réparti entre les deux fiches, et le critère du texte tapé (D10) est rétabli ici (AC13)** : **48-13** garde **#1233, #1243, #1226** (chemin de la facture, se fusionne avant l'Epic 49) ;
**`48-15-troncatures-silencieuses`** prend **#1331, #1332, #1333, #1334** et cinq champs du PDF (nom d'article, numéro de client, conditions de paiement, raison sociale, délai de paiement ; cinq depuis la validation P4 de la 48-15, V4-8)
(fiche `48-15-troncatures-silencieuses.md`, qui ne bloque pas l'Epic 49 et part de la 48-13 fusionnée pour `onboarding.rs` et `settings/+page.svelte`).*

## Story

En tant qu'administrateur qui émet des factures QR réelles, je veux que l'écran de facture ne remplace jamais le client que je viens de
choisir, que le nom et l'adresse de ma société ne soient jamais enregistrés coupés ou refusés plus tard par la QR-facture sans que Kesh me le
dise, afin que le premier PDF que je remets sur papier porte ce que j'ai écrit.

## Ce qui se lit au code (`773df80da`)

### #1233 — le chargement tardif du client

- `frontend/src/lib/components/invoices/InvoiceForm.svelte:399-416` : l'effet qui charge le client d'une facture ouverte en modification. Il se
  déclenche une fois par facture (`initialContactInvoiceId`, `:277`) puis, à la réponse, écrit sans condition :
  `:404` `getContact(invoice.contactId)`, `:405` `.then((c) => {`, `:406` `selectedContact = c;`, et au `.catch` (`:408-414`) **pose `errorMsg`**
  (« Impossible de charger le contact … sélectionnez-le manuellement »).
- Le choix de l'utilisateur passe par `onContactSelect` (`:447-461`) : `:450` `selectedContact = c;`, `:453` `errorMsg = '';`, puis préremplit
  conditions et échéance. Rien n'avertit le chargement initial que ce choix a eu lieu. **Deux effets, donc** : si la réponse tardive **réussit**,
  elle remplace le client choisi (le champ suit : `<ContactPicker selected={selectedContact} …>`, `:826-827`, et c'est `selectedContact.id` qui part au
  `:697` `contactId: selectedContact!.id,`) ; si elle **échoue**, elle réécrit le bandeau d'erreur que `onContactSelect` venait d'effacer.
- `reloadFromServer` (`:736-790`) charge aussi un client (`:775` `const c = await getContact(fresh.contactId);`) mais sous la garde `reloadSeq`
  (`:748-749` `if (seq !== reloadSeq) return;`, `:776`), et remplace tout le formulaire par la version du serveur : le remplacement du client y est voulu. Il
  n'est **pas** protégé contre une réponse tardive du chargement **initial** (voir D2).
- Le patron de garde du dépôt est un compteur de génération : `loadGen` (`ReconciliationProposals.svelte:164`, `:183` `const myGen = ++loadGen;`, `:190`
  `if (myGen !== loadGen) return; // stale response, drop`) ; ici même, `settingsSeq` (`InvoiceForm.svelte:283`, `:425` `const seq = ++settingsSeq;`, `:431`
  `if (seq !== settingsSeq) return;`) et `ContactPicker.svelte:156-168` (`searchSeq`).
- **Inventaire par le symptôme** — tous les chargements asynchrones des trois composants de l'écran de facture :

  ```
  $ cd frontend/src/lib/components/invoices && /usr/bin/grep -n "await \|\.then(" InvoiceForm.svelte ProductPicker.svelte ContactPicker.svelte
  InvoiceForm.svelte:73:				const rates = await getVatRates();
  InvoiceForm.svelte:200:				const list = await listProjects();
  InvoiceForm.svelte:234:				const list = await fetchAccounts(true);
  InvoiceForm.svelte:405:			.then((c) => {
  InvoiceForm.svelte:430:				const settings = await getInvoiceSettings();
  InvoiceForm.svelte:644:			const freshSettings = await getInvoiceSettings();
  InvoiceForm.svelte:706:				const updated = await updateInvoice(invoice.id, req);
  InvoiceForm.svelte:713:				const created = await createInvoice(payload);
  InvoiceForm.svelte:748:			const fresh = await getInvoice(invoice.id);
  InvoiceForm.svelte:775:					const c = await getContact(fresh.contactId);
  ProductPicker.svelte:48:			const r = await listProducts({ search: q, includeArchived: false, limit: 50 });
  ContactPicker.svelte:160:			const r = await listContacts({ search: q, limit: 50, includeArchived: false });
  ```

  | Site | Écrit | Peut écraser un choix de l'utilisateur ? | Verdict |
  |---|---|---|---|
  | `:73` taux de TVA | `vatOptions` (liste) ; l'effet `:100-107` ne touche que les lignes `_vatPlaceholder` (v0172, #882) | non : une ligne choisie ou relue garde son taux | sans objet |
  | `:200` projets | `projects`, `projectsLoaded` (liste) ; `projectId` n'est pas écrit | non | sans objet |
  | `:234` comptes | `accounts` (liste) | non | sans objet |
  | `:430` réglages | `invoiceSettings`, déjà gardé par `settingsSeq` | non (déjà gardé) | sans objet |
  | `:644` réglages au submit | `invoiceSettings = freshSettings` (`:661`), sous `submitting` | non : écrit au submit seulement | sans objet |
  | `:706`, `:713` enregistrement | navigation, `submitting` | sans objet | sans objet |
  | `:748`, `:775` rechargement | tout le formulaire, sous `reloadSeq` ; écrase **voulu** (modale de conflit) | voulu | à relire en D2 (réponse tardive du chargement initial) |
  | **`:405` client initial** | **`selectedContact`, `errorMsg`** | **oui, sans aucune garde** | **dans le périmètre** |
  | `ProductPicker.svelte:48` | résultats de recherche | recherche gardée côté `ContactPicker` (`searchSeq`) ; `ProductPicker` : voir D3 | à relire en D3 |
  | `ContactPicker.svelte:160` | résultats de recherche, gardés par `searchSeq` (`:161`, `:164`) | non | sans objet |

  **`getContact` hors du formulaire** (`git grep -n "getContact(" -- frontend/src ':!*.test.ts'`) : `invoices/+page.svelte:92`, `invoices/due-dates/+page.svelte:162`
  (filtre de liste, lu depuis l'URL), `supplier-invoices/[id]/edit/+page.svelte:100` ; aucun n'est un choix de l'utilisateur écrasable : hors périmètre.

### #1243 — les `maxlength` de l'adresse de la société

- `frontend/src/routes/(app)/settings/+page.svelte:821`, `:826`, `:831`, `:836`, `:841` : les cinq champs de l'adresse portent `maxlength={70}`, `{16}`, `{16}`,
  `{35}`, `{2}` ; un texte **collé** plus long est coupé par le navigateur à la saisie, puis enregistré coupé, avec le toast de succès
  (`saveAddress`, `:575-589`, envoie `{ ...addressForm }` tel quel).
- Le **serveur refuse déjà** un champ trop long, en nommant le champ et le nombre : `crates/kesh-api/src/address_input.rs:58-80` `check_lengths` (`:60` `("rue", self.street.trim().chars().count(), STREET_MAX),`
  … `:68` `"adresse : le champ {label} dépasse {max} caractères ({got})"`), et un pays qui n'est pas un code de deux lettres (`:75`, « le pays doit être un code ISO à 2 lettres (ex. CH) »),
  appelé par `validate_required_for_qr` (`:123`) que la route `PUT /companies/current/address` emploie (`companies.rs:371` `let address = req.address.validate_required_for_qr()?;`).
  Sur refus, `saveAddress` garde la saisie et affiche `err.message` (`:621-622`). **Le `maxlength` est donc seul responsable de la coupure** : sans lui, le serveur refuse et la saisie reste.
- Les bornes sont celles de la QR-facture : `kesh-qrbill/src/validation.rs:13-16` `STREET_MAX = 70`, `BUILDING_MAX = 16`, `POSTAL_CODE_MAX = 16`, `TOWN_MAX = 35` ; `address_input.rs:13-16` les répète.
- **Précédent (même terrain)** : `+page.svelte:346-351` — la story v0190-nom-societe a retiré le `maxlength={70}` du nom (revue P1 E3, C-v0190c-25) : « Il tronquait en silence un nom collé trop long ». Le champ du nom
  refuse **localement** (`settings-company-name-too-long`, avec le nombre) ; test d'absence : `settings-page.test.ts:1327-1328` `expect(input.hasAttribute('maxlength')).toBe(false);`. Le même motif est écrit
  pour le pays de l'assistant (`onboarding/+page.svelte:751-753` « Pas de maxlength : « Suisse » doit atteindre le serveur et revenir expliqué, non être tronqué en « Su », accepté »).
- **Inventaire par le symptôme** — tous les `maxlength` des champs qui finissent sur la QR-facture ou le PDF :

  ```
  $ /usr/bin/grep -n "maxlength" 'frontend/src/routes/(app)/settings/+page.svelte' | cut -c1-120
  346:	// Revue P1 (E3, C-v0190c-25) : pas de `maxlength` sur le champ. Il tronquait en silence un
  821:									<Input id="company-address-street" type="text" bind:value={addressForm.street} readonly={addressSubmitting}
  826:									<Input id="company-address-building" type="text" bind:value={addressForm.building} readonly={addressSubmitt
  831:									<Input id="company-address-postal-code" type="text" bind:value={addressForm.postalCode} readonly={addressSu
  836:									<Input id="company-address-city" type="text" bind:value={addressForm.city} readonly={addressSubmitting} cla
  841:									<Input id="company-address-country" type="text" bind:value={addressForm.country} readonly={addressSubmittin
  906:									maxlength={320}
  964:								maxlength={50}
  974:								maxlength={255}
  ```

  (hors réglages : `git grep -n "maxlength" -- frontend/src/lib frontend/src/routes ':!*.test.ts' ':!frontend/src/routes/(app)/settings/+page.svelte'` : **40 lignes, dont 3 commentaires (`bank-accounts/format.ts:88`, `bank-accounts/+page.svelte:866`, `onboarding/+page.svelte:751`) : 37 sites de code**, relevés ci-dessous.)

  | Site | Borne | Finit sur le PDF / la QR ? | Le serveur refuse-t-il ? | Verdict |
  |---|---|---|---|---|
  | `settings/+page.svelte:821-841` adresse société (rue 70, n° 16, NPA 16, localité 35, pays 2) | SIX | oui (créancier) | **oui** (`check_lengths`) | **dans le périmètre (#1243)** |
  | `settings/+page.svelte:964` téléphone (50) | colonne `VARCHAR(50)` | oui (`invoice_pdf_service.rs:426` `creditor_phone: company.phone.clone(),`) | **oui** (`companies.rs:214` `MAX_PHONE_LEN`, `normalize_contact_field` `:251`, clé `error-company-phone-too-long`) | **dans le périmètre** (même défaut, même bloc ; D4) |
  | `settings/+page.svelte:974` site web (255) | colonne `VARCHAR(255)` | oui (`:428` `creditor_website`) | **oui** (`MAX_WEBSITE_LEN`, `error-company-website-too-long`) | **dans le périmètre** (D4) |
  | `settings/+page.svelte:906` e-mail (320) | colonne | oui (`:427` `creditor_email`) | **non** : aucune longueur côté serveur (`contacts.rs:266` `is_valid_email_simple`) | **hors périmètre : 48-15 (#1333)** — le `maxlength={320}` **reste** dans la 48-13 : le retirer sans borne serveur ouvrirait un refus muet (`DATA_LENGTH_OR_RANGE`) |
  | `contacts/+page.svelte:992-1008` adresse du client ; `:872`, `:882` prénom/nom ; `:1066` numéro de client ; `:1105` conditions de paiement | SIX / 70 / 50 / 100 | oui (débiteur de la QR, `invoice_pdf_service.rs:439`, `:418`) | voir 48-15 | **hors périmètre : 48-15 (#1331, #1334 et les cinq champs du PDF)** |
  | `products/+page.svelte:637` nom de l'article (255) | colonne | oui (`InvoiceForm.svelte:482` `description: p.name,`) | oui (`products.rs:187`) | **hors périmètre : 48-15** |
  | `SupplierCreatePanel.svelte:210` pays du fournisseur (2) | ISO | non | oui (`check_lengths`) | **hors périmètre : 48-15 (#1332)** |
  | `products/+page.svelte:646` description de l'article (1000) | colonne | non (le PDF porte `p.name`, `:482`) | — | sans objet |
  | `bank-accounts/+page.svelte:860`, `:1118` et **`onboarding/+page.svelte:814`** (description du compte bancaire, à l'assistant) `BANK_ACCOUNT_DESCRIPTION_MAX` | colonne | non (la QR porte l'IBAN et le nom du créancier, non la description) | oui | hors périmètre : **#1005** (existante, P4) |
  | autres (`ContactPersonsManager`, `BankProfileForm`, `RuleFormModal`, `api-keys`, `projects`, `fiscal-years`, `DunningPauseDialog`, `supplier-invoices/+page.svelte:284`, `ManualMatchModal`, `TransactionSplitModal`, `contacts/+page.svelte:960` e-mail, `:965` téléphone) | diverses | non | — | **hors périmètre ; portés par la 48-15 (« hors périmètre ») avec leurs issues : #1335 (ContactPersonsManager), #1336 (e-mail et téléphone des contacts ; le délai `:1086` est dans le périmètre de la 48-15, V4-8), #1337 (projets), #1338 (rapprochement), #1339 (profils d'import bancaire), #1340 (divers)** |

### #1226 — l'assistant ne contrôle le nom que non vide

- `crates/kesh-api/src/routes/onboarding.rs:526-541` `set_coordinates` : recompose `name` en « prénom nom » (`:533`) ou prend `body.name.trim()` ; **`:536` `if name.is_empty() {`** est le seul contrôle
  (« Le nom (ou prénom + nom) ne peut pas être vide ») ; **`:542` `let address = body.address.validate_required()?;`** : longueurs SIX et NPA/localité non vides **au sens de la QR** (`address_input.rs:107-118`), **mais pas le jeu de caractères**.
- Le contrôle de la route de modification du nom existe et est publié : `crates/kesh-api/src/routes/companies.rs:504` `pub(crate) fn validate_company_name_for_qr(raw: &str) -> Result<String, AppError> {` — rogne, refuse le vide au
  sens de la QR (`champ_vide_pour_qr`), plus de 70 caractères (avec le nombre) et un caractère hors jeu (avec son point de code), dans l'ordre du PDF. Il est `pub(crate)` : `onboarding.rs` l'atteint par `super::companies::`.
- Le jeu de caractères de l'adresse a son équivalent : `address_input.rs:123` `pub fn validate_required_for_qr` (nomme le champ et le caractère), appelé par la modification de l'adresse (`companies.rs:371`) mais **pas par l'assistant**.
  Conséquence : une adresse saisie à l'assistant avec un caractère que la QR-facture refuse (par exemple « ʼ » ou « 张 ») s'enregistre, puis la QR-facture est refusée, et le remède est la correction de l'adresse dans les réglages — le même
  défaut que #1097 pour le nom, qui est **l'origine de #1226**.
- `onboarding.rs:559` `let locked = lock_state_at_step(&mut tx, 5, true).await?;` : la route n'agit **qu'à l'étape 5 de l'assistant** (société provisoire). Une société déjà configurée — celle de la production — ne repasse jamais par là.
- Côté écran : `frontend/src/routes/onboarding/+page.svelte:193-205` ne fait qu'une précontrôle « non vide » (`trim() !== ''`) ; sur refus du serveur, `:222-225` `toast.error(errorMessageOf(err, …))` affiche le message du serveur. Aucun
  `maxlength` n'est posé sur les champs de l'assistant (`:707`, `:714`, `:720`, `:735-755`). **Rien à changer dans l'écran** : le serveur refuse, l'écran le dit.
- **Inventaire par le symptôme** — les contrôles « non vide » ou de longueur des champs qui finissent sur la QR-facture ou le PDF :

  ```
  $ /usr/bin/grep -n "if name.is_empty() {\|validate_required()" crates/kesh-api/src/routes/onboarding.rs
  536:    if name.is_empty() {
  542:    let address = body.address.validate_required()?;
  ```

  | Site | Contrôle aujourd'hui | Manque | Verdict |
  |---|---|---|---|
  | `onboarding.rs:536` nom (ou prénom + nom) | non vide (`is_empty`) | vide au sens QR, 70 caractères, jeu SIX | **dans le périmètre (#1226)** |
  | `onboarding.rs:542` adresse de la société | longueurs, NPA/localité non vides au sens QR | jeu de caractères SIX | **dans le périmètre** (D5 ; défaut consigné au Change Log, sans issue, V1-11) |
  | `onboarding/+page.svelte:193-205` précontrôle `trim() !== ''` | confort d'écran, le serveur juge | — | inchangé (D6) |
  | `companies.rs:371` modification de l'adresse ; `:545` modification du nom | complets (`validate_required_for_qr`, `validate_company_name_for_qr`) | — | sans objet |
  | IDE (`onboarding.rs:547-554`) | `CheNumber::new` | ne finit pas sur la QR | sans objet |
  | contact / client | le PDF refuse explicitement un nom ou un NPA/localité vides ou trop longs (`invoice-pdf-error-empty-client-*`, `invoice_pdf_service.rs:715-723` (`:715` `"invoice-pdf-error-empty-client-name"`)) | — | sans objet (refus explicite au PDF) |

### Factures réelles — ce qui ne doit pas bouger

- Le PDF d'une facture se compose depuis `invoice_pdf_service.rs` (`:281` `name: company.name.clone(),`, `:419` `creditor_name: company.name.clone(),`, `:426-428` téléphone, e-mail, site) : **la story n'y touche pas**, ni à `kesh-qrbill`, ni à
  `kesh-report`. Un PDF archivé d'une facture validée est un blob : il ne se recalcule pas ; une facture validée jamais téléchargée se rend à partir des données de la société, que la story ne modifie pas.
- La story ne **modifie aucune donnée existante** : l'assistant ne tourne qu'à l'étape 5 (`:559`) ; le formulaire d'adresse n'envoie rien tant qu'on ne l'enregistre pas ; l'écran de facture ne change que la **réponse tardive** d'un chargement.
- Le détail et l'édition d'une facture : `routes/(app)/invoices/[id]/+page.svelte` (affichage d'une facture validée, **n'utilise pas** `InvoiceForm`) et `routes/(app)/invoices/[id]/edit/+page.svelte:40` `<InvoiceForm {invoice} />` (le brouillon en modification).

### Manuels

- `docs/manual/fr/user-manual.tex:1520` (« Corriger votre adresse ») dit que NPA, localité et pays se corrigent, qu'un caractère refusé l'est avec son champ et son caractère, mais **ne dit rien des longueurs** : à compléter d'une phrase (voir D7). `:1522`
  (« Corriger le nom de votre entreprise ») dit déjà « 70 caractères au plus », « le message dit pourquoi ». Le manuel d'administration dit au `admin-manual.tex:2701` (« Pays des adresses », section datée de la 0.17.4) que
  `POST /api/v1/onboarding/coordinates` refuse un pays inconnu : **section datée, on n'y ajoute rien** ; la phrase des contrôles de l'assistant va à la section « Passer à la 0.20.0 », posée à l'intégration par l'orchestrateur (dans la Note de version
  ci-dessous). Le manuel utilisateur décrit l'assistant à `user-manual.tex:214` (légende de capture, sans règle de saisie). `git grep -n -i "assistant" -- docs/manual/fr/user-manual.tex` (au développement) dit où l'étape des coordonnées est décrite ; une
  phrase y suffit.
- Aucun manuel ne décrit le chargement du client d'un brouillon : rien à changer pour #1233.

## Décisions

- **D1 — #1233 : un compteur de génération du client, non un indicateur booléen.** `let contactGen = 0;` (non réactif, comme `reloadSeq` et `settingsSeq`). L'effet initial prend `const myGen = ++contactGen;`, et **sa réponse** — `.then` **et** `.catch` —
  ne s'applique que si `myGen === contactGen`. `onContactSelect` fait `contactGen++` à son entrée. Une réponse tardive d'un chargement dont l'utilisateur a déjà fait un choix est **jetée sans bruit** : ni écrasement du client, ni bandeau d'erreur.
  **Option écartée** : un booléen `contactChosen` — il ne couvre pas un second choix, ni le rechargement (D2), et l'idiome du dépôt est le compteur (`loadGen`, `settingsSeq`, `searchSeq`). **Option écartée** : annuler la requête (`AbortController`) — `getContact` ne
  prend pas de signal, et la réponse jetée suffit.
- **D2 — le rechargement sur conflit incrémente aussi le compteur, par précaution : ce geste est **défensif et non rougissant** (V1-2).** `reloadFromServer` fait `contactGen++` avant de charger son propre client ; sa garde `reloadSeq` est conservée telle quelle.
  **Pourquoi non rougissant** : la modale de conflit ne s'ouvre qu'après un envoi (`InvoiceForm.svelte:722` `conflictOpen = true;`), et un envoi exige un client (`:562` `if (!selectedContact) return 'Veuillez sélectionner un contact';`) ; pendant le chargement initial, `selectedContact` ne peut donc être non nul
  que par un choix de l'utilisateur, qui fait déjà `contactGen++` (D1). Aucun test ne peut le tuer : **pas de mutation M3** ; l'AC2 est un **filet de non-régression sans preuve du rouge** (écrit, assumé).
- **D3 — `ProductPicker` : relevé, déjà gardé, rien à changer** (V1-5). `ProductPicker.svelte:42` `let seq = 0;`, `:45` `const s = ++seq;`, `:49` `if (s !== seq) return;` : la réponse d'une recherche plus ancienne est jetée. Pas d'issue.
- **D4 — #1243 : le champ ne tronque plus ; le serveur refuse, l'écran affiche son message.** On retire `maxlength` des cinq champs de l'adresse, du téléphone et du site web de la même section (même défaut, même bloc, serveur qui refuse déjà). L'**e-mail garde** son `maxlength={320}` : le serveur ne le borne pas, c'est la **48-15** (#1333). La saisie reste en cas de refus
  (comportement existant de `saveAddress`, `:621-622`). **Option écartée** : un refus **local** avec le nombre (comme le nom, `settings-company-name-too-long`) — il copierait dans le frontend les bornes que le serveur énonce déjà et exigerait une clé i18n par champ ; le nom l'a fait parce qu'il n'avait ni l'un ni l'autre
  message avant #1097. **Messages** (V2-7) : celui de l'**adresse** est en français littéral (`address_input.rs:68`, #710) et dit le champ et le nombre saisi ; ceux du **téléphone et du site web** sont **localisés** (`crate::errors::t("error-company-phone-too-long", …)`, `companies.rs:281`, quatre catalogues) et ne disent que le maximum (V1-10).

- **D5 — #1226 : le nom est jugé par `validate_company_name_for_qr`, l'adresse par `validate_required_for_qr`.** `set_coordinates` compose le nom comme aujourd'hui (`:526-534`), puis le fait juger par
  `super::companies::validate_company_name_for_qr` à la place de `is_empty` (le nom **composé** « prénom nom » est jugé : c'est lui qui part au PDF), et appelle `validate_required_for_qr` à la place de `validate_required` (jeu de caractères de l'adresse).
  **Pas de seconde fonction** (DRY) : la fonction de la route de modification est la source. L'ordre des refus reste : nom, adresse, IDE, puis verrou d'étape (un 400 de forme précède un refus d'étape, comme aujourd'hui). Le nom enregistré est le nom **rogné** que la
  fonction rend. **Option écartée** : ne traiter que le nom, laisser l'adresse (le défaut est le même et la correction tient en un appel ; le défaut est consigné au Change Log, sans issue (V1-11 : un bug trouvé dans la story se corrige dans la story)).
  **Limite assumée** : le message du nom dit « Le nom de la société … » même pour une personne physique (prénom + nom) ; la phrase reste exacte (le nom de la raison individuelle).
- **D6 — l'écran de l'assistant ne change pas.** Il affiche déjà le message du serveur (`:222-225`) et ne tronque rien ; copier les bornes dans l'écran de l'assistant dupliquerait celles de la QR-facture. **Option écartée** : un compteur de caractères sous le champ.
- **D7 — le manuel** (`.tex` seulement, pas de PDF, régénéré à la release ; la phrase pour le manuel d'administration est dans la Note de version) : (1) `user-manual.tex:1520` reçoit une phrase (« rue 70, numéro 16, NPA 16, localité 35 caractères au plus : un champ trop long est refusé, jamais coupé, et le message nomme le champ et le nombre ») ;
  (2) **V1-9** : `user-manual.tex:1522` (« Un nom saisi à l'assistant et refusé ensuite par la QR-facture se corrige ainsi, sans repasser par l'assistant. ») est **réécrit** : « …se corrigeait ainsi pour une société configurée avant la version~0.20.0 ; depuis, l'assistant refuse un tel nom » (numéro posé à l'intégration) ; (3) une phrase dit que l'assistant applique **les mêmes contrôles** (nom et adresse), placée à **l'item « Coordonnées légales », `user-manual.tex:219`** (qui décrit déjà les refus de l'assistant), non à `:214` (légende de capture). Contrôle : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé ensuite par la QR-facture se corrige ainsi'` avant (`1`) / après.
- **D8 — E2E dans le navigateur pour #1233, #1243 et #1226.** Le défaut du client se voit dans le navigateur (une réponse qui arrive tard) : `page.route` retarde `GET /api/v1/contacts/{id}`. Le défaut de l'adresse se voit dans le navigateur (le champ coupe un texte collé). **#1226 aussi** (V1-1) : la KF-029 ne vise que `onboarding-path-b.spec.ts` (`:65`, `:92`) et `onboarding.spec.ts` (`:57`, `:77`, `:150`) ;
  **`onboarding-motif.spec.ts` (#809) est verte, hors de la KF-029**, et joue déjà l'écran des coordonnées jusqu'au toast du serveur (`onboarding-motif.spec.ts:14-15`, `:57` `await page.fill('#coord-name', 'Exemple Sàrl');`, `:60` `await page.fill('#coord-country', 'Suisse');`) : un cas neuf s'y ajoute (AC8). Les tests Rust (AC6, AC7) prouvent la frontière HTTP, Vitest l'écran isolé.
- **D10 — #1233 : un texte tapé dans le champ client compte comme une action de l'utilisateur** (V1-3, décision de l'orchestrateur du 2026-11-18). `ContactPicker` jette le brouillon quand `selected` change (`reinitialiser()`, `draft = null`) : une réponse tardive réussie effacerait le texte tapé et ferait tomber `contactUnresolved`.
  `onContactDraftChange(true)` fait donc `contactGen++` : la réponse tardive est jetée, et le texte tapé et non choisi se refuse visiblement (« Choisissez le client dans la liste », `:561`) ; un champ vidé se refuse par « Veuillez sélectionner un contact » (`:562`). Le défaut est dans le périmètre de #1233 (un bug trouvé dans la story se corrige dans la story). **Option écartée** : une issue à part. **Limite assumée** (V2-8, comportement du `ContactPicker` de la v0190-correction, sans test) : si l'utilisateur tape une lettre pendant le chargement puis l'efface, la réponse initiale est déjà jetée ; le champ reste vide, l'enregistrement dit « Veuillez sélectionner un contact » (`:562`) et il faut rechoisir le client — rien n'est écrit de faux.

## Critères d'acceptation

1. **AC1 — #1233, le client choisi n'est plus écrasé** (`InvoiceForm.test.ts`, section neuve « Story 48-13 », `getContact` mocké par une promesse résolue à la main) : facture en modification ouverte ; **avant** la réponse de `getContact`, l'utilisateur choisit un autre client **sans rien taper** (focus du champ puis clic dans la liste, ou appel direct du rappel `onSelect` : c'est ce cas, sans frappe, qui fait rougir M3 — une frappe jetterait déjà la réponse, D10)
   la réponse arrive : `selectedContact` reste le client choisi (champ affiché, et `updateInvoice` reçoit `contactId` du client choisi) ; **idem si la réponse tardive échoue** : aucun `errorMsg` « Impossible de charger le contact » ; **sans choix**, la réponse s'applique comme aujourd'hui.
2. **AC2 — le rechargement sur conflit (filet de non-régression, sans preuve du rouge : D2)** : une réponse tardive du chargement **initial** arrivant **après** « Recharger » n'écrase pas le client rechargé du serveur (test Vitest, même section) ; le rechargement lui-même remplace toujours le client comme aujourd'hui (le test existant « AC9-bis », `InvoiceForm.test.ts:462`, reste vert).
3. **AC3 — #1233, E2E** (`frontend/tests/e2e/invoice-contact-tardif.spec.ts`, ou une section de `invoices.spec.ts`) : un brouillon est ouvert en modification, `GET /api/v1/contacts/{id}` du client d'origine est **retardé** par `page.route` ; l'utilisateur choisit un autre client ; la réponse est libérée ;
   le champ client montre le client choisi ; l'enregistrement part avec lui ; **après rechargement dur**, le brouillon porte le client choisi. Sans le correctif, le test rougit (mutation M1).
4. **AC4 — #1243, les champs ne tronquent plus** (`settings-page.test.ts`) : **aucun** des cinq champs de l'adresse, ni le téléphone ni le site web n'a d'attribut `maxlength` (patron `:1327-1328` : **c'est cette assertion qui porte le rouge** — dans jsdom un `maxlength` ne contraint pas une valeur posée par programme : « une rue de 71 caractères part entière » passe aussi **avec** l'attribut, V1-6, et est écrite comme filet, sans preuve du rouge) ; Le `maxlength={320}` de l'e-mail **reste** (assertion de présence : 48-15).
   la réponse `400` du serveur (« adresse : le champ rue dépasse 70 caractères (71) ») s'affiche sous le formulaire, **la saisie reste** et rien n'est enregistré ; un pays « Suisse » part entier et revient refusé ; raccourcie, la rue s'enregistre.
5. **AC5 — #1243, E2E** (`frontend/tests/e2e/company-address.spec.ts`, patron `:113`) : un texte **collé** de 71 caractères dans la rue (`page.fill`, que le `maxlength` tronquait) est refusé avec le message du serveur qui nomme « rue » et « 71 » ; la valeur enregistrée de la société est **inchangée** (relue par l'API) ;
   l'adresse de la société partagée est restaurée en `finally` (comme `restoreAddress`).
6. **AC6 — #1226, le nom de l'assistant** (`crates/kesh-api/tests/onboarding_path_b_e2e.rs`, trois ou quatre `#[sqlx::test]` neufs sur le patron de `coordinates_rejects_country_name`, `:225`) : `POST /api/v1/onboarding/coordinates` rend `400 VALIDATION_ERROR` et **n'écrit rien** (état toujours à l'étape 5,
   société inchangée) pour : un nom de **71 caractères** (message : `70` et `71`) ; un nom portant un caractère hors du jeu (« 张 » ; le message le nomme avec son point de code) ; un nom fait du seul U+00AD (« obligatoire ») ; **un prénom + nom dont la composition dépasse 70 caractères** (nom composé jugé) ; et **accepte** un nom de 70 caractères, un nom avec « & », « é » et « ’ » (U+2019),
   et un nom entouré d'espaces **rogné** à l'enregistrement.
7. **AC7 — #1226, l'adresse de l'assistant** (même fichier) : une rue ou une localité portant un caractère hors du jeu est refusée avec le champ et le caractère nommés (« adresse : le champ localité contient un caractère que la QR-facture n'admet pas : … »), rien d'écrit ; les tests existants
   `coordinates_rejects_country_name`, `coordinates_rejects_unknown_iso_country_817`, `coordinates_validates_ide`, `onboarding_audit_e2e` et `fiscal_years_e2e.rs:998` restent verts **sans modification de leurs données**.
8. **AC8 — l'assistant à l'écran** (V1-1) : (a) Vitest (`onboarding-screens.test.ts` ou voisin) : sur `400`, le message du serveur s'affiche (toast), l'écran reste à l'étape des coordonnées, saisie conservée, aucune borne copiée dans `onboarding/+page.svelte` (grep : aucun `70` ni `maxlength` neufs) ; (b) **E2E dans `onboarding-motif.spec.ts`** (verte, hors KF-029) : cas « nom de 71 caractères » — le toast nomme `70` et `71`, état toujours à l'étape 5 ; puis saisie corrigée (70 caractères) qui passe. Preuve du rouge : M6 rejouée en E2E.
9. **AC9 — factures réelles : rien ne change pour une facture existante** (contrainte de Guy du 2026-11-18). Prouvé par :
   a. **ouverture d'un brouillon en modification, sans aucune interaction** : le client d'origine s'affiche, les lignes, les totaux, la TVA, les conditions et l'échéance sont **identiques** à la valeur chargée, et l'enregistrement sans toucher envoie le même corps qu'avant (test Vitest d'aller-retour, patron des tests « aller-retour » `InvoiceForm.test.ts:696-722`) ;
   b. **affichage d'une facture validée** : `git diff origin/epic/48-corrections-0-20-0..HEAD --stat -- 'frontend/src/routes/(app)/invoices/[id]/+page.svelte' crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report` **vide** (sortie copiée au Dev Agent Record) ;
   c. **PDF à l'identique** : un test d'intégration existant du PDF (`invoice_pdf_e2e`, nommé au développement) passe **sans modification**, et le test **avant/après** de la release (`scripts/recette-montee.sh`) est à la charge de l'orchestrateur ; la story n'écrit rien dans `invoices`, `invoice_lines`, `companies` hors des gestes décrits ;
   d. **société déjà configurée** : `POST /api/v1/onboarding/coordinates` après la fin de l'assistant reste refusé comme avant (`lock_state_at_step(…, 5, …)`, `onboarding.rs:559`) ; un test Rust le pose (l'étape 5 passée, le même POST valide rend le refus d'étape existant, société inchangée).
10. **AC10 — compteurs, gardes, manuel** : `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) **recompté depuis la source** (aucune clé neuve attendue ; s'il bouge, le dire) ; `npm run lint-i18n-ownership` vert ; `audit_route_registry` et `audit_acteur_garde` verts (aucune route ajoutée) ;
    `cargo test -p kesh-api --test textes_coherents` vert ; manuel (D7) : `.tex` seulement ; `git grep -n -E "70 caract" -- docs/manual/fr/user-manual.tex` cité avant/après.
11. **AC11 — recette visuelle** (thème clair, base de démonstration, captures jointes au Dev Agent Record et au prompt de la revue, `kesh-gate-logs/e4813/captures/`) : (1) *Paramètres → Organisation* : formulaire d'adresse avec un texte collé de 71 caractères **non coupé** ; (2) le refus du serveur sous le formulaire, saisie conservée ;
    (3) le formulaire du téléphone avec un texte trop long refusé ; (4) l'écran d'édition d'un brouillon **après** choix d'un client pendant que la réponse initiale est retardée : le client choisi est affiché ; (5) l'assistant, étape des coordonnées, nom de 71 caractères : le toast du serveur.

12. **AC12 — aucune donnée existante n'est refusée à l'enregistrement** (V2-5) : la story n'ajoute **aucune borne serveur sur une route que traverse une donnée existante** (les trois refus neufs de D5 — longueur et jeu du nom, jeu de l'adresse — ne s'appliquent qu'à l'**étape 5 de l'assistant**, sur des données saisies à ce moment) ; les refus de l'adresse (`check_lengths`), du téléphone et du site (`normalize_contact_field`) existent déjà, et l'assistant n'agit qu'à l'étape 5 (`onboarding.rs:559`). Ce qui change à l'écran : le message du serveur s'affiche (`addressError`) au lieu d'une coupure, la saisie reste. Une société déjà enregistrée garde son adresse, son téléphone et son site ; l'ouverture du formulaire les préremplit **en entier**.

13. **AC13 — D10, texte tapé dans le champ client** (`InvoiceForm.test.ts`) : facture en modification, réponse initiale retardée ; l'utilisateur **tape** un texte (sans choisir) ; la réponse arrive : le texte tapé est **conservé**, `selectedContact` reste `null`, et l'enregistrement est refusé (« Choisissez le client dans la liste » (`InvoiceForm.svelte:561`, `contactUnresolved`)). Rouge sans `contactGen++` de `onContactDraftChange(true)` (**M8**).

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; le journal attendu est dans `/home/gcorbaz/devel/kesh-gate-logs/e4813/`. Ordre : **le test d'abord, rouge** (journal `rouge-<n>.log`, sortie copiée), puis le correctif, vert.

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC1 (choix avant la réponse ; réponse en échec) | la garde `myGen` : `selectedContact` revient au client d'origine, ou `errorMsg` s'affiche | **M1** : retirer `if (myGen !== contactGen) return;` du `.then` ; **M2** : du `.catch` ; **M3** : retirer `contactGen++` de `onContactSelect` (V2-6) ; journaux `mut-m1.log`, `mut-m2.log`, `mut-m3.log` |
| AC2 (réponse initiale après « Recharger ») | **ne rougit pas** : `contactGen++` de `reloadFromServer` est défensif (D2 : scénario inatteignable par l'interface, `:562`, `:722`) | aucune mutation ; filet de non-régression, déclaré tel |
| AC3 (E2E client tardif) | la garde | M1 rejouée en E2E ; `e2e-m1.log` |
| AC4 (aucun `maxlength` sur les sept champs) | les attributs `maxlength` | **M4** (mutation par l'outil Edit : remettre un `maxlength` sur un champ) ; `mut-m4.log` — **pas de `git stash`**. L'assertion « 71 caractères partent entiers » ne rougit pas (jsdom) : filet déclaré |
| AC5 (E2E texte collé) | idem | **M5** (mutation Edit : `maxlength` remis sur la rue) rejouée en E2E ; `e2e-m5.log` |
| AC6, AC7 (Rust) | `validate_company_name_for_qr` et `validate_required_for_qr` dans `set_coordinates` | **M6** : remettre `if name.is_empty()` ; **M7** : remettre `validate_required()` ; `mut-m6.log`, `mut-m7.log` |
| AC8 b (E2E `onboarding-motif.spec.ts`) | idem | M6 rejouée en E2E ; `e2e-m6.log` |
| AC13 (texte tapé) | `contactGen++` de `onContactDraftChange(true)` | **M8** : retirer cette ligne ; `mut-m8.log` |
| AC6 (« accepte » : 70 caractères, `&`, `é`, `’`, rognage) | **ne rougissent pas** (filets : le nom accepté l'était déjà), à dire dans le journal | — |
| AC9 a, d ; AC12 | **ne rougissent pas** (non-régression par construction), à dire | — |

*Numérotation des mutations : **M1 à M8, uniques, un journal chacune**.*

Après une mutation restaurée : `touch` du fichier (cargo garde sinon le binaire muté). Tout test qui ne rougit pas sans le correctif est **déclaré tel**, dans le Dev Agent Record, avec la raison.

## Tâches

1. **Étape 0** : relire les lignes citées à la tête de la branche (celles de ce document ; copier trois au Change Log) ; relire `ContactPicker.svelte` (`reinitialiser()`) pour écrire le test AC13.
2. `InvoiceForm.svelte` : `contactGen`, garde du `.then` et du `.catch`, `contactGen++` dans `onContactSelect`, `onContactDraftChange(true)` (D10) et `reloadFromServer` (D2, défensif) ; tests AC1, AC2, AC9 a, AC13 ; E2E AC3.
3. `settings/+page.svelte` : retrait des **sept** `maxlength` (cinq de l'adresse, téléphone, site web ; l'e-mail reste, D4) ; tests AC4 ; E2E AC5.
4. `onboarding.rs` : `validate_company_name_for_qr` et `validate_required_for_qr` (D5) ; tests AC6, AC7, AC9 d ; test d'écran AC8 a ; **cas E2E dans `onboarding-motif.spec.ts`** (AC8 b).
5. Manuel (D7), `.tex` seulement ; recomptes (AC10) ; captures (AC11).
6. Rédiger au Dev Agent Record les mutations, les gates, les captures et le grep de l'AC9 b. **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.**
## Gates (`e35-dev-commun.md` § *Quels gates, quand*)

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(onboarding_path_b_e2e) | binary(onboarding_audit_e2e) | binary(fiscal_years_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e)'` ;
  `npm run check` ; Vitest de `InvoiceForm`, `settings`, `onboarding`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** (V1-8, aligné sur `e35-dev-commun.md` § *Quels gates, quand* : « Pas d'E2E ») : `scripts/test-fast.sh --ci` et `scripts/gate-ci.sh` (voie 1, sa clé de base), frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`). **Pas d'E2E complet** à ce stade ; les E2E neufs de la story (AC3, AC5, AC8 b) se jouent seuls pendant le développement.
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (montage de `docs/testing.md` § *Prérequis Playwright local*, `KESH_COOKIE_SECURE=false` ; « vert » = les seuls échecs attendus (sept de la KF-029 plus zéro à deux de pollution), jugés fichier par fichier). **Une commande lourde à la fois**, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant chaque commande lourde, verrou `.gate.lock`. Bases : celles de la voie donnée par le prompt d'appel.

## Note de version

**Corrigé**

- Facture : ouvrir un brouillon puis choisir un autre client avant que le client d'origine ne soit chargé ne remplace plus le client choisi (refs #1233).
- *Paramètres → Organisation* : l'adresse de la société, son téléphone et son site web ne sont plus coupés en silence quand on colle un texte trop long ; Kesh refuse et garde la saisie (l'adresse dit le champ et le nombre de caractères saisis, le téléphone et le site web disent le maximum) (refs #1243).
- Facture : un texte tapé dans le champ client avant la fin du chargement du client d'origine n'est plus effacé par ce chargement (refs #1233).
- Assistant de démarrage : le nom de la société (ou « prénom nom ») et son adresse sont contrôlés comme la QR-facture les contrôlera — non vide, 70 caractères au plus pour le nom, jeu de caractères de la QR-facture — au lieu de s'enregistrer puis de faire refuser la première QR-facture (refs #1226).

*Pour le manuel d'administration, section « Passer à la 0.20.0 » (posée à l'intégration)* : `POST /api/v1/onboarding/coordinates` refuse désormais, en `400 VALIDATION_ERROR`, un nom de plus de 70 caractères ou portant un caractère que la QR-facture n'admet pas, et un caractère
refusé dans l'adresse ; aucune société déjà configurée n'est modifiée, aucune facture ni aucun PDF existant ne change.

## Dev Notes

- **Modules touchés : quatre** — **règle de décompte** (V2-2, demandée par la validation P2) : un module est une **crate** backend, un **domaine** frontend (`components/<domaine>`, `features/<domaine>`, `routes/(app)/<domaine>`) ou **les manuels** ; les tests et les catalogues suivent leur module. Ici : (1) `kesh-api` (`routes/onboarding.rs` ; `companies.rs` n'est lu que pour `validate_company_name_for_qr`, non modifié), (2) frontend `components/invoices` (`InvoiceForm.svelte`), (3) frontend `routes/(app)/settings`, (4) manuels (`.tex`). **Quatre ≤ cinq : pas de découpage de plus.** Les tests d'écran de l'AC8 (Vitest `routes/onboarding/onboarding-screens.test.ts` et cas E2E de `onboarding-motif.spec.ts`) portent sur un écran **non modifié** : **ils suivent `kesh-api`, le module corrigé** (sinon l'écran de l'assistant compterait pour un cinquième module : cinq, toujours dans la limite).
  La story était à neuf modules à ce grain après l'élargissement : le **signal de splitting a été déclaré au Project Lead par l'orchestrateur** et la story coupée (48-15).
- **Issues** : #1233, #1243, #1226. **Aucune issue à ouvrir par cette fiche** ; les sites au même symptôme sont listés dans la 48-15, avec leurs issues (#1335 à #1340, **ouvertes par l'orchestrateur, une par écran**). Le jeu de caractères de l'adresse de l'assistant est corrigé par D5 et consigné au Change Log, sans issue (V1-11).
- **Dépendance** : la 48-15 part de la 48-13 fusionnée (`onboarding.rs`, `settings/+page.svelte`). La 48-13 ne dépend d'aucune autre story de l'Epic 48.
- **Fichiers communs** (plan de la release et fichier d'épique 48) : `InvoiceForm.svelte` avec l'Epic 49 (49-1, 49-2, 49-3, éditeur de lignes) — **la 48-13 se fusionne d'abord** ; `settings/+page.svelte` avec la v0200-menu-sous-groupes (blocs distincts) ;
  `i18n-keys.test.ts` (`sitesTotal`), catalogues i18n, `sprint-status.yaml` : recomptés / résolus en gardant les deux côtés. Pas de PDF de manuel commité. **Contributeur Beat** (PR #1327, `kesh-report/src/pdf.rs`, `kesh-qrbill` `largeurs.rs` et `lib.rs`) : la story ne touche aucun de ces fichiers.
- **Ce qui doit rester vrai** : un brouillon s'ouvre avec son client d'origine quand l'utilisateur ne fait rien ; le rechargement sur conflit remplace le client par celui du serveur ; l'adresse de la société se refuse côté serveur avec le champ et le nombre ; l'assistant refuse *avant* le verrou d'étape ;
  un nom de 70 caractères à `é`, `&`, `’` est accepté.
- **Aucune donnée réelle** dans les tests, captures ni journaux : « Démo SA », adresses factices.
- Références : `v0190-nom-societe.md` (D2, D5, C-v0190c-25), `v0190-correction.md` (`ContactPicker`, #1162), `e0200-commun.md`, plan `release-0.20.0.md` et `epic-48-corrections-0-20-0.md` (branche `chore/kickoff-0.20.0`).

## Dev Agent Record

*Développement du 2026-11-18 (Opus 5.5, `bmad-dev-story`), worktree `kesh-48-13`, clé `e4813`, port 3413. Journaux sous
`/home/gcorbaz/devel/kesh-gate-logs/e4813/`. Registre : `e0200-choix-autonomes-e4813.md` (C-e4813-1 à 6).*

### Commits

- `b41df52da` — code, tests, E2E, manuel (`.tex`) : **dernier commit de code**.
- Commit suivant — fiche (ce Dev Agent Record, Change Log, statut `review`), registre, prompt de la revue P1.

### Fichiers touchés (`git diff --stat 78d8c4c55 b41df52da`, 11 fichiers)

`crates/kesh-api/src/routes/onboarding.rs`, `crates/kesh-api/tests/onboarding_path_b_e2e.rs`,
`frontend/src/lib/components/invoices/InvoiceForm.svelte`, `InvoiceForm.test.ts`, `frontend/src/routes/(app)/settings/+page.svelte`,
`settings-page.test.ts`, `frontend/src/routes/onboarding/onboarding-motif.test.ts`, `frontend/tests/e2e/invoice-contact-tardif.spec.ts`
(neuf), `company-address.spec.ts`, `onboarding-motif.spec.ts`, `docs/manual/fr/user-manual.tex`. Modules : quatre (`kesh-api`,
`components/invoices`, `routes/(app)/settings`, manuels), comme la fiche. **Aucune migration**, aucune clé i18n, aucune route.

### Tests ajoutés (recomptés depuis le diff `78d8c4c55..b41df52da`)

- Rust : **4** `#[sqlx::test]` (`onboarding_path_b_e2e.rs`) — refus du nom (4 cas en boucle), nom de 70 caractères accepté et rogné,
  refus de l'adresse (2 cas), société configurée inchangée (AC9 d).
- Vitest : **12** `it(` — `InvoiceForm.test.ts` 7 (AC1 ×4, AC2, AC13, AC9 a), `settings-page.test.ts` 4 (AC4), `onboarding-motif.test.ts` 1 (AC8 a).
- E2E : **3** `test(` — `invoice-contact-tardif.spec.ts` (AC3), `company-address.spec.ts` (AC5), `onboarding-motif.spec.ts` (AC8 b).

### Les rouges (avant correctif, code des tests seul)

- `rouge-rust.log` : 2 des 4 tests neufs rougissent — `coordinates_rejects_a_name_the_qr_bill_refuses_1226` (« 71 caractères : left 200,
  right 400 » : le nom de 71 caractères était **accepté**, étape passée à 6) et `coordinates_rejects_an_address_character_…_1226` ;
  les deux autres passent, **filets déclarés** (le nom accepté l'était déjà ; AC9 d est le refus d'étape existant).
- `rouge-vitest.log` : 5 rouges — AC1 (choix sans frappe : « expected 'Client' to be 'Autre client' »), AC1 échec tardif (bandeau
  présent), AC2 (« expected 'Client' to be 'Client du serveur' »), AC13 (« expected 'Client' to be 'Mar' »), AC4 (« street: expected
  true to be false »). Verts avant correctif, **filets déclarés** : AC1 « sans geste » ×2 (comportement d'avant, gardé), AC9 a
  (non-régression), AC4 « part entier » ×3 (jsdom ne contraint pas une valeur posée par programme, V1-6).
- **AC2** rougit avant correctif, mais par D1 (le choix de l'utilisateur n'avançait rien), non par D2 : sous M3 (D1 retiré de
  `onContactSelect`), AC2 **reste vert** grâce au `contactGen++` de `reloadFromServer` (`mut-m3.log`). D2 n'est donc pas prouvé seul :
  conforme à la fiche (D2 défensif, pas de mutation propre).

### Mutations (`mut/muter.sh`, code commité avant, restauration `git restore --worktree` + `touch`, motif vérifié unique)

| Mutation | Geste | Tuée par | Journal |
|---|---|---|---|
| M1 | garde du `.then` retirée | AC1 (choix), AC2, AC13 | `mut-m1.log` |
| M2 | garde du `.catch` retirée | AC1 (échec tardif) | `mut-m2.log` |
| M3 | `contactGen++` de `onContactSelect` retiré | AC1 ×2 (AC2 reste vert : voir ci-dessus) | `mut-m3.log` |
| M4 | `maxlength={70}` remis sur la rue | AC4 (absence de l'attribut) | `mut-m4.log` |
| M6 | `if name.is_empty()` remis | `coordinates_rejects_a_name_…_1226` | `mut-m6.log` |
| M7 | `validate_required()` remis | `coordinates_rejects_an_address_…_1226` | `mut-m7.log` |
| M8 | `contactGen++` de `onContactDraftChange(true)` retiré | AC13 | `mut-m8.log` |
| M1 (E2E) | idem M1, frontend reconstruit | AC3, ligne 93 (le client d'origine revient) | `e2e-m1.log` |
| M5 (E2E) | `maxlength={70}` sur la rue, frontend reconstruit | AC5, ligne 144 (le champ ne garde que 70 caractères après le collage ; revue P1, B4) | `e2e-m5.log` |
| M6 (E2E) | idem M6, backend reconstruit | AC8 b, ligne 83 (pas de toast « (71) ») | `e2e-m6.log` |

**Écart constaté et corrigé** : la première série M1, M2, M3, M8 a rendu « TUÉE » à tort — vitest, lancé avec `--root frontend`
depuis la racine, ne trouvait aucun fichier (« No test files found », code 1). Rejouée depuis `frontend/` ; les journaux
`mut-m{1,2,3,8}.log` sont ceux du rejeu (chaque rouge nommé ci-dessus). Après les mutations E2E, frontend et backend reconstruits
propres (`build-front-dev2.log`, `build-api-dev2.log`).

### E2E (les trois specs touchées, seules ; pas d'E2E complet au passage en `review`)

`e2e-neufs.log`, commit `b41df52da`, base `kesh_e2e_e4813` recréée, backend sur 3413 : **8 passed** (`company-address.spec.ts` 5,
`invoice-contact-tardif.spec.ts` 1, `onboarding-motif.spec.ts` 2). L'E2E complet et le `gate-ci` de clôture tournent à la clôture.

### Recette visuelle (AC11)

`/home/gcorbaz/devel/kesh-gate-logs/e4813/captures/` (thème clair, base de test jetable, données factices ; `captures-4813.sh` et
`.mjs`, `captures.log`, commit `b41df52da`) : `01-adresse-rue-71-non-coupee.png`, `02-adresse-refus-serveur-saisie-gardee.png`
(« adresse : le champ rue dépasse 70 caractères (71) »), `03-telephone-trop-long-refuse.png` (« Le numéro de téléphone de la société
est trop long (50 caractères au plus). »), `04-brouillon-client-choisi-garde-apres-reponse-tardive.png`,
`05-assistant-nom-71-refuse-toast.png` (« Le nom de la société dépasse 70 caractères (71) : la QR-facture n'en admet pas davantage. »).

### Preuves des AC sans test propre

- **AC9 b** : `git diff origin/epic/48-corrections-0-20-0..HEAD --stat -- 'frontend/src/routes/(app)/invoices/[id]/+page.svelte' crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report` → **sortie vide**.
- **AC9 c** : `invoice_pdf_e2e` passe sans modification (gate ciblé `vert-rust-cible.log` et `test-fast-ci-review.log`) ; la recette avant/après
  (`scripts/recette-montee.sh`) est à l'orchestrateur.
- **AC8 a** : `git diff … --stat -- frontend/src/routes/onboarding/+page.svelte` → vide : aucune borne copiée dans l'écran.
- **AC10** : aucune clé i18n (`git diff … --stat -- crates/kesh-i18n frontend/src/lib/shared/i18n-keys.test.ts` vide) : `sitesTotal`
  inchangé ; `git grep -c -E "70 caract" -- docs/manual/fr/user-manual.tex` : `1` à `65fd4efcd`, `3` à `b41df52da`. PDF non régénéré
  (à la release) : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé ensuite par la QR-facture se corrige ainsi'` → `1` (avant ; `0` attendu après régénération) ; `'jamais coupé'` → `0` (avant ; `1` attendu après). *Annoté le 2026-11-18 (revue P1, A5) :
  la phrase de `:1522` revient au présent (`448186f11`) ; la première phrase de contrôle rend donc encore `1` après régénération. Contrôle de
  la release : `'avant la version 0.20.0 et refusé ensuite'` → `1` (le `~` de LaTeX devient une espace).*
- Mots-clés de fermeture : `git log origin/epic/48-corrections-0-20-0..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` → vide.

### Gates

*Une commande lourde à la fois, `CARGO_BUILD_JOBS=2`, verrous de voie. **Le gate ciblé a tourné sur l'arbre de travail, avant le
commit** (ses journaux, de 21 h 16 à 21 h 22, précèdent `b41df52da`, daté de 21 h 22 min 56, et ne nomment aucun commit) ; **les gates
de review ont tourné au commit `b41df52da`** (journaux postérieurs, arbre propre). Correction de la revue P1 (A1) : la ligne disait
« Tous au commit `b41df52da` ».*

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` vert (`fmt.log`) ; `cargo clippy --workspace --all-targets -- -D warnings`
  vert (`clippy.log`) ; `cargo nextest run -p kesh-api -E 'binary(onboarding_path_b_e2e) | binary(onboarding_audit_e2e) | binary(fiscal_years_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e) | binary(company_address_e2e)'`
  : **211 passed** (`vert-rust-cible.log`) ; Vitest des trois fichiers touchés : **186 passed** (`vert-vitest-cible.log`).
- **Passage en `review`** — base `kesh_e4813` remise à neuf (migrations du worktree, `seed-dev-db.sql`) :
  - `scripts/test-fast.sh --ci` : **6137 tests run, 6137 passed, 14 skipped** (`test-fast-ci-review.log`) ;
  - `scripts/gate-ci.sh kesh_ci_e4813` : **6154 passés, 0 échoués, 18 ignorés**, code 0 (`gate-ci-review.log`) ;
  - frontend complet : `npm run check` 0 erreur (29 avertissements, aucun dans un fichier touché ; `front-check-review.log`),
    `lint-i18n-ownership` PASS, `test:unit` **278 fichiers, 4624 tests passed**, `build` vert (`front-*-review.log`).
- **Non lancés** (à la clôture) : E2E complet et `gate-ci` de clôture. Seules les trois specs touchées ont tourné (`e2e-neufs.log`, 8 passed).

### Remédiation de la revue P1 (commit `448186f11`)

- **Gate ciblé, au commit `448186f11`** (arbre propre ; chaque journal porte le commit en tête) : `cargo fmt --all -- --check` vert
  (`fmt-p1.log`, lancé sur l'arbre juste avant le commit, sans autre changement) ; `clippy -D warnings` vert (`clippy-p1.log`) ;
  `cargo nextest run -p kesh-api -E 'binary(kesh_api) | binary(onboarding_path_b_e2e) | binary(onboarding_audit_e2e) | binary(fiscal_years_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e) | binary(company_address_e2e)'`
  : **934 passed, 5 skipped** (`cible-rust-p1.log` ; `binary(kesh_api)` porte les tests unitaires d'`address_input.rs`) ; Vitest
  `InvoiceForm.test.ts` et `settings-page.test.ts` : **158 passed** (`cible-vitest-p1.log`) ; `npm run check` 0 erreur (`check-p1.log`).
- **Non lancé** : la spec `invoice-contact-tardif.spec.ts` modifiée (E8, une ligne d'attente) ; elle tourne à l'E2E complet de clôture.
- **Tests** : les quatre refus du nom deviennent quatre `#[sqlx::test]` sur l'aide `assert_name_refused` (le test en boucle
  `coordinates_rejects_a_name_the_qr_bill_refuses_1226` disparaît) : **7** tests `1226` au lieu de 4. Vitest : le cas du téléphone devient
  deux cas (téléphone, site web), **13** `it` ajoutés par la story au lieu de 12.
- **Mutations rejouées sur les tests séparés (E7)** : **M6** (`mut-m6-p1.log`) tue les **quatre** tests de refus du nom, chacun pour sa
  raison ; **M6-bis** (le nom juge `body.name` au lieu du nom composé, `mut-m6bis.log`) tue **seulement**
  `coordinates_rejects_a_composed_name_over_70_1226` (« « 70 » absent de « Le nom de la société est obligatoire. » ») : le point propre
  de D5, le nom composé jugé, est prouvé rouge.

## Change Log

- **2026-11-18 — création de la fiche** (Sonnet 5.5, `bmad-create-story`). Lu à `773df80da` : `InvoiceForm.svelte:399-416` (chargement initial du client, sans garde), `settings/+page.svelte:821-841` (sept `maxlength` dont cinq d'adresse),
  `onboarding.rs:536`, `:542` (contrôles de l'assistant), `companies.rs:504` (`validate_company_name_for_qr`), `address_input.rs:58-80`, `:123`. Périmètre étendu par rapport au fichier d'épique : le téléphone et le site web (même défaut, serveur qui refuse déjà) et le jeu de caractères de l'adresse de l'assistant (D5) ;
  e-mail, contacts et fournisseur hors périmètre, **quatre issues à ouvrir** (Dev Notes). Quatre modules, pas de découpage, aucune migration.
- **2026-11-18 — validation, passe 1** (Opus, contexte frais ; `kesh-gate-logs/e4813/validation-p1.md`) : 0 CRITICAL / 0 HIGH / **4 MEDIUM** / **8 LOW**, **tous remédiés dans cette révision** (Sonnet 5.5).
  - V1-1 (MEDIUM) : E2E de #1226 admis dans `onboarding-motif.spec.ts` (hors KF-029) — D8, AC8 b. V1-2 (MEDIUM) : `contactGen++` du rechargement déclaré défensif et non rougissant, M3 retirée — D2, AC2. V1-3 (MEDIUM) : le texte tapé compte comme une action (`contactGen++` dans `onContactDraftChange(true)`), test AC14, M9 — D10.
    V1-4 (MEDIUM) : trois sites reclassés (nom de l'article, numéro de client, conditions de paiement), **dans la story** avec le périmètre élargi — D9, AC13.
  - V1-5 : `ProductPicker` gardé (`seq`), D3. V1-6 : assertion jsdom déclarée filet, preuve par mutation Edit au lieu de `git stash`. V1-7 : mutations renumérotées M1 à M10. V1-8 : pas d'E2E complet au passage en `review`. V1-9 : manuel `:1522` réécrit, phrase de l'assistant à `:219`. V1-10 : Note de version et D4 nuancés (téléphone et site web : maximum seulement).
    V1-11 : pas d'issue, défaut consigné ici. V1-12 : repris **dans la story** (D11, AC15, #1334, décision de Guy du 2026-11-18), non en issue.
  - **Périmètre élargi** (décision de Guy du 2026-11-18, sur l'avis de l'orchestrateur) : #1331, #1332, #1333, plus V1-4 ; **modules : quatre au grain de la crate et de l'application** (`kesh-api`, `kesh-i18n`, frontend, manuels) ; pas de découpage.
- **2026-11-18 — validation, passe 2** (Sonnet 5.5 ; `kesh-gate-logs/e4813/validation-p2.md`) : 0 / 0 / **2 MEDIUM** / **7 LOW**, remédiés. **V2-2 (MEDIUM) : la story est coupée en deux** (neuf modules au grain du domaine après l'élargissement ; règle de splitting préventif, signal déclaré au Project Lead par l'orchestrateur) : #1331, #1332, #1333, #1334 et les trois champs du PDF partent dans la **48-15** ; la 48-13 revient à #1233, #1243, #1226 (quatre modules, règle de décompte écrite). **V2-1** (#1334) : traité dans la 48-15 (les deux sites, contacts et assistant). V2-3, V2-4, V2-5, V2-6 (M1 à M8 uniques, mutation de `onContactSelect` nommée), V2-7 (D4 : messages localisés du téléphone et du site), V2-8 (D10 : limite assumée), V2-9 (« issue à ouvrir » résiduels retirés) : remédiés ici ou dans la 48-15.
- **2026-11-18 — validation, passe 3** (Opus 5.5 ; `kesh-gate-logs/e4813/validation-p3.md`) : 0 / 0 / **1 MEDIUM** / **8 LOW**, remédiés. **V3-1 (MEDIUM)** : la découpe avait perdu l'AC du texte tapé (D10, M8) que le tableau des tests, les tâches et la Note de version citaient encore : **rétabli en AC13** ; **signal « recycle » constaté** (V3-1 naît du correctif de la P2, à sévérité égale ; V3-2 reprend V1-7 et V2-6 sous une troisième forme), **déclaré au Project Lead par l'orchestrateur**. V3-2 : AC8 b cite M6. V3-3 : le test de l'AC1 choisit sans rien taper (M3). V3-4 : D7 recomposé. V3-5 : 40 lignes dont 3 commentaires = 37 sites, `onboarding/+page.svelte:814` ajouté à #1005. V3-6 : renvois aux issues #1335 à #1340 et #1005 ; le délai `contacts:1086` va à #1336 ; la raison sociale n'est plus citée ici (elle reste à la 48-15, D4). V3-7 : règle de décompte des tests d'écran. V3-8 : AC12 reformulé. V3-9 : à l'orchestrateur (fichier d'épique). *Les renvois « AC n », « M n » et « D n » (D9, D11, partis à la 48-15) des entrées P1 et P2 ci-dessus suivent la numérotation d'alors ; ceux du corps de la fiche ont été relus et sont à jour (AC1 à AC13, M1 à M8).*
- **2026-11-18 — validation terminée** (tendance des findings au-dessus de LOW : **P1 Opus 0/0/4/8 → P2 Sonnet 0/0/2/7 → découpe de la story (48-15) → P3 Opus 0/0/1/8 → P4 ciblée Haiku 0/0/0/4**). Modèles : Opus, Sonnet, Opus, Haiku (passe ciblée sur le seul commit `25e7b63ab`). Signal « recycle » constaté à la P3 (V3-1 né de la découpe, à sévérité égale ; V3-2 troisième forme de la numérotation des mutations), **déclaré au Project Lead par l'orchestrateur**. Les issues #1335 à #1340 des sites hors périmètre sont **ouvertes par l'orchestrateur, une par écran**. **Les quatre LOW de la P4 (P4-1 à P4-4) sont corrigés après la dernière passe, dans le texte de la fiche seulement** (aucun code, aucun AC modifié : « Choisissez le client dans la liste » précisé à l'AC13 et à D10, `(`onSelect`)` orphelin retiré, annotation des renvois D, #1335 à #1340). Rapport : `kesh-gate-logs/e4813/validation-p4-ciblee.md`. Statut : `ready-for-dev`.
- **2026-11-18 — développement** (Opus 5.5, `bmad-dev-story`). **Étape 0** : les quatre LOW de la P4 étaient déjà traités dans la
  fiche (relus) ; lignes relues à la tête `78d8c4c55`, trois citées : `InvoiceForm.svelte:404` `getContact(invoice.contactId)` (sans
  garde), `settings/+page.svelte:821` `… class={PENDING_FIELD} maxlength={70} data-testid="settings-company-address-street-input" />`,
  `onboarding.rs:542` `let address = body.address.validate_required()?;` ; `ContactPicker.svelte` relu (`reinitialiser()` au changement
  de `selected`, `onDraftChange` par effet sur `unresolved`). Aucune prémisse démentie. **Note de l'orchestrateur** (validation 48-15,
  V4-8) : deux renvois du corps corrigés — « les trois champs du PDF » devient « cinq champs » (en-tête, tableau de #1243) et le délai
  `contacts:1086` n'est plus rattaché à #1336 (il est dans le périmètre de la 48-15) ; les entrées passées du Change Log gardent leurs
  mots. Code au commit `b41df52da` ; rouges, mutations, E2E, captures et gates : Dev Agent Record. Le jeu de caractères de l'adresse à
  l'assistant (défaut trouvé à la fiche, D5) est corrigé ici, sans issue (V1-11). Statut : `review`.
- **2026-11-18 — revue de code, passe 1** (trois lentilles, contexte frais ; `kesh-gate-logs/e4813/review-p1-{B,E,A}.md`) : **aucun
  finding au-dessus de LOW** — B 0/0/0/5, E 0/0/0/8, A 0/0/0/5. Décisions de l'orchestrateur :
  - **corrigés dans la story** (règle « un LOW qui touche la production peut se corriger », suivi d'une passe ciblée), commit de code
    `448186f11` : **A2** (AC9 a vérifie `200.00`, au lieu de comparer le pied du tableau à lui-même), **A3/B3** (téléphone simulé avec le
    vrai message du serveur, et un cas neuf pour le site web), **E7** (les quatre refus du nom en quatre tests indépendants, chacun sur
    sa base, le nom composé d'abord : un rouge ne s'arrête plus au premier cas), **E8** (`await (await reponse).finished()` dans l'E2E de
    l'AC3), **E4** (`validate_required` privée), **E5/B5/A4** (doc-comments d'`address_input.rs` vrais sur les appelants), **E6**
    (commentaire du pays, champ libre), **A5** (manuel `:1522` au présent pour une société configurée avant la version 0.20.0) ; au
    Dev Agent Record, **A1** (le gate ciblé a tourné sur l'arbre de travail, avant le commit) et **B4** (ligne 144 de M5 décrite juste) ;
  - **en issues, hors story** (ouvertes par l'orchestrateur, une par problème) : B1/E1 (prénom ou nom fait du seul U+00AD) → **#1343** ;
    B2 (raison sociale saisie avec prénom et nom ignorée sans un mot) → **#1344** ; E3/B2 (message du nom composé : « nom de la société »,
    nombre qu'aucun champ ne montre) → **#1345** ; E2 (prénom ou nom seul non bornés) → **#1334**, déjà portée par la 48-15.
  Gate ciblé de la remédiation : Dev Agent Record. La passe ciblée sur `448186f11` est lancée par l'orchestrateur.
