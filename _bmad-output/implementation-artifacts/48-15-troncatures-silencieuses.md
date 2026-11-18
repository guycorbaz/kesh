# Story 48-15 : Troncatures silencieuses — adresse du client, pays du fournisseur, e-mail de la société, prénom et nom, trois champs du PDF (#1331, #1332, #1333, #1334)

Status: ready-for-dev

*Fiche rédigée le 2026-11-18 (Sonnet 5.5), **découpée de la 48-13 après la validation P2** (V2-2 : la 48-13, élargie à ces sites sur décision de Guy du 2026-11-18 « diminuer le nombre de bugs », touchait neuf modules au grain du domaine ;
l'orchestrateur a déclaré le signal de splitting au Project Lead et coupé la story). Worktree `kesh-48-13`, branche `story/48-13-avant-la-vraie-facture` pour l'instant (la branche `story/48-15-troncatures-silencieuses` se crée depuis
`epic/48-corrections-0-20-0` **après la fusion de la 48-13**, dont elle dépend). Base lue : `773df80da` (= `main` = v0.19.0). Release **0.20.0** (coupure le jeudi 2026-11-26 au soir). **Ne bloque pas l'Epic 49** : aucun fichier de l'éditeur de lignes.
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
| `contacts/+page.svelte:906` raison sociale | 255 | colonne | oui | **oui** : `contacts.rs:396-398` | **inchangé** : le `maxlength` égale la borne serveur, et la raison sociale d'une entreprise n'a pas de borne plus courte ; **hors famille** (non retiré) |

*Le `maxlength` de la raison sociale (255) n'est pas dans la story : il égale la colonne et la borne serveur ; le retirer n'apporterait que le message du serveur. Décision D4.*

### #1334 — le prénom et le nom : deux sites d'entrée, aucune borne par champ

Recherche par le symptôme (tous les points d'entrée qui reçoivent un prénom ou un nom seul) :

```
$ git grep -n "first_name" -- crates/kesh-api/src/routes
routes/contacts.rs:377  let (trimmed_name, first_name, last_name) = match contact_type {
routes/contacts.rs:378      ContactType::Personne => match (clean(first_name), clean(last_name)) {
routes/contacts.rs:396  if trimmed_name.chars().count() > MAX_NAME_LEN {          // 255, sur le nom COMPOSÉ seulement
routes/contacts.rs:656/724  req.first_name, req.last_name,                           // create_contact / update_contact
routes/onboarding.rs:531  let (first_name, last_name) = (clean(&body.first_name), clean(&body.last_name));
routes/contact_persons.rs:93  if first_name.chars().count() > MAX_NAME || last_name.chars().count() > MAX_NAME {   // borné (70)
```

- **`POST` et `PUT /api/v1/contacts`** (`validate_common`, `contacts.rs:377-400`) : le prénom et le nom ne sont bornés qu'**ensemble, à 255**, alors que les colonnes sont à **70** : un prénom de 100 et un nom de 100 passent le contrôle puis butent sur la base (`400 DATA_LENGTH_OR_RANGE`, champ non nommé). Le chemin est atteint **sans `maxlength`** par `SupplierCreatePanel.svelte` (aucun `maxlength` sur le prénom ni le nom, `:129-135`) et par l'API.
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

- `docs/manual/fr/user-manual.tex:1353` (item « Adresse structurée » du contact) dit que le pays inconnu est « refusé à l'enregistrement, avec un message qui le nomme » mais **ne dit rien des longueurs** : **une phrase est ajoutée** (D5) — « rue 70, numéro 16, NPA 16, localité 35 caractères au plus ; un champ trop long est refusé, jamais coupé ».
- L'e-mail de la société se dit au paragraphe des coordonnées de la société (`user-manual.tex`, section « Vos coordonnées sur la facture », `git grep -n -i "e-mail" -- docs/manual/fr/user-manual.tex` au développement) : une phrase (320 caractères au plus).
- L'API : `POST` et `PUT /api/v1/contacts` refusent un prénom ou un nom de plus de 70 caractères, `PUT /api/v1/companies/current/email` une adresse de plus de 320, `POST /api/v1/onboarding/coordinates` un prénom ou un nom de plus de 70 : phrase pour la section « Passer à la 0.20.0 » du manuel d'administration, **posée à l'intégration** (Note de version). `admin-manual.tex:2701` est une section datée : on n'y ajoute rien.

## Décisions

- **D1 — le périmètre et la règle** (décision de Guy du 2026-11-18, découpe de l'orchestrateur après la P2) : #1331, #1332, #1333, #1334 et les trois champs du PDF (nom d'article, numéro de client, conditions de paiement). Pour chaque `maxlength` retiré, refus serveur **vérifié et cité**, sinon **borne ajoutée avant** le retrait.
  **Option écartée** : laisser ces sites en issues (la décision de Guy est de réduire les bugs) ; **option écartée** : retirer un `maxlength` sans borne serveur (refus muet `DATA_LENGTH_OR_RANGE`).
- **D2 — #1333 : borne de 320 sur l'e-mail de la société.** `update_company_email` refuse, en `400 VALIDATION_ERROR`, une adresse de plus de **320 caractères** (comptés en caractères, après rognage, avant `is_valid_email_simple`), avec la clé `error-company-email-too-long` (**quatre catalogues**, patron `error-company-phone-too-long`, repli français « L'adresse e-mail de la société est trop longue (320 caractères au plus). ») ; `MAX_EMAIL_LEN` de `contacts.rs` devient `pub(crate)` et sert aux deux (DRY, même valeur). Le message est **localisé** et dit le maximum, non le nombre saisi (comme le téléphone). Puis `maxlength={320}` retiré de `settings/+page.svelte:906`.
- **D3 — #1334 : 70 caractères, par champ, séparément.** Une constante partagée `PERSON_NAME_PART_MAX: usize = 70` (dans `helpers.rs`, à côté de `validate_text_len`), appliquée avec `validate_text_len(&prénom, PERSON_NAME_PART_MAX, "Prénom")` et `validate_text_len(&nom, PERSON_NAME_PART_MAX, "Nom")` : (a) dans `contacts.rs::validate_common`, branche `Personne`, **avant** le contrôle du nom composé (`:396`) ; (b) dans `onboarding.rs::set_coordinates`, sur chacun des deux **quand il est fourni**
  (avant le nom composé de la 48-13 et avant l'adresse). Refus `400 VALIDATION_ERROR`, message « Prénom : 70 caractères maximum » (nomme le champ ; ne dit pas le nombre saisi : limite du helper existant, assumée). **Option écartée** : un message qui dit le nombre saisi (obligerait un helper neuf ; le champ nommé suffit, et le contrôle dit « maximum 70 »). **Option écartée** : borner le seul nom composé (c'est l'état actuel, qui laisse passer 100 + 100). Une **seule histoire** pour #1334 : les deux sites sont dans cette story (le site de l'assistant a été tranché ici plutôt que dans la 48-13 pour garder une seule fiche cohérente ; l'ordre de fusion — 48-13 d'abord — s'en trouve imposé, car les deux touchent `set_coordinates`).
- **D4 — la raison sociale (255) reste avec son `maxlength`**, qui égale la borne serveur (`contacts.rs:396`) : aucune coupure possible, aucun gain.
- **D5 — le manuel** : trois phrases (adresse du contact `:1353`, e-mail de la société, et l'API dans la Note de version). `.tex` seulement, pas de PDF.
- **D6 — E2E** : le défaut de #1331 se voit dans le navigateur (le champ coupe un texte collé) : **un E2E** dans la spec des contacts existante (`frontend/tests/e2e/contacts.spec.ts`, nom confirmé à l'étape 0) : un texte collé de 71 caractères dans la rue est refusé avec le message du serveur, le contact n'est pas modifié (relu par l'API), et la donnée de test est restaurée. #1334 côté assistant n'est atteint que par l'API : tests Rust seulement. #1332 : Vitest (le panneau) ; pas d'E2E, la spec du panneau (`supplier-invoice-*`) étant lourde à monter, dit ici.

## Critères d'acceptation

1. **AC1 — #1333, borne serveur de l'e-mail** (`crates/kesh-api/tests/companies_e2e.rs`, patron `:492`, `sqlx::test`) : `PUT /api/v1/companies/current/email` avec une adresse valide de **320** caractères → `200` ; de **321** → `400 VALIDATION_ERROR` (message de la clé `error-company-email-too-long`), **rien d'écrit**, `version` inchangée, aucune entrée d'audit ; la clé est présente dans les quatre catalogues (gardes de catalogue vertes). Rouge sans la borne (M1).
2. **AC2 — #1334, prénom et nom du contact** (nouveau fichier `contact_person_names_e2e.rs`, ou section de `contact_address_country_e2e.rs`, `sqlx::test`) : `POST /api/v1/contacts` (Personne) avec un **prénom de 71** caractères et un nom court → `400 VALIDATION_ERROR` qui nomme « Prénom », **sans `DATA_LENGTH_OR_RANGE`**, rien d'écrit ; idem **nom de 71** (message « Nom ») ; idem `PUT` ; **70 et 70 acceptés** (filet) ; 100 + 100 (composé 201 ≤ 255) refusé (c'est le cas que le contrôle du composé laissait passer). Rouge sans la borne : M2 (prénom), M3 (nom).
3. **AC3 — #1334, prénom ou nom seul à l'assistant** (`onboarding_path_b_e2e.rs`, patron `coordinates_rejects_country_name`) : `POST /onboarding/coordinates` avec **`firstName` seul de 71** caractères → `400 VALIDATION_ERROR` « Prénom », société inchangée, état à l'étape 5 ; idem `lastName` seul ; **70 acceptés** (filet). Rouge sans le contrôle : M4 (prénom), M5 (nom).
4. **AC4 — #1331, adresse du contact** (`contacts-page` Vitest, nom confirmé à l'étape 0) : **aucun** des cinq champs de l'adresse n'a d'attribut `maxlength` (c'est cette assertion qui porte le rouge : dans jsdom un `maxlength` ne contraint pas une valeur posée par programme) ; une rue de 71 partie au serveur revient en refus affiché dans `formError`, **saisie conservée** ; raccourcie, elle s'enregistre. Rouge : M6. **Test Rust de la route** (filet, le refus existe) : `POST /contacts` avec rue de 71 → `400`, message « adresse : le champ rue dépasse 70 caractères (71) ».
5. **AC5 — #1331, E2E** (`contacts.spec.ts`, D6) : texte collé de 71 caractères dans la rue → refus avec le message du serveur (« rue », « 71 »), saisie conservée, contact inchangé en base ; restauration. Rouge : M6 rejouée en E2E (`e2e-m6.log`).
6. **AC6 — #1332, pays du fournisseur neuf** (`SupplierCreatePanel` Vitest) : pas d'attribut `maxlength` sur le pays ; « Suisse » part **entier** à `createContact` et le refus du serveur (« code ISO à 2 lettres ») s'affiche dans `supplier-new-error`, saisie conservée ; « CH » s'enregistre. Rouge : M7.
7. **AC7 — les trois champs du PDF** (Vitest + Rust) : `maxlength` retiré du nom de l'article (`products/+page.svelte:637`), du numéro de client (`contacts/+page.svelte:1066`) et des conditions de paiement (`:1105`) : assertions d'absence (rouge : M8, M9, M10) ; refus serveur **cité par un test Rust nommé** pour chacun : article 256 (`products.rs:564`, existant), numéro de client 51 (`contact_client_number_e2e.rs:437`, existant), **conditions de paiement 101** (à écrire, **filet** : le refus existe, `contacts.rs:429`).
8. **AC8 — prénom et nom du contact (écran)** : `maxlength` retirés (`contacts/+page.svelte:872`, `:882`), le refus de la borne D3 s'affiche, saisie conservée (Vitest ; rouge : M11). `settings/+page.svelte:906` : `maxlength` de l'e-mail retiré (assertion d'absence ; rouge : M12), et le `400` de l'AC1 s'affiche.
9. **AC9 — aucune donnée existante n'est refusée** (V2-5) : chaque borne neuve égale sa colonne (cités plus haut) ; un contact ou une société déjà enregistrés se rouvrent et se réenregistrent comme avant (test Rust : une société avec e-mail de 320 et un contact avec prénom et nom de 70 se mettent à jour sans changement → `200`, ou no-op).
10. **AC10 — factures réelles** (contrainte de Guy du 2026-11-18) : aucune facture, aucun PDF, aucune donnée de la société n'est modifié : `git diff origin/epic/48-corrections-0-20-0..HEAD --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices` **vide** (sortie copiée) ; les tests PDF existants passent sans modification ; le seul geste sur une donnée est un refus de plus long que la colonne, jamais atteignable par une donnée existante (AC9).
11. **AC11 — compteurs, gardes, manuel** : `sitesTotal` (`i18n-keys.test.ts`) recompté (aucune clé d'écran neuve ; la clé `error-company-email-too-long` est une clé de catalogue serveur) ; `lint-i18n-ownership` vert ; les quatre catalogues portent la clé neuve, même ordre ; `textes_coherents` vert ; manuel (D5) `.tex` seulement, `git grep -n "1353\|320 caract" -- docs/manual/fr/user-manual.tex` cité avant/après.
12. **AC12 — recette visuelle** (thème clair, base jetable, `kesh-gate-logs/e4815/captures/`) : (1) adresse d'un contact : texte collé de 71 caractères non coupé ; (2) son refus, saisie conservée ; (3) pays « Suisse » d'un fournisseur neuf : refus explicite ; (4) e-mail de la société de plus de 320 caractères : refus du serveur ; (5) prénom de 71 caractères d'un contact : refus qui nomme « Prénom ».

## Tests à écrire, et la preuve du rouge

Ordre : **le test d'abord, rouge**, puis le correctif, vert. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e4815/`. Mutations par l'outil **Edit** (jamais `git stash`) ; après restauration, `touch` du fichier.

| Test | Rougit sans… | Preuve |
|---|---|---|
| AC1 (320 / 321) | la borne de l'e-mail | **M1** : retirer la borne ; `mut-m1.log` |
| AC2 (prénom 71 ; nom 71 ; 100 + 100) | la borne par champ de `validate_common` | **M2** : borne du prénom retirée ; **M3** : borne du nom retirée ; `mut-m2.log`, `mut-m3.log` (avant le correctif, le code rendu est `DATA_LENGTH_OR_RANGE`, non `VALIDATION_ERROR`) |
| AC3 (assistant, prénom ou nom seul) | la borne de `set_coordinates` | **M4** (prénom), **M5** (nom) ; `mut-m4.log`, `mut-m5.log` |
| AC4 (adresse du contact, absence de `maxlength`) | l'attribut | **M6** : `maxlength` remis sur la rue ; `mut-m6.log` ; le test Rust de la route (rue 71) **ne rougit pas** : filet déclaré |
| AC5 (E2E) | idem | M6 rejouée en E2E ; `e2e-m6.log` |
| AC6 (pays du fournisseur) | l'attribut | **M7** ; `mut-m7.log` |
| AC7 (article, numéro de client, conditions) | les attributs | **M8** (article), **M9** (numéro), **M10** (conditions) ; un journal chacune ; le test Rust des conditions (101) **ne rougit pas** : filet déclaré |
| AC8 (prénom, nom ; e-mail) | les attributs | **M11** (prénom et nom, un journal), **M12** (e-mail) ; `mut-m11.log`, `mut-m12.log` |
| AC2 (70 + 70 acceptés), AC3 (70 acceptés), AC9, AC10 | **ne rougissent pas** (non-régression par construction) | déclarés filets |

Numérotation : **M1 à M12, uniques, un journal chacune** (V2-6).

## Tâches

*Deux lots ordonnés (A serveur, B écrans), sous la dérogation à la règle de splitting.*

**Lot A — serveur** (`kesh-api`, `kesh-i18n`) :
1. **Étape 0** : relire les lignes citées à la tête de la branche (celles de ce document, y compris `onboarding.rs` après la fusion de la 48-13 : les numéros ont bougé) ; copier trois au Change Log ; confirmer les noms de fichiers de test (D6, AC2, AC4).
2. `helpers.rs` : `PERSON_NAME_PART_MAX` ; `contacts.rs` : `validate_common` (D3 a), `MAX_EMAIL_LEN` en `pub(crate)` ; `onboarding.rs` : `set_coordinates` (D3 b) ; tests AC2, AC3, AC9.
3. `companies.rs` : borne de l'e-mail (D2) ; clé `error-company-email-too-long` ×4 (`kesh-i18n`) ; test AC1.

**Lot B — écrans** (frontend) :
4. `settings/+page.svelte:906`, `contacts/+page.svelte` (adresse, prénom, nom, numéro de client, conditions), `SupplierCreatePanel.svelte:210`, `products/+page.svelte:637` : retrait des `maxlength` (D1) ; tests AC4, AC6, AC7, AC8 ; E2E AC5. **Le lot B ne s'applique qu'après le lot A** (borne serveur avant retrait).
5. Manuel (D5), `.tex` seulement ; recomptes (AC11) ; captures (AC12) ; Dev Agent Record (mutations, gates, captures, grep de l'AC10). **Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.**

## Gates (`e35-dev-commun.md` § *Quels gates, quand*)

- **En cours de développement** : `cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo nextest run -E 'binary(companies_e2e) | binary(contact_client_number_e2e) | binary(contact_address_country_e2e) | binary(onboarding_path_b_e2e) | binary(textes_coherents)'` ; `npm run check` ; Vitest de `contacts`, `settings`, `SupplierCreatePanel`, `products`. Le story file n'affirme que ce qui a tourné.
- **Au passage en `review`** : `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh`, frontend complet. **Pas d'E2E complet.** **À la clôture**, au dernier commit de code : `gate-ci` et **E2E complet** (`docs/testing.md` § *Prérequis Playwright local*, `KESH_COOKIE_SECURE=false` ; « vert » = les seuls échecs attendus, jugés fichier par fichier). `kesh-db` non touché (pas de migration, pas de repository, pas de `post_restore.rs`).
- Une commande lourde à la fois, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant, verrou `.gate.lock`.

## Note de version

**Corrigé**

- Contacts : l'adresse d'un client, le prénom et le nom, le numéro de client et les conditions de paiement ne sont plus coupés en silence quand on colle un texte trop long ; Kesh refuse et dit pourquoi, la saisie reste (refs #1331).
- Contacts : un prénom ou un nom de plus de 70 caractères est refusé en `400` qui nomme le champ, au lieu d'une erreur générique qui ne le nomme pas ; il en va de même, par l'API, pour un prénom ou un nom seul à l'assistant (refs #1334).
- Facture fournisseur : le pays d'un fournisseur créé depuis la facture n'est plus coupé à deux caractères (« Suisse » ne devient plus « Su ») ; Kesh refuse et demande un code ISO à deux lettres (refs #1332).
- Articles : le nom d'un article n'est plus coupé en silence.
- Paramètres → Organisation : l'e-mail de la société est borné à 320 caractères côté serveur, avec un refus explicite, et n'est plus coupé à la saisie (refs #1333).

*Pour le manuel d'administration, section « Passer à la 0.20.0 » (posée à l'intégration)* : `POST` et `PUT /api/v1/contacts` refusent un prénom ou un nom de plus de 70 caractères, `PUT /api/v1/companies/current/email` une adresse de plus de 320, `POST /api/v1/onboarding/coordinates` un prénom ou un nom de plus de 70 ; aucune donnée existante n'est modifiée ni refusée.

## Dev Notes

- **Modules : sept au grain du domaine** (règle de décompte : une **crate** backend, un **domaine** frontend — `components/<d>`, `features/<d>`, `routes/(app)/<d>` —, ou **les manuels** ; les tests et les catalogues suivent leur module) : (1) `kesh-api` (`routes/contacts.rs`, `routes/companies.rs`, `routes/onboarding.rs`, `helpers.rs`), (2) `kesh-i18n` (une clé, quatre catalogues), (3) frontend `routes/(app)/settings`, (4) frontend `routes/(app)/contacts`, (5) frontend `routes/(app)/products`, (6) frontend `features/supplier-invoices`, (7) manuels.
  **Sept > cinq : voir la section `Dérogation règle de splitting` ci-dessous.**
- **Dépendances** : **la 48-13 se fusionne d'abord** (`onboarding.rs` : `set_coordinates` ; `settings/+page.svelte` : blocs distincts) ; la 48-15 **ne bloque pas** l'Epic 49 (aucun fichier de l'éditeur de lignes ; `InvoiceForm.svelte` n'est pas touché).
- **Fichiers communs** (plan de la release et fichier d'épique 48) : `contacts/+page.svelte` — aucune autre story de la release ; `settings/+page.svelte` avec la 48-13 (#1243, adresse/téléphone/site) et la v0200-menu-sous-groupes (blocs distincts) ; `i18n-keys.test.ts`, catalogues i18n, `sprint-status.yaml` : recomptés / résolus en gardant les deux côtés. Pas de PDF de manuel commité. Contributeur Beat (PR #1327, `kesh-report/src/pdf.rs`, `kesh-qrbill` `largeurs.rs`, `lib.rs`) : aucun de ces fichiers.
- **Hors périmètre — sites au même symptôme (un `maxlength` qui coupe, serveur qui sait ou doit refuser)** : issues **ouvertes** (une issue par famille, P4) :
  1. `frontend/src/lib/features/contacts/ContactPersonsManager.svelte:141-145` : prénom et nom (70), fonction (100), e-mail (320), téléphone (50) d'un contact de société ; serveur : `contact_persons.rs:93` (prénom/nom 70) — **#1335**.
  2. `frontend/src/routes/(app)/contacts/+page.svelte:960` (e-mail, 320), `:965` (téléphone, 50) et `:1086` (délai de paiement, 3 chiffres) ; serveur : `contacts.rs:408`, `:420` ; l'e-mail est l'adresse d'envoi de la facture (`invoice_email.rs`), la coupure exigerait plus de 320 caractères : effet nul — **#1336**.
  3. `frontend/src/routes/(app)/settings/projects/+page.svelte:199` (code, 32), `:203` (nom, 150) — **#1337** (refus serveur à vérifier).
  4. `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte:146` (120), `:196` (255) ; `ManualMatchModal.svelte:205` et `TransactionSplitModal.svelte:423` (`MAX_DESCRIPTION_LEN`, #1179 les touche déjà) — **#1338**.
  5. `frontend/src/lib/features/bank-import/BankProfileForm.svelte:139` (100), `:157` (200), `:170` (50) — **#1339**.
  6. Divers à bornes de colonne, sans effet PDF : `settings/api-keys/+page.svelte:320` (255), `products/+page.svelte:646` (description, 1000), `supplier-invoices/+page.svelte:284` (128), `DunningPauseDialog.svelte:81`, `settings/fiscal-years/+page.svelte:881` — **#1340**.
  7. `bank-accounts/+page.svelte:860`, `:1118` : **#1005** (existante, P4).
- **Ce qui doit rester vrai** : un contact, une société, un article existants se rouvrent et se réenregistrent comme avant ; le refus de l'adresse dit le champ et le nombre ; un prénom de 70 caractères est accepté ; l'assistant n'agit qu'à l'étape 5.
- **Aucune donnée réelle** dans les tests, captures ni journaux : « Démo SA », adresses factices.
- Références : `48-13-avant-la-vraie-facture.md` (D4, D5, #1243), `v0190-nom-societe.md` (C-v0190c-25), `e0200-commun.md`, plan `release-0.20.0.md` et `epic-48-corrections-0-20-0.md` (branche `chore/kickoff-0.20.0`), `validation-p1.md` et `validation-p2.md` (`kesh-gate-logs/e4813/`).

## Dérogation règle de splitting

La story touche **sept modules au grain du domaine** (liste en Dev Notes), au-dessus de la borne de cinq du `CLAUDE.md`. **Décision du Project Lead (Guy) du 2026-11-18 : la story reste unique**, avec dérogation, pour ces raisons : elle ne porte **qu'un seul geste mécanique répété** sur chaque site — vérifier (ou ajouter) la borne serveur, puis retirer le `maxlength` —, revu fichier par fichier ; les sites ne se commandent entre eux que par un ordre simple, sans cycle de dépendances ni fusion intermédiaire intestable ; le découpage en deux stories (serveur, écrans) doublerait le coût de gates, de revue et d'E2E pour un même geste.
**Garde-fou** : la story est rangée en **deux lots ordonnés** (lot A serveur, lot B écrans), le lot B ne s'appliquant qu'après le lot A (borne serveur avant retrait de l'attribut) ; chaque lot a ses commits et sa revue lisible séparément. La proposition de découpage 48-15a / 48-15b est **retirée**.

## Dev Agent Record

*(vide — rempli au développement)*

## Change Log

- **2026-11-18 — création de la fiche, par découpe de la 48-13 après la validation P2** (Sonnet 5.5). Origine : la 48-13 avait été élargie le 2026-11-18 (décision de Guy) à ces sites ; la passe P2 (V2-2) a relevé neuf modules au grain du domaine ; l'orchestrateur a déclaré le signal de splitting au Project Lead et coupé la story. Reprend les remédiations de la P1 et de la P2 qui la concernent : V1-4 (trois champs du PDF), V2-1 (#1334, deux sites, constante partagée, mutations nommées), V2-3 (le défaut est un `400 DATA_LENGTH_OR_RANGE` générique, non un 500), V2-4 (sites hors périmètre listés, une issue par famille), V2-5 (bornes égales aux colonnes), V2-6 (M1 à M12 uniques), V2-7 (manuel `:1353`). Lu à `773df80da` : `contacts.rs:377-400`, `:429`, `:451` ; `companies.rs:140-149`, `:214-215`, `:251` ; `onboarding.rs:526-541` ; `address_input.rs:58-80` ; `helpers.rs:26` ; `errors.rs:6022` (kesh-api), `errors.rs:2602` (kesh-db) ; `SupplierCreatePanel.svelte:129-135`, `:210`.
  Modules : sept au grain du domaine (règle écrite en Dev Notes) ; découpage 48-15a / 48-15b proposé, non écrit.
- **2026-11-18 — dérogation à la règle de splitting** (décision de Guy, Project Lead, 2026-11-18) : la 48-15 reste une story unique, sept domaines, un même geste mécanique en deux lots ordonnés (A serveur, B écrans) ; section `Dérogation règle de splitting` écrite, proposition 48-15a / 48-15b retirée. Sites hors périmètre rattachés à leurs issues : ContactPersonsManager #1335, e-mail et téléphone des contacts #1336, projets #1337, rapprochement #1338, profils d'import bancaire #1339, divers #1340, comptes bancaires #1005 (existante).
