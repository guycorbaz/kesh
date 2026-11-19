# Story 49-4 : désignation longue sur le PDF de facture et d'avoir, capacité contrôlée à la validation, caractères signalés (#1363, #1364, #1400)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-4 (Opus 5.5), workflow `bmad-create-story`. Release **0.20.0**
(coupure jeudi 2026-11-26 au soir). **Aucune migration.** Code lu à la tête de la branche `story/49-4-designation-longue-pdf` =
`3379a971a` (49-1a et 49-1b fusionnées dans `epic/49-types-de-lignes-facture` ; la 49-1c ne l'est pas encore). Sources :
`epic-49-types-de-lignes-facture.md` (ligne 49-4 du tableau des stories, règles de sortie, § *Contraintes de production*),
`release-0.20.0.md`, la fiche close `49-1b-types-de-lignes-pdf.md`, la fiche de la 49-1c lue sur sa branche
(`story/49-1c-types-de-lignes-ecran`, `7cc2bf0d5`), la fiche de la 49-1d lue dans son worktree (`16bdaacda`, D8), la PR #1327
de Beat (`gh pr diff 1327`), les issues #1363, #1364 (arbitrage de Guy du 2026-11-19 en commentaire), #1400, #1110. Les choix de
cette fiche sont D1 à D10.*

**Issues** : `refs #1363`, `refs #1364`, `refs #1400` ; **aucun mot-clé de fermeture** dans un commit (il ne s'écrit que dans le
corps de la PR de la release). #1110 est recoupée, non traitée (D5).

## Story

En tant que Guy, qui émet de vraies factures et les remet sur papier, je veux que le PDF d'une facture et d'un avoir imprime la
**désignation entière** d'une ligne, sur plusieurs lignes au besoin, au lieu de la couper à 44 caractères suivis de « … » ; qu'une
facture dont les lignes ne tiendraient pas sur la page soit **refusée à la validation**, quand le brouillon se corrige encore,
et non après ; et qu'un caractère que la police du PDF n'a pas ne **disparaisse plus en silence** — le signe moins et les traits
d'union s'impriment en trait d'union, les autres font refuser la validation en nommant la ligne et le caractère.

**Sans toucher aux pièces déjà émises** : le PDF archivé d'une facture fait foi et ne change jamais ; une facture d'avant la
0.20.0 qui ne tiendrait plus avec le rendu neuf s'imprime **comme en 0.19.0**, jamais refusée. La dérogation à « une facture
d'avant se réimprime à l'identique » est écrite en D3, avec son périmètre exact.

Cette story est **la condition de sortie de la 49-1c** (fichier d'épique, règle « la 49-1c ne sort pas sans la 49-4 ») : sans
elle, un commentaire long rendrait une facture validée impossible à imprimer.

## Ce qui se lit au code (`3379a971a`)

Chaque ligne citée est copiée du fichier, lue à la tête du worktree.

### Le rendu des lignes

- `crates/kesh-qrbill/src/pdf.rs:415` `fn draw_invoice_section(` : le seul dessinateur des lignes, partagé par la facture, le
  rappel (plancher `SEP_Y` = 105 mm) et l'avoir (plancher `CONTENT_FLOOR_NO_QR` = 15 mm, `pdf.rs:191`).
- `pdf.rs:548` `    let mut ty = PAGE_H - 130.0;` : **l'ordonnée du tableau est une constante**, que l'en-tête ne repousse jamais
  (le commentaire `pdf.rs:550-566` le dit ; un en-tête trop haut rend `HeaderOverflow`). La capacité du tableau ne dépend donc
  **que** des lignes, du récapitulatif et du bloc de rappel, jamais de l'en-tête : elle se calcule sans contact ni compte bancaire.
- `pdf.rs:574` `    let col_desc = left;`, `pdf.rs:575` `    let col_qty = left + 90.0;` : la description commence à 20 mm, la
  quantité à 110 mm, **écrite à gauche** à cette abscisse. Une désignation dispose donc de 90 mm, moins un écart.
- `pdf.rs:631` `    let recap = recap_lines(inv, i18n);` et `pdf.rs:644` `    let plancher_des_lignes = content_floor + 15.0 + recap_reserve;` :
  la réserve du récapitulatif (sous-total, une ligne par taux, arrondi) et du bloc de rappel. `pdf.rs:1894` `fn recap_lines(inv:
  &InvoicePdfData, i18n: &QrBillI18n) -> Vec<RecapLine> {` décide le nombre de lignes du récapitulatif à partir de `vat_lines`,
  `subtotal_ht` et `rounding` (au centime, `pdf.rs:1898-1903`).
- `pdf.rs:653` et `pdf.rs:665` `return Err(QrBillError::TooManyLines(inv.lines.len()));` : la garde, avant **chaque** ligne
  dessinée (ligne chiffrée, ou ligne de texte d'un commentaire depuis la 49-1b, D6) ; pas de 5 mm.
- `pdf.rs:667-668` `        layer.use_text(` / `            truncate_display(&line.description, 45),` : la description d'une
  ligne chiffrée est passée **brute** — ni `texte_imprime`, ni repli — et réduite, au-delà de 45 caractères, à 44 suivis de
  « … » (`pdf.rs:2006` `fn truncate_display(s: &str, max_chars: usize) -> String {`). Aucune mesure de largeur : 45 capitales
  larges débordent dans la colonne de la quantité.
- `pdf.rs:1148` `fn texte_de_commentaire(description: &str) -> String {` et `pdf.rs:1182` `fn lignes_de_commentaire(description:
  &str) -> Vec<String> {` (49-1b) : le texte d'un commentaire — sauts de ligne normalisés, blancs en espace, U+00AD retiré,
  **125 lettres repliées** (`crate::texte::repli`, `texte.rs:62` `pub(crate) fn repli(c: char) -> Option<&'static str> {`),
  **tout caractère sans largeur WinAnsi retiré en silence** (`pdf.rs:1164-1167`, filtre `largeur_mm(…) > 0.0`) —, coupé à
  `COMMENTAIRE_LARGEUR` (170 mm) par `pdf.rs:1208` `fn couper_ligne_a(ligne: &str, largeur: f32, taille: f32) -> Vec<String> {`.
- `crates/kesh-qrbill/src/largeurs.rs:509` `pub(crate) fn largeur_mm(texte: &str, taille: f32, gras: bool) -> f32 {` : la mesure
  AFM d'Helvetica ; un caractère sans entrée WinAnsi compte zéro, et `lopdf` le retire à l'écriture (`largeurs.rs:496-505`).
  C'est ce qui fait disparaître U+2212, U+2010, U+2011, le grec et le cyrillique d'une désignation (#1400) : la police n'a pas
  de glyphe, et rien ne le dit.

### Le refus de capacité, aujourd'hui après la validation

- `crates/kesh-api/src/routes/invoices.rs:931` `pub async fn validate_invoice_handler(` appelle directement
  `invoices.rs:940` `        invoices::validate_invoice(&state.pool, company.id, id, current_user.user_id)` : **aucun contrôle
  du PDF** à la validation. C'est le **seul** appelant de production par une route (`git grep -n "validate_invoice(" --
  crates/*/src` : `kesh-api/src/routes/invoices.rs:940` et `kesh-seed/src/pieges.rs:590`, la démonstration).
- `crates/kesh-api/src/routes/invoice_pdf_service.rs:95` `pub const MAX_LINES_PER_PDF: usize = 9;` et `:235`
  `    if lines.len() > MAX_LINES_PER_PDF {` (pré-filtre grossier) ; `:542`
  `        QrBillError::TooManyLines(n) => return AppError::InvoiceTooManyLinesForPdf(*n),` : le refus tombe au rendu, sur une
  facture déjà validée (#1364).
- `crates/kesh-api/src/errors.rs:2310` `pub(crate) fn message_trop_de_lignes_pdf(n: usize) -> String {` : la phrase « Réduisez le
  nombre de lignes ou scindez la facture », impossible à suivre sur une facture validée (#1110).
- **Nuance à #1364** : une facture validée **se dévalide** (`crates/kesh-db/src/repositories/invoices.rs:1637` `pub async fn
  unvalidate(`, Story 25-2-b-1), sauf réglée, créditée, rappelée, envoyée par courriel, rapprochée ou sur un exercice clos
  (`invoices.rs:1617-1620`). Une facture qui n'a jamais pu s'imprimer n'a été ni envoyée ni rappelée : elle se dévalide en
  général. Le titre de #1364 (« ne peut plus être … corrigée ») est donc trop fort, mais le défaut demeure : le refus arrive après
  que le numéro et l'écriture sont posés, et rien ne dit à l'utilisateur de dévalider.
- `crates/kesh-db/src/repositories/invoices.rs:2300-2301` : l'arrondi de la facture se calcule **à la validation**,
  `kesh_core::accounting::vat::invoice_rounding(ttc_brut, settings.round_to_5_centimes)` ; l'aperçu d'un brouillon le refait par
  `routes/invoices.rs:332` `async fn with_draft_rounding_preview(` (lecture du réglage `round_to_5_centimes`). Le contrôle de
  capacité peut donc connaître, avant la validation, le récapitulatif exact (taux, arrondi) que portera le PDF.
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte:448` `				validateError = err.message;` puis `notifyError(validateError)` :
  l'écran **affiche le message du serveur** pour tout refus de validation. Un refus neuf, localisé côté serveur, s'affiche sans
  changement de l'écran.

### Les PDF qui se rendent de nouveau, et ceux qui ne bougent jamais

- `crates/kesh-api/src/routes/issued_invoice_pdf.rs:104` `    if is_frozen(&invoice) {` : une facture au PDF figé sert le fichier
  archivé ; `issued_invoice_pdf.rs:238` `pub async fn refreeze(` rend de nouveau un fichier absent. Une facture validée **jamais
  téléchargée** se rend au premier téléchargement, avec le code en fonction, puis se fige (49-1b, D7).
- `crates/kesh-api/src/routes/credit_notes.rs:362` `        kesh_qrbill::generate_credit_note_pdf(&pdf_data, &i18n).map_err(map_qrbill_error)?;` :
  **le PDF d'un avoir n'est jamais figé**, il se rend à chaque demande, avoir ancien compris.
- `invoice_pdf_service.rs:213-217` (doc-comment de `render_document`) : « Les rappels, eux, restent régénérés (#502) » : **le PDF
  d'un rappel n'est jamais figé** non plus.
- Les tests avant / après de la 49-1b (`crates/kesh-qrbill/tests/rendu_avant_apres.rs`, fixtures
  `crates/kesh-qrbill/tests/fixtures/avant-0.20.0/`, et `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`, fixtures
  `crates/kesh-api/tests/fixtures/avant-0.20.0/`) portent neuf scénarios `kesh-qrbill` (S1 à S9) et quatre pièces HTTP (a à d).
  **Leurs désignations sont courtes et en caractères ordinaires**, sauf S7 : `rendu_avant_apres.rs:178` `                &format!("Prestation {}", i + 1),`,
  `pdf_avant_apres_e2e.rs:269-276` (« Loyer du mois d'avril », « Conseil », « Ouvrages de référence », « Frais de dossier ») ;
  S7 (`rendu_avant_apres.rs:294-300`) porte « Entretien annuel de la chaudière et des radiateurs du bâtiment B », 64 caractères,
  **réduite** en 0.19.0. Ce sont ces tests qui disent ce qui doit rester identique (D8).

## Inventaire par le symptôme

### S1 — « un texte du PDF de facture est réduit et suivi de « … » » (#1363)

```
$ git grep -n "truncate_display(" -- crates/kesh-qrbill/src/pdf.rs | grep -v "fn truncate_display\|///"
crates/kesh-qrbill/src/pdf.rs:318:                truncate_display(&line, META_MAX_CHARS)
crates/kesh-qrbill/src/pdf.rs:486:                truncate_display(&format!("{}: {}", i18n.get(key), v), IDENTITY_MAX_CHARS),
crates/kesh-qrbill/src/pdf.rs:668:            truncate_display(&line.description, 45),
crates/kesh-qrbill/src/pdf.rs:745:                truncate_display(&line.label, REMINDER_NOTE_MAX_CHARS)
crates/kesh-qrbill/src/pdf.rs:747:                truncate_display(&line.label, REMINDER_LABEL_MAX_CHARS)
```

- `:668` (désignation d'une ligne chiffrée) : **dans le périmètre** (D1).
- `:318` (bloc « Facture / numéro / date / client », 32 caractères) et `:486` (IDE, téléphone, e-mail, site de l'émetteur,
  46 caractères) : **hors périmètre** — ce sont des données de la société et du contact, modifiables après la validation, non
  du contenu figé de la facture ; leur largeur est l'objet de **#1229** (bloc d'identité).
- `:745`, `:747` (libellés du bloc de rappel) : textes produits par Kesh (`reminder_lines`, clés i18n), non saisis ; **hors
  périmètre**, aucun défaut constaté.

### S2 — « un refus du rendu PDF qui dépend du contenu figé de la facture tombe après sa validation » (#1364)

```
$ sed -n 520,560p crates/kesh-api/src/routes/invoice_pdf_service.rs | grep -n "QrBillError::"
(InvalidIban, InvalidQrIban, InvalidQrr | InvalidPayload, FieldTooLong, FieldEmpty, InvalidAmount, InvalidCurrency,
 InvalidCountry, InvalidCharset, PdfGeneration, TooManyLines, HeaderOverflow)
```

Chaque cause, triée selon que sa donnée se corrige après la validation :

- `TooManyLines` (lignes, récapitulatif) : contenu figé de la facture → **dans le périmètre** (D4).
- `InvalidCharset` / `FieldTooLong` sur le **numéro de facture** (il entre dans le message de la QR) : contenu figé → **hors
  périmètre**, porté par **#1046** (« Format de numérotation : le jeu de caractères de la QR-facture n'est pas contrôlé ») ;
  `NUMBER_FORMAT_UNRENDERABLE` contrôle déjà la longueur à la validation (`docs/api-external.md:520`).
- `HeaderOverflow`, `InvalidIban`, `InvalidQrIban`, `InvalidCountry`, `FieldEmpty`, `InvalidCharset` sur les noms et adresses :
  données de la société, du contact ou du compte bancaire, **corrigibles après la validation** (le message dit où) → hors du
  symptôme.
- `InvalidAmount` (montant hors bornes de la QR) : contenu figé, mais contrôlé à la validation par le montant minimum
  (`kesh-db/src/repositories/invoices.rs:2306-2318`) ; la borne haute (999 999 999.99) n'est pas un cas réel → hors du symptôme,
  écrit ici comme angle mort assumé.
- **Rappel** : le bloc de rappel est réservé dans la capacité (`reminder_block_height`, `pdf.rs:1998`) ; une facture qui tient
  juste sans rappel peut ne plus tenir avec. Ce n'est pas la validation de la facture qui le décide (le rappel vient après) ;
  comportement de la 0.19.0, inchangé ici → **issue à ouvrir** (Dev Notes).

### S3 — « un caractère saisi disparaît du PDF de facture sans avertissement » (#1400)

```
$ grep -n "layer.use_text(" crates/kesh-qrbill/src/pdf.rs | awk -F: '$1>=415 && $1<=772'   (puis la ligne suivante de chacun)
431 texte_imprime(&inv.creditor_name)      440 texte_imprime(line) (adresse émetteur)
486 identité de l'émetteur, brute          510 titre i18n         528 meta (numéro, dates, n° client), brute
532 i18n                                   540/543 texte_imprime (destinataire)
580-608 en-têtes i18n                      657 commentaire (texte_de_commentaire)
668 désignation chiffrée, brute            674-689 montants      709 récapitulatif   720/727 total
745/747 libellés du rappel (i18n)          759 conditions de paiement, brutes
```

- `:668` désignation chiffrée et `:657` commentaire (texte de la ligne) : **dans le périmètre** (D6).
- `:759` **conditions de paiement** (`payment_terms`, texte de la facture, figé à la validation) : même défaut, **hors
  périmètre** — #1400 ne nomme que les lignes, et une issue est une release (règle de Guy du 2026-11-19) → **issue à ouvrir**
  (Dev Notes).
- `:486` identité de l'émetteur et `:528` bloc meta : données de la société et du contact (#943 porte la police embarquée qui
  lèverait le défaut pour tout le corps) → hors périmètre.
- Noms et adresses (`:431`, `:440`, `:540`, `:543`) : `texte_imprime` y replie déjà les 125 lettres ; une lettre hors de ces 125
  y disparaît encore, déjà dit au manuel (`user-manual.tex:1594`) ; NFD → #1411 ; hors périmètre.

## Décisions

- **D1 — La désignation d'une ligne chiffrée se coupe, elle ne se tronque plus.** Elle passe par **le même texte de ligne** que
  le commentaire (D6) et se coupe par `couper_ligne_a` à la largeur `DESIGNATION_LARGEUR = col_qty - col_desc - 2.0` = **88 mm**
  (2 mm d'écart avant la quantité), à 9 pt. La généralisation est **une** fonction, `lignes_de_texte(description, largeur)`,
  dont `lignes_de_commentaire` devient l'appel à `COMMENTAIRE_LARGEUR` (DRY, comme `couper_ligne` en 49-1b). La quantité, le
  prix, la TVA et le total s'écrivent **à la hauteur de la première ligne** ; chaque ligne de texte suivante prend un pas de
  **5 mm** et passe par la garde, comme une ligne de commentaire (49-1b, D6) — d'où la règle du manuel : *chaque ligne imprimée
  prend la place d'une ligne de facture*. *Écartées* : (a) un pas réduit (4 mm) pour les lignes de suite — plus serré, mais une
  seconde unité de capacité à expliquer ; (b) réduire la taille du texte d'une désignation longue (lisibilité, et la mesure
  changerait de base) ; (c) garder la troncature et avertir à la saisie (remède alternatif de #1363) — le client ne verrait
  toujours pas la fin de la désignation.
- **D2 — Repli sur le rendu de la 0.19.0 pour une pièce d'avant qui ne tiendrait plus.** Le rendu calcule d'abord le **plan
  neuf** (D1, D6) ; s'il ne tient pas **et que toutes les lignes de la pièce sont chiffrées**, la pièce se rend **exactement
  comme en 0.19.0** : désignation brute, `truncate_display(…, 45)`, une ligne par ligne chiffrée — le code d'aujourd'hui,
  conservé tel quel dans une fonction nommée (`dessiner_ligne_chiffree_0_19_0`, ou nom équivalent), avec un `tracing::warn!`
  qui nomme la pièce. S'il ne tient pas et qu'une ligne non chiffrée est présente (pièce née en 0.20.0, contrôlée à sa
  validation, D4), ou si le rendu de la 0.19.0 ne tient pas non plus : `TooManyLines`, comme aujourd'hui. **Ce repli n'est
  atteignable que pour une pièce validée sans le contrôle de D4** : facture validée avant la 0.20.0 (ou validée par la
  démonstration, `kesh-seed`), son avoir, son rappel, ou la course de D4. *Écartées* : (a) refuser (une facture d'avant,
  imprimable en 0.19.0, deviendrait impossible à imprimer — interdit par les contraintes de production) ; (b) choisir le rendu
  selon la date de validation ou la version (aucune colonne ne la porte ; une migration est interdite en 0.20.0) ; (c) un repli
  ligne par ligne (une pièce mi-coupée, mi-tronquée : deux règles sur une même page).
- **D3 — La dérogation, écrite.** *« Une facture déjà émise se réimprime à l'identique »* reste vrai pour **toute facture dont le
  PDF est archivé** : `issued_invoice_pdf.rs` ne change pas (`git diff` vide, AC11), le fichier figé est servi octet pour octet
  (test T12 de la 49-1b, inchangé). **La dérogation admise par Guy** (fichier d'épique, ligne 49-4 ; sprint-status, commentaire de
  la clé 49-4) vaut pour **une facture validée jamais téléchargée** et pour **un regel** : elles se rendent avec le code de la
  0.20.0. **Elle s'étend nécessairement à deux PDF que Kesh ne fige jamais** : **l'avoir** (`credit_notes.rs:362`) et **le
  rappel** (#502) — tout avoir et tout rappel, même d'une facture d'avant, se rendent avec le code en fonction ; la 49-1d le
  dit déjà pour l'avoir (sa D8 et son AC14, « une facture jamais téléchargée, une facture regelée et un avoir ») ; **le rappel est
  à confirmer par Guy** (arbitrage demandé au rapport). Ce qui change, sur ces quatre chemins, et seulement là :
  (1) une désignation de **plus de 45 caractères**, ou **plus large que 88 mm**, s'imprime entière sur plusieurs lignes au lieu
  d'être réduite à 44 caractères et « … » (ou de déborder dans la colonne de la quantité) ; les lignes, le récapitulatif et le
  total qui suivent descendent d'autant ;
  (2) une désignation qui contient U+2010, U+2011, U+2012 ou U+2212 les imprime en « - » au lieu de les perdre ; une des 125
  lettres du § 4.1.1 (Č, ł, Ș…) s'imprime par sa base au lieu de disparaître ; U+00A0 et tout blanc s'impriment en espace ; un
  saut de ligne coupe la désignation au lieu d'être perdu ; U+00AD est retiré ;
  (3) si le plan neuf ne tient pas, rien ne change : D2 rend la pièce comme en 0.19.0.
  **Ce qui ne change pas** : une désignation d'**au plus 45 caractères**, tenant dans **88 mm**, faite de caractères que
  WinAnsi porte et sans blanc autre que U+0020 ni caractère de (2), s'imprime **au même dessin** qu'en 0.19.0 — c'est le cas de
  toutes les pièces des tests avant / après de la 49-1b hors S7 (AC2), et de F2 à F8 et A1 de la recette de la 49-1d si leurs
  désignations suivent cette règle (Dev Notes, cohérence avec la 49-1d). Les montants, la TVA, les écritures, les numéros ne
  changent sur aucun chemin : la story n'écrit rien en base. *Écartée* : rendre l'ancien dessin à toute pièce validée avant la
  0.20.0 (pas de critère sans migration, D2 b).
- **D4 — La capacité se contrôle à la validation, par la même mise en page que le rendu.** `kesh-qrbill` expose une fonction
  **pure** (sans document PDF) qui calcule le **plan du tableau des lignes** — pour chaque ligne, ses lignes de texte, et le
  nombre de lignes imprimées que demande la pièce contre celui qu'offre la page — à partir des seules données qui le décident :
  les lignes (`InvoiceLinePdf`), le récapitulatif (`vat_lines`, `subtotal_ht`, `rounding`, lus par `recap_lines` avec
  `QrBillI18n::default()`, comme `reminder_block_height` à `pdf.rs:2001` : seul le **nombre** de lignes compte), le bloc de rappel
  et le plancher. `draw_invoice_section` **dessine ce plan** : il n'y a pas deux calculs (DRY ; T-PROP le prouve). Une fonction
  publique, `controler_les_lignes(…) -> Result<(), RefusDesLignes>` (nom au choix du développement, signature à la tâche),
  l'appelle au plancher d'une **facture** (`SEP_Y`), **sans** bloc de rappel. `validate_invoice_handler` l'appelle **avant**
  `invoices::validate_invoice` : il lit le brouillon et ses lignes (`find_by_id_with_lines`), construit les `InvoiceLinePdf` par
  `ligne_pdf` (49-1b, D10), les `vat_lines` par `vat_lines_pdf`, l'arrondi par `kesh_core::accounting::vat::invoice_rounding`
  sur le réglage `round_to_5_centimes` (comme `with_draft_rounding_preview`), et refuse **sans rien écrire** (statut, numéro,
  écriture, version inchangés). Le contrôle ne lit **ni** contact, **ni** compte bancaire, **ni** société : la validation
  n'acquiert aucun prérequis neuf. Une facture **dévalidée** qui se revalide y passe de même.
  **Course assumée** : le contrôle précède la transaction de validation ; un `PUT` concurrent du brouillon entre les deux peut
  faire valider des lignes non contrôlées. Conséquence bornée : la pièce se rend par D2 (lignes toutes chiffrées), ou rend le
  refus d'aujourd'hui, et se dévalide. *Écartées* : (a) le contrôle **dans** la transaction de `kesh-db` (un rappel passé à
  `validate_invoice`) — `kesh-db` deviendrait le cinquième module de code et la story passerait la règle de splitting ; la
  course ne justifie pas ce coût ; (b) le contrôle au `PUT` du brouillon (il bloquerait la saisie d'un brouillon en cours) ;
  (c) un compteur dans l'écran (il dupliquerait la mesure AFM de `kesh-qrbill` : D12 de la 49-1c) ; (d) le PDF sur plusieurs
  pages (#1374, CR distincte) ; (e) réserver le bloc de rappel dans le contrôle (réduirait la capacité de **toute** facture
  d'environ trois lignes pour un rappel hypothétique ; S2, issue à ouvrir).
- **D5 — Un code neuf pour le refus à la validation : `400 INVOICE_LINES_DO_NOT_FIT_PDF`.** La phrase dit ce qui est possible
  sur un brouillon, avec le besoin et la place en **lignes imprimées** : *« Les lignes de cette facture ne tiennent pas sur la page
  de son PDF : elles demandent { $besoin } lignes imprimées, la page en offre { $place }. Raccourcissez des désignations ou des
  commentaires, retirez des lignes, ou répartissez-les sur deux factures. La facture reste un brouillon. »* (clé
  `error-invoice-lines-do-not-fit-pdf`, quatre catalogues). *Écartée* : reprendre `INVOICE_TOO_MANY_LINES_FOR_PDF` — son code est
  documenté pour le **rendu** (`docs/api-external.md:1281`), sa phrase compte des **lignes de la pièce** (une facture de trois
  lignes longues dirait « contient 3 lignes — ne peut pas toutes les afficher », incompréhensible). Le refus au rendu garde son
  code et sa phrase : **#1110** (la phrase impossible à suivre sur une facture validée) reste ouverte, hors périmètre — elle ne
  se lit plus, après cette story, que pour une pièce d'avant ou un rappel (S2).
- **D6 — Caractères (#1400) : replier ce qui a un équivalent, refuser à la validation ce qui n'en a pas, ne rien perdre en
  silence.** Une fonction unique de `kesh-qrbill`, `texte_de_ligne` (la `texte_de_commentaire` de la 49-1b, renommée et
  étendue), sert **la désignation chiffrée et le commentaire** : en plus de ce qu'elle fait (sauts de ligne, blancs, U+00AD,
  125 lettres), elle replie la **table close** U+2010, U+2011, U+2012, U+2212 → « - » (U+2013 et U+2014 sont dans WinAnsi et
  restent tels quels) ; elle retire sans rien dire la **liste close des invisibles** U+200B, U+200C, U+200D, U+2060, U+FEFF (rien
  de visible ne se perd). **Tout autre caractère** que la police ne porte pas (grec, cyrillique, flèches, ≤ ≥, émojis, CJK…) est
  un **caractère sans glyphe** : une fonction publique de `kesh-qrbill`, `caracteres_sans_glyphe(description) -> Vec<char>` (sans
  doublon, dans l'ordre d'apparition), **dérivée du même code** que le filtre du rendu (une seule source de ce que dessine
  `kesh-qrbill`, réponse au motif (c) de la D10 de la 49-1c), les énumère. `validate_invoice_handler` refuse alors, **avant** le
  contrôle de capacité, en `400 INVOICE_LINE_UNPRINTABLE_CHARACTERS` : *« Ligne { $ligne } : « { $caracteres } » ne s'impriment
  pas sur le PDF de la facture (la police ne les porte pas). Remplacez-les ou retirez-les ; la facture reste un brouillon. »*,
  chaque caractère nommé comme le fait déjà `CaractereRefuse` (`invoice_pdf_service.rs:646-650` : `escape_debug`, puis
  `U+XXXX`) ; la **première** ligne fautive est nommée, avec tous ses caractères (la suivante se dira au prochain essai). Le rendu,
  lui, continue de retirer un caractère sans glyphe (il ne refuse pas : une pièce d'avant qui en porte s'imprime comme avant, D3).
  *Écartées* : (a) une police embarquée (#943) — elle change les ressources de **tout** PDF (49-1b, D3) ; (b) le repli dans le
  seul commentaire (morcelle #1400 sur deux releases, D10 de la 49-1c) ; (c) refuser au `PUT` du brouillon (bloque la saisie) ;
  (d) un avertissement non bloquant dans la réponse du brouillon (contrat d'API neuf et écran neuf, pour un cas rare) ; (e)
  replier le grec ou le cyrillique sur une translittération (un nom propre changé en silence : pire que le refus).
- **D7 — Aucun changement de l'écran ; un E2E le joue.** Le dialogue de validation affiche le message du serveur
  (`+page.svelte:448`). Les deux refus neufs passent par là, localisés. Aucun fichier de `frontend/src` ne change ; un E2E
  Playwright joue les deux refus dans le navigateur (AC9). *Écartée* : une mise en évidence de la ligne fautive dans le
  formulaire (écran neuf, `InvoiceForm.svelte`, commun avec 49-1c, 49-2, 49-3 : hors de la coupure).
- **D8 — La preuve avant / après reprend le mécanisme de la 49-1b, sans toucher à ses fixtures ni à son comparateur.** Le
  comparateur `crates/kesh-qrbill/src/empreinte.rs` (feature `test-support`) est **gelé** : il ne change pas. Les fixtures
  `avant-0.20.0/` des deux crates ne changent pas. (a) **Ce qui doit rester identique** : S1 à S6, S8, S9 et les pièces HTTP a à
  d — les tests de la 49-1b restent verts **sans modification**, sauf l'exclusion de **S7** du test
  `neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0`, dont la désignation de 64 caractères change par décision (D3) : S7 passe
  à un test dédié (T3) qui compare le rendu neuf à **la même fixture S7** et borne l'écart. (b) **Références neuves, capturées
  avant tout changement du rendu** (le rendu à `3379a971a` est celui de la 0.19.0 pour les lignes chiffrées, prouvé par la
  49-1b) : S10 (facture d'avant **pleine** : huit lignes chiffrées à désignations de 60 caractères, sans TVA — elle tient
  tronquée, ne tiendrait pas coupée), S11 (une ligne « Remise −50 sur l'entretien Čolić », U+2212 et une des 125 lettres,
  moins de 45 caractères), S12 (avoir de douze lignes dont trois à désignations longues), et, au niveau HTTP, (e) une facture
  de la forme S10 créée et **validée par le dépôt** (comme en 0.19.0, sans le contrôle de D4) et (f) une facture à une
  désignation de 64 caractères validée par le dépôt. Le graphe est celui de la 49-1b, réduit : un **commit C'** (tests de capture
  et leurs scénarios, aucune ligne de `RENDU`), un worktree **détaché à C'** sous `kesh-gate-logs/e494/capture/` où les tests
  écrivent leurs références (`KESH_REFERENCES=ecrire`, puis `comparer` : stabilité), un **commit F'** des seules fixtures, dans
  des dossiers **neufs** `crates/kesh-qrbill/tests/fixtures/avant-0.20.0-49-4/` et `crates/kesh-api/tests/fixtures/avant-0.20.0-49-4/`,
  **puis** le code. `RENDU` = `crates/kesh-qrbill/src/{pdf,types,largeurs,texte,generator,validation,parser}.rs` et
  `crates/kesh-api/src/`. *Écartée* : réécrire la fixture S7 (elle est la preuve de ce qu'imprimait la 0.19.0 ; la geler et
  mesurer l'écart est plus fort) ; capturer sur un worktree de `773df80da` (la 49-1b a prouvé l'égalité du rendu chiffré).
- **D9 — Le manuel reprend les phrases de la 49-1c après sa fusion.** Les phrases à reprendre (`user-manual.tex:1456`,
  `:1594` sur la branche de la 49-1c) n'existent pas encore dans la branche de cette story. La tâche du manuel se fait **en
  dernier**, après la fusion de la 49-1c dans `epic/49-types-de-lignes-facture` et sa reprise dans cette branche (`git merge`
  de la branche d'épique, commit de fusion relu). Si la 49-1c n'est pas fusionnée quand le reste de la story est clos, la 49-4
  écrit ses phrases sur le texte actuel (`user-manual.tex:1456`, `:1594` à `3379a971a`) et **le second arrivé réconcilie**
  (CLAUDE.md § *Fusions et fichiers communs*) ; l'orchestrateur en est averti au Dev Agent Record. *Écartée* : écrire au manuel
  des phrases de la 49-1c avant elle (conflit certain, et texte de la 49-1c décrit par une autre story).
- **D10 — Aucune ligne de `kesh-db`, aucune écriture en base, aucun changement de l'API de création.** `git diff 3379a971a..HEAD --
  crates/kesh-db crates/kesh-core crates/kesh-api/src/routes/issued_invoice_pdf.rs` est vide (AC11). `validate_line` et
  `MAX_DESCRIPTION_LEN` (1000) ne changent pas : la saisie d'un brouillon reste libre ; seule la validation contrôle.

## Critères d'acceptation

- **AC1 — Une désignation longue s'imprime entière.** Niveau `kesh-qrbill`, facture, avoir et rappel : une ligne chiffrée dont
  la désignation fait 300 caractères (mots courts et un mot de 60 `W`) s'imprime sans « … » ; (1) le texte sans blancs, après
  `texte_de_ligne`, égale la concaténation sans blancs des lignes dessinées à `x = 20` mm ; (2) chaque ligne mesure au plus 88 mm
  (`largeur_mm(ligne, 9.0, false)`) ; (3) aucun texte de la désignation n'est dessiné à `x ≥ 108` mm ; (4) quantité, prix, TVA et
  total sont à l'ordonnée de la **première** ligne ; (5) la ligne suivante de la pièce descend de `n × 5` mm (`n` = lignes de
  texte de la désignation).
- **AC2 — Ce qui doit rester identique reste identique.** `neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0` (S7 exclu, la seule
  modification du fichier `rendu_avant_apres.rs`, avec un commentaire qui renvoie à D3 et au test T3),
  `les_deux_coupes_de_la_qr_sont_inchangees`, `quatre_pieces_chiffrees_se_rendent_comme_en_0_19_0` (a à d, regel compris) sont
  **verts** ; `git diff 3379a971a..HEAD -- crates/kesh-qrbill/tests/fixtures/avant-0.20.0/ crates/kesh-api/tests/fixtures/avant-0.20.0/
  crates/kesh-qrbill/src/empreinte.rs crates/kesh-api/tests/pdf_avant_apres_e2e.rs` est **vide**.
- **AC3 — La dérogation est bornée et prouvée.** S7 (fixture de la 49-1b) et S11 (fixture C'/F') rendus par le code neuf :
  la liste `(x, y, texte)` (`empreinte::textes`) égale celle de la référence **hors** (i) les textes de la désignation modifiée,
  remplacés par ses lignes neuves (S7 : la désignation entière, coupée ; S11 : « Remise -50 sur l'entretien Colic ») et (ii) les
  ordonnées de tout ce qui suit, décalées d'exactement `(n − 1) × 5` mm ; polices, `MediaBox`, nombre de pages égaux. Au niveau
  HTTP, (f) même épreuve par `GET /pdf` d'une facture validée jamais téléchargée, puis par le regel ; et un PDF **figé** avant
  le changement est servi octet pour octet (T12 de la 49-1b vert sans modification).
- **AC4 — Une pièce d'avant qui ne tiendrait plus se rend comme en 0.19.0.** S10 et S12 rendus par le code neuf **égalent**
  leur référence (empreinte complète) ; au niveau HTTP, (e) `GET /pdf` rend `200` et un contenu égal à sa référence, puis son
  regel de même ; un `tracing::warn!` nomme la pièce. Une pièce **à commentaire** qui ne tient pas rend toujours
  `TooManyLines` (pas de repli : test `invoice_pdf_e2e.rs:1030`, le commentaire de mille caractères, vert sans modification).
- **AC5 — La capacité se contrôle à la validation, et seulement elle.** `POST /api/v1/invoices/{id}/validate` sur un brouillon
  (a) d'une ligne de 1000 caractères, (b) de huit lignes courtes à deux taux de TVA (récapitulatif), (c) de sept lignes dont
  une coupée en deux, arrondi activé et non nul (l'arrondi fait la différence : la même pièce **passe** arrondi désactivé)
  rend `400 INVOICE_LINES_DO_NOT_FIT_PDF`, message aux deux nombres ; après le refus : `status = 'draft'`, `invoice_number`
  et `journal_entry_id` inchangés (nuls pour un brouillon jamais validé), `version` inchangée, aucune ligne
  d'`journal_entries` neuve, compteur de numérotation inchangé (la validation suivante d'une facture qui tient reçoit le numéro
  attendu). (d) Au seuil exact (huit lignes imprimées sans récapitulatif) la validation **réussit**, et le PDF se rend.
  (e) **Un seul plan** : pour une famille de pièces engendrées (nombre de lignes, longueurs, taux, arrondi), `controler_les_lignes`
  rend `Ok` **si et seulement si** `generate_qr_bill_pdf_with_date` rend `Ok` sans passer par le repli de D2 (T-PROP).
- **AC6 — Caractères.** (a) Rendu : U+2010, U+2011, U+2012, U+2212 s'impriment « - », dans une désignation et un commentaire ;
  les 125 lettres se replient dans une désignation ; les cinq invisibles de D6 sont retirés sans refus. (b) Validation : un
  brouillon dont la ligne 2 porte « Ωmega — проект » rend `400 INVOICE_LINE_UNPRINTABLE_CHARACTERS` nommant la ligne 2 et
  « Ω » (U+03A9), « п » (U+043F)… dans l'ordre, sans doublon ; « — » (U+2014, dans WinAnsi) n'y figure pas ; même effet sans
  écriture qu'en AC5 ; « Remise −50 » se valide et son PDF porte « Remise -50 » (relu par `empreinte::textes`). (c) **Source
  unique** : `caracteres_sans_glyphe(t)` est vide **si et seulement si** `texte_de_ligne(t)` garde, pour chaque caractère non
  blanc et hors des invisibles de D6, un caractère imprimé (T-SRC, sur le balayage U+0000 à U+FFFF).
- **AC7 — Textes et contrat.** Deux codes neufs (`INVOICE_LINES_DO_NOT_FIT_PDF`, `INVOICE_LINE_UNPRINTABLE_CHARACTERS`, `400`) ;
  leurs clés dans les **quatre** catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, bloc de
  l'Epic 49) et dans la garde `crates/kesh-api/tests/textes_coherents.rs` (même forme que `error-invoice-too-many-lines-for-pdf`,
  `:775`) ; `docs/api-external.md` : la route de validation (`:520`) nomme les deux refus, la table des codes (`:1281`) gagne
  deux lignes et dit que `INVOICE_TOO_MANY_LINES_FOR_PDF` ne se rencontre plus que pour une pièce validée avant la 0.20.0 ou un
  rappel ; `admin-manual.tex` § *Pour les intégrations* de `sec:maj-0-20-0` (`:2559`) gagne une phrase sur ces deux refus.
- **AC8 — Manuel utilisateur.** Après D9 : (a) l'item « Lignes de facture » (`:1456` de la 49-1c) dit qu'une désignation et un
  commentaire **s'impriment entiers**, sur plusieurs lignes, que chaque ligne imprimée prend la place d'une ligne de facture,
  qu'une facture en tient **huit lignes imprimées au plus, moins avec le récapitulatif de TVA**, et que **Kesh le vérifie à la
  validation** : une facture qui ne tiendrait pas est refusée en le disant, le brouillon reste modifiable ; (b) le paragraphe de
  `:1594` ne dit plus « disparaît du PDF sans avertissement » pour les lignes : le moins et les traits d'union s'impriment en
  trait d'union, les 125 lettres par leur base **dans la désignation comme dans le commentaire**, tout autre caractère que la
  police ne porte pas fait **refuser la validation** en nommant la ligne et le caractère ; le **conseil d'écrire le moins en trait
  d'union ordinaire est retiré** (caduc) ; « le reste du corps (… conditions, téléphone, e-mail et site) n'a pas ce repli » reste
  vrai **sans les libellés des lignes** ; (c) une phrase « depuis la version~0.20.0 » dit la dérogation de D3 en mots
  d'utilisateur : PDF archivé inchangé ; une facture validée avant la 0.20.0 et jamais téléchargée, un regel, un avoir, un rappel
  s'impriment avec la désignation entière, ou comme avant si elle ne tient plus. **Contrôles** (sortie copiée au Dev Agent
  Record) : `git grep -n "sans avertissement" -- docs/manual/fr/user-manual.tex` ne nomme plus de phrase sur les lignes de
  facture ; `git grep -n -i "trait d'union ordinaire" -- docs/manual/fr/user-manual.tex` est **vide** ; `git grep -n
  "44 caract" -- docs/manual/fr` ne nomme que la phrase de la dérogation. Le PDF du manuel n'est pas commité ;
  `pdftotext` du PDF régénéré localement relu sur ces trois points.
- **AC9 — E2E.** Un fichier Playwright (`frontend/tests/e2e/invoice-validation-pdf.spec.ts`) : (a) un brouillon dont une ligne
  porte une désignation de 1000 caractères → *Valider* → le message du refus de capacité est **visible** dans le dialogue,
  la facture reste « Brouillon », sans numéro ; (b) une ligne portant « Ωmega » → le message nomme la ligne et « Ω » ; (c) après
  correction (désignation raccourcie), la validation réussit et le téléchargement du PDF répond `200`. Le smoke de la CI n'est
  pas modifié.
- **AC10 — Recette visuelle** (`CLAUDE.md` § *Recette visuelle*) : captures PNG des **états touchés**, thème clair, sur une base
  de démonstration (`kesh_e2e_e494`, jamais la base du NAS) — le dialogue de validation sur chacun des deux refus ; le PDF
  (`mutool draw -r 150`) d'une facture à désignation longue coupée, d'une facture à « Remise -50 », d'un avoir à désignation
  longue, et, en témoin, d'une facture courte inchangée. Dans `kesh-gate-logs/e494/captures/`, jointes au Dev Agent Record et au
  prompt de revue ; chaque capture se **lit** (désignation entière, alignée à gauche, sans chevaucher la quantité ; montants à la
  première ligne).
- **AC11 — Factures et avoirs existants inchangés.** Aucune écriture en base : `git diff 3379a971a..HEAD -- crates/kesh-db
  crates/kesh-core crates/kesh-api/src/routes/issued_invoice_pdf.rs crates/kesh-api/src/routes/invoice_pdf.rs` **vide** ; les
  lignes, totaux, TVA, écritures, numéros d'une facture validée ne bougent sur aucun chemin ; le PDF archivé est servi tel quel
  (AC3) ; ce qui change du rendu d'une pièce d'avant est **exactement** la liste de D3, prouvé par AC2, AC3 et AC4.
- **AC12 — Coordination.** La story ne touche ni `crates/kesh-qrbill/src/largeurs.rs`, ni `kesh-report/`, ni `CHANGELOG.md`, ni
  `frontend/src/`. `lib.rs` de `kesh-qrbill` ne change que par l'ajout des fonctions publiques de D4 et D6 et de leurs types
  au ré-export (`lib.rs:24-31`), dans un hunk distinct de celui de #1327 (`pub use largeurs::largeur_mm;`, ligne voisine :
  relire à la fusion). `sprint-status.yaml` : seul l'orchestrateur le tient.
- **AC13 — Gates.** § *Tâches*, dernière tâche ; le Dev Agent Record n'affirme que ce qui a tourné.

## Tests à écrire, et la preuve du rouge

Journaux de rouge dans `/home/gcorbaz/devel/kesh-gate-logs/e494/` (`rouge-T<n>[-m].log`), commande et sortie. Chaque mutation
s'**ajoute au code**, le test tourne, le journal se garde, la mutation se retire **et le fichier se touche** (`touch`) avant la
reconstruction (cargo garde sinon le binaire muté).

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T0 | `crates/kesh-qrbill/tests/rendu_designations.rs::capturer_references_avant_49_4` (`#[ignore]`, `KESH_REFERENCES=ecrire\|comparer`, défaut `comparer`) | écrit S10, S11, S12 dans `avant-0.20.0-49-4/` **dans le worktree détaché à C'** ; `sources.sha256` des fichiers de `RENDU` lus | `ecrire` puis `comparer` dans le même worktree propre : stables (`t0-stabilite.log`, avec `git status --porcelain` vide) |
| T0-bis | `crates/kesh-api/tests/pdf_designations_e2e.rs::capturer_references_http_avant_49_4` (`#[ignore]`, `#[sqlx::test]`, mêmes modes) | (e) et (f) créées et validées **par le dépôt**, rendues par `GET /pdf` puis regel ; références dans `kesh-api/tests/fixtures/avant-0.20.0-49-4/` | idem (`t0bis-stabilite.log`) |
| T1 | `pdf.rs::tests::une_designation_longue_s_imprime_entiere` (AC1) | les cinq invariants, facture, avoir et rappel | mutations : (1) la désignation reprend `truncate_display(…, 45)` → **ROUGE** ; (2) largeur 88 → 110 → **ROUGE** (chevauchement) ; (3) montants à la dernière ligne de texte → **ROUGE** (`rouge-T1-1..3.log`) |
| T2 | tests existants de la 49-1b, `rendu_avant_apres.rs` et `pdf_avant_apres_e2e.rs` (AC2) | S1 à S6, S8, S9, a à d identiques | mutation : la désignation passe par `texte_de_ligne` **et** une espace de fin s'ajoute à chaque ligne → **ROUGE** sur S1 (`rouge-T2.log`) |
| T3 | `rendu_designations.rs::s7_et_s11_ne_changent_que_la_designation` (AC3, niveau `kesh-qrbill`) | écart borné à la désignation et au décalage `(n − 1) × 5` | mutations : (1) le décalage des lignes suivantes passe à `n × 5` → **ROUGE** ; (2) le repli U+2212 retiré → **ROUGE** (S11 : « Remise 50 ») (`rouge-T3-1..2.log`) |
| T4 | `rendu_designations.rs::une_piece_d_avant_pleine_se_rend_comme_en_0_19_0` (AC4) | S10 et S12 égaux à leur référence | mutation : le repli de D2 supprimé (le plan neuf refuse) → **ROUGE** (`TooManyLines`) ; mutation 2 : le repli applique `texte_de_ligne` → **ROUGE** (`rouge-T4-1..2.log`) |
| T5 | `pdf_designations_e2e.rs::designation_longue_par_le_http` (AC3 f, AC4 e) | (e) et (f) par `GET /pdf` et regel, contre leurs références ; T12 de la 49-1b vert | mutation : `render_and_pose` ignore D2 → **ROUGE** sur (e) (400 au lieu de 200) (`rouge-T5.log`) |
| T6 | `kesh-api/tests/invoice_validation_pdf_e2e.rs::la_validation_refuse_une_facture_qui_ne_tient_pas` (AC5 a à d) | refus 400, rien d'écrit (statut, numéro, écriture, version, compteur), seuil accepté, l'arrondi compté | mutations : (1) le handler n'appelle pas le contrôle → **ROUGE** ; (2) le contrôle ignore l'arrondi → **ROUGE** sur (c) ; (3) le contrôle compte une ligne par ligne de la pièce (sans la coupe) → **ROUGE** sur (a) (`rouge-T6-1..3.log`) |
| T7 | `invoice_validation_pdf_e2e.rs::la_validation_nomme_les_caracteres_sans_glyphe` (AC6 b) | refus 400 nommant ligne et caractères, U+2014 absent du message, rien d'écrit ; « Remise −50 » validée, PDF « Remise -50 » | mutations : (1) le contrôle des caractères retiré → **ROUGE** ; (2) U+2014 compté sans glyphe → **ROUGE** ; (3) la ligne nommée est l'index 0 au lieu de la position 1 → **ROUGE** (`rouge-T7-1..3.log`) |
| T-PROP | `pdf.rs::tests::le_controle_et_le_rendu_ont_le_meme_plan` (AC5 e) | équivalence `controler_les_lignes` ⇔ rendu sans repli, sur une famille engendrée (déterministe, graine fixe, au moins 500 pièces dont des cas au seuil exact) | mutation : la garde du dessin passe à `ty < plancher - 0.5` (le dessin admet une ligne de plus que le plan) → **ROUGE** (`rouge-TPROP.log`) |
| T-SRC | `pdf.rs::tests::caracteres_sans_glyphe_et_texte_de_ligne_disent_la_meme_chose` (AC6 c) | équivalence sur le balayage U+0000–U+FFFF (hors surrogats) | mutation : ajouter U+03A9 → « O » dans la table de repli **sans** toucher `caracteres_sans_glyphe` → impossible si la source est unique : le test reste vert et c'est la preuve ; mutation réelle : `caracteres_sans_glyphe` réécrit avec sa propre table WinAnsi qui omet U+20AC → **ROUGE** (`rouge-TSRC.log`) |
| T8 | `pdf.rs::tests::repli_des_tirets_et_invisibles` (AC6 a) | quatre tirets → « - » dans désignation et commentaire, 125 lettres dans la désignation, cinq invisibles retirés | mutation : U+2011 retiré de la table → **ROUGE** (`rouge-T8.log`) |
| T9 | E2E `frontend/tests/e2e/invoice-validation-pdf.spec.ts` (AC9) | (a), (b), (c) dans le navigateur | rouge constaté en lançant le fichier sur le binaire **sans** le contrôle (handler muté, binaire reconstruit) : (a) et (b) échouent (`rouge-T9.log`) |
| T10 | épreuve, non test neuf : `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`, `dunning_reminders_e2e`, `golden_test`, `invoice_line_types_e2e`, tests de `pdf.rs` | verts **sans modification d'assertion** ; un test existant qui valide **par la route** une pièce qui ne tient pas devient rouge par conception : il se liste au Dev Agent Record, avec le changement fait (validation par le dépôt, ou pièce raccourcie) et sa raison | — |

## Tâches

- [ ] **Étape 0 — lire, puis capturer avant tout changement du rendu** (D8). (1) Relire les citations de « Ce qui se lit au code »
  à la tête ; `git log --oneline 3379a971a..origin/epic/49-types-de-lignes-facture -- crates/kesh-qrbill crates/kesh-api/src/routes`
  pour voir si la branche d'épique a bougé (49-1c, 49-3) : si oui, la reprendre avant C'. (2) **Commit C'** : `rendu_designations.rs`
  (T0 et les scénarios S10 à S12), `pdf_designations_e2e.rs` (T0-bis et le montage de (e), (f)), **aucune ligne de `RENDU`**.
  (3) `git worktree add --detach /home/gcorbaz/devel/kesh-gate-logs/e494/capture C'` ; dans ce worktree : `git status --porcelain`
  vide, `DATABASE_URL` vers le MariaDB de dev (compte `kesh`, droit `\_sqlx\_test%`, `docs/testing.md:22`), puis
  `KESH_REFERENCES=ecrire` et `=comparer` pour T0 et T0-bis, sous `scripts/mem-guard.sh` et le verrou de la voie. (4) Copier les
  fixtures vers l'arbre de développement, `diff -r` identique, les **déplacer** hors du worktree de capture (`mv` vers
  `kesh-gate-logs/e494/fixtures-capturees/`), `git status --porcelain` vide, `git worktree remove` sans `--force` ; **commit F'**
  (`test(49-4) : références du rendu d'avant la 49-4 (refs #1363)`). (5) Contrôles, script en lecture seule
  `kesh-gate-logs/e494/controles-graphe.sh` (modèle : `kesh-gate-logs/e491b/ac1-controles.sh`) : `git diff 3379a971a C' -- RENDU`
  vide ; `git diff --stat C' F'` ne nomme que les deux dossiers `avant-0.20.0-49-4/` ; `git merge-base --is-ancestor F' <premier
  commit de RENDU>` rend 0 ; à la clôture, `git diff F'..HEAD` des fixtures neuves et de `empreinte.rs` vide. Hashs de C' et F' au
  seul Dev Agent Record.
- [ ] **`kesh-qrbill`** : `texte_de_ligne` (D6 : table des quatre tirets, liste des cinq invisibles), `caracteres_sans_glyphe`
  (même source), `lignes_de_texte(description, largeur)` (D1), le plan du tableau des lignes et `controler_les_lignes` (D4) ;
  `draw_invoice_section` dessine le plan, avec le repli de D2 (code d'aujourd'hui, isolé et nommé) ; doc-comment de
  `draw_invoice_section` réécrit (la désignation se coupe, la phrase « inchangé depuis la 0.19.0 … D8, #1363 » devient fausse,
  `pdf.rs:400-403`) ; doc-comment de `InvoiceLinePdf` (`types.rs:196` : « au-delà de 45 caractères, réduite à 44 et « … » ») ;
  `README.md` de la crate (`README.md:20` : « shortened to 44 and "…" ») — trois phrases devenues fausses, réécrites ;
  ré-export dans `lib.rs`. **Ne pas** toucher `largeurs.rs` (`largeur_mm` est `pub(crate)`, suffisant dans la crate).
- [ ] **`kesh-api`** : dans `validate_invoice_handler`, avant `invoices::validate_invoice` : lecture du brouillon
  (`find_by_id_with_lines`, scopée société ; un statut autre que `draft` laisse parler la validation, qui rend son refus
  habituel), contrôle des caractères puis de la capacité (D4, D6) ; deux variantes d'`AppError` et leurs réponses
  (`errors.rs`, à côté de `InvoiceTooManyLinesForPdf`), phrases par `t_args` ; rien d'autre dans `routes/invoices.rs` (la 49-1c
  y touche `validate_line` et la garde : hunks distincts).
- [ ] **`kesh-i18n`** : deux clés × quatre catalogues, bloc de l'Epic 49 ; `textes_coherents.rs` (deux entrées).
- [ ] **Tests** T0 à T10, T-PROP, T-SRC, avec leurs rouges.
- [ ] **Documents** (AC7, AC8, D9) : `docs/api-external.md`, `admin-manual.tex:2559`, puis — après la fusion de la 49-1c, ou
  selon D9 — `user-manual.tex` (`:1456`, `:1594`, phrase de la dérogation) ; contrôles de l'AC8, sortie copiée.
- [ ] **E2E et captures** (AC9, AC10) : le fichier Playwright, son rouge (T9), les captures relues.
- [ ] **Gates** (voie A, base et Rust ; aucune ligne de `kesh-db` : pas d'exception `kesh-db`). En cours de boucle, gate ciblé :
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -E 'package(kesh-qrbill)
  | binary(kesh_api) | binary(pdf_avant_apres_e2e) | binary(pdf_designations_e2e) | binary(invoice_validation_pdf_e2e) |
  binary(invoice_pdf_e2e) | binary(invoice_frozen_pdf_e2e) | binary(invoice_send_email_e2e) | binary(dunning_reminders_e2e) |
  binary(invoice_line_types_e2e) | binary(textes_coherents) | binary(configuration_transmise)'` ; frontend, parce que les
  catalogues changent : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit` (sous `mem-guard`). **Au passage en
  `review`** : `scripts/test-fast.sh --ci` puis `scripts/gate-ci.sh kesh_ci_e494`, bases `kesh_e494` remises à neuf (CLAUDE.md
  § *Base de gate*, sans redémarrer le conteneur : bases propres à la story). **À la clôture** (dernier commit de code) :
  `gate-ci` complet et **E2E Playwright complet** sur `kesh_e2e_e494` (montage `docs/testing.md` § *Prérequis Playwright local*),
  « vert » = les sept KF-029 seulement, jugés fichier par fichier. Une commande lourde à la fois, `scripts/mem-guard.sh --status`
  avant chacune, `flock` du verrou de la voie.
- [ ] **Contrôle des mots-clés** : `git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B | /usr/bin/grep -niE
  '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` vide.

## Note de version

*(Texte pour le `CHANGELOG`, assemblé à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

- **Factures et avoirs — désignation entière sur le PDF** : la désignation d'une ligne s'imprime entière, sur plusieurs lignes au
  besoin, au lieu d'être coupée à 44 caractères suivis de « … » ; le signe moins (U+2212) et les traits d'union U+2010 à U+2012
  s'impriment en trait d'union, et les lettres comme Č ou ł par leur lettre de base, au lieu de disparaître (`refs #1363`,
  `refs #1400`).
- **Factures — contrôle à la validation** : une facture dont les lignes ne tiendraient pas sur la page de son PDF est refusée à
  la validation, en le disant (`400 INVOICE_LINES_DO_NOT_FIT_PDF`), quand le brouillon se corrige encore, au lieu de devenir
  impossible à imprimer après ; une ligne qui porte un caractère que la police du PDF n'a pas (grec, cyrillique, émoji…) est
  refusée de même, en nommant la ligne et le caractère (`400 INVOICE_LINE_UNPRINTABLE_CHARACTERS`) (`refs #1364`, `refs #1400`).
- **Factures déjà émises** : un PDF archivé n'est jamais réécrit. Une facture validée avant la 0.20.0 et jamais téléchargée, un
  PDF regelé, un avoir et un rappel s'impriment désormais avec la désignation entière ; s'ils ne tiennent plus sur la page, ils
  s'impriment comme avant. Les montants, la TVA et les écritures ne changent pas.

## Dev Notes

- **Hors périmètre, avec ce qui le porte** : identité de l'émetteur et bloc « Facture » tronqués → #1229 ; numéro de facture et
  jeu de la QR → #1046 ; police embarquée → #943 ; NFD des contacts → #1411 ; PDF sur plusieurs pages → #1374 ; phrase du refus au
  rendu → #1110 (D5) ; montants au-delà du filet → #1399 ; titre de section (`heading`) → 49-3, qui partira du plan de D4 (son
  rendu en évidence passe par la même fonction de plan).
- **Issues à ouvrir** (demandées à l'orchestrateur, une par problème) : (1) *PDF de facture : les conditions de paiement perdent
  sans avertissement les caractères que la police n'a pas* (S3, `pdf.rs:759`), P3 ; (2) *PDF de rappel : une facture qui tient
  juste sur sa page ne tient plus avec le bloc du rappel, et son rappel est refusé* (S2, `reminder_block_height`), P3, comportement
  de la 0.19.0 ; (3) *Validation d'une facture : le contrôle du PDF précède la transaction (course avec un `PUT` concurrent du
  brouillon)* — seulement si la revue juge la course de D4 à tracer (`technical-debt`, P4).
- **Cohérence avec la 49-1d (sa D8)** : la 49-1d écrit « au même dessin qu'en 0.19.0 pour des désignations de 44 caractères au
  plus ». La règle exacte (D3) est plus étroite **et** plus large : *au plus 45 caractères, tenant dans 88 mm, en caractères que
  WinAnsi porte, sans blanc autre que l'espace, ni tiret de D6, ni lettre des 125*. Pour que sa phase 6 reste verte avec la 49-4,
  les désignations de F2 à F8 et de A1 doivent suivre cette règle (des mots français ordinaires la suivent ; « Remise −50 » ou
  « Čolić » ne la suivent pas). Sa phrase du manuel d'administration et sa note de version, conditionnées à la 49-4, nomment
  les quatre chemins de D3 (jamais téléchargée, regel, avoir, **rappel**) et la règle (2) des caractères, pas seulement la
  longueur. **À porter par l'orchestrateur à la fiche de la 49-1d** (cette fiche n'écrit pas dans son worktree).
- **Fichiers communs avec les autres stories de la release** :
  - `crates/kesh-qrbill/src/pdf.rs` : la **49-3** (rendu en évidence du titre) réécrira le même bloc de lignes ; elle part de la
    branche d'épique après la 49-4, ou le second arrivé résout.
  - `crates/kesh-api/src/routes/invoices.rs` : la **49-1c** (garde `comment`, `verify_vat_rates_against_db`) et la **49-3**
    (garde `heading`) ; la 49-4 n'y touche que `validate_invoice_handler` : hunks distincts.
  - `docs/manual/fr/user-manual.tex` : la **49-1c** (`:1456`, `:1594`, `:1924`) — D9 ; la PR #1327 (`:4082`, `:4181`) : distinct.
  - `docs/manual/fr/admin-manual.tex` § `sec:maj-0-20-0` : 49-1a, 49-1c, 49-1d ; la 49-4 ajoute une phrase au paragraphe « Pour
    les intégrations » ; le second arrivé garde les deux intentions.
  - `docs/api-external.md` : 49-1c (`:512`), 49-3 ; la 49-4 touche `:520` et la table des codes (`:1281`).
  - Catalogues i18n ×4 : toutes les stories ; bloc de l'Epic 49.
- **PR #1327 de Beat** (`gh pr diff 1327`, lecture seule ; ouverte, base `main`, branche `fix/pdf-montants-alignes`) :
  `crates/kesh-qrbill/src/largeurs.rs` (doc-comment, `largeur_mm` passe de `pub(crate)` à `pub`), `crates/kesh-qrbill/src/lib.rs`
  (doc de tête et `pub use largeurs::largeur_mm;` **juste avant** `pub use parser::…` et `pub use pdf::{…}`),
  `crates/kesh-qrbill/README.md` (lignes 6-7, positionnement), quatre catalogues (deux clés `reports-pdf-col-*`, bloc des rapports,
  fr-CH `:2165`), `docs/manual/fr/user-manual.tex` (`:4082`, `:4181`, PDF des rapports), `kesh-report/`. **Recoupements** :
  (1) `lib.rs` — la 49-4 étend le ré-export de `pdf::{…}` (`lib.rs:24-27`) et peut-être de `types::{…}` : **hunk voisin** de
  la ligne ajoutée par Beat, conflit de texte possible, sans contradiction d'intention ; (2) `README.md` de la crate : la 49-4 réécrit
  la ligne 20, Beat les lignes 6-7 — hunks distincts ; (3) catalogues : blocs distincts ; (4) `largeurs.rs` : la 49-4 n'y touche pas (AC12), elle se sert de `largeur_mm`
  dans la crate, `pub(crate)` ou `pub` ; (5) `user-manual.tex` : sections distinctes. Aucun recoupement de **règle** : #1327
  aligne les montants des PDF de **rapports** (`kesh-report`), la 49-4 coupe les désignations du PDF de **facture**. La note
  de Beat au manuel (`:4181`, « le PDF abrège seulement, d'un « … », les textes qui dépassent ») parle des rapports : vraie
  avant et après la 49-4. Ordre : #1327 dans `main` pour la 0.20.0 ; l'épique 49 la reprend à l'intégration ; si la 49-4 est
  close avant, elle ne se bloque pas, le second arrivé résout `lib.rs`.
- **48-11 (arrondi au centime, #1245)** : vérifié, elle **ne touche pas** le PDF de facture. Son tableau des fichiers
  (`epic-48-corrections-0-20-0.md:204`) nomme `kesh-core/src/types/money.rs`, `kesh-report/src/pdf.rs`, `kesh-report/src/csv.rs`,
  `kesh-api/src/exports/csv_tables.rs`, `kesh-api/src/routes/bank_statements_manual.rs` ; `kesh-qrbill/src/generator.rs:115` y est
  classé **sain** (`:91`, « arrondissent avant ») ; le PDF de facture formate par `pdf.rs:1791` `pub fn format_ch(value: Decimal,
  decimals: u32) -> String {`, qui arrondit (`:1792`, `round_dp_with_strategy(…, MidpointAwayFromZero)`). Aucun fichier commun.
- **Splitting** (CLAUDE.md, au grain de la crate) : `kesh-qrbill`, `kesh-api`, `kesh-i18n`, documents (`docs/manual/fr`,
  `docs/api-external.md`) — **quatre modules**, plus des tests (Rust et un fichier Playwright, aucun fichier de `frontend/src`).
  Sous le plafond de cinq. **Charge** (estimation de la rédaction) : étape 0 et capture une demi-journée ; `kesh-qrbill` (texte de
  ligne, plan, repli, contrôle) un jour et demi ; `kesh-api` et i18n une demi-journée ; tests et mutations un jour ; manuel,
  E2E, captures une demi-journée à un jour ; gates une demi-journée ; **quatre à cinq jours** plus la boucle de revue, pour une
  coupure au jeudi 2026-11-26. **Serré.** Découpage possible, **non tranché ici** (au rapport) : 49-4a (#1363 et #1364 : D1 à D5,
  D8) et 49-4b (#1400 : D6) ; la 49-4b peut sortir sans emporter la 49-1c (la règle de sortie de Guy porte sur la capacité :
  « un commentaire long ne doit pas rendre une facture validée impossible à imprimer ») ; si #1400 sort, la phrase de la 49-1c
  (« disparaît du PDF sans avertissement », conseil du trait d'union) **reste vraie** et le manuel ne la reprend pas, et #1400
  part entière à une release ultérieure (règle « une issue, une release »).
- **Manuels** : `git grep -n -i "tronqu\|44 caract\|trop de lignes\|huit lignes" -- docs/manual/fr/*.tex` ne nomme aucune phrase
  sur la désignation d'une ligne de facture à `3379a971a` ; les phrases à reprendre sont celles de la 49-1c (D9) et `:1594`
  (« libellés des lignes … n'a pas ce repli »). Manuels DE, EN, IT : n'existent pas (`docs/manual/de/README.md:3`).
- **Ce qui doit rester vrai** : (1) un PDF archivé n'est jamais réécrit ; (2) aucune pièce d'avant ne devient impossible à
  imprimer (D2) ; (3) les pièces des tests de la 49-1b hors S7 se rendent à l'identique ; (4) la validation n'exige rien de plus
  qu'aujourd'hui pour une facture qui tient et s'imprime ; (5) aucune ligne de `kesh-db`.
- **Pièges** : (a) le plan de D4 doit être **celui** du dessin, non une copie (T-PROP) ; (b) `recap_lines` avec
  `QrBillI18n::default()` ne sert qu'à compter ; (c) l'arrondi du contrôle se décide au centime comme `recap_lines`
  (`pdf.rs:1898-1903`) ; (d) le repli de D2 rend le code d'aujourd'hui **à l'octet** : ne pas « améliorer » ce chemin ; (e) la
  ligne nommée par le refus est la **position** (1, 2…), comme `validate_line` (`Ligne {} :`) ; (f) `kesh-seed` valide par le
  dépôt (`pieges.rs:590`) : ses factures passent par D2 si elles ne tiennent pas — angle mort assumé (démonstration) ; (g) un
  caractère hors du plan de base (U+10000 et au-delà : émojis) est un `char` Rust comme un autre : T-SRC balaie le plan de base,
  un test nommé ajoute deux émojis.
- **Aucune donnée réelle** de Guy dans la fiche, les tests, les fixtures, les captures ou les journaux : tout est synthétique.
  `~/kesh/` n'est ni lu ni écrit.
- **Références** : `epic-49-types-de-lignes-facture.md` (ligne 49-4, règles de sortie, § *Contraintes de production*),
  `release-0.20.0.md`, fiches `49-1b-types-de-lignes-pdf.md` (D4, D5, D6, D8), `49-1c-types-de-lignes-ecran.md` (D10, D12, AC10,
  Dev Notes « La 49-4 »), `49-1d-types-de-lignes-recette-montee.md` (D8), #1363, #1364, #1400, #1110, #1327,
  `kesh-gate-logs/prompts/e0200-commun.md`, `e0200-fiche.md`.

## Dev Agent Record

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | **Création de la fiche** (Opus 5.5, `bmad-create-story`). Code lu à `3379a971a` (49-1a et 49-1b fusionnées). Dix décisions : la désignation se coupe à 88 mm, montants à la première ligne (D1) ; une pièce d'avant qui ne tiendrait plus se rend comme en 0.19.0 (D2) ; dérogation écrite, étendue à l'avoir et au rappel, jamais figés — le rappel à confirmer par Guy (D3) ; capacité contrôlée à la validation par le plan même du rendu, course assumée, `kesh-db` intouché (D4) ; code neuf `INVOICE_LINES_DO_NOT_FIT_PDF` (D5) ; quatre tirets repliés, cinq invisibles retirés, tout autre caractère sans glyphe refusé à la validation, source unique (D6) ; aucun changement d'écran (D7) ; preuve avant / après par le mécanisme de la 49-1b, comparateur et fixtures gelés, S7 seule exclue (D8) ; manuel après la fusion de la 49-1c (D9) ; aucune écriture en base (D10). Treize AC, treize tests. Quatre modules. Aucune migration. Issues à ouvrir : deux (trois si la revue le juge). Validation P1 à lancer : `49-4-designation-longue-pdf-validation-prompt-p1.md`. Statut : `ready-for-dev`. |
