# Prompt — validation P3, Story 32-11 (brancher un agent, de bout en bout)

*Versionné le 2026-10-27. Passe **complète** (troisième passe sur huit), après la remédiation de la P2, qui a **refondu
le montage du parcours**. **Deux lentilles, R et F**, chacune en **contexte frais**, **Sonnet**. Lecture seule stricte,
**sans interpréteur**, **sans fichier temporaire**, **sans redirection**.*

Dépôt `/home/gcorbaz/devel/kesh-32-11`, branche `story/32-11` (partie de `origin/main` `ed981c7d` : 32-1 à 32-6c et 32-8
fusionnées ; 32-7, 32-9a, 32-9b, 32-10a, 32-10b **non** fusionnées). Fiche :
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md` (C-32-11-1 à 29) ; § *32-11* de
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (l. 516-546) ; clé `32-11-brancher-un-agent` de
`_bmad-output/implementation-artifacts/sprint-status.yaml`. **La remédiation à relire** : le dernier commit de la branche
(`git show HEAD --stat`, puis `git diff HEAD~1 HEAD -- <fichier>`), son entrée « validation P2 » au Change Log de la
fiche, et les rapports de la P2 : `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-R.md`,
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-F.md`. Contexte à lire : la spike
`_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 1.1, 1.3, 2, 2.1, 2.4, 6) ; la fiche 31-4
`_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (périmètre, T8, C-31-4-15) ; les fiches des stories
non fusionnées, **en lecture seule dans leurs worktrees** :
`/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md` (AC-18, AC-21, l. 880-895),
`/home/gcorbaz/devel/kesh-32-9/_bmad-output/implementation-artifacts/32-9-outils-de-lecture.md` (l. 340-360, 1320-1380),
`/home/gcorbaz/devel/kesh-32-10a/_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` (l. 15-30, 195-210,
360-400, 815-860, 1380-1400), `/home/gcorbaz/devel/kesh-32-10b/_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`
(AC-7, AC-9, AC-19, AC-20) ; dans ce worktree, `32-6-endpoint-mcp.md` (l. 1259-1323), `32-8-validation-avant-envoi.md`
(l. 1425-1443). Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée*, *Inventorier les sites NON
RÉSOLUS*, *Règle de splitting préventif*, *Le prompt d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes
rendus*, *Test Locally First*), `docs/agents-orchestration.md` (§ 2, § 2.1).

**Contraintes de la story, à tenir pour vraies et à contrôler** : dernière story de l'Epic 32, **développée après la
fusion de toutes les autres** ; **documentation exécutée** — le périmètre du parcours est **clos** (AC-A2), un défaut
révélé **ne se corrige pas** dans la story (AC-A7), **aucune garde de texte nouvelle** hors du compte des routes
(AC-T1) ; **Hermes hors du périmètre** ; **aucun texte ne promet une écriture** ; **aucune donnée réelle** ; au plus
cinq modules. **Depuis la P2** : l'agent de développement joue le parcours **en client HTTP** (`curl`), **sans lancer
`claude` ni `socat`** ; les gestes de Claude Code et la capture de ses requêtes sont **la recette de Guy** (AC-A8) ; le
pilotage de `claude -p` est une **variante soumise à Guy** (point 1, Annexe), non retenue par défaut — **l'Annexe est une
esquisse, non une spécification** : ses défauts ne sont des findings que s'ils fuient dans le montage par défaut. Ce que
la fiche **prescrit** n'existe pas encore : son absence n'est pas un défaut. Ce qu'elle dit du code de stories non
fusionnées se recontrôle à leur étape 0 : vérifie qu'il figure au § *À recontrôler à l'étape 0*, non qu'il soit vrai
aujourd'hui.

## Les deux axes obligatoires, pour les deux lentilles

- ⛔ **Chaque ligne citée se rouvre.** Pour chaque `fichier:ligne` de la fiche et du registre (faits 1 à 16, AC, Dev
  Notes, § *À recontrôler*, Change Log P2), ouvre le site (`sed -n`, `grep -nF`) et **copie la sortie**. Une citation que
  le fichier ne porte pas est un finding. Les comptes écrits se **rejouent** : 130/124/4/2 (`sed -n 877,930p
  crates/kesh-api/tests/audit_route_registry.rs`), « 122 des 128 » (`grep -nF '122 des 128'
  docs/manual/fr/admin-manual.tex`), `grep -n -i mcp README.md website/*.html docs/api-external.md`.
- ⛔ **Les manuels, `.tex` et PDF aplati.** Pour chaque phrase que la fiche dit vraie ou fausse aujourd'hui, et pour
  chaque motif de l'AC-C9 : le `.tex` **et** `pdftotext docs/manual/fr/<m>.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE
  '.{60}<motif>.{80}'`, sortie copiée. La règle « 0 à la base » de l'AC-C9 se rejoue sur chaque motif nommé.

## Lentille R — la spécification contre le code, les fiches et l'epic

1. **Les faits 1 à 16**, tels que la P2 les a amendés (faits 2, 3, 5, 6, 9, 13, 14) : vrais à `ed981c7d`, ou à la ligne
   citée d'une fiche voisine ?
2. **La couverture de l'epic** : chaque élément du § 32-11 (« le parcours entier joué sur une démonstration **avec un
   client réel** », contrat en CI, `api-external.md`, manuels, brochure, restauration jouée, description détaillée,
   `README.md`, `website/roadmap.html`) a-t-il son AC ? **Le client réel** n'étant plus lancé par l'agent, l'exigence de
   l'epic est-elle tenue par la recette de Guy (AC-A8), et la fiche le dit-elle sans masquer l'écart ? Chaque report des
   fiches voisines est-il repris ou écarté par un choix écrit ?
3. **Les décisions de Guy et les règles** : la question du point 1 nomme-t-elle exactement la règle dérogée
   (`docs/agents-orchestration.md` § 2.1, `CLAUDE.md`) ? Le montage par défaut respecte-t-il « un agent n'écrit jamais hors
   de son worktree (et de `kesh-gate-logs/`) » partout (Dev Notes, AC-A1, AC-A4, AC-E2) ? La capture de la recette
   est-elle bien un geste **de Guy**, jamais de l'agent ?
4. **Le manuel prescrit** (AC-C1 à AC-C9, AC-E3) : l'AC-C1 (v) et (vi) dit-il vrai de `proposer_prompt` (fiche 32-10a,
   l. 203, 821, 837, 854, 1392) ? L'AC-C2 et l'AC-C3 sont-ils cohérents avec un parcours sans Claude Code (gestes
   « non éprouvé », observations datées après la recette) ? Les motifs de l'AC-C9 rendent-ils 0 sur les PDF de la base
   d'aujourd'hui (rejoue-les) ?
5. **Les textes publics** (AC-D1 à AC-D5) : `README.md:233`, `website/roadmap.html:43`, `:223`, `:375` — les
   corrections prescrites sont-elles exactes à la lettre, et une autre phrase « read-only » / « lecture seule » sur le
   serveur MCP reste-t-elle hors de prise (`grep -n -i -E 'read-only|lecture seule' README.md website/*.html
   docs/api-external.md docs/manual/fr/marketing-brochure.tex`) ?
6. **Les décomptes** : recompte à la source — 33 AC (A 8, B 6, C 9, D 5, E 3, T 2), 11 tâches, 22 mutations M1 à M22
   (+ MT1, MT2 ; MF1 retirée), 22 cas K dont 21 actifs (K4 retiré), 12 gestes P0 à P11 sans conditionnel, 9 requêtes
   R01 à R09, 4 requêtes E01 à E04, 16 faits, 5 points pour Guy, 29 choix (11 amendés en P2), 4 modules — par `grep -c`,
   sortie copiée. Le trend écrit au Change Log (P1 0/1/5/16 → P2 0/0/9/13) est-il cohérent avec les bilans des rapports
   P2 ?

## Lentille F — faisabilité, simplicité, tests et fausses prémisses

1. **Le parcours en client HTTP est-il jouable tel qu'écrit ?** Les neuf requêtes R01 à R09 et E01 à E04 : méthodes,
   paramètres, en-têtes (`Content-Type`, `Accept`, `MCP-Protocol-Version`, `Authorization`) — conformes à ce que
   `crates/kesh-api/src/mcp/transport.rs` et `repartiteur.rs` exigent (lis-les) ? L'argument de `kesh_lire_envoi`
   (`repartiteur.rs:73-77`) ? Le 401 de R08 arrive-t-il avant le corps ? La forme `curl … -H @fichier` est-elle renvoyée à
   l'étape 0 ? Le secret de la clé reste-t-il hors du journal et du dépôt ?
2. **Le périmètre est-il clos ?** Une phrase ouvre-t-elle une requête, une session ou un geste hors du tableau de l'AC-A2
   et de l'AC-E2 ? L'Annexe ou la recette de Guy offrent-elles une porte au développement (un agent qui lancerait
   `claude` ou `socat` « pour voir ») ? Le rejeu T-8 (AC-A6) est-il borné, et jouable (troisième instance sur `P`) ?
3. **AC-A5** : le contrôle (b) (aucun montant dans R01 à R06) et son témoin (R07) peuvent-ils passer à vide (format des
   montants, échappement JSON, réponse en `text/event-stream`) ? Le contrôle (c) (octet pour octet) est-il réalisable ?
4. **Le test de contrat** (AC-B1 à AC-B6) : la règle de rangement de l'AC-B2 est-elle **appliquée sans contradiction**
   dans la table K, l'AC-B3 et les mutations — une case encore `CLAUSE` qui n'est ni citée par la spike en MUST ni une
   exigence de base nommée ? Une case `DECISION_KESH` dont la spike cite pourtant le MUST ? K14 et son témoin (amorçage
   des standards dans le routeur de test, `tests/common/mcp.rs`) ; K21 sans branche « retiré ».
5. **Les mutations M19 à M22** : réalisables sur `repartiteur.rs` (`Methode::depuis`, l. 106-116), `textes.rs:101`,
   `textes.rs:55`, et tuées par le cas **nommé** ? M20 suppose-t-elle un `prompts/list` qui lit les standards — vrai au
   code de la 32-10a, ou à renvoyer à l'étape 0 ? Les cas sans mutation sont-ils tous écrits en angle mort ?
6. **Les points pour Guy** : le point 1 est-il une vraie question, avec la recommandation et ce que coûte chaque
   réponse ? Le point 3 dépend-il proprement du point 1 ? Un oui au point 1 arrête-t-il l'étape 0 (§ *À recontrôler*) ?
7. **Le découpage** : quatre modules ; un fichier prévu hors d'eux est-il rendu nécessaire ?

## Axe commun aux deux lentilles — les régressions de la remédiation P2

⛔ La P2 a été remédiée par l'agent de spécification ; ses patches n'ont été relus par personne, et elle a **refondu**
la partie A. Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire*. Pour chaque finding de
la P2 (R1 à R11, F1 à F11), lis le site remédié et dis : (1) la correction tient-elle, ou le retrait est-il justifié ;
(2) a-t-elle introduit un défaut — en particulier :
- **Les résidus du montage retiré** : `grep -nE 'claude|socat|session|--resume|transcript|S[1-4]\b|P3-direct|P-pièges|
  fixture|MF1|\bK4\b|P\+[12]|~/\.claude|stream-json'` sur la fiche et le registre, sortie copiée — chaque occurrence
  hors de l'Annexe, du point 1, des sites qui l'excluent et des relevés datés est-elle un résidu ?
- **La cohérence de la partie A refondue** avec les parties B à E, T-1 à T-10, les Gates, le § *Ce que les gardes ne
  voient pas*, les Dev Notes, le point 1 et le registre (C-32-11-2, 4, 5, 6, 9, 22, 24, 26 amendés ; 28, 29 neufs).
- **L'oracle réétiqueté** (AC-B2, table K, C-32-11-27 amendé) : chaque étiquette relue contre la spike l. 80-100 et
  161-163.
- **R1** : AC-C1, AC-D2, AC-D3 et le registre (C-32-11-15) ; la ligne `roadmap.html:43` est-elle touchée **seulement**
  à la formule prescrite ?
- **Les citations ajoutées ou corrigées** (fait 5 `:22`, `:26-28` ; fait 13 `:2941` ; origine `:1330-1333`, `:1378` ;
  32-7 l. 887-889 ; 32-10a l. 23, 371 ; spike l. 85-96, 253) se rouvrent ; la **réfutation partielle** de R10 sur
  `:1378` est-elle juste ?
- **L'historique** : aucune entrée passée du Change Log réécrite (`git diff HEAD~1 HEAD` sur la fiche : une seule ligne
  ajoutée au tableau du Change Log) ; registre en ajouts seuls (`git diff HEAD~1 HEAD --
  _bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md | grep -c '^-[^-]'` → 0).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt ou de la fiche
voisine, preuve (commande et sortie copiée ; `grep -nF` pour toute affirmation de présence ou d'absence), correction
proposée, et, pour chacun, **s'il naît de la remédiation P2** ou d'un défaut d'origine. Un bilan d'en-tête **cohérent
avec le corps**. ⛔ **La liste des axes exercés ET non exercés** (R1 à R6 ou F1 à F7, l'axe des régressions et les deux
axes obligatoires) — sans elle, « 0 finding » ne compte pas comme passe. Rapport écrit **d'un seul jet**, un seul
fichier par lentille, avec l'outil d'écriture de fichier :
`/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ **Rappel : la P1 et la P2 ont enfreint ces interdits** (P1 : `claude --version`, un `awk` redirigé vers `/dev/null`,
une substitution de processus ; P2 : une redirection `2>&1`). **`claude` et `socat` ne se lancent sous aucune forme**,
pas même `--version` ou `--help` ; **aucune redirection**, ni `>`, ni `>>`, ni `2>`, ni **`2>&1`**, ni `2>/dev/null`,
ni `<(…)`, ni `| tee` ; une erreur de commande se lit telle qu'elle s'affiche.

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad — un fichier créé par
erreur se déclare au rapport, il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande
qui écrit, compile, exécute ou joint le réseau : `scripts/*` (dont **`scripts/prepare-release.sh`** et
**`scripts/test-fast.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`, `docker`, `curl`, `wget`, `socat`,
**`claude`**, **`python3`, `python`, `perl`, `ruby`, `node`, `awk`, `jq`** et tout autre interpréteur, `mariadb`/`mysql`,
`sed -i`, `git commit|add|stash|fetch|pull|reset|rebase|tag|push|worktree`, et **toute commande `gh` qui écrit** (`pr`,
`issue create|edit|close|comment`, `api` en écriture) — la lecture d'une issue (`gh issue view <n>`) est permise.
`pdftotext` vers la sortie standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal
`/home/gcorbaz/devel/kesh` et les autres worktrees sont **en lecture seule**. Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `pdftotext … -`, `gh issue view`, et les filtres en lecture seule `tr`, `sort`,
`cut`, `head`, `tail`, `wc`, `uniq` vers la sortie standard.

## Annexe — le prompt de l'orchestrateur pour la remédiation de la P2 (recopié tel quel)

> Tu es l'agent de spécification de la Story 32-11 de Kesh. Tu travailles **uniquement** dans le worktree `/home/gcorbaz/devel/kesh-32-11`, sur la branche `story/32-11`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl, ni `claude`, ni `socat` ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/` ;
> - Hermes hors périmètre.
>
> ## Tâche
> Remédier la validation P2 de la fiche `_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md`. Les rapports sont `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-R.md` (0/0/6/5) et `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p2-F.md` (0/0/3/8). Lis-les en entier.
>
> La plupart des findings naissent du parcours joué par `claude -p`. C'est un **recyclage** : déclare-le au Change Log comme signal D5. Nous sommes à la passe 2 sur 8.
>
> ### Décision de l'orchestrateur : retirer le pilotage de `claude` par l'agent de développement
> C'est ce montage qui produit la plupart des défauts : sessions non bornées, sessions mal comptées, écritures dans `~/.claude*` qui dérogent à la règle « un agent n'écrit jamais hors de son worktree ». Il passe donc en **variante soumise à Guy**, non retenue par défaut.
>
> Le montage par défaut devient le suivant :
> - **L'agent de développement joue le parcours P0 à P11 en client HTTP**, sans `claude`. Il envoie les requêtes JSON-RPC décrites par la spécification et par la fiche (initialize, tools/list, prompts/get, tools/call, kesh_lire_envoi…), contre une instance de démonstration, avec un outil déjà permis dans le harnais de test. De préférence, c'est un **test d'intégration Rust** qui pilote le routeur ou un serveur réel ; sinon c'est `curl`, à déclarer dans le gate du développement. Le parcours est **borné par construction** : la liste des requêtes est écrite, et le nombre d'envois est exact.
> - **La capture des requêtes réellement envoyées par Claude Code**, qui sert à construire la fixture du contrat, n'est faite **que par Guy, à la recette**. La fiche décrit le geste, en une étape, avec un fichier de capture déposé dans `kesh-gate-logs/`. Le test de contrat de la CI ne dépend pas de cette capture ; il repose sur les clauses citées. Si la capture de Guy contredit une clause, on l'ouvre en issue.
> - **Le rejeu depuis le PDF** des gestes interactifs avec Claude Code (brancher, lister, appeler, valider) est **la recette de Guy**, d'une durée d'environ 20 à 30 minutes, déjà prévue.
> - **Le point 1 pour Guy est réécrit en question.** « Acceptez-vous qu'un agent lance `claude -p` sur votre compte, ce qui écrit dans `~/.claude/projects/` et `~/.claude.json` ? Ce serait une dérogation nommée à `docs/agents-orchestration.md` § 2.1. Si oui, la variante est écrite en annexe. » Recommandation : **non**, le montage par défaut suffit.
>
> ### Les autres MEDIUM
> - **R1 (garanties générales du manuel) :** le manuel ne dit pas « lecture seule » sans réserve. `proposer_prompt` écrit, et sa trace est la validation par l'Administrateur. Prends en charge `README.md:233` et `website/roadmap.html:223` et `:375`, ou écarte-les avec un motif écrit.
> - **R4 (motifs du PDF déjà présents) :** reprends la règle de la 32-7, AC-21. Chaque motif rend 0 sur le PDF de base et 1 sur le PDF régénéré. Le développement cite les deux sorties à l'étape 0, et choisit de nouveaux motifs si l'un d'eux est déjà présent.
> - **R5 = F2, F5 (étiquettes) :** K15, K16, K21, le type `text` de K8 et K14, K3 et K17 sont rangés selon la spike et, à défaut, sous `DECISION_KESH`. Tout cas incertain est marqué « à constater à l'étape 0 sur la page ». Ce qui ne peut être tranché est rangé sous `DECISION_KESH`. **Ne marque jamais `CLAUSE` ce qui est douteux.**
>
> ### LOW
> Intègre-les tous ou retire-les, notamment :
> - F6 : K14 et l'amorçage des standards dans le routeur de test ;
> - F7 ;
> - F9 : les ports et les dossiers ;
> - F10 ;
> - F11 / R9 : les mutations manquantes, en en ajoutant au plus quatre, sinon un angle mort écrit ;
> - R7 ;
> - R8 ;
> - R10 ;
> - R11.
>
> Sont à **retirer**, parce qu'ils tombent avec la nouvelle conception : F1/R2 (session S1), F3/R6 (déplacé au point 1), F4/R3 (comptage des sessions), F8 (rejeu).
>
> **L'écart de la lentille R** (une redirection `2>&1`) est à mentionner au Change Log.
>
> **Après le patch :**
> - grep des valeurs (`claude -p`, `S1`, `S4`, `--resume`, `~/.claude`, `CLAUSE`) ;
> - recompte à la source ;
> - aucune entrée passée réécrite ;
> - entrée P2 ajoutée, avec le trend 0/1/5/16 → 0/0/9/13.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P3, `_bmad-output/implementation-artifacts/3211-validate-prompt-p3.md` : lentilles R et F, **Sonnet**, contexte frais, rapports dans `/home/gcorbaz/devel/kesh-gate-logs/3211-validate-p3-{R,F}.md`, mêmes interdits (dont toute redirection, `claude` et `socat`), un axe sur les régressions de cette remédiation, et ce prompt-ci en annexe.
> - Un rapport final bref.
