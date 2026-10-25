# Epic 32 — registre des choix autonomes de la Story 32-6c

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** dans
`epic-32-choix-autonomes.md` au merge. Fiche : `32-6c-journal-filtre-adresse.md`. Numérotation `C-32-6c-k`. Les
numéros de ligne cités sont ceux de `c5287dec`. Chaque choix : contexte, option retenue, alternatives écartées,
réversibilité.

---

### C-32-6c-1 — Une story à part, de diagnostic, sans changement de comportement

- **Contexte** : Guy, 2026-10-25 : « on vérifiera lors de la mise à jour : ajoute le nécessaire dans les logs au cas
  où ». La 32-6a et la 32-6b sont fusionnées (`7e47bff4`, `cd86dc74`) ; aucune décision du filtre n'est journalisée
  (fiche, fait 2), alors que C-32-6-4 l'écrivait.
- **Retenu** : une clé neuve `32-6c-journal-filtre-adresse`, après la 32-6b, qui ajoute un évènement et son mode
  d'emploi, sans toucher aux réponses ni à la décision. Trois modules (`mcp`, `logging` par ses tests, le manuel).
- **Écartées** : l'amender dans la 32-7 (écran et manuel MCP : la mise à jour du NAS peut précéder sa fin, et la 32-7
  est déjà chargée) ; un correctif hors story (il touche une garde de sécurité : il mérite une boucle de revue).
- **Réversible** : oui.

### C-32-6c-2 — Un évènement par décision et par société, émis par `decider` ; `Ecartee` garde le sien

- **Contexte** : la décision est prise par société (`reseau.rs:104`, `transport.rs:91-94`) ; le pair, les en-têtes et
  le client vivent dans `decider`. L'orchestrateur demande « un seul évènement par décision du filtre ».
- **Retenu** : l'évènement naît dans `decider`, à ses sorties `Admis` et `Refuse` — un par société activée et par
  requête. `Ecartee` garde son `error!` (`reseau.rs:110-114`) et n'en reçoit pas d'autre : c'est déjà l'évènement de
  sa décision, et aucun client ne doit être calculé quand un mandataire ne s'analyse pas.
- **Écartées** : un évènement **par requête** au transport, après `servies` (il devrait résumer N décisions dont le
  client retenu diffère d'une société à l'autre — les mandataires sont par société — et rapatrier du transport ce que
  `reseau` sait) ; deux évènements (en-têtes, puis verdict), qui obligent à recouper deux lignes. **Un seul suffit** :
  il porte les six éléments que Guy veut lire.
- **Réversible** : oui.

### C-32-6c-3 — Une liste close de champs, dans un ordre fixe, vérifiée par égalité complète

- **Contexte** : l'évènement doit dire le pair, la présence et la forme de `X-Forwarded-For`, le statut de mandataire,
  le client retenu, le verdict et la présence de `X-Real-IP` ; et ne rien porter d'autre (contrainte de
  l'orchestrateur).
- **Retenu** : onze champs (`company_id`, `filtre`, `plages`, `pair`, `pair_mandataire`, `xff_lignes`, `xff_entrees`,
  `x_real_ip`, `forwarded`, `client`, `verdict`) et un message fixe ; les tests comparent la **chaîne complète** de
  chaque évènement de décision à la chaîne attendue. Une égalité ferme la classe « champ ajouté » sur les douze cas,
  là où un `contains` la laisserait ouverte. `plages` (le nombre d'entrées stockées) s'ajoute à la demande : sans lui,
  le cas « filtre activé, liste vide, personne admis » ne se lit pas.
- **Écartées** : des `contains` par champ (`pair=172.18.0.1` contient aussi `pair=172.18.0.10`, et un champ de trop
  passe) ; le nombre de mandataires déclarés (lisible à l'écran des réglages, sans valeur pour trancher).
- **Réversible** : oui (les tests suivent).
- **Annoté à la validation P1 (F1, R5)** : l'AC-T1 compte désormais **quatorze** cas (13 et 14 ajoutés) ; la
  ligne attendue commence par le message, puis les onze champs.

### C-32-6c-4 — Refus en `warn`, admission en `debug` (décision de l'orchestrateur, retenue)

- **Contexte** : un refus est le cas qui appelle une action ; une admission arrive à chaque requête d'un agent.
- **Retenu** : `warn` pour `Refuse` (niveau par défaut `info`, `logging.rs:129-131`), `debug` pour `Admis`. Les champs
  sont les mêmes aux deux niveaux.
- **Coût écrit** : chaque requête refusée écrit une ligne, sans limite de débit (Guy : pas de limite en 0.16.0) — même
  classe que le `warn!` par requête de la connexion limitée (`routes/auth.rs:190`). Question 1 pour Guy.
- **Écartées** : tout en `debug` (un poste légitime refusé ne se verrait qu'après avoir changé `RUST_LOG`) ; tout en
  `info` (une ligne par appel d'agent au niveau par défaut).
- **Réversible** : oui.
- **Confirmé par Guy le 2026-10-25** (point 1 : « oui »). Complément de la validation P1 (F3) : le manuel dit
  qu'un `warn` part par société qui refuse et que le cas DSM non déclaré ne s'écrit qu'en `debug`.

### C-32-6c-5 — Le filtre désactivé journalise le client qu'il retiendrait

- **Contexte** : `decider` rend `Admis` avant de calculer le client quand le filtre est coupé (`reseau.rs:119-121`) ;
  or le diagnostic le plus sûr se fait filtre coupé, pour ne pas couper l'agent pendant qu'on règle les mandataires.
  La consigne citait « le filtre désactivé (aucune plage, ou filtre coupé) » : **aucune plage n'est pas un filtre
  désactivé** — filtre activé et liste vide n'admet personne (AC-a8, test (o)) ; la fiche traite les deux cas
  séparément (cas 8 à 10 de l'AC-T1).
- **Retenu** : le client se calcule **avant** l'état du filtre, sans changer la décision (`Admis` filtre coupé, même
  avec un en-tête illisible : `client=aucun`) ; l'évènement part en `debug`, `filtre=false`.
- **Écartées** : ne rien journaliser filtre coupé (le diagnostic exigerait d'activer le filtre à l'aveugle).
- **Réversible** : oui. Risque : avancer le calcul peut changer le verdict par mégarde — mutation M15, cas 9.
- **Confirmé par Guy le 2026-10-25** (point 3 : « oui »).

### C-32-6c-6 — La forme des en-têtes, jamais leur valeur ; `Forwarded` avec `X-Real-IP`

- **Contexte** : une valeur d'en-tête est écrite par le client ; `/mcp` n'est jamais lu de `X-Real-IP` ni de
  `Forwarded` (C-32-6-5), et le manuel fait poser `X-Real-IP` à DSM (`admin-manual.tex:654`).
- **Retenu** : lignes et entrées de `X-Forwarded-For` comptées sur les octets (une ligne non ASCII compte, rien n'est
  converti en texte) ; présence de `X-Real-IP` **et** de `Forwarded` (le même cas — un en-tête de mandataire que `/mcp`
  ignore —, un booléen de plus) ; le client retenu est une adresse **analysée**. Ni `Authorization`, ni `Cookie`, ni
  `Origin`, ni corps.
- **Écartées** : la valeur brute de `X-Forwarded-For` (texte forgé au journal, et rien de plus utile que le client
  retenu) ; une classification de la forme d'une entrée illisible (port, `unknown`…) : un angle d'analyse de plus pour
  un cas que `client=aucun` signale déjà.
- **Réversible** : oui.
- **Confirmé par Guy le 2026-10-25** (point 2 : « oui »). Validation P1 (F1) : le comptage « sur les octets » et
  « valides ou non » est désormais gardé par les cas 13 et 14 et les mutations M17 à M19.

### C-32-6c-7 — Le niveau se règle par `RUST_LOG`, ciblé sur le filtre ; aucune configuration neuve

- **Contexte** : `RUST_LOG` est lu par `EnvFilter` (`logging.rs:104-131`) et transmis par les composes
  (`docker-compose.yml:91`) ; une directive de cible vaut pour le préfixe de la cible (`tracing-subscriber-0.3.23`,
  `filter/env/directive.rs:246`).
- **Retenu** : le manuel prescrit `RUST_LOG=info,kesh_api::mcp::reseau=debug`, puis le retour à `info` ; un test de
  `logging.rs` prouve la phrase (AC-8). Ni variable neuve, ni `.env.example` modifié.
- **Écartées** : une variable `KESH_MCP_JOURNAL_FILTRE` (une configuration de plus pour une vérification ponctuelle) ;
  `RUST_LOG=debug` (tout Kesh en `debug`) ; une ligne commentée à `.env.example` (le manuel suffit, un module de moins).
- **Réversible** : oui.
- **Annoté à la validation P1 (R1)** : `RUST_LOG` est déjà décrit au manuel (`admin-manual.tex:792`) ; la
  sous-section y renvoie, et dit que les variables du terminal l'emportent sur le `.env` (`:1641`).
- **Annoté à la validation P2 (F1, R4, F7)** : la précédence du terminal (`:1641`, écrite pour une machine de
  développement, sans `sudo`) n'est plus transposée au NAS ; la sous-section fait **remplacer** la ligne
  `RUST_LOG=info` du `.env` (`.env.example:253`), lancer `sudo docker compose up -d`, et vérifier l'effet par la
  présence des lignes `DEBUG` au journal.

### C-32-6c-8 — L'exemple de ligne du manuel se copie d'une sortie réelle

- **Contexte** : le journal standard met les champs texte entre guillemets et pas les `%` (fait 9) ; la capture des
  tests ne le fait pas.
- **Retenu** : la ligne d'exemple du manuel vient d'une sortie réelle du binaire, au développement ; adresses de
  documentation.
- **Écartées** : une ligne composée à la main ou depuis la capture (elle mentirait sur les guillemets).
- **Réversible** : oui.

### C-32-6c-9 — Le manuel : une sous-section sous *Monitoring et logs*, un renvoi depuis DSM, aucune prescription DSM

- **Contexte** : la section *Le serveur MCP* est à la 32-7 (`backlog`) ; le manuel fait poser `X-Real-IP`, que `/mcp`
  ignore ; rien n'établit ce que DSM pose (Guy).
- **Retenu** : `\label{sec:mcp-journal-filtre}` après § *Logs applicatifs*, un tableau de lecture, et une phrase à
  l'étape 4 de DSM qui y renvoie. Rien n'est conseillé à DSM avant la vérification sur le NAS (question 4).
- **Écartées** : attendre la 32-7 (la story livrerait un journal sans mode d'emploi) ; conseiller
  `X-Forwarded-For: $proxy_add_x_forwarded_for` à DSM (non vérifié).
- **Réversible** : oui.
- **Confirmé par Guy le 2026-10-25** (point 4 : « non » au conseil DSM ; point 5 : « oui », avec la description
  détaillée du fonctionnement de MCP, renvoyée à la 32-11). Validation P1 (R4) : la réserve sur la déclaration du
  pair comme mandataire est écrite dès la 32-6c (AC-9 (ii)).

### C-32-6c-10 — Le CHANGELOG ne change pas

- **Contexte** : le CHANGELOG de la 0.16.0 ne présente pas encore le serveur MCP (seulement « le futur serveur MCP »,
  `CHANGELOG.md:15-23`) ; la 32-11 l'annoncera.
- **Retenu** : aucune entrée ; le journal du filtre entrera dans l'annonce du serveur MCP.
- **Écartées** : une entrée isolée pour une fonction qu'aucune entrée ne présente encore.
- **Réversible** : oui.

### C-32-6c-11 — Le journal se lit dans le fichier de log ; la sortie standard, par un filtre de repli

- **Contexte** (validation P1, R2 = F2) : la sortie standard émet des codes couleur (`logging.rs:135`, sans
  `with_ansi(false)` ; `tracing-subscriber-0.3.23/src/fmt/fmt_layer.rs:305-312` : ANSI par défaut sauf `NO_COLOR`), si
  bien qu'un `grep` sur `champ=valeur` échoue sur `docker compose logs` ; le fichier de log est sans couleurs
  (`logging.rs:143`) et actif par défaut dans les deux composes (`docker-compose.yml:96`, `docker-compose.prod.yml:119`).
- **Retenu** : le manuel fait lire `./log/kesh.*.log` (`sudo grep "filtre d'adresse"`) ; si le fichier est désactivé,
  `docker compose logs … | sed 's/\x1b\[[0-9;]*m//g' | grep …` ; la ligne d'exemple du manuel vient du fichier.
  **Aucun changement du format global du journal.**
- **Écartées** : `.with_ansi(false)` sur la sortie standard (une ligne, mais un changement visible de tout le journal
  de Kesh — `docker logs` perdrait ses couleurs pour tous — hors du périmètre d'une story de diagnostic) ; `NO_COLOR`
  posé par les composes (deux fichiers de plus, un module de plus, même effet global) ; dire seulement « ne greppez
  que le message » (le tableau de lecture se lit champ par champ).
- **Réversible** : oui.
- **Annoté à la validation P2 (F8, R3, R5, F4/R6, F10)** : le repli par `sed` sur la sortie standard est **retiré** —
  il n'a été éprouvé sur aucun NAS (ni le `sed` de DSM, ni `sudo`) ; le manuel garde une seule commande,
  `sudo grep "filtre d'adresse" ./log/kesh.*.log`, avec ses limites en une phrase (rotation `never` → `kesh.log`,
  `KESH_LOG_HOST_DIR`, motif développé avant `sudo`). La ligne de code qui décide des couleurs est
  `fmt_layer.rs:743` (`:305-312` en est la documentation). L'alternative écartée, `.with_ansi(false)` sur la sortie
  standard, rendrait justes les `docker compose logs | grep` du manuel ; à soumettre à Guy si le fichier se révèle
  insuffisant à la mise à jour du NAS.
- **Annoté à l'étape 0 du développement (LOW F5 de la validation P3)** : le **titre** de ce choix (« la sortie
  standard, par un filtre de repli ») et son *Retenu* (`docker compose logs … | sed … | grep …`) décrivent l'état
  d'avant la validation P2 ; ils restent tels quels, comme relevé daté. **Ce qui vaut** : le journal se lit dans le
  fichier de log seulement ; **aucune** commande n'est prescrite pour la sortie standard, dont le manuel dit
  seulement que les codes couleur y coupent les recherches sur `champ=valeur`.

### C-32-6c-12 — Une mutation de plus que les deux demandées pour le comptage, et une pour l'AC-T3 et l'AC-T5

- **Contexte** (validation P1, F1 et R7) : l'orchestrateur demande les cas 13 et 14 et deux mutations (M17 : `plages`
  sur les entrées valides ; M18 : lignes non ASCII ignorées), tuées par le cas 13. Le cas 14 (ligne vide, virgule
  finale) ne serait alors tué par rien de neuf — M6 l'est déjà par les cas 5 et 7.
- **Retenu** : M19 (`xff_entrees` saute les morceaux vides), tuée par le cas 14 ; M20 (évènement au transport quand
  `servies` est vide, AC-T3) et M21 (une seule décision journalisée par requête, AC-T5), demandées par R7. **21
  mutations.**
- **Écartées** : retirer le cas 14 (la propriété « une ligne vide compte 1 » resterait sans garde).
- **Réversible** : oui — une mutation se retire sans toucher à la conception.

### C-32-6c-13 — Le nom réel du fichier de log : un défaut préexistant, hors périmètre, à tracer

- **Contexte** : avec la rotation `daily`, `tracing-appender-0.2.5` nomme le fichier `kesh.<AAAA-MM-JJ>.log`
  (`src/rolling.rs:648`, `:564`) ; `admin-manual.tex:957` et `.env.example:289` écrivent `./log/kesh.log`, qui
  n'existe pas.
- **Retenu** : la sous-section de la 32-6c écrit le nom réel (`kesh.*.log`) ; les deux lignes existantes ne sont pas
  touchées ; l'orchestrateur ouvre une issue (P3, manuel qui décrit autre chose que le code) — « on trace toujours ».
- **Écartées** : corriger `:957` et `.env.example` dans la story (un fichier de configuration de plus, hors du but
  diagnostique ; la correction mérite sa propre vérification au NAS).
- **Réversible** : oui.
