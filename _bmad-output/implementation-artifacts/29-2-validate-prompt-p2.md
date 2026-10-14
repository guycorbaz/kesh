# Prompt — validation P2 de la spec, Story 29-2 (un paiement reçu après l'échéance, ou après un rappel, est proposé)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 était **Sonnet** ; P2 est
**Opus** pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-29-2`, branche `story/29-2` (base `16eeed21`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` — état relu : commit `375f2fbf` (remédiation
P1) ; état validé en P1 : `45f157f7`. Le diff de la remédiation : `git diff 45f157f7 375f2fbf -- _bmad-output/`.
Rapports de la P1 (à lire, **pas** à recopier) : `/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p1-R.md` et
`-F.md` ; leur traitement : Change Log de la fiche, entrée « Validation P1 ».
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-2*. Issue #529 (l'objet) :
`gh api repos/guycorbaz/kesh/issues/529` ; voisines #548 (Story 28-7, **à ne pas rouvrir** : proposition et
acceptation gardent une seule borne), #482 (28-6), #527 (29-3, tri des candidates, sérialisée après), #666 (reliquat,
choix manuel d'une facture), #665 (trop-perçu proposé puis refusé), #401 (frais de rappel).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` (C-29-2-1 à 12 ; 10 à 12 neufs,
7 et 9 complétés en P1).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif, dont
l'amendement D5 ; § Pattern batch ; § *Un appariement automatique propose, il ne crée jamais*) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (B5 : une modification qui
touche une transaction se relit sur l'ordre des verrous **et** la naissance de la vue).

## Axe 0 — obligatoire, commun aux deux lentilles : relire ce que la remédiation P1 a écrit

*La sévérité se déplace vers ce qu'on vient d'écrire* (`CLAUDE.md`, § *La passe ciblée*). Lis **chaque hunk** de
`git diff 45f157f7 375f2fbf` et, pour chacun : est-il **vrai au code** de `16eeed21` ? **cohérent** avec le reste de
la fiche (une phrase neuve qui en contredit une ancienne non touchée) ? **complet** (le symptôme corrigé subsiste-t-il
ailleurs — fiche, registre, prompt, Dev Notes) ? En particulier :

- **Cas j et k, M10a, M10b** : recalcule-les (`D` = 2026-06-15, bornes incluses). M10a et M10b sont-elles **tuées**
  par les tests nommés, et toutes deux à la fois par la parité HTTP ? Une autre forme de « borne haute sur l'ancre »
  survivrait-elle ?
- **Cas inter-sociétés et M11** : la mutation est-elle tuée par l'assertion `Ok(None)` dans ses deux formes (prédicat
  retiré avec et sans son `bind`) ?
- **`CAST(… AS DATE)` (C-29-2-10), M12, M13** : le `CAST` change-t-il quoi que ce soit au `WHERE` de la proposition
  (index, comparaison au `NaiveDate` lié), au décodage scalaire ? M12 est-elle **vraiment** équivalente dans les deux
  lecteurs ? Une mutation déclarée équivalente à tort est un test muet.
- **C-29-2-11 (faux rattachement non borné)** : la phrase réécrite est-elle exacte (ensemble `A ≥ D − 30` ; facture
  candidate de `F − 1` à `E + 30`) ? L'angle mort est-il **assumé avec une issue** (leçon de la rétro 28 : aucune
  dette assumée sans issue) — le commentaire demandé sur #527 suffit-il, ou faut-il une issue propre ? Le risque
  est-il assez grave pour que la 29-2 le traite elle-même (règle du `CLAUDE.md` sur le faux rattachement muet) ?
- **Textes prescrits** (AC8 : manuel `:1305`, `:2029`, `:2031`, `:2902`, `api-external.md`, CHANGELOG) : après la P1,
  promettent-ils encore plus que le code (montant, frais de rappel, #665) ou **moins** (paiement avant l'échéance) ?
  La phrase prescrite pour `api-external.md` est-elle juste sur l'histoire des raisons (`git log -S
  payment_date_outside_window`) ?
- **Inventaire (c) élargi** : refais la commande copiée ; la sortie est-elle identique ? Les verdicts « mention
  textuelle » des cinq fichiers ajoutés sont-ils justes (`bank_account_closed_inventory.rs`, `letterings_lexical.rs`
  sont des **inventaires d'écrivains** : la lecture ajoutée y entre-t-elle) ?
- **Numéros de ligne recomptés** (`:1716`, `:1740-1765`, `:1745`, `:653-655`, `:656-662`, `:721-845`,
  `invoice_settlements_write.rs:500`, `dunning_eligibility.rs:80`, `:128`) : chacun au `sed -n`.
- **Change Log P1** : ses décomptes (1 HIGH, 3 MEDIUM, 8 LOW distincts ; 14 lignes de mutation ; 11 cas ; 12
  choix ; 8 lignes d'exemple) se recomptent-ils depuis la source ? Le refus partiel de R9 est-il fondé ?

Un défaut introduit par la remédiation P1 est **au moins MEDIUM** s'il touche un AC, un test ou un texte prescrit.

## Axes communs (repris de la P1, sur l'état neuf)

1. **Prémisses** (§ *Les prémisses, établies au code*) : toujours justes au code et au manuel de `16eeed21` ?
2. **Périmètre minimal et fermeture** : la règle ferme-t-elle #529 tel qu'il est écrit ; le reliquat est-il couvert
   par #666 et son contournement (C-29-2-8, rejeter la transaction) juste au code ? Recompte les modules (trois).
3. **Une seule borne** : proposition et acceptation lisent-elles la **même** définition de l'ancre (C-29-2-4) ? Toute
   voie de divergence qui ferait proposer ce qui sera refusé (#548) est au moins **HIGH**.
4. **Faux rattachement** : cf. axe 0 ; `propose_matches` (`matching.rs:96-103`), `candidates[0]`, *Accepter (N)*.
5. **Inventaire clos des sites** (a), (b), (c) : refais et élargis les `grep` — un détecteur part du symptôme.
6. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1305, 1307 (`keshwarning` des frais de
   rappel), 2029, 2031 (`\label{sec:facture-posterieure}`), 2902 ; contrôle aussi le **PDF**
   `docs/manual/fr/user-manual.pdf`, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
   Même exigence pour `docs/api-external.md` § *Accepter des propositions de rapprochement* et le CHANGELOG prescrit.
   Une promesse que le code prescrit ne tiendrait pas est au moins **MEDIUM**.
7. **Recomptes** : chaque nombre et chaque numéro de ligne de la fiche, depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #529 et du § *29-2* de l'epic couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** ; chaque fait, chemin et numéro de ligne revérifié (`sed -n`, `grep -nF`) ; les textes prescrits
  confrontés au code prescrit ; les gardes existantes qui pourraient changer de sens (`textes_coherents.rs`,
  `test_schema_guard.rs`, `failed-proposal-label.test.ts`, les `post_accept_rejects_payment_date_*`, les deux tests de
  la 28-7, les inventaires d'écrivains) ; le Pattern batch sur chaque issue de la lecture ajoutée, et l'honnêteté de
  l'angle mort qu'écrit désormais l'AC4.
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : l'expression
  `CAST(GREATEST(i.date, COALESCE(i.due_date, i.date), COALESCE(DATE(rmd.last_sent_at), i.date)) AS DATE)` en MariaDB
  10.11 (types, `NULL`, comparaison au `NaiveDate` lié, décodage) ; la table dérivée `rmd` avec les jointures du reste
  dû (alias, multiplication de lignes, `HAVING amount_due`) ; les tableaux (exemples, cas a-k) recalculés ligne par
  ligne et réalisables (exercice qui couvre `D − 90` et `D + 32`, montants uniques sous la tolérance de 0,05, colonnes
  `NOT NULL` d'`invoice_reminders`) ; **B5** (première lecture ordinaire du chemin d'acceptation, verrous) ; l'ordre
  des refus ; l'index ; **toutes les mutations** M1-M13 tuables, et celles qui manquent encore.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dis s'il vient de la
**remédiation P1** (hunk cité) ou de la conception d'origine. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés**, axe 0 compris, hunk par hunk — un « 0 finding »
sans elle ne compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p2-R.md`
(lentille R) ou `/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p2-F.md` (lentille F) ; dernier message : le chemin
du rapport, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris, `git checkout <ref> -- <fichier>` compris)/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/
`worktree`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh`
sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de
`~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`/`blame` (pour lire une version ancienne : `git show <ref>:<chemin>`), `gh api` en lecture,
`pdftotext` vers la sortie standard.
