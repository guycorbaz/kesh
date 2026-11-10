# Story 34-5 : un second envoi du même règlement n'écrit pas un second règlement (#951)

Status: in-progress

*Fiche rédigée le 2026-11-10 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e34`, branche `chore/epic-34-kickoff`,
numéros de ligne de `a75835db`. Release **0.17.6**, Epic 34. **Aucune migration** : le kickoff la disait probable, la fiche
tranche pour la version de la facture et non pour une clé d'idempotence (C-e343-1). Registre : `e34-choix-autonomes-e343.md`,
C-e343-1 à 5 et 13. Commits en `refs #951` ; à la PR : `closes #951`.*

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
- **Appelants Rust du repository.** Commande : `git grep -n "settle_invoice(" -- crates`, hors
  `invoice_settlements_write.rs` et `src/routes`. Elle rend **31 appels dans 14 fichiers** à `a75835db` :
  - `kesh-api/tests/` : six fichiers ;
  - `kesh-db/tests/` : sept fichiers, dont `invoice_settlement.rs` (16 appels) et le module partagé
    `support/lettering_documents.rs`, qui touche tous ses utilisateurs ;
  - **`crates/kesh-report/tests/aged_receivables.rs`**, un autre crate.

  Dans `invoice_settlement.rs`, deux règlements se suivent (`:204` puis `:221`, `:2228` puis `:2246`) : la seconde version
  vient de la réponse du premier.
  *(Corrigé à la validation P1 : la liste d'origine nommait huit sites et `invoice_settlement.rs`. Le rapport en compte 35
  dans 15 fichiers ; la commande ci-dessus en rend 31 dans 14, route exclue. Recompter au développement.)*
- **Appelants HTTP de la route en test.** Rust : `bank_accounts_closed_e2e.rs`, `dates_bornes_e2e.rs`,
  `invoice_echeancier_e2e.rs`, `invoice_settlement_response_e2e.rs`, `reconciliation_e2e.rs`, `reconciliation_link_e2e.rs`,
  `reconciliation_link_entry_e2e.rs`, `rejeu_interblocage_e2e.rs`. Playwright : dix spécifications, dont
  `invoices-settle-unknown-outcome.spec.ts`, `reconciliation-link.spec.ts` et `open-items.spec.ts`. Tout corps de règlement
  sans `version` y devient `422`. **Recenser par `git grep` au développement**, sans se fier à cette liste.
- **L'outillage de release poste aussi un règlement.**
  - `scripts/recette-montee.sh:507` envoie `POST /api/v1/invoices/$F2/settlements` sans version, juste après la lecture de la
    facture (`:504`, `R=$(… GET /api/v1/invoices/$F2)`), qui porte `.version`. Le `jq` du reste dû est à `:505`.
  - Le script peuple l'image **précédente**. Pour la 0.17.6, c'est la 0.17.5, qui ignore le champ : rien ne casse maintenant.
    Mais dès que l'image de départ porte la 0.17.6, le peuplement rendrait 422.
  - `scripts/tests/recette-montee.test.sh:436` énumère `/settlements` parmi les chemins d'échec simulés.

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
3. **AC-3 — envois simultanés.** Le montage de l'AC-2 : 100 sur une facture de 300, deux envois lancés en parallèle avec la
   même version. Exactement un 200 et un 409, quel que soit l'ordre d'arrivée, et une seule écriture.
   - Les montants sont fixés pour que M2 rougisse par « deux 200 ». Un règlement du total serait refusé au second envoi par le
     trop-perçu (validation P3, 34-5-F2).
4. **AC-4 — ordre des refus** (C-e343-3). Les refus de forme précèdent la transaction et ne changent pas : le corps à la
   route (`into_parts`), puis le montant non positif (`amountMustBePositive`, `invoice_settlements_write.rs:108-110`).
   **Parmi les refus lus sous le verrou de l'étape (1)**, le statut vient d'abord : une facture annulée ou en brouillon rend
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
   - Si la relecture qui suit le 409 échoue, l'écran passe dans un état bloquant **propre au conflit**,
     `conflict-blocked`. Son texte : « Le règlement n'a pas été enregistré : la facture a changé, et elle n'a pas pu être
     relue. Le règlement reste indisponible tant qu'elle n'a pas été relue. » Le règlement reste indisponible, et *Relire*
     mène, s'il réussit, à `conflict` et non à `verify`.
     - Sans ce blocage, le dialogue rouvert repartirait avec la version périmée et rendrait 409 en boucle.
     - **Les états `blocked` et `verify` de la 33-6 ne servent pas ici** : leurs textes disent « l'issue n'est pas connue »
       (`messages.ftl:827-828`, `invoices/[id]/+page.svelte:1142-1163`), ce qui est faux après un 409. Rien n'a été écrit
       (validation P2, 34-5-F1).
   - Un autre 4xx garde le dialogue ouvert avec son message, comme avant.
   - L'issue inconnue de la 33-6 garde son comportement (C-e343-5).
8. **AC-8 — échéancier.** Même chose, avec `cible.version` capturée avant l'envoi (`due-dates/+page.svelte:270-272`) et la liste
   rechargée. L'avis de conflit nomme la facture (`cible.invoiceNumber`, comme `settleUnknownNumber` à la 33-6) : la liste
   montre plusieurs lignes, et le dialogue est fermé. Les mêmes deux états, `conflict` et `conflict-blocked`, s'y
   appliquent : le second dit que la **liste** n'a pas pu être relue, comme `due-dates-settle-unknown-reread-failed`
   (`messages.ftl:831`).
9. **AC-9 — textes vrais.** Ces textes ne disent plus qu'un second envoi écrit un second règlement :
   - le doc-comment de `SettleInvoiceRequest`, et celui de `settle_invoice` (qui gagne la version à son ordre des refus,
     doc-comment `:78-98`) ;
   - le doc-comment de `settleInvoice` et le type `SettleInvoiceRequest` (`invoices.types.ts:250-268`) ;
   - le doc-comment d'`isSettlementOutcomeUnknown` ;
   - le commentaire de `settleOutcome` (`invoices/[id]/+page.svelte:461-464`) ;
   - le commentaire du test `invoices/[id]/invoice-settlements-page.test.ts:266-268` (« le POST ne porte pas de version ») ;
   - le manuel utilisateur `:1365-1373`.

   Contrôle : `git grep -nE "ne porte (pas de|aucune) version|Pas de .version|Aucun champ .version" -- frontend crates` ne
   rend plus aucun de ces sites.

   Le paragraphe du manuel dit désormais que le serveur refuse un règlement lancé sur une facture qui a changé, depuis tout
   onglet ou toute intégration, et que l'écran relit. Il garde le conseil de vérifier la liste après une issue inconnue.
10. **AC-10 — API documentée.** `docs/api-external.md` décrit le corps de `POST /invoices/{id}/settlements` (champs, `version`
    exigée, `409 OPTIMISTIC_LOCK_CONFLICT`, « relisez la facture par `GET` avant de relancer ») et sa réponse. La liste des
    routes qui exigent une version, si le document en tient une, est mise à jour.
11. **AC-11 — CHANGELOG.** `## [0.17.6] — Non publié`, *Corrigé* (`refs #951`), et une ligne ⚠️ « **Changement de contrat de
    l'API** » sous *Modifié*. Le précédent est `CHANGELOG.md:202` (#849). C'est un écart voulu au prompt commun, qui ne
    prévoit qu'*Ajouté* et *Corrigé* : il est à dire au prompt de développement.
11 bis. **AC-11 bis — manuel d'administration** (validation P2, 34-5-F4).
    - Un paragraphe sous `\subsubsection{Passer à la 0.17.6}` de `admin-manual.tex`, sur le modèle de `:2659-2661` (#849,
      « changement de contrat »). La section est à créer avant `Passer à la 0.17.5` (`:2531`) si elle n'existe pas encore,
      et elle se fusionne par union avec les autres stories de l'epic.
    - Le paragraphe nomme la route, le champ `version` exigé, le `422` sans lui, le `409` et la relecture par `GET`.
    - Le PDF est régénéré et contrôlé aplati.
12. **AC-12 — outillage de release.** `scripts/recette-montee.sh:507` lit `.version` sur `$R` (`jq -r '.version'`) et l'envoie
    dans le corps. La 0.17.5 accepte ce corps, faute de `deny_unknown_fields`. `scripts/tests/recette-montee.test.sh` reste
    vert.

## Tâches

- [ ] **T1 — rouge d'abord**, journalisé dans `kesh-gate-logs/e343/rouges.log`.
  - Test HTTP de l'AC-2 dans `crates/kesh-api/tests/invoice_settlement_response_e2e.rs`, ou dans un fichier neuf
    `invoice_settlement_idempotence_e2e.rs` : rouge **à l'exécution** (deux 200).
  - Test `kesh-db` de l'AC-4 dans `crates/kesh-db/tests/invoice_settlement.rs` : version périmée → `OptimisticLockConflict`,
    rien d'écrit, version inchangée. Rouge à la compilation, faute de paramètre.
- [ ] **T2 — `kesh-db`.**
  - `settle_invoice(…, expected_version: i32)`. La colonne `version` s'ajoute au `SELECT … FOR UPDATE` de l'étape (1), et la
    comparaison suit la garde de statut. `#[allow(clippy::too_many_arguments)]` si nécessaire, comme `write_off_invoice`.
  - Les appelants en test passent la version lue, par **une aide par crate de test** (C-e343-13), et non par 31 lectures
    recopiées (règle DRY).
    - `kesh-db/tests/support/` (le module existe) : `settle_at_current_version(pool, …)`, qui lit la version puis appelle
      `settle_invoice`.
    - `kesh-api/tests/common/` : la même aide, ou un réemploi si le module de `kesh-db` est visible. Deux ensembles sont
      à distinguer (validation P4, 34-5-F3) :
      - **les six fichiers qui appellent le repository** `settle_invoice`. Trois ne déclarent pas `mod common;` :
        `invoice_echeancier_e2e.rs`, `invoice_send_email_e2e.rs` et `reconciliation_e2e.rs` ;
      - **les huit qui postent la route** (liste des prémisses). Trois ne déclarent pas `mod common;` :
        `invoice_echeancier_e2e.rs`, `invoice_settlement_response_e2e.rs` et `reconciliation_e2e.rs`.

      Décomptes par `grep -c "^mod common"` = 0. `invoice_send_email_e2e.rs` n'appelle que le repository (`:2128`), sans
      `POST`. Ils le déclarent, ce qui ne produit pas d'avertissement, le module
      portant `#![allow(dead_code)]` (`common/mod.rs:9`) ; ou ils gardent une lecture locale (validation P3, 34-5-F4).
    - `kesh-report/tests/aged_receivables.rs` : un seul appel, avec une lecture locale.

    Les dossiers `tests/` de crates différents ne partagent aucun module, et `kesh-db` n'a pas de feature de test
    (validation P2, 34-5-F8). Pour les règlements successifs, la version vient de la réponse du précédent.
  - `kesh-report` est touché : le gate ciblé `kesh-db`/`kesh-api` ne le compile pas. Lancer
    `cargo build --workspace --all-targets`.
  - ⚠️ `kesh-db` est touché : **gate complet même en cours de boucle** (exception `kesh-db` du `CLAUDE.md`).
- [ ] **T3 — route.** `SettleInvoiceRequest.version: i32`, passée à `into_parts` ou à côté, et doc-comments (AC-9). Les
  corps de test Rust qui postent un règlement (liste ci-dessus, recensée à nouveau) portent la version.
- [ ] **T4 — AC-3, montage déterministe** (validation P2, 34-5-F2), sur le patron de `victime`
  (`rejeu_interblocage_e2e.rs:346-367`) :
  1. une transaction de test tient la facture par `SELECT … FROM invoices WHERE id = ? FOR UPDATE` ;
  2. les deux requêtes partent, avec la même version ;
  3. on attend qu'elles soient **toutes deux** en attente sur le `FOR UPDATE` de l'étape (1)
     (`kesh_db::test_fixtures::attendre_des_requetes_en_cours(pool, motifs, 2, …)`, `test_fixtures.rs:582`, motifs
     `FROM invoices` et `FOR UPDATE`) ;
  4. la transaction de test est annulée.

  Code juste : un 200, un 409, une écriture. Sous M2, les deux lectures ordinaires de la version précèdent l'attente : deux
  200. Sans ce montage, M2 ne rougit que si les deux lectures précèdent le premier commit, ce que rien ne garantit.
  L'attente de plusieurs requêtes existe déjà : `attendre_une_requete_en_cours` (`:566-572`) en est le cas `au_moins = 1`
  (Story 30-1a), visible de `kesh-api/tests` par `kesh_db::test_fixtures` (`rejeu_interblocage_e2e.rs:47`). Validation P3,
  34-5-F1. ⚠️ Reste **à établir à l'étape 0** : ce que rend `PROCESSLIST` sous ce montage à deux requêtes. Si le montage ne
  peut pas être tenu, le dire au Change Log, et M2 est déclarée non prouvée.
- [ ] **T5 — frontend.**
  - `SettleInvoiceRequest.version: number` ; la fiche et l'échéancier l'envoient.
  - `settleOutcome` gagne deux états, `conflict` (relu, rien d'enregistré) et `conflict-blocked` (pas relu). Les clés i18n,
    aux **quatre** locales, chacune avec un repli Svelte identique :
    - fiche : `invoice-settle-conflict`, `invoice-settle-conflict-reread-failed` ;
    - échéancier : `due-dates-settle-conflict` et `due-dates-settle-conflict-reread-failed`, avec `{ $number }`, passé par
      le troisième argument d'`i18nMsg` (validation P2, 34-5-F7).
  - **Câblage** (validation P3, 34-5-F3). Aujourd'hui, le bouton n'est désactivé que pour `blocked`
    (`invoices/[id]/+page.svelte:1004`, échéancier `due-dates/+page.svelte:539`), et *Relire* n'est offert qu'en `blocked`
    (`:1159`, échéancier `:381`). La fiche change trois choses :
    - les **deux** états bloquants, `blocked` et `conflict-blocked`, désactivent le geste et offrent *Relire* ;
    - la relecture (`rereadAfterUnknownOutcome`, `:468-483`, et sa jumelle de l'échéancier, `:104-112`) prend en paramètre
      l'état bloquant et l'état d'arrivée, aujourd'hui écrits en dur ;
    - l'encadré de conflit a son propre `data-testid` (par exemple `settle-conflict`), distinct de `settle-unknown-outcome`
      (`:1146`), pour que l'AC-7 distingue `conflict` de `verify`.
  - Vitest de l'AC-7 : 409, relecture en échec, texte sans « n'est pas connue » ; puis *Relire* réussi, texte de conflit.
  - Tests Vitest : `invoice-settlements-page.test.ts`, et le test de l'échéancier. On vérifie la version envoyée, puis le 409
    (dialogue fermé, relecture, avis), puis un 400 (dialogue ouvert).
  - Les spécifications Playwright qui postent un règlement portent la version.
- [ ] **T6 — E2E** de l'AC-2 à l'écran. On règle 100 depuis la fiche. Un second `POST` part par `authedApiContext` avec la
  version d'avant : il rend 409. La fiche affiche un seul règlement. Puis l'avis de conflit, provoqué par un règlement passé
  par l'API pendant que le dialogue est ouvert.
- [ ] **T7 — textes et outillage** : AC-9 à AC-12 ; les PDF des deux manuels sont régénérés et contrôlé aplati
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
- **M5** : l'avis de conflit de l'échéancier lit `markTarget?.invoiceNumber` **après** la relecture (où `markTarget` vaut
  `null`, `due-dates/+page.svelte:292-293`) au lieu de `cible.invoiceNumber` → rouge du Vitest de l'AC-8 (numéro absent de
  l'avis). *(Remplace la M5 d'origine, équivalente : `cible` et `markTarget` désignent le même objet au moment où le corps
  se construit, sans `await` entre les deux, `:268-276`. Validation P2, 34-5-F3.)*
- **M6** : *Relire* depuis `conflict-blocked` mène à `verify` → rouge du Vitest de l'AC-7 (texte « n'est pas connue »).

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
- **34-11a1 et 34-11a2** (34-11a découpée) : elles ne touchent pas `settle_invoice`. La 34-11a1 range `:170` en classe C,
  non touchée (`34-11a1-compte-sans-iban.md:76`, ligne C5). Elles touchent le dialogue de règlement,
  `SettleInvoiceDialog.svelte:249` : libellé du compte (34-11a1, S13) et carte non proposée (34-11a2, `:38`).
  - La 34-5 ne modifie pas ce dialogue : la version s'ajoute dans les pages (`{ ...payload, version }`), et le type
    `SettlementPayload` reste sans version. Fusion sans conflit.
  - Aucune migration à ordonner : la 34-5 n'en porte pas.

  *(Rectifié à la validation P2, 34-5-F6 : la P1 avait écrit un recoupement sur `settle_invoice`, faux.)*
- `reconciliation_e2e.rs` (appels `:5510`, `:7363`) : aussi touché par 34-3, 34-4 et 34-11b.
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
- 2026-11-10 : **validation P1 (Sonnet) : 2 M, 4 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p1-e343.md`. Chaque prémisse a été relue au code avant d'appliquer :
  - **F1 (M)** : l'inventaire des appelants est remplacé par sa commande (31 appels dans 14 fichiers, route exclue,
    `kesh-report` compris), avec l'aide de test unique et le build du workspace.
  - **F2 (M)** : `scripts/recette-montee.sh:505-507` est lu, d'où l'AC-12.
  - **F3 (L)** : recoupements avec 34-11a (`:170`, même fonction) et avec `reconciliation_e2e.rs`. La mention d'une migration
    à ordonner avec 34-5, dans la fiche 34-11a, est transmise par l'orchestrateur.
  - **F4 (L)** : `invoice-settlements-page.test.ts:266-268` lu, ajouté à l'AC-9 avec le grep de contrôle.
  - **F5 (L)** : AC-7 (relecture en échec → `blocked`) et AC-8 (l'avis nomme la facture).
  - **F6 (L)** : *Modifié* tranché, précédent `CHANGELOG.md:202` lu.
- 2026-11-10 : **validation P2 (Opus) : 4 M, 5 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p2-e343.md`. Prémisses relues : `messages.ftl:827-831` ;
  `invoice_settlements_write.rs:108-110` ; `34-11a1-compte-sans-iban.md:76` ; `34-11a2-carte-credit-creation.md:38` ;
  `admin-manual.tex:2531,2659-2661` ; `kesh-db/Cargo.toml` sans `[features]` ; `attendre_une_requete_en_cours`
  (`rejeu_interblocage_e2e.rs`).
  - **La sévérité n'a pas baissé** : P1 2 M → P2 4 M. Signal de la règle de splitting, avec le constat de l'amendement D5,
    à déclarer au Project Lead :
    - **F1 (M), né de la remédiation P1** (F5) : l'état bloquant d'après un 409 affichait un texte d'issue inconnue. États
      `conflict` et `conflict-blocked`, clés, M6.
    - **F2 (M), neuf**, présent depuis la création : M2 dépendait d'une course. Montage déterministe à T4, à établir à
      l'étape 0.
    - **F3 (M), neuf** : la M5 d'origine était équivalente. Elle est remplacée.
    - **F4 (M), neuf** : manuel d'administration, AC-11 bis.

    Trois des quatre sont des défauts d'origine distincts, non un recyclage. Le seul né d'une remédiation est F1. La fiche
    reste dans cinq modules au plus. Pas de découpage proposé.
  - **LOW** :
    - **F5** : six fichiers et sept fichiers, ventilation corrigée.
    - **F6** : recoupement 34-11a récrit (34-11a1, 34-11a2, dialogue).
    - **F7** : clés de l'échéancier.
    - **F8** : aide par crate, C-e343-13.
    - **F9** : ordre des refus, « sous le verrou ».
- 2026-11-10 : **validation P3 (Sonnet) : 0 M, 5 L, tous remédiés** (Opus 5.5). Rapport
  `kesh-gate-logs/e34-validate-p3-e343.md`. Prémisses relues :
  - `test_fixtures.rs:566-582` ;
  - `^mod common` dans les six fichiers `kesh-api/tests` ;
  - `recette-montee.sh:504-505` ;
  - `invoices/[id]/+page.svelte:1004,1146,1159` et `due-dates/+page.svelte:104-112`.

  Les remédiations :
  - **F1** : `attendre_des_requetes_en_cours(…, 2, …)` existe, et le conditionnel est retiré.
  - **F2** : l'AC-3 reprend 100 sur 300.
  - **F3** : câblage des états bloquants, relecture paramétrée, `data-testid` propre.
  - **F4** : `mod common;` dans trois fichiers.
  - **F5** : `:504`.
- 2026-11-10 : **validation P4 ciblée (Haiku) : 1 L, remédié** (Opus 5.5). **F3** : relu, `^mod common` dans les huit
  appelants HTTP. Les deux ensembles sont distingués, repository (six fichiers) et route (huit fichiers). Prémisse du rapport
  nuancée : « six » désignait les appelants du repository, et c'était juste ; la liste des appelants HTTP manquait.
- 2026-11-10/11 : **développement** (Opus 5.5, worktree `/home/gcorbaz/devel/kesh-e345`, base `665de151`, clé `e345` ; la
  fiche disait `e343` pour les journaux, ils sont sous `kesh-gate-logs/e345/`). Commits `71beefe6` (développement),
  `3224a199` (type d'une aide Vitest), `b8b749af` (borne des sites i18n), `0a886d6d` (`cargo fmt`). Choix neufs :
  `e34-choix-autonomes-e345.md`, C-e345-1 à 9. Détail au Dev Agent Record.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie.

### Completion Notes List

**Étape 0.** Les deux LOW de la validation P5 (`e34-validate-p5-e343.md`) portent sur la 34-6 (F1) et la 34-8 (F2) :
rien à traiter ici. Le point « à établir à l'étape 0 » de T4 (ce que rend `PROCESSLIST` sous le montage à deux
requêtes) est **établi par le rouge** : sur le code d'avant, `attendre_des_requetes_en_cours(&pool, &["FROM invoices",
"journal_entry_id", "FOR UPDATE"], 2, …)` a rendu vrai — l'assertion de montage est passée, l'échec portait sur les
statuts (`[200, 200]`). M2 est donc prouvée, non déclarée. Inventaire des appelants recompté :
`git grep -n "settle_invoice(" -- crates`, hors `src/` : **31 appels dans 14 fichiers** à la base (20 lignes à `HEAD`,
l'aide `settle_at_current_version` en absorbant seize dans `invoice_settlement.rs`).

**Rouges d'abord** (`kesh-gate-logs/e345/rouges.log`, journaux `rouge-ac2.log`, `rouge-ac4-compil.log`) :
- HTTP, à l'exécution, sur le code de base : AC-1 (200 au lieu de 422), AC-2 (second envoi 200 au lieu de 409), AC-3
  (`[200, 200]`) ; AC-6 vert, comme attendu.
- `kesh-db`, à la compilation : six `E0061` (« takes 7 arguments but 8 arguments were supplied »).

**Ce qui est fait.**
- `settle_invoice(…, expected_version: i32)` : `version` lue dans le `SELECT … FOR UPDATE` de l'étape (1), comparée
  après le statut, avant la date (`OptimisticLockConflict`). Doc-comment : ordre des refus avec la version, rejeu sûr.
- Route : `SettleInvoiceRequest.version: i32` (obligatoire, `422` sans lui), passée au repository dans
  `retry_on_deadlock`. Doc-comments récrits, et le commentaire voisin de `invoices.rs:986-991` qui disait encore « le
  règlement ne prend pas de `version` » (trouvé par le grep du symptôme, hors liste de l'AC-9).
- Tests : AC-1, AC-2, AC-3 (montage déterministe), AC-6 dans `invoice_settlement_response_e2e.rs` (C-e345-3) ; AC-4 en
  deux tests dans `kesh-db/tests/invoice_settlement.rs` (version périmée sans rien écrire ; ordre statut → version →
  date / contrepartie / trop-perçu). AC-5 : `settlement_is_replayed_when_it_is_the_deadlock_victim` passe avec la
  version (gate). Aides : `kesh-db/tests/support/reglement_version.rs` et `kesh-api/tests/common/mod.rs`
  (`invoice_version`, `settle_at_current_version`) ; lectures locales dans `lettering_documents.rs` et `kesh-report`
  (C-e345-1, C-e345-2). `mod common;` ajouté à `invoice_echeancier_e2e.rs`, `invoice_send_email_e2e.rs`,
  `reconciliation_e2e.rs`.
- Frontend : la fiche envoie `invoice.version`, l'échéancier `cible.version` ; états `conflict` et `conflict-blocked`,
  relecture paramétrée `rereadAfterSettle(blocking, arrival)` et `rereadFromBlocked()` ; encadrés `settle-conflict` et
  `due-dates-settle-conflict` ; quatre clés aux quatre locales, replis identiques à `fr-CH` (C-e345-4, 5).
- Playwright : `invoiceVersionViaApi` (C-e345-6) dans les huit specs qui postent un règlement ; spec neuve
  `invoices-settle-version.spec.ts` (T6 : AC-2 à l'écran, avis de conflit sur la fiche et l'échéancier).
- Textes : AC-9 (contrôle `git grep -nE "ne porte (pas de|aucune) version|Pas de .version|Aucun champ .version" --
  frontend crates` : vide), manuel utilisateur § *Enregistrer et annuler un règlement*, manuel d'administration
  § *Passer à la 0.17.6* (créée, C-e345-7), `docs/api-external.md` § *Enregistrer un règlement* (neuve, codes relus
  dans `errors.rs` et `dates_recues.rs`), CHANGELOG `## [0.17.6] — Non publié` (*Corrigé* et *Modifié* ⚠️),
  `scripts/recette-montee.sh` (C-e345-8 ; `scripts/tests/recette-montee.test.sh` vert, « aucune assertion en échec »).
- Garde `i18n-keys.test.ts` : `sitesTotal` 2922 → 2928 (+6 : quatre avis, deux *Relire*), mesuré par la garde.

**Mutations** (`kesh-gate-logs/e345/mut/M*.log`, `mutations.py`, `run.sh` ; motif unique vérifié, restauration
`git checkout --` puis `touch`, « restauré : 0 fichier(s) modifié(s) » à chacune ; jouées sur `b8b749af`) :
- M1 (comparaison retirée) : tuée — AC-2, AC-3 et les deux tests AC-4 rouges.
- M2 (comparaison hors verrou, lecture ordinaire) : tuée — AC-3 `left: [200, 200]`, et le test d'ordre AC-4.
- M3 (version d'ouverture de l'écran) : tuée — « deux acomptes successifs » (`expected 3 to be 4`) et le 409 (clic
  suivant).
- M4 fiche / M4 échéancier (409 traité comme un 4xx) : tuées, deux Vitest chacune.
- M5 (numéro lu sur `markTarget` après relecture) : tuée — « expected 'Le règlement de la facture  n’a pas é…' to contain
  'F-2026-005' ».
- M6 (*Relire* depuis `conflict-blocked` → `verify`, fiche) : tuée. **Non jouée** : la M6 côté échéancier
  (`rereadFromBlocked` jumeau), couverte par le test « 409 puis liste illisible… » sans mutation dédiée.

**Gates** (au dernier commit de code `0a886d6d`, bases remises à neuf ; verrous de la station, `free -g` avant
chaque commande, journaux `kesh-gate-logs/e345/gates-g2-*`, `front-*`) :
- `scripts/test-fast.sh --ci` : **5144 passés, 12 ignorés** (fmt et clippy compris).
- `cargo build --workspace --all-targets` : vert.
- `scripts/gate-ci.sh kesh_ci_e345` : **5151 passés, 0 échec, 16 ignorés**.
- Frontend : `npm run check` 0 erreur (r2, `3224a199`), `lint-i18n-ownership` vert, `npm run build` vert (r2) ;
  `npm run test:unit` **2889 passés, 222 fichiers** (r3, `0a886d6d`). Entre r2 et `0a886d6d`, seuls un test et du
  formatage Rust ont changé.
- Gate ciblé intermédiaire (`cible-1.log`, 23 binaires touchés) : 753 passés.
- ⚠️ Un premier `test-fast` (g1, `b8b749af`) a échoué sur `cargo fmt --check` (deux fichiers neufs) : corrigé par
  `0a886d6d`, puis tout rejoué (g2).
- **Différés par la pause des commandes lourdes** (décision de Guy du 2026-11-11, `kesh-gate-logs/PAUSE-HEAVY`) : le
  `make fr` des manuels (PDF), l'E2E complet et les captures. Voir la suite de ce record.

**Écarts à la consigne Edit.** Deux fichiers de test écrits par le shell, non par l'outil : les tests Vitest de la
fiche ajoutés en fin de `invoice-settlements-page.test.ts` par `cat >>` (ajout, rien remplacé), et la spec neuve
`invoices-settle-version.spec.ts` créée par `cat >` (l'outil Write convenait). Aucun `sed -i` ni `perl -pi`. `cargo fmt
--all` a formaté quatre fichiers Rust (outil du projet). L'outil ne manquait de rien : c'est un écart de commodité.

**Limites voisines.** Aucune nouvelle issue à ouvrir au développement.

### File List

- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-api/src/routes/invoices.rs`
- `crates/kesh-db/tests/support/reglement_version.rs` (neuf), `support/lettering_documents.rs`, `invoice_settlement.rs`,
  `credit_notes_repository.rs`, `filet_bilan_clos.rs`, `invoice_amount_due_parity.rs`, `invoice_write_off.rs`,
  `invoices_validate_vat.rs`
- `crates/kesh-api/tests/common/mod.rs`, `invoice_settlement_response_e2e.rs`, `accounts_designated_archive_e2e.rs`,
  `admin_full_import_e2e.rs`, `bank_accounts_closed_e2e.rs`, `dates_bornes_e2e.rs`, `invoice_echeancier_e2e.rs`,
  `invoice_send_email_e2e.rs`, `reconciliation_e2e.rs`, `reconciliation_link_e2e.rs`, `reconciliation_link_entry_e2e.rs`,
  `rejeu_interblocage_e2e.rs`, `services_rollout.rs`
- `crates/kesh-report/tests/aged_receivables.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/invoices/invoices.api.ts`, `invoices.types.ts`, `frontend/src/lib/shared/utils/unknown-outcome.ts`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `invoice-settlements-page.test.ts`
- `frontend/src/routes/(app)/invoices/due-dates/+page.svelte`, `due-dates-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/helpers/api-fixtures.ts`, `invoices-settle-version.spec.ts` (neuf), `invoices-settle-unknown-outcome.spec.ts`,
  `invoices_echeancier.spec.ts`, `invoices-settlement-cancel.spec.ts`, `invoice-write-off.spec.ts`, `open-items.spec.ts`,
  `reconciliation-amount-due.spec.ts`, `reconciliation-link.spec.ts`, `reconciliation-priority.spec.ts`, `reminders.spec.ts`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`, `docs/api-external.md`, `CHANGELOG.md`
- `scripts/recette-montee.sh`
- `_bmad-output/implementation-artifacts/e34-choix-autonomes-e345.md` (neuf),
  `34-5-reglement-idempotent-review-prompt-p1.md` (neuf), cette fiche, `sprint-status.yaml`
