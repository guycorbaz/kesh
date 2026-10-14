# Prompt — validation P2 de la spec, Story 29-1 (préparer la clôture d'un exercice)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 a été faite par **Sonnet** et
remédiée par Opus : P2 est confiée à **Opus** pour les deux lentilles — en contexte frais, sans rien reprendre de la
session qui a écrit la remédiation.*

Dépôt `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`. Fiche :
`_bmad-output/implementation-artifacts/29-1-preparer-cloture.md` (une fiche, **deux** stories : 29-1a contrôles
préalables, AC1-AC9 et **AC3-bis**, neuf ; 29-1b contre-passation des transitoires, AC10-AC19). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 22 ; 16 à 22 ajoutés par la
remédiation P1). Rapports de P1 : `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p1-R.md` et `-F.md`. Le Change Log
de la fiche résume la remédiation, finding par finding. Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`
§ 29-1. Issue #232 (`gh api repos/guycorbaz/kesh/issues/232` et `…/232/comments`, lecture seule) ; #667 et #398 (même
façon). Rétrospective `epic-28-retro-2026-10-14.md` § 4.2 et § 7 (B3, B4, B5). Règles : `CLAUDE.md` (§ Review
Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation
post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Issue Tracking Rule ; § Priorités des
défauts) ; `docs/agents-orchestration.md`.

**Bornes du diff de la remédiation** : `git diff b473714b 0828925a -- _bmad-output/` (fiche, registre, registre de
sprint). La fiche d'avant P1 : `git show b473714b:_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`.

## Axe obligatoire — relire ce que la remédiation P1 a écrit

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire* (Epics 22 et 23 : la plupart des
défauts de passe N+1 sont nés du patch de passe N). Lis le diff ci-dessus **en entier**, et pour chaque passage neuf :

1. **Est-il vrai au code ?** Chaque ligne, fichier, nom de fonction, constante, motif de requête cité par la
   remédiation se revérifie (`grep -nF`, `sed -n`) : `fiscal_years.rs` l. 150-151, 178-179, 189-192, 710-760, 789,
   1081-1090 ; `bank_accounts.rs` l. 1392-1515 (requête, ordre des `.bind`, `match` de `BalancesRow`) ;
   `journal_entries.rs` l. 2488, 2574-2579, 2603-2607, 2616-2664, 2847 ; `homepage.ts` l. 53-72 et
   `homepage.test.ts` l. 57-80 ; `+page.svelte` des exercices l. 67-71, 371-373, 393, 459 ; fiche d'écriture
   l. 305-350 ; `fiscal-years-page.test.ts` l. 42-48 ; `journal-entry-page.test.ts` l. 34-38 ;
   `frontend/scripts/lint-i18n-ownership.js` ; `frontend/tests/e2e/helpers/test-state.ts` (préréglage
   `with-company-no-fy` : vide-t-il vraiment la base, pose-t-il des comptes bancaires ?) ;
   `docker-compose.yml` l. 5, 46, 50, 134-135, 159, 169 ; `docker-compose.prod.yml` l. 62-63, 73, 159, 165-177 ;
   `scripts/prepare-release.sh` l. 237-248 ; `audit_route_registry.rs` l. 27-32, 659-662, 682, 1012 ;
   `crates/kesh-core/src/types/money.rs` l. 66-71.
2. **A-t-elle introduit un défaut ?** Contradiction avec un passage resté de la version P1 ; règle énoncée deux fois de
   deux façons ; décompte du Change Log faux (20 AC, T-a1 à T-a15, T-b1 à T-b9, M-a1 à M-a18, M-b1 à M-b12, C-29-1-1
   à 22 — **recompte depuis la fiche et le registre**) ; mutation déclarée tuée qui ne le serait pas, ou équivalente
   sans le dire ; test qui passerait à vide ; promesse que le code prescrit ne tiendrait pas.
3. **Les remèdes partiels déclarés** (R6 : deux définitions de l'écart, TypeScript et Rust ; R11 : priorité de #667)
   sont-ils acceptables tels qu'écrits, et tracés ?
4. **Ce que la remédiation n'a pas touché** mais aurait dû : chaque finding de P1 est-il réellement fermé dans la
   fiche (relis les deux rapports P1 ligne à ligne contre la fiche actuelle) ?

## Axes prioritaires (communs aux deux lentilles)

5. **Périmètre minimal** : l'AC3-bis (lectures partagées dans `fiscal_years.rs`, factorisation du texte de l'étape (d)
   de `close`), la colonne d'actions rendue pour tous, le motif de la fiche d'écriture, la brochure et le README ajoutés
   à la 29-1b — chacun se rattache-t-il à #232, à une règle du dépôt (DRY, « le motif à la place du bouton ») ou à un
   finding de P1 ? Le décompte des modules (cinq par sous-story, `fiscal_years.rs` étant dans `kesh-db`) tient-il ?
6. **K3 refondu** (C-29-1-16) : la population par l'activité dans Y est-elle calculable par une requête lisible sous
   l'instantané, sans `bank_transactions` dans le même `FROM` que les lignes du grand livre ? La précédence couvre-t-elle
   **tous** les cas (compte clos pendant Y sans relevé ; compte lié partagé **et** CSV seul ; relevé à solde unique
   postérieur à Y) ? Un compte vérifiable mais sans activité dans Y laisse-t-il passer un vrai défaut (solde de fin
   d'année non confirmé) que le manuel devrait nommer ?
7. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Clôture d'un exercice* (l. 732), la
   phrase l. 728 « (si clôturé) », § *Arrêter les livres à une date* (l. 581-626), § *Corriger une écriture : la
   contre-passation* (l. 628), § *Exercices dans le désordre* (l. 760-766, la phrase l. 764), résumé l. 2992 ;
   `docs/manual/fr/admin-manual.tex` § *Test de restauration périodique* (l. 1564), § *Export/import d'installation*
   (l. 1817-1940 ; avertissement « secret » l. 1868, sauvegarde pré-import l. 1891) ;
   `docs/manual/fr/marketing-brochure.tex` l. 418 ; `README.md` l. 47. Ce que les AC8, AC17 et AC19 prescrivent
   d'écrire sera-t-il vrai au code prescrit ? **La marche de répétition à blanc (AC19) est-elle exécutable pas à pas**
   par quelqu'un qui n'a que le manuel — commandes, nom de projet, lignes à éditer, image épinglée, base qui n'est
   jamais celle de la production, aucun `KESH_SMTP_*`, suppression du fichier, de la copie et de la sauvegarde
   pré-import — et n'expose-t-elle aucun secret (rien dans un dossier synchronisé ni un dépôt) ? Contrôle aussi les
   **PDF**, aplatis : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, de même pour
   `admin-manual.pdf` et `marketing-brochure.pdf`. Une promesse que le code ne tiendrait pas est au moins MEDIUM ;
   une marche qui pourrait toucher la base de production ou exposer un secret est au moins HIGH.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque AC couvert par une tâche et par un test **qui rougirait** ; chaque mutation (M-a1 à M-a18, M-b1 à M-b12)
  confrontée au test nommé — y compris la justification d'équivalence de M-a9 (non-chevauchement des exercices :
  vérifie `find_overlapping` et tous les chemins de création d'exercice, `create`, `create_if_absent_in_tx`,
  `create_for_seed`, l'import d'installation) ; les textes prescrits (i18n ×4 aux préfixes `fiscal-years-checks-` et
  `journal-entries-reverse-next-year-`, manuels, brochure, README, CHANGELOG `## [0.14.0] — Non publié`,
  `api-external.md`) ; les gardes existantes qui changeront de sens ou rougiront — `audit_route_registry.rs` et ses
  compteurs nommés, `textes_coherents.rs` (lis le **contenu** des gardes, non leurs seuls noms : l'une lit-elle le
  README, la brochure, les clés i18n ?), `audit_label_registry.rs`, `i18n-keys.test.ts`, `lint-i18n-ownership`, les
  deux tests de page à adapter, `fiscal-years.spec.ts`, `journal-entries.spec.ts`, les tests 15-8a et 15-12a qui
  reconnaissent le texte de l'étape (d) de `close` — la fiche les nomme-t-elle toutes ? L'écran (AC5, AC6, AC15) :
  rôles, états vide / erreur / chargement, liens, clair et sombre.
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — **axe B5 nommé** : *une
  modification qui touche une transaction se relit sur l'ordre des verrous **et** la naissance de la vue*.
  - 29-1a : la factorisation de `FIND_EARLIER_OPEN_SQL` en `EARLIER_OPEN_SQL` + `FOR UPDATE` laisse-t-elle **strictement**
    inchangés le texte exécuté par `close`, ses verrous et leur ordre ? `find_earliest_earlier_open` et
    `find_next_after` sous `ReadSnapshot` : exécuteur générique, aucune lecture verrouillante ? La borne `(? IS NULL OR
    period_to <= ?)` dans la dérivée `st`, avant `ROW_NUMBER`, et l'ordre des sept liaisons prescrit à l'AC3 : laisse-t-il
    le tableau de bord inchangé pour `None` ? La garde T-a2 sur constantes est-elle réalisable sans faux positif et
    sans angle mort (une requête construite par `format!` ?) ?
  - 29-1b : relis `reverse_in_tx_inner` (`journal_entries.rs` l. 2595 et suiv.), `create_in_tx_inner` (l. 281),
    `fiscal_years::close` (l. 1054), `reopen` (l. 1197), `create` (l. 313), `find_open_covering_date` (l. 735). La
    lecture neuve `find_next_after` après `reversal_blockers` déplace-t-elle la naissance de la vue sur le chemin
    `authority = None` ? L'AC12 réécrit — sûreté par la confirmation verrouillante de N+1, attente de `close(N+1)`
    seulement si N est le premier exercice, cycle connu sinon — est-il **vrai** ? **T-b5 tel que réécrit est-il
    montable et dit-il vrai** : ordre (i) (A va au bout sans valider, B attend en (b') au motif de
    `LOCK_EARLIER_BY_ID_SQL` — ce motif est-il unique parmi toutes les requêtes que B et A peuvent avoir en cours ?
    « N tenu deux fois » est-il établi, et par quoi ?) ; ordre (ii) (B monté à la main avec les verrous et l'effet de
    `close`, A attend à l'étape (1), puis rend `NEXT_FISCAL_YEAR_CLOSED` — la vue de A naît-elle bien **après** le
    `COMMIT` de B ? M-b3 et M-b11 y sont-ils réellement tués, ou M-b11 rend-il le même résultat par un autre chemin ?).
    Cherche un cycle **neuf** (avec `reopen`, non rejouée ; avec `create` ; avec le lettrage ; avec le verrou de
    période). Les refus de `create_in_tx_inner` pour une date au premier jour de N+1 se comportent-ils comme l'AC11
    l'affirme ? La route neuve contre `docs/api-external.md` § 4 et § 7 et le registre des routes.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il **naît de la remédiation P1**
(passage du diff `b473714b..0828925a`) ou s'il préexistait. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** (axe obligatoire et axes 5 à 7 compris) — un
« 0 finding » sans elle ne compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p2-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p2-F.md` (lentille F) ; dernier message : le chemin du
rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

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
