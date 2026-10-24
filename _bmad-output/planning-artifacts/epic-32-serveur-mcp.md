# Epic 32 — Le serveur MCP de Kesh : un agent lit les livres, en lecture seule

**Kickoff : 2026-10-24**, en autonomie (Guy absent jusqu'en fin d'après-midi) : ce fichier **verse** les décisions du
Project Lead, il n'en prend aucune ; ce qui reste à trancher est au § *Questions pour Guy*, chaque fois avec la
recommandation de l'équipe. Les choix de découpage sont au registre `epic-32-choix-autonomes.md` (C-32-0-1 à 12).

**Cible : v0.16.0.** La v0.15.0 est publiée (2026-10-24, `gcorbaz/kesh:0.15.0` = `:latest`, `sha256:77cfb96d…`) ; la
v0.14.0 reste en fonction sur le NAS jusqu'à sa mise à jour (action A5 de la rétrospective de l'Epic 31, Guy). Guy y
tient sa **comptabilité réelle** depuis le 2026-10-23 — c'est la première fois qu'un epic ajoute une surface d'accès à
des livres réels.

Sources : décisions de Guy du 2026-10-24 (`memory/serveur-mcp-decisions.md`, note `~/travail/Projets actuels/kesh/notes/
MCP serveur.md`, compléments après la table ronde Winston / John / Murat / Mary du même jour) ; rétrospective de l'Epic
31 (`implementation-artifacts/epic-31-retro-2026-10-24.md`, § 8 à 10) ; relevé du code au kickoff (§ *Ce que le code
dit déjà*).

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. **Aucune issue ne porte encore
le serveur MCP lui-même** (#100, l'API à clés, est fermée ; #720 et #722 sont des CR voisins) : l'ouverture d'un CR
« serveur MCP intégré, lecture seule (0.16.0) » est demandée à l'orchestrateur (« on trace toujours »). L'epic se clôt
quand les stories ci-dessous sont `done` et la v0.16.0 publiée.

---

## Décisions du Project Lead (2026-10-24)

**Le périmètre de la 0.16.0** :

- Un serveur MCP **intégré à Kesh**, en plus de l'API, pour « n'importe quelle IA » : endpoint **`/mcp` dans le même
  processus Axum**, transport **Streamable HTTP** ; protocole **écrit à la main** derrière une couche de traits
  (`McpTool`, `McpPrompt`), un SDK pris plus tard, quand il sera mûr.
- **Désactivé par défaut**, activable et désactivable par l'administrateur.
- **Réseau local seulement** (clients locaux : Claude Code, Hermes…) ; Kesh n'est **jamais exposé sur Internet**, donc
  **pas d'OAuth**. La sécurité réseau est l'affaire de l'infrastructure (NAS non exposé, accès externe par VPN, une partie
  des conteneurs derrière Traefik) ; Kesh garde un **filtre d'adresse de défense en profondeur, désactivable**, décrit
  comme un complément.
- Authentification par un **second type de clé, dédié au MCP**, à **portées fines** (`books:read` et `documents:read`
  séparées…), **jamais de portée d'administration**.
- **Lecture seule sur les livres, prouvée par construction** (connexion en lecture seule).
- **Une couche de services partagée** par l'API et le MCP, le contrôle d'accès **dans les services**.
- **Prompts MCP en base**, avec **historique**, éditables sans recompiler ; l'administrateur et l'agent en écrivent —
  l'agent par une portée dédiée, son prompt **marqué « écrit par un agent »** et **« à valider »** jusqu'à
  l'approbation de l'administrateur ; des **prompts standard installés d'office**, servant d'exemples, qui **évoluent
  avec les versions s'ils n'ont pas été personnalisés** ; les **prompts dans la sauvegarde**.
- **Clés restaurées valides** après une restauration.
- **Avant d'envoyer des données à l'agent, les montrer à l'utilisateur** et lui demander s'il les envoie telles quelles
  ou les modifie (mécanisme à concevoir : élicitation MCP côté client, ou écran de Kesh).
- L'**agent de développement** lit les livres réels en lecture seule ; l'outil de **diagnostic structurel** est
  **repoussé** (l'accès aux journaux suffit pour commencer).
- Données chez le fournisseur du modèle **acceptées** ; un modèle local est envisagé à terme.

**Règles de l'agent sur les données réelles** : jamais de montant, nom, IBAN ni libellé réel hors de Kesh ; un défaut se
décrit par sa structure et se reproduit sur la démonstration ; **en cas de doute, il demande**. Elles entrent dans le
prompt standard de l'agent de développement (32-10a).

**La sauvegarde** doit contenir tout ce qui permet de redémarrer les opérations, **sauf le `.env`**, propre à
l'installation, qui à terme ne portera que l'accès à la base. Les tables neuves de cet epic entrent donc au
`.keshbackup` ; les réglages nouveaux se font **dans l'interface** (C-31-0-4).

**Règle de release, reformulée par Guy le 2026-10-24** : moins d'issues ouvertes au tag qu'au tag précédent est **un
objectif à atteindre à terme, pas une barrière** ; le dépasser n'est pas critique ; corriger un bug dans un epic n'est
pas interdit (`CLAUDE.md` § *Priorités des défauts*).

---

## Hors périmètre, et où cela va

| Sujet | Où |
|---|---|
| **Propositions** de l'agent, acceptées, modifiées ou refusées dans un écran (jamais d'écriture directe pour commencer — `CLAUDE.md` § *Un appariement automatique propose, il ne crée jamais*) | **0.17.0** |
| [#235] — pièce justificative sur une écriture manuelle (une écriture de correction peut s'en passer), P1 | **0.17.0** |
| Extraction fiscale (marquage déductible seulement, rubriques de Vaud configurables, export et import de la configuration) | **0.18.0**, pas avant le début 2027 |
| Budgets [#403], [#197] | CR, epic dédié |
| Refacturation d'énergie [#273] | CR, epic dédié |
| Évolution pluriannuelle [#164], [#404] (P1) | CR, epic dédié |
| Ouvrir une comptabilité à partir de pièces, assistée par l'agent [#720] | CR, epic dédié (demande l'écriture : après 0.17.0) |
| Ce qui manque encore à un redémarrage complet [#722] (réglages de la sauvegarde automatique, `_kesh_backup_schedule`, hors `.keshbackup`) | hors epic — **cet epic n'y ajoute rien** : ses réglages entrent au `.keshbackup` (C-32-0-8) |
| Outil de diagnostic structurel pour l'agent de développement | repoussé (décision de Guy) |
| Toute écriture par le MCP, toute portée d'administration | exclu de la 0.16.0 |
| [#709] (critère OLICo), les deux choix de procédure de [#714] | décision de Guy (rétrospective 31, Q4) |
| [#671], [#695], [#712], [#710], [#708], [#705], [#699] | backlog, inchangés |

---

## Le décompte, et l'objectif de la 0.16.0

```sh
gh issue list --state open --limit 500 --json number --jq length
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
```

Exécutées le 2026-10-24 pour ce kickoff (lecture seule).

| Moment | Issues ouvertes | dont bugs |
|---|---|---|
| Tag v0.15.0 (2026-10-24, 11:02) | **140** | 51 |
| Rétrospective de l'Epic 31 | 141 (+#720) | 51 |
| **Kickoff de l'Epic 32** (2026-10-24) | **142** (+#722, action A4 de la rétrospective) | **51** |
| **Objectif au tag v0.16.0** | **moins de 140**, à la lettre de la règle — ou moins de 136 si Guy le préfère (Q1) | en baisse |

**Ce que l'epic peut y faire, sans fermer pour le chiffre** : la 32-1 vise [#716] et [#711] (fermées si leurs constats
sont tous soldés ou reportés dans une issue plus étroite) et réduit [#714] à ses deux choix de procédure — de −2 à −3. Le
CR du serveur MCP, s'il est ouvert, ajoute une issue. L'objectif est donc **serré** ; c'est un objectif, pas une
barrière (§ *Décisions*).

**Engagements de l'epic** (`CLAUDE.md` § *Priorités des défauts*) :

1. **Le nombre de bugs baisse** — 51 au kickoff, deux epics de suite sans baisse (Epics 30 et 31) : la 32-1 ferme
   [#711] (bug) et, si Guy tranche ses deux choix, [#714] (bug).
2. **Aucun P1 ne passe hors TVA, dans la mesure du possible** — restent, nommés : [#235] (**0.17.0**, décision de Guy),
   [#404] (epic pluriannuel dédié), [#98] (`v0.2-milestone`, inchangé) ; les dix P1 de la TVA restent en **report
   assumé** (décision du 2026-10-08).
3. **Une fonctionnalité nouvelle** : le serveur MCP.

---

## Ce que le code dit déjà (relevé au kickoff, `ce6b3188`)

Chaque point est une **prémisse** que la fiche concernée revérifie au code avant de concevoir.

1. **Les clés d'API existantes** (`crates/kesh-api/src/auth/api_key.rs`, `routes/api_keys.rs`, table `api_keys`) :
   préfixe `kesh_pat_`, 160 bits, **seul le condensat SHA-256 est stocké** (index unique), affichage unique à la
   création, révocation, expiration, `last_used_at` ; deux portées, `read` et `read-write`
   (`kesh_db::entities::ApiKeyScope`). La gestion est refusée à une requête authentifiée par clé (DC6) ;
   l'administration est fermée aux clés (`API_KEY_ADMIN_FORBIDDEN`, `middleware/rbac.rs:64`). **Tout cela se réutilise**
   (génération, condensat, affichage unique) ; la table, non (C-32-0-6).
2. **La garde « lecture = GET »** : `middleware/auth.rs:141` refuse une méthode non sûre à une clé `read`
   (`is_safe_method`, l. 172). **MCP est du JSON-RPC sur `POST`** : cette garde ne protège rien sur `/mcp`, et le serveur
   MCP doit avoir **sa propre garde**, par outil et par portée.
3. **`api_keys` est déjà au `.keshbackup`** (`crates/kesh-db/src/backup.rs`, `TABLES_TO_TRUNCATE`) et préservée par la
   remise à zéro (`RESET_PRESERVED_TABLES`) : une clé restaurée reste valide, puisque son condensat revient. Les tables
   neuves de cet epic suivent le même chemin ; `backup_inventory_matches_schema` rougit si l'une manque.
4. ⚠️ **Une table neuve rend les sauvegardes antérieures non importables par la nouvelle version** :
   `parse_and_verify` (`crates/kesh-api/src/admin_backup/import.rs`, l. 172 et suivantes) exige que chaque table de
   `TABLES_TO_TRUNCATE` figure au manifeste, et réciproquement (`CLAUDE.md`, P7 : « un backup n'est importable que si son
   inventaire de tables est identique au nôtre »). La v0.15.0 n'a pas changé l'inventaire (`_kesh_backup_schedule` en est
   exclu par son préfixe) ; **la v0.16.0 le changera**. Conséquence pour des livres réels : après la mise à jour, les
   sauvegardes — automatiques comprises — prises en 0.15.0 ne se restaurent plus que par une image 0.15.0. À écrire au
   CHANGELOG et au manuel, et à soumettre à Guy (Q4).
5. **Les handlers de lecture sont minces** : `list_accounts` (`routes/accounts.rs:188`), `list_supplier_invoices`
   (`routes/supplier_invoices.rs:551`), `get_balance_sheet` (`routes/reports.rs:215`) valident la requête, résolvent la
   société et la période, puis appellent un dépôt de `kesh-db` ou un générateur de `kesh-report`. **La « couche de
   services » est donc surtout une extraction** — validation, contrôle d'accès, appel, mise en forme — et non une
   réécriture de la logique comptable. Taille des modules de routes concernés : `accounts` 384 lignes, `journal_entries`
   1 081, `invoices` 1 827, `supplier_invoices` 884, `contacts` 1 000, `reports` 1 648 ; `reconciliation` (7 016) n'est
   **pas** au périmètre de lecture de la 0.16.0 (C-32-0-4).
6. ⚠️ **Des lectures écrivent** : chaque rapport écrit une ligne au journal d'audit (`emit_report_audit`,
   `routes/reports.rs:1407`), et l'authentification par clé met à jour `last_used_at` (`api_keys::touch_last_used`). Une
   connexion en lecture seule ferait donc **échouer** un rapport tel qu'il est écrit : la couche de services sépare
   **la lecture des données** (connexion en lecture seule) **de la trace** (journal d'audit, dernière utilisation, sur la
   connexion ordinaire). C'est le point le plus délicat de la 32-3.
7. ⚠️ **La production ne construit pas son pool par `kesh_db::create_pool`** : `crates/kesh-api/src/main.rs:74` emploie
   `MySqlPoolOptions` directement (sans le `after_connect` de `kesh-db/src/pool.rs`). Un pool en lecture seule doit naître
   **sur le chemin réel de la production**, et le test qui prouve la lecture seule l'emploie, avec l'utilisateur `kesh`
   du fichier versionné et sans privilège posé à la main (`docs/agents-orchestration.md` § 6.1 ; précédent #669).
8. **L'adresse du pair TCP est disponible** : `into_make_service_with_connect_info::<SocketAddr>()`
   (`main.rs:399`) ; elle est déjà lue par `routes/auth.rs` et `routes/setup.rs`. Aucun code ne lit aujourd'hui
   `X-Forwarded-For`.
9. **Un précédent pour les prompts standard** : les modèles d'e-mail ont des **textes par défaut en code**
   (`kesh_db::entities::email_template_defaults`) et une table `email_templates` au `.keshbackup`. Les prompts suivent ce
   motif, avec en plus un historique et une origine.
10. **`docs/api-external.md` § 6.4** dit aujourd'hui qu'« il n'existe pas de serveur MCP Kesh-natif » et conseille un
    serveur MCP générique avec une clé `read`, et le § 9 le range « hors périmètre » ; le manuel d'administration y
    renvoie (`admin-manual.tex:2913`), la brochure le promet (`marketing-brochure.tex:480`). **Ces trois textes changent
    avec la 32-11**, pas avant : jusque-là, ils disent vrai.

---

## Points d'attention (table ronde du 2026-10-24 et rétrospective de l'Epic 31)

Chaque fiche reprend ceux qui la concernent ; la colonne dit où ils se traitent.

| Point | Story |
|---|---|
| `/mcp` dans le processus Axum, Streamable HTTP ; révision de la spécification **figée** (et testée en CI) | 32-2, 32-6, 32-11 |
| Protocole écrit à la main derrière `McpTool` et `McpPrompt` | 32-6 |
| **Garde propre au MCP** (la garde « read = GET » ne protège rien sur `POST`) | 32-6 |
| **Inventaire outil → portée testé** : un outil sans portée fait rougir le build (méthode de trait sans défaut) ; matrice portée × outil **générée** et comparée au manuel | 32-6, 32-9 |
| `tools/call` d'un outil non listé pour la clé : **refusé**, même réponse qu'un outil inconnu (aucun oracle) | 32-6 |
| Lecture seule **par construction** (connexion ou transaction en lecture seule), prouvée sur le chemin de production | 32-2, 32-6 |
| Portées séparant `books:read` et `documents:read`, jamais d'administration | 32-5 |
| Second type de clé : affichage unique, révocation, expiration, condensat ; **une clé MCP refusée sur `/api/v1`, un PAT refusé sur `/mcp`** | 32-5, 32-6 |
| Filtre d'adresse sur **le pair TCP**, désactivable, proxies de confiance déclarés (Traefik) ; tests avec IP publique simulée et `X-Forwarded-For` forgé | 32-6 |
| **Validation de l'en-tête `Origin`** (la spécification Streamable HTTP l'exige contre le *DNS rebinding*) — à établir dans la révision figée | 32-2, 32-6 |
| MCP désactivé → **404**, indiscernable d'une route absente (même port, aucun écouteur dédié) | 32-6 |
| Prompts en base avec historique (nouvelle version, jamais d'écrasement), origine `standard` / `admin` / `agent`, prompt d'agent « à valider » non servi avant approbation, marque visible dans le texte servi | 32-10a |
| Prompts standard : défauts de code copiés en base, mis à jour s'ils ne sont pas personnalisés ; **P7** si une migration écrit des données | 32-10a |
| Prompts, clés (condensat) et réglages MCP au `.keshbackup` (inventaire des tables) ; une restauration **jouée** | 32-5, 32-10a, 32-11 |
| **Validation avant envoi** : deux formes à départager par un essai (élicitation MCP, ou écran de Kesh avec appel en attente et délai) | 32-2, 32-8 |
| **Injection de prompt** : champs libres renvoyés dans une enveloppe « donnée non fiable » ; fixtures de démonstration **piégées** | 32-9a, 32-9b |
| Journal d'audit des appels MCP, **sans arguments sensibles** | 32-6 |
| **Surface de sécurité neuve** : la fiche énumère l'ensemble clos des chemins d'accès non contrôlés (*Inventorier les sites non résolus*) ; le manuel dit ce que le filtre réseau **ne garantit pas** | 32-6, 32-7 |
| **Migrations sur des livres réels** : le retour à la 0.15.0 passe par la sauvegarde ; le refus de l'image précédente **constaté à l'exécution** (A6 de la rétrospective 31) | release |

---

## Les stories

Chaque story tranche ses points en autonomie, au registre `epic-32-choix-autonomes.md` (ou un registre propre à la story,
versé par union), et suit `docs/agents-orchestration.md` — § 2.1 compris : **écritures dans le worktree seulement**,
chemins absolus, `&&`, arrêt sur un refus, dépôt principal en lecture seule ; agent de fiche sans `docker` ni `curl`.
**Périmètre minimal dès la fiche** ; un défaut voisin s'intègre s'il est mécanique, sinon il s'ouvre. **Recette
visuelle** : captures, thème clair. **Manuel** nommé parmi les axes de chaque passe (et le PDF contrôlé).

Le découpage applique la **règle de découpage préventif** (≤ 5 modules par story) ; les modules comptés sont donnés pour
chacune. 🗄️ = touche `kesh-db` (migration, dépôt ou `backup.rs`) : **gate complet même en cours de boucle**, et
*Migration breaking policy* P1 à P8.

### Ordre de dépendance

```
32-0 ──┬── 32-1a, 32-1b, 32-1c (défauts, indépendants de tout le reste, en parallèle)
       └── 32-2 (spike) ──┬── 32-3 (services : motif) ── 32-4a, 32-4b (services : rollout)
                          └── 32-12 🗄️ (import d'une sauvegarde antérieure) ── 32-5 🗄️ (clés et réglages) ── 32-6 🗄️ (endpoint /mcp)
32-6 ── 32-7 (écran Serveur MCP)
32-6 ── 32-8 (validation avant envoi) ── 32-9a, 32-9b (outils de lecture ; exigent aussi 32-4a, 32-4b)
32-6 ── 32-10a 🗄️ (prompts en base) ── 32-10b (écran des prompts)
32-7, 32-8, 32-9a, 32-9b, 32-10b ── 32-11 (brancher un agent, de bout en bout) ── release v0.16.0
```

**En parallèle possible** (worktrees, bases et ports distincts, verrou de gate) : la 32-1 avec tout ; la 32-3 avec la
32-5 ; la 32-7 avec la 32-8 et la 32-10a. **Une seule migration à la fois en développement** : les trois stories 🗄️ se
suivent (32-5 → 32-6 → 32-10a), pour qu'aucune ne se reporte sur une version de migration prise par une autre.

### 32-0 — Kickoff (doc seulement) — `done`

Ce fichier, le registre `epic-32-choix-autonomes.md` (C-32-0-1 à 12), la section `epic-32` de `sprint-status.yaml`, la
feuille de route du `README.md` et de `website/roadmap.html`, la phrase des epics ouverts du `CLAUDE.md` ; et
**l'action A2** de la rétrospective de l'Epic 31 : registres des Epics 30 et 31 versés par union (124 et 187 choix
distincts) et *Status* des huit fiches alignés sur `done` (commit `ce6b3188`).

### 32-1 — Défauts courts (une fiche, trois clés ; action A8)

Solder les **LOW mécaniques** de [#711], [#714] et [#716] — fermés par correctif, jamais pour le chiffre. Une fiche,
trois clés, comme la 31-1, pour que chaque revue reste courte. La fiche établit **chaque constat au code** (la prémisse
d'un LOW se vérifie comme celle d'un HIGH, `docs/agents-orchestration.md` § 3) et range ceux qui ne sont pas mécaniques
dans une issue plus étroite, nommée dans le rapport.

- **32-1a — tests et commentaires** ([#716] entier ; E-9 de [#711]). Modules : `kesh-api` (tests,
  `invoice_pdf_service`, `audit_log`), `kesh-db` (`company_invoice_settings`, tests), frontend (`settings/fiscal-years`),
  `kesh-i18n` (repli de `invoice-qr-message-prefix`, si la fiche le retient) — **4**. 🗄️ (un test de dépôt `kesh-db`).
  **AC** : chaque constat a son test ou son commentaire ; les mutations M4 et M9 de la 31-5 sont **tuées** ;
  `textes_coherents` borne mois et jour, cas positifs et négatifs. **Risque** : faible ; un test neuf qui passe à vide
  (`memory/tests-qui-prouvent-moins`).
- **32-1b — l'écran de la sauvegarde automatique** ([#711] : E-6, E-7, E-8, L-B3, L-B5, L-B6, L-B7). Modules :
  frontend (`admin-backup`), `kesh-api` (`routes/backup_schedule`, `auto_backup` si L-B5 le demande), `kesh-i18n`
  (L-B7, quatre locales), manuel d'administration — **4**. **AC** : une saisie non enregistrée n'est plus perdue par
  « Sauvegarder maintenant » ; un délai client dépassé recharge l'état ; un échec de sondage n'éteint plus le bandeau ;
  une date future ne produit plus de libellé absurde ; unités cohérentes ; le refus du `PUT` traduit ; le manuel dit
  que le seuil d'espace ne mesure pas la base. Captures. **E-1** (verrou tenu sans limite) et **E-2** (échec de
  `record_success`) **ne sont pas mécaniques** — conception d'un délai, chemin d'erreur : la fiche les sort dans une
  issue propre, sauf preuve contraire au code. **Risque** : moyen (comportement d'écran, E2E).
- **32-1c — la procédure de répétition** ([#714], sauf ses **deux choix de procédure**, qui reviennent à Guy). Module :
  manuel d'administration (et sa fiche) — **1**. **AC** : chaque constat textuel corrigé, PDF régénéré et relu ;
  [#714] réduite aux deux choix. **Risque** : la documentation exécutée est chère en revue (31-4 : 8 + 4 passes) — la
  fiche **ne rejoue pas** la procédure, elle corrige des phrases ; toute correction qui changerait un geste se renvoie à
  Guy avec les deux choix.

### 32-2 — Spike : protocole, transport, élicitation, lecture seule (décision, aucun code fusionné)

**Pourquoi en tête** : quatre choix structurent toutes les stories suivantes et ne se tranchent pas à la lecture ;
**chacun se tranche par un essai**. Un prototype jetable, sur une branche qui ne se fusionne pas ; le livrable est un
**document de décision** (`planning-artifacts/epic-32-spike-mcp.md`) et ses choix au registre.

1. **La révision de la spécification MCP** à figer — la plus récente stable à la date de l'essai —, ce qu'elle exige
   du serveur pour `initialize`, `tools/*`, `prompts/*`, la session (`Mcp-Session-Id` ou serveur sans état), les réponses
   JSON ou SSE, le lot JSON-RPC, et l'**en-tête `Origin`**. Le texte de référence est cité et versionné (lien et date),
   pour que le test de contrat de la 32-11 s'y adosse.
2. **Les clients réels** : **Claude Code** et **Hermes** se branchent-ils sur un `/mcp` minimal (un outil factice) en
   Streamable HTTP, avec un en-tête `Authorization` ? Lequel supporte l'**élicitation**, et avec quel délai d'attente d'un
   `tools/call` ?
3. **La validation avant envoi** : élicitation côté client, ou **écran de Kesh** avec appel en attente et délai —
   **décidé par l'essai** sur les deux clients (une forme qu'un des deux clients ne sait pas afficher ne suffit pas).
4. **La lecture seule par construction** — recommandation à éprouver : un **second pool**, construit dans `main.rs`
   comme le pool de production, dont chaque connexion passe `SET SESSION TRANSACTION READ ONLY` ; l'essai établit, avec
   l'utilisateur `kesh` du fichier versionné sur MariaDB 10.11, qu'un `INSERT`, un `UPDATE` et un `DELETE` y échouent
   **en autocommit comme en transaction** (erreur 1792), et ce qui pourrait le défaire (un `SET SESSION TRANSACTION READ
   WRITE` : aucun code ne doit l'émettre — garde lexicale). Alternative écartée sauf échec de l'essai : un second
   utilisateur MariaDB `SELECT` seul (plus fort, mais une variable `.env` de plus et un script d'init à modifier sur le
   NAS — le chemin que Guy veut quitter).
5. **Le SDK** : l'état du SDK Rust officiel à la date de l'essai, pour dessiner les traits `McpTool` / `McpPrompt` de façon
   qu'un passage ultérieur ne change que la couche de transport.

Modules : aucun fusionné. **AC** : le document de décision répond aux cinq points, chacun avec l'essai qui le fonde (sortie
citée, version des clients) ; ce qui n'a pas pu être essayé est écrit comme tel. **Risque** : un client qui ne se
comporte pas comme sa documentation — c'est la raison de la spike. ⚠️ **Aucune donnée réelle** dans l'essai : base de
démonstration seulement.

### 32-3 — La couche de services : le motif, sur deux modules pilotes

Story-zéro du motif (règle de découpage préventif). Un module `services` dans `kesh-api` ; un **contexte d'accès** (société,
utilisateur, rôle, **canal** API ou MCP, portées) ; le **contrôle d'accès dans le service** ; la **séparation de la
lecture et de la trace** (prémisse 6) : le service reçoit la connexion de lecture et, à part, de quoi écrire la trace.
Pilotes : **les rapports** (`reports`, le cas difficile : il écrit au journal d'audit) et **le plan comptable**
(`accounts`, le cas simple). Les handlers de l'API appellent les services ; **leur comportement ne change pas**.

Modules : `kesh-api/services` (neuf), `routes/reports`, `routes/accounts`, `kesh-report` (seulement si une signature
doit accepter une connexion plutôt qu'un pool) — **3 ou 4**. **AC** : les tests d'API existants des deux modules restent
verts **sans modification** ; un test par service montre le refus d'un canal ou d'une portée non autorisés ; le journal
d'audit des rapports s'écrit toujours, et sur la connexion ordinaire ; le motif est écrit (doc-comment de module) pour
le rollout. **Risque** : moyen — un rapport qui n'écrit plus sa ligne d'audit est un défaut **muet** ; la fiche
l'énumère comme mutation.

### 32-4 — La couche de services : rollout des lectures (deux clés)

Rollout **mécanique** du motif de la 32-3 aux **seules lectures** que serviront les outils (C-32-0-4) ; les écritures
restent dans les handlers jusqu'à la 0.17.0 (propositions). Revue fichier par fichier.

- **32-4a — écritures, postes ouverts, contacts** : `routes/journal_entries` (liste, détail), `routes/letterings` ou
  `accounts/{id}/open-items` (postes ouverts), `routes/contacts` (liste, détail) — **4** avec `services`.
- **32-4b — factures** : `routes/invoices` (liste, détail, échéances, règlements), `routes/supplier_invoices` (liste,
  détail), balance âgée (`reports`, déjà passé par la 32-3 : appel seulement) — **3** avec `services`.

**AC** : tests d'API inchangés et verts ; chaque service porte son contrôle d'accès ; aucun handler de lecture des
modules visés n'appelle plus un dépôt directement (inventaire clos, garde lexicale si elle est simple). **Risque** :
faible à moyen ; une liste paginée dont les bornes (`clamp(1, 200)`) se perdent dans le déplacement.

*(Annoté le 2026-10-24, validation P1 de la 32-4a : la 32-4a migre cinq des sept lectures de ses fichiers et laisse
`get_lettering` et `get_lettering_proposals` sur `state.pool`, inventoriés et figés par la garde, faute d'outil qui
les serve — C-32-4a-1, `implementation-artifacts/epic-32-choix-autonomes-324a.md` ; écart à la lettre de cet AC soumis
à Guy, fiche 32-4a, Q1. Les plafonds réels sont 500, 500 et 100, non `clamp(1, 200)` : fiche 32-4a, fait 3.)*

### 32-12 — Une sauvegarde antérieure reste importable 🗄️ *(ajoutée par l'orchestrateur le 2026-10-24, sous réserve de Guy — Q4)*

> ⛔ **Retirée le 2026-10-24 au soir** — Q4 répondue par la négative (§ *Réponses de Guy*) : Kesh n'est pas en
> service, l'installation se fera à neuf. Ne restent que la note du manuel et du CHANGELOG, portées par la 32-5. Le
> texte ci-dessous reste pour l'historique.

**Objet** : que les sauvegardes prises en 0.15.0 — automatiques comprises, celles des livres réels de Guy — restent
importables par la 0.16.0, alors que l'epic ajoute des tables à l'inventaire du `.keshbackup`. Aujourd'hui,
`parse_and_verify` exige un inventaire **identique dans les deux sens** (point 4 de *Ce que le code dit déjà*) : un
fichier plus ancien, auquel manquent les tables nées depuis, est refusé en 400.

**AC de haut niveau** : un `.keshbackup` dont l'inventaire est un **sous-ensemble** du nôtre, et dont les tables
absentes sont toutes **postérieures** à sa version (vérifié par le registre des migrations, non par le seul nom), est
importé — ces tables restent vides ou à leurs défauts, et le bilan d'import le dit ; une table **inconnue** du fichier,
ou une table manquante qui n'est pas postérieure, reste un refus ; le rejeu des backfills (`POST_RESTORE_BACKFILLS`,
P7) et sa fenêtre (`registry_entries_are_within_import_window`) sont revus, puisque la fenêtre d'import s'élargit ;
fixture réelle d'un `.keshbackup` 0.15.0 synthétique ; manuel d'administration (restauration) et CHANGELOG.

**Risques** : `kesh-db` et l'import (gate complet même en boucle) ; la fenêtre de rejeu P7, dont des exemptions
« Hors fenêtre » reposent sur l'inventaire identique — à réexaminer une à une (`scripts/prepare-release.sh` les
contrôle) ; à développer **avant** la 32-5, première story qui change l'inventaire. Si Guy préfère accepter et écrire
la limite (Q4), la story se réduit à la note du CHANGELOG et du manuel.

### 32-5 — Les clés MCP et les réglages du serveur, en base 🗄️

Le **second type de clé** : une table propre (C-32-0-6), préfixe distinct (par exemple `kesh_mcp_`), condensat SHA-256,
affichage unique, révocation, expiration, dernière utilisation, **liste de portées** ; génération et condensat
**réutilisés** d'`auth/api_key.rs` (DRY). Les **portées** de la 0.16.0 (C-32-0-7) : `books:read`, `invoices:read`,
`contacts:read`, `documents:read`, `prompts:write` — **aucune** portée d'administration ni d'écriture des livres. Les
**réglages du serveur** (activé ou non — **désactivé par défaut** —, filtre d'adresse activé ou non, plages admises,
proxies de confiance) dans une table **de société, au `.keshbackup`** (C-32-0-8). Routes de gestion (création réservée à
l'**Administrateur**, C-32-0-9 ; refusées à toute requête authentifiée par clé, comme DC6).

Modules : `kesh-db` (migration, entités, dépôts, `backup.rs`), `kesh-api` (`auth`, routes de gestion), docs
(`migrations-idempotence-audit.md`) — **3**. **AC** : la migration est **non breaking** (`CREATE TABLE`) et le dit ;
ligne d'audit d'idempotence et compteurs recomptés (P5) ; P6 inspecté ; P7 : aucune donnée écrite par la migration, ou
triage ; tables à `TABLES_TO_TRUNCATE` (et `RESET_PRESERVED_TABLES` pour les clés, comme `api_keys`) ; **une clé
restaurée authentifie** (test d'export puis import) ; une clé MCP présentée à `/api/v1` rend 401, un PAT présenté aux
routes MCP aussi. **Risque** : élevé — surface de sécurité neuve ; et la prémisse 4 (sauvegardes 0.15.0 non
importables) naît ici : la fiche l'écrit au CHANGELOG et au manuel, selon la réponse de Guy (Q4).

### 32-6 — L'endpoint `/mcp` : protocole, garde, lecture seule, filtre, journal 🗄️ (probable)

Le socle, selon la décision de la spike : `POST /mcp` en Streamable HTTP, JSON-RPC 2.0 ; `initialize`, `ping`,
`tools/list`, `tools/call`, `prompts/list`, `prompts/get` ; traits `McpTool` (dont une méthode **`portee()` sans
défaut** : un outil sans portée ne compile pas) et `McpPrompt` ; registre des outils. Un outil de démonstration
(`kesh_version`, portée `books:read`) pour tout éprouver avant la 32-9.

- **Garde propre** : clé MCP seulement (ni PAT ni session) ; `tools/list` filtré par les portées de la clé ;
  `tools/call` d'un outil hors portée = même réponse qu'un outil inconnu.
- **Lecture seule par construction** (forme de la spike) : chaque outil reçoit la connexion de lecture seule ; la
  trace va sur l'autre. Test sur le chemin de production, utilisateur `kesh`, sans `GRANT` manuel.
- **Désactivé → 404**, même corps qu'une route absente ; aucun écouteur dédié.
- **Filtre d'adresse** sur le **pair TCP** ; `X-Forwarded-For` lu **seulement** si le pair est un proxy de confiance
  déclaré ; désactivable. Tests : pair public refusé, `X-Forwarded-For` forgé par un pair non déclaré ignoré, proxy
  déclaré avec client privé admis, proxy déclaré avec client public refusé.
- **En-tête `Origin`** validé selon la révision figée.
- **Journal d'audit** de chaque appel : outil, clé, issue, durée — **sans les arguments** ni les données rendues.

Modules : `kesh-api/mcp` (neuf), `kesh-api` (`middleware`, `lib.rs` et `main.rs` pour le routeur et le pool), `kesh-db`
(si le pool en lecture seule ou un réglage y vit) — **3 ou 4**. **AC** : chaque garde a son test, et la fiche **énumère
l'ensemble clos des chemins par lesquels une requête atteint un outil** (inventaire des non-résolus) ; une mutation qui
retire la lecture seule fait rougir un test ; une qui retire le filtrage de `tools/list` aussi. **Risque** : **élevé** —
c'est la story de sécurité de l'epic ; boucle complète, Sonnet ↔ Opus.

### 32-7 — L'écran « Serveur MCP » (Administration)

Activation (désactivé par défaut), filtre d'adresse et proxies de confiance, **clés MCP** (création avec ses portées,
affichage unique du secret, révocation, expiration, dernière utilisation), avec ce que le filtre **ne garantit pas**
écrit à l'écran et au manuel (il ne remplace ni le pare-feu ni le VPN). Le manuel d'administration gagne une section
*Le serveur MCP* (activer, créer une clé, le filtre, ce qui est journalisé) ; le manuel utilisateur, rien encore
(32-11).

Modules : frontend (`features/mcp-server`, neuf, et la navigation d'administration), `kesh-i18n` (quatre locales),
manuel d'administration (PDF), `kesh-api` (seulement si une route de réglage manque après la 32-5) — **3 ou 4**. **AC** :
E2E du parcours (activer, créer une clé, la voir une fois, la révoquer) ; captures, thème clair ; le secret n'apparaît
ni dans un journal ni dans la liste. **Risque** : moyen ; un secret affiché deux fois, ou conservé dans l'état du
composant après fermeture.

### 32-8 — La validation avant envoi

Selon la forme décidée par la spike : **avant que des données partent à l'agent, l'utilisateur les voit et choisit** —
envoyer telles quelles, modifier, ou refuser. **Par construction** : le répartiteur de `tools/call` fait passer **toute**
sortie d'outil de données par cette étape ; un outil ne peut pas l'éviter (le trait ne rend pas une réponse, il rend une
proposition d'envoi). Si c'est l'écran de Kesh : appel en attente, délai borné et dit, refus à l'expiration, état en
mémoire (perdu au redémarrage : l'appel échoue, rien ne part) ; si c'est l'élicitation : ce que voit l'utilisateur est
exactement ce qui part.

Modules : `kesh-api/mcp`, frontend (écran ou avis d'approbation), `kesh-i18n`, manuel — **4**. **AC** : une donnée
modifiée à l'écran est celle que reçoit le client (test de bout en bout) ; un refus ne transmet rien ; une expiration ne
transmet rien ; l'outil de démonstration y passe. **Risque** : **élevé** — mécanisme neuf, dépendant des clients ; c'est
pourquoi il passe **avant** les outils de données (32-9).

### 32-9 — Les outils de lecture (deux clés)

Chaque outil est un `McpTool` sur un service de la 32-3 / 32-4, sa portée déclarée, son schéma d'entrée strict, ses
bornes (pagination). **Les champs libres** (libellés, noms, notes, descriptions) sont rendus dans une **enveloppe
« donnée non fiable »**, que les prompts standard apprennent à l'agent à ne jamais suivre comme une instruction ; la
**démonstration** (`kesh-seed`) gagne des **fixtures piégées** (libellés qui contiennent une instruction) et un test
vérifie qu'elles sortent enveloppées. La **matrice portée × outil** est **générée** depuis le registre et comparée à
celle du manuel par un test.

- **32-9a — les livres et les factures** (`books:read`, `invoices:read`) : plan comptable, écritures (liste, détail),
  grand livre, balance, bilan, compte de résultat, postes ouverts ; factures clientes (liste, détail, échéances,
  impayées), balance âgée, factures fournisseur (liste, détail). Modules : `kesh-api/mcp/tools`, `kesh-seed`, manuel
  — **3**.
- **32-9b — les contacts et les documents** (`contacts:read`, `documents:read`) : contacts (liste, détail) — données
  personnelles, d'où leur portée propre ; documents (liste des pièces et de leurs rattachements, contenu d'une pièce
  comme ressource MCP si la spike le permet, sinon métadonnées seulement — C-32-0-10). Modules : `kesh-api/mcp/tools`,
  `kesh-api/document_storage` (lecture), `kesh-seed`, manuel — **4**. **Peut glisser en 0.17.0** sans toucher le reste.

**AC** : chaque outil a son test d'autorisation (portée présente, absente) et de lecture seule ; l'enveloppe est
présente sur chaque champ libre (**inventaire clos** des champs libres rendus, non une liste de ceux qu'on a pensé à
traiter) ; la matrice générée égale celle du manuel. **Risque** : moyen ; un champ libre oublié hors de l'enveloppe est
un défaut muet.

### 32-10 — Les prompts (deux clés)

- **32-10a — les prompts en base** 🗄️ : tables des prompts et de leurs **versions** (jamais d'écrasement : une
  modification crée une version), origine `standard` / `admin` / `agent`, état `approuvé` / `à valider` / `refusé` ;
  `prompts/list` et `prompts/get` ne servent que la version approuvée, et un prompt d'agent porte **une marque visible
  dans le texte servi** ; un outil `proposer_prompt` (portée `prompts:write`) crée une version `agent` `à valider`. Les
  **prompts standard** sont des défauts de code (motif `email_template_defaults`), copiés en base **au démarrage** et mis
  à jour tant qu'ils ne sont pas personnalisés (C-32-0-11 : aucune migration n'écrit de données, donc pas de rejeu P7) ;
  parmi eux, **le prompt de l'agent de développement**, qui porte les règles de Guy sur les données réelles. Tables au
  `.keshbackup`. Modules : `kesh-db`, `kesh-api/mcp` (prompts, outil), `kesh-api` (routes de gestion, amorçage au
  démarrage) — **3**. **AC** : historique jamais écrasé ; une version d'agent non approuvée n'est jamais servie ; un
  standard personnalisé n'est pas remplacé par une nouvelle version de code ; restauration jouée (les prompts reviennent,
  leur historique aussi). **Risque** : moyen à élevé — un prompt est une instruction donnée à un agent qui lit des livres
  réels : c'est un vecteur d'injection, et un prompt d'agent approuvé par mégarde en est un autre.
- **32-10b — l'écran des prompts** : liste, édition (nouvelle version), historique, approbation ou refus d'une version
  d'agent, retour au standard. Modules : frontend, `kesh-i18n`, manuel d'administration — **3**. **AC** : E2E
  approbation et refus ; captures. **Risque** : faible à moyen.

### 32-11 — Brancher un agent, de bout en bout

Le parcours entier, joué : un client MCP réel (Claude Code, selon la spike) branché sur une **démonstration**, une clé
`books:read`, un prompt standard, un outil appelé, la validation avant envoi, la trace au journal. Le **test de contrat**
en CI (requêtes et réponses de la révision figée, rejouées contre le serveur dans un test d'intégration Rust ; aucun
réseau). Les textes qui changent : **`docs/api-external.md` § 6.4 et § 9** (le serveur natif existe), manuel
d'administration (*Brancher un agent*, configuration de Claude Code), manuel utilisateur (ce que l'agent peut lire, ce
qu'on lui montre avant l'envoi), brochure (`marketing-brochure.tex:480`). Une **restauration jouée** de bout en bout
(clés, réglages, prompts).

Modules : `kesh-api/tests`, docs (`api-external.md`), manuels FR (admin, utilisateur, brochure) — **3**. **AC** : le
parcours est décrit pas à pas au manuel **et rejoué depuis le PDF** ; le test de contrat rougit si une réponse s'écarte
de la révision figée ; aucun texte ne promet une écriture. **Risque** : moyen — documentation exécutée (31-4) ; la fiche
**fixe le périmètre du parcours** dès le départ pour éviter le recyclage qu'a connu la 31-4.

### Release v0.16.0

`scripts/prepare-release.sh`, macros des manuels, URL au tag (G19), CHANGELOG daté — avec : **trois migrations non
breaking au plus** (32-5, 32-6 si elle en porte une, 32-10a) — le retour à la 0.15.0 passe **par la sauvegarde** prise
avant la mise à jour ; **les sauvegardes 0.15.0 ne se restaurent plus en 0.16.0** (prémisse 4, selon Q4) ; le serveur
MCP est **désactivé** après la mise à jour. **A6 de la rétrospective 31** : le refus de l'image 0.15.0 sur une base
migrée en 0.16.0 est **constaté à l'exécution**, et le journal de l'essai cité dans la PR de release. **A9** (témoin du
nombre de tables du NAS) : à Guy avant la release.

---

## Actions des rétrospectives portées par cet epic

| Action (rétro 31) | Où | État au kickoff |
|---|---|---|
| A2 — verser les registres des Epics 30 et 31, aligner les *Status* | 32-0 | ✅ fait (`ce6b3188`) |
| A3 — clôture d'une validation au plafond | Guy | ouverte (Q2 de la rétrospective) |
| A4 — issue « ce qui manque à un redémarrage complet » | orchestrateur | ✅ [#722] ouverte |
| A5 — v0.15.0 sur le NAS | Guy | avant l'Epic 32 — non constatée au kickoff |
| A6 — refus de l'image précédente constaté | release v0.16.0 | à faire |
| A7 — parcours chaîné de la facture fournisseur | première release qui touche cette chaîne | sans objet ici (l'epic ne la touche pas) |
| A8 — LOW mécaniques de #711, #714, #716 | 32-1 | à faire |
| A9 — témoin du nombre de tables du NAS | Guy | avant la release 0.16.0 |
| A10 — `/tmp/kesh-e2e-313`, PR #719 | Guy, orchestrateur | à leur retour |

---

## Questions pour Guy

1. **L'objectif du tag v0.16.0** : moins de 140 (le compte au tag v0.15.0, à la lettre de la règle) ou moins de 136 ?
   *Recommandation* : moins de 140 (rétrospective 31, Q3) — et il reste serré (142 au kickoff).
2. **La validation avant envoi est-elle toujours obligatoire ?** Pour l'agent de développement, qui lit beaucoup, une
   approbation par appel sera lourde. *Recommandation* : obligatoire pour toute clé en 0.16.0 ; une option par clé
   (« sans validation », réservée à une clé que l'administrateur marque comme telle) se discutera après la recette.
3. **Qui crée une clé MCP ?** *Recommandation* : l'Administrateur seulement (C-32-0-9) — c'est un canal neuf vers les
   livres, et l'activation est déjà à lui.
4. **Les sauvegardes 0.15.0 deviennent non importables en 0.16.0** dès qu'une table s'ajoute (prémisse 4). L'accepter et
   l'écrire (restaurer une sauvegarde 0.15.0 demande alors l'image 0.15.0), ou rendre l'import tolérant aux tables
   ajoutées depuis (vides à la restauration) ? *Recommandation* : pour des livres réels, **rendre l'import tolérant**
   dans une story courte avant la release — mais c'est une règle de restauration, elle te revient ; en attendant, prendre
   une sauvegarde complète **juste après** la mise à jour.
5. **Les portées** `books:read`, `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` (C-32-0-7) te
   conviennent-elles ? En particulier : les contacts à part des livres, parce qu'ils portent des noms et des IBAN.
6. **Un CR pour le serveur MCP** : l'orchestrateur l'ouvre-t-il (+1 au compte, « on trace toujours ») ?
   *Recommandation* : oui.
7. **Les réglages MCP restaurés tels quels** (serveur activé si la sauvegarde l'était, C-32-0-8) — ou désactivés à toute
   restauration, par prudence ? *Recommandation* : tels quels — c'est le sens de « redémarrer les opérations » ; le
   manuel le dit.

### Réponses de Guy (2026-10-24, soir)

- **Q1** : « dans l'idéal, j'aimerais que le nombre d'issues n'augmente pas, mais il faut être pragmatique : si c'est
  compliqué, on peut accepter un peu plus d'issues. » L'objectif du tag v0.16.0 est donc **moins de 140**, sans en
  faire une barrière.
- **Q2** : tranchée avec la Q12 de la spike — **validation par clé**, `requise` par défaut, `dispensée` pour une clé que
  l'administrateur désigne, chaque envoi restant journalisé.
- **Q4** : **non**. Kesh **n'est pas encore en service** : Guy installera une instance **neuve** quand la 0.16.0, ou une
  version ultérieure, sera terminée ; les sauvegardes déjà prises n'ont pas à être restaurées. La **story 32-12 est
  retirée** (`superseded` au registre ; la PR #743 de la 32-12a, en brouillon, est fermée sans fusion). Ce qui reste, à la
  demande de Guy (« il faut le documenter au cas où ») : la limite s'écrit au **manuel d'administration** et au
  **CHANGELOG** par la story qui change l'inventaire (32-5) — une sauvegarde ne se réimporte que dans une version qui
  porte les mêmes tables ; pour restaurer une sauvegarde plus ancienne, installer l'image de la version qui l'a produite,
  importer, puis mettre à jour.
- **Q6** : fait — CR #723.
- **Q3, Q5, Q7** : pas encore de réponse ; les recommandations restent l'hypothèse de travail des fiches 32-5 et 32-6.

[#98]: https://github.com/guycorbaz/kesh/issues/98
[#164]: https://github.com/guycorbaz/kesh/issues/164
[#197]: https://github.com/guycorbaz/kesh/issues/197
[#235]: https://github.com/guycorbaz/kesh/issues/235
[#273]: https://github.com/guycorbaz/kesh/issues/273
[#403]: https://github.com/guycorbaz/kesh/issues/403
[#404]: https://github.com/guycorbaz/kesh/issues/404
[#671]: https://github.com/guycorbaz/kesh/issues/671
[#695]: https://github.com/guycorbaz/kesh/issues/695
[#699]: https://github.com/guycorbaz/kesh/issues/699
[#705]: https://github.com/guycorbaz/kesh/issues/705
[#708]: https://github.com/guycorbaz/kesh/issues/708
[#709]: https://github.com/guycorbaz/kesh/issues/709
[#710]: https://github.com/guycorbaz/kesh/issues/710
[#711]: https://github.com/guycorbaz/kesh/issues/711
[#712]: https://github.com/guycorbaz/kesh/issues/712
[#714]: https://github.com/guycorbaz/kesh/issues/714
[#716]: https://github.com/guycorbaz/kesh/issues/716
[#720]: https://github.com/guycorbaz/kesh/issues/720
[#722]: https://github.com/guycorbaz/kesh/issues/722
