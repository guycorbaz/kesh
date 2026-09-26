# Prompt — validation P2, Story 25-4-b1 (le résiduel aux agrégats)

*Versionné le 2026-09-27. **Une lentille** (Haiku), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b-residuel-aux-agregats` (empilée sur la 25-4-a,
PR #472). Fiche à valider : `_bmad-output/implementation-artifacts/25-4-b1-residuel-aux-agregats.md`.
Mère : `25-4-propager-le-residuel.md` (arbitrages de Guy : **ne pas les contester**). Sœur livrée :
`25-4-a-residuel-juste.md`. Issue : `gh issue view 416`. Contexte : `24-2-encaissement-client.md`
§ D3, D4, invariants. Règles : `CLAUDE.md`. Checklist : `.claude/skills/bmad-create-story/checklist.md`.

## Axes — tous obligatoires

1. **Chaque référence `fichier:ligne`** existe et dit ce que la fiche affirme.
2. **Inventaire des sites qui somment le TTC là où le reste dû est la grandeur** — pars du symptôme,
   pas de la liste de la fiche : `grep -rn "lt.ttc\|total_ttc\|totalTtc\|INVOICE_TTC" crates/ frontend/src`.
   Pour chaque site, dis s'il est dans la b1, dans la b2 (rappels), ou légitimement au TTC. Un site
   oublié est au moins MEDIUM.
3. **L'invariant de concordance (AC 5)** : est-il vrai en théorie ? Liste ce qui meut le compte
   1100 hors factures (écritures manuelles, soldes d'ouverture, contre-passations, annulation de
   règlement, avoir, facture annulée avec règlement hérité) et vérifie que le montage prescrit les
   exclut ou les inclut à bon escient. Le « périmètre `validated AND paid_at IS NULL` » rend-il la
   concordance possible (factures `cancelled` à reste dû négatif hérité ?).
4. **Le N+1 (AC 8)** : la liste utilise aujourd'hui la forme **corrélée** du TTC
   (`INVOICE_TTC_SUBQUERY_SQL`) ; la fiche prescrit la forme jointe pour le reste dû — cohérence,
   coût, et risque de divergence entre `total_ttc` (corrélé) et `amount_due` (joint) sur une même
   ligne.
5. **Le frontend** : `statusOf` / `paymentStatus` partageables ? Le dialogue pré-rempli : que se
   passe-t-il pour une facture à reste dû négatif ou nul ? Le filtre « payées » de l'échéancier
   montre-t-il des factures dont `amountDue` = 0 ?
6. **Les tests** (AC 12) : chacun prouve-t-il ce qu'il annonce ? L'E2E modifié de la 24-3 : le
   montage (facture partiellement réglée) est-il faisable depuis Playwright ? Les mutations sont-
   elles tuables ?
7. **Le manuel** : lignes citées, et **PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`).
   D'autres textes (admin-manual, api-external, README, website) promettent-ils le TTC ?
8. **Le découpage b1/b2** : quelque chose de b1 appartient-il à b2, ou l'inverse ? La PR de b1 en
   `refs #416` est-elle juste ?

## Ce que tu rends

- **Findings** : sévérité, endroit exact, **preuve** (commande et résultat, code lu), correction.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad, `cargo check`.

## Contexte P1 — à vérifier, pas à re-signaler

La P1 a trouvé : l'invariant AC 5 sans portée (état hérité `cancelled`, soldes d'ouverture) et
trois passages « TTC » du manuel absents de l'AC 13 ; corrigés dans `git diff cfd2a779 73f72e8d`
(**diff unique, à lire en priorité**). Vérifie que ces corrections sont justes et complètes :
la portée de l'AC 5 énumère-t-elle **tout** ce qui meut le compte 1100 hors factures de Kesh
(lis `crates/kesh-db/src/repositories/opening_balances*.rs`, les contre-passations, l'import
fournisseurs n'y touche pas ?) ; les quatre passages du manuel sont-ils les seuls de la section et
du document (greppe `TTC`, `total dû`, `encours` dans le `.tex` ET le PDF aplati) ?

⛔ Pour chaque finding, cite la ligne exacte lue dans le fichier ACTUEL, et vérifie ton affirmation
par `grep -nF` avant de l'écrire.
