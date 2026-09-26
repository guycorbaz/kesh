# Prompt — revalidation R4 CIBLÉE, Story 25-1c-b1

*Versionné le 2026-09-15. Une lentille Sonnet en contexte frais, orthogonale à R3 (Opus).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. **Périmètre : la remédiation R3**,
`git diff 717f7763 44d638f9 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md` — commits
« docs(25-1c-b1): revalidation R2 ciblée » et « docs(25-1c-b1): revalidation R3 ciblée ». Si un hash ne résout pas,
retrouve le commit par son message. Nomme la base.

⚠️ **Motif de cette fiche** : une preuve ajoutée par la remédiation qui ne prouve rien. Pour chaque test prescrit ou
modifié par R3, **construis l'état où il devrait rougir et montre qu'il rougit — par exécution**, sous vitest ou
Node, sur une copie jetable.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **Le test `i18nLocale()` sur module frais** : `vi.resetModules()` puis `await import('./i18n.svelte')` dans
   `i18n.svelte.test.ts` — le `vi.mock` de l'`api-client` survit-il au reset ? Le module frais rend-il `'fr-CH'` ?
   Rougit-il sous « valeur initiale vide » **quelle que soit sa place dans le fichier** ?
2. **Le tri sous `'sv-SE'`** (« Zahlung » avant « Öffnung ») : vrai sous l'ICU du **jsdom de vitest**, pas seulement
   sous Node ? Et sur la machine de CI, dont l'ICU peut différer (Node du CI : lis `.github/workflows/ci.yml`) ?
3. **La date sous `'de-CH'`** (`16.09.2026`, sans `sept.`) : `Intl.DateTimeFormat('de-CH', { dateStyle: 'medium',
   timeStyle: 'short' })` rend-il bien ce texte sous vitest/jsdom ? Le fuseau du test peut-il faire basculer la date
   d'une entrée `2026-09-16T12:26:33.123Z` (fuseaux extrêmes) ? Les mutations `'fr-CH'` en dur et
   `navigator.language` rougissent-elles ?
4. **Le `login` à trois arguments** (`company-contact-details.spec.ts:39-49`, appel `:249`) et `test-state.ts:174-191`
   — relis.
5. **Propagation et Change Log R3** : la table des mutations, l'AC 12, l'AC 5 (date) disent-ils la même chose ;
   décomptes (2 MEDIUM, 4 LOW, 1 hors périmètre).

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés**.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`/`npm run test:e2e`, ni
`sqlx migrate`, ni `cargo test/nextest`. Autorisés : lecture, `grep`, `git show/diff/log`, `npx vitest run` en
lecture, Python/Node en lecture, copie jetable dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r4-b1/`. `grep` est
`ugrep` : Python pour les extractions à contexte.
