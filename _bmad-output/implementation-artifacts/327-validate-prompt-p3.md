# Prompt — validation P3, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **complète** (troisième passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte. La P1 (Sonnet ×2) a rendu 0/0/5/19, la P2 (Opus ×2) 0/0/5/25 ; la remédiation de la
P2 est le commit qui suit `605db417` sur la branche `story/32-7` (`git log --oneline -4`).*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC en trois groupes : A l'écran, B le manuel,
C la garde G23) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 24).
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-{R,F}.md` ; rapports P2 :
`/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-{R,F}.md` ; entrées « validation P1 » et « validation P2 » du Change
Log de la fiche. Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-7*, § *32-11*, § *Points
d'attention*, **§ *Réponses de Guy (2026-10-24, soir)*** et **§ *Réponses de Guy (2026-10-25)***) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, § 2.2, § 2.4, § 6, § 7) ; fiche
`32-5-cles-et-reglages-mcp.md` et registre `epic-32-choix-autonomes-325.md` (même worktree). **Fiches voisines, non
fusionnées, en lecture seule dans leurs worktrees** :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (et le registre
`epic-32-choix-autonomes-326.md`, C-32-6-52 et C-32-6-53 ; code de la 32-6a sous `crates/kesh-api/src/mcp/` du même
worktree), `/home/gcorbaz/devel/kesh-32-8/…/32-8-validation-avant-envoi.md`,
`/home/gcorbaz/devel/kesh-32-9/…/32-9-outils-de-lecture.md`, `/home/gcorbaz/devel/kesh-32-10a/…/32-10a-prompts-en-base.md`.
Règles : `CLAUDE.md` (§ *Recette visuelle*, § *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que
sur une passe sans finding au-dessus de LOW ; une remédiation de l'orchestrateur ou d'un agent n'a aucun privilège de
justesse —, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting
préventif*, la règle E2E, *Recompter ses propres comptes rendus*), `docs/agents-orchestration.md`, `docs/testing.md`
§ *Les échecs attendus*.

**Contraintes à tenir pour vraies et à contrôler** (décisions de Guy) : création des clés par l'**Administrateur**
seul ; portées `books:read`, `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` ; politique de
validation **par clé**, `required` par défaut ; serveur **désactivé** par défaut ; filtre d'adresse désactivable, et
derrière Traefik ou Docker **il ne filtre rien — l'écran et le manuel le disent** ; une restauration rend les réglages
**tels quels** ; **pas de limite de débit** ; la sécurité relève de l'infrastructure (jamais exposé sur Internet, VPN) ;
Q11 (a) ; Q10 (c) ; la **Q9** (Hermes) n'a **pas** de réponse : le manuel n'écrit rien de Hermes qui ne soit sourcé, et
la configuration de Claude Code reste marquée « non éprouvée » (décision acceptée de Guy). Au plus cinq modules. Aucune
donnée réelle.

**Décisions de l'orchestrateur, acquises** (ne pas les contester ; contrôler qu'elles sont **appliquées sans résidu**) :
- **l'ordre nominal** : la 32-7 se développe après la fusion de 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c, 32-9a,
  32-9b, 32-10a ; l'étape 0 le vérifie et s'arrête sinon ; **aucun AC conditionnel** à l'une de ces fusions, aucun ordre
  de rechange (la 32-10b, qui n'est pas prérequise, garde ses conditionnels) ;
- la garde « 2 bis » de la 32-6 **non retenue** ; **G23** réservé à la 32-7 ; le compte de `admin-manual.tex:2935` à la
  32-10b, puis à la 32-11 ; `sec:mcp-client` **appartient à la 32-7** (C-32-7-22) ;
- **neuves en P2** : `nav-admin-mcp` **retirée** de l'extension de G13 et tenue par `FAMILLES_RESOLUES['nav-']` de
  `i18n-keys.test.ts`, qui ne compare pas son repli au fr-CH — angle mort écrit, #758 cité ; `sitesTotal` ajusté par la
  story (C-32-7-23, AC-14 (d)) ; à l'AC-18 (5) et (7), **403** quand aucune société servie n'admet l'origine ou quand
  l'en-tête est répété, **401 fixe** quand la société de la clé ne l'admet pas ; une ligne de réglages corrompue :
  l'écran affiche un état d'échec de lecture qui le dit, le manuel donne les voies de réparation qui existent
  réellement (C-32-7-24) ; le graphe de l'epic et la fiche 32-9 seront amendés par l'orchestrateur (demandes 6 et 8).

⚠️ **L'ordre** : ce que la fiche prescrit d'écrire sur les stories prérequises est vérifiable **contre leurs fiches**
(et, pour la 32-6a, contre son code dans `kesh-32-6`), non contre le code du worktree de la 32-7 : un écart entre la
fiche 32-7 et une fiche voisine est un finding ; l'absence d'un code que la fiche dit « à recontrôler à l'étape 0 » n'en
est pas un.

⚠️ **Outillage de ce poste** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges
(`.{0,80}` → « exceeds complexity limits »). Employer `command grep`, et des fenêtres de 30 au plus.

## Lentille R — la spécification contre le dépôt et les fiches voisines

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `command grep -nF`) **copiée**. Axes, tous à exercer :

1. **Les faits 1 à 21** (§ *Ce que le code dit*), à `966baed5` ; en particulier le **fait 21, neuf** (`i18n-keys.test.ts` :
   l. 715-725, 858-868, 932, 1008-1016, 1034, 1037-1045, 1127-1135, 1137-1145, 1153-1163 ; `+layout.svelte:174-176` ;
   `i18n-repli-divergent-actif.test.ts:120-123` ; les clés `audit-log-…-mcp-server-…` du FTL), et les lignes neuves
   citées en P2 : `kesh-db/src/repositories/mcp_server_settings.rs:40-45`, `kesh-api/src/errors.rs:4270-4276`,
   `kesh-db/src/backup.rs:105-129`, `kesh-api/src/routes/onboarding.rs:252-268`,
   `kesh-api/src/routes/mcp_server_settings.rs:245`, `admin-manual.tex:1404`, et `svelte-sonner` 1.1.0
   (`dist/toast-state.svelte.js:213-225`, lisible dans `/home/gcorbaz/devel/kesh/frontend/node_modules/`, en lecture
   seule).
2. **Les reports** (R1 à R34) : refais l'inventaire **toi-même** — `grep -n '32-7'` sur l'epic, la fiche 32-5, les quatre
   fiches voisines, la spike, le registre `-325`, et `gh issue view 738 --json title,body,comments` (lecture seule ; les
   points B2 à B8 sont dans un commentaire) — et cherche un report absent, un rattachement faux, un « écarté » sans
   raison juste. Contrôle les reclassements de la P2 (R10 rattaché en partie, R11, R18, R21 complétés, R34 neuf) et le
   décompte 24 rattachés / 10 écartés ou sans objet.
3. **Fidélité aux fiches voisines** : chaque phrase que la fiche prescrit d'écrire à l'écran ou au manuel sur le
   comportement de la 32-6 (route absente, 401 unique, **403 et 401 de l'`Origin`** — ordre des contrôles de la fiche
   32-6 l. 1070-1085, C-32-6-52, C-32-6-53, et le code `kesh-32-6/crates/kesh-api/src/mcp/{transport,reponse,textes}.rs` —,
   mandataires, Traefik et Docker, P7-H3 et son second volet, ligne illisible, requête anonyme qui lit la base,
   `X-Real-IP`, 415, 400, 1 Mio, 30 s, B4, en-tête d'un pair non déclaré ignoré, transport Streamable HTTP), de la 32-8
   (politique, 24 h, 20 envois, issues d'outil sans validation, `sec:mcp-envois`, `/admin/mcp/sends`,
   `nav-admin-mcp-sends`, `### Ajouté` du CHANGELOG), de la 32-9 (`kesh_pieces`, `kesh_echeancier`, ce que
   `contacts:read` démasque, `sec:mcp-outils`, l. 1326-1327) et de la 32-10a (`prompts:write`) dit-elle **ce que ces
   fiches prescrivent** ?
4. **La propriété du secret** (AC-3, AC-9, AC-11, G-a, G-b, mutations M1 à M6, M16, M20, M21, M23) : les cinq « ni »
   sont-ils testables tels qu'écrits **après** la P2 — `toast` simulé comme fonction **et** porteur de méthodes,
   `mockClear()` après chaque témoin, secret simulé `kesh_mcp_SECRET…` distinct des témoins `kesh_mcp_TEMOIN`,
   `oublier()` avant l'envoi du `POST` ? Chaque témoin prouve-t-il que son espion voit, sans fausser l'assertion qui suit ?
5. **Les gardes lexicales et i18n** (AC-14 (a)-(d), AC-15) : refais la mesure de G-a ; son périmètre « sans descente »
   dans `admin/mcp/` est-il réalisable et ferme-t-il la question des sous-écrans d'autres stories ? G-b après
   `masquerCommentaires` ; l'extension de G13 par `startsWith` ferme-t-elle la classe `mcp-server-` ; l'AC-14 (d)
   (`FAMILLES_RESOLUES`, `sitesTotal`, M26) est-il juste et exécutable ; l'angle mort de `nav-admin-mcp` est-il écrit
   **exactement** à la mesure de ce que la garde vérifie ?
6. **Les dates** (AC-4) : inchangées en P2 ; contrôle qu'aucune phrase neuve ne les contredit.
7. **Le manuel — axe obligatoire, PDF compris** : la place de `\section{Le serveur MCP}` ; les renvois de l'AC-19 ; ce que
   le texte actuel **et le PDF aplati** disent déjà du MCP et de la réinitialisation :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -oiE '.{0,30}(serveur MCP|kesh_mcp|réinitialisation).{0,30}'`
   et de même pour `user-manual.pdf`. Un passage existant devient-il faux ? Les **phrases neuves** de l'AC-18 (1)
   (voies de réparation), (4) (requête anonyme, `X-Real-IP`, P7-H3 second volet), (5) (403/401), (7) et (8) (Claude
   Code, Hermes) sont-elles vraies, sourcées à la ligne citée, et non contraires au texte publié ? L'AC-22 (CHANGELOG)
   est-il juste contre `CHANGELOG.md` et C-32-5-49 ?

## Lentille F — faisabilité, découpage, fausses prémisses et régressions de la P2

1. **Prémisses** : chaque « établi au code » tient-il, dont les neuves de la P2 (le 500 au `GET` et au `PUT` d'une ligne
   corrompue, la remise à zéro refusée sur une installation de production, la réparation #528 sans
   `mcp_server_settings`, une ligne supprimée rendue en défauts à `version: 0`, `FAMILLES_RESOLUES` sans comparaison du
   repli, `'nav-'` hors couverture close) ?
2. **L'ordre et le découpage** (C-32-7-1, C-32-7-3) : recompte les modules au grain du kickoff (l'amendement de
   `i18n-keys.test.ts` change-t-il le compte ?) ; reste-t-il un AC, une tâche, une ligne du § *À recontrôler* ou du
   registre qui suppose une fusion absente ou une branche « sinon » sur un prérequis ?
3. **Fidélité aux décisions de Guy et de l'orchestrateur** (listes en tête) : chacune portée à l'écran **et** au manuel
   quand elle le demande ; rien de contraire ; les décisions neuves de la P2 appliquées sans résidu.
4. **Tests, E2E et recette** : chaque AC est-il vérifiable tel qu'écrit, dont AC-5 (g) et (h) ? La liste des **29**
   captures couvre-t-elle **chaque** état touché, dont les états neufs de la P2 ? M26 rougit-elle comme écrit ?
5. **La garde G23** (AC-25) : réalisable — extraction sur le source **brut**, premier `\texttt{…}` de chaque **rangée**
   (texte découpé sur `\\`), accès direct à `DEFAULT_ALLOWED_NETWORKS` ? Ses témoins empêchent-ils de passer à vide ?
6. **Décomptes** : recompte à la source — 25 AC (17 / 7 / 1), 9 tâches, 26 mutations (M1 à M26), 34 reports (24
   rattachés, 10 écartés ou sans objet), 24 choix, 29 captures, 21 faits, 4 modules ; et le trend P2 0/0/5/25 contre les
   deux rapports P2.
7. **Régressions de la remédiation P2** (axe **obligatoire** — *la sévérité se déplace vers ce qu'on vient d'écrire* ; la
   P2 a trouvé un MEDIUM né de la remédiation P1, F1 = R1) : lis le diff de la remédiation
   (`git diff 605db417 HEAD -- _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md
   _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md`) et cherche ce qu'il a cassé ou contredit :
   une phrase nouvelle contraire à une ancienne restée (par exemple sur `nav-admin-mcp`, le 401 unique, « dernière
   utilisation », ce que l'écran répare), une référence croisée (AC, mutation, capture, demande, choix, fait) devenue
   fausse, un numéro de ligne inexact (fiche 32-6 à `dd4b8398`, fiche 32-8 à `81cbd3f5`, dépôt à `966baed5`), une
   entrée passée du Change Log réécrite (interdit), une demande à l'orchestrateur close à tort ou ouverte sans motif, une
   phrase prescrite au manuel qui affirme plus que sa source. **Et ce que la fiche omet** : un état d'écran, un refus du
   serveur, un texte publié, une garde existante du frontend, un risque absent de *Ce que les gardes ne voient pas*.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée — `grep -nF` pour toute présence ou absence), correction proposée. Un bilan
d'en-tête **cohérent avec le corps**. Pour chaque finding, dis s'il **naît de la remédiation P2** (oui / non). ⛔ **La
liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**,
par l'outil d'écriture de fichiers, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/327-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

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
`frontend/node_modules/svelte-sonner/` ; les worktrees voisins ne se lisent qu'en lecture. Autorisés : lecture de
fichiers, `grep`/`command grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, les deux `gh issue
view` ci-dessus, et les filtres `tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une
commande est refusée, arrête-toi et dis-le au rapport.

---

## Annexe — le prompt de la remédiation P2, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh. Tu travailles **uniquement** dans le worktree
> `/home/gcorbaz/devel/kesh-32-7`, sur sa branche, et tu écris en français.
>
> **Consignes.** Applique en entier `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`. En plus :
> - chemins absolus ;
> - le dépôt principal est en lecture seule ;
> - pas de push, pas de `gh` qui écrit, pas de prepare-release ;
> - aucun cargo, npm, docker ni curl ;
> - la lecture est permise (pour #738 : `gh issue view 738 --json title,body,comments`) ;
> - si une commande est refusée, tu t'arrêtes ;
> - tu n'ouvres jamais `~/kesh-test/`.
>
> **Tâche.** Remédier la validation P2 de la fiche `_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`. Les rapports sont dans `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-R.md` (0/0/3/12) et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-F.md` (0/0/2/13) : lis-les en entier. Pour chaque finding, vérifie la prémisse au dépôt et écris au Change Log ce que tu réfutes. **Recyclage à déclarer** : F1 = R1 vient de ta remédiation P1 (C-32-7-21).
>
> **Décisions de l'orchestrateur :**
>
> **R1 = F1 — `nav-admin-mcp`.** La solution la plus simple qui ne rougit pas par construction : retirer `nav-admin-mcp` de l'extension de G13. La clé de menu reste alors couverte par ce qui couvre déjà les clés `nav-` (`FAMILLES_RESOLUES['nav-']`, `i18n-keys.test.ts:858`) :
> - lis ce que cette garde vérifie vraiment ;
> - si elle ne compare pas le repli au fr-CH, écris l'angle mort et cite l'issue #758, déjà ouverte pour cette classe ;
> - intègre le compte exact `sitesTotal` (`i18n-keys.test.ts:724`, `:1034`) : la fiche prescrit son ajustement et nomme le fichier au § *Fichiers touchés*.
>
> Vérifie aussi qu'aucune clé `mcp-server-` n'est résolue autrement que par des littéraux, puis écris-le comme fait sourcé.
>
> **R2 et F2 (403/401 de l'`Origin`, requête anonyme, `X-Real-IP`), R3 (ligne corrompue → 500 au GET et au PUT, réparation impossible par l'écran), R9 :** écris chaque point à l'AC qui le porte, avec la phrase du manuel :
> - 403 quand aucune société servie n'admet l'origine ou quand l'en-tête est répété ;
> - 401 fixe quand la société de la clé ne l'admet pas.
>
> Pour R3, l'écran affiche un état d'échec de lecture qui le dit. Le manuel donne la voie de réparation qui existe réellement : remise à zéro ou réparation de la 32-5d, à vérifier au code. Si aucune voie n'existe, écris-le.
>
> **LOW :** tout est intégré, ou renvoyé au § *À recontrôler* avec sa correction. Quelques indications :
> - F6 / R15 (graphe de l'epic) et R11 (fiche 32-9, l. 1326-1327) vont aux demandes à l'orchestrateur, qui amendera l'epic et la 32-9 ;
> - R4 / F8 : un `mockClear()` après chaque témoin ;
> - R5 : espionner `toast` lui-même ;
> - F10 / R7 : n'écrire que ce que la spike source ; ce qui est déduit est marqué comme tel ou retiré ;
> - F13 : corriger « présentée et acceptée ».
>
> Après le patch :
> - grep le symptôme, c'est-à-dire la valeur : `nav-admin-mcp`, `403`, `demande 1`, `demande 4`, `sinon omis`, `sinon créée` ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris une entrée P2 neuve, avec le trend P1 0/0/5/19 → P2 0/0/5/25.
>
> **Livrables :**
> - un commit `refs #723`, sans mot-clé de fermeture ;
> - le prompt de la P3 dans `_bmad-output/implementation-artifacts/327-validate-prompt-p3.md` : deux lentilles R et F, passe complète, **Sonnet**, contexte frais, rapports `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p3-{R,F}.md`, mêmes exigences et mêmes interdits que le prompt P2, plus un axe obligatoire sur les régressions de cette remédiation. Recopie ce prompt-ci en annexe ;
> - un rapport final : commit, décisions, réfutations, demandes.
