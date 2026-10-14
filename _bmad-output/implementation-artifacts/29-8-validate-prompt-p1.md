# Prompt — validation P1 de la spec, Story 29-8 (une transaction réglée depuis la fiche se rattache à ce règlement)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` (commit `a1b7cd94`).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-8*, § *Ordre*, § *Releases*. Issue #670
(l'objet) : `gh api repos/guycorbaz/kesh/issues/670` ; voisines #666 (piste b), #548 (une seule borne), #651 (clôture
concurrente). Fiches **non mergées**, en lecture seule :
`/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` (et son registre
`epic-29-choix-autonomes-29-2.md`, C-29-2-8),
`/home/gcorbaz/devel/kesh-29-6/_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md` (AC7, AC10),
`/home/gcorbaz/devel/kesh-29-1/_bmad-output/implementation-artifacts/29-1-preparer-cloture.md` (contrôle K2).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-8.md` (C-29-8-1 à 16).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous et naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Les prémisses de #670*) : revérifie chacune au code de `fcd2ae75` — rejet qui laisse `pending`,
   `is_reconciled`, décompte de `close_in_tx`, les cinq sites qui posent `matched_entry_id`, l'état que produit
   `accept_one_invoice`. Une prémisse mal établie est au moins **MEDIUM**.
2. **Le choix de la voie (C-29-8-3)** : les raisons d'écarter (b) et (c) sont-elles **vraies au code** ((b) ne couvre
   pas la 29-6 ; (c) est breaking et muette) ? La restriction aux règlements clients par virement produit-elle
   **réellement** l'état d'une proposition acceptée — et **toutes** les gardes en aval le traitent-elles comme tel
   (rang 3 de `settlement_cancellation`, motif 5 de la dévalidation, `is_reconciled` de la suppression d'import,
   `reconciliation_cancel::classify`, `list_by_company_with_balances`, `journal_entries::reversal_blockers`, export
   CSV, sauvegarde) ? Cherche un consommateur de `matched_entry_id` qui suppose que l'écriture a été **créée par** le
   rapprochement (date de l'écriture = date de valeur, journal Banque, description, audit `reconciliation.accepted`).
   Un consommateur que le rattachement fausserait est au moins **HIGH**.
3. **Les quatre conditions (C-29-8-4 à 6)** : le faux rattachement est-il **impossible ou visible** ? Même compte
   (`settlement_bank_account_id`), écriture non rapprochée (invariant « une écriture, une transaction »), écart
   `0 ≤ t − M ≤ 0,05` (sens, bornes, `M` lu sur le compte du grand livre **actuel**, cas d'une écriture à trois lignes
   avec arrondi), absence de fenêtre de dates (l'argument du dialogue préremplissant au jour est-il vrai,
   `SettleInvoiceDialog.svelte`) ? Le contournement de la 29-6 tel que sa fiche l'écrit produit-il bien un écart
   **positif** de 1 à 5 centimes, et celui de la 29-2 un écart nul ? Une condition qui laisse passer un faux
   rattachement muet est **HIGH**.
4. **Verrous et naissance de la vue (B5, C-29-8-12)** : l'ordre de l'AC2 étape 4 ; relis les ordres réels de
   `cancel_settlement_in_tx`, `reconciliation_cancel::cancel_in_tx`, `accept_one_invoice`, `settle_invoice`,
   `journal_entries::delete_in_tx` / `reverse_owned_in_tx` et `bank_accounts::close_in_tx` : un cycle peut-il se
   fermer avec le rattachement ? Une lecture non verrouillante précède-t-elle un verrou (dont `with_account_lock` : un
   `SELECT GET_LOCK` fait-il naître la vue) ? La requête de l'AC1 rejouée en lecture ordinaire après les verrous
   voit-elle bien l'état courant des lignes verrouillées et de `journal_entry_lines` ? La course T16 est-elle la
   bonne, et suffit-elle ? Un cycle ou une vue mal née est au moins **HIGH**.
5. **Inventaire clos** (AC7, Dev Notes *Inventaire (a)* et *(b)*) : refais les `grep` et **élargis-les** — tout texte
   (code, doc-comment, manuel, `api-external.md`, i18n, tests) qui suppose qu'une écriture rapprochée a été créée par
   le rapprochement, ou qui énumère les chemins de rapprochement. Les angles morts L1 à L5 sont-ils complets (paiement
   fournisseur, écriture manuelle, plusieurs factures, mauvais compte, ancien compte lié, lot de paiement pain.001) ?
   Un site ignoré est au moins **MEDIUM**.
6. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1689-1694, 2021, 2029, 2043-2060, 2079,
   2125-2150 ; ce que le manuel dit **aujourd'hui** est-il vrai au code de `fcd2ae75`, et ce que l'AC6 prescrit le
   sera-t-il au code prescrit (aucune écriture ; *Annuler le rapprochement* qui retire le règlement ; rejetée → *Annuler
   le rejet* d'abord) ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' '
   | tr -s ' '`. Les phrases « puis rejetez » que la 29-2 et la 29-6 écrivent (lis leurs fiches) sont-elles toutes
   visées par l'AC6 ? Même exigence pour `docs/api-external.md` (`:240`, `:571`, `:594`, tableau des erreurs) et le
   CHANGELOG prescrit. Une promesse que le code ne tiendrait pas est au moins **MEDIUM**.
7. **Zones partagées** : confronte le § *Zones partagées* aux fiches 29-2, 29-6, 29-1. La 29-8 casse-t-elle un test ou
   un texte prescrit par elles (AC10 de la 29-6 : case, *Accepter* grisé, indication ; K2 de la 29-1) ? L'indication
   de la 29-6 « ne peut qu'être rejetée » reste-t-elle vraie une fois le bouton *Rattacher* présent ?
8. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (10), tâches
   (T0-T8), tests (T1-T16), mutations (M1-M13), choix (16), modules (4), sites de l'AC7 (5), sites de l'inventaire (a)
   (5 + 1).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #670 et du § *29-8* de l'epic couverte par un AC testable, chaque AC par une tâche, chaque AC par
  un test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les
  registres que la route neuve doit rejoindre (`audit_route_registry.rs`, `bank_account_closed_inventory.rs`,
  `audit_labels.rs`, `audit_label_registry.rs`, `textes_coherents.rs` G9, `lint-i18n-ownership`, `rbac_e2e.rs`,
  `idor_multi_tenant_e2e.rs`) — la fiche les nomme-t-elle tous ? Les codes d'erreur existants réutilisés (`404
  RECONCILIATION_TRANSACTION_NOT_PENDING`, `409 OPTIMISTIC_LOCK_CONFLICT`, `412 BANK_ACCOUNT_NOT_CONFIGURED`) ont-ils
  bien ces statuts et ces formes ? La règle *Un appariement automatique propose, il ne crée jamais* est-elle tenue à
  l'écran (aucune présélection) comme à l'API ?
- **F — Faisabilité et exactitude de la conception (adversaire)** : axes 3 et 4 en profondeur ; la requête de l'AC1
  (une seule, rejouée par le `POST` ; somme par écriture sans produit ; `NOT EXISTS` ; tri par écart puis jours ;
  `LIMIT` après filtre ; types `DECIMAL(18,2)` contre `DECIMAL(19,4)` en MariaDB 10.11) ; la faisabilité des tests
  (décor par l'API : `POST /invoices/{id}/settlements` sur un compte bancaire choisi, import CAMT de fixture
  synthétique, clôture posée par SQL pour T13, course T16 à deux connexions) ; les mutations M1-M13 (chacune
  **tuable** par le test nommé ? M8 équivalente ? en manque-t-il une — compte du grand livre ancien contre actuel,
  `daysApart` mal signé, `FOR UPDATE` de l'étape 4.3) ; l'absence de migration (AC10) est-elle vraie — aucune
  contrainte, aucun index, aucun `CHECK` à changer ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p1-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p1-F.md` (lentille F) ;
dernier message : le chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris)/`switch`/`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`,
`node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit),
`docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Aucune donnée réelle de Guy ne se lit ni ne se
cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la
sortie standard.
