# Prompt — revue de code P2 de la Story 32-10b (l'écran des prompts du serveur MCP) : passe ciblée, Opus

Tu fais la **passe ciblée** de la revue de code de la **Story 32-10b** de Kesh (comptabilité suisse, SvelteKit, Rust/Axum) :
**une seule lentille, la chasse aux régressions**, braquée sur **le seul diff de la remédiation de la passe P1**, non sur
la story entière. Modèle : **Opus**. Contexte frais. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10b`. Le diff à relire :
`git -C /home/gcorbaz/devel/kesh-32-10a diff fd6741c6..HEAD -- frontend crates docs` — trois commits : `bf810366`
(la remédiation), `61d41aed` (un test de la liste), et celui qui porte ce prompt (manuel, registre, Change Log). Ne
relis le reste de la story que pour comprendre ce que le diff touche.

**Pourquoi une passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : la sévérité ne stagne pas, elle se déplace vers ce
qu'on vient d'écrire. La remédiation P1 touche du **code de production** (`McpPromptDetail.svelte`,
`McpPromptsList.svelte`, `mcp-prompts-format.ts`, la page `[id]`) : c'est là, et seulement là, que tu cherches.

## Ce que la P1 a demandé, et ce que la remédiation prétend faire

Lis les rapports P1 en entier : `/home/gcorbaz/devel/kesh-gate-logs/3210b-review-p1-{B,E,A}.md`. Puis, dans la fiche
`_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md`, l'entrée **« revue de code P1 »** du Change Log, et au
registre `epic-32-choix-autonomes-3210b.md` les choix **C-32-10b-29 à 35**. Les trois MEDIUM :

1. **B1 = E1 = A1** — `agir` sépare l'écriture de la relecture : une écriture réussie suivie d'un `GET` en échec doit
   notifier le succès, dire « enregistré, relecture échouée » et offrir *Recharger* ; jamais annoncer l'échec d'un
   geste qui a eu lieu.
2. **E2 = B2 = A2** — un *Recharger* en échec ne met plus `detail` à nul, ne perd pas la saisie conservée, et offre
   de réessayer ; l'amendement de l'AC-12 (la saisie survit au 409, KF-055 #796) tient hors du chemin nominal.
3. **E3** — l'erreur d'un geste de l'éditeur s'affiche près de lui, défile et prend le focus.

Et les LOW pris : E7, B5 = E6, B3 = E8 (état `occupe`), B7, E9, la pastille *En vigueur*.

## L'axe obligatoire : les régressions de la remédiation

Pour **chaque** ligne de production changée, cherche ce qu'elle a cassé ou ouvert :

- **`agir`** : les deux `try` imbriqués et le `finally` — `envoiEnCours` et `gesteEnCours` sont-ils toujours remis, y
  compris sur le `return` anticipé ? `saisieAConserver` après un succès, après une relecture en échec ? `fermerDialogues`
  sur chaque chemin ? `afficherErreur` appelé sans `await` (une course avec un `charger` suivant, un `erreur = null`
  posé entre-temps) ? la notification d'un succès suivie d'une erreur affichée : contradictoire pour l'utilisateur ?
- **`charger`** : `saisieAConserver` n'est plus remis à faux au début — un 409 d'un **autre** geste après un 409
  d'enregistrement, puis *Recharger* : la saisie est-elle gardée à tort (le contraire de l'AC-12 pour un geste sans
  saisie) ? Le chemin `id` invalide ; `detail === null` après un premier chargement réussi est-il possible ? *Réessayer*
  au premier chargement.
- **`occupe` = `envoiEnCours || chargement`** : `chargement` vaut vrai au montage — le premier rendu et le bouton
  *Réessayer* (`disabled={chargement}`) ; un geste désormais impossible au mauvais moment ? les dialogues, dont la
  fermeture reste liée à `envoiEnCours` seul ; l'éditeur, qui reçoit `occupe` pour `envoiEnCours`.
- **L'alerte en `{#snippet}`** rendue à deux places avec un seul `bind:this={elementErreur}` : la référence est-elle
  juste quand l'alerte change de place, ou quand elle disparaît ? `focus()` et `scrollIntoView` sur un élément démonté ?
  `role="alert"` et `tabindex="-1"` : un lecteur d'écran l'annonce-t-il deux fois ?
- **`McpPromptsList.creer`** : `let ecrite;` sans type ; le `return` dans le premier `catch` remet `envoiEnCours` ; la
  relecture `void charger()` après une navigation en échec.
- **`idDeLUrl`** et la page `[id]` : `page.params.id ?? ''`.
- **Les tests** de la remédiation (`McpPromptDetail.test.ts`, `describe('… remédiation de la revue de code P1')` ;
  `McpPromptsList.test.ts`, `mcp-prompts-format.test.ts`) : passent-ils **à vide** (une réponse « une fois » restée en
  file, une assertion après une promesse jamais libérée, `document.activeElement` comparé trop tôt) ? Le témoin du test
  des dialogues (Échap ferme hors envoi) existe-t-il bien ? Les mutations M20 à M26 (dont M20b, M21b, M22b, M24a, M24b)
  — journaux `/home/gcorbaz/devel/kesh-gate-logs/3210b-mut-M2[0-6]*.log`, `3210b-mutations-p1.log`, script
  `3210b-mutations-p1.sh` (à **lire**, jamais à exécuter) — sont-elles tuées pour la raison visée ?
- **Le symptôme ailleurs** : la même relecture « écriture + GET dans un seul `try` », le même `detail = null` sur
  rechargement, le même `Number(page.params.id)` vivent-ils dans un autre écran (`features/mcp-sends/`, la page des
  envois) ? Un tel site hors de la story est un finding **antérieur**, à signaler, non à reprocher à la remédiation.
- **i18n** : trois clés neuves (`mcp-prompts-error-reread`, `-retry`, `-created-open`) dans les quatre locales,
  `NOMBRE_DE_CLES` 94, `sitesTotal` 2513 et sa généalogie (recompte par `grep -oE '(i18nMsg|[^a-zA-Z]msg)\('`).

## Axe du manuel

Le paragraphe « Recharger après un conflit » de `\subsection{Les prompts du serveur MCP}` (`docs/manual/fr/admin-manual.tex`)
gagne trois phrases (rechargement en échec, geste enregistré dont la relecture échoue, refus d'un enregistrement
au-dessus de l'éditeur). Confronte-les au code, puis au **PDF aplati** :
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE ".{80}(Recharger après un conflit|le geste a eu lieu|Un rechargement qui échoue).{160}"`.

## Les captures

`/home/gcorbaz/devel/kesh-gate-logs/3210b-captures/` a été refait après la remédiation : ouvre au moins `12-…`
(l'erreur près de l'éditeur), `13a-…`, `13b-…` et `09-…` (la pastille *En vigueur*), et dis s'ils montrent ce que le code
fait.

## Interdits absolus (les mêmes que la P1)

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`). Un fichier créé par erreur se **déclare** au rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git restore`), aucun `git` qui écrit (`commit`,
  `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs` (dont
  `3210b-mutations-p1.sh`, `3210b-muter.mjs`, `3210b-captures.mjs`, `3210b-e2e.sh`) ; ni `make`, ni `latexmk`, ni
  `cargo`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, la lecture d'images.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; ne lis rien sous `~/kesh-test/`.

## Rapport

Un seul fichier, par l'outil d'écriture de fichier (jamais une redirection) :
`/home/gcorbaz/devel/kesh-gate-logs/3210b-review-p2.md`. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et s'il est
   **né de la remédiation P1** ou **antérieur**.
2. **Pour chacun des trois MEDIUM de la P1** : fermé, fermé en partie, ou non fermé — avec la preuve.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
