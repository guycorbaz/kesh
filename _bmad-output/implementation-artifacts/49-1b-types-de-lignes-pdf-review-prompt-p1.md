# Revue de code P1 — Story 49-1b « Types de lignes — rendu PDF » (#1242)

*Prompt de la passe 1 de `bmad-code-review`, trois lentilles en contexte frais : **B** (Blind Hunter), **E** (Edge Case
Hunter), **A** (Acceptance Auditor). Une première passe n'est jamais ciblée et ne va pas à Haiku.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1b`, branche `story/49-1b-types-de-lignes-pdf`.
- **Diff** : `git -C /home/gcorbaz/devel/kesh-49-1b diff cbdd70026..HEAD -- crates` (`cbdd70026` = la 49-1a fusionnée dans
  `epic/49-types-de-lignes-facture` ; la fiche et le registre sont hors `crates`).
- **Fiche** : `_bmad-output/implementation-artifacts/49-1b-types-de-lignes-pdf.md` (D1 à D10, AC1 à AC14, T0 à T13, Dev Agent
  Record) ; registre : `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491b.md` (C-e491b-1 à 4).
- **Commits** : C (de capture) = `709370ed5` (précédé d'un premier essai `a8b15b240`, voir Dev Agent Record) ; F (références)
  = `12be35434` ; code de la story = `03afbfbca` et suivants.
- **Journaux** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491b/` — `t0-stabilite.log` (capture), `ac1-controles-*.log`,
  `rouge-*.log` et `mut/bilan.log` (mutations), `gate-*.log`, `test-fast-*.log`, `gate-ci-*.log`.
- **Captures** (AC12) : `/home/gcorbaz/devel/kesh-gate-logs/e491b/captures/*.png` (lis-les : outil de lecture d'image).

## Le contexte qui commande la revue

Guy a déjà émis de **vraies factures**, remises sur papier à un locataire. Un avoir n'est jamais figé et se rend toujours
avec le code en fonction ; une facture validée jamais téléchargée, ou un regel, aussi. **Rien de la 0.20.0 ne doit changer
le PDF d'une facture ou d'un avoir faits de lignes chiffrées.** La garde de l'API (`LineType::open_to_api()`) reste fermée :
cette story ne rend aucun commentaire créable par l'utilisateur.

## Axe premier — la preuve avant / après (toutes les lentilles)

1. **Le graphe** : rejoue en lecture `/home/gcorbaz/devel/kesh-gate-logs/e491b/ac1-controles.sh` (script en lecture seule ;
   lis-le avant) ou ses commandes une à une ; vérifie (1) à (7) de l'AC1 toi-même, notamment que C et F ne touchent aucun
   fichier de `RENDU`, que F ne porte que des fixtures, que F est ancêtre du premier commit de rendu, que fixtures et
   comparateur n'ont plus bougé depuis F, que chaque `sources.sha256` égale `git show cbdd70026:<fichier> | sha256sum`.
2. **Le comparateur** (`crates/kesh-qrbill/src/empreinte.rs`) : prouve-t-il ce qu'il prétend ? Que peut changer au rendu
   sans changer l'empreinte (ressources non suivies, opérandes, ordre, pages, flux, XObject, annotations) ? Les arrondis en
   `f32` des textes cachent-ils un écart ? Une assertion dont les deux côtés sortent du même extracteur est vraie par
   construction : où est-ce le cas, et est-ce assumé ?
3. **Les scénarios** (S1 à S9, `crates/kesh-qrbill/tests/rendu_avant_apres.rs` ; (a) à (d), `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`) :
   couvrent-ils ce que Guy imprime (QR-IBAN et QRR, multi-taux, arrondi, rappel, avoir, adresses longues, message long) ?
   Les scénarios ont-ils changé entre C et HEAD autrement que par `kind: LinePdfKind::Priced` (`git diff 709370ed5..HEAD --
   crates/kesh-qrbill/tests/rendu_avant_apres.rs crates/kesh-api/tests/pdf_avant_apres_e2e.rs`) ?
4. **La garde `configuration_transmise`** (`crates/kesh-api/tests/configuration_transmise.rs`, commit `89f5e77cb`,
   registre C-e491b-5) : l'exclusion `MODULES_DE_SOUTIEN_AUX_TESTS` est-elle assez étroite ? Son test de la raison
   prouve-t-il que le fichier exclu n'entre jamais en production, ou seulement qu'il est déclaré sous une feature ?
   Peut-on y glisser une lecture d'environnement de production sans que rien ne rougisse ?
5. **Le chemin chiffré** de `draw_invoice_section` : son flux d'opérations est-il identique à la 0.19.0 (garde, ordre des
   `use_text`, pas) ? `couper_ligne` à 8 pt rend-elle exactement la même coupe qu'avant ?

## Lentille B — Blind Hunter

Diff seul, sans la fiche. Défauts de code : logique, panique, arithmétique flottante, DRY (un seul comparateur, un seul
constructeur de ligne, une seule coupe), doc-comments faux, code mort, tests qui prouvent moins qu'ils ne disent.

## Lentille E — Edge Case Hunter

Chaque branche et chaque borne : description vide, blanche (`\n`, U+00A0, U+00AD, tabulations), `\r\n`, `\n` initiaux et
finaux, paragraphes vides consécutifs, mot plus large que 170 mm, caractères hors WinAnsi (CJK, émoji), commentaire seul,
commentaire en dernière ligne, au seuil exact de capacité (facture avec et sans récapitulatif, rappel, avoir), titre de
section, `TooManyLines(n)` avec `n` = lignes de la pièce, pré-filtre `MAX_LINES_PER_PDF` (il compte les commentaires),
courriel et lot de rappels (`invoice_email.rs`), PDF figé (`issued_invoice_pdf.rs` inchangé).

## Lentille A — Acceptance Auditor

AC1 à AC14 contre le code, un par un, avec la ligne lue citée ; T0 à T13 présents et à leur nom ; chaque mutation de la
fiche jouée et tuée (`rouge-*.log`, `mut/bilan.log`) ; D1 à D10 respectées ; AC8 (`git diff cbdd70026..HEAD -- crates/kesh-db
crates/kesh-core crates/kesh-api/src/routes/invoices.rs crates/kesh-api/src/routes/issued_invoice_pdf.rs
crates/kesh-api/src/routes/invoice_pdf.rs` vide) ; AC10 (`crates/kesh-i18n` inchangé) ; AC13 (`largeurs.rs`, `kesh-report/`,
`CHANGELOG.md`, `sprint-status.yaml` inchangés) ; contrôles de production de D4 (Dev Agent Record) ; Dev Agent Record
honnête (chaque chiffre a son journal).

## Le manuel (obligatoire)

Vérifie le manuel contre le code, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-49-1b/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(existence par `ls -l`). La fiche dit qu'**aucun manuel ne change** (la saisie d'un commentaire vient avec la 49-1c) :
vérifie que rien dans `docs/manual/fr/*.tex` ni le PDF ne décrit le rendu des lignes d'une manière que cette story rend
fausse (description réduite à 44 caractères, « figé au premier téléchargement », colonnes du tableau).

## Les captures (obligatoire)

Lis chaque PNG de `kesh-gate-logs/e491b/captures/` : le commentaire est-il lisible, aligné à gauche à la marge du tableau,
sans colonnes ni « 0.00 » ? Coupé sans mot tronqué ni texte hors page ? Le témoin chiffré est-il conforme au gabarit ? Le
titre de section est-il rendu comme un commentaire (D1) ? L'avoir porte-t-il le commentaire ?

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo` (build, test, run, fmt), ni `npm`, ni
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, ni aucun script mutant
(`kesh-gate-logs/e491b/mut/mutations.py`, `capture-e491b.sh`), ni `git commit`, `git checkout`, `git restore`, `git stash`,
`git worktree`, ni `rm`, `mv`, `cp`, `sed -i`, ni redirection vers un fichier, ni fichier temporaire. Seule écriture permise :
ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491b/review-p1-{B,E,A}.md` selon ta lentille. Chaque finding : sévérité
(CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et
non exercés** (un « 0 finding » sans elle ne compte pas) et le décompte par sévérité.
