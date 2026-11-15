# Epic 36 — registre des choix autonomes de la story 36-12 (clé `36-12`)

Choix faits à la rédaction de la fiche `36-12-qr-facture-separation.md` (2026-11-15, tête du worktree `fed34a9c7`,
= `origin/epic/36-0.18.0`), hors de ce que le fichier d'epic et C-e350-1 tranchent déjà (story à elle seule, rang 12 de
la phase 2, release 0.18.0 sans migration, après la première facture réelle de Guy). Chaque choix : contexte, option
retenue, options écartées, réversibilité.

### C-36-12-1 — Sept points de #944 dans la story, deux hors d'elle

- **Contexte** : #944 réunit neuf points (corps et quatre commentaires) ; le fichier d'epic en nomme quatre
  (« ciseaux, ligne de séparation, place de *Payable by*, largeurs »). Trois autres sont la conséquence directe de ces
  quatre : la largeur du message, la coupe calée sur les capitales (27 `W` sur le QR, 46 `W` hors de la feuille), et le
  point de dépôt recouvert — déplacer *Payable by* au-dessus du montant fait sinon déborder les longues adresses sur la
  section montant au lieu du point de dépôt. Deux points n'ont aucun chemin en production : le cadre vide du débiteur
  (`invoice_pdf_service.rs:384` `ultimate_debtor: Some(debtor.clone()),` — toujours un débiteur) et le type K (aucun
  chemin d'impression ne l'emploie, C-e3312-7 ; le lecteur de QR scannés, `parser.rs:206`, en a besoin).
- **Retenu** : les sept points dans la story ; le cadre vide du débiteur et le type K en deux issues neuves, demandées
  à l'orchestrateur ; `closes #944` à la PR de release une fois ces deux issues ouvertes.
- **Écartées** : (a) les quatre points nommés seuls — laisserait la story aggraver un débordement qu'elle crée ; (b) les
  neuf — deux chemins sans production, dont un (type K) touche le lecteur des factures fournisseur scannées.
- **Réversible** : oui.

### C-36-12-2 — Des ciseaux dessinés, sur chacune des deux lignes

- **Contexte** : § 3.7 de la directive : « Each of these lines must bear the scissors symbol ✂ or alternatively the
  instruction "Separate before paying in" above the line ». Helvetica intégrée en WinAnsi ne porte pas ✂
  (`pdf.rs:744`) ; printpdf pose `WinAnsiEncoding` sur toute police intégrée, ZapfDingbats comprise (printpdf 0.7,
  `src/font.rs:76`). La clé `invoice-pdf-qr-separate-before-paying` existe dans les quatre langues, jamais employée.
- **Retenu** : un symbole de ciseaux **dessiné en traits vectoriels** (deux anneaux, deux lames), posé sur la ligne
  horizontale près du bord gauche et sur la ligne verticale près du haut ; aucune police, aucune langue.
- **Écartées** : (a) la mention « A détacher avant le versement » : elle se place « au-dessus de la ligne » et ne sait
  pas porter la ligne verticale ; (b) le glyphe de ZapfDingbats : l'encodage imposé par printpdf rend son affichage
  incertain selon le lecteur ; (c) une police embarquée : c'est #943, hors de la story.
- **Réversible** : oui (rendu seul).

### C-36-12-3 — La grille se tire de la directive et des échantillons SIX mesurés

- **Contexte** : les figures 4, 6 et 7 de la directive n'ont pas de cotes ; le *Style Guide* de SIX n'est pas dans le
  dépôt. Les échantillons `docs/six-references/samples-payment-part-en/muster-qr-zahlteile-en/*.jpg` (150 dpi,
  210 × 105 mm) le sont : mesurés par `kesh-gate-logs/e36-12/mesure-echantillon-six.py` (Nr. 1, 2, 13 concordants).
- **Retenu** : les cotes **normatives** du texte (210 × 105, 62 / 148, QR 46 × 46, bordure et espaces de 5 mm, marges
  de 5 mm) sont exactes dans les tests ; les cotes **lues sur les échantillons** (haut des titres, haut de la section
  montant à 68,6 mm du haut, point de dépôt à 82,7 mm, colonne d'information à 118 mm) le sont à ± 1 mm.
- **Écartées** : (a) aller chercher le *Style Guide* sur le site de SIX : hors du dépôt, non versionné, non relisible
  par une revue ; (b) recopier les cotes d'une bibliothèque tierce : même défaut.
- **Réversible** : oui.

### C-36-12-4 — Couper par la largeur mesurée, table AFM d'Helvetica

- **Contexte** : `MAX_CARACTERES_RECU` (27) et `MAX_CARACTERES_PAIEMENT` (46) sont calés sur la largeur moyenne des
  capitales (`pdf.rs:795-810`) ; le doc-comment écrit lui-même que 27 `W` débordent de dix millimètres sur le QR.
- **Retenu** : une table des largeurs d'Helvetica et d'Helvetica-Bold (métriques AFM d'Adobe, `phvr8a.afm` et
  `phvb8a.afm` de TeX Live, chasses `WX` en 1/1000 em), indexée par le caractère Unicode que WinAnsi porte ; la coupe
  mesure chaque ligne en millimètres. Un caractère absent de la table compte pour la plus large de la table (borne,
  jamais sous-estimée).
- **Écartées** : (a) garder la coupe par caractères en baissant les bornes (19 et 32 pour des `W`) : couperait en deux
  presque toute adresse ordinaire ; (b) embarquer une police et en lire les métriques : #943.
- **Réversible** : oui ; la table sera à remplacer le jour où #943 embarque une autre police — écrit au doc-comment.

### C-36-12-5 — Une cascade écrite pour les adresses qui ne tiennent pas

- **Contexte** : *Payable by* entre dans la section d'information, et la section montant devient fixe : la place du
  récépissé est bornée (≈ 48 mm de haut). Le pire cas admis par la validation (nom 70, rue 70 + numéro 16, NPA 16,
  localité 35, pays étranger, deux adresses, message 140, faits du caractère le plus large) ne tient pas à 8 pt.
- **Retenu** : des étapes, appliquées seulement si la précédente ne tient pas, toutes permises par la directive :
  récépissé — R1 rue et numéro omis (§ 3.6.2, « omit the street name and building number »), R2 valeurs à 6 pt
  (§ 3.6.2, « minimum font size is 6 pt »), R3 noms ramenés à deux lignes terminées par « ... » (§ 3.6.2, « it can be
  truncated ») ; partie paiement — P1 message raccourci, « ... » en fin (§ 3.5.4), P2 noms à deux lignes. Une adresse
  ordinaire n'en déclenche aucune. Si rien ne tient, l'erreur `PdfGeneration` existante — un test prouve qu'aucune
  entrée admise par la validation ne l'atteint.
- **Écartées** : (a) refuser le PDF dès le premier débordement : bloquerait l'émission d'une facture pour une adresse
  valide ; (b) omettre la rue au récépissé toujours, comme l'échantillon Nr. 1 : change toutes les factures ordinaires
  sans nécessité ; (c) réduire toutes les tailles d'office.
- **Réversible** : oui.

### C-36-12-6 — Les tailles de police restent 6 / 8 pt (titres 11)

- **Contexte** : § 3.4 recommande 8 / 10 pt en partie paiement, et permet 6 à 10 pt, intitulés deux points sous les
  valeurs ; Kesh imprime 6 / 8.
- **Retenu** : inchangées. **Écartée** : 8 / 10 recommandé — un quart de place en moins dans la colonne d'information,
  et le pire cas ne tiendrait plus sans tronquer les adresses. **Réversible** : oui.

### C-36-12-7 — La section montant à une place fixe, la même au récépissé et en partie paiement

- **Contexte** : au récépissé, le montant suit les adresses (`pdf.rs:926` `y -= 8.0;`) ; en partie paiement il est à
  `qr_top - 52.0` (`:1045`), à 4,5 mm du QR. Les échantillons alignent les deux sections, à 68,6 mm du haut.
- **Retenu** : une ordonnée fixe, commune aux deux colonnes, au moins 5 mm sous le QR. Conséquence : sans référence, le
  montant ne remonte plus (le test `sans_reference_aucune_rubrique` change d'attente, dit au Dev Agent Record).
- **Écartée** : le montant qui suit le contenu : c'est lui qui laissait recouvrir le point de dépôt. **Réversible** : oui.
