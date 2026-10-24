# Prompt — validation P3, Story 32-12 (une sauvegarde antérieure reste importable)

*Versionné le 2026-10-24. Passe **complète**, **deux lentilles, R et F**, chacune en contexte frais, **Sonnet**.
Lecture seule stricte. Elle suit la P2 (Opus ; R 0/0/2/6, F 0/0/3/6), remédiée au commit `c4d3f8bb`.*

Dépôt `/home/gcorbaz/devel/kesh-32-12`, branche `story/32-12`. Fiche :
`_bmad-output/implementation-artifacts/32-12-import-sauvegarde-anterieure.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3212.md` (C-32-12-1 à 29) ; fichier d'epic
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-12*, § *Ce que le code dit déjà* point 4, Q4) ; rapports
précédents : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p{1,2}-{R,F}.md`. Règles : `CLAUDE.md` (§ *Review
Iteration Rule* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —, *Règle de splitting
préventif* et son amendement D5, *Exception kesh-db*, *Migration breaking policy* **P1, P2-bis, P7, P8**, *Le prompt
d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter ses propres
comptes rendus*, *Recette visuelle*), `docs/agents-orchestration.md` § 3 et § 6. Issue : `gh issue view 722` (lecture
seule).

## Attention particulière : la remédiation de la P2, et l'inventaire de la classe

La sévérité se déplace vers ce qu'on vient d'écrire. Lis `git show c4d3f8bb` **en entier** avant le reste, et pour
chaque hunk demande-toi ce qu'il a cassé ailleurs dans la fiche ou le registre.

**L'inventaire de la classe** est le cœur de cette remédiation. Les deux MEDIUM de la P1 et les trois de la P2 avaient le
même motif : *des textes qui deviennent faux sans être nommés*. La fiche ne les corrige plus site par site : l'AC17
définit la classe (toute phrase qui affirme l'égalité des inventaires, la non-importabilité d'une sauvegarde antérieure,
une fenêtre « recalculée depuis le `MIGRATOR` » / « jamais codée en dur » / refermée par toute table neuve, ou que
`20260917000001` est « la dernière créatrice »), l'inventorie par **cinq familles de `grep`** (G1 à G5) et la valeur, et
les Dev Notes portent un tableau *Inventaire de la classe* où chaque site a un sort.

- **Exécute chacun des cinq greps et la valeur tels qu'écrits** (portée `S` comprise) ; recompte (`| wc -l`) contre les
  nombres déclarés (50, 7, 29, 10, 18, 5). Une commande mal formée, un échappement qui ne fonctionne pas sous `grep -E`,
  une portée qui manque un répertoire : finding.
- **Confronte chaque ligne rendue au tableau** : une ligne de la classe qui n'y figure pas, ou qui y figure avec un sort
  faux (« reste vrai » pour une phrase qui devient fausse ; « relevé daté » pour une phrase au présent), est un
  finding. Les homonymes exclus doivent l'être à bon droit.
- **Cherche ce que les cinq familles manquent encore** — c'est la question qui a produit les MEDIUM des deux passes
  précédentes. Ne pars pas des greps : pars du **sens** (après la story, qu'est-ce qui cesse d'être vrai ?) et cherche
  les phrases qui le disent avec d'autres mots — par exemple sur « toute table neuve », « la fenêtre se referme »,
  « l'inventaire croît », « une sauvegarde 0.x ne s'importe pas », « tables du manifeste = inventaire », dans le code,
  les doc-comments, les messages d'assertion, les migrations, `CLAUDE.md`, le manuel `.tex` **et son PDF**, `docs/`, le
  README, `website/`, les fichiers d'epic et de planning.
- **Les quatre migrations P8** (`20261001000004`, `20261003000001`, `20261009000001`, `20261024000001`) : sont-elles les
  seules ? `grep -rniE "table neuve|nouvelle table|inventaire" crates/kesh-db/migrations` ; le sort « intouchable » est-il
  juste pour chacune (publiée : `git tag --contains` ou `git ls-tree <tag>`) ?
- **Le reste de la remédiation** : `last_table_creating_migration` retirée et AC2 (ii) sur le maximum de la carte
  (C-32-12-23) — aucun autre appelant ? `split_statements` `pub(crate)` suffit-il à `backup.rs` ? ; `TOLEREES_ECRITES` /
  `tolerees_calculees` (C-32-12-28) — l'AC2 (iii) est-il non tautologique, M3 tuée par quoi exactement ? ; AC6 au code
  seul et AC7 (b) au message exact (C-32-12-24) — vérifie toi-même `errors.rs:2171-2180`, et que l'AC7 (b) est faisable
  sans base, avec les règles de production, sur la fixture privée de `products` ; message `HorsOrdre` exact dans les deux
  cas ; AC12 réparti entre `full_import` et `run_backup_and_restore` (lignes justes ? `ImportOutcome` atteignable ?) ;
  six tests lexicaux (C-32-12-25) — il n'y en a pas d'autres ? ; `version.rs:272-275` (C-32-12-26) ; `post_restore.rs:441`
  confié à la 32-5 (C-32-12-27) ; AC20 (v) et (vi) ; *Dérogation règle de splitting* (C-32-12-29 : huit modules, le
  décompte se retrouve-t-il ?).
- Le **Change Log** « Validation P2 » : décomptes, convergences, sort de chaque finding se retrouvent-ils dans la fiche et
  dans les rapports P2 ? Le relevé P1 a-t-il été laissé intact (règle *Recompter n'est pas réécrire l'historique*) ?

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Inventaire actuel.** `parse_and_verify` (`import.rs:172` et suivantes) compare-t-il l'inventaire dans les deux sens
   (l. 232-250), et nulle part ailleurs ? Inventaire **clos** des sites qui lisent `parsed.tables` ou
   `TABLES_TO_TRUNCATE`.
2. **Point 1 — la carte table → migration.** Une migration créatrice pour chacune des 39 tables ; `20260917000001`
   dernière créatrice applicative à la v0.15.0 ; le raisonnement prescrit au doc-comment de la reconnaissance déplacée
   (AC1 : `split_statements` suffit ; graphie ratée → zéro créatrice → (i) rouge) est-il vrai ?
3. **Point 2 — les exemptions.** Huit « Hors fenêtre », fondements non marqués, `assert_eq!(checked, 8)`, analyse après
   la 32-5 ; `scripts/prepare-release.sh` (lecture seule, **ne pas l'exécuter**).
4. **Point 3 — l'état après import.** `restore_body`, rejeu, `missing_sentinels`, `repair_installation_in_tx`,
   `force_onboarding_done_if_eligible`, contrôle des pièces ; tables système exclues par nom.
5. **Point 4 — le bilan.** Réponse et audit aux lignes dites ; où `parsed` est déplacé ; `ImportOutcome`.
6. **Point 5 — la fixture.** Générateur 0.14.0, fixture existante, `git diff --stat v0.15.0 HEAD -- crates/ frontend/src`,
   repli.
7. **Manuel.** `docs/manual/fr/admin-manual.tex` § *Importer* (l. 2242 et suivantes), refus (l. 2252-2263), § *Les
   pièces à l'import* (l. 2273), § *Ce que dit l'écran* (l. 2277), § *Reprises* (l. 2305), migration de machine
   (l. 2370) : les retouches de l'AC20 (i) à (vi) sont-elles exactes et suffisantes ? Une phrase **ailleurs** (manuel
   utilisateur, `docs/`, README, `website/`) lie-t-elle encore l'importabilité à un inventaire identique ou à « la même
   version » ? Contrôle le **PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' |
   grep -oiE "même version|plus récente|importable"`.
8. **Tests et mutations.** Chaque test prescrit rougit-il pour la bonne raison ? M1-M9 : tuée par un test **nommé**, ou
   déclarée équivalente / reportée avec la raison ? En particulier M1 (AC3 et AC7 (b) ; plus l'AC6) et M3. Un test qui
   passerait à vide — la fiche le dit-elle ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : les cinq points « Ce que le code établit » sont-ils établis au code ? C-32-12-3 tient-il ?
2. **La règle** : plancher et suffixe ferment-ils les cas visés ? Un cas légitime refusé, un cas illégitime accepté et
   non nommé ?
3. **Périmètre et découpage** : la *Dérogation règle de splitting* est-elle fondée (deux sites lourds, six mécaniques ;
   pas de recyclage de défauts P1 → P2, au sens de l'amendement D5) ? Migration réellement inutile ?
   `parse_and_verify_with` compatible avec `spawn_blocking` et le générique `R` ?
4. **Faisabilité des tests** : AC2 dans `backup.rs` sans `last_table_creating_migration` ; AC4 sur `"./test-schema"` ;
   AC7 (a) et (b) dans `import.rs` (`include_bytes!`, `pub(crate)`, sans base pour (b)) ; AC11 et M7 ; AC1 (i) sans base.
5. **Décomptes** : recompte chaque nombre de la fiche et du registre (39 tables, 83 migrations, 8 « Hors fenêtre »,
   12 490 octets, 837 attributs `"./test-schema"`, 29 choix au registre, six tests lexicaux, huit modules, quatre
   migrations P8, les nombres de G1-G5 et de la valeur, lignes citées) ; signale celui qui ne se retrouve pas.
6. **Cohérence interne** : AC1-AC21, T0-T7, registre C-32-12-1 à 29 (annotations de C-32-12-10, 12, 17), encadré de
   forme réduite, tableau des sites, inventaire de la classe, *Contrat* (cinq points), Change Log — contradiction,
   critère invérifiable, commande mal formée (exécute chaque `grep`).
7. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (Rust, Vitest, Playwright), une
   conséquence pour la recette visuelle, le CHANGELOG, la 32-5.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour chacun, dire s'il vient **de la remédiation P2** (`c4d3f8bb`) ou de la fiche
antérieure. ⛔ **La liste des axes exercés ET non exercés** (R1 à R8, F1 à F7, et l'*Attention particulière*, inventaire
de la classe compris) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe
pas encore : son absence n'est pas un défaut. Bilan par sévérité **recompté** sur la liste. Rapport écrit **d'un seul
jet**, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p3-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/3212-validate-p3-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire ; aucune redirection d'écriture (`>`, `tee`)
ailleurs que vers ton rapport. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `unzip` vers le disque (`unzip -l`
et `unzip -p` vers la sortie standard seulement), `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
`gh` en écriture. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame|ls-tree|tag --contains`,
`gh issue view`, `pdftotext … -`, `unzip -l|-p`.
