# Prompt — revue de code P1 de la Story 32-8a (la table des envois MCP et sa place dans la sauvegarde), trois lentilles, Sonnet

Tu relis le développement de la **partie a de la Story 32-8** de Kesh (comptabilité suisse, Rust/Axum, MariaDB 10.11) :
*la table `mcp_sends` — les envois du serveur MCP en attente de validation par un administrateur —, son entité, son
dépôt complet (dépôt sous plafond, purge des échus, relecture par la clé, liste, détail, décision sous verrou), et sa
place dans la sauvegarde : `TABLES_TO_TRUNCATE`, squash de test, chemin d'upgrade, export de sauvegarde, export global
CSV, aller-retour d'import et remise à zéro.* L'état « échu » n'est jamais écrit : il se dérive à la lecture, à
l'horloge de MariaDB (`ETAT_DERIVE_SQL`). C'est la fondation de la **validation avant envoi**, la partie de sécurité de
l'Epic 32 : ce qui sera stocké ici est une copie de données lues dans des livres comptables.

Worktree `/home/gcorbaz/devel/kesh-32-8a`, branche `story/32-8a`. **Seuls les commits de la 32-8a sont en revue** : ils
suivent le commit de fusion `5c8ff91b` (fiche + `origin/main` à `cd86dc74`). Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-32-8a diff 5c8ff91b..HEAD`. Langue du rapport : français. Modèle : **Sonnet**.
Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Hors périmètre** : la poignée côté serveur (politique, `kesh_lire_envoi`, routes de décision, audit, gardes
lexicales) est la **32-8b** ; le prédicat des caractères cachés est la **32-8b0** (en revue ailleurs, ne lis pas son
worktree `/home/gcorbaz/devel/kesh-32-8`) ; l'écran, les manuels, leurs PDF et le CHANGELOG sont la **32-8c** (C-32-8-29).
Un défaut que la 32-8a ne touche pas est un finding **antérieur**, à classer comme tel. Un besoin de la 32-8b que le dépôt
ne couvre pas **est** un finding de la 32-8a (AC-A4 : « la 32-8b n'y ajoute rien »).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/32-8-validation-avant-envoi.md` — *Partie A* (AC-A1 à AC-A12), *Tâches 32-8a*,
*Tests et mutations prévus* (MA1 à MA16), *Gates*, *Dev Notes* (§ *Les états d'un envoi*, *Le schéma*, *Ce que le dépôt
fournit*, *Le flux d'un `tools/call`*, *Les signatures*), le § *À recontrôler à l'étape 0* (dont P3-F3) et le **Dev
Agent Record** de la 32-8a (étape 0, mutations, gates, décomptes) — puis le registre
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-328.md`, **C-32-8-40 à C-32-8-43** (choix du
développement) et C-32-8-2, 3, 13, 29. Le Dev Agent Record déclare des gates, des mutations et un E2E : **ne les prends
pas pour argent comptant**, recoupe-les avec les journaux `/home/gcorbaz/devel/kesh-gate-logs/328a-*` (lecture seule).
Aucun contrôle visible ne change : pas de capture attendue.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au rapport ; il ne se
  supprime pas (supprimer, c'est écrire).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/`python3`, ni `perl`, ni `ruby`, ni
  `node`/`npm`/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/regen-test-schema.sh`, ni `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni `scripts/install-hooks.sh`,
  ni aucun script de `kesh-gate-logs` ou d'un scratchpad (dont `mutations-328a.py`) ; ni `make`, ni `latexmk`, ni
  `cargo` (aucune sous-commande, `cargo fmt` compris), ni `rustc`, ni `docker`, ni `mysql`/`mariadb`, ni `sqlx`, ni
  `curl`, ni `sed -i`.
- **Aucune commande `gh`**, pas même en lecture.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `sha384sum`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques. Ne lis rien sous `~/kesh-test/`.

## Les axes obligatoires, pour chaque lentille

1. **La politique des migrations (P1 à P8 de `CLAUDE.md`)**, chacune avec sa preuve : DDL seul (aucun `INSERT`,
   `UPDATE`, `DELETE`, `REPLACE` décommenté dans la migration), aucun `DROP`/`RENAME`/`MODIFY`/`CHANGE`, pas de bump ;
   ligne d'audit (`docs/migrations-idempotence-audit.md`) **et** compteurs recomptés depuis la source (`ls
   crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20' docs/migrations-idempotence-audit.md`, la partition
   `yes`/`tracked-by-sqlx`/`no`) ; P6 : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et chaque site
   inspecté, en particulier `migrations_upgrade_path.rs` — **greper la valeur** (`grep -nE "\b(84|85|50|51|49)\b"
   crates/kesh-db/tests/migrations_upgrade_path.rs`) et trier chaque ligne (frontière toujours 34) ; P8 : la somme de
   la ligne ajoutée à `crates/kesh-db/migrations.sha384` égale `sha384sum` du fichier, et cette ligne est la dernière
   modification de la branche sur la migration (`git log --format=%h -- crates/kesh-db/migrations/20261025000002_mcp_sends.sql`).
2. **Le squash et le schéma réel** : la table du squash (`crates/kesh-db/test-schema/0001_schema_squash.sql`) dit-elle
   la même chose que la migration (contraintes, collation du `CHECK` de `state`, colonnes sans `DEFAULT`) ? Les tests
   du dépôt montent le squash : une contrainte présente dans la migration et absente du squash, ou l'inverse, les
   ferait-elle passer à vide ?
3. **Chaque dépendance lue à la source.** Toute affirmation du code, de ses commentaires, du registre ou du Dev Agent
   Record sur le comportement de MariaDB 10.11 (`NOW(3)` constant dans une instruction, `INTERVAL ? SECOND` lié en
   paramètre, comparaison `PAD SPACE` de `utf8mb4_unicode_ci`, `rows_affected` d'un `UPDATE` qui n'aurait rien changé)
   ou de `sqlx-mysql` 0.8.6 (décodage d'un `BOOLEAN`/d'une expression booléenne vers `bool`, `#[sqlx(flatten)]`) se
   vérifie — pour `sqlx`, dans `~/.cargo/registry/src/*/sqlx-mysql-<version>/` (version lue dans `Cargo.lock`, ligne
   citée) ; pour MariaDB, dis ce qui ne se vérifie pas en lecture seule.

## Lentille B — Blind (le diff seul, sans la fiche)

Lis le diff aplati **avant** la fiche, comme un relecteur qui ne connaît pas l'intention.

1. **`crates/kesh-db/src/repositories/mcp_sends.rs`** — chaque requête : est-elle scopée par société (et, pour
   `lire_pour_cle`, par clé **et** société) ? `ETAT_DERIVE_SQL` est-elle la **seule** définition de l'échéance (une
   requête qui compare `expires_at` à la main ?) — `decider_in_tx` lit `expires_at <= NOW(3)` en direct : est-ce cohérent
   avec la constante ? Le `SELECT … FOR UPDATE` puis l'`UPDATE` peuvent-ils diverger (ligne d'une autre société,
   `rows != 1`) ? `modified` se calcule-t-il sur les **octets** (jamais par la collation) ? Un refus pose-t-il bien
   `content`, `summary` à `NULL` et `modified` à `FALSE` ? `COALESCE(?, content)` est-il juste quand le texte reçu est
   une chaîne vide ? `lister` : ordre, borne 200, un `LIMIT ?` lié en `u32`. Une valeur hors de l'énumération stockée
   en base ferait-elle échouer la liste entière (et est-ce souhaitable) ?
2. **`crates/kesh-db/src/entities/mcp_send.rs`** — décodage strict de `EtatEnvoi` ; `Debug` manuel des types porteurs
   de contenu (C-32-8-40) : un chemin par lequel `content` ou `summary` serait recopié dans un `Debug`, un `Display`
   d'erreur ou un évènement `tracing` ? `ErreurDepot`/`ErreurDecision` : leurs messages nomment-ils une donnée ?
3. **`crates/kesh-db/src/backup.rs`** — place de `"mcp_sends"` (avant `mcp_keys`, enfant de `mcp_keys`, `companies`,
   `users`) ; absence de `RESET_PRESERVED_TABLES`. Une interaction avec la réparation des principaux orphelins
   (`companies::reattach_orphan_principals_in_tx`, qui **repointe** `mcp_keys.company_id`) peut-elle laisser une ligne
   d'envoi dont la société n'est plus celle de sa clé ? Est-ce un défaut, et de quelle partie ?
4. **Les tests** (`mcp_sends_repository.rs`, `full_import_restores_mcp_sends_byte_for_byte`, le montage de
   `onboarding_audit_e2e.rs`, `admin_full_export_e2e.rs`) — un test peut-il passer **à vide** : un contre-exemple de
   contrainte refusé pour une **autre** raison que la contrainte nommée (le test cherche-t-il le nom de la contrainte
   dans le message ?), une égalité qui compare deux valeurs sorties de la même source, une échéance « passée » qui ne
   l'est pas à l'horloge de la base, une ligne d'aller-retour égale parce que l'import ne l'a pas touchée ?

## Lentille E — Edge cases

1. **L'horloge** : `expires_at` exactement égal à `NOW(3)` (vivant ou échu ?) ; même règle partout (lecture, plafond,
   purge, décision) ? Un dépôt et une lecture dans la même milliseconde ? Une durée `0` ou `u32::MAX` passée à
   `deposer_in_tx` (dépassement de `DATETIME`, erreur rendue ou ligne invalide) ?
2. **Le plafond** (AC-A5, C-32-8-13) : souple par construction (aucun verrou) — de combien peut-il être dépassé, et
   est-ce écrit ? Un plafond `0` ? Un envoi échu non purgé compte-t-il ?
3. **La décision** (AC-A6) : l'ordre des quatre refus et sa conséquence (un envoi validé **et** échu → `DejaDecidee`) ;
   `Valider { contenu: Some("") }` ; un texte reçu identique au stocké mais d'une autre normalisation Unicode (NFC/NFD)
   → `modified` vrai : souhaité ? ; un envoi en attente dont le contenu aurait été purgé (pourquoi impossible ?).
4. **La relecture** (AC-A7) : les trois `None` sont-ils réellement indiscernables (même chemin de code, même erreur) ?
   L'invariant « validé vivant sans contenu » rendu en erreur (C-32-8-42) : reste-t-il atteignable par l'import d'une
   sauvegarde forgée ?
5. **La sauvegarde** : export → import d'un envoi à `content` de 65 536 octets ; d'un `MEDIUMTEXT` proche de 16 Mio
   (le format NDJSON de la sauvegarde a-t-il une borne ?) ; d'une ligne dont `decided_by_user_id` désigne un utilisateur
   absent de la sauvegarde ; l'ordre d'import (FK `RESTRICT` vers `mcp_keys`).
6. **Les mutations** (`/home/gcorbaz/devel/kesh-gate-logs/328a-mutations.log`) : chacune tuée par une **assertion
   nommée** et pour la raison visée (lis le journal, pas seulement le verdict) ? Une mutation plausible **non jouée**
   survivrait-elle — `lister` sans filtre de société, `trouver` qui rend le contenu d'un échu, le `CHECK`
   « décision complète » retiré, `chk_mcp_sends_modified` retiré, `chk_mcp_sends_tool` retiré, `deposer_in_tx` qui pose
   `modified = TRUE`, l'ordre de la liste inversé ? Raisonne sur le code, n'exécute rien.

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC-A1 à AC-A12, rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve. En
particulier :

- **AC-A1** : l'en-tête suit-il le patron de `20261025000001_…sql:1-44` (objet, table applicative, inventaire changé,
  non breaking, P1, idempotence, triage P7) ? Ligne de `migrations.sha384` en dernier.
- **AC-A2** : chaque colonne et chaque contrainte du § *Le schéma* ; `state` et `modified` **sans `DEFAULT`** ; chaque
  contre-exemple écrit de l'AC testé.
- **AC-A4** : la table de correspondance du § *Ce que le dépôt fournit* — chaque signature, chaque type de retour,
  chaque variante d'erreur. Compare-la au § *Le flux d'un `tools/call`* et aux *Signatures* de la 32-8b : manque-t-il
  quelque chose dont la 32-8b aura besoin ?
- **AC-A10** : partition de la remise à zéro, tests de partition **non modifiés** (`git diff 5c8ff91b..HEAD --
  crates/kesh-db/src/backup.rs` : seule la liste change) ; squash ; compte 42 et sa généalogie ; aller-retour ; remise à
  zéro (`EMPTY_BEFORE_RESET` inchangée).
- **AC-A11** et **AC-A12** : les gestes de grep prescrits (`grep -nwE "onze|douze|41|42"
  crates/kesh-api/src/exports/global.rs`), chaque ligne triée.
- **Les choix C-32-8-40 à C-32-8-43** : chacun est-il justifié ; l'un d'eux contredit-il un AC ou une décision de Guy ?
- **Le Dev Agent Record** : chaque nombre (tests, mutations, gates, E2E) se recompte-t-il depuis sa source, avec son
  périmètre ?

## Axe du manuel

La 32-8a ne touche **aucun** manuel : ses textes passent à la 32-8c (C-32-8-29, AC-C11). Vérifie qu'**aucun texte
n'est oublié** de la liste de l'AC-C11 et qu'aucun ne devient faux sans y figurer, PDF compris :
`grep -nE "MCP|deux tables|en ajoute deux" docs/manual/fr/*.tex CHANGELOG.md`, puis les **PDF aplatis**
(`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(mcp_keys|tables).{120}"`).
Nomme tout site qui compte les tables de la sauvegarde ou les tables du serveur MCP et que l'AC-C11 ne liste pas.

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/328a-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/328a-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/328a-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par l'outil
d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier et
   ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée, et
   s'il est **né du développement** de la 32-8a ou **antérieur**.
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, s'il y en a.
