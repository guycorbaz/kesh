# Prompt — revue de code P1 de la Story 32-10b (l'écran des prompts du serveur MCP), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-10b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
*l'écran **Administration → Prompts du serveur MCP** (`/admin/mcp/prompts` et `/admin/mcp/prompts/[id]`) — la liste
et la création, le détail (le bloc « Texte servi par Kesh », les versions à valider, l'historique), l'approbation et le
refus d'une proposition d'agent derrière un dialogue, l'éditeur d'une nouvelle version (avis P26), le retrait, le
retour au texte standard qui montre le texte du code avant le geste ; les clés `mcp-prompts-*` et
`nav-admin-mcp-prompts` dans les quatre locales et leur garde ; la section `sec:mcp-prompts` du manuel
d'administration et le compte des routes mutantes ; le CHANGELOG ; l'E2E `mcp-prompts.spec.ts`.* Les routes et le
texte servi sont la 32-10a (fusionnée) : la 32-10b **n'ajoute aucune route et ne touche aucun code Rust**.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-10b`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-10a diff f2ac34d2...HEAD` (`f2ac34d2` = `main` fusionné à l'étape 0). Langue du
rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe d'une revue de code). Contexte frais : ne
t'appuie sur aucune passe antérieure.

⚠️ **La propriété centrale : la valeur vue est la valeur servie.** Un prompt est une **instruction** donnée à un agent
qui lit des livres. Le danger n'est pas l'écran qui plante : c'est l'Administrateur qui **approuve, enregistre ou
rétablit un texte qu'il n'a pas vu tel qu'il part** — marque absente, caractère invisible, proposition arrivée entre la
lecture et le clic, bouton au mauvais endroit, saisie perdue ou renvoyée d'office. L'écran ne doit avoir **aucune
logique Unicode** : il affiche ce que le serveur a rendu visible (`rendre_visible`) et le verdict `servable` du serveur.

⚠️ **Hors périmètre** : un défaut de la 32-10a (routes, composition, validation) que la 32-10b ne touche pas est un
finding **antérieur**, à classer comme tel.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-10b-ecran-prompts.md` — § *La conception, en une page*, les **AC-1 à AC-20**,
§ *L'inventaire clos* (E1 à E18), *Tests et mutations prévus* (M1 à M18), *Ce que les gardes ne voient pas*, *Dev
Notes*, le § *À recontrôler à l'étape 0* **en entier** — dont la note de l'orchestrateur **« Amendement de l'AC-12 : la
saisie survit au 409 »** (KF-055, #796 ; règle de Guy : un texte proposé par un agent n'est servi qu'après approbation,
un agent ne modifie jamais directement une version validée, C-32-10a-106), **à contrôler comme une remédiation** — et
le **Dev Agent Record** et le **Change Log** ; puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3210b.md`, **C-32-10b-20 à 28** (choix du
développement). Le code des routes : `crates/kesh-api/src/routes/mcp_prompts.rs` (DTO, codes d'erreur) et
`crates/kesh-api/src/services/prompts_mcp.rs` (`composer_version`, `marque_agent`, `valider_contenu`). Le Dev Agent
Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les
journaux `/home/gcorbaz/devel/kesh-gate-logs/3210b-*` (lecture seule) et les scripts `3210b-mutations.sh`,
`3210b-muter.mjs`, `3210b-captures.mjs` (à **lire**, jamais à exécuter).

**Les captures** (recette visuelle, thème clair) : `/home/gcorbaz/devel/kesh-gate-logs/3210b-captures/` (01 à 17,
13 et 14 et 15 en deux états). Ouvre-les (lecture d'image) et confronte-les à l'AC-17 et au code.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) : supprimer, c'est écrire.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`, ni aucun script de
  `kesh-gate-logs` (dont `3210b-mutations.sh`, `3210b-e2e.sh`, `3210b-captures.sh`) ; ni `make`, ni `latexmk`, ni
  `cargo`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, la lecture d'images.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## L'axe obligatoire pour chaque lentille : ce qui est vu, et ce qui part

1. **Ce qui est affiché** — pour chaque `<pre>`, `<textarea>`, `<input>` et chaque interpolation d'un texte venu du
   serveur (`features/mcp-prompts/*.svelte`) : quel champ du DTO (`texteServi.visible`, `texte.visible`,
   `texte.exact`, `standardDuCode.texte.visible`…), dans quelle zone (`servi-`, `non-servi-`, `a-valider-`,
   `historique-`) ; le texte **exact** d'un texte servi est-il jamais affiché ? l'éditeur se pré-remplit-il jamais avec
   un texte servi (marque comprise) ? un `{@html` ? les blancs du gabarit dans un `<pre>` ?
2. **Ce qui part** — chaque appel de `mcp-prompts.api.ts` : chemin, méthode, corps (fabriques seules), et **d'où vient**
   la `version` et le `numero` envoyés (l'objet affiché, une relecture, un état périmé ?). Un `GET` entre l'affichage
   et le `POST` ? un double envoi (double clic, Entrée dans un formulaire, clic sur un bouton désactivé) ? une relance
   automatique après un 409 ?
3. **La saisie de l'Administrateur** — après un 400, un 409 d'enregistrement, un 409 d'un autre geste, un succès,
   *Recharger*, *Reprendre cette version* : la saisie est-elle gardée, remise, ou perdue, et l'écran le dit-il ?
   L'amendement de l'AC-12 est-il tenu à la lettre (saisie disponible, rien d'envoyé d'office, réenregistrement à la
   `version` relue) ?

Recoupe avec l'inventaire E1 à E18 : un site non résolu que l'inventaire n'énumère pas est un finding.

## Lentille B — Blind (le diff seul, sans la fiche)

1. **`McpPromptDetail.svelte`** — `agir`, `poser`, `charger(garderSaisie)`, `recharger`, les quatre
   `confirmer…` (gardes, valeurs capturées), les quatre dialogues (`open`, `onOpenChange`, fermeture pendant l'envoi),
   `serviParKesh`, `historique`, `preRemplir`, l'ordre écriture → relecture → `notifySuccess`. Une relecture qui
   échoue après une écriture réussie : que voit l'Administrateur ? `saisieConservee` réinitialisée quand ?
2. **`McpPromptVersion.svelte`**, **`McpPromptEditor.svelte`** (`$bindable`, garde d'envoi), **`McpPromptsList.svelte`**
   (formulaire, `goto`, garde d'envoi, `onsubmit`).
3. **`mcp-prompts-format.ts`** (`peutApprouver`, `peutRefuser`, `peutRevenirAuStandard`, `etiquetteStandard`,
   `erreurDeGeste`) et **`mcp-prompts.api.ts`**, **`mcp-prompts.types.ts`** contre les DTO de
   `routes/mcp_prompts.rs` (champ par champ, nullabilité comprise).
4. **`shared/i18n-repli-catalogue.ts`** et la garde des envois qui l'emploie désormais : le comportement de
   `mcp-sends-i18n.test.ts` est-il inchangé (tests, `NOMBRE_DE_CLES`) ?
5. **Les tests** — un test peut-il passer **à vide** : une attente calculée par la fonction éprouvée, une assertion
   levée dans une simulation et avalée par un `catch` de l'écran (c'est arrivé à M1, `Dev Agent Record`), une
   réponse « une fois » qui fuit d'un test à l'autre, un sélecteur `^=` qui ne trouve rien et rend un tableau vide ?

## Lentille E — Edge cases

1. **Les états du détail** : aucune version approuvée ; en vigueur = retrait ; en vigueur non servable ; une version
   d'agent approuvée dont la clé est révoquée ; deux propositions (la plus ancienne non la dernière) ; une proposition
   non servable qui est la dernière ; un standard personnalisé, retiré, au drapeau levé sans texte du code ; un
   historique de 50 versions ; un texte de 16 000 caractères ; un `\u{202E}` écrit en toutes lettres.
2. **Les courses** : une proposition qui arrive pendant l'édition, pendant un dialogue ouvert, entre la confirmation et
   la réponse ; deux onglets ; un prompt supprimé ou passé à une autre société (404) pendant l'affichage.
3. **Les erreurs** : 400, 403, 404, 409 (`OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION`), 500, réseau ; après
   un 409, *Recharger* puis un nouveau 409 ; un `id` illisible dans l'URL.
4. **Les mutations** (`/home/gcorbaz/devel/kesh-gate-logs/3210b-mut-*.log`, `3210b-mutations-2.log`) : chacune tuée
   par une **assertion nommée** et pour la raison visée (lis le journal) ? M18 déclarée équivalente (C-32-10b-25) : la
   démonstration tient-elle ? Une mutation plausible **non jouée** survivrait-elle — le pré-remplissage par
   `texteServi.exact`, `saisieAConserver` posé pour tout geste, la zone `servi-` montée pour une version non servable,
   `Dialog.Root` qui se ferme pendant l'envoi, *Retirer* offert sur un retrait en vigueur ? Raisonne sur le code,
   n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque **AC-1 à AC-20**, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve, **en
tenant compte de l'étape 0** (DTO réels, C-32-10b-21 et 22 ; amendement de l'AC-12 ; LOW J1 à J5). En particulier :
AC-5 et AC-6 (aucune logique Unicode : `grep -nE "\\\\p\{|codePointAt|normalize\(|charCodeAt" frontend/src/lib/features/mcp-prompts`),
AC-7 (la `version` lue, un seul `GET`, le dialogue, l'envoi en cours), AC-10 (le texte du code avant le geste), AC-15
(la garde, `sitesTotal` 2394 → 2510 et sa généalogie recomptée, `FAMILLES_RESOLUES`), AC-16 (chaque cas de l'E2E, et
le journal `3210b-e2e-complet.log` jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*), AC-17
(chaque capture présente et conforme), AC-19 (le compte des routes : `audit_route_registry.rs` — 129 tracées, 4
exemptées, 2 sans objet, 135 — contre le `.tex` **et** le PDF aplati), AC-20 (aucun fichier Rust ni migration dans le
diff : `git diff --stat f2ac34d2...HEAD -- crates/ ':!crates/kesh-i18n/locales'`).

Et : les choix C-32-10b-20 à 28 sont-ils justifiés, l'un contredit-il un AC, la fiche ou une décision de Guy ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E, clés) se recompte-t-il depuis sa source, avec son périmètre ?

## Axe du manuel (obligatoire pour les trois lentilles)

Confronte **au code** chaque affirmation de `\subsection{Les prompts du serveur MCP}` (`docs/manual/fr/admin-manual.tex`,
après `sec:mcp-envois`) — standards et personnalisation (`amorcer_standards`, `differe_du_standard`), propositions
(`proposer_prompt`, plafond `PLAFOND_A_VALIDER`, noms en `kesh`), approbation de la seule dernière, *Recharger* et la
saisie conservée, la marque (`marque_agent`, texte exact), les caractères invisibles et les angles morts, le retour au
standard, le journal d'audit (`mcp.prompt_served` et les cinq actions), sauvegarde et réinitialisation —, puis le **PDF
aplati** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(prompts du serveur
MCP|ôte la marque|Revenir au texte standard|129 des 135|129 \+ 4 \+ 2).{120}"`. Le PDF dit-il la même chose que le
`.tex` ? Le CHANGELOG (entrée « L'écran des prompts du serveur MCP ») est-il vrai ? Un texte publié devenu faux
(`README.md`, `website/`, confiés à la 32-11) se signale avec son porteur.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3210b-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/3210b-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/3210b-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la 32-10b ou **antérieur**.
2. **L'énumération de ce qui est vu et de ce qui part** (axe obligatoire), champ par champ, appel par appel.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, s'il y en a.
