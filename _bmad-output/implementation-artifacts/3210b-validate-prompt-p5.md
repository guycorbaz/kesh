# Prompt — validation P5, Story 32-10b (l'écran des prompts du serveur MCP)

*Versionné le 2026-10-27. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), cinquième sur huit. **Une seule
lentille, *regression hunter***, en contexte frais, **Sonnet**. Lecture seule stricte. Elle suit la remédiation de la P4
(trend **0/0/2/18 → 0/0/1/7** ; entrée « validation P4 » du Change Log de la fiche) : un MEDIUM antérieur (la liste
d'arrêt) et sept LOW, tous des régressions de la P3. La consigne reste **« retirer ou simplifier plutôt qu'enrichir »** :
un finding qui propose d'ajouter un mécanisme doit dire pourquoi le retrait de la promesse ne suffit pas. La passe
ciblée vise l'endroit exact que personne n'a relu : **le seul commit de cette remédiation**.*

Dépôt `/home/gcorbaz/devel/kesh-32-10b`, branche `story/32-10b`. Le commit à relire : **`c89a2b7c`** (« Story 32-10b :
validation P4 remédiée… »), lu par `git show c89a2b7c` **en entier** — fiche
`_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`, registre `epic-32-choix-autonomes-3210b.md` et
`sprint-status.yaml`. Le commit précédent (`0dc8c9d2`) est la remédiation P3 ; ne relis la fiche au-delà du diff que pour
juger ce que le diff touche (cohérence avec le reste de la fiche, résidus). Rapport de la P4, qui a motivé le patch :
`/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p4.md`. **La fiche 32-10a** se lit dans le worktree
`/home/gcorbaz/devel/kesh-32-10a` (branche `story/32-10a`), `_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`
— en particulier la note d'étape 0 « D1, champs du détail » (l. 1534-1541 à `8e7ea39d`) et l'AC-iii6 (l. 899-907). Si son
HEAD a avancé, lis `git show 8e7ea39d:<fichier>` et signale l'écart. Le modèle voisin, fusionné :
`frontend/src/lib/features/mcp-sends/` (`McpSendDetail.svelte`, en particulier l. 56, 190-194, 345-347) et
`crates/kesh-api/src/services/envois_mcp.rs`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : au plus trois modules (frontend, `kesh-i18n`, manuel
d'administration) ; la 32-10b n'ajoute **aucune route** ni code Rust ; **l'écran n'a aucune logique Unicode** ; il
affiche le texte rendu visible par le serveur et le verdict `servable` du serveur ; l'approbation se fait sur la version
**vue**, avec contrôle de version ; l'Administrateur seul voit et décide ; captures en thème clair seulement ; Hermes hors
périmètre ; aucune donnée réelle. Ce que la fiche **prescrit** n'existe pas encore : son absence n'est pas un défaut.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-7` ni dans
`/home/gcorbaz/devel/kesh-32-10a` ; n'exécute **ni** `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `curl`, **ni aucun
interpréteur** (`python`, `perl`, `awk`, `sh -c`…), **ni redirection** (`>`, `>>`, `tee`) — **pas même vers un fichier
temporaire ou un scratchpad** (la lentille R de la P3 l'a fait ; c'était un écart, sans effet sur le dépôt) — hors de l'écriture de ton rapport
par l'outil d'écriture, **ni** `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/regen-test-schema.sh` ou
tout autre script ; **aucune** commande `gh` qui écrit (`gh issue view` est permis), **ni** `git` qui écrit (`commit`,
`push`, `checkout`, `switch`, `reset`, `stash`, `merge`, `rebase`, `tag`, `worktree`) ; n'ouvre jamais `~/kesh-test/`.
Commandes permises : `grep`, `sed -n`, `ls`, `wc`, `cut`, `sort`, `head`, `tail`, `git log`, `git show`, `git diff`,
`gh issue view`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

## Les axes de la lentille

1. ⛔ **Chaque affirmation de présence ou d'absence, prouvée par une sortie `grep -nF` copiée** (ou `sed -n` de la
   ligne citée). Chaque ligne **neuve** citée par le patch — fiche 32-10a l. 1534-1541 ; `McpSendDetail.svelte:56` — s'ouvre
   et se cite.
2. ⛔ **Ce que le patch a cassé ou laissé** — classe chaque finding « **régression P4** » ou « **antérieur** » :
   - **résidus** : greppe les **valeurs** (`grep -nF`), non les formulations, sur la fiche, le registre et
     `sprint-status.yaml` : `aujourdhui`, `aujourd'hui`, `Actualiser`, `drafted`, `non-servi-`, `servi-`, `ready-for-dev`
     (pour cette story), `tel que Kesh le servirait`, `quel que soit le cas`, `0/0/2/18` ;
   - **H1, la liste d'arrêt** : `version`, `numero`, `origine`, `etat`, `nature` sont-ils bien promis par la note « D1,
     champs du détail » de la 32-10a, aux lignes citées ? Les types des *Dev Notes* et l'AC-7 les consomment-ils tous, et
     seulement eux ?
   - **H2, les zones `servi-` / `non-servi-`** : la règle de l'AC-5, l'AC-6, l'E2E (c) et le test
     `bloc_servi_seulement_si_servable` disent-ils tous la même chose ? « Le test juge par le préfixe » : un
     `data-testid` préfixé `servi-` est-il bien absent d'une version en vigueur non servable ? Le préfixe `servi-`
     entre-t-il en collision avec `a-valider-` ou `historique-` (sous-chaîne), ou avec `servi.texte` ?
   - **H3, H4** : la phrase sur *Recharger* de l'AC-12 et celle de l'AC-18 (manuel) disent-elles la même chose que
     l'AC-9 ? Reste-t-il un *Actualiser* hors de la liste (AC-3) ?
   - **H5, l'avis de l'AC-4** : la nouvelle borne (au-dessus du seul bloc *Texte servi par Kesh*) contredit-elle l'AC-6,
     l'AC-7 (version à valider), le test `avis_permanent_exact`, l'AC-18 ou l'inventaire ? Le test `avis_permanent_exact`
     dit-il où l'avis se trouve ? Reste-t-il « quel que soit le cas » ?
   - **H6, H7, H8** : l'annotation de C-32-10b-8 concorde-t-elle avec l'AC-9 et l'AC-12 ? Le statut `ready-for-dev`
     et son commentaire sont-ils identiques dans la fiche et `sprint-status.yaml` ? L'annotation de l'entrée P3 (R5 et
     F4 sont deux constats) est-elle fidèle à `3210b-validate-p3-F.md` et `-R.md` ?
3. ⛔ **Recompter les décomptes du patch depuis la source** : 20 AC (A 4, B 2, C 8, D 6), 9 tâches, 18 mutations
   (M1 à M18), E1 à E18, C-32-10b-1 à 19, 6 points pour Guy, 5 demandes ; le trend et la ventilation de l'entrée
   « validation P4 » du Change Log contre le rapport de la P4 (0/0/1/7 ; H1 antérieur, H2 à H8 régressions P3)
   (`CLAUDE.md` § *Recompter*). Ne réécris rien : signale.
4. ⛔ **Le manuel** : l'AC-18, telle que le patch l'a modifiée, prescrit-elle ce que le code fera ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p5.md`. Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
2. **Findings** numérotés (`J1`, `J2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site de la fiche
   (section, AC), la **preuve** (commande exécutée et sortie copiée — `grep -nF` pour toute présence ou absence —, ou
   ligne lue), la correction proposée (de préférence un **retrait**), et la classe « régression P4 » ou « antérieur ».
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste, et la part « régression P4 ».

## Annexe — la consigne de remédiation de la P4, recopiée

*Ce que l'orchestrateur a demandé à l'agent de remédiation de la P4, à titre de contexte : juge le patch contre cette
consigne aussi (une décision appliquée de travers est un finding).*

> Tu es l'agent de spécification de la Story 32-10b de Kesh, l'écran des prompts. Tu travailles **uniquement** dans le
> worktree `/home/gcorbaz/devel/kesh-32-10b`, sur la branche `story/32-10b`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push, ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture est permise ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/`.
>
> ## Tâche
> Remédier la validation P4 de la fiche `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Le rapport est dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p4.md` (0/0/1/7) ; lis-le en entier. La fiche 32-10a se lit dans `/home/gcorbaz/devel/kesh-32-10a`. Elle porte maintenant, à son étape 0, la note « D1, champs du détail » : la route de détail rend la `version` du prompt, et pour chaque version `numero`, `origine`, `etat` et `nature`.
>
> Nous sommes à la passe 4 sur 8. **Change le moins de texte possible.**
>
> **Décisions de l'orchestrateur :**
> - **H1 (MEDIUM)** : ajoute à la liste d'arrêt de l'étape 0 la `version` du prompt et les champs `numero`, `origine`, `etat` et `nature` de chaque version, en citant la note de la 32-10a. Ajoute un mot, pas un mécanisme.
> - **H2** : la zone du bloc servi prend un préfixe neutre, `servi-`, et le texte non servi en prend un distinct, `non-servi-`. Le test juge par le préfixe, et non par le titre.
> - **H3** : écris à l'AC-12 et au manuel qu'après un 409, *Recharger* remet l'éditeur sur la version en vigueur, et que la saisie est perdue. Ne change pas le comportement.
> - **H4** : retire *Actualiser* de la liste des relectures de l'éditeur.
> - **H5** : l'avis de l'AC-4 ne s'affiche qu'au-dessus du bloc servi. Il ne s'affiche ni sur une version non servable, ni sur une version refusée, ni dans l'éditeur. Écris-le.
> - **H6** : annote C-32-10b-8.
> - **H7** : remets le statut à `ready-for-dev` dans la fiche, avec la mention « validation en cours » en commentaire. `drafted` n'est pas défini.
> - **H8** : corrige le Change Log par une annotation datée, sans réécrire l'entrée P3.
>
> **Après le patch :**
> - greppe `aujourdhui`, `Actualiser` et `drafted` ;
> - recompte à la source ;
> - écris l'entrée P4, trend 0/0/2/18 → 0/0/1/7.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P5, `_bmad-output/implementation-artifacts/3210b-validate-prompt-p5.md` :
>   - passe **ciblée**, une lentille *regression hunter*, **Sonnet**, contexte frais, sur le seul commit de cette remédiation ;
>   - rapport dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p5.md` ;
>   - mêmes exigences et mêmes interdits que la P4, y compris aucune redirection ;
>   - ce prompt-ci recopié en annexe.
> - Un rapport final bref.
