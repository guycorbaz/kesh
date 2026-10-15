# Prompt — validation P2 de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. Passe **complète**, deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6,
rétro 25) : P1 Sonnet → **P2 Opus**, pour les deux lentilles.*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1` (base `15686d55`). Fiche :
`_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` (HEAD de la branche) — **une fiche,
deux sous-stories** (30-1a puis 30-1b). Registre : `epic-30-choix-autonomes-301.md` (C-30-1-1 à 29 ; amendés : 11, 12,
14, 16, 20, 27). Registre de l'epic : `epic-30-choix-autonomes.md`. Epic : `_bmad-output/planning-artifacts/epic-30-
factures-fournisseur.md` § *30-1*. Issues, en lecture : `gh api repos/guycorbaz/kesh/issues/686`, `…/671`, `…/687`,
`…/482`. Rapports de la P1 (lecture) : `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p1-F.md`, `…-R.md` ; la
remédiation est l'entrée « Validation P1 — 2026-10-15 » du Change Log de la fiche.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ;
§ *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous **et** naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axe obligatoire, commun aux deux lentilles — relire ce que la remédiation P1 a écrit

La sévérité se déplace vers ce qu'on vient d'écrire. Relis **chaque** passage réécrit en P1 (`git diff 4862053a HEAD
-- _bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md`) et cherche le défaut qu'il
introduit :
- la **valeur absolue** de l'AC-b1 : tout lecteur du montant (score, masquage, affichage, audit, `refuse_wrong_direction`)
  la prend-il là où il faut, et **pas** là où il ne faut pas (un crédit ne doit jamais être comparé à une facture
  fournisseur) ? Le score client reste-t-il au bit près ?
- le **critère de masquage** (AC-b5, AC-b3) : sur-blocage (un débit légitime privé de sa règle sans raison visible) ou
  sous-blocage (un double comptage muet resté) ? La `notice` est-elle déterministe quand plusieurs factures sont
  masquantes ? La requête masquante, **sans borne de dates**, a-t-elle un `LIMIT` qui écarte la facture qui devait
  bloquer ?
- l'**ordre des verrous** facture → compte → articles (AC-b6 étape 3, AC-b10) : la table est-elle exacte au code
  (`pay_in_tx`, `create_batch`, `confirm_batch`, `pay`, `close_in_tx`, `fiscal_years::close`, `reconciliation_cancel`) ?
  Un cycle manque-t-il (lot mixte, `cancel_batch`, `supplier_invoices::cancel_in_tx`, `settle_invoice` client sur le
  même compte) ? Le rejeu de `create_batch` (AC-a8) est-il sûr (la route est-elle idempotente au rejeu, écrit-elle hors
  transaction — fichier, `msg_id` —, peut-elle produire deux lots) ?
- la **traduction commune des refus de dates** (AC-b2) et `details.tolerance_days` : contrat client réellement additif ?
- le **texte commun « rapproché »** (AC-a6) : vrai sur la fiche client, la fiche fournisseur, le dé-rapprochement, et
  pour un client d'API ? Les quatre locales ?
- la **référence fournisseur** (AC-b1, C-30-1-12) : l'énoncé « aucun faux indice ne départage contre l'ordre
  d'imputation » est-il vrai (cas d'un numéro de 6 caractères égal à une référence non structurée, `Ustrd` reporté en
  `reference` ?) ;
- l'**inventaire des refus** (tableau de l'AC-b6) : l'argument « à l'étape 6, `NotFound` ne vient plus que du projet »
  tient-il au code ?

Un défaut introduit par la remédiation est au moins **MEDIUM** ; un double comptage muet ou un cycle non rejoué est
**HIGH**.

## Axes prioritaires (repris de la P1, sur l'état actuel)

1. **Prémisses** (§ *Ce qui existe*, § *Prémisses de #686*) : revérifie au code ; une prémisse fausse est au moins
   **MEDIUM**.
2. **Découpage** (C-30-1-3, C-30-1-27) : 30-1a puis 30-1b ; modules recomptés (5 et 5) — le décompte tient-il au code
   prescrit ? L'AC-a8 appartient-il à la 30-1a sans en brouiller l'objet ?
3. **Aucun faux rattachement muet** (règle du `CLAUDE.md`, #482, leçon de la 29-6) : par un chemin quelconque
   (proposition, acceptation directe, lot mixte), un débit peut-il être rapproché d'une facture client, un crédit d'une
   facture fournisseur, une règle porter une charge déjà comptée ?
4. **Verrous et naissance de la vue (B5)** : AC-a4, AC-b8, AC-b10 en profondeur.
5. **DRY** : un jumeau prescrit, ou une généralisation qui change un comportement client, est au moins **MEDIUM**.
6. **La fenêtre** (C-30-1-13) et `paid_at` (AC-b16) : cohérents avec ce que Kesh écrit et affiche ?
7. **Inventaire clos des refus** et codes de `failed-proposal-label.ts` (décompte du doc-comment **et** du test).
8. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1586-1640, 2111-2160, 2252 et suivantes ;
   ce que le manuel dit aujourd'hui est-il vrai au code, et ce que l'AC-b13 prescrit le sera-t-il (dont `paidAt`, la
   `notice`, le masquage) ? Contrôle aussi le **PDF**, aplati vers la **sortie standard** seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (⛔ jamais vers un fichier). Même exigence pour
   `docs/api-external.md` (`:366`, `:572-612`, § lots), le CHANGELOG `## [0.14.0] — Non publié` et le README.
9. **Recomptes** : AC (24 = 8 + 16), tâches (24, T0 à T23), tests (52 = T-a1 à a13 + T-b1 à b39, dont 7 Vitest),
   mutations (29 = 6 + 23, une équivalente déclarée), choix (29), modules (5 et 5), migrations (81) — et chaque numéro de
   ligne cité.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #686 et du § *30-1* de l'epic couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les registres
  (`bank_account_closed_inventory.rs`, `letterings_lexical.rs`, `textes_coherents.rs`, `audit_labels.rs`,
  `audit_label_registry.rs`, `reconciliation_e2e.rs` pour l'IDOR et le rôle, `failed-proposal-label.ts` et son test,
  `lint-i18n-ownership`) ; la faisabilité de l'E2E de parcours (AC-b12) et des Vitest T-b38, T-b39.
- **F — Faisabilité et exactitude de la conception (adversaire)** : axe obligatoire et axes 3, 4, 6 en profondeur ; les
  requêtes de l'AC-b3 (candidates et masquantes, partage du texte SQL, `LIMIT`, index) ; les courses T-a6, T-a11, T-b22,
  T-b24, T-b25, T-b34 avec les sondes existantes (**jamais** `INNODB_TRX`) — chacune peut-elle être rouge au sol ? ; les
  mutations M-a1 à M-b23 (chacune **tuable** par le test nommé ; M-a2 équivalente à bon droit ?) ; l'AC-a8 (rejeu de
  `create_batch`) ; l'absence de migration (AC-b15).

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p2-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` — ni sortie de `pdftotext`, ni
note, ni copie. Aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`checkout` **sous aucune forme** (`--detach` compris)/`switch`/`restore`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`/`worktree`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`,
`perl`, `ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb`/`mysql`** et toute requête SQL, `gh`
en écriture. Aucune donnée réelle de Guy ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`,
`git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
