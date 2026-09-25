# Prompt — passe 1 de `bmad-code-review`, Story 25-3-a-2

*Versionné le 2026-09-25. Trois lentilles en contexte frais (Sonnet), orthogonales à l'auteur du
code (Opus). Protocole du workflow : Blind Hunter, Edge Case Hunter, Acceptance Auditor.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-2-annuler-reglement-fournisseur`. **Diff
revu** : `git diff main..HEAD` sur le code et la documentation, enregistré dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-a-2.diff`
(2621 lignes). Fiche : `_bmad-output/implementation-artifacts/25-3-a-2-annuler-reglement-fournisseur.md`.
Sœur **mergée** : 25-3-a-1 (annuler un règlement client) — cette story en réutilise le socle, la
queue commune des motifs et la leçon de sa revue (le verrou de l'exercice avant de juger).

Objet : **annuler le règlement d'une facture fournisseur** (`paid → open`) par contre-passation
datée du jour. Nouveau : `ReversalAuthority::SupplierSettlement` (le socle vérifie que l'écriture
est bien celle du RÈGLEMENT, jamais l'achat), `SettlementCancelBlocker::SupplierInvoiceNotPaid`,
`supplier_invoices::cancel_settlement_in_tx`, `payment_batches::last_confirmed_batch_for_invoice`
(historique, pour avertir d'un double paiement), la route
`POST /api/v1/supplier-invoices/{id}/settlement/cancel`, les champs de lecture de
`SupplierInvoiceResponse`, l'écran, et une sonde d'entrelacement mise en commun dans
`kesh_db::test_fixtures`.

## Lentille 1 — Blind Hunter (diff SEUL)

Tu ne lis **que** le fichier de diff. Revue adversariale générale : logique, erreurs, concurrence,
multi-tenant (`company_id` sur chaque requête, y compris dans `last_confirmed_batch_for_invoice`
et la sonde), tests qui ne prouvent rien, incohérences code / commentaires.

## Lentille 2 — Edge Case Hunter (diff + dépôt)

Chaque branche du code neuf, en lisant l'appelé et l'appelant. Priorités :
1. **L'autorité fournisseur** (`reverse_in_tx_inner`, étape 1-bis) : peut-elle être satisfaite
   pour une autre écriture que celle du règlement ? Que se passe-t-il si la colonne a déjà été
   vidée ? L'ordre « contre-passation AVANT l'`UPDATE` qui vide la colonne » est-il tenu ?
2. **La concurrence** : pour chaque lecture sans verrou qui décide d'un refus, quel verrou posé
   avant la rend sûre ? Ordre des verrous de `cancel_settlement_in_tx` face à `pay_in_tx`,
   `payment_batches::create_batch` / `confirm_batch` / `cancel_batch`, `fiscal_years::close` ;
   un interblocage est-il possible ? Une annulation concurrente d'un `confirm_batch` en cours ?
3. **`with_settlement_cancellation`** : appelé par le GET, `pay` et l'annulation — une seconde
   connexion du pool pendant la requête, un état lu hors transaction : défaut ou non ?
4. **La sonde partagée** (`test_fixtures::attendre_une_requete_en_cours`) : correcte, sûre, et
   le test client qui l'utilise désormais prouve-t-il toujours ce qu'il dit ?
5. Le frontend (`supplier-invoices/[id]/+page.svelte`) : états limites (facture annulée, payée
   sans champ calculé, double clic, refus au clic, confirmation refusée).

## Lentille 3 — Acceptance Auditor (diff + fiche + documentation)

Chaque AC (1 à 10) contre le diff ; les décomptes du Dev Agent Record **recomptés** (tests
ajoutés, clés i18n, `sitesTotal`, `CLES_RELEVEES`, registre de routes, libellés d'audit) ; le
manuel `docs/manual/fr/user-manual.tex` et son **PDF aplati**
(`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`), `docs/api-external.md`,
`CHANGELOG.md`, `README.md`, `docs/testing.md` (deux numéros de ligne recalés). ⚠️ C'est cette
story qui **ferme #414** : reste-t-il, dans le dépôt, une phrase qui dit que le côté fournisseur
ne sait pas annuler un règlement ?

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), `fichier:ligne`, **preuve** (code lu,
  commande et résultat), correction. Pour tout CRITICAL ou HIGH affirmant qu'une chose est absente
  ou présente : la commande `grep -nF` exécutée et son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `pdftotext` (vers le scratchpad) et
`cargo check` sont autorisés.
