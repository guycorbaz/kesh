# Prompt — passe 7 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b1

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Sonnet), orthogonale à la passe 6 (Opus).
Avant-dernière passe autorisée par le plafond de 8.*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la remédiation de la passe 6**, et seulement elle :
`git diff 32af4148 -- _bmad-output/implementation-artifacts/25-1c-b1-journal-audit-ecran.md`. Nomme cette
base dans ton rapport. Cette remédiation **retire** une assertion E2E (un compteur de requêtes) et la
mutation qui la prouvait, et renvoie la preuve au test unitaire de l'AC 12.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le retrait laisse-t-il un trou ?** Sans le compteur, quelle mutation plausible de la garde (AC 6)
   n'est plus attrapée par **aucun** test — ni le test unitaire de `load()` (AC 12, patron
   `frontend/src/routes/(app)/users/users-page.test.ts:26-53`), ni `toHaveURL('/')` et l'accueil rendu
   (AC 13, scénario 7) ? Lis le patron unitaire : prouve-t-il vraiment que `load()` **jette** pour un
   Consultation, ou seulement autre chose ?
2. **La table des mutations de l'AC 14 est-elle encore juste ?** La ligne « garde retirée » annonce deux
   tests rouges (unitaire et E2E) : est-ce vrai des deux ? Une ligne a-t-elle perdu sa raison d'être, ou en
   manque-t-il une ?
3. **Propagation** : le retrait est-il complet (grep `aucune requête`, `page.on`, `listener`, `onMount`,
   `77-8` hors Change Log) ? Le texte neuf du scénario 7 se lit-il sans renvoi cassé ?
4. **Le Change Log de la passe 6** : décomptes, et fidélité de la décision au texte du corps.

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
`git diff`/`git show`, `npx vitest run` en lecture, et une copie jetable **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p7-b1/`.
