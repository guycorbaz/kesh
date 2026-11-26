# Prompt — validation P3 de la fiche 53-1b (passe complète, Sonnet, contexte frais)

Tu es **Sonnet**, en **contexte frais**. Tu conduis la **passe P3 (complète)** de la validation de la fiche de la story **53-1b** de Kesh (« la conversion de devises ne passe pas deux fois », #1365, P1). La P2 était **Opus** : la règle d'alternance du `CLAUDE.md` (§ *Review Iteration Rule*) veut un autre modèle. La passe est **complète, non ciblée** : la remédiation de la P2 touche plusieurs AC (AC-1, AC-4, AC-7, AC-8, AC-9), D1, D2.4, D4, D5, les mutations, les tâches et les questions.

## Matériel

- Fiche : `/home/gcorbaz/devel/kesh-53-1b/_bmad-output/implementation-artifacts/53-1b-conversion-pas-deux-fois.md` (lis-la en entier, Change Log et section « Dérogation règle de splitting » compris).
- Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e531b/validation-p1.md` (0/0/5/11) et `validation-p2.md` (0/0/4/11 : V2-1 à V2-15).
- Code : worktree `/home/gcorbaz/devel/kesh-53-1b`, branche `story/53-1b-conversion-pas-deux-fois`, partie de `epic/53-ecritures-justes` `2131261f7` ; la remédiation de la P2 est le commit de tête.
- Lis `/home/gcorbaz/devel/kesh/CLAUDE.md` (§ *Review Iteration Rule*, *Règle de splitting préventif*, *Pattern batch*, *Priorités des défauts*, *Migration breaking policy*) et la fiche de la 53-1a (`53-1a-saisie-pas-deux-fois.md`, table J, AC-11, Dev Agent Record).
- **Décisions du Project Lead à respecter, non à rouvrir** : Q5 (⚠️ en tête des notes, aucune migration), Q7 (une seule écriture identique suffit à refuser la conversion), et **la décision du 2026-11-26 de remédier la P2 sans découper** (déclarée à la fiche). **Q2 (nommer la colonne) et Q3 (écart 400/422) restent ouvertes** : ne les tranche pas ; contrôle que la fiche **tient sous chaque réponse**, ligne à ligne.

## Interdits (commandes nommées)

Lecture du code seulement : **aucune compilation, aucun test** (`cargo build|test|nextest|clippy|fmt`, `npm run …`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`), **aucune écriture** dans le dépôt (`Edit`, `Write`, `sed -i`, `git add|commit|push|checkout|reset|stash|merge|rebase`), **aucun script mutant** (`scripts/prepare-release.sh`, `scripts/recette-montee.sh`, `scripts/seed-dev-db.sql`, `sqlx migrate run`), **aucun `docker compose`** ni client MariaDB, **aucune commande `gh` en écriture** (`gh issue create|edit|close|comment`, `gh pr create|edit|merge|comment`, `gh api -X POST|PATCH|PUT|DELETE`), **jamais `pkill`, `killall` ni `kill`**. Seule écriture permise : ton rapport, `/home/gcorbaz/devel/kesh-gate-logs/e531b/validation-p3.md`, hors du dépôt. Chemins absolus.

## Axes — à déclarer exercés ou non exercés (un « 0 finding » sans cette liste ne compte pas)

**Axe 0 — chaque finding de la P2 réellement remédié, sans défaut neuf né de la correction.** Pour chacun de V2-1 à V2-15 (et Q2, Q3), relis **la fiche et le code**, non le Change Log : cite la ligne de la fiche qui remédie, vérifie qu'elle dit ce que le Change Log affirme, et que le texte nouveau **ne crée pas** un défaut (la sévérité se déplace vers ce qu'on vient d'écrire ; les quatre MEDIUM de la P2 sont nés de la remédiation de la P1). Points d'attention, à **refaire contre le mécanisme réel** :
- **V2-1** : K5 par deux clés plates `-debit`/`-credit`. Vérifie `crates/kesh-i18n/src/loader.rs:158` (`format_pattern(pattern, None, …)`), la garde `SELECTEURS_RESOLUS_COTE_SERVEUR` (`:383`), `frontend/src/lib/shared/utils/i18n.svelte.ts:27`, `frontend/scripts/lint-i18n-ownership.js` (préfixe `journal-entries-`) ; qu'aucun reste de « sélecteur Fluent » ni de `$entry` ne subsiste dans la fiche (`grep -nF` sur `\$entry`, `[1]`, `sélecteur`) ; que le décompte des clés de K (AC-8, `sitesTotal`) est cohérent.
- **V2-2** : le libellé est calculé par l'assistant (`ConversionAssistant.svelte:181`, `conversion.ts:316`) ; AC-7 distingue par date ou montant ; le manuel ne conseille pas le libellé pour la conversion ; `fx-entries.spec.ts:45`, `:335-347`.
- **V2-3** : le motif commun `["FROM fiscal_years", "FOR UPDATE"]` : reconnaît-il bien le pré-verrou (`fiscal_years.rs`/dépôt de la conversion à écrire) **et** l'étape 1 (`journal_entries.rs:578-580`) ? `test_fixtures.rs:600-640` (`LIKE`, dix secondes) ; **le verdict de chaque mutation (M2b, « pré-verrou retiré ») est-il correctement décrit** ? La limite relevée (le motif ne distingue pas F1 de F2 ; témoin `INNODB_TRX`) est-elle exacte, et ne contredit-elle pas « échec par le verdict » ?
- **V2-4** : AC-9 (c), M11 équivalente déclarée, comparaison avec `53-1a-saisie-pas-deux-fois.md:138` et `:156` ; la lecture du câblage tient-elle comme preuve ? l'inatteignabilité par l'interface est-elle exacte (`ConversionAssistant.svelte:101-115`, `:254-255`) ?
- **V2-5 à V2-15** : un à un (citation `user-manual.tex:1369`, `grep -rnF`, ordre AC-8/AC-9, « identique », Q2/Q3, sites frontend de D4, `needsEntry` dans `stateAfterRejection` — `entry-submit-state.ts:121-122` — est-il cohérent avec K2b et K7 et ne change-t-il pas la saisie ? preuve du rouge, `services_motif.rs:2435-2448`, `sprint-status.yaml:1257`, recoupements Epics 47 et 51).

**Axe 1 — prémisses contre le code livré** : relis chaque citation de la fiche (`sed -n`, `grep -nF`), y compris celles que la remédiation vient d'écrire (numéros de ligne de `journal_entries.rs:3867`, `:5117`, `accounts.rs:3023`, `:4671`, `:3263`, `invoices.rs:4506`, etc.) ; un grep vide sur un symptôme qu'on vient de corriger éveille le soupçon.

**Axe 2 — inventaire clos des sites qui écrivent une conversion** (et des sites NON RÉSOLUS) : refais le relevé toi-même (appelants de `create_conversion`, `create_conversion_in_tx`, `create_provider_conversion_in_tx`, `createConversion`, `kesh-seed`, tests HTTP, specs Playwright) ; ne reprends pas l'extracteur de la fiche. Inventorie aussi les **sites de texte** que la remédiation de V2-10 prétend complets (types, en-têtes, doc-comments) par `grep` de `confirmNotDuplicate` et de « 53-1b ».

**Axe 3 — mécanisme et concurrence** : l'ordre des verrous de D2/D4 fonction par fonction (aucune des corrections de la P2 n'est censée le toucher : vérifie-le) ; cycles possibles ; précédence des refus ; ce qui est mesuré, ce qui est supposé (MariaDB 10.11).

**Axe 4 — AC, mutations, preuve du rouge** : chaque AC est-il prouvé par un test qui rougirait sous sa mutation ? mutations équivalentes (M11 déclarée) ? la liste des mutations est-elle cohérente avec les AC et les tâches (F3/F4) ? collisions de tests existants à relever toi-même.

**Axe 5 — contrat de l'API** : extracteur `ConversionBody` (400) contre `Json` (422), `conversion_refusal`/`refus_avec_detail`, `details.entry`, tableaux des routes et des codes ; Q3 sous chaque réponse ; aucune migration ; version et ⚠️.

**Axe 6 — manuel contre le code, PDF compris.** Ce prompt porte sur du code visible de l'utilisateur : vérifie `docs/manual/fr/user-manual.tex` **et** son PDF **aplati avant de greper** : `pdftotext /home/gcorbaz/devel/kesh-53-1b/docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (idem `admin-manual.pdf`, et `marketing-brochure.pdf` si pertinent) ; `grep` sur le texte aplati, non sur le `.tex` seul. Contrôle ce que la fiche prévoit du manuel (section « Convertir d'une devise à une autre », « Si la réponse se perd », refus du doublon et libellé calculé, `Passer à la 0.21.0` à créer à la release) contre ce que le code fera ; signale les écarts DE/IT/EN seulement s'ils contredisent une règle du `CLAUDE.md`.

**Axe 7 — taille, splitting, recyclage** : modules touchés (≤ 5), borne de six stories (`sprint-status.yaml`, fichier d'epic) ; **les défauts de la P2 reviennent-ils sous une autre forme à la P3 ?** Si oui, à déclarer au Project Lead (§ *Règle de splitting préventif*), avec le constat ; la décision du 2026-11-26 ne vaut que pour la P2.

**Axe 8 — recoupements** : 53-1a, 53-3, PR #1595 (lecture `gh pr view|diff`, sans écriture), Epics 47 et 51, autres branches touchant `journal_entry_conversions.rs`, `ConversionAssistant.svelte`, catalogues i18n, `api-external.md`.

**Axe 9 — cohérence interne** : D1, D2, D5 et AC sans contradiction ; AC numérotés dans l'ordre ; mutations citées partout ; tâches alignées ; Note de version ; Dev Agent Record vide ; Change Log « validation P2 remédiée » lisible et exact ; section « Dérogation règle de splitting » conforme à `CLAUDE.md` ; statut (ligne 3) et `sprint-status.yaml:1257` alignés.

## Règles de la passe

- Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code ou un anti-pattern : vérifie par `grep -nF` (`-F` obligatoire) ou lecture directe avant de l'écrire, et cite l'extrait.
- Tout décompte écrit dans ton rapport se recompte depuis la source, avec son périmètre.
- Classe chaque finding CRITICAL / HIGH / MEDIUM / LOW, numérote-les `V3-n`, cite la ligne de la fiche et celle du code, propose la correction.
- Rapport : décompte, findings, points vérifiés et tenus, **tableau des axes exercés / non exercés avec le comment**, ce qui n'a pas pu être exercé et pourquoi, questions au Project Lead. Réponse finale : le décompte et le chemin du rapport.
