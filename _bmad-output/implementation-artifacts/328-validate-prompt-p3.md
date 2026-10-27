# Prompt — validation P3, Story 32-8 (la validation avant envoi — la poignée)

*Versionné le 2026-10-25. Passe **complète** (troisième passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**, **sans interpréteur**, **sans fichier temporaire**.*

Dépôt `/home/gcorbaz/devel/kesh-32-8`, branche `story/32-8` (partie de `main` `fa5b3468`). Fiche :
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — quatre parties : 32-8a (la table), 32-8b0 (le
prédicat partagé), 32-8b (la poignée côté serveur), 32-8c (l'écran et les textes) ; registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-328.md` (C-32-8-1 à 29) ; annotations datées du § 32-8 de
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` ; entrées `32-8*` de `sprint-status.yaml`. La remédiation P2 est
le dernier commit de la branche (`git show HEAD --stat`, `git diff HEAD~1 -- _bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md`) ;
l'entrée « validation P2 » du Change Log la résume ; les rapports P2 sont
`/home/gcorbaz/devel/kesh-gate-logs/328-validate-p2-{R,F}.md` (lis-les : chaque finding a-t-il été traité comme le Change
Log le dit ?).

Contexte : la spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.4 (d), § 3, § 5.3, § 6, § 6.2, § 7 et ses
réponses) ; la fiche 32-5 (routes de la 32-5c, l. 495-530) ; en **lecture seule**, la fiche 32-6
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (AC-a1, AC-a15 à AC-a31,
AC-b1 à AC-b4, § *La frontière avec la 32-8*) et la fiche 32-10a
`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (AC-ii1, AC-ii3,
AC-iii1 à AC-iii3, et la note de l'orchestrateur en fin de fiche). Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La
passe ciblée*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une boucle de revue* et son
exception `kesh-db`, *Migration breaking policy* P1 à P8, *Le prompt d'une passe doit NOMMER le manuel*, *Propagation
post-patch* — greper la valeur —, *Recompter ses propres comptes rendus*, *Recette visuelle*, *Règle de commit et push* —
la règle d'inclusion), `docs/agents-orchestration.md`.

**Décisions de l'orchestrateur, à contrôler dans leur application** : ordre 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c,
puis 32-10a ; la 32-8b0 seule porte le prédicat, selon l'AC-ii1 de la 32-10a ; `entree_appel` garde `cle:
&CleMcpAuthentifiee` et `prompts_mcp::proposer` (32-10a) le recevra aussi ; chaque `tools/call` journalisé, dépôt refusé
compris ; les spans interdits dans le chemin des envois ; le grep de la valeur plutôt qu'une liste de lignes. Contraintes
de la story : Q10 (c), Q12, Q11, Q4, la propriété centrale (octet pour octet), au plus cinq modules par partie, aucune
donnée réelle. Ce que la fiche prescrit n'existe pas encore ; ce qu'elle cite des fiches 32-6 et 32-10a se contrôle
contre ces fiches.

⚠️ **Recyclage mesuré** : en P2, neuf MEDIUM sur dix étaient nés de la remédiation P1. Regarde d'abord **ce que la
remédiation P2 a écrit** : l'AC-03 (le changement mesuré de la description d'un compte, ses 41 points, le CHANGELOG), le
test `description_refuse_par_categorie` et M08 ; l'AC-B5 et la journalisation d'un dépôt refusé (MB30), l'AC-B11
(`journaliser_appel` à `envoi`), l'AC-B15 2(b), 3, 9 à 11 ; l'AC-B12 (périmètre élargi, appariement qui saute les
littéraux, garde `aucun_span_dans_le_chemin_des_envois`, MB31) ; la forme « littéral » d'`envois_lus_par_leurs_lieux` ;
les greps de valeur de l'AC-A11, de l'AC-A12 et de l'AC-B8 ; la note pour la 32-10a (points 1, 2, 2 bis) ; les points 7,
11 et 13 pour Guy ; la renumérotation (AC-C 28 à 38, faits déplacés) ; MB9 retirée.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Motifs et mesures greppés sur le dépôt réel.** Dresse la liste des motifs et mesures écrits dans la fiche
  (`grep -nF '→ **' …`, `grep -nF 'grep -' …`, `grep -nF '\b' …`) ; rejoue **chacun** dans son périmètre et **copie la
  sortie** — dont les neufs : `grep -nE "\b(84|50|49)\b" crates/kesh-db/tests/migrations_upgrade_path.rs` (la liste de
  lignes que l'AC-A11 en tire), `grep -nwE "onze|douze|41|42" crates/kesh-api/src/exports/global.rs`, `grep -nE
  "\b(33|8|25|6)\b|vingt-cinq|trente-trois" crates/kesh-api/tests/admin_pat_denied_e2e.rs crates/kesh-api/src/lib.rs`,
  `grep -rnE '#\[(tracing::)?instrument' crates`, `grep -rnE '_span!\(' crates/kesh-api/src`, `grep -rnw 'mcp_keys'
  crates/kesh-api/src | grep '"'`, les lignes de `routes/exports.rs:71`, `:72`, `:81`, `bank_accounts.rs:303-323`,
  `:312`, `:318`, `:1470`, `:1516`, `:1525`. Une mesure que ton grep ne reproduit pas est un finding ; un témoin
  « existant » qui n'existe pas aussi.
- ⛔ **Chaque affirmation sur une dépendance se prouve à sa source**, version du `Cargo.lock` (`~/.cargo/registry/src/*/`) :
  `unicode-properties-0.1.4/src/tables.rs` — **la mesure de l'AC-03** : chaque plage Cf qu'elle dit omise par l'ancienne
  liste (lignes citées), le compte de **41** points, les plages Co, et l'affirmation « aucun caractère refusé aujourd'hui
  n'est admis » (chaque point de l'ancienne liste dans Cf, Cn ou une des listes) ; `serde_json-1.0.150`, `axum-core-0.5.6`
  si une affirmation en dépend ; et la **documentation de `tracing`** à sa source pour l'affirmation de l'AC-B12
  (« `#[instrument]` enregistre par défaut chaque argument ») : `tracing-attributes-*/src/lib.rs`, version du
  `Cargo.lock`. Une affirmation dont tu ne trouves pas la ligne est un finding.

## Lentille R — la spécification contre le code et les fiches

1. **La remédiation P2, finding par finding** (R1 à R13, F1 à F15) : fait comme dit, à la ligne et partout (propagation) ?
   Un symptôme resté ailleurs (« emojis composés » donnés pour neufs, `:211`, « si fusionné », « seule lectrice »,
   « onze ci-dessous », « n'appelle pas `journaliser_appel` » sans la branche de refus, « ordre … 32-8b, puis 32-10a »
   sans la 32-8c) est un finding.
2. **La 32-8b0** : l'AC-03 dit-elle vrai, mesure comprise ? Le test `description_refuse_par_categorie` tue-t-il M08 et
   n'est-il pas vert sur l'ancienne liste ? L'entrée du CHANGELOG dit-elle ce qui change, sans plus ?
3. **La 32-8a** : P1 à P8 ; le tri des lignes de l'AC-A11 (une généalogie historique traitée comme à changer, ou
   l'inverse ?) ; l'AC-A12 et ses consommateurs (`global.rs:695`, `exports_global_e2e.rs:1343`).
4. **La frontière avec la 32-6 et la 32-10a** : l'AC-B15 est-elle **close** après la P2 ? La note pour la 32-10a (points
   1, 2, 2 bis) est-elle juste contre l'AC-ii3 et l'AC-iii2 de la 32-10a et contre la note que l'orchestrateur y a déjà
   versée ?
5. **La spike** : E1, E2, E4, E6-4 et 6, M-1, M-2 — chacun garde-t-il son AC et son test ?
6. **Le manuel et la documentation** (axe obligatoire) : AC-C10, AC-C11, point 13 (la dérogation) contre `CLAUDE.md` ;
   les **PDF aplatis** (`pdftotext docs/manual/fr/{admin,user}-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   ".{60}(MCP|deux tables|export|description).{80}"`) ; `CHANGELOG.md:1-40` ; un texte que la 32-8 rend faux et que la
   fiche ne nomme pas.

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Le découpage** : 5 / 3 / 5 / 4, douze distincts — recompte avec la convention ; les fichiers touchés restent-ils
   dans leurs modules après la P2 (`bank_accounts.rs` en 32-8b0 seulement ; `serveur_mcp.rs` et `journaliser_appel` en
   32-8b) ?
2. **La journalisation d'un dépôt** (AC-B5, AC-B11, AC-B15 2(b), MB30) : chaque issue écrite une fois et une seule ? Un
   chemin qui écrirait deux entrées (transaction validée puis `journaliser_appel`) ou aucune (panic, abandon après la
   transaction, D27) ? Le test `depot_et_journal_dans_la_meme_transaction` a-t-il encore un sens précis ?
3. **Les gardes neuves** : `aucun_span_dans_le_chemin_des_envois` (formes, périmètre, témoins), l'appariement de
   `aucun_evenement_ne_nomme_le_contenu` (littéraux, chaînes brutes, `'('` en caractère, interpolation), la forme
   « littéral » d'`envois_lus_par_leurs_lieux` — exécutables telles qu'écrites, conformes à la règle commune, un faux
   vert possible ?
4. **La propriété centrale et l'inventaire D1 à D27** : un octet qui diffère, un chemin absent, après la P2.
5. **Tests et mutations** : M08, MB30, MB31, MA15, MA16, MB7, MB10 — réalisables et tuées par le test nommé ? MB9 retirée
   à juste titre ? Aucune mutation équivalente restante ?
6. **Décomptes** : 42 AC (12 / 4 / 15 / 11), 18 tâches, 61 lignes de mutation dont 60 actives (MA 16 / M0 8 / MB 31 / MC
   6), D1 à D27, C-32-8-1 à 29, trend P1 0/4/10/15 → P2 0/0/10/18 ; numérotation des AC sans doublon ; `sprint-status.yaml`
   (cinq entrées `32-8*`, YAML lisible).
7. **Ce que la fiche omet** : points pour Guy, demandes à l'orchestrateur, ce qui revient aux 32-7, 32-9, 32-10a, 32-11.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la dépendance,
preuve (commande et sortie copiée), correction proposée — et, pour chacun, s'il est **né de la remédiation P2** ou
antérieur. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R6 ou F1
à F7, et les deux axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul
jet**, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/328-validate-p3-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/328-validate-p3-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`** et tout autre interpréteur, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes** (même en lecture). `pdftotext`
vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees `kesh-32-6` et `kesh-32-10a` se lisent, ne
s'écrivent pas. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les
filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
