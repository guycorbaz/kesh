# Prompt — validation P1 de la spec, Story 28-6 (le rapprochement refuse un débit contre une facture)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-28-6`, branche `story/28-6` (base `5aad8b35`, kickoff de l'Epic 28).
Fiche : `_bmad-output/implementation-artifacts/28-6-rapprochement-debit-facture.md` (commit `3563944d`).
Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md`, § *28-6*. Issue #482
(`gh api repos/guycorbaz/kesh/issues/482` et `…/482/comments`). Registre :
`_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` (C-28-6-1 à 10). Consignes :
`docs/agents-orchestration.md`. Règles : `CLAUDE.md` (§ *Pattern batch — FailedProposal per-proposal* ; § *Review
Iteration Rule* et ses sous-sections — le manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation
post-patch, le recompte des comptes rendus ; § *Règle de splitting préventif*).

⚠️ La fiche **corrige la prémisse de l'issue** (C-28-6-1) : selon elle, rien n'est écrit aujourd'hui — la contrainte
`chk_jel_debit_credit_exclusive` refuse la ligne négative et le refus sort en `DATABASE_ERROR`. Vérifie la chaîne
toi-même, maillon par maillon (`settlement_journal_lines`, migration `20260412000001_journal_entries.sql`,
`map_db_error`, `project_error_to_failed_proposal`), et cherche le chemin qui l'invaliderait (une ligne positive
malgré un montant négatif, l'écart d'arrondi, un `INSERT` antérieur à celui des lignes qui ne serait pas annulé).

## Axes prioritaires (communs aux deux lentilles)

1. **Inventaire clos** (Dev Notes § *Inventaire clos* et § *sites NON RÉSOLUS*) : refais les `grep` copiés dans la
   fiche, et **élargis-les** (un détecteur part du symptôme, pas d'une liste) — tout écrivain de
   `bank_transactions.matched_entry_id`, de `invoice_settlements`, de `letterings` qui touche une facture à partir
   d'une transaction ; `kesh-seed`, `test_endpoints.rs`, `admin_backup`, `kesh-payment`, `payment_batches`,
   `supplier_invoices`, `credit_notes`, `imported_supplier_invoices`, le frontend (`ReconciliationProposals.svelte`
   et tout composant qui poste vers `/reconciliation/accept`). Un chemin qui lie une transaction de débit à une
   facture et que la fiche omet est au moins **HIGH**. Le symétrique (crédit ↔ facture fournisseur) et les avoirs :
   l'absence affirmée (C-28-6-9) est-elle vraie ?
2. **Périmètre minimal** : la fiche ferme-t-elle #482 **et rien de plus** ? Chaque AC, tâche et fichier est-il
   nécessaire à la fermeture ou à un invariant existant (C-28-6-6 justifie le libellé frontend malgré un refus non
   atteignable depuis l'écran — tiens-tu l'argument ?) ? À l'inverse, une exclusion de § *Exclusions écrites*
   laisse-t-elle #482 ouverte (par exemple, la garde absente du pré-vol) ? Le décompte de **4 modules** est-il juste ?
3. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex`, § *Réconciliation bancaire*
   (l. 1909 et suivantes, dont l. 1919 « créditrice, en CHF », l. 1946, l. 1952 *Un refus n'empêche pas les
   autres*) et `admin-manual.tex` (l. 1994). L'AC9 affirme qu'aucun manuel ne doit changer : est-ce vrai, avant et
   après la story ? Contrôle aussi le **PDF**, aplati : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '` (idem `admin-manual.pdf`). Une promesse que le code ne tiendrait pas est au moins MEDIUM.
4. **Recomptes** : chaque nombre de la fiche se recompte depuis la source — 27 codes littéraux distincts
   (`grep -ohE 'error_code: "[A-Z_]+"' crates/kesh-api/src/routes/reconciliation.rs | sort -u | wc -l`), « 29 codes »
   de `failed-proposal-label.test.ts`, `sitesTotal` 2075 et les 34 `i18nMsg(` de `failed-proposal-label.ts`, les
   numéros de ligne cités, le nombre d'AC (9), de mutations (7), de choix (10).

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque phrase de l'issue #482 et du § *28-6* de l'epic couverte par un AC testable (dont « adapter
  `post_accept_filters_signed_amount` », « débit à référence reconnue », « montant nul », « vérifier les autres
  `accept_one_*` ») ; chaque AC par une tâche, chaque AC par un test **qui rougirait** ; chaque mutation M1-M7 tuée par
  le test nommé (raisonne : le montage du test atteint-il la garde ? `seed_validated_invoice` pose-t-il bien une
  écriture de vente à ligne de débit, sans quoi `INVOICE_SALE_ENTRY_MALFORMED` répondrait avant ?) ; chaque fait,
  chemin et numéro de ligne cité **revérifié** (`sed -n`, `grep -nF`) ; les textes prescrits (quatre locales,
  `docs/api-external.md`, CHANGELOG) confrontés à ce que le code fera ; les gardes existantes qui changeront de sens
  (`failed-proposal-label.test.ts`, `i18n-keys.test.ts`, `lint-i18n-ownership`, `textes_coherents.rs`, tout test qui
  asserte `RECONCILIATION_SCORE_TOO_LOW` ou `DATABASE_ERROR` pour un débit) — la fiche les nomme-t-elle toutes ?
- **F — Faisabilité et conception (adversaire)** : la place de la garde (après le statut, avant la facture, sur la
  transaction rechargée sous verrou) est-elle la bonne, et l'ordre des refus qui en résulte est-il cohérent avec le
  § *Pattern batch* et avec `docs/api-external.md:536-552` ? La comparaison `<= Decimal::ZERO` classe-t-elle
  correctement `-0.00`, et la valeur peut-elle seulement atteindre la base (`DECIMAL(18,2)`) ? Le code et le
  `details` choisis (C-28-6-2) tiennent-ils contre les codes voisins (`VALIDATION_ERROR` +
  `zero_amount_transaction` des autres variantes) ? Le refus d'un **remboursement client** par débit, d'un **avoir**,
  d'une **extourne** est-il un cas légitime que la garde casserait ? Une transaction de **crédit** peut-elle, par un
  chemin, régler une facture **fournisseur** ? Les régions déclarées communes avec la 28-5 et la 28-7 (§ *Fichiers
  communs*) sont-elles justes au code (`post_reject`, `reject_batch`, étape 6 d'`accept_one_invoice`) ?

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence =
sortie de `grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne
compte pas comme passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/28-6-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-6-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport,
le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh` et les
scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`pull`/
`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`, **tout interpréteur** (`python3`, `python`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrit), `docker`, **`mariadb` / `mysql` et toute requête SQL**, `gh` en
écriture. Rien de `~/kesh-test/` (relevé réel de Guy) ne se lit ni ne se cite ; aucune donnée réelle (IBAN, solde,
ligne) dans le rapport. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`/`blame`, `gh api` en
lecture, `pdftotext` vers la sortie standard.
