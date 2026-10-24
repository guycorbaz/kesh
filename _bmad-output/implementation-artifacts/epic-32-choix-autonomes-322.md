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

### C-32-2-2 — Sans état : aucun `Mcp-Session-Id`, `GET`/`DELETE` 405, pas de lot, JSON par défaut

- **Contexte** : en `2025-11-25`, la session est facultative (« MAY assign ») et le `GET` peut rendre 405 ; les lots
  sont interdits depuis `2025-06-18`. Prototype essayé au `curl` (`proto-curl.out`).
- **Retenu** : aucune session ; `GET`/`DELETE` 405 ; notification 202 ; lot 400 ; `application/json`, SSE seulement
  pour l'appel en attente de validation.
- **Écartée** : sessions en mémoire (état perdu au redémarrage, rien à y mettre, et supprimées par la révision suivante).
- **Réversible** : oui.

### C-32-2-3 — `MCP-Protocol-Version` non servi → 400 à corps texte ; `Origin` absent admis, étranger → 403

- **Contexte** : un client double ère (Claude Code v2) sonde par une requête moderne et ne se replie que sur un `4xx`
  « without a recognized modern error body » ; la spécification impose le 403 sur un `Origin` présent et invalide, et
  un client en ligne de commande n'envoie a priori pas d'`Origin` (non éprouvé).
- **Retenu** : version non servie (dont `2026-07-28`) → 400 texte, jamais `-32022` ; en-tête absent → admis ;
  `Origin` présent ≠ origines de Kesh → 403 ; absent → admis, la clé restant la barrière.
- **Écartée** : exiger `Origin` (refuserait les clients en ligne de commande) ; répondre `-32022` (bloquerait le repli).
- **Réversible** : oui.

### C-32-2-4 — Validation avant envoi : l'écran de Kesh, attente bornée à 240 s en SSE

- **Contexte** : l'élicitation est garantie par le client — qui reçoit les données — et contournable par lui (*hook*
  `Elicitation` de Claude Code, YOLO de Hermes), limitée à des formulaires plats, refusée hors session interactive ;
  Hermes coupe un `tools/call` à 300 s, Claude Code exige un premier octet sous 60 s et une activité toutes les 5 min.
- **Retenu** : écran de Kesh ; le répartiteur reçoit une proposition d'envoi, rend la connexion de lecture, attend
  **240 s** au plus sur une réponse SSE entretenue (premier évènement immédiat, progression toutes les 15 s) ; refus ou
  expiration → `isError: true`, rien ne part. Repli si l'essai client échoue : poignée explicite rappelée par l'agent.
- **Écartée** : élicitation en 0.16.0 (complément possible plus tard, en mode URL vers la page d'approbation).
- **Réversible** : oui ; **reste à éprouver** avec les deux clients (§ 2.4 d du document).

### C-32-2-5 — Lecture seule : second pool `READ ONLY` + `before_acquire` + garde lexicale

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

### C-32-2-6 — SDK : écrit à la main en 0.16.0, traits à types propres à Kesh

- **Contexte** : `rmcp` 3.7.0 (2026-10-22) est officiel, Tier 1, double ère, MSRV 1.88, monté par `nest_service` ;
  mais 70 versions et trois majeures en huit mois (1.0.0 2026-03-03, 2.0.0 2026-06-29, 3.0.0 2026-07-28).
- **Retenu** : la décision de Guy (écrit à la main) ; aucun type `rmcp`/JSON-RPC dans `McpTool`/`McpPrompt` ;
  réexamen quand un client exigera `2026-07-28` (Q8 à Guy).
- **Écartée** : adopter `rmcp` dès la 32-6 (contredit une décision du Project Lead ; le critère de maturité lui
  revient).
- **Réversible** : oui, c'est l'objet des traits.

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
