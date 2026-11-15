# Story 36-12 : la QR-facture conforme à SIX — lignes de séparation, ciseaux, *Payable by*, largeurs (#944)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-12`, branche
`story/36-12-qr-facture-separation`, partie de `origin/epic/36-0.18.0` (`fed34a9c7`), par le workflow
`bmad-create-story`, prompts `kesh-gate-logs/prompts/e36-fiche.md` et `e36-p2-commun.md`. Release **0.18.0**, phase 2.
**Aucune migration** ; aucune ligne de `kesh-api` ni de `kesh-db` ne change. Code lu à la tête du worktree (`fed34a9c7`).
Voie de développement : **voie 2** (bases `kesh_e3612`, `kesh_e2e_e3612` ; `kesh_ci_e3612` en voie 1). Registres :
`e35-choix-autonomes-e350.md` (C-e350-1, qui fait de #944 une story à elle seule) ; `e36-choix-autonomes-36-12.md`
(C-36-12-1 à C-36-12-7). Validation non lancée.*

## Story

En tant que titulaire qui envoie ses QR-factures en PDF — Guy en a déjà émis de réelles —, je veux que la section de
paiement suive la directive SIX : la partie paiement et le récépissé délimités par des lignes portant des ciseaux,
*Payable by* à sa place dans la section d'information, chaque texte dans sa zone, sans recouvrir le code QR ni sortir de
la feuille, afin qu'un débiteur qui l'imprime sache où couper et qu'aucune banque ni aucun guichet n'ait de motif de
refuser le bulletin. Les PDF déjà émis et figés ne changent pas.

## Ce qui se lit au code (`fed34a9c7`)

### Le rendu de la section de paiement — `crates/kesh-qrbill/src/pdf.rs` (3018 lignes)

Repère : printpdf place l'origine en bas à gauche ; les ordonnées ci-dessous sont en mm depuis le bas de la page A4, la
section de paiement occupe `y ∈ [0, 105]`. Constantes `:26-30` : `const PAGE_W: f32 = 210.0;`, `const PAYMENT_H: f32 =
105.0;`, `const RECEIPT_W: f32 = 62.0;`, `const SEP_Y: f32 = PAYMENT_H;`. Appel `:101-104` :
`draw_invoice_section(…)`, `draw_separator(&layer);`, `draw_receipt(…)`, `draw_payment_part(…)`.

**La ligne de séparation** — `draw_separator`, `:728-745` : un pointillé horizontal à `SEP_Y`, traits de 2 mm tous les
4 mm (`:733` `while x < PAGE_W {`, `:742` `x += 4.0;`), épaisseur `:731` `layer.set_outline_thickness(0.3);`. **Aucune
ligne verticale** entre le récépissé et la partie paiement, **aucun ciseau** : `:744`
`// Scissors glyph is outside Helvetica's WinAnsi encoding; omitted in v0.1.`

**Le récépissé** — `draw_receipt`, `:893-973` : `:901` `let x = 5.0;`, `:902` `let top = SEP_Y - 5.0;` ; titre 11 pt
gras à la ligne de base `top` = 100, donc le haut des capitales à 100 + 0,718 × 11 × 0,3528 ≈ 102,8 — **2,2 mm du bord**
pour une marge de 5 mm ; `:912` `let mut y = top - 7.0;` (intitulé *Account / Payable to*), IBAN, créancier, référence
(`dessiner_reference`, `:849-871`) ; **puis le montant**, `:926` `y -= 8.0;` — il **suit** le contenu ; **puis
*Payable by***, `:949-950` `// Debtor.` `y -= 8.0;`, et le débiteur sous le montant. Le point de dépôt `:963-969` :
`Mm(RECEIPT_W - 15.0),` `Mm(5.0),` — ancré **à gauche** à x = 47, ligne de base 5, sans alignement à droite.

**La partie paiement** — `draw_payment_part`, `:977-1083` : `:986` `let x0 = RECEIPT_W + 5.0;` (67), titre à la ligne
de base 100 (même défaut de marge) ; QR `:999-1000` `let qr_top = top - 7.0;` `draw_qr_matrix(layer, qr, qr_x, qr_top,
46.0)?;` — 46 mm de x = 67 à 113, de y = 93 à 47 ; colonne de texte `:1004` `let tx = qr_x + 50.0;` — **x = 117, à
4 mm du QR** pour une bordure de 5 mm ; message `:1041` `layer.use_text(msg, 8.0, Mm(tx), Mm(y), helv);` — **une ligne,
sans borne de largeur** ; montant `:1045` `let amount_y = qr_top - 52.0;` — intitulé à y = 41, son haut à 42,5, **4,5 mm
sous le QR** ; débiteur `:1068-1070` `// Debtor block (bottom-left).` `let mut dy = ay - 8.0;` — ***Payable by* sous le
montant, dans la colonne du QR**, et non dans la section d'information.

**La coupe des lignes** — `:795-810` : `const MAX_CARACTERES_RECU: usize = 27;`, `const MAX_CARACTERES_PAIEMENT: usize
= 46;`, calés sur la largeur **moyenne** des capitales ; le doc-comment `:803-804` écrit la limite : « 27 `W` occupent
72 mm depuis x = 5, jusqu'à x = 77, dix millimètres sur le QR. Limite écrite, à #944. » `couper_ligne` (`:820-847`)
compte des caractères ; `dessiner_adresse` (`:875-889`) dessine à pas fixe `ADRESSE_PAS` = 3,5 (`:750`), sans borne
basse.

**Les tests qui figent la mise en page d'aujourd'hui** (`mod tests`, `:1428-`) : `textes_du_pdf` (`:1521`) relit chaque
`Tj` à la position du `Td` précédent ; `operations_du_pdf` (`:1542`) rend les opérations de contenu ; `texte_a`
(`:1577`). Ils posent des positions exactes que la story change : `constantes_de_coupe_recalculees` (`:1807`),
`quatre_blocs_impriment_la_composition` (`:1823`, `(117.0, 86.5, …)`, `(67.0, 26.0, "Pia Rutschmann")`),
`sans_reference_aucune_rubrique` (`:1851`, « le montant remonte d'autant »), `rue_longue_les_y_suivent` (`:1865`),
`le_pays_s_imprime_hors_de_suisse` (`:1910`), `cinq_cas_au_dessus_du_point_de_depot` (`:1931`),
`chaque_site_imprime_la_base_des_lettres` (`:1974`), et les `couper_ligne_*` (`:1707`, `:1726`, `:1757`, `:1786`,
`:1797`).

### Les données réelles : ce qui atteint le rendu

`crates/kesh-api/src/routes/invoice_pdf_service.rs` : créancier et débiteur toujours de type S (`:280`, `:300`
`address_type: AddressType::Structured,`) ; **un débiteur toujours présent** (`:384` `ultimate_debtor:
Some(debtor.clone()),`) ; un montant toujours présent ; le message `:388-392`, « préfixe + numéro de facture », borné à
140 caractères (`:391` `msg.chars().take(140).collect::<String>()`). Bornes de la validation
(`crates/kesh-qrbill/src/validation.rs:10-17`) : `NAME_MAX` 70, `STREET_MAX` 70, `BUILDING_MAX` 16, `POSTAL_CODE_MAX`
16, `TOWN_MAX` 35, `USTRD_MAX` 140 ; jeu admis `est_caractere_qr` (`:222-234`) : U+0020–007E, U+00A0–00FF,
U+0100–017F, U+0218–021B, €, U+2019, U+2014. Ce qui s'imprime passe par `texte_imprime`
(`crates/kesh-qrbill/src/texte.rs:46-55`), qui replie les lettres hors WinAnsi sur leur base.

### Ce qui est figé, ce qui se régénère

`crates/kesh-api/src/routes/issued_invoice_pdf.rs:1-19` : le PDF d'une facture est **figé au premier rendu après
validation** ; ensuite, « facture **figée** → relit le fichier sous `KESH_DOCUMENTS_DIR`, vérifie son SHA-256 contre
`pdf_sha256`, rend ces octets — y compris pour une facture annulée par un avoir après le gel » (`:9-11`), et « Un PDF
figé introuvable n'est **jamais** régénéré en silence » (`:16`). Donc, après la mise à jour :

| Document | Chemin | Rendu |
|---|---|---|
| Facture déjà téléchargée ou envoyée (figée) | `get_or_freeze` → `serve_frozen` (`issued_invoice_pdf.rs:95`, `:104-105`) | **inchangé**, octet pour octet |
| Facture figée restaurée d'un `.keshbackup` (format 2, PDF figés compris) | même chemin | **inchangé** |
| Facture validée **jamais ouverte** | `render_and_pose` (`:141-`) : rendu puis gel | **nouveau rendu**, figé à cette ouverture |
| Rappel (seul ou en lot) | `invoice_email.rs:566-572`, `:1317-1323`, `PdfDocument::Reminder(amounts)` : rendu à chaque envoi | **nouveau rendu** — le rappel d'une facture émise avant la mise à jour ne ressemble plus à la facture |
| Refigeage par un administrateur (fichier disparu) | `refreeze` (`:238-`), route `invoice_pdf.rs:74-88` | **nouveau rendu**, l'audit garde les deux empreintes |
| Avoir | `generate_credit_note_pdf` (`pdf.rs:116-`, `:112` « **SANS section QR Bill** »), sans section de paiement | inchangé (n'appelle aucune fonction touchée) |

Aucun de ces chemins n'est modifié par la story : seul le contenu de `generate_qr_bill_pdf` change.

### La directive — `docs/six-references/ig-qr-bill-v2.4-en.pdf`

- **§ 3.7** (p. 27) : « If the QR-bill … are generated as a PDF document and sent electronically, the A6 format of the
  payment part and the receipt on the left must be indicated by lines. Each of these lines must bear the scissors
  symbol ✂ or alternatively the instruction "Separate before paying in" above the line (outside the payment part). »
- **§ 3.3** : partie paiement A6 148 × 105 mm ; récépissé 62 × 105 mm à sa gauche ; ensemble 210 × 105.
- **§ 3.5** (fig. 4) : cinq sections de la partie paiement ; les espaces entre sections « must be at least 5 mm in
  height and width, and must not be printed ». **§ 3.5.2** : bordure de 5 mm autour du QR. **§ 6.4** : QR de 46 × 46 mm
  sans zone de silence.
- **§ 3.5.4** (tableau 4) : ordre de la section d'information — *Account / Payable to*, *Reference*, *Additional
  information*, *Payable by* ; le message raccourci « must be marked with an ellipsis "..." at the end ».
- **§ 3.6** (fig. 7) : récépissé — titre, information, montant, point de dépôt ; « The acceptance point section should
  have a height of at least 2 cm » ; marges de 5 mm, réductibles en faveur du point de dépôt. **§ 3.6.2** (tableau 5) :
  *Account / Payable to*, *Reference*, *Payable by* ; « Because of the limited space, it is permitted to: enter
  information in smaller or different font sizes … The minimum font size is 6 pt ; omit the street name and building
  number from the addresses ». **§ 3.6.4** : « Acceptance point » « printed right-aligned ».
- **§ 3.4** : titres 11 pt gras ; récépissé 6 pt (intitulés, gras) / 8 pt (valeurs) ; partie paiement 6 à 10 pt,
  intitulés deux points sous les valeurs.

Les figures n'ont pas de cotes ; le *Style Guide* de SIX n'est pas dans le dépôt. Les **échantillons** de SIX le sont
(`docs/six-references/samples-payment-part-en/muster-qr-zahlteile-en/Nr. 1 englisch.jpg`, 1242 × 621 px, 150 dpi =
210 × 105 mm). Mesurés par `/home/gcorbaz/devel/kesh-gate-logs/e36-12/mesure-echantillon-six.py`, journal
`mesure-echantillon-six-nr1.log` (Nr. 1, 2 et 13, concordants), en mm **depuis le haut** de la section :

```
recu x min 5.1 mm, x max 56.8 mm
  bandes y (mm depuis le haut): [(6.3, 9.8), (12.9, 14.9), …, (68.6, 70.7), (72.2, 74.1), (82.7, 84.7)]
paiement-gauche x min 67.0 mm, x max 111.9 mm
  bandes y (mm depuis le haut): [(5.9, 9.5), (18.3, 62.4), (68.6, 70.7), (72.2, 74.1)]
paiement-info x min 117.9 mm, x max 164.0 mm
  bandes y (mm depuis le haut): [(5.2, 7.3), (8.8, 10.7), …]
```

et une colonne de pointillé à x = 61,7–61,9. Lecture : ligne verticale à 62 ; récépissé de 5 à 57 ; colonne
d'information à 118 ; titres (haut) à 5,9–6,3 mm du haut ; montant (haut des intitulés) à 68,6 mm du haut dans les deux
colonnes ; point de dépôt (haut) à 82,7 mm, bord droit à 57 ; *Payable by* dans la section d'information des deux côtés
(échantillon Nr. 1 vu : au récépissé, sous la référence ; en partie paiement, sous *Additional information*).

### Les libellés — rien à ajouter

`crates/kesh-qrbill/src/types.rs:259-270` : les clés `invoice-pdf-qr-*`, dont `invoice-pdf-qr-separate-before-paying`
(`:270`), présente dans les quatre locales (`fr-CH/messages.ftl:789` `= A détacher avant le versement`) et **employée
nulle part** (`git grep -n "separate-before-paying" -- crates frontend` hors locales : seulement `types.rs:270`). La
story n'en ajoute aucune (C-36-12-2). Le point de dépôt dans les quatre langues : `Point de dépôt`, `Annahmestelle`,
`Punto di accettazione`, `Acceptance point` (`*/messages.ftl`, clé `invoice-pdf-qr-acceptance-point`).

### Inventaire par le symptôme

Symptôme : « la section de paiement d'une QR-facture est dessinée ». Détecteur par lecture, parti du dessin et des
libellés de la section, non d'une liste de fonctions :

`git grep -n "PdfDocument::new\|draw_separator(\|draw_receipt(\|draw_payment_part(\|invoice-pdf-qr-payable-by\|invoice-pdf-qr-acceptance-point" -- 'crates/*' | /usr/bin/grep -v "^crates/kesh-i18n"` :
```
crates/kesh-api/src/qr_decode_synthetique.rs:462:    let (doc, p1, l1) = PdfDocument::new(
crates/kesh-api/src/qr_decode_synthetique.rs:524:    let (doc, p1, l1) = PdfDocument::new(
crates/kesh-qrbill/src/pdf.rs:75:    let (doc, page_idx, layer_idx) = PdfDocument::new(
crates/kesh-qrbill/src/pdf.rs:102:    draw_separator(&layer);
crates/kesh-qrbill/src/pdf.rs:103:    draw_receipt(&layer, data, invoice, i18n, &helv, &helv_bold)?;
crates/kesh-qrbill/src/pdf.rs:104:    draw_payment_part(&layer, data, invoice, i18n, &helv, &helv_bold, &qr)?;
crates/kesh-qrbill/src/pdf.rs:129:    let (doc, page_idx, layer_idx) = PdfDocument::new(
crates/kesh-qrbill/src/pdf.rs:728:fn draw_separator(layer: &PdfLayerReference) {
crates/kesh-qrbill/src/pdf.rs:893:fn draw_receipt(
crates/kesh-qrbill/src/pdf.rs:952:        i18n.get("invoice-pdf-qr-payable-by"),
crates/kesh-qrbill/src/pdf.rs:964:        i18n.get("invoice-pdf-qr-acceptance-point"),
crates/kesh-qrbill/src/pdf.rs:977:fn draw_payment_part(
crates/kesh-qrbill/src/pdf.rs:1072:        i18n.get("invoice-pdf-qr-payable-by"),
crates/kesh-qrbill/src/types.rs:266:    "invoice-pdf-qr-payable-by",
crates/kesh-qrbill/src/types.rs:269:    "invoice-pdf-qr-acceptance-point",
crates/kesh-report/src/pdf.rs:317:            PdfDocument::new(title, Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), "Layer 1");
```

**Sonde positive** : le détecteur rend `draw_separator` (`pdf.rs:728`), site connu du défaut (ciseaux absents, `:744`).

| Site | Verdict |
|---|---|
| `kesh-qrbill/src/pdf.rs:75` et `:101-104` (QR-facture) | **dans le périmètre** — le seul rendu d'une section de paiement |
| `kesh-qrbill/src/pdf.rs:129` | hors : avoir, sans section de paiement (`:112` « **SANS section QR Bill** ») |
| `kesh-qrbill/src/types.rs:266`, `:269` | hors : liste des clés, inchangée |
| `kesh-api/src/qr_decode_synthetique.rs:462`, `:524` | hors : générateur **de test** d'un scan (`:1` « Scans synthétiques de QR-factures — générateur de test ») ; il pose un QR seul, sans section de paiement (`:4-7`), pour éprouver le lecteur |
| `kesh-report/src/pdf.rs:317` | hors : rapports comptables, sans QR |

Angle mort écrit : un rendu de section de paiement qui ne passerait ni par printpdf ni par ces libellés (aucun aujourd'hui).

## Décisions

- **D1 — Périmètre : sept points de #944** (C-36-12-1) : lignes de séparation, ciseaux, *Payable by*, largeurs des zones,
  largeur du message, coupe mesurée (le `W`), point de dépôt jamais recouvert. **Hors** : le cadre vide du débiteur
  (« Payable by (name/address) », 65 × 25 / 52 × 20 mm) et le type K — deux issues neuves. *Écartées* : les quatre points
  du fichier d'epic seuls ; les neuf.
- **D2 — Deux lignes pointillées** : l'horizontale d'aujourd'hui (y = 105, x de 0 à 210, même motif et même épaisseur)
  et une **verticale** à x = 62, de y = 0 à y = 105, même motif. *Écartée* : un trait plein — les échantillons de SIX
  sont en pointillé, et c'est le motif déjà imprimé.
- **D3 — Des ciseaux dessinés sur chaque ligne** (C-36-12-2) : un symbole vectoriel d'environ 5 × 3 mm (deux anneaux,
  deux lames croisées), noir, centré **sur** la ligne horizontale avec son centre à x ∈ [5, 15], et **sur** la ligne
  verticale, tourné d'un quart de tour, son centre à y ∈ [90, 100] ; le pointillé peut s'interrompre sous le symbole.
  *Écartées* : la mention « A détacher avant le versement » (ne porte pas la ligne verticale) ; le glyphe de ZapfDingbats
  (printpdf impose `WinAnsiEncoding`, `printpdf-0.7.0/src/font.rs:76`) ; une police embarquée (#943).
- **D4 — La grille SIX** (C-36-12-3), en mm depuis le bas de la page ; « haut » = ligne de base + 0,718 × taille ×
  0,35278 (`CapHeight` 718 des AFM), « bas » = ligne de base − 0,207 × taille × 0,35278 (`Descender` −207) :

  | Élément | Aujourd'hui (`fed34a9c7`) | Cible | Source |
  |---|---|---|---|
  | Titres *Receipt*, *Payment part* (11 pt gras) | base 100, haut 102,8 | haut ∈ [98, 100], x = 5 et x = 67 | marge 5 mm ; échantillons 5,9–6,3 du haut |
  | QR 46 × 46 | x 67→113, y 47→93 | x 67→113, **y 42→88** (haut à 17 mm du haut) | § 6.4 ; § 3.5.2 ; échantillon 17,8 |
  | Colonne d'information (partie paiement) | x = 117 | **x = 118**, toute ligne ≤ 205 | § 3.5.2 (5 mm) ; échantillons 117,9 |
  | Premier intitulé de cette colonne | base 93 | haut ∈ [98,8, 100] | échantillon 5,2 du haut |
  | Premier intitulé du récépissé | base 93 | haut ∈ [91,1, 93,1] | échantillon 12,9 du haut |
  | Texte du récépissé | x = 5, coupe à 27 caractères | toute ligne dans [5, 57] | § 3.3 (62 − 2 × 5) ; échantillon 5,1–56,8 |
  | Section montant | récépissé : suit le contenu ; paiement : intitulé base 41 (haut 42,5) | **même ligne de base dans les deux colonnes**, haut des intitulés ∈ [35,4, 37,0] (≥ 5 mm sous le QR) | § 3.5 ; échantillons 68,6 du haut (y = 36,4) |
  | *Payable by* | sous le montant, deux colonnes | **dans la section d'information** : récépissé après la référence, partie paiement après *Additional information* | § 3.5.4, § 3.6.2 ; échantillon Nr. 1 |
  | Fin de la section d'information du récépissé | aucune borne | bas de la dernière ligne ≥ haut des intitulés du montant + 5 | § 3.6, fig. 7 |
  | Point de dépôt (6 pt gras) | x = 47 à gauche, base 5 | **bord droit à 57** (± 0,3), haut ∈ [21,3, 23,3] | § 3.6.4 ; échantillons 82,7 du haut |
  | Bas de toute ligne de la section | non borné | ≥ 5 | marges de 5 mm |

  Les cotes du texte sont exactes dans les tests ; celles lues sur les échantillons, à ± 1 mm. *Écartées* : le *Style
  Guide* de SIX hors du dépôt ; les cotes d'une bibliothèque tierce.
- **D5 — Largeurs mesurées** (C-36-12-4) : un module neuf `crates/kesh-qrbill/src/largeurs.rs`, la table des chasses
  `WX` d'Helvetica et d'Helvetica-Bold (AFM d'Adobe, `phvr8a.afm` et `phvb8a.afm` de TeX Live), indexée par le caractère
  Unicode que WinAnsi porte, et `largeur_mm(texte, taille, gras) -> f32` ; un caractère absent compte pour la chasse la
  plus large de la table. `couper_ligne` coupe à une largeur en mm (récépissé 52, colonne d'information 87), toujours à
  une frontière de mot, un mot trop large coupé au dernier caractère qui tient. `MAX_CARACTERES_RECU` et
  `MAX_CARACTERES_PAIEMENT` disparaissent. Le doc-comment de la table dit d'où elle vient et qu'elle change avec la police
  (#943). *Écartées* : des bornes en caractères plus basses ; les métriques d'une police embarquée.
- **D6 — Le message** coupé en lignes dans les 87 mm de la colonne d'information, comme une adresse ; raccourci et
  terminé par « ... » **seulement** si la colonne ne tient pas (étape P1 de D7). En production il tient sur une ligne
  (préfixe + numéro de facture).
- **D7 — Une cascade pour ce qui ne tient pas** (C-36-12-5), étapes appliquées dans l'ordre, chacune seulement si la
  précédente ne suffit pas, à toute la colonne :
  - récépissé : **R0** tout à 8 pt ; **R1** rue et numéro omis des deux adresses (§ 3.6.2) ; **R2** valeurs à 6 pt, pas
    de ligne proportionnel (3,5 × taille / 8) (§ 3.6.2) ; **R3** noms ramenés à deux lignes, la seconde terminée par
    « ... » (§ 3.6.2) ;
  - colonne d'information : **P0** tout ; **P1** message ramené aux lignes qui tiennent (une au moins), « ... » en fin
    (§ 3.5.4) ; **P2** noms à deux lignes, « ... ».
  Une adresse ordinaire n'en déclenche aucune (la rue reste au récépissé). Si rien ne tient : l'erreur
  `QrBillError::PdfGeneration` existante, qu'aucune entrée admise par la validation n'atteint (AC8). La garde est une
  fonction pure (hauteur calculée avant de dessiner), testée sans PDF. *Écartées* : refuser le PDF au premier
  débordement ; omettre la rue toujours ; réduire les tailles d'office.
- **D8 — Les tailles restent 6 / 8 pt, titres 11 pt** (C-36-12-6) hors de R2. *Écartée* : 8 / 10 recommandé (§ 3.4).
- **D9 — La section montant fixe**, alignée dans les deux colonnes (C-36-12-7). Les montants restent où ils sont dans
  la section (intitulés *Currency* / *Amount*, décalage de 12 mm au récépissé et de 15 mm en partie paiement).
- **D10 — Le figé ne bouge pas, et la story ne touche que le rendu** : aucune ligne de `crates/kesh-api`,
  `crates/kesh-db` ni `crates/kesh-i18n` ne change. Les chemins figés relisent leurs octets ; ce qui se régénère est au
  tableau ci-dessus et s'écrit au manuel. *Écartée* : refiger les factures émises avec le nouveau rendu — ce serait
  fabriquer une pièce qui n'a pas été émise (`issued_invoice_pdf.rs:16-19`).
- **D11 — Le code QR ne change pas** : charge utile (`build_payload`) et matrice (`render_qr_image`) intouchées ; seule
  sa place (haut de 93 à 88) change.
- **D12 — Aucune migration.** Rien n'est écrit en base ; rendu seul.

## Critères d'acceptation

1. **Deux lignes** : le contenu du PDF porte des segments sur y = 105 qui couvrent x de 0 à 210 (aucun trou de plus de
   4 mm hors du symbole de ciseaux) et des segments sur x = 62 qui couvrent y de 0 à 105 (même règle), relevés dans les
   opérations `m` / `l` du PDF produit.
2. **Des ciseaux sur chaque ligne** : un tracé hors QR (opérations de chemin, ni `re` du QR ni pointillé) dont la boîte
   englobante, de 3 à 7 mm de long, est centrée sur y = 105 avec x ∈ [5, 15], et un second centré sur x = 62 avec
   y ∈ [90, 100] ; les deux en noir ; la capture le montre lisible comme des ciseaux.
3. **La grille de D4 est tenue**, mesurée dans le PDF produit (positions des `Td`, largeurs par la table) : titres,
   QR (x = 67, haut 88, 46 mm), colonne d'information à x = 118, premiers intitulés, section montant à la même ligne de
   base dans les deux colonnes et au moins 5 mm sous le QR, point de dépôt au bord droit 57 dans **les quatre langues**
   (libellés des quatre `messages.ftl` posés dans `QrBillI18n`).
4. ***Payable by*** : en partie paiement, l'intitulé et le débiteur sont à x = 118, sous *Additional information* (ou
   sous la référence sans message) ; au récépissé, à x = 5, entre la référence (ou le créancier sans référence) et la
   section montant ; plus rien n'est dessiné à x = 67 sous la section montant.
5. **Aucune ligne hors de sa zone**, pour chaque fixture du jeu (ordinaire, rue longue, étrangère, sans référence, sans
   message, pire cas) : toute ligne du récépissé tient dans [5, 57], toute ligne de la colonne d'information dans
   [118, 205], aucune ne touche le rectangle du QR élargi de 5 mm, aucun bas de ligne sous y = 5, aucune ligne de la
   section d'information du récépissé à moins de 5 mm de la section montant, et aucune n'atteint le point de dépôt.
6. **La coupe mesure** : 27 caractères les plus larges du jeu ne dépassent plus x = 57, 46 ne dépassent plus x = 205 ;
   une ligne ordinaire (« Robert Schneider SA », « Rue du Lac 1268 ») reste sur une ligne ; rien n'est perdu (les lignes
   coupées, rejointes, redonnent l'entrée aux espaces près), hors les étapes R1, R3, P1, P2.
7. **La cascade** : la fixture ordinaire n'en déclenche aucune étape (la rue s'imprime au récépissé) ; une fixture
   construite pour chaque étape la déclenche, et seulement elle ; un message de 140 caractères larges se termine par
   « ... » dans la colonne d'information quand elle ne tient pas, et s'imprime entier quand elle tient.
8. **Le pire cas admis se rend** : noms 70, rue 70, numéro 16, NPA 16, localité 35, pays étranger, pour le créancier et
   le débiteur, message 140, faits du caractère le plus large que `texte_imprime` puisse rendre (calculé dans le test
   sur `est_caractere_qr`, non écrit en dur) : `generate_qr_bill_pdf` rend un PDF, et l'AC5 tient.
9. **Le code QR inchangé** : pour la fixture de `golden_test.rs`, la charge utile égale le littéral relevé à
   `fed34a9c7` (écrit dans le test) ; la matrice reconstruite depuis les rectangles noirs du PDF égale celle de
   `render_qr_image`, et `rxing` la relit en cette même charge utile.
10. **Le figé** : `git diff origin/epic/36-0.18.0 --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` est vide ;
    les tests de `crates/kesh-api/tests/invoice_frozen_pdf_e2e.rs` (`deux_telechargements_rendent_les_memes_octets`,
    `modifier_le_client_apres_le_gel_ne_change_pas_le_pdf`, `une_facture_annulee_garde_son_pdf_fige`, …) et de
    `invoice_pdf_e2e.rs` passent sans modification.
11. **Les tests existants** de `kesh-qrbill` passent ; ceux dont l'attente de position change sont listés au Dev Agent
    Record, chacun avec l'ancienne et la nouvelle valeur et la raison ; aucun n'est retiré sans son remplaçant (décompte
    des `#[test]` de `crates/kesh-qrbill` aux deux bornes, `git show <ref>:<fichier> | /usr/bin/grep -c '#\[test\]'`).
12. **La table des largeurs** : des valeurs témoins recopiées de l'AFM (Helvetica : espace 278, `A` 667, `W` 944, `@`
    1015, `é` 556 ; Helvetica-Bold : `A` 722, `W` 944) dans un test ; et tout caractère que `texte_imprime` rend à partir
    du jeu admis a son entrée dans la table.
13. **Le manuel** dit la nouvelle mise en page (§ *Manuel*) et ne dit plus « 27 caractères au reçu et 46 en partie
    paiement » ni « Un message long peut aussi sortir de la feuille » ; contrôle par `git grep` du `.tex`.
14. **Captures avant / après** de la section de paiement (fixture ordinaire, rue longue, pire cas ; français et
    allemand), et l'échantillon SIX Nr. 1 à côté, dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12/captures/`.

## Tests à écrire, et la preuve du rouge

Les tests s'écrivent **avant** le correctif, dans `crates/kesh-qrbill/src/pdf.rs` (`mod tests`, avec ses helpers
`textes_du_pdf`, `operations_du_pdf`) et `crates/kesh-qrbill/src/largeurs.rs`, et se lancent sur le rendu de
`fed34a9c7` : journal `/home/gcorbaz/devel/kesh-gate-logs/e36-12/rouges.log`. Helpers neufs : `traits_du_pdf` (segments
`m`/`l`, en mm, avec l'épaisseur et la couleur de trait en vigueur), `chemins_du_pdf` (boîtes englobantes des chemins hors
`re`), `rectangles_noirs` (les `re` remplis en noir), `etendue(texte, x, y, taille, gras)` (boîte d'une ligne par la table).

| Test | AC | Rouge attendu à `fed34a9c7` |
|---|---|---|
| `deux_lignes_de_separation` | 1 | aucun segment sur x = 62 |
| `des_ciseaux_sur_chaque_ligne` | 2 | aucun chemin hors QR et pointillé |
| `la_grille_six_est_tenue` (quatre langues pour le point de dépôt) | 3 | titre haut 102,8 ; colonne à 117 ; QR haut 93 ; montant à 4,5 mm du QR ; point de dépôt à gauche |
| `payable_by_dans_la_section_d_information` | 4 | débiteur à (67, 26,0) |
| `chaque_ligne_dans_sa_zone` (six fixtures) | 5 | rouge par la colonne à 117 (bordure du QR), par le pire cas (lignes hors de la feuille) et par le message long ; les fixtures ordinaires peuvent y être vertes, le rouge se juge au test entier |
| `la_coupe_mesure_les_largeurs` | 6 | 27 caractères les plus larges finissent à x ≈ 77 |
| `la_cascade_par_etape` (une fixture par étape) | 7 | aucune étape n'existe |
| `le_pire_cas_admis_se_rend_dans_ses_zones` | 8 | lignes sous y = 5, hors de la feuille |
| `le_code_qr_est_inchange` | 9 | **vert** à `fed34a9c7` par construction (garde de non-régression) ; son rouge se prouve par mutation (M8) |
| `table_des_largeurs_temoins`, `table_couvre_le_jeu_imprime` | 12 | le module n'existe pas (rouge de compilation, journalisé) |

**Mutations** (après le commit du code, E5 ; scripts versionnés dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12/mut/`,
restauration `git restore --worktree -- <fichier>` puis `touch <fichier>`) :

- **M1** la ligne verticale non dessinée → `deux_lignes_de_separation` rouge ;
- **M2** les ciseaux non dessinés → `des_ciseaux_sur_chaque_ligne` rouge ;
- **M3** colonne d'information remise à `qr_x + 50.0` → `la_grille_six_est_tenue` et `chaque_ligne_dans_sa_zone` rouges ;
- **M4** `largeur_mm` remplacée par `nombre de caractères × 1,911` (calibrage d'aujourd'hui) → `la_coupe_mesure_les_largeurs` et `le_pire_cas_admis…` rouges ;
- **M5** l'étape R1 sautée → `la_cascade_par_etape` rouge (et `le_pire_cas…` si R2-R3 ne rattrapent pas) ;
- **M6** *Payable by* remis sous le montant en partie paiement → `payable_by_dans_la_section_d_information` rouge ;
- **M7** point de dépôt ancré à gauche → `la_grille_six_est_tenue` rouge, dans chacune des quatre langues ;
- **M8** un module du QR décalé d'un cran au dessin (`draw_qr_matrix`) → `le_code_qr_est_inchange` rouge ;
- **M9** la garde de D7 qui renvoie toujours « tient » → `chaque_ligne_dans_sa_zone` (pire cas) rouge.

Une mutation équivalente se dit. Journal : `mut/M<n>.log`.

## Tâches

0. **Étape 0** : relire les lignes citées à la tête de la branche ; rejouer l'inventaire ; relever la charge utile de la
   fixture de `golden_test.rs` à `fed34a9c7` (AC9) ; vérifier `ls -l /usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/phvr8a.afm phvb8a.afm`.
1. **Capture « avant »** : ajouter `crates/kesh-qrbill/examples/capture_qr_facture.rs` (écrit les PDF des fixtures de
   l'AC14 dans un répertoire donné en argument, données synthétiques), le committer **seul**, le lancer
   (`scripts/mem-guard.sh cargo run -p kesh-qrbill --example capture_qr_facture -- <dir>`), rendre en PNG
   (`mutool draw -r 150 -o <png> <pdf>`), dans `kesh-gate-logs/e36-12/captures/avant/`.
2. **Tests rouges** (tableau ci-dessus), commit, `rouges.log`.
3. `largeurs.rs` : table générée depuis les deux AFM par un script versionné dans `kesh-gate-logs/e36-12/` (sortie
   copiée dans le module par l'outil Edit ou Write ; commande et empreinte des AFM au Dev Agent Record), doc-comment de
   provenance ; `pub(crate)` pour `est_caractere_qr` si le test de couverture l'exige.
4. `pdf.rs` : `draw_separator` (D2, D3), grille (D4) en constantes nommées et documentées (source de chaque cote),
   `couper_ligne` à la largeur (D5), message (D6), cascade (D7) en fonction pure de mise en page puis dessin,
   *Payable by* (D4), section montant (D9), point de dépôt aligné à droite ; doc-comments du module (`:1-9`) et des
   fonctions mis à jour ; le commentaire `:744` et le doc-comment `:803-804` disparaissent avec leur défaut.
5. Mettre à jour les tests de position existants (AC11), chacun justifié au Dev Agent Record.
6. Mutations M1 à M9.
7. **Capture « après »**, mêmes fixtures, `captures/apres/`, et l'échantillon SIX Nr. 1 rendu à côté
   (`captures/comparaison-*.png`) ; contrôle à l'œil de la grille et des ciseaux, écrit au Dev Agent Record.
8. **Manuel** (`docs/manual/fr/user-manual.tex`, § *Génération du PDF avec QR Bill*), le PDF régénéré à l'intégration.
9. Gates (§ *Dev Notes*), Dev Agent Record, `Status: review`, prompt de revue P1.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; l'orchestrateur l'assemble dans `## [0.18.0] — Non publié`) :

- QR-facture : la section de paiement suit la directive SIX. Le récépissé et la partie paiement sont délimités par deux
  lignes pointillées portant chacune des ciseaux ; *Payable by* s'imprime dans la section d'information, avant le
  montant ; le montant est à la même hauteur au récépissé et en partie paiement ; le point de dépôt est aligné à droite ;
  aucun texte ne touche plus le code QR ni ne sort de la feuille, même fait de lettres très larges, et un message trop
  long se termine par « ... ». Une adresse trop longue pour le récépissé y perd d'abord la rue et le numéro, comme la
  directive le permet. Le code QR et ce qu'il contient ne changent pas. **Les PDF déjà émis et figés ne changent pas** ;
  une facture validée jamais ouverte, un rappel et un PDF refigé par un administrateur prennent le nouveau rendu.
  (refs #944)

## Dev Notes

### Hors périmètre

- **Cadre vide du débiteur** (« Payable by (name/address) », 65 × 25 mm en partie paiement, 52 × 20 mm au récépissé,
  § 3.5.4 et § 3.6.2) : aucun chemin de production sans débiteur (`invoice_pdf_service.rs:384`). **Issue à ouvrir**
  (`technical-debt`, P4).
- **Type d'adresse K**, retiré par la directive v2.4 : aucun chemin d'impression ne l'emploie (C-e3312-7) ; le lecteur
  des QR scannés (`kesh-qrbill/src/parser.rs:206`, `kesh-api/src/qr_decode.rs:404`) le lit encore. **Issue à ouvrir**
  (`technical-debt`, P3) : distinguer ce qu'on émet de ce qu'on lit.
- **Police embarquée** (#943) : la table des largeurs sera à refaire avec elle — écrit au doc-comment de `largeurs.rs`.
- **Les PDF figés d'avant la 0.17.5** (#957) : inchangés, comme tout PDF figé.
- **Tailles 8 / 10 pt recommandées** (§ 3.4) : non adoptées (C-36-12-6) ; pas d'issue — permis par la directive.
- **#12** (CR-005, « Symbole ciseaux ✂ sur la ligne de séparation ») : son titre est réalisé par D3 ; sa portée « police
  embarquée » est #943. À l'orchestrateur : `closes #12` à la PR, avec un commentaire qui renvoie à #943.

### Splitting — deux modules

`crates/kesh-qrbill` (`src/pdf.rs`, `src/largeurs.rs` neuf, `src/lib.rs` pour le `mod`, `examples/` neuf) et le manuel
utilisateur. Sous le seuil de cinq ; une seule crate. Si la validation juge la story longue, la coupe naturelle est :
36-12a (lignes, ciseaux, grille, *Payable by*, section montant, point de dépôt) et 36-12b (largeurs mesurées, message,
cascade, pire cas) — mais 36-12a seule ferait déborder les longues adresses sur la section montant, d'où le choix d'une
story.

### Fichiers communs avec les autres stories de la phase 2

| Fichier | Autres stories | Règle |
|---|---|---|
| `docs/manual/fr/user-manual.tex` | 36-15 (vocabulaire analytique), 36-16, 36-18 (possibles) | paragraphes distincts : la 36-12 ne touche que § *Génération du PDF avec QR Bill* (`:1273-1302`) ; PDF régénéré après l'union |
| `crates/kesh-qrbill/*` | aucune | — |
| `CHANGELOG.md` | toutes | non touché (C-e360-11) |

### Ce qui doit rester vrai

- Le code QR, octet pour octet, et sa taille de 46 mm ; le refus d'un QR trop dense (`pdf.rs:67-73`).
- Toute valeur du QR imprimée l'est par `texte_imprime` (Story 33-12a) ; l'ordre des rubriques du § 3.5.4 / § 3.6.2 ;
  aucune ligne vide (`cinq_cas_au_dessus_du_point_de_depot`, réécrit) ; tout texte en noir
  (`chaque_texte_s_imprime_en_noir`, `:2023`).
- La partie haute de la facture et ses gardes de capacité (`draw_invoice_section`, `content_floor = SEP_Y`) : inchangées.
- Les PDF figés, servis à l'identique.

### Manuel

`git grep -n "à 27~caractères au reçu\|faute de borne de largeur (issue \\\\#944)\|Ce que cette coupe ne garantit pas" -- docs/manual/fr/user-manual.tex`
rend `:1294`, `:1298`, `:1300`. À réécrire : `:1294` (dernière phrase, le message long), `:1298` (la coupe à 27 et
46 caractères devient une coupe à la largeur des zones), `:1300` (ce que la coupe ne garantit pas : remplacé par la
cascade — la rue omise au récépissé, puis une taille réduite, puis les noms raccourcis par « ... », et le message), et
`:1302` (« Un PDF déjà figé ne change pas » : y ajouter la 0.18.0, les lignes, les ciseaux et *Payable by* ; une
facture jamais ouverte, un rappel ou un refigeage prennent le nouveau rendu). Ajouter, au § *Génération du PDF avec QR
Bill* (`:1273-`), une phrase sur les deux lignes pointillées à ciseaux, à suivre pour détacher le bulletin. Le manuel
d'administration ne décrit pas la section de paiement (`git grep -n -i "récépissé\|section de paiement" --
docs/manual/fr/admin-manual.tex` : rien). Le PDF se contrôle aplati (E6).

### Gates (`e35-dev-commun.md` § *Quels gates, quand*, voie 2)

- En cours : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`
  (`scripts/mem-guard.sh`, sans verrou), `cargo nextest run -p kesh-qrbill` (ni base ni backend ; `mem-guard` seul).
- Passage en `review` : bases `kesh_e3612` remises à neuf sur `kesh-mariadb-dev2:3307`, puis
  `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock scripts/mem-guard.sh scripts/test-fast.sh --ci` avec
  `TMPDIR=/tmp/kesh-voie2` et `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3612` ; `scripts/gate-ci.sh
  kesh_ci_e3612` en **voie 1**, sous son verrou. Pas de frontend (non touché). Pas d'E2E.
- Remédiation : `test-fast --ci` seul. Clôture : `gate-ci` et E2E complet sur `kesh_e2e_e3612` recréée, au dernier
  commit de code. Une commande lourde à la fois (E7).

### E2E

Aucun E2E neuf : le défaut est dans la géométrie du document, que Playwright ne mesure pas ; le chemin HTTP qui sert le
PDF ne change pas (AC10) et reste couvert par `invoice_pdf_e2e.rs` et `invoice_frozen_pdf_e2e.rs`. La suite complète
tourne à la clôture (règle de l'epic).

### Références

- Directive `docs/six-references/ig-qr-bill-v2.4-en.pdf` § 3.3 à 3.7, § 6.4 ; échantillons
  `docs/six-references/samples-payment-part-en/muster-qr-zahlteile-en/` ; mesure
  `kesh-gate-logs/e36-12/mesure-echantillon-six.py` et `mesure-echantillon-six-nr1.log`.
- Stories 33-12a (`texte_imprime`, #941), 33-12b (adresses entières, la coupe par caractères), 25-6-b (PDF figé, #387).
- Issues : #944 (cette story), #12 (CR-005), #943 (police), #957 (anciens PDF figés).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche rédigée (Epic 36, phase 2, release 0.18.0), sur `fed34a9c7` (`origin/epic/36-0.18.0`). Choix
  C-36-12-1 à C-36-12-7 au registre `e36-choix-autonomes-36-12.md`. Échantillons SIX mesurés
  (`kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`). Clé passée à `ready-for-dev` au `sprint-status.yaml`.
  Validation non lancée.
