# Story v0171-correctifs : la 0.17.1 — la QR-facture à lignes CR+LF (#854), la quantité (#866, #878) et les dates (#871, #879)

Status: ready-for-dev

*Amendée le 2026-11-03 sur demande du Project Lead : périmètre élargi à **#878** (champ de saisie de la quantité d'une facture cliente) et **#879** (dates ISO des écrans clients) — issues ouvertes par l'orchestrateur sur les relevés de la première rédaction ; C-v0171-7 et C-v0171-10 sont remplacés par C-v0171-13 et 14. Fiche rédigée le 2026-11-03, en autonomie (Sonnet 5.5), worktree `/home/gcorbaz/devel/kesh-0171`, branche
`release/0.17.1` partie de `910e5d5a` (le merge de la v0.17.0 sur `main`). Release **corrective 0.17.1**, sans
migration. Registre : `v0171-choix-autonomes.md` (C-v0171-1 à 18). Clé courte : `v0171`. Issues : **#854** (bug, P2),
**#866**, **#871**, **#878**, **#879** (bugs, P4), en `refs` seulement dans les commits ; la fermeture se porte sur la PR de
release. Les numéros de ligne sont ceux de `910e5d5a`.*

## Story

En tant que Comptable qui dépose dans le dossier d'import le PDF scanné d'une facture communale, je veux que Kesh en
lise la QR-facture comme celle de n'importe quel autre émetteur ; et, en tant que lecteur d'une facture fournisseur, je
veux lire les quantités et les dates dans la forme suisse (`1`, `29.01.2026`), non dans celle du serveur (`1.0000`,
`2026-01-29`).

## Origine et contrainte

- **#854 (P2)** — recette de la 0.17.0-rc1 (2026-10-31), puis constat sur la 0.17.0 installée (2026-11-03) : deux
  factures communales scannées, section paiement présente, entrent « Aucune QR-facture exploitable », alors que la QR-facture
  d'un autre émetteur, scannée elle aussi, est lue. Le second rendu à ≈ 171 dpi de la v017-5 n'y change rien. Cause
  établie le 2026-11-03 par l'orchestrateur sur les pièces réelles (hors dépôt) : voir § *Ce que le code dit*.
- **#866 (P4)** — la fiche d'une facture fournisseur affiche la quantité brute du serveur (`1.0000`).
- **#878 (P4)** — le champ de saisie de la quantité d'une ligne de facture cliente en brouillon recopie la valeur brute (`1.0000`).
- **#879 (P4)** — les écrans clients (factures, avoirs, relances, échéances ; listes et fiches) affichent les dates en ISO.
- **#871 (P4)** — la confirmation d'annulation de règlement d'une facture fournisseur afficherait la date du lot en ISO.
  ⚠️ **Prémisse à rectifier** : voir § 8.
- **Aucune migration, aucune donnée touchée.** Du code de lecture d'un payload et des formats d'affichage. L'inventaire
  des tables est inchangé ; `kesh_version_min_required` reste à 0.17.0 ; **le retour à la 0.17.0 est possible sans
  restauration** (aucune écriture nouvelle, aucun schéma nouveau). Contrainte du Project Lead du 2026-10-30
  (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`), valable pour l'installation de production : Kesh 0.17.0 y tient
  des livres réels. Aucun essai de cette story ne se joue sur le NAS ni sur des pièces réelles : fixtures synthétiques
  seulement, bases `#[sqlx::test]` jetables.
- **Hors périmètre** (release, faits par l'orchestrateur) : bump de version Cargo, macros des manuels
  (`kesh-style.sty`), tag, PDF des manuels (régénérés par le développeur pour les seuls textes modifiés ici).

## Ce que le code dit (relevé le 2026-11-03, `910e5d5a`)

### #854 — le préfiltre strict

1. **Le préfiltre** — `crates/kesh-api/src/qr_decode.rs:31` : `const SPC_HEADER: &str = "SPC\n0200";` ; `:261` :
   `if p.starts_with(SPC_HEADER) {` dans `first_spc` (`:259-275`). Un payload `SPC\r\n0200\r\n…` n'a pas ce préfixe :
   `first_spc` le saute, passe au suivant, rend `Ok(None)` ; `decode_pages` (`:172-`) en conclut « aucune QR-facture »
   (`qr: none`), et l'import affiche « Aucune QR-facture exploitable ». `first_spc` est le **seul** appelant de
   `parse_spc_payload` de la chaîne d'import (`decode_pages`, `:172-`, pour les PDF ; `decode_spc_from_image_bytes`, `:277`, pour les images).
2. **Le parseur tolère CR+LF** — `crates/kesh-qrbill/src/parser.rs:94-95` : `let lines: Vec<&str> =
   text.split('\n').map(|l| l.trim_end_matches('\r')).collect();`. Aucun test existant ne l'exerce (`grep -n 'r\\n'
   crates/kesh-qrbill/src/parser.rs` ne rend que le commentaire de la ligne 94) : **la tolérance du parseur est une
   affirmation sans preuve**. L'orchestrateur l'a constatée par exécution sur les trois payloads réels ; la story la fige
   par un test synthétique.
3. **Aucun `\r` ne peut subsister dans un champ restitué, par construction** : chaque ligne est retaillée de son `\r`
   final (`:95`) avant tout usage ; `creditor_iban` passe par `validate_iban`, `reference` par `validate_qrr`/`SCOR`,
   `creditor.*`, `currency`, `unstructured_message`, `billing_information` sont des `lines[i]` retaillées. Un `\r`
   **interne** à une ligne (rare, hors norme) ne serait pas retiré ; hors périmètre (C-v0171-3). Côté écran, les
   champs de `ScannedQrBill` sont recopiés tels quels en base (`imported_supplier_invoices.qr_*`) puis relus : aucune
   découpe de texte intermédiaire.
4. **Les messages de test** : `qr_decode.rs:444` (`assert!(payload.starts_with(SPC_HEADER))`) et `:973`
   (`qr("SPC\n0200\n1\nPAS-UN-IBAN")`) supposent LF. Le premier dépend du **type** de `SPC_HEADER` (voir AC-1).

### #854 — inventaire clos : tous les sites qui testent ou découpent un payload SPC par `\n`

Méthode : `grep -rnE 'SPC\\n|SPC_HEADER|split\(.\\n.\)|\.lines\(\)' crates/kesh-api/src crates/kesh-qrbill/src
crates/kesh-import/src --include=*.rs`, relevé site par site, puis `grep -rn 'SPC' frontend/src` (trois fichiers). Les
`.lines()`/`split('\n')` qui ne portent pas sur un payload SPC sont classés en bloc (dernière ligne du tableau).

| Site | Ce qu'il fait | Verdict |
|---|---|---|
| `kesh-api/src/qr_decode.rs:31, :261` (`SPC_HEADER`, `first_spc`) | écarte tout payload qui ne commence pas par `SPC\n0200` | **à corriger** (AC-1) |
| `qr_decode.rs:7, :237` (doc-comments) | écrivent « commençant par `SPC\n0200` » | **à corriger** (la phrase devient fausse) |
| `qr_decode.rs:444` (test `decode_spc_from_png_roundtrip`) | `payload.starts_with(SPC_HEADER)` | **à adapter** si le type de la constante change |
| `qr_decode.rs:973` (test `image_de`, `Faux::SpcInvalide`) | payload LF fabriqué, volontairement invalide | conforme : rien à changer (le cas CR+LF s'ajoute à côté, AC-2 d) |
| `kesh-qrbill/src/parser.rs:95` | `split('\n')` + `trim_end_matches('\r')` | **tolère CR+LF** (à prouver, AC-3) |
| `routes/supplier_invoices.rs:1095` (`scan_qr_supplier_invoice`, chemin du navigateur) | `parse_spc_payload(&req.spc_text)` **sans préfiltre** | tolère CR+LF via le parseur ; **à prouver** (AC-4) |
| `frontend/src/lib/features/supplier-invoices/supplier-invoice-scan.ts:28-29` | `jsQR(...)` rend `result.data` tel quel, aucune découpe ni test de préfixe | tolère CR+LF : le texte part à `scan-qr` inchangé |
| `frontend/src/lib/features/imported-supplier-invoices/error-label.ts:36` | libellé de `INVALID_SPC_PAYLOAD` | hors classe : texte seul |
| `inbox_import.rs:799, :807` | appelle `decode_spc_from_pdf_bytes` / `decode_spc_from_image_bytes` | tolère : passe par `first_spc` (aucune découpe propre) |
| `kesh-qrbill/src/generator.rs:63` (`lines.join("\n")`) et ses tests (`:163-222`, `split('\n')`) | **émission** du payload par Kesh, en LF | hors chemin d'import (justifié) : la norme SIX admet LF ; Kesh émet LF, inchangé |
| `kesh-qrbill/src/validation.rs:222` | commentaire sur `\n` comme séparateur du payload | hors chemin d'import (justifié) |
| `routes/invoice_pdf_service.rs:461` (`split_lines`) | découpe les lignes d'une **adresse** de facture | hors classe : pas un payload SPC |
| `kesh-import/src/csv/parser.rs:1644, :2181, :2216` ; `admin_backup/*`, `auto_backup.rs`, `config.rs`, `exports/csv_tables.rs`, `document_storage.rs`, `services/*`, `errors.rs`, `routes/admin.rs`, `routes/mcp_envois.rs` (`.lines()`) | CSV, sauvegarde, configuration, tests de texte | hors classe : aucun ne lit un payload SPC |

**Les fixtures de test de l'import** (`crates/kesh-api/tests/inbox_import_e2e.rs:309-335`,
`supplier_invoice_scan_qr_e2e.rs`) fabriquent des payloads **LF** : elles n'ont jamais pu rougir sur CR+LF — c'est la
raison pour laquelle le défaut a passé la v017-5, dont les fixtures venaient du générateur de Kesh, en LF.

### #866 — la quantité brute

5. **Les sites qui affichent la quantité d'une ligne** — `grep -rn 'quantity' frontend/src --include=*.svelte` rend
   cinq lectures, trois d'**affichage** :
   `routes/(app)/supplier-invoices/[id]/+page.svelte:790` (`{line.quantity}` — l'objet de #866),
   `routes/(app)/invoices/[id]/+page.svelte:1200` (`{l.quantity}`, facture cliente),
   `routes/(app)/credit-notes/[id]/+page.svelte:174` (`{line.quantity}`, avoir) ; et deux d'**édition** (un champ
   `<Input>`) : `lib/features/supplier-invoices/SupplierInvoiceForm.svelte:696` (déjà lisible : `formLineFromResponse`
   passe par `readableAmount`, `supplier-invoice-form.ts:246`), `lib/components/invoices/InvoiceForm.svelte:113, :800`
   (facture cliente, **#878**, § 11 : `quantity: l.quantity` recopié **brut** — `1.0000` dans le champ à l'ouverture d'un brouillon).
6. **Le helper existe** — `readableAmount(raw, minDecimals)` (`supplier-invoice-form.ts:213`) : `1.0000` → `1` pour
   `minDecimals = 0`, `33.3333` gardé, `2.5000` → `2.5`, valeur illisible rendue telle quelle. C'est celui du
   formulaire de saisie (v017-4b, AC-23). **Aucun helper n'est créé** (C-v0171-6). `formatSwissAmount` ne convient pas :
   décimales fixes, apostrophes de milliers (un montant, pas une quantité).
7. **La liste des factures fournisseur n'affiche pas de quantité** (colonnes : numéro, fournisseur, dates, statut,
   total) ; les MCP rendent déjà une forme unique (#812, 0.17.0).

### #871 — les dates ISO

8. **La prémisse de l'issue est inexacte sur le code de `910e5d5a`** : la confirmation d'annulation de règlement
   formate déjà la date du lot — `routes/(app)/supplier-invoices/[id]/+page.svelte:231` :
   `{ batch: batch.id, date: formatSwissDate((batch.confirmedAt ?? '').slice(0, 10)) }` ; de même `:336` (dévalidation)
   et `:142` (« Facture réglée le … »). C'est le finding **B2-4** de la revue P2 de la v017-4b, remédié **avant** la
   release, avec son test — `supplier-settlement-page.test.ts:202-205` : `toContain("02.07.2026")` et
   `not.toContain("2026-07-02")`. #871 a été ouverte le 2026-11-03 à 08:32 (« relevé pendant la remédiation de la revue
   P1 de la v017-4b »), la remédiation P2 l'a réglée : **l'issue décrit un état antérieur au code publié.** Le
   développement le **vérifie par exécution** (AC-9) avant de conclure ; si une autre confirmation montre bien l'ISO
   (le texte de l'issue dit « la date du lot »), il la corrige. Le commentaire à l'issue est demandé à l'orchestrateur
   (C-v0171-8).
9. **Les dates ISO qui, elles, restent affichées brutes sur les écrans des factures fournisseur** (inventaire clos,
   `grep -rnE '\{[a-zA-Z.]*(invoiceDate|dueDate|requestedExecutionDate|confirmedAt)' frontend/src --include=*.svelte`
   puis lecture) :

   | Site | Valeur | Verdict |
   |---|---|---|
   | `routes/(app)/supplier-invoices/+page.svelte:144` | `inv.invoiceDate` (liste) | **à corriger** |
   | `routes/(app)/supplier-invoices/+page.svelte:145` | `inv.dueDate ?? '—'` | **à corriger** |
   | `routes/(app)/supplier-invoices/[id]/+page.svelte:569` | `invoice.invoiceDate` (fiche) | **à corriger** |
   | `routes/(app)/supplier-invoices/[id]/+page.svelte:571` | `invoice.dueDate ?? '—'` | **à corriger** |
   | `routes/(app)/payment-batches/+page.svelte:239` | `b.requestedExecutionDate` (liste des lots) | **à corriger** (C-v0171-9) |
   | `routes/(app)/payment-batches/[id]/+page.svelte:97` | `batch.requestedExecutionDate` | **à corriger** |
   | `routes/(app)/payment-batches/[id]/+page.svelte:104` | `batch.confirmedAt` (horodatage naïf `2026-…T…`) | **à corriger** : `slice(0, 10)` d'abord |
   | `…/supplier-invoices/[id]/+page.svelte:142, :231, :336` ; `lib/features/supplier-invoices/supplier-invoice-form.ts:194` ; `lib/features/imported-supplier-invoices/imported-supplier-invoices.types.ts:65` | `formatSwissDate` déjà appliqué | conforme |
   | `SupplierInvoiceForm.svelte:550-588`, `payment-batches/+page.svelte:145`, `[id]/+page.svelte:151`, `supplier-invoices/[id]/+page.svelte:851` | `<input type="date">` | hors périmètre (champ de saisie : le navigateur affiche la date dans sa langue) |

### #879 — dates ISO des écrans clients (inventaire clos)

10. `grep -rnE '(\{|date: )[a-zA-Z.!?]*\.(date|dueDate|paidAt|pdfFrozenAt|emailedAt|settledOn|sentAt|lastReminderAt|cancelledAt)\b'`
    sur les huit fichiers d'écrans clients, relu ligne à ligne, rend **seize sites** d'affichage brut (`ReminderHistory` et
    `InvoiceSettlements` sont des composants des fiches). Les horodatages (`paidAt`, `pdfFrozenAt`, `emailedAt`, `sentAt`,
    `lastReminderAt`, `cancelledAt`) passent déjà par `.slice(0, 10)` ; il reste à les envelopper de `formatSwissDate`.

    | Site | Valeur | Verdict |
    |---|---|---|
    | `routes/(app)/invoices/+page.svelte:413` | `inv.date` (liste des factures) | **à corriger** |
    | `invoices/+page.svelte:494` | `deleteTarget?.date` dans « Confirmer la suppression de la facture du { $date } » | **à corriger** (la phrase est celle d'une confirmation) |
    | `invoices/[id]/+page.svelte:1135, :1139` | `invoice.date`, `invoice.dueDate ?? '—'` | **à corriger** |
    | `invoices/[id]/+page.svelte:1156, :1164, :1172` | `paidAt`, `pdfFrozenAt` (`data-testid="invoice-pdf-frozen-at"`), `emailedAt` (`invoice-emailed-at`), tous `.slice(0, 10)` | **à corriger** |
    | `invoices/due-dates/+page.svelte:449, :450, :467` | `inv.date`, `inv.dueDate ?? '—'`, `inv.paidAt … .slice(0, 10) : '—'` | **à corriger** |
    | `invoices/reminders/+page.svelte:345, :358` | `inv.dueDate` ; `inv.lastReminderAt.slice(0, 10)` dans « dernier le { $date } » | **à corriger** |
    | `lib/features/reminders/ReminderHistory.svelte:56, :70` | `r.sentAt.slice(0, 10)` ; `r.cancelledAt!.slice(0, 10)` dans « Annulé le { $date } » | **à corriger** |
    | `credit-notes/+page.svelte:64` ; `credit-notes/[id]/+page.svelte:153` | `cn.date`, `creditNote.date` | **à corriger** |
    | `lib/features/invoices/InvoiceSettlements.svelte:75` | `s.settledOn` (règlements d'une facture) | **à corriger** |

    **Exceptions closes, justifiées** : (i) les champs `<input type="date">` (`InvoiceForm`, `SettleInvoiceDialog.svelte:295`,
    `WriteOffDialog.svelte:210`, `ManualReminderDialog.svelte:126`, filtres) : le navigateur les affiche dans sa langue ;
    (ii) les valeurs **envoyées à l'API** (`createCreditNote({ date: today })`, `settledOn`, `sentAt`, `recordManualReminder`,
    `invoiceDate={…}` passé à un dialogue comme borne) et les `toISOString().slice(0, 10)` de calcul ; (iii) `DunningPausedBadge`
    et les `dunningPausedAt` (test de nullité, aucun affichage) ; (iv) les écrans qui ne sont **pas** ceux des factures clientes, des
    avoirs, des relances et des échéances : `journal-entries` (`+page.svelte:551`, `[id]/+page.svelte:590`), `settings/fiscal-years`
    (`:459-460, :636-642`), `bank-import` (`[id]/+page.svelte:139, :144, :179`, `BankImportList.svelte:38`,
    `BankImportUpload.svelte:544, :799`), le rapprochement (`ReconciliationProposals.svelte:480`, `ManualMatchModal.svelte:173`,
    `TransactionSplitModal.svelte:234`, `LinkSettlementModal.svelte:205, :290, :350`) — leurs tests figent d'ailleurs l'ISO
    (`ReconciliationProposals.test.ts:398`, `LinkSettlementModal.test.ts:287`). **Issue demandée** pour ceux-là (C-v0171-18).

    **Tests existants qui figent l'ISO sur ces sites** (à mettre à jour, avec la raison au Change Log) :
    `routes/(app)/invoices/[id]/invoice-pdf-page.test.ts:171-173` (`"2026-03-02"` ⇒ `"02.03.2026"`) et `:220-222`
    (`"2026-04-01"` ⇒ `"01.04.2026"`). Un grep des `.test.ts` et des specs E2E des huit fichiers (assertions de texte portant une date ISO)
    est joint au Change Log ; `tests/e2e/invoice-send-email.spec.ts` ne lit que la visibilité de `invoice-emailed-at`.

### #878 — champs de saisie de la quantité (inventaire clos)

11. `grep -rn 'quantity' frontend/src --include=*.svelte` : **deux** champs de saisie. (a) `SupplierInvoiceForm.svelte:696`
    (`bind:value={line.quantity}`) : **conforme** — les brouillons relus passent par `formLineFromResponse` → `readableAmount(line.quantity, 0)`
    (`supplier-invoice-form.ts:246`, v017-4b AC-23). (b) `lib/components/invoices/InvoiceForm.svelte:800`
    (`<Input … bind:value={line.quantity} />`), alimenté par **deux** sites qui recopient la valeur brute du serveur : `initLines`
    (`:113`, ouverture d'un brouillon) et `reloadInvoice` (`:610`, rechargement après un conflit de version) ; les deux autres sites
    (`:125`, `:410`, `:424`) posent `'1'` (conformes). Les avoirs n'ont pas de formulaire (créés depuis la facture, `createCreditNote`).
    **Aller-retour** : à l'enregistrement, `stripUiKey` envoie la chaîne du champ telle quelle (`:555`) ; le serveur accepte `"1"`
    (c'est déjà la valeur des lignes neuves) ; `readableAmount` ne tronque rien (`33.3333` et `0.0125` gardés, seuls les zéros
    de fin tombent) : **la valeur numérique envoyée est la même, la précision à quatre décimales est conservée quand l'utilisateur
    en a saisi**. Le calcul du total (`computeLineTotal`, `:446`) et le contrôle `qty.lte(0)` (`:482`) lisent un `Big` : insensibles à la forme.

## Acceptance Criteria

**AC-1 — Le préfiltre retient les deux séparateurs.** `first_spc` retient un payload dont les deux premières lignes sont
`SPC` et `0200`, que le séparateur soit `\n` ou `\r\n` (`SPC\n0200` ou `SPC\r\n0200`), et le passe tel quel à
`parse_spc_payload` (parseur inchangé). Une seule définition de la règle (une constante-tableau ou une fonction
`est_payload_spc(&str) -> bool`), utilisée par `first_spc` et par le test de `:444`. Un `\r` seul (`SPC\r0200`), une version
autre que `0200` (`SPC\r\n0100`) et un texte qui ne commence pas par `SPC` ne sont **pas** retenus. Les doc-comments de
`:7` et `:237` disent « `SPC` puis `0200`, lignes séparées par LF ou CR+LF ».

**AC-2 — Les tests du préfiltre et du décodage, sur des payloads engendrés par Kesh.**
 (a) `first_spc` : le payload de `build_payload(&donnees_spc_test())` réécrit en CR+LF (`replace('\n', "\r\n")`) est retenu ;
     `creditor_iban == "CH9300762011623852957"`, `amount == Some(100.00)`.
 (b) `decode_spc_from_image_bytes` de bout en bout : un PNG QR (`qrcodegen_image`, `qr_decode.rs:412`) du même payload CR+LF
     rend `Some(..)` avec les mêmes valeurs — **c'est le test qui aurait rougi sur #854**.
 (c) **Non-régression LF** : le test existant `decode_spc_from_png_roundtrip` reste vert, inchangé en son corps.
 (d) `decode_pages` (banc `jouer`, `qr_decode.rs:985` ; `enum Faux`, `:951`) : un cas `Faux::SpcCrlf` (QR CR+LF) à `Temps::Premier` rend `Some(..)`
     **sans second rendu** (suite d'appels `[(0, Premier)]`) ; un cas `Faux::SpcCrlfInvalide` (`SPC\r\n0200\r\n1\r\nPAS-UN-IBAN`)
     rend `Err(InvalidIban/InvalidSpcPayload)` comme son jumeau LF.
 (e) Négatifs du préfiltre (AC-1) : `SPC\r0200…`, `SPC\r\n0100…`, `HTTP://…` rendent `Ok(None)`.

**AC-3 — Aucun `\r` résiduel, au parseur.** Un test de `kesh-qrbill/src/parser.rs` parse un payload **structuré**
(`structured_payload()`, `:270`) réécrit en CR+LF, **avec** les lignes 32 à 34 (information de facturation, AltPmtInf) et un
`\r\n` final : tous les champs de `ScannedQrBill` (`creditor_iban`, `creditor.name/street_or_line1/building_or_line2/
postal_code/town/country`, `currency`, `reference`, `unstructured_message`, `billing_information`) sont **égaux** à ceux
du même payload en LF, et aucun ne contient `\r` (`!s.contains('\r')`). Un second cas : référence `QRR` (le checksum se
calcule sur la valeur retaillée).

**AC-4 — Le chemin du navigateur.** `POST /api/v1/supplier-invoices/scan-qr` avec `spcText` en CR+LF rend 200 et les mêmes
champs que la version LF (`supplier_invoice_scan_qr_e2e.rs`, helper `spc_payload`, `:104`) ; test unitaire jumeau dans
`scan_qr_tests` (`supplier_invoices.rs:1101-`) sur `ScanQrResponse::from_scanned(parse_spc_payload(..))`.

**AC-5 — L'import de bout en bout.** Un test d'intégration (`inbox_import_e2e.rs`) dépose une image PNG dont le QR est un
payload CR+LF (variante de `qr_invoice_png_with`, `:309-335`, qui reçoit le séparateur) et vérifie que le document entre
avec `qr == "found"` (ou la forme du fichier : les champs IBAN, montant, créancier lus), puis le **contraire** de l'ancien
comportement : pas de `qrError`, pas de « sans QR ». Même test en LF inchangé.

**AC-6 — La quantité (affichage, #866).** La fiche d'une facture fournisseur, la fiche d'une facture cliente et la fiche d'un avoir affichent
la quantité via `readableAmount(raw, 0)` : `1.0000` → `1`, `2.5000` → `2.5`, `33.3333` → `33.3333`, `1250.0000` →
`1250`. Aucun helper nouveau (C-v0171-6).

**AC-7 — Les dates fournisseur (#871).** Chaque site « à corriger » du tableau du § 9 passe par `formatSwissDate` : une date
`2026-01-29` s'affiche `29.01.2026` ; `confirmedAt` (`2026-01-29T10:15:00`) s'affiche `29.01.2026` ; une échéance absente
reste `—`. Aucune date ISO ne subsiste sur ces écrans (grep de contrôle, AC-8).

**AC-8 — Tests Vitest qui figent l'affichage**, un par site corrigé (mutation : retirer le helper du site rougit le test) :
 - fiche fournisseur : ligne `1.0000` → texte `1` (cellule `quantity`), **et** `not.toContain('1.0000')` ; dates de la fiche
   `29.01.2026` / `not.toContain('2026-01-29')` ;
 - liste fournisseur (`supplier-invoices-list-page.test.ts`) : date de facture et échéance en forme suisse ; échéance nulle `—` ;
 - fiche cliente (`invoices/[id]`) et fiche d'avoir (`credit-notes/[id]`) : quantité `1` ;
 - lots de paiement (liste et fiche) : date d'exécution et `Confirmé le` en forme suisse ;
 - **#878** (`InvoiceForm.test.ts`) : un brouillon dont une ligne porte `1.0000` ouvre un champ `1` ; après un conflit de version et
   « Recharger » (`reloadInvoice`), le champ porte encore `1` ; `33.3333` reste `33.3333` ; l'enregistrement sans toucher la quantité envoie
   `quantity: "1"` (corps de la requête lu) et un total inchangé ; saisir `2.5` l'envoie `"2.5"` ;
 - **#879**, un test par écran (listes et fiches) : `inv.date`/`dueDate` des factures (liste, fiche, échéances), `paidAt`, `pdfFrozenAt`, `emailedAt`
   de la fiche, `lastReminderAt` (relances), `sentAt` et `cancelledAt` (`ReminderHistory`), `date` des avoirs (liste, fiche), `settledOn`
   (`InvoiceSettlements`), la date de la confirmation de suppression d'une facture : chacun affiche `JJ.MM.AAAA` et `not.toContain` l'ISO ;
   échéance nulle ⇒ `—`. Mettre à jour `invoice-pdf-page.test.ts:171-173, :220-222`.
 Grep de contrôle, à joindre au Change Log, **valeur** et non formulation : `grep -rnE '\{(inv|invoice|b|batch)\.(invoiceDate|dueDate|requestedExecutionDate|confirmedAt)\b'
 frontend/src --include=*.svelte` ne rend plus **que** des lignes d'écrans clients — plus aucune après #879, voir AC-12 ; sonde positive :
 le même motif sur `910e5d5a` en rend **dix** (les sept des écrans fournisseur et trois d'écrans clients). Pour #879, voir AC-12.

**AC-9 — #871 vérifiée, non supposée.** Le développement rejoue, sur une installation de dev jetable (base `kesh_v0171`),
l'annulation du règlement d'une facture fournisseur figurant dans un lot confirmé, et **capture le texte** de la
confirmation (`window.confirm` intercepté par Playwright ou Vitest). Attendu : `29.01.2026`. Si c'est exact, l'issue est
déclarée **déjà corrigée en 0.17.0** (par B2-4) dans le rapport, et l'orchestrateur la commente ; sinon le site fautif est
corrigé, avec son test.

**AC-10 — Manuel et CHANGELOG** : voir § *Manuel* et § *CHANGELOG*.

**AC-11 — Le champ de saisie de la quantité (#878).** À l'ouverture d'un brouillon de facture cliente (`initLines`) et après un
rechargement sur conflit de version (`reloadInvoice`), le champ quantité porte `readableAmount(l.quantity, 0)` : `1.0000` ⇒ `1`,
`2.5000` ⇒ `2.5`, `33.3333` ⇒ `33.3333`, `0.0125` ⇒ `0.0125`. Aller-retour : ouvrir un brouillon puis l'enregistrer sans toucher la
quantité envoie `"1"` (jamais `"1.0000"`) et ne change pas le total ; saisir `33.3333` l'envoie tel quel. Le formulaire fournisseur
(conforme) ne change pas.

**AC-12 — Les dates des écrans clients (#879).** Chacun des seize sites « à corriger » du § 10 passe par `formatSwissDate` (les
horodatages après `.slice(0, 10)`) : `2026-03-02` s'affiche `02.03.2026` ; l'échéance absente reste `—` ; les phrases « dernier le », « Annulé le »,
« Confirmer la suppression de la facture du » portent la forme suisse. Les `data-testid` (`invoice-pdf-frozen-at`, `invoice-emailed-at`) sont
conservés. **Grep de contrôle** : le motif du § 10, rejoué sur les huit fichiers après correction, ne rend que des lignes enveloppées de
`formatSwissDate` ou rangées aux exceptions (i) à (iii) ; la liste (iv) est celle d'une issue, pas d'une tolérance. Sonde positive : le même
motif sur `910e5d5a` rend les seize lignes du tableau.

## Tests et mutations

Tests rouges d'abord (sur `910e5d5a`), puis le correctif, puis les mutations.

| # | Test | Ce qui doit rougir |
|---|---|---|
| T1 | `first_spc` CR+LF retenu (AC-2 a) | rouge avant correctif |
| T2 | `decode_spc_from_image_bytes` sur PNG QR CR+LF (AC-2 b) | rouge avant correctif |
| T3 | LF inchangé (AC-2 c) | vert avant et après |
| T4 | `decode_pages` : CR+LF au premier temps (AC-2 d) | rouge avant correctif |
| T5 | négatifs du préfiltre (AC-2 e) | vert avant et après ; rouge sous M2 |
| T6 | parseur : égalité LF/CR+LF et aucun `\r` (AC-3) | vert avant et après ; rouge sous M3 |
| T7 | `scan-qr` CR+LF (AC-4) | vert avant et après |
| T8 | import PNG CR+LF (AC-5) | rouge avant correctif |
| T9 | Vitest fiche fournisseur : quantité, dates (AC-6, 8) | rouge avant correctif |
| T10 | Vitest liste fournisseur : dates (AC-8) | rouge avant correctif |
| T11 | Vitest fiches cliente et d'avoir : quantité (AC-8) | rouge avant correctif |
| T12 | Vitest lots de paiement : dates (AC-8) | rouge avant correctif |
| T13 | Vitest `InvoiceForm` : quantité à l'ouverture, après rechargement, aller-retour (AC-11) | rouge avant correctif (sauf l'aller-retour `33.3333`, vert avant et après) |
| T14 | Vitest écrans clients : dates, un test par écran (AC-12) | rouge avant correctif ; `invoice-pdf-page.test.ts` mis à jour |

**Mutations** (sous le verrou de gate ; après une restauration par `cp`/`mv`, **toucher le fichier** — cargo garde le
binaire muté) :
- **M1** — rétablir le préfiltre strict (`starts_with("SPC\n0200")` seul) : **T1, T2, T4, T8 rougissent**. C'est la
  mutation exigée par le mandat.
- **M2** — accepter `SPC\r0200` (ou tout préfixe `SPC`) : T5 rougit.
- **M3** — retirer `.map(|l| l.trim_end_matches('\r'))` du parseur : T6, T2 (champ avec `\r` ⇒ IBAN invalide) rougissent.
- **M4** — retirer `readableAmount` d'un site de quantité (trois mutations, une par fiche) : T9 / T11 rougissent.
- **M5** — retirer `formatSwissDate` d'un site de date (une par site corrigé) : le test du site rougit seul.
- **M6** (#878) — retirer `readableAmount` de `reloadInvoice` (`:610`) seul : le test « après conflit » rougit, celui de l'ouverture reste
  vert ; puis de `initLines` (`:113`) seul : l'inverse. **M6-bis** — remplacer `readableAmount(l.quantity, 0)` par `Number(l.quantity).toString()` : le test de précision doit rougir ; il vise donc une
  valeur que `Number` altère (`0.12345678`, huit décimales) en plus de `33.3333`. Si M6-bis ne rougit pas, ajouter le cas.
- **M7** (#879) — retirer `formatSwissDate` du seul `.slice(0, 10)` de `invoice.emailedAt` (`:1172`), puis de `ReminderHistory.svelte:70` : chaque
  test d'écran rougit seul (un test unique par écran ne couvrirait pas le second site d'une même page : le test de la fiche lit les
  trois horodatages, celui de l'historique lit `sentAt` **et** `cancelledAt`).

Sondes positives : chaque contrôle « vide » (grep d'AC-8) prouve d'abord sa sensibilité sur `910e5d5a` (pathspec `crates/*/src/*`, non
`crates/*/src`).

## Tâches

1. **Tests rouges** T1, T2, T4, T8, T9–T12 sur `910e5d5a` (T3, T5–T7 d'abord verts : les noter).
2. `qr_decode.rs` : la règle du préfixe (AC-1), doc-comments ; adapter `:444`.
3. Parseur : T6 seulement (aucune modification de production attendue ; si T6 révèle un `\r` résiduel, **le corriger** et
   le consigner au Change Log — c'est alors le second défaut de #854).
4. `inbox_import_e2e.rs` : le séparateur en paramètre de `qr_invoice_png_with` ; T8. `supplier_invoice_scan_qr_e2e.rs` : T7.
5. Frontend : `readableAmount` aux trois fiches ; `formatSwissDate` aux sept sites fournisseur et lots ; **`readableAmount` aux deux sites
   d'`InvoiceForm` (#878)** ; **`formatSwissDate` aux seize sites des écrans clients (#879)** ; tests Vitest ; mise à jour de
   `invoice-pdf-page.test.ts`.
6. AC-9 : essai de la confirmation d'annulation du règlement, texte capturé au Change Log.
7. Manuel FR (§ *Manuel*), PDF régénéré par le développeur (`make fr` dans `docs/manual/`), CHANGELOG.
8. Captures d'écran, **thème clair** : fiche fournisseur (quantité, dates), liste fournisseur, fiche d'un lot de paiement ;
   jointes au Dev Agent Record. Base de démonstration jetable.
9. Gates : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh`
   (sous `mem-guard`, verrou de gate), `cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit &&
   npm run build`, E2E complet au **dernier commit de code** (7 à 10 échecs attendus, `docs/testing.md`). Base remise à zéro
   avant le gate complet.

## Manuel

`docs/manual/fr/user-manual.tex`, § *Importer des factures fournisseur* (`:1965-2030`) :
- Le badge « Aucune QR-facture exploitable » (`:1983-1989`) énumère les causes (« aucun QR, un autre QR, un QR illisible, un scan de mauvaise
  qualité ») : **ajouter** que la 0.17.1 lit les QR-factures dont les lignes sont séparées par un retour chariot
  (CR+LF), que la norme admet et que certains émetteurs utilisent ; un document entré avant cette version avec la mention
  « Aucune QR-facture exploitable » **alors que sa section de paiement porte un QR** se fait relire par le geste déjà
  décrit au § *Faire relire le QR d'un document* (`:2010-2022`) : *Télécharger*, *Retirer de la liste*, redéposer,
  *Importer le dossier*.
- Le § *Faire relire le QR d'un document* attribue la lecture manquée à la résolution (« Sur des scans de test, ce second rendu lit des QR-factures
  que le premier manquait ; une pièce réelle peut encore y échapper ») : **vrai** (le second rendu reste en place) mais incomplet ;
  aucune phrase n'est fausse, rien ne se retire.
- Les formats d'affichage (`1`, `29.01.2026`) : le manuel ne décrit ni la forme de la quantité ni celle des dates des écrans
  fournisseur (`grep -nE '1\.0000|AAAA-MM-JJ|jj\.mm' docs/manual/fr/user-manual.tex` : rien d'applicable, `:2229` est la mention
  « Clos le JJ.MM.AAAA » d'un autre écran) : **rien à corriger** pour #866, #871, #878 et #879 (idem pour les dates des écrans clients : aucune n'est décrite au format ISO).
- `docs/manual/{de,en,it}/` : `ls` ne rend que `README.md` (aucun `.tex`) : rien à traduire.
- Le manuel d'administration (`admin-manual.tex`, *Passer à la 0.17.0*) : la **procédure de mise à jour** vers la 0.17.1 (sans migration,
  retour possible) relève de la préparation de la release, faite par l'orchestrateur.
- **Le PDF est à contrôler** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -F` de la phrase neuve.

## CHANGELOG

Nouvelle section `## [0.17.1] — Non publié`, au-dessus de la 0.17.0, une intro d'une phrase — « La 0.17.1 est une version corrective :
**aucune migration**, aucune donnée réécrite, **le retour à la 0.17.0 est possible sans restauration**. » — puis `### Corrigé` :
- **La QR-facture de certains émetteurs est enfin lue** ([#854](…/issues/854)) : une QR-facture dont les lignes sont séparées
  par CR+LF (que la norme SIX admet) était écartée avant lecture et entrait « Aucune QR-facture exploitable ». *Un document entré ainsi avant
  la 0.17.1 se fait relire : le télécharger, le retirer de la liste, le redéposer.*
- **La quantité d'une ligne s'affiche `1`, non `1.0000`**, sur la fiche d'une facture fournisseur, d'une facture cliente et d'un avoir
  ([#866](…/issues/866)).
- **Les dates des factures fournisseur et des lots de paiement s'affichent `29.01.2026`**, non `2026-01-29` ([#871](…/issues/871)) —
  la confirmation d'annulation d'un règlement l'affichait déjà ainsi depuis la 0.17.0 *(à ajuster selon AC-9)*.
- **Le champ quantité d'une facture cliente en brouillon s'ouvre à `1`, non `1.0000`** ([#878](…/issues/878)) ; la précision saisie
  (`33.3333`) est conservée.
- **Les dates des factures clientes, des avoirs, des relances et des échéances s'affichent `29.01.2026`**, non `2026-01-29`, en liste comme
  en fiche ([#879](…/issues/879)). D'autres écrans (journal, exercices, relevés bancaires, rapprochement) gardent la forme ISO jusqu'à une
  prochaine version.

`refs #854`, `refs #866`, `refs #871`, `refs #878`, `refs #879` aux commits ; **aucun mot-clé de fermeture** (la PR de release porte `closes #866`, `closes #871`, `closes #878`, `closes #879` ;
`#854` attend le constat de Guy sur ses deux pièces, comme le disait le commentaire du 2026-10-31).

## Dev Notes

- **Jamais de pièce réelle.** Les fixtures sont les payloads de `build_payload` réécrits (`replace('\n', "\r\n")`) ; le QR est
  encodé par `rxing` (`qrcodegen_image`) ou le générateur de Kesh. La question « rxing préserve-t-il `\r` dans un QR en
  mode octet ? » est tranchée **par T2**, sur le QR synthétique ; si T2 rougit pour une autre raison que le préfiltre, le dire.
- **Pourquoi la v017-5 n'a rien vu** : ses fixtures et son banc `jouer` (`Faux::Spc`) ont tous pour source `build_payload`, en LF.
  La classe « payload valide mais non produit par Kesh » n'avait aucun représentant — AC-2 d/AC-5 le sont.
- **Le second rendu de la v017-5 reste** : utile pour d'autres scans, non en cause ici (commentaire de l'issue du 2026-11-03).
- Ne pas toucher `generator.rs` : Kesh émet LF (norme : LF ou CR+LF).
- **Gate complet** (`kesh-api` touche `inbox_import`) ; aucune migration : pas de P5, P6, P7.

## Change Log

- 2026-11-03 — fiche créée (Sonnet 5.5, autonomie). Prémisses vérifiées au code : #854 (cause confirmée, lignes citées) ; #866
  (confirmée, trois sites d'affichage dont deux hors fournisseur) ; #871 (**prémisse fausse sur le code publié**, voir § 8).
  Registre C-v0171-1 à 12. Prochaine : validation P1 (Opus, une lentille R+F, `v0171-validate-prompt-p1.md`).
- 2026-11-03 (même jour, avant toute validation) — **amendement de périmètre** sur demande du Project Lead : #878 et #879 intégrées. Relevés
  ajoutés : deux sites d'`InvoiceForm` (`:113`, `:610`) pour #878 ; seize sites d'affichage des écrans clients pour #879 (§ 10), avec huit
  exceptions closes et une liste d'écrans hors périmètre (issue demandée) ; deux tests existants qui figent l'ISO
  (`invoice-pdf-page.test.ts`). AC-11, AC-12, T13, T14, M6, M6-bis, M7 ajoutés ; C-v0171-7 et 10 remplacés par C-v0171-13 et 14 ; registre
  porté à C-v0171-1 à 18. La validation P1 n'avait pas rendu de rapport : rien à reprendre.
