# Prompt — passe 5 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b1

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Sonnet), orthogonale à la passe 4 (Opus).
Passe ciblée : `CLAUDE.md` § « La passe ciblée ».*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la remédiation de la passe 4**, et seulement elle :
`git diff 637c6508 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md` (arbre de
travail ou commit suivant, le contenu est le même). Nomme cette base dans ton rapport. Elle réécrit le
**scénario E2E 7** (redirection d'un Consultation, entrée de menu absente) et le Change Log.

⚠️ **Le motif à chasser est précis** : deux passes de suite, une assertion ajoutée pour empêcher un test
*vrai par construction* l'était elle-même. Pour **chaque** assertion que le scénario 7 prescrit maintenant,
construis l'état où elle **devrait rougir** et montre qu'elle rougit — sinon c'est un finding.

⛔ **Rien ne se croit sur parole** — ni le Change Log, ni les lignes citées.
⚠️ **Ne conteste PAS les arbitrages du Project Lead** ni le contrat de `25-1c-a-journal-audit-route.md`.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Chaque assertion du scénario 7 discrimine-t-elle ?**
   - `toHaveURL('/')` : quelles mutations la font rougir (garde retirée ; garde qui redirige ailleurs ;
     login raté) ?
   - `homepage-card-open-invoices` visible : un Consultation voit-il cette carte (lis
     `routes/(app)/+page.svelte`, conditions de rôle) ? Si elle est masquée pour ce rôle, l'assertion
     rougit **toujours**.
   - **aucune requête** vers `/api/v1/audit-log` : est-ce vrai avec la garde `+page.ts` prescrite
     (`ssr = false`, `redirect` dans `load`) ? SvelteKit précharge-t-il la page au survol du lien
     (`data-sveltekit-preload-data` dans `app.html` ou le layout) — et le lien est-il seulement présent pour
     ce rôle ? Le listener est-il posé avant `goto` ?
   - `nav-link-settings` à `toHaveCount(1)` : ce testid existe-t-il exactement ainsi
     (`+layout.svelte:258-261`, slug de `/settings`) ? Le groupe replié rend-il ses liens dans le DOM
     (`<details>` fermé) ? Un autre lien produit-il le même testid (sous-routes `/settings/...`) ?
2. **Chaque référence neuve dit-elle vrai ?** `homepage-reminders.spec.ts:77-80,83-84,88`,
   `invoice-send-email.spec.ts:154-157`, `+layout.svelte:126`, `(app)/+layout.ts:11,40,43`, les lignes
   d'`api-client.ts` sur `/login` et `/setup`, les trois specs qui créent un Consultation.
3. **Propagation** : l'AC 14 (mutations) et le reste de la fiche sont-ils cohérents avec le scénario 7
   réécrit (la mutation « entrée de menu passée à `items` », la mutation « garde retirée ») ?
4. **Le Change Log de la passe 4** : décomptes et affirmations recomptés.

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
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p5-b1/`.
