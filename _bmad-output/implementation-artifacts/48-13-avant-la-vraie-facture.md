# Story 48-13 : Avant la vraie facture — client écrasé par un chargement tardif, adresse tronquée en silence, nom de société non contrôlé à l'assistant (#1233, #1243, #1226)

Status: ready-for-dev

*Fiche rédigée le 2026-11-18 (Sonnet 5.5), worktree `kesh-48-13`, branche `story/48-13-avant-la-vraie-facture` partie de
`epic/48-corrections-0-20-0` (= `main` = v0.19.0, `773df80da`), par le workflow `bmad-create-story`. Release **0.20.0** (sortie le
vendredi 2026-11-27, coupure le jeudi 2026-11-26 au soir). Story **prioritaire** : elle porte sur le chemin de la première facture client
QR de fin novembre, et **se fusionne avant** les stories de l'Epic 49 qui touchent `InvoiceForm.svelte` (arbitrage de Guy du 2026-11-18).
**Aucune migration** : aucune colonne ne change (`companies.name` `VARCHAR(255)`, adresse en cinq colonnes, inchangées) ; si le
développement en découvre une, il s'arrête et le dit. `#1234` (`billing_information`) est **retirée** de la story et de la 0.20.0 (décision de
Guy du 2026-11-18) : `kesh-qrbill` n'est pas touché.*

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
  | `settings/+page.svelte:906` e-mail (320) | colonne | oui (`:427` `creditor_email`) | **non** : `update_company_email` ne contrôle que la forme (`contacts.rs:266` `is_valid_email_simple`, aucune longueur) | **hors périmètre** : retirer le `maxlength` ouvrirait un 500 ; **issue à ouvrir** (voir Dev Notes) |
  | `contacts/+page.svelte:992-1008` adresse du client (rue 70, n° 16, NPA 16, localité 35, pays 2) | SIX | oui (débiteur de la QR) | oui (`check_lengths`, `validate_optional`) | **hors périmètre** (autre écran, autre story) : **issue à ouvrir** |
  | `contacts/+page.svelte:872`, `:882` prénom/nom (70), `:906` raison sociale (255) | 70 / 255 | oui (nom du débiteur) | le PDF refuse **explicitement** un nom client trop long (`invoice_pdf_service.rs:765` `"invoice-pdf-error-too-long-client-name"`) | sans objet (refus explicite) |
  | `SupplierCreatePanel.svelte:210` pays du fournisseur (2) | ISO | non (fournisseur : pas d'adresse sur la facture client) | oui | hors périmètre : **issue à ouvrir** (même symptôme que #809) |
  | `bank-accounts/+page.svelte:860`, `:1118` description (`BANK_ACCOUNT_DESCRIPTION_MAX`) | colonne | non (la QR porte l'IBAN et le nom du créancier, non la description) | oui (#1005 le suit : P4) | hors périmètre, **issue #1005 existe** |
  | autres (`ContactPersonsManager`, `BankProfileForm`, `RuleFormModal`, `products/+page.svelte:637-646`, `api-keys`, `projects`, `fiscal-years`, `DunningPauseDialog`, `supplier-invoices/+page.svelte:284`, `ManualMatchModal`, `TransactionSplitModal`, `contacts/+page.svelte:960-1105`) | diverses | non | — | sans objet (ne finit pas sur la QR-facture ni le PDF client) |

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
- **D2 — le rechargement sur conflit incrémente aussi le compteur.** `reloadFromServer` fait `contactGen++` avant de charger son propre client, de sorte qu'une réponse tardive du chargement **initial** n'écrase pas le client rechargé du serveur ; sa propre
  garde (`reloadSeq`) est conservée telle quelle. **Option écartée** : donner au rechargement la même garde `myGen` — l'utilisateur ne peut pas choisir un client pendant la modale de conflit (`conflictOpen`), la garde `reloadSeq` suffit.
- **D3 — `ProductPicker` : relevé, non modifié par défaut.** `ProductPicker.svelte:48` charge une liste de recherche ; le développement relit son code à l'étape 0. Si la réponse d'une recherche plus ancienne peut **écraser une liste plus récente**
  (pas de garde `seq`), c'est une **issue à demander à l'orchestrateur** (hors du chemin « client écrasé »), non un correctif de cette story. Si elle est gardée, la fiche l'écrit au Dev Agent Record.
- **D4 — #1243 : le champ ne tronque plus ; le serveur refuse, l'écran affiche son message.** On retire `maxlength` des cinq champs de l'adresse **et** du téléphone et du site web de la même section (même défaut, même bloc, serveur qui refuse déjà
  avec une phrase). La saisie reste en cas de refus (comportement existant de `saveAddress`, `:621-622`). **Option écartée** : un refus **local** avec le nombre (comme le nom, `settings-company-name-too-long`) — il copierait dans le frontend quatre bornes
  que le serveur énonce déjà, avec le champ et le nombre, et exigerait une clé i18n par champ (quatre catalogues) ; le nom l'a fait parce qu'il n'avait ni l'un ni l'autre message avant #1097. **Limite assumée** : le message du serveur est en français seul (#710,
  comme celui de la modification de l'adresse, `user-manual.tex:1520`). **Hors périmètre** : l'e-mail (`:906`, pas de borne serveur : le `maxlength` y reste, **issue à ouvrir**).
- **D5 — #1226 : le nom est jugé par `validate_company_name_for_qr`, l'adresse par `validate_required_for_qr`.** `set_coordinates` compose le nom comme aujourd'hui (`:526-534`), puis le fait juger par
  `super::companies::validate_company_name_for_qr` à la place de `is_empty` (le nom **composé** « prénom nom » est jugé : c'est lui qui part au PDF), et appelle `validate_required_for_qr` à la place de `validate_required` (jeu de caractères de l'adresse).
  **Pas de seconde fonction** (DRY) : la fonction de la route de modification est la source. L'ordre des refus reste : nom, adresse, IDE, puis verrou d'étape (un 400 de forme précède un refus d'étape, comme aujourd'hui). Le nom enregistré est le nom **rogné** que la
  fonction rend. **Option écartée** : ne traiter que le nom, laisser l'adresse (le défaut est le même et la correction tient en un appel ; mais l'adresse a son **issue à ouvrir** pour la tracer, car #1226 ne la nomme pas — « on trace toujours »).
  **Limite assumée** : le message du nom dit « Le nom de la société … » même pour une personne physique (prénom + nom) ; la phrase reste exacte (le nom de la raison individuelle).
- **D6 — l'écran de l'assistant ne change pas.** Il affiche déjà le message du serveur (`:222-225`) et ne tronque rien ; copier les bornes dans l'écran de l'assistant dupliquerait celles de la QR-facture. **Option écartée** : un compteur de caractères sous le champ.
- **D7 — le manuel** : `user-manual.tex:1520` reçoit une phrase (« rue 70, numéro 16, NPA 16, localité 35 caractères au plus : un champ trop long est refusé, jamais coupé, et le message nomme le champ et le nombre »), et une phrase dit où l'assistant
  applique **les mêmes contrôles** (nom et adresse). `.tex` seulement, pas de PDF (régénéré à la release). La phrase destinée au manuel d'administration est dans la Note de version.
- **D8 — E2E dans le navigateur pour #1233 et #1243 ; pas d'E2E navigateur pour #1226.** Le défaut du client se voit **dans le navigateur** (une réponse qui arrive tard) : `page.route` retarde `GET /api/v1/contacts/{id}`. Le défaut de l'adresse se voit dans le navigateur
  (le champ coupe un texte collé). L'assistant : les specs `onboarding*.spec.ts` comptent parmi les **échecs attendus** de la KF-029 (`docs/testing.md` § *Les échecs attendus* : `onboarding-path-b.spec.ts:65`, `:92` ; `onboarding.spec.ts:57`, `:77`, `:150`) — un E2E neuf
  y serait indistinguable d'un rouge connu ; la frontière HTTP du refus se prouve en Rust (`onboarding_path_b_e2e.rs`, qui monte l'app et passe par `POST /api/v1/onboarding/coordinates`) et le comportement d'écran en Vitest.

## Critères d'acceptation

1. **AC1 — #1233, le client choisi n'est plus écrasé** (`InvoiceForm.test.ts`, section neuve « Story 48-13 », `getContact` mocké par une promesse résolue à la main) : facture en modification ouverte ; **avant** la réponse de `getContact`, l'utilisateur choisit un autre client
   (`onSelect`) ; la réponse arrive : `selectedContact` reste le client choisi (champ affiché, et `updateInvoice` reçoit `contactId` du client choisi) ; **idem si la réponse tardive échoue** : aucun `errorMsg` « Impossible de charger le contact » ; **sans choix**, la réponse s'applique comme aujourd'hui.
2. **AC2 — le rechargement sur conflit** : une réponse tardive du chargement **initial** arrivant **après** « Recharger » n'écrase pas le client rechargé du serveur (test Vitest, même section) ; le rechargement lui-même remplace toujours le client comme aujourd'hui (le test existant « AC9-bis », `InvoiceForm.test.ts:462`, reste vert).
3. **AC3 — #1233, E2E** (`frontend/tests/e2e/invoice-contact-tardif.spec.ts`, ou une section de `invoices.spec.ts`) : un brouillon est ouvert en modification, `GET /api/v1/contacts/{id}` du client d'origine est **retardé** par `page.route` ; l'utilisateur choisit un autre client ; la réponse est libérée ;
   le champ client montre le client choisi ; l'enregistrement part avec lui ; **après rechargement dur**, le brouillon porte le client choisi. Sans le correctif, le test rougit (mutation M1).
4. **AC4 — #1243, les champs ne tronquent plus** (`settings-page.test.ts`) : aucun des cinq champs de l'adresse, ni le téléphone ni le site web, n'a d'attribut `maxlength` (patron `:1327-1328`) ; une rue de 71 caractères posée par le test part **entière** à `updateCompanyAddress` ; la réponse `400` du serveur
   (« adresse : le champ rue dépasse 70 caractères (71) ») s'affiche sous le formulaire, **la saisie reste** et rien n'est enregistré ; un pays « Suisse » part entier et revient refusé ; raccourcie, la rue s'enregistre. L'e-mail **garde** son `maxlength={320}` (test d'absence de régression, hors périmètre assumé).
5. **AC5 — #1243, E2E** (`frontend/tests/e2e/company-address.spec.ts`, patron `:113`) : un texte **collé** de 71 caractères dans la rue (`page.fill`, que le `maxlength` tronquait) est refusé avec le message du serveur qui nomme « rue » et « 71 » ; la valeur enregistrée de la société est **inchangée** (relue par l'API) ;
   l'adresse de la société partagée est restaurée en `finally` (comme `restoreAddress`).
6. **AC6 — #1226, le nom de l'assistant** (`crates/kesh-api/tests/onboarding_path_b_e2e.rs`, trois ou quatre `#[sqlx::test]` neufs sur le patron de `coordinates_rejects_country_name`, `:225`) : `POST /api/v1/onboarding/coordinates` rend `400 VALIDATION_ERROR` et **n'écrit rien** (état toujours à l'étape 5,
   société inchangée) pour : un nom de **71 caractères** (message : `70` et `71`) ; un nom portant un caractère hors du jeu (« 张 » ; le message le nomme avec son point de code) ; un nom fait du seul U+00AD (« obligatoire ») ; **un prénom + nom dont la composition dépasse 70 caractères** (nom composé jugé) ; et **accepte** un nom de 70 caractères, un nom avec « & », « é » et « ’ » (U+2019),
   et un nom entouré d'espaces **rogné** à l'enregistrement.
7. **AC7 — #1226, l'adresse de l'assistant** (même fichier) : une rue ou une localité portant un caractère hors du jeu est refusée avec le champ et le caractère nommés (« adresse : le champ localité contient un caractère que la QR-facture n'admet pas : … »), rien d'écrit ; les tests existants
   `coordinates_rejects_country_name`, `coordinates_rejects_unknown_iso_country_817`, `coordinates_validates_ide`, `onboarding_audit_e2e` et `fiscal_years_e2e.rs:998` restent verts **sans modification de leurs données**.
8. **AC8 — l'assistant à l'écran** (`onboarding-screens.test.ts` ou fichier voisin) : sur `400`, le message du serveur s'affiche (toast) et l'écran reste à l'étape des coordonnées, saisie conservée ; aucune borne n'est copiée dans l'écran (grep : aucun `70` ni `maxlength` neufs dans `onboarding/+page.svelte`).
9. **AC9 — factures réelles : rien ne change pour une facture existante** (contrainte de Guy du 2026-11-18). Prouvé par :
   a. **ouverture d'un brouillon en modification, sans aucune interaction** : le client d'origine s'affiche, les lignes, les totaux, la TVA, les conditions et l'échéance sont **identiques** à la valeur chargée, et l'enregistrement sans toucher envoie le même corps qu'avant (test Vitest d'aller-retour, patron des tests « aller-retour » `InvoiceForm.test.ts:696-722`) ;
   b. **affichage d'une facture validée** : `git diff origin/epic/48-corrections-0-20-0..HEAD --stat -- 'frontend/src/routes/(app)/invoices/[id]/+page.svelte' crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report` **vide** (sortie copiée au Dev Agent Record) ;
   c. **PDF à l'identique** : un test d'intégration existant du PDF (`invoice_pdf_e2e`, nommé au développement) passe **sans modification**, et le test **avant/après** de la release (`scripts/recette-montee.sh`) est à la charge de l'orchestrateur ; la story n'écrit rien dans `invoices`, `invoice_lines`, `companies` hors des gestes décrits ;
   d. **société déjà configurée** : `POST /api/v1/onboarding/coordinates` après la fin de l'assistant reste refusé comme avant (`lock_state_at_step(…, 5, …)`, `onboarding.rs:559`) ; un test Rust le pose (l'étape 5 passée, le même POST valide rend le refus d'étape existant, société inchangée).
10. **AC10 — compteurs, gardes, manuel** : `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) **recompté depuis la source** (aucune clé neuve attendue : s'il bouge, le dire) ; `npm run lint-i18n-ownership` vert ; `audit_route_registry` et `audit_acteur_garde` verts (aucune route ajoutée) ;
    `cargo test -p kesh-api --test textes_coherents` vert ; manuel (D7) : `.tex` seulement ; `git grep -n -E "70 caract" -- docs/manual/fr/user-manual.tex` cité avant/après.
11. **AC11 — recette visuelle** (thème clair, base de démonstration, captures jointes au Dev Agent Record et au prompt de la revue, `kesh-gate-logs/e4813/captures/`) : (1) *Paramètres → Organisation* : formulaire d'adresse avec un texte collé de 71 caractères **non coupé** ; (2) le refus du serveur sous le formulaire, saisie conservée ;
    (3) le formulaire du téléphone avec un texte trop long refusé ; (4) l'écran d'édition d'un brouillon **après** choix d'un client pendant que la réponse initiale est retardée : le client choisi est affiché ; (5) l'assistant, étape des coordonnées, nom de 71 caractères : le toast du serveur. Les captures du brouillon sont prises sur une **base jetable** (jamais sur celle du NAS).

## Tests à écrire, et la preuve du rouge

Chaque test neuf rougit sans le correctif ; le journal attendu est dans `/home/gcorbaz/devel/kesh-gate-logs/e4813/`. Ordre : **le test d'abord, rouge** (journal `rouge-<n>.log`, sortie copiée), puis le correctif, vert.

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC1 (choix avant la réponse ; réponse en échec) | la garde `myGen` : `selectedContact` revient au client d'origine, ou `errorMsg` s'affiche | mutation M1 : retirer `if (myGen !== contactGen) return;` du `.then`, M2 : du `.catch` ; journal `mut-m1.log`, `mut-m2.log` |
| AC2 (réponse initiale après « Recharger ») | `contactGen++` de `reloadFromServer` | M3 : retirer cette ligne ; `mut-m3.log` |
| AC3 (E2E client tardif) | la garde | M1 rejouée en E2E ; `e2e-m1.log` |
| AC4 (aucun `maxlength` ; 71 caractères partent entiers) | les attributs `maxlength` | `git stash` du correctif de `+page.svelte` ; `rouge-ac4.log` |
| AC5 (E2E texte collé) | idem | `e2e-m4.log` |
| AC6, AC7 (Rust) | `validate_company_name_for_qr` et `validate_required_for_qr` dans `set_coordinates` | M4 : remettre `if name.is_empty()` ; M5 : remettre `validate_required()` ; `mut-m4.log`, `mut-m5.log` |
| AC6 (« accepte » : 70 caractères, `&`, `é`, `’`, rognage) | **ne rougissent pas** (filets de non-régression : le nom accepté l'était déjà), à dire dans le journal | — |
| AC9 a, d | **ne rougissent pas** (non-régression par construction), à dire | — |

Après une mutation restaurée : `touch` du fichier (cargo garde sinon le binaire muté). Tout test qui ne rougit pas sans le correctif est **déclaré tel**, dans le Dev Agent Record, avec la raison.

## Tâches

1. **Étape 0** : relire les lignes citées à la tête de la branche (celles de ce document ; copier trois au Change Log) ; lire `ContactPicker.svelte` pour savoir si un **brouillon tapé** (texte non choisi) est effacé par une réponse tardive du chargement
   initial — si oui et reproduit, le dire à l'orchestrateur (issue à ouvrir, hors périmètre : ce n'est pas « un client choisi »). Lire `ProductPicker.svelte:48` (D3).
2. `InvoiceForm.svelte` : `contactGen`, garde du `.then` et du `.catch`, `contactGen++` dans `onContactSelect` et `reloadFromServer` (D1, D2) ; tests AC1, AC2, AC9 a ; E2E AC3.
3. `settings/+page.svelte` : retrait des sept `maxlength` (D4) ; tests AC4 ; E2E AC5.
4. `onboarding.rs` : `validate_company_name_for_qr` et `validate_required_for_qr` (D5) ; tests AC6, AC7, AC9 d ; test d'écran AC8.
5. Manuel (D7), `.tex` seulement ; recomptes (AC10) ; captures (AC11).
6. Rédiger au Dev Agent Record les mutations, les gates, les captures et le grep de l'AC9 b. **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.**

## Gates (`e35-dev-commun.md` § *Quels gates, quand*)

- **En cours de développement** (gate ciblé) : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(onboarding_path_b_e2e) | binary(onboarding_audit_e2e) | binary(fiscal_years_e2e) | binary(textes_coherents) | binary(invoice_pdf_e2e)'` ;
  `npm run check` ; Vitest de `InvoiceForm`, `settings`, `onboarding`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** : `scripts/test-fast.sh --ci` et `scripts/gate-ci.sh` (voie 1, sa clé de base), frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`), **E2E complet** (montage de `docs/testing.md` § *Prérequis Playwright local*, `KESH_COOKIE_SECURE=false`) ;
  « vert » = les seuls échecs attendus (sept de la KF-029 plus zéro à deux de pollution), jugés fichier par fichier. **`kesh-db` non touché** (ni migration, ni repository, ni `post_restore.rs`) : pas d'exception.
- **À la clôture**, au dernier commit de code : `gate-ci` et E2E complet. **Une commande lourde à la fois**, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant chaque commande lourde, verrou `.gate.lock`. Bases : celles de la voie donnée par le prompt d'appel.

## Note de version

**Corrigé**

- Facture : ouvrir un brouillon puis choisir un autre client avant que le client d'origine ne soit chargé ne remplace plus le client choisi (refs #1233).
- *Paramètres → Organisation* : l'adresse de la société, son téléphone et son site web ne sont plus coupés en silence quand on colle un texte trop long ; Kesh refuse, dit le champ et le nombre de caractères, et garde la saisie (refs #1243).
- Assistant de démarrage : le nom de la société (ou « prénom nom ») et son adresse sont contrôlés comme la QR-facture les contrôlera — non vide, 70 caractères au plus pour le nom, jeu de caractères de la QR-facture — au lieu de s'enregistrer puis de faire refuser la première QR-facture (refs #1226).

*Pour le manuel d'administration, section « Passer à la 0.20.0 » (posée à l'intégration)* : `POST /api/v1/onboarding/coordinates` refuse désormais, en `400 VALIDATION_ERROR`, un nom de plus de 70 caractères ou portant un caractère que la QR-facture n'admet pas, et un caractère
refusé dans l'adresse ; aucune société déjà configurée n'est modifiée, aucune facture ni aucun PDF existant ne change.

## Dev Notes

- **Modules touchés : quatre** (règle du `CLAUDE.md`, au plus cinq sans découpage) : (1) `kesh-api` (`routes/onboarding.rs`, tests), (2) frontend composants de facture (`components/invoices/InvoiceForm.svelte`), (3) frontend réglages
  (`routes/(app)/settings/+page.svelte`), (4) manuels (`.tex`). **`kesh-i18n` n'est pas touché** (aucune clé neuve : les messages sont ceux du serveur). Pas de splitting ; aucune dérogation à écrire.
- **Hors périmètre, tracé — issues à demander à l'orchestrateur** (une issue par problème ; aucune n'existe au 2026-11-18, vérifié par `gh issue list --state open --search "maxlength"`) :
  1. *Contacts : les champs de l'adresse du client sont coupés en silence par `maxlength`* (`contacts/+page.svelte:992-1008`, même défaut que #1243, adresse du débiteur de la QR) — P3, `bug`.
  2. *Paramètres → Organisation : l'e-mail de la société n'a aucune borne de longueur côté serveur* (`update_company_email` ne contrôle que la forme, `contacts.rs:266` ; `maxlength={320}` seul protège) — P3, `bug`.
  3. *Assistant de démarrage : le jeu de caractères de la QR-facture n'est pas contrôlé sur l'adresse de la société* (`onboarding.rs:542`, corrigé dans cette story ; l'issue sert à tracer et à fermer) — P2, `bug` (même niveau que #1226).
  4. *Fournisseur : `SupplierCreatePanel.svelte:210` coupe le pays à deux caractères en silence* (« Suisse » devient « Su », même défaut que #809) — P3, `bug`.
  5. Sous condition (étape 0 de la tâche 1) : *Facture : un texte tapé dans le champ client est effacé par la réponse tardive du client initial* ; et, si `ProductPicker` n'est pas gardé : *Facture : la recherche d'article d'une réponse plus ancienne écrase une plus récente*.
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
