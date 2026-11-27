# Prompt — validation P1 de la fiche 58-2 « Comptes désignés et tests Rust de revue » (#592, #732, #736, #1060, #1062, #1063, #1064, #1066, #1127, #1433), passe complète

Tu valides la **fiche** de la story **58-2** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), epic de **nettoyage** (Epic 58, dix issues P3/P4 de **tests**), release visée v0.21.0.
Kesh tient les livres réels du Project Lead. La story **n'ajoute que des tests** : aucun code de production, aucune migration, aucun contrat d'API, aucun texte visible. Première passe : **complète**,
contexte frais, modèle **Opus** (la fiche a été écrite par Sonnet ; la passe suivante sera Sonnet). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-58-2`, branche `story/58-2-comptes-designes-tests-revue`, partie de `epic/58-nettoyage` à `90192c7b3`. Fiche : `_bmad-output/implementation-artifacts/58-2-comptes-designes-tests-revue.md`
(statut `ready-for-dev` sous réserve de ta validation). Fichier d'epic : `_bmad-output/planning-artifacts/epic-58-nettoyage.md` (§ *Story 58-2*, « Fichiers et modules touchés », « Recoupements »).
Fiches voisines au même format : `58-1-numero-facture-textes-tests.md`, `58-3-manuel-guide-api-ecarts.md`.

**Ce que la fiche affirme et que tu dois vérifier toi-même, avant tout le reste** : (1) **aucune des dix issues n'est déjà réglée** — pour chacune, le test « manquant » manque encore à `90192c7b3`
(cherche-le : `grep -rn` dans le fichier nommé, le binaire voisin, les tests unitaires du module) ; seules #732-1 et #1433-2 sont à moitié couvertes ; (2) la mutation que chaque AC nomme **survit bien** au test existant
(raisonne le chemin : quel test existant la tuerait ?) ; (3) la story ne touche **aucun** fichier de `crates/*/src/*` (D1, y compris le doc-comment de #732-2) ; (4) D5 : l'ordre réel des verrous ne se lit pas à
travers MariaDB, le test tient le plan ; (5) D6 : la validation lit-elle le **réglage** (donc un compte de charge sans rôle, posé par SQL aux deux réglages, est refusé deux fois) ? ; (6) D7 : l'avoir sur une créance retypée
passe-t-il (`routes/credit_notes.rs`, `credit_notes` du dépôt) et sur une TVA due retypée ? Ne prends rien pour acquis.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code* (tests), *Test Locally First* (gates, idefix), *Review Iteration Rule* (dont *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*,
   *Inventorier les sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter*), *Priorités des défauts* (règle du 2026-11-26 : vérifier qu'une issue n'est pas déjà couverte), *Règle de splitting préventif*, *Fusions et fichiers communs*,
   *Plafonds mémoire*, *Worktrees et agents*.
2. Le fichier d'epic (section 58-2) et la fiche en entier.
3. Les issues, **en lecture seule**, **au moins dix secondes entre deux requêtes `gh`**, pas plus de dix requêtes : `gh issue view N --json number,title,labels,milestone,body,comments`
   pour N dans 592, 732, 736, 1060, 1062, 1063, 1064, 1066, 1127, 1433.
4. Les fiches et revues sources, en lecture seule : `_bmad-output/implementation-artifacts/35-3-*.md`, `35-5a-date-du-jour-horloge.md` (Change Log, A4), `/home/gcorbaz/devel/kesh-gate-logs/321a-review-p1-{B,E}.md`.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre contre le fichier d'epic** : les dix issues de la fiche sont-elles exactement celles de la section 58-2 ? L'écart déclaré (#732-4 est un test **frontend**, #592 aussi : D2) est-il fondé ?
  Le compte de modules (zéro de production ; quatre ensembles de fichiers de test) est-il juste au sens du `CLAUDE.md` (« un module est un crate ou un domaine dont le **code de production** change ») ? La règle de splitting
  est-elle respectée, et la story tient-elle en une journée ?
- **A2 — l'écart existe-t-il encore ? (une issue par issue)** : pour chacune des dix, la chaîne ou l'absence citée par le § *Contrôle de couverture* se vérifie-t-elle au fichier et à la ligne (`grep -nF`, ligne copiée) ?
  Une issue est-elle en réalité déjà réglée (un test voisin, un test unitaire de `kesh-db`/`kesh-core`, un test Vitest, un test E2E Playwright) ? **Cherche aussi côté Playwright** (`frontend/tests/e2e/`) et dans les tests
  d'intégration d'autres crates (`kesh-seed`, `kesh-report`). Les lignes que l'issue donne ont bougé : la fiche les relève-t-elle juste ?
- **A3 — la mutation survit-elle vraiment ?** Pour chaque AC, la mutation nommée est-elle réalisable (compile, change un comportement) **et** survit-elle aux tests existants ? Raisonne chemin par chemin ; pour AC7 (`account_type_in_words`),
  la mutation `Revenue => "charge"` est-elle atteignable (le message `[Expense, Revenue]` joint « charge ou produit ») ? Pour AC8, retirer `.then(a.setting.cmp(&b.setting))` change-t-il le résultat quand les deux entrées ont même numéro
  **et** même identifiant ? Pour AC13, l'ordre version / garde de parent est-il celui du code (`accounts.rs:1224`, `:1266`) ?
- **A4 — les tests proposés sont-ils vrais par construction ?** Pour chaque AC : l'assertion compare-t-elle deux valeurs issues du **même extracteur** (« vrai par construction ») ? C'est le risque de D3 (table de clés) et de D5
  (séquence de numéros) : la table de D3 est-elle bien **écrite en littéraux** et indépendante du code ; la séquence de D5 dépend-elle de `WriteOffNature::ALL` des deux côtés ? Un test peut-il rester vert alors que la garde qu'il
  prétend tenir est défaite (`tests-qui-prouvent-moins`) ? Le mock « rend la clé » de D3 change-t-il l'assertion d'un autre cas de `JournalEntryForm.edit.test.ts` ?
- **A5 — le montage de chaque test existe-t-il ?** D6 : `new_account` + `UPDATE company_invoice_settings` aux deux réglages, sans rôle — la validation de vente (`DesignatedAccountsSnapshot`, `check_written`) les juge-t-elle ? Le tri
  `(numéro, identifiant, réglage)` donne-t-il bien `receivable` puis `vatPayable` ? AC4 : deux sociétés dans le même `#[sqlx::test]` (`companies::create` deux fois, plan PME de chacune : contrainte d'unicité sur `accounts` ou
  `singleton_role` ?) ; la `version` de `company_invoice_settings` bouge-t-elle à la désignation (le doc-comment dit « n'incrémente pas `version` ») ? AC10 : `POST /api/v1/credit-notes` sur une facture dont la créance est retypée,
  sans exercice clos : 201 ? AC9 : `GET /api/v1/supplier-invoices/{id}` rend-il `status` et `internalNumber` aux noms de la fiche ?
- **A6 — #1127 (campagne de mutations)** : la fiche demande de jouer M3/M7 sur chaque site restant. La liste des sites se relève-t-elle par le `grep` de la fiche (rejoue-le, compte les sites d'AC2 et d'AC3 de la 35-5a) ?
  La procédure (mutation en local, restauration `git checkout --` **et** `touch`, `git diff --stat -- 'crates/*/src/*'` vide avant commit, pathspec non muet) est-elle sûre pour le travail parallèle d'autres agents (le dépôt principal est en lecture
  seule, le worktree est celui de la story) ? Le coût d'une compilation de mutation tient-il dans les plafonds mémoire (`mem-guard`, un `-E 'binary(jour_suisse_nuit_e2e)'`) ? Faut-il scinder #1127 (règle de splitting, une issue = une release) ?
- **A7 — #732 en quatre points, une issue, une release** : la fiche garde-t-elle les quatre points dans la même story (`Une issue, une release`) ? Le point 2 (D5) est-il fermable par un test qui ne tient que le plan ? L'angle mort
  (ordre des appels SQL) est-il écrit comme tel ? Le point 4 (test frontend) change-t-il quoi que ce soit à ce que `trois clos` tenait déjà (la fiche dit : étroit) — est-il à retenir ou à écarter avec la preuve ?
- **A8 — manuel, PDF compris** (le prompt d'une passe sur du code visible doit nommer le manuel : cette story n'a **aucun** texte visible, mais ses tests **figent des phrases** que le manuel cite) : les phrases assertées par AC7
  (« type de compte incompatible (attendu : actif|passif|charge ou produit) », libellés `Compte créance`, `Compte TVA due`…) sont-elles celles du manuel utilisateur (`docs/manual/fr/user-manual.tex`, § *Rôles des comptes*, « Contrôle préventif »,
  `:1619` environ) **et du PDF** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, aplati avant de greper) ? Le manuel dit-il que l'avoir « n'est pas soumis à ce contrôle » et « relit la TVA due … sans la contrôler »
  (AC10, D7) — le code le confirme-t-il ? Une phrase du manuel contredit-elle un test que la story ajoute ? Les PDF ne sont pas touchés : la fiche le dit-elle ?
- **A9 — gates et fichiers communs** : le gate prévu est-il celui du `CLAUDE.md` (gate ciblé par binaire, gate complet au dernier commit **sur idefix** par `gate-distant.sh`, exception `kesh-db` : ici non déclenchée, est-ce juste ?) ?
  Le tableau des fichiers communs (Epic 53 sur `designated_account_type_e2e.rs`, 58-4/58-5 sur `JournalEntryForm.*`, epics de contributeurs sur `i18n-keys.test.ts`) est-il juste (`git diff --name-only 90192c7b3...origin/<branche>` sur les
  branches `epic/53*`, `epic/58*` ouvertes) ? `sitesTotal` de `i18n-keys.test.ts` : un nouveau fichier de test qui **mocke** `i18nMsg` change-t-il le moissonneur (le test lit-il les `*.test.ts` ?) — **rejoue la règle de périmètre** de
  `dansLePerimetreDeFichier` (`frontend/src/lib/shared/i18n-harvest.ts`) sans exécuter Vitest.
- **A10 — priorité, décisions, issues à ouvrir, questions** : D1 à D8 ont-elles chacune leur option écartée, motivée ? Les questions Q1 à Q4 sont-elles les bonnes, et la fiche tranche-t-elle ce qui lui revient ? La *Note de version*
  nomme-t-elle une release, jamais un epic, avec `refs #N` et **sans** mot-clé de fermeture (`git log` du commit de fiche : `grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`) ? Une donnée réelle de Guy (IBAN, solde, ligne de relevé) figure-t-elle
  dans la fiche : elle ne doit pas.
- **A11 — charge et ordre** : « environ une journée » est-il crédible (dix tests, 14 mutations d'AC1, la campagne d'AC12) ? L'ordre des tâches laisse-t-il la campagne de mutations **en dernier**, sur un arbre propre ? Chaque tâche est-elle un commit
  distinct avec `refs` ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. **Commandes interdites** : `cargo`, `npm`, `npx`, `node`, `vitest`, `docker`, `mariadb`, `sqlx`, `curl` vers un serveur, `ssh`,
tout `scripts/*` — en particulier `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`, `kesh-gate-logs/serveur/gate-distant.sh` —,
`git commit`, `git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git rebase`, `git worktree`, `git add`. Aucun `gh` qui écrit (`issue create/edit/comment/close`, `pr …`, `api` avec `-X` ou `-f`), et au moins dix secondes
entre deux requêtes `gh`. Rien sous `~/kesh/`. **Jamais `pkill`, `killall` ni `kill` par motif.** Tu **ne mutes aucun fichier** pour « vérifier une mutation » : tu la raisonnes (ou tu la fais jouer par le développeur, en le demandant au rapport).
`python3 -I` en lecture pour recompter et extraire un contexte est admis (`grep -E` avec de longs quantificateurs échoue sur les chaînes accentuées : préférer Python). `pdftotext` en lecture est admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e582/validation-p1.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`V1-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction proposée ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les prémisses de la fiche que tu as **confirmées** au code (une ligne chacune, ligne citée), et celles que tu as **réfutées**.

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
