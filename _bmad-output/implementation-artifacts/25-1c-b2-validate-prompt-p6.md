# Prompt — passe 6 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b2

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Opus), orthogonale à la passe 5 (Sonnet).
Passe ciblée : les trois findings de la passe 5 portaient tous sur la remédiation de la passe 4.*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la remédiation de la passe 5**, et seulement elle :
`git diff c5d14f32 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`. Nomme cette
base dans ton rapport. Elle trie un site de la brochure, réécrit l'introduction de l'AC 4, corrige un
décompte, et ajoute le Change Log de la passe 5.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Le rejeu de l'AC 8 est-il maintenant exact, ensemble contre ensemble ?** Exécute le motif élargi
   (`piste|trace d.audit|dans l.audit\b|audit-?trail|audit log`, sans casse) en Python sur les **PDF
   aplatis** des trois documents (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) **et** sur `README.md`, pas
   seulement sur les `.tex`. Chaque occurrence est-elle au tableau de l'AC 3 ou dans ses tris ? Un PDF
   coupe-t-il un mot (`piste` en fin de ligne, césure) ou en fait-il apparaître un que le `.tex` ne
   montre pas (macro, texte généré) ?
2. **L'introduction réécrite de l'AC 4** (« quatre faux ou périmés, le premier vrai mais incomplet — il se
   nuance ») est-elle cohérente avec **chaque** ligne du tableau et avec la phrase de conclusion (« L'encadré
   se réécrit ou disparaît ») ? Un encadré qui « disparaît » supprime-t-il la phrase vraie qu'on vient de dire
   à nuancer ?
3. **Propagation** : un décompte touché (« cinq cellules », sites triés, « dix-sept ») est-il répété
   ailleurs sous l'ancienne valeur — corps, tâches, Dev Notes, Change Logs antérieurs qui le présentent
   comme actuel ?
4. **Le Change Log de la passe 5** : tallies brut et après reclassement, recomptés contre sa liste.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`, `git diff`/`git show`, Python en
lecture, `pdftotext` vers la sortie standard ou vers le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p6-b2/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes : utilise Python pour les extractions à
contexte.
