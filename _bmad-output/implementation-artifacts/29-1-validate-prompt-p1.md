# Prompt — validation P1 de la spec, Story 29-1 (préparer la clôture d'un exercice)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1` (base `16eeed21`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-1-preparer-cloture.md` (une fiche, **deux** stories : 29-1a contrôles
préalables, AC1-AC9 ; 29-1b contre-passation des transitoires, AC10-AC19). Registre de la story :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 15). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-1 ; registre de l'epic
`epic-29-choix-autonomes.md` (C-29-0-3 : la fiche commence par un inventaire). Issue #232 (`gh api
repos/guycorbaz/kesh/issues/232` et `…/232/comments`, lecture seule). Fiche de référence du modèle comptable :
`14-1-cloture-report-des-soldes.md`. Rétrospective `epic-28-retro-2026-10-14.md` § 4.2 et § 7 (B3, B4, B5).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Issue
Tracking Rule ; § Un appariement automatique propose ; § Priorités des défauts, report de la TVA) ;
`docs/agents-orchestration.md`.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire de l'existant est-il vrai ?** La fiche affirme, tableau par tableau, ce que le code fait déjà
   (clôture, réouverture, report calculé, soldes de départ, contre-passation datée du jour, verrou de période, écart
   du relevé) et corrige des prémisses de #232 : report à nouveau existant, écritures de clôture écartées par
   conception, **aucun compte d'attente** dans Kesh, contrôle TVA reporté. Revérifie chaque ligne au code (`grep -nF`,
   `sed -n`) — une prémisse corrigée à tort, ou un besoin de #232 écarté sans raison valable, est au moins **HIGH**.
   Cherche en particulier un compte d'attente ou une notion voisine que l'inventaire aurait manquée (plans JSON,
   rôles, réconciliation, import, `kesh-seed`).
2. **Périmètre minimal** : chaque AC, tâche et test se rattache-t-il à une phrase de #232 (ou au fichier d'epic) ?
   Toute extension non justifiée est au moins **MEDIUM** ; toute exclusion (table *Dehors*, C-29-1-2, C-29-1-4) qui
   laisserait #232 fermée sans que sa demande soit tenue **ni** tracée ailleurs (issue demandée) est au moins **HIGH**.
   Le décompte des modules (cinq par sous-story) et le découpage C-29-1-1 tiennent-ils ?
3. **Le modèle comptable** : la 29-1b ne génère qu'une contre-passation ; vérifie qu'elle ne double aucun calcul du
   modèle virtuel (`kesh-report/src/balance_sheet.rs`, `income_statement.rs`) — bilans de N et de N+1 justes,
   équation tenue, résultat de N inchangé par une contre-passation datée dans N+1. K6 couvre-t-il **toutes** les
   causes d'un `equation_holds` faux (relis `balance_sheet.rs` et `income_statement.rs` : filtres `entry_date`,
   `fiscal_year_id`) ?
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` — § *Clôture d'un exercice* (l. 732) et la
   phrase l. 728 « (si clôturé) » que la fiche dit fausse ; § *Arrêter les livres à une date* (l. 581-626, la
   contre-passation « datée du jour ») ; § *Corriger une écriture : la contre-passation* (l. 628) ; résumé l. 2992 ;
   `docs/manual/fr/admin-manual.tex` § *Test de restauration périodique* (l. 1564) et § *Export/import d'installation*
   (l. 1817), où la répétition à blanc s'insère. Ce que le manuel dit **aujourd'hui** est-il vrai au code ? Ce que les
   AC8, AC17 et AC19 prescrivent d'écrire sera-t-il vrai au code que la fiche prescrit (dont les limites de K3, K5,
   K6 et l'angle mort C-29-1-14) ? La marche de répétition à blanc est-elle **exécutable** telle qu'écrite (version de
   la copie, documents hors sauvegarde, rien rapporté vers la production) ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Une promesse que le code ne tiendrait pas
   est au moins MEDIUM.
5. **Recomptes** : 19 AC, T-a1 à T-a13, T-b1 à T-b9, M-a1 à M-a10, M-b1 à M-b9, C-29-1-1 à 15, les routes du registre
   (118 / 121 dans la doc de `audit_route_registry.rs`), chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #232 retenue couverte par un AC testable, chaque AC par une tâche et par un test **qui rougirait** ;
  chaque mutation tuée par le test nommé (une mutation équivalente sans le dire est un finding) ; chaque fait, chemin
  et numéro de ligne cité revérifié ; les sept contrôles (AC1) : la donnée existe-t-elle, la requête décrite rend-elle
  ce que la table promet (bornes inclusives, multi-société, comptes clos/archivés de K3, absence de date métier pour
  K5) ? Les textes prescrits (i18n ×4, manuels, CHANGELOG, `api-external.md`) confrontés à ce que le code fera ; les
  gardes existantes qui changeront de sens ou rougiront (`audit_route_registry.rs` et ses compteurs,
  `audit_label_registry.rs`, `textes_coherents.rs`, `i18n-keys.test.ts`, `lint-i18n-ownership`, tests de
  `list_by_company_with_balances`, tests de la contre-passation et des quatre annulations, `fiscal-years.spec.ts`,
  `journal-entries.spec.ts`) — la fiche les nomme-t-elle toutes ? L'écran (AC5, AC6, AC15) : rôles, états vide /
  erreur / chargement, liens qui existent réellement (routes du frontend), clair et sombre.
- **F — Faisabilité, verrous et naissance de la vue (adversaire de la conception)** — axe B5 nommé : *une
  modification qui touche une transaction se relit sur l'ordre des verrous **et** la naissance de la vue*.
  - 29-1a : `ReadSnapshot` convient-il (connexion, `READ ONLY`, exécuteur générique pour l'extraction de l'AC3) ?
    L'extraction de `list_by_company_with_balances` avec une borne laisse-t-elle **strictement** inchangé le résultat
    du tableau de bord (forme des sous-requêtes, liaisons `?`, `ROW_NUMBER`) ? K3 mesure-t-il le bon écart à la
    borne (correction des décalages valeur / comptabilisation relative à `period_to`) ? La garde statique T-a2 est-elle
    réalisable sans faux positif ?
  - 29-1b : relis `reverse_in_tx_inner` (`crates/kesh-db/src/repositories/journal_entries.rs:2595` et suivantes),
    `create_in_tx_inner` (l. 281), `fiscal_years::close` (l. 1054), `reopen` (l. 1197), `create` (l. 313),
    `find_open_covering_date` (l. 735). La lecture neuve de l'exercice suivant, placée après `reversal_blockers`,
    déplace-t-elle la naissance de la vue (vérifie que `reversal_blockers` est bien la première lecture **ordinaire**
    de la transaction, y compris quand une autorité fournisseur est passée — étape 1-bis) ? Les verrous et leur ordre
    sont-ils identiques au chemin `Today` ? L'affirmation de l'AC12 — la clôture de N+1 attend la contre-passation
    parce que l'étape (1) tient N et que `close` verrouille ses antérieurs en (b') — est-elle vraie, et T-b5 est-il
    montable tel qu'écrit ? Cherche un cycle **neuf** (avec `reopen`, non rejouée ; avec `create` d'un exercice ; avec
    le lettrage ; avec le verrou de période). Les refus de `create_in_tx_inner` (`PeriodLocked`,
    `LaterFiscalYearClosed`, `DateOutsideFiscalYear`) se comportent-ils comme l'AC11 l'affirme pour une date au
    premier jour de N+1 ? Le lettrage de l'étape (5) (`Mode::System { held_open_fiscal_year_id }`) tient-il quand
    l'origine est dans un exercice clos ? La route neuve (C-29-1-10) contre `docs/api-external.md` § 4 et § 7 (clés
    d'API, liste des routes rejouées l. 730) et contre le registre des routes.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Lecture seule stricte : **aucun `git checkout`,
`git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/
`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`, `perl`, `ruby` et tout autre
interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête SQL**, `bash` sur un script du dépôt, `gh` en
écriture. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés : lecture, `grep`,
`sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie
standard.
