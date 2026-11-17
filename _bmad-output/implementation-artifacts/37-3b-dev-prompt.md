# Prompt de développement — story 37-3b (dev-story)

Tu développes la story **37-3b** (`_bmad-output/implementation-artifacts/37-3b-ecritures-en-devise-ecrans.md`) selon le
skill `bmad-dev-story` (lis `.claude/skills/bmad-dev-story/SKILL.md` et suis-le), dans le worktree
`/home/beat/claude/kesh/.claude/worktrees/kesh-s37-3b` (branche `story/37-3b-ecritures-en-devise-ecrans`, partie de
`epic/37-multi-devises`). **N'écris jamais hors de ce worktree**, sauf les journaux dans
`/home/beat/claude/kesh-gate-logs/s37-3b/`. Chemins absolus.

## À lire d'abord

- la fiche 37-3b en entier (ses révisions CR-5 à CR-7 priment ; AC, Tâches, Notes de développement, Note de version) ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions, registre C-37-*, CR-6 à CR-8, A30–A40) et, pour
  les points de contact, les fiches 37-1 (livrée, fusionnée) et 37-3b à 37-5 ;
- `CLAUDE.md` en entier (règles du dépôt : *Test Locally First*, *Plafonds mémoire*, *Migration breaking policy* P1–P8,
  *Travailler à plusieurs développeurs*, *Review Iteration Rule*, *Pattern batch*, recette visuelle) ;
- `docs/agents-orchestration.md` (base de données et port **propres au worktree** : clé `s37-3b`, base `kesh_s37_3b`,
  cible cargo propre ; **ni redémarrage ni recréation du conteneur MariaDB** ; verrou `flock` des gates lourds ;
  `scripts/mem-guard.sh`) et `docs/testing.md`.

## Règles

- Étape 0 : recontrôle au code des prémisses de la fiche (lignes citées), consigné au Change Log.
- Commits locaux à chaque étape structurante (pas de push). Messages de commit sans mot-clé de fermeture
  (`closes`/`fixes`/`resolves`). Aucune mention d'IA ni de Claude dans les commits ou le code.
- Manuels : `.tex` seulement, **aucun PDF commité**. Pas de `CHANGELOG.md` : la section « Note de version » de la fiche.
- Migration : fichier horodaté neuf ; P5 (ligne d'audit + compteurs recomptés), P6 (grep positionnel), P7 (triage) ;
  aucune migration breaking attendue en 37-3b (vérifie).
- Gates : ciblés pendant l'itération ; **gate complet** (fmt, clippy `-D warnings`, nextest complet sur ta base,
  frontend `check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code, sous
  `scripts/mem-guard.sh` et le verrou ; journaux dans `kesh-gate-logs/s37-3b/`. Le Dev Agent Record n'affirme que ce qui
  a tourné.
- Captures d'écran (thème clair) des états visibles touchés, rangées dans `kesh-gate-logs/s37-3b/captures/`.
- Ne touche ni `main`, ni les autres branches, ni le conteneur de dev partagé au-delà de ta base.

## Fin

Statut de la fiche → `review` ; Dev Agent Record complet (fichiers, tests ajoutés recomptés, gates réellement
exécutés avec chiffres, écarts) ; commit. Message final bref : ce qui est livré, gates (chiffres), écarts à la fiche,
questions.

## Propre à 37-3b

- **Base de départ** : cette branche part de la pointe de `story/37-3a-ecritures-en-devise-modele` (37-3a livrée en
  `review`, qui contient 37-1 et 37-2 terminées). La 37-3a entre en revue de code en parallèle : **ne modifie pas son
  code** (contrôle central, API, MCP) ; un défaut trouvé là va à
  `/home/beat/claude/kesh-gate-logs/issues-a-ouvrir-epic-37.md` sous « pour la revue de 37-3a ». L'orchestrateur te dira
  de fusionner ses correctifs.
- **Étape 0 déjà faite** contre 37-2 : `/home/beat/claude/kesh-gate-logs/etape0/37-3b.md` ; vérifie maintenant les
  prémisses « attend 37-3a » contre ta base, et reporte le tout au Change Log.
- **Reçu de la 37-3a** : libellés d'écran des nouveaux codes de lot (aujourd'hui le code brut s'affiche), motif
  `ENTRY_IS_FX_REVALUATION` dans `blocker-messages.ts`, type `modificationBlockedBy`. Ordre des refus au `PUT` retenu par
  la 37-3a : `ENTRY_IS_FX_REVALUATION` juste après la garde d'écriture, avant `PERIOD_LOCKED`.
- `CurrencySettingsSection.svelte` (37-1) importe `fetchAccounts` et manque à l'inventaire de l'AC-4 : exempte-le ou
  filtre-le, avec justification. Le manuel n'existe qu'en français. La 37-4 (rapports) avance en parallèle : n'y touche
  pas.
- Pour régénérer le squash, le client MariaDB exige TLS : `MARIADB_HOME=/home/beat/claude/kesh-gate-logs/s37-3a/mycnf`.
- Base de test `kesh_s37_3b` / `kesh_e2e_s37_3b`, port **3375**. Exécutions lourdes **uniquement** par
  `/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` ; backend lancé avec `7>&- 8>&- 9>&-`.
  Captures d'écran (thème clair) dans `/home/beat/claude/kesh-gate-logs/s37-3b/captures/`.
- Les défauts hors périmètre vont à `/home/beat/claude/kesh-gate-logs/issues-a-ouvrir-epic-37.md`, jamais en issue.
