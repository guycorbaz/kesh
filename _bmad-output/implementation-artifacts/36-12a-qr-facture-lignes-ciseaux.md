# Story 36-12a : QR-facture — les deux lignes de séparation et leurs ciseaux (SIX § 3.7, #944)

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus 5.5) par le découpage de la fiche `36-12-qr-facture-separation.md` (`split`), décidé
par l'orchestrateur à la remédiation de la passe 1 de validation (V1-7, `kesh-gate-logs/e36-12/validation-p1.md`).
Worktree `kesh-36-12`, branche `story/36-12-qr-facture-separation`, partie de `origin/epic/36-0.18.0` (`fed34a9c7`).
Release **0.18.0**, phase 2. **Aucune migration** ; une seule fonction de production change (`draw_separator`). Code lu à
`fed34a9c7`. Voie 2 (bases `kesh_e3612`, `kesh_e2e_e3612` ; `kesh_ci_e3612` en voie 1). Registres :
`e35-choix-autonomes-e350.md` (C-e350-1) ; `e36-choix-autonomes-36-12.md` (C-36-12-2, C-36-12-8). La suite — grille,
*Payable by*, largeurs — est la story **36-12b**, qui part de celle-ci fusionnée.*

## Story

En tant que titulaire qui envoie ses QR-factures en PDF, je veux que le récépissé et la partie paiement soient
délimités par deux lignes portant chacune des ciseaux, comme l'exige la directive SIX pour un PDF, afin que le débiteur
qui imprime le bulletin sache où couper. Rien d'autre de la section de paiement ne bouge ; les PDF déjà figés ne
changent pas.

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
- Ce qui borde les lignes, et que les ciseaux ne doivent pas toucher : titre du récépissé 11 pt gras à x = 5, ligne de
  base 100 (`:901-909`), haut des capitales ≈ 102,8 ; titre de la partie paiement à x = 67, même hauteur (`:986-995`) ;
  QR de x = 67 à 113, de y = 47 à 93 (`:999-1000` `let qr_top = top - 7.0;` `draw_qr_matrix(layer, qr, qr_x, qr_top,
  46.0)?;`), dont la bordure de 5 mm (§ 3.5.2) va de x = 62 à 118 et de y = 42 à **98** ; au-dessus de la ligne, le corps
  de la facture ne descend pas sous une ligne de base de 110 (`:718` `Mm(ty.max(content_floor + 5.0)),`, `content_floor
  = SEP_Y`).
- `chaque_texte_s_imprime_en_noir` (`:2023-2077`) fixe exactement cinq `rg` dans l'ordre
  `[0,0,0] [1,1,1] [0.85,0,0] [1,1,1] [0,0,0]` (`:2052-2062`) et **17 `Tj` après le dernier `re`** (`:2076`
  `assert_eq!(apres_la_croix, 17);`) : des ciseaux **au trait** (`RG`, `S`), sans `rg` ni `re`, le laissent intact.
- Les helpers de test existants : `operations_du_pdf` (`:1542`), `nombre` (`:1558`), `textes_du_pdf` (`:1521`).

**La directive** (`docs/six-references/ig-qr-bill-v2.4-en.pdf`, § 3.7, p. 27) : « If the QR-bill with payment part and
receipt … are generated as a PDF document and sent electronically, the A6 format of the payment part and the receipt on
the left must be indicated by lines. Each of these lines must bear the scissors symbol ✂ or alternatively the
instruction "Separate before paying in" above the line (outside the payment part). » § 3.3 : partie paiement
148 × 105 mm, récépissé 62 × 105 à sa gauche. Échantillons SIX mesurés (`kesh-gate-logs/e36-12/mesure-echantillon-six-nr1.log`) :
un pointillé vertical à x = 61,7–61,9.

**Pourquoi pas un glyphe** : Helvetica intégrée ne porte pas ✂ (`:744`) ; printpdf pose `WinAnsiEncoding` sur toute
police intégrée, ZapfDingbats comprise (`printpdf-0.7.0/src/font.rs:76`). La clé
`invoice-pdf-qr-separate-before-paying` existe dans les quatre langues (`crates/kesh-qrbill/src/types.rs:270`,
`fr-CH/messages.ftl:789` `= A détacher avant le versement`), employée nulle part ; la story ne l'emploie pas (D2).

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

| Site | Verdict |
|---|---|
| `kesh-qrbill/src/pdf.rs:728-745` (`draw_separator`) | **dans le périmètre**, seul site de production modifié |
| `kesh-api/src/routes/invoice_pdf_service.rs:244` | inchangé ; prend le nouveau rendu |
| `kesh-api/src/qr_decode.rs:900`, test `#[ignore]` (« nécessite libpdfium.so », `:921`) | inchangé ; **à rejouer** au passage en revue : le lecteur de QR scannés lit exactement un PDF émis par Kesh, et la ligne verticale passe à 5 mm du QR. Commande documentée `:913-914` : `LD_LIBRARY_PATH=/home/gcorbaz/devel/kesh-gate-logs/pdfium-7920/lib cargo test -p kesh-api --lib qr_decode::tests::t4_pdfium -- --ignored --nocapture` |
| `kesh-qrbill/src/pdf.rs:129` (avoir), `kesh-api/src/qr_decode_synthetique.rs:462`, `:524` (générateur de test d'un scan, `:1`), `kesh-report/src/pdf.rs:317` (rapports) | hors : aucune section de paiement |

## Décisions

- **D1 — Deux lignes pointillées** : l'horizontale d'aujourd'hui inchangée (y = 105, x de 0 à 210, traits de 2 mm tous les
  4 mm, 0,3 pt) et une **verticale** à x = 62, de y = 0 à y = 105, même motif. *Écartée* : un trait plein — les
  échantillons SIX sont en pointillé, et c'est le motif déjà imprimé.
- **D2 — Des ciseaux dessinés au trait, sur chaque ligne** (C-36-12-2) : un symbole vectoriel d'environ 5 × 3 mm (deux
  anneaux en courbes de Bézier, deux lames croisées), **au trait seul** (`set_outline_color` noir, `Line` non fermées ou
  fermées non remplies : aucun `rg`, aucun `re`, aucun remplissage), dessiné dans `draw_separator`. Horizontal : centré sur
  y = 105, centre x ∈ [5, 15]. Vertical : tourné d'un quart de tour, centré sur x = 62, **centre y ∈ [100,5 ; 102,5]**, de
  sorte que le symbole entier reste dans y ∈ [98 ; 105] — au-dessus de la bordure de 5 mm du QR, qui monte à y = 98
  tant que le QR est à 93 (V1-6 ; la 36-12b, qui descend le QR, garde cette contrainte). Le pointillé s'interrompt sous
  chaque symbole. *Écartées* : la mention « A détacher avant le versement » (se place « au-dessus de la ligne », ne porte
  pas la ligne verticale) ; le glyphe de ZapfDingbats (encodage imposé) ; une police embarquée (#943) ; des ciseaux
  remplis (ajouteraient un `rg` et romperaient `chaque_texte_s_imprime_en_noir` — V1-10).
- **D3 — Rien d'autre ne bouge** : récépissé, partie paiement, QR et corps de la facture sont dessinés comme à
  `fed34a9c7` ; les tests de position existants restent inchangés.
- **D4 — Le figé ne bouge pas** : aucune ligne de `crates/kesh-api`, `crates/kesh-db` ni `crates/kesh-i18n` ne change.
- **D5 — Aucune migration.**

## Critères d'acceptation

1. **Ligne horizontale** : dans les opérations du PDF produit (`m` / `l`, en mm), des segments sur y = 105 couvrent x de
   0 à 210, sans trou de plus de 2,5 mm hors de l'interruption sous les ciseaux.
2. **Ligne verticale** : des segments sur x = 62 couvrent y de 0 à 105, même règle.
3. **Ciseaux** : deux tracés hors pointillé et hors QR (chemins à courbes ou lignes, ni `re`), chacun de boîte englobante
   de 3 à 7 mm sur sa grande dimension : l'un centré sur y = 105 (± 0,5) avec x ∈ [5, 15], l'autre centré sur x = 62
   (± 0,5) avec y ∈ [100,5 ; 102,5] et sa boîte entièrement dans y ∈ [98 ; 105].
4. **Rien ne se touche** : la boîte de chaque symbole est disjointe de la boîte de tout texte (majorée : de `Td.x` à
   `Td.x` + nombre de caractères × taille × 0,35278, de la ligne de base − 0,207 × taille × 0,35278 à la ligne de base +
   0,718 × taille × 0,35278) et du rectangle du QR élargi de 5 mm (x ∈ [62, 118], y ∈ [42, 98]) — sauf la ligne
   verticale elle-même, qui est le bord de cette bordure.
5. **Au trait, en noir** : les tracés des lignes et des ciseaux sont en `RG` 0 0 0 ; `chaque_texte_s_imprime_en_noir`
   passe **sans modification** (cinq `rg`, 17 `Tj` après le dernier `re`).
6. **Le reste inchangé** : tous les tests existants de `kesh-qrbill` passent sans modification (décompte des `#[test]` de
   `crates/kesh-qrbill` aux deux bornes, `git show <ref>:<fichier> | /usr/bin/grep -c '#\[test\]'`) ; le diff de
   production se limite à `draw_separator` et à ses constantes (`git diff origin/epic/36-0.18.0 -- crates/kesh-qrbill/src`
   lu et cité au Dev Agent Record).
7. **Le figé** : `git diff origin/epic/36-0.18.0 --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide ;
   `invoice_frozen_pdf_e2e.rs` et `invoice_pdf_e2e.rs` passent sans modification.
8. **Le lecteur de scans** : `t4_pdfium_scans_synthetiques` rejoué avec la commande de `qr_decode.rs:913-914`, vert, journal
   cité ; si `libpdfium` manque, le dire comme non lancé.
9. **Manuel** : § *Génération du PDF avec QR Bill* dit les deux lignes à ciseaux (AC contrôlé par `git grep` du `.tex`).
10. **Captures** avant / après de la section de paiement (fixture ordinaire), à côté de l'échantillon SIX Nr. 1, dans
    `/home/gcorbaz/devel/kesh-gate-logs/e36-12/captures/` ; les ciseaux se reconnaissent à l'œil, écrit au Dev Agent Record.

## Tests à écrire, et la preuve du rouge

Dans `crates/kesh-qrbill/src/pdf.rs`, `mod tests`, avant le correctif ; lancés sur `fed34a9c7`, journal
`/home/gcorbaz/devel/kesh-gate-logs/e36-12/rouges-12a.log` (nom du test et message d'échec de chacun). Helpers neufs :
`traits_du_pdf` (segments `m`/`l`, en mm, avec la couleur de trait `RG` en vigueur) et `chemins_du_pdf` (boîtes
englobantes des chemins hors `re`, courbes `c` comprises). Ils ne dépendent d'aucun module neuf : le binaire de test
compile sur `fed34a9c7`, et chaque rouge est un rouge de comportement.

| Test | AC | Rouge attendu à `fed34a9c7` |
|---|---|---|
| `la_ligne_horizontale_couvre_la_largeur` | 1 | **vert** (garde de non-régression ; son rouge par M3) |
| `une_ligne_verticale_separe_le_recepisse` | 2 | aucun segment sur x = 62 |
| `des_ciseaux_sur_chaque_ligne` | 3 | aucun chemin hors pointillé et QR |
| `les_ciseaux_ne_touchent_ni_texte_ni_qr` | 4 | rouge par l'absence de ciseaux (le test exige les deux boîtes avant de les comparer) |
| `lignes_et_ciseaux_au_trait_noir` | 5 | rouge par l'absence de ciseaux ; vert pour la ligne horizontale |

**Mutations** (après le commit du code, E5 ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-12/mut/`, restauration
`git restore --worktree -- crates/kesh-qrbill/src/pdf.rs` puis `touch`) :
- **M1** ligne verticale non dessinée → `une_ligne_verticale_separe_le_recepisse` rouge ;
- **M2** ciseaux non dessinés → `des_ciseaux_sur_chaque_ligne` rouge ;
- **M3** pointillé horizontal arrêté à x = 150 → `la_ligne_horizontale_couvre_la_largeur` rouge ;
- **M4** ciseaux verticaux centrés à y = 92 → `les_ciseaux_ne_touchent_ni_texte_ni_qr` rouge ;
- **M5** ciseaux remplis (`set_fill_color` + remplissage) → `chaque_texte_s_imprime_en_noir` rouge.

## Tâches

0. **Étape 0** : relire les lignes citées ; rejouer l'inventaire ; `ls -l` des chemins cités.
1. **Capture « avant »** : ajouter `crates/kesh-qrbill/examples/capture_qr_facture.rs` (écrit les PDF de fixtures
   synthétiques — ordinaire de type S, rue longue, sans référence — dans le répertoire donné en argument ; la 36-12b y
   ajoute les siennes), le committer seul, le lancer (`scripts/mem-guard.sh cargo run -p kesh-qrbill --example
   capture_qr_facture -- <dir>`), rendre en PNG (`mutool draw -r 150 -o <png> <pdf>`) dans `captures/avant/`.
2. Tests rouges, commit, `rouges-12a.log`.
3. `draw_separator` : ligne verticale, ciseaux (une fonction `dessiner_ciseaux(layer, cx, cy, vertical)`), constantes
   nommées et documentées (source de chaque cote) ; le commentaire `:744` disparaît ; doc-comment du module (`:1-9`) mis à
   jour.
4. Mutations M1 à M5.
5. Capture « après », `captures/apres/`, et comparaison avec l'échantillon SIX Nr. 1.
6. Manuel (`docs/manual/fr/user-manual.tex`, § *Génération du PDF avec QR Bill*, `:1273-`) : une phrase sur les deux
   lignes pointillées à ciseaux ; à `:1302`, la 0.18.0 parmi ce qu'un PDF figé ne prend pas. PDF régénéré à l'intégration.
7. Gates, Dev Agent Record, `Status: review`, prompt de revue P1.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; assemblée par l'orchestrateur dans `## [0.18.0] — Non publié`) :

- QR-facture : le récépissé et la partie paiement sont délimités par deux lignes pointillées portant chacune des
  ciseaux, comme la directive SIX l'exige pour une QR-facture en PDF ; la ligne verticale entre le récépissé et la partie
  paiement manquait. Le reste du bulletin et le code QR ne changent pas. **Les PDF déjà émis et figés ne changent pas** ;
  une facture validée jamais ouverte, un rappel et un PDF refigé par un administrateur prennent le nouveau rendu.
  (refs #944, refs #12)

## Dev Notes

### Hors périmètre

- **Tout le reste de #944** (grille, *Payable by*, largeurs, message, cascade) : **36-12b**.
- **#12** (CR-005, « Symbole ciseaux ✂ sur la ligne de séparation ») : la story en réalise la puce « Ajouter le glyph ✂ »
  (par un tracé, que § 3.7 permet : « scissors symbol ») ; ses autres puces (police embarquée, `é è ü €`, golden test)
  sont #943 (`docs/change_request.md:56-60`). **`refs #12`** à la PR, avec un commentaire qui dit ce qui est fait et
  renvoie à #943 ; pas `closes` (V1-9).
- Cadre vide du débiteur : **#1203** ; type K : **#1204**.

### Splitting — deux modules

`crates/kesh-qrbill` (`src/pdf.rs`, `examples/` neuf) et le manuel utilisateur.

### Fichiers communs

| Fichier | Autres stories | Règle |
|---|---|---|
| `crates/kesh-qrbill/src/pdf.rs` | **36-12b** | en série : la 36-12b part de la 36-12a fusionnée |
| `docs/manual/fr/user-manual.tex` | 36-12b, 36-15, 36-16, 36-18 (possibles) | paragraphes distincts ; PDF régénéré après l'union |
| `CHANGELOG.md` | toutes | non touché (C-e360-11) |

### Ce qui doit rester vrai

Le code QR, octet pour octet ; toutes les positions de texte ; le tout-noir des textes (`:2023`) ; les PDF figés.

### Gates (`e35-dev-commun.md` § *Quels gates, quand*, voie 2)

En cours : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo nextest run -p
kesh-qrbill` (sous `scripts/mem-guard.sh`, sans base). Passage en `review` : bases `kesh_e3612` remises à neuf sur
`kesh-mariadb-dev2:3307`, `flock /home/gcorbaz/devel/kesh-gate-logs/.gate-base2.lock scripts/mem-guard.sh
scripts/test-fast.sh --ci` (`TMPDIR=/tmp/kesh-voie2`, `DATABASE_URL=mysql://kesh:kesh_dev@127.0.0.1:3307/kesh_e3612`),
puis `scripts/gate-ci.sh kesh_ci_e3612` en voie 1, et le test `t4_pdfium` (AC8). Remédiation : `test-fast --ci`. Clôture :
`gate-ci` et E2E complet sur `kesh_e2e_e3612`. Une commande lourde à la fois.

### E2E

Aucun E2E neuf : le défaut est dans le dessin du document, que Playwright ne mesure pas ; la route qui sert le PDF ne
change pas (AC7). Suite complète à la clôture.

### Références

Directive § 3.3, § 3.5.2, § 3.7 ; échantillons `docs/six-references/samples-payment-part-en/muster-qr-zahlteile-en/` ;
fiche d'origine `36-12-qr-facture-separation.md` (`split`) ; validation P1 `kesh-gate-logs/e36-12/validation-p1.md`.

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
