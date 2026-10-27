# Prompt — validation P1, Story 32-6 (l'endpoint `/mcp` : protocole, garde, lecture seule, filtre, journal)

*Versionné le 2026-10-25. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte, **sans `gh`**.*

Dépôt `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6` (partie de `main` `b723e3d9`, la 32-5a fusionnée ; les
32-5b, 32-5d et 32-5c ne le sont pas encore — la fiche en dépend et le dit). Fiche :
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (deux parties : 32-6a, 32-6b) ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md` (C-32-6-1 à 30). Contexte :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-6* et ses *Reports de la 32-5*, § *Ce que le code dit
déjà*, § *Points d'attention*, § *Réponses de Guy*) ; spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md`
(**en entier** : § 1 révision et transport, § 2.4, § 4 lecture seule, § 5.3 exigences E1 à E6 et points remis à la
32-6, § 6, § 6.2, § 7 et ses **réponses** Q10, Q11, Q12) ; fiche `32-5-cles-et-reglages-mcp.md` (AC-b3, AC-b4,
AC-c4, § *Ce que reçoivent les stories suivantes*, § *Pour la 32-5b*) et son registre `epic-32-choix-autonomes-325.md`
(C-32-5-5, 7 à 11, 14, 19, 37) ; fiches `32-3-services-motif.md` (gardes v-1, v-2, vi ; C-32-3-5, C-32-3-30),
`32-4a-*`, `32-4b-*`. Règles : `CLAUDE.md` (§ *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que
sur une passe sans finding au-dessus de LOW —, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*,
*Pendant une boucle de revue* et son exception `kesh-db`, *Migration breaking policy* P1 à P8, *Le prompt d'une passe
doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Recette visuelle*), `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : **Q11** — lecture seule **applicative** (poignée
`Lecture`) **plus** mode de session `READ ONLY` ; **ni second utilisateur MariaDB, ni variable `.env`** ; les mots
« par construction » ne valent pas pour la lecture seule ; **Q10 (c)** et **Q12** — la poignée et sa table sont la
32-8, la 32-6 applique la politique au répartiteur seulement ; **réseau** — local, filtre de défense en profondeur
désactivable, `X-Forwarded-For` des seuls mandataires déclarés, Traefik devant une partie des conteneurs ; **désactivé
par défaut**, et désactivé il doit être **indiscernable d'une route absente** (la fiche montre qu'une route absente
rend 405 au `POST` : contrôle-le) ; **journal sans arguments ni données** ; au plus cinq modules par partie ; aucune
donnée réelle. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un défaut.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 12** (§ *Ce que le code dit*). En particulier : (a) le comportement d'une route absente —
   `lib.rs:213-214`, `:1225-1227`, et la source de `tower-http` à la version du `Cargo.lock`
   (`~/.cargo/registry/src/*/tower-http-0.6.11/src/services/fs/serve_dir/mod.rs:317-327`, `future.rs:172-190`) : un
   `POST` vers un chemin absent rend-il 405 `Allow: GET,HEAD` à corps vide ? un `GET` la page de l'application ? ;
   (b) `require_auth` (`middleware/auth.rs:83-170`) ; (c) `ConnectInfo` et `MockConnectInfo`
   (`axum-0.8.9/src/extract/connect_info.rs`) ; (d) `Pool::connect_options`, `options`, `before_acquire`,
   `connect_lazy_with` dans `sqlx-core-0.8.6` — signatures réelles, et **quand** `before_acquire` est appelé (connexion
   neuve ou seulement reprise d'une connexion au repos ?) ; (e) les gardes v-1, v-2, vi de `tests/services_motif.rs`
   et ce que le câblage proposé (AC-a25) y change ; (f) l'extracteur d'`audit_route_registry.rs` : la forme
   `post(routes::mcp::recevoir).fallback(routes::mcp::autre_methode)` est-elle lue, et `appels_non_handlers` la
   signale-t-il comme la fiche le dit ? (g) `ActorType` et l'`ENUM` d'`actor_type`.
2. **Le montage et la règle « servi »** (AC-a1 à AC-a5) : la délégation au service statique est-elle réalisable dans un
   gestionnaire Axum 0.8 (un `ServeDir` cloné, appelé par `tower::ServiceExt::oneshot`) ? Rend-elle **exactement** la
   réponse du repli, CSP comprise, pour chaque méthode ? La route `/mcp` avec un `MethodRouter::fallback` : une
   requête `GET` y arrive-t-elle, et `HEAD` ? L'ordre route / repli / couche (`lib.rs:1204-1227`) permet-il la
   position prescrite ? Le test d'égalité peut-il passer **à vide** (deux 404 identiques faute d'`index.html`) — la
   fiche le ferme-t-elle ?
3. **Le réseau** (AC-a6 à AC-a9) : l'algorithme de `X-Forwarded-For` contre le cas (i) et un client qui forge
   plusieurs entrées ; `IpAddr::to_canonical` (version de Rust du dépôt, `rust-toolchain`) ; `ipnet` (dépendance
   directe après la 32-5c seulement : le dire) ; la normalisation d'origine et sa cohérence avec la forme stockée par
   la 32-5c (C-32-5-19) ; `public_base_url` (`config.rs:334`, `:1089-1091`).
4. **La lecture seule** (AC-a24 à AC-a27) : la fabrique proposée compile-t-elle en esprit (types de `before_acquire`,
   `get_acquire_timeout`) ? La séquence B de la spike (§ 4.2) est-elle reproductible par le test (e) — une connexion
   rendue puis reprise passe-t-elle par `before_acquire` ? Les tests de `services_motif.rs` et `services_rollout.rs`
   restent-ils verts si `tests/common/lecture_seule.rs` emploie la fabrique de production (même comportement, un
   `before_acquire` de plus) ? La garde lexicale (AC-a26) : relance `grep -rniE "read\s+write|tx_read_only\s*=|transaction_read_only" crates`
   — vierge ? Le motif capte-t-il une ligne légitime (doc-comment, squash) ?
5. **Le protocole** (AC-a10 à AC-a23) contre la spike § 1.1, § 1.3, § 5.3.1, § 5.3.3 et § 6 (l. 932-944) : chaque
   exigence de la spike destinée à la 32-6 (E1 côté répartiteur, E2, E3, E5, mutations 1, 2, 3 et 5, `readOnlyHint`,
   corps borné, `MCP-Protocol-Version`, `DELETE` 405, `ping`, message de type réponse, les six LOW de § 6.2) a-t-elle
   son AC ? Une décision de la fiche contredit-elle la spike sans le dire ?
6. **Les reports de la 32-5** (fichier d'epic l. 395-403 ; fiche 32-5, § *Ce que reçoivent les stories suivantes*) :
   refus d'un PAT à l'endpoint avec `last_used_at` inchangée, règle du 404 par société dans les deux branches,
   `authentifier` depuis la garde, politique au répartiteur, réglages sur la `Lecture` sans écriture — chacun
   présent, testé ? Le point R5 de la revue P2 de la 32-5a ?
7. **Le manuel et la documentation** : la fiche dit que la 32-6 n'écrit rien au manuel et qu'aucun texte ne devient
   faux. Vérifie-le : `grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md README.md website/*.html` ; et **le PDF
   aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}(MCP|journal d.audit).{80}"`,
   idem `user-manual.pdf`. Un passage dit-il « une lecture n'écrit rien » (#730) d'une manière que l'endpoint rend
   plus fausse ? Le journal d'audit est-il décrit au manuel utilisateur comme une liste close d'actions que
   `mcp.tool_called` contredirait ? La liste de ce qui est laissé à la 32-7 (§ *Le manuel*) est-elle complète au
   regard des AC ?

## Lentille F — faisabilité, découpage, sécurité et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse du kickoff ou de la spike reprise sans
   vérification (« désactivé → 404 », prémisse 7 « le pool doit naître sur le chemin de production », C-32-3-5) ?
2. **Découpage (C-32-6-1)** : recompte les modules avec la convention écrite — six distincts, 4 et 4 ? Les fichiers
   touchés de chaque partie restent-ils dans ses modules ? L'état intermédiaire après la 32-6a ouvre-t-il un accès
   (un outil qui rendrait une donnée des livres sans journal, une clé `required` qui exécuterait) ?
3. **L'inventaire clos des chemins** (§ *L'inventaire clos*, N1 à N23) : cherche un chemin **qui n'y est pas** —
   une route, une tâche, un `nest`, un `merge`, un gestionnaire d'erreur, un `From<AppError>`, une conversion
   automatique d'Axum, `HEAD`, HTTP/2, un chemin percent-encodé, une requête dont le corps arrive avant les en-têtes,
   un appel de service par un autre canal. Chaque ligne est-elle **fermée par ce qu'elle dit** (relis la garde
   proposée : un faux vert est-il possible — commentaire, macro, alias, `use … as`) ?
4. **Sécurité de la garde** : l'ordre des contrôles (Dev Notes) peut-il fuir quelque chose avant l'authentification
   (temps, statut, en-tête, différence entre « réglage absent » et « filtre refusé ») ? Les réponses dites identiques
   (route absente, 401, outil inconnu) le sont-elles **octet pour octet** dans la conception (en-têtes
   `content-length`, `content-type`, CSP) ? Le refus d'un PAT, d'un JWT, d'un cookie ; la politique `required` ;
   `DonneesEnvoyables` à champ privé — la visibilité Rust tient-elle vraiment (un `impl` dans un module enfant, un
   `Default`, un `Clone` d'une instance obtenue ailleurs) ? Le fail closed du journal (AC-b3) est-il éprouvable tel
   qu'écrit (le pool de l'application fermé, la `Lecture` du MCP ouverte : est-ce vrai si la fabrique la dérive de
   `state.pool` par `connect_lazy_with` ?) ?
5. **Tests et mutations** : chaque AC est-il vérifiable tel qu'écrit, avec sa commande ? Chaque mutation Ma1 à Ma30,
   Mb1 à Mb6 est-elle **réalisable** et tuée par le test **nommé** — une passerait-elle à vide (Ma3 si `lectureSeule`
   était lu sur une autre connexion ; Ma4 si `touch_last_used` n'était pas appelé ; Ma20 si le test ne compare pas à un
   vrai `index.html` ; Ma22 et l'astuce de la table renommée sous l'utilisateur `kesh`) ? Une mutation ne compile-t-elle
   pas (alors elle ne prouve rien par un test) ? Manque-t-il une mutation qui retire une garde nommée (refus d'un PAT,
   filtrage de `tools/list`, filtre d'adresse, `Origin`, lecture seule — elles sont exigées) ?
6. **Décomptes** : recompte à la source — 35 AC (30 / 5), 11 tâches (7 / 4), 36 mutations (30 / 6), C-32-6-1 à 30, les
   modules par partie, les numéros de ligne cités au § *Lignes de code lues et citées*, « soixante-dix fichiers » du
   fait 3 (`grep -rln into_make_service_with_connect_info crates/kesh-api/tests | wc -l`).
7. **Ce que la fiche omet** : un report de la 32-5 ou de la spike non repris ; un site qui compte les routes, les
   actions d'audit, les fichiers de `services/` ou les fabriques de poignées et que la fiche ne nomme pas ; les
   « points à soumettre à Guy » qui devraient y être et n'y sont pas (et inversement : un choix pris en autonomie qui
   revient à Guy) ; la frontière avec la 32-8 (tableau) — juste, complète ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**,
un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/326-validate-p1-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/326-validate-p1-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`** et
**`scripts/regen-test-schema.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`,
**`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push`,
et **`gh` sous toutes ses formes** (même en lecture : cette passe s'en passe). `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors
de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et les
filtres en lecture seule `tr`, `sort`, `cut`, `head`, `tail`, `awk`, `wc`, `uniq` vers la sortie standard.
