# Prompt — validation P1 de la spec, Story 29-5 (à score égal, la facture échue la première passe en tête)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-5`, branche `story/29-5` (base `fcd2ae75`, `main` après le kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-5-departage-exigibilite.md` (commit `92a8e51f`).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-5* et § *Ordre*. Issue #668 (l'objet) **et
son commentaire de rectification** : `gh api repos/guycorbaz/kesh/issues/668` et
`gh api repos/guycorbaz/kesh/issues/668/comments`. Voisines **non mergées**, en lecture seule, pour les zones
partagées : `/home/gcorbaz/devel/kesh-29-2/_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` et son
registre `epic-29-choix-autonomes-29-2.md` (C-29-2-7, C-29-2-11, C-29-2-13 : date d'ancrage, fenêtre élargie,
troncature renvoyée à la 29-5) ; `/home/gcorbaz/devel/kesh-29-3/_bmad-output/implementation-artifacts/29-3-meilleure-proposition.md`
(`order_candidates`, AC8 : phrase d'ordre dans `api-external.md` ; C-29-3-12). Voisine mergée : #548 (Story 28-7,
C-28-7-10, l'`ORDER BY … LIMIT 50`).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-5.md` (C-29-5-1 à 10).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § *Un
appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md` ; rétrospective
`epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Les prémisses, établies au code*) : départage `invoice_id ASC` (`matching.rs:97-105`) ;
   désignation qui prime par le score ; rôle de l'`ORDER BY … LIMIT 50` ; données disponibles au départage
   (`Invoice.due_date`) ; « premières poursuites = rappels » déclaré **faux** ; attendu d'origine écarté. Revérifie
   chacune au code de `fcd2ae75`. Une prémisse fausse ou ignorée est au moins **MEDIUM**.
2. **Le critère légal** : la fiche cite l'art. 87 CO **en substance**, sans l'avoir relu à la source. Si tu peux lire
   le texte officiel (RS 220) en lecture seule — `WebFetch`/`WebSearch` si disponibles, sinon une source juridique
   citable —, confronte-le aux quatre points de § *Le critère légal* et à la clé retenue ; dis-le si tu ne le peux
   pas. Examine aussi l'art. 75 (facture sans échéance exigible dès sa date) et la distinction rappel / poursuite
   (C-29-5-3). Une règle retenue contraire au texte est **HIGH**.
3. **L'équivalence « échues d'abord ⇔ `X` croissante »** (C-29-5-2) : est-elle vraie pour toute date de paiement `D`,
   bornes comprises (`X = D`, `X = D + 1`, échéance antérieure à la date, échéance nulle) ? La clé `(X, F, id)` donne-t-
   elle un ordre **total** et déterministe ? Un contre-exemple est **HIGH**.
4. **La troncature** (AC3, C-29-5-4, complément de l'orchestrateur) : avec la fenêtre de la 29-2 (`F` jusqu'à
   `D − 60` à 30 jours net, non échues à échéance lointaine), l'`ORDER BY` aligné garantit-il que, dans chaque classe
   de score, les candidates gardées forment un préfixe de l'ordre de `propose_matches` ? Ce qui reste non résolu
   (troncature aveugle au score, C-28-7-10) est-il honnêtement écrit ? Un cas où le `LIMIT 50` écarterait encore la
   facture que l'art. 87 désigne parmi des scores égaux est **HIGH**.
5. **Périmètre minimal et fermeture de #668** (issue rectifiée) : chaque AC est-il nécessaire ; l'attendu révisé (1,
   2, 3 du commentaire) est-il couvert par un test qui rougirait au sol ? La composition de l'ancre de la 29-2 (AC1,
   C-29-5-9) est-elle mécanique et sûre, ou un empiètement ? Recompte les modules (quatre).
6. **Inventaire clos des sites** (Dev Notes § *Inventaire (a)*, *(b)*, *(c)*) : refais les `grep` et **élargis-les** —
   un détecteur part du symptôme : tout code qui trie ou choisit des factures pour un paiement (`sort_by`, `min_by`,
   `ORDER BY` sur `invoices`, `.first()` sur une liste de factures), dans `crates/*/src`, `kesh-seed`, le frontend
   (`frontend/src/lib/features/reconciliation/`). Pour (c) : tout test qui dépend de l'ordre de deux factures de même
   score, ou de l'ordre rendu par `find_unpaid_invoices_for_window`. Un site ou un test ignoré dont le sens change
   est au moins **HIGH**.
7. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Algorithme de matching* (l. 2027-2052)
   — ce qu'il dit **aujourd'hui** sur l'ordre des candidates est-il vrai au code de `fcd2ae75` ? Ce que l'AC7 prescrit
   sera-t-il vrai au code prescrit (désignation qui prime, montant exact devant, clients différents, recours par la
   fiche puis *Rejeter*) ? Contrôle aussi le **PDF**, aplati et ligatures normalisées :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g'`. Même
   exigence pour la phrase de `docs/api-external.md` et le texte de CHANGELOG prescrits. Une promesse que le code ne
   tiendrait pas est au moins **MEDIUM**.
8. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (9), tâches
   (T0-T6), mutations (M1-M11), choix (10), modules (4), lignes d'exemple (5), cas d'AC5 (7 neufs).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #668, de sa rectification et du § *29-5* de l'epic couverte par un AC testable, chaque AC par une
  tâche et par un test **qui rougirait** ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`,
  `grep -nF`) ; les zones partagées avec la 29-2 et la 29-3 (§ *Zones partagées*) confrontées à leurs fiches —
  conflit non vu, ou phrase de l'autre fiche que la 29-5 rend fausse sans le dire ; les gardes existantes qui
  pourraient changer de sens (`textes_coherents.rs`, `test_schema_guard.rs`, les deux tests d'ordre de
  `reconciliation_repository.rs`, `propose_matches_returns_sorted_desc`, le smoke de performance).
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : l'expression
  `CAST(GREATEST(i.date, COALESCE(i.due_date, i.date)) AS DATE)` est-elle juste en MariaDB 10.11 dans un `ORDER BY`
  à côté du `HAVING amount_due` et des jointures du reste dû (et de la jointure `rmd` de la 29-2) ? Le tri `NULL`
  (M9) est-il bien celui qu'annonce la fiche ? La fonction Rust (`max`, `unwrap_or`) et le piège `Option` (M3)
  sont-ils exacts ? `kesh-reconciliation` peut-il appeler `kesh_db::repositories::reconciliation` sans cycle ni
  visibilité manquante ? Le tri de `propose_matches` peut-il garder la facture à côté de la proposition sans changer
  la signature ? Les tableaux (exemples de § *La règle retenue*, cas de l'AC5, montage de l'AC4 à 51 factures, cas
  HTTP de l'AC6 « dans la fenêtre avec ou sans la 29-2 ») sont-ils **exacts** ligne par ligne et réalisables ? Les
  mutations M1-M11 sont-elles toutes **tuables** par les tests nommés (en particulier M6, M8, M9), et en manque-t-il
  une évidente ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-5-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris)/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`,
**tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`,
**`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite. Les worktrees `kesh-29-2` et `kesh-29-3` se lisent, ne se modifient pas. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard, lecture web.
