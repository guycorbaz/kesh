# Prompt — revue de code P1 de la Story 36-12a, trois lentilles (B, E, A), première passe complète

Tu relis la **Story 36-12a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB), release **0.18.0**, sans
migration : **QR-facture — les deux lignes de séparation, leurs ciseaux, et le point de dépôt aligné à droite** (directive
SIX v2.4, § 3.7 et § 3.6.4 ; refs #944, refs #12). Elle porte, dans `crates/kesh-qrbill` seulement :

- `src/pdf.rs` — `draw_separator` récrite (ligne horizontale à y = 105, ligne **verticale** neuve à x = 62, pointillé
  interrompu sous chaque ciseau et reparti à 0,25 mm de sa boîte — C-36-12a-1), `traits_du_pointille`, `trait_droit`,
  `dessiner_ciseaux` (deux anneaux en Bézier, deux lames en triangles fermés, au trait seul), leurs constantes ; le point de
  dépôt aligné à droite à x = 57 par `largeur_mm` (`RECU_BORD_DROIT`) ; le doc-comment du module ; sept tests neufs
  (AC1 à AC7) et leurs helpers (`chemins_du_pdf`, `relever`, `trous`, `textes_mesures`, `i18n_de`…) ;
- `src/largeurs.rs` (neuf) — la table des chasses d'Helvetica et d'Helvetica-Bold par nom de glyphe WinAnsi
  (`WINANSI`, `CHASSES`, engendrées par un script ; `Euro` ajouté à 556), `largeur_mm`, quatre tests ;
- `src/lib.rs` (déclaration du module), `src/validation.rs` (`est_caractere_qr` passée `pub(crate)`) ;
- `examples/capture_qr_facture.rs` (neuf) — les PDF synthétiques de la recette visuelle, fr et en ;
- `docs/manual/fr/user-manual.tex` — § *Génération du PDF avec QR Bill* (le PDF se régénère à l'intégration, non dans la
  story : contrôle le `.tex`, et dis que le PDF n'est pas à jour).

Worktree `/home/gcorbaz/devel/kesh-36-12`, branche `story/36-12-qr-facture-separation`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-12 diff origin/epic/36-0.18.0 <dernier commit de code> -- crates docs` (le commit est
nommé au Dev Agent Record de la fiche ; les commits suivants ne portent que des comptes rendus). Langue du rapport :
français. Modèle : **Sonnet** (passe complète, première passe). Contexte frais.

⚠️ **Les propriétés centrales.**
1. **Le code QR ne change pas** : mêmes modules (`re`) avant et après ; aucune donnée du QR ne passe par le code neuf.
2. **Le figé ne change pas** : aucune ligne de `crates/kesh-api`, `crates/kesh-db`, `crates/kesh-i18n` ; un PDF figé est
   relu sur disque, jamais régénéré (`crates/kesh-api/src/routes/issued_invoice_pdf.rs:9-16`).
3. **Le tout-noir des textes** : aucun `rg` neuf (ciseaux au trait seul, `RG`) ; `chaque_texte_s_imprime_en_noir`
   inchangé (`pdf.rs`, cinq `rg`, 17 `Tj` après le dernier `re`).
4. **Les ciseaux ne touchent rien** : ni un texte, ni le QR élargi de 5 mm (x ∈ [62, 118], y ∈ [42, 98]) ; le vertical
   reste sous l'horizontale (boîte jusqu'à y = 103,35).
5. **Le point de dépôt finit à x = 57 dans les quatre langues** et au repli anglais, ligne de base 5.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-12a-qr-facture-lignes-ciseaux.md` en entier (*Ce qui se lit au code*, D1 à
D7 et leurs annotations d'étape 0, *Définitions des tests*, AC1 à AC13, *Tests à écrire*, mutations M1 à M9, *Dev Notes*
— limite connue —, **Change Log** et **Dev Agent Record**) ; le registre `e36-choix-autonomes-36-12.md` (C-36-12-2, -4,
-10 ; C-36-12a-1, -2) ; la fiche suivante `36-12b-qr-facture-grille-largeurs.md` (ce qui est renvoyé à elle). Ne prends
pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt, en lecture seule, sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-12/` : `rouges-12a.log`, `mut/M1.log` à `mut/M9.log` (les scripts
`mut/muter.py` et `mut/jouer.sh` sont à **lire**, jamais à exécuter), `engendrer-largeurs.py` (à lire),
`largeurs-tables.rs.txt` (sa sortie), `comparaison-avant-apres.log`, `t4-pdfium.log`, et les journaux de gate nommés au
Dev Agent Record. **Captures** (thème clair, 150 dpi ; zooms à 600 dpi) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-12/captures/` : `comparaison-fr.png`, `comparaison-en.png` (avant / après /
échantillon SIX Nr. 1, de haut en bas), `apres/zoom-ciseaux-deux-fr.png`, `avant/*-section.png`, `apres/*-section.png`,
et les PNG des pages entières des trois fixtures — regarde-les si ton outil lit les images : les ciseaux sont-ils
reconnaissables, le point de dépôt dégagé de la ligne, rien d'autre déplacé ?

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script
  du dépôt ou de `kesh-gate-logs` (dont `mut/jouer.sh`, `mut/muter.py`, `engendrer-largeurs.py`) ; ni `make`, `latexmk`,
  `cargo`, `docker`, `mariadb`, `curl`, `mutool`, `magick`, **`claude`**, `sed -i`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`,
  `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `diff` (sans redirection), `sha256sum`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
  (la seule forme permise du contrôle d'un PDF, E6).
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ;
  tout chemin que tu cites se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`), ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel contre le code** : `docs/manual/fr/user-manual.tex`, les deux paragraphes « Deux lignes à découper,
   chacune avec ses ciseaux » et « Un PDF déjà figé ne change pas » (`git grep -n "Deux lignes à découper" -- docs`).
   Chaque phrase est-elle vraie au code : « il finit à 5 mm de la ligne verticale dans toutes les langues » ; « au lieu de
   la toucher en allemand et de la chevaucher en anglais et en italien » ; « un PDF refigé par un administrateur prend le
   nouveau rendu » (lis `refreeze`, `crates/kesh-api/src/routes/issued_invoice_pdf.rs`) ; la limite des capitales larges ?
   Le **PDF** (`docs/manual/fr/user-manual.pdf`) n'est **pas** régénéré dans la story : vérifie-le par la forme E6
   (« Deux lignes à découper » attendu à 0) et dis-le, sans en faire un finding.
2. **La géométrie** : recalcule depuis les constantes (`CISEAU_H_X`, `CISEAU_V_Y`, `ANNEAU_*`, `LAME_*`, `CISEAU_U`,
   `SEP_*`) les boîtes des deux ciseaux, le début et la fin des deux pointillés, leurs trous ; confronte-les à AC1 à AC4 et à
   D2 (grande dimension de 3 à 5 mm, petite ≤ 3,5 ; centres ; pas de contact avec la bordure du QR, les titres, la ligne
   horizontale au croisement). `traits_du_pointille` : bornes, flottants accumulés (`decalage += SEP_PAS`), trait de
   longueur nulle ou négative, intervalle vide ou inversé. `dessiner_ciseaux` : les points marqués `true` produisent-ils
   bien des `c` (règle de `printpdf-0.7.0/src/line.rs:55-111`, à lire) et jamais un `v`/`y` dégénéré ou un `l` ?
3. **La table des largeurs** : `largeurs.rs` contre son script (`engendrer-largeurs.py`) et sa sortie
   (`largeurs-tables.rs.txt`, comparée au fichier au Dev Agent Record) ; contre `lopdf-0.31.0/src/encodings/mappings.rs`
   (`WIN_ANSI_ENCODING`, premier octet d'un caractère, comme `string_to_bytes` de `encodings/mod.rs`) et les AFM
   (`/usr/share/texlive/texmf-dist/fonts/afm/adobe/helvetic/phvr8a.afm`, `phvb8a.afm`, empreintes au Dev Agent Record).
   Les témoins (AC8) sont-ils lus à l'AFM et non à la table ? Un caractère que `lopdf` **retire** (sans octet) compte pour
   la plus large : est-ce le bon choix pour un alignement à droite ? Le doc-comment du module dit-il vrai ?
4. **Les tests et les mutations** : chaque AC (1 à 13) a-t-il son test ou sa preuve, et ce test peut-il passer **à
   vide** — `chemins_de_la_section` filtre-t-il trop (y ≤ 108) ou trop peu ; `relever` classe-t-il un chemin du QR, de la
   croix ou du corps comme ciseau ; `segment` reconnaît-il un trait écourté ; `textes_mesures` lit-il la graisse juste
   (ressources indirectes) ; un test qui « réunit les écarts » masque-t-il un `montage` ? Chaque mutation M1 à M9 est-elle
   tuée **par le test attendu** (`mut/M*.log`) ? M8 a été jouée au commit de la table, avant le correctif : est-ce
   suffisant ? Propose les mutations plausibles non jouées (par exemple : `SEP_ECART_CISEAUX` à 0 ; lames vers la gauche ;
   `is_closed: false` sur un anneau ; `largeur_mm` en maigre pour le point de dépôt ; le point de dépôt à `RECEIPT_W` au
   lieu de `RECU_BORD_DROIT` ; `CISEAU_V_Y` à 102,5 ; une ligne verticale jusqu'à 108).
5. **Ce qui doit rester vrai** : décompte des `#[test]` de `kesh-qrbill` aux deux bornes (`git show <ref>:<fichier> |
   /usr/bin/grep -c '#\[test\]'`) ; aucun test existant modifié (le diff des lignes retirées de `mod tests` doit être
   vide) ; `git diff origin/epic/36-0.18.0 --stat -- crates/kesh-api crates/kesh-db crates/kesh-i18n` vide ; le diff de
   production limité à ce que liste l'AC9 ; `comparaison-avant-apres.log` (QR identique, seul le `Td` du point de dépôt
   change).
6. **Les comptes rendus** : Dev Agent Record, Change Log et registre — chaque chiffre recompté à sa source (journal et
   commit cités), chaque chemin existant, aucune affirmation « vert » sans journal ; gates déclarés et non lancés (E2E et
   `gate-ci` de clôture) écrits comme tels ; `## Note de version` vraie au code (« quelle que soit la langue du client » ;
   « un rappel et un PDF refigé … prennent le nouveau rendu »).
7. **Les mots-clés de fermeture** : `git log origin/epic/36-0.18.0..HEAD --format=%B | /usr/bin/grep -niE
   '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide.

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), puis la fiche pour trier ; cherche les défauts de code :
  calculs, bornes, flottants, cas limites de `traits_du_pointille` et `largeur_mm`, lisibilité des constantes, commentaires
  faux. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e36-12/review-p1-B.md`.
- **E — Edge Case Hunter** : chaque chemin et chaque borne — langues (quatre et repli), traductions plus longues, texte
  vide ou fait d'un caractère sans octet WinAnsi, fixtures (sans débiteur, sans référence, adresse étrangère, capitales
  larges — la limite connue est-elle exactement bornée ?), facture de rappel (`ReminderPdf`), PDF figé et refigé.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e36-12/review-p1-E.md`.
- **A — Acceptance Auditor** : AC1 à AC13, D1 à D7, les annotations d'étape 0 (V3-1 à V3-8) contre le code, les journaux
  et les captures ; les choix C-36-12a-1 et -2 sont-ils écrits, justifiés, réversibles ? Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e36-12/review-p1-A.md`.

## Forme du rapport

Pour chaque finding : identifiant (`B-1`, `E-1`, `A-1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (fichier:ligne),
preuve (sortie de commande copiée), remède proposé. Puis la liste des **axes exercés** et celle des **axes non exercés**
(une passe qui ne la donne pas ne compte pas). Puis le décompte par sévérité, recompté.
