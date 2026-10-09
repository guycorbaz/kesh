# Prompt — validation P2 des specs, Stories 15-1c-i et 15-1c-ii

*Versionné le 2026-10-09. Deux lentilles (Opus), contexte frais chacune, en lecture seule. Rotation (décision D6 de la
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0` = origin/main,
qui contient le socle du lettrage : 15-1a-i, PR #587, et 15-1a-ii, PR #593). **Fiche** :
`_bmad-output/implementation-artifacts/15-1c-i-ecran-postes-ouverts.md` et `15-1c-ii-lettrage-dans-kesh.md` (index : `15-1c-proposition-ecran.md`) — **relis-les ENSEMBLE, la couture entre elles est un axe**. Deuxième passe. La P1 (Sonnet ×2 : 4 HIGH, 16 MEDIUM bruts ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p1-{R,F}.md`)
est remédiée par `bb894777`, qui a DÉCOUPÉ la 15-1c (C-15-1c-1) en 15-1c-i (l'écran, `refs #518`) et 15-1c-ii (le
lettrage dans le reste de Kesh, `closes #518`) ; choix C-15-1c-1 à 13. Axes prioritaires :
1. **AC15 de la 15-1c-i face au contrat de la 15-1b** : l'enrichissement de `GET /letterings/{key}` réemploie-t-il
   réellement la requête B de la 15-1b (factorisation possible, sans N+1 pour `journal` et `description`) ? La prévision
   `manualDissolutionBlockedBy`, calculée par une fonction pure que `dissolve_group_in_tx` appellerait aussi : la
   refonte de `dissolve_group_in_tx` change-t-elle un comportement livré (tests de la 15-1a-i et de la 15-1a-ii
   inchangés et verts) ? C'est un axe de sécurité.
2. **Le test 3 de la 15-1c-i** : un geste RÉEL produit-il un groupe `reversal` contenant une ligne encore possédée par
   sa pièce (annulation d'une facture fournisseur ?), à vérifier contre `document_owners` (15-1b-0) et le code ; sinon,
   le repli en SQL brut est-il honnête ?
3. **La dérogation de la 15-1c-i** (C-15-1c-11), élargie au-delà de C-15-1a2-23 aux éditions mécaniques d'une ligne :
   tient-elle, ou faut-il une coupe ?
4. **La 15-1c-ii** : l'inventaire du manuel (`.tex` ET PDF aplati, toutes les occurrences « lettr ») ; l'entrée
   CHANGELOG unique alors que les 15-1b et 15-1a2-* en écrivent chacune une ; « aucun tag entre 15-1c-i et 15-1c-ii ».
5. Couture : rien en double, rien entre deux chaises ; ordre … → 15-1b → 15-1c-i → 15-1c-ii.
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
elle ne compte pas. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p2-<R|F>.md` ; dernier message : le
chemin, le bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`,
`sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
