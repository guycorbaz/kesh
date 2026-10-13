# Prompt — revue de code P3 **ciblée**, Story 28-5b (annuler le rejet d'une transaction, côté écran, #526)

*Versionné le 2026-10-13. **Une seule lentille**, **Haiku**, contexte frais, lecture seule, braquée sur **la seule
remédiation de la passe ciblée P2** (`CLAUDE.md`, § *La passe ciblée* ; D6, rétrospective de l'Epic 25). Trend : P1
(Sonnet, B, E, A) **0/0/1/11** → P2 ciblée (Haiku) **0/0/0/2** (P2-1 : raison fausse de B-L2 ; P2-2 : commentaire de
fuseau). ⚠️ La remédiation P2 **touche une ligne de production** (décision de l'orchestrateur) : la boucle n'est pas
close tant qu'une passe ne l'a pas relue.*

Dépôt `/home/gcorbaz/devel/kesh-28-5`, branche `story/28-5b`. **Périmètre : la remédiation P2, lue en DIFF UNIQUE
aplati** — ne lis pas la séquence des commits :

```sh
git diff 201b8edb HEAD -- frontend                    # production (une ligne) et tests
git diff 201b8edb HEAD -- _bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md
```

Rapport P2 que ce diff remédie (à lire) : `/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p2-ciblee.md`. Journal de la
mutation : `/home/gcorbaz/devel/kesh-gate-logs/28-5b/mutations-p2-run.txt` et `28-5b/mutations/M24.log`. Gate
frontend : `28-5b/p2-remed-frontend.log`.

Fichier de production touché : **un seul**, d'une ligne utile. Vérifie-le d'abord :
`git diff --stat 201b8edb HEAD -- frontend/src ':(exclude)*.test.ts' ':(exclude)*.spec.ts'` doit ne rendre que
`frontend/src/routes/(app)/bank-import/[id]/+page.svelte`, et `git diff --stat 201b8edb HEAD -- frontend/tests` rien.

## La lentille — chasseur de régressions de la remédiation P2

1. **La ligne de production** — `errorMessage = null` en tête de `load()` : place (avant la garde de l'identifiant
   invalide, qui pose elle-même un message) ; effet sur **chaque** appelant de `load()` — `onMount`, `onUnrejected`
   d'`UnrejectTransactionAction`, `onSuccess` de `CancelReconciliationDialog` (`grep -nF "load()" "frontend/src/routes/(app)/bank-import/[id]/+page.svelte"`).
   Une relecture qui **réussit** pendant qu'une autre **échoue** : l'ordre des réponses (la plus ancienne après la plus
   récente) peut-il encore laisser la bannière, ou effacer une erreur légitime ? `detail` reste-t-il celui de la
   précédente lecture pendant la relecture, et la bannière masque-t-elle toujours les lignes (`{:else if
   errorMessage}`) ? Le message « Identifiant invalide » ou « Import introuvable. » peut-il clignoter, ou disparaître à
   tort ?
2. **Le test neuf** (`bank-import-detail-page.test.ts`, « une relecture réussie après une relecture en échec… ») :
   prouve-t-il ce qu'il nomme ? La bannière est-elle **asserée présente** avant la seconde relecture (sinon le test
   passerait à vide) ? Les trois réponses de `getMock` (`mockResolvedValueOnce` / `mockRejectedValueOnce`) sont-elles
   consommées dans l'ordre supposé — et `getMock` appelé exactement trois fois ? Le second `UnrejectTransactionAction`,
   démonté par la bannière, appelle-t-il encore `onUnrejected` après sa résolution — et le test en dépend-il de façon
   fragile ? Le mock de `$lib/features/reconciliation/reconciliation.api` casse-t-il les autres tests du fichier
   (`CancelReconciliationDialog` importe `getReconciliationTransaction` et `cancelReconciliation`) ? M24 rougit-elle,
   sur ce test (`mutations/M24.log`) ?
3. **P2-2** — le commentaire de la fixture (`UnrejectTransactionAction.test.ts`) dit-il désormais vrai (aucun fuseau
   fixé par la suite : `grep -rn "TZ\|timeZone" frontend/vite.config.ts frontend/package.json`) ? L'arithmétique
   « UTC-11 à UTC+12 ; rougirait en UTC+13 ou UTC+14 » est-elle exacte pour 2026-03-04T11:00Z ?
4. **Fiche** : l'entrée « Revue de code P2 ciblée » dit-elle vrai (B-L2 **corrigé**, non plus assumé ; M24 au tableau
   des mutations ; *Annuler le rapprochement* couvert par construction, **non** par un test — est-ce écrit ainsi) ?
   L'entrée P1 est-elle annotée et **non réécrite** (consigne 15) ? Les chiffres de gates déclarés correspondent-ils à
   `p2-remed-frontend.log` (recompte : `grep -E "Test Files|Tests  " …`) ?
5. **Manuel et CHANGELOG contre code (obligatoire), `.tex` ET PDF aplati** : la correction change un comportement
   visible (la page se répare après une relecture en échec) — un texte du manuel, du CHANGELOG ou de
   `docs/api-external.md` le contredit-il, ou devrait-il le dire ? `git diff --stat 201b8edb HEAD -- docs CHANGELOG.md` ;
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` pour « Rejetée le », « Annuler le rejet ».
6. **Propagation** : greper le **symptôme** — d'autres pages qui posent `errorMessage` (ou un équivalent) dans un
   `catch` de relecture sans jamais le remettre à `null`, et dont la bannière masque le contenu
   (`grep -rn "errorMessage = " frontend/src/routes --include=*.svelte`) : à **lister** comme constat, sans en exiger la
   correction (hors périmètre de la story) ; et `auto-réparé` dans la fiche (la raison fausse de B-L2 doit n'y survivre
   qu'annotée).

⚠️ **Entrées passées** : les entrées antérieures du Change Log et du Dev Agent Record gardent leurs chiffres d'alors
(consigne 15) : ce n'est pas un résidu.
⚠️ **Grep ground-truth obligatoire** : toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée, sur le
**fichier final**. Un CRITICAL ou HIGH qui affirme qu'un patch « n'a pas été appliqué » se vérifie ainsi avant d'être
écrit.
⚠️ **Erreur de catégorie** : B-L1 = E-3 (date invalide) reste un LOW non corrigé, écrit avec sa raison ; la limite des
100 (#649, C-28-5-39) est un angle mort assumé. Ne les reprends que si leur raison écrite est fausse.

## Ce que tu rends

Findings numérotés `P3-1…` avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve** (commande et
sortie copiée), origine (remédiation P2 / rédaction antérieure), correction proposée. ⛔ **La liste des axes exercés
ET non exercés** (les six ci-dessus) — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-5b-review-p3-ciblee.md` ; dernier message : le chemin du rapport, le bilan par
sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`,
`latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`,
`cargo`, `npm`, `npx`, `node`, `vitest`, `playwright`, `docker`, `bash` ou `sh` sur un script, **tout interpréteur**
(`python`, `python3`, `perl`, `ruby`, `deno`, `awk` avec un programme qui écrit), `gh` en écriture, toute requête SQL.
Rien de `~/kesh-test/` (relevé réel de Guy). Autorisés : lecture, `grep`, `sed -n`, `wc`, `find`, `git log`/`show`/
`diff`/`blame`/`grep`, `gh api` en lecture, `pdftotext` vers la sortie standard.
