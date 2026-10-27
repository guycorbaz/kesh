# Prompt — validation P2, Story 32-10b (l'écran des prompts du serveur MCP)

*Versionné le 2026-10-27. Passe **complète**, deuxième sur huit. **Deux lentilles, R et F**, chacune en contexte frais,
**Opus**. Lecture seule stricte. Elle suit la remédiation de la P1 (trend P1 **0/3/8/15** ; entrée « validation P1 » du
Change Log de la fiche), qui a **refondu** la conception de l'écran : il n'a plus aucune logique Unicode et affiche ce
que le serveur rend (amendement « D1 étendu » de la 32-10a).*

Dépôt `/home/gcorbaz/devel/kesh-32-10b`, branche `story/32-10b` (fiche 32-10a + `origin/main` `ed981c7d` fusionné,
merge `42680725` : la 32-8 complète, 32-8c comprise ; la **32-10a est validée, non développée** ; la 32-7 n'est pas
fusionnée). Fiche : `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210b.md` (C-32-10b-1 à 19). Le commit de la remédiation
P1 se lit par `git log -1 --format=%H` puis `git show <ce commit>` (et `git diff c6497a36 HEAD --
_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`). **La fiche 32-10a à jour de D1 étendu est sur une autre
branche** : lis-la dans le worktree `/home/gcorbaz/devel/kesh-32-10a` (branche `story/32-10a`, commit `e7a90572`),
`_bmad-output/implementation-artifacts/32-10a-prompts-en-base.md`, § *À recontrôler à l'étape 0* (amendements D1, D2,
D1 étendu, l. 1533-1538 au relevé) — la copie présente dans le worktree `kesh-32-10b` est **antérieure** à ces
amendements. Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-R.md` et `…-p1-F.md`.

Contexte : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *Décisions du Project Lead*, § *32-10*,
*Réponses de Guy*) ; la fiche 32-10a (AC-i5, AC-i6, AC-i7, AC-i8, AC-ii2, AC-ii5 à AC-ii7, AC-iii6 à AC-iii8, AC-iii11,
inventaire P1 à P27, § *La frontière avec la 32-10b*, § *Le manuel*, *Dev Notes*) et son registre ; **le modèle voisin,
fusionné** : la fiche `32-8-validation-avant-envoi.md` partie C, le registre `epic-32-choix-autonomes-328.md`, le code
`frontend/src/lib/features/mcp-sends/`, `frontend/src/routes/(app)/admin/mcp/sends/`, `frontend/tests/e2e/mcp-sends.spec.ts`,
et **`crates/kesh-api/src/services/envois_mcp.rs`** (`rendre_visible`, l. 75, et ses tests) ; **la fiche 32-7, en
lecture seule** : `/home/gcorbaz/devel/kesh-32-7/_bmad-output/implementation-artifacts/32-7-ecran-serveur-mcp.md`.
Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Recette visuelle*, *Le prompt
d'une passe doit NOMMER le manuel*, *Recompter ses propres comptes rendus*, *Propagation post-patch*, *Règle de
splitting préventif*), `docs/testing.md`, `docs/agents-orchestration.md`.

**Contraintes de la story, à tenir pour vraies et à contrôler** : au plus trois modules (frontend, `kesh-i18n`, manuel
d'administration) ; la 32-10b n'ajoute **aucune route** ni code Rust ; **l'écran n'a aucune logique Unicode** : il
affiche le texte rendu visible par le serveur et le verdict `servable` du serveur (décision de l'orchestrateur, P1) ;
un prompt est une instruction donnée à un agent qui lit des livres réels : l'écran doit montrer **exactement** le texte
servi, et l'approbation se faire sur la version **vue**, avec contrôle de version ; la marque d'un prompt d'agent
visible à l'écran comme dans le texte servi ; le retour au standard sans écrasement, sur un texte vu ; l'Administrateur
seul voit et décide ; captures en thème clair seulement ; Hermes hors périmètre ; aucune donnée réelle. Ce que la fiche
**prescrit** n'existe pas encore : son absence n'est pas un défaut. Ce que la fiche cite **de la fiche 32-10a** est
prévu, non codé : juge sa cohérence avec cette fiche (à `e7a90572`), non sa présence dans le code.

⛔ **Interdits** : n'écris **aucun** fichier dans le dépôt ni dans `/home/gcorbaz/devel/kesh-32-7` ni dans
`/home/gcorbaz/devel/kesh-32-10a` ; n'exécute **ni** `cargo`, `npm`, `npx`, `node`, `make`, `docker`, `curl`, **ni aucun
interpréteur** (`python`, `perl`, `awk`, `sh -c`…), **ni redirection** (`>`, `>>`, `tee`) hors de l'écriture de ton
rapport par l'outil d'écriture, **ni** `scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/regen-test-schema.sh` ou tout autre script ; **aucune** commande `gh` ni `git` qui écrit (`commit`, `push`,
`checkout`, `switch`, `reset`, `stash`, `merge`, `rebase`, `tag`, `worktree`) ; n'ouvre jamais `~/kesh-test/`.
Commandes permises : `grep`, `sed -n`, `ls`, `wc`, `cut`, `sort`, `head`, `tail`, `git log`, `git show`, `git diff`,
`pdftotext` vers la sortie standard. Le seul fichier que tu écris est ton rapport.

## Les quatre axes obligatoires, pour les deux lentilles

1. ⛔ **Chaque affirmation de présence ou d'absence, prouvée par une sortie `grep -nF` copiée** (ou `sed -n` de la
   ligne citée). Dresse d'abord la liste des motifs et des mesures : `grep -nE "grep -|→ \*\*|au relevé|mesure"
   _bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Pour chaque mesure (faits 1 à 13, AC-13, AC-15,
   AC-19), exécute-la dans son périmètre et copie la sortie ; une mesure que tu ne reproduis pas, un témoin
   « existant » absent de la ligne citée, une ligne citée qui ne dit pas ce que la fiche lui prête, sont des findings.
2. ⛔ **Le manuel, au `.tex` ET au PDF aplati.** `grep -n 'sec:mcp-envois\|routes à verbe mutant\|section{Sécurité}\|
   section{Monitoring' docs/manual/fr/admin-manual.tex` ; `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' |
   tr -s ' ' | grep -oiE '.{60}(envois du serveur MCP|routes à verbe mutant|prompt).{80}'`. L'AC-18 prescrit-elle tout
   ce que la fiche 32-10a laisse à la 32-10b, et chaque phrase prescrite décrit-elle ce que **le serveur** fera (en
   particulier la phrase sur les caractères invisibles, réécrite en P1, contre `rendre_visible` et `valider_contenu`) ?
3. ⛔ **Recompter les décomptes de la fiche depuis sa source** : 20 AC (A 4, B 2, C 8, D 6), 9 tâches (T0 à T8), **17**
   mutations (M1 à M17), **E1 à E17**, C-32-10b-1 à **19** (chacun cité par la fiche existe au registre et y dit la
   même chose), 7 points pour Guy, **5** demandes ; chaque total contre sa ventilation, **et** contre l'entrée
   « validation P1 » du Change Log (`CLAUDE.md` § *Recompter*).
4. ⛔ **Les régressions de la remédiation P1** (axe neuf, `CLAUDE.md` § *La passe ciblée* : « la sévérité se déplace
   vers ce qu'on vient d'écrire »). Lis le diff de la P1 **en entier** et cherche ce qu'il a cassé ou laissé :
   - **résidus** de la logique Unicode retirée : une AC, un test, une mutation, une capture, une ligne du manuel, un
     angle mort, une question pour Guy ou une entrée du registre qui parle encore de `segmenter`, de pastilles
     `⟦U+…⟧`, de comptes `refuse` / `admis`, de mode `ligne`, de garde de version d'Unicode, de variante B — greppe
     les **valeurs** (`grep -nF`), non les formulations ;
   - **cohérence des champs neufs** (`servi.texteVisible`, `descriptionVisible`, `servable`, `standardDuCode`,
     `auteurNom`, `decideurNom`, `modifieLe`) entre les AC, les types des *Dev Notes*, l'E2E, la note D1 étendu de la
     fiche 32-10a (worktree `kesh-32-10a`) — un champ nommé dans une AC et absent des types, ou l'inverse, est un
     finding ;
   - **propriétés neuves tenues par ce qu'elles disent** : la règle des blancs de gabarit et les tests au niveau du
     DOM (`texte_affiche_egal_au_champ_visible`, `marque_dans_le_bloc`) prouvent-ils ce qu'ils annoncent, sans passer
     à vide ? les mutations réécrites (M2, M3, M4, M12, M13) et M17 sont-elles tuées par le test **nommé** ?
     `servable` est-il employé de façon cohérente (AC-4, AC-6, AC-7, C-32-10b-17, E14) ? la condition du bouton de
     retour au standard réduite au drapeau `standardDiffere` (AC-10, C-32-10b-19) est-elle exacte au regard de l'AC-i5
     de la 32-10a ?
   - **prémisses des nouvelles affirmations** : `rendre_visible` (fait 12) fait-il ce que la fiche en dit
     (`crates/kesh-api/src/services/envois_mcp.rs:75`, tests l. 291-334) ? Une version servable a-t-elle
     nécessairement `texteVisible === texte` (le cas `\r`, écrit à l'étape 0) ? Le fait 8 (130, 124) et le fait 13
     (patron du Comptable) sont-ils exacts ?
   - **angles morts neufs** (E16, E17, les caractères admis non montrés, les lettres RTL) : la conception en ouvre-t-elle
     un qu'elle ne nomme pas ?
   Classe chaque finding de cet axe « **régression P1** » ou « **antérieur** » dans ton rapport.

## Lentille R — la spécification contre le code et contre la fiche 32-10a

1. **Les faits 1 à 11 et § *Ce qui est lu et cité*** : chaque numéro de ligne, ouvert.
2. **Le contrat avec la 32-10a** : chaque ligne du tableau « Ce que dit la fiche 32-10a » contre l'AC qu'il cite ;
   les règles d'approbation (AC-i7 : dernière version, états terminaux), de version en vigueur (AC-i6), de retour au
   standard (AC-iii11 (d), AC-i10), les codes d'erreur (AC-iii6, AC-iii8) ; la portée qui sert un prompt (C-32-10a-16) ;
   la forme de la marque (*Dev Notes — La marque*). Une règle de l'écran qui contredit ou refait la 32-10a est un
   finding. **D1 étendu** (la note de la fiche 32-10a à `e7a90572`) est-il réalisable dans les modules déjà comptés par
   la 32-10a-iii (le `routes` et le `mcp` de sa table) sans affaiblir une garde de la 32-6 ou de la 32-10a
   (`prompts_lus_par_leur_service`, `mcp_sans_pool`, la chaîne des types de l'AC-ii8) — en particulier l'appel de
   `rendre_visible` (`services/envois_mcp.rs`) depuis la route des prompts, et le texte du code d'un standard ?
3. **Le modèle 32-8c** : la fiche reprend-elle juste ses motifs (fabriques de corps seules, version lue, 409 et
   *Recharger*, `ecran_sans_html_brut`, garde des replis, `FAMILLES_RESOLUES`, `sitesTotal`, E2E à contexte neuf) ?
   S'en écarte-t-elle sans le dire (C-32-10b-3 annoté contre C-32-8-20 ; C-32-10b-9 ; C-32-10b-17 à 19) ?
4. **Les caractères cachés** (AC-5, AC-6) : ce que l'écran affiche vient-il **entièrement** du serveur — rendu visible
   par `rendre_visible` (`crates/kesh-api/src/services/envois_mcp.rs:75`, prédicat `is_hidden_in_text`,
   `crates/kesh-core/src/text.rs:119`) et verdict `valider_contenu` (AC-ii2 de la 32-10a) ? Ce que la fiche dit admis
   et non montré reprend-il l'angle mort écrit (`text.rs:79-85`) ? D1 étendu, tel qu'inscrit à la fiche 32-10a
   (`kesh-32-10a`, `e7a90572`), rend-il **tous** les champs que la fiche 32-10b consomme ?
5. **L'i18n** : préfixe `mcp-prompts-` contre le catalogue (`grep -n '^mcp-prompt' crates/kesh-i18n/locales/fr-CH/
   messages.ftl`) et contre ce que la 32-10a-iii écrira ; les variantes (a) et (b) de l'AC-15 contre `mcp-sends-i18n.
   test.ts` et la fiche 32-7 (AC-14) ; G3-bis (`crates/kesh-i18n/src/loader.rs`) — un texte prescrit écrit-il
   « Administration → … » dans un catalogue ?
6. **L'E2E** (AC-16) contre `mcp-sends.spec.ts` et la fiche 32-10a : la clé à deux portées se crée-t-elle par la route
   de la 32-5c (`crates/kesh-api/src/routes/` — lis la validation des portées) ? `proposer_prompt` et `prompts/get` ont-ils
   la forme supposée (AC-iii2, AC-iii3, AC-ii6 de la 32-10a) ? Un préréglage Comptable existe-t-il (`frontend/tests/
   e2e/helpers/`) ?

## Lentille F — la propriété centrale, la sécurité, la faisabilité

1. **La propriété : ce qui est approuvé est ce qui a été vu, et ce qui est servi est ce qui a été approuvé.** Cherche
   un chemin **hors** de l'inventaire E1 à E17 par lequel l'Administrateur approuve, enregistre ou rétablit un texte
   différent de celui que l'agent recevra : rendu, relecture, cache du navigateur, ordre des requêtes, double clic,
   deux onglets, un standard modifié par un amorçage entre la lecture et le clic, une restauration, une version
   d'administrateur venue d'un autre onglet. Chaque ligne de l'inventaire est-elle fermée par ce qu'elle dit ?
2. **La marque** : peut-elle manquer, être imitée par le texte d'un agent sans que l'écran le montre, être prise pour
   un ornement ? La règle « encadrée seulement si le texte commence par la marque » est-elle sûre ?
3. **Les caractères cachés** : un caractère qui échappe au rendu visible **et** au verdict du serveur (homoglyphes,
   `Zs`, `Mn` hors liste) ; un `\u{202E}` écrit en toutes lettres (E17) — le verdict `servable` suffit-il à le
   distinguer, et l'écran le dit-il là où l'Administrateur décide ? Les lettres bidirectionnelles visibles (angle mort
   écrit) ?
4. **Les rôles** : un Comptable, une clé d'API, une clé MCP — que voit-on, que peut-on faire ? L'écran offre-t-il un
   geste que la route refuserait, ou l'inverse ?
5. **Découpage et faisabilité** : recompte les modules selon la convention écrite (trois) ; chaque fichier de
   l'esquisse et des AC est-il dans un module compté ? La variante (b) de l'AC-15, qui touche la garde des envois,
   reste-t-elle dans le frontend ? Une tâche suppose-t-elle un fichier, une fonction, un composant (`Dialog`,
   `Button`, pastille) qui n'existe pas ?
6. **Tests et mutations** : chaque AC vérifiable tel qu'écrit ; chaque mutation M1 à M17 réalisable et tuée par le test
   **nommé** — une passerait-elle à vide (un test qui ne monte pas le cas) ? L'E2E (a) prouve-t-il vraiment « valeur
   vue = valeur servie » (texte affiché égal au champ visible, champ visible égal au champ exact pour ce texte, champ
   exact égal à `prompts/get`) ? L'E2E (d) prouve-t-il « texte vu dans le dialogue = texte servi » ?
7. **Le registre et les questions** : chaque choix dit-il la même chose que la fiche ? Une question pour Guy est-elle
   déjà tranchée (décisions de l'epic, de la 32-10a) ? Une décision de Guy (§ *Réponses de Guy*) est-elle contredite ?

## Rapport

Écris ton rapport — et rien d'autre — dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p2-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p2-F.md` (lentille F). Contenu :

1. **Axes exercés** et **axes non exercés** (avec la raison) — liste explicite, axe par axe, y compris les **quatre**
   axes obligatoires. Un « 0 finding » sans cette liste ne compte pas comme passe.
2. **Findings** numérotés (`R1`, `R2`… ou `F1`, `F2`…), chacun avec : sévérité (CRITICAL, HIGH, MEDIUM, LOW), le site
   de la fiche (section, AC), la **preuve** (commande exécutée et sortie copiée — `grep -nF` pour toute présence ou
   absence —, ou ligne lue), la correction proposée, et, pour l'axe 4, la classe « régression P1 » ou « antérieur ».
3. Le décompte `CRITICAL/HIGH/MEDIUM/LOW`, recompté depuis ta liste, et la part « régression P1 ».

## Annexe — la consigne de remédiation de la P1, recopiée

*Ce que l'orchestrateur a demandé à l'agent de remédiation de la P1, à titre de contexte : juge la fiche contre cette
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
> Remédie la validation P1 de la fiche `_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`. Les rapports
> sont dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-R.md` (0/2/5/9) et
> `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p1-F.md` (0/1/3/6) ; lis-les en entier. Vérifie chaque prémisse au
> dépôt, et écris au Change Log ce que tu réfutes.
>
> ### Décision de l'orchestrateur : fermer les classes par la conception, pas par des gardes
> La fiche 32-10a a reçu une note, sur la branche `story/32-10a` au commit `e7a90572`, que tu lis dans le worktree
> `/home/gcorbaz/devel/kesh-32-10a`, § *À recontrôler à l'étape 0* : « D1 étendu ». La route de détail rendra, pour
> chaque version :
> - **`servable`**, le verdict de `valider_contenu` calculé par le serveur ;
> - le texte servi **exact** ;
> - le texte servi **rendu visible par le serveur** (`rendre_visible` de la 32-8b) ;
> - pour un standard, **le texte du code** qu'un retour au standard ferait servir, rendu visible lui aussi ;
> - les **noms** de l'auteur et du décideur, et la **date** de dernière modification.
>
> Conséquences à écrire dans la fiche 32-10b :
> - **F1 = R2 (HIGH) et R1 (HIGH), plus E13 et E14 :** l'écran **n'a plus aucune logique Unicode**. Il affiche le texte
>   rendu visible fourni par le serveur et le verdict `servable` du serveur. Retire en conséquence :
>   - la garde `version_unicode_du_moteur` ;
>   - la copie des listes de `kesh-core` et sa garde ;
>   - les pastilles calculées dans le navigateur.
>
>   Le texte **exact** sert à l'égalité avec `prompts/get` dans l'E2E. Le texte **rendu visible** sert à l'affichage.
>   Prouve par un test que ce qui est affiché est bien le champ rendu visible, sans transformation.
> - **F2 = R7 :** le dialogue de retour au standard montre le texte du code, rendu visible, avant le geste. Ajoute la
>   ligne d'inventaire correspondante.
> - **F3 :** le verdict « servi ou non » vient de `servable`. Corrige l'avis de l'AC-4 pour qu'il soit exact pour une
>   version à valider.
> - **R3 :** corrige le libellé « Le standard a changé » selon le sens réel du drapeau dans la 32-10a (AC-i5).
> - **R4 :** les champs viennent de D1 étendu. Ajoute la contrainte des primitives.
> - **F4 et R8 :** un test au niveau du DOM vérifie que `textContent` du bloc égale le champ rendu visible et que la
>   marque se trouve **dans** le bloc. Écris la règle des blancs de gabarit. M13 ne doit pas pouvoir passer à vide.
>
> **Les LOW** sont tous à intégrer :
> - F7 = R5 : « deux versions de plus », avec des comptes relatifs ;
> - F6 = R6 : fixture d'une version à valider qui n'est pas la dernière ;
> - F8 : règle d'envoi en cours sur chaque dialogue ;
> - F9 = R9 : à écrire en angle mort ;
> - R10 : relève « 122 des 128 » ; le registre en compte 130, à recompter au développement ;
> - R14 : pas de helper Comptable ; reprendre le patron de `backup-schedule.spec.ts:206` ;
> - R16 : signaler dans les demandes à l'orchestrateur la ligne 2978 de la note 32-10a.
>
> Les autres (F5, F10, R11, R12, R13, R15) sont à intégrer aussi, ou à retirer s'ils tombent avec la nouvelle
> conception. Écris lesquels.
>
> **Les questions pour Guy** : la 1 et la 2 sont tranchées par l'orchestrateur, avec la recommandation, en attendant
> Guy. La 3 change de nature, puisque c'est le serveur qui rend visible : reformule-la.
>
> **Après le patch :**
> - greppe les valeurs (`version_unicode`, `UNICODE_VERSION`, `is_hidden_in_line`, `trois versions`, `standard a
>   changé`) ;
> - recompte à la source ;
> - ne réécris aucune entrée passée ;
> - écris l'entrée P1 du Change Log : trend 0/3/8/15.
>
> ## Livrables
> - Un commit `refs #723`, sans mot-clé de fermeture.
> - Le prompt de la P2 : `_bmad-output/implementation-artifacts/3210b-validate-prompt-p2.md`. Lentilles R et F,
>   **Opus**, contexte frais, rapports dans `/home/gcorbaz/devel/kesh-gate-logs/3210b-validate-p2-{R,F}.md`. Il garde
>   les mêmes exigences et les mêmes interdits, ajoute un axe obligatoire sur les régressions de cette remédiation, et
>   recopie ce prompt-ci en annexe.
> - Un rapport final bref.
