# Prompt — validation P4 ciblée, Story 25-4-d2c

*Versionné le 2026-10-02. **Une lentille** (Haiku), contexte frais — passe ciblée sur la remédiation de P3. Trend :
P1 1H/1M/2L → P2 1H/3M/5L → P3 3M/2L.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-d2c-rapport-tva-soldes`. **Diff à relire : `git diff beb67f61 a4c257f0`**
(la fiche `_bmad-output/implementation-artifacts/25-4-d2c-rapport-tva-soldes.md`, un seul commit, à lire aplati). La
fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté ; une
affirmation de vérification au Change Log n'appelle pas forcément de modification de code.

## Lentille — regression hunter

1. AC 4 : la règle « section et TVA nette **seulement s'il y a des soldes**, partout » contredit-elle une autre phrase de
   la fiche (AC 2, AC 7, Dev Notes) ?
2. AC 1 : la référence `crates/kesh-api/src/errors.rs:954-963` (`TrialBalanceUnbalanced`) existe-t-elle et fait-elle
   bien `tracing::error!` puis `AppError::Internal` ? (`sed -n '950,965p'`)
3. AC 7 : l'ordre « après le solde et avant son annulation » est-il réalisable dans `frontend/tests/e2e/invoice-write-off.spec.ts`
   tel qu'il est (lis le test) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (commande ET sa sortie, obligatoirement). ⛔ La liste des axes exercés ET
non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base (`scripts/*` dont
`scripts/prepare-release.sh`, `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`.
