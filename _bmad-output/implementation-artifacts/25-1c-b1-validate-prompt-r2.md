# Prompt — revalidation R2 CIBLÉE, Story 25-1c-b1

*Versionné le 2026-09-15. Une lentille Sonnet en contexte frais, orthogonale à R1 (Opus).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. Contrat : `25-1c-a-journal-audit-route.md`.
**Périmètre : la remédiation R1**, `git diff 2efbc76c 6cda8b89 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`.
Nomme cette base. ⚠️ **Motif à chasser** : sur cette fiche, plusieurs passes ont trouvé qu'une assertion ajoutée
pour en garantir une autre était **muette** à son tour. Pour **chaque** test ou mutation ajouté par R1, construis
l'état où il **devrait rougir**, et montre qu'il rougit.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **`i18nLocale()` (AC 3)** : écrivable dans `src/lib/shared/utils/i18n.svelte.ts` sans casser ses tests
   existants ni ses mocks ailleurs (`grep` des `vi.mock('$lib/shared/utils/i18n.svelte'`) ? Un mock qui
   n'exporte pas `i18nLocale` fait-il échouer des tests de pages **existantes** qui n'en ont pas besoin ?
   La mutation « `i18nLocale()` rend toujours `'fr-CH'` » rougit-elle réellement le test annoncé ?
2. **L'espion `Intl.Collator`** : `vi.spyOn(Intl, 'Collator')` est-il possible (propriété configurable) et
   compatible avec `new Intl.Collator(…)` ? Exécute-le sous vitest ou Node dans le scratchpad.
3. **Le test de page « la page passe par `toSelectOptions` »** : l'option `zz.legacy` et l'ordre par libellé
   sont-ils observables dans jsdom sur un `<select>` natif lié par Svelte 5 ? Rougit-il sous la mutation
   annoncée, et reste-t-il vert sur une implémentation correcte ?
4. **La preuve de traduction E2E (scénario 1)** : lire `/api/v1/audit-log/vocabulary` par `authedApiContext`
   — quel rôle porte ce contexte (le Comptable créé, ou l'Admin du seed) ? Les deux sont autorisés (25-1c-a
   AC 6) ? Le libellé lu est-il dans la même langue que l'écran ?
5. **Les faits réécrits (AC 5, Dev Notes)** : `invoices/+page.svelte:314-318`, `invoices.spec.ts:167`,
   `bits-ui/dist/bits/select/select.svelte.js:909`, « treize pages », `i18n.svelte.ts:25-35`, `app.html:2`,
   `api-client.ts:19-28`, `reports.api.ts:338-346`, `imported-supplier-invoices.api.ts:66-69` — recompte et
   relis.
6. **Propagation et cohérence** : 133 / 92, AC 9 (trois commentaires) contre T1, l'erreur de liste (AC 5)
   contre les tests (AC 12), le Change Log R1 (décomptes : 2 MEDIUM, 8 LOW).

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés** — rien de « vérifié » sans exécution.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`/`npm run test:e2e`, ni
`sqlx migrate`, ni `cargo test/nextest`. Autorisés : lecture, `grep`, `git show/diff`, `npx vitest run` en
lecture, Python/Node en lecture, copie jetable dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r2-b1/`. `grep` est
`ugrep` : Python pour les extractions à contexte.
