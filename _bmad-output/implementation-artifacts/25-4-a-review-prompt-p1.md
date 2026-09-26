# Prompt — revue de code P1, Story 25-4-a (le résiduel juste)

*Versionné le 2026-09-27. **Une lentille** (Sonnet), contexte frais.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-4-propager-le-residuel`. Diff à relire :
`git diff ab9b9448 9bf2bb2b` (commit de dev, après la validation close). Fiche :
`_bmad-output/implementation-artifacts/25-4-a-residuel-juste.md` (AC, Dev Notes, Dev Agent Record).
Issues : `gh issue view 455`, `gh issue view 456`. Règles : `CLAUDE.md`. **Arbitrages de Guy** (fiche
§ Arbitrage, fiche mère `25-4-propager-le-residuel.md`) : ne pas les contester.

## Axes — tous obligatoires

1. **SQL** : les quatre constantes produites par `line_ttc_sql!` (`invoices.rs`,
   `invoice_settlements.rs`) — développe-les à la main et vérifie le texte exact (espaces, alias,
   `\` de continuation dans un `concat!`), l'égalité **octet pour octet** des deux constantes TTC
   facture avec leur forme d'avant (`git show ab9b9448:crates/kesh-db/src/repositories/invoices.rs`),
   et l'agrégation de la forme jointe de l'avoir (GROUP BY, jointure, filtre `issued`).
2. **La garde** (`credit_notes.rs`) : ordre verrou → lecture verrouillante, scoping `company_id`,
   chemins d'erreur, rien écrit en cas de refus. Les tests prouvent-ils ce qu'ils annoncent, ou
   peuvent-ils passer à vide (surtout l'entrelacement : que se passe-t-il si
   `attendre_une_requete_en_cours` voit une AUTRE requête que celle de l'avoir) ?
3. **Tous les lecteurs** des grandeurs changées : `grep -rn "INVOICE_CREDITED\|amount_due\|INVOICE_TTC" crates/`
   — un appelant change-t-il de valeur sans test ? Et tout appelant de `create_credit_note`, ou qui
   matchait `IllegalStateTransition` pour le cas « payée » (API, frontend, tests) ?
4. **La fixture `monter`** : le gabarit tient-il pour les quinze cas ? Un motif lit-il la facture
   auxiliaire ?
5. **Le manuel dit-il vrai ?** `docs/manual/fr/user-manual.tex` — les passages modifiés (§ Avoirs, les
   deux limites, les deux « cas hérité ») contre le code ; **le PDF aplati**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, vers
   `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`),
   `??` compris. Le renvoi `\S\ref{sec:reglement-client}` pointe-t-il où il faut ? Le CHANGELOG
   décrit-il juste (l'API, le bouton) ?
6. **i18n** : la clé ×4 — sens, registre du glossaire (`docs/i18n-glossaire.md` § Registre), terme
   « avoir » du glossaire.
7. **Frontend** : le bouton, les commentaires, les `data-testid` ; les Vitest tuent-ils vraiment m6 ?
8. **Dev Agent Record** : chaque décompte (tests, mutations, gates) **recompté** depuis la source.

## Ce que tu rends

- **Findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne, **preuve** (commande et
  résultat, code lu), correction proposée.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`. Autorisés : lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`,
`pdftotext` vers le scratchpad, `cargo check`, `cargo expand` s'il est installé.
