# Prompt — revue de code P1 de la Story 32-10a-i (les tables des prompts MCP, leur dépôt, l'amorçage des standards et la sauvegarde), trois lentilles, Sonnet

Tu relis le développement de la **partie i de la Story 32-10a** de Kesh (comptabilité suisse, Rust/Axum, MariaDB
10.11) : *les deux tables des prompts du serveur MCP (`mcp_prompts`, `mcp_prompt_versions` — un historique qui ne se
réécrit ni ne s'efface), leurs entités, leur dépôt complet (lectures, écritures sous verrou de société, approbation de
la dernière version seulement, plafond des propositions d'agent), les deux prompts standard en code
(`kesh-developpement`, qui porte les règles de Guy sur les données réelles, et `kesh-assistant`), leur amorçage
idempotent au démarrage et à la fin de chaque point d'entrée qui crée une société, le classeur SQL `sql_lexical` sorti
de `post_restore.rs` et les gardes lexicales des écritures, et la place des deux tables dans la sauvegarde
(`TABLES_TO_TRUNCATE`, export global, remise à zéro, restauration).* **Rien n'est encore servi à un agent** : le
catalogue est la 32-10a-ii, les écritures par l'agent et l'administrateur la 32-10a-iii.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a-dev`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-10a diff origin/main...HEAD`. Les commits de **code** de la 32-10a-i suivent le
commit de fusion `56af24f2` (fiche + `origin/main` à `cb432098`) ; ce qui précède `56af24f2` dans ce diff est la fiche
et ses validations (hors revue de code, sauf pour la lire). Langue du rapport : français. Modèle : **Sonnet**. Contexte
frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Hors périmètre** : le prédicat des caractères refusés, la validation d'un contenu, le catalogue servi
(`prompts/list`, `prompts/get`), la marque d'un prompt d'agent (**32-10a-ii**) ; l'outil `proposer_prompt`, les routes
de gestion, le journal, l'audit (**32-10a-iii**) ; l'écran et la section du manuel *Les prompts du serveur MCP*
(**32-10b**). Un défaut que la 32-10a-i ne touche pas est un finding **antérieur**, à classer comme tel. Mais **un besoin
de la ii ou de la iii que le dépôt ne couvre pas est un finding de la 32-10a-i** : « la 32-10a-iii ne touche pas
`kesh-db` » (fiche, § *Le découpage*, table de correspondance besoin → fonction) — y compris les amendements D1, D1
étendu, D1 champs du détail et D2 inscrits au § *À recontrôler à l'étape 0* (noms de l'auteur, du décideur, de la clé
et sa révocation, `standardDiffere`, `numeroEnVigueur`, texte du standard du code).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` — § *Ce que le code dit*, § *Le découpage*,
**Partie i** (AC-i1 à AC-i16), *Tâches 32-10a-i*, *Tests et mutations prévus* (Mi1 à Mi28, Mi23 retirée), *Gates*,
*Ce que les gardes ne voient pas*, *Dev Notes* (§ *Schéma*, *Les textes standard*), le § *À recontrôler à l'étape 0*
en entier, et le **Dev Agent Record** et le **Change Log** de la 32-10a-i (étape 0, choix, mutations, gates, décomptes) —
puis le registre `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md`, **C-32-10a-60 et suivants**
(choix du développement) et C-32-10a-6, 7, 8, 9, 13, 20, 27, 28, 32, 33. Le Dev Agent Record déclare des gates, des
mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux
`/home/gcorbaz/devel/kesh-gate-logs/3210a-i-*` (lecture seule). Aucun contrôle visible ne change : pas de capture
attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au rapport ; il
  ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont `mut-3210a.py` et `run-mut-3210a.sh`) ; ni `make`, ni
  `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni
  `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `sha384sum`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_3210A`). Ne lis rien sous `~/kesh-test/`.

## Les axes obligatoires, pour chaque lentille

1. **La politique des migrations (P1 à P8 de `CLAUDE.md`)**, chacune avec sa preuve : la migration
   `crates/kesh-db/migrations/20261027000001_mcp_prompts.sql` est-elle horodatée **après** la dernière de `origin/main`
   (`git ls-tree --name-only origin/main crates/kesh-db/migrations/ | tail -2`) ; DDL seul (aucun `INSERT`, `UPDATE`,
   `DELETE`, `REPLACE` décommenté), aucun `DROP`/`RENAME`/`MODIFY`/`CHANGE`, pas de bump de
   `kesh_version_min_required` ni de version Cargo (P1-P3) ; **P5** : la ligne d'audit
   (`docs/migrations-idempotence-audit.md`) **et** les compteurs recomptés depuis la source (`ls
   crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`, la partition
   `yes`/`tracked-by-sqlx`/`no` et ses **deux** sites du total) ; **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to"
   crates/` et chaque site inspecté, en particulier `migrations_upgrade_path.rs` — **greper la valeur** (`grep -nE
   "\b(85|86|51|52)\b" crates/kesh-db/tests/migrations_upgrade_path.rs`) et trier chaque ligne (frontière toujours 34) ;
   **P7** : la migration n'écrit rien, donc ni registre ni exemption — vérifie que rien n'a été ajouté à
   `POST_RESTORE_BACKFILLS` ni à `EXEMPT_MIGRATIONS` ; **P8** : la somme de la ligne ajoutée à
   `crates/kesh-db/migrations.sha384` égale `sha384sum` du fichier, et cette ligne est la dernière modification de la
   branche sur la migration (`git log --format=%h -- crates/kesh-db/migrations/20261027000001_mcp_prompts.sql
   crates/kesh-db/migrations.sha384`).
2. **Le squash et le schéma réel** : les deux tables du squash (`crates/kesh-db/test-schema/0001_schema_squash.sql`)
   disent-elles la même chose que la migration (chaque `CHECK`, la collation `utf8mb4_nopad_bin` des énumérations et du
   nom, l'ancre `\z` du motif du nom, les colonnes **sans** `DEFAULT` `is_standard` et `nature`) ? Les tests du dépôt
   montent le squash : une contrainte présente dans la migration et absente du squash, ou l'inverse, les ferait-elle
   passer à vide ?
3. **Chaque dépendance lue à la source.** Toute affirmation du code, de ses commentaires, du registre ou du Dev Agent
   Record sur MariaDB 10.11 (le `$` d'un `REGEXP` qui s'apparie avant un `\n` final, `\z`, `LIKE … COLLATE`, `FOR
   UPDATE` sur une jointure `LEFT JOIN` à sous-requête, `SELECT … FOR UPDATE` d'une ligne de `companies` comme verrou de
   sérialisation, `innodb_lock_wait_timeout` de session) ou sur `sqlx-mysql` 0.8.6 (décodage d'un `BOOLEAN`, d'un `CASE`
   rendu en `INT`, d'un `MAX(number)` en `i32`, `#[sqlx(flatten)]`) se vérifie — pour `sqlx`, dans
   `~/.cargo/registry/src/*/sqlx-mysql-<version>/` (version lue dans `Cargo.lock`, ligne citée) ; pour MariaDB, dis ce qui
   ne se vérifie pas en lecture seule et ce que les tests éprouvent à sa place.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-db/src/repositories/mcp_prompts.rs`** — chaque lecture est-elle scopée par société ? Chaque écriture
   prend-elle **d'abord** `SELECT id FROM companies WHERE id = ? FOR UPDATE`, puis le prompt `FOR UPDATE`, puis `MAX(number)
   + 1` ? Ce qui se refuse sans lire la base est-il refusé **avant** le verrou ? `NUMERO_EN_VIGUEUR_SQL` est-elle la
   **seule** définition de la version en vigueur ? `decider` : l'ordre des refus (version attendue, version existante,
   origine, état, dernière) et ses conséquences ; l'`UPDATE` gardé par `AND state = 'pending'` dont le nombre de lignes
   n'est pas relu — défaut ? `inserer_version` : la `version` du prompt **non** incrémentée à la création, incrémentée
   ailleurs — cohérent partout (y compris l'amorçage) ? `amorcer_standards` : une transaction par société, refus d'un
   standard invalide **avant** toute écriture, société disparue entre la liste et le verrou, rapport compté ; la
   comparaison de contenu **en Rust**. `en_vigueur_par_nom` : la comparaison de nom sous la collation de la table, et le
   refus d'un nom non conforme avant toute lecture.
2. **`crates/kesh-db/src/entities/mcp_prompt.rs` et `mcp_prompt_standards.rs`** — décodage strict des trois
   énumérations (la macro `enum_texte!`) ; `Debug` manuel des types porteurs de texte (patron C-32-8-40) : un chemin par
   lequel une description ou un texte serait recopié dans un `Debug`, un `Display` d'erreur ou un évènement `tracing` ?
   `differe_du_standard` et `contenu_attendu` : un prompt standard sans version approuvée ; un standard retiré du code ;
   la casse. Les textes standard : continuations `\` qui laisseraient un blanc doublé ou mangeraient un saut de ligne
   voulu ; la forme `{"nonFiable": "…"}` nommée dans les deux.
3. **`crates/kesh-db/src/sql_lexical.rs` et `post_restore.rs`** — `split_statements` déplacée **sans changer de
   comportement** (compare l'ancien corps, `git show origin/main:crates/kesh-db/src/post_restore.rs`) ; `jetons`,
   `tables_nommees`, `est_ecriture`, `colonnes_du_set` : une forme d'écriture des tables des prompts qui échapperait
   (alias, nom qualifié, `WITH`, `INSERT … SELECT`, littéral à continuation, `r#"…"#`, commentaire de bloc dans un
   littéral) ; un faux rouge plausible.
4. **Les appels d'amorçage** (`main.rs`, `routes/onboarding.rs` — `set_language`, `reset` —, `routes/test_endpoints.rs`
   — `seed_handler`, `reset_handler`) : chacun **après** la validation de la transaction qui crée la société ? non
   fatal là où il le doit (démarrage, `set_language`, remise à zéro), `500` aux deux préréglages de test ? Un échec
   journalisé porte-t-il un texte de prompt ? Les doc-comments que ces appels rendent faux sont-ils corrigés ?
5. **Les tests** (`mcp_prompts_repository.rs`, `mcp_prompts_lexical.rs`, `prompts_amorcage.rs`, et les montages de
   `onboarding_audit_e2e.rs`, `test_endpoints_e2e.rs`, `admin_backup_e2e.rs`, `admin_full_export_e2e.rs`) — un test
   peut-il passer **à vide** : un contre-exemple de contrainte refusé pour une **autre** raison que la contrainte visée
   (le test ne cherche pas le nom de la contrainte dans le message : est-ce un défaut ?), une égalité qui compare deux
   valeurs sorties de la même source (`prompts_de_societe` contre `prompts_standard_seuls`), un test de concurrence qui
   passerait sans attendre, une ligne d'aller-retour égale parce que l'import ne l'a pas touchée ?

## Lentille E — Edge cases

1. **Les verrous** : deux amorçages concurrents (deux instances au démarrage, ou un démarrage et une remise à zéro) —
   peuvent-ils créer deux fois le même standard, ou se bloquer ? Un amorçage pendant `reset_demo` (qui tient `companies`
   puis `audit_log`) ? Une proposition pendant un amorçage ? Une société supprimée par la réparation #542 entre la liste
   et le verrou de l'amorçage ? La réparation #542 face à une société provisoire qui a reçu ses standards (fiche, § *Ce
   que les gardes ne voient pas* ; choix du développement à ce sujet).
2. **Le plafond** : compté sous le verrou de société — un plafond `0` ; des versions `pending` d'un prompt **standard**
   posées par une restauration (comptent-elles ?) ; le test de concurrence, et ce qu'il prouve exactement.
3. **Les décisions** : approuver une version qui est la dernière mais dont une version **plus ancienne** est encore à
   valider (que devient l'ancienne ?) ; refuser puis proposer de nouveau ; deux propositions du même nom neuf dans deux
   transactions.
4. **L'amorçage** : un standard dont le nom est pris par un prompt non standard (impossible par le `CHECK` : vérifie) ;
   un prompt standard dont la version en vigueur est un retrait **standard** et que le code rétablit ; un prompt
   standard sans aucune version approuvée (restauration forgée) ; un standard de test au nom sans préfixe ; un standard
   dont le texte dépasse 16 000 caractères.
5. **La sauvegarde et la remise à zéro** : l'ordre des deux tables à `TABLES_TO_TRUNCATE` (enfants → parents ; FK vers
   `mcp_keys` et `users`) ; une version d'agent dont la clé appartient à une autre société ; la remise à zéro qui vide les
   tables des prompts **puis** réamorce hors de sa transaction — que voit un appel concurrent entre les deux ? L'import
   n'amorce pas : un standard altéré dans la sauvegarde est servi jusqu'au démarrage suivant — écrit où ?
6. **Les mutations** (`/home/gcorbaz/devel/kesh-gate-logs/3210a-i-mut-*.log`) : chacune tuée par une **assertion
   nommée** et pour la raison visée (lis le journal, pas seulement le verdict) ? Une mutation plausible **non jouée**
   survivrait-elle — l'historique sans filtre de société, la synthèse qui compte les `pending` de toute la société, une
   version standard créée `pending`, `creer_prompt_admin_in_tx` qui ne contrôle plus le nom pris, l'amorçage hors du
   verrou de société, `en_vigueur_par_societe` qui sert un retrait ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-i1 à AC-i16, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-i3** : chaque contrainte de la fiche, testée par un cas qui ne peut être refusé que par elle ; la liste comptée.
- **AC-i5** : la liste close des fonctions, leurs signatures (`_in_tx`, transaction de l'appelant), les erreurs
  énumérées (« aucune chaîne libre »), et la table de correspondance du § *Le découpage* — puis les amendements D1 :
  le dépôt rend-il **tout** ce que la route de détail de la iii doit rendre, sans que la iii touche `kesh-db` ?
- **AC-i9** : les règles de Guy présentes, le témoin, et la forme `{"nonFiable": "…"}` (étape 0, précision de
  l'orchestrateur).
- **AC-i11** : la liste fermée des six créateurs, la frontière du code de production (témoins `bootstrap.rs`), un appel
  par point d'entrée et son test d'exécution, la garde `amorcage_au_demarrage` ; les angles morts écrits.
- **AC-i12** : `TABLES_TO_TRUNCATE`, `RESET_PRESERVED_TABLES` inchangée, `reset_partition_is_guarded_by_the_schema`
  **non modifié** (`git diff origin/main...HEAD -- crates/kesh-db/src/backup.rs`), le test de la remise à zéro,
  `EMPTY_BEFORE_RESET` inchangée, l'export global (`grep -nwE "douze|treize|quatorze|42|43|44"
  crates/kesh-api/src/exports/global.rs`, chaque ligne triée) et le compte de l'export d'installation et sa généalogie.
- **AC-i14** : les deux gardes, leurs témoins rouges et verts (factices **et** existants), la mesure sur l'analogue
  `mcp_keys`.
- **Les choix C-32-10a-60 et suivants** : chacun est-il justifié ; l'un d'eux contredit-il un AC, la fiche ou une décision
  de Guy ?
- **Le Dev Agent Record** : chaque nombre (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son
  périmètre ?

## Axe du manuel

La 32-10a-i écrit trois textes que ses tables rendaient faux (AC-i16) : le manuel d'administration (ce que porte le
fichier de sauvegarde, le nombre de tables ajoutées par la 0.16.0, la remise à zéro), le manuel utilisateur (ce que
l'export ne contient pas) et le CHANGELOG. Vérifie-les **contre le code**, PDF compris :
`grep -nE "prompts du serveur MCP|cinq tables|en ajoute cinq" docs/manual/fr/*.tex CHANGELOG.md`, puis les **PDF
aplatis** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(prompts du serveur
MCP|cinq tables).{120}"`, idem `user-manual.pdf`). Nomme tout site qui compte les tables de la sauvegarde ou du serveur
MCP, ou décrit la remise à zéro ou la restauration, et que la 32-10a-i rend faux sans l'avoir corrigé (dont le
paragraphe de la démonstration du manuel utilisateur, l. ≈190).

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3210a-i-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/3210a-i-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-i-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-10a-i ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **La vérification de P5 à P8**, point par point, avec les sorties de commande.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
