# Prompt — validation P1 de la spec, Story 29-6 (la réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-6`, branche `story/29-6` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md` (commit `80c5a278`).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-6* et § *Ordre*. Issue #665 (l'objet) :
`gh api repos/guycorbaz/kesh/issues/665` ; voisines #548 (Story 28-7, une seule borne de dates — le modèle), #476
(Story 25-4-c3-b, `classify_payment` à double borne), #529 (29-2) et #527 (29-3), fiches **non mergées** à lire en
lecture seule : `/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` et
`/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`.
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-6.md` (C-29-6-1 à 9).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § *Un appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md` ; rétrospective
`epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 : verrous et naissance de la vue).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Les prémisses, établies au code*) : revérifie chacune au code de `fcd2ae75`, et la **précision**
   (tolérance symétrique ; seul le côté « le paiement dépasse » est refusé ; doc-comment du dépôt faux). Une prémisse
   mal établie est au moins **MEDIUM**.
2. **Le choix de la voie (C-29-6-2)** : les quatre raisons d'écarter la voie 2 sont-elles **vraies au code** (refus
   `ROUNDING_ACCOUNT_NOT_CONFIGURED`, partage de `classify_payment` avec `settle_invoice`, promesses écrites) ? La
   voie 1 ferme-t-elle #665 tel qu'il est écrit (« une seule borne pour la proposition et l'acceptation ») ? Le
   contournement écrit au manuel (C-29-6-7 : régler le reste dû, **rejeter** la transaction, traiter l'écart) est-il
   **praticable** au code (flux de rejet, règlement manuel, état de la transaction après) ? Une raison fausse, ou un
   contournement impraticable, est au moins **MEDIUM**.
3. **La démonstration** (§ *La règle retenue*) : `classify_payment(P, R) = Overpayment ⇔ R < P − 0,005` pour `P > 0`
   au centime — refais-la contre `classify_payment` et `Money::round_to_centimes` (`MidpointAwayFromZero`), bornes
   incluses, `R` négatif, `R` à plus de quatre décimales (type de l'expression SQL du reste dû), `P` à plus de deux
   décimales. Une erreur de borne est **HIGH** : elle referait proposer ce qui est refusé, ou cacherait ce qui est
   accepté (10.01 sur 10.0050).
4. **Une seule borne, deux lecteurs** : la borne liée au `HAVING` et `classify_payment` peuvent-elles diverger
   (précision du paramètre lié, comparaison `DECIMAL` en MariaDB, `LIMIT 50`, facture modifiée entre la proposition et
   l'acceptation) ? Le test de grille d'AC4 lie-t-il réellement les deux (M6) ? Une divergence possible non écrite est
   au moins **HIGH**.
5. **Inventaire clos des sites** (Dev Notes § *Inventaire (a)*, *(b)*, *(c)*) : refais les `grep` copiés et
   **élargis-les** — tout code qui charge des factures pour une transaction ou compare un paiement à un reste dû
   (`kesh-reconciliation`, `kesh-seed`, `test_endpoints.rs`, frontend, imports). Pour (c) : tout test, Rust ou
   Playwright, qui attend la **proposition** d'une facture dont le reste est **inférieur** au paiement de 0,0051 à
   0,05. Un site ou un test ignoré dont le sens change est au moins **HIGH**. Les refus restants de l'inventaire (b)
   sont-ils tous écrits, et chacun à juste titre « hors classe » ?
6. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 971, 1308, 2029, 2049, 2056, 2330, 2901 ;
   ce que le manuel dit **aujourd'hui** est-il vrai au code de `fcd2ae75`, et ce que l'AC7 prescrit d'écrire le sera-t-il
   au code prescrit (paiement partiel, demi-centime, client qui a trop payé) ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Même exigence pour `docs/api-external.md`
   (`:505`, `:551`) et le texte de CHANGELOG prescrit. Une promesse que le code ne tiendrait pas est au moins
   **MEDIUM**.
7. **Zones partagées** : confronte le § *Zones partagées* aux deux fiches voisines. La 29-6, développée après elles,
   casse-t-elle un de leurs tests ou de leurs textes (montants des cas de la 29-3 : 99.99 sur 100.00, 972,89 sur
   972,90 ; cas de la 29-2) ? Les phrases #665 que la 29-3 écrit sont-elles toutes inventoriées ?
8. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (9), tâches (T0-T6),
   mutations (M1-M7), choix (9), modules (3), cas d'AC6 (a-h), lignes d'exemple (10), sites d'appel (13), sorties
   d'inventaire copiées.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #665 et du § *29-6* de l'epic couverte par un AC testable, chaque AC par une tâche, chaque AC par
  un test **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les
  textes prescrits confrontés au code prescrit ; les gardes existantes qui pourraient changer de sens
  (`textes_coherents.rs`, `failed-proposal-label.test.ts`, `a_centime_payment_settles_a_half_centime_invoice_with_a_rounding_line`,
  `two_centimes_over_a_half_centime_invoice_is_an_overpayment`, `find_unpaid_invoices_filters_amount_within_tolerance`,
  `a_written_off_invoice_is_no_reconciliation_candidate`) — la fiche les nomme-t-elle correctement ? Le Pattern batch
  (`FailedProposal` par proposition, jamais d'`AppError` global) est-il intact ?
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : la démonstration (axe 3), la
  constante `HALF_CENTIME` (construction `const` d'un `Decimal` valide en Rust stable avec la version de
  `rust_decimal` du dépôt), le `HAVING amount_due BETWEEN ? AND ? + ?` en MariaDB 10.11 (types des paramètres liés,
  comparaison à l'alias `amount_due`), la faisabilité des tests (AC4 : coût de la grille ; AC5 : `insert_test_invoice`
  à quatre décimales ; AC6 : `seed_validated_invoice` à quatre décimales, `designate_rounding_account`, exercice
  couvrant la date, unicité des montants sous la tolérance, helper `accept_couples_296`), les mutations M1-M7
  (chacune **tuable** par les tests nommés ? M4 et M7 vraiment distinctes ? en manque-t-il une évidente, ex. bornes du
  `BETWEEN` interverties, `least_due_without_overpayment` appelée sur la tolérance), et **B5** : `get_proposals`
  est-il vraiment hors transaction, et l'acceptation vraiment intouchée ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-6-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris)/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`,
**tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`,
**`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
