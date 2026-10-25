# Prompt — validation P2, Story 32-6 (l'endpoint `/mcp` : protocole, garde, lecture seule, filtre, journal)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe ; la P1 a rendu 0/0/11/22, remédiés). **Deux lentilles, R
et F**, chacune en contexte frais, **Opus** (modèle différent de la P1, Sonnet). Lecture seule stricte, **sans `gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6` (partie de `main` `b723e3d9`, la 32-5a fusionnée ; les
32-5b, 32-5d et 32-5c ne le sont pas encore — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (deux parties : 32-6a, 32-6b) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md` (C-32-6-1 à 34). **Rapports de la P1**, à lire
pour savoir ce qui a été remédié — **non pour les reprendre crus** : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p1-R.md`
et `…-p1-F.md` ; l'entrée « validation P1 » du Change Log de la fiche dit ce qui a été fait de chacun, et un écart (F7,
colonne de rejeu). Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-6* et ses *Reports de la
32-5*, § *Ce que le code dit déjà*, § *Points d'attention*, § *Réponses de Guy*) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (**en entier**, dont § 5.3 E1 à E6, § 5.3.3, § 6, § 6.2, § 7 et
ses **réponses** Q10, Q11, Q12) ; fiche `32-5-cles-et-reglages-mcp.md` (AC-b3, AC-b4, AC-c4, § *Ce que reçoivent les
stories suivantes*, § *Pour la 32-5b*) et `epic-32-choix-autonomes-325.md` (C-32-5-5, 7 à 11, 14, 19, 37) ; fiche
`32-3-services-motif.md` (gardes v-1, v-2, vi ; C-32-3-5, C-32-3-30). Règles : `CLAUDE.md` (§ *Review Iteration Rule*
et *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW ; une remédiation de
l'orchestrateur ou d'un agent n'a aucun privilège de justesse —, *Inventorier les sites NON RÉSOLUS*, *Règle de
splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`, *Migration breaking policy*, *Le prompt
d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Propagation post-patch*),
`docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : **Q11** — lecture seule **applicative** plus mode de
session `READ ONLY` ; ni second utilisateur MariaDB, ni variable `.env` (un champ d'`AppState` n'est **pas** interdit ;
la fiche a choisi un sous-routeur à état propre, C-32-6-31) ; « par construction » ne vaut pas pour la lecture seule ;
**Q10 (c)** et **Q12** — la poignée et sa table sont la 32-8 ; **réseau** — local, filtre de défense en profondeur
désactivable, `X-Forwarded-For` des seuls mandataires déclarés, Traefik devant une partie des conteneurs ; désactivé
par défaut et alors **indiscernable d'une route absente** ; journal sans arguments ni données ; au plus cinq modules par
partie ; aucune donnée réelle. Le découpage 32-6a / 32-6b est **confirmé** par l'orchestrateur. Ce que la fiche
**prescrit** n'existe pas encore : son absence n'est pas un défaut.

⚠️ **Le motif mesuré de ce dépôt** : la sévérité se déplace vers ce qu'on vient d'écrire. La remédiation P1 a réécrit
la quasi-totalité des AC (numérotation décalée : 37 AC au lieu de 35) et ajouté C-32-6-31 à 34. **Cherche d'abord les
défauts nés de cette remédiation** : renvois `AC-aN` qui pointent le mauvais AC, mutation dont le test nommé ne la voit
pas, garde nouvelle qui ne ferme pas ce qu'elle annonce, contradiction entre l'AC et le registre.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 12 et le § *Lignes de code lues et citées*** : chaque numéro de ligne (les citations de
   `services/acces.rs`, `audit_route_registry.rs`, `errors.rs`, `sqlx-core-0.8.6/src/pool/inner.rs`,
   `axum-0.8.9/src/extract/connect_info.rs` sont neuves).
2. **Le canal (C-32-6-31, AC-a1)** : dans Axum 0.8.9 (`~/.cargo/registry/src/*/axum-0.8.9/src/routing/mod.rs`),
   `Router<S>::with_state::<S2>` rend-il un `Router<S2>` que `Router<AppState>::merge` accepte ? Le sous-routeur fusionné
   **avant** `.fallback_service` est-il sous la couche `csp_html` ? `Result<ConnectInfo<SocketAddr>, ExtensionRejection>`
   est-il un extracteur valide (`axum-core` : `Result<T, T::Rejection>`), et garde-t-il le repli `MockConnectInfo` ?
   `Request` comme dernier extracteur après un extracteur de parties : la requête remise au `ServeDir` est-elle bien
   entière (méthode, URI, en-têtes, extensions, corps non lu) ?
3. **Le registre des routes (AC-a6, C-32-6-32, AC-b6)** : relis `extract_counted`, `appels_non_handlers` et leurs tests ;
   l'extension proposée à `fallback(` est-elle juste (rustfmt qui coupe la ligne, `fallback_service(`, le `ServeDir…
   .fallback(ServeFile…)` du repli, `service_statique`) ? Les compteurs nommés (`:768`, `:770-771`, `:788`, `:789-791`,
   `:793-797`, `:1119-1127`, `:1129`, doc `:27-32`) sont-ils **tous** ceux qui bougent, et les deltas justes en 32-6a
   puis en 32-6b ? `every_exemption_names_the_issue_that_follows_it` (`:801-814`) accepte-t-il le motif proposé ?
   L'écart à F7 (rejeu `SansEcritureAuJournal`) tient-il au code (`:284-300`, `:326`) ?
4. **Le réseau et l'`Origin`** (AC-a7 à AC-a10) : l'algorithme de `X-Forwarded-For` contre (a)–(r) ; la
   normalisation appliquée des deux côtés contre la forme stockée par la 32-5c (C-32-5-19) ; `public_base_url`
   (`config.rs:334`, `:1089-1091`) et le cas invalide ; l'avertissement dédupliqué de l'AC-a8 — réalisable sans état
   partagé neuf, ou l'état est-il nommé ?
5. **La lecture seule et les erreurs** (AC-a18, AC-a20, AC-a25 à AC-a27, AC-a31) : la fabrique (`before_acquire`,
   `connect_lazy_with` et le runtime) ; le maintien de la signature de `tests/common/lecture_seule.rs` ; la garde de
   l'AC-a27 (commentaires écartés en `.rs` **et** `.sql`) ; la traduction exhaustive `depuis_app_error` — **lis
   l'énumération `AppError`** (`crates/kesh-api/src/errors.rs`) : chaque variante est-elle classable comme la fiche le
   dit, un `match` sans `_` est-il réaliste ? La garde `mcp_ne_rend_jamais_une_app_error` voit-elle un `?` qui
   remonterait une `AppError` par une fonction intermédiaire ?
6. **Le protocole et la frontière 32-8** contre la spike § 1.3, § 5.3.1, § 5.3.3 et § 6 : chaque exigence destinée à la
   32-6 a-t-elle encore son AC après la renumérotation ? `Donnees` en `politique.rs` sans accesseur public : la 32-8
   (écran, représentation source) et la 32-9 (outils) peuvent-elles s'en accommoder, et la fiche le dit-elle ?
7. **Le manuel et la documentation** : la fiche dit que la 32-6 n'écrit rien au manuel. Vérifie :
   `grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md README.md website/*.html` ; et **le PDF aplati** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}(MCP|journal d.audit|clé API).{80}"`,
   idem `user-manual.pdf`. La liste laissée à la 32-7 et à la 32-11 (§ *Le manuel*) est-elle complète au regard des
   AC remédiés (401 d'une clé d'une société qui ne sert pas, marque « clé API » absente au journal, `api-external.md:829`) ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** neuves de la remédiation : chaque « établi au code » ajouté en P1 tient-il (Axum, `sqlx-core`,
   `audit_route_registry.rs`) ? Une remédiation repose-t-elle sur un finding P1 sans vérification ?
2. **Découpage** : recompte les modules — 32-6a 4, 32-6b 5 (`routes` par `audit_route_registry.rs`), six distincts,
   neuf en somme ? Les fichiers touchés de chaque partie restent-ils dans ses modules (le `repositories/mcp_keys.rs`
   conditionnel compris) ? L'état intermédiaire après la 32-6a ouvre-t-il un accès ?
3. **L'inventaire clos** (N1 à N26) : cherche un chemin **qui n'y est pas** — en particulier ceux qu'ouvrent les
   nouveautés : `EtatMcp.app` (un `AppState` entier dans l'état du MCP : un outil ou le transport peut-il y lire
   `app.pool` ? la garde `outils_sans_pool` suffit-elle ?), `Registre::avec_delai` et `build_router_avec_registre`
   publics, `Donnees::nouvelles` public, `ContexteMcp::nouveau` en `pub(super)`. Chaque ligne est-elle **fermée par ce
   qu'elle dit** ?
4. **Sécurité de la garde** : l'ordre des contrôles (Dev Notes) — le 401 fixe pour une clé d'une société qui ne sert pas
   (C-32-6-3 révisé) crée-t-il un autre oracle (temps, `last_used_at`) ? Les réponses dites égales (route absente, 401,
   outil inconnu) le sont-elles **octet pour octet** dans la conception ? Le corps de 3 Mio : 401 avant lecture, 413
   après, route absente quand désactivé — la conception le tient-elle ?
5. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit ; chaque mutation Ma1 à Ma44, Mb1 à Mb6
   **réalisable** et tuée par le test **nommé** (une passerait-elle à vide ? une ne compile-t-elle pas ?) ; les
   mutations exigées (PAT accepté, filtrage de `tools/list`, filtre d'adresse, `Origin`, lecture seule) sont-elles
   toujours là ?
6. **Décomptes** : recompte à la source — 37 AC (31 / 6), 11 tâches (7 / 4), 50 mutations (44 / 6), C-32-6-1 à 34, et
   les chiffres de l'entrée « validation P1 » du Change Log (cohérence du total avec sa ventilation, `CLAUDE.md`
   § *Recompter*).
7. **Ce que la fiche omet** : un report de la 32-5 ou de la spike, un compteur, une garde ou un site que la remédiation
   a rendu nécessaire et qu'elle ne nomme pas ; les « points à soumettre à Guy » (le 2 bis est-il posé honnêtement, ses
   conséquences dites ?) ; un choix pris en autonomie qui reviendrait à Guy.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour un finding né de la remédiation P1, dis-le. Un bilan d'en-tête **cohérent
avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte
pas comme passe. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/326-validate-p2-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p2-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
et **`gh` sous toutes ses formes**. `pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de
`~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre. Autorisés : lecture, `grep`,
`sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`,
`head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
