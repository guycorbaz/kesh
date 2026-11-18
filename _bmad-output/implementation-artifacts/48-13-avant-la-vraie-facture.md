# Story 48-13 : Avant la vraie facture — client écrasé par un chargement tardif, adresse tronquée en silence, nom de société non contrôlé à l'assistant (#1233, #1243, #1226)

Status: ready-for-dev

*Fiche rédigée le 2026-11-18 (Sonnet 5.5), worktree `kesh-48-13`, branche `story/48-13-avant-la-vraie-facture` partie de
`epic/48-corrections-0-20-0` (= `main` = v0.19.0, `773df80da`), par le workflow `bmad-create-story`. Release **0.20.0** (sortie le
vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Story **prioritaire** : elle porte sur le chemin de la première facture client
QR de fin novembre, et **se fusionne avant** les stories de l'Epic 49 qui touchent `InvoiceForm.svelte` (arbitrage de Guy du 2026-11-18).
**Aucune migration** : aucune colonne ne change (`companies.name` `VARCHAR(255)`, adresse en cinq colonnes, inchangées) ; si le
développement en découvre une, il s'arrête et le dit. `#1234` (`billing_information`) est **retirée** de la story et de la 0.20.0 (décision de
Guy du 2026-11-18) : `kesh-qrbill` n'est pas touché.*

*Révisée le 2026-11-18 après la validation P1 (Opus, 0/0/4/8) et sur décision de Guy du 2026-11-18 (« diminuer le nombre de bugs ») : la story prend aussi
**#1331** (adresse d'un contact client coupée par `maxlength`), **#1332** (pays du fournisseur neuf coupé à deux caractères), **#1333** (e-mail de la société sans borne
serveur) et trois sites relevés par la validation (nom de l'article, numéro de client, conditions de paiement) : même remède, **dans la story**. Titre inchangé.*

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

  (hors réglages : `git grep -n "maxlength" -- frontend/src/lib frontend/src/routes ':!*.test.ts'`, 38 sites, relevés ci-dessous.)

  | Site | Borne | Finit sur le PDF / la QR ? | Le serveur refuse-t-il ? | Verdict |
  |---|---|---|---|---|
  | `settings/+page.svelte:821-841` adresse société (rue 70, n° 16, NPA 16, localité 35, pays 2) | SIX | oui (créancier) | **oui** (`check_lengths`) | **dans le périmètre (#1243)** |
  | `settings/+page.svelte:964` téléphone (50) | colonne `VARCHAR(50)` | oui (`invoice_pdf_service.rs:426` `creditor_phone: company.phone.clone(),`) | **oui** (`companies.rs:214` `MAX_PHONE_LEN`, `normalize_contact_field` `:251`, clé `error-company-phone-too-long`) | **dans le périmètre** (même défaut, même bloc ; D4) |
  | `settings/+page.svelte:974` site web (255) | colonne `VARCHAR(255)` | oui (`:428` `creditor_website`) | **oui** (`MAX_WEBSITE_LEN`, `error-company-website-too-long`) | **dans le périmètre** (D4) |
  | `settings/+page.svelte:906` e-mail (320) | colonne | oui (`:427` `creditor_email`) | **non** : `update_company_email` ne contrôle que la forme (`contacts.rs:266` `is_valid_email_simple`, aucune longueur) | **dans le périmètre (#1333)** : borne serveur **ajoutée** (D9), puis `maxlength` retiré |
  | `contacts/+page.svelte:992-1008` adresse du client (rue 70, n° 16, NPA 16, localité 35, pays 2) | SIX | oui (débiteur de la QR) | oui (`check_lengths`, `validate_optional`) | **dans le périmètre (#1331)** |
  | `contacts/+page.svelte:872`, `:882` prénom/nom (70), `:906` raison sociale (255) | 70 / 255 | oui (nom du débiteur) | raison sociale : oui (`contacts.rs:396-398`, 255) ; **prénom et nom : aucune longueur propre n'a été trouvée** (`validate_common` ne borne que le nom composé, `:396`) | **dans le périmètre (D9)** : refus serveur vérifié champ par champ ; s'il manque, borne ajoutée (colonne) |
  | `products/+page.svelte:637` nom de l'article (255) | colonne | oui (`InvoiceForm.svelte:482` `description: p.name,`, imprimée sur le PDF) | oui (`products.rs:187` `if trimmed_name.chars().count() > MAX_NAME_LEN {`) | **dans le périmètre (V1-4, D9)** |
  | `contacts/+page.svelte:1066` numéro de client (50) | 50 | oui (`invoice_pdf_service.rs:439` `debtor_client_number: contact.client_number.clone(),`) | oui (`contacts.rs:451`, `MAX_CLIENT_NUMBER_LEN`) | **dans le périmètre (V1-4, D9)** |
  | `contacts/+page.svelte:1105` conditions de paiement (100) | 100 | oui (`onContactSelect` → `paymentTerms` → `invoice_pdf_service.rs:418` `payment_terms: invoice.payment_terms.clone(),`) | oui (`contacts.rs:429`, `MAX_PAYMENT_TERMS_LEN`) | **dans le périmètre (V1-4, D9)** |
  | `SupplierCreatePanel.svelte:210` pays du fournisseur (2) | ISO | non (fournisseur) | oui (`check_lengths`, « code ISO à 2 lettres ») | **dans le périmètre (#1332)** : refus explicite au lieu de la troncature |
  | `products/+page.svelte:646` description de l'article (1000) | colonne | non (le PDF porte `p.name`, `:482`) | — | sans objet |
  | `bank-accounts/+page.svelte:860`, `:1118` description (`BANK_ACCOUNT_DESCRIPTION_MAX`) | colonne | non (la QR porte l'IBAN et le nom du créancier, non la description) | oui (#1005 le suit : P4) | hors périmètre, **issue #1005 existe** |
  | autres (`ContactPersonsManager`, `BankProfileForm`, `RuleFormModal`, `api-keys`, `projects`, `fiscal-years`, `DunningPauseDialog`, `supplier-invoices/+page.svelte:284`, `ManualMatchModal`, `TransactionSplitModal`, `contacts/+page.svelte:960` e-mail, `:965` téléphone, `:1086` délai (3 chiffres)) | diverses | non | — | sans objet (ne finit pas sur la QR-facture ni le PDF client) |

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
  | `onboarding.rs:542` adresse de la société | longueurs, NPA/localité non vides au sens QR | jeu de caractères SIX | **dans le périmètre** (D5, **issue à ouvrir** pour la tracer : même défaut, autre champ) |
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
- **D4 — #1243 : le champ ne tronque plus ; le serveur refuse, l'écran affiche son message.** On retire `maxlength` des cinq champs de l'adresse, du téléphone, du site web **et de l'e-mail** (après D9) de la même section (même défaut, même bloc, serveur qui refuse avec une phrase). La saisie reste en cas de refus
  (comportement existant de `saveAddress`, `:621-622`). **Option écartée** : un refus **local** avec le nombre (comme le nom, `settings-company-name-too-long`) — il copierait dans le frontend les bornes que le serveur énonce déjà et exigerait une clé i18n par champ ; le nom l'a fait parce qu'il n'avait ni l'un ni l'autre
  message avant #1097. **Limites assumées** : le message du serveur est en français seul (#710) ; **l'adresse dit le champ et le nombre saisi** (`address_input.rs:68`), alors que **le téléphone et le site web ne disent que le maximum** (V1-10 : « … (50 caractères au plus) », `companies.rs:281`).
- **D5 — #1226 : le nom est jugé par `validate_company_name_for_qr`, l'adresse par `validate_required_for_qr`.** `set_coordinates` compose le nom comme aujourd'hui (`:526-534`), puis le fait juger par
  `super::companies::validate_company_name_for_qr` à la place de `is_empty` (le nom **composé** « prénom nom » est jugé : c'est lui qui part au PDF), et appelle `validate_required_for_qr` à la place de `validate_required` (jeu de caractères de l'adresse).
  **Pas de seconde fonction** (DRY) : la fonction de la route de modification est la source. L'ordre des refus reste : nom, adresse, IDE, puis verrou d'étape (un 400 de forme précède un refus d'étape, comme aujourd'hui). Le nom enregistré est le nom **rogné** que la
  fonction rend. **Option écartée** : ne traiter que le nom, laisser l'adresse (le défaut est le même et la correction tient en un appel ; mais l'adresse a son **issue à ouvrir** pour la tracer, car #1226 ne la nomme pas — « on trace toujours »).
  **Limite assumée** : le message du nom dit « Le nom de la société … » même pour une personne physique (prénom + nom) ; la phrase reste exacte (le nom de la raison individuelle).
- **D6 — l'écran de l'assistant ne change pas.** Il affiche déjà le message du serveur (`:222-225`) et ne tronque rien ; copier les bornes dans l'écran de l'assistant dupliquerait celles de la QR-facture. **Option écartée** : un compteur de caractères sous le champ.
- **D7 — le manuel** : `user-manual.tex:1520` reçoit une phrase (« rue 70, numéro 16, NPA 16, localité 35 caractères au plus : un champ trop long est refusé, jamais coupé, et le message nomme le champ et le nombre »), et une phrase dit où l'assistant **V1-9** : `user-manual.tex:1522` (« Un nom saisi à l'assistant et refusé ensuite par la QR-facture se corrige ainsi, sans repasser par l'assistant. ») est **réécrit** : « …se corrigeait ainsi pour une société configurée avant la version~0.20.0 ; depuis, l'assistant refuse un tel nom » (numéro posé à l'intégration) ; la phrase « l'assistant applique les mêmes contrôles (nom et adresse) » va à **l'item « Coordonnées légales », `user-manual.tex:219`** (qui décrit déjà les refus de l'assistant), non à `:214` (légende de capture). Contrôle : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé ensuite par la QR-facture se corrige ainsi'` avant (`1`) / après.
  applique **les mêmes contrôles** (nom et adresse). `.tex` seulement, pas de PDF (régénéré à la release). La phrase destinée au manuel d'administration est dans la Note de version.
- **D8 — E2E dans le navigateur pour #1233, #1243 et #1226.** Le défaut du client se voit dans le navigateur (une réponse qui arrive tard) : `page.route` retarde `GET /api/v1/contacts/{id}`. Le défaut de l'adresse se voit dans le navigateur (le champ coupe un texte collé). **#1226 aussi** (V1-1) : la KF-029 ne vise que `onboarding-path-b.spec.ts` (`:65`, `:92`) et `onboarding.spec.ts` (`:57`, `:77`, `:150`) ;
  **`onboarding-motif.spec.ts` (#809) est verte, hors de la KF-029**, et joue déjà l'écran des coordonnées jusqu'au toast du serveur (`onboarding-motif.spec.ts:14-15`, `:57` `await page.fill('#coord-name', 'Exemple Sàrl');`, `:60` `await page.fill('#coord-country', 'Suisse');`) : un cas neuf s'y ajoute (AC8). Les tests Rust (AC6, AC7) prouvent la frontière HTTP, Vitest l'écran isolé.
- **D9 — le périmètre élargi (décision de Guy du 2026-11-18) : #1331, #1332, #1333 et trois sites de la validation (V1-4), même remède, dans la story, sans issue.** Règle : pour chaque `maxlength` retiré, **le refus du serveur est vérifié champ par champ** ; **s'il n'existe pas, la borne est ajoutée côté serveur** (refus explicite, `400 VALIDATION_ERROR`) **avant** de retirer l'attribut — sans quoi la coupure ferait place à un 500.
  - **#1333, e-mail de la société** : `update_company_email` (`companies.rs:140-149`) reçoit une borne de **320 caractères** (celle de la colonne ; `contacts.rs:38` `MAX_EMAIL_LEN` = 320, même valeur), avec une clé de catalogue `error-company-email-too-long` (quatre locales, patron `error-company-phone-too-long`, `companies.rs:281`) et un repli français ; `kesh-i18n` est touché. Puis `maxlength={320}` retiré (`+page.svelte:906`).
  - **#1331, adresse du contact** : retrait des cinq `maxlength` de `contacts/+page.svelte:992-1008` ; le serveur refuse (`address_input.rs:58-80`, via `validate_optional`). Le refus s'affiche, la saisie reste (comportement du formulaire : à lire au développement).
  - **#1332, pays du fournisseur neuf** : `maxlength="2"` retiré de `SupplierCreatePanel.svelte:210` ; le serveur refuse « adresse : le pays doit être un code ISO à 2 lettres (ex. CH) » ; l'erreur s'affiche dans `supplier-new-error` (`:230-231`, `catch` `:138`), la saisie reste.
  - **V1-4, trois sites** : nom de l'article (`products/+page.svelte:637`, serveur : `products.rs:187`), numéro de client (`contacts/+page.svelte:1066`, serveur : `contacts.rs:451`), conditions de paiement (`contacts/+page.svelte:1105`, serveur : `contacts.rs:429`) : `maxlength` retiré, refus serveur existant.
  - **Prénom et nom du contact** (`contacts/+page.svelte:872`, `:882`, 70) : le serveur ne borne que le nom composé (255) ; la colonne est à vérifier au développement (étape 0) ; **s'il n'y a pas de borne serveur, elle est ajoutée** (D9) avant de retirer le `maxlength` ; sinon retrait simple.
  - **Option écartée** : laisser ces sites en issues (la décision de Guy est de réduire les bugs) ; **option écartée** : retirer un `maxlength` sans borne serveur.
  - **Restent hors périmètre, avec leur issue ou sans objet** : description du compte bancaire (#1005, P4) ; `maxlength` d'e-mail et de téléphone du contact (`contacts/+page.svelte:960`, `:965` : bornes serveur `MAX_EMAIL_LEN`, `MAX_PHONE_LEN` existantes, mais **ne finissent pas** sur le PDF du client) — laissés tels quels .
- **D10 — #1233 : un texte tapé dans le champ client compte comme une action de l'utilisateur** (V1-3, décision de l'orchestrateur du 2026-11-18). `ContactPicker` jette le brouillon quand `selected` change (`reinitialiser()`, `draft = null`) : une réponse tardive réussie effacerait le texte tapé et ferait tomber `contactUnresolved`.
  `onContactDraftChange(true)` fait donc `contactGen++` : la réponse tardive est jetée, et un champ resté vide se refuse visiblement (`:562`). Le défaut est dans le périmètre de #1233 (un bug trouvé dans la story se corrige dans la story). **Option écartée** : une issue à part.

- **D11 — #1334 (décision de Guy du 2026-11-18) : prénom ou nom seul d'une personne physique, borné par l'API.** `set_coordinates` (`onboarding.rs:526-534`) ne borne pas `first_name` / `last_name` quand un seul est fourni (`companies.first_name`/`last_name` `VARCHAR(70)`, `20260705000002_person_names.sql:11-12`) : le nom composé n'existe pas, `name` vient de `body.name`, et la valeur trop longue atteint la base (erreur, probablement 500).
  Remède : refus `400 VALIDATION_ERROR` **nommant le champ** (« Le prénom … », « Le nom de famille … » + nombre saisi), **même borne que le nom composé** (`kesh_qrbill::validation::NAME_MAX`, 70), jugé avec le nom, **avant** l'adresse et le verrou d'étape. Même contrôle pour le cas où les deux sont fournis (la composition reste jugée par D5). **Option écartée** : une issue (décision de Guy).

## Critères d'acceptation

1. **AC1 — #1233, le client choisi n'est plus écrasé** (`InvoiceForm.test.ts`, section neuve « Story 48-13 », `getContact` mocké par une promesse résolue à la main) : facture en modification ouverte ; **avant** la réponse de `getContact`, l'utilisateur choisit un autre client
   (`onSelect`) ; la réponse arrive : `selectedContact` reste le client choisi (champ affiché, et `updateInvoice` reçoit `contactId` du client choisi) ; **idem si la réponse tardive échoue** : aucun `errorMsg` « Impossible de charger le contact » ; **sans choix**, la réponse s'applique comme aujourd'hui.
2. **AC2 — le rechargement sur conflit (filet de non-régression, sans preuve du rouge : D2)** : une réponse tardive du chargement **initial** arrivant **après** « Recharger » n'écrase pas le client rechargé du serveur (test Vitest, même section) ; le rechargement lui-même remplace toujours le client comme aujourd'hui (le test existant « AC9-bis », `InvoiceForm.test.ts:462`, reste vert).
3. **AC3 — #1233, E2E** (`frontend/tests/e2e/invoice-contact-tardif.spec.ts`, ou une section de `invoices.spec.ts`) : un brouillon est ouvert en modification, `GET /api/v1/contacts/{id}` du client d'origine est **retardé** par `page.route` ; l'utilisateur choisit un autre client ; la réponse est libérée ;
   le champ client montre le client choisi ; l'enregistrement part avec lui ; **après rechargement dur**, le brouillon porte le client choisi. Sans le correctif, le test rougit (mutation M1).
4. **AC4 — #1243, les champs ne tronquent plus** (`settings-page.test.ts`) : **aucun** des cinq champs de l'adresse, ni le téléphone, le site web et l'e-mail n'a d'attribut `maxlength` (patron `:1327-1328` : **c'est cette assertion qui porte le rouge** — dans jsdom un `maxlength` ne contraint pas une valeur posée par programme : « une rue de 71 caractères part entière » passe aussi **avec** l'attribut, V1-6, et est écrite comme filet, sans preuve du rouge) ;
   la réponse `400` du serveur (« adresse : le champ rue dépasse 70 caractères (71) ») s'affiche sous le formulaire, **la saisie reste** et rien n'est enregistré ; un pays « Suisse » part entier et revient refusé ; raccourcie, la rue s'enregistre.
5. **AC5 — #1243, E2E** (`frontend/tests/e2e/company-address.spec.ts`, patron `:113`) : un texte **collé** de 71 caractères dans la rue (`page.fill`, que le `maxlength` tronquait) est refusé avec le message du serveur qui nomme « rue » et « 71 » ; la valeur enregistrée de la société est **inchangée** (relue par l'API) ;
   l'adresse de la société partagée est restaurée en `finally` (comme `restoreAddress`).
6. **AC6 — #1226, le nom de l'assistant** (`crates/kesh-api/tests/onboarding_path_b_e2e.rs`, trois ou quatre `#[sqlx::test]` neufs sur le patron de `coordinates_rejects_country_name`, `:225`) : `POST /api/v1/onboarding/coordinates` rend `400 VALIDATION_ERROR` et **n'écrit rien** (état toujours à l'étape 5,
   société inchangée) pour : un nom de **71 caractères** (message : `70` et `71`) ; un nom portant un caractère hors du jeu (« 张 » ; le message le nomme avec son point de code) ; un nom fait du seul U+00AD (« obligatoire ») ; **un prénom + nom dont la composition dépasse 70 caractères** (nom composé jugé) ; et **accepte** un nom de 70 caractères, un nom avec « & », « é » et « ’ » (U+2019),
   et un nom entouré d'espaces **rogné** à l'enregistrement.
7. **AC7 — #1226, l'adresse de l'assistant** (même fichier) : une rue ou une localité portant un caractère hors du jeu est refusée avec le champ et le caractère nommés (« adresse : le champ localité contient un caractère que la QR-facture n'admet pas : … »), rien d'écrit ; les tests existants
   `coordinates_rejects_country_name`, `coordinates_rejects_unknown_iso_country_817`, `coordinates_validates_ide`, `onboarding_audit_e2e` et `fiscal_years_e2e.rs:998` restent verts **sans modification de leurs données**.
8. **AC8 — l'assistant à l'écran** (V1-1) : (a) Vitest (`onboarding-screens.test.ts` ou voisin) : sur `400`, le message du serveur s'affiche (toast), l'écran reste à l'étape des coordonnées, saisie conservée, aucune borne copiée dans `onboarding/+page.svelte` (grep : aucun `70` ni `maxlength` neufs) ; (b) **E2E dans `onboarding-motif.spec.ts`** (verte, hors KF-029) : cas « nom de 71 caractères » — le toast nomme `70` et `71`, état toujours à l'étape 5 ; puis saisie corrigée (70 caractères) qui passe. Preuve du rouge : M4 rejouée en E2E.
9. **AC9 — factures réelles : rien ne change pour une facture existante** (contrainte de Guy du 2026-11-18). Prouvé par :
   a. **ouverture d'un brouillon en modification, sans aucune interaction** : le client d'origine s'affiche, les lignes, les totaux, la TVA, les conditions et l'échéance sont **identiques** à la valeur chargée, et l'enregistrement sans toucher envoie le même corps qu'avant (test Vitest d'aller-retour, patron des tests « aller-retour » `InvoiceForm.test.ts:696-722`) ;
   b. **affichage d'une facture validée** : `git diff origin/epic/48-corrections-0-20-0..HEAD --stat -- 'frontend/src/routes/(app)/invoices/[id]/+page.svelte' crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report` **vide** (sortie copiée au Dev Agent Record) ;
   c. **PDF à l'identique** : un test d'intégration existant du PDF (`invoice_pdf_e2e`, nommé au développement) passe **sans modification**, et le test **avant/après** de la release (`scripts/recette-montee.sh`) est à la charge de l'orchestrateur ; la story n'écrit rien dans `invoices`, `invoice_lines`, `companies` hors des gestes décrits ;
   d. **société déjà configurée** : `POST /api/v1/onboarding/coordinates` après la fin de l'assistant reste refusé comme avant (`lock_state_at_step(…, 5, …)`, `onboarding.rs:559`) ; un test Rust le pose (l'étape 5 passée, le même POST valide rend le refus d'étape existant, société inchangée).
10. **AC10 — compteurs, gardes, manuel** : `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) **recompté depuis la source** (la clé `error-company-email-too-long` est une clé de **catalogue serveur**, hors des sites de l'écran ; s'il bouge, le dire) ; `npm run lint-i18n-ownership` vert ; `audit_route_registry` et `audit_acteur_garde` verts (aucune route ajoutée) ;
    `cargo test -p kesh-api --test textes_coherents` vert ; manuel (D7) : `.tex` seulement ; `git grep -n -E "70 caract" -- docs/manual/fr/user-manual.tex` cité avant/après ; **les quatre catalogues portent la clé neuve**, même ordre.
11. **AC11 — recette visuelle** (thème clair, base de démonstration, captures jointes au Dev Agent Record et au prompt de la revue, `kesh-gate-logs/e4813/captures/`) : (1) *Paramètres → Organisation* : formulaire d'adresse avec un texte collé de 71 caractères **non coupé** ; (2) le refus du serveur sous le formulaire, saisie conservée ;
    (3) le formulaire du téléphone avec un texte trop long refusé ; (4) l'écran d'édition d'un brouillon **après** choix d'un client pendant que la réponse initiale est retardée : le client choisi est affiché ; (5) l'assistant, étape des coordonnées, nom de 71 caractères : le toast du serveur ; (6) adresse d'un contact collée de 71 caractères : refus, saisie conservée ; (7) pays « Suisse » d'un fournisseur neuf : refus explicite, non « Su » ; (8) e-mail de la société de plus de 320 caractères : refus du serveur. Les captures du brouillon sont prises sur une **base jetable** (jamais sur celle du NAS).

12. **AC12 — #1333, borne serveur de l'e-mail** (`crates/kesh-api/tests/company_*_e2e.rs`, fichier de l'e-mail à nommer au développement ; `sqlx::test`) : `PUT /api/v1/companies/current/email` avec 320 caractères valides → `200` ; avec 321 → `400 VALIDATION_ERROR` (message de la clé `error-company-email-too-long`), **rien d'écrit**, `version` inchangée ; clé présente dans les quatre catalogues (garde de catalogue verte). Rouge sans la borne (M7).
13. **AC13 — #1331, #1332 et les sites de V1-4** (Vitest ; l'E2E à choisir : `contacts` existant) : pour chacun — adresse du contact (cinq champs), pays du fournisseur neuf, nom de l'article, numéro de client, conditions de paiement (et prénom/nom du contact) — **aucun attribut `maxlength`** (rouge sans le correctif) ; la valeur trop longue partie au serveur revient en refus affiché, **saisie conservée** ; une valeur de longueur permise s'enregistre. **Pour chaque champ, le refus serveur est vérifié par un test Rust nommé** (déjà existant : le citer ; sinon à écrire : si le prénom ou le nom du contact n'est pas borné, `contacts` refuse 71 caractères en `400`, rouge sans la borne, M8).
14. **AC14 — D10, texte tapé dans le champ client** (`InvoiceForm.test.ts`) : facture en modification, réponse initiale retardée ; l'utilisateur **tape** un texte (sans choisir) ; la réponse arrive : le texte tapé est **conservé**, `selectedContact` reste `null`, et l'enregistrement est refusé (« Veuillez sélectionner un contact » / « Choisissez le client dans la liste »). Rouge sans `contactGen++` de `onContactDraftChange(true)` (M9).
15. **AC15 — #1334, prénom ou nom seul borné** (`onboarding_path_b_e2e.rs`, `sqlx::test`) : `POST /onboarding/coordinates` avec **`firstName` seul de 71 caractères** → `400 VALIDATION_ERROR` qui nomme « prénom » et `71`, **sans erreur de base ni 500**, rien d'écrit, état à l'étape 5 ; idem `lastName` seul ; 70 caractères acceptés. Rouge sans le contrôle (M11 : le retirer).


## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; le journal attendu est dans `/home/gcorbaz/devel/kesh-gate-logs/e4813/`. Ordre : **le test d'abord, rouge** (journal `rouge-<n>.log`, sortie copiée), puis le correctif, vert.

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC1 (choix avant la réponse ; réponse en échec) | la garde `myGen` : `selectedContact` revient au client d'origine, ou `errorMsg` s'affiche | mutation M1 : retirer `if (myGen !== contactGen) return;` du `.then`, M2 : du `.catch` ; journal `mut-m1.log`, `mut-m2.log` |
| AC2 (réponse initiale après « Recharger ») | **ne rougit pas** : `contactGen++` de `reloadFromServer` est défensif (D2 : le scénario est inatteignable par l'interface, `:562`, `:722`) | aucune mutation ; filet de non-régression, déclaré tel |
| AC3 (E2E client tardif) | la garde | M1 rejouée en E2E ; `e2e-m1.log` |
| AC4 (aucun `maxlength`) | les attributs `maxlength` | **mutation par l'outil Edit** : remettre un `maxlength` sur un champ (M6) ; `mut-m6.log` — **pas de `git stash`** (consigne Edit seulement). L'assertion « 71 caractères partent entiers » ne rougit pas (jsdom) : filet déclaré |
| AC5 (E2E texte collé) | idem | M10 (mutation Edit : `maxlength` remis sur la rue) rejouée en E2E ; `e2e-m10.log` |
| AC6, AC7 (Rust) | `validate_company_name_for_qr` et `validate_required_for_qr` dans `set_coordinates` | M4 : remettre `if name.is_empty()` ; M5 : remettre `validate_required()` ; `mut-m4.log`, `mut-m5.log` |
| AC8 b (E2E `onboarding-motif.spec.ts`) | idem | M4 rejouée en E2E ; `e2e-m4.log` |
| AC12 (borne e-mail) | la borne serveur | M7 : retirer la borne ; `mut-m7.log` |
| AC13 (retraits des `maxlength`, bornes serveur) | les attributs ; la borne ajoutée si prénom/nom | M8 : retirer la borne ajoutée ; mutations Edit remettant un `maxlength` par site ; `mut-m8.log` |
| AC14 (texte tapé) | `contactGen++` de `onContactDraftChange(true)` | M9 : retirer cette ligne ; `mut-m9.log` |
| AC15 (prénom ou nom seul) | le contrôle de longueur ajouté | M11 : retirer le contrôle ; `mut-m11.log` |
| AC6 (« accepte » : 70 caractères, `&`, `é`, `’`, rognage) | **ne rougissent pas** (filets de non-régression : le nom accepté l'était déjà), à dire dans le journal | — |
| AC9 a, d | **ne rougissent pas** (non-régression par construction), à dire | — |

*Numérotation des mutations (V1-7) : M1 à M11 sont uniques ; M4 ne désigne que la mutation Rust du nom de l'assistant (rejouée en E2E par AC8 b), M10 que celle de l'AC5.*

Après une mutation restaurée : `touch` du fichier (cargo garde sinon le binaire muté). Tout test qui ne rougit pas sans le correctif est **déclaré tel**, dans le Dev Agent Record, avec la raison.

## Tâches

1. **Étape 0** : relire les lignes citées à la tête de la branche (celles de ce document ; copier trois au Change Log) ; lire la colonne de `contacts.first_name`/`last_name` et décider de la borne serveur (D9) ; relire `ContactPicker.svelte` (`reinitialiser()`) pour écrire le test AC14.
2. `InvoiceForm.svelte` : `contactGen`, garde du `.then` et du `.catch`, `contactGen++` dans `onContactSelect`, `onContactDraftChange(true)` (D10) et `reloadFromServer` (D2, défensif) ; tests AC1, AC2, AC9 a, AC14 ; E2E AC3.
3. `settings/+page.svelte` : retrait des huit `maxlength` (D4, après la borne de l'e-mail, tâche 6) ; tests AC4 ; E2E AC5.
4. `onboarding.rs` : `validate_company_name_for_qr` et `validate_required_for_qr` (D5) ; borne du prénom ou du nom seul (D11, AC15) ; tests AC6, AC7, AC9 d ; test d'écran AC8 a ; **cas E2E dans `onboarding-motif.spec.ts`** (AC8 b).
5. Manuel (D7), `.tex` seulement ; recomptes (AC10) ; captures (AC11).
6. **Périmètre élargi (D9)** : borne serveur de l'e-mail et clé `error-company-email-too-long` ×4 (AC12) ; retraits des `maxlength` de `contacts/+page.svelte` (adresse, prénom, nom, numéro de client, conditions), `SupplierCreatePanel.svelte`, `products/+page.svelte:637` ; pour chaque champ, **vérifier le refus serveur et le citer** ; borne ajoutée si absente (prénom, nom du contact : colonne à lire) ; tests AC13.
7. Rédiger au Dev Agent Record les mutations, les gates, les captures et le grep de l'AC9 b. **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.**
## Gates (`e35-dev-commun.md` § *Quels gates, quand*)

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(onboarding_path_b_e2e) | binary(onboarding_audit_e2e) | binary(fiscal_years_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e)'` ;
  `npm run check` ; Vitest de `InvoiceForm`, `settings`, `onboarding`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** (V1-8, aligné sur `e35-dev-commun.md` § *Quels gates, quand* : « Pas d'E2E ») : `scripts/test-fast.sh --ci` et `scripts/gate-ci.sh` (voie 1, sa clé de base), frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`). **Pas d'E2E complet** à ce stade ; les E2E neufs de la story (AC3, AC5, AC8 b) se jouent seuls pendant le développement.
- **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (montage de `docs/testing.md` § *Prérequis Playwright local*, `KESH_COOKIE_SECURE=false` ; « vert » = les seuls échecs attendus (sept de la KF-029 plus zéro à deux de pollution), jugés fichier par fichier). **Une commande lourde à la fois**, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant chaque commande lourde, verrou `.gate.lock`. Bases : celles de la voie donnée par le prompt d'appel.

## Note de version

**Corrigé**

- Facture : ouvrir un brouillon puis choisir un autre client avant que le client d'origine ne soit chargé ne remplace plus le client choisi (refs #1233).
- *Paramètres → Organisation* : l'adresse de la société, son téléphone, son site web et son e-mail ne sont plus coupés en silence quand on colle un texte trop long ; Kesh refuse et garde la saisie (l'adresse dit le champ et le nombre de caractères saisis, les autres champs disent le maximum) ; l'e-mail de la société est désormais borné à 320 caractères côté serveur (refs #1243, refs #1333).
- Contacts, fournisseurs et articles : l'adresse d'un client, le prénom et le nom, le numéro de client, les conditions de paiement, le nom d'un article et le pays d'un fournisseur créé depuis une facture fournisseur ne sont plus coupés en silence ; Kesh refuse et dit pourquoi (refs #1331, refs #1332).
- Facture : un texte tapé dans le champ client avant la fin du chargement du client d'origine n'est plus effacé par ce chargement (refs #1233).
- Assistant de démarrage : le nom de la société (ou « prénom nom ») et son adresse sont contrôlés comme la QR-facture les contrôlera — non vide, 70 caractères au plus pour le nom, jeu de caractères de la QR-facture — au lieu de s'enregistrer puis de faire refuser la première QR-facture (refs #1226) ; un prénom ou un nom seul de plus de 70 caractères est refusé en `400` au lieu de provoquer une erreur de base (refs #1226, refs #1334).

*Pour le manuel d'administration, section « Passer à la 0.20.0 » (posée à l'intégration)* : `POST /api/v1/onboarding/coordinates` refuse désormais, en `400 VALIDATION_ERROR`, un nom de plus de 70 caractères ou portant un caractère que la QR-facture n'admet pas, et un caractère
refusé dans l'adresse ; aucune société déjà configurée n'est modifiée, aucune facture ni aucun PDF existant ne change.

## Dev Notes

- **Modules touchés : quatre, comptés au grain de la crate et de l'application** (même règle que le découpage de la 49-1 ; règle du `CLAUDE.md` : plus de cinq → découpage) : (1) `kesh-api` (`routes/onboarding.rs`, `routes/companies.rs` pour la borne de l'e-mail, éventuellement `routes/contacts.rs`, tests), (2) `kesh-i18n` (la clé `error-company-email-too-long`, quatre catalogues), (3) le frontend (`InvoiceForm.svelte`, `settings/+page.svelte`, `contacts/+page.svelte`, `products/+page.svelte`, `SupplierCreatePanel.svelte`, tests Vitest et E2E),
  (4) les manuels (`.tex`). **Quatre ≤ cinq : pas de découpage, aucune dérogation à écrire.** *(Au grain du dossier — composants, routes, features —, la story toucherait davantage ; ce grain n'est pas celui de la règle.)*
- **Issues** : toutes les issues du périmètre sont dans la story (#1233, #1243, #1226, #1331, #1332, #1333, #1334). **Aucune issue à ouvrir.** Le jeu de caractères de l'adresse de l'assistant est corrigé par D5 et consigné au Change Log, sans issue (V1-11).
- **Fichiers communs** (plan de la release et fichier d'épique 48) : `InvoiceForm.svelte` avec l'Epic 49 (49-1, 49-2, 49-3, éditeur de lignes) — **la 48-13 se fusionne d'abord** ; `settings/+page.svelte` avec la v0200-menu-sous-groupes (blocs distincts) ;
  `i18n-keys.test.ts` (`sitesTotal`), catalogues i18n, `sprint-status.yaml` : recomptés / résolus en gardant les deux côtés. Pas de PDF de manuel commité. **Contributeur Beat** (PR #1327, `kesh-report/src/pdf.rs`, `kesh-qrbill` `largeurs.rs` et `lib.rs`) : la story ne touche aucun de ces fichiers.
- **Ce qui doit rester vrai** : un brouillon s'ouvre avec son client d'origine quand l'utilisateur ne fait rien ; le rechargement sur conflit remplace le client par celui du serveur ; l'adresse de la société se refuse côté serveur avec le champ et le nombre ; l'assistant refuse *avant* le verrou d'étape ;
  un nom de 70 caractères à `é`, `&`, `’` est accepté.
- **Aucune donnée réelle** dans les tests, captures ni journaux : « Démo SA », adresses factices.
- Références : `v0190-nom-societe.md` (D2, D5, C-v0190c-25), `v0190-correction.md` (`ContactPicker`, #1162), `e0200-commun.md`, plan `release-0.20.0.md` et `epic-48-corrections-0-20-0.md` (branche `chore/kickoff-0.20.0`).

## Dev Agent Record

*(vide — rempli au développement)*

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

