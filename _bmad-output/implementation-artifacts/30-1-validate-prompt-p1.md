# Prompt — validation P1 de la spec, Story 30-1 (la réconciliation règle les factures fournisseur)

*Versionné le 2026-10-15. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) : passes
complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux lentilles
(la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-30-1`, branche `story/30-1` (base `15686d55`, kickoff de l'Epic 30).
Fiche : `_bmad-output/implementation-artifacts/30-1-reconciliation-factures-fournisseur.md` (commit `53d8c593`) —
**une fiche, deux sous-stories** (30-1a puis 30-1b).
Registre : `_bmad-output/implementation-artifacts/epic-30-choix-autonomes-301.md` (C-30-1-1 à 24) ; registre de l'epic
`epic-30-choix-autonomes.md` (C-30-0-1 à 3). Epic : `_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md`
§ *30-1*. Issues, en lecture : `gh api repos/guycorbaz/kesh/issues/686`, `…/671`, `…/482`.
Fiches de référence (lecture seule) : `29-8-transaction-reglee-ailleurs.md`, `29-6-trop-percu-centimes.md`,
`29-3-meilleure-proposition.md`, `29-4-cloture-compte-concurrente.md`, `29-2-paiement-tardif.md`.
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON
RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Règle de splitting préventif ; § Pattern batch ;
§ *Un appariement automatique propose, il ne crée jamais* ; § Migration breaking policy) ;
`docs/agents-orchestration.md` ; rétrospective `epic-28-retro-2026-10-14.md` § 4 et § 7 (périmètre minimal ; B5 :
verrous **et** naissance de la vue ; § 4.3 : aucune dette assumée sans issue).

## Axes prioritaires (communs aux deux lentilles)

1. **Prémisses** (§ *Ce qui existe* et § *Prémisses de #686*) : revérifie chaque ligne au code de `15686d55` — en
   particulier le verdict **« fausse »** (le dé-rapprochement classerait un règlement fournisseur rapproché en `Entry`, et
   `reverse_in_tx` le refuserait : lis `journal_entries::reversal_blockers` et `reverse_in_tx`), l'immuabilité d'une
   facture fournisseur (aucune route de modification), le TTC possible à quatre décimales, `paid_at =
   CURRENT_TIMESTAMP`, la lecture non verrouillante de `in_generated_batch`. Une prémisse mal établie est au moins
   **MEDIUM** ; une prémisse qui fonde le découpage et se révèle fausse est **HIGH**.
2. **Le découpage (C-30-1-3)** : l'ordre 30-1a puis 30-1b tient-il ? La 30-1a est-elle testable sans la 30-1b (décor :
   `pay` puis lien posé en SQL par un helper de fixture) ? Un état produit par la 30-1b reste-t-il défaisable à tout
   commit de `main` ? Le nombre de modules par sous-story est-il sous le seuil ?
3. **Aucun faux rattachement muet** (règle du `CLAUDE.md`, #482, leçon de la 29-6 « retirer — ou ne pas ajouter — une
   candidate peut démasquer une autre proposition ») : un débit peut-il, par **un** chemin quelconque (proposition,
   acceptation directe par l'API, lot mixte), être rapproché d'une facture **client**, ou un crédit d'une facture
   fournisseur ? La règle d'affectation peut-elle porter en charge un débit dont la facture fournisseur, saisie et
   ouverte, compte déjà la charge (AC-b5 : facture candidate, facture « presque au montant » avec indice, facture au TTC
   à quatre décimales, facture hors fenêtre, facture en lot `generated`, facture déjà payée — L1) ? Chaque trou non
   écrit est au moins **HIGH**.
4. **Verrous et naissance de la vue (B5)** : l'ordre de l'AC-a4 (classement fournisseur verrouillant, coût de
   l'intervalle) et de l'AC-b10 ; la garde du lot de l'AC-b8 (le scénario de double paiement est-il réel ? la lecture
   verrouillante après le verrou de la facture le ferme-t-elle ? un `LOCK IN SHARE MODE` sans résultat pose-t-il un
   intervalle qui bloque `create_batch` d'autres factures — attente ou cycle ?). Relis les ordres réels de `pay_in_tx`,
   `create_batch`, `confirm_batch`, `supplier_invoices::cancel_in_tx` et `cancel_settlement_in_tx`,
   `reconciliation_cancel::cancel_in_tx`, `accept_one_invoice`, `bank_accounts::close_in_tx` : un cycle peut-il se
   fermer ? `pay_in_tx` prend `bank_accounts` en **`FOR UPDATE`** (exclusif) au milieu d'un lot d'acceptation qui en
   compte d'autres : la règle de la 29-4b (« la revérification verrouillante en dernière acquisition ») est-elle
   compatible ? Un cycle ou une vue mal née est au moins **HIGH**.
5. **DRY et réutilisation** : la généricité de l'AC-b1 (traits `PayableDocument`, `MatchableDocument`) laisse-t-elle le
   score client **au bit près** (l'égalité « 0,40 + 0,10 contre 0,50 » de la 29-5) ? `pay_in_tx` appelé et non réécrit ;
   `cancel_settlement_in_tx` appelé ; `refuse_wrong_direction` sans changement du contrat client ; une seule
   `InvoiceDateWindow`. Un jumeau prescrit, ou une généralisation qui change un comportement client, est au moins
   **MEDIUM**.
6. **La fenêtre (C-30-1-13)** : `F ≤ D + 7`, `max(F, échéance) ≥ D − 365`, jamais `created_at` — les bornes sont-elles
   cohérentes avec ce que Kesh **écrit** (règlement daté avant la facture : une contrainte, un lettrage, un rapport le
   refuse-t-il ? cf. `chk_invoices_paid_at_after_date` côté client — existe-t-il un équivalent fournisseur ?), et avec
   l'usage décrit (facture saisie après le paiement) ?
7. **Inventaire clos des refus** (tableau de l'AC-b6) : refais l'inventaire des `DbError` atteignables par
   `pay_in_tx`, `journal_entries::create_in_tx` (période verrouillée, exercice postérieur clos, projet…),
   `letterings::sync_supplier_invoice_in_tx` et la fonction neuve de l'étape 3 ; une variante atteignable non classée,
   ou classée sous un code trompeur, est au moins **MEDIUM**. Les codes neufs (C-30-1-18) entrent-ils dans
   `failed-proposal-label.ts` et son décompte ?
8. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` l. 1586-1640 (*Régler une facture
   fournisseur*), 2111-2160 (*Réconciliation*, *Algorithme de matching*, *Accepter ou rejeter*), 2252 et suivantes
   (*Annuler un rapprochement*) ; ce que le manuel dit **aujourd'hui** est-il vrai au code de `15686d55`, et ce que
   l'AC-b13 prescrit le sera-t-il ? Contrôle aussi le **PDF**, aplati vers la **sortie standard** seulement :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (⛔ jamais vers un fichier). Même exigence pour
   `docs/api-external.md` (`:572-612`), le CHANGELOG `## [0.14.0] — Non publié` et le README prescrits. Une promesse que
   le code ne tiendrait pas est au moins **MEDIUM**.
9. **Recomptes** : chaque nombre et numéro de ligne de la fiche se recompte depuis la source — AC (22 = 7 + 15), tâches
   (21, T0 à T20), tests (38 = T-a1 à a10 + T-b1 à b28, dont 5 Vitest), mutations (20 = 6 + 14), choix (24), modules par
   sous-story, migrations (81).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de #686 et du § *30-1* de l'epic couverte par un AC testable, chaque AC par une tâche et par un test
  **qui rougirait** au sol ; chaque fait, chemin et numéro de ligne **revérifié** (`sed -n`, `grep -nF`) ; les
  registres que les écrivains neufs doivent rejoindre (`bank_account_closed_inventory.rs` — `REGISTRE`,
  `APPELANTS_REGLEMENT`, `FENETRE` —, `audit_route_registry.rs`, `audit_labels.rs`, `audit_label_registry.rs`,
  `textes_coherents.rs`, `lint-i18n-ownership`, `rbac_e2e.rs`, `idor_multi_tenant_e2e.rs`, la liste des codes de
  `failed-proposal-label.ts`) — la fiche les nomme-t-elle tous ? Les codes réutilisés (`RECONCILIATION_INVOICE_NOT_
  ELIGIBLE`, `BANK_ACCOUNT_CLOSED`, `BANK_ACCOUNT_NOT_CONFIGURED`, `FISCAL_YEAR_INVALID`, `PERIOD_LOCKED`…) ont-ils
  bien ces formes de `details` aujourd'hui ? L'E2E de parcours (AC-b12) est-il **faisable** avec le harnais (fixture CAMT
  réécrite en `DBIT`, helper de création d'une facture fournisseur, détail de l'import, dialogue d'annulation) et
  suffit-il à porter la recette que Guy ne fera qu'en fin d'epic ?
- **F — Faisabilité et exactitude de la conception (adversaire)** : axes 3, 4 et 6 en profondeur ; la requête de
  l'AC-b3 (égalité `DECIMAL(19,4)` contre `DECIMAL(18,2)`, `NOT EXISTS` du lot, `GREATEST`/`COALESCE`, ordre avant
  `LIMIT 50`, index disponibles) ; la généricité de `propose_matches` et d'`imputation_order` (le crate
  `kesh-reconciliation` dépend de `kesh-db` : les traits sont-ils placés sans cycle ?) ; le changement de signature de
  `pay_in_tx` (tous les appelants, tests compris) ; la faisabilité des courses T-a6, T-b22, T-b24, T-b25 avec les sondes
  existantes (**jamais** `INNODB_TRX`) ; les mutations M-a1 à M-b14 (chacune **tuable** par le test nommé ? M-a2
  équivalente ? en manque-t-il une — la normalisation des majuscules, le `status = 'paid'` du classement, le compte de
  l'import imposé à `BankTransfer`) ; l'absence de migration (AC-b15) est-elle vraie ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte
pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/301-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/301-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
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
