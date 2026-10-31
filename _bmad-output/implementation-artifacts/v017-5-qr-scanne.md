# Story v017-5 : lire le QR d'une facture QR scannée

Status: done

*Fiche rédigée le 2026-10-31, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0176`, branche
`story/v017-5-qr-scanne` partie de `8e126cfd` (tête de `release/0.17.0`). Release **0.17.0**, candidate suivante
(décision de Guy du 2026-10-31). Registre : `v017-5-choix-autonomes.md` (C-v017-5-1 à C-v017-5-19). Clé courte :
`v0175`. Issue : **#854** (bug, P2), en `refs` seulement — dans les commits **et dans la PR de release** : #854 ne se
ferme qu'au constat du Project Lead sur son installation (§ *Fermeture de #854*). Les numéros de ligne sont ceux de `8e126cfd` ; ceux des dépendances, ceux de `~/.cargo/registry`
(`pdfium-render-0.8.37`, `rxing-0.7.1`, `printpdf-0.7.0`).*

⛔ **Aucune pièce réelle** : ni les PDF du Project Lead (`~/kesh/`, son dossier de projet, Paperless, `~/kesh-test/`) ni
rien qui en dérive n'entre dans le dépôt, un commit, un test, un journal versé ou une capture. Toutes les fixtures sont
synthétiques (C-v017-5-5).

## Story

En tant que Comptable qui importe ses factures fournisseur **scannées** (archives PDF/A d'un gestionnaire de documents),
je veux que Kesh lise le QR de leur section paiement comme celui d'une facture née numérique, que l'écran dise « aucune
QR-facture exploitable » plutôt que d'affirmer que le document n'en porte pas, et pouvoir faire relire une pièce déjà importée, afin de ne
pas ressaisir à la main l'IBAN, la référence et le montant qu'un QR présent sur la pièce porte déjà.

## Origine

- **#854 (P2)**, recette de la `0.17.0-rc1` par Guy le 2026-10-31 sur des pièces réelles (non jointes, dépôt public) :
  sur quatre PDF importés, le QR d'une facture d'assurance est lu ; ceux de **deux factures communales**, dont la section
  paiement est présente, ne le sont pas — classées « sans QR-code ». Ces PDF sont la version archivée PDF/A (OCR) de
  **scans** : le QR y est une image matricielle à la résolution du scan.
- **Prémisse de l'issue corrigée** : #854 parle d'un QR qui échappe « à la résolution de rendu **retenue** ». Le code
  n'en retient **aucune** : la page est rendue à **72 dpi**, la borne de 2000 px n'étant jamais atteinte (§ *Ce que le
  code dit*, prémisse 1). Commenté sur #854 le 2026-10-31.

## Données à garder, migration — ce que la story ne fait pas

**Cette story n'ajoute aucune migration, aucune table, aucun bump** de `kesh_version_min_required` ni de version de
crate (contrainte du Project Lead du 2026-10-30, `kesh-gate-logs/prompts/v017-donnees-a-garder.md`) ; la release 0.17.0
en porte déjà une (`20261030000001_bank_imports_manual_source`, #816), qui relève `kesh_version_min_required` à 0.17.0. L'inventaire des tables ne change pas :
une sauvegarde de la 0.16.x continue de s'importer dans la 0.17.0. **Rien n'est relu d'office** : un document importé
avant la 0.17.0 garde ses données (ou leur absence) tant que l'utilisateur ne le **redépose** pas après l'avoir retiré de
la liste (AC-8) ; alors seules les colonnes QR **vides** d'une ligne réactivée s'écrivent (C-v017-5-9). Une ligne qui a
déjà un QR, une ligne `to_complete` ou `completed` n'est jamais réécrite (une ligne détachée de sa facture redevient
`to_complete`, `imported_supplier_invoices.rs:339-341` : elle suit alors le sort d'une ligne `to_complete`). Toute recette se joue sur une installation
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
   défaut voisin, non traité (#855).
8. **Les sept colonnes QR** obligatoires vont ensemble (`chk_imported_si_qr_all_or_nothing`, migration
   `20261015000001_imported_documents_without_qr.sql`) ; `has_qr()` = `creditor_iban.is_some()`.

**Cause retenue (C-v017-5-1)** : **H1, le rendu à 72 dpi.** Le QR d'une QR-facture mesure 46 mm ; pour un payload de
facture réelle (QR de version 10 à 15, 57 à 77 modules), 72 dpi donnent **1,7 à 2,3 px par module** : un QR vectoriel
peut s'y lire, un QR scanné — flou, bruité, rééchantillonné — ne le peut plus. H1 est **établie au code, non prouvée** :
la preuve est T4 (AC-2), jouée avant tout correctif. **T4, sous pdfium, fait foi** : le test d'hôte T2 n'approche le
rendu de pdfium que par un rééchantillonnage ; si les deux se contredisent, T4 l'emporte.

## Critères d'acceptation

*Vocabulaire* : une **QR-facture** est un QR dont le payload commence par `SPC\n0200` (`qr_decode.rs:26`, `first_spc`,
`:98-113`). Un QR d'un autre usage (lien, code produit) est **lu** par `rxing` puis écarté : pour Kesh, la page ne livre
alors **aucune QR-facture**. C'est l'absence de QR-facture, non l'absence de QR, qui déclenche tout ce qui suit.

### La cause, prouvée avant le correctif

**AC-1 — Fixtures synthétiques.** Un générateur de test (C-v017-5-5) produit, à graine fixe :
(a) trois PDF d'une page A4 portant chacun une image JPEG en niveaux de gris d'une QR-facture scannée à **150, 200 et
300 dpi**, flou gaussien de 0,1 mm, bruit uniforme ± 12 niveaux, papier à 240 et encre à 30 (sur 255) ; (b) un PDF
d'une **page scannée pleine, sans QR** (300 dpi, lignes de « texte » figurées par des blocs sombres de largeurs tirées au
sort sur toute la hauteur de la page, même flou, **même bruit sur toute la page**) ; (c) les **octets JPEG** de ces
images, conservés en mémoire pour les tests d'hôte (T2 et T3 décodent ce JPEG, à la qualité retenue, non l'image avant
compression). Le QR des scans est dessiné par **`kesh_qrbill::generator::render_qr_image`** (`generator.rs:94-97`,
`qrcodegen`, niveau de correction **M** de la norme SIX et des factures émises par Kesh) — jamais par
`rxing::qrcode::QRCodeWriter`, dont le niveau par défaut est L (`rxing-0.7.1/src/qrcode/qr_code_writer.rs:80`). Son
payload, construit par `kesh_qrbill::build_payload` (référence QRR, adresses structurées, débiteur final, message), fait
**au moins 280 octets UTF-8** et donne un QR de **version 12 à 15** (assertions sur `QrCode::version()` dans le
générateur : au-delà de 15, 81 modules et plus, le second rendu tomberait sous 4 px par module). Les quatre PDF sont
**produits une fois et versionnés comme données** sous `crates/kesh-api/tests/fixtures/` (`qr-scan-150dpi.pdf`,
`qr-scan-200dpi.pdf`, `qr-scan-300dpi.pdf`, `scan-sans-qr.pdf`), lus **à l'exécution** par
`std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/…"))` — chemin absolu fixé à la compilation,
indépendant du répertoire courant, et sans `include_bytes!`, qui empêcherait de compiler les tests tant que les fixtures
n'existent pas, donc de lancer le générateur qui les produit ; le générateur (un test `#[ignore]` qui écrit les quatre
fichiers) est versionné et documenté (paramètres, commande qui l'exécute), **sans exigence de reproductibilité au bit
près** — `printpdf` tire un identifiant d'instance et date le document à chaque écriture
(`printpdf-0.7.0/src/pdf_document.rs:793`, `pdf_metadata.rs:58`). La preuve porte sur le **décodage** (lu, non lu).
**Plafond : 1 Mio par fixture** ; pour le tenir sur les trois fixtures **à QR**, le développement peut baisser la qualité
JPEG (jamais sous 60) ou restreindre le bruit à la section paiement ; `scan-sans-qr.pdf` garde son bruit sur toute la
page (qualité seule) et, s'il dépasse 1 Mio à la qualité 60, passe à 200 dpi — il ne doit prouver que `Ok(None)` ;
s'il dépasse **encore** 1 Mio à 200 dpi, le développement s'arrête et **demande une issue** : le plafond n'est jamais
relevé en silence. Le banc, lui, engendre la page de D1 **à 300 dpi en mémoire** quelle que soit la résolution de la
fixture versionnée, pour ne pas sous-estimer le pire document. Valeurs retenues et tailles au Dev Agent Record. Aucune donnée réelle : IBAN et QR-IBAN d'exemple
SIX (`CH9300762011623852957`, déjà employé par `qr_decode.rs:211`), noms et adresses fictifs.

**AC-2 — La reproduction, rouge d'abord.** Au **premier commit** de la story (fixtures, générateur, tests, banc, la boucle
extraite d'AC-3 **sans second temps**, et `render_scale` extraite, que rien n'appelle encore), sans aucun correctif :
- **T4** (pdfium, `#[ignore]`, joué en local, C-v017-5-6) : (α) une **sonde** indépendante de la résolution — la page de
  `qr-scan-300dpi.pdf` rendue à 595 × 842 px n'est pas uniforme (au moins deux niveaux de gris distincts sur plus de
  1 % des pixels chacun) ; (β) `decode_spc_from_pdf_bytes` rend `Ok(None)` pour les trois `qr-scan-*.pdf` et pour
  `scan-sans-qr.pdf` ; (γ) `Ok(Some(_))` pour le **témoin vectoriel** — un PDF engendré à l'exécution par
  `kesh_qrbill::generate_qr_bill_pdf_with_date(data, &InvoicePdfData, &QrBillI18n, OffsetDateTime)` (`pdf.rs:40-45`,
  devises de `data` et de `InvoicePdfData` égales), avec un `InvoicePdfData` minimal construit comme dans
  `crates/kesh-qrbill/tests/golden_test.rs`, dont le `QrBillData` est **celui de `render_spc_qr_png`** (`qr_decode.rs:210-227` :
  « Robert Schneider SA », adresse combinée, 100.00 CHF, sans référence, message « Facture test » — environ 130 octets,
  QR de version basse, ≈ 2,7 px par module à 72 dpi) ;
- **T1** (hôte) : les tests de `render_scale` (AC-3) sont **rouges** (la fonction rend encore `Some(1.0)`) ;
- **Branches d'arrêt** — le développement s'**arrête**, consigne la sortie et la signale à l'orchestrateur, sans
  correctif à l'aveugle :
  - (i) T4 **lit** une fixture `qr-scan-*` avec le code d'avant : **la fixture ne reproduit pas le défaut** à 72 dpi —
    H1 n'est ni prouvée ni réfutée (les fixtures sont peut-être moins dégradées que les pièces de #854).
    L'orchestrateur peut alors ordonner, **dans cet ordre écrit d'avance** et sans régler les paramètres sur le
    résultat attendu, de régénérer les fixtures avec : (1) bruit ± 24 ; (2) puis flou 0,2 mm ; (3) puis image **binarisée
    avant l'encodage JPEG** (seuil à mi-chemin papier-encre ; noir et blanc purs, encodés en JPEG gris comme les autres
    paliers — T2 et T3 décodent toujours un JPEG, et le plafond de 1 Mio tient ; un vrai PDF 1 bit, CCITT ou JBIG2, que
    `printpdf` 0.7 n'écrit pas, reste un angle mort). Le premier palier qui reproduit le défaut devient celui des fixtures ; si
    aucun ne le reproduit, la story est suspendue et la cause rouverte ;
  - (ii) la **sonde** (α) échoue : le chemin pdfium ne fonctionne pas sur la station (bibliothèque, liaison) et T4 ne
    prouve rien. Si la sonde passe mais que le **témoin vectoriel** (γ) n'est pas lu, ce n'est pas une panne : constat
    consigné, décision de l'orchestrateur ;
  - (iii) après AC-3 et AC-4, une fixture du périmètre reste illisible : la découpe pdfium d'une bande à plus haute
    résolution (C-v017-5-2 (d)) est alors une décision de l'orchestrateur ;
- la sortie de T4 au commit rouge est **copiée** au Dev Agent Record (commande, version de `libpdfium`, résultat par
  fixture).

### Le remède

**AC-3 — Rendu en deux passes par document (C-v017-5-2).** La boucle des pages est **extraite** de
`decode_spc_from_pdf_bytes` en une fonction générique, sans pdfium, sur un rendeur injecté :

```rust
pub(crate) enum Temps { Premier, Second }
fn decode_pages<R>(page_count: usize, max_pages: usize, render: R)
    -> Result<Option<ScannedQrBill>, DecodeError>
where R: FnMut(usize, Temps) -> Result<Option<DynamicImage>, String>
```

`decode_spc_from_pdf_bytes` l'appelle avec un rendeur pdfium : `Temps::Premier` = la configuration d'avant, inchangée
(échelle 1,0, bornes `max_dimension`) ; `Temps::Second` = `scale_page_by_factor(s)` avec `s = render_scale(page.width()
.value, page.height().value, cfg.max_dimension)`, bornes `set_maximum_width/height(cfg.max_dimension)` conservées, et
`Ok(None)` (pas d'image) quand `render_scale` rend `None`. La conduite de `decode_pages`, entièrement :
1. **Première passe** : les pages `0 .. min(page_count, max_pages)` dans l'ordre, au premier temps ; la première
   QR-facture **exploitable** est rendue ; une QR-facture **inexploitable** (`InvalidSpcPayload`, `InvalidIban`) est
   rendue en erreur aussitôt, comme aujourd'hui (le QR a été lu : pas de second temps) ; une erreur de rendu est
   retenue dans `last_render_err`, comme aujourd'hui (`qr_decode.rs:183-188`), et la page est marquée.
2. **Seconde passe**, **seulement si aucune page de la première n'a livré de QR-facture** : les mêmes pages, dans
   l'ordre, **sauf** celles dont le premier rendu a échoué, au second temps ; même conduite pour une QR-facture
   exploitable ou inexploitable. Une **erreur du second rendu n'est pas une erreur du document** (la page s'est rendue au
   premier temps) : elle est journalisée (`tracing::warn!`, numéro de page et message, aucun contenu) et la page compte
   comme sans QR-facture — **une erreur du second rendu ne change jamais le diagnostic** (un document `none` à cause
   d'elle reste `none`, jamais `PDF_RENDER_ERROR`). Le second temps change en revanche le résultat quand il **lit** une
   QR-facture : `none` devient `present`, ou `unreadable` si elle est inexploitable — c'est le but.
3. **Fin** : si `page_count > max_pages`, `Err(PdfRender(last_render_err ou « cap de N pages atteint sans QR SPC »))`,
   comme aujourd'hui (`:173-182`) ; sinon `last_render_err` → `Err(PdfRender)` ; sinon `Ok(None)`.

Ainsi, un document dont une page porte une QR-facture lisible à 72 dpi (née numérique, quel que soit le rang de la page)
coûte exactement ce qu'il coûtait ; seul un document dont **aucune** page ne livre de QR-facture au premier temps paie
la seconde passe.

`qr_decode.rs` expose une fonction pure documentée `render_scale(width_pt: f32, height_pt: f32, max_dimension: u16) ->
Option<f32>` = `Some(min(max_dimension / width_pt, max_dimension / height_pt, MAX_RENDER_DPI / 72))` avec
`const MAX_RENDER_DPI: f32 = 300.0`, et `None` sauf si `width_pt.is_finite() && width_pt > 0.0 &&
height_pt.is_finite() && height_pt > 0.0 && max_dimension > 0` (garde explicite : `f32::min` ignore un `NaN`). Tests
d'hôte (T1), échelles à 10⁻³ près, dimensions de sortie comparées **après arrondi** (`(dim × échelle).round()`, comme
`apply_to_page`, `render_config.rs:825-826`) :
- A4 portrait (595,276 × 841,89 pt), borne 2000 → 2000 / 841,89 ≈ **2,3756** (≈ 171 dpi) ; sortie 1414 × 2000 ;
- A4 paysage → même échelle (borne par la largeur) ;
- section paiement seule, 210 × 105 mm (595,28 × 297,64 pt) → 2000 / 595,28 ≈ **3,3598** (≈ 242 dpi) ;
- A6 portrait (297,64 × 419,53 pt) → **300 / 72** ≈ 4,1667 (le plafond dpi l'emporte sur 2000 / 419,53 ≈ 4,767) ;
- A3 portrait (841,89 × 1190,55 pt) → 2000 / 1190,55 ≈ 1,6799 ; sortie ≤ 2000 px ;
- page minuscule (10 × 10 pt) → 300 / 72 ;
- largeur 0, largeur −1, hauteur `NaN`, hauteur `f32::INFINITY`, `max_dimension = 0` → `None` (cinq cas).
`DecodeConfig::default()` est **inchangé** (20 pages, 2000 px). Aucun rendu ne dépasse 2000 px de côté. **Page de plus
de 2000 pt** : le premier rendu la **déforme** aujourd'hui (sans facteur d'échelle, `apply_to_page` ne garde pas les
proportions, `render_config.rs:770-773`, et les maxima ne réduisent qu'une dimension) — défaut préexistant, inchangé au
premier temps ; le second temps la rend sans déformation, à une échelle qui peut être inférieure à 1. Aucune facture
n'est attendue dans ce cas ; angle mort écrit (§ *Ce que les gardes ne voient pas*), déformation du premier
rendu tracée en issue (P4, ouverte par l'orchestrateur).

**T10** (hôte, **en CI**) — `decode_pages` avec un **faux rendeur** (images synthétiques en mémoire, compteur d'appels
par page et par temps, erreurs simulées) : (a) QR-facture lisible au premier temps en page 3 d'un document de 3 pages →
trouvée, **aucun** appel au second temps ; (b) aucune QR-facture au premier temps, lisible au second en page 2 →
trouvée, les trois pages rendues au premier temps avant tout second temps ; (c) QR-facture inexploitable au premier
temps → l'erreur, aucun second temps ; (d) erreur de rendu de la page 2 au premier temps, aucune QR-facture ailleurs →
page 2 non rendue au second temps, résultat `PdfRender` ; (e) erreur de rendu au second temps, aucune QR-facture →
`Ok(None)`, non `PdfRender` ; (f) `page_count = 25`, `max_pages = 20`, aucune QR-facture → pages 20 à 24 rendues à
**aucun** temps, résultat `PdfRender` « cap » ; (g) une page portant un QR non SPC au premier temps → second temps
appelé.

**AC-4 — Décodeur.** `decode_qr_from_image` passe par `detect_multiple_in_luma_with_hints` avec `PossibleFormats =
{QR_CODE}` et `TryHarder = true` (C-v017-5-3), aux deux temps. Test d'hôte **T2** : chacune des trois images
synthétiques (150, 200, 300 dpi), **décodée depuis ses octets JPEG**, rééchantillonnée (`image::imageops::resize`,
filtre `Triangle`) aux dimensions du second rendu d'une A4 (1414 × 2000 px), est **lue** — IBAN, montant et référence
égaux au payload. **T1-bis** (grandeur causale, pur) : pour le QR des fixtures (`QrCode::size()` du générateur), les
pixels par module au premier rendu (46 mm à 72 dpi) sont **inférieurs à 2,5**, et au second rendu (≈ 171 dpi)
**supérieurs à 4**. Le contrôle « les mêmes images à 595 × 842 px ne sont pas lues » est un **diagnostic** `#[ignore]`
(T2-diag), joué à la main avec T4, résultat au Dev Agent Record. **T3** (contrôle) : les mêmes octets JPEG, à leur
résolution native, par `decode_spc_from_image_bytes`, sont lus. **Échelon supplémentaire** (`FilteredImageReader`, puis
découpe de la bande inférieure) : écrit **seulement** si une image de T2 n'est pas lue sans lui et l'est avec lui, et
alors couvert par une mutation (M5) ; sinon non écrit, et le Dev Agent Record le dit. En cas de contradiction entre T2
et T4, **T4 fait foi**.

**AC-5 — Le correctif, prouvé par pdfium.** Au dernier commit de code, T4 (même commande qu'à AC-2) : sonde (α) verte ;
`Ok(Some(_))` pour les trois `qr-scan-*.pdf` (IBAN, montant, référence égaux au payload) et pour le témoin vectoriel —
lu, lui, **dès la première passe** — point d'accroche : `qr_decode.rs` expose `pub(crate) fn
decode_spc_from_pdf_bytes_observed(bytes, cfg, on_render: &mut dyn FnMut(usize, Temps))`, appelé par
`decode_spc_from_pdf_bytes` avec un observateur vide ; T4 et le banc passent un compteur d'appels au second temps, qui
doit rester à 0 pour le témoin vectoriel (T4) et pour D4 (banc) — D1 à D3 doivent en compter un par page ; `Ok(None)` pour `scan-sans-qr.pdf`. Sortie copiée au Dev Agent Record. Le test cassé
`decode_spc_from_pdf_single_page` (`qr_decode.rs:301-313`) lit désormais `qr-scan-300dpi.pdf` par le chemin absolu d'AC-1 —
correction mécanique (C-v017-5-6).

**AC-6 — Coût, mesuré et plafonné, contre les deux délais.** Un banc `#[ignore]` (`qr_decode::tests::banc_cout`) mesure,
**avec le même banc et les mêmes fixtures**, au commit rouge (première passe seule) **et** au dernier commit de code :
(a) le temps de rendu + décodage **par page** de chaque fixture, médiane de 5 passages ; (b) quatre documents
« pire cas » de 20 pages, engendrés en mémoire par le banc (non versionnés), chaque page subissant les deux passes sans
QR-facture : **D1** 20 × une page engendrée en mémoire à 300 dpi, de même facture que `scan-sans-qr.pdf` (AC-1) ; **D2** 20 pages **carrées** (842 × 842 pt) scannées sans QR
(2000 × 2000 px au second temps, soit 1,41 fois les pixels d'une A4) ; **D3** 20 pages A4 portant chacune une douzaine
de QR **non SPC** (`GenericMultipleBarcodeReader` relance la recherche autour de chaque code trouvé,
`rxing-0.7.1/src/multi/generic_multiple_barcode_reader.rs:113`, `MAX_DEPTH = 4`) ; **D4** une facture née numérique de
trois pages, QR vectoriel en dernière page (contrôle : la seconde passe n'est **pas** jouée) ; le **pire document** est
le plus lent de D1 à D3 ; (c) la mémoire résidente de pointe du **binaire de test**. Commande, sous le verrou de gate et
`scripts/mem-guard.sh` :

```sh
cd /home/gcorbaz/devel/kesh-0176 && \
  CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-0176/target \
  cargo test --release -p kesh-api --lib --no-run --message-format=json \
  | jq -r 'select(.profile.test == true and .target.name == "kesh_api") | .executable'
# puis, avec le chemin rendu :
LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib \
  /usr/bin/time -v <binaire> --ignored --exact qr_decode::tests::banc_cout --nocapture
```

(« Maximum resident set size » du binaire ; les fixtures sont lues par le chemin absolu d'AC-1, le répertoire courant est
indifférent). Le profil `--release` est le seul dont le coût est représentatif de l'image Docker. **Plafonds**, sur la
station : médiane **≤ 1,0 s par page** sans QR-facture (deux passes), pire document **≤ 20 s**, D4 dans ± 10 % de sa
mesure au commit rouge, pointe de mémoire **≤ 128 Mio de plus** qu'au commit rouge. Par construction, chaque bitmap
rendu reste sous **2000 × 2000 px × 4 octets ≈ 15,3 Mio**.

**Les deux délais.** L'écran *Documents à traiter* attend la réponse de `POST /api/v1/inbox-import`, qui traite tout le
passage (jusqu'à `KESH_INBOX_MAX_FILES_PER_RUN` fichiers, défaut 200, borne 10 000 — `config.rs:1011-1030`) dans une
seule requête synchrone (#855). Deux délais la bornent : celui du **client** — aujourd'hui **30 s**
(`api-client.ts:42`, `REQUEST_TIMEOUT_MS`, que `triggerInboxImport` emploie, `imported-supplier-invoices.api.ts:27-29`)
— et celui du **proxy** documenté, **300 s** (`admin-manual.tex:341`, `proxy_read_timeout`). La story :
- **aligne le client sur le proxy** : `imported-supplier-invoices.api.ts` exporte `INBOX_IMPORT_TIMEOUT_MS = 300_000`
  (commentaire : aligné sur `proxy_read_timeout` du manuel d'administration) et `triggerInboxImport` appelle
  `apiClient.post('/api/v1/inbox-import', {}, { timeoutMs: INBOX_IMPORT_TIMEOUT_MS, retry: false })` (les
  `CallOptions` existent, `api-client.ts:59-64`, `:589-600`) ; test Vitest **T11** qui épingle ces deux options ;
- **mesure contre les deux bornes** : le Dev Agent Record écrit, depuis le banc, combien de pages scannées sans
  QR-facture tiennent sous 30 s et sous 300 s sur la station, et une projection pour un NAS (facteur déclaré, non
  mesuré) ;
- **vérifie, par une exécution, ce qui se passe quand le client ou le proxy abandonne** (AC-6-bis).

**AC-6-bis — L'abandon de la requête : sort de l'import et verrou de run.** Lu au code : le verrou nommé est pris par
`GET_LOCK` sur une connexion dédiée (`inbox_import.rs:172-190`) et rendu par `RELEASE_LOCK` **après** le retour de
`process_inbox` (`:199-207`) ; la route l'appelle sans `tokio::spawn` (`routes/imported_supplier_invoices.rs:62`) ; quand
le client ferme la connexion, hyper abandonne le futur du gestionnaire à son prochain `await`. Si c'est le cas, le
`RELEASE_LOCK` n'est jamais exécuté, et la connexion — dont la **session** MariaDB tient le verrou — peut être **rendue
au pool** au lieu d'être fermée (à établir dans la source de `sqlx`, retour d'une `PoolConnection` abandonnée). Le
développement **exécute** la vérification, au choix, et en consigne la commande et la sortie :
- un test d'intégration (`tests/inbox_import_e2e.rs`) qui dépose assez de fichiers pour qu'un passage dure plusieurs
  secondes, lance l'import avec un client `reqwest` à délai court (abandon en cours de passage), attend la fin du
  passage côté serveur (fichiers restants stables dans le dossier pendant 2 s), puis relance l'import ; ou
- un essai sur l'installation **jetable** (image de la branche, fixtures synthétiques), `curl --max-time 2` puis un second
  `curl` après la fin du passage.
**AC décidable** : une fois le premier passage terminé côté serveur, (1) **`SELECT IS_FREE_LOCK('kesh_inbox_import:<base>')`**
— la clé de `inbox_import.rs:180`, lue sur une connexion neuve — vaut **1** (un `GET_LOCK` de MariaDB est
ré-entrant pour sa session : si le second import reprenait la connexion fuyante, il répondrait 200 malgré la fuite, d'où
cette assertion directe) ; (2) le second import répond **200**, non **409** ; (3) les fichiers traités avant l'abandon
sont dans la liste, les autres sont restés dans le dossier et sont traités par le second import ; aucun document n'est
en double. Exception admise et écrite : le fichier en cours au moment de l'abandon, si sa ligne a été validée avant que
le fichier ne quitte le dossier, part au second passage en `DUPLICATE` (sous-dossier `failed/`) — il est déjà dans la
liste. Résultat consigné au Dev Agent Record. **Si le verrou fuit** (409
persistant) : le développement le signale et demande une **issue P2** (« opération enregistrée mais annoncée en
échec », import bloqué) ; il ne corrige dans cette story **que si la correction est mécanique** — par exemple une garde
qui, abandonnée sans `RELEASE_LOCK`, **détache** la connexion (`PoolConnection::detach`) pour que sa fermeture rende le
verrou —, avec son test. Ce qui se passe est écrit aux deux manuels (AC-10, AC-11).

Un plafond dépassé se corrige en réduisant le travail (échelon supplémentaire limité à la première et à la dernière page,
ou retiré) — **jamais** en relâchant une borne (C-v017-5-4).

### Ce que l'écran dit

**AC-7 — « Aucune QR-facture exploitable ».** Les valeurs (clés inchangées, C-v017-5-7) :

| clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `imported-supplier-invoices-badge-no-qr` | Aucune QR-facture exploitable | Keine verwertbare QR-Rechnung | Nessuna fattura QR utilizzabile | No usable QR-bill |
| `imported-supplier-invoices-report-no-qr` | aucune QR-facture exploitable | keine verwertbare QR-Rechnung | nessuna fattura QR utilizzabile | no usable QR-bill |

Le libellé reprend celui du badge voisin (`badge-qr` : « QR-facture », « QR-Rechnung », « Fattura QR », « QR-bill ») et
est vrai dans **chacun** des cas où le badge s'affiche (toute ligne sans données QR stockées, `+page.svelte:293`) :
(1) document sans QR, (2) QR d'un autre usage, lu puis écarté par `first_spc`, (3) **QR-facture présente mais non
détectée** — le cas de #854 : `rxing` ne trouve rien (`qr_decode.rs:92`), `first_spc` rend `Ok(None)` (`:112`), rangé en
`none` (`inbox_import.rs:564`) —, (4) QR-facture lue puis refusée (`INVALID_SPC_PAYLOAD`, `INVALID_IBAN`,
`FIELD_TOO_LONG`, `QR_AMOUNT_OUT_OF_RANGE`), (5) erreur de rendu (`PDF_RENDER_ERROR`, bibliothèque absente ou cap de
pages compris) — dans aucun, Kesh ne dispose d'une QR-facture exploitable. « Aucune QR-facture lue » serait faux au
cas (4) (C-v017-5-7 ; libellé soumis au Project Lead, Q1 ; tant
qu'il n'a pas répondu, ce libellé fait foi). Les replis de
`+page.svelte:237`, `:312` et `AttachDocumentPicker.svelte:148` égalent la valeur fr-CH, et les **deux clés sont
ajoutées à `REPLIS_A_SITE_UNIQUE`** (`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts:222`, garde G13, qui
compare chaque repli au catalogue fr-CH, `:346`) — sans quoi aucune garde ne compare ces replis au catalogue (le préfixe
`imported-supplier-invoices-` n'est pas à `PREFIXES_REPLI_EGAL_AU_CATALOGUE`) ; le commentaire de la garde
(`:210`, « quatre-vingt-quatorze clés ») passe à **quatre-vingt-seize**, recompté depuis le tableau. Le commentaire d'en-tête de
`+page.svelte:12` suit. `README.md:44` nomme le badge : *QR-facture* ou *Aucune QR-facture exploitable*. La valeur d'API
`qr: "none"` ne change pas. **Inventaire de la valeur** (`grep -rnF "Sans QR-code"` et `grep -rnF "sans QR-code"` sur
tout le dépôt hors `_bmad-output/` et `CHANGELOG.md`, relevé le 2026-10-31 ; et sur les PDF aplatis, `pdftotext … - |
tr '\n' ' ' | tr -s ' ' | grep -oF … | wc -l`, qui voit aussi une expression coupée sur deux lignes du `.tex`) :

| Lieu | Décision |
|---|---|
| `fr-CH/messages.ftl:2538`, `:2552` ; replis `+page.svelte:237`, `:312`, `AttachDocumentPicker.svelte:148` ; commentaire `+page.svelte:12` ; `README.md:44` ; `user-manual.tex:1851`, `:1857` ; tests `import-page.test.ts:112`, `:127`, `:162`, `:185`, `supplier-documents.spec.ts:94`, `:104`, `:107` | **libellé d'écran : changé** |
| `fr-CH/messages.ftl:2533` (commentaire 30-2a), `+page.svelte:10`, `user-manual.tex:807`, `:1609`, `admin-manual.tex:818`, `:885-886` (coupé sur deux lignes), `:2541`, `marketing-brochure.tex:396`, `:446`, `api-external.md:255`, `:774`, `README.md:232`, `crates/kesh-db/migrations/20261015000001_imported_documents_without_qr.sql:10` (intouchable, P8) | **emploi générique** (« un document avec ou sans QR-code ») : conservé |
| autres formes « sans QR » : `user-manual.tex:1844` (« un document scanné sans QR »), `:1889` (« Document sans QR : tout se saisit »), `:1898` (« Sans QR, rien n'est proposé »), `admin-manual.tex:886`, `:905`, `:1492`, `:2546`, `:2551`, `:2555` | **conservées** : elles désignent un document **entré sans données QR** (ce que le formulaire et l'API voient), vrai que le document porte un QR ou non ; seule `:1844`, qui parle d'un document scanné, gagne « (ou dont la QR-facture n'a pas pu être lue) » |

Le grep (`.tex` et PDF) se rejoue après le patch : aucune occurrence de la première ligne ne doit subsister. **Limite
assumée** : la liste ne distingue pas les cas (1) à (5) — tous portent le même badge, la cause n'étant pas stockée
(C-v017-5-7) ; le rapport d'import, à l'instant de l'import, sépare seulement (1) à (3) — « aucune QR-facture
exploitable », sans pouvoir les distinguer entre eux — de (4) et (5) — « QR illisible » et son code.

### Relire une pièce déjà importée

**AC-8 — Relecture à la réactivation.** Quand un fichier déposé a l'empreinte d'une ligne `discarded` **sans données
QR** (`!existing.has_qr()`), Kesh le **décode** (même fonction que l'import, extraite pour les deux chemins — DRY) avant
la transaction de réactivation, puis :
- QR-facture lue **et** exploitable (`check_scanned_fits_columns` vert) → `reactivate_to_complete_with_qr`
  (C-v017-5-9) : une seule instruction, garde `status = 'discarded' AND creditor_iban IS NULL`, qui pose
  `status = 'to_complete', supplier_invoice_id = NULL` (comme `reactivate_to_complete`, `imported_supplier_invoices.rs:480-499`,
  qui efface le pointeur résiduel d'une complétion antérieure) et écrit les **quinze**
  colonnes QR — `creditor_iban`, `is_qr_iban`, `creditor_address_type`, `creditor_name`, `creditor_line1`,
  `creditor_line2`, `creditor_postal_code`, `creditor_town`, `creditor_country`, `reference_type`, `reference_value`,
  `amount`, `currency`, `unstructured_message`, `billing_information` (`NULL` explicite pour une facultative absente) —
  depuis une projection `QrColumns::from_scanned`, **extraite** de `NewImportedSupplierInvoice::from_scanned`
  (`entities/imported_supplier_invoice.rs:260-296`) et consommée par `from_scanned` et par
  `reactivate_to_complete_with_qr` ; **`NewImportedSupplierInvoice` garde ses champs plats** (huit littéraux de trois
  crates en dépendent — `kesh-seed`, `kesh-api` (tests), `kesh-db` (tests) —, dont `kesh-seed/src/pieges.rs:613`, `:654`) ; rapport `qr: "present"` ;
- QR-facture inexploitable (`InvalidSpcPayload`, `InvalidIban`, `PdfRender`, colonnes) → réactivation **sans** QR
  (`reactivate_to_complete`, inchangée) ; rapport `qr: "unreadable"` et `qrError` (mêmes codes qu'à l'import) ;
- aucune QR-facture → réactivation sans QR ; rapport `qr: "none"` ;
- `ImageDecode` (impossible à octets égaux sauf changement de la crate `image`) → réactivation sans QR, rapport
  `qr: "none"` — **jamais** un refus d'une pièce déjà archivée ;
- zéro ligne réactivée (course) → conduite actuelle (`DUPLICATE`, `inbox_import.rs:527-530`).
Une ligne `discarded` **avec** données QR est réactivée **sans décodage** (conduite actuelle, rapport `present`). La trace
`imported_supplier_invoice.reactivated` (même transaction) porte en plus `"qr"` (la valeur du rapport) et `"qr_reread"`
(`true` si le fichier a été décodé). Aucun fichier n'est réarchivé (même empreinte, même `storage_path`).

**AC-9 — Rien d'autre n'est relu ni réécrit.** Un document `to_complete` redéposé reste un `DUPLICATE` ; un document
`completed` (joint à une facture) redéposé aussi ; aucune ligne n'est relue au démarrage, à la migration ou à la liste.
Tests T7 et T9 (ci-dessous).

### Textes

Partout, le texte dit ce que fait le code : le déclencheur du second rendu est l'absence de **QR-facture**, et le rendu à
72 dpi reste le premier temps.

**AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, § *Documents à traiter*, l. 1838-1880), `.tex` **et**
PDF régénéré :
- l. 1851 : « avec la mention \emph{aucune QR-facture exploitable} ou \emph{QR illisible} » ;
- l. 1857-1861, le badge : « \textbf{Aucune QR-facture exploitable}~: Kesh n'a lu aucune QR-facture exploitable sur le
  document --- il ne porte aucun QR, ou un autre QR qu'une QR-facture, ou son QR n'a pas pu être lu (un scan de mauvaise
  qualité peut échapper à la lecture), ou il n'a pas pu être exploité (illisible, montant hors des limites admises,
  valeur trop longue). » — la suite (« Le document est alors accepté sans les données du QR… ») inchangée ;
- *Retirer de la liste* (l. 1874-1876) : « … le déposer de nouveau le fait revenir dans la liste~; s'il n'avait aucune
  QR-facture exploitable, Kesh le relit alors. Pour le redéposer, \emph{Téléchargez}-le d'abord~: une fois retiré, il ne se
  télécharge plus depuis la liste. Un fichier redéposé \emph{sans} l'avoir retiré est refusé comme doublon, sans être
  relu. » ;
- un paragraphe **neuf**, *Faire relire le QR d'un document*, après la liste des quatre gestes, qui dit le **fait** et non
  l'effet espéré : depuis la 0.17.0, quand aucune page d'un PDF ne livre de QR-facture à un premier rendu à 72 dpi, Kesh
  rend ses pages une seconde fois, à environ 170 dpi pour une page A4, ce qui permet de lire la QR-facture de bien des
  documents **scannés** ; un document entré plus tôt avec « Aucune QR-facture exploitable » n'est **pas** relu de lui-même ;
  pour le faire relire : *Télécharger*, *Retirer de la liste*, déposer le fichier téléchargé dans le dossier d'import,
  *Importer le dossier* — le document revient, avec les données du QR s'il est lu, sans doublon ; le même geste vaut
  après que l'administrateur a relevé la borne de pages ou installé la bibliothèque PDF ; un document dont la
  QR-facture a été lue n'est jamais relu, ni un document **tant qu'il est joint à une facture** ; si le QR reste
  illisible, l'IBAN, la référence et le montant se saisissent à la main ;
- un paragraphe **neuf**, *Un import long*, au même endroit : les documents scannés sans QR-facture lisible demandent
  plus de temps (deux rendus par page) ; déposez-les **par petits lots** ; l'écran attend la réponse jusqu'à cinq
  minutes, **mais un proxy inversé peut couper plus tôt** (sur un NAS Synology, le délai du portail DSM n'est pas
  documenté) ; l'écran affiche alors, selon le cas (`+page.svelte:106-109` affiche le message de l'`ApiError`, toujours
  présent) : « Le serveur ne répond pas. Réessayez ultérieurement. » quand l'écran abandonne lui-même au bout de cinq
  minutes (`error-timeout`, `api-client.ts:320`, `fr-CH/messages.ftl:2251`) ; « Erreur serveur (504) » quand un proxy
  coupe et répond par une page d'erreur (`error-server-status`, `api-client.ts:243-247`, `messages.ftl:2252`) ;
  « Impossible de contacter le serveur. Vérifiez votre connexion. » quand la connexion est fermée sans réponse
  (`error-network`, `api-client.ts:316-321`, `messages.ftl:2250`) — le repli « Erreur inattendue lors de l’import. »
  ne sort pas de ce chemin et n'est pas cité ; dans chacun de ces cas, attendez un moment, rechargez la page — en cas d'erreur, la liste n'est pas rechargée
  (`+page.svelte:96-108`) — puis relancez *Importer le dossier* : les documents déjà traités sont dans la liste, les
  autres sont restés dans le dossier et sont repris — la phrase exacte suit le résultat d'AC-6-bis (si le verrou fuit,
  elle dit ce que l'utilisateur voit et renvoie à l'administrateur) ;
- les emplois génériques (l. 807, 1609, 1889, 1898) restent ; l. 1844 complétée (AC-7, inventaire).

**AC-11 — Manuel d'administration** (`admin-manual.tex`), `.tex` et PDF :
- § *Passer à la 0.17.0* (l. 2481-2491) : les documents importés avant la mise à jour ne sont pas relus ; renvoi au
  manuel utilisateur (*Faire relire le QR d'un document*) ;
- § *Intégrations par l'API (0.17.0)* : la réactivation d'un document sans QR le relit, et son rapport dit `present`,
  `none` ou `unreadable` (avec `qrError`) selon cette lecture ;
- § *Réglage `KESH_INBOX_MAX_PDF_PAGES`* (l. 920-923) : les pages d'un PDF, jusqu'à la borne, sont rendues d'abord à
  72 dpi ; si aucune ne livre de QR-facture, celles qui se sont rendues le sont une seconde fois, à la plus haute résolution que permet la borne de
  2000 pixels par côté, au plus 300 dpi (environ 170 dpi pour une A4) ; un document sans QR-facture lisible coûte donc
  deux rendus par page ; l'ordre de grandeur mesuré à AC-6 (temps par page sur une station, et le fait qu'un NAS est
  plus lent) ; relever cette borne augmente ce coût d'autant — elle va jusqu'à 500 (`admin-manual.tex:822`), et un seul
  document de quelques dizaines de pages scannées peut alors, à lui seul, dépasser le délai de l'écran ;
- au même paragraphe, **les délais** : l'import d'un dossier se fait en une seule requête synchrone ; l'écran l'attend
  jusqu'à 300 s, comme le proxy nginx de l'exemple documenté (`proxy_read_timeout 300s`, \S\ref{sec:reverse-proxy}) ;
  **derrière le proxy inversé de DSM** (Application Portal, § *Étape 4*, l. 638-661), aucun délai n'est documenté : un
  import long peut y être coupé plus tôt — vérifiez, pour la version de DSM utilisée, le délai que le proxy admet, comme
  pour la sauvegarde d'installation (l. 661) ; le conseil des petits lots vaut d'autant plus ; au-delà,
  ce qui se passe est celui qu'établit AC-6-bis (import interrompu au fichier en cours, fichiers restants repris au
  passage suivant ; et, si le verrou fuit, la conduite et l'issue) ; pour des documents scannés nombreux, réduire
  `KESH_INBOX_MAX_FILES_PER_RUN` (défaut 200) ou faire déposer par petits lots ;
- les emplois génériques (l. 818, 885-886, 2541) et les formes « sans QR » (l. 886, 905, 1492, 2546, 2551, 2555) restent.

**AC-12 — `docs/api-external.md`** l. 778 : `none` devient « aucune QR-facture n'a été détectée — le document peut ne
porter aucun QR, un QR d'un autre usage, ou une QR-facture que la lecture n'a pas trouvée » — à l'API, `none` et
`unreadable` restent distincts ; la dernière phrase (« Un document
**réactivé** … porte `present` ou `none` selon ce qui a été stocké ») est remplacée par : « *Depuis la v0.17.0*, un
document **réactivé** — retiré de la liste puis redéposé — est relu s'il n'avait aucune donnée QR : `present`, `none` ou
`unreadable` (avec `qrError`) selon cette lecture ; sinon `present`. » ; et, à la route `POST /api/v1/inbox-import`, une
phrase sur la durée (requête synchrone, deux rendus par page sans QR-facture, conduite au délai dépassé selon AC-6-bis).

**AC-13 — CHANGELOG**, `[0.17.0] — Non publié`, *Corrigé* : « **La QR-facture d'un document scanné peut être lue à
l'import** ([#854](https://github.com/guycorbaz/kesh/issues/854)) : Kesh rendait chaque page d'un PDF à 72 dpi, ce qui,
sur des scans synthétiques, ne suffit pas à lire leur QR ; quand aucune page n'y livre de QR-facture, il rend désormais les pages une
seconde fois, à environ 170 dpi pour une page A4 (au plus 300 dpi, sous la même borne de 2000 pixels par côté).
L'écran attend un import jusqu'à cinq minutes (un proxy inversé peut couper plus tôt), et dit « Aucune QR-facture exploitable » plutôt que « Sans QR-code ». Un
document importé auparavant n'est pas relu de lui-même : *téléchargé*, retiré de la liste puis redéposé, il l'est
(manuel utilisateur, *Faire relire le QR d'un document*). Pour les intégrations : le rapport de la réactivation d'un
document sans QR peut désormais porter `unreadable` et `qrError` (`docs/api-external.md`). » Le chapeau de la section
n'est pas touché : la PR de release le récrit. Dans un commit comme dans la PR : `refs #854` seulement.

## Fermeture de #854 — recette du Project Lead

La PR de release porte `refs #854`, **jamais `closes`**. #854 se ferme quand le Project Lead a constaté, **sur son
installation, après la mise à jour** vers la candidate qui porte cette story, que ses deux factures communales sont
lues. Sa recette, qu'il joue lui-même sur ses livres (aucun geste de la release sur sa base) : pour chacune, *Télécharger*,
*Retirer de la liste*, déposer le fichier dans le dossier d'import, *Importer le dossier* ; il rapporte seulement
« lues » ou « non lues », **sans aucune donnée** (ni IBAN, ni montant, ni référence, ni capture). « Lues » : il ferme
#854 (ou le demande). « Non lues » : #854 reste ouverte, et la suite (paliers de dégradation d'AC-2 (i), découpe à plus
haute résolution, échelon du décodeur) se décide sur fixtures synthétiques, jamais sur ses pièces.

Recette de l'orchestrateur, avant : sur une installation **jetable** de l'image Docker de la branche, déposer les quatre
fixtures et, comme témoin vectoriel, le PDF d'une facture QR **émise par cette installation jetable** (données de
démonstration) ; attendus : trois scans et le témoin lus, `scan-sans-qr.pdf` « Aucune QR-facture exploitable » ;
puis un scan importé, ses colonnes QR vidées en SQL sur cette base jetable, retiré, redéposé : il revient « QR-facture ».

## Tâches

1. **Commit rouge** (AC-1, AC-2) : générateur et fixtures ; boucle `decode_pages` extraite **sans second temps** (aucun
   changement de comportement) ; `render_scale` extraite, non appelée, rendant `Some(1.0)` ; T1 (rouge), T1-bis, T2, T3,
   T2-diag, T4 (`#[ignore]`), T10 (au commit rouge : (b) et (g) rouges, (a), (c), (d), (e), (f) verts), banc AC-6 ; `printpdf = "0.7"` en
   `dev-dependencies` de `kesh-api` — le `Cargo.lock` ne gagne **qu'une ligne** (`"printpdf",` dans les dépendances de
   `kesh-api`), aucun paquet neuf (le vérifier par `git diff Cargo.lock`) ; T4, T2-diag et le banc joués en local,
   sorties copiées. **La bibliothèque pdfium** n'est pas sur la station ; la release épinglée du `Dockerfile`
   (`:30-38`) s'installe hors du dépôt :

   ```sh
   mkdir -p /home/gcorbaz/devel/kesh-gate-logs/pdfium-7920 && cd /home/gcorbaz/devel/kesh-gate-logs/pdfium-7920 && \
     curl -fsSL -o pdfium-linux-x64.tgz \
       https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F7920/pdfium-linux-x64.tgz && \
     echo "49ab3afbd4e6c1e284b5f2898129c8bb8a10fd785c1c5392c8c1fc70242f9ced  pdfium-linux-x64.tgz" | sha256sum -c - && \
     tar -xzf pdfium-linux-x64.tgz lib/libpdfium.so
   ```

   (puis `LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib` pour T4, T2-diag et le banc).
2. **Correctif** (AC-3 à AC-6) : seconde passe à `render_scale`, `PossibleFormats`, échelon supplémentaire seulement sur
   preuve ; délai du client (T11) ; T4 et banc rejoués ; vérification AC-6-bis exécutée.
3. **Relecture** (AC-8, AC-9) : extraction du décodage (une fonction pour l'import et la réactivation), extraction de
   `QrColumns`, `reactivate_to_complete_with_qr` (`kesh-db`), trace, tests T6, T7, T9.
4. **Textes** (AC-7, AC-10 à AC-13) : quatre locales, replis, garde G13, `README.md`, Vitest et E2E mis à jour, manuels
   (`make fr` dans `docs/manual/`, sous verrou de gate), `api-external.md`, CHANGELOG ; grep de la valeur rejoué
   (`.tex` et PDF).
5. **Captures** et gates.

## Tests et mutations prévus

| Test | Lieu | Ce qu'il prouve |
|---|---|---|
| T1 | `qr_decode.rs` (`mod tests`) | AC-3 : les onze cas de `render_scale` (six pages, cinq `None`) ; rouge au commit rouge |
| T1-bis | `qr_decode.rs` | AC-4 : grandeur causale, < 2,5 px par module au premier rendu, > 4 au second |
| T2 | `qr_decode.rs` | AC-4 : lu à 1414 × 2000, depuis les octets JPEG |
| T2-diag | `qr_decode.rs`, `#[ignore]` | AC-4 : non lu à 595 × 842 (diagnostic, résultat déclaré) |
| T3 | `qr_decode.rs` | AC-4 : chemin image, résolution native, lu |
| T4 | `qr_decode.rs`, `#[ignore]` | AC-2 / AC-5 : sonde, PDF par pdfium avant (non lus) / après (lus), témoin vectoriel lu en première passe, sans-QR |
| T5 | `qr_decode.rs`, `#[ignore]` | AC-6 : banc de coût (D1 à D4) |
| T6 | `tests/inbox_import_e2e.rs` | AC-8 : un PNG d'une QR-facture **à quinze colonnes non `NULL`** (adresse structurée — code postal et localité —, QR-IBAN et référence QRR, message, information de facturation ; payload par `build_payload`, image par `qr_png_from_payload`, `:261`) importé ; ses quinze valeurs **relevées**, puis remises à `NULL` en SQL, statut `discarded` ; le même PNG redéposé → `present`, les quinze colonnes égales aux valeurs relevées, trace `reactivated` avec `"qr": "present"`, `"qr_reread": true` ; variante QR inexploitable (`qr_invoice_png_with`, valeur trop longue) → `unreadable` / `FIELD_TOO_LONG`, colonnes `NULL` ; `import_no_qr_duplicate_and_reactivation` (`:851-883`) reste vert (`none`, `"qr_reread": true`) |
| T7 | `tests/inbox_import_e2e.rs` | AC-8 / AC-9 : ligne `discarded` avec QR dont `creditor_name` est changé en SQL (`'Ancien nom'`) → redéposée, `present`, `creditor_name` **inchangé**, `"qr_reread": false` ; ligne `to_complete` sans QR redéposée → `DUPLICATE`, colonnes inchangées |
| T8 | `import-page.test.ts`, `supplier-documents.spec.ts` (E2-a) | AC-7 : « Aucune QR-facture exploitable » / « aucune QR-facture exploitable » ; l'E2E asserte le badge **et** le rapport |
| T9 | `crates/kesh-db/tests/imported_supplier_invoices_repository.rs`, `#[sqlx::test]` | AC-8 / C-v017-5-9 : `reactivate_to_complete_with_qr` (QR à quinze colonnes non `NULL`, comme T6) sur une ligne `discarded` **pourvue** d'un QR **différent** de celui passé (valeurs distinctes sur les quinze colonnes) rend `false`, quinze colonnes inchangées ; sur une ligne `discarded` sans QR **portant un `supplier_invoice_id` résiduel**, rend `true`, écrit les quinze et remet `supplier_invoice_id` à `NULL` ; sur une ligne `discarded` sans QR, rend `true` et écrit les quinze ; sur une ligne `to_complete`, rend `false` |
| T10 | `qr_decode.rs` | AC-3 : `decode_pages` et faux rendeur, cas (a) à (g) — **en CI** |
| T11 | `imported-supplier-invoices.api.test.ts` (Vitest, fichier neuf) | AC-6 : `triggerInboxImport` passe `{ timeoutMs: INBOX_IMPORT_TIMEOUT_MS, retry: false }`, `INBOX_IMPORT_TIMEOUT_MS === 300_000` |
| T12 | `tests/inbox_import_e2e.rs` ou essai sur installation jetable | AC-6-bis : abandon de la requête, puis second import → 200, fichiers restants repris |

T6, T7, T9, T10 et T11 tournent en CI. Le temps de T2 et T3 en profil de test (`image` et `rxing` sans optimisation,
`Cargo.toml` `[profile.dev]`) est mesuré et déclaré ; s'il dépasse 30 s sur la station, T2/T3 n'engendrent en CI que les
scans à 150 et 200 dpi, celui à 300 dpi passant en `#[ignore]`.

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `render_scale` rend `Some(1.0)` (l'ancienne valeur) | T1 ; T4 (local) |
| M2 | plafond `MAX_RENDER_DPI` retiré du `min` | T1 (A6 portrait, page minuscule) |
| M3 | borne `max_dimension` retirée du `min` | T1 (A4, A3 : sortie arrondie ≤ 2000 px) |
| M3a | garde `is_finite` retirée (seulement `> 0.0`) | T1 (`INFINITY`) |
| M4 | `PossibleFormats` retiré | **aucun test fonctionnel** — coût seulement ; déclarée avec le chiffre du banc |
| M5 | chaque échelon supplémentaire retiré (s'il existe, AC-4) | le cas de T2 qui l'a justifié |
| M6 | réactivation sans relecture (code d'avant) | T6 |
| M7 | garde `creditor_iban IS NULL` retirée **et** décodage forcé pour une ligne pourvue | T7 (`creditor_name` réécrit) |
| M7a | garde `creditor_iban IS NULL` retirée **seule** | T9 (ligne pourvue réécrite) |
| M8 | rapport de réactivation = statut stocké au lieu du résultat de la relecture | T6 |
| M9 | trace sans `"qr_reread"` | T6 |
| M10 | valeur fr-CH du badge rendue à « Sans QR-code » | G13 (`REPLIS_A_SITE_UNIQUE`) |
| M10a | repli de `+page.svelte:237` laissé à « sans QR-code » | G13 |
| M11 | seconde passe retirée (première seule, code d'avant) | T10 (b) — **en CI** ; T4 (local) |
| M12 | première passe retirée (seconde seule) | T10 (a) (second temps appelé) — en CI |
| M12a | passes page par page au lieu de par document | T10 (a), (b) |
| M12b | second temps tenté après une QR-facture inexploitable | T10 (c) |
| M12c | erreur du second rendu rangée dans `last_render_err` | T10 (e) |
| M12d | page en erreur au premier temps rendue au second | T10 (d) |
| M12e | seconde passe sur `page_count` au lieu de `min(page_count, max_pages)` | T10 (f) |
| M13 | colonnes facultatives non écrites par `reactivate_to_complete_with_qr` | T6, T9 |
| M13b | `supplier_invoice_id = NULL` omis de `reactivate_to_complete_with_qr` | T9 |
| M13a | une seule colonne omise (`reference_value`) | T6, T9 |
| M14 | `triggerInboxImport` sans options (délai de 30 s) | T11 |

Mutations sous verrou de gate, motif vérifié unique, restauration par `git checkout -- <fichier>` **puis `touch`**
(`docs/agents-orchestration.md` § 7).

## E2E

Aucune spec neuve : la relecture se prouve en Rust (T6, T7, T9) — un même fichier ne peut pas être illisible puis lisible
pour une même version, et l'E2E n'a pas pdfium. `supplier-documents.spec.ts` (E2-a, l. 94-107) suit le texte : titre,
assertion du rapport, et une assertion du **texte du badge** (`imported-badge-no-qr` contient « Aucune QR-facture
exploitable »). E2E complet au **dernier commit de code** (`docs/testing.md`, 7 à 10 échecs attendus jugés fichier par fichier).

## Captures (thème clair)

Sur une base de démonstration, dossier `kesh-gate-logs/v0175/` : (1) la liste *Documents à traiter* avec une ligne
« Aucune QR-facture exploitable » et une ligne « QR-facture » ; (2) le rapport d'import portant « aucune QR-facture exploitable » ; (3) le
sélecteur de document à joindre (`AttachDocumentPicker`) avec le badge ; (4) **avant / après relecture** : la même pièce
synthétique « Aucune QR-facture exploitable », puis, après retrait et redépôt, « QR-facture » dans la liste ; le rapport de
cette réactivation ne porte que le **compte** des documents importés — une ligne `present` n'y est pas listée
(`+page.svelte:224-226`) (colonnes QR vidées en SQL sur la base de démonstration) ; (5) en `de-CH` (`KESH_LANG=de`) : le badge et le rapport ;
(6) le message de l'écran quand l'import dépasse son délai, s'il est simulé (client à délai court, AC-6-bis) — et,
si l'installation jetable est derrière un proxy dont on peut abaisser le délai, le message de la coupure par le proxy. Jointes au Dev Agent Record
et au prompt de revue de code.

## Gates

`kesh-db` est touché (une fonction de dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Test Locally
First*, exception `kesh-db`). Au dernier commit de code : `scripts/test-fast.sh` sous verrou, `npm run check`,
`npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, E2E complet ; T4, T2-diag et le banc joués en local
avec `libpdfium` épinglée (déclarés comme tels, jamais comme « CI »). Base remise à zéro avant tout gate complet, selon
`docs/agents-orchestration.md` § 4 (bases `kesh_v0175`, `kesh_e2e_v0175`).

## Ce que les gardes ne voient pas

- **La CI ne voit pas le rendu pdfium** (#856) : T4, le diagnostic et le banc ne tournent qu'en local. La logique des
  deux passes est en CI (T10) ; le câblage pdfium du second temps (échelle effectivement appliquée) ne l'est que par T1
  et T4.
- **Un scan réel n'est pas une fixture** : rotation, moiré, QR partiellement masqué, images en 1 bit (CCITT, JBIG2 —
  fréquentes dans les archives PDF/A, où le sous-échantillonnage de pdfium est le plus agressif ; seul le palier (3)
  d'AC-2 (i) s'en approche) ne sont pas simulés. La vérification sur les deux pièces de #854 est celle du Project Lead
  (§ *Fermeture de #854*).
- **Le seuil de lecture n'est pas borné vers le bas** : un scan à 100 dpi ou très flou peut rester illisible ; le
  manuel le dit.
- **La durée d'un import** n'a pas de borne d'horloge côté serveur (C-v017-5-4) : le délai du client et du proxy la
  bornent du côté de l'appelant ; AC-6-bis établit ce qui se passe au-delà ; #855.
- **Pages de plus de 2000 pt** (issue P4 demandée) : déformées au premier rendu (préexistant), rendues sous 72 dpi au second ; rares, aucune
  facture attendue.
- **M4** n'a pas de test fonctionnel (coût seulement).

## Dev Notes

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/qr_decode.rs` — `render_scale`, `MAX_RENDER_DPI`, `decode_pages` et ses deux passes, décodeur,
  générateur, tests T1 à T5, T10.
- `crates/kesh-api/src/inbox_import.rs` — décodage extrait, relecture à la réactivation, trace ; éventuellement la garde
  du verrou (AC-6-bis, si mécanique).
- `crates/kesh-db/src/entities/imported_supplier_invoice.rs` — `QrColumns` extrait de `from_scanned` (champs plats de
  `NewImportedSupplierInvoice` gardés).
- `crates/kesh-db/src/entities/mod.rs` — ré-export de `QrColumns` (`:64-65`).
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` — `reactivate_to_complete_with_qr`.
- `crates/kesh-db/tests/imported_supplier_invoices_repository.rs` — test T9.
- `crates/kesh-api/Cargo.toml` — `printpdf` en `dev-dependencies` ; `Cargo.lock` (une ligne).
- `crates/kesh-api/tests/fixtures/` — quatre PDF synthétiques.
- `crates/kesh-api/tests/inbox_import_e2e.rs` — T6, T7, (T12).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — deux valeurs chacune.
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts` (+ test T11),
  `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`, `.../import-page.test.ts`,
  `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte`,
  `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts` (deux clés à `REPLIS_A_SITE_UNIQUE`),
  `frontend/tests/e2e/supplier-documents.spec.ts`.
- `README.md` (l. 44), `docs/manual/fr/user-manual.tex` (+ `.pdf`), `docs/manual/fr/admin-manual.tex` (+ `.pdf`),
  `docs/api-external.md`, `CHANGELOG.md`.

Pas de découpage (C-v017-5-10).

### Le manuel (axe de revue)

Les phrases prescrites (AC-10, AC-11) doivent être vraies au code livré : « d'abord à 72 dpi, puis, si aucune page ne
livre de QR-facture, une seconde fois », « environ 170 dpi » (2000 / 11,69 in = 171), « au plus 300 dpi », « sans
doublon » (même ligne réactivée), « n'est jamais relu » pour un document lu ou **tant qu'il est joint** à une facture,
« une fois retiré, il ne se télécharge plus depuis la liste », « l'écran attend jusqu'à cinq minutes » (T11), la
conduite au délai dépassé (AC-6-bis, exécutée). Contrôler le **PDF** aplati :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

### Références

- `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs:282-290` (`scale_page_by_factor`), `:314-327`
  (maxima), `:716-835` (`apply_to_page`), `:770-773` (échelle 1,0), `:825-826` (arrondi).
- `rxing-0.7.1/src/helpers.rs:232-249` (`detect_multiple_in_luma_with_hints`),
  `rxing-0.7.1/src/decode_hints.rs:264` (`PossibleFormats`), `:270` (`TryHarder`), `:322` (`AlsoInverted`),
  `rxing-0.7.1/src/filtered_image_reader.rs`, `rxing-0.7.1/src/multi/generic_multiple_barcode_reader.rs:113`,
  `rxing-0.7.1/src/qrcode/qr_code_writer.rs:80` (niveau L par défaut).
- `printpdf-0.7.0/src/xobject.rs:136-159` (`ImageXObject`, filtre DCT sans la fonctionnalité `embedded_images`) ;
  `pdf_document.rs:793` (identifiant d'instance tiré à chaque écriture).
- `crates/kesh-api/src/inbox_import.rs:172-207` (verrou de run), `routes/imported_supplier_invoices.rs:62`,
  `frontend/src/lib/shared/utils/api-client.ts:42`, `:53`, `:59-64`, `:589-600`.
- Norme SIX (QR-facture) : QR de 46 × 46 mm, section paiement de 210 × 105 mm en bas d'une A4, récépissé de 62 mm.

### Issues liées

- **#855** (P3, dette) — le décodage QR occupe un fil de l'exécuteur tokio ; parent de #820 pour CAMT/CSV.
- **#856** (P3, test) — le chemin PDF du décodage QR n'est exercé par aucun test qui tourne.
- **#858** (demande d'évolution) — relire le QR d'un document à traiter depuis l'écran (bouton), que #854 propose et
  que la story ne livre pas (C-v017-5-8).
- #854 commentée le 2026-10-31, puis corrigée (commentaires correctifs posés sur #854 et #858 après la validation P2).
- Issue **P4 à ouvrir** par l'orchestrateur (texte au rapport de la remédiation P3) : une page de PDF de plus de 2000 pt
  est déformée au premier rendu.
- Issue P2 **à demander** si AC-6-bis montre une fuite du verrou de run.

## Questions au Project Lead

- **Q1** — Le badge de la liste disait « Sans QR-code ». Il s'affiche pour toute pièce **sans données de QR
  enregistrées**, c'est-à-dire dans cinq cas, que la liste ne distingue pas :
  1. le document ne porte aucun QR — au rapport d'import : « aucune QR-facture exploitable » ;
  2. il porte un QR d'un autre usage (lien, code produit), lu puis écarté — au rapport : « aucune QR-facture
     exploitable » ;
  3. il porte une QR-facture que la lecture n'a pas trouvée (le cas de vos deux factures communales scannées) — au
     rapport : « aucune QR-facture exploitable » ;
  4. il porte une QR-facture lue puis refusée (IBAN invalide, valeur trop longue, montant hors limites) — au rapport :
     « QR illisible » et le motif ;
  5. le PDF n'a pas pu être rendu (bibliothèque absente, trop de pages) — au rapport : « QR illisible » et le motif.
  « Sans QR-code » est faux aux cas 2, 3 et 4 ; « Aucun QR lu » aux cas 2 et 4 ; « Aucune QR-facture lue » au cas 4.
  La fiche retient **« Aucune QR-facture exploitable »** (de « Keine verwertbare QR-Rechnung », it « Nessuna fattura QR
  utilizzabile », en « No usable QR-bill »), vrai dans les cinq cas ; tant que vous n'avez pas répondu, il fait foi.
  **Limite** : la liste ne dit pas lequel des cinq cas ; le rapport, au moment de l'import, sépare seulement les cas 1
  à 3 des cas 4 et 5. Ce libellé vous convient-il, ou en préférez-vous un autre ?
- **Q2** — Vos deux factures communales déjà importées : après la mise à jour, le redépôt (Télécharger, Retirer,
  déposer, Importer) est un geste de votre part sur vos livres, et vous rapportez seulement « lues » ou « non lues »
  (§ *Fermeture de #854*). Cela vous convient-il ?

## Dev Agent Record

### Agent Model Used

Opus 5.5 (Claude Code), autonomie, worktree `/home/gcorbaz/devel/kesh-0176`, 2026-10-31.

### Completion Notes List

#### Étape 0 — les cinq LOW de la validation P5, lignes lues

- **P1** (AC-6, D1) — ligne lue (fiche, l. 239) : « QR-facture : **D1** 20 × la page de `scan-sans-qr.pdf` ; » →
  récrite « D1 20 × une page engendrée en mémoire à 300 dpi, de même facture que `scan-sans-qr.pdf` (AC-1) ».
- **P2** (citation de `+page.svelte`) — `grep -nF "isApiError(err) && err.message"` rend `107:` ; le `notifyError(` s'ouvre
  l. 106 et se ferme l. 109 (lu, `sed -n 100,112p`). AC-10 cite désormais `:106-109`.
- **P3** (`qr_decode.rs:93`) — `grep -nF "Err(_) => Vec::new()" crates/kesh-api/src/qr_decode.rs` rend
  `92:        Err(_) => Vec::new(),`. Corrigé en `:92` à AC-7 (fiche) et à C-v017-5-7 (registre, amendement P4).
- **P4** (`admin-manual.tex:2555`) — `sed -n 2555p` : « … reste accepté) : sans QR, elle applique la même validation … ».
  Ajouté à l'inventaire d'AC-7 et à la liste des formes conservées d'AC-11.
- **P5** (C-v017-5-10) — ligne lue (registre) : « Risque accepté : une story à six modules de code `kesh-db`/`kesh-api`
  liés (cinq fichiers de production : … ». Récrite « six modules de code (quatre dans `kesh-db`/`kesh-api`, cinq
  fichiers de production : … ».
- Les citations `:93` et `:103-106` de l'entrée datée « Validation P4 » du Change Log ne sont pas réécrites (relevé
  daté) : elles se lisent `:92` et `:106-109`.

#### Étape 0 — « § À recontrôler à l'étape 0 »

La fiche ne porte **aucune** section de ce nom (`grep -n "recontrôler"` sur la fiche et le registre : aucune sortie).
À la place, les prémisses de § *Ce que le code dit* ont été relues au code de `8e126cfd` : `qr_decode.rs:163-165`
(rendu sans facteur d'échelle), `:46-53` (`DecodeConfig::default`), `:84-94` (`detect_multiple_in_luma`),
`inbox_import.rs:172-207` (verrou de run, `RELEASE_LOCK` après `process_inbox`), `:476-537` (réactivation avant le
décodage), `:539-576` (décodage), `imported_supplier_invoices.rs:480-499` (`reactivate_to_complete`) ;
`sqlx-core-0.8.6/src/pool/connection.rs:199-210` et `:275-320` (une `PoolConnection` abandonnée **retourne au pool**
après un `ping`, sans `close_on_drop` ; `after_release` n'est pas posé par le pool de production, `main.rs:82-85`).
Toutes exactes.

#### Commit rouge — la reproduction (AC-1, AC-2)

**pdfium** : release `chromium/7920` du `Dockerfile`, installée sous
`/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/` par les commandes de la tâche 1 (`sha256sum -c` : `pdfium-linux-x64.tgz:
OK`) ; `libpdfium.so` : SHA-256 `687dce861f959c7097d47c5864509d51a926a71b38322596a8ee3e7a99c6b96e`.

**Branche d'arrêt (i), déclenchée puis franchie par les paliers écrits d'avance.** Avec la dégradation d'AC-1 (flou
0,1 mm, bruit ± 12), T4 au code d'avant **lit** les trois scans (journal `kesh-gate-logs/v0175/rouge-t4.log`) :

```
T4 (α) sonde : 595 × 842 px, 13 niveaux de gris sur plus de 1 % des pixels
T4 (β) qr-scan-150dpi.pdf : LU (rendus au second temps : 0)
T4 (β) qr-scan-200dpi.pdf : LU (rendus au second temps : 0)
T4 (β) qr-scan-300dpi.pdf : LU (rendus au second temps : 0)
T4 (β) scan-sans-qr.pdf : Ok(None) (rendus au second temps : 0)
T4 (γ) témoin vectoriel : LU (rendus au second temps : 0)
T2-diag qr-scan-{150,200,300}dpi.pdf à 595 × 842 : LU (les trois)
```

Le mandat (« même après les paliers de dégradation écrits d'avance ») a été lu comme l'ordre de jouer les paliers
d'AC-2 (i), **dans leur ordre**, sans régler aucun paramètre sur le résultat (C-v017-5-16) :

| palier | dégradation | T4 au code d'avant (pdfium) | T2-diag (595 × 842) | journal |
|---|---|---|---|---|
| 0 (AC-1) | flou 0,1 mm, bruit ± 12 | 3 scans **lus** | 3 lus | `rouge-t4.log` |
| 1 | flou 0,1 mm, bruit **± 24** | 150 et 200 dpi **lus** | 3 lus | `palier1.log` |
| 2 | flou **0,2 mm**, bruit ± 24 | **3 scans non lus** (`Ok(None)`) | 3 non lus | `palier2.log`, `palier2d.log` |

> *Annotation datée du 2026-10-31, revue de code P1 (A1) — ce tableau affirme plus que ce qui a tourné.* Le générateur
> écrivait chaque fixture avant de contrôler la suivante : au palier 1 (`palier1.log`) et au premier essai du palier 2
> (`palier2.log`), il a paniqué sur `qr-scan-300dpi.pdf` (1 613 827 puis 1 606 339 octets), si bien que
> `qr-scan-300dpi.pdf` et `scan-sans-qr.pdf` n'ont **pas** été régénérés : T4 y relisait ceux du palier 0. D'où, dans
> `palier2.log`, « qr-scan-300dpi.pdf : LU ». Ce qui vaut : au palier 1, la ligne T4 ne vaut que pour 150 et 200 dpi
> (régénérées, encore lues) ; la colonne T2-diag, elle, porte sur le générateur en mémoire (dégradation complète) et
> vaut pour les trois. Au palier 2, `palier2.log` ne prouve rien pour 300 dpi ; **la preuve du palier 2 est
> `palier2d.log`**, joué sur les quatre fixtures finales (les trois scans en `Ok(None)` au premier temps, T2-diag
> non lu). Depuis la remédiation P1, le générateur contrôle toutes les tailles avant d'écrire une seule fixture.

Le palier 2 est le premier qui reproduit le défaut : il devient celui des fixtures (`DEGRADATION`,
`qr_decode_synthetique.rs`). Le palier 3 (binarisation) n'a pas été joué.

**Plafond de 1 Mio** (AC-1), tenu par les moyens prescrits, sans le relever : au palier 2, `qr-scan-300dpi.pdf` pesait
1 606 339 octets (qualité 65) → **bruit restreint à la section paiement** pour les trois fixtures à QR (le QR et la croix
y sont tout entiers) ; `scan-sans-qr.pdf`, bruit sur toute la page, pesait 1 610 809 octets (qualité 65), puis
1 417 096 (qualité **60**) → **200 dpi** (AC-1). Valeurs retenues : qualité JPEG 65 pour les scans à QR, 60 pour la page
sans QR ; graine fixe `0x5EED_0854_0000_0001`.

| fixture | octets |
|---|---|
| `qr-scan-150dpi.pdf` | 237 575 |
| `qr-scan-200dpi.pdf` | 388 270 |
| `qr-scan-300dpi.pdf` | 774 286 |
| `scan-sans-qr.pdf` (200 dpi) | 652 699 |

Payload synthétique : ≥ 280 octets, QR de version 12 à 15 (test `payload_synthetique_version_12_a_15`, vert).

**T4 au commit rouge** (fixtures du palier 2) — commande :
`LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib cargo test -p kesh-api --lib -- --ignored
--nocapture qr_decode::tests::t4_pdfium qr_decode::tests::t2_diag` :

```
T4 (α) sonde : 595 × 842 px, 12 niveaux de gris sur plus de 1 % des pixels
T4 (β) qr-scan-150dpi.pdf : Ok(None) (rendus au second temps : 0)
T4 (β) qr-scan-200dpi.pdf : Ok(None) (rendus au second temps : 0)
T4 (β) qr-scan-300dpi.pdf : Ok(None) (rendus au second temps : 0)
T4 (β) scan-sans-qr.pdf : Ok(None) (rendus au second temps : 0)
T4 (γ) témoin vectoriel : LU (rendus au second temps : 0)
T2-diag qr-scan-150dpi.pdf à 595 × 842 : non lu
T2-diag qr-scan-200dpi.pdf à 595 × 842 : non lu
T2-diag qr-scan-300dpi.pdf à 595 × 842 : non lu
test qr_decode::tests::t4_pdfium_scans_synthetiques ... FAILED   (scans non lus : les trois)
```

Sonde verte, témoin vectoriel lu : le chemin pdfium fonctionne, et la QR-facture scannée n'est pas lue au premier temps.

**Tests d'hôte au commit rouge** (`cargo test -p kesh-api --lib qr_decode`, journal `rouge-hote2.log`) :
`12 passed; 14 failed; 5 ignored`. Rouges : les onze cas de T1 (`render_scale_*`, la fonction rend `Some(1.0)`),
T1-bis (`grandeur_causale_pixels_par_module` — il passe par `render_scale`, d'où son rouge), T10 (b) et (g). Verts :
T10 (a), (c), (d), (e), (f) ; T2 (lu à 1414 × 2000 depuis les octets JPEG) ; T3 (lu à la résolution native) ;
`fixtures_sous_le_plafond`, `payload_synthetique_version_12_a_15`, et les tests d'avant. Durée du module en profil de
test : 12,1 s (T2 et T3 compris, trois scans chacun) — sous les 30 s : le scan à 300 dpi reste en CI.

**Banc au commit rouge** (première passe seule ; binaire `--release` copié en `kesh-gate-logs/v0175/banc-binaire-rouge`,
sous verrou et `mem-guard`, `/usr/bin/time -v`, journal `banc-rouge.log`) :

| mesure | commit rouge |
|---|---|
| `qr-scan-150dpi.pdf`, médiane par page | 26,8 ms (`Ok(None)`) |
| `qr-scan-200dpi.pdf` | 30,0 ms (`Ok(None)`) |
| `qr-scan-300dpi.pdf` | 37,1 ms (`Ok(None)`) |
| `scan-sans-qr.pdf` | 28,4 ms (`Ok(None)`) |
| D1 (20 × A4 sans QR, 300 dpi, en mémoire) | 779 ms (39,0 ms par page) |
| D2 (20 × carrée 842 pt, 200 dpi) | 722 ms (36,1 ms par page) |
| D3 (20 × A4 à douze QR non SPC, 200 dpi) | 1 230 ms (61,5 ms par page) |
| D4 (facture vectorielle de 3 pages, QR en page 3) | 38,9 ms, lue, 0 rendu au second temps |
| pointe de mémoire résidente du binaire | 189 552 Kio |

#### Le correctif (AC-3 à AC-5)

`render_scale` (garde explicite `is_finite`, `> 0`, borne non nulle ; `min` des trois termes), seconde passe de
`decode_pages` sur les pages rendues au premier temps, seulement sans QR-facture à la première passe ; une erreur du
second rendu journalisée (`tracing::warn!`, numéro de page et message pdfium, aucun contenu) ; décodeur
`detect_multiple_in_luma_with_hints` avec `PossibleFormats = {QR_CODE}` et `TryHarder`. **Aucun échelon
supplémentaire** (AC-4) : T2 lit les trois scans sans lui — non écrit, M5 sans objet.

**T4 au correctif** (même commande, journal `kesh-gate-logs/v0175/vert-t4.log`) :

```
T4 (α) sonde : 595 × 842 px, 12 niveaux de gris sur plus de 1 % des pixels
T4 (β) qr-scan-150dpi.pdf : LU (rendus au second temps : 1)
T4 (β) qr-scan-200dpi.pdf : LU (rendus au second temps : 1)
T4 (β) qr-scan-300dpi.pdf : LU (rendus au second temps : 1)
T4 (β) scan-sans-qr.pdf : Ok(None) (rendus au second temps : 1)
T4 (γ) témoin vectoriel : LU (rendus au second temps : 0)
T2-diag qr-scan-{150,200,300}dpi.pdf à 595 × 842 : non lu (les trois)
test qr_decode::tests::t4_pdfium_scans_synthetiques ... ok
test qr_decode::tests::decode_spc_from_pdf_single_page ... ok
```

IBAN, montant et référence égaux au payload (assertions de T4). Le test réparé `decode_spc_from_pdf_single_page` lit
`qr-scan-300dpi.pdf` et passe. Tests d'hôte du module : `26 passed; 0 failed; 5 ignored` (9,0 s, journal
`vert-hote.log`).

**Banc au correctif** (binaire `--release`, sous verrou et `mem-guard`, `/usr/bin/time -v`, journal `banc-vert.log`) :

| mesure | commit rouge | correctif | plafond |
|---|---|---|---|
| `qr-scan-150dpi.pdf`, médiane par page | 26,8 ms (non lu) | 60,7 ms (**lu**) | ≤ 1,0 s ✓ |
| `qr-scan-200dpi.pdf` | 30,0 ms (non lu) | 67,6 ms (**lu**) | ✓ |
| `qr-scan-300dpi.pdf` | 37,1 ms (non lu) | 81,1 ms (**lu**) | ✓ |
| `scan-sans-qr.pdf` (deux passes) | 28,4 ms | 60,8 ms | ✓ |
| D1, 20 × A4 sans QR à 300 dpi | 779 ms | 1 523 ms (76,2 ms par page), 20 rendus au second temps | — |
| D2, 20 × carrée 842 pt à 200 dpi | 722 ms | 1 574 ms (78,7 ms par page), 20 | — |
| D3, 20 × A4 à douze QR non SPC | 1 230 ms | **7 295 ms** (364,7 ms par page), 20 | pire document ≤ 20 s ✓ |
| D4, facture vectorielle de 3 pages | 38,9 ms | 12,5 ms, 0 rendu au second temps | ± 10 % : **hors bande, vers le bas** |
| pointe de mémoire résidente | 189 552 Kio | 200 100 Kio (+10,3 Mio) | ≤ + 128 Mio ✓ |

Pire document : D3, 7,3 s. **D4 sort de la bande de ± 10 % par le bas** (− 68 %) : le premier temps n'a pas changé de
travail, mais le décodeur n'essaie plus que le format QR (M4, mesuré ci-dessous) ; une seule mesure par document. Écart
déclaré : le plafond visait une régression, c'est une baisse.

**Contre les deux délais** (station de développement, 32 cœurs ; un rendu et un décodage par fil, le passage étant
séquentiel) : une page scannée sans QR-facture coûte 61 à 81 ms (fixtures, D1, D2), jusqu'à 365 ms quand elle porte une
douzaine de QR d'un autre usage (D3). Sous **30 s** : environ 390 pages scannées (D1), 80 dans le pire cas (D3) ; sous
**300 s** : environ 3 900 pages (D1), 820 (D3). **Projection pour un NAS** (facteur **× 10 déclaré, non mesuré** — un
processeur de NAS d'entrée de gamme contre cette station) : environ 39 pages sous 30 s et 390 sous 300 s ; 8 et 80 dans
le pire cas.

#### L'abandon de la requête (AC-6-bis) — vérifié par exécution : le verrou **fuyait**

Test d'intégration `import_abandoned_by_client_releases_run_lock` (T12, `tests/inbox_import_e2e.rs`) : douze PNG de
bruit de 1500 × 1500, client `reqwest` à délai de 2 s, attente d'un dossier stable pendant 2 s, `IS_FREE_LOCK` sur une
connexion neuve hors du pool, second import. Commande : `cargo nextest run -p kesh-api -E
'test(import_abandoned_by_client_releases_run_lock)' --no-capture`.

1. Premier essai, **pool de `#[sqlx::test]`** : vert — mais ce pool ferme toute connexion inactive au bout d'une seconde
   (`sqlx-mysql-0.8.6/src/testing/mod.rs:181`) ; la production n'a pas ce réglage (`main.rs:82-85`, délais par défaut
   de `sqlx`, `idle_timeout` de dix minutes, `sqlx-core-0.8.6/src/pool/options.rs:161`). Vert à tort.
2. **Pool réglé comme la production**, code d'avant la garde (journal `t12-rouge.log`) :

```
T12 : abandon après 2s ; 5 fichier(s) dans le dossier à l'abandon, 5 après stabilisation (4.12s) ; 7 document(s) dans la liste
T12 : IS_FREE_LOCK('kesh_inbox_import:_sqlx_test_…') = Some(0)
T12 : second import → 200 {"accepted":[… bruit-04 … bruit-00, "qr":"none" …],"failed":[]…}
test import_abandoned_by_client_releases_run_lock ... FAILED   (le verrou de run a fui après l'abandon)
```

   Le verrou **fuit** : la connexion, rendue au pool sans `RELEASE_LOCK`, le garde ; et le second import répond **200**
   parce qu'il reprend cette même session (verrou ré-entrant) — sans la libérer. Ce que l'on voit aussi : hyper
   **abandonne** le gestionnaire à l'abandon du client — le passage s'arrête au fichier en cours (7 traités, 5 restés,
   repris par le second import, aucun doublon).
3. **Correction mécanique** (C-v017-5-18) : `lock_conn.close_on_drop()` après l'acquisition. Même test (journal
   `t12-vert.log`) : `IS_FREE_LOCK … = Some(1)`, second import **200**, douze documents, dossier vide, aucun
   `DUPLICATE` — **verrou libéré**.

Issue P2 demandée à l'orchestrateur (la fuite existait depuis la 12-5c ; corrigée ici). Les deux manuels disent la
conduite (AC-10, AC-11).

#### La relecture (AC-8, AC-9)

`lire_qr` (une seule lecture, import et réactivation) et `LectureQr` (classement : exploitable, inexploitable avec son
code, aucune, déjà stockée) dans `inbox_import.rs` ; `QrColumns::from_scanned` extraite de
`NewImportedSupplierInvoice::from_scanned` (champs plats gardés) ; `reactivate_to_complete_with_qr` (une instruction,
garde `status = 'discarded' AND creditor_iban IS NULL`, `supplier_invoice_id = NULL`). La relecture se fait **avant**
la transaction (le rendu ne tient aucune connexion). Trace `reactivated` : `qr`, `qr_reread`. T6
(`reactivation_rereads_a_document_without_qr_data`), T7 (`reactivation_never_rewrites_stored_qr_and_duplicates_stay`),
T9 (`reactivate_with_qr_writes_only_empty_discarded_rows`) verts ; `import_no_qr_duplicate_and_reactivation` vert,
avec l'assertion `"qr_reread": true` ajoutée.

#### Mutations (campagne sous verrou de gate, `kesh-gate-logs/v0175/mutations.py`, journaux `mutations/*.log`)

Chaque mutation : motif vérifié unique (le script s'arrête sinon), test ciblé rejoué, restauration par
`git checkout -- <fichier>` puis `touch` ; `git status` propre à la fin de la campagne. Une mutation est tuée quand la
commande échoue **sur un test** (nextest : code 100 ; Vitest : 1) — aucune n'a échoué à la compilation.

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `render_scale` rend `Some(1.0)` | les onze T1, T1-bis |
| M2 | plafond `MAX_RENDER_DPI` retiré du `min` | T1 A6 portrait, page minuscule |
| M3 | borne `max_dimension` retirée du `min` | T1 A4 portrait, A4 paysage, A3, section paiement |
| M3a | garde `is_finite` retirée | T1 hauteur infinie |
| M4 | `PossibleFormats` retiré | **aucun test fonctionnel** (déclarée) — coût mesuré ci-dessous |
| M5 | échelon supplémentaire | sans objet : non écrit (AC-4) |
| M6 | réactivation sans relecture | T6 ; `import_no_qr_duplicate_and_reactivation` |
| M7 | garde `creditor_iban IS NULL` retirée **et** relecture forcée | T7 (`creditor_name` réécrit) |
| M7a | garde retirée seule | T9 |
| M8 | rapport = statut stocké | T6 |
| M9 | trace sans `"qr_reread"` | T6, T7, `import_no_qr_duplicate_and_reactivation` |
| M10 | valeur fr-CH du badge à « Sans QR-code » | G13 |
| M10a | repli de `+page.svelte` (rapport) à « sans QR-code » | G13 |
| M11 | seconde passe retirée | T10 (b), (g) — en CI ; T4 au commit rouge (local) |
| M12 | première passe retirée | T10 (a), (b), (c), (d) |
| M12a | passes page par page | T10 (a), (b) |
| M12b | second temps après une QR-facture inexploitable | T10 (c) |
| M12c | erreur du second rendu rangée dans `last_render_err` | T10 (e) |
| M12d | page en erreur au premier temps rendue au second | T10 (d) |
| M12e | seconde passe sur `page_count` | T10 (f) |
| M13 | colonnes facultatives (ligne 1, ligne 2, code postal, localité) non écrites | T6, T9 |
| M13a | `reference_value` non écrite | T6, T9 |
| M13b | `supplier_invoice_id = NULL` omis | T9 |
| M14 | `triggerInboxImport` sans options | T11 |
| M-verrou | `close_on_drop` retiré (mutation ajoutée, C-v017-5-18) | T12 |

Vingt-trois mutations jouées, toutes tuées ; M4 déclarée sans test fonctionnel ; M5 sans objet.

**M4, le chiffre** (banc `--release` avec `PossibleFormats` retiré, restauré ensuite ; journal `banc-m4.log`) : médianes
par page 305 / 194 / 222 / 159 ms (contre 61 / 68 / 81 / 61 ms au correctif) ; D1 3,5 s, D2 3,7 s, **D3 17,6 s** (contre
7,3 s, à 2,4 s du plafond de 20 s) ; D4 48,0 ms (38,9 ms au commit rouge, 12,5 ms au correctif) ; pointe 204 852 Kio.
Sans le format QR seul, la seconde passe coûterait 2,5 à 3 fois plus, et D4 revient dans l'ordre de sa mesure rouge :
c'est `PossibleFormats` qui fait sortir D4 de sa bande par le bas.

#### Gates au dernier commit de code (`9c26f50d`)

- **Backend** : `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest), base `kesh_v0175` remise à zéro avant,
  sous verrou — **4610 exécutés, 4610 passés, 12 ignorés** (journal `gate-final.log`). Gate complet aussi joué en cours
  de travail après le commit de la relecture (`kesh-db` touché) : 4610/4610, 12 ignorés (`gate-relecture2.log`) ; le
  premier essai (`gate-relecture.log`) avait échoué à **clippy** (`large_enum_variant` sur `LectureQr`,
  `type_complexity` sur un test) — corrigé (`Box`, alias de type) avant le second.
- **Frontend** : `npm run check` 0 erreur (29 avertissements, préexistants, dans des fichiers que la story ne touche
  pas) ; `npm run lint-i18n-ownership` PASS ; `npm run test:unit` **191 fichiers, 2293 tests passés** ; `npm run build`
  vert.
- **E2E complet** (montage de `docs/testing.md`, port 3175, base `kesh_e2e_v0175` reconstruite, dossiers
  `/tmp/kesh-e2e-v0175-1/`, backend tué par PID) : **330 passés, 7 échecs, 18 ignorés** (13,5 min). Les sept échecs, un
  à un contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **KF-029 (#97)**, tous attendus ;
  aucun autre. `supplier-documents.spec.ts:94` (E2-a, badge « Aucune QR-facture exploitable ») vert.
- **Local seulement** (déclarés comme tels) : T4, T2-diag, `decode_spc_from_pdf_single_page`, le banc, avec
  `libpdfium` épinglée.

#### Captures (thème clair, `kesh-gate-logs/v0175/captures/`)

Backend de la branche avec `libpdfium` chargée, base `kesh_e2e_v0175` après seed `with-data`, fixtures synthétiques
seulement : `1-2-liste-et-rapport-fr.png` (ligne « QR-facture » d'un scan à 200 dpi lu au second temps, ligne « Aucune
QR-facture exploitable », rapport « aucune QR-facture exploitable ») ; `3-selecteur-fr.png` (sélecteur de document à
joindre, les deux badges) ; `4a-avant-relecture-fr.png`, `4b-apres-relecture-rapport-fr.png`,
`4c-apres-relecture-ligne-fr.png` (même pièce, colonnes QR vidées en SQL, puis retirée, redéposée, relue :
« QR-facture ») ; `5-liste-et-rapport-de.png` (`KESH_LANG=de` : « Keine verwertbare QR-Rechnung ») ;
`6a-proxy-504-fr.png` (« Erreur serveur (504) », réponse 504 HTML simulée) et `6b-connexion-fermee-fr.png` (connexion
fermée sans réponse, simulée). Le message du délai de cinq minutes de l'écran n'est pas capturé (non simulable sans
attendre cinq minutes).

#### Écarts à la fiche

1. **Fixtures au palier 2** (C-v017-5-16) et non à la dégradation d'AC-1 ; plafond tenu par le bruit restreint à la
   section paiement et la page sans QR à 200 dpi, qualité 60 (C-v017-5-17).
2. **T1-bis rouge au commit rouge** : il passe par `render_scale` (la fiche ne prescrivait pas son état rouge).
3. **D4 hors de sa bande de ± 10 %, vers le bas** (− 68 %), expliqué par M4.
4. **AC-6-bis : le verrou fuyait** ; correction mécanique dans la story (C-v017-5-18), mutation `M-verrou` ajoutée.
   T12 tourne avec un pool réglé comme la production (le pool de `#[sqlx::test]` masquait la fuite).
5. **§ « À recontrôler à l'étape 0 »** absent de la fiche : les prémisses de § *Ce que le code dit* ont été relues à sa
   place.
6. **Édition par script** : contrairement à `docs/agents-orchestration.md` § 7, trois modifications de code ont été
   faites hors de l'outil d'édition, toutes sur du code **neuf** de la story, avant tout commit : un remplacement
   `python3` dans `qr_decode_synthetique.rs` (factorisation de `dessiner_modules`), et l'ajout de T12 en fin de
   `tests/inbox_import_e2e.rs` par `cat >>`. Fichiers relus et formatés (`cargo fmt`) ; aucun dégât constaté.
7. **Gates ciblés hors verrou** avant la consigne de l'orchestrateur sur la saturation mémoire (fixtures, tests ciblés,
   Vitest ciblé, `npm run check`) ; tout ce qui a suivi la consigne est passé sous verrou et `mem-guard`. Aucun OOM
   subi par cette story.

#### Remédiation de la revue de code P1 (2026-10-31)

Revue P1 (Opus, B, E, A) : 0 CRITICAL, 0 HIGH, 1 MEDIUM (A1), 14 LOW (B-1 à B-5, E-1 à E-4, A2 à A6). Prémisse de
chaque finding relue au code avant remède ; E-3 n'était **qu'à moitié vraie** (C-v017-5-19).

- **A1** : générateur à contrôle préalable de toutes les tailles ; annotations datées du tableau des paliers et de
  C-v017-5-16 ; réponse à la question de B en annotation de C-v017-5-17 (la restriction du bruit a été décidée sur la
  taille, après la reproduction, et laisse la section paiement entière dégradée).
- **E-3** : `echelle_du_second_temps` — le second rendu est omis seulement quand il redonnerait le premier (page plus
  grande que la borne dans les deux sens aux proportions de la borne, ou page à l'échelle 1) ; une page plus grande
  dans un seul sens, déformée au premier temps, garde son second rendu (lu dans `render_config.rs:763-822`). Quatre
  tests d'hôte ; mutation **M15** tuée par `second_temps_page_plus_grande_que_la_borne_dans_les_deux_sens_aux_memes_proportions`.
- **B-2** : T10 (h) (`Ok(None)` au premier temps, lu au second) et (i) (inexploitable au seul second temps) ; mutation
  **M16** (`Ok(None) => *ok = true` → `{}`) tuée par T10 (h).
- **B-3** : **T3-bis**, non ignoré : extrait le flux JPEG de chaque fixture versionnée (sans pdfium), l'exige identique
  au générateur, le lit à la résolution native et à 1414 × 2000 ; la page sans QR rend `Ok(None)`.
- **B-1 = E-4** : `assert!(restants > 0)` dans T12. **E-1, E-2, B-5** : commentaires corrigés.
- **B-4 = A5** : manuels et README disent le fait (« sur des scans de test… une pièce réelle peut encore y échapper »).
  **A3** : trace `reactivated` documentée dans `api-external.md`. **A4** : entrée CHANGELOG propre à **#863** ; le
  manuel d'administration la cite. **A6** : en-tête de la fiche (C-v017-5-1 à 19).
- **A2** : T4 et le banc **rejoués au dernier commit de code** (`dee8f982` ; le journal porte `25b2367a`, commit de
  textes seulement) — `p1-t4.log` : sonde verte, trois scans lus au second temps (un rendu chacun), sans QR `Ok(None)`,
  témoin vectoriel lu sans second temps, `decode_spc_from_pdf_single_page` vert ; `p1-banc.log` : médianes 57 / 65 /
  80 / 58 ms par page, D1 1,48 s, D2 1,54 s, D3 7,19 s, D4 11,0 ms (0 second rendu), pointe 198 944 Kio.

Gates au dernier commit de code : **backend** `test-fast.sh`, base remise à zéro — **4617 exécutés, 4617 passés, 12
ignorés** (`p1-gate.log`) ; **frontend** check 0 erreur (29 avertissements préexistants), lint PASS, **Vitest 191
fichiers, 2293 tests**, build vert ; **E2E complet** (base `kesh_e2e_v0175` reconstruite, `/tmp/kesh-e2e-v0175-4/`, backend tué par PID) : **330 passés, 7 échecs, 18 ignorés** — `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`, tous **KF-029 (#97)**, attendus ; E2-a vert (`p1-e2e-run.log`).

### File List

- `crates/kesh-api/src/qr_decode.rs` — `MAX_RENDER_DPI`, `render_scale`, `Temps`, `decode_pages` (deux passes),
  `decode_spc_from_pdf_bytes_observed`, décodeur QR seul ; tests T1, T1-bis, T2, T2-diag, T3, T4, T10, banc.
- `crates/kesh-api/src/qr_decode_synthetique.rs` (neuf, tests seulement) — générateur des scans synthétiques.
- `crates/kesh-api/src/inbox_import.rs` — `lire_qr`, `LectureQr`, relecture à la réactivation, trace, `close_on_drop`.
- `crates/kesh-api/tests/inbox_import_e2e.rs` — T6, T7, T12 ; assertion ajoutée à
  `import_no_qr_duplicate_and_reactivation`.
- `crates/kesh-api/tests/fixtures/{qr-scan-150dpi,qr-scan-200dpi,qr-scan-300dpi,scan-sans-qr}.pdf` (neufs).
- `crates/kesh-api/Cargo.toml`, `Cargo.lock` (une ligne : `printpdf` en `dev-dependencies`).
- `crates/kesh-db/src/entities/imported_supplier_invoice.rs` (`QrColumns`), `entities/mod.rs`,
  `repositories/imported_supplier_invoices.rs` (`reactivate_to_complete_with_qr`),
  `crates/kesh-db/tests/imported_supplier_invoices_repository.rs` (T9).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
- `frontend/src/lib/features/imported-supplier-invoices/imported-supplier-invoices.api.ts` (+ `.api.test.ts`, neuf,
  T11), `AttachDocumentPicker.svelte`, `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`,
  `import-page.test.ts`, `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`,
  `frontend/tests/e2e/supplier-documents.spec.ts`.
- `docs/manual/fr/user-manual.tex` (+ `.pdf`), `docs/manual/fr/admin-manual.tex` (+ `.pdf`), `docs/api-external.md`,
  `README.md`, `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/v017-5-qr-scanne.md`, `v017-5-choix-autonomes.md` (C-v017-5-16 à 18),
  `v017-5-review-prompt-p1.md` (neuf), `sprint-status.yaml`.

## Change Log

- 2026-10-31 — Fiche créée (Opus 5.5, autonomie). Cause H1 (rendu à 72 dpi) établie au code ; preuve prescrite (T4).
  Registre C-v017-5-1 à C-v017-5-11. Prochaine étape : validation P1 (`v017-5-validate-prompt-p1.md`, Sonnet, R et F).
- 2026-10-31 — **Validation P1** (Sonnet, lentilles R et F, contexte frais) : 0 CRITICAL, 0 HIGH, 8 MEDIUM (R1 à R5, F1
  à F3), 23 LOW (R6 à R15, F4 à F16). Remédiation (Opus 5.5), prémisses vérifiées au code et dans la source des
  dépendances (`printpdf-0.7.0/src/pdf_document.rs:63`, `utils.rs` ; `config.rs:524` ; `admin-manual.tex:341`,
  `:920-923` ; `imported_supplier_invoices.rs:339-341` ; `entities/imported_supplier_invoice.rs:260-296` ; `README.md:44`),
  avec les décisions de l'orchestrateur : **rendu en deux temps** (72 dpi, puis `render_scale` seulement si la page ne
  livre aucun QR — R4, C-v017-5-2 récrit) ; fixtures produites une fois, versionnées comme données, preuve sur le
  décodage, plafond de 1 Mio (R1, F10) ; test de dépôt T9 et mutation M7a (R2) ; commande du banc et pire document
  défini (R3) ; témoin vectoriel nommé, branche d'arrêt, « T4 fait foi » (R5) ; `refs #854` jusqu'au constat du Project
  Lead, recette écrite (R14, § *Fermeture de #854*) ; `README.md` et inventaire de la valeur « Sans QR-code » (F1, F11) ;
  coût au manuel d'administration (F2) ; #855, #856, #858 citées, anciennes Q1 et Q3 retirées, Q2 et Q4 devenues Q1 et Q2 (F3, F4, R12) ; chapeau du
  CHANGELOG laissé à la PR de release (F5). LOW appliqués : R6 (grandeur causale, diagnostic `#[ignore]`), R7 (M11,
  cas `None`, arrondi), R8 (`QrColumns`, quinze colonnes, M13), R9, R10 (angle mort 1 bit écrit), R11, R13, R15, F6 à
  F9, F12 à F16. Registre : C-v017-5-2, -3, -5, -6, -8 à -11 amendés, C-v017-5-12 et -13 ajoutés. Prochaine étape :
  validation P2 (`v017-5-validate-prompt-p2.md`, Opus, R et F, passe complète).
- 2026-10-31 — **Validation P2** (Opus, lentilles R et F, contexte frais) : 0 CRITICAL, 1 HIGH (F1), 10 MEDIUM (R1 à R8,
  F2, F3), 12 LOW (R9 à R15, F4 à F8). Remédiation (Opus 5.5), prémisses lues au code et dans les dépendances
  (`api-client.ts:42`, `:53`, `:59-64`, `:589-600` ; `imported-supplier-invoices.api.ts:27-29` ; `inbox_import.rs:172-207` ;
  `config.rs:1011-1030` ; `i18n-repli-divergent-actif.test.ts:222` ; `rxing-0.7.1/src/qrcode/qr_code_writer.rs:80` ;
  `render_config.rs:825-826`), avec les arbitrages de l'orchestrateur : **délai du client dans la story** —
  `INBOX_IMPORT_TIMEOUT_MS = 300_000` aligné sur le proxy, test T11, mesure contre 30 s et 300 s, conduite à l'abandon
  et verrou de run vérifiés par exécution (AC-6-bis, T12 ; issue P2 demandée si le verrou fuit) (F1, R8) ; **deux passes
  par document** (R2) ; vocabulaire « QR-facture » partout, libellé **« Aucune QR-facture lue »** retenu et soumis en Q1
  (F3) ; branche (i) récrite avec paliers de dégradation écrits d'avance (R1) ; erreur du second rendu sans effet sur le
  diagnostic (R3) ; boucle `decode_pages` extraite avec rendeur injecté, T10 en CI (R4) ; codeur `render_qr_image`
  niveau M, version 12 à 15 (R5) ; quinze colonnes non `NULL` dans T6/T9, M13a (R6) ; pire document D1 à D4 (R7) ;
  garde G13 et M10/M10a (F2). LOW appliqués : R9 (`include_bytes!`), R10, R11 (sonde), R12 (octets JPEG), R13, R14,
  R15 (M3a), F4, F6, F7, F8 ; F5 : texte correctif des commentaires de #854 et #858 remis à l'orchestrateur. Registre :
  C-v017-5-2, -3, -4, -5, -7 amendés, C-v017-5-14 et -15 ajoutés. Prochaine étape : validation P3
  (`v017-5-validate-prompt-p3.md`, Sonnet, R et F, passe complète, axe obligatoire sur cette remédiation).
- 2026-10-31 — **Validation P3** (Sonnet, lentilles R et F, contexte frais) : 0 CRITICAL, 0 HIGH, 2 MEDIUM (F1, F2),
  22 LOW (R1 à R9, F3 à F15). Remédiation (Opus 5.5), prémisses lues au code (`+page.svelte:96-108`, `:224-226` ;
  `fr-CH/messages.ftl:2251`, `:2509` ; `admin-manual.tex:638-661`, `:822` ; `inbox_import.rs:180` ;
  `i18n-repli-divergent-actif.test.ts:210` ; `entities/mod.rs:64-65` ; `printpdf-0.7.0/src/pdf_document.rs:793`), avec
  les arbitrages de l'orchestrateur : libellé **« Aucune QR-facture exploitable »** (de, it, en proposés), Q1 récrite
  cas par cas et limite assumée récrite (F1) ; phrase sur le proxy DSM, délai de cinq minutes assorti de sa réserve
  (F2) ; `IS_FREE_LOCK` à AC-6-bis et exception du fichier en cours (R1) ; point d'accroche
  `decode_spc_from_pdf_bytes_observed` (R2) ; M12e, M13b, T9 à données différentes et `supplier_invoice_id` résiduel,
  cas de T10 au commit rouge écrits exactement (R3) ; phrase du diagnostic récrite (R6). LOW appliqués : R4 (palier 3
  binarisé avant JPEG), R5 (fixtures lues à l'exécution, plus d'`include_bytes!`), R7 (commandes d'installation de
  pdfium), R8, R9, F3 à F10, F12 à F15 ; F11 : texte de l'issue P4 (pages de plus de 2000 pt) remis à l'orchestrateur.
  À l'API, `none` reste « aucune QR-facture lue » (AC-12). Registre : C-v017-5-2, -5, -7, -10 amendés. Prochaine étape :
  validation P4 ciblée (`v017-5-validate-prompt-p4.md`, Opus, sur le diff de cette remédiation).
- 2026-10-31 — **Validation P4** (Opus, passe ciblée sur la remédiation P3) : 0 CRITICAL, 0 HIGH, 3 MEDIUM (P1 à P3),
  6 LOW (P4 à P9). Remédiation (Opus 5.5), prémisses lues au code (`api-client.ts:243-247`, `:316-321` ;
  `fr-CH/messages.ftl:2250-2252` ; `+page.svelte:103-106` ; `qr_decode.rs:93`, `:112` ; `inbox_import.rs:564` ;
  `admin-manual.tex:1492`) : le libellé de l'E2E, coupé sur deux lignes, corrigé après recherche sur la fiche aplatie
  (P1) ; AC-10 ne cite plus que les trois messages que l'écran affiche, cas par cas (P2) ; cinquième cas, « QR-facture
  présente mais non détectée », ajouté à AC-7, à la limite assumée, à Q1 et à C-v017-5-7, et `none` redéfini à l'API
  (P3) ; registre C-v017-5-5 (amendement P3), C-v017-5-7 (titre), C-v017-5-10 (« six modules ») ; `admin-manual.tex:1492`
  à l'inventaire (P7) ; compteur du second temps nuancé (P8) ; dépassement à 200 dpi → issue demandée, D1 à 300 dpi en
  mémoire (P9). Prochaine étape : validation P5 ciblée (`v017-5-validate-prompt-p5.md`, Sonnet).
- 2026-10-31 — **Développement** (Opus 5.5, autonomie). Étape 0 : les cinq LOW de la validation P5 appliqués (lignes
  lues au Dev Agent Record ; les citations `:93` et `:103-106` de l'entrée « Validation P4 » ci-dessus se lisent `:92`
  et `:106-109`). Commit rouge `67727ad1` (fixtures au palier 2 des branches d'arrêt, la dégradation d'AC-1 étant lue à
  72 dpi), correctif `4c823ed3`, relecture et garde du verrou de run `3531cc00` (AC-6-bis : fuite constatée, corrigée
  par `close_on_drop`), textes et manuels `9c26f50d`. Gates : backend 4610/4610, Vitest 2293, E2E 330 + les 7 de la
  KF-029 ; 23 mutations tuées, M4 déclarée. Registre C-v017-5-16 à 18. Prochaine étape : revue de code P1
  (`v017-5-review-prompt-p1.md`, Opus, B, E et A).
- 2026-10-31 — **Revue de code P1** (Opus, lentilles B, E, A, contexte frais) : 0 CRITICAL, 0 HIGH, 1 MEDIUM (A1),
  14 LOW (B-1 à B-5, E-1 à E-4, A2 à A6). Remédiation (Opus 5.5), prémisses relues au code et dans les dépendances
  (`render_config.rs:763-822` : E-3 à moitié vraie, C-v017-5-19 ; `sqlx-core-0.8.6/src/pool/options.rs:161-162` :
  E-1) : générateur à contrôle préalable des tailles et annotations datées des paliers (A1, sans réécrire les
  entrées) ; réponse à la question de B en annotation de C-v017-5-17 ; second rendu omis quand il redonnerait le
  premier (E-3, M15) ; T10 (h), (i) (B-2, M16) ; T3-bis sur les fixtures versionnées sans pdfium (B-3) ;
  `restants > 0` dans T12 (B-1 = E-4) ; commentaires E-1, E-2, B-5 ; textes au fait (B-4 = A5), trace `reactivated` à
  l'API (A3), #863 au CHANGELOG (A4), en-tête (A6) ; T4 et banc rejoués au dernier commit de code (A2). Gates :
  backend 4617/4617, Vitest 2293, E2E 330 + les 7 de la KF-029. Commits `dee8f982` (code), `25b2367a` (textes, PDF).
  Prochaine étape : revue de code P2 ciblée (`v017-5-review-prompt-p2.md`, Sonnet, une lentille).

- **2026-10-31 — Revue de code P2 (Sonnet, ciblée)** : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW. Elle confirme au code (pdfium-render `apply_to_page`) l écart C-v017-5-19 à la consigne P1. Boucle close en deux passes (P1 Opus 0/0/1/14 → P2 Sonnet 0/0/0/4). LOW de la P2 tracés en #864. Statut : `done`. #854 reste ouverte jusqu à la recette du Project Lead sur ses pièces.
