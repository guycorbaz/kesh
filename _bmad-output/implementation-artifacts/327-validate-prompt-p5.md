# Prompt — validation P5, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **complète** (cinquième passe sur huit). **Deux lentilles, R et F**, chacune en
contexte frais, **Sonnet**. Lecture seule stricte. La P1 (Sonnet ×2) a rendu 0/0/5/19, la P2 (Opus ×2) 0/0/5/25, la
P3 (Sonnet ×2) 0/0/2/9, la P4 (Opus ×2) 0/0/3/17 ; la remédiation de la P4 est le commit qui suit `5b473eb6` sur la
branche `story/32-7` (`git log --oneline -6`). ⚠️ Après la sixième passe, l'orchestrateur ne remédie plus lui-même : il
retire ou renvoie au développement (`CLAUDE.md`, *Review Iteration Rule*). Un finding qui propose de **retirer** une
phrase invérifiable vaut mieux qu'un finding qui propose de l'enrichir.*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC en trois groupes : A l'écran, B le manuel,
C la garde G23) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 26).
Rapports P1 à P4 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p{1,2,3,4}-{R,F}.md` ; entrées « validation P1 » à
« validation P4 » du Change Log de la fiche. Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md`
(§ *32-7*, § *32-11*, § *Points d'attention*, **§ *Réponses de Guy (2026-10-24, soir)*** et **§ *Réponses de Guy
(2026-10-25)***) ; spike `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, § 2.2, § 2.4, § 6, § 7, et
l. 990-1001) ; fiche `32-5-cles-et-reglages-mcp.md` et registre `epic-32-choix-autonomes-325.md` (même worktree).
**Fiches voisines, non fusionnées, en lecture seule dans leurs worktrees** :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (et le registre
`epic-32-choix-autonomes-326.md`, C-32-6-52 et C-32-6-53 ; code de la 32-6a sous `crates/kesh-api/src/mcp/` et
`crates/kesh-api/src/lib.rs` du même worktree, branche `story/32-6`, PR #760), `/home/gcorbaz/devel/kesh-32-8/…/32-8-validation-avant-envoi.md`,
`/home/gcorbaz/devel/kesh-32-9/…/32-9-outils-de-lecture.md`, `/home/gcorbaz/devel/kesh-32-10a/…/32-10a-prompts-en-base.md`.
Règles : `CLAUDE.md` (§ *Recette visuelle*, § *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que
sur une passe sans finding au-dessus de LOW ; une remédiation de l'orchestrateur ou d'un agent n'a aucun privilège de
justesse —, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting
préventif*, la règle E2E, *Recompter ses propres comptes rendus*), `docs/agents-orchestration.md`, `docs/testing.md`
§ *Les échecs attendus*.

**Contraintes à tenir pour vraies et à contrôler** (décisions de Guy) : création des clés par l'**Administrateur**
seul ; portées `books:read`, `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` ; politique de
validation **par clé**, `required` par défaut ; serveur **désactivé** par défaut ; filtre d'adresse désactivable, et
derrière un mandataire non déclaré ou Docker **il ne filtre rien — l'écran et le manuel le disent** ; une restauration
rend les réglages **tels quels** ; **pas de limite de débit** ; la sécurité relève de l'infrastructure : le serveur MCP
est **réservé au réseau local** — l'écran et le manuel disent « n'activez pas le serveur MCP sur une installation
exposée à Internet : `/mcp` est servi par le même port que l'application », et **n'affirment pas** que Kesh n'est
jamais exposé (le manuel publié documente l'exposition) ; Q11 (a) ; Q10 (c) ; la **Q9** (Hermes) n'a **pas** de
réponse : le manuel n'écrit rien de Hermes qui ne soit sourcé, et la configuration de Claude Code reste marquée « non
éprouvée » (décision acceptée de Guy). Au plus cinq modules. Aucune donnée réelle.

**Décisions de l'orchestrateur, acquises** (ne pas les contester ; contrôler qu'elles sont **appliquées sans résidu**) :
- **l'ordre nominal** : la 32-7 se développe après la fusion de 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c, 32-9a,
  32-9b, 32-10a ; l'étape 0 le vérifie et s'arrête sinon ; **aucun AC conditionnel** à l'une de ces fusions, aucun ordre
  de rechange (la 32-10b, qui n'est pas prérequise, garde ses conditionnels) ;
- la garde « 2 bis » de la 32-6 **non retenue** ; **G23** réservé à la 32-7 ; le compte de `admin-manual.tex:2935` à la
  32-10b, puis à la 32-11 ; `sec:mcp-client` **appartient à la 32-7** (C-32-7-22) ;
- **de la P2** : `nav-admin-mcp` hors de l'extension de G13, tenue par `FAMILLES_RESOLUES['nav-']` (C-32-7-23,
  AC-14 (d)) ; 403 / 401 fixe de l'`Origin` (AC-18 (5), (7)) ; une ligne de réglages corrompue : état d'échec de
  lecture à l'écran, voies de réparation au manuel (C-32-7-24) ;
- **de la P3** : familles d'adresses (`0.0.0.0/0` tout client IPv4, `::/0` tout client IPv6, les deux entrées pour les
  deux familles ; même règle pour un mandataire `/0`) ; clé dont le créateur est désactivé ou n'est plus
  Administrateur, dite au manuel et non à l'écran (C-32-7-25) ; réparation d'une ligne de réglages : SQL d'abord,
  restauration en dernier recours ;
- **neuves en P4** :
  - **l'exposition** (R1) : la phrase de l'AC-6 (2) et de l'AC-18 (4) ci-dessus ; la citation des réponses de Guy
    (l. 61) annotée, non réécrite ;
  - **les mandataires** (R2) : à l'écran, une phrase courte (« Traefik, Nginx, le proxy inversé du NAS ») ; au
    manuel, le `keshwarning` nomme Traefik, Nginx, le proxy inversé de DSM et la passerelle Docker — Kesh voit le
    mandataire, le filtre ne filtre rien, sauf mandataire déclaré (exemple `127.0.0.1/32`) qui pose
    `X-Forwarded-For` ; pour DSM, « non éprouvé » ;
  - **le contrôle du PDF** (F1 = R3) : seize motifs, chacun à zéro sur le PDF de la base, cités au développement ;
  - l'écran affiche une adresse mappée telle qu'enregistrée, le serveur la lit sous sa forme IPv4 (F4 = R5) ; la
    puce « Ce qui se répare à l'écran » de l'AC-5 et le blocage d'enregistrement par une entrée posée hors de l'écran
    (F5) ; `'admin-mcp-sends'` non ajouté (C-32-7-26).

⚠️ **L'ordre** : ce que la fiche prescrit d'écrire sur les stories prérequises est vérifiable **contre leurs fiches**
(et, pour la 32-6a, contre son code dans `kesh-32-6`), non contre le code du worktree de la 32-7 : un écart entre la
fiche 32-7 et une fiche voisine est un finding ; l'absence d'un code que la fiche dit « à recontrôler à l'étape 0 » n'en
est pas un.

⚠️ **Outillage de ce poste** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges
(`.{0,80}` → « exceeds complexity limits »). Employer `command grep`, et des fenêtres de 30 au plus.

## Lentille R — la spécification contre le dépôt et les fiches voisines

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `command grep -nF`) **copiée**. Axes, tous à exercer :

1. **Les faits 1 à 21** (§ *Ce que le code dit*), à `966baed5`, et **les lignes neuves citées en P4** :
   `crates/kesh-api/src/routes/mcp_server_settings.rs:113-115`, `:126-135`, `:171-195` (dont `:175-188`), `:198-208` ;
   `crates/kesh-api/src/services/cles_mcp.rs:94-99`, `:101` ; `crates/kesh-api/src/routes/users.rs:82-86`, `:366`,
   `:442` ; `crates/kesh-db/src/repositories/mcp_keys.rs:238-268` ;
   `crates/kesh-db/src/repositories/mcp_server_settings.rs:83-90` ; `docs/manual/fr/admin-manual.tex:169`, `:177`,
   `:307`, `:330-335`, `:351`, `:398`, `:400`, `:493-495`, `:535`, `:638-658` (dont `:653-654`), `:675`, `:1371` ;
   `kesh-32-6/crates/kesh-api/src/lib.rs:1278` ; `kesh-32-6/crates/kesh-api/src/mcp/reseau.rs:11-12`, `:47-54`,
   `:119-122`, `:201-204` ; `svelte-sonner` 1.1.0 (`dist/index.js:4-5`, `dist/toast-state.svelte.js:213-226`, lisibles
   dans `/home/gcorbaz/devel/kesh/frontend/node_modules/`, en lecture seule).
2. **Les reports** (R1 à R35) : refais l'inventaire **toi-même** — `grep -n '32-7'` sur l'epic, la fiche 32-5, les quatre
   fiches voisines, la spike, le registre `-325`, et `gh issue view 738 --json title,body,comments` (lecture seule ; les
   points B2 à B8 sont dans un commentaire) — et cherche un report absent, un rattachement faux, un « écarté » sans
   raison juste ; le décompte 24 rattachés / 11 écartés ou sans objet.
3. **Fidélité aux fiches voisines et au code de la 32-6a** : chaque phrase que la fiche prescrit d'écrire à l'écran ou au
   manuel sur le comportement de la 32-6 — **dont les phrases neuves de la P4** : le `keshwarning` des mandataires
   (que voit Kesh derrière chaque recette ; quand le filtre redevient opérant), l'avertissement (c) filtre activé
   seulement, l'adresse mappée lue sous sa forme IPv4, le 403 d'une origine « ni propre ni admise » — et de la 32-8, de
   la 32-9 et de la 32-10a dit-elle **ce que ces fiches et ce code prescrivent** ?
4. **La propriété du secret** (AC-3, AC-9, AC-11, G-a, G-b, mutations M1 à M6, M16, M20, M21, M23) : les cinq « ni »
   restent-ils testables tels qu'écrits, dont le simulacre de l'**objet** `toast` et la reprise des autres
   exportations du module par `vi.importActual` ?
5. **Les gardes lexicales et i18n** (AC-14 (a)-(d), AC-15) : refais la mesure de G-a ; G-b ; l'extension de G13 par
   `startsWith` ; l'AC-14 (d) (`'admin-mcp-sends'`, C-32-7-26) et M26 ; G23 (c) toujours satisfaite par la clé de
   l'AC-6 (1) récrite.
6. **Les dates** (AC-4) : inchangées en P4 ; aucune phrase neuve ne les contredit.
7. **Le manuel — axe obligatoire, PDF compris** : la place de `\section{Le serveur MCP}` ; les renvois de l'AC-19, de
   l'AC-18 (1) et ceux, neufs, de l'AC-18 (4) (`sec:traefik`, `sec:reverse-proxy`, `sec:synology`, `sec:mcp-activer`)
   — étiquettes présentes (`command grep -nF '\label{sec:'`) ; ce que le texte actuel **et le PDF aplati** disent
   déjà du MCP, de l'exposition à Internet, des mandataires, de la réinitialisation, de la restauration et des clés
   d'API :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -oiE '.{0,30}(serveur MCP|Internet|X-Forwarded-For|proxy inversé|réinitialisation|désactiver le créateur).{0,30}'`
   et de même pour `user-manual.pdf`. Un passage existant devient-il faux, ou contredit-il une phrase neuve ? Les
   **phrases neuves de la P4** sont-elles vraies, sourcées à la ligne citée, et non contraires au texte publié ? Le
   contrôle de l'AC-21 : **rejoue** chacun des seize motifs (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' '
   | tr -s ' ' | command grep -cE '<motif>'`) et copie la sortie — chacun doit rendre 0 ; chaque motif correspond-il à
   une phrase que la fiche prescrit **réellement** d'écrire (sans quoi il rendrait 0 aussi après la story) ?

## Lentille F — faisabilité, découpage, fausses prémisses et régressions de la P4

1. **Prémisses** : chaque « établi au code » tient-il, dont les neuves de la P4 (la recette Nginx pose
   `X-Forwarded-For`, la recette DSM non ; `canonical_network` sans `to_canonical` ; `validate_settings` refuse plus que
   la 32-6a — bits d'hôte, bornes, plages et origines — ; les deux messages sans `{ $entry }` ; `decider` rend *admis*
   avant `X-Forwarded-For` filtre désactivé ; aucune révocation des clés MCP dans `routes/users.rs` ;
   `find_active_auth_by_key_hash` ferme une clé sans portée connue ; `/mcp` servi par le même port) ?
2. **L'ordre et le découpage** (C-32-7-1, C-32-7-3) : recompte les modules au grain du kickoff ; le point 4 à Guy et
   C-32-7-16 / C-32-7-25 décrivent-ils maintenant exactement le coût des deux voies (route jointe ; lecture de
   `/api/v1/users`) ? Reste-t-il un AC, une tâche, une ligne du § *À recontrôler* ou du registre qui suppose une fusion
   absente ou une branche « sinon » sur un prérequis ?
3. **Fidélité aux décisions de Guy et de l'orchestrateur** (listes en tête) : chacune portée à l'écran **et** au manuel
   quand elle le demande ; rien de contraire ; aucune affirmation résiduelle que Kesh n'est jamais exposé, hors des
   citations et annotations historiques ; les décisions neuves de la P4 appliquées sans résidu.
4. **Tests, E2E et recette** : chaque AC est-il vérifiable tel qu'écrit, dont le test d'absence de l'avertissement (c)
   filtre désactivé, la puce « Ce qui se répare à l'écran » de l'AC-5, la recette de la capture (22) (enregistrement
   d'abord, lignes touchées citées) et la capture (5) dans ses deux familles ? La liste des **29** captures
   couvre-t-elle **chaque** état touché, dont le blocage d'enregistrement par une entrée posée hors de l'écran ?
5. **La garde G23** (AC-25) : réalisable, témoins suffisants ; la clé de l'AC-6 (1) récrite contient toujours `Traefik`
   et `Docker`.
6. **Décomptes** : recompte à la source — 25 AC (17 / 7 / 1), 9 tâches, 26 mutations (M1 à M26), 35 reports (24
   rattachés, 11 écartés ou sans objet), **26 choix**, 29 captures, 21 faits, 4 modules, seize motifs à l'AC-21, six
   points « ajoutés en validation P4 » au § *À recontrôler* ; et le trend P4 0/0/3/17 contre les deux rapports P4.
7. **Régressions de la remédiation P4** (axe **obligatoire** — *la sévérité se déplace vers ce qu'on vient d'écrire* ;
   la P4 a trouvé un MEDIUM né en partie de la remédiation P3) : lis le diff de la remédiation
   (`git diff 5b473eb6 HEAD -- _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md
   _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md`) et cherche ce qu'il a cassé ou contredit :
   une phrase nouvelle contraire à une ancienne restée (par exemple « jamais exposé », « Traefik » seul, « le message
   la nomme », « troisième état », « ramenée à sa forme IPv4 », « aucune société servie », « toucher la route ») ; une
   phrase prescrite au manuel qui affirme plus que sa source (l'exemple `127.0.0.1/32` sous Docker ; « Kesh voit le
   mandataire » ; « la recette Nginx le pose ») ; une référence croisée devenue fausse (AC, mutation, capture,
   demande — la **11** est neuve —, choix — **C-32-7-26** est neuf —, fait, étiquette du manuel) ; un numéro de ligne
   inexact (la P4 en a cité beaucoup, relis-les) ; une entrée passée du Change Log ou une entrée du registre
   **réécrite** au lieu d'être annotée (interdit — C-32-7-25 doit être annoté, non réécrit) ; un motif de l'AC-21 qui
   ne prouve rien. **Et ce que la fiche omet** : un état d'écran, un refus du serveur, un texte publié, une garde
   existante du frontend, un risque absent de *Ce que les gardes ne voient pas*.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée — `grep -nF` pour toute présence ou absence), correction proposée — de
préférence un **retrait** plutôt qu'un enrichissement. Un bilan d'en-tête **cohérent avec le corps**. Pour chaque
finding, dis s'il **naît de la remédiation P4** (oui / non). ⛔ **La liste des axes exercés ET non exercés** (R1 à R7,
F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe pas encore :
son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, par l'outil d'écriture de fichiers, un seul fichier
par lentille : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad. **Aucune
redirection** (`>`, `>>`, `2>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. **Aucun interpréteur** : ni `python3`, `python`, `perl`, `ruby`, `node`, `deno`, ni
**`awk`**, ni `sed` autrement qu'en `sed -n`. **Aucun `gh` qui écrit** (`pr`, `issue create|edit|close|comment`,
`api` en écriture…) ; seuls `gh issue view 738 --json title,body,comments` et `gh issue view 758 --json title,body`
sont permis. Aucun `git checkout`/`switch`/`commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`. Aucune
commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, Playwright, `docker`, `curl`,
`mariadb`/`mysql`, `sed -i`. `pdftotext` vers la sortie standard seulement (`-` en sortie). Rien de `~/kesh-test/`,
aucune donnée réelle. Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre, sauf la **lecture** de
`frontend/node_modules/svelte-sonner/` ; le registre cargo (`~/.cargo/registry/src/`) ne se lit qu'en lecture, pour
`ipnet` ; les worktrees voisins ne se lisent qu'en lecture. Autorisés : lecture de fichiers, `grep`/`command grep`,
`sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, les deux `gh issue view` ci-dessus, et les filtres
`tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une commande est refusée, arrête-toi et
dis-le au rapport.

---

## Annexe — le prompt de la remédiation P4, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh. Tu travailles **uniquement** dans le worktree
> `/home/gcorbaz/devel/kesh-32-7`, sur sa branche, et tu écris en français.
>
> **Consignes** : applique en entier `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`. En particulier :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push, ni `gh` qui écrit, ni prepare-release ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture et `pdftotext` vers la sortie standard sont permis ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> **Tâche** : remédier la validation P4 de la fiche `_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`. Les deux rapports sont à lire en entier :
> - `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p4-R.md` (0/0/2/9) ;
> - `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p4-F.md` (0/0/1/8).
>
> Vérifie la prémisse de chaque finding au dépôt, et écris au Change Log ce que tu réfutes. C'est la quatrième passe sur huit : après la sixième, l'orchestrateur ne remédie plus et l'on retire plutôt que corriger. Préfère donc **retirer** une phrase invérifiable plutôt que l'enrichir.
>
> **Décisions de l'orchestrateur** :
> - **R1 (exposition sur Internet).** Guy a décidé que le serveur MCP est réservé au réseau local (« la sécurité est une affaire d'infrastructure »). La phrase devient donc, à l'écran comme au manuel : « n'activez pas le serveur MCP sur une installation exposée à Internet : `/mcp` est servi par le même port que l'application ». Retire toute affirmation que Kesh n'est jamais exposé.
> - **R2 (mandataires).** L'avertissement du filtre nomme les mandataires que le manuel documente (Traefik, Nginx, proxy inversé de DSM) et le cas Docker. Derrière chacun, Kesh voit le mandataire, et le filtre ne filtre plus rien, sauf si ce mandataire est déclaré (exemple `127.0.0.1/32`) et qu'il pose `X-Forwarded-For`. Pour DSM, la pose de `X-Forwarded-For` n'est sourcée nulle part : écris « non éprouvé ». Garde la phrase courte à l'écran ; le détail va au manuel.
> - **F1 = R3 (contrôle du PDF qui passe à vide).** Choisis des motifs absents du PDF de base. Écris la règle à l'AC-21 : chaque motif rend zéro sur le PDF actuel, et le développement en cite la sortie à l'étape 0. Exécute toi-même la commande sur le PDF actuel (`pdftotext … - | tr '\n' ' ' | tr -s ' '` puis `command grep -c`) pour chaque motif choisi, et cite la sortie.
> - **Les LOW** sont tous intégrés.
>   - F4 = R5 : l'écran affiche l'entrée telle qu'enregistrée ; le serveur la lit sous sa forme IPv4.
>   - F5 : déplacer le troisième état dans « ce qui se répare à l'écran », et écrire le blocage d'enregistrement par une entrée posée hors de l'écran.
>   - F8 : corriger « exigerait de toucher la route », et me signaler si le point 4 soumis à Guy doit être reformulé. Ne le reformule pas toi-même au-delà de l'exactitude.
>   - R4 : ajouter le report manquant, ce qui porte le total à 35.
>
> Après le patch :
> - greppe le symptôme (la valeur) : `jamais exposé`, `Traefik`, `X-Real-IP`, `rouvre`, `forme IPv4`, `toucher la route` ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - ajoute une entrée P4 neuve, avec le trend P3 0/0/2/9 → P4 0/0/3/17.
>
> **Livrables** :
> - un commit `refs #723`, sans mot-clé de fermeture ;
> - le prompt de la P5, `_bmad-output/implementation-artifacts/327-validate-prompt-p5.md` : passe complète, lentilles R et F, **Sonnet**, contexte frais, rapports `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p5-{R,F}.md`, mêmes exigences et interdits, plus un axe obligatoire sur les régressions de cette remédiation ; ce prompt-ci y est recopié en annexe ;
> - un rapport final.
