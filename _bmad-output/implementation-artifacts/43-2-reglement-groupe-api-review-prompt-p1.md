# Prompt — revue de code P1 de la Story 43-2 (#823), trois lentilles, première passe complète

Tu relis la **Story 43-2** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB 10.11), release **0.19.0**,
Epic 43 (« une transaction, plusieurs factures ») : **l'API du règlement groupé** — la recherche des factures candidates
(`GET /api/v1/reconciliation/transactions/{id}/group-candidates`, dépôt neuf
`crates/kesh-db/src/repositories/reconciliation_group_candidates.rs`), le type `invoice_group` (et, dans un commit
**retirable**, `supplier_invoice_group`) de `POST /api/v1/reconciliation/accept`, la vue et l'annulation d'un groupe. Le
règlement lui-même, ses refus et son audit vivent dans le dépôt de la 43-1a (`grouped_settlements.rs`), dont le contrat
fait foi : `/home/gcorbaz/devel/kesh-gate-logs/e43/t3.md` (commit `241839568`). Règle de la différence :
`/home/gcorbaz/devel/kesh-gate-logs/e43/decision-difference.md`. **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-43-2`, branche `story/43-2-reglement-groupe-api`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-43-2 diff cf77cff18 <DERNIER_COMMIT_DE_CODE>` (`cf77cff18` = base : contrat de la 43-1a et
fiches à jour ; le Dev Agent Record de la fiche nomme le dernier commit de code et le commit D4). Le commit D4
(type fournisseur) se relit aussi **seul** (`git show <D4>`) : il doit être retirable sans rien casser. Langue du rapport :
français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/43-2-reglement-groupe-api.md` en entier (D1 à D9, Contrat, AC1 à AC10, Tests,
Dev Notes, **Change Log** et **Dev Agent Record**) ; le registre `e0190-choix-autonomes-e432.md` (C-e432-1 à -11) ; le
fichier d'épique `_bmad-output/planning-artifacts/epic-43-transaction-plusieurs-factures.md` (C-43-*). Ne prends pas le
Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e432/` (`rouges.log`, `cible-*.log`, `mut/campagne.py`, `mut/campagne.log`, les
journaux de gates nommés au Dev Agent Record).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**,
  nulle part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par
  erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (le banc de
  mutations se **lit**, il ne se lance pas) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`,
  **`claude`**, `sed -i`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`,
  `awk` (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et,
  pour un PDF, la seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ;
  tout chemin cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Pattern batch (D2, `CLAUDE.md` § *Pattern batch*)** : aucune erreur d'un groupe ne sort au lot. Tout `Err(DbError)`
   passe-t-il par `group_error_to_failed_proposal` (délégation à `lettering_error_to_failed_proposal`) ? Un interblocage
   absorbé en `DATABASE_ERROR` puis un `ROLLBACK TO SAVEPOINT` en 1305 remonte-t-il bien en `TransactionAborted`, donc en
   rejeu du lot (`accept_batch`) ? Le pré-vol (`preflight_group`) ne refuse-t-il au lot que ce que la fiche prescrit
   (identifiant `≤ 0`, `differenceAccountId ≤ 0`) ?
2. **La table des refus (M8)** : `group_refusal_to_failed_proposal`, `match` exhaustif sans joker — chaque variante de
   `GroupRefusal` (liste finale de `t3.md`) a-t-elle le code et les `details` de la table de la fiche ? Les constructeurs
   de l'unitaire sont-ils **appelés** (`refuse_wrong_direction`, `non_postable_failed_proposal`,
   `counterparty_is_bank_ledger_failed_proposal`, `period_locked_failed_proposal`, `later_fiscal_year_closed_failed_proposal`,
   `claim_account_refusal_details`, `payment_before_invoice_details`) ? Les écarts déclarés (C-e432-6, -7, -10) sont-ils
   justes ? Le discriminant des deux causes de `role: "counterparty"` (présence de `bankAccountId`) tient-il ?
3. **Montants (D5)** : brut à quatre décimales (`difference`, `roundingDifference`, `amountDueRaw`,
   `settledInvoices[].amount`, `groupedInvoices[].amount`), centime pour `gap`, `amountDue`, `totalTtc`,
   `transactionAmount`, `maxDifference` — par la seule fonction `fixed` ; zéro négatif ; arrondi demi loin de zéro.
   L'invariant `difference = roundingDifference + gap` ; le `gap` de la vue, recalculé par `group_difference` sur les
   montants du règlement (M5) et non sur les réglages.
4. **La recherche (AC5)** : la liste n'offre-t-elle jamais ce que le `POST` refuserait (statut, `paid_at`, écriture de
   vente, reste dû brut `> 0`, borne `latest_invoice_date`, lots `generated`) ? `docs/search-patterns.md` § 3.1 et § 6 :
   disjonction dans les deux branches, repli `LIKE` du nom à jetons courts, `escape_like`/`escape_boolean_ft` ;
   échantillon déterminé à 500 (`LIMIT 501`, C-e432-1), tri par `imputation_order` puis troncature, `hasMore` ; `exclude`
   avant le plafond ; société de la requête partout (KF-002) ; ordre des refus de `get_link_candidates`.
5. **Vue et annulation (AC7)** : `kind` ajouté à `ReconciliationCancellation` (C-e432-4, fichier de la 43-1a) — juste pour
   tous les genres, retrait seul compris ? `differenceAccount` lu dans la société de la requête ; `cancelBlockedInvoice` ;
   « un groupe n'est jamais un retrait seul ».
6. **Le commit D4** (type fournisseur) : un seul commit, retirable — sans lui, le code compile-t-il, un débit rend-il une
   liste vide (`family_of`), le message de l'extracteur nomme-t-il cinq types (test) ? Avec lui : six types, symétrie
   `SettlementDirection::Debit`, tests marqués « exige la 43-1b » (`pay_supplier_invoices_in_tx` est encore
   `unimplemented!` dans le dépôt : une requête `supplier_invoice_group` y paniquerait — le dire au rapport si la release
   peut partir avec D4 sans la 43-1b).
7. **Tests et mutations** : chaque AC a-t-il son cas, qui peut rougir ? Les tests passent-ils à vide (fixture qui rend la
   condition fausse) ? La campagne (a) à (h) (`mut/campagne.log`) : chaque mutation tuée par le test annoncé ; une
   survivante est-elle déclarée et expliquée ? Propose des mutations plausibles non jouées.
8. **Le guide d'API contre le code** : `docs/api-external.md`, § « Régler plusieurs factures par une transaction » (AC9) —
   chaque phrase est-elle vraie au code (ordre des refus, codes, champs, `exclude`/`limit`, vue, annulation) ? Les gardes
   `textes_coherents.rs` qui lisent ce fichier (G4-bis, G4-ter, G25) restent-elles vraies ? Aucun manuel LaTeX n'est
   touché par la story (l'écran est la 43-3) : le contester si une phrase de `docs/manual/fr/user-manual.tex` est devenue
   fausse ; un PDF ne se contrôle que par la forme permise.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `reconciliation_group.rs`, `reconciliation_group_candidates.rs`, les retouches de
`reconciliation.rs` (variante, pré-vol, extracteur, aiguillage, date du jour transmise, champs de la vue et de
l'annulation), `reconciliation_cancel.rs`, et les deux fichiers de tests. Cherche : une erreur qui sort au lot ; un bind
décalé dans le `QueryBuilder` ; une injection ; un champ sérialisé sous un mauvais nom ; un test qui passe à vide.

## Lentille E — Edge (chaque branche, chaque borne)

`q` vide, blanc, fait d'opérateurs, à jetons courts, mixte ; `exclude` vide, à 50 et 51 identifiants, avec doublons ;
`limit` 0, négatif, 100, 101, énorme ; 500 et 501 factures ; transaction nulle, EUR, rejetée, rapprochée, d'une autre
société ; groupe de 1, 2, 50, 51 factures, doublon, facture inexistante, d'une autre société ; `differenceAccountId`
présent alors que `e = 0` ; plafond 0.00 et 100 ; `r = ±0.005` ; compte choisi = banque non imputable, archivé, d'une autre
société, 2000, grand livre d'une autre banque ; deux lots concurrents ; une clé d'API en lecture. Chaque cas :
comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC10 : livré, testé, au lieu prescrit ? L'étape 0 (LOW de la passe ciblée 2, L-5, L6, L4 de la P5 ;
lignes lues citées au Change Log) ; les rouges (`rouges.log`) ; les mutations ; les gates **au dernier commit de code**
(chiffres au Dev Agent Record, chaque chiffre au journal nommé) ; l'E2E complet **non lancé** au passage en `review` (règle
`e35-dev-commun.md`) — il tourne à la clôture ; les écarts déclarés (C-e432-1 à 11) sont-ils exacts ? Le Dev Agent Record
n'affirme-t-il que ce qui a tourné ? Les tests fournisseur marqués « exige la 43-1b » sont-ils listés comme non exécutés ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e432/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM /
LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non
exercés** ; le décompte final **recompté depuis la liste**.
