# Prompt — validation P4 de la spec, Story 28-4 (clore un compte bancaire, en lecture seule)

*Versionné le 2026-10-13. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : Sonnet ; P2 :
Opus ; P3 : Sonnet ; **P4 : Opus** pour les deux lentilles. La remédiation de la P3 a été écrite par Opus ; elle ne
change pas la conception (la garde reste non verrouillante, C-28-4-21) mais ajoute une **règle de placement** et un
**second volet** au critère de l'AC7 : ce n'est pas une passe ciblée, c'est toute la fiche que tu relis, en commençant
par ce qui vient d'être écrit.*

Dépôt `/home/gcorbaz/devel/kesh-28-4`, branche `story/28-4` (base `5aad8b35`, kickoff de l'Epic 28 ; le code de la
branche est celui d'`origin/main` — vérifie-le : `git diff --stat origin/main HEAD -- crates frontend` doit être
vide). Fiche : `_bmad-output/implementation-artifacts/28-4-cloture-compte-bancaire.md` (une fiche, **deux** stories :
28-4a serveur, AC1-AC10 et AC6-bis ; 28-4b écran, AC11-AC17). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md`
§ 28-4. Issue #642 (`gh api repos/guycorbaz/kesh/issues/642` et `…/642/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md`, section *Story 28-4* (C-28-4-1 à C-28-4-23 ;
**C-28-4-23 neuf** ; **C-28-4-21 annoté en P3** — règle de placement, critère à deux volets, M36-M39 rétablies, L1
reclassée P2 ; C-28-4-14 et C-28-4-15 annotés). Story voisine non mergée (W30, R8) : `story/28-5`, fiche
`28-5-transaction-rejetee-atteignable.md` (`git show story/28-5:_bmad-output/implementation-artifacts/28-5-transaction-rejetee-atteignable.md`
si la branche est visible ; sinon, le dire).

**La remédiation à relire** : `git show 61b09ebc` (fiche + registre), soit `git diff 3823569b 61b09ebc`. Les rapports
de la P3 (hors dépôt, à lire pour savoir ce qui a été demandé) : `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p3-R.md`
et `…-p3-F.md`. L'entrée P3 du Change Log de la fiche dit, finding par finding, ce qui a été corrigé, et nomme les
écarts de la remédiation à la consigne.

**La décision qui gouverne cette passe** (C-28-4-21 et son annotation P3, AC7) : la garde « compte clos » ne prend
**aucun** verrou ; elle se pose **après le dernier verrou que le chemin prend avant sa première lecture ordinaire**, de
sorte que la story ne déplace jamais la naissance de la vue cohérente ; L1 est un angle mort assumé, classé **P2**,
dette motivée. Le critère décidable de l'AC7 a **deux volets** : (1) *aucun verrou de ligne ajouté, explicite ou
implicite par clé étrangère, à aucun chemin de `origin/main`, et aucun ordre de verrous changé* ; (2) *aucun
déplacement de la naissance de la vue : la première lecture non verrouillante de chaque chemin reste celle du sol — ou
la garde, posée immédiatement avant elle, sans aucun verrou entre les deux ; sur un chemin qui n'en a aucune au sol,
la garde suit le dernier verrou qu'il prend avant sa première écriture*. Ta passe ne rediscute pas la conception —
elle vérifie que les deux volets sont **tenus partout** et que la fiche **dit vrai** ; si tu trouves que la conception
laisse #642 non fermée, c'est un finding, avec sa preuve.

Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus, *recompter n'est pas réécrire l'historique* ;
§ Migration breaking policy P1-P8 ; § Pattern batch ; § Priorités des défauts ; § Règle de splitting préventif et son
amendement D5 ; § Issue Tracking Rule) ; `docs/agents-orchestration.md` ; la mémoire *invariant dans le temps*
(L1 dit que I1 et I2 peuvent être défaits : tout raisonnement de la fiche qui les suppose inviolables est à relire).

## Axe obligatoire — le DOUBLE critère, chemin par chemin, sur le code d'`origin/main` (les deux lentilles)

Pour **chaque** ligne de la table *Position par chemin* (Dev Notes § *Concurrence*) **et** pour tout chemin d'écriture
de l'inventaire (W1-W30, R1-R8) qui n'y figurerait pas :

1. écris, **depuis le code au sol** (`sed -n` des lignes), la suite de ses verrous dans l'ordre — y compris les
   **implicites** (verrou partagé sur le parent posé par une clé étrangère à l'`INSERT` d'un enfant ou à l'`UPDATE`
   d'une colonne à clé étrangère ; verrous de trou ; verrous des fonctions appelées : `journal_entries::create_in_tx`,
   `reverse_owned_in_tx`, `reverse_in_tx`, `letterings::dissolve_*`, `refuse_if_claim_account`,
   `company_invoice_settings::*`, `flip_primary_off_for_company`, `fiscal_years::find_open_covering_date`, l'audit) ;
2. écris, **depuis le code au sol**, la **première lecture non verrouillante** de la transaction — en remontant
   **jusqu'à la route** et à toute fonction qui compose (`_in_tx` appelée par un autre geste : W25 depuis W9, et tout
   autre appelant que `grep -rn "<fonction>(" crates/*/src` révèle) ; un `SELECT` sans table (`GET_LOCK` de
   `with_account_lock`) n'ouvre pas de vue InnoDB : vérifie-le, ne le suppose pas ;
3. écris ce que la fiche fait ajouter au chemin, **et où** (instruction élargie de colonnes, lecture non verrouillante
   neuve — à quelle ligne exacte, entre quels verrous —, lecture existante élargie, rien) ;
4. réponds, avec preuve : **un verrou est-il ajouté ?** **l'ordre change-t-il ?** **la vue naît-elle plus tôt qu'au
   sol ?** — c'est-à-dire : la garde précède-t-elle un verrou qui, au sol, précède la première lecture non
   verrouillante ? ;
5. compare ta table à celle de la fiche, ligne à ligne, colonne « 1re lecture non verrouillante : au sol → avec la
   story » comprise, et signale tout écart (ligne fausse, numéro de ligne faux, chemin manquant, verrou implicite omis,
   lecture ordinaire du sol omise).

Rends le résultat sous forme de table (chemin / suite des verrous au sol / 1re lecture non verrouillante au sol / ajout
de la story et sa position / verrou ajouté ? / ordre changé ? / vue déplacée ? / écart avec la fiche). **Un verrou
ajouté, un ordre changé, ou une vue avancée devant un verrou que la fiche ne déclare pas est au moins HIGH** — une vue
avancée devant un verrou d'exercice est une erreur comptable possible (P1).

Points à examiner en particulier :

- **W9, W25, W28, W29** : les positions neuves (W9 entre (3) et (4) ; W25 entre (2-bis) et (3) ; W28 après
  `supplier_invoices.rs:1070-1080`, avant (2) ; W29 après (1-bis), avant (2)) sont-elles bien **après tous** les
  verrous qui, au sol, précèdent la première lecture non verrouillante ? Y a-t-il, **dans** les motifs de l'étape
  suivante ou dans une fonction qu'elle appelle, un verrou pris **avant** la première lecture ordinaire du sol, qui
  ferait de la position prescrite une position encore trop tôt ? Pour W29 et W28, que se passe-t-il quand la facture
  n'est pas payée (la garde ne passe pas) ou payée en espèces ?
- **W9 → W25 imbriqué** (étape (6)) : la garde de W25 lit la vue née en (4) de W9 — est-ce juste, et cela change-t-il
  quoi que ce soit au critère ?
- **W24** : garde après l'étape 2, avant le comptage de l'étape 3 ; l'ordre des refus (clos avant `HasReconciled`)
  est-il écrit et voulu ? **M54** déclarée équivalente sous MariaDB 10.11 : la raison est-elle vraie (lis
  `bank_imports.rs:389-425` et la version de MariaDB de `docker-compose*.yml` et `.github/workflows/*.yml`) ?
- **W22** : « n'importe où dans le dépôt » — la vue est-elle vraiment née avant dans la route, **pour les deux appels**
  (`routes/bank_imports.rs:1173` → `:1312` CAMT ; `:1978` → `:2199` CSV), dans la même transaction ?
- **W30 / `unreject`** (28-5) : la position « après le verrou de sa ligne » tient-elle au texte de la fiche 28-5 ?
- **Les chemins à instruction élargie** (W8, W27, W13-W15, W17, W20) : la fiche dit « vue inchangée » ; W8 et W27 ont,
  au sol, une lecture ordinaire **avant** le verrou du compte et de l'exercice (étape (2)) — la fiche la nomme-t-elle
  juste, et la story n'y ajoute-t-elle bien rien ?
- **Les deux chemins neufs** : la vue de la clôture naît-elle bien après son verrou (condition de T-a31) ?

## Axe premier — les régressions de la remédiation P3

*La sévérité se déplace vers ce qu'on vient d'écrire.* Relis **chaque** changement de `61b09ebc` :

- **La règle de placement et le second volet** (AC6, AC7, inventaire, table, T4, T6, C-28-4-21 annoté) : sont-ils
  formulés de façon **décidable** et **cohérente** d'un lieu à l'autre ? Le second volet couvre-t-il tous les cas
  (garde juste avant la première lecture du sol ; chemin sans lecture ordinaire au sol ; chemin dont la vue est née
  dans la route) sans en ouvrir un qu'il ne dit pas ? Reste-t-il, dans une partie **vivante** de la fiche ou du
  registre, une position ancienne (garde avant un verrou d'exercice, W24 entre les étapes 1 et 2) ou la phrase
  « sa position ne règle plus aucun ordre ; elle se place au plus tôt » ? (Les entrées P1/P2 du Change Log et le texte
  P1/P2 des choix annotés les gardent à dessein.)
- **Tests** : **T-a48** (entrelacement par virement, W25, W29, W28) est-il **réalisable** avec les montages existants
  (`crates/kesh-db/tests/invoice_settlement.rs:967`, `supplier_invoices_repository.rs:1186` et `:1568`) adaptés au
  virement, et `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:561-592`) ? Le motif
  `["fiscal_years", "FOR UPDATE"]` désigne-t-il la bonne requête bloquée, et seulement elle ? Sous la mutation (garde
  avancée), l'annulation **réussirait-elle** vraiment (le socle relit-il le statut de l'exercice ?), ou rendrait-elle
  un autre refus qui ferait passer le test pour une mauvaise raison ? Pour le cas (c), quel exercice faut-il clore pour
  que W28 refuse `FiscalYearClosed` ? **M36** : `a_concurrent_close_waits_for_the_unreconciliation`
  (`crates/kesh-db/tests/reconciliation_cancel.rs:128`) rougirait-il vraiment sous la garde avancée (son compte
  bancaire est-il ouvert, la garde passe-t-elle, la contre-passation réussirait-elle) ? Les trois tests existants
  déclarés aveugles (`invoice_settlement.rs:1266`, `supplier_invoices_repository.rs:1714`, `:2059`) le sont-ils
  vraiment ? T-a26 (cas `PUT` sur un clos avec `isPrimary: true`) : l'issue décrite est-elle celle du code (route
  `routes/bank_accounts.rs:719-760`) ? T-a47 sur la source aplatie : tue-t-il M53 pour **chacun** des deux appels, sans
  rougir sur un formatage correct ?
- **Mutations** : M16 (raison réécrite), M14-bis (réserve du lot sain), M36-M39 (rétablies), M54 (neuve, équivalente)
  — chaque raison est-elle vraie ?
- **L1 reclassée P2** (AC7, § *Angles morts*, AC16, C-28-4-21 annoté) : la classe et le motif de la dette tiennent-ils
  contre § *Priorités des défauts* du `CLAUDE.md` ? La phrase neuve d'`api-external.md` (AC16) dira-t-elle vrai ? La
  fenêtre de l'import (vue née à la recherche de doublons) est-elle juste ?
- **C-28-4-23** (W21 `SortieAutorisee`, cinquième statut du registre de l'AC8) : le statut est-il défini sans ambiguïté,
  le test de l'AC8 peut-il l'exiger, et l'exception à C-28-4-10 est-elle dite partout où C-28-4-10 est invoquée
  (AC5, *Périmètre minimal*, manuel) ?
- **Table refaite** : lignes `PUT` (principal démis l. 730 **avant** le compte cible), `accept_one_*` (lectures sans
  verrou, exercice l. 1911), `upsert_primary_in_tx`, W21, W26, `first_active_bank_account_linked_to` — exactes au sol ?
- **AC10** : les cinq assertions d'`audit_route_registry.rs` (l. 645, 648-657, 667, 990-991, 997) et leurs valeurs cibles.
- **Recomptes de la remédiation**, chacun **depuis la source** : 18 AC ; T-a1-T-a48, cinq retirés, 43 actifs ; 57
  numéros de mutation (M1-M54 + 3 `-bis`), 5 sans objet (M10, M19, M35, M49, M50), 4 équivalentes (M14-bis, M20-bis,
  M51, M54), 48 à tuer ; 16 sites « Gardée » hors W30 ; dix-sept lignes pour T-a27 ; 23 choix ; et le trend du Change
  Log (P1 R 0/1/6/7, F 0/3/3/5 ; P2 R 0/0/4/2, F 0/1/5/7 ; P3 R 0/1/2/5, F 0/1/2/6) contre les rapports. Vérifie aussi
  que l'entrée P2 du Change Log n'a été qu'**annotée** (ses chiffres d'origine intacts).

## Axes communs (repris de la P3)

1. **Périmètre minimal** : la section *Périmètre minimal* contre #642, phrase par phrase ; l'extension n° 6 (archivé)
   bornée comme elle le dit ; L1 et C-28-4-23 laissent-ils #642 fermée (« plus rien ne s'y enregistre »), au sens où
   la fiche le dit ? Le décompte des modules et la dérogation (C-28-4-1).
2. **Inventaire clos des écrivains** (W1-W30, R1-R8, R5 retiré) : refais les `grep` et élargis-les — les sept tables
   sous toutes graphies, les colonnes `bank_account_id`, `settlement_bank_account_id`, `is_primary`,
   `journal_account_id`, `status`, `auto_match_rejected_at`, `archived`, `closed_on` ; crates annexes, `admin_backup`,
   `post_restore`, `test_endpoints.rs`. Un écrivain absent, ou mal classé, est au moins **HIGH**. Chaque classement
   **Invariant** contre **tous** les écrivains de l'état et contre L1.
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Comptes bancaires* (l. 1642 et
   suivantes), § *Supprimer un import* (l. 1724, l. 1754-1757), § *Import bancaire* (l. 1696), § *Solde comptable*
   (l. 238-262), le règlement par virement ; `docs/manual/fr/admin-manual.tex` — l. 1943, § *Rollback en cas d'échec*
   (l. 2234, l. 2251), § *Passer à la 0.13.2* (l. 1972). Ce que l'AC14 prescrit d'écrire — dont la phrase sur L1 —
   sera-t-il vrai au code prescrit ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
4. **Recomptes** : chaque nombre de la fiche depuis la source (migrations 80 → 81, compteurs de
   `docs/migrations-idempotence-audit.md`, `migrations_upgrade_path.rs` et ses lignes `\b(80|46)\b`,
   `migrations.sha384`, routes 115 / 118 et les assertions d'`audit_route_registry.rs`, `sitesTotal` 2075, les 29 sites,
   les seize clés UI, numéros de ligne cités).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  l'axe obligatoire ; chaque phrase de #642 couverte par un AC testable, chaque AC par une tâche et par un test **qui
  rougirait**, chaque mutation par un test qui la tue ; chaque fait, chemin et numéro de ligne cité **revérifié**
  (`sed -n`, `grep -nF`) ; les textes prescrits (i18n ×4, manuels, CHANGELOG `### Ajouté` et `### Corrigé`,
  `api-external.md`) confrontés à ce que le code prescrit fera ; les gardes existantes qui changent de sens (dont
  `reconciliation_cancel.rs:128`, nommé à l'AC10) — la fiche les nomme-t-elle toutes ?
- **F — Faisabilité, chemins d'écriture et concurrence (adversaire de la conception)** : l'axe obligatoire en entier,
  puis : L1 sous InnoDB REPEATABLE READ — y a-t-il une fenêtre que la fiche ne déclare pas, ou une fenêtre déclarée
  qui n'existe pas ? Y a-t-il, sur un chemin gardé, une relecture faite **après** une attente qui dépend de la
  fraîcheur de la vue (exercice, lettrage, statut) et que la position de la garde condamnerait ? ; T-a31, T-a48 et
  leurs prémisses ; la contrainte `CHECK` de l'AC1 et le non-bump de `min_required` (C-28-4-11) ; le refus par
  proposition dans `accept` / `reject` contre § *Pattern batch* ; le registre statique de l'AC8 (cinq statuts) et son
  second inventaire ; l'AC6-bis route par route (mappage `DbError` → `AppError` lu, code HTTP rendu).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chacun, dis s'il vient de la **remédiation P3**
(commit `61b09ebc`), d'une remédiation antérieure, ou de la conception d'origine. Toute affirmation de présence ou
d'absence = sortie de `grep -nF` copiée. **Les deux lentilles rendent la table de l'axe obligatoire, à deux volets.**
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Un verdict de
concurrence est un raisonnement statique sur le code : écris-le comme tel, jamais comme une observation. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p4-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p4-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`,
`perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script
du dépôt, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle
dans le rapport. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard (ou vers un fichier d'aplatissement **dans** `kesh-gate-logs/`).
