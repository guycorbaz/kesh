# Prompt — validation P3, Story 31-5 (quatre défauts courts)

*Versionné le 2026-10-24. Passe **complète** (non ciblée : la remédiation P2 a changé la forme de la lecture
verrouillante de #707, ajouté un test et trois mutations, et réécrit le budget d'issues). **Deux lentilles, R et F**,
chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-5`, branche `story/31-5`. Fiche :
`_bmad-output/implementation-artifacts/31-5-defauts-courts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-31-choix-autonomes-315.md` (C-31-5-0 à C-31-5-20). Règles : `CLAUDE.md`
(§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW —,
*Règle de splitting préventif*, *Exception kesh-db*, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les
sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter ses propres comptes rendus*, *Priorités des défauts*),
`docs/agents-orchestration.md` § 3 et § 6. La fiche traite #707, #589, #504 et #469 et écarte #687 et #645 : lis
chacune (`gh issue view <N>`, lecture seule).

**Ce que la P2 a changé** (Change Log de la fiche, entrée « Validation P2 » ; diff : `git show da6fa4c7`) :

- **#707, lecture verrouillante** : la requête `IN (…) ORDER BY id FOR UPDATE` est remplacée par **trois lectures
  ponctuelles** `singleton_role = ? FOR UPDATE` dans l'ordre du code (`VatPayable`, `VatRecoverable`, `VatSettlement`),
  et la fiche décrit l'**ordre réel** des verrous (AC1, Dev Notes, T2, C-31-5-13 annoté, C-31-5-18).
- **#707, filtres** : le filtre de type est celui du **réglage** (types de la route), écrit une fois en `kesh-db`, et
  **non** `accepts_account_type` (C-31-5-19 — écart assumé à une consigne de l'orchestrateur, à juger) ; test `kesh-db`
  de la lecture à quatre cas (AC1) ; mutations M7a, M7b et M8 (AC15).
- **Budget d'issues** : *Origine* inconditionnelle (137 au tag si rien d'autre ne s'ouvre, 138 si l'issue des libellés
  s'ouvre ; plafond < 136 ; arbitrage au Project Lead ; C-31-5-16 et 17 annotés).
- **LOW** : numéros de ligne recalés ; AC6 élargi à `rouvert` (34 lignes) ; contrôles de l'AC16 resserrés
  (`grep -c 'format!("Facture'`, « toujours six » jetons) ; AC9 (a) en deux temps ; préfixe `audit-log-query-error-*`
  (C-31-5-20) ; justification de l'AC3 nuancée (l'écran nomme le menu à un administrateur) ; capture
  d'*Administration → Facturation* ; 283 lignes hors journal d'audit.

Les rapports P2 sont `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p2-{R,F}.md` : ne les prends pas pour argent
comptant — revérifie au code ce qu'ils affirment et ce que la fiche en a fait, et cherche **en priorité ce que la
remédiation P2 a introduit** (motif mesuré : la sévérité se déplace vers ce qu'on vient d'écrire).

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **#707.** Cause, appelants, chemin du refus (`invoices.rs:1877-1881`, `:2364`, `check_written` `:2372`) : revérifie.
   **La forme P2** : trois lectures `SELECT id, account_type, postable FROM accounts WHERE company_id = ? AND
   singleton_role = ? FOR UPDATE` — l'accès est-il bien `const` sur `uq_accounts_company_singleton_role` (schéma,
   colonne générée, comparer `company_invoice_settings.rs:1019-1027` et `:1076-1105`) ? Les verrous décrits (entrée de
   l'index, enregistrement du compte, intervalle pour un rôle absent) sont-ils justes ? L'ordre annoncé (2200, 1171, 2206
   au plan PME) correspond-il au code prescrit ? L'analyse du seul ordre opposé plausible
   (`lock_designated_accounts_in_tx`, validation d'une facture) tient-elle, et existe-t-il un autre site qui verrouille
   ces comptes dans un autre ordre (`finalize`, `reset_demo`, le PUT des réglages, l'archivage ou le retypage d'un compte,
   Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md`) ? La séquence `onboarding_state → companies → accounts →
   company_invoice_settings → vat_rates → audit_log`, la condition `invoice_settings_created` et le `COALESCE`
   tiennent-ils ? Le doc de `final_body` est-il bien l. 364-369 ?
2. **#707, filtres et type (C-31-5-19).** `accepts_account_type` (`kesh-core/src/chart_of_accounts/mod.rs:161`,
   appliquée par `kesh-db/src/repositories/accounts.rs:89`) est-elle vraiment la règle du rôle, laxiste pour les trois
   rôles de TVA, et donc inopérante comme filtre ? La règle du réglage n'existe-t-elle vraiment qu'à la route
   (`routes/company_invoice_settings.rs:337-364`) ? Une constante de `kesh-db` est-elle le bon emplacement, et le risque
   de dérive entre les deux copies est-il gardé ou seulement nommé ? Les quatre cas du test `kesh-db` (heureux ; retypé
   en actif ; non imputable ; archivé) sont-ils constructibles par `UPDATE` en montage sans heurter une contrainte, un
   déclencheur ou une colonne générée, et chacun produit-il bien l'état annoncé (l'archivage annule-t-il
   `singleton_role`) ?
3. **#589.** Garde serveur, quatre sites du renommage, critère de l'AC6 désormais `rouvr|Rouvr|réouv|Réouv|rouvert|Rouvert`
   (34 lignes) : recompte et vérifie le tri a-d **sans reste** ; une autre forme de la famille échappe-t-elle encore
   (`Réouvert`, `réouvert`, `rouvrir` en majuscule, frontend, serveur) ?
4. **#504.** `build_qrbill_inputs` seul site ; contrôles de l'AC16 (zéro hors `supplier_invoices.rs`, `grep -c
   'format!("Facture'` → 1) essayés contre le code ; AC9 en deux temps — le premier temps (paramètre ajouté, ignoré)
   laisse-t-il bien (a) rouge sur la valeur et (b) rouge, sans casser d'autre test ? `pdf_contains`, isolation Fluent,
   rapprochement : revérifie.
5. **#469.** Six sites, dix cas, contrôle multi-ligne (« toujours six » et zéro au filtre) avant et après correctif ;
   préfixe `audit-log-query-error-*` sans collision (catalogues, frontend, `lint-i18n-ownership.js`) ; G9 faisable.
6. **Manuel.** Lignes citées de `docs/manual/fr/user-manual.tex` (183, 185, 205, 848, 3258) ; retouches prescrites
   (AC3, AC10) exactes après correctif ; la justification nouvelle de l'AC3 (écran de la facture,
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte:409-411`) ; **PDF aplati** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "indique lequel rouvrir"` (et la
   phrase de la l. 3258) ; le manuel d'administration contredit-il quelque chose ?
7. **Tests et mutations.** M1-M8 (M7a, M7b) : chacune est-elle tuée par un test **nommé**, ou équivalente et déclarée ?
   M8 est-elle réalisable telle qu'écrite (`onboarding.rs:830`, transaction, types) et tue-t-elle bien le test de
   `onboarding_path_b_e2e.rs` ? Un test passerait-il à vide ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses des issues** : chacune des quatre causes est-elle établie au code ? Une prémisse corrigée à tort ?
2. **Périmètre et découpage** : toujours cinq modules (`kesh-db` — deux fonctions, une constante, un test —,
   `kesh-seed`, `kesh-api`, `kesh-i18n`, `frontend`) ? M8 touche-t-elle un fichier de production non listé (elle n'est
   qu'une mutation, restaurée) ? Volets indépendants ? Migration nécessaire ?
3. **Faisabilité** : signatures `-> Result<VatRoleAccounts, DbError>` et `-> Result<(), DbError>` compatibles avec
   `final_body` et `SeedAttemptError` ; trois lectures sur une transaction `&mut Transaction<'_, MySql>` ; décodage de
   `account_type` et `postable` ; `t_args` `pub(crate)` ; sept appels de test de `build_qrbill_inputs` ; test HTTP de
   l'AC2 ; la capture d'*Administration → Facturation* sur une démonstration.
4. **Décomptes** : recompte chaque nombre à sa source — 141 issues et la chaîne 140/136/137/138 de l'*Origine*
   (`gh issue list --state open --limit 500 --json number --jq length` ; l'issue des libellés et celle de la 31-1c
   sont-elles toujours non ouvertes ?), six sites, dix cas, sept appels, sept clés, 283 et 289 lignes (commande écrite),
   34 lignes de l'AC6 et leur tri, 10 et 37 de #645. Le budget est-il exposé sans condition cachée, et l'arbitrage
   renvoyé au Project Lead ?
5. **Cohérence interne** : AC1-AC16, T0-T6, registre C-31-5-0 à 20, *Hors périmètre*, *Fichiers touchés*, Change Log —
   contradiction, critère invérifiable, commande mal formée (essaie mentalement **chaque** commande de l'AC16 contre le
   code actuel et contre un correctif plausible) ; résidu d'une formulation corrigée en P2 (grep des symptômes :
   `ORDER BY id` employé comme forme prescrite, « clé primaire » comme ordre des verrous, `audit-log-filter-error`,
   `289` comme compte hors journal d'audit, `l. 204`, `:389`/`:415` comme sites de #687, `344-347`, `362-367`,
   `-> VatRoleAccounts` sans `Result`, « promet », « qui ouvre l'issue », `M1-M6` comme liste complète).
6. **Ce que la fiche omet** : un site, un test existant qui rougira sans être nommé (tests de `kesh-seed`, de
   `company_invoice_settings`, d'onboarding, tests qui attendent « Facture », `nearestLaterClosed` ou les messages
   français du journal d'audit), une conséquence pour la recette visuelle, le CHANGELOG (`### Corrigé` à créer).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps** (la P1 R annonçait 3 MEDIUM pour 4 dans le
corps). ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F6) — sans elle, « 0 finding » ne compte
pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit
**d'un seul jet**, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/315-validate-p3-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/315-validate-p3-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, aucun fichier temporaire. Aucun `git checkout`/`switch` sous aucune forme ;
aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`,
`cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule **`tr`, `sort`, `cut`,
`head`, `awk`** vers la sortie standard (jamais de redirection vers un fichier).
