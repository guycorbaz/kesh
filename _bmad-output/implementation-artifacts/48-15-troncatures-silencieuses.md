# Story 48-15 : Troncatures silencieuses — adresse du client, pays du fournisseur, e-mail de la société, prénom et nom, cinq champs du PDF (#1331, #1332, #1333, #1334)

Status: done

*Fiche rédigée le 2026-11-18 (Sonnet 5.5), **découpée de la 48-13 après la validation P2** (V2-2 : la 48-13, élargie à ces sites sur décision de Guy du 2026-11-18 « diminuer le nombre de bugs », touchait neuf modules au grain du domaine ;
l'orchestrateur a déclaré le signal de splitting au Project Lead et coupé la story). Worktree `kesh-48-13`, branche `story/48-13-avant-la-vraie-facture` pour l'instant (la branche `story/48-15-troncatures-silencieuses` se crée depuis
`epic/48-corrections-0-20-0` **après la fusion de la 48-13**, dont elle dépend pour `onboarding.rs` : c'est l'ordre de **lancement** ; l'**ordre de sortie à la coupure** du kickoff (corrigé au commit `8c96fbc78`, cinq champs) la fait sortir avant la 48-11 et la 48-13, sans contradiction : sortir n'est pas se lancer). Base lue : `773df80da` (= `main` = v0.19.0). Release **0.20.0** (coupure le jeudi 2026-11-26 au soir). **Ne bloque pas l'Epic 49** : aucun fichier de l'éditeur de lignes. **Huitième story de l'Epic 48** : le kickoff de la 0.20.0 est à jour depuis le commit `cd823aacb` de `origin/chore/kickoff-0.20.0` (huit stories, 48-15 déclarée, ordre de sortie, fichiers communs, décompte des bugs), sous la dérogation de la borne de six stories décidée par Guy (le Project Lead) le 2026-11-18.
**Aucune migration** : les colonnes ne changent pas (chaque borne neuve **égale** la colonne) ; si le développement en découvre une, il s'arrête et le dit.*

## Story

En tant qu'utilisateur de Kesh qui colle un texte trop long dans une fiche de contact, de fournisseur, d'article ou dans l'e-mail de ma société, je veux que Kesh me le refuse en le disant, afin que ce qui part sur la QR-facture et le PDF soit ce que j'ai écrit, non une version coupée en silence.

## Ce qui se lit au code (`773df80da`)

### La famille : un `maxlength` qui coupe, un serveur qui sait refuser

Le motif est celui de #1243 (story 48-13, réglages de la société) : un attribut `maxlength` coupe en silence un texte **collé**, la valeur coupée est enregistrée et le toast annonce le succès ; le serveur, lui, refuse déjà (ou doit refuser) avec un message. Précédents : `settings/+page.svelte:346-351` (nom, v0190-nom-societe, C-v0190c-25) et
`onboarding/+page.svelte:751-753` (pays de l'assistant, #809 : « Pas de maxlength : « Suisse » doit atteindre le serveur et revenir expliqué, non être tronqué en « Su », accepté »).

**Règle de la story (D1)** : pour chaque `maxlength` retiré, **le refus serveur est vérifié champ par champ et cité** ; **s'il n'existe pas, la borne est ajoutée côté serveur avant de retirer l'attribut**. Sans borne serveur, la coupure ferait place à un refus muet :
`kesh-db/src/errors.rs:2602` (`DbError::DataLengthOrRange`, MariaDB 1406 et 1264) puis `kesh-api/src/errors.rs:6022` rendent un **`400 DATA_LENGTH_OR_RANGE` générique** (« Une valeur fournie est trop longue ou hors plage. ») **qui ne nomme pas le champ** (V2-3 : ce n'est pas un 500).

### Champ par champ

| Site (écran) | `maxlength` | Colonne | Finit sur le PDF / la QR ? | Refus serveur aujourd'hui (lu) | Dans la story |
|---|---|---|---|---|---|
| `contacts/+page.svelte:992-1008` adresse du client (rue, n°, NPA, localité, pays) | 70, 16, 16, 35, 2 | cinq colonnes d'adresse | oui (débiteur de la QR) | **oui** : `address_input.rs:58-80` `check_lengths`, via `validate_optional` ; pays : « le pays doit être un code ISO à 2 lettres (ex. CH) » (`:75`) | **#1331** : retrait des cinq `maxlength` |
| `SupplierCreatePanel.svelte:210` pays du fournisseur neuf | 2 | idem | non | **oui** (même `check_lengths`) ; l'erreur s'affiche dans `supplier-new-error` (`:230-231`, `catch` `:138`) | **#1332** : retrait de `maxlength="2"` |
| `settings/+page.svelte:906` e-mail de la société | 320 | `companies.email VARCHAR(320)` (`20260709000003_companies_email.sql:9`) | oui (`invoice_pdf_service.rs:427` `creditor_email`) | **non** : `companies.rs:140-149` ne contrôle que `is_valid_email_simple` (`contacts.rs:266-280`), aucune longueur | **#1333** : borne serveur **ajoutée** (D2), puis retrait |
| `contacts/+page.svelte:872`, `:882` prénom, nom d'une personne | 70 | `contacts.first_name`, `last_name` `VARCHAR(70)` (`20260705000002_person_names.sql:15-16`) | oui (nom du débiteur) | **non** : `contacts.rs:377-400` `validate_common` ne borne que le **nom composé** à 255 (`:396`) | **#1334** : borne **ajoutée** (D3), puis retrait |
| `contacts/+page.svelte:1066` numéro de client | 50 | `client_number VARCHAR(50)` | oui (`invoice_pdf_service.rs:439` `debtor_client_number`) | **oui** : `contacts.rs:451` (`MAX_CLIENT_NUMBER_LEN`, 50) | retrait ; test `contact_client_number_e2e.rs:437` existant |
| `contacts/+page.svelte:1105` conditions de paiement | 100 | `default_payment_terms VARCHAR(100)` (`20260414000001_contacts.sql:15`) | oui (`onContactSelect` → `paymentTerms` → `invoice_pdf_service.rs:418`) | **oui** : `contacts.rs:429` (`MAX_PAYMENT_TERMS_LEN`, 100) | retrait ; test de route à écrire (filet) |
| `products/+page.svelte:637` nom de l'article | 255 | colonne | oui (`InvoiceForm.svelte:482` `description: p.name,`) | **oui** : `products.rs:187` (`MAX_NAME_LEN`) ; test unitaire `products.rs:564` existant | retrait |
| `contacts/+page.svelte:906` raison sociale (entreprise) | 255 | `contacts.name` (255) | oui (nom du débiteur) | **oui** : `contacts.rs:396-398` (`MAX_NAME_LEN`, 255) | **retrait** (V3-1) : une raison sociale **collée** de 300 caractères est coupée à 255 par le navigateur, le serveur accepte ces 255 et le toast annonce le succès : la coupure est silencieuse même quand le `maxlength` égale la borne ; `maxlength` compte des unités UTF-16, le serveur des caractères |
| `contacts/+page.svelte:1086` délai de paiement (jours) | 3 | `default_payment_terms_days` | oui (il fixe l'échéance pré-calculée, imprimée sur le PDF) | **oui** : `contacts.rs:439` (`MAX_PAYMENT_TERMS_DAYS`, 0 à 365) ; l'écran refuse aussi (`contacts/+page.svelte:459-465`, « entre 0 et 365 jours ») | **retrait** (V3-2) : « 1000 » collé devient « 100 », valide, enregistré et imprimé : un enregistrement faux et muet |

*Le `maxlength` n'est pas la borne : c'est ce qui rend la coupure silencieuse. Les cinq champs du PDF de la story sont : nom de l'article, numéro de client, conditions de paiement, raison sociale, délai de paiement.*

### #1334 — le prénom et le nom : deux sites d'entrée, aucune borne par champ

Recherche **par le symptôme** (toute écriture d'un prénom ou d'un nom dans une colonne `VARCHAR(70)`, de l'API à l'écran ; sortie copiée, `773df80da`) :

```
$ git grep -n "contacts::create(\|contacts::update(\|update_company_coordinates_in_tx(\|createContact(\|updateContact(" 773df80da -- 'crates/kesh-api/src/*' crates/kesh-db/src/backup.rs frontend/src/lib/features/supplier-invoices/SupplierCreatePanel.svelte 'frontend/src/routes/(app)/contacts/+page.svelte'
crates/kesh-api/src/routes/contacts.rs:698:    let contact = contacts::create(&state.pool, current_user.user_id, new)
crates/kesh-api/src/routes/contacts.rs:764:    let contact = contacts::update(&state.pool, id, req.version, current_user.user_id, changes)
crates/kesh-api/src/routes/onboarding.rs:560:        update_company_coordinates_in_tx(
crates/kesh-api/src/routes/onboarding.rs:1757:async fn update_company_coordinates_in_tx(
crates/kesh-db/src/backup.rs:2495:        let contact = contacts::create(
frontend/src/lib/features/supplier-invoices/SupplierCreatePanel.svelte:128:			const created = await createContact({
frontend/src/routes/(app)/contacts/+page.svelte:515:				await updateContact(editing.id, { ...payload, version: editing.version });
frontend/src/routes/(app)/contacts/+page.svelte:518:				await createContact(payload);
```

Tous les écrivains : `contacts.rs:698`, `:764` (API) ; `onboarding.rs:560` (assistant) ; `backup.rs:2495` (restauration : colonne de même taille) ; deux écrans. Les routes de `companies.rs` recopient l'état lu (`:155`, `:293`, `:375`, `:550`) ; le MCP lit (`mcp/outils/contacts.rs:149`, `:176`) ; les interlocuteurs sont bornés (`contact_persons.rs:93`). Inventaire complet.

- **`POST` et `PUT /api/v1/contacts`** (`validate_common`, `contacts.rs:377-400`) : le prénom et le nom ne sont bornés qu'**ensemble, à 255**, alors que les colonnes sont à **70** : un prénom de 100 et un nom de 100 passent le contrôle puis butent sur la base (`400 DATA_LENGTH_OR_RANGE`, champ non nommé). Le chemin est atteint **sans `maxlength`** par `SupplierCreatePanel.svelte` (aucun `maxlength` sur le prénom ni le nom : champs `:175` et `:179`, charge envoyée `:128-135`) et par l'API.
- **`POST /api/v1/onboarding/coordinates`** (`onboarding.rs:526-541` à `773df80da`) : quand **un seul** des deux est fourni, `name` vient de `body.name` (`:532-534`) et `first_name` seul est enregistré sans contrôle de longueur : `companies.first_name`/`last_name` `VARCHAR(70)` (`20260705000002_person_names.sql:11-12`). L'écran de l'assistant exige les deux (`onboarding/+page.svelte:194-196`) : défaut atteint par l'API seulement. Quand les **deux** sont fournis, le nom composé « prénom nom » est jugé par la 48-13 (D5 de la 48-13) à 70 ; chacun des deux reste **non borné séparément**.
- Contacts de société (`contact_persons.rs:93`) : borné à 70 ; **sans objet**. MCP (`mcp/outils/contacts.rs:149`) : lecture. Aucun import ne crée de contact.
- **Aide existante (DRY)** : `crates/kesh-api/src/helpers.rs:26` `pub fn validate_text_len(value: &str, max: usize, field: &str) -> Result<(), AppError>` rend `AppError::Validation("{field} : {max} caractères maximum")` — elle nomme le champ.

### #1333 — l'e-mail de la société

`companies.rs:140-149` `update_company_email` : `None | Some("")` efface ; sinon seul `is_valid_email_simple` juge. Le patron du téléphone et du site existe : `companies.rs:214-215` (`MAX_PHONE_LEN`, `MAX_WEBSITE_LEN`), `:251` `normalize_contact_field`, clé `error-company-phone-too-long` (`crates/kesh-i18n/locales/{fr-CH:2899, de-CH:2759, en-CH:2764, it-CH:2757}/messages.ftl`) ; test du téléphone et du site : `companies_e2e.rs:492` (`for (champ, valeur) in [("phone", "0".repeat(51)), ("website", "x".repeat(256))]`).
La borne de l'e-mail du **contact** est `contacts.rs:38` `MAX_EMAIL_LEN` = 320 (privée) ; la colonne de la société est aussi 320.

### Aucune donnée existante ne peut être refusée (V2-5)

Chaque borne neuve **égale la colonne** : e-mail de la société 320 (`VARCHAR(320)`) ; prénom et nom du contact 70 (`VARCHAR(70)`) ; prénom et nom de la société 70 (`VARCHAR(70)`). Une valeur déjà enregistrée tient dans sa colonne : **aucun contact ni aucune société existants ne peut être refusé à l'enregistrement**. Les refus de l'adresse, des conditions de
paiement, du numéro de client et du nom de l'article existent déjà : la story ne fait que retirer l'attribut qui les masquait. Ce que voit l'utilisateur d'un refus : le message du serveur dans `formError` et le toast (`contacts/+page.svelte:549-550`, `formError = err.message; notifyError(err.message);`), la fenêtre reste ouverte, la saisie reste.

### Manuels

- `docs/manual/fr/user-manual.tex:1353` (item « Adresse structurée » du contact) dit que le pays inconnu est « refusé à l'enregistrement, avec un message qui le nomme » mais **ne dit rien des longueurs** : **une phrase est ajoutée** (D5) — « rue 70, numéro 16, NPA 16, localité 35 caractères au plus ; un champ trop long est refusé, jamais coupé ». `user-manual.tex:1350` (« **Nom** : … **prénom et nom** pour une personne ») reçoit « prénom et nom : 70 caractères au plus chacun » (V3-13 : le refus remplace la coupure).
- L'e-mail de la société se dit au paragraphe des coordonnées de la société (`user-manual.tex`, section « Vos coordonnées sur la facture », `git grep -n -i "e-mail" -- docs/manual/fr/user-manual.tex` au développement) : une phrase (320 caractères au plus).
- L'API : `POST` et `PUT /api/v1/contacts` refusent un prénom ou un nom de plus de 70 caractères, `PUT /api/v1/companies/current/email` une adresse de plus de 320, `POST /api/v1/onboarding/coordinates` un prénom ou un nom de plus de 70 : phrase pour la section « Passer à la 0.20.0 » du manuel d'administration, **posée à l'intégration** (Note de version). `admin-manual.tex:2701` est une section datée : on n'y ajoute rien.

## Décisions

- **D1 — le périmètre et la règle** (décision de Guy du 2026-11-18, découpe de l'orchestrateur après la P2) : #1331, #1332, #1333, #1334 et les **cinq champs du PDF** (nom d'article, numéro de client, conditions de paiement, raison sociale, délai de paiement). Pour chaque `maxlength` retiré, refus serveur **vérifié et cité**, sinon **borne ajoutée avant** le retrait.
  **Option écartée** : laisser ces sites en issues (la décision de Guy est de réduire les bugs) ; **option écartée** : retirer un `maxlength` sans borne serveur (refus muet `DATA_LENGTH_OR_RANGE`).
- **D2 — #1333 : borne de 320 sur l'e-mail de la société.** `update_company_email` refuse, en `400 VALIDATION_ERROR`, une adresse de plus de **320 caractères** (comptés en caractères, après rognage, avant `is_valid_email_simple`), avec la clé `error-company-email-too-long` (**quatre catalogues**, patron `error-company-phone-too-long`, repli français « L'adresse e-mail de la société est trop longue (320 caractères au plus). ») ; `MAX_EMAIL_LEN` de `contacts.rs` devient `pub(crate)` et sert aux deux sites de la story (DRY entre eux ; `contact_persons.rs:25` `MAX_EMAIL` garde sa constante, tracée par #1335). Le message est **localisé** et dit le maximum, non le nombre saisi (comme le téléphone). Puis `maxlength={320}` retiré de `settings/+page.svelte:906`.
- **D3 — #1334 : 70 caractères, par champ, séparément.** Une constante partagée `PERSON_NAME_PART_MAX: usize = 70` (dans `helpers.rs`, à côté de `validate_text_len`), appliquée avec `validate_text_len(&prénom, PERSON_NAME_PART_MAX, "Prénom")` et `validate_text_len(&nom, PERSON_NAME_PART_MAX, "Nom")` : (a) dans `contacts.rs::validate_common`, branche `Personne`, **avant** le contrôle du nom composé (`:396`) ; (b) dans `onboarding.rs::set_coordinates`, sur chacun des deux **quand il est fourni**
  (avant le nom composé de la 48-13 et avant l'adresse). Refus `400 VALIDATION_ERROR`, message « Prénom : 70 caractères maximum » (nomme le champ). **Limites assumées** : le message ne dit pas le nombre saisi (helper existant) et il est **en français seulement** (V3-11 : `validate_text_len`, `helpers.rs:29-31`, et `validate_common` n'a « aucun i18n dans ce fichier », `contacts.rs:436-437` ; #710) — alors que le refus de l'e-mail (D2) est localisé. DRY : la constante `PERSON_NAME_PART_MAX` sert les deux sites de la story ; `contact_persons.rs:23` `MAX_NAME` garde la sienne, tracée par #1335. **Option écartée** : un message qui dit le nombre saisi (obligerait un helper neuf ; le champ nommé suffit, et le contrôle dit « maximum 70 »). **Option écartée** : borner le seul nom composé (c'est l'état actuel, qui laisse passer 100 + 100). Une **seule histoire** pour #1334 : les deux sites sont dans cette story (le site de l'assistant a été tranché ici plutôt que dans la 48-13 pour garder une seule fiche cohérente ; l'ordre de fusion — 48-13 d'abord — s'en trouve imposé, car les deux touchent `set_coordinates`).
- **D4 — la raison sociale (255) et le délai de paiement (3) perdent leur `maxlength`** (V3-1, V3-2 ; décisions de l'orchestrateur du 2026-11-18). Un `maxlength` **égal** à la borne serveur coupe quand même un texte **collé** : la raison sociale de 300 caractères devient 255, acceptée et annoncée enregistrée ; « 1000 » devient « 100 ». Le précédent (`settings/+page.svelte:346-351`, C-v0190c-25) retirait un `maxlength={70}` égal à la borne pour cette raison. Refus serveur vérifié : `contacts.rs:396-398` (255) et `:439` (0 à 365). **Option écartée** : les garder « parce qu'ils égalent la borne » (prémisse fausse).
- **D5 — le manuel** : quatre phrases (adresse du contact `:1353`, prénom et nom `:1350`, e-mail de la société, et l'API dans la Note de version). `.tex` seulement, pas de PDF.
- **D6 — E2E** : le défaut de #1331 se voit dans le navigateur (le champ coupe un texte collé) : **un E2E** dans la spec des contacts existante (`frontend/tests/e2e/contacts.spec.ts`, nom confirmé à l'étape 0) : un texte collé de 71 caractères dans la rue est refusé avec le message du serveur, le contact n'est pas modifié (relu par l'API), et la donnée de test est restaurée. #1334 côté assistant n'est atteint que par l'API : tests Rust seulement. #1332 : Vitest (le panneau) ; pas d'E2E, la spec du panneau (`supplier-invoice-*`) étant lourde à monter, dit ici.

## Critères d'acceptation

1. **AC1 — #1333, borne serveur de l'e-mail** (`crates/kesh-api/tests/companies_e2e.rs`, patron `:492`, `sqlx::test`) : `PUT /api/v1/companies/current/email` avec une adresse valide de **320** caractères → `200` ; de **321** → `400 VALIDATION_ERROR` (message de la clé `error-company-email-too-long`), **rien d'écrit**, `version` inchangée, aucune entrée d'audit ; la clé est présente dans les quatre catalogues (gardes de catalogue vertes). Rouge sans la borne (M1).
2. **AC2 — #1334, prénom et nom du contact** (nouveau fichier `contact_field_lengths_e2e.rs`, `sqlx::test`) : `POST /api/v1/contacts` (Personne) avec un **prénom de 71** caractères et un nom court → `400 VALIDATION_ERROR` qui nomme « Prénom », **sans `DATA_LENGTH_OR_RANGE`**, rien d'écrit ; idem **nom de 71** (message « Nom ») ; idem `PUT` ; **70 et 70 acceptés** (filet) ; 100 + 100 (composé 201 ≤ 255) refusé (c'est le cas que le contrôle du composé laissait passer). Rouge sans la borne : M2 (prénom), M3 (nom).
3. **AC3 — #1334, prénom ou nom seul à l'assistant** (`onboarding_path_b_e2e.rs`, patron `coordinates_rejects_country_name`) : `POST /onboarding/coordinates` avec **`firstName` seul de 71** caractères → `400 VALIDATION_ERROR` « Prénom », société inchangée, état à l'étape 5 ; idem `lastName` seul ; **un prénom seul de 70 et un nom seul de 70 acceptés** (filets ; prénom 70 + nom 70 donneraient un nom composé de 141 caractères, refusé par la 48-13 : ce cas n'est pas à écrire). Rouge sans le contrôle : M4 (prénom), M5 (nom).
4. **AC4 — #1331, adresse du contact** (`contacts-page` Vitest, nom confirmé à l'étape 0) : **sonde positive d'abord** — chaque champ est **trouvé** (`#form-street`, `#form-building`, `#form-npa`, `#form-city`, `#form-country`) avant son assertion d'absence de `maxlength` (une assertion d'absence sur un sélecteur vide passerait à vide) ; puis **aucun** des cinq n'a d'attribut `maxlength` (c'est cette assertion qui porte le rouge : dans jsdom un `maxlength` ne contraint pas une valeur posée par programme) ; une rue de 71 partie au serveur revient en refus affiché dans `formError`, **saisie conservée** ; raccourcie, elle s'enregistre. Rouge : **M6 à M10, une mutation par champ**. **Test Rust de la route** (filet, le refus existe) : `POST /contacts` avec rue de 71 → `400`, message « adresse : le champ rue dépasse 70 caractères (71) ».
5. **AC5 — #1331, E2E** (`contacts.spec.ts`, D6) : **NPA et localité renseignés** (sans quoi `addressPartial` refuse d'abord côté client, `contacts/+page.svelte:457-466`, et le message du serveur n'apparaît jamais) ; texte collé de 71 caractères dans la rue → refus avec le message du serveur (« rue », « 71 »), saisie conservée, contact inchangé en base ; restauration. Rouge : M6 rejouée en E2E (`e2e-m6.log`).
6. **AC6 — #1332, pays du fournisseur neuf** (`SupplierCreatePanel` Vitest) : sonde positive (`[data-testid="supplier-new-country"]` trouvé), puis pas d'attribut `maxlength` ; **NPA et localité renseignés** (un bloc d'adresse vide est ignoré : `address_input.rs:44-49` `is_blank` ne lit pas le pays, `validate_optional_keeping` rend `Ok(None)` ; un pays saisi seul est donc jeté sans refus — **#1342**, hors périmètre) ; « Suisse » part **entier** à `createContact` et le refus du serveur (« code ISO à 2 lettres ») s'affiche dans `supplier-new-error`, saisie conservée ; « CH » s'enregistre. Rouge : M11.
7. **AC7 — les cinq champs du PDF** (Vitest + Rust) : `maxlength` retiré du nom de l'article (`products/+page.svelte:637`, `#form-name`), du numéro de client (`#form-client-number`), des conditions de paiement (`#form-payment-terms`), de la raison sociale (`#form-name` du contact, `:902`) et du délai (`#form-payment-terms-days`) : **chacun trouvé d'abord** (sonde positive, **dans le bon état du formulaire** : type « Entreprise » pour `#form-name` de la raison sociale, `contacts/+page.svelte:902` ; type « Personne » pour `#form-firstname` et `#form-lastname`, `:868`, `:878`), puis assertion d'absence, **un `it` par champ, titre `« <champ> : aucun maxlength »`** (rouge : **M12 à M16, une mutation chacune**) ; refus serveur **cité par un test Rust nommé** pour chacun : article 256 (`products.rs:564`, existant), numéro de client 51 (`contact_client_number_e2e.rs:437`, existant), **conditions de paiement 101 et raison sociale 256** (à écrire dans `contact_field_lengths_e2e.rs`, **filets** : les refus existent, `contacts.rs:429`, `:396`) ; **délai de paiement : déjà testé** (`contact_payment_terms_e2e.rs:364` `contact_days_out_of_bounds_returns_400`, cas `[-1, 366]` ; on y ajoute seulement 1000, filet, ou on le cite). Écran : un délai de 4 chiffres collé revient refusé (« entre 0 et 365 jours », `:462-465`), non tronqué.
8. **AC8 — prénom, nom (écran) et e-mail** : sonde positive (`#form-firstname`, `#form-lastname`, `[data-testid="settings-company-email-input"]` trouvés), puis `maxlength` retirés (`contacts/+page.svelte:872`, `:882` ; rouge : **M17** prénom, **M18** nom, un journal chacune), le refus de la borne D3 s'affiche, saisie conservée (Vitest). `settings/+page.svelte:906` : `maxlength` de l'e-mail retiré ; la 48-13 dit seulement que ce `maxlength` « reste » (AC4 de la 48-13) et n'écrit aucun test de présence : **la 48-15 écrit l'assertion d'absence** (V3-6, V4-2) ; rouge : **M19** ; et le `400` de l'AC1 s'affiche.
9. **AC9 — aucune donnée existante n'est refusée** (V2-5) : chaque borne neuve égale sa colonne (cités plus haut) ; un contact ou une société déjà enregistrés se rouvrent et se réenregistrent comme avant (test Rust : une société avec e-mail de 320 et un contact avec prénom et nom de 70 se mettent à jour sans changement → `200`, ou no-op).
10. **AC10 — factures réelles** (contrainte de Guy du 2026-11-18) : aucune facture, aucun PDF, aucune donnée de la société n'est modifié : `git diff origin/epic/48-corrections-0-20-0...HEAD --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices` **vide** (trois points : base commune, pour que l'avance de la branche d'épique, par exemple la 48-11 sur `kesh-report/src/pdf.rs`, ne s'affiche pas à l'envers ; sortie copiée) ; les tests PDF existants passent sans modification ; le seul geste sur une donnée est un refus de plus long que la colonne, jamais atteignable par une donnée existante (AC9).
11. **AC11 — compteurs, gardes, manuel** : `sitesTotal` (`i18n-keys.test.ts`) recompté (aucune clé d'écran neuve ; la clé `error-company-email-too-long` est une clé de catalogue serveur) ; `lint-i18n-ownership` vert ; les quatre catalogues portent la clé neuve, même ordre ; `textes_coherents` vert ; manuel (D5) `.tex` seulement, `sed -n '1350p;1353p' docs/manual/fr/user-manual.tex` et `git grep -n "320 caract" -- docs/manual/fr/user-manual.tex` cités avant/après.
12. **AC12 — recette visuelle** (thème clair, base jetable, `kesh-gate-logs/e4815/captures/`) : (1) adresse d'un contact : texte collé de 71 caractères non coupé ; (2) son refus, saisie conservée ; (3) pays « Suisse » d'un fournisseur neuf, **NPA et localité renseignés** : refus explicite (le pays saisi seul : #1342, hors périmètre) ; (4) e-mail de la société de plus de 320 caractères : refus du serveur ; (5) prénom de 71 caractères d'un contact : refus qui nomme « Prénom » ; (6) délai de paiement de 4 chiffres collé : refus, non « 100 ».

## Tests à écrire, et la preuve du rouge

Ordre : **le test d'abord, rouge**, puis le correctif, vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4815/`. Mutations par l'outil **Edit** (jamais `git stash`) ; après restauration, `touch` du fichier.

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC1 (320 / 321) | la borne de l'e-mail | **M1** : retirer la borne ; `mut-m1.log` ; **sans la borne, MariaDB répond aussi 400** (`DATA_LENGTH_OR_RANGE`) : le test **lit `error.code`** et l'exige égal à `VALIDATION_ERROR` (le patron `companies_e2e.rs:492`, qui n'asserte que le statut, ne suffit pas) |
| AC2 (prénom 71 ; nom 71 ; 100 + 100) | la borne par champ de `validate_common` | **M2** : borne du prénom retirée ; **M3** : borne du nom retirée ; `mut-m2.log`, `mut-m3.log` (avant le correctif, le code rendu est `DATA_LENGTH_OR_RANGE`, non `VALIDATION_ERROR`) |
| AC3 (assistant, prénom ou nom seul) | la borne de `set_coordinates` | **M4** (prénom), **M5** (nom) ; `mut-m4.log`, `mut-m5.log` |
| AC4 (adresse du contact, absence de `maxlength`) | l'attribut | **M6** (rue), **M7** (numéro), **M8** (NPA), **M9** (localité), **M10** (pays) : `maxlength` remis, un journal chacune (`mut-m6.log` … `mut-m10.log`) ; la sonde positive empêche le passage à vide ; le test Rust de la route (rue 71) **ne rougit pas** : filet déclaré |
| AC5 (E2E) | idem | M6 rejouée en E2E (`e2e-m6.log`) |
| AC6 (pays du fournisseur) | l'attribut | **M11** ; `mut-m11.log` |
| AC7 (cinq champs du PDF) | les attributs | **M12** (article), **M13** (numéro de client), **M14** (conditions), **M15** (raison sociale), **M16** (délai) ; un journal chacune ; les tests Rust des conditions (101), de la raison sociale (256) et du délai (1000) **ne rougissent pas** : filets déclarés |
| AC8 (prénom, nom ; e-mail) | les attributs | **M17** (prénom), **M18** (nom), **M19** (e-mail) ; `mut-m17.log`, `mut-m18.log`, `mut-m19.log` |
| AC2 (70 + 70 acceptés), AC3 (70 acceptés), AC9, AC10 | **ne rougissent pas** (non-régression par construction) | déclarés filets |

Numérotation : **M1 à M19, uniques, un journal chacune** ; le rejeu en E2E d'une mutation (AC5) a son propre journal `e2e-m<N>.log` et ne prend pas de numéro neuf.

## Tâches

*Deux lots ordonnés (A serveur, B écrans), sous la dérogation à la règle de splitting.*

**Lot A — serveur** (`kesh-api`, `kesh-i18n`) :
1. **Étape 0** : relire les lignes citées à la tête de la branche (celles de ce document, y compris `onboarding.rs` après la fusion de la 48-13 : les numéros ont bougé) ; copier trois au Change Log ; confirmer les noms de fichiers de test (D6, AC2, AC4, AC7).
2. `helpers.rs` : `PERSON_NAME_PART_MAX` ; `contacts.rs` : `validate_common` (D3 a), `MAX_EMAIL_LEN` en `pub(crate)` ; `onboarding.rs` : `set_coordinates` (D3 b) ; tests AC2, AC3, AC9.
3. `companies.rs` : borne de l'e-mail (D2) ; clé `error-company-email-too-long` ×4 (`kesh-i18n`) ; test AC1.

**Lot B — écrans** (frontend) :
4. `settings/+page.svelte:906`, `contacts/+page.svelte` (adresse, prénom, nom, raison sociale, numéro de client, conditions, délai), `SupplierCreatePanel.svelte:210`, `products/+page.svelte:637` : retrait des `maxlength` (D1) ; tests AC4, AC6, AC7, AC8 ; E2E AC5. **Le lot B ne s'applique qu'après le lot A** (borne serveur avant retrait).
5. Manuel (D5), `.tex` seulement ; recomptes (AC11) ; captures (AC12) ; Dev Agent Record (mutations, gates, captures, grep de l'AC10). **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.**

## Gates (`e35-dev-commun.md` § *Quels gates, quand*)

- **En cours de développement** : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(companies_e2e) | binary(contact_field_lengths_e2e) | binary(contact_client_number_e2e) | binary(contact_address_country_e2e) | binary(contact_payment_terms_e2e) | binary(onboarding_path_b_e2e) | binary(textes_coherents) | (package(kesh-api) & kind(lib))'` ; `npm run check` ; Vitest de `contacts`, `settings`, `SupplierCreatePanel`, `products`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh`, frontend complet. **Pas d'E2E complet.** **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*, `KESH_COOKIE_SECURE=false` ; « vert » = les seuls échecs attendus, jugés fichier par fichier). `kesh-db` non touché (pas de migration, pas de repository, pas de `post_restore.rs`).
- Une commande lourde à la fois, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou `.gate.lock`.

## Note de version

**Corrigé**

- Contacts : l'adresse d'un client n'est plus coupée en silence quand on colle un texte trop long ; Kesh refuse et dit pourquoi, la saisie reste (refs #1331).
- Contacts : un prénom ou un nom de plus de 70 caractères n'est plus coupé à la saisie : il est refusé avec un message qui nomme le champ (par l'API et depuis une facture fournisseur, il revenait jusqu'ici en erreur générique qui ne le nommait pas) ; il en va de même, par l'API, pour un prénom ou un nom seul à l'assistant (refs #1334).
- Facture fournisseur : le pays d'un fournisseur créé depuis la facture n'est plus coupé à deux caractères (« Suisse » ne devient plus « Su ») ; avec un NPA et une localité, Kesh refuse et demande un code ISO à deux lettres (un pays saisi seul : #1342) (refs #1332).
- Paramètres → Organisation : l'e-mail de la société est borné à 320 caractères côté serveur, avec un refus explicite, et n'est plus coupé à la saisie (refs #1333).
- Contacts et articles : la raison sociale, le numéro de client, les conditions de paiement, le délai de paiement et le nom d'un article ne sont plus coupés en silence ; Kesh refuse et dit pourquoi (sans issue propre : champs qui finissent sur le PDF, admis dans la story par la décision de Guy du 2026-11-18, « diminuer le nombre de bugs »).

*Pour le manuel d'administration, section « Passer à la 0.20.0 » (posée à l'intégration)* : `POST` et `PUT /api/v1/contacts` refusent un prénom ou un nom de plus de 70 caractères, `PUT /api/v1/companies/current/email` une adresse de plus de 320, `POST /api/v1/onboarding/coordinates` un prénom ou un nom de plus de 70 ; aucune donnée existante n'est modifiée ni refusée.

## Dev Notes

- **Modules : sept au grain du domaine** (règle de décompte : une **crate** backend, un **domaine** frontend — `components/<d>`, `features/<d>`, `routes/(app)/<d>` —, ou **les manuels** ; les tests et les catalogues suivent leur module) : (1) `kesh-api` (`routes/contacts.rs`, `routes/companies.rs`, `routes/onboarding.rs`, `helpers.rs`), (2) `kesh-i18n` (une clé, quatre catalogues), (3) frontend `routes/(app)/settings`, (4) frontend `routes/(app)/contacts`, (5) frontend `routes/(app)/products`, (6) frontend `features/supplier-invoices`, (7) manuels.
  **Sept > cinq : voir la section `Dérogation règle de splitting` ci-dessous.**
- **Dépendances** : **la 48-13 se fusionne d'abord** (`onboarding.rs` : `set_coordinates` ; `settings/+page.svelte` : blocs distincts) ; la 48-15 **ne bloque pas** l'Epic 49 (aucun fichier de l'éditeur de lignes ; `InvoiceForm.svelte` n'est pas touché).
- **Fichiers communs** (plan de la release et fichier d'épique 48, à jour depuis `cd823aacb`) : `contacts/+page.svelte` — aucune autre story de la release ; **`products/+page.svelte` avec la 48-10 et `features/supplier-invoices/` avec la 48-12** (V3-5 : blocs distincts, le second arrivé résout) ; `settings/+page.svelte` avec la 48-13 (#1243, adresse/téléphone/site) et la v0200-menu-sous-groupes (blocs distincts) ; `i18n-keys.test.ts`, catalogues i18n, `sprint-status.yaml` : recomptés / résolus en gardant les deux côtés. Pas de PDF de manuel commité. Contributeur Beat (PR #1327, `kesh-report/src/pdf.rs`, `kesh-qrbill` `largeurs.rs`, `lib.rs`) : aucun de ces fichiers.
- **Hors périmètre — sites au même symptôme (un `maxlength` qui coupe, serveur qui sait ou doit refuser)** : issues **ouvertes par l'orchestrateur, une par écran** (choix de l'orchestrateur : les sites d'un même écran sont un même problème ; le `CLAUDE.md` demande une issue par problème) :
  1. `frontend/src/lib/features/contacts/ContactPersonsManager.svelte:141-145` : prénom et nom (70), fonction (100), e-mail (320), téléphone (50) d'un contact de société ; serveur : `contact_persons.rs:93` (prénom/nom 70) — **#1335**.
  2. `frontend/src/routes/(app)/contacts/+page.svelte:960` (e-mail, 320), et `:965` (téléphone, 50) ; serveur : `contacts.rs:408`, `:420` ; l'e-mail est l'adresse d'envoi de la facture (`invoice_email.rs`), la coupure exigerait plus de 320 caractères : effet nul — **#1336**.
  3. `frontend/src/routes/(app)/settings/projects/+page.svelte:199` (code, 32), `:203` (nom, 150) — **#1337** (refus serveur à vérifier).
  4. `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte:146` (120), `:196` (255) ; `ManualMatchModal.svelte:205` et `TransactionSplitModal.svelte:423` (`MAX_DESCRIPTION_LEN`, #1179 les touche déjà) — **#1338**.
  5. `frontend/src/lib/features/bank-import/BankProfileForm.svelte:139` (100), `:157` (200), `:170` (50) — **#1339**.
  6. Divers à bornes de colonne, sans effet PDF : `settings/api-keys/+page.svelte:320` (255), `products/+page.svelte:646` (description, 1000), `supplier-invoices/+page.svelte:284` (128), `DunningPauseDialog.svelte:81`, `settings/fiscal-years/+page.svelte:881` — **#1340**.
  7. `bank-accounts/+page.svelte:860`, `:1118` et `onboarding/+page.svelte:814` (description du compte bancaire, `BANK_ACCOUNT_DESCRIPTION_MAX`) : **#1005** (existante, P4).
- **Ce qui doit rester vrai** : un contact, une société, un article existants se rouvrent et se réenregistrent comme avant ; le refus de l'adresse dit le champ et le nombre ; un prénom de 70 caractères est accepté ; l'assistant n'agit qu'à l'étape 5.
- **Aucune donnée réelle** dans les tests, captures ni journaux : « Démo SA », adresses factices.
- Références : `48-13-avant-la-vraie-facture.md` (D4, D5, #1243), `v0190-nom-societe.md` (C-v0190c-25), `e0200-commun.md`, plan `release-0.20.0.md` et `epic-48-corrections-0-20-0.md` (branche `chore/kickoff-0.20.0`), `validation-p1.md`, `validation-p2.md` et `validation-48-15-p3.md` (`kesh-gate-logs/e4813/`).

## Dérogation règle de splitting

La story touche **sept modules au grain du domaine** (liste en Dev Notes), au-dessus de la borne de cinq du `CLAUDE.md`. **Source : décision de Guy (Project Lead) du 2026-11-18, option (a) de l'orchestrateur — « ok pour ta proposition (a) » : la story reste unique.** **Ce motif sort du critère du `CLAUDE.md`** (« si le découpage crée des cycles de dépendances ou des fusions intermédiaires intestables ») : il n'y a **ni cycle ni fusion intestable** — le découpage serait possible, la fiche le reconnaît — ; la dérogation repose sur le **coût** et sur la **nature du travail** : un seul geste mécanique répété sur chaque site (vérifier ou ajouter la borne serveur, puis retirer le `maxlength`), revu fichier par fichier, que deux stories (serveur, écrans) feraient payer deux fois en gates, revue et E2E. C'est une **décision du Project Lead**, non une application du critère.
**Garde-fou** : la story est rangée en **deux lots ordonnés** (lot A serveur, lot B écrans), le lot B ne s'appliquant qu'après le lot A (borne serveur avant retrait de l'attribut) ; chaque lot a ses commits et sa revue lisible séparément. La proposition de découpage 48-15a / 48-15b est **retirée**.

## Dev Agent Record

*Agent de développement : Opus 5.5, 2026-11-18 et 2026-11-19, en autonomie. Worktree `/home/gcorbaz/devel/kesh-48-15`, branche
`story/48-15-troncatures-silencieuses`, partie de `epic/48-corrections-0-20-0` = `45eb99ddd`. Clé `e4815`, port 3415. Journaux :
`/home/gcorbaz/devel/kesh-gate-logs/e4815/`. Registre : `e0200-choix-autonomes-e4815.md` (C-e4815-1 à 4).*

### Commits de code

| Commit | Contenu |
|---|---|
| `846746fbb` | lot A : `PERSON_NAME_PART_MAX` (`helpers.rs`), prénom et nom bornés dans `validate_common` et `set_coordinates`, e-mail de la société borné (clé ×4), `MAX_EMAIL_LEN` en `pub(crate)` ; tests AC1, AC2, AC3, AC4 (route), AC7 (route), AC9 |
| `865e54132` | lot B : retrait des `maxlength` (contacts : 11 champs — prénom, nom, raison sociale, rue, numéro, NPA, localité, pays, numéro de client, délai, conditions ; « 12 » écrit d'abord, corrigé à la revue P1, A4 ; fournisseur neuf : pays ; réglages : e-mail ; articles : nom) ; motif sous le champ (C-e4815-4) ; Vitest AC4, AC6, AC7, AC8 ; E2E AC5 ; manuel `.tex` ; registre |
| `81cd199fd` | lot B : retrait d'un doublon sur la page des articles (le motif s'y affichait déjà), vu à la recette visuelle ; registre corrigé |
| `867be3a79` | lot B : l'E2E de l'AC5 vise `data-testid="contact-form-submit"` et `contact-form-error` (ajoutés à la page des contacts), non des libellés : la garde KF-043 (`e2e-selecteurs-traduits.test.ts`) avait rougi au `test:unit` complet (`front-test-unit-review.log` : `contacts.spec.ts :: Enregistrer`, `contacts.spec.ts :: adresse : le champ rue dépasse 70 caractères (71)`) |

Le lot B n'est venu qu'après le lot A, vert et commité (borne serveur avant retrait de l'attribut, garde-fou de la dérogation).

### Rouges d'abord

- **Rust** (`rouge-rust.log`, commit `45eb99ddd` + tests seuls) : 12 tests, **7 rouges** — `company_email_is_bounded_at_320_by_the_route`,
  `a_71_character_first_name_is_refused_and_named`, `a_71_character_last_name_is_refused_and_named`,
  `a_100_plus_100_person_is_refused_by_the_route`, `put_refuses_a_71_character_first_or_last_name`,
  `coordinates_rejects_a_lone_71_character_first_name_1334`, `coordinates_rejects_a_lone_71_character_last_name_1334` — **tous** pour
  la raison attendue : `left: "DATA_LENGTH_OR_RANGE"`, `right: "VALIDATION_ERROR"`. Les 5 verts sont les filets déclarés (70 + 70,
  rue 71, champs du PDF, prénom seul 70, nom seul 70).
- **Vitest** (`rouge-vitest.log`, commit `846746fbb` + tests du lot B) : 150 tests, **17 rouges** — les quatorze « `<champ>` : aucun
  `maxlength` » (`expected true to be false`), et les trois motifs absents (délai, raison sociale, nom d'article ; ce dernier retiré
  ensuite, C-e4815-4). Verts dès avant : les parcours « part entier, refus affiché, saisie gardée » (rue 71, prénom 71, « Suisse »,
  e-mail 321) — **filets** : dans jsdom un `maxlength` ne contraint pas une valeur posée par programme ; c'est l'assertion d'absence
  qui porte le rouge, et l'E2E (AC5) qui le montre dans un navigateur.

### Mutations (M1 à M19, une par journal, `mut/`)

Code commité avant la première mutation ; motif vérifié **unique** avant remplacement ; restauration par `git restore --worktree`
puis `touch` ; « `== restauré` » dans chacun des 19 journaux. Scripts : `mut/mut-rust-4815.sh`, `mut/mut-front-4815.sh`,
`mut/e2e-m6.sh`.

| Mutation | Geste | Tuée par |
|---|---|---|
| M1 | borne de l'e-mail neutralisée (`if false`) | `company_email_is_bounded_at_320_by_the_route` (`DATA_LENGTH_OR_RANGE` lu) |
| M2 | borne du prénom retirée (`validate_common`) | `a_71_character_first_name_is_refused_and_named`, `a_100_plus_100_…`, `put_refuses_…` |
| M3 | borne du nom retirée (`validate_common`) | `a_71_character_last_name_is_refused_and_named`, `put_refuses_…` |
| M4 | borne du prénom retirée (`set_coordinates`) | `coordinates_rejects_a_lone_71_character_first_name_1334` |
| M5 | borne du nom retirée (`set_coordinates`) | `coordinates_rejects_a_lone_71_character_last_name_1334` |
| M6 à M10 | `maxlength` remis : rue (70), numéro (16), NPA (16), localité (35), pays (2) | « rue / numéro / NPA / localité / pays : aucun maxlength » (un `it` chacune, 1 rouge sur 51) |
| M11 | `maxlength="2"` remis, pays du fournisseur | « pays : aucun maxlength » (`SupplierCreatePanel.test.ts`) |
| M12 | `maxlength={255}` remis, nom d'article | « nom de l'article : aucun maxlength » ; rejouée au commit `81cd199fd` (`mut-m12.log`) |
| M13 à M16 | numéro de client (50), conditions (100), raison sociale (255), délai (3) | leur « … : aucun maxlength » |
| M17, M18 | prénom (70), nom (70) | « prénom : aucun maxlength », « nom de famille : aucun maxlength » |
| M19 | `maxlength={320}` remis, e-mail de la société | « e-mail de la société : aucun maxlength » (1 rouge sur 90) |
| M6 en E2E | `maxlength={70}` remis, frontend reconstruit | `e2e-m6.log.e2e` : « une rue de 71 caractères collée… » rouge, `Received` = 70 « R » (`page.fill` respecte l'attribut comme un collage) ; frontend reconstruit restauré. Joué au commit `865e54132`, **rejoué** au commit `867be3a79` (sélecteurs changés) : le journal est celui du rejeu, le premier a été écrasé (même rouge, même raison, relevé ici) |

Filets déclarés (ne rougissent sans aucune mutation de la story) : rue 71 par la route, conditions 101, raison sociale 256, délai
1000 (refus existants), 70 + 70, prénom ou nom seul de 70, e-mail de 320 réenregistré (AC9), AC10 ; et (revue P1, A5)
`products-page.test.ts` « un nom de 256 caractères collé est refusé en le disant… », non-régression du motif **existant** de la
page des articles — rouge au premier passage (`rouge-vitest.log`) parce qu'il cherchait un `data-testid` retiré depuis, il ne
rougit plus sous aucune mutation de la story.

### Tests ajoutés (recomptés depuis `git diff origin/epic/48-corrections-0-20-0...HEAD`)

- Rust, `#[sqlx::test]` : **12** — `companies_e2e.rs` 1, `onboarding_path_b_e2e.rs` 4, `contact_field_lengths_e2e.rs` (nouveau) 7.
- Vitest : **21 exécutés** — `contacts-page.test.ts` 7 `it(` écrits, dont 3 dans des boucles (5 + 4 + 2 champs) : 15 exécutés ;
  `products-page.test.ts` 2 ; `settings-page.test.ts` 2 (et le test AC4 de la 48-13, retitré, sans son assertion de présence :
  C-e4815-1) ; `SupplierCreatePanel.test.ts` 2.
- Playwright : **1** (`contacts.spec.ts`, AC5).

### Gates, tels qu'ils ont tourné

| Gate | Commit | Journal | Résultat |
|---|---|---|---|
| `cargo fmt --check` | `846746fbb` (avant commit) | `fmt-lotA.log` | vert |
| `cargo clippy --workspace --all-targets -D warnings` | lot A, avant commit | `clippy-lotA.log` | vert |
| nextest ciblé (binaires de la fiche + `kesh-i18n`) | lot A, avant commit | `cible-rust-lotA.log` | 852 / 852 |
| Vitest ciblé (4 fichiers) | lot B, avant commit | `vert-vitest-cible.log` | 150 / 150 |
| Vitest des articles | `81cd199fd` (avant commit) | `vert-vitest-products.log` | 6 / 6 |
| E2E ciblé `contacts.spec.ts` | `865e54132` | `e2e-contacts.log` | 9 verts, 2 ignorés (`test.skip` de longue date), port libéré |
| `scripts/test-fast.sh --ci` (base `kesh_e4815` remise à neuf) | `81cd199fd` | `test-fast-ci-review.log` | **6152 / 6152**, 14 ignorés |
| `scripts/gate-ci.sh kesh_ci_e4815` | `81cd199fd` | `gate-ci-review.log` | **6169 passés, 0 échoué**, 18 ignorés |
| frontend complet, première prise | `81cd199fd` | `front-{check,lint-i18n-ownership,test-unit,build}-review.log` | `check`, `lint`, `build` verts ; `test:unit` **1 rouge / 4646** (garde KF-043, corrigé par `867be3a79`) |
| Vitest de la garde et de la page des contacts | `867be3a79` (avant commit) | `vert-vitest-garde.log` | 54 / 54 |
| frontend complet, seconde prise | `867be3a79` | `front-{check,lint-i18n-ownership,test-unit,build}-review2.log` | `check` 0 erreur (29 avertissements, les mêmes qu'à la clôture de la 48-13, aucun dans un fichier de la story) ; `lint-i18n-ownership` vert ; `test:unit` **4646 / 4646** (278 fichiers) ; `build` vert |
| E2E ciblé `contacts.spec.ts`, seconde prise | `867be3a79` | `e2e-contacts2.log` | 9 verts, 2 ignorés, port libéré |
| `scripts/gate-ci.sh kesh_ci_e4815`, seconde prise | `867be3a79` | `gate-ci-review2.log` | **6169 passés, 0 échoué**, 18 ignorés |

`867be3a79` ne touche que l'écran des contacts (deux `data-testid`) et la spec E2E. `test-fast --ci` n'est **pas** rejoué à ce
commit (joué à `81cd199fd`) ; `gate-ci`, qui couvre les mêmes binaires un processus par binaire, l'est (des tests Rust lisent des
sources du frontend). **Non lancés** : l'E2E complet et le `gate-ci` de clôture (à la clôture, après la revue, `e35-dev-commun.md`).

### Recette visuelle (AC12)

Thème clair, base jetable `kesh_e2e_e4815`, données factices, `captures-4815.sh` / `captures-4815.mjs` (`captures.log`), frontend du
commit `81cd199fd` : `captures/01-contact-rue-71-non-coupee.png` (1), `02-contact-rue-refus-saisie-gardee.png` (2),
`03-fournisseur-pays-suisse-refuse.png` (3, NPA et localité renseignés), `04-societe-email-321-refuse.png` (4),
`05a-contact-prenom-71-saisie-gardee.png` et `05b-contact-prenom-71-refus-nomme.png` (5), `06-contact-delai-1000-refuse.png` (6),
`07-contact-raison-sociale-256-motif.png` et `08-article-nom-256-motif.png` (C-e4815-4 et son pendant existant). Le script vérifie,
avant chaque prise, que la valeur du champ est **entière** (`exiger`). `05-contact-prenom-71-refus-nomme.png` est une prise périmée
(le refus hors du cadre), remplacée par 05a et 05b, laissée faute de `rm`.

### AC10 — factures réelles

`git diff origin/epic/48-corrections-0-20-0...HEAD --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices`
: **sortie vide** (rc 0), relevé aux commits `81cd199fd` et `867be3a79` (`...867be3a79`, branche d'épique à `45eb99ddd`). Les tests PDF existants passent sans modification (`test-fast-ci-review.log`).
Le seul geste sur une donnée est un refus au-delà de la colonne : AC9.

### AC9 — aucune donnée existante refusée

Bornes neuves = colonnes, lues au schéma : `contacts.first_name`/`last_name` et `companies.first_name`/`last_name` `VARCHAR(70)`
(`20260705000002_person_names.sql:11-12`, `:15-16`), `companies.email` `VARCHAR(320)` (`20260709000003_companies_email.sql:9`). Les
valeurs enregistrées sont rognées (`clean`, `trim`), et le contrôle compte en caractères (`chars().count()`), comme `VARCHAR` en
`utf8mb4` : une valeur en base ne peut dépasser sa borne. Les routes qui recopient l'état de la société (`companies.rs`, adresse,
coordonnées, nom) ne passent pas par les bornes neuves. Tests : `a_70_plus_70_person_is_accepted_and_saves_again_unchanged` (PUT du
même contact → 200), `company_email_is_bounded_at_320_by_the_route` (e-mail de 320 réenregistré → 200),
`coordinates_accept_a_lone_70_character_{first,last}_name_1334`.

### AC11 — compteurs, gardes, manuel

- `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts:1153`, 3495) **inchangé** : aucune clé d'écran neuve, et les deux appels
  `i18nMsg` de `contacts/+page.svelte` (C-e4815-4) se sont déplacés, non multipliés ; vert au `test:unit` complet.
- Clé `error-company-email-too-long` aux quatre catalogues, juste après `error-company-website-too-long` (fr-CH, de-CH, it-CH,
  en-CH) ; gardes de catalogue vertes (`test-fast-ci-review.log`).
- Manuel, avant (`45eb99ddd`) : `sed -n '1350p;1353p'` rendait « \item \textbf{Nom} : raison sociale pour une entreprise ;
  \textbf{prénom et nom} pour une personne. » et l'item « Adresse structurée » sans longueur ; `git grep -n "320 caract"` : vide.
  Après : l'item « Nom » dit « Le prénom et le nom comptent 70 caractères au plus chacun~: plus longs, ils sont \textbf{refusés,
  jamais coupés}, et le message nomme le champ. » ; l'item « Adresse structurée » dit les quatre longueurs ; `git grep -n "320 caract"`
  rend la ligne 1516 (e-mail de la société). `.tex` seul ; PDF à la release.

### Constats hors périmètre (issues demandées à l'orchestrateur)

- **La clé `error-data-length-or-range` n'existe dans aucun catalogue** : `t()` rend alors la clé brute (`errors.rs:76-81`,
  `bundle.format(locale, key, None)`), si bien qu'en production le refus générique de MariaDB s'affiche « error-data-length-or-range »
  (vu dans `rouge-rust.log` : `{"error":{"code":"DATA_LENGTH_OR_RANGE","message":"error-data-length-or-range"}}` quand l'i18n est
  initialisée, comme en production). Ouverte par l'orchestrateur : **#1348**.
- #1342 (pays saisi seul jeté sans refus) : déjà ouverte, citée par la fiche.
- Revue P2, L1 : les états du bouton grisé encore sans motif visible sur le formulaire des contacts, **antérieurs à la story**
  (aucun n'est ouvert par elle) : **#1349**, ouverte par l'orchestrateur.
- Revue P2, L2 — **limite négligeable assumée** : le `trim()` de JavaScript et celui de Rust (`str::trim`, espaces blancs Unicode)
  ne couvrent pas exactement les mêmes caractères (U+FEFF est retiré par JavaScript et non par Rust ; U+0085 l'inverse). À la
  borne exacte, une valeur entourée de ces seuls caractères peut être comptée d'un caractère de différence entre l'écran et le
  serveur ; le serveur décide toujours, et son refus s'affiche. Sans issue.
- La description d'un article (`maxlength={1000}`) ne passe pas sur la facture (`InvoiceForm.svelte` ne recopie que `p.name`) :
  hors PDF, reste à #1340.

### Décalages relevés à l'étape 0 (fiche écrite sur `773df80da`, code lu à `45eb99ddd`)

- `onboarding.rs` : la lecture du prénom et du nom est à `:531-535` (`let (first_name, last_name) = (clean(&body.first_name),
  clean(&body.last_name));`), suivie du contrôle du nom composé de la 48-13 (`validate_company_name_for_qr`, `:543`).
- `settings/+page.svelte` : le `maxlength={320}` de l'e-mail est à `:911` (fiche : `:906`).
- `contacts.rs` : `validate_common` commence à `:356` ; la ligne citée `:396` (`if trimmed_name.chars().count() > MAX_NAME_LEN {`)
  est juste.
- **V4-2 démentie par le code** : la 48-13 **a écrit** une assertion de présence (`settings-page.test.ts:1573-1574`,
  `expect(getByTestId('settings-company-email-input').getAttribute('maxlength')).toBe('320');`) ; traitée par C-e4815-1.
- **Prémisse de l'AC7 (écran du délai) démentie** : le message « entre 0 et 365 jours » ne s'affichait pas, le bouton se désactivant
  avant toute soumission (`contacts/+page.svelte:1132`) ; traitée par C-e4815-4.

### Remédiation de la revue P1 (commit `3aaefa33c`)

Décisions de l'orchestrateur du 2026-11-19, toutes dans la story. Rapports : `kesh-gate-logs/e4815/review-p1-{B,E,A}.md`.

| Finding | Remède |
|---|---|
| B1 = E1 = A1 (MEDIUM) | la raison sociale ne se juge qu'en type « Entreprise » (`formContactType === 'Entreprise' && …`) ; amendement de C-e4815-4 ; test de la bascule |
| B2, E2 | longueur comptée en caractères (`[...formName.trim()].length`), contacts et articles ; tests avec 60 caractères hors BMP |
| B4, E3 | tests aux bornes multi-octets et entourés de blancs : `multibyte_and_padded_names_are_counted_in_characters_after_trimming` (contacts), `company_email_bound_counts_characters_after_trimming` (société), `coordinates_count_a_lone_first_name_in_characters_after_trimming_1334` (assistant) |
| B5, E5 | délai : chiffres seulement (`/^\d+$/` avant `Number`) ; tests « 0x1E », « 3e2 », « abc », « 30.0 », « -1 » |
| B3, E6 | l'E2E neuf archive son contact dans un `finally`, par l'API (`PUT /api/v1/contacts/{id}/archive`) |
| E4 | états du motif : ramené à 255, délai vidé, bascule, ouverture d'une fiche existante (et les cinq écritures du délai) |
| A2, A3 | Note de version corrigée (pays : refus avec NPA et localité, #1342 ; prénom et nom : coupés à l'écran, erreur générique seulement par l'API et le panneau fournisseur) |
| A4 | « 11 champs » (recompte du diff) |
| A5 | `products-page.test.ts` « un nom de 256… » déclaré filet |
| A6 | AC4 et AC8 lisent `[data-testid="contact-form-error"]`, non le `body` |

**Tests ajoutés** (recomptés depuis `git diff 867be3a79 3aaefa33c`) : Rust `#[sqlx::test]` **3** (un par fichier) ; Vitest
**11 exécutés** — `contacts-page.test.ts` 6 `it(` écrits, dont un dans une boucle de 5 écritures du délai, soit 10 exécutés ;
`products-page.test.ts` 1 — plus 2 tests existants (AC4, AC8) dont l'assertion change.

**Mutations neuves** (journaux `mut/mut-m20.log` … `mut-m25.log`, « restauré » dans chacun) :

| Mutation | Geste | Tuée par |
|---|---|---|
| M20 | `e.chars().count()` → `e.len()` (`update_company_email`) | `company_email_bound_counts_characters_after_trimming` |
| M21 | `value.chars().count()` → `value.len()` (`helpers.rs`, `exceeds_len`) | `multibyte_and_padded_names_are_counted_in_characters_after_trimming`, `coordinates_count_a_lone_first_name_in_characters_after_trimming_1334` |
| M22 | garde `formContactType === 'Entreprise' &&` retirée | « bascule vers « Personne » après une raison sociale de 256 … » |
| M23 | `[...formName.trim()].length` → `formName.trim().length` (contacts) | « raison sociale : compte en caractères, non en unités UTF-16 … » |
| M24 | idem (articles) | « le nom se compte en caractères, non en unités UTF-16 … » |
| M25 | `/^\d+$/` remplacé par l'ancien `Number.isInteger` | « délai « 0x1E » », « délai « 3e2 » », « délai « 30.0 » » (3 rouges) |

**Gate ciblé de la remédiation** (`e35-dev-commun.md` : pas de `gate-ci` ni d'E2E à la remédiation) :

| Gate | Journal | Résultat |
|---|---|---|
| Vitest (contacts, articles, garde KF-043, `i18n-keys`) | `remed-p1-vitest.log` | 82 / 82 |
| `npm run check` | `remed-p1-check.log` | 0 erreur, 29 avertissements (les mêmes) |
| `cargo fmt --check` | `remed-p1-fmt.log` | vert |
| `cargo clippy --workspace --all-targets -D warnings` | `remed-p1-clippy.log` | vert |
| nextest `companies_e2e`, `contact_field_lengths_e2e`, `onboarding_path_b_e2e` | `remed-p1-nextest.log` | 45 / 45 |

**Non lancés** : `test-fast --ci`, frontend complet, l'E2E (dont `contacts.spec.ts`, réécrit avec son `finally`) et `gate-ci` :
à la passe ciblée puis à la clôture.

## Change Log

- **2026-11-18 — création de la fiche, par découpe de la 48-13 après la validation P2** (Sonnet 5.5). Origine : la 48-13 avait été élargie le 2026-11-18 (décision de Guy) à ces sites ; la passe P2 (V2-2) a relevé neuf modules au grain du domaine ; l'orchestrateur a déclaré le signal de splitting au Project Lead et coupé la story. Reprend les remédiations de la P1 et de la P2 qui la concernent : V1-4 (trois champs du PDF), V2-1 (#1334, deux sites, constante partagée, mutations nommées), V2-3 (le défaut est un `400 DATA_LENGTH_OR_RANGE` générique, non un 500), V2-4 (sites hors périmètre listés, une issue par famille), V2-5 (bornes égales aux colonnes), V2-6 (M1 à M12 uniques), V2-7 (manuel `:1353`). Lu à `773df80da` : `contacts.rs:377-400`, `:429`, `:451` ; `companies.rs:140-149`, `:214-215`, `:251` ; `onboarding.rs:526-541` ; `address_input.rs:58-80` ; `helpers.rs:26` ; `errors.rs:6022` (kesh-api), `errors.rs:2602` (kesh-db) ; `SupplierCreatePanel.svelte:129-135`, `:210`.
  Modules : sept au grain du domaine (règle écrite en Dev Notes) ; découpage 48-15a / 48-15b proposé, non écrit.
- **2026-11-18 — dérogation à la règle de splitting** (décision de Guy, Project Lead, 2026-11-18) : la 48-15 reste une story unique, sept domaines, un même geste mécanique en deux lots ordonnés (A serveur, B écrans) ; section `Dérogation règle de splitting` écrite, proposition 48-15a / 48-15b retirée. Sites hors périmètre rattachés à leurs issues : ContactPersonsManager #1335, e-mail et téléphone des contacts #1336, projets #1337, rapprochement #1338, profils d'import bancaire #1339, divers #1340, comptes bancaires #1005 (existante).
- **2026-11-18 — validation, première passe sur cette fiche** (« P3 » du dossier, Opus 5.5 ; `kesh-gate-logs/e4813/validation-48-15-p3.md`) : 0 / 0 / **4 MEDIUM** / **11 LOW**, remédiés. V3-1 : la raison sociale perd son `maxlength` (D4 réécrit : un `maxlength` égal à la borne coupe quand même un texte collé). V3-2 : le délai de paiement (`:1086`) entre dans la story ; **les « trois champs du PDF » deviennent cinq**. V3-3 : sonde positive avant chaque assertion d'absence, une mutation par champ (M1 à M19), prénom et nom séparés, rejeux E2E à journal propre. V3-4 : kickoff à jour depuis `cd823aacb`, huitième story citée. V3-5 : fichiers communs avec la 48-10 et la 48-12. V3-6 : l'assertion de présence de la 48-13 est remplacée par l'assertion d'absence (AC8). V3-7 : `onboarding:814` à #1005. V3-8 : issues hors périmètre **une par écran**, choix de l'orchestrateur. V3-9 : recherche de #1334 partie du symptôme, sortie copiée, `:175`/`:179`. V3-10 : fichiers et binaires de test nommés (`contact_field_lengths_e2e.rs`). V3-11 : message de D3 en français seul, déclaré. V3-12 : une puce de Note de version par issue. V3-13 : manuel `:1350` (prénom et nom, 70 caractères). V3-14 : DRY limité aux deux sites de la story, `contact_persons.rs` tracé par #1335. V3-15 : la dérogation cite sa source et dit qu'elle sort du critère du `CLAUDE.md`.
- **2026-11-18 — validation terminée.** Lignée : fiche née de la 48-13 après sa P2 (découpe, signal de splitting déclaré au Project Lead) ; **P3 Opus 0/0/4/11 → remédiée → P4 Sonnet 0/0/0/9** (tendance des findings au-dessus de LOW : 4 MEDIUM puis aucun). Modèles : Opus, Sonnet. Dérogation à la règle de splitting : décision de Guy du 2026-11-18, motif hors critère du `CLAUDE.md`, écrit en section. **Les neuf LOW de la P4 (V4-1 à V4-9) sont corrigés après la dernière passe, dans le texte de la fiche seulement** : NPA et localité exigés aux AC5, AC6 et à la recette (le pays saisi seul : #1342, hors périmètre) ; assertion d'absence de l'e-mail écrite par la 48-15 ; `git diff` à trois points ; un `it` par champ et état du formulaire dits ; M1 lit `error.code` ; délai déjà testé (`contact_payment_terms_e2e.rs:364`) ; contrôle du manuel par `sed -n` ; ordre de lancement et ordre de sortie distingués ; prénom seul de 70 à l'AC3. Rapport : `kesh-gate-logs/e4813/validation-48-15-p4.md`. Statut : `ready-for-dev`.
- **2026-11-18 — étape 0 du développement** (Opus 5.5 ; code lu à `45eb99ddd`, la 48-13 fusionnée). Les neuf LOW de la P4 sont déjà
  dans le texte ; leurs prémisses relues au code. Trois lignes citées : `onboarding.rs:531` `let (first_name, last_name) =
  (clean(&body.first_name), clean(&body.last_name));` (fiche : `:526-541` à `773df80da`), suivie de
  `validate_company_name_for_qr` de la 48-13 ; `settings/+page.svelte:911` `maxlength={320}` (fiche : `:906`) ; `contacts.rs:396`
  `if trimmed_name.chars().count() > MAX_NAME_LEN {` (juste). Noms de fichiers confirmés : `frontend/tests/e2e/contacts.spec.ts` (D6),
  `contact_field_lengths_e2e.rs` (AC2, nouveau), `contacts-page.test.ts` (AC4), `products-page.test.ts`, `settings-page.test.ts`,
  `SupplierCreatePanel.test.ts`. **Deux prémisses démenties** : V4-2 (la 48-13 a bien écrit une assertion de présence,
  `settings-page.test.ts:1573-1574` ; C-e4815-1) ; AC7 « écran du délai » (le motif ne s'affichait pas, bouton désactivé avant toute
  soumission, `contacts/+page.svelte:1132` ; C-e4815-4). Aucune migration requise.
- **2026-11-19 — développement terminé, statut `review`** (Opus 5.5). Commits de code `846746fbb` (lot A), `865e54132` (lot B),
  `81cd199fd` (doublon de la page des articles retiré, vu à la recette), `867be3a79` (E2E sur `data-testid`, après un rouge de la garde
  KF-043 au `test:unit` complet). Rouges d'abord : 7 Rust, 17 Vitest ; mutations M1 à M19 tuées, M6 rejouée en E2E. Gates du passage
  en `review` : `test-fast --ci` 6152 / 6152 (`81cd199fd`), `gate-ci` 6169 / 0 (`81cd199fd`) puis rejoué à `867be3a79`, frontend
  complet vert à `867be3a79` (4646 / 4646) ; E2E ciblé `contacts.spec.ts` vert. **Pas d'E2E complet** (clôture). Choix C-e4815-1 à 4.
  Constat hors périmètre : `error-data-length-or-range` absente des catalogues (issue demandée). Détail : Dev Agent Record.
- **2026-11-19 — revue de code P1, trois lentilles en contexte frais (Opus ×3 : B, E, A)** : B 0/0/1/4, E 0/0/1/5, A 0/0/1/5 ;
  le MEDIUM est le même finding vu par les trois (B1 = E1 = A1) ; après dédoublonnage **0 / 0 / 3 / 14** (CRITICAL / HIGH / MEDIUM /
  LOW, décompte de l'orchestrateur). Tous traités dans la story (décisions de l'orchestrateur) au commit `3aaefa33c` : refus muet à
  la bascule Entreprise → Personne (C-e4815-4 amendé), comptage en caractères à l'écran, délai en chiffres, tests aux bornes
  multi-octets et entourés de blancs, E2E qui archive dans un `finally`, états du motif, Note de version et Dev Agent Record corrigés,
  AC4/AC8 sur `contact-form-error`, test des articles déclaré filet. Mutations neuves M20 à M25, toutes tuées. Gate ciblé vert
  (Vitest 82 / 82, `check`, `fmt`, `clippy`, nextest 45 / 45). Aucun reclassement. Suite : passe ciblée de l'orchestrateur sur
  `3aaefa33c`. Détail : Dev Agent Record, « Remédiation de la revue P1 ».
- **2026-11-19 — revue de code P2, passe complète en contexte frais (Sonnet ; `kesh-gate-logs/e4815/review-p2.md`, prompt
  `review-p2-prompt.md`)** : **0 / 0 / 0 / 2** — **boucle de revue terminée**. L1 (boutons grisés sans motif, antérieurs à la story)
  : **#1349** ; L2 (`trim` de JavaScript et de Rust) : limite négligeable assumée, écrite au Dev Agent Record. Constat de
  développement : **#1348** (clé `error-data-length-or-range` absente des catalogues). La description d'article ne passe pas sur la
  facture : #1340.
- **2026-11-19 — clôture, statut `done`** (Opus 5.5). **Change Log final de la boucle de revue** : P1 trois lentilles (Opus ×3 :
  B, E, A) **0/0/3/14** (un MEDIUM vu trois fois, dédoublonné par l'orchestrateur) → remédiation `3aaefa33c` (MEDIUM et quatorze LOW
  traités dans la story ; mutations M20 à M25) → P2 complète (Sonnet) **0/0/0/2**. Tendance au-dessus de LOW : 3 puis 0. Modèles
  alternés : Opus, puis Sonnet. Aucun reclassement. Gates de clôture au dernier commit de code `3aaefa33c` (HEAD `6349030f1`, qui
  ne porte en plus que la fiche ; bases `kesh_e4815`, `kesh_e2e_e4815` et `kesh_ci_e4815` remises à neuf ; enchaînement
  `cloture-4815.sh`, `cloture.log`) :

  | Gate | Journal | Résultat |
  |---|---|---|
  | `scripts/test-fast.sh --ci` | `test-fast-ci-cloture.log` | **6155 / 6155**, 14 ignorés |
  | `scripts/gate-ci.sh kesh_ci_e4815` | `gate-ci-cloture.log` | **6172 passés, 0 échoué**, 18 ignorés |
  | `npm run check` | `front-check-cloture.log` | 0 erreur, 29 avertissements (les mêmes, aucun dans un fichier de la story) |
  | `npm run lint-i18n-ownership` | `front-lint-i18n-ownership-cloture.log` | vert |
  | `npm run test:unit` | `front-test-unit-cloture.log` | **4657 / 4657** |
  | `npm run build` | `front-build-cloture.log` | vert |
  | **E2E complet** (`docs/testing.md`, `KESH_COOKIE_SECURE=false`, base `kesh_e2e_e4815` recréée, dossiers `/tmp/kesh-e2e-e4815/cloture/`) | `e2e-complet-cloture.log` (+ `.backend`) | **484 verts, 7 échecs, 18 ignorés** ; port 3415 libéré |

  **Les 7 échecs, jugés un par un** contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **exactement les sept de la KF-029 (#97)** ; aucun
  échec de pollution, aucun autre. Le test neuf (`contacts.spec.ts:197`, AC5) est vert. Le rouge connu
  `disque_sature_pendant_l_ecriture` (KF-063, #1261) ne s'est pas produit au `gate-ci` : pas de rejeu. Statut : **`done`**
  (`sprint-status.yaml` : `done`, posé par l'orchestrateur).
