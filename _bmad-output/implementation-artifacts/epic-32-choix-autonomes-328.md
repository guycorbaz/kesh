# Epic 32 — registre des choix autonomes de la Story 32-8

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-8-validation-avant-envoi.md`. Numérotation `C-32-8-k`. Les numéros
de ligne cités sont ceux de `fa5b3468` ; les fiches 32-6 et 32-10a, validées mais non fusionnées, sont citées d'après
elles. Chaque choix : contexte, option retenue, alternatives écartées, réversibilité.

---

### C-32-8-1 — Une fiche, trois parties : 32-8a (la table), 32-8b (la poignée côté serveur), 32-8c (l'écran)

- **Contexte** : le kickoff comptait **4** modules (`kesh-api/mcp`, frontend, `kesh-i18n`, manuel) ; la spike (§ 6.1)
  annonçait qu'avec la poignée s'y ajoutent la migration, le dépôt et la sauvegarde. Compté à la convention des 32-5 et
  32-6, le tout fait **dix** modules distincts (`kesh-db`, docs, `admin_backup`, `routes`, manuel, `mcp`, `services`,
  `audit_labels`, `kesh-i18n`, frontend) — au-delà de cinq, la règle de découpage préventif s'applique.
- **Retenu** : trois parties, ordre a → b → c — **a** : migration, entité, dépôt **complet**, sauvegarde, remise à
  zéro, manuel (5) ; **b** : politique, dépôt, `kesh_lire_envoi`, routes de décision, journal et audit (5) ; **c** :
  écran, E2E, recette visuelle, manuel, CHANGELOG (4). Le dépôt de la 32-8a est complet (table de correspondance des
  Dev Notes) pour que la 32-8b ne touche pas `kesh-db`.
- **Écartées** : deux parties « serveur » et « écran » (la partie serveur ferait huit modules) ; quatre parties (les
  routes de décision séparées du dépôt : une partie sans test de bout en bout possible côté serveur) ; laisser les
  routes de décision à la 32-8c (elles sont de la sécurité — qui peut valider — non de l'écran).
- **Réversible** : oui — les parties se fusionnent sans rien réécrire.
- *(Annoté le 2026-10-25, validation P1, R1, R3 = F1 : **quatre parties**, ordre a → b0 → b → c. La **32-8b0** (prédicat partagé, `kesh-core`, `routes`, docs : 3 modules) est née de l'ordre fixé par l'orchestrateur (C-32-8-28). L'export global entre à la 32-8a, dont la partie documentaire passe à la 32-8c (C-32-8-29). Modules 5 / 3 / 5 / 4, douze distincts.)*

### C-32-8-2 — Une table, trois états stockés, « échu » dérivé

- **Contexte** : Q10 (c) exige une table ; la spike (E4) liste « en attente », contenu approuvé, « refusé »,
  « expiré ». Un état « expiré » écrit exigerait qu'on l'écrive — tâche de fond, ou écriture sur une lecture.
- **Retenu** : `state` ∈ {`pending`, `approved`, `refused`} ; « échu » se lit par une seule expression SQL
  (`ETAT_DERIVE_SQL`) sur `expires_at <= NOW(3)`, l'horloge étant celle de MariaDB. Aucune tâche de fond, aucun état en
  mémoire : un redémarrage ne change rien.
- **Écartées** : un état `expired` écrit par une tâche de fond (code de plus, et une fenêtre où l'état écrit ment) ; la
  comparaison faite en Rust avec l'horloge du serveur d'application (deux horloges).
- **Réversible** : oui.

### C-32-8-3 — Une seule échéance, 24 h depuis le dépôt

- **Contexte** : la spike demande une échéance et une purge, sans en fixer la durée ; la validation se fait « à son
  rythme ».
- **Retenu** : `expires_at = NOW(3) + 86 400 s` au dépôt ; passé ce délai, ni validation ni relecture, quel que soit
  l'état. Constante `MCP_ENVOI_DUREE_S`.
- **Écartées** : deux échéances (validation, puis relecture comptée depuis la décision) — plus juste pour un envoi
  validé tard, mais deux règles à dire et à tester ; une durée réglable à l'écran (un réglage de plus, à la 32-7, sans
  demande).
- **Réversible** : oui (constante) — soumis à Guy (point 1).

### C-32-8-4 — Une seule représentation : le texte des blocs ; la donnée structurée n'est ni déposée ni envoyée

- **Contexte** : E1, M-2 de la spike : « une seule représentation source est montrée, modifiée et validée ; la seconde
  est dérivée après validation » ; le choix revient à la 32-8. `Donnees` (32-6a) porte `contenu` (blocs) et
  `structure` (`structuredContent`). Un texte modifié librement ne se reconvertit pas fidèlement en structure.
- **Retenu** : `texte_a_valider` joint les textes des blocs, dans l'ordre, par une ligne vide ; c'est **le seul**
  texte déposé, montré, modifié, stocké et relu. Sous une clé `required`, aucune représentation dérivée n'est produite :
  aucune ne peut diverger. La 32-9 doit faire porter toute la donnée au texte (recommandation d'E1).
- **Écartées** : la structure en source, montrée en JSON à modifier (l'utilisateur éditerait du JSON, et le bloc texte
  devrait être resérialisé — une seconde représentation) ; les deux, dépôts séparés (deux choses à valider, et un
  écart possible entre elles) ; dériver la structure en analysant le texte s'il est du JSON (un texte modifié ne l'est
  plus forcément).
- **Réversible** : oui — une structure dérivée pourrait s'ajouter plus tard, par analyse du texte validé.

### C-32-8-5 — Les caractères cachés rendus visibles au dépôt, refusés à la modification ; l'écran en texte seul

- **Contexte** : une donnée lue peut porter une injection (libellé, nom, note) ; la plus sournoise est invisible —
  contrôle bidirectionnel, espace de largeur nulle, sélecteur de variante. Une `textarea` les masque, et normalise les
  fins de ligne : ce que l'utilisateur verrait ne serait plus exactement ce qui part.
- **Retenu** : au dépôt, `rendre_visible` remplace chaque caractère `est_cache` — `char::is_control` sauf `\n` et `\t`,
  ou le prédicat partagé `is_hidden_format` de la 32-10a — par `\u{XXXX}` ; le texte montré et envoyé n'a plus aucun
  caractère caché. Une modification qui en contient est **refusée** (400), jamais corrigée. L'écran n'affiche le contenu
  que comme valeur de champ ou texte de Svelte (aucune `{@html}`), sous un avis permanent « données, non consignes ».
- **Écartées** : refuser le dépôt d'un contenu à caractère caché (un libellé réel à emoji composé bloquerait toute
  lecture) ; marquer les caractères à l'écran seulement (ce qui part différerait de ce qui est vu) ; corriger la
  modification côté serveur (idem) ; supprimer les caractères (une donnée silencieusement altérée).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P1, R17 : le prédicat n'est plus une fonction locale `est_cache` ; c'est `kesh_core::text::is_hidden_in_text` de la 32-8b0 (C-32-8-28), le même que celui des prompts. Et un code `\u{…}` écrit tel quel dans la donnée ne se distingue pas d'un caractère remplacé : l'avis de l'écran et le manuel le disent (F6).)*

### C-32-8-6 — La politique à deux sorties : `Autorisation::{Envoi, Depot}`, `Sortie::{Envoyer, Deposer}`

- **Contexte** : la frontière de la fiche 32-6 (« la 32-8 change `autoriser` et `appliquer`, signatures comprises ») ;
  C-32-6-36. En 32-6a, `Required` refuse sans exécuter ; la poignée exige d'exécuter puis de déposer.
- **Retenu** : `autoriser(validation) -> Autorisation` (deux variantes à jeton privé, `Refus` disparu) ;
  `appliquer(Autorisation, Donnees) -> Sortie` ; `EnvoiAValider`, illisible, dont la seule méthode dépose et rend un
  `AccuseEnvoi` (un numéro). Les chaînes de données ne quittent `politique.rs` que comme arguments du service de dépôt.
- **Écartées** : un booléen « à déposer » rendu au répartiteur, qui lirait lui-même `Donnees` (la politique en deux
  lieux) ; `appliquer` asynchrone qui déposerait lui-même (la politique dépendrait d'une poignée d'écriture, et la
  construction pure deviendrait un effet).
- **Réversible** : oui.

### C-32-8-7 — `kesh_lire_envoi` hors registre, sans `autoriser`, sans portée, listé pour toute clé

- **Contexte** : M-1 de la spike — un outil de relecture soumis à la politique se remettrait en validation (boucle) ;
  exempté, il obligerait à fabriquer des données validées hors du répartiteur. La 32-10a pose le précédent d'un outil
  hors registre (`proposer_prompt`, avec son propre accusé `ReponseMcp::Accuse(AccuseProposition)`).
- **Retenu** : nom réservé, résolu avant le registre ; descripteur statique ; il rend un `ContenuValide`, type construit
  par le service seul, et ne touche ni `autoriser` ni `Donnees` ni `DonneesEnvoyables`. Listé pour **toute** clé : la
  politique d'une clé se change après coup (PUT de la 32-5c), et une clé passée de `required` à `exempt` doit pouvoir
  relire ses envois validés. Variante de réponse **propre**, `ReponseMcp::Envoi(ReponseEnvoi)`, distincte de l'accusé de
  la 32-10a.
- **Écartées** : un `McpTool` à portée (quelle portée ? celle de l'outil d'origine, à stocker, pour un contrôle que la
  règle « même clé » rend plus étroit) ; le lister seulement pour une clé `required` (la politique lue hors
  d'`autoriser`) ; réemployer `ReponseMcp::Accuse` (deux sens pour une variante).
- **Réversible** : oui.

### C-32-8-8 — « Même clé, même société » ; un envoi d'autrui se lit comme un envoi inconnu

- **Contexte** : E4 de la spike ; AC « aucun oracle » de la 32-6.
- **Retenu** : `lire_pour_cle` filtre par société **et** par clé ; inexistant, autre clé, autre société → le même
  `None`, la même réponse (`TEXTE_ENVOI_INCONNU(X)`, X étant le numéro demandé). Dérogation écrite à la règle R2 des
  services : aucune `Exigence` ne vaut pour toute clé ; la règle « même clé » est le contrôle d'accès.
- **Écartées** : une `Exigence` de portée (contrôle plus large que la clé) ; un 403 pour un envoi d'autrui (oracle).
- **Réversible** : oui.

### C-32-8-9 — Un envoi validé se relit autant de fois qu'on veut jusqu'à l'échéance

- **Contexte** : « envoi relu deux fois » ; une réponse perdue en route (réseau, client) ne doit pas perdre l'envoi.
- **Retenu** : relecture idempotente, mêmes octets ; chaque relecture journalisée (issue `servi`, numéro au détail).
- **Écartées** : une relecture unique (une réponse perdue obligerait à refaire la demande et la validation) ; un
  compteur de relectures en base (une écriture sur le chemin de lecture, que le journal rend inutile).
- **Réversible** : oui.

### C-32-8-10 — Valider : un Administrateur de la société, par sa session ; aucune clé ne valide

- **Contexte** : les clés MCP sont créées par l'Administrateur (C-32-5-7, hypothèse Q3) ; valider, c'est autoriser la
  sortie de données des livres vers un tiers. Un agent qui détiendrait aussi une clé d'API ne doit pas valider son
  propre envoi.
- **Retenu** : trois routes dans `admin_routes` : RBAC Administrateur puis `require_not_pat` (`API_KEY_ADMIN_FORBIDDEN`
  pour tout PAT, `rbac.rs:44-58`) ; une clé MCP n'ouvre jamais `/api/v1` (32-5b). Tout Administrateur de la société,
  non seulement le créateur de la clé.
- **Écartées** : le créateur seul (sans objet pour un seul utilisateur, et une clé dont le créateur est rétrogradé
  cesse d'authentifier) ; le Comptable (une décision de divulgation, à Guy — point 2).
- **Réversible** : oui.

### C-32-8-11 — Les décisions dans leurs gestionnaires, sur le pool ordinaire, avec leur audit en une transaction

- **Contexte** : les mutations restent dans les handlers (C-32-0-4) ; les services sont des lectures (règle R4) ; les
  routes d'administration existantes appellent leurs dépôts sur `state.pool` (`routes/api_keys.rs`).
- **Retenu** : `routes/mcp_envois.rs` appelle le dépôt ; `decider_in_tx` et `audit_log::insert_in_tx` dans une
  transaction ; actions `mcp_send.approved`, `mcp_send.refused`, type `mcp_send`, détail `{envoiId, cleMcpId, outil,
  modifie}` ; erreurs par les variantes existantes (`IllegalState`, `OptimisticLockConflict`, `Validation`).
- **Écartées** : un service de décision (une écriture dans `services`, contre R4) ; des variantes d'`AppError` neuves (un
  module de plus pour aucune distinction utile au client).
- **Réversible** : oui.

### C-32-8-12 — Le dépôt et l'entrée d'appel dans une seule transaction ; le journal des appels amendé

- **Contexte** : « opération enregistrée mais annoncée en échec » est un P2 (`CLAUDE.md` § *Priorités*) ; la 32-10a
  (C-32-10a-18) écrit la proposition et son audit en une transaction. Le journal de la 32-6b est fail closed.
- **Retenu** : `EcritureEnvois::deposer` insère l'envoi et l'entrée `mcp.tool_called` (issue `envoi_depose`, numéro au
  détail) dans une transaction ; `journaliser_appel` n'est pas appelé en plus. L'énumération des issues perd
  `validation_requise`, gagne les issues des envois ; le détail gagne `envoi` pour un dépôt et une relecture.
- **Écartées** : le dépôt puis `journaliser_appel` (un journal en échec laisserait un envoi déposé, annoncé en échec à
  l'agent, visible à l'écran) ; pas de numéro au journal (l'appel ne se rattacherait à l'envoi que par l'heure).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P2, F3 : un dépôt **refusé ou en échec** n'écrit rien dans sa transaction ; le
  répartiteur appelle alors `journaliser_appel` après l'échec (issue `envoi_trop_volumineux`, `limite_atteinte` ou
  `erreur`, `envoi: None`), fail closed. Une seule entrée par appel : dans la transaction s'il réussit, à part sinon.)*

### C-32-8-13 — Vingt envois en attente au plus par clé, limite souple

- **Contexte** : un agent en boucle déposerait sans fin ; l'écran déborderait, la base se remplirait de copies.
- **Retenu** : `MCP_ENVOIS_EN_ATTENTE_MAX = 20` envois en attente **vivants** par clé, comptés dans la transaction du
  dépôt, sans verrou ; au-delà, `LimiteAtteinte`, rien n'est déposé.
- **Écartées** : une limite stricte (un verrou de la ligne de la clé, un ordre de verrous à documenter, pour une
  protection contre l'inondation, non une règle comptable) ; aucune limite.
- **Réversible** : oui (constante) — pour information de Guy (point 6).

### C-32-8-14 — 64 Kio de texte rendu visible par envoi, 1 Kio de résumé

- **Contexte** : Claude Code avertit à 10 000 jetons et coupe à 25 000 (spike § 2.1) ; la limite de corps d'Axum est de
  2 Mio (`axum-core-0.5.6/src/ext_traits/request.rs:319`).
- **Retenu** : `MCP_ENVOI_OCTETS_MAX = 65 536`, mesurée **après** le rendu visible (ce qui est montré et part) ;
  `MCP_ENVOI_RESUME_OCTETS_MAX = 1 024` ; la même borne pour une modification.
- **Écartées** : 128 Kio (au-delà de 25 000 jetons pour un texte français ou JSON) ; tronquer (une donnée altérée
  silencieusement).
- **Réversible** : oui (constantes).

### C-32-8-15 — La purge paresseuse, au dépôt, hors transaction

- **Contexte** : la spike demande une purge à l'échéance ; une tâche de fond serait un écrivain de plus ; la purge dans
  la transaction du dépôt verrouillerait des lignes que la décision verrouille aussi.
- **Retenu** : `purger_echus(company_id)` vide contenu et résumé des envois échus, en une instruction, avant chaque
  dépôt de la société, hors de sa transaction, *best-effort* (un échec est journalisé par l'id de société seul).
- **Écartées** : une tâche de fond périodique (soumise à Guy, point 5) ; la purge à la lecture de l'écran (une écriture
  sur un `GET`) ; supprimer les lignes (l'agent ne pourrait plus apprendre « échu »).
- **Réversible** : oui.

### C-32-8-16 — La table est au `.keshbackup`, et vidée à la remise à zéro

- **Contexte** : Guy veut que la sauvegarde porte tout ce qui permet de redémarrer, sauf le `.env` (#722) ; P7. Une
  table **exclue** de la sauvegarde garderait, après un import, des envois rattachés par leur numéro de clé à des clés
  restaurées d'une autre installation — une fuite entre clés.
- **Retenu** : `mcp_sends` à `TABLES_TO_TRUNCATE`, juste avant `mcp_keys` ; hors `RESET_PRESERVED_TABLES` (elle rejoint
  `companies` : règle 1 du test de partition) ; la migration n'écrit aucune donnée (P7 sans objet). Le manuel dit que le
  fichier porte désormais ces contenus, version modifiée comprise.
- **Écartées** : l'exclure (préfixe `_kesh_`) et la vider à chaque import (deux mécanismes de plus pour un état
  transitoire, et le trou ci-dessus si l'un manque) ; la conserver à la remise à zéro (des copies de données de
  démonstration attachées à des clés conservées).
- **Réversible** : oui, mais une table ôtée de l'inventaire change de nouveau la compatibilité des sauvegardes.

### C-32-8-17 — Le numéro d'envoi est l'identifiant séquentiel de la table

- **Contexte** : l'utilisateur et l'agent parlent de « l'envoi n° X ».
- **Retenu** : `id` auto-incrémenté ; la règle « même clé, même société » ferme la lecture d'autrui. L'agent peut
  déduire le nombre d'envois créés entre deux des siens : oracle faible, écrit.
- **Écartées** : un jeton aléatoire (un identifiant long à dicter et à retrouver à l'écran, pour un oracle de volume
  dans une installation à un utilisateur).
- **Réversible** : oui avant la release.

### C-32-8-18 — Sous une clé `required`, l'issue d'un outil part sans validation (angle mort écrit)

- **Contexte** : E5 fait des erreurs des textes fixes ; mais « introuvable », « trop volumineux » ou une erreur
  disent un bit (existence, taille) sans que l'utilisateur ait rien vu.
- **Retenu** : l'accepter, l'écrire au manuel et au § *Ce que les gardes ne voient pas* (D3, D4).
- **Écartées** : déposer aussi les erreurs (l'écran ferait valider « introuvable » ; un mécanisme de plus pour un bit) —
  soumis à Guy (point 3).
- **Réversible** : oui.

### C-32-8-19 — Le contenu modifié est contrôlé, jamais corrigé

- **Contexte** : la propriété centrale — ce qui part est ce que l'utilisateur a vu et soumis.
- **Retenu** : `verifier_contenu_modifie` refuse un texte trop long ou à caractère caché ; aucun rognage, aucune
  normalisation Unicode ni de fin de ligne ; `modified` se calcule en comparant les **octets** en Rust (la collation de
  la colonne tiendrait `É` et `é` pour égaux).
- **Écartées** : normaliser en NFC ou rogner (ce qui part différerait de ce qui a été soumis) ; comparer en SQL.
- **Réversible** : oui.

### C-32-8-20 — Aucun contrôle du prédicat côté client

- **Contexte** : l'écran pourrait signaler un caractère caché avant l'envoi.
- **Retenu** : le serveur seul juge ; l'écran affiche son message.
- **Écartées** : une copie TypeScript du prédicat (deux implémentations qui dériveraient, l'une fondée sur les tables
  Unicode de `unicode-properties`, l'autre sur celles du navigateur).
- **Réversible** : oui — un signalement côté client peut s'ajouter, à titre d'aide, si l'usage le demande.

### C-32-8-21 — Les chemins : `/api/v1/admin/mcp/sends…` et `/admin/mcp/sends` ; la section `sec:mcp-envois`

- **Contexte** : la 32-5c monte ses routes sous `/api/v1/admin/mcp/keys…` (fiche 32-5, l. 501) ; la 32-7 fera l'écran
  *Serveur MCP* (`features/mcp-server`) et la section *Le serveur MCP* du manuel ; la 32-7 et la 32-8 peuvent se
  développer en parallèle.
- **Retenu** : API `/api/v1/admin/mcp/sends`, `/{id}`, `/{id}/decision` ; écran `/admin/mcp/sends` (feature
  `mcp-sends`, entrée de navigation propre) ; sous-section `sec:mcp-envois`, rattachée à la section de la 32-7 si elle
  existe.
- **Écartées** : un onglet de l'écran de la 32-7 (dépendance d'ordre entre deux stories parallèles) ; un bandeau
  global « N envois en attente » (utile, mais un composant de plus dans la mise en page de toutes les pages — à
  proposer après la recette).
- **Réversible** : oui.

### C-32-8-22 — Les poignées des envois : `LectureEnvois` sur le pool en lecture seule, `EcritureEnvois` sur le pool ordinaire

- **Contexte** : Q11 (les lectures du MCP sur la connexion `READ ONLY`) ; le patron `LecturePrompts`/`EcriturePrompts`
  de la 32-10a (C-32-10a-30) : des poignées qui n'exposent que leur dépôt, sans `pool()`.
- **Retenu** : `LectureEnvois` (une méthode, `lire_pour_cle`) posée sur le pool de `pool_lecture_seule` ;
  `EcritureEnvois` (`deposer`, `purger_echus`) sur le pool ordinaire ; fabriquées une fois en `lib.rs`, gardées par
  (v-2) ; les routes d'administration n'emploient pas ces poignées.
- **Écartées** : `Lecture`/`Trace` existantes (le pool lisible de tout le service, et la trace qui écrirait autre chose
  qu'un journal) ; lire les envois sur le pool ordinaire (une lecture du MCP hors de la connexion `READ ONLY`).
- **Réversible** : oui.

### C-32-8-23 — Les dépendances envers la 32-10a : le prédicat, `entree_appel`, `LimiteAtteinte`

- **Contexte** : la 32-10a-ii crée `is_hidden_format` (AC-ii1) ; la 32-10a-iii extrait `entree_appel` de
  `journaliser_appel` et ajoute `ErreurOutil::LimiteAtteinte` (AC-iii2, AC-iii3). L'ordre des fusions n'est pas fixé.
- **Retenu** : recommander la 32-8b **après** la 32-10a-ii ; sinon, la 32-8b crée le prédicat exactement selon l'AC-ii1
  de la 32-10a. `entree_appel` et `LimiteAtteinte` : la première des deux stories qui fusionne les introduit selon la
  forme de la 32-10a, l'autre les réemploie. Demandes à l'orchestrateur 2 et 3.
- **Écartées** : un prédicat propre à la 32-8 (deux prédicats des caractères cachés, contre DRY) ; attendre toute la
  32-10a (la 32-8 ne dépend pas de ses tables).
- **Réversible** : oui.
- *(Annoté le 2026-10-25, validation P1, R3 = F1, R8, décision de l'orchestrateur : l'ordre est fixé — 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, puis 32-10a. La « recommandation » et le « sinon » ci-dessus sont tranchés : la **32-8b0** crée le prédicat selon l'AC-ii1 de la 32-10a (C-32-8-28) ; la **32-8b** introduit `entree_appel` sous la signature de son AC-B11 (`envoi: Option<i64>`, cinq ou six clés) et `LimiteAtteinte` ; la 32-10a les réemploie — note au § *Note pour la 32-10a* de la fiche.)*

### C-32-8-24 — L'epic est annoté : la table remplace « l'état en mémoire »

- **Contexte** : le § 32-8 du kickoff écrit « état en mémoire (perdu au redémarrage) » et « par construction », périmés
  par la spike (C-32-2-17, C-32-2-24) et par Q10 (c) ; la consigne demande de l'annoter.
- **Retenu** : une annotation datée sous le § 32-8 de `epic-32-serveur-mcp.md` (texte d'origine conservé) : forme
  (c), table `mcp_sends`, 🗄️, trois parties, une migration de plus au plafond de la release.
- **Écartées** : réécrire le paragraphe (`CLAUDE.md` : un relevé daté s'annote, il ne se réécrit pas).
- **Réversible** : oui.

### C-32-8-25 — Le résumé est montré, jamais envoyé

- **Contexte** : `Donnees.resume` (32-6a) est « ce que l'écran de validation montre en tête » ; il peut porter des
  données (un nom de compte).
- **Retenu** : déposé à part (`summary`), rendu visible, borné à 1 Kio, montré sous le titre « Résumé de la demande —
  n'est pas envoyé » ; `ContenuValide` ne le porte pas ; purgé avec le contenu.
- **Écartées** : l'envoyer avec le contenu (une seconde chose à valider) ; ne pas le garder (l'écran ne dirait plus ce
  que l'agent a demandé).
- **Réversible** : oui.

### C-32-8-26 — Les consignes à l'agent, dans `instructions` et dans la description de `kesh_lire_envoi`

- **Contexte** : la poignée exige que l'agent rappelle la relecture (spike § 3.3) ; le risque est l'oubli ou
  l'interrogation répétée (observation (d) du § 2.4, non éprouvée).
- **Retenu** : l'accusé dit quoi faire (demander à l'utilisateur, puis relire, sans boucler) ; `instructions` et la
  description de l'outil le répètent ; textes constants, éprouvés par l'égalité des corps.
- **Écartées** : une notification au client quand l'envoi est décidé (le serveur est sans session et sans flux,
  32-6a) ; une attente côté serveur (la forme (b), écartée par Q10).
- **Réversible** : oui — la 32-11 ajustera les textes à l'observation (d).

### C-32-8-27 — L'observation (d) de la spike n'est pas rejouée avant la 32-8 : un écart déclaré (validation P1, R11)

- **Contexte** : la spike (§ 6, entrée 32-8) demande que « avant la 32-8, Guy ou la 32-8 elle-même rejoue § 2.4 (d) » —
  un agent réel rappelle-t-il l'outil de relecture, et combien de fois ? Ce rejeu demande un client réel (Claude Code,
  Hermes) ; les agents de fiche n'en lancent pas, et seul Guy le peut.
- **Retenu** (décision de l'orchestrateur) : l'écrire comme un **écart déclaré** au § *Ce que les gardes ne voient pas*
  et le poser à Guy (point 10), sans bloquer la fiche. Si l'agent oublie de relire, rien ne part : l'échec est du bon
  côté, et la correction est un texte (`TEXTE_ENVOI_DEPOSE`, `instructions`), non la conception.
- **Écartées** : attendre le rejeu (la fiche resterait bloquée sur un geste qui n'est pas le sien) ; une tâche d'essai
  sur le prototype de la spike pendant le développement (le prototype n'est pas un client réel : il ne dit rien de
  l'agent).
- **Réversible** : oui — les textes s'ajustent à l'observation, à la 32-11 au plus tard.

### C-32-8-28 — La sous-partie 32-8b0 : le prédicat partagé, créé par la 32-8, réemployé par la 32-10a (validation P1, R3 = F1, R17)

- **Contexte** : sous l'ordre fixé (32-8 avant 32-10a), la 32-8b devenait la créatrice de `is_hidden_format`, que la
  fiche 32-10a plaçait en 32-10a-ii : `kesh-core`, une dépendance, le refactor de `routes/bank_accounts.rs` — la 32-8b
  passait à six modules. Et `est_cache` de la 32-8 était, octet pour octet, le `is_forbidden_in_prompt` de la 32-10a :
  deux définitions d'une même règle.
- **Retenu** (décision de l'orchestrateur) : une sous-partie amont **32-8b0**, avant la 32-8b, qui porte seule le
  prédicat **exactement** selon l'AC-ii1 de la 32-10a (citée l. 583-635), la dépendance, la description d'un compte
  bancaire et le test déménagé, avec ses AC (AC-01 à AC-04), ses tests et ses mutations (M01 à M08) ; le prédicat des
  textes multi-lignes est **un seul** : `kesh_core::text::is_hidden_in_text`, nom neutre, que la 32-8b emploie tel quel
  et que la 32-10a-ii emploiera pour le texte d'un prompt (son `is_forbidden_in_prompt`). Le changement de comportement
  des descriptions de comptes est soumis à Guy (point 11). Une note pour la 32-10a dit que son AC-ii1 devient
  « réemployer ».
- **Écartées** : le prédicat dans la 32-8b (six modules) ; un prédicat propre à la 32-8 (contre DRY) ; garder le nom
  `is_forbidden_in_prompt` (il dirait « prompt » d'une règle qui sert aussi aux envois).
- **Réversible** : oui.

### C-32-8-29 — La partie documentaire de la 32-8a passe à la 32-8c (validation P1, R1, R5)

- **Contexte** : l'export global (`exports/global.rs`), qui rougit si une table n'y est pas classée, portait la 32-8a à
  six modules ; la consigne de l'orchestrateur : redécouper, ou déplacer la partie documentaire vers la 32-8c, en
  motivant le choix.
- **Retenu** : les passages de `admin-manual.tex` (`:1414`, `:2230`, `:2237`, `:2249`), de `user-manual.tex:3027-3033`,
  leurs PDF et la phrase de `CHANGELOG.md:15-16` passent à la 32-8c (AC-C11), qui écrit déjà le manuel d'administration
  et le CHANGELOG : **aucun module de plus** en 32-8c (4), la 32-8a revient à 5. Ces textes décrivent l'installation
  publiée, et aucune partie n'est publiable seule : aucun utilisateur ne les lit en retard.
- **Écartées** : une 32-8a-bis (une partie de plus pour quelques tests) ; sortir de la 32-8a le test de remise à zéro
  ou l'aller-retour d'import (la table séparée de la preuve de sa sauvegarde) ; une dérogation à la règle de découpage
  (rien ne la justifiait, un déplacement suffisant).
- **Réversible** : oui. *Coût* : entre les fusions de la 32-8a et de la 32-8c, deux textes de `main`, non publiés,
  comptent deux tables pour trois.
- *(Annoté le 2026-10-25, validation P2, F13 : c'est une **dérogation à la règle d'inclusion** de `CLAUDE.md` (« tout
  update de doc … dans le même commit ou la même PR que le code qui le motive »), déclarée et soumise à Guy (point 13).
  Le coût réel est de **six** sites incomplets sur `main` entre les deux fusions, non deux : `admin-manual.tex:1414`,
  `:2230`, `:2237`, `:2249`, `user-manual.tex:3027-3033`, `CHANGELOG.md:15-16` — jamais dans une version publiée.)*
