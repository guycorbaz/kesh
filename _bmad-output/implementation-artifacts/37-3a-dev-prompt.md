# Prompt de développement — story 37-3a (dev-story)

Tu développes la story **37-3a** (`_bmad-output/implementation-artifacts/37-3a-ecritures-en-devise-modele.md`) selon le
skill `bmad-dev-story` (lis `.claude/skills/bmad-dev-story/SKILL.md` et suis-le), dans le worktree
`<worktree>/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`, partie de
`epic/37-multi-devises`). **N'écris jamais hors de ce worktree**, sauf les journaux dans
`$GATE_LOGS/s37-3a/`. Chemins absolus.

## À lire d'abord

- la fiche 37-3a en entier (ses révisions CR-5 à CR-7 priment ; AC, Tâches, Notes de développement, Note de version) ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (décisions, registre C-37-*, CR-6 à CR-8, A30–A40) et, pour
  les points de contact, les fiches 37-1 (livrée, fusionnée) et 37-3a à 37-5 ;
- `CLAUDE.md` en entier (règles du dépôt : *Test Locally First*, *Plafonds mémoire*, *Migration breaking policy* P1–P8,
  *Travailler à plusieurs développeurs*, *Review Iteration Rule*, *Pattern batch*, recette visuelle) ;
- `docs/agents-orchestration.md` (base de données et port **propres au worktree** : clé `s37-3a`, base `kesh_s37_3a`,
  cible cargo propre ; **ni redémarrage ni recréation du conteneur MariaDB** ; verrou `flock` des gates lourds ;
  `scripts/mem-guard.sh`) et `docs/testing.md`.

## Règles

- Étape 0 : recontrôle au code des prémisses de la fiche (lignes citées), consigné au Change Log.
- Commits locaux à chaque étape structurante (pas de push). Messages de commit sans mot-clé de fermeture
  (`closes`/`fixes`/`resolves`). Aucune mention d'IA ni de Claude dans les commits ou le code.
- Manuels : `.tex` seulement, **aucun PDF commité**. Pas de `CHANGELOG.md` : la section « Note de version » de la fiche.
- Migration : fichier horodaté neuf ; P5 (ligne d'audit + compteurs recomptés), P6 (grep positionnel), P7 (triage) ;
  aucune migration breaking attendue en 37-3a (vérifie).
- Gates : ciblés pendant l'itération ; **gate complet** (fmt, clippy `-D warnings`, nextest complet sur ta base,
  frontend `check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code, sous
  `scripts/mem-guard.sh` et le verrou ; journaux dans `kesh-gate-logs/s37-3a/`. Le Dev Agent Record n'affirme que ce qui
  a tourné.
- Captures d'écran (thème clair) des états visibles touchés, rangées dans `kesh-gate-logs/s37-3a/captures/`.
- Ne touche ni `main`, ni les autres branches, ni le conteneur de dev partagé au-delà de ta base.

## Fin

Statut de la fiche → `review` ; Dev Agent Record complet (fichiers, tests ajoutés recomptés, gates réellement
exécutés avec chiffres, écarts) ; commit. Message final bref : ce qui est livré, gates (chiffres), écarts à la fiche,
questions.

## Propre à 37-3a

- **Base de départ** : cette branche part de `story/37-2-cours-mensuels-afc` (code complet, gates verts, E2E de clôture
  en cours, revue de code à venir), et non de `epic/37-multi-devises`, pour gagner du temps. Réutilise le code de 37-1
  (fusionnée) et de 37-2 (tables et dépôt des cours, `RateKind` dans kesh-core) sans le dupliquer. Ne modifie pas le
  code de 37-2 : un défaut trouvé dans 37-2 va à `$GATE_LOGS/issues-a-ouvrir-epic-37.md` sous
  « pour la revue de 37-2 », et l'orchestrateur le transmet. Les correctifs de la revue de 37-2 seront fusionnés dans
  cette branche par l'orchestrateur ; ne fais pas de `git merge` toi-même.
- **Migration** : nomme la tienne après celle de 37-2 (`20261114000002_exchange_rates.sql`), et dis-la avant de l'écrire
  (fiche).
- **Ordre de travail** : commite tôt le modèle de ligne, la validation et le contrôle central (kesh-core et kesh-db),
  dans un commit à part : la 37-4 pourra partir de ce commit.
- Base de test `kesh_s37_3a` / `kesh_e2e_s37_3a`, port **3373**.
- Les défauts trouvés hors périmètre vont à `$GATE_LOGS/issues-a-ouvrir-epic-37.md`, jamais en
  issue GitHub.
