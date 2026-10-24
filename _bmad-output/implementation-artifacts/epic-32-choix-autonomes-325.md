# Epic 32 — registre des choix autonomes de la Story 32-5

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-5-cles-et-reglages-mcp.md`. Numérotation `C-32-5-k`. Les numéros
de ligne cités sont ceux de `e59d63e8`.

---

### C-32-5-1 — Une fiche, quatre parties (a, b, c, d), quatre revues — découpage proposé

- **Contexte** : le kickoff comptait **3** modules (`kesh-db`, `kesh-api` auth et routes, docs). Le relevé au code en
  trouve davantage : la règle des principaux orphelins (`companies.rs:308`) et ses trois consommateurs
  (`kesh-seed/src/lib.rs:646`, `routes/onboarding.rs:1245`, `auth/bootstrap.rs:432`), dès que la clé MCP entre à
  `RESET_PRESERVED_TABLES` (règle 4 du test, `backup.rs:1025-1036`) ; la fenêtre d'importabilité de P7, qui se referme
  (`post_restore.rs:929`) et retire le seul backfill du registre, avec quatre tests d'import qui en dépendent
  (`admin_full_import_e2e.rs:1860, 1899, 1949, 1978`) ; l'export global (`exports/global.rs:168`) ; les libellés d'audit
  (`audit_labels.rs`) et les quatre locales ; le manuel et le CHANGELOG. Compté au même grain que le kickoff (crate
  pour `kesh-db`, `kesh-seed`, `kesh-i18n` et la documentation ; module de second niveau pour `kesh-api`), le tout fait
  **onze** modules — au-delà de cinq, la règle de découpage préventif s'applique.
- **Retenu** : une seule fiche, **trois parties**, comme la 32-1 : **a** — les tables et la sauvegarde (🗄️ ; `kesh-db`,
  `kesh-api/exports`, `kesh-api/admin_backup` par ses tests, docs : 4) ; **b** — la clé MCP, principal comme une clé
  d'API (🗄️ ; `kesh-db`, `kesh-seed`, `kesh-api/auth`, `kesh-api/routes/onboarding`, `kesh-api/services` : 5) ;
  **c** — les routes de gestion (`kesh-api/routes` et `lib.rs`, `kesh-api/audit_labels`, `kesh-i18n`,
  `docs/api-external.md` : 4). Ordre a → b → c, chacune fusionnable seule (aucun état intermédiaire n'ouvre d'accès :
  sans route, aucune clé ne se crée ; sans endpoint `/mcp`, aucune clé ne sert). **Proposé** à l'orchestrateur : les
  clés `32-5a`, `32-5b`, `32-5c` au registre de sprint ; d'ici là, la clé `32-5-cles-et-reglages-mcp` porte la fiche.
- **Écartées** : une seule clé (onze modules, une revue qui ne tient pas dans un modèle mental) ; deux parties (la
  moitié « clé et routes » ferait six modules) ; déplacer les routes à la 32-7 (le kickoff les met ici, et leurs gardes
  — Administrateur seul, fermées aux clés — sont de la sécurité, non de l'écran).
- **Réversible** : oui — les parties se fusionnent sans rien réécrire.
- ⚠️ **Révisé en validation P1** (F3, F4, R4, R10) : l'ancienne partie b gagnait un quatrième consommateur
  d'`OrphanPrincipals`, `tests/onboarding_audit_e2e.rs`, `MULTI-TENANT-SCOPING-PATTERNS.md` et deux passages du manuel —
  **six** modules. Elle se scinde : **b** — l'authentification (`kesh-api/auth`, `kesh-api/services`, le test de
  restauration : 3) ; **d** — la remise à zéro et la réparation (`kesh-db`, `kesh-seed`, `kesh-api/auth`
  (`bootstrap.rs`), `kesh-api/routes` (`onboarding.rs`, `admin.rs`), docs : 5). Total recompté : douze modules ; ordre
  a → b, puis c et d dans l'un ou l'autre ordre. Jusqu'à d, la remise à zéro vide `mcp_keys` et l'authentification joint
  `companies` : aucun état intermédiaire n'ouvre d'accès ni ne fait mentir le manuel.

### C-32-5-2 — Deux tables neuves dans une seule migration

- **Contexte** : C-32-0-6 (table propre aux clés) et C-32-0-8 (réglages dans une table de société, au `.keshbackup`).
  Chaque table applicative neuve change l'inventaire du `.keshbackup` (`import.rs:234-250`) et referme la fenêtre de P7.
- **Retenu** : `20261025000001_mcp_keys_and_server_settings.sql`, deux `CREATE TABLE` (`mcp_keys`,
  `mcp_server_settings`), **aucune donnée écrite** : l'inventaire change une fois, la fenêtre se referme une fois, P7
  n'a rien à trier.
- **Écartées** : deux migrations (deux fermetures de fenêtre, deux lignes d'audit, aucun gain) ; les réglages dans une
  table système `_kesh_*` (hors sauvegarde : aggraverait #722).
- **Réversible** : oui avant publication ; ensuite, par une migration suivante.

### C-32-5-3 — Les portées en colonne `JSON` (tableau), validées par Rust

- **Contexte** : C-32-0-7 (cinq portées) ; une clé en porte une liste. Le dépôt a un précédent de colonne `JSON` avec
  `CHECK (JSON_VALID(…))` (`20260505000001_bank_profiles.sql:27, 49`) ; `decode_cell` exporte le JSON en chaîne
  (`backup.rs:307-316`) et l'import la rebinde.
- **Retenu** : `scopes JSON NOT NULL`, `CHECK` : JSON valide, tableau, au moins un élément ; tri et unicité imposés à
  l'écriture par Rust ; décodage **strict** (une portée inconnue est une erreur, jamais ignorée en silence).
- **Écartées** : `SET(...)` MariaDB (membres contrôlés par le type, mais le rejet d'un membre inconnu dépend du
  `sql_mode`, que la production n'applique pas — #669) ; une table fille (une table de plus à l'inventaire, une
  jointure à chaque authentification) ; une chaîne à virgules (format ad hoc).
- **Réversible** : oui avant publication.

### C-32-5-4 — Une seule énumération des portées : `kesh_db::entities::McpScope`, réexportée en `Portee`

- **Contexte** : `services::acces::Portee` (32-3) porte `BooksRead` et `InvoicesRead` (`acces.rs:20-27`) et « la 32-5
  ajoute les autres et leur persistance » (C-32-3-4) ; `kesh-db` ne peut pas dépendre de `kesh-api`, et le décodage de
  la colonne exige l'énumération côté `kesh-db` (motif `ApiKeyScope`, `entities/api_key.rs:22-71`).
- **Retenu** : l'énumération vit dans `kesh-db` (`McpScope`, cinq variantes aux **mêmes noms** que `Portee`, `as_str`,
  `FromStr`, dérivations `Ord` et `Hash` comprises) ; `services::acces` la réexporte (`pub use … McpScope as Portee`).
  Les emplois de `Portee::` hors de `tests/services_motif.rs` (quatorze occurrences sur onze lignes en tout) compilent inchangés ; *révisé en validation P1 (R2 = F1, F7)* : `services_motif.rs` énumère les deux variantes (`TOUTES: [Portee; 2]`, l. 869 ; `match` exhaustif d'`autre_portee`, l. 894-898) et se réécrit (AC-b2).
- **Écartées** : deux énumérations et une conversion (deux listes à tenir égales — DRY) ; l'énumération dans
  `kesh-api` et des chaînes en `kesh-db` (le décodage strict quitterait le dépôt).
- **Réversible** : oui.

### C-32-5-5 — La politique de validation avant envoi, par clé, en colonne

- **Contexte** : Q12 (par clé, `requise` par défaut, `dispensée` pour une clé que l'administrateur désigne, chaque envoi
  journalisé) ; Q10 (c), la poignée, appliquée par les stories suivantes.
- **Retenu** : `send_validation VARCHAR(16) NOT NULL DEFAULT 'required'`, `CHECK IN ('required', 'exempt')` ; posée à
  la création (défaut `required`) et **modifiable** ensuite par une route d'administration, tracée. La 32-5 la
  **stocke et la rend** à l'authentification ; elle ne l'applique pas : le répartiteur de la 32-6 (refus d'une clé
  `required` tant que la 32-8 n'est pas livrée, spike § 6.2, point 6), la poignée et sa table de la 32-8, le journal
  de chaque envoi (32-6 pour la trace, 32-8 pour l'envoi validé).
- **Écartées** : un booléen (moins lisible au journal et à l'export, et ferme la porte à une troisième valeur) ; la
  politique fixée à la création seulement (l'administrateur devrait réémettre la clé pour passer de l'un à l'autre).
- **Réversible** : oui.

### C-32-5-6 — Les portées d'une clé sont immuables ; on réémet

- **Contexte** : les PAT n'ont pas de route de modification de portée (`routes/api_keys.rs` : liste, création,
  révocation).
- **Retenu** : nom, portées et expiration fixés à la création ; seule la politique de validation (C-32-5-5) change
  ensuite. Élargir une portée = créer une clé neuve et révoquer l'ancienne : le secret qu'un client détient ne gagne
  jamais de droit sans un geste qui lui donne un secret neuf.
- **Écartée** : une route `PATCH` des portées (élargirait en silence les droits d'un secret déjà distribué).
- **Réversible** : oui (une route s'ajoute).

### C-32-5-7 — Une clé MCP exige un créateur actif et toujours Administrateur (hypothèse Q3)

- **Contexte** : Q3 sans réponse ; recommandation du kickoff (C-32-0-9) : création réservée à l'Administrateur. Un PAT
  relit le rôle et l'état **courants** de son créateur (`api_keys.rs:123-141`, `auth/api_key.rs:163-167`).
- **Retenu** : une constante `ROLE_CLE_MCP = Role::Admin`, seule source de deux règles : la création (route sous
  `admin_routes`) et l'authentification (créateur `active` **et** `role >= ROLE_CLE_MCP`, relus à chaque requête). Un
  administrateur rétrogradé ou désactivé coupe ses clés MCP sur-le-champ. **Hypothèse de travail**, écrite comme telle
  à la fiche ; si Guy ouvre la création au Comptable, la constante change et les deux règles suivent.
- **Écartées** : le rôle relu seulement à la création (une clé survivrait à la rétrogradation de son créateur) ; aucune
  règle de rôle à l'authentification (incohérent avec une création réservée).
- **Réversible** : oui (une constante).

### C-32-5-8 — Réglages absents = défauts ; aucune ligne écrite ni par la migration ni à la lecture

- **Contexte** : `company_invoice_settings::get_or_create_default` écrit sur un chemin de lecture (spike § 4.4) ; un
  `INSERT` de migration relèverait de P7 ; la 32-6 lira les réglages sur la `Lecture`, peut-être en lecture seule.
- **Retenu** : aucune ligne n'existe tant que l'administrateur n'a rien enregistré ; la lecture rend les **défauts du
  code** (`McpServerSettings::defaults(company_id)`) ; l'écriture est un `PUT` à verrou optimiste où `version = 0`
  désigne « aucune ligne » (`INSERT`, 409 si une ligne est apparue) et `version ≥ 1` un `UPDATE … WHERE version = ?`.
- **Écartées** : `get_or_create` (une écriture sur la lecture, qui échouerait sur la `Lecture` de la 32-6) ; une ligne
  par migration (P7, et une ligne figée aux défauts de cette version).
- **Réversible** : oui.

### C-32-5-9 — Ce que portent les réglages : activation, filtre, plages, mandataires, origines

- **Contexte** : C-32-0-8 (activé ou non, filtre activé ou non, plages admises, proxies de confiance) ; la spike § 6
  (32-6) : « origines du réseau local en liste réglable et jamais déduites de `Host` ». Une liste réglable vit en base.
- **Retenu** : `enabled` (défaut **faux**), `address_filter_enabled` (défaut vrai), `allowed_networks`,
  `trusted_proxies`, `allowed_origins` (trois tableaux JSON de chaînes canoniques, validés à l'écriture). Les origines
  entrent **ici**, pour épargner à la 32-6 une migration et une seconde fermeture de la fenêtre de P7. La 32-5 stocke et
  valide la **forme** ; leur **sens** (le filtre, la lecture de `X-Forwarded-For`, la garde `Origin`, le 404) est
  celui de la 32-6.
- **Écartées** : les origines à la 32-6 (une migration de plus) ; des réglages dans le `.env` (C-31-0-4).
- **Réversible** : oui avant publication.

### C-32-5-10 — Les plages admises par défaut : boucle locale et réseaux privés, sans la plage partagée des VPN

- **Contexte** : le filtre est un complément « de défense en profondeur, désactivable » (décisions du 2026-10-24) ; Guy
  accède de l'extérieur par VPN. Certains VPN (Tailscale, par exemple) adressent leurs clients dans `100.64.0.0/10`.
- **Retenu** : `127.0.0.0/8`, `::1/128`, `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`, `fc00::/7`, `fe80::/10`.
  `100.64.0.0/10` **n'y est pas** : c'est aussi l'espace des opérateurs (CGNAT), et l'ouvrir par défaut admettrait
  plus que le réseau local. L'administrateur l'ajoute s'il en a besoin ; la 32-7 le dit à l'écran et au manuel.
  Soumis à Guy (*Points à soumettre*).
- **Écartées** : une liste vide qui signifierait « privé » (règle implicite, invisible à l'écran) ; inclure la plage
  partagée.
- **Réversible** : oui (une constante ; seules les installations sans réglage enregistré suivent le changement).

### C-32-5-11 — Restaurés tels quels ; à la remise à zéro, clés conservées, réglages remis aux défauts

- **Contexte** : Q7 sans réponse (recommandation : tels quels) ; `RESET_PRESERVED_TABLES` conserve `api_keys`, « un
  principal n'est pas une donnée de la société » (`backup.rs:102-118`).
- **Retenu** : (1) un import restaure les deux tables **telles quelles** (hypothèse Q7) — un serveur activé dans la
  sauvegarde l'est après l'import ; (2) la remise à zéro de démonstration **conserve** `mcp_keys` (consigne de
  l'orchestrateur, comme `api_keys`) et **vide** `mcp_server_settings` (une ligne de réglages de société, comme
  `company_invoice_settings`) : après une remise à zéro, le serveur est désactivé et les clés demeurent.
- **Écartées** : désactiver le serveur à toute restauration (plus prudent, contraire à « redémarrer les opérations » :
  à Guy, Q7) ; conserver aussi les réglages à la remise à zéro (une démonstration réinitialisée garderait un serveur
  ouvert).
- **Réversible** : oui.

### C-32-5-12 — La règle des principaux orphelins s'étend aux clés MCP

- **Contexte** : la règle 4 du test `reset_partition_is_guarded_by_the_schema` (`backup.rs:1025-1036`) exige que les
  tables conservées portant une clé vers `companies` soient **exactement** `users` et `api_keys`, « le fait sur lequel
  repose la règle des principaux orphelins » ; conserver `mcp_keys` la fait rougir. La règle
  (`companies.rs:278-311`) : aucune clé orpheline ne redevient active par aucun chemin.
- **Retenu** : `reattach_orphan_principals_in_tx` verrouille, révoque puis repointe aussi les clés MCP orphelines,
  après `api_keys` (ordre des verrous `users` → `api_keys` → `mcp_keys`) ; `OrphanPrincipals` gagne
  `mcp_keys_revoked` et `mcp_keys_repointed`, que ses **quatre** consommateurs (*révisé en validation P1, R4 = F3* : `repair_installation_in_tx` lui-même, `companies.rs:680-683`, s'ajoute aux trois premiers) écrivent **toujours** au détail de leur entrée d'audit (`[]` et `0` quand il n'y a rien : forme stable, tests à égalité JSON complétés) ;
  la règle 4 attend `{api_keys, mcp_keys, users}`. Défense de plus, sans coût : l'authentification d'une clé MCP joint
  `companies` — une clé orpheline n'authentifie jamais, même avant la réparation.
- **Écartées** : mêler les clés MCP aux `api_keys_revoked` (une seule liste, deux natures de secret, et un JSON d'audit
  dont le sens change en silence) ; vider `mcp_keys` à la remise à zéro pour éviter la règle (contraire à la consigne et
  au statut de principal).
- **Réversible** : oui.

### C-32-5-13 — La fenêtre d'importabilité se referme : le rejeu du lettrage (`20261010000001`) se retire

- **Contexte** : `registry_entries_are_within_import_window` (`post_restore.rs:929-940`) exige que chaque entrée du
  registre soit postérieure à la dernière migration créatrice de table applicative ; la 32-5 en crée deux, en
  `20261025000001`. L'unique entrée, `20261010000001` (`post_restore.rs:225-254`), devient antérieure : son cas de
  déclenchement — une sauvegarde d'avant la 0.13.0 — est refusé en 400 par `parse_and_verify` bien avant le rejeu.
  Q4 répondue : on l'accepte et on l'écrit.
- **Retenu** : l'entrée passe à `RETIRED_BACKFILLS` (sa machinerie reste éprouvée par `post_restore_class_a.rs:83-89`),
  son exemption s'écrit en tête « Hors fenêtre depuis 20261025000001 (mcp_keys, Story 32-5) » ; compteurs codés en dur
  8 → 9 (`post_restore.rs:1128`) et 20 → 21 (`:1228`), chacun commenté ; les quatre tests de l'AC16 de la 15-1a2-ii
  passent au registre retiré (`replay_retired`, motif de la 24-2, `admin_full_import_e2e.rs:943-969`) ;
  `full_import_report_mirrors_the_production_registry` attend de nouveau un registre vide ; le manuel
  (§ *Reprises de données rejouées à l'import*) dit qu'à partir de la 0.16.0 aucune reprise n'est rejouée.
- **Écartées** : garder l'entrée (le test rougit, et ce serait du code mort exécuté à chaque import) ; supprimer les
  quatre tests (la machinerie redeviendra active à la prochaine migration de backfill, cf. le commentaire de
  `replay_retired`).
- **Réversible** : non au sens strict — c'est la conséquence mécanique d'une table neuve ; le rejeu redeviendrait
  joignable seulement avec un import tolérant (la 32-12, retirée).

### C-32-5-14 — L'authentification d'une clé MCP est un service : lecture sur la `Lecture`, dernière utilisation sur la `Trace`

- **Contexte** : `validate_pat` lit et écrit sur `&state.pool` (`auth/api_key.rs:153-190`) ; la spike § 4.4 et le
  module `services` (`services/mod.rs:80-83`) prévoient que la clé MCP écrive sa dernière utilisation sur la `Trace` ;
  la 32-6 lira peut-être par une `Lecture` en session `READ ONLY` (Q11 : lecture seule applicative).
- **Retenu** : `services::cles_mcp::authentifier(&Lecture, &Trace, jeton) -> Result<CleMcpAuthentifiee, AppError>` :
  préfixe exact, condensat, recherche de la clé active (jointe à `users` et `companies`) sur la `Lecture`, contrôle du
  créateur (C-32-5-7), dernière utilisation *best-effort* sur la `Trace` ; rend le `ContexteAcces` MCP (société,
  créateur, portées), l'identifiant de la clé et sa politique de validation. **Aucun branchement** : la 32-6 l'appelle
  depuis sa garde ; le middleware de `/api/v1` ne la connaît pas.
- **Écartées** : un module `auth/mcp_key.rs` hors services (la `Trace` n'y est pas lisible, `poignees.rs:49`) ; brancher
  la clé MCP dans `require_auth` (C-32-0-6 : l'API ne lit jamais la table MCP).
- **Réversible** : oui.

### C-32-5-15 — Génération et condensat partagés : un préfixe en paramètre

- **Contexte** : `generate_pat` et `generate_reset_token` partagent déjà l'entropie et l'encodage
  (`auth/api_key.rs:113-140`).
- **Retenu** : une fonction privée `generer_avec_prefixe(prefixe)` (160 bits `OsRng`, `base62_fixed_width`,
  `sha256_hex`), appelée par `generate_pat` et par `generate_mcp_key` ; `MCP_PREFIX = "kesh_mcp_"` à côté de
  `PAT_PREFIX`, avec un test qu'aucun des deux préfixes n'est préfixe de l'autre.
- **Écartée** : une copie de `generate_pat` (DRY).
- **Réversible** : oui.

### C-32-5-16 — Aucun écran en 32-5, donc aucune recette visuelle

- **Contexte** : le kickoff confie l'écran « Serveur MCP » à la 32-7 (activation, filtre, clés, affichage unique) ; la
  consigne demande de décider si l'écran est au périmètre.
- **Retenu** : hors périmètre. La 32-5 livre les routes ; la 32-7 l'écran, ses captures et la section du manuel. Aucun
  contrôle visible ne change : la règle de recette visuelle ne s'applique pas, et la fiche le dit.
- **Écartée** : l'écran ici (frontend et i18n de l'écran : deux modules de plus, au-delà du plafond).
- **Réversible** : oui.

### C-32-5-17 — Les routes sous `admin_routes` : Administrateur, et fermées aux clés

- **Contexte** : `admin_routes` porte `require_admin_role` puis `require_not_pat` (`lib.rs:393-400`) ; le test
  `admin_pat_denied_e2e` borne ce bloc entre ses marqueurs (`lib.rs:221`, `:402`). Les routes des PAT sont ailleurs,
  gardées Comptable + `ensure_not_pat` (`lib.rs:803-813`, `routes/api_keys.rs:95-100`).
- **Retenu** : `GET`/`POST /api/v1/admin/mcp/keys`, `DELETE /api/v1/admin/mcp/keys/{id}`,
  `PUT /api/v1/admin/mcp/keys/{id}/send-validation`, `GET`/`PUT /api/v1/admin/mcp/settings`, toutes dans le bloc : un
  PAT reçoit `403 API_KEY_ADMIN_FORBIDDEN`, un Comptable `403 FORBIDDEN`, une clé MCP `401` (le middleware ne la
  reconnaît pas).
- **Écartée** : `/api/v1/settings/mcp-keys` sur le motif des PAT (Comptable, contraire à l'hypothèse Q3).
- **Réversible** : oui.

### C-32-5-18 — Quatre actions et deux types d'entité au journal d'audit

- **Contexte** : `api_key.created` / `api_key.revoked` (`routes/api_keys.rs:171-184, 230-243`) ; toute action écrite
  par la production a son libellé (`audit_labels.rs:79-80`) dans les quatre locales, sous la garde
  `tests/audit_label_registry.rs`.
- **Retenu** : `mcp_key.created`, `mcp_key.revoked`, `mcp_key.send_validation_changed`, `mcp_server_settings.updated` ;
  types `mcp_key`, `mcp_server_settings` ; détails sans secret ni condensat (nom, portées, politique ; avant/après pour
  les deux modifications).
- **Écartée** : réutiliser `api_key.*` (deux natures de clé confondues au journal).
- **Réversible** : non une fois publié (codes stockés) ; oui avant.

### C-32-5-19 — Plages et mandataires validés avec `ipnet`, déjà dans l'arbre

- **Contexte** : aucune dépendance directe de Kesh ne lit une notation CIDR ; `ipnet 2.12.0` est déjà dans `Cargo.lock`
  (l. 1872-1873), par dépendance transitive.
- **Retenu** : `ipnet` en dépendance directe de `kesh-api` (même version, aucun téléchargement), pour valider et
  **canoniser** chaque entrée à l'écriture (*révisé en validation P1, R7* : lue d'abord comme `IpAddr`, stockée `/32` ou `/128` ; sinon comme `IpNet`, dont `from_str` exige le préfixe ; un réseau à bits d'hôte, `10.1.2.3/8`, est **refusé** plutôt que tronqué en silence — l'administrateur voit la forme attendue) ; la 32-6 s'en sert
  pour l'appartenance. Origines : `scheme://hôte[:port]`, `http` ou `https`, sans chemin, comparées en minuscules.
- **Écartée** : un analyseur maison (une seconde implémentation d'une notation normée).
- **Réversible** : oui.

### C-32-5-20 — L'inventaire passe de 39 à 41 tables ; les deux tables hors de l'export global

- **Contexte** : `admin_full_export_e2e.rs:282` compte 39 fichiers de données ; `TABLES_HORS_EXPORT`
  (`exports/global.rs:168`) doit motiver toute table de l'inventaire non exportée, sous la garde de `:637-690`.
- **Retenu** : `mcp_keys` hors export, motif « SECRET » (patron d'`api_keys`, `:183-188`) ; `mcp_server_settings` hors
  export, motif « réglage d'installation, sans valeur comptable » ; le compte de l'export d'installation passe à 41 en
  **nommant** les deux tables ; le commentaire « les 39 tables » (`:167`) suit.
- **Écartée** : exporter les réglages au CSV (aucun usage pour un autre logiciel).
- **Réversible** : oui.

### C-32-5-21 — La limite de Q4 s'écrit à trois endroits du manuel et au CHANGELOG

- **Contexte** : Q4 (2026-10-24, soir) — documenter « au cas où » qu'une sauvegarde ne se réimporte que dans une version
  qui porte les mêmes tables. Le manuel dit aujourd'hui qu'une sauvegarde antérieure « retrouve, à l'import, le
  lettrage » (`admin-manual.tex:2321`) et que le `.keshbackup` porte « les condensés des clés API (PAT) »
  (`:2230`).
- **Retenu** : (1) § *Ce que la sauvegarde emporte*, paragraphe **Compatibilité** : la règle, la marche à suivre
  (installer l'image de la version qui a produit la sauvegarde, importer, puis mettre à jour), le message de refus ; (2)
  § *Reprises de données rejouées à l'import* : plus aucune reprise rejouée depuis la 0.16.0, et pourquoi ; (3)
  l'avertissement « secret » : les condensés des clés MCP ; CHANGELOG `[0.16.0]`, un ⚠️ en tête de *Modifié*. PDF
  régénéré.
- **Écartée** : une note au CHANGELOG seul (le manuel est ce qu'on lit le jour de la panne).
- **Réversible** : oui.

### C-32-5-22 — La partie qui change le comportement corrige le manuel (F4)

- **Contexte** : la 32-5d fait conserver les clés MCP par la remise à zéro et révoquer les clés MCP orphelines par la
  réparation ; le manuel dit aujourd'hui que la remise à zéro « conserve les utilisateurs et les clés d'API »
  (`admin-manual.tex:1414`) et que la réparation révoque « les clés d'API » (`:1416`, `:2461`). Entre la 32-5d et la
  32-7, il mentirait par omission — sur un point de sécurité (une clé de démonstration qui garde l'accès aux livres de
  production).
- **Retenu** : la 32-5d corrige les trois passages et régénère le PDF dans sa PR (*Règle d'inclusion* de `CLAUDE.md`) ;
  la 32-7, qui écrit la section *Le serveur MCP*, y renvoie (point de suite inscrit à la fiche, pour la section 32-7 du
  fichier d'epic).
- **Écartée** : laisser la 32-7 reprendre les deux passages, la 32-5d le disant — moins de modules, mais un manuel faux
  sur `main` pendant plusieurs stories, et une règle d'inclusion contournée.
- **Réversible** : oui.

### C-32-5-23 — Où vivent les tests : là où le montage existe

- **Contexte** : `kesh-seed` n'a aucun test de base et ne dépend pas de `kesh-api` (R3) ; les helpers d'export et
  d'import sont privés à `admin_full_import_e2e.rs` (R8 = F8).
- **Retenu** : le test de la remise à zéro dans `kesh-api/tests/onboarding_audit_e2e.rs` (où vivent ceux de
  `reset_demo`) ; le test de la clé restaurée dans `admin_full_import_e2e.rs` (montage réutilisé, `DELETE FROM mcp_keys`
  direct) ; les refus croisés dans un `cles_mcp_e2e.rs` neuf (montage de `tests/common/`).
- **Écartée** : extraire les helpers d'import vers `tests/common/` (un déplacement de 800 lignes hors du sujet).
- **Réversible** : oui.

### C-32-5-24 — La ligne de `migrations.sha384` dans la même PR, en dernier (R1)

- **Contexte** : `published_migrations_keep_their_checksums` (`test_schema_guard.rs:602`) exige une ligne par migration ;
  l'en-tête du registre dit qu'elle s'ajoute à la création ; la 31-1c l'a fait dans sa PR. La première rédaction
  affirmait le contraire.
- **Retenu** : la 32-5a inscrit `20261025000001 <sha384>` une fois les `CHECK` éprouvés ; dès lors le fichier de
  migration ne se modifie plus (P8).
- **Écartée** : attendre la release (gate rouge).
- **Réversible** : non (c'est l'objet même du registre).

### C-32-5-25 — Cohérence avec #722 : la 32-5 n'ajoute rien hors sauvegarde

- **Contexte** : #722 demande l'inventaire clos de ce qui vit hors du `.keshbackup`, réglages de l'Epic 32 compris
  (prompts, activation du serveur, clés MCP), et, pour chacun, de l'emporter ou de dire pourquoi, au manuel
  § restauration.
- **Retenu** : les clés MCP et les réglages du serveur — activation comprise — **entrent** au `.keshbackup` (C-32-0-8,
  C-32-5-2) ; le manuel le dit à *Ce que la sauvegarde emporte* (AC-a13 (4)) ; la liste de ce qui reste hors du fichier
  ne change pas. La 32-5 ne ferme pas #722 (`_kesh_backup_schedule` reste à part ; les prompts sont la 32-10a) et le
  dit en `refs`.
- **Écartée** : traiter ici l'inventaire entier de #722 (hors périmètre de la story).
- **Réversible** : oui.
