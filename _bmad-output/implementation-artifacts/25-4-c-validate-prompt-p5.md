# Prompt — validation P5 ciblée, Story 25-4-c (le résiduel au rapprochement)

*Versionné le 2026-09-29. **Passe ciblée** (CLAUDE.md § « La passe ciblée ») : une lentille (Haiku),
contexte frais, braquée sur la remédiation de la P4. La P4 a trouvé un seul MEDIUM : un passage faux
du manuel du rapprochement, absent de l'inventaire de la fiche.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-c-residuel-au-rapprochement`. Fiche :
`_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md` — **lis le fichier dans son
état actuel**. La remédiation à relire : `git show 08646df7 -- _bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md`
(les numéros de ligne font foi dans le fichier courant, pas dans le diff).

## Axes — tous obligatoires

1. **La remédiation elle-même** : `:1226` (dans `crates/kesh-api/src/routes/reconciliation.rs`) est-il
   bien le test `score.total <= 0.0` ? `:1406` de `docs/manual/fr/user-manual.tex` est-il bien l'action
   « Modifier » ? Le décompte « trois autres passages » est-il cohérent **partout** dans la fiche
   (AC 6, T4, section Arbitrages) — `grep -n "deux autres\|trois autres\|deux passages\|trois passages" <fiche>` ?
2. **Le symptôme, pas le site** : la P4 a trouvé un passage faux que la liste omettait ; il peut y en
   avoir d'autres. Lis **toute** la section rapprochement du manuel
   (`grep -n "section{Réconciliation\|subsection" docs/manual/fr/user-manual.tex`, puis la section
   entière) et, pour chaque affirmation vérifiable (bouton, action, code d'erreur, compte, seuil,
   comportement), confronte-la au code (`crates/kesh-api/src/routes/reconciliation.rs`,
   `frontend/src/lib/features/reconciliation/`, `frontend/src/routes/(app)/reconciliation/` s'il existe).
   Toute affirmation fausse **non** listée par la fiche (ni corrigée par l'AC 6, ni renvoyée à l'issue
   séparée) est un finding.
3. **Le PDF** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/p5-manual.txt` ;
   les passages que tu signales y figurent-ils tels quels ?

## Ce que tu rends

- **Findings** : sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact, **preuve** : pour chaque finding,
  la commande exécutée **et sa sortie copiée**, ou l'extrait de code lu avec son numéro de ligne. Pour
  toute affirmation qu'un élément est **absent** du code, le `grep -rnF` qui le prouve et sa sortie
  (vide). Un finding sans preuve ne sera pas retenu ; un « 0 finding » sans preuve non plus.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été**, et pour l'axe 2 la liste
  des affirmations du manuel contrôlées (ligne, verdict).

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad.
