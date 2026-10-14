# Prompt — validation P3 de la spec, Story 29-8 (une transaction réglée depuis la fiche se rattache à ce règlement)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : passes
complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 : Opus ; **P3 : Sonnet**
pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-8`, branche `story/29-8` (base `fcd2ae75`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-8-transaction-reglee-ailleurs.md` ; registre
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-8.md` (C-29-8-1 à 28 ; 25 à 28 nés de la P2, et des
annotations datées sur C-29-8-15, 17, 18).
**La remédiation P2 à relire** est le commit `0a832608` : `git show 0a832608` (diff unique). Rapports P2 :
`/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p2-R.md` et `…-p2-F.md` ; entrée
« Validation P2 » du Change Log de la fiche (signal D5 et décisions de l'orchestrateur).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-8*. Issues (lecture) : #670, #671, #672,
#401, #651. Fiches **non mergées**, en lecture seule : `/home/gcorbaz/devel/kesh-29-6/_bmad-output/implementation-artifacts/29-6-trop-percu-centimes.md`
(**telle qu'elle est**, `3495aca1` : AC7, `:270-345` ; CHANGELOG `:355-362` ; indication `:478`),
`/home/gcorbaz/devel/kesh-29-2/…/29-2-paiement-tardif.md`, `/home/gcorbaz/devel/kesh-29-1/…/29-1-preparer-cloture.md`
(K2), `/home/gcorbaz/devel/kesh-29-4/_bmad-output/implementation-artifacts/29-4-cloture-compte-concurrente.md` (AC5,
`FENETRE`, ensemble `G`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et toutes ses sous-sections ; § Règle de splitting préventif, amendement
D5 ; § *Un appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md` ;
`/home/gcorbaz/devel/kesh-gate-logs/prompts/epic29-fiche-commun.md` § *Piège connu* (pas d'`INNODB_TRX`).

## Axe 0 — obligatoire, pour les deux lentilles : relire la remédiation P2

Le motif mesuré : *la sévérité se déplace vers ce qu'on vient d'écrire* ; la P2 a trouvé quatre défauts nés de la P1.
La P2 a été remédiée par une règle explicite de l'orchestrateur, **« retirer plutôt qu'ajouter »**. Vérifie que :

- **les retraits sont complets** : plus **aucune** prescription d'observation de l'attente pour T16 et T17
  (`grep -nE "INNODB_TRX|LOCK WAIT|vu bloqué|observ|attendre_une_requete_en_cours|PROCESSLIST"` sur la fiche et le
  registre ; seule reste légitime la sonde `GET_LOCK` de T13 (b) et l'historique daté du Change Log P1) ; plus aucune
  affirmation que M2, M15 ou M17 est tuée par un test ; plus aucune clôture du compte dans T15 ; plus aucun « (#670) »
  prescrit au manuel ;
- **rien ne les remplace en douce** : aucune autre forme d'observation (sonde, horodatage, `sleep`, compteur de
  tentatives, `SHOW PROCESSLIST`) n'est réintroduite ailleurs pour T16, T17, M10 ou M14 ; aucun décor forgé en SQL
  pour M2, M15, M17 ; aucun test neuf pour L7 ;
- **ce qui est écrit « non prouvé » l'est honnêtement** : T16 et T17 tels que réécrits (courses `tokio::join!`)
  sont-ils exécutables et déterministes **dans leur assertion** (« exactement un réussit », état final cohérent) quel
  que soit l'entrelacement ? Énumère les entrelacements (annulation / rattachement ; deux rattachements sous le même
  verrou nommé, relâché avant le commit) et dis si un seul peut produire un autre résultat que ceux qu'accepte le test
  — un test qui rougirait au hasard est au moins **MEDIUM** ;
- **les équivalences déclarées sont vraies** au code : M2 (`chk_invoice_settlements_counterparty`,
  `20261001000004_invoice_settlements_write_off.sql:44-53`), M15 (jointure interne ; un état atteignable donne-t-il
  une somme ≤ 0 sur le compte lié ?), M17 (étape 4.2 (a), `invoice_settlements_write.rs:137-142`) ; M1 est-elle tuée
  par T6 avec le décor à compte du grand livre partagé ?
- **T15 réécrit** : l'ordre des six assertions passe-t-il au code (codes réels de la dévalidation, de l'annulation
  depuis la fiche, du témoin par virement sur le même compte, de `…/cancel` en dernier) ? **T1** peut-il asserter la
  clôture refusée puis réussie (compte non principal, seule transaction en attente, aucun lot ouvert) ?
- **les ajouts de la P2** (il y en a malgré la règle) : le `GET` avec le pré-vol du `POST` (C-29-8-27) ; le message
  du code neuf ; la garantie d'échelle (`routes/invoices.rs:1247`) ; l'appel qualifié ; les compteurs en prose
  (`audit_route_registry.rs:27`, `:31`, `:32`, `:659`, `:1007`) et `sites.len()` 31 → 32 ; L7 ; l'entrée de
  CHANGELOG de la 29-6 réécrite — chacun exact, et sans défaut neuf ?
- pour **chacun** des 14 findings distincts de la P2 (R1-R10, F1-F11), dis s'il est **réellement** fermé par
  `0a832608` et si le correctif a introduit un défaut.

## Axes communs

1. **Prémisses** : celles de la fiche, revérifiées par sondage au code (au moins cinq citations `fichier:ligne` de la
   remédiation P2, `sed -n`).
2. **Gardes en aval** : la tuile d'écart (`list_by_company_with_balances`, `bank_accounts.rs:1394-1440`) avec une
   transaction rattachée dont l'écriture porte la date du règlement — ce que prescrit le manuel (`:259-267`) est-il
   vrai au code ?
3. **Conditions** : faux rattachement muet possible ? (un faux rattachement muet est **HIGH**).
4. **Verrous** (lentille F) : la table *Absence de cycle* des Dev Notes, ligne `reconciliation_cancel` réécrite en P2 —
   exacte au code ?
5. **Inventaire clos** : AC7 (**neuf** sites) et tableau du manuel de l'AC6 (**treize** lignes) — refais les greps
   prescrits, **élargis-les**, cherche un site que ni l'un ni l'autre ne trie ; angles morts L1 à L7, chacun adossé à
   une issue.
6. **Manuel contre code et fiche 29-6 (obligatoire)** : `docs/manual/fr/user-manual.tex` aux sites du tableau de l'AC6
   (`:259-267`, `:1198`, `:1305`, `:1308`, `:1689-1694`, `:2021`, `:2029`, `:2050`, `:2057`, `:2076`, `:2911`,
   `:3053`) — chaque citation de la 29-6 dans la fiche 29-8 est-elle **mot pour mot** celle de la fiche 29-6 actuelle ?
   Ce que l'AC6 prescrit sera-t-il vrai au code prescrit ? Contrôle le **PDF aplati et normalisé** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g"`,
   puis `grep -oF` sur chacune des huit chaînes du relevé de l'AC6 — la fiche affirme 1, 1, 1, 1, 1, 1, 0, 0 sur
   `fcd2ae75` : **vérifie**. Les contrôles positifs prescrits sont-ils des portions courtes qui survivront aux
   césures ? Même exigence pour `docs/api-external.md` (`:571`, `:594`, `:719`, `:730`, `:240`). Une promesse que le
   code ne tiendrait pas est au moins **MEDIUM**.
7. **Zones partagées** : 29-6 (textes et indication), 29-4 (`G`, appel qualifié), 29-1 (K2 dans T15).
8. **Recomptes** : 10 AC, 9 tâches, 20 tests, 19 mutations (dont trois équivalentes), 28 choix, 9 sites AC7,
   13 lignes au tableau du manuel, « 21 findings rapportés, 14 distincts » du Change Log P2 — chacun **recompté**.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque fait, chemin et numéro de ligne ajouté en P2 **revérifié** (`sed -n`, `grep -nF`) ; chaque AC couvert par un
  test qui rougirait au sol ; registres (`audit_route_registry.rs`, `bank_account_closed_inventory.rs`,
  `audit_labels.rs`, `textes_coherents.rs` G9) ; axes 5, 6, 7, 8 en profondeur.
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe 0 en profondeur (entrelacements de T16 et
  T17, équivalences, ordre de T15, T1) ; axes 2, 3, 4 ; requête de l'AC1 et pré-vol du `GET`.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée — en préférant, comme la remédiation P2, **retirer** à
ajouter ; pour chaque finding, dis s'il est **né de la remédiation P2** (`0a832608`) ou d'origine. Toute affirmation de
présence ou d'absence = sortie de `grep -nF` copiée. Le **verdict de l'axe 0**, finding P2 par finding P2. ⛔ **La liste
des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe ; ce qui a été raisonné et non
exécuté se dit comme tel. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p3-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/29-8-transaction-reglee-ailleurs-validate-p3-F.md` (lentille F) ;
dernier message : le chemin du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/` (ton seul rapport).
Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous
aucune forme** (`--detach` compris)/`switch`/`restore`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`,
`cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script,
`awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture (aucun commentaire
d'issue). Aucune donnée réelle de Guy ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git
log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
