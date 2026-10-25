# Prompt — revue de code P1 de la Story 32-6b (le journal d'audit des appels MCP), trois lentilles, Sonnet

Tu relis le développement de la **partie b de la Story 32-6** de Kesh (comptabilité suisse, Rust/Axum, MariaDB) :
*le journal d'audit de chaque appel d'outil du serveur MCP — une entrée `mcp.tool_called` par `tools/call`, au nom du
créateur de la clé, au détail fermé `{cleMcpId, outil, issue, dureeMs, validation}`, **sans arguments ni données**,
écrite sur la `Trace` avant l'envoi de la réponse et **fail closed** (si elle échoue, aucune donnée ne part) ; écrite
dans une **tâche de l'appel** qui englobe la tâche de l'outil (C-32-6-58), si bien qu'un appel abandonné par le client
est journalisé quand même.* C'est une partie de **la story de sécurité de l'epic** : le journal est la seule trace de
ce qu'un agent a lu dans les livres réels de Guy.

Worktree `/home/gcorbaz/devel/kesh-32-6`, branche `story/32-6b`, partie de `7e47bff4` (`main`, la 32-6a fusionnée).
Le diff à relire, **aplati** (un seul diff, pas la suite des commits) :
`git -C /home/gcorbaz/devel/kesh-32-6 diff origin/main...HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Seule la partie b est en revue.** Le point d'entrée et ses gardes (32-6a) sont fusionnés et revus ; un défaut que
la 32-6b ne touche pas est un finding **antérieur**, à classer comme tel. L'écran et la section *Le serveur MCP* du
manuel sont la 32-7 ; la validation avant envoi (la poignée, la table des envois) est la 32-8 ; les outils de lecture
des livres sont la 32-9 ; la synchronisation des documents de release est la 32-11.

La branche porte aussi, **sans élargir le code**, des corrections demandées par l'orchestrateur : les marges de deux
tests d'abandon (`mcp_transport_e2e.rs`, issue #759, LOW R1 de la revue P3 de la 32-6a) et quatre annotations datées
de la fiche (`0.0.0.0/0` = tout client **IPv4** ; la garde « 2 bis » écartée par Guy ; le « repli égal à la valeur
fr-CH » de l'AC-b5, issue #758 ; N26).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-6-endpoint-mcp.md` — *Partie B* (AC-b1 à AC-b6), *L'inventaire clos*
(N26 surtout), *Ce que les gardes ne voient pas* (le panic ou l'abandon), *Tests et mutations prévus* (Mb1 à Mb6),
*Gates*, et le **Dev Agent Record** de la partie b (étape 0, mutations, gates, décomptes) — puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-326.md` (C-32-6-21, 24, 30, 56, et les choix du
développement, **C-32-6-58 à C-32-6-61**). Le Dev Agent Record déclare des gates, des mutations et un E2E : **ne les
prends pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/326b-*` (lecture
seule). Aucun contrôle visible ne change : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe), lis les PDF vers la sortie standard seulement. Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` (dont `326b-reset-db.sh`, `326b-e2e.sh`) ou d'un scratchpad (dont
  `mutations_326b.py`) ; ni `make`, ni `latexmk`, ni `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`,
  ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Les deux axes obligatoires, pour chaque lentille

1. **Chaque garde touchée, son motif greppé sur le dépôt réel.** La 32-6b touche ou s'appuie sur : la garde des
   libellés (`tests/audit_label_registry.rs` — extraction du littéral d'action au site `NewAuditLogEntry::user(`), le
   registre des routes (`tests/audit_route_registry.rs`, `/mcp` passé à `Traced`, compteurs), les gardes lexicales du
   MCP (`tests/mcp_gardes_lexicales.rs` : `executer_appele_par_le_repartiteur_seul`,
   `contexte_mcp_construit_par_le_repartiteur_seul`, `mcp_sans_pool`, `reponses_par_rendre_seul`) et celles de
   `tests/services_motif.rs` (v-1, v-2, vi). Pour chacune, dis si le nouveau code y entre par la porte prévue ou passe
   à côté (un appel de `trace.pool()` hors des fonctions d'audit ; une écriture neuve non inscrite à l'inventaire clos
   de `services/mod.rs` ; un `AppError` nommé dans `src/mcp/`). Copie les sorties `grep -nF`.
2. **Chaque dépendance lue à la source.** Toute affirmation du code, de ses commentaires, du registre ou du Dev Agent
   Record sur le comportement de `tokio` (`spawn` : une tâche détachée n'est pas annulée quand son `JoinHandle` est
   abandonné ; `JoinError` d'un panic ; `time::timeout` qui abandonne le futur qu'il enveloppe), d'hyper 1.10.1
   (abandon du futur du service à la coupure du client) ou de `sqlx-core` 0.8.6 (une transaction abandonnée est
   annulée) se vérifie dans `~/.cargo/registry/src/*/<crate>-<version>/` (version lue dans `Cargo.lock`, ligne citée).

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-api/src/mcp/repartiteur.rs`** — `appeler`, `appeler_et_journaliser`, `traiter_appel`. Une donnée
   d'outil (`ReponseMcp::Outil`) peut-elle être rendue **sans** que `journaliser_appel` ait rendu `Ok` ? Un chemin de
   `tools/call` évite-t-il le journal (retour anticipé, `?`, panic, branche de `repartir` qui ne passe pas par
   `appeler`) ? Un appel peut-il écrire **deux** entrées ? L'issue écrite peut-elle différer de la réponse rendue
   (délai, panic, refus de service) ? Le nom écrit au journal peut-il venir du client ? Le passage de `params` non objet
   à `Value::Null` pour `tools/call` change-t-il la réponse d'une autre méthode ?
2. **`crates/kesh-api/src/services/serveur_mcp.rs`** — `journaliser_appel` : transaction, `actor_type`, `company_id`
   et `actor_label` (sous-SELECT de `audit_log::insert_in_tx`), une erreur réellement **rendue** (jamais avalée),
   aucune donnée ni argument possibles dans `AppelOutil` ; `IssueAppel` fermée et ses codes ceux de l'AC-b1.
3. **Le coût de la tâche de l'appel** — un `EtatMcp` cloné par appel (service statique compris) : défaut réel ou
   négligeable ? Une tâche de l'appel qui attend un permis du pool des outils indéfiniment : qu'est-ce qui la borne ?
   Une tâche détachée qui écrit au journal **après** la fin d'un test ou l'arrêt du serveur ?
4. **Les tests** (`mcp_journal_e2e.rs`, les deux tests d'abandon de `mcp_transport_e2e.rs`, les outils ajoutés à
   `tests/common/mcp.rs`) — un test peut-il passer **à vide** : une « ligne ajoutée » comptée sur la mauvaise table, un
   détail comparé après qu'on en a retiré ce qu'il fallait contrôler, un témoin qui ne prouve pas ce qu'il annonce, un
   `appel_abandonne_journalise` qui passerait sans la tâche de l'appel parce que la requête n'est pas réellement
   abandonnée, une égalité du corps sous journal indisponible qui vaudrait aussi avec le journal disponible ? Les
   marges élargies (#759) gardent-elles le sens des deux tests d'abandon — la mutation « attente directe du futur »
   les fait-elle toujours rougir, et sinon, est-ce écrit ?

## Lentille E — Edge cases

1. **Chaque issue de l'AC-b1** (`servi`, `validation_requise`, `inconnu` — nom inconnu, hors portée, refus de service —,
   `arguments_invalides` — deux sources —, `introuvable`, `erreur` — erreur d'outil et panic —, `delai_depasse`) :
   jouée par un test, et le test rougirait-il si l'issue écrite était une autre ? Une issue qui ne peut pas se produire,
   ou un cas réel qui n'a pas d'issue ?
2. **Fail closed (AC-b3)** : journal indisponible au milieu d'une transaction ; `audit_log` présente mais `users`
   illisible (le sous-SELECT de `insert_in_tx`) ; un créateur supprimé (`actor_label` `(inconnu)`, `company_id` `NULL`) ;
   le pool de la trace saturé (délai d'acquisition) — chaque fois, aucune donnée ne part-elle ? La réponse d'erreur
   sous journal indisponible est-elle un oracle (C-32-6-60) ?
3. **Ce qui n'est pas journalisé (AC-b4)** : 401, 403, 405, 413, 415, 400, route absente, notification, lot, message de
   type réponse, `initialize`, `ping`, `tools/list`, `prompts/*` — aucun n'écrit-il en base ? Inversement, un appel
   d'outil légitime peut-il ne rien écrire ?
4. **L'abandon et le panic (C-32-6-58, N26)** : abandon pendant l'attente d'un permis ; abandon au délai puis outil
   qui finit (une seule entrée, `delai_depasse`) ; panic de l'outil après le délai ; arrêt du serveur pendant une tâche
   d'appel. Ce qui reste sans entrée est-il exactement ce que la fiche et le registre écrivent ?
5. **Les mutations** (`326b-mut-*.log`) : chacune tuée par une **assertion nommée** et pour la raison visée (lis le
   journal, pas seulement le verdict) ? Une mutation plausible **non jouée** survivrait-elle — l'entrée écrite avant
   l'issue (durée nulle), l'issue `servi` écrite pour un refus de service, `validation` écrite d'après la clé de test au
   lieu de la clé, le journal écrit après la réponse, `ce_qui_n_est_pas_journalise` passé sur une société non servie ?
   Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-b1 à AC-b6, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-b1** : l'entrée a-t-elle **exactement** les champs prescrits (action, type d'entité, `entity_id`, `user_id`,
  `actor_type`, détail à cinq clés) ? `journaliser_appel` est-elle la seule fonction neuve de `services/` qui emploie
  `trace.pool()` ? Le doc-comment de `Trace::pool` la nomme-t-il ? La dérogation aux règles R1, R2 et R7 est-elle
  écrite, et l'inventaire clos de `services/mod.rs` gagne-t-il cette écriture ?
- **AC-b5** : `mcp.tool_called` dans `audit_labels::ACTIONS` **à sa place triée** (`.` avant `_`), libellé dans les
  **quatre** `.ftl` (`grep -n "audit-log-action-mcp-tool-called" crates/kesh-i18n/locales/*/messages.ftl`) ; l'action
  est un littéral au site `NewAuditLogEntry::user(`. L'annotation de la fiche sur le « repli égal à la valeur fr-CH »
  (issue #758) est-elle exacte — aucun repli frontend pour ces clés (`grep -rn "audit-log-action" frontend/src`) ?
- **AC-b6** : `("route_service", "mcp::recevoir", Traced, …)` ; `traced` 123 et sa généalogie, `exempt` 4 et son
  message d'avant la 32-6a (`git show 76869cc9:crates/kesh-api/tests/audit_route_registry.rs | grep -n "exempt, 4"`),
  `LIB_ROUTES.len()`, totaux et partition de rejeu **inchangés**. Greper la **valeur** (`git grep -nE '\b(122|5)\b'
  crates/kesh-api/tests/audit_route_registry.rs`) et trier à la main.
- **Les choix C-32-6-58 à C-32-6-61** : chacun est-il justifié ; l'un d'eux contredit-il un AC, une décision de Guy
  (Q12 : « chaque envoi reste journalisé ») ou C-32-6-56 ?
- **Les annotations de la fiche** demandées par l'orchestrateur sont-elles des **annotations datées**, sans réécriture
  d'une entrée passée (`git diff origin/main...HEAD -- _bmad-output/implementation-artifacts/32-6-endpoint-mcp.md`) ?

## Axe du manuel et de `docs/api-external.md`

La 32-6b ne touche aucun manuel. Vérifie qu'**aucun texte ne devient faux** par elle, PDF compris :
`grep -n "audit\\\\_log\|MCP\|routes à verbe mutant" docs/manual/fr/admin-manual.tex docs/manual/fr/user-manual.tex`,
puis les **PDF aplatis** (`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE
".{80}routes à verbe mutant.{120}"`). En particulier `admin-manual.tex:2935` (« 122 des 128 routes à verbe mutant »,
« quatre routes exemptées », « 122 + 4 + 2 = 128 ») : la fiche (§ *Le manuel*, A3) en renvoie la correction à la 32-11 ;
dis ce que la 32-6b change à ces nombres (123 tracées sur 129, 4 exemptées) et si le report reste défendable. Le manuel
utilisateur décrit-il le journal d'audit d'une façon que `mcp.tool_called` rend fausse (acteur, marque « clé API »,
`user-manual.tex:3152-3153`) ? Relève ce que la 32-7 devra écrire et que la fiche n'a pas listé.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/326b-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/326b-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/326b-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-6b ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
