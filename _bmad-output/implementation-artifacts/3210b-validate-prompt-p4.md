# Prompt — validation P4, Story 32-10b (l'écran des prompts du serveur MCP)

*Versionné le 2026-10-27. Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*), quatrième sur huit. **Une seule
lentille, *regression hunter***, en contexte frais, **Opus**. Lecture seule stricte. Elle suit la remédiation de la P3
(trend **0/0/10/16 → 0/0/2/18** ; entrée « validation P3 » du Change Log de la fiche), où l'orchestrateur a déclaré un
**recyclage** pour la seconde fois (les deux MEDIUM de la P3 naissaient des remédiations P1 et P2) et maintenu la
consigne **« retirer ou simplifier plutôt qu'enrichir »**. La passe ciblée vise l'endroit exact que personne n'a relu :
**le seul commit de cette remédiation**.*

Dépôt `/home/gcorbaz/devel/kesh-32-10b`, branche `story/32-10b`. Le commit à relire : `git log -1 --format=%H` (message
« Story 32-10b : validation P3 remédiée… »), lu par `git show <ce commit>` **en entier** — fiche
`_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`, registre `epic-32-choix-autonomes-3210b.md`,
`sprint-status.yaml` et ce prompt. Le commit précédent (`611d5343`) est la remédiation P2 ; ne relis la fiche au-delà du
diff que pour juger ce que le diff touche (cohérence avec le reste de la fiche, résidus). Rapports de la P3, qui ont
motivé le patch : `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-R.md` et `3210b-validate-p3-F.md`. **La fiche
32-10a** se lit dans le worktree `/home/gcorbaz/devel/kesh-32-10a` à **`8e7ea39d`** (branche `story/32-10a`),
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md` — en particulier l'AC-iii6 (l. 899-907, « textes
compris » l. 902), l'AC-ii2 (l. 635-640) et la note d'étape 0 (l. 1534-1541). Si son HEAD a avancé, lis `git show
8e7ea39d:<fichier>` et signale l'écart. Le modèle voisin, fusionné : `frontend/src/lib/features/mcp-sends/`
(`McpSendDetail.svelte`, en particulier l. 56, 190-194, 345-347 ; `McpSendDetail.test.ts`) et
`crates/kesh-api/src/services/envois_mcp.rs` (`rendre_visible`, l. 75).

**Contraintes de la story, à tenir pour vraies et à contrôler** : au plus trois modules (frontend, `kesh-i18n`, manuel
d'administration) ; la 32-10b n'ajoute **aucune route** ni code Rust ; **l'écran n'a aucune logique Unicode** ; il
affiche le texte rendu visible par le serveur et le verdict `servable` du serveur ; l'approbation se fait sur la version
**vue**, avec contrôle de version ; l'Administrateur seul voit et décide ; captures en thème clair seulement ; Hermes hors
périmètre ; aucune donnée réelle. Ce que la fiche **prescrit** n'existe pas encore : son absence n'est pas un défaut.
**Retirer plutôt qu'enrichir** : un finding qui propose d'ajouter un mécanisme doit dire pourquoi le retrait de la
promesse ne suffit pas.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-7` ni dans
`/home/gcorbaz/devel/kesh-32-10a` ; n'exécute **ni** `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `curl`, **ni aucun
interpréteur** (`python`, `perl`, `awk`, `sh -c`…), **ni redirection** (`>`, `>>`, `tee`) — **pas même vers un fichier
temporaire ou un scratchpad** (la lentille R de la P3 l'a fait ; c'était un écart) — hors de l'écriture de ton rapport
par l'outil d'écriture, **ni** `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/regen-test-schema.sh` ou
tout autre script ; **aucune** commande `gh` qui écrit (`gh issue view` est permis), **ni** `git` qui écrit (`commit`,
`push`, `checkout`, `switch`, `reset`, `stash`, `merge`, `rebase`, `tag`, `worktree`) ; n'ouvre jamais `~/kesh-test/`.
Commandes permises : `grep`, `sed -n`, `ls`, `wc`, `cut`, `sort`, `head`, `tail`, `git log`, `git show`, `git diff`,
`gh issue view`, `pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

## Les axes de la lentille

1. ⛔ **Chaque affirmation de présence ou d'absence, prouvée par une sortie `grep -nF` copiée** (ou `sed -n` de la
   ligne citée). Chaque ligne **neuve** citée par le patch — fiche 32-10a l. 902 et 635-640 à `8e7ea39d` ;
   `McpSendDetail.svelte:56`, `:190-194`, `:345-347` ; le précédent `aria-busy` d'`admin-backup/` — s'ouvre et se cite.
2. ⛔ **Ce que le patch a cassé ou laissé** — classe chaque finding « **régression P3** » ou « **antérieur** » :
   - **résidus** de ce qui a été retiré ou renommé : greppe les **valeurs** (`grep -nF`), non les formulations, sur la
     fiche, le registre et `sprint-status.yaml` : `équivalente`, `aujourd'hui`, `reçoit aujourd`, `-{numero}` (tout
     `data-testid` de version sans préfixe de zone), `servi.texte` (tout usage hors tests, E2E et liste « E2E
     seulement »), `contrôle et mise en page`, `avis_distingue`, `bloc_du_jour`, `toutes lettres`, `dernière phrase`,
     `17 mutations`, `M1 à M17`, `ready-for-dev` (pour cette story) ;
   - **la règle des `data-testid`** (AC-5) : écrite une seule fois, appliquée à l'AC-6 et aux E2E (a) et (c) ; la
     zone de chaque montage est-elle déterminée (le texte non servi sous sa phrase, la version en vigueur aussi
     présente à l'historique, une version à valider présente à l'historique) ? Un test ou un E2E nommé vise-t-il un
     `data-testid` sans zone, ou une zone qui ne monte pas la version visée ?
   - **la liste d'arrêt** (§ *À recontrôler à l'étape 0*) : nomme-t-elle tout ce que l'écran consomme — et rien
     qu'il ne consomme pas hors de la mention « E2E seulement » — d'après les AC, les types des *Dev Notes* et la note
     de la 32-10a ? `description` et `texte` stockés sont-ils bien promis par l'AC-iii6 à la ligne citée ?
   - **l'avis de l'AC-4** : la phrase est-elle vraie dans **tous** les cas (version en vigueur, à valider, non servable,
     description, standard) ? Le paramètre Fluent est-il décrit comme à la 32-8c ? Le test `avis_permanent_exact`
     établit-il le sens (attendu écrit en dur) ? Reste-t-il une phrase de la fiche ou du manuel prescrit qui s'appuie
     sur l'avis retiré ?
   - **E17 en angle mort** : dit tel à l'inventaire, au § *Ce que les gardes ne voient pas* et à l'AC-18, sans
     contradiction avec l'AC-6 ni avec le recontrôle du `\r` de l'étape 0 ?
   - **M18 et `gestes_desactives_pendant_l_envoi`** : deux `fireEvent.click`, un seul `POST` — M17 et M18 sont-elles
     chacune tuées par ce test tel qu'il est écrit, sans qu'une assertion masque l'autre ? Le décompte 18 est-il
     propagé partout (T8, Change Log, `sprint-status.yaml`) ?
   - **l'éditeur réinitialisé après relecture** (AC-9, `editeur_reinitialise_apres_relecture`) : cohérent avec l'AC-12
     (saisies conservées sur un 400, perdues à *Recharger*) et avec l'avis P26 ? La fixture (v3 d'administrateur, v4
     d'agent dernière) est-elle approuvable selon l'AC-7 ?
   - **les dialogues pendant l'envoi** (trois réglages de la 32-8c) : citation juste ?
   - **le registre** : chaque annotation P3 dit-elle la même chose que la fiche ? Un titre ou un « Retenu » encore
     contraire à la fiche reste-t-il sans annotation ?
   - **le statut** `drafted` : cohérent entre la fiche et `sprint-status.yaml` ?
3. ⛔ **Recompter les décomptes du patch depuis la source** : 20 AC (A 4, B 2, C 8, D 6), 9 tâches, **18** mutations
   (M1 à M18), E1 à E18, C-32-10b-1 à 19, 6 points pour Guy, 5 demandes ; le trend et la ventilation des classes de
   l'entrée « validation P3 » du Change Log contre les deux rapports de la P3 (`CLAUDE.md` § *Recompter*). Ne réécris
   rien : signale.
4. ⛔ **Le manuel** : l'AC-18, telle que le patch l'a modifiée (« Texte servi par Kesh », E17 en angle mort), prescrit-elle
   ce que le code fera ? Contrôle aussi qu'aucune phrase prescrite ne cite une section ou une ligne qui n'existe pas :
   `grep -n 'sec:mcp-envois\|routes à verbe mutant\|section{Sécurité}' docs/manual/fr/admin-manual.tex` ;
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE '.{60}(envois du serveur MCP|prompt).{80}'`.

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p4.md`. Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
2. **Findings** numérotés (`H1`, `H2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site de la fiche
   (section, AC), la **preuve** (commande exécutée et sortie copiée — `grep -nF` pour toute présence ou absence —, ou
   ligne lue), la correction proposée (de préférence un **retrait**), et la classe « régression P3 » ou « antérieur ».
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste, et la part « régression P3 ».

## Annexe — la consigne de remédiation de la P3, recopiée

*Ce que l'orchestrateur a demandé à l'agent de remédiation de la P3, à titre de contexte : juge le patch contre cette
consigne aussi (une décision appliquée de travers est un finding).*

> Tu es l'agent de spécification de la Story 32-10b de Kesh, l'écran des prompts. Tu travailles **uniquement** dans le
> worktree `/home/gcorbaz/devel/kesh-32-10b`, sur la branche `story/32-10b`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - dépôt principal en lecture seule ;
> - ni push ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture est permise ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/` ;
> - Hermes est hors périmètre.
>
> ## Tâche
> Remédier la validation P3 de la fiche `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Les rapports
> sont dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-R.md` (0/0/1/9) et
> `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-F.md` (0/0/1/9) ; lis-les en entier. La fiche 32-10a se lit
> dans `/home/gcorbaz/devel/kesh-32-10a`, HEAD `8e7ea39d`.
>
> Les deux MEDIUM et la plupart des LOW naissent des remédiations P1 et P2 : c'est un **recyclage**, à déclarer au
> Change Log (signal D5). Nous sommes à la passe 3 sur 8. La consigne reste : **retirer ou simplifier plutôt
> qu'enrichir**.
>
> **Décisions sur les MEDIUM :**
> - **F1 (testid en double).** Garde une seule règle simple : un `data-testid` préfixé par la zone (`aujourdhui-`,
>   `a-valider-`, `historique-`), et l'E2E vise la zone qu'il veut. Écris la règle une seule fois et applique-la à
>   l'AC-5, à l'AC-6 et aux E2E (a) et (c).
> - **R1 (liste d'arrêt inversée).** La liste d'arrêt nomme ce que l'écran **consomme** :
>   - le texte et la description **stockés**, pour l'éditeur et *Reprendre cette version*. Vérifie que l'AC-iii6 de
>     la 32-10a les rend (« textes compris ») et cite la ligne ;
>   - les champs **rendus visibles**, pour l'affichage.
>
>   Les champs servis **exacts** sont marqués « E2E seulement ». L'éditeur ne se pré-remplit jamais avec le texte
>   servi.
>
> **Décisions sur les LOW :**
> - **R3, R4, F3 et R8 (l'avis de l'AC-4).** Réduis-le à une phrase vraie quel que soit le cas : « Le texte est montré
>   tel que Kesh le servirait ; certains caractères invisibles y sont écrits sous la forme `\u{…}`. » Le test compare
>   à un attendu **écrit en dur**, avec le paramètre ou l'échappement Fluent dit. Retire toute énumération des
>   caractères concernés. E17 ne s'appuie plus sur l'avis : écris-la en angle mort.
> - **R5 et F4 (le retour anticipé).** Il n'est pas équivalent, puisque Svelte appelle le gestionnaire délégué même
>   sur un bouton désactivé. Le test envoie deux événements par `fireEvent` et attend **un seul** POST. La mutation
>   « retour anticipé retiré » se joue et doit être tuée. Retire le mot « équivalente ».
> - **F5 (éditeur et relecture).** Après toute relecture, l'éditeur est réinitialisé sur la version en vigueur, comme
>   `poser()` dans la 32-8c. Ajoute un test.
> - **F2.** Retire la parenthèse de l'AC-7.
> - **F6.** « Ce que l'agent reçoit aujourd'hui » se renomme en « Texte servi par Kesh », sans supposer que le serveur
>   est activé.
> - **F7 = R2.** Annote le registre, sans le réécrire.
> - **F8.** L'imitation de la marque devient une clause de la puce « homoglyphes ».
> - **F9 = R9.** Cite `8e7ea39d` et corrige la parenthèse de D5.
> - **F10.** Reprends les trois réglages du dialogue de la 32-8c.
> - **R6.** Ajoute le contenu vide.
> - **R7.** Écris-le en angle mort.
> - **R10.** Le statut redevient `drafted` en attendant la fin de la boucle. L'orchestrateur le passera à
>   `ready-for-dev`.
> - **L'écart de la lentille R** (redirection dans son scratchpad) est sans effet sur la fiche : mentionne-le au
>   Change Log.
>
> **Après le patch :**
> - greppe les valeurs : `équivalente`, `aujourd'hui`, `-{numero}`, `servi.texte`, `contrôle et mise en page` ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris l'entrée P3, avec le trend 0/0/10/16 → 0/0/2/18.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P4 : `_bmad-output/implementation-artifacts/3210b-validate-prompt-p4.md`.
>   - Passe **ciblée** : une lentille *regression hunter*, **Opus**, contexte frais, sur le seul commit de cette
>     remédiation.
>   - Le rapport va dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p4.md`.
>   - Mêmes exigences et mêmes interdits que les passes précédentes.
>   - Ce prompt-ci est recopié en annexe.
> - Un rapport final bref.
