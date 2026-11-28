# Prompt — validation P2 de la fiche 58-2 « Comptes désignés et tests Rust de revue » (#592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1127, #1433), passe complète

Tu valides la **fiche** de la story **58-2** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), epic de **nettoyage** (Epic 58, dix issues P3/P4 de **tests**), release visée v0.21.0.
Kesh tient les livres réels du Project Lead. La story **n'ajoute que des tests** : aucun code de production, aucune migration, aucun contrat d'API, aucun texte visible. Deuxième passe : **complète**, contexte frais,
modèle **Sonnet** (la P1 était Opus ; elle a rendu 0/0/6/7, tous remédiés dans la fiche). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-58-2`, branche `story/58-2-comptes-designes-tests-revue`, partie de `epic/58-nettoyage` à `90192c7b3`. Fiche : `_bmad-output/implementation-artifacts/58-2-comptes-designes-tests-revue.md`.
Fichier d'epic : `_bmad-output/planning-artifacts/epic-58-nettoyage.md` (§ *Story 58-2*). Rapport P1 : `/home/gcorbaz/devel/kesh-gate-logs/e58/58-2-validation-p1.md` — **lis-le après** ta propre lecture de la fiche, pour ne pas
t'ancrer sur ses conclusions ; il sert à vérifier que ses corrections tiennent, non à te dire quoi penser.

## Ce que tu dois établir d'abord, toi-même, au code de `90192c7b3`

1. **Les remédiations P1 sont-elles justes ?** Pour chacune (V1-1 à V1-13), relis la ligne citée et dis si la fiche corrigée est vraie :
   - V1-1 : `chart_designated_accounts` (`crates/kesh-db/src/repositories/company_invoice_settings.rs:674-724`) écrit-elle bien les natures en dur ? Le test proposé (natures en littéraux + `ALL == [Discount, BankFees, BadDebt]`)
     est-il indépendant du code (« vrai par construction » ?) ; les mutations d'AC3 dans `crates/kesh-core/assets/charts/pme.json` laissent-elles un plan **valide** pour `load_chart`/`validate_chart` (sinon le test rougit pour une autre raison) ?
   - V1-2 : la garde `git status --porcelain | grep -vE '^.. (crates/kesh-(api|db)/tests/|frontend/src/.*\.test\.ts$|_bmad-output/)'` voit-elle **toutes** les mutations des AC (frontend, `kesh-core/assets`, `kesh-core/src`, `kesh-db/src`, `kesh-api/src`) ?
     Une mutation dans un fichier **non suivi** ou renommé y échapperait-elle ? La regex est-elle juste (format `XY chemin` de `--porcelain`, chemins avec parenthèses `(app)`) ?
   - V1-3 : la mutation `.then(setting)` survit-elle vraiment (ordre d'itération du `BTreeSet<DesignatedRole>`, stabilité de `sort_by`) ? Le renvoi à #1059 est-il juste (lis `gh issue view 1059`) ?
   - V1-4 : la mutation (a) d'AC13 (bloc du garde de parent déplacé avant le `match`, `accounts.rs:1216`, `:1256-1268`) est-elle **réalisable** (compile ; `own_fx` dépend-il d'autre chose que `before` ?) et survit-elle aux tests
     existants (`kesh-db/tests/account_parent_change.rs`, `accounts_e2e.rs`, tests unitaires d'`accounts.rs`) ? La mutation (b) (`.bind(parent_id)`, l. 1336) : quel est le type de `changes.parent` (variante « absent ») ?
   - V1-5 : l'ensemble clos d'AC12 (sept tests → route → dépôt) est-il complet ? Suis toi-même **au moins deux** chemins (ex. `ac3_annulation_d_un_rapprochement_la_nuit` et `ac2_solde_d_ouverture_la_nuit_du_nouvel_an`) jusqu'au site de M3 et de M7.
   - V1-6 : rejoue `git diff --name-only 90192c7b3...origin/<branche>` sur les branches `epic/53*`, `story/53-*`, `epic/58*`, `story/58-*` ; le tableau corrigé est-il juste et complet ?
2. **Ce que la P1 n'a pas vu** : relis chaque AC et chaque décision comme si la P1 n'existait pas.

## À lire

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Test Locally First*, *Review Iteration Rule* (*Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*, *Inventorier les sites NON RÉSOLUS*,
   *Propagation post-patch*, *Recompter*), *Priorités des défauts* (vérifier qu'une issue n'est pas déjà couverte), *Règle de splitting préventif*, *Fusions et fichiers communs*, *Worktrees et agents*.
2. La fiche en entier ; le fichier d'epic (section 58-2).
3. Les issues, **en lecture seule**, **au moins dix secondes entre deux requêtes `gh`**, **dix requêtes au plus** : `gh issue view N --json number,title,state,labels,milestone,body,comments` pour celles dont tu as besoin
   (au moins 1059, 1127, 1433).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre** : dix issues = section 58-2 de l'epic ; zéro module de production ; splitting ; une journée.
- **A2 — l'écart existe-t-il encore ?** Une issue est-elle déjà couverte (test voisin, unitaire, Vitest, Playwright `frontend/tests/e2e/`, autre crate) ?
- **A3 — chaque mutation nommée est-elle réalisable et survit-elle aux tests existants ?** Chemin par chemin, pour AC1 à AC13, y compris les **survivantes attendues** que la fiche déclare (AC3, AC8, AC9) : le sont-elles vraiment ?
- **A4 — vrai par construction ?** Tables de D3 et D5, messages d'AC7, espion d'`onboarding.svelte` (D3) ; un test peut-il rester vert la garde défaite ?
- **A5 — montages** : D6 (`new_account` + `UPDATE` aux deux réglages ; `check_written`), AC4 (deux sociétés, `create_settings_row`, version), AC9 (`GET /api/v1/supplier-invoices/{id}`), AC10 (deux factures, `app.retype`, `POST /api/v1/credit-notes` → 201),
  AC13 (`caisse` avec écritures, code `OPTIMISTIC_LOCK_CONFLICT`).
- **A6 — #1127** : ensemble clos, procédure de mutation et de restauration (copie + `touch`, garde `git status --porcelain`), sûreté pour le travail parallèle, coût mémoire (`mem-guard`), journal sous `kesh-gate-logs/e58/`.
- **A7 — #732 en quatre points, une issue, une release** ; angle mort de D5 écrit (Q3).
- **A8 — manuel, PDF compris** : les phrases que la story fige (« type de compte incompatible (attendu : … ) », libellés `Compte …`) contre `docs/manual/fr/user-manual.tex` (§ *Rôles des comptes*, l. ~619 ; « Contrôle préventif », l. ~1619)
  et `admin-manual.tex` (l. ~4143-4147), **et les PDF aplatis** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, idem `admin-manual.pdf`) ; Q4 (l'avoir ne contrôle pas la TVA due retypée) : le code
  (`kesh-db/src/repositories/credit_notes.rs:545`, `:576-580`, `:661`) et le manuel concordent-ils ?
- **A9 — gates et fichiers communs** : gate ciblé par binaire, gate complet au dernier commit sur idefix (`gate-distant.sh`) ; exception `kesh-db` non déclenchée (aucun repository) : juste ? `sitesTotal` (`dansLePerimetreDeFichier`,
  `frontend/src/lib/shared/i18n-harvest.js`) inchangé ?
- **A10 — décisions, questions, note de version** : D1 à D8 (option écartée, motivée) ; Q1 à Q4 marquées tranchées ; note de version (release ou date, jamais un epic ; `refs #N`, aucun mot-clé de fermeture :
  `git log origin/epic/58-nettoyage..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`) ; aucune donnée réelle de Guy.
- **A11 — charge et ordre** : campagne d'AC12 en dernier, sur un arbre propre ; un commit par étape.
- **A12 — propagation des remédiations P1** : greppe dans la fiche les symptômes corrigés (`e582`, `preset`, `WriteOffNature::ALL`, `crates/\*/src/\*`, `dernière clé`, `4062`, `1281`) : reste-t-il un site non corrigé ? Le Change Log
  et le statut disent-ils 0/0/6/7, ventilation cohérente avec le total (recompte depuis le rapport P1) ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. **Commandes interdites** : `cargo`, `npm`, `npx`, `node`, `vitest`, `docker`, `mariadb`, `sqlx`, `curl` vers un serveur, `ssh`,
tout `scripts/*` — en particulier `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`, `kesh-gate-logs/serveur/gate-distant.sh` —,
`git commit`, `git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git rebase`, `git worktree`, `git add`, `git restore`. Aucun `gh` qui écrit (`issue create/edit/comment/close`, `pr …`, `api` avec `-X` ou `-f`).
Rien sous `~/kesh/`. **Jamais `pkill`, `killall` ni `kill` par motif.** Tu **ne mutes aucun fichier** : tu raisonnes les mutations. `python3 -I` en lecture et `pdftotext` sont admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e58/58-2-validation-p2.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`V2-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction proposée ;
- pour chaque remédiation P1 (V1-1 à V1-13) : **tient** / **ne tient pas** (avec la ligne lue) ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les prémisses de la fiche **confirmées** au code (une ligne chacune, ligne citée) et **réfutées**.

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
