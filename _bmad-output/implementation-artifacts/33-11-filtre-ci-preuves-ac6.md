# 33-11 — mode opératoire des preuves de l'AC-6, pour l'orchestrateur

Le filtre ne se prouve que sur de vraies PR. **Aucune de ces commandes n'a tourné** : elles écrivent sur GitHub, et reviennent à l'orchestrateur. Chaque relevé se cite au Change Log de la fiche, lien de la PR compris.

Le relevé, à chaque étape (`<run>` : `gh run list --branch <branche> --workflow CI --limit 3`) :

```sh
gh run view <run> --json jobs --jq '.jobs[] | "\(.name) | \(.status) | \(.conclusion)"'
```

Les quatre noms requis : `Backend (Rust)`, `Frontend (Svelte)`, `Docker build (sanity)`, `E2E smoke (Playwright)`. `changes` s'affiche `Filtre des artefacts`, et n'est pas requis.

⚠️ `cancel-in-progress` (`ci.yml:9-11`) : pousser sur la branche annule le run en cours. Laisser chaque relevé aller assez loin avant le commit suivant : `e2e-smoke` n'a de conclusion qu'après `docker-build`.

## (a) La PR d'intégration, qui touche du code

Ouvrir la PR, en `refs #928`. **Attendu** : `Filtre des artefacts` `success`, les quatre autres `success`. Lire aussi l'étape *Décision* : `code=true`, motif `hors du périmètre : …`.

## (b) Le chemin de l'échec, avant la fusion

Sur la même branche, commit temporaire : en **première** étape du job `changes`, avant `actions/checkout`,

```yaml
      - run: exit 1
```

Pousser, et relever quand `docker-build` a conclu. **Attendu** : `Filtre des artefacts` `failure` ; les quatre autres **exécutés**, aucun `skipped`, `E2E smoke` compris. Puis `git revert --no-edit <commit>`, sans réécrire la branche ; pousser ; la CI repasse.

**Si un job est `skipped` : la PR ne se fusionne pas avec le filtre, et Guy est prévenu.**

## (d) L'annulation, avant la fusion

Commit temporaire, même place :

```yaml
      - run: sleep 180
```

Pousser, puis, pendant le sommeil, `gh run cancel <run>`. Relever une fois le run terminé (`gh run watch <run>`), puis `git revert --no-edit <commit>` et pousser.

- **Aucun `skipped`** (`cancelled`, ou jamais créés) : la condition reste. Relever aussi ce que `gh pr view <PR> --json mergeStateStatus,statusCheckRollup` dit des quatre checks.
- **Un job `skipped`**, donc vert pour la protection : **le filtre passe à `always()`**, dans `.github/workflows/ci.yml`, aux quatre jobs : `if: ${{ always() && !(needs.changes.result == 'success' && needs.changes.outputs.code == 'false') }}`, `e2e-smoke` gardant son `&& needs.docker-build.result == 'success'`. La garde de `ci-seuls-artefacts.test.sh` lit l'ouverture du fichier et suit sans modification (C-e3311-3) ; `bash scripts/tests/ci-seuls-artefacts.test.sh` le contrôle. Rejouer (a), (b), (d). **Coût à relever au Change Log et à dire à Guy** : sous `always()`, un job lancé ne s'annule plus, et un run remplacé par `cancel-in-progress` va à son terme.

## (c) Après la fusion : la PR de la rétrospective

Seul moment où `ci.yml` n'est plus dans le diff. La PR ne porte que `_bmad-output/`, avec `closes #928`. **Attendu** : `Filtre des artefacts` `success`, étape *Décision* `code=false`, les quatre autres `skipped`, et

```sh
gh pr view <PR> --json mergeStateStatus,statusCheckRollup
```

rend `mergeStateStatus` à `CLEAN`. **Sinon, le filtre est retiré et Guy prévenu.** (c) garde l'issue ouverte, non la story : la story passe `done` sur (a), (b) et (d).

## AC-7, au passage

Le `push` sur `main` de la fusion : les quatre jobs tournent, étape *Décision* `code=true`, motif `événement « push » : jamais filtré`. Un commit d'artefacts seul sur `main` ne se joue pas exprès : la règle interdit le commit direct, et la fusion de la PR de la rétrospective le montrera.
