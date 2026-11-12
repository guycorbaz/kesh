# Story 35-2 : le refus du PDF dit quel champ et quel caractère corriger ; une adresse sans rien de visible est dite incomplète (#955, #964)

Status: split

> **Découpée à la validation P2 (2026-11-12, C-e350-9)** en **35-2a** (`35-2a-refus-pdf-contrat-details.md` : routes
> unitaires, contrat `details`, #964, pays, écran, manuel, E2E) et **35-2b** (`35-2b-lot-rappels-refus-pdf.md` : lot de
> rappels). Le corps ci-dessous est l'état remédié en P1, gardé comme historique ; les fiches filles font foi.

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**, tête d'epic : à livrer avant la première facture client de Guy (au plus tard le
2026-11-30). **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`), d'où la branche de développement
partira. Registre du kickoff : `e35-choix-autonomes-e350.md` (C-e350-1, C-e350-3, C-e350-4, C-e350-8) ; registre de la
story : `e35-choix-autonomes-e352.md`, à créer au premier choix. Validation P1 remédiée (Change Log).*

## Story

En tant que titulaire qui émet sa première QR-facture, quand le PDF est refusé, je veux lire **quel champ** (nom, rue,
NPA, localité du client ou de la société, communication) et **quel caractère** corriger, au téléchargement comme dans
le rapport d'un lot de rappels, afin de corriger la fiche du client sans chercher à l'aveugle.

## Ce qui se lit au code (`release/0.17.6`)

**Le serveur sait, l'écran tait.**

- `kesh-qrbill` refuse avec un champ nommé : `QrBillError::InvalidCharset { field, codepoint }`, `FieldEmpty(field)`,
  `FieldTooLong { field, max, got }` (`crates/kesh-qrbill/src/types.rs:363-409`). Les noms de champ atteignables sont
  douze : `creditor.` et `debtor.` suivis de `name`, `line1`, `line2`, `postal_code`, `town`
  (`crates/kesh-qrbill/src/validation.rs:372-407`), puis `unstructured_message` et `billing_information` (`:469-474`).
  Les bras par défaut de `field_prefix_*` rendent en outre cinq noms `address.*`, inatteignables depuis Kesh (adresse
  toujours de type S, préfixe toujours `creditor` ou `debtor`).
- `map_qrbill_error` (`crates/kesh-api/src/routes/invoice_pdf_service.rs:493-542`) les rend en
  `AppError::InvoiceNotPdfReady(String)`, avec des phrases **en français en dur et des noms techniques** :
  `format!("Champ {field} trop long (max {max}, got {got})")` (`:499`), `format!("Champ {field} vide (requis)")`
  (`:502`), `format!("Pays invalide: {c}")` (`:508`), `err.to_string()` pour le jeu de caractères (`:510`, texte de
  `types.rs:385-387` : « Champ debtor.name contient un caractère non autorisé … U+2013 »), et le texte brut de
  `kesh-qrbill` pour l'IBAN, le QR-IBAN, la QRR, le montant, la devise et le payload (`:495-497`, `:503-505`, `:540`).
- `AppError::InvoiceNotPdfReady(msg)` → `build_response(StatusCode::BAD_REQUEST, "INVOICE_NOT_PDF_READY", &msg)`
  (`crates/kesh-api/src/errors.rs:2073-2075`) ; `build_response` (`:1420-1431`) ne porte que `code` et `message`,
  **jamais de `details`**.
- `invoice_pdf_service.rs:338` passe la **clé nue** `"qrbill-error-qrr-generation"` comme message ; le commentaire
  de `:334-335` affirme que `errors.rs` la résout : c'est faux, `build_response` reçoit `&msg` tel quel, et **cette clé
  n'existe dans aucun catalogue** (`git grep` ne la trouve qu'à `:338`).
- Les replis Rust des refus d'adresse ne sont pas ceux du catalogue : `:289` rend « Adresse entreprise incomplète (NPA
  et localité requis). », fr-CH `invoice-pdf-error-company-address-empty` dit « L'adresse de l'entreprise est vide —
  renseignez-la avant de générer un PDF. » (`messages.ftl:969`). Aucun de ces replis n'est dans la table **G9**
  (`crates/kesh-api/tests/textes_coherents.rs:723`). `invoice-pdf-error-client-address-empty` (`messages.ftl:971`)
  n'est employée nulle part.
- `errors::t` lit un état i18n **global** (`errors.rs:21-42`) et rend le repli tant que `init_error_i18n` n'a pas été
  appelée ; `t` ne passe pas d'arguments (`:36`), `t_args` oui, et retire les marques BiDi (`:56`).

**Les routes et les écrans qui montrent ce refus** (V1-5 de la validation P1) :

| Route | Site du rendu | Écran | Aujourd'hui |
|---|---|---|---|
| `GET /invoices/{id}/pdf` (téléchargement) | `invoice_pdf.rs:50` → `issued_invoice_pdf::get_or_freeze` → `render_document` (`issued_invoice_pdf.rs:151`) | fiche facture, `pdfErrorMessage` (`invoices/[id]/+page.svelte:949`) | phrase fixe |
| `POST /invoices/{id}/pdf/refreeze` | `invoice_pdf.rs:88` → `render_document` (`issued_invoice_pdf.rs:256`) | `refreezeError = err.message` (`invoices/[id]/+page.svelte:911`) | message brut |
| envoi de la facture par e-mail | `invoice_email.rs:808` (`get_or_freeze`) | branche `default`, `err.message` (`invoices/[id]/+page.svelte:864-866`) | message brut |
| rappel seul | `invoice_email.rs:560` | `sendError = err.message` (`invoices/reminders/+page.svelte:232`) | message brut |
| lot de rappels | `invoice_email.rs:1260-1269` → `classify_render_error` (`:1023-1036`) | `ReminderBatchReport.svelte:30`, `reminderErrorLabel(code)` | « Facture non imprimable en PDF », sans `details` |
| PDF d'un avoir | `credit_notes.rs:295` (contact absent), `:363` (`map_qrbill_error`) | fiche avoir, `pdfErrorMessage` (`credit-notes/[id]/+page.svelte:106`) | phrase fixe |

Pour l'avoir, `generate_credit_note_pdf_with_date` (`kesh-qrbill/src/pdf.rs:124-`) n'appelle ni `build_payload` ni
`validate` : aucun refus de **champ** n'en sort ; seuls le contact absent, `TooManyLines` et `HeaderOverflow`.

**#964.** `build_qrbill_inputs` juge le NPA et la localité par `trim().is_empty()` : société
(`invoice_pdf_service.rs:287`, le `return` à `:288`) et client (`:310`). Un champ fait du seul U+00AD (ou U+200B), que
`str::trim` ne retire pas, passe ce contrôle, puis `kesh-qrbill` le refuse en `FieldEmpty` (« Champ creditor.postal_code
vide (requis) ») au lieu du message « adresse incomplète » de Kesh. La règle de `kesh-qrbill` est publique :
`kesh_qrbill::champ_vide_pour_qr` (`validation.rs:244-246`), déjà employée par la saisie de l'adresse de la société
(`crates/kesh-api/src/address_input.rs:110`).

**Le modèle à suivre existe** : la correction de l'adresse de la société nomme déjà champ et caractère,
« adresse : le champ {label} contient un caractère que la QR-facture n'admet pas : « {c} » (U+XXXX) », avec
`c.escape_debug()` (`address_input.rs:131-137`).

## Décisions (C-e350-3, à reprendre au registre `e352` si la story les précise)

- **D1 — Le serveur résout le message, l'écran le reprend.** Chaque `InvoiceNotPdfReady` porte un message résolu par
  une clé Fluent, dans la langue de l'installation, **par `t_args`** quand il a des variables (`t` rendrait `{ $field }`
  brut) ; `INVOICE_NOT_PDF_READY` rejoint `SERVER_RESOLVED_CODES` (`pdf-error.ts:36`). La clé fixe
  `invoice-pdf-error-invoice-not-pdf-ready` reste le repli d'un `err.message` vide.
- **D2 — Les champs portent des `details`** : `{ "field": "debtor.town", "reason": "INVALID_CHARSET" | "EMPTY" |
  "TOO_LONG", "char": "–", "codepoint": "U+2013", "max": 35, "got": 40 }` (seuls les membres utiles à la raison). Les
  discriminants sont en SCREAMING_SNAKE, comme les autres discriminants machine du dépôt (`errors.rs:62-69`). Les noms
  de champ sont ceux de `kesh-qrbill`, stables, pour un intégrateur ; le message les traduit en mots (« la localité du
  client »). `details.char` porte le caractère **brut** (un `"\n"`, un U+00AD) : c'est une donnée ; seul le message le
  rend lisible (D4).
- **D3 — Forme côté Rust, laissée au développeur** : une variante neuve à charge structurée (par exemple
  `InvoiceQrFieldRefused { field, reason }`) ou une charge `details` ajoutée à `InvoiceNotPdfReady`. ⚠️ **Toute variante
  neuve s'ajoute à `classify_render_error`** (`invoice_email.rs:1023-1036`) : sinon elle tombe dans le bras final et le
  lot l'annonce `DATABASE_ERROR`, fausse panne (règle écrite au doc-comment de `:1010-1022`). Le test
  `classify_render_error_ne_deguise_pas_un_refus_en_panne` (`:1749`) s'étend à elle. La correspondance des noms de champ a
  un **bras de repli** pour un nom inconnu (les `address.*`, un nom futur) : message générique, jamais « Champ … ».
- **D4 — Le caractère s'affiche par `escape_debug`** (un U+00AD, un saut de ligne, une espace insécable restent lisibles),
  suivi de son point de code `U+XXXX`, comme `address_input.rs:131-137`.
- **D5 — Les refus sans champ** (IBAN, QR-IBAN, QRR, montant, devise, pays, payload) reçoivent une phrase localisée et
  générique ; le détail technique de `kesh-qrbill` part au journal (`tracing::warn!`), non au client. La clé de la QRR
  (`:338`) est **créée** ×4.
- **D6 — Les refus d'adresse** (V1-8) : un client sans adresse, ou dont le NPA ou la localité ne porte rien de visible,
  reçoit la même clé `invoice-pdf-error-client-address-required` ; la société, `invoice-pdf-error-company-address-empty`.
  Les replis Rust de ces refus et de `:216`, `:226` s'alignent sur leur valeur fr-CH et entrent à G9. La clé morte
  `invoice-pdf-error-client-address-empty` est retirée des quatre catalogues si aucun test ne l'épingle ; sinon, une issue.

## Critères d'acceptation

1. **AC1 — Un caractère refusé est nommé, au serveur.** Facture validée dont le client a pour localité
   `Ve\u{2013}vey` (fixture synthétique) : `GET /api/v1/invoices/{id}/pdf` rend 400 `INVOICE_NOT_PDF_READY`, un message
   (fr-CH) qui contient « localité du client », « – » et « U+2013 », et `details` égal à
   `{ "field": "debtor.town", "reason": "INVALID_CHARSET", "char": "–", "codepoint": "U+2013" }`. Même chose, côté
   société, pour le nom (`creditor.name`, « nom de la société ») et pour la communication (`unstructured_message`).
   Tests d'intégration, application montée, donc catalogue chargé : les libellés fr-CH ne s'assertent **qu'ici**.
2. **AC2 — Vide et trop long, la table complète.** `FieldEmpty` et `FieldTooLong` rendent leur champ en mots,
   `details.reason` à `EMPTY` ou `TOO_LONG`, `max` et `got` pour le second. Un test unitaire sur la fonction de
   correspondance couvre **les douze champs × trois raisons** par une table écrite dans le test, plus le bras de repli
   (D3). ⚠️ **Il ne dépend pas de l'état i18n global** (V1-1 : sous `cargo test`, un processus par binaire, l'état
   dépend de l'ordre des tests ; sous `nextest`, il est vide — le piège des 17 tests rouges à la seule CI de la 0.17.4) :
   il n'asserte que la clé choisie, les arguments passés, les `details`, et l'absence de `creditor.`, `debtor.`,
   `address.`, `unstructured_message`, `billing_information` et du mot « Champ » dans le message rendu **sous repli**.
   S'il veut lire le catalogue, il prend le verrou et initialise le bundle lui-même (précédent : `verrou_i18n`,
   `crates/kesh-api/tests/audit_log_e2e.rs:33`). Les combinaisons inatteignables de la table (vide × `line1`, …)
   valent pour la complétude, non pour le comportement.
3. **AC3 — Inventaire clos des sites de `InvoiceNotPdfReady`**, tous résolus par une clé. Les seize lignes de
   `release/0.17.6` (`git grep -n 'InvoiceNotPdfReady' -- crates`, hors `errors.rs`), soit treize constructions :
   `credit_notes.rs:295` ; `invoice_pdf_service.rs:216, 226, 288, 324, 338, 497, 499, 502, 505, 508, 510, 540` ; et
   trois lignes qui n'en sont pas : `invoice_pdf_service.rs:491` (doc-comment), `invoice_email.rs:1030` (le
   classement) et `:1754` (un test). La story recompte cette liste et le tableau des routes au code de départ, et écrit
   au Change Log le verdict de chaque site et de chaque route. **`:338`** cesse d'envoyer une clé nue (D5), et son
   commentaire `:334-335` est corrigé.
4. **AC4 — #964.** Le NPA et la localité de la société et du client se jugent par `kesh_qrbill::champ_vide_pour_qr`
   (`invoice_pdf_service.rs:287`, `:310`). Une société dont le NPA vaut `"\u{AD}"` (posé en SQL par le test) reçoit le
   message `invoice-pdf-error-company-address-empty` ; un client dont la localité vaut `"\u{200B}"`, le message
   `invoice-pdf-error-client-address-required` (D6). Aucun des deux ne dit « Champ … vide ».
5. **AC5 — L'écran reprend le message.** `pdfErrorMessage` rend `err.message` pour `INVOICE_NOT_PDF_READY` (fiche
   facture et fiche avoir), et la clé fixe si `err.message` est vide. Le mock de `pdf-error.test.ts` reste un catalogue
   qui l'emporte sur le repli (`:1-19`) : le test neuf rougit sur le code d'aujourd'hui.
6. **AC6 — Le lot de rappels.** **Tout** échec `INVOICE_NOT_PDF_READY` du lot né d'un `InvoiceNotPdfReady` porte au
   moins `details.message` (la phrase résolue), et pour un refus de champ les `details` de AC1 en plus ; ceux nés de
   `InvoiceTooManyLinesForPdf` et `InvoicePdfHeaderOverflow`, qui partagent ce code, n'en portent pas.
   `ReminderBatchReport.svelte` affiche « Facture non imprimable en PDF — *phrase* » quand `details.message` est là, le
   libellé seul sinon. Tests : par HTTP, `POST` du lot (`crates/kesh-api/tests/invoice_send_email_e2e.rs`, montage de
   `:1552-1822`), un refus de champ **et** un refus **sans champ** (adresse du client incomplète), pour voir `details`
   traverser la route ; Vitest du composant.
7. **AC7 — Pas de fausse panne.** Un refus de champ dans un lot de rappels sort `INVOICE_NOT_PDF_READY`, jamais
   `DATABASE_ERROR` (D3).
8. **AC8 — Quatre langues et G9.** Chaque clé neuve existe dans `fr-CH`, `de-CH`, `it-CH`, `en-CH`
   (`parity_between_locales`, `crates/kesh-i18n/src/loader.rs:771`), placée **à côté du bloc `invoice-pdf-error-*`**
   (C-e350-8), jamais en fin de fichier ; aucun sélecteur de pluriel (sinon inscription à
   `SELECTEURS_RESOLUS_COTE_SERVEUR`, `loader.rs:377`). Les `messages.ftl` portent les libellés des douze champs. Les
   replis Rust neufs et ceux de D6 égalent leur valeur fr-CH et entrent à **G9**
   (`les_replis_rust_suivent_le_catalogue`) : une **entrée neuve en fin de table** pour
   `crates/kesh-api/src/routes/invoice_pdf_service.rs` (la longueur passe de 8 à 9), et d'éventuelles clés d'`errors.rs`
   en fin de son bloc, sous `// Story 35-2` (C-e350-8). Paramètres : sur le modèle de `CHAMP` et `CHAMP_MAX`.
9. **AC9 — Tests existants qui épinglent l'ancien comportement**, réécrits sans perdre ce qu'ils prouvent :
   - `crates/kesh-api/tests/company_address_e2e.rs:583-589` (« Pays invalide ») ; `errors.rs:5715-5723` (message tel
     quel) ;
   - `frontend/src/lib/shared/utils/pdf-error.test.ts:35-38`, « un code sans variable : la clé du catalogue », qui
     attend la phrase fixe pour `INVOICE_NOT_PDF_READY` : il **rougira** ; il garde sa preuve sur un autre code sans
     variable (`INVOICE_NOT_VALIDATED`), et le cas `INVOICE_NOT_PDF_READY` passe à AC5 ;
   - `frontend/tests/e2e/invoices.spec.ts:723-755` : l'assertion reste vraie (« compte bancaire principal »), mais son
     commentaire (« le message backend intercepté n'est pas affiché tel quel ») devient faux : à corriger ;
   - le commentaire de `company_address_e2e.rs:375-376` (« va être recalée … (33-12) ») passe au présent (LOW de #964).
10. **AC10 — Manuel, CHANGELOG, contrat.** `docs/manual/fr/user-manual.tex:1251` dit aujourd'hui que le téléchargement
    échoue « sans nommer le champ » et que le lot affiche « Facture non imprimable en PDF » (issue #955) : réécrit pour
    dire ce que l'écran montre désormais, avec un exemple synthétique. PDF régénéré, contrôlé aplati
    (`pdftotext … | tr '\n' ' ' | tr -s ' '`). CHANGELOG : section `## [0.17.7] — Non publié`, rubrique *Corrigé*,
    refs #955 et refs #964, par la règle d'union C-e350-8. `docs/api-external.md` ne cite ni `INVOICE_NOT_PDF_READY` ni
    la route du PDF : une ligne neuve au tableau des erreurs, **juste après `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`**
    (`:1058`), qui dit le code, le 400, les routes et `details` (C-e350-8).
11. **AC11 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e352/captures/`) : le message du
    téléchargement refusé sur la fiche facture ; la ligne du rapport d'un lot de rappels ; l'erreur dans la modale
    d'envoi par e-mail ; l'erreur d'un rappel seul. Le refigeage reprend aussi `err.message` (`+page.svelte:911`) : rien
    à y changer, une capture si le cas se monte sans détour, sinon la phrase au Dev Agent Record.
12. **AC12 — E2E.** Dans `frontend/tests/e2e/invoice-qr-caracteres.spec.ts`, un scénario : client à la localité
    `Ve–vey`, facture validée, clic sur *Télécharger le PDF* : le message affiché contient « localité du client » et
    « U+2013 ». `createContactWithAddressViaApi` (`frontend/tests/e2e/helpers/api-fixtures.ts:73-79`) fixe la localité
    `Rorschach` : il reçoit un paramètre **facultatif, en dernier**, `city`, sans toucher aux appels existants. C'est le
    seul test qui voie la phrase traverser HTTP jusqu'à l'écran.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, puis lancer sur le code de départ, et garder le journal (`kesh-gate-logs/e352/rouges.log`) :

- `invoice_pdf_service.rs`, `mod tests` : la table de AC2, indépendante de l'état i18n (rouge : « Champ debtor.town … »),
  et AC4 sur `build_qrbill_inputs` (rouge : passe le contrôle de Kesh).
- `crates/kesh-api/tests/invoice_pdf_e2e.rs` : AC1 et AC4 de bout en bout (rouge : pas de `details`, message
  technique).
- `crates/kesh-api/tests/invoice_send_email_e2e.rs` : AC6 par le lot, deux refus (rouge : `details` à `null`) ;
  `invoice_email.rs`, `mod tests` : AC7.
- `frontend/src/lib/shared/utils/pdf-error.test.ts` : AC5 (rouge : la clé fixe) et la réécriture de AC9.
- un Vitest de `ReminderBatchReport.svelte` (AC6 ; le composant n'a pas de test aujourd'hui).
- E2E de AC12 (rouge sur le binaire de départ).

Un test qui passe sous `nextest` doit passer sous `cargo test` : `scripts/gate-ci.sh` au dernier commit de code
(V1-1).

**Mutations à jouer et consigner** : M1 `INVOICE_NOT_PDF_READY` retiré de `SERVER_RESOLVED_CODES` ; M2 un libellé de
champ retiré ; M3 `trim().is_empty()` rétabli à `:287` ; M4 idem à `:310` ; M5 la variante neuve (si D3 en crée une)
retirée de `classify_render_error` ; M6 `details.message` non lu par le rapport du lot ; M7 `escape_debug` retiré ;
M8 `details.message` omis pour un refus sans champ du lot ; M9 `t` au lieu de `t_args` pour un message à variables.

## Tâches

- [ ] Étape 0 : recompter l'inventaire de AC3, le tableau des routes et les lignes citées sur la branche de départ ;
      écrire au Change Log toute ligne qui a bougé.
- [ ] Tests de la section précédente, rouges consignés.
- [ ] Serveur : correspondance des refus de champ, `details`, clés Fluent ×4, `champ_vide_pour_qr` aux deux sites,
      `:338`, replis alignés et G9, lot de rappels (AC1–AC4, AC6–AC8).
- [ ] Écran : `pdf-error.ts`, `ReminderBatchReport.svelte` (AC5, AC6).
- [ ] Tests existants mis à jour (AC9) ; manuel, CHANGELOG, `api-external.md` (AC10) ; captures (AC11) ; E2E (AC12).
- [ ] Gates : `test-fast.sh --ci` et `scripts/gate-ci.sh`, frontend (`npm run check`, `lint-i18n-ownership`,
      `test:unit`, `build`), E2E complet au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs
      attendus ».

## Dev Notes

### Hors périmètre

- **#944** (ligne verticale, ciseaux, « Payable by »…) : C-e350-1.
- **#942** (admettre plus de typographies) et **#952** (contrôler le jeu de caractères à la saisie d'un contact) : la
  story nomme le caractère, elle ne change pas ce qui est admis.
- Le **contrat** de `kesh-qrbill` ne change pas : ses variantes portent déjà le champ.

### Fichiers communs avec la 35-3 — règle d'union (C-e350-8)

`crates/kesh-api/src/errors.rs` (ici le bras `InvoiceNotPdfReady` et ce qu'il appelle ; la 35-3, un bras `DbError` après
`DesignatedAccountsNotPostable`), `messages.ftl` ×4 (ici à côté du bloc `invoice-pdf-error-*` ; la 35-3, après
`error-designated-account-not-postable`), `textes_coherents.rs` (G9 : ici une entrée neuve en fin de table et, au
besoin, des clés en fin du bloc `errors.rs` sous `// Story 35-2` ; la 35-3 sous `// Story 35-3`), `user-manual.tex` (ici
l. 1251 ; la 35-3 l. 452-461, 1169, 1788-1796), `CHANGELOG.md` (chaque branche crée `## [0.17.7] — Non publié` avec la
seule rubrique `### Corrigé` si elle manque ; une seule section à l'union, 35-2 puis 35-3), `docs/api-external.md` (ici
après `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` ; la 35-3 après `ACCOUNT_NOT_POSTABLE`). Le PDF du manuel n'est jamais
fusionné : il se régénère depuis le `.tex` uni. Ne pas réordonner ni reformater ces fichiers hors des passages touchés.

### Ce qui doit rester vrai

- Le 400 et le code `INVOICE_NOT_PDF_READY` ne changent pas ; `details` s'ajoute (AC10).
- Les codes `INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`, `INVOICE_PDF_GONE` gardent leur traitement.
- Un PDF figé n'est pas régénéré : la story ne touche que le refus.
- Le texte des messages ne contient aucune donnée réelle ; les fixtures sont synthétiques (`Ve–vey`, `Muster SA`).

### Références

- Issues #955, #964 (et son commentaire) ; classement `kesh-gate-logs/epic35-classement-bugs.md` § 7.3 ; validation P1
  `kesh-gate-logs/e35-2-validation-p1.md`.
- Story 33-5 (`address_input.rs`, modèle du message) ; Story 33-12a (jeu de caractères, `invoice-qr-caracteres.spec.ts`).
- `CLAUDE.md` : § *Test Locally First* (`cargo test` contre `nextest`) ; § *Pattern batch — FailedProposal
  per-proposal* ; § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`).

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (kickoff de l'Epic 35). Validation non lancée.
- 2026-11-12 — **Validation P1** (Sonnet) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 8 LOW (le rapport en annonce sept dans son
  verdict ; sa liste en porte huit, V1-6 à V1-13). Prémisses relues au code de `release/0.17.6` avant chaque patch ;
  aucune réfutée. Appliqués :
  - V1-1 : AC2 indépendant de l'état i18n global (ou verrou et bundle explicites, `audit_log_e2e.rs:33`) ; libellés fr-CH
    assertés dans les seuls tests d'intégration ; `scripts/gate-ci.sh` aux gates ;
  - V1-2 : G9 à AC8 ; replis de `build_qrbill_inputs` alignés sur fr-CH (D6) ;
  - V1-3 : AC6, `details.message` pour **tout** `InvoiceNotPdfReady` du lot, test d'un refus sans champ, M8 ;
  - V1-4 : AC9, `pdf-error.test.ts:35-38` et le commentaire de `invoices.spec.ts:723-755` ;
  - V1-5 : tableau des routes (refigeage et rappel seul ajoutés ; l'avoir ne sort aucun refus de champ), captures ;
  - V1-6 : `:287` au lieu de `:288` (AC4, M3) ; V1-7 : clé de la QRR à créer (D5) ; V1-8 : D6, une clé par adresse
    incomplète, clé morte retirée ; V1-9 : discriminants en SCREAMING_SNAKE, `details.char` brut ; V1-10 : `t_args`
    (D1, M9) ; V1-11 : bras de repli et noms `address.*` (D3, AC2) ; V1-12 : paramètre facultatif `city`, test du lot
    par HTTP ; V1-13 : `api-external.md` sans le code (ligne neuve), CHANGELOG et fichiers communs par la règle d'union
    C-e350-8, clés jamais en fin de fichier.
  - Symptômes greppés dans la fiche et le fichier d'epic : `:288` comme site du contrôle, « quatre écrans »,
    `"charset" | "empty" | "tooLong"`, « si le code y figure » — aucune occurrence restante hors de cette entrée.
- 2026-11-12 — **Validation P2** (Opus) : 0 CRITICAL, 0 HIGH, 5 MEDIUM, 6 LOW (`kesh-gate-logs/e35-2-validation-p2.md`).
  Prémisses relues au code de `release/0.17.6` ; aucune réfutée (`init_error_i18n` : `main.rs:340` seul hors définition ;
  `messages.ftl:970` ; `credit_notes.rs:295-297` ; `invoice_pdf_service.rs:404-410` ; `validation.rs:10`, `:259`, `:285`).
  - **Signal de découpage** (`CLAUDE.md`, § *Règle de splitting préventif*) : P1 5 MEDIUM → P2 5 MEDIUM, sévérité égale. Constat :
    **recyclage** — quatre des cinq MEDIUM naissent de la remédiation P1 (V2-1 : l'échappatoire « verrou et bundle » écrite pour
    V1-1 ; V2-2 : D6 écrit pour V1-8 ; V2-4 : la forme de `details` fixée pour les seuls refus de champ en P1 ; V2-5 : M9 ajoutée
    en P1 sans test qui la tue) ; seul V2-3 (pays) est un défaut d'origine. Et le contrat grossit (onze raisons) sur plus de cinq
    modules (`routes/invoice_pdf_service`, `routes/invoice_email`, `routes/credit_notes`, `errors`, `kesh-i18n`, `shared/utils`,
    `features/reminders`, E2E). **Décision : découper** (C-e350-9) en 35-2a et 35-2b, la 35-2b partant de la 35-2a fusionnée.
    Signal à déclarer à Guy par l'orchestrateur.
  - Remédiés dans la **35-2a** : V2-1 (D9, AC2 sans état global, garde AC13) ; V2-2 (D6 : le champ vide est nommé, adresse
    « incomplète ») ; V2-3 (D7 : pays nommé, `INVALID_COUNTRY`) ; V2-4 (D2 : contrat unique, `reason` toujours présent, onze
    raisons) ; V2-5 (AC1 : `\n`, `TOO_LONG` à 71, `EMPTY` sur le nom de la société, M9 par raison) ; V2-6 (onze champs
    atteignables, fixture `F–1`) ; V2-7 (D8, clé de l'avoir) ; V2-8 (aperçus nommés) ; V2-9 (D4, variables nommées pour G9) ;
    V2-11 (AC7, `message: ""`). Dans la **35-2b** : V2-10 (modèle `send_reminder_batch_partial_success`, `:2019`).
  - Symptômes greppés (fiches 35-2, 35-2a, 35-2b, fichier d'epic) : « prend le verrou et initialise le bundle »,
    « douze champs », `client-address-required` pour une adresse partielle, « adresse incomplète » au fichier d'epic.

