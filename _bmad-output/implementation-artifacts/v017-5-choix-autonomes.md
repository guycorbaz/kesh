# Release 0.17.0 — registre des choix autonomes de la Story v017-5 (lire le QR d'une facture scannée)

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au merge. Fiche :
`v017-5-qr-scanne.md` (#854). Numérotation `C-v017-5-k`. Issue en `refs` seulement dans les commits. Les numéros de
ligne cités sont ceux de `8e126cfd` (tête de `release/0.17.0`, point de départ de la branche `story/v017-5-qr-scanne`) ;
ceux des dépendances, ceux de `~/.cargo/registry` (`pdfium-render-0.8.37`, `rxing-0.7.1`, `printpdf-0.7.0`).

Rédigé le 2026-10-31 en autonomie (Opus 5.5), sous la contrainte du Project Lead du 2026-10-30
(`kesh-gate-logs/prompts/v017-donnees-a-garder.md` : les données de la 0.16.1-rc1 se gardent) et sans ouvrir aucune
pièce réelle. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Aucun choix n'est soumis au Project Lead depuis la validation P1 (le bouton est tracé en #858) ; les questions
restantes sont dans la fiche (§ *Questions au Project Lead*).

---

### C-v017-5-1 — La cause retenue : la page PDF est rendue à 72 dpi, non à la résolution que suggère la borne

- **Contexte** : #854 avance, « non vérifié », qu'un QR scanné échappe au décodage « à la résolution de rendu
  retenue ». Lu au code : `qr_decode.rs:163-165` construit `PdfRenderConfig::new().set_maximum_width(2000)
  .set_maximum_height(2000)` et **aucun facteur d'échelle ni taille cible**. Dans `pdfium-render-0.8.37`,
  `render_config.rs:771-773` pose alors une échelle de **1,0** (« Set default scale to 1.0 if neither dimension is
  specified ») — un pixel par point PDF, soit **72 dpi** —, et les bornes maximales ne font que **réduire**
  (`:799`, `:813`). Une page A4 (595 × 842 pt) est donc rendue en 595 × 842 px ; la borne de 2000 px n'est jamais
  atteinte. Le QR d'une QR-facture mesure 46 mm (SIX) : à 72 dpi, 130 px pour 57 à 77 modules (versions 10 à 15, selon
  la longueur du payload), soit **1,7 à 2,3 px par module**. Un QR vectoriel (facture née numérique) peut encore se lire
  ainsi ; un QR **scanné**, déjà flou et bruité, rééchantillonné à moins de 2,5 px par module, ne se lit plus. Cela rend
  compte des deux constats de #854 : facture d'assurance lue (vraisemblablement née numérique — **non vérifiable** sans
  ouvrir la pièce, ce que le mandat interdit), factures communales scannées non lues.
- **Retenu** : cause **H1** = la résolution de rendu (72 dpi de fait). Elle est **établie au code**, **non encore
  prouvée** : la preuve est le test T4 (PDF synthétiques rendus par pdfium, code d'avant : aucun QR lu ; après : lus),
  joué avant tout correctif. Si T4 lit les fixtures **avec le code d'avant**, H1 est réfutée et le développement
  **s'arrête** et le signale (fiche, AC-2).
- **Écartées** : (b) H2, « le décodeur manque d'options » (`TryHarder`, binariseur) — `rxing` reçoit déjà `TryHarder`
  (`rxing-0.7.1/src/helpers.rs:241`) et le binariseur hybride (`:244`) ; H2 n'est retenue qu'en complément, sur preuve
  (C-v017-5-3). (c) « La bibliothèque PDF manque » — elle rendrait `PDF_RENDER_ERROR` (`unreadable`), non `none`.
- **Réversible** : oui (diagnostic) ; la fiche prescrit ce qui le réfute.

### C-v017-5-2 — Le remède : un rendu en deux temps — 72 dpi, puis, si aucune page ne livre de QR-facture, la plus haute résolution que la borne existante permet, plafonnée à 300 dpi

*(Récrit à la validation P1, sur décision de l'orchestrateur — finding R4 : le choix d'origine, « un seul rendu par
page » à `render_scale`, multipliait par 5,6 le coût de **chaque** page, QR vectoriels compris, alors qu'un import traite
jusqu'à 200 fichiers par passage (`config.rs:524`) derrière un proxy documenté à 300 s (`admin-manual.tex:341`), sur un
NAS.)*

- **Contexte** : le mandat propose une résolution plus élevée, et un second essai plus fin seulement si le premier ne
  trouve rien, **sans relâcher** les bornes anti-DoS. La borne de dimension (2000 px, `qr_decode.rs:46-53`) permet, pour
  une A4 portrait, **171 dpi** (2000 / 11,69 in) — **4,0 à 5,4 px par module** pour 57 à 77 modules (309,8 px / 77 et
  / 57), assez pour un QR scanné de 150 à 300 dpi : au-delà de la résolution source, un rendu plus fin n'apporte aucune
  information. Le rendu actuel (72 dpi) lit les QR vectoriels (constat de #854 : la facture d'assurance est lue).
- **Retenu** *(remplacé, pour l'ordre des passes, par l'amendement de la validation P2 ci-dessous : deux passes par
  document)* : pour chaque page, (1) le **rendu actuel**, inchangé (échelle 1,0, bornes `max_dimension`) ; (2)
  **seulement si** ce rendu ne livre aucun payload SPC, un second rendu à `render_scale(largeur_pt, hauteur_pt,
  max_dimension) -> Option<f32>` = `min(max_dimension / largeur, max_dimension / hauteur, MAX_RENDER_DPI / 72)`,
  `MAX_RENDER_DPI = 300`, appliqué par `scale_page_by_factor`, bornes `set_maximum_width/height` conservées ; `None`
  (pas de second rendu) pour une page de dimension nulle, négative ou non finie, ou `max_dimension == 0`. Les deux
  rendus restent sous 2000 px de côté : la borne n'est pas franchie. Le surcoût ne frappe que les pages sans QR lisible
  au premier temps.
- **Écartées** : (b) un seul rendu par page à `render_scale` (choix d'origine) — surcoût sur toutes les pages, QR
  vectoriels compris ; (c) un second rendu « plus fin » de la page entière au-delà de 171 dpi — franchirait la borne ;
  (d) un second rendu de la seule bande inférieure (section paiement) à 240 dpi par matrice de transformation et découpe
  pdfium — faisable dans la borne, mais intestable sur l'hôte et en CI (pas de `libpdfium`, C-v017-5-6), et sans gain
  attendu sur une source de 150 à 300 dpi rendue déjà à 171 dpi ; il reste ouvert **seulement** si T4 l'exige (fiche,
  AC-2, branche d'arrêt (iii)) ; (e) relever `max_dimension` — interdit par le mandat.
- **Réversible** : oui (une constante, une fonction, un second appel de rendu).
- **Amendé à la validation P2** (R2, R3, arbitrage de l'orchestrateur) : les deux temps se jouent **par document**, non
  page par page — **première passe** sur toutes les pages à 72 dpi (arrêt à la première QR-facture), **seconde passe**
  sur les pages dont le premier rendu a réussi, seulement si **aucune** page n'a livré de QR-facture. Page par page, une
  facture née numérique de trois pages, QR en dernière page — cas courant —, payait le second rendu des pages 1 et 2.
  Une **erreur du second rendu** n'est pas une erreur du document : journalisée, la page compte comme sans QR-facture,
  si bien que le diagnostic d'un document ne change jamais à cause du second temps. Le déclencheur est l'absence de
  **QR-facture** (payload `SPC\n0200`), non de QR : un QR d'un autre usage est lu puis écarté (`first_spc`). Le
  défaut de production du nombre de fichiers par passage est lu à `config.rs:1011-1030` (`config.rs:524`, cité plus
  haut, est la configuration de test ; même valeur, 200, borne 10 000).

### C-v017-5-3 — Le décodeur : format QR seul ; tout autre essai seulement sur preuve d'une fixture

- **Contexte** : `decode_qr_from_image` (`qr_decode.rs:84-94`) appelle `detect_multiple_in_luma`, qui essaie **tous**
  les formats (aucun `PossibleFormats`). Le second rendu porte une page A4 de 0,5 à 2,8 Mpx (× 5,6) : le coût du
  décodage d'une page sans QR lisible au premier temps croît d'autant.
- **Retenu** : (1) `PossibleFormats = {QR_CODE}` (via `detect_multiple_in_luma_with_hints`), aux deux temps, `TryHarder` conservé —
  mesure de **coût**, sans effet sur ce que Kesh retient (seul un payload `SPC\n0200` compte, `first_spc`). (2) Un
  échelon supplémentaire — `FilteredImageReader` (pyramide et fermeture morphologique, `rxing-0.7.1/src/
  filtered_image_reader.rs`), puis, si besoin, une découpe de la bande inférieure de la page — n'est ajouté **que si**
  une fixture du périmètre (AC-1) n'est pas lue sans lui et l'est avec lui ; la mutation qui le retire doit faire rougir
  ce test. Sans une telle fixture, il n'est pas écrit.
- **Écartées** : (b) poser d'office toute l'échelle (inversion `AlsoInverted`, binariseur global, filtre) — coût
  multiplié sans preuve d'utilité ; un QR-facture est noir sur blanc, l'inversion n'a pas de cas ; (c) un second
  décodeur (autre crate) — dépendance neuve, sans preuve qu'il lise mieux.
- **Réversible** : oui.

### C-v017-5-4 — Les bornes : inchangées ; pas de budget de temps d'horloge

- **Contexte** : bornes existantes : 20 pages (`KESH_INBOX_MAX_PDF_PAGES`), 2000 px par côté de rendu, 25 Mio par
  fichier, 10 000 px et 256 Mio pour une image directe (`qr_decode.rs:31-34`). Aucune borne de temps.
- **Retenu** : bornes **inchangées** ; le coût se **mesure** (temps par page, pire document admis, mémoire) contre des
  plafonds écrits dans la fiche (AC-6). Un dépassement se corrige en réduisant le travail (un échelon de moins, l'échelon
  limité à la première et à la dernière page), jamais en relâchant une borne.
- **Écartées** : (b) un budget de temps par document (arrêt du décodage après N secondes) — le résultat dépendrait de
  la charge de la machine : un même document lu un jour, « aucun QR lu » le lendemain. Une borne structurelle (pages ×
  pixels) est décidable ; une borne d'horloge ne l'est pas.
- **Réversible** : oui.
- **Amendé à la validation P2** (F1, R8, arbitrage de l'orchestrateur) : l'absence de borne d'horloge côté serveur
  reste ; mais le **délai du client** passe dans la story (C-v017-5-14), et la conduite quand le client ou le proxy
  abandonne est **vérifiée par une exécution** (fiche, AC-6-bis).

### C-v017-5-5 — Les fixtures : synthétiques, produites une fois et versionnées comme données

- **Contexte** : le mandat exige une fixture synthétique — une QR-facture générée par Kesh, rastérisée, floue et
  bruitée, dans un PDF image — et interdit tout document réel.
- **Retenu** *(précisé par les amendements des validations P1 à P3 ci-dessous : 280 octets UTF-8, version 12 à 15,
  qualité JPEG 60 au moins, fixtures lues à l'exécution)* : un générateur de test (`#[cfg(test)]`) qui (1) construit un payload par `kesh_qrbill::build_payload`
  (IBAN d'exemple SIX déjà employé par les tests, noms fictifs ; référence QRR, adresses structurées, débiteur final et
  message, pour un payload d'**au moins 280 caractères** et un QR de **version ≥ 12**), (2) le dessine sur une page A4 en
  niveaux de gris à 150, 200 ou 300 dpi, à la place que lui donne la norme (section paiement en bas, QR de 46 mm à
  67 mm du bord gauche et 209 mm du haut, croix suisse de 7 mm), papier gris clair et encre gris foncé, (3) applique un
  flou gaussien de 0,1 mm et un bruit uniforme de ± 12 niveaux tiré d'un générateur à graine fixe, (4) encode en JPEG
  (qualité 75), (5) l'enveloppe dans un PDF d'une page A4 (`printpdf` 0.7, `ImageXObject` filtre DCT — paquet déjà au
  `Cargo.lock` par `kesh-qrbill`, ajouté aux `dev-dependencies` de `kesh-api` : aucun paquet neuf, une seule ligne au
  lock). Les trois PDF et un PDF d'une page scannée pleine sans QR sont **produits une fois et versionnés comme
  données** sous `crates/kesh-api/tests/fixtures/`, au plus 1 Mio chacun ; le générateur est versionné et documenté,
  **sans exigence de reproductibilité au bit près** : la preuve porte sur le décodage (lu, non lu), non sur les octets.
  *(Amendé à la validation P1, R1 et F10 : la version d'origine exigeait une régénération « à l'identique (octets
  comparés) », impossible — `printpdf-0.7.0` tire un identifiant d'instance (`pdf_document.rs:791`, `utils.rs`, graine
  globale du processus) et date le document (`pdf_metadata.rs:58`) à chaque écriture ; et elle ne bornait pas la
  taille de fichiers versionnés dans un dépôt public.)*
  *(Amendé à la validation P2, R5, R9, R7, R12 : le QR est dessiné par `kesh_qrbill::generator::render_qr_image`,
  niveau M de la norme SIX — non par `rxing::qrcode::QRCodeWriter`, niveau L par défaut, `qr_code_writer.rs:80` —, payload
  d'au moins 280 octets UTF-8, version **12 à 15** (au-delà, le second rendu tomberait sous 4 px par module) ; fixtures
  lues par `include_bytes!` depuis `CARGO_MANIFEST_DIR` ; `scan-sans-qr.pdf` garde son bruit sur toute la page ; T2 et T3
  décodent les octets JPEG.)*
- **Écartées** : (b) ImageMagick / `img2pdf` — outils hors dépôt, non reproductibles en CI ; (c) une pièce réelle
  anonymisée — interdit.
- **Réversible** : oui.

- **Amendé à la validation P3** *(écrit à la validation P4, finding P4 : l'en-tête du « Retenu » y renvoyait sans qu'il
  existe)* : fixtures lues **à l'exécution** par `std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), …))` — non par
  `include_bytes!`, comme le disait l'amendement P2 ci-dessus, qui empêcherait de compiler les tests avant que le
  générateur ait produit les fichiers ; qualité JPEG **60 au moins** ; `scan-sans-qr.pdf` repasse à 200 dpi s'il dépasse
  1 Mio, et, s'il le dépasse encore, issue demandée — le plafond ne se relève pas en silence ; le banc engendre sa page à
  300 dpi en mémoire ; palier (3) des branches d'arrêt **binarisé avant l'encodage JPEG**.

### C-v017-5-6 — Le chemin PDF ne se teste pas en CI : la preuve pdfium se joue en local, avec la bibliothèque épinglée

- **Contexte** : `libpdfium.so` n'est ni sur la station (`ls /usr/local/lib/libpdfium*` : absent, `ldconfig -p` : rien)
  ni sur les exécuteurs de CI ; seule l'image Docker la porte (`Dockerfile:30-38`, `PDFIUM_RELEASE=chromium/7920`, somme
  SHA-256 vérifiée). Le seul test PDF du module est `#[ignore]` (`qr_decode.rs:301-313`) et lit
  `tests/fixtures/qr_invoice.pdf`, **qui n'existe pas** (`ls crates/kesh-api/tests/fixtures/` :
  `keshbackup-0.14.0-format1.keshbackup`, `spa-stub`) — même lancé avec la bibliothèque, il paniquerait.
- **Retenu** : (1) ce qui se teste **sans** pdfium se teste en CI : `render_scale` (fonction pure) et le décodage
  d'images synthétiques rééchantillonnées à la résolution de rendu ; (2) les tests pdfium restent `#[ignore]` et se
  **jouent en local** au développement, avec la même release épinglée que le `Dockerfile` (téléchargée par le
  développeur sous `/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/`, somme vérifiée, `LD_LIBRARY_PATH`), sortie copiée
  au Dev Agent Record ; (3) le test cassé de `qr_decode.rs:301-313` est **réparé** (correction mécanique : il lit une
  fixture qui existe) ; (4) la recette de l'orchestrateur passe par l'image Docker de la branche, sur une installation
  jetable.
- **Écartées** : (b) installer pdfium en CI — changement d'outillage hors périmètre ; tracé en **#856**.
- **Réversible** : oui.

### C-v017-5-7 — Ce que l'écran dit : « Aucune QR-facture exploitable » (« Aucun QR lu » puis « Aucune QR-facture lue » jusqu'aux validations P2 et P3), par les valeurs des clés existantes

- **Contexte** : le badge « Sans QR-code » et la mention « sans QR-code » du rapport affirment un fait sur le document
  que Kesh ne connaît pas : il sait seulement qu'il n'a **lu** aucun QR. Le badge couvre aussi le QR inexploitable
  (`+page.svelte:293-315`, `{#if hasQr(row)}` … `{:else}` : tout ce qui n'a pas de données QR stockées) ; le manuel le dit (`user-manual.tex:1857`).
- **Retenu** *(libellé remplacé par les amendements des validations P2 et P3 ci-dessous)* : changer les **valeurs** de `imported-supplier-invoices-badge-no-qr` et `-report-no-qr` dans les quatre
  locales — fr « Aucun QR lu » / « aucun QR lu », de « Kein QR-Code gelesen » / « kein QR-Code gelesen », it « Nessun
  codice QR letto » / « nessun codice QR letto », en « No QR code read » / « no QR code read » — et les replis égaux des
  deux composants ; **clés inchangées**, valeur d'API `none` inchangée (contrat). Le serveur MCP dit déjà `qr_lu`
  (`services/pieces.rs:80`) : le vocabulaire se rejoint. `README.md:44`, qui nomme le badge, suit (F1 de la validation P1).
  **Limite assumée** (F8) : un QR lu puis refusé (`FIELD_TOO_LONG`, montant, IBAN) est « QR illisible » au rapport et
  « Aucun QR lu » dans la liste — la distinction n'est pas stockée ; le manuel dit « aucun QR **exploitable** ».
- **Écartées** : (b) renommer les clés — churn sans gain ; (c) distinguer à l'écran « document sans QR » et « QR non
  lu » — impossible sans lire le texte du document (#854 le dit) ; (d) changer la valeur `none` de l'API — rupture de
  contrat pour un mot.
- **Réversible** : oui.

- **Amendé à la validation P2** (F3, F2, arbitrage de l'orchestrateur) : « Aucun QR lu » est faux pour un document qui
  porte un autre QR qu'une QR-facture, lu puis écarté par `first_spc`. Libellé retenu : **« Aucune QR-facture lue »**
  (de « Keine QR-Rechnung gelesen », it « Nessuna fattura QR letta », en « No QR-bill read » ; minuscule au rapport),
  calqué sur le badge voisin `badge-qr` (« QR-facture », « QR-Rechnung », « Fattura QR », « QR-bill ») — exact dans tous
  les cas où il s'affiche ; soumis au Project Lead (fiche, Q1). Les deux clés sont ajoutées à `REPLIS_A_SITE_UNIQUE`
  (garde G13 de `i18n-repli-divergent-actif.test.ts:222`) : aucune autre garde ne compare ces replis au catalogue.
  Écartée : « Aucun QR lu » (faux pour un QR non SPC) ; « QR non lu » (même défaut).

- **Amendé à la validation P3** (F1, arbitrage de l'orchestrateur) : « Aucune QR-facture lue » est faux quand une
  QR-facture a été lue puis refusée (`check_scanned_fits_columns`, `inbox_import.rs:555-559`). Libellé retenu :
  **« Aucune QR-facture exploitable »** (de « Keine verwertbare QR-Rechnung », it « Nessuna fattura QR utilizzabile », en
  « No usable QR-bill ») — vrai dans les quatre cas où le badge s'affiche (aucun QR, QR non SPC, QR-facture refusée,
  erreur de rendu). Il fait foi tant que le Project Lead n'a pas répondu à Q1. À l'API, la valeur `none` se décrit
  toujours « aucune QR-facture lue » : elle ne couvre que les deux premiers cas.
- **Amendé à la validation P4** (P3, arbitrage de l'orchestrateur) : l'énumération omettait le cas de #854 lui-même —
  une QR-facture **présente mais non détectée** (`qr_decode.rs:92`, `:112` ; `inbox_import.rs:564`), qui sort `none`.
  Cinq cas, donc : aucun QR ; QR d'un autre usage ; QR-facture non détectée ; QR-facture lue puis refusée ; erreur de
  rendu. Le libellé retenu reste vrai dans les cinq. À l'API, `none` se décrit : « aucune QR-facture n'a été détectée —
  le document peut ne porter aucun QR, un QR d'un autre usage, ou une QR-facture que la lecture n'a pas trouvée » ; la
  phrase « elle ne couvre que les deux premiers cas » ci-dessus est remplacée par : elle couvre les trois premiers.

### C-v017-5-8 — Relire une pièce déjà importée : à la réactivation, pas par un bouton (bouton tracé en #858)

- **Contexte** : rien n'est relu d'office (contrainte des données à garder). Aujourd'hui, retirer un document puis le
  redéposer le **réactive sans le relire** : le chemin `discarded` (`inbox_import.rs:477-531`) rend le statut stocké
  (`:519-523`) et le décodage (`:539-576`) vient **après** le court-circuit du doublon. Un document importé en 0.16.x
  « sans QR » le resterait donc après la mise à jour, sans aucun geste qui le fasse relire.
- **Retenu** : à la **réactivation** d'un document `discarded` **sans données QR stockées**, Kesh **relit** le fichier
  déposé (mêmes octets, même empreinte) avec le décodeur courant et, s'il lit un QR exploitable, en remplit les colonnes
  dans la même transaction que la réactivation. Le geste de l'utilisateur : *Télécharger*, *Retirer de la liste*,
  déposer le fichier téléchargé dans le dossier d'import, *Importer le dossier*. Aucun doublon : la ligne existante est
  réactivée, l'empreinte est la même. Un bouton « Relire le QR » est **hors du périmètre minimal**.
- **Pourquoi pas le bouton** : le décodeur est déterministe (aucune source d'aléa dans pdfium ni `rxing`) — à
  version, réglage de pages (`KESH_INBOX_MAX_PDF_PAGES`) et bibliothèque PDF égaux, relire rend le même résultat ; relire
  n'a de sens qu'après un changement de l'un d'eux (mise à jour de Kesh, borne relevée, `libpdfium` installée),
  exactement les cas que couvre la réactivation, par un geste que le
  manuel décrit déjà (« le déposer de nouveau le fait revenir dans la liste », `user-manual.tex:1875-1876`). Un bouton
  demanderait une route, son contrôle de rôle, sa trace d'audit, son exposition aux clés d'API, quatre locales, un E2E et
  des captures, pour un geste rare. La demande de #854 (« proposer de réessayer la lecture ») est tracée en **#858**.
- **Écartées** : (b) relire d'office à la montée de version — interdit (rien ne se relit d'office ; une migration ne
  décode pas de fichier) ; (c) relire aussi un document `to_complete` redéposé — il est aujourd'hui un doublon
  (`DUPLICATE`) ; le relire changerait ce contrat sans nécessité, le retrait d'abord suffit ; (d) relire un document
  joint à une facture (`completed`) — ses coordonnées de paiement ont été saisies, un QR relu les contredirait. Un
  document **détaché** de sa facture redevient `to_complete` (`imported_supplier_invoices.rs:339-341`) et suit alors le
  sort d'un document à traiter : retiré puis redéposé, il est relu s'il n'a aucun QR (F6 de la validation P1).
- **Réversible** : oui (aucune donnée n'est réécrite hors des colonnes QR vides d'une ligne réactivée).

### C-v017-5-9 — La relecture n'écrit que des colonnes vides, en une instruction gardée

- **Contexte** : la réactivation (`imported_supplier_invoices.rs:480-499`) est un `UPDATE … WHERE status =
  'discarded'` ; les sept colonnes QR obligatoires vont ensemble (`chk_imported_si_qr_all_or_nothing`, migration
  `20261015000001`).
- **Retenu** : une fonction de dépôt neuve, `reactivate_to_complete_with_qr(tx, company_id, id, &données_qr)`, **une
  seule** instruction : `UPDATE … SET status = 'to_complete', supplier_invoice_id = NULL, <colonnes QR> … WHERE id = ?
  AND company_id = ? AND status = 'discarded' AND creditor_iban IS NULL`. Zéro ligne → même conduite qu'aujourd'hui (la
  course est traitée en doublon, `inbox_import.rs:527-530`). Elle écrit les **quinze** colonnes QR (les sept
  obligatoires et `creditor_line1`, `creditor_line2`, `creditor_postal_code`, `creditor_town`, `reference_value`,
  `amount`, `unstructured_message`, `billing_information`), depuis une projection `QrColumns` **extraite** de
  `NewImportedSupplierInvoice::from_scanned` (`entities/imported_supplier_invoice.rs:260-296`) et partagée par l'import
  et la réactivation — aucune seconde règle, aucun `DocumentMeta` fabriqué (R8 de la validation P1). Un test de dépôt
  (T9) prouve la garde seule, tuant la mutation M7a (R2). La trace `imported_supplier_invoice.reactivated`
  porte en plus `"qr"` (`present`, `none`, `unreadable`) et `"qr_reread"` (booléen). Le rapport dit le résultat de la
  relecture, avec `qrError` si inexploitable.
- **Écartées** : (b) réactiver puis remplir en deux instructions — fenêtre où la ligne est `to_complete` sans QR et
  qu'une garde manquée laisserait écrire sur une ligne pourvue ; (c) réécrire les colonnes d'une ligne qui a déjà un
  QR — interdit (données à garder).
- **Réversible** : oui.

### C-v017-5-10 — Pas de découpage

- **Contexte** : modules touchés : `kesh-api/qr_decode`, `kesh-api/inbox_import`, `kesh-db/repositories`
  (une fonction), `kesh-i18n` (deux valeurs × quatre locales), le frontend (deux replis et leurs tests), la
  documentation. Cinq modules de code (deux dans `kesh-api`), dont deux par des valeurs de texte, et la documentation —
  pas « plus de cinq »
  *(Recompté à la validation P3, F8 : l'extraction de `QrColumns` ajoute `kesh-db/entities` (et son `mod.rs`) au
  dépôt `kesh-db/repositories` ; les deux portent la même table, `imported_supplier_invoices`, et se comptent comme un
  seul module métier, « kesh-db / documents importés » ; côté frontend, la route d'import, la fonctionnalité
  `imported-supplier-invoices` et la garde i18n sont un seul module métier, « documents à traiter ». Cinq modules métier
  de code : `qr_decode`, `inbox_import`, kesh-db / documents importés, `kesh-i18n`, frontend / documents à traiter —
  le seuil n'est pas franchi. **Dérogation règle de splitting**, au cas où un lecteur compterait `entities` et
  `repositories` à part (six) : la relecture à la réactivation n'a de sens qu'avec le décodeur qu'elle applique, et
  `QrColumns` existe pour que l'import et la réactivation projettent les mêmes colonnes ; les séparer livrerait une
  projection sans consommateur. Risque accepté : une story à six modules de code (quatre dans `kesh-db`/`kesh-api`, cinq fichiers de production :
  `qr_decode.rs`, `inbox_import.rs`, `entities/imported_supplier_invoice.rs`, `entities/mod.rs`,
  `repositories/imported_supplier_invoices.rs`).)*
  (`CLAUDE.md` § *Règle de splitting préventif*).
- **Retenu** : une story. La relecture à la réactivation est ce qui rend le correctif utile aux pièces déjà importées ;
  la séparer laisserait livrer un décodeur meilleur que personne ne peut appliquer à l'existant.
- **Écartées** : (b) v017-5a décodeur, v017-5b relecture — deux fiches pour un correctif d'une release candidate.
- **Réversible** : oui.

### C-v017-5-11 — Défauts voisins : tracés, non intégrés

- **Contexte** : (1) le décodage est synchrone dans une fonction `async` (`inbox_import.rs:540-550`, aucun
  `spawn_blocking`) : il occupe un fil de l'exécuteur tokio pendant le rendu et le décodage — le coût par page augmente
  avec cette story ; (2) le chemin PDF n'est exercé par aucun test qui tourne, en CI comme en local (C-v017-5-6).
- **Retenu** : (1) tracé en **#855** (P3) — le passage en `spawn_blocking` change la propagation d'une panique et le
  partage du verrou pdfium : pas mécanique ; (2) tracé en **#856** (P3, test) ; la réparation du test cassé, elle, est
  mécanique et intégrée (C-v017-5-6).
- **Réversible** : oui.

### C-v017-5-12 — #854 ne se ferme qu'au constat du Project Lead

*(Ajouté à la validation P1, décision de l'orchestrateur — R14.)*

- **Contexte** : la preuve de la story porte sur des fixtures synthétiques ; les deux pièces de #854 sont réelles et ne
  s'ouvrent pas ici. Fermer #854 sur la seule preuve synthétique fermerait peut-être un défaut réel non corrigé.
- **Retenu** : commits **et PR de release** en `refs #854` ; #854 se ferme quand le Project Lead a constaté, sur son
  installation, après la mise à jour, que ses deux factures communales sont lues (retrait et redépôt), et rapporte
  seulement « lues » ou « non lues ». Recette écrite dans la fiche (§ *Fermeture de #854*).
- **Écartées** : (b) `closes #854` sur la PR — fermeture sans constat sur le cas réel.
- **Réversible** : oui.

### C-v017-5-13 — La cause se teste en CI par sa grandeur, non par une faiblesse du décodeur

*(Ajouté à la validation P1 — R5, R6.)*

- **Contexte** : le témoin « non lu à 595 × 842 px » de T2 exigeait que `rxing` **échoue** ; `rxing = "0.7"` admet toute
  0.7.x, et une montée peut le faire lire sans que rien ne soit faux. Le rééchantillonnage `Triangle` n'est pas celui de
  pdfium.
- **Retenu** : en CI, une assertion sur la **grandeur causale** (pixels par module au premier et au second rendu,
  calculés depuis la version du QR des fixtures) ; le contrôle « non lu à 72 dpi » devient un diagnostic `#[ignore]`,
  joué à la main avec T4 ; **T4, sous pdfium, fait foi** en cas de contradiction avec T2. Le témoin vectoriel de T4 a un
  payload nommé (celui de `render_spc_qr_png`, `qr_decode.rs:210-227`) et une branche d'arrêt s'il n'est pas lu au
  commit rouge.
- **Écartées** : (b) garder le témoin négatif en CI — fragile par construction.
- **Réversible** : oui.

### C-v017-5-14 — Le délai du client s'aligne sur celui du proxy

*(Ajouté à la validation P2 — F1, R8, arbitrage de l'orchestrateur.)*

- **Contexte** : `triggerInboxImport` (`imported-supplier-invoices.api.ts:27-29`) emploie le délai par défaut du client,
  **30 s** (`api-client.ts:42`) ; le proxy documenté coupe à **300 s** (`admin-manual.tex:341`). La story double le rendu
  des pages sans QR-facture : l'écran abandonnerait un import de quelques documents scannés.
- **Retenu** : `INBOX_IMPORT_TIMEOUT_MS = 300_000`, exporté par `imported-supplier-invoices.api.ts`, et `{ timeoutMs:
  INBOX_IMPORT_TIMEOUT_MS, retry: false }` (`CallOptions`, `api-client.ts:59-64`) ; test Vitest ; coût mesuré contre 30 s
  et 300 s ; conduite à l'abandon (sort de l'import, verrou `RELEASE_LOCK`, `inbox_import.rs:199-207`) vérifiée par
  exécution ; si le verrou fuit, issue P2 demandée, correction dans la story seulement si mécanique.
- **Écartées** : (b) `LONG_OPERATION_TIMEOUT_MS` (une heure, `api-client.ts:53`) — au-delà du proxy, l'écran attendrait
  une réponse qui ne viendra jamais ; (c) laisser 30 s et tracer — l'écran annoncerait en échec un import qui se
  poursuit.
- **Réversible** : oui.

### C-v017-5-15 — La boucle des deux passes est extraite sur un rendeur injecté, et testée en CI

*(Ajouté à la validation P2 — R4, arbitrage de l'orchestrateur.)*

- **Contexte** : la logique des deux passes (ordre, déclencheur, QR-facture inexploitable, erreurs de rendu des deux
  temps, cap) ne dépend de pdfium que par « rendre la page *p* au temps *t* » ; écrite dans `decode_spc_from_pdf_bytes`,
  elle ne serait testée que par T4, en local.
- **Retenu** : `decode_pages(page_count, max_pages, render: FnMut(usize, Temps) -> Result<Option<DynamicImage>,
  String>)`, appelée en production avec un rendeur pdfium ; tests d'hôte avec un faux rendeur (T10), qui tuent en CI les
  mutations des deux passes (M11, M12, M12a à M12d). T4 reste la preuve de bout en bout.
- **Écartées** : (b) garder la boucle dans la fonction pdfium — logique centrale non testée en CI (#856).
- **Réversible** : oui.

### C-v017-5-16 — Les paliers de dégradation joués par le développement, dans leur ordre ; le palier 2 retenu

*(Ajouté au développement, 2026-10-31.)*

- **Contexte** : avec la dégradation d'AC-1 (flou 0,1 mm, bruit ± 12), T4 au code d'avant **lit** les trois scans :
  branche d'arrêt (i). La fiche confie l'ordre des paliers à l'orchestrateur ; le mandat de développement dit
  « si la fixture ne reproduit pas la cause, **même après les paliers de dégradation écrits d'avance** », arrête-toi.
- **Retenu** : les paliers joués **dans l'ordre écrit**, cumulatifs comme la fiche les écrit (« puis »), sans régler
  aucun autre paramètre : palier 1 (bruit ± 24) — 150 et 200 dpi encore lus ; palier 2 (+ flou 0,2 mm) — les trois scans
  non lus à 72 dpi, toujours lus à 1414 × 2000 (T2) et à la résolution native (T3). Le palier 2 devient celui des
  fixtures ; le palier 3 n'est pas joué. Journaux : `kesh-gate-logs/v0175/rouge-t4.log`, `palier1.log`, `palier2*.log`.
- **Écartées** : (b) s'arrêter dès la dégradation d'AC-1 — le mandat nomme les paliers comme partie de la reproduction ;
  (c) régler bruit ou flou à la main jusqu'au rouge — interdit (AC-2 (i)).
- **Réversible** : oui (une constante, `DEGRADATION`, et quatre fichiers de données).
- *Annotation datée du 2026-10-31, revue de code P1 (A1)* : au palier 1 et au premier essai du palier 2, le générateur a
  paniqué sur la fixture à 300 dpi **après** avoir écrit les précédentes ; `qr-scan-300dpi.pdf` et `scan-sans-qr.pdf`
  sont restés ceux du palier 0, et T4 les a relus (`palier1.log`, `palier2.log` : « qr-scan-300dpi.pdf : LU »). La
  conclusion tient : au palier 1, les fixtures à 150 et 200 dpi, régénérées, étaient encore lues ; le palier 2 est
  prouvé par **`palier2d.log`**, sur les quatre fixtures finales. Le générateur contrôle désormais toutes les tailles
  avant d'en écrire une.

### C-v017-5-17 — Le plafond de 1 Mio tenu par les deux moyens prescrits

- **Contexte** : au palier 2, `qr-scan-300dpi.pdf` (1 606 339 octets, qualité 65) et `scan-sans-qr.pdf` (1 610 809 octets,
  qualité 65 ; 1 417 096 à la qualité 60) dépassent 1 Mio.
- **Retenu** : pour les trois scans à QR, **bruit restreint à la section paiement** (dès 192 mm du haut ; flou sur
  toute la page), qualité 65 ; pour la page sans QR, bruit sur toute la page, qualité **60**, puis **200 dpi** (AC-1).
  Le banc engendre sa page D1 à 300 dpi en mémoire, à la qualité 60.
- **Écartées** : (b) qualité sous 60 — interdit ; (c) relever le plafond — interdit ; (d) bruit restreint sur la page
  sans QR — AC-1 l'exclut.
- **Réversible** : oui.
- *Annotation datée du 2026-10-31, revue de code P1 (question de la lentille B)* — **la restriction du bruit ne règle
  aucun paramètre sur le résultat.** (1) Elle a été décidée sur la **taille** (1 606 339 octets), après que le palier 2,
  bruit sur toute la page, eut déjà reproduit le défaut : `palier2.log` montre les fixtures à 150 et 200 dpi non lues à
  72 dpi, et T2-diag, sur le générateur en mémoire avec le bruit sur toute la page, les trois non lues. (2) Elle ne
  touche pas la zone qui décide : la section paiement, où sont tout le QR et la croix, garde son bruit de ± 24 et son
  flou de 0,2 mm ; seul le haut de la page (au-dessus de 192 mm), sans QR, perd son bruit. Une réserve : le tirage du
  bruit sur le QR n'est pas le même octet pour octet (la suite pseudo-aléatoire ne parcourt plus le haut de la page) —
  même loi, autre réalisation. (3) Les fixtures finales **échouent au premier temps et sont lues au second** :
  `palier2d.log` (code d'avant : les trois scans `Ok(None)`), `vert-t4.log` (correctif : les trois lus, un rendu au
  second temps chacun), et T4 rejoué au dernier commit de code de la remédiation P1 (Dev Agent Record).

### C-v017-5-19 — Pas de second rendu quand le premier a déjà donné la même image

*(Ajouté à la remédiation de la revue de code P1 — E-3, ordonné par l'orchestrateur.)*

- **Contexte** : E-3 avançait qu'une page de plus de 2000 pt de côté se rend au second temps en doublon du premier.
  Prémisse relue dans `pdfium-render-0.8.37/src/pdf/document/page/render_config.rs:763-822` : **à moitié vraie**. Sans
  facteur d'échelle, `apply_to_page` ne garde pas les proportions (`do_maintain_aspect_ratio = false`) et réduit chaque
  côté séparément à la borne ; le second rendu, à `scale_page_by_factor`, les garde. Le second redonne donc exactement
  le premier **seulement** quand les deux facteurs du premier, `min(1, borne / côté)`, égalent l'échelle du second —
  page plus grande que la borne dans les deux sens aux proportions de la borne, ou page à l'échelle 1. Une page plus
  grande que la borne dans un seul sens est **déformée** au premier temps : son second rendu, sans déformation, n'est
  pas un doublon. La consigne littérale (« pas de second rendu dès que l'échelle est ≤ 1 ») aurait retiré le seul rendu
  non déformé de ces pages, contre AC-3 (« le second temps la rend sans déformation »).
- **Retenu** : `echelle_du_second_temps` (fonction pure, `qr_decode.rs`) — `render_scale`, sauf quand le premier rendu
  a donné la même image (`None`, pas de second rendu). Tests d'hôte : A4 (2,3756), 3000 × 3000 pt (`None`),
  2500 × 1000 pt (0,8, gardé), dimension nulle (`None`) ; mutation M15 (filtre retiré).
- **Écartées** : (b) `None` dès que l'échelle est ≤ 1 — supprime le rendu non déformé (ci-dessus) ; (c) laisser le
  doublon en angle mort — coût sans contrepartie, même borné.
- **Réversible** : oui.

### C-v017-5-18 — Le verrou de run fuit à l'abandon : la connexion du verrou est fermée à sa restitution

*(Ajouté au développement, 2026-10-31 — AC-6-bis.)*

- **Contexte** : T12 (client à délai de 2 s, douze PNG de bruit) montre que hyper **abandonne** le gestionnaire à
  l'abandon du client : l'import s'arrête au fichier en cours (7 documents traités, 5 restés dans le dossier). Avec un
  pool **réglé comme la production** (`main.rs:82-85` : `max_connections(5)`, délais par défaut de `sqlx`, dont
  `idle_timeout` de dix minutes), `IS_FREE_LOCK` vaut **0** une fois le passage arrêté : la `PoolConnection` du verrou,
  abandonnée sans `RELEASE_LOCK`, **retourne au pool** (`sqlx-core-0.8.6/src/pool/connection.rs:199-210`, `:275-330`)
  en gardant le verrou dans sa session ; et le second import répond **200** en reprenant cette même session (verrou
  ré-entrant) sans le rendre — exactement le piège que la fiche prévoyait. Avec le pool de `#[sqlx::test]`, qui ferme
  toute connexion inactive au bout d'une seconde (`sqlx-mysql-0.8.6/src/testing/mod.rs:181`), le premier essai de T12
  était **vert à tort**.
- **Retenu** : correction **mécanique**, autorisée par AC-6-bis — `lock_conn.close_on_drop()` juste après l'acquisition
  (`inbox_import.rs`) : la connexion du verrou est fermée à sa restitution, normale ou abandonnée ; la session s'achève et
  MariaDB rend le verrou. Coût : une connexion neuve par import. T12 tourne avec un pool réglé comme la production, et
  le dit en tête. Issue P2 **demandée** à l'orchestrateur (fuite constatée, corrigée dans la story) pour la trace.
- **Écartées** : (b) une garde `Drop` qui `detach()` la connexion — même effet, plus de code ; (c) `tokio::spawn` du
  passage — change la conduite à l'abandon (le passage irait au bout sans client), hors périmètre (#855) ; (d) ne rien
  corriger — l'écran annoncerait un import déjà en cours pendant jusqu'à dix minutes après un délai dépassé, que la
  story rend plus probable.
- **Réversible** : oui (une ligne).
