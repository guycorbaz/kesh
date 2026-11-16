# Prompt — revue de code P1 de la Story 35-6 (#1026, #1028, #1025), trois lentilles, première passe complète

Tu relis la **Story 35-6** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **la saisie du solde d'ouverture d'un compte bancaire est classée à l'écran avant tout envoi** —
une saisie ambiguë (« 1.234 » : mille ou un virgule deux ?) est refusée avec la forme à écrire, une saisie non lisible ou
négative aussi, les saisies claires passent avec la valeur d'aujourd'hui, l'aperçu et l'avis de succès montrent la charge
envoyée (`1_000` → « 1 000.00 CHF »). **Le serveur ne change pas** (un test de parité seul, dans le `mod tests` de
`opening_balances.rs`). **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-e356`, branche `story/35-6-solde-ouverture-saisie`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e356 diff 1ebe1240 d64520db` (`1ebe1240` = base `epic-35/base2` ; `d64520db` = dernier commit
de code ; `fe1ca570` ne porte que le compte rendu). Langue du rapport : français. Modèle : **Opus** (première passe : jamais
Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-6-solde-ouverture-saisie.md` en entier (le principe C-e350-19, D1 à D7 dont la
**table D3** et la **règle D4**, AC1 à AC8, tests et mutations M1 à M19, *Ce qui doit rester vrai*, **Change Log** et **Dev
Agent Record**) ; les registres `e35-choix-autonomes-e350.md` (C-e350-8, -15, -17, -18, -19, -30 à -35) et
`e35-choix-autonomes-e356.md` (C-e356-1 à 6). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e356/` : `rouges.log`, `rouges-brut.log`,
`verts-cible.log`, `test-fast-ci.log`, `gate-ci-journal.log`, `gate-ci.out`, `front-*.log`, `e2e-r1.log`, `mutations.log`,
`make-fr.log`. **Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e356/captures/` — regarde-les si ton outil
lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
  aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `cat -A` (pour voir les espaces spéciales), `od -c`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport. Pour un
  classement de saisie, **déroule le pipeline à la main**, étape par étape (L1 à L7), contre le code de `format.ts` lu ligne à ligne.

## Les axes obligatoires, pour chaque lentille

1. **Le pipeline contre D2** (`openingAmountKind`, `format.ts`) : l'ordre L1 → L7, « la première qui conclut l'emporte » ; L1
   rogne blancs Unicode **et** apostrophes ; L2 re-rogne après le signe ; L3 souligné avant le premier chiffre ; L4 plus d'un
   `.`/`,` ; L5 fraction en `[0-9]` seuls ; L6 « groupée » = séparateur **entre deux chiffres** ; L7 dans l'ordre (plus de quatre
   décimales significatives > négatif non nul > ambiguë > clair). Le motif de D4 `^0*[1-9][0-9]{0,2}[.,][0-9]{3}$` : zéro seul
   exclu, zéros de tête non comptés. Une regex JavaScript `\s` couvre-t-elle bien U+00A0, U+202F, U+2009, tabulation, et rien de
   ce qui devrait être refusé (U+200B ? retour à la ligne ?) ? Les caractères spéciaux du code sont-ils écrits par échappement
   (lisibles, stables à l'édition) ou bruts — le Dev Agent Record déclare des scripts de réécriture à cause d'espaces brutes :
   vérifie `format.ts:237`, `:242` à `cat -A`.
2. **La charge (D5)** : toute charge claire suit `^[0-9]+(\.[0-9]{1,4})?$` ; signe `+` ôté, `-0` → 0, zéros de fin au-delà de
   la quatrième décimale retirés ; **même valeur qu'aujourd'hui** pour toute saisie claire (C-e350-19) — compare à
   `openingAmountPayload` de la base (`git show 1ebe1240:frontend/src/lib/features/bank-accounts/format.ts`).
3. **La table D3 et la fixture** : chaque ligne de `opening-amount-cases.json` est-elle celle de la fiche (issue **et** charge) ?
   Les deux lignes ajoutées pour M8 (`1 2.345`, `12'3.456`) : vraies au pipeline, et acceptées aujourd'hui à la même valeur ?
   Une ligne de la fiche manque-t-elle à la fixture, ou l'inverse ? Les échappements JSON sont-ils ceux d'AC1 ?
4. **Les deux formulaires** (`bank-accounts/+page.svelte`) : refus sans appel au serveur, `formError` à la création,
   `openingError` au compte existant ; classement joué à la création **seulement si un compte lié est choisi** ; compte lié et
   montant vide → aucun `openingBalance` envoyé. L'aperçu et l'avis de succès formatent la charge, jamais « NaN ». Le Vitest de
   la page prouve-t-il chaque branche, ou passe-t-il à vide (assertion sur un mock jamais appelé, `expect` dans un `if`) ?
5. **i18n** : la clé d'ambiguïté réécrite et `bank-accounts-opening-error-amount-unreadable`, ×4 locales, placées à côté des
   `bank-accounts-opening-*` ; replis littéraux égaux au fr-CH ; aucune clé à sélecteur ; `sitesTotal` 3026 → 3027 recompté
   (`i18n-keys.test.ts`) ; les traductions disent-elles la même règle (exemple, forme à écrire) ? Le message négatif : quelle clé ?
6. **La parité écran/serveur (AC8)** : le test lit la fixture à l'exécution (`CARGO_MANIFEST_DIR`) ; les cinq assertions (a) à
   (e) et leurs nombres affirmés en dur (58, 1, 24, 13, 4) ; l'alphabet de (c)/(d) `[0-9._+-]` après `str::trim` ; (e) épingle
   `1e3`, `1E3`, `5e-1`, la ligne E. Un filtre de ligne trop large ou trop étroit rendrait-il une assertion vide sans rougir ?
   **Aucune ligne de production du serveur ne change** : vérifie-le par `git diff 1ebe1240 d64520db -- crates/ | /usr/bin/grep -n '^[+-]'`
   hors `mod tests`.
7. **Les manuels contre le code** (`user-manual.tex:2456` et voisins), **le PDF aplati** (sondes d'AC6 : absences de
   « (#1026) », « (#1025) », « toujours lu ainsi », « écrivez-les avec un point » ; présences de « 1.2340 », « 1000.000 ») ; le
   CHANGELOG `## [0.17.7]` *Corrigé*, un point par issue, la règle de D4 (familles A à E) et non une liste, famille D « refusée
   à l'écran, toujours acceptée par l'API ». Chaque phrase est-elle vraie au code ? `docs/api-external.md` inchangé ?
8. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`mutations.log`) ? M1 à M19 et M16-bis jouées et
   tuées, M4/M6 sur les deux formulaires, M17 côté Vitest et Rust ? Aucun attendu de test existant retouché sans motif
   (`git diff 1ebe1240 d64520db -- '*.test.ts'`, lignes `-`) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `openingAmountKind`, `openingAmountPayload`, `openingAmountPreview`, les deux soumissions de la page, les
messages, le test de parité. Cherche : une regex qui laisse passer ou refuse plus qu'elle ne dit ; une saisie claire
d'aujourd'hui dont la valeur change ; un état Svelte (`$state`, `$derived`) qui garde une erreur périmée après correction de la
saisie ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Déroule au code, étape par étape : `0.500`, `0,000`, `000.123`, `0001.234`, `1.000`, `1000.000`, `100.250`, `0.999`, `1'234,567`,
`1 234,567`, `1,234.56`, `-0`, `-0.000`, `+ 1,234`, `'+5`, `1.5'`, `1_`, `1_.5`, `_1`, `1__0`, `.5`, `5.`, `.`, `1.23450`,
`1.23456`, `1e3`, `١٢٣` (chiffres arabes : `[0-9]` ou `\d` ?), une saisie avec U+200B, une avec un retour à la ligne, une saisie
très longue (seize chiffres et plus), U+FF10 (chiffre pleine chasse). Chaque cas : issue au code, issue attendue (fiche ou
principe C-e350-19), ligne de la fixture qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC8 : livré, testé, au lieu prescrit ? Les rouges de départ (`rouges.log` : 96 sur 229) ; les verts au
départ dits tels ; les mutations nommées jouées et tuées ; les gates au dernier commit de code `d64520db` (journaux : test-fast
5308, gate-ci 5315, Vitest 3325, E2E 403/7 KF-029) — chaque chiffre se retrouve-t-il au journal ? Les écarts déclarés (fiche
corrigée sur M8 ; scripts Python et marqueur ASCII contre la consigne Edit ; PDF d'administration et de brochure non commités,
C-e356-5) sont-ils exacts et sans effet sur le livré ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e356-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque rapport :
findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou
preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
