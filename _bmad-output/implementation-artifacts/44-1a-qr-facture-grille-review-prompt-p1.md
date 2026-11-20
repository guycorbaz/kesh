# Prompt — revue de code P1 de la Story 44-1a, trois lentilles (B, E, A), première passe complète

Tu relis la **Story 44-1a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB), release **0.19.0**, sans
migration : **QR-facture — la grille SIX, *Payable by* à sa place, des largeurs mesurées, des paliers de secours**
(directive SIX v2.4, § 3.3 à 3.6.4, § 6.4 ; refs #944). Elle porte, dans `crates/kesh-qrbill` seulement :

- `src/pdf.rs` — la grille en constantes nommées (`TITRE_BASE`, `QR_X`, `QR_HAUT`, `INFO_X`, `INFO_BASE`,
  `RECU_INFO_BASE`, `MONTANT_BASE`, `POINT_DE_DEPOT_BASE`, `INTITULE_PAS`, `RUBRIQUE_PAS`, `ADRESSE_PAS`, `MESSAGE_PAS`,
  `RECU_INFO_BAS_MIN`…) ; `couper_ligne` à la largeur mesurée (`largeur_mm`) et `raccourcir` (« ... ») ;
  `lignes_adresse_par_role` (rôles Nom / Rue / Localité / LigneK) ; la **mise en page pure** `mise_en_page` (paliers
  `PalierRecu` R0, R1, R4 ; `PalierPaiement` P0, P3 ; `MiseEnPage`, `Colonne`, `Forme`, `information_du_recu`,
  `information_du_paiement`, `section_montant`) ; le dessin `dessiner_section_de_paiement`, qui remplace `draw_receipt` et
  `draw_payment_part` ; `MAX_CARACTERES_RECU` et `MAX_CARACTERES_PAIEMENT` retirés ; le doc du module ; les tests de la
  liste fermée de l'AC11 mis à jour et dix tests neufs (AC1 à AC9) ;
- `examples/capture_qr_facture.rs` — fixtures de la story (réaliste de neuf lignes, pire cas), allemand ajouté ;
- `docs/manual/fr/user-manual.tex` — § *Génération du PDF avec QR Bill* (le PDF se régénère à l'intégration, non dans la
  story : contrôle le `.tex`, et dis que le PDF du dépôt n'est pas à jour).

Worktree `/home/gcorbaz/devel/kesh-44-1`, branche `story/44-1-qr-facture-grille-largeurs`, partie de
`origin/epic/44-qr-facture-conforme`. Le diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-44-1 diff
origin/epic/44-qr-facture-conforme <dernier commit de code> -- crates docs` (le commit est nommé au Dev Agent Record de la
fiche ; les commits suivants ne portent que des comptes rendus). Langue du rapport : français. Modèle : **Sonnet** (passe
complète, première passe). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **Aucune entrée admise ne fait échouer le PDF** là où il se rendait à la v0.18.0 : la mise en page ne rend pas de
   `Result` ; le dernier palier (R4, P3) tient par construction ; `QrBillError::PdfGeneration` ne subsiste que pour ses
   causes d'avant (QR trop dense, devise, en-tête trop long), `InvalidAmount` pour le montant absent.
2. **Le code QR ne change pas** : charge utile et matrice intouchées ; seule sa place change (haut 88, croix en (90, 65)).
3. **Le figé ne change pas** : aucune ligne de `crates/kesh-api`, `crates/kesh-db`, `crates/kesh-i18n` ; un PDF figé est
   relu sur disque (`crates/kesh-api/src/routes/issued_invoice_pdf.rs:9-16`).
4. **Le tout-noir des textes** : `chaque_texte_s_imprime_en_noir` inchangé (cinq `rg`, 17 `Tj` après le dernier `re`) —
   l'ordre du dessin (titre de la partie paiement, QR, croix, puis les textes) doit le garder.
5. **Chaque texte dans sa zone** (AC3) : récépissé dans [5, 57], colonne dans [118, 205], rien dans le QR élargi
   (x ∈ [62, 118], y ∈ [37, 93]), rien sous y = 5, information du récépissé au-dessus de 41,52, rien sur le point de dépôt.
6. **Les paliers** : R1 retire la rue du créancier **et** du débiteur ensemble, seulement s'il y a une rue ; R4 garde nom
   et dernière ligne des deux adresses ; P3 une ligne chacune ; la cascade du récépissé ne dépend pas de celle de la
   colonne (« P3 n'entraîne pas R4 »).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/44-1a-qr-facture-grille.md` en entier (*Ce qui se lit au code*, D1 à D10, AC1 à
AC14, *Tests à écrire*, mutations, *Dev Notes*, **Change Log** et **Dev Agent Record**) ; le registre
`e44-choix-autonomes-44-1.md` (C-44-1-k) et, en lecture, `e36-choix-autonomes-36-12.md` (C-36-12-3 à -10) ; la fiche
suivante `44-1b-qr-facture-cascade.md` (ce qui est renvoyé à elle : R2, R3, P1, P2). Ne prends pas le Dev Agent Record pour
argent comptant : recoupe-le avec les journaux hors dépôt, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/e44/` :
`rouges.log` (et `rouges-brut.log`), `mut/synthese-campagne-1.log`, `mut/synthese-campagne-2.log`, `mut/M*.log` (le
script `mut/mutations_44_1a.py` est à **lire**, jamais à exécuter), `outils/coupe_afm.py` (à lire : le modèle de coupe
indépendant qui a servi aux fixtures), `pire-cas-boites.log`, `manuel-xelatex-*.log`, et les journaux de gate nommés au Dev
Agent Record. **Captures** (thème clair, 150 dpi, section de paiement recadrée) sous
`/home/gcorbaz/devel/kesh-gate-logs/e44/captures/` : `avant/section-*.png` (v0.18.0), `apres/section-*.png`, fixtures
ordinaire, rue longue, sans référence, réaliste R1, pire cas, en fr, de et en, et l'échantillon `echantillon-six-nr1.jpg`
— regarde-les si ton outil lit les images : la grille concorde-t-elle avec l'échantillon, *Payable by* est-il à sa place,
rien ne touche-t-il le QR ni la ligne verticale ?

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script
  du dépôt ou de `kesh-gate-logs` (dont `mut/mutations_44_1a.py`, `outils/coupe_afm.py`, `reset-db2-e441.sh`) ; ni
  `make`, `xelatex`, `latexmk`, `cargo`, `docker`, `mariadb`, `curl`, `mutool`, `pdftoppm`, `magick`, **`claude`**,
  `sed -i`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`,
  `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `diff` (sans redirection), `sha256sum`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
  (la seule forme permise du contrôle d'un PDF, E6).
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ;
  tout chemin que tu cites se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`), ligne lue citée (E9).

## Les axes obligatoires, pour chaque lentille

1. **Le manuel contre le code** : `docs/manual/fr/user-manual.tex`, les paragraphes « Le reçu et la partie paiement
   portent l'adresse », « La section de paiement suit la grille de la directive », « Une adresse très longue est
   raccourcie, le PDF se produit toujours », la fin de « Les caractères d'une QR-facture » (le message long), « Deux lignes
   à découper » et « Un PDF déjà figé ne change pas » (`git grep -n "suit la grille de la directive" -- docs`). Chaque
   phrase est-elle vraie au code : « le créancier et le débiteur y perdent ensemble leur rue » ; « leurs noms et leurs
   localités restent entiers » (sous R1 seulement) ; « le message tient sur une ligne et chaque ligne d'adresse aussi » ;
   « aucune adresse admise ne fait échouer le PDF » ; « la colonne de droite se décale d'un millimètre » ; « un PDF refigé
   … prend le nouveau rendu » (`refreeze`, `issued_invoice_pdf.rs`) ? Reste-t-il une phrase fausse (27 / 46 caractères,
   « ne bougent pas », « Deux limites demeurent ») ? Les manuels `de`, `en`, `it`, le README et `website/` disent-ils encore
   quelque chose de la mise en page du bulletin ? Le **PDF** du dépôt (`docs/manual/fr/user-manual.pdf`) n'est **pas**
   régénéré dans la story : vérifie-le par la forme E6 (« suit la grille de la directive » attendu à 0) et dis-le, sans en
   faire un finding.
2. **La géométrie** : recalcule depuis les constantes les positions de la fixture ordinaire (bases 96,5 ; 90,5 ; 98,0 ;
   35,0 ; 31,5 ; 20,5) et des paliers R4 et P3 (bas 57,9 et 49,4), le seuil 41,52, les marges serrées (haut des intitulés
   du montant 36,52 contre la bordure du QR à 37) ; confronte-les à D2 et à l'échantillon (C-36-12-3). Les titres, les
   intitulés de la partie paiement (x = 67, 82) et le point de dépôt dans les quatre langues : restent-ils à gauche de 118 ou
   de 57 ? `haut_de_ligne`/`bas_de_ligne` (CapHeight 718, Descender 207 de l'AFM) : juste pour les intitulés en gras ?
3. **La coupe et le raccourci** : `couper_ligne` (frontière de mot, mot coupé au dernier caractère qui tient, dernier
   morceau ouvert au mot suivant, espaces multiples, ligne vide) ; `raccourcir` (espace de fin retirée, « ... » ASCII dont
   la largeur compte, ligne qui tient laissée entière) ; cas limites : caractère sans octet WinAnsi (largeur 0), lettre
   repliée par `texte_imprime`, un seul caractère plus large que la colonne (possible ?), boucle infinie possible ?
   Coût : `largeur_valeur(&format!(…))` à chaque caractère (quadratique) — mesurable sur 140 caractères ?
4. **La cascade** : l'ordre et le critère de chaque palier ; `une_rue` (type S sans rue, type K, débiteur absent) ;
   `split_last` et le repli sur le dernier palier (aucun `unwrap`/`expect` atteignable par une entrée ?) ; les deux
   `debug_assert!` sont-ils vrais pour toute entrée (une adresse d'une seule ligne, un débiteur absent, sans référence) ?
   R4 sur une adresse dont `lignes_adresse` n'a qu'une ligne. La 44-1b pourra-t-elle insérer R2, R3, P1, P2 sans toucher
   R0, R1, R4, P0, P3 (D4) ?
5. **Les tests et les mutations** : chaque AC (1 à 11, 13, 14) a-t-il son test ou sa preuve ; un test peut-il passer **à
   vide** — `zones_violees` (le filtre de l'information du récépissé par `t.y > 35` ; le point de dépôt reconnu par sa
   base), `valeurs_du_recu` (8 pt et y > 35), `colonne` (x ≥ 118), `modules_noirs` (noir et y < 105 : la croix, des
   rectangles du corps ?), `aucune_entree_admise_n_echoue` (le caractère le plus large calculé, `@` ; 80 cas ; les mots de
   ⌊L/2⌋+1 et ⌊L/3⌋+1), `pire_cas_boites_independantes` (poppler, `MediaBox`, conversion depuis le haut ; ses filtres
   d'information du récépissé) ? Chaque mutation (M1 à M14, M6b, M13b) est-elle tuée **par le test attendu**
   (`mut/M*.log`) ? M6 est déclarée équivalente (module (10, 10) blanc) : l'argument tient-il ? Propose les mutations
   plausibles non jouées (par exemple : `RUBRIQUE_PAS` à 5 ; `INFO_LARGEUR` à 88 ; `raccourcir` sans `trim_end` ; R1
   appliqué même sans rue ; P3 qui garde le message entier ; l'ordre « titre puis QR » inversé).
6. **Ce qui doit rester vrai** : décompte des `#[test]` de `pdf.rs` aux deux bornes (`git show <ref>:<fichier> |
   /usr/bin/grep -c '#\[test\]'`) et liste fermée de l'AC11 : chaque test existant modifié est-il de la liste, avec ancienne
   et nouvelle valeur au Dev Agent Record ; aucun retiré sans remplaçant ; `git diff origin/epic/44-qr-facture-conforme
   --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide ; `invoice_frozen_pdf_e2e.rs` et `invoice_pdf_e2e.rs`
   verts sans modification (journal du gate).
7. **Les comptes rendus** : Dev Agent Record, Change Log et registre — chaque chiffre recompté à sa source (journal et
   commit cités), chaque chemin existant, aucune affirmation « vert » sans journal ; gates déclarés et non lancés (E2E,
   `gate-ci` de clôture, `t4_pdfium`) écrits comme tels ; `## Note de version` vraie au code.
8. **Les mots-clés de fermeture** : `git log origin/epic/44-qr-facture-conforme..HEAD --format=%B | /usr/bin/grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide.

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), puis la fiche pour trier ; cherche les défauts de code :
  calculs, bornes, flottants, `couper_ligne`/`raccourcir`, la cascade, l'ordre du dessin, commentaires faux.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e44/review-p1-B.md`.
- **E — Edge Case Hunter** : chaque chemin et chaque borne — langues (quatre et repli), débiteur absent, sans référence,
  sans message, message fait d'invisibles, type K, adresse étrangère, `Ĳ` et caractères sans octet WinAnsi, adresse d'une
  ligne, montant maximal (999 999 999.99), facture de rappel (`ReminderPdf`), PDF figé et refigé. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e44/review-p1-E.md`.
- **A — Acceptance Auditor** : AC1 à AC14, D1 à D10, les LOW de la validation P5 traités à l'étape 0, contre le code, les
  journaux et les captures ; les choix C-44-1-k sont-ils écrits, justifiés, réversibles ? Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e44/review-p1-A.md`.

## Forme du rapport

Pour chaque finding : identifiant (`B-1`, `E-1`, `A-1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (fichier:ligne),
preuve (sortie de commande copiée), remède proposé. Puis la liste des **axes exercés** et celle des **axes non exercés**
(une passe qui ne la donne pas ne compte pas). Puis le décompte par sévérité, recompté.
