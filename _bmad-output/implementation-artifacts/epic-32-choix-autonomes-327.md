# Epic 32 — registre des choix autonomes de la Story 32-7

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-7-ecran-serveur-mcp.md`. Numérotation `C-32-7-k`. Les numéros de
ligne cités sont ceux de `966baed5`. Ce qui revient à Guy est à la fiche, § *Points à soumettre à Guy*.

---

### C-32-7-1 — Une seule clé, pas de découpage

- **Contexte** : règle de découpage préventif (au plus cinq modules). La fiche touche le **frontend** (feature neuve,
  route, navigation, E2E), **`kesh-i18n`** (quatre catalogues), les **docs** (deux manuels et leurs PDF, CHANGELOG) et
  **`kesh-api`** par un seul fichier de test (`tests/textes_coherents.rs`, G23). Aucune route ne manque (fait 1 de la
  fiche : `lib.rs:321-340`).
- **Retenu** : **4** modules, une clé `32-7-ecran-serveur-mcp`, une boucle de revue.
- **Écartées** : deux clés d'office (écran, manuel) — la seconde ne vivrait que de la première, et un écran fusionné
  sans sa section ferait parler Kesh d'un serveur que le manuel ne décrit pas ; ajouter une route (créateur affiché,
  C-32-7-16) — un cinquième module pour un confort.
- **Réversible** : oui. *(Amendé en validation P1, 2026-10-25 : la phrase d'origine — « l'ordre de rechange (C-32-7-3)
  découpe sans réécrire » — tombe avec la rechange, retirée par l'orchestrateur ; la clé unique se développe après ses
  neuf prérequis, C-32-7-3.)*

### C-32-7-2 — `/admin/mcp`, feature `mcp-server`, préfixe `mcp-server-`

- **Contexte** : la 32-8c pose `/admin/mcp/sends` et la feature `mcp-sends` (fiche 32-8, l. 214, 861-863) ; la 32-10a
  sert `/api/v1/admin/mcp/prompts` ; la règle d'appartenance i18n juge une feature à nom composé par le préfixe
  `${feature}-` (`lint-i18n-ownership.js:177-186`).
- **Retenu** : page `/admin/mcp` (racine des écrans MCP), dossier `features/mcp-server/`, clés `mcp-server-*` et
  `nav-admin-mcp`, `data-testid` préfixés `mcp-server-`.
- **Écartées** : `/settings/mcp` (les clés d'API y vivent, mais elles sont ouvertes au Comptable ; l'écran MCP est
  Administrateur seul, comme `/admin/backup`) ; préfixe `mcp-` seul (avalerait `mcp-sends-*` à la règle d'appartenance
  d'un dossier `mcp`).
- **Réversible** : oui, avant publication.

### C-32-7-3 — L'ordre de développement : en dernier, ordre nominal, sans rechange

- **Contexte** : la consigne place la 32-7 après les 32-6a/b, 32-8, 32-9 et 32-10a. Le code de l'écran ne dépend que
  de la 32-5 ; ses textes et le manuel décrivent le code de ces stories (401 fixe, `/mcp` exact, validation avant envoi
  et son écran, outils, `prompts:write`) et le manuel range leurs sous-sections. La validation P1 (F1) a montré que
  l'ordre de rechange écrit à la création n'était pas mécanique : ses trois exceptions (AC-16 (c), AC-13 (d), AC-10)
  n'étaient pas les seules dépendances du groupe A (AC-13 (c), AC-16 (e), AC-7 (a)-(b), AC-6, AC-8 (b) en avaient
  aussi).
- **Retenu** (récrit en validation P1, 2026-10-25, décision de l'orchestrateur) : développement **après** la fusion,
  dans l'ordre nominal, de 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c, 32-9a, 32-9b, 32-10a ; l'**étape 0 le vérifie**
  (`git log --oneline origin/main` porte chacune ; sinon arrêt et rapport). Aucun AC conditionnel à une fusion ; les
  groupes A, B, C ne servent qu'à la lecture.
- **Écartées** : l'ordre de rechange `32-7a`/`32-7b` (retiré : non mécanique, F1) ; développer l'écran tout de suite et
  le manuel « plus tard » sans le dire (une fusion où l'écran existe sans texte) ; écrire le manuel d'après les fiches
  voisines plutôt que d'après leur code (il mentirait à la première remédiation de leur revue).
- **Réversible** : oui, par une nouvelle décision de l'orchestrateur — elle demanderait de réécrire les AC dépendants,
  ce que la rechange prétendait éviter.
- *(Version de la création, pour l'historique : « développement après ces fusions ; un ordre de rechange `32-7a`
  (groupe A sans trois exceptions) sur la seule 32-5, puis `32-7b` ; réversible, l'orchestrateur choisit au
  lancement ».)*

### C-32-7-4 — *Le serveur MCP* est une `\section`, après *Sécurité*

- **Contexte** : `admin-manual.tex` est un `article` (`\documentclass…{article}`, l. 5) : `\section` est le plus haut
  niveau. Les 32-8 et 32-9 écrivent des `\subsection` (`sec:mcp-envois`, `sec:mcp-outils`), la 32-9 en repli dans
  `\section{Sécurité}` (l. 2862) entre les clés d'API (l. 2898) et le RBAC (l. 2927).
- **Retenu** : `\section{Le serveur MCP}\label{sec:mcp}` entre la fin de *Sécurité* et `\section{Monitoring et logs}`
  (l. 3017) ; les sous-sections des 32-8, 32-9 (et 32-10b si elle existe) y sont **déplacées sans rétrogradation**,
  étiquettes inchangées.
- **Écartées** : une `\subsection` de *Sécurité* (les sous-sections voisines deviendraient des `\subsubsection`,
  renumérotées, et le test de la matrice de la 32-9 pourrait dépendre du niveau) ; un chapitre neuf (la classe n'en a
  pas).
- **Réversible** : oui — un déplacement de bloc.

### C-32-7-5 — Le secret : séparé à la couche d'API, tenu par une instance locale au composant

- **Contexte** : la réponse 201 aplatit la clé et le secret dans un seul objet (`mcp_keys.rs:108-116`) ; la page des
  clés d'API fait circuler l'objet de création entier dans son code (`created`, `api-keys/+page.svelte:154`) et en tire
  le secret pour l'encart (l. 155) : rien ne le sépare de la clé à la frontière de l'API.
- **Retenu** : `createMcpKey` rend `{ cle, secret }`, `cle` construite champ par champ (type sans `key`) ; le secret
  vit dans une instance de `SecretUnique` créée dans le `<script>` du composant, effacée par *Fermer* et par toute
  nouvelle création, perdue au démontage ; la liste se recharge par `GET`. Cinq « ni » testés (DOM, état, liste,
  journal, stockage et adresse) ; garde G-b contre une instance de module.
- **Écartées** : recopier la page des clés d'API (l'objet complet circulerait) ; un magasin global (survivrait à la
  navigation) ; masquer le secret sans l'effacer (mutation M1).
- **Réversible** : oui.

### C-32-7-6 — Aucune portée cochée par défaut

- **Contexte** : le serveur exige au moins une portée (`mcp_keys.rs:197-200`) ; chaque portée ouvre une part
  différente des livres, `contacts:read` des données personnelles.
- **Retenu** : cinq cases vides ; refus à l'écran si aucune n'est cochée.
- **Écartées** : `books:read` d'office (le geste le plus fréquent, mais une clé à portée non choisie).
- **Réversible** : oui — soumis à Guy (point 1).

### C-32-7-7 — `sendValidation` toujours envoyé

- **Contexte** : absent, le serveur prend `required` (`mcp_keys.rs:277-280`) ; `null` y retombe aussi sans test
  (#738 B5 = E2).
- **Retenu** : l'écran envoie toujours `"required"` ou `"exempt"` ; un test compare le corps entier.
- **Écartées** : se fier au défaut du serveur (un changement de défaut côté serveur changerait l'écran en silence).
- **Réversible** : oui.

### C-32-7-8 — Les dates : naïf lu en UTC, affichage local, expiration en fin de journée locale

- **Contexte** : #738 B7 = E6 — clés en `NaiveDateTime` sans `Z` (`mcp_keys.rs:68-71`), réglages avec `Z`
  (`mcp_server_settings.rs:74-75`) ; la page des clés d'API envoie `T23:59:59Z` et affiche par `slice` (l. 73-77,
  150-153), donc en journées UTC.
- **Retenu** : `depuisUtcNaif` ajoute `Z` à une chaîne sans fuseau ; affichage par `Intl.DateTimeFormat` dans le
  fuseau du navigateur ; expiration = `23:59:59.999` locale du jour choisi, envoyée en `toISOString()`. Tests sous
  `TZ=Europe/Zurich`, avec un témoin du fuseau.
- **Écartées** : le motif des clés d'API (une clé suisse expirerait à 01:59 ou 00:59 le lendemain, et la date
  affichée serait celle d'UTC) ; corriger le serveur pour qu'il rende des `Z` (route de la 32-5c, cinquième module ;
  reste sur #738).
- **Réversible** : oui.

### C-32-7-9 — La révocation offerte à toute clé non révoquée, expirée comprise

- **Contexte** : la page des clés d'API cache la révocation d'une clé expirée (`api-keys/+page.svelte:375`) ; le serveur
  révoque toute clé non révoquée (`mcp_keys.rs:340-372`).
- **Retenu** : offerte tant que `revokedAt` est vide.
- **Écartées** : le motif des clés d'API (une clé expirée resterait « non révoquée » au journal et dans une
  sauvegarde).
- **Réversible** : oui — soumis à Guy (point 2).

### C-32-7-10 — Confirmation vers `dispensée` seulement

- **Contexte** : passer en `exempt` fait partir les données sans les montrer ; revenir en `required` ne fait que
  restreindre.
- **Retenu** : confirmation au seul passage vers `exempt`.
- **Écartées** : deux confirmations (friction sans risque) ; aucune (un clic suffirait à lever la validation).
- **Réversible** : oui.

### C-32-7-11 — Les réglages en un seul formulaire, une entrée par ligne

- **Contexte** : le `PUT` remplace tout (`mcp_server_settings.rs:7-11`) ; une entrée vide est refusée (l. 116) ; le
  serveur met chaque entrée en forme canonique sans le signaler (#738 B2 = E4).
- **Retenu** : un formulaire, *Enregistrer* unique ; zones de texte à une entrée par ligne, lignes vides retirées ;
  après succès, les champs sont **remplacés par la réponse** (la forme canonique devient visible).
- **Écartées** : une liste d'entrées à ajouter une à une (plus de composants, même contrat) ; une case d'activation
  enregistrée seule (deux `PUT` concurrents sur une même version).
- **Réversible** : oui.

### C-32-7-12 — Des avertissements informatifs, jamais bloquants

- **Contexte** : `/0` est accepté (#738 B3 = E5) ; un filtre désactivé ou une liste vide sont des réglages légitimes ;
  la sécurité relève de l'infrastructure (décision de Guy).
- **Retenu** : quatre avertissements calculés sur le texte affiché (filtre désactivé, plage `/0`, mandataire `/0`,
  liste vide filtre activé), sans empêcher l'enregistrement.
- **Écartées** : refuser `/0` à l'écran (la route l'accepte : deux règles divergentes) ; ne rien dire.
- **Réversible** : oui.

### C-32-7-13 — « Configurer un client » écrit par la 32-7, éprouvé par la 32-11

- **Contexte** : la consigne demande au manuel la configuration d'un client (Claude Code) ; la spike la donne à la 32-7
  (l. 945-946) ; l'epic donne *Brancher un agent* (parcours pas à pas, rejoué depuis le PDF) à la 32-11.
- **Retenu** : une sous-section courte `sec:mcp-client` — où va la clé, portée `user`/`local`, jamais `project` sauf
  `${KESH_MCP_KEY}`, `~/.claude.json` en clair, l'historique du shell, 25 000 jetons, Hermes —, qui dit qu'elle n'est
  pas éprouvée et renvoie à la 32-11.
- **Écartées** : tout laisser à la 32-11 (l'écran qui montre le secret ne dirait pas où le mettre) ; écrire le parcours
  complet ici (doublon de la 32-11, non éprouvé).
- **Réversible** : oui.
- *(Amendé en validation P1, 2026-10-25, R11 et F8 : la propriété de `sec:mcp-client` est fixée par C-32-7-22 ; les
  gestes y sont **sourcés** un à un — Claude Code par la spike § 2.1, Hermes Agent par la spike § 2.2, rien de plus,
  la Q9 restant sans réponse —, la portée `user` nommée sans lieu de stockage (la spike ne le dit pas), le conseil sur
  l'historique du shell présenté comme un geste général de Bash.)*

### C-32-7-14 — Les textes que la 32-7 ne porte pas

- **Contexte** : la consigne demande de corriger `admin-manual.tex:2935` et `api-external.md:185`, `:831` « si c'est la
  32-7 qui les porte selon les fiches ». La fiche 32-6 (l. 1279-1283) les donne à la **32-11** ; la fiche 32-10a
  (l. 1008, 1384) donne le compte de l. 2935 à la **32-10b**, après ses routes. La 32-7 n'ajoute aucune route.
- **Retenu** : la 32-7 ne les corrige pas ; elle les relit à l'étape 0 et écrit au Dev Agent Record s'ils sont faux
  (AC-20) ; l'orchestrateur est prié de fixer le propriétaire unique du compte (demande 1). *(Amendé en validation
  P2, F4 : tranché par l'orchestrateur en validation P1 — le compte à la 32-10b, puis à la 32-11 au dernier état ;
  demande 1 close.)*
- **Écartées** : corriger 128 → 129 ici (le compte bougera encore avec les 32-8 et 32-10a : une correction à refaire).
- **Réversible** : oui.

### C-32-7-15 — Pas de compteur des envois au menu

- **Contexte** : question 12 de la fiche 32-8 (l. 1506-1507), sans réponse.
- **Retenu** : rien ici ; soumis à Guy (point 3).
- **Réversible** : oui.

### C-32-7-16 — Le créateur d'une clé n'est pas affiché

- **Contexte** : la réponse ne porte que `createdByUserId` (`mcp_keys.rs:67`) ; la liste des utilisateurs est un autre
  écran.
- **Retenu** : non affiché ; le manuel dit qu'un appel paraît au journal au nom du créateur.
- **Écartées** : joindre `/api/v1/users` (une requête et un couplage de plus) ; changer la route (cinquième module).
- **Réversible** : oui — soumis à Guy (point 4).

### C-32-7-17 — Une garde G23 lie le manuel au code

- **Contexte** : la liste des plages par défaut et les cinq portées sont écrites au manuel ; une liste recopiée dérive
  en silence (précédent G22, `textes_coherents.rs:1673`).
- **Retenu** : G23 compare deux blocs balisés du manuel à `DEFAULT_ALLOWED_NETWORKS` et `McpScope::ALL`, et vérifie
  `100.64.0.0/10`, `Traefik` et `Docker` dans les quatre catalogues ; témoin du bloc absent.
- **Écartées** : une relecture seule (le motif « aucun gate ne lit le manuel ») ; une garde frontend (elle ne lit pas
  les constantes Rust).
- **Réversible** : oui.

### C-32-7-18 — L'adresse du serveur affichée depuis l'origine du navigateur

- **Contexte** : l'administrateur doit donner une adresse au client ; Kesh ne connaît pas sûrement la sienne
  (`KESH_PUBLIC_BASE_URL` facultatif).
- **Retenu** : `new URL('/mcp', window.location.origin)`, avec la mention « exactement `/mcp` » et la réserve qu'un agent
  du réseau local peut devoir en employer une autre.
- **Écartées** : ne rien afficher ; la déduire de `KESH_PUBLIC_BASE_URL` (une route de plus).
- **Réversible** : oui.

### C-32-7-19 — L'E2E présente la clé à `/mcp`, et range derrière lui

- **Contexte** : un secret affiché peut être faux sans qu'un test d'écran le voie ; la base E2E est partagée entre
  specs.
- **Retenu** : la clé lue à l'écran fait un `initialize` 200, puis un 401 après révocation ; un `afterAll` désactive le
  serveur et révoque les clés **si** `seedTestState` ne vide pas `mcp_keys` et `mcp_server_settings` (constaté à
  l'étape 0). *(Amendé en validation P1, 2026-10-25, R8 = F5 : c'est tranché au dépôt — `seedTestState` appelle
  `truncate_all` (`routes/test_endpoints.rs:172`), qui vide `TABLES_TO_TRUNCATE`, où sont `mcp_keys` et
  `mcp_server_settings` (`kesh-db/src/backup.rs:78`, `:83`) ; **aucun `afterAll`**, le recontrôle seul reste à l'étape 0.)*
- **Écartées** : vérifier seulement la forme du secret (un secret faux bien formé passerait).
- **Réversible** : oui.

### C-32-7-20 — « Dernière utilisation », avec sa note, jamais « dernier appel »

- **Contexte** : `last_used_at` est posée à l'authentification, même pour un appel non servi ensuite (fiche 32-6,
  l. 1015-1018).
- **Retenu** : libellé « Dernière utilisation » et une note qui dit ce qu'il mesure ; formulation relue à l'étape 0.
- **Écartées** : « Dernier appel » (faux) ; masquer la colonne (le kickoff la demande).
- **Réversible** : oui.

### C-32-7-21 — Le repli égal à la valeur fr-CH : G13 étendue par préfixe

- **Contexte** (validation P1, F2) : l'AC-14 s'adossait à une « garde frontend existante » qui tiendrait le repli égal
  au fr-CH. Lue au dépôt, aucune ne le fait pour un domaine : `i18n-un-repli-par-cle.test.ts` compare les replis d'une
  clé **entre eux** (`parTexte.size > 1`, l. 216), sur ses seuls `PREFIXES` (l. 51-72), jamais au catalogue — l'option
  préférée de l'orchestrateur (y ajouter `mcp-server-`) n'imposerait donc pas l'égalité ; G13
  (`i18n-repli-divergent-actif.test.ts`) compare bien au catalogue (`valeurDuCatalogueFr`, l. 188 ; test l. 248-252),
  mais sur la liste fermée `REPLIS_A_SITE_UNIQUE` (l. 216-235). Le commentaire de G22 (`textes_coherents.rs:1688`)
  répétait la même prémisse fausse.
- **Retenu** : étendre G13 dans son fichier par une liste de **préfixes** (`['mcp-server-']`, plus `nav-admin-mcp`
  nommée) ; pour chaque clé du préfixe relevée par `replisParCle()`, replis égaux à `[valeurDuCatalogueFr(cle)]`, par une
  fonction pure `ecartsAuCatalogue` à témoins rouge et vert ; ensemble des clés du relevé **égal** à celui du FTL fr-CH
  (une clé bâtie par gabarit ou morte rougit) ; compte exact en dur. Le commentaire de G22 corrigé (« aucune garde ») ;
  le trou de `admin-backup-` laissé à l'orchestrateur (issue ou angle mort).
- **Écartées** : ajouter `mcp-server-` aux `PREFIXES` de `i18n-un-repli-par-cle` (n'impose pas l'égalité au catalogue) ;
  inscrire quelques clés à `REPLIS_A_SITE_UNIQUE` (ne ferme pas la classe : une clé neuve y échapperait) ; un fichier de
  test neuf (redoublerait `replisParCle` et `valeurDuCatalogueFr`, contre la règle DRY) ; retirer la propriété de
  l'AC-14 (le repli de l'encadré de l'AC-6 porte une décision de Guy).
- **Réversible** : oui — une liste et un test.
- *(Amendé en validation P2, F1 = R1 : `nav-admin-mcp` est **retirée** de l'extension — clé de donnée du menu, résolue
  par `i18nMsg(item.i18nKey, item.fallback)`, que `replisParCle()` ne relève pas ; voir C-32-7-23. La recherche par
  préfixe se fait par `startsWith` : le FTL porte deux clés `audit-log-…-mcp-server-…` qui contiennent la chaîne. Le
  trou de `admin-backup-` est tracé par #758, ouverte par l'orchestrateur.)*

### C-32-7-22 — `sec:mcp-client` appartient à la 32-7 ; la 32-11 l'amende en place

- **Contexte** (validation P1, F8) : la 32-7 écrit `sec:mcp-client` (où va la clé, comment configurer Claude Code et
  Hermes Agent) ; le kickoff donne à la 32-11 « manuel d'administration (*Brancher un agent*, configuration de Claude
  Code) » (`epic-32-serveur-mcp.md:491-493`), et la fiche 32-9 (l. 1325-1327) lui attribue aussi « la clé en portée
  `local`/`user` » : deux sections du même manuel diraient la même chose.
- **Retenu** : une seule propriétaire, la **32-7** — l'encart du secret renvoie à `sec:mcp-client`, qui doit exister dès
  que l'écran montre une clé ; la 32-11 n'ouvre pas de seconde section de configuration : *Brancher un agent* (le
  parcours joué) y renvoie et, les gestes éprouvés, **amende `sec:mcp-client` en place**. Demande à l'orchestrateur 6
  (inscrire ce partage à la section 32-11 de l'epic).
- **Écartées** : donner la configuration à la 32-11 (l'encart renverrait à une étiquette absente, `??` au PDF, jusqu'à
  la 32-11) ; laisser deux textes (dérive assurée).
- **Réversible** : oui — un déplacement de texte entre deux stories du même epic.

### C-32-7-23 — `nav-admin-mcp` hors de l'extension de G13, tenue par `FAMILLES_RESOLUES['nav-']`

- **Contexte** (validation P2, F1 = R1 ; recyclage de la remédiation P1, C-32-7-21) : la P1 avait nommé `nav-admin-mcp`
  dans l'extension de G13. Or `replisParCle()` ne retient que les sites à clé et repli littéraux
  (`i18n-repli-divergent-actif.test.ts:120-123`), et le menu résout ses clés par `i18nMsg(item.i18nKey, item.fallback)`
  (`+layout.svelte:174-176`) : le relevé n'a jamais d'entrée pour elle, le test prescrit était rouge par construction.
- **Retenu** (décision de l'orchestrateur, la plus simple qui ne rougit pas par construction) : la retirer de
  l'extension ; elle est tenue, comme les autres clés de menu, par `FAMILLES_RESOLUES['nav-']` de `i18n-keys.test.ts`
  (`'admin-mcp'` ajouté), qui exige son existence au catalogue fr-CH (l. 1037-1045) — **sans** comparer son repli au
  fr-CH ni voir un oubli à la famille (`'nav-'` n'est pas à couverture close, l. 932) : angle mort écrit, classe tracée
  par #758. `ATTENDU.sitesTotal` (l. 724, assertion l. 1034) recompté et commenté par la story (AC-14 (d)) ; mutation
  M26.
- **Écartées** : un relevé propre lisant le `fallback:` qui suit `i18nKey: 'nav-admin-mcp'` dans `+layout.svelte`
  (une fonction et deux témoins pour une clé : à porter par #758 pour toute la famille, non par la 32-7) ; ajouter
  `'nav-'` à `PREFIXES_A_COUVERTURE_CLOSE` (ferait rougir toute clé `nav-` du catalogue que le menu ne demande pas,
  hors périmètre).
- **Réversible** : oui.

### C-32-7-24 — Une ligne de réglages illisible : l'écran le dit, le manuel donne les voies hors de l'écran

- **Contexte** (validation P2, R3 ; #738 B8, seconde moitié) : une ligne `mcp_server_settings` dont un tableau JSON est
  illisible rend `DbError::Invariant` au décodage (`kesh-db/src/repositories/mcp_server_settings.rs:40-45`), traduit en
  500 (`kesh-api/src/errors.rs:4270-4276`) ; `/mcp` paraît désactivé (32-6, AC-a2) et l'écran ne lit pas la
  `version`, donc n'enregistre rien (AC-5 (g)). Aucune voie de réparation dans Kesh sur une installation de
  production : la remise à zéro efface la ligne (`backup.rs:105-129`) mais est refusée au-delà de l'étape 2
  (`routes/onboarding.rs:252-268`), et la réparation au démarrage (#528) ne la touche pas.
- **Retenu** : à l'écran, une phrase fixe dans l'état d'échec de lecture (AC-5 (g)) ; au manuel (AC-18 (1)), les deux
  états qui ne se réparent pas depuis l'écran (ligne à `version = 0`, ligne corrompue) et les voies qui existent :
  restaurer une sauvegarde antérieure, ou supprimer la ligne par SQL (le `GET` rend alors les défauts, serveur
  désactivé). Voies relues au code à l'étape 0.
- **Écartées** : un bouton « rétablir les défauts » à l'écran (exigerait une route qui supprime la ligne : `kesh-api`
  hors périmètre, la story n'ajoute aucune route) ; un `PUT` toléré sans version lue (contournerait le verrou optimiste).
- **Réversible** : oui — une phrase et un paragraphe.
- *Annotation du 2026-10-25, validation P3 (F2, R2, F6)* : « Aucune voie de réparation dans Kesh » ci-dessus est trop
  large — la restauration est une fonction de Kesh. Lire : **aucune voie depuis l'écran**, ni par une action ciblée de
  Kesh. L'ordre des voies au manuel devient : d'abord supprimer la ligne par SQL (la moins coûteuse), en dernier
  recours restaurer une sauvegarde antérieure, qui remplace toute l'installation et refuse une sauvegarde d'une
  version qui ne porte pas les mêmes tables (`admin-manual.tex:2249`, `:2262`) ; le renvoi sur la réinitialisation va à
  la rangée `KESH\_PRODUCTION\_RESET` (`sec:env-vars`, l. 767). « Un tableau JSON illisible » se lit : un tableau qui
  ne contient pas que des chaînes — la base garantit un tableau JSON valide (migration `20261025000001`, l. 81-83).
  Un troisième état, une entrée invalide des mandataires, se répare **à l'écran** (fiche, AC-5).

### C-32-7-25 — Une clé dont le créateur est désactivé ou rétrogradé : dit au manuel, non à l'écran

- **Contexte** (validation P3, F1) : une clé MCP n'authentifie que tant que son créateur est actif et de rôle
  `Admin`, relus à chaque requête (`crates/kesh-api/src/services/cles_mcp.rs:20-21`, `:47`, `:94-99` ; refus avant
  `touch_last_used`, `:100`). La liste `GET /api/v1/admin/mcp/keys` ne porte que `createdByUserId`
  (`kesh-db/src/repositories/mcp_keys.rs:197-217`, sans jointure ; `routes/mcp_keys.rs:260-266`) : l'écran montre
  donc *Active* une clé qui n'authentifie plus.
- **Retenu** (décision de l'orchestrateur : « le plus simple ») : la phrase au manuel (AC-18 (2)), une ligne à *Ce que
  les gardes ne voient pas*, une mention à l'AC-8 (b) ; **aucun état à l'écran**.
- **Écartées** : un état « inopérante » calculé côté serveur (la route ne le fournit pas ; il faudrait joindre
  `users` dans `list_by_company` et ajouter un champ à `McpKeyResponse` — du code de production de `kesh-db` et de
  `kesh-api`, hors du seul fichier de test que la story touche dans `kesh-api`) ; un état calculé à l'écran depuis
  `/api/v1/users` (même écart que C-32-7-16 : une requête et un couplage de plus) ; une phrase fixe sous le tableau
  (« *Active* : ni révoquée ni expirée ») — non retenue, la consigne bornant la correction au manuel et à la liste
  des gardes quand l'écran ne peut pas calculer l'état.
- **Réversible** : oui — un état ajouté plus tard ne change rien au manuel ; lié au point 4 soumis à Guy (créateur
  non affiché).
- *Annotation du 2026-10-25, validation P4 (R7 (a), F8)* : « refus avant `touch_last_used`, `:100` » se lit `:94-99`
  pour les refus et `:101` pour la pose (la l. 100 est vide). Et « quand l'écran ne peut pas calculer l'état » ci-dessus
  est trop fort : il le pourrait par `/api/v1/users` (`UserResponse` porte `role` et `active`,
  `routes/users.rs:82-86`) — voie écartée ci-dessus pour une requête et un couplage de plus, non pour impossibilité.

### C-32-7-26 — `'admin-mcp-sends'` n'est pas ajouté à `FAMILLES_RESOLUES['nav-']`

- **Contexte** (validation P4, R9 = F7) : l'AC-14 (d) traitait l'absence de `'admin-mcp-sends'` comme « un écart à
  déclarer » de la 32-8c ; la fiche 32-8 ne prescrit ni l'entrée de famille ni le recompte de `sitesTotal`
  (`command grep -nF 'FAMILLES_RESOLUES'` sur la fiche 32-8 → rien). `'nav-'` n'est pas à couverture close
  (fait 21) : l'absence ne fait rougir aucun test.
- **Retenu** : écrire le fait — son absence est l'état attendu, constatée à l'étape 0 ; la 32-7 n'ajoute que
  `'admin-mcp'` (correction de R9).
- **Écartées** : l'ajouter aussi et le déclarer (correction de F7) — une entrée d'une autre story, qu'aucun test
  n'exige, élargirait le diff sans rien prouver.
- **Réversible** : oui — une ligne d'un tableau de test.

### C-32-7-27 — l'entrée refusée et stockée hors de l'écran est un angle mort écrit, non un test ni une capture

- **Contexte** (validation P5, F4) : la P4 a ajouté à l'AC-5 (*Ce qui se répare à l'écran*) et à l'AC-18 (1) qu'une
  entrée que l'écran refuse, posée hors de l'écran, bloque tout enregistrement ; aucun test ni aucune capture ne pose
  une telle entrée — le cas (d) et la capture (7) montrent le 400 d'une entrée **saisie**, au même rendu.
- **Retenu** : l'écrire comme angle mort au § *Ce que les gardes ne voient pas* ; la phrase du manuel reste tenue
  par la relecture de `validate_settings` à l'étape 0 (§ *À recontrôler*, P4). Option la plus simple, consigne de
  l'orchestrateur (« choisis le plus simple »), et remédiation par retrait de la prétention de couverture.
- **Écartées** : un cas de test et une capture neufs (une entrée posée par SQL, l'écran rechargé, *Enregistrer*
  cliqué) — l'écran n'a aucun rendu propre à ce cas, le 400 est générique ; le test vérifierait surtout le serveur,
  et ajouterait une recette SQL de plus à la base de démonstration.
- **Réversible** : oui — une capture et un cas de test à ajouter en développement ou en revue.

## Choix du développement (2026-10-27, worktree `kesh-32-10a`, branche `story/32-7`)

### C-32-7-28 — G13 par préfixe : trois préfixes, sur le module partagé de la 32-10b

- **Contexte** : la fiche (AC-14 (a), C-32-7-21) prescrit `PREFIXES_REPLI_EGAL_AU_CATALOGUE = ['mcp-server-']` et une
  fonction pure `ecartsAuCatalogue`. Entre-temps, la 32-10b a extrait cette fonction et le relevé dans
  `frontend/src/lib/shared/i18n-repli-catalogue.ts` (employé par les gardes des écrans des envois et des prompts) ;
  l'orchestrateur demande de le substituer par la liste de préfixes si la fiche l'introduit, sinon de le réemployer.
- **Retenu** : l'extension vit dans `i18n-repli-divergent-actif.test.ts`, sur le module partagé (`releveDesReplis`,
  `sourcesDesDossiers(['src'])`, `ecartsAuCatalogue`, `catalogueDe`) — aucune copie (DRY) ; la liste porte les **trois**
  préfixes `mcp-server-` (95), `mcp-sends-` (48) et `mcp-prompts-` (94), chacun avec son compte écrit en dur ; le relevé
  couvre tout `src/`, non le seul dossier d'un écran.
- **Écartées** : une fonction pure locale à G13 (doublon du module) ; retirer les sections 1 des gardes des écrans
  des envois et des prompts (elles contrôlent aussi les quatre catalogues et la clé de menu, ce que G13 ne fait pas :
  les retirer perdrait des contrôles).
- **Réversible** : oui.

### C-32-7-29 — `SecretUnique` est admise nommément par la garde « aucune classe dans `src/` »

- **Contexte** : `i18n-libelle-en-dur.test.ts` exige qu'aucune classe ne vive dans `src/` (prémisse de sa limite
  « méthodes de classe ») ; la fiche fixe une classe `SecretUnique` (AC-11, garde G-b sur `new SecretUnique`).
- **Retenu** : une liste `CLASSES_ADMISES` d'une entrée, `secret-unique.svelte.ts`, avec son motif : ses trois méthodes
  ne rendent aucun libellé. Toute autre classe rougit toujours.
- **Écartées** : une fabrique (`creerSecretUnique()`) au lieu d'une classe — contraire à la signature fixée par la
  fiche et à la forme de G-b.
- **Réversible** : oui.

### C-32-7-30 — le déplacement de sous-sections du manuel par coupe de lignes, vérifié

- **Contexte** : `sec:mcp` devait se placer après *Sécurité* et avant *Monitoring*, et y ranger `sec:mcp-outils`,
  `sec:mcp-envois` et `sec:mcp-prompts` sans changer leurs étiquettes. Ces trois sous-sections (≈ 33 Ko) étaient au
  milieu de *Sécurité*, suivies de cinq autres (RBAC, journal d'audit, OLICo, pare-feu, mises à jour, ≈ 12 Ko).
- **Retenu** : déplacer les cinq sous-sections de *Sécurité* avant les trois sous-sections MCP, par une coupe de lignes
  (`head`/`sed -n`/`tail`), sans aucune substitution de motif, vérifiée par l'égalité des multiensembles de lignes
  (`diff <(sort avant) <(sort après)` vide, 3611 lignes des deux côtés) ; la `\section{Le serveur MCP}` et les
  sous-sections neuves se sont ensuite écrites à l'outil d'édition.
- **Écartées** : recopier 12 à 33 Ko par l'outil d'édition (risque de recopie inexacte, que l'égalité des lignes
  exclut ici) ; ranger le reste sous une « Sécurité (suite) ».
- **Réversible** : oui (git).

### C-32-7-31 — la phrase de `documents:read` nomme la facture fournisseur liée, et `invoices:read` l'adresse courriel

- **Contexte** (étape 0) : la fiche (AC-10, R4 (a)) écrivait « le nom du fichier, le type et l'état de chaque pièce
  importée, sans son contenu », en retirant « rattachements » ; le code de la 32-9b rend aussi `taille_octets`,
  `importee_le`, `qr_lu` et `facture_fournisseur_id` (`crates/kesh-api/src/mcp/outils/pieces.rs:47-56`). Et
  « l'adresse d'envoi » masquée sans `contacts:read` est l'adresse **courriel** d'envoi (`emailed_to`,
  `mcp/outils/factures.rs:472`).
- **Retenu** : réécrire selon le code (règle de la fiche, tête) — « le nom du fichier, le type, la taille, l'état et la
  facture fournisseur liée de chaque pièce importée, sans son contenu » ; « l'adresse courriel d'envoi d'une facture » ;
  aux quatre locales et au bloc `% mcp-portees:` du manuel.
- **Réversible** : oui.

### C-32-7-32 — l'erreur « hors `ApiError` » n'a pas de capture réelle

- **Contexte** : la capture (24) prescrit l'erreur générique hors `ApiError` à l'enregistrement des réglages ; dans un
  vrai navigateur, `api-client.ts` convertit toute panne réseau en `ApiError` traduite (`toFetchApiError`, l. 312-321) :
  le générique n'est atteignable qu'en simulation.
- **Retenu** : le cas est tenu par le test Vitest (AC-5 (f)) ; la capture (24) montre l'erreur réseau réelle (requête
  `PUT` interrompue), déclarée telle.
- **Réversible** : oui.

### C-32-7-33 — la navigation entre les trois écrans MCP : le menu et deux liens de l'écran Serveur MCP

- **Contexte** : l'orchestrateur demande que l'entrée de menu et la navigation tiennent ensemble `/admin/mcp`,
  `/admin/mcp/sends` et `/admin/mcp/prompts`.
- **Retenu** : l'entrée `nav-admin-mcp` immédiatement avant `nav-admin-mcp-sends` (les trois se suivent au groupe
  *Administration*), et deux liens en tête de l'écran *Serveur MCP* vers les deux autres. Les écrans des envois et des
  prompts ne sont pas touchés (leurs clés appartiennent à leurs stories).
- **Écartées** : un lien retour dans les écrans des envois et des prompts — clés et tests d'autres features.
- **Réversible** : oui.
