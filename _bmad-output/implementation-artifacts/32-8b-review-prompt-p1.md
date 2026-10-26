# Prompt — revue de code P1 de la Story 32-8b (la poignée côté serveur de la validation avant envoi), trois lentilles, Opus

Tu relis le développement de la **sous-partie b de la Story 32-8** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*sous une clé MCP `required`, la sortie d'un outil n'est plus refusée mais **déposée** dans la table des envois
(`mcp_sends`, 32-8a) ; un Administrateur la voit par trois routes d'administration (`/api/v1/admin/mcp/sends` : liste,
détail, décision) et la valide telle quelle, modifiée, ou la refuse ; l'agent la relit par l'outil hors registre
`kesh_lire_envoi`.* C'est **la partie de sécurité** de l'Epic 32 : sa propriété centrale est « **ce que l'utilisateur
voit est exactement ce qui part**, octet pour octet, et rien ne part sans lui » (fiche, § *La propriété centrale*).
Un défaut ici est une donnée des livres qui part vers un agent sans avoir été vue.

Worktree `/home/gcorbaz/devel/kesh-32-8a`, branche `story/32-8b`. Le diff à relire, **aplati** :

```sh
git -C /home/gcorbaz/devel/kesh-32-8a diff origin/main...HEAD -- crates
git -C /home/gcorbaz/devel/kesh-32-8a diff origin/main...HEAD --stat
```

Langue du rapport : français. Modèle : **Opus**. Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la 32-8b est en revue.** La table, son dépôt et la sauvegarde (32-8a), le prédicat des caractères cachés
(32-8b0) sont fusionnés et relus ; l'écran, les manuels et le CHANGELOG sont à la 32-8c ; les prompts à la 32-10a. Leur
absence n'est pas un finding. La 32-8b ne touche **aucun fichier de `kesh-db`** (cinq modules : `kesh-api/mcp`,
`kesh-api/services`, `kesh-api/routes` avec `lib.rs`, `kesh-api/audit_labels`, `kesh-i18n`) ; un défaut **du dépôt**
se signale, mais sa correction relève d'une autre story (point 2 de #765, C-32-8-56).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — § *La conception, en une page* et *La propriété
centrale*, la **Partie B** (AC-B1 à AC-B15), l'*inventaire clos des chemins* (D1 à D27), les tâches *32-8b*, les
mutations **MB1 à MB31** (MB9 retirée), le § *Gates* (32-8b), *Ce que les gardes ne voient pas*, les *Dev Notes* (flux,
signatures, textes), la *Note pour la 32-10a* (points 2, 2 bis, 3), le § *À recontrôler à l'étape 0* (note de
l'orchestrateur sur **24 576** octets, constats P3-R3, P3-R4, P3-R6, P3-F1, P3-F3 à P3-F7), puis le **Dev Agent Record**
et les entrées *32-8b* du **Change Log** — puis le registre `epic-32-choix-autonomes-328.md` (C-32-8-1 à C-32-8-29 pour
la conception, **C-32-8-50 à C-32-8-61** pour le développement), et l'issue **#765** telle que la décrit C-32-8-56 (tu
ne lances pas `gh`). Le Dev Agent Record déclare des gates, des mutations et un E2E : **ne les prends pas pour argent
comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/328b-*` (lecture seule). Aucun contrôle
visible ne change (l'écran est à la 32-8c) : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`),
  aucun `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande,
  `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) **copiée** dans
  le rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (`PIEGE_*_328`). Ne lis rien sous `~/kesh-test/`.

## Les deux axes obligatoires, pour chaque lentille

1. **Les chemins vers l'agent.** Énumère toi-même, depuis le code, **chaque** voie par laquelle un octet lu dans les
   livres ou dans `mcp_sends` peut sortir par `/mcp` — chaque bras de `repartiteur::traiter_appel`, de `relire`, de
   `reponse::rendre` et de `ReponseEnvoi::en_resultat` — et confronte ton énumération à l'inventaire D1 à D27. Une voie
   que l'inventaire ne nomme pas est un finding, même si elle te paraît fermée.
2. **Ce que l'écran montrera.** Les routes rendent ce que la 32-8c affichera et ce que l'Administrateur validera : le
   détail rend-il **la valeur stockée**, octet pour octet, celle-là même que `kesh_lire_envoi` relira après une
   validation tel quel ? Une valeur modifiée est-elle stockée **sans aucune transformation** (ni rognage, ni
   normalisation, ni fin de ligne) — par le serveur comme par la sérialisation JSON ?

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`mcp/politique.rs`** — `autoriser`, `appliquer`, `Sortie`, `EnvoiAValider`, les jetons : une donnée peut-elle
   sortir de `EnvoiAValider` autrement que par `deposer` ? `Donnees::avec_resume` ouvre-t-il un chemin du résumé vers
   l'agent ? `texte_a_valider` lit-il autre chose que le texte ?
2. **`mcp/repartiteur.rs`** — la résolution du nom réservé avant le registre ; la branche de dépôt et **le journal**
   (`Journal::DejaEcrit` contre `AEcrire`) : chaque `tools/call` a-t-il **exactement une** entrée, servi ou non, dépôt
   réussi, refusé ou en échec, relecture valide, inconnue, en erreur ? Le fail closed tient-il dans chaque bras ? La
   durée (`debut`) est-elle mesurée de la même façon dans le dépôt et hors dépôt ?
3. **`services/envois_mcp.rs`** — `rendre_visible` (format, idempotence, `\r`), l'ordre « rendre visible puis
   mesurer » (C-32-8-51), la purge *best-effort*, la transaction du dépôt et de son audit, `lire` et sa table d'états
   (`(Valide, None)`, `Echu`, `Refuse`) ; `ContenuValide` et `AccuseEnvoi` peuvent-ils se construire ailleurs ?
4. **`services/poignees.rs`** — les deux poignées : un pool peut-il en sortir ? L'audit du dépôt est-il dans **la même**
   transaction que l'`INSERT` ?
5. **`routes/mcp_envois.rs`** et `lib.rs` — les trois routes sont-elles **dans** le bloc `admin_routes`, au-dessus du
   marqueur ? La décision et son audit dans une transaction ? Le détail d'audit sans contenu ni résumé ? L'ordre des
   refus (`erreur_de_decision`) et les codes HTTP ; un refus porteur de contenu ; une version invalide ; un corps
   inconnu (`deny_unknown_fields`).
6. **Les tests** — un test peut-il passer **à vide** (un piège jamais posé, une boucle sur une liste vide, une
   comparaison à la constante elle-même) ? En particulier : `initialize_ping_prompts` compare-t-il les `instructions` à
   un **littéral** (MB25) ; `e1_rien_ne_part_sous_cle_requise` a-t-il son témoin ; les gardes lexicales neuves
   (`aucun_evenement_ne_nomme_le_contenu`, `aucun_span_dans_le_chemin_des_envois`, `envois_lus_par_leurs_lieux`,
   `envoi_valide_construit_par_le_service_seul`) ont-elles un témoin rouge **et** un vert, et leur tokeniseur
   (`squelette`) saute-t-il vraiment les littéraux, les caractères et les commentaires — essaie mentalement une chaîne
   brute à dièses, un `'"'`, un `/* … */` ?

## Lentille E — Edge cases

1. **Les bornes** : 24 576 octets **après** le rendu visible (un texte juste sous la borne qui la dépasse rendu
   visible), 1 024 pour le résumé, 20 envois en attente (souple : deux dépôts concurrents), 24 h d'échéance ; un texte
   modifié vide, d'exactement 24 576 octets, d'un octet de plus ; `envoi` = 0, `i64::MAX`, `i64::MAX + 1`, flottant,
   chaîne, champ inconnu.
2. **Les états** : relire un envoi en attente, validé, refusé, échu (en attente et validé), inexistant, d'une autre clé,
   d'une autre société, d'une **ligne incohérente** (la clé de l'appel, la société d'autrui) ; décider un envoi déjà
   décidé, échu, à mauvaise version, **à deux causes à la fois** (point 1 de #765) ; une ligne en attente **sans
   contenu** (point 3 de #765, C-32-8-55) ; deux décisions concurrentes.
3. **Les changements entre deux appels** : la politique de la clé changée entre dépôt et relecture (D25) ou pendant un
   appel (D24) ; la clé révoquée ; l'application redémarrée ; une remise à zéro simultanée d'une décision (P3-R6,
   C-32-8-58) — l'angle mort est-il juste au code ?
4. **Les erreurs** : `audit_log` indisponible au dépôt (rien ne doit rester dans `mcp_sends`), à la décision (l'envoi
   reste en attente), au journal d'un dépôt refusé (fail closed) ; la purge en échec ; une erreur de la base à la
   relecture. Ce qui part au client et ce qui va au journal du serveur dans chacun de ces cas.
5. **Les caractères** : `\r\n`, U+2028, U+0085, U+FEFF, U+202E dans le texte d'un outil (rendus visibles) et dans un
   texte modifié (refusés) ; une suite littérale `\u{202E}` déjà présente ; un emoji composé (`⚠️`) dans un libellé.

## Lentille A — Acceptance (la fiche contre le code)

1. **AC-B1 à AC-B15** : chacun est-il tenu **au code**, `grep -nF` à l'appui ? Les écarts déclarés — `Donnees` sans
   `BlocContenu` (C-32-8-50), le rendu visible au service (C-32-8-51), `entree_appel` `pub` (C-32-8-52), `pool_outils`
   (C-32-8-53), `AppelOutil` retiré (C-32-8-54) — sont-ils fondés, et leur conséquence est-elle écrite là où elle doit
   l'être (la *Note pour la 32-10a*) ?
2. **AC-B15** : la liste close des amendements du code de la 32-6 — le diff touche-t-il **autre chose** que ce qu'elle
   nomme (`git diff origin/main...HEAD --stat -- crates/kesh-api/src/mcp crates/kesh-api/src/services`) ? Un test de la
   32-6 a-t-il été affaibli plutôt qu'amendé (compare les deux versions de `cle_requise_*`,
   `e1_rien_ne_part_sous_cle_requise`, `tools_list_filtre`, `chaque_appel_est_journalise`,
   `journal_sans_arguments_ni_donnees`) ?
3. **#765** : les points (1), (3), (4) sont-ils tenus par un test **et** une mutation tuée (journal
   `kesh-gate-logs/328b-mutations.log`, `328b-mutations-resultats.json`) ? Le point (2) est-il honnêtement laissé
   (C-32-8-56) ?
4. **Les registres** : `ADMIN_COUPLES` (42), `(reached_layer, stopped_by_scope) = (12, 30)`, `discriminating = 10`,
   `audit_route_registry.rs` (130, 124, 105, 133), `FABRIQUES_V2`, `MOTIF_V2` et ses témoins, (vi) à treize fichiers :
   chaque nombre se recompte-t-il depuis la source, avec sa généalogie ? Le grep de la valeur
   (`grep -nE "\b(39|42|10|12|29|30|8)\b" crates/kesh-api/tests/admin_pat_denied_e2e.rs crates/kesh-api/src/lib.rs`)
   laisse-t-il un résidu ?
5. **Les libellés** : `mcp_send`, `mcp_send.approved`, `mcp_send.refused` dans `audit_labels.rs` et les quatre
   catalogues (`chaque_code_a_son_libelle_dans_les_quatre_catalogues`) ; les six messages `mcp-send-*` dans les quatre
   catalogues, replis égaux à la valeur fr-CH (garde G9 de `textes_coherents.rs`) ; le message de taille dit-il
   **24 576** ?
6. **Les mutations MB1 à MB31** (MB9 retirée), plus MF5 (P3-F5) et M765-1a, M765-1b, M765-3, M765-4 : chacune est-elle
   tuée **par le test que la fiche nomme** ? Celles jouées sous une forme différente de la fiche (MB6, MB8, MB11, MB12,
   MB31) le sont-elles sous une forme **équivalente en effet** ? Une mutation déclarée tuée l'est-elle **ailleurs** que
   par l'assertion visée ?
7. **Le Dev Agent Record et le Change Log** : chaque décompte (tests neufs, mutations, gates, E2E) se recompte-t-il
   depuis la source, avec son périmètre ? Les gates déclarés ont-ils tourné (journaux `328b-gate-*`, `328b-e2e*`) ? Les
   échecs E2E sont-ils exactement ceux de la KF-029 (`docs/testing.md` § *Les échecs attendus*) ?
8. **Le manuel** (axe obligatoire de `CLAUDE.md`) : la 32-8b ne l'écrit pas, mais des phrases peuvent être **devenues
   fausses** — la fiche cite la liste de la 32-7 et `README.md:233`, `website/roadmap.html:375`. Cherche dans le `.tex`
   **et** dans le PDF aplati ce qui décrit le comportement d'une clé `required` ou la validation avant envoi :
   `grep -niE "valid|requise|required|dispens" docs/manual/fr/admin-manual.tex docs/manual/fr/user-manual.tex`,
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(validation avant|clé requise|soumise à validation).{80}"`,
   et `grep -rnE "aucun outil|validation_requise|Refus" CHANGELOG.md README.md website docs`. Ce qui est faux doit être
   **nommé** (le corriger relève de la 32-8c ou de la 32-11).

## Le rapport

Un fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/328b-review-p1-{B,E,A}.md`. Chacun contient :

- **les axes exercés et ceux qui ne l'ont pas été**, nommément (un « 0 finding » sans cette liste ne compte pas) ;
- les findings, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le fichier et la ligne, la preuve
  (sortie `grep -nF` ou `sed -n` copiée), et la correction proposée ;
- le décompte final par sévérité.
