# Story 36-15 — revue de code, passe 1 (trois lentilles B, E, A)

*Prompt versionné le 2026-11-16 par l'agent de développement (Opus 5.5). Passe complète : Sonnet ou Opus, jamais Haiku
(première passe). Une lentille par agent, en contexte frais.*

## Cadre commun aux trois lentilles

- **Worktree** : `/home/gcorbaz/devel/kesh-36-15`, branche `story/36-15-compte-analytique`. **Base de comparaison** :
  `git merge-base origin/epic/36-0.18.0 HEAD` (la branche part de `0bf3c3b4b` ; `origin/epic/36-0.18.0` a avancé depuis,
  un `git diff origin/epic/36-0.18.0` brut montrerait aussi ses six commits). Diff de la story :
  `git diff $(git merge-base origin/epic/36-0.18.0 HEAD)..HEAD`.
- **Fiche** : `_bmad-output/implementation-artifacts/36-15-compte-analytique.md` (D1 à D9, AC-1 à AC-11, Change Log,
  Dev Agent Record) ; registre `_bmad-output/implementation-artifacts/e36-choix-autonomes-36-15-compte-analytique.md`
  (C-36-15-1 à 7 ; **C-36-15-7 est neuf**, pris au développement : le type du nom de fichier exporté n'est plus coupé à
  20 caractères).
- **Arbitrage de Guy du 2026-11-16 (option A)** : de-CH « analytisches Konto », it-CH « conto analitico », en
  « analytical account », toujours en entier (« Konto », « conto », « account » seuls désignent le compte du plan).
- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `git commit|restore|checkout|stash`,
  ni `sed -i`, ni redirection `>` vers un fichier, ni fichier temporaire. Lire par `git show`, `git diff`, `git grep`,
  `/usr/bin/grep` (dans ce shell, `grep` est une fonction vers `ugrep`), `sed -n`, `awk` sans redirection, `pdftotext … -`.
- **Rapport** : la lentille écrit son seul rapport, par l'outil Write, dans
  `/home/gcorbaz/devel/kesh-gate-logs/e36-15/review-p1-<B|E|A>.md` (jamais sous `target/`). Chaque finding : sévérité
  (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, **sortie de commande copiée** pour toute affirmation de présence ou
  d'absence (`grep -nF`, `-F` obligatoire), correctif proposé. Le rapport **déclare les axes exercés et ceux qui ne
  l'ont pas été** ; un « 0 finding » sans cette liste ne compte pas.
- **Manuel** : `docs/manual/fr/user-manual.tex` (section `\section{Comptabilité analytique}` et les lignes de la fiche
  § 5), `admin-manual.tex` (`:2787`, `:3509`), `marketing-brochure.tex` (`:195`) se vérifient contre le code. **PDF** :
  les PDF commités ne sont pas régénérés par la story (D8, à l'intégration) ; un PDF compilé localement au développement
  est cité au Dev Agent Record. Seule forme permise du contrôle (E6), sans redirection :
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` ; l'existence par `ls -l`.
- **Captures** : `/home/gcorbaz/devel/kesh-gate-logs/e36-15/captures/` (liste au Dev Agent Record) — les regarder.
- **Gates** : `gate-ci.sh` et E2E complet n'ont pas tourné au développement pour l'E2E (ils tournent à la clôture) ; ce
  qui a tourné est au Dev Agent Record, avec journal et commit. Ne pas les relancer.
- **Chemins** (E9) : tout chemin cité se vérifie (`git cat-file -e HEAD:<chemin>` ou `ls`), ligne lue citée.

## Lentille B — Blind Hunter (diff seul)

Ne lis **que** le diff de la story (commande ci-dessus), sans la fiche. Cherche : un texte resté « projet » / « Projekt »
/ « progetto » / « project » visible de l'utilisateur ; un « Konto », « conto » ou « account » laissé seul là où il désigne
le compte analytique ; une phrase bancale (accord, article, genre — *das* Konto neutre, *il* conto masculin) ; une valeur
fr-CH et son repli frontend qui divergent ; une clé du catalogue renommée, ajoutée ou retirée ; un code, une route, un champ
JSON, un `data-testid` qui aurait changé ; la logique de `slugify_max` (bornes, caractère multioctet, tiret final) et ses
deux sites d'appel ; la garde G25 et son extracteur `litteraux_de_chaine` (états du lexème, numérotation des lignes,
commentaires `/* */`, chaînes brutes, caractères) ; la garde Vitest (regex `repliEnObjet`, `replisDuNomDeFichier`).

## Lentille E — Edge Case Hunter (diff + code lu)

Parcours chaque branche et chaque borne :
- `litteraux_de_chaine` : littéral non fermé en fin de fichier, `\` final, `r#` sans guillemet, durée de vie `'a`
  suivie d'un caractère, `'"'`, commentaire de bloc non fermé, `//` dans un littéral ; numéro de ligne après un littéral
  multiligne ; effet de `sans_modules_de_test` sur la numérotation (le module retiré décale-t-il les lignes citées ?).
- G25 : catalogue avec continuation (`  …` indenté) ou sélecteur Fluent ; motif anglais `\bprojects?\b` contre « Projected »
  ; une exception dont l'extrait apparaît aussi dans une ligne analytique (excuse-t-elle à tort ?) ; casse.
- `slugify_max` à `max = 0`, `max` plus grand que l'entrée, coupe sur un tiret ; les huit types × quatre langues tiennent-ils
  en 40 ? Un autre `reports-filename-*` change-t-il de nom (relevé `awk` de C-36-15-7) ? Le repli frontend
  (`reports/+page.svelte`, `kesh-${typeSlug}-…`) et le `Content-Disposition` du serveur donnent-ils le même nom ?
- Les refus de `/api/v1/projects` (`routes/projects.rs`) : chaque chemin rend-il le nouveau texte ? Un test les affirmait-il
  ailleurs (`git grep` sur l'ancien texte dans `crates/`, `frontend/`) ?
- Le couplage `reconciliation.rs:763` (`msg.contains("projet")`) : `crates/kesh-db/` est-il hors du diff ? Le texte interne
  `"le projet analytique est archivé"` est-il intact à l'octet (trois sites de production, deux de test) ?
- `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts:1020`) inchangé à 3050 ; `lint-i18n-ownership` sans changement de
  registre.

## Lentille A — Acceptance Auditor (fiche + diff + code)

Contrôle AC par AC (AC-1 à AC-11), avec la commande qui le prouve, et chaque décision D1 à D9 :
- AC-1 : les 49 clés × 4 portent les valeurs de D2 (relire la table **dans les trois langues** contre les catalogues, clé
  par clé) ; `git diff` des quatre `.ftl` ne touche que des lignes de valeur et le commentaire `fr-CH:3089` ; aucune clé
  ajoutée ou retirée.
- AC-2, AC-8 : 46 sites ; repli = valeur fr-CH.
- AC-3 : 11 + 4 littéraux ; nom de fichier (`export_filenames_say_compte_analytique`) ; en-têtes CSV inchangés (D5) ;
  **C-36-15-7** : la décision est-elle juste, bornée, dite à la note de version si elle doit l'être ?
- AC-4 : `git diff --stat` contre la base de fusion ne nomme ni `crates/kesh-db/`, ni migration, ni
  `routes/reconciliation.rs`, ni `kesh-report/src/csv.rs`, ni `i18n-keys.test.ts`.
- AC-5 : `frontend/tests/e2e/reports.spec.ts` ; rouge prouvé (`kesh-gate-logs/e36-15/rouges-e2e.log`).
- AC-6, AC-9, AC-11 : manuels, `api-external.md` (phrase de D9), glossaire (`:105`, ligne « projet » retirée, ligne
  « compte analytique », note annotée et nouvelle note, compte de la partie A), `README.md:47` ; comptes du PDF local.
- AC-7 : liste d'exceptions de G25 contre les 15 + 1 + 3 + 1 lignes au sens courant (fiche § 5, § 6) ; mutations
  journalisées dans `kesh-gate-logs/e36-15/mut/`.
- AC-10 : `git log origin/epic/36-0.18.0..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  vide ; `CHANGELOG.md` non touché ; la `## Note de version` de la fiche dit vrai (termes arbitrés, contrat de l'API).
- Dev Agent Record et Change Log : chaque chiffre cite son journal et son commit, et se recompte depuis la source.
