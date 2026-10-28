# Prompt — validation P2, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Opus**. Lecture seule stricte. La P1 (Sonnet ×2) a rendu 0/0/5/19 ; sa remédiation est le commit qui suit
`0dc8e419` sur la branche `story/32-7` (`git log --oneline -3`).*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC en trois groupes : A l'écran, B le manuel,
C la garde G23) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 22).
Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-R.md` et `-F.md` ; entrée « validation P1 » du Change
Log de la fiche. Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-7*, § *32-11*, § *Points
d'attention*, **§ *Réponses de Guy (2026-10-24, soir)*** et **§ *Réponses de Guy (2026-10-25)***) ; spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 2.1, § 2.2, § 2.4, § 6, § 7) ; fiche
`32-5-cles-et-reglages-mcp.md` (même worktree). **Fiches voisines, non fusionnées, en lecture seule dans leurs
worktrees** : `/home/gcorbaz/devel/kesh-32-6/_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`,
`/home/gcorbaz/devel/kesh-32-8/…/32-8-validation-avant-envoi.md`, `/home/gcorbaz/devel/kesh-32-9/…/32-9-outils-de-lecture.md`,
`/home/gcorbaz/devel/kesh-32-10a/…/32-10a-prompts-en-base.md`. Règles : `CLAUDE.md` (§ *Recette visuelle*, § *Review
Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding au-dessus de LOW ; une
remédiation de l'orchestrateur ou d'un agent n'a aucun privilège de justesse —, *Le prompt d'une passe doit NOMMER le
manuel*, *Inventorier les sites NON RÉSOLUS*, *Règle de splitting préventif*, la règle E2E, *Recompter ses propres
comptes rendus*), `docs/agents-orchestration.md`, `docs/testing.md` § *Les échecs attendus*.

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
- la garde « 2 bis » de la 32-6 **non retenue** ;
- **G23** réservé à la 32-7 ;
- le compte de `admin-manual.tex:2935` à la 32-10b, puis à la 32-11 ;
- `sec:mcp-client` **appartient à la 32-7** ; la 32-11 l'amende en place (C-32-7-22).

⚠️ **L'ordre** : ce que la fiche prescrit d'écrire sur les stories prérequises est vérifiable **contre leurs fiches**,
non contre leur code : un écart entre la fiche 32-7 et une fiche voisine est un finding ; l'absence d'un code que la
fiche dit « à recontrôler à l'étape 0 » n'en est pas un.

⚠️ **Outillage de ce poste** : `grep` est une fonction qui lance `ugrep`, qui refuse les répétitions bornées larges
(`.{0,80}` → « exceeds complexity limits »). Employer `command grep`, et des fenêtres de 30 au plus.

## Lentille R — la spécification contre le dépôt et les fiches voisines

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Toute affirmation de
présence ou d'absence s'appuie sur une sortie `grep -nF` (ou `command grep -nF`) **copiée**. Axes, tous à exercer :

1. **Les faits 1 à 20** (§ *Ce que le code dit*), à `966baed5` ; en particulier le fait 19 corrigé en P1 (CHANGELOG) et
   les lignes neuves citées en P1 : `i18n-un-repli-par-cle.test.ts:51-72` et `:216`,
   `i18n-repli-divergent-actif.test.ts:110`, `:188`, `:216-235`, `:248-252`, `textes_coherents.rs:1688`,
   `routes/test_endpoints.rs:172`, `kesh-db/src/test_fixtures.rs:392-402`, `backup.rs:78`, `:83`,
   `tests/e2e/helpers/test-state.ts:33-41`, `backup-schedule.spec.ts:200-218`, `+layout.svelte:225`, `:267-270`, `:366`,
   `admin/backup/+page.ts:14`, `contacts/contacts-page.test.ts:17`, `bank-accounts-page.test.ts:701-720`,
   `tests/common/manuel.rs:14-16`, `:32-34`, `mcp_server_settings.rs:26` (entité) et `:35-38` (route),
   `admin-manual.tex:1297`.
2. **Les reports** (R1 à R33) : refais l'inventaire **toi-même** — `grep -n '32-7'` sur l'epic, la fiche 32-5, les quatre
   fiches voisines, la spike, et `gh issue view 738 --comments` (lecture seule ; les points B2 à B8 sont dans un
   commentaire) — et cherche un report absent, un rattachement faux, un « écarté » sans raison juste. Contrôle les
   reclassements de la P1 (R11, R13, R18, R28) et le décompte 22 rattachés / 11 écartés ou sans objet.
3. **Fidélité aux fiches voisines** : chaque phrase que la fiche prescrit d'écrire à l'écran ou au manuel sur le
   comportement de la 32-6 (route absente, 401 unique, `Origin`, mandataires, Traefik et Docker, P7-H3, ligne
   illisible, 415, 400, 1 Mio, 30 s, B4, en-tête d'un pair non déclaré ignoré), de la 32-8 (politique, 24 h, 20 envois,
   erreurs d'outil, `sec:mcp-envois`, `/admin/mcp/sends`, `nav-admin-mcp-sends`), de la 32-9 (`kesh_pieces`,
   `kesh_echeancier`, ce que `contacts:read` démasque, « l'IBAN n'est pas une donnée du carnet d'adresses »,
   `sec:mcp-outils`) et de la 32-10a (`prompts:write`) dit-elle **ce que ces fiches prescrivent** ? La phrase de l'AC-6 (1)
   est-elle égale, au fond, à celle de l'AC-18 (4) et du point 2 de la 32-6 ?
4. **La propriété du secret** (AC-3, AC-9 (d)-(e), AC-11, G-a, G-b, mutations M1 à M6, M16, M20, M21, M23) : les cinq
   « ni » sont-ils testables tels qu'écrits **après** la P1 — `vi.stubGlobal('indexedDB', …)` et `vi.unstubAllGlobals()`,
   `vi.spyOn(Storage.prototype, 'setItem')`, `vi.spyOn(history, …)`, l'espion de `toast.*` ; chaque **témoin** prouve-t-il
   que son espion voit ? La signature fixée de `SecretUnique` est-elle cohérente partout (AC-11, Dev Notes, schéma) ?
   Chaque mutation est-elle tuée par le test **nommé** (M1 par la sonde de (ii), M4 par le double `GET`) ?
5. **Les gardes lexicales et i18n** (AC-14 (a)-(c), AC-15) : refais la mesure de G-a ; G-b, telle que réécrite (blocs
   `<script>` d'instance contre `<script module>` / `context="module"`, occurrence hors bloc), est-elle réalisable et ses
   témoins le sont-ils ? L'extension de G13 (AC-14 (a), C-32-7-21) **ferme-t-elle vraiment la classe** — égalité des
   ensembles relevé ↔ FTL, clés bâties par gabarit, replis non littéraux, clés à `{ $x }`, `nav-admin-mcp` — et
   repose-t-elle sur des fonctions réelles du fichier (`replisParCle`, `valeurDuCatalogueFr`) ? Le commentaire de G22
   est-il bien faux, et la phrase de remplacement juste ? La cardinalité exacte est-elle définie ?
6. **Les dates** (AC-4) : inchangées en P1 hors le précédent cité ; contrôle que le précédent
   `bank-accounts-page.test.ts:701-720` est bien un changement de `process.env.TZ` en cours de processus.
7. **Le manuel — axe obligatoire, PDF compris** : la place de `\section{Le serveur MCP}` ; les renvois de l'AC-19 ; ce que
   le texte actuel **et le PDF aplati** disent déjà du MCP :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | command grep -oiE '.{0,30}(serveur MCP|kesh_mcp).{0,30}'`
   et de même pour `user-manual.pdf`. Un passage existant devient-il faux ? Une phrase de l'AC-18 est-elle **contraire**
   au texte publié ? L'AC-18 (8) **réécrit en P1** : chaque geste y est-il sourcé à la ligne de la spike qu'il cite
   (§ 2.1 l. 211-213, l. 252, l. 302-304 ; § 2.2 l. 255-260 ; l. 206-207 ; Q9 l. 1027) ? Rien n'y affirme-t-il de
   Hermes plus que la spike, ou de la portée `user` ce que la spike ne dit pas ? L'AC-21 (commande corrigée, `25\,000`)
   est-il exécutable sur ce poste ?

## Lentille F — faisabilité, découpage, fausses prémisses et régressions de la P1

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse reprise sans vérification (dont les nouvelles de la
   P1 : `seedTestState` → `truncate_all`, `navTestid`, groupe *Administration* replié, `browser` dans le `load`,
   absence d'`indexedDB` sous jsdom 29.0.2) ?
2. **L'ordre et le découpage** (C-32-7-1, C-32-7-3 récrit) : recompte les modules au grain du kickoff (l'amendement de
   `i18n-repli-divergent-actif.test.ts` et du commentaire de G22 change-t-il le compte ?) ; reste-t-il **un** AC, une
   tâche, une ligne du § *À recontrôler* ou du registre qui suppose une fusion absente ou un ordre de rechange ? Un
   prérequis manque-t-il à la liste des neuf, ou y figure-t-il à tort ?
3. **Fidélité aux décisions de Guy et de l'orchestrateur** (listes en tête) : chacune portée à l'écran **et** au manuel
   quand elle le demande ; rien de contraire ; la branche « 2 bis » entièrement retirée ; Q9 et R11 respectés.
4. **Tests, E2E et recette** : chaque AC est-il vérifiable tel qu'écrit ? L'E2E (AC-16) est-il montable — ouverture du
   groupe replié, création du Comptable, `initialize` puis 401 ? La liste des 28 captures couvre-t-elle **chaque** état
   touché ? Le test `formulaire_a_l_ouverture_requise_et_aucune_portee` tue-t-il M20 et M21 ?
5. **La garde G23** (AC-25) : réalisable — extraction sur le source **brut**, premier code de chaque ligne, accès direct
   à `DEFAULT_ALLOWED_NETWORKS` ? Ses témoins empêchent-ils de passer à vide ? M17 à M19 la font-elles rougir ?
6. **Décomptes** : recompte à la source — 25 AC (17 / 7 / 1), 9 tâches, 25 mutations (M1 à M25), 33 reports (22
   rattachés, 11 écartés ou sans objet), 22 choix, 28 captures, 4 modules, 20 faits ; et le trend P1 0/0/5/19 contre les
   deux rapports P1.
7. **Régressions de la remédiation P1** (axe obligatoire — *la sévérité se déplace vers ce qu'on vient d'écrire*) : lis
   le diff de la remédiation (`git diff 0dc8e419 HEAD -- _bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md
   _bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md`) et cherche ce qu'il a cassé ou contredit :
   une phrase nouvelle contraire à une ancienne restée, une référence croisée (AC, mutation, capture, demande, choix)
   devenue fausse, un numéro de ligne inexact, une entrée passée du Change Log réécrite (interdit), une demande à
   l'orchestrateur close à tort ou ouverte sans motif. **Et ce que la fiche omet** : un état d'écran, un refus du
   serveur, un texte publié, une garde existante du frontend, un risque absent de *Ce que les gardes ne voient pas*.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée — `grep -nF` pour toute présence ou absence), correction proposée. Un bilan
d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle,
« 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe pas encore : son absence n'est pas un
défaut. Rapport écrit **d'un seul jet**, par l'outil d'écriture de fichiers, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad. **Aucune
redirection** (`>`, `>>`, `2>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. **Aucun interpréteur** : ni `python3`, `python`, `perl`, `ruby`, `node`, `deno`, ni
**`awk`**, ni `sed` autrement qu'en `sed -n`. **Aucun `gh` qui écrit** (`pr`, `issue create|edit|close|comment`,
`api` en écriture…) ; seul `gh issue view 738 --comments` est permis. Aucun `git
checkout`/`switch`/`commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`. Aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, Playwright, `docker`, `curl`,
`mariadb`/`mysql`, `sed -i`. `pdftotext` vers la sortie standard seulement (`-` en sortie). Rien de `~/kesh-test/`,
aucune donnée réelle. Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees voisins ne
se lisent qu'en lecture. Autorisés : lecture de fichiers, `grep`/`command grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, `gh issue view 738 --comments`, et les filtres `tr`, `sort`,
`cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard. Si une commande est refusée, arrête-toi et dis-le au rapport.

---

## Annexe — le prompt de la remédiation P1, recopié pour que la boucle soit rejouable

> Tu es l'agent de spécification de la Story 32-7 de Kesh, l'écran d'administration du serveur MCP. Tu travailles
> **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-7`, branche courante, et tu écris en français.
>
> ## Consignes communes
> Lis `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md`, qui s'applique en
> entier.
> - Chemins absolus, et `&&` après chaque cd.
> - Le dépôt principal est en lecture seule.
> - Pas de push, pas de `gh` qui écrit, pas de prepare-release.
> - Aucun cargo, npm, Playwright, docker ni curl : c'est une fiche.
> - La lecture (grep, sed -n, cat, `gh issue view --comments`) est permise.
> - Si une commande est refusée, tu t'arrêtes et tu le dis.
> - N'ouvre jamais `~/kesh-test/`.
>
> ## La tâche
> Remédie la validation P1 de la fiche `_bmad-output/implementation-artifacts/32-7-*.md`. Les rapports sont
> `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-R.md` (0/0/1/12) et
> `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-F.md` (0/0/4/7) ; lis-les en entier. Vérifie au dépôt la prémisse
> de chaque finding avant de l'appliquer, et écris au Change Log celles que tu réfutes.
>
> **Décisions de l'orchestrateur, à appliquer d'abord :**
> - **F1 — l'ordre de rechange est retiré.** La 32-7 se développe seulement après la fusion de ses prérequis, dans
>   l'ordre nominal : 32-6a, 32-6b, 32-8a, 32-8b0, 32-8b, 32-8c, 32-9a/b, 32-10a, puis 32-7. L'étape 0 le vérifie :
>   `git log origin/main` doit porter chacune. Récris C-32-7-1 dans ce sens, et retire les AC conditionnels et les
>   exceptions qui ne servaient que la rechange. R7 suit la même logique : la garde « 2 bis » est écartée, donc la
>   branche conditionnelle d'AC-18 (4), R13 et les Demandes 1, 3, 4 et 5 sont retirées ou closes, avec une ligne au
>   Change Log.
> - **F2 — le repli égal à fr-CH.** Choisis la solution la plus simple qui ferme la classe et qui s'appuie sur une garde
>   réelle, puis écris-la au registre. L'option préférée : ajouter `mcp-server-` aux préfixes de
>   `i18n-un-repli-par-cle.test.ts`, s'il impose bien l'égalité au fr-CH ; à vérifier en lisant ce test. Sinon, ajoute
>   un test propre, avec ses témoins rouge et vert. Corrige le commentaire de G22.
> - **F3 = R2 — `<script module>`.** G-b doit couvrir le bloc `<script module>` des `.svelte`, avec un témoin rouge
>   `.svelte` et une mutation.
> - **F4 — les défauts du formulaire.** Nomme les tests et les mutations : la politique `required` par défaut
>   (mutation : `exempt` par défaut) et l'absence de portée cochée (mutation : une portée précochée).
> - **R1 — `indexedDB` sous jsdom.** Utilise `vi.stubGlobal('indexedDB', …)`. Chaque espion (`Storage.prototype`,
>   `history`, `indexedDB`) reçoit un appel témoin posé par le test lui-même, pour prouver qu'il voit.
> - **Les LOW (R3 à R13, F5 à F11)** sont intégrés à la fiche. Ceux qui relèvent de l'étape 0 vont au § *À recontrôler
>   à l'étape 0*, chacun avec sa correction attendue.
>   - **R11** : la Q9 (Hermes) n'a pas de réponse. Le manuel n'écrit donc rien de Hermes qui ne soit sourcé, et la
>     configuration de Claude Code reste marquée « non éprouvée » au manuel ; c'est une décision acceptée de Guy.
>   - **F8** : le recoupement avec *Brancher un agent* de la 32-11 se règle en donnant la propriété à une seule story.
>     Choisis, écris-le, et note-le dans les demandes à l'orchestrateur.
>
> Après le patch, grep le symptôme : la valeur, sur la fiche et sur les fiches voisines. Les fiches voisines se lisent
> seulement ; ce qu'il faudrait y corriger va à la note pour l'orchestrateur.
>
> Recompte à la source : AC, tâches, mutations, reports, choix, modules.
>
> Tu ne réécris aucune entrée passée. L'entrée P1 du Change Log est nouvelle, avec le trend P1 : 0/0/5/19, doublons
> R2 = F3 et R8 ≈ F5 comptés une fois par lentille.
>
> ## Livrables
> - Un commit `refs #723`, sans aucun mot-clé de fermeture.
> - Le prompt de la P2 : `_bmad-output/implementation-artifacts/327-validate-prompt-p2.md`. C'est une passe complète,
>   avec deux lentilles R et F, modèle **Opus**, en contexte frais. Le prompt exige : les axes exercés et non exercés ;
>   les sorties `grep -nF` pour toute présence ou absence ; la vérification du manuel, PDF compris ; les interdits de
>   commandes (cargo, npm, docker, curl, interpréteurs, awk, redirections, `scripts/prepare-release.sh`, `gh` qui
>   écrit) ; les rapports dans `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p2-{R,F}.md`. Il doit aussi recopier le
>   contenu de ce prompt-ci, pour qu'il soit rejouable.
> - Un rapport final : commit, décisions, réfutations, demandes à l'orchestrateur.
