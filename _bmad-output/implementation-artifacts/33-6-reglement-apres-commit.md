# Story 33-6 : le règlement n'est plus annoncé en échec après son commit (#450)

Status: ready-for-dev

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
| #768 point 1 : `decider_envoi` relit après commit (`routes/mcp_envois.rs:262-267`), un 500 annonce en échec une décision écrite | **Traité** : quatre lignes, lecture dans la tx avant le commit, si `mcp_sends::trouver` accepte un `Executor` (sinon sa forme `_in_tx`) |
| #773 point 3 : si la réponse d'une décision se perd, l'écran ne propose pas *Recharger* | **Traité si le point 1 ne suffit pas à le rendre sans objet** : `McpSendDetail.svelte:106`, le `catch` non `ApiError` (délai, réseau) montre le bouton existant de `:296` ; une ligne et un test Vitest |
| Facture fournisseur : `load_with_settlement_cancellation` après `pay`, `cancel`, `validate`, `unvalidate`, etc. (`supplier_invoices.rs:776,881,917,946,970,1002`, `imported_supplier_invoices.rs:141,595,700`) | **Non traité, à tracer** : neuf sites, et une lecture à quatre champs calculés ; déborde, et 33-1a/b récrit ce fichier. **Issue demandée à l'orchestrateur** (C-e334-8) |
| Le reste de #768 (points 2 à 7) et de #773 | Autre nature : les deux issues restent ouvertes, chacune commentée de ce qui a été fait |

## Acceptance Criteria

1. **AC-1** : `SettlementOutcome` porte `invoice: Invoice`, `lines: Vec<InvoiceLine>`, `amount_settled: Decimal`, lus dans la transaction, après la
   dernière écriture et avant le commit. Mêmes champs pour `WriteOffOutcome` (avec `amount_due`), et pour un retour de `cancel_settlement`.
2. **AC-2 — garde** : les corps de `settle_invoice_handler`, `write_off_invoice_handler` et `cancel_invoice_settlement_handler` ne contiennent ni
   `find_by_id_with_lines`, ni `amount_settled(`, ni `amount_due(`, ni `state.pool` après l'appel d'écriture (garde lexicale T3, avec **sonde positive**).
3. **AC-3 — ce qui est rendu est ce qui est écrit** : pour les trois routes, la réponse HTTP égale champ à champ un `GET /invoices/{id}` qui la suit
   (`version`, `paidAt`, `amountSettled`, `amountDue`, `updatedAt`, lignes) : règlement partiel, règlement total, écart d'arrondi, solde, annulation.
4. **AC-4 — le rejeu sur interblocage reste sûr** : la lecture étant dans la fermeture de `retry_on_deadlock`, la réponse vient de la tentative qui a
   réussi. Comportement des refus (statut, trop-perçu, verrou) inchangé : les tests existants passent sans modification.
5. **AC-5** : aucun appelant des trois fonctions ne change de comportement (`grep` des sites en `tests/`, `reconciliation`, `kesh-report`), `reconciliation_cancel`
   ne lit rien de plus.
6. **AC-6** : `decider_envoi` : la réponse vient de la tx ; égale à `GET /admin/mcp/sends/{id}` qui la suit ; ses refus inchangés.
7. **AC-7** : (si traité) l'écran propose *Recharger* quand la décision échoue sans réponse du serveur.
8. **AC-8** : une défaillance de la lecture interne fait échouer et annule la transaction : **pas de commit sans réponse**. Écrit au doc-comment, c'est ce
   qui ferme la classe.

## Tâches

- [ ] **T1 — rouge** : `tests/invoice_settlement_response_e2e.rs`, AC-3, et tests `kesh-db` : `settle_invoice(...)` rend `outcome.invoice.version` égal à la ligne
  relue, `outcome.amount_settled` égal au `SUM` : rouges à la compilation, écrits d'abord.
- [ ] **T2** : forme `find_with_lines_in_tx` à `repositories/invoices.rs`, réemployée par `find_by_id_with_lines` (DRY) ; sorties des trois fonctions ; trois handlers.
  ⚠️ `kesh-db` touché : **gate complet même en cours de boucle** (`CLAUDE.md`, exception `kesh-db`).
- [ ] **T3** : garde lexicale dans le même fichier de test.
- [ ] **T4** : `mcp_envois.rs`, `mcp_sends` ; `McpSendDetail.svelte` et son test si traité.
- [ ] **T5** : CHANGELOG *Corrigé* (`refs #450`, `refs #768`, `refs #773`).

### Mutations nommées (P2)

- **M1** : la lecture d'`amount_settled` déplacée avant l'`INSERT` du règlement → rouge AC-3.
- **M2** : la lecture de la facture déplacée avant l'incrément de `version` → rouge AC-3 (écart de version, celui qui ferait échouer le geste suivant en 409).
- **M3** : `find_by_id_with_lines` réintroduit dans un handler → rouge AC-2. **M4** : `outcome.lines` rendu vide → rouge AC-3.
- **M5** : `amount_due` omis du solde (retombé à `Decimal::ZERO`, plausible et faux sur une annulation) → rouge AC-3 côté annulation.

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, E2E complet au dernier commit de code. `sqlx` : aucune migration, aucun schéma.

## Manuels

Aucun comportement visible ne change hors la panne. **Prompt de revue : le dire et le vérifier** : `docs/api-external.md:503-521` et les manuels ne décrivent pas de 404
après règlement (`/usr/bin/grep -n 'règlement.*404' docs/api-external.md docs/manual/fr/*.tex`) ; si le manuel admin § MCP décrit l'écran d'envoi perdu, l'aligner et régénérer le PDF.

## Recoupements

`routes/invoices.rs` avec 33-4 (champs de date, autres lignes) ; `McpSendDetail.svelte` : aucun. Pas de fichier commun avec 33-1 : le fournisseur est écarté ici.

## Change Log

- 2026-11-06 : fiche créée (Opus 5.5).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
