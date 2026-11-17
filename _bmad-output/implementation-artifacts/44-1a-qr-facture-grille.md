# Story 44-1a : QR-facture — la grille SIX, *Payable by* à sa place, des largeurs mesurées, un palier de secours (#944)

Status: ready-for-dev

*Fiche née de la 36-12b (`36-12b-qr-facture-grille-largeurs.md`, rédigée le 2026-11-15, laissée telle quelle), reprise le
2026-11-17 comme **première story de l'Epic 44 « QR-facture conforme »** (#944), release **0.19.0** (vendredi 2026-11-20 ;
coupure le jeudi 19 au soir). Découpée à la remédiation de la validation P1 (H1, option (b),
`kesh-gate-logs/e44/validation-p1.md`) : **cette story (44-1a)** garde la grille, *Payable by*, le montant fixe, la hauteur
du point de dépôt, la coupe à la largeur, le « ... » du message et **R4 et P3 comme seul palier de secours** ; les étapes
intermédiaires (R1–R3, P1–P2) vont à la **44-1b** (`44-1b-qr-facture-cascade.md`, backlog).
Part de `epic/44-qr-facture-conforme` (= v0.18.0, 36-12a comprise). Clé de base `e441`, journaux
`/home/gcorbaz/devel/kesh-gate-logs/e44/`. Registres des choix : lire en lecture seule
`e36-choix-autonomes-36-12.md` (C-36-12-1, -3 à -10, qui s'appliquent encore) ; les choix neufs vont à
`e44-choix-autonomes-44-1.md`. **Aucune migration.** Code relu à `8b50fd195` ; **les numéros de ligne se relisent à l'étape 0**.*

> **Dépendance à la 36-12a** (fusionnée) : elle livre la **table des largeurs** (`crates/kesh-qrbill/src/largeurs.rs`, par
> nom de glyphe WinAnsi, `Euro` compris, ses témoins) et l'**alignement à droite du point de dépôt** (bord x = 57). Cette
> story les réemploie ; elle ne garde du point de dépôt que sa **hauteur**.

## Story

En tant que titulaire qui envoie ses QR-factures, je veux que la section de paiement suive la grille de la directive SIX
— *Payable by* dans la section d'information, le montant à la même hauteur des deux côtés, chaque texte dans sa zone sans
toucher le code QR ni sortir de la feuille, même fait de lettres très larges —, afin que le bulletin soit conforme, **sans
qu'aucune adresse valide ne fasse échouer un PDF qui se rendait jusqu'ici**. Les PDF figés ne changent pas.

## Ce qui se lit au code (`8b50fd195`, `crates/kesh-qrbill/src/pdf.rs`, 3856 lignes)

Repère : origine en bas à gauche, mm, section de paiement `y ∈ [0, 105]` ; « haut » d'une ligne = base + 0,718 × taille ×
0,35278 (`CapHeight`), « bas » = base − 0,207 × taille × 0,35278 (`Descender`) ; à 8 pt : bas = base − 0,58.

- **Récépissé** — `draw_receipt` `:1055` : `x = 5.0`, `top = SEP_Y - 5.0` ; premier intitulé à `top - 7.0` ; IBAN (pas 3),
  `dessiner_adresse` (`:1037`, pas `ADRESSE_PAS` = 3,5, `:912`) ; `dessiner_reference` (`:1011`, pas 6 puis 3) ; **montant
  qui suit le contenu**, `y -= 8.0` (`:1090`), intitulés à 6 pt, valeur 3,5 plus bas, montant à `x + 12.0` ; ***Payable by*
  sous le montant**, `y -= 8.0` (`:1111`) ; point de dépôt (6 pt gras, bord droit `RECU_BORD_DROIT` = 57, 36-12a), base 5.
- **Partie paiement** — `draw_payment_part` `:1143` : QR `qr_top = top - 7.0` (= 93, `:1165`), `draw_qr_matrix(…, 46.0)`
  `:1166`, croix `:1167` (centre (90, 70), carré blanc 8 × 8 puis rouge 7 × 7) ; colonne de texte `tx = qr_x + 50.0` = 117
  (`:1170`) ; message **une ligne sans borne**, `layer.use_text(msg, 8.0, Mm(tx), Mm(y), helv)` ; montant `amount_y =
  qr_top - 52.0` = 41 (`:1211`), débiteur `dy = ay - 8.0` (`:1234`) — ***Payable by* sous le montant, dans la colonne du QR**.
- **La coupe** — `MAX_CARACTERES_RECU = 27` (`:967`), `MAX_CARACTERES_PAIEMENT = 46` (`:972`), doc « limite écrite, à
  #944 » (`:966`) ; `couper_ligne` (`:982`) compte des caractères, **sauf « un mot coupé à la limite, que seule leur
  concaténation redonne »** (doc de `:982`). `lignes_adresse` (`:924`) : type S = nom, `rue + numéro`, `NPA localité` (pays
  devant le NPA hors CH) ; type K = nom, ligne 1, ligne 2 ; **les lignes vides sont filtrées**.
- **Hors périmètre en production** — `crates/kesh-api/src/routes/invoice_pdf_service.rs` : type S toujours (`:280`, `:300`),
  un débiteur toujours (`:384`), message « préfixe + numéro » borné à 140 (`:391`) ; erreur `PdfGeneration` →
  `AppError::PdfGenerationFailed` (`:507`) ; `generate_qr_bill_pdf` appelé à `:244`. Bornes `validation.rs:10-19` : nom 70,
  rue 70, numéro 16, NPA 16, localité 35, message 140 ; jeu `est_caractere_qr` (`:222`) ; `texte_imprime` (`texte.rs:47`).
- **Le figé** — `issued_invoice_pdf.rs:9-16` : relu octet pour octet, SHA-256 vérifié ; nouveau rendu pour une facture
  validée jamais ouverte (`render_and_pose`, `:141`), un rappel (`invoice_email.rs`), un refigeage (`refreeze`, `:238`).
- **La directive** (`docs/six-references/ig-qr-bill-v2.4-en.pdf`) : § 3.3 (210 × 105 ; 62 / 148), § 3.4 (6–10 pt ; 6 / 8 au
  récépissé ; titres 11 pt), § 3.5 (espaces ≥ 5 mm), § 3.5.2 (bordure de 5 mm du QR), § 3.5.4 (ordre *Account*, *Reference*,
  *Additional information*, *Payable by* ; « ... » ; troncature du nom du créancier « in consultation with the recipient
  bank »), § 3.6 (point de dépôt ≥ 2 cm), § 3.6.2 (6 pt minimum, rue omissible), § 3.6.4 (« right-aligned »), § 6.4 (QR 46).
  Échantillons SIX (`kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`) : titres (haut) à 5,9–6,3 mm du haut, premier
  intitulé du récépissé à 12,9, colonne d'information à 117,9 et premier intitulé à 5,2, montant (haut) à 68,6 des deux
  côtés, point de dépôt (haut) à 82,7. **46 mm est la cote de la directive** ; la bande mesurée de l'échantillon (18,3 à
  62,4 depuis le haut, soit 44,1 mm) est l'étendue des modules sombres ; le centre (≈ 64,65 mesuré, 65 posé) concorde (L5).

### Inventaire par le symptôme

Symptôme : « une QR-facture en PDF est produite ». `git grep -n "generate_qr_bill_pdf" -- crates` hors `kesh-qrbill` :
`kesh-api/src/routes/invoice_pdf_service.rs:244` (production, **lu, non modifié** : il prend le nouveau rendu) et
`kesh-api/src/qr_decode.rs:900` (témoin du test `#[ignore]` `t4_pdfium_scans_synthetiques`, **à rejouer** : le QR descend de
5 mm). **`kesh-api` n'est donc pas modifié** : le plan `release-0.19.0.md` (§ Epic 44) le cite parmi les fichiers de
l'epic ; ce sont des fichiers *lus* (invariants du figé), non des fichiers à changer — le plan est à corriger, personne ne
doit s'y croire autorisé (AC10).

## Décisions

- **D1 — Périmètre** : la grille, *Payable by*, la section montant fixe, la **hauteur** du point de dépôt, la coupe à la
  largeur mesurée, le « ... » du message, **R4 et P3**. Hors : R1–R3, P1–P2 (**44-1b**) ; lignes, ciseaux, alignement à
  droite du point de dépôt, table des largeurs (36-12a) ; cadre vide du débiteur (**#1203**) ; type K (**#1204**) ; police
  embarquée (#943) ; PDF figés (#957).
- **D2 — La grille** (mm depuis le bas ; **constantes nommées et sourcées** dans `pdf.rs`) :

  | Élément | Aujourd'hui | Cible | Source |
  |---|---|---|---|
  | Titres (11 pt gras) | base 100, haut 102,8 | **base 96,5** (haut = base + 2,79 = 99,29 ∈ [98, 100]) ; x = 5 et 67 | marge 5 mm ; échantillons |
  | QR 46 × 46 | y 47→93 | **y 42→88**, x 67→113 ; croix centrée en (90, 65) | § 6.4, § 3.5.2 |
  | Colonne d'information | x = 117 | **x = 118**, toute ligne ≤ 205 | § 3.5.2 ; échantillon 117,9 |
  | Premier intitulé de la colonne | base 93 | **base 98,0** (haut 99,5 ∈ [98,8, 100]) | échantillon 5,2 du haut |
  | Premier intitulé du récépissé | base 93 | **base 90,5** (haut 92,0 ∈ [91,1, 93,1]) | échantillon 12,9 (écart assumé, D8) |
  | Montant (intitulés 6 pt) | suit le contenu / base 41 | **base 35,0 des deux côtés** (haut 36,52 ∈ [35,4, 37,0]) ; valeur 3,5 plus bas (31,5) | § 3.5 ; échantillons |
  | *Payable by* | sous le montant | **section d'information**, après la référence (récépissé), après *Additional information* ou la référence (partie paiement) | § 3.5.4, § 3.6.2 |
  | Fin de l'information du récépissé | non bornée | bas ≥ **41,52** (= haut des intitulés du montant 36,52 + 5), mesuré sur la boîte avec descendantes | § 3.5, § 3.6 |
  | Point de dépôt (6 pt gras) | bord droit 57 (36-12a), base 5 | bord droit 57 inchangé ; **base 20,5** (haut 22,02 ∈ [21,3, 23,3]) | § 3.6.4 ; échantillon 82,7 |
  | Bas de toute ligne | non borné | ≥ 5 | marges |

  **Pas verticaux, constantes** : intitulé → valeur 3,0 (IBAN, référence, message) et 3,5 (montant) ; valeur d'adresse → ligne
  suivante `ADRESSE_PAS` = 3,5 (`:912`) ; dernière valeur → intitulé suivant 6,0 (comme `dessiner_reference`, `:1013`). Pas de
  nombre magique : chacun est une constante nommée, citée par les tests. Ciseaux de la 36-12a hors de la bordure du QR
  (y ≥ 93 une fois le QR descendu).
- **D3 — Largeurs mesurées** : `largeur_mm(texte, taille, gras)` de `largeurs.rs`, réemployé tel quel. `couper_ligne(ligne,
  largeur_mm)` coupe à une largeur en mm (**52** au récépissé, **87** dans la colonne d'information), à une frontière de
  mot, un mot trop large au dernier caractère qui tient ; `MAX_CARACTERES_RECU` et `MAX_CARACTERES_PAIEMENT` disparaissent.
  **Les « ... » sont trois points ASCII (`"..."`, § 3.5.4) dont la largeur compte dans la coupe** : le dernier caractère
  conservé est celui qui laisse la place des trois points (L3).
- **D4 — La cascade, réduite à son palier de secours** (C-36-12-5, -9) — chaque palier seulement si le précédent ne
  « tient » pas ; la mise en page est une fonction **pure** qui rend une mise en page, **jamais une erreur** :
  - **Récépissé : R0 → R4.** **R0** : tout à 8 pt, adresses complètes, coupées à 52 mm (les lignes « tiennent » si le bas
    de la dernière ligne ≥ 41,52). **R4** : le **créancier et le débiteur ensemble** (pas l'un après l'autre) sont réduits à
    **la première ligne (le nom) et la dernière ligne de `lignes_adresse`** (NPA-localité pour le type S ; **type K** : la
    dernière ligne non vide, la rue n'ayant pas de ligne propre), chacune **ramenée à 52 mm avec « ... »** ; IBAN et
    référence sont de largeur bornée (≈ 33 mm au plus). R4 compte donc **au plus 2 lignes par adresse, 4 au total**.
  - **Colonne d'information : P0 → P3.** **P0** : tout, adresses complètes, message coupé aux mots à 87 mm, toutes ses
    lignes (tient si le bas de la dernière ligne ≥ 5). **P3** : message **sur une seule ligne** ramenée à 87 mm avec « ... »
    et chaque adresse sur **au plus trois lignes (celles de `lignes_adresse`), chacune d'une ligne** ramenée à 87 mm avec
    « ... ». **Au plus 3 + 3 lignes d'adresse.**
  - **Hauteurs de R4 et P3, calculées à la main** : R4 de la base du premier intitulé (90,5) : IBAN 87,5 ; créancier 84,0 et
    80,5 ; référence 74,5 (intitulé) et 71,5 ; *Payable by* 65,5 ; débiteur 62,0 et 58,5 → **bas 57,9 ≥ 41,52**, marge 16,4 mm.
    P3 depuis 98,0 : IBAN 95,0 ; créancier 91,5, 88,0, 84,5 ; référence 78,5 et 75,5 ; *Additional information* 69,5 et 66,5 ;
    *Payable by* 60,5 ; débiteur 57,0, 53,5, 50,0 → **bas 49,4 ≥ 5**. Les deux hauteurs sont **indépendantes de l'entrée**
    (nombre de lignes constant) et se calculent à partir des constantes de D2 (AC6).
  - **Types d'entrée** : le **débiteur absent** (`ultimate_debtor: None`, que la production n'émet jamais, `:384`) imprime
    l'intitulé *Payable by* seul, comme aujourd'hui (le cadre de la directive est #1203) ; les mesures le comptent. Le
    **type K** suit R4/P3 ci-dessus (`invoice_fixture()` en est un).
  - **Exigence** : aucune entrée admise par la validation ne rend d'erreur là où le rendu réussit à `8b50fd195` ; une adresse
    ordinaire ne déclenche aucun palier. `QrBillError::PdfGeneration` ne subsiste que pour les causes d'aujourd'hui (QR trop
    dense, devise, en-tête trop long, montant absent). *Écartées* : refuser le PDF au premier débordement ; omettre la rue
    toujours.
  - **Laissé à la 44-1b** : R1 (rue omise), R2 (6 pt), R3 (noms à deux lignes), P1 (message raccourci par lignes), P2 (noms à
    deux lignes). **Ce que la 44-1a lui laisse** : la mise en page pure à paliers (`enum` ou liste ordonnée de paliers, chacun
    avec son critère « tient »), les constantes de pas, `couper_ligne` à la largeur, le garde `aucune_entree_admise_n_echoue`,
    le dispositif de mutations ; la 44-1b **insère** des paliers entre R0 et R4, P0 et P3 sans toucher à ces quatre ; sans la
    44-1b, une adresse trop longue passe de la rue complète à R4 d'un seul saut — assumé (D8).
- **D5 — Tailles 6 / 8 pt, titres 11 pt** (C-36-12-6) : inchangées.
- **D6 — Section montant fixe**, alignée des deux côtés (C-36-12-7) : intitulés à la base 35,0 ; montants placés comme
  aujourd'hui dans la section (décalage 12 mm au récépissé, 15 mm en partie paiement) ; le montant ne « remonte » plus.
- **D7 — Le code QR ne change pas** : charge utile et matrice intouchées ; seule sa place change (haut de 93 à 88), **croix
  comprise**.
- **D8 — Écarts à la lettre de la directive, assumés** : (a) P3 tronque le nom du **créancier**, que § 3.5.4 ne permet de
  tronquer qu'« in consultation with the recipient bank » — atteint seulement par des noms qu'aucune société réelle ne porte,
  et préféré à un 500 ; (b) R4 et P3 tronquent la rue et la localité, que la directive ne prévoit pas, et R4 supprime la
  rue — dernier palier pour des entrées adverses, le code QR restant entier ; (c) le premier intitulé du récépissé suit
  l'échantillon officiel de SIX (3,1 mm sous le titre), non les 5 mm des espaces entre sections.
- **D9 — Le figé ne bouge pas** ; **aucune ligne de `kesh-api`, `kesh-db`, `kesh-i18n`**. **D10 — Aucune migration.**

## Critères d'acceptation

1. **La grille de D2** est tenue, mesurée dans le PDF produit (`Td`, largeurs par la table) : titres, QR (x = 67, haut 88,
   46 mm), colonne d'information à 118, premiers intitulés, section montant à la base 35,0 des deux côtés et ≥ 5 mm sous le
   QR, point de dépôt à la base 20,5, son bord droit restant à 57 dans **les quatre langues** (libellés des quatre
   `messages.ftl`). **Le titre de la partie paiement (x = 67, 11 pt gras) et les intitulés restent à gauche de x = 118 dans les
   quatre langues** (capitales larges : `Zahlbar durch`, `Sezione pagamento`…).
2. ***Payable by*** : en partie paiement, à x = 118, sous *Additional information* (ou sous la référence) ; au récépissé,
   à x = 5, avant la section montant ; **plus rien à x = 67 sous la section montant**.
3. **Aucune ligne hors de sa zone**, pour chaque fixture (ordinaire de type S, rue longue, étrangère, sans référence, sans
   message, **débiteur absent**, **type K**, une qui déclenche R4, une qui déclenche P3) : récépissé dans [5, 57], colonne
   d'information dans [118, 205], rien dans le rectangle du QR élargi de 5 mm (x ∈ [62, 118], y ∈ [37, 93]) hors le QR et la
   ligne verticale, rien sous y = 5, l'information du récépissé ≥ 5 mm au-dessus du montant (bas ≥ 41,52), rien sur le point
   de dépôt ; boîtes des ciseaux de la 36-12a hors de la bordure du QR.
4. **La coupe mesure, avec des valeurs calculées à la main depuis l'AFM, hors de la table** (`phvr8a.afm`, `W` = 944) :
   27 `W` à 8 pt = 27 × 944 × 8 / 1000 × 25,4 / 72 = **71,93 mm** (ne tient plus à 52 mm) ; **19 `W` = 50,62 mm tiennent à 52,
   20 = 53,28 non** ; **32 `W` = 85,25 mm tiennent à 87, 33 = 87,91 non** ; « Robert Schneider SA » = **26,51 mm** et « Rue du
   Lac 1268 » = **21,49 mm** tiennent chacune sur une ligne. Ces nombres sont écrits **en dur** dans les tests, ils ne sortent
   pas de `largeur_mm`. Les lignes coupées, rejointes, redonnent l'entrée **aux espaces près, hors un mot plus large que la
   colonne, que seule leur concaténation redonne** (L2) ; hors paliers R4 et P3.
5. **La cascade** : la fixture ordinaire ne déclenche aucun palier (la rue s'imprime au récépissé) ; une fixture déclenche R4
   et seulement lui, une autre P3 et seulement lui ; au palier P3, le message tient sur **une ligne** terminée par `...` ;
   à P0 un message long s'imprime **entier**, sur plusieurs lignes (nombre de lignes attendu écrit en dur) ; R4 s'applique
   au créancier **et** au débiteur ensemble (une fixture où seul le débiteur déborde vérifie que le créancier perd aussi sa
   rue).
6. **R4 et P3 tiennent par construction** : un test calcule leur hauteur depuis les constantes de D2 (bas **57,9** et
   **49,4**, valeurs écrites en dur) et l'affirme ≥ 41,52 et ≥ 5, sans PDF.
7. **La mise en page ne rend jamais d'erreur** : sa signature ne rend pas de `Result` (ou rend une erreur que seul un défaut
   interne atteint, `debug_assert!`), lu au code et cité au Dev Agent Record.
8. **Propriété « aucune entrée admise n'échoue »** : sur des entrées générées par le test, toutes admises par
   `validation::validate` — mots de longueur `⌊L/2⌋+1` et `⌊L/3⌋+1` (L = 19 au récépissé, 32 dans la colonne : nombre de
   `W` qui tiennent), chaînes uniformes du caractère le plus large, NPA 16 et localité 35 (pays étranger), nom 70, rue 70 +
   numéro 16, avec et sans référence, message 0 / 140 —, `generate_qr_bill_pdf` rend `Ok` et l'AC3 tient. Le caractère le
   plus large est calculé sur les sorties de `texte_imprime` sur le jeu admis, non écrit en dur. **Relevé indépendant du pire
   cas** : un test `#[ignore]` (comme `t4_pdfium`, rejoué à la clôture) rend le pire cas en PDF puis en relève les **boîtes de
   texte par `pdftotext -bbox`** (poppler, présent sur la station) ou par pdfium, **sans passer par `largeur_mm`**, et
   vérifie l'AC3 sur elles ; son journal est joint au Dev Agent Record. Le nombre de cas parcourus y est écrit.
9. **Le code QR inchangé** : pour une fixture **de type S** de `mod tests`, la charge utile égale le littéral relevé à la
   base de la story ; la matrice reconstruite des rectangles noirs du PDF égale `render_qr_image` ; `rxing` la relit en cette
   charge ; **le carré blanc 8 × 8 et le carré rouge 7 × 7 de la croix sont centrés en (90, 65) à 0,01 mm**.
10. **Le figé** : `git diff origin/epic/44-qr-facture-conforme --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide
    (`kesh-api` reste intouché malgré le plan) ; `invoice_frozen_pdf_e2e.rs` et `invoice_pdf_e2e.rs` passent sans
    modification ; `t4_pdfium_scans_synthetiques` rejoué (`qr_decode.rs`), vert, ou dit non lancé.
11. **Tests existants** — liste fermée, chacun avec ancienne et nouvelle valeur et sa raison au Dev Agent Record, aucun retiré
    sans remplaçant (décompte des `#[test]` aux deux bornes) :
    - `quatre_blocs_impriment_la_composition`, `le_pays_s_imprime_hors_de_suisse`, `chaque_site_imprime_la_base_des_lettres` :
      positions nouvelles ;
    - `sans_reference_aucune_rubrique` : le montant ne « remonte » plus (D6), son sens change ;
    - `rue_longue_les_y_suivent` : à 52 mm, la rue de 61 caractères ne fait plus trois lignes ;
    - `constantes_de_coupe_recalculees` : **supprimé** avec les constantes, remplacé par les tests de l'AC4 ;
    - les `couper_ligne_*` (`couper_ligne_mot_long_sans_perte`, `_rue_entiere`, `_frontieres_en_caracteres`,
      `_reduit_les_espaces_multiples`, `_npa_peut_rester_seul`) : signature changée (largeur, non nombre de caractères) ;
    - **`les_ciseaux_ne_touchent_rien`** : fige le rectangle du QR `y0: 42.0, y1: 98.0` ; il devient **`37…93`** (QR descendu
      de 5 mm, bordure comprise) — sans quoi il passe à vide sur la mauvaise zone ;
    - **`cinq_cas_au_dessus_du_point_de_depot`** : sa règle « tout texte hors *Acceptance point* à y ≥ 5 + pas » ne protège plus
      rien une fois le point de dépôt à la base 20,5 ; elle devient une règle de **non-recouvrement avec la boîte du point
      de dépôt** ;
    - **`la_ligne_verticale_ne_traverse_aucun_texte`** : **retirer l'exclusion** de « l'adresse de capitales très larges
      (limite connue, jusqu'à la 36-12b) » et ajouter la fixture ; corriger son doc-comment ;
    - `le_point_de_depot_est_aligne_a_droite` : ligne de base seule, justifiée ;
    - `chaque_texte_s_imprime_en_noir` : garde ses cinq `rg` et son compte de `Tj` après le dernier `re` (assertion `17`) ;
      à recompter et justifier s'il change ;
    - **commentaires et doc-comments périmés** à reprendre : `:758-759` (« zone de 5 mm du QR (qui monte à 98) »), `:966`
      (« Limite écrite, à #944 »), le doc du module (`:1-12`, « 7×7 mm white square » alors que le carré blanc fait 8 × 8),
      les doc-comments de `MAX_CARACTERES_*` (`:960-972`).
12. *(Numéro gardé pour ne pas décaler les références : la table et ses témoins sont l'AC8 de la 36-12a.)*
13. **Manuel** (`docs/manual/fr/user-manual.tex`, lignes relues à `8b50fd195`) : **dit** la grille, *Payable by*, le palier de
    secours (rue omise au récépissé, lignes raccourcies par « ... », message sur une ligne), et **corrige chaque phrase
    qui devient fausse** :
    - `:1307` « Un message long peut aussi sortir de la feuille, faute de borne de largeur (issue \#944) » ;
    - `:1311` « à 27~caractères au reçu et à 46 en partie paiement~: rien n'est tronqué » (la coupe est à la largeur, et un
      palier de secours tronque par « ... ») ;
    - `:1313` « Ce que cette coupe ne garantit pas » **et** sa dernière phrase « Une ligne de 28 à 37~caractères, qui tenait
      au reçu, se coupe désormais en deux » ;
    - `:1315` (36-12a, « depuis la version~0.18.0 ») : « Le code QR et le reste du bulletin ne bougent pas » (le QR descend
      de 5 mm, le point de dépôt remonte) et le paragraphe « Deux limites demeurent… (issue \#944) » ;
    - `:1317` « émis avant la version~0.18.0 » reste juste ; y ajouter ce qui prend le nouveau rendu dès la 0.19.0.
    **Contrôle sur le PDF aplati** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, PDF régénéré à l'intégration) **en plus
    du `git grep` du `.tex`** ; les manuels `de`, `en`, `it`, le README et `website/` n'ont rien à changer (relu par la
    validation P1).
14. **Captures** avant (v0.18.0) / après, fixtures ordinaire, rue longue, pire cas, en français et en allemand, et l'échantillon
    SIX Nr. 1, dans `/home/gcorbaz/devel/kesh-gate-logs/e44/captures/`.

## Tests à écrire, et la preuve du rouge

**Ordre** : la table est déjà là (36-12a) ; les tests de comportement compilent à la base de la story et rougissent un à un
contre l'ancienne mise en page ; journal `/home/gcorbaz/devel/kesh-gate-logs/e44/rouges.log` (nom et message d'échec).

| Test | AC | Rouge attendu (v0.18.0) |
|---|---|---|
| `la_grille_six_est_tenue` (quatre langues, titre à gauche de x = 118 compris) | 1 | titre haut 102,8 ; colonne 117 ; QR haut 93 ; montant 4,5 mm sous le QR ; point de dépôt à la base 5 |
| `payable_by_dans_la_section_d_information` | 2 | débiteur à (67, 26,0) ; au récépissé, sous le montant |
| `chaque_ligne_dans_sa_zone` | 3 | colonne à 117 dans la bordure ; lignes hors de la feuille |
| `la_coupe_mesure_les_largeurs` (valeurs de l'AC4 en dur) | 4 | 27 `W` jusqu'à x ≈ 77 |
| `la_cascade_par_palier` (R4, P3, débiteur absent, type K, nombre de lignes du message) | 5 | aucun palier n'existe |
| `les_derniers_paliers_tiennent_par_construction` | 6 | écrit **après** les constantes, rouge par mutation (M10) |
| `aucune_entree_admise_n_echoue` | 8 | lignes hors de la feuille (rouge par l'AC3 dans la propriété) |
| `le_code_qr_est_inchange` | 9 | croix : rouge (centre (90, 70)) ; charge et matrice : vertes (garde) |
| `pire_cas_boites_independantes` (`#[ignore]`) | 8 | idem, relevé par `pdftotext -bbox` |

**Mutations** (après le commit du code ; `/home/gcorbaz/devel/kesh-gate-logs/e44/mut/`, restauration `git restore
--worktree -- <fichier>` puis `touch`) :
- **M1** colonne d'information remise à `qr_x + 50.0` → `la_grille_six_est_tenue`, `chaque_ligne_dans_sa_zone` ;
- **M2** `largeur_mm` remplacée par `caractères × 1,911` → `la_coupe_mesure_les_largeurs` (valeurs en dur),
  `aucune_entree_admise_n_echoue` ;
- **M3** R4 retiré (R0 gardé seul) → `aucune_entree_admise_n_echoue`, `la_cascade_par_palier` ;
- **M4a** *Payable by* remis sous le montant en partie paiement, **M4b** au récépissé → `payable_by_dans_la_section_d_information` ;
- **M5** point de dépôt remis à la base 5 → `la_grille_six_est_tenue`, dans chacune des quatre langues ;
- **M6** un module du QR décalé d'un cran → `le_code_qr_est_inchange` ;
- **M7** P3 retiré → `aucune_entree_admise_n_echoue` ;
- **M8** croix laissée à l'ancien centre → `le_code_qr_est_inchange` ;
- **M9** message raccourci sans « ... » → `la_cascade_par_palier` ;
- **M10** pas de ligne de R4 porté à 5 mm → `les_derniers_paliers_tiennent_par_construction` ;
- **M11** montant du récépissé qui suit le contenu (D6 défait) → `la_grille_six_est_tenue`, `sans_reference_aucune_rubrique`.
Une mutation équivalente se dit.

## Tâches

0. **Étape 0** : relire chaque ligne citée à la base de la story (les numéros bougent) ; rejouer l'inventaire ; relever la
   charge utile de la fixture de type S (AC9) ; relire `largeurs.rs` (script et empreinte) ; recompter les `#[test]` de
   `pdf.rs`.
1. **Capture « avant »** par l'exemple de la 36-12a (`captures/avant/`), avec les fixtures de cette story.
2. **Tests de comportement** rouges, commit, `rouges.log`.
3. `pdf.rs` : grille en constantes nommées et sourcées ; mise en page pure (R0, R4, P0, P3) puis dessin ; `couper_ligne` à la
   largeur et « ... » ; *Payable by* ; section montant ; hauteur du point de dépôt ; doc-comments ; disparition de
   `MAX_CARACTERES_*`.
4. Tests existants mis à jour (AC11), chacun justifié.
5. Mutations. 6. Capture « après » et comparaison avec l'échantillon SIX.
7. Manuel (AC13), PDF régénéré à l'intégration.
8. Gates, Dev Agent Record, `Status: review`, prompt de revue P1.

## Note de version

*Rubrique* **Modifié** (la 0.19.0 sépare les rubriques ; *Corrigé* si elle ne le fait pas) :

- QR-facture : la section de paiement suit la grille de la directive SIX. *Payable by* s'imprime dans la section
  d'information, avant le montant ; le montant est à la même hauteur au récépissé et en partie paiement ; le point de dépôt
  remonte à la place que la directive lui donne ; aucun texte ne touche plus le code QR ni ne sort de la feuille, même fait
  de lettres très larges, et un message trop long se termine par « ... ». Une adresse hors de toute mesure perd sa rue au
  récépissé et est raccourcie par « ... » plutôt que de faire échouer le PDF. Le code QR et son contenu ne changent pas.
  **Les PDF déjà émis et figés ne changent pas** ; une facture validée jamais ouverte, un rappel et un PDF refigé prennent le
  nouveau rendu. (refs #944)

## Dev Notes

### Hors périmètre

Étapes intermédiaires : **44-1b**. Lignes et ciseaux : 36-12a. Cadre vide du débiteur : **#1203**. Type K : **#1204**. Police
embarquée : #943. Anciens PDF figés : #957. **Le corps de la facture** (`IDENTITY_MAX_CHARS`, `pdf.rs:206-212`, qui dit
qu'une chaîne de 46 `W` occupe 137,9 mm et déborde) est de la même famille et hors périmètre (L4) : **aucune issue ouverte
n'a été trouvée** (recherches `gh issue list --search` sur `IDENTITY_MAX_CHARS`, « 46 W » et « débordement identité facture »,
2026-11-17) ; à ouvrir par l'orchestrateur. **`closes #944`** à la PR de release **seulement si** la 44-1a est fusionnée :
le corps de l'issue ne porte plus que *Payable by*, que cette story livre ; les points de la 44-1b suivent leur propre
issue.

### Splitting — deux modules

`crates/kesh-qrbill` (`src/pdf.rs` ; `src/largeurs.rs` et `examples/` réemployés) et le manuel utilisateur. **`kesh-api`
n'est pas modifié.**

### Fichiers communs (Epic 44)

| Fichier | Autres epics / stories | Règle |
|---|---|---|
| `crates/kesh-qrbill/src/pdf.rs`, `examples/capture_qr_facture.rs` | 44-1b (en série) | la 44-1b part de la 44-1a fusionnée |
| `docs/manual/fr/user-manual.tex` | Epics 42 et 43 (même semaine) | paragraphes distincts ; PDF jamais commités en branche, régénérés à l'intégration |
| `CHANGELOG.md` | toutes | non touché ; assemblé à la release depuis les *Notes de version* |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | Epics 42 et 43 | bloc propre ; en cas de conflit, garder les deux côtés |

### Ce qui doit rester vrai

Le code QR, octet pour octet, et ses 46 mm ; le refus d'un QR trop dense ; toute valeur imprimée par `texte_imprime` ;
l'ordre des rubriques ; aucune ligne vide ; tout texte en noir ; la partie haute de la facture et ses gardes ; les PDF
figés ; **aucune facture valide qui se rendait ne cesse de se rendre**.

### Gates et E2E

Voie, bases (`kesh_e441`, `kesh_e2e_e441`, `kesh_ci_e441`) et gates comme les consignes de la release
(`kesh-gate-logs/prompts/e0190-commun.md`) ; `t4_pdfium` et `pire_cas_boites_independantes` rejoués à la clôture. Aucun E2E
neuf : la géométrie du document n'est pas mesurable par Playwright ; la route ne change pas (AC10). Suite complète à la clôture.

### Références

Directive § 3.3–3.7, § 6.4 ; échantillons SIX et leur mesure ; AFM `phvr8a.afm`, `phvb8a.afm` (`/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/`) ; fiche d'origine `36-12-qr-facture-separation.md` (`split`) ;
fiche `36-12b-qr-facture-grille-largeurs.md` ; validation P1 `kesh-gate-logs/e44/validation-p1.md` ; stories 33-12a, 33-12b, 25-6-b.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-17 — Fiche reprise de la 36-12b (Epic 44, release 0.19.0) et **découpée** à la remédiation de la validation P1
  (Opus/Sonnet 5.5, `kesh-gate-logs/e44/remediation-p1.md`) : H1 (R1–R3, P1–P2 vers la 44-1b ; R4 et P3 gardés), M1 (valeurs
  AFM en dur, relevé indépendant `pdftotext -bbox`), M2 (liste fermée des tests, commentaires périmés), M3 (pas verticaux,
  « tient » à 41,52, R4/P3 précisés, débiteur absent, type K, intitulés des quatre langues), M4 (lignes du manuel
  relues), M5 (contexte Epic 44, `kesh-api` intouché), L1 à L6. Hauteurs de R4 et P3 **calculées à la main** (non
  mesurées : à confirmer à l'étape 0).
