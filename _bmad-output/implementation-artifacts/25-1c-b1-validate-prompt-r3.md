# Prompt — revalidation R3 CIBLÉE, Story 25-1c-b1

*Versionné le 2026-09-15. Une lentille Opus en contexte frais, orthogonale à R2 (Sonnet).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. **Périmètre : la remédiation R2**,
`git diff 0bbefc51 e9b15645 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`. Nomme
cette base. ⚠️ **Motif de cette fiche** : une preuve ajoutée par la remédiation qui ne prouve rien. Pour chaque
test prescrit ou modifié par R2, construis l'état où il **devrait rougir** et montre qu'il rougit.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **Le test direct d'`i18nLocale()`** : étendre `servir` (`i18n.svelte.test.ts:28`) à la locale casse-t-il les
   tests existants de ce fichier ? L'assertion « avant tout chargement ⇒ `'fr-CH'` » est-elle observable,
   alors que l'état du module persiste entre tests (lis comment le fichier réinitialise son module) ? Rougit-il
   sous la mutation « `i18nLocale()` rend toujours `'fr-CH'` » — **exécute-le** sur une copie jetable.
2. **L'angle mort de la locale du tri** : la fiche dit la provenance « prouvée par le test direct » et le
   passage à `toSelectOptions` « relu en revue ». Une mutation plausible — la page passe `'fr-CH'` en dur à
   `toSelectOptions`, ou utilise `navigator.language` pour la date — échappe-t-elle à **tous** les tests ? Si
   oui, l'angle mort est-il écrit assez largement (la colonne Date aussi) ?
3. **La connexion du Comptable (scénario E2E 1)** : `clearAuthStorage` puis `login(page, username, password)`
   — signatures réelles des helpers (`tests/e2e/helpers/`) ? L'utilisateur créé par l'API en Admin peut-il se
   connecter aussitôt (mot de passe, changement forcé au premier login, onboarding) — cherche le patron du
   scénario 7 (`reports.spec.ts:421-439`) et ce que fait `login` ?
4. **Propagation et Change Log R2** : la table des mutations, l'AC 12 et le scénario 1 disent-ils la même
   chose ; décomptes (3 MEDIUM).

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés**.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`/`npm run test:e2e`, ni
`sqlx migrate`, ni `cargo test/nextest`. Autorisés : lecture, `grep`, `git show/diff`, `npx vitest run` en
lecture, Python/Node en lecture, copie jetable dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r3-b1/`. `grep` est
`ugrep` : Python pour les extractions à contexte.
