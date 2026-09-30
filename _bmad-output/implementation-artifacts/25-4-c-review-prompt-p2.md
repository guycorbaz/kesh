# Prompts — revue de code P2, Story 25-4-c (le résiduel au rapprochement)

*Versionné le 2026-09-30. Deux lentilles en parallèle (Blind Hunter, Edge Case Hunter), **Haiku 4.5**, contexte frais. Diff : l'implémentation et la remédiation P1
`git diff 7aa56d2f 8d33460c -- . ':(exclude)*.pdf' ':(exclude)_bmad-output'` (diff UNIQUE aplati, implémentation + remédiation P1 — règle Haiku du CLAUDE.md), écrit dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/25-4-c-p2.diff`.*

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

Reçoit **le diff seul**, aucun contexte projet. Revue adversariale générale : défauts logiques, SQL,
cas limites, tests qui ne prouvent pas ce qu'ils annoncent, incohérences entre commentaires et code.

## Lentille 2 — Edge Case Hunter (`bmad-review-edge-case-hunter`)

Reçoit le diff **et** l'accès en lecture au dépôt. Parcourir chaque branche et chaque frontière :

- la requête de `find_unpaid_invoices_for_window` (`crates/kesh-db/src/repositories/reconciliation.rs`) :
  ambiguïté de colonnes avec les trois tables dérivées, `HAVING` sans `GROUP BY` sur des `LEFT JOIN`,
  facture sans ligne, avoir émis (facture `cancelled`), reste négatif, `LIMIT 50` ;
- **tous les lecteurs** de `UnpaidInvoiceCandidate` et de `invoice_amount` / `invoiceAmount` —
  `grep -rn "total_ttc\|amount_due\|invoiceAmount\|invoiceTotalTtc\|UnpaidInvoiceCandidate" crates/ frontend/src` ;
- le re-score à l'acceptation (`accept_one_invoice`, `crates/kesh-api/src/routes/reconciliation.rs`) :
  ordre des gardes (score avant trop-perçu), reste nul ou négatif, facture sans ligne ;
- `invoiceTotalTtc` : égalité `Decimal` à échelles différentes (`600.0000` contre `600`), sérialisation ;
- le frontend (`ReconciliationProposals.svelte`) : rendu, espaces, i18n (4 catalogues, clé
  `reconciliation-labels-amount-due-of`), garde `i18n-keys.test.ts` ;
- le Playwright (`frontend/tests/e2e/reconciliation-amount-due.spec.ts`) : dépendance à l'état partagé
  de la base, collisions de montants, exercice couvrant le 2026-05-10, sélection du compte bancaire.

## Lentille 3 — Acceptance Auditor

Reçoit le diff, la fiche `_bmad-output/implementation-artifacts/25-4-c-residuel-au-rapprochement.md`
et l'accès en lecture au dépôt. Vérifier chaque AC (1 à 6) contre le code, les « Ce qu'il ne faut pas
faire » (aucun `FOR UPDATE`, aucun arrondi, forme jointe, score binaire), le Dev Agent Record
(affirme-t-il seulement ce qui a tourné ? décomptes recomptés depuis la source ?).

⛔ **Le manuel** : `docs/manual/fr/user-manual.tex` (section *Réconciliation bancaire* et le
paragraphe *Ce que réclame un rappel*) et `docs/manual/fr/marketing-brochure.tex:167` — chaque
affirmation contre le code (critères, poids, fenêtre, tolérance, « créditrice, en CHF », « pas de
seuil ni d'acceptation automatique », QR du rappel, versement reste + frais). **Contrôler le PDF**,
aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et la brochure),
vers le scratchpad. Le CHANGELOG `[0.12.1]` dit-il vrai ?

## Spécifique à la P2

- La lentille 3 (Acceptance Auditor) n'est pas rejouée : elle a rendu 0 finding en P1, preuves à l'appui.
- ⚠️ **Toute affirmation qu'un code est ABSENT, ou qu'un anti-pattern est PRÉSENT, doit citer le
  `grep -nF` qui le prouve, avec sa sortie** — lu sur le fichier courant du dépôt (lentille 2) ou sur
  le diff (lentille 1), jamais sur un numéro de ligne de hunk.
- Poids particulier sur ce qu'a écrit la remédiation P1 : l'emplacement de l'entrée #420 dans
  `CHANGELOG.md` (section `### Fixed` de `[0.12.1]`, rendu Markdown), les libellés EN/DE de
  `reconciliation-labels-amount-due-of`, et la boucle de remplacements du Playwright (la contrepartie
  `<Nm>` = le nom du contact : le score de contact rend-il le test moins probant ? la fixture
  contient-elle chaque motif exactement là où on l'attend ?).
