# Story v017-5 : lire le QR d'une facture QR scannée

Status: ready-for-dev

*Fiche rédigée le 2026-10-31, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0176`, branche
`story/v017-5-qr-scanne` partie de `8e126cfd` (tête de `release/0.17.0`). Release **0.17.0**, candidate suivante
(décision de Guy du 2026-10-31). Registre : `v017-5-choix-autonomes.md` (C-v017-5-1 à C-v017-5-11). Clé courte :
`v0175`. Issue : **#854** (bug, P2), en `refs` seulement dans les commits ; la fermeture se porte sur la PR (question
Q3). Les numéros de ligne sont ceux de `8e126cfd` ; ceux des dépendances, ceux de `~/.cargo/registry`
(`pdfium-render-0.8.37`, `rxing-0.7.1`, `printpdf-0.7.0`).*

⛔ **Aucune pièce réelle** : ni les PDF du Project Lead (`~/kesh/`, son dossier de projet, Paperless, `~/kesh-test/`) ni
rien qui en dérive n'entre dans le dépôt, un commit, un test, un journal versé ou une capture. Toutes les fixtures sont
synthétiques (C-v017-5-5).

## Story

En tant que Comptable qui importe ses factures fournisseur **scannées** (archives PDF/A d'un gestionnaire de documents),
je veux que Kesh lise le QR de leur section paiement comme celui d'une facture née numérique, que l'écran dise « aucun QR
lu » plutôt que d'affirmer que le document n'en porte pas, et pouvoir faire relire une pièce déjà importée, afin de ne
pas ressaisir à la main l'IBAN, la référence et le montant qu'un QR présent sur la pièce porte déjà.

## Origine

- **#854 (P2)**, recette de la `0.17.0-rc1` par Guy le 2026-10-31 sur des pièces réelles (non jointes, dépôt public) :
  sur quatre PDF importés, le QR d'une facture d'assurance est lu ; ceux de **deux factures communales**, dont la section
  paiement est présente, ne le sont pas — classées « sans QR-code ». Ces PDF sont la version archivée PDF/A (OCR) de
  **scans** : le QR y est une image matricielle à la résolution du scan.
- **Prémisse de l'issue corrigée** : #854 parle d'un QR qui échappe « à la résolution de rendu **retenue** ». Le code
  n'en retient **aucune** : la page est rendue à **72 dpi**, la borne de 2000 px n'étant jamais atteinte (§ *Ce que le
  code dit*, prémisse 1). L'orchestrateur le commentera sur l'issue.

## Données à garder, migration — ce que la story ne fait pas

**Aucune migration, aucune table, aucun bump** de `kesh_version_min_required` ni de version de crate (contrainte du
Project Lead du 2026-10-30, `kesh-gate-logs/prompts/v017-donnees-a-garder.md`). L'inventaire des tables ne change pas :
une sauvegarde de la 0.16.x continue de s'importer dans la 0.17.0. **Rien n'est relu d'office** : un document importé
avant la 0.17.0 garde ses données (ou leur absence) tant que l'utilisateur ne le **redépose** pas après l'avoir retiré de
la liste (AC-8) ; alors seules les colonnes QR **vides** d'une ligne réactivée s'écrivent (C-v017-5-9). Une ligne qui a
déjà un QR, une ligne `to_complete` ou `completed` n'est jamais réécrite. Toute recette se joue sur une installation
**jetable** (image Docker de la branche) avec les fixtures synthétiques — jamais sur la base du NAS.

## Ce que le code dit (relevé le 2026-10-31, `8e126cfd`)

1. **Le rendu est à 72 dpi.** `qr_decode.rs:163-165` : `PdfRenderConfig::new().set_maximum_width(cfg.max_dimension as
   i32).set_maximum_height(cfg.max_dimension as i32)` — aucun facteur d'échelle, aucune taille cible. Dans
   `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs`, sans l'un ni l'autre, l'échelle vaut 1,0 (`:771-773`,
   « Set default scale to 1.0 if neither dimension is specified ») ; les maxima ne font que la **réduire** (`:799`,
   `:813`). Une page A4 (595 × 842 pt) sort en 595 × 842 px ; `max_dimension: 2000` (`qr_decode.rs:46-53`) n'agit sur
   aucune page courante.
2. **Le décodeur** : `decode_qr_from_image` (`qr_decode.rs:84-94`) passe la luminance à
   `rxing::helpers::detect_multiple_in_luma`, qui pose `TryHarder` (`rxing-0.7.1/src/helpers.rs:241`), le binariseur
   hybride (`:244`) et **aucun** `PossibleFormats` : tous les formats sont essayés.
3. **Le chemin image** (PNG, JPEG) décode à la résolution native (`qr_decode.rs:116-133`), sous les bornes 10 000 px et
   256 Mio (`:31-34`). Un scan déposé en image n'est pas rééchantillonné.
4. **Aucun test du chemin PDF ne tourne.** `libpdfium.so` est absente de la station et de la CI ; seule l'image Docker la
   porte (`Dockerfile:30-38`, `chromium/7920`, somme SHA-256 vérifiée). L'unique test PDF, `decode_spc_from_pdf_single_page`
   (`qr_decode.rs:301-313`), est `#[ignore]` et lit `tests/fixtures/qr_invoice.pdf`, **absente** (`ls
   crates/kesh-api/tests/fixtures/` : `keshbackup-0.14.0-format1.keshbackup`, `spa-stub`).
   `import_accepts_pdf_without_qr` (`tests/inbox_import_e2e.rs:640-658`) accepte `none` **ou** `unreadable /
   PDF_RENDER_ERROR` pour cette raison.
5. **Le rapport et le badge** : `QrStatus::Absent` sérialisé `none` (`inbox_import.rs:114-125`) ; à l'écran, la mention
   du rapport `imported-supplier-invoices-report-no-qr` (`+page.svelte:237`) et le badge
   `imported-supplier-invoices-badge-no-qr` (`+page.svelte:310-314`, `AttachDocumentPicker.svelte:148`), affiché pour
   toute ligne **sans données QR stockées** — QR absent **ou** inexploitable (`+page.svelte:293`, `hasQr(row)`).
6. **La réactivation ne relit pas.** Le court-circuit du doublon par empreinte (`inbox_import.rs:476-537`) précède le
   décodage (`:539-576`). Un document `discarded` redéposé est réactivé (`reactivate_to_complete`,
   `imported_supplier_invoices.rs:480-499`) et le rapport dit le statut **stocké** (`inbox_import.rs:517-523`). Un
   document `to_complete` ou `completed` redéposé est un `DUPLICATE` (`:532-535`).
7. **Le décodage est synchrone** dans `process_one_file` (`async`, `inbox_import.rs:540-550`, aucun `spawn_blocking`) —
   défaut voisin, non traité (I1).
8. **Les sept colonnes QR** obligatoires vont ensemble (`chk_imported_si_qr_all_or_nothing`, migration
   `20261015000001_imported_documents_without_qr.sql`) ; `has_qr()` = `creditor_iban.is_some()`.

**Cause retenue (C-v017-5-1)** : **H1, le rendu à 72 dpi.** Le QR d'une QR-facture mesure 46 mm ; pour un payload de
facture réelle (QR de version 10 à 15, 57 à 77 modules), 72 dpi donnent **1,7 à 2,3 px par module** : un QR vectoriel
peut s'y lire, un QR scanné — flou, bruité, rééchantillonné — ne le peut plus. H1 est **établie au code, non prouvée** :
la preuve est T4 (AC-2), jouée avant tout correctif.

## Critères d'acceptation

### La cause, prouvée avant le correctif

**AC-1 — Fixtures synthétiques.** Un générateur de test (C-v017-5-5) produit, de façon déterministe (graine fixe) :
(a) trois PDF d'une page A4 portant chacun une image JPEG en niveaux de gris d'une QR-facture scannée à **150, 200 et
300 dpi**, flou gaussien de 0,1 mm, bruit uniforme ± 12 niveaux, papier à 240 et encre à 30 (sur 255) ; (b) un PDF de
même facture **sans** QR (section paiement sans QR, même chaîne) ; (c) les mêmes images, en mémoire, pour les tests
d'hôte. Le payload, construit par `kesh_qrbill::build_payload`, fait **au moins 280 caractères** et donne un QR de
**version ≥ 12** (assertions dans le générateur). Les quatre PDF sont versionnés sous `crates/kesh-api/tests/fixtures/`
(`qr-scan-150dpi.pdf`, `qr-scan-200dpi.pdf`, `qr-scan-300dpi.pdf`, `scan-sans-qr.pdf`), leur taille déclarée au Dev
Agent Record ; un test `#[ignore]` les régénère et compare les octets. Aucune donnée réelle : IBAN d'exemple SIX
(`CH9300762011623852957`, déjà employé par `qr_decode.rs:211`, ou le QR-IBAN d'exemple SIX pour la référence QRR), noms
et adresses fictifs.

**AC-2 — La reproduction, rouge d'abord.** Au **premier commit** de la story (fixtures, tests, et la seule extraction de
`render_scale` **rendant encore 1,0**), sans aucun correctif :
- **T4** (pdfium, `#[ignore]`, joué en local, C-v017-5-6) : `decode_spc_from_pdf_bytes` rend `Ok(None)` pour les trois
  `qr-scan-*.pdf`, et `Ok(Some(_))` pour un PDF engendré par Kesh (`kesh_qrbill::generate_qr_bill_pdf*`, QR vectoriel —
  témoin que le chemin fonctionne) ; `Ok(None)` pour `scan-sans-qr.pdf`.
- **T1** (hôte) : les tests de `render_scale` (AC-3) sont **rouges**.
- **Branche d'arrêt** : si T4 **lit** une fixture `qr-scan-*` avec le code d'avant, H1 est réfutée : le développement
  s'**arrête**, consigne la sortie et le signale à l'orchestrateur — aucun correctif à l'aveugle. De même si, après
  AC-3 et AC-4, une fixture du périmètre reste illisible : arrêt et signalement (la découpe pdfium d'une bande à plus
  haute résolution, C-v017-5-2 (c), est alors une décision de l'orchestrateur, non du développement).
- La sortie de T4 au commit rouge est **copiée** au Dev Agent Record (commande, version de `libpdfium`, résultat par
  fixture).

### Le remède

**AC-3 — Résolution de rendu.** `qr_decode.rs` expose une fonction pure documentée `render_scale(width_pt: f32,
height_pt: f32, max_dimension: u16) -> f32` = `min(max_dimension / width_pt, max_dimension / height_pt,
MAX_RENDER_DPI / 72)` avec `const MAX_RENDER_DPI: f32 = 300.0` ; `decode_spc_from_pdf_bytes` l'applique à chaque page
par `scale_page_by_factor(render_scale(page.width().value, page.height().value, cfg.max_dimension))`, et garde
`set_maximum_width/height(cfg.max_dimension)`. Tests d'hôte (T1), valeurs exactes à 10⁻³ près :
- A4 portrait (595,276 × 841,89 pt), borne 2000 → échelle 2000 / 841,89 ≈ **2,3756** (≈ 171 dpi) ; sortie ≤ 2000 px
  sur les deux côtés ;
- A4 paysage → même échelle (borne par la largeur) ;
- section paiement seule, 210 × 105 mm (595,28 × 297,64 pt) → 2000 / 595,28 ≈ **3,3598** (≈ 242 dpi, la borne de
  largeur l'emporte) ;
- A6 portrait (297,64 × 419,53 pt) → **300 / 72** ≈ 4,1667 (le plafond dpi l'emporte sur 2000 / 419,53 ≈ 4,767) ;
- A3 portrait (841,89 × 1190,55 pt) → 2000 / 1190,55 ; sortie ≤ 2000 px ;
- page minuscule (10 × 10 pt) → 300 / 72 (jamais au-delà de 300 dpi).
`DecodeConfig::default()` est **inchangé** (20 pages, 2000 px).

**AC-4 — Décodeur.** `decode_qr_from_image` passe par `detect_multiple_in_luma_with_hints` avec `PossibleFormats =
{QR_CODE}` et `TryHarder = true` (C-v017-5-3). Test d'hôte **T2** : chacune des trois images synthétiques (150, 200,
300 dpi), rééchantillonnée (`image::imageops::resize`, filtre `Triangle`) aux dimensions que donne `render_scale` pour
une A4 (1414 × 2000 px), est **lue** — IBAN, montant et référence égaux au payload. **Témoin de cause** (même test,
assertion contraire) : les mêmes images rééchantillonnées en **595 × 842 px** ne sont **pas** lues ; son message
d'échec dit « si ce témoin lit le QR, la cause H1 n'est plus établie : relire la fiche v017-5 ». **T3** (contrôle) : les
mêmes images à leur résolution native, par `decode_spc_from_image_bytes` (JPEG), sont lues — le décodeur n'est pas en
cause sur le chemin image. **Échelon supplémentaire** (`FilteredImageReader`, puis découpe de la bande inférieure) :
écrit **seulement** si une image de T2 n'est pas lue sans lui et l'est avec lui, et alors couvert par une mutation
(M5) ; sinon non écrit, et le Dev Agent Record le dit.

**AC-5 — Le correctif, prouvé par pdfium.** Au dernier commit de code, T4 (même commande qu'à AC-2) rend `Ok(Some(_))`
pour les trois `qr-scan-*.pdf` (IBAN, montant, référence égaux au payload) et pour le PDF vectoriel de Kesh, `Ok(None)`
pour `scan-sans-qr.pdf`. Sortie copiée au Dev Agent Record. Le test cassé `decode_spc_from_pdf_single_page`
(`qr_decode.rs:301-313`) lit désormais une fixture qui existe (`qr-scan-300dpi.pdf` ou le PDF vectoriel engendré) —
correction mécanique (C-v017-5-6).

**AC-6 — Coût, mesuré et plafonné.** Un banc `#[ignore]` (pdfium, compilé en `--release`) mesure, au commit rouge
(rendu à 72 dpi) **et** au dernier commit de code : (a) le temps de rendu + décodage **par page** de chaque fixture
(médiane de 5 passages) ; (b) le temps du **pire document admis** — un PDF synthétique de **20 pages** sans QR
(engendré par le banc, non versionné), soit 20 pages × (rendu + échelle complète du décodeur) ; (c) la mémoire résidente
de pointe du processus de banc (`/usr/bin/time -v`, « Maximum resident set size »). **Plafonds**, sur la station :
médiane **≤ 1,0 s par page**, pire document **≤ 20 s**, pointe de mémoire **≤ 128 Mio de plus** qu'au commit rouge. Par
construction, chaque bitmap rendu reste sous **2000 × 2000 px × 4 octets = 16 Mio** (borne inchangée). Les chiffres
avant / après vont au Dev Agent Record. Un plafond dépassé se corrige en réduisant le travail (échelon supplémentaire
limité à la première et à la dernière page, ou retiré) — **jamais** en relâchant une borne (C-v017-5-4).

### Ce que l'écran dit

**AC-7 — « Aucun QR lu ».** Les valeurs (clés inchangées, C-v017-5-7) :

| clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `imported-supplier-invoices-badge-no-qr` | Aucun QR lu | Kein QR-Code gelesen | Nessun codice QR letto | No QR code read |
| `imported-supplier-invoices-report-no-qr` | aucun QR lu | kein QR-Code gelesen | nessun codice QR letto | no QR code read |

Les replis de `+page.svelte:237`, `:312` et `AttachDocumentPicker.svelte:148` égalent la valeur fr-CH (règle des gardes
i18n du dépôt). La valeur d'API `qr: "none"` ne change pas. Aucun autre texte n'est touché (`supplier-invoices-scan-no-qr`,
scan côté navigateur, dit déjà « Aucun QR-code détecté »).

### Relire une pièce déjà importée

**AC-8 — Relecture à la réactivation.** Quand un fichier déposé a l'empreinte d'une ligne `discarded` **sans données
QR** (`!existing.has_qr()`), Kesh le **décode** (même fonction que l'import, extraite pour les deux chemins — DRY) avant
la transaction de réactivation, puis :
- QR lu **et** exploitable (`check_scanned_fits_columns` vert) → `reactivate_to_complete_with_qr` (C-v017-5-9) : une
  seule instruction, garde `status = 'discarded' AND creditor_iban IS NULL`, colonnes projetées comme
  `NewImportedSupplierInvoice::from_scanned` ; rapport `qr: "present"` ;
- QR inexploitable (`InvalidSpcPayload`, `InvalidIban`, `PdfRender`, colonnes) → réactivation **sans** QR
  (`reactivate_to_complete`, inchangée) ; rapport `qr: "unreadable"` et `qrError` (mêmes codes qu'à l'import) ;
- aucun QR → réactivation sans QR ; rapport `qr: "none"` ;
- `ImageDecode` (impossible à octets égaux sauf changement de la crate `image`) → réactivation sans QR, rapport
  `qr: "none"` — **jamais** un refus d'une pièce déjà archivée ;
- zéro ligne réactivée (course) → conduite actuelle (`DUPLICATE`, `inbox_import.rs:527-530`).
Une ligne `discarded` **avec** données QR est réactivée **sans décodage** (conduite actuelle, rapport `present`). La trace
`imported_supplier_invoice.reactivated` (même transaction) porte en plus `"qr"` (la valeur du rapport) et `"qr_reread"`
(`true` si le fichier a été décodé). Aucun fichier n'est réarchivé (même empreinte, même `storage_path`).

**AC-9 — Rien d'autre n'est relu ni réécrit.** Un document `to_complete` redéposé reste un `DUPLICATE` ; un document
`completed` redéposé aussi ; aucune ligne n'est relue au démarrage, à la migration ou à la liste. Tests T7 (ci-dessous).

### Textes

**AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, § *Documents à traiter*, l. 1838-1880), `.tex` **et**
PDF régénéré :
- l. 1851 : « avec la mention \emph{aucun QR lu} ou \emph{QR illisible} » ;
- l. 1857-1861, le badge : « \textbf{Aucun QR lu}~: Kesh n'a lu aucun QR exploitable sur le document --- il n'en porte
  pas, son QR n'a pas pu être lu (un scan de mauvaise qualité peut échapper à la lecture), ou il n'a pas pu être exploité
  (illisible, montant hors des limites admises, valeur trop longue). » — la suite (« Le document est alors accepté sans
  les données du QR… ») inchangée ;
- *Retirer de la liste* (l. 1874-1876) : « … le déposer de nouveau le fait revenir dans la liste~; s'il n'avait aucun QR
  lu, Kesh le relit alors. » ;
- un paragraphe **neuf**, *Faire relire le QR d'un document*, après la liste des quatre gestes : depuis la 0.17.0, Kesh
  lit mieux le QR d'un document **scanné** ; un document entré plus tôt avec « Aucun QR lu » n'est **pas** relu de
  lui-même ; pour le faire relire : *Télécharger*, *Retirer de la liste*, déposer le fichier téléchargé dans le dossier
  d'import, *Importer le dossier* — le document revient avec les données du QR s'il est lu, sans doublon ; un document
  dont le QR a été lu, ou dont la facture est créée, n'est jamais relu ; si le QR reste illisible, l'IBAN, la référence
  et le montant se saisissent à la main.

**AC-11 — Manuel d'administration** (`admin-manual.tex`), `.tex` et PDF : § *Passer à la 0.17.0* (l. 2481-2491), une
phrase : les documents importés avant la mise à jour ne sont pas relus ; renvoi au manuel utilisateur (*Faire relire le
QR d'un document*). § *Intégrations par l'API (0.17.0)* : la réactivation d'un document sans QR le relit, et son rapport
dit `present`, `none` ou `unreadable` selon cette lecture. § *Documents à traiter : import depuis un dossier*
(l. 879-886) : une phrase — chaque page d'un PDF est rendue à la résolution la plus haute que permet la borne de 2000
pixels par côté, au plus 300 dpi (environ 170 dpi pour une page A4).

**AC-12 — `docs/api-external.md`** l. 778 : `none` devient « aucun QR lu » ; la dernière phrase (« Un document
**réactivé** … porte `present` ou `none` selon ce qui a été stocké ») est remplacée par la conduite d'AC-8 (« … est relu
s'il n'avait aucune donnée QR : `present`, `none` ou `unreadable` selon cette lecture ; sinon `present` »).

**AC-13 — CHANGELOG**, `[0.17.0] — Non publié`, *Corrigé* : « **Le QR d'une facture QR scannée est lu à l'import**
([#854](https://github.com/guycorbaz/kesh/issues/854)) : Kesh rendait chaque page d'un PDF à 72 dpi, trop peu pour le QR
d'un document scanné ; il la rend désormais à environ 170 dpi pour une page A4 (au plus 300 dpi, sous la même borne de
2000 pixels par côté). L'écran dit « Aucun QR lu » plutôt que « Sans QR-code ». Un document importé auparavant n'est pas
relu de lui-même : retiré de la liste puis redéposé, il l'est (manuel utilisateur, *Faire relire le QR d'un
document*). » Dans un commit : `refs #854` seulement.

## Tâches

1. **Commit rouge** (AC-1, AC-2) : générateur et fixtures ; extraction de `render_scale` rendant 1,0 (aucun changement
   de comportement) ; T1 (rouge), T2 (rouge sur la lecture à 1414 × 2000 si le décodeur actuel n'y suffit pas, sinon
   vert — le dire), T3, T4 (`#[ignore]`), banc AC-6 ; `printpdf = "0.7"` en `dev-dependencies` de `kesh-api` (aucun
   paquet neuf au `Cargo.lock` — le vérifier par `git diff Cargo.lock`) ; T4 et le banc joués en local, sorties
   copiées.
2. **Correctif** (AC-3, AC-4, AC-5, AC-6) : `render_scale`, `PossibleFormats`, échelon supplémentaire seulement sur
   preuve ; T4 et banc rejoués.
3. **Relecture** (AC-8, AC-9) : extraction du décodage (une fonction pour l'import et la réactivation),
   `reactivate_to_complete_with_qr` (`kesh-db`), trace, tests T6 et T7.
4. **Textes** (AC-7, AC-10 à AC-13) : quatre locales, replis, Vitest et E2E mis à jour, manuels (`make fr` dans
   `docs/manual/`, sous verrou de gate), `api-external.md`, CHANGELOG.
5. **Captures** et gates.

## Tests et mutations prévus

| Test | Lieu | Ce qu'il prouve |
|---|---|---|
| T1 | `qr_decode.rs` (`mod tests`) | AC-3 : les six cas de `render_scale` ; rouge au commit rouge |
| T2 | `qr_decode.rs` | AC-4 : lu à 1414 × 2000 ; témoin non lu à 595 × 842 |
| T3 | `qr_decode.rs` | AC-4 : chemin image, résolution native, lu |
| T4 | `qr_decode.rs`, `#[ignore]` | AC-2 / AC-5 : PDF par pdfium, avant (non lus) / après (lus), vectoriel et sans-QR |
| T5 | `qr_decode.rs`, `#[ignore]` | AC-6 : banc de coût |
| T6 | `tests/inbox_import_e2e.rs` | AC-8 : un PNG à QR importé, ses colonnes QR remises à `NULL` en SQL (les sept **et** les colonnes facultatives, comme une ligne entrée sans QR) et son statut à `discarded` ; le même PNG redéposé → `present`, colonnes égales au payload, trace `reactivated` avec `"qr": "present"`, `"qr_reread": true` ; variante QR inexploitable (`qr_invoice_png_with`, valeur trop longue) → `unreadable` / `FIELD_TOO_LONG`, colonnes `NULL` ; `import_no_qr_duplicate_and_reactivation` (`:851-883`) reste vert (`none`, `"qr_reread": true`) |
| T7 | `tests/inbox_import_e2e.rs` | AC-8 / AC-9 : ligne `discarded` avec QR dont `creditor_name` est changé en SQL (`'Ancien nom'`) → redéposée, `present`, `creditor_name` **inchangé** (`'Ancien nom'`), `"qr_reread": false` ; ligne `to_complete` sans QR redéposée → `DUPLICATE`, colonnes inchangées |
| T8 | `import-page.test.ts`, `supplier-documents.spec.ts` (E2-a) | AC-7 : « Aucun QR lu » / « aucun QR lu » |

Les tests T6 et T7 passent par le chemin **image** (pas de pdfium) : ils tournent en CI.

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `render_scale` rend `1.0` (l'ancienne valeur) | T1 ; T4 (local) |
| M2 | plafond `MAX_RENDER_DPI` retiré du `min` | T1 (A6 portrait, page minuscule) |
| M3 | borne `max_dimension` retirée du `min` | T1 (A4, A3 : sortie ≤ 2000 px) |
| M4 | `PossibleFormats` retiré | **aucun test fonctionnel** — mesure de coût seulement ; déclarée au Dev Agent Record avec le chiffre du banc |
| M5 | chaque échelon supplémentaire retiré (s'il existe, AC-4) | le cas de T2 qui l'a justifié |
| M6 | réactivation sans relecture (code d'avant) | T6 |
| M7 | garde `creditor_iban IS NULL` retirée **et** décodage forcé pour une ligne pourvue | T7 (`creditor_name` réécrit) |
| M8 | rapport de réactivation = statut stocké au lieu du résultat de la relecture | T6 |
| M9 | trace sans `"qr_reread"` | T6 |
| M10 | valeur fr-CH du badge rendue à « Sans QR-code » | T8 (Vitest) et garde « repli égal fr-CH » |

Mutations sous verrou de gate, motif vérifié unique, restauration par `git checkout -- <fichier>` **puis `touch`**
(`docs/agents-orchestration.md` § 7).

## E2E

Aucune spec neuve : la relecture se prouve en Rust (T6, T7) — un même fichier ne peut pas être illisible puis lisible
pour une même version, et l'E2E n'a pas pdfium. `supplier-documents.spec.ts` (E2-a, l. 94-107) suit le texte : titre et
assertion « aucun QR lu ». E2E complet au **dernier commit de code** (`docs/testing.md`, 7 à 10 échecs attendus jugés
fichier par fichier).

## Captures (thème clair)

Sur une base de démonstration, dossier `kesh-gate-logs/v0175/` : (1) la liste *Documents à traiter* avec une ligne
« Aucun QR lu » et une ligne « QR-facture » ; (2) le rapport d'import portant « aucun QR lu » ; (3) le sélecteur de
document à joindre (`AttachDocumentPicker`) avec le badge. Une capture en `de-CH` du badge (`KESH_LANG=de`). Jointes au
Dev Agent Record et au prompt de revue de code.

## Gates

`kesh-db` est touché (une fonction de dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Test Locally
First*, exception `kesh-db`). Au dernier commit de code : `scripts/test-fast.sh` sous verrou, `npm run check`,
`npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, E2E complet ; T4 et le banc joués en local avec
`libpdfium` épinglée (déclarés comme tels, jamais comme « CI »). Base remise à zéro avant tout gate complet, selon
`docs/agents-orchestration.md` § 4 (bases `kesh_v0175`, `kesh_e2e_v0175`).

## Ce que les gardes ne voient pas

- **La CI ne voit pas le chemin PDF** : T4 et le banc ne tournent qu'en local (I2). Une régression du rendu ne
  rougirait en CI que par T1 (la fonction pure), non par un rendu réel.
- **Un scan réel n'est pas une fixture** : rotation, moiré, compression JBIG2, QR partiellement masqué ne sont pas
  simulés (pas de rotation arbitraire sans dépendance neuve). La vérification sur les deux pièces de #854 se fait par
  Guy, en local, et se rapporte « lues » / « non lues », sans aucune donnée.
- **Le seuil de lecture n'est pas borné vers le bas** : un scan à 100 dpi ou très flou peut rester illisible ; le
  manuel le dit (« un scan de mauvaise qualité peut échapper à la lecture »).
- **M4** n'a pas de test fonctionnel (coût seulement).

## Dev Notes

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/qr_decode.rs` — `render_scale`, `MAX_RENDER_DPI`, décodeur, générateur et tests T1-T5.
- `crates/kesh-api/src/inbox_import.rs` — décodage extrait, relecture à la réactivation, trace.
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` — `reactivate_to_complete_with_qr`.
- `crates/kesh-api/Cargo.toml` — `printpdf` en `dev-dependencies`.
- `crates/kesh-api/tests/fixtures/` — quatre PDF synthétiques.
- `crates/kesh-api/tests/inbox_import_e2e.rs` — T6, T7.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — deux valeurs chacune.
- `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`, `.../import-page.test.ts`,
  `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte`,
  `frontend/tests/e2e/supplier-documents.spec.ts`.
- `docs/manual/fr/user-manual.tex` (+ `.pdf`), `docs/manual/fr/admin-manual.tex` (+ `.pdf`), `docs/api-external.md`,
  `CHANGELOG.md`.

Pas de découpage (C-v017-5-10).

### Le manuel (axe de revue)

Les phrases prescrites (AC-10, AC-11) doivent être vraies au code livré : « environ 170 dpi » (2000 / 11,69 in = 171),
« au plus 300 dpi », « sans doublon » (même ligne réactivée), « n'est jamais relu » pour un document lu ou `completed`.
Contrôler le **PDF** aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

### Références

- `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs:282-290` (`scale_page_by_factor`), `:314-327`
  (maxima), `:716-835` (`apply_to_page`).
- `rxing-0.7.1/src/helpers.rs:232-249` (`detect_multiple_in_luma_with_hints`),
  `rxing-0.7.1/src/decode_hints.rs:264` (`PossibleFormats`), `:270` (`TryHarder`), `:322` (`AlsoInverted`),
  `rxing-0.7.1/src/filtered_image_reader.rs`.
- `printpdf-0.7.0/src/xobject.rs:136-159` (`ImageXObject`, filtre DCT sans la fonctionnalité `embedded_images`).
- Norme SIX (QR-facture) : QR de 46 × 46 mm, section paiement de 210 × 105 mm en bas d'une A4, récépissé de 62 mm.

### Issues à ouvrir (par l'orchestrateur)

- **I1 (P3, dette)** — « Import de pièces : le décodage QR occupe un fil de l'exécuteur tokio » : `process_one_file`
  (`inbox_import.rs:540-550`) décode de façon synchrone dans une fonction `async`, sans `spawn_blocking` ; un PDF de 20
  pages occupe un fil pendant toute sa lecture (AC-6 : jusqu'à 20 s). Remède : `spawn_blocking`, en tenant compte du
  verrou `PDFIUM_LOCK` et de la propagation d'une panique.
- **I2 (P3, test)** — « Le chemin PDF du décodage QR n'est exercé par aucun test qui tourne » : `libpdfium` absente de
  la CI et de la station ; tests `#[ignore]` ; installer la release épinglée du `Dockerfile` dans le job `backend` (ou un
  job dédié) et lever les `#[ignore]`.
- **I3 (CR, enhancement, si Q1 le confirme)** — « Relire le QR d'un document à traiter depuis l'écran » (bouton), que
  #854 propose et que la story ne livre pas (C-v017-5-8).

### Commentaire d'issue à poser (par l'orchestrateur)

Sur #854 : la prémisse « résolution de rendu retenue » est corrigée — aucune n'est retenue, la page est rendue à 72 dpi
(`pdfium-render`, échelle 1,0 par défaut) ; la cause H1 et sa preuve prévue (fixtures synthétiques, T4) ; la relecture
d'une pièce déjà importée passe par retrait et redépôt ; le bouton est renvoyé (I3).

## Questions au Project Lead

- **Q1** — Le bouton « Relire le QR » n'est pas livré (C-v017-5-8 : à version égale, relire rend le même résultat ; le
  redépôt après retrait couvre le cas d'une mise à jour). Faut-il ouvrir une demande d'évolution (I3) ?
- **Q2** — « Aucun QR lu » vous convient-il, ou préférez-vous « QR non lu » ?
- **Q3** — La PR peut-elle porter `closes #854`, le bouton étant renvoyé à I3 ?
- **Q4** — Vos deux factures communales déjà importées : après la mise à jour, le redépôt (Télécharger, Retirer,
  déposer, Importer) est un geste de votre part sur vos livres ; la release n'en fait rien d'office. Vous convient-il de
  rapporter seulement « lues » / « non lues » ?

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-31 — Fiche créée (Opus 5.5, autonomie). Cause H1 (rendu à 72 dpi) établie au code ; preuve prescrite (T4).
  Registre C-v017-5-1 à C-v017-5-11. Prochaine étape : validation P1 (`v017-5-validate-prompt-p1.md`, Sonnet, R et F).
