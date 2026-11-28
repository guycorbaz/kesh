# Story 49-2 — prompt de la revue de code P3 (Haiku, passe ciblée, contexte frais)

Passe **ciblée** (une lentille, le seul commit de la dernière remédiation) de la revue de code de la story 49-2 de Kesh
(réordonnancement des lignes d'une facture en brouillon, #1756, v0.20.1). Kesh est en production. Workflow
`bmad-code-review`, mais restreint : P1 (Opus) 0/0/2/11, P2 (Sonnet) 0/0/1/0 ; **seul le dernier correctif porte la sévérité**.
Rapport de la P2 : `$GATE_LOGS/e492/49-2-revue-p2.md`. Worktree `/home/gcorbaz/devel/kesh-49-2`, branche
`story/49-2-reordonnancement-lignes`. Donne à ta lecture un **diff unique aplati** : `git diff <tête de la P2>..HEAD`
(la remédiation de M-P2-1 seule).

## Le finding à vérifier comme résolu

**M-P2-1** — `frontend/tests/e2e/invoice-line-reorder.spec.ts` (bloc « L'ordre dans le PDF », `execFileSync('pdftotext', …)`).
Aucune autre spec E2E n'emploie `pdftotext`, le serveur des gates (idefix, `ssh kesh-runner`) ne l'a pas (`command -v
pdftotext` : rc 1), `docs/testing.md` § *Prérequis Playwright local* ne le liste pas, la CI (`e2e-smoke`, `ci.yml`) ne
l'installe pas : l'E2E complet y rougit par `ENOENT`. L'ordre des lignes dans le PDF est déjà prouvé par
`de_haut_en_bas` / `ordonnee` (`crates/kesh-api/tests/invoice_line_types_e2e.rs`, T4, `kesh_qrbill::empreinte::textes`).
Remède attendu : retirer le bloc `pdftotext` (et l'import `execFileSync`, `testInfo` s'il n'a plus d'usage), garder `%PDF-`,
corriger le commentaire d'en-tête (« texte extrait par `pdftotext` ») et les mentions de la fiche (Dev Agent Record,
Change Log : `grep -n pdftotext` sur la fiche) ; ou, si l'orchestrateur a préféré documenter le prérequis, qu'il soit
écrit à `docs/testing.md` et que le test s'ignore proprement s'il manque. Vérifier : plus aucune spec ne dépend d'un outil
absent des gates ; le test reste fidèle à son titre (« l'ordre tient jusqu'au PDF » : si le titre promet encore le PDF,
dire où l'ordre du PDF est prouvé) ; `grep -rn pdftotext frontend/tests` ; la spec se lit encore (types, imports inutilisés :
`npx tsc`/`npm run check` ciblé si possible).

## Axes à déclarer exercés ou non exercés

1. M-P2-1 résolu (ci-dessus). 2. Propagation : `grep -rnF pdftotext` sur `frontend/`, la fiche, `docs/testing.md`.
3. Le diff ne touche ni code de production ni manuel (`git diff --stat`). 4. Aucun mot-clé de fermeture dans les messages
(`git log origin/main..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`).
5. Manuel : sans objet si le diff n'en touche pas (le dire). **Un « 0 finding » sans cette liste ne compte pas.**

## Interdits

Rien hors du worktree, de `/tmp` et de `$GATE_LOGS/e492/` ; aucun correctif ; ni `gh` en écriture, ni
`scripts/prepare-release.sh`, ni `docker`, ni `sed -i` ; jamais `pkill`/`killall`/`kill` par motif ; rien sous `~/kesh/`.
Pour tout CRITICAL ou HIGH affirmant une absence : `grep -nF` d'abord (garde-fou Haiku).
