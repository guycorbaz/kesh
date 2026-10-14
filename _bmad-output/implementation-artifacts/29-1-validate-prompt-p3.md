# Prompt — validation P3 de la spec, Story 29-1 (préparer la clôture d'un exercice)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 : Opus ;
remédiations par Opus. P3 est confiée à **Sonnet** pour les deux lentilles — en contexte frais, sans rien reprendre
de la session qui a écrit la remédiation P2.*

Dépôt `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`. Fiche :
`_bmad-output/implementation-artifacts/29-1-preparer-cloture.md` (une fiche, **deux** stories : 29-1a contrôles
préalables, AC1-AC9 et AC3-bis ; 29-1b contre-passation des transitoires, AC10-AC18 ; **l'AC19 est retirée** et ne
garde que son titre en renvoi vers la Story 29-7). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 29 ; 23 à 26 sont des **décisions
de l'orchestrateur**, 27 à 29 des choix de la remédiation P2). Registre de sprint :
`_bmad-output/implementation-artifacts/sprint-status.yaml` (clés `29-1*` et `29-7-repetition-a-blanc`). Fichier
d'epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-1. Rapports de P2 :
`/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p2-R.md` et `-F.md` (et ceux de P1, `…-p1-R.md`, `…-p1-F.md`). Le
Change Log de la fiche résume la remédiation P2, finding par finding, et déclare le **signal D5**. Issue #232
(`gh api repos/guycorbaz/kesh/issues/232` et `…/232/comments`, lecture seule). Règles : `CLAUDE.md` (§ Review Iteration
Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le
recompte des comptes rendus, « recompter n'est pas réécrire l'historique » ; § Règle de splitting préventif et son
amendement D5 ; § Issue Tracking Rule) ; `docs/agents-orchestration.md`.

**Bornes du diff de la remédiation P2** : `git diff 46b6f794 63eb6b1b -- _bmad-output/` (fiche, registre, registre de
sprint, fichier d'epic). La fiche d'avant P2 : `git show 46b6f794:_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`.

## Axe obligatoire 1 — relire ce que la remédiation P2 a écrit

Le motif mesuré du dépôt, vérifié encore en P2 (13 findings R sur 16 nés de la remédiation P1) : *la sévérité se
déplace vers ce qu'on vient d'écrire*. Lis le diff ci-dessus **en entier**, et pour chaque passage neuf :

1. **Est-il vrai au code ?** Revérifie (`grep -nF`, `sed -n`) chaque ligne, fichier, fonction, constante, champ cité :
   `crates/kesh-api/src/routes/bank_accounts.rs` (`BankAccountWithBalance` l. 166-183, handler de liste l. 530-552,
   route d'archivage l. 912, `claims_for_target` l. 448-466 et le refus `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`) ;
   `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` l. 50-112 ; `frontend/src/lib/features/homepage/`
   (`homepage.ts` l. 50-72, `homepage.test.ts`, `BankAccountsCard.svelte` — reste-t-il vraiment inchangé si
   `statementGap` lit le champ ?) ; les fabriques de `BankAccountSummary` nommées à l'AC3 (`grep -rln
   "ledgerBalanceAtStatement" frontend/src`) ; `crates/kesh-db/src/repositories/bank_accounts.rs` l. 1338-1515
   (`BalancesRow`, `sharing`) ; `crates/kesh-db/src/repositories/fiscal_years.rs` l. 162-192, 313-440, 629, 789-799,
   868, 999, 1081-1115 ; `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` l. 118 ;
   `crates/kesh-db/src/test_fixtures.rs` l. 265-330 (préréglage) ; `crates/kesh-api/tests/audit_route_registry.rs`
   l. 27-32, 659-662, 682, 994, 1012 ; `crates/kesh-api/tests/bank_accounts_e2e.rs` (les tests nommés à l'AC3) ;
   `crates/kesh-api/tests/textes_coherents.rs` (gardes G4-bis, G7, G9 — lis leur **contenu**) ;
   `crates/kesh-core/assets/charts/*.json` (`1020 Banque`, `1300 Actifs transitoires`) ; `docs/api-external.md` § 10.
2. **A-t-elle introduit un défaut ?** Contradiction avec un passage resté de P1 ou d'avant ; règle énoncée deux fois
   de deux façons ; décompte faux — **recompte depuis la fiche et le registre** : 19 AC vivants (AC1 à AC18 et
   AC3-bis ; l'AC19 en renvoi), T-a1 à T-a16, T-b1 à T-b9, M-a1 à M-a21 (M-a9 équivalente), M-b1 à M-b13 (M-b13
   équivalente), C-29-1-1 à 29 ; mutation déclarée tuée qui ne le serait pas (M-a4 avec sa fixture SQL directe,
   M-a19, M-a20, M-a21, M-b3 par T-b3 seul), ou déclarée équivalente à tort (M-b13 : l'état « ordinaire ouvert,
   verrouillante clos » est-il vraiment inatteignable par **tout** chemin — `close`, `reopen`, `onboarding::reset`,
   l'import d'installation ?) ; test qui passerait à vide ; promesse que le code prescrit ne tiendrait pas.
3. **Les entrées passées sont-elles intactes ?** La remédiation P2 ne doit avoir réécrit **aucune** entrée datée de la
   création ni de la validation P1 (Change Log, Completion Notes, registre) : seules des annotations datées y sont
   permises. `git diff 46b6f794 63eb6b1b` le montre ligne à ligne.
4. **Chaque finding de P2 est-il réellement fermé ?** Relis les deux rapports P2 ligne à ligne contre la fiche
   actuelle, y compris les LOW (R9 à R16, F L1 à L9). Ceux que la remédiation déclare « sans objet par retrait de
   l'AC19 » (R1, R2, F5, F L9, la part G19 de R15) le sont-ils vraiment, et sont-ils versés au commentaire de la clé
   `29-7-repetition-a-blanc` ?

## Axe obligatoire 2 — le retrait de l'AC19 est-il complet ?

Décision de l'orchestrateur : l'AC19 (répétition à blanc au manuel d'administration) est **retirée, non réécrite**, et
devient la Story 29-7. Vérifie :

- qu'il ne reste **aucune prose de procédure** de la répétition dans la fiche (commandes `docker compose`, nom de
  projet, `container_name`, ports, image épinglée, `.env` de la copie, étapes d'import et de suppression) — seulement
  un renvoi : `grep -nE "docker compose|container_name|kesh-repetition|keshbackup|pre-import|KESH_SMTP" <fiche>` et
  trie chaque occurrence ;
- qu'aucune tâche (Ta*, Tb*), aucun test (T-a*, T-b*), aucune mutation, aucune ligne du tableau des modules, du
  périmètre ou de l'en-tête ne prescrit encore une modification de `docs/manual/fr/admin-manual.tex` :
  `grep -nE "admin-manual|d'administration|AC19" <fiche>` et trie (les renvois et les entrées datées sont voulus) ;
- que les deux autres livrables de l'ancienne AC19 — `api-external.md` (route de l'AC13, deux codes de l'AC14) et
  CHANGELOG de la 29-1b — sont **portés ailleurs** (AC17, C-29-1-28) et non perdus ;
- que le registre de sprint porte `29-7-repetition-a-blanc: backlog` avec un commentaire qui cite **R1, R2 et F5 de
  P2** comme cahier des charges, sans donnée réelle, et que les clés `29-1*` disent AC10-AC18 ; que le fichier d'epic
  et le registre (C-29-1-1, 13, 21) sont **annotés**, non réécrits ;
- que le retrait ne laisse aucune référence pendante (`sec:repetition-cloture`, « répétition à blanc » dans un AC
  vivant, C-29-1-21 cité comme choix actif).

## Axes prioritaires (communs aux deux lentilles)

5. **Périmètre minimal et modules** : l'écart servi au tableau de bord (C-29-1-25) est une décision de
   l'orchestrateur ; vérifie qu'il est **mécanique** comme annoncé (ajout d'un champ, lecture, suppression de
   `toCentime`) et que le décompte des modules — cinq au grain de la crate, dispersion au grain fin déclarée — est
   écrit sans contradiction.
6. **K3 refondu** (C-29-1-24) : le troisième critère d'activité (ligne d'écriture sur le compte lié, datée dans Y) est-il
   calculable par trois `EXISTS` séparés sous l'instantané ? La précédence couvre-t-elle les cas neufs : compte lié
   **partagé** avec écriture dans Y (→ `NOT_COMPARABLE` ?) ; compte **sans** relevé à solde avec écriture dans Y
   (→ `NO_BALANCE_STATEMENT`) ; compte archivé ou clos avant Y dont le compte lié porte des écritures dans Y (exclu ?
   la fixture de M-a4 le dit-elle ?). Une écriture de **contre-passation** ou d'ouverture sur le compte lié rend-elle un
   compte « actif » à tort ?
7. **T-a2 inversée** (C-29-1-26) : le détecteur décrit est-il **réalisable** et **sans angle mort** ? Un site
   `sqlx::query_as::<_, (i64, Option<String>)>(\n CONST_SQL)` est-il résolu ; `query_as!` (macro) ; un appel à travers
   un alias (`use sqlx::query_as as q`) ; une méthode `.fetch_*` sur une requête construite ailleurs ; une fonction
   externe hors de la liste close appelée par un chemin `use` importé (le volet (c) relève-t-il les noms importés par
   `use bank_accounts::{…}` ?) ; le cas de contrôle du détecteur prouve-t-il ce qu'il prétend ?
8. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Clôture d'un exercice* (l. 732),
   l. 728 « (si clôturé) », § *Arrêter les livres à une date* (l. 581-626), § *Corriger une écriture : la
   contre-passation* (l. 628), § *Exercices dans le désordre* (l. 760-766, dont l. 764 et le paragraphe « Des boutons
   offerts », l. 766), résumé l. 2992 ; `docs/manual/fr/marketing-brochure.tex` l. 418 ; `README.md` l. 47 ;
   `docs/api-external.md` § 10. Ce que les AC8 et AC17 prescrivent d'écrire — en particulier la phrase neuve de l'AC8
   sur l'activité (transaction, relevé ou écriture sur le compte lié) et le relevé « arrêté au dernier jour de
   l'exercice » — sera-t-il vrai au code prescrit ? Contrôle aussi les **PDF**, aplatis :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, de même pour `marketing-brochure.pdf` ;
   et `admin-manual.pdf` pour t'assurer qu'aucune phrase actuelle n'y annonce la répétition à blanc comme livrée.
   Une promesse que le code ne tiendrait pas est au moins MEDIUM.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC vivant couvert par une tâche et par un test **qui rougirait** ; chaque mutation (M-a1 à M-a21, M-b1 à
  M-b13) confrontée au test nommé, y compris les deux équivalences déclarées (M-a9 : quatre chemins de création ;
  M-b13 : état inatteignable) ; les textes prescrits (i18n ×4, manuel, brochure, README, CHANGELOG `## [0.14.0] — Non
  publié`, `api-external.md`) ; les gardes existantes nommées aux tâches — `audit_route_registry.rs` et ses cinq
  sites de compteurs, G4-bis / G7 / G9 de `textes_coherents.rs`, `audit_label_registry.rs`, `i18n-keys.test.ts`,
  `lint-i18n-ownership`, les deux tests de page à adapter, `homepage.test.ts` et les fabriques de
  `BankAccountSummary`, `bank_accounts_e2e.rs` — la fiche les nomme-t-elle toutes, et l'une rougira-t-elle sans y
  être nommée ? L'écran (AC5, AC6, AC15) et la carte du tableau de bord (AC3) : rôles, états vide / erreur /
  chargement, liens, clair et sombre. L'E2E de l'AC18 : les comptes créés à l'étape 0 (`1300`, `1020`, compte
  bancaire lié) le sont-ils par des routes qui existent, avec les rôles du préréglage, et l'étape 3 (rapprochement)
  rend-elle K3 `ok` avec 1020 comme compte lié ?
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — **axe B5 nommé** : *une
  modification qui touche une transaction se relit sur l'ordre des verrous **et** la naissance de la vue*.
  - 29-1a : `earlier_open_locking_sql()` laisse-t-elle **strictement** inchangés le texte exécuté par `close`, ses
    verrous et leur ordre, et les motifs des tests 15-8a / 15-12a ? Le troisième `EXISTS` de K3 et la lecture de Y par
    `FISCAL_YEAR_SQL` sous `ReadSnapshot` : aucune lecture verrouillante, une seule vue ? Le champ `statement_gap`
    du tableau de bord change-t-il quoi que ce soit au calcul ou à la requête de `list_by_company_with_balances` ?
  - 29-1b : la prescription de l'AC11 (statut jugé par la seule confirmation) et l'argument « état inatteignable » de
    l'AC12 et de M-b13 sont-ils **vrais** — relis `fiscal_years::close` (l. 1054, (b') l. 1081-1090), `reopen`
    (l. 1197), `create` (l. 313), `find_open_covering_date` (l. 735), `reverse_in_tx_inner` (l. 2595 et suiv.,
    étape (1) l. 2603-2607) ? Le cycle ajouté à l'AC12 (`create` d'un exercice postérieur, M < N) est-il exact ?
    T-b5 (ii) : la recopie des constantes privées est-elle montable, et l'ordre ne tue-t-il plus que M-b11 ? L'angle
    mort de l'intercalé (chemin pool contre `_in_tx`, Dev Notes) est-il juste ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il **naît de la remédiation P2**
(passage du diff `46b6f794..63eb6b1b`) ou s'il préexistait, et s'il **recycle** un finding de P1 ou de P2 sous une
autre forme (signal D5). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. ⛔ **La liste des
axes exercés ET non exercés** (axes obligatoires 1 et 2 et axes 5 à 8 compris) — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`, et là seulement ton rapport. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach`, `git worktree` ni aucune autre forme** ; aucune commande
qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/
`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et
tout autre interpréteur**, `docker`, `docker compose`, **`mariadb`, `mysql` et toute requête SQL**, `curl` vers une
instance, `bash` sur un script du dépôt, `sed -i`, `gh` en écriture (`issue`, `pr`, `comment`…). Aucune donnée réelle
de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`, aucun `.keshbackup`). Autorisés : lecture, `grep`, `sed -n`,
`awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
