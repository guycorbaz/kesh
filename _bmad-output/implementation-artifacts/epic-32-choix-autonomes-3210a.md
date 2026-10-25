# Epic 32 — choix autonomes de la Story 32-10a (les prompts MCP en base)

*Registre ouvert le 2026-10-25 à la rédaction de la fiche `32-10a-prompts-en-base.md` (Opus 5.5, en autonomie, contre
`d993106c`). Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Ce qui revient à Guy est au
§ *Points à soumettre à Guy* de la fiche. À verser par union au registre de l'epic à la clôture.*

### C-32-10a-1 — Une fiche, trois parties (i, ii, iii)

- **Contexte** : le kickoff compte trois modules ; au code, deux tables neuves touchent aussi l'export global, les tests
  de la sauvegarde, le test de la remise à zéro et trois textes publiés (manuels, CHANGELOG) ; l'audit des routes de
  gestion ajoute `audit_labels` et `kesh-i18n` ; le refus des caractères invisibles exige un prédicat partagé avec
  `bank_accounts.rs` (DRY), donc `kesh-core`. Dix modules distincts, quatorze en somme.
- **Retenu** : **i** tables, dépôt, standards, amorçage, sauvegarde et textes (5) ; **ii** prédicat, validation et
  catalogue servi (4) ; **iii** écritures, journal et audit (5). Ordre i → ii → iii ; clé `32-10a-prompts-en-base` à
  `split`, trois sous-clés `ready-for-dev`. Écart à la consigne de l'orchestrateur (« la clé passe à `ready-for-dev` »),
  déclaré et soumis à Guy.
- **Écartées** : une seule partie (dix modules) ; deux parties (la partie des écritures et du catalogue montait à six
  avec `kesh-core`) ; dupliquer la liste des caractères refusés pour économiser `kesh-core` (contraire à DRY).
- **Réversible** : oui, tant qu'aucune partie n'est développée.
- *Annoté le 2026-10-25, après la validation P1* : découpage **confirmé** par l'orchestrateur.

### C-32-10a-2 — Les prompts sont des données de société

- **Contexte** : toutes les tables applicatives portent `company_id` ; une clé MCP appartient à une société ; une table
  d'installation servirait le prompt proposé par une clé de A à une clé de B.
- **Retenu** : `mcp_prompts.company_id`, filtre au dépôt et au service.
- **Écartée** : une table d'installation (plus simple à amorcer, mais injection d'une société à l'autre et écart au motif
  de scoping).
- **Réversible** : difficilement après publication (migration).

### C-32-10a-3 — Deux tables ; le contenu d'une version ne se modifie jamais

- **Contexte** : « en base avec leur historique », « jamais d'écrasement ».
- **Retenu** : `mcp_prompts` (identité, verrou optimiste) et `mcp_prompt_versions` (une ligne par version) ; la seule
  écriture sur une version existante est la **décision** d'une version d'agent (`state`, `decided_by_user_id`,
  `decided_at`), tenue par une garde lexicale.
- **Écartées** : une table des décisions à part (plus de jointures, aucun gain : une décision est unique et terminale) ;
  des déclencheurs MariaDB (aucun précédent au dépôt).
- **Réversible** : non après publication.

### C-32-10a-4 — La version en vigueur est la version approuvée de plus grand numéro

- **Contexte** : une proposition d'agent peut arriver avant une nouvelle version standard ; « servir la dernière
  approuvée » doit rester juste quand on approuve plus tard.
- **Retenu** : en vigueur = approuvée de plus grand numéro ; une version d'agent ne s'approuve que si elle est la
  **dernière** du prompt. Ordre total, sans horloge ni pointeur.
- **Écartées** : un pointeur `mcp_prompts.version_en_vigueur_id` (FK circulaire entre les deux tables, possible sous
  `FOREIGN_KEY_CHECKS = 0` mais une dépendance de plus) ; un ordre par date d'approbation (dépend de l'horloge) ;
  approuver une ancienne proposition par-dessus un standard plus récent (l'administrateur approuverait un texte écrit
  contre un état périmé).
- **Réversible** : oui (règle de dépôt).

### C-32-10a-5 — Retirer un prompt est une version (« retrait »), non une suppression

- **Contexte** : un prompt d'agent approuvé par mégarde, ou un standard que l'administrateur ne veut plus servir, doit
  cesser d'être servi sans perdre son historique.
- **Retenu** : nature `withdrawal`, sans texte, d'origine `admin` (ou `standard` quand le code retire un standard) ; un
  prompt dont la version en vigueur est un retrait n'est pas servi.
- **Écartées** : un drapeau `actif` sur le prompt (état mutable hors de l'historique) ; une suppression.
- **Réversible** : oui.

### C-32-10a-6 — Copie en base au démarrage ; « personnalisé » ; comparaison du contenu

- **Contexte** : C-32-0-11 (copie au démarrage, aucune migration de données) ; le précédent `email_template_defaults`
  résout le défaut **à la lecture**, sans ligne — ce qui ne donne pas d'historique.
- **Retenu** : un prompt est **personnalisé** quand sa version en vigueur n'est pas d'origine `standard` ; l'amorçage
  ajoute une version `standard` quand le contenu du code **diffère** de la version en vigueur standard (et non quand un
  numéro de révision change), ce qui remet aussi au texte du code un standard altéré en base.
- **Écartées** : un numéro de révision par standard (un oubli de l'incrémenter rendrait la mise à jour muette) ; la
  résolution à la lecture (pas d'historique ; deux sources de texte).
- **Réversible** : oui.

### C-32-10a-7 — Noms : `[a-z0-9_-]{1,64}`, préfixe `kesh-` réservé aux standards

- **Contexte** : Claude Code montre `/serveur:nom` (spike § 6) ; un standard ajouté par une version future ne doit pas
  heurter un prompt d'administrateur ou d'agent du même nom.
- **Retenu** : nom conforme au motif, et `is_standard` ⇔ préfixe `kesh-`, en `CHECK` et au dépôt ; aucun renommage en
  0.16.0.
- **Écartées** : une liste des noms de standards réservés (ne protège pas les standards futurs) ; un espace de noms
  séparé (deux colonnes, même effet).
- **Réversible** : non après publication (contrainte).

### C-32-10a-8 — L'amorçage au démarrage seulement, non fatal

- **Contexte** : C-32-0-11 ; une société peut naître ou être vidée hors du démarrage (assistant, remise à zéro,
  import).
- **Retenu** : un appel en `main.rs`, après le bootstrap, avant `build_router` ; un échec se journalise et le démarrage
  continue (un prompt n'est pas un livre). Les fenêtres jusqu'au démarrage suivant sont écrites (fiche, *Ce que les
  gardes ne voient pas*) et soumises à Guy.
- **Écartées** : amorcer aussi à l'activation du serveur MCP, à la fin d'un import, à la remise à zéro (trois sites de
  plus, dans `routes` et `kesh-seed` ; gain faible : après une remise à zéro la base ne porte aucun livre réel, après un
  import les textes sont ceux que Kesh a écrits) ; un échec fatal (bloquerait la comptabilité pour un prompt).
- **Réversible** : oui.
- *Annoté le 2026-10-25, validation P1 (F5), décision de l'orchestrateur* : la prémisse de l'« Écartée » — « après une
  remise à zéro la base ne porte aucun livre réel » — était **fausse pour l'usage visé** : la remise à zéro est la
  sortie de la démonstration, donc l'entrée des livres réels (`admin-manual.tex:1412`), et rien n'oblige à redémarrer
  avant de réactiver le serveur MCP. **Retenu désormais** : deux appels de la même fonction, au démarrage (après
  `ensure_admin_user`, ordre gardé) **et** à la fin de la remise à zéro, dans `routes::onboarding::reset`, après le
  succès de `reset_demo` ; non fatal dans les deux cas. Écartés toujours : l'activation du serveur MCP et la fin d'un
  import (une restauration rend les textes de la sauvegarde ; un standard altéré revient au démarrage suivant).

### C-32-10a-9 — Deux standards, en français, en constantes de `kesh-db`

- **Contexte** : « des prompts standard […] servent d'exemples » ; le prompt de l'agent de développement porte les
  règles de Guy ; Guy travaille en français.
- **Retenu** : `kesh-developpement` et `kesh-assistant`, textes en français, `entities/mcp_prompt_standards.rs` (motif
  `email_template_defaults`), avec tests d'auto-cohérence et la présence des règles vérifiée par un test.
- **Écartées** : un seul standard (pas d'exemple d'un usage ordinaire) ; quatre langues (aucun utilisateur non
  francophone en vue) ; des fichiers de ressources (le motif du dépôt est la constante).
- **Réversible** : oui (un standard se change par une version du code ; l'amorçage le publie).

### C-32-10a-10 — Aucun argument de prompt en 0.16.0 ; `title` non servi

- **Contexte** : un gabarit interpolé est un chemin de plus (valeurs du client dans le texte) ; aucun standard n'en a
  besoin.
- **Retenu** : `arguments: []`, arguments du client ignorés, jamais interpolés ; `rendre()` sans paramètre ; `title`
  non servi (le nom suffit).
- **Écartées** : des arguments déclarés par version (colonne JSON, validation, rendu) ; refuser une requête qui en porte
  (un client pourrait envoyer `{}`).
- **Réversible** : oui.

### C-32-10a-11 — La sauvegarde emporte tout ; l'export global rien ; la remise à zéro vide

- **Contexte** : « les prompts dans la sauvegarde » ; « tout ce qui permet de redémarrer les opérations » ; un export
  destiné à un tiers ne porte que des données comptables.
- **Retenu** : les deux tables entières au `.keshbackup` (versions `pending` et `refused` comprises : c'est
  l'historique) ; à `TABLES_HORS_EXPORT` ; vidées par la remise à zéro (données de société, règle 1), les standards
  revenant au démarrage suivant.
- **Écartées** : exclure de la sauvegarde les versions non approuvées (historique tronqué) ; conserver les prompts à la
  remise à zéro (changerait la règle 4 de la partition, attachée aux principaux).
- **Réversible** : oui avant publication.

### C-32-10a-12 — Le prédicat des caractères refusés, partagé en `kesh-core` ; la validation hors de `kesh-db`

- **Contexte** : un contrôle bidirectionnel ou un caractère de largeur nulle ferait lire à l'administrateur un autre
  texte que celui que reçoit l'agent ; le dépôt porte déjà cette liste, privée, dans `bank_accounts.rs`.
- **Retenu** : la liste devient `kesh_core::text::is_hidden_format`, réemployée par `bank_accounts.rs` sans changement
  de comportement ; `is_forbidden_in_prompt` admet `\n` et `\t` ; la validation d'un contenu vit dans
  `services::prompts_mcp` (écritures de la iii, et **revalidation au service** de la ii pour un texte restauré ou écrit
  hors de Kesh).
- **Écartées** : la validation dans `kesh-db` (aurait mis `kesh-core` dans la partie i, six modules) ; une seconde liste
  (DRY) ; `kesh_core::text::is_invisible` (refuse l'espace insécable, courant en français, et ne couvre pas le
  bidirectionnel).
- **Réversible** : oui.
- *Annoté le 2026-10-25, validation P1 (F4), décision de l'orchestrateur* : le prédicat partagé est **complété** des
  points de code que l'angle mort de `bank_accounts.rs:297-302` énumérait (U+115F, U+1160, U+FFA0, U+17B4, U+17B5,
  U+180B–U+180D et U+180F, U+2065, U+1D173–U+1D17A, U+E0100–U+E01EF), retirés de cet angle mort ; la description d'un
  compte bancaire en profite (changement de comportement voulu). Reste écrit : les caractères de format d'une version
  future d'Unicode, les homoglyphes.

### C-32-10a-13 — Bornes : texte 16 000 caractères, description 500, vingt propositions en attente

- **Contexte** : un agent peut proposer sans fin ; `TEXT` tient 65 535 octets.
- **Retenu** : `body` ≤ 16 000 caractères (64 000 octets au plus), `description` ≤ 500, au plus vingt versions `pending`
  par société (`PLAFOND_A_VALIDER`).
- **Écartées** : `MEDIUMTEXT` sans borne ; un plafond par clé (une clé révoquée puis recréée le contournerait) ; aucun
  plafond.
- **Réversible** : oui (constante ; la borne en `CHECK` demande une migration).

### C-32-10a-14 — La marque : un gabarit constant, deux entiers, en tête de la description et du texte

- **Contexte** : « une marque visible dans le texte servi » ; tout texte libre interpolé (nom de clé, nom
  d'utilisateur) serait un chemin de plus.
- **Retenu** : `textes::marque_agent(numero, cle_id)`, seulement quand la version en vigueur est d'origine `agent` ; elle
  précède la description et le texte, ce qu'un texte d'agent ne peut pas défaire.
- **Écartées** : une marque en métadonnée seulement (`_meta`, invisible à l'agent) ; marquer aussi les versions
  d'administrateur (l'administrateur est la racine de confiance).
- **Réversible** : oui.

### C-32-10a-15 — Le catalogue lit sur `etat.garde`, non sur `LectureOutils`

- **Contexte** : la 32-6 réserve `LectureOutils` aux outils (N28) et lit la règle « servi » et l'authentification sur
  la `Lecture` ordinaire.
- **Retenu** : `services::prompts_mcp::{lister, obtenir}(&acces, &etat.garde, …)` ; aucune écriture sur ce chemin (R4).
- **Écartée** : la `Lecture` des outils (déballer `LectureOutils` hors de `outil.rs`, contre une garde de la 32-6).
- **Réversible** : oui.

### C-32-10a-16 — La portée des prompts est `books:read`, sans colonne

- **Contexte** : le trait exige une portée sans défaut ; un prompt sert un agent qui lit les livres.
- **Retenu** : tout prompt a la portée `BooksRead` ; `EXIGENCE_PROMPTS` = `role_api: Admin` (le service n'est servi
  qu'au MCP), `portee_mcp: Some(BooksRead)` ; une clé sans `books:read` reçoit une liste vide.
- **Écartées** : une portée par prompt (colonne, choix à l'écran) ; aucune portée (le trait l'interdit).
- **Réversible** : oui.

### C-32-10a-17 — `proposer_prompt` : un outil à part du répartiteur, sans lecture ; un accusé sans numéro

- **Contexte** : un `McpTool` reçoit `ContexteMcp` (lecture seule) et rend des `Donnees` soumises à la politique ;
  `proposer_prompt` écrit et n'envoie rien des livres (spike E2) ; l'esquisse de la spike proposait un trait
  `McpToolPrompts`.
- **Retenu** : descripteur statique, traité par le répartiteur, qui appelle `services::prompts_mcp::proposer` avec la
  poignée `EcriturePrompts` et `cle_id` ; hors `politique::autoriser` ; `readOnlyHint` à `false` pour ce seul outil ;
  accusé `{nom, etat: "a_valider"}`, **sans** numéro (il dirait à l'agent combien de versions porte un prompt qu'il ne
  voit pas).
- **Écartées** : un trait de plus pour un seul outil (la forme se décide au code : la propriété est exigée, pas la
  forme) ; l'accusé avec numéro (spike ; petit oracle sans usage, l'agent ne relit rien).
- **Réversible** : oui.
- *Annoté le 2026-10-25, validation P1 (R1, F11)* : la forme est **fixée** — l'outil n'est ni un `McpTool` ni au
  `Registre` ; la 32-10a-iii ajoute `ReponseMcp::Accuse(AccuseProposition)` (charge à champ privé, construite en un
  seul lieu), n'appelle pas `politique::autoriser`, et ajoute à `tools/list` un descripteur statique à
  `readOnlyHint: false` (explicite) sous `prompts:write` ; `registre_coherent` et `registre_standard_inventaire` restent
  inchangés, `tools_list_filtre` gagne deux cas (AC-iii2 de la fiche). Le nom réservé sort en `arguments_invalides`.

### C-32-10a-18 — Le journal : `prompts/get` tracé et fail closed ; `prompts/list` non ; une issue de plus

- **Contexte** : C-32-6-24 (« la 32-10a ajoutera `prompts/get` ») ; AC-b3 de la 32-6b (fail closed).
- **Retenu** : action `mcp.prompt_served` (détail : clé, nom du catalogue ou `"(inconnu)"`, numéro, issue) ;
  `proposer_prompt` journalisé par `journaliser_appel`, l'énumération des issues gagnant `limite_atteinte` ;
  `prompts/list` au journal du serveur seulement.
- **Écartées** : une action d'audit `mcp_prompt.proposed` en plus du journal de l'appel (doublon : la version porte
  l'auteur et la date) ; ranger le plafond sous `arguments_invalides` (faux).
- **Réversible** : oui.

### C-32-10a-19 — Les routes de gestion, dans `admin_routes`, par le dépôt et dans la transaction de l'audit

- **Contexte** : C-32-0-9 (l'Administrateur) ; précédent des routes des clés MCP (32-5c) ; C-32-0-4 (les services ne
  portent que les lectures partagées).
- **Retenu** : sept routes sous `/api/v1/admin/mcp/prompts` (deux `GET`, cinq mutantes), verrou optimiste
  `mcp_prompts.version`, écritures par le dépôt de la partie i.
- **Écartées** : le rôle Comptable (un prompt instruit un agent qui lit les livres) ; des routes de copie d'une version
  (« rétablir la version n » — l'écran pré-remplit, l'administrateur enregistre).
- **Réversible** : oui.

### C-32-10a-20 — Une version approuvée par mégarde ne se « désapprouve » pas

- **Contexte** : « préciser le traitement d'une version d'agent approuvée par mégarde ».
- **Retenu** : états décidés terminaux ; correction par une nouvelle version d'administrateur, un retrait ou le retour
  au standard, chacun audité ; prévention : approbation de la dernière version seulement, numéro restaté, verrou
  optimiste, caractères invisibles refusés, marque.
- **Écartées** : un état `revoked` (réécrit une version passée et brouille ce qui a été servi) ; une suppression.
- **Réversible** : oui.

### C-32-10a-21 — L'audit ne porte jamais le texte ni la description

- **Contexte** : l'`audit_log` est exporté par l'export global et lisible par l'API (y compris d'un agent branché par un
  PAT) ; le journal du serveur est lu par l'agent de développement.
- **Retenu** : détails `{promptId, nom, numero, origine, nature}` ; journal du serveur : identifiants seulement.
- **Écartée** : l'avant/après du texte au détail (le pratique de `mcp_server_settings.updated` ne se transpose pas à un
  texte qui instruit un agent).
- **Réversible** : oui.

### C-32-10a-22 — Aucun prompt dans la démonstration de `kesh-seed`

- **Contexte** : l'amorçage installe les standards dans toute société ; la remise à zéro vide les tables.
- **Retenu** : `kesh-seed` inchangé ; les tests posent leurs prompts par le dépôt.
- **Écartée** : des prompts de démonstration (un module de plus, sans usage).
- **Réversible** : oui.

### C-32-10a-23 — L'ordre et la migration de la 32-8

- **Contexte** : C-32-0-12 (une migration à la fois) ; la 32-8 (poignée, Q10 (c)) est 🗄️ aussi ; la consigne place la
  32-10a après la 32-6a.
- **Retenu** : i après 32-6a et 32-5d ; ii après i ; iii après ii et 32-6b. Horodatage et compteurs de la migration
  fixés à l'étape 0, après la dernière migration de `main` ; si la 32-8 fusionne entre-temps, tout compteur se recompte.
- **Écartée** : réserver dès la fiche un horodatage (deux branches se reporteraient mal).
- **Réversible** : oui.

### C-32-10a-24 — La frontière avec la 32-10b : aucune route de plus

- **Contexte** : la 32-10b est l'écran (frontend, i18n, manuel) ; une route ajoutée là touche `routes` et l'audit.
- **Retenu** : la 32-10a-iii livre toutes les routes dont l'écran a besoin (liste, historique, création, version,
  approbation, refus, retrait, retour au standard, drapeau « le standard diffère ») ; la 32-10b écrit la section du
  manuel et recompte les routes mutantes du manuel.
- **Écartée** : laisser l'écran demander ses routes.
- **Réversible** : oui.

### C-32-10a-25 — `main.rs` compte avec `routes`

- **Contexte** : la convention des fiches 32-5 et 32-6 range `lib.rs` et le `Cargo.toml` de `kesh-api` avec `routes`,
  sans nommer `main.rs`.
- **Retenu** : `main.rs` (le câblage du binaire) compte avec `routes`, comme `lib.rs`.
- **Écartée** : un module à part (porterait la partie i à six sans différence de risque).
- **Réversible** : oui (convention de compte).

### C-32-10a-26 — Les prompts sont servis quelle que soit la politique de validation de la clé

- **Contexte** : AC-a15 de la 32-6 ; spike E1 (le rendu d'un prompt ne lit pas les livres ; texte recopié : cas
  assumé, couvert par l'approbation).
- **Retenu** : `prompts/list` et `prompts/get` sous une clé `required` sont servis ; `proposer_prompt` aussi.
- **Écartée** : soumettre un prompt à la validation avant envoi (il ne porte aucune donnée lue ; la validation
  deviendrait une approbation bis, sans objet).
- **Réversible** : oui.

### C-32-10a-27 — Un agent ne propose jamais sur un nom en `kesh`

- **Contexte** : validation P1, F12 — un standard (`kesh-developpement`) porte les règles de Guy sur les données
  réelles ; une proposition d'agent approuvée le remplacerait en vigueur, règles comprises.
- **Retenu** (recommandation de l'orchestrateur, hypothèse de travail, soumis à Guy) : `proposer_version_agent` refuse
  tout nom commençant par `kesh`, existant ou non ; l'agent propose sous un autre nom ; seul l'administrateur
  personnalise un standard.
- **Écartée** : permettre la proposition sur un standard, l'approbation en tenant lieu (un agent proposerait de
  modifier ses propres règles).
- **Réversible** : oui (règle de dépôt).

### C-32-10a-28 — Le plafond et les noms neufs sous le verrou de la société

- **Contexte** : validation P1, F10 — un verrou de prompt ne rend pas atomique un plafond compté sur la société, et un
  nom neuf n'a pas de ligne à verrouiller.
- **Retenu** : toute écriture qui crée un prompt ou une version `pending` prend d'abord `companies` en `FOR UPDATE`
  (Pattern 5 du dépôt), puis le prompt ; un conflit d'unicité résiduel remonte en erreur de base.
- **Écartées** : une reprise sur conflit d'unicité (une branche de plus pour un cas que le verrou exclut) ; un plafond
  approximatif écrit comme borne.
- **Réversible** : oui.

### C-32-10a-29 — Le README et la feuille de route du site, à la 32-11

- **Contexte** : validation P1, F7 = R10 — `README.md:233` et `website/roadmap.html:375` disent que l'agent n'écrit rien
  en 0.16.0 ; `proposer_prompt` écrit une version de prompt (non les livres).
- **Retenu** (décision de l'orchestrateur) : la reformulation est affectée à la 32-11, qui réécrit les textes publics
  du serveur MCP ; la fiche le note (§ *Le manuel*), l'orchestrateur l'inscrit au fichier d'epic.
- **Écartée** : la corriger en 32-10a-iii (un module de plus, docs, pour une partie déjà à cinq).
- **Réversible** : oui.
