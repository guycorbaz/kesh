# Prompt — revue de code P1 de la Story 32-11 (brancher un agent, de bout en bout), trois lentilles, Sonnet

Tu relis le développement de la **Story 32-11** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) :
*le **test de contrat** du serveur MCP contre la révision `2025-11-25` du protocole (`crates/kesh-api/tests/mcp_contrat.rs`,
cas K1 à K22, K4 retiré) ; la garde du compte des routes (`le_manuel_dit_le_compte_du_registre`,
`tests/audit_route_registry.rs`) ; au manuel d'administration, `\subsection{Comment fonctionne le serveur MCP}`
(`sec:mcp-fonctionnement`) et `\subsection{Brancher un agent, pas à pas}` (`sec:mcp-brancher`), écrites depuis un
**parcours joué** contre le serveur en client HTTP, puis **rejoué depuis le PDF** ; `sec:mcp-client` amendée ; deux
limites ajoutées à `sec:mcp-envois` (#768, #773) ; le renvoi de `sec:api-keys` ; au manuel utilisateur,
`\section{L'agent IA et le serveur MCP}` (`sec:agent-mcp`) ; la brochure ; `docs/api-external.md` § 6.4 et § 9 ;
`README.md` (ligne v0.16.0) ; `website/roadmap.html` (l. 43, 223, 375) ; le CHANGELOG.* **Aucun code de production,
aucune migration** : la story décrit et éprouve le code des stories 32-5 à 32-10b, fusionnées.

Worktree `/home/gcorbaz/devel/kesh-32-10a`, branche `story/32-11`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-10a diff cee46531...HEAD` (`cee46531` = `main` fusionnée avant le développement).
Langue du rapport : français. Modèle : **Sonnet**. Passe **complète** (première passe d'une revue de code). Contexte
frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les deux propriétés centrales.**
1. **L'oracle du contrat n'est jamais la sortie de Kesh.** Chaque attendu de `mcp_contrat.rs` est un **littéral** écrit
   dans le test, ou une clause de la spécification citée ; aucun n'est tiré de `kesh_api::mcp::textes`, des aides
   `attendu_*` de `tests/common/mcp.rs`, ni d'une réponse capturée. Chaque assertion porte la marque `CLAUSE` (une phrase
   **MUST** citée, ou une clé requise du schéma officiel) ou `DECISION_KESH` ; rien de douteux n'est écrit `CLAUSE`.
2. **Le manuel dit ce que le serveur et les écrans font.** Le manuel est la seule preuve durable du parcours : chaque
   geste de `sec:mcp-brancher` doit se retrouver au **journal d'exécution**, chaque phrase de `sec:mcp-fonctionnement`
   et de `sec:agent-mcp` **au code**. Aucune phrase ne promet que l'agent **écrit, modifie ou crée** dans les livres.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-11-brancher-un-agent.md` — les **AC** (groupes A à E et T), *Tests et
mutations prévus* (M1 à M22, MT1, MT2), *Ce que les gardes ne voient pas*, les *Dev Notes*, le § *Réponses de Guy*
(il **prime**), le § *À recontrôler à l'étape 0*, le **Dev Agent Record** et le **Change Log** ; puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-3211.md`, **C-32-11-30 à 36** (choix du développement).
Le Dev Agent Record déclare un parcours, une restauration, un rejeu, des gates, des mutations et un E2E : **ne les prends
pas pour argent comptant**, recoupe-les avec les pièces hors dépôt (lecture seule) :

- `/home/gcorbaz/devel/kesh-gate-logs/3211/parcours.log`, `restauration.log`, `t8/parcours-t8.log` (journaux) ;
- `…/3211/requetes/`, `…/3211/reponses/` (corps et en-têtes, R01 à R09, E01 à E04), `…/3211/t8/requetes/`, `t8/reponses/` ;
- `…/3211/envois.log`, `…/3211/t8/envois.log` (une ligne d'envoi et une de retour par requête) ;
- `…/3211/captures/`, `…/3211/captures-r/`, `…/3211/t8/captures/` (thème clair ; ouvre-les, lecture d'image) ;
- `…/3211/mutations/campagne*.log` et `mutations.py` (à **lire**, jamais à exécuter) ; les journaux de gate
  `…/3211/gate-*.log`, `…/3211/e2e-*.log`.

⛔ **Les fichiers `…/3211/cle-parcours.entete`, `…/3211/amorcage/cle-amorcage.entete` et `…/3211/t8/*.entete`
portent des secrets** (clés de démonstration révoquées) : **ne les ouvre pas**, ne les cite pas. Ni les
`export/*.keshbackup`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs` (dont `pw.cjs`, `envoi.sh`,
  `mutations.py`) ; ni `make`, ni `latexmk`, ni `cargo`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni **`curl`**,
  ni **`claude`** sous aucune forme, ni `socat`, ni **`sed -i`**, ni `playwright`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau** : les clauses citées au test se vérifient
  contre les phrases que le test et la fiche citent, et un doute s'écrit « non vérifiable sans la page ».
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`, la lecture d'images,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** (deux `grep` de
  ce genre ont pris 1,7 Go chacun le 2026-10-27) : `grep -cF` ou `grep -oF` sur des **motifs fixes**, sous `timeout 60`.
  Exemple : `timeout 60 pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -cF 'Brancher un agent, pas à pas'`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures et les montants (17 345.10, 4 321.55, 1 234.60) sont synthétiques. Ne
  lis rien sous `~/kesh-test/`.

## Les axes obligatoires, pour chaque lentille

1. **Le manuel (`.tex` et PDF aplati) contre le journal d'exécution et contre le code.** Pour chaque geste de
   `sec:mcp-brancher` (dix gestes numérotés et *Après une restauration*) : la ligne du journal (`parcours.log`,
   `restauration.log`, `t8/parcours-t8.log`) qui le porte, libellés compris ; ce qui est marqué « non éprouvé » l'est-il
   bien, et rien d'éprouvé n'y est-il marqué ainsi, ni l'inverse ? Pour chaque phrase de `sec:mcp-fonctionnement` (les
   douze étapes contre `crates/kesh-api/src/mcp/mod.rs`, `transport.rs`, `repartiteur.rs`, `reponse.rs` ; les garanties
   contre `kesh-db/src/pool.rs` `pool_lecture_seule`, `politique.rs`, `registre.rs`, le service des prompts) et de
   `sec:agent-mcp` : le code le dit-il ? Les ajouts de `sec:mcp-envois` (« les 200 envois les plus récents »,
   `LISTE_MAX` ; la décision annoncée en échec, #768) sont-ils vrais ? Le PDF dit-il la même chose que le `.tex` ?
   Chaque `\ref` a-t-il son `\label`, une seule fois (`grep -nE 'sec:mcp[a-z-]*'`) ? Aucune seconde copie de la matrice
   des portées, de la configuration du client ni de l'écran des envois ? Aucun « Hermes » ?
2. **L'oracle du contrat.** `grep -nE 'attendu_|\btextes\b' crates/kesh-api/tests/mcp_contrat.rs` (lignes de code ?) ;
   pour chaque cas K : l'attendu est-il un littéral, la marque est-elle juste (une phrase en MUST pour `CLAUSE`, sinon
   `DECISION_KESH`), la phrase citée est-elle plausible et entière ? Un cas peut-il passer **à vide** (une boucle sur
   une liste vide, un `any` sur rien, un témoin manquant — K14, K9) ? Les mutations M1 à M22 sont-elles tuées **par le
   cas nommé** (journal de campagne) et pour la raison visée ? Une mutation plausible non jouée survivrait-elle ?
3. **Aucune promesse d'écriture.** `grep -n -i -E 'MCP|agent' README.md website/*.html docs/api-external.md` et le PDF
   aplati des trois manuels : une phrase dit-elle que l'agent écrit, modifie ou crée dans les livres ? « lecture seule »
   sans réserve là où `proposer_prompt` écrit une version de prompt ?
4. **Aucune donnée réelle, aucun secret** dans un fichier versionné du diff (`git diff cee46531...HEAD | grep -nE
   'kesh_(mcp|pat)_[0-9A-Za-z]{20}'` — motif sur le **diff**, non sur un PDF).

## Lentille B — Blind (le diff seul, sans la fiche)

1. **`mcp_contrat.rs`** : `banc`, `en_tetes`, `post`/`post_avec`/`methode`, `sans_session` (appelé sur chaque `Rendu` ?),
   `enveloppe_conforme` (branche `Some`/`None`, XOR, codes), `resultat_d_outil` ; chaque cas K. Un `assert!` qui ne
   peut pas échouer ? Une requête dont le statut n'est jamais vérifié ?
2. **`audit_route_registry.rs`** : `le_manuel_dit_le_compte_du_registre` — normalisation, regex, « exactement une
   fois », le message d'échec ; peut-il passer à vide ?
3. **Les textes** : cohérence interne des manuels, de `api-external.md`, du README, du site, du CHANGELOG ; LaTeX qui
   compile mal (guillemets, deux-points, `\emph` coupé autour de `\keshcommand`).

## Lentille E — Edge cases

1. **Le contrat** : K21 (202 **ou** 4xx), K18 (version absente), K12 (corps pour corps), K10 (la décision par la route,
   `version` lue au détail), K14 (amorçage des standards par le test — `amorcer_standards`), K22 (1 Mio + 1 : le corps
   n'est pas du JSON — est-ce bien la borne qui répond, et non le JSON ?). Que se passe-t-il si une révision ajoute une
   clé à `InitializeResult`, `Tool`, `Prompt` : quel cas rougit, sous quelle marque ?
2. **Le parcours** : une requête envoyée deux fois (`envois.log`) ? le témoin positif de l'AC-A5 (b) a-t-il pu passer à
   vide (motif `20432\.05` contre le format `"20432.0500"`) ? l'AC-A5 (c) (`jq -j`, `cmp`) compare-t-il le bon bloc ?
   la restauration (E01 à E04) a-t-elle employé **la même clé**, prise **avant** la révocation ?
3. **La garde du compte** : MT1 et MT2 tuées ? Une phrase coupée sur deux lignes, un `\textbf` au milieu, un second
   « N + N + N = N » ailleurs dans le manuel ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC des groupes **A** (A1 à A8), **B** (B1 à B6), **C** (C1 à C9), **D** (D1 à D5), **E** (E1 à E3) et **T**
(T1, T2), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve, **en tenant compte des
réponses de Guy** (pas de `claude` ; clauses à la main ; pas de pièges ; un défaut ne se corrige pas ici ;
`README.md` v0.15.0 et `roadmap.html:43` hors du « Kesh now keeps real books » laissés à la release) et de l'étape 0
(Dev Agent Record). En particulier : AC-A5 (b) et (c) (sorties citées), AC-A6 (le rejeu suit-il le PDF ? les écarts
écrits ont-ils corrigé le manuel ?), AC-B2 (rangement, littéraux), AC-C4 (« déjà vraie » : `129 des 135` et
`129 + 4 + 2 = 135` contre la partition), AC-C9 (chaque motif : 0 à la base `origin/main`, 1 au PDF — recompte par
`git show cee46531:docs/manual/fr/<m>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' ' | grep -cF '<motif>'`), AC-D3 (les
trois lignes du site), AC-T2 (les gardes inchangées : `git diff cee46531...HEAD -- crates/kesh-api/tests/textes_coherents.rs
crates/kesh-api/tests/configuration_transmise.rs crates/kesh-api/tests/mcp_gardes_lexicales.rs` vide ?). Aucun code de
production : `git diff --stat cee46531...HEAD -- crates/ ':!crates/kesh-api/tests'` vide ?

Et : les choix C-32-11-30 à 36 sont-ils justifiés, l'un contredit-il un AC, la fiche ou une réponse de Guy ? Chaque
nombre du Dev Agent Record (cas, tests, mutations, gates, E2E, requêtes) se recompte-t-il depuis sa source, avec son
périmètre ? Les **défauts observés** que le Dev Agent Record demande en issue sont-ils réels (prémisse lue au code) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/3211-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/3211-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/3211-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la 32-11 ou **antérieur** (un défaut du produit que le parcours révèle se
   **signale**, il ne se corrige pas dans la 32-11 — réponse 4 de Guy).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
