# Prompt — validation P2 de la fiche 58-3 « Manuel et guide de l'API : écarts avec le code » (#730, #748, #767, #814, #1061, #1067, #1074, #1076, #1087, #1405), passe complète

Tu valides la **fiche** de la story **58-3** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), premier epic de **nettoyage** (Epic 58,
dix issues P3/P4 de documentation), release visée v0.21.0. Kesh tient les livres réels du Project Lead. La story ne change **aucun comportement, aucune règle
métier, aucune migration, aucun contrat d'API** : elle corrige des phrases que le code dément. Deuxième passe : **complète**, contexte frais, modèle **Sonnet**
(la P1, Opus, a rendu 0/1/3/12 ; la P3 éventuelle sera Opus). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-58-3`, branche `story/58-3-manuel-guide-api-ecarts`, partie de `epic/58-nettoyage` à `ee684a69f`, fiche remédiée de la P1 au commit de tête de la branche. Fiche :
`_bmad-output/implementation-artifacts/58-3-manuel-guide-api-ecarts.md` (statut `drafted`). Fichier d'epic :
`_bmad-output/planning-artifacts/epic-58-nettoyage.md` (section 58-3 ; « Fichiers et modules touchés » ; « Recoupements »).

**Ce que la fiche affirme et que tu dois vérifier toi-même, avant tout le reste** : (1) **aucune des dix issues ne sort** — pour chacune, le code est juste et le texte
diverge ; (2) quatre des sept sites que le fichier d'epic donne pour #767 sont des **valeurs de la variable `KESH_LOG_FILE_PATH`, justes**, et ne doivent pas être « corrigés » ;
(3) le CHANGELOG ne porte pas la phrase de #1061 ; (4) les deux justifications de `post_restore.rs` (#748) se réécrivent **sans** changer le fondement `PerishableSince` (D2) ;
(5) la release v0.20.0 (branche `docs/release-0200`) ne recoupe aucun site de la story, sauf `docs/api-external.md:1415-1416`, voisin de son tableau des erreurs. Ne prends rien pour acquis.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Documentation à synchroniser* (documents pour l'utilisateur : numéros de release, jamais d'epic ; PDF jamais commités dans `epic/…` ni `story/…` ;
   `CHANGELOG.md` non écrit dans les stories), *Review Iteration Rule* (dont *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*, *Inventorier les
   sites NON RÉSOLUS*, *Propagation post-patch*, *Recompter*), *Priorités des défauts* (dont la règle du 2026-11-26 : vérifier qu'une issue n'est pas déjà couverte), *Migration breaking
   policy* (P5, P7, P8), *Règle de splitting préventif*, *Fusions et fichiers communs*.
2. Le fichier d'epic (section 58-3) et la fiche en entier.
3. Les issues, **en lecture seule**, **au moins dix secondes entre deux requêtes `gh`**, pas plus de dix requêtes dans la passe : `gh issue view N --json number,title,labels,milestone,body,comments`
   pour N dans 730, 748, 767, 814, 1061, 1067, 1074, 1076, 1087, 1405.
4. La branche de la release, **en lecture seule** : `git diff -U0 ee684a69f...origin/docs/release-0200 -- docs/manual/fr/admin-manual.tex docs/manual/fr/user-manual.tex docs/api-external.md CHANGELOG.md README.md`.

## Axe 0 — la remédiation de la P1 (à exercer en premier)

Le rapport de la P1 est `/home/gcorbaz/devel/kesh-gate-logs/e583/validation-p1.md` (lis-le en entier). **Axe 0 : chaque finding de la P1 (V1-1 à V1-16) est-il réellement remédié dans la fiche, sans défaut neuf né de la correction ?**
Pour chacun : relis le passage corrigé (le Change Log « validation P1 remédiée » les liste), **recontrôle au code** la correction (`grep -nF`, ligne copiée), et cherche ce que la correction a rendu faux ailleurs dans la fiche (propagation post-patch : une date, un nombre de fichiers, un compte, un renvoi). Points à contrôler en particulier :
V1-1 : la sortie de fenêtre de 20260909000001 et 20260915000001 est-elle **partout** `20260917000001` (D2, AC2, AC3, I1, prémisse 3, § code), et `20261025000001` n'est-elle gardée que pour la ligne 91 ? Reste-t-il une occurrence fausse (`grep -n 20261025000001` sur la fiche) ?
V1-2 : AC4 et AC12 s'accordent-ils (deux fichiers de `crates/`, `config.rs` compris ; modules : deux crates) ? Le commentaire de `.env.example:330-334` peut-il être réécrit sans rougir `controle_raisons_valeurs` ?
V1-3 : D6 et AC9 ne citent-ils plus aucune saisie mal groupée, et le motif de l'option écartée est-il juste ? V1-4 : la normalisation de l'apostrophe est-elle écrite et les comptes « avant » au PDF non nuls ?
V1-7 : les prémisses sur `prepare-release.sh`, `perishable_exemptions.rs` et `admin_full_import_e2e.rs` sont-elles justes ? V1-8 : le nouveau compte d'AC2 (3 → 1) est-il exact (`grep -cF` à `ee684a69f`) ? Les autres décomptes de la fiche sont-ils encore exacts ?
Un finding mal remédié ou un défaut neuf compte comme finding de cette passe (sévérité propre).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — périmètre contre le fichier d'epic** : les dix issues de la fiche sont-elles exactement celles de la section 58-3 ? La fiche déclare trois écarts (`already-booked.ts` hors périmètre, manuels de/en/it inexistants, sites de #767 resserrés) ;
  chacun est-il fondé (`ls docs/manual/de docs/manual/en docs/manual/it`, `grep -rn already-booked` dans les dix issues) ? Le compte de modules (un : `kesh-db`) est-il juste au sens du `CLAUDE.md` ?
  Une story de « zéro code exécutable » peut-elle toucher `post_restore.rs` (D2) sans violer la règle de l'epic ? La conclusion de D2 (rester dans la story, gate complet `kesh-db`, E2E proposé non rejoué) est-elle juste ?
- **A2 — l'écart existe-t-il encore ? (une issue par issue)** : pour chacune des dix, **la chaîne copiée par la fiche existe-t-elle** au fichier et à la ligne cités (`grep -nF` de la chaîne, ligne copiée), et **au PDF** aplati
  (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`, idem `user-manual.pdf`) ? Le verdict « gardée » tient-il ? Une issue est-elle en réalité déjà corrigée, ou le code lui donne-t-il tort ?
- **A3 — le code prouve la phrase juste** : relis chaque `chemin:ligne` du § *Ce qui se lit au code* (ligne copiée). #730 : `services/rapports.rs` (`report.generated`, `report.exported`), `routes/exports.rs` (`exports.global`), `auth/api_key.rs`
  (`touch_last_used`) ; l'export global est-il bien atteignable par une clé `read` ? #748 : `last_table_creating_migration()` est-elle bien ≥ `20261025000001` (lis `TABLES_TO_TRUNCATE` et `ABSENCES_ADMISES_EPIC_37` : le comptage de « table applicative » exclut-il `mcp_keys` ?) ;
  20260909000001 et 20260915000001 sont-elles bien hors fenêtre ? #767 : `tracing-appender` 0.2.5 (`~/.cargo/registry/src/*/tracing-appender-0.2.5/src/rolling.rs`) nomme-t-il bien `kesh.<date>.log` ? #814 : `textes.rs`, `sortie.rs`, `mcp_contrat.rs` ;
  #1061 : `errors.rs` (tri par numéro-chaîne) ; #1067 : `invoices.rs` (le défaut n'est contrôlé que si une ligne **chiffrée** sans compte le suit — la précision « chiffrée » de D5 est-elle juste ?) ; #1074/#1076 :
  `format.ts`, `amount-ambiguity.ts`, `opening-amount-cases.json` (`1.5'` accepté ? `1'234,567` clair par ses quatre chiffres ? `12'3.456` et `1 2.345` clairs *grâce* au groupement ?) ; #1405 : le formulaire de contact n'a-t-il vraiment aucun champ de compte ?
- **A4 — les phrases justes proposées sont-elles justes** : la fiche ne donne pas le texte final de chaque phrase, elle donne ce qu'elle doit dire (AC1 à AC11). Cherche ce qu'une réécriture conforme à l'AC écrirait **faussement** : « aucune modification des livres »
  (le PDF figé d'une facture est-il une modification des livres ?) ; la liste des trois écritures des clés `read` est-elle complète (`last_used_at`, journal des rapports, export global — une quatrième : `HEAD` ? un refus 403 écrit-il ?) ;
  `kesh.<AAAA-MM-JJ>.log` vaut-il pour la rotation horaire (`kesh.<AAAA-MM-JJ-HH>.log`) et `never` (`kesh.log`) ? La commande `tail -f ./log/kesh.<date>.log` désigne-t-elle un fichier existant le jour où elle est lue ?
- **A5 — inventaire par le symptôme, ensemble clos** : rejoue chaque commande de l'inventaire de la fiche (`grep -rnE`, `grep -nF`, `python3 -I` de contexte) sur **tout** le dépôt hors `_bmad-output/`, `node_modules/`, `target/`, `.git/`, **et sur les PDF aplatis** ;
  cherche ce qu'elles **omettent** : une autre formulation du même défaut (« lecture seule » du rôle Consultation dans le guide de l'API, la brochure `marketing-brochure.tex`, `docs/user-guide/`, `website/`, `README.md` ; « ne rend que » / « en attente » ailleurs ;
  un autre « Elle » orphelin au manuel utilisateur ; une autre mention du fichier de log dans `docs/` ; un autre « milliers déjà » ; une autre liste de champs de contact). Chaque site est-il dans un AC, écrit « sans objet » avec sa raison, ou omis ? Les angles morts écrits sont-ils vraiment morts ?
- **A6 — AC constatables** : chaque AC dit-il une chaîne `grep -cF` avant/après, un fichier, une ligne de code ? Les comptes « avant » de la fiche sont-ils justes (relance-les à `ee684a69f`) ? Un AC est-il vrai par construction (les deux côtés du même extracteur) ?
  La « preuve du rouge » par grep avant/après suffit-elle pour une story sans comportement, ou faut-il une garde de texte (la fiche l'écarte : est-ce justifié) ? L'AC5d (Overfull) est-il constatable : la fiche cite `Overfull \hbox (49.6949pt too wide) in paragraph at lines 3805--3807` ;
  **reconstruis `admin-manual.pdf` dans une copie** (voir Interdits) et retrouve-le.
- **A7 — fichiers communs avec la release v0.20.0** : le tableau de recoupement est-il juste ? Recompte les hunks de la branche `docs/release-0200` sur les cinq fichiers ; la fiche dit « aucun hunk à moins de 30 lignes » pour `admin-manual.tex`, « 33 hunks » pour
  `api-external.md` avec `:1412` voisin de `:1415` et `:1425` voisin de `:1416`. Un site de la story est-il **modifié par la release** (auquel cas sa chaîne « avant » n'existerait plus après la reprise) ? La reprise de `main` après la release (D8) suffit-elle ? Le CHANGELOG
  hors story (D3) est-il conforme à la règle du `CLAUDE.md` et à la déclaration du kickoff ?
- **A8 — gardes et gates** : `textes_coherents`, `mcp_matrice_manuel`, `configuration_transmise` lisent-ils un site que la story modifie (blocs marqués du manuel MCP, `.env.example:330-334`, tableau de `kesh-style.sty`) ? Les tests de `post_restore.rs` cités
  (`every_exemption_declares_a_coherent_basis`, `exemptions_claiming_out_of_window_really_are_out_of_window`, `every_data_backfill_migration_is_triaged`, `registry_entries_are_within_import_window`) rougiraient-ils pour une réécriture conforme à D2 ?
  `scripts/prepare-release.sh` lit-il les chaînes que D2 impose de garder ? Le gate prévu pour le commit de code est-il celui du `CLAUDE.md` (exception `kesh-db`) ?
- **A9 — manuel, PDF compris** : la story est une story de manuel. Pour **chaque** phrase corrigée, le PDF committé (`docs/manual/fr/*.pdf`) dit-il la même chose que le `.tex` (la fiche relève les comptes d'occurrences) ? Existe-t-il une **phrase voisine que la
  correction rend contradictoire** (par exemple `admin-manual.tex:3621`, `:3967`, `:2999`, ou une phrase de `user-manual.tex` sur le solde d'ouverture ou le rejet) ? Les PDF ne sont pas commités : la fiche le dit-elle partout ?
- **A10 — priorité, décisions, issues à ouvrir** : D1 à D8 ont-elles chacune leur option écartée, motivée ? Les trois issues I1 à I3 sont-elles fondées au code, et ne doublonnent-elles aucune issue existante (`gh issue list --state all --search`, trois requêtes au plus) ? La *Note de version* nomme-t-elle une release,
  jamais un epic, avec `refs #N` et **sans** mot-clé de fermeture ? L'erratum du CHANGELOG remplace-t-il des phrases réellement présentes (`CHANGELOG.md:132-133`) ? Une donnée réelle de Guy figure-t-elle quelque part (IBAN, solde, ligne de relevé) : elle ne doit pas.
- **A11 — charge et ordre** : la charge (une demi-journée) est-elle crédible pour dix remplacements, un aplati, une construction de manuel, un gate Rust complet ? L'ordre des tâches laisse-t-il `post_restore.rs` en dernier commit de code ?

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. **Commandes interdites** : `cargo`, `npm`, `npx`, `node`, `docker`, `mariadb`, `sqlx`, `curl` vers un serveur, tout `scripts/*` — en particulier
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh` —, `git commit`, `git push`, `git checkout`, `git switch`, `git stash`, `git merge`, `git rebase`, `git worktree`, `git add`.
Aucun `gh` qui écrit (`issue create/edit/comment/close`, `pr …`, `api` avec `-X` ou `-f`), et au moins dix secondes entre deux requêtes `gh`. Rien sous `~/kesh/`.
**Exception pour la construction du manuel (A6)** : `make` et `xelatex` sont admis **uniquement** dans une copie hors dépôt (`cp -r docs/manual <scratchpad>/man && make -C <scratchpad>/man admin`), jamais dans le worktree ni sur le dépôt principal ; aucun PDF n'est copié vers le dépôt.
`python3 -I` en lecture pour recompter et extraire un contexte est admis (`grep -E` avec de longs quantificateurs échoue sur les chaînes accentuées : préférer Python).

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e583/validation-p2.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, et le **bilan de l'axe 0** (V1-1 à V1-16 : remédié / partiel / non remédié) ;
- chaque finding : identifiant (`V2-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction proposée ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les prémisses de la fiche que tu as **confirmées** au code (une ligne chacune, ligne citée), et celles que tu as **réfutées**.

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
