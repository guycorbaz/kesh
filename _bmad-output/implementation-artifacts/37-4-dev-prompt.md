# Prompt de développement — story 37-4 (dev-story)

Tu développes la story **37-4** (`_bmad-output/implementation-artifacts/37-4-rapports-en-devise.md`) selon le
skill `bmad-dev-story` (lis `.claude/skills/bmad-dev-story/SKILL.md` et suis-le), dans le worktree
`<dépôt>/.claude/worktrees/kesh-s37-4` (branche `story/37-4-rapports-en-devise`, partie de
`epic/37-multi-devises`). **N'écris jamais hors de ce worktree**, sauf les journaux dans
`<station>/kesh-gate-logs/s37-4/`. Chemins absolus.

## À lire d'abord

- la fiche 37-4 en entier (ses révisions CR-5 à CR-7 priment ; AC, Tâches, Notes de développement, Note de version) ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions, registre C-37-*, CR-6 à CR-8, A30–A40) et, pour
  les points de contact, les fiches 37-1 (livrée, fusionnée) et 37-4 à 37-5 ;
- `CLAUDE.md` en entier (règles du dépôt : *Test Locally First*, *Plafonds mémoire*, *Migration breaking policy* P1–P8,
  *Travailler à plusieurs développeurs*, *Review Iteration Rule*, *Pattern batch*, recette visuelle) ;
- `docs/agents-orchestration.md` (base de données et port **propres au worktree** : clé `s37-4`, base `kesh_s37_4`,
  cible cargo propre ; **ni redémarrage ni recréation du conteneur MariaDB** ; verrou `flock` des gates lourds ;
  `scripts/mem-guard.sh`) et `docs/testing.md`.

## Règles

- Étape 0 : recontrôle au code des prémisses de la fiche (lignes citées), consigné au Change Log.
- Commits locaux à chaque étape structurante (pas de push). Messages de commit sans mot-clé de fermeture
  (`closes`/`fixes`/`resolves`). Aucune mention d'IA ni de Claude dans les commits ou le code.
- Manuels : `.tex` seulement, **aucun PDF commité**. Pas de `CHANGELOG.md` : la section « Note de version » de la fiche.
- Migration : fichier horodaté neuf ; P5 (ligne d'audit + compteurs recomptés), P6 (grep positionnel), P7 (triage) ;
  aucune migration breaking attendue en 37-4 (vérifie).
- Gates : ciblés pendant l'itération ; **gate complet** (fmt, clippy `-D warnings`, nextest complet sur ta base,
  frontend `check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code, sous
  `scripts/mem-guard.sh` et le verrou ; journaux dans `kesh-gate-logs/s37-4/`. Le Dev Agent Record n'affirme que ce qui
  a tourné.
- Captures d'écran (thème clair) des états visibles touchés, rangées dans `kesh-gate-logs/s37-4/captures/`.
- Ne touche ni `main`, ni les autres branches, ni le conteneur de dev partagé au-delà de ta base.

## Fin

Statut de la fiche → `review` ; Dev Agent Record complet (fichiers, tests ajoutés recomptés, gates réellement
exécutés avec chiffres, écarts) ; commit. Message final bref : ce qui est livré, gates (chiffres), écarts à la fiche,
questions.

## Propre à 37-4

- **Base de départ** : cette branche part du commit `946aa86a` de `story/37-3a-ecritures-en-devise-modele` (modèle de
  ligne en devise, validation, contrôle central ; kesh-core et kesh-db), lui-même parti de `story/37-2` (cours). La
  37-3a continue en parallèle (API `journal-entries`, MCP, exports) ; la 37-2 est en revue. **Ne modifie ni le modèle de
  ligne ni le code de 37-2 et 37-3a** : un défaut trouvé là va à
  `<station>/kesh-gate-logs/issues-a-ouvrir-epic-37.md` sous « pour la revue de 37-2/37-3a ». L'orchestrateur
  fusionnera les branches amont dans celle-ci ; ne fais pas de `git merge` toi-même. Évite de toucher aux fichiers que
  la 37-3a modifie encore (routes `journal-entries`, MCP des écritures) sauf nécessité, et dis-le.
- **Étape 0 déjà faite** contre 37-2 : `<station>/kesh-gate-logs/etape0/37-4.md` (les prémisses « attend
  37-3a » se vérifient maintenant contre `946aa86a`). Reporte-la au Change Log.
- `balances_as_of` vit dans `kesh-db` (cycle `kesh-db` ↔ `kesh-report` confirmé). Aucune migration n'est attendue ; si
  une s'impose, dis-le avant de l'écrire.
- Base de test `kesh_s37_4` / `kesh_e2e_s37_4`, port **3374**. Exécutions lourdes **uniquement** par
  `<station>/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus deux sur la station) ; backend
  lancé avec `7>&- 8>&- 9>&-`.
- Les défauts hors périmètre vont à `<station>/kesh-gate-logs/issues-a-ouvrir-epic-37.md`, jamais en issue.
