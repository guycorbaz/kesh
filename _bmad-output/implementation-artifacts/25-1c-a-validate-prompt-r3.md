# Prompt — revalidation R3 CIBLÉE, Story 25-1c-a

*Versionné le 2026-09-15. Une lentille Opus en contexte frais, orthogonale à R2 (Sonnet).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche courante `story/25-1c-b-journal-audit-ecran` (qui contient la fiche à jour).
**Périmètre : la remédiation R2**, `git diff 02d3d45f 8715effb -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`.
Nomme cette base.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **La forme « conditionnelle en argument »** ajoutée aux formes résolues de l'AC 18 : est-elle écrite de
   façon implémentable par lecture de texte (bornes de l'expression `if … { } else { }` au milieu d'une liste
   d'arguments séparés par des virgules, accolades imbriquées) ? Est-ce vraiment le **seul** site de cette
   forme — et existe-t-il une **autre** forme encore non écrite (`match`, bloc `{ … }`, appel de fonction qui
   rend l'action, constante `const`) ? Rejoue l'extraction en Python sur `crates/*/src` avec **exactement** les
   formes et l'inventaire écrits dans la fiche : l'ensemble obtenu vaut-il 92 actions et 28 types, **sans**
   aucun site non résolu non inventorié ?
2. **Les annotations de péremption** (« 110 libellés », « 82 codes ») : lisibles sans ambiguïté ? Reste-t-il,
   dans tout le fichier, un nombre périmé non annoté ?
3. **Le Change Log R2** : fidèle au diff ; le reclassement HIGH → MEDIUM est-il justifié par ce qu'il affirme
   (total juste, échec bruyant) ? Décomptes.

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés**.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`, ni `sqlx migrate`, ni
`cargo test/nextest/build`. Autorisés : lecture, `grep`, `git show/diff`, Python en lecture, copie jetable dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r3-a/`. `grep` est
`ugrep` : Python pour les extractions à contexte.
