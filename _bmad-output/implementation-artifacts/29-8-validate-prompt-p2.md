# Prompt — validation P2 de la spec, Story 29-8 (une transaction réglée depuis la fiche se rattache à ce règlement)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : passes
complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; **P2 : Opus** pour les deux
lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` ; registre
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-8.md` (C-29-8-1 à 24 ; 17 à 24 nés de la P1).
**La remédiation P1 à relire** est le commit `db448987` : `git show db448987` (le diff unique, aplati — à préférer à
la séquence des commits). Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p1-R.md`
et `…-p1-F.md` ; entrée « Validation P1 » du Change Log de la fiche.
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-8*, § *Ordre*. Issues (lecture) : #670
(l'objet), #671 (fournisseurs, écriture manuelle, paiement multiple, lots pain.001 ; commentaires de l'orchestrateur
sur L5, les lots et L6), #672 (détacher sans annuler), #401 (frais de rappel), #651 (clôture concurrente).
Fiches **non mergées**, en lecture seule : `/home/gcorbaz/devel/kesh-29-2/…/29-2-paiement-tardif.md`,
`/home/gcorbaz/devel/kesh-29-6/…/29-6-trop-percu-centimes.md`, `/home/gcorbaz/devel/kesh-29-1/…/29-1-preparer-cloture.md`
(K2), `/home/gcorbaz/devel/kesh-29-4/_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (AC5,
registre `FENETRE`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et toutes ses sous-sections — passe ciblée, manuel nommé, axes déclarés,
sites NON RÉSOLUS, propagation post-patch, recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (B5 : une simplification qui
touche une transaction se relit sur les verrous **et** la naissance de la vue ; § 4.3 : aucune dette sans issue).

## Axe 0 — obligatoire, pour les deux lentilles : relire la remédiation P1

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Pour **chacun** des 20 findings P1
(R1-R11, F1-F9), dis s'il est **réellement** fermé par `db448987`, et si le correctif a **introduit** un défaut :

- F1 : l'étape 4.3 retirée — l'argument « le verrou de ligne de `s` sérialise » est-il vrai au code
  (`mutex.rs:126-132`, `conclude_locked_attempt`, condition 2 qui force le même compte) ? Existe-t-il un **troisième**
  écrivain de `matched_entry_id` qui viserait l'écriture d'un règlement **existant** sans prendre `s` (grep
  `UPDATE bank_transactions` et `matched_entry_id` sur `crates`, hors tests) ? Un chemin d'**import** ou de
  **restauration** qui poserait la colonne ?
- La nouvelle étape 4.2 en deux instructions (`s` puis `je`, sans l'exercice) : un cycle naît-il avec un geste qui
  prend `je` puis `s` (`journal_entries::delete_in_tx`, `reverse_in_tx_inner`, modification d'écriture 15-8a,
  `lettering`) ? Le retrait de l'exercice du verrou laisse-t-il une fenêtre utile (C-29-8-21) ?
- `ensure_open_in_tx` à l'étape 4.3 : respecte-t-il la règle de placement (`bank_accounts.rs:782-794`) ? Le statut
  `Gardee` et la forme de raison prescrits sont-ils ceux qu'attend `bank_account_closed_inventory.rs` ? La
  coordination avec `FENETRE` (29-4) est-elle exacte au regard de l'AC5 de la 29-4 (ensemble `G`, statut `SansFenetre`
  de W30) ?
- `M > 0`, table dérivée et `HAVING` : la requête prescrite reste-t-elle **une** requête rejouée par le `POST`
  (C-29-8-9) ? `ABS(DATEDIFF(s.settled_on, ?))` avec la date liée : le tri tient-il, la date de paiement n'est-elle
  écrite qu'une fois ?
- R2 = F4 : les codes prescrits (`409 MATCHED_BANK_TRANSACTION` via `EntryNotReversable`, `409
  INVOICE_HAS_SETTLEMENTS`) sont-ils ceux que rend la route HTTP réelle de l'annulation depuis la fiche et de la
  dévalidation ? Le **témoin** (acompte non rattaché de la même facture) rend-il l'assertion causale — un second
  règlement sur la même facture change-t-il l'état d'une autre garde ?
- Les tests neufs T13 (b), T17, T18, T19, T20 et les mutations M14 à M19 : chaque mutation est-elle **tuée** par le
  test nommé (raisonne le cas exact) ? T17 peut-il observer le `POST` bloqué sur un verrou de **ligne** (et non sur
  `GET_LOCK`) ? Les équivalences déclarées (M8, garde `IS NULL`, `LIMIT`, `FOR UPDATE` de `je`) sont-elles honnêtes ?
- Les compteurs d'`audit_route_registry.rs` attendus (119/113/95/122/122) : recompte-les depuis le registre (routes
  **mutantes** seules ?) ; le point (vi) sept → huit est-il juste ?

## Axes communs

1. **Prémisses** (§ *Les prémisses de #670*) et faits ajoutés en P1 (lots pain.001 = L2 + L4 ; `settle_invoice` refuse
   `amount <= 0`, `invoice_settlements_write.rs:76`) : revérifiés au code. Une prémisse mal établie est au moins
   **MEDIUM**.
2. **Gardes en aval** : tout consommateur de `matched_entry_id` qui supposerait l'écriture créée par le rapprochement
   (date, journal, description, audit) ; le rattachement d'un règlement **d'exercice clos** (C-29-8-21) fausse-t-il
   un rapport, la correction de dates de `list_by_company_with_balances`, ou le contrôle K2 ? Un consommateur faussé
   est au moins **HIGH**.
3. **Les conditions** : faux rattachement impossible ou visible ? Cas d'une écriture de règlement à trois lignes
   (arrondi) ; d'un compte lié **changé deux fois** ; d'un acompte puis d'un solde sur la même facture (deux règlements
   candidats pour la même transaction) ; d'un règlement dont le montant porte plus de deux décimales significatives
   (sortie `rescale(2)` contre condition exacte). Un faux rattachement muet est **HIGH**.
4. **Verrous et naissance de la vue** (lentille F en profondeur) : remplis la **table des verrous** ci-dessous, ligne
   par ligne, **au code** (fichier:ligne), et dis pour chaque paire si un cycle peut se fermer. Une lecture
   ordinaire avant le dernier verrou, un verrou d'intervalle, un cycle : au moins **HIGH**.
5. **Inventaire clos** : AC7 (sept sites) et tableau du manuel de l'AC6 (dix lignes) — refais les greps prescrits et
   **élargis-les** ; cherche un site que ni l'un ni l'autre ne trie. Angles morts L1 à L6 : complets, chacun adossé à
   une issue existante ? Un site ignoré est au moins **MEDIUM**.
6. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — `:1198`, `:1305`, `:1308`, `:1431`,
   `:1689-1694`, `:1840`, `:2021`, `:2029`, `:2076`, `:2124-2150`, `:2911`, `:2940-2952` ; ce qu'il dit aujourd'hui est-il
   vrai au code, ce que l'AC6 prescrit le sera-t-il au code prescrit ? Contrôle le **PDF aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -oE` sur « ne la relie pas »,
   « tout seul. », « rejet » — la fiche affirme une occurrence de chacune des deux premières sur `fcd2ae75` :
   **vérifie**. Même exigence pour `docs/api-external.md` et le CHANGELOG prescrit. Une promesse que le code ne
   tiendrait pas est au moins **MEDIUM**.
7. **Zones partagées** : 29-2, 29-6, 29-1 (K2), **29-4** (`FENETRE`, compteurs) ; la puce d'ordre réécrite dans
   l'epic se lit-elle juste ?
8. **Recomptes** : 10 AC, 9 tâches, 20 tests (T1-T20), 19 mutations (M1-M19), 24 choix, 7 sites AC7, 10 lignes au
   tableau du manuel, « 17 findings distincts » du Change Log (20 moins trois doublons) — chacun **recompté** depuis
   la source.

## Table des verrous (lentille F — à remplir, pas à recopier)

| Geste | Fichier:ligne | Ordre des verrous au code | Première lecture ordinaire | Paire avec le rattachement (`GET_LOCK` → `t` → `s` → `je`, puis `ensure_open_in_tx`) : cycle ? |
|---|---|---|---|---|
| rattachement (prescrit) | fiche AC2 | | | — |
| `cancel_settlement_in_tx` | `invoice_settlements_write.rs` | | | |
| `reconciliation_cancel::cancel_in_tx` | `reconciliation_cancel.rs` | | | |
| `accept_one_invoice` / `settle_invoice` | `routes/reconciliation.rs`, `invoice_settlements_write.rs` | | | |
| `journal_entries::delete_in_tx` / `reverse_in_tx_inner` | `journal_entries.rs` | | | |
| modification d'écriture (15-8a) | `journal_entries.rs` | | | |
| `bank_accounts::close_in_tx` | `bank_accounts.rs` | | | |
| suppression d'import | `bank_imports.rs` | | | |
| `unreject_once`, `post_manual_once` | `routes/reconciliation.rs` | | | |
| `fiscal_years::close` | `fiscal_years.rs` | | | |

Dis aussi, explicitement : `SELECT GET_LOCK` fait-il naître la vue ? (établi par lecture, **non exécutable** ici : le
dire comme tel.) Le cycle préexistant `reconciliation_cancel` (`s` → facture) contre `cancel_settlement_in_tx`
(facture → `s`), nommé aux Dev Notes, est-il exact ?

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #670 et du § *29-8* de l'epic couverte par un AC testable, chaque AC par une tâche, chaque AC par
  un test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne cité ou ajouté en P1 **revérifié**
  (`sed -n`, `grep -nF`) ; les registres que la route neuve doit rejoindre (`audit_route_registry.rs`,
  `bank_account_closed_inventory.rs`, `audit_labels.rs`, `audit_label_registry.rs`, `textes_coherents.rs` G9,
  `lint-i18n-ownership`) ; les codes d'erreur réutilisés (statut et forme) ; la règle *propose, ne crée jamais* à
  l'écran comme à l'API (la recharge de la modal sur `409`, C-29-8-24, ne présélectionne-t-elle rien ?).
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe 0, axes 3 et 4 en profondeur, table des
  verrous ; la requête de l'AC1 (une seule, rejouée ; somme par écriture sans produit ; `NOT EXISTS` ; `HAVING M > 0` ;
  tri lié ; `LIMIT` après filtre ; `DECIMAL(18,2)` contre `DECIMAL(19,4)` en MariaDB 10.11, tolérance liée en
  `Decimal`) ; faisabilité des décors (T13 (b) et T17 : observer un `POST` bloqué ; T19 : exercice clos posé par
  SQL ; T20 : archivage posé par SQL) ; tuabilité de chaque mutation.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il est **né de la remédiation
P1** (`db448987`) ou d'origine. Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. Le **verdict
de l'axe 0** finding P1 par finding P1. La table des verrous remplie (lentille F). ⛔ **La liste des axes exercés ET
non exercés** — un « 0 finding » sans elle ne compte pas comme passe ; ce qui a été raisonné et non exécuté se dit
comme tel. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p2-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p2-F.md` (lentille F) ;
dernier message : le chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (ton seul rapport).
Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git
commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/`restore`/`stash`/`fetch`/`pull`/`reset`/
`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en
écriture (aucun commentaire d'issue). Aucune donnée réelle de Guy ne se lit ni ne se cite. Autorisés : lecture,
`grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
