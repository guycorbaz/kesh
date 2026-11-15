# Epic 36 — registre des choix autonomes de la story 36-7 (`36-7-tests-desancres-2026`, #1125)

Choix faits à la rédaction de la fiche (2026-11-15, Opus 5.5, worktree `kesh-36-7`, code lu sur `0bf3c3b4b`), dans le cadre
de C-e360-13. Chaque choix : contexte, option retenue, options écartées, réversibilité.

### C-36-7-1 — L'inventaire de la fiche se fait par exécution, à la tête, avant toute écriture

- **Contexte** : #1125 nomme des sites relevés à la 35-5a, sur un commit antérieur ; les 35-5b, 35-5c et la phase 1 de
  l'Epic 36 ont ajouté des tests depuis. `e35-dev-commun.md` § *Inventaires* demande un détecteur par exécution quand il
  existe.
- **Retenu** : nextest du workspace sous `KESH_TEST_INSTANT=2027-01-04T10:00:00Z` (voie 2, base `kesh_e367`) et Vitest
  sous un préchargement Node qui décale `Date`, joués par l'agent de la fiche ; scripts et journaux dans
  `kesh-gate-logs/e36-7/`. Résultat : 13 rouges Rust (les mêmes qu'à la 35-5a), 0 Vitest.
- **Écartées** : reprendre la liste de l'issue ; inventaire par lecture des 175 fichiers de tests Rust qui portent « 2026 »
  (2762 lignes).
- **Réversible** : oui (relevé seulement, aucune écriture au dépôt).

### C-36-7-2 — L'E2E en 2027 sous libfaketime, dont l'installation est demandée à Guy

- **Contexte** : le critère de l'epic exige l'E2E vert « sous `KESH_TEST_INSTANT` », variable que ni le binaire servi
  (`docs/testing.md:119`), ni le runner (`api-fixtures.ts:393`), ni le navigateur ne lisent. La bibliothèque n'est pas
  installée (`apt-cache policy faketime` : « Installé : (aucun) »).
- **Retenu** *(récrit le 2026-11-15, validation P3 de la 36-7a, V3-6 ; la forme d'origine portait
  `FAKETIME_DONT_FAKE_MONOTONIC=1` et l'installation à faire)* : backend (binaire construit) et runner lancés sous
  `flock … scripts/mem-guard.sh env TZ=Europe/Zurich NO_FAKE_STAT=1 faketime --exclude-monotonic -f '@2027-01-04
  11:00:00' <commande>` ; sonde à trois étages (serveur, runner, navigateur) ; MariaDB à l'heure réelle, angle mort écrit
  et preuve exigée pour tout rouge qui s'y rattache. La bibliothèque est installée (amendement ci-dessous).
- **Écartées** : faire lire la variable au binaire en mode test (code de production, contraire à la garde) ; `page.clock`
  (navigateur seul) ; conclure sur la seule lecture des specs ; construire libfaketime depuis ses sources hors dépôt
  (téléchargement de code externe, à décider par l'orchestrateur si Guy n'installe pas le paquet).
- **Réversible** : oui (outillage de la station, aucun fichier du dépôt).
- **Amendé le 2026-11-15 (validation P2 de la 36-7a, V2-5, V2-9)** : libfaketime est **installée**
  (`/usr/bin/faketime`, `/usr/lib/x86_64-linux-gnu/faketime/`) ; le détecteur `ldconfig -p` ne la voyait pas. Commande
  retenue : `flock … scripts/mem-guard.sh env TZ=Europe/Zurich NO_FAKE_STAT=1 faketime --exclude-monotonic -f '@2027-01-04
  11:00:00' <binaire construit>`, chaque option citée de `man faketime`, sonde `sonde-p2-S3b.log`.

### C-36-7-3 — Les trois sites à borne de 2026 antérieure entrent au périmètre

- **Contexte** : `admin_full_import_e2e:2007`, `:2281` et `services_rollout:207` rougissent sous un instant antérieur à
  leur borne, jamais à l'heure réelle ; #1125 les porte en commentaire.
- **Retenu** : les corriger dans la story (D2), pour que #1125 puisse se fermer et que la campagne d'instants n'ait plus de
  rouge attendu.
- **Écartée** : une issue à part — même classe, même fichier pour deux d'entre eux.
- **Réversible** : oui.
- **Amendé le 2026-11-15 (validation P1, V1-1)** : la classe compte **neuf** tests, non trois. Relevé par la suite
  entière sous `KESH_TEST_INSTANT=2026-03-10T23:30:00Z` à la tête (`kesh-gate-logs/e36-7/inventaire-B2026.log`, 9
  failed) : les trois d'origine, plus `invoice_echeancier_e2e`, `reconciliation_e2e`, `supplier_settlement_cancel_e2e`
  (les trois au `lock_books` d'une borne de 2026), `project_reports_e2e`, `supplier_invoice_draft_e2e` et
  `kesh-db::supplier_invoices_draft`. Tous entrent au périmètre ; l'AC2 garde l'instant antérieur, désormais atteignable.

### C-36-7-6 — AC4 et AC5 (libfaketime) sont des gates de clôture

- **Contexte** *(récrit le 2026-11-15, validation P3 de la 36-7a, V3-6 ; la forme d'origine disait libfaketime absente)* :
  AC4 et AC5 dépendent de libfaketime, installée depuis le 2026-11-15 ; les corrections de montage (AC1-AC3, AC6-AC9) n'en
  dépendent pas.
- **Retenu** : le développement passe en `review` sans libfaketime ; la story ne passe pas `done` et la PR de release ne
  porte pas `closes #1125` tant qu'AC4 et AC5 n'ont pas tourné ; un rouge rattaché à l'angle mort MariaDB exige sa preuve
  (ligne SQL, colonne, rejeu isolé) ; si le navigateur ne suit pas `LD_PRELOAD`, l'AC5 se déclare non constatable pour
  lui, sans contournement non éprouvé ; `TZ=Europe/Zurich` fixé dans la commande.
- **Écartées** : sortir AC4/AC5 de la story (le critère de l'epic exige l'E2E en 2027) ; bloquer tout le développement
  sur l'installation.
- **Réversible** : oui.
- **Amendé le 2026-11-15 (validation P2 de la 36-7a, V2-5)** : la prémisse « absente » est tombée (bibliothèque
  installée) ; la règle reste : pas de `done` sans AC4 et AC5 joués.

### C-36-7-8 — L'étage navigateur de la sonde est une spec versionnée

- **Contexte** : validation P2 de la 36-7a, V2-9 — l'étage navigateur doit se jouer dans les conditions de la suite ; le
  runner ne lit que `testDir: 'tests/e2e'` (`frontend/playwright.config.ts:22`).
- **Retenu** : `frontend/tests/e2e/horloge-sonde.spec.ts`, qui vérifie que serveur, runner et navigateur disent le même
  jour ; vraie à l'heure réelle, sonde à trois étages sous faketime ; entrée justifiée dans `ALLOWLIST_SPECS` de
  `local-date-guard.test.ts`. Ajoute le module `frontend` à la 36-7a (cinq modules).
- **Écartées** : spec hors dépôt (non lue par le runner) ; `chromium.launch()` autonome (autres conditions) ; aucune
  sonde navigateur.
- **Réversible** : oui (une spec et une entrée de garde).
- **Amendé le 2026-11-15 (validation P3 de la 36-7a, V3-4)** : l'alternative d'une configuration Playwright hors dépôt,
  qui importe celle du dépôt et ne déplace que `testDir`, a été **essayée** (`kesh-gate-logs/e36-7/sonde-pw/`, journal
  `sonde-pw-list.log`) : la spec hors dépôt résout `@playwright/test` dans `/usr/share/nodejs/playwright` (paquet système),
  Playwright refuse (« two different versions of @playwright/test »), 0 test listé. La faire marcher demanderait un
  contournement non éprouvé ; la spec versionnée est gardée, au prix du cinquième module, et sert aussi aux campagnes de
  fin d'année. L'arbitrage reste à l'orchestrateur, qui le sait. La garde reçoit l'entrée et ses trois mentions du nombre
  (`:28-29`, `:263`, `:318`) passent à quatre.

### C-36-7-4 — Aucun outillage versionné : les recettes vont dans `docs/testing.md`

- **Contexte** : Guy veut des stories courtes (2026-11-15). Un script de campagne de fin d'année ou un setup Vitest lisant
  `KESH_TEST_INSTANT` serait une fonction neuve.
- **Retenu** : recettes écrites dans `docs/testing.md` ; préchargement Vitest et scripts hors dépôt.
- **Écartée** : `scripts/test-2027.sh`, setup Vitest versionné — proposables en issue.
- **Réversible** : oui.

### C-36-7-5 — Le placeholder `CLI-2026-00042` n'est pas un défaut

- **Contexte** : `frontend/src/routes/(app)/contacts/+page.svelte:1067` porte une année dans un exemple de saisie.
- **Retenu** : laissé tel quel, sans issue : c'est un exemple de forme, non une date affichée comme actuelle — à la
  différence de l'aperçu de numérotation de `settings/invoicing/+page.svelte:159`, pour lequel une issue est demandée.
- **Écartée** : une issue P4 de plus.
- **Réversible** : oui.

### C-36-7-7 — La 36-7 se découpe en 36-7a (2027) et 36-7b (instant antérieur)

- **Contexte** : après la remédiation de la validation P1, la story portait vingt-deux tests, quatorze fichiers,
  dix-neuf mutations et quatre campagnes de suite entière, plus un E2E dépendant de libfaketime. Règle de Guy du
  2026-11-15 : « éviter les stories trop longues ou trop complexes ». Découpage proposé par l'agent de la fiche, approuvé
  par l'orchestrateur le même jour.
- **Retenu** : **36-7a** — les treize rouges de 2027 (sept fichiers, quatre modules, M1 à M10), AC4 et AC5 sous faketime
  en gates de clôture (C-36-7-6), urgente d'ici le 2026-12-28, porteuse des décisions communes ; **36-7b** — les neuf rouges
  sous instant antérieur (huit fichiers, trois modules, M1 à M9), sans échéance, développée après la fusion de la 36-7a
  (fichiers communs : `admin_full_import_e2e.rs`, `docs/testing.md`). Les deux portent `refs #1125` ; la PR de release ne
  porte `closes #1125` qu'avec la 36-7b close et AC4, AC5 joués. La fiche d'origine passe à `split`.
- **Écartées** : une seule story (trop longue) ; un découpage par crate (la 36-7a mêlerait alors 2027 et l'instant
  antérieur, et l'échéance du 2026-12-28 porterait sur des tests qui n'en ont pas) ; AC4/AC5 en story à part (le critère de
  l'epic les lie aux treize rouges).
- **Réversible** : oui (fiches seulement).
- **Amendé le 2026-11-15 (validations P2 et P3 de la 36-7a, V3-6)** : la 36-7a compte désormais **cinq modules**
  (`frontend` ajouté par la sonde navigateur, C-36-7-8), sept fichiers corrigés plus une spec et la garde, **dix-neuf
  mutations** (M1a à M10) et trois mutants équivalents déclarés ; sa **fusion doit précéder le 2026-12-26**
  (`open_items_invariant` rougit à l'heure réelle dès ce jour, D5 de la 36-7a), l'échéance du 2026-12-28 restant celle de
  Guy pour la story.
- **Amendé le 2026-11-15 (validation P4 ciblée de la 36-7a, V4-1, V4-4)** : deux mutants équivalents déclarés (`:1332`,
  `:533`), et non trois — l'équivalence de `bank_statements_manual_e2e:1008` se tranche à l'étape 0 ; M2a porte sur un
  `UPDATE` de l'exercice dans le seul test corrigé, `setup()` restant intouché (21 tests). Dix-neuf mutations, inchangé.
