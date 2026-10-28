# Prompt — validation P1, Story 32-7 (l'écran « Serveur MCP » et la section du manuel)

*Versionné le 2026-10-25. Passe **complète** (première passe). **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-7`, branche `story/32-7` (partie de `main` `966baed5`). Fiche :
`_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (25 AC en trois groupes : A l'écran, B le manuel,
C la garde G23) ; registre : `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-327.md` (C-32-7-1 à 20).
Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-7*, § *Points d'attention*, **§ *Réponses de
Guy (2026-10-24, soir)*** et **§ *Réponses de Guy (2026-10-25)***) ; spike `_bmad-output/planning-artifacts/
epic-32-spike-mcp.md` (§ 2.1, § 2.4, § 6, *Réponses*) ; fiche `32-5-cles-et-reglages-mcp.md` (même worktree). **Fiches
voisines, non fusionnées, en lecture seule dans leurs worktrees** : `/home/gcorbaz/devel/kesh-32-6/_bmad-output/
implementation-artifacts/32-6-endpoint-mcp.md`, `/home/gcorbaz/devel/kesh-32-8/…/32-8-validation-avant-envoi.md`,
`/home/gcorbaz/devel/kesh-32-9/…/32-9-outils-de-lecture.md`, `/home/gcorbaz/devel/kesh-32-10a/…/32-10a-prompts-en-base.md`.
Règles : `CLAUDE.md` (§ *Recette visuelle*, § *Review Iteration Rule* et *La passe ciblée* — une boucle ne se clôt que
sur une passe sans finding au-dessus de LOW —, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON
RÉSOLUS*, *Règle de splitting préventif*, la règle E2E, *Recompter ses propres comptes rendus*), `docs/agents-
orchestration.md`, `docs/testing.md` § *Les échecs attendus*.

**Contraintes à tenir pour vraies et à contrôler** (décisions de Guy) : création des clés par l'**Administrateur**
seul ; portées `books:read`, `invoices:read`, `contacts:read`, `documents:read`, `prompts:write` ; politique de
validation **par clé**, `required` par défaut ; serveur **désactivé** par défaut ; filtre d'adresse désactivable, et
derrière Traefik ou Docker **il ne filtre rien — l'écran et le manuel le disent** ; une restauration rend les réglages
**tels quels** ; **pas de limite de débit** ; la sécurité relève de l'infrastructure (jamais exposé sur Internet, VPN) ;
Q11 (a) (aucun second utilisateur MariaDB) ; Q10 (c) (la poignée). Au plus cinq modules. Aucune donnée réelle.

⚠️ **L'ordre** : la 32-7 se développe **après** des stories non fusionnées. Ce que la fiche prescrit d'en écrire est
vérifiable **contre leurs fiches**, non contre leur code : un écart entre la fiche 32-7 et une fiche voisine est un
finding ; l'absence d'un code que la fiche dit « à recontrôler à l'étape 0 » n'en est pas un.

## Lentille R — la spécification contre le dépôt et les fiches voisines

Pour chaque affirmation, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**. Axes, tous à exercer :

1. **Les faits 1 à 20** (§ *Ce que le code dit*) : chaque numéro de ligne et chaque citation, à `966baed5`. En
   particulier : la forme de `McpKeyResponse` et de `CreateMcpKeyResponse` (`#[serde(flatten)]`) ; que les dates des
   clés sortent **sans** fuseau et `updatedAt` **avec** ; qu'une entrée vide est bien refusée par
   `canonical_network` ; les défauts de `mcp_server_settings.rs` ; l'absence de journal de corps (refais le `grep`) ;
   les lignes du manuel (fait 16, 17) et de `docs/api-external.md` (fait 18).
2. **Les reports** (§ *Inventaire des reports*, R1 à R33) : refais l'inventaire **toi-même** — `grep -n '32-7'` sur
   l'epic, la fiche 32-5, les quatre fiches voisines, la spike, et la lecture de #738 **par la fiche** (pas de `gh`) —
   et cherche un report **absent** du tableau, un report rattaché à un AC qui ne le porte pas, un « écarté » sans
   raison juste. Contrôle chaque ligne citée des fiches voisines.
3. **Fidélité aux fiches voisines** : chaque phrase que la fiche prescrit d'écrire à l'écran ou au manuel sur le
   comportement de la 32-6 (route absente, 401 unique, `Origin`, mandataires, P7-H3, 415, 400, 1 Mio, 30 s, B4), de la
   32-8 (politique, 24 h, 20 envois, erreur d'outil, `sec:mcp-envois`), de la 32-9 (ce que `contacts:read` démasque,
   `sec:mcp-outils`, `kesh_lire_envoi`) et de la 32-10a (`prompts:write`) dit-elle **ce que ces fiches prescrivent** ?
   Une phrase contraire à une fiche voisine est un finding.
4. **La propriété du secret** (AC-3, AC-11, G-a, G-b, mutations M1 à M6, M16) : les cinq « ni » sont-ils tous
   testables tels qu'écrits ? Un test passerait-il **à vide** (champs privés `#` et `JSON.stringify` — la fiche
   l'anticipe-t-elle juste ? espions de `console` sous Vitest ; `Storage.prototype.setItem` sous jsdom) ? Chaque
   mutation est-elle tuée par le test **nommé** ?
5. **Les gardes lexicales** (AC-15, règle commune) : refais la mesure écrite de G-a (`grep -rnE …` de l'AC-15) et
   compare ; `masquerCommentaires` (`frontend/src/lib/shared/i18n-literal-reader.js:261`) écarte-t-il vraiment le
   commentaire témoin ? G-b et son témoin vert sont-ils réalisables ? La cardinalité du périmètre est-elle définie ?
6. **Les dates** (AC-4) : `new Date('…T10:00:00.123')` est-il bien lu en heure locale ? Le changement de
   `process.env.TZ` en cours de processus est-il pris par Node (le témoin suffit-il) ? L'aller-retour de l'expiration
   en fin de journée locale tient-il face au refus « expiration passée » du serveur ?
7. **Le manuel — axe obligatoire, PDF compris** : la place proposée de `\section{Le serveur MCP}` (classe `article`,
   l. 5 ; `\section{Monitoring et logs}` l. 3017) ; les renvois de l'AC-19 (lignes 1414, 1416, 2237, 2467, 2472, 2898 ;
   `user-manual.tex` 190, 3204-3205, 3214-3215) ; ce que le texte actuel **et le PDF aplati** disent déjà du MCP :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE '.{0,80}(serveur MCP|kesh_mcp).{0,120}'`
   et de même pour `user-manual.pdf`. Un passage existant devient-il faux du fait de la 32-7 ? Une phrase prescrite à
   l'AC-18 est-elle **contraire** au texte publié (sauvegarde l. 2237, remise à zéro l. 1414) ? `admin-manual.tex:2935`
   et `docs/api-external.md:185`, `:831` : la fiche a-t-elle raison de ne pas les porter (C-32-7-14), selon les fiches
   32-6 et 32-10a ?

## Lentille F — faisabilité, découpage et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse de l'epic, de la spike ou d'une fiche voisine
   reprise sans vérification (par exemple : l'adresse `/mcp` exacte, le 401 fixe, `seedTestState`, la route
   `/admin/mcp/sends`) ?
2. **Découpage** (C-32-7-1, C-32-7-3) : recompte les modules au grain du kickoff ; l'ordre de rechange est-il vraiment
   mécanique (les trois exceptions sont-elles les seules dépendances du groupe A envers des stories non fusionnées ?) ;
   un AC du groupe A dépend-il en fait du groupe B ?
3. **Fidélité aux décisions de Guy** (liste en tête) : chacune portée à l'écran **et** au manuel quand la décision le
   demande ; rien de contraire (une limite de débit, un second utilisateur MariaDB, un filtre présenté comme une
   protection, une portée d'écriture).
4. **Tests, E2E et recette** : chaque AC est-il vérifiable tel qu'écrit ? L'E2E (AC-16) est-il montable avec le motif
   `frontend/tests/e2e/api-keys.spec.ts` et ses helpers ? Pollue-t-il les specs suivantes ? Respecte-t-il la garde des
   sélecteurs traduits (`frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts`) ? La liste des captures (AC-17)
   couvre-t-elle **chaque** état touché que les AC créent ?
5. **La garde G23** (AC-25) : réalisable dans `crates/kesh-api/tests/textes_coherents.rs` (accès aux constantes de
   `kesh-db`, motif de G22 l. 1673) ? Ses témoins empêchent-ils de passer à vide ? Les mutations M17 à M19 la font-elles
   rougir ?
6. **Décomptes** : recompte à la source — 25 AC (17 / 7 / 1), 9 tâches, 19 mutations, 33 reports (24 rattachés, 9
   écartés ou sans objet), 20 choix, 4 modules, 20 faits.
7. **Ce que la fiche omet** : un état d'écran, un refus du serveur, un texte publié (README, `website/`, CHANGELOG), une
   garde existante du frontend (boutons rouges, i18n, sélecteurs) que la story touchera sans la nommer ; un risque
   absent de *Ce que les gardes ne voient pas*.

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La
liste des axes exercés ET non exercés** (R1 à R7, F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, par
l'outil d'écriture de fichiers, un seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-R.md`
et `/home/gcorbaz/devel/kesh-gate-logs/327-validate-p1-F.md`. Dernier message : chemin, bilan par sévérité, une ligne
par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad. **Aucune
redirection** (`>`, `>>`, `2>`, `tee`, `<<`), pas même vers un fichier jetable — un fichier créé par erreur se déclare
au rapport, il ne se supprime pas. **Aucun interpréteur** : ni `python3`, `python`, `perl`, `ruby`, `node`, `deno`, ni
**`awk`**, ni `sed` autrement qu'en `sed -n`. **Aucun `gh`**, sous aucune forme (l'issue #738 se lit par la fiche).
Aucun `git checkout`/`switch`/`commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`. Aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, **`scripts/test-fast.sh`**,
**`scripts/mem-guard.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `mariadb`/`mysql`,
`sed -i`. `pdftotext` vers la sortie standard seulement (`-` en sortie). Rien de `~/kesh-test/`, aucune donnée réelle.
Le dépôt principal `/home/gcorbaz/devel/kesh` est hors de ton périmètre ; les worktrees voisins ne se lisent qu'en
lecture. Autorisés : lecture de fichiers, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`, et
les filtres `tr`, `sort`, `cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.
