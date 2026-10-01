# Prompts — revue de code P1, Story 25-4-c3-a2 (le compte d'arrondi dans les plans livrés)

*Versionné le 2026-09-30. Trois lentilles en parallèle, **Sonnet**, contexte frais. Diff : le commit
d'implémentation, `git show 07661e85 (hors `_bmad-output` et PDF)`, écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/c3a2-p1.diff`.*

## Commun aux trois lentilles

**Rendu** : findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit exact (`fichier:ligne`),
**preuve** (code lu cité, commande et sortie), correction proposée. ⛔ **La liste des axes réellement
exercés ET de ceux qui ne l'ont pas été** — un rapport sans elle ne compte pas.

**Interdits** : n'écrire aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une
base — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout `git commit`/`push`/`add`/
`stash`/`reset`/`rebase`/`checkout`/`switch`/`worktree`, `sqlx migrate`, `cargo test`/`nextest`,
`npm run`, `npx playwright`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers le scratchpad, `cargo check`.

## Lentille 1 — Blind Hunter (`bmad-review-adversarial-general`)

Reçoit **le diff seul**, aucun contexte projet. Revue adversariale générale.
## Lentille 1 — Blind Hunter (`bmad-review-adversarial-general`)

Reçoit **le diff seul**, aucun contexte projet. Revue adversariale générale.

## Lentille 2 — Edge Case Hunter (`bmad-review-edge-case-hunter`)

Diff **et** lecture du dépôt.
- `rounding_account_from_chart` (`crates/kesh-db/src/repositories/company_invoice_settings.rs`) : chaque
  chemin qui rend `None` ; `org_type` stocké en base contre ce qu'attend `load_chart` (casse, valeurs) ; une
  erreur SQL propagée par `?` fait-elle échouer l'onboarding alors que le compte est facultatif ? Le
  `FOR UPDATE` sur la recherche par numéro : ordre des verrous par rapport aux recherches par rôle ; le
  lock tenu par la variante pool jusqu'à quand ?
- Tous les chemins qui créent une société et ses comptes (`grep -rn "bulk_create_from_chart\|insert_with_defaults" crates/`) :
  seed démo, onboarding démo et production, reprise ; l'un d'eux peut-il appeler `insert_with_defaults`
  **avant** que les comptes du plan existent (le compte n'est alors jamais désigné) ?
- `validate_chart` : le marqueur accepté sur un compte qui porte aussi un rôle ? Deux plans livrés modifiés
  en JSON — la ligne insérée est-elle au bon endroit (ordre, virgules, parent `6` présent dans les trois) ?
- Tests : prouvent-ils ce qu'ils annoncent (variante `_in_tx` réellement exercée, renumérotation réelle) ?
  Les tests modifiés de la c3-a1 gardent-ils leur force ?
- Sauvegarde / import : un plan importé ou restauré porte-t-il le marqueur (il n'est pas en base) — est-ce
  un problème ?

## Lentille 3 — Acceptance Auditor

Diff, fiche `_bmad-output/implementation-artifacts/25-4-c3-a2-compte-arrondi-plans.md`, lecture du dépôt.
Chaque AC (1 à 6) contre le code ; « Ce qu'il ne faut pas faire » (pas de rôle, pas de `6940` dans le Rust
applicatif, pas d'échec d'onboarding, pas de migration) ; le Dev Agent Record (affirme-t-il seulement ce qui
a tourné ? décomptes recomptés ?). ⛔ **Le manuel** : `docs/manual/fr/admin-manual.tex` (paragraphe *Compte
de différences d'arrondi*) et son **PDF aplati** (`pdftotext … | tr '\n' ' ' | tr -s ' '` — attention aux
ligatures « ﬀ », « ﬁ » qui cassent un `grep` naïf) : dit-il vrai sur le code et sur ce qui n'est pas encore
livré ? Le CHANGELOG dit-il vrai ?
