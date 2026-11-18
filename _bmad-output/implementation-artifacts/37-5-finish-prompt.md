# Prompt de fin de story — 37-5

Worktree `/home/beat/claude/kesh/.claude/worktrees/kesh-s37-5` (branche `story/37-5-reevaluation-fin-exercice`) ;
journaux dans `/home/beat/claude/kesh-gate-logs/s37-5/final2/`. Règles : `CLAUDE.md` (*Test Locally First*, E2E,
*Plafonds mémoire*), `docs/agents-orchestration.md` (base `kesh_s37_5` / `kesh_e2e_s37_5`, port propre, verrou
`flock`, `scripts/mem-guard.sh`, aucun `GRANT` à la main, aucun redémarrage du conteneur), `docs/testing.md`
§ « Les échecs attendus ».

0. **Gate backend complet d'abord** (base `kesh_s37_5` remise à zéro) : `scripts/test-fast.sh --ci` par heavy.sh,
   puis Vitest complet et `npm run build`. Un rouge causé par la story se corrige (« fix(37-5) : … ») avant l'E2E.
1. **E2E complet au dernier commit de code** (la fusion de `epic/37-multi-devises` qui apporte la 37-3a terminée ; code changé par les revues P1 et P2 depuis le dernier E2E) :
   frontend rebuild, base E2E remise à zéro et seedée, backend du worktree, suite complète **avec `LANG=fr_CH.UTF-8`**
   (la station est en `C.UTF-8`, ce qui fait rougir `date-filtre-frappe.spec.ts`). Juge chaque échec fichier par fichier
   contre la liste attendue (KF-029, #97).
2. Pour tout échec hors liste (p. ex. `opening-balances.spec.ts:156`) : rejoue la spec seule **sur la base du commit
   d'avant la story** (`git worktree add` temporaire sur `epic/37-multi-devises` à `19bf12fb`, même montage) pour établir
   s'il est préexistant ; retire ce worktree temporaire après (`git worktree remove`). Un échec causé par la story se
   corrige (commit « fix(37-5) : … »), puis l'E2E complet se rejoue.
3. Si tout est vert (au sens de `docs/testing.md`) : statut de la fiche → `done`, Dev Agent Record complété (chiffres
   E2E, verdict des échecs hors liste), `sprint-status.yaml` : `37-5-…: done` ; commit « docs(37-5) : story terminée ».
   Aucun mot-clé de fermeture, aucune mention d'IA, aucun push.

Rapport bref : E2E (passés, échecs, ignorés, verdict de chaque échec hors liste), commits.

Exécutions lourdes uniquement par `/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus
deux sur la station ; backend lancé avec `7>&- 8>&- 9>&-`). Les échecs hors liste déjà établis comme préexistants à
`19bf12fb` (`backup-restore-new-install.spec.ts:262`, `:381`, `bank-import.spec.ts:130`, `opening-balances.spec.ts:156`,
point 7 de la liste des issues) n'ont pas à être rejoués sur la base. N'arrête que tes propres processus, par PID
(jamais `pkill` à motif large). Deux autres agents travaillent en parallèle (revue de 37-3b port 3375 sur 3306, 37-5 sur 3308).

Aucune mention d'IA dans les commits (pas de `Co-Authored-By: Claude`), aucun `GRANT` à la main.

**Base de données : `kesh-mariadb-dev3`, port 3308** (la tienne). La branche contient déjà tout l'Epic 37 terminé
(fusion `d818fad6`) : pas de fusion à faire, sauf si `epic/37-multi-devises` a bougé depuis (vérifie par
`git log HEAD..epic/37-multi-devises`). Le gate backend complet est déjà vert au dernier commit de code de production
(`8ab80b05`, 5877 passés ; `e1ae8b85` ne touche que des commentaires) : ne le rejoue pas. Fais : Vitest complet (seul)
et `npm run build`, puis l'E2E complet avec `LANG=fr_CH.UTF-8` (port 3376, base `kesh_e2e_s37_5` remise à zéro) à la
pointe de la branche. Captures à jour si l'écran de réévaluation a changé depuis les dernières (P1 à P3). Ne rends pas
ton rapport avant le commit « docs(37-5) : story terminée ».
