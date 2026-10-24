# Epic 32 — Spike 32-2 : protocole, transport, validation avant envoi, lecture seule, SDK

**Date de l'essai : 2026-10-24.** Spike de décision : aucun code fusionné. Branche `story/32-2` (doc seulement), partie de
`6f0cee00`. Base de l'essai : `kesh_322` (démonstration, seed `scripts/seed-dev-db.sql`, 83 migrations du worktree),
MariaDB **10.11.19** (`10.11.19-MariaDB-ubu2204`), utilisateur `kesh` du fichier versionné
(`scripts/mariadb-init/01-dev-grants.sql` — `ALL` sur `kesh%`.* et `\_sqlx\_test%`.*, **aucun droit global**, aucun
`GRANT` posé). **Aucune donnée réelle**, `~/kesh-test/` jamais ouvert.

Choix au registre `implementation-artifacts/epic-32-choix-autonomes-322.md` (C-32-2-1 à 9). Journaux bruts des essais,
hors dépôt : `/home/gcorbaz/devel/kesh-gate-logs/322-spike/` (`ro-essai.sql`/`.out`, `ro-pool-main.rs`/`.out`,
`proto-main.rs`, `proto-curl.out`) — ils suffisent à rejouer chaque essai.

---

## Les cinq décisions, en une ligne chacune

1. **Révision figée : `2025-11-25`** (revision *Final*), servie **sans session**, réponses JSON — `2025-06-18` acceptée à
   la négociation ; la révision *Current* `2026-07-28` (sans `initialize`) n'est **pas** servie en 0.16.0, parce que
   Hermes ne la parle pas.
2. **Clients** : Claude Code (2.1.310 installé) et Hermes Agent se branchent tous deux, d'après leur documentation, en
   Streamable HTTP avec un en-tête `Authorization` fixe ; **rien n'a été essayé avec un client réel** (consigne) — les
   gestes pour l'éprouver sont au § 2.4.
3. **Validation avant envoi : l'écran de Kesh**, appel `tools/call` en attente **bornée à 240 s** sur une réponse SSE
   entretenue ; l'élicitation est écartée en 0.16.0 (contournable par le client lui-même, formulaires plats, et elle
   impose de garder un flux serveur → client que la révision sans état n'a pas).
4. **Lecture seule par construction : un second pool dont chaque connexion passe `SET SESSION TRANSACTION READ ONLY`
   (`after_connect`)** — essai réel : `INSERT`, `UPDATE`, `DELETE`, `SELECT … FOR UPDATE`, DDL et même table temporaire
   refusés en **1792 (25006)**, en autocommit comme en transaction ; **mais une session peut le défaire** (quatre formes
   relevées) et sqlx **ne remet pas** l'état de session à zéro entre deux emprunts — d'où un `before_acquire` qui
   vérifie `@@tx_read_only` et une garde lexicale.
5. **SDK : écrit à la main, derrière des traits à types propres à Kesh** (décision de Guy maintenue) — `rmcp` 3.7.0
   (2026-10-22) est officiel, *Tier 1*, double ère et intégrable à Axum, mais a connu **trois versions majeures en huit
   mois** ; il redevient candidat dès qu'un client exige `2026-07-28`.

---

## 1. La révision de la spécification à figer

### 1.1 Ce que dit la spécification (consultée le 2026-10-24)

- `https://modelcontextprotocol.io/specification/versioning` : « The **current** protocol version is **2026-07-28** ».
  Les révisions se marquent *Draft*, *Current* (« may continue to receive backwards compatible changes ») ou *Final*
  (« past, complete specifications that will not be changed »). **`2025-11-25` est donc *Final*.**
- `https://modelcontextprotocol.io/specification/2026-07-28/changelog` : la `2026-07-28` **supprime la poignée de main
  `initialize`** (version et capacités dans le `_meta` de chaque requête), **supprime les sessions** et
  `Mcp-Session-Id`, le `GET`, `ping`, la reprise SSE ; impose `server/discover`, les en-têtes `Mcp-Method` et
  `Mcp-Name` (contrôlés contre le corps, `-32020 HeaderMismatch`), `resultType` sur tout résultat, `ttlMs` et
  `cacheScope` sur les listes ; remplace les requêtes serveur → client (dont `elicitation/create`) par le motif
  **MRTR** (`InputRequiredResult`, le client rejoue sa requête avec `inputResponses`).
- `https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning` § *Compatibility Matrix* :
  « Legacy [client] | Modern [server] | **Fails** » ; « Legacy | Dual-era | Works » ; « Dual-era | Legacy | Works. […]
  HTTP: the modern request returns a `4xx` without a recognized modern error body, and the client falls back to
  `initialize` ».
- `https://modelcontextprotocol.io/specification/2025-11-25/basic/transports` (Streamable HTTP, la révision figée) :
  - « Servers **MUST** validate the `Origin` header on all incoming connections to prevent DNS rebinding attacks — If
    the `Origin` header is present and invalid, servers **MUST** respond with HTTP 403 Forbidden » (corps JSON-RPC sans
    `id` permis) ; même texte en `2026-07-28`.
  - « The body of the POST request **MUST** be a single JSON-RPC *request*, *notification*, or *response* » — **pas de
    lot** (supprimé depuis `2025-06-18`).
  - Notification ou réponse acceptée : « **202 Accepted** with no body ». Requête : `application/json` **ou**
    `text/event-stream`, « The client **MUST** support both ».
  - `GET` : « return `Content-Type: text/event-stream` […] **or else return HTTP 405 Method Not Allowed** ».
  - Session : « A server […] **MAY** assign a session ID at initialization time » — elle est **facultative** : un serveur
    qui n'en émet aucun est conforme.
  - `MCP-Protocol-Version` : en-tête envoyé par le client après l'initialisation ; absent, le serveur « **SHOULD**
    assume protocol version `2025-03-26` » ; « invalid or unsupported […] **MUST** respond with `400 Bad Request` ».

### 1.2 L'essai — un `/mcp` écrit à la main, au `curl`

Prototype jetable (≈ 150 lignes, Axum 0.8.9 et sqlx 0.8.6 du `Cargo.lock` du dépôt, `--offline`), hors du dépôt, sur
`127.0.0.1:3322`, contre `kesh_322` par un pool en lecture seule ; source gardée :
`kesh-gate-logs/322-spike/proto-main.rs`. Sortie complète : `proto-curl.out`. Extraits (en-tête `Date` retiré) :

```
### 1 initialize (2025-11-25)
HTTP/1.1 200 OK            content-type: application/json        (aucun Mcp-Session-Id)
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"prompts":{"listChanged":false},"tools":{"listChanged":false}},
 "instructions":"…","protocolVersion":"2025-11-25","serverInfo":{"name":"kesh","version":"0.16.0-spike"}}}
### 2 notifications/initialized          HTTP/1.1 202 Accepted   content-length: 0
### 4 tools/call nombre_comptes (pool lecture seule)
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"5 comptes","type":"text"}],"isError":false}}
### 5 tools/call essai_ecriture
{"id":4,…"text":"refusée : error returned from database: 1792 (25006): Cannot execute statement in a READ ONLY transaction"…,"isError":true}}
### 6 tools/call outil inconnu
{"error":{"code":-32602,"message":"Unknown tool"},"id":5,"jsonrpc":"2.0"}
### 8 Origin étrangère                   HTTP/1.1 403 Forbidden  {"error":{"code":-32600,"message":"Origin refusée"},"jsonrpc":"2.0"}
### 9 sans clé                           HTTP/1.1 401 Unauthorized
### 10 requête moderne 2026-07-28 (sonde double ère)
HTTP/1.1 400 Bad Request   content-type: text/plain   unsupported MCP-Protocol-Version
### 11 lot JSON-RPC                      HTTP/1.1 400 Bad Request {"error":{"code":-32600,…},"id":null,…}
### 12 GET                               HTTP/1.1 405 Method Not Allowed   allow: POST
### 13 attendre_validation 3 s, SSE + progress
HTTP/1.1 200 OK            content-type: text/event-stream
event: message  data: {"jsonrpc":"2.0","method":"notifications/progress","params":{…"progress":1,"progressToken":"p1","total":3}}
… (2, 3) …
event: message  data: {"id":11,"jsonrpc":"2.0","result":{"content":[{"text":"validé (simulation)","type":"text"}],"isError":false}}
```

L'essai établit que la révision `2025-11-25` **sans session** se sert en quelques fonctions, que le refus d'une requête
moderne par un `400` à corps non « moderne » est exactement ce que la matrice attend pour qu'un client double ère se
replie, et qu'une réponse SSE à progression se produit avec `axum::response::sse` sans dépendance de plus
(`tokio-stream`, déjà au `Cargo.lock`).

### 1.3 Décision (C-32-2-1, C-32-2-2, C-32-2-3)

- **Révision de référence : `2025-11-25`**, texte cité ci-dessus, liens versionnés par la date de révision dans l'URL. À
  `initialize`, Kesh répond la version demandée si c'est `2025-11-25` ou `2025-06-18` (mêmes méthodes pour ce que Kesh
  sert), `2025-11-25` sinon.
- **Pourquoi pas `2026-07-28`** : Hermes Agent s'appuie sur le SDK Python `mcp` 1.x (« mcp Python SDK ≥ 1.11.0 »,
  § 2.2), c'est-à-dire sur la poignée de main ; un serveur moderne seul est « Legacy | Modern | **Fails** ». Un serveur
  double ère est permis mais double le travail de protocole écrit à la main. Claude Code, lui, se replie sur l'ancienne
  poignée de main (§ 2.1). **`2025-11-25` est la seule révision que les deux clients parlent.**
- **Sans état** : aucun `Mcp-Session-Id` émis, ni lu ; `GET` et `DELETE` → **405** ; `notifications/*` → **202** ;
  `ping` → `{}` ; lot → **400** ; corps borné (1 Mio proposé).
- **`MCP-Protocol-Version`** : présent et non servi (dont `2026-07-28`) → **400, corps texte**, *jamais* un code
  `-32022` (sinon un client double ère croirait parler à un serveur moderne et ne se replierait pas) ; absent → admis
  (le seul effet de « assume `2025-03-26` » serait ici le même jeu de méthodes), journalisé.
- **Réponses** : `application/json` partout, **sauf** l'appel en attente de validation (32-8) : `text/event-stream`, un
  premier évènement immédiat, `X-Accel-Buffering: no`, une progression périodique.
- **`Origin`** : **absent → admis** (un client en ligne de commande n'en envoie pas — à éprouver, § 2.4) ; **présent et
  différent des origines de Kesh lui-même** (l'URL publique configurée, et `http://127.0.0.1:<port>` /
  `http://localhost:<port>`) → **403**, corps JSON-RPC sans `id`. La clé `Authorization` reste la vraie barrière : une
  page piégée par *DNS rebinding* ne la connaît pas.
- **Outil inconnu ou hors portée** : erreur JSON-RPC **`-32602`**, même message — conforme à l'exemple « Unknown tool »
  de la spécification et à l'AC « aucun oracle » de la 32-6. Une erreur **d'exécution** d'un outil reste un résultat
  `isError: true`.
- **Tous les outils annotés `readOnlyHint: true`** (Hermes s'en sert pour dispenser d'approbation, § 2.2).

---

## 2. Les clients réels

⛔ **Aucun client n'a été lancé ni configuré** (consigne de la spike) : tout ce qui suit vient de leur documentation
officielle, consultée le 2026-10-24, et est **non éprouvé** tant que Guy ou la 32-11 n'a pas rejoué le § 2.4.

### 2.1 Claude Code — `https://code.claude.com/docs/en/mcp` (version locale : `claude --version` → `2.1.310`)

- **Streamable HTTP et en-tête** : « `claude mcp add --transport http secure-api https://api.example.com/mcp --header
  "Authorization: Bearer your-token"` » ; portées `local` (défaut, `~/.claude.json`), `project` (`.mcp.json` à la
  racine du projet, **versionné**), `user`.
- **Révision** : deux runtimes. « The v2 runtime is the same code on MCP TypeScript SDK 2.0, which adds MCP protocol
  revision 2026-07-28 » ; il « Asks HTTP, stdio, and claude.ai connector servers whether they support the newer revision
  and uses it with those that do. […] Claude Code connects to every other server as v1 does. » Réglages :
  `MCP_SDK_GENERATION=v1|v2`, `MCP_PROTOCOL_NEGOTIATION=auto|legacy`. → un serveur `2025-11-25` est servi « as v1
  does » ; la version exacte que le runtime v1 demande à `initialize` n'est pas écrite dans la page (observation (h)
  du § 2.4).
- **Élicitation** : supportée, formulaire et URL (« Claude Code displays an interactive dialog ») ; mais : « To
  auto-respond to elicitation requests without showing a dialog, use the `Elicitation` hook », et, pour les tâches,
  « In a non-interactive session, Claude Code asks the server to cancel a task that requests input ».
- **Délais d'un `tools/call`** : « The per-server `timeout` is a hard wall-clock limit per tool call, and progress
  notifications from the server don't extend it » (défaut de `MCP_TOOL_TIMEOUT` : « about 28 hours ») ; « a second,
  per-request timer that covers each request through to the server's first response byte » — **au moins 60 s** ; « A
  tool call to an MCP server that sends no response and no progress notification for the idle window aborts » — fenêtre
  **5 min** en HTTP (`CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`). Un appel de plus de deux minutes passe en tâche de fond.
- **Taille** : avertissement au-delà de 10 000 jetons, **limite 25 000** (`MAX_MCP_OUTPUT_TOKENS`).
- **Prompts** : « Claude Code lists each MCP prompt as `/servername:promptname (MCP)` ».

### 2.2 Hermes — `https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp` et `…/docs/reference/mcp-config-reference`

⚠️ **« Hermes » est ambigu** : j'ai retenu **Hermes Agent de Nous Research** (seul client MCP de ce nom trouvé), **à
confirmer par Guy** (C-32-2-9). Rien n'est installé sur la station (`which hermes` vide, pas de `~/.hermes`).

- Configuration `~/.hermes/config.yaml`, clé `mcp_servers.<nom>` : `url`, `headers` (`Authorization: "Bearer ***"`),
  `transport: sse` seulement pour l'ancien transport ; Streamable HTTP par défaut.
- **Délais** : « `timeout` — Tool call timeout in seconds (default: `300`) », `connect_timeout` 60 s (« also bounds the
  MCP initialize handshake » — la poignée de main existe : client *legacy*).
- **SDK** : « mcp Python SDK ≥ 1.11.0 » (pour l'élicitation) ; aucune révision nommée.
- **Élicitation** : formulaire, « routed through Hermes' approval surface », `elicitation.timeout` 300 s ; « URL-mode
  elicitations […] are declined as unsupported ».
- **Approbation** : `trust: full` par défaut ; en `untrusted`, « any tool without a `readOnlyHint: true` annotation
  requires user approval » ; YOLO (`/yolo`, `--yolo`) saute l'approbation.
- **Prompts et ressources** : exposés par des outils d'emballage (`list_prompts`, `get_prompt`, `list_resources`,
  `read_resource`).

### 2.3 Ce que ces deux documentations tranchent déjà

- Les deux parlent `2025-11-25` (Hermes : seulement l'ère *legacy* ; Claude Code : par repli) → C-32-2-1.
- **Le délai d'attente le plus court est celui de Hermes : 300 s, dur.** Claude Code exige un premier octet sous 60 s et
  une activité (réponse ou `notifications/progress`) toutes les 5 min. D'où **240 s** d'attente au plus, en SSE, avec
  un premier évènement immédiat et une progression toutes les 15 à 30 s (C-32-2-4).
- La clé en clair dans `~/.claude.json` (portée `local`) ou, pire, dans un `.mcp.json` **versionné** (portée `project`)
  : le manuel (32-7, 32-11) doit dire **portée `local` ou `user`**, jamais `project` sauf par expansion de variable
  (`${KESH_MCP_KEY}`).

### 2.4 Non éprouvé — les gestes exacts pour l'éprouver (Guy, ou la 32-11)

Contre une démonstration (jamais les livres réels), Kesh 0.16.0 de développement ou le prototype de la spike reconstruit
depuis `kesh-gate-logs/322-spike/proto-main.rs` sur `127.0.0.1:3322` :

```sh
# Claude Code, sans rien écrire dans la configuration (fichier jetable) :
cat > /tmp/kesh-mcp-essai.json <<'EOF'
{"mcpServers":{"kesh-demo":{"type":"http","url":"http://127.0.0.1:3322/mcp",
  "headers":{"Authorization":"Bearer kesh_mcp_spike_factice"}}}}
EOF
claude --strict-mcp-config --mcp-config /tmp/kesh-mcp-essai.json        # puis /mcp dans la session
MCP_PROTOCOL_NEGOTIATION=auto claude --strict-mcp-config --mcp-config /tmp/kesh-mcp-essai.json   # repli v2 → legacy
# ou, de façon persistante (portée local, ~/.claude.json) :
claude mcp add --transport http --scope local kesh-demo http://127.0.0.1:3322/mcp \
  --header "Authorization: Bearer kesh_mcp_spike_factice"
claude mcp get kesh-demo ; claude mcp remove kesh-demo --scope local
```

```yaml
# Hermes Agent — ~/.hermes/config.yaml
mcp_servers:
  kesh-demo:
    url: "http://127.0.0.1:3322/mcp"
    headers:
      Authorization: "Bearer kesh_mcp_spike_factice"
    timeout: 300
```

À observer, et à consigner avec la version du client : (a) `initialize` puis `tools/list` sans session ; (b) le
client envoie-t-il un `Origin` ? (c) un `progressToken` dans le `_meta` de `tools/call` ? (d) `attendre_validation`
à **90 s** (au-delà du minuteur de premier octet de Claude Code) puis à **280 s** (sous les 300 s de Hermes) passe-t-il ?
(e) Claude Code v2 se replie-t-il après le `400` texte de la sonde moderne (journal `claude --debug`) ? (f) un `401`
déclenche-t-il une tentative OAuth (découverte `/.well-known/oauth-protected-resource`) plutôt qu'un message clair ?
(g) le prompt apparaît-il comme `/kesh-demo:bilan_commente (MCP)` ? (h) quelle `protocolVersion` chaque client
demande-t-il à `initialize` (si ce n'est ni `2025-11-25` ni `2025-06-18`, Kesh répond `2025-11-25` et le client peut
refuser) ?

---

## 3. La validation avant envoi

### 3.1 Les deux formes, confrontées aux faits

| Critère | Élicitation MCP (forme) | Écran de Kesh, appel en attente |
|---|---|---|
| Qui garantit que l'utilisateur a vu | **le client**, c'est-à-dire la partie qui reçoit les données ; Claude Code l'auto-répond par un *hook*, Hermes par YOLO | **Kesh** : rien ne part sans un clic dans Kesh |
| Modifier avant envoi | champs plats primitifs (« flat objects with primitive properties only ») : un extrait de grand livre n'y tient pas, sauf en un champ texte | éditeur propre à Kesh, aussi riche qu'il le faut |
| Révision `2025-11-25` sans session | requête `elicitation/create` sur le flux SSE du `tools/call`, réponse du client en `POST` à corréler — non essayé | réponse SSE ordinaire (essayée, § 1.2) |
| Révision `2026-07-28` | autre mécanisme (MRTR) : à réécrire au passage | inchangé |
| Hors session interactive | refusée ou annulée (Claude Code `-p`) | l'appel attend, puis expire |
| Délais | 300 s chez Hermes | 240 s choisis sous les 300 s de Hermes et la fenêtre de 5 min de Claude Code |
| Les deux clients l'affichent | oui en formulaire (URL : non chez Hermes) | aucun affichage requis du client |

### 3.2 Décision (C-32-2-4)

**L'écran de Kesh**, conformément à l'AC de la 32-8 (« par construction ») : le répartiteur de `tools/call` reçoit de
l'outil une **proposition d'envoi**, la garde en mémoire sous un identifiant aléatoire lié à la clé et à la société,
**rend la connexion de lecture avant d'attendre**, ouvre une réponse SSE (premier évènement immédiat, progression toutes
les 15 s si le client a fourni un `progressToken`, sinon commentaire SSE `: attente`), et rend : le contenu **approuvé**
(tel quel ou modifié), ou un résultat `isError: true` « refusé » / « non validé dans les 240 s, rien n'a été envoyé ».
Un redémarrage perd l'attente : l'appel échoue, rien ne part. L'élicitation est écartée **en 0.16.0** ; elle pourra
compléter l'écran plus tard (une élicitation URL vers la page d'approbation de Kesh, chez Claude Code seulement).

**Reste à éprouver** (§ 2.4 d) : que les deux clients tiennent 240 s sur une réponse SSE entretenue — le minuteur de
premier octet de Claude Code est satisfait par les en-têtes ou par le premier évènement, la doc ne le dit pas ; et que
Claude Code accepte les commentaires SSE comme activité (la doc ne nomme que la réponse et `notifications/progress`).
Si l'essai échoue, le repli est une **poignée explicite** (l'outil rend tout de suite « en attente, demande `<id>` »,
l'agent rappelle `kesh_recuperer_envoi(<id>)`), motif même de la `2026-07-28` (« explicit, server-minted handles passed
as ordinary tool arguments »).

---

## 4. La lecture seule par construction — essai réel

### 4.1 Au client `mariadb`, utilisateur `kesh`, base `kesh_322` (`ro-essai.sql` → `ro-essai.out`)

Après `SET SESSION TRANSACTION READ ONLY` (`@@session.tx_read_only = 1`, `@@autocommit = 1`) :

| Instruction | Résultat |
|---|---|
| `INSERT INTO audit_log …` (autocommit) | `ERROR 1792 (25006) … Cannot execute statement in a READ ONLY transaction` |
| `UPDATE companies …` (autocommit) | `ERROR 1792 (25006)` |
| `DELETE FROM audit_log …` (autocommit) | `ERROR 1792 (25006)` |
| `START TRANSACTION; INSERT; UPDATE; DELETE; ROLLBACK` | trois fois `ERROR 1792 (25006)` |
| `SELECT … FOR UPDATE` | `ERROR 1792 (25006)` |
| `CREATE TABLE spike_ddl` | `ERROR 1792 (25006)` |
| `CREATE TEMPORARY TABLE` puis `INSERT` | `ERROR 1792 (25006)` deux fois — **MariaDB refuse aussi la table temporaire** (MySQL l'admet) |
| `SET GLOBAL read_only=1` | `ERROR 1227 (42000) … READ_ONLY ADMIN privilege` (l'utilisateur `kesh` n'a aucun droit global) |

**Ce qui la défait** — quatre formes, toutes admises sans privilège :

| Instruction | Effet constaté |
|---|---|
| `START TRANSACTION READ WRITE; UPDATE …` | **accepté** (la transaction explicite l'emporte) |
| `SET TRANSACTION READ WRITE` puis `UPDATE` | **accepté** pour la transaction suivante ; `@@tx_read_only` relu à `1` ensuite, l'`UPDATE` d'après refusé |
| `SET SESSION tx_read_only=0` | `@@tx_read_only = 0`, `UPDATE` **accepté** (`transaction_read_only`, nom MySQL : `ERROR 1193 Unknown system variable` sur MariaDB 10.11 — essai ajouté en fin de `ro-essai.out`) |
| `SET SESSION TRANSACTION READ WRITE` | `@@tx_read_only = 0`, `UPDATE` **accepté** |

Contrôle après l'essai : `SELECT COUNT(*) FROM audit_log WHERE action='spike'` → `0` ; aucune table `spike%`.

### 4.2 Par un pool sqlx 0.8.6 (`after_connect`) — `ro-pool-main.rs` → `ro-pool.out`

```
== A. pool lecture seule, 3 connexions physiques simultanées
conn 0 id=313122 tx_read_only=1
  autocommit INSERT : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  autocommit UPDATE : ERREUR Some(1792) …     autocommit DELETE : ERREUR Some(1792) …
  tx INSERT : ERREUR Some(1792) …   tx UPDATE : ERREUR Some(1792) …   tx DELETE : ERREUR Some(1792) …
  SELECT : OK (1 société)
conn 1 id=313123 tx_read_only=1   (idem)      conn 2 id=313124 tx_read_only=1   (idem)
  pool.begin() + UPDATE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
== B. une connexion rendue au pool garde son état de session (max 1)
  SET SESSION TRANSACTION READ WRITE : OK (0 lignes)
  après réacquisition, tx UPDATE : OK (1 lignes)
== C. idem avec before_acquire qui vérifie @@tx_read_only (max 1)
  conn avant=313126 après=313127 ; tx UPDATE : ERREUR Some(1792) …
== D. connexion perdue (KILL) : la neuve repasse after_connect
  KILL 313128 : OK (0 lignes)
  nouvelle conn id=313130 tx_read_only=1
== E. pool ordinaire (sans after_connect) : écrit
  tx UPDATE : OK (1 lignes)
== lignes 'spike' au journal d'audit : 0
```

Lecture : `after_connect` s'applique à **chaque connexion physique**, y compris celle qui remplace une connexion perdue
(D) ; `BEGIN` de sqlx hérite du mode de session (A) ; **sqlx ne remet pas l'état de session à zéro** quand une
connexion revient au pool (B) — une seule instruction fautive sur le pool de lecture le rendrait inscriptible pour la
suite de sa vie ; un `before_acquire` qui relit `@@session.tx_read_only` et **jette** la connexion qui n'est plus à `1`
ferme ce trou (C), au prix d'un aller-retour par emprunt.

### 4.3 Décision (C-32-2-5)

- **Un second pool**, construit **dans `main.rs` sur le chemin de production**, par **une seule fonction** de
  `kesh-db` qui fabrique les options des deux pools (réglages de session communs, plus `READ ONLY` pour le second) —
  ce qui règle au passage [#669] (la production n'applique ni `sql_mode` ni `time_zone`), ou s'y adosse si la 32-6 la
  laisse à part.
- **Trois verrous** : `after_connect` (`SET SESSION TRANSACTION READ ONLY`) ; `before_acquire` (relit
  `@@session.tx_read_only`, jette la connexion sinon) ; **garde lexicale** — aucune occurrence de `READ WRITE`,
  `tx_read_only` ni `transaction_read_only` (inconnue de MariaDB 10.11, mais d'une version future peut-être) dans
  `crates/` hors du test qui la tient (l'état au kickoff est vierge :
  `grep -rniE "read write|tx_read_only|transaction_read" crates` ne rend que des `API_KEY_READ_ONLY`).
- **Test sur le chemin de production** (AC de la 32-6) : le pool construit par la fonction de production, utilisateur
  `kesh` du fichier versionné, `INSERT`/`UPDATE`/`DELETE` en autocommit et en transaction → **1792** ; une mutation qui
  retire l'`after_connect` doit le faire rougir ; une qui retire le `before_acquire`, un second test (séquence B).
- **Écarté** : un second utilisateur MariaDB `SELECT` seul — plus fort (il résiste aux quatre formes du § 4.1), mais une
  variable `.env` et un script d'initialisation de plus sur le NAS ; il reste le repli si un des verrous se révèle
  contournable autrement.

### 4.4 Les lectures qui écrivent — la séparation à poser en 32-3

| Site | Ce qu'il écrit | Sur la connexion de lecture |
|---|---|---|
| `emit_report_audit` (`routes/reports.rs:1407`, appelé l. 233, 266, 401, 437, 1123) et sa variante d'export | `pool.begin()` + `audit_log::insert_in_tx` | échouerait en 1792 — **et l'échec est avalé** par `tracing::warn!` : la trace disparaîtrait **en silence** |
| `api_keys::touch_last_used` (`auth/api_key.rs:185`) | `last_used_at` | idem, *best-effort*, avalé |
| `company_invoice_settings::get_or_create_default` (`routes/company_invoice_settings.rs:263, 298`, `exports/global.rs:304`) ; `company_dunning_settings::get_or_create_default` (`exports/global.rs:313`) | `INSERT` paresseux d'une ligne de réglages | échouerait **bruyamment** (erreur 500) |

**Proposition** : le service reçoit **deux poignées distinctes et typées** — `Lecture` (enveloppe du pool en lecture
seule, seule passée aux dépôts de lecture) et `Trace` (journal d'audit, dernière utilisation, sur le pool ordinaire) —
de sorte qu'un service ne puisse pas écrire la trace sur la lecture par erreur. Les `get_or_create_default` ne sont
appelés par aucun outil (l'outil lit, et dit « réglages absents » si la ligne manque). La fiche de la 32-3 **inventorie
l'ensemble clos** des écritures sur les chemins de lecture visés (ce tableau est un relevé, non un inventaire prouvé) et
fait de « un rapport n'écrit plus sa ligne d'audit » une mutation à tuer — c'est le défaut muet de la colonne 3.

---

## 5. Le SDK

### 5.1 État de `rmcp` (consulté le 2026-10-24)

- crates.io (`https://crates.io/api/v1/crates/rmcp`) : **3.7.0 du 2026-10-22**, `rust_version` **1.88** (toolchain de la
  station : 1.97.1), licence Apache-2.0, **70 versions** ; majeures **1.0.0 (2026-03-03), 2.0.0 (2026-06-29),
  3.0.0 (2026-07-28)** — le jour même de la révision `2026-07-28`.
- Dépôt `https://github.com/modelcontextprotocol/rust-sdk` : « The official Rust SDK for the Model Context Protocol » ;
  « implements the stable MCP **`2026-07-28`** specification while remaining fully compatible with the **`2025-11-25`**
  release and earlier versions » ; `https://modelcontextprotocol.io/docs/sdk` le classe **Tier 1**.
- Serveur HTTP (`https://docs.rs/rmcp/3.7.0/…/StreamableHttpServerConfig.html`), feature
  `transport-streamable-http-server` : `StreamableHttpService` est un service Tower (`Router::new().nest_service("/mcp",
  service)`) ; configuration `#[non_exhaustive]` : `legacy_session_mode` (sans session même en *legacy* si `false`),
  `json_response`, `allowed_hosts` (boucle locale par défaut), `allowed_origins` (« Defaults to an empty list, which
  disables Origin validation »), `max_request_body_bytes`, `sse_keep_alive`, `stateless_protocol_metadata_required`.
- Dépendances normales non optionnelles : `chrono`, `futures`, `indexmap`, `libc`, `pin-project-lite`, `serde`,
  `serde_json`, `thiserror`, `tokio`, `tokio-util`, `tracing` ; le serveur HTTP ajoute notamment `hyper`, `http-body`,
  `sse-stream`, `uuid`, `rand 0.10`, `tower-service`. **Non mesuré** : ce qu'elles ajoutent réellement à l'arbre de
  Kesh (aucune compilation de `rmcp` faite).

### 5.2 Décision (C-32-2-6)

**Écrit à la main en 0.16.0** — la décision de Guy tient : le besoin est minuscule (sept méthodes, sans état, JSON ;
le prototype tient en ≈ 150 lignes), et une API qui change de majeure tous les deux à trois mois imposerait des
migrations sans rapport avec Kesh. **Mais le critère « quand il sera mûr » est en partie rempli** (officiel, Tier 1,
double ère, garde `Origin`/`Host` intégrée) : je propose de le réexaminer **dès qu'un des deux clients exige
`2026-07-28`**, ou si la validation doit un jour passer par l'élicitation (MRTR) — question pour Guy (Q8 ci-dessous).

### 5.3 Les traits — indicatifs, pour que le passage au SDK ne change que le transport

Règle : **aucun type de `rmcp` ni de JSON-RPC ne traverse les traits** ; le registre parle des types de Kesh, et
l'adaptateur (le nôtre aujourd'hui, un `impl rmcp::ServerHandler` demain : `list_tools`, `call_tool`, `list_prompts`,
`get_prompt`) traduit.

```rust
/// Portée d'une clé MCP (C-32-0-7). Aucune portée d'administration ni d'écriture des livres.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Portee { BooksRead, InvoicesRead, ContactsRead, DocumentsRead, PromptsWrite }

/// Ce que reçoit un outil : l'accès résolu, la lecture, la trace — jamais le pool ordinaire.
pub struct ContexteMcp<'a> {
    pub acces: &'a ContexteAcces,   // société, clé, portées, canal = Mcp (32-3)
    pub lecture: &'a Lecture,       // pool en lecture seule (§ 4.3)
    pub trace: &'a Trace,           // journal d'audit, sur le pool ordinaire
}

/// Ce qu'un outil rend : une PROPOSITION d'envoi, jamais une réponse (32-8, « par construction »).
pub struct PropositionEnvoi {
    pub resume: String,                       // ce que l'écran de validation montre en tête
    pub contenu: Vec<BlocContenu>,            // texte, champs libres enveloppés « donnée non fiable » (32-9)
    pub structure: Option<serde_json::Value>, // structuredContent, si l'outil en déclare un
}

pub enum ErreurOutil { ArgumentsInvalides(String), Introuvable, Interne }

pub trait McpTool: Send + Sync + 'static {
    fn nom(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema_entree(&self) -> serde_json::Value; // JSON Schema strict, additionalProperties: false
    /// SANS défaut : un outil sans portée ne compile pas (AC de la 32-6).
    fn portee(&self) -> Portee;
    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        arguments: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<PropositionEnvoi, ErreurOutil>> + Send + 'a>>;
}

pub struct ArgumentPrompt { pub nom: &'static str, pub description: &'static str, pub requis: bool }
pub struct MessagePrompt { pub role: Role, pub texte: String }

pub trait McpPrompt: Send + Sync {
    fn nom(&self) -> &str;              // les prompts viennent de la base (32-10a) : pas de &'static
    fn description(&self) -> &str;
    fn arguments(&self) -> &[ArgumentPrompt];
    fn portee(&self) -> Portee;         // sans défaut, comme pour les outils
    fn rendre<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        arguments: &'a BTreeMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<MessagePrompt>, ErreurOutil>> + Send + 'a>>;
}

/// Le registre est le seul point d'entrée du transport : filtrage par portée, refus indiscernable d'un inconnu.
pub struct Registre { outils: Vec<Box<dyn McpTool>> /* ordre stable */ }
impl Registre {
    pub fn lister(&self, portees: &HashSet<Portee>) -> Vec<DescripteurOutil> { /* … */ }
    pub fn trouver(&self, nom: &str, portees: &HashSet<Portee>) -> Option<&dyn McpTool> { /* None = inconnu OU hors portée */ }
}
```

Le futur boxé (plutôt que `async fn` dans le trait) rend le trait **compatible `dyn`**, ce qu'exige un registre
hétérogène ; `async-trait` n'est pas dans le dépôt et n'est pas nécessaire.

---

## 6. Conséquences sur les stories 32-3 à 32-11 (AC à ajuster)

- **32-3** — AC ajouté : le service reçoit `Lecture` et `Trace` **typées et distinctes** (§ 4.4) ; la fiche inventorie
  l'ensemble clos des écritures sur les chemins de lecture des deux pilotes (`emit_report_audit` et sa variante
  d'export, `touch_last_used`) ; mutation « la trace part sur `Lecture` » tuée — l'échec y serait **avalé** par le
  `warn!` actuel, donc muet.
- **32-4a / 32-4b** — AC ajouté : aucun service de lecture n'appelle un `get_or_create_*` ni un dépôt qui écrit ;
  inventaire clos des appels.
- **32-5** — AC ajouté au manuel (32-7) : la clé se donne au client en portée `local`/`user`, jamais en `.mcp.json`
  versionné sauf `${VARIABLE}`.
- **32-6** — révision **`2025-11-25`** (liens du § 1.1), `2025-06-18` acceptée ; sans session (aucun `Mcp-Session-Id`),
  `GET`/`DELETE` 405, notification 202, lot 400, `ping` ; `MCP-Protocol-Version` non servi → **400 texte** (jamais
  `-32022`) ; `Origin` présent et étranger → 403, absent admis ; outil inconnu ou hors portée → `-32602` identique ;
  `readOnlyHint: true` sur tout outil ; corps borné. Pool de lecture : **fonction unique** des options (cf. [#669]),
  `after_connect` + `before_acquire` + **garde lexicale** ; test sur le chemin de production avec les deux mutations du
  § 4.3. Le module JSON-RPC n'expose aux outils que les types du § 5.3.
- **32-7** — le manuel dit la configuration des deux clients (§ 2.4), la limite de 25 000 jetons de Claude Code, et que
  le filtre `Origin` ne protège que d'un navigateur.
- **32-8** — forme : **écran de Kesh** ; attente **240 s** (constante nommée, sous les 300 s de Hermes et la fenêtre de
  5 min de Claude Code) ; réponse SSE, premier évènement immédiat, `X-Accel-Buffering: no`, progression toutes les 15 s
  (`notifications/progress` si `progressToken`, commentaire SSE sinon) ; la connexion de lecture est **rendue avant
  l'attente** ; expiration et refus → `isError: true`, rien ne part ; repli documenté (poignée explicite) si l'essai
  § 2.4 d échoue. **Avant la 32-8, Guy ou la 32-8 elle-même rejoue § 2.4 (d)** — c'est sa prémisse.
- **32-9a / 32-9b** — bornes de pagination choisies pour rester **sous 10 000 jetons** par réponse (avertissement de
  Claude Code) ; ressources MCP (`resources/read`) disponibles en `2025-11-25` pour le contenu d'une pièce
  (C-32-0-10) — capacité `resources` déclarée seulement si elle est servie.
- **32-10a** — `prompts/list`/`prompts/get` selon `2025-11-25` ; Claude Code les montre en commandes
  `/serveur:prompt (MCP)`, Hermes par `get_prompt` : le nom d'un prompt doit être court et sans espace.
- **32-11** — test de contrat sur la séquence du § 1.2 (initialize, initialized, tools/list, tools/call, outil inconnu,
  Origin, sans clé, sonde `2026-07-28`, lot, `GET`) ; parcours joué avec **les deux** clients et leurs versions ; les
  observations (a) à (h) du § 2.4 consignées.

## 7. Questions pour Guy

- **Q8 — Le SDK** : `rmcp` est officiel, Tier 1 et double ère ; on garde l'écriture à la main en 0.16.0 et on le
  réexamine quand un client exigera `2026-07-28` ? *Recommandation* : oui.
- **Q9 — Hermes** : s'agit-il bien de **Hermes Agent (Nous Research)** ? Si c'est un autre client, le § 2.2 et la
  révision figée sont à revoir.

[#669]: https://github.com/guycorbaz/kesh/issues/669
