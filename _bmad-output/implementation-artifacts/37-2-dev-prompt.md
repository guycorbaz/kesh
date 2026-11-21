# Prompt de développement — story 37-2 (dev-story)

Tu développes la story **37-2** (`_bmad-output/implementation-artifacts/37-2-cours-mensuels-afc.md`) selon le
skill `bmad-dev-story` (lis `.claude/skills/bmad-dev-story/SKILL.md` et suis-le), dans le worktree
`<dépôt>/.claude/worktrees/kesh-s37-2` (branche `story/37-2-cours-mensuels-afc`, partie de
`epic/37-multi-devises`). **N'écris jamais hors de ce worktree**, sauf les journaux dans
`<station>/kesh-gate-logs/s37-2/`. Chemins absolus.

## À lire d'abord

- la fiche 37-2 en entier (ses révisions CR-5 à CR-7 priment ; AC, Tâches, Notes de développement, Note de version) ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions, registre C-37-*, CR-6 à CR-8, A30–A40) et, pour
  les points de contact, les fiches 37-1 (livrée, fusionnée) et 37-3a à 37-5 ;
- `CLAUDE.md` en entier (règles du dépôt : *Test Locally First*, *Plafonds mémoire*, *Migration breaking policy* P1–P8,
  *Travailler à plusieurs développeurs*, *Review Iteration Rule*, *Pattern batch*, recette visuelle) ;
- `docs/agents-orchestration.md` (base de données et port **propres au worktree** : clé `s37-2`, base `kesh_s37_2`,
  cible cargo propre ; **ni redémarrage ni recréation du conteneur MariaDB** ; verrou `flock` des gates lourds ;
  `scripts/mem-guard.sh`) et `docs/testing.md`.

## Règles

- Étape 0 : recontrôle au code des prémisses de la fiche (lignes citées), consigné au Change Log.
- Commits locaux à chaque étape structurante (pas de push). Messages de commit sans mot-clé de fermeture
  (`closes`/`fixes`/`resolves`). Aucune mention d'IA ni de Claude dans les commits ou le code.
- Manuels : `.tex` seulement, **aucun PDF commité**. Pas de `CHANGELOG.md` : la section « Note de version » de la fiche.
- Migration : fichier horodaté neuf ; P5 (ligne d'audit + compteurs recomptés), P6 (grep positionnel), P7 (triage) ;
  aucune migration breaking attendue en 37-2 (vérifie).
- Gates : ciblés pendant l'itération ; **gate complet** (fmt, clippy `-D warnings`, nextest complet sur ta base,
  frontend `check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code, sous
  `scripts/mem-guard.sh` et le verrou ; journaux dans `kesh-gate-logs/s37-2/`. Le Dev Agent Record n'affirme que ce qui
  a tourné.
- Captures d'écran (thème clair) des états visibles touchés, rangées dans `kesh-gate-logs/s37-2/captures/`.
- Ne touche ni `main`, ni les autres branches, ni le conteneur de dev partagé au-delà de ta base.

## Fin

Statut de la fiche → `review` ; Dev Agent Record complet (fichiers, tests ajoutés recomptés, gates réellement
exécutés avec chiffres, écarts) ; commit. Message final bref : ce qui est livré, gates (chiffres), écarts à la fiche,
questions.

## Propre à 37-2

- 37-1 est livrée et fusionnée dans `epic/37-multi-devises` : réutilise son code (devise des comptes, réglage, exercices,
  module ISO 4217) sans le dupliquer.
- Source BAZG : **aucun appel réseau dans les tests** (source factice et route `_test/exchange-rates/source` de la fiche) ;
  le client HTTP borné et le filtrage XML hostile (R5) sont exigés. Un seul GET réel de contrôle vers
  `backend-rates.bazg.admin.ch` est permis pendant le développement, pour vérifier la forme de la réponse.
- Les défauts trouvés hors périmètre vont à `<station>/kesh-gate-logs/issues-a-ouvrir-epic-37.md` (ajoute une
  entrée numérotée), jamais en issue GitHub.
