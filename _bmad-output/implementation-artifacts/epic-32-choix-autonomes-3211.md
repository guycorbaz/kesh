# Epic 32 — registre des choix autonomes de la Story 32-11

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-11-brancher-un-agent.md`. Numérotation `C-32-11-k`. Les numéros de
ligne cités sont ceux de `ed981c7d`, sauf mention d'une fiche voisine. Chaque choix : contexte, option retenue,
alternatives écartées, réversibilité.

---

### C-32-11-1 — Une seule clé, quatre modules ; le découpage en 32-11a et 32-11b est écarté, et pré-déclaré

- **Contexte** : l'epic compte trois modules (`kesh-api/tests`, docs, manuels FR) ; le report du 2026-10-25 ajoute
  `README.md` et `website/roadmap.html`. La règle de découpage préventif joue au-delà de cinq.
- **Retenu** : une clé, **quatre** modules — `kesh-api/tests` (contrat, garde du compte), `docs/api-external.md`, les
  manuels FR (trois `.tex` et leurs PDF), les textes publics (`README.md`, `website/roadmap.html`, `CHANGELOG.md`).
  Aucun code de production. Le test de contrat dépend du parcours (sa fixture vient de la capture de P3), ce qui lie les
  deux parties.
- **Écartées** : 32-11a (contrat) et 32-11b (parcours, manuels, textes) dès la fiche — sous le seuil, et la fixture de
  la 32-11a attendrait la 32-11b. **Pré-déclaré** : si une passe de validation ou de revue montre un recyclage sur le
  contrat seul, il se détache en 32-11a, la fixture K4 retirée (AC-B5).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur)** : le test de contrat ne dépend plus du parcours (aucune fixture, AC-B5) ; le lien « la fixture de la 32-11a attendrait la 32-11b » tombe. Le découpage reste écarté (quatre modules) et pré-déclaré.

### C-32-11-2 — Le périmètre du parcours est une liste close : P1 à P11, et P-pièges selon Q3

- **Contexte** : l'epic demande que la fiche « fixe le périmètre du parcours dès le départ » ; la 31-4 a recyclé sur ce
  que les remédiations ajoutaient **autour** de la procédure (C-31-4-15).
- **Retenu** : onze gestes nommés (AC-A2), plus la restauration (E) ; tout le reste (clé dispensée, autres portées,
  refus et échéance d'un envoi, filtre refusant, mandataire, plusieurs sociétés, DSM, Traefik) est **décrit par le
  manuel tel que le code le fait**, non rejoué. Ajouter un geste au parcours est un changement de fiche, non une
  remédiation.
- **Écartées** : « jouer tout ce que le manuel décrit » (sans fin, et le DSM n'est pas sur la station) ; un parcours
  minimal sans restauration (l'epic l'exige).
- **Réversible** : oui, par une décision écrite.
- **Amendé le 2026-10-27 (validation P1, R1, F1, F3)** : douze gestes, P0 à P11 — P0, l'amorçage, ajouté (C-32-11-25) — et deux conditionnels nommés, P3-direct et P-pièges ; la liste de l'AC-A2 est la seule source des gestes, les variantes des Dev Notes en sont retirées.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur ; R11)** : douze gestes, P0 à P11 — P0 compris, que la rédaction d'origine (« P1 à P11 ») omettait —, **aucun geste conditionnel** : P3 devient le client HTTP (C-32-11-28), P3-direct est retiré (il naissait du mandataire placé devant Claude Code), P-pièges passe à l'Annexe (C-32-11-24).

### C-32-11-3 — Un défaut révélé par le parcours ne se corrige pas dans la 32-11

- **Contexte** : C-32-8-26 écrit « la 32-11 ajustera les textes à l'observation (d) » ; corriger en cours de parcours
  rouvrirait du code de production dans une story de documentation, et chaque correction appellerait un nouveau
  rejeu — le motif de recyclage de la 31-4.
- **Retenu** : la 32-11 observe, écrit le constat, demande l'issue ; le manuel décrit le comportement réel. Un défaut
  bloquant arrête le parcours ; la correction est une story à part (Q4). Cela vaut pour les textes de la poignée
  (`TEXTE_ENVOI_DEPOSE`, `instructions`) : C-32-8-26 est repris ici, la correction éventuelle sort de la 32-11.
- **Écartées** : corriger les textes dans la 32-11 (cinquième module `kesh-api/mcp`, code de production, tests et
  mutations de plus) ; ignorer le constat.
- **Réversible** : oui (Q4).

### C-32-11-4 — Qui joue : l'agent de développement, en mode non interactif ; Guy rejoue l'interactif à la recette

- **Contexte** : le parcours demande la station et un client réel ; la spike (§ 2, l. 206-207) le laissait « à Guy
  ou à la 32-11 ». Claude Code 2.1.311 est installé ; un agent peut le lancer en `claude -p`.
- **Retenu** (hypothèse de travail, Q1) : l'agent joue P1 à P11 et la restauration, preuve par journal, transcript,
  capture `socat`, captures d'écran et export du journal d'audit ; Guy rejoue à la recette les gestes que le mode non
  interactif ne sert pas (AC-A8). Si le système de permissions refuse le lancement de `claude`, l'agent s'arrête.
- **Écartées** : Guy seul (coûteux à chaque correction, non rejouable par un agent) ; le test serveur seul (ne dit rien
  du client).
- **Réversible** : oui (réponse de Guy).
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur ; R11)** : l'agent joue P0 à P11 et la restauration **en client HTTP**, sans lancer `claude` (C-32-11-28) ; Guy joue les gestes de Claude Code et capture ses requêtes à la recette (C-32-11-29). Le pilotage de `claude -p` par l'agent devient une variante soumise à Guy (point 1, Annexe), non retenue par défaut.

### C-32-11-5 — La configuration de Claude Code de Guy n'est pas un banc

- **Contexte** : `claude mcp add --scope local` écrit `~/.claude.json` (spike § 2.1) ; `--scope project` écrit un
  `.mcp.json`.
- **Retenu** : l'agent configure le client par un fichier jetable, `--mcp-config` et `--strict-mcp-config`, sous
  `kesh-gate-logs/3211/` ; il n'écrit jamais `~/.claude.json`, `~/.claude/` ni un `.mcp.json` du dépôt. `claude mcp add`
  ne s'éprouve que si la documentation, relue à l'étape 0, nomme une variable qui déplace le dossier de configuration ;
  sinon le geste reste « non éprouvé » au manuel (AC-A8).
- **Écartées** : `claude mcp add` sur la configuration de Guy, puis `remove` (une écriture dans ses fichiers, et ses
  serveurs MCP propres chargés pendant le parcours).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P1, F2)** : « il n'écrit jamais `~/.claude.json`, `~/.claude/` » devient « il n'y écrit jamais **lui-même** » : `claude`, lancé avec l'authentification de Guy, écrit ses sessions dans le magasin habituel de Guy, ce qui est accepté et déclaré (C-32-11-26). `claude mcp add` ne s'éprouve plus du tout : la condition « variable qui déplace le dossier de configuration » est retirée, un tel dossier retirant l'authentification.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur ; F3 = R6)** : l'amendement de la validation P1 (« jamais **lui-même** ») tombe : l'agent ne lance pas `claude` et n'écrit rien dans `~/.claude.json` ni `~/.claude/`. Les écritures de `claude` dans ces fichiers dérogeraient à `docs/agents-orchestration.md` § 2.1 ; elles ne se posent qu'à la variante de l'Annexe, sur un oui de Guy (point 1).

### C-32-11-6 — Les échanges se capturent par un mandataire `socat`, sans changer le produit

- **Contexte** : l'observation (h) (révision demandée) et les observations (b), (c), (e), (f) demandent de voir les
  requêtes ; Kesh ne journalise pas la `protocolVersion` demandée (`repartiteur.rs:164`, fiche, fait 6) ni les corps.
- **Retenu** : `socat -v` sur `127.0.0.1:<P+1>`, relayant vers Kesh ; sa sortie au journal d'exécution, hors du
  dépôt (elle porte la clé de démonstration, révoquée ensuite). La fixture du contrat en est tirée, clé remplacée.
- **Écartées** : ajouter un `debug!` de la révision demandée à `initialize` (code de production, hors du périmètre
  fixé par C-32-11-3 ; utile peut-être, à proposer en issue si le parcours le montre) ; le journal de débogage de Claude
  Code (son contenu n'est pas documenté dans les sources de la spike).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur)** : l'agent ne lance plus `socat` ; la capture des requêtes de Claude Code est un geste de Guy à la recette (AC-A8, C-32-11-29). Le parcours de l'agent garde ses requêtes et ses réponses dans des fichiers (AC-A1).

### C-32-11-7 — L'oracle du contrat est la phrase de la spécification, jamais la sortie de Kesh

- **Contexte** : les tests existants comparent à `mcp::textes` ou à la route absente, c'est-à-dire à ce que Kesh a
  décidé (fiche, fait 3). Un contrat dont l'attendu sort du serveur testé serait vert par construction (D4-ter).
- **Retenu** : chaque cas porte une `CLAUSE` (URL datée de la révision `2025-11-25`, section, phrase citée) et l'attendu
  s'en déduit ; ce que la spécification laisse au serveur est marqué `DECISION_KESH` avec sa référence. Clés closes des
  objets de résultat recopiées de la spécification.
- **Écartées** : instantanés des réponses de Kesh (auto-certifiants) ; le `schema.json` officiel validé par une
  dépendance `jsonschema` (absente du dépôt ; Q2, réexaminé avec le SDK).
- **Réversible** : oui (Q2).
- **Amendé le 2026-10-27 (validation P1, F4, R2, R3)** : voir C-32-11-27 — les « clés closes » et la présence d'`isError` sont des décisions de Kesh, non des clauses.

### C-32-11-8 — Un binaire neuf `mcp_contrat`, liste close K1 à K22, recoupement des tests existants assumé

- **Contexte** : la spike § 6 (l. 973-975) donne la séquence ; la 32-8 y ajoute le dépôt et la relecture, la 32-9 un
  outil de données, la 32-10a les prompts ; une partie est déjà testée par garde dans `mcp_transport_e2e.rs`.
- **Retenu** : `tests/mcp_contrat.rs`, rangé **par clause**, réemployant `tests/common/mcp.rs` ; les tests existants ne
  bougent pas. Le recoupement est voulu : l'un range par garde, l'autre par clause, et le contrat est l'endroit unique où
  se lit « Kesh sert la révision `2025-11-25` ».
- **Écartées** : déplacer les cas existants dans le contrat (réécriture de tests verts, risque de test muet) ; ajouter
  des cas à `mcp_transport_e2e.rs` (2081 lignes, rangé par garde).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P1, R2, F6)** : le réemploi de `tests/common/mcp.rs` **exclut** les aides `attendu_*`, qui tirent leur attendu de `mcp::textes` (C-32-11-27).

### C-32-11-9 — La fixture ne porte que des requêtes réellement envoyées par Claude Code, avec un témoin

- **Contexte** : rejouer « ce qu'un client envoie » n'a de valeur que si c'est ce qu'il envoie.
- **Retenu** : `tests/fixtures/mcp-contrat/` : `initialize`, `notifications/initialized`, `tools/list` tirés de
  `socat.log`, en-tête de provenance, clé remplacée par un marqueur ; K4 exige que `clientInfo.name` nomme Claude Code
  (mutation MF1). Sans P3 joué, K4 est retiré, aucune fixture n'est inventée.
- **Écartées** : une requête écrite à la main « comme Claude Code » (rien ne la distingue d'une invention) ; versionner
  les réponses (C-32-11-7).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur)** : **retiré** — aucune fixture de Claude Code n'est versionnée par la 32-11 (AC-B5) ; K4 et MF1 sont retirés ; les requêtes capturées par Guy se comparent aux clauses, une contradiction s'ouvre en issue.

### C-32-11-10 — Le compte des routes se recompte, et une garde le relie au registre ; retrait pré-déclaré

- **Contexte** : `admin-manual.tex:2978` dit 122 des 128 ; le registre en compte 124 des 130 ; trois stories de l'epic
  l'ont laissé faux, et la 32-10b le recompte encore.
- **Retenu** : le compte recompté depuis `audit_route_registry.rs` (AC-C4) et un test dans **ce** binaire, qui lit le
  manuel par `common/manuel.rs` et compare les deux phrases chiffrées à la partition (AC-T1). Deux expressions
  régulières, aucune autre garde de texte. **Retrait pré-déclaré** si une passe montre un recyclage qui en naît.
- **Écartées** : recompter sans garde (quatrième dérive assurée) ; une garde dans `textes_coherents.rs` (elle ne voit
  pas `LIB_ROUTES`, binaire distinct) ; retirer les nombres du manuel (la phrase « comptées et suivies » perdrait sa
  preuve).
- **Réversible** : oui.

### C-32-11-11 — La description détaillée, manuel d'administration : une sous-section en tête de `sec:mcp`, par renvois

- **Contexte** : décision de Guy (« ajoute la description détaillée du fonctionnement de MCP ») ; la 32-7 pose
  `sec:mcp` et ses sous-sections, la 32-6c, la 32-8c, la 32-9 et la 32-10b en écrivent d'autres.
- **Retenu** : `sec:mcp-fonctionnement`, première de `sec:mcp` : ce qu'est MCP, le parcours d'une requête dans l'ordre
  de `mcp/mod.rs`, les portées, les prompts, la validation, ce que Kesh garantit et ne garantit pas — chaque point
  détaillé **par renvoi** aux sous-sections existantes, aucune seconde copie (matrice, configuration, écran des envois).
- **Écartées** : un chapitre à part hors de `sec:mcp` (deux lieux pour un même sujet) ; tout réécrire dans la
  description (dérive avec les sections des autres stories).
- **Réversible** : oui.

### C-32-11-12 — La description détaillée, manuel utilisateur : une section entre *Traçabilité* et *FAQ*

- **Contexte** : Guy veut la description aux deux manuels ; le lecteur du manuel utilisateur est un Comptable, qui
  n'administre ni les clés ni les envois.
- **Retenu** : `sec:agent-mcp`, plus courte : ce qu'est MCP, ce que l'agent lit, ce qu'on montre avant l'envoi (par un
  Administrateur), invoquer un prompt, reconnaître l'agent au journal (#776), ce que Kesh garantit — renvoi au manuel
  d'administration par le **titre** de section (un `\ref` ne traverse pas les manuels).
- **Écartées** : renvoyer le Comptable au seul manuel d'administration (Guy demande les deux) ; recopier la section
  d'administration (deux textes à tenir).
- **Réversible** : oui.

### C-32-11-13 — *Brancher un agent* après `sec:mcp-client`, qui reste la seule configuration

- **Contexte** : C-32-7-22 donne `sec:mcp-client` à la 32-7 et demande que la 32-11 l'amende en place.
- **Retenu** : `sec:mcp-brancher`, le parcours tel que joué, renvoie à `sec:mcp-client` pour la configuration ; la 32-11
  retire « non éprouvée » des seuls gestes éprouvés et corrige ce que l'essai réfute.
- **Écartées** : une seconde section de configuration ; fondre le parcours dans `sec:mcp-client`.
- **Réversible** : oui.

### C-32-11-14 — `docs/api-external.md` § 6.4 : le serveur intégré d'abord, la voie générique qualifiée

- **Contexte** : `:185` nie un serveur MCP natif ; `:187-205` conseille un serveur MCP générique avec une clé `read` ;
  `admin-manual.tex:2963` dit qu'une clé d'API donnée à l'agent lit **sans** validation.
- **Retenu** : § 6.4 renvoie au serveur intégré et à ses clés ; la voie générique reste décrite comme possible, avec ce
  qu'elle perd (validation avant envoi, journal des appels MCP) ; § 9 perd « serveur MCP Kesh-natif » de sa phrase hors
  périmètre, rien d'autre.
- **Écartées** : supprimer la voie générique (elle marche, et un utilisateur la rencontrera) ; réécrire tout le § 9
  (hors du périmètre ; ses bornes « v0.2 » sont un défaut voisin, non mécanique ici).
- **Réversible** : oui.

### C-32-11-15 — `README.md` et la feuille de route du site : deux affirmations réécrites, rien d'autre

- **Contexte** : fiches 32-6 (R13) et 32-10a (C-32-10a-29) : « montrées avant de partir » est faux pour une clé
  dispensée ; « aucune écriture par l'agent » est faux depuis `proposer_prompt`.
- **Retenu** : « montrées à l'Administrateur avant l'envoi sous une clé à validation requise (le défaut) ; une clé
  dispensée envoie sans montrer, chaque envoi journalisé » ; « l'agent n'écrit rien dans les livres ; il peut proposer un
  prompt, qu'un Administrateur approuve ou refuse ». Même chose en anglais au site.
- **Écartées** : réécrire toute la ligne (elle est juste pour le reste) ; attendre la release (elles mentent dès que le
  code fusionné les dément).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, R1)** : une troisième affirmation se réécrit — « en lecture seule » devient « en lecture seule des livres » (`README.md:233`), « read-only » « read-only on the books » (`roadmap.html:375`) —, `proposer_prompt` écrivant une version de prompt ; la même formule « a built-in, read-only MCP server » est corrigée à `roadmap.html:223` et `:43`, à ce seul endroit de chaque ligne ; le reste de `:43` reste à la release (point 5).

### C-32-11-16 — Le ✅ et la section *Fonctionnalités* à la release ; le CHANGELOG, une ligne sans date

- **Contexte** : la feuille de route ne passe à ✅ qu'à la publication ; la section *Fonctionnalités* du `README.md` ne
  nomme pas le serveur MCP ; les stories de l'epic écrivent leurs entrées `Ajouté` sans date.
- **Retenu** : la 32-11 laisse 🚧 et *Fonctionnalités* à l'audit des documents de la release ; elle ajoute une entrée
  `Ajouté` (manuels, `api-external.md`), sans date.
- **Écartées** : passer à ✅ (le serveur n'est pas publié) ; aucune entrée (la documentation de branchement est une
  nouveauté visible).
- **Réversible** : oui.

### C-32-11-17 — La brochure : le serveur MCP sort de la *Vision* et entre aux fonctions livrées

- **Contexte** : `marketing-brochure.tex:480` le promet en *Vision (v0.4+)* ; `:399` nomme l'API à clés.
- **Retenu** : retirer « MCP server » de `:480` ; une ligne aux fonctions livrées, datée `(v0.16.0)`, sans promesse
  d'écriture.
- **Écartées** : laisser la vision et ajouter la ligne (contradiction) ; réécrire la section *Vision* (hors périmètre).
- **Réversible** : oui.

### C-32-11-18 — Hermes n'est nommé nulle part

- **Contexte** : décision de Guy du 2026-10-25 au soir (epic, l. 36) ; la fiche 32-7 l'écrivait encore à l'AC-18 (8)
  avant sa réponse.
- **Retenu** : aucun texte de la 32-11 ne nomme Hermes ; si la 32-7 en a laissé une mention au manuel, la 32-11 la
  retire (`grep -n -i hermes`, sortie citée).
- **Écartées** : garder une mention « non éprouvé » de Hermes (Guy l'a sorti du projet).
- **Réversible** : oui.

### C-32-11-19 — Le binaire du worktree, non l'image ; la release rejoue un parcours court avec l'image

- **Contexte** : l'image 0.16.0 n'existe qu'à la release ; la 31-4 a substitué une image locale, déclarée.
- **Retenu** : le parcours éprouve le binaire du worktree (même `main.rs`, même pool en lecture seule) ; la fiche
  recommande à la release de rejouer P1 à P8 avec l'image publiée (à porter par l'orchestrateur dans la PR de release,
  avec A6 de la rétrospective 31).
- **Écartées** : construire une image locale pour la 32-11 (coût de construction, et ce n'est pas l'image publiée non
  plus) ; attendre l'image (la 32-11 précède la release).
- **Réversible** : oui.

### C-32-11-20 — La restauration par l'assistant d'une installation neuve, sur une seconde instance

- **Contexte** : Guy installera la 0.16.0 à neuf (epic, *Réponses de Guy*) ; l'aller-retour des tables est prouvé par
  test (`admin_backup_e2e.rs:323`) ; rien ne rebranche un client après une restauration.
- **Retenu** : seconde instance, base vide, *Restaurer une sauvegarde* par l'assistant (31-1b), constats à l'écran, puis
  Claude Code avec la même clé ; la révocation de P11 sur la première instance seulement, après l'export.
- **Écartées** : restaurer sur l'instance en place (pas le chemin de Guy) ; un test automatisé de plus (l'aller-retour
  est déjà testé ; ce qui manque est le client rebranché).
- **Réversible** : oui.

### C-32-11-21 — Les limites connues : citées si l'issue est ouverte, par renvoi, à l'endroit fixé

- **Contexte** : #759, #761, #765, #768, #770, #773, #776 portent des limites ; certaines sections d'autres stories en
  disent déjà une partie.
- **Retenu** : la table de l'AC-C6 fixe le lieu de chacune ; une issue fermée à l'étape 0 sort ; une limite déjà dite
  ailleurs se renvoie, elle ne se recopie pas ; #761 n'entre pas au manuel (tests).
- **Écartées** : une liste générale « limites connues » (doublons, dérive) ; ne rien citer (#776 demande que « les deux
  manuels le disent »).
- **Réversible** : oui.

### C-32-11-22 — Le rejeu depuis le PDF est joué par l'agent, sur une démonstration neuve

- **Contexte** : l'epic veut le parcours « rejoué depuis le PDF » ; la 31-4 l'a fait en T8 (l. 749-758).
- **Retenu** : T-8 : un second passage de P1 à P11, base `kesh_3211_t8`, en suivant le PDF ; les écarts corrigent le
  manuel ; un défaut du produit suit C-32-11-3.
- **Écartées** : faire jouer le rejeu par une passe de revue (une lentille ne lance rien) ; le confier à Guy seul (Q1 :
  il ne rejoue que l'interactif).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur ; F8 ; R11)** : le rejeu, **un seul**, repasse P0 à P11 (P0 compris) : P0 et les requêtes depuis la fiche, les gestes d'écran depuis le PDF ; ses corrections du manuel n'appellent pas de troisième passage ; les gestes de Claude Code du PDF se rejouent à la recette de Guy.

### C-32-11-23 — L'outil appelé est le bilan, sous une clé `books:read` seule ; un prompt personnalisé avant l'export

- **Contexte** : l'epic demande « une clé `books:read`, un prompt standard, un outil appelé » ; la restauration doit
  rapporter les prompts **et leur historique**.
- **Retenu** : la demande du bilan (l'agent appelle `kesh_exercices` puis `kesh_bilan`, ou ce qu'il choisit : chaque
  appel de données déposé) — la consultation d'un rapport montre aussi #776 au journal ; une version personnalisée d'un
  prompt standard avant P10, pour que E ait un historique à constater.
- **Écartées** : `kesh_plan_comptable` seul (un appel, mais rien de #776 à observer) ; plusieurs clés (hors du périmètre
  clos).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P1, F1)** : l'agent n'appelle plus `kesh_exercices` ni « ce qu'il choisit » : la consigne de P6, écrite à la fiche, lui donne l'identifiant d'exercice lu en P0 et lui fait appeler `kesh_bilan` seul — **un** envoi, validé modifié en P7.

### C-32-11-24 — Les libellés piégés : un appel conditionnel, sur un binaire de parcours

- **Contexte** : Q2 de la fiche 32-9 (C-32-9-14) : les pièges sont derrière une fonctionnalité de compilation des seuls
  tests ; la 32-9 demande si la 32-11 les rencontre avec un vrai client.
- **Retenu** : P-pièges, un appel, **seulement si Guy répond oui** (Q3) ; binaire compilé avec la fonctionnalité,
  démonstration publique inchangée ; constat écrit, rien corrigé.
- **Écartées** : les jouer d'office (la décision appartient à Guy, ouverte à la 32-9) ; les ajouter à la démonstration
  publique (C-32-9-14 l'écarte).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur)** : P-pièges sort de la liste des gestes et passe à l'Annexe : observer un agent réel demande un agent réel ; la Q3 ne se pose que si Guy répond oui au point 1. Sinon, angle mort écrit.

### C-32-11-25 — L'amorçage P0 : trois écritures synthétiques par l'API, avec un témoin positif

- **Contexte** (validation P1, R1, HIGH) : la démonstration ne porte aucune écriture (fait 16) ; le bilan n'a aucun
  montant non nul, et le contrôle « aucun montant ne part avant la validation » (AC-A5 (b)) rendait « rien » quel que
  soit le serveur — un test muet.
- **Retenu** : un geste P0, avant P1, joué depuis la fiche : une clé d'API `read-write` créée à l'écran, trois
  `POST /api/v1/journal-entries` aux montants fixés par la fiche (bilan attendu : 1020 à 20 432.05, 2800 à 17 345.10),
  la clé révoquée ensuite, le bilan constaté à l'écran ; P0 parle à Kesh directement, avant le lancement de `socat`.
  Un **témoin positif** : le même `grep` doit trouver les montants dans la part de `socat.log` postérieure à P7.
- **Écartées** : la saisie à l'écran (*Saisie des écritures*, trois formulaires : plus de gestes d'écran et de
  captures pour rien que le manuel ne décrit) ; un SQL direct (contourne l'application, et l'AC-A1 charge la
  démonstration par l'écran) ; la fonctionnalité `pieges` de `kesh-seed` (elle ne sert que P-pièges, Q3).
- **Réversible** : oui.

### C-32-11-26 — L'isolation du client : un dossier de lancement neuf, un fichier MCP jetable, un plafond

- **Contexte** (validation P1, F2) : lancé depuis le worktree, l'agent d'essai chargerait le `CLAUDE.md` de Kesh et
  ses skills ; un dossier de configuration jetable retirerait l'authentification de Guy ; rien ne bornait le coût.
- **Retenu** : `kesh-gate-logs/3211/cwd/`, neuf, hors de tout dépôt et sans `CLAUDE.md` parent ; `--mcp-config` et
  `--strict-mcp-config` ; aucun dossier de configuration jetable — l'authentification et le magasin de sessions de Guy
  restent, déclarés ; `--allowedTools` et `--disallowedTools` ; `--max-turns 8` par session, un budget si `claude -p`
  en offre un, et arrêt si aucun plafond n'existe. Tout comportement de `claude` se constate à l'étape 0.
- **Écartées** : le dossier de configuration jetable avec une clé d'API facturée à part (change la facturation et le
  montage, Q1) ; neutraliser `~/.claude/CLAUDE.md` et les hooks de Guy (aucune option connue sans retirer
  l'authentification ; ce qui reste chargé se cite au journal).
- **Réversible** : oui (Q1).
- **Amendé le 2026-10-27 (validation P2, décision de l'orchestrateur)** : **retiré du montage par défaut** — l'agent ne lance pas `claude` (C-32-11-28) ; ce montage est repris en esquisse à l'Annexe, avec la dérogation qu'il demande, pour le cas où Guy répond oui au point 1.

### C-32-11-27 — L'oracle du contrat : `CLAUSE` pour ce que la spécification exige, `DECISION_KESH` pour le reste, attendus en littéraux

- **Contexte** (validation P1, F4, R2, R3) : les listes closes de clés, la présence d'`isError`, l'absence de session
  et la forme de l'accusé étaient écrites comme des clauses ; la spécification les laisse au serveur (session « MAY »,
  spike § 1.1 ; `isError` et propriétés supplémentaires à constater au schéma). Et les aides `attendu_*` de
  `tests/common/mcp.rs` tirent leur attendu de `mcp::textes`.
- **Retenu** : chaque assertion rangée sous `CLAUSE` (clés requises, types, valeurs imposées) ou `DECISION_KESH`
  (listes closes, `isError` présent, pas de `Mcp-Session-Id`, K10, K12, K17, K19, K22, `readOnlyHint`) ; attendus en
  littéraux dans le test ; aides `attendu_*` et `mcp::textes` interdits au contrat, prouvé par les mutations M17 et
  M18 (un mot changé dans les textes de production fait rougir K10), sans garde de texte nouvelle ; tout ce qui touche
  au schéma officiel constaté à l'étape 0 sur la page de la spécification.
- **Écartées** : une garde qui lirait le source de `mcp_contrat.rs` (garde de texte nouvelle, contraire à la clôture
  de la fiche) ; le schéma officiel validé par `jsonschema` (Q2).
- **Réversible** : oui.
- **Amendé le 2026-10-27 (validation P2, R5 = F2, F5, R9, F11)** : rangement **par défaut** sous `DECISION_KESH` de tout ce qui est douteux, marqué « à constater à l'étape 0 sur la page » ; `CLAUSE` seulement pour ce que la spike cite en MUST et pour les exigences de base nommées, chacune à citer. Rangés : K3 (une version servie en `CLAUSE`, la plus récente en `DECISION_KESH`), K8 à K10 et K14 (blocs typés en `CLAUSE`, du texte seulement en `DECISION_KESH`), K11 (`-32602`, exemple de la page), K15 (403 en `CLAUSE`, corps en `DECISION_KESH`), K16 (401, `DECISION_KESH`), K17 (400 en `CLAUSE`, corps en `DECISION_KESH`), K18 (SHOULD, `DECISION_KESH`), K20 (`GET` 405 en `CLAUSE`, `DELETE` et `Allow` en `DECISION_KESH`), K21 (400 en `DECISION_KESH`, la clause « 202 ou statut d'erreur » à constater, jamais « 400 » seul). Preuve des littéraux étendue : M21 (`INSTRUCTIONS` → K1), M22 (texte du 400 → K17) ; M19 (K13) et M20 (K14, témoin d'une liste non vide) ajoutées.

### C-32-11-28 — Le parcours en client HTTP, par `curl` ; l'agent ne lance pas `claude`

- **Contexte** (validation P2, décision de l'orchestrateur) : le pilotage de `claude -p` par l'agent produisait la
  plupart des défauts de la P2 — sessions non bornées (F1 = R2), mal comptées (F4 = R3), écritures dans `~/.claude*`
  qui dérogent à `docs/agents-orchestration.md` § 2.1 (F3 = R6) — après avoir produit ceux de la P1 : un recyclage.
- **Retenu** : l'agent joue P0 à P11 et la restauration **en client HTTP** : chaque requête JSON-RPC (`initialize`,
  `notifications/initialized`, `tools/list`, `prompts/list`, `prompts/get`, `tools/call`, `kesh_lire_envoi`) est un
  fichier écrit avant l'envoi, envoyé **une fois** par `curl` à l'instance de démonstration, sa réponse gardée ; neuf
  requêtes au parcours, quatre à la restauration, neuf au rejeu ; un envoi par instance, **par construction**. `curl`
  est déclaré au § *Gates*.
- **Écartées** : un **test d'intégration Rust** qui piloterait le routeur — c'est déjà le cas K10 du contrat, qui joue
  dépôt, décision et relecture sans écran, en CI ; le parcours, lui, entrelace ces requêtes avec des gestes d'écran sur
  **la même instance** (P5, P7, P9, P10) : un test Rust contre un serveur réel devrait attendre un geste du navigateur
  (un harnais de plus) ou s'ignorer en CI (un test muet, `#[ignore]` proscrit) ; le pilotage de `claude -p` (variante
  de l'Annexe, point 1).
- **Réversible** : oui (point 1).

### C-32-11-29 — La recette de Guy : Claude Code joué depuis le PDF, et la capture de ses requêtes

- **Contexte** (validation P2, décision de l'orchestrateur) : sans `claude` lancé par l'agent, le comportement du
  client (observations (a) à (h) de la spike), les gestes interactifs et les requêtes réelles ne s'observent qu'avec
  Claude Code ouvert par Guy.
- **Retenu** : à la recette de la 0.16.0, déjà prévue, vingt à trente minutes : la capture en une étape (`socat -v`,
  sortie dans `kesh-gate-logs/3211/capture-recette.log`), puis les gestes de Claude Code depuis le PDF ; l'orchestrateur
  tranche les observations et compare les requêtes aux clauses ; une contradiction s'ouvre en issue ; les marques
  « non éprouvé » se retirent ensuite. Le test de contrat de la CI ne dépend pas de la capture (AC-B5). Le point 1 pour
  Guy devient une question : accepte-t-il qu'un agent lance `claude -p` sur son compte (dérogation nommée) ?
  Recommandation : non.
- **Écartées** : une fixture de la capture versionnée par la 32-11 (elle attendrait la recette, après le
  développement) ; garder le pilotage de `claude -p` par défaut (C-32-11-28).
- **Réversible** : oui (point 1).

*(Amendé le 2026-10-27, validation P4 de la fiche, A2 : « déjà prévue » (C-32-11-29, Retenu) est devenu vrai depuis la P3 — la recette de Guy est **inscrite à la § *Release v0.16.0* de l'epic** (ligne datée du 2026-10-27), non plus seulement attendue. Le texte du choix est laissé tel quel.)*

---

*Entrées du développement (Opus 5.5, en autonomie, 2026-10-27, worktree `kesh-32-10a`, branche `story/32-11`).*

### C-32-11-30 — Les gestes d'écran par un pilote Playwright jetable, et les secrets masqués dans les captures

- **Contexte** (AC-A1, AC-A4) : les gestes d'écran se jouent par le navigateur ; le secret d'une clé est affiché une
  fois, et les captures vivent hors du dépôt.
- **Retenu** : un pilote jetable (`kesh-gate-logs/3211/pw.cjs`, un fichier par geste sous `gestes/` puis `t8/gestes/`),
  Chromium de Playwright, thème clair ; le secret n'est écrit qu'au fichier d'en-tête et **masqué dans la capture**
  (« kesh_mcp_(masqué pour la capture) ») ; au rejeu T-8, le menu *Administration → …* est cliqué comme le PDF le dit.
- **Écartées** : la capture du secret en clair (l'AC-A2 P2 dit « le secret se lit une fois (capture) ») — une capture
  porterait un secret, même de démonstration, que rien n'oblige à garder.
- **Réversible** : oui.

### C-32-11-31 — Le rangement des cas K après lecture de la révision 2025-11-25

- **Contexte** (AC-B2, § *À recontrôler*) : les pages *basic*, *lifecycle*, *transports*, *ping*, *tools*, *prompts*,
  *authorization* et *schema* de la révision `2025-11-25`, et la spécification JSON-RPC 2.0, lues le 2026-10-27.
- **Retenu** : **promus `CLAUSE`** — K5 (« If the server accepts the input, the server **MUST** return HTTP status code
  202 Accepted with no body »), le `GET` de K20 (« The server **MUST** either return `Content-Type: text/event-stream`
  […] or else return HTTP 405 Method Not Allowed »), K21 sous la forme « 202 sans corps, ou un statut d'erreur 4xx »
  (« If the server cannot accept the input, it **MUST** return an HTTP error status code (e.g., 400 Bad Request) »),
  K13 (*basic* : « All messages […] **MUST** follow the JSON-RPC 2.0 specification » ; JSON-RPC § 5.1 : `-32601` « Method
  not found »), les clés requises du schéma (*basic* : le schéma est « the source of truth for all protocol messages
  and structures ») ; **restés `DECISION_KESH`** — K3 « la plus récente » (SHOULD), K11 `-32602` (exemple seulement),
  K14 `role` (énumération du schéma, sans MUST), le corps sans `id` de K15 (MAY), K16 (*authorization* :
  « Authorization is **OPTIONAL** »), K18 (SHOULD), K19, le `DELETE` de K20 (MAY), K22 ; la branche `None` de l'AC-B3 exige
  l'`id` **absent** (`JSONRPCErrorResponse.id?: RequestId`, `RequestId` = `string | number`, jamais `null`).
- **Écartées** : promouvoir `role` sur la foi du type du schéma (la fiche n'admet qu'une phrase en MUST).
- **Réversible** : oui (une révision nouvelle se relit).

### C-32-11-32 — L'outil de données de K9 et K10 : `kesh_plan_comptable`, sur un compte que pose le test

- **Contexte** (AC-B4) : K9 et K10 veulent un outil de données de la 32-9a et une donnée posée par le test.
- **Retenu** : `kesh_plan_comptable`, sans argument, sur un compte « Compte du contrat CONTRAT_3211 » posé par
  `common::mcp::compte` ; aucune écriture à fabriquer, et le texte de la donnée se cherche tel quel.
- **Écartées** : `kesh_bilan` (il faudrait un exercice et des écritures en fixture, pour la même preuve).
- **Réversible** : oui.

### C-32-11-33 — Les motifs du PDF remplacés quand la base les rendait déjà

- **Contexte** (AC-C9) : à l'étape 0, les PDF de `origin/main` (`186302d9`) rendent déjà `2025-11-25`, `#776` (admin et
  utilisateur), `Brancher un agent` (admin, dans `sec:mcp-client`) et `129 des 135`.
- **Retenu** : motifs neufs, chacun 0 à la base et 1 au PDF régénéré — admin : `Comment fonctionne le serveur MCP`,
  `Brancher un agent, pas à pas`, `parcours d’une requête`, `Après une restauration`, `issue #759` ; utilisateur :
  `L’agent IA et le serveur MCP`, `/kesh:kesh-assistant` ; brochure : `Serveur MCP intégré` (0 → 1) et `MCP server`
  (1 → 0). Tous par `grep -cF` (motifs fixes, apostrophe typographique écrite telle quelle), sous `LC_ALL=fr_FR.UTF-8`.
  AC-C4 déclarée **déjà vraie** (la 32-10b a écrit « 129 des 135 », recompté).
- **Écartées** : `grep -cE 'L.{1,3}agent…'` (quantificateur sur la ligne aplatie, déconseillé par le prompt).
- **Réversible** : oui.

### C-32-11-34 — Le message « Réglages enregistrés. » paraphrasé, non cité

- **Contexte** (AC-T2) : la garde G2 `aucun_renvoi_au_menu_reglages` (`textes_coherents.rs`) interdit le mot
  « Réglages » capitalisé dans les manuels ; citer le statut de l'écran l'a fait rougir.
- **Retenu** : « l'écran confirme l'enregistrement et date la dernière modification » — la garde reste sans
  modification.
- **Écartées** : inscrire l'exception à la garde (l'AC-T2 ne l'admet que justifiée ; une paraphrase exacte suffit).
- **Réversible** : oui.

### C-32-11-35 — La forme d'un prompt dans Claude Code : `/kesh:kesh-assistant (MCP)`

- **Contexte** (AC-C2, AC-C7) : la documentation de Claude Code (spike § 2.1, l. 253) donne
  `/servername:promptname (MCP)` ; `sec:mcp-client` nomme le serveur `kesh` dans `claude mcp add`.
- **Retenu** : `/kesh:kesh-assistant (MCP)`, « le nom du serveur donné à `claude mcp add` précédant celui du prompt »,
  marqué « non éprouvé ».
- **Écartées** : `/kesh-demo:…` (nom du fichier jetable de la spike, absent du manuel).
- **Réversible** : oui (recette de Guy).

### C-32-11-36 — Les bases du parcours supprimées à la fin de chaque phase

- **Contexte** (AC-E2 : « les deux bases supprimées » ; AC-A6 : base `kesh_3211_t8`).
- **Retenu** : `DROP DATABASE` de `kesh_3211` et `kesh_3211_r` après la restauration, de `kesh_3211_t8` après le
  rejeu, par l'utilisateur `kesh` ; instances arrêtées par PID. `kesh_e2e_3211` reste pour l'E2E.
- **Écartées** : garder les bases jusqu'à la revue (les preuves sont les journaux, les réponses et les captures).
- **Réversible** : non (les bases sont jetables ; les preuves restent sous `kesh-gate-logs/3211/`).

*Entrées de la remédiation de la revue de code P1 (Opus 5.5, 2026-10-27).*

### C-32-11-37 — Le `Content-Type` des réponses au contrat (B1)

- **Contexte** : aucun cas ne vérifiait l'en-tête de la réponse, alors que la page *transports* en fait un MUST.
- **Retenu** : dans `enveloppe_conforme`, pour la réponse à une requête dont l'`id` est lu, `CLAUSE` — « If the input is
  a JSON-RPC *request*, the server **MUST** either return `Content-Type: text/event-stream`, to initiate an SSE stream,
  or `Content-Type: application/json`, to return one JSON object. » (*Sending Messages to the Server*, point 5) ; pour
  toute réponse JSON, `DECISION_KESH` : exactement `application/json` (32-6a, jamais de SSE). Mutation **MR1**
  (`reponse.rs`, `jsonrpc` rend `text/plain`) : tuée par K6 et tous les cas à `id`.
- **Écartées** : vérifier l'en-tête dans un cas seul (K1) — l'enveloppe le voit sur chaque réponse.
- **Réversible** : oui.

### C-32-11-38 — L'`id` d'une erreur dont l'`id` n'a pu être lu : absent ou `null` (B2 = A6)

- **Contexte** : JSON-RPC 2.0 § 5 (« If there was an error in detecting the id in the Request object (e.g. Parse
  error/Invalid Request), it MUST be Null. ») et le schéma MCP (`JSONRPCErrorResponse.id?: RequestId`, `RequestId` =
  `string | number`) se contredisent sur la forme.
- **Retenu** : `CLAUSE` sur ce que les deux admettent ensemble — `id` absent **ou** `null` ; `DECISION_KESH`
  (C-32-6-42) : absent. Mutation **MR2** (`CORPS_REQUETE_INVALIDE` portant `"id":null`) : la `CLAUSE` reste verte, la
  `DECISION_KESH` rougit (K19, K21) — la frontière tient.
- **Écartées** : garder « absent » sous `CLAUSE` (une décision de Kesh, non une exigence).
- **Réversible** : oui.

### C-32-11-39 — Toute assertion porte une marque ; K13 et le 4xx de K21 rangés (B4, B5 = E2, A6)

- **Retenu** : deux marques de plus, hors protocole — `TEMOIN` (ce qui empêche un passage à vide : K9, K14) et
  `MONTAGE` (amorçage, route de décision de l'Administrateur) ; le statut 200 d'une réponse JSON-RPC est
  `DECISION_KESH` (la page impose le `Content-Type`, non le statut) ; K8 vérifie désormais le contenu (`DECISION_KESH` :
  un bloc, l'en-tête des données, `structuredContent` aux clés `delaiInstructionS`, `lectureSeule`, `version`) ; K13 :
  `-32601` passe sous `DECISION_KESH` (le code n'est donné que dans un tableau de JSON-RPC § 5.1, sans MUST) ; K11 prend
  l'`id` 11 ; K21 : la `CLAUSE` admet « un statut d'erreur HTTP » (4xx ou 5xx), le 400 restant `DECISION_KESH`.
- **Réversible** : oui.

### C-32-11-40 — Ce que la remédiation P1 ne prend pas

- **E5** (K9, K12, K16, K20, K21, K22 sans mutation dédiée) : angle mort déjà écrit au § *Ce que les gardes ne voient
  pas* ; MR2 touche désormais K21. **A3** (le `.keshbackup` de P10 et celui du rejeu restés sous
  `kesh-gate-logs/3211/export/` et `t8/export/`) : supprimer demanderait `rm`, interdit ; fichiers hors dépôt, clés
  révoquées — à retirer par l'orchestrateur ou Guy. **A2** (capture 13 du premier passage blanche) : non refaite — la
  première instance et sa base n'existent plus ; l'état *En attente* est établi par `parcours.log` (P7, 21:13:51) et par
  la capture 13 du rejeu (`t8/captures/13-P7-envoi-en-attente.png`).
- **Réversible** : oui.

*Entrées de la remédiation de la revue de code P2 (Opus 5.5, 2026-10-27).*

### C-32-11-41 — Le fondement de l'`id` absent, cas par cas ; K16 en décision de Kesh (R1)

- **Contexte** (revue P2, R1) : la branche « `id` absent ou `null` » d'`enveloppe_conforme` marquait `CLAUSE` quatre
  appelants ; aucune phrase ne couvre K16 (401 sans `Authorization`). La requête de K16 est bien formée et son `id`
  lisible ; Kesh refuse **avant de lire le corps** (`transport.rs` : la clé à l'étape 7, le corps à l'étape 10 ;
  C-32-6-27). BASIC, *Error Responses*, dit : « Error responses **MUST** include the same ID as the request they
  correspond to (except in error cases where the ID could not be read due a malformed request). »
- **Retenu** : `IdAttendu::{Lu(n), SansId(Fondement)}` et un `Fondement` par appelant — K15 `OrigineRefusee`
  (*Security Warning*, « MAY comprise a JSON-RPC *error response* that has no `id` »), K19 `RequeteInvalide` (JSON-RPC
  § 5, « … it MUST be Null »), K21 `EntreeNonAcceptee` (*transports*, point 4, phrase entière citée, « … that has no
  `id` ») gardent la `CLAUSE` « absent ou `null` » et la `DECISION_KESH` « absent » ; K16 `RefusAvantLeCorps` : la seule
  `DECISION_KESH` « absent », la tension avec BASIC écrite au test. Mutation **MR3** (`TEXTE_NON_AUTHENTIFIE` portant
  `"id":16`) tuée par cette `DECISION_KESH` (« 401 sans id, refusé avant la lecture du corps ») ; MR2 tuée par la
  `DECISION_KESH` de K19 et K21 (messages relevés, `mutations/campagne-p2-messages.log`).
- **Le défaut** : à la lettre de BASIC, le corps JSON-RPC du 401 devrait porter l'`id` de la requête. Le choix de Kesh
  (ne rien lire d'une requête non authentifiée) est défendable ; il n'est pas corrigé ici (réponse 4 de Guy) — **issue
  demandée** pour arbitrage (un 401 sans corps JSON-RPC, ou l'`id` lu après le refus).
- **Écartées** : garder K16 sous la `CLAUSE` commune (une phrase manque) ; corriger le transport (hors de la 32-11).
- **Réversible** : oui.

### C-32-11-42 — K8 vérifie la version et le JSON du texte (R2)

- **Retenu** : `structuredContent.version` égale `env!("CARGO_PKG_VERSION")` lu par le crate de test, et le texte
  porte, après l'en-tête et une ligne vide, un JSON égal à `structuredContent`. `DECISION_KESH` (32-9a, AC-a3, AC-a8).
- **Réversible** : oui.

### C-32-11-43 — Le manuel : ce qu'un appel écrit, et le plafond par clé (R3, R4)

- **Retenu** : chapeau de `sec:mcp` — « aucun outil n'écrit dans les livres ; un appel n'écrit que sa trace (la
  dernière utilisation de la clé, son entrée au journal d'audit) et, sous une clé à validation requise, l'envoi qu'il
  dépose, et `proposer_prompt` une version de prompt à valider » ; symptôme propagé à `sec:mcp-fonctionnement` (« Un
  seul outil écrit autre chose que la trace de l'appel et l'envoi qu'il dépose ») et à `sec:mcp-outils` (« un seul
  outil qui écrit autre chose que la trace de l'appel et l'envoi qu'il dépose ») ; `sec:mcp-envois` — « un envoi
  au-delà ne s'y voit plus — en attente compris, si plus de 200 attendent, le plafond de 20 envois en attente valant
  par clé ».
- **Réversible** : oui.
