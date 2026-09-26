# Prompt — revalidation R2 CIBLÉE, Story 25-1c-b2

*Versionné le 2026-09-15. Une lentille Sonnet en contexte frais, orthogonale à R1 (Opus).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-1c-b-journal-audit-ecran`. **Périmètre : la remédiation R1**,
`git diff 2efbc76c 6cda8b89 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md` **et** la
section *fin de soirée* de `_bmad-output/planning-artifacts/epic-25-vague1-suite.md` modifiée par `32ebe675`.
Nomme ces bases.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages du Project Lead.

## Axes (déclare lesquels tu as exercés)

1. **Le procédé Python de l'AC 2** : exécute-le sur les quatre `.ftl` **actuels**. Est-il écrit sans ambiguïté
   (motif, groupe de la valeur, casse) ? Que rendrait-il sur une clé multiligne Fluent ? Le grep hérité
   « en excluant les clés `^(nav-)?audit-log` » est-il exécutable tel qu'écrit ?
2. **L'AC 6 et `admin-manual.tex:1797`** : la citation est-elle exacte ? `invoices.rs:1350` et
   `journal_entries.rs:1061` disent-ils ce que la fiche affirme (suppression d'une facture validée ⇒
   `journal_entry.deleted`) ? La consigne de reformulation est-elle cohérente avec celle de `:1796` et avec
   l'AC 4 ?
3. **L'AC 5, action historique** : « n'est pas proposée dans la liste », « seul un code déjà présent dans
   l'adresse » — est-ce exactement ce que dit la b1 rouverte (AC 3) ? Un utilisateur retrouve-t-il vraiment
   l'entrée « par le type et l'identifiant d'entité » ?
4. **L'epic** : les trois questions et réponses citées (*« ok »*, *« non, maintenant »*, *« corrige »*) sont-elles
   rendues fidèlement, sans résumé qui réécrive l'arbitrage ? Le paragraphe barré est-il sans ambiguïté ? Les
   « 82 codes » concordent-ils avec la 25-1c-a, qui en compte désormais **92** ?
5. **Propagation** : 133, `invoices.rs:1350` / `:1339-1340`, « réouverte », les renvois aux AC de la 25-1c-a ;
   le Change Log R1 (décomptes 2 MEDIUM, 6 LOW).

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés** — rien de « vérifié » sans exécution.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `latexmk`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`, ni `sqlx migrate`, ni
`cargo test/nextest`. Autorisés : lecture, `grep`, `git show/diff`, Python en lecture, `pdftotext` vers la
sortie standard ou vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r2-b2/`.
`grep` est `ugrep` : Python pour les extractions à contexte.
