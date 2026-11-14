# Prompt — revue de code P1 de la Story 36-3 (#1051), trois lentilles, première passe complète

Tu relis la **Story 36-3** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.8**,
Epic 36, phase 1 : **l'écriture manuelle et l'assistant TVA achat ne lisent plus « 1.234 » comme 1,234 franc en silence** —
un point ou une virgule suivis d'exactement trois chiffres, après une partie entière non groupée de un à trois chiffres, est
refusé comme **ambigu** (prédicat de la 35-6 extrait vers un utilitaire partagé) ; l'**apostrophe des milliers** est admise
(« 1'234,50 » part en `1234.50`) ; un montant mal formé est signalé **« Montant illisible »** (la clé « Maximum 4 décimales »
est retirée). **Le serveur ne change pas** ; seuls les catalogues `crates/kesh-i18n/locales/*/messages.ftl` sont touchés sous
`crates/`. **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-36-3`, branche `story/36-3-ecriture-montant-ambigu`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-3 diff a78115b69 <DERNIER_CODE>` (`a78115b69` = tête de la fiche validée, base de la
branche ; `<DERNIER_CODE>` = dernier commit de code, nommé au Dev Agent Record ; les commits suivants ne portent que des
comptes rendus). Langue du rapport : français. Modèle : **Sonnet ou Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-3-ecriture-montant-ambigu.md` en entier (D1 à D6, AC1 à AC11, mutations M1 à M11,
annotations « Étape 0 » qui traitent V1-1 à V1-8, **Change Log** et **Dev Agent Record**) ; les registres
`e36-choix-autonomes-e360.md` (C-e360-6, C-e360-11) et `e36-choix-autonomes-36-3.md` (C-36-3-1 à C-36-3-11) ; le rapport de
validation `/home/gcorbaz/devel/kesh-gate-logs/e36-3/validation-p1.md`. Ne prends pas le Dev Agent Record pour argent
comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3/` :
`etape0-inventaires.log`, `rouges.log`, `e2e-rouges.log`, `vitest-cible-2.log`, `e2e-cible-vert.log`, `mut/resume.log` et
`mut/M*.log` (script `mut/mut-36-3.py`), `test-fast-ci-review.log`, `gate-ci-review.log`, `front-*.log`.
**Captures** (thème clair, AC11) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3/captures/` (`01-ecriture-1.234-ambigu.png`,
`02-ecriture-12-34-illisible.png`, `03-ecriture-1234-50-equilibree.png`, `04-assistant-ht-1.234-ambigu.png`) — regarde-les si
ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont
  `mut/mut-36-3.py`, `e2e-36-3.sh`, `captures-36-3.cjs`) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`,
  `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `cat -A`, `od -c`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  et, pour un PDF, la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. Pour
  un classement de saisie, **déroule la regex et le prédicat à la main** contre le code lu ligne à ligne.
- **Tout chemin cité** se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`), la ligne lue citée.

## Les axes obligatoires, pour chaque lentille

1. **Le prédicat partagé (D1)** — `frontend/src/lib/shared/utils/amount-ambiguity.ts` : équivalent à l'expression d'origine
   de `format.ts` (`hasDecimal && typedFraction.length === 3 && !grouped && /^0*[1-9][0-9]{0,2}$/.test(integerDigits)`) ?
   L'appel dans `openingAmountKind` passe-t-il `typedFraction` (et non la fraction rognée) ? `format.test.ts` et
   `opening-amount-cases.json` **inchangés** (`git diff a78115b69 <DERNIER_CODE> -- frontend/src/lib/features/bank-accounts/`
   ne doit nommer que `format.ts`) ; le test de parité Rust relit la même table.
2. **La grammaire du journal (D2, C-36-3-2, -7, -8)** — `readJournalAmount` (`balance.ts`) : regex
   `^([1-9]\d{0,2}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$`, garde des quinze chiffres, `grouped`, charge. Une saisie
   claire part-elle **à la valeur que l'utilisateur voit** ? Une forme refusée par le serveur peut-elle passer `clear` ? Une
   saisie d'aujourd'hui acceptée à une valeur l'est-elle à une autre (hors « 1.234 » et sœurs, voulues) ? `\d` sans drapeau
   `u` : chiffres non ASCII ? `$` et un retour à la ligne final ?
3. **Le lecteur facultatif (D3)** — `computeBalance` et `classifyLine` à défaut `defaultAmountReader` : **strictement** la
   lecture d'avant pour les soldes de départ (`settings/opening-balances/+page.svelte:135`, #1050) ? `lines.map(classifyLine)`
   ne subsiste nulle part (`git grep -n "map(classifyLine" -- frontend/src`) ? Un appelant non vu ?
4. **L'écran (D4)** — `JournalEntryForm.svelte` : message ambigu prioritaire sur illisible, bordure rouge d'un champ ni vide
   ni clair, charge par `journalAmountPayload`, garde `null` dans `handleSubmit`, *Ctrl+S* par la même garde ; une écriture
   rouverte (montants à quatre décimales du serveur) jamais refusée. État Svelte (`$derived`) qui resterait périmé ?
5. **L'assistant (D5, C-36-3-9)** — `readHtAmount`, `purchaseVatEntry`, `insert()` : aucune ligne tirée du HT brut ;
   règle du centime inchangée ; `lineVatAmount` et `buildPurchaseVatLines` restent partagés avec
   `supplier-invoice-form.ts:11` — leur comportement change-t-il ? Le commentaire d'en-tête de `vat-purchase.ts` dit-il vrai
   (« 100.005 » refusé à l'écran) ?
6. **Textes (AC9)** — deux clés × quatre locales à la place de `journal-entry-form-max-decimals` (même endroit, jamais en fin
   de fichier) ; replis littéraux égaux au fr-CH ; traductions qui disent la même règle ; `lint-i18n-ownership.js` ;
   `sitesTotal` 3036 → 3038 (`i18n-keys.test.ts`) ; plus aucune occurrence de `max-decimals` ni de « Maximum 4 décimales »
   hors `_bmad-output/` (`git grep -n "max-decimals\|Maximum 4 d" -- . ':!_bmad-output'`).
7. **Le manuel contre le code (AC10)** — `docs/manual/fr/user-manual.tex`, item **Montant** du § *Saisie d'une écriture en
   partie double* : chaque phrase est-elle vraie au code (espace refusée, groupes de trois, règle d'ambiguïté, « Montant
   illisible », assistant) ? **Le PDF n'est pas régénéré dans la branche** (C-36-3-6, à l'intégration) : vérifie-le
   (`git diff --stat a78115b69 <DERNIER_CODE> -- docs/`) et dis la sonde à jouer à l'intégration :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'apostrophe des milliers'`
   (attendu ≥ 1 après régénération ; 0 aujourd'hui).
8. **Les témoins et les mutations** — chaque AC a-t-il un test qui peut rougir ? M1 à M11 et M10b jouées et tuées
   (`mut/resume.log`) ; M3 n'est tuée que par AC1 et la table 35-6 (C-36-3-7 : aucune saisie du journal ne la tue) ; les
   tests du formulaire choisissent-ils un compte (V1-4), et le cas témoin « 1.23 » prouve-t-il *Valider* actif ? Une
   assertion sur `className` qui passerait à vide (la classe de base contient `aria-invalid:border-destructive`) ? Aucun
   attendu de test existant retouché sans motif (`git diff a78115b69 <DERNIER_CODE> -- '*.test.ts' '*.spec.ts'`, lignes `-`) ?
   Propose des mutations plausibles non jouées.
9. **Le périmètre (AC6)** — `git diff --stat a78115b69 <DERNIER_CODE>` ne nomme ni `bank-import/`, ni `opening-balances/`,
   ni `settings/opening-balances/`, ni `supplier-invoices/`, ni `crates/` hors `kesh-i18n/locales/`. Les sites hors
   périmètre de l'inventaire par le symptôme (fiche, lignes 4 à 10) : la décision « une issue par site » est-elle tenue au
   rapport de développement ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `amount-ambiguity.ts`, le diff de `format.ts`, `balance.ts`, `vat-purchase.ts`, les deux composants, les
quatre catalogues, le lint et les tests. Cherche : une regex qui laisse passer ou refuse plus qu'elle ne dit ; une valeur
envoyée différente de la valeur lue ; un `$derived` qui garde un état périmé ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Déroule au code : `0.500`, `000.123`, `0001.234`, `1.000`, `1000.000`, `100.250`, `1'234,567`, `1'234'567`, `1’234,5`,
`12'345'678'901'234`, `123'456'789'012'345`, `1'234'567'890'123'456`, `0'123`, `01'234`, `1234'567`, `1'23'456`,
`1 234,50` (espace, U+00A0, U+202F), `1‘234` (U+2018), `1´234` (U+00B4), `.5`, `5.`, `1.23450`, `1.23456`, `1,2340`,
`١٢٣`, `１２３` (pleine chasse), une saisie suivie d'un retour à la ligne, une saisie avec blanc de tête. Et à l'assistant :
`0.005`, `0.004`, `100.005`, `1'000`. Chaque cas : issue au code, issue attendue (fiche), test qui le couvre (ou absence,
prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC11 : livré, testé, au lieu prescrit ? Les rouges de départ (`rouges.log`, deux passes : la seconde
après le constat V1-6) ; `e2e-rouges.log` ; les verts au départ dits tels ; les mutations ; les gates au dernier commit de
code (chiffres du Dev Agent Record, chacun retrouvé au journal) ; les écarts déclarés. Le Dev Agent Record n'affirme-t-il que
ce qui a tourné (E2E complet **non lancé** : il tourne à la clôture) ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-3/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM /
LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non
exercés** ; le décompte final **recompté depuis la liste**.
