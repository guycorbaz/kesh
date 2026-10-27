# Prompt — validation P3, Story 32-10b (l'écran des prompts du serveur MCP)

*Versionné le 2026-10-27. Passe **complète**, troisième sur huit. **Deux lentilles, R et F**, chacune en contexte frais,
**Sonnet**. Lecture seule stricte. Elle suit la remédiation de la P2 (trend **0/3/8/15 → 0/0/10/16** ; entrée
« validation P2 » du Change Log de la fiche), où l'orchestrateur a déclaré un **recyclage** (la plupart des findings de
la P2 naissaient de la remédiation P1) et donné la consigne **« retirer plutôt qu'enrichir »** : l'encadré séparé de la
marque, le repli de la version en vigueur, la prétention de l'E2E sur le champ rendu visible et une promesse de l'avis
ont été retirés.*

Dépôt `/home/gcorbaz/devel/kesh-32-10b`, branche `story/32-10b` (fiche 32-10a + `origin/main` `ed981c7d` fusionné,
merge `42680725` : la 32-8 complète, 32-8c comprise ; la **32-10a est validée, non développée** ; la 32-7 n'est pas
fusionnée). Fiche : `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210b.md` (C-32-10b-1 à 19). Le commit de la remédiation
P2 se lit par `git log -1 --format=%H` puis `git show <ce commit>` (et `git diff 486e64ab HEAD --
_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` ; `486e64ab` est la remédiation P1). **La fiche 32-10a à
jour est sur une autre branche** : lis-la dans le worktree `/home/gcorbaz/devel/kesh-32-10a` à **`20a02a86`** (branche
`story/32-10a`), `_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`, § *À recontrôler à l'étape 0*,
l. 1534-1541 au relevé — en particulier la note **« D1 étendu, complément »** (l. 1540) : texte **et** description
servis, chacun exact et rendu visible, **la marque comprise dans le texte servi, sans champ `marque` séparé** ; de même
pour le standard du code ; `creeLe`, `decideLe`, `numeroEnVigueur` ; la composition dans `services/` ; le refus de D2
en **400**. Si le HEAD de ce worktree a avancé, lis `git show 20a02a86:<fichier>` et signale l'écart. La copie présente
dans le worktree `kesh-32-10b` est **antérieure** à ces amendements. Rapports des passes précédentes :
`/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-{R,F}.md` et `3210b-validate-p2-{R,F}.md`.

Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du Project Lead*, § *32-10*,
*Réponses de Guy*) ; la fiche 32-10a (AC-i5, AC-i6, AC-i7, AC-i8, AC-ii2, AC-ii5 à AC-ii7, AC-iii6 à AC-iii8, AC-iii11,
inventaire P1 à P27, § *La frontière avec la 32-10b*, § *Le manuel*, *Dev Notes*) et son registre ; **le modèle voisin,
fusionné** : la fiche `32-8-validation-avant-envoi.md` partie C, le registre `epic-32-choix-autonomes-328.md`, le code
`frontend/src/lib/features/mcp-sends/` (dont `McpSendDetail.svelte`, `mcp-sends-i18n.test.ts`),
`frontend/src/routes/(app)/admin/mcp/sends/`, `frontend/tests/e2e/mcp-sends.spec.ts`, et
**`crates/kesh-api/src/services/envois_mcp.rs`** (`rendre_visible`, l. 75, et ses tests) ; **la fiche 32-7, en
lecture seule** : `/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`.
Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Recette visuelle*, *Le prompt
d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Propagation post-patch*, *Règle de
splitting préventif*), `docs/testing.md`, `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : au plus trois modules (frontend, `kesh-i18n`, manuel
d'administration) ; la 32-10b n'ajoute **aucune route** ni code Rust ; **l'écran n'a aucune logique Unicode** : il
affiche le texte rendu visible par le serveur et le verdict `servable` du serveur ; **l'écran ne consomme que les champs
de la note complétée de la 32-10a** ; la marque se lit dans le texte rendu visible, sans traitement de l'écran ; un
prompt est une instruction donnée à un agent qui lit des livres réels : l'écran doit montrer **exactement** le texte
servi, et l'approbation se faire sur la version **vue**, avec contrôle de version ; le retour au standard sans
écrasement, sur un texte vu ; l'Administrateur seul voit et décide ; captures en thème clair seulement ; Hermes hors
périmètre ; aucune donnée réelle. Ce que la fiche **prescrit** n'existe pas encore : son absence n'est pas un défaut.
Ce que la fiche cite **de la fiche 32-10a** est prévu, non codé : juge sa cohérence avec cette fiche (à `20a02a86`),
non sa présence dans le code. **Retirer plutôt qu'enrichir** : un finding qui propose d'ajouter un mécanisme doit dire
pourquoi le retrait de la promesse ne suffit pas.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-7` ni dans
`/home/gcorbaz/devel/kesh-32-10a` ; n'exécute **ni** `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `curl`, **ni aucun
interpréteur** (`python`, `perl`, `awk`, `sh -c`…), **ni redirection** (`>`, `>>`, `tee`) hors de l'écriture de ton
rapport par l'outil d'écriture, **ni** `scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/regen-test-schema.sh` ou tout autre script ; **aucune** commande `gh` qui écrit (`gh issue view` est permis),
**ni** `git` qui écrit (`commit`, `push`, `checkout`, `switch`, `reset`, `stash`, `merge`, `rebase`, `tag`,
`worktree`) ; n'ouvre jamais `~/kesh-test/`. Commandes permises : `grep`, `sed -n`, `ls`, `wc`, `cut`, `sort`, `head`,
`tail`, `git log`, `git show`, `git diff`, `gh issue view`, `pdftotext` vers la sortie standard. Le seul fichier que
tu écris est ton rapport.

## Les quatre axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque affirmation de présence ou d'absence, prouvée par une sortie `grep -nF` copiée** (ou `sed -n` de la
   ligne citée). Dresse d'abord la liste des motifs et des mesures : `grep -nE "grep -|→ \*\*|au relevé|mesure"
   _bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Pour chaque mesure (faits 1 à 13, AC-13, AC-15,
   AC-19) et chaque ligne neuve citée par la P2 (`mcp-sends-i18n.test.ts:52`, `:136`, `:231` ; `McpSendDetail.svelte:47`,
   `:92`, `:309-368` ; #773 point 15 ; fiche 32-10a l. 1534-1541 à `20a02a86`), exécute-la et copie la sortie ; une
   mesure que tu ne reproduis pas, un témoin « existant » absent de la ligne citée, une ligne citée qui ne dit pas ce
   que la fiche lui prête, sont des findings.
2. ⛔ **Le manuel, au `.tex` ET au PDF aplati.** `grep -n 'sec:mcp-envois\|routes à verbe mutant\|section{Sécurité}\|
   section{Monitoring' docs/manual/fr/admin-manual.tex` ; `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' |
   tr -s ' ' | grep -oiE '.{60}(envois du serveur MCP|routes à verbe mutant|prompt).{80}'`. L'AC-18 prescrit-elle tout
   ce que la fiche 32-10a laisse à la 32-10b, et chaque phrase prescrite décrit-elle ce que **le serveur** fera — en
   particulier la phrase sur les caractères invisibles, rendue conditionnelle en P2 (cas du `\r` restauré), contre
   `rendre_visible` et `valider_contenu` ?
3. ⛔ **Recompter les décomptes de la fiche depuis sa source** : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **17**
   mutations (M1 à M17, plus une équivalente déclarée hors table), **E1 à E18**, C-32-10b-1 à **19** (chacun cité par
   la fiche existe au registre et y dit la même chose, annotations P2 comprises), **6** points pour Guy, **5**
   demandes ; chaque total contre sa ventilation, **et** contre l'entrée « validation P2 » du Change Log (`CLAUDE.md`
   § *Recompter*), sans réécrire les entrées passées.
4. ⛔ **Les régressions de la remédiation P2** (`CLAUDE.md` § *La passe ciblée* : « la sévérité se déplace vers ce
   qu'on vient d'écrire »). Lis le diff de la P2 **en entier** et cherche ce qu'il a cassé ou laissé :
   - **résidus** de ce qui a été retiré : l'encadré de la marque et le champ `marque` (`servi.marque`,
     `mcp-prompt-mark`, `marque_dans_le_bloc`, `marque_manquante`, `mark-missing`, `encadr`), le repli de la version en
     vigueur (`en_vigueur_selon_la_regle`, « fonction pure »), l'avis « chaque caractère invisible », le 409 de D2,
     `un_seul_post_pendant_l_envoi`, la question 6, l'E2E qui vérifierait le champ rendu visible — greppe les
     **valeurs** (`grep -nF`), non les formulations, sur la fiche, le registre et `sprint-status.yaml` ;
   - **cohérence des champs** entre les AC, les types des *Dev Notes* (`ServiPrompt` sans `marque`, `standardDuCode:
     ServiPrompt | null`), la liste d'arrêt de l'étape 0, l'E2E et la note complétée de la 32-10a (`20a02a86`) — un
     champ consommé que la note ne promet pas, ou promis et nommé différemment, est un finding ;
   - **propriétés neuves tenues par ce qu'elles disent** : `texte_affiche_egal_au_champ_visible` sur une version
     d'agent tue-t-il M2, M3 et la M13 réécrite ? `gestes_desactives_pendant_l_envoi` tue-t-il la M17 réécrite, et
     couvre-t-il **tous** les gestes de la page (dialogues compris) ? La mutation déclarée équivalente l'est-elle
     vraiment ? M10 réécrite est-elle tuée par le cas nommé ? L'avis de l'AC-4 est-il **vrai** mot pour mot contre
     `rendre_visible` et `is_hidden_in_text`, et le test qui vérifie son texte suffit-il ?
   - **prémisses** : la condition `standardDiffere && standardDuCode !== null` contre l'AC-i5, l'AC-i10 et l'AC-iii6 de
     la 32-10a ; le 400 de D2 contre l'AC-iii8 ; E18 contre #773 ; l'affirmation « un bouton désactivé ne reçoit pas le
     clic » (raisonnée, non constatée : dis-le si tu ne peux la vérifier) ;
   - **angles morts neufs** (E18, « la liste ne juge pas », l'E2E à caractère caché) : sont-ils justes, et la
     conception en ouvre-t-elle un qu'elle ne nomme pas ?
   Classe chaque finding de cet axe « **régression P2** », « **régression P1** » ou « **antérieur** » dans ton rapport.

## Lentille R — la spécification contre le code et contre la fiche 32-10a

1. **Les faits 1 à 13 et § *Ce qui est lu et cité*** : chaque numéro de ligne, ouvert.
2. **Le contrat avec la 32-10a** : chaque ligne du tableau « Ce que dit la fiche 32-10a » contre l'AC qu'il cite ;
   les règles d'approbation (AC-i7), de version en vigueur (AC-i6, désormais `numeroEnVigueur`), de retour au standard
   (AC-iii11 (d), AC-i10), les codes d'erreur (AC-iii6, AC-iii8, D2 en 400) ; la portée qui sert un prompt
   (C-32-10a-16) ; la forme de la marque (*Dev Notes — La marque*). Une règle de l'écran qui contredit ou refait la
   32-10a est un finding. La note complétée est-elle réalisable sans affaiblir une garde de la 32-6 ou de la 32-10a
   (`un_seul_point_d_entree`, `LIB_NOMME_MCP`, `prompts_lus_par_leur_service`, `mcp_sans_pool`) ?
3. **Le modèle 32-8c** : la fiche reprend-elle juste ses motifs (fabriques de corps seules, version lue, 409 et
   *Recharger*, un seul état d'envoi par page, `ecran_sans_html_brut` et son témoin rouge composé, garde des replis et
   son témoin vert, `FAMILLES_RESOLUES`, `sitesTotal`, E2E à contexte neuf) ? S'en écarte-t-elle sans le dire ?
4. **Les caractères cachés** (AC-4, AC-5, AC-6) : ce que l'écran affiche vient-il **entièrement** du serveur ? L'avis
   permanent dit-il **seulement** ce que fait `rendre_visible` (prédicat `is_hidden_in_text`,
   `crates/kesh-core/src/text.rs:119`) ? Ce que la fiche dit admis et non montré reprend-il l'angle mort écrit
   (`text.rs:79-85`) ?
5. **L'i18n** : préfixe `mcp-prompts-` contre le catalogue et contre la 32-10a-iii ; variantes (a) et (b) de l'AC-15
   contre `mcp-sends-i18n.test.ts` et la fiche 32-7 (AC-14) ; témoin vert `audit-log-entity-mcp-prompts-x` ; G3-bis
   (`crates/kesh-i18n/src/loader.rs`) ; le texte de l'avis (désormais vérifié par un test) est-il une valeur de
   catalogue cohérente avec la règle des replis ?
6. **L'E2E** (AC-16) contre `mcp-sends.spec.ts` et la fiche 32-10a : (a) prouve-t-il toujours « valeur vue = valeur
   servie » sans prétendre vérifier le champ rendu visible ? (b) sur un nom neuf, membre `error` égal : réalisable ?
   (d) compte relatif `k → k + 2` ; (e) patron du Comptable.

## Lentille F — la propriété centrale, la sécurité, la faisabilité

1. **La propriété : ce qui est approuvé est ce qui a été vu, et ce qui est servi est ce qui a été approuvé.** Cherche
   un chemin **hors** de l'inventaire E1 à E18 : rendu, relecture, cache, ordre des requêtes, double clic, deux
   onglets, gestes concurrents sur une même page, amorçage entre lecture et clic, restauration, version
   d'administrateur venue d'un autre onglet. Chaque ligne de l'inventaire est-elle fermée — ou dite angle mort — par ce
   qu'elle dit ?
2. **La marque, sans encadré** : l'Administrateur la distingue-t-il encore du texte de l'agent (une imitation de la
   marque par l'agent, servie après la vraie) ? Le retrait de l'encadré ouvre-t-il un chemin que l'inventaire ne nomme
   pas ? Le test sur une version d'agent prouve-t-il que la marque est affichée à sa place ?
3. **Les caractères cachés** : l'avis réduit dit-il vrai là où l'Administrateur décide ? E17 tient-elle avec la phrase
   ajoutée, et sa réserve (`\r`) est-elle portée partout où la phrase l'est ?
4. **Les rôles** : un Comptable, une clé d'API, une clé MCP — que voit-on, que peut-on faire ? L'écran offre-t-il un
   geste que la route refuserait (retour au standard, approbation d'une version non servable), ou l'inverse ?
5. **Découpage et faisabilité** : trois modules ; chaque fichier de l'esquisse et des AC dans un module compté ; aucune
   tâche ne suppose un fichier, un champ ou un composant introuvable.
6. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation M1 à M17 réalisable et tuée par le test
   **nommé** — une passerait-elle à vide ? La mutation équivalente déclarée l'est-elle ? Les tests renommés
   (`en_vigueur_selon_le_serveur`, `gestes_desactives_pendant_l_envoi`, `avis_p26_si_pre_rempli_par_une_version_d_agent`,
   `pastille_agent_hors_du_bloc`) sont-ils définis là où ils sont cités ?
7. **Le registre et les questions** : chaque choix, annotations P2 comprises, dit-il la même chose que la fiche ? Une
   question pour Guy est-elle déjà tranchée ? Une décision de Guy est-elle contredite ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les **quatre**
   axes obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée — `grep -nF` pour toute présence ou
   absence —, ou ligne lue), la correction proposée (de préférence un **retrait**), et, pour l'axe 4, la classe
   « régression P2 », « régression P1 » ou « antérieur ».
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste, et la part « régression P2 ».

## Annexe — la consigne de remédiation de la P2, recopiée

*Ce que l'orchestrateur a demandé à l'agent de remédiation de la P2, à titre de contexte : juge la fiche contre cette
consigne aussi (une décision appliquée de travers est un finding).*

> Tu es l'agent de spécification de la Story 32-10b de Kesh, l'écran des prompts. Tu travailles **uniquement** dans le
> worktree `/home/gcorbaz/devel/kesh-32-10b`, sur la branche `story/32-10b`, et tu écris en français.
>
> ## Consignes
> Applique `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` :
> - chemins absolus ;
> - le dépôt principal est en lecture seule ;
> - ni push, ni `gh` qui écrit ;
> - ni cargo, ni npm, ni docker, ni curl ; la lecture est permise ;
> - si une commande est refusée, tu t'arrêtes ;
> - n'ouvre jamais `~/kesh-test/` ;
> - Hermes est hors périmètre.
>
> ## Tâche
> Remédier la validation P2 de la fiche `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Les rapports
> sont dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p2-R.md` (0/0/3/11) et
> `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p2-F.md` (0/0/7/5). Lis-les en entier.
>
> **Recyclage.** La plupart de ces findings naissent de la remédiation P1. Déclare-le au Change Log comme signal D5,
> pour le Project Lead. Nous en sommes à la passe 2 sur 8. **Consigne de l'orchestrateur : retirer plutôt
> qu'enrichir.** Toute promesse que l'écran ne peut pas tenir simplement se retire.
>
> **La fiche 32-10a a été complétée.** Lis la version `20a02a86` dans `/home/gcorbaz/devel/kesh-32-10a`, au § *À
> recontrôler à l'étape 0*, note « D1 étendu, complément ». Elle rend :
> - le texte **et** la description servis, chacun exact **et** rendu visible. La marque fait partie du texte servi :
>   il n'y a **pas** de champ `marque` séparé ;
> - la même chose pour le standard du code ;
> - `creeLe`, `decideLe` et `numeroEnVigueur`.
>
> La composition se fait dans `services/`, et le refus de D2 rend un **400**.
>
> **Décisions sur les MEDIUM :**
> - **R1 = F6 (champs manquants, marque).** L'écran ne consomme que les champs de la note complétée. **Retire
>   l'encadré séparé de la marque** : elle se lit dans le texte rendu visible, à sa place, sans traitement de l'écran.
>   Retire `marque_dans_le_bloc` et M13 dans leur forme actuelle. Le test DOM d'égalité de `textContent` avec le champ
>   visible suffit, à condition de porter sur **une version d'agent**, donc marque comprise, avec une origine fixée
>   (F2). Mets à jour la liste d'arrêt de l'étape 0 pour qu'elle nomme les champs consommés.
> - **R2 (fonction de composition).** Elle est réglée par la note : la composition est dans `services/`. Cite-la,
>   sans rien ajouter de plus.
> - **R3 = F1 (avis de l'AC-4).** Réduis l'avis à ce qui est vrai : l'écran montre le texte que Kesh servira, et les
>   caractères de contrôle et de mise en page y sont écrits en code. Ajoute la phrase sur le `\u{…}` écrit en toutes
>   lettres, sur laquelle E17 s'appuie, ou bien retire l'affirmation « l'avis le dit » d'E17 et de l'AC-6. **Ne
>   promets pas** « chaque caractère invisible ». Un test vérifie le texte de l'avis.
> - **F3 = R13 (M17 qui passe à vide).** Suis le modèle de la 32-8c. Le test vérifie l'état `disabled` des gestes
>   pendant l'envoi ; F8 demande de **tous** les gestes de la page, comme la 32-8c. Si aucune mutation ne peut tuer de
>   façon distincte le retour anticipé, déclare-la équivalente et retire-la.
> - **F4 (E2E et champ visible).** Retire de la fiche la prétention que l'E2E vérifie le champ rendu visible, aux
>   l. ≈670-672. Le test DOM couvre l'affichage, l'E2E couvre l'égalité du texte exact avec `prompts/get`. Si tu veux
>   un E2E à caractère caché, il passe par une version restaurée ; c'est trop lourd ici, écris-le en angle mort.
> - **F5 = R11 (bouton de retour au standard).** Le bouton n'apparaît que si `standardDiffere && standardDuCode !==
>   null`. Retire le cas impossible.
> - **F7 (version après restauration).** Écris la ligne d'inventaire E18 en angle mort : le verrou `version` compte
>   des écritures, il ne désigne pas un contenu. C'est le même angle mort que l'issue #773 (point 15) pour les envois.
>
> **LOW :** tous à intégrer ou à retirer. Les plus nets :
> - R4 : 400 ;
> - R5 : `numeroEnVigueur` vient du serveur, l'écran ne recalcule rien ;
> - R6 = F11 : mettre à jour `sprint-status.yaml` ;
> - R7 : faire de la phrase sur le `\r` restauré une affirmation conditionnelle ;
> - R9, R10 : témoins de la garde ;
> - R12 : citer `20a02a86`, et noter D5 comme inscrit ;
> - R14 : E2E (b) ;
> - F9 : cas impossible ;
> - F10 : question 6 retirée, déjà tranchée par l'AC-iii7 ;
> - F12 : avis P26 au pré-remplissage.
>
> **Après le patch :**
> - greppe les valeurs (`marque_dans_le_bloc`, `chaque caractère invisible`, `409`, `e7a90572`, `pastilles`,
>   `16 mutations`) ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris l'entrée P2 au Change Log, avec le trend 0/3/8/15 → 0/0/10/16.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P3 : `_bmad-output/implementation-artifacts/3210b-validate-prompt-p3.md`. Lentilles R et F,
>   **Sonnet**, contexte frais. Rapports dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p3-{R,F}.md`. Mêmes
>   exigences et interdits, plus un axe sur les régressions de cette remédiation. Fiche 32-10a à lire à `20a02a86`. Ce
>   prompt-ci est recopié en annexe.
> - Un rapport final bref.
