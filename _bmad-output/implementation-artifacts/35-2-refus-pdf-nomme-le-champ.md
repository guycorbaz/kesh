# Story 35-2 : le refus du PDF dit quel champ et quel caractère corriger ; une adresse sans rien de visible est dite incomplète (#955, #964)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story`. Release **0.17.7**, tête d'epic : à livrer avant la première facture client de Guy (au plus tard le
2026-11-30). **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`), d'où la branche de développement
partira. Registre du kickoff : `e35-choix-autonomes-e350.md` (C-e350-1, C-e350-3, C-e350-4) ; registre de la story :
`e35-choix-autonomes-e352.md`, à créer au premier choix. Validation à lancer par l'orchestrateur.*

## Story

En tant que titulaire qui émet sa première QR-facture, quand le PDF est refusé, je veux lire **quel champ** (nom, rue,
NPA, localité du client ou de la société, communication) et **quel caractère** corriger, au téléchargement comme dans
le rapport d'un lot de rappels, afin de corriger la fiche du client sans chercher à l'aveugle.

## Ce qui se lit au code (`release/0.17.6`)

**Le serveur sait, l'écran tait.**

- `kesh-qrbill` refuse avec un champ nommé : `QrBillError::InvalidCharset { field, codepoint }`, `FieldEmpty(field)`,
  `FieldTooLong { field, max, got }` (`crates/kesh-qrbill/src/types.rs:363-409`). Les noms de champ sont douze :
  `creditor.` et `debtor.` suivis de `name`, `line1`, `line2`, `postal_code`, `town`
  (`crates/kesh-qrbill/src/validation.rs:372-407`), puis `unstructured_message` et `billing_information` (`:469-474`).
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
  de `:334-335` affirme que `errors.rs` la résout : c'est faux, `build_response` reçoit `&msg` tel quel.

**Les quatre écrans.**

1. **Fiche facture et fiche avoir, téléchargement** : `pdfErrorMessage` (`frontend/src/lib/shared/utils/pdf-error.ts:39-46`)
   remplace le message du serveur par la clé fixe `invoice-pdf-error-invoice-not-pdf-ready` (`:18`), « La facture n'est
   pas prête pour la génération PDF. ». Appelants : `frontend/src/routes/(app)/invoices/[id]/+page.svelte:949`,
   `frontend/src/routes/(app)/credit-notes/[id]/+page.svelte:106`.
2. **Lot de rappels** : `classify_render_error` (`crates/kesh-api/src/routes/invoice_email.rs:1023-1036`) rend
   `BatchItemError::failed("INVOICE_NOT_PDF_READY")`, sans `details` ; `ReminderBatchReport.svelte:30` n'affiche que
   `reminderErrorLabel(f.errorCode)`, « Facture non imprimable en PDF ».
3. **Envoi de la facture par e-mail** : la branche `default` affiche `err.message` brut
   (`invoices/[id]/+page.svelte:864-866`) — la phrase technique d'aujourd'hui.
4. **Envoi d'un rappel seul** : l'erreur unitaire passe par `AppError`, même message.

**#964.** `build_qrbill_inputs` juge le NPA et la localité par `trim().is_empty()` : société
(`invoice_pdf_service.rs:287-292`) et client (`:310`). Un champ fait du seul U+00AD (ou U+200B) passe ce contrôle,
puis `kesh-qrbill` le refuse en `FieldEmpty` (« Champ creditor.postal_code vide (requis) ») au lieu du message
« adresse incomplète » de Kesh. La règle de `kesh-qrbill` est publique : `kesh_qrbill::champ_vide_pour_qr`
(`validation.rs:244-246`), déjà employée par la saisie de l'adresse de la société (`crates/kesh-api/src/address_input.rs:110`).

**Le modèle à suivre existe** : la correction de l'adresse de la société nomme déjà champ et caractère,
« adresse : le champ {label} contient un caractère que la QR-facture n'admet pas : « {c} » (U+XXXX) », avec
`c.escape_debug()` (`address_input.rs:131-137`).

## Décisions (C-e350-3, à reprendre au registre `e352` si la story les précise)

- **D1 — Le serveur résout le message, l'écran le reprend.** Chaque `InvoiceNotPdfReady` porte un message résolu par
  une clé Fluent, dans la langue de l'installation ; `INVOICE_NOT_PDF_READY` rejoint `SERVER_RESOLVED_CODES`
  (`pdf-error.ts:36`). La clé fixe `invoice-pdf-error-invoice-not-pdf-ready` reste le repli d'un `err.message` vide.
- **D2 — Les champs portent des `details`** : `{ "field": "debtor.town", "reason": "charset" | "empty" | "tooLong",
  "char": "–", "codepoint": "U+2013", "max": 35, "got": 40 }` (seuls les membres utiles à la raison). Les noms de champ
  sont ceux de `kesh-qrbill`, stables, pour un intégrateur ; le message les traduit en mots (« la localité du client »).
- **D3 — Forme côté Rust, laissée au développeur** : une variante neuve à charge structurée (par exemple
  `InvoiceQrFieldRefused { field, reason }`) ou une charge `details` ajoutée à `InvoiceNotPdfReady`. ⚠️ **Toute variante
  neuve s'ajoute à `classify_render_error`** (`invoice_email.rs:1023-1036`) : sinon elle tombe dans le bras final et le
  lot l'annonce `DATABASE_ERROR`, fausse panne (règle écrite au doc-comment de `:1010-1022`). Le test
  `classify_render_error_ne_deguise_pas_un_refus_en_panne` (`:1749`) s'étend à elle.
- **D4 — Le caractère s'affiche par `escape_debug`** (un U+00AD, un saut de ligne, une espace insécable restent lisibles),
  suivi de son point de code `U+XXXX`, comme `address_input.rs:131-137`.
- **D5 — Les refus sans champ** (IBAN, QR-IBAN, QRR, montant, devise, pays, payload) reçoivent une phrase localisée et
  générique ; le détail technique de `kesh-qrbill` part au journal (`tracing::warn!`), non au client.

## Critères d'acceptation

1. **AC1 — Un caractère refusé est nommé, au serveur.** Facture validée dont le client a pour localité
   `Ve\u{2013}vey` (fixture synthétique) : `GET /api/v1/invoices/{id}/pdf` rend 400 `INVOICE_NOT_PDF_READY`, un message
   (fr-CH) qui contient « localité du client », « – » et « U+2013 », et `details` égal à
   `{ "field": "debtor.town", "reason": "charset", "char": "–", "codepoint": "U+2013" }`. Même chose, côté société, pour
   le nom (`creditor.name`, « nom de la société ») et pour la communication (`unstructured_message`).
2. **AC2 — Vide et trop long.** `FieldEmpty` et `FieldTooLong` rendent leur champ en mots, `details.reason` à `empty` ou
   `tooLong`, `max` et `got` pour le second. Un test unitaire sur la fonction de correspondance couvre **les douze
   champs × trois raisons** par une table écrite dans le test : chaque champ a un libellé, aucun message ne contient
   `creditor.`, `debtor.`, `unstructured_message`, `billing_information` ni le mot « Champ ».
3. **AC3 — Inventaire clos des sites de `InvoiceNotPdfReady`**, tous résolus par une clé. Les seize lignes de
   `release/0.17.6` (`git grep -n 'InvoiceNotPdfReady' -- crates`, hors `errors.rs`), soit treize constructions :
   `credit_notes.rs:295` ; `invoice_pdf_service.rs:216, 226, 288, 324, 338, 497, 499, 502, 505, 508, 510, 540` ; et
   trois lignes qui n'en sont pas : `invoice_pdf_service.rs:491` (doc-comment), `invoice_email.rs:1030` (le
   classement) et `:1754` (un test). La story recompte cette liste au code de départ, et écrit au
   Change Log le verdict de chaque site. **`:338`** cesse d'envoyer une clé nue.
4. **AC4 — #964.** Le NPA et la localité de la société et du client se jugent par `kesh_qrbill::champ_vide_pour_qr`
   (`invoice_pdf_service.rs:288`, `:310`). Une société dont le NPA vaut `"\u{AD}"` (posé en SQL par le test) reçoit le
   message `invoice-pdf-error-company-address-empty` ; un client dont la localité vaut `"\u{200B}"`, le message
   `invoice-pdf-error-client-address-required`. Aucun des deux ne dit « Champ … vide ».
5. **AC5 — L'écran reprend le message.** `pdfErrorMessage` rend `err.message` pour `INVOICE_NOT_PDF_READY` (fiche
   facture et fiche avoir), et la clé fixe si `err.message` est vide. Le mock de `pdf-error.test.ts` reste un catalogue
   qui l'emporte sur le repli (`:1-19`) : le test qui l'exerce rougit sur le code d'aujourd'hui.
6. **AC6 — Le lot de rappels.** Un échec `INVOICE_NOT_PDF_READY` du lot porte des `details` (ceux de AC1 plus
   `message`, la phrase résolue) ; `ReminderBatchReport.svelte` affiche « Facture non imprimable en PDF — *phrase* » quand
   `details.message` est là, et le libellé seul sinon (`TOO_MANY_LINES`, `HEADER_OVERFLOW`, qui partagent ce code).
   Test Rust du lot et Vitest du composant.
7. **AC7 — Pas de fausse panne.** Un refus de champ dans un lot de rappels sort `INVOICE_NOT_PDF_READY`, jamais
   `DATABASE_ERROR` (D3).
8. **AC8 — Quatre langues.** Chaque clé neuve existe dans `fr-CH`, `de-CH`, `it-CH`, `en-CH` (`parity_between_locales`,
   `crates/kesh-i18n/src/loader.rs:771`) ; aucun sélecteur de pluriel n'est employé (sinon inscription à
   `SELECTEURS_RESOLUS_COTE_SERVEUR`). Les `messages.ftl` portent les libellés des douze champs.
9. **AC9 — Tests existants qui épinglent une phrase.** `crates/kesh-api/tests/company_address_e2e.rs:583-589` attend
   « Pays invalide » ; `errors.rs:5715-5723` attend le message tel quel. Ils suivent les nouvelles phrases, sans
   perdre ce qu'ils prouvent. Le commentaire de `company_address_e2e.rs:375-376` (« va être recalée … (33-12) ») passe
   au présent (LOW de #964).
10. **AC10 — Manuel.** `docs/manual/fr/user-manual.tex:1251` dit aujourd'hui que le téléchargement échoue « sans nommer
    le champ » et que le lot affiche « Facture non imprimable en PDF » (issue #955) : réécrit pour dire ce que l'écran
    montre désormais, avec un exemple synthétique. PDF régénéré, contrôlé aplati
    (`pdftotext … | tr '\n' ' ' | tr -s ' '`). CHANGELOG, section `## [0.17.7] — Non publié`, *Corrigé*, refs #955 et
    refs #964.
11. **AC11 — Captures** (thème clair, base de démonstration jetable, `kesh-gate-logs/e352/captures/`) : le message du
    téléchargement refusé sur la fiche facture ; la ligne du rapport d'un lot de rappels ; l'erreur dans la modale
    d'envoi par e-mail.
12. **AC12 — E2E.** Dans `frontend/tests/e2e/invoice-qr-caracteres.spec.ts`, un scénario : client à la localité
    `Ve–vey` (créé par l'API, fixture synthétique), facture validée, clic sur *Télécharger le PDF* : le message affiché
    contient « localité du client » et « U+2013 ». C'est le seul test qui voie la phrase traverser HTTP jusqu'à l'écran.

## Tests à écrire, et la preuve du rouge

Écrire d'abord, puis lancer sur le code de départ, et garder le journal (`kesh-gate-logs/e352/rouges.log`) :

- `invoice_pdf_service.rs`, `mod tests` : la table de AC2 (rouge : « Champ debtor.town … »), et AC4 sur
  `build_qrbill_inputs` (rouge : passe le contrôle de Kesh).
- `crates/kesh-api/tests/invoice_pdf_e2e.rs` : AC1 et AC4 de bout en bout (rouge : pas de `details`, message
  technique).
- `invoice_email.rs`, `mod tests` : AC6 et AC7 (rouge : `details` à `None`).
- `frontend/src/lib/shared/utils/pdf-error.test.ts` : AC5 (rouge : la clé fixe).
- un Vitest de `ReminderBatchReport.svelte` (AC6 ; le composant n'a pas de test aujourd'hui).
- E2E de AC12 (rouge sur le binaire de départ).

**Mutations à jouer et consigner** : M1 `INVOICE_NOT_PDF_READY` retiré de `SERVER_RESOLVED_CODES` ; M2 un libellé de
champ retiré ; M3 `trim().is_empty()` rétabli à `:288` ; M4 idem à `:310` ; M5 la variante neuve (si D3 en crée une)
retirée de `classify_render_error` ; M6 `details.message` non lu par le rapport du lot ; M7 `escape_debug` retiré.

## Tâches

- [ ] Étape 0 : recompter l'inventaire de AC3 et les lignes citées sur la branche de départ ; écrire au Change Log
      toute ligne qui a bougé.
- [ ] Tests de la section précédente, rouges consignés.
- [ ] Serveur : correspondance des refus de champ, `details`, clés Fluent ×4, `champ_vide_pour_qr` aux deux sites,
      `:338`, lot de rappels (AC1–AC4, AC6–AC8).
- [ ] Écran : `pdf-error.ts`, `ReminderBatchReport.svelte` (AC5, AC6).
- [ ] Tests existants mis à jour (AC9) ; manuel et CHANGELOG (AC10) ; captures (AC11) ; E2E (AC12).
- [ ] Gates : `test-fast.sh --ci`, frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`), E2E complet
      au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs attendus ».

## Dev Notes

### Hors périmètre

- **#944** (ligne verticale, ciseaux, « Payable by »…) : C-e350-1.
- **#942** (admettre plus de typographies) et **#952** (contrôler le jeu de caractères à la saisie d'un contact) : la
  story nomme le caractère, elle ne change pas ce qui est admis.
- Le **contrat** de `kesh-qrbill` ne change pas : ses variantes portent déjà le champ.

### Fichiers communs avec la 35-3 (C-e350-4)

`crates/kesh-api/src/errors.rs` (ici, le bras `InvoiceNotPdfReady` et `build_response` ; la 35-3 ajoute un bras
`DbError`), les quatre `messages.ftl` (clés distinctes), `user-manual.tex` (ici l. 1251 ; la 35-3 vers l. 1169). Ne pas
réordonner ni reformater ces fichiers hors des passages touchés.

### Ce qui doit rester vrai

- Le 400 et le code `INVOICE_NOT_PDF_READY` ne changent pas (intégrateurs, `docs/api-external.md` si le code y figure :
  ajouter `details`).
- Les codes `INVOICE_TOO_MANY_LINES_FOR_PDF`, `INVOICE_PDF_HEADER_OVERFLOW`, `INVOICE_PDF_GONE` gardent leur traitement.
- Un PDF figé n'est pas régénéré : la story ne touche que le refus.
- Le texte des messages ne contient aucune donnée réelle ; les fixtures sont synthétiques (`Ve–vey`, `Muster SA`).

### Références

- Issues #955, #964 (et son commentaire) ; classement `kesh-gate-logs/epic35-classement-bugs.md` § 7.3.
- Story 33-5 (`address_input.rs`, modèle du message) ; Story 33-12a (jeu de caractères, `invoice-qr-caracteres.spec.ts`).
- `CLAUDE.md` : § *Pattern batch — FailedProposal per-proposal* ; § *Recette visuelle* ; § *Le prompt d'une passe doit
  NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

## Change Log

- 2026-11-12 — Fiche rédigée (kickoff de l'Epic 35). Validation non lancée.
