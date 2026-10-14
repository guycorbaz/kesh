# Prompt — validation P1 de la spec, Story 29-2 (un paiement reçu après l'échéance, ou après un rappel, est proposé)

*Versionné le 2026-10-14. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-29-2`, branche `story/29-2` (base `16eeed21`, kickoff de l'Epic 29).
Fiche : `_bmad-output/implementation-artifacts/29-2-paiement-tardif.md` (commit `45f157f7`).
Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-2* et § *Ordre*. Issue #529 (l'objet) :
`gh api repos/guycorbaz/kesh/issues/529` ; voisines #548 (Story 28-7, fiche `28-7-facture-posterieure-paiement.md`,
**à ne pas rouvrir** : proposition et acceptation gardent une seule borne), #482 (Story 28-6, refus d'un débit), #527
(29-3, tri des candidates, sérialisée après).
Registre : `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` (C-29-2-1 à 9) ; antécédents :
`epic-28-choix-autonomes.md` C-28-7-1 à 16.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern
batch ; § *Un appariement automatique propose, il ne crée jamais*) ; `docs/agents-orchestration.md` ; rétrospective
`epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 : une modification qui touche une transaction se
relit sur l'ordre des verrous **et** la naissance de la vue).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** : la fiche (§ *Les prémisses, établies au code*) déclare périmées deux prémisses de #529 et de la
   consigne (fenêtre « centrée » ; manuel qui « promet »). Revérifie-les au code et au manuel de `16eeed21`. Une
   prémisse déclarée périmée qui ne l'est pas, ou une prémisse vraie que la fiche ignore, est au moins **MEDIUM**.
2. **Périmètre minimal et fermeture du défaut** : la règle `A = max(F, E, R)`, `F ≤ D + 1` et `A ≥ D − 30` ferme-t-elle
   #529 tel qu'il est écrit (« la fenêtre tient compte de l'échéance (et des rappels), et/ou l'écran permet de
   choisir… ») ? Le reliquat (C-29-2-9) est-il honnêtement écrit, et son contournement (C-29-2-8, rejeter la
   transaction après un règlement saisi depuis la fiche) est-il **juste** au code (`invoice_settlements_write.rs`,
   flux de rejet) ? Chaque AC est-il nécessaire ? Un ajout non nécessaire est au moins **MEDIUM** ; une exclusion
   qui laisse #529 ouvert, **HIGH**. Recompte les modules (§ *Modules touchés* : trois).
3. **Une seule borne** : la proposition et l'acceptation lisent-elles vraiment la **même** définition de l'ancre
   (C-29-2-4) ? Une voie par laquelle elles divergeraient (filtre SQL vs `check` Rust, `LIMIT 50`, scalaire scopée
   autrement, rappel inséré ou annulé entre les deux) est au moins **HIGH** si elle fait proposer ce qui sera
   refusé (#548).
4. **Faux rattachement** (§ *Un appariement automatique…* du `CLAUDE.md`) : la fiche affirme que supprimer la borne
   basse ferait passer la plus ancienne facture en tête à score égal, et que l'ancre borne ce risque. Vérifie
   `propose_matches` et l'écran (`ReconciliationProposals.svelte`, `candidates[0]`, *Accepter (N)*). Le risque
   résiduel transmis à la 29-3 (C-29-2-7) est-il juste et suffisant, ou la 29-2 l'aggrave-t-elle au point de devoir
   le traiter elle-même ?
5. **Inventaire clos des sites** (Dev Notes § *Inventaire (a)*, *(b)*, *(c)*) : refais les `grep` copiés et
   **élargis-les** — un détecteur part du symptôme : toute requête qui filtre `invoices` par date en présence d'une
   transaction, tout code qui compare `invoice.date` ou `due_date` à une date de transaction, `kesh-reconciliation`,
   `kesh-seed`, `test_endpoints.rs`, `seed-dev-db.sql`, le frontend. Pour (c) : tout test (Rust ou Playwright) dont
   une facture **porte une échéance ou un rappel** et qui lit les propositions ou accepte une facture — y compris par
   le pipeline de création réel (`POST /api/v1/invoices`, échéance par défaut = date + délai du contact). Un site ou
   un test que la fiche ignore et dont le sens change est au moins **HIGH**.
6. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1305 (rappels), 2029 (*Algorithme de
   matching*), 2031 (`\label{sec:facture-posterieure}`), 2902 (FAQ). Ce que le manuel dit **aujourd'hui** est-il vrai
   au code de `16eeed21` ? Ce que l'AC8 prescrit d'écrire sera-t-il vrai au code que la fiche prescrit (rappel
   annulé, échéance nulle, échéance antérieure à la date, reliquat) ? Contrôle aussi le **PDF**, aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`. Même exigence pour `docs/api-external.md`
   § *Accepter des propositions de rapprochement* et le texte de CHANGELOG prescrit. Une promesse que le code ne
   tiendrait pas est au moins **MEDIUM**.
7. **Recomptes** : chaque nombre et chaque numéro de ligne de la fiche se recompte depuis la source — AC (10), tâches
   (T1-T7), mutations (M1-M9), choix (9), modules (3), cas d'AC5 (a-i), sorties d'inventaire copiées.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #529 et du § *29-2* de l'epic (« la fenêtre `InvoiceDateWindow` […] un paiement à 30 jours net en
  retard n'est jamais proposé », « `kesh-db` touché : gate complet même en boucle », « ne pas rouvrir #548 »)
  couverte par un AC testable, chaque AC par une tâche, chaque AC par un test **qui rougirait** ; chaque fait,
  chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (manuel, `api-external.md`,
  CHANGELOG) confrontés à ce que le code prescrit fera ; les gardes existantes qui pourraient changer de sens
  (`textes_coherents.rs`, `test_schema_guard.rs`, `i18n-keys.test.ts`, `failed-proposal-label.test.ts`, les tests
  `post_accept_rejects_payment_date_*` et les deux tests de la 28-7) — la fiche les nomme-t-elle correctement ? Le
  Pattern batch (`FailedProposal` par proposition, jamais d'`AppError` global) est-il tenu par l'AC4 sur **chaque**
  issue de la lecture ajoutée (`Ok(None)`, `Err`) ?
- **F — Faisabilité et exactitude de la règle (adversaire de la conception)** : l'expression
  `GREATEST(i.date, COALESCE(i.due_date, i.date), COALESCE(DATE(rmd.last_sent_at), i.date))` est-elle juste en
  MariaDB 10.11 (sémantique de `GREATEST` avec `NULL`, types `DATE` / `DATETIME(6)`, comparaison au paramètre lié
  `NaiveDate`) ? La table dérivée des rappels (`MAX(sent_at) … WHERE cancelled_at IS NULL GROUP BY invoice_id`)
  cohabite-t-elle avec les jointures du reste dû (`amount_due_derived_joins`, alias, `HAVING amount_due`) sans
  multiplier les lignes ni fausser les agrégats ? L'alias `rmd` est-il libre ? Les tableaux de la fiche (exemples de
  § *La règle retenue*, cas a-i d'AC5) sont-ils **exacts** cas par cas (recalcule chaque ligne, bornes incluses) et
  réalisables (exercice qui couvre `D − 90`, unicité des montants sous la tolérance de 0.05, colonnes `NOT NULL` de
  `invoice_reminders`, contrainte d'échéance) ? **B5** : la lecture ajoutée à `accept_one_invoice` déplace-t-elle la
  naissance de la vue ou ajoute-t-elle un verrou (identifie au code la première lecture ordinaire du chemin
  d'acceptation, pré-vol et `accept_batch` compris) ? L'ordre des refus (`check` : avant-facture puis trop tard ;
  étape 4bis du débit) est-il préservé ? Le changement de `WHERE` (`BETWEEN` → `i.date <= ?` + ancre) change-t-il
  l'usage de `idx_invoices_company_validated_unpaid_date` au point de compter ? Les mutations M1-M9 sont-elles toutes
  **tuables** par les tests nommés, et en manque-t-il une évidente (ex. scalaire non scopée société, `DATE()` retiré,
  borne haute calculée sur l'ancre au lieu de la date) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/29-2-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout` **sous aucune forme** (`--detach`
compris)/`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`,
**tout interpréteur** (`python3`, `python`, `perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`,
**`mariadb`/`mysql`** et toute requête SQL, `gh` en écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit
ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en lecture,
`pdftotext` vers la sortie standard.
