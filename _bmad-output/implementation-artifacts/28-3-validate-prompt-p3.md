# Prompt — validation P3 de la spec, Story 28-3 (la sauvegarde lit ses tables dans un instantané commun)

*Versionné le 2026-10-13. Deux lentilles, contexte frais chacune, en lecture seule. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 Sonnet, P2 Opus ; P3 :
**Sonnet** pour les deux lentilles (C-28-3-32). La remédiation de P2 retire un mécanisme (la garde de moteur à
l'exécution) et touche AC1 à AC5, AC7, AC8, AC10, AC11, T2, T6, les mutations et les Dev Notes : trop large pour une
passe ciblée.*

Dépôt `/home/gcorbaz/devel/kesh-28-3`, branche `story/28-3`, **rebasée** sur `5aad8b35` (kickoff de l'Epic 28,
mergé sur `main`). Hachages après le rebase : création de la fiche `32247825`, remédiation P1 `921056f1`, prompt P2
`3df73c64`, **remédiation P2 `016bcb50`**. Fiche :
`_bmad-output/implementation-artifacts/28-3-sauvegarde-instantane-commun.md`. Le diff de la remédiation P2 :
`git diff 3df73c64 016bcb50 -- _bmad-output/`. Rapports de P2 : `/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p2-R.md`
et `…-p2-F.md` ; de P1 : `…-p1-R.md`, `…-p1-F.md` (à lire **après** ta propre lecture de la fiche, pour ne pas en
hériter les angles). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-3* (sur la branche
depuis le rebase). Issues #633 et #465 (`gh api repos/guycorbaz/kesh/issues/633`, `…/465`, et leurs `/comments`).
Registre : `_bmad-output/implementation-artifacts/epic-28-choix-autonomes.md` § *Story 28-3* (C-28-3-1 à 32 ;
C-28-3-25 à 31 sont ceux de la remédiation P2 ; le registre séparé de la création y a été versé par union au rebase,
puis supprimé — vérifie que l'union n'a rien perdu : `git show 3df73c64:_bmad-output/implementation-artifacts/epic-28-choix-autonomes-28-3.md`).
Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le manuel nommé, les axes déclarés, les sites
NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ; § Migration breaking policy, dont
l'exception `kesh-db` au gate ciblé ; § Règle de splitting préventif, **amendement D5** ; § Issue Tracking Rule).
Sources externes utiles, **en lecture** : sqlx 0.8.6 sous `~/.cargo/registry/src/*/sqlx-core-0.8.6/` et
`…/sqlx-mysql-0.8.6/` (`src/transaction.rs`, `src/pool/mod.rs`, `src/pool/connection.rs`, `src/error.rs`,
`src/connection/executor.rs`).

## Contexte de la passe — le signal D5 et la décision prise

P1 : 3 MEDIUM ; P2 : 5 MEDIUM, dont **4 nés de la remédiation P1** (garde de moteur à l'exécution qui échouait fermé,
repli `INNODB_TRX`, preuve `git diff` T1→T2). L'orchestrateur, sur l'autonomie déléguée par Guy, a décidé de ne pas
découper et de remédier **par simplification** : retirer les mécanismes neufs plutôt que les réparer (Change Log
« Validation P2 », C-28-3-25 à 27). Cette passe dit si la simplification a tenu.

## Axe premier (commun aux deux lentilles) — les régressions de la remédiation P2

*La sévérité ne stagne pas, elle se déplace vers ce qu'on vient d'écrire* (CLAUDE.md § La passe ciblée). Relis
d'abord, ligne à ligne, le diff `3df73c64..016bcb50`, et cherche ce que la remédiation a **cassé ou laissé
incohérent**. Pour **chaque** finding né de cette remédiation, réponds explicitement à la question :
**« ce défaut est-il un recyclage ? »** — c'est-à-dire un défaut de P1 ou de P2 qui revient sous une autre forme, ou
qui naît du correctif de P2 (amendement D5 du § *Règle de splitting préventif*) — ou bien un défaut distinct, d'origine.
Le décompte final dit combien de recyclages, combien de défauts d'origine.

1. **Retrait de la garde de moteur (C-28-3-25)** : reste-t-il, dans la fiche, le registre (hors entrées historiques
   annotées), le CHANGELOG prescrit (AC11), le manuel prescrit (AC10), l'AC7, les tâches ou les mutations, une trace
   d'échec fermé, de 81 requêtes, de (e1)/(e2), de M12 active ? Le test (e) refondu est-il réalisable et
   discriminant (assertion de montage sur le nombre de tables, `Option<String>`, ordre de `e.tables` contre
   `TABLES_TO_TRUNCATE`) ? M9 (mutation du squash `_kesh_version` en `MyISAM`) est-elle applicable et tuée par (e) —
   **non vérifié** si l'acceptation du squash muté exige une requête ? La justification (« pire que le mal » sur le
   NAS) est-elle cohérente avec ce que la fiche promet encore (AC1, AC10, AC11) ? R2-1, R2-2, F2-10, R2-6 sont-ils
   réellement fermés, et non déplacés ?
2. **Manuel prescrit (AC10)** : la phrase de moteur (« l'instantané ne vaut que pour les tables InnoDB », « sans que
   l'export le signale », « sur l'image MariaDB de `docker-compose.yml`, toutes les tables de Kesh sont InnoDB »,
   « deux d'entre elles ») est-elle vraie au dépôt ? La requête de vérification est-elle juste (`DATABASE()` sous le
   client indiqué, `_sqlx_migrations` et toute autre table hors sauvegarde qu'elle listerait) ? La commande
   `docker compose exec mariadb mariadb -u kesh -p kesh` est-elle conforme à `docker-compose.yml` et à la façon dont le
   manuel décrit déjà l'accès à la base (`admin-manual.tex:178`, `:1496`, `:1548`) ? Le remède (`ALTER TABLE …
   ENGINE=InnoDB`, `default_storage_engine`) est-il sûr, et le compte Kesh en a-t-il le droit ? La reformulation de
   R2-3 (« une base qui en porte déjà les garde ») est-elle exacte ? Les contrôles aplatis prescrits sont-ils justes
   sur le PDF (apostrophes, `lstlisting`) ?
3. **`@@in_transaction` et le repli (C-28-3-26)** : « le test vérifie lui-même que la variable existe » est-il vrai
   (une erreur de variable inconnue fait-elle échouer le test, ou un `fetch_optional`/une conversion le ferait-il
   passer à vide) ? Le repli `SET TRANSACTION READ WRITE` → `1568` est-il discriminant sous M6, sans privilège, et
   sans laisser d'état gênant sur la connexion du pool d'une connexion ? (Faits MariaDB : **non vérifiés**, à signaler
   comme tels.)
4. **Preuve `git log -L` (C-28-3-27)** : la commande prescrite est-elle exécutable telle qu'écrite (citation de
   l'expression, `/fn <nom>(/` sur un nom qui est le préfixe d'un autre — `orphelins` contre un futur
   `orphelins_…` ? —, borne `^    }$` contre une fonction qui contient une ligne `    }` à quatre espaces avant sa
   fin, plage résolue dans T2 et suivie en arrière) ? Une sortie vide prouve-t-elle bien que les trois tests et leurs
   helpers n'ont pas changé ? L'argument contre `-L :<fonction>:` (pas de `.gitattributes`, pilote par défaut) est-il
   exact au dépôt et dans git ? AC5 (a) et T2 sont-ils désormais sans contradiction ?
5. **Cœur et `finish` (C-28-3-29)** : la signature prescrite d'`export_installation_in` est-elle cohérente avec les
   gardes lexicales (g)(i), (g)(ii), avec (a), (b), (d), (f), et avec l'AC3 (« `export_installation` clôt
   l'instantané ») ? `begin` = `acquire` + `begin_on` est-il compatible avec T1 (intérieur dégradé) ?
   `#[cfg_attr(not(test), allow(dead_code))]` est-il la bonne forme, et suffit-il (variant `Ouvert`, `Etape`
   elle-même, `begin_on`) sous `clippy -D warnings` ?
6. **Signatures sqlx (C-28-3-30)** : `BoxFuture<'a, T>` et son emploi `BoxFuture<'static, ()>` ;
   `Some("…".into())` vers `Option<Cow<'static, str>>` ; le cas `BeginFailed` et la borne « une fois l'instantané
   ouvert » de l'AC1 ; « signalée, non épinglée » contre `Cargo.toml:16` et `Cargo.lock`. Lis le code de sqlx.
7. **Recompte des moteurs** : refais le recompte des `CREATE TABLE` sans clause `ENGINE` (hors lignes de
   commentaire) par une commande **à toi**, et confronte-le aux « 2 des 40 » de la fiche et à l'explication donnée
   pour les « 5 » de la lentille R de P2 (`;` dans un commentaire ou une chaîne : `bank_imports` l. 25, `bank_profiles`
   l. 39, `api_keys` l. 25).
8. **Angles morts écrits (C-28-3-31)** : alias du pool, DDL `INSTANT` — la formulation est-elle exacte, et n'en
   manque-t-il pas un autre de même nature ?
9. **Propagation et recomptes** : refais les greps de l'AC8 ; greppe les symptômes corrigés en P2 — `INNODB_TRX`,
   `échoue fermé`/`échoue **fermé**`, `\b81\b`, `13 des 51`, `\b51\b`, `(e1)`, `(e2)`, `épingl`, `487-488`,
   `git diff <commit T1>`, `ne porte jamais`, `epic-28-choix-autonomes-28-3` — sur la fiche, le registre, le
   `sprint-status.yaml` et le dépôt ; les occurrences historiques (Change Logs, entrées annotées, prompts P1/P2) sont
   légitimes, les autres non. Recompte : 12 AC, T0-T6, 12 lignes de mutations dont 11 actives (M12 retirée), chaque
   mutation tuée par un test qui **existe** dans la fiche, C-28-3-1 à 32 (suite continue), 80 requêtes, 39 tables,
   40/40, le Change Log de P2 (19 bruts — R 3 MEDIUM + 5 LOW, F 2 MEDIUM + 9 LOW —, 17 distincts, 5 MEDIUM, 14 LOW
   bruts), chaque numéro de ligne neuf (`test_schema_guard.rs:154-168`, `:166`, `reconciliation_e2e.rs:7244-7248`,
   `backup.rs:488-489`, `kesh-db/Cargo.toml:16`, `sqlx-core-0.8.6/src/transaction.rs:101`,
   `sqlx-mysql-0.8.6/src/transaction.rs:33-34`).

## Axes de P1 et P2, à reprendre (communs aux deux lentilles)

1. **Inventaire clos des lectures** (Dev Notes § *Inventaire clos*) : refais les `grep` copiés, et **élargis-les** —
   toute lecture de la base faite pendant la construction d'un `.keshbackup`, sous toutes formes (`sqlx::query*`,
   `query_scalar`, `QueryBuilder`, `format!` de SQL, appels de dépôt, `INFORMATION_SCHEMA`, `manifest.rs`, le handler
   `full_export`, `run_backup_and_restore`, `kesh-seed`, `test_endpoints.rs`, tout autre producteur). Une lecture qui
   échapperait au `ReadSnapshot` est au moins **HIGH**.
2. **Manuel contre code** (obligatoire) : `docs/manual/fr/admin-manual.tex` § *Exporter (sauvegarde complète)*
   (l. 1823 et suivantes), § *Importer* et la sauvegarde pré-import, § *Quelle méthode de sauvegarde choisir ?*
   (l. 1901), l'installation Synology (base fournie, l. 1580 environ) ; `docs/manual/fr/user-manual.tex`
   § *Exports de souveraineté*. Ce que les manuels disent aujourd'hui, et ce que l'AC10 prescrit, est-il vrai au
   code ? Contrôle les **PDF**, aplatis : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`.
   Une promesse que le code ne tiendrait pas est au moins MEDIUM.
3. **Couverture** (lentille R surtout) : chaque phrase de #633 et du § *28-3* de l'epic par un AC testable ; chaque
   AC par une tâche et par un test qui rougirait ; gardes existantes ; séparation de #465 ; mots-clés de PR
   (`closes #633`, `refs #465`). La demande à l'orchestrateur (requête de moteur sur le NAS, T6) est-elle bien hors
   agent et sans donnée réelle écrite au dépôt ?
4. **Sémantique MariaDB/sqlx et concurrence** (lentille F surtout) : mécanisme de l'AC1 ; `ReadSnapshot` ;
   déterminisme de chaque test de l'AC5 ; chaque mutation tuée ; écrivains concurrents et `FOR UPDATE` du pré-import ;
   DDL, durée, purge, pool, mémoire.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  axe premier (points 1, 2, 7, 9 en priorité), puis axes 1 à 3 ; chaque fait, chemin et numéro de ligne cité par la
  fiche **revérifié** ; textes prescrits confrontés au code prescrit ; union du registre vérifiée.
- **F — Faisabilité, sémantique MariaDB/sqlx et tests (adversaire de la conception)** : axe premier (points 1, 3 à 6
  en priorité), puis axe 4 ; lis le code de sqlx 0.8.6 pour chaque appel que la fiche prescrit, au lieu de t'en
  remettre à ce que la fiche ou les rapports précédents en disent.

## Ce que tu rends

Findings numérotés (`R3-…` ou `F3-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`,
**preuve** (commande exécutée et sortie copiée), correction proposée ; pour chaque finding, dire s'il est **né de la
remédiation P2** (diff `3df73c64..016bcb50`) ou **d'origine**, et, s'il est né de la remédiation, **s'il est un
recyclage** (réponse motivée). Toute affirmation de présence ou d'absence = sortie de `grep -nF` copiée. Un fait
MariaDB que tu ne peux établir qu'en exécutant une requête — interdit ici — se signale comme **non vérifié**, avec le
test de la fiche qui le trancherait ; il ne se tranche pas de mémoire.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme passe. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p3-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/28-3-validate-p3-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, le décompte recyclages / défauts d'origine, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
`scripts/regen-test-schema.sh` et les scripts Synology), `make`, `latexmk`, `xelatex`, `git commit`/`add`/`checkout`/
`switch`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `rustc`, `npm`, `npx`, `node`, `docker`,
**`mariadb`, `mysql`, `mariadb-dump`** et toute requête SQL, **tout interpréteur** (`python`, `python3`, `perl`,
`ruby`, `bash`/`sh` sur un script, `awk` qui écrirait un fichier), `gh` en écriture. Rien de `~/kesh-test/` (relevé
réel de Guy) ne se lit ni ne se cite. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`, `git log`/`show`/`diff`/
`blame`, `gh api` en lecture, `pdftotext` vers la sortie standard.
