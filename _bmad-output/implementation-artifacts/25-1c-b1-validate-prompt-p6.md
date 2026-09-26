# Prompt — passe 6 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b1

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), orthogonale à la passe 5 (Sonnet).*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la remédiation de la passe 5**, et seulement elle :
`git diff c5d14f32 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`. Nomme cette
base dans ton rapport. Elle touche le scénario E2E 7 (ordre du listener de requêtes), ajoute **une ligne
à la table des mutations de l'AC 14**, et le Change Log.

⚠️ **Le motif à chasser** : trois passes de suite, l'assertion ajoutée pour en garantir une autre était
muette. La remédiation de la passe 5 répond par **une mutation**. Vérifie que cette mutation est **réelle** :

1. **Est-elle écrivable et fait-elle ce qu'elle dit ?** Une garde déplacée de `load()` (`+page.ts`) dans le
   composant (`onMount` qui fait `goto('/')`) : avec `ssr = false`, la page monte-t-elle, et l'appel
   `listAuditLog` part-il **avant** le `goto` (ordre des `onMount`/`$effect` de Svelte 5, et où la page
   prescrite par l'AC 5 déclenche-t-elle son premier chargement) ? Si le chargement part après la
   redirection, ou n'a pas le temps de partir, la mutation **ne rougit pas** le test — et la ligne ajoutée
   à l'AC 14 est fausse.
2. **Sa colonne « test attendu rouge » dit-elle vrai** : `toHaveURL('/')` reste-t-il vert sous cette
   mutation, comme la ligne l'affirme ?
3. **Le listener « attaché avant le goto »** : la requête est-elle réellement observable par `page.on('request')`
   (requête `fetch` du client vers la même origine) ? Un préchargement ou un cache pourrait-il la masquer ?
4. **Références neuves** (`homepage-reminders.spec.ts:77-82,88`) et cohérence du Change Log (décomptes).

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`npm run test:e2e`, `sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`,
`git diff`/`git show`, et une copie jetable **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p6-b1/`.
