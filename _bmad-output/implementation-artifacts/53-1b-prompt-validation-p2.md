# Prompt — validation P2 de la fiche 53-1b (passe complète, Opus, contexte frais)

Tu es **Opus**, en **contexte frais**. Tu conduis la **passe P2 (complète)** de la validation de la fiche de la story **53-1b** de Kesh (« la conversion de devises ne passe pas deux fois », #1365, P1). La P1 était **Opus** : la règle d'alternance du `CLAUDE.md` (§ *Review Iteration Rule*) veut un autre modèle que celui de la remédiation (Sonnet) **et** une passe complète ; Opus convient puisque la remédiation est de Sonnet. Une première passe n'est jamais ciblée ; celle-ci est complète.

## Matériel

- Fiche : `/home/gcorbaz/devel/kesh-53-1b/_bmad-output/implementation-artifacts/53-1b-conversion-pas-deux-fois.md` (lis-la en entier, Change Log compris).
- Rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e531b/validation-p1.md` (0 CRITICAL, 0 HIGH, 5 MEDIUM V1-1 à V1-5, 11 LOW V1-6 à V1-16, questions Q1 à Q3).
- Code : worktree `/home/gcorbaz/devel/kesh-53-1b`, branche `story/53-1b-conversion-pas-deux-fois`, partie de `epic/53-ecritures-justes` `2131261f7` (53-1a et 53-3 livrées) ; la remédiation de la P1 est le commit de tête.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` (§ *Review Iteration Rule*, *Règle de splitting préventif*, *Pattern batch*, *Priorités des défauts*, *Migration breaking policy*, *Commits, branches et push*), `docs/agents-orchestration.md`, la fiche et le Dev Agent Record de la 53-1a (`53-1a-saisie-pas-deux-fois.md`).
- Décisions du Project Lead à respecter, non à rouvrir : Q5 (⚠️ en tête des notes, aucune migration), Q7 (une seule écriture identique suffit à refuser la conversion). **Q2 (nommer la colonne) et Q3 (écart 400/422)** sont **ouvertes** : ne les tranche pas ; contrôle seulement que la fiche **tient sous chaque réponse**, comme elle l'affirme.

## Interdits (commandes nommées)

Lecture du code seulement : **aucune compilation, aucun test** (`cargo build|test|nextest|clippy|fmt`, `npm run …`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`), **aucune écriture** dans le dépôt (`Edit`, `Write`, `sed -i`, `git add|commit|push|checkout|reset|stash|merge|rebase`), **aucun script mutant** (`scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/seed-dev-db.sql`, `sqlx migrate run`), **aucun `docker compose`** ni client MariaDB, **aucune commande `gh` en écriture** (`gh issue create|edit|close|comment`, `gh pr create|edit|merge|comment`, `gh api -X POST|PATCH|PUT|DELETE`), **jamais `pkill`, `killall` ni `kill`**. Seule écriture permise : ton rapport, `/home/gcorbaz/devel/kesh-gate-logs/e531b/validation-p2.md`, hors du dépôt. Chemins absolus.

## Axes — à déclarer exercés ou non exercés (un « 0 finding » sans cette liste ne compte pas)

**Axe 0 — chaque finding de la P1 réellement remédié.** Pour chacun de V1-1 à V1-16 (et la prise en compte de Q1, Q2, Q3), relis **la fiche et le code**, non le Change Log : cite la ligne de la fiche qui remédie, vérifie qu'elle dit ce que le Change Log affirme, et que le texte nouveau **ne crée pas** un défaut (la sévérité se déplace vers ce qu'on vient d'écrire). Points d'attention : V1-1 (D2.2-D2.4 : la boucle juge-t-elle bien `NoFiscalYear` puis `FiscalYearClosed` avant la création ? la preuve « aucun `UPDATE` de `start_date`/`end_date` » est-elle vraie ? `grep -nF "UPDATE fiscal_years" -r /home/gcorbaz/devel/kesh-53-1b/crates/*/src/*`) ; V1-2 (la table K est-elle complète, le rangement K→J cohérent avec `entry-submit-state.ts`, AC-9 et M9 à M11 prouvent-ils le cas permis ?) ; V1-3 (la mesure (a)/(b) et M2b sont-elles écrites sans contradiction, et laquelle décide est-il clair ? « arrêt si (a) positif » est-il cohérent avec le reste ?) ; V1-4 (chaque ligne citée de `api-external.md`, du manuel, de l'admin existe-t-elle ? `:324`, `:234`, `:1467`, `:1273-1276`, `user-manual.tex:1362`, `admin-manual.tex:2733`) ; V1-5 (AC-7 Playwright, décor de `fx-entries.spec.ts` réellement réutilisable ?).

**Axe 1 — prémisses contre le code livré** : relis chaque citation de la fiche (`sed -n`, `grep -nF`), y compris celles que la remédiation vient de recaler ; un grep vide sur un symptôme qu'on vient de corriger éveille le soupçon (`grep -nE "1217|:1333|4d540a6ea"` sur la fiche).

**Axe 2 — inventaire clos des sites qui écrivent une conversion** (et des sites NON RÉSOLUS) : refais le relevé par toi-même (appelants de `create_conversion`, `create_conversion_in_tx`, `create_provider_conversion_in_tx`, `createConversion`, `kesh-seed`, tests HTTP, specs Playwright) ; ne reprends pas l'extracteur de la fiche.

**Axe 3 — mécanisme et concurrence** : l'ordre des verrous de D2/D4 fonction par fonction ; cycles possibles avec la clôture, l'activation du multi-devises, la saisie à politique, une autre conversion, le flux de prestataire (`None`/`None`) ; la précédence des refus avant et après ; la prémisse de l'instantané (MariaDB 10.11, `LOCK IN SHARE MODE`, `EXISTS`) : ce qui est mesuré, ce qui est supposé.

**Axe 4 — AC, mutations, preuve du rouge** : chaque AC est-il prouvé par un test qui rougirait sous sa mutation ? mutations équivalentes ? tests verts par construction non couverts ? collisions de tests existants (`journal_entries_fx_ecrans_e2e.rs`, `fx_recette_api_e2e.rs`, `fx-entries.spec.ts`, Vitest) à relever toi-même.

**Axe 5 — contrat de l'API** : extracteur `ConversionBody` (400) contre `Json` (422), `conversion_refusal`/`refus_avec_detail`, `details.entry`, tableau des routes et des codes ; aucune migration ; version et ⚠️.

**Axe 6 — manuel contre le code, PDF compris.** Ce prompt porte sur du code visible de l'utilisateur : vérifie `docs/manual/fr/user-manual.tex` **et** son PDF **aplati avant de greper** : `pdftotext /home/gcorbaz/devel/kesh-53-1b/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`, et `marketing-brochure.pdf` si pertinent) ; `grep` sur le texte aplati, non sur le `.tex` seul. Contrôle ce que la fiche dit du manuel (section « Convertir d'une devise à une autre », « Si la réponse se perd », refus du doublon, `Passer à la 0.21.0` à créer à la release) contre ce que le code fera ; signale les écarts DE/IT/EN seulement s'ils contredisent une règle du `CLAUDE.md`.

**Axe 7 — taille, splitting, borne de l'epic** : modules touchés (≤ 5), borne de six stories de l'epic 53 (`sprint-status.yaml`, fichier d'epic), recyclage des défauts (P1 → P2 : les défauts reviennent-ils sous une autre forme ? à déclarer au Project Lead si oui, § *Règle de splitting préventif*).

**Axe 8 — recoupements** : 53-1a, 53-3, PR #1595 (lecture `gh pr view|diff`, sans écriture), autres epics ouverts qui touchent `journal_entry_conversions.rs`, `ConversionAssistant.svelte`, les catalogues i18n, `api-external.md`.

**Axe 9 — cohérence interne de la fiche** : Q2/Q3 « la fiche tient sous chaque réponse » est-il vrai ligne à ligne ? numérotation des AC (AC-9 placé après AC-8 ?), mutations citées partout, tâches alignées, Note de version, Dev Agent Record vide, Change Log lisible, aucune contradiction entre D1, D2, D5 et les AC ; texte de K5 (sélecteur Fluent, quatre locales) et clés de K toutes nommées.

## Règles de la passe

- Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code ou un anti-pattern : vérifie par `grep -nF` (`-F` obligatoire) ou lecture directe avant de l'écrire, et cite l'extrait.
- Tout décompte écrit dans ton rapport se recompte depuis la source, avec son périmètre.
- Classe chaque finding CRITICAL / HIGH / MEDIUM / LOW, numérote-les `V2-n`, cite la ligne de la fiche et celle du code, propose la correction.
- Rapport : décompte, findings, points vérifiés et tenus, **tableau des axes exercés / non exercés avec le comment**, ce qui n'a pas pu être exercé et pourquoi, questions au Project Lead. Réponse finale : le décompte et le chemin du rapport.
