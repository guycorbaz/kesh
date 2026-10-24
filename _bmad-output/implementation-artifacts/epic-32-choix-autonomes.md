# Epic 32 — registre des choix autonomes

Chaque choix tranché en autonomie : contexte, option retenue, alternatives écartées, réversibilité. Numérotation
`C-<story>-<k>`. Mise à jour **par union** (`docs/agents-orchestration.md` § 1) : chaque branche ajoute ses entrées dans
un registre propre à sa story (`epic-32-choix-autonomes-<clé>.md`), que l'orchestrateur verse ici au merge — **et non à
la rétrospective** : les Epics 30 et 31 l'avaient laissé à la fin, et il ne s'est fait qu'à la story 0 suivante (A2).

Fichier d'epic : `planning-artifacts/epic-32-serveur-mcp.md`.

---

## Story 32-0 — Kickoff (doc seulement)

### C-32-0-1 — Le registre vit dans `implementation-artifacts/`

- **Contexte** : le prompt du kickoff nomme `planning-artifacts/epic-32-choix-autonomes.md` ; `docs/agents-orchestration.md`
  § 1 place le registre de l'epic dans `implementation-artifacts/`, où sont ceux des Epics 15 à 31 (même écart déjà
  tranché ainsi par C-31-5-0).
- **Retenu** : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes.md`.
- **Écartée** : `planning-artifacts/` — un registre de plus hors de l'endroit où les agents et les rétrospectives le
  cherchent.
- **Réversible** : oui (un `git mv`).

### C-32-0-2 — Une spike de décision en tête, avant toute fiche MCP

- **Contexte** : la révision de la spécification, la forme du transport, la validation avant envoi (élicitation ou écran
  de Kesh) et la façon de prouver la lecture seule conditionnent toutes les stories suivantes, et dépendent du
  comportement réel de deux clients (Claude Code, Hermes) et de MariaDB 10.11 — rien de cela ne se tranche à la lecture.
- **Retenu** : 32-2, une spike à prototype jetable, non fusionné ; livrable : un document de décision et ses choix au
  registre. Recommandation à éprouver pour la lecture seule : un second pool construit **dans `main.rs`** (la production
  n'appelle pas `kesh_db::create_pool`, prémisse 7 du fichier d'epic), `SET SESSION TRANSACTION READ ONLY` à chaque
  connexion, essayé avec l'utilisateur `kesh` du fichier versionné, en autocommit comme en transaction.
- **Écartées** : trancher au kickoff sur la documentation des clients (c'est le mode d'échec « hypothèse éliminée par
  raisonnement », `CLAUDE.md` § E2E) ; un second utilisateur MariaDB `SELECT` seul d'emblée (une variable `.env` et un
  script d'init de plus sur le NAS, le chemin que Guy veut quitter — il reste le repli si l'essai échoue).
- **Réversible** : oui.

### C-32-0-3 — La story de défauts en tête d'epic, une fiche, trois clés

- **Contexte** : la rétrospective 31 recommande une story de défauts courte en tête (Q3, laissée à Guy) ; les bugs n'ont
  pas baissé sur deux epics ; A8 vise les LOW mécaniques de #711, #714 (hors ses deux choix), #716. Les constats touchent
  plus de cinq modules en tout (`kesh-api`, `kesh-db`, frontend `admin-backup` et `fiscal-years`, `kesh-i18n`, manuel).
- **Retenu** : 32-1, une fiche, trois clés (32-1a tests et commentaires, 32-1b écran de la sauvegarde automatique, 32-1c
  procédure de répétition), comme la 31-1 dont le découpage a donné trois revues courtes ; indépendante du MCP, en
  parallèle. **E-1 et E-2 de #711** (délai d'un verrou tenu, échec de `record_success`) ne sont pas mécaniques : la
  fiche les sort dans une issue plus étroite, sauf preuve contraire au code ; les deux choix de procédure de #714 restent
  à Guy.
- **Écartées** : une seule clé (au-delà de cinq modules) ; repousser les défauts (troisième epic sans baisse des bugs) ;
  attendre la réponse de Guy à Q3 (la story peut se retirer sans rien défaire).
- **Réversible** : oui (la story se retire ou se réduit).

### C-32-0-4 — La couche de services ne porte que les lectures, et pas la réconciliation

- **Contexte** : Guy décide la refactorisation vers une couche de services partagée API / MCP ; le MCP de la 0.16.0 est
  en lecture seule ; les écritures viendront avec les propositions (0.17.0). `reconciliation.rs` fait 7 016 lignes.
- **Retenu** : motif sur deux pilotes (32-3 : rapports, plan comptable), rollout mécanique des seules **lectures** que
  serviront les outils (32-4a : écritures, postes ouverts, contacts ; 32-4b : factures clientes et fournisseur). La
  réconciliation, les lots de paiement, les imports et les écritures restent dans leurs handlers.
- **Écartées** : tout déplacer en 0.16.0 (une refactorisation sans consommateur, et le plus gros risque de régression de
  l'epic) ; aucune couche de services (outils MCP appelant les dépôts directement : le contrôle d'accès vivrait à deux
  endroits, contraire à la décision de Guy).
- **Réversible** : oui ; les écritures suivront en 0.17.0.

### C-32-0-5 — La validation avant envoi passe avant les outils de données

- **Contexte** : Guy exige que les données soient montrées à l'utilisateur avant de partir à l'agent. Si les outils de
  données arrivaient d'abord, il existerait, sur `main`, un état où des données partent sans validation.
- **Retenu** : 32-8 (validation) avant 32-9a et 32-9b (outils) ; **par construction**, le répartiteur fait passer toute
  sortie d'outil de données par la validation — le trait rend une proposition d'envoi, non une réponse.
- **Écartée** : la validation en dernier, « avant la release » — un garde-fou ajouté après coup à un chemin existant se
  contourne par l'oubli.
- **Réversible** : oui.

### C-32-0-6 — Une table propre pour les clés MCP, non une colonne d'`api_keys`

- **Contexte** : Guy veut un « second type de clé dédié au MCP ». Deux formes : une colonne `kind` et une liste de portées
  sur `api_keys`, ou une table neuve.
- **Retenu** : une table neuve, un préfixe distinct (par exemple `kesh_mcp_`) ; génération, condensat et affichage
  unique **réutilisés** d'`auth/api_key.rs` (DRY). L'isolation vient de la construction : le middleware de l'API ne lit
  jamais la table MCP, l'endpoint MCP ne lit jamais `api_keys`. La table entre au `.keshbackup` (et à
  `RESET_PRESERVED_TABLES`, comme `api_keys`) : une clé restaurée reste valide.
- **Écartée** : la colonne — un défaut de filtrage dans le middleware de l'API ouvrirait `/api/v1` à une clé MCP, et la
  portée `read` existante ne recouvre pas les portées fines.
- **Réversible** : oui avant la publication de la migration ; ensuite, par une migration suivante.

### C-32-0-7 — Les portées de la 0.16.0

- **Contexte** : Guy veut des portées plus fines que `read` / `read-write`, `books:read` et `documents:read` séparées,
  jamais d'administration, et une portée dédiée à l'écriture de prompts par l'agent.
- **Retenu** : `books:read` (plan comptable, écritures, rapports, postes ouverts), `invoices:read` (factures clientes et
  fournisseur, échéances, balance âgée), `contacts:read` (données personnelles : noms, adresses, IBAN), `documents:read`
  (pièces), `prompts:write` (proposer un prompt, « à valider »). La lecture des prompts approuvés ne demande aucune
  portée (ce ne sont pas des données). Proposé à Guy (Q5).
- **Écartées** : une seule portée de lecture (l'agent de développement n'a pas besoin des contacts) ; une portée par
  outil (illisible pour un administrateur).
- **Réversible** : oui avant la publication ; une portée ajoutée ensuite ne casse rien, une portée retirée oui.

### C-32-0-8 — Les réglages du serveur MCP dans une table de société, au `.keshbackup`, restaurés tels quels

- **Contexte** : la sauvegarde doit contenir tout ce qui permet de redémarrer, sauf le `.env` ; #722 relève que le
  réglage de la sauvegarde automatique (`_kesh_backup_schedule`) en est exclu ; les réglages nouveaux se font dans
  l'interface (C-31-0-4).
- **Retenu** : activation, filtre d'adresse, plages admises et proxies de confiance dans une table de société, au
  `.keshbackup`, **désactivé par défaut**, restaurés tels quels (le manuel le dit). Proposé à Guy (Q7).
- **Écartées** : une table `_kesh_*` hors sauvegarde (aggraverait #722) ; une variable d'environnement (C-31-0-4) ;
  désactiver le serveur à toute restauration (contraire à « redémarrer les opérations », mais plus prudent : à Guy).
- **Réversible** : oui.

### C-32-0-9 — La création d'une clé MCP est réservée à l'Administrateur

- **Contexte** : un Comptable crée des PAT (17-2a, DC4) ; l'activation du MCP revient à l'administrateur (décision de
  Guy) ; une clé MCP ouvre un canal neuf vers des livres réels.
- **Retenu** : création, révocation et liste réservées à l'Administrateur, refusées à toute requête authentifiée par clé
  (comme DC6). Proposé à Guy (Q3).
- **Écartée** : le Comptable, comme pour les PAT — défendable, mais le moindre privilège l'emporte pour un canal neuf.
- **Réversible** : oui.

### C-32-0-10 — `documents:read` peut glisser en 0.17.0 ; le contenu d'une pièce selon la spike

- **Contexte** : rendre le contenu d'une pièce (PDF, image) à un agent pose deux questions que la spike tranche : la forme
  (ressource MCP, contenu binaire) et la validation avant envoi d'un document.
- **Retenu** : la 32-9b porte `contacts:read` et `documents:read` ; à défaut d'une forme éprouvée par la spike, elle livre
  les **métadonnées** seulement (liste, rattachements), et le contenu glisse en 0.17.0 sans rien toucher d'autre.
- **Écartée** : bloquer la release sur le contenu des pièces.
- **Réversible** : oui.

### C-32-0-11 — Les prompts standard s'amorcent au démarrage, non par une migration

- **Contexte** : les prompts standard sont des défauts de code copiés en base, mis à jour tant qu'ils ne sont pas
  personnalisés. Une migration qui `INSERT` relèverait de P7 (`POST_RESTORE_BACKFILLS` ou exemption justifiée), et sa
  donnée serait figée à la version de la migration.
- **Retenu** : la migration crée les tables, sans données ; l'amorçage (création si absent, nouvelle version `standard` si
  le code a changé et que la dernière version est encore `standard`) se fait au démarrage, idempotent, sur le motif
  `email_template_defaults`. Le test de triage P7 n'a rien à trier.
- **Écartée** : l'amorçage par migration (P7, et pas d'évolution avec les versions sans une migration par révision).
- **Réversible** : oui.

### C-32-0-12 — Une seule release, et les stories qui touchent `kesh-db` en série

- **Contexte** : trois stories touchent le schéma (32-5, 32-6 probablement, 32-10a) ; deux branches qui ajoutent chacune
  une migration se reportent mal l'une sur l'autre (versions horodatées, ligne d'audit d'idempotence et compteurs, P6) ;
  la prémisse 4 (inventaire des tables du `.keshbackup`) veut que le changement d'inventaire n'arrive qu'une fois.
- **Retenu** : une release v0.16.0 en fin d'epic ; 32-5 → 32-6 → 32-10a développées l'une après l'autre ; le reste en
  parallèle selon le graphe du fichier d'epic.
- **Écartées** : une 0.15.1 avec la 32-1 seule (Guy n'a pas demandé de correctif ; à réévaluer s'il en veut un) ; trois
  stories de schéma en parallèle.
- **Réversible** : oui.
