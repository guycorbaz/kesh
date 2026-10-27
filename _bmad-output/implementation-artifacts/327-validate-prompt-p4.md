# Prompt — validation P4, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **complète** (quatrième passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Opus**. Lecture seule stricte. La P1 (Sonnet ×2) a rendu 0/0/5/19, la P2 (Opus ×2) 0/0/5/25, la P3 (Sonnet ×2)
0/0/2/9 ; la remédiation de la P3 est le commit qui suit `bf07e3d2` sur la branche `story/32-7`
(`git log --oneline -5`).*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC en trois groupes : A l'écran, B le manuel,
C la garde G23) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 25).
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-{R,F}.md` ; P2 : `327-validate-p2-{R,F}.md` ; P3 :
`327-validate-p3-{R,F}.md` (même dossier) ; entrées « validation P1 » à « validation P3 » du Change Log de la fiche.
Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-7*, § *32-11*, § *Points d'attention*,
**§ *Réponses de Guy (2026-10-24, soir)*** et **§ *Réponses de Guy (2026-10-25)***) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, § 2.2, § 2.4, § 6, § 7) ; fiche
`32-5-cles-et-reglages-mcp.md` et registre `epic-32-choix-autonomes-325.md` (même worktree). **Fiches voisines, non
fusionnées, en lecture seule dans leurs worktrees** :
`/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` (et le registre
`epic-32-choix-autonomes-326.md`, C-32-6-52 et C-32-6-53 ; code de la 32-6a sous `crates/kesh-api/src/mcp/` du même
worktree, branche `story/32-6`, PR #760), `/home/gcorbaz/devel/kesh-32-8/…/32-8-validation-avant-envoi.md`,
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
- **de la P2** : `nav-admin-mcp` hors de l'extension de G13, tenue par `FAMILLES_RESOLUES['nav-']` (C-32-7-23,
  AC-14 (d)) ; 403 / 401 fixe de l'`Origin` (AC-18 (5), (7)) ; une ligne de réglages corrompue : état d'échec de
  lecture à l'écran, voies de réparation au manuel (C-32-7-24) ;
- **neuves en P3** :
  - **familles d'adresses** — `0.0.0.0/0` admet tout client IPv4, `::/0` tout client IPv6, il faut les deux entrées
    pour les deux familles ; même règle pour un mandataire `/0` ; une adresse IPv4 mappée compte comme IPv4 (fait
    sourcé au code de la 32-6a, AC-6 (b)) — à l'écran (AC-6 (b), (c) et leurs tests), au manuel (AC-18 (4)) ;
  - **clé dont le créateur est désactivé ou n'est plus Administrateur** — dite au manuel (AC-18 (2)), à l'AC-8 (b) et à
    *Ce que les gardes ne voient pas*, **aucun état à l'écran** (la route ne le fournit pas ; C-32-7-25) ;
  - **réparation d'une ligne de réglages** — d'abord la suppression de la ligne par SQL, en dernier recours la
    restauration (qui remplace toute l'installation et refuse une sauvegarde d'une version qui ne porte pas les mêmes
    tables) ; « aucune voie depuis l'écran », non « dans Kesh » ; une entrée invalide des mandataires **se répare à
    l'écran** (AC-5, troisième état).

⚠️ **L'ordre** : ce que la fiche prescrit d'écrire sur les stories prérequises est vérifiable **contre leurs fiches**
(et, pour la 32-6a, contre son code dans `kesh-32-6`), non contre le code du worktree de la 32-7 : un écart entre la
fiche 32-7 et une fiche voisine est un finding ; l'absence d'un code que la fiche dit « à recontrôler à l'étape 0 » n'en
est pas un.

⚠️ **Outillage de ce poste** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges
(`.{0,80}` → « exceeds complexity limits »). Employer `command grep`, et des fenêtres de 30 au plus.

## Lentille R — la spécification contre le dépôt et les fiches voisines

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `command grep -nF`) **copiée**. Axes, tous à exercer :

1. **Les faits 1 à 21** (§ *Ce que le code dit*), à `966baed5`, et **les lignes neuves citées en P3** :
   `crates/kesh-api/src/services/cles_mcp.rs:20-21`, `:47`, `:94-100` ; `crates/kesh-db/src/repositories/mcp_keys.rs:197-217` ;
   `crates/kesh-api/src/routes/mcp_keys.rs:260-266` ; `crates/kesh-api/src/routes/mcp_server_settings.rs:198-208` ;
   `crates/kesh-db/migrations/20261025000001_mcp_keys_and_server_settings.sql:81-83` ;
   `docs/manual/fr/admin-manual.tex:705`, `:767`, `:2177`, `:2249`, `:2262`, `:2910` ;
   `frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts:122` ; `svelte-sonner` 1.1.0
   (`dist/toast-state.svelte.js:213-226`, lisible dans `/home/gcorbaz/devel/kesh/frontend/node_modules/`, en lecture
   seule) ; `Cargo.lock:1872-1873` et `ipnet` 2.12.0 (`~/.cargo/registry/src/*/ipnet-2.12.0/src/ipnet.rs:1419-1426`,
   lecture seule).
2. **Les reports** (R1 à R34) : refais l'inventaire **toi-même** — `grep -n '32-7'` sur l'epic, la fiche 32-5, les quatre
   fiches voisines, la spike, le registre `-325`, et `gh issue view 738 --json title,body,comments` (lecture seule ; les
   points B2 à B8 sont dans un commentaire) — et cherche un report absent, un rattachement faux, un « écarté » sans
   raison juste ; le décompte 24 rattachés / 10 écartés ou sans objet.
3. **Fidélité aux fiches voisines et au code de la 32-6a** : chaque phrase que la fiche prescrit d'écrire à l'écran ou au
   manuel sur le comportement de la 32-6 — **dont les phrases neuves de la P3** : la famille d'adresses et la forme
   canonique du pair (`kesh-32-6/crates/kesh-api/src/mcp/reseau.rs:9-12`, `:34-43`, `:57`, `:64`, `:73`, `:128` ;
   `transport.rs:81`), la voie utile du 403 et du 401 de l'`Origin` (`transport.rs:105-111`, `:151-162` ;
   `reseau.rs:182-205`) — et de la 32-8, de la 32-9 et de la 32-10a dit-elle **ce que ces fiches et ce code
   prescrivent** ?
4. **La propriété du secret** (AC-3, AC-9, AC-11, G-a, G-b, mutations M1 à M6, M16, M20, M21, M23) : les cinq « ni »
   restent-ils testables tels qu'écrits, dont le simulacre de `toast` porteur de **toutes les entrées de l'export
   réel** ?
5. **Les gardes lexicales et i18n** (AC-14 (a)-(d), AC-15) : refais la mesure de G-a ; G-b ; l'extension de G13 par
   `startsWith` ; l'AC-14 (d) et M26.
6. **Les dates** (AC-4) : inchangées en P3 ; aucune phrase neuve ne les contredit.
7. **Le manuel — axe obligatoire, PDF compris** : la place de `\section{Le serveur MCP}` ; les renvois de l'AC-19 et
   ceux, neufs, de l'AC-18 (1) (`sec:env-vars`, `sec:backup-ui-keshbackup`) ; ce que le texte actuel **et le PDF aplati**
   disent déjà du MCP, de la réinitialisation, de la restauration et des clés d'API :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -oiE '.{0,30}(serveur MCP|kesh_mcp|réinitialisation|remplace l.intégralité|désactiver le créateur).{0,30}'`
   et de même pour `user-manual.pdf`. Un passage existant devient-il faux ? Les **phrases neuves de la P3** — AC-18 (1)
   (ordre des voies, restauration en dernier recours), (2) (clé dont le créateur est désactivé), (4) (familles,
   adresse mappée, mandataire invalide réparé à l'écran), (5) (voie utile du 403 et du 401) — sont-elles vraies,
   sourcées à la ligne citée, et non contraires au texte publié ? Le contrôle aplati de l'AC-21, avec son second motif,
   est-il exécutable sous `ugrep` et suffisant ?

## Lentille F — faisabilité, découpage, fausses prémisses et régressions de la P3

1. **Prémisses** : chaque « établi au code » tient-il, dont les neuves de la P3 (`IpNet::contains` faux entre
   familles ; pair et entrées de `X-Forwarded-For` canonisés ; un réseau mappé `/96` ramené à `0.0.0.0/0` ; le refus du
   créateur avant `touch_last_used` ; `list_by_company` sans jointure ; `validate_settings` qui refuse au `PUT` toute
   entrée que la 32-6a jugerait invalide ; la base qui garantit un tableau JSON valide) ?
2. **L'ordre et le découpage** (C-32-7-1, C-32-7-3) : recompte les modules au grain du kickoff ; C-32-7-25 écarte un
   état à l'écran parce qu'il élargirait la story — le raisonnement tient-il ? Reste-t-il un AC, une tâche, une ligne
   du § *À recontrôler* ou du registre qui suppose une fusion absente ou une branche « sinon » sur un prérequis ?
3. **Fidélité aux décisions de Guy et de l'orchestrateur** (listes en tête) : chacune portée à l'écran **et** au manuel
   quand elle le demande ; rien de contraire ; les décisions neuves de la P3 appliquées sans résidu.
4. **Tests, E2E et recette** : chaque AC est-il vérifiable tel qu'écrit, dont les tests par famille de l'AC-6 (b) et
   (c), le troisième état de l'AC-5, la recette SQL de la capture (22) et la capture (4) dans ses deux familles ? La
   liste des **29** captures couvre-t-elle **chaque** état touché, dont ceux de la P3 ?
5. **La garde G23** (AC-25) : réalisable, témoins suffisants (inchangée en P3 — contrôle qu'aucune phrase neuve ne la
   contredit).
6. **Décomptes** : recompte à la source — 25 AC (17 / 7 / 1), 9 tâches, 26 mutations (M1 à M26), 34 reports (24
   rattachés, 10 écartés ou sans objet), **25 choix**, 29 captures, 21 faits, 4 modules ; et le trend P3 0/0/2/9 contre
   les deux rapports P3.
7. **Régressions de la remédiation P3** (axe **obligatoire** — *la sévérité se déplace vers ce qu'on vient d'écrire* ;
   la P2 avait trouvé un MEDIUM né de la remédiation P1) : lis le diff de la remédiation
   (`git diff bf07e3d2 HEAD -- _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md
   _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md`) et cherche ce qu'il a cassé ou contredit :
   une phrase nouvelle contraire à une ancienne restée (par exemple « toute adresse », « tout client », « aucune voie »,
   « Active », « reconnue », l'ordre des voies de réparation, l'AC-5 qui disait « deux états ») ; une référence croisée
   devenue fausse (AC, mutation, capture, demande — la **10** est neuve —, choix, fait) ; un numéro de ligne inexact (la
   P3 en a corrigé plusieurs, relis-les) ; une entrée passée du Change Log ou une entrée du registre **réécrite** au
   lieu d'être annotée (interdit — C-32-7-24 doit être annoté, non réécrit) ; une phrase prescrite au manuel qui
   affirme plus que sa source (par exemple « rendre au compte son état ou son rôle rouvre » les clés). **Et ce que la
   fiche omet** : un état d'écran, un refus du serveur, un texte publié, une garde existante du frontend, un risque
   absent de *Ce que les gardes ne voient pas*.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée — `grep -nF` pour toute présence ou absence), correction proposée. Un bilan
d'en-tête **cohérent avec le corps**. Pour chaque finding, dis s'il **naît de la remédiation P3** (oui / non). ⛔ **La
liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**,
par l'outil d'écriture de fichiers, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/327-validate-p4-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p4-F.md`.
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
`frontend/node_modules/svelte-sonner/` ; le registre cargo (`~/.cargo/registry/src/`) ne se lit qu'en lecture, pour
`ipnet` ; les worktrees voisins ne se lisent qu'en lecture. Autorisés : lecture de fichiers, `grep`/`command grep`,
`sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, les deux `gh issue view` ci-dessus, et les filtres
`tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une commande est refusée, arrête-toi et
dis-le au rapport.

---

## Annexe — le prompt de la remédiation P3, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh. Tu travailles **uniquement** dans le worktree
> `/home/gcorbaz/devel/kesh-32-7`, sur sa branche, et tu écris en français.
>
> Applique en entier les consignes de `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`. En particulier :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push, ni `gh` qui écrit, ni prepare-release ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture reste permise ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> **Tâche.** Remédier la validation P3 de la fiche `_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`. Les rapports, à lire en entier :
> - `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p3-R.md` (0/0/1/2) ;
> - `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p3-F.md` (0/0/1/7).
>
> Vérifie au code la prémisse de chaque finding, et écris au Change Log ce que tu réfutes.
>
> **Décisions de l'orchestrateur :**
>
> **R1 — familles d'adresses.**
> - Écran, manuel et tests de l'AC-6 : `0.0.0.0/0` admet tout client IPv4, `::/0` tout client IPv6. Pour admettre les deux familles, il faut les deux entrées.
> - Même correction pour un mandataire `/0` (AC-18 (4)).
> - Vérifie au code de `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6`, PR #760, que le pair est bien ramené à sa forme canonique : une adresse IPv4 mappée dans IPv6 doit compter comme IPv4. Écris ce fait sourcé.
>
> **F1 — clé dont le créateur est désactivé ou n'est plus Administrateur.**
> - Ajoute la phrase au manuel (AC-18 (2)), la ligne dans « Ce que les gardes ne voient pas », et une mention à l'AC-8 (b) : l'écran montre la clé « active » alors qu'elle n'authentifie plus.
> - Choisis le plus simple. L'écran peut-il le dire sans afficher le créateur (C-32-7-16) ? Par exemple, un état « inopérante » calculé côté serveur, si la route le fournit déjà ou peut le fournir sans élargir la 32-7. Sinon, tu l'écris seulement au manuel et à la liste des gardes.
> - Inscris la décision au registre.
>
> **LOW :** tous à intégrer.
> - F2 : mettre en premier la suppression de la ligne par SQL, la moins coûteuse. Corriger « aucune voie dans Kesh » en « aucune voie depuis l'écran ». Dire que la restauration remplace toute l'installation et refuse une sauvegarde d'une autre version.
> - F7 : une entrée invalide de `trusted_proxies` éteint le serveur ; écrire la réparation par l'écran.
> - R2 : renvoyer à la ligne du manuel qui porte l'information.
> - R3 = F4 : remplacer la liste fermée par « toutes les entrées de l'export réel ».
> - F5, F6 et F8 : à intégrer aussi.
>
> **Après le patch :**
> - greppe le symptôme, c'est-à-dire la valeur : `0.0.0.0/0`, `::/0`, `tout client`, `aucune voie`, `neuf méthodes` ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris une entrée P3 neuve avec le trend P2 0/0/5/25 → P3 0/0/2/9, en déclarant que les deux MEDIUM sont des omissions d'origine et que les LOW sont nés de la P2.
>
> **Livrables :**
> - un commit `refs #723`, sans mot-clé de fermeture ;
> - le prompt de la P4, `_bmad-output/implementation-artifacts/327-validate-prompt-p4.md`. C'est une passe complète, R et F, sous **Opus**, en contexte frais, avec les rapports `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p4-{R,F}.md`. Mêmes exigences et interdits que la P3, plus un axe obligatoire sur les régressions de cette remédiation. Recopie ce prompt-ci en annexe ;
> - un rapport final : commit, décisions, réfutations, demandes.
