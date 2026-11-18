# Prompt de fin de story — 37-2

Worktree `/home/beat/claude/kesh/.claude/worktrees/kesh-s37-2` (branche `story/37-2-cours-mensuels-afc`) ;
journaux dans `/home/beat/claude/kesh-gate-logs/s37-2/final2/`. Règles : `CLAUDE.md` (*Test Locally First*, E2E,
*Plafonds mémoire*), `docs/agents-orchestration.md` (base `kesh_s37_2` / `kesh_e2e_s37_2`, port propre, verrou
`flock`, `scripts/mem-guard.sh`, aucun `GRANT` à la main, aucun redémarrage du conteneur), `docs/testing.md`
§ « Les échecs attendus ».

1. **E2E complet au dernier commit de code** (`9a665075` ; le frontend a changé depuis le dernier E2E) :
   frontend rebuild, base E2E remise à zéro et seedée, backend du worktree, suite complète **avec `LANG=fr_CH.UTF-8`**
   (la station est en `C.UTF-8`, ce qui fait rougir `date-filtre-frappe.spec.ts`). Juge chaque échec fichier par fichier
   contre la liste attendue (KF-029, #97).
2. Pour tout échec hors liste (p. ex. `opening-balances.spec.ts:156`) : rejoue la spec seule **sur la base du commit
   d'avant la story** (`git worktree add` temporaire sur `epic/37-multi-devises` à `19bf12fb`, même montage) pour établir
   s'il est préexistant ; retire ce worktree temporaire après (`git worktree remove`). Un échec causé par la story se
   corrige (commit « fix(37-2) : … »), puis l'E2E complet se rejoue.
3. Si tout est vert (au sens de `docs/testing.md`) : statut de la fiche → `done`, Dev Agent Record complété (chiffres
   E2E, verdict des échecs hors liste), `sprint-status.yaml` : `37-2-…: done` ; commit « docs(37-2) : story terminée ».
   Aucun mot-clé de fermeture, aucune mention d'IA, aucun push.

Rapport bref : E2E (passés, échecs, ignorés, verdict de chaque échec hors liste), commits.

Exécutions lourdes uniquement par `/home/beat/claude/kesh-gate-logs/heavy.sh scripts/mem-guard.sh <commande>` (au plus
deux sur la station ; backend lancé avec `7>&- 8>&- 9>&-`). Les échecs hors liste déjà établis comme préexistants à
`19bf12fb` (`backup-restore-new-install.spec.ts:262`, `:381`, `bank-import.spec.ts:130`, `opening-balances.spec.ts:156`,
point 7 de la liste des issues) n'ont pas à être rejoués sur la base. N'arrête que tes propres processus, par PID
(jamais `pkill` à motif large). Deux autres agents travaillent en parallèle (37-3a port 3373, 37-4 port 3374).
