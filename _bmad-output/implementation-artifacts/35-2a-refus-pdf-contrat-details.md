# Story 35-2a : le refus du PDF dit quoi corriger — champ, caractère, adresse, pays — par un contrat `details` stable (#955, #964)

Status: ready-for-dev

*Fiche née le 2026-11-12 du découpage de la 35-2 à la validation P2 (C-e350-9), par l'agent de kickoff de l'Epic 35
(Opus 5.5), worktree `kesh-e35`. Elle reprend le contenu de la 35-2 remédié en P1 et P2, **sans le lot de rappels**, qui
passe à la **35-2b**. Release **0.17.7**, tête d'epic : à livrer avant la première facture client de Guy (au plus tard le
2026-11-30). **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`). Registres : `e35-choix-autonomes-e350.md`
(C-e350-1, -3, -4, -8, -9, -10) ; `e35-choix-autonomes-e352.md` pour la story, à créer au premier choix.*

## Story

En tant que titulaire qui émet sa première QR-facture, quand le PDF est refusé, je veux lire **quel champ** (nom, rue,
numéro, NPA, localité, pays du client ou de la société, communication) et **quel caractère** corriger ; et en tant
qu'agent qui appelle l'API par une clé, je veux lire **la raison** dans un champ stable, sans analyser une phrase.

## Ce qui se lit au code (`release/0.17.6`)

**Le serveur sait, l'écran tait.**

- `kesh-qrbill` refuse avec un champ nommé : `QrBillError::InvalidCharset { field, codepoint }`, `FieldEmpty(field)`,
  `FieldTooLong { field, max, got }` (`crates/kesh-qrbill/src/types.rs:363-409`). Noms de champ :
  `creditor.`/`debtor.` × `name`, `line1`, `line2`, `postal_code`, `town` (`validation.rs:373-407`), puis
  `unstructured_message` et `billing_information` (`:469-474`). **Onze sont atteignables** : `billing_information` vaut
  toujours `None` (`invoice_pdf_service.rs:410`) ; `unstructured_message` est tronqué à 140 caractères (`:404-407`, égal à
  `USTRD_MAX`), si bien que `TOO_LONG` n'y arrive pas. Les bras par défaut de `field_prefix_*` rendent cinq noms
  `address.*`, inatteignables (type S, préfixe `creditor` ou `debtor`).
- `validate_country` (`validation.rs:259`) refuse un pays sans nommer de champ (`InvalidCountry(code)`) ; Kesh sait
  pourtant de quelle partie il s'agit (`invoice_pdf_service.rs:300-304` et `:318-322`).
- `map_qrbill_error` (`invoice_pdf_service.rs:493-542`) rend des phrases **en français en dur et des noms techniques** :
  `:499`, `:502`, `:508` (« Pays invalide: {c} »), `:510` (`err.to_string()`, « Champ debtor.name contient … U+2013 »),
  et le texte brut de `kesh-qrbill` pour l'IBAN, le QR-IBAN, la QRR, le montant, la devise et le payload.
- `AppError::InvoiceNotPdfReady(msg)` → `build_response(…, "INVOICE_NOT_PDF_READY", &msg)` (`errors.rs:2073-2075`) ;
  `build_response` (`:1420-1431`) ne porte jamais de `details`.
- `:338` passe la **clé nue** `"qrbill-error-qrr-generation"`, absente de tout catalogue ; le commentaire `:334-335` dit
  à tort que `errors.rs` la résout.
- Les replis Rust des refus d'adresse ne sont pas ceux du catalogue (`:289` contre fr-CH `messages.ftl:969`) ; aucun
  n'est dans **G9** (`crates/kesh-api/tests/textes_coherents.rs:723`). `invoice-pdf-error-client-address-empty`
  (`:971`) n'est employée nulle part. `credit_notes.rs:295-297` passe la clé de la **facture**
  (`invoice-pdf-error-contact-missing`, fr-CH « … lié à la facture … », `:967`) avec un repli qui dit « l'avoir ».
- **État i18n global** : `errors::t` lit un `RwLock` (`errors.rs:21-42`) et rend le repli tant que `init_error_i18n`
  n'a pas été appelée ; le seul appel hors tests est `main.rs:340` (`git grep -n init_error_i18n -- crates/kesh-api/src`
  ne rend que lui et la définition, `errors.rs:26`). Il n'y a pas de remise à `None`. `t` ne passe pas d'arguments
  (`:36`), `t_args` oui (`:56`).

**Routes et écrans qui montrent ce refus** (la 35-2b prend le lot) :

| Route | Site du rendu | Écran | Aujourd'hui |
|---|---|---|---|
| `GET /invoices/{id}/pdf` | `invoice_pdf.rs:50` → `get_or_freeze` → `render_document` (`issued_invoice_pdf.rs:151`) | fiche facture, `pdfErrorMessage` (`invoices/[id]/+page.svelte:949`) | phrase fixe |
| `POST /invoices/{id}/pdf/refreeze` | `invoice_pdf.rs:88` → `render_document` (`issued_invoice_pdf.rs:256`) | `refreezeError = err.message` (`:911`) | message brut |
| envoi de la facture par e-mail | `invoice_email.rs:808` | branche `default`, `err.message` (`:864-866`) | message brut |
| rappel seul | `invoice_email.rs:560` | `sendError = err.message` (`invoices/reminders/+page.svelte:232`) | message brut |
| PDF d'un avoir | `credit_notes.rs:295`, `:363` (`map_qrbill_error`) | fiche avoir, `pdfErrorMessage` (`credit-notes/[id]/+page.svelte:106`) | phrase fixe |
| lot de rappels | `invoice_email.rs:1260-1269` | `ReminderBatchReport.svelte:30` | **35-2b** |

Les **aperçus** (`preview_invoice_email`, `invoice_email.rs:262` ; `preview_reminder_email`, `:410`) ne rendent pas le
PDF : ils ne produisent pas ce refus (l'envoi qui suit, si). Pour l'avoir, `generate_credit_note_pdf_with_date`
(`kesh-qrbill/src/pdf.rs:124-`) n'appelle pas `validate` : aucun refus de champ n'en sort. Rien d'autre, MCP compris
(`render_document` : `issued_invoice_pdf.rs:151`, `:256`, `invoice_email.rs:560`, `:1260` ; `map_qrbill_error` :
`invoice_pdf_service.rs:257`, `credit_notes.rs:363`).

**#964.** `build_qrbill_inputs` juge le NPA et la localité par `trim().is_empty()` : société (`:287`, le `return` à
`:288`) et client (`:310`). U+00AD et U+200B ne sont pas retirés par `str::trim` : un tel champ passe, puis `kesh-qrbill`
le refuse en « Champ creditor.postal_code vide (requis) ». La règle est publique : `kesh_qrbill::champ_vide_pour_qr`
(`validation.rs:244-246`), employée par la saisie de l'adresse de la société (`address_input.rs:110`).

**Le modèle existe** : `address_input.rs:131-137` nomme champ et caractère (`escape_debug`, `U+XXXX`).

## Décisions

- **D1 — Le serveur résout le message, l'écran le reprend.** Chaque refus porte un message résolu par une clé Fluent,
  **par `t_args`** quand il a des variables ; `INVOICE_NOT_PDF_READY` rejoint `SERVER_RESOLVED_CODES` (`pdf-error.ts:36`).
- **D2 — Contrat unique et exhaustif de `details`** (C-e350-10). **Tout** refus `INVOICE_NOT_PDF_READY`, sur toutes les
  routes du tableau, porte `details`, avec **toujours** un discriminant `reason` en SCREAMING_SNAKE (`errors.rs:62-69`),
  qu'une clé d'API lit sans lire le message :

  | `reason` | Autres membres | Source |
  |---|---|---|
  | `INVALID_CHARSET` | `field`, `char` (brut), `codepoint` (`"U+2013"`) | `InvalidCharset` |
  | `EMPTY` | `field` | `FieldEmpty` ; et le contrôle d'adresse de Kesh (D6) |
  | `TOO_LONG` | `field`, `max`, `got` | `FieldTooLong` |
  | `INVALID_COUNTRY` | `field` (`creditor.country` \| `debtor.country`), `code` | contrôle de Kesh (D7) |
  | `CLIENT_ADDRESS_MISSING` | — | client sans adresse structurée (`:310`, bras `_`) |
  | `CONTACT_MISSING` | — | `:216`, `credit_notes.rs:295` |
  | `NO_PRIMARY_BANK` | — | `:226` |
  | `INVALID_BANK_DATA` | — | IBAN, QR-IBAN, QRR (`:338`, `:495-497`) |
  | `INVALID_AMOUNT` | — | `InvalidAmount` |
  | `INVALID_CURRENCY` | — | `InvalidCurrency` |
  | `INTERNAL_PAYLOAD` | — | `InvalidPayload` (défensif, inatteignable en génération) |

  `field` prend les noms de `kesh-qrbill`, plus `creditor.country`/`debtor.country` ; un nom inconnu (les `address.*`,
  un nom futur) passe par un bras de repli : `reason` gardé, message générique, jamais « Champ … ». La 35-2b ajoute au lot
  le membre `message` à ce **même** objet. Le contrat est recopié à `docs/api-external.md` (AC10).
- **D3 — Forme côté Rust, laissée au développeur** (variante neuve à charge structurée, ou charge ajoutée à
  `InvoiceNotPdfReady`). ⚠️ Toute variante neuve s'ajoute à `classify_render_error` (`invoice_email.rs:1023-1036`), sans
  quoi le lot l'annonce `DATABASE_ERROR` ; la 35-2b s'appuie dessus.
- **D4 — Le caractère s'affiche par `escape_debug`**, lié à une variable nommée (`let car = c.escape_debug().to_string();`
  puis `format!("… « {car} » ({codepoint})")`), pour que G9 puisse comparer le repli (`textes_coherents.rs:896-913`
  remplace chaque `{ $x }` par un nom Rust). `details.char` reste brut.
- **D5 — Les refus sans champ** reçoivent une phrase localisée propre à leur `reason` ; le détail technique de
  `kesh-qrbill` part au journal (`tracing::warn!`). La clé de la QRR est créée ×4.
- **D6 — #964, adresse incomplète : le champ est nommé** (V2-2). Le contrôle de Kesh (`:287`, `:310`) juge par
  `champ_vide_pour_qr` et rend, **pour le premier champ vide**, le refus `EMPTY` de D2 avec son champ
  (`creditor.postal_code`, `creditor.town`, `debtor.postal_code`, `debtor.town`) et un message qui dit l'adresse
  incomplète et le champ, par exemple « Adresse du client incomplète : la localité est vide. Complétez la fiche du
  client (NPA et localité sont exigés par la QR-facture). ». Un client **sans aucune** adresse garde
  `CLIENT_ADDRESS_MISSING` et `invoice-pdf-error-client-address-required`. Les clés `invoice-pdf-error-company-address-empty`
  et `invoice-pdf-error-client-address-empty`, sans usage après la story, sont retirées des quatre catalogues (vérifier
  par `git grep` qu'aucun test ni écran ne les lit).
- **D7 — Le pays** (V2-3) : `build_qrbill_inputs` contrôle le pays de chaque partie par
  `kesh_qrbill::is_iso_3166_alpha2` (`validation.rs:285`) **avant** `kesh-qrbill`, et rend `INVALID_COUNTRY` avec la partie
  et le code : « Le pays de la société (« SU ») n'est pas un code ISO valide. ».
- **D8 — L'avoir a sa clé** (V2-7) : `credit-note-pdf-error-contact-missing` ×4, repli égal à fr-CH, G9.
- **D9 — L'état i18n global** (V2-1) : **aucun test de `crates/kesh-api/src` n'appelle `init_error_i18n`** ; les libellés du
  catalogue ne s'assertent que dans les tests d'intégration (`crates/kesh-api/tests/`), qui l'initialisent tous. Contrôle
  automatique (AC13).

## Critères d'acceptation

1. **AC1 — Refus de champ, au serveur** (tests d'intégration, `crates/kesh-api/tests/invoice_pdf_e2e.rs`, catalogue chargé) :
   - client à la localité `Ve\u{2013}vey` : 400 `INVOICE_NOT_PDF_READY`, message fr-CH qui contient « localité du client »,
     « – » et « U+2013 », `details` **égal** à `{ "reason": "INVALID_CHARSET", "field": "debtor.town", "char": "–",
     "codepoint": "U+2013" }` ;
   - numéro de bâtiment du client `1\n2` (posé en SQL) : `details.char == "\n"`, le message contient `\n` écrit en deux
     caractères et `U+000A` (tue M7) ;
   - nom du client de **71** caractères (`NAME_MAX = 70`, `validation.rs:10` ; le nom d'un contact n'est pas borné à 70) :
     `TOO_LONG`, `max` 70, `got` 71 ;
   - nom de la société `"\u{AD}"` (SQL) : `EMPTY`, `creditor.name` ;
   - communication : numéro de facture `F–1` (posé en SQL, `UPDATE invoices SET invoice_number = …`, `validate_template`
     ne contrôlant pas le jeu de caractères) : `INVALID_CHARSET`, `unstructured_message` ;
   - pour chacun, le message ne contient ni `{`, ni `$`, ni `Champ`, ni un nom technique (tue M9 sur chaque raison).
2. **AC2 — La table complète, unitaire et sans état global.** Un test de `invoice_pdf_service.rs`, `mod tests`, couvre
   par une table écrite dans le test les onze champs atteignables × leurs raisons, `billing_information` et le bras de
   repli pour la complétude, et chaque `reason` sans champ. Il n'asserte que la clé choisie, les arguments, les `details`,
   et l'absence de noms techniques et de « Champ » dans le message **rendu sous repli** — le seul état de ce binaire (D9).
3. **AC3 — Inventaire clos** : les seize lignes de `git grep -n 'InvoiceNotPdfReady' -- crates` hors `errors.rs`, treize
   constructions — `credit_notes.rs:295` ; `invoice_pdf_service.rs:216, 226, 288, 324, 338, 497, 499, 502, 505, 508, 510,
   540` — et trois non-constructions (`invoice_pdf_service.rs:491`, `invoice_email.rs:1030`, `:1754`). Recomptées à
   l'étape 0 avec le tableau des routes ; verdict de chaque site au Change Log ; chaque construction a sa `reason`.
4. **AC4 — #964 nommé** (D6) : société au NPA `"\u{AD}"` → `EMPTY`, `creditor.postal_code`, message qui dit « adresse de
   la société incomplète » et « NPA » ; client à la localité `"\u{200B}"` → `EMPTY`, `debtor.town`, « adresse du client
   incomplète » et « localité » ; client sans adresse → `CLIENT_ADDRESS_MISSING`.
5. **AC5 — Le pays** (D7) : société au pays `SU` → `INVALID_COUNTRY`, `creditor.country`, `code` `SU`, message qui dit
   « pays de la société » et « SU » ; contact au pays hors liste (posé en SQL, cas d'avant la 0.17.4) → `debtor.country`.
6. **AC6 — Les refus sans champ** : contact absent, compte principal absent, données bancaires invalides : chacun sa
   `reason` et une phrase localisée, sans texte de `kesh-qrbill`.
7. **AC7 — L'écran reprend le message.** `pdfErrorMessage` rend `err.message` pour `INVOICE_NOT_PDF_READY` (fiches facture
   et avoir), et la clé fixe si le serveur envoie `message: ""` — un `message` **absent** est déjà remplacé par
   `parseErrorResponse` (`api-client.ts:283`) : le test de ce repli mocke `''`.
8. **AC8 — Quatre langues et G9.** Clés neuves ×4 (`parity_between_locales`), à côté du bloc `invoice-pdf-error-*`
   (C-e350-8) ; aucun sélecteur de pluriel. Replis Rust égaux à fr-CH, à variables nommées (D4), inscrits à G9 : une
   entrée neuve en fin de table pour `invoice_pdf_service.rs`, une pour `credit_notes.rs` (D8), et d'éventuelles clés
   d'`errors.rs` en fin de son bloc sous `// Story 35-2` (C-e350-8).
9. **AC9 — Tests existants réécrits sans perdre leur preuve** : `company_address_e2e.rs:583-589` (« Pays invalide » →
   AC5) et `:667-675` (« adresse de l'entreprise est vide » → AC4) ; `errors.rs:5715-5723` ; `pdf-error.test.ts:35-38`
   (sa preuve passe à `INVOICE_NOT_VALIDATED`) ; le commentaire de `frontend/tests/e2e/invoices.spec.ts:723-755` ; le
   commentaire de `company_address_e2e.rs:375-376` au présent (LOW de #964).
10. **AC10 — Manuel, CHANGELOG, contrat.** `user-manual.tex:1251` réécrit (le message nomme désormais le champ et le
    caractère ; le lot de rappels relève de la 35-2b, qui complète la phrase) ; PDF régénéré, contrôlé aplati. CHANGELOG
    `## [0.17.7] — Non publié`, *Corrigé*, refs #955 et refs #964 (C-e350-8). `docs/api-external.md` : ligne neuve
    `INVOICE_NOT_PDF_READY` juste après `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE` (`:1058`), avec les routes et **le
    tableau de D2** ; la 35-2b y ajoutera `message` pour le lot.
11. **AC11 — Captures** (thème clair, base jetable, `kesh-gate-logs/e352/captures/`) : téléchargement refusé (caractère,
    adresse incomplète, pays) ; modale d'envoi par e-mail ; rappel seul ; refigeage si le cas se monte sans détour.
12. **AC12 — E2E** : dans `invoice-qr-caracteres.spec.ts`, client à la localité `Ve–vey`, facture validée, *Télécharger le
    PDF* : le message contient « localité du client » et « U+2013 ». `createContactWithAddressViaApi`
    (`helpers/api-fixtures.ts:73-79`) reçoit un paramètre facultatif `city`, en dernier.
13. **AC13 — Garde de l'état global** (D9) : un test de `crates/kesh-api/tests/textes_coherents.rs` parcourt
    `crates/kesh-api/src/**/*.rs` et échoue si `init_error_i18n(` y apparaît ailleurs qu'à sa définition (`errors.rs`) et
    à `main.rs` ; sonde positive au Change Log (un appel ajouté dans un `mod tests` le fait rougir, puis retiré).

## Tests à écrire, et la preuve du rouge

Écrire d'abord, lancer sur le code de départ, garder le journal (`kesh-gate-logs/e352/rouges.log`) : AC1, AC4, AC5, AC6
(`invoice_pdf_e2e.rs`, `company_address_e2e.rs`) ; AC2 (`mod tests`) ; AC7 et AC9 (`pdf-error.test.ts`) ; AC13 (vert dès le
départ : sa sonde positive tient lieu de rouge) ; AC12. Un test vert sous `nextest` doit l'être sous `cargo test` :
`scripts/gate-ci.sh` au dernier commit de code.

**Mutations à jouer et consigner** : M1 `INVOICE_NOT_PDF_READY` retiré de `SERVER_RESOLVED_CODES` ; M2 un libellé de champ
retiré ; M3 `trim().is_empty()` rétabli à `:287` ; M4 idem à `:310` ; M7 `escape_debug` retiré (AC1, `\n`) ; M9 `t` au lieu
de `t_args`, appliquée à chaque raison (AC1) ; M10 `reason` omise pour un refus sans champ (AC6) ; M11 le contrôle du pays
retiré (AC5) ; M12 un appel d'`init_error_i18n` ajouté à un `mod tests` (AC13).

## Tâches

- [ ] Étape 0 : recompter l'inventaire, le tableau des routes et les lignes citées ; Change Log.
- [ ] Tests, rouges consignés.
- [ ] Serveur : contrat de D2, messages ×4, adresse (D6), pays (D7), clé de l'avoir (D8), `:338`, G9 ; garde AC13.
- [ ] Écran : `pdf-error.ts` (AC7).
- [ ] Tests existants (AC9) ; manuel, CHANGELOG, `api-external.md` (AC10) ; captures ; E2E.
- [ ] Gates : `test-fast.sh --ci` et `scripts/gate-ci.sh`, frontend complet, E2E complet au dernier commit de code, échecs
      nommés contre `docs/testing.md` § « Les échecs attendus ».

## Dev Notes

### Hors périmètre

- **Le lot de rappels** : 35-2b. **#944** : C-e350-1. **#942**, **#952** : ce qui est admis ne change pas.
- Le contrat de `kesh-qrbill` ne change pas.

### Fichiers communs — règle d'union C-e350-8

Avec la 35-3 : `errors.rs`, `messages.ftl` ×4, G9, `user-manual.tex`, `CHANGELOG.md`, `docs/api-external.md` (détail au
registre et au fichier d'epic). Avec la 35-2b, qui part de cette story fusionnée : `invoice_email.rs`, `user-manual.tex:1251`,
`api-external.md`.

### Ce qui doit rester vrai

- Le 400 et le code `INVOICE_NOT_PDF_READY` ne changent pas ; `details` s'ajoute.
- `INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`, `INVOICE_PDF_GONE` inchangés.
- Un PDF figé n'est pas régénéré. Fixtures synthétiques seulement (`Ve–vey`, `Muster SA`).

### Références

- Issues #955, #964 ; validations `kesh-gate-logs/e35-2-validation-p1.md`, `e35-2-validation-p2.md` ; fiche d'origine
  `35-2-refus-pdf-nomme-le-champ.md` (statut `split`).
- `CLAUDE.md` : § *Test Locally First* ; § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-12 — Née du découpage de la 35-2 à la validation P2 (C-e350-9) ; P2 remédiée ici (entrée « Validation P2 » de
  la fiche d'origine). Validation à poursuivre sur cette fiche.
