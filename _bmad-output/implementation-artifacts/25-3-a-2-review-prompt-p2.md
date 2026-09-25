# Prompt — passe 2 de `bmad-code-review`, Story 25-3-a-2

*Versionné le 2026-09-25. Trois lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku →
Opus. ⚠️ Haiku : les diffs sont fournis **aplatis**, un fichier chacun (pas de séquence de commits),
pour éviter la confusion d'indexation d'un diff multi-commit.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-2-annuler-reglement-fournisseur`, tête
`b6d041d5`. **Diffs revus**, dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/` :

- `25-3-a-2-p2.diff` — `git diff main..HEAD` complet (3000 lignes, PDF exclus) ;
- `25-3-a-2-p1-remediation.diff` — la seule remédiation de la passe 1 (`fae82d40..HEAD`, 411 lignes).

Fiche : `_bmad-output/implementation-artifacts/25-3-a-2-annuler-reglement-fournisseur.md` — son
Change Log (ligne `review P1`) dit ce que la passe 1 a trouvé et corrigé.

Objet : **annuler le règlement d'une facture fournisseur** (`paid → open`) par contre-passation
datée du jour (cf. prompt de la passe 1, `25-3-a-2-review-prompt-p1.md`, pour le détail). ⛔ **Motif
mesuré sur ce projet : la sévérité se déplace vers ce qu'on vient d'écrire.** La remédiation de la
passe 1 est le premier suspect :

- `supplier_invoices::get_settlement_view` (une transaction de **lecture** pour un instantané) et
  `SupplierInvoiceResponse::load_with_settlement_cancellation`, qui sert désormais le GET, `pay` et
  l'annulation en **relisant** la facture après l'écriture ;
- la jointure ajoutée à `payment_batches::last_confirmed_batch_for_invoice` ;
- la **propagation** au côté client : `list_invoice_settlements_handler` (`routes/invoices.rs`) lit
  la liste et les motifs dans une transaction, et `invoice_settlements::list_for_invoice` accepte
  tout exécuteur.

## Lentille 1 — Blind Hunter (diffs SEULS)

Tu ne lis **que** les deux fichiers de diff. Revue adversariale générale : logique, erreurs,
concurrence, multi-tenant (`company_id` sur chaque requête), tests qui ne prouvent rien,
incohérences code / commentaires.

## Lentille 2 — Edge Case Hunter (diffs + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **L'instantané** : l'affirmation « sous `REPEATABLE READ`, toutes les lectures non
   verrouillantes d'une transaction voient l'instantané pris à la première » est-elle vraie pour
   **chaque** requête lancée par `get_settlement_view` — y compris celles de la queue commune
   (`settlement_cancellation::settlement_entry_cancel_blocker`, `fiscal_years::has_open_covering_date`)
   ? L'une d'elles est-elle verrouillante, ou change-t-elle l'isolation ?
2. **La relecture après écriture** : `pay` et l'annulation répondent désormais avec la facture
   **relue** et non celle que la transaction a produite. Qu'est-ce qui change pour l'écran
   (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`, verrou optimiste `version`),
   pour un client de l'API ? Une facture supprimée entre-temps (possible ?) → 404 après une
   écriture réussie : défaut ou non ?
3. **La propagation client** : une transaction ouverte sans `COMMIT` sur un chemin d'erreur `?`
   — que fait `sqlx` au `drop` ? Le pool, l'ordre des verrous, un interblocage ?
4. Les autres appelants de `list_for_invoice` (`grep -rn "list_for_invoice(" crates`) compilent-ils
   toujours avec la même sémantique ?

## Lentille 3 — Acceptance Auditor (diffs + fiche + documentation)

Chaque AC (1 à 10) contre le diff ; les décomptes du Dev Agent Record et du Change Log
**recomptés** depuis la source (tests ajoutés, clés i18n, registre de routes, libellés d'audit,
2448 tests du gate) ; le manuel `docs/manual/fr/user-manual.tex` et son **PDF aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), `docs/api-external.md`,
`CHANGELOG.md`, `README.md`. La phrase ajoutée à `docs/api-external.md` par la passe 1 est-elle
vraie ? Reste-t-il dans le dépôt une phrase qui dit que le côté fournisseur ne sait pas annuler un
règlement (cette story **ferme #414**) ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve**, correction.
  ⛔ Pour tout CRITICAL ou HIGH affirmant qu'une chose est **absente** ou **présente** dans un
  fichier : la commande `grep -nF "<chaîne>" <fichier>` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Un « 0 finding »
  sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
