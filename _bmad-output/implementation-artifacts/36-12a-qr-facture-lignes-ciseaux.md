# Story 36-12a : QR-facture — les deux lignes de séparation, leurs ciseaux, et le point de dépôt aligné à droite (SIX § 3.7, #944)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus 5.5) par le découpage de la fiche `36-12-qr-facture-separation.md` (`split`), décidé
par l'orchestrateur à la remédiation de la passe 1 de validation (V1-7, `kesh-gate-logs/e36-12/validation-p1.md`), puis
reprise à la remédiation de la passe 2 (`kesh-gate-logs/e36-12/validation-36-12a-p2.md`, V2-1 : la ligne verticale
barrait le point de dépôt ; arbitrage de l'orchestrateur : la 36-12a doit rester **publiable seule**, elle aligne donc le
point de dépôt à droite, et rien d'autre de la grille). Worktree `kesh-36-12`, branche
`story/36-12-qr-facture-separation`, partie de `origin/epic/36-0.18.0` (`fed34a9c7`). Release **0.18.0**, phase 2.
**Aucune migration**. Code lu à `fed34a9c7`. Voie 2 (bases `kesh_e3612`, `kesh_e2e_e3612` ; `kesh_ci_e3612` en voie 1).
Registres : `e35-choix-autonomes-e350.md` (C-e350-1) ; `e36-choix-autonomes-36-12.md` (C-36-12-2, -4, -8, -10). La suite —
grille, *Payable by*, coupe des lignes — est la story **36-12b**, qui part de celle-ci fusionnée et réemploie sa table
des largeurs.*

## Story

En tant que titulaire qui envoie ses QR-factures en PDF, dans la langue de chacun de mes clients, je veux que le
récépissé et la partie paiement soient délimités par deux lignes portant chacune des ciseaux, comme l'exige la directive
SIX pour un PDF, sans qu'aucune de ces lignes ne barre un texte du bulletin, afin que le débiteur qui l'imprime sache où
couper. Les PDF déjà figés ne changent pas.

## Ce qui se lit au code (`fed34a9c7`)

Repère : origine en bas à gauche, mm ; la section de paiement occupe `y ∈ [0, 105]`.

- `crates/kesh-qrbill/src/pdf.rs:26-30` : `const PAGE_W: f32 = 210.0;`, `const RECEIPT_W: f32 = 62.0;`,
  `const SEP_Y: f32 = PAYMENT_H;` (105).
- `draw_separator`, `:728-745` : pointillé horizontal à `SEP_Y`, traits de 2 mm tous les 4 mm (`:733`
  `while x < PAGE_W {`, `:742` `x += 4.0;`), trait noir `:730`
  `layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));`, épaisseur `:731`
  `layer.set_outline_thickness(0.3);`. **Aucune ligne verticale** à x = 62, **aucun ciseau** : `:744`
  `// Scissors glyph is outside Helvetica's WinAnsi encoding; omitted in v0.1.` Appelée une fois, `:102`
  `draw_separator(&layer);`, **avant** le récépissé et la partie paiement (`:103-104`).
- **Le point de dépôt**, `:962-969` : `// "Acceptance point" at bottom-right of the receipt column.` puis
  `layer.use_text(i18n.get("invoice-pdf-qr-acceptance-point"), 6.0, Mm(RECEIPT_W - 15.0), Mm(5.0), helv_bold,` —
  **ancré à gauche à x = 47**, 6 pt gras, ligne de base 5. Libellés : `de-CH/messages.ftl:737` `Annahmestelle`,
  `en-CH:737` `Acceptance point`, `fr-CH:788` `Point de dépôt`, `it-CH:737` `Punto di accettazione` ; repli anglais
  `crates/kesh-qrbill/src/types.rs:344` `"Acceptance point",`. Chiffrés aux chasses de
  `/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/phvb8a.afm` (somme des `WX` × 6 × 25,4 / 72 / 1000, recalculée
  à la remédiation de la P2) :

  | Langue | Largeur | Bord droit aujourd'hui (depuis 47) | Contre une ligne à x = 62 | Départ aligné à 57 |
  |---|---|---|---|---|
  | fr « Point de dépôt » | 14,70 mm | 61,70 | à 0,3 mm | 42,30 |
  | de « Annahmestelle » | 15,06 mm | 62,06 | touche | 41,94 |
  | en « Acceptance point » | 17,64 mm | **64,64** | **traversé** | 39,36 |
  | it « Punto di accettazione » | 21,76 mm | **68,76** | **traversé** | 35,24 |

  Le texte occupe y ∈ [4,56 ; 6,52] ; un pointillé vertical à traits de 2 mm tous les 4 mm depuis y = 0 a un trait sur
  [4 ; 6] : il passerait **sur** les lettres (V2-1). La section de paiement est rendue « dans la **langue de
  correspondance du client** » (`docs/manual/fr/user-manual.tex:1282`).
- Ce qui borde les lignes : titre du récépissé 11 pt gras à x = 5, base 100 (`:901-909`), largeurs aux chasses AFM :
  « Récépissé » 19,20 mm, « Empfangsschein » 31,05, « Ricevuta » 16,17, « Receipt » 14,02 — **bord droit ≤ 36,05** ;
  titre de la partie paiement à x = 67 (`:986-995`) ; QR de x = 67 à 113, de y = 47 à 93 (`:999-1000` `let qr_top = top
  - 7.0;` `draw_qr_matrix(layer, qr, qr_x, qr_top, 46.0)?;`), bordure de 5 mm (§ 3.5.2) de x = 62 à 118, de y = 42 à
  **98** ; le corps de la facture ne descend pas sous une ligne de base de 110 (`:718` `Mm(ty.max(content_floor +
  5.0)),`). Boîte réelle d'un texte : `FontBBox -170 -228 1003 962` (AFM gras), soit de −0,228 à +0,962 em autour de la
  ligne de base (« Récépissé » monte à 102,91 par `eacute`, `B 23 -14 528 750` ; par la `FontBBox`, à 103,73).
- `chaque_texte_s_imprime_en_noir` (`:2023-2077`) fixe cinq `rg` dans l'ordre `[0,0,0] [1,1,1] [0.85,0,0] [1,1,1]
  [0,0,0]` (`:2054-2065`) et **17 `Tj` après le dernier `re`** (`:2076` `assert_eq!(apres_la_croix, 17);`) : des ciseaux
  **au trait** (`RG`, `S`/`s`), sans `rg` ni `re`, le laissent intact ; le premier `rg` est posé par `draw_qr_matrix`
  (`:1094`).
- **Les opérateurs que printpdf écrit pour un `Line`** (`printpdf-0.7.0/src/line.rs:2-4`, `:72`, `:78`, `:111`) : `m`, `l`,
  `c`, et `v` / `y` quand un point de contrôle coïncide avec une extrémité ; fermeture par `s` si `is_closed`, sinon `S`.
  Un `Line` = un chemin ; un symbole fait de plusieurs `Line` = plusieurs chemins.
- Helpers de test existants : `operations_du_pdf` (`:1542`), `nombre` (`:1558`), `textes_du_pdf` (`:1521`).

**La directive** (`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 3.7, p. 27) : « If the QR-bill with payment part and
receipt … are generated as a PDF document and sent electronically, the A6 format of the payment part and the receipt on
the left must be indicated by lines. Each of these lines must bear the scissors symbol ✂ or alternatively the
instruction "Separate before paying in" above the line (outside the payment part). » Elle ne place pas les ciseaux sur
la ligne. § 3.3 : partie paiement 148 × 105 mm, récépissé 62 × 105 à sa gauche. § 3.6.4 : « Acceptance point » « printed
right-aligned ». Échantillons SIX mesurés (`kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`) : pointillé vertical
à x = 61,7–61,9 ; récépissé jusqu'à 56,8.

**Pourquoi pas un glyphe** : Helvetica intégrée ne porte pas ✂ (`:744`) ; printpdf pose `WinAnsiEncoding` sur toute
police intégrée, ZapfDingbats comprise (`printpdf-0.7.0/src/font.rs:76`). La clé
`invoice-pdf-qr-separate-before-paying` existe dans les quatre langues (`types.rs:270`, `fr-CH/messages.ftl:789`),
employée nulle part ; la story ne l'emploie pas (D2).

**Les métriques**, pour aligner à droite : AFM d'Adobe (1990), `phvr8a.afm` et `phvb8a.afm` ; **pas de glyphe `Euro`**
(`/usr/bin/grep -ic euro` : 0 et 0) ; la colonne `C` est l'*AdobeStandardEncoding* (`C 39 ; WX 222 ; N quoteright`,
`C 169 ; WX 191 ; N quotesingle`, gras 238 ; `C 193 ; WX 333 ; N grave`), alors que printpdf écrit en WinAnsi, par la
table `WIN_ANSI_ENCODING` de `lopdf-0.31.0/src/encodings/mappings.rs:521` (`Some(Glyph::Euro)` à `:650`).

### Ce qui est figé, ce qui se régénère

`crates/kesh-api/src/routes/issued_invoice_pdf.rs:9-11` : une facture **figée** est relue sous `KESH_DOCUMENTS_DIR`,
son SHA-256 vérifié, ses octets rendus ; `:16` « Un PDF figé introuvable n'est **jamais** régénéré en silence ». Prennent le
nouveau rendu : une facture validée jamais ouverte (`render_and_pose`, `:141-`), chaque rappel (`invoice_email.rs:566-572`,
`:1317-1323`, `PdfDocument::Reminder`), un refigeage d'administrateur (`refreeze`, `:238-`). L'avoir n'a pas de section de
paiement (`pdf.rs:112` « **SANS section QR Bill** »). Aucun de ces chemins ne change.

### Inventaire par le symptôme

Symptôme : « une QR-facture en PDF est produite » (parti du produit, non des fonctions de dessin — V1-5).
`git grep -n "generate_qr_bill_pdf\|draw_separator(" -- crates` (à relancer à l'étape 0) rend, hors `kesh-qrbill` et
ses tests : `crates/kesh-api/src/routes/invoice_pdf_service.rs:244` (le rendu de production) et
`crates/kesh-api/src/qr_decode.rs:900` (`kesh_qrbill::generate_qr_bill_pdf_with_date(`, le « témoin vectoriel » du test
`t4_pdfium_scans_synthetiques`, `:975`). **Sonde positive** : `pdf.rs:102` `draw_separator(&layer);`, site du défaut.
Second symptôme, ouvert par la ligne verticale (V2-1) : « un texte du récépissé dépasse x = 57 vers la ligne ». Lecture
des `use_text` de `draw_receipt` (`:893-973`) : tous à x = 5 sauf le montant (`x + 12.0`, une valeur courte) et le point
de dépôt (`:966` `Mm(RECEIPT_W - 15.0),`). Les lignes d'adresse à x = 5 peuvent dépasser 57 pour des capitales très
larges (doc-comment `:803-804` : « 27 `W` occupent 72 mm depuis x = 5, jusqu'à x = 77 ») — c'est la coupe des lignes,
**36-12b** ; limite connue écrite aux Dev Notes.

| Site | Verdict |
|---|---|
| `kesh-qrbill/src/pdf.rs:728-745` (`draw_separator`) | **dans le périmètre** |
| `kesh-qrbill/src/pdf.rs:962-969` (point de dépôt) | **dans le périmètre** : aligné à droite à 57, ligne de base inchangée |
| `kesh-qrbill/src/pdf.rs:795-810`, `:820-847` (coupe à 27 / 46 caractères) | hors : **36-12b** ; limite connue |
| `kesh-api/src/routes/invoice_pdf_service.rs:244` | inchangé ; prend le nouveau rendu |
| `kesh-api/src/qr_decode.rs:900`, test `#[ignore]` (« nécessite libpdfium.so », `:921`) | inchangé ; **à rejouer** au passage en revue : le lecteur de QR scannés lit exactement un PDF émis par Kesh, et la ligne verticale passe à 5 mm du QR. Commande documentée `:913-914` : `LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib cargo test -p kesh-api --lib qr_decode::tests::t4_pdfium -- --ignored --nocapture` |
| `kesh-qrbill/src/pdf.rs:129` (avoir), `kesh-api/src/qr_decode_synthetique.rs:462`, `:524` (générateur de test d'un scan, `:1`), `kesh-report/src/pdf.rs:317` (rapports) | hors : aucune section de paiement |

## Décisions

- **D1 — Deux lignes pointillées** : l'horizontale d'aujourd'hui inchangée (y = 105, x de 0 à 210, traits de 2 mm tous les
  4 mm, 0,3 pt) et une **verticale** à x = 62, de y = 0 à y = 105, même motif. *Écartée* : un trait plein — les
  échantillons SIX sont en pointillé, et c'est le motif déjà imprimé.
- **D2 — Des ciseaux dessinés au trait, sur chaque ligne** (C-36-12-2) : un symbole vectoriel (deux anneaux en Bézier,
  deux lames), **au trait seul** (`RG` noir, chemins `S`/`s` ; aucun `rg`, aucun `re`, aucun remplissage), dessiné par une
  fonction `dessiner_ciseaux` appelée de `draw_separator`. Dimensions : **grande dimension de 3 à 5 mm, petite de 3,5 mm
  au plus**. Horizontal : centré sur y = 105 (± 0,5), **centre x ∈ [45, 52]** — à droite des titres du récépissé (bord
  droit ≤ 36,05 dans les quatre langues), pour que sa boîte ne rencontre pas leur boîte réelle, qui monte à 103,73.
  Vertical : tourné d'un quart de tour, centré sur x = 62 (± 0,5), **centre y ∈ [101 ; 102,5]**, de sorte que sa boîte
  reste dans y ∈ ]98 ; 105] — au-dessus de la bordure de 5 mm du QR, qui monte à 98 tant que le QR est à 93 (V1-6 ; la
  36-12b garde la contrainte). Le pointillé s'interrompt sous chaque symbole, et seulement là. *Écartées* : un ciseau
  horizontal près du bord gauche (x ∈ [5, 15]), sur les titres (V2-4) ; la mention « A détacher avant le versement » (ne
  porte pas la ligne verticale) ; le glyphe de ZapfDingbats (encodage imposé) ; une police embarquée (#943) ; des ciseaux
  remplis (un `rg` de plus romprait `chaque_texte_s_imprime_en_noir` — V1-10).
- **D3 — Le point de dépôt aligné à droite** (V2-1, arbitrage de l'orchestrateur ; § 3.6.4) : son intitulé finit au bord
  **x = 57** de la directive (62 − 5), mesuré par la table de D4, dans toutes les langues ; sa ligne de base reste à 5.
  *Écartées* : (a) lier la publication de la 36-12a à celle de la 36-12b ; (b) une ancre fixe à 57 − 21,76 (le libellé
  le plus large d'aujourd'hui), qui casse dès qu'une traduction s'allonge ; (c) le déplacer aussi en hauteur (grille,
  36-12b).
- **D4 — Une table des largeurs, pour mesurer** (C-36-12-4, C-36-12-10) : module neuf `crates/kesh-qrbill/src/largeurs.rs`,
  deux tables (Helvetica, Helvetica-Bold) **indexées par nom de glyphe WinAnsi** : pour chaque octet 0x20–0xFF de
  `WIN_ANSI_ENCODING` (`mappings.rs:521`), le nom du glyphe, puis sa chasse `WX` lue à l'AFM par ce nom ; **`Euro` ajouté
  à la main, 556 en maigre et en gras** (AFM Adobe actuel d'Helvetica ; absent de celui de 1990 — provenance écrite au
  doc-comment). `largeur_mm(texte, taille, gras)` part du caractère Unicode, passe par son octet WinAnsi puis par le nom ;
  un caractère sans entrée compte pour la plus large de la table. Script de génération versionné dans
  `kesh-gate-logs/e36-12/`, commande et empreinte des AFM au Dev Agent Record ; doc-comment : la table change avec la
  police (#943). La 36-12a ne s'en sert que pour le point de dépôt et les tests ; la 36-12b, pour couper les lignes.
  *Écartée* : une table du seul gras (même script ; la 36-12b aurait à le refaire).
- **D5 — Rien d'autre ne bouge** : récépissé (hors point de dépôt), partie paiement, QR et corps de la facture dessinés
  comme à `fed34a9c7`.
- **D6 — Le figé ne bouge pas** : aucune ligne de `crates/kesh-api`, `crates/kesh-db` ni `crates/kesh-i18n` ne change.
- **D7 — Aucune migration.**

**Définitions des tests** (V2-2 à V2-4). *Segment de pointillé* : un chemin fait d'un `m` et d'un seul `l`, de longueur
≤ 2,05 mm, sur y = 105 ou x = 62. *Symbole de ciseaux* : l'union des boîtes englobantes des autres chemins tracés
(opérateurs `m`, `l`, `c`, `v`, `y`, jusqu'à `S` ou `s`), hors `re` ; ceux dont le centre de boîte est à x < 57 forment le
ciseau horizontal, ceux dont le centre est à x ∈ [57, 67] le ciseau vertical. *Boîte d'un texte* : de `Td.x` à `Td.x +
largeur_mm(texte)` + 0,5 mm, de la ligne de base − 0,228 × taille × 0,35278 à la ligne de base + 0,962 × taille ×
0,35278 (`FontBBox`). *Disjointes* : deux boîtes fermées d'intersection vide (inégalités strictes).

## Critères d'acceptation

1. **Ligne horizontale** : des segments de pointillé sur y = 105 couvrent x de 0 à 210 ; tout trou de plus de 2,5 mm est
   contenu dans la boîte du ciseau horizontal élargie de 0,5 mm ; **aucun segment ne rencontre cette boîte**.
2. **Ligne verticale** : des segments sur x = 62 couvrent y de 0 à 105, même règle avec le ciseau vertical.
3. **Ciseaux** : un symbole horizontal, grande dimension de 3 à 5 mm, petite ≤ 3,5 mm, centré sur y = 105 (± 0,5), centre
   x ∈ [45, 52] ; un symbole vertical, mêmes dimensions, centré sur x = 62 (± 0,5), centre y ∈ [101 ; 102,5].
4. **Les ciseaux ne touchent rien** : chaque symbole est disjoint de la boîte de tout texte et du rectangle du QR élargi
   de 5 mm (x ∈ [62, 118], y ∈ [42, 98]).
5. **Le point de dépôt** : dans **les quatre langues** (libellés des quatre `messages.ftl` posés dans `QrBillI18n`) et au
   repli anglais, l'intitulé a son `Td.x` égal à 42,30 (fr), 41,94 (de), 39,36 (en), 35,24 (it) à 0,05 mm près — valeurs
   calculées depuis l'AFM et écrites en dur dans le test, indépendantes de la table —, donc son bord droit à 57, soit
   **une marge d'au moins 4,5 mm à gauche de la ligne verticale** ; sa ligne de base reste 5. Rouge à `fed34a9c7` : `Td.x`
   vaut 47 dans les quatre langues, et la marge est négative en **anglais et en italien** (bords à 64,64 et 68,76).
6. **La ligne verticale ne traverse aucun texte** : aucune boîte de texte du récépissé ne rencontre un segment sur x = 62,
   pour les fixtures du jeu (ordinaire de type S, rue longue, sans référence), **en français et en anglais**. Une adresse
   de capitales très larges est hors de ce critère, comme limite connue (Dev Notes).
7. **Au trait, en noir** : lignes et ciseaux en `RG` 0 0 0 ; `chaque_texte_s_imprime_en_noir` passe **sans modification**.
8. **La table** : témoins recopiés de l'AFM — maigre : espace 278, `A` 667, `W` 944, `@` 1015, `é` 556, `'` **191**
   (`quotesingle`), `` ` `` **333** (`grave`) ; gras : `A` 722, `W` 944, `'` **238** ; `€` **556** dans les deux (ajout
   écrit) — et toute **sortie** de `texte_imprime` sur le jeu admis (`est_caractere_qr`, `IJ` compris) a son entrée.
9. **Le reste inchangé** : tous les tests existants de `kesh-qrbill` passent sans modification (décompte des `#[test]` aux
   deux bornes, `git show <ref>:<fichier> | /usr/bin/grep -c '#\[test\]'`) ; le diff de production se limite à
   `draw_separator`, `dessiner_ciseaux`, leurs constantes, l'appel du point de dépôt dans `draw_receipt`, le doc-comment du
   module (`:1-9`), le module `largeurs.rs` et sa déclaration dans `lib.rs` (`git diff origin/epic/36-0.18.0 --
   crates/kesh-qrbill/src` lu et cité au Dev Agent Record).
10. **Le figé** : `git diff origin/epic/36-0.18.0 --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide ;
    `invoice_frozen_pdf_e2e.rs` et `invoice_pdf_e2e.rs` passent sans modification.
11. **Le lecteur de scans** : `t4_pdfium_scans_synthetiques` rejoué (`qr_decode.rs:913-914`), vert, journal cité ; dit non
    lancé si `libpdfium` manque.
12. **Manuel** : § *Génération du PDF avec QR Bill* dit les deux lignes à ciseaux et le point de dépôt aligné à droite
    (`git grep` du `.tex`).
13. **Captures** avant / après de la section de paiement, fixture ordinaire, **en français et en anglais**, à côté de
    l'échantillon SIX Nr. 1, dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12/captures/` ; ciseaux reconnaissables et
    point de dépôt dégagé de la ligne, constatés à l'œil et écrits au Dev Agent Record.

## Tests à écrire, et la preuve du rouge

**Ordre** : la table d'abord (`largeurs.rs`, ses témoins, sa couverture), **commitée seule** — elle ne change aucun rendu ;
son rouge se prouve par mutation (M8). Puis les tests de comportement dans `crates/kesh-qrbill/src/pdf.rs` (`mod tests`),
qui compilent alors et se lancent sur le rendu de `fed34a9c7` : journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-12/rouges-12a.log` (nom et message d'échec de chacun). Helpers neufs :
`traits_du_pdf` (segments `m`/`l` avec la couleur `RG` en vigueur), `chemins_du_pdf` (boîtes des chemins, opérateurs `m`,
`l`, `c`, `v`, `y`, jusqu'à `S`/`s`, hors `re`), `boite_texte` (selon les définitions).

| Test | AC | Rouge attendu à `fed34a9c7` |
|---|---|---|
| `table_des_largeurs_temoins`, `table_couvre_les_sorties_imprimees` | 8 | par mutation (M8) |
| `la_ligne_horizontale_couvre_la_largeur` | 1 | rouge : le test exige l'interruption sous le ciseau, absent |
| `une_ligne_verticale_separe_le_recepisse` | 2 | aucun segment sur x = 62 |
| `des_ciseaux_sur_chaque_ligne` | 3 | aucun symbole |
| `les_ciseaux_ne_touchent_rien` | 4 | rouge : le test exige les deux symboles avant de comparer |
| `le_point_de_depot_est_aligne_a_droite` (quatre langues) | 5 | `Td.x` = 47 partout ; marge négative en anglais et en italien |
| `la_ligne_verticale_ne_traverse_aucun_texte` (fr, en) | 6 | rouge : le test exige des segments sur x = 62 ; après le correctif, son rouge par M7 en anglais |
| `lignes_et_ciseaux_au_trait_noir` | 7 | rouge : aucun ciseau |

**Mutations** (après le commit du code, E5 ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12/mut/`, restauration
`git restore --worktree -- <fichier>` puis `touch`) :
- **M1** ligne verticale non dessinée → `une_ligne_verticale_separe_le_recepisse` ;
- **M2** ciseaux non dessinés → `des_ciseaux_sur_chaque_ligne` ;
- **M3** pointillé horizontal arrêté à x = 150 → `la_ligne_horizontale_couvre_la_largeur` ;
- **M4** ciseau vertical centré à y = 92 → `les_ciseaux_ne_touchent_rien`, `des_ciseaux_sur_chaque_ligne` ;
- **M5** ciseaux remplis (`set_fill_color` + remplissage) → `chaque_texte_s_imprime_en_noir` ;
- **M6** ciseau horizontal centré à x = 10 → `des_ciseaux_sur_chaque_ligne`, `les_ciseaux_ne_touchent_rien` (V2-5) ;
- **M7** point de dépôt remis à `Mm(RECEIPT_W - 15.0)` (ancré à gauche) → `le_point_de_depot_est_aligne_a_droite` dans les
  quatre langues, `la_ligne_verticale_ne_traverse_aucun_texte` en anglais ;
- **M8** `W` à 900 dans la table maigre → `table_des_largeurs_temoins` ;
- **M9** pointillé non interrompu sous le ciseau horizontal → `la_ligne_horizontale_couvre_la_largeur` (V2-5).

Une mutation équivalente se dit.

## Tâches

0. **Étape 0** : relire les lignes citées ; rejouer l'inventaire ; `ls -l` des AFM, de `mappings.rs`, de `line.rs`.
1. **Capture « avant »** : ajouter `crates/kesh-qrbill/examples/capture_qr_facture.rs` (PDF de fixtures synthétiques —
   ordinaire de type S, rue longue, sans référence —, **en français et en anglais**, dans le répertoire donné en argument ;
   la 36-12b y ajoute les siennes), le committer seul, le lancer (`scripts/mem-guard.sh cargo run -p kesh-qrbill
   --example capture_qr_facture -- <dir>`), rendre en PNG (`mutool draw -r 150 -o <png> <pdf>`), `captures/avant/`.
2. **Table** : script, `largeurs.rs`, témoins, couverture, `pub(crate)` pour `est_caractere_qr` si le test l'exige ;
   **commit seul** ; M8.
3. Tests de comportement rouges, commit, `rouges-12a.log`.
4. `draw_separator` (ligne verticale, interruption sous les ciseaux), `dessiner_ciseaux(layer, cx, cy, vertical)`, point
   de dépôt aligné à droite par `largeur_mm` ; constantes nommées et documentées (source de chaque cote) ; le commentaire
   `:744` disparaît ; doc-comment du module (`:1-9`) mis à jour.
5. Mutations M1 à M9.
6. Capture « après » (fr, en), `captures/apres/`, comparaison avec l'échantillon SIX Nr. 1.
7. Manuel (`docs/manual/fr/user-manual.tex`, § *Génération du PDF avec QR Bill*, `:1273-`) : les deux lignes à ciseaux,
   le point de dépôt aligné à droite ; à `:1302`, la 0.18.0 parmi ce qu'un PDF figé ne prend pas. PDF régénéré à
   l'intégration.
8. Gates, Dev Agent Record, `Status: review`, prompt de revue P1.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; assemblée par l'orchestrateur dans `## [0.18.0] — Non publié`) :

- QR-facture : le récépissé et la partie paiement sont délimités par deux lignes pointillées portant chacune des
  ciseaux, comme la directive SIX l'exige pour une QR-facture en PDF ; la ligne verticale entre le récépissé et la partie
  paiement manquait. L'intitulé du point de dépôt, au bas du récépissé, est désormais aligné à droite, comme la directive
  le demande, et ne touche plus cette ligne, quelle que soit la langue du client. Le code QR et le reste du bulletin ne
  changent pas. **Les PDF déjà émis et figés ne changent pas** ; une facture validée jamais ouverte, un rappel et un PDF
  refigé par un administrateur prennent le nouveau rendu. (refs #944, refs #12)

## Dev Notes

### Hors périmètre, et limite connue

- **Tout le reste de #944** (grille, *Payable by*, coupe des lignes à la largeur, message, cascade) : **36-12b**.
- **Limite connue jusqu'à la 36-12b** : une ligne d'adresse du récépissé faite de capitales très larges peut dépasser
  x = 57, et même 62 (doc-comment `pdf.rs:803-804`) ; la ligne verticale la traverse alors. Préexistant pour le
  débordement (qui atteint déjà le QR, et que le manuel dit à `:1300` : « une ligne faite de lettres très larges peut
  encore atteindre le code QR au reçu »), neuf pour la ligne. Une adresse ordinaire reste en deçà de 57 (AC6). À écrire
  au Dev Agent Record.
- **#12** (CR-005, « Symbole ciseaux ✂ sur la ligne de séparation ») : la story en réalise la puce « Ajouter le glyph ✂ »
  (par un tracé, que § 3.7 permet : « scissors symbol ») ; ses autres puces (police embarquée, `é è ü €`, golden test)
  sont #943 (`docs/change_request.md:56-60`). **`refs #12`** à la PR, avec un commentaire qui dit ce qui est fait et
  renvoie à #943 ; pas `closes` (V1-9).
- Cadre vide du débiteur : **#1203** ; type K : **#1204**.

### Splitting — deux modules

`crates/kesh-qrbill` (`src/pdf.rs`, `src/largeurs.rs` neuf, `src/lib.rs`, `examples/` neuf) et le manuel utilisateur.

### Fichiers communs

| Fichier | Autres stories | Règle |
|---|---|---|
| `crates/kesh-qrbill/src/pdf.rs`, `largeurs.rs`, `examples/` | **36-12b** | en série : la 36-12b part de la 36-12a fusionnée |
| `docs/manual/fr/user-manual.tex` | 36-12b, 36-15, 36-16, 36-18 (possibles) | paragraphes distincts ; PDF régénéré après l'union |
| `CHANGELOG.md` | toutes | non touché (C-e360-11) |

### Ce qui doit rester vrai

Le code QR, octet pour octet ; toutes les positions de texte hors le point de dépôt ; le tout-noir des textes
(`:2023`) ; les PDF figés.

### Gates (`e35-dev-commun.md` § *Quels gates, quand*, voie 2)

En cours : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p
kesh-qrbill` (sous `scripts/mem-guard.sh`, sans base). Passage en `review` : bases `kesh_e3612` remises à neuf sur
`kesh-mariadb-dev2:3307`, `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock scripts/mem-guard.sh
scripts/test-fast.sh --ci` (`TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3612`),
puis `scripts/gate-ci.sh kesh_ci_e3612` en voie 1, et le test `t4_pdfium` (AC11). Remédiation : `test-fast --ci`. Clôture :
`gate-ci` et E2E complet sur `kesh_e2e_e3612`. Une commande lourde à la fois.

### E2E

Aucun E2E neuf : le défaut est dans le dessin du document, que Playwright ne mesure pas ; la route qui sert le PDF ne
change pas (AC10). Suite complète à la clôture.

### Références

Directive § 3.3, § 3.5.2, § 3.6.4, § 3.7 ; échantillons `docs/six-references/samples-payment-part-en/muster-qr-zahlteile-en/` ;
AFM `phvr8a.afm`, `phvb8a.afm` ; `lopdf-0.31.0/src/encodings/mappings.rs:521` ; `printpdf-0.7.0/src/line.rs` ; fiche
d'origine `36-12-qr-facture-separation.md` (`split`) ; validations `kesh-gate-logs/e36-12/validation-p1.md`,
`validation-36-12a-p2.md`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche écrite au découpage de la 36-12 (décision de l'orchestrateur, V1-7 de la validation P1), sur
  `fed34a9c7`. Findings de la P1 portés ici : V1-5 (témoin `qr_decode.rs:900`, AC8), V1-6 (ciseaux verticaux hors de la
  bordure du QR, D2, AC3-AC4), V1-9 (`refs #12`), V1-10 (ciseaux au trait, AC5, M5), V1-8 (#1203, #1204). Choix
  C-36-12-8. Validation de cette fiche non lancée.
- 2026-11-15 — **Validation P2** (complète, Opus 5.5, `kesh-gate-logs/e36-12/validation-36-12a-p2.md`) : 0 CRITICAL /
  0 HIGH / 1 MEDIUM / 6 LOW. Prémisses relues, toutes confirmées : chasses du point de dépôt recalculées à l'AFM gras (fr
  14,70, de 15,06, en 17,64, it 21,76 mm ; bords 61,70 / 62,06 / 64,64 / 68,76 depuis x = 47) ; `FontBBox -170 -228 1003
  962` ; `printpdf-0.7.0/src/line.rs:72`, `:78` (`v`, `y`), `:111` (`s`). Remédiation :
  - **V2-1** (MEDIUM, arbitrage de l'orchestrateur : la 36-12a publiable seule) : point de dépôt aligné à droite à 57 (D3,
    AC5, AC6, M7) ; la table des largeurs passe de la 36-12b à la 36-12a, pour mesurer l'intitulé (D4, AC8, M8,
    C-36-12-10) ; fixtures et captures en français et en anglais ; note de version qui dit ce qui change visiblement ;
    limite connue écrite (Dev Notes).
  - **V2-2** : opérateurs `m`, `l`, `c`, `v`, `y`, `S`/`s` (définitions, `chemins_du_pdf`).
  - **V2-3** : symbole = union des chemins hors pointillé et hors `re`, réparti par le centre de boîte.
  - **V2-4** : boîte de texte par la `FontBBox` et la largeur mesurée ; dimensions des ciseaux bornées (3 à 5 mm, petite
    ≤ 3,5) ; « disjointes » défini (strict) ; ciseau horizontal déplacé à x ∈ [45, 52], hors des titres ; vertical à
    y ∈ [101 ; 102,5].
  - **V2-5** : trou du pointillé contenu dans la boîte du symbole élargie de 0,5 mm, aucun segment dans la boîte ; M6 et M9.
  - **V2-6** : AC9 aligné sur la tâche 4.
  - **V2-7** : fichier d'epic, à la charge de l'orchestrateur.
  Passe de validation suivante due.
