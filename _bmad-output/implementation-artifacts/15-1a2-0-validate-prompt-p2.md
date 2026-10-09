# Prompt — validation P2 de la spec, Story 15-1a2-0

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1a2-0-lettrage-fige-avec-la-periode.md`. Deuxième passe. La P1 (Sonnet ×2 : R 4 MEDIUM, F 3 MEDIUM ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-0-validate-p1-{R,F}.md`)
est remédiée par `18f25973` : la story est DORMANTE et n'écrit plus que ce qu'exige le code (la documentation publique du
refus est passée à la 15-1a2-i, AC18 — C-15-1a2-24) ; douze textes (trois familles × quatre locales) écrits dans D2,
conformes à G8 (marqueur d'ordre) et G8-bis (borne « jusqu'à celui-ci » avec antécédent), clés ajoutées à `CLES_569`
(C-15-1a2-26) ; G9 étendu ; `RANGS` à six rangs (20 cas, la paire `InvoiceCredited` × 2 bis exclue et nommée), un bras à
`ecriture_attendue`, la paire 1 bis × 2 bis dans `invoice_write_off.rs`, les bancs fournisseurs à +4 cas ; aucun fichier
de test Rust neuf (C-15-1a2-27) ; `line_in_open_period` sur exercice inconnu → Result (C-15-1a2-25, F-11 en partie
réfuté). **Relis d'abord ce que `18f25973` a écrit dans CETTE fiche**, contre `056997b0` : les douze textes (grammaire
de/it/en, vérité au code, antécédent de « celui-ci », aucun « son avoir » pour un fournisseur) passent-ils réellement
G8/G8-bis/G9 tels qu'ils sont codés dans `loader.rs` et `textes_coherents.rs` ? la matrice `RANGS` à six rangs est-elle
constructible (chaque paire montable) ? « rang 6 × 2 bis à trancher au T0 » : est-ce une décision laissée en suspens qui
devrait être prise maintenant ? la frontière avec la 15-1a2-i (AC18) est-elle sans trou ?
Fiches voisines (contexte, ne pas valider) : `15-1a-i-marque-du-lettrage.md`, `15-1a-ii-gardes-du-lettrage.md`,
`15-1a2-lettrage-des-pieces.md`, `15-1b-vue-lignes-ouvertes.md`, `15-1c-proposition-ecran.md`. Issue (par
`gh api repos/guycorbaz/kesh/issues/518`) : #518. Registre : `epic-15-choix-autonomes.md` (C90–C131, C-15-1a-i-*,
C-15-1a-ii-*). Règles : `CLAUDE.md` (dont § « Un appariement automatique propose, il ne crée jamais » et § Migration
breaking policy).

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de #518 qui
  revient à cette story couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; implémentable
  sans deviner (fonctions, fichiers, codes d'erreur, clés i18n 4 locales, replis Svelte, migration éventuelle et son
  triage P5/P6/P7) ; faits cités revérifiés au code ; recompte des AC et des tâches ; frontière avec les fiches
  voisines (rien en double, rien entre deux chaises).
- **F — Full-scope adversary** : la conception elle-même, contre le socle livré. Les chemins qui écrivent ou lisent ce
  que la story change (inventorier les sites NON résolus) ; transactions, verrous, ordre des contrôles, cycles avec
  la création/dissolution de groupe, la contre-passation qui lettre, la clôture, la modification et la suppression
  d'écriture, les règlements et les avoirs ; tests existants qui changeront de sens ; frontend ; i18n ; le **manuel**
  (`docs/manual/fr/*.tex` et le PDF aplati `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`),
  `docs/api-external.md`, CHANGELOG ; la règle de découpage (§ « Règle de splitting préventif », amendement D5).

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-0-validate-p2-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
