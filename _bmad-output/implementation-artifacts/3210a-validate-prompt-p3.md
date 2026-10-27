# Prompt — validation P3, Story 32-10a (les prompts MCP en base)

*Versionné le 2026-10-25. Passe **complète**, troisième sur huit (P1 Sonnet ×2 0/0/11/18, P2 Opus ×2 0/0/13/19,
remédiés — **en P2, dix MEDIUM sur treize étaient nés de la remédiation P1**). **Deux lentilles, R et F**, chacune en
contexte frais, **Sonnet** (modèle différent de la P2). Lecture seule stricte, **sans `gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a` (partie de `main` `d993106c` : 32-5a et 32-5b
fusionnées ; 32-5c, 32-5d, 32-6a, 32-6b **non fusionnées** — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (trois parties i, ii, iii ; **découpage confirmé** par
l'orchestrateur) ; registre : `epic-32-choix-autonomes-3210a.md` (C-32-10a-1 à 32 ; C-1, 8, 11, 12, 15, 17, 18, 28 annotés). **Rapports
de la P1 et de la P2**, à lire pour savoir ce qui a été remédié — **non pour les reprendre crus** :
`/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p{1,2}-{R,F}.md` ; les entrées « validation P1 » et « validation P2 »
du Change Log de la fiche disent ce qui a été fait de chacun. Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du
Project Lead*, *Règles de l'agent sur les données réelles*, § *32-10*, *Ce que le code dit déjà* point 9, *Réponses de
Guy*) ; spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 5.3.1 E1 et E2, § 5.3.2, § 5.3.3, § 6 ligne
*32-10a*, § 7 et ses réponses) ; `epic-32-choix-autonomes.md` (C-32-0-7, 9,
11, 12) ; fiche `32-5-cles-et-reglages-mcp.md` (parties a, c et d) ; **la fiche 32-6, en lecture seule dans un autre
worktree** : `/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` et
`epic-32-choix-autonomes-326.md` (AC-a13, a15 à a17, a21, a23, a24, a30, a31, AC-b1 à b4, l'inventaire N1 à N28, la
règle commune des gardes lexicales, C-32-6-23, C-32-6-24). Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe
ciblée*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une boucle de revue* et son
exception `kesh-db`, *Migration breaking policy* P1 à P8, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses
propres comptes rendus*, *Propagation post-patch*, *Priorités des défauts* — P2 : « opération enregistrée mais annoncée
en échec »), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : prompts éditables sans recompiler ; l'administrateur
et l'agent en écrivent, l'agent par `prompts:write`, ses versions marquées et « à valider » jusqu'à l'approbation ;
standards installés d'office qui évoluent tant qu'ils ne sont pas personnalisés ; prompts en base avec leur historique,
dans la sauvegarde ; règles de Guy dans le prompt de développement ; **aucune migration n'écrit de données** ; Kesh pas
en service. **Décisions de l'orchestrateur après la P1 et la P2**, à tenir pour vraies :
découpage i/ii/iii ; amorçage au démarrage, à la fin de la remise à zéro **et** au préréglage de test ; README et
feuille de route du site affectés à la 32-11 ; aucune proposition d'agent sur un nom en `kesh` (hypothèse, soumise à
Guy) ; la proposition et son audit dans **une** transaction ; le service des prompts rend des **types propres aux
services**, jamais `AppError` ; les gardes d'écriture classent sur le **premier mot-clé** (classeur de P7 réemployé) ;
une **poignée qui n'expose que le dépôt des prompts** ; le prédicat fondé sur la **catégorie générale** Unicode
(`unicode-properties`). Ce que la fiche **prescrit** n'existe pas encore ; ce qu'elle cite de la fiche 32-6
est prévu, non codé : juge la cohérence avec cette fiche.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-6` ; n'exécute **ni**
`cargo`, `npm`, `make`, `docker`, `curl`, `gh`, `git commit`, `git push`, `git checkout`, **ni**
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, ni aucun script qui écrit ;
n'ouvre jamais `~/kesh-test/`. Commandes permises : `grep`, `sed -n`, `awk`, `ls`, `wc`, `git log`, `git show`,
`git diff`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

⚠️ **Le motif mesuré de ce dépôt** : la sévérité se déplace vers ce qu'on vient d'écrire. En P2, dix MEDIUM sur treize
naissaient de la remédiation P1. La P2 a été remédiée **par cinq simplifications de conception** (poignées sans pool
visible ; types propres aux services ; proposition et audit dans une transaction ; classeur de P7 réemployé ; catégorie
générale Unicode) et par le retrait de trois gardes que le typage rend inutiles. **Cherche d'abord les défauts nés de la
remédiation P2** — et vérifie que chaque simplification **ferme vraiment** la classe qu'elle prétend fermer (un typage
qui se contourne, un classeur qui rate une forme, une catégorie mal lue). Ne propose pas une garde de plus là où une
conception plus simple fermerait la classe : dis-le.

## Les deux axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque motif écrit dans la fiche, greppé sur le dépôt réel.** Dresse la liste : `grep -nE "grep -|\\\\b|\\\\s|
   REGEXP|Mesure|mesure|témoin" _bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`. Pour chaque motif (faits 4,
   8, 10, 13 ; AC-i11, AC-i14, AC-ii9, AC-ii10, AC-i15), exécute-le **dans le périmètre que la fiche lui donne**, **sur
   l'analogue existant** quand la fiche en nomme un (`mcp_key`, `api_keys`), et **copie la sortie** : une mesure que ton
   grep ne reproduit pas est un finding ; un motif qui attrape une ligne légitime hors liste blanche aussi ; une mesure
   **vide par construction** présentée comme une preuve aussi ; un **témoin vert dit « existant »** absent de la ligne
   citée aussi (ouvre la ligne). Vérifie que chaque garde a propriété, périmètre, formes, un témoin rouge et un témoin
   vert (règle commune) — y compris `amorcage_en_trois_lieux` et sa sémantique d'ancres uniques, les deux gardes du
   classeur (AC-i14) avec leurs témoins **existants** (`companies.rs:358`, `open_items.rs:1401-1403`,
   `test_fixtures.rs:403`, `journal_entries.rs:3956-3958`, migration `:61-63`), `service_des_prompts_sans_poignee_des_livres`
   et `prompts_scelles` amendée.
2. ⛔ **Chaque affirmation sur une dépendance, lue à sa source** (`~/.cargo/registry/src/*/<crate>-<version>/`, version
   tirée de `Cargo.lock`) : `sqlx-mysql 0.8.6` et **`unicode-properties 0.1.4`** (fait 14, AC-ii1 : chaque catégorie
   citée, `tables.rs`, le défaut `Unassigned`, les fonctionnalités, le graphe par `stringprep`), la source de
   `char::is_control` dans la documentation installée, et toute affirmation sur `sqlx`, `axum`, `serde_json` ou MariaDB. Une affirmation qui ne se lit pas à sa source doit être **dite non relue** et confiée à un test
   (`char::is_control`, `REGEXP` et `$`, les catégories des points de code ajoutés au prédicat) — vérifie que la fiche
   le fait.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous
à exercer :

1. **Les faits 1 à 14 et le § *Lignes de code lues et citées*** (dont le bloc « Ajoutées en validations P1 et P2 ») :
   chaque numéro de ligne.
2. **Le classeur réemployé** (AC-i14) : `post_restore.rs:766-896` (`split_statements`, `writes_data`) — peut-il sortir en
   `kesh_db::sql_lexical` **sans changer** le triage P7 (`every_data_backfill_migration_is_triaged`,
   `migrations_contain_no_block_comment`, `registry_sql_has_no_literal_hazard`) ? Ses limites écrites (commentaires de
   bloc, littéraux) valent-elles pour le SQL des `.rs` ? `tables_nommees` et le retrait des alias tiennent-ils les
   témoins cités ?
3. **L'amorçage en trois lieux** (AC-i11, AC-i12, AC-i16) : `routes/onboarding.rs:252-299`, `routes/test_endpoints.rs:
   160-240` (le préréglage, ses erreurs, `users_exist`), `main.rs:3-22`, `:189-198`, `:380-385` ; les tests de la remise à
   zéro et de `test_endpoints_e2e.rs` qui supposent des tables vides ou un compte d'audit (`sole_entry`).
4. **Sauvegarde, P5 à P8, aller-retour** (AC-i12, AC-i13, AC-i15) : la mutation intercalée de l'aller-retour
   (`admin_backup_e2e.rs:465-560`), Mi13 sous ses deux formes contre `restore_body` (`backup.rs:634-720`).
5. **Le dépôt et ses verrous** (AC-i5 à AC-i10) : le verrou de société en tête de **toute** écriture, contre
   `docs/MULTI-TENANT-SCOPING-PATTERNS.md` et `kesh-seed/src/lib.rs:555-610` ; la comparaison en Rust.
6. **Les types propres aux services et l'amendement de la 32-6a** (AC-ii3, AC-ii8, AC-iii1 à AC-iii3) contre la fiche
   32-6 : `ErreurOutil` et ses motifs (AC-a19, AC-a20), `depuis_app_error` (AC-a18), AC-a23, AC-a31 et
   `reponses_par_rendre_seul`, N25, `mcp_sans_pool`, AC-b1 et AC-b3 (journal, fail closed) — l'audit de la proposition
   « est » le journal de l'appel : est-ce compatible avec l'AC-b1 (un seul site, `journaliser_appel`) et l'AC-b6 ?
7. **Le prédicat** (AC-ii1) : chaque point des deux listes et chaque témoin, relus dans `unicode-properties-0.1.4/src/
   tables.rs` ; l'ancienne liste (`bank_accounts.rs:303-322`) entièrement couverte ; les tests de `bank_accounts.rs`.
8. **Le manuel et la documentation** : ce que la 32-10a-i écrit (`admin-manual.tex:1412-1414`, `:2237`, `:2249` ;
   `user-manual.tex:3027-3030` ; `CHANGELOG.md:16-17`) et ce qu'elle laisse aux 32-10b, 32-7, 32-11. Au `.tex` **et au
   PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
   '.{60}(MCP|deux tables|réinitialisation).{80}'`, idem `user-manual.pdf`.

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** neuves de la remédiation P2 : chaque « établi au code » ajouté (le graphe de `unicode-properties`,
   « aucune crate de plus à compiler », la portée des poignées sans pool, l'unicité des ancres, le classeur sans
   changement de P7) tient-il ? Une remédiation repose-t-elle sur un finding P2 sans vérification ?
2. **Découpage** : recompte les modules (i 5, ii 4 avec `lib.rs` et le `Cargo.toml` de `kesh-core`, iii 5 avec
   `outil.rs` ; dix distincts, quatorze en somme) ; chaque fichier du § *Fichiers touchés* dans un module compté de sa
   partie ; `LecturePrompts` naît en ii, `EcriturePrompts` en iii : l'état après ii est-il cohérent ?
3. **Les simplifications ferment-elles leur classe ?** Une poignée sans `pool()` empêche-t-elle vraiment le service
   des prompts de lire les livres (un autre chemin vers un pool dans `services/` ou `crate::` ?) ; les types propres
   empêchent-ils vraiment une `AppError` d'atteindre `src/mcp/` ; une transaction unique tient-elle quand la `Trace` de
   la 32-6b et l'`EcriturePrompts` sont deux poignées ; le classeur rate-t-il une forme (commentaire de bloc dans un
   littéral Rust, `WITH … UPDATE`, `INSERT … SELECT` qui nomme la table au `FROM`) ?
4. **L'inventaire clos** (P1 à P27) : un chemin qui n'y est pas, en particulier ceux qu'ouvrent la remédiation P2
   (préréglage de test, `entree_appel`, `LecturePrompts`). Chaque ligne fermée par ce qu'elle dit ? Les restes écrits de
   P7, P14, P17, P19 sont-ils exacts ?
5. **Sécurité** : la marque ; l'approbation par mégarde ; la portée `prompts:write` ; l'oracle du plafond ; un échec de
   l'amorçage (démarrage, remise à zéro, préréglage de test) ; la société à société ; le verrou de société (un cycle
   restant avec l'onboarding, la réparation, l'import ?).
6. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation Mi1 à Mi21, Mii1 à Mii11, Miii1 à
   Miii12 **réalisable** et tuée par le test **nommé** — en particulier Mi13 (deux formes), Mi21, Mii9, Mii11, Miii12 ;
   `proposition_et_audit_dans_la_meme_transaction` passe-t-il à vide (table renommée par quel pool, rétablie quand) ?
7. **Décomptes** : recompte à la source — 39 AC (16 / 10 / 13), 16 tâches (6 / 5 / 5), 44 mutations (21 / 11 / 12),
   C-32-10a-1 à 32, P1 à P27 — et les chiffres de l'entrée « validation P2 » du Change Log (trend 0/0/13/19, recyclage
   « dix sur treize »).
8. **Le registre** : chaque choix cité existe et dit la même chose ; annotations datées, texte d'origine intact ; un
   point à soumettre à Guy déjà tranché ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p3-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les deux axes
   obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée, ou ligne lue), si le défaut **naît de la
   remédiation P2** (ou P1) ou lui préexistait, et la correction proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
