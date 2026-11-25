# Revue de code P1 — Story 49-1c « Types de lignes — l'écran, la ligne de commentaire » (#1242)

*Prompt de la passe 1 de `bmad-code-review`, trois lentilles **Opus** en contexte frais : **B** (Blind Hunter), **E** (Edge
Case Hunter), **A** (Acceptance Auditor). Une première passe n'est jamais ciblée et ne va pas à Haiku.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1c`, branche `story/49-1c-types-de-lignes-ecran`.
- **Diff** : `git -C /home/gcorbaz/devel/kesh-49-1c diff 5156f2b7b..HEAD -- crates frontend docs` (`5156f2b7b` = la
  49-1a, la 49-1b et la 48-13 dans la branche, base sur laquelle la fiche a été écrite ; la fiche, le registre et
  `sprint-status.yaml` sont sous `_bmad-output/`).
- **Fiche** : `_bmad-output/implementation-artifacts/49-1c-types-de-lignes-ecran.md` (D1 à D12, AC1 à AC12, T1 à T12, Dev
  Agent Record) ; registre : `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491c.md` (C-e491c-1 à 9).
- **Commits** : code de la story = `cea6910db`, `4cb6633e9` (fixture de contact à adresse structurée pour T5 et T6) et
  `7f8bf6a00` (garde G4-bis de `crates/kesh-api/tests/textes_coherents.rs` : un point de code `U+NNNN` n'est plus lu
  comme un compte, C-e491c-9) ; voir `git log --oneline 7cc2bf0d5..HEAD`. La session de développement a été tuée par
  systemd-oomd le 2026-11-20 à 04:23 pendant ses gates, puis reprise : le Dev Agent Record le dit.
- **Journaux** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/` — `rouge-T1-T6.log` (mutations backend),
  `rouge-T7-T8-T9.log` (mutations frontend), `rouge-T10.log`, `rouge-T12.log`, `rejeu-tests-5156f2b7b.log` (AC8 c),
  `gate-cible-*.log`, `test-fast-1.log` (rouge à `4cb6633e9`, motif de `7f8bf6a00`), `test-fast-2.log`, `gate-ci-2.log`,
  `front-*.log`, `e2e-complet-1.log`, `e2e-T10-vert.log`, `check-types.log`, `vitest-*.log`, `make-manuels.log` ;
  scripts de mutation dans `mut/` (à lire, jamais à lancer). Le Dev Agent Record, § *Gates de passage en `review`*,
  donne pour chacun son commit et ses chiffres.
- **Captures** (AC11) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/captures/` — `01-formulaire-avec-commentaire.png`,
  `02-formulaire-commentaire-vide-bloque.png`, `03-fiche-brouillon.png`, `04-fiche-validee.png`,
  `05-facture-pdf-1.png` (rendu de `05-facture.pdf`), `06-fiche-avoir.png`. Lis-les (outil de lecture d'image).
- **Manuels compilés** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/manuel/fr/user-manual.pdf` et
  `admin-manual.pdf`, compilés depuis les `.tex` de la branche (copie hors dépôt, C-e491c-7 : les PDF du dépôt ne
  bougent pas dans une story).

## Le contexte qui commande la revue

Guy a déjà émis de **vraies factures**, remises sur papier à un locataire. **Rien ne doit changer pour une facture ou un avoir
existants** : lignes, totaux, TVA, écritures, numéros, PDF, et ce qu'un brouillon fait de lignes chiffrées envoie quand on le
rouvre et l'enregistre (AC8). Cette story **ouvre `comment`** à l'API (la garde `LineType::open_to_api()`), garde `heading`
fermé, soustrait le taux neutre d'un commentaire à la liste blanche des taux, et livre l'écran.

## Axes communs (toutes les lentilles)

1. **AC8, les factures existantes** : `git diff 5156f2b7b..HEAD` vide sur `crates/kesh-qrbill`, `kesh-core`, `kesh-report`,
   les migrations, `test-schema`, les dépôts de `kesh-db`, `invoice_pdf_service.rs`, `issued_invoice_pdf.rs`,
   `invoice_pdf.rs`, `credit_notes.rs`, `invoice_email.rs`, les deux dossiers `tests/fixtures/avant-0.20.0/` et les trois
   tests avant/après. Un brouillon chiffré-seulement rouvert envoie-t-il **exactement** ce qu'il envoyait, plus
   `lineType: 'priced'` ? Cherche toute différence de charge (clé en plus ou en moins, `revenueAccountId` absent au lieu
   de `null`, prix non normalisé) entre l'ancien `{ ...stripUiKey(l), unitPrice }` et `ligneAEnvoyer`.
2. **La relecture d'un brouillon** : les deux sites (`initLines`, `reloadFromServer`) passent-ils par le seul
   `ligneDepuisReponse` ? Un champ peut-il encore manquer à l'un des deux ? Une ligne `heading` relue reste-t-elle `heading` ?
3. **Le total et les contrôles** : `computeInvoiceTotal`, `inactiveRates`, le repli `_vatPlaceholder`,
   `invalidRevenueAccountLines`, `validateClient` et la charge filtrent-ils tous par `estChiffree` ? Un commentaire
   peut-il recevoir un taux, un compte, un montant, ou bloquer l'envoi pour une raison de ligne chiffrée ?
4. **La liste blanche** (`taux_des_lignes_chiffrees`, D4) : seul point de collecte au `POST` et au `PUT` ? Une ligne
   chiffrée à 0 % désactivé reste-t-elle refusée ?
5. **Le cycle d'imports** évité par `invoice-line-type.ts` (C-e491c-1) : est-il réel, et la solution tient-elle ?
6. **La garde G4-bis assouplie** (`7f8bf6a00`) : l'exclusion de `U+NNNN` dans `nombres_isoles` ouvre-t-elle un faux vert
   (un vrai numéro de compte précédé de `U+` dans un texte contrôlé) ? Inventorie les sites `U+` des six fichiers que la
   garde lit (`git grep -n -F "U+" -- docs/manual/fr/user-manual.tex docs/user-guide/fr/getting-started.md
   crates/kesh-i18n/locales`).

## Lentille B — Blind Hunter

Diff seul, sans la fiche. Défauts de code : logique, type TypeScript trop large ou trop étroit (`CreateInvoiceLineRequest`
aux montants désormais optionnels : qui d'autre le construit ?), gabarit Svelte (`{#if}` imbriqués, `colspan`, `white-space:
pre-line` et blancs du gabarit), accessibilité (`aria-label`, `placeholder`), DRY, doc-comments faux, code mort, tests qui
prouvent moins qu'ils ne disent (une assertion dont les deux côtés sortent du même extracteur).

## Lentille E — Edge Case Hunter

Chaque branche et chaque borne : commentaire vide, blanc seul, sauts de ligne en tête et en fin (le serveur fait `trim`),
mille caractères et plus (borne serveur, D12 : aucune borne à l'écran), commentaire seul, commentaires seuls (brouillon
enregistré, validation refusée), commentaire en première et en dernière position, suppression de la dernière ligne
chiffrée, ligne `heading` relue (import, SQL), conflit de version avec et sans commentaire, ligne sans `lineType` (fixture,
donnée ancienne), saisie NFD au `POST` et au `PUT`, taux 0 % désactivé avec et sans ligne chiffrée à 0 %, avoir d'une
facture à commentaire, PDF de l'avoir.

## Lentille A — Acceptance Auditor

AC1 à AC12 contre le code, un par un, avec la ligne lue citée ; T1 à T12 présents et à leur nom ; chaque mutation de la
fiche jouée et tuée (`rouge-*.log`) ; D1 à D12 respectées, et les écarts déclarés (C-e491c-3 : `colspan` 6 et non 5 ;
C-e491c-4 : positions 0 à 2 et non 1 à 3) sont-ils justes ? AC8 (c) : le rejeu de `5156f2b7b` n'a-t-il qu'`AC9 a` pour
échec, et pour le seul `lineType` ? `git diff --numstat 5156f2b7b..HEAD` des quatre fichiers de test : ajouts seulement ?
AC9 : `sitesTotal` recompté (3495 → 3499) et les quatre clés dans les quatre catalogues. AC10 (d) : les quatre contrôles
du Dev Agent Record, rejoués. Dev Agent Record honnête : chaque chiffre a son journal et son commit.

## Le manuel (obligatoire)

Vérifie le manuel contre le code, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-gate-logs/e491c/manuel/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(et de même pour `admin-manual.pdf` ; existence par `ls -l` ; l'apostrophe du PDF est typographique). Points à vérifier :
`user-manual.tex` (item « Lignes de facture » : les trois boutons, le commentaire hors total/TVA/écriture, huit lignes au
plus, commentaires seuls non validables ; paragraphe des caractères : ce que le commentaire replie, garde, perd — la règle
est-elle celle de `texte_de_commentaire`, `crates/kesh-qrbill/src/pdf.rs` ? ; l'avoir qui reprend les commentaires),
`admin-manual.tex` (« Pour les intégrations ») et `docs/api-external.md` (`comment` accepté, `heading` refusé, le contrat
de D3). Une phrase devenue fausse ailleurs au manuel ?

## Les captures (obligatoire)

Lis chaque PNG : le bouton « Commentaire » est-il à côté des deux autres ? Le champ couvre-t-il la largeur, sans taux, compte
ni total ? Le bandeau du commentaire vide nomme-t-il la ligne ? Sur les fiches (brouillon, validée, avoir), le commentaire
tient-il toute la largeur, sauts de ligne gardés, sans « 0.00 » ni « (défaut société) » ? Le PDF l'imprime-t-il à gauche,
sans colonnes ?

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo` (build, test, run, fmt, nextest), ni `npm`/`npx`
(vitest, playwright, check, build), ni `make`, ni `scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/check-dev-grants.sh`, `sqlx`, ni aucun script de
`kesh-gate-logs/e491c/` (`mut/mut-back-e491c.py`, `mut/mut-front-e491c.py`, `gate-cible-e491c.sh`, `e2e-e491c.sh`,
`bases-neuves-e491c.sh` — il efface les bases de la story —, `test-fast-ci-e491c.sh`, `gate-ci-e491c.sh`,
`front-e491c.sh`), ni `pdftotext` vers un fichier (seulement vers `-`), ni `git commit`, `git checkout`, `git restore`,
`git stash`, `git worktree`, ni `rm`, `mv`, `cp`, `sed -i`, ni `docker`, ni `mariadb`, ni redirection vers un fichier, ni
fichier temporaire. Seule écriture permise : ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep`
ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491c/review-p1-{B,E,A}.md` selon ta lentille. Chaque finding : sévérité
(CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et
non exercés** (un « 0 finding » sans elle ne compte pas) et le décompte par sévérité.
