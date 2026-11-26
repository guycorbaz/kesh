# Prompt — revue de code P1 de la story 53-3 « L'ouverture ne se passe pas sur un compte de résultat » (#513, P1), passe complète

Tu relis le **code** de la story **53-3** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB 10.11, SvelteKit), Epic 53 « Écritures justes »,
release **v0.21.0**. Kesh est **en production** : un bilan d'ouverture posé sur un compte de produits ou de charges fausse les livres
réels. **Aucune migration**, aucun frontend. Première passe de revue : **complète**, contexte frais, modèle **Opus** (la fiche et le
développement sont de Sonnet). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-53-3`, branche `story/53-3-ouverture-sous-verrou`. Fiche :
`_bmad-output/implementation-artifacts/53-3-ouverture-sous-verrou.md` (statut `review` ; Dev Agent Record rempli).

**Le diff à relire** : le code de la story est `git diff 4d540a6ea 607fb31e8 -- crates` (base `4d540a6ea` = `origin/epic/53-ecritures-justes` ;
`607fb31e8` = dernier commit de code). Le commit `98e2051af` (fusion de `origin/main`) ne touche aucun fichier de la story : contrôle-le
(`git diff --stat 607fb31e8 98e2051af -- crates/kesh-db/src/repositories/journal_entries.rs crates/kesh-db/src/repositories/accounts.rs crates/kesh-api/src/routes/opening_balances.rs`
doit être vide). Les commits `4b61e8c04`, `ff2c982dd`, `607fb31e8` sont : fiche, tests rouges, correctif.

## Ce que fait la story (à vérifier, non à croire)

`create_opening_entry` (`kesh-db`, `repositories/journal_entries.rs`) lit désormais, **dans la transaction qui écrit**, le type des comptes
visés par `accounts::lock_types_by_ids_in_tx` (`LOCK IN SHARE MODE`, `FORCE INDEX (PRIMARY)`, `ORDER BY id`), **après**
`fiscal_years FOR UPDATE` et **avant** `count_by_company` ; il refuse `Revenue`/`Expense` par `DbError::Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY)`
**après** les gardes « société vierge » et « premier exercice clos ». Le handler `POST /api/v1/opening-balances` (`kesh-api`,
`routes/opening_balances.rs`) a perdu sa transaction jetée et son pré-contrôle ; `map_opening_balances_error` mappe la clé neuve en 400.
**Décision D3 (Q1 du Project Lead, 2026-11-25)** : `accounting::validate` précède désormais le contrôle des types ; une requête qui cumule
un compte de résultat et un refus de `validate` rend le code de `validate` (`ENTRY_UNBALANCED`, `FX_*`, `VALIDATION_ERROR`), statut 400.
Annoncé ⚠️ aux notes de version ; **ne pas le signaler comme défaut**, mais vérifier que l'effet est exactement celui que D3 décrit.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Review Iteration Rule* (dont *Inventorier les sites NON RÉSOLUS*, *Propagation post-patch*,
   *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*), *Priorités des défauts*, *Migration breaking policy*
   (pour constater qu'elle ne s'applique pas).
2. `docs/agents-orchestration.md` (§ 2, § 2.1, § 3, § 6) et `docs/testing.md` (§ *Base de dev jetable*).
3. La fiche entière, surtout D1 à D6, P5 et P6, § *Verrous*, AC-1 à AC-9, § *Tests et mutations*, Dev Agent Record (T0, rouges, mutations, gates).
4. `crates/kesh-db/src/repositories/{journal_entries,accounts,fiscal_years,opening_complement,company_invoice_settings}.rs`, `retry.rs`,
   `crates/kesh-api/src/routes/{opening_balances,journal_entries}.rs`, `errors.rs`.
5. Les journaux de la story : `/home/gcorbaz/devel/kesh-gate-logs/e533/` (`t0-mesures*.txt`, `rouge-*.log`, `vert-*.log`, `mutations/`, `gate-*.log`, `e2e-*.log`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **R1 — le correctif ferme-t-il #513 ?** Reconstitue la course : retypage d'un compte sans écriture par `accounts::update` (société en partage, compte
  `FOR UPDATE`) contre `create_opening_entry` (société `FOR UPDATE`). Tout retypage commité avant le sentinelle est-il vu ? Tout retypage qui démarre
  après attend-il ? Existe-t-il un autre écrivain de `account_type` (rejoue `git grep` du symptôme : `account_type =`, `UPDATE accounts`, `REPLACE INTO accounts`, `*.sql`, `post_restore`) ?
- **R2 — verrous et interblocage** : place de la lecture (après `fiscal_years FOR UPDATE`, avant `count_by_company`) ; vue non ouverte ; verrou « avancé, non neuf »
  (même lignes que `validate_lines_accounts_in_tx`, même transaction ?) ; aucun cycle neuf (graphe des attentes avec `update`, `archive_guarded`,
  `reactivate`, le complément d'ouverture, `create_in_tx_inner`, les flux « exercice avant société ») ; une `DbError::Invariant` n'est jamais rejouée (`retry.rs`).
- **R3 — la requête** : `FORCE INDEX (PRIMARY)`, `ORDER BY id`, `LOCK IN SHARE MODE` (jamais `FOR SHARE`), scopée société, `ids` vide → `Ok(vec![])`, placeholders ;
  identifiants dupliqués dans `ids` ; un identifiant d'une autre société (aucune fuite : verdict `INACTIVE_OR_INVALID_ACCOUNTS`, anti-énumération KF-002).
- **R4 — ordre des verdicts** : NotFound → ALREADY_HAS_ENTRIES → FIRST_YEAR_CLOSED → NON_BALANCE → refus de `create_in_tx`. Comparer à l'ordre de `4d540a6ea`
  (hors D3). Les rollbacks sont-ils faits avant chaque `return Err` ? Rien n'est-il écrit ni audité sur un refus ?
- **R5 — handler et mapping** : le handler ne lit plus de type ; l'arm de `map_opening_balances_error` existe, avec la même clé i18n ; une clé non mappée
  donnerait 500 ; les quatre locales portent `error-opening-balances-non-balance-account` ; doc-comments à jour (`opening_balances.rs`, `journal_entries.rs`,
  `accounts.rs`) ; import `accounts` retiré à bon droit.
- **R6 — effet exact de D3** : suis `accounting::validate`, `map_core_error` (`routes/journal_entries.rs`), `parse_line_amounts`. Les trois cas de
  `opening_balances_e2e.rs` (`d3_*`) épinglent-ils ce que D3 décrit (codes, statut) ? Un cas de D3 est-il mal décrit ou non couvert ?
- **R7 — tests** : AC-3 (entrelacement forcé, assistant partagé `kesh_db::test_fixtures::attendre_une_requete_en_cours`, motifs), AC-4, AC-5 (a) à (e), AC-8 (garde lexicale
  `opening_types_sous_verrou_lexical.rs`, sondes positive et négative). Un test vrai par construction ? Un `sleep` ? Un motif `PROCESSLIST` qui attraperait la
  mauvaise requête ? Les rouges au code de base sont-ils au journal (`rouge-*.log`) et pour la bonne raison ? Les mutations M1 à M12 (`mutations/`) : chacune
  tue-t-elle bien ce que la fiche annonce, M10 reste-t-elle vivante comme angle mort écrit ? Relance **mentalement** (sans exécuter) deux mutations de ton choix.
- **R8 — inventaire clos (W1 à W11 de la fiche)** : rejoue les commandes d'inventaire et cherche ce qu'elles omettent (un chemin qui écrit une ouverture sans
  `create_opening_*` ; MCP ; `provider_review`, `reconciliation*`, `bank_statements_manual.rs`, `services/ecritures.rs`). AC-7 : aucun fichier de production de
  `opening_complement` ni de `bank_accounts` n'est touché.
- **R9 — manuel et documentation, PDF compris** : vérifie `docs/manual/` contre le code, **PDF aplati avant de greper** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `produits ou de charges`, `comptes de bilan`, `soldes de départ`, `ce que la génération refuse` ;
  `admin-manual.tex` ; `docs/api-external.md` (la précédence des refus de `POST /opening-balances` n'y est pas documentée). La fiche conclut « aucune modification de manuel » : juste ?
  Notes de version : numéro de release, jamais un epic, ⚠️ pour D3, `refs #513`, aucun mot-clé de fermeture dans les commits
  (`git log 4d540a6ea..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`).
- **R10 — propagation du symptôme** : `git grep -n` de `find_types_by_ids_in_tx`, `lock_types_by_ids_in_tx`, `non-balance`, `NON_BALANCE`, `Revenue.*Expense` sur tout le dépôt
  (code, fiches, doc-comments, tests, locales, manuels, `_bmad-output`) : reste-t-il un site périmé hors de la prose historique des fiches 14-4 et 25-7 ?
- **R11 — mesures (AC-6)** : `t0-mesures.txt`, `t0-mesures-bis.txt` : la lecture de la fiche et du Dev Agent Record est-elle fidèle (plan `range` sur `PRIMARY`, 1205, `INSERT`) ?
  Le constat « plan `ALL` de `validate_lines_accounts_in_tx` sur statistiques périmées » est-il établi et correctement renvoyé à une issue, sans réécriture dans la story ?
- **R12 — périmètre et gates** : story minimale (deux crates) ; aucune migration (`git diff 4d540a6ea 607fb31e8 --stat -- crates/kesh-db/migrations` vide) ; gates déclarés
  (`test-fast --ci`, `gate-ci`, frontend, E2E) : chiffres des journaux = chiffres du Dev Agent Record ? « Le story file n'affirme que ce qui a tourné ».
- **R13 — défauts voisins** : tout défaut vu hors de la story est à **signaler** (issue demandée), non corrigé.

## Interdits

**Lecture seule.** N'écris **aucun** fichier du worktree ni du dépôt principal, sauf ton rapport. **Aucune redirection de shell (`>`, `>>`, `tee`) vers autre chose que ton rapport.**
Aucun `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `docker compose`, `mariadb`, `mysql`, `sqlx`, `curl`, `python3` autre que `python3 -I` en lecture pour recompter ;
aucun `scripts/*` — en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`, ni
`scripts/install-hooks.sh`. Aucun `git commit`, `git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git reset`, `git rebase`, `git worktree`, `git tag`, `git fetch`.
**Aucun `pkill`, `killall`**, ni `kill` par motif ; aucune commande qui arrête un processus. Aucun `gh` qui écrit (`issue create|edit|comment|close`, `pr …`, `api` avec `-X` ou `-f`) ;
au plus deux requêtes `gh` de lecture, espacées d'une minute. Rien sous `~/kesh/`. Une commande préalable refusée t'arrête : signale-le dans le rapport.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e533/code-review-p1.md` :

- le **verdict chiffré** CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`R1-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction proposée ; pour tout finding CRITICAL/HIGH
  qui affirme une absence ou un anti-pattern, la vérification `grep -nF` de la ligne ;
- les axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les prémisses de la fiche **confirmées** au code (une ligne chacune, ligne citée) et celles **démenties** ;
- la liste des issues à demander (une par problème).

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
