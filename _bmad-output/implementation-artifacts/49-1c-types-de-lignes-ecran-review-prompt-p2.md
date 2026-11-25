# Revue de code P2 — Story 49-1c « Types de lignes — l'écran, la ligne de commentaire » (#1242)

*Prompt de la passe 2 de `bmad-code-review`, passe **complète** (non ciblée : la remédiation de la P1 touche du code de
production), **un seul agent Sonnet 5.5** en contexte frais (la P1 était Opus : on alterne), qui exerce **les trois
lentilles — B (Blind Hunter), E (Edge Case Hunter), A (Acceptance Auditor) — l'une après l'autre**, sur **toute la story et
sur la remédiation**. Cette passe ne va pas à Haiku.*

## Ce que tu relis

- **Worktree** : `/home/gcorbaz/devel/kesh-49-1c`, branche `story/49-1c-types-de-lignes-ecran`, HEAD à relever par
  `git rev-parse --short HEAD` et à citer au rapport.
- **Toute la story** : `git -C /home/gcorbaz/devel/kesh-49-1c diff 5156f2b7b..HEAD -- crates frontend docs`
  (`5156f2b7b` = la 49-1a, la 49-1b et la 48-13 dans la branche).
- **La remédiation de la P1** : `git -C /home/gcorbaz/devel/kesh-49-1c diff 54471b566..HEAD -- crates frontend docs`
  (commit de code `c00858669` ; les commits suivants ne portent que la fiche, le registre et ce prompt).
- **Fiche** : `_bmad-output/implementation-artifacts/49-1c-types-de-lignes-ecran.md` (D1 à D12, AC1 à AC12, T1 à T12,
  Dev Agent Record, § *Gates de passage en `review`*, Change Log) ; registre :
  `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e491c.md` (C-e491c-1 à 17).
- **Rapports de la P1** (à lire d'abord, pour ne pas les refaire et pour juger leur remédiation) :
  `/home/gcorbaz/devel/kesh-gate-logs/e491c/review-p1-{B,E,A}.md` (0/0/1/10).
- **Journaux** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/` — dont `front-2.log` (frontend complet à
  `c00858669`), `e2e-T10-p1.log`, `test-fast-2.log` et `gate-ci-2.log` (Rust, `7f8bf6a00`, code inchangé depuis :
  contrôle-le par `git diff --stat 7f8bf6a00 HEAD -- crates`), `rouge-*.log`, `rejeu-tests-5156f2b7b.log`.
- **Captures** (AC11) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/captures/` — six PNG, à lire (outil de lecture d'image).
  L'écart « graine E2E, non base de démonstration » est déclaré à la fiche (C-e491c-15) : juge-le, ne le re-signale pas
  comme neuf.
- **Manuels compilés** (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/e491c/manuel/fr/user-manual.pdf` et
  `admin-manual.pdf` (copie hors dépôt, C-e491c-7).

## Le contexte qui commande la revue

Guy a déjà émis de **vraies factures**, remises sur papier à un locataire. **Rien ne doit changer pour une facture ou un avoir
existants** : lignes, totaux, TVA, écritures, numéros, PDF, et ce qu'un brouillon de lignes chiffrées envoie quand on le
rouvre et l'enregistre (AC8). La story **ouvre `comment`** à l'API, garde `heading` fermé, soustrait le taux neutre d'un
commentaire à la liste blanche des taux, et livre l'écran.

## Ce que la remédiation a changé (à relire avec soin : la sévérité se déplace vers ce qu'on vient d'écrire)

- `invoice-line-type.ts` : `estChiffree` accepte `lineType` `null` (`== null`) ; `estLigneDeTexte(t)` devient
  `!estChiffree({ lineType: t })`. Cherche : un appelant dont le comportement change (`null`, valeur inconnue) ; une valeur
  inconnue est désormais du texte partout (affichage, total écarté, envoi `{ lineType, description }`) — est-ce cohérent
  de bout en bout, y compris `ligneDepuisReponse` (`invoice-lines.ts`, `?? 'priced'`) et la réponse du serveur ? Les types
  élargis (`| null`) cassent-ils un site de `npm run check` ou un test ?
- `+page.svelte` des deux fiches : `colspan={COLONNES_LIGNES}` (constante 6) au lieu de `colspan="6"` ; les tests des
  fiches lisent toujours `'6'`. La constante est-elle juste (six `<th>` dans chaque `<thead>`) ?
- `invoices.types.ts` : doc-comment. `invoice-lines.test.ts` : deux cas neufs — prouvent-ils quelque chose (l'assertion
  « contraire exact » a-t-elle deux côtés indépendants ?) ?
- Fiche : section des gates, Change Log, « C-e491c-1 à 17 », recompte de `i18nMsg(`. Chaque chiffre a-t-il son journal et son
  commit ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ? (Le Rust est cité à `7f8bf6a00` sans rejeu : le code
  n'a-t-il vraiment pas bougé ?)

## Axes communs

1. **AC8, les factures existantes** : `git diff 5156f2b7b..HEAD` vide sur `crates/kesh-qrbill`, `kesh-core`, `kesh-report`,
   les migrations, `test-schema`, les dépôts de `kesh-db`, `invoice_pdf_service.rs`, `issued_invoice_pdf.rs`,
   `invoice_pdf.rs`, `credit_notes.rs`, `invoice_email.rs`, les deux dossiers `tests/fixtures/avant-0.20.0/` et les trois
   tests avant/après. Un brouillon chiffré-seulement rouvert envoie-t-il exactement ce qu'il envoyait, plus
   `lineType: 'priced'` ?
2. **La relecture d'un brouillon** : les deux sites (`initLines`, `reloadFromServer`) passent-ils par le seul
   `ligneDepuisReponse` ?
3. **Le total et les contrôles** : `computeInvoiceTotal`, `inactiveRates`, `_vatPlaceholder`, `invalidRevenueAccountLines`,
   `validateClient` et la charge filtrent-ils tous par `estChiffree` ?
4. **La liste blanche** (`taux_des_lignes_chiffrees`) au `POST` et au `PUT`.
5. **Le cycle d'imports** évité par `invoice-line-type.ts` (C-e491c-1).
6. **La garde G4-bis** (`7f8bf6a00`) : un faux vert possible (`U+` devant un vrai compte) ?
7. **Les LOW de la P1 laissés en issue** (B-1, E-1, A-4 : issues demandées, sans numéro tant que l'orchestrateur ne les a
   pas ouvertes) : le report est-il justifié, sans que le défaut soit plus grave qu'écrit ?

## Les trois lentilles, l'une après l'autre

- **B — Blind Hunter** : le diff seul, sans la fiche, d'abord ; logique, types TypeScript, gabarit Svelte, accessibilité,
  DRY, doc-comments faux, code mort, tests qui prouvent moins qu'ils ne disent.
- **E — Edge Case Hunter** : chaque branche et chaque borne : commentaire vide, blanc seul, sauts de ligne, mille
  caractères et plus, commentaire seul / commentaires seuls, première et dernière position, suppression de la dernière ligne
  chiffrée, ligne `heading` relue, valeur de type inconnue ou `null`, conflit de version, ligne sans `lineType`, saisie NFD,
  taux 0 % désactivé, avoir d'une facture à commentaire, PDF de l'avoir.
- **A — Acceptance Auditor** : AC1 à AC12 contre le code, un par un, avec la ligne lue citée ; T1 à T12 présents et à leur
  nom ; mutations jouées et tuées (`rouge-*.log`) ; D1 à D12 ; écarts déclarés (C-e491c-1 à 17) justes ; Dev Agent Record
  honnête, chaque chiffre contre son journal et son commit.

## Le manuel (obligatoire)

Vérifie le manuel contre le code, **PDF compris**, aplati avant de greper, par cette seule forme :
`pdftotext /home/gcorbaz/devel/kesh-gate-logs/e491c/manuel/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`
(de même pour `admin-manual.pdf` ; existence par `ls -l` ; l'apostrophe du PDF est typographique), et les `.tex` de la
branche. Points : `user-manual.tex` (item « Lignes de facture » : trois boutons, commentaire hors total/TVA/écriture, huit
lignes au plus, commentaires seuls non validables ; paragraphe des caractères contre `texte_de_commentaire`,
`crates/kesh-qrbill/src/pdf.rs` ; avoir), `admin-manual.tex` (« Pour les intégrations »), `docs/api-external.md`
(`comment` accepté, `heading` refusé). La remédiation ne touche pas le manuel : une phrase devenue fausse par elle ?

## Les captures (obligatoire)

Lis chaque PNG : bouton « Commentaire » à côté des deux autres ; champ sur toute la largeur sans taux, compte ni total ;
bandeau du commentaire vide qui nomme la ligne ; fiches (brouillon, validée, avoir) à pleine largeur, sauts de ligne gardés,
sans « 0.00 » ni « (défaut société) » ; PDF à gauche, sans colonnes. Le passage à `colspan={COLONNES_LIGNES}` ne change
aucun pixel : confirme-le par le code (valeur 6), non par une nouvelle capture.

## Interdits

Lecture seule. Aucune commande qui écrit, compile ou exécute : ni `cargo` (build, test, run, fmt, nextest), ni `npm`/`npx`
(vitest, playwright, check, build), ni `make`, ni `scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/check-dev-grants.sh`, `sqlx`, ni aucun script de
`kesh-gate-logs/e491c/` (`mut/mut-back-e491c.py`, `mut/mut-front-e491c.py`, `gate-cible-e491c.sh`, `e2e-e491c.sh`,
`e2e-T10-p1.sh`, `e2e-complet-e491c.sh`, `bases-neuves-e491c.sh` — il efface les bases de la story —,
`test-fast-ci-e491c.sh`, `gate-ci-e491c.sh`, `front-e491c.sh`), ni `pdftotext` vers un fichier (seulement vers `-`), ni
`git commit`, `git checkout`, `git restore`, `git stash`, `git worktree`, ni `rm`, `mv`, `cp`, `sed -i`, ni `docker`, ni
`mariadb`, ni redirection vers un fichier, ni fichier temporaire, aucune commande `gh` qui écrit. Seule écriture permise :
ton rapport. `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.

## Rapport

Dans `/home/gcorbaz/devel/kesh-gate-logs/e491c/review-p2.md` (une section par lentille). Chaque finding : sévérité
(CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la ligne lue citée, le scénario qui échoue, le remède proposé. Toute
affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par la liste des **axes exercés et
non exercés** pour **chacune** des trois lentilles (un « 0 finding » sans elle ne compte pas), et le décompte par sévérité.
**Arrêt de la boucle** : une passe sans finding au-dessus de LOW (CLAUDE.md § *Review Iteration Rule*) ; un MEDIUM corrigé
hors production appelle une passe de plus.
