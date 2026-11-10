# Story 34-5 : un second envoi du même règlement n'écrit pas un second règlement (#951)

Status: ready-for-dev

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** : le kickoff la disait probable, la fiche
tranche pour la version de la facture et non pour une clé d'idempotence (C-e343-1). Registre : `e34-choix-autonomes-e343.md`,
C-e343-1 à 5. Commits en `refs #951` ; à la PR : `closes #951`.*

## Story

En tant que Comptable, ou qu'intégrateur de l'API, je veux qu'un règlement de facture client renvoyé une seconde fois soit
refusé, qu'il vienne d'une réponse perdue, d'un second onglet ou d'une relance automatique. Ainsi, un même encaissement n'est
pas écrit deux fois au journal, et le reste dû ne baisse pas deux fois.

## Ce que le code dit (prémisses établies)

- **Le défaut est là.**
  - `SettleInvoiceRequest` n'a pas de `version` (`crates/kesh-api/src/routes/invoices.rs:1054-1069`). Son doc-comment en fait
    une règle : « Deux règlements concurrents ne se contredisent pas — le garde qui compte est le refus du trop-perçu »
    (`:1056-1059`).
  - `settle_invoice` verrouille la facture (`kesh-db/src/repositories/invoice_settlements_write.rs:115-130`) sans lire sa
    version.
  - Un règlement partiel égal au premier passe donc le refus du trop-perçu (`:253-266`) tant que le reste dû le couvre.
    Deux envois de 100 sur une facture de 300 écrivent deux règlements, deux écritures, deux entrées d'audit.
- **La version bouge déjà à chaque règlement, partiel compris** (`:345-374`, `UPDATE invoices SET … version = version + 1`). Son
  doc-comment en fait l'invariant de l'acceptation d'un rapprochement. La lire suffit : aucune colonne n'est à ajouter.
- **Le patron est dans le même fichier.** `write_off_invoice` reçoit `expected_version` (`:483-491`) et le compare sous le
  verrou, après le statut et « déjà payée », avant la date (`:526-544`) → `DbError::OptimisticLockConflict` → `409
  OPTIMISTIC_LOCK_CONFLICT`. Le corps de la route est `{ nature, settledOn, version }` (`routes/invoices.rs:1187-1190`).
- **Le rejeu sur interblocage reste sûr.** `retry_on_deadlock` (`kesh-db/src/retry.rs:151-157`) rejoue une transaction
  **annulée**, donc avec la version intacte : la tentative suivante passe. Seul un commit incrémente la version.
- **Un envoi concurrent est refusé de même.** Le second attend le verrou `FOR UPDATE` de l'étape (1), puis lit la ligne
  verrouillée : une lecture verrouillante lit la dernière version validée, et trouve donc la version incrémentée.
- **Les deux écrans qui appellent la route ont la version.**
  - Fiche : `invoice.version` (`InvoiceResponse`), appel `invoices/[id]/+page.svelte:604-629`.
  - Échéancier : `DueDateItem` étend `InvoiceListItemResponse`, qui porte `version` (`routes/invoices.rs:418`,
    `invoices.types.ts:204,218`), appel `due-dates/+page.svelte:265-303`.
  - `settleInvoice` (`frontend/src/lib/features/invoices/invoices.api.ts:131-148`) le dit dans son doc-comment : « ⚠️ Pas de
    `version` ».
  - Aucun autre appelant : `git grep settleInvoice -- frontend/src` ; ni le serveur MCP, qui n'écrit aucun règlement
    (`git grep settle -- crates/kesh-api/src/mcp`), ni `kesh-seed`.
- **Ce que la 33-6 a posé, et qui change de sens.**
  - Sur une issue inconnue, la fiche et l'échéancier ferment le dialogue, relisent, et gardent le geste indisponible jusqu'à
    une relecture réussie (`invoices/[id]/+page.svelte:458-485,1142-1165`).
  - `isSettlementOutcomeUnknown` (`frontend/src/lib/shared/utils/unknown-outcome.ts`) le justifie ainsi : « le POST d'un
    règlement ne porte aucune version, si bien qu'un second clic écrirait un second règlement ».
  - Le manuel utilisateur dit : « Rien n'empêchant d'enregistrer un second règlement partiel » et « Cette protection ne vaut que
    dans l'onglet » (`docs/manual/fr/user-manual.tex:1365-1373`).
  - Ces trois textes deviennent faux.
- **Un 4xx laisse le dialogue ouvert** (`invoices/[id]/+page.svelte:623-626`, `due-dates/+page.svelte:298-301`). Après un 409, le
  clic suivant repartirait avec la même version et rendrait 409 de nouveau : d'où C-e343-4.
- **Le corps de `POST /invoices/{id}/settlements` n'est documenté nulle part** dans `docs/api-external.md` : seuls ses refus
  le sont (`:536`). C'est une limite voisine, comblée ici.
- **Un corps JSON sans champ requis** est refusé par l'extracteur `Json` d'Axum : `422`, dit à `docs/api-external.md:294`.
- **Appelants Rust du repository** : `git grep -n 'invoice_settlements_write::settle_invoice'`. On trouve la route et huit
  sites de test (`accounts_designated_archive_e2e.rs:641,767`, `admin_full_import_e2e.rs:1702`,
  `invoice_echeancier_e2e.rs:1210`, `invoice_send_email_e2e.rs:2128`, `reconciliation_e2e.rs:5510,7363`,
  `services_rollout.rs:304`), plus `crates/kesh-db/tests/invoice_settlement.rs`.
- **Appelants HTTP de la route en test.** Rust : `bank_accounts_closed_e2e.rs`, `dates_bornes_e2e.rs`,
  `invoice_echeancier_e2e.rs`, `invoice_settlement_response_e2e.rs`, `reconciliation_e2e.rs`, `reconciliation_link_e2e.rs`,
  `reconciliation_link_entry_e2e.rs`, `rejeu_interblocage_e2e.rs`. Playwright : dix spécifications, dont
  `invoices-settle-unknown-outcome.spec.ts`, `reconciliation-link.spec.ts` et `open-items.spec.ts`. Tout corps de règlement
  sans `version` y devient `422`. **Recenser par `git grep` au développement**, sans se fier à cette liste.

## Décision : version, et non clé d'idempotence (C-e343-1)

La version couvre les quatre cas de #951 : réponse perdue, deux onglets, client de l'API, relance. Une clé d'idempotence ne
couvre pas les deux onglets, puisque chacun tire la sienne, et elle demande une migration sur une base de production. Le prix
de la version est qu'un rejeu rend 409 et non la réponse d'origine : l'écran relit et montre ce qui est écrit. Le détail est
au registre.

## Acceptance Criteria

1. **AC-1 — contrat.** `POST /api/v1/invoices/{id}/settlements` exige `version` (entier), à côté de `settlementType`,
   `bankAccountId` ou `accountId`, `amount` et `settledOn`. Un corps sans `version` est refusé par l'extracteur, `422`, sans
   rien écrire (C-e343-2).
2. **AC-2 — le défaut de l'issue.** Deux envois **identiques**, même version, d'un règlement partiel de 100 sur une facture de
   300 : le premier rend 200, le second `409 OPTIMISTIC_LOCK_CONFLICT`. En base, une ligne `invoice_settlements`, une écriture
   de règlement, une entrée d'audit `invoice.partially_settled`, et un reste dû de 200. **Test rouge avant le correctif** : la
   route actuelle ignore le champ inconnu et rend 200 deux fois.
3. **AC-3 — envois simultanés.** Deux envois lancés en parallèle avec la même version : exactement un 200 et un 409, quel que
   soit l'ordre d'arrivée. Une seule écriture.
4. **AC-4 — ordre des refus** (C-e343-3). Statut d'abord : une facture annulée ou en brouillon rend
   `ILLEGAL_STATE_TRANSITION`, même avec une version périmée. Puis la version, avant la date, la contrepartie, le trop-perçu et
   l'arrondi. Aucun refus n'écrit rien, et la version reste inchangée sur un refus.
5. **AC-5 — le rejeu sur interblocage reste sûr.** `settlement_is_replayed_when_it_is_the_deadlock_victim`
   (`rejeu_interblocage_e2e.rs:550-623`) passe avec la version dans le corps : 200, un seul règlement, un rejeu capturé.
6. **AC-6 — la réponse.** Elle porte `invoice.version` incrémentée (déjà vrai, 33-6). Un règlement qui la reprend comme
   `version` passe : deux acomptes successifs depuis le même écran restent possibles.
7. **AC-7 — fiche** (C-e343-4).
   - Le règlement part avec `invoice.version`.
   - Sur `409 OPTIMISTIC_LOCK_CONFLICT`, le dialogue se ferme, facture et règlements sont relus, et un avis dit que le
     règlement **n'a pas été enregistré** et que la facture a changé, avec la liste des règlements à vérifier.
   - Un autre 4xx garde le dialogue ouvert avec son message, comme avant.
   - L'issue inconnue de la 33-6 garde son comportement (C-e343-5).
8. **AC-8 — échéancier.** Même chose, avec `cible.version` capturée avant l'envoi (`due-dates/+page.svelte:270-272`) et la liste
   rechargée.
9. **AC-9 — textes vrais.** Ces textes ne disent plus qu'un second envoi écrit un second règlement :
   - le doc-comment de `SettleInvoiceRequest`, et celui de `settle_invoice` (qui gagne la version à son ordre des refus,
     doc-comment `:78-98`) ;
   - le doc-comment de `settleInvoice` et le type `SettleInvoiceRequest` (`invoices.types.ts:250-268`) ;
   - le doc-comment d'`isSettlementOutcomeUnknown` ;
   - le commentaire de `settleOutcome` (`invoices/[id]/+page.svelte:461-464`) ;
   - le manuel utilisateur `:1365-1373`.

   Le paragraphe du manuel dit désormais que le serveur refuse un règlement lancé sur une facture qui a changé, depuis tout
   onglet ou toute intégration, et que l'écran relit. Il garde le conseil de vérifier la liste après une issue inconnue.
10. **AC-10 — API documentée.** `docs/api-external.md` décrit le corps de `POST /invoices/{id}/settlements` (champs, `version`
    exigée, `409 OPTIMISTIC_LOCK_CONFLICT`, « relisez la facture par `GET` avant de relancer ») et sa réponse. La liste des
    routes qui exigent une version, si le document en tient une, est mise à jour.
11. **AC-11 — CHANGELOG.** `## [0.17.6] — Non publié`, *Corrigé* (`refs #951`), et une ligne ⚠️ de changement de contrat
    pour les intégrations (section *Modifié*, ou la convention du fichier).

## Tâches

- [ ] **T1 — rouge d'abord**, journalisé dans `kesh-gate-logs/e343/rouges.log`.
  - Test HTTP de l'AC-2 dans `crates/kesh-api/tests/invoice_settlement_response_e2e.rs`, ou dans un fichier neuf
    `invoice_settlement_idempotence_e2e.rs` : rouge **à l'exécution** (deux 200).
  - Test `kesh-db` de l'AC-4 dans `crates/kesh-db/tests/invoice_settlement.rs` : version périmée → `OptimisticLockConflict`,
    rien d'écrit, version inchangée. Rouge à la compilation, faute de paramètre.
- [ ] **T2 — `kesh-db`.**
  - `settle_invoice(…, expected_version: i32)`. La colonne `version` s'ajoute au `SELECT … FOR UPDATE` de l'étape (1), et la
    comparaison suit la garde de statut. `#[allow(clippy::too_many_arguments)]` si nécessaire, comme `write_off_invoice`.
  - Les appelants en test passent la version lue.
  - ⚠️ `kesh-db` est touché : **gate complet même en cours de boucle** (exception `kesh-db` du `CLAUDE.md`).
- [ ] **T3 — route.** `SettleInvoiceRequest.version: i32`, passée à `into_parts` ou à côté, et doc-comments (AC-9). Les
  corps de test Rust qui postent un règlement (liste ci-dessus, recensée à nouveau) portent la version.
- [ ] **T4 — AC-3.** Deux requêtes `tokio::spawn` sur la même version, contrôle « un 200, un 409, une écriture ».
- [ ] **T5 — frontend.**
  - `SettleInvoiceRequest.version: number` ; la fiche et l'échéancier l'envoient.
  - `settleOutcome` gagne un état `conflict` (relu, rien d'enregistré), avec une clé i18n
    `invoice-settle-conflict` aux **quatre** locales et le repli Svelte identique.
  - Tests Vitest : `invoice-settlements-page.test.ts`, et le test de l'échéancier. On vérifie la version envoyée, puis le 409
    (dialogue fermé, relecture, avis), puis un 400 (dialogue ouvert).
  - Les spécifications Playwright qui postent un règlement portent la version.
- [ ] **T6 — E2E** de l'AC-2 à l'écran. On règle 100 depuis la fiche. Un second `POST` part par `authedApiContext` avec la
  version d'avant : il rend 409. La fiche affiche un seul règlement. Puis l'avis de conflit, provoqué par un règlement passé
  par l'API pendant que le dialogue est ouvert.
- [ ] **T7 — textes** : AC-9, AC-10, AC-11 ; le PDF du manuel utilisateur est régénéré et contrôlé aplati
  (`pdftotext … | tr '\n' ' ' | tr -s ' '`, « Rien n'empêchant » absent).
- [ ] **T8 — captures** : l'avis de conflit sur la fiche et sur l'échéancier, en thème clair, sur une base jetable, dans
  `kesh-gate-logs/e343/captures/`.

### Mutations nommées

- **M1** : la comparaison de version retirée de `settle_invoice` → rouge AC-2, AC-3, et le test `kesh-db` de l'AC-4.
- **M2** : la comparaison faite **avant** le `SELECT … FOR UPDATE`, par une lecture ordinaire → rouge AC-3 (deux 200). C'est
  la course d'une lecture hors verrou.
- **M3** : la fiche envoie la version rendue par le `GET` initial au lieu de celle de la réponse → rouge du test « deux
  acomptes successifs depuis le même écran » (AC-6, Vitest).
- **M4** : le 409 traité comme un autre 4xx (dialogue ouvert) → rouge Vitest AC-7 et AC-8.
- **M5** : l'échéancier envoie `markTarget.version` lu après l'envoi au lieu de `cible` → rouge Vitest (une autre ligne
  ouverte pendant l'attente).

## Gates

`scripts/test-fast.sh` à chaque patch `kesh-db`, le frontend entier, l'E2E complet au dernier commit de code. Les sept échecs
de la KF-029 se nomment un par un.

## Ce qui ne se traite pas ici

- **La réponse rejouée** (ce qu'aurait donné une clé d'idempotence) : écartée, C-e343-1.
- **`POST /invoices/{id}/settlements/{settlementId}/cancel`** : un second envoi est déjà refusé, le règlement n'existe plus.
- **`POST /invoices/{id}/write-off`** : il a déjà sa version.
- **Côté fournisseur** : `POST /supplier-invoices/{id}/pay` est gardé par le statut
  (`kesh-db/src/repositories/supplier_invoices.rs:1980`).

## Recoupements

- `routes/invoices.rs` et `invoices/[id]/+page.svelte` : aucune autre story de l'Epic 34 ne les touche.
- `unknown-outcome.ts` : 34-8 en élargit le doc-comment au fournisseur (C-e343-12) ; à fusionner par union.
- `user-manual.tex` : 34-3, 34-4 et 34-11b touchent la réconciliation, d'autres sections.

## Dev Notes

- Le contrôle de version **ne remplace pas** le refus du trop-perçu, qui reste : deux règlements successifs et légitimes,
  chacun avec la version de la réponse précédente, doivent encore être bornés par le reste dû.
- Ne pas réutiliser la relecture d'issue inconnue **telle quelle** pour le 409 : son avis dit « l'issue n'est pas connue »,
  faux ici. Factoriser la relecture et distinguer l'état.
- Une `version` reçue en `i32` : même type que `WriteOffInvoiceRequest` et que la colonne.

## Change Log

- 2026-11-10 : fiche créée (Opus 5.5). Version contre clé tranchée (C-e343-1) ; aucune migration.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
