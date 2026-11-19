# Story 49-1b : types de lignes de la facture client — le rendu PDF d'un commentaire, pour la facture et l'avoir (#1242)

Status: ready-for-dev

*Fiche rédigée le 2026-11-19 par l'agent de rédaction de la 49-1b (Sonnet 5.5), par le workflow `bmad-create-story`.
Release **0.20.0** (coupure jeudi 2026-11-26 au soir). **Aucune migration**, aucune clé i18n, aucun changement d'écran ni de
contrat d'API. Code lu à la tête de la branche `story/49-1b-types-de-lignes-pdf` = `cbdd70026` (la 49-1a fusionnée dans
`epic/49-types-de-lignes-facture`). Sources : `epic-49-types-de-lignes-facture.md` et `release-0.20.0.md` (le kickoff est
dans `main`, `e2801cf18`, pas encore dans la branche : lus au dépôt principal, en lecture seule),
`kesh-gate-logs/e491/decoupage-49-1.md`, la fiche close `49-1a-types-de-lignes-migration-api.md`. Les choix de cette fiche
sont D1 à D10. Validation : passes P1 à P3 faites, voir le Change Log.*

**Issues** : `refs #1242` ; **aucun mot-clé de fermeture** dans un commit. #1242 ne se ferme pas avec cette story.

## Story

En tant que Guy, qui a déjà émis de vraies factures remises sur papier à un locataire et qui en émettra d'autres avant la fin
novembre, je veux que le PDF d'une facture **et** d'un avoir sache imprimer une **ligne de commentaire** — du texte sur toute
la largeur, sans colonnes de quantité, de prix, de TVA ni de total — **sans que le PDF d'aucune facture ni d'aucun avoir faits
de lignes chiffrées change d'un trait, d'une position ou d'un caractère**.

Cette story ne rend rien visible à l'utilisateur : la garde de l'API (`LineType::open_to_api()`, 49-1a) refuse encore `comment`
et `heading`. Elle pose le rendu **avant** que la 49-1c ouvre `comment` (couplage de coupure : *la 49-1c ne sort pas sans la
49-1b*, `decoupage-49-1.md` § 3). Elle est aussi la story où la consigne de production du kickoff se **prouve** : le test
avant / après du rendu d'une facture et d'un avoir de la 0.19.0 (`epic-49`, § *Contraintes de production*, point 3).

## Ce qui se lit au code (`cbdd70026`)

Chaque ligne citée est copiée du fichier, lue à la tête du worktree.

### Le rendu d'une ligne : un seul gabarit, partagé par la facture, l'avoir et le rappel

- `crates/kesh-qrbill/src/pdf.rs:405` : `fn draw_invoice_section(` — **le seul** endroit qui dessine les lignes. Il est appelé
  par `generate_qr_bill_pdf_with_date` (facture et rappel, plancher `SEP_Y`) et par `generate_credit_note_pdf_with_date`
  (avoir, plancher `CONTENT_FLOOR_NO_QR` = 15 mm, `pdf.rs:191`).
- `pdf.rs:564` `    let col_desc = left;` … `pdf.rs:568` `    let col_tot = left + 160.0;` : les cinq colonnes (description à
  20 mm, quantité à 110, prix à 130, TVA à 160, total à 180). Le filet du tableau court de `left` à `PAGE_W - 20.0`
  (`pdf.rs:607` `    hline(layer, left, PAGE_W - 20.0, ty);`) : **170 mm utiles** pour une ligne pleine largeur.
- `pdf.rs:628` `    for line in &inv.lines {`, puis `pdf.rs:629` `        if ty < content_floor + 15.0 + recap_reserve {` et
  `pdf.rs:634` `            return Err(QrBillError::TooManyLines(inv.lines.len()));` : la garde de capacité, **avant** chaque
  ligne dessinée ; pas de ligne = 5 mm. Premier `ty` = `PAGE_H - 130.0 - 5.0 - 3.0` = 159 mm. Sur une **facture** (plancher
  105 + 15 = 120 mm), **huit** lignes tiennent sans récapitulatif de TVA, moins avec ; sur un **avoir** (plancher 15 + 15 = 30
  mm), vingt-six.
- `pdf.rs:637` `            truncate_display(&line.description, 45),` : **la description d'une ligne chiffrée est réduite à
  44 caractères suivis de « … »** (`truncate_display`, `pdf.rs:1890`, qui prend `max_chars - 1` caractères puis pose `…`,
  imprimée : `largeurs.rs:146` `(0x85, '\u{2026}', "ellipsis"),`). La coupe se **voit** sur la pièce, mais rien ne
  l'annonce à la saisie et le texte entier n'y figure pas. La 49-1b n'y touche pas (D8, hors périmètre ; issue déjà ouverte).
- `pdf.rs:643-662` : la quantité, le prix, `format!("{}%", …)` et le total se dessinent à `col_qty`, `col_unit`, `col_vat`,
  `col_tot` — **sans condition**. Une ligne non chiffrée y passerait en `0.00 … 0.00%  0.00`.
- `pdf.rs:1778` `fn recap_lines(inv: &InvoicePdfData, i18n: &QrBillI18n) -> Vec<RecapLine> {` : le récapitulatif lit
  `inv.vat_lines`, `inv.subtotal_ht` et `inv.rounding`, **jamais** `inv.lines` : un commentaire n'y entre que par les sommes
  que l'appelant a faites (ci-dessous).
- `pdf.rs:107` `    let helv_bold = doc` : le document porte **deux polices intégrées**, `Helvetica` et `Helvetica-Bold`, ajoutées
  dans tous les cas. **Ajouter une troisième (Oblique, par exemple) changerait le dictionnaire de ressources de tout PDF**,
  chiffré ou non : D3.
- `crates/kesh-qrbill/src/largeurs.rs:509` `pub(crate) fn largeur_mm(` : la largeur imprimée d'un texte à une taille donnée,
  par la table AFM d'Helvetica ; `pdf.rs:1077` `fn largeur_valeur(` la fixe à 8 pt et `pdf.rs:1093` `fn couper_ligne(` s'en
  sert pour couper l'adresse de la section de paiement « sans rien tronquer » (Story 44-1a, D3). La coupe est faite à 8 pt ;
  la 49-1b en a besoin à **9 pt**.
- `crates/kesh-qrbill/src/types.rs:191` `pub struct InvoiceLinePdf {` : `description`, `quantity`, `unit_price`, `vat_rate`,
  `line_total` ; **aucun champ de type**. Réexportée `lib.rs:27` (`Address, AddressType, Currency, InvoiceLinePdf, …`).
- La capacité du rappel (`reminder_block_height`) et du récapitulatif est déjà **réservée** dans `recap_reserve` : une ligne
  de commentaire ne change aucun de ces blocs.

### Qui construit les `InvoiceLinePdf`

- `crates/kesh-api/src/routes/invoice_pdf_service.rs:396` `    let invoice_lines_pdf: Vec<InvoiceLinePdf> = lines` (facture et
  rappel : `build_qrbill_inputs`) et `crates/kesh-api/src/routes/credit_notes.rs:310` `    let pdf_lines: Vec<InvoiceLinePdf> = lines`
  (avoir) : **deux fermetures jumelles**, sans filtre ni type, qui recopient les cinq champs.
- `invoice_pdf_service.rs:408` `    let subtotal_ht: Decimal = lines.iter().map(|l| l.line_total).sum();` et `credit_notes.rs:330` (même
  ligne) : le sous-total somme **toutes** les lignes ; `vat_lines_pdf(lines.iter().map(|l| (l.line_total, l.vat_rate)))`
  (`invoice_pdf_service.rs:409`) délègue à `kesh_core::accounting::vat::vat_breakdown_by_rate`, qui écarte tout taux nul :
  `crates/kesh-core/src/accounting/vat.rs:132` `        if vat_rate <= Decimal::ZERO {`. **Une ligne non chiffrée porte
  `line_total = 0` et `vat_rate = 0` (contrainte `chk_*_unpriced_neutral`, 49-1a)** : elle n'ajoute rien à ces sommes. La
  49-1a n'a donc **pas** filtré ces deux sites ; la contrainte de base les protège (D9) ; AC5 (a) vérifie seulement, par un test, que la comparaison `[A,C,B]` = `[A,B]` lirait un écart si l'un de ces sites en introduisait un.
- `invoice_pdf_service.rs:206` `    if lines.len() > MAX_LINES_PER_PDF {` (constante `= 9`, ligne 66) : le pré-filtre grossier d'une
  facture compte **toutes** les lignes, commentaires compris ; `credit_notes.rs:289` `    // Pas de pré-filtre \`MAX_LINES_PER_PDF\` ici :
  un avoir n'a PAS de section QR` : l'avoir n'en a pas, la garde de `pdf.rs` décide.
- `invoice_pdf_service.rs:510` `        QrBillError::TooManyLines(n) => return AppError::InvoiceTooManyLinesForPdf(*n),` : le refus de
  capacité devient `400 INVOICE_TOO_MANY_LINES_FOR_PDF`, aussi pour le courriel et le lot de rappels (`invoice_email.rs:1050`).

### Les chemins du PDF figé

- `crates/kesh-api/src/routes/issued_invoice_pdf.rs:104` `    if is_frozen(&invoice) {` : une facture au PDF figé **sert le fichier
  archivé** (`serve_frozen`), sans rendre ; une facture validée jamais téléchargée **se rend** (`render_and_pose`) puis se fige.
  `issued_invoice_pdf.rs:238` `pub async fn refreeze(` : le regel d'un fichier **absent** rend de nouveau avec le code en
  fonction. Ces deux derniers chemins, **et eux seuls**, reçoivent le rendu neuf (D7).
- **Le PDF d'un avoir n'est jamais figé** : `credit_notes.rs:362` `        kesh_qrbill::generate_credit_note_pdf(&pdf_data, &i18n)…`
  le rend à chaque demande. Tout avoir, ancien ou neuf, se rend donc avec le code en fonction : **le test avant / après de
  l'avoir est d'autant plus nécessaire**.

### Où tourne le même rendu, et ce qui prouve qu'il n'a pas bougé depuis la 0.19.0

- `git diff --stat 773df80da..HEAD -- crates/kesh-qrbill crates/kesh-api/src/routes/issued_invoice_pdf.rs` : **vide**. La
  49-1a n'a touché aucun fichier de `kesh-qrbill` ni le gel (AC14 de la 49-1a). Dans les trois fichiers de `kesh-api` qui
  construisent le PDF, `line_type: kesh_db::entities::LineType::Priced,` dans `invoice_pdf_service.rs` et `invoice_email.rs` (littéraux de
  tests), et dans `credit_notes.rs` l'import de `LineType` et le champ `line_type` du type de la réponse de l'avoir
  (`line_type: l.line_type,`), hors rendu. **Le rendu de
  `cbdd70026` est donc celui de la 0.19.0** : la référence « avant » se capture sur ce commit, avant tout changement (D4),
  et la commande ci-dessus se rejoue à la capture. **Limite dite** : la 49-1a a changé les lectures de `kesh-db`
  (colonnes de lignes, reprise des lignes de l'avoir, même `ORDER BY position`) ; l'équivalence au niveau HTTP n'est prouvée
  que **depuis `cbdd70026`** ; la preuve 0.19.0 → 0.20.0 sur base peuplée est celle de la 49-1d.
- `pdf.rs:1994` `    fn textes_du_pdf(bytes: &[u8]) -> Vec<(f32, f32, String)> {` (module de tests) relit le contenu d'un PDF — `Content::decode`,
  `Tj`, position du `Td` — et `pdf.rs:2667` `    fn fixed_date() -> printpdf::OffsetDateTime {` fige la date. `tests/golden_test.rs:3-6` : *« on ne
  compare pas l'octet exact du PDF (printpdf introduit un random dans le second élément du trailer `/ID`) »* ; `pdf.rs:55-57`
  le dit de même. `pdf.rs:55-57` dit « byte-identical *modulo* … `/ID` » : l'égalité des octets **n'est possible qu'une fois ce second élément
  neutralisé** ; D4 retient d'abord la comparaison du contenu, et permet l'octet neutralisé si l'étape 0 le montre stable.

## Inventaire par le symptôme

Symptôme : *une ligne de facture ou d'avoir se dessine ligne par ligne, avec ses montants*. Commandes rejouées à `cbdd70026`.

```
$ git grep -nE "InvoiceLinePdf \{" -- crates
crates/kesh-api/src/qr_decode.rs:885:            lines: vec![InvoiceLinePdf {
crates/kesh-api/src/routes/credit_notes.rs:312:        .map(|l| InvoiceLinePdf {
crates/kesh-api/src/routes/invoice_pdf_service.rs:398:        .map(|l| InvoiceLinePdf {
crates/kesh-qrbill/examples/capture_qr_facture.rs:186:        lines: vec![InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:1955:            lines: vec![InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:2930:                .map(|i| InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:3030:                .map(|i| InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:3088:                InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:3136:            .map(|i| InvoiceLinePdf {
crates/kesh-qrbill/src/pdf.rs:3172:            .map(|i| InvoiceLinePdf {
crates/kesh-qrbill/src/types.rs:191:pub struct InvoiceLinePdf {
crates/kesh-qrbill/tests/golden_test.rs:70:        lines: vec![InvoiceLinePdf {
```

12 lignes = 11 littéraux + la définition. **Tous dans le périmètre** : l'ajout du champ de type les change (D2). Deux sont des
sites de **production** (`invoice_pdf_service.rs:398`, `credit_notes.rs:312`) ; `qr_decode.rs:885` est un test.

```
$ git grep -n "invoice-pdf-description" -- 'crates/*'      (la colonne « Description » : un seul dessin ; sortie abrégée, s'y ajoutent `types.rs:269` (`I18N_KEYS`) et les quatre catalogues `messages.ftl:720` / `:777`)
crates/kesh-qrbill/src/pdf.rs:571:        i18n.get("invoice-pdf-description"),
$ git grep -n "generate_qr_bill_pdf\b\|generate_credit_note_pdf\b" -- 'crates/*/src/*' ':!crates/kesh-qrbill/src/pdf.rs'   (les appelants)
crates/kesh-api/src/routes/credit_notes.rs:362:        kesh_qrbill::generate_credit_note_pdf(&pdf_data, &i18n).map_err(map_qrbill_error)?;
crates/kesh-api/src/routes/invoice_pdf_service.rs:244:    let bytes = kesh_qrbill::generate_qr_bill_pdf(&qr_data, &pdf_data, &qr_i18n)
```

(`qr_decode.rs:900` appelle la variante `_with_date`, en test.) Donc **deux** constructeurs de lignes, **un** dessinateur.

Autres rendus de lignes de facture, cherchés et **écartés** :

- **Courriel** : `git grep -nE "lines\.iter\(\)|\.lines\b" -- crates/kesh-api/src/routes/invoice_email.rs` →
  `invoice_email.rs:190:        lines.iter().map(|l| (l.line_total, l.vat_rate)),` : le corps du courriel ne liste **aucune**
  ligne (il porte le montant TTC, le numéro, l'échéance) ; la pièce jointe est le PDF de `get_or_freeze`
  (`invoice_email.rs:814`) : **même rendu**, rien à faire.
- **Aperçu à l'écran** : `git grep -nil "pdf" -- frontend/src` ne nomme que le téléchargement (`invoices.api.ts`) et des écrans
  d'autres domaines : **aucun aperçu de PDF** ; l'affichage des lignes à l'écran est la **49-1c**.
- **Exports PDF** : `git grep -n "invoice_lines\|credit_note_lines" -- 'crates/kesh-report/src/*'` →
  `vat_report.rs:123: FROM invoice_lines il` seulement (le rapport TVA, filtré par la 49-1a) ; **aucun PDF de rapport ne liste
  les lignes d'une facture**. Les PDF de rapports sont l'objet de la PR #1327 de Beat, étrangère à cette story.
- **Exports CSV, MCP** : portés par la 49-1a (colonne `line_type`, champ `typeLigne`).

Aucun site hors périmètre sans issue : le seul fait voisin est la réduction à 44 caractères et « … » de `pdf.rs:637` (D8, #1363).

## Décisions

- **D1 — Le titre de section (`heading`) est rendu comme un commentaire en 49-1b ; sa mise en évidence est la 49-3.**
  Le découpage (`decoupage-49-1.md`, § 2) et l'épique (§ 49-3) réservent à la 49-3 *« rend le titre en évidence (PDF) »* ;
  la 49-1b ne dessine donc ni gras, ni filet. Mais le rendu ne doit **jamais** laisser un `heading` s'imprimer
  `0.00 … 0.00% 0.00` : il peut exister en base (dépôt, import, SQL), l'API ne le refuse que par la garde. **`Heading`
  prend le chemin du commentaire** (texte pleine largeur), que la 49-3 remplace par la mise en évidence. *Écartées* :
  (a) refuser un `heading` au rendu (un PDF de facture validée qui ne se produit plus : pire que moche) ; (b) le rendre en
  gras dès maintenant (préjuge du dessin de la 49-3 et de ses tests) ; (c) ne pas distinguer `heading` de `comment` dans le
  type de `kesh-qrbill` (la 49-3 aurait à ajouter la variante et à retoucher les deux fabricants : on la pose maintenant, en
  une ligne de `match`).
- **D2 — Un champ `kind: LinePdfKind` à `InvoiceLinePdf`**, énumération **propre à `kesh-qrbill`** (`Priced`, `Comment`,
  `Heading`), non le `LineType` de `kesh-db` (`kesh-qrbill` ne dépend que de `kesh-core`, `Cargo.toml` : DD-14, jamais de
  `kesh-db`). Le champ est obligatoire : **9 littéraux à mettre à jour** (inventaire : les 11 moins les deux de `kesh-api` en production, `invoice_pdf_service.rs:398` et `credit_notes.rs:312`, **remplacés** par `ligne_pdf`, D10), aucun oubli possible (le compilateur le
  dit). *Écartées* : (a) rendre `quantity`, `unit_price`, `vat_rate`, `line_total` optionnels (`Option<Decimal>`) — le chemin
  chiffré, celui qui **ne doit pas bouger**, changerait de types partout ; (b) un `Default` sur `InvoiceLinePdf` (un champ
  oublié passerait en silence `Priced`, justement ce que l'on veut éviter pour `heading`) ; (c) **filtrer** les commentaires
  côté API et ne rien passer au PDF (ils disparaîtraient de la facture imprimée). *Pas de variante `Subtotal`* : hors 0.20.0
  (#1330).
- **D3 — Le commentaire se dessine en `Helvetica` 9 pt, aucune police neuve.** Ni gras ni italique : le document ne gagne
  **aucun objet** de ressources ; le dictionnaire de polices d'un PDF chiffré-seulement reste identique (AC1, qui le compare).
  *Écartée* : un commentaire en italique (`BuiltinFont::HelveticaOblique`) — plus lisible comme « note », mais ajoute une
  police à **tout** PDF (si l'ajout est inconditionnel) ou la crée à la demande (un second chemin de création du
  document) ; au choix de la 49-3 ou d'une story ultérieure.
- **D4 — Le test avant / après compare le CONTENU rendu, non les octets du fichier.** Le `/ID` change à chaque rendu
  (`pdf.rs:55-57`, `golden_test.rs:3-6`) : l'égalité brute des octets est exclue. La référence est donc, par scénario, (a) le
  **SHA-256 de la suite complète des opérations de contenu** de la page (chaque opérateur et ses opérandes, dans l'ordre :
  textes, positions, polices, traits, rectangles du QR, couleurs — tout ce qui se dessine), (b) le **dictionnaire complet de chaque police** (`BaseFont`, `Encoding`…), la `MediaBox` et le nombre de pages
  (le dictionnaire `Info` porte la date : exclu et dit tel), (c) en lisible, la liste `(x, y, texte)` de
  `textes_du_pdf`, pour qu'un rouge **se lise** et ne soit pas qu'un condensat. Le fichier de référence est un fichier texte
  du dépôt, diffable. **Un seul comparateur** calcule cette empreinte : il est exposé par `kesh-qrbill` **seulement** sous une feature
  `test-support` (module `crates/kesh-qrbill/src/empreinte.rs`, activée pour les tests de la crate par une **auto-dev-dépendance** `kesh-qrbill = { path = ".", features = ["test-support"] }`, et pour `kesh-api` par sa dev-dépendance ; **jamais `required-features`**, qui sauterait T1 en silence hors workspace) (pas de `#[doc(hidden)] pub`), appelé par T0,
  T0-bis, T1, T1-bis, T3, T9 et T10, et par les helpers de T4 à T8. **API publique énumérée, figée dans C** :
  `textes(&[u8]) -> Vec<(f32, f32, String)>`, `operations(&[u8]) -> Vec<Operation>`, `polices(&[u8]) -> String` (dictionnaire
  complet de chaque police), `media_box(&[u8]) -> String`, `nombre_de_pages(&[u8]) -> usize`, `empreinte(&[u8]) -> Empreinte`
  (condensat des opérations, polices, `MediaBox`, pages, textes) avec son écriture et sa relecture en texte ; toute fonction
  manquante se pose **dans C**, non plus tard. **Contrôles de la production** : `cargo tree -e normal -p kesh-api -f '{p} {f}'`
  ne montre pas `test-support` ; `cargo build --release -p kesh-api -v` (la construction du `Dockerfile:6`) montre une ligne
  `rustc` de `kesh_qrbill` **sans** `feature="test-support"` (la ligne n'apparaît que si la crate est **recompilée** : worktree neuf, ou
  `touch crates/kesh-qrbill/src/lib.rs` avant ; journal copié). Pas de sonde sur les chaînes du binaire : des littéraux
  « empreinte » y figurent déjà en production (`document_storage.rs:336`, `issued_invoice_pdf.rs:378`…). (`cargo build --workspace --all-targets`, celui de la CI,
  **unifie** la feature dans le build de test : inoffensif, non livré.) Les helpers
  privés de `pdf.rs::tests` (`textes_du_pdf`, `operations_du_pdf`, `texte_du_tj`) se **rebranchent** sur lui **sans le
  modifier**. **Le comparateur est gelé avec les fixtures** : il entre dans le commit C et n'est plus modifié
  ensuite (AC1) ; un test (T1-bis) le prouve **rouge**, composante par composante (paires de PDF fabriqués). Aucune
  seconde implémentation (ni recopie dans `kesh-api`, ni `pdf_contains` comme comparateur : DRY).
  **La référence est capturée avant tout changement du rendu, dans un worktree propre et détaché**, jamais dans l'arbre de
  travail du développement (un changement de rendu non commité y serait compilé et deviendrait « la référence »). Le
  graphe est le suivant, chaque étape étant décrite à l'étape 0 : un **commit C** (comparateur, `lib.rs`, `Cargo.toml` ×2,
  `Cargo.lock`, tests de capture, **sans fixtures**) posé **au-dessus de la tête de la branche** (qui porte déjà les commits de la fiche ; tous les diffs se mesurent **contre `cbdd70026`**, la fiche étant hors `crates`) ; un **worktree détaché à C**, où T0 et T0-bis
  écrivent les références ; un **commit F** des seules fixtures, au-dessus de C ; **seulement ensuite** le code de rendu.
  Les tests de capture ont deux modes, par la variable d'environnement `KESH_REFERENCES` : `ecrire` (échoue si une référence
  existe déjà) et `comparer` (relit les références et vérifie qu'un second rendu les égale : la **stabilité**). En outre,
  T0 et T0-bis inscrivent chacun un `sources.sha256` (un par dossier de fixtures) avec le **SHA-256 de chaque fichier de rendu** lu à l'exécution
  (liste à l'AC1), que l'AC1 compare à `git show cbdd70026:<fichier> | sha256sum` : un fichier de rendu modifié au moment
  de la capture se voit a posteriori. AC1 exige que les fixtures et le comparateur ne bougent plus après F.
  *Écartées* : (a) capturer sur un
  worktree de `773df80da` (20 à 50 Go de cible cargo, pour un résultat prouvé identique par un `git diff` vide) ; (b) ne
  comparer que `textes_du_pdf` (texte et positions) — il laisse passer un trait, une couleur ou une police qui bouge ;
  (c) comparer la taille en octets (la méthode des tests de la 16-3a : un delta de taille ne voit pas un déplacement).
  **Étape 0** : générer chaque scénario **deux fois** et vérifier que le condensat est stable ; s'il ne l'est pas (un
  identifiant interne au contenu), la cause se **nomme** et se retire de la comparaison, jamais une tolérance. Le
  développeur **choisit et justifie**, à l'étape 0, entre le condensat des opérations et l'octet du fichier neutralisé du
  `/ID` si une relecture montre ce dernier stable (il serait plus fort, et s'écrirait de même) : la fiche tient le
  condensat des opérations pour acquis, l'octet neutralisé comme amélioration permise.
- **D5 — Un commentaire se coupe, il ne se tronque pas.** Largeur utile : de `col_desc` (20 mm) à 190 mm = **170 mm**, à
  9 pt, mesurée par `largeur_mm(texte, 9.0, false)` ; texte passé d'abord par `texte_imprime` (la chaîne mesurée est la
  chaîne dessinée, consigne de `largeurs.rs`) ; coupe sur `\n` en paragraphes (les `\r` sont retirés avant, CRLF d'un copier-coller ; **tous** les `\n` finaux sont ignorés, ils ne créent pas de ligne blanche ; la tabulation est **remplacée par une espace avant la coupe, dans le seul chemin du commentaire** (jamais dans `couper_ligne_a`, dont le contrat reste celui de `pdf.rs:1089-1091`, ni dans le rendu d'une ligne chiffrée, qui passe par `truncate_display`, `pdf.rs:637` ; les deux appelants de `couper_ligne` ne reçoivent jamais de tabulation, `validation.rs:676` la refuse), puisque `couper_ligne` ne sépare les mots que sur `' '` (`pdf.rs:1099`) et que `lopdf` retire le caractère à l'écriture : sans cela deux mots se colleraient à l'impression ; testé en T5) (le champ accepte des sauts de ligne :
  `validate_line` ne fait que `trim`, `invoices.rs:507`), puis gloutonne à la frontière de mot, un mot plus large que la
  ligne se coupe au dernier caractère qui tient. **`couper_ligne` (8 pt, adresse de la QR) est généralisée par une taille**
  (`couper_ligne_a(ligne, largeur, taille)` ; `couper_ligne` en devient l'appel à `VALEUR_TAILLE`) — pas de seconde
  coupe : DRY. Un paragraphe vide donne une ligne vide (l'aération voulue par l'auteur) ; une description **entièrement
  blanche** ne dessine rien et ne consomme aucune ligne (la contrainte de base est `CHECK (CHAR_LENGTH(TRIM(description)) > 0)`, `20260416000001_invoices.sql:58`, et `TRIM`
  ne retire que les espaces : une description faite de `\n` la passe ; le rendu ne suppose donc ni la contrainte ni la
  validation). Pas de ligne = **5 mm**, comme une ligne chiffrée. *Écartées* : (a) la réduction
  de `truncate_display` (perte du texte sur la pièce, justement ce que les #1331 à #1334 retirent ailleurs) ; (b) ne
  couper qu'au caractère (coupe au milieu des mots).
- **D6 — La capacité se garde par ligne dessinée, avec la formule de la ligne chiffrée.** Chaque ligne de texte d'un
  commentaire passe par la garde de `pdf.rs:629` ; un dépassement rend **la même erreur** `TooManyLines(inv.lines.len())`
  (le compte est celui des lignes de la pièce, non des lignes de texte : la signature ne change pas). Aucune coupe, aucun
  chevauchement de la zone QR. *Écartées* : (a) une variante d'erreur neuve (`CommentTooLong`…) — elle exige une clé i18n
  dans quatre catalogues, un code d'erreur, le classement du courriel (`invoice_email.rs:1050`) et du lot de rappels : une
  story de plus, et la phrase actuelle est déjà l'objet de **#1110** ; (b) élargir `MAX_LINES_PER_PDF` ou le remplacer
  par un compte de lignes de texte (il resterait un pré-filtre ; la garde géométrique reste l'autorité). **Conséquence
  assumée et à dire** : un commentaire de mille caractères (la borne de `MAX_DESCRIPTION_LEN`, `invoices.rs:58`) fait une
  dizaine de lignes de texte et **ne tient pas** sur une facture (huit lignes au plus) : le PDF est refusé en `400`. C'est
  un défaut de **saisie** (le refus tombe après la validation de la facture, quand elle ne se corrige plus) : la 49-1c
  borne ou signale la longueur d'un commentaire (tâche à lui écrire, Dev Notes), et une issue le trace.
- **D7 — Le PDF archivé fait foi ; le code neuf ne rend que ce qui ne l'est pas déjà.** `issued_invoice_pdf.rs` **ne change
  pas** (`git diff` vide, AC8). Trois cas, chacun prouvé :
  (a) facture **au PDF figé** → le fichier archivé est servi tel quel, **même si le code neuf l'aurait rendu autrement** ;
  les tests existants de `invoice_frozen_pdf_e2e.rs` (dont `deux_telechargements_rendent_les_memes_octets` `:317` et
  `modifier_le_client_apres_le_gel_ne_change_pas_le_pdf` `:409`) en sont l'épreuve ; un test neuf (T12) pose un fichier
  archivé de contenu **différent** de ce que le code rendrait et le relit octet pour octet ;
  (b) facture validée **jamais téléchargée** → rendue par le code neuf : AC2 compare son contenu à la référence ;
  (c) **regel** après un fichier absent (`refreeze_invoice_pdf`) → rendu neuf, **égal à la référence** pour une facture
  chiffrée-seulement (AC2) ; pour une facture à commentaire, il imprime le commentaire.
  *Écartée* : « geler » rétroactivement toutes les factures validées par un script de migration — écrire dans les
  livres de production pour un cas que le contenu identique rend sans objet, et une migration de plus (interdite en 0.20.0).
- **D8 — Hors périmètre : la réduction à 44 caractères et « … » de la description d'une ligne chiffrée** (`pdf.rs:637`). La corriger
  changerait le rendu de factures existantes (une description de 46 caractères et plus s'imprimerait entière), donc le
  test avant / après et le PDF d'un avoir jamais figé ; c'est un défaut à part, à tracer (issue à ouvrir, Dev Notes), non à
  cacher dans cette story. Un commentaire, lui, n'est pas coupé (D5) : la différence de traitement entre les deux est
  **voulue** et se dit dans le doc-comment de `draw_invoice_section`.
- **D9 — Les sommes des appelants ne changent pas.** `subtotal_ht`, `vat_lines` et le TTC continuent de lire **toutes** les
  lignes : une ligne non chiffrée porte `0` (contrainte de la 49-1a) et `vat_breakdown_by_rate` écarte le taux `0`
  (`vat.rs:132`). Les filtrer (`filter(|l| l.line_type.is_priced())`) ajouterait une branche à un chemin **qui ne doit pas
  bouger** pour un gain nul ; la **contrainte de base** garantit la neutralité ; AC5 (a) vérifie que la comparaison `[A,C,B]` = `[A,B]` (pas un centime, pas de taux « 0 % » fantôme au récapitulatif) lirait un écart, ce que les mutations de T9 et T10 éprouvent. *Écartée* : filtrer par type aux deux sites (symétrique de ce que la 49-1a a fait pour les lecteurs qui
  **comptent** ou **écrivent** ; ici la somme est neutre par construction).
- **D10 — Un seul constructeur de ligne côté `kesh-api`.** Les deux fermetures jumelles (facture, avoir) se remplacent par
  une fonction `pub(crate) fn ligne_pdf(description, line_type, quantity, unit_price, vat_rate, line_total) -> InvoiceLinePdf`
  de `invoice_pdf_service.rs`, qui porte **le seul** `match` `LineType → LinePdfKind`, **exhaustif et sans joker** (un type
  de ligne neuf ne compile plus tant que son rendu n'est pas décidé). *Écartée* : conserver les deux fermetures et y
  ajouter `kind:` (deux endroits à tenir identiques : le défaut même que le commentaire de `credit_notes.rs:225-228` signale
  pour `InvoicePdfData`).

### Testabilité : un commentaire se rend sans ouvrir la garde

La garde `open_to_api()` **ne bouge pas** : `git diff cbdd70026..HEAD -- crates/kesh-db/src/entities/line_type.rs
crates/kesh-api/src/routes/invoices.rs` est **vide** (AC8), et le test T9 de la 49-1a (un `comment` par l'API rend `400`) reste
vert. Les tests de rendu fabriquent leurs lignes **sans l'API** :

- **niveau `kesh-qrbill`** : un `InvoiceLinePdf { kind: LinePdfKind::Comment, … }` en littéral ;
- **niveau `kesh-api`, facture** : `invoices::create(pool, user_id, NewInvoice { lines: vec![…, NewInvoiceLine { line_type:
  LineType::Comment, quantity: 0, unit_price: 0, vat_rate: 0, … }] })` — **le dépôt accepte les trois types** (49-1a, D9 ;
  `kesh-db/tests/invoice_line_types.rs:360` en fait déjà le geste) —, puis `invoices::validate_invoice` (modèle :
  `seed_validated_invoice`, `invoice_pdf_e2e.rs:175`, qui crée les lignes au dépôt et valide), puis
  `GET /api/v1/invoices/{id}/pdf` ;
- **niveau `kesh-api`, avoir** : une facture validée à commentaire, puis `credit_notes::create_credit_note(&pool,
  NewCreditNote { company_id, invoice_id, date }, user_id)` (le dépôt **reprend** le type de chaque ligne, 49-1a ; modèle
  `invoice_pdf_e2e.rs:882`, `credit_notes_repository.rs:1189`), puis `GET /api/v1/credit-notes/{id}/pdf` ;
- **SQL direct** en repli : `INSERT INTO invoice_lines (…, line_type, …) VALUES (…, 'comment', …)` à valeurs neutres sur une
  facture validée (forme que la contrainte `chk_invoice_lines_unpriced_neutral` accepte), si le dépôt refusait un
  assemblage à l'étape 0.

## Critères d'acceptation

- **AC1 — Une facture et un avoir de lignes chiffrées se rendent à l'identique de la 0.19.0 (niveau `kesh-qrbill`).** Neuf
  scénarios, construits en littéraux, rendus par `generate_qr_bill_pdf_with_date` (facture et rappel) et
  `generate_credit_note_pdf_with_date` (avoir), date fixée : (S1) facture, une ligne, QR-IBAN et référence QRR ; (S2) facture
  à cinq lignes, **deux taux** (8.10 et 2.60) et arrondi à 5 centimes ; (S3) facture à **huit lignes** (la limite d'une
  facture sans TVA) ; (S4) avoir à douze lignes, **multi-taux**, pleine page ; (S5) rappel (réglé, reste dû, frais) ;
  (S6) facture non assujettie, sans récapitulatif de TVA ; (S7) facture dont une description dépasse 45 caractères
  (réduction inchangée, D8) ; (S8) facture à `Reference::None` (NON : `kesh-qrbill` ne produit ni SCOR, `types.rs:95-100`, ni autre référence que QRR) au lieu de QRR et à adresses longues (la **section de
  paiement** entière est dans la comparaison : la coupe d'adresse, `pdf.rs:1283`) ; (S9) facture dont le **message de la QR**
  (« Facture F-… », `unstructured_message`, nourri par le numéro de facture) est assez long pour se couper sur plusieurs
  lignes au **palier P0** (`pdf.rs:1351` `PalierPaiement::P0 => couper_ligne(message, INFO_LARGEUR),`) : second appelant de
  production de `couper_ligne`, présent sur **toute facture numérotée**. **Tous** les scénarios portent le message que
  produit l'API (« Facture F-… »). Les deux chemins de `couper_ligne`, généralisée par D5, sont ainsi éprouvés.
  Pour chacun : le condensat des opérations de contenu, la liste de polices, la `MediaBox`, le nombre de pages et la liste lisible
  `(x, y, texte)` **égalent** le fichier de référence capturé **avant** le changement (D4). **Contrôles du graphe**
  (C = commit de capture, F = commit des fixtures ; la fiche est exclue de tous les `--stat` par un pathspec explicite ;
  `RENDU` = `crates/kesh-qrbill/src/{pdf,types,largeurs,texte,generator,validation,parser}.rs` et
  `crates/kesh-api/src/` ; les fichiers de `RENDU` lus pour `sources.sha256` sont `crates/kesh-qrbill/src/{pdf,types,largeurs,texte,generator,validation,parser}.rs`
  et `crates/kesh-api/src/routes/{invoice_pdf_service,credit_notes,issued_invoice_pdf,invoice_pdf}.rs`) :
  (1) `git diff cbdd70026 C -- RENDU` est **vide** ; (2) `git diff --stat cbdd70026 C -- . ':!_bmad-output/implementation-artifacts/49-1b-types-de-lignes-pdf.md'` (tout l'arbre : `Cargo.toml` racine, `.cargo/`, chaîne d'outils compris) nomme **exactement**
  `crates/kesh-qrbill/src/empreinte.rs`, `crates/kesh-qrbill/src/lib.rs`, `crates/kesh-qrbill/Cargo.toml`,
  `crates/kesh-api/Cargo.toml`, `crates/kesh-qrbill/tests/rendu_avant_apres.rs`,
  `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`, et `Cargo.lock` **seulement s'il change** (l'entrée de `kesh-qrbill` y
  liste déjà `sha2` ; seule l'auto-dev-dépendance pourrait le modifier : constaté à l'étape 0 et écrit au Dev Agent Record) ; (3) `git diff --stat C F` ne nomme **que des fichiers sous** les deux dossiers
  `tests/fixtures/avant-0.20.0/` (qrbill et api) ; (4) chaque `sources.sha256` égale `git show cbdd70026:<fichier> | sha256sum` pour
  chaque fichier listé ; (5) F est **ancêtre** du premier commit qui touche `RENDU`, cherché par
  `git log --reverse --format=%H cbdd70026..HEAD -- RENDU` (et non par une recherche de `LinePdfKind`) :
  `git merge-base --is-ancestor F <ce premier hash>` rend 0 (C et F ne touchent pas `RENDU`, contrôles (1) et (3) : le
  premier hash est donc postérieur à F ; une sortie vide, avant tout code, est conforme) ; (6) `git diff F..HEAD -- crates/kesh-qrbill/tests/fixtures/avant-0.20.0/
  crates/kesh-api/tests/fixtures/avant-0.20.0/ crates/kesh-qrbill/src/empreinte.rs` est **vide** (fixtures et comparateur
  gelés) ; (7) les hashs de C et de F sont consignés au **seul** Dev Agent Record.
- **AC2 — Même épreuve par la base et le HTTP (niveau `kesh-api`).** Quatre pièces faites **seulement de lignes `priced`**,
  créées par le dépôt, validées, rendues par `GET /api/v1/invoices/{id}/pdf` ou `…/credit-notes/{id}/pdf` : (a) facture à
  une ligne et un taux ; (b) facture **multi-taux** (7.70 et 2.60) ; (c) facture à arrondi non nul ; (d) **avoir** d'une facture **distincte** (b'), de même contenu que (b) : créer un avoir annule la facture d'origine
  (`crates/kesh-db/src/repositories/credit_notes.rs:843` `UPDATE invoices SET status = 'cancelled'`) et son regel serait refusé (`InvoiceCancelled`). Le contenu rendu (même condensat, même liste de polices) égale la référence capturée avant le changement ;
  pour (a) et (b), le **regel** (`POST …/pdf/refreeze` après suppression du fichier archivé, D7 c) rend le même contenu. La
  **QR-facture** (section de paiement, IBAN, montant, référence) fait partie du contenu comparé.
- **AC3 — Un commentaire s'imprime sur toute la largeur, sans colonnes.** Pour une ligne `kind = Comment`, le texte est
  dessiné à `x = 20.0` mm ; **aucun texte** n'est dessiné à `x = 110`, `130`, `160` ni `180` mm à la hauteur de cette ligne ;
  aucun « 0.00 » ni « % » n'y figure. Vrai pour la facture, l'avoir et le rappel (le gabarit est le même).
- **AC4 — Rien ne se perd, rien ne dépasse.** Un commentaire de 600 caractères mêlant mots courts et un mot de 90 `W` se
  coupe en lignes selon le **contrat réel de `couper_ligne`** (`pdf.rs:1089-1091` : « hors un mot plus large que la
  colonne, que seule leur concaténation redonne »), soit trois invariants : (1) le texte **sans blancs** — **après `texte_imprime`**, qui remplace U+00A0 et retire U+00AD, non le texte brut — égale la
  concaténation des lignes **sans blancs** ; (2) chaque frontière entre deux mots courts tombe **à l'intérieur** d'une ligne
  ou **en fin** de ligne (un mot court n'est jamais coupé) ; (3) **chaque** ligne mesure au plus 170 mm
  (`largeur_mm(ligne, 9.0, false)`). Le « rien ne se perd » vaut **dans WinAnsi et les 125 replis** de `texte_imprime` : un
  caractère sans entrée (CJK, émoji) compte zéro et `lopdf` le retire à l'écriture (`largeurs.rs:499-505`), comme il disparaît
  aujourd'hui d'une description. Un `\n` fait un saut de ligne, une ligne vide s'imprime vide ; une description
  blanche ne dessine rien et ne consomme aucune ligne. Le texte passe par `texte_imprime` (U+00A0, lettres hors WinAnsi).
- **AC5 — Un commentaire ne change aucun montant (niveau `kesh-api`) et décale la mise en page de `n × 5` mm (niveau
  `kesh-qrbill`).** (a) *`kesh-api`, là où se font les sommes* (`invoice_pdf_service.rs:408-409`, `credit_notes.rs:320-333`) :
  `[chiffrée A, commentaire, chiffrée B]` et `[A, B]`, créées au dépôt, donnent la **même** ligne de sous-total, la **même**
  ventilation de TVA (pas de ligne « TVA 0.0% »), le même arrondi, le même TTC et le même montant de QR, **pour la
  facture et pour l'avoir** (D9 est prouvé **ici**, non dans `kesh-qrbill`, où le test fournirait lui-même les deux côtés
  de l'égalité) ; (b) *`kesh-qrbill`* : à sommes égales, **seules** les ordonnées des lignes qui suivent le commentaire, du
  récapitulatif et du total changent, **d'exactement** `n × 5.0` mm (`n` = nombre de lignes de texte du commentaire).
- **AC6 — Un titre de section ne s'imprime jamais en `0.00`.** Une ligne `kind = Heading` se dessine **exactement comme** un
  commentaire de même texte (D1) ; le test l'écrit tel quel et **nomme la 49-3** comme la story qui le remplace.
- **AC7 — La capacité reste gardée, sans chevauchement du QR.** Une facture dont le commentaire ne tient pas rend
  `Err(QrBillError::TooManyLines(n))` avec `n = inv.lines.len()` ; la même pièce **tient** sur un avoir de la même taille
  (pleine page) ; au seuil exact (huit lignes de texte sur une facture sans récapitulatif, neuf refuse), aucun texte n'est
  dessiné sous le plancher (`SEP_Y`, 105 mm, pour une facture). Au niveau HTTP, un commentaire de mille caractères sur une
  facture validée rend `400 INVOICE_TOO_MANY_LINES_FOR_PDF` (la limite et sa conséquence sont **dites** à la fiche, D6 ;
  pas de coupe silencieuse).
- **AC8 — La garde de l'API, le gel et le chemin chiffré ne bougent pas.** `git diff cbdd70026..HEAD -- crates/kesh-db
  crates/kesh-core crates/kesh-api/src/routes/invoices.rs crates/kesh-api/src/routes/issued_invoice_pdf.rs
  crates/kesh-api/src/routes/invoice_pdf.rs` est **vide** ; `LineType::open_to_api()` rend `true` pour `priced` seulement ;
  le test T9 de la 49-1a est vert. Le `match` `LineType → LinePdfKind` est exhaustif (aucun `_ =>`), contrôlé par un test qui
  énumère les trois types.
- **AC9 — Un PDF archivé se sert tel quel ; une facture jamais téléchargée et un regel se rendent par le code neuf.** (a) Une
  facture validée dont le PDF est figé avec un contenu **autre** que celui que le code rendrait est servie **octet pour
  octet** (SHA-256 égal) ; (b) une facture validée jamais téléchargée se rend et se fige ; (c) le regel d'un fichier absent
  rend de nouveau ; (b) et (c) sont comparés à la référence pour une pièce chiffrée (AC2) et contiennent le commentaire
  pour une pièce à commentaire ; les tests existants de `invoice_frozen_pdf_e2e.rs` sont verts **sans modification** de leurs
  assertions.
- **AC10 — Aucune ressource neuve.** La liste des polices du dictionnaire de ressources d'un PDF avec commentaire est
  `{Helvetica, Helvetica-Bold}`, **identique** à celle d'un PDF sans ; aucune clé i18n n'est ajoutée
  (`git diff cbdd70026..HEAD -- crates/kesh-i18n` vide) ; `QrBillI18n` / `I18N_KEYS` inchangés.
- **AC11 — L'API publique de `kesh-qrbill` se documente.** `LinePdfKind` et `InvoiceLinePdf::kind` portent leur `///` ;
  `draw_invoice_section` dit (doc-comment) que le commentaire se coupe et la description chiffrée se tronque, et pourquoi
  (D8) ; le `README.md` de la crate, qui n'a **aucune** section décrivant `InvoicePdfData` (seulement le `use` de la ligne 14),
  gagne **une phrase** après l'exemple d'usage (Beat retouche deux lignes de ce fichier : à relire à la fusion).
- **AC12 — Recette visuelle** (`CLAUDE.md` § *Recette visuelle*, un document visible change) : captures PNG (`mutool draw -r 150`)
  de **chaque état touché**, prises par l'exemple de capture étendu (`capture_qr_facture`) sur des **données synthétiques** :
  facture à commentaire entre deux lignes ; facture à commentaire multi-lignes ; commentaire long coupé ; avoir à
  commentaire ; facture à `Heading` (rendu D1) ; et, en **témoin**, la facture S2 de l'AC1. Elles vivent dans
  `kesh-gate-logs/e491b/captures/`, jointes au Dev Agent Record et au prompt de revue ; une capture se **lit** (le
  commentaire est lisible, sans colonnes, aligné à gauche).
- **AC13 — Coordination.** La story ne touche ni `crates/kesh-qrbill/src/largeurs.rs`, ni `kesh-report/`, ni les
  catalogues i18n, ni `sprint-status.yaml`, ni `CHANGELOG.md`. Les changements de `lib.rs` sont : (i) dans C, le module du
  comparateur et son ré-export sous `#[cfg(feature = "test-support")]` ; (ii) dans la story, `LinePdfKind` ajouté à la ligne
  de ré-export de `InvoiceLinePdf` (`lib.rs:27`). Hors `lib.rs`, C porte les `Cargo.toml` et `Cargo.lock` : `[features]
  test-support`, `sha2` optionnel sous cette feature et l'auto-dev-dépendance dans `kesh-qrbill/Cargo.toml`, la
  dev-dépendance `kesh-qrbill = { …, features = ["test-support"] }` dans `kesh-api/Cargo.toml`, `Cargo.lock` régénéré.
- **AC14 — Gates.** § *Tâches*, dernière tâche ; le Dev Agent Record n'affirme que ce qui a tourné.

## Tests à écrire, et la preuve du rouge

Journaux de rouge dans `kesh-gate-logs/e491b/` (`rouge-T<n>-<m>.log`), commande et sortie.

| # | Test | Ce qu'il prouve | Preuve du rouge |
|---|---|---|---|
| T0 | `crates/kesh-qrbill/tests/rendu_avant_apres.rs::capturer_references_avant_0_20_0` (`#[ignore]` ; `KESH_REFERENCES=ecrire` échoue si une référence existe, `=comparer` vérifie la stabilité) | écrit les références de l'AC1 et `sources.sha256` dans `tests/fixtures/avant-0.20.0/` **dans le worktree détaché à C** | `ecrire` puis `comparer` dans le même worktree propre : deux rendus identiques (journal `t0-stabilite.log`, avec la sortie de `git status --porcelain`, vide) ; sinon la cause se nomme (D4) |
| T0-bis | `crates/kesh-api/tests/pdf_avant_apres_e2e.rs::capturer_references_http_avant_0_20_0` (`#[ignore]`, mêmes deux modes de `KESH_REFERENCES`, V1-1) | crée par le dépôt les pièces de l'AC2, les rend par le service et le HTTP (facture jamais téléchargée, regel) et écrit leurs références dans `kesh-api/tests/fixtures/avant-0.20.0/` ; **même worktree, même montage que T3** (un seul helper de montage : `sqlx::test`, graines, dates d'émission et d'échéance posées) | `ecrire` puis `comparer` : sorties identiques, sinon la cause se nomme (la date de création va dans `Info`, hors empreinte : à confirmer) |
| T1-bis | `rendu_avant_apres.rs::l_empreinte_change_des_qu_un_rendu_change` (V2-2, V3-4, V5-1) | **l'empreinte change dès qu'un rendu change** : pour chaque paire de PDF fabriqués par le même code `printpdf` — (1) décalage (un texte à `x` et à `x + 0.01` mm) ; (2) texte (un caractère changé) ; (3) une opération changée d'un cheveu ; (4) une police intégrée de plus ; (5) `MediaBox` différente ; (6) une page de plus — l'assertion est seulement `empreinte(a) != empreinte(b)` ; une paire **identique** donne `empreinte(a) == empreinte(b)` (stabilité). Le test n'affirme **pas** quelle composante diffère (condensat, textes, polices, pages se recouvrent). Dans C, comparateur gelé | **une seule mutation du comparateur** : une empreinte qui ne dépend plus du contenu (constante, ou condensat vide) → **ROUGE** pour **toutes** les paires `!=` (journal `rouge-T1bis.log`) ; la mutation 4 de T1 éprouve en outre la lecture des polices |
| T1 | `rendu_avant_apres.rs::neuf_pieces_chiffrees_se_rendent_comme_en_0_19_0` (AC1) | neuf scénarios (S1 à S9) égaux à leur référence, condensat + polices + pages + liste lisible | quatre mutations numérotées, chacune **ajoutée au code de rendu** : (1) `col_qty` décalé de `0.1` mm ; (2) `ty -= 5.0` → `ty -= 5.01` à la fin de la boucle ; (3) une ligne chiffrée passée **par le chemin du commentaire** (`kind` forcé) ; (4) `BuiltinFont::HelveticaOblique` ajoutée au document, jamais utilisée (**ne bouge aucune opération de contenu** : seule la liste de polices la tue, ce qui éprouve la comparaison des ressources) → **ROUGE** ×4 (`rouge-T1-1.log` à `-4.log`) |
| T2 | `rendu_avant_apres.rs::les_deux_coupes_de_la_qr_sont_inchangees` (S8 : adresse ; S9 : message au palier P0) | la section de paiement égale la référence pour l'adresse **et** pour le message | mutation : `couper_ligne` appelle `couper_ligne_a` avec `9.0` au lieu de `VALEUR_TAILLE` → **ROUGE** pour **chacun** des deux chemins (une mutation, un test) (le test compare et **rapporte par scénario** : S8 puis S9 se lisent rouges séparément dans un même journal `rouge-T2.log`) |
| T3 | `kesh-api/tests/pdf_avant_apres_e2e.rs::quatre_pieces_chiffrees_se_rendent_comme_en_0_19_0` (AC2) | (a) à (d) par la base et le HTTP, le regel de (a) et (b) ; références capturées par **T0-bis** | mutations : `ligne_pdf` met `Comment` pour toute ligne → **ROUGE** ; `ligne_pdf` perd `line_total` (`Decimal::ZERO`) → **ROUGE** ; chacune tuée par la comparaison, non par un `assert!(status == 200)` (`rouge-T3-1.log`, `-2.log`) |
| T4 | `pdf.rs::tests::un_commentaire_se_dessine_pleine_largeur_sans_colonnes` (AC3) | `x = 20.0`, rien aux quatre colonnes à la hauteur de la ligne, pas de « 0.00 », sur facture, avoir et rappel | mutation : le chemin du commentaire dessine aussi `format_ch(line.quantity, 2)` → **ROUGE** (`rouge-T4.log`) |
| T5 | `pdf.rs::tests::un_commentaire_long_se_coupe_sans_rien_perdre` (AC4) | les trois invariants de l'AC4 (texte sans blancs égal à la concaténation, frontières de mots courts, ≤ 170 mm), `\n`, ligne vide, `\r` (CRLF), tabulation, `\n` et `\n\n` **finaux** (ignorés), description blanche, mot de 90 `W` | mutations : (1) le chemin du commentaire reprend `truncate_display(…, 45)` → **ROUGE** (perte) ; (2) largeur 170 → 190 → **ROUGE** (dépassement) (`rouge-T5-1.log`, `-2.log`) |
| T6 | `pdf.rs::tests::un_commentaire_decale_de_n_fois_5_mm` (AC5 b) | [A, C, B] contre [A, B] à sommes fournies égales : ordonnées décalées de `n × 5.0`, abscisses et textes inchangés | mutation : le commentaire ne consomme pas son pas (`ty` non décrémenté) → **ROUGE** (`rouge-T6.log`) ; ce test **ne prouve pas** les sommes (AC5 a, T9, T10) |
| T7 | `pdf.rs::tests::un_titre_de_section_se_rend_comme_un_commentaire_en_49_1b` (AC6) | `Heading` = `Comment` au texte près ; nomme la 49-3 | mutation : `Heading` retombe sur le chemin chiffré → **ROUGE** (`rouge-T7.log`) |
| T8 | `pdf.rs::tests::la_capacite_se_garde_par_ligne_de_texte` (AC7) | refus `TooManyLines(n)` au seuil, succès sur l'avoir, aucun texte sous le plancher | mutation : la garde ne s'applique qu'à la **première** ligne de texte d'un commentaire → **ROUGE** (texte sous `SEP_Y`, ou `Ok` à tort) (`rouge-T8.log`) |
| T9 | `kesh-api/tests/invoice_pdf_e2e.rs::pdf_d_une_facture_a_commentaire_l_imprime` (AC3, AC5 a, AC7) | facture `[A, commentaire, B]` créée **au dépôt**, validée, `GET /pdf` 200 ; le PDF contient le commentaire (recherche du texte en hexadécimal, comme `pdf_contains`, `invoice_frozen_pdf_e2e.rs:379`, ou relecture `lopdf` à confirmer à l'étape 0), pas de « 0.00 » à sa ligne ; **AC5 a** : `[A, B]` créée au dépôt donne les mêmes textes de récapitulatif, le même TTC et le même montant de QR (relus dans le PDF, par le comparateur exposé) ; un commentaire de mille caractères rend `400 INVOICE_TOO_MANY_LINES_FOR_PDF` | mutations : `ligne_pdf` met `Priced` à tout (le commentaire s'imprime `0.00`) → **ROUGE** ; **au site réel des sommes, mutations qui changent une valeur** : (2) le wrapper `vat_lines_pdf` (`invoice_pdf_service.rs:37`) ajoute une ligne `InvoiceVatLinePdf { rate_percent: 0, … }` quand une paire a un taux `<= 0` (ligne « TVA 0.0% » au récapitulatif ; **le filtre réel est `kesh-core/src/accounting/vat.rs:132`**, que l'AC8 veut inchangé : on ne le mute pas, une mutation y ferait échouer `validate_invoice` avant l'assertion) ; (3) `subtotal_ht` (`:408`) devient `lines.iter().map(\|l\| l.line_total).sum::<Decimal>() + Decimal::from(nombre de lignes non chiffrées)` → **ROUGE** ×3 (`rouge-T9-1.log` à `-3.log`). **D9 est protégé par construction** (la contrainte `chk_*_unpriced_neutral` interdit une ligne non chiffrée à `line_total ≠ 0`) : (3) vérifie seulement que la comparaison `[A,C,B]` = `[A,B]` lit bien la somme ; compter « sans changement » une ligne à 0 serait un mutant équivalent, écarté |
| T10 | `kesh-api/tests/invoice_pdf_e2e.rs::pdf_d_un_avoir_reprend_le_commentaire` (AC3, AC5) | avoir d'une facture à commentaire : `GET …/credit-notes/{id}/pdf` imprime le commentaire pleine largeur ; **AC5 a pour l'avoir** : mêmes sommes, TTC et récapitulatif que l'avoir de `[A, B]` (facture distincte) | mutations : `credit_notes.rs` revient à l'ancienne fermeture avec `kind: Priced` forcé (elle ne compile plus sans `kind`) → **ROUGE** ; (2) `subtotal_ht` de l'avoir (`credit_notes.rs:330`) ajoute `Decimal::from(nombre de lignes non chiffrées)` (même raisonnement que T9 (3) : mutation qui change la valeur) → **ROUGE** (`rouge-T10-1.log`, `-2.log`) |
| T11 | `invoice_pdf_service.rs::tests::le_match_de_type_est_exhaustif_et_neutre_pour_priced` (AC8) | `ligne_pdf` pour `Priced` : tous les champs recopiés ; les trois `LineType` énumérés ; `Priced` → `LinePdfKind::Priced` | mutation : `Comment` → `Priced` dans le `match` → **ROUGE** (`rouge-T11.log`) |
| T12 | `kesh-api/tests/invoice_frozen_pdf_e2e.rs::un_pdf_fige_est_servi_tel_quel_meme_si_le_rendu_aurait_change` (AC9 a) | un fichier archivé de contenu **autre** (octets d'un PDF quelconque + SHA-256 posé par SQL de forme 0.19.0, quatre colonnes `pdf_*`) est servi octet pour octet ; le regel d'un fichier absent rend le contenu neuf | mutation : `get_or_freeze` rend toujours (ignore `is_frozen`) → **ROUGE** (`rouge-T12.log`) |
| T13 | épreuve, non test neuf : `invoice_pdf_e2e`, `invoice_frozen_pdf_e2e`, `invoice_send_email_e2e`, `golden_test`, les tests de `pdf.rs` existants | **verts sans modification de leurs assertions** (les littéraux gagnent `kind: LinePdfKind::Priced`, rien d'autre) | — |

Chaque mutation s'**ajoute au code**, le test s'exécute, le journal est conservé, puis la mutation se retire **et le fichier se
touche** (`touch`) avant la reconstruction : cargo garde sinon le binaire muté.

## Tâches

- [ ] **Étape 0 — lire et capturer, dans cet ordre, avant tout changement du rendu.** (1) Rejouer `git diff --stat
  773df80da..HEAD -- crates/kesh-qrbill crates/kesh-api/src/routes/issued_invoice_pdf.rs` (**vide**) ; (2) **commit C**, posé au-dessus
  de la tête de la branche (les diffs se mesurent contre `cbdd70026`) : le comparateur `crates/kesh-qrbill/src/empreinte.rs` et ses deux lignes de `lib.rs`, les deux
  `Cargo.toml`, `Cargo.lock`, T0, T0-bis, T1-bis — **sans fixtures** ni aucune ligne de `RENDU` (AC1, contrôle (1)) ; (3)
  `git worktree add --detach /home/gcorbaz/devel/kesh-gate-logs/e491b/capture/ C` (zone permise aux agents : jamais hors de son worktree et de `kesh-gate-logs/`), puis **dans ce worktree** : `git status --porcelain` **vide**
  (sortie copiée à `t0-stabilite.log`), exporter `DATABASE_URL` vers le MariaDB de dev (compte `kesh`, droit `kesh%`, `docs/testing.md` § *Base de dev jetable* ; le
  worktree détaché n'a pas de `.env`, et `docker compose` ne se lance que du dépôt principal) : les tests de capture sont
  des `#[sqlx::test]`, dont sqlx crée et détruit la base temporaire (squash `test-schema/` de C) — **aucune base nommée à
  créer, migrer ni supprimer**, puis lancer
  `KESH_REFERENCES=ecrire cargo test -p kesh-qrbill --test rendu_avant_apres -- --ignored capturer` et
  `KESH_REFERENCES=ecrire cargo test -p kesh-api --test pdf_avant_apres_e2e -- --ignored capturer` (sous `mem-guard`), puis
  les mêmes avec `KESH_REFERENCES=comparer` (stabilité, D4) ; **le mode par défaut, variable absente, est `comparer`** (jamais
  d'écriture involontaire) ; (4) **séquence exacte** : copier les fixtures du worktree de capture vers l'arbre de
  développement, les comparer (`diff -r` du worktree et de l'arbre : identiques), **supprimer les fixtures du worktree de
  capture**, constater `git status --porcelain` **vide**, puis `git worktree remove` (sans `--force`) du worktree de capture, retiré après usage ; **commiter F**
  (`test(49-1b) : références du rendu de la 0.19.0 (refs #1242)`) au-dessus de C ; (5) **consigner les hashs de C et de F au Dev Agent
  Record, seul** ; contrôler le graphe (AC1 (1) à (7)) ; (6) **seulement alors**, écrire le code de la story
  (`LinePdfKind`, `couper_ligne_a`, `ligne_pdf`…). Vérifier que `couper_ligne` a **deux** appelants de production
  (`pdf.rs:1283`, adresse, et `pdf.rs:1351`, message au palier P0) : `git grep -n "couper_ligne(" -- crates`. Le worktree de
  capture compile la crate à neuf ; pour ne pas refaire 20 à 50 Go de cible, il peut partager `CARGO_TARGET_DIR` avec
  l'arbre de développement (les sources sont celles de C, cargo reconstruit ce qui diffère).
- [ ] **`kesh-qrbill`** : `LinePdfKind` et `InvoiceLinePdf::kind` (`types.rs`, `lib.rs:27`) ; `couper_ligne_a` (D5) ;
  le chemin du commentaire dans `draw_invoice_section` (D1, D3, D5, D6) ; mise à jour des littéraux (`pdf.rs` ×6,
  `golden_test.rs`, `examples/capture_qr_facture.rs`) ; `README.md` de la crate ; **le rebranchement des helpers de `pdf.rs::tests` sur le comparateur** (sans le modifier). **Le comparateur, `[features]`, `sha2`
  optionnel, l'auto-dev-dépendance et `Cargo.lock` sont du commit C, non de la story** (étape 0).
- [ ] **`kesh-api`** : `ligne_pdf` et son `match` (D10) dans `invoice_pdf_service.rs` ; ses deux appelants (`:396`,
  `credit_notes.rs:310`) ; le littéral de `qr_decode.rs:885` ; **la dev-dépendance `kesh-qrbill` avec `features = ["test-support"]`** de
  `kesh-api/Cargo.toml` est du commit C. **Aucune autre ligne** (AC8, AC13).
- [ ] **Tests** T1 à T12 (T0, T0-bis et T1-bis sont du commit C), avec leurs rouges (journaux dans `kesh-gate-logs/e491b/`).
- [ ] **Captures** (AC12) : étendre l'exemple de capture, produire les PNG dans `kesh-gate-logs/e491b/captures/`, les lire.
- [ ] **Gates** (`e35-dev-commun.md` § *Quels gates, quand*, voie A : base et Rust). **Aucun module `kesh-db`** : en cours de
  boucle, gate ciblé — `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo nextest run -E 'package(kesh-qrbill) | binary(kesh_api) | binary(pdf_avant_apres_e2e) |
  binary(invoice_pdf_e2e) | binary(invoice_frozen_pdf_e2e) | binary(invoice_send_email_e2e) | binary(dunning_reminders_e2e) |
  binary(company_address_e2e)'` (`binary(kesh_api)` porte T11 et le test de `qr_decode.rs`) ; **à la clôture** (revue terminée) : base remise à
  neuf (CLAUDE.md § *Base de gate*), `scripts/gate-ci.sh` complet, **E2E Playwright complet une fois** (aucun contrôle de
  navigateur ne change, mais la règle du dépôt veut la suite avant le push), démarrage sur base persistante inchangé
  (aucune migration). **Au passage en `review`** (`e35-dev-commun.md`, ligne « Passage en `review` ») : `scripts/test-fast.sh --ci`
  puis `scripts/gate-ci.sh`. Tout sous `scripts/mem-guard.sh` et `flock` du verrou de la voie ; une commande lourde à la fois.
- [ ] **Contrôle des mots-clés** : `git log origin/epic/49-types-de-lignes-facture..HEAD --format=%B | /usr/bin/grep -niE
  '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` **vide**.

## Note de version

*(Texte pour le `CHANGELOG`, assemblé à l'intégration ; la story n'écrit pas `CHANGELOG.md`.)*

- **Factures et avoirs — rendu PDF des lignes de commentaire** : le PDF d'une facture et d'un avoir sait imprimer une ligne de
  commentaire sur toute la largeur de la page, sans colonnes de quantité, de prix, de TVA ni de total, coupée en plusieurs
  lignes quand elle est longue, sans rien tronquer. Le PDF d'une facture ou d'un avoir **sans** ligne de commentaire est
  identique à celui de la 0.19.0, vérifié par un test sur des pièces types ; un PDF déjà archivé n'est jamais réécrit ; la montée réelle d'une base peuplée par la 0.19.0 est éprouvée par la recette de montée (49-1d). La
  saisie d'une ligne de commentaire vient avec l'écran de la même fonctionnalité (`refs #1242`).

## Dev Notes

- **Hors périmètre, avec la story qui le prend** : saisie du type, ouverture de `comment` dans l'API, affichage à l'écran,
  `verify_vat_rates_against_db`, manuel utilisateur → **49-1c** ; mise en évidence et ouverture de `heading` → **49-3** ;
  réordonnancement → **49-2** ; recette de montée et section « Passer à la 0.20.0 » → **49-1d** ; sous-totaux → **#1330**.
- **À écrire en tâche à la fiche de la 49-1c (consigne de cette fiche, D6)** : borner ou signaler la **longueur d'un
  commentaire** à la saisie (la facture validée ne se corrige plus : un commentaire qui ne tient pas sur le PDF la rend
  impossible à imprimer), par exemple en refusant à l'enregistrement du brouillon une facture dont les lignes de texte
  dépassent la capacité de la page, avec la phrase de #1110 corrigée.
- **Issues à ouvrir** (demandées à l'orchestrateur) : (1) *la description d'une ligne chiffrée est tronquée à 44 caractères suivis de « … » sur le PDF de facture et
  d'avoir, sans avertissement à la saisie* — `pdf.rs:637` ; hors périmètre (D8) ; **issue déjà ouverte par l'orchestrateur
  (#1363, dont il corrige le corps)** ; la priorité se rediscute sur cette base (la coupe est visible sur la pièce) ; (2) *une facture validée dont le PDF
  ne tient pas sur la page (trop de lignes, ou un commentaire long) ne peut plus être imprimée et ne se corrige plus* — le
  refus tombe après la validation ; recoupe #1110 (la phrase) sans en couvrir la cause ; **P2** ; à lier à la 49-1c.
- **Fichiers communs avec la PR #1327 de Beat** (mise en page des PDF de **rapports**) : `git fetch origin pull/1327/head`
  puis `git diff $(git merge-base FETCH_HEAD origin/main)..FETCH_HEAD --stat` (lecture seule) donne, côté `kesh-qrbill`,
  `README.md`, `src/largeurs.rs` (doc-comment, `pub(crate)` → `pub` pour `largeur_mm`) et `src/lib.rs` (doc de tête et
  `pub use largeurs::largeur_mm;`). **Elle ne touche ni `pdf.rs` ni `types.rs`**, les deux fichiers de rendu de la 49-1b.
  **Seul fichier commun : `lib.rs`** (la ligne de ré-export de `types`, `lib.rs:27`, contre les lignes de tête et le `pub use`
  de Beat : hunks distincts, fusion sans conflit probable) et le `README.md` de la crate (la 49-1b ajoute une phrase, Beat
  en retouche deux lignes : à relire). **Ordre de fusion** : aucun n'est imposé par le code (la 49-1b n'emploie `largeur_mm`
  que **dans le crate**, où elle est `pub(crate)` avant comme après) ; **#1327 d'abord** (décision de Guy du 2026-11-18 : elle
  se fusionne avant la 48-11, après la CI de la copie #1329) ; la 49-1b, **second arrivé**, reprend `main` dans sa
  branche (CLAUDE.md § *Fusions et fichiers communs*), relit `lib.rs` et le `README.md`, rejoue les gates et l'E2E. Si la
  49-1b est close avant : elle **ne se bloque pas** ; c'est #1327, ou la branche de l'épique, qui reprend.
- **Écarts déclarés** : (a) le fichier d'épique range `issued_invoice_pdf.rs` parmi les fichiers de la 49-1b ; la fiche **n'y touche pas** (D7, AC8), écart voulu ; (b) la recette visuelle du `CLAUDE.md` veut des captures « sur une base de démonstration » ; AC12 les prend par l'exemple de capture sur des **données synthétiques**, inévitable tant que la garde de l'API est fermée : **dérogation**.
- **Splitting** : **deux modules** touchés (`kesh-qrbill`, `kesh-api`), plus les tests et fixtures : **sous le plafond de
  cinq**, aucun découpage à proposer. Aucune dérogation.
- **Manuels** : `git grep -n -i "PDF" -- docs/manual/fr/user-manual.tex | grep -i -E "ligne|facture"` ne nomme que la description
  générale du PDF (`user-manual.tex:1486`, `:1577`, `:1586` : « figé au premier téléchargement », « identique d'un
  téléchargement à l'autre ») — **vraie avant et après** cette story ; `git grep -n -i "ligne de commentaire" -- docs` est
  **vide**. **Aucun manuel ne change** : la saisie et le rendu d'un commentaire se documentent à la **49-1c**, qui les livre à
  l'utilisateur. Manuels DE, EN, IT : n'existent pas (`docs/manual/de/README.md:3`).
- **Ce qui doit rester vrai** : (1) une facture chiffrée-seulement se rend à l'identique de la 0.19.0 (AC1, AC2) ;
  (2) le PDF archivé n'est jamais réécrit par ce code (AC9) ; (3) la garde de l'API reste fermée (AC8) : cette story **ne
  rend aucun commentaire créable** ; (4) la 49-1c ne retire la garde qu'après la fusion de cette story (première tâche de la
  49-1c : vérifier que la branche de l'épique porte `LinePdfKind` et le chemin du commentaire).
- **Pièges** : (a) **ne pas ajouter de police** au document (D3) — la comparaison des ressources le voit, la comparaison des
  seules opérations de contenu ne le verrait pas ; (b) `couper_ligne` a **deux** appelants de production, l'**adresse** de la QR (`pdf.rs:1283`) et le **message** au palier P0
  (`pdf.rs:1351`, présent sur toute facture numérotée) : la généraliser par une taille sans changer son résultat à 8 pt (T2) ; (c) un `Heading` en base **existe peut-être déjà** par import ou SQL : le
  rendu D1 le protège ; (d) `truncate_display` (44 caractères et « … ») est **voulu pour une ligne chiffrée** tant que D8 n'est pas
  traité : ne pas « harmoniser » ; (e) une description du dépôt peut contenir `\n`, `\r`, tabulation, U+00A0 : D5.
- **Aucune donnée réelle** de Guy dans la fiche, les fixtures, les tests, les captures ou les journaux : tout est
  **synthétique** (noms, adresses et IBAN de la directive SIX ou de la fixture de `pdf.rs`). `~/kesh/` est en lecture seule,
  jamais lu ni écrit.
- **Références** : `epic-49-types-de-lignes-facture.md` (§ *Contraintes de production*, § *49-1*), `release-0.20.0.md`,
  `kesh-gate-logs/e491/decoupage-49-1.md` (§ 2, § 3), fiche `49-1a-types-de-lignes-migration-api.md` (D9, AC14, Dev Notes
  « Test avant / après du PDF »), #1242, #1110, #1330, #1327, `kesh-gate-logs/prompts/e0200-commun.md`.

## Dev Agent Record

*(vide — à remplir au développement)*

## Change Log

| Date | Entrée |
|---|---|
| 2026-11-19 | **Création de la fiche** (Sonnet 5.5, `bmad-create-story`). Code lu à `cbdd70026`. Dix décisions (D1 à D10) ; quatorze AC ; treize tests. Points arrêtés : le titre de section est rendu comme un commentaire en 49-1b, sa mise en évidence reste à la 49-3 (D1) ; avant / après par le **contenu rendu** (condensat des opérations, polices, pages, liste lisible), référence capturée **avant** tout changement sur `cbdd70026` (rendu prouvé égal à celui de `773df80da` par un `git diff` vide), non par les octets du fichier (le `/ID` change) (D4) ; un commentaire se coupe, ne se tronque pas, et **la réduction à 44 caractères et « … » d'une description chiffrée reste hors périmètre (#1363)** (D5, D8) ; aucune migration, aucune clé i18n, aucune police neuve. Deux issues à ouvrir (Dev Notes). Validation non lancée. Statut : `ready-for-dev`. |
| 2026-11-19 | **Validation P1** (Opus 5.5, `kesh-gate-logs/e491b/validation-p1.md` : 0 CRITICAL, 1 HIGH, 5 MEDIUM, 7 LOW). **Remédiation** (décisions de l'orchestrateur) : V1-1 (HIGH) T0-bis côté `kesh-api`, capture avant toute source, ordre de l'étape 0 écrit ; V1-2 un seul comparateur exposé (`test-support`) ; V1-3 `couper_ligne` a deux appelants de production (`pdf.rs:1351`, message P0), scénario S9, T2 sur les deux chemins ; V1-4 preuve de D9 au niveau `kesh-api` (facture et avoir, mutations aux sites réels), T6 réduit à la mise en page ; V1-5 AC4/T5 selon le contrat réel de `couper_ligne` ; V1-6 « 44 caractères suivis de « … » » (#1363 corrigée par l'orchestrateur) ; V1-7 formulation de l'octet, `MediaBox` et dictionnaire des polices à l'empreinte ; V1-8 avoir sur une facture distincte ; V1-9 limite dite et note de version nuancée ; V1-10 filtre de gate et passage en review ; V1-11 deux écarts déclarés ; V1-12 sortie d'inventaire et README ; V1-13 WinAnsi et `CHECK` exacts. Modèles : Sonnet (rédaction), Opus (P1). Une passe 2 reste à lancer (modèle différent). |
| 2026-11-19 | **Validation P2** (Sonnet 5.5, `kesh-gate-logs/e491b/validation-p2.md` : 0 CRITICAL, 0 HIGH, 4 MEDIUM, 5 LOW). **Remédiation** : V2-1 contenu exact du commit de capture (comparateur, `Cargo.toml`/`Cargo.lock`, trois tests, fixtures, aucune ligne de rendu), hash consigné, ancêtre du premier commit de rendu (`merge-base --is-ancestor`), tests de capture qui refusent d'écraser ; V2-2 comparateur gelé, helpers de `pdf.rs::tests` rebranchés sans le modifier, test T1-bis qui le prouve rouge ; V2-3 mutations de T9/T10 qui changent une valeur, D9 dit protégé par construction ; V2-4 AC13 et tâches nomment `lib.rs`, `[features]`, `sha2` optionnel, dev-dépendance, `Cargo.lock` ; V2-5 pas de `doc(hidden)` en production, contrôle `cargo tree` ; V2-6 chemin `kesh-db` de la citation 843 ; V2-7 site réel du filtre de TVA (`kesh-core`), mutation du wrapper ; V2-8 `\r`, `\n` final, tabulation ; V2-9 neuf littéraux restants, montage fixe de T0-bis et T3. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2). Une passe 3 reste à lancer (modèle différent). |
| 2026-11-19 | **Validation P3 ciblée** (Opus 5.5, `kesh-gate-logs/e491b/validation-p3-ciblee.md` : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 5 LOW). **Signal « recycle »** : les défauts reviennent dans le **mécanisme de capture** (V3-1, V3-2, V3-3, V3-6 nés de la remédiation P2 ; V3-4 aussi), déclaré au Project Lead par l'orchestrateur. **Simplification retenue** plutôt qu'un nouvel empilement : la capture se fait **dans un worktree propre et détaché** à un commit C (comparateur, `Cargo.toml` ×2, `Cargo.lock`, tests de capture, sans fixtures), puis un commit F des seules fixtures, puis le code de rendu ; les contrôles de l'AC1 sont réécrits sur ce graphe (diff de `RENDU` vide, `--stat` explicite, F ancêtre du premier commit touchant `RENDU`, comparateur et fixtures gelés, `sources.sha256` comparés à `git show cbdd70026:<fichier>`) ; les hashs de C et F vont au seul Dev Agent Record ; `KESH_REFERENCES=ecrire|comparer` lève la contradiction « deux fois » / « échoue si existe ». **Remédiation** : V3-1 (HIGH) worktree propre + `sources.sha256` ; V3-2 hash hors des fixtures ; V3-3 contrôles de graphe corrigés (fiche exclue, premier commit de `RENDU` par `git log`, chemins nommés un par un) ; V3-4 T1-bis par paires de PDF fabriqués, une mutation du comparateur par composante ; V3-5 mécanisme de la feature nommé (auto-dev-dépendance, jamais `required-features`) et AC13 reformulé ; V3-6 variable `KESH_REFERENCES` ; V3-7 `RENDU` complet (`texte.rs`, `generator.rs`, `validation.rs`, `parser.rs`) ; V3-8 T5 couvre `\r`, tabulation, `\n` finaux (D5 : tous ignorés) ; V3-9 en-tête et « AC5 le prouve » alignés sur T9. **Table des renvois** (relue sur la fiche entière) : AC1 = S1 à S9, D4, T0, T0-bis, T1, T1-bis, T2, contrôles (1) à (7) ; AC2 = T0-bis, T3, D7 ; AC3 = T4, T9, T10 ; AC4 = T5, D5 ; AC5 = a T9 et T10 (D9), b T6 ; AC6 = T7, D1 ; AC7 = T8, T9, D6 ; AC8 = T11, D7, D10 ; AC9 = T12, D7 ; AC10 = D3, T1 mutation 4 ; AC11 = doc-comments, README ; AC12 = captures ; AC13 = lib.rs, Cargo, commit C ; AC14 = gates ; D2 = 9 littéraux, D10 ; D4 = comparateur unique et gelé, C, F ; D9 = AC5 a, mutations T9 et T10 ; S7 = D8, #1363 ; S8 = `couper_ligne` adresse (`pdf.rs:1283`) ; S9 = `couper_ligne` message P0 (`pdf.rs:1351`), T2. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3 ciblée). Une passe 4 reste à lancer, après une relecture complète. |
| 2026-11-19 | **Validation P4** (Sonnet 5.5, `kesh-gate-logs/e491b/validation-p4.md` : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 7 LOW). Le mécanisme de capture tient ; défauts locaux. **Remédiation** : V4-1 S8 en `Reference::None` (pas de SCOR dans `kesh-qrbill`, `types.rs:95-100`) ; V4-2 T1-bis à six paires (décalage, texte, opérations, polices, `MediaBox`, pages) et six mutations alignées une à une ; V4-3 séquence exacte de l'étape 0 (4) (copie, `diff -r`, suppression des fixtures du worktree, `git status --porcelain` vide, `git worktree remove` sans `--force`) ; V4-4 C posé au-dessus de la tête de la branche, diffs contre `cbdd70026` ; V4-5 contrôle (2) sur tout l'arbre, fiche exclue ; V4-6 commandes de capture, mode par défaut `comparer`, `DATABASE_URL` du worktree détaché ; V4-7 API publique du comparateur énumérée, appelants complétés, tâche du rebranchement des helpers ; V4-8 sonde de la construction de production (`cargo build --release -p kesh-api -v`, `strings`) en plus de `cargo tree`, unification de la CI dite ; V4-9 tabulation remplacée par une espace avant la coupe, invariant (1) relatif à `texte_imprime` ; V4-10 libellés (fichiers sous les dossiers, un `sources.sha256` par dossier, mutation de T10, T2 par scénario). **Table des renvois** (fiche relue en entier) : AC1 = S1 à S9, D4, T0, T0-bis, T1, T1-bis, T2, contrôles (1) à (7) ; AC2 = T0-bis, T3, D7 ; AC3 = T4, T9, T10 ; AC4 = T5, D5 ; AC5 = a T9 et T10 (D9), b T6 ; AC6 = T7, D1 ; AC7 = T8, T9, D6 ; AC8 = T11, D7, D10 ; AC9 = T12, D7 ; AC10 = D3, T1 mutation 4 ; AC11 = doc-comments, README ; AC12 = captures ; AC13 = lib.rs, Cargo, commit C ; AC14 = gates ; D2 = 9 littéraux, D10 ; D4 = comparateur unique et gelé (API énumérée), C, F ; D9 = AC5 a, mutations T9 et T10 ; S7 = D8, #1363 ; S8 = `couper_ligne` adresse (`pdf.rs:1283`), `Reference::None` ; S9 = `couper_ligne` message P0 (`pdf.rs:1351`), T2. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3 ciblée), Sonnet (P4). Une passe 5 reste à lancer (modèle différent). |
| 2026-11-19 | **Validation P5 ciblée** (Opus 5.5, `kesh-gate-logs/e491b/validation-p5-ciblee.md` : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW). **Signal « recycle »** : les deux MEDIUM sont nés de la remédiation P4 (V5-1 de la réécriture de T1-bis en six paires, V5-2 de la sonde `strings`), dans le même mécanisme de test du comparateur ; déclaré au Project Lead par l'orchestrateur. **Simplification retenue** plutôt qu'un raffinement de plus : T1-bis ne prouve plus « une composante à la fois » mais seulement que **l'empreinte change dès qu'un rendu change** (six paires `!=`, une paire identique `==`), avec **une seule mutation** du comparateur (empreinte indépendante du contenu) ; les six mutations par composante sont retirées. **Remédiation** : V5-1 T1-bis simplifié ; V5-2 sonde `strings` retirée, ligne `rustc` de `kesh_qrbill` recompilé conservée ; V5-3 `#[sqlx::test]` (base temporaire créée et détruite par sqlx, aucune base nommée) et worktree de capture sous `kesh-gate-logs/e491b/capture/` ; V5-4 `Cargo.lock` au contrôle (2) seulement s'il change ; V5-5 tabulation → espace dans le seul chemin du commentaire (`validation.rs:676`) ; V5-6 `MediaBox` énumérée à l'AC1. **Table des renvois** : AC1 = S1 à S9, D4, T0, T0-bis, T1, T1-bis, T2, contrôles (1) à (7) ; AC2 = T0-bis, T3, D7 ; AC3 = T4, T9, T10 ; AC4 = T5, D5 ; AC5 = a T9 et T10 (D9), b T6 ; AC6 = T7, D1 ; AC7 = T8, T9, D6 ; AC8 = T11, D7, D10 ; AC9 = T12, D7 ; AC10 = D3, T1 mutation 4 ; AC11 = doc-comments, README ; AC12 = captures ; AC13 = lib.rs, Cargo, commit C ; AC14 = gates ; D2 = 9 littéraux, D10 ; D4 = comparateur unique et gelé, C, F ; D9 = AC5 a, mutations T9 et T10 ; S7 = D8, #1363 ; S8 = adresse (`pdf.rs:1283`), `Reference::None` ; S9 = message P0 (`pdf.rs:1351`), T2. Modèles : Sonnet (rédaction), Opus (P1), Sonnet (P2), Opus (P3 ciblée), Sonnet (P4), Opus (P5 ciblée). Une passe 6 reste à lancer (modèle différent). |
