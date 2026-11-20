# Revue de code P1 de la story 49-4a — prompt des trois lentilles (Opus, contexte frais)

Trois agents **Opus**, en parallèle, contexte frais, un par lentille : **B** (Blind Hunter), **E** (Edge Case Hunter),
**A** (Acceptance Auditor). Français soigné. Aucune question : tu tranches et tu le dis.

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-49-4`, branche `story/49-4-designation-longue-pdf` (porte aussi la fiche 49-4b, **hors
  revue**). Fiche : `_bmad-output/implementation-artifacts/49-4a-capacite-pdf-avant-validation.md` — elle fait foi (décisions
  D1 à D7, AC1 à AC11, tests T1 à T6 et T10, Dev Agent Record).
- **Base du diff** : `$BASE` = `git merge-base HEAD origin/epic/49-types-de-lignes-facture` = `f57f6ac80` (Dev Agent Record).
  Diff à relire : `git -C /home/gcorbaz/devel/kesh-49-4 diff f57f6ac80..HEAD` (les commits `docs(49-4a)` et `docs(49-4b)` de
  la fiche sont du texte de fiche ; seul le volet Q1 (c) de la 49-4b est touché).
- Kesh est **en production** chez Guy, avec de vraies factures émises et remises sur papier. **Aucun octet de PDF d'aucune pièce
  ne doit changer** (factures, avoirs, rappels, PDF figés). Une facture validée n'est concernée par rien.
- Décisions de Guy du 2026-11-20 qui font foi : **aucune réserve du bloc de rappel** (D2 ; limite au manuel, sortie par avoir,
  plusieurs pages en v0.21.0, #1374) ; ne rien élargir — un point hors fiche s'écrit comme issue à ouvrir.

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (`git diff f57f6ac80..HEAD -- crates frontend/tests`), sans la fiche, puis la
  fiche pour trier. Défauts de code, de logique, de tests qui prouvent moins qu'ils ne disent (vrais par construction, oracle
  partagé avec ce qu'ils testent, montage qui passe à vide).
- **E — Edge Case Hunter** : chaque branche et chaque bord — seuils à l'égalité flottante (`ty < plancher`), récapitulatif de
  0 à 5 lignes, arrondi nul au centime (`0.004`) ou non (`0.005`), commentaires vides, faits de blancs, sans rien d'imprimable
  (« Ω », U+200B), pré-filtre `MAX_LINES_PER_PDF` après le contrôle, brouillon absent ou non brouillon (404/409 inchangés),
  revalidation d'une facture dévalidée, réglage `round_to_5_centimes` absent (lecture à `true`), avoir et rappel non contrôlés,
  course `PUT`/validation (assumée, D1).
- **A — Acceptance Auditor** : chaque AC (AC1 à AC11) contre le code, les tests et le Dev Agent Record ; chaque décision D1 à
  D7 ; chaque affirmation du Dev Agent Record contre son journal (chiffre, commit).

## Axes obligatoires (toutes lentilles, chacune selon son angle)

1. **Aucun octet de PDF ne change** : `crates/kesh-qrbill/src/pdf.rs` — la géométrie extraite (`TABLEAU_Y`,
   `TABLEAU_EN_TETES_PAS`, `TABLEAU_FILET_PAS`, `LIGNE_PAS`, `RESERVE_DU_TOTAL`, `plancher_des_lignes`) reproduit-elle **les
   mêmes opérations `f32`, dans le même ordre**, que le code d'avant (`git show f57f6ac80:crates/kesh-qrbill/src/pdf.rs`) ? Les
   tests avant/après de la 49-1b (`crates/kesh-qrbill/tests/rendu_avant_apres.rs`, `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`)
   sont-ils **non modifiés** (`git diff f57f6ac80..HEAD --stat -- crates/kesh-qrbill/tests crates/kesh-api/tests/fixtures
   crates/kesh-api/tests/pdf_avant_apres_e2e.rs crates/kesh-qrbill/src/empreinte.rs`) et verts au journal ?
2. **Source unique, oracle indépendant** (D1, AC3) : `controler_les_lignes` et la garde de `draw_invoice_section` partagent-ils
   la géométrie et le compte des lignes sans que T2 (`pdf.rs::tests::le_controle_et_le_rendu_s_accordent`) devienne vrai par
   construction ? Le dessin parcourt `lignes_de_commentaire` directement, `lignes_imprimees` en compte la longueur (écart déclaré
   au Dev Agent Record) : juger.
3. **La route** : `crates/kesh-api/src/routes/invoices.rs` (`validate_invoice_handler`) et
   `crates/kesh-api/src/routes/invoice_pdf_service.rs` (`depasse_le_prefiltre`, `lignes_pdf`, `controler_la_capacite_du_pdf`) :
   rien d'écrit sur un refus ; l'arrondi calculé comme la validation le figera ; ordre contrôle puis pré-filtre (V2-1) ; aucune
   réponse du rendu changée (`render_document`).
4. **Erreur et textes** : `crates/kesh-api/src/errors.rs` (variante `InvoiceLinesDoNotFitPdf`, `message_lignes_hors_page`), les
   quatre catalogues `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (clé `error-invoice-lines-do-not-fit-pdf`),
   `crates/kesh-api/tests/textes_coherents.rs` (G9 et le test neuf). Traductions justes ?
5. **Tests** : `crates/kesh-api/tests/invoice_validation_pdf_e2e.rs` (T1, trois tests) et
   `frontend/tests/e2e/invoice-validation-pdf.spec.ts` (T6) ; leurs rouges et mutations au journal
   (`/home/gcorbaz/devel/kesh-gate-logs/e494/dev/`).
6. **Le manuel, PDF compris** (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : `docs/manual/fr/user-manual.tex`
   (item « Lignes de facture », `:1456`), `docs/manual/fr/admin-manual.tex` (§ `sec:maj-0-20-0`, « Pour les intégrations »),
   `docs/api-external.md` (§ *Valider une facture — des lignes qui ne tiennent pas sur la page du PDF* et table des codes, lignes
   `INVOICE_TOO_MANY_LINES_FOR_PDF` et `INVOICE_LINES_DO_NOT_FIT_PDF`). Le manuel dit-il ce que fait le code (huit lignes,
   récapitulatif TVA ou arrondi, refus à la validation, brouillon modifiable, limite du rappel et sortie par avoir) ? Les PDF
   régénérés localement (non commités) : **seule forme permise**, sans redirection :
   `pdftotext /home/gcorbaz/devel/kesh-49-4/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Kesh le vérifie à la validation'`
   (et de même `admin-manual.pdf` sur `INVOICE_LINES_DO_NOT_FIT_PDF` — le PDF peut couper le mot : essayer aussi un fragment) ;
   l'existence par `ls -l`.
7. **Captures** (recette visuelle, AC7) dans `/home/gcorbaz/devel/kesh-gate-logs/e494/captures/` : les **regarder** (outil de
   lecture d'image) — dialogue du refus lisible et entier ; PDF au seuil avec conditions de paiement (chevauchement ?) ; PDF au
   seuil avec récapitulatif.
8. **Périmètre et coordination** : script en lecture seule `/home/gcorbaz/devel/kesh-gate-logs/e494/dev/controles-diff.sh
   /home/gcorbaz/devel/kesh-49-4 f57f6ac80` (sortie vide attendue : D5, AC4, AC10) ; `crates/kesh-qrbill/src/lib.rs` ne change
   que par le ré-export ; aucune ligne de `kesh-db`, `kesh-core`, `frontend/src`, `CHANGELOG.md`.

## Interdits

Lecture seule. **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (build, test, nextest, clippy, fmt), ni `npm`,
ni `npx playwright`, ni `make`, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`,
`scripts/mem-guard.sh`, ni `sqlx`, ni `docker`, ni `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `restore`,
`stash`, `push`, `tag`, `rebase`, `merge`, `worktree`), ni `gh` en écriture, ni `rm`, `mv`, `cp`, `sed -i`, `perl -pi`, ni
redirection `>`/`>>`, ni fichier temporaire. Permis : `git diff`, `git show`, `git log`, `git grep`, `git cat-file`, `cat`,
`sed -n`, `/usr/bin/grep`, `ls -l`, `pdftotext … -` (forme de l'axe 6), le script `controles-diff.sh` (lecture seule) et
l'outil de lecture d'image. `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`. Rien sous `~/kesh/`.

## Rapport

Seul fichier que tu écris, par l'outil d'écriture : `/home/gcorbaz/devel/kesh-gate-logs/e494/review-p1-<B|E|A>.md`.

- Verdict `CRITICAL/HIGH/MEDIUM/LOW = n/n/n/n`.
- Chaque finding : sévérité, lieu (`fichier:ligne`), **prémisse relue** (ligne citée ; toute affirmation de présence ou
  d'absence = sortie `grep -nF` copiée), conséquence, remède proposé, et s'il touche la production.
- **Axes exercés et axes non exercés**, listés (un « 0 finding » sans cette liste ne compte pas).
