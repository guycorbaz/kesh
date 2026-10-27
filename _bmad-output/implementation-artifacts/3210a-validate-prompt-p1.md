# Prompt — validation P1, Story 32-10a (les prompts MCP en base)

*Versionné le 2026-10-25. Passe **complète**, première sur huit. **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a` (partie de `main` `d993106c` : 32-5a et 32-5b
fusionnées ; 32-5c, 32-5d, 32-6a, 32-6b **non fusionnées** — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (trois parties : 32-10a-i, ii, iii) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210a.md` (C-32-10a-1 à 26). Contexte :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du Project Lead*, *Règles de l'agent sur les
données réelles*, § *32-10*, *Ce que le code dit déjà* point 9, *Points d'attention*, *Réponses de Guy*) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 5.3.1 E1 et E2, § 5.3.2, § 5.3.3, § 6 ligne *32-10a*, § 7 et
ses réponses) ; `epic-32-choix-autonomes.md` (C-32-0-7, 9, 11, 12) ; fiche `32-5-cles-et-reglages-mcp.md` (parties a et
d, § *Ce que le code dit*) ; **la fiche 32-6, en lecture seule dans un autre worktree** :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` et
`epic-32-choix-autonomes-326.md` (AC-a13, a15, a23, a24, a30, a31, AC-b1 à b4, l'inventaire N1 à N28, la règle commune
des gardes lexicales, C-32-6-23, C-32-6-24). Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée*,
*Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception
`kesh-db`, *Migration breaking policy* P1 à P8, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres
comptes rendus*, *Propagation post-patch*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : prompts éditables sans recompiler ; l'administrateur
**et l'agent** en écrivent, l'agent par `prompts:write`, ses versions marquées « écrit par un agent » et « à valider »
jusqu'à l'approbation de l'administrateur ; des standards installés d'office, servant d'exemples, qui évoluent avec les
versions tant qu'ils ne sont pas personnalisés ; prompts **en base avec leur historique** et **dans la sauvegarde** ; le
prompt de l'agent de développement porte les règles de Guy (jamais de montant, nom, IBAN ni libellé réel hors de Kesh ;
un défaut se décrit par sa structure et se reproduit sur la démonstration ; en cas de doute, il demande) ; **aucune
migration n'écrit de données** (C-32-0-11), les standards sont copiés au démarrage ; Kesh n'est pas en service, aucune
sauvegarde antérieure à importer ; au plus cinq modules par partie ; aucune donnée réelle. Ce que la fiche **prescrit**
n'existe pas encore : son absence n'est pas un défaut. Ce que la fiche cite **de la fiche 32-6** est prévu, non codé :
juge sa cohérence avec cette fiche, non sa présence dans le code.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-6` ; n'exécute **ni**
`cargo`, `npm`, `make`, `docker`, `curl`, `gh`, `git commit`, `git push`, `git checkout`, **ni**
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, ni aucun script qui écrit ;
n'ouvre jamais `~/kesh-test/`. Commandes permises : `grep`, `sed -n`, `awk`, `ls`, `wc`, `git log`, `git show`,
`git diff`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

## Les deux axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque motif écrit dans la fiche, greppé sur le dépôt réel.** Dresse d'abord la liste : `grep -nE "grep -|\\\\b|
   \\\\s|REGEXP|mesure" _bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`. Pour chaque motif (faits 4, 8,
   10, 13 ; AC-i11, AC-i14, AC-ii9, AC-ii10, AC-i15), exécute-le **dans le périmètre que la fiche lui donne** et **copie
   la sortie** : une « mesure » que ton grep ne reproduit pas est un finding ; un motif qui attrape une ligne légitime
   hors liste blanche aussi ; un **témoin vert dit « existant »** qui n'existe pas à la ligne citée aussi (ouvre la
   ligne). Vérifie que chaque garde a sa propriété, son périmètre, ses formes, un témoin rouge et un témoin vert
   (règle commune).
2. ⛔ **Chaque affirmation sur une dépendance, lue à sa source** (`~/.cargo/registry/src/*/<crate>-<version>/`, version
   tirée de `Cargo.lock`) : le décodage `String`/`bool` de `sqlx-mysql 0.8.6` (fait 14), et toute autre affirmation que
   tu trouves sur `sqlx`, `axum`, `serde_json` ou MariaDB. Une affirmation qui ne se lit pas à la source est un finding ;
   une affirmation sur la bibliothèque standard (`char::is_control`), dont la source n'est pas installée, doit être
   **dite non relue** et confiée à un test — vérifie que la fiche le fait.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous
à exercer :

1. **Les faits 1 à 14 et le § *Lignes de code lues et citées*** : chaque numéro de ligne.
2. **La sauvegarde et la remise à zéro** (AC-i12, AC-i13) : l'ordre proposé dans `TABLES_TO_TRUNCATE` (enfants →
   parents ; les versions référencent `mcp_keys`, `users`, `mcp_prompts`) ; la partition de la remise à zéro contre les
   règles 1 à 4 de `reset_partition_is_guarded_by_the_schema` (`backup.rs:897-1037`), **avant et après** la 32-5d
   (fiche 32-5, AC-d1 : `mcp_keys` conservée) — le test reste-t-il vert sans modification ? `EMPTY_BEFORE_RESET` et le
   montage `populate` (`onboarding_audit_e2e.rs`) : la fiche dit « inchangée » — est-ce juste si le montage peuple les
   deux tables ? L'export global : sa garde (`exports/global.rs:652-708`, tests de `TABLES_HORS_EXPORT`) et ses compteurs (`:166-167`) ; le compte de
   fichiers de l'export d'installation et **tout autre** littéral du nombre de tables (grep de `\b41\b` dans `crates/`,
   `docs/`, `CHANGELOG.md`).
3. **P5 à P8** (AC-i1, AC-i15) : le patron de la ligne d'audit, les deux sites du total et la partition ; **P6** —
   `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`, chaque site, et les littéraux de
   `migrations_upgrade_path.rs` (`:44-49`, `:120-126`, `:210`) ; **P7** — le détecteur
   `every_data_backfill_migration_is_triaged`, `registry_entries_are_within_import_window`,
   `exemptions_claiming_out_of_window_really_are_out_of_window` (`post_restore.rs`) : une migration DDL qui crée deux
   tables applicatives les laisse-t-elle verts ? Les justifications « Hors fenêtre depuis 20261025000001 » restent-elles
   vraies ? `scripts/prepare-release.sh` (lis-le, ne le lance pas) en dit-il quelque chose ?
4. **Le dépôt et l'amorçage** (AC-i5 à AC-i11) : les règles (version en vigueur, approbation de la dernière, états
   terminaux, préfixe réservé, plafond, comparaison du contenu) sont-elles cohérentes entre elles et avec les `CHECK` de
   l'AC-i3 ? Un cas que la fiche n'écrit pas (standard retiré du code puis remis ; retrait d'administrateur puis retour
   au standard ; proposition sur un nom de standard non encore amorcé dans la société) ? La place de l'appel dans
   `main.rs` contre l'ordre réel (`:148-198`, `:380-385`) et le précédent `lancement_par_main_seulement`.
5. **Le service, le catalogue et la sortie** (AC-ii3 à AC-ii9) contre la fiche 32-6 : `McpPrompt` scellé (AC-a24), sa
   signature amendée ; `ReponseMcp` et `reponse::rendre` (AC-a31) ; `EtatMcp.garde` ; les gardes
   `prompts_scelles`, `mcp_sans_pool`, `outils_et_politique_en_leur_lieu` — la fiche 32-10a les amende-t-elle sans les
   affaiblir ? La règle « servi » de la 32-6 s'applique-t-elle avant le catalogue ?
6. **Les écritures, le journal et l'audit** (AC-iii1 à AC-iii10) contre la fiche 32-5 (AC-c1, AC-c2, AC-c5) et la fiche
   32-6 (AC-b1 à AC-b4, la règle « `readOnlyHint: true` sur tout outil ») : sites des compteurs
   d'`admin_pat_denied_e2e.rs` et d'`audit_route_registry.rs`, garde `audit_label_registry.rs`, garde G9 de
   `textes_coherents.rs`, `FABRIQUES_V2` de `services_motif.rs`.
7. **Le manuel et la documentation** : la fiche dit ce que la 32-10a-i écrit (`admin-manual.tex:2237`, `:2249`, la
   remise à zéro ; `user-manual.tex:3027-3030` ; `CHANGELOG.md:16-17`) et ce qu'elle laisse aux 32-10b, 32-7 et 32-11.
   Vérifie au `.tex` **et au PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' |
   grep -oiE '.{60}(MCP|deux tables|remise à zéro).{80}'`, idem `user-manual.pdf` ; `grep -rn -i 'prompt\|deux tables'
   docs/ CHANGELOG.md README.md website/`. Une phrase publiée que les tables neuves rendent fausse et que la fiche ne
   nomme pas est un finding.

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** : chaque « établi au code » de la fiche tient-il ? En particulier le fait 3 (sociétés nées hors du
   démarrage) et ses conséquences (C-32-10a-8) ; le fait 10 (le prédicat de `bank_accounts.rs`, ses appelants, ses
   tests) ; le fait 14.
2. **Découpage** : recompte les modules de chaque partie selon la convention écrite (i 5, ii 4, iii 5 ; dix distincts,
   quatorze en somme) ; chaque fichier du § *Fichiers touchés* est-il dans un module compté de sa partie ? La partie iii
   ne touche-t-elle vraiment pas `kesh-db` (le dépôt de la partie i suffit-il à toutes ses routes et à l'outil) ?
   L'état intermédiaire après i, puis après ii, ouvre-t-il un accès ou fait-il mentir un texte ?
3. **L'inventaire clos** (P1 à P27) : cherche un chemin **qui n'y est pas** par lequel le contenu des tables des prompts,
   ou un texte présenté comme un prompt, atteint un agent — client MCP, API par PAT, export, journal d'audit, journal du
   serveur (lu par l'agent de développement), fichiers, sauvegarde automatique, écrans, e-mails. Chaque ligne est-elle
   **fermée par ce qu'elle dit** ?
4. **Sécurité** : la marque (peut-elle être défaite, imitée, omise ?) ; la version approuvée par mégarde (les gestes
   suffisent-ils, la prévention tient-elle ?) ; les caractères refusés (le prédicat est-il appliqué à **tous** les
   chemins d'écriture et au service ?) ; la portée `prompts:write` (permet-elle de lire quoi que ce soit ? d'approuver ?) ;
   un oracle (l'accusé, les réponses de `prompts/get`, le temps) ; l'injection d'une société à l'autre.
5. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation Mi1 à Mi17, Mii1 à Mii9, Miii1 à Miii10
   **réalisable** et tuée par le test **nommé** (une passerait-elle à vide ? une ne compile-t-elle pas ? une se joue-t-elle
   sur une migration appliquée, contre P8 ?).
6. **Décomptes** : recompte à la source — 39 AC (16 / 10 / 13), 16 tâches (6 / 5 / 5), 36 mutations (17 / 9 / 10),
   C-32-10a-1 à 26, P1 à P27 — et la cohérence de chaque total avec sa ventilation (`CLAUDE.md` § *Recompter*).
7. **Le registre** : chaque choix cité par la fiche existe-t-il, et dit-il la même chose ? Une décision de Guy (§
   *Contraintes*) est-elle contredite ? Un « Point à soumettre à Guy » est-il en réalité tranché par une décision déjà
   prise ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p1-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les deux axes
   obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée, ou ligne lue), et la correction
   proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
