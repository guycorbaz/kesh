# Prompt — revue de code P3 de la Story 32-11 : passe ciblée, chasse aux régressions, Sonnet

Tu relis **la seule remédiation** de la revue de code P2 de la **Story 32-11** de Kesh (brancher un agent sur le serveur
MCP) : le commit **`ae17c0aa`**, dans le worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-11`. Le diff à
relire : `git -C /home/gcorbaz/devel/kesh-32-10a show ae17c0aa -- ':!*.pdf'` (et `git show --stat ae17c0aa`). Langue
du rapport : français. Modèle : **Sonnet**. Passe **ciblée** : **une seule lentille**, la chasse aux régressions —
*qu'est-ce que cette remédiation a cassé, affaibli, ou affirmé à tort ?* Contexte frais : ne t'appuie sur aucune passe
antérieure, hormis ce qui suit.

**Ce que la remédiation dit avoir fait** (Change Log de la fiche `32-11-brancher-un-agent.md`, entrée « revue de code
P2 », et registre `epic-32-choix-autonomes-3211.md`, **C-32-11-41 à 43**) — à vérifier, non à croire :
1. `crates/kesh-api/tests/mcp_contrat.rs` — `enveloppe_conforme` prend un `IdAttendu` : `Lu(n)` (présent et égal,
   `CLAUSE`) ou `SansId(Fondement)` ; K15 (`OrigineRefusee`), K19 (`RequeteInvalide`), K21 (`EntreeNonAcceptee`) gardent
   une `CLAUSE` « `id` absent ou `null` » et une `DECISION_KESH` « absent » ; K16 (`RefusAvantLeCorps`) n'a plus que la
   `DECISION_KESH`, la tension avec BASIC *Error Responses* écrite en commentaire ; K21 cite le point 4 de *transports*
   en entier ; K8 vérifie `structuredContent.version` contre `env!("CARGO_PKG_VERSION")` et que le texte porte le JSON de
   la sortie.
2. `docs/manual/fr/admin-manual.tex` (et son PDF) — le chapeau de `\section{Le serveur MCP}` (ce qu'un appel écrit) ;
   `sec:mcp-fonctionnement` (« Un seul outil écrit autre chose que la trace de l'appel et l'envoi qu'il dépose ») ;
   `sec:mcp-outils` (même formule) ; `sec:mcp-envois` (le plafond de 20 en attente par clé, un envoi en attente pouvant
   sortir des 200).
3. Mutations MR1 à MR3 (`/home/gcorbaz/devel/kesh-gate-logs/3211/mutations/campagne-p2.log`,
   `campagne-p2-messages.log`, `mutations.py` — à **lire**, jamais à exécuter) ; gate ciblé `gate-cible-p2.log` ; gate
   complet `gate-complet-p2.log`.

## Ce que tu cherches, en priorité

- **Une `CLAUSE` qui n'en est pas une**, ou une marque fausse **pour un appelant précis** : pour K15, K16, K19, K21, la
  phrase citée au `Fondement` couvre-t-elle **ce** cas (requête bien formée ou non, `id` lisible ou non) ? Une phrase
  citée existe-t-elle mot pour mot sur la page, en MUST pour ce qui est asserté ?
- **Une assertion affaiblie ou passée à vide** : un appelant oublié (`grep -nF 'enveloppe_conforme(' mcp_contrat.rs`) ;
  un `match` qui laisse un fondement tomber dans la mauvaise branche ; K8 — `split_once("\n\n")` et la comparaison JSON
  peuvent-ils réussir sans rien prouver (un texte sans JSON, un `ok()` qui rend `None` des deux côtés) ? `env!` du
  crate de test est-il bien une valeur indépendante du serveur, ou la même source ?
- **Un texte du manuel devenu faux**, contre le code : que recouvre « la trace » d'un appel (`services/serveur_mcp`,
  `cles_mcp` : dernière utilisation ; journal d'audit ; dépôt d'un envoi, `envois_mcp::deposer`) ? Un `prompts/get`
  écrit-il aussi (journal) — la formule le couvre-t-elle ? `proposer_prompt` écrit-il aussi sa trace ? Le plafond
  (`MCP_ENVOIS_EN_ATTENTE_MAX`, `services/envois_mcp.rs`) vaut-il bien par clé (`kesh-db/src/repositories/mcp_sends.rs`) ?
  Le PDF dit-il la même chose que le `.tex` ?
- **La propagation** : le symptôme (« seul `proposer_prompt` écrit », « un seul outil qui écrit », « décidé ou échu
  au-delà ») subsiste-t-il ailleurs — les deux manuels, la brochure, `README.md`, `website/*.html`,
  `docs/api-external.md`, le CHANGELOG, la fiche ?
- **Les comptes rendus** : chaque nombre de l'entrée « revue de code P2 » du Change Log (trend 0/0/1/4, 110/110,
  4456/4456, MR1 à MR3) se recompte-t-il depuis sa source (journaux ci-dessus) ?

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
  Exemple : `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'le plafond de 20 envois en attente valant par clé'`
  (l'apostrophe du PDF est typographique : `’`).
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée.
- **Données réelles** : aucune ; ne lis rien sous `~/kesh-test/`.

## Rapport

Écris ton rapport dans **un seul** fichier, `/home/gcorbaz/devel/kesh-gate-logs/3211-review-p3.md`, par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`R1`, `R2`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie copiée ; phrase de la spécification citée telle quelle, URL), la correction proposée, et
   s'il est **né de la remédiation `ae17c0aa`** ou **antérieur**.
2. **Le relevé des `CLAUSE` touchées par `ae17c0aa`** : pour chacune, la phrase de la page lue (ou « non vérifiable :
   page non lue ») et ton verdict, **appelant par appelant** pour la branche `SansId`.
3. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
4. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
5. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
