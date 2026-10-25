# Prompt — revue de code P1 de la Story 32-6a (l'endpoint `/mcp` et toutes ses gardes), trois lentilles, Sonnet

Tu relis le développement de la **partie a de la Story 32-6** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*l'endpoint `/mcp` du serveur MCP — monté par `route_service` à côté du repli statique ; la règle « servi » (sociétés
activées dont le filtre d'adresse admet le client) ; le filtre d'adresse, les mandataires de confiance et
`X-Forwarded-For` ; l'`Origin` ; l'enveloppe HTTP et JSON-RPC 2.0 ; la clé MCP seule (`Authorization: Bearer`) ; la
politique de validation au répartiteur ; le registre, le trait `McpTool`, l'outil `kesh_version` ; le pool des outils
en lecture seule (`kesh_db::pool::pool_lecture_seule`) ; les gardes lexicales.* C'est **la story de sécurité de
l'epic** : un défaut ici ouvre les livres réels de Guy à un agent ou à une page web.

Worktree `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`, partie de `76869cc9` (`main`, toute la Story 32-5
fusionnée). Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-6 diff 76869cc9...HEAD -- crates`. Langue du rapport : français. Modèle :
**Sonnet**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie a est en revue.** Le journal d'audit de chaque appel d'outil est la **32-6b** (après celle-ci) :
son absence n'est pas un finding, et la route `/mcp` est inscrite `Exempt("issue #723 — … Story 32-6b")` au registre
des routes à dessein. L'écran et la section *Le serveur MCP* du manuel sont la 32-7 ; la validation avant envoi (la
poignée, la table des envois) est la 32-8 ; les outils de lecture des livres sont la 32-9.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` — *Ce que le code dit* (faits 1 à 12), *Partie A*
(AC-a1 à AC-a31), *L'inventaire clos* (N1 à N28), *Ce que les gardes ne voient pas*, les *Dev Notes* (ordre des
contrôles, le canal, le pool), *À recontrôler à l'étape 0* (P7-H1 à P7-H4 compris), et le **Dev Agent Record** de la
partie a (étape 0, mutations, gates, décomptes) — puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md` (C-32-6-2, 3, 6, 9, 31, 33, 35 à 41, et les
choix du développement, **C-32-6-42 à C-32-6-51**). Le Dev Agent Record déclare des gates, des mutations et un E2E :
**ne les prends pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/326a-*`
(lecture seule). Aucun contrôle visible ne change : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Les deux axes obligatoires, pour chaque lentille

1. **Chaque motif greppé sur le dépôt réel.** Les gardes lexicales (`crates/kesh-api/tests/mcp_gardes_lexicales.rs`,
   les volets étendus de `audit_route_registry.rs`, `EXCEPTIONS_V1` et `FABRIQUES_V2` de `services_motif.rs`) portent
   chacune un motif, une liste blanche, un témoin rouge et un témoin vert. Pour **chaque** garde : greppe toi-même son
   motif sur son périmètre (`grep -rnE '<motif>' crates/kesh-api/src`, lignes de commentaire écartées à la main) et
   confronte la sortie à la liste blanche écrite ; cherche une forme **plausible** que le motif ne voit pas (alias,
   coupure de ligne, chemin qualifié, `use … as`, `Self::`, `<T>::`), et dis si elle est écrite comme angle mort. Un
   témoin vert « existant » doit exister au dépôt : greppe-le.
2. **Chaque dépendance lue à la source.** Toute affirmation du code, de ses commentaires ou du Dev Agent Record sur le
   comportement d'Axum 0.8.9, de `tower-http` 0.6.11, de `sqlx-core` 0.8.6, d'hyper 1.10.1, d'`ipnet` 2.12.0 ou d'`url`
   se vérifie dans `~/.cargo/registry/src/*/<crate>-<version>/` (ligne citée) — en particulier : `route_service` et
   `Endpoint::Route` (`routing/path_router.rs`, `routing/route.rs`), l'extracteur `Result<ConnectInfo<_>, _>` et
   `MockConnectInfo` (`extract/connect_info.rs`), `ServeDir` sur une méthode autre que `GET`/`HEAD`
   (`services/fs/serve_dir/`), `axum::body::to_bytes` et `LengthLimitError` (`body/mod.rs`), `after_release` qui rend
   `Ok(false)` (`pool/connection.rs`, `pool/inner.rs`), `connect_lazy_with` et le runtime, `Ipv6Addr::to_ipv4_mapped`
   et `IpAddr::to_canonical` (bibliothèque standard), `IpNet::contains` sur des familles mêlées, `Url::origin()` et le
   port par défaut.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/mcp/transport.rs`** — l'ordre des contrôles. Une requête peut-elle atteindre
   `cles_mcp::authentifier`, lire le corps ou atteindre le répartiteur **avant** la règle « servi », la méthode,
   l'`Origin`, le `Content-Type` ? Un refus peut-il différer selon la validité de la clé **avant** l'étape 8 (oracle de
   validité d'une clé volée) ? Le corps est-il jamais lu avant l'authentification ? Que se passe-t-il pour : un `id`
   flottant, un `id` chaîne vide, un objet JSON sans `jsonrpc`, un `method` vide, `params: null`, un corps UTF-8
   invalide, deux en-têtes `Content-Type`, `Authorization: Bearer  <jeton>` (deux espaces), un jeton entouré de blancs ?
2. **`crates/kesh-api/src/mcp/reseau.rs`** — `client` (droite à gauche, mandataires sautés, « toutes de confiance → la
   plus à gauche »), `decider` (filtre désactivé, entrée invalide de `trusted_proxies` qui écarte la société, entrée
   invalide d'`allowed_networks` ignorée), `reseau_canonique` (réseau IPv6 mappé de préfixe < 96, `::ffff:0:0/96`
   lui-même, `::/0`), `origine_admise` (plusieurs lignes `Origin`, `Origin` non ASCII, `null`, IPv6 entre crochets,
   port explicite égal au défaut), `origines_propres` (`public_base_url` avec chemin, avec port, invalide). Un client
   public peut-il être admis par une combinaison que les tests ne jouent pas ?
3. **`crates/kesh-api/src/mcp/{reponse,repartiteur,politique,registre,outil}.rs`** — une réponse peut-elle sortir
   ailleurs que par `reponse::rendre` ? Une `Donnees` peut-elle être lue ou une `DonneesEnvoyables` construite hors de
   `politique.rs` ? Un outil peut-il être exécuté sans `autoriser`, ou avant ? Le délai coupe-t-il vraiment l'outil
   (le futur est-il abandonné, et que devient une requête SQL en vol) ? `outil_pour` et `descripteurs_pour`
   filtrent-ils de la même façon ? Un `Resultat` statique peut-il porter une donnée lue ?
4. **`crates/kesh-db/src/pool.rs`** et `crates/kesh-api/src/lib.rs` — le pool des outils : est-il **réellement** en
   lecture seule pour chaque connexion (`after_connect`), chaque connexion est-elle fermée à sa restitution, qu'arrive-t-il
   à une connexion **jamais rendue** (abandon du futur au délai) ? Le câblage de `EtatMcp` : la `garde` (pool
   ordinaire) et la `lecture` des outils peuvent-elles être confondues ? `trace` est-elle sur le pool ordinaire ?
5. **Les tests** (`mcp_transport_e2e.rs`, `mcp_reseau_e2e.rs`, `mcp_traduction.rs`, `mcp_gardes_lexicales.rs`,
   `crates/kesh-db/tests/pool_lecture_seule.rs`, les tests unitaires de `reseau.rs`, `tests/common/{mcp,capture_trace}.rs`)
   — un test peut-il passer **à vide** : une égalité « route absente » vraie parce que les deux réponses sont deux 404,
   un 401 obtenu pour une autre raison que celle visée, un compteur jamais incrémenté parce que l'outil n'est pas au
   registre, une capture `tracing` vide, un « refusé » qui serait aussi rendu sans le filtre ? Chaque témoin positif
   existe-t-il ?

## Lentille E — Edge cases

1. **Indiscernabilité (AC-a3, AC-a7, AC-a2)** : pour chaque état non servi (aucune ligne, ligne désactivée, filtre qui
   refuse, `ConnectInfo` absent, erreur de la base, ligne illisible, mandataire invalide), la réponse est-elle **égale**
   à celle d'une route absente, pour toute méthode, avec ou sans clé, avec un corps de 3 Mio ? Les en-têtes `Allow` et
   `content-length` ? `HEAD` ? Un chemin voisin (`/mcp/`, `/MCP`, `/%6Dcp`, `//mcp`, `/mcp?x`) ? Reste-t-il un oracle
   **autre que temporel** (écrit comme angle mort) ?
2. **La clé (AC-a14, AC-a4)** : PAT actif, JWT, cookie portant une clé MCP **valide**, clé révoquée, expirée, créateur
   inactif ou rétrogradé, PAT derrière le préfixe `kesh_mcp_`, clé d'une société qui ne sert pas, clé d'une société
   servie mais écartée pour un mandataire invalide : le **même** 401, octet pour octet ? La `last_used_at` d'un PAT
   reste-t-elle intacte ? Une erreur de la base est-elle un 500 à corps fixe, jamais un 401 ? Le jeton, l'en-tête ou le
   corps paraissent-ils dans un évènement `tracing` (lis chaque `tracing::` de `src/mcp/`, de `services/serveur_mcp.rs`
   et de `services/cles_mcp.rs`) ?
3. **Le réseau (AC-a8, AC-a9)** : cas (a) à (s) de la fiche — chacun joué par un test, et chacun **discriminant** (le
   test rougirait-il sans la règle qu'il vise) ? IPv6 de lien local avec zone, IPv4 mappée dans `allowed_networks` en
   `/128`, `0.0.0.0/0` stocké (#738), mandataire déclaré en réseau (`172.16.0.0/12`) dont le client fait partie.
4. **La politique et les outils (AC-a15 à AC-a22)** : sous `required`, rien ne s'exécute — compteur à 0 — y compris par
   une notification, un lot, un message de type réponse ; sous `exempt`, l'outil s'exécute une fois ; hors portée,
   inconnu et refusé par le service rendent le même corps, `id` mis à part ; une erreur interne ne recopie jamais un
   message (MariaDB, `Validation(msg)`) ; le délai.
5. **Les mutations** (`326a-mut-*.log`) : chacune tuée par une **assertion nommée** et par la raison visée (lis le
   journal, pas seulement le verdict) ? Ma12 n'est tuée que si **les deux** normalisations sont retirées (la variante
   `Ma12-transport-seul` survit, déclarée) : est-ce une défense en profondeur légitime ou un test trop faible ? Une
   mutation plausible **non jouée** survivrait-elle — `Origin` multiple admis, `Authorization` multiple lue, le
   `Content-Type` sans contrôle des paramètres, `outil_pour` comparant le nom sans tenir compte de la casse, la société
   de la clé cherchée dans **tous** les réglages activés au lieu de S, `after_connect` qui échoue en silence ?
   Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-a1 à AC-a31, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-a1, AC-a6** : la ligne `route_service` de `lib.rs` est-elle **avant** le marqueur `// NOTE: les stories futures
  ajouteront leurs routes protégées` ? Le registre des routes la lit-il (`extract_counted`, `appels_non_handlers`,
  `the_detector_reads_route_service`) ? Les compteurs à **chaque** site — doc d'en-tête (129 / 132), `LIB_ROUTES.len()`
  129, `exempt` 5, `traced` 122 inchangé, `no_matter` 2, total 132, `sans_ecriture` 104 et son message « 103 routes de
  lib.rs », `tout.len()` 132. Greper la **valeur** (`git grep -nE '\b(128|131|103|102)\b'
  crates/kesh-api/tests/audit_route_registry.rs`) et trier à la main.
- **AC-a26** : `services_motif.rs` — `EXCEPTIONS_V1` (une entrée), `FABRIQUES_V2` (trois), le compte de `services/`
  (douze) ; les lignes écrites égalent-elles **exactement** les lignes de `lib.rs` (rustfmt compris) ?
- **AC-a25** : `tests/common/lecture_seule.rs` délègue-t-il à la fabrique de production, signature inchangée ; les
  appelants (`services_motif.rs`, `services_rollout.rs`) sont-ils restés verts au gate complet (journal) ?
- **AC-a10** : la normalisation de l'`Origin` est-elle **la même** des deux côtés (`util::origine_canonique`,
  C-32-6-44) ; le changement de `routes::mcp_server_settings::canonical_origin` (32-5c) en préserve-t-il exactement le
  comportement (messages, cas refusés) ?
- **AC-a12, AC-a19, C-32-6-42** : la liste des méthodes du schéma `2025-11-25` et la forme des erreurs (`id`
  facultatif, jamais `null` ; validation d'arguments en `isError: true` ; outil inconnu et demande mal formée en
  `-32602`) sont-elles celles de la révision figée ? (Source : `schema/2025-11-25/schema.ts` et
  `specification/2025-11-25/server/tools` du dépôt `modelcontextprotocol` ; si tu ne peux pas les lire, dis-le et
  juge contre la spike `planning-artifacts/epic-32-spike-mcp.md`.)
- **Les choix C-32-6-42 à C-32-6-51** : chacun est-il justifié, et l'un d'eux contredit-il un AC ou une décision de Guy
  (Q10 à Q12) ?

## Axe du manuel et de `docs/api-external.md`

La 32-6a ne dit rien au manuel (fiche, § *Le manuel*). Vérifie qu'**aucun texte ne devient faux** par elle :
`grep -n "MCP" docs/manual/fr/*.tex docs/api-external.md`, puis les **PDF aplatis**
(`pdftotext docs/manual/fr/<f>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{60}MCP.{80}"`). En particulier
`docs/api-external.md:185` et `:831` (« pas de serveur MCP Kesh-natif en v0.2 »), `admin-manual.tex:2237`, `:2249`,
`:2925`, `user-manual.tex:3028-3030`, `:3204`, `:3214`, la brochure (`marketing-brochure.tex:480`). Un passage qui
décrirait les réglages du serveur MCP (32-5c) dit-il maintenant autre chose que ce que `/mcp` fait (filtre, mandataires,
origines, « désactivé ») ? Relève aussi ce que la 32-7 devra écrire et que la fiche n'a pas listé.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/326a-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/326a-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/326a-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée), la correction proposée, et s'il est **né du développement** de la 32-6a
   ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
