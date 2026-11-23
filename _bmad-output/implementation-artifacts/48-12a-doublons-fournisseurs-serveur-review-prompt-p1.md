# Prompt — revue de code P1 de la Story 48-12a (#975), trois lentilles Opus, première passe complète

Tu relis la **Story 48-12a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5, MariaDB), release **0.20.0**, Epic 48 :
**le refus serveur des factures fournisseur en double**. Le serveur refuse désormais d'enregistrer deux fois la même facture
fournisseur : même fournisseur et même numéro du fournisseur (normalisé : blancs retirés, casse ignorée) **dans la même année
civile** de la date de facture → `409 SUPPLIER_INVOICE_DUPLICATE`, sans sortie à la saisie ; même date et même total TTC quand
l'une au moins des deux n'a pas de numéro → `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, levé par `confirmNotDuplicate: true` et
inscrit à l'audit (`supplier_invoice.duplicate_confirmed`) ; les annulées ne comptent pas ; la règle vaut à la création, au `PUT`
(seulement si fournisseur, numéro, date ou total change), à la complétion d'un document, et au **repassage en brouillon d'une
annulée** (les deux règles s'y lèvent par confirmation). Un verrou du fournisseur (`contacts FOR UPDATE`) est pris **avant toute
lecture ordinaire** sur les quatre chemins ; `POST /invoices` et `PUT /invoices/{id}` rejouent l'interblocage que ce verrou peut
former (D3ter). **Aucune migration.** Guy tient ses livres réels dans Kesh, avec de vraies factures fournisseur et des lots
pain.001 : **aucune facture existante ne doit être refusée à la relecture, aucune donnée ne doit bouger, aucune facture client ni
avoir ni PDF ne doit changer.** La 48-12b (écran) dépend de cette fiche et sort avec elle ; elle est sur la même branche, après.

Worktree `/home/gcorbaz/devel/kesh-48-12`, branche `story/48-12-reponses-perdues-fournisseurs`, base `a8ffbb488` (fusion de
`epic/48-corrections-0-20-0`). Le diff de la 48-12a, **aplati** : `git -C /home/gcorbaz/devel/kesh-48-12 diff a8ffbb488 <dernier
commit de code de la 48-12a>` — les commits sont listés au Dev Agent Record de la fiche (§ *Commits*) ; les commits de la 48-12b qui
suivent sont **hors périmètre** de cette revue, sauf l'axe 4 (textes partagés). Langue du rapport : français. Modèle : **Opus**,
contexte frais, une lentille par agent.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md` en entier — « Ce qui se lit au code » 1 à 10,
**D1 à D7** (D3bis relevé A, D3ter, D6bis), AC-1 à AC-14, tests et mutations, **relevé B**, Note de version, Dev Notes, **Change
Log** et **Dev Agent Record** — et la fiche de la 48-12b pour les textes partagés (D7 de la 48-12a, « Textes : un seul par refus »
de la 48-12b). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule)
sous `/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/` — journaux de gate nommés au Dev Agent Record, `controle-pdf-12a.log`,
`make-manuels-12a.log`, `sans-ecriture*.txt`, et `mut-12a/*.log` (script `mut-12a/muter.py` : lis-le, ne le lance pas).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**,
  il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (`muter.py`,
  `reset-bases.sh` compris) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `tr`, et
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (seule forme du contrôle de PDF).
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **AC et décisions de Guy** (D1 à D7, D6bis, D3ter ; décisions datées du 2026-11-19) : année civile et jamais l'exercice ;
   règle 1 sans sortie à la saisie, levée à la dévalidation seulement ; règle 2 levée par confirmation et auditée dans la même
   transaction ; règle 3 ; annulées exclues ; D6 (les quatre éléments, `id <> ?`) ; `details.confirmable` ; acteur de l'audit
   (`user` à la saisie, `par(acteur)` à la dévalidation) ; précédence de la dévalidation (après l'exercice clos, avant
   `delete_in_tx`) ; ordre des refus de saisie inchangé ; `lock_supplier_in_tx` qui rend `false` sans erreur. Code :
   `crates/kesh-core/src/payment_text.rs`, `crates/kesh-db/src/repositories/supplier_invoices.rs` (section « Story 48-12a »,
   `create_draft_in_tx_with_policy`, `lock_draft_and_supplier_in_tx`, `update_draft_in_tx_with_policy`,
   `unvalidate_in_tx_with_policy`), `crates/kesh-db/src/errors.rs`, `crates/kesh-api/src/errors.rs`
   (`supplier_invoice_duplicate_response`), `crates/kesh-api/src/routes/{supplier_invoices,imported_supplier_invoices,invoices}.rs`.
2. **Relevés A à D refaits par le symptôme** (« Inventorier les sites NON RÉSOLUS » du `CLAUDE.md`) :
   - **A** (D3bis) — rejoue les deux commandes de la fiche (`grep -niE "references +contacts *\(" crates/kesh-db/migrations/*.sql`
     et le `git grep` des écritures sur `invoices|credit_notes|supplier_invoices|contact_persons|contacts`) et cherche **tout chemin
     neuf** qui tient en exclusif une ressource que la story demande après le contact, puis demande un contact — la ligne 7 (avoir
     ↔ dévalidation par un autre exercice) est-elle bien couverte par le rejeu des deux routes ? un chemin non rejoué reste-t-il ?
   - **B** (relevé B) — rejoue la commande gawk de la fiche (`LC_ALL=C.UTF-8 gawk …` sur `docs/manual/fr/*.tex docs/*.md`) et vérifie
     chaque rang « oui — 48-12a » au `.tex` et au `.md` livrés, plus `user-manual.tex` (rôle du numéro du fournisseur) et `README.md`
     (feuille « Fonctionnalités ») ; cherche un paragraphe resté faux que le classement aurait manqué.
   - **C** et **D** (relevés de la 48-12b, écran) : **hors périmètre** de la 48-12a ; vérifie seulement que rien dans le diff de la
     48-12a ne les invalide (forme du contrat `details`, codes).
   - **Les sites qui lisent avant le verrou** : par le symptôme, **toute lecture ordinaire** (`SELECT` sans `FOR UPDATE` /
     `LOCK IN SHARE MODE`, appel d'une fonction de lecture du dépôt) qui précède `lock_supplier_in_tx` dans la transaction de
     chacun des quatre chemins — route comprise (`update_supplier_invoice_once`, `complete_import_once`), `get_company_for` (sur le
     pool, hors transaction ?) compris.
3. **Rejeu de `POST` et `PUT /invoices`** (D3ter, AC-14) : enveloppes `retry_on_deadlock("invoices::create"|"invoices::update")`,
   clonage, aucune écriture hors de la transaction de `invoices::create`/`update` (lis leur corps) ; tests
   `invoice_create_is_replayed_when_it_is_the_deadlock_victim` et `invoice_update_is_replayed_when_it_is_the_deadlock_victim`
   (`crates/kesh-api/tests/rejeu_interblocage_e2e.rs`) : le cycle monté est-il réel (le projet neuf **et** le contact neuf au `PUT`,
   V5-7) ? la transaction lourde obtient-elle bien son verrou (`.expect`) ? `capture.exiger_un_rejeu` ; la prose et le compte de
   `crates/kesh-api/tests/audit_route_registry.rs` (point (vi) : « trente-quatre », recompté) ; `docs/api-external.md` (paragraphe
   des interblocages) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (Pattern 5, `:306` et `:342`). **Aucune facture client, aucun avoir
   ni leur PDF ne change** : seule la réponse à un interblocage change — vérifie-le au diff.
4. **Textes partagés entre les deux fiches** : les trois clés `error-supplier-invoice-duplicate`, `-probable-duplicate`,
   `-duplicate-unvalidate`, aux quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`), égales aux replis Rust
   (`SUPPLIER_INVOICE_*_FR`, `crates/kesh-api/src/errors.rs`) et aux textes de D7 de la fiche (ordre « si c'est la même facture »
   puis « 15-b » ; aucun « 15-b » ni promesse de suppression à la dévalidation) ; l'écran (48-12b) lit ces clés et n'en a pas de
   copie ; justesse des traductions DE, EN, IT ; le libellé d'audit `audit-log-action-supplier-invoice-duplicate-confirmed` et
   `crates/kesh-api/src/audit_labels.rs`.
5. **Mutations** : `mut-12a/*.log` et `muter.py` — chaque mutation de la fiche jouée ? tuée par le test qu'elle vise ? une mutation
   déclarée équivalente ou non jouée l'est-elle à raison ? Restauration (`git restore` puis `touch`) à chaque fois ? Propose des
   mutations plausibles non jouées (ex. : `<>` → `<` sur le statut, `min_by_key` → `max_by_key`, `Decimal` comparé en chaîne).
6. **Manuels** : `docs/manual/fr/user-manual.tex` (« Les doublons. » `sec:doublons-fournisseur`, *Modifier* et *Supprimer le
   brouillon*, *Repasser une facture en brouillon* et sa liste de refus, annulation, document détaché, liste récapitulative),
   `docs/manual/fr/admin-manual.tex` (clés `read-write`, journal d'audit, dévalidation, « Passer à la 0.20.0 — Pour les
   intégrations »), `docs/api-external.md` — chaque phrase vraie au code ? Contrôle **négatif** (phrases périmées absentes) et
   **positif** (« 15-b » présent), sur le `.tex` aplati **et** sur le PDF aplati, **commande de la fiche rejouée** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'une seconde fois, et la première'`
   (et de même pour « ne signale pas la première facture », « sous les mêmes refus, plus le paiement détaché », « le lot de paiement
   en cours, le paiement détaché non contre-passé » sur `admin-manual.pdf`, « 15-b » des deux). ⚠️ Les PDF du dépôt ne sont **pas**
   régénérés par la story (jamais commités) : le PDF versionné porte encore l'état d'avant ; le contrôle sur le PDF régénéré a été
   fait au développement (`controle-pdf-12a.log`) — vérifie la cohérence entre ce journal et le `.tex`, et nomme le PDF versionné
   comme point d'intégration, non comme défaut. Le `.tex` aplati : `tr '\n' ' ' < docs/manual/fr/user-manual.tex | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
7. **Factures fournisseur réelles existantes** : aucune n'est refusée **à la relecture** (aucune lecture, liste, fiche, validation,
   paiement, lot, annulation ne passe par la règle — vérifie au code que `validate_in_tx`, `pay_in_tx`, `cancel_in_tx`, les lots et
   la réconciliation n'appellent ni `guard_duplicates_in_tx` ni `lock_supplier_in_tx`) ; un doublon d'avant la 0.20.0 se modifie
   sans changer les quatre éléments ; aucune migration (`git diff --stat` ne nomme pas `crates/kesh-db/migrations/`) ; seeds et
   fixtures corrigés plutôt que la règle (relevé des fixtures, journal de gate).
8. **Captures** : la 48-12a n'a pas d'écran ; vérifie qu'aucune capture n'était due (aucun contrôle visible changé par le diff de
   la 48-12a) — les captures du refus sont à la 48-12b.

## Lentille B — Blind Hunter (le diff seul, sans la fiche)

Relis ligne à ligne `payment_text.rs`, la section neuve du dépôt et les quatre fonctions modifiées, les erreurs, les routes, les
catalogues et les tests neufs (`crates/kesh-db/tests/supplier_invoice_duplicate.rs`, `supplier_invoice_duplicate_lock.rs`,
`crates/kesh-api/tests/supplier_invoice_duplicate_e2e.rs`, le bloc neuf de `rejeu_interblocage_e2e.rs`). Cherche : une lecture
ordinaire avant le verrou ; une comparaison décimale ou de date fausse ; un refus qui laisse une écriture ; un audit écrit hors de la
transaction ou au mauvais acteur ; un test qui passerait **à vide** (attente prouvée ? `assert!` sur la valeur de
`attendre_une_requete_en_cours` ?) ; une incohérence entre replis Rust et catalogues.

## Lentille E — Edge Case Hunter (chaque branche, chaque borne)

Numéros : `" "`, `"\u{200B}"`, `"15"` contre `"15 "`, `"İ"` (minuscule Unicode qui allonge), `"ß"`, numéros de 255 caractères ;
dates : 31.12/01.01, 29.02, années différentes du même exercice ; totaux : `120.5` contre `120.50`, totaux négatifs ou nuls ; une
facture existante `paid`, `open`, `draft`, `cancelled` ; plusieurs candidates des deux règles à la fois (règle 1 d'abord ? plus
petit identifiant ?) ; `PUT` qui change de fournisseur vers un fournisseur absent ; `PUT` d'une facture d'une autre société ;
dévalidation d'une annulée dont la facture active est elle-même la facture (exclue ?) ; `confirmNotDuplicate` absent, `null`,
`"true"` (chaîne), `1` — que fait serde ? ; corps de validation portant `confirmNotDuplicate` (sans effet ?) ; rejeu de la route de
création après un refus de doublon (pas de rejeu sur un 409 ?) ; clé `read` sur les routes d'écriture. Chaque cas : comportement au
code, test qui le couvre ou absence prouvée par `grep -nF`.

## Lentille A — Acceptance Auditor (la fiche contre le livré)

Chaque critère AC-1 à AC-14 : livré, testé, au lieu prescrit ? Chaque ligne du tableau « Tests à écrire » : le test existe-t-il au
binaire nommé, et sa mutation a-t-elle été jouée ? L'étape 0 (citations relues au code de la base) ; les écarts déclarés au Change Log
(nom du dossier des journaux, scripts plutôt que l'outil Edit pour les documents, tests adaptés) sont-ils exacts et sans effet sur
le livré ? Les gates (`test-fast`, `gate-ci`, frontend) : chaque chiffre se retrouve-t-il au journal, au commit annoncé ? Ce qui
n'a pas tourné (E2E de la 48-12a seule : aucun, l'E2E se joue à la fin de la 48-12b) est-il dit comme tel ? Le statut `review`
au `sprint-status.yaml` et à la fiche ; la Note de version (« changement de contrat ») ; le module compté (cinq au plus).

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12a-p1-B.md`, `…/review-12a-p1-E.md`,
`…/review-12a-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW),
lieu (`fichier:ligne`), ligne lue copiée, preuve, remède proposé ; **la liste des axes exercés ET non exercés** (les huit axes
ci-dessus, un par un) ; le décompte final **recompté depuis la liste**. Un « 0 finding » non adossé à cette liste ne compte pas
comme passe.
