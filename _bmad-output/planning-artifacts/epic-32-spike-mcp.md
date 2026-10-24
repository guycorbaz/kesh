# Epic 32 — Spike 32-2 : protocole, transport, validation avant envoi, lecture seule, SDK

**Date de l'essai : 2026-10-24.** Spike de décision : aucun code fusionné. Branche `story/32-2` (doc seulement), partie de
`6f0cee00`. Base de l'essai : `kesh_322` (démonstration, seed `scripts/seed-dev-db.sql`, 83 migrations du worktree),
MariaDB **10.11.19** (`10.11.19-MariaDB-ubu2204`), utilisateur `kesh` du fichier versionné
(`scripts/mariadb-init/01-dev-grants.sql` — `ALL` sur `kesh%`.* et `\_sqlx\_test%`.*, **aucun droit global**, aucun
`GRANT` posé). **Aucune donnée réelle**, `~/kesh-test/` jamais ouvert.

Choix au registre `implementation-artifacts/epic-32-choix-autonomes-322.md` (C-32-2-1 à 23). Journaux bruts des essais,
hors dépôt : `/home/gcorbaz/devel/kesh-gate-logs/322-spike/` — `ro-essai.sql` → `ro-essai.out` ; `ro-pool-main.rs` et
`ro-pool-Cargo.toml` → `ro-pool.out` ; `proto-main.rs` → `proto-curl.out` ; ils suffisent à rejouer chaque essai.
*(Relecture P2, L-9 : `proto-server.log`, cité jusqu'ici, est vide — 0 octet — et n'est plus compté parmi les journaux.)*

⚠️ **Relecture P1 (2026-10-24, `kesh-gate-logs/322-review-p1.md`, 0 CRITICAL / 0 HIGH / 7 MEDIUM / 7 LOW).** Deux faits
de la première rédaction étaient faux (YOLO de Hermes, protocole de Hermes), un était cité sans en tirer la conséquence
(tâche de fond de Claude Code), un mal expliqué (minuteur de premier octet de Claude Code — sa valeur de 60 s par défaut
était juste, cf. C-32-2-12) ; le raisonnement qui en dépendait est refait aux §§ 1.3, 2.3 et 3. **Trois décisions ne sont plus prises
par la spike : elles sont recommandées et remises à Guy** (Q10 à Q12, § 7) — la forme de la validation, la forme de la
lecture seule, et la portée de la validation. Registre : C-32-2-10 à 18.

⚠️ **Relecture P2 (2026-10-24, `kesh-gate-logs/322-review-p2.md`, 0 / 0 / 4 MEDIUM / 11 LOW).** La justification de
`2025-11-25` disait plus que les documentations des clients (§ 1.3, rendue à ce qui est établi) ; une phrase sur les
délais oubliait que l'élicitation tient l'appel ouvert (§ 2.3) ; la garantie « par construction » de la validation ne
couvrait que `tools/call` (§ 5.3, étendue à toute méthode servie, ressources non servies en 0.16.0) ; le bloc du § 4.2
portait des identifiants de connexion absents du journal (recopié depuis `ro-pool.out`). Registre : C-32-2-19 à 23.

---

## Les cinq décisions, en une ligne chacune

1. **Révision figée : `2025-11-25`** (revision *Final*), servie **sans session**, réponses JSON — `2025-06-18` acceptée à
   la négociation ; la révision *Current* `2026-07-28` (sans `initialize`) n'est **pas** servie en 0.16.0 : les deux
   clients savent la parler, mais certains de leurs réglages documentés n'ont **que** la poignée de main `initialize`
   (Hermes `legacy`, Claude Code v1 et `legacy`), les autres la tentent ou — Claude Code v2, d'après la matrice de la
   spécification, à éprouver — s'y replient (§ 1.3). Ce qui n'est **pas** établi : quelle
   `protocolVersion` chaque client demande, et s'il accepte `2025-11-25` en réponse — observation (h) du § 2.4, à faire
   avant la 32-6.
2. **Clients** : Claude Code (2.1.310 installé) et Hermes Agent se branchent tous deux, d'après leur documentation, en
   Streamable HTTP avec un en-tête `Authorization` fixe ; **rien n'a été essayé avec un client réel** (consigne) — les
   gestes pour l'éprouver sont au § 2.4.
3. **Validation avant envoi — à trancher par Guy (Q10)** entre trois formes : élicitation dans le client, appel tenu
   ouvert au plus N s pendant la validation dans Kesh, ou **poignée** (l'outil rend aussitôt un identifiant,
   l'utilisateur valide dans un écran de Kesh, l'agent relit le résultat). *Recommandation* : la poignée (§ 3.4).
   L'élicitation n'est pas recommandée pour des raisons d'ergonomie, de fonctionnement sans session interactive et de
   réemploi — **non** parce qu'elle serait contournable (§ 3.2). Sa portée (chaque appel ?) : Q12.
4. **Lecture seule** : essai réel — une session `SET SESSION TRANSACTION READ ONLY` refuse `INSERT`, `UPDATE`, `DELETE`,
   `SELECT … FOR UPDATE`, DDL et table temporaire en **1792 (25006)** ; **mais une instruction sans privilège la défait**
   (quatre formes) et sqlx **ne remet pas** l'état de session à zéro entre deux emprunts. Ce verrou n'est donc pas une
   garantie **par construction** ; seul un **utilisateur MariaDB limité au `SELECT`** en est une. **La forme reste à
   trancher par Guy (Q11)** ; *recommandation* : l'utilisateur `SELECT`, contrôlé au démarrage, plus le mode de session
   en complément (§ 4.3).
5. **SDK : écrit à la main, derrière des traits à types propres à Kesh** (décision de Guy maintenue) — `rmcp` 3.7.0
   (2026-10-22) est officiel, *Tier 1*, double ère et intégrable à Axum, mais a connu **trois versions majeures en moins
   de cinq mois** (2026-03-03 → 2026-07-28) ; il redevient candidat si Kesh doit servir `2026-07-28` (§ 5.2, Q8).

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
`kesh-gate-logs/322-spike/proto-main.rs`. Sortie complète : `proto-curl.out`. Extraits **remis en forme**, non
recopiés octet pour octet *(précisé en relecture P2 — la mention « en-tête `Date` retiré » était fausse : le journal n'en
porte aucun)* : en-têtes et corps rapprochés sur une ligne, `content-length` omis, champs longs abrégés par `…`, les
mentions entre parenthèses sont des commentaires ; codes, statuts et identifiants JSON-RPC sont ceux du journal ; les essais 3
(`tools/list`, l. 13-18) et 7 (`prompts/get`, l. 41-46) ne sont pas repris :

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
- **Pourquoi pas `2026-07-28`** — *raisonnement refait en relecture P1 (C-32-2-11), ramené en relecture P2 à ce que
  disent les sources (C-32-2-19)* : **les deux clients savent parler les deux ères**. Hermes a un réglage `protocol` dont
  le défaut est `auto` — « legacy `initialize` handshake first, falling back to the 2026-07-28 `server/discover`
  stateless probe when the server rejects the handshake as modern-only » —, avec `stateless` (« probe
  `server/discover` first; one legacy retry ») et `legacy` (« handshake only, no fallback »)
  (`…/docs/reference/mcp-config-reference`) ; Claude Code a deux runtimes : le v2 « adds MCP protocol revision
  2026-07-28 », « Asks HTTP, stdio, and claude.ai connector servers whether they support the newer revision and uses it
  with those that do », puis « […] connects to every other server as v1 does » (la page ne dit pas si « every other
  server » désigne les serveurs qui ont décliné ou les autres transports) ;
  `MCP_PROTOCOL_NEGOTIATION=legacy` « keeps every server on » la poignée de main antérieure (§ 2.1).
  - **Établi par les documentations** : Hermes **tente ou retente** la poignée de main `initialize` dans ses trois
    réglages (`auto` d'abord, `stateless` une fois, `legacy` seulement) ; Claude Code la garde en `legacy` et en v1. Pour
    le v2 face à un serveur de l'ère `initialize`, le repli n'est pas écrit en toutes lettres : c'est ce que la matrice de
    compatibilité attend d'un client double ère (« Dual-era | Legacy | Works »), **à éprouver** (§ 2.4 e). Trois réglages
    **n'ont qu'elle** (Hermes `legacy`, Claude Code v1 — *déduit* de ce que le v2 « ajoute » la `2026-07-28` —, Claude
    Code `legacy`). Un serveur moderne seul est donc exclu de ces réglages ; un serveur de l'ère `initialize` ne l'est
    d'aucun réglage documenté, **au niveau de l'ère**, sous la réserve du repli du v2.
  - **Non établi** : quelle `protocolVersion` chaque client demande à `initialize` — aucune des deux pages ne le dit — ni
    s'il accepte `2025-11-25` en réponse. Or la spécification veut que le serveur réponde une autre version qu'il sert si
    la demandée n'en est pas une, et que le client « SHOULD disconnect » s'il ne la connaît pas
    (`…/2025-11-25/basic/lifecycle`, *Version Negotiation*). Un client qui demanderait `2025-03-26` recevrait
    `2025-11-25` et **pourrait se déconnecter**. L'affirmation « servi par les deux clients dans chacun de leurs
    réglages » de la relecture P1 allait donc au-delà des sources : elle vaut pour l'ère, **non pour la révision**.
  - **Pourquoi `2025-11-25` reste le choix, et à quelle condition** : c'est la révision *Final* la plus récente de l'ère
    `initialize`, la seule ère que tous les réglages décrits atteignent ; Kesh accepte aussi `2025-06-18`, la
    précédente, qui couvre un client resté sur l'avant-dernière révision. Le choix tient **sous réserve de l'observation (h) du § 2.4**,
    à faire avant la 32-6 : si un client demande une révision plus ancienne, l'ajouter à la liste acceptée est
    réversible et local, **à condition** que les méthodes servies par Kesh y aient la même forme (non vérifié pour
    `2025-03-26`). Un serveur double ère marcherait partout, mais double le protocole écrit à la main. Le premier
    `initialize` de Hermes en `auto` et le repli de Claude Code v2 restent **à éprouver** (§ 2.4 e, h).
- **Sans état** : aucun `Mcp-Session-Id` émis, ni lu ; `GET` et `DELETE` → **405** ; `notifications/*` → **202** ;
  `ping` → `{}` ; lot → **400** ; corps borné (1 Mio proposé). *Éprouvés par le prototype* (`proto-curl.out`) : sans
  session, `GET` 405, notification 202, lot 400. **Décidés mais non éprouvés** : `DELETE` 405 (le prototype ne monte que
  `POST`, axum rendrait 405, aucun `curl` ne l'essaie), `ping` (codé, jamais appelé), la borne de 1 Mio (absente du
  prototype).
- **`MCP-Protocol-Version`** : présent et non servi (dont `2026-07-28`) → **400, corps texte**, *jamais* un code
  `-32022` (sinon un client double ère croirait parler à un serveur moderne et ne se replierait pas) — éprouvé
  (`proto-curl.out` n° 10) ; absent → admis (le seul effet de « assume `2025-03-26` » serait ici le même jeu de
  méthodes), journalisé — **décidé, non essayé** (le prototype le laisse passer sans qu'aucun `curl` l'omette
  délibérément).
- **Réponses** : `application/json` partout — **et partout sans exception si Guy retient la poignée** (Q10) ; dans la
  forme « appel tenu ouvert » seulement, l'appel en attente de validation (32-8) répond en `text/event-stream`, un
  premier évènement immédiat, `X-Accel-Buffering: no`, une progression périodique.
- **`Origin`** : **absent → admis** (un client en ligne de commande n'en envoie pas — à éprouver, § 2.4) ; **présent et
  différent des origines de Kesh lui-même** (l'URL publique configurée, et `http://127.0.0.1:<port>` /
  `http://localhost:<port>`) → **403**, corps JSON-RPC sans `id`. La clé `Authorization` reste la vraie barrière : une
  page piégée par *DNS rebinding* ne la connaît pas. *Précision (L5)* : si l'URL publique n'est pas configurée, un
  navigateur qui joint le NAS par son adresse du réseau local (`http://192.168.x.y:<port>`) est refusé — sans
  conséquence pour les clients en ligne de commande, qui n'envoient pas d'`Origin`. **Ne pas** admettre « l'origine qui
  correspond à l'en-tête `Host` » pour combler ce cas : sous *DNS rebinding*, `Origin` et `Host` portent tous deux le
  nom de l'attaquant et concorderaient. Si une origine du réseau local doit être admise, c'est une liste réglable
  (32-5/32-6). La spécification n'impose rien pour un `Origin` absent : « absent admis » est conforme.
- **Outil inconnu ou hors portée** : erreur JSON-RPC **`-32602`**, même message — conforme à l'exemple « Unknown tool »
  de la spécification et à l'AC « aucun oracle » de la 32-6. Une erreur **d'exécution** d'un outil reste un résultat
  `isError: true`, à texte fixe (§ 5.3). Méthode non servie (dont `resources/*`) → **`-32601`**.
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
  `MCP_SDK_GENERATION=v1|v2`, `MCP_PROTOCOL_NEGOTIATION=auto|legacy`. Le v2 est le runtime « on Claude Code v2.1.232 or
  later » dans les sessions qui lisent les *feature flags* ; dans celles qui ne les lisent pas, « by default on Claude
  Code v2.1.274 or later », la page citant pour exemples les sessions sur Amazon Bedrock, Claude Platform on AWS,
  Google Cloud's Agent Platform ou Microsoft Foundry (« unless a host platform that embeds Claude Code sets
  `CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST` »), celles ouvertes par une passerelle Claude apps et celles où la télémétrie
  ou la lecture des *flags* est coupée ; elle ne dit pas quel runtime vaut en deçà de ces versions *(précisé en
  relecture P2, L-3)*. → la version exacte qu'un runtime demande à `initialize` n'est pas écrite dans la page, ni ce
  que fait le v1 d'un serveur moderne seul (observation (h) du § 2.4).
- **Élicitation** : supportée, formulaire et URL (« Claude Code displays an interactive dialog ») ; mais : « To
  auto-respond to elicitation requests without showing a dialog, use the `Elicitation` hook » (le *hook* rend
  `accept`/`decline`/`cancel` avec un `content`, `https://code.claude.com/docs/en/hooks`) ; pour les tâches MCP, « In a
  non-interactive session, Claude Code asks the server to cancel a task that requests input, without running your
  `Elicitation` hooks » ; et « A call waiting on an open elicitation dialog isn't backgrounded while the dialog is
  open ».
- **Délais d'un `tools/call`** — *relus en relecture P1 (C-32-2-12)* :
  - « The per-server `timeout` is a hard wall-clock limit per tool call, and progress notifications from the server
    don't extend it » ; sans `timeout` par serveur ni `MCP_TOOL_TIMEOUT`, le défaut est « about 28 hours ».
  - Minuteur par requête jusqu'au **premier octet** de la réponse, pour « an HTTP, SSE, or claude.ai connector
    server » seulement (« Stdio and WebSocket servers have no per-request timer » — *portée ajoutée en relecture P2,
    L-2* ; Kesh est servi en HTTP) : « Claude Code sets that timer to the greatest of
    three values: 60 seconds, the tool timeout that applies to the server, and `MCP_TIMEOUT`. The 28-hour default of an
    unset `MCP_TOOL_TIMEOUT` doesn't enter that comparison ». **Avec les réglages par défaut, il vaut donc 60 s** (il ne
    s'allonge que si l'utilisateur fixe un délai d'outil ou `MCP_TIMEOUT` plus long) ; il porte sur le **premier octet**,
    si bien qu'une réponse qui commence aussitôt — un JSON immédiat, ou les en-têtes et un premier évènement SSE — n'y est
    jamais soumise.
  - Inactivité : « A tool call to an MCP server that sends no response and no progress notification for the idle
    window aborts » — **5 min** en HTTP (`CLAUDE_CODE_MCP_TOOL_IDLE_TIMEOUT`).
- **Passage en tâche de fond** — *conséquence tirée en relecture P1 (C-32-2-13)* : « An MCP tool call in the main
  conversation that is still running after two minutes moves to a background task instead of blocking the session.
  Claude receives the task ID immediately and keeps working, and the result arrives as a task notification when the call
  settles » ; « If Claude is idle when the result arrives, Claude starts on it unprompted » ; la tâche « doesn't survive
  exiting the session » ; seuil réglable par `CLAUDE_CODE_MCP_AUTO_BACKGROUND_MS` (`0` : jamais). En session non
  interactive, pas de passage en tâche de fond (sauf `CLAUDE_AUTO_BACKGROUND_TASKS=1`). Autrement dit : **au-delà de
  2 min, Claude Code transforme de lui-même un appel bloquant en poignée** — l'agent continue sans le résultat et le
  reçoit plus tard, tant que la session vit.
- **Taille** : avertissement au-delà de 10 000 jetons, **limite 25 000** (`MAX_MCP_OUTPUT_TOKENS`).
- **Prompts** : « Claude Code lists each MCP prompt as `/servername:promptname (MCP)` ».

### 2.2 Hermes — `https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp` et `…/docs/reference/mcp-config-reference`

⚠️ **« Hermes » est ambigu** : j'ai retenu **Hermes Agent de Nous Research** (seul client MCP de ce nom trouvé), **à
confirmer par Guy** (C-32-2-9). Rien n'est installé sur la station (`which hermes` vide, pas de `~/.hermes`).

- Configuration `~/.hermes/config.yaml`, clé `mcp_servers.<nom>` : `url`, `headers` (`Authorization: "Bearer ***"`),
  `transport` : « Set to `sse` to use the SSE transport instead of Streamable HTTP ». La référence **ne nomme pas** de
  transport par défaut ; « Streamable HTTP faute de `transport: sse` » est une lecture de cette phrase, **non sourcée**
  en toutes lettres *(relecture P2, L-4)* — à éprouver (§ 2.4 a).
- **Délais** : « `timeout` — Tool call timeout in seconds (default: `300`) », `connect_timeout` 60 s (« Initial
  connection timeout »). La page ne dit pas si ce délai est une coupure dure ou s'il est réarmé par une progression
  (**non sourcé**, *relecture P2, L-4*) ; le document le traite comme une coupure dure, l'hypothèse prudente.
- **Protocole** — *rectifié en relecture P1 (C-32-2-11)* : réglage `protocol`, défaut **`auto`** (« legacy `initialize`
  handshake first, falling back to the 2026-07-28 `server/discover` stateless probe when the server rejects the
  handshake as modern-only » — *citation complétée en relecture P2, L-5*), `stateless` (« probe `server/discover` first; one
  legacy retry »), `legacy` (« handshake only, no fallback »). **Hermes parle donc les deux ères** ; la première
  rédaction l'avait déduit *legacy* seul de la mention du SDK Python 1.x, à tort.
- **SDK** : « mcp Python SDK ≥ 1.11.0 » (pour l'élicitation).
- **Élicitation** : formulaire, routée vers « an interactive prompt in the CLI/TUI, or approval buttons on gateway
  platforms like Telegram and Slack », `elicitation.timeout` 300 s ; « URL-mode elicitations […] are declined as
  unsupported ». Sans personne pour répondre (cron, `hermes chat -q`), la documentation ne dit rien pour l'élicitation
  — seule la borne de 300 s est écrite.
- **Approbation** : `trust: full` par défaut ; en `untrusted`, les appels capables d'écrire demandent approbation, et
  « Tools annotated `readOnlyHint: true` stay unprompted unless the budget requires it ». YOLO (`/yolo`, `--yolo`) et
  `approvals.mode: off` sautent cette approbation — **mais pas l'élicitation** (*rectifié en relecture P1,
  C-32-2-10*) : « YOLO and `approvals.mode: off` do not answer these for you: the server is asking you a question, not
  asking permission ».
- **Prompts et ressources** : exposés par des outils d'emballage (`list_prompts`, `get_prompt`, `list_resources`,
  `read_resource`).

### 2.3 Ce que ces deux documentations tranchent déjà

- **Révision** *(rectifié en relecture P2, C-32-2-19)* : l'ère `initialize` est atteinte par chacun des réglages
  documentés (Hermes `auto` la tente d'abord, `stateless` la retente, `legacy` n'a qu'elle ; Claude Code v1 et `legacy`
  n'ont qu'elle, le v2 est censé s'y replier — à éprouver) ; les deux savent aussi parler `2026-07-28`. Quelle
  **révision** de cette ère chacun demande, et s'il accepte `2025-11-25`, **n'est pas écrit** : observation (h) du § 2.4
  → C-32-2-1, justification refaite par C-32-2-11 puis C-32-2-19.
- **Délais, pour un appel qui reste ouvert** — les formes **(a)** (l'élicitation attend la réponse de l'utilisateur) et
  **(b)** du § 3 ; *rectifié en relecture P2 (C-32-2-20)*, la phrase précédente ne nommait que (b) :
  - *Hermes* : `timeout` d'un appel d'outil 300 s par défaut (nature de la coupure non écrite, § 2.2) ; pour (a),
    `elicitation.timeout` 300 s en plus — la page ne dit pas lequel l'emporte quand les deux courent.
  - *Claude Code* : premier octet sous 60 s par défaut (satisfait par une réponse qui commence aussitôt — pour (a), le
    flux SSE qui porte `elicitation/create`) et une activité toutes les 5 min ; **passage en tâche de fond à 2 min** en
    session interactive pour (b) — l'agent continue alors sans le résultat (§ 2.1) —, **pas** pour (a) tant que le
    dialogue est ouvert (« isn't backgrounded while the dialog is open »). Qu'un dialogue ouvert compte comme activité
    au regard des 5 min n'est pas écrit (**non établi**).
  - Seule la forme **(c)** répond aussitôt : aucun de ces délais ne la concerne.
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
client envoie-t-il un `Origin` ? (c) un `progressToken` dans le `_meta` de `tools/call` ? (d) **selon la forme retenue
(Q10)** — *forme (b), appel tenu ouvert* : le minuteur de premier octet se juge **au premier octet**, non à la durée
totale *(rectifié en relecture P2, L-1)* — d'où un couple d'essais : `attendre_validation` à **90 s** avec premier
évènement SSE **immédiat** (l'appel ne doit pas être coupé à 60 s : c'est ce qui distingue un minuteur de premier octet
d'un minuteur de réponse complète), et un contrôle négatif, premier évènement **retardé à 70 s** (l'appel doit être
coupé vers 60 s ; le prototype devra recevoir ce retard en argument, il ne l'a pas) ; puis à **150 s** (Claude Code le passe-t-il en tâche de fond, et
l'agent reçoit-il le résultat ensuite ?), puis à **280 s** (sous les 300 s de Hermes) ; les commentaires SSE
`: attente` comptent-ils comme activité pour Claude Code (sa doc ne nomme que la réponse et `notifications/progress`) ?
*Forme (c), poignée* : l'agent, sur la seule description de l'outil, rappelle-t-il l'outil de lecture du résultat, et
combien de fois s'il est encore « en attente » ? (e) Claude Code v2 se replie-t-il après le `400` texte de la sonde
moderne (journal `claude --debug`), et Hermes en `protocol: auto` s'arrête-t-il bien à `initialize` ? (f) un `401`
déclenche-t-il une tentative OAuth (découverte `/.well-known/oauth-protected-resource`) plutôt qu'un message clair ?
(g) le prompt apparaît-il comme `/kesh-demo:bilan_commente (MCP)` ? (h) quelle `protocolVersion` chaque client
demande-t-il à `initialize` (si ce n'est ni `2025-11-25` ni `2025-06-18`, Kesh répond `2025-11-25` et le client peut
refuser) ?

---

## 3. La validation avant envoi

*Section refaite en relecture P1 (C-32-2-14, C-32-2-15) : la première rédaction comparait deux formes et écartait
l'élicitation sur trois motifs dont un et demi était faux ; elle reléguait la poignée en « repli » sans la peser.*

### 3.1 Les trois formes

- **(a) Élicitation MCP** — pendant le `tools/call`, Kesh demande au client `elicitation/create` (en `2025-11-25` : sur le
  flux SSE de la réponse, « The server **MAY** send JSON-RPC *requests* and *notifications* before sending the JSON-RPC
  *response* » (*citation rétablie en relecture P2, L-5*) ; en
  `2026-07-28` : `InputRequiredResult`, le client rejoue sa requête avec `inputResponses`, motif MRTR, sans état). Le
  client montre un formulaire ; la réponse revient ; Kesh rend le contenu accepté.
- **(b) Appel tenu ouvert au plus N s** — Kesh garde la proposition en mémoire, l'utilisateur la valide (telle quelle,
  modifiée, ou refusée) dans un écran de Kesh, et l'appel, resté ouvert en SSE entretenu, rend le contenu approuvé ou
  « refusé » / « non validé dans les N s ».
- **(c) Poignée** — l'outil rend **aussitôt** un résultat sans donnée : « envoi n° X en attente de validation dans
  Kesh » ; l'utilisateur valide (tel quel ou modifié) ou refuse dans un écran de Kesh, à son rythme ; l'agent appelle un
  outil de lecture du résultat (`kesh_lire_envoi(X)`, même clé, même société) qui rend « en attente », le contenu
  approuvé, ou « refusé » / « expiré ». C'est le motif que la `2026-07-28` donne elle-même pour remplacer l'état de
  session : « explicit, server-minted handles passed as ordinary tool arguments ».

### 3.2 Ce qui reste du rejet de l'élicitation (C-32-2-14)

La première rédaction l'écartait sur trois motifs. Relus :

1. **« Contournable par le client lui-même — Claude Code par un *hook*, Hermes par YOLO. »** **À moitié faux** : YOLO
   « do[es] not answer these for you » (C-32-2-10). Reste le *hook* `Elicitation` de Claude Code, qui peut répondre
   `accept` avec un `content` sans montrer de dialogue — mais c'est **l'utilisateur** qui le pose dans la configuration
   de son propre client, non l'agent. Ce qui demeure est un argument **de principe**, non un contournement : avec
   l'élicitation, la garantie que l'utilisateur a vu est tenue par le logiciel qui reçoit les données et par sa
   configuration, que Kesh ne voit pas ; avec (b) ou (c), elle est tenue dans Kesh. Pour un seul utilisateur qui
   configure lui-même ses clients, **ce poids est faible**.
2. **« Formulaires plats. »** **Tient**, et c'est désormais l'argument principal : le schéma d'élicitation est fait
   de « flat objects with primitive properties only ». Montrer et faire modifier un extrait de grand livre revient à un seul
   champ texte de plusieurs lignes, dans un dialogue de terminal (Claude Code) ou une interface TUI, Telegram ou Slack
   (Hermes). Faisable, pauvre pour relire et corriger un tableau. **Non essayé.**
3. **« Elle impose un flux serveur → client que la révision sans état n'a pas. »** **Faux tel qu'écrit** (L1) : en
   `2025-11-25`, la requête part sur le flux SSE du `tools/call` et la réponse du client revient par un `POST`, que Kesh
   corrèle à l'appel en attente par l'identifiant de la requête — faisable sans session, non essayé ; en `2026-07-28`,
   MRTR est précisément le mécanisme sans état. Ce qui reste est un **coût** : deux implémentations le jour où Kesh
   servirait aussi `2026-07-28`.

Ce que la relecture ajoute :

4. **Hors session interactive** : Claude Code annule une tâche MCP qui demande une saisie (« without running your
   `Elicitation` hooks ») ; pour Hermes sans personne pour répondre (cron, `hermes chat -q`), la documentation est
   muette. La poignée (c) ne dépend d'aucune interface du client : la validation peut venir plus tard, ou jamais, et
   alors rien ne part.
5. **Réemploi en 0.17.0** : les **propositions** de l'agent seront « acceptées, modifiées ou refusées dans un écran de
   Kesh » (décision de Guy). Un écran de validation de Kesh en 0.16.0 en est la première moitié ; l'élicitation ne l'est
   pas.
6. **En faveur de l'élicitation, honnêtement** : l'utilisateur valide sans quitter la fenêtre de son client ; Kesh n'a
   aucun écran à construire en 0.16.0 ; et Claude Code ne fait pas passer en tâche de fond un appel dont le dialogue est
   ouvert.

**Conclusion** : le rejet ne repose plus sur un contournement, mais sur l'**ergonomie de la modification** (2), le
**fonctionnement sans session interactive** (4) et le **réemploi pour les propositions** (5). C'est une **préférence
raisonnée, non une impossibilité** ; l'élicitation peut compléter l'écran plus tard (en mode URL vers la page de
validation de Kesh — Claude Code seulement, Hermes refuse le mode URL).

### 3.3 Les trois formes, confrontées aux faits

| Critère | (a) Élicitation | (b) Appel tenu ouvert ≤ N s | (c) Poignée |
|---|---|---|---|
| **Délais des clients** | l'appel reste ouvert pendant que l'utilisateur répond : Hermes, 300 s pour répondre (`elicitation.timeout`, et `timeout` d'outil 300 s) ; Claude Code : premier octet satisfait par le flux SSE ouvert aussitôt, pas de tâche de fond tant que le dialogue est ouvert, inactivité de 5 min pendant le dialogue **non établie** | Hermes : 300 s (coupure supposée dure, non écrite) → N ≤ 240 s ; Claude Code : premier octet sous 60 s (satisfait par un évènement immédiat), activité toutes les 5 min, **tâche de fond à 2 min** (l'agent continue sans le résultat, le reçoit ensuite si la session vit) | **aucun** : chaque appel répond aussitôt |
| **Mandataires** (Traefik devant une partie des conteneurs) | flux SSE tenu ouvert : tampon et délais d'inactivité du mandataire à régler | idem, pendant N s ; `X-Accel-Buffering: no` ne vaut que pour nginx ; Traefik **non vérifié** | **aucun** : requêtes courtes ordinaires |
| **Expérience utilisateur** | dialogue dans le client, champs plats ; modifier un tableau = un champ texte | passer dans Kesh et valider **dans les N s**, sinon tout est perdu ; l'agent bloqué (Hermes) ou poursuivant sans le résultat (Claude Code) | valider dans Kesh **à son rythme** ; puis l'agent relit (seul, ou quand on le lui dit) — **un tour d'agent de plus** |
| **Simplicité côté Kesh** | SSE, corrélation d'un `POST` à un appel en attente, état en mémoire ; pas d'écran ; à réécrire pour la `2026-07-28` | SSE, progression, minuteur, état en mémoire, écran qui doit apprendre en direct qu'une proposition attend | JSON seul ; **une table** (proposition, état, contenu approuvé, échéance), un outil de plus, une purge à l'échéance ; un écran ; **migration** (🗄️) |
| **Testabilité** | test de contrat avec un client qui répond ; comportements des clients à la main | minuteurs, SSE, concurrence (tests avec N court) ; 120 s et 300 s seulement à la main | requête/réponse déterministe, de bout en bout au `curl` et en test de contrat ; rien n'attend |
| **Ce que l'agent doit savoir faire** | rien | rien chez Hermes ; chez Claude Code, la notification de tâche (automatique) | **rappeler l'outil de lecture** après validation — consigne dans la description de l'outil et dans les `instructions` d'`initialize` ; risque : interrogations répétées (jetons) ou oubli |
| **Hors session interactive** | annulée (Claude Code, tâches) ; non documenté (Hermes) | Claude Code `-p` attend sans tâche de fond ; possible si quelqu'un valide dans les N s | fonctionne ; une exécution unique qui se termine ne lit simplement jamais le résultat — rien ne part |
| **Redémarrage de Kesh** | attente perdue, rien ne part | attente perdue, rien ne part | **survit** (en base) |
| **Révision `2026-07-28`** | à réécrire (MRTR) | **non établi** : la révision supprime la reprise SSE et le `GET` ; qu'une réponse SSE à un `POST` y reste admise n'a pas été vérifié (*relecture P2, L-11*) | **native** (poignée explicite) |
| **Qui garantit que l'utilisateur a vu** | le client et sa configuration | Kesh | Kesh |
| **Réemploi pour les propositions (0.17.0)** | non | l'écran | **l'écran et la table** |

Coût propre à (c) : la proposition approuvée est une **copie de données réelles dans une table de plus** — dans Kesh,
donc conforme à la règle de Guy, mais à purger à l'échéance ; sa place dans la sauvegarde est à décider avec la 32-12
(une table de plus change l'inventaire que compare l'import).

### 3.4 Recommandation (C-32-2-15) — **à trancher par Guy (Q10)**

**La poignée (c).** Elle est la seule forme qu'aucun délai de client ni aucun mandataire ne contraint, qui survit à un
redémarrage, qui se teste de bout en bout sans minuteur, qui est native en `2026-07-28` et qui pose déjà la table et
l'écran des propositions de la 0.17.0. Son coût — un tour d'agent de plus, et un agent qui doit savoir rappeler — est
réel mais visible : s'il oublie, rien ne part, ce qui est le bon côté de l'échec. Le comportement de l'agent est
l'observation (d) du § 2.4.

Si Guy retient **(b)** : N = **240 s** (constante nommée), sous les 300 s de Hermes ; premier évènement SSE immédiat,
progression toutes les 15 s (`notifications/progress` si `progressToken`, commentaire SSE sinon), connexion de lecture
**rendue avant l'attente** ; et le manuel dit qu'au-delà de 2 min Claude Code poursuit sans le résultat. Si Guy retient
**(a)** : les motifs 2 et 4 du § 3.2 sont à écrire comme limites acceptées.

Dans les trois cas : refus et échéance → résultat `isError: true`, **rien ne part** ; le contenu reçu par le client est
exactement celui validé (AC de la 32-8).

---

## 4. La lecture seule — essai réel

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

*Recopié en relecture P2 (C-32-2-22)* : la version précédente de ce bloc portait des identifiants de connexion
(`313122`, `313126`…`313130`) **absents du journal**, et abrégeait les lignes. Ci-dessous, les lignes citées sont
celles de `ro-pool.out`, octet pour octet, avec leur numéro de ligne en tête de chaque morceau ; les seules lignes
ajoutées sont les marques `[…]` entre crochets, qui disent ce qui est omis.

```
[l. 1-10]
== A. pool lecture seule, 3 connexions physiques simultanées
conn 0 id=313133 tx_read_only=1
  autocommit INSERT : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  autocommit UPDATE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  autocommit DELETE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  tx INSERT : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  tx UPDATE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  tx DELETE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
  SELECT : OK (1 société)
conn 1 id=313134 tx_read_only=1
[… l. 11-17 : les six mêmes refus 1792 et SELECT OK, pour conn 1 …]
[l. 18]
conn 2 id=313135 tx_read_only=1
[… l. 19-25 : idem pour conn 2 …]
[l. 26-37]
  pool.begin() + UPDATE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
== B. une connexion rendue au pool garde son état de session (max 1)
  SET SESSION TRANSACTION READ WRITE : OK (0 lignes)
  après réacquisition, tx UPDATE : OK (1 lignes)
== C. idem avec before_acquire qui vérifie @@tx_read_only (max 1)
  conn avant=313137 après=313138 ; tx UPDATE : ERREUR Some(1792) Cannot execute statement in a READ ONLY transaction
== D. connexion perdue (KILL) : la neuve repasse after_connect
  KILL 313139 : OK (0 lignes)
  nouvelle conn id=313141 tx_read_only=1
== E. pool ordinaire (sans after_connect) : écrit
  tx UPDATE : OK (1 lignes)
== lignes 'spike' au journal d'audit : 0
```

Lecture : `after_connect` s'applique à **chaque connexion physique**, y compris celle qui remplace une connexion perdue
(D) ; `BEGIN` de sqlx hérite du mode de session (A) ; **sqlx ne remet pas l'état de session à zéro** quand une
connexion revient au pool (B) — une seule instruction fautive sur le pool de lecture le rendrait inscriptible pour la
suite de sa vie ; un `before_acquire` qui relit `@@session.tx_read_only` et **jette** la connexion qui n'est plus à `1`
ferme ce trou **entre deux emprunts** (C), au prix d'un aller-retour par emprunt — **pas au sein d'un même emprunt** :
une instruction fautive suivie d'une écriture, dans le même emprunt, passe.

### 4.3 Trois formes, et ce que chacune garantit (C-32-2-16) — **à trancher par Guy (Q11)**

*Refait en relecture P1 : la première rédaction écrivait « lecture seule par construction » d'un verrou qu'une
instruction sans privilège défait, et écartait la seule forme qui l'est vraiment sans dire la vraie raison.*

**Commun aux trois** : un **second pool**, construit **dans `main.rs` sur le chemin de production**, par **une seule
fonction** de `kesh-db` qui fabrique les options des deux pools (réglages de session communs) — ce qui règle au passage
[#669] (la production n'applique ni `sql_mode` ni `time_zone`), ou s'y adosse si la 32-6 la laisse à part. Kesh
**n'exécute jamais de SQL fourni par l'agent** : le risque visé est un défaut du code de Kesh (un service qui écrit sur
le mauvais pool, une instruction fautive), non une attaque de l'agent.

**(a) Pool en mode de session `READ ONLY` + `before_acquire` + garde lexicale.**
- *Garantit* : aucune écriture **tant que le code de Kesh n'émet aucune des quatre formes du § 4.1** — c'est une
  garantie **par discipline outillée, non par construction**.
- *Limites* : `before_acquire` ne contrôle qu'**à l'emprunt** — au sein d'un même emprunt, `START TRANSACTION READ
  WRITE; UPDATE …` passe (et ne change même pas `@@tx_read_only`) ; la garde lexicale ne couvre que le texte de
  `crates/`, ni un SQL composé à l'exécution, ni une dépendance. Elle interdit `READ WRITE`, `tx_read_only` et
  `transaction_read_only` (inconnue de MariaDB 10.11, mais d'une version future peut-être) dans `crates/` hors du test
  qui la tient ; l'état au kickoff est vierge : `grep -rniE "read write|tx_read_only|transaction_read" crates` ne rend
  **aucune ligne** (*rectifié en relecture P1, L3* : la première rédaction disait « que des `API_KEY_READ_ONLY` », qui
  ne contient aucun de ces motifs).
- *Coût d'installation* : aucun.

**(b) Un second utilisateur MariaDB limité au `SELECT`** (`GRANT SELECT ON <base>.*`).
- *Garantit* : le **serveur** refuse toute écriture, quel que soit le SQL que Kesh émet — c'est la seule forme **par
  construction**. Refus attendu en `1142` ; **non essayé dans la spike** (aucun utilisateur créé, consigne).
- *La vraie contrainte* : les scripts d'initialisation (`/docker-entrypoint-initdb.d`, monté depuis
  `scripts/mariadb-init` par `docker-compose.dev.yml:116`) **ne s'exécutent qu'à la création du répertoire de données** :
  sur la base existante du NAS, l'utilisateur se crée **à la main**, avec un compte MariaDB qui a le droit de créer des
  utilisateurs (root) ; `docker-compose.prod.yml` n'a pas de service MariaDB (base fournie par l'exploitant) : à la
  main aussi ; et les `MARIADB_*` de `docker-compose.yml` (l. 12-15) ne servent qu'à la création — la documentation de l'image
  officielle (`https://hub.docker.com/_/mariadb`, consultée le 2026-10-24) : elles « will have no effect if you start
  the container with a data directory that already contains a database », et l'initialisation a lieu « When a container
  is started for the first time » *(source rectifiée en relecture P2, L-10 : `.env.example`, cité jusqu'ici, ne le dit
  pas)*. **Kesh ne
  peut pas le créer lui-même** : `CREATE USER` est un droit global et l'utilisateur `kesh` n'en a aucun (déduit du
  fichier de droits et du `1227` sur `SET GLOBAL`, § 4.1 ; `CREATE USER` non essayé).
- *Une variable de plus dans le `.env`* (nom à fixer, p. ex. `KESH_MCP_DATABASE_URL`), avec un second mot de passe.
  Guy veut que le `.env` ne porte à terme que l'accès à la base : **c'en est bien un** — compatible avec cette cible par
  nature, mais c'est un second secret et une étape manuelle de plus à la procédure de mise à jour du manuel
  d'administration.
- *Ce qu'il ne garantit pas* : `SELECT` sur toute la base lit aussi `users` et `api_keys` (empreintes) — la garantie est
  « aucune écriture », non « rien de sensible lisible » ; des droits table par table seraient à tenir à chaque
  migration (à éviter). Et des droits élargis par erreur l'annuleraient **en silence** — d'où, si (b) est retenu, un
  **contrôle au démarrage** : `SHOW GRANTS FOR CURRENT_USER()` sur le second pool ne doit rendre que `USAGE` et
  `SELECT`, sinon le serveur MCP refuse de s'activer (testable).
- *Tests* : la 32-6 a besoin d'un utilisateur `SELECT` en développement — une ligne de plus dans
  `scripts/mariadb-init/01-dev-grants.sql` (droit `SELECT` sur `kesh%`.* et `\_sqlx\_test%`.*), qui ne prend effet qu'à
  la **recréation** du conteneur de dev, geste interdit aux agents (`docs/agents-orchestration.md` § 2) : c'est à Guy.

**(c) Les deux** : (b) pour la garantie, plus `after_connect` `SET SESSION TRANSACTION READ ONLY` en complément (aucun
aller-retour par emprunt, ferme le cas où les droits auraient été élargis entre deux démarrages). `before_acquire` et la
garde lexicale deviennent redondants.

**Recommandation : (c)** — l'utilisateur `SELECT`, le contrôle de ses droits au démarrage, le mode de session en
complément ; le serveur MCP **ne s'active pas** sans la variable. C'est la seule forme qui tienne la décision de Guy
(« prouvée par construction ») ; son coût est une étape manuelle sur le NAS, une fois, que le manuel d'administration
écrit. **Si Guy refuse l'étape manuelle**, (a) — mais alors les mots « par construction » disparaissent des AC et du
manuel, remplacés par la limite : « garantie tant que le code de Kesh n'émet pas ces instructions ».

**Tests sur le chemin de production** (AC de la 32-6), selon la forme : le pool construit par la fonction de
production, `INSERT`/`UPDATE`/`DELETE` en autocommit et en transaction → refus (`1142` en (b)/(c), `1792` en (a)) ;
mutations à tuer : retirer l'`after_connect` ; en (a), retirer le `before_acquire` (séquence B) ; en (b)/(c), donner
`INSERT` à l'utilisateur → le contrôle de démarrage doit rougir.

### 4.4 Les lectures qui écrivent — la séparation à poser en 32-3

| Site | Ce qu'il écrit | Sur la connexion de lecture |
|---|---|---|
| `emit_report_audit` (`routes/reports.rs:1407`, appelé l. 233, 266, 401, 437, 1123) et sa variante d'export | `pool.begin()` + `audit_log::insert_in_tx` | échouerait (1792, ou 1142 avec un utilisateur `SELECT`) — **et l'échec est avalé** par `tracing::warn!` : la trace disparaîtrait **en silence** |
| `api_keys::touch_last_used` (`auth/api_key.rs:185`, dans `validate_pat`, sur `&state.pool`) | `last_used_at` | idem, *best-effort*, avalé — **ne concerne le MCP que si la clé MCP réutilise ce chemin** : le middleware (`middleware/auth.rs`) ne route vers `validate_pat` que le préfixe `kesh_pat_` ; une clé MCP d'un second type (décision de Guy) aura sa propre validation, et la 32-5 dira où elle écrit sa dernière utilisation (sur `Trace`) |
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
double ère, garde `Origin`/`Host` intégrée) : je propose de le réexaminer **dès que Kesh doit servir `2026-07-28`** —
parce qu'un des deux clients abandonnerait l'ère *legacy* (aucun ne l'a fait : tous deux la parlent, § 1.3), ou parce
que Guy voudrait les poignées natives ou MRTR de cette révision — question pour Guy (Q8 ci-dessous). *(Critère
reformulé en relecture P1, C-32-2-11 : « dès qu'un client exige `2026-07-28` » supposait, à tort, qu'aucun ne la parlait
encore.)*

### 5.3 Les traits — indicatifs, pour que le passage au SDK ne change que le transport

Règle : **aucun type de `rmcp` ni de JSON-RPC ne traverse les traits** ; le registre parle des types de Kesh, et
l'adaptateur (le nôtre aujourd'hui, un `impl rmcp::ServerHandler` demain : `list_tools`, `call_tool`, `list_prompts`,
`get_prompt`) traduit. **Aucune ressource MCP n'est servie en 0.16.0** (C-32-2-21) : ni capacité `resources` à
`initialize`, ni `resources/list`, `resources/read`, `resources/templates/list`, `resources/subscribe` — ces méthodes
rendent `-32601` (*Method not found*), comme `completion/complete`, `logging/setLevel` et `tasks/*`, que Kesh ne sert
pas non plus.

```rust
/// Portée d'une clé MCP (C-32-0-7). Aucune portée d'administration ni d'écriture des livres.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Portee { BooksRead, InvoicesRead, ContactsRead, DocumentsRead, PromptsWrite }

/// Ce que reçoit un outil de LECTURE : l'accès résolu, la lecture, la trace — jamais le pool ordinaire.
pub struct ContexteMcp<'a> {
    pub acces: &'a ContexteAcces,   // société, clé, portées, canal = Mcp (32-3)
    pub lecture: &'a Lecture,       // pool en lecture seule (§ 4.3)
    pub trace: &'a Trace,           // journal d'audit, sur le pool ordinaire
}

/// Ce que reçoit l'outil qui PROPOSE un prompt (portée PromptsWrite) — et lui seul : aucune `Lecture`,
/// donc aucune lecture des livres ; `EcriturePrompts` n'écrit qu'une version d'origine `agent`, « à valider » (32-10).
pub struct ContextePrompts<'a> {
    pub acces: &'a ContexteAcces,
    pub prompts: &'a EcriturePrompts,
    pub trace: &'a Trace,
}

/// Ce qu'un outil de lecture PRODUIT. Il ne l'envoie pas : seul le répartiteur fabrique la réponse du transport,
/// et c'est lui qui décide, selon la politique de la clé (Q12), si ces données passent par la validation (Q10).
pub struct Donnees {
    pub resume: String,                       // ce que l'écran de validation montre en tête
    pub contenu: Vec<BlocContenu>,            // texte, champs libres enveloppés « donnée non fiable » (32-9)
    pub structure: Option<serde_json::Value>, // structuredContent, si l'outil en déclare un
}

/// Nom d'un prompt : `[a-z0-9_-]{1,64}`, validé à la construction (Claude Code l'affiche `/serveur:nom`).
pub struct NomPrompt(String);

/// Ce que rend l'outil qui propose un prompt : un accusé fermé. Il ne porte que le nom que l'AGENT a lui-même fourni et
/// un numéro de version : rien qui vienne des livres.
pub enum Accuse { PromptPropose { nom: NomPrompt, version: u32 } } // état toujours « à valider »

/// Erreur d'un outil ou d'un prompt — SANS texte libre (relecture P2, C-32-2-21) : un message d'erreur est un chemin
/// vers le client, et un `String` pourrait y porter une valeur lue (le prototype renvoyait le message brut de MariaDB,
/// `proto-curl.out` l. 32). Le répartiteur traduit chaque variante en un texte FIXE ; le détail d'`Interne` est
/// journalisé côté Kesh, jamais envoyé.
pub enum ErreurOutil {
    ArgumentsInvalides { argument: &'static str, motif: MotifArgument }, // nom tiré du schéma, motif énuméré
    Introuvable,                         // texte fixe : ne distingue pas « n'existe pas » de « autre société »
    Interne,
}
pub enum MotifArgument { Absent, MauvaisType, HorsBornes, Inconnu }

pub trait McpTool: Send + Sync + 'static {
    fn nom(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema_entree(&self) -> serde_json::Value; // JSON Schema strict, additionalProperties: false
    /// SANS défaut : un outil sans portée ne compile pas (AC de la 32-6). Jamais `PromptsWrite`.
    fn portee(&self) -> Portee;
    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        arguments: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<Donnees, ErreurOutil>> + Send + 'a>>;
}

/// L'outil d'écriture de prompt (`proposer_prompt`, 32-10) : portée `PromptsWrite` implicite, autre contexte,
/// autre type de retour — il ne passe pas par la validation avant envoi, parce qu'il n'envoie rien des livres.
pub trait McpToolPrompts: Send + Sync + 'static {
    fn nom(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema_entree(&self) -> serde_json::Value;
    fn executer<'a>(
        &'a self,
        ctx: &'a ContextePrompts<'a>,
        arguments: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<Accuse, ErreurOutil>> + Send + 'a>>;
}

pub struct ArgumentPrompt { pub nom: &'static str, pub description: &'static str, pub requis: bool }
pub struct MessagePrompt { pub role: Role, pub texte: String }

/// Un prompt instruit, il ne lit pas : `rendre` ne reçoit aucune `Lecture`. Son texte vient de la table des prompts
/// (standard, administrateur, ou agent approuvé) et des arguments que le CLIENT envoie ; `MessagePrompt` n'a qu'un
/// texte — aucun contenu `resource` embarqué. Servi seulement à l'état `approuvé` ; un prompt `agent` porte sa marque
/// dans le texte servi (32-10).
/// L'absence de `Lecture` ne tient que par la signature : une implémentation pourrait capturer un pool à sa
/// construction (relecture P2, L-8). D'où un trait SCELLÉ (`mod scelle { pub trait Scelle {} }`), une seule
/// implémentation, `PromptEnBase` (champs : nom, description, arguments, portée, texte — aucun pool), construite par le
/// catalogue. Même règle pour les outils : structures construites par `Registre::standard()` sans pool (le contexte
/// arrive par `executer`), ce que la revue de chaque outil vérifie.
pub trait McpPrompt: Send + Sync + scelle::Scelle {
    fn nom(&self) -> &str;              // les prompts viennent de la base (32-10a) : pas de &'static
    fn description(&self) -> &str;
    fn arguments(&self) -> &[ArgumentPrompt];
    fn portee(&self) -> Portee;         // sans défaut, comme pour les outils
    fn rendre(&self, arguments: &BTreeMap<String, String>) -> Result<Vec<MessagePrompt>, ErreurOutil>;
}

/// Politique de validation — propriété de la CLÉ, lue par le répartiteur, jamais déclarée par un outil (Q12).
pub enum Validation { Requise, Dispensee /* posée par l'administrateur seul ; chaque envoi reste journalisé */ }

/// Le registre est le seul point d'entrée du transport : filtrage par portée, refus indiscernable d'un inconnu.
pub struct Registre {
    outils: Vec<Box<dyn McpTool>>,          // ordre stable
    prompts_ecriture: Vec<Box<dyn McpToolPrompts>>,
    prompts: CataloguePrompts,              // lit la table des prompts (état `approuvé` seul) sur le pool ordinaire,
                                            // rend des `PromptEnBase` ; ajouté en relecture P2 (L-8)
}

/// Données des livres APRÈS la politique de la clé : seul `repartiteur` la construit (constructeur privé au module).
/// Le transport ne sait sérialiser qu'une `Reponse` — aucun chemin ne mène d'une `Donnees` au client sans elle.
pub struct DonneesValidees { donnees: Donnees /* champ privé */ }

/// Tout ce que le transport peut envoyer en `result` (C-32-2-21).
pub enum Reponse {
    Statique(ReponseStatique),        // initialize, tools/list, prompts/list (noms et descriptions), ping
    Outil(DonneesValidees),           // tools/call d'un McpTool
    EnAttente(Poignee),               // forme (c) : « envoi n° X en attente », sans donnée
    Accuse(Accuse),                   // tools/call de proposer_prompt
    Prompt(Vec<MessagePrompt>),       // prompts/get
    Echec(ErreurOutil),               // isError: true, texte fixe par variante
}
impl Registre {
    pub fn lister(&self, portees: &HashSet<Portee>) -> Vec<DescripteurOutil> { /* … */ }
    pub fn trouver(&self, nom: &str, portees: &HashSet<Portee>) -> Option<OutilTrouve<'_>> { /* None = inconnu OU hors portée */ }
}
```

**Où se tient « par construction » pour la validation** (C-32-2-17, étendu en relecture P2 par C-32-2-21) : non plus
dans le type de retour de l'outil — la première rédaction faisait rendre une `PropositionEnvoi` à **tout** outil, y
compris `proposer_prompt`, qui n'envoie rien, et ne laissait aucune place à une clé dispensée —, mais dans le
**répartiteur** et dans le type `Reponse` du transport. La relecture P2 a relevé que la formule précédente (« le seul
chemin des `Donnees` vers le client passe par `Validation` ») ne couvrait que `tools/call` : la garantie doit porter sur
**chaque méthode JSON-RPC servie et chaque message d'erreur**. Inventaire clos, en `2025-11-25` :

| Méthode servie | Ce qui part vers le client | Peut-il porter des données des livres ? |
|---|---|---|
| `initialize`, `ping`, `notifications/*` (202) | capacités, `instructions`, version — textes de Kesh | non : `ReponseStatique`, constantes du binaire |
| `tools/list` | noms, descriptions, schémas des outils | non : `&'static str` des outils |
| `tools/call` d'un `McpTool` | `Donnees` | **oui** → seulement en `DonneesValidees`, construite par le répartiteur après la politique de la clé (Q12) ; en forme (c), `EnAttente` sans donnée ; en forme (a), la requête `elicitation/create` est elle-même construite par le répartiteur depuis les `Donnees` — c'est l'étape de validation, et elle envoie les données **au client** avant l'accord (§ 3.2, motif 1) |
| `tools/call` de `proposer_prompt` | `Accuse` | non : nom fourni par l'agent, numéro de version |
| `prompts/list`, `prompts/get` | noms, descriptions, texte des prompts approuvés, arguments du client | **pas par Kesh** : aucune `Lecture` (trait scellé, `PromptEnBase` sans pool). Reste un cas assumé : un texte de prompt dans lequel un humain a **recopié** des valeurs des livres, ou qu'un agent a proposé avec elles et que l'administrateur a approuvé — l'approbation du texte en tient lieu de validation, ce que le manuel dira (32-10) |
| `notifications/progress` (forme (b)) | `message` de progression | non : texte fixe (« en attente de validation dans Kesh », `proto-curl.out` l. 87) |
| toute erreur JSON-RPC ou `isError: true` | texte d'erreur | non : `ErreurOutil` sans texte libre, traduit en textes fixes ; erreurs du transport (`-32600`, `-32601`, `-32602`) en constantes |
| `resources/*`, `completion/complete`, `logging/setLevel`, `tasks/*`, toute autre | `-32601` | non servies en 0.16.0 |

**Tests de la 32-6 qui tiennent cet inventaire** (AC ajoutés) :
- *inventaire des méthodes* : le répartiteur JSON-RPC associe chaque nom de méthode à une variante d'une énumération
  `MethodeServie` ; un test compare l'ensemble des méthodes acceptées à la liste close ci-dessus et **rougit** si une
  méthode est ajoutée sans y être inscrite ; un second envoie chaque méthode du schéma `2025-11-25` non servie
  (`resources/list`, `resources/read`, `resources/templates/list`, `resources/subscribe`, `completion/complete`,
  `logging/setLevel`, `tasks/get`, `tasks/list`…) et exige `-32601` ;
- *par le type* : `DonneesValidees` n'a pas de constructeur public hors du module `repartiteur`, et le transport ne
  sérialise qu'une `Reponse` — un futur gestionnaire `resources/read` qui voudrait rendre des `Donnees` sans passer par le
  répartiteur **ne compile pas** ; mutation à tuer : rendre le constructeur public (un test de compilation négatif,
  `trybuild`, ou à défaut une garde lexicale sur `DonneesValidees {` hors du module) ;
- *erreurs* : un outil de démonstration dont l'exécution échoue sur une valeur piégée de la fixture (32-9) ne fait
  apparaître cette valeur ni dans `isError`, ni dans une erreur JSON-RPC.

Une clé `Requise` ne peut donc pas être contournée par un outil ; une clé `Dispensee` ne l'est que par
l'administrateur, et chaque envoi est journalisé sur `Trace`. Si Guy retient « toujours » (Q12), `Validation` disparaît
et le répartiteur valide tout. **Le jour où Kesh sert des ressources** (contenu d'une pièce, 0.17.0 au plus tôt), elles
passent par un trait `McpRessource` qui produit des `Donnees` et par le même répartiteur, jamais par un gestionnaire à
part — ou bien le contenu d'une pièce se rend par un **outil** (`McpTool`, bloc de contenu image ou ressource embarquée
dans le résultat), ce qui le place d'office sous la validation.

Le futur boxé (plutôt que `async fn` dans le trait) rend le trait **compatible `dyn`**, ce qu'exige un registre
hétérogène ; `async-trait` n'est pas dans le dépôt et n'est pas nécessaire.

---

## 6. Conséquences sur les stories 32-3 à 32-11 (AC à ajuster)

*Hors du champ de la spike, et déjà portés par le kickoff (`epic-32-serveur-mcp.md`), que la spike ne modifie pas
(L7) : le **filtre d'adresse** de défense en profondeur, sur le pair TCP, désactivable, proxies de confiance déclarés
(32-5 réglages, 32-6 garde, 32-7 écran) ; l'**activation et la désactivation par l'administrateur**, désactivé par
défaut, désactivé → 404 (32-5, 32-6, 32-7) ; les prompts écrits par un agent **« à valider »**, non servis avant
approbation (32-10). Trois AC ci-dessous dépendent des réponses de Guy à Q10, Q11 et Q12 (§ 7).*

- **32-3** — AC ajouté : le service reçoit `Lecture` et `Trace` **typées et distinctes** (§ 4.4) ; la fiche inventorie
  l'ensemble clos des écritures sur les chemins de lecture des deux pilotes (`emit_report_audit` et sa variante
  d'export, `touch_last_used`) ; mutation « la trace part sur `Lecture` » tuée — l'échec y serait **avalé** par le
  `warn!` actuel, donc muet.
- **32-4a / 32-4b** — AC ajouté : aucun service de lecture n'appelle un `get_or_create_*` ni un dépôt qui écrit ;
  inventaire clos des appels.
- **32-5** — AC ajouté au manuel (32-7) : la clé se donne au client en portée `local`/`user`, jamais en `.mcp.json`
  versionné sauf `${VARIABLE}` ; la validation de la clé MCP, distincte de `validate_pat`, écrit sa dernière
  utilisation sur `Trace` (§ 4.4). **Selon Q12** : réglage `validation` par clé (`requise` par défaut, `dispensée`
  posée par l'administrateur seul). **Selon Q11 (b)/(c)** : la variable d'accès du second utilisateur, et le serveur MCP
  non activable sans elle.
- **32-6** — révision **`2025-11-25`** (liens du § 1.1), `2025-06-18` acceptée ; sans session (aucun `Mcp-Session-Id`),
  `GET`/`DELETE` 405, notification 202, lot 400, `ping` ; `MCP-Protocol-Version` non servi → **400 texte** (jamais
  `-32022`), absent admis ; `Origin` présent et étranger → 403, absent admis, origines du réseau local en liste réglable
  et jamais déduites de `Host` (§ 1.3) ; outil inconnu ou hors portée → `-32602` identique ; `readOnlyHint: true` sur
  tout outil ; corps borné ; **inventaire clos des méthodes servies** et `DonneesValidees` à constructeur privé, avec
  leurs tests (§ 5.3) ; `ErreurOutil` sans texte libre. **Ce que le prototype n'a pas éprouvé** (`DELETE`, `ping`, borne du corps, version absente)
  entre au test de contrat. Pool de lecture : **fonction unique** des options (cf. [#669]) ; **selon Q11** : (a)
  `after_connect` + `before_acquire` + garde lexicale, ou (b)/(c) utilisateur `SELECT` + contrôle de ses droits au
  démarrage (+ `after_connect` en (c)) ; tests et mutations du § 4.3. Le module JSON-RPC n'expose aux outils que les
  types du § 5.3.
- **32-7** — le manuel dit la configuration des deux clients (§ 2.4), la limite de 25 000 jetons de Claude Code, que
  le filtre `Origin` ne protège que d'un navigateur ; **selon Q11 (b)/(c)**, la création à la main de l'utilisateur
  `SELECT` sur le NAS (commande, compte qui la passe, variable du `.env`) ; **selon Q10 (b)**, qu'au-delà de 2 min Claude
  Code poursuit sans le résultat.
- **32-8** — **selon Q10**. *(c) poignée* (recommandée) : une table des envois en attente (🗄️ : migration, audit
  d'idempotence, triage P7, place dans la sauvegarde avec la 32-12), un outil de lecture du résultat sous la même clé
  et la même société, une échéance et une purge, l'écran de validation ; l'« état en mémoire » de la fiche du kickoff ne
  vaut plus. *(b) appel tenu ouvert* : attente **240 s** (constante nommée, sous les 300 s de Hermes), réponse SSE,
  premier évènement immédiat, progression toutes les 15 s (`notifications/progress` si `progressToken`, commentaire SSE
  sinon), connexion de lecture **rendue avant l'attente**, état en mémoire. Dans tous les cas : expiration et refus →
  `isError: true`, rien ne part ; ce que reçoit le client est exactement ce qui a été validé ; le répartiteur applique la
  politique de la clé (Q12). **Avant la 32-8, Guy ou la 32-8 elle-même rejoue § 2.4 (d)** pour la forme retenue.
- **32-9a / 32-9b** — bornes de pagination choisies pour rester **sous 10 000 jetons** par réponse (avertissement de
  Claude Code) ; **aucune ressource MCP en 0.16.0** (C-32-2-21) : la spike n'a pas éprouvé de forme pour le
  contenu d'une pièce, si bien que la 32-9b livre les **métadonnées** seulement, comme C-32-0-10 le prévoit à défaut ;
  le contenu glisse en 0.17.0, par un outil ou par un trait de ressource soumis au répartiteur (§ 5.3) — capacité
  `resources` non déclarée.
- **32-10a** — `prompts/list`/`prompts/get` selon `2025-11-25` ; Claude Code les montre en commandes
  `/serveur:prompt (MCP)`, Hermes par `get_prompt` : le nom d'un prompt doit être court et sans espace ; `rendre` sans
  `Lecture`, trait scellé, seule implémentation `PromptEnBase` sans pool (§ 5.3) ; `proposer_prompt` est un `McpToolPrompts` : contexte sans `Lecture`, retour `Accuse`, hors validation
  avant envoi (il n'envoie rien des livres) ; la version créée est d'origine `agent`, « à valider ».
- **32-11** — test de contrat sur la séquence du § 1.2 (initialize, initialized, tools/list, tools/call, outil inconnu,
  Origin, sans clé, sonde `2026-07-28`, lot, `GET`, `DELETE`, `ping`, version absente) ; parcours joué avec **les deux**
  clients et leurs versions ; les observations (a) à (h) du § 2.4 consignées.

## 7. Questions pour Guy

- **Q8 — Le SDK** : `rmcp` est officiel, Tier 1 et double ère ; on garde l'écriture à la main en 0.16.0 et on le
  réexamine le jour où Kesh doit servir `2026-07-28` (un client qui abandonne l'ancienne poignée de main, ou le besoin
  des poignées natives et de MRTR) ? *Recommandation* : oui. *(Reformulée en relecture P1 : les deux clients parlent
  déjà `2026-07-28`, aucun ne l'**exige**.)*
- **Q9 — Hermes** : s'agit-il bien de **Hermes Agent (Nous Research)** ? Si c'est un autre client, les §§ 2.2, 1.3 et 3
  sont à revoir.
- **Q10 — La forme de la validation avant envoi** (§ 3) : (a) élicitation dans le client, (b) appel tenu ouvert au plus
  240 s pendant que tu valides dans Kesh, ou (c) **poignée** — l'outil rend aussitôt « envoi n° X en attente », tu
  valides (tel quel ou modifié) ou refuses dans un écran de Kesh à ton rythme, et l'agent relit le résultat ensuite ?
  *Recommandation* : **(c)**, la seule qu'aucun délai de client ni mandataire ne contraint, qui survit à un
  redémarrage, se teste sans minuteur et prépare l'écran des propositions de la 0.17.0 ; son prix est un tour d'agent
  de plus et une table de plus (story 🗄️).
- **Q11 — La lecture seule** (§ 4.3) : (a) mode de session `READ ONLY` avec contrôles (aucune installation, mais
  garantie seulement tant que le code de Kesh n'émet pas certaines instructions), (b) un **second utilisateur MariaDB
  limité au `SELECT`** (garantie tenue par le serveur, mais créé **à la main** sur le NAS — les scripts d'init ne
  rejouent pas sur une base existante — et une variable d'accès à la base de plus dans le `.env`), ou (c) les deux ?
  *Recommandation* : **(c)** — l'utilisateur `SELECT`, ses droits contrôlés au démarrage, le mode de session en
  complément, le MCP non activable sans lui ; c'est la seule forme qui tienne « prouvée par construction ». Si tu
  refuses l'étape manuelle, (a), et les mots « par construction » disparaissent.
- **Q12 — La validation s'applique-t-elle à chaque appel ?** Ta règle : « montrer les données qui vont être envoyées et
  demander la permission de les envoyer telles quelles ou de les modifier ». Pour l'agent de développement, qui lit
  beaucoup, chaque lecture deviendrait une validation (et, avec la poignée, un aller-retour). Options : **toujours**
  (toute clé, tout outil de lecture) ; **par clé, réglable** par l'administrateur (`requise` par défaut, `dispensée`
  pour une clé qu'il désigne, chaque envoi restant journalisé) ; **seuil** (validation au-delà de N lignes, ou seulement
  pour les champs nominatifs — difficile à définir sans trou). Hors question dans tous les cas : un prompt servi et
  l'accusé de `proposer_prompt` ne portent aucune donnée **lue par Kesh** et ne passent pas par la validation (§ 5.3 ;
  *précisé en relecture P2* : un texte de prompt où un humain a recopié des valeurs reste un cas assumé, couvert par
  l'approbation du texte).
  *Recommandation* : **par clé, `requise` par défaut**, dès la 0.16.0 — elle coûte un réglage au répartiteur, et l'usage
  de l'agent de développement est un objectif de la 0.16.0. *(Elle s'écarte de la recommandation de la Q2 du kickoff —
  « obligatoire en 0.16.0, option après la recette » — parce que la poignée rend chaque validation plus coûteuse ; la Q2
  et la Q12 sont la même question, à trancher une fois.)*

[#669]: https://github.com/guycorbaz/kesh/issues/669
