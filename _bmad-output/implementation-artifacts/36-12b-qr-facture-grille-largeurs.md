> **Découpée le 2026-11-17 en 44-1a (`44-1a-qr-facture-grille.md`, 0.19.0) et 44-1b (`44-1b-qr-facture-cascade.md`, backlog).** Cette fiche est laissée telle quelle, à titre d'historique ; elle ne fait plus foi.

# Story 36-12b : QR-facture — la grille SIX, *Payable by* à sa place, des largeurs mesurées (#944)

Status: backlog

*Fiche rédigée le 2026-11-15 (Opus 5.5) par le découpage de la fiche `36-12-qr-facture-separation.md` (`split`), décidé
par l'orchestrateur à la remédiation de la passe 1 de validation (V1-7, `kesh-gate-logs/e36-12/validation-p1.md`).
**Part de la 36-12a fusionnée** dans `epic/36-0.18.0` (même fichier, `pdf.rs`) ; worktree, branche et bases à attribuer
par l'orchestrateur (proposé : clé de base `e3612b`). Release **0.18.0**, phase 2. **Aucune migration** ; aucune ligne
de `kesh-api`, `kesh-db` ni `kesh-i18n` ne change. Code lu à `fed34a9c7` : **les numéros de ligne de `pdf.rs` se relisent
à l'étape 0**, la 36-12a ayant réécrit `draw_separator` (`:728-745`) et décalé ce qui suit. Registres : C-e350-1 ;
`e36-choix-autonomes-36-12.md` (C-36-12-1, -3 à -10).*

> **Dépendance à la 36-12a** (annoté le 2026-11-15, remédiation de la validation P2 de la 36-12a, V2-1) : la 36-12a
> livre désormais la **table des largeurs** (`crates/kesh-qrbill/src/largeurs.rs`, par nom de glyphe WinAnsi, `Euro`
> compris, ses témoins et sa couverture) et l'**alignement à droite du point de dépôt** au bord x = 57. La 36-12b les
> réemploie sans les refaire : elle ne garde du point de dépôt que sa **hauteur** (grille), et de la table que son usage
> pour couper les lignes. Ce qui suit en tient compte ; les mentions de la table comme livrable de cette story ont été
> retirées.

## Story

En tant que titulaire qui envoie ses QR-factures, je veux que la section de paiement suive la grille de la directive SIX
— *Payable by* dans la section d'information, le montant à la même hauteur des deux côtés, le point de dépôt aligné à
droite, chaque texte dans sa zone sans toucher le code QR ni sortir de la feuille, même fait de lettres très larges —,
afin que le bulletin soit conforme, **sans qu'aucune adresse valide ne fasse échouer un PDF qui se rendait jusqu'ici**.
Les PDF figés ne changent pas.

## Ce qui se lit au code (`fed34a9c7`)

Repère : origine en bas à gauche, mm, section de paiement `y ∈ [0, 105]` ; « haut » d'une ligne = ligne de base +
0,718 × taille × 0,35278 (`CapHeight` 718 des AFM), « bas » = ligne de base − 0,207 × taille × 0,35278 (`Descender`).

**Le récépissé** — `crates/kesh-qrbill/src/pdf.rs`, `draw_receipt`, `:893-973` : `:901` `let x = 5.0;`, `:902`
`let top = SEP_Y - 5.0;` ; titre 11 pt gras à la ligne de base 100 (haut ≈ 102,8 : **2,2 mm du bord**) ; `:912`
`let mut y = top - 7.0;` ; IBAN, créancier, référence ; **puis le montant**, `:926` `y -= 8.0;` (il **suit** le contenu) ;
**puis *Payable by***, `:949-950` `// Debtor.` `y -= 8.0;` ; point de dépôt `:963-969`, `Mm(RECEIPT_W - 15.0),`
`Mm(5.0),` — **ancré à gauche** à x = 47, ligne de base 5 *(lecture d'avant la 36-12a, qui l'aligne à droite au bord
x = 57 par `largeur_mm` ; annoté le 2026-11-15, V3-7)*.

**La partie paiement** — `draw_payment_part`, `:977-1083` : titre à x = 67, base 100 ; QR `:999-1000` `let qr_top = top -
7.0;` `draw_qr_matrix(layer, qr, qr_x, qr_top, 46.0)?;` (x 67→113, y 47→93) ; croix `:1001` `draw_swiss_cross(layer,
qr_x + 46.0 / 2.0, qr_top - 46.0 / 2.0);` (carré blanc 8 × 8 mm, `:1127-1134`, puis rouge 7 × 7, `:1138-1145`) ; colonne
de texte `:1004` `let tx = qr_x + 50.0;` — **x = 117, à 4 mm du QR** ; message `:1041` `layer.use_text(msg, 8.0, Mm(tx),
Mm(y), helv);` — **une ligne sans borne** ; montant `:1045` `let amount_y = qr_top - 52.0;` — intitulé base 41, haut 42,5,
**4,5 mm sous le QR** ; débiteur `:1068-1070` `// Debtor block (bottom-left).` `let mut dy = ay - 8.0;` — ***Payable by*
sous le montant, dans la colonne du QR**.

**La coupe** — `:795-810` : `const MAX_CARACTERES_RECU: usize = 27;`, `const MAX_CARACTERES_PAIEMENT: usize = 46;`, calés
sur la largeur moyenne des capitales ; `:803-804` « 27 `W` occupent 72 mm depuis x = 5, jusqu'à x = 77, dix millimètres
sur le QR. Limite écrite, à #944. » ; `couper_ligne` (`:820-847`) compte des caractères ; `dessiner_adresse`
(`:875-889`) à pas fixe `ADRESSE_PAS` = 3,5 (`:750`), sans borne basse.

**Les tests qui figent la mise en page d'aujourd'hui** (`mod tests`, `:1428-`) : `constantes_de_coupe_recalculees`
(`:1807`), `quatre_blocs_impriment_la_composition` (`:1823`), `sans_reference_aucune_rubrique` (`:1851`, « le montant
remonte d'autant »), `rue_longue_les_y_suivent` (`:1865`), `le_pays_s_imprime_hors_de_suisse` (`:1910`),
`cinq_cas_au_dessus_du_point_de_depot` (`:1931`), `chaque_site_imprime_la_base_des_lettres` (`:1974`), les
`couper_ligne_*` (`:1707`, `:1726`, `:1757`, `:1786`, `:1797`) ; `chaque_texte_s_imprime_en_noir` (`:2023-2077`, cinq
`rg` dans l'ordre, `:2076` `assert_eq!(apres_la_croix, 17);`). Fixtures : `invoice_fixture()` (`:1435`) est de type K
(`AddressType::Combined`, QRR `build_qrr(42, 100)`) ; `adresse_s` (`:1586`) fabrique le type S de la production ;
`tests/golden_test.rs:23-26` `sample_qr_data()` est de type K et n'est pas visible d'un `mod tests` de `src/`.

**Ce qui atteint le rendu en production** — `crates/kesh-api/src/routes/invoice_pdf_service.rs` : type S toujours
(`:280`, `:300`) ; un débiteur toujours (`:384` `ultimate_debtor: Some(debtor.clone()),`) ; message « préfixe + numéro »
borné à 140 (`:391`). Une erreur `QrBillError::PdfGeneration` devient `AppError::PdfGenerationFailed` (`:507`), un 500
**à la première ouverture ou au premier envoi** de la facture. Bornes (`validation.rs:10-17`) : nom 70, rue 70, numéro
16, NPA 16, localité 35, message 140 ; jeu admis `est_caractere_qr` (`:222-234`), dont `€` (U+20AC) ; `texte_imprime`
(`texte.rs:46-55`) replie U+2019 en `'` et `Ĳ` en `IJ`.

**Ce qui est figé** — `crates/kesh-api/src/routes/issued_invoice_pdf.rs:9-11`, `:16` : une facture figée est relue
octet pour octet, SHA-256 vérifié, jamais régénérée en silence. Nouveau rendu : facture validée jamais ouverte
(`render_and_pose`, `:141-`), rappels (`invoice_email.rs:566-572`, `:1317-1323`), refigeage (`refreeze`, `:238-`).

**Les métriques** — AFM d'Adobe (1990) de TeX Live : `/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/phvr8a.afm`
et `phvb8a.afm` (relus : `/usr/bin/grep -ic euro` rend `0` et `0` — **pas de glyphe `Euro`** ; `C 39 ; WX 222 ; N
quoteright`, `C 169 ; WX 191 ; N quotesingle`, gras `WX 238` ; `C 193 ; WX 333 ; N grave`). La colonne `C` est
l'*AdobeStandardEncoding* : 0x27 y est `quoteright`, alors qu'en WinAnsi 0x27 est `quotesingle`. Ce que printpdf écrit,
c'est WinAnsi (`printpdf-0.7.0/src/font.rs:76`), par la table `WIN_ANSI_ENCODING` de
`lopdf-0.31.0/src/encodings/mappings.rs:521` (`Some(Glyph::Euro)` à `:650`). **La table s'indexe donc par nom de glyphe
WinAnsi**, non par le code AFM.

**La directive** (`docs/six-references/ig-qr-bill-v2.4-en.pdf`) : § 3.3 (210 × 105 ; 62 / 148) ; § 3.4 (titres 11 pt ;
récépissé 6 / 8 ; partie paiement 6 à 10 pt, intitulés deux points sous les valeurs) ; § 3.5 (espaces entre sections
« at least 5 mm ») ; § 3.5.2 (bordure de 5 mm du QR) ; § 3.5.4 (ordre *Account / Payable to*, *Reference*, *Additional
information*, *Payable by* ; message raccourci « marked with an ellipsis "..." » ; nom du créancier tronqué « in
consultation with the recipient bank », du débiteur « only if the information remains clear ») ; § 3.6 (point de dépôt
« at least 2 cm », marges de 5 mm réductibles) ; § 3.6.2 (« smaller or different font sizes … minimum 6 pt », « omit the
street name and building number ») ; § 3.6.4 (« right-aligned ») ; § 6.4 (QR 46 × 46). Échantillons SIX mesurés
(`kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`, Nr. 1, 2, 13) : titres (haut) à 5,9–6,3 mm du haut ; premier
intitulé du récépissé à 12,9 ; colonne d'information à 117,9, premier intitulé à 5,2 ; montant (haut) à 68,6 des deux
côtés ; point de dépôt (haut) à 82,7 ; récépissé de 5,1 à 56,8 ; *Payable by* dans la section d'information des deux
côtés (Nr. 1). ⚠️ Le bord droit du point de dépôt à 57 vient de la directive (marge, « right-aligned »), non de la mesure.

### Inventaire par le symptôme

Symptôme : « une QR-facture en PDF est produite ». `git grep -n "generate_qr_bill_pdf" -- crates` hors `kesh-qrbill` :
`kesh-api/src/routes/invoice_pdf_service.rs:244` (production, inchangé, prend le nouveau rendu) et
`kesh-api/src/qr_decode.rs:900` (témoin vectoriel du test `#[ignore]` `t4_pdfium_scans_synthetiques`, `:975`) —
**à rejouer** : la story déplace le QR de 5 mm (commande `:913-914`). Sonde positive : `pdf.rs:101-104`. Hors : avoir
(`pdf.rs:112`), `qr_decode_synthetique.rs:462`, `:524`, `kesh-report/src/pdf.rs:317`.

## Décisions

- **D1 — Périmètre** (C-36-12-1) : la grille, *Payable by*, la section montant, la **hauteur** du point de dépôt, la coupe
  des lignes à la largeur mesurée, le message, la cascade et le pire cas. Hors : lignes, ciseaux, alignement à droite du
  point de dépôt et table des largeurs (36-12a, C-36-12-10) ; cadre vide du débiteur (**#1203**) ; type K (**#1204**).
- **D2 — La grille** (C-36-12-3), en mm depuis le bas :

  | Élément | Aujourd'hui | Cible | Source |
  |---|---|---|---|
  | Titres (11 pt gras) | base 100, haut 102,8 | haut ∈ [98, 100], x = 5 et 67 | marge 5 mm ; échantillons |
  | QR 46 × 46 | y 47→93 | **y 42→88**, x 67→113 ; croix centrée en (90, 65) | § 6.4, § 3.5.2 ; échantillons |
  | Colonne d'information | x = 117 | **x = 118**, toute ligne ≤ 205 | § 3.5.2 ; échantillons 117,9 |
  | Premier intitulé de cette colonne | base 93 | haut ∈ [98,8, 100] | échantillon 5,2 du haut |
  | Premier intitulé du récépissé | base 93 | haut ∈ [91,1, 93,1] | échantillon 12,9 du haut (écart assumé, D9) |
  | Texte du récépissé | coupe à 27 caractères | toute ligne dans [5, 57] | § 3.3 |
  | Section montant | suit le contenu / base 41 | **même ligne de base des deux côtés**, haut des intitulés ∈ [35,4, 37,0] | § 3.5 ; échantillons 68,6 |
  | *Payable by* | sous le montant | **section d'information** : après la référence (récépissé), après *Additional information* (partie paiement) | § 3.5.4, § 3.6.2 |
  | Fin de l'information du récépissé | non bornée | bas ≥ haut des intitulés du montant + 5 | § 3.6 |
  | Point de dépôt (6 pt gras) | bord droit 57 (36-12a), base 5 | bord droit 57 inchangé ; **haut ∈ [21,3, 23,3]** | § 3.6.4 ; échantillons 82,7 |
  | Bas de toute ligne | non borné | ≥ 5 | marges |

  Cotes du texte de la directive exactes dans les tests ; cotes des échantillons à ± 1 mm. Les ciseaux de la 36-12a
  restent hors de la bordure du QR (y ≥ 93 une fois le QR descendu).
- **D3 — Largeurs mesurées** (C-36-12-4) : `largeur_mm(texte, taille, gras)` de `crates/kesh-qrbill/src/largeurs.rs`,
  **livré par la 36-12a** (tables par nom de glyphe WinAnsi, `Euro` compris, témoins et couverture), réemployé tel quel.
  `couper_ligne` coupe à une largeur en mm (52 au récépissé, 87 dans la colonne d'information), à une frontière de mot, un
  mot trop large au dernier caractère qui tient ; `MAX_CARACTERES_RECU` et `MAX_CARACTERES_PAIEMENT` disparaissent.
- **D4 — Le message** coupé dans les 87 mm comme une adresse ; raccourci, « ... » en fin, seulement à l'étape P1.
- **D5 — La cascade, dont le dernier palier ne peut pas échouer** (C-36-12-5, C-36-12-9 ; V1-3) : étapes appliquées
  dans l'ordre, chacune seulement si la précédente ne tient pas :
  - récépissé : **R0** tout à 8 pt ; **R1** rue et numéro omis (§ 3.6.2) ; **R2** valeurs **et intitulés** à 6 pt, pas de
    ligne 3,5 × 6 / 8 (§ 3.6.2) ; **R3** noms ramenés à deux lignes, « ... » ; **R4** chaque adresse réduite à **une ligne
    de nom et une ligne NPA-localité**, chacune ramenée à 52 mm avec « ... » ;
  - colonne d'information (8 pt) : **P0** tout ; **P1** message ramené aux lignes qui tiennent (une au moins), « ... » ;
    **P2** noms à deux lignes, « ... » ; **P3** message sur une ligne et chaque adresse à **une ligne de nom, une de rue,
    une de NPA-localité**, chacune ramenée à 87 mm avec « ... ».
  R4 et P3 ont une **hauteur fixe**, indépendante de l'entrée (nombre de lignes constant ; IBAN et référence d'une
  largeur bornée) : un test calcule cette hauteur à partir des constantes de la grille et l'affirme inférieure à la place
  disponible. La mise en page est une fonction **pure** qui rend une mise en page, **jamais une erreur** ;
  `QrBillError::PdfGeneration` ne subsiste que pour les causes d'aujourd'hui (QR trop dense, devise, en-tête trop long,
  montant absent), inchangées. **Exigence** : aucune entrée admise par la validation ne rend d'erreur là où le rendu
  réussit à `fed34a9c7` — prouvé par propriété (AC8). Une adresse ordinaire ne déclenche aucune étape.
  *Écartées* : refuser le PDF au premier débordement (un 500 à l'ouverture d'une facture valide) ; garder l'erreur en
  dernier recours « prouvée inatteignable » par une fixture (V1-3 : une fixture uniforme n'est pas le pire cas en hauteur) ;
  omettre la rue toujours.
- **D6 — Tailles 6 / 8 pt, titres 11** hors de R2 (C-36-12-6).
- **D7 — Section montant fixe**, alignée des deux côtés (C-36-12-7) ; montants placés comme aujourd'hui dans la section
  (décalage 12 mm au récépissé, 15 mm en partie paiement).
- **D8 — Le code QR ne change pas** : charge utile et matrice intouchées ; seule sa place change (haut de 93 à 88), **croix
  comprise** (V1-4).
- **D9 — Écarts à la lettre de la directive, assumés** (V1-11, V1-13) : (a) P2 et P3 tronquent le nom du **créancier**,
  que § 3.5.4 ne permet de tronquer qu'« in consultation with the recipient bank » — atteint seulement par des noms
  qu'aucune société réelle ne porte, après P1, et préféré à un 500 ; (b) R2 met les intitulés à 6 pt comme les valeurs,
  quand § 3.4 les veut deux points plus petits — § 3.6.2 permet au récépissé « smaller or different font sizes »,
  minimum 6 pt ; (c) R4 et P3 tronquent la rue et la localité, que la directive ne prévoit pas — dernier palier pour des
  entrées adverses, le code QR restant entier ; (d) le premier intitulé du récépissé suit l'échantillon officiel de SIX
  (3,1 mm sous le titre), non les 5 mm des espaces entre sections : la figure 7 accole la section de titre et la section
  d'information, et l'échantillon de SIX lui-même est à 3,1 mm.
- **D10 — Le figé ne bouge pas** ; aucune ligne de `kesh-api`, `kesh-db`, `kesh-i18n`.
- **D11 — Aucune migration.**

## Critères d'acceptation

1. **La grille de D2** est tenue, mesurée dans le PDF produit (`Td`, largeurs par la table) : titres, QR (x = 67, haut
   88, 46 mm), colonne d'information à 118, premiers intitulés, section montant alignée et ≥ 5 mm sous le QR, point de
   dépôt à sa nouvelle hauteur, son bord droit restant à 57 dans **les quatre langues** (libellés des quatre
   `messages.ftl` posés dans `QrBillI18n` ; le test de la 36-12a, `le_point_de_depot_est_aligne_a_droite`, mis à jour pour
   la seule ligne de base, justifié au Dev Agent Record).
2. ***Payable by*** : en partie paiement, à x = 118, sous *Additional information* (ou sous la référence) ; au récépissé,
   à x = 5, avant la section montant ; plus rien à x = 67 sous la section montant.
3. **Aucune ligne hors de sa zone**, pour chaque fixture (ordinaire de type S, rue longue, étrangère, sans référence,
   sans message, et une par étape de la cascade) : récépissé dans [5, 57], colonne d'information dans [118, 205], rien
   dans le rectangle du QR élargi de 5 mm (x ∈ [62, 118], y ∈ [37, 93]) hors le QR et la ligne verticale, rien sous y = 5,
   l'information du récépissé ≥ 5 mm au-dessus du montant, rien sur le point de dépôt ; les boîtes des ciseaux de la
   36-12a hors de la bordure du QR.
4. **La coupe mesure** : 27 caractères les plus larges du jeu ne dépassent plus 57, 46 ne dépassent plus 205 ;
   « Robert Schneider SA », « Rue du Lac 1268 » tiennent sur une ligne ; les lignes coupées, rejointes, redonnent
   l'entrée aux espaces près, hors étapes de la cascade.
5. **La cascade** : la fixture ordinaire ne déclenche aucune étape (la rue s'imprime au récépissé) ; une fixture par étape
   la déclenche, et seulement elle ; un message long finit par « ... » quand la colonne ne tient pas, entier sinon.
6. **R4 et P3 tiennent par construction** : un test calcule leur hauteur depuis les constantes et l'affirme ≤ la place
   disponible, sans PDF.
7. **La mise en page ne rend jamais d'erreur** : sa signature ne rend pas de `Result` (ou rend une erreur que seul un
   défaut interne atteint, `debug_assert!`), lu au code et cité au Dev Agent Record.
8. **Propriété « aucune entrée admise n'échoue »** : sur des entrées générées par le test, toutes admises par
   `validation::validate` — mots de longueur `⌊L/2⌋+1` et `⌊L/3⌋+1` (L = nombre de caractères les plus larges qui tiennent
   dans la colonne), chaînes uniformes du caractère le plus large, ligne NPA-localité à ses bornes (NPA 16, localité 35,
   pays étranger), nom 70, rue 70 + numéro 16, avec et sans référence, message 0 / 140 —, `generate_qr_bill_pdf` rend
   `Ok` et l'AC3 tient. Le caractère le plus large est calculé sur les sorties de `texte_imprime` sur le jeu admis, non
   écrit en dur. Le nombre de cas parcourus est écrit au Dev Agent Record.
9. **Le code QR inchangé** : pour une fixture **de type S** de `mod tests` (comme la production, `invoice_pdf_service.rs:280`),
   la charge utile égale le littéral relevé à la base de la story (36-12a fusionnée) ; la matrice reconstruite des
   rectangles noirs du PDF égale `render_qr_image` ; `rxing` la relit en cette charge ; **le carré blanc 8 × 8 et le carré
   rouge 7 × 7 de la croix sont centrés en (90, 65) à 0,01 mm**, lus aux `re` blancs et rouges.
10. **Le figé** : `git diff origin/epic/36-0.18.0 --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide ;
    `invoice_frozen_pdf_e2e.rs` et `invoice_pdf_e2e.rs` passent sans modification ; `t4_pdfium_scans_synthetiques` rejoué
    (`qr_decode.rs:913-914`), vert, ou dit non lancé.
11. **Tests existants** : ceux dont la position change sont listés au Dev Agent Record (ancienne et nouvelle valeur,
    raison), aucun retiré sans remplaçant (décompte des `#[test]` aux deux bornes) ; `chaque_texte_s_imprime_en_noir`
    garde ses cinq `rg` et son compte de `Tj` après le dernier `re` (à recompter et justifier s'il change).
12. *(La table et ses témoins sont l'AC8 de la 36-12a ; rien à refaire ici. Numéro gardé pour ne pas décaler les
    références.)*
13. **Manuel** : ne dit plus « à 27~caractères au reçu et à 46 en partie paiement » ni « faute de borne de largeur
    (issue \#944) » ni « Ce que cette coupe ne garantit pas » sous sa forme actuelle ; dit la grille, *Payable by*, la
    cascade (rue omise au récépissé, taille réduite, noms puis lignes raccourcis par « ... », message) ; contrôle par
    `git grep` du `.tex`.
14. **Captures** avant (36-12a fusionnée) / après, fixtures ordinaire, rue longue, pire cas, en français et en allemand,
    et l'échantillon SIX Nr. 1, dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12b/captures/`.

## Tests à écrire, et la preuve du rouge

**Ordre (V1-2)** : la table est déjà là (36-12a) ; les tests de comportement compilent donc à la base de la story et
rougissent un à un contre l'ancienne mise en page ; journal `/home/gcorbaz/devel/kesh-gate-logs/e36-12b/rouges.log` (nom
et message d'échec de chacun).

| Test | AC | Rouge attendu (36-12a fusionnée) |
|---|---|---|
| `la_grille_six_est_tenue` (quatre langues) | 1 | titre haut 102,8 ; colonne 117 ; QR haut 93 ; montant 4,5 mm sous le QR ; point de dépôt à la base 5 |
| `payable_by_dans_la_section_d_information` | 2 | débiteur à (67, 26,0) ; au récépissé, sous le montant |
| `chaque_ligne_dans_sa_zone` | 3 | colonne à 117 dans la bordure ; lignes hors de la feuille pour les fixtures longues |
| `la_coupe_mesure_les_largeurs` | 4 | 27 caractères les plus larges jusqu'à x ≈ 77 |
| `la_cascade_par_etape` | 5 | aucune étape n'existe |
| `les_derniers_paliers_tiennent_par_construction` | 6 | les constantes n'existent pas : écrit **après** les constantes, rouge par mutation (M13) |
| `aucune_entree_admise_n_echoue` | 8 | lignes hors de la feuille (le rendu actuel réussit mais déborde : rouge par l'AC3 dans la propriété) |
| `le_code_qr_est_inchange` | 9 | croix : rouge (centre à (90, 70) aujourd'hui) ; charge et matrice : vertes (garde) |

**Mutations** (après le commit du code, E5 ; `/home/gcorbaz/devel/kesh-gate-logs/e36-12b/mut/`, restauration
`git restore --worktree -- <fichier>` puis `touch`) :
- **M1** colonne d'information remise à `qr_x + 50.0` → `la_grille_six_est_tenue`, `chaque_ligne_dans_sa_zone` ;
- **M2** `largeur_mm` remplacée par `caractères × 1,911` → `la_coupe_mesure_les_largeurs`, `aucune_entree_admise_n_echoue` ;
- **M3** l'étape R1 sautée → `la_cascade_par_etape` ;
- **M4a** *Payable by* remis sous le montant en partie paiement → `payable_by_dans_la_section_d_information` ;
- **M4b** *Payable by* remis sous le montant au récépissé → idem (V1-12) ;
- **M5** point de dépôt remis à la ligne de base 5 → `la_grille_six_est_tenue`, dans chacune des quatre langues ;
- **M6** un module du QR décalé d'un cran → `le_code_qr_est_inchange` ;
- **M7** R4 retiré (la cascade s'arrête à R3) → `aucune_entree_admise_n_echoue` ;
- **M8** P3 retiré → idem ;
- **M10** croix laissée à l'ancien centre (`qr_top` d'avant) → `le_code_qr_est_inchange` (V1-4) ;
- **M11** message raccourci sans « ... » → `la_cascade_par_etape` (V1-12) ;
- **M13** hauteur de ligne de R4 portée à 5 mm → `les_derniers_paliers_tiennent_par_construction`.

Une mutation équivalente se dit.

## Tâches

0. **Étape 0** : relire chaque ligne citée **après la fusion de la 36-12a** (les numéros de `pdf.rs` ont bougé) ; rejouer
   l'inventaire ; relever la charge utile de la fixture de type S (AC9) ; `ls -l` des deux AFM et de
   `mappings.rs` *(la table est livrée par la 36-12a, `crates/kesh-qrbill/src/largeurs.rs` : relire son script et son
   empreinte plutôt que les AFM — annoté le 2026-11-15, V3-7)* ; relire les LOW de la validation de cette fiche.
1. **Capture « avant »** par l'exemple de la 36-12a (`captures/avant/`), avec les fixtures de cette story ajoutées.
2. *(La table est livrée par la 36-12a.)*
3. **Tests de comportement** rouges, commit, `rouges.log`.
4. `pdf.rs` : grille en constantes nommées et sourcées ; mise en page pure (étapes R0–R4, P0–P3) puis dessin ;
   `couper_ligne` à la largeur ; *Payable by* ; section montant ; hauteur du point de dépôt ; doc-comments du module et
   des fonctions ; disparition des constantes `MAX_CARACTERES_*` et du doc-comment `:803-804`.
5. Tests de position existants mis à jour (AC11), chacun justifié.
6. Mutations.
7. Capture « après » et comparaison avec l'échantillon SIX.
8. Manuel (`docs/manual/fr/user-manual.tex`, `:1294`, `:1298`, `:1300`, `:1302`), PDF régénéré à l'intégration.
9. Gates, Dev Agent Record, `Status: review`, prompt de revue P1.

## Note de version

*Rubrique* **Corrigé** (C-e360-11) :

- QR-facture : la section de paiement suit la grille de la directive SIX. *Payable by* s'imprime dans la section
  d'information, avant le montant ; le montant est à la même hauteur au récépissé et en partie paiement ; le point de
  dépôt remonte à la place que la directive lui donne ; aucun texte ne touche plus le code QR ni ne sort de la feuille, même fait de lettres très
  larges, et un message trop long se termine par « ... ». Une adresse trop longue pour le récépissé y perd d'abord la rue
  et le numéro, comme la directive le permet, puis s'imprime plus petit ; une adresse hors de toute mesure est raccourcie
  par « ... » plutôt que de faire échouer le PDF. Le code QR et son contenu ne changent pas. **Les PDF déjà émis et figés
  ne changent pas** ; une facture validée jamais ouverte, un rappel et un PDF refigé prennent le nouveau rendu. (refs #944)

## Dev Notes

### Hors périmètre

Lignes et ciseaux : 36-12a. Cadre vide du débiteur : **#1203**. Type K : **#1204**. Police embarquée : #943 (la table
sera à refaire avec elle). Anciens PDF figés : #957. Tailles 8 / 10 recommandées : non adoptées (C-36-12-6). **`closes
#944`** à la PR de release, une fois la 36-12a et cette story fusionnées (les sept points couverts, les deux autres
suivis par #1203 et #1204).

### Splitting — deux modules

`crates/kesh-qrbill` (`src/pdf.rs` ; `src/largeurs.rs` et `examples/` réemployés de la 36-12a) et le manuel utilisateur.

### Fichiers communs

| Fichier | Autres stories | Règle |
|---|---|---|
| `crates/kesh-qrbill/src/pdf.rs`, `examples/capture_qr_facture.rs` | 36-12a | en série : part de la 36-12a fusionnée |
| `docs/manual/fr/user-manual.tex` | 36-12a, 36-15, 36-16, 36-18 | paragraphes distincts ; PDF après l'union |
| `CHANGELOG.md` | toutes | non touché |

### Ce qui doit rester vrai

Le code QR, octet pour octet, et ses 46 mm ; le refus d'un QR trop dense (`pdf.rs:67-73`) ; toute valeur imprimée par
`texte_imprime` ; l'ordre des rubriques ; aucune ligne vide ; tout texte en noir ; la partie haute de la facture et ses
gardes ; les PDF figés ; **aucune facture valide qui se rendait ne cesse de se rendre**.

### Gates

Comme la 36-12a (`e35-dev-commun.md` § *Quels gates, quand*), sur la voie et les bases que l'orchestrateur attribuera ;
`t4_pdfium` rejoué au passage en revue.

### E2E

Aucun E2E neuf : la géométrie du document n'est pas mesurable par Playwright ; la route ne change pas (AC10). Suite
complète à la clôture.

### Références

Directive § 3.3–3.7, § 6.4 ; échantillons SIX et leur mesure ; AFM `phvr8a.afm`, `phvb8a.afm` ;
`lopdf-0.31.0/src/encodings/mappings.rs:521` ; fiche d'origine `36-12-qr-facture-separation.md` (`split`) ; validation P1
`kesh-gate-logs/e36-12/validation-p1.md` ; stories 33-12a, 33-12b, 25-6-b.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche écrite au découpage de la 36-12 (décision de l'orchestrateur, V1-7), sur `fed34a9c7`. Findings de la
  P1 portés ici : V1-1 (table par nom de glyphe, `Euro`, témoins `'` et `€`), V1-2 (ordre : la table d'abord), V1-3 (R4 et
  P3 infaillibles, propriété AC8), V1-4 (croix, M10, fixture de type S), V1-5 (témoin `qr_decode.rs:900`), V1-8 (#1203,
  #1204), V1-11 et V1-13 (écarts assumés, D9), V1-12 (M4a/M4b, M11). Choix C-36-12-8 et C-36-12-9. Statut `backlog`
  jusqu'à la fusion de la 36-12a. Validation de cette fiche non lancée.
- 2026-11-15 — Remédiation de la validation P2 **de la 36-12a** (V2-1, arbitrage de l'orchestrateur) : l'alignement à
  droite du point de dépôt et la table des largeurs (V1-1, V1-2 : D3, AC12, M12, tâche 2) passent à la 36-12a ; cette
  story garde la hauteur du point de dépôt (D2, AC1, M5) et réemploie `largeur_mm`. Dépendance écrite en tête. Choix
  C-36-12-10.
