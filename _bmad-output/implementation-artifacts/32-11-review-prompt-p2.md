# Prompt — revue de code P2 de la Story 32-11 : passe ciblée, chasse aux régressions, Opus

Tu relis **la seule remédiation** de la revue de code P1 de la **Story 32-11** de Kesh (brancher un agent sur le serveur
MCP) : le commit **`06c3c104`**, dans le worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-11`. Le diff à
relire : `git -C /home/gcorbaz/devel/kesh-32-10a show 06c3c104 -- ':!*.pdf'` (et `git show --stat 06c3c104`). Langue
du rapport : français. Modèle : **Opus**. Passe **ciblée** : **une seule lentille**, la chasse aux régressions —
*qu'est-ce que cette remédiation a cassé, affaibli, ou affirmé à tort ?* Contexte frais : ne t'appuie sur aucune passe
antérieure, hormis ce qui suit.

**Ce que la remédiation dit avoir fait** (Change Log de la fiche, entrée « revue de code P1 », et registre
`epic-32-choix-autonomes-3211.md`, **C-32-11-37 à 40**) — à vérifier, non à croire :
1. `crates/kesh-api/tests/mcp_contrat.rs` — `enveloppe_conforme` : une assertion `CLAUSE` sur le `Content-Type` de la
   réponse à une requête et une `DECISION_KESH` « exactement `application/json` » ; la branche `None` (erreur sans `id`
   lisible) accepte sous `CLAUSE` un `id` absent **ou** `null`, et exige l'absence sous `DECISION_KESH` ; les marques
   `TEMOIN` et `MONTAGE` ; le statut 200 en `DECISION_KESH` (constante `STATUT_200`) ; K8 vérifie le contenu de
   `kesh_version` ; K11 à l'`id` 11 ; K13 (`-32601`) passé sous `DECISION_KESH` ; K21 : la `CLAUSE` admet 4xx ou 5xx.
2. `docs/manual/fr/admin-manual.tex` (et son PDF) — la phrase d'ouverture de `\section{Le serveur MCP}` (« en lecture
   seule des livres … seul `proposer_prompt` écrit une version de prompt à valider ») ; `sec:mcp-envois` (« au plus 200
   envois … d'abord ceux qui attendent, par échéance, puis les autres, des plus récents aux plus anciens ») ;
   `sec:mcp-brancher` (chapeau, avis de création de clé cité en entier, relecture avant décision « non éprouvé »,
   paragraphe *Après une restauration*).
3. `docs/manual/fr/user-manual.tex` (et son PDF) — *Reconnaître l'activité de l'agent*.
4. Mutations MR1 et MR2 (`/home/gcorbaz/devel/kesh-gate-logs/3211/mutations/campagne-p1.log`, `mutations.py` — à
   **lire**, jamais à exécuter) ; gate complet `kesh-gate-logs/3211/gate-complet-p1.log`.

## Ce que tu cherches, en priorité

- **Une `CLAUSE` qui n'en est pas une.** Pour **chaque** assertion marquée `CLAUSE` que le commit ajoute ou modifie, et
  pour chaque phrase citée en commentaire : la phrase existe-t-elle **mot pour mot** sur la page, porte-t-elle bien
  **MUST** pour ce qui est asserté, et l'assertion n'exige-t-elle rien de plus que la phrase ? (Ex. : la clause du
  `Content-Type` vaut-elle pour une **erreur** du transport, ou seulement pour la réponse à une requête ? « an HTTP
  error status code » couvre-t-il 5xx ?)
- **Une assertion affaiblie.** Une `CLAUSE` élargie (absent **ou** `null` ; 4xx **ou** 5xx) laisse-t-elle encore une
  `DECISION_KESH` qui tienne la valeur exacte ? Un cas a-t-il perdu sa capacité à rougir (une mutation de la campagne
  P1 qui ne tuerait plus) ? K8 peut-il passer à vide ?
- **Un texte du manuel devenu faux ou plus faible**, contre le code (`kesh-db/src/repositories/mcp_sends.rs`, l'ordre de
  la liste ; `mcp/repartiteur.rs`, `proposer_prompt` ; `services` du journal) et contre les journaux du parcours
  (`kesh-gate-logs/3211/parcours.log`, `restauration.log`, `t8/parcours-t8.log`) ; une marque « non éprouvé » posée sur
  un geste éprouvé, ou l'inverse ; le PDF dit-il la même chose que le `.tex` ?
- **La propagation** : le symptôme corrigé (« lecture seule » sans réserve ; « les 200 plus récents » ; « chaque appel
  de l'agent ») subsiste-t-il ailleurs — les deux manuels, la brochure, `README.md`, `website/*.html`,
  `docs/api-external.md`, le CHANGELOG, la fiche ?
- **Les comptes rendus** : chaque nombre de l'entrée « revue de code P1 » du Change Log (trend, 110/110, 4456/4456,
  MR1/MR2) se recompte-t-il depuis sa source ?

## Autorisation de lecture réseau (et elle seule)

Tu **peux** lire, en lecture seule, les pages de la spécification MCP **révision `2025-11-25`** sur
`modelcontextprotocol.io` (par exemple `https://modelcontextprotocol.io/specification/2025-11-25/basic`,
`…/basic/lifecycle`, `…/basic/transports`, `…/basic/utilities/ping`, `…/server/tools`, `…/server/prompts`,
`…/basic/authorization`, `…/schema`) et `https://www.jsonrpc.org/specification`, **pour vérifier chaque phrase citée
comme `CLAUSE`** dans `mcp_contrat.rs`. Cite au rapport la phrase lue telle quelle ; si une page ne s'ouvre pas, écris
« non vérifiable : page non lue », jamais une clause de mémoire. Aucun autre accès réseau.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`, `> /dev/null`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se
  **déclare** au rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs` (dont `pw.cjs`, `envoi.sh`,
  `e2e.sh`, `mutations.py`) ; ni `make`, ni `latexmk`, ni `cargo`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni
  **`curl`**, ni **`claude`** sous aucune forme, ni `socat`, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture.
- ⛔ **Les fichiers `*.entete` et `export/*.keshbackup` de `kesh-gate-logs/3211/`** (et de `t8/`) portent des secrets :
  ne les ouvre pas.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`, la lecture d'images,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et la lecture des pages de la spécification ci-dessus.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** (deux `grep` de ce
  genre ont pris 1,7 Go chacun le 2026-10-27) : `grep -cF` ou `grep -oF` sur des **motifs fixes**, sous `timeout 60`.
  Exemple : `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'en lecture seule des livres'`
  (l'apostrophe du PDF est typographique : `’`).
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; ne lis rien sous `~/kesh-test/`.

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/3211-review-p2.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie copiée ; phrase de la spécification citée telle quelle, URL), la correction proposée, et
   s'il est **né de la remédiation `06c3c104`** ou **antérieur**.
2. **Le relevé des `CLAUSE`** : pour chaque assertion `CLAUSE` du fichier, la phrase de la page lue (ou « non
   vérifiable : page non lue »), et ton verdict (fondée / plus stricte que la phrase / sans phrase en MUST).
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
