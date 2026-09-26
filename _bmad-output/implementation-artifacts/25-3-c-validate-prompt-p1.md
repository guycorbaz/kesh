# Prompt — passe 1 de `bmad-create-story validate`, Story 25-3-c

*Versionné le 2026-09-26. Deux lentilles en contexte frais (Sonnet), orthogonales à l'auteur de la
fiche (Opus 5.5) — cycle Sonnet → Haiku → Opus.*

Ton objet est `_bmad-output/implementation-artifacts/25-3-c-annuler-facture-fournisseur.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-c-annuler-facture-fournisseur` (le code est celui de
`main` à `0e4c2682`). Elle ferme **#454** (`gh issue view 454`) : annuler une facture fournisseur
**même payée**, en faisant passer `supplier_invoices::cancel` par le socle de contre-passation
(`journal_entries::reverse_owned_in_tx`) au lieu de sa réécriture à la main. Sœurs mergées dont elle
réutilise le code : 25-3-a-2 (règlement fournisseur, `cancel_settlement_in_tx`, autorité
`SupplierSettlement`, vue `get_settlement_view`) et 25-3-b (famille de textes propre, leçon
`REPEATABLE READ`). Sources des arbitrages : `25-3-a-annuler-reglement.md` (§ Arbitrages, Change Log
du 2026-09-24).

⚠️ **Ne conteste pas les arbitrages de Guy** — « dans tous les cas sauf exercice de l'achat clos »
(2026-09-24) et **Q1 du 2026-09-26** : le règlement d'une facture payée annulée est **détaché**
(colonnes remises à `NULL`, écriture de règlement laissée au grand livre, libre). Conteste leur
**mise en œuvre**.

## Lentille A — la fiche contre le code

1. **« Ce que fait `cancel` aujourd'hui »** : chaque affirmation est-elle **vraie** ? Relis
   `supplier_invoices.rs` (`cancel`, `guard_not_in_generated_batch`, `pay`, `cancel_settlement_in_tx`,
   `supplier_settlement_cancel_blocker`, `get_settlement_view`), `journal_entries.rs`
   (`reversal_blockers`, `ReversalAuthority`, `reverse_in_tx_inner`, `create_in_tx` et la recopie
   du projet par ligne), `settlement_cancellation.rs`, `errors.rs` des deux crates.
2. **L'autorité `SupplierPurchase` (AC 1)** : la factorisation de l'étape 1-bis est-elle
   implémentable sans affaiblir `SupplierSettlement` ? Le test prescrit peut-il échouer ?
3. **La queue appliquée à l'écriture d'ACHAT (AC 2)** : `settlement_entry_cancel_blocker` est-il
   réellement neutre vis-à-vis de la pièce ? Ses doc-comments, son nom et ses textes serveur
   parlent de « règlement » — cela crée-t-il un défaut ? Le rang 3 est-il vraiment inatteignable
   pour une écriture d'achat (`grep -rn matched_entry_id crates`) ? La précédence (lot en dernier)
   est-elle cohérente entre lecture et clic, compte tenu du partage « le geste refuse 1, 2, 6 ; le
   socle refuse 3, 4, 5 » ? Un cas où l'écran annonce un motif et le clic en rend un autre ?
4. **Le geste (AC 3)** : l'ordre des verrous et des lectures est-il juste sous `REPEATABLE READ` ?
   Contre `payment_batches::create_batch`, `confirm_batch`, `pay`, `cancel_settlement_in_tx`,
   `fiscal_years::close` / `reopen` : interblocage ou fenêtre ? Vider les colonnes de règlement
   (Q1) casse-t-il quelque chose — contrainte, FK, lecture qui suppose `cancelled ⇒ colonnes`,
   export, `payment_batch_items`, rapports (`kesh-report`), tableau de bord, écrans de lots ?
5. **Exactitude** : chaque `fichier:ligne`, nom de fonction ou de test, code d'erreur, clé FTL —
   depuis la source.

## Lentille B — relecture adversariale complète, à froid

Lis la fiche comme le développeur qui devra l'implémenter demain, sans autre contexte. Pour chaque
AC : peut-on l'implémenter **sans deviner** ? Peut-on le **vérifier** ? Qu'est-ce qui cassera
ailleurs (tests existants de `supplier_invoices_repository.rs`, `supplier_settlement_cancel_e2e.rs`,
`payment_batches_repository.rs`, vitest `supplier-settlement-page.test.ts`, Playwright
`supplier-invoices.spec.ts`) ? Les champs de lecture (AC 5) : nommage, discipline `null`, les
appelants. L'écran (AC 7) : conditions d'affichage complètes pour `open` / `paid` / `cancelled`, rôle,
refus local. Les textes (AC 6, 8) : la liste des sites est-elle **close** (grep de la clé, du code
**et** de la phrase, quatre locales, replis serveur et Svelte) ? `lint-i18n-ownership`
(`frontend/scripts/lint-i18n-ownership.js`) accepte-t-il les clés prévues ? Les tests (AC 9) :
l'un passerait-il **à vide**, ou porte-t-il sur un état impossible ? Le **manuel**
(`docs/manual/fr/user-manual.tex` et son **PDF aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : la liste de l'AC 10
est-elle close — reste-t-il une phrase qui dit qu'une facture fournisseur payée ne s'annule pas ?
`docs/api-external.md`, `README.md`, `CHANGELOG.md`. Périmètre : **une** story (règle de découpage :
plus de 5 modules) ?

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (code
  lu, commande et résultat), ce qu'il faut changer.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`/`switch`, `sqlx migrate`, `cargo test`/`cargo nextest`,
`npm run`, `npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext`
(vers `/tmp/claude-1000/-home-gcorbaz-devel-kesh/ca5ce2e2-67a3-4eeb-817f-c2de35620a1c/scratchpad/`)
et `cargo check` sont autorisés.
