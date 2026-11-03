# Prompt — revue de code P3 de la Story v016-1 (correctif 0.16.1 du serveur MCP), passe ciblée, une lentille, Sonnet

Tu relis la **remédiation de la revue de code P2** de la **Story v016-1** de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : *le correctif 0.16.1 du serveur MCP* — #811 (plus aucun `structuredContent` dans un
`result` de `tools/call`) et #812 (une seule forme de montant). Passe **ciblée** (`CLAUDE.md`, § *La passe ciblée*) :
**une seule lentille**, modèle **Sonnet**, contexte frais. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`. La remédiation à relire, **aplatie** :
`git -C /home/gcorbaz/devel/kesh-0161 diff a47f08dc..HEAD` — le commit de code `f6b338a9` (manuels FR et PDF,
CHANGELOG, `sortie.rs`, `mcp_contrat.rs`, `mcp_outils_e2e.rs`, annotations de `_bmad-output/`, registre, sprint-status),
puis le commit du Change Log et de ce prompt. Les findings de la P2 et leur sort sont à l'entrée « revue de code P2 »
du Change Log de la fiche `_bmad-output/implementation-artifacts/v016-1-mcp-sorties.md` ; les choix neufs au registre
`v016-1-choix-autonomes.md`, **C-v016-1-18 et C-v016-1-19**. Les rapports P2
(`/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p2-{B,E,A}.md`) se lisent pour juger si chaque remède répond à son
finding — ne reprends pas leurs conclusions sans les refaire.

**Le motif mesuré, qui justifie la passe** : les trois MEDIUM de la P2 portaient sur des phrases du manuel, deux écrites
par la story ou par sa remédiation P1. La sévérité se déplace vers ce qu'on vient d'écrire : la P2 a réécrit une dizaine
de phrases du manuel. C'est là que tu cherches.

Le commentaire de recette (#723, 2026-10-28), **recopié ici** puisque tu n'as pas droit à `gh` :

> (a) `initialize` puis `tools/list` sans session : **Oui** : aucun `Mcp-Session-Id` dans tout l'échange ;
> `initialize`, `notifications/initialized` (202), puis `tools/list` et `prompts/list` (200), sur des connexions
> parallèles. (b) `Origin` envoyé ? **Non** : aucun en-tête `Origin` dans toute la capture. (c) `progressToken` dans
> `_meta` de `tools/call` ? **Oui**, à chaque appel (avec `claudecode/toolUseId`). (d) l'agent rappelle-t-il l'outil de
> lecture, combien de fois ? **Une fois par envoi, après l'accord de l'utilisateur**, sans boucle ; il explique
> l'attente et demande qu'on le prévienne. Une relecture de plus sur demande explicite. ⚠️ Le contenu validé n'atteint
> pas le modèle : **#811** (P2). (e) Repli après le `400` texte de la sonde moderne ? **Oui** : sonde
> `server/discover` en `2026-07-28`, `400` texte (« MCP-Protocol-Version non servie… »), puis `initialize` en
> `2025-11-25` réussi, immédiatement. (f) Un `401` déclenche-t-il une découverte OAuth ? **Non observé**. (g) Forme du
> prompt : Menu : `kesh:kesh-assistant`, `kesh-direct:kesh-assistant`… ; commande envoyée :
> `/mcp__kesh-direct__kesh-assistant` ; `prompts/get` sans argument, cadre repris par le modèle. (h) `protocolVersion`
> demandée à `initialize` : `2025-11-25` (après la sonde `2026-07-28`). Contexte : NAS, Kesh 0.16.0, base neuve sans
> donnée réelle, derrière Traefik en HTTP ; Claude Code 2.1.312, transport HTTP, portée `local`.

⛔ Ne lis **pas** `kesh-gate-logs/3211/capture-recette.log` (capture de la recette), ni rien sous `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` (dont
  `muter.py`, `campagne.sh`, `v0161-annoter.py`, `v0161-e2e.sh`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni
  **`docker`**, ni `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni
  `playwright`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`,
  et `pdftotext -f <p> -l <p> <f>.pdf -` (une page, vers la sortie standard).
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -cF` ou
  `grep -oF` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'Ce que Kesh rend à l’agent'`.
  ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) : `grep -cF "l’agent"`, non `l'agent`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques.

## Axe 1 — le diff de la remédiation seul (`a47f08dc..HEAD`)

- **Chaque remède répond-il à son finding**, sans en dire plus ? B1 = E2 = A1 (`sec:mcp-outils`, paragraphe renommé
  « Ce que Kesh rend à l'agent ») ; B2 (« son numéro et son état », manuel item 5 **et** CHANGELOG, contre
  `git show d3b3b649:crates/kesh-api/src/mcp/reponse.rs`, l. 53 et 59) ; E1 (`sec:mcp-envois` 3e puce,
  `user-manual.tex` *Ce qu'on vous montre avant l'envoi*) ; B3 = E5 (doc-comment de `Montant` : recompte des types de
  colonnes, `grep -ohiE "DECIMAL *\( *[0-9]+ *, *[0-9]+ *\)" crates/kesh-db/migrations/*.sql`, et « seize chiffres
  entiers au plus » juste ?) ; B4 (`bilan_montants_une_seule_forme` : les totaux relus après l'écriture — l'assertion
  `totalActifs == "10.00"` est-elle juste par construction du bilan, ou fragile au signe ?) ; B5 (statut HTTP à chaque
  site de `k23_un_seul_canal`, S1 à S9, aucun oublié : `grep -nF 'Lu(23' crates/kesh-api/tests/mcp_contrat.rs`) ; E3
  (titre *Corrigé* de #811 : vrai dans les deux issues de la recette, et dit-il ce qui change réellement ?) ; E4
  (*Passer à la 0.16.1* : la phrase « modifié » contre `textes.rs`, `texte_envoi_valide`) ; E6 = A2 (item 7) ; A3
  (item 5, marque « non éprouvé ») ; A4 (`sprint-status.yaml`, commentaire de la clé `v016-1-mcp-sorties`, sans
  réécriture du relevé passé) ; A5 (dix-huit annotations « dépassé le 2026-10-29 par la v016-1 », onze fichiers :
  `git grep -cF`, et `git diff a47f08dc..HEAD -- _bmad-output | grep '^-'` ne doit retirer que la date ou la ligne de
  `sprint-status.yaml`) ; A6 (aucun changement au README, motif au Change Log).
- **Les phrases réécrites hors findings** (relecture des chapitres, C-v016-1-18) : chacune dit-elle vrai contre le code
  (`crates/kesh-api/src/mcp/reponse.rs`, `politique.rs`, `textes.rs`, `sortie.rs`) ? Une réécriture a-t-elle changé le
  sens d'une phrase qui était juste (par exemple « Kesh sert » au lieu de « l'agent lit » sous `contacts:read` et
  `documents:read` ; « l'appel rend une erreur » et « l'outil rend une erreur » dans `sec:mcp-journal` et
  `sec:mcp-erreurs` ; « Kesh répond à l'agent par le refus d'une clé inconnue » dans `sec:mcp-cles` ; « ce que Kesh lui
  répond » dans `sec:mcp-client`) ?
- **Les tests** : les assertions ajoutées peuvent-elles passer à vide ? Le doc-comment du test dit-il ce que le test
  vérifie ?
- **Les messages de commit** : `git log a47f08dc..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
  doit être vide.

## Axe 2 — la relecture **entière** des chapitres MCP des deux manuels FR, `.tex` et PDF aplati

`admin-manual.tex` : `\section{Le serveur MCP}` (`sec:mcp`) et toutes ses sous-sections jusqu'à « Après une
restauration » inclus, et `\subsubsection{Passer à la 0.16.1}` (`sec:maj-0-16-1`) ; `user-manual.tex` :
`\section{L'agent IA et le serveur MCP}` (`sec:agent-mcp`) en entier. **Chaque phrase** qui dit ce que Kesh rend ou ce
que l'agent reçoit, relit ou lit — partir du symptôme, non d'une liste :
`grep -nE "agent (re)?(reçoit|relit|relira|lit)|reçoi[tv]|parvien|atteign|atteint|transmet|apprend" docs/manual/fr/admin-manual.tex docs/manual/fr/user-manual.tex`,
puis les phrases voisines. Pour chacune :

1. Dit-elle ce que **Kesh rend**, cas par cas — clé dispensée : en-tête et données ; clé requise : l'accusé au dépôt,
   « en attente » à la relecture d'un envoi qui attend, deux blocs à la relecture validée (la phrase de Kesh, puis le
   texte validé) ; refus, échéance, numéro inconnu : une erreur, sans contenu ; erreurs d'outil — contre `reponse.rs`
   (`ReponseEnvoi::en_resultat`), `politique.rs` et `textes.rs` ?
2. Là où elle affirme une **réception par le client** (Claude Code, le modèle), porte-t-elle la réserve « à confirmer
   avec Claude Code sur la 0.16.1 » — sauf ce que la recette a vu, (a) à (h) ? Inversement, une réserve est-elle posée
   là où la recette **a** vu ?
3. Chaque « observé avec Claude Code 2.1.312 » s'appuie-t-il sur une observation (a) à (h), et seulement sur ce
   qu'elle dit ? Chaque « non éprouvé » restant est-il justifié ? L'intro de `sec:mcp-brancher` (« chaque geste dit
   s'il est observé ou « non éprouvé » ») est-elle vraie geste par geste ?
4. Le PDF aplati dit-il la même chose que le `.tex` (`grep -cF` des phrases réécrites) ? Comparer, au besoin, à
   `git show a47f08dc:docs/manual/fr/admin-manual.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`. Une ligne réécrite
   déborde-t-elle de la marge (page du paragraphe par `pdftotext -f <p> -l <p>`, relevés hors dépôt
   `/home/gcorbaz/devel/kesh-gate-logs/v0161-r2-bbox-{admin,user}-manual.txt`, à **lire** : « page, nombre de mots au-delà
   de x = 524 ») ?
5. Le manuel se contredit-il d'un paragraphe à l'autre (par exemple `sec:mcp-outils` contre `sec:mcp-envois` et les
   items 5 et 7 ; le manuel utilisateur contre celui d'administration ; le CHANGELOG `[0.16.1]` contre les deux) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p3.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`P3-1`, `P3-2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né de la remédiation P2** ou **antérieur** (un défaut antérieur se **signale**).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
