# Prompt — validation P2 de la fiche 53-3 « L'ouverture ne se passe pas sur un compte de résultat » (#513, P1), passe complète

Tu valides la **fiche** de la story **53-3** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), Epic 53 « Écritures justes »,
release **v0.21.0** (vendredi 2026-12-04). Kesh est **en production** : un bilan d'ouverture passé sur un compte de produits ou de charges
fausse les livres réels. **Aucune migration**, aucun frontend dans la story. Deuxième passe : **complète**, contexte frais, modèle
**Sonnet** (la P1 était Opus ; la fiche est de Sonnet, remédiée le 2026-11-25). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-53-3`, branche `story/53-3-ouverture-sous-verrou`, partie de `origin/epic/53-ecritures-justes` à
`4d540a6ea`. Fiche : `_bmad-output/implementation-artifacts/53-3-ouverture-sous-verrou.md` (statut `drafted`).

**La P1 (Opus, 0/0/3/8) et sa remédiation** : rapport `/home/gcorbaz/devel/kesh-gate-logs/e533/validation-p1.md`, Change Log de la fiche. **Axe A12 : chaque finding V1-1 à V1-11 est-il réellement remédié, au texte de la fiche, ligne lue citée (pas seulement annoncé au Change Log) ? Un patch a-t-il introduit un défaut neuf ?** Décisions du Project Lead du 2026-11-25 : Q1 acceptée (D3), Q2 tracée en issue (ne pas les remettre en question ; juger leur écriture).

**Le point le plus important** : la fiche affirme (P5, D5, AC-3 ; D1 place désormais la lecture **avant** `count_by_company`, P6 ii) que `accounts::update` (le retypage) et `create_opening_entry` se
sérialisent **déjà** sur la ligne `companies` (partage contre `FOR UPDATE`), donc que la fenêtre de #513 est entièrement **en amont** du
sentinelle ; et (P6, AC-4) qu'une relecture **ordinaire** du type lirait l'instantané REPEATABLE READ, qu'il faut `LOCK IN SHARE MODE`, et que la placer avant `count_by_company` évite `ER_CHECKREAD` (1020) sous MariaDB ≥ 11.6 (#669). Vérifie les deux au code, ligne à ligne, avant tout le reste. Si l'une est fausse, la fiche se trompe de
verrou ou de test : c'est un HIGH au moins.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Review Iteration Rule* (dont *Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe
   doit NOMMER le manuel*, *Une passe déclare ses axes exercés*), *Priorités des défauts*, *Migration breaking policy* (pour constater
   qu'elle ne s'applique pas), *Test Locally First*.
2. `docs/agents-orchestration.md` (§ 2, § 2.1 : interdits ; § 3 : processus).
3. `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md` (la 53-3, ses fichiers déclarés, ses recoupements).
4. Le code : `crates/kesh-api/src/routes/opening_balances.rs` (handler `generate_opening_balances`, `map_opening_balances_error`),
   `crates/kesh-db/src/repositories/journal_entries.rs` (`create_opening_entry`, `create_opening_in_tx`, `create_in_tx_inner`,
   `validate_lines_accounts_in_tx`), `crates/kesh-db/src/repositories/accounts.rs` (`update`, `lock_company_currency_in_tx`,
   `find_types_by_ids_in_tx`, `count_by_company`), `crates/kesh-db/src/repositories/opening_complement.rs`,
   `crates/kesh-db/src/repositories/fiscal_years.rs` (clés `FY_OPENING_*`), `crates/kesh-db/src/retry.rs`.
5. L'issue #513, **en lecture seule**, **au plus deux requêtes `gh` dans toute la passe, espacées d'au moins une minute** :
   `gh issue view 513 --json number,title,labels,body` ; au besoin `gh issue view 274 --json number,title,state,body`.
6. `_bmad-output/implementation-artifacts/14-4-bilan-ouverture-soldes-depart.md` (surtout `:318`, `:381`, ECH-1) et
   `25-7-soldes-de-depart.md` (§ « Limites assumées », mesure de l'ordre des verrous).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — prémisses P1 à P4 relues au code** : chaque `chemin:ligne` cité est-il juste à `4d540a6ea` (copie la ligne) ? Le handler
  ouvre-t-il bien une transaction jetée ? `create_opening_entry` ne relit-elle bien aucun type ? Le seul écrivain de `account_type` est-il
  bien `accounts.rs:1271` (rejoue le `git grep`, y compris `sqlx::query` construits par `format!`, les `QueryBuilder`, les fichiers
  `*.sql` de `crates/kesh-db/src` et `post_restore.rs`) ?
- **A2 — verrous (le cœur)** : P5, D5 et le tableau « Verrous : ordre et interblocage ». Reconstitue **toi-même** le graphe des attentes :
  `update` (société S → réglages S → compte X → …), `create_opening_entry` (société X → exercice X → …), le complément (société S →
  comptes X `ORDER BY id` → report S → exercice X), `create_in_tx_inner` (exercice X → … → comptes S), les flux « exercice avant société ».
  Existe-t-il un cycle **nouveau** créé par un `LOCK IN SHARE MODE` sur `accounts` placé après `fiscal_years FOR UPDATE` ? Un cycle avec
  `accounts::update` (qui prend ensuite les liens bancaires et la ligne d'écriture en lecture verrouillante) ? Avec les gestes qui prennent
  `accounts` avant `companies` (cherche-les : `archive_guarded`, `bank_accounts`, `reactivate`, rapprochements) ? Le rejeu
  (`retry_on_deadlock`) couvre-t-il le cas, et une `DbError::Invariant` n'est-elle bien jamais rejouée (`retry.rs`) ? La fiche dit
  « verrou avancé, non neuf » : `validate_lines_accounts_in_tx` lit-il bien les **mêmes** lignes dans la **même** transaction ?
- **A3 — P6, l'instantané** : sous REPEATABLE READ MariaDB, une lecture ordinaire qui suit des lectures verrouillantes (`FOR UPDATE`) et un
  `COUNT(*)` ordinaire voit-elle l'instantané de ce `COUNT` ? Un `LOCK IN SHARE MODE` lit-il la dernière version commitée (lecture
  « courante ») et attend-il un écrivain non commité ? Le niveau d'isolation de la base de Kesh est-il bien celui par défaut (cherche un
  `SET TRANSACTION ISOLATION` ou une option de pool dans `crates/kesh-db/src/` et `crates/kesh-api/src/`) ? Si `READ COMMITTED` était posé,
  P6 et AC-4 changent de sens : dis-le.
- **A4 — inventaire clos des chemins (W1 à W9)** : rejoue les commandes de la fiche et cherche ce qu'elles **omettent** : un site qui
  appelle `create_in_tx` avec un journal OD et une date de début d'exercice sans passer par `create_opening_*` ; un chemin MCP ou
  d'import de relevé qui écrit une ouverture ; `provider_review`, `reconciliation*`, `bank_statements_manual.rs`, `invoices.rs`,
  `services/ecritures.rs` (ils mentionnent « ouverture »). Chaque site est-il **couvert**, **non concerné** (raison vérifiable) ou
  **angle mort assumé** ? W5 (restauration) est-il bien un angle mort et non un défaut ?
- **A5 — décisions** : chaque D a-t-elle son option écartée, motivée ? D2 (renommer `find_types_by_ids_in_tx` en lecture verrouillante plutôt
  que d'ajouter `account_type` à `validate_lines_accounts_in_tx`) est-elle juste ? D3 (changement d'ordre d'un verdict sur requête
  doublement fautive) : l'effet décrit est-il exact (suis `accounting::validate` et `map_core_error`) et l'ordre des autres verdicts, listé
  en D1, est-il bien l'ordre **actuel** du handler (`opening_balances.rs:398-464`) puis de `create_in_tx` ? D4 : la clé neuve, mappée
  nulle part, donnerait-elle bien un 500 (`AppError::from(DbError::Invariant)`) ? La clé existante `error-opening-balances-non-balance-account`
  est-elle dans les quatre locales ?
- **A6 — AC constatables** : chaque AC dit-il un test, une valeur attendue, un fichier ? AC-3 et AC-4 sont-ils **tenables** : les motifs
  `PROCESSLIST` annoncés attrapent-ils les requêtes visées (le texte montre les `?`, `INFO LIKE ?`) ? Le test AC-3 rougit-il vraiment sans
  correctif (que rend `create_opening_entry` aujourd'hui sur un compte retypé `Revenue` : `Ok`, ou un refus d'une autre garde, par
  exemple `validate_lines_accounts_in_tx` ?) ? Peut-on l'écrire sans `sleep` ? AC-5 : les quatre verdicts attendus sont-ils justes au code ?
- **A7 — mutations** : chacune des sept (M1 à M7) tue-t-elle bien le test annoncé, et **seulement** lui ? M4 est déclarée tuée par AC-4
  seul : est-ce vrai (relis AC-3 : la société est déjà un verrou commun) ? Une mutation manque-t-elle (par exemple : le contrôle placé
  **après** `create_opening_in_tx`, ou une lecture sur le pool au lieu de la transaction) ?
- **A8 — manuel, PDF compris** : `docs/manual/fr/user-manual.tex:1051` et `:1069`, `admin-manual.tex` (recherche `soldes de départ`,
  `ouverture`) ; cherche dans les `.tex` **et** dans les PDF aplatis
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o -i '.\{120\}comptes de bilan.\{120\}'`, puis
  `produits ou de charges`, `soldes de départ`) toute phrase que la story rend fausse ; la fiche conclut « aucune modification » : est-ce
  juste ? Les langues `de`, `en`, `it` existent-elles vraiment (la fiche dit que non) ?
- **A9 — tests existants et bruit** : `opening_balances_repository.rs:260`, `:321` (tests de `find_types_by_ids_in_tx`),
  `opening_balances_e2e.rs:571`, `chf_line_lexical.rs`, `test_schema_guard.rs` : le renommage ou le déplacement du contrôle en casse-t-il
  un autre, ou en fait-il passer un à vide ? Les tests de `opening_complement_*` et `bank_account_opening_balance_e2e` dépendent-ils
  d'un message ou d'un ordre que la story change ?
- **A10 — périmètre, splitting, priorité** : « deux crates, pas de splitting » est-il juste (un module = un crate ou un domaine du
  frontend dont le code de production change) ? La story est-elle **minimale** (le défaut voisin) ? La note de version nomme-t-elle une
  release (v0.21.0), jamais un epic, et `refs #513` sans mot-clé de fermeture ? L'issue et la fiche se contredisent-elles (« pas de
  données de production » : la fiche le déclare périmé) ? Les questions Q1 et Q2 sont-elles bien posées ?
- **A11 — mesures (AC-6)** (deux requêtes, `const`/`range`, 1205, intervalles ; l'autre société) : l'`EXPLAIN` et la sonde `NOWAIT` prescrites prouvent-elles ce qu'elles prétendent ? Une sonde `NOWAIT`
  sur un compte **non visé** peut-elle réussir pour une autre raison (verrou d'intervalle, plan en `ref` sur `company_id`) ? Dis ce qui
  manque.

- **A12 — remédiation de la P1** : voir en tête. Relis aussi D1 (ordre d'exécution et ordre des refus), D3, D5 (arête déplacée et non créée : refais le raisonnement avec la place avant `count_by_company`), AC-5 (a) à (e) au niveau du dépôt (M6, M6-bis meurent-elles ?), W10 et W11, l'issue rédigée du § *Décisions du Project Lead* (les fichiers et fonctions cités existent-ils ? le compte est-il juste : recompte `grep -rln "FROM information_schema.PROCESSLIST" crates/*/tests`), la ligne d'epic à ajouter.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal, sauf ton rapport. Aucun `Edit`/`Write` ailleurs, **aucune redirection de shell (`>`, `>>`, `tee`) vers autre chose que ton rapport**. Aucun `cargo`,
`npm`, `npx`, `node`, `make`, `docker`, `docker compose`, `mariadb`, `mysql`, `sqlx`, `python3` autre que `python3 -I` en lecture pour
recompter, `curl` vers un serveur, aucun `scripts/*` — en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`. Aucun `git commit`, `git push`, `git checkout`,
`git switch`, `git stash`, `git merge`, `git reset`, `git rebase`, `git worktree`, `git tag`. **Aucun `pkill`, `killall`** ni `kill` par motif. Aucun `gh` qui écrit (`issue
create|edit|comment|close`, `pr …`, `api` avec `-X` ou `-f`). **Jamais `pkill`, `killall` ni `kill` par motif** ; aucune commande qui
arrête un processus. Rien sous `~/kesh/`.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e533/validation-p2.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`V1-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction
  proposée ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette
  liste ne compte pas comme passe ;
- les prémisses de la fiche que tu as **confirmées** au code (une ligne chacune, ligne citée) et celles que tu as **démenties**.

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
