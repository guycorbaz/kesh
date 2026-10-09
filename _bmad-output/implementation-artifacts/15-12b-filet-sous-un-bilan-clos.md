# Story 15.12b : Ne plus écrire sous un bilan clos — le filet des données héritées, son écran, sa réparation

Status: ready-for-dev

<!-- Créée le 2026-10-09 par DÉCOUPAGE de la Story 15-12 à la remédiation de sa validation P2 (décision de
     l'orchestrateur, choix C107 de `epic-15-choix-autonomes.md`). Version complète de la 15-12 avant
     découpage : commit `dae3a618`
     (`git show dae3a618:_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`).
     Choix applicables : C89, révisé par C100 (validation P1), C107 à C112 (validation P2), C117 (reçu
     de la 15-1a) et C120, C121, C123 (validation P3).
     ⚠️ **Numérotation des AC conservée de la 15-12** — d'où des trous (1-7, 9, 13, 14, 17, 22 sont dans
     la 15-12a) : les références croisées restent stables d'une fiche à l'autre. Un AC partagé porte
     « part B ».
     ⚠️ **Numéros de ligne** relevés sur `origin/main` `8f9811d8` (code Rust identique à `9cb5083b` hors
     `kesh-api/config.rs`, `main.rs`, `lib.rs` et `configuration_transmise.rs`) ; manuel administrateur
     recalé sur `8f9811d8`. T0 les refait sur le `main` du moment, par leur texte. ⚠️ **`main` a avancé
     depuis** : `5e4bec50` (15-5d mergée) décale notamment `kesh-api/src/errors.rs`, `invoices.rs` (les
     commentaires C-15-8-29 : `:1657` au lieu de `:1656`) et les catalogues. Les relevés **refaits en
     P3** (AC 11, 12, 18, 21) le sont sur `5e4bec50` et le disent. -->

**Issues** : **ferme #543** (P1 — « on peut écrire dans un exercice ouvert alors qu'un exercice
postérieur est déjà clos — le bilan clos change en silence », relevée par la validation P3 de la 15-8 ;
ses deux commentaires listent les flux sans garde). La PR porte `closes #543` (mot-clé **dans la PR** :
le dépôt merge en squash). Voisines citées, **non traitées ici** : #551 et #552 (Story 15-13).

**Dépend de** la **15-12a** (clôture dans l'ordre : `DbError::EarlierFiscalYearOpen`, la clé
`error-later-fiscal-year-closed` et le mapping de l'AC 9, l'invariant I sur lequel repose la preuve du
filet, la clôture ordonnée de la réparation), de la 15-8a et de la 15-8b (`LaterFiscalYearClosed`,
`find_later_closed_in_tx` / `find_later_closed`, garde du `PUT` et du `DELETE`), de la 15-5e1/15-5e2
(rejeu de toutes les routes qui écrivent au journal, registre à deux colonnes). Toutes mergées sauf la
15-12a.

**Touche des fichiers que d'autres stories en vol touchent** : `docs/manual/fr/user-manual.tex` et
`admin-manual.{tex,pdf}` (la 15-11a, mergée, a modifié le manuel administrateur ; la 15-12a aussi),
`CHANGELOG.md` (rubriques de `[0.13.0]`), `crates/kesh-db/src/repositories/journal_entries.rs` (la 15-5d
ne fait qu'y lire, les 15-6/15-6a n'y touchent pas — T0 le vérifie sur leurs branches ; **la 15-1a** y
pose la garde `ENTRY_LETTERED` dans `delete_in_tx`, cf. frontière). En cas de rebase : garder les deux
côtés, **régénérer** les PDF après le rebase (conflit binaire).

## Story

En tant que **comptable** dont l'installation porte déjà un exercice ouvert suivi d'un exercice clôturé
(données d'une v0.12.x, restauration d'une sauvegarde),
je veux que **Kesh refuse toute écriture et toute suppression d'écriture dans un exercice dont un
exercice postérieur est clôturé**, et m'indique comment rétablir l'ordre,
afin que **le bilan d'un exercice clôturé ne puisse plus changer en silence** — le bilan est cumulatif
depuis l'origine (`crates/kesh-report/src/balance_sheet.rs:8-11`).

### Le défaut, établi au code

- La 15-12a rend l'état « N ouvert, N+1 clos » **inatteignable** depuis un état sain. Mais les
  installations qui le portent déjà ne repassent par aucune transition.
- Dans cet état, seuls le `PUT` et le `DELETE` d'une écriture refusent (`LATER_FISCAL_YEAR_CLOSED`,
  15-8a/15-8b). **Dix-neuf** routes créent une écriture dans N et la **dévalidation** en supprime une,
  sans contrôle (inventaire fermé, AC 12).

### Conception retenue (C89, C100)

- **Le filet** : la garde `LATER_FISCAL_YEAR_CLOSED` de la 15-8a est **généralisée** aux deux points de
  passage de toutes les écritures du journal — `journal_entries::create_in_tx_inner` et
  `journal_entries::delete_in_tx` — ; les trois voies d'acceptation par lot du rapprochement rendent le
  refus dans `failed[]`.
- **L'écran des exercices signale** l'état et la marche à suivre pour le réparer (clôturer l'exercice
  ouvert le plus ancien, puis les suivants dans l'ordre ; ou faire rouvrir les exercices clôturés, en
  commençant par le plus récent — C100).

### Frontière avec la 15-12a

La 15-12a tient l'invariant I (clôture, création, réouverture), rend la clôture rejouée et pose le
message neutre de `LATER_FISCAL_YEAR_CLOSED` (AC 9 : clé `error-later-fiscal-year-closed`). Cette fiche
**lit** cette clé et ce mapping sans les modifier ; elle complète les doc-comments que la 15-12a a écrits
sans le filet (module `fiscal_years.rs`, `DbError::LaterFiscalYearClosed`, `find_later_closed`) et la
documentation (AC 23, part B). La dépendance est à sens unique : B → A.

### Frontière avec la 15-1a (C112)

- La 15-1a **n'exige pas** cette fiche : son prérequis réel est la 15-12a. Le filet couvre la création et
  la suppression d'écritures, **pas** le lettrage : dans l'état hérité, un lettrage d'un groupe tout en N
  sous un N+1 clos n'est refusé par rien d'ici (à la 15-1a de le tolérer, de le garder ou de l'écrire).
- Ordre écrit : **15-12a → 15-12b → 15-1a**. Motif : les deux fiches touchent `delete_in_tx` — cette fiche
  y lève la condition `enforce_ownership` de la lecture des postérieurs clos (AC 10, étape 2-bis), la
  15-1a y pose `ENTRY_LETTERED` (étape 3-ter-bis) hors de ce drapeau.
- **Précédence des deux refus, tranchée (C117)** : **« exercice postérieur clos » (2-bis) avant
  `ENTRY_LETTERED` (3-ter-bis)** — l'état des exercices parle avant la marque, comme `FISCAL_YEAR_CLOSED`
  avant `LATER_FISCAL_YEAR_CLOSED` ; l'inverse dirait « délettrez d'abord » à qui ne pourrait de toute
  façon rien supprimer. **La seconde des deux stories mergée** l'écrit au doc-comment « Ordre des refus »
  de `delete_in_tx` (et à la liste de précédence de `unvalidate`, AC 10) et la teste par une **paire** :
  écriture lettrée dans N, N+1 clos, `enforce_ownership` à `false` **et** à `true` →
  `LaterFiscalYearClosed` ; mutation « permuter les étapes » observée rouge. **Dans l'ordre préféré
  (15-12a → 15-12b → 15-1a), c'est la 15-1a qui le fait** ; cette fiche n'a alors rien à écrire sur
  `ENTRY_LETTERED`. ⚠️ **Si la 15-1a est mergée avant cette fiche**, la paire, la mutation et le
  doc-comment reviennent ici : T0 le constate (`grep -n "ENTRY_LETTERED\|EntryLettered"
  crates/kesh-db/src/repositories/journal_entries.rs`), et T1 les prend.

## Acceptance Criteria

### Le filet — écrire sous un bilan clos est refusé partout

8. **Point de passage de toute création d'écriture.** `journal_entries::create_in_tx_inner`
   (`journal_entries.rs:258`, appelé par `create_in_tx` et par la contre-passation) refuse
   `DbError::LaterFiscalYearClosed` (plus proche postérieur clos) quand un exercice de `start_date`
   postérieure à celle de l'exercice de l'écriture est clôturé. Placement : **après** le verdict
   « exercice clos » de l'étape 1, **avant** le contrôle des bornes de date et le verrou de période —
   précédence `NotFound` → `FISCAL_YEAR_CLOSED` → `LATER_FISCAL_YEAR_CLOSED` →
   `DATE_OUTSIDE_FISCAL_YEAR` → `PERIOD_LOCKED`, celle du `PUT` (`journal_entries.rs:1378-1403`).
   **Lecture NON verrouillante** (`fiscal_years::find_later_closed`, sur `&mut **tx`), et c'est un
   choix (C89, preuve réécrite en C100 et aux Dev Notes) : la sûreté repose sur **(α)** l'invariant I,
   vrai de **tout état validé** depuis un état sain — tenu par la 15-12a, dont la lecture verrouillante
   de la clôture est l'instrument face à la course réouverture/clôture (test 13 a de la 15-12a) — et
   **(β)** le verrou de son exercice que l'écrivain tient dès l'étape 1 : aucun postérieur ne peut se
   clôturer tant que l'exercice de l'écrivain est ouvert, et il l'est jusqu'au `COMMIT`. La lecture n'a
   donc à voir que l'état **hérité**, qui ne peut que se résorber. Un verrou ici ajouterait des verrous
   d'intervalle sur `fiscal_years` aux dix-neuf routes — dont la place est le sujet de toute la 15-5e —
   pour aucun gain de garantie. Le message est celui de l'AC 9 de la 15-12a (inchangé ici).

10. **Point de passage de toute suppression d'écriture.** `journal_entries::delete_in_tx` lit les
    exercices postérieurs clos **sans condition** (aujourd'hui `if enforce_ownership`,
    `journal_entries.rs:1681`) : la **dévalidation** d'une facture (`invoices::unvalidate`, seul
    appelant à passer `false`, `invoices.rs:1659`) refuse désormais `LATER_FISCAL_YEAR_CLOSED`. La
    lecture reste la verrouillante `find_later_closed_in_tx` déjà en place (code existant, testé par la
    15-8b ; la dévalidation est rare). **Précédence** : les cinq motifs propres de la facture
    (`invoices.rs:1462-1474`, contrôlés `:1505-1600`) parlent **avant** `delete_in_tx`, dont les
    gardes suivent — `INVOICE_HAS_SETTLEMENTS → INVOICE_CREDITED → INVOICE_HAS_REMINDERS →
    INVOICE_EMAILED → MATCHED_BANK_TRANSACTION → FISCAL_YEAR_CLOSED → LATER_FISCAL_YEAR_CLOSED →
    ENTRY_IS_REVERSED → PERIOD_LOCKED` (à relire au code en T0 et à écrire telle quelle au
    doc-comment de `unvalidate` — avec `ENTRY_LETTERED` à son rang si la 15-1a est mergée avant) ; test
    de paire : facture **envoyée** sous un postérieur clos → `INVOICE_EMAILED`. **Le test qui fixait le
    comportement actuel** (`journal_entries.rs:3427`, choix C-15-8-29 — « pour qu'il rougisse quand #543
    sera corrigée ») **est inversé**, et les commentaires `C-15-8-29` (`invoices.rs:1656`,
    `journal_entries.rs:1642`, `:1680`, `:3427`) sont réécrits.

11. **Acceptation par lot du rapprochement — trois voies** *(réécrit en P2 : R1 = F1, F8 ; C108)*. Un
    refus `LaterFiscalYearClosed` sorti de `journal_entries::create_in_tx` dans **chacune** des trois
    implémentations devient un `FailedProposal` **`LATER_FISCAL_YEAR_CLOSED`** avec `details: {
    fiscalYearId, fiscalYearName }` (et `projectId` quand la voie le connaît — cf. la voie « règle »
    ci-dessous) — jamais
    `DATABASE_ERROR` (§ *Pattern batch* du `CLAUDE.md`, qui déclare les trois `accept_one_*`
    inviolables). HTTP `200`, les autres propositions du lot passent. Sites (relevés sur `8f9811d8`,
    `crates/kesh-api/src/routes/reconciliation.rs`, `grep -nF "journal_entries::create_in_tx("`) :
    - **facture** — `accept_one_invoice`, `create_in_tx` `:1631`, erreur passée à
      `project_error_to_failed_proposal` (`:1634`) ;
    - **ventilé** — `accept_one_split`, `create_in_tx` `:2158`, idem (`:2164`) ;
    - **règle** — `accept_one_rule`, `create_in_tx` `:2528`, **branche en ligne** `:2530-2555` : son
      seul cas particulier est `PeriodLocked` (`:2538-2548`), puis un repli `DATABASE_ERROR`
      (`:2549-2553`). Elle n'emprunte **pas** `project_error_to_failed_proposal`, et ne doit pas
      l'emprunter : le bras `NotFound → PROJECT_NOT_FOUND` de ce mapper mal-étiquetterait un `NotFound`
      étranger au projet (commentaire `:2531-2534`, doc-comment du mapper `:180-181`). **`projectId` :
      `None`** (R4, C123) — patron de son `PeriodLocked` (`:2541-2546`), qui passe `None` alors que la
      règle a un `default_project_id` : les deux refus de la même branche publient les mêmes `details`,
      et le constructeur **omet** `projectId` quand il vaut `None` (convention de
      `period_locked_failed_proposal`, `:186-193`). Le test de la voie « règle » asserte cette absence.
    ⚠️ **La commande rend cinq sites, pas trois** (R4 ; relevé `5e4bec50`, numéros identiques) : s'y
    ajoutent `:3304` (`post_manual_once`) et `:3756` (`post_split_once`), les tentatives uniques des
    routes **unitaires** `POST /reconciliation/manual` et `/split`. Hors du lot : leur refus
    `LaterFiscalYearClosed` sort par le **mapping global** (`ReconciliationError::Db` →
    `AppError::Database`, puis `400 LATER_FISCAL_YEAR_CLOSED`, message de l'AC 9 de la 15-12a — T0 relit
    la conversion de chacune des deux routes), comme toute écriture refusée (AC 12) — rien à y ajouter,
    à ne pas chercher comme une voie manquante.
    **Forme imposée** (patron de `period_locked_failed_proposal`, `:186-193` — « un seul constructeur
    pour les DEUX sites ») : un constructeur `later_fiscal_year_closed_failed_proposal(bank_transaction_id,
    project_id, fiscal_year_id, fiscal_year_name)`, appelé par un bras de
    `project_error_to_failed_proposal` (à côté de `PeriodLocked`, `:282-290`) **et** par la branche en
    ligne de `accept_one_rule` (à côté de son `PeriodLocked`). Le code y est un **littéral**
    `"LATER_FISCAL_YEAR_CLOSED"` (F8 : visible au `grep` du décompte, ce que `err.error_code()` ne serait
    pas). **Tests, un par voie** (`reconciliation_e2e.rs` ou fichier du lot) : facture, ventilé, règle —
    chacun dans `failed[]` avec code et `details`, une seconde proposition saine du même lot acceptée,
    aucune écriture créée pour la proposition refusée (compte avant/après).
    **Côté écran**, `failed-proposal-label.ts` reçoit le code — **tranché (C100)** : clé neuve
    `reconciliation-failed-later-fiscal-year-closed` (« Exercice postérieur « { $name } » clôturé »),
    `$name` lu dans `details.fiscalYearName` (repli sans nom si le champ manque, testé). La clé globale
    de l'AC 9 ne convient pas : elle porte une phrase entière, trop longue pour une ligne d'échec. La
    fonction lit alors `details` pour **trois** codes : le doc-comment « Deux codes seulement… » (`:24`)
    est réécrit. **Recompte du doc-comment « Les 26 codes »** (`:9-13`) : sa formule (les littéraux de
    `grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u`, plus
    `ACCOUNT_NOT_POSTABLE` posé par `DbError::error_code()`) **reste** ; elle rend 25 littéraux sur
    `8f9811d8`, **26** avec le littéral neuf — total attendu **27**, recompté par la commande et non
    incrémenté.

12. **Inventaire fermé des écrivains, et preuve qu'aucun ne contourne le filet.** Les **22** routes
    `Rejouee` et les **2** `Exemptee` de production du registre (`audit_route_registry.rs:168-301`)
    se répartissent ainsi — table à reproduire **telle que recomptée en T0** dans les Dev Notes du Dev
    Agent Record. Commande de recompte **bornée à `LIB_ROUTES`** —
    `awk '/^const LIB_ROUTES/,/^const TEST_ENDPOINT_ROUTES/' crates/kesh-api/tests/audit_route_registry.rs | grep -c 'Rejouee)'`
    (→ 22) et `… | grep -c 'Exemptee('` (→ 2) ; ⚠️ sur le fichier entier, les mêmes `grep -c` rendent
    **25** et **6** (routes de test, `:304-305`, et mentions du doc-comment), et une borne `/^\];/`
    déborde sur les routes de test (→ 4), la fin de `LIB_ROUTES` s'écrivant `),];` :

    | Point de passage | Routes |
    |---|---|
    | `create_in_tx_inner` (AC 8) — 19 | `journal_entries::create_journal_entry`, `reverse_journal_entry` ; `opening_balances::generate_opening_balances`, `complete_opening_balances` ; `credit_notes::create_credit_note` ; `supplier_invoices::create_supplier_invoice`, `pay_supplier_invoice`, `cancel_supplier_invoice`, `cancel_supplier_invoice_settlement` ; `imported_supplier_invoices::complete_import` ; `payment_batches::confirm_payment_batch` ; `invoices::validate_invoice_handler`, `settle_invoice_handler`, `write_off_invoice_handler`, `cancel_invoice_settlement_handler` ; `reconciliation::post_accept`, `post_manual`, `post_split`, `post_cancel_reconciliation` |
    | `update` (garde 15-8a, inchangée) — 1 | `journal_entries::update_journal_entry` |
    | `delete_in_tx` (AC 10) — 2 | `journal_entries::delete_journal_entry` (garde 15-8b), `invoices::unvalidate_invoice_handler` (**neuf**) |
    | Hors filet, écrit — 2 | `admin::full_import` (remplace toute la base : c'est une **source** de l'état fautif, traitée par la détection de l'AC 15) ; `onboarding::reset` (efface la société entière) |

    **Ce que T0 doit prouver, et pas seulement relire** : qu'aucune instruction d'écriture sur
    `journal_entries` / `journal_entry_lines` hors tests n'échappe à ces trois points de passage.
    Commande de départ, élargie au SQL dynamique et à `TRUNCATE` :
    `grep -rnE "(DELETE FROM|UPDATE|INSERT INTO|INSERT IGNORE INTO|REPLACE INTO|TRUNCATE( TABLE)?) +\`?(journal_entr(y|ies)|\{)" crates --include=*.rs`
    — sites hors `mod tests` / `tests/` attendus : `journal_entries.rs` (insertions de
    `create_in_tx_inner` ; `UPDATE`/`DELETE` des lignes de `update` ; `DELETE` de `delete_in_tx` ;
    `delete_all_by_company` `:1859` — **tranché** : aucun appelant de production, `grep -rn
    "delete_all_by_company"` ne rend que des `mod tests`), `journal_entry_number_sequences.rs`
    (compteur, pas d'écriture comptable), `kesh-seed` (effacement de la démo, `lib.rs:257-272`) ; et les
    trois sites **dynamiques**, que le motif sur le nom de table ne voit pas : `kesh-db/src/backup.rs:457`
    (``DELETE FROM `{table}` ``) et `:509` (``INSERT INTO `{table}` ``) — c'est `admin::full_import`,
    hors filet (table ci-dessus) —, et `kesh-db/src/test_fixtures.rs:401` (`TRUNCATE TABLE {table}`,
    routes `/reset` et `/seed` du mode test, absentes en production). **Faux positifs connus** (F7 ;
    relevé `5e4bec50`), que le motif `TRUNCATE( TABLE)? +\{` attrape dans le nom de constante
    `TABLES_TO_TRUNCATE {` d'une boucle `for` — aucune instruction SQL : `kesh-api/src/admin_backup/export.rs:55`,
    `admin_backup/import.rs:118`, `:280`, `:367`, `kesh-db/src/backup.rs:453`, `test_fixtures.rs:400`
    (les instructions de ces boucles sont les sites dynamiques ci-dessus). Les sites de `accounts.rs`
    (`:1215`, `:1221`) et `invoices.rs` (`:4380`, `:4423`) sont dans leur `mod tests`. Tout **autre** site
    est un **trou** à fermer dans la story ou à écrire comme angle mort assumé (règle « inventorier les
    sites non résolus » du `CLAUDE.md`). ⚠️ **Le lettrage de la 15-1a** (un `UPDATE` de `journal_entry_lines`,
    colonnes de lettrage) n'existe pas encore sur `main` : s'il est mergé avant cette fiche, T0 le trouve
    par cette commande et l'écrit (hors filet, motif à la frontière ci-dessus). La contre-passation
    (`reverse_in_tx` / `reverse_owned_in_tx` → `reverse_in_tx_inner` → `create_in_tx_inner`,
    `journal_entries.rs:2313`) est datée du **jour** : elle tombe dans l'exercice ouvert du jour, et le
    filet ne la refuse que si cet exercice-là a un postérieur clos — c'est voulu (contre-passer dans
    l'exercice courant ne touche aucun bilan clos). ⚠️ Ce refus-là, les **prédicteurs d'annulation**
    ne l'annoncent pas : angle mort assumé, AC 18 (F1, C120).

### Les installations déjà dans l'état fautif

15. **Détection à l'écran, sans changement d'API.** La page `/settings/fiscal-years` calcule, depuis la
    liste qu'elle charge déjà, l'ensemble des exercices **ouverts suivis d'un exercice clos** (la
    fonction `nearestLaterClosed` existe, `+page.svelte:93-106`). S'il est non vide, un **bandeau
    d'avertissement** (`role="alert"`, `data-testid="fiscal-year-out-of-order"`) s'affiche au-dessus du
    tableau, **pour tous les rôles** (Consultation compris : c'est une information sur l'état des
    comptes). Il nomme `{open}` — le **plus ancien** exercice ouvert, qui est toujours concerné dès que
    l'état est fautif (tout exercice ouvert antérieur à un exercice ouvert concerné l'est aussi) —, son
    plus proche postérieur clos `{closed}`, et le plus récent exercice clos `{latest}`. Marche à suivre
    (C100), **dans cet ordre** :
    1. **la réparation directe** : clôturer « {open} » si ses comptes sont arrêtés — l'AC 1 de la 15-12a
       l'accepte, aucun exercice antérieur à « {open} » n'étant ouvert ; geste ouvert au Comptable ;
       puis, s'il en reste, les exercices ouverts suivants, du plus ancien au plus récent (le bandeau se
       recalcule à chaque rechargement de la liste) ;
    2. **sinon** : un administrateur rouvre les exercices clôturés postérieurs à « {open} », **en
       commençant par le plus récent** (« {latest} ») et jusqu'à « {closed} » — c'est le seul ordre que
       la garde LIFO accepte : rouvrir « {closed} » d'abord est refusé dès qu'un exercice plus récent est
       clos. ⛔ Le bandeau ne prescrit **jamais** de rouvrir « {closed} » en premier.
    Texte fr-CH de référence : « L'exercice « { $open } » est ouvert alors qu'un exercice postérieur,
    « { $closed } », est clôturé : rien ne peut y être enregistré tant que l'ordre n'est pas rétabli.
    Clôturez « { $open } » si ses comptes sont arrêtés, puis les exercices ouverts suivants, du plus
    ancien au plus récent. Sinon, un administrateur rouvre les exercices clôturés, en commençant par le
    plus récent, « { $latest } » : Kesh ne rouvre un exercice que si aucun exercice plus récent n'est
    clôturé. » Clé `fiscal-year-out-of-order-warning` (variables `$open`, `$closed`, `$latest`),
    4 locales ; test Vitest du texte et des trois noms, avec un cas où `{closed}` ≠ `{latest}`.
    ⚠️ Pas de détection au démarrage, pas de refus à l'import (C89). Le message de refus (AC 9 de la
    15-12a) ne renvoie pas à ce bandeau : il porte sa propre marche à suivre (C111).

16. **Réparation exerçable** (test `kesh-db`, état fautif posé par `UPDATE fiscal_years SET status =
    'Closed'` direct, comme les tests de la 15-8a) : N ouvert, N+1 clos → écrire dans N refusé
    (`LaterFiscalYearClosed`) ; `close(N+2)` (ouvert) refusé (`EarlierFiscalYearOpen`, N) ;
    `reopen(N+1)` **accepté** (LIFO : rien de postérieur clos) ; `close(N)` accepté ; écriture dans N
    refusée (`FiscalYearClosed`) ; `close(N+1)` accepté. État final sain. **Second scénario — la
    réparation directe du bandeau** : N ouvert (tous ses antérieurs clos), N+1 et N+2 clos →
    `reopen(N+1)` **refusé** (LIFO, N+2 clos : le geste que le bandeau ne doit pas prescrire) ;
    `close(N)` **accepté** → état sain, écriture dans N refusée `FiscalYearClosed`.

### Les autres écrans

18. **Inventaire des sites NON résolus** (règle du `CLAUDE.md`). L'inventaire ne part **pas** des formes
    traitées (`grep FISCAL_YEAR_CLOSED`) mais des **appelants** : pour chacune des 22 routes de l'AC 12,
    sa fonction cliente dans `frontend/src/lib` (grep de son chemin d'API) puis chaque écran qui
    l'appelle, et, pour chaque écran, ce qu'il fait d'une erreur de code inconnu. Verdict écrit par
    écran au Dev Agent Record : « retombe sur le message du serveur » (rien à faire, vérifié au code) ou
    « changé » (un test). Relevé de la validation P1 sur `9cb5083b`, à refaire en T0 :
    - **changé** — `invoices/[id]/+page.svelte`, boîte de validation : elle ne se ferme que sur
      `FISCAL_YEAR_INVALID` et `CONFIGURATION_REQUIRED` (`:429-433`) et resterait ouverte sur un refus
      que rien n'y rend corrigeable ; `LATER_FISCAL_YEAR_CLOSED` s'ajoute à la liste (test Vitest ou
      de composant) ;
    - **inchangés, vérifiés** — `notify.ts` (`notifyMissingFiscalYearOrFallback`, `:71-72`/`:100-104` :
      égalité stricte sur `FISCAL_YEAR_INVALID`, `NO_FISCAL_YEAR`, `FISCAL_YEAR_CLOSED` ; le nouveau
      code y retombe **déjà** sur le message du serveur — aucun test à écrire pour un changement qui
      n'existe pas) ; `payment-batches/[id]`, `settings/opening-balances`, `supplier-invoices/import`
      (`completeErrorLabel` → `default: err.message`), dévalidation de `invoices/[id]` (`:359`) :
      `err.message` ;
    - **à relever en T0** — les autres appelants (saisie et contre-passation des écritures, règlements
      et soldes, avoirs, factures fournisseur et leurs annulations, rapprochement manuel, ventilé et
      son annulation), et les dix fichiers qui citent `FISCAL_YEAR_CLOSED` (`grep -rln
      "FISCAL_YEAR_CLOSED" frontend/src --include=*.ts --include=*.svelte | grep -v test`), à titre de
      contrôle : un `switch` qui y traite `FISCAL_YEAR_CLOSED` peut avoir à traiter le nouveau code.

    **Les prédicteurs serveur** *(ajouté en P3 : F1, C120)* — l'inventaire part aussi des champs d'API
    qui **annoncent** à l'écran si une annulation est possible, calculés par la fonction même qui
    refuse : `cancelBlockedBy` (`routes/invoices.rs:1410`, `routes/reconciliation.rs:3925`),
    `settlementCancelBlockedBy` et `cancelBlockedBy` (`routes/supplier_invoices.rs:93`, `:111`) — tous
    issus de la queue commune `settlement_cancellation::settlement_entry_cancel_blocker` (rangs 2 à 5 ;
    `supplier_invoices::supplier_invoice_cancel_blocker` la compose) —, et `reversalBlockedBy`
    (`journal_entries::reversal_blockers`, fiche d'écriture). Relevé `5e4bec50`. **Site non résolu** : le
    rang 5 (`NoOpenFiscalYearToday`) contrôle qu'un exercice **ouvert** couvre le jour où la
    contre-passation serait datée, **pas** qu'aucun exercice postérieur à celui-ci n'est clos. Dans
    l'état hérité où l'exercice du jour est suivi d'un exercice clos — un exercice **futur** clôturé
    d'avance —, les quatre écrans annoncent l'annulation possible et le clic rend `400
    LATER_FISCAL_YEAR_CLOSED` (filet de l'AC 8), avec le message de l'AC 9 de la 15-12a, qui porte sa
    propre marche à suivre. `reversalBlockedBy` ne prédit déjà pas l'exercice du jour : même trou, déjà
    ouvert. **Tranché : angle mort assumé, écrit** (C120), non un rang neuf — le rang est tracé par l'issue
    **#568** (P3), que le doc-comment et `api-external.md` citent :
    - **où il est écrit** : au doc-comment de `settlement_entry_cancel_blocker`, à côté de la « Limite
      assumée » existante du verrou de période du jour (`settlement_cancellation.rs:38-40`) — même
      nature : un refus du socle, au clic, que la lecture ne reproduit pas ; dans `docs/api-external.md`,
      aux deux paragraphes qui documentent ces champs (`:353`, `:361`) et à celui des règlements client
      (`:319`) ; au Dev Agent Record ;
    - **ce qui le fixe** : un test (`kesh-db`, état posé par SQL) qui asserte, dans cet état,
      `settlement_entry_cancel_blocker` → `None` **et** l'annulation → `LaterFiscalYearClosed` —
      patron C-15-8-29 : il rougit le jour où le rang est ajouté, et son doc-comment le dit ;
    - **pourquoi pas le rang** : il ajouterait une variante à `SettlementCancelBlocker`, trois bras aux
      `match` exhaustifs de `kesh-api/src/errors.rs` (`:2988`, `:3519`, `:3556` sur `5e4bec50`), trois
      `switch` exhaustifs (`never`) côté écran (`settlement-cancel-blocked.ts`,
      `reconciliation-cancel.ts`, `invoice-cancel.ts`) avec leurs types, des clés ×4 et la documentation
      des champs — cinq modules de premier niveau de plus pour une fiche qui en compte déjà dix (§
      *Dérogation*), pour un état hérité étroit (un exercice futur clos) dont le refus au clic est
      exact et explique la marche à suivre. **Issue à ouvrir par l'orchestrateur** (P3 : écran qui
      annonce un geste que le serveur refuse) pour ajouter le rang hors de cette story.

### Ce qui doit être prouvé, et le reste

19. **Mutations — part B** (chacune jouée, observée rouge, restaurée — ⚠️ toucher le fichier après
    restauration, cargo garde le binaire muté) : (v) retirer le filet de `create_in_tx_inner` → les tests
    de l'AC 20 rouges ; (vi) rétablir `if enforce_ownership` dans `delete_in_tx` → test de dévalidation
    rouge ; **(vii-a)** retirer le bras `LaterFiscalYearClosed` de `project_error_to_failed_proposal` →
    tests **facture et ventilé** de l'AC 11 rouges (`DATABASE_ERROR`) — chacun des deux doit rougir,
    sinon l'un ne passe pas par la voie qu'il prétend exercer ; **(vii-b)** retirer l'appel du
    constructeur dans la branche en ligne d'`accept_one_rule` → test **règle** de l'AC 11 rouge. Les
    mutations (i)-(iv), (viii)-(x) sont dans la 15-12a. Résultat de chaque mutation au Dev Agent Record.

20. **Le filet, flux par flux.** Un test **par point de passage et par famille** — pas les 22 routes,
    mais au moins : saisie manuelle (`POST /journal-entries`, HTTP `400` et corps complet, message de
    l'AC 9), validation d'une facture, règlement client, paiement fournisseur, complément des soldes de
    départ, contre-passation **par la route** (`POST /journal-entries/{id}/reverse`, datée du jour :
    patron de la 15-8b, `journal_entries.rs:3444-3455` — un exercice postérieur **clos**, commençant le
    lendemain de la fin de l'exercice qui couvre le jour, posé par SQL direct, fait de l'exercice du jour
    l'exercice fautif), dévalidation, acceptation par lot (`failed[]` — les trois voies de l'AC 11). État
    fautif posé par SQL direct. Chaque test asserte aussi qu'**aucune** écriture n'a été créée ou
    supprimée (compte avant/après) — un refus qui laisse une trace partielle n'est pas un refus.

21. **Tests existants à reprendre — part B** (ceux que le filet change de sens : un exercice postérieur
    clos posé par SQL, puis une écriture ou une suppression — dévalidation — dans un exercice antérieur)
    — triage au gate, chaque rouge classé (a) scénario désormais refusé à bon droit → réécrit
    (l'écriture placée dans un exercice sans postérieur clos, ou le refus asserté si le test vise le
    filet), ou (b) défaut réel → corrigé. *(Réécrit en P3 : R1 = F3.)*
    **Inventaire par le SYMPTÔME, non par une forme** — le symptôme est « un exercice `Closed` en base de
    test ». La P2 cherchait la seule graphie `SET status = 'Closed'` : elle ne voyait ni les aides qui
    lient le statut par `?` (`set_status`, appelée par `exercice_de`), ni les `INSERT … 'Closed'`, ni
    `ON DUPLICATE KEY UPDATE status = 'Closed'`. Son « 20 sites » était le compte d'**une forme**, non de
    la classe. Commande (relevé sur `5e4bec50`) :
    `git grep -nE "'Closed'|\"Closed\"|status = \?" -- 'crates/*.rs'` — **72 lignes** (70 sur
    `8f9811d8`), triées en deux :
    - **28 lignes qui ne posent rien** : code de production qui lit ou écrit le statut
      (`fiscal_years.rs` ×6, `journal_entries.rs` `:53`, `:322`, `:663`, `:1097`, `:1378`, `:1689`,
      `accounts.rs:438`, `settlement_cancellation.rs:81`, `entities/fiscal_year.rs` ×2,
      `imported_supplier_invoices.rs:281` — autre table), doc-comments de routes
      (`routes/fiscal_years.rs` ×2, `routes/opening_balances.rs:95`), assertions sur une réponse
      (`fiscal_years_e2e.rs:681`, `opening_balances_e2e.rs:965`, `:1100`, `fiscal_years_repository.rs:318`,
      `:753`), constructeurs en mémoire sans base (`opening_complement.rs:730`, `:749`),
      `vat_report.rs:510` ;
    - **44 lignes — 42 sites — qui posent un exercice clos en base de test** (deux sites s'écrivent sur
      deux lignes : `journal_entries.rs:3258-3259`, `:3452-3453`). Chacun lu, avec la question qui décide :
      *le test écrit-il ou dévalide-t-il, après la pose, dans un exercice **antérieur** à l'exercice
      clos ?*

    | Sites (relevé `5e4bec50`) | Ce que le test fait après la pose | Verdict |
    |---|---|---|
    | `journal_entries.rs` `mod tests` `:3452-3453` (`la_devalidation_ne_voit_pas_l_exercice_posterieur`, C-15-8-29) | dévalidation (`delete_in_tx`, `enforce_ownership = false`) d'une écriture de N sous N+1 clos | **change de sens** — inversé par l'AC 10 (résolu) |
    | `journal_entries.rs` `:3258-3259` (aide de précédence 15-8b), `:3269` ; `journal_entries_modification.rs:395`, `:499` | `delete_in_tx(…, true)` ou `update` : gardes 15-8a/15-8b, inchangées | inchangé ; doc-comment réécrit par la 15-12a (AC 21) |
    | `journal_entry_reversal_e2e.rs` aide `set_status` `:1372` et ses appels `:1846`, `:2086`, `:2135`, `:2526`, `:2593` ; aide `exercice_de` (`:1353`, qui crée puis appelle `set_status`) aux appels clos `:1863-1865`, `:2077`, `:2510`, `:2585` | `PUT` / `DELETE` (gardes 15-8a/15-8b) ; les écritures et contre-passations de ces tests sont faites **avant** la pose, ou dans l'exercice du jour quand le seul exercice clos lui est **antérieur** (`:2135` : N-1 clos, écritures dans l'exercice courant) | inchangé — ⚠️ l'AC 21 de la P2 le disait « un seul exercice » : faux pour ce fichier, mais le verdict tient pour une autre raison |
    | `journal_entry_reversal_e2e.rs:1103`, `:2723`, `:2894` | `:1103`, `:2723` : un seul exercice ; `:2894` : clôt l'exercice **passé** puis contre-passe dans l'exercice courant, postérieur | inchangé |
    | `opening_complement_repository.rs` aide `set_status` `:197`, appels `:288`, `:311`, `:484`, `:685` | clôt le **premier** exercice (`co.fy[0]`) ; aucun exercice ne le précède | inchangé |
    | `opening_balances_e2e.rs:1492` (`WHERE company_id = ?`) | clôt **tous** les exercices : aucun ouvert ne précède un clos | inchangé |
    | `reconciliation_manual_e2e.rs:264` (aide `insert_closed_fiscal_year`, appelée `:1023`) | seul exercice de la société, clos | inchangé |
    | `accounts_e2e.rs:363` ; `accounts.rs` `mod tests` `:1453`, `:1562`, `:1753` ; `period_lock_e2e.rs:608`, `:709` ; `invoice_settlement.rs:1212` ; `invoice_write_off.rs:756` ; `invoices_validate_vat.rs:1330` ; `reconciliation_cancel.rs:133` ; `supplier_invoices_repository.rs:1660`, `:1966`, `:2240` ; `invoices.rs` `mod tests` `:4876`, `:4954` | un seul exercice dans le montage (société propre, ou exercice du jour sur la base partagée) : aucun postérieur | inchangé |

    **Liste fermée des sites qui changent de sens : un seul**, `journal_entries.rs` `:3452-3453`, résolu
    par l'AC 10. `invoices_validate_vat.rs:1330` et `supplier_invoices_repository.rs:2240` sont **neufs**
    depuis `8f9811d8` (15-5d) ; les numéros d'`invoices.rs` `mod tests` ont glissé (`:4797` → `:4876`,
    `:4875` → `:4954`). T0 refait la commande sur le `main` du moment et trie **chaque ligne neuve** avec
    la même question ; la table recomptée va au Dev Agent Record. Le triage au gate reste le filet des
    rouges ; cet inventaire est celui des tests qui changeraient de sens **sans** rougir.
    ⚠️ **Pollution de la base partagée, aggravée** : le filet rend fatal, pour **toute** création
    d'écriture de la société, un exercice clos **postérieur** laissé par un test interrompu, et
    `ensure_open_fiscal_year` (`journal_entries.rs:2483-2523`) ne répare que l'exercice qui **couvre** le
    jour. Décision (C100) : les tests qui posent un postérieur clos le retirent en fin de test (patron
    15-8a), et le mode d'échec est écrit dans `docs/testing.md` § « Base de dev jetable » (la remise à
    zéro inconditionnelle du `CLAUDE.md` le couvre) ; pas de réparation ajoutée à
    `ensure_open_fiscal_year`, qui masquerait un résidu au lieu de le signaler.

23. **Documentation — part B.**
    - **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : l'item « Verrouille l'exercice »
      (`:708`) perd la parenthèse « pour la modification et la suppression seulement… limite suivie par
      l'issue #543 » et dit que **rien** ne s'écrit plus dans un exercice suivi d'un exercice clos ;
      § « Réouverture » (`:717-723`) — **la procédure de réparation** d'un état hérité, **celle du
      bandeau** (AC 15, dans le même ordre : d'abord clôturer l'exercice ouvert le plus ancien ; sinon
      rouvrir à partir du plus récent clos — jamais « rouvrir l'exercice clos le plus proche » en
      premier) ; conditions de modification (`:494-500`) — retirer « ⚠️ Cette garde ne couvre encore…
      #543 » ; soldes de départ (`:752`) — « Si un exercice suivant est déjà clôturé, son bilan reporté
      l'est aussi » devient faux : réécrire (le complément est refusé tant qu'un exercice postérieur est
      clôturé) ; note de la contre-passation (`:627`) relue ; **liste des refus de la dévalidation**
      (`:1314-1337`, « Le refus nomme toujours son motif », puce de l'exercice clos `:1333`) — puce neuve
      « écriture d'un exercice **suivi d'un exercice clôturé** », à son rang de précédence (AC 10, après
      « exercice clos »). **Inventaire des sites par recherche, non par énumération** : `grep -nE
      "exercice (est )?(clos|clôturé|fermé)|exercice postérieur|période verrouillée|refuse"
      user-manual.tex` ; chaque site rendu est soit modifié, soit écrit « inchangé » au Dev Agent Record
      avec son motif (relevé P1 : `:352`, `:540`, `:582`, `:593`, `:1174`, `:1394`, `:1416`, `:1610` —
      liste « motif du refus en clair : … etc. » du lot, qui reste vraie —, `:1698-1702`, `:2219-2228`
      restent vrais ; la part A a déjà traité `:697-723` pour la clôture).
    - **Manuel admin** (`admin-manual.tex`, recalé sur `8f9811d8`) : § « Export/import d'installation via
      l'interface Kesh » (`:1641`), « Importer (restaurer la même installation) » (`:1657`), « Reprises de
      données rejouées à l'import » (`:1674`) — une sauvegarde d'une version antérieure peut porter des
      exercices clôturés dans le désordre : l'écran le signale, voici la réparation (celle du bandeau,
      même ordre). Le PDF admin est aplati et contrôlé comme l'utilisateur.
    - **PDF régénérés** (`make fr` dans `docs/manual/`), contrôlés **aplatis**
      (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : plus aucune mention « #543 » ni « limite »
      attachée à la garde ; les phrases neuves présentes.
    - **`docs/api-external.md`** — inventaire des sites **par recherche** : `grep -nE
      "FISCAL_YEAR_CLOSED|FISCAL_YEAR_INVALID|PERIOD_LOCKED|failed: \[|ordre de précédence"
      docs/api-external.md` rend tous les tableaux et listes de refus des routes qui écrivent au journal ;
      **chaque** site rendu est traité ou écrit « inchangé, motif » au Dev Agent Record. Relevé
      `8f9811d8` (numéros identiques à `9cb5083b`) :
      - `:253` — l'avertissement « ne garde aujourd'hui que ce `PUT` et la suppression… #543 » est
        supprimé ; `:238-251` (`PUT`) et `:270-279` (`DELETE`) : déjà conformes ;
      - **dévalidation** `:287-297` — ligne `LATER_FISCAL_YEAR_CLOSED` (`400`) après `FISCAL_YEAR_CLOSED`
        (précédence de l'AC 10) ; la table des `details` (`:301-309`) gagne sa forme propre
        (`fiscalYearId` / `fiscalYearName`, **pas** `documentId` / `documentNumber`) ; la phrase `:311`
        (« Les **cinq autres** codes… 10 lignes en tout, 5 avec `details` et 5 sans ») est **recomptée
        depuis le tableau** (attendu : 11 lignes, 6 avec `details`, 5 sans — à vérifier au tableau
        final, pas à recopier d'ici) ;
      - **annulation d'un règlement client** `:321-331`, **solde du reste** `:338-347`, **annulation
        d'un rapprochement** `:381-391` — ligne `LATER_FISCAL_YEAR_CLOSED` (pour les annulations : la
        contre-passation datée du jour, refusée si l'exercice du jour est suivi d'un exercice clos) ;
      - **listes** `:355` (annulation du règlement fournisseur) et `:363` (annulation d'une facture
        fournisseur, « dans l'ordre de précédence ») — le code inséré **à son rang**, relu au code en
        T0 (il naît de `create_in_tx_inner`, donc après `FISCAL_YEAR_INVALID`) ;
      - **acceptation par lot** `:371` — `LATER_FISCAL_YEAR_CLOSED` cité parmi les `errorCode` de
        `failed[]`, avec ses `details`, pour les trois voies (sans `projectId` pour la voie « règle »,
        AC 11) ;
      - **prédicteurs** `:319` (règlements client, `cancelBlockedBy`), `:353` (`settlementCancelBlockedBy`),
        `:361` (`cancelBlockedBy` de la facture fournisseur) — l'angle mort de l'AC 18 : le champ peut
        dire l'annulation possible et le clic rendre `LATER_FISCAL_YEAR_CLOSED` (C120) ;
      - `:484` (tableau des erreurs) — routes concernées : toute écriture au journal (dont
        `POST /journal-entries`, qui n'a pas de section propre), la dévalidation, `failed[]` de
        `POST /reconciliation/accept` ; retrait de « ne garde pas encore les autres chemins (#543) »
        (la 15-12a y a déjà ajouté `EARLIER_FISCAL_YEAR_OPEN` et la création d'exercice).
    - **`CHANGELOG.md` `[0.13.0]`** : l'entrée de la 15-12a sous `### Corrigé` est complétée (#543 : le
      filet, l'écran, la réparation des états hérités — `closes #543`) ; la parenthèse de l'entrée #532
      (`:15`, « garde posée sur la modification et la suppression : la saisie, la contre-passation… ne la
      portent pas encore, #543 ») est réécrite — **propagation** : `grep -rn "543"` sur `CHANGELOG.md docs
      crates frontend/src` ne doit plus rendre de « limite connue » (relevé `8f9811d8` : `CHANGELOG.md:15`,
      `api-external.md:253`, `:484`, `user-manual.tex:500`, `:708`, commentaires `C-15-8-29`).

## Tasks / Subtasks

- [ ] **T0 — Relevés au sol sur le `main` du moment** (AC 11, 12, 18, 21) *(ex-T0, part B)*
  - [ ] Refaire les numéros de ligne cités ; recompter le registre (22 / 2) ; produire la table de l'AC 12.
  - [ ] Inventaire des écritures SQL sur `journal_entries` / `journal_entry_lines` hors tests (AC 12) ;
        `delete_all_by_company` reconfirmé ; lettrage de la 15-1a s'il est mergé.
  - [ ] Les trois sites `create_in_tx` du lot et leur traitement d'erreur (AC 11) ; les deux routes
        unitaires (`post_manual_once`, `post_split_once`) et leur conversion d'erreur.
  - [ ] Relever les sites frontend et les prédicteurs serveur de l'AC 18 ; refaire la commande de
        l'AC 21 par le symptôme et trier chaque ligne neuve.
  - [ ] Constater si la 15-1a est mergée (`grep -n "ENTRY_LETTERED\|EntryLettered"
        crates/kesh-db/src/repositories/journal_entries.rs`) : si oui, la paire de précédence de C117
        revient à cette fiche (frontière).
- [ ] **T1 — Le filet** (AC 8, 10, 12, 20) *(ex-T4, sans le message, passé à la 15-12a)* —
      `create_in_tx_inner`, `delete_in_tx` sans condition, test C-15-8-29 inversé, précédence de la
      dévalidation et sa paire, tests flux par flux ; doc-comments complétés (module `fiscal_years.rs`,
      `DbError::LaterFiscalYearClosed`, `find_later_closed` — second appelant et raison de l'absence de
      verrou —, étape 1 de `create_in_tx_inner`) ; ligne `admin::full_import` du registre (« exclusif »
      devient « exclusif des autres imports seulement »). **Vérification que le filet n'ajoute aucun
      verrou** (R6, reprise de l'ancien AC 14) : `grep -rnE "find_later_closed(_in_tx)?\("
      crates/kesh-db/src` — la version `_in_tx` n'est appelée que par `update`, `delete_in_tx`, `reopen`
      et `create` d'exercice ; la version non verrouillante, que par le motif d'écran de
      `GET /journal-entries/{id}` et par `create_in_tx_inner` (neuf). Résultat au Dev Agent Record.
      Angle mort des prédicteurs écrit (AC 18, C120) : doc-comment de
      `settlement_entry_cancel_blocker`, test qui le fixe.
- [ ] **T2 — Lot de rapprochement** (AC 11) *(ex-T5)* — constructeur, bras de
      `project_error_to_failed_proposal`, branche d'`accept_one_rule`, libellé écran, recompte des codes
      (27), tests Rust par voie et Vitest.
- [ ] **T3 — Écran : bandeau et autres écrans** (AC 15, 18) *(ex-T7, part B)* — bandeau (deux voies,
      trois noms), clé `fiscal-year-out-of-order-warning` ×4, inventaire de l'AC 18 par les appelants
      (boîte de validation d'une facture) ; `npm run lint-i18n-ownership` ; Vitest.
- [ ] **T4 — Réparation** (AC 16) *(ex-T8)*.
- [ ] **T5 — Tests existants** (AC 21, part B) *(ex-T9, part B)* — triage écrit, test par test, `mod
      tests` de `src/` compris ; mode d'échec de la base partagée écrit dans `docs/testing.md`.
- [ ] **T6 — Mutations** (AC 19, part B) *(ex-T11, part B)*.
- [ ] **T7 — Documentation** (AC 23, part B) *(ex-T12, part B)* — manuels + PDF aplatis,
      `api-external.md`, CHANGELOG, grep `543`.
- [ ] **T8 — Gates** *(ex-T13)* — base remise à zéro (DROP/CREATE de **ses** bases, jamais de
      redémarrage du conteneur) ; `scripts/test-fast.sh` complet (story `kesh-db` : gate complet même en
      cours de boucle) ; frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au
      **dernier commit de code**, jugé fichier par fichier contre `docs/testing.md` § « Les échecs
      attendus ». Ne déclarer que ce qui a tourné.

## Dev Notes

### Pourquoi deux points de passage suffisent

Découverte de la spécification (15-12, T0) : les dix-neuf routes qui créent une écriture passent toutes
par `create_in_tx_inner`, et les deux qui en suppriment une par `delete_in_tx`. Le filet coûte donc
deux sites et non dix-neuf ; l'inventaire de l'AC 12 est ce qui le prouve, et c'est à T0 de le refaire.

### La preuve du filet non verrouillant (AC 8)

*(Réécrite en P1, C100 ; répartie à la découpe : (α) est tenue par la 15-12a.)*

1. **(α) L'invariant I** est vrai de tout état **validé** obtenu par les transitions de l'application
   depuis un état sain (clôture : AC 1 ; création : AC 5 ; réouverture : LIFO — tous de la 15-12a).
   C'est lui qui dépend de la lecture verrouillante de la clôture, face à la course réouverture/clôture
   (15-12a, test 13 a, mutation (ii)).
2. **(β) Le verrou de l'écrivain** : un écrivain dans N tient le verrou `FOR UPDATE` de N dès
   l'étape 1 de `create_in_tx_inner` (`journal_entries.rs:310-317`) — ou avant, par
   `find_open_covering_date` — et lit son statut **sous ce verrou** (verdict « exercice clos » de
   l'étape 1, sur l'état courant, non sur la vue).
3. Sous (α), un exercice L > N ne peut être clos qu'une fois N clos ; sous (β), N reste ouvert jusqu'au
   `COMMIT` de l'écrivain. Donc, si l'état était sain quand l'écrivain a lu N ouvert sous verrou, aucun
   postérieur clos n'apparaît avant son `COMMIT`, et la lecture du filet — même sur une vue plus
   ancienne — n'a rien de neuf à voir. Si l'état était **hérité** fautif, il ne peut que se résorber
   (réouverture LIFO, clôture de N) : une vue ancienne ne produit qu'un **refus de trop**, jamais une
   acceptation de trop.
4. ⚠️ **Angle mort assumé — la restauration en vol** (C100). `admin::full_import` ne se sérialise
   qu'avec les autres imports (verrou `_kesh_version id = 1 FOR UPDATE`, `routes/admin.rs:173-179`),
   **pas** avec les écrivains : un écrivain dont la vue précède une restauration peut lire par le filet
   la table `fiscal_years` d'avant, alors que son verrou porte sur la ligne restaurée. Rien dans le code
   ne l'exclut ; c'est le cas, préexistant et plus large, de toute écriture en vol pendant une
   restauration. Écrit tel quel au doc-comment du module et à la ligne `admin::full_import` du registre.
5. Si une future transition écrit `fiscal_years.status` sans passer par `close` / `reopen`, la preuve
   tombe : le doc-comment du module le dit (15-12a, AC 7).

### Ordre des verrous — ce qui change, ce qui ne change pas

- **Change** : `delete_in_tx` prend les postérieurs (`find_later_closed_in_tx`) aussi pour la
  dévalidation.
- **Ne change pas** : les dix-neuf flux de création (lecture non verrouillante) ; `update` ; `reopen`.
- Dévalidation contre clôture : aucun cycle trouvé à la lecture **sous le plan par l'index** — la
  dévalidation tient N par clé primaire puis parcourt les postérieurs ; la clôture d'un Y > N (15-12a,
  C119) verrouille ses antérieurs un par un, **N compris**, avant Y : elle attend N sans rien tenir que
  la dévalidation demande (ses verrous déjà pris sont antérieurs à N). Sous un plan qui ne suit pas
  l'index, le parcours des postérieurs peut demander des antérieurs que la clôture tient : cycle
  possible, absorbé par le rejeu des **deux** routes (`unvalidate_invoice_handler` est `Rejouee`, la
  clôture le devient). T0 le confirme. La ligne `DELETE` de
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (`:327`, « No known cycle ») est relue et reste vraie, ou est
  corrigée.

### Codes et messages

| Situation | Code | HTTP | Clé |
|---|---|---|---|
| Écriture / dévalidation sous un postérieur clos | `LATER_FISCAL_YEAR_CLOSED` | 400 | `error-later-fiscal-year-closed` (posée par la 15-12a) |
| Idem, dans `failed[]` d'un lot (trois voies) | `LATER_FISCAL_YEAR_CLOSED` | 200 | `reconciliation-failed-later-fiscal-year-closed` (neuve, `$name`) |
| Bandeau d'état hérité | — | — | `fiscal-year-out-of-order-warning` |

### Ce qui doit être préservé

- Le comportement de la 15-8a/15-8b (`PUT`, `DELETE`, écran de la fiche) — codes, `details`, ordre.
- Les tests qui posent l'état fautif par SQL : ils sont les seuls moyens de l'exercer, ne pas les
  « simplifier » en passant par l'API.
- `admin::full_import` : accepte une sauvegarde fautive (C89).

### Hors périmètre, et écrit

- Une route de réparation automatique — la réparation est comptable (rouvrir ou clôturer), elle passe
  par les gestes existants.
- La détection de l'état hérité ailleurs que sur l'écran des exercices (tableau de bord, journal au
  démarrage) — C89.
- Le lettrage dans l'état hérité (15-1a, C112).

### Dérogation règle de splitting

*(Section rédigée en P3 — F9, décision de l'orchestrateur, C121.)*

- **Le critère franchi** : le premier critère de la § « Règle de splitting préventif » du `CLAUDE.md`
  (plus de 5 modules). Recompté en P3, au même critère que la 15-12a (un module dont seul un
  doc-comment change compte) : **10** modules de premier niveau — `kesh-db/repositories/journal_entries`,
  `kesh-db/repositories/invoices` (doc-comment de `unvalidate`, commentaires C-15-8-29),
  `kesh-db/repositories/fiscal_years` et `kesh-db/errors` (doc-comments complétés),
  `kesh-db/repositories/settlement_cancellation` (doc-comment de l'angle mort, AC 18 — **neuf en P3**,
  C120) ; `kesh-api/routes/reconciliation` ; `kesh-i18n` ; `frontend/routes/(app)/settings/fiscal-years`
  (bandeau), `frontend/features/reconciliation`, `frontend/routes/(app)/invoices/[id]` (boîte de
  validation). Hors décompte : tests, doc, écrans de l'AC 18 au verdict « inchangé ».
- **Pourquoi l'exception codifiée s'applique** : la story a **déjà** été découpée une fois (C107), selon
  la seule couture naturelle — l'ordre (15-12a) et le filet (cette fiche). Un second découpage
  séparerait le filet de son inventaire (AC 8, 10, 12 : les deux points de passage ne prouvent rien sans
  la preuve qu'aucun écrivain ne les contourne), ou le lot de ses voies (AC 11), ou le bandeau de la
  réparation qu'il prescrit (AC 15, 16) : des sous-stories qui ne seraient **pas testables isolément**
  — l'exception que le `CLAUDE.md` écrit (« merges intermédiaires impossibles à tester en isolation »).
  Sept des dix modules ne portent qu'un ou deux sites (trois doc-comments, un bras et un constructeur,
  un libellé, une condition de fermeture, des clés).
- **Risque accepté** : une revue de code qui embrasse dix modules relit moins finement chacun ; la
  remédiation d'une passe peut introduire un défaut dans un module que la passe suivante ne relit pas.
  Mitigation : la passe ciblée sur chaque remédiation (§ *La passe ciblée*), les inventaires par le
  symptôme (AC 12, 18, 21, 23) refaits à chaque passe, et le rang neuf des prédicteurs **sorti** de la
  story (C120) plutôt qu'ajouté à cinq modules de plus.
- **Signal déclaré au Project Lead** (Guy), dans cette fiche et au registre (C121).

### Tests — ce qui rendrait un test vert sans rien prouver

- Un état fautif posé **dans la transaction même** de l'écrivain : il verrait sa propre écriture ;
  le poser par une connexion distincte, validée.
- Un test de refus qui ne compte pas les écritures avant/après (AC 20).
- Un test du lot qui ne passe que par une voie (facture) en croyant couvrir les trois (AC 11, mutations
  (vii-a) et (vii-b)).
- **Angle mort assumé — le bandeau n'a pas d'E2E** (R6, phrase de l'ancien AC 22 rendue à cette fiche) :
  l'état fautif ne s'obtient pas par l'API (la 15-12a le rend inatteignable), et le montage E2E n'a pas
  de SQL direct ; le bandeau de l'AC 15 est couvert par Vitest (texte, trois noms, cas `{closed}` ≠
  `{latest}`), **pas** par Playwright. Écrit au Dev Agent Record.
- Une mutation restaurée par `cp`/`mv` sans `touch` : le binaire muté reste en cache.

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/journal_entries.rs`, `crates/kesh-db/src/repositories/invoices.rs`
(commentaires C-15-8-29, précédence de `unvalidate`), `crates/kesh-db/src/repositories/fiscal_years.rs`
et `crates/kesh-db/src/errors.rs` (doc-comments complétés),
`crates/kesh-db/src/repositories/settlement_cancellation.rs` (doc-comment de l'angle mort, AC 18) et le
test qui le fixe, `crates/kesh-api/src/routes/reconciliation.rs`,
`crates/kesh-api/tests/audit_route_registry.rs` (ligne `admin::full_import`), tests listés aux AC 11,
16, 20, 21, `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` et son test,
`frontend/src/lib/features/reconciliation/failed-proposal-label.ts` et son test,
`frontend/src/routes/(app)/invoices/[id]/+page.svelte` (AC 18), autres sites de l'AC 18 si T0 en trouve,
`docs/manual/fr/{user,admin}-manual.{tex,pdf}`, `docs/api-external.md`, `docs/testing.md` (mode d'échec
de la base partagée, AC 21), `CHANGELOG.md`. **Aucune migration** (P1-P8 sans objet).

### References

- Issue #543 et ses deux commentaires (15-8a, 15-8b).
- `crates/kesh-db/src/repositories/fiscal_years.rs` — `find_later_closed_in_tx` `:646`,
  `FIND_LATER_CLOSED_SQL` `:668`, `find_later_closed` `:678`.
- `crates/kesh-db/src/repositories/journal_entries.rs` — `create_in_tx_inner` `:258` (étape 1
  `:308-335`), `update` `:1260` (refus `:1378-1403`), `delete_in_tx` `:1648` (`:1681`).
- `crates/kesh-api/src/routes/reconciliation.rs` — `period_locked_failed_proposal` `:193`,
  `project_error_to_failed_proposal` `:244`, `accept_one_invoice` `:1225`, `accept_one_split` `:1879`,
  `accept_one_rule` `:2282`.
- `crates/kesh-api/tests/audit_route_registry.rs:168-301`.
- `crates/kesh-report/src/balance_sheet.rs:1-20` (bilan cumulatif).
- Fiches `15-12a-cloture-dans-l-ordre.md` (prérequis), `15-12-cloture-dans-l-ordre.md` (index ; version
  complète au commit `dae3a618`), `15-8a-modifier-une-ecriture.md`, `15-8b-supprimer-une-ecriture.md`
  (C-15-8-22, C-15-8-29), `15-5e1-socle-rejeu.md`, `15-1a-socle-lettrage.md` (frontière).
- `CLAUDE.md` § Pattern batch, § Règle de splitting, § Inventorier les sites non résolus,
  § Propagation post-patch.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-09 — **Créée au découpage de la 15-12** (remédiation de la validation P2, décision de
  l'orchestrateur, C107). Historique de la 15-12 (création C89 ; validation P1, Opus ×2 : 1 HIGH / 4
  MEDIUM / 14 LOW distincts, C100) : Change Log de l'index `15-12-cloture-dans-l-ordre.md`.
  **Validation P2** (Sonnet ×2, contexte frais ; rapports `target/gate-logs/15-12-p2-{R,F}.md`) : **0
  CRITICAL, 0 HIGH, 4 MEDIUM, 10 LOW distincts** (14 sur 18 bruts ; détail au Change Log de la 15-12a).
  Trend : P1 1 HIGH / 4 MEDIUM → P2 0 HIGH / 4 MEDIUM. Traités **ici** : **R1 = F1** (MEDIUM) — l'AC 11
  nomme les trois voies, dont la branche en ligne d'`accept_one_rule`, un constructeur unique, un test
  par voie, mutations (vii-a) et (vii-b) (C108) ; **F8** — code posé par un littéral, recompte à 27 par
  la commande ; **F10** (part B) — 20 sites de tests relevés par recherche à l'AC 21 (dont les 8 de la
  lentille, dans 5 fichiers, et 4 sites non cités de fichiers cités) ; **R6** (part B) — frontière avec
  la 15-1a sur `delete_in_tx` et le lettrage (C112) ; **R5 = F6** (part B) — manuel administrateur
  recalé (`:1641`, `:1657`, `:1674`). Le reste (F3/R3 découpage, R2, F2, R4/F5, R7/F9, F4, F11) est dans
  la 15-12a. Comptes : **11 AC** (8, 10, 11, 12, 15, 16, 18, 19, 20, 21, 23 — numéros d'origine),
  **9 tâches** (T0-T8), recomptés. Passe suivante : cf. index.
- 2026-10-09 — **Validation P3** (Opus ×2, contexte frais, sur les deux fiches ; rapports
  `target/gate-logs/15-12-p3-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 15 LOW distincts** (détail et
  origine au Change Log de la 15-12a ; deux MEDIUM nés de la remédiation P2). Trend : P1 1 HIGH / 4
  MEDIUM → P2 0 HIGH / 4 MEDIUM → P3 0 HIGH / 3 MEDIUM. Traités **ici** : **R1 = F3** (MEDIUM) — AC 21
  part B réécrit **par le symptôme** (`'Closed'`, `"Closed"`, `status = ?` des aides `set_status` /
  `exercice_de`, `INSERT … 'Closed'`, `ON DUPLICATE KEY UPDATE`) : 72 lignes sur `5e4bec50`, 28 qui ne
  posent rien, 44 lignes = 42 sites qui posent un exercice clos, **tous lus** ; liste fermée des sites qui
  changent de sens : **un seul** (`journal_entries.rs:3452-3453`, C-15-8-29), résolu par l'AC 10 ; « 20
  sites » requalifié en compte d'une forme ; la lecture « un seul exercice » de
  `journal_entry_reversal_e2e.rs` corrigée (le verdict « inchangé » tient pour une autre raison) ;
  **F1** (MEDIUM) — prédicteurs serveur ajoutés à l'inventaire de l'AC 18 ; le rang « exercice du jour
  suivi d'un exercice clos » **non** ajouté : angle mort assumé, écrit au doc-comment de
  `settlement_entry_cancel_blocker`, dans `api-external.md` et fixé par un test, issue à ouvrir (C120 —
  l'option préférée par l'orchestrateur, écartée pour son coût : cinq modules de plus) ; **C117** —
  précédence 2-bis avant 3-ter-bis dans `delete_in_tx`, écrite et testée par la 15-1a dans l'ordre
  préféré, par cette fiche si la 15-1a est mergée d'abord (frontière, T0) ; **R4** — AC 11 : les deux
  sites unitaires (`:3304`, `:3756`) nommés et écartés du lot, `projectId` omis pour la voie « règle »
  (C123) ; **F7** — faux positifs de la commande de l'AC 12 listés ; **R6** — angle mort « bandeau sans
  E2E » rendu aux Dev Notes, vérification « le filet n'ajoute aucun verrou » rendue à la T1 ; **F9**
  (part B) — section « Dérogation règle de splitting », modules recomptés : **10** (le doc-comment de
  `settlement_cancellation` s'ajoute, C120 ; C121) ; références `FIND_LATER_CLOSED_SQL` `:668`.
  Comptes : **11 AC** (8, 10, 11, 12, 15, 16, 18, 19, 20, 21, 23), **9 tâches** (T0-T8), recomptés.
  Passe suivante : cf. index.

- **2026-10-09 — validation P4 ciblée (Haiku, prompt `15-12b-validate-prompt-p4-ciblee.md`)** : rapport
  `target/gate-logs/15-12b-p4-ciblee.md`, 3 MEDIUM + 1 LOW annoncés. **Les trois MEDIUM sont écartés comme
  erreurs de catégorie**, vérifiées par l'orchestrateur : la lentille a cherché **dans le code** le test
  `la_devalidation_ne_voit_pas_l_exercice_posterieur` (M1), le doc-comment C120 de
  `settlement_entry_cancel_blocker` (M2) et l'entrée CHANGELOG qui ferme #543 (M3) — trois livrables que
  cette fiche **prescrit** et que le développement écrira ; aucun n'est un défaut de la spec. Son
  inventaire « 60 lignes » est faux : la commande de l'AC 21 rejouée sur `origin/main` (`5e4bec50`) rend
  **72** lignes, dont les deux de `journal_entries.rs:3452-3453`, conformément à la fiche. Axes repris par
  l'orchestrateur : (1) recompte ci-dessus ; (2) C120 — **un LOW réel** : l'issue du rang écarté (**#568**,
  ouverte le 2026-10-09) n'était pas citée ; ajoutée au paragraphe C120, avec obligation de la citer au
  doc-comment et dans `api-external.md` ; (3)-(5) AC 11/12/18, C117, dérogation (10 modules) et rubriques
  du CHANGELOG relus au diff `c7a5dbcc` : cohérents. **Bilan P4 : 0 au-dessus de LOW — validation close.**
  Trend : P1 (15-12 entière) 1 HIGH / 4 MEDIUM → P2 0 HIGH / 4 MEDIUM → P3 0 HIGH / 3 MEDIUM → P4 ciblée 0.
  Modèles : Opus, Sonnet ×2, Opus ×2, Haiku (ciblée).

