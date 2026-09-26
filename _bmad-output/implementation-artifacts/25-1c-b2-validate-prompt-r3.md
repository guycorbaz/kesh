# Prompt — revalidation R3 CIBLÉE, Story 25-1c-b2

*Versionné le 2026-09-15. Une lentille Opus en contexte frais, orthogonale à R2 (Sonnet).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. **Périmètre : la remédiation R2**,
`git diff 5dfbafc2 0bbefc51 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md _bmad-output/planning-artifacts/epic-25-vague1-suite.md`.
Nomme cette base. ⚠️ **Motif de cette fiche** : les passes 6, R1 et R2 ont chacune trouvé une **propagation
oubliée** par la remédiation précédente. Pour chaque valeur ou notion modifiée, cherche sa jumelle ailleurs.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **« Trois défauts voisins »** : tous les sites qui en parlent (corps, tâches, Dev Notes, table, Change Log
   courant) disent-ils trois, et la ligne `:1797` ajoutée à la table est-elle exacte contre `admin-manual.tex`
   et contre `crates/kesh-db/src/repositories/{invoices,journal_entries}.rs` ?
2. **Le pipeline `grep … | grep -vE ':[0-9]+:(nav-)?audit-log'`** : exécute-le sur les `.ftl` actuels et sur une
   copie jetable où tu ajoutes des clés `audit-log-*` ; exclut-il bien les clés, et elles seules (le format de
   sortie de `ugrep` avec `-rn` a-t-il la forme `fichier:ligne:contenu` attendue) ?
3. **Le rattachement des valeurs Fluent multilignes** : la consigne est-elle implémentable et juste (lignes de
   continuation indentées, lignes vides, commentaires, attributs `.attr`) ?
4. **L'epic** : « 92 codes d'action — 82 en littéral et 10 indirects » concorde-t-il exactement avec la 25-1c-a
   (AC 15, AC 18, Dev Notes) ? La généalogie « 81, puis 82 » est-elle exacte ?
5. **Chemins qualifiés** : tous les renvois à `invoices.rs` et `journal_entries.rs` du **repository** sont-ils
   désormais sans ambiguïté ; Change Log R2 fidèle, décomptes (2 MEDIUM, 3 LOW).

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés**.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `latexmk`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`, ni `sqlx migrate`, ni
`cargo test/nextest`. Autorisés : lecture, `grep`, `git show/diff`, Python en lecture, `pdftotext` vers la
sortie standard ou vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r3-b2/`.
`grep` est `ugrep` : Python pour les extractions à contexte.
