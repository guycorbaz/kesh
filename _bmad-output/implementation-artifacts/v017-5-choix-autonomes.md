# Release 0.17.0 — registre des choix autonomes de la Story v017-5 (lire le QR d'une facture scannée)

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** au merge. Fiche :
`v017-5-qr-scanne.md` (#854). Numérotation `C-v017-5-k`. Issue en `refs` seulement dans les commits. Les numéros de
ligne cités sont ceux de `8e126cfd` (tête de `release/0.17.0`, point de départ de la branche `story/v017-5-qr-scanne`) ;
ceux des dépendances, ceux de `~/.cargo/registry` (`pdfium-render-0.8.37`, `rxing-0.7.1`, `printpdf-0.7.0`).

Rédigé le 2026-10-31 en autonomie (Opus 5.5), sous la contrainte du Project Lead du 2026-10-30
(`kesh-gate-logs/prompts/v017-donnees-a-garder.md` : les données de la 0.16.1-rc1 se gardent) et sans ouvrir aucune
pièce réelle. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Les choix marqués **(Q)**
figurent aussi parmi les questions au Project Lead (fiche, § *Questions au Project Lead*).

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

### C-v017-5-2 — Le remède : un seul rendu par page, à la plus haute résolution que la borne existante permet, plafonnée à 300 dpi

- **Contexte** : le mandat propose une résolution plus élevée, et un second essai plus fin seulement si le premier ne
  trouve rien, **sans relâcher** les bornes anti-DoS. La borne de dimension (2000 px, `qr_decode.rs:46-53`) permet, pour
  une A4 portrait, **171 dpi** (2000 / 11,69 in) — 4,2 à 6,3 px par module, assez pour un QR scanné de 150 à 300 dpi :
  au-delà de la résolution source, un rendu plus fin n'apporte aucune information.
- **Retenu** : une fonction pure `render_scale(largeur_pt, hauteur_pt, max_dimension) -> f32` = `min(max_dimension /
  largeur, max_dimension / hauteur, MAX_RENDER_DPI / 72)` avec `MAX_RENDER_DPI = 300`, appliquée par
  `scale_page_by_factor` ; les bornes `set_maximum_width/height` restent posées (défense en profondeur). Un seul rendu
  par page, comme aujourd'hui.
- **Écartées** : (b) un premier rendu à 150 dpi puis un second plus fin — au-delà de 171 dpi sur une A4, la borne de
  2000 px est franchie : un second rendu « plus fin » de la page entière **relâcherait** la borne, ce que le mandat
  interdit ; (c) un second rendu de la seule bande inférieure (section paiement) à 240 dpi par matrice de transformation
  et découpe pdfium — faisable dans la borne, mais intestable sur l'hôte et en CI (pas de `libpdfium`, C-v017-5-6), et
  sans gain attendu sur une source de 150 à 300 dpi rendue déjà à 171 dpi ; il reste ouvert **seulement** si T4 l'exige
  (fiche, AC-2, branche d'arrêt) ; (d) relever `max_dimension` — interdit par le mandat.
- **Réversible** : oui (une constante, une fonction).

### C-v017-5-3 — Le décodeur : format QR seul ; tout autre essai seulement sur preuve d'une fixture

- **Contexte** : `decode_qr_from_image` (`qr_decode.rs:84-94`) appelle `detect_multiple_in_luma`, qui essaie **tous**
  les formats (aucun `PossibleFormats`). Le rendu passe de 0,5 à 2,8 Mpx par page A4 (× 5,6) : le coût du décodage
  croît d'autant.
- **Retenu** : (1) `PossibleFormats = {QR_CODE}` (via `detect_multiple_in_luma_with_hints`), `TryHarder` conservé —
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

### C-v017-5-5 — Les fixtures : synthétiques, engendrées par un générateur versionné et déterministe

- **Contexte** : le mandat exige une fixture synthétique — une QR-facture générée par Kesh, rastérisée, floue et
  bruitée, dans un PDF image — et interdit tout document réel.
- **Retenu** : un générateur de test (`#[cfg(test)]`) qui (1) construit un payload par `kesh_qrbill::build_payload`
  (IBAN d'exemple SIX déjà employé par les tests, noms fictifs ; référence QRR, adresses structurées, débiteur final et
  message, pour un payload d'**au moins 280 caractères** et un QR de **version ≥ 12**), (2) le dessine sur une page A4 en
  niveaux de gris à 150, 200 ou 300 dpi, à la place que lui donne la norme (section paiement en bas, QR de 46 mm à
  67 mm du bord gauche et 209 mm du haut, croix suisse de 7 mm), papier gris clair et encre gris foncé, (3) applique un
  flou gaussien de 0,1 mm et un bruit uniforme de ± 12 niveaux tiré d'un générateur à graine fixe, (4) encode en JPEG
  (qualité 75), (5) l'enveloppe dans un PDF d'une page A4 (`printpdf` 0.7, `ImageXObject` filtre DCT — paquet déjà au
  `Cargo.lock` par `kesh-qrbill`, ajouté aux `dev-dependencies` de `kesh-api` : aucun paquet neuf). Les trois PDF et un PDF
  sans QR (même chaîne, sans QR) sont **versionnés** sous `crates/kesh-api/tests/fixtures/`, le générateur à côté, et un
  test `#[ignore]` les régénère à l'identique (octets comparés).
- **Écartées** : (b) ImageMagick / `img2pdf` — outils hors dépôt, non reproductibles en CI ; (c) une pièce réelle
  anonymisée — interdit.
- **Réversible** : oui.

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
- **Écartées** : (b) installer pdfium en CI — changement d'outillage hors périmètre ; demandé en issue (fiche, I2).
- **Réversible** : oui.

### C-v017-5-7 — Ce que l'écran dit : « Aucun QR lu », par les valeurs des clés existantes

- **Contexte** : le badge « Sans QR-code » et la mention « sans QR-code » du rapport affirment un fait sur le document
  que Kesh ne connaît pas : il sait seulement qu'il n'a **lu** aucun QR. Le badge couvre aussi le QR inexploitable
  (`+page.svelte:293-315`, `{#if hasQr(row)}` … `{:else}` : tout ce qui n'a pas de données QR stockées) ; le manuel le dit (`user-manual.tex:1857`).
- **Retenu** : changer les **valeurs** de `imported-supplier-invoices-badge-no-qr` et `-report-no-qr` dans les quatre
  locales — fr « Aucun QR lu » / « aucun QR lu », de « Kein QR-Code gelesen » / « kein QR-Code gelesen », it « Nessun
  codice QR letto » / « nessun codice QR letto », en « No QR code read » / « no QR code read » — et les replis égaux des
  deux composants ; **clés inchangées**, valeur d'API `none` inchangée (contrat). Le serveur MCP dit déjà `qr_lu`
  (`services/pieces.rs:80`) : le vocabulaire se rejoint.
- **Écartées** : (b) renommer les clés — churn sans gain ; (c) distinguer à l'écran « document sans QR » et « QR non
  lu » — impossible sans lire le texte du document (#854 le dit) ; (d) changer la valeur `none` de l'API — rupture de
  contrat pour un mot.
- **Réversible** : oui.

### C-v017-5-8 — Relire une pièce déjà importée : à la réactivation, pas par un bouton (Q)

- **Contexte** : rien n'est relu d'office (contrainte des données à garder). Aujourd'hui, retirer un document puis le
  redéposer le **réactive sans le relire** : le chemin `discarded` (`inbox_import.rs:477-531`) rend le statut stocké
  (`:519-523`) et le décodage (`:539-576`) vient **après** le court-circuit du doublon. Un document importé en 0.16.x
  « sans QR » le resterait donc après la mise à jour, sans aucun geste qui le fasse relire.
- **Retenu** : à la **réactivation** d'un document `discarded` **sans données QR stockées**, Kesh **relit** le fichier
  déposé (mêmes octets, même empreinte) avec le décodeur courant et, s'il lit un QR exploitable, en remplit les colonnes
  dans la même transaction que la réactivation. Le geste de l'utilisateur : *Télécharger*, *Retirer de la liste*,
  déposer le fichier téléchargé dans le dossier d'import, *Importer le dossier*. Aucun doublon : la ligne existante est
  réactivée, l'empreinte est la même. Un bouton « Relire le QR » est **hors du périmètre minimal**.
- **Pourquoi pas le bouton** : le décodeur est déterministe — à version égale, relire rend le même résultat ; relire
  n'a de sens qu'**après une mise à jour** de Kesh, exactement le cas que couvre la réactivation, par un geste que le
  manuel décrit déjà (« le déposer de nouveau le fait revenir dans la liste », `user-manual.tex:1875-1876`). Un bouton
  demanderait une route, son contrôle de rôle, sa trace d'audit, son exposition aux clés d'API, quatre locales, un E2E et
  des captures, pour un geste rare.
- **Écartées** : (b) relire d'office à la montée de version — interdit (rien ne se relit d'office ; une migration ne
  décode pas de fichier) ; (c) relire aussi un document `to_complete` redéposé — il est aujourd'hui un doublon
  (`DUPLICATE`) ; le relire changerait ce contrat sans nécessité, le retrait d'abord suffit ; (d) relire un document
  dont la facture est créée (`completed`) — ses coordonnées de paiement ont été saisies, un QR relu les contredirait.
- **Réversible** : oui (aucune donnée n'est réécrite hors des colonnes QR vides d'une ligne réactivée).

### C-v017-5-9 — La relecture n'écrit que des colonnes vides, en une instruction gardée

- **Contexte** : la réactivation (`imported_supplier_invoices.rs:480-499`) est un `UPDATE … WHERE status =
  'discarded'` ; les sept colonnes QR obligatoires vont ensemble (`chk_imported_si_qr_all_or_nothing`, migration
  `20261015000001`).
- **Retenu** : une fonction de dépôt neuve, `reactivate_to_complete_with_qr(tx, company_id, id, &données_qr)`, **une
  seule** instruction : `UPDATE … SET status = 'to_complete', supplier_invoice_id = NULL, <colonnes QR> … WHERE id = ?
  AND company_id = ? AND status = 'discarded' AND creditor_iban IS NULL`. Zéro ligne → même conduite qu'aujourd'hui (la
  course est traitée en doublon, `inbox_import.rs:527-530`). Les valeurs viennent de la même projection que l'import
  (`NewImportedSupplierInvoice::from_scanned`) — aucune seconde règle. La trace `imported_supplier_invoice.reactivated`
  porte en plus `"qr"` (`present`, `none`, `unreadable`) et `"qr_reread"` (booléen). Le rapport dit le résultat de la
  relecture, avec `qrError` si inexploitable.
- **Écartées** : (b) réactiver puis remplir en deux instructions — fenêtre où la ligne est `to_complete` sans QR et
  qu'une garde manquée laisserait écrire sur une ligne pourvue ; (c) réécrire les colonnes d'une ligne qui a déjà un
  QR — interdit (données à garder).
- **Réversible** : oui.

### C-v017-5-10 — Pas de découpage

- **Contexte** : modules touchés : `kesh-api/qr_decode`, `kesh-api/inbox_import`, `kesh-db/repositories`
  (une fonction), `kesh-i18n` (deux valeurs × quatre locales), le frontend (deux replis et leurs tests), la
  documentation. Cinq modules au plus, dont deux par des valeurs de texte — pas « plus de cinq »
  (`CLAUDE.md` § *Règle de splitting préventif*).
- **Retenu** : une story. La relecture à la réactivation est ce qui rend le correctif utile aux pièces déjà importées ;
  la séparer laisserait livrer un décodeur meilleur que personne ne peut appliquer à l'existant.
- **Écartées** : (b) v017-5a décodeur, v017-5b relecture — deux fiches pour un correctif d'une release candidate.
- **Réversible** : oui.

### C-v017-5-11 — Défauts voisins : tracés, non intégrés

- **Contexte** : (1) le décodage est synchrone dans une fonction `async` (`inbox_import.rs:540-550`, aucun
  `spawn_blocking`) : il occupe un fil de l'exécuteur tokio pendant le rendu et le décodage — le coût par page augmente
  avec cette story ; (2) le chemin PDF n'est exercé par aucun test qui tourne, en CI comme en local (C-v017-5-6).
- **Retenu** : (1) issue demandée (I1, P3) — le passage en `spawn_blocking` change la propagation d'une panique et le
  partage du verrou pdfium : pas mécanique ; (2) issue demandée (I2, P3, test) ; la réparation du test cassé, elle, est
  mécanique et intégrée (C-v017-5-6).
- **Réversible** : oui.
