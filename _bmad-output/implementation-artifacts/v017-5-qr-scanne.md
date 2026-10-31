# Story v017-5 : lire le QR d'une facture QR scannée

Status: ready-for-dev

*Fiche rédigée le 2026-10-31, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0176`, branche
`story/v017-5-qr-scanne` partie de `8e126cfd` (tête de `release/0.17.0`). Release **0.17.0**, candidate suivante
(décision de Guy du 2026-10-31). Registre : `v017-5-choix-autonomes.md` (C-v017-5-1 à C-v017-5-13). Clé courte :
`v0175`. Issue : **#854** (bug, P2), en `refs` seulement — dans les commits **et dans la PR de release** : #854 ne se
ferme qu'au constat du Project Lead sur son installation (§ *Fermeture de #854*). Les numéros de ligne sont ceux de `8e126cfd` ; ceux des dépendances, ceux de `~/.cargo/registry`
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

### La cause, prouvée avant le correctif

**AC-1 — Fixtures synthétiques.** Un générateur de test (C-v017-5-5) produit, à graine fixe :
(a) trois PDF d'une page A4 portant chacun une image JPEG en niveaux de gris d'une QR-facture scannée à **150, 200 et
300 dpi**, flou gaussien de 0,1 mm, bruit uniforme ± 12 niveaux, papier à 240 et encre à 30 (sur 255) ; (b) un PDF
d'une **page scannée pleine, sans QR** (300 dpi, lignes de « texte » figurées par des blocs sombres de largeurs tirées au
sort sur toute la hauteur de la page, même flou, même bruit) — c'est aussi la page du pire document du banc (AC-6) ;
(c) les mêmes images, en mémoire, pour les tests d'hôte. Le payload des scans, construit par
`kesh_qrbill::build_payload`, fait **au moins 280 caractères** et donne un QR de **version ≥ 12** (assertions dans le
générateur). Les quatre PDF sont **produits une fois et versionnés comme données** sous `crates/kesh-api/tests/fixtures/`
(`qr-scan-150dpi.pdf`, `qr-scan-200dpi.pdf`, `qr-scan-300dpi.pdf`, `scan-sans-qr.pdf`) ; le générateur est versionné et
documenté (paramètres, commande qui l'exécute), **sans exigence de reproductibilité au bit près** — `printpdf` tire un
identifiant d'instance et date le document à chaque écriture (`printpdf-0.7.0/src/pdf_document.rs:791`,
`pdf_metadata.rs:58`). La preuve porte sur le **décodage** (lu, non lu), non sur les octets. **Plafond : 1 Mio par
fixture** ; pour le tenir, le développement peut baisser la qualité JPEG (jamais sous 60) ou restreindre le bruit à la
section paiement, et écrit la valeur retenue au Dev Agent Record avec la taille de chaque fichier. Aucune donnée
réelle : IBAN d'exemple SIX (`CH9300762011623852957`, déjà employé par `qr_decode.rs:211`, ou le QR-IBAN d'exemple SIX
pour la référence QRR), noms et adresses fictifs.

**AC-2 — La reproduction, rouge d'abord.** Au **premier commit** de la story (fixtures, générateur, tests, banc, et la
seule extraction de `render_scale`, que **rien n'appelle encore**), sans aucun correctif :
- **T4** (pdfium, `#[ignore]`, joué en local, C-v017-5-6) : `decode_spc_from_pdf_bytes` rend `Ok(None)` pour les trois
  `qr-scan-*.pdf` ; `Ok(None)` pour `scan-sans-qr.pdf` ; `Ok(Some(_))` pour le **témoin vectoriel** — un PDF engendré à
  l'exécution par `kesh_qrbill::generate_qr_bill_pdf_with_date` (QR dessiné en rectangles), dont le payload est **celui
  de `render_spc_qr_png`** (`qr_decode.rs:210-227` : créancier « Robert Schneider SA », adresse combinée, 100.00 CHF,
  sans référence, message « Facture test » — un petit payload, QR de version basse), afin que le témoin se lise à
  72 dpi ;
- **T1** (hôte) : les tests de `render_scale` (AC-3) sont **rouges** (la fonction rend encore `Some(1.0)`) ;
- **Branches d'arrêt** — le développement s'**arrête**, consigne la sortie et le signale à l'orchestrateur, sans
  correctif à l'aveugle : (i) T4 **lit** une fixture `qr-scan-*` avec le code d'avant — H1 est réfutée ; (ii) le
  **témoin vectoriel n'est pas lu** au commit rouge — le chemin pdfium ne fonctionne pas sur la station (bibliothèque,
  liaison) et T4 ne prouve rien ; (iii) après AC-3 et AC-4, une fixture du périmètre reste illisible — la découpe pdfium
  d'une bande à plus haute résolution (C-v017-5-2 (d)) est alors une décision de l'orchestrateur ;
- la sortie de T4 au commit rouge est **copiée** au Dev Agent Record (commande, version de `libpdfium`, résultat par
  fixture).

### Le remède

**AC-3 — Rendu en deux temps (C-v017-5-2).** Pour chaque page, dans l'ordre :
1. **premier rendu**, inchangé : `PdfRenderConfig` à l'échelle 1,0 (72 dpi) bornée par `max_dimension`, exactement le
   rendu d'avant — il lit les QR vectoriels à peu de frais ;
2. **seulement si ce premier rendu ne livre aucun payload SPC**, un **second rendu** de la même page à l'échelle
   `render_scale(...)`, par `scale_page_by_factor`, bornes `set_maximum_width/height(cfg.max_dimension)` conservées.

Un payload SPC **inexploitable** au premier rendu (`InvalidSpcPayload`, `InvalidIban`) est rendu tel quel, sans second
rendu (le QR a été lu). Une erreur de rendu du premier temps compte comme aujourd'hui (`last_render_err`) et le second
temps n'est pas tenté pour cette page. `qr_decode.rs` expose une fonction pure documentée `render_scale(width_pt: f32,
height_pt: f32, max_dimension: u16) -> Option<f32>` = `Some(min(max_dimension / width_pt, max_dimension / height_pt,
MAX_RENDER_DPI / 72))` avec `const MAX_RENDER_DPI: f32 = 300.0`, et `None` si une dimension est nulle, négative ou non
finie, ou si `max_dimension == 0` (pas de second rendu). Tests d'hôte (T1), échelles à 10⁻³ près, et dimensions de
sortie comparées **après arrondi** (`(dim × échelle).round()`, comme `apply_to_page`, `render_config.rs:812-820`) :
- A4 portrait (595,276 × 841,89 pt), borne 2000 → 2000 / 841,89 ≈ **2,3756** (≈ 171 dpi) ; sortie 1414 × 2000 ;
- A4 paysage → même échelle (borne par la largeur) ;
- section paiement seule, 210 × 105 mm (595,28 × 297,64 pt) → 2000 / 595,28 ≈ **3,3598** (≈ 242 dpi, la borne de
  largeur l'emporte) ;
- A6 portrait (297,64 × 419,53 pt) → **300 / 72** ≈ 4,1667 (le plafond dpi l'emporte sur 2000 / 419,53 ≈ 4,767) ;
- A3 portrait (841,89 × 1190,55 pt) → 2000 / 1190,55 ≈ 1,6799 ; sortie ≤ 2000 px ;
- page minuscule (10 × 10 pt) → 300 / 72 ;
- page de largeur 0, de hauteur `NaN`, et `max_dimension = 0` → `None`.
`DecodeConfig::default()` est **inchangé** (20 pages, 2000 px). Aucun rendu ne dépasse 2000 px de côté : la borne n'est
pas franchie.

**AC-4 — Décodeur.** `decode_qr_from_image` passe par `detect_multiple_in_luma_with_hints` avec `PossibleFormats =
{QR_CODE}` et `TryHarder = true` (C-v017-5-3), aux deux temps. Test d'hôte **T2** : chacune des trois images
synthétiques (150, 200, 300 dpi), rééchantillonnée (`image::imageops::resize`, filtre `Triangle`) aux dimensions du
second rendu d'une A4 (1414 × 2000 px), est **lue** — IBAN, montant et référence égaux au payload. **Grandeur causale**
(test d'hôte, pur) : pour le QR des fixtures (version et nombre de modules lus du générateur), les pixels par module au
premier rendu (46 mm à 72 dpi) sont **inférieurs à 2,5**, et au second rendu (≈ 171 dpi) **supérieurs à 4** — une
assertion sur la grandeur, non sur une faiblesse du décodeur. Le contrôle « les mêmes images à 595 × 842 px ne sont
pas lues » est un **diagnostic** `#[ignore]`, joué à la main avec T4, dont le résultat va au Dev Agent Record (à la
montée de `rxing`, il peut se retourner sans que rien ne soit faux). **T3** (contrôle) : les mêmes images à leur
résolution native, par `decode_spc_from_image_bytes` (JPEG), sont lues — le décodeur n'est pas en cause sur le chemin
image. **Échelon supplémentaire** (`FilteredImageReader`, puis découpe de la bande inférieure) : écrit **seulement** si
une image de T2 n'est pas lue sans lui et l'est avec lui, et alors couvert par une mutation (M5) ; sinon non écrit, et le
Dev Agent Record le dit. En cas de contradiction entre T2 et T4, **T4 fait foi**.

**AC-5 — Le correctif, prouvé par pdfium.** Au dernier commit de code, T4 (même commande qu'à AC-2) rend `Ok(Some(_))`
pour les trois `qr-scan-*.pdf` (IBAN, montant, référence égaux au payload) et pour le témoin vectoriel — lu, lui, **dès
le premier rendu** (le test le vérifie par un compteur de rendus exposé aux tests, ou par une fonction de décodage d'une
page qui rend le temps qui a lu) —, `Ok(None)` pour `scan-sans-qr.pdf`. Sortie copiée au Dev Agent Record. Le test
cassé `decode_spc_from_pdf_single_page` (`qr_decode.rs:301-313`) lit désormais une fixture qui existe
(`qr-scan-300dpi.pdf`) — correction mécanique (C-v017-5-6).

**AC-6 — Coût, mesuré et plafonné.** Un banc `#[ignore]` (`qr_decode::tests::banc_cout`) mesure, **avec le même banc et
les mêmes fixtures**, au commit rouge (premier rendu seul) **et** au dernier commit de code : (a) le temps de rendu +
décodage **par page** de chaque fixture, médiane de 5 passages ; (b) le **pire document** : 20 pages, chacune la page de
`scan-sans-qr.pdf` (engendré par le banc en mémoire, non versionné) — chaque page subit les deux rendus et deux
décodages sans rien trouver ; (c) la mémoire résidente de pointe du **binaire de test** (non de `cargo`). Commande,
sous le verrou de gate et `scripts/mem-guard.sh` :

```sh
cd /home/gcorbaz/devel/kesh-0176 && \
  CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-0176/target \
  cargo test --release -p kesh-api --lib --no-run --message-format=json \
  | jq -r 'select(.profile.test == true and .target.name == "kesh_api") | .executable'
# puis, avec le chemin rendu :
LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib \
  /usr/bin/time -v <binaire> --ignored --exact qr_decode::tests::banc_cout --nocapture
```

(« Maximum resident set size » du binaire). Le profil `--release` recompile `kesh-api` et ses dépendances : c'est le
seul profil dont le coût est représentatif de l'image Docker. **Plafonds**, sur la station : médiane **≤ 1,0 s par
page** pour une page scannée sans QR (les deux temps), pire document **≤ 20 s**, pointe de mémoire **≤ 128 Mio de plus**
qu'au commit rouge ; un QR vectoriel, lu au premier temps, coûte ce qu'il coûtait. Par construction, chaque bitmap rendu
reste sous **2000 × 2000 px × 4 octets ≈ 15,3 Mio** (borne inchangée). **Coût agrégé** : un import traite jusqu'à 200
fichiers par passage (`config.rs:524`) dans une seule requête HTTP, synchrone (#855), derrière un proxy documenté à
300 s (`admin-manual.tex:341`) ; le Dev Agent Record écrit, depuis le banc, le temps d'un passage de 200 fichiers de deux
pages scannées sans QR sur la station, et une projection pour un NAS (facteur déclaré, non mesuré). Les chiffres avant /
après vont au Dev Agent Record et l'ordre de grandeur au manuel d'administration (AC-11). Un plafond dépassé se corrige
en réduisant le travail (échelon supplémentaire limité à la première et à la dernière page, ou retiré) — **jamais** en
relâchant une borne (C-v017-5-4).

### Ce que l'écran dit

**AC-7 — « Aucun QR lu ».** Les valeurs (clés inchangées, C-v017-5-7) :

| clé | fr-CH | de-CH | it-CH | en-CH |
|---|---|---|---|---|
| `imported-supplier-invoices-badge-no-qr` | Aucun QR lu | Kein QR-Code gelesen | Nessun codice QR letto | No QR code read |
| `imported-supplier-invoices-report-no-qr` | aucun QR lu | kein QR-Code gelesen | nessun codice QR letto | no QR code read |

Les replis de `+page.svelte:237`, `:312` et `AttachDocumentPicker.svelte:148` égalent la valeur fr-CH (garde
`i18n-repli-divergent-actif.test.ts`) ; le commentaire d'en-tête de `+page.svelte:12` suit. `README.md:44` nomme le
badge : *QR-facture* ou *Aucun QR lu*. La valeur d'API `qr: "none"` ne change pas. **Inventaire de la valeur**
(`grep -rnF "Sans QR-code"` et `grep -rnF "sans QR-code"` sur tout le dépôt hors `_bmad-output/` et `CHANGELOG.md`,
relevé le 2026-10-31) — chaque occurrence a sa décision :

| Lieu | Décision |
|---|---|
| `fr-CH/messages.ftl:2538`, `:2552` ; replis `+page.svelte:237`, `:312`, `AttachDocumentPicker.svelte:148` ; commentaire `+page.svelte:12` ; `README.md:44` ; `user-manual.tex:1851`, `:1857` ; tests `import-page.test.ts:112`, `:127`, `:162`, `:185`, `supplier-documents.spec.ts:94`, `:104`, `:107` | **libellé d'écran : changé** |
| `fr-CH/messages.ftl:2533` (commentaire 30-2a), `+page.svelte:10`, `user-manual.tex:807`, `:1609`, `admin-manual.tex:818`, `:2541`, `marketing-brochure.tex:396`, `:446`, `api-external.md:255`, `:774`, `README.md:232` | **emploi générique** (« un document avec ou sans QR-code ») : conservé |

Le grep se rejoue après le patch : aucune occurrence de la première ligne ne doit subsister. **Limite assumée** : un
document dont le QR a été lu puis refusé (`FIELD_TOO_LONG`, montant hors limites, IBAN invalide) est annoncé « QR
illisible » au rapport et « Aucun QR lu » dans la liste — la distinction n'est pas stockée (C-v017-5-7) ; le manuel dit
« aucun QR **exploitable** ».

### Relire une pièce déjà importée

**AC-8 — Relecture à la réactivation.** Quand un fichier déposé a l'empreinte d'une ligne `discarded` **sans données
QR** (`!existing.has_qr()`), Kesh le **décode** (même fonction que l'import, extraite pour les deux chemins — DRY) avant
la transaction de réactivation, puis :
- QR lu **et** exploitable (`check_scanned_fits_columns` vert) → `reactivate_to_complete_with_qr` (C-v017-5-9) : une
  seule instruction, garde `status = 'discarded' AND creditor_iban IS NULL`, qui écrit les **quinze** colonnes QR —
  `creditor_iban`, `is_qr_iban`, `creditor_address_type`, `creditor_name`, `creditor_line1`, `creditor_line2`,
  `creditor_postal_code`, `creditor_town`, `creditor_country`, `reference_type`, `reference_value`, `amount`,
  `currency`, `unstructured_message`, `billing_information` (`NULL` explicite pour une facultative absente) — depuis une
  projection **extraite** de `NewImportedSupplierInvoice::from_scanned` (`entities/imported_supplier_invoice.rs:260-296`),
  type `QrColumns` partagé par l'import et la réactivation (aucune seconde règle, aucun `DocumentMeta` fabriqué) ;
  rapport `qr: "present"` ;
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
`completed` (joint à une facture) redéposé aussi ; aucune ligne n'est relue au démarrage, à la migration ou à la liste.
Tests T7 et T9 (ci-dessous).

### Textes

**AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`, § *Documents à traiter*, l. 1838-1880), `.tex` **et**
PDF régénéré :
- l. 1851 : « avec la mention \emph{aucun QR lu} ou \emph{QR illisible} » ;
- l. 1857-1861, le badge : « \textbf{Aucun QR lu}~: Kesh n'a lu aucun QR exploitable sur le document --- il n'en porte
  pas, son QR n'a pas pu être lu (un scan de mauvaise qualité peut échapper à la lecture), ou il n'a pas pu être exploité
  (illisible, montant hors des limites admises, valeur trop longue). » — la suite (« Le document est alors accepté sans
  les données du QR… ») inchangée ;
- *Retirer de la liste* (l. 1874-1876) : « … le déposer de nouveau le fait revenir dans la liste~; s'il n'avait aucun QR
  lu, Kesh le relit alors. Pour le redéposer, \emph{Téléchargez}-le d'abord~: une fois retiré, il ne se télécharge plus
  depuis la liste. » ;
- un paragraphe **neuf**, *Faire relire le QR d'un document*, après la liste des quatre gestes, qui dit le **fait** et non
  l'effet espéré : depuis la 0.17.0, Kesh rend une page de PDF dont il ne lit aucun QR une seconde fois, à environ
  170 dpi pour une page A4 au lieu de 72, ce qui permet de lire le QR de bien des documents **scannés** ; un document
  entré plus tôt avec « Aucun QR lu » n'est **pas** relu de lui-même ; pour le faire relire : *Télécharger*, *Retirer de
  la liste*, déposer le fichier téléchargé dans le dossier d'import, *Importer le dossier* — le document revient, avec
  les données du QR s'il est lu, sans doublon ; le même geste vaut après que l'administrateur a relevé la borne de pages
  ou installé la bibliothèque PDF ; un document dont le QR a été lu, ou joint à une facture, n'est jamais relu ; si le QR
  reste illisible, l'IBAN, la référence et le montant se saisissent à la main.
- Les emplois génériques (l. 807, 1609) restent (AC-7, inventaire).

**AC-11 — Manuel d'administration** (`admin-manual.tex`), `.tex` et PDF :
- § *Passer à la 0.17.0* (l. 2481-2491) : les documents importés avant la mise à jour ne sont pas relus ; renvoi au
  manuel utilisateur (*Faire relire le QR d'un document*) ;
- § *Intégrations par l'API (0.17.0)* : la réactivation d'un document sans QR le relit, et son rapport dit `present`,
  `none` ou `unreadable` (avec `qrError`) selon cette lecture ;
- § *Réglage `KESH_INBOX_MAX_PDF_PAGES`* (l. 920-923) : chaque page est rendue d'abord à 72 dpi, puis, si aucun QR n'y
  est lu, une seconde fois à la plus haute résolution que permet la borne de 2000 pixels par côté, au plus 300 dpi
  (environ 170 dpi pour une A4) ; un document sans QR lisible coûte donc deux rendus par page ; l'ordre de grandeur
  mesuré à AC-6 (temps par page, sur une station, et le fait qu'un NAS est plus lent) ; l'import d'un dossier se fait en
  une seule requête, synchrone, derrière le délai du proxy (300 s dans la configuration documentée) : déposer les
  documents scannés par petits lots ; relever la borne de pages augmente ce coût d'autant.
- Les emplois génériques (l. 818, 2541) restent.

**AC-12 — `docs/api-external.md`** l. 778 : `none` devient « aucun QR lu » ; la dernière phrase (« Un document
**réactivé** … porte `present` ou `none` selon ce qui a été stocké ») est remplacée par : « *Depuis la v0.17.0*, un
document **réactivé** — retiré de la liste puis redéposé — est relu s'il n'avait aucune donnée QR : `present`, `none` ou
`unreadable` (avec `qrError`) selon cette lecture ; sinon `present`. »

**AC-13 — CHANGELOG**, `[0.17.0] — Non publié`, *Corrigé* : « **Le QR d'une facture QR scannée peut être lu à
l'import** ([#854](https://github.com/guycorbaz/kesh/issues/854)) : Kesh rendait chaque page d'un PDF à 72 dpi, trop
peu pour le QR d'un document scanné ; une page où il ne lit aucun QR est désormais rendue une seconde fois, à environ
170 dpi pour une page A4 (au plus 300 dpi, sous la même borne de 2000 pixels par côté). L'écran dit « Aucun QR lu »
plutôt que « Sans QR-code ». Un document importé auparavant n'est pas relu de lui-même : *téléchargé*, retiré de la
liste puis redéposé, il l'est (manuel utilisateur, *Faire relire le QR d'un document*). » Le chapeau de la section n'est
pas touché : la PR de release le récrit. Dans un commit comme dans la PR : `refs #854` seulement.

## Fermeture de #854 — recette du Project Lead

La PR de release porte `refs #854`, **jamais `closes`**. #854 se ferme quand le Project Lead a constaté, **sur son
installation, après la mise à jour** vers la candidate qui porte cette story, que ses deux factures communales sont
lues. Sa recette, qu'il joue lui-même sur ses livres (aucun geste de la release sur sa base) : pour chacune, *Télécharger*,
*Retirer de la liste*, déposer le fichier dans le dossier d'import, *Importer le dossier* ; il rapporte seulement
« lues » ou « non lues », **sans aucune donnée** (ni IBAN, ni montant, ni référence, ni capture). « Lues » : il ferme
#854 (ou le demande). « Non lues » : #854 reste ouverte, et la suite (branche (iii) d'AC-2 : découpe à plus haute
résolution, échelon du décodeur) se décide sur fixtures synthétiques, jamais sur ses pièces.

Recette de l'orchestrateur, avant : sur une installation **jetable** de l'image Docker de la branche, déposer les quatre
fixtures et le témoin vectoriel ; attendus : trois scans et le témoin lus, `scan-sans-qr.pdf` « Aucun QR lu » ; puis un
scan importé, ses colonnes QR vidées en SQL sur cette base jetable, retiré, redéposé : il revient « QR-facture ».

## Tâches

1. **Commit rouge** (AC-1, AC-2) : générateur et fixtures ; `render_scale` extraite, non appelée, rendant `Some(1.0)` ;
   T1 (rouge), test de grandeur causale, T2, T3, diagnostic, T4 (`#[ignore]`), banc AC-6 ; `printpdf = "0.7"` en
   `dev-dependencies` de `kesh-api` — le `Cargo.lock` ne gagne **qu'une ligne** (`"printpdf",` dans les dépendances de
   `kesh-api`), aucun paquet neuf (le vérifier par `git diff Cargo.lock`) ; T4, le diagnostic et le banc joués en local,
   sorties copiées.
2. **Correctif** (AC-3, AC-4, AC-5, AC-6) : second rendu à `render_scale`, `PossibleFormats`, échelon supplémentaire
   seulement sur preuve ; T4 et banc rejoués.
3. **Relecture** (AC-8, AC-9) : extraction du décodage (une fonction pour l'import et la réactivation), extraction de
   `QrColumns`, `reactivate_to_complete_with_qr` (`kesh-db`), trace, tests T6, T7, T9.
4. **Textes** (AC-7, AC-10 à AC-13) : quatre locales, replis, `README.md`, Vitest et E2E mis à jour, manuels (`make fr`
   dans `docs/manual/`, sous verrou de gate), `api-external.md`, CHANGELOG ; grep de la valeur rejoué.
5. **Captures** et gates.

## Tests et mutations prévus

| Test | Lieu | Ce qu'il prouve |
|---|---|---|
| T1 | `qr_decode.rs` (`mod tests`) | AC-3 : les neuf cas de `render_scale` (six pages, trois `None`) ; rouge au commit rouge |
| T1-bis | `qr_decode.rs` | AC-4 : grandeur causale, < 2,5 px par module au premier rendu, > 4 au second |
| T2 | `qr_decode.rs` | AC-4 : lu à 1414 × 2000 |
| T2-diag | `qr_decode.rs`, `#[ignore]` | AC-4 : non lu à 595 × 842 (diagnostic, résultat déclaré) |
| T3 | `qr_decode.rs` | AC-4 : chemin image, résolution native, lu |
| T4 | `qr_decode.rs`, `#[ignore]` | AC-2 / AC-5 : PDF par pdfium, avant (non lus) / après (lus), témoin vectoriel lu au premier rendu, sans-QR |
| T5 | `qr_decode.rs`, `#[ignore]` | AC-6 : banc de coût |
| T6 | `tests/inbox_import_e2e.rs` | AC-8 : un PNG à QR importé, ses **quinze** colonnes QR remises à `NULL` en SQL et son statut à `discarded` ; le même PNG redéposé → `present`, les quinze colonnes égales à celles d'un import neuf du même PNG, trace `reactivated` avec `"qr": "present"`, `"qr_reread": true` ; variante QR inexploitable (`qr_invoice_png_with`, valeur trop longue) → `unreadable` / `FIELD_TOO_LONG`, colonnes `NULL` ; `import_no_qr_duplicate_and_reactivation` (`:851-883`) reste vert (`none`, `"qr_reread": true`) |
| T7 | `tests/inbox_import_e2e.rs` | AC-8 / AC-9 : ligne `discarded` avec QR dont `creditor_name` est changé en SQL (`'Ancien nom'`) → redéposée, `present`, `creditor_name` **inchangé**, `"qr_reread": false` ; ligne `to_complete` sans QR redéposée → `DUPLICATE`, colonnes inchangées |
| T8 | `import-page.test.ts`, `supplier-documents.spec.ts` (E2-a) | AC-7 : « Aucun QR lu » / « aucun QR lu » |
| T9 | `kesh-db`, `#[sqlx::test]` (`imported_supplier_invoices`) | AC-8 / C-v017-5-9 : `reactivate_to_complete_with_qr` sur une ligne `discarded` **pourvue** d'un QR rend `false` et laisse ses quinze colonnes inchangées ; sur une ligne `discarded` sans QR, rend `true` et écrit les quinze ; sur une ligne `to_complete`, rend `false` |

T6, T7 et T9 passent par le chemin **image** ou le dépôt (pas de pdfium) : ils tournent en CI. Le temps de T2 et T3 en
profil de test (`image` et `rxing` sans optimisation, `Cargo.toml` `[profile.dev]`) est mesuré et déclaré au Dev Agent
Record ; s'il dépasse 30 s sur la station, T2/T3 n'engendrent en CI que les scans à 150 et 200 dpi, celui à 300 dpi
passant en `#[ignore]`.

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | `render_scale` rend `Some(1.0)` (l'ancienne valeur) | T1 ; T4 (local) |
| M2 | plafond `MAX_RENDER_DPI` retiré du `min` | T1 (A6 portrait, page minuscule) |
| M3 | borne `max_dimension` retirée du `min` | T1 (A4, A3 : sortie arrondie ≤ 2000 px) |
| M4 | `PossibleFormats` retiré | **aucun test fonctionnel** — coût seulement ; déclarée avec le chiffre du banc |
| M5 | chaque échelon supplémentaire retiré (s'il existe, AC-4) | le cas de T2 qui l'a justifié |
| M6 | réactivation sans relecture (code d'avant) | T6 |
| M7 | garde `creditor_iban IS NULL` retirée **et** décodage forcé pour une ligne pourvue | T7 (`creditor_name` réécrit) |
| M7a | garde `creditor_iban IS NULL` retirée **seule** | T9 (ligne pourvue réécrite) |
| M8 | rapport de réactivation = statut stocké au lieu du résultat de la relecture | T6 |
| M9 | trace sans `"qr_reread"` | T6 |
| M10 | valeur fr-CH du badge rendue à « Sans QR-code » | T8 (Vitest) et garde `i18n-repli-divergent-actif.test.ts` |
| M11 | second rendu retiré (le premier seul, code d'avant) | T4 (local) — **non tuée en CI**, déclarée |
| M12 | premier rendu retiré (second seul) | T4 (témoin vectoriel lu au premier rendu) — local ; coût au banc |
| M13 | colonnes facultatives non écrites par `reactivate_to_complete_with_qr` | T6, T9 |

Mutations sous verrou de gate, motif vérifié unique, restauration par `git checkout -- <fichier>` **puis `touch`**
(`docs/agents-orchestration.md` § 7).

## E2E

Aucune spec neuve : la relecture se prouve en Rust (T6, T7, T9) — un même fichier ne peut pas être illisible puis lisible
pour une même version, et l'E2E n'a pas pdfium. `supplier-documents.spec.ts` (E2-a, l. 94-107) suit le texte : titre et
assertion « aucun QR lu ». E2E complet au **dernier commit de code** (`docs/testing.md`, 7 à 10 échecs attendus jugés
fichier par fichier).

## Captures (thème clair)

Sur une base de démonstration, dossier `kesh-gate-logs/v0175/` : (1) la liste *Documents à traiter* avec une ligne
« Aucun QR lu » et une ligne « QR-facture » ; (2) le rapport d'import portant « aucun QR lu » ; (3) le sélecteur de
document à joindre (`AttachDocumentPicker`) avec le badge ; (4) **avant / après relecture** : la même pièce synthétique
« Aucun QR lu », puis, après retrait et redépôt, « QR-facture », avec le rapport de réactivation (préparée comme la
recette de l'orchestrateur : colonnes QR vidées en SQL sur la base de démonstration) ; (5) en `de-CH`
(`KESH_LANG=de`) : le badge et le rapport. Jointes au Dev Agent Record et au prompt de revue de code.

## Gates

`kesh-db` est touché (une fonction de dépôt) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Test Locally
First*, exception `kesh-db`). Au dernier commit de code : `scripts/test-fast.sh` sous verrou, `npm run check`,
`npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`, E2E complet ; T4, le diagnostic et le banc joués en
local avec `libpdfium` épinglée (déclarés comme tels, jamais comme « CI »). Base remise à zéro avant tout gate complet,
selon `docs/agents-orchestration.md` § 4 (bases `kesh_v0175`, `kesh_e2e_v0175`).

## Ce que les gardes ne voient pas

- **La CI ne voit pas le chemin PDF** (#856) : T4, M11, M12 et le banc ne tournent qu'en local. Une régression du rendu
  ne rougirait en CI que par T1 et T1-bis.
- **Un scan réel n'est pas une fixture** : rotation, moiré, QR partiellement masqué, images en 1 bit (CCITT, JBIG2 —
  fréquentes dans les archives PDF/A, où le sous-échantillonnage de pdfium est le plus agressif) ne sont pas simulés.
  La vérification sur les deux pièces de #854 est celle du Project Lead (§ *Fermeture de #854*).
- **Le seuil de lecture n'est pas borné vers le bas** : un scan à 100 dpi ou très flou peut rester illisible ; le
  manuel le dit.
- **La durée d'un import** n'a pas de borne d'horloge (C-v017-5-4) : un passage de nombreux PDF scannés peut dépasser
  le délai d'un proxy (AC-6, AC-11, #855).
- **M4** n'a pas de test fonctionnel (coût seulement) ; **M11** et **M12** ne sont tuées qu'en local.

## Dev Notes

### Fichiers touchés (prévus) et modules

- `crates/kesh-api/src/qr_decode.rs` — `render_scale`, `MAX_RENDER_DPI`, rendu en deux temps, décodeur, générateur,
  tests T1-T5.
- `crates/kesh-api/src/inbox_import.rs` — décodage extrait, relecture à la réactivation, trace.
- `crates/kesh-db/src/entities/imported_supplier_invoice.rs` — `QrColumns` extrait de `from_scanned`.
- `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` — `reactivate_to_complete_with_qr`, test T9.
- `crates/kesh-api/Cargo.toml` — `printpdf` en `dev-dependencies` (et la ligne correspondante de `Cargo.lock`).
- `crates/kesh-api/tests/fixtures/` — quatre PDF synthétiques.
- `crates/kesh-api/tests/inbox_import_e2e.rs` — T6, T7.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — deux valeurs chacune.
- `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte`, `.../import-page.test.ts`,
  `frontend/src/lib/features/imported-supplier-invoices/AttachDocumentPicker.svelte`,
  `frontend/tests/e2e/supplier-documents.spec.ts`.
- `README.md` (l. 44), `docs/manual/fr/user-manual.tex` (+ `.pdf`), `docs/manual/fr/admin-manual.tex` (+ `.pdf`),
  `docs/api-external.md`, `CHANGELOG.md`.

Pas de découpage (C-v017-5-10).

### Le manuel (axe de revue)

Les phrases prescrites (AC-10, AC-11) doivent être vraies au code livré : « d'abord à 72 dpi, puis une seconde fois »,
« environ 170 dpi » (2000 / 11,69 in = 171), « au plus 300 dpi », « sans doublon » (même ligne réactivée), « n'est
jamais relu » pour un document lu ou joint à une facture, « une fois retiré, il ne se télécharge plus depuis la liste ».
Contrôler le **PDF** aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.

### Références

- `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs:282-290` (`scale_page_by_factor`), `:314-327`
  (maxima), `:716-835` (`apply_to_page`).
- `rxing-0.7.1/src/helpers.rs:232-249` (`detect_multiple_in_luma_with_hints`),
  `rxing-0.7.1/src/decode_hints.rs:264` (`PossibleFormats`), `:270` (`TryHarder`), `:322` (`AlsoInverted`),
  `rxing-0.7.1/src/filtered_image_reader.rs`.
- `printpdf-0.7.0/src/xobject.rs:136-159` (`ImageXObject`, filtre DCT sans la fonctionnalité `embedded_images`) ;
  `pdf_document.rs:791` (identifiant d'instance tiré à chaque écriture).
- Norme SIX (QR-facture) : QR de 46 × 46 mm, section paiement de 210 × 105 mm en bas d'une A4, récépissé de 62 mm.

### Issues liées

- **#855** (P3, dette) — le décodage QR occupe un fil de l'exécuteur tokio ; parent de #820 pour CAMT/CSV.
- **#856** (P3, test) — le chemin PDF du décodage QR n'est exercé par aucun test qui tourne.
- **#858** (demande d'évolution) — relire le QR d'un document à traiter depuis l'écran (bouton), que #854 propose et
  que la story ne livre pas (C-v017-5-8).
- #854 commentée le 2026-10-31 (prémisse corrigée, cause, preuve prévue, relecture par redépôt, bouton renvoyé).

## Questions au Project Lead

- **Q1** — « Aucun QR lu » vous convient-il, ou préférez-vous « QR non lu » (la même mention couvre aussi un QR lu mais
  inexploitable, que le rapport appelle « QR illisible ») ?
- **Q2** — Vos deux factures communales déjà importées : après la mise à jour, le redépôt (Télécharger, Retirer,
  déposer, Importer) est un geste de votre part sur vos livres, et vous rapportez seulement « lues » ou « non lues »
  (§ *Fermeture de #854*). Cela vous convient-il ?

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

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
