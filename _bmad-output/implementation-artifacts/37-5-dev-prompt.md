# Prompt de développement — story 37-5 (dev-story)

Tu développes la story **37-5** (`_bmad-output/implementation-artifacts/37-5-reevaluation-fin-exercice.md`) selon le
skill `bmad-dev-story` (lis `.claude/skills/bmad-dev-story/SKILL.md` et suis-le), dans le worktree
`<dépôt>/.claude/worktrees/kesh-s37-5` (branche `story/37-5-reevaluation-fin-exercice`, partie de
`epic/37-multi-devises`). **N'écris jamais hors de ce worktree**, sauf les journaux dans
`<station>/kesh-gate-logs/s37-5/`. Chemins absolus.

## À lire d'abord

- la fiche 37-5 en entier (ses révisions CR-5 à CR-7 priment ; AC, Tâches, Notes de développement, Note de version) ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions, registre C-37-*, CR-6 à CR-8, A30–A40) et, pour
  les points de contact, les fiches 37-1 (livrée, fusionnée) et 37-5 à 37-5 ;
- `CLAUDE.md` en entier (règles du dépôt : *Test Locally First*, *Plafonds mémoire*, *Migration breaking policy* P1–P8,
  *Travailler à plusieurs développeurs*, *Review Iteration Rule*, *Pattern batch*, recette visuelle) ;
- `docs/agents-orchestration.md` (base de données et port **propres au worktree** : clé `s37-5`, base `kesh_s37_5`,
  cible cargo propre ; **ni redémarrage ni recréation du conteneur MariaDB** ; verrou `flock` des gates lourds ;
  `scripts/mem-guard.sh`) et `docs/testing.md`.

## Règles

- Étape 0 : recontrôle au code des prémisses de la fiche (lignes citées), consigné au Change Log.
- Commits locaux à chaque étape structurante (pas de push). Messages de commit sans mot-clé de fermeture
  (`closes`/`fixes`/`resolves`). Aucune mention d'IA ni de Claude dans les commits ou le code.
- Manuels : `.tex` seulement, **aucun PDF commité**. Pas de `CHANGELOG.md` : la section « Note de version » de la fiche.
- Migration : fichier horodaté neuf ; P5 (ligne d'audit + compteurs recomptés), P6 (grep positionnel), P7 (triage) ;
  aucune migration breaking attendue en 37-5 (vérifie).
- Gates : ciblés pendant l'itération ; **gate complet** (fmt, clippy `-D warnings`, nextest complet sur ta base,
  frontend `check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code, sous
  `scripts/mem-guard.sh` et le verrou ; journaux dans `kesh-gate-logs/s37-5/`. Le Dev Agent Record n'affirme que ce qui
  a tourné.
- Captures d'écran (thème clair) des états visibles touchés, rangées dans `kesh-gate-logs/s37-5/captures/`.
- Ne touche ni `main`, ni les autres branches, ni le conteneur de dev partagé au-delà de ta base.

## Fin

Statut de la fiche → `review` ; Dev Agent Record complet (fichiers, tests ajoutés recomptés, gates réellement
exécutés avec chiffres, écarts) ; commit. Message final bref : ce qui est livré, gates (chiffres), écarts à la fiche,
questions.

## Propre à 37-5

- **Base de départ** : cette branche part de `story/37-4-rapports-en-devise` (revue close, clôture en cours), qui
  contient 37-1, 37-2 et 37-3a terminées et 37-4 complète (`balances_as_of` dans `kesh-db`). La 37-4 finit sa clôture
  et la 37-3b est en revue de code, en parallèle : **ne modifie pas leur code** ; un défaut trouvé là va à
  `<station>/kesh-gate-logs/issues-a-ouvrir-epic-37.md` sous « pour la revue de 37-3b/37-4 ». L'orchestrateur
  te dira de fusionner leurs correctifs ; ne fais pas de `git merge` toi-même.
- **Écrans de la 37-3b non présents** dans ta base (formulaire en devise, motif `ENTRY_IS_FX_REVALUATION` dans
  `blocker-messages.ts`) : si l'écran des exercices en a besoin, dis-le, et ne le recrée pas.
- **Étape 0 déjà faite** contre 37-2 : `<station>/kesh-gate-logs/etape0/37-5.md` (table de correspondance des
  lignes de `fiscal_years.rs`, `errors.rs`, `journal_entries.rs`, `lib.rs`, `audit_route_registry.rs`) ; revérifie
  contre ta base et reporte au Change Log.
- `balances_as_of(None)` rend aussi les comptes de résultat en devise, cumulés sur tous les exercices, et les comptes
  archivés (relevé de la revue de la 37-4) : la réévaluation ne vise que le bilan, filtre en conséquence.
- Contrôle de clôture **K9** `FOREIGN_ACCOUNTS_NOT_REVALUED` bloquant (CR-8), aperçu de la réévaluation avant le bouton,
  « huit contrôles » → neuf partout (documents cités par l'étape 0). Cours 2024 du responsable de l'epic : journaliers AFC du 31.12.2024.
- **Base de données : troisième MariaDB, conteneur `kesh-mariadb-dev3`, port 3308**, réservé à cette story (mêmes
  identifiants ; base `kesh` migrée et seedée). Bases `kesh_s37_5` / `kesh_e2e_s37_5` dessus, port d'application
  **3376**. Toute opération de base s'y fait, jamais sur `kesh-mariadb-dev` ni `kesh-mariadb-dev2`.
- Exécutions lourdes **uniquement** par `<station>/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` ;
  backend lancé avec `7>&- 8>&- 9>&-`. Captures (thème clair) dans `<station>/kesh-gate-logs/s37-5/captures/`.
- **Aucune mention d'IA dans les commits** (pas de `Co-Authored-By: Claude`), aucun `GRANT` à la main.
- Les défauts hors périmètre vont à `<station>/kesh-gate-logs/issues-a-ouvrir-epic-37.md`, jamais en issue.
