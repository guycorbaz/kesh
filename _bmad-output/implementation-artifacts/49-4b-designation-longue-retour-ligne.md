# Story 49-4b : désignation longue entière sur le PDF de facture et d'avoir, caractères repliés ou signalés (#1363, #1400)

Status: backlog

*Fiche issue du découpage de la `49-4-designation-longue-pdf` (décision de Guy du 2026-11-20, sur la recommandation de
l'orchestrateur du 2026-11-19 et le finding V1-6 de la validation P1 de la 49-4 :
`/home/gcorbaz/devel/kesh-gate-logs/e494/validation-49-4-p1.md`). Rédigée le 2026-11-20 (Opus 5.5) à partir de la fiche de la 49-4
(ses D1, D2, D3, D6, D8, D9, ses AC1 à AC4, AC6, AC8, AC10, AC11 et leurs tests) ; **les findings de la validation P1 qui touchent ce
périmètre y sont reportés comme points à traiter** (§ *Points à traiter*), non remédiés : la fiche repasse une validation complète
avant le développement. Release visée **0.20.0** (coupure jeudi 2026-11-26 au soir) **si elle est close à la coupure** ; sinon
**#1363 et #1400 partent entières à la release suivante, jamais en deux fois** (règle « une issue, une release », décision de Guy du
2026-11-20). **Aucune migration.** Code lu à `a1877ef9f` (identique à `3379a971a` pour `crates/`). **Part de la 49-4a** (elle en
reprend le contrôle de capacité) : son développement commence après la fusion de la 49-4a dans `epic/49-types-de-lignes-facture`.
Statut `backlog` : fiche écrite, validation à venir, **questions ouvertes à Guy** (§ *Questions ouvertes*).*

**Issues** : `refs #1363`, `refs #1400` ; **aucun mot-clé de fermeture** dans un commit. #1110 est recoupée, non traitée.

## Story

En tant que Guy, qui émet de vraies factures et les remet sur papier, je veux que le PDF d'une facture et d'un avoir imprime la
**désignation entière** d'une ligne, sur plusieurs lignes au besoin, au lieu de la couper à 44 caractères suivis de « … » ; et qu'un
caractère que la police du PDF n'a pas ne **disparaisse plus en silence** — le signe moins et les traits d'union s'impriment en
trait d'union, les autres font refuser la validation en nommant la ligne et le caractère (**décision à confirmer par Guy**, Q2).

**Sans toucher aux pièces déjà émises dont le PDF est archivé** : il fait foi et ne change jamais. La dérogation à « une facture
d'avant se réimprime à l'identique » — pour les PDF que Kesh rend de nouveau — est écrite en D3, avec son périmètre exact, et reste
**à compléter** (V1-1, HIGH) et **à confirmer** pour le rappel et l'avoir (Q1, Q3).

## Ce qui se lit au code (`a1877ef9f`)

### Le rendu des lignes

- `crates/kesh-qrbill/src/pdf.rs:415` `fn draw_invoice_section(` : le seul dessinateur des lignes, partagé par la facture, le
  rappel (plancher `SEP_Y` = 105 mm) et l'avoir (plancher `CONTENT_FLOOR_NO_QR` = 15 mm, `pdf.rs:191`).
- `pdf.rs:574` `    let col_desc = left;`, `pdf.rs:575` `    let col_qty = left + 90.0;` : la description commence à 20 mm, la
  quantité à 110 mm, écrite à gauche. Une désignation dispose de 90 mm, moins un écart.
- `pdf.rs:667-668` `        layer.use_text(` / `            truncate_display(&line.description, 45),` : la désignation d'une
  ligne chiffrée est passée **brute** — ni `texte_imprime`, ni repli — et réduite, au-delà de 45 caractères, à 44 suivis de « … »
  (`pdf.rs:2006` `fn truncate_display(`). Aucune mesure de largeur : 45 capitales larges débordent dans la colonne de la quantité.
- `pdf.rs:1148` `fn texte_de_commentaire(description: &str) -> String {` et `pdf.rs:1182` `fn lignes_de_commentaire(` (49-1b) :
  sauts de ligne normalisés, blancs en espace, U+00AD retiré, **125 lettres repliées** (`crate::texte::repli`, `texte.rs:62`),
  **tout caractère sans largeur WinAnsi retiré en silence** (`pdf.rs:1164-1167`), coupé à `COMMENTAIRE_LARGEUR` (170 mm) par
  `pdf.rs:1208` `fn couper_ligne_a(`. `couper_ligne_a` découpe sur `split(' ').filter(|mot| !mot.is_empty())` (`pdf.rs:1208-1228`) :
  **tout enchaînement d'espaces devient une espace**, en tête, en fin et à l'intérieur (V1-1).
- `lignes_de_commentaire` rend **zéro** ligne pour une description sans rien d'imprimable (`pdf.rs:1176-1181`) (V1-7).
- `crates/kesh-qrbill/src/largeurs.rs:509` `pub(crate) fn largeur_mm(` : mesure AFM d'Helvetica ; un caractère sans entrée WinAnsi
  compte zéro et `lopdf` le retire à l'écriture (`largeurs.rs:496-505`) : c'est ce qui fait disparaître U+2212, U+2010, U+2011, le
  grec et le cyrillique d'une désignation (#1400).
- **Après la 49-4a** : `lignes_imprimees(&InvoiceLinePdf)` (1 pour une ligne chiffrée) et la géométrie partagée sont la seule source
  du compte du dessin et du contrôle à la validation (`controler_les_lignes`) ; `validate_invoice_handler` refuse en
  `400 INVOICE_LINES_DO_NOT_FIT_PDF`. **Ces noms sont ceux de la fiche de la 49-4a ; relire ceux du code fusionné à l'étape 0.**
- `routes/invoices.rs:507` `req.description.trim().nfc().collect()` : la saisie rogne les bords, **garde les espaces doubles
  intérieurs** ; les autres chemins d'écriture (restauration, `kesh-seed`, propositions) ne passent pas forcément par ce `trim`
  (V1-1).

### Les PDF qui se rendent de nouveau, et ceux qui ne bougent jamais

- `crates/kesh-api/src/routes/issued_invoice_pdf.rs:104` `    if is_frozen(&invoice) {` : une facture au PDF figé sert le fichier
  archivé ; `:238` `pub async fn refreeze(` rend de nouveau un fichier absent. Une facture validée **jamais téléchargée ni envoyée**
  se rend au premier téléchargement avec le code en fonction, puis se fige (l'envoi par courriel fige aussi).
- `crates/kesh-api/src/routes/credit_notes.rs:364-365` (`generate_credit_note_pdf`, citation corrigée par V1-10) : **le PDF d'un
  avoir n'est jamais figé**.
- `invoice_pdf_service.rs:213-217` (doc-comment de `render_document`) : **le PDF d'un rappel n'est jamais figé** (#502).
- Les tests avant / après de la 49-1b (`crates/kesh-qrbill/tests/rendu_avant_apres.rs`, fixtures
  `crates/kesh-qrbill/tests/fixtures/avant-0.20.0/` ; `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`, fixtures
  `crates/kesh-api/tests/fixtures/avant-0.20.0/`) portent S1 à S9 et les pièces HTTP a à d ; désignations courtes en caractères
  ordinaires, **sans espace double** (V1-1), sauf S7 (`rendu_avant_apres.rs:294-300`, 64 caractères, réduite en 0.19.0).

## Inventaire par le symptôme

### S1 — « un texte du PDF de facture est réduit et suivi de « … » » (#1363)

```
$ git grep -n "truncate_display(" -- crates/kesh-qrbill/src/pdf.rs | grep -v "fn truncate_display\|///"
pdf.rs:318 (numéro de client, META_MAX_CHARS)   pdf.rs:486 (identité de l'émetteur, IDENTITY_MAX_CHARS)
pdf.rs:668 (désignation d'une ligne chiffrée)   pdf.rs:745, :747 (libellés du bloc de rappel)
```

- `:668` : **dans le périmètre** (D1).
- `:318` : le **numéro de client** borné à `META_MAX_CHARS` (`pdf.rs:277-299`) ; le débordement des autres lignes du bloc meta est
  **#293** (citation corrigée par V1-10) ; `:486`, identité de l'émetteur : **#1229**. Données de la société et du contact,
  modifiables après la validation → hors périmètre.
- `:745`, `:747` : textes produits par Kesh, non saisis → hors périmètre.

### S3 — « un caractère saisi disparaît du PDF de facture sans avertissement » (#1400)

- `:668` désignation chiffrée et `:657` commentaire : **dans le périmètre** (D4).
- `:759` **conditions de paiement** (texte figé à la validation) : même défaut, **hors périmètre** (#1400 ne nomme que les lignes)
  → issue à ouvrir.
- `:486` identité, `:528` bloc meta : données de la société et du contact (#943, police embarquée) → hors périmètre.
- Noms et adresses (`:431`, `:440`, `:540`, `:543`) : `texte_imprime` replie déjà les 125 lettres ; le reste est au manuel
  (`user-manual.tex:1594`) ; NFD → #1411 → hors périmètre.

## Décisions

*(Reprises de la 49-4, numérotées à neuf ; les findings P1 qui les mettent en cause sont au § Points à traiter et ne sont pas
encore intégrés.)*

- **D1 — La désignation d'une ligne chiffrée se coupe, elle ne se tronque plus.** (Ex-D1.) Elle passe par **le même texte de
  ligne** que le commentaire (D4) et se coupe par `couper_ligne_a` à `DESIGNATION_LARGEUR = col_qty - col_desc - 2.0` = **88 mm**,
  à 9 pt. Une fonction, `lignes_de_texte(description, largeur)`, dont `lignes_de_commentaire` devient l'appel à
  `COMMENTAIRE_LARGEUR` (DRY). Quantité, prix, TVA et total s'écrivent **à la hauteur de la première ligne** ; chaque ligne de
  texte suivante prend un pas de **5 mm**. **`lignes_imprimees` de la 49-4a** devient, pour une ligne chiffrée,
  `max(1, lignes_de_texte(…, DESIGNATION_LARGEUR).len())` (V1-7) : le contrôle à la validation suit sans autre changement, et la
  phrase du refus `error-invoice-lines-do-not-fit-pdf` gagne « raccourcissez des désignations » (quatre catalogues). *Écartées* :
  (a) un pas réduit pour les lignes de suite ; (b) réduire la taille du texte ; (c) garder la troncature et avertir à la saisie.
- **D2 — Repli sur le rendu de la 0.19.0 pour une pièce qui ne tiendrait plus.** (Ex-D2.) Le rendu calcule d'abord le plan neuf ;
  s'il ne tient pas **et que toutes les lignes sont chiffrées**, la pièce se rend **exactement comme avant** (désignation brute,
  `truncate_display(…, 45)`, une ligne par ligne chiffrée — le code d'aujourd'hui, isolé dans une fonction nommée), avec un
  `tracing::warn!`. Sinon `TooManyLines`, comme aujourd'hui. La portée annoncée par la 49-4 (« seulement les pièces validées sans
  le contrôle ») est **fausse pour le rappel** (V1-2) : à reprendre selon Q1. *Écartées* : (a) refuser (une facture d'avant
  deviendrait impossible à imprimer) ; (b) choisir le rendu selon la date de validation (aucune colonne, migration interdite) ;
  (c) un repli ligne par ligne.
- **D3 — La dérogation, écrite.** (Ex-D3.) « Une facture déjà émise se réimprime à l'identique » reste vrai pour **toute facture
  dont le PDF est archivé** (`issued_invoice_pdf.rs` inchangé, fichier figé servi octet pour octet, T12 de la 49-1b). **Source de
  la dérogation** (citation corrigée par V1-4) : le commentaire de la clé 49-4 de `sprint-status.yaml` (« dérogation assumée … pour
  une facture jamais téléchargée ou un regel », admise par Guy le 2026-11-19) et la D8 de la 49-1d — **non** le fichier d'épique,
  dont la ligne `:130` dit « s'applique aux factures émises à partir de la 0.20.0 » : **écart au texte de l'épique**, à faire corriger
  par l'orchestrateur ; la consigne commune `e0200-commun.md` (« une facture validée jamais téléchargée, ou un regel, doit se rendre à
  l'identique ») est de même dépassée par l'admission de Guy. Elle vaut pour **une facture validée jamais téléchargée ni envoyée** et
  pour **un regel** ; l'étendre à **l'avoir** et au **rappel**, jamais figés, est **à confirmer par Guy** (Q1, Q3). Ce qui change sur
  ces chemins : (1) une désignation de plus de 45 caractères, ou plus large que 88 mm, s'imprime entière sur plusieurs lignes ; ce
  qui suit descend d'autant ; (2) U+2010, U+2011, U+2012, U+2212 s'impriment « - » ; une des 125 lettres s'imprime par sa base ;
  U+00A0 et tout blanc en espace ; un saut de ligne coupe ; U+00AD est retiré ; **(2-bis, V1-1, à trancher) les espaces de suite,
  de tête et de fin** ; (3) si le plan neuf ne tient pas, D2. **Ce qui ne change pas** : une désignation d'au plus 45 caractères,
  tenant dans 88 mm, en caractères WinAnsi, sans blanc autre que U+0020 — **et, selon le remède de V1-1, sans espace double ni de
  bord** — s'imprime au même dessin. Montants, TVA, écritures, numéros ne changent sur aucun chemin.
- **D4 — Caractères (#1400) : replier ce qui a un équivalent, refuser à la validation ce qui n'en a pas, ne rien perdre en
  silence.** (Ex-D6 ; **le refus est la question Q2**, non tranchée.) `texte_de_ligne` (la `texte_de_commentaire` renommée et
  étendue) sert la désignation chiffrée et le commentaire : en plus de ce qu'elle fait, elle replie la table close U+2010, U+2011,
  U+2012, U+2212 → « - » et retire la liste close des invisibles U+200B, U+200C, U+200D, U+2060, U+FEFF (liste à revoir, V1-12).
  Tout autre caractère sans glyphe est énuméré par `caracteres_sans_glyphe(description) -> Vec<char>` (sans doublon, ordre
  d'apparition), dérivée du même code que le filtre du rendu. **Proposition soumise à Guy (Q2)** : `validate_invoice_handler`
  refuse alors, **avant** le contrôle de capacité de la 49-4a, en `400 INVOICE_LINE_UNPRINTABLE_CHARACTERS` : *« Ligne { $ligne } :
  « { $caracteres } » ne s'impriment pas sur le PDF de la facture (la police ne les porte pas). Remplacez-les ou retirez-les ; la
  facture reste un brouillon. »*, chaque caractère nommé comme `CaractereRefuse` (`invoice_pdf_service.rs:646-650`), la **première**
  ligne fautive nommée par sa **position**. Le rendu, lui, continue de retirer un caractère sans glyphe (une pièce d'avant qui en
  porte s'imprime comme avant). *Écartées* : (a) une police embarquée (#943, change tout PDF) ; (b) le repli dans le seul
  commentaire (morcelle #1400) ; (c) refuser au `PUT` du brouillon ; (d) un avertissement non bloquant (contrat et écran neufs) ;
  (e) translittérer le grec ou le cyrillique.
- **D5 — La preuve avant / après reprend le mécanisme de la 49-1b, sans toucher à ses fixtures ni à son comparateur.** (Ex-D8.)
  `crates/kesh-qrbill/src/empreinte.rs` est **gelé** ; les fixtures `avant-0.20.0/` ne changent pas. (a) S1 à S6, S8, S9 et a à d
  restent identiques sans modification de test, sauf l'exclusion de **S7** de `neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0`
  (commentaire renvoyant à D3), comparée à sa fixture par un test dédié qui borne l'écart. (b) **Références neuves, capturées avant
  tout changement du rendu** — la base de capture est la tête de `epic/49-types-de-lignes-facture` **après la fusion de la 49-4a**,
  dont le rendu est celui de la 0.19.0 pour les lignes chiffrées (prouvé par l'AC4 de la 49-4a) : S10 (facture d'avant pleine), S11
  (« Remise −50 sur l'entretien Čolić »), S12 (avoir à désignations longues) — **longueurs à recalculer** (V1-9) —, un cas à espace
  double et à espace de bord (V1-1), un cas grec dans une pièce d'avant (V1-12), et au niveau HTTP (e) une facture de la forme S10 et
  (f) une facture à désignation de 64 caractères, validées **par le dépôt**. Graphe : commit C' (tests de capture, aucune ligne de
  `RENDU`), worktree détaché à C' sous `kesh-gate-logs/e494b/capture/` (`KESH_REFERENCES=ecrire` puis `comparer`), commit F' des
  seules fixtures dans des dossiers **neufs** `…/fixtures/avant-0.20.0-49-4b/`, **puis** le code. `RENDU` =
  `crates/kesh-qrbill/src/{pdf,types,largeurs,texte,generator,validation,parser}.rs` et `crates/kesh-api/src/`.
- **D6 — Le manuel reprend les phrases de la 49-1c après sa fusion.** (Ex-D9.) `user-manual.tex:1456` (désignation et commentaire
  entiers, chaque ligne imprimée prend une place) et `:1594` (le moins et les traits d'union s'impriment, les 125 lettres dans la
  désignation, le refus d'un caractère sans glyphe selon Q2, conseil du trait d'union **retiré**, espaces multiples selon V1-1), et
  une phrase « depuis la version~0.20.0 » qui dit la dérogation en mots d'utilisateur. Après la fusion de la 49-1c et de la 49-4a ;
  sinon le second arrivé réconcilie.
- **D7 — Aucune ligne de `kesh-db`, aucune écriture en base, aucun changement de l'API de création, aucun changement de l'écran.**
  (Ex-D7, D10.) Le dialogue de validation affiche le message du serveur (`+page.svelte:448`) ; aucun fichier de `frontend/src` ne
  change.

## Réponses de Guy du 2026-11-20

- **Q1 (a) et (b)** : « on imprime le rappel avec le nouveau rendu ». La dérogation s'étend au rappel ; **pas de repli « comme en
  0.19.0 »** (désignations abrégées) sur un rappel. Conséquence à écrire en D2 et au manuel : un rappel qui ne tient pas avec le
  nouveau rendu est refusé comme aujourd'hui (`TooManyLines`), à confirmer par Guy si la fiche y voit une autre issue.
  **Q1 (c)** : « oui » (2026-11-20) — **réserver** le bloc de rappel à la validation des pièces à commentaires ; c'est la 49-4a
  qui le fait (sa D2).
- **Q2** : oui — la validation est refusée en nommant la ligne et le caractère (proposition de D4).
- **Q3** : oui — la dérogation s'étend à l'avoir.

## Questions ouvertes à Guy (non tranchées)

- **Q1 — Le rappel.** (V1-2, V1-5.) Trois volets, liés : (a) **la dérogation s'étend-elle au rappel ?** Un rappel n'est jamais
  figé : tout rappel, même d'une facture d'avant, se rendra avec le code neuf (désignation entière) ; (b) **une facture née en
  0.20.0, toutes lignes chiffrées, qui tient juste** : son rappel, avec le bloc du rappel, ne tient plus ; D2 le rend alors « comme
  en 0.19.0 » — **désignations abrégées à 44 caractères et « … » sur le rappel alors que la facture les porte entières** —, ou bien
  le rappel est refusé comme aujourd'hui (`TooManyLines`) ; (c) **une facture à commentaires qui tient juste** (49-1c avec la 49-4a) :
  son rappel est refusé (pas de repli possible), avec la phrase de #1110 ; faut-il **réserver le bloc de rappel** dans le contrôle
  de la validation pour les seules pièces à commentaires (environ trois lignes de moins), ou l'accepter et le dire au manuel ?
  *Le volet (c) touche la sortie de la 49-4a avec la 49-1c* : si Guy tranche « réserver » avant la clôture de la 49-4a, elle le fait
  (un paramètre de `controler_les_lignes`) ; sinon la 49-4b.
- **Q2 — Le refus d'un caractère non imprimable.** Refuser la validation d'une facture dont une ligne porte un caractère que la
  police n'a pas (grec, cyrillique, flèches, ≤ ≥, émojis, CJK, et les invisibles hors de la liste close de D4 : U+200E, U+200F,
  U+202A à U+202E…), en nommant la ligne et le caractère (proposition de D4) ; ou l'une des voies écartées : avertir sans bloquer
  (contrat d'API et écran neufs), retirer en silence comme aujourd'hui en le disant au manuel, police embarquée (#943).
- **Q3 — L'avoir** (V1-4). Admise pour « une facture jamais téléchargée ou un regel », la dérogation s'étend-elle à **l'avoir**,
  jamais figé (tout avoir d'une facture d'avant se rendra avec la désignation entière) ? La 49-1d (D8, AC14) le tient déjà pour
  acquis ; la fiche de la 49-4 aussi, sans l'avoir demandé.

## Points à traiter (findings de la validation P1 de la 49-4, reportés)

À intégrer à la fiche avant sa validation, prémisse relue au code (rapport :
`/home/gcorbaz/devel/kesh-gate-logs/e494/validation-49-4-p1.md`) :

- **V1-1 — HIGH — espaces doubles, de tête ou de fin.** `couper_ligne_a` réduit tout enchaînement d'espaces à une : une
  désignation « Loyer  avril » (deux espaces), imprimée telle quelle en 0.19.0, s'imprimerait avec une seule — rendu changé
  **hors de la liste de D3**, sur une facture jamais téléchargée, un regel, un avoir ou un rappel. Aucune pièce de la 49-1b n'en
  porte : les tests avant / après resteraient verts. **Deux remèdes** : (a) ajouter le cas à la liste de D3 (2-bis), au manuel
  (`:1594`) et à la phrase de dérogation ; (b) faire que `lignes_de_texte` rende **telle quelle** (repli des caractères appliqué,
  blancs intérieurs conservés) une désignation d'un seul paragraphe qui tient sur une ligne, et ne coupe qu'au besoin.
  **Recommandation de la rédaction : (b)**, qui garde au même dessin toute désignation courte d'une pièce existante ; à trancher à
  la reprise. Dans les deux cas : un cas à espace double et à espace de bord dans les références (D5), rouge prouvé ; et dire à la
  49-1d (D8) que sa règle des désignations « inchangées » l'exclut ou l'inclut.
- **V1-2 — MEDIUM — portée de D2 inexacte (rappel)** : corriger la phrase, ajouter un cas rappel aux tests du repli, selon Q1.
- **V1-3 — MEDIUM (moitié T-SRC)** — le test « `caracteres_sans_glyphe` ⇔ `texte_de_ligne` » est vrai par construction (même
  table). Remède : un oracle indépendant — le texte relu du PDF rendu (`empreinte::textes`), ou l'encodage WinAnsi de `lopdf`
  (`lopdf::encodings::string_to_bytes`). Et rendre le repli de D2 **observable** (valeur de retour privée `Plan { Neuf, Repli }`
  lisible des tests), un `warn!` ne s'assertant pas. (La moitié T-PROP est remédiée par la 49-4a, D1.)
- **V1-4 — MEDIUM — source de la dérogation** : intégré à D3 (source citée, écart à l'épique `:130` déclaré) ; la confirmation de
  l'avoir et du rappel est Q1 (a) et Q3 ; la correction de `epic :130` revient à l'orchestrateur.
- **V1-7 — LOW — désignation sans rien d'imprimable** : « au moins une ligne (vide) par ligne chiffrée », intégré à D1
  (`max(1, …)`), un cas au test de la désignation entière.
- **V1-9 — LOW — S10 et S12 mal dimensionnées** : un avoir tient environ 26 lignes imprimées sans récapitulatif ; S12 ne déborde
  que si ses trois désignations longues font environ 300 caractères chacune ; S10 à 70 caractères de mots courants. Écrire
  l'arithmétique des deux seuils dans la fiche.
- **V1-10 — LOW — citations** : `credit_notes.rs:364-365` et `pdf.rs:318` / #293 corrigés ci-dessus ; `types.rs:196` est la doc de
  `LinePdfKind::Priced`, non d'`InvoiceLinePdf` — relire la phrase « au-delà de 45 caractères, réduite à 44 et « … » » à sa vraie
  place avant de la réécrire.
- **V1-12 — LOW — renvois** : un cas grec ou cyrillique dans une pièce d'avant (le rendu retire, ne refuse pas) ; la mutation du test
  HTTP du repli doit porter sur le repli dans `kesh-qrbill`, non sur `render_and_pose` ; la liste close des invisibles (U+200E,
  U+200F, U+202A à U+202E, U+2061 à U+2064, U+180E) à élargir ou à assumer au manuel (lié à Q2).
- **V1-13** (conditions de paiement au seuil) : traité par la 49-4a (D6, capture) ; la coupe des désignations fait que plus de
  pièces atteignent ce seuil — reprendre la capture de la 49-4a avec une désignation coupée.

## Critères d'acceptation

*(Ceux de la 49-4, à reprendre avec les points à traiter et les réponses de Guy.)*

- **AC1 — Une désignation longue s'imprime entière.** Facture, avoir et rappel : une ligne chiffrée de 300 caractères (mots courts
  et un mot de 60 `W`) s'imprime sans « … » ; (1) texte sans blancs après `texte_de_ligne` = concaténation sans blancs des lignes
  dessinées à `x = 20` mm ; (2) chaque ligne ≤ 88 mm (`largeur_mm(ligne, 9.0, false)`) ; (3) aucun texte de la désignation à
  `x ≥ 108` mm ; (4) montants à l'ordonnée de la **première** ligne ; (5) la ligne suivante descend de `n × 5` mm, `n ≥ 1` (V1-7).
- **AC2 — Ce qui doit rester identique reste identique.** Tests de la 49-1b verts (S7 seule exclue, avec un commentaire qui renvoie
  à D3) ; `git diff` des fixtures `avant-0.20.0/`, d'`empreinte.rs` et de `pdf_avant_apres_e2e.rs` **vide** ; plus le cas d'espace
  double et de bord selon V1-1.
- **AC3 — La dérogation est bornée et prouvée.** S7 et S11 rendus par le code neuf : `empreinte::textes` égale la référence hors
  (i) les textes de la désignation modifiée et (ii) les ordonnées de ce qui suit, décalées d'exactement `(n − 1) × 5` mm ; polices,
  `MediaBox`, pages égaux. Au niveau HTTP, (f) par `GET /pdf` d'une facture validée jamais téléchargée, puis par le regel ; un PDF
  **figé** servi octet pour octet (T12 de la 49-1b).
- **AC4 — Une pièce d'avant qui ne tiendrait plus se rend comme en 0.19.0.** S10 et S12 égaux à leur référence ; (e) par `GET /pdf`
  et regel ; le repli **observable** (V1-3) ; une pièce à commentaire qui ne tient pas rend `TooManyLines`
  (`invoice_pdf_e2e.rs:1032`, vert sans modification) ; le rappel selon Q1.
- **AC5 — Le contrôle de la 49-4a suit la coupe.** Une facture d'une ligne chiffrée de 1000 caractères est refusée à la validation
  (`INVOICE_LINES_DO_NOT_FIT_PDF`) ; sept lignes dont une coupée en deux, arrondi actif et non nul, refusée ; la même arrondi
  désactivé, acceptée ; l'équivalence contrôle ⇔ rendu (test de la 49-4a) reste verte, sa famille élargie aux désignations longues
  et le repli de D2 exclu par son observable.
- **AC6 — Caractères.** (a) Rendu : les quatre tirets → « - » dans une désignation et un commentaire ; les 125 lettres dans une
  désignation ; les invisibles retirés. (b) Validation, **selon Q2** : « Ωmega — проект » en ligne 2 → `400
  INVOICE_LINE_UNPRINTABLE_CHARACTERS` nommant la ligne 2, « Ω » (U+03A9), « п » (U+043F)… dans l'ordre, sans doublon, sans « — » ;
  rien d'écrit ; « Remise −50 » validée, PDF « Remise -50 ». (c) **Source unique, oracle indépendant** (V1-3).
- **AC7 — Textes et contrat.** Phrase de `error-invoice-lines-do-not-fit-pdf` étendue aux désignations ; selon Q2, un code neuf,
  sa clé ×4 et `textes_coherents.rs` ; `docs/api-external.md` (`:520`, `:1281`) ; `admin-manual.tex` `sec:maj-0-20-0`.
- **AC8 — Manuel utilisateur** (D6) : contrôles `git grep -n "sans avertissement"` (plus de phrase sur les lignes),
  `git grep -n -i "trait d'union ordinaire"` **vide**, `git grep -n "44 caract"` (seule la phrase de la dérogation) ; PDF régénéré
  localement, aplati, relu.
- **AC9 — E2E.** Le fichier `frontend/tests/e2e/invoice-validation-pdf.spec.ts` de la 49-4a gagne : une désignation de 1000
  caractères refusée ; selon Q2, « Ωmega » refusée en nommant la ligne et « Ω » ; après correction, validation et PDF `200`.
- **AC10 — Recette visuelle** : PDF d'une facture à désignation longue coupée, d'une facture à « Remise -50 », d'un avoir à
  désignation longue, d'une facture courte inchangée en témoin, du dialogue sur le refus des caractères (selon Q2) ; base
  `kesh_e2e_e494b`, captures dans `kesh-gate-logs/e494b/captures/`, chacune lue.
- **AC11 — Factures et avoirs existants inchangés hors de D3.** `git diff` vide sur `crates/kesh-db`, `crates/kesh-core`,
  `issued_invoice_pdf.rs`, `invoice_pdf.rs` ; ce qui change d'une pièce d'avant est **exactement** la liste de D3, prouvé par AC2 à
  AC4 (test du `git diff` : script en lecture seule, comme la 49-4a, V1-12).
- **AC12 — Coordination.** Ni `largeurs.rs`, ni `kesh-report/`, ni `CHANGELOG.md`, ni `frontend/src/` ; `lib.rs` : ré-export de
  `caracteres_sans_glyphe` seulement, hunk distinct de #1327.
- **AC13 — Gates.** Comme la 49-4a, avec les binaires `rendu_designations`, `pdf_designations_e2e` en plus.

## Tests à écrire, et la preuve du rouge

*(Ceux de la 49-4, à reprendre avec les points à traiter.)* Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e494b/`.

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T0 | `crates/kesh-qrbill/tests/rendu_designations.rs::capturer_references_avant_49_4b` (`#[ignore]`, `KESH_REFERENCES=ecrire\|comparer`) | S10, S11, S12, le cas d'espaces (V1-1), le cas grec (V1-12), écrits dans le worktree détaché à C' | `ecrire` puis `comparer`, stables, `git status --porcelain` vide |
| T0-bis | `crates/kesh-api/tests/pdf_designations_e2e.rs::capturer_references_http_avant_49_4b` | (e), (f) validées par le dépôt, `GET /pdf` puis regel | idem |
| T1 | `pdf.rs::tests::une_designation_longue_s_imprime_entiere` (AC1) | les cinq invariants, facture, avoir, rappel ; `n ≥ 1` | (1) troncature rétablie ; (2) largeur 88 → 110 ; (3) montants à la dernière ligne ; (4) `max(1, …)` retiré, désignation « 😀 » → **ROUGE** |
| T2 | tests de la 49-1b (AC2) | S1 à S6, S8, S9, a à d identiques ; cas d'espaces selon V1-1 | une espace de fin ajoutée à chaque ligne → **ROUGE** sur S1 ; espaces intérieurs réduits → **ROUGE** sur le cas V1-1 |
| T3 | `rendu_designations.rs::s7_et_s11_ne_changent_que_la_designation` (AC3) | écart borné | décalage `n × 5` ; repli U+2212 retiré |
| T4 | `rendu_designations.rs::une_piece_d_avant_pleine_se_rend_comme_en_0_19_0` (AC4) | S10, S12 égaux, repli observable | repli supprimé ; repli qui applique `texte_de_ligne` |
| T5 | `pdf_designations_e2e.rs::designation_longue_par_le_http` (AC3 f, AC4 e) | (e), (f) par `GET /pdf` et regel | mutation **dans le repli de `kesh-qrbill`** (V1-12) → **ROUGE** sur (e) |
| T6 | `invoice_validation_pdf_e2e.rs` de la 49-4a, étendu (AC5) | la coupe comptée au contrôle | `lignes_imprimees` d'une ligne chiffrée remis à 1 → **ROUGE** |
| T7 | `invoice_validation_pdf_e2e.rs::la_validation_nomme_les_caracteres_sans_glyphe` (AC6 b, selon Q2) | refus nommant ligne et caractères, rien d'écrit | contrôle retiré ; U+2014 compté ; index 0 au lieu de la position 1 |
| T-SRC | `pdf.rs::tests::caracteres_sans_glyphe_et_le_pdf_disent_la_meme_chose` (AC6 c) | équivalence contre un **oracle indépendant** (V1-3) sur U+0000–U+FFFF hors surrogats, plus deux émojis | une table WinAnsi réécrite qui omet U+20AC → **ROUGE** ; un repli ajouté sans que le PDF le porte → **ROUGE** |
| T8 | `pdf.rs::tests::repli_des_tirets_et_invisibles` (AC6 a) | tirets, 125 lettres, invisibles | U+2011 retiré de la table |
| T9 | E2E (AC9) | dans le navigateur | binaire sans le contrôle |
| T10 | épreuve : `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`, `dunning_reminders_e2e`, `golden_test`, `invoice_line_types_e2e` | verts sans modification d'assertion ; tout test changé se liste avec sa raison | — |

## Tâches

- [ ] **Étape 0** — reprendre la fiche : réponses de Guy à Q1 à Q3, points à traiter intégrés, validation (passe complète). Puis
  relire le code après la fusion de la 49-4a (noms réels de `lignes_imprimees`, de la géométrie et du contrôle) et de la 49-1c.
- [ ] **Capture** (D5) : C', worktree détaché, F', contrôles du graphe (`kesh-gate-logs/e494b/controles-graphe.sh`, modèle
  `kesh-gate-logs/e491b/ac1-controles.sh`).
- [ ] **`kesh-qrbill`** : `texte_de_ligne`, `caracteres_sans_glyphe`, `lignes_de_texte`, la coupe de la désignation, `lignes_imprimees`
  étendue, le repli de D2 isolé et observable ; doc-comments devenus faux (`draw_invoice_section`, `pdf.rs:400-403` ; la phrase
  « réduite à 44 et « … » », V1-10) ; `README.md` de la crate (`:20`) ; ré-export.
- [ ] **`kesh-api`** : selon Q2, le contrôle des caractères dans `validate_invoice_handler`, avant celui de la 49-4a, et sa variante
  d'`AppError`.
- [ ] **`kesh-i18n`** : phrase du refus de capacité étendue ×4 ; selon Q2, une clé ×4 ; `textes_coherents.rs`.
- [ ] **Tests**, **documents** (D6, AC7, AC8), **E2E et captures**, **gates** et **contrôle des mots-clés** : comme la 49-4a.

## Note de version

*(Texte pour le `CHANGELOG`, à reprendre selon Q1 à Q3.)*

- **Factures et avoirs — désignation entière sur le PDF** : la désignation d'une ligne s'imprime entière, sur plusieurs lignes au
  besoin, au lieu d'être coupée à 44 caractères suivis de « … » ; le signe moins (U+2212) et les traits d'union U+2010 à U+2012
  s'impriment en trait d'union, et les lettres comme Č ou ł par leur lettre de base, au lieu de disparaître (`refs #1363`,
  `refs #1400`).
- **Factures — caractères refusés à la validation** *(selon Q2)* : une ligne qui porte un caractère que la police du PDF n'a pas
  (grec, cyrillique, émoji…) est refusée à la validation en nommant la ligne et le caractère
  (`400 INVOICE_LINE_UNPRINTABLE_CHARACTERS`) (`refs #1400`).
- **Factures déjà émises** : un PDF archivé n'est jamais réécrit. Une facture validée avant la 0.20.0 et jamais téléchargée, un PDF
  regelé *(et, selon Q1 et Q3, un avoir et un rappel)* s'impriment désormais avec la désignation entière ; s'ils ne tiennent plus
  sur la page, ils s'impriment comme avant. Les montants, la TVA et les écritures ne changent pas.

## Dev Notes

- **Règle de sortie** : si la 49-4b n'est pas close à la coupure du jeudi 2026-11-26 au soir, **#1363 et #1400 partent entières** à
  la release suivante ; rien de leur traitement n'entre en 0.20.0 (pas de repli des tirets seul, pas de coupe sans les caractères).
  La 49-1c, elle, ne dépend que de la 49-4a.
- **Hors périmètre, avec ce qui le porte** : identité de l'émetteur → #1229 ; bloc meta → #293 ; numéro de facture et jeu de la QR
  → #1046 ; police embarquée → #943 ; NFD des contacts → #1411 ; PDF sur plusieurs pages → #1374 ; phrase du refus au rendu → #1110 ;
  montants au-delà du filet → #1399 ; titre de section → 49-3 (passera par `lignes_imprimees`).
- **Issues à ouvrir** (une par problème) : *PDF de facture : les conditions de paiement perdent sans avertissement les caractères
  que la police n'a pas* (`pdf.rs:759`), P3.
- **Cohérence avec la 49-1d (sa D8)** : elle écrit « au même dessin qu'en 0.19.0 pour des désignations de 44 caractères au plus ».
  La règle exacte (D3, à compléter par V1-1) est : au plus 45 caractères, tenant dans 88 mm, en caractères WinAnsi, sans blanc
  autre que l'espace, ni tiret de D4, ni lettre des 125 (et sans espace double ni de bord, selon V1-1). Les désignations de F2 à F8
  et de A1 de sa recette doivent la suivre ; sa phrase du manuel d'administration et sa note de version nomment les chemins de D3
  confirmés par Guy. **À porter par l'orchestrateur à la fiche de la 49-1d.**
- **Fichiers communs** : `pdf.rs` (49-4a avant, 49-3 après) ; `routes/invoices.rs` (49-1c, 49-3, 49-4a — même handler) ;
  `errors.rs` (48-1, 48-2, 49-4a) ; `docs/api-external.md` (48-1, 49-1c, 49-3, 49-4a) ; `admin-manual.tex`, `user-manual.tex` (48-1,
  49-1a, 49-1c, 49-1d, 49-4a) ; catalogues ×4. PR #1327 de Beat : `lib.rs` (hunk voisin), `README.md` de la crate (lignes 6-7 contre
  la ligne 20), `largeurs.rs` (non touché ici).
- **Splitting** : `kesh-qrbill`, `kesh-api`, `kesh-i18n`, documents — quatre modules, plus des tests. Sous le plafond.
- **Charge** (estimation) : reprise de la fiche et validation une demi-journée à un jour (hors délai de réponse de Guy) ; capture
  une demi-journée ; `kesh-qrbill` (texte de ligne, coupe, repli, caractères) un jour et demi ; `kesh-api` et i18n un quart de
  journée à une demi-journée ; tests et mutations un jour ; manuel, E2E, captures une demi-journée à un jour ; gates une
  demi-journée — **quatre à cinq jours** plus la boucle de revue. **Improbable pour la coupure du 2026-11-26** si la 49-4a passe
  d'abord ; la release suivante est le cas probable.
- **Ce qui doit rester vrai** : (1) un PDF archivé n'est jamais réécrit ; (2) aucune pièce d'avant ne devient impossible à
  imprimer (D2) ; (3) les pièces des tests de la 49-1b hors S7 se rendent à l'identique ; (4) aucune ligne de `kesh-db`.
- **Pièges** : (a) le repli de D2 rend le code d'aujourd'hui sans « amélioration » ; (b) la ligne nommée par le refus est la
  **position** (1, 2…), comme `validate_line` ; (c) `kesh-seed` valide par le dépôt (`pieges.rs:590`) : ses factures passent par D2
  si elles ne tiennent pas ; (d) un caractère hors du plan de base (émojis) est un `char` comme un autre : T-SRC balaie le plan de
  base, un test nommé ajoute deux émojis.
- **Aucune donnée réelle** de Guy ; `~/kesh/` ni lu ni écrit.
- **Références** : fiche d'origine `49-4-designation-longue-pdf.md` (statut `split`, texte complet des décisions D1 à D10 d'avant le
  découpage) ; fiche sœur `49-4a-capacite-pdf-avant-validation.md` ; validation P1 de la 49-4 ; `epic-49-types-de-lignes-facture.md`
  (`:130`) ; `49-1b-types-de-lignes-pdf.md` (D4 à D8) ; `49-1c-types-de-lignes-ecran.md` (D10, D12) ; `49-1d-…` (D8) ; #1363, #1400,
  #1110, #1327 ; `kesh-gate-logs/prompts/e0200-commun.md`.

## Dev Agent Record

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-20 | **Création par découpage de la 49-4** (Opus 5.5 ; décision de Guy du 2026-11-20). Reprend D1, D2, D3, D6, D8, D9 de la 49-4 (renumérotées D1 à D6), ses AC1 à AC4, AC6, AC8, AC10, AC11 et leurs tests ; s'appuie sur le contrôle de capacité de la 49-4a. Findings de la validation P1 **reportés comme points à traiter** : V1-1 (HIGH, espaces doubles ou de bord ; recommandation (b)), V1-2, V1-3 (moitié T-SRC), V1-4 (source de la dérogation intégrée à D3), V1-7 (intégré à D1), V1-9, V1-10 (citations corrigées), V1-12. Trois questions ouvertes à Guy : Q1 le rappel (dérogation, repli, réserve du bloc — son volet (c) touche la sortie de la 49-4a), Q2 le refus d'un caractère non imprimable, Q3 l'avoir. Sortie en 0.20.0 seulement si close le jeudi 2026-11-26 ; sinon #1363 et #1400 entières à la release suivante. Statut `backlog` : reprise, réponses de Guy et validation à venir. |
