# Prompt — validation P3 des specs, Stories 15-1c-0, 15-1c-i et 15-1c-ii

*Versionné le 2026-10-09. Deux lentilles (Sonnet), contexte frais chacune, en lecture seule. Rotation (décision D6
de la rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle.*

Dépôt `/home/gcorbaz/devel/kesh-15-1c`, branche `story/15-1c-0-validation-p2` (base `f9b6b199` = origin/main, qui
contient le socle du lettrage livré : 15-1a-i, 15-1a-ii ; les 15-1a2-*, 15-1b-0, 15-1b sont des **fiches validées non
développées**, prises comme contrat). Remédiation de la P2 : commit `82524343`. **Fiches** (dans
`_bmad-output/implementation-artifacts/`) : `15-1c-0-groupe-de-lettrage-enrichi.md` (NEUVE : partie serveur extraite
de la 15-1c-i), `15-1c-i-ecran-postes-ouverts.md`, `15-1c-ii-lettrage-dans-kesh.md` ; index
`15-1c-proposition-ecran.md` (son Change Log porte le bilan de la P2, finding par finding). **Relis-les ENSEMBLE : la
couture entre les trois est un axe.** Troisième passe. La P2 (Opus ×2 : 7 MEDIUM distincts, tous nés de la
remédiation P1 ; rapports `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p2-{R,F}.md`) est remédiée par
`82524343` (`git show 82524343`), qui a extrait la 15-1c-0 (C-15-1c-14) ; choix C-15-1c-14 à 23 au registre.

⚠️ **Le motif mesuré de cet epic : la sévérité se déplace vers ce qu'on vient d'écrire.** Lis d'abord `git show 82524343`
et vérifie chaque texte neuf contre le code et les fiches amont. Axes prioritaires :

1. **La fonction d'ordre à étapes de la 15-1c-0** (`ManualDissolutionBlocker`, `DissolutionStep`,
   `manual_dissolution_step`, C-15-1c-15) contre `crates/kesh-db/src/repositories/letterings.rs`
   (`dissolve_group_in_tx`, `first_document_owner`, `books_locked_through`, `any_line_in_open_period`), la 15-1a2-i
   (`dissolve_group_inner`) et la 15-1b-0 (D3, `document_owners`) : la séquence des lectures sous verrou est-elle
   **exactement** celle d'aujourd'hui ? les `details` du refus 2 inchangés ? le mode `System` intact ? l'ordre vit-il
   réellement une fois ? les tests existants restent-ils verts sans modification ? Axe de sécurité.
2. **La part « pièce et période » de la requête B** réemployée par la lecture d'un groupe (15-1c-0 AC15), contre la
   15-1b (AC1-AC4, T1) : la factorisation est-elle possible et nommée sans deviner ? `journal`/`description` lus sans
   N+1 (`FIND_GROUP_SQL`) ?
3. **Le sens au pied** (15-1c-i AC8, C-15-1c-16) et sa propagation (15-1c-ii AC9, AC12) contre `kesh-report`
   (`opening.rs`, `general_ledger.rs`, `trial_balance.rs`) et la 15-1b « Définitions ».
4. **« Tout 404/409 recharge »** (15-1c-i AC4, AC5, AC6, C-15-1c-17) contre les statuts réels des refus
   (`crates/kesh-api/src/errors.rs`, table des codes du lettrage ; la route `routes/letterings.rs`) et les chemins
   qui modifient ou suppriment une écriture.
5. **La 15-1c-ii** : inventaire du manuel (les trois `.tex` de `docs/manual/fr/` ET les PDF aplatis
   `pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' '`), brochure, `colspan` du Grand livre
   (`frontend/src/lib/features/reports/GeneralLedgerView.svelte`), CHANGELOG et README tenus par chaque story
   (C-15-1c-21), « aucun tag entre 15-1c-i et 15-1c-ii ».
6. **Couture** : rien en double, rien entre deux chaises entre 15-1c-0, -i, -ii ; ordre
   … → 15-1b → 15-1c-0 → 15-1c-i → 15-1c-ii partout ; la table de correspondance de l'index ; numérotation (AC15,
   AC16, AC18 → 15-1c-0 ; AC19 → 15-1c-i ; tests renumérotés) ; recompte des AC, tâches, tests de chaque fiche contre
   les Change Logs.

Fiches voisines (contexte, ne pas valider) : `15-1a-i-marque-du-lettrage.md`, `15-1a-ii-gardes-du-lettrage.md`,
`15-1a2-0-lettrage-fige-avec-la-periode.md`, `15-1a2-i-lettrage-des-pieces-clients.md`,
`15-1a2-ii-fournisseurs-et-rattrapage.md`, `15-1b-0-propriete-des-lignes-par-lot.md`, `15-1b-vue-lignes-ouvertes.md`.
Issue : #518 (`gh api repos/guycorbaz/kesh/issues/518`). Registre : `epic-15-choix-autonomes.md` (C-15-1c-1 à 23).
Règles : `CLAUDE.md` (§ « Un appariement automatique propose, il ne crée jamais », § Review Iteration Rule, § Règle
de splitting préventif et amendement D5, § Migration breaking policy).

## Lentilles

- **R — Auditeur d'acceptation** : applique `.claude/skills/bmad-create-story/checklist.md`. Chaque phrase de #518 qui
  revient à ces stories couverte par un AC testable, chaque AC par une tâche, chaque tâche par un test ; implémentable
  sans deviner (fonctions, fichiers, codes d'erreur, clés i18n 4 locales, replis Svelte) ; faits cités revérifiés au
  code ; recompte des AC, tâches et tests ; frontière avec les fiches voisines (rien en double, rien entre deux
  chaises).
- **F — Full-scope adversary** : la conception elle-même, contre le socle livré. Les chemins qui écrivent ou lisent ce
  que les stories changent (inventorier les sites NON résolus) ; transactions, verrous, ordre des contrôles, cycles
  avec la création/dissolution de groupe, la contre-passation qui lettre, la clôture, la modification et la
  suppression d'écriture, les règlements, les avoirs, les annulations ; tests existants qui changeront de sens ;
  frontend ; i18n ; le **manuel** (`.tex` et PDF aplati), `docs/api-external.md`, CHANGELOG, README, site, brochure ;
  la règle de découpage.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (fiche et code), **preuve** (commande et
sortie copiée, ou code cité relu), correction proposée, et pour chaque MEDIUM+ : **né de la remédiation P2** (le texte
fautif est-il dans `git show 82524343` ?) ou **d'origine**. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas. Rapport
dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-validate-p3-<R|F>.md` ; dernier message : le chemin, le bilan, une
ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/
`fetch`/`reset`/`rebase`, `sqlx`, `cargo`, `npm`, `npx`, `docker`, `gh` en écriture, SQL. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
