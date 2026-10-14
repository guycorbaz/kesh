# Prompt — revue de code P1, Story 29-2 (un paiement reçu après l'échéance, ou après un rappel, est proposé)

*Versionné le 2026-10-14. Trois lentilles — B (Blind), E (Edge cases), A (Acceptance) —, contexte frais chacune,
lecture seule stricte. Modèle : **Sonnet** pour les trois (passe complète ; rotation D6 de la rétrospective 25 :
Sonnet ↔ Opus pour les passes complètes, Haiku pour la seule passe ciblée de fin de boucle).*

Dépôt `/home/gcorbaz/devel/kesh-29-2`, branche `story/29-2` (rebasée sur `main` `fcd2ae75`). Le code à relire :
`git -C /home/gcorbaz/devel/kesh-29-2 diff c50f56e8 HEAD` (`c50f56e8` = fiche validée, dernier commit avant le
développement). Langue du rapport : français.

Fiche : `_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` (AC1 à AC10, Dev Agent Record) ; registre :
`_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` (C-29-2-1 à 16 ; 15 et 16 neufs au
développement, 8 annoté). Issue #529 (`gh issue view 529`) ; voisines #548 (28-7, **une seule borne, à ne pas
rouvrir**), #668 (29-5, départage à score égal — elle réécrira l'ancre à partir d'une expression d'exigibilité :
C-29-2-15), #665 (29-6, qui livrera le rejet d'une ligne sans proposition : C-29-2-8), #666 (reliquat), #401 (frais de
rappel). Règles : `CLAUDE.md` (§ Review Iteration Rule et sous-sections : manuel nommé, axes déclarés, sites non
résolus, propagation, recompte ; § Pattern batch ; § Un appariement automatique propose, il ne crée jamais ; § Migration
breaking policy) ; `docs/agents-orchestration.md` § 6 ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (B5 :
ordre des verrous et naissance de la vue).

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport.
- **N'exécute rien qui écrive, compile ou exécute** : ni `cargo` (aucune sous-commande), ni `npm`, ni `make`, ni
  `latexmk`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script
  du dépôt ; ni `python3`, ni `perl`, ni `sed -i`, ni `docker`, ni `mariadb`.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni de fichier, ni de branche), aucun `git` qui écrit
  (`commit`, `stash`, `reset`, `restore`, `tag`, `push`, `worktree`). Aucune commande `gh` qui écrit ; `gh issue view`
  est permis.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `ls`, `wc`, `pdftotext` vers la
  sortie standard uniquement.

## Axes communs aux trois lentilles

1. **Une seule borne** : proposition (`find_unpaid_invoices_for_window`) et acceptation (`accept_one_invoice` étape 6,
   `invoice_payment_anchor_date`, `InvoiceDateWindow::check`) lisent-elles **la même** ancre, par les mêmes macros
   (`invoice_payment_anchor_sql!`, `invoice_payment_anchor_join_sql!`) ? Toute voie qui ferait proposer ce qui sera
   refusé (ou l'inverse) est au moins **HIGH**.
2. **Manuel contre code (obligatoire)** : `docs/manual/fr/user-manual.tex` — § *Ce que réclame un rappel* (vers
   l. 1305) et son `keshwarning`, § *Algorithme de matching* (`\label{sec:algorithme-matching}`, neuf), §
   `sec:facture-posterieure`, FAQ « aucun match automatique » (vers l. 2902) — **et le PDF**
   `docs/manual/fr/user-manual.pdf`, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`
   (le PDF porte-t-il les phrases neuves, plus aucune des six formulations remplacées listées à AC8, aucun « ?? » de
   renvoi cassé ?). Même exigence pour `docs/api-external.md` (paragraphe **Dates** neuf du § *Accepter des
   propositions de rapprochement*) et `CHANGELOG.md` (`## [0.13.4] — Non publié`). Une promesse que le code ne tient
   pas — ou une restriction qu'il n'a pas — est au moins **MEDIUM**. Le geste « rejeter la transaction » est-il dit
   praticable alors qu'il ne l'est pas encore (C-29-2-8, annotation) ?
3. **Propagation** : greper les **valeurs** (`earliest_invoice_date`, `30 jours`, `window_days`, `#529`,
   `check(`) sur `crates`, `frontend/src`, `docs/`, `CHANGELOG.md` : reste-t-il une phrase ou un doc-comment qui
   décrit l'ancienne borne (`F ≥ D − 30`) comme règle actuelle ?
4. **Le compte rendu** (Dev Agent Record, Change Log, registre) : chaque décompte et chaque chiffre de gate se
   recompte-t-il depuis la source et les journaux (`/home/gcorbaz/devel/kesh-gate-logs/292-*.log`) ? Les mutations
   déclarées tuées le sont-elles par les tests nommés ? Les équivalences déclarées (M12, M13) sont-elles démontrées ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff comme un relecteur qui ne connaît pas l'intention : SQL (`GREATEST`/`NULL` en MariaDB, `CAST … AS DATE`,
`DATE()` sur `DATETIME(6)`, table dérivée et `GROUP BY`, ordre des `bind` contre les `?`, alias `rmd` libre dans la
requête de proposition), types et décodage (`query_scalar::<_, NaiveDate>`), Pattern batch des deux bras neufs
(`Ok(None)` → `INVOICE_NOT_FOUND`, `Err` → `DATABASE_ERROR`, jamais `?` vers un `AppError`), doc-comments justes,
tests qui prouvent ce qu'ils disent (une assertion qui passerait aussi sans le code ?), code mort, nommage.

## Lentille E — Edge cases

Parcours chaque branche et chaque borne : facture sans échéance, échéance antérieure à la date, rappels multiples,
tous annulés, rappel le jour même du paiement, rappel futur, rappel à 23 h 59 UTC (`DATE(sent_at)`), facture non
échue à échéance lointaine (2099), bornes `D − 30`/`D − 31`/`D + 1`/`D + 2` sur `F` et sur `A`, transaction sans
date de valeur, facture d'une autre société, `LIMIT 50` et ordre `(date, id)` avec des factures non échues avancées
dans la fenêtre, course « rappel annulé entre proposition et acceptation » (verrous, vue de transaction : la lecture
ajoutée est-elle bien après la première lecture ordinaire et sans `FOR UPDATE` ?). Un cas non couvert par un test et
dont le comportement est faux est au moins **MEDIUM**.

## Lentille A — Acceptance (la fiche contre le livré)

Chaque AC (1 à 10) et chaque tâche (T1 à T7) : livré, testé par un test **qui rougirait**, déclaré honnêtement au
Dev Agent Record ? En particulier : AC5 (cas a à k, `Vec` ordonné avec répétitions, scalaire pour chaque cas,
inter-factures, inter-sociétés), AC6 (parité a à k avec raisons, cas de #529 avec `paid_at` = `D` à minuit, doc-comment
du test de la 28-7 réécrit sans toucher aux assertions), AC7 (inventaire (c) recopié), AC8 (textes prescrits, six
formulations à 0 dans le PDF), AC9 (M1 à M15, M10a/b), AC10 (gates réellement exécutés, E2E au dernier commit de code,
échecs jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*). Les écarts à la fiche
(C-29-2-15 : ancre en deux termes par macro ; C-29-2-16 : label neuf) sont-ils justifiés et sans effet sur un AC ?

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-2-review-p1-<B|E|A>.md` (selon ta lentille) :

- findings numérotés `B1…` / `E1…` / `A1…`, chacun avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), fichier:ligne,
  preuve (**sortie `grep -nF` ou `sed -n` copiée** pour toute affirmation de présence ou d'absence), correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** — un « 0 finding » sans cette liste ne
  compte pas ;
- rien d'autre n'est écrit nulle part.
