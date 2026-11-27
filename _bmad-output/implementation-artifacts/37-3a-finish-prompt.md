# Prompt de fin de story — 37-3a

Worktree `<worktree>/kesh-s37-3a` (branche `story/37-3a-ecritures-en-devise-modele`) ;
journaux dans `$GATE_LOGS/s37-3a/final2/`. Règles : `CLAUDE.md` (*Test Locally First*, E2E,
*Plafonds mémoire*), `docs/agents-orchestration.md` (base `kesh_s37_3a` / `kesh_e2e_s37_3a`, port propre, verrou
`flock`, `scripts/mem-guard.sh`, aucun `GRANT` à la main, aucun redémarrage du conteneur), `docs/testing.md`
§ « Les échecs attendus ».

0. **Gate backend complet d'abord** (base `kesh_s37_3a` remise à zéro) : `scripts/test-fast.sh --ci` par heavy.sh,
   puis Vitest complet et `npm run build`. Un rouge causé par la story se corrige (« fix(37-3a) : … ») avant l'E2E.
1. **E2E complet au dernier commit de code** (`5ca6e24c` ; code changé depuis le dernier E2E par les revues P1 à P4) :
   frontend rebuild, base E2E remise à zéro et seedée, backend du worktree, suite complète **avec `LANG=fr_CH.UTF-8`**
   (la station est en `C.UTF-8`, ce qui fait rougir `date-filtre-frappe.spec.ts`). Juge chaque échec fichier par fichier
   contre la liste attendue (KF-029, #97).
2. Pour tout échec hors liste (p. ex. `opening-balances.spec.ts:156`) : rejoue la spec seule **sur la base du commit
   d'avant la story** (`git worktree add` temporaire sur `epic/37-multi-devises` à `19bf12fb`, même montage) pour établir
   s'il est préexistant ; retire ce worktree temporaire après (`git worktree remove`). Un échec causé par la story se
   corrige (commit « fix(37-3a) : … »), puis l'E2E complet se rejoue.
3. Si tout est vert (au sens de `docs/testing.md`) : statut de la fiche → `done`, Dev Agent Record complété (chiffres
   E2E, verdict des échecs hors liste), `sprint-status.yaml` : `37-3a-…: done` ; commit « docs(37-3a) : story terminée ».
   Aucun mot-clé de fermeture, aucune mention d'IA, aucun push.

Rapport bref : E2E (passés, échecs, ignorés, verdict de chaque échec hors liste), commits.

Exécutions lourdes uniquement par `$GATE_LOGS/heavy.sh scripts/mem-guard.sh <commande>` (au plus
deux sur la station ; backend lancé avec `7>&- 8>&- 9>&-`). Les échecs hors liste déjà établis comme préexistants à
`19bf12fb` (`backup-restore-new-install.spec.ts:262`, `:381`, `bank-import.spec.ts:130`, `opening-balances.spec.ts:156`,
point 7 de la liste des issues) n'ont pas à être rejoués sur la base. N'arrête que tes propres processus, par PID
(jamais `pkill` à motif large). Deux autres agents travaillent en parallèle (37-3b port 3375, 37-4 port 3374).

Aucune mention d'IA dans les commits (pas de `Co-Authored-By: Claude`), aucun `GRANT` à la main.
