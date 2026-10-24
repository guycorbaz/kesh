# Epic 32 — registre des choix autonomes de la story 32-2 (spike)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Document de
décision : `planning-artifacts/epic-32-spike-mcp.md` ; journaux des essais hors dépôt :
`/home/gcorbaz/devel/kesh-gate-logs/322-spike/`. Date : 2026-10-24.

---

### C-32-2-1 — Révision figée : `2025-11-25`, et non la *Current* `2026-07-28`

- **Contexte** : la révision *Current* est `2026-07-28` (sans `initialize`, sans session) ; Hermes Agent repose sur le
  SDK Python 1.x (poignée de main), et la matrice de la spécification donne « Legacy client | Modern server | Fails » ;
  Claude Code se replie sur l'ancienne poignée de main pour un serveur qui ne parle pas la nouvelle.
- **Retenu** : servir `2025-11-25` (révision *Final*, figée par définition), accepter `2025-06-18` à la négociation.
- **Écartées** : `2026-07-28` seule (Hermes exclu) ; double ère écrite à la main (deux protocoles à tenir pour un
  besoin minuscule).
- **Réversible** : oui — ajouter `2026-07-28` est un adaptateur de transport de plus derrière le registre (§ 5.3).
- **Relecture P1** : prémisse « Hermes repose sur la poignée de main » fausse — justification refaite par **C-32-2-11** ; le choix tient.


### C-32-2-2 — Sans état : aucun `Mcp-Session-Id`, `GET`/`DELETE` 405, pas de lot, JSON par défaut

- **Contexte** : en `2025-11-25`, la session est facultative (« MAY assign ») et le `GET` peut rendre 405 ; les lots
  sont interdits depuis `2025-06-18`. Prototype essayé au `curl` (`proto-curl.out`).
- **Retenu** : aucune session ; `GET`/`DELETE` 405 ; notification 202 ; lot 400 ; `application/json`, SSE seulement
  pour l'appel en attente de validation.
- **Écartée** : sessions en mémoire (état perdu au redémarrage, rien à y mettre, et supprimées par la révision suivante).
- **Réversible** : oui.
- **Relecture P3 (L-1)** : la forme « élicitation » répond aussi en SSE (la requête `elicitation/create` part sur le flux
  du `tools/call`) ; § 1.3 rectifié, cf. **C-32-2-26**.

### C-32-2-3 — `MCP-Protocol-Version` non servi → 400 à corps texte ; `Origin` absent admis, étranger → 403

- **Contexte** : un client double ère (Claude Code v2) sonde par une requête moderne et ne se replie que sur un `4xx`
  « without a recognized modern error body » ; la spécification impose le 403 sur un `Origin` présent et invalide, et
  un client en ligne de commande n'envoie a priori pas d'`Origin` (non éprouvé).
- **Retenu** : version non servie (dont `2026-07-28`) → 400 texte, jamais `-32022` ; en-tête absent → admis ;
  `Origin` présent ≠ origines de Kesh → 403 ; absent → admis, la clé restant la barrière.
- **Écartée** : exiger `Origin` (refuserait les clients en ligne de commande) ; répondre `-32022` (bloquerait le repli).
- **Réversible** : oui.

### C-32-2-4 — Validation avant envoi : l'écran de Kesh, attente bornée à 240 s en SSE

*⚠️ Remplacé en relecture P1 par C-32-2-14 et C-32-2-15 — annotation de la relecture P2 (L-7).*

- **Contexte** : l'élicitation est garantie par le client — qui reçoit les données — et contournable par lui (*hook*
  `Elicitation` de Claude Code, YOLO de Hermes), limitée à des formulaires plats, refusée hors session interactive ;
  Hermes coupe un `tools/call` à 300 s, Claude Code exige un premier octet sous 60 s et une activité toutes les 5 min.
- **Retenu** : écran de Kesh ; le répartiteur reçoit une proposition d'envoi, rend la connexion de lecture, attend
  **240 s** au plus sur une réponse SSE entretenue (premier évènement immédiat, progression toutes les 15 s) ; refus ou
  expiration → `isError: true`, rien ne part. Repli si l'essai client échoue : poignée explicite rappelée par l'agent.
- **Écartée** : élicitation en 0.16.0 (complément possible plus tard, en mode URL vers la page d'approbation).
- **Réversible** : oui ; **reste à éprouver** avec les deux clients (§ 2.4 d du document).
- **Relecture P1** : prémisses fausses ou incomplètes (YOLO, minuteur, tâche de fond) ; **le choix n'est plus pris par la spike** — remplacé par **C-32-2-14** (ce qui reste du rejet de l'élicitation) et **C-32-2-15** (trois formes, recommandation, Q10 à Guy).


### C-32-2-5 — Lecture seule : second pool `READ ONLY` + `before_acquire` + garde lexicale

*⚠️ Remplacé en relecture P1 par C-32-2-16 — annotation de la relecture P2 (L-7).*

- **Contexte** : essai réel sur `kesh_322`, utilisateur `kesh` : `INSERT`/`UPDATE`/`DELETE`, `FOR UPDATE`, DDL et table
  temporaire refusés en 1792, autocommit comme transaction ; mais `START TRANSACTION READ WRITE`, `SET TRANSACTION READ
  WRITE`, `SET SESSION tx_read_only=0` et `SET SESSION TRANSACTION READ WRITE` le défont sans privilège, et sqlx ne
  remet pas l'état de session à zéro entre deux emprunts (séquence B de `ro-pool.out`).
- **Retenu** : second pool construit dans `main.rs` par une fonction unique de `kesh-db` (cf. #669), `after_connect`
  `SET SESSION TRANSACTION READ ONLY`, `before_acquire` qui jette une connexion dont `@@session.tx_read_only` ≠ 1, garde
  lexicale sur `READ WRITE` / `tx_read_only` / `transaction_read_only` dans `crates/`.
- **Écartée** : second utilisateur MariaDB `SELECT` seul (repli ; une variable `.env` et un script d'init de plus sur
  le NAS) ; `after_connect` seul (séquence B).
- **Réversible** : oui.
- **Relecture P1** : « par construction » ne vaut pas pour ce verrou ; **le choix n'est plus pris par la spike** — remplacé par **C-32-2-16** (trois formes, recommandation, Q11 à Guy).


### C-32-2-6 — SDK : écrit à la main en 0.16.0, traits à types propres à Kesh

- **Contexte** : `rmcp` 3.7.0 (2026-10-22) est officiel, Tier 1, double ère, MSRV 1.88, monté par `nest_service` ;
  mais 70 versions et trois majeures en huit mois (1.0.0 2026-03-03, 2.0.0 2026-06-29, 3.0.0 2026-07-28).
- **Retenu** : la décision de Guy (écrit à la main) ; aucun type `rmcp`/JSON-RPC dans `McpTool`/`McpPrompt` ;
  réexamen quand un client exigera `2026-07-28` (Q8 à Guy).
- **Écartée** : adopter `rmcp` dès la 32-6 (contredit une décision du Project Lead ; le critère de maturité lui
  revient).
- **Réversible** : oui, c'est l'objet des traits.
- **Relecture P1** : critère de réexamen reformulé par **C-32-2-11** (« quand Kesh doit servir `2026-07-28` », les deux clients la parlant déjà) ; traits revus par **C-32-2-17**.
- **Relecture P2 (L-6)** : « en huit mois » ne se rapporte à aucune fenêtre définie — les trois majeures tiennent en moins de cinq mois (2026-03-03 → 2026-07-28) ; document corrigé, entrée laissée telle quelle.


### C-32-2-7 — Prototype hors du dépôt plutôt que sur une branche `spike/32-2-proto`

- **Contexte** : le prompt proposait un prototype dans le worktree sur une branche locale non fusionnée ; une cible
  autonome (deux fichiers) suffisait et évitait de changer de branche dans le worktree et de compiler le workspace.
- **Retenu** : cible cargo jetable dans le scratchpad de la session (`--offline`, `Cargo.lock` du dépôt copié), sous
  `flock` et `mem-guard.sh`, supprimée après l'essai ; sources et sorties gardées dans `kesh-gate-logs/322-spike/`.
- **Écartée** : branche `spike/32-2-proto` (aucun gain, risque de commit par mégarde).
- **Réversible** : oui (les sources se reconstruisent).

### C-32-2-8 — Clients réels : documentation seulement, gestes écrits

- **Contexte** : consigne de la spike — aucune configuration utilisateur modifiée, aucun client lancé.
- **Retenu** : décisions fondées sur les documentations officielles citées ; tout comportement client est marqué **non
  éprouvé**, avec les commandes (dont `claude --strict-mcp-config --mcp-config <fichier jetable>`, qui n'écrit rien) et
  les huit observations à consigner (§ 2.4).
- **Écartée** : lancer `claude -p` avec un fichier de configuration jetable (lancement d'un client, hors consigne).
- **Réversible** : sans objet.

### C-32-2-9 — « Hermes » = Hermes Agent de Nous Research (hypothèse)

- **Contexte** : le nom est ambigu ; rien n'est installé sur la station.
- **Retenu** : Hermes Agent (`hermes-agent.nousresearch.com`), seul client MCP de ce nom trouvé — **à confirmer par
  Guy** (Q9).
- **Réversible** : oui ; si c'est un autre client, C-32-2-1 et C-32-2-4 sont à revoir.

---

*Rectifications de la relecture P1 (rapport hors dépôt `/home/gcorbaz/devel/kesh-gate-logs/322-review-p1.md`,
0 CRITICAL / 0 HIGH / 7 MEDIUM / 7 LOW), 2026-10-24. Sources revérifiées en ligne le même jour.*

### C-32-2-10 — M1 : le mode YOLO de Hermes ne répond pas à l'élicitation

- **Contexte** : C-32-2-4 donnait « Hermes par YOLO » comme contournement de l'élicitation. La documentation de Hermes
  (`…/docs/user-guide/features/mcp`) dit : « YOLO and `approvals.mode: off` do not answer these for you: the server is
  asking you a question, not asking permission ».
- **Retenu** : seul le *hook* `Elicitation` de Claude Code répond sans dialogue — posé par l'utilisateur dans sa propre
  configuration. Document §§ 2.2, 3.2.
- **Réversible** : sans objet (rectification de fait).

### C-32-2-11 — M2 : Hermes parle les deux ères (`protocol: auto`) ; justification de `2025-11-25` refaite

- **Contexte** : le document disait Hermes *legacy* seul. `…/docs/reference/mcp-config-reference` : `protocol` par
  défaut `auto` (« legacy `initialize` handshake first, falling back to the 2026-07-28 `server/discover` »), `stateless`,
  `legacy`.
- **Retenu** : `2025-11-25` maintenue, au motif qu'un serveur `2025-11-25` fonctionne dans **tous** les réglages
  documentés des deux clients (y compris Hermes `legacy`, Claude Code v1 ou `MCP_PROTOCOL_NEGOTIATION=legacy`), ce
  qu'un serveur moderne seul ne garantit pas. Critère de réexamen du SDK : « quand Kesh doit servir `2026-07-28` », et
  non « quand un client l'exige ». Document §§ 1.3, 2.2, 5.2, Q8.
- **Réversible** : oui (C-32-2-1).
- **Relecture P2 (M-1, L-5)** : le motif « fonctionne dans **tous** les réglages documentés » dit plus que les sources — elles établissent l'**ère** `initialize`, non la révision que chaque client demande ni qu'il accepte `2025-11-25` ; justification refaite par **C-32-2-19**. La citation de Hermes ci-dessus est tronquée sans marque (« … `server/discover` stateless probe when the server rejects the handshake as modern-only ») ; le document la cite entière.

### C-32-2-12 — M3 : minuteur de premier octet de Claude Code — finding **partiellement réfuté**

- **Contexte** : le rapport lisait « avec la configuration par défaut, ce minuteur vaut des heures ». La documentation
  (`https://code.claude.com/docs/en/mcp`) poursuit : « The 28-hour default of an unset `MCP_TOOL_TIMEOUT` doesn't enter
  that comparison » — **par défaut, il vaut 60 s**, comme la première rédaction l'écrivait. Ce qui était faux : « la doc
  ne dit pas » si le premier évènement le satisfait — elle le dit (« through to the server's first response byte »).
- **Retenu** : formule complète citée (max de 60 s, du délai d'outil du serveur et de `MCP_TIMEOUT`, défaut non
  compté) ; une réponse qui commence aussitôt n'y est jamais soumise ; l'essai § 2.4 (d) à 90 s garde son objet (le
  premier évènement SSE suffit-il ?). Document §§ 2.1, 2.3, 2.4.
- **Réversible** : sans objet.
- **Relecture P2 (L-1, L-2)** : l'essai à 90 s seul ne juge pas le premier octet — un contrôle négatif (premier évènement retardé à 70 s) lui est adjoint au § 2.4 (d) ; la portée du minuteur (HTTP, SSE, connecteur claude.ai ; ni stdio ni WebSocket) est ajoutée au § 2.1.

### C-32-2-13 — M4 : Claude Code passe en tâche de fond un appel de plus de 2 min — conséquence tirée

- **Contexte** : « Claude receives the task ID immediately and keeps working, and the result arrives as a task
  notification when the call settles » ; la tâche « doesn't survive exiting the session » ; pas de tâche de fond en
  session non interactive (sauf `CLAUDE_AUTO_BACKGROUND_TASKS=1`) ; aucune tant qu'un dialogue d'élicitation est ouvert.
- **Retenu** : un appel tenu ouvert 240 s devient, chez Claude Code, une poignée de fait à 120 s — l'agent poursuit sans
  le résultat. Critère inscrit au tableau du § 3.3 ; observation (d) du § 2.4 à 150 s.
- **Réversible** : sans objet.

### C-32-2-14 — Ce qui reste du rejet de l'élicitation

- **Contexte** : C-32-2-4 reposait sur trois motifs : contournable (à moitié faux, C-32-2-10), formulaires plats (tient),
  flux serveur → client absent de la révision sans état (faux, L1 : la requête part sur le flux SSE du `tools/call`, et
  MRTR est sans état).
- **Retenu** : l'élicitation n'est pas recommandée pour l'ergonomie de la modification (champs plats), le fonctionnement
  sans session interactive, et le réemploi de l'écran pour les propositions de la 0.17.0 — **préférence raisonnée, non
  impossibilité** ; l'argument « le client garantit » demeure comme principe, de faible poids pour un seul utilisateur.
  Ce qui plaide pour elle est écrit (pas d'écran à construire, pas de sortie de la fenêtre du client). Document § 3.2.
- **Réversible** : oui.

### C-32-2-15 — M5 : trois formes de la validation pesées ; recommandation la poignée ; **décision à Guy (Q10)**

- **Contexte** : la poignée était reléguée en repli sans être pesée contre les risques d'un appel bloquant 240 s (deux
  délais clients, tâche de fond, mandataire, état en mémoire).
- **Retenu** : tableau à onze critères (§ 3.3) — (a) élicitation, (b) appel tenu ouvert ≤ N s, (c) poignée.
  *Recommandation* : (c), soumise à Guy (Q10) ; la spike ne tranche pas. Conséquences sur la 32-8 écrites pour chaque
  forme (§ 6) — en (c), la 32-8 devient 🗄️ (table des envois en attente).
- **Écartée** : trancher dans la spike (c'est une décision d'ergonomie et de modèle de données, elle revient à Guy).
- **Réversible** : oui jusqu'à la 32-8.

### C-32-2-16 — M6 : trois formes de la lecture seule ; « par construction » retiré là où il ne vaut pas ; **décision à Guy (Q11)**

- **Contexte** : le mode de session `READ ONLY` est défait par quatre instructions sans privilège ; `before_acquire` ne
  contrôle qu'à l'emprunt ; la garde lexicale ne voit que le texte du dépôt. Le second utilisateur `SELECT` avait été
  écarté pour « une variable `.env` et un script d'init de plus », sans dire la vraie contrainte.
- **Retenu** : (a) mode de session + `before_acquire` + garde lexicale — garantie « tant que le code de Kesh n'émet pas
  ces instructions » ; (b) utilisateur `SELECT` — par construction, mais créé **à la main** sur le NAS (les scripts
  d'init ne rejouent pas sur une base existante ; `docker-compose.prod.yml` sans MariaDB ; Kesh sans droit global pour le
  créer) et une variable d'accès à la base de plus dans le `.env` ; (c) les deux. *Recommandation* : (c), avec contrôle
  des droits au démarrage et MCP non activable sans la variable — soumise à Guy (Q11). Document §§ 4.3, 6.
- **Écartée** : garder « par construction » pour (a).
- **Réversible** : oui jusqu'à la 32-6.

### C-32-2-17 — M7 : la politique de validation sort du trait ; `PromptsWrite` a son propre trait ; **portée à Guy (Q12)**

- **Contexte** : le trait faisait rendre une `PropositionEnvoi` à tout outil, y compris celui qui écrit un prompt (une
  écriture, non un envoi), et ne laissait aucune place à une clé dispensée ; la règle de Guy (« montrer les données qui
  vont être envoyées ») pèse lourd sur l'agent de développement.
- **Retenu** : `McpTool` rend des `Donnees` ; seul le répartiteur fabrique la réponse et applique `Validation` de la clé
  (`Requise` / `Dispensee`, posée par l'administrateur, envois journalisés) ; `McpToolPrompts` (contexte sans
  `Lecture`, retour `Accuse` fermé) pour `proposer_prompt` ; `McpPrompt::rendre` sans `Lecture`. Question Q12 (toujours ;
  par clé ; seuil), *recommandation* : par clé, `requise` par défaut — elle s'écarte de la Q2 du kickoff, même question.
  Document §§ 5.3, 6, 7.
- **Réversible** : oui (traits indicatifs).
- **Relecture P3** : la direction (politique de la clé appliquée par le répartiteur, non par l'outil) est gardée comme
  exigence E1 ; les types qui la portaient deviennent une esquisse non contraignante — **C-32-2-24**.

### C-32-2-18 — LOW L1 à L7 : rectifications mécaniques

- **L1** : décision 3 et § 3.2 — le flux serveur → client existe en `2025-11-25` sur le SSE du `tools/call`, MRTR est
  sans état.
- **L2** : liste des journaux du § en-tête — `ro-pool-main.rs` et `ro-pool-Cargo.toml` → `ro-pool.out` ; `proto-server.log`
  ajouté.
  *(Relecture P2, L-9 : ce journal est vide, 0 octet ; retiré de la liste du document.)*
- **L3** : `grep -rniE "read write|tx_read_only|transaction_read" crates` ne rend **aucune ligne** (revérifié).
- **L4** : `DELETE` 405, `ping`, borne de 1 Mio, `MCP-Protocol-Version` absent — écrits « décidés, non éprouvés par le
  prototype » (§ 1.3) et ajoutés au test de contrat (32-6, 32-11).
- **L5** : `Origin` d'un navigateur sur l'adresse du réseau local refusé si l'URL publique est vide ; liste réglable,
  jamais déduite de `Host` (*DNS rebinding*) (§ 1.3).
- **L6** : `touch_last_used` ne concerne le MCP que si la clé MCP réutilise `validate_pat` ; la 32-5 dit où la clé MCP
  écrit sa dernière utilisation (§ 4.4).
- **L7** : filtre d'adresse, activation par l'administrateur, prompts « à valider » — rappelés en tête du § 6 et
  renvoyés aux stories du kickoff (32-5, 32-6, 32-7, 32-10).

---

*Rectifications de la relecture P2 (rapport hors dépôt `/home/gcorbaz/devel/kesh-gate-logs/322-review-p2.md`,
0 CRITICAL / 0 HIGH / 4 MEDIUM / 11 LOW, Haiku 5.5, passe ciblée), 2026-10-24. Sources revérifiées en ligne le même
jour : `code.claude.com/docs/en/mcp`, `…/2025-11-25/basic/transports`, `…/2025-11-25/basic/lifecycle`, la référence de
configuration de Hermes, `hub.docker.com/_/mariadb`.*

### C-32-2-19 — M-1 : `2025-11-25` justifiée par l'ère `initialize`, la révision demandée restant à observer

- **Contexte** : le § 1.3, le § 2.3 et C-32-2-11 disaient qu'un serveur `2025-11-25` « est servi par les deux clients dans
  chacun de leurs réglages documentés » et que « Claude Code v1 ne parle qu'elle ». Les sources disent : Hermes tente ou
  retente `initialize` dans ses trois réglages ; Claude Code v1 et `legacy` gardent la poignée de main antérieure, le v2
  « adds » la `2026-07-28` ; aucune ne dit quelle `protocolVersion` est demandée, et le repli du v2 sur un serveur de
  l'ère `initialize` n'y est pas écrit en toutes lettres. La spécification (`lifecycle`) veut que le client « SHOULD
  disconnect » s'il ne connaît pas la version répondue.
- **Retenu** : `2025-11-25` maintenue — révision *Final* la plus récente de la seule ère que tous les réglages décrits
  atteignent, `2025-06-18` acceptée — **sous réserve de l'observation (h) du § 2.4, à faire avant la 32-6** ; si un client
  demande une révision plus ancienne, l'accepter est réversible et local, à condition que les méthodes servies y aient
  la même forme (non vérifié pour `2025-03-26`). Document : décision 1, §§ 1.3, 2.3.
- **Écartée** : servir double ère dès la 0.16.0 pour lever l'incertitude (double le protocole écrit à la main, pour un
  risque qu'une observation tranche).
- **Réversible** : oui.

### C-32-2-20 — M-2 : l'élicitation tient l'appel ouvert ; délais du § 2.3 réécrits par forme

- **Contexte** : « pour une réponse immédiate (formes (a) et (c)) aucun de ces délais ne joue » était faux pour (a) :
  l'élicitation attend l'utilisateur (Hermes : `elicitation.timeout` 300 s, et `timeout` d'outil 300 s).
- **Retenu** : § 2.3 par forme — (a) et (b) tiennent l'appel ouvert, (c) seule répond aussitôt ; pour (a), premier octet
  satisfait par le flux SSE, pas de tâche de fond chez Claude Code tant que le dialogue est ouvert, inactivité de 5 min
  pendant le dialogue **non établie**, préséance des deux délais de Hermes non écrite. Cellule (a) du tableau du § 3.3
  alignée.
- **Réversible** : sans objet (rectification).

### C-32-2-21 — M-3 : la garantie de validation porte sur toute méthode servie ; pas de ressources en 0.16.0

- **Contexte** : « le seul chemin des `Donnees` vers le client passe par `Validation` » ne couvrait que `tools/call` ;
  le § 6 annonçait `resources/read` pour le contenu d'une pièce, sans trait ni passage par le répartiteur ; les messages
  d'erreur (`ErreurOutil::ArgumentsInvalides(String)`) et `prompts/get` n'étaient pas examinés.
- **Retenu** : **aucune ressource MCP servie en 0.16.0** (méthodes `resources/*` → `-32601`, capacité non déclarée) ; la
  32-9b livre les métadonnées des pièces, comme C-32-0-10 le prévoit à défaut d'une forme éprouvée par la spike ; le
  contenu, en 0.17.0, passera par un outil ou par un trait `McpRessource` soumis au même répartiteur. Au § 5.3 :
  inventaire clos des méthodes servies et de ce que chacune envoie ; `DonneesValidees` à constructeur privé au
  répartiteur et type `Reponse` seul sérialisable ; `ErreurOutil` sans texte libre ; `McpPrompt` scellé, seule
  implémentation sans pool ; prompts au `Registre`. Tests de la 32-6 : inventaire des méthodes (rougit sur une méthode
  non inscrite), `-32601` sur chaque méthode non servie du schéma, constructeur privé (mutation à tuer), valeur piégée
  absente des erreurs. Cas assumé, écrit : un texte de prompt où un humain a recopié des valeurs (couvert par
  l'approbation du texte) ; en forme (a), l'élicitation envoie les données au client avant l'accord.
- **Écartée** : ajouter dès maintenant un trait de ressources (rien à servir en 0.16.0, et une forme non éprouvée).
- **Réversible** : oui (C-32-0-10 laisse glisser le contenu des pièces sans toucher le reste).
- **Relecture P3** : « aucune ressource en 0.16.0 » et l'inventaire des méthodes sont gardés (exigences E2, E3) ; la
  garantie « par construction » par `DonneesValidees` et `Reponse` est **retirée** (non prouvée, et `Reponse` ouverte) —
  **C-32-2-24**.

### C-32-2-22 — M-4 : bloc du § 4.2 recopié depuis `ro-pool.out`

- **Contexte** : le bloc citait `313122`, `313126`/`313127`, `313128`, `313130`, absents du journal (qui porte `313133`…
  `313141`) ; préexistant à la relecture P1.
- **Retenu** : lignes 1-10, 18 et 26-37 de `ro-pool.out` recopiées octet pour octet (contrôle : chaque ligne du bloc,
  hors marques `[…]`, se trouve telle quelle dans le journal) ; les omissions sont marquées entre crochets. Le bloc du
  § 1.2 (`proto-curl.out`) est dit **remis en forme** — sa mention « en-tête `Date` retiré » était fausse (le journal
  n'en porte aucun).
- **Réversible** : sans objet.

### C-32-2-23 — LOW L-1 à L-11 de la relecture P2

- **L-1** : § 2.4 (d) — essai à 90 s avec premier évènement immédiat, plus un contrôle négatif à premier évènement
  retardé à 70 s (le prototype devra recevoir ce retard).
- **L-2** : § 2.1 — portée du minuteur de premier octet (HTTP, SSE, connecteur claude.ai), citée.
- **L-3** : § 2.1 — conditions du runtime v2 par défaut recopiées de la page (sessions qui lisent ou non les *feature
  flags*, exemples, réserve `CLAUDE_CODE_PROVIDER_MANAGED_BY_HOST`, silence sur les versions antérieures).
- **L-4** : § 2.2 — « Streamable HTTP par défaut » et « coupe à 300 s, dur » marqués **non sourcés** ; la coupure dure est
  gardée comme hypothèse prudente (tableau du § 3.3 : « supposée dure, non écrite »).
- **L-5** : citations rétablies — spécification « before sending the JSON-RPC *response* » (§ 3.1) ; Hermes, phrase
  entière (§§ 1.3, 2.2) ; C-32-2-11 annoté.
- **L-6** : « trois majeures en moins de cinq mois » (décision 5) ; C-32-2-6 annoté.
- **L-7** : sous les titres de C-32-2-4 et C-32-2-5, une ligne « remplacé » ajoutée (titres laissés intacts, registre tenu par union).
- **L-8** : § 5.3 — `CataloguePrompts` au `Registre`, `NomPrompt` défini, `McpPrompt` scellé (l'absence de `Lecture` ne
  tenait que par la signature) ; même règle pour les outils, vérifiée en revue.
- **L-9** : `proto-server.log` (vide) retiré de la liste des journaux ; C-32-2-18 annoté.
- **L-10** : § 4.3 — les `MARIADB_*` sourcés par la documentation de l'image officielle, non plus par `.env.example`.
- **L-11** : § 3.3 — « (b) inchangée en `2026-07-28` » remplacé par **non établi**.

### C-32-2-24 — Relecture P3 : retrait — le § 5.3 garde les exigences, la conception des types passe aux fiches 32-6 et 32-8

- **Contexte** : relecture P3 (`kesh-gate-logs/322-review-p3.md`, 0 / 0 / 3 MEDIUM / 11 LOW). Les trois MEDIUM portent
  tous sur la conception des types du § 5.3 écrite par la remédiation P2 : M-1 (la forme (c) n'a pas de chemin pour le
  contenu approuvé relu), M-2 (`Donnees` porte deux représentations, la modification n'en vise qu'une), M-3 (« par
  construction » non tenu : `Reponse` ouverte, mutation `pub(crate)` invisible à `trybuild`, garantie qui change avec
  `rmcp`). **Signal de recyclage**, déclaré par l'orchestrateur : P1 7 MEDIUM → P2 4, dont 3 nés de la remédiation P1 →
  P3 3, tous nés de la remédiation P2 et sur le même objet. Chaque passe corrige la conception de la précédente et en
  ouvre une autre : la boucle ne converge pas sur cet objet, parce qu'il n'est pas de la spike.
- **Retenu** : remédiation **par retrait**. Une spike décide ; elle ne conçoit pas les types d'une story.
  - § 5.3.1 : **exigences E1 à E6**, écrites comme AC à porter par les fiches — E1 toute donnée des livres vers le client
    (réponse d'outil, `structuredContent` **et** bloc texte, erreur, progression, élicitation, prompt, contenu relu par
    la poignée) passe par la validation et la donnée envoyée est exactement celle validée, une seule représentation
    source, l'autre dérivée après validation (M-2) ; E2 liste close des méthodes servies avec ce que chacune envoie, test
    d'inventaire, non servies tirées du schéma ; E3 aucune ressource MCP en 0.16.0 ; E4 forme (c) : table des envois en
    attente, outil de relecture, « même clé, même société » (M-1) ; E5 erreurs sans texte libre issu de la base ; E6
    les six mutations qui doivent rougir (M-3).
  - § 5.3.2 : les types (`Donnees`, `DonneesValidees`, `Reponse`, `MessagePrompt`, traits scellés, `trybuild`) deviennent
    une **esquisse indicative, non contraignante** ; les commentaires qui affirmaient une garantie sont rectifiés.
  - § 5.3.3 : les trous relevés (M-1, M-2, M-3, L-4 à L-7) deviennent des **points à porter** par les fiches 32-6 et 32-8.
  - « Par construction » retiré partout où la propriété n'est pas prouvée, avec ce qui la prouverait : pour la
    validation, la compilation du code des 32-6/32-8 et les mutations d'E6 ; pour la lecture seule (b)/(c), le test de
    la 32-6 sur le chemin de production (le refus `1142` n'a pas été essayé). § 6 (32-6, 32-8, 32-10a) réécrit pour
    renvoyer aux exigences et non plus aux types.
- **Écartées** : (i) une quatrième conception des types qui fermerait M-1 à M-3 — elle aurait recyclé une fois de plus,
  sans compilateur pour la juger ; (ii) découper la spike — l'objet en trop n'est pas une seconde décision, c'est du
  travail de story.
- **Réversible** : oui — les fiches 32-6 et 32-8 peuvent reprendre l'esquisse telle quelle si leur conception la
  retrouve, à condition de tenir E1 à E6.

### C-32-2-25 — Relecture P3, L-8 et L-9 : conséquences sur le kickoff, écrites au § 6.1, à reprendre par l'orchestrateur

- **Contexte** : la forme (c) fait de la 32-8 une story 🗄️, ce que le kickoff ne prévoit pas ; une restauration
  « serveur MCP activé » sans l'utilisateur `SELECT` ni sa variable n'a pas de comportement écrit.
- **Retenu** : § 6.1 — si (c) : ordre des migrations (la 32-8 s'insère dans la suite 32-5 → 32-6 → 32-10a, plus de
  parallèle avec une autre 🗄️), plafond de migrations de la release (une de plus), sauvegarde (table portée par la 32-8
  elle-même — « avec la 32-12 », écrit jusqu'ici, était faux), nombre de modules (au-delà de 5 : découpage préventif,
  p. ex. 32-8a / 32-8b), texte périmé de la fiche 32-8 du kickoff. Restauration sans le second utilisateur :
  comportement vraisemblable écrit (le serveur ne s'active pas, l'écran le dit), AC à attribuer (32-5, 32-6, 32-7), lié
  à #722. La spike ne modifie pas le fichier d'epic.
- **Réversible** : sans objet (relevé pour l'orchestrateur).

### C-32-2-26 — LOW mécaniques de la relecture P3 : L-1, L-2, L-3, L-10, L-11

- **L-1** : § 1.3 « Réponses » — la forme (a) répond aussi en SSE ; C-32-2-2 annoté.
- **L-2** : § 3.3, ligne `2026-07-28`, cellule (b) — « non établi » remplacé par le fait établi (la réponse SSE à un
  `POST` reste admise, reprise et `GET` supprimés, un flux rompu perd la requête), citations du changelog **telles que
  rapportées par la relecture P3** (non refetchées par la remédiation).
- **L-3** : § 5.3.2 — `async-trait` n'est pas une dépendance directe, mais figure au `Cargo.lock` en transitif (0.1.89,
  vérifié) ; la conclusion (inutile) tient.
- **L-10** : § 5.2 — « sept méthodes » → six requêtes et les notifications du client (E2).
- **L-11** : § 4.1 — parenthèse « MySQL l'admet », non sourcée, retirée.
- **L-4 à L-7** : non corrigés dans la spike — points à porter par les fiches (§ 5.3.3) ; L-8 et L-9 : C-32-2-25.
