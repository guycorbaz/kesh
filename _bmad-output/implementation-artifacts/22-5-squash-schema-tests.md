# Story 22.5 : Le schéma de test se rejoue en un geste, plus en soixante et un

## Status

in-progress

## Story

**As a** développeur de Kesh (humain ou agent) qui doit passer le gate complet avant chaque push,
**I want** que la suite d'intégration cesse de payer soixante et une migrations par test,
**so that** un gate complet coûte un quart d'heure et non pas l'heure qui, à huit petites modifications par jour, mange la journée.

Ferme **#251** — sous condition d'AC4 : un gain sous le seuil ne ferme pas, il documente. Story de l'**Epic 22 « Technical Debt Closure »**. Engagement explicite de Guy (2026-08-14) : « crée la story #251 dès que la 22-1 est bouclée » — elle l'est (PR #309).

## Contexte — le goulot est mesuré, et il n'est pas le CPU

Chaque test `#[sqlx::test]` crée une base éphémère **vide** puis rejoue le `MIGRATOR` entier — un cycle `INSERT _sqlx_migrations` / DDL à commit implicite / `UPDATE success=TRUE` répété **61 fois par test**, sérialisé par les metadata-locks MariaDB. C'est ce coût qui a figé le plafond nextest à **6 threads** (au-delà : contention, flakes mesurés) et limité le gain de nextest à 1,40×. Chaque migration ajoutée ralentit *tous* les tests — la 22-1 vient d'en ajouter une 61ᵉ.

**Le recensement qui fait foi — recompté au commit de cette passe, par attributs RÉELS** (un grep naïf de `#[sqlx::test` rend 1182 en comptant les mentions de doc-comments ; le compte s'ancre sur les lignes d'attribut, méthode ci-dessous) :

| Population `#[sqlx::test]` | Compte | Où | Sort dans cette story |
|---|---|---|---|
| `migrator = "kesh_db::MIGRATOR"` | **1080** | 3 crates | → `migrations = "…/test-schema"` (graphie de son crate), sauf exclusions D2 |
| `migrations = "../kesh-db/migrations"` | **23** | `bank_profiles_e2e.rs` (14), `email_templates_e2e.rs` (9) | → `migrations = "../kesh-db/test-schema"` |
| `migrations = false` | **17** | backfills à fenêtre, upgrade_path | inchangés (gèrent eux-mêmes leurs migrations) |
| `migrator = "crate::MIGRATOR"` | **12** | `kesh-db/src/{backup,test_fixtures}.rs` (tests internes au crate) | → `migrations = "./test-schema"` |
| `migrations = "./migrations"` | **9** | `kesh-db/src/repositories/bank_profiles.rs` | → `migrations = "./test-schema"` |
| attribut **nu** (`InferredPath`) | **1** | `accounts_role_backfill.rs` | fichier d'exclusion D2 : **reste exclu**, l'attribut s'explicite en `migrator = "kesh_db::MIGRATOR"` — ⚠️ un attribut nu applique TOUT le migrator (`InferredPath`), il n'en désactive aucun : `migrations = false` viderait la base du test (*réfuté sur pièces en passe 3*) |
| **Total** | **1142** | 718 kesh-api, 393 kesh-db, 31 kesh-report | la ventilation SOMME au total — c'est le contrôle |

*Méthode de recomptage (à rejouer, pas à relire)* : attributs réels seuls, doc-comments exclus —

```sh
python3 -c "
import re, pathlib
pat = re.compile(r'^\s*#\[sqlx::test(\(([^\]]*)\))?\]\s*\$')
print(sum(1 for p in pathlib.Path('crates').rglob('*.rs')
            for l in p.read_text(errors='replace').splitlines() if pat.match(l)))"
```

⚠️ **Cinq graphies distinctes atteignent le même chemin de 61 migrations** — le recensement compte SIX populations, la sixième étant `migrations = false`, qui n'atteint aucun chemin (autogérée). La première rédaction n'en voyait qu'une (« 1092 attributs `migrator = "kesh_db::MIGRATOR"` ») — un nombre qui additionnait en réalité deux littéraux sans le dire, et laissait 33 tests hors de la bascule ET hors de l'audit. *(Relevé en passe 1 par les trois lentilles ; recompté par l'orchestrateur, ventilation ci-dessus prouvée sommante.)* D'où **AC3 : la complétude est tenue par un TEST, pas par un grep.**

**Autres compteurs périmés que cette story rafraîchit** : `.config/nextest.toml` (« ~894 tests / 51 migrations », 2026-07-13), les deux commentaires CI « 84 tests » (`ci.yml:124`, `:130`) sur base partagée (réels : 154), **et la § *Plafonds mémoire* du `CLAUDE.md`** qui porte les mêmes « ~894 » et « 51 migrations » (ligne 114) — le site que la § *Propagation post-patch* aurait rendu au grep de `\b894\b`, nommé ici pour ne pas dépendre de la vertu de l'exécutant.

## Ce que l'exploration a fermé comme voies (contraintes dures, vérifiées dans les sources)

- **sqlx 0.8.6 n'offre aucun point d'accroche** pour un schéma pré-migré : `test_context()` fait `CREATE DATABASE` vide puis `migrator.run_direct` ; `TestSupport` est un impl global non surchargeable ; `snapshot()` est un `todo!()` pour MySQL ; `fixtures(...)` se rejoue *après* le migrator.
- **MariaDB ne clone pas une base** : ni `TEMPLATE` (Postgres), ni plugin `CLONE` (MySQL 8), ni `mariabackup` par schéma à chaud.
- **Le partage de base entre tests est ÉCARTÉ** (arbitrage du 2026-08-14) : modes d'échec « base piégée » payés deux fois sur l'Epic 16 ; les tests destructeurs ne peuvent rien partager. **L'isolation par test se conserve.**
- Les réglages de durabilité sont **serveur** (flags de démarrage), pas session : les pools de test sqlx n'ont pas d'`after_connect`.

**Et ce qu'elle a confirmé possible, sur pièces** : `Migration` a tous ses champs `pub` et un constructeur public (`sqlx-core-0.8.6/src/migrate/migration.rs:7-36`), et le chemin d'attribut est un `syn::Path` quelconque (`test_attr.rs:278-295`). *(Vérifié en passe 1 — mais la voie du `static` a finalement été ABANDONNÉE en passe 3 : `Migration::new` n'est pas `const`, et le répertoire de squash fait mieux sans une ligne de Rust, cf. D1 révisé.)*

## Décisions

**D1 — Le template est un RÉPERTOIRE DE SQUASH : `crates/kesh-db/test-schema/`, une seule migration portant le dump DDL du schéma complet.** *(Révisé en passe 3.)*

Le squash vit dans `crates/kesh-db/test-schema/0001_schema_squash.sql` et les tests le visent par l'attribut **`migrations = "…/test-schema"`** (chemin relatif au `CARGO_MANIFEST_DIR` du crate du test : `"./test-schema"` dans kesh-db, `"../kesh-db/test-schema"` dans kesh-api et kesh-report). C'est **l'option 1 de l'issue #251 elle-même**, que la première rédaction avait écartée sans motif au profit d'un `static TEST_MIGRATOR` — écart réfuté en passe 3 sur deux pièces : `Migration::new` n'est pas `const` (le `static` littéral ne compile pas tel qu'esquissé), et un `pub static` non-`cfg(test)` embarquerait ~70 Ko de DDL de test dans la rlib et le binaire de production. Le répertoire n'a aucun de ces coûts : zéro code Rust, checksum et embarquement gérés par la macro `sqlx::test` exactement comme pour le vrai chemin, et les DEUX graphies licites sont énumérées par le test de complétude d'AC3.

Le SQL du squash contient le dump DDL (sans données) **plus la réinjection de la ligne `_kesh_version` (id=1)** — sans elle, `check_downgrade_protection` et le verrou d'installation (`FOR UPDATE` sur id=1) changent de comportement dans les tests. ⚠️ **La ligne se `SELECT`e depuis la base jetable APRÈS passage du vrai `MIGRATOR`** — jamais recopiée du SQL d'amorçage : l'état réel après 61 migrations est `min_required = '0.10.0'` (deux bumps l'ont réécrite), pas `'0.1.0'` ; un littéral codé en dur se périmerait à chaque bump P2 et contredirait la promesse « aucune checklist » de D3. `applied_at` est **omis** (défaut serveur, jamais lu). *(Relevé en passe 3 par deux lentilles.)*

Les populations du recensement basculent selon la colonne « Sort » du tableau ; un test paie alors **un** batch DDL au lieu de 61 cycles.

⚠️ **Le schéma d'origine porte 7 INSERT répartis sur 3 migrations** (`20260428000001` ×4, `20260522000001` ×1, `20260614000001` ×2) — pas « un seul ». `mysqldump --no-data` les élimine tous ; seule la ligne `_kesh_version` doit être réinjectée à la main, les six autres étant des backfills conditionnés à `companies`, no-ops sur une base fraîche. *(La première rédaction disait « le seul INSERT » — faux, recompté en passe 1.)*

⚠️ **La base éphémère reste une base par test** — rien ne change à l'isolation, seul le chemin de construction change.

**D2 — Une liste d'EXCLUSIONS fermée ICI, au grain du fichier, et tenue par le test d'AC3.**

Restent sur le chemin réel des migrations — parce qu'ils testent **ce chemin lui-même** :

| Fichier | Motif |
|---|---|
| `kesh-db/tests/migrations_fresh_install.rs` | l'installation fraîche EST le sujet |
| `kesh-db/tests/migrations_upgrade_path.rs` | fenêtre d'upgrade partielle (2 `migrations = false` + sub-`Migrator` ; les 17 du recensement : 2 ici + 2 `accounts_role_backfill` + 13 `invoice_lines_…` — *ventilation corrigée en passe 3*) |
| `kesh-db/tests/accounts_role_backfill.rs` | backfill à fenêtre — 2 `migrations = false` + l'attribut nu à expliciter en `migrator = "kesh_db::MIGRATOR"` (il applique TOUT le migrator aujourd'hui) |
| `kesh-db/tests/invoice_lines_revenue_account_backfill.rs` | backfill à fenêtre |
| `kesh-db/tests/post_restore_class_a.rs`, `post_restore_transactionality.rs` | triage P7 |
| `kesh-db/tests/client_number_canonical_backfill.rs` | backfill D6 sur schéma réel |
| `kesh-db/tests/test_schema_guard.rs` (créé par T3/T5) | **le garde-fou lui-même** : il compare les deux chemins, il lui faut le vrai — l'oublier ici rendrait la story incapable de satisfaire son propre AC3 (*relevé en passe 3 par deux lentilles*) |

Tout le reste bascule. Un fichier qui voudrait rejoindre cette liste le fera **en modifiant le test d'AC3**, qui la porte en dur — c'est le rappel automatique, pas la prose de cette spec.

**D3 — Le squash SE RÉGÉNÈRE, il ne s'édite JAMAIS — un garde-fou à TROIS assertions le tient (structure, données d'amorçage, suivi `_sqlx_migrations`), et ce garde-fou EST le mécanisme d'entretien.**

- `scripts/regen-test-schema.sh` produit `crates/kesh-db/test-schema/0001_schema_squash.sql` : base jetable montée par le vrai `MIGRATOR`, dump **`--no-data` en excluant `_sqlx_migrations`**. ⚠️ **Le mode d'échec de l'inclusion est MUET, pas bruyant** — la passe 1 l'annonçait en « table already exists » ; la passe 3 a joué le dump réel : `--add-drop-table` (défaut) émet `DROP TABLE IF EXISTS` avant chaque `CREATE`, le DROP détruirait la ligne de suivi que sqlx vient d'insérer, l'`UPDATE success=TRUE` affecterait zéro ligne **sans erreur**, et `information_schema` serait identique des deux côtés — aucune jambe structurelle ne le verrait. D'où l'assertion dédiée de T3 (« exactement une ligne dans `_sqlx_migrations` ») et sa mutation (AC2). Sortie normalisée (`AUTO_INCREMENT=` volatils, commentaires horodatés — **liste exacte des normalisations écrite dans le script**, le diff du garde-fou excluant exactement cette liste, ni plus ni moins), en-tête/pied `FOREIGN_KEY_CHECKS` standard du dump CONSERVÉS (le pied restaure — vérifié sur pièces ; et les variables de session du dump ne fuient de toute façon pas vers les tests : sqlx migre sur SA connexion, le pool du test naît après — *inquiétude de lentille réfutée sur pièces en passe 3*). `--skip-triggers` posé, et le script **échoue bruyamment** si `information_schema` révèle une vue, un trigger ou une routine (« le squash ne sait pas les porter — étendre le script d'abord » : `--routines` émettrait des `DELIMITER`, directive client que le serveur rejette). Le script détecte `mariadb-dump` **ou** `mysqldump` (vérifié : seul `mysqldump` 8.4 ici, fidèle contre MariaDB 10.11 — `GENERATED`, `CHECK` nommés, collations explicites ; warning bénin).
- **Jambe structure** : un test monte les DEUX migrators sur deux bases éphémères et diffe `information_schema` — tables, colonnes (type, nullabilité, défaut, `EXTRA` généré), index, contraintes (CHECK et **actions FK `ON DELETE`/`ON UPDATE`** comprises), collations, **et vues/triggers/routines** (le schéma n'en porte aucun aujourd'hui — le diff les couvre pour que leur premier ajout ne passe pas sous le radar).
- **Jambe données d'amorçage** : le même test compare la **ligne** `_kesh_version` (id, `kesh_version_min_required`, `kesh_version_last_applied`) entre les deux bases — `information_schema` ne décrit que la structure, et c'est précisément la donnée que D1 déclare critique. *(Trou relevé en passe 1, CRITICAL : le garde-fou ne vérifiait pas ce que la spec désignait comme essentiel.)*
- Toute migration ajoutée sans régénération **rougit ce test en le disant** (« régénérez : scripts/regen-test-schema.sh »). **C'est LE mécanisme d'entretien pour les stories futures** — leur gate rougit, aucune checklist à tenir, aucune ligne d'audit supplémentaire. *(La première rédaction ajoutait un vœu « à inscrire dans la ligne d'audit P5 des stories à migration » — un vœu sans mécanisme, retiré : le garde-fou suffit et fait mieux.)*
- **Exception unique à « ne s'édite jamais »** : la mutation de preuve d'AC2 — jouée puis restaurée **par copie** avec `diff` de contrôle (jamais via git sur un arbre non commité — règle tirée deux fois dans ce cycle).

**D4 — Volet vitesse machine : MariaDB dev sur tmpfs, durabilité relâchée — la base JETABLE seulement.**

`docker-compose.dev.yml` gagne `tmpfs: /var/lib/mysql` et `command: --innodb_flush_log_at_trx_commit=0 --sync_binlog=0 --innodb-doublewrite=0`. **Conséquence assumée et documentée** : la base dev `kesh` perd sa persistance au restart du conteneur — le seed se rejoue (`docs/testing.md` le dira ; Guy a déjà pratiqué ce montage à la main pour la 14-3a, port 3307, jamais formalisé). Un oubli de re-seed est **déjà bruyant** : les 154 tests sur base partagée s'ouvrent sur `expect("need at least one company in DB for tests")` — le gate le dit en toutes lettres, pas de faux vert possible. **CI hors périmètre nominal** : `services:` GitHub Actions ne passe pas de `command:` mariadbd ; la piste `options: --tmpfs` (option de `docker create`) est un **spike optionnel non bloquant** — le gain CI vient de D1, qui s'applique partout.

**D5 — Les mesures se PUBLIENT, l'« avant » se REMESURE, et le plafond de threads ne bouge qu'APRÈS.**

Le « ~69 min » cité en ouverture est un **ordre de grandeur emprunté** au gate de convergence de la 22-1, mesuré sur SON commit — il ne vaut pas « avant » pour AC4. **T4 et T5 forment UN SEUL commit, venant après T1-T3 dans l'ordre de la branche** : le test de complétude verrouille l'état POST-bascule — posé avant elle, il serait structurellement rouge sur ~1102 attributs pas encore basculés (les graphies de chemin réel hors des 7 fichiers D2, où elles sont licites), polluant la mesure de référence et contredisant AC1 (*relevé en passe 4 : trois passes avaient laissé T5 avant la mesure*). L'AVANT se mesure donc sur le **parent direct du commit T4+T5** (`~1` — squash et garde-fou D3 posés, bascule et test de complétude pas encore), l'APRÈS sur le commit T4+T5 — deux runs qui ne diffèrent que par la bascule et ses tests, l'écart de compte étant les ajouts nommés d'AC1. **Protocole, pour que le seuil d'AC4 ne soit pas arbitrable par le bruit** *(relevé en passe 3)* : **T6 (tmpfs) vient APRÈS les deux runs** — sinon, plus le tmpfs réussit, plus le gain résiduel du squash paraît faible, et la story refuserait de fermer #251 parce que son AUTRE volet a marché ; workspace CHAUD, compilation hors chronomètre (`cargo nextest run --no-run` avant chaque mesure) ; machine au repos (`mem-guard --status` et absence d'autre charge vérifiées) ; **deux runs par état, écart publié** — si l'écart brouille le verdict au voisinage du seuil, l'arbitrage est remis à Guy, chiffres en main. Le gain de D4 se mesure ENSUITE, par-dessus, comme second étage documenté. Et **la bascule D1 vaut PARTOUT, CI comprise** (les attributs sont dans le code, `cargo test` de la CI les lit comme nextest) — seul le volet D4 (tmpfs) est dev-only. Le plafond de 6 threads n'est **pas** touché dans cette story — s'il devient débloquable, c'est une re-mesure dédiée future, consignée.

## Acceptance Criteria

**AC0 — Le transfert de couverture est NOMMÉ, et son porteur identifié.** Aujourd'hui, 1100+ tests rejouent la chaîne réelle des 61 migrations — un filet brutal contre une migration invalide. Après la bascule, l'invariant « les 61 migrations s'appliquent proprement, dans l'ordre, sur une base vide » n'est plus tenu QUE par `migrations_fresh_install.rs`, `migrations_upgrade_path.rs` et le garde-fou D3, qui monte le vrai chemin à chaque gate. Arbitrage assumé — le filet redondant à ×1100 était précisément le coût — mais il se DIT, ici et au Dev Agent Record. *(Relevé en passe 3 : un AC de décompte ne voit pas une propriété qui cesse d'être exercée.)*

**AC1 — Aucun test perdu, aucun test dégradé.** Le compte de tests exécutés après la bascule = compte avant **+ les tests que cette story ajoute elle-même** (garde-fou D3, complétude AC3 — nommés au Dev Agent Record avec leur compte). Zéro échec.
*Preuve* : les deux comptes de runs, l'écart ventilé test par test. *(« Même compte » tout court était contradictoire avec T3, qui ajoute des tests — relevé en passe 1.)*

**AC2 — Le squash est indiscernable du vrai schéma : STRUCTURE, AMORÇAGE, SUIVI.** Le garde-fou D3 (trois assertions) est vert.
*Preuve* : le test lui-même — **et ses QUATRE mutations jouées, par copie** : colonne retirée du squash → rouge nominatif ; migration ajoutée sans régénération → rouge ; ligne `_kesh_version` altérée → rouge (jambe données) ; `_sqlx_migrations` réintégrée au dump → rouge (jambe SUIVI — le mode d'échec MUET de passe 3 exige sa mutation propre). ⚠️ **La mutation « migration ajoutée » se joue contre un `DATABASE_URL` JETABLE dédié**, jamais la base dev partagée : un `.sql` temporaire appliqué par un boot concurrent à une base persistante y laisserait une ligne orpheline dans `_sqlx_migrations` — la famille « base piégée », voisine de P8. Base créée pour la preuve, détruite après, remise à zéro nommée. *(Relevé en passe 3.)*

**AC3 — La complétude de la bascule est un TEST, pas un grep.** Un test de source balaie tous les attributs `#[sqlx::test]` du **workspace** et exige : chaque attribut porte l'une des DEUX graphies de squash (`migrations = "./test-schema"` / `"../kesh-db/test-schema"`), **sauf** dans un fichier de la liste D2 **portée en dur par ce test** — et `migrations = false`, comme toute graphie de chemin réel, n'est licite QUE dans ces fichiers-là : un futur fichier ne peut pas s'auto-exempter du squash en silence, ni par un attribut nu, ni par `migrations = false`, ni par une graphie nouvelle — tout hors-liste rougit en nommant fichier et ligne. *(Resserré en passe 2 ; durci en passe 3 :)*
- **le balayage part de `CARGO_MANIFEST_DIR/../..`** — le précédent 22-4a ne balaie qu'UN crate ; repris tel quel il raterait 749 attributs en silence — avec **plancher fail-loud** : ≥ 1100 attributs vus et les 3 crates représentés, sinon « balayage suspect » ;
- **un invariant SOMMANT interne** : le compte brut d'occurrences de la chaîne `#[sqlx::test` (hors doc-comments) doit égaler le compte d'attributs parsés par le patron ancré — un attribut REPLIÉ sur plusieurs lignes casserait le patron mono-ligne en silence (rustfmt replie dès que l'attribut s'allonge) ; l'écart rougit en nommant le fichier. « Détecter, c'est chercher large » — la leçon 16-1c, appliquée au détecteur lui-même.
*Preuve* : le test, **et sa mutation jouée dans un fichier de `kesh-api`** — pas de `kesh-db` : jouée dans le crate du test, elle ne prouverait pas que le balayage franchit la frontière de crate. Un attribut re-basculé à la main vers `kesh_db::MIGRATOR` hors liste doit rougir. *(La première rédaction prouvait par un grep mono-littéral, aveugle à 4 graphies sur 5 et pollué par les doc-comments — relevé en passe 1 par deux lentilles, CRITICAL.)*

**AC4 — La mesure est publiée, et elle DÉCIDE.** Avant/après selon D5 (même machine, commits ne différant que par la bascule et ses tests — T4+T5, un seul commit), gate complet et `nextest` seul.
*Preuve* : tableau au Dev Agent Record. **Règle de décision** : si le gain sur le `nextest` seul est **< 2×**, la story ne ferme PAS #251 (la PR passe en `refs`), et consigne l'analyse de l'écart + la suite proposée — un gain décevant se documente, il ne se déclare pas victoire.

**AC5 — Les compteurs périmés sont rafraîchis, aux TROIS sites nommés.** `.config/nextest.toml` (~894/51 → réels datés, et son commentaire de plafond renvoie à la re-mesure D5), les deux commentaires CI « 84 tests » (→ 154), **`CLAUDE.md` § Plafonds mémoire** (~894/51 → réels).
*Preuve* : `grep -rnE '\b894\b|\b84 tests\b'` → zéro résidu hors sites LÉGITIMES (`notes-251-exploration.md`, stories historiques datées) — le grep les rend aussi, ils se trient à la main : c'est le prix, et il est bas.

**AC6 — Le volet tmpfs est actif en dev et DIT.** `docker-compose.dev.yml` porte tmpfs + flags ; `docs/testing.md` documente la non-persistance et le re-seed.
*Preuve* : ~~`docker inspect kesh-mariadb-dev --format '{{json .Mounts}}' | grep -o '"Type":"tmpfs"'`~~ — **cette commande est FAUSSE et rend zéro occurrence** : un tmpfs déclaré par compose atterrit dans `.HostConfig.Tmpfs`, pas dans `.Mounts`, qui ne liste que bind mounts et volumes. Elle avait pourtant été « vérifiée exacte » en passe 2 de validate — contre rien, puisque le tmpfs n'existait pas encore : *une commande relue n'est pas une commande exécutée.* Les preuves réelles, jouées le 2026-08-16 :

```sh
docker inspect kesh-mariadb-dev --format '{{json .HostConfig.Tmpfs}}'
#   {"/var/lib/mysql":"size=4g,mode=1777"}
docker exec kesh-mariadb-dev sh -c 'grep " /var/lib/mysql " /proc/mounts'
#   tmpfs /var/lib/mysql tmpfs rw,nosuid,nodev,noexec,relatime,size=4194304k,inode64 0 0
```

et la section `docs/testing.md` § *Base de dev jetable* existe.

## Tasks / Subtasks

- [x] **T1 — Script de régénération + squash initial** (D3, AC2). `scripts/regen-test-schema.sh` : détection `mariadb-dump`/`mysqldump`, `--no-data`, **exclusion `_sqlx_migrations`**, normalisation (`AUTO_INCREMENT=`, horodatages), en-tête `FOREIGN_KEY_CHECKS=0`, réinjection de la ligne `_kesh_version`. `crates/kesh-db/test-schema/0001_schema_squash.sql` versionné.
- [x] **T2 — Le répertoire `crates/kesh-db/test-schema/`** (D1, AC2, AC3). `0001_schema_squash.sql` (produit par T1) + un `README.md` de trois lignes : « se régénère (`scripts/regen-test-schema.sh`), ne s'édite JAMAIS », la note P8 (la migration synthétique ne vit qu'en bases ÉPHÉMÈRES — toujours recréées à neuf par sqlx, `drop database if exists` avant `create` : aucun checksum persistant ne la rencontre, *vérifié sur pièces en passe 3*), et le renvoi à cette story. *(Le `static TEST_MIGRATOR` de la première rédaction est ABANDONNÉ — cf. D1 révisé.)*
- [x] **T3 — Garde-fou anti-dérive** (D3, AC2). Fichier **`crates/kesh-db/tests/test_schema_guard.rs`** — INSCRIT à la liste D2, il monte le vrai chemin, c'est sa fonction —, trois assertions : `squash_matches_real_schema_structure` (diff `information_schema` complet — vues/triggers/routines/actions FK compris), `squash_seeds_the_kesh_version_row` (jambe données, valeurs comparées à la base réelle migrée) et `squash_database_tracks_exactly_one_migration` (jambe SUIVI — le mode d'échec muet de `_sqlx_migrations`). **La seconde base** (vrai `MIGRATOR`) se crée à la main : nom unique (préfixe réservé + pid), `DROP DATABASE IF EXISTS` en tête ET destruction garantie même en panic — sqlx ne nettoie que les tests verts. Messages actionnables (« régénérez : scripts/regen-test-schema.sh »). **Quatre mutations jouées, par copie** (scratchpad, restauration, `diff -q`).
- [x] **T4 — Les graphies de chemin réel : quatre basculent, la cinquième s'explicite — MÊME COMMIT que T5** (D1, D2, AC1, AC3, AC4). Les populations du recensement, selon leur colonne « Sort » ; l'attribut nu d'`accounts_role_backfill.rs` explicité en `migrator = "kesh_db::MIGRATOR"` — PAS `migrations = false`, qui viderait sa base (un attribut nu applique TOUT le migrator). **La mesure AVANT se prend au commit précédant immédiatement cette bascule, l'APRÈS au commit de la bascule** (D5).
- [x] **T5 — Test de complétude d'AC3 — MÊME COMMIT que T4** (AC3, cf. D5 : posé avant la bascule, il serait rouge par construction). Même fichier `test_schema_guard.rs`, fonction `every_sqlx_test_attribute_is_accounted_for` : balayage de source par attributs ancrés (le patron du dispositif de la 22-4a, appliqué au harnais), liste D2 en dur, `migrations = false` restreint à cette liste, mutation jouée.
- [x] **T6 — tmpfs + durabilité dev** (D4, AC6). Compose : **retirer le volume nommé `kesh-mariadb-data`** (ses DEUX lignes — montage et déclaration ; un `tmpfs:` posé par-dessus donnerait « Duplicate mount point », le conteneur ne démarrerait pas) et poser le `tmpfs` avec **taille explicite** (`size=` — défaut 50 % de la RAM hôte, sur la station aux deux OOM documentés ; les bases éphémères orphelines coûteront de la RAM, et le restart les efface — un avantage à écrire). `docs/testing.md` : non-persistance, re-seed, balayage des orphelines. ⚠️ **Gestes destructeurs et sensibles au moment** : la suppression du volume DÉTRUIT la base dev `kesh` de Guy (le prévenir AU MOMENT de T6) ; le restart tue tout gate en vol ; et **T6 s'exécute APRÈS les deux runs de mesure** — cf. D5. *(Montage dupliqué, taille et ordre relevés en passe 3.)*
- [x] **T7 — Compteurs rafraîchis aux trois sites** (AC5). `nextest.toml`, commentaire CI, `CLAUDE.md` § Plafonds mémoire — avec dates.
- [x] **T8 — Mesures publiées et règle de décision appliquée** (D5, AC4). Tableau avant/après ; verdict fermeture (#251 `closes` ou `refs`) motivé par le seuil d'AC4 ; plafond de threads intouché, renvoi explicite à une re-mesure future.

## Dev Notes

### Références (chaque affirmation sourcée à la ligne)

- **`notes-251-exploration.md`** (commit `e0e91eac`) — mécanique sqlx 0.8.6 (fichiers:lignes du registry), contraintes dures, état compose/CI. ⚠️ Ses comptages datent d'avant le merge 22-1 : **le recensement qui fait foi est celui de CETTE spec**, recompté par attributs ancrés.
- Issue **#251** ; `.config/nextest.toml` (plafond 6, historique 32→cassé) ; § *Plafonds mémoire* et § *Test Locally First* du `CLAUDE.md`.
- Constructibilité `Migration` : `sqlx-core-0.8.6/src/migrate/migration.rs:7-36` ; chemin d'attribut arbitraire : `sqlx-macros-core-0.8.6/src/test_attr.rs:278-295`. Précédent sub-`Migrator` : `crates/kesh-db/tests/common/mod.rs:50-83` *(sous-ensemble du vrai migrator — le type autorise la migration synthétique, mais ce précédent n'en est pas une)*.
- Précédent tmpfs manuel de Guy (14-3a, port 3307) : `14-3a-socle-roles-comptes.md:266,485,526`.
- Dump vérifié sur pièces (passe 1, lecture seule contre la base dev migrée) : `GENERATED ALWAYS … VIRTUAL`, `CHECK` nommés et `utf8mb4_bin` explicites **survivent** au dump — la matière du garde-fou D3 est réelle.

### Les pièges nommés d'avance

- **`_sqlx_migrations` HORS du dump** — sqlx crée la sienne ; l'inclure casse les 1000+ tests basculés d'un coup (CRITICAL de passe 1).
- **La ligne `_kesh_version` est une DONNÉE** : ni `--no-data` ni `information_schema` ne la voient — d'où la réinjection (T1) et la jambe données du garde-fou (T3).
- **Le dump doit être NORMALISÉ** (`AUTO_INCREMENT=` volatils, commentaires horodatés) et **encadré de `FOREIGN_KEY_CHECKS=0`** (ordre alphabétique ≠ ordre FK).
- **Cinq graphies, pas une** : tout motif de substitution mono-littéral rate 4 populations sur 5 — c'est le test d'AC3 qui tient la complétude, pas un sed.
- **Ne pas basculer** les 17 `migrations = false` ni la liste D2.
- **L'« avant » d'AC4 se mesure au commit précédant la bascule**, pas au chiffre d'une autre story (le « 69 min » de la 22-1 est un ordre de grandeur, pas une référence).

## Dérogation règle de splitting

Le critère de non-convergence de la § *Règle de splitting préventif* a été atteint en passe 3 de validate (P2 `0/0/2/~5` → P3 `1/4/12/6`). **Arbitrage de Guy, 2026-08-15 : pas de split, poursuite de la boucle.** Motif : les patches des passes 1-2 ont tous tenu (vérifié au grep par l'audit de passe 3) ; la hausse de sévérité vient de l'escalade DÉLIBÉRÉE vers une lentille d'architecture Opus, pas d'une remédiation qui n'entame pas le problème ; et chaque finding est fermable par amendement concret — rien n'énumère sans fin comme l'AC6 de la 22-4. *(L'exception codifiée de la règle vise les cycles Cargo d'un split forcé — le cas présent est l'arbitrage du Project Lead que l'amendement D-C prévoit ; la règle exigeant une section dédiée, la voici — relevé en passe 4.)*

## Dev Agent Record

**État d'exécution au 2026-08-16** : **T1 à T8 faites, AC0 à AC6 satisfaites.** T6 a été exécutée en dernier, comme D5 l'impose (après les deux runs de mesure), et sur arbitrage explicite de Guy — c'est un geste destructeur pour les bases de dev.

| Commit | Contenu |
|---|---|
| `d5f0cf73` | T1-T3 — `scripts/regen-test-schema.sh`, `test-schema/0001_schema_squash.sql` (38 tables, 67 Ko), garde-fou à 4 assertions, 4 mutations jouées |
| `76fb8e92` | T4+T5 — 1102 attributs basculés, test de complétude, propagation de 17 sites de doc |
| celui-ci | T7-T8 — compteurs rafraîchis, mesures publiées, verdict de fermeture |

**AC0 — le transfert de couverture, nommé.** Avant cette story, 1102 tests rejouaient la chaîne réelle des 61 migrations : un filet redondant, et c'était exactement son coût. Depuis la bascule, l'invariant « les 61 migrations s'appliquent proprement, dans l'ordre, sur une base vide » n'est plus tenu que par `migrations_fresh_install.rs`, `migrations_upgrade_path.rs`, les backfills à fenêtre de la liste D2, et le garde-fou `test_schema_guard.rs`, qui monte le vrai `MIGRATOR` à chaque gate. Arbitrage assumé, et dit ici comme l'AC l'exige.

**AC1 — aucun test perdu, l'écart ventilé nominativement.** 2208 → **2209** tests, **0 échec des deux côtés**, 4 skipped inchangés, aucun `SLOW`, aucun retry. La preuve n'est pas le total mais le **diff des listes de tests exécutés**, extraites des deux logs, triées et comparées : **zéro disparu**, **un seul apparu** — `kesh-db::test_schema_guard every_sqlx_test_attribute_is_accounted_for`, précisément le test que T5 ajoute. Les deux runs APRÈS rendent des listes identiques. Les 4 tests du garde-fou de T3 étaient déjà dans le compte AVANT, leur commit étant le parent de la mesure. *(Un +1 net peut masquer une suppression compensée — mode d'échec payé sur 16-1a et 16-2a ; le diff nominatif est ce qui l'exclut.)*

**AC2 — garde-fou vert, 4 assertions.** Structure (`information_schema` complet), ligne d'amorçage `_kesh_version`, suivi `_sqlx_migrations`, et la quatrième née d'une mutation restée verte (le `MIGRATOR` compilé peut ignorer une migration ajoutée — `sqlx::migrate!` est compile-time). 4 mutations jouées, 4 rougissements — déclaré au commit `d5f0cf73`. Les 3 assertions à base réelle ont tourné dans les deux runs de mesure.

**AC3 — la complétude est un test, et sa mutation a été jouée.** Recensement recompté depuis la source au commit de bascule, ventilation qui somme :

| Graphie | Compte |
|---|---|
| `migrations = "../kesh-db/test-schema"` | 749 |
| `migrations = "./test-schema"` | 353 |
| `migrator = "kesh_db::MIGRATOR"` (liste D2) | 26 |
| `migrations = false` (liste D2) | 17 |
| **Total** | **1145** = les 1142 du recensement de spec + les 3 attributs du garde-fou livré en T3 |

Mutation jouée le 2026-08-16 : un attribut de `crates/kesh-api/tests/accounts_e2e.rs` re-basculé à la main vers `migrator = "kesh_db::MIGRATOR"` → le test **rougit en nommant `crates/kesh-api/tests/accounts_e2e.rs:211`**, depuis un binaire de test de `kesh-db` : la frontière de crate est bien franchie. Restauration **par copie**, vérifiée deux fois (`diff -q` contre la référence, puis `git status` propre vis-à-vis de `HEAD`), test re-vert.

**AC4 — la mesure, et le verdict qu'elle dicte.** Protocole D5 : même machine, profil `ci`, compilation hors chronomètre, deux runs par état, `mem-guard` actif, station au repos. L'AVANT sur `d5f0cf73`, l'APRÈS sur son enfant `76fb8e92` — les deux commits ne diffèrent que par la bascule et son test.

| | run 1 | run 2 | moyenne | résultat |
|---|---|---|---|---|
| **AVANT** (`d5f0cf73`) | 3677 s | 4051 s | **64,4 min** | 2208 passed, 0 failed, 4 skipped |
| **APRÈS** (`76fb8e92`) | 1266 s | 1111 s | **19,8 min** | 2209 passed, 0 failed, 4 skipped |

**Gain : 3,25×** sur les moyennes — 3,31× meilleur contre meilleur, 3,20× pire contre pire, et **2,90× dans la lecture la plus défavorable constructible** (l'AVANT le plus rapide contre l'APRÈS le plus lent). Par test : 1,75 s → 0,54 s. Économie : **44,6 min par gate complet**.

**Verdict d'AC4 : le seuil de 2× est franchi même par la lecture la moins favorable → la PR ferme #251** (`closes`, porté sur le message de PR et non sur les commits intermédiaires, cf. § *Issue Tracking Rule*).

⚠️ **Le bruit de mesure est réel et se dit** : 10 % d'écart entre les deux runs AVANT, 14 % entre les deux APRÈS (et le run 1 y est plus lent que le run 2, l'inverse de l'AVANT). Il ne menace pas la conclusion — l'intervalle le plus défavorable reste à 2,90× — mais il interdit d'annoncer « 3,25× » comme une constante.

**Plafond de threads : intouché, et pourquoi.** Le `test-threads = 6` a été fixé par la contention des metadata-locks sur le rejeu des migrations — cause que cette story vient de supprimer. Il est donc vraisemblablement trop bas aujourd'hui. Il **reste à 6 faute de mesure, pas par conviction** : le relever exige de rejouer le tableau flakes compris (KF-038 #228), ce qui est un travail de mesure à part entière. L'avertissement est écrit dans `.config/nextest.toml` et dans le `CLAUDE.md`, aux deux endroits où quelqu'un ira lire le chiffre.

**AC5 — compteurs rafraîchis aux trois sites**, avec dates :

- `.config/nextest.toml` — le tableau de 2026-07-13 est **étiqueté historique** (« AVANT le squash ») au lieu d'être effacé, le paragraphe « ~894 tests / 51 migrations » est remplacé par l'état réel (1102 des 1145 attributs sur le squash, 43 en liste D2) et par les mesures datées du 2026-08-16 ; le commentaire du plafond renvoie à la re-mesure.
- `.github/workflows/ci.yml:124` et `:130` — « 84 tests » → **154**, recompté depuis la source et ventilé : 151 tests dont le corps appelle `test_pool()` (24 accounts + 3 audit_log + 33 contacts + 37 invoices + 34 journal_entries + 20 products) **+ 3** qui ouvrent leur propre connexion (2 dans `invoices.rs`, 1 dans `retry.rs`). La graphie de remédiation citée y est corrigée en `migrations = "./test-schema"`.
- `CLAUDE.md` § *Gate rapide* (le site que la spec désignait « § Plafonds mémoire, ligne 114 ») — le « 1,40× / ~894 / 51 migrations » devient l'histoire au passé, suivi du tableau de mesure et de l'avertissement sur le plafond ; la ligne `test-threads` du tableau des parallélismes porte le même renvoi.

*Preuve* : `grep -rnE '\b894\b|\b84 tests\b|51 migrations'` sur le dépôt ne rend plus que des sites **légitimes** — `notes-251-exploration.md`, la spec de cette story qui les nomme, et des stories historiques datées (6-4, 3-4, 7-2) dont les décomptes appartiennent à leur époque. Aucun résidu vivant.

**AC6 — satisfaite le 2026-08-16, après arbitrage de Guy.** `docker-compose.dev.yml` monte `/var/lib/mysql` en tmpfs de 4 Go (taille explicite : le défaut est la moitié de la RAM hôte, soit 15 Go sur cette station) et porte les trois flags de D4 ; `docs/testing.md` gagne une section § *Base de dev jetable*. Vérifié en jouant la procédure documentée **telle qu'écrite**, après un `restart` qui a bien remis la base à zéro (0 table dans `kesh`) : 61 migrations rejouées par `sqlx migrate run`, seed appliqué, **283/283 tests de la base partagée verts en 3,1 s**, 0 base éphémère orpheline. Les flags sont confirmés en base (`innodb_flush_log_at_trx_commit=0`, `sync_binlog=0`, `innodb_doublewrite=0`).

⚠️ **Trois choses que la spec n'avait pas vues, et qui ont dû être traitées** :

1. **La commande de preuve d'AC6 était fausse** — cf. l'AC corrigé plus haut. Un tmpfs de compose vit dans `.HostConfig.Tmpfs`, jamais dans `.Mounts`.
2. **`docker compose up --force-recreate` a conservé l'ancien volume nommé**, pourtant retiré du fichier : le conteneur recréé le portait encore dans `.Mounts`, le tmpfs monté par-dessus. Inerte sur le moment, mais c'est un piège différé — retirer un jour la ligne tmpfs aurait fait silencieusement réapparaître d'anciennes données. Corrigé par un `down` + `up` complet, puis suppression du volume ; `.Mounts` est désormais `[]`.
3. **Le tmpfs efface aussi les tables SYSTÈME**, donc les droits globaux de l'utilisateur `kesh` — ceux que le README de `kesh-db` documente comme étape d'installation manuelle. Sans eux, `#[sqlx::test]` ne peut plus créer ses bases éphémères et **toute** la suite d'intégration tombe, à chaque redémarrage. Traité par `scripts/mariadb-init/01-dev-grants.sql`, monté dans `/docker-entrypoint-initdb.d/` : l'entrypoint MariaDB le rejoue dès que le datadir est vide, ce qui ici arrive à tous les coups. Vérifié par un vrai redémarrage, pas par lecture. Le seed applicatif, lui, reste manuel (`scripts/seed-dev-db.sql`) — son oubli est bruyant par construction.

La base de dev de Guy et les bases de gate ont été détruites comme prévu ; `steadyinvest_test`, base d'un autre projet présente dans le même volume, a été signalée avant destruction et sacrifiée sur son arbitrage explicite.

**Gates réellement exécutés** — et rien d'autre n'est déclaré :

- sur `76fb8e92` : `cargo fmt --all -- --check` vert, `cargo clippy --workspace --all-targets -- -D warnings` vert (exit 0), puis **le gate complet deux fois** (`cargo nextest run --profile ci`), 2209/2209.
- après T7 : les modifications ne portent que sur des **commentaires** (`nextest.toml`, `ci.yml`) et de la documentation (`CLAUDE.md`) — aucun code exécutable, aucun réglage de comportement changé. Le gate complet n'a pas été rejoué pour elles, et ne l'est pas déclaré.

## Change Log

**2026-08-15 — `bmad-create-story validate`, PASSE 1 (Sonnet ×3, contextes frais : aveugle, ground-truth, audit).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Aveugle (spec seule) | 2 | 4 | 4 | 3 |
| Ground-truth (spec vs sources/registry/DB réelle) | 0 | 1 | 2 | 1 |
| Audit (checklist + CLAUDE.md + issue) | 2 | 0 | 2 | 3 |
| **dédupliqué** | **4** | **4** | **8** | **5** |

*(Le tableau a d'abord annoncé « 6 MED » pour une puce qui en énumérait 8 — l'incohérence a été relevée en passe 2 par l'audit, et le recomptage donne bien 8. Le compteur de compteurs n'est pas exempté de la règle.)*

Déduplication notable : les graphies multiples d'attribut, vues par les TROIS lentilles sous trois angles (grep aveugle au grain, 44 tests invisibles, 12 `crate::MIGRATOR` dans le total même) → **un** CRITICAL, fermé par la refonte d'AC3 en test de complétude. Les deux lentilles se CONTREDISAIENT sur les décomptes (audit : « périmés, 1182 » ; ground-truth : « exacts une fois ancrés, 1142 ») — **tranché par recomptage de l'orchestrateur** : 1142 exact, ventilation à six populations qui somme juste ; le grep naïf comptait les doc-comments.

- **CRIT 1 — le dump embarquait `_sqlx_migrations`** : sqlx crée la sienne avant le migrator — collision « table already exists » sur la totalité des tests basculés. T1 exclut la table, piège nommé.
- **CRIT 2 — le garde-fou ne pouvait pas vérifier la ligne `_kesh_version`** : `information_schema` ne décrit que la structure, et c'est une DONNÉE que D1 déclare critique. Le garde-fou gagne une jambe données (+ mutation dédiée en AC2).
- **CRIT 3 — les décomptes étaient recopiés de l'exploration, pas recomptés** — sur une story dont le sujet EST un compteur de tests. Tableau refait au commit de la passe, méthode ancrée ÉCRITE dans la spec, ventilation sommante exigée.
- **CRIT 4 — cinq graphies atteignent les mêmes 61 migrations** (1080 + 23 + 12 + 9 + 1 nu), la substitution et sa preuve n'en voyaient qu'une. AC3 refondu : la complétude est un TEST fail-loud (liste D2 en dur, toute graphie nouvelle rougit) — la philosophie du dispositif de la 22-4a, appliquée au harnais de test.
- **4 HIGH** : AC1 « même compte » contradictoire avec T3 (reformulé : aucun test perdu, ajouts nommés) · grain fichier/attribut de D2 (résolu par le test d'AC3) · `migrations_upgrade_path` invisible au grep (idem) · tableau non sommant (recompté).
- **8 MED** : « le seul INSERT » réfuté (7 INSERT sur 3 migrations — seul `_kesh_version` à réinjecter, dit avec le vrai compte) · « 69 min » était la mesure d'une AUTRE story sur un AUTRE commit (rétrogradé en ordre de grandeur, l'« avant » se remesure — D5/T4) · exception de mutation à « ne s'édite jamais » écrite · périmètre du diff élargi (vues/triggers/routines/actions FK) · script robuste (mariadb-dump/mysqldump, FK_CHECKS=0) · seuil de décision d'AC4 (< 2× sur nextest seul → la PR ne ferme pas #251) · le « vœu P5 » remplacé par le vrai mécanisme (le garde-fou D3 EST l'entretien) · `CLAUDE.md` § Plafonds mémoire ajouté aux sites d'AC5.
- **5 LOW** : commande d'inspection d'AC6 explicitée · rationale du plafond de threads dans nextest.toml (renvoi D5) · emplacement de la note P8 (doc-comment de TEST_MIGRATOR) · fallback client de dump vérifié sur pièces · précédent `common/mod.rs` requalifié (sous-ensemble, pas synthétique).

**Vérifié conforme par le ground-truth, et c'est porteur** : `Migration` constructible hors crate (champs publics, `Migration::new`) et chemin d'attribut arbitraire accepté — **D1 est faisable sur pièces** ; le dump réel (client MySQL 8.4 contre MariaDB 10.11) rend fidèlement colonnes générées, CHECK nommés et collations explicites ; la liste D2 est complète (grep exhaustif des patrons de fenêtre) ; compose dev vierge de tout réglage, conforme aux notes.

**Patches appliqués (réécriture complète de la spec), prochaine passe : Haiku, contexte frais.**

**2026-08-15 — `bmad-create-story validate`, PASSE 2 (Haiku ×3, contextes frais).**

| Lentille | retenu après triage |
|---|---|
| Aveugle | **2 MED** réels + LOW de formulation — et 1 « CRITICAL » d'arithmétique RÉFUTÉ de tête (sa propre somme oubliait la ligne de l'attribut nu : 1080+23+17+12+9+1 = 1142, exacte) ; 2 « CRITICAL » rétrogradés en points de nommage/formulation |
| Ground-truth | **0** — le bloc python de la spec exécuté TEL QUEL rend 1142 ; chaque population, chaque fichier, les 7 INSERT (4/1/2), les sites périmés (CLAUDE.md:114, CI ×2) et la commande docker d'AC6 vérifiés exacts ; les éléments « T6 futur » correctement reconnus comme travail à venir |
| Audit | **0** > LOW — les 4 CRITICAL de passe 1 vérifiés intégrés au grep, AC toutes exécutables, chaînage complet ; 1 LOW réel : le « 6 MED » de passe 1 en énumérait 8 |

**Patchés en passe 2** : `migrations = false` restreint aux fichiers D2 dans le test d'AC3 (un futur fichier ne peut plus s'auto-exempter en silence — le vrai MED de la passe) · les artefacts de test NOMMÉS (`crates/kesh-db/tests/test_schema_guard.rs`, trois fonctions) · le timing d'AC4 rendu littéral (avant = parent direct du commit T4) · la bascule D1 dite valable en CI aussi (seul D4 est dev-only) · « cinq graphies / six populations » désambiguïsé · l'attribut nu dit « reste exclu » · le re-seed oublié dit déjà bruyant (`expect` nominatif des 154 tests partagés) · le tableau de passe 1 corrigé (6→8 MED, avec son aveu).

**Trend : passe 1 `4/4/8/5` → passe 2 `0/0/2/~5`, les 2 MED patchés. Une passe 3 s'impose** (des MED ont été relevés — rotation : Opus).

**2026-08-15 — `bmad-create-story validate`, PASSE 3 (Opus ×3, contextes frais — la passe d'architecture).**

| Lentille | CRIT | HIGH | MED | LOW | notes |
|---|---|---|---|---|---|
| Aveugle | 1 | 6 | 6 | 3 | dont 2 findings majeurs RÉFUTÉS ensuite sur pièces par le ground-truth |
| Ground-truth (sources sqlx, dump réel joué, DB réelle) | 1 | 1 | 7 | 2 | réfute aussi le MÉCANISME de mon CRIT de passe 1 |
| Audit | 0 | 0 | 4 | 6 | + signal de non-convergence (§ Règle de splitting) |
| **dédupliqué retenu** | **1** | **4** | **~12** | **~6** | après réfutations croisées |

**Les réfutations d'abord, parce qu'elles enseignent** : la « fuite de session `FOREIGN_KEY_CHECKS` vers le pool » (CRIT aveugle) n'existe pas — sqlx migre sur SA connexion, le pool du test naît après, et le pied du dump restaure (vérifié sur pièces) ; la « collision de bases inter-branches » non plus — `do_cleanup` droppe avant chaque `create`, `VersionMismatch` inatteignable ; et mon CRIT de passe 1 avait le bon remède pour le MAUVAIS mécanisme — inclure `_sqlx_migrations` échoue en **silence** (`--add-drop-table` détruit la ligne de suivi, l'`UPDATE` affecte 0 ligne sans erreur), pas en collision : d'où l'assertion et la mutation dédiées.

**Le CRITICAL retenu** : « expliciter l'attribut nu en `migrations = false` » — écrit à TROIS sites — aurait VIDÉ la base du test concerné : un attribut nu applique TOUT le migrator (`InferredPath`), il n'en désactive aucun. Corrigé aux trois sites en `migrator = "kesh_db::MIGRATOR"`.

**La révision de conception** : D1 abandonne le `static TEST_MIGRATOR` pour **le répertoire `test-schema/`** — l'option 1 de l'issue elle-même, que j'avais écartée sans motif (l'audit l'a relevé en LOW, le ground-truth a fourni les pièces : `Migration::new` non-`const`, ~70 Ko de DDL embarqués dans la rlib de production). Zéro code Rust, la macro gère checksum et embarquement, AC3 énumère les deux graphies licites.

**Les 4 HIGH fermés** : le garde-fou inscrit à sa PROPRE liste D2 (sans quoi la story ne satisfaisait pas son propre AC3) · balayage d'AC3 étendu au workspace, plancher fail-loud, invariant SOMMANT interne (l'attribut replié ne contourne plus le patron mono-ligne), mutation jouée dans kesh-api · protocole de mesure d'AC4 (T6 APRÈS les runs, workspace chaud, machine au repos, deux runs par état, arbitrage Guy si le bruit brouille) · la mutation « migration ajoutée » sur `DATABASE_URL` jetable dédié (la base dev ne peut plus être briquée).

**Les 12 MED fermés** : le mécanisme muet de `_sqlx_migrations` requalifié + assertion + mutation dédiées · la révision D1 elle-même (static → répertoire, sur les deux pièces du ground-truth) · la ligne `_kesh_version` SELECTée de la base migrée (`0.10.0` réel, pas l'amorçage `0.1.0` — un littéral en dur aurait trahi D3 à chaque bump P2) · `--skip-triggers` + échec bruyant sur vue/trigger/routine (le `DELIMITER` d'un dump `--routines` est une directive client) · seconde base du garde-fou : nom unique + destruction panic-safe · T6 : volume nommé `kesh-mariadb-data` retiré (Duplicate mount point sinon), destruction de la base dev DÉCLARÉE · AC0 créé (le transfert de couverture est nommé, son porteur identifié) · ventilation du « 17 » corrigée sur sa ligne · la liste D2 lisible par motifs par fichier (la sémantique « exerce le chemin réel » tient, les `migrations = false` étant tous des testeurs de fenêtre) · la colonne « Sort » des 12 internes rendue SANS OBJET par la révision D1 (le répertoire n'a pas de graphie de crate) · la liste EXACTE des normalisations du dump écrite dans le script, le diff du garde-fou excluant exactement cette liste · le tmpfs dimensionné (`size=`) et les orphelines documentées.

*(Cette liste a été comptée fausse DEUX fois — 11 items mixtes en passe 3, 9 sous l'étiquette « 12 » après la restructuration de passe 4 — et les affirmations dérivées avec elle : le « restructuré en trois listes qui somment » de la passe 4 ne sommait pas, le « 22 distincts pour 23 » de la passe 5 comptait une liste de 12 qui n'existait pas (le réel d'alors : 20 items dont un doublon). Les 3 MED manquants sont restitués ci-dessus depuis le triage source de la passe 3 ; c'est la passe 6 qui a exigé le compte juste. QUATRE listes, TROIS recomptages faux : la leçon est que la ventilation s'écrit EN MÊME TEMPS que le total, jamais après lui.)*

**Et le ground-truth a vérifié conforme, sur pièces, tout le reste de l'architecture** : aucun lecteur de `_sqlx_migrations`/du nombre de migrations hors D2 · le boot réel (downgrade, record, backfill D6) intégralement en D2 · kesh-api/kesh-report dépendent bien de kesh-db · `GET_LOCK` sans effet de bord · dump réel fidèle (40 CREATE TABLE, 4 FULLTEXT, 3 GENERATED, 43 actions FK, CHECK nommés, collations) · aucune macro `sqlx::query!` dans le dépôt.

⚠️ **Signal de la § Règle de splitting préventif : atteint** (P2 `0/0/2` → P3 sévérité en hausse). **Arbitrage de Guy (2026-08-15) : PAS de split, passe 4 après patches** — diagnostic partagé : les patches des passes 1-2 ont tous tenu (vérifié au grep par l'audit), la hausse vient de l'escalade DÉLIBÉRÉE vers une lentille d'architecture, et chaque finding est fermable par amendement concret — rien n'énumère sans fin comme l'AC6 de la 22-4. La dérogation est consignée ici, comme la règle le demande.

**Les ~6 LOW** : « deux lentilles » → « trois » (l'arithmétique des fusions tranchait) · les deux commentaires CI nommés (`ci.yml:124`, `:130`) · le grep d'AC5 et ses occurrences légitimes à trier · `locking`/checksum du static — rendus SANS OBJET par la révision D1 · l'en-tête `sql_mode` du dump conservé tel quel (le diff D3 le couvre) · la « voie de secours » vers le vrai migrator ÉCARTÉE avec motif : le garde-fou D3 monte le vrai chemin à chaque gate, c'est déjà le commutateur.

**Trend : P1 `4/4/8/5` → P2 `0/0/2/~5` → P3 `1/4/12/6` → passe 4 (rotation : Sonnet, orientation ground-truth sur les patches de cette passe).**

**2026-08-15 — `bmad-create-story validate`, PASSE 4 (Sonnet ×2 : ground-truth de contrôle + audit de cohérence).**

| Lentille | retenu |
|---|---|
| Ground-truth (sources sqlx rejouées, dump réel sondé) | **1 MED + 1 LOW** *(le LOW : « ~70 Ko » plausible — 68 319 octets re-sondés — mais non re-vérifiable sur pièces, l'artefact n'existant pas encore)* — et tout le reste vérifié conforme SUR PIÈCES : la voie du répertoire tient jusque dans le parseur de noms (`0001_` accepté, résolution au `CARGO_MANIFEST_DIR` du crate du test), le triplet `_kesh_version` exact (`0.10.0`/`0.1.0`/NULL), les trois sites de l'attribut nu corrigés, zéro reliquat de livrable `TEST_MIGRATOR`, ~70 Ko plausible (68 319 octets re-sondés) |
| Audit de cohérence | **4 MED** de forme, tous réels |

- **Le MED du ground-truth, que TROIS passes avaient raté** : au commit « AVANT » de la mesure, le test de complétude (T5, posé avant la bascule) aurait été **structurellement rouge** sur ~1102 attributs (1124 graphies de chemin réel, moins les 22 licites en D2) — polluant le run de référence et contredisant AC1. Fermé : **T4 et T5 forment un seul commit**, l'AVANT se mesure sur son parent.
- **Les 4 MED de l'audit** : le Change Log de passe 3 ne ventilait pas son `4/12/6` (restructuré en trois listes étiquetées qui somment — la § *Recompter* me reprend pour la TROISIÈME fois du cycle, sur l'artefact qui la cite) · « jambe amorçage » collée par erreur sur la jambe SUIVI aux deux sites qui guident l'implémentation (T3, AC2 — corrigés) · le résidu `schema-squash.sql` sans préfixe ni chemin en T1 (aligné) · la dérogation de splitting sortie du Change Log vers **sa section dédiée**, comme la règle l'exige littéralement.

**Trend : P3 `1/4/12/6` → P4 `0/0/5/1`. Encore au-dessus du seuil — passe 5 (rotation : Haiku), sur un lot désormais mince.**

**2026-08-15 — `bmad-create-story validate`, PASSE 5 (Haiku ×2 : aveugle de cohérence + audit de recompte).**

Retenu : **2 MED, 1 LOW** — tous sur mes comptes rendus, aucun sur le fond. Le MED de l'audit : ma restructuration de passe 4 avait DUPLIQUÉ l'item « deux lentilles → trois » dans les 12 MED ET les 6 LOW de la passe 3 (22 items distincts pour 23 déclarés) — retiré des MED, remplacé par l'item réellement manquant (la sémantique de la liste D2). Le MED de l'aveugle : le trend `0/0/5/1` de la passe 4 portait un LOW jamais étiqueté — c'était le « ~70 Ko plausible » du ground-truth, désormais étiqueté dans son tableau. Le LOW : la dérogation citait le trend P3 tronqué (`/6` ajouté). **Un finding réfuté au grep** : « la dérogation n'est pas tracée au Change Log de P4 » — elle l'est, quatrième item de la liste des 4 MED.

La § *Recompter ses propres comptes rendus* m'aura repris QUATRE fois sur ce seul cycle — chaque fois sur un artefact qui la cite. Le fond de la spec, lui, n'a plus bougé depuis la passe 4.

**Trend : P4 `0/0/5/1` → P5 `0/0/2/1`. Passe 6 (rotation : Opus, lentille unique de recompte final).**

**2026-08-15 — `bmad-create-story validate`, PASSE 6 (Opus, lentille unique de recompte final).**

Retenu : **1 MED, 4 LOW** — le fond (D1-D5, AC0-AC6, T1-T8) n'a produit **aucune contradiction frontale**, tout le vérifiable a été revérifié conforme sur pièces (recensement, ventilations, 7 INSERT, 154, 749, sites d'AC5, patches de passe 5). Le MED : **la liste « 12 MED » de la passe 3 n'a JAMAIS sommé** — 11 items mixtes d'abord, 9 sous l'étiquette « 12 » après ma « restructuration » de passe 4, dont l'affirmation « qui somment » était elle-même fausse, comme le « 22 distincts pour 23 » de la passe 5 qui en dérivait. Les 3 MED manquants restitués depuis le triage source, les deux affirmations dérivées rectifiées en place, et la leçon écrite : **la ventilation s'écrit EN MÊME TEMPS que le total, jamais après lui.** Les 4 LOW propagés : trend P2 complété (`/~5`) aux deux sites · AC4 aligné sur la fusion T4+T5 · « ~1124 » corrigé en ~1102 (les 22 licites en D2 ne rougissent pas) · le titre de T4 dit désormais « quatre basculent, la cinquième s'explicite ».

**Trend : P4 `0/0/5/1` → P5 `0/0/2/1` → P6 `0/0/1/4`. Passe 7 (rotation : Sonnet, lentille unique) — mandat : recompter la SEULE liste corrigée et ses dérivées, rien d'autre.**

**2026-08-15 — `bmad-create-story validate`, PASSE 7 (Sonnet, lentille unique, mandat d'une liste).**

Retenu : **1 MED** — la « restitution » de passe 6 avait recréé un doublon : l'item 12 (tmpfs/orphelines) recouvrait à 80 % l'item 6, qui avait FUSIONNÉ deux findings sources distincts (le volume dupliqué d'audit-P3, le tmpfs/orphelines du ground-truth-P3). 12 segments, 11 distincts. Le compte juste EST 12 : l'item 6 est **scindé** (il ne garde que volume + destruction déclarée), l'item 12 porte seul tmpfs/orphelines — les deux findings sources retrouvent chacun leur ligne. Tout le reste du mandat vérifié conforme : 4+12+6 aux « · », total 23, rectifications dérivées cohérentes (« 20 dont un doublon » arithmétiquement recoupé), les 4 LOW de passe 6 en place — y compris la précision que le 3ᵉ site `0/0/2/~5` était déjà correct avant la passe 6, son « aux deux sites » étant exact.

**Trend : P5 `0/0/2/1` → P6 `0/0/1/4` → P7 `0/0/1/0`. Passe 8 — LA DERNIÈRE du budget (rotation : Haiku, mandat : l'unique item scindé). Si un MED survit, l'arbitrage de sortie revient à Guy.**

**2026-08-15 — `bmad-create-story validate`, PASSE 8 (Haiku, lentille unique, mandat minimal). CONVERGENCE : 0 CRIT / 0 HIGH / 0 MED — au plafond exact du budget de 8 passes.**

Les 12 items distincts vérifiés un à un, la scission 6/12 en place, « tmpfs dimensionné » à une seule occurrence, l'entrée de passe 7 conforme à l'état du fichier.

**Bilan du cycle** : `4/4/8/5` → `0/0/2/~5` → `1/4/12/6` → `0/0/5/1` → `0/0/2/1` → `0/0/1/4` → `0/0/1/0` → `0/0/0/0`. Modèles : rédaction Fable → Sonnet ×3 → Haiku ×3 → **Opus ×3 (architecture — la passe qui a tout changé : réfutations sur pièces, révision D1 vers le répertoire de l'issue, dérogation de splitting arbitrée par Guy)** → Sonnet ×2 → Haiku ×2 → Opus ×1 → Sonnet ×1 → Haiku ×1. Le FOND est scellé depuis la passe 4 ; les passes 5-8 n'auront corrigé que les comptes rendus du cycle lui-même — la § *Recompter* prise en défaut CINQ fois sur ses propres artefacts, d'où la leçon désormais écrite au corps de la spec : la ventilation s'écrit EN MÊME TEMPS que le total. Statut → `ready-for-dev`.
