# Prompt — validation P2, Story 31-5 (quatre défauts courts)

*Versionné le 2026-10-24. Passe **complète** (non ciblée : la remédiation P1 a changé la conception de #707 et les
critères de contrôle). **Deux lentilles, R et F**, chacune en contexte frais, **Opus**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-5`, branche `story/31-5`. Fiche :
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-17). Règles : `CLAUDE.md`
(§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —,
*Règle de splitting préventif*, *Exception kesh-db*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les
sites NON RÉSOLUS*, *Propagation post-patch*, *Priorités des défauts*), `docs/agents-orchestration.md` § 3 et § 6. La
fiche traite #707, #589, #504 et #469 et écarte #687 et #645 : lis chacune (`gh issue view <N>`, lecture seule).

**Ce que la P1 a changé** (Change Log de la fiche, entrée « Validation P1 » ; diff : `git show b73414c8`) : contrôles de
l'AC11/AC16 réécrits (exception `supplier_invoices.rs:512`, contrôle multi-ligne d'`audit_log.rs`) ; site de validation
de #707 (`invoices.rs:1877-1881`, `:2364`, `check_written`) ; **#707 refondu en deux fonctions**
(`lock_vat_role_accounts_in_tx` avant `insert_with_defaults_in_tx`, `designate_vat_accounts_in_tx` après) pour tenir
l'ordre `accounts → company_invoice_settings` ; budget d'issues recompté (141 → 136/137, déficit soumis au Project
Lead) ; AC6 clos sur un critère ; G9 étendue aux six clés du journal d'audit ; `onboarding_path_b_e2e.rs` ; `### Corrigé`
au CHANGELOG. Les rapports P1 sont `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p1-{R,F}.md` : ne les prends pas
pour argent comptant — revérifie au code ce qu'ils affirment, et cherche **en priorité ce que la remédiation P1 a
introduit**.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **#707.** `insert_with_defaults_in_tx` n'écrit-il vraiment aucun compte de TVA ? `final_body` est-il le seul appelant
   de démonstration ? Le chemin du refus (`invoices.rs:1877-1881`, `:2364`, `check_written` `:2372` →
   `company_invoice_settings.rs:954-974`) est-il exact ? Les rôles `VatPayable`/`VatRecoverable`/`VatSettlement`
   existent-ils dans le plan **PME**, avec les types que l'AC1 exige, et sont-ils des rôles singleton (une seule ligne
   par société) ? **La nouvelle conception** : l'ordre `onboarding_state → companies → accounts (TVA, puis rôles) →
   company_invoice_settings → vat_rates → audit_log` est-il bien celui qu'elle produit, et est-il sûr face à `finalize`,
   à `reset_demo` et à une modification ou un archivage concurrent d'un compte (Pattern 5,
   `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) ? Verrouiller trois comptes d'une requête `IN (…) ORDER BY id FOR UPDATE`
   change-t-il les verrous pris (verrous d'écart, index utilisé — compare avec `FORCE INDEX (PRIMARY)` de
   `company_invoice_settings.rs:905` et sa justification) ? La condition `invoice_settings_created` et le `COALESCE`
   tiennent-ils (reset, rejeu de `final_attempt`) ? La ligne `seed_demo` du Pattern 5 et le doc de `final_body`
   (`crates/kesh-seed/src/lib.rs:362-367`) sont-ils bien prescrits à jour ?
2. **#589.** La garde serveur `reopen` (`crates/kesh-db/src/repositories/fiscal_years.rs`) refuse-t-elle tout exercice
   suivi d'un clos ? L'inventaire **clos** de l'AC6 (critère `grep -nE "rouvr|Rouvr|réouv|Réouv"` sur le catalogue
   `fr-CH`, 30 lignes, tri a-d) est-il juste et sans reste ? Le critère lui-même laisse-t-il échapper un texte qui
   **nomme** un exercice à rouvrir sans employer ces verbes (`Réouvrir`, `rouvert`, frontend, serveur) ? Les quatre
   sites du renommage sont-ils tous ?
3. **#504.** Toute la communication passe-t-elle par `build_qrbill_inputs` ? Un autre site écrit-il « Facture » (ou
   son équivalent) dans une QR **émise** ? Le contrôle de l'AC16 (zéro hors `supplier_invoices.rs`, compte 1 dans ce
   fichier) est-il décidable et attrape-t-il la récidive visée ? L'exception `supplier_invoices.rs:512` et
   l'inventaire des libellés d'écriture (*Hors périmètre*, C-31-5-17) sont-ils exacts ? Rapprochement insensible au
   préfixe, isolation Fluent, `pdf_contains` : revérifie.
4. **#469.** Six sites, dix cas, bras `VALIDATION_ERROR` : revérifie. Le contrôle multi-ligne de l'AC16 (`tr '\n' ' '`,
   jeton après `AppError::Validation(`) est-il juste avant **et** après correctif (formes `t(`, `crate::errors::t(`,
   `t_args(`, un `map_err(|e| …)`) ? L'extension de **G9** (`crates/kesh-api/tests/textes_coherents.rs:680`, entrée
   `audit_log.rs`, paramètres `{ $field }` → `{nom}`, `{ $max }` → `{maximum}`) est-elle faisable telle qu'écrite (forme
   du repli, littéral comparé, nombre de sites par clé, type du tableau) ?
5. **Écartées.** Les motifs d'écart de #687 et #645 tiennent-ils ? Les comptes attribués à l'issue #645 et recomptés
   (10, 37) sont-ils justes ?
6. **Manuel.** Les lignes citées de `docs/manual/fr/user-manual.tex` (183, 185, 192-204, 848, 3258) disent-elles ce que
   la fiche prétend ? Les retouches prescrites (AC3, AC10) sont-elles exactes après correctif ? Contrôle le **PDF
   aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "indique lequel rouvrir"`
   (et la phrase de la l. 3258). Le manuel d'administration contredit-il quelque chose ?
7. **Tests et mutations.** Chaque test prescrit rougit-il avant correctif pour la bonne raison ? M1-M6 (M1 et M3
   réécrites en P1) sont-elles tuées par un test nommé, ou équivalentes ? Un test qui passerait à vide (le test
   production de `onboarding_path_b_e2e.rs`, vert avant comme après, prouve-t-il quelque chose) ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses des issues** : chacune des quatre causes est-elle établie au code ? Une prémisse corrigée à tort ?
2. **Périmètre et découpage** : les modules sont-ils toujours cinq après la refonte de #707 (deux fonctions, le
   document du Pattern 5, `textes_coherents.rs`) ? Les volets restent-ils indépendants ? Une migration est-elle
   nécessaire ?
3. **Faisabilité** : `t_args` `pub(crate)` ; sept appels de test de `build_qrbill_inputs` ; clé
   `invoice-qr-message-prefix` sans conflit de garde ; test HTTP de l'AC2 sur une démonstration ; signature et type de
   retour de `lock_vat_role_accounts_in_tx` / `designate_vat_accounts_in_tx` compatibles avec `final_body` et
   `SeedAttemptError`.
4. **Décomptes** : recompte chaque nombre à sa source (141 issues et la chaîne 140/136/137 de l'*Origine* —
   `gh issue list --state open --limit 500 --json number --jq length` —, six sites, dix cas, sept appels, sept clés, 289
   lignes, 30 lignes de l'AC6, 10 et 37 de #645). Le budget d'issues est-il correctement exposé, et l'arbitrage
   correctement renvoyé au Project Lead ?
5. **Cohérence interne** : AC1-AC16, T0-T6, registre C-31-5-0 à 17, *Hors périmètre*, *Fichiers touchés*, Change Log —
   contradiction, critère invérifiable, commande mal formée (essaie mentalement **chaque** commande de l'AC16 contre le
   code actuel et contre un correctif plausible) ; résidu d'une formulation corrigée en P1 (grep des symptômes :
   `designate_vat_accounts_by_role`, `vat_payable_account_for_write` employé comme site de validation, `140` comme compte
   courant, `demandée`, `Fixed`).
6. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (tests de `kesh-seed`, de
   `company_invoice_settings`, tests qui attendent « Facture », `nearestLaterClosed` ou les messages français du journal
   d'audit), une conséquence pour la recette visuelle, le CHANGELOG.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps** (la P1 R annonçait 3 MEDIUM pour 4
dans le corps). ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F6) — sans elle, « 0 finding » ne compte
pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit
**d'un seul jet**, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire. Aucun `git checkout`/`switch` sous aucune forme ;
aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule **`tr`, `sort`, `cut`,
`head`, `awk`** vers la sortie standard (jamais de redirection vers un fichier).
