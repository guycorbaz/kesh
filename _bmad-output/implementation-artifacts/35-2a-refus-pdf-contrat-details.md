# Story 35-2a : le refus du PDF dit, dans un message localisé, quoi corriger — champ, caractère, adresse, pays (#955, #964)

Status: ready-for-dev

*Validation terminée (P1 à P7, 2026-11-12).*

*Fiche née le 2026-11-12 du découpage de la 35-2 à la validation P2 (C-e350-9), par l'agent de kickoff de l'Epic 35
(Opus 5.5), worktree `kesh-e35` ; **recadrée à la validation P3** (C-e350-11) : elle ne publie plus de contrat `details`
lisible par machine. Le lot de rappels est la **35-2b**. Release **0.17.7**, tête d'epic : à livrer avant la première
facture client de Guy (au plus tard le 2026-11-30). **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`).
Registres : `e35-choix-autonomes-e350.md` (C-e350-1, -3, -4, -8, -9, -11) ; `e35-choix-autonomes-e352a.md` pour la story,
à créer au premier choix. Le nom du fichier garde « contrat-details » pour ne pas casser les renvois ; son objet est le
message.*

## Story

En tant que titulaire qui émet sa première QR-facture, quand le PDF est refusé, je veux lire, dans le message que montre
l'écran, **quel champ** (nom, rue, numéro, NPA, localité, pays du client ou de l'entreprise, numéro de facture) et **quel
caractère** corriger, afin de corriger la bonne fiche sans chercher à l'aveugle.

## Ce qui se lit au code (`release/0.17.6`)

**Le serveur sait, l'écran tait.**

- `kesh-qrbill` refuse avec un champ nommé : `QrBillError::InvalidCharset { field, codepoint }` (le caractère n'y est
  porté que par son `codepoint: u32` ; il se retrouve par `char::from_u32`), `FieldEmpty(field)`,
  `FieldTooLong { field, max, got }` (`crates/kesh-qrbill/src/types.rs:363-409`). Noms de champ :
  `creditor.`/`debtor.` × `name`, `line1`, `line2`, `postal_code`, `town` (`validation.rs:373-407`), puis
  `unstructured_message` et `billing_information` (`:469-474`). **Onze sont atteignables** : `billing_information` vaut
  toujours `None` (`invoice_pdf_service.rs:409`) ; `unstructured_message` est tronqué à 140 caractères (`:404-407`), si bien
  que `TOO_LONG` n'y arrive pas. Les bras par défaut de `field_prefix_*` rendent cinq noms `address.*`, inatteignables.
- `validate_country` (`validation.rs:259`) refuse un pays sans nommer de partie (`InvalidCountry(code)`) ; Kesh connaît la
  partie (`invoice_pdf_service.rs:300-304`, `:318-322`).
- `map_qrbill_error` (`invoice_pdf_service.rs:493-542`) rend des phrases **en français en dur et des noms techniques** :
  `:499`, `:502`, `:508` (« Pays invalide: {c} »), `:510` (`err.to_string()`, « Champ debtor.name contient … U+2013 »),
  et le texte brut de `kesh-qrbill` pour l'IBAN, le QR-IBAN, la QRR, le montant, la devise et le payload.
- `AppError::InvoiceNotPdfReady(String)` → `build_response(…, "INVOICE_NOT_PDF_READY", &msg)` (`errors.rs:2073-2075`) ;
  **aucun `details`** aujourd'hui, ni sur la route ni au lot (`classify_render_error`, `invoice_email.rs:1023-1036`,
  `BatchItemError::failed`) : la story n'a donc aucun `details` existant à préserver.
- `:338` passe la **clé nue** `"qrbill-error-qrr-generation"`, absente de tout catalogue ; le commentaire `:334-335` dit à
  tort que `errors.rs` la résout. Ce cas est une **panne interne**, non corrigeable par l'utilisateur.
- Les replis Rust des refus d'adresse ne sont pas ceux du catalogue (`:289` contre fr-CH `messages.ftl:969`) ; aucun n'est
  dans **G9** (`crates/kesh-api/tests/textes_coherents.rs:734`, `TABLE` à 8 entrées). `invoice-pdf-error-client-address-empty`
  (`:971`) n'est employée nulle part. `credit_notes.rs:295-297` passe la clé de la **facture** avec un repli qui dit
  « l'avoir ». Le bloc `invoice-pdf-error-*` dit « entreprise » (`:968-969`).
- **État i18n global** : `errors::t`/`t_args` (`errors.rs:21-64`) rendent le repli tant que `init_error_i18n` n'a pas été
  appelée ; hors tests, seul `main.rs:340` l'appelle (définition : `errors.rs:26`). Pas de remise à `None`. Un test peut
  charger un **bundle local** sans y toucher : `I18nBundle::load(Path::new(env!("CARGO_MANIFEST_DIR"))…)`, précédents
  `audit_labels.rs:277-279` et `errors.rs:5307`.
- **Écran** : `pdfErrorMessage` (`pdf-error.ts:39-46`) remplace le message du serveur par une clé fixe ; un code de
  `SERVER_RESOLVED_CODES` (`:36`) rend `err.message` **sans condition** (`:43`). `parseErrorResponse` ne remplace qu'un
  `message` **absent** (`api-client.ts:283`) : une chaîne vide arrive vide.

**Routes et écrans qui montrent ce refus** :

| Route | Site du rendu | Écran | Aujourd'hui |
|---|---|---|---|
| `GET /invoices/{id}/pdf` | `invoice_pdf.rs:50` → `get_or_freeze` → `render_document` (`issued_invoice_pdf.rs:151`) | fiche facture, `pdfErrorMessage` (`invoices/[id]/+page.svelte:949`) | phrase fixe |
| `POST /invoices/{id}/pdf/refreeze` | `invoice_pdf.rs:88` → `render_document` (`issued_invoice_pdf.rs:256`) | `refreezeError = err.message` (`:911`) | message brut |
| envoi de la facture par e-mail | `invoice_email.rs:808` | branche `default`, `err.message` (`:864-866`) | message brut |
| rappel seul | `invoice_email.rs:560` | `sendError = err.message` (`invoices/reminders/+page.svelte:232`) | message brut |
| PDF d'un avoir | `credit_notes.rs:295`, `:363` | fiche avoir, `pdfErrorMessage` (`credit-notes/[id]/+page.svelte:106`) | phrase fixe |
| lot de rappels | `invoice_email.rs:1260-1269` | `ReminderBatchReport.svelte:30` | **35-2b** |

Les aperçus (`invoice_email.rs:262`, `:410`) ne rendent pas le PDF. L'avoir n'appelle pas `validate` : seuls le contact
absent, `TooManyLines` et `HeaderOverflow` en sortent. Rien d'autre, MCP compris.

**#964.** `build_qrbill_inputs` juge le NPA et la localité par `trim().is_empty()` : entreprise (`:287`) et client
(`:310`). U+00AD et U+200B ne sont pas retirés par `str::trim`. La règle est `kesh_qrbill::champ_vide_pour_qr`
(`validation.rs:244-246`), employée par la saisie de l'adresse de l'entreprise (`address_input.rs:110`), qui refuse
désormais un tel NPA à la saisie.

**Le modèle existe** : `address_input.rs:131-137` nomme champ et caractère (`escape_debug`, `U+XXXX`).

## Décisions

- **D1 — Le serveur résout le message, l'écran l'affiche.** `pdfErrorMessage` reprend `err.message` pour
  `INVOICE_NOT_PDF_READY` **s'il n'est pas vide après `trim()`**, sinon la clé fixe — un bras propre, distinct de
  `SERVER_RESOLVED_CODES`, qui garde son comportement pour `INVOICE_TOO_MANY_LINES_FOR_PDF` et `INVOICE_PDF_GONE` (V3-3).
- **D2 — Pas de contrat lisible par machine** (C-e350-11). Le refus ne gagne **pas** de `details.reason` ni de
  `details.field` : un contrat public, irréversible, ne se fixe pas sous l'échéance du 30 novembre. La proposition
  (C-e350-10) et les findings V2-4, V3-4, V3-5, V3-6 partent dans **#1045**, pour une epic ultérieure. Le code
  `INVOICE_NOT_PDF_READY` et le 400 ne changent pas.
- **D3 — Une fonction pure, une fonction de construction** (V3-1, V4-1, V4-2).
  - La fonction pure `refus_pdf(cause) -> (clé, arguments, repli)` (noms au choix) ne résout rien : elle rend la clé, ses
    arguments et le **repli déjà formaté** (variables nommées, D4), seul endroit où le caractère est échappé.
  - **Forme (a) : une clé par champ et par raison**, sans sélecteur Fluent ni libellé imbriqué : G9 compare un littéral par
    clé (`textes_coherents.rs:896-913`), ce qu'une phrase composée ou un sélecteur sur `$champ` ne permettraient pas.
  - **Une seule** fonction de construction, `refus(cause) -> AppError` (nom au choix), appelle `refus_pdf` puis `t_args` (ou
    `t` sans argument) et rend **`AppError::InvoiceNotPdfReady(String)`**, tel quel ; les treize constructions de
    production (AC3) passent toutes par elle : `map_qrbill_error`, les contrôles de `build_qrbill_inputs`, `render_document`
    (`:216`, `:226`) et `credit_notes.rs:295`. Aucune variante neuve : `classify_render_error` et le test
    `invoice_email.rs:1754` restent inchangés (V3-7).
- **D4 — Le caractère s'affiche par `escape_debug`**, lié à une variable nommée (`let car = c.escape_debug().to_string();`),
  suivi de son point de code passé à Fluent **en chaîne déjà formatée** (`let point = format!("U+{:04X}", c as u32);`) :
  un nombre serait formaté par Fluent en décimal et selon la locale (V4-6). G9 compare alors le repli
  (`textes_coherents.rs:896-913`).
- **D5 — Une phrase par cause, sur un ensemble clos** (V4-4), localisée, sans texte de `kesh-qrbill` (le détail technique
  au journal, `tracing::warn!`). Les **dix variantes** de `QrBillError` qui construisent `InvoiceNotPdfReady`
  (`invoice_pdf_service.rs:495-540`) et les causes propres à Kesh :

  | Cause | Phrase |
  |---|---|
  | `InvalidCharset { field, codepoint }` | le champ en mots, le caractère échappé, `U+XXXX` (une clé par champ) |
  | `FieldEmpty(field)` | le champ en mots est vide (une clé par champ ; NPA et localité : la phrase d'adresse incomplète de D6) |
  | `FieldTooLong { field, max, got }` | le champ en mots dépasse `max` caractères (une clé par champ) |
  | `InvalidIban` | IBAN du compte bancaire principal invalide |
  | `InvalidQrIban` | QR-IBAN du compte bancaire principal invalide |
  | `InvalidQrr` (`:497`), `InvalidPayload` (`:540`) et l'échec de `build_qrr` (`:338`) | panne interne, **non corrigeable** par l'utilisateur, « contactez l'administrateur » : une seule clé |
  | `InvalidAmount` | montant hors des bornes de la QR-facture |
  | `InvalidCurrency` | devise non admise |
  | `InvalidCountry` (bras défensif, `:507-509`) | pays au code invalide, sans partie (D7) |
  | Kesh : pays d'une partie (D7) | pays de l'entreprise / du client, avec le code |
  | Kesh : NPA ou localité vide (D6) | adresse incomplète, le champ nommé |
  | Kesh : client sans adresse (`:310`, bras `_`) | `invoice-pdf-error-client-address-required` (existante) |
  | Kesh : contact absent (`:216`) ; avoir (`credit_notes.rs:295`) | `invoice-pdf-error-contact-missing` (existante) ; `credit-note-pdf-error-contact-missing` (D8) |
  | Kesh : compte principal absent (`:226`) | `invoice-pdf-error-no-primary-bank` (existante) |

  Un nom de champ inconnu (les cinq `address.*`, un nom futur) passe par un **repli par raison** (trois clés génériques).
- **D6 — #964, adresse incomplète : le champ est nommé.** Le contrôle de Kesh (`:287`, `:310`) juge par
  `champ_vide_pour_qr` et rend, pour le premier champ vide, une phrase qui dit l'adresse incomplète **et** le champ :
  « Adresse du client incomplète : la localité est vide. Complétez la fiche du client (NPA et localité sont exigés par la
  QR-facture). » ; de même pour l'entreprise et le NPA. Un client sans aucune adresse garde
  `invoice-pdf-error-client-address-required`. Les clés `invoice-pdf-error-company-address-empty` et
  `invoice-pdf-error-client-address-empty`, sans usage après la story, sont retirées des quatre catalogues (vérifier par
  `git grep` qu'aucun test ni écran ne les lit).
- **D7 — Le pays** : `build_qrbill_inputs` contrôle le pays de chaque partie par `kesh_qrbill::is_iso_3166_alpha2`
  (`validation.rs:285`) avant `kesh-qrbill`, et rend la partie et le code : « Le pays de l'entreprise (« SU ») n'est pas un
  code ISO valide. ». Le bras `InvalidCountry` de `map_qrbill_error` (`:507-509`) reste, défensif : phrase sans partie, avec
  le code (V3-6 b). Le code est montré **tel que stocké** : une valeur **stockée** en minuscules (`ch`, posée en SQL — la saisie la met en
  majuscules, `address_input.rs:37-41`) est refusée par le PDF aujourd'hui ; elle reste refusée, avec cette phrase (V5-8,
  V6-1) ; la normaliser serait un changement de la saisie, hors de la story.
- **D8 — L'avoir a sa clé** : `credit-note-pdf-error-contact-missing` ×4, repli égal à fr-CH ; sa clé et son repli sont
  rendus par la fonction pure, qui vit dans `invoice_pdf_service.rs` : son entrée G9 est donc dans ce fichier (V5-1).
- **D9 — La communication** (V3-13) : un caractère refusé dans `unstructured_message` vient du **numéro de facture**, qu'une
  facture validée ne corrige pas : la phrase le dit (« le numéro de facture contient … : vérifiez le format de numérotation
  dans Administration → Facturation »).
- **D10 — L'état i18n global** : aucun test de `crates/kesh-api/src` n'appelle `init_error_i18n` ; les tests qui lisent le
  catalogue chargent un bundle local (D3, AC2) ou montent l'application (`crates/kesh-api/tests/`). Garde : AC13. Le nom de
  la fonction ne s'écrit pas non plus dans un commentaire de `src/` hors de ses deux sites : la garde compte toute
  occurrence (V4-11).
- **D11 — Un terme** (V3-15) : « entreprise », celui du bloc `invoice-pdf-error-*`.

## Critères d'acceptation

1. **AC1 — Le message nomme le champ et le caractère** (tests d'intégration, `crates/kesh-api/tests/invoice_pdf_e2e.rs`,
   catalogue chargé), sur `GET /invoices/{id}/pdf` :
   - client à la localité `Ve\u{2013}vey` : 400 `INVOICE_NOT_PDF_READY`, message fr-CH qui contient « localité du client »,
     « – » et « U+2013 » ;
   - numéro de bâtiment du client `1\n2` (posé en SQL) : le message contient `\n` écrit en deux caractères et `U+000A`
     (tue M7) ;
   - nom du client de **71** caractères (`NAME_MAX = 70`, `validation.rs:10`) : le message nomme « nom du client » et la
     limite de 70 ;
   - nom de l'entreprise `"\u{AD}"` (SQL) : « nom de l'entreprise » et « vide » ;
   - numéro de facture `F–1` (posé en SQL, `validate_template` ne contrôlant pas le jeu de caractères) : la phrase de D9 ;
   - pour chacun, le message ne contient ni `{`, ni `$`, ni « Champ », ni un nom technique (`creditor.`, `debtor.`,
     `address.`, `unstructured_message`, `billing_information`).
2. **AC2 — La table complète, par la fonction pure, sans état global** (V3-1). Un test de `invoice_pdf_service.rs`,
   `mod tests` : pour les onze champs atteignables × leurs raisons, `billing_information` et le bras de repli pour la
   complétude, et **l'ensemble clos de D5** (dix variantes et causes propres à Kesh), il asserte (a) la **clé, les
   arguments et le repli** rendus par la fonction pure ; (b) leur résolution par un **bundle chargé localement**
   (`I18nBundle::load` sur `crates/kesh-i18n/locales`, comme `audit_labels.rs:277`) **dans les quatre locales** : le rendu
   n'est pas la clé elle-même, hors fr-CH il n'est pas la valeur fr-CH (précédent
   `qr_message_prefix_is_translated_in_all_four_locales`, `invoice_pdf_service.rs:1082`, V4-7), et il ne contient ni `{`,
   ni `$`, ni « Champ », ni nom technique ; (c) **une table écrite à part dans le test** (V5-2), « champ → (partie, mot du
   champ) » — `creditor.*` → « entreprise », `debtor.*` → « client » ; `name` → « nom », `line1` → « rue », `line2` →
   « numéro », `postal_code` → « NPA », `town` → « localité » ; `unstructured_message` → « numéro de facture » — : le rendu
   **fr-CH** de chacune des 27 clés par champ contient le mot de la partie et le mot du champ ; en de-CH, it-CH et en-CH, au
   moins le mot de la partie (table par locale dans le test). Le repli se compare au rendu fr-CH de `bundle.format`, marques
   U+2068/U+2069 retirées, **jamais à un littéral** dans `mod tests` : un second littéral ferait rougir G9, qui compte les
   littéraux dans tout le fichier (V5-5) ; le caractère d'essai ne contient ni `{` ni `$`. Jamais `init_error_i18n`. Ce test
   tue M2 ; M9 se joue sur la fonction de construction unique (D3) et AC1 la tue. Le rendu de `bundle.format` garde les
   marques d'isolation que `t_args` retire (`errors.rs:44-55`) : les assertions de sous-chaîne n'enjambent pas un argument.
3. **AC3 — Inventaire clos** : seize lignes de `git grep -n 'InvoiceNotPdfReady' -- crates` hors `errors.rs`, soit
   **quatorze constructions** — treize en production (`credit_notes.rs:295` ; `invoice_pdf_service.rs:216, 226, 288, 324,
   338, 497, 499, 502, 505, 508, 510, 540`) et une de test (`invoice_email.rs:1754`) — et deux non-constructions
   (`invoice_pdf_service.rs:491`, doc-comment ; `invoice_email.rs:1030`, classement). Plus, dans `errors.rs`, le test
   `:5717`. Recomptées à l'étape 0 avec le tableau des routes ; verdict de chaque site au Change Log ; chaque construction
   de production passe par la fonction pure (D3).
4. **AC4 — #964 nommé** (D6 ; V4-5 : la phrase d'un NPA ou d'une localité vide est **la même** que le refus vienne du
   contrôle de Kesh ou de `kesh-qrbill`, si bien que M3 et M4 sont **équivalentes** pour le message ; ce qui tient #964,
   c'est cette phrase, qu'AC4 asserte, et `champ_vide_pour_qr` reste par cohérence avec la saisie) : entreprise au NPA `"\u{AD}"` (**posé en SQL** : la saisie le refuse depuis la 0.17.5,
   `address_input.rs:110`) → « adresse de l'entreprise incomplète » et « NPA » ; client à la localité `"\u{200B}"` (SQL)
   → « adresse du client incomplète » et « localité » — comparaisons **en minuscules** (la phrase commence par une
   majuscule, V5-4 ; de même pour « nom de l'entreprise » d'AC1) ; client sans adresse → la phrase de
   `invoice-pdf-error-client-address-required`.
5. **AC5 — Le pays** (D7) : entreprise au pays `SU` → « pays de l'entreprise » et « SU » ; contact au pays hors liste
   (SQL, cas d'avant la 0.17.4) → « pays du client ».
6. **AC6 — Les causes sans champ** (D5) : contact absent, compte principal absent, IBAN invalide, QR-IBAN invalide : une
   phrase chacune, sans texte de `kesh-qrbill` ; la QRR : la phrase « non corrigeable ».
7. **AC7 — L'écran** (D1) : `pdfErrorMessage` rend `err.message` pour `INVOICE_NOT_PDF_READY` (fiches facture et avoir)
   quand il est non vide, la clé fixe pour `message: ''` et `'   '` ; `INVOICE_TOO_MANY_LINES_FOR_PDF` et `INVOICE_PDF_GONE`
   gardent leur comportement.
8. **AC8 — Quatre langues et G9.** Clés ×4 (`parity_between_locales`), à côté du bloc `invoice-pdf-error-*` (C-e350-8) ;
   aucun sélecteur. **Décompte** (forme (a) de D3, à recompter à l'étape 0) :
   - par champ et par raison : caractère refusé × 11 champs atteignables, vide × 6 (`name`, `postal_code`, `town` des deux
     parties), trop long × 10 (champs d'adresse) = **27** ; repli par raison pour un nom inconnu : **3** ;
   - causes sans champ : IBAN, QR-IBAN, panne interne, montant, devise, pays sans partie, pays de l'entreprise, pays du
     client, contact absent de l'avoir = **9** ;
   - soit **39 clés neuves** ; trois existantes reprises (`invoice-pdf-error-client-address-required`,
     `-contact-missing`, `-no-primary-bank`) ; deux retirées (`-company-address-empty`, `-client-address-empty`, D6).

   G9 : **une entrée neuve en fin de table**, `crates/kesh-api/src/routes/invoice_pdf_service.rs`, qui porte les 39 clés
   neuves (celle de l'avoir comprise, D8) et les trois reprises : **42** clés ; aucune entrée pour `credit_notes.rs`, qui
   ne porte plus de repli ; la table passe de 8 à **9** entrées (V5-1). Les replis des trois clés reprises sont **réécrits
   égaux à leur valeur fr-CH** (`:228` dit « cette company », `:326` « obligatoire et complète (NPA et localité) », V5-6, V6-3).
   `max` et `got` passent à Fluent **en chaînes** (comme le point de code, D4), pour ne pas être formatés selon la locale
   (V5-8). Paramètres : `("{ $car }", "{car}")`, `("{ $point }", "{point}")` pour un caractère ;
   `("{ $max }", "{max}")`, `("{ $got }", "{got}")` pour une longueur ; `("{ $code }", "{code}")` pour un pays. Aucune clé
   dans `errors.rs` (D3) : le bloc `errors.rs` de G9 n'est pas touché.
9. **AC9 — Tests existants réécrits sans perdre leur preuve** : `company_address_e2e.rs:583-589` (« Pays invalide » →
   AC5) et `:667-675` (« adresse de l'entreprise est vide » → AC4) ; `pdf-error.test.ts:35-38` (sa
   preuve passe à `INVOICE_NOT_VALIDATED`) ; le commentaire de `frontend/tests/e2e/invoices.spec.ts:723-755` ; le
   commentaire de `company_address_e2e.rs:375-376` au présent (LOW de #964).
10. **AC10 — Manuel, CHANGELOG, API.** `user-manual.tex:1251` réécrit (le message nomme le champ et le caractère ; la
    phrase « Cherchez ces caractères dans le nom et l'adresse du client et de la société » nomme aussi le **numéro de
    facture**, D9 ; la 35-2b complète la phrase du lot) ; PDF régénéré, contrôlé aplati — le PDF porte des ligatures
    (`pdftotext` rend « aﬀiche ») : greper des mots sans `ff`/`fi`/`fl`, ou normaliser (V4-12). CHANGELOG `## [0.17.7] — Non publié`, *Corrigé*,
    un point pour #964 et un pour la part #955 de cette story (C-e350-8), qui dit que **le texte du message a changé**
    (V3-14). `docs/api-external.md` : ligne neuve `INVOICE_NOT_PDF_READY` juste après
    `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` (`:1058`) : le 400, les routes, un message **localisé, destiné à
    l'utilisateur**, qui a changé à la 0.17.7 et ne se lit pas par machine ; pas de `details` (contrat à venir, issue).
11. **AC11 — Les autres routes** (V3-8) : un test HTTP par route, qui asserte que le message nomme le champ (client à la
    localité `Ve–vey`) : envoi par e-mail (`invoice_send_email_e2e.rs`), rappel seul (même fichier), refigeage
    (`invoice_frozen_pdf_e2e.rs` ou voisin) ; l'avoir : contact absent si le montage le permet, sinon angle mort écrit.
    **Montages** (V4-8) : `get_or_freeze` sert un fichier figé sans rendre — le téléchargement et l'envoi exigent une
    facture **jamais figée** ; le refigeage exige un PDF figé **dont le fichier manque** (`issued_invoice_pdf.rs:240-249` :
    sinon `InvoicePdfNotFrozen` ou `InvoicePdfPresent`), pour que le test rougisse pour la bonne raison.
12. **AC12 — E2E et captures.** Dans `invoice-qr-caracteres.spec.ts`, client à la localité `Ve–vey`, facture validée,
    *Télécharger le PDF* : le message affiché contient « localité du client » et « U+2013 » ; `createContactWithAddressViaApi`
    (`helpers/api-fixtures.ts:73-79`) reçoit un paramètre facultatif `city`, en dernier. Captures (thème clair, base
    jetable, `kesh-gate-logs/e352a/captures/`) : téléchargement refusé (caractère, adresse incomplète, pays) ; modale
    d'envoi par e-mail ; rappel seul.
13. **AC13 — Garde de l'état global** (D10, V3-2) : un test de `crates/kesh-api/tests/textes_coherents.rs` parcourt
    `crates/kesh-api/src/**/*.rs` et exige **exactement deux** occurrences de `init_error_i18n` : la ligne
    `pub fn init_error_i18n(` d'`errors.rs` et l'appel de `main.rs` ; toute autre occurrence, `mod tests` compris, rougit.
    Mutation M12 jouée **dans le `mod tests` d'`errors.rs`**. Le garde compte toute occurrence du nom, alias
    (`use … init_error_i18n as …`) compris.
14. **AC14 — Garde des constructions** (V5-7) : un test de `textes_coherents.rs`, sur le modèle d'AC13, exige dans
    `crates/kesh-api/src` hors `errors.rs` et hors modules `#[cfg(test)]` **exactement deux** occurrences de
    `InvoiceNotPdfReady(` : la fonction de construction (D3) et le motif de `classify_render_error`
    (`invoice_email.rs:1030`) ; le doc-comment `:491` ne porte pas le motif (un backtick suit le nom) : aucune
    reformulation requise (V6-4). Sonde positive au Change Log (M14).

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, garder le journal (`kesh-gate-logs/e352a/rouges.log`) : AC1, AC4, AC5, AC6
(`invoice_pdf_e2e.rs`, `company_address_e2e.rs`) ; AC2 (`mod tests`, bundle local) ; AC7 et AC9 (`pdf-error.test.ts`) ;
AC11 ; AC13 (vert au départ : sa sonde M12 tient lieu de rouge) ; AC14 (rouge au départ : 14 occurrences hors tests,
dont `invoice_email.rs:1030`, V6-2) ; AC12. Un test vert sous `nextest` doit l'être sous
`cargo test` : `scripts/gate-ci.sh` au dernier commit de code.

**Mutations** : M1 le bras `INVOICE_NOT_PDF_READY` de `pdfErrorMessage` retiré ; M2 deux clés de champ échangées dans la
fonction pure, et un mot de partie changé dans une phrase du catalogue (AC2 c) ; M3 `trim().is_empty()` rétabli à `:287` et
M4 idem à `:310` : **équivalentes, non jouées** (AC4 : #964 est tenu par la phrase, `champ_vide_pour_qr` est de
cohérence, V5-3) ; M7 `escape_debug` retiré ; M9 `t` au lieu de `t_args` ; M11 le
contrôle du pays retiré ; M12 un appel d'`init_error_i18n` dans le `mod tests` d'`errors.rs` ; M13 la clé de la QRR
retirée du catalogue (AC2) ; M14 une construction `InvoiceNotPdfReady(…)` ajoutée hors de la fonction de construction
(AC14).

## Tâches

- [ ] Étape 0, **sur `release/0.17.6`** (les lignes citées sont celles de cette branche ; le worktree du kickoff est sur
      `main`) : recompter l'inventaire, le tableau des routes et les lignes citées ; Change Log.
- [ ] Tests, rouges consignés.
- [ ] Serveur : fonction pure, phrases ×4, adresse (D6), pays (D7), avoir (D8), communication (D9), `:338`, G9 ; gardes AC13 et AC14.
- [ ] Écran : `pdf-error.ts` (D1).
- [ ] Tests existants (AC9) ; manuel, CHANGELOG, `api-external.md` ; captures ; E2E.
- [ ] Gates : `test-fast.sh --ci` et `scripts/gate-ci.sh`, frontend complet, E2E complet au dernier commit de code, échecs
      nommés contre `docs/testing.md`.

## Dev Notes

### Hors périmètre

- **Le contrat lisible par machine** (`details.reason`, `details.field`) : C-e350-11, **#1045**.
- **Le jeu de caractères du format de numérotation** (`validate_template` ne le contrôle pas) : **#1046** (V3-13).
- **Le lot de rappels** : 35-2b. **#944** : C-e350-1. **#942**, **#952** : ce qui est admis ne change pas.

### Fermeture des issues (V3-11)

La PR de la 35-2a porte `closes #964` et `refs #955` ; celle de la 35-2b, `closes #955`. Dans les commits : `refs` seulement.

### Fichiers communs — règle d'union C-e350-8

Avec la 35-3 : `messages.ftl` ×4, G9 (ici une entrée neuve en fin de table ; la 35-3, des clés en fin du bloc
`errors.rs`), `user-manual.tex`, `CHANGELOG.md`, `docs/api-external.md`. `errors.rs` : cette story n'y touche pas (V4-3). Avec la 35-2b, qui part de celle-ci fusionnée : `user-manual.tex:1251`, `api-external.md`, `CHANGELOG.md`.

### Ce qui doit rester vrai

- Le 400, le code `INVOICE_NOT_PDF_READY` et l'absence de `details` ; `AppError::InvoiceNotPdfReady(String)`.
- `INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`, `INVOICE_PDF_GONE` inchangés.
- Un PDF figé n'est pas régénéré. Fixtures synthétiques seulement (`Ve–vey`, `Muster SA`).

### Références

- Issues #955, #964 ; validations `kesh-gate-logs/e35-2-validation-p1.md`, `e35-2-validation-p2.md`,
  `e35-2a-validation-p3.md` ; fiche d'origine `35-2-refus-pdf-nomme-le-champ.md` (statut `split`).
- `CLAUDE.md` : § *Test Locally First* ; § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ;
  § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-12 — Née du découpage de la 35-2 à la validation P2 (C-e350-9) ; P2 remédiée ici (entrée « Validation P2 » de
  la fiche d'origine).
- 2026-11-12 — **Validation P3** (Sonnet) : 0 CRITICAL, 0 HIGH, 8 MEDIUM, 7 LOW (`kesh-gate-logs/e35-2a-validation-p3.md`).
  **Série 5 → 5 → 8** (P1 et P2 sur la 35-2, P3 sur la 35-2a) : la sévérité monte, et quatre MEDIUM naissent de la
  remédiation P2, dont trois du contrat `details` (V3-4, V3-5, V3-6). **Recadrage décidé par l'orchestrateur**
  (C-e350-11, réversible) : la story revient au besoin de Guy — le message localisé nomme le champ et le caractère, l'écran
  l'affiche — **sans contrat `details`** ; le contrat part en issue. Prémisses relues au code de `release/0.17.6` ; aucune
  réfutée (`errors.rs:36-64`, `:5307` ; `audit_labels.rs:277` ; `pdf-error.ts:36-40` ; `api-client.ts:283` ;
  `invoice_pdf_service.rs:409` ; `types.rs:363-409`). Verdicts :
  - **sans objet après recadrage** : V3-4 (source unique des `reason`), V3-5 (correspondance `field` → ressource), V3-6
    pour sa part contractuelle — repris dans l'issue du contrat ; V3-8 pour `details.reason` (le test par route asserte
    désormais le message, AC11) ;
  - **appliqués** : V3-1 (D3, fonction pure, AC2 par bundle local dans les quatre locales) ; V3-2 (AC13 par sites, M12
    dans `errors.rs`, alias) ; V3-3 (D1, bras propre, message vide) ; V3-6 a et b pour les phrases (D5, D7) ; V3-7 (D3 :
    la variante reste `String`, rien à changer dans `classify_render_error` ni à `:1754` ; quatorze constructions, AC3) ;
    V3-8 (AC11) ; V3-9 (G9 à 10 entrées, recompté) ; V3-10 (`:409`, `char::from_u32`) ; V3-11 (fermeture des issues) ;
    V3-12 (NPA posé en SQL) ; V3-13 (D9, et une issue pour `validate_template`) ; V3-14 (texte changé, dit au CHANGELOG et à
    l'API) ; V3-15 (« entreprise », registre `e352a`, captures `e352a`).
  - Symptômes greppés (35-2a, 35-2b, fichier d'epic, registre) : `details.reason`, `INVALID_CHARSET`, « onze raisons »,
    « longueur 9 », `e352/` — aucun reste hors du recadrage écrit et du registre C-e350-10, annoté.
- 2026-11-12 — **Validation P4** (Opus) : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 9 LOW (`kesh-gate-logs/e35-2a-validation-p4.md`).
  **Série 5 → 5 → 8 → 4.** Deux MEDIUM viennent de la remédiation P3 (V4-1, V4-2), un des restes du contrat retiré (V4-3),
  un de la conception d'origine (V4-4). Prémisses relues au code de `release/0.17.6` ; aucune réfutée
  (`textes_coherents.rs:900-910` : G9 compare un littéral ; `pdf-error.ts:43` ; `invoice_pdf_service.rs:1082` ;
  `issued_invoice_pdf.rs:240-249`). Appliqués :
  - V4-1 : forme (a), une clé par champ et par raison ; la fonction pure rend aussi le repli ; décompte à AC8 (39 clés
    neuves, 41 + 1 entrées de G9) ;
  - V4-2 : une seule fonction de construction appelle `t_args`, M9 s'y joue, AC2 ne la revendique plus ; marques
    d'isolation dites ;
  - V4-3 : `errors.rs:5715-5723` retiré d'AC9 (la variante et `build_response` ne changent pas) ; propagé **hors de la
    fiche** : fichier d'epic (`errors.rs` et `invoices/[id]/+page.svelte` : la 35-2a n'y touche pas), fiche 35-3 (liste
    d'union), fiche 35-2b (AC5 : `details` porte `message`) ; registre : C-e350-3 et C-e350-8 point 4 **annotés**
    suspendus par C-e350-11, texte non réécrit ;
  - V4-4 : ensemble clos de D5 (dix variantes de `QrBillError` et causes propres à Kesh) ; `InvalidQrr` et `InvalidPayload`
    rejoignent la phrase de panne interne ;
  - LOW : V4-5 (M3/M4 déclarées équivalentes pour le message ; la phrase tient #964) ; V4-6 (point de code en chaîne) ;
    V4-7 (rendu ≠ clé, ≠ fr-CH) ; V4-8 (montages d'AC11) ; V4-9 (`:43`, appelants alignés, hunk `:949` retiré de l'epic et
    de la 35-3) ; V4-10 (C-e350-8 point 1 annoté : la 35-2b complète le point #955 de la 35-2a) ; V4-11 (D10) ; V4-12
    (ligatures, numéro de facture au manuel) ; V4-13 (commentaire rédigé pour #1045, à poster par l'orchestrateur).
  - Symptômes greppés dans la fiche, la 35-2b, la 35-3, le fichier d'epic et le registre : « porte en plus `message` »,
    « le seul test `:5715-5723` », « n'y touche qu'un test », « 35-2a `:949` », « bras `InvoiceNotPdfReady` et ce qu'il
    appelle » — restes traités ou annotés.
- 2026-11-12 — **Validation P5** (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW (`kesh-gate-logs/e35-2a-validation-p5.md`).
  **Série 5 → 5 → 8 → 4 → 2** ; les deux MEDIUM sont d'origine (décompte de P4, conception), aucun signal de recyclage.
  Prémisses relues au code de `release/0.17.6` ; aucune réfutée (G9 compte le littéral dans le fichier de l'entrée,
  `textes_coherents.rs:896-913` ; replis `:227`, `:325`). Appliqués :
  - V5-1 : recommandation (a) : le repli de l'avoir est dans la fonction pure ; G9 **42 / 0 / 9** (une entrée,
    `invoice_pdf_service.rs`, 42 clés ; aucune pour `credit_notes.rs` ; table 8 → 9) ; propagé au fichier d'epic, au
    registre (C-e350-8, point 3, annotation datée) ; la fiche 35-3 ne portait pas ce décompte (vérifié) ;
  - V5-2 : AC2 (c), table « champ → (partie, mot) » écrite à part ; M2 réécrite ;
  - LOW : V5-3 (M3/M4 équivalentes, non jouées) ; V5-4 (comparaisons en minuscules) ; V5-5 (repli comparé au rendu
    fr-CH, jamais à un littéral) ; V5-6 (replis des trois reprises alignés) ; V5-7 (AC14, garde des constructions, M14) ;
    V5-8 (`max`/`got` en chaînes ; code du pays tel que stocké) ; V5-9 (renvoi de la 35-3 vers « 35-2a, D10 »). Étape 0
    sur `release/0.17.6`. Reste corrigé au passage : « Fichiers communs » disait encore que la story touche
    `errors.rs:5715-5723` (V4-3).
  - Symptômes greppés (fiche, 35-3, 35-2b, fichier d'epic, registre) : `41`, `\b10\b` près de G9, `credit_notes` —
    restes traités ; les entrées passées du Change Log gardent leurs chiffres.
- 2026-11-12 — **Validation P6** (Haiku, passe ciblée sur `ae2c9fd8`) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 3 LOW, plus un écart
  hors remédiation (`kesh-gate-logs/e35-2a-validation-p6-ciblee.md`). Appliqués après lecture au code de `release/0.17.6` :
  V6-1 (D7 : la saisie met le pays en majuscules, `address_input.rs:37-41` lu ; seule une valeur stockée en minuscules est
  refusée) ; V6-2 (AC14 aux tests, rouge au départ, et à la tâche Serveur) ; V6-3 (`:228`, `:326`, lus) ; V6-4 (`:491` lu :
  « `InvoiceNotPdfReady` (400) », sans le motif ; consigne retirée) ; H-1 (table G9 à `textes_coherents.rs:734`, lue ; 1
  renvoi corrigé ici ; dans la 35-3, le renvoi vise la fonction `les_replis_rust_suivent_le_catalogue`, à `:723`, et le
  reste — corrigé en P7, V7-1). Aucune autre modification.
- 2026-11-12 — **Validation P7** (Sonnet, passe ciblée sur `1694f215`) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW
  (`kesh-gate-logs/e35-2a-validation-p7-ciblee.md`). V7-1 appliqué : la 35-3 renvoie à la fonction (`:723`), la 35-2a à la
  table (`:734`). Hors code de production, sur une passe sans rien au-dessus de LOW.
- 2026-11-12 — **Validation terminée.** Série des MEDIUM : P1 5 → P2 5 (sur la 35-2) → P3 8 → **recadrage C-e350-11** →
  P4 4 → P5 2 → P6 1 → P7 0 ; CRITICAL et HIGH à 0 sur toutes les passes. Modèles : P1 Sonnet, P2 Opus, P3 Sonnet, P4 Opus,
  P5 Sonnet (complètes), P6 Haiku et P7 Sonnet (ciblées). Décisions : **découpage C-e350-9** à la P2 (35-2a et 35-2b, signal
  de recyclage), **recadrage C-e350-11** à la P3 (message localisé, sans contrat `details` ; contrat renvoyé à #1045) ;
  fonction pure et fonction de construction uniques, une clé par champ et par raison (G9 42 / 9), ensemble clos des causes,
  gardes AC13 et AC14 ; #944 hors (C-e350-1) ; #1046 pour le format de numérotation.

