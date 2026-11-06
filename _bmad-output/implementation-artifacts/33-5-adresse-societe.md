# Story 33-5 : corriger l'adresse de la société après l'assistant (#909)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-5 à 7. Issue en `refs` dans les commits ; à la PR : `closes #909`.*

## Story

En tant qu'Administrateur, je veux corriger l'adresse de ma société dans les paramètres, avec le contrôle de pays de la 0.17.4,
pour qu'une société enregistrée avec un pays invalide puisse de nouveau produire ses QR-factures, sans repasser par l'assistant.

## Ce que le code dit (prémisses établies)

- **Aucune route** : `routes/companies.rs:1-3` et `lib.rs:446-454` n'offrent que `/companies/current/email` et `/contact-details` ;
  le doc-comment `companies.rs:233-235` le dit (« il n'existe aucune route générique d'update company »). À l'écran,
  `settings/+page.svelte:296-297` affiche l'adresse en lecture seule.
- **La QR-facture échoue bien** : `routes/invoice_pdf_service.rs:286-303` prend `company.structured_address().country` ;
  `kesh-qrbill/src/validation.rs:258-271` refuse un code hors ISO ; `map_qrbill_error` (`invoice_pdf_service.rs:495-497`) rend
  `InvoiceNotPdfReady` « Pays invalide ».
- **Un second cas que l'issue ne nomme pas** : une société créée avant le 2026-07-05 n'a que le texte libre `address`, les colonnes
  structurées vides (`repositories/companies.rs:876-889`) : `invoice-pdf-error-company-address-empty` (`invoice_pdf_service.rs:286-292`).
  La même route est son remède. À couvrir (AC-5).
- ⚠️ **La colonne `companies.country` (`fetch_country`, `:553`) n'est pas écrite par `update_in_tx`** (`companies.rs:903-910`), et ne sert
  que de repli quand `address_country` est vide. Elle reste telle quelle : aucune migration, et `validate_required` écrit toujours un
  pays (`crates/kesh-api/src/address_input.rs:37-42`).
- **Le contrôle existe** : `StructuredAddressInput::validate_required` (`crates/kesh-api/src/address_input.rs:100-108`), NPA et localité obligatoires, longueurs
  SIX, pays ISO connu, `check_lengths(None)`. L'assistant l'emploie (`onboarding.rs`, `body.address.validate_required()`).
- **Le patron de route** : `update_company_contact_details` (`companies.rs:240-312`) : `CompanyUpdate` reconstruit en full-replace,
  `update_in_tx`, audit `company.updated` borné aux champs touchés et écrit seulement si `version` a changé, routeur Admin (`lib.rs:453`).
- **Le patron d'avis** : contacts, `addressCountryKnown` (`routes/contacts.rs:213-216`), `contacts/+page.svelte:148-153,1010-1013`.
- ⚠️ **`CompanyJson` est un miroir écrit à la main** (`companies.rs:49-52`), et le type TypeScript ne déclare pas `addressStructured`
  (`settings.types.ts`, seul `address: string`), bien que le serveur le rende (`companies.rs:39`).

## Conception

- **Route** `PUT /api/v1/companies/current/address`, corps `{ address: StructuredAddressInput, version }`, routeur Admin, **validate_required**
  (C-e334-5 : un pays inconnu reste refusé même inchangé : c'est la correction elle-même, et le laisser passer laisserait la QR-facture en
  panne en annonçant un succès. Le contact, lui, garde son pays à l'identique parce que la QR y est facultative).
- `is_stub` ne bouge pas : le lever reste l'affaire de l'assistant (`companies.rs:233-235`).
- `CompanyJson.address_country_known: bool`, par `is_known_country`. Le champ se relit aussi pour une société sans structure : pays vide
  ⇒ `true` (`crates/kesh-api/src/address_input.rs:141-146`).
- **Écran** : dans *Paramètres → Organisation*, un bouton *Modifier* sur le patron e-mail/téléphone (Admin, 409 rechargé, message du
  serveur affiché), cinq champs ; le pays reste un champ libre de deux lettres, comme aux contacts. **Pas de sélecteur commun** : #910 reste
  ouverte (P4), son libellé en quatre langues n'est pas le moindre geste (C-e334-6). Un avis si `addressCountryKnown === false`.
- **Audit** : `company.updated`, `before`/`after` sur les cinq composants seulement (AC 4/8/9 de la 25-1b, sur le modèle de `:286-300`).
- **Une facture déjà émise ne change pas** : PDF figés, § `sec:pdf-fige` ; rien à faire.

## Acceptance Criteria

1. **AC-1** : `PUT …/address` valide → 200, `CompanyJson` à jour (`address`, `addressStructured`, `addressCountryKnown: true`), `version + 1`.
2. **AC-2** : pays `XX` ou `SU` → 400 qui nomme le code ; rien n'est écrit ; pays de plus de deux lettres → 400 ; NPA ou localité vides → 400 ;
   longueur SIX dépassée → 400.
3. **AC-3 — le défaut de l'issue, de bout en bout** : société dont `address_country` est mis à `SU` en SQL (état d'une base d'avant la
   0.17.4) : `GET` du PDF d'une facture validée **refusé** avec « Pays invalide » (**test rouge avant**, par l'absence de la route) ; après le
   `PUT` à `CH`, le même `GET` rend un PDF, et `addressCountryKnown` passe de `false` à `true`.
4. **AC-4** : version périmée → 409 `OPTIMISTIC_LOCK_CONFLICT`, sans écriture ni audit. Valeur identique → 200 sans audit (no-op KF-004).
5. **AC-5** : société aux seules colonnes de texte libre : la route renseigne les composants ; le PDF ne rend plus
   `invoice-pdf-error-company-address-empty`.
6. **AC-6 — droits** : Comptable, Consultation → 403 ; clé d'API → refus (`require_not_pat`) ; jamais d'autre société (`get_company_for`).
7. **AC-7 — full-replace** : nom, IDE, e-mail, téléphone, site, langues et `is_stub` sont inchangés après l'appel (patron
   `each_route_preserves_the_fields_it_does_not_touch`, `tests/companies_e2e.rs:280`) ; et les routes e-mail et coordonnées ne touchent pas l'adresse.
8. **AC-8 — audit** : une entrée `company.updated`, `before`/`after` limités aux cinq composants ; aucune entrée sur un refus.
9. **AC-9 — écran** : Admin voit *Modifier*, Comptable non ; préremplissage depuis `addressStructured` ; enregistrement, refus affiché, 409
   rechargé ; avis si le pays est inconnu (Vitest et `frontend/tests/e2e/company-contact-details.spec.ts`, élargie, ou spec voisine).
10. **AC-10** : `audit_route_registry.rs` à jour, toutes assertions recomptées : `LIB_ROUTES` **145** (`:962`) et tracées **137** (`:978`) ; `LIB_ROUTES.len() + TEST_ENDPOINT_ROUTES.len()` **148** (`:1015`) ; `sans_ecriture` **120** (`:1385`), la route portant `SansEcritureAuJournal` comme ses jumelles (`:397-398`) ; `tout.len()` **148** (`:1404`) ; commentaires d'en-tête `:27,33,35`. Valeurs de la 33-5 seule : la 33-7 fait bouger tracées et exemptées (recoupement).

## Tâches

- [ ] **T1 — rouge d'abord** : `crates/kesh-api/tests/company_address_e2e.rs`, AC-1 à 8 ; AC-3 rouge par le 404/405 de la route absente.
- [ ] **T2** : route, `CompanyJson`, enregistrement à `lib.rs` (routeur Admin) ; registre d'audit.
- [ ] **T3** : `settings.types.ts` (`addressStructured`, `addressCountryKnown`), `settings.api.ts` (`updateCompanyAddress`), écran, Vitest.
- [ ] **T4** : clés i18n `settings-company-address-*` dans les **quatre** locales (`crates/kesh-i18n/locales/*/messages.ftl`) ; réemployer
  `field-country` ; `npm run lint-i18n-ownership`.
- [ ] **T5** : E2E, captures thème clair (lecture, édition, refus de pays, avis) dans `kesh-gate-logs/e334/captures/`, base de démonstration jetable.
- [ ] **T6** : manuels, CHANGELOG *Ajouté* (`refs #909`, `refs #817`).

### Mutations nommées (P2)

- **M1** : `validate_required` remplacée par `validate_optional_keeping(Some(pays courant))` → rouge AC-2/AC-3 (`SU` conservé).
- **M2** : `email: company.email.clone()` remplacé par `None` → rouge AC-7. **M3** : audit écrit sans la condition de version → rouge AC-4.
- **M4** : la route posée sur le routeur des comptables → rouge AC-6. **M5** : `address_country_known` toujours `true` → rouge AC-3 et avis Vitest.

## Gates

`test-fast.sh`, `npm run check`, `lint-i18n-ownership`, `test:unit`, `build`, **E2E complet au dernier commit de code**, chacun par le double
`flock` et `mem-guard`. `crates/kesh-db` n'est pas touché : aucune écriture de dépôt neuve (`update_in_tx` existe).

## Manuels

`docs/manual/fr/user-manual.tex` : § `sec:coordonnees-emetteur` (`:1093`, où saisir l'adresse ; `:1097` n'y parle que de téléphone et site) et
`:213`, qui peut renvoyer à cette correction. `admin-manual.tex:3157` : le décompte (`136 + 5 + 3 = 144`) suit l'AC-10. **PDF régénérés** et
contrôlés aplatis : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`. Le manuel ne doit pas dire qu'on corrige un PDF déjà figé.

## Recoupements avec les autres fiches

`audit_route_registry.rs` et `admin-manual.tex:3157` avec **33-7** (les deux bougent les mêmes compteurs : recompter à l'intégration, ne pas
additionner de confiance) ; `settings/+page.svelte` avec **33-1b** s'il y pose le réglage (cette dernière vise plutôt `settings/invoicing`).

## Hors périmètre

#910 (sélecteur commun) ; modifier le nom, l'IDE ou le type : pas demandé ; la colonne morte `companies.country` : dette à signaler, aucun effet
aujourd'hui ; le nom d'une société au QR.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5), remédiation (Opus 5.5). Aucun finding au-dessus de LOW. **L-F5-1** : AC-10 nomme les cinq assertions et les commentaires (lus : `audit_route_registry.rs:962,978,1015,1385,1404`, le rapport disait « ~1384 »). **L-F5-2** : `crates/kesh-api/src/address_input.rs`, `:100-108`, `:141-146`, `:37` (lus). Aucun choix ajouté.
- 2026-11-06 : validation P2 (Opus 5.5) : aucun finding. Fiche inchangée, hors cette entrée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
