# 33-11 — mode opératoire des preuves de l'AC-6, pour l'orchestrateur

Le filtre ne se prouve que sur de vraies PR. **Aucune de ces commandes n'a tourné** : elles écrivent sur GitHub, et reviennent à l'orchestrateur. Chaque relevé se cite au Change Log de la fiche, lien de la PR compris. *(Récrit le 2026-11-07, remédiation de la revue P1 : observables nommés, relevé (e), conduite de (c) sous `strict`.)*

## Ce qui s'observe, et comment

- **L'état des jobs** (`<run>` : `gh run list --branch <branche> --workflow CI --limit 3`) :

  ```sh
  gh run view <run> --json jobs --jq '.jobs[] | "\(.name) | \(.status) | \(.conclusion)"'
  ```

  Les quatre noms requis : `Backend (Rust)`, `Frontend (Svelte)`, `Docker build (sanity)`, `E2E smoke (Playwright)`. `changes` s'affiche `Filtre des artefacts`, et n'est pas requis.
- **Le motif de la décision**, au journal de `Filtre des artefacts`. `code=…` part dans `$GITHUB_OUTPUT` et **n'est pas au journal** : seul le motif y est, sur la sortie d'erreur (`ci-seuls-artefacts : …`), et le code se déduit de l'état des quatre jobs.

  ```sh
  JOB=$(gh run view <run> --json jobs --jq '.jobs[] | select(.name == "Filtre des artefacts") | .databaseId')
  gh run view <run> --log --job "$JOB" | /usr/bin/grep -F 'ci-seuls-artefacts :'
  ```

- **Les quatre contextes requis vus par la PR** :

  ```sh
  gh pr view <PR> --json mergeStateStatus,statusCheckRollup \
    --jq '.mergeStateStatus, (.statusCheckRollup[] | select(.name == "Backend (Rust)" or .name == "Frontend (Svelte)" or .name == "Docker build (sanity)" or .name == "E2E smoke (Playwright)") | "\(.name) | \(.status) | \(.conclusion)")'
  ```

- **La protection de `main`**, relue sans rien écrire : `gh api repos/guycorbaz/kesh/branches/main/protection --jq '{strict: .required_status_checks.strict, contexts: .required_status_checks.contexts}'`. Lue le 2026-11-07 : `strict: true`, les quatre contextes ci-dessus, aucune revue exigée, conversations non exigées.

⚠️ `cancel-in-progress` (`ci.yml:9-11`) : pousser sur la branche annule le run en cours. Chaque relevé attend l'état qu'il nomme avant le commit suivant.

⛔ **Ne pas fusionner** tant qu'un commit temporaire de (b), (d) ou (e) n'est pas défait par son `git revert --no-edit`, poussé, **et** que la CI de ce revert n'est pas verte.

## (a) La PR d'intégration, qui touche du code

Ouvrir la PR, en `refs #928`. **Attendu** : `Filtre des artefacts` `success`, motif `hors du périmètre : …` ; les quatre autres `success`.

## (b) Le chemin de l'échec, avant la fusion

Sur la même branche, commit temporaire : en **première** étape du job `changes`, avant `actions/checkout`,

```yaml
      - run: exit 1
```

Pousser. **Relever quand `E2E smoke (Playwright)` a quitté `queued`** (il attend `docker-build`), et de nouveau à la fin du run. **Attendu** : `Filtre des artefacts` `failure` ; les quatre autres **exécutés** (`in_progress`, puis `success`), aucun `skipped`. Puis `git revert --no-edit <commit>`, sans réécrire la branche ; pousser ; la CI repasse.

**Si un job est `skipped` : la PR ne se fusionne pas avec le filtre, et Guy est prévenu.**

## (d) L'annulation, avant la fusion

Commit temporaire, même place :

```yaml
      - run: sleep 180
```

Pousser, attendre que `Filtre des artefacts` soit **`in_progress`**, puis, **dans les 180 s**, `gh run cancel <run>`. Relever une fois le run terminé (`gh run watch <run>`), puis le rollup des quatre contextes, puis `git revert --no-edit <commit>` et pousser.

- **Aucun `skipped`** (`cancelled`, ou jamais créés) : la condition reste.
- **Un job `skipped`**, donc vert pour la protection : **le filtre passe à `always()`**, dans `.github/workflows/ci.yml`, aux quatre jobs : `if: ${{ always() && !(needs.changes.result == 'success' && needs.changes.outputs.code == 'false') }}`, `e2e-smoke` gardant son `&& needs.docker-build.result == 'success'`. La garde de `ci-seuls-artefacts.test.sh` lit l'ouverture sur la ligne du job `backend` et l'exige aux quatre (C-e3311-3) ; `bash scripts/tests/ci-seuls-artefacts.test.sh` le contrôle. Rejouer (a), (b), (d), (e). **Coût à relever au Change Log et à dire à Guy** : sous `always()`, un job lancé ne s'annule plus, et un run remplacé par `cancel-in-progress` va à son terme.

## (e) Le délai dépassé du job `changes`, avant la fusion

La documentation dit qu'un job qui dépasse `timeout-minutes` est annulé ; elle ne dit pas ce que vaut `cancelled()` pour un job dont **seule une dépendance** a été annulée. Ce cas se relève, il ne se suppose pas.

Commit temporaire sur le job `changes` : `timeout-minutes: 1` à la place de `10`, et, en première étape,

```yaml
      - run: sleep 180
```

Pousser, et relever à la fin du run : `Filtre des artefacts` doit être `cancelled` (ou `failure`) ; puis l'état des quatre jobs et le rollup des quatre contextes. `git revert --no-edit <commit>`, pousser, CI verte.

- **Aucun `skipped`** : la condition reste.
- **Un job `skipped`** : **même sanction qu'en (d)**, le filtre passe à `always()`, et (a), (b), (d), (e) se rejouent.

## Une fois (a), (b), (d), (e) relevés, avant la fusion : les textes passent au relevé

Les textes sont au conditionnel tant que rien n'a tourné. Une fois (a), (b), (d) et (e) relevés, et avant de fusionner, repasser au relevé, PR et runs cités : `docs/ci.md` (§ du filtre, l. 9 et 12, et la ligne `changes` du tableau des timeouts), `CHANGELOG.md` (entrée *Ajouté* du filtre, sa réserve retirée) et `CLAUDE.md:297`. Si un relevé contredit le texte, c'est le texte qui change, ou le filtre qui part. Puis `cargo test -p kesh-api --test textes_coherents`.

## (c) Après la fusion : la PR de la rétrospective

Seul moment où `ci.yml` n'est plus dans le diff. La PR ne porte que `_bmad-output/`, avec `closes #928`. **Attendu** : `Filtre des artefacts` `success`, motif `seulement des artefacts de travail` ; **les quatre contextes requis en `SKIPPED`** au rollup ; et la PR fusionnable.

- **`mergeStateStatus` à `UNKNOWN`** : GitHub calcule encore ; réinterroger une minute plus tard, avant toute conclusion.
- **`BEHIND`** : `main` a bougé, sans rapport avec le filtre ; la protection étant `strict`, mettre la branche à jour (bouton *Update branch*, ou `gh api -X PUT repos/guycorbaz/kesh/pulls/<PR>/update-branch`), puis **relever de nouveau le motif de la décision sur le run qui suit la mise à jour** : il dit si le filtre reste efficace après une fusion de `main` dans la branche.
- **`CLEAN`**, les quatre contextes `SKIPPED` : le filtre est prouvé.
- ⚠️ **Relever après la fin de `Filtre des artefacts`** : tant qu'il tourne, les quatre contextes sont en attente et la PR est `BLOCKED`, sans rien dire du filtre.
- **`BLOCKED`** ou un contexte requis **en attente** (`PENDING`, absent du rollup) alors que `Filtre des artefacts` est terminé et que les quatre sont sautés : **le filtre est retiré, et Guy prévenu**. Un `BLOCKED` d'une autre cause (revue, conversation) se lève d'abord, et ne compte pas contre le filtre.

(c) garde l'issue ouverte, non la story : la story passe `done` sur (a), (b), (d) et (e).

## AC-7, au passage

Le `push` sur `main` de la fusion : les quatre jobs tournent, motif `événement « push » : jamais filtré`. Un commit d'artefacts seul sur `main` ne se joue pas exprès : la règle interdit le commit direct, et la fusion de la PR de la rétrospective le montrera.
