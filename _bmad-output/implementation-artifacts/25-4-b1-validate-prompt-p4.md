# Prompt — validation P4 CIBLÉE, Story 25-4-b1 (le résiduel aux agrégats)

*Versionné le 2026-09-27. **Une lentille** (Sonnet), contexte frais. Passe **ciblée** sur la
remédiation de la P3 : la portée de l'AC 5 a été **réécrite par une règle** après trois passes qui
trouvaient chacune des cas hors de la liste précédente.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-b-residuel-aux-agregats`. Fiche :
`_bmad-output/implementation-artifacts/25-4-b1-residuel-aux-agregats.md`. Diff à relire :
`git diff 72255481 HEAD -- _bmad-output/implementation-artifacts/25-4-b1-residuel-aux-agregats.md`.
Issues sorties : `gh issue view 473`, `gh issue view 474`.

## La lentille

1. **La règle est-elle juste ?** Trop large (elle exclut un cas où l'égalité tient) ou trop étroite
   (un mouvement qu'elle autorise rompt l'égalité) ? Éprouve-la contre : la **dévalidation** d'une
   facture (l'écriture de vente disparaît), l'**annulation d'un règlement** (contre-passation +
   suppression de la ligne), l'**annulation d'un rapprochement** de facture, une facture **soldée**
   (`paid_at`) — hors du périmètre de la balance âgée mais ses mouvements restent au grand livre et
   s'annulent-ils exactement ? —, un **reste dû négatif** sur une facture validée.
2. **« n'a jamais changé dans les réglages »** : formulation testable ? Le test peut-il la tenir ?
3. **Le test prescrit** (quatre états vivants, `as_of` ≥ toutes les pièces) : prouve-t-il la règle ou
   passe-t-il par construction ? Quelle mutation de la requête de la balance âgée le ferait rougir ?
4. **T0** : les quatre sites cités existent-ils aux lignes dites, et la correction prescrite est-elle
   juste (`git show v0.12.0:crates/kesh-db/src/repositories/credit_notes.rs`) ? En existe-t-il un
   cinquième (greppe la **valeur** : `keshbackup`, `sauvegarde antérieure`, `import d'une sauvegarde`
   dans `crates/`, `docs/`, les fiches 25-4-*) ?
5. **La réserve du manuel** : exacte, lisible par un utilisateur, cohérente avec la règle ?
6. **Propagation** : le mot « quatre » (cas), les références aux handlers, #473/#474 — cohérents
   partout (fiche, fiche mère, `sprint-status.yaml`) ?

## Ce que tu rends

- **Findings** : sévérité, endroit, **preuve** (commande et résultat, code lu), correction.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`cargo check`.
