# Story 33-6 : le règlement n'est plus annoncé en échec après son commit (#450)

Status: review

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Release **0.17.5**, Epic 33. **Aucune migration.** Registre : `e33-choix-autonomes-e334.md`,
C-e334-8 à 10. Commits en `refs` ; à la PR : `closes #450`, `refs #768`, `refs #773`.*

## Story

En tant que Comptable ou intégrateur de l'API, je veux que la réponse d'un règlement dise ce qui a été enregistré, sans seconde
lecture après le commit, pour qu'aucun 404 ni 500 n'annonce en échec un règlement réussi, et qu'on ne le rejoue pas.

## Ce que le code dit (prémisses établies)

- **Le défaut est là** : `routes/invoices.rs:1153-1157`, après le `settle_invoice` commité (`invoice_settlements_write.rs:392`), relit la
  facture sur le pool, rend 404 si elle manque, puis relit `amount_settled`.
- ⚠️ **Prémisse corrigée — « le seul qui reste » est faux.** #450 dit que `settle_invoice_handler` est le dernier site du fichier. Il en
  reste **trois** dans `routes/invoices.rs` : règlement (`:1153`), **solde** (`write_off_invoice_handler`, `:1228-1233`, qui relit en plus
  `amount_due`) et **annulation de règlement** (`cancel_invoice_settlement_handler`, `:1357-1361`). Les trois écrivent au journal.
- **Les sorties de tx** : `SettlementOutcome` (`:39-45`), `WriteOffOutcome` (`:410-413`), `SettlementCancellation` (`:797-802`), tous trois sans la
  facture. `cancel_settlement` enveloppe `cancel_settlement_in_tx` (`:1030-1050`), réemployée par `reconciliation_cancel` : ce dernier chemin
  n'a pas à lire la facture, d'où une lecture **dans l'enveloppe**, non dans la forme `_in_tx` (C-e334-9).
- **Le patron** : `ValidatedInvoice` (`repositories/invoices.rs:1734-1738`). `find_by_id_with_lines` (`:829-855`) ne prend qu'un `&MySqlPool` ;
  `amount_settled` et `amount_due` (`invoice_settlements.rs:552`) prennent déjà un `Executor`.
- **L'ordre dans la tx compte** : `version` s'incrémente (`:~330`) et le lettrage se synchronise (`sync_invoice_in_tx`, `:~350`) avant l'audit. La
  lecture se place **juste avant `tx.commit()`** ; sous `REPEATABLE READ`, une transaction voit ses propres écritures, et la ligne `invoices` est
  sous `FOR UPDATE` depuis l'étape (1).
- ⚠️ **Reproduire le 404 de façon déterministe n'est pas possible** sans crochet de production : la fenêtre est entre deux requêtes SQL. Le rouge
  vient de la compilation, et la garantie de la garde lexicale (T3) ; c'est écrit ici plutôt que simulé (C-e334-10).

## Même famille — ce qui se traite, ce qui reste dit

| Site | Décision |
|---|---|
| #768 point 1 : `decider_envoi` relit après commit (`routes/mcp_envois.rs:262-267`), un 500 annonce en échec une décision écrite | **Traité** : lecture dans la tx avant le commit. `mcp_sends::trouver` prend un `&MySqlPool` (`kesh-db/src/repositories/mcp_sends.rs:235-239`) : forme `trouver_in_tx`, réemployée par `trouver` (DRY), et ⚠️ `kesh-db` touché, gate complet |
| #773 point 3 : si la réponse d'une décision se perd, l'écran ne propose pas *Recharger* | **Traité** : le point 1 ne le rend pas sans objet (délai, coupure réseau). `erreurDeDecision` (`mcp-sends-format.ts:81-103`), hors `ApiError` : `recharger: true`, qui montre le bouton existant (`McpSendDetail.svelte:295-296`) ; une ligne et un test Vitest |
| Facture fournisseur : `load_with_settlement_cancellation` après `pay`, `cancel`, `validate`, `unvalidate`, etc. (`supplier_invoices.rs:776,881,917,946,970,1002`, `imported_supplier_invoices.rs:141,595,700`) | **Non traité, à tracer** : neuf sites, et une lecture à quatre champs calculés ; déborde, et 33-1a/b récrit ce fichier. **Issue demandée à l'orchestrateur** (C-e334-8) |
| Le reste de #768 (points 2 à 7) et de #773 | Autre nature : les deux issues restent ouvertes, chacune commentée de ce qui a été fait |

## Acceptance Criteria

1. **AC-1** : `SettlementOutcome` porte `invoice: Invoice`, `lines: Vec<InvoiceLine>`, `amount_settled: Decimal`, lus dans la transaction, après la
   dernière écriture et avant le commit. Mêmes champs, plus `amount_due`, pour `WriteOffOutcome`. Le retour de `cancel_settlement` gagne `invoice`, `lines` et `amount_settled` : `amount_due_after` y est déjà (`invoice_settlements_write.rs:797-802`).
2. **AC-2 — garde** : les corps de `settle_invoice_handler`, `write_off_invoice_handler` et `cancel_invoice_settlement_handler` ne contiennent ni
   `find_by_id_with_lines`, ni `amount_settled(`, ni `amount_due(`, ni `state.pool` après l'appel d'écriture (garde lexicale T3, avec **sonde positive**).
3. **AC-3 — ce qui est rendu est ce qui est écrit** : pour les trois routes, la réponse HTTP égale champ à champ un `GET /invoices/{id}` qui la suit
   (`version`, `paidAt`, `amountSettled`, `amountDue`, `updatedAt`, lignes) : règlement partiel, règlement total, écart d'arrondi, solde, annulation.
4. **AC-4 — le rejeu sur interblocage reste sûr** : la lecture étant dans la fermeture de `retry_on_deadlock`, la réponse vient de la tentative qui a
   réussi. Comportement des refus (statut, trop-perçu, verrou) inchangé : les tests existants passent sans modification.
5. **AC-5** : aucun appelant des trois fonctions ne change de comportement (`grep` des sites en `tests/`, `reconciliation`, `kesh-report`), `reconciliation_cancel`
   ne lit rien de plus.
6. **AC-6** : `decider_envoi` : la réponse vient de la tx ; égale à `GET /admin/mcp/sends/{id}` qui la suit ; ses refus inchangés.
7. **AC-7** : l'écran propose *Recharger* quand la décision échoue sans réponse du serveur ; une `ApiError` garde son comportement.
8. **AC-8** : une défaillance de la lecture interne fait échouer et annule la transaction : **pas de commit sans réponse**. Écrit au doc-comment, c'est ce
   qui ferme la classe.

## Tâches

- [x] **T1 — rouge** : `tests/invoice_settlement_response_e2e.rs`, AC-3, et tests `kesh-db` : `settle_invoice(...)` rend `outcome.invoice.version` égal à la ligne
  relue, `outcome.amount_settled` égal au `SUM` : rouges à la compilation, écrits d'abord.
- [x] **T2** : forme `find_with_lines_in_tx` à `repositories/invoices.rs`, réemployée par `find_by_id_with_lines` (DRY) ; sorties des trois fonctions ; trois handlers.
  ⚠️ `kesh-db` touché : **gate complet même en cours de boucle** (`CLAUDE.md`, exception `kesh-db`).
- [x] **T3** : garde lexicale dans le même fichier de test.
- [x] **T4** : `mcp_envois.rs`, `mcp_sends` ; `mcp-sends-format.ts`, `McpSendDetail.test.ts`.
- [x] **T5** : CHANGELOG *Corrigé* (`refs #450`, `refs #768`, `refs #773`).

### Mutations nommées (P2)

- **M1** : la lecture d'`amount_settled` déplacée avant l'`INSERT` du règlement → rouge AC-3.
- **M2** : la lecture de la facture déplacée avant l'incrément de `version` → rouge AC-3 (écart de version, celui qui ferait échouer le geste suivant en 409).
- **M3** : `find_by_id_with_lines` réintroduit dans un handler → rouge AC-2. **M4** : `outcome.lines` rendu vide → rouge AC-3.
- **M5** : l'`amount_due` du solde lu avant son écriture (le reste d'avant) → rouge AC-3, solde.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, E2E complet au dernier commit de code. `sqlx` : aucune migration, aucun schéma.

## Manuels

Aucun comportement visible ne change hors la panne. **Prompt de revue : le dire et le vérifier** : `docs/api-external.md:503-521` et les manuels ne décrivent pas de 404
après règlement (`/usr/bin/grep -n 'règlement.*404' docs/api-external.md docs/manual/fr/*.tex`) ; si le manuel admin § MCP décrit l'écran d'envoi perdu, l'aligner et régénérer le PDF.

## Recoupements

`routes/invoices.rs` avec 33-4 (champs de date, autres lignes) ; `McpSendDetail.svelte` : aucun. Pas de fichier commun avec 33-1 : le fournisseur est écarté ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).
- 2026-11-06 : validation P1 (Sonnet 5.5) : aucun finding, lentilles R et F. Fiche inchangée, hors cette entrée (Opus 5.5).
- 2026-11-06 : validation P2 (Opus 5.5), remédiation (Opus 5.5). Un LOW. **L-R6-1** : tranché, forme `_in_tx` (lu : `mcp_sends.rs:235-266`). Prémisse du rapport nuancée : `trouver` ne fait qu'**une** requête, et non plusieurs ; le geste reste petit, mais touche `kesh-db`. « Quatre lignes » retiré.
- 2026-11-06 : validation P3 (Sonnet 5.5, lentilles R et F), remédiation (Opus 5.5). Deux LOW, aucun réfuté. **L-R6-1** : tranché, traité, plus de condition à l'AC-7 ni à T4 (lus : `McpSendDetail.svelte:106-109,295-296`, `mcp-sends-format.ts:81-103`) ; le site est dans le format, non dans le composant. **L-R6-2** : AC-1 et M5 (lus : `invoice_settlements_write.rs:797-802`, `invoices.rs:1228-1233,1357-1361`). ⚠️ M5 changée plus que le rapport ne le demandait : un solde solde **tout** le reste (`:415`), si bien que `Decimal::ZERO` y est la valeur juste, et la mutation d'origine, placée sur le solde, aurait survécu. Mutations : 5, inchangées.

- 2026-11-06 : développement (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e335`, commit de code `25b0ed3d`. **Étape 0** : les deux LOW de la P3 sont déjà portés par la fiche (lus : `mcp-sends-format.ts:81-103`, `invoice_settlements_write.rs:797-802`) ; aucun point « à établir ». **Prémisse corrigée, C-e334-33** : la fiche et sa P3 parlaient d'une réponse perdue « hors `ApiError` ». Lu : `api-client.ts:338-347`, `toFetchApiError` lève une `ApiError` de statut 0. Le correctif tel qu'écrit n'aurait donc jamais servi sur une vraie coupure : la branche vise `!isApiError(err) || err.status === 0`. Une capture et une phrase du manuel d'administration ajoutées, l'état étant visible. Le retour de `cancel_settlement` est un type neuf, `CancelledSettlement`, `cancel_settlement_in_tx` gardant le sien.

- 2026-11-07 : remédiation de la revue P1 (Opus 5.5), commits `737e4c57`, `80cf06cb`. Passe 1 (trois lentilles) : 0 CRITICAL, 0 HIGH, des MEDIUM. Sur décision de l'orchestrateur, tout ce qui suit est traité ici :
  - **E-4 = A-4, le rejeu sur réponse perdue** (P1 à la lettre du barème) : prémisse lue, `invoices/[id]/+page.svelte` et `invoices/due-dates/+page.svelte` sont les deux seuls appelants de `settleInvoice` (`git grep`). Sur `!isApiError(err) || isUnknownOutcome(err)` : dialogue fermé, relecture, avis ; bouton indisponible jusqu'à une relecture entière réussie, *Relire*. **Côté fournisseur, pas de double écriture** : `paySupplierInvoice` ne porte pas de montant, et un second envoi est refusé par `inv.status != "open"` (`supplier_invoices.rs:1747`) ; seul manque l'affichage après coup (issue demandée). CHANGELOG récrit. C-e334-36.
  - **E-2** : `isUnknownOutcome`, 502 et 504 compris ; **B5** : le message dit pourquoi *Recharger* ; **B4 = E-3** : ni l'échec du GET, ni un GET réussi ne perdent le texte sans le dire ; **E-9** : une réponse vide est une issue inconnue.
  - **B2, B3, A-2, A-3, E-5, E-6, E-7** : la garde lexicale, qui énumérait des formes, cède la place à une garde de structure lue par `syn`, sur l'ensemble clos « aucun `.await` après l'écriture ni après le commit, la lecture avant lui », pour les trois handlers, les trois fonctions et `decider_envoi`, sonde passant par le même découpage. Égalités MCP pour une version modifiée et un refus ; annulation partielle (E-11).
  - **B1 = E-1 = A-1** : `admin-manual.tex:3437` récrit. **A-5, B7, E-8** : doc-comments. **A-6** : `R-read-before-writeoff` mord sur le solde lui-même. **A-7** : File List. **E-10, A-8** : l'échec du `COMMIT` reste une issue inconnue, dit au manuel.
  - La capture a montré l'avis écrasé dans la rangée des boutons, et une garde de replis (`i18n-repli-divergent-actif.test.ts`) a rougi au gate de `737e4c57` : `80cf06cb`, d'où un rejeu complet.
  - **Écart de méthode** : la garde Rust et les ajouts `mcp_envois_e2e.rs` ont été posés par un script Python (consigne 14) ; diff relu.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5.

### Completion Notes List

- **Rouges d'abord** (`kesh-gate-logs/e335/rouges.log`) : tests `kesh-db` rouges **à la compilation** (`E0609`, champs `invoice`, `lines`, `amount_settled`, `amount_due` absents) ; côté API, avant le correctif, 7 tests, 6 passés, 1 rouge, la garde `the_three_handlers_read_nothing_after_the_write`. ⚠️ **Les cinq tests d'égalité passaient déjà avant le correctif, et c'est attendu** : le comportement nominal ne change pas, le 404 n'étant pas reproductible (C-e334-10). Ils prouvent que la lecture dans la transaction rend le même état, et les mutations M1, M2, M4, M5 prouvent qu'ils mordent.
- **Mutations** (`e335/mut/synthese-33-6.log`), cinq sur cinq tuées. M1 : `la_sortie_du_reglement_porte_la_facture_relue` et les trois tests HTTP de règlement. M2 : les tests HTTP dont `a_cancellation_returns_what_was_written`, et la sortie `kesh-db`. M3 : `the_three_handlers_read_nothing_after_the_write`. M4 : les trois tests HTTP de règlement. M5 : `a_write_off_returns_what_was_written`, `la_sortie_du_solde_porte_la_facture_relue`.
- **Gates au commit `25b0ed3d`** (`e335/33-6/`) : `test-fast.sh --ci` vert, 4823 exécutés, 4823 passés, 12 ignorés ; `cargo build --workspace --all-targets` vert ; `npm run check` 0 erreur ; `lint-i18n-ownership` vert ; Vitest 2699 tests, 218 fichiers ; `npm run build` vert. **E2E complet** sur `kesh_e2e_e335` recréée : 377 tests, 352 passés, 18 ignorés, **7 échecs, tous de la KF-029** (`mode-expert:26,41`, `onboarding-path-b:65,92`, `onboarding:57,77,150`), 16,2 min.
- **Capture** : `e335/captures/33-6-reponse-perdue-recharger.png`, réponses d'API simulées (requête de décision interrompue) sur base jetable, thème clair.
- **Manuels** : `admin-manual.tex` (*Envoyer tel quel*), PDF régénéré, contrôlé aplati (« quand la réponse du serveur se perd » lu). `user-manual` et `api-external.md` : aucun 404 après règlement décrit (`:519` est le 404 d'une facture réellement introuvable). PDF de la brochure et du manuel utilisateur, régénérés sans changement de source, rétablis.
- **AC-5** : `cancel_settlement_in_tx` et `reconciliation_cancel` ne lisent rien de plus ; les appelants `tests/` compilent sans modification, leurs champs étant conservés.
- **Non traité, à tracer** : neuf sites de relecture après commit côté fournisseur (issue demandée, C-e334-8). Une réponse 500 ne propose pas *Recharger* : depuis le correctif, un 500 de la décision annule l'écriture, mais un proxy qui rendrait 502 ou 504 sur réponse perdue resterait sans bouton : à dire au commentaire de #773.
- **Écart de méthode** : aucun fichier de code n'a été écrit par script hors des mutations.

- **Remédiation P1, rouges** (`e335/rouges.log`) : Vitest, 20 rouges sur 57, dont les cas d'issue inconnue des deux écrans et de `McpSendDetail` ; Rust, 36 passés sur 36, ces preuves-là (garde `syn`, égalités) ne rougissant que sous mutation : c'est écrit ici pour ne pas passer pour un rouge.
- **Mutations P1** (`e335/mut/synthese-33-6-p2.log`), seize sur seize tuées : `R-decider`, `R-settle`, `R-writeoff`, `R-cancel` (relecture remise après le commit), `R-service`, `R-mapped` (formes que l'ancienne garde laissait passer), `R-read-before-writeoff`, `V-page-condition`, `V-page-open`, `V-page-list`, `V-due-condition`, `V-due-open`, `V-mcp-rule`, `V-mcp-screen`, `V-mcp-text`, `V-mcp-silent`.
- **Gates à `80cf06cb`** (`e335/33-6-p1b/`) : `test-fast.sh --ci` 4825 passés, 12 ignorés ; build vert ; `check`, `lint-i18n-ownership`, `npm run build` verts ; Vitest 2716, 218 fichiers ; **E2E complet** 379 tests, 354 passés, 18 ignorés, 7 échecs tous KF-029, 15,9 min, les deux tests `invoices-settle-unknown-outcome.spec.ts` passant.
- **Captures** (`e335/captures/33-6-p1-*.png`) : fiche à l'issue inconnue, échéancier en relecture échouée puis relu, texte non repris.
- **Manuels** : `admin-manual` et `user-manual`, PDF contrôlés aplatis (« Si la réponse se perd » lu, « annoncée en échec alors » absent) ; brochure rétablie.

### File List

*(Ajouts de la remédiation, et oublis de la première liste signalés par A-7.)* `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`, leurs tests, `frontend/src/lib/features/mcp-sends/McpSendDetail.svelte`, `mcp-sends-i18n.test.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-repli-divergent-actif.test.ts`, `frontend/tests/e2e/invoices-settle-unknown-outcome.spec.ts` (neuf), `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`, `docs/manual/fr/user-manual.{tex,pdf}`, `_bmad-output/implementation-artifacts/e33-choix-autonomes-e334.md`, `sprint-status.yaml`, `33-6-reglement-apres-commit-review-prompt-p2.md` (neuf).


`crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `crates/kesh-db/src/repositories/invoices.rs`, `crates/kesh-db/src/repositories/mcp_sends.rs`, `crates/kesh-db/tests/invoice_settlement.rs`, `crates/kesh-db/tests/invoice_write_off.rs`, `crates/kesh-api/src/routes/invoices.rs`, `crates/kesh-api/src/routes/mcp_envois.rs`, `crates/kesh-api/tests/invoice_settlement_response_e2e.rs` (neuf), `crates/kesh-api/tests/mcp_envois_e2e.rs`, `frontend/src/lib/features/mcp-sends/mcp-sends-format.ts`, `frontend/src/lib/features/mcp-sends/McpSendDetail.test.ts`, `docs/manual/fr/admin-manual.{tex,pdf}`, `CHANGELOG.md`, `_bmad-output/implementation-artifacts/33-6-reglement-apres-commit-review-prompt-p1.md` (neuf).
