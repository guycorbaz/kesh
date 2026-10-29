# Prompt — revue de code P2 de la Story v016-1 (correctif 0.16.1 du serveur MCP), trois lentilles, Opus

Tu relis le développement de la **Story v016-1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
*le correctif 0.16.1 du serveur MCP* — **#811** : plus aucun `structuredContent` dans un `result` de `tools/call`
(`crates/kesh-api/src/mcp/politique.rs`, `reponse.rs`, `prompts.rs`, `sortie.rs`), `Donnees` réduite à un texte,
l'invariant de contrat **K23** (`tests/mcp_contrat.rs`), la garde lexicale `structured_content_nulle_part`
(`tests/mcp_gardes_lexicales.rs`), les tests qui lisaient la clé (aides de `tests/common/`, `mcp_*_e2e.rs`) et la spec
Playwright `frontend/tests/e2e/mcp-sends.spec.ts` ; **#812** : `Montant` sous une seule forme (`sortie.rs`), tests
`montant_forme_unique` et `bilan_montants_une_seule_forme` ; **les textes** : manuels d'administration et utilisateur
FR (`.tex` **et** PDF régénérés), `CHANGELOG.md` (section `[0.16.1] — Non publié`), `README.md` (l. 49), et des
annotations datées dans les relevés de l'Epic 32 (`_bmad-output/`). **Aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-0161`, branche `story/v016-1-mcp-sorties`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0161 diff d3b3b649...HEAD` (`d3b3b649` = `main` fusionnée avant le développement ;
la validation de la fiche s'est terminée à `a7df4c17`). Langue du rapport : français. Modèle : **Opus**. Passe
**complète** (deuxième passe de la revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les trois propriétés centrales.**
1. **Un seul canal.** Aucun `result` de `tools/call` ne porte d'autre clé que `content` et `isError` — sur **chaque**
   site (S1 à S9 de l'AC-1 de la fiche), réussi ou en erreur ; et chaque site réussi rend **dans son texte** ce qu'il
   doit rendre (données et en-tête, accusé et sa consigne, texte validé au second bloc). Un test qui ne verrait plus la
   clé parce qu'il ne la cherche plus est un défaut.
2. **Une seule forme de montant, sans perte.** `Montant` sort la valeur normalisée, portée à deux décimales si elle en
   a moins : aucune décimale réelle ne se perd (quantités et taux à quatre décimales compris), `-0` sort `0.00`.
3. **Le manuel ne retire un « non éprouvé » que sur une observation de la recette du 2026-10-28** (commentaire de
   recette de #723, observations (a) à (h), recopié ci-dessous), et n'en laisse aucun là où la recette a vu ; ce que la
   recette n'a pas pu voir — la réception d'un texte sans contenu structuré par Claude Code — reste **réservé**.

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

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v016-1-mcp-sorties.md` — les **AC-1 à AC-11**, *Sites de test à ajuster*,
*Tests et mutations prévus* (M1 à M15), *Ce que les gardes ne voient pas*, *À recontrôler à l'étape 0*, le **Dev Agent
Record** et le **Change Log** (les § *Ce que la release doit contenir* et *Renvoyé à la préparation de la release* ne
sont pas du développement : ne les juge pas) ; puis le registre `v016-1-choix-autonomes.md`, **C-v016-1-1 à 10** (fiche)
et **C-v016-1-11 et suivants** (choix du développement). Le Dev Agent Record déclare des rouges, des gates, des
mutations et un E2E : **ne les prends pas pour argent comptant**, recoupe-les avec les journaux hors dépôt (lecture
seule) : `/home/gcorbaz/devel/kesh-gate-logs/v0161-etape0-rouges.log`, `v0161-cible1.log`, `v0161-gate-*.log`,
`v0161-e2e-*.log`, `v0161-mutations/*.log` et `v0161-mutations/bilan.txt`, `v0161-mutations/muter.py` (à **lire**,
jamais à exécuter) ; pour la remédiation de la revue P1, `v0161-r1-*.log`, `v0161-r1-bbox-*.txt`,
`v0161-e2e-r1-*.log` et `v0161-mutations/R1-lt3.log`, `R1-lt4.log`.

**La remédiation de la revue P1 est elle-même un objet de cette passe** (axe 5 ci-dessous). Son diff :
`git -C /home/gcorbaz/devel/kesh-0161 diff fa1a804b...HEAD` — le commit de code `d2b65bc0` (manuels et PDF,
CHANGELOG, `sortie.rs`, `mcp_outils_e2e.rs`, annotations de la fiche et du registre), puis le commit du Change Log et de
ce prompt. Les findings de la P1 et leur sort sont à l'entrée « revue de code P1 » du Change Log de la fiche ; les
rapports P1 (`/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p1-{B,E,A}.md`) ne se lisent **que** pour l'axe 5, pour
juger si chaque remède répond à son finding — ne reprends pas leurs conclusions ailleurs sans les refaire.

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
  `muter.py`, `campagne.sh`, `v0161-annoter.py`, `v0161-e2e.sh`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni
  `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau** : la spécification `2025-11-25` se juge sur les
  phrases que la fiche et le registre citent ; un doute s'écrit « non vérifiable sans la page ».
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -cF` ou
  `grep -oF` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'Passer à la 0.16.1'`.
  ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) : `grep -cF "n’en envoie pas"`, non `n'en`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`CONTRAT_3211`, `PIEGE_*`, « Société vide v0161 »).

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le commentaire de recette et contre le code.** Pour chaque lieu du tableau
   de l'AC-8 (`sec:mcp-client`, intro de `sec:mcp-brancher`, items 3, 4, 5, 7 et 10, `sec:mcp-origin`,
   `sec:mcp-outils` « Ce que l'agent reçoit », `sec:mcp-envois` 3e puce, `\subsubsection{Passer à la 0.16.1}`) et pour
   le paragraphe *Invoquer un prompt* du manuel utilisateur : chaque « observé avec Claude Code 2.1.312 » s'appuie-t-il
   sur une observation (a) à (h) — et seulement sur ce qu'elle dit ? chaque « non éprouvé » restant est-il justifié ?
   la réserve des items 5 et 7 (« à confirmer… sur la 0.16.1 ») est-elle là ? Le PDF dit-il la même chose que le
   `.tex` (comparer à `git show d3b3b649:docs/manual/fr/admin-manual.pdf | pdftotext - - | …`) ? Chaque affirmation de
   *Passer à la 0.16.1* (aucune migration, retour à la 0.16.0, import croisé des sauvegardes, phrases de Kesh par cas)
   contre le code (`crates/kesh-db/migrations/`, `crates/kesh-api/src/main.rs`, `crates/kesh-db/src/version.rs`,
   `routes/admin.rs`, `mcp/textes.rs`) ; la forme des montants du manuel (« au moins deux décimales », `2.00`, `7.70`,
   `30.0`) contre `Montant` et `kesh_version`. Chaque `\ref` neuf a-t-il son `\label`, une seule fois ?
2. **L'invariant I et ses témoins.** Pour chaque site S1 à S9 : quel test le couvre (K23, `attendu_*` de
   `tests/common/mcp.rs`, `accuse` de `mcp_prompts_e2e.rs`, `echeance_ne_transmet_rien`, `mcp-sends.spec.ts`), compare-t-il
   les **clés exactes** du `result` (et non un champ), et son attendu est-il un littéral (K23 : règle de
   `mcp_contrat.rs:25-31`) ? Un cas peut-il passer **à vide** (numéro mal lu, boucle vide, `structure` dérivée qui
   avalerait une erreur) ? La garde `structured_content_nulle_part` : motif, témoins, angles morts écrits. Les mutations
   M1 à M15 sont-elles tuées **par le test nommé** (journal de campagne) et pour la raison visée ? Une mutation plausible
   non jouée survivrait-elle (par exemple une clé posée par `jsonrpc()` à côté de `result`, ou par un site neuf) ?
3. **Les tests existants gardent-ils leur sens.** `mcp_pieges::structure` dérivée du texte : les assertions qui
   comparaient le texte à la structure (`mcp_enveloppe_e2e.rs`, `mcp_outils_e2e.rs` « forme canonique indentée ») et
   K8 prouvent-elles encore quelque chose, ou sont-elles devenues tautologiques ? `une_seule_representation` sans
   `PIEGE_STRUCT` ; `numero_depose` au préfixe « Envoi n° » ; les tests de taille (`tailles_mesurees`,
   `plafond_au_maximum`) sous la nouvelle forme des montants.
4. **Grep du symptôme sur tout le worktree** : `git grep -nI -e structuredContent -e 'donnée structurée' -e 'contenu
   structuré' -- crates frontend/src frontend/tests docs README.md CHANGELOG.md website` ; chaque occurrence restante
   est-elle justifiée (commentaire qui dit l'absence, manuel, CHANGELOG) ? Et `git grep -n 'structure' --
   crates/kesh-api/src/mcp` : un reste de l'ancien champ ?
5. **La remédiation de la revue P1** (`git diff fa1a804b...HEAD`) — là où la sévérité se déplace d'ordinaire.
   - `admin-manual.tex`, `sec:mcp-outils` « Ce que l'agent reçoit » (B3 = E1) : la phrase dit-elle désormais ce que
     **Kesh** rend (`content` seul), et réserve-t-elle ce que Claude Code transmet au modèle, comme les items 5 et 7 ?
     Affirme-t-elle, en 0.16.0, plus que le constat de #811 ? Le PDF aplati porte-t-il la même phrase ? Une autre phrase
     (manuels, CHANGELOG, README, `website/`) affirme-t-elle encore la lecture côté client (`grep -nF 'préfère le
     contenu structuré'`, `grep -nF 'lit donc le même texte'`, `grep -nF 'structur'`) ?
   - `user-manual.tex`, *Invoquer un prompt* (B1) : la commande `/mcp__<serveur>__kesh-assistant` paraît-elle entière
     au PDF (`grep -cF`) ; l'environnement `center` coupe-t-il la phrase de façon fautive (le `\noindent\emph{(observé…)}`
     qui suit) ? `pdftotext -bbox` : un mot de la page du paragraphe dépasse-t-il la marge (`xMax` > 524) ?
   - `CHANGELOG.md` (E2 = A1, A2) : le titre de l'entrée *Corrigé* de #811 est-il vrai dans les deux issues de la
     recette ? la réserve ajoutée au paragraphe « Observé avec Claude Code 2.1.312 » dit-elle la même chose que l'item 5
     du manuel, sans en dire plus ?
   - `bilan_montants_une_seule_forme` (B2) : la seconde moitié (écriture de `10.0000`, relecture) prouve-t-elle qu'au
     moins une ligne est lue, et que chaque solde de ligne sort sous une forme unique ? Le choix des comptes (premier
     actif et premier passif postables et actifs) peut-il tomber sur un compte que le bilan ne liste pas, ou sur un
     compte d'une autre section que celle attendue ? L'acceptation de `"10.00"` **ou** `"-10.00"` affaiblit-elle le
     test au point d'en faire passer une forme fautive ?
   - `montant_forme_unique` (E3) : les cas `1.125` et `2.5000` ; la mutation `scale() < 3` (journal `R1-lt3.log`) et la
     mutation `scale() < 4` (`R1-lt4.log`) — la première est-elle **réellement équivalente** (que fait `rescale(2)` sur
     une échelle de 2 ? source de `rust_decimal-1.43.0`, `src/decimal.rs`, doc de `rescale`) ? la seconde est-elle tuée
     par le cas `1.125` et par lui seul ?
   - Doc-comments de `Montant` (A5 = B6), de `texte_et_valeur` et de `donnees` (B5 = E4) : disent-ils vrai, à la lettre,
     contre le code et contre la source de `rust_decimal` (la borne de 7,9·10^26) ? Un lien rustdoc mort ?
   - Annotations de la fiche et du registre (A4) : datées, sans réécriture d'un relevé passé
     (`git diff fa1a804b...HEAD -- _bmad-output | grep '^-'`) ; les choix C-v016-1-15 à 17 sont-ils justifiés ?
   - Les rejets de A3 et B4 (en-tête `Authorization: Bearer`, prémisse relue par l'orchestrateur sur la capture, que tu
     ne lis pas) : ne les rejuge pas ; vérifie seulement que les phrases visées n'ont pas changé.

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** (`politique.rs`, `reponse.rs`, `prompts.rs`, `sortie.rs`) : le comportement est-il
   exactement « `content` + `isError` » partout, les doc-comments disent-ils vrai, un reste (`valeur` calculée inutilement,
   un import mort) ? `Montant::serialize` : `normalize` puis `rescale(2)` si l'échelle est inférieure — une valeur à
   échelle 28, un très grand nombre, un négatif, un zéro négatif ?
2. **Les tests** : `k23_un_seul_canal` (la fermeture `decider`, les numéros lus, l'ordre des sites), la garde, les aides
   de `tests/common/`, `bilan_montants_une_seule_forme` (le montage « sans écriture » est-il vérifié ?), la spec
   Playwright (`unSeulCanal`, `premierTexte`, l'expression régulière de `deposer`).
3. **Les textes** : cohérence interne des manuels, du CHANGELOG (ordre *Modifié* avant *Corrigé*, forme
   `## [0.16.1] — Non publié`), du README ; LaTeX qui compile mal (`\_\_`, `<serveur>` en `\texttt`) ; aucun mot-clé de
   fermeture (`closes`, `fixes`, `resolves`) dans les messages de commit (`git log d3b3b649..HEAD --format=%B`).

## Lentille E — Edge cases

1. **Les sites** : `ReponseEnvoi::Depose` passe désormais par `un_bloc` — l'`isError` reste-t-il `false` ? `EnAttente`
   idem. S9 (`{"envoi": 0}`) et S8 (numéro inconnu) dans K23 : le numéro inconnu peut-il exister (`m + 1000`) ? S6 : la
   décision `refuse` sans `content`. Que lit Claude Code d'une erreur d'outil (`isError: true`) — rien n'a changé ?
2. **Les montants** : `Decimal::new(125, 4)` (`0.0125`), une quantité `2.5000`, un taux `7.7000`, `-0.0000`, une
   valeur de 28 décimales, la mantisse maximale : `rescale(2)` peut-il échouer ou arrondir ? Les tests de taille et le
   plafond `MCP_ENVOI_OCTETS_MAX` : un texte qui grossit (`0` → `0.00`) peut-il faire refuser un appel qui passait ?
3. **Le manuel** : une phrase qui affirme plus que l'observation (une commande « employée », un suffixe « (MCP) »
   absent, la mise en garde `nonFiable` « reçue ») ? une phrase devenue fausse ailleurs que dans les lieux de l'AC-8
   (`grep -nF 'non éprouv' docs/manual/fr/admin-manual.tex`, `grep -nF 'structur'`) ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-1 à AC-11**), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier : AC-1 (tableau S1 à S9 ; doc-comments réécrits, chaque occurrence de `structur` dans `src/mcp/`
justifiée au Dev Agent Record) ; AC-2 (`Donnees::nouvelles(texte)`, `Fixe.structure` retiré, `une_seule_representation`
annoté) ; AC-3 (l'invariant écrit tel quel en tête de K23, les marques, les littéraux) ; AC-4 (motif, trois témoins,
angles morts, message nommant #811 et C-v016-1-1) ; AC-5 (chaque ligne du tableau *Sites de test à ajuster*) ; AC-6 et
AC-7 (les huit cas de la fiche à `montant_forme_unique`, plus les deux de la remédiation P1, le témoin du zéro négatif, l'e2e rouge avant le correctif — journal
`v0161-etape0-rouges.log`) ; AC-8 (chaque ligne du tableau, et le manuel utilisateur) ; AC-9 (CHANGELOG : la phrase
*Corrigé* de #811 vraie avant la recette) ; AC-10 (README) ; AC-11 (`sprint-status.yaml`). Le § *À recontrôler à
l'étape 0* : chaque point est-il recontrôlé au Dev Agent Record, ligne lue citée ? Les annotations de la tâche 4 :
la liste triée du Dev Agent Record est-elle juste (une décision devenue fausse laissée sans annotation ? un relevé daté
réécrit au lieu d'annoté ? `git diff d3b3b649...HEAD -- _bmad-output | grep '^-'` ne doit rendre que des en-têtes de
diff) ?

Et : les choix C-v016-1-11 et suivants sont-ils justifiés, l'un contredit-il un AC, la fiche ou le registre ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son périmètre ? Les
**défauts observés** que le Dev Agent Record demande en issue sont-ils réels (prémisse lue au code) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p2-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p2-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0161-review-p2-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v016-1 ou **antérieur** (un défaut antérieur se **signale**, il ne se
   corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
