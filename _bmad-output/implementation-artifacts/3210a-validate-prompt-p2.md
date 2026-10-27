# Prompt — validation P2, Story 32-10a (les prompts MCP en base)

*Versionné le 2026-10-25. Passe **complète**, deuxième sur huit (P1 Sonnet ×2 : R 0/0/4/9, F 0/0/7/9, remédiés). **Deux
lentilles, R et F**, chacune en contexte frais, **Opus** (modèle différent de la P1). Lecture seule stricte, **sans
`gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10a` (partie de `main` `d993106c` : 32-5a et 32-5b
fusionnées ; 32-5c, 32-5d, 32-6a, 32-6b **non fusionnées** — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (trois parties i, ii, iii ; **découpage confirmé** par
l'orchestrateur) ; registre : `epic-32-choix-autonomes-3210a.md` (C-32-10a-1 à 29 ; C-1, 8, 12, 17 annotés). **Rapports
de la P1**, à lire pour savoir ce qui a été remédié — **non pour les reprendre crus** :
`/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p1-{R,F}.md` ; l'entrée « validation P1 » du Change Log de la fiche
dit ce qui a été fait de chacun. Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du
Project Lead*, *Règles de l'agent sur les données réelles*, § *32-10*, *Ce que le code dit déjà* point 9, *Réponses de
Guy*) ; spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 5.3.1 E1 et E2, § 5.3.2, § 5.3.3, § 6 ligne
*32-10a*, § 7 et ses réponses — **la P1 ne l'a pas confrontée** : à faire) ; `epic-32-choix-autonomes.md` (C-32-0-7, 9,
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
en service. **Décisions de l'orchestrateur après la P1**, à tenir pour vraies : découpage i/ii/iii ; amorçage au
démarrage **et** à la fin de la remise à zéro (même fonction) ; prédicat de `kesh-core` complété des points de code de
l'ancien angle mort ; README et feuille de route du site affectés à la 32-11 ; aucune proposition d'agent sur un nom en
`kesh` (hypothèse, soumise à Guy). Ce que la fiche **prescrit** n'existe pas encore ; ce qu'elle cite de la fiche 32-6
est prévu, non codé : juge la cohérence avec cette fiche.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-6` ; n'exécute **ni**
`cargo`, `npm`, `make`, `docker`, `curl`, `gh`, `git commit`, `git push`, `git checkout`, **ni**
`scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/test-fast.sh`, ni aucun script qui écrit ;
n'ouvre jamais `~/kesh-test/`. Commandes permises : `grep`, `sed -n`, `awk`, `ls`, `wc`, `git log`, `git show`,
`git diff`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

⚠️ **Le motif mesuré de ce dépôt** : la sévérité se déplace vers ce qu'on vient d'écrire. En P1, elle était dans des
**gardes mesurées sur des noms inexistants** ; la remédiation a réécrit ces gardes, en a créé de neuves (ordre d'appel,
`accuse_construit_en_un_lieu`) et a changé une conception (l'amorçage à la remise à zéro, l'amendement des types de la
32-6a). **Cherche d'abord les défauts nés de la remédiation P1.**

## Les deux axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque motif écrit dans la fiche, greppé sur le dépôt réel.** Dresse la liste : `grep -nE "grep -|\\\\b|\\\\s|
   REGEXP|Mesure|mesure" _bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`. Pour chaque motif (faits 4,
   8, 10, 13 ; AC-i11, AC-i14, AC-ii9, AC-ii10, AC-i15), exécute-le **dans le périmètre que la fiche lui donne**, **sur
   l'analogue existant** quand la fiche en nomme un (`mcp_key`, `api_keys`), et **copie la sortie** : une mesure que ton
   grep ne reproduit pas est un finding ; un motif qui attrape une ligne légitime hors liste blanche aussi ; une mesure
   **vide par construction** présentée comme une preuve aussi ; un **témoin vert dit « existant »** absent de la ligne
   citée aussi (ouvre la ligne). Vérifie que chaque garde a propriété, périmètre, formes, un témoin rouge et un témoin
   vert (règle commune) — y compris les gardes neuves (`amorcage_par_main_et_remise_a_zero_seulement` et sa fonction
   d'ordre, `accuse_construit_en_un_lieu`).
2. ⛔ **Chaque affirmation sur une dépendance, lue à sa source** (`~/.cargo/registry/src/*/<crate>-<version>/`, version
   tirée de `Cargo.lock`) : `sqlx-mysql 0.8.6` (fait 14), et toute affirmation sur `sqlx`, `axum`, `serde_json`,
   MariaDB ou Unicode. Une affirmation qui ne se lit pas à sa source doit être **dite non relue** et confiée à un test
   (`char::is_control`, `REGEXP` et `$`, les catégories des points de code ajoutés au prédicat) — vérifie que la fiche
   le fait.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous
à exercer :

1. **Les faits 1 à 14 et le § *Lignes de code lues et citées*** : chaque numéro de ligne, les neufs en particulier
   (`routes/onboarding.rs:275-299`, `:282`, `:297` ; `bank_accounts.rs:1462-1530`, `:1466-1513`, `:1475` ;
   `backup.rs:543` ; `companies.rs:305` ; `kesh-seed/src/lib.rs:576` ; `mcp_keys.rs:282-284`, `:336` ;
   `journal_entries.rs:3956-3958` ; `migrations/20261025000001_…sql:61-63` ; `onboarding_audit_e2e.rs:2011`,
   `:2095-2099` ; `test_schema_guard.rs:366` ; `services/cles_mcp.rs:36-41` ; `services/plan_comptable.rs:7` ;
   `fr-CH/messages.ftl:2601` ; `services/acces.rs:29`).
2. **L'amorçage à la remise à zéro** (AC-i11, AC-i12, AC-i16) : la route `reset` (`routes/onboarding.rs`), son
   `get_or_init_state`, la réponse, le détail d'`installation.reset` (`kesh-seed/src/lib.rs`, `tables_cleared`) — dit-il
   encore vrai ? Le test `reset_clears_every_company_table_and_traces_itself` peut-il asserter ce que la fiche dit
   (montage, appel par `reset_with_flag`, boucle `:2095-2099`, `EMPTY_BEFORE_RESET`) ? Les autres tests de la remise à
   zéro (`grep -n 'reset_with_flag\|reset_demo' crates/kesh-api/tests/*.rs`) supposent-ils des tables vides après ? La
   phrase du manuel (`admin-manual.tex:1412-1414`) et le PDF aplati.
3. **Sauvegarde, P5 à P8** (AC-i12, AC-i13, AC-i15) : mêmes axes qu'en P1, contre la fiche remédiée ; les colonnes sans
   `DEFAULT` (`is_standard`, `nature`) et `ColumnConstraint::is_required` (`backup.rs:536-545`) : l'import d'une
   sauvegarde de la même version reste-t-il possible ?
4. **Le dépôt** (AC-i5 à AC-i10) : `synthese_par_societe` (une requête : réalisable ? `standard_differe` exige-t-il les
   textes ?), `VersionEcrite`, l'ordre des verrous « société puis prompt » contre `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
   et les sites qui prennent `companies` ; le refus des noms en `kesh` pour l'agent contre l'AC-i10 (standard retiré du
   code puis remis) et l'AC-iii11.
5. **L'amendement des types de la 32-6a** (AC-iii2) contre la fiche 32-6 : `ReponseMcp` et `rendre` (AC-a31, garde
   `reponses_par_rendre_seul`), `tools_list_filtre` et la pose de `readOnlyHint` (AC-a17), `registre_coherent` (AC-a16),
   `outil_pour` (AC-a21), le journal (AC-b1 : cinq clés exactes, `outil`, `validation`, issues), `mcp_sans_pool` ;
   l'amendement est-il **complet** (aucun test ni garde de la 32-6 qui rougirait sans être nommé) ?
6. **Le prédicat complété** (AC-ii1) : le déménagement du test des plages, la fusion des plages voisines et les voisins
   assertés acceptés ; `controles_cc_refuses` et `lettres_et_espaces_ordinaires_acceptes` ; un autre usage du prédicat ?
7. **Le manuel et la documentation** : ce que la 32-10a-i écrit (`admin-manual.tex:1414`, `:2237`, `:2249` ;
   `user-manual.tex:3027-3030` ; `CHANGELOG.md:16-17`) et ce qu'elle laisse aux 32-10b, 32-7, 32-11 (dont
   `README.md:233`, `website/roadmap.html:375`). Au `.tex` **et au PDF aplati** : `pdftotext docs/manual/fr/admin-manual.pdf
   - | tr '\n' ' ' | tr -s ' ' | grep -oiE '.{60}(MCP|deux tables|réinitialisation).{80}'`, idem `user-manual.pdf` ; `grep
   -rn -i 'prompt\|deux tables' docs/ CHANGELOG.md README.md website/`.

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** neuves de la remédiation : chaque « établi au code » ajouté en P1 (fait 2 complété, fait 10 complété,
   fait 13 et ses analogues, la démonstration « la iii ne touche pas `kesh-db` ») tient-il ? Une remédiation repose-t-elle
   sur un finding P1 sans vérification ?
2. **Découpage** : recompte les modules (i 5 avec `routes/onboarding.rs`, ii 4, iii 5 ; dix distincts, quatorze en
   somme) ; chaque fichier du § *Fichiers touchés* est-il dans un module compté de sa partie ? La table « besoin de la
   iii → fonction de la i » est-elle complète (routes, outil, journal, audit) ? L'état après i, puis après ii, ouvre-t-il
   un accès ou fait-il mentir un texte (l'amorçage de la remise à zéro arrive en i : que dit le manuel entre i et ii) ?
3. **L'inventaire clos** (P1 à P27) : cherche un chemin qui n'y est pas — en particulier ceux qu'ouvrent les nouveautés
   de la P1 : la remise à zéro qui écrit des prompts, `ReponseMcp::Accuse`, `synthese_par_societe` (ce que la route de
   liste rend à l'écran : textes ?), le refus des noms en `kesh`. Chaque ligne est-elle **fermée par ce qu'elle dit** ?
   P17 est-il honnêtement qualifié ?
4. **Sécurité** : la marque ; l'approbation par mégarde ; la portée `prompts:write` ; l'accusé (oracle ?) ; un
   **échec de l'amorçage à la remise à zéro** (que voit l'administrateur ? est-ce une « opération annoncée en succès qui
   a partiellement échoué » ?) ; l'injection d'une société à l'autre ; le verrou de société (interblocage avec
   `reset_demo`, l'onboarding, la réparation ?).
5. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation Mi1 à Mi20, Mii1 à Mii10, Miii1 à Miii11
   **réalisable** et tuée par le test **nommé** — en particulier Mi7 et Mi14 (squash), Mi18 à Mi20, Mii10, Miii11 ; le
   test de concurrence du plafond (deux transactions, attente constatée) est-il écrivable sans flake ?
6. **Décomptes** : recompte à la source — 39 AC (16 / 10 / 13), 16 tâches (6 / 5 / 5), 41 mutations (20 / 10 / 11),
   C-32-10a-1 à 29, P1 à P27 — et les chiffres de l'entrée « validation P1 » du Change Log (trend 0/0/11/18 et ses
   doublons ; cohérence du total avec sa ventilation, `CLAUDE.md` § *Recompter*).
7. **Le registre** : chaque choix cité par la fiche existe-t-il et dit-il la même chose ? Les annotations de C-1, 8, 12,
   17 sont-elles datées, sans réécriture du texte d'origine ? Une décision de Guy ou de l'orchestrateur est-elle
   contredite ? Un point à soumettre à Guy est-il déjà tranché ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210a-validate-p2-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les deux axes
   obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée, ou ligne lue), si le défaut **naît de la
   remédiation P1** ou lui préexistait, et la correction proposée.
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste.
