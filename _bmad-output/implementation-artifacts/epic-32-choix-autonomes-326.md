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
- *(Annoté le 2026-10-25, validation P2, F5 = R6 : depuis la validation P1, la 32-6b compte **5** modules — `routes` y entre par `audit_route_registry.rs` (AC-b6) — et la somme est **neuf**, non huit ; les nombres ci-dessus sont ceux de la création.)*

### C-32-6-2 — « Désactivé » est indiscernable d'une route absente : délégation au service statique, non un 404 littéral

- **Contexte** : une route absente rend **405** à un `POST` (`ServeDir`, `tower-http 0.6.11`, `serve_dir/mod.rs:317-327`)
  et la page de l'application à un `GET` (`lib.rs:213-214`) ; les tests l'admettent (`password_recovery_e2e.rs:690-703`).
  Un 404 littéral rendrait `/mcp` reconnaissable.
- **Retenu** : quand `/mcp` n'est pas servi, le gestionnaire **remet la requête au même service statique** que le repli
  (une fonction `service_statique(static_dir)` dans `lib.rs`), sous la même couche CSP ; test d'égalité statut,
  en-têtes, octets contre un chemin absent. Indiscernable par **délégation au même service**, non par imitation
  *(« par construction », écrit d'abord, retiré en validation P1, R14 : l'oracle temporel reste ouvert)*. La
  `Request` est remise **entière, corps non lu** (validation P1, F2).
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
  suivante. *(Révisé en validation P1, F11)* : après l'authentification, une clé dont la société n'est pas dans S
  reçoit le **401 fixe** d'une clé inconnue, non la réponse de route absente — sans quoi `/mcp` dirait à qui détient une
  clé volée qu'elle est valide. Une ligne de réglages illisible est écartée seule (F16).
- **Écartées** : « servi si une société au moins est activée » sans contrôle de la société de la clé (une clé d'une
  société désactivée lirait) ; un cache des réglages (un serveur désactivé resterait ouvert jusqu'à l'expiration) ; une
  erreur 500 (révélerait l'existence de réglages).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P2, F5 = R6 : la phrase « sinon route absente » du *Retenu* est **remplacée** par la révision P1 qui la suit — une clé dont la société n'est pas dans S reçoit le 401 fixe ; c'est la seule règle en vigueur.)*

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
- *(Annoté le 2026-10-25, validation P3, R7 : l'entrée invalide est journalisée **à chaque requête en `tracing::debug!`**, sans état partagé, depuis la validation P2 (R8 = F10) — non plus en `warn!`.)*

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
- *(Annoté le 2026-10-25, validation P2, F5 : l'étape « société de la clé (route absente) » rend le **401 fixe** depuis la révision P1 de C-32-6-3 ; deux étapes s'ajoutent : `politique::autoriser` avant `executer` (C-32-6-36) et `reponse::rendre` comme seule sortie (C-32-6-37). L'ordre en vigueur est celui des Dev Notes de la fiche.)*

### C-32-6-9 — Le pool du MCP dérivé du pool de production dans `build_router`, par une fabrique de `kesh-db` ; aucun champ dans `AppState`

- **Contexte** : Q11 (mode de session `READ ONLY`, aucune variable) ; prémisse 7 du kickoff (le pool doit naître sur
  le chemin de production) ; six sites de test et `middleware/auth.rs:267` construisent `AppState` par littéral (fiche
  32-3, l. 782-783) ; `Pool::connect_options()` et `connect_lazy_with` existent (`sqlx-core 0.8.6`).
- **Retenu** *(le canal qui porte ce pool au gestionnaire : C-32-6-31)* : `kesh_db::pool::pool_lecture_seule(&state.pool, n)` — mêmes options de connexion et délai
  d'acquisition, `after_connect` `SET SESSION TRANSACTION READ ONLY`, `before_acquire` qui jette une connexion dont
  `@@session.tx_read_only` ≠ 1, construction paresseuse — appelée dans `build_router_avec_registre`, que la production
  et les tests empruntent. **Aucun autre réglage de session** : le pool de production n'en a pas (#669), et une lecture
  doit rendre la même chose par l'API et par le MCP. `tests/common/lecture_seule.rs` emploie la même fabrique.
- **Écartées** : un champ `AppState` (sept littéraux à modifier, et des tests qui y mettraient un pool ordinaire ; *reconsidéré
  en validation P1 sur rappel de l'orchestrateur — Q11 n'interdit pas un champ — et écarté pour ces raisons, C-32-6-31*) ; un
  pool construit dans `main.rs` (les tests ne l'emprunteraient pas) ; régler #669 ici (changement de comportement de la
  production sur des livres réels, hors périmètre) ; un second utilisateur MariaDB (refusé par Guy, Q11).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P2, R4 : le `before_acquire` est **remplacé** par un `after_release` qui ferme toute connexion à sa restitution (C-32-6-35) ; et le canal (C-32-6-31) ne porte plus `AppState`.)*

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
- *(Annoté le 2026-10-25, validation P3, R1 = F1 : le motif `\bread\W+write\b` de l'annotation P2 rendait **53 lignes** (le `scope` `read-write` des clés d'API, `use std::io::{Read, Write}`) : il est retiré. La fiche prescrit désormais la **propriété** et les **formes à attraper**, avec des témoins rouges et verts, et la règle commune des gardes lexicales ; l'exemple mesuré est `(?i)\bread\s+write\b` après remplacement des commentaires de bloc par une espace — 0 ligne au relevé.)*
- *(Annoté le 2026-10-25, validation P2, R9 = F8 : le motif `tx_read_only\s*=` est **remplacé** par l'interdiction du nom entier `tx_read_only` (comme la spike, § 4.3 (a)), exceptions écrites ; les commentaires de bloc sont retirés de la ligne avant l'analyse, et `\bread\W+write\b` remplace `read\s+write`.)*

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
- *(Annoté le 2026-10-25, validation P2, F2 : le contrôle se fait par une fonction nommée, `politique::autoriser`, **avant** `executer` (C-32-6-36) ; `appliquer` ne fait que construire la réponse. La 32-8 change les deux fonctions, signatures comprises, et leurs deux sites d'appel — non « ce seul bras ».)*

### C-32-6-15 — `DonneesEnvoyables`, à champ privé, défini dans `politique.rs`

- **Contexte** : spike § 5.3.3 (« l'énumération de réponse est ouverte ») ; E6, mutation 2.
- **Retenu** : le seul type qui porte des données d'outil vers le transport est construit par `politique::appliquer`
  seul ; les réponses statiques ne prennent que des `&'static str` et les descripteurs du registre. *(Révisé en
  validation P1, F6)* : `Donnees` vit **aussi** dans `politique.rs`, champs privés, constructeur public, aucun
  accesseur public ; ni `Serialize`, ni `Default`, ni `Debug` sur l'un ou l'autre — sans quoi le répartiteur pourrait
  sérialiser `Donnees` sans passer par `appliquer`, ou en fabriquer un par `Default::default()`.
- **Écartées** : un `enum Reponse` public à variantes publiques (spike : ne ferme aucun chemin) ; une garde lexicale
  seule (le compilateur est plus fort, la garde nomme le rouge).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P2, F2 : `appliquer` prend un jeton `Autorisation`, que seul `autoriser` fabrique (C-32-6-36).)*

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
- **Retenu** : gestionnaire mince dans `routes/mcp.rs`, route sur un sous-routeur fusionné dans `main_router` avant le
  repli (C-32-6-31) ; entrées `("post", "mcp::recevoir", Exempt("issue #723 — …"), SansEcritureAuJournal)` en 32-6a,
  `Traced` en 32-6b, et `("fallback", "mcp::autre_methode", NoMatter(…), SansEcritureAuJournal)` ; le détecteur du
  registre est étendu à `fallback(` (C-32-6-32). *(Révisé en validation P1, R2 = F8 et R8 = F7 : « inscrit aux appels
  non-handlers » était impossible — `appels_non_handlers` ne voit pas ce site — et `Traced` faux avant la 32-6b.)*
- **Écartées** : `any(…)` (forme non lue par l'extracteur) ; un `nest`/`merge` d'un routeur MCP après `build_router`
  (hors CSP : un `GET /mcp` désactivé se distinguerait d'une route absente par l'en-tête CSP).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P4, F1 : le montage `post(routes::mcp::recevoir).fallback(routes::mcp::autre_methode)` est **remplacé** par `route_service("/mcp", routes::mcp::recevoir.with_state(etat))` (C-32-6-40) : un `MethodRouter` ajoute `Allow` et `content-length` à ses réponses, ce qui rendait `/mcp` désactivé discernable. `autre_methode` disparaît ; l'entrée du registre des routes devient `("route_service", "mcp::recevoir", …)`.)*

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

### C-32-6-31 — Le canal : un sous-routeur à état propre, `EtatMcp` (validation P1, F3 = R6)

- **Contexte** : la forme `post(routes::mcp::recevoir)` exclut une fermeture (registre des routes) ; le gestionnaire a
  besoin de la `Lecture` du MCP, de la `Trace`, du registre, du service statique et du délai, qu'`AppState` ne porte
  pas ; une `Extension` manquante rendrait un 500, donc un `/mcp` reconnaissable. L'orchestrateur rappelle que Q11
  interdit une variable `.env` et un second compte MariaDB, **non** un champ d'`AppState`.
- **Retenu** : `Router::<EtatMcp>::new().route("/mcp", …).with_state(EtatMcp { … })`, construit dans
  `build_router_avec_registre`, fusionné dans `main_router` avant le repli et la couche CSP. L'état est **obligatoire à la
  compilation** ; `EtatMcp` porte `app: AppState`, `lecture`, `trace`, `registre`, `statique`, `delai_outil`.
- **Écartées** : un champ d'`AppState` — `AppState` ignore `static_dir`, et les sept constructions par littéral devraient
  fournir un pool que rien ne garantirait en lecture seule (repli écrit si la fusion d'Axum ne se prêtait pas au
  sous-routeur) ; une `Extension` (500 si absente) ; une fermeture (illisible par le registre des routes).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P4, F1 et F2 = R1 : il n'y a plus de sous-routeur — l'état est porté par le service, `routes::mcp::recevoir.with_state(etat)` (C-32-6-40) ; la lecture des outils est un type propre, `LectureOutils` (C-32-6-41).)*
- *(Annoté le 2026-10-25, validation P3, F3 = R8, R9 : `EtatMcp` ne porte plus `config: Arc<Config>` (qui porte `database_url` et `admin_password`) mais `origines_propres`, calculées une fois ; il gagne `garde: Lecture` — la `Lecture` ordinaire, pour la règle « servi » et l'authentification (C-32-6-38).)*
- *(Annoté le 2026-10-25, validation P2, F3, R7, R12 = F11 : `EtatMcp` ne porte **plus** `app: AppState` — son champ public `pool` (`lib.rs:56`) mettait le pool ordinaire à portée de tout `src/mcp/` — mais `config: Arc<Config>` ; le délai vit dans le registre seul ; garde `mcp_sans_pool` sur tout `src/mcp/`. Les constructions par littéral sont **huit** (la production et sept de test), non sept.)*

### C-32-6-32 — Le détecteur du registre des routes lit `fallback(routes::…)` (validation P1, R2 = F8)

- **Contexte** : `extract_counted` ne lit que `post`, `put`, `delete`, `patch` (`audit_route_registry.rs:496-517`) ;
  `appels_non_handlers` écarte tout chemin qui contient un blanc avant la première `(` (`:681-684`) : un
  `.fallback(routes::m::h)` échappe aux deux.
- **Retenu** : `fallback` devient un « verbe » d'`extract_counted`, **seulement** si l'argument commence par `routes::`
  (le `ServeDir::new(…).fallback(ServeFile::new(…))` du repli n'est pas une route) ; `appels_non_handlers` compte
  `fallback(` parmi les constructeurs de gestionnaire ; témoins sur source factice.
- **Écartées** : retirer l'instruction et laisser `autre_methode` hors registre (un gestionnaire accroché par
  `.fallback(` pourrait demain muter sans que rien le signale) ; monter `get`/`put`/`delete`/`patch` séparément (OPTIONS
  et les méthodes non nommées tomberaient au 405 d'Axum, distinct de celui de `ServeDir`).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P4, F1 : l'extension du détecteur à `fallback(` **disparaît** avec `autre_methode` ; le détecteur lit désormais `route_service(` — argument après la première virgule, rogné, jusqu'à `.with_state` —, un argument non `routes::` allant à l'inventaire des non-résolus, et `extract_counted` s'applique à `lib.rs` lignes de commentaire retirées.)*
- *(Annoté le 2026-10-25, validation P2, R3 = F4 : **tout** `fallback(` est extrait ; un argument non `routes::` va à l'inventaire des non-résolus, moins une liste écrite des sites `ServeFile::new(` (une occurrence, `service_statique`) — le filtre « seulement si `routes::` » rendait muet un `.fallback(h)` écrit après un `use`.)*

### C-32-6-33 — Une seule traduction `AppError` → réponse MCP, exhaustive ; aucun `IntoResponse` d'`AppError` (validation P1, F5, R15)

- **Contexte** : `impl IntoResponse for AppError` (`errors.rs:1520`) rend le corps de l'API, et `Validation(msg)`
  recopie `msg` (`:1538-1540`) ; seule la traduction de `Forbidden` était écrite.
- **Retenu** : `mcp::textes::depuis_app_error`, `match` exhaustif sans `_` : `Forbidden` → outil inconnu, `NotFound` →
  `Introuvable`, demandes invalides → `ArgumentsInvalides { argument: "(demande)", … }` sans le message, le reste →
  `Interne` ; l'authentification a ses 401 et 500 fixes ; les gestionnaires rendent `Response` ; garde
  `mcp_ne_rend_jamais_une_app_error`.
- **Écartées** : laisser `?` remonter une `AppError` au framework (corps de l'API, message recopié) ; un `_ =>
  Interne` (une variante neuve passerait sans examen).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P2, R1 = F1 et R2 : le `match` exhaustif sans `_` est **remplacé** par une liste blanche courte — `Forbidden`, `Database(DbError::NotFound)`, `Validation(_)` —, tout le reste en erreur interne fixe (`AppError` compte environ 125 variantes et n'a ni `NotFound` ni `Conflict`) ; la garde de formes est remplacée par une sortie unique, `reponse::rendre(ReponseMcp)` (C-32-6-37), et un inventaire des lignes qui nomment `AppError`, `IntoResponse` ou `into_response`.)*

### C-32-6-34 — Les gardes lexicales : frontières de mot, forme qualifiée, commentaires écartés, témoins des deux sens (validation P1, F12, R12)

- **Contexte** : `mcp::` attrapait `cles_mcp::` ; `.executer(` ratait `McpTool::executer(…)` ; un doc-comment
  décrivant une forme interdite aurait fait rougir la garde de lecture seule.
- **Retenu** : `\bmcp::`, `\bexecuter\s*\(` hors `fn executer`, lignes de commentaire écartées partout, chaque garde
  éprouvée sur une source factice qui doit rougir et une qui ne doit pas ; `pool_lecture_seule_en_un_lieu` ne balaie que
  `crates/kesh-api/src` et le dit.
- **Écartées** : des gardes syntaxiques par `syn` pour toutes (plus lourdes ; gardées pour le seul cas où le texte ne
  suffirait pas, à juger au développement).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P3, R1, R2 : les motifs écrits ici étaient des formes non mesurées (`lister(` rougissait sur onze services). Les gardes sont désormais prescrites par leur **propriété**, leur périmètre et leurs formes, sous une règle commune écrite en tête des AC de la fiche : motif greppé sur le dépôt réel à l'étape 0, vide hors liste blanche, un témoin rouge et un témoin vert existant, sorties citées. Les points d'accès du registre s'appellent `outil_pour` et `descripteurs_pour`.)*
- *(Annoté le 2026-10-25, validation P5, R5 = F6 : « onze services » est faux — `lister` est le nom de **six** fonctions de services (`grep -n "pub async fn lister" crates/kesh-api/src/services/*.rs` → 6) ; onze est le nombre de services du rollout. La conclusion tient.)*

### C-32-6-35 — Toute connexion du pool du MCP est fermée à sa restitution (validation P2, R4)

- **Contexte** : la forme 2 de la spike (`SET TRANSACTION READ WRITE`, caractéristique de la transaction **suivante**,
  § 4.1) n'apparaît pas dans `@@session.tx_read_only` : un `before_acquire` qui relit cette variable garderait la
  connexion, et la première écriture de l'emprunt suivant passerait.
- **Retenu** : `after_release` → `Ok(false)` (`sqlx-core-0.8.6/src/pool/connection.rs:289-296` ferme la connexion) ;
  chaque emprunt part d'une connexion neuve passée par `after_connect` : aucun état de session ne survit d'un emprunt à
  l'autre, quelle que soit la forme. Tests (e) forme 4 et (e bis) forme 2. Le `before_acquire` est retiré.
- **Écartées** : un `before_acquire` qui exécuterait une transaction vide pour consommer la caractéristique en attente
  (raisonnement sur MariaDB non essayé, et une garde de plus) ; écrire la forme 2 en angle mort (le test est faisable
  sans infrastructure nouvelle).
- **Coût** : une connexion par emprunt (quelques millisecondes), pour un trafic MCP faible.
- **Réversible** : oui.

### C-32-6-36 — La politique en deux temps : `autoriser` avant l'exécution, `appliquer` ensuite (validation P2, F2)

- **Contexte** : `appliquer(validation, Donnees)` prenait le résultat de l'exécution, si bien qu'un second contrôle,
  innommé, devait refuser une clé `required` **avant** ; la mutation « bras intervertis » ne voyait pas ce contrôle.
- **Retenu** : `autoriser(validation) -> Result<Autorisation, Refus>`, seule lectrice de la politique, appelée avant
  `executer` ; `appliquer(Autorisation, Donnees) -> DonneesEnvoyables` ne fait que construire la réponse ; `Autorisation`
  à champ privé, fabriquée par `autoriser` seule — appeler `appliquer` sans autorisation ne compile pas. Mutations :
  `autoriser` qui laisse passer `Required` (tuée par le compteur et E1) ; `appliquer` sans `autoriser` (ne compile pas).
- **Écartées** : `executer_sous(validation, fermeture)` (une fermeture asynchrone empruntée, plus lourde à écrire et à
  lire) ; un contrôle préalable dans le répartiteur, hors de `politique.rs` (deux lieux pour une règle).
- **Réversible** : oui (c'est le point de branchement de la 32-8).

### C-32-6-37 — Une seule sortie : `reponse::rendre(ReponseMcp)` (validation P2, R2)

- **Contexte** : une garde lexicale sur les **formes** qui mèneraient une `AppError` à `IntoResponse` ne voyait pas un
  `Result<_, AppError>` remonté par une fonction intermédiaire puis rendu par `e.into_response()`.
- **Retenu** : toute réponse de `/mcp` sort de `mcp::reponse::rendre`, à partir de `ReponseMcp`, énumération propre au MCP
  sans `AppError` ni `Response` ; `AppError` n'est convertible que par `depuis_app_error` et par le `match` du résultat
  d'`authentifier` ; garde d'**inventaire des sites** (lignes qui nomment `AppError`, `IntoResponse` ou `into_response`
  dans `src/mcp/**` et `routes/mcp.rs`, liste écrite).
- **Écartées** : garder la garde de formes (une énumération de ce qui marche, `CLAUDE.md` § *Inventorier les sites NON
  RÉSOLUS*) ; une analyse par `syn` des types de retour (plus lourde, sans fermer davantage).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P3, R4 = F4 : `rendre` est **`async fn rendre(&EtatMcp, ReponseMcp) -> Response`** et `ReponseMcp` porte `RouteAbsente(Request)` — C-32-6-39.)*

### C-32-6-38 — La règle « servi » et l'authentification lisent sur la `Lecture` ordinaire, avant tout emprunt du pool des outils (validation P3, R9)

- **Contexte** : depuis `after_release` (C-32-6-35), chaque emprunt du pool des outils ouvre une connexion neuve ;
  la règle « servi » s'y faisait **avant** le filtre et l'authentification, si bien que toute requête anonyme — même
  sur une installation où le MCP n'a jamais été activé — ouvrait et fermait une connexion MariaDB, et qu'un poste
  pouvait saturer les trois connexions du pool des outils au détriment de l'agent légitime.
- **Retenu** : `EtatMcp.garde = state.lecture()` (la `Lecture` ordinaire de l'API, connexions réutilisées) pour
  `reglages_actifs` et `cles_mcp::authentifier` — comme `validate_pat` de l'API lit sur le pool ordinaire ; le pool des
  outils, en lecture seule, n'est emprunté qu'**après** l'authentification et `autoriser`. Ces deux lectures sont du
  code fixe de Kesh, sans outil ni donnée de l'agent : Q11 (les outils lisent en lecture seule) est tenue.
- **Écartées** : un troisième pool en lecture seule pour la garde (un pool de plus pour deux requêtes fixes) ; un cache
  des réglages (un serveur désactivé resterait ouvert jusqu'à l'expiration) ; une limite de débit (soumise à Guy,
  point 8).
- **Ce qui reste** : une inondation du réseau local occupe les connexions ordinaires, comme `/api/v1/auth/login` le
  permet déjà — angle mort écrit, point 8 pour Guy.
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P4, F2 = R1 et F12 : la `garde` et la lecture des outils ne sont plus du même type (C-32-6-41) ; la lecture de Q11 que ce choix suppose — la connexion des outils en lecture seule, non celle de la garde — est soumise à Guy (point 8 de la fiche).)*

### C-32-6-39 — `rendre` est asynchrone et reçoit l'état ; `ReponseMcp::RouteAbsente(Request)` (validation P3, R4 = F4)

- **Contexte** : la réponse de route absente est celle du service statique, qui ne se produit qu'en l'appelant avec la
  `Request` ; une `rendre(ReponseMcp) -> Response` synchrone et sans état ne le pouvait pas, et `DonneesEnvoyables`
  n'avait pas de lecture prévue.
- **Retenu** : `pub async fn rendre(etat: &EtatMcp, r: ReponseMcp) -> Response` ; `RouteAbsente(Request)` remise à
  `etat.statique` par `oneshot`, puis `.map(Body::new)` ; `DonneesEnvoyables::en_resultat`, définie en `politique.rs` ;
  corps des gestionnaires : `rendre(&etat, transport::traiter(&etat, pair, requete).await).await`.
- **Écartées** : une variante qui porterait déjà la `Response` statique (une deuxième sortie de réponse, hors de
  `rendre`) ; deux fonctions de sortie (statique et MCP).
- **Réversible** : oui.

### C-32-6-40 — `/mcp` monté par `route_service`, comme le repli (validation P4, F1)

- **Contexte** : un `MethodRouter` (`post(…)`, `.fallback(…)`, `any(…)`) rend un futur « top level » qui ajoute `Allow`
  (l'en-tête accumulé, attaché au repli du `MethodRouter`, `axum-0.8.9/src/routing/method_routing.rs:1168-1174`) et
  `content-length` (`routing/route.rs:163-167`) ; le repli statique, lui, est un `Endpoint::Route` appelé
  `not_top_level` (`routing/mod.rs:366-370`, `routing/path_router.rs:413`, `routing/route.rs:44-46`). Un `GET /mcp`
  désactivé aurait porté `Allow: POST`, la route absente non.
- **Retenu** : `main_router.route_service("/mcp", routes::mcp::recevoir.with_state(etat))` — un `Endpoint::Route`
  (`path_router.rs:138`) appelé par le même `call_owned` que le repli ; la méthode est tranchée par le transport ;
  `Handler::with_state` (`handler/mod.rs:202`) garde les extracteurs. Le registre des routes lit `route_service(`.
- **Écartées** : `any(…)`, qui retire `Allow` (`skip_allow_header`) mais garde le `content-length` du « top level » ; une
  couche qui retirerait les en-têtes ajoutés (une garde de plus, fragile à la version d'Axum).
- **Réversible** : oui.

### C-32-6-41 — La lecture des outils a un type propre, `LectureOutils` (validation P4, F2 = R1)

- **Contexte** : depuis la P3, `EtatMcp` porte deux `Lecture` du même type : la `garde` (pool ordinaire) et celle des
  outils (lecture seule). Les confondre — les outils sur la `garde`, ou la règle « servi » sur le pool des outils —
  compilait, et la seconde confusion n'était vue par aucun test.
- **Retenu** : `LectureOutils`, défini en `mcp/outil.rs`, champ privé, construit une fois en `lib.rs`, jamais déballé
  hors de ce fichier ; `ContexteMcp::nouveau` l'exige et `ContexteMcp::lecture()` rend seul la `&Lecture` à l'outil.
  Les deux confusions sont des erreurs de type (Ma51, Ma52, déclarées non compilables).
- **Écartées** : une garde lexicale « `etat.lecture` n'est nommé qu'au répartiteur » (le type ferme la classe sans
  garde) ; ne remettre au répartiteur que `(&Registre, &LectureOutils, &Trace)` (même effet, plus de signatures).
- **Angle mort** : un accesseur public ajouté à `LectureOutils` — la revue.
- **Réversible** : oui.
