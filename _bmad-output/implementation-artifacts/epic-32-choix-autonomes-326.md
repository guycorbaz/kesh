# Epic 32 — registre des choix autonomes de la Story 32-6

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-6-endpoint-mcp.md`. Numérotation `C-32-6-k`. Les numéros de ligne
cités sont ceux de `b723e3d9`. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité.

---

### C-32-6-1 — Une fiche, deux parties : 32-6a (le point d'entrée et ses gardes), 32-6b (le journal)

- **Contexte** : le kickoff comptait 3 ou 4 modules (`kesh-api/mcp`, `middleware`/`lib.rs`/`main.rs`, `kesh-db`). Le
  relevé au code en trouve **six distincts** : `mcp` ; `routes` (le registre des routes mutantes n'extrait que
  `post(routes::m::h)`, d'où un gestionnaire mince dans `routes/mcp.rs`, et `lib.rs`) ; `services` (le pool d'une
  poignée n'est lisible que là, `poignees.rs:31` ; gardes de `services_motif.rs`) ; `kesh-db` (fabrique du pool, dépôt
  `list_enabled`) ; `audit_labels` et `kesh-i18n` (libellé de l'action du journal). Huit en somme.
- **Retenu** : 32-6a (4 : `mcp`, `routes`, `services`, `kesh-db`) puis 32-6b (4 : `mcp`, `services`, `audit_labels`,
  `kesh-i18n`). Le journal se détache sans ouvrir d'accès : après 32-6a, le seul outil, `kesh_version`, ne rend aucune
  donnée des livres (test d'inventaire du registre), et une clé `required` n'exécute aucun outil.
- **Écartées** : une story à six modules (règle de découpage préventif) ; séparer le réseau (filtre, `Origin`) du
  protocole (un état intermédiaire servi sans filtre ni `Origin` : à éviter pour la story de sécurité) ; trois parties
  (le protocole seul n'a pas de garde à éprouver).
- **Réversible** : oui (avant développement).

### C-32-6-2 — « Désactivé » est indiscernable d'une route absente : délégation au service statique, non un 404 littéral

- **Contexte** : une route absente rend **405** à un `POST` (`ServeDir`, `tower-http 0.6.11`, `serve_dir/mod.rs:317-327`)
  et la page de l'application à un `GET` (`lib.rs:213-214`) ; les tests l'admettent (`password_recovery_e2e.rs:690-703`).
  Un 404 littéral rendrait `/mcp` reconnaissable.
- **Retenu** : quand `/mcp` n'est pas servi, le gestionnaire **remet la requête au même service statique** que le repli
  (une fonction `service_statique(static_dir)` dans `lib.rs`), sous la même couche CSP ; test d'égalité statut,
  en-têtes, octets contre un chemin absent. Indiscernable par construction de l'appel, non par imitation.
- **Écartées** : un 404 littéral (oracle) ; recopier à la main le 405 de `ServeDir` (dérive au premier changement de
  `tower-http`) ; changer le repli de toute l'application pour rendre 404 à toute méthode non `GET` (hors périmètre,
  soumis à Guy).
- **Réversible** : oui.

### C-32-6-3 — La règle « servi » par société : l'ensemble S, avant et après l'authentification ; fail closed

- **Contexte** : `mcp_server_settings` a une ligne par société ; « désactivé → indiscernable » se décide **avant**
  l'authentification (report (2) de la 32-5) ; aucune fonction ne liste les sociétés activées.
- **Retenu** : dépôt `mcp_server_settings::list_enabled` (lecture seule) lu sur la `Lecture` du MCP par
  `services::serveur_mcp::reglages_actifs` ; S = réglages actifs dont le filtre admet le client ; S vide → route
  absente ; après l'authentification, la société de la clé doit être dans S, sinon route absente. Une erreur de la base
  pendant la décision → route absente et `tracing::error!`. Aucun cache : un changement de réglage vaut à la requête
  suivante.
- **Écartées** : « servi si une société au moins est activée » sans contrôle de la société de la clé (une clé d'une
  société désactivée lirait) ; un cache des réglages (un serveur désactivé resterait ouvert jusqu'à l'expiration) ; une
  erreur 500 (révélerait l'existence de réglages).
- **Réversible** : oui.

### C-32-6-4 — Un pair refusé par le filtre voit une route absente (pas un 403)

- **Contexte** : le filtre est de défense en profondeur ; le kickoff dit « pair public refusé » sans code.
- **Retenu** : refus du filtre = réponse de route absente : depuis une adresse non admise, le serveur MCP n'existe pas.
  Le refus est au journal du serveur (`tracing`), jamais en base.
- **Écartées** : 403 (confirme l'existence du serveur à un pair hors réseau) ; fermer la connexion (inhabituel, plus
  difficile à tester).
- **Réversible** : oui.

### C-32-6-5 — `X-Forwarded-For` : lu des seuls mandataires déclarés, de droite à gauche ; `Forwarded` et `X-Real-IP` ignorés

- **Contexte** : décision de Guy (« `X-Forwarded-For` n'est lu que d'un proxy de confiance déclaré ») ; Traefik devant
  une partie des conteneurs ; un client peut forger la partie gauche de l'en-tête, que le mandataire complète à droite.
- **Retenu** : pair hors des mandataires → l'en-tête est ignoré ; pair mandataire → lecture de toutes les lignes de
  l'en-tête, de droite à gauche, en sautant les mandataires de confiance, le premier autre étant le client ; en-tête
  absent → client = pair ; toutes les entrées de confiance → la plus à gauche ; entrée qui n'est pas une adresse nue →
  refus. Adresses normalisées par `IpAddr::to_canonical` (IPv4 mappée).
- **Écartées** : prendre l'entrée la plus à gauche (forgeable) ; accepter `ip:port` ou `[v6]` (formes que Traefik
  n'émet pas, angle d'analyse de plus) ; lire `Forwarded` (RFC 7239) ou `X-Real-IP` (deux sources de vérité).
- **Réversible** : oui.

### C-32-6-6 — Une entrée stockée invalide est ignorée, jamais élargissante ; filtre activé et liste vide = personne

- **Contexte** : la 32-5c valide la forme à l'écriture, mais une ligne peut venir d'un SQL à la main ou d'une version
  future (même raisonnement que C-32-5-37).
- **Retenu** : l'entrée qui ne s'analyse pas est sautée et journalisée (`tracing::warn!`, colonne et société, sans la
  valeur) ; une liste vide sous filtre activé n'admet personne.
- **Écartées** : faire échouer tout le filtre (le serveur deviendrait inaccessible pour une entrée) ; traiter l'erreur
  comme « tout admettre » ; une liste vide qui signifierait « privé » (règle invisible, déjà écartée par C-32-5-10).
- **Réversible** : oui.

### C-32-6-7 — `Origin` : absent admis ; présent, admis s'il est propre à Kesh ou listé ; 403 sinon ; jamais déduit de `Host`

- **Contexte** : spike § 1.1 et § 1.3 ; origines du réseau local en liste réglable (`allowed_origins`, 32-5a).
- **Retenu** : origines propres = `KESH_PUBLIC_BASE_URL` réduite à son origine, `http://127.0.0.1:<port>`,
  `http://localhost:<port>`, `http://[::1]:<port>` ; comparaison normalisée (minuscules, port par défaut retiré, aucun
  chemin) ; `null` refusée ; contrôle avant l'authentification sur les origines de S, rejoué après sur celles de la
  société de la clé ; 403 à corps JSON-RPC sans `id`.
- **Écartées** : admettre l'origine égale à `Host` (concorde sous *DNS rebinding*, spike § 1.3) ; refuser un `Origin`
  absent (casserait les clients en ligne de commande) ; un réglage `.env` des origines (C-31-0-4).
- **Réversible** : oui.

### C-32-6-8 — L'ordre des contrôles d'une requête

- **Contexte** : chaque contrôle peut révéler quelque chose ou coûter une lecture.
- **Retenu** : pair → S (route absente) → méthode (405) → `Origin` (403) → `Content-Type` (415) →
  `MCP-Protocol-Version` (400) → clé (401/500) → société de la clé (route absente) → `Origin` de la société (403) →
  corps borné et JSON-RPC → répartiteur. Le corps n'est lu qu'après l'authentification.
- **Écartées** : authentifier avant la règle « servi » (une clé serait vérifiée, et sa dernière utilisation écrite, pour
  un serveur désactivé ou un pair refusé) ; lire le corps avant l'authentification (analyse de données anonymes).
- **Réversible** : oui.

### C-32-6-9 — Le pool du MCP dérivé du pool de production dans `build_router`, par une fabrique de `kesh-db` ; aucun champ dans `AppState`

- **Contexte** : Q11 (mode de session `READ ONLY`, aucune variable) ; prémisse 7 du kickoff (le pool doit naître sur
  le chemin de production) ; six sites de test et `middleware/auth.rs:267` construisent `AppState` par littéral (fiche
  32-3, l. 782-783) ; `Pool::connect_options()` et `connect_lazy_with` existent (`sqlx-core 0.8.6`).
- **Retenu** : `kesh_db::pool::pool_lecture_seule(&state.pool, n)` — mêmes options de connexion et délai
  d'acquisition, `after_connect` `SET SESSION TRANSACTION READ ONLY`, `before_acquire` qui jette une connexion dont
  `@@session.tx_read_only` ≠ 1, construction paresseuse — appelée dans `build_router_avec_registre`, que la production
  et les tests empruntent. **Aucun autre réglage de session** : le pool de production n'en a pas (#669), et une lecture
  doit rendre la même chose par l'API et par le MCP. `tests/common/lecture_seule.rs` emploie la même fabrique.
- **Écartées** : un champ `AppState` (sept littéraux à modifier, et des tests qui y mettraient un pool ordinaire) ; un
  pool construit dans `main.rs` (les tests ne l'emprunteraient pas) ; régler #669 ici (changement de comportement de la
  production sur des livres réels, hors périmètre) ; un second utilisateur MariaDB (refusé par Guy, Q11).
- **Réversible** : oui.

### C-32-6-10 — Trois connexions pour le pool du MCP

- **Contexte** : le pool de production en a cinq (`main.rs:75`) ; un agent appelle surtout en série.
- **Retenu** : `MCP_POOL_CONNEXIONS = 3` (constante nommée) : borne la charge que le MCP impose à MariaDB.
- **Écartées** : cinq (doublerait les connexions) ; une (un appel lent bloquerait tout autre appel, y compris la règle
  « servi »).
- **Réversible** : oui.

### C-32-6-11 — La garde lexicale des formes qui défont le mode de session

- **Contexte** : spike § 4.1 (quatre formes sans privilège) ; état de départ vierge (`grep -rniE "read
  write|tx_read_only|transaction_read" crates` → rien).
- **Retenu** : motifs `(?i)read\s+write`, `(?i)tx_read_only\s*=`, `(?i)transaction_read_only` sur les `.rs` et `.sql`
  de `crates/`, une seule exception écrite (le test de la séquence B, qui exécute volontairement une forme interdite) ;
  la lecture `@@session.tx_read_only` reste permise.
- **Écartées** : interdire `tx_read_only` sans `=` (empêcherait `before_acquire`) ; aucune garde (Q11 rend la garantie
  applicative : il faut l'outiller).
- **Réversible** : oui.

### C-32-6-12 — `kesh_version` rend la version et l'état de la session, aucune donnée des livres

- **Contexte** : le kickoff veut un outil de démonstration `books:read` ; le câblage du pool doit être observable par
  l'endpoint (sinon la mutation « `Lecture` du MCP sur le pool ordinaire » survivrait).
- **Retenu** : `{"version", "lectureSeule"}`, `lectureSeule` lu par `SELECT @@session.tx_read_only` dans un service, sur
  la connexion de la `Lecture`. Les tests E1 et E5 se font par des **outils de test** (registre injectable, C-32-6-13).
- **Écartées** : rendre le nom de la société (une donnée de la société, envoyée sans journal entre 32-6a et 32-6b) ;
  un outil de démonstration qui lit les livres en production.
- **Réversible** : oui.

### C-32-6-13 — Un registre injectable pour les tests : `build_router_avec_registre`, `#[doc(hidden)]`

- **Contexte** : les tests d'E1, d'E5 et d'absence d'exécution exigent des outils que la production n'a pas ; un outil
  hors de `services` ne peut pas toucher un pool (fait 5 de la fiche).
- **Retenu** : `build_router` appelle `build_router_avec_registre(…, Registre::standard())` ; `Registre::vide().avec(…)`
  public ; gardes lexicales : `main.rs` n'appelle que `build_router`, `Registre::standard()` n'est appelé qu'en
  `lib.rs`.
- **Écartées** : des outils de test compilés dans le binaire derrière `KESH_TEST_MODE` (code de test en production) ;
  `#[cfg(test)]` (invisible des tests d'intégration).
- **Réversible** : oui.

### C-32-6-14 — La politique de validation au répartiteur : `required` refusé avant exécution, `exempt` servi

- **Contexte** : Q12 ; spike § 6.2, point 6 (« refus, par défaut » tant que la 32-8 n'est pas livrée) ; Q10 (c).
- **Retenu** : le répartiteur seul lit `CleMcpAuthentifiee::validation` ; `Required` → l'outil n'est pas exécuté,
  `isError: true` à texte fixe ; `Exempt` → exécution et envoi (journal en 32-6b). La 32-8 change ce seul bras.
- **Écartées** : exécuter puis refuser (une lecture faite pour rien, et des données en mémoire) ; une exemption par outil
  (Q12 : la politique est celle de la clé) ; une erreur JSON-RPC (le refus n'est pas une faute de protocole).
- **Réversible** : oui (c'est le point de branchement de la 32-8).

### C-32-6-15 — `DonneesEnvoyables`, à champ privé, défini dans `politique.rs`

- **Contexte** : spike § 5.3.3 (« l'énumération de réponse est ouverte ») ; E6, mutation 2.
- **Retenu** : le seul type qui porte des données d'outil vers le transport est construit par `politique::appliquer`
  seul ; les réponses statiques ne prennent que des `&'static str` et les descripteurs du registre.
- **Écartées** : un `enum Reponse` public à variantes publiques (spike : ne ferme aucun chemin) ; une garde lexicale
  seule (le compilateur est plus fort, la garde nomme le rouge).
- **Réversible** : oui.

### C-32-6-16 — `schema_entree()` rend un `&'static str`

- **Contexte** : spike § 5.3.3 L-5 : un `serde_json::Value` non `'static` est le seul canal par lequel un outil qui
  capturerait un pool logerait une donnée dans `tools/list`.
- **Retenu** : JSON texte du binaire, analysé et contrôlé par le test `registre_coherent` (objet,
  `additionalProperties: false`).
- **Écartées** : `Value` contrôlé à l'exécution.
- **Réversible** : oui.

### C-32-6-17 — Erreurs : énumération sans texte libre ; arguments invalides en erreur d'exécution

- **Contexte** : E5 ; le message de `serde` recopie les noms et valeurs fournis ; spike § 1.3 (erreur d'exécution =
  `isError: true`).
- **Retenu** : `ErreurOutil { ArgumentsInvalides { argument: &'static str, motif }, Introuvable, Interne }`, textes
  fixes en constantes ; arguments invalides → `isError: true` (forme à recontrôler contre la page `server/tools` de
  `2025-11-25` à l'étape 0) ; délai dépassé → `isError: true`.
- **Écartées** : renvoyer le message de `serde` ou de MariaDB (fuite du prototype) ; `-32602` pour un argument invalide
  sans vérification de la révision.
- **Réversible** : oui.

### C-32-6-18 — Le transport : `POST` seul, JSON seul, corps borné, `Content-Type` exigé, `Accept` non contrôlé

- **Contexte** : spike § 1.1 et § 1.3 ; une requête « simple » d'un navigateur (`text/plain`) part sans
  pré-vérification CORS.
- **Retenu** : 405 `Allow: POST` hors `POST` ; 415 si `Content-Type` n'est pas `application/json` (ferme la requête
  simple d'un navigateur) ; 1 Mio ; lot, réponse du client, JSON invalide → 400 ; notification → 202 sans traitement ;
  `Cache-Control: no-store` ; `Accept` non contrôlé (les clients l'envoient, un refus n'apporterait rien) ; aucun
  `Mcp-Session-Id`.
- **Écartées** : exiger `Accept` (casserait un client tolérant sans gain) ; accepter un lot (supprimé depuis
  `2025-06-18`).
- **Réversible** : oui.

### C-32-6-19 — Un délai de 30 s par appel d'outil

- **Contexte** : Claude Code coupe à 60 s sans premier octet (spike § 2.1) ; un outil bloqué tiendrait une connexion du
  pool.
- **Retenu** : `DELAI_OUTIL = 30 s`, `tokio::time::timeout`, `isError: true` à texte fixe.
- **Écartées** : aucun délai ; 60 s (au bord du minuteur du client).
- **Réversible** : oui.

### C-32-6-20 — `/mcp` monté dans `lib.rs` par `post(routes::mcp::recevoir).fallback(routes::mcp::autre_methode)`

- **Contexte** : le registre des routes mutantes n'extrait que `post(routes::m::h)` et fait rougir toute autre forme
  (`audit_route_registry.rs:540-567`) ; un routeur monté depuis `crate::mcp` y échapperait **en silence** — ou, s'il
  était lu, la ferait rougir ; la couche CSP ne couvre que ce qui précède `.layer` (`lib.rs:1225-1227`).
- **Retenu** : gestionnaire mince dans `routes/mcp.rs`, route sur `main_router` avant le repli ; entrée
  `("post", "mcp::recevoir", Traced, SansEcritureAuJournal)` ; `routes::mcp::autre_methode` inscrit aux appels
  non-handlers.
- **Écartées** : `any(…)` (forme non lue par l'extracteur) ; un `nest`/`merge` d'un routeur MCP après `build_router`
  (hors CSP : un `GET /mcp` désactivé se distinguerait d'une route absente par l'en-tête CSP).
- **Réversible** : oui.

### C-32-6-21 — Le journal : `mcp.tool_called`, au nom du créateur de la clé, la clé au détail ; « inconnu » couvre hors portée

- **Contexte** : `actor_type ENUM('user','api_key')` (`20260605000002_audit_log_actor.sql:23`), décodé strictement
  (`entities/audit_log.rs:53-62`) ; un `MODIFY COLUMN` qui ajouterait `mcp_key` serait lu en erreur par un binaire
  antérieur (P1, P3) ; question laissée par la 32-3 sur l'acteur d'une clé MCP ; #431.
- **Retenu** : `NewAuditLogEntry::user(créateur, "mcp.tool_called", "mcp_key", id_clé, détail)` ; détail fermé
  `{cleMcpId, outil, issue, dureeMs, validation}` ; `outil` = nom du registre ou `"(inconnu)"`, jamais le nom fourni
  par le client ; `issue` ne distingue pas inconnu de hors portée.
- **Écartées** : un type d'acteur neuf (migration breaking) ; `actor_type = 'api_key'` avec l'id de la clé MCP dans
  `actor_api_key_id` (pointerait une clé d'API inexistante ou une autre) ; distinguer « hors portée » au journal
  (diagnostic de plus, mais une ligne de plus à protéger ; la portée se lit sur la clé).
- **Réversible** : non une fois publié (code d'action stocké) ; oui avant.

### C-32-6-22 — Les textes envoyés au client sont des constantes françaises, non des messages traduits

- **Contexte** : les messages de l'API passent par `t(…)` (garde G9) ; les textes MCP sont lus par un agent, non
  affichés par l'interface.
- **Retenu** : `mcp/textes.rs`, constantes en français ; aucun ne contient de valeur.
- **Écartées** : quatre locales pour des textes de protocole (la locale du client n'est pas connue au protocole) ;
  l'anglais (le reste du produit parle français).
- **Réversible** : oui.

### C-32-6-23 — Les prompts en 32-6 : le socle, sans prompt

- **Contexte** : le kickoff confie `McpPrompt` à la 32-6, les prompts en base à la 32-10a ; spike § 5.3.3 L-5 (sceau).
- **Retenu** : capacité `prompts` déclarée, `prompts/list` vide, `prompts/get` → `-32602` ; trait scellé sans
  implémentation, que la 32-10a amende en connaissance de cause.
- **Écartées** : renvoyer `prompts/*` en `-32601` jusqu'à la 32-10a (un client garderait en cache l'absence de la
  capacité) ; concevoir `PromptEnBase` ici (anticipe la 32-10a).
- **Réversible** : oui.

### C-32-6-24 — Ce qui n'est pas journalisé en base

- **Contexte** : Guy veut un journal d'audit des appels ; une écriture en base par requête anonyme ouvrirait un
  remplissage de la table.
- **Retenu** : seulement `tools/call` (32-10a ajoutera `prompts/get`) ; refus avant authentification, métadonnées et
  notifications au journal du serveur (`tracing`) seulement.
- **Écartées** : tout journaliser (bruit, et écriture par requête anonyme) ; journaliser `tools/list` (aucune donnée).
- **Réversible** : oui.

### C-32-6-25 — Le cycle de vie n'est pas imposé

- **Contexte** : serveur sans session (spike § 1.3) ; aucune mémoire entre deux requêtes.
- **Retenu** : `tools/list` ou `tools/call` sans `initialize` préalable est servi.
- **Écartées** : exiger `initialize` (demanderait un état de session).
- **Réversible** : oui.

### C-32-6-26 — `tests/common/lecture_seule.rs` emploie la fabrique de production

- **Contexte** : le helper de test refait à la main le pool en lecture seule (`lecture_seule.rs:17-32`), sans
  `before_acquire`.
- **Retenu** : il appelle `kesh_db::pool::pool_lecture_seule` : les tests des services éprouvent la fabrique de
  production (DRY).
- **Écartées** : garder deux fabriques (dérive).
- **Réversible** : oui.

### C-32-6-27 — Le 401 de `/mcp` : corps fixe, sans `WWW-Authenticate`

- **Contexte** : spike § 2.4 (f) : un 401 peut déclencher une découverte OAuth chez le client ; Kesh n'a pas d'OAuth.
- **Retenu** : 401, corps JSON-RPC sans `id`, texte fixe, aucun en-tête `WWW-Authenticate` ; identique pour toutes les
  causes.
- **Écartées** : `WWW-Authenticate: Bearer resource_metadata=…` (inviterait une découverte qui n'aboutit pas) ; le
  corps d'`AppError::Unauthenticated` de l'API (son message varie selon la cause).
- **Réversible** : oui.

### C-32-6-28 — Aucune migration en 32-6

- **Contexte** : le kickoff marquait la 32-6 « 🗄️ (probable) » ; la 32-5a a posé les réglages, origines comprises
  (C-32-5-9) ; le journal tient dans `audit_log` (C-32-6-21).
- **Retenu** : aucune migration ; la 32-6a reste 🗄️ par son dépôt et sa fabrique de pool (gate complet en boucle).
- **Écartées** : une colonne d'acteur MCP (C-32-6-21) ; une table de journal propre (doublon d'`audit_log`).
- **Réversible** : oui.

### C-32-6-29 — Le jeton ne se lit que dans `Authorization: Bearer` ; le cookie de session est ignoré

- **Contexte** : `require_auth` lit le cookie avant l'en-tête (`middleware/auth.rs:83-100`) ; un navigateur joint le
  cookie de lui-même.
- **Retenu** : le transport ne lit pas les cookies ; une session ouverte dans le navigateur ne donne aucun accès à
  `/mcp`.
- **Écartées** : lire le cookie « comme l'API » (une page piégée ouverte par l'utilisateur connecté porterait sa
  session).
- **Réversible** : oui.

### C-32-6-30 — Le journal est fail closed

- **Contexte** : Q12 : « chaque envoi reste journalisé » ; les traces des rapports sont *best-effort*
  (`tracing::warn!`, spike § 4.4).
- **Retenu** : l'entrée s'écrit avant l'envoi de la réponse ; si elle échoue, la réponse devient une erreur à texte
  fixe, sans donnée.
- **Écartées** : *best-effort* (un envoi sans trace, muet) ; écrire après l'envoi (la durée serait exacte, l'envoi non
  garanti tracé).
- **Réversible** : oui.
