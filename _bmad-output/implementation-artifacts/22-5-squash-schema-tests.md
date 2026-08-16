# Story 22.5 : Le schéma de test se rejoue en un geste, plus en soixante et un

## Status

done

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
#   {"/var/lib/mysql":"size=4g"}
docker exec kesh-mariadb-dev sh -c 'grep " /var/lib/mysql " /proc/mounts'
#   tmpfs /var/lib/mysql tmpfs rw,nosuid,nodev,noexec,relatime,size=4194304k,mode=755,inode64 0 0
docker exec kesh-mariadb-dev stat -c '%a %U:%G %n' /var/lib/mysql
#   755 mysql:mysql /var/lib/mysql
```

⚠️ **Ce bloc a menti pendant une passe** : il montrait encore `size=4g,mode=1777` après que la passe 1 de revue eut RETIRÉ ce `mode` (trop permissif — l'entrypoint pose `755 mysql:mysql` tout seul). Le correctif avait été appliqué au compose sans que la preuve écrite ici soit rejouée — la § *Propagation post-patch* prise en défaut sur mon propre artefact, une fois de plus. Relevé en passe 3, indirectement : la lentille cherchait un `mode=1777` manquant, en ayant inversé le sens du correctif.

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

### Review Findings — `bmad-code-review` passe 1 (2026-08-16, Opus ×3 : Blind Hunter, Edge Case Hunter, Acceptance Auditor)

Diff ciblé sur le cœur logique (1487 lignes) — arbitrage de Guy : la bascule mécanique de 2244 lignes et le corps de l'artefact généré sont couverts par échantillonnage, leur complétude étant tenue par un test dont la mutation a été jouée.

**Décisions requises**

- [x] [Review][Decision] **Le tmpfs supprime le dernier détecteur LOCAL du garde-fou P8** — le `CLAUDE.md` (P8) pose que seul un démarrage réel contre une base **persistante** révèle une migration appliquée puis modifiée, « c'est-à-dire, en pratique, la suite E2E ou un `cargo run` de dev ». T6 rend `kesh` ET `kesh_e2e` éphémères : plus rien en local ne rencontre le checksum, et le défaut se déplace en aval, chez qui met à jour une installation réelle. Options : (a) l'assumer et amender P8 ; (b) redonner un volume persistant à la seule `kesh_e2e` ; (c) ancrer les checksums dans un test du dépôt. *(Blind Hunter — le diff consacre un paragraphe au raisonnement P8 quinze lignes plus haut sans voir qu'il se le retire.)*
- [x] [Review][Decision] **La flakiness KF-038 (#228) redevient atteignable à 6 threads** — mesuré pendant cette revue : `reconciliation_e2e::post_accept_skips_non_chf_transaction` et `post_reject_after_accept_returns_already_reconciled_failed` ont échoué à ~5,36 s puis passé au retry, dans 1 run tmpfs sur 3. `.config/nextest.toml` documente cette famille comme apparaissant **à 32 threads**. Le profil `ci` la masque (`retries = 1`), mais elle rougira un jour sans explication. Options : (a) commenter #228 avec les nouvelles conditions ; (b) baisser le plafond de threads ; (c) traiter la cause (attente de 5 s côté test).

**Correctifs**

- [x] [Review][Patch] L'invariant SOMMANT est auto-référentiel : `raw_mentions` est incrémenté DANS le filtre `starts_with` qui alimente déjà `attrs` — le message promet « toute mention de `#[sqlx::test` », le code ne tient que « toute ligne qui l'ouvre » **(HIGH)** [`crates/kesh-db/tests/test_schema_guard.rs:481`]
- [x] [Review][Patch] `parse_attribute` fait `rfind(")]")` sur la ligne entière : un commentaire de fin de ligne citant `"./test-schema"` fait passer un attribut sur le vrai migrator pour un attribut de squash — exemption muette [`test_schema_guard.rs:501`]
- [x] [Review][Patch] Un `/* … */` dont une ligne ouvre par `#[sqlx::test(` est compté comme attribut réel (le filtre ne saute que `//`) [`test_schema_guard.rs:473`]
- [x] [Review][Patch] `unwrap_or_default()` sur le relevé vues/triggers/routines — seul des six à ne pas `expect` : une erreur SQL y devient « zéro objet », dans le relevé dont le but déclaré est de voir le PREMIER de ces objets [`test_schema_guard.rs:259`]
- [x] [Review][Patch] Bases du garde-fou déterministes sans PID : deux gates concurrents se détruisent mutuellement (le `DROP … IF EXISTS` de l'un frappe la base de l'autre) — **T3 exigeait « préfixe réservé + pid »**, la dérogation n'est pas déclarée [`test_schema_guard.rs:68`]
- [x] [Review][Patch] `diff_report` compare des ensembles, pas des multi-ensembles : une facette présente deux fois d'un côté passe [`test_schema_guard.rs:268`]
- [x] [Review][Patch] Plancher global `> 500` facettes : une catégorie entière (les FK, par exemple) peut disparaître sans le franchir — planchers par facette [`test_schema_guard.rs:311`]
- [x] [Review][Patch] `swap_database` perd la query-string de `DATABASE_URL` (`?ssl-mode=…`, `pool_max_conns=…` documenté en CI) : le garde-fou se connecte autrement que les 1102 tests qu'il valide [`test_schema_guard.rs:106`]
- [x] [Review][Patch] `squash_database_tracks_exactly_one_migration` monte un `MIGRATOR` complet dans un `_pool` jamais lu — 61 migrations payées pour rien dans la story qui les supprime [`test_schema_guard.rs:356`]
- [x] [Review][Patch] Le balayage est cloué à `root.join("crates")` alors que le doc-comment et AC3 disent « tout le workspace » [`test_schema_guard.rs:459`]
- [x] [Review][Patch] `collect_rs` suit les liens symboliques sans ensemble de visités (récursion infinie possible) et avale les erreurs de lecture [`test_schema_guard.rs:519`]
- [x] [Review][Patch] Rien ne vérifie que les 8 chemins d'`ALLOWED_REAL_MIGRATOR_FILES` existent encore — une entrée morte ré-exempte tacitement un futur fichier homonyme [`test_schema_guard.rs:36`]
- [x] [Review][Patch] `information_schema.EVENTS` et les `SEQUENCE` MariaDB ne sont couverts ni par le détecteur du script ni par le relevé — `mariadb-dump` les omet faute de `--events`, divergence silencieuse [`test_schema_guard.rs:241`, `scripts/regen-test-schema.sh:119`]
- [x] [Review][Patch] `generated_column_is_excluded_from_backup` (ex-attribut nu) paiera les 61 migrations à perpétuité alors qu'il n'exerce pas ce chemin — l'exemption est au grain du FICHIER [`crates/kesh-db/tests/accounts_role_backfill.rs:229`]
- [x] [Review][Patch] Parsing `.env` du script : sous `pipefail`, un `.env` sans la clé tue le script AVANT son message d'erreur soigné ; ni guillemets, ni `\r`, ni commentaire de fin de ligne ne sont retirés ; `@` dans le mot de passe casse le découpage ; l'absence de `:` met le nom d'utilisateur dans `DB_PASS` ; pas de percent-décodage là où sqlx en fait [`scripts/regen-test-schema.sh:28-45`]
- [x] [Review][Patch] Le fichier d'options écrit `password=$DB_PASS` sans guillemets — un `#` tronque le mot de passe, un `\` le transforme, en silence [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] `SQUASH_DB` est un override documenté sans aucune garde : `SQUASH_DB=kesh` exécute `DROP DATABASE kesh` — le commentaire d'à côté jure « jamais la base dev » [`scripts/regen-test-schema.sh:55,99`]
- [x] [Review][Patch] `RAW`/`NORM` hors du `trap`, et `trap` posé sur `EXIT` seul (ni INT, ni TERM, ni HUP) — le `$CNF` laissé derrière contient le mot de passe [`scripts/regen-test-schema.sh:143,160`]
- [x] [Review][Patch] `{ … } > "$OUT"` tronque l'artefact versionné dès l'ouverture du bloc : un échec en cours laisse un squash tronqué là où le run précédent était bon — écrire dans un temporaire puis `mv` [`scripts/regen-test-schema.sh:185`]
- [x] [Review][Patch] Le script affiche `✓` sans jamais vérifier que le fichier produit s'APPLIQUE (`grep -c '^CREATE TABLE' … || true` rend « 0 tables » en succès) — un squash inchargeable casse 1102 tests au gate suivant [`scripts/regen-test-schema.sh:185-205`]
- [x] [Review][Patch] Le détecteur d'objets exotiques lit un résultat vide comme `0` (`${EXOTIC:-0}`), et `information_schema` filtre par privilèges [`scripts/regen-test-schema.sh:119`]
- [x] [Review][Patch] L'ordre d'application des migrations est celui du glob (dépendant de `LC_COLLATE`), pas le tri par version de sqlx ; et chaque fichier ouvre sa propre session [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] `REPO_ROOT` calculé puis jamais utilisé ; le commentaire « AUCUNE erreur n'a besoin d'être masquée » est contredit 14 lignes plus bas par `|| true` + double redirection dans `cleanup()` [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] Le commentaire du script justifiant le préfixe `_sqlx_test_` par des droits restreints est devenu FAUX : `01-dev-grants.sql`, livré par la même story, donne `ALL ON *.*` [`scripts/regen-test-schema.sh` vs `scripts/mariadb-init/01-dev-grants.sql`]
- [x] [Review][Patch] `GRANT ALL PRIVILEGES ON *.* … WITH GRANT OPTION` est plus large que le besoin (`_sqlx_test%` + les bases `kesh%`) ; `WITH GRANT OPTION` n'est requis par rien [`scripts/mariadb-init/01-dev-grants.sql`]
- [x] [Review][Patch] `seed-dev-db.sql` n'est pas idempotent et n'a pas de transaction : rejoué — geste ATTENDU après chaque restart — il crée une seconde société puis casse sur l'unicité de `users`, laissant une société orpheline que le `SELECT … LIMIT 1` des tests peut atteindre [`scripts/seed-dev-db.sql:26`]
- [x] [Review][Patch] Son commentaire annonce les empreintes de `admin/admin123` **et** `changeme/changeme` alors qu'un seul utilisateur est inséré [`scripts/seed-dev-db.sql`]
- [x] [Review][Patch] Le seed est un doublon textuel du heredoc de `ci.yml` sans rien qui rougisse si l'un dérive — faire consommer le fichier par la CI [`.github/workflows/ci.yml:155-178`]
- [x] [Review][Patch] L'en-tête de `ci.yml` annonce toujours « ~1900 tests », « ~54 min mesuré 2026-07-13 » et « le vrai levier … suivi dans l'issue #251 » — trois affirmations fausses depuis `76fb8e92`, dans le fichier que la story a édité 100 lignes plus bas. Invisible au grep d'AC5 : la valeur périmée s'y écrit `~1900` et `~54 min` [`.github/workflows/ci.yml:24-28`]
- [x] [Review][Patch] Le libellé « les tests `kesh-db::repositories::*` … se connectent directement à DATABASE_URL » recouvre deux familles : `repositories::bank_profiles::tests` utilise bel et bien `#[sqlx::test]` [`.github/workflows/ci.yml:124`]
- [x] [Review][Patch] Le commentaire du tmpfs affirme que 4 Go « couvrent largement les éphémères d'un run complet » : vrai d'un run vert (≤ 6 bases vivantes), FAUX d'un run rouge — sqlx ne détruit pas la base d'un test échoué, ~17 Mo pièce, ~190 bases tiennent dans les 3,3 Go libres, et un squash cassé fait échouer 1102 tests d'un coup [`docker-compose.dev.yml:75-79`]
- [x] [Review][Patch] `mode=1777` rend le datadir MariaDB accessible en écriture à tout utilisateur du conteneur, là où l'installation normale est en `0700`/`0750` [`docker-compose.dev.yml:79`]
- [x] [Review][Patch] Le budget du healthcheck (`start_period: 30s`, 3 × 10 s) n'a pas suivi le passage au démarrage à froid SYSTÉMATIQUE (`mysql_install_db` + scripts d'init à chaque start) [`docker-compose.dev.yml:92`]
- [x] [Review][Patch] `docs/testing.md` affirme que `docker-compose.prod.yml` « garde son volume persistant » — ce fichier ne déclare aucun service MariaDB [`docs/testing.md`]
- [x] [Review][Patch] Non documenté : le conteneur `kesh` survit à un redémarrage de MariaDB et pointe alors sur une base vide, sans se remigrer (il ne le fait qu'au boot) [`docs/testing.md`, `docker-compose.dev.yml:39`]
- [x] [Review][Patch] Le § Prérequis de `crates/kesh-db/README.md` dit toujours « exécuter une fois » le GRANT (désormais automatisé, et « une fois » est faux sur tmpfs) et « appliquer la migration initiale, une fois » [`crates/kesh-db/README.md:37-50`]
- [x] [Review][Patch] `.config/nextest.toml` et `CLAUDE.md` juxtaposent deux « avant » irréconciliés — 38 min pour 1802 tests / 51 migrations (2026-07-13) et 64 min pour 2208 tests / 61 migrations (2026-08-16) — sans dire que l'écart s'explique par +22,6 % de tests et +19,6 % de migrations par test. Le dénominateur du « 3,25× » mérite d'être réconcilié [`.config/nextest.toml:4-8`]
- [x] [Review][Patch] Les chiffres du régime tmpfs (90,2 s de moyenne) manquent aux deux sites, dont les « 19,8 min » sont déjà périmés pour le poste de dev — la dérive que cette story corrige, recréée à sa propre échelle [`.config/nextest.toml`, `CLAUDE.md`]
- [x] [Review][Patch] « 283/283 tests de la base partagée verts » nomme la mauvaise population : 283 est le TOTAL des tests lib de `kesh-db` ; ceux de la base partagée sont 154 — dans la seule phrase qui prouve AC6 [story file, Dev Agent Record]
- [x] [Review][Patch] La table des commits du Dev Agent Record s'arrête à « celui-ci | T7-T8 » et ne liste pas le commit T6 ; la § « Gates réellement exécutés » ne déclare aucun gate pour lui [story file]
- [x] [Review][Patch] AC4 exigeait « gate complet **et** `nextest` seul » : seul le second est publié, alors que la mesure du gate complet existe désormais [story file]

**Reportés**

- [x] [Review][Defer] Le grain FICHIER de la liste d'exclusions — tout test ajouté à l'un des 8 fichiers hérite de la dérogation sans signal. Le grain attribut demanderait une annotation par test ; reporté, l'instance connue étant corrigée. [`test_schema_guard.rs:569`]
- [x] [Review][Defer] Le nombre « 61 » est recopié dans cinq artefacts de prose qu'aucun test ne contrôle. La 62ᵉ migration fera rougir le garde-fou, ce qui déclenchera la relecture — mais les cinq sites resteront faux entre-temps. [`nextest.toml`, `CLAUDE.md`, deux README, `test_schema_guard.rs`]

**Écartés**

- `kesh_e2e` créée en `utf8mb4_general_ci` alors que 36 tables sur 38 sont en `utf8mb4_unicode_ci` — écarté : la base `kesh` et le défaut serveur sont eux aussi en `general_ci`, et chaque table porte sa propre collation. S'aligner sur les tables ferait diverger `kesh_e2e` de `kesh`. Un commentaire d'explication est ajouté avec le correctif du GRANT.

### Review Findings — `bmad-code-review` passe 2 (2026-08-16, Sonnet ×3, contextes frais)

Cible : le **diff de remédiation** de la passe 1 (1394 lignes, story file exclu) — c'est là que se logent les régressions d'une passe précédente. Mandats différenciés : régressions introduites (aveugle), mécanismes NEUFS (cas limites), et **vérification que chaque correctif coché est réellement appliqué** (acceptation).

**Le HIGH** — et il vient du mandat « cas limites » :

- [x] [Review][Patch] `seed-dev-db.sql` s'accrochait à **n'importe quelle** société préexistante : la garde testait « existe-t-il une société », pas « existe-t-il LA société de seed ». Sur une base `kesh` où un développeur a créé sa propre société, le seed lui greffait un exercice « CI 2020-2030 » et des comptes « Ventes CI »/« Charges CI » — les numéros 1000/1100/2000/3000/4000 étant ceux du plan comptable suisse, la collision n'est pas improbable. Sans erreur ni avertissement. Garde désormais nominative. **(HIGH)** [`scripts/seed-dev-db.sql:44`]

**Deux correctifs de la passe 1 qui rouvraient le défaut qu'ils fermaient** :

- [x] [Review][Patch] Le garde `//` de `parse_attribute` était **tautologique** : l'appelant garantissant déjà que la ligne ouvre par le jeton, la condition était toujours vraie et le découpage coupait au premier `//` venu, fût-il dans les arguments (`migrations = "./a//b"`). Remplacé par une recherche du `//` **hors chaîne**. [`test_schema_guard.rs`]
- [x] [Review][Patch] `percent_decode` réinterprétait les backslashes **déjà présents** dans le mot de passe (`printf '%b'` s'applique à toute la chaîne) — le défaut même que la fonction venait fermer, par un autre vecteur. Les backslashes sont désormais doublés avant substitution. [`scripts/regen-test-schema.sh`]

**Deux correctifs de la passe 1 appliqués à MOITIÉ**, cochés comme faits — relevés par l'audit d'acceptation, dont c'était le mandat premier :

- [x] [Review][Patch] `collect_rs` : le volet « suit les symlinks » était corrigé, le volet « avale les erreurs de lecture » non. Un sous-arbre illisible sous-comptait en silence. [`test_schema_guard.rs`]
- [x] [Review][Patch] Script de régénération : le volet « ordre du glob » était corrigé (`LC_ALL=C`), le volet « chaque fichier ouvre sa propre session » non. Les 61 migrations s'appliquent désormais en **une seule session**, comme le vrai `MIGRATOR` (`run_direct`). [`scripts/regen-test-schema.sh`]

**Le reste** :

- [x] [Review][Patch] Le GRANT `` `_sqlx_test%` `` est plus large que son commentaire ne l'affirme : `_` est un **joker** dans un motif de GRANT, et les backticks n'y changent rien. **Vérifié par une sonde** : un utilisateur ainsi doté voyait une base `Xsqlx_testZZZ`. Échappé en `` `\_sqlx\_test%` ``, re-vérifié. [`scripts/mariadb-init/01-dev-grants.sql`]
- [x] [Review][Patch] Les commentaires de bloc `/* … */` sont suivis par **profondeur** et non par booléen : Rust les autorise imbriqués, et un `*/` interne rouvrait le scan sur des lignes encore commentées — un attribut fantôme y aurait été compté par les DEUX compteurs, donc **sans déclencher l'invariant sommant**. [`test_schema_guard.rs`]
- [x] [Review][Patch] Un bloc ouvert **et** refermé sur la même ligne échappait au filtre. [`test_schema_guard.rs`]
- [x] [Review][Patch] Le registre de checksums : doublon de version détecté (une `BTreeMap` écrasait en silence — et si la seconde ligne portait le checksum du fichier MODIFIÉ, le garde-fou validait ce qu'il existe pour interdire), entrée orpheline détectée, comparaison rendue insensible à la casse. [`test_schema_guard.rs`]
- [x] [Review][Patch] Les planchers par facette de la passe 1 **remplaçaient** le plancher global au lieu de s'y ajouter : leur somme vaut 401 contre 500, ce qui abaissait la garde contre une érosion diffuse. Les deux coexistent. [`test_schema_guard.rs`]
- [x] [Review][Patch] `diff_report` compare désormais des **multi-ensembles** : le rattrapage par égalité des longueurs, posé en passe 1, ne couvrait pas les décalages compensés. Compter est plus simple que rattraper. [`test_schema_guard.rs`]
- [x] [Review][Patch] Un `%` isolé faisait échouer `printf` sans arrêter le script (une substitution de commande en position d'affectation échappe à `set -e`) : le mot de passe partait avec un `\x` littéral, pour un « Access denied » sans rapport apparent. [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] Un glob de migrations sans correspondance n'était rattrapé que bien plus loin, par le plancher de tables. Contrôle direct ajouté. [`scripts/regen-test-schema.sh`]

**Traité par la documentation, et non par le code — décision assumée**

- [x] [Review][Decision] Une chaîne littérale s'étendant sur plusieurs lignes **sans** continuation `\` n'est pas suivie par l'analyse, qui est par ligne. Porter la parité des guillemets d'une ligne à l'autre fermerait ce cas, mais **désynchroniserait sur les 25 chaînes brutes `r#"…"#` et les 14 littéraux `'"'` de `crates/`** — un angle mort théorique échangé contre de faux rouges bien réels. Le compromis est écrit dans le doc-comment, avec son coût borné : un jeton logé dans une telle chaîne serait compté par les deux compteurs, donc au pire signalé comme contrevenant, jamais tu sur un vrai test. *(L'Edge Case Hunter avait raison sur un point : cette limite n'était pas reconnue, alors que ses deux voisines l'étaient.)*

**Écarté (1)**

- Le « 42,8× » serait un arrondi faux, 42,9× étant attendu. **Réfuté par recalcul depuis les secondes brutes** : 3863,919 / 90,243 = **42,82**. Le 42,9 du finding vient de la multiplication de deux arrondis intermédiaires (3,25 × 13,2) — précisément ce que la § *Recompter ses propres comptes rendus* proscrit.

### Review Findings — `bmad-code-review` passe 3 (2026-08-16, Haiku ×3, diff APLATI)

**Convergence : 0 CRITICAL / 0 HIGH / 0 MEDIUM, 4 LOW — le critère d'arrêt de la § *Review Iteration Rule* est atteint.** 12 findings bruts, 4 retenus après ground-truth, 6 réfutés, 2 hors sujet.

- [x] [Review][Patch] Port vide sur la forme `mysql://u:p@host:/base` — `${hostport#*:}` rend une chaîne vide, différente de l'hôte, donc le repli ne se déclenchait pas [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] Les trois valeurs de `_kesh_version` sont réinjectées telles quelles dans un `INSERT` du fichier généré : leur forme est désormais contrôlée. Elles viennent de la base — un chemin que ce script ne maîtrise pas — et une apostrophe rendrait le squash inchargeable pour 1102 tests [`scripts/regen-test-schema.sh`]
- [x] [Review][Patch] `docs/testing.md` ne disait pas ce que le SQL du seed documente : il suppose posséder la base, cible sa propre société par son nom, et saute la création de l'`admin` si un utilisateur de ce nom existe ailleurs [`docs/testing.md`]
- [x] [Review][Patch] **Le bloc de preuve d'AC6 affichait encore `mode=1777`** alors que la passe 1 l'avait retiré du compose — la § *Propagation post-patch* prise en défaut sur mon propre artefact, une fois de plus [story file]

**Réfutés au ground-truth (6)** — la discipline du `grep -nF` avant traitement, et elle a servi :

| Affirmation | Réfutation |
|---|---|
| **HIGH** « `mode=1777` manque, correctif coché non appliqué » | **Sens INVERSÉ** : le finding de passe 1 dit que `mode=1777` est trop permissif, le correctif était de le RETIRER. Datadir vérifié `755 mysql:mysql` |
| « mot de passe non guillemeté dans le fichier d'options » | `grep -nF 'password="$DB_PASS"'` → ligne 149 |
| « `@company_id` peut être NULL » | La garde et le `SELECT` portent sur le même nom (lignes 52 et 54) |
| « débordement d'index sur l'échappement en fin de ligne » | Instruit et couvert dès la passe 2 : la condition de boucle est revérifiée |
| **MED** « écart de 2 attributs au recomptage » | 1144 est ce que le Dev Agent Record annonce APRÈS revue ; la lentille comparait au tableau de spec, antérieur à la bascule — sa propre note le reconnaît |
| plafond de threads / `start_period` | L'un est l'élément ouvert que le fichier documente ; l'autre propose de relever un nombre **sans mesure** |

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
| `a0d13e84` | T7-T8 — compteurs rafraîchis, mesures publiées, verdict de fermeture |
| `31aaa20a` | T6 — tmpfs de dev, script d'init des droits, seed versionné, doc |
| passe 1 de revue | 43 correctifs — cf. § *Review Findings* et l'entrée de Change Log |

**AC0 — le transfert de couverture, nommé.** Avant cette story, 1102 tests rejouaient la chaîne réelle des 61 migrations : un filet redondant, et c'était exactement son coût. Depuis la bascule, l'invariant « les 61 migrations s'appliquent proprement, dans l'ordre, sur une base vide » n'est plus tenu que par `migrations_fresh_install.rs`, `migrations_upgrade_path.rs`, les backfills à fenêtre de la liste D2, et le garde-fou `test_schema_guard.rs`, qui monte le vrai `MIGRATOR` à chaque gate. Arbitrage assumé, et dit ici comme l'AC l'exige.

**AC1 — aucun test perdu, l'écart ventilé nominativement.** 2208 → **2209** tests, **0 échec des deux côtés**, 4 skipped inchangés, aucun `SLOW`, aucun retry. La preuve n'est pas le total mais le **diff des listes de tests exécutés**, extraites des deux logs, triées et comparées : **zéro disparu**, **un seul apparu** — `kesh-db::test_schema_guard every_sqlx_test_attribute_is_accounted_for`, précisément le test que T5 ajoute. Les deux runs APRÈS rendent des listes identiques. Les 4 tests du garde-fou de T3 étaient déjà dans le compte AVANT, leur commit étant le parent de la mesure. *(Un +1 net peut masquer une suppression compensée — mode d'échec payé sur 16-1a et 16-2a ; le diff nominatif est ce qui l'exclut.)*

**AC2 — garde-fou vert, 4 assertions.** Structure (`information_schema` complet), ligne d'amorçage `_kesh_version`, suivi `_sqlx_migrations`, et la quatrième née d'une mutation restée verte (le `MIGRATOR` compilé peut ignorer une migration ajoutée — `sqlx::migrate!` est compile-time). 4 mutations jouées, 4 rougissements — déclaré au commit `d5f0cf73`. Les 3 assertions à base réelle ont tourné dans les deux runs de mesure.

**AC3 — la complétude est un test, et sa mutation a été jouée.** Recensement recompté depuis la source au commit de bascule, ventilation qui somme :

| Graphie | Au commit de bascule | Après la passe 1 de revue |
|---|---|---|
| `migrations = "../kesh-db/test-schema"` | 749 | 749 |
| `migrations = "./test-schema"` | 353 | **354** |
| `migrator = "kesh_db::MIGRATOR"` (liste D2) | 26 | **24** |
| `migrations = false` (liste D2) | 17 | 17 |
| **Total** | **1145** | **1144** |

Le total de 1145 était celui des 1142 du recensement de spec plus les 3 attributs du garde-fou livré en T3. Les deux mouvements de la revue : `squash_database_tracks_exactly_one_migration` cesse d'être un `#[sqlx::test]` (il montait un `MIGRATOR` complet pour un `_pool` jamais lu), et `generated_column_is_excluded_from_backup` bascule au squash (il n'exerce pas le chemin des migrations, mais logeait dans un fichier exempté). Les deux ventilations somment.

Mutation jouée le 2026-08-16 : un attribut de `crates/kesh-api/tests/accounts_e2e.rs` re-basculé à la main vers `migrator = "kesh_db::MIGRATOR"` → le test **rougit en nommant `crates/kesh-api/tests/accounts_e2e.rs:211`**, depuis un binaire de test de `kesh-db` : la frontière de crate est bien franchie. Restauration **par copie**, vérifiée deux fois (`diff -q` contre la référence, puis `git status` propre vis-à-vis de `HEAD`), test re-vert.

**AC4 — la mesure, et le verdict qu'elle dicte.** Protocole D5 : même machine, profil `ci`, compilation hors chronomètre, deux runs par état, `mem-guard` actif, station au repos. L'AVANT sur `d5f0cf73`, l'APRÈS sur son enfant `76fb8e92` — les deux commits ne diffèrent que par la bascule et son test.

| | run 1 | run 2 | moyenne | résultat |
|---|---|---|---|---|
| **AVANT** (`d5f0cf73`) | 3677 s | 4051 s | **64,4 min** | 2208 passed, 0 failed, 4 skipped |
| **APRÈS** (`76fb8e92`) | 1266 s | 1111 s | **19,8 min** | 2209 passed, 0 failed, 4 skipped |

**Gain : 3,25×** sur les moyennes — 3,31× meilleur contre meilleur, 3,20× pire contre pire, et **2,90× dans la lecture la plus défavorable constructible** (l'AVANT le plus rapide contre l'APRÈS le plus lent). Par test : 1,75 s → 0,54 s. Économie : **44,6 min par gate complet**.

**Le gate complet, chronométré lui aussi** (AC4 en demandait deux mesures, et la première rédaction n'en publiait qu'une) : `scripts/test-fast.sh --ci`, soit `fmt` + `clippy` + `nextest`, **91 s** au total sur workspace chaud dont 89,8 s de `nextest` — les deux gates ne se distinguent donc plus qu'à la marge une fois la compilation en cache.

**Et une troisième mesure, non prévue par AC4, que T6 a rendue nécessaire** : dans le régime tmpfs, deux runs au même protocole donnent **91,1 s et 89,4 s**, soit **1,5 min** de moyenne. Le tmpfs apporte **13,2×** par-dessus le squash — **42,8× depuis le point de départ**. Il pèse donc plus lourd que le squash lui-même : la création d'une base éphémère était dominée par les `fsync`, et le squash réduisait le nombre d'opérations DDL sans toucher à leur coût unitaire. ⚠️ Ce régime est **celui du poste de dev uniquement** ; la CI n'a pas de tmpfs. Sans cette ligne, les « 19,8 min » que la story vient d'écrire dans `nextest.toml` et le `CLAUDE.md` auraient été périmés pour le dev **le jour même** — la dérive que la story corrige, recréée à sa propre échelle.

⚠️ **Un flake est réapparu dans ce régime, et il se dit** : `reconciliation_e2e::post_accept_skips_non_chf_transaction` et `post_reject_after_accept_returns_already_reconciled_failed` ont échoué à ~5,36 s puis passé au retry, dans 1 run sur 3 — la famille **KF-038 (#228)**, que `.config/nextest.toml` documentait comme apparaissant *à 32 threads*. Le régime rapide a densifié la contention sur les verrous en supprimant l'attente disque. Le profil `ci` la masque (`retries = 1`), le profil par défaut non. Consigné dans `nextest.toml` et commenté sur l'issue #228, sur arbitrage de Guy.

**Verdict d'AC4 : le seuil de 2× est franchi même par la lecture la moins favorable → la PR ferme #251** (`closes`, porté sur le message de PR et non sur les commits intermédiaires, cf. § *Issue Tracking Rule*).

⚠️ **Le bruit de mesure est réel et se dit** : 10 % d'écart entre les deux runs AVANT, 14 % entre les deux APRÈS (et le run 1 y est plus lent que le run 2, l'inverse de l'AVANT). Il ne menace pas la conclusion — l'intervalle le plus défavorable reste à 2,90× — mais il interdit d'annoncer « 3,25× » comme une constante.

**Plafond de threads : intouché, et pourquoi.** Le `test-threads = 6` a été fixé par la contention des metadata-locks sur le rejeu des migrations — cause que cette story vient de supprimer. Il est donc vraisemblablement trop bas aujourd'hui. Il **reste à 6 faute de mesure, pas par conviction** : le relever exige de rejouer le tableau flakes compris (KF-038 #228), ce qui est un travail de mesure à part entière. L'avertissement est écrit dans `.config/nextest.toml` et dans le `CLAUDE.md`, aux deux endroits où quelqu'un ira lire le chiffre.

**AC5 — compteurs rafraîchis aux trois sites**, avec dates :

- `.config/nextest.toml` — le tableau de 2026-07-13 est **étiqueté historique** (« AVANT le squash ») au lieu d'être effacé, le paragraphe « ~894 tests / 51 migrations » est remplacé par l'état réel (1102 des 1145 attributs sur le squash, 43 en liste D2) et par les mesures datées du 2026-08-16 ; le commentaire du plafond renvoie à la re-mesure.
- `.github/workflows/ci.yml:124` et `:130` — « 84 tests » → **154**, recompté depuis la source et ventilé : 151 tests dont le corps appelle `test_pool()` (24 accounts + 3 audit_log + 33 contacts + 37 invoices + 34 journal_entries + 20 products) **+ 3** qui ouvrent leur propre connexion (2 dans `invoices.rs`, 1 dans `retry.rs`). La graphie de remédiation citée y est corrigée en `migrations = "./test-schema"`.
- `CLAUDE.md` § *Gate rapide* (le site que la spec désignait « § Plafonds mémoire, ligne 114 ») — le « 1,40× / ~894 / 51 migrations » devient l'histoire au passé, suivi du tableau de mesure et de l'avertissement sur le plafond ; la ligne `test-threads` du tableau des parallélismes porte le même renvoi.

*Preuve* : `grep -rnE '\b894\b|\b84 tests\b|51 migrations'` sur le dépôt ne rend plus que des sites **légitimes** — `notes-251-exploration.md`, la spec de cette story qui les nomme, et des stories historiques datées (6-4, 3-4, 7-2) dont les décomptes appartiennent à leur époque. Aucun résidu vivant.

**AC6 — satisfaite le 2026-08-16, après arbitrage de Guy.** `docker-compose.dev.yml` monte `/var/lib/mysql` en tmpfs de 4 Go (taille explicite : le défaut est la moitié de la RAM hôte, soit 15 Go sur cette station) et porte les trois flags de D4 ; `docs/testing.md` gagne une section § *Base de dev jetable*. Vérifié en jouant la procédure documentée **telle qu'écrite**, après un `restart` qui a bien remis la base à zéro (0 table dans `kesh`) : 61 migrations rejouées par `sqlx migrate run`, seed appliqué, **283/283 tests lib de `kesh-db` verts en 3,1 s** — dont les **154** qui travaillent sur la base partagée et qui sont la vraie preuve du re-seed ; les 129 autres sont des tests lib sans base. *(Le périmètre annoncé était faux : « 283 tests de la base partagée » nommait la mauvaise population, dans la seule phrase qui prouve AC6. Relevé en passe 1 de revue.)* 0 base éphémère orpheline. Les flags sont confirmés en base (`innodb_flush_log_at_trx_commit=0`, `sync_binlog=0`, `innodb_doublewrite=0`).

⚠️ **Trois choses que la spec n'avait pas vues, et qui ont dû être traitées** :

1. **La commande de preuve d'AC6 était fausse** — cf. l'AC corrigé plus haut. Un tmpfs de compose vit dans `.HostConfig.Tmpfs`, jamais dans `.Mounts`.
2. **`docker compose up --force-recreate` a conservé l'ancien volume nommé**, pourtant retiré du fichier : le conteneur recréé le portait encore dans `.Mounts`, le tmpfs monté par-dessus. Inerte sur le moment, mais c'est un piège différé — retirer un jour la ligne tmpfs aurait fait silencieusement réapparaître d'anciennes données. Corrigé par un `down` + `up` complet, puis suppression du volume ; `.Mounts` est désormais `[]`.
3. **Le tmpfs efface aussi les tables SYSTÈME**, donc les droits globaux de l'utilisateur `kesh` — ceux que le README de `kesh-db` documente comme étape d'installation manuelle. Sans eux, `#[sqlx::test]` ne peut plus créer ses bases éphémères et **toute** la suite d'intégration tombe, à chaque redémarrage. Traité par `scripts/mariadb-init/01-dev-grants.sql`, monté dans `/docker-entrypoint-initdb.d/` : l'entrypoint MariaDB le rejoue dès que le datadir est vide, ce qui ici arrive à tous les coups. Vérifié par un vrai redémarrage, pas par lecture. Le seed applicatif, lui, reste manuel (`scripts/seed-dev-db.sql`) — son oubli est bruyant par construction.

La base de dev de Guy et les bases de gate ont été détruites comme prévu ; `steadyinvest_test`, base d'un autre projet présente dans le même volume, a été signalée avant destruction et sacrifiée sur son arbitrage explicite.

**La CI, mesurée pour la première fois après la bascule** (PR #311, 2026-08-16) : le job « Backend (Rust) » complet — build, clippy et `cargo test --workspace -j1 --test-threads=1` en série — a pris **19 min 14 s**, là où il **dépassait 30 minutes et se faisait annuler à 30 min 17** avant le squash. Les deux autres jobs : Frontend 2 min 0 s, Docker build 7 min 37 s.

C'est la première mesure sur des runners à 4 cœurs, sans tmpfs — le gain y est donc celui du squash seul, et il porte. ⚠️ **La marge `timeout-minutes: 120` n'a PAS été baissée** : un timeout se dimensionne sur le pire cas et non sur le cas mesuré ; un seul run ne dit rien de la dispersion sur des runners partagés, et un timeout trop juste transforme un ralentissement passager en rouge qu'on apprend à ignorer. Le chiffre est consigné, la décision de le suivre appartient à une mesure ultérieure.

**Suite E2E — jouée le 2026-08-16, avant le push, selon la recette de `docs/testing.md`**

`181 passed, 38 failed, 19 skipped` en 11,9 min. Le backend a démarré sur `:3000` contre `kesh_e2e`, base **créée par le script d'init de T6** et migrée au boot — ce qui exerce en vrai tout le montage de la story : datadir en RAM, droits resserrés, base recréée à chaque démarrage.

Les 38 échecs se rangent dans les familles documentées, sans reliquat :

| Famille | Échecs | Rattachement |
|---|---|---|
| `bank-*` (import, CSV, confirms, crud, journal-link) | **13** | **KF-030**, ouverte, dont le titre annonce précisément « 13 tests fail » |
| `localStorage` / « JWT introuvable post-login » | 19 | famille documentée dans `docs/testing.md` § *Prérequis Playwright local* |
| `GET /_test/sent-emails → 400 — backend démarré sans SMTP factice` | 5 | le message se déclare lui-même ; le backend de ce run n'avait pas de SMTP |
| cascades (timeouts, éléments absents) | le solde | conséquences des trois familles ci-dessus |

⚠️ **Le différentiel branche ↔ `main` n'a PAS été mesuré**, et c'est un choix, pas un oubli. Il est remplacé par une preuve plus forte et vérifiable : **la branche ne modifie aucune ligne de code applicatif.** Cinq fichiers de `crates/*/src/` sont touchés, et chaque ligne changée y est soit un attribut `#[sqlx::test]`, soit un commentaire :

```sh
for f in $(git diff main...HEAD --name-only -- crates | grep '/src/'); do
  git diff main...HEAD -- "$f" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---)' \
    | grep -vE '^[+-]\s*#\[(sqlx::test|tokio::test|test)' \
    | grep -vE '^[+-]\s*(//|///|//!|/\*|\*)' | grep -vE '^[+-]\s*$'
done
#   (aucune sortie)
```

Une régression E2E par le comportement de l'application est donc **structurellement impossible** ici ; ce que la suite avait à prouver, c'est que le **harnais est vivant** et que le montage de T6 tient — les 181 tests verts le montrent. *(Attention à la relecture de cette commande : le chemin `crates/*/src` ne correspond à rien pour git et rend un résultat VIDE, qu'on lirait volontiers comme « rien n'a changé ». Piège rencontré, puis corrigé.)*

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

**2026-08-16 — `bmad-code-review`, PASSE 1 (Opus ×3, contextes frais : Blind Hunter à l'aveugle, Edge Case Hunter avec accès dépôt, Acceptance Auditor avec la spec).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter (diff seul) | 0 | 1 | 13 | 18 |
| Edge Case Hunter (diff + dépôt) | 0 | 0 | 6 | 12 |
| Acceptance Auditor (diff + spec) | 0 | 0 | 4 | 6 |
| **après fusion et triage** | **0** | **1** | **—** | **—** → 2 décisions, 41 correctifs, 2 reportés, 1 écarté |

Diff **ciblé sur le cœur logique** (1487 lignes) sur arbitrage de Guy : les 2244 lignes de bascule mécanique et le corps de l'artefact généré sont couverts par échantillonnage, leur complétude étant tenue par un test dont la mutation a été jouée. Le découpage suit le risque, pas le volume.

**Le HIGH, et ce qu'il enseigne** : dans `scan_attributes`, le compteur `raw_mentions` était incrémenté **à l'intérieur** du filtre `starts_with` qui alimente déjà `attrs`. L'invariant sommant ne pouvait donc pas voir ce que le filtre écartait, tout en promettant dans son message de couvrir « toute mention de `#[sqlx::test` ». Un attribut n'ouvrant pas sa ligne (`#[ignore] #[sqlx::test(…)]`) échappait **simultanément** au contrôle de complétude et au garde-fou censé signaler l'angle mort. Corrigé par un compte des mentions hors commentaires et hors littéraux de chaîne ; **mutation jouée** : le test rougit désormais en nommant l'écart (1143 analysés contre 1144 mentions), là où il restait vert.

**Le fil rouge des trois lentilles** : le garde-fou avait plusieurs façons de se taire. Outre le HIGH — `unwrap_or_default()` sur le seul relevé dont la raison d'être est de voir apparaître un objet ; un commentaire de fin de ligne suffisant à faire passer un attribut du vrai migrator pour un attribut de squash ; un plancher global de 500 facettes tolérant la disparition d'une catégorie entière ; `EVENTS` et `SEQUENCE` couverts par aucun des deux garde-fous ; le script affichant `✓` sans jamais vérifier que le squash produit s'applique. Dans une story dont la thèse est « la complétude est un TEST, pas un grep », c'est le cœur qui était visé.

**Les deux arbitrages de Guy** :
- **Garde-fou P8** — T6 rendant `kesh` et `kesh_e2e` éphémères, plus aucun démarrage local ne rencontrait le checksum d'une migration modifiée : le défaut se serait déplacé chez qui met à jour une installation réelle. Réponse retenue : **ancrer les checksums dans le dépôt** (`crates/kesh-db/migrations.sha384` + `published_migrations_keep_their_checksums`). Plus fort que le détecteur perdu — il rougit à chaque gate, et non le jour où une base persistante croise par hasard la migration modifiée.
- **KF-038 (#228)** — la famille de flakes documentée « à 32 threads » redevient atteignable **à 6** sous tmpfs. Commentée sur l'issue avec ses conditions nouvelles, et consignée dans `.config/nextest.toml`.

**Deux affirmations de mes propres artefacts réfutées par les lentilles** : la commande de preuve d'AC6 ne rendait rien (un tmpfs de compose vit dans `.HostConfig.Tmpfs`, pas dans `.Mounts`), et le « 283/283 tests de la base partagée » nommait la mauvaise population — 283 est le total des tests lib de `kesh-db`, ceux de la base partagée sont 154. Les deux corrigées en place. La § *Recompter ses propres comptes rendus* aura donc repris cette story **sept fois** au total, cycle de spec compris.

**Reportés (2)** : le grain FICHIER de la liste d'exclusions (l'instance connue est corrigée, le grain attribut demanderait une annotation par test) ; le nombre « 61 » recopié dans cinq artefacts de prose qu'aucun test ne contrôle — le message du garde-fou les nomme désormais, ce qui déclenchera la relecture à la 62ᵉ migration.

**Écarté (1)** : la collation de `kesh_e2e`, alignée sur `kesh` et sur le défaut serveur, les tables portant chacune la leur.

**Gate après correctifs** : `scripts/test-fast.sh --ci` — voir l'entrée de commit pour le résultat déclaré.

**2026-08-16 — `bmad-code-review`, PASSE 2 (Sonnet ×3, contextes frais, sur le DIFF DE REMÉDIATION de la passe 1).**

| Lentille | mandat | retenu |
|---|---|---|
| Blind Hunter (diff seul) | les régressions que la remédiation a introduites | 4 MED, 2 LOW-MED, 3 LOW |
| Edge Case Hunter (diff + dépôt) | les mécanismes NEUFS et leurs bornes | **1 HIGH**, 5 MED, 5 LOW |
| Acceptance Auditor (diff + spec) | chaque correctif coché est-il RÉELLEMENT appliqué ? | 2 MED |
| **après fusion** | | **1 HIGH, 8 MED, 6 LOW** → 14 correctifs, 1 décision documentée, 1 écarté |

**Ce que cette passe a démontré, et qui justifie à elle seule la règle d'itération** : trois des quinze findings portent sur des correctifs de la passe 1 qui **rouvraient le défaut qu'ils fermaient** ou ne le fermaient qu'à moitié — un garde tautologique, un décodage qui réinterprétait les backslashes qu'il devait préserver, et deux findings traités sur un seul de leurs deux volets tout en étant cochés « fait ». Aucune relecture par l'auteur n'aurait vu cela : ce sont des défauts d'angle, pas d'attention.

**Le HIGH** : le seed s'accrochait à n'importe quelle société préexistante. Sur une base de dev où quelqu'un a saisi sa propre société, il lui greffait un exercice et des comptes de test — silencieusement, et avec des numéros de compte qui sont ceux du plan comptable suisse.

**L'audit d'acceptation a par ailleurs tout revérifié sur pièces** : 39 des 41 correctifs pleinement appliqués, les 61 checksums du registre **recalculés indépendamment** (0 écart), la ventilation 749 + 354 + 24 + 17 = 1144 recomptée, le commentaire GitHub sur #228 vérifié posté, le grep d'AC5 rejoué.

**Un finding réfuté** : le « 42,8× » serait faux. Recalcul depuis les secondes brutes — 3863,919 / 90,243 = 42,82. Le 42,9 attendu par la lentille venait du produit de deux arrondis.

**Un finding traité par la documentation plutôt que par le code**, et dit comme tel : suivre l'état « dans une chaîne » d'une ligne à l'autre fermerait un angle mort théorique mais désynchroniserait sur les 39 chaînes brutes et littéraux de caractère du workspace. Le compromis et son coût sont écrits dans le doc-comment.

**Trend : passe 1 `0/1/23/36` → passe 2 `0/1/8/6`.** Le HIGH de la passe 2 ne porte pas sur le code de la story mais sur un correctif de la passe 1 : la sévérité ne stagne pas, elle se déplace vers ce qui vient d'être écrit — comportement attendu d'une boucle qui travaille. **Une passe 3 s'impose** (§ Review Iteration Rule) — rotation : Haiku, avec le diff aplati que le `CLAUDE.md` recommande à partir de la passe 2.

**2026-08-16 — `bmad-code-review`, PASSE 3 (Haiku ×3, contextes frais, DIFF APLATI `HEAD` vs `main`).**

Diff aplati et non suite de commits, conformément au § *Haiku-specific guardrails* du `CLAUDE.md` : quatre commits se repassent sur les mêmes fonctions, ce qui est le cas pathologique connu de l'indexation Haiku. Les trois prompts portaient en outre l'interdiction d'affirmer une absence sans citation.

| Lentille | brut | retenu après ground-truth |
|---|---|---|
| Blind Hunter | 0 bug + 3 points mineurs | **1 LOW** (les 2 autres sont les éléments déjà reportés) |
| Edge Case Hunter | 7 | **2 LOW** (5 écartés) |
| Acceptance Auditor | 1 HIGH + 1 MED | **1 LOW** (les 2 écartés — mais le HIGH a fait trouver le LOW) |
| **total** | **12** | **0 CRIT / 0 HIGH / 0 MED / 4 LOW** |

**Le critère d'arrêt de la § Review Iteration Rule est atteint : plus rien au-dessus de LOW.**

**Six findings écartés au ground-truth, et la discipline a payé** :
- « le mot de passe n'est pas guillemeté dans le fichier d'options » → `grep -nF 'password="$DB_PASS"'` rend la ligne 149. Réfuté ;
- « `@company_id` peut être NULL » → la garde et le `SELECT` portent sur le même nom (lignes 52 et 54). Jamais NULL. Réfuté ;
- « l'échappement en fin de ligne peut faire déborder l'index » → instruit et trouvé couvert dès la passe 2 (la condition de boucle est revérifiée). Réfuté ;
- **« HIGH : le `mode=1777` manque au tmpfs, correctif coché non appliqué »** → **le sens du correctif est INVERSÉ**. Le finding coché (passe 1, ligne 186) dit que `mode=1777` est trop permissif : le correctif était de le RETIRER, et le datadir est bien en `755 mysql:mysql`. Réfuté ;
- « écart de 2 attributs dans le recomptage » → 1144 est exactement ce que le Dev Agent Record annonce après revue ; la lentille comparait au tableau de la spec, daté d'avant la bascule. Réfuté par sa propre note ;
- plafond de threads et `start_period` → l'un est l'élément ouvert que le fichier documente lui-même, l'autre une proposition de relever un nombre **sans mesure**, ce que ce même fichier proscrit.

**Quatre LOW retenus, tous corrigés** : port vide sur la forme `…@host:/base` · les trois valeurs de `_kesh_version` contrôlées de forme avant d'être réinjectées dans un `INSERT` généré (elles viennent de la base, une apostrophe rendrait le squash inchargeable pour 1102 tests) · `docs/testing.md` dit désormais ce que seul le SQL disait (le seed suppose posséder la base et saute l'`admin` s'il existe ailleurs) · **et le bloc de preuve d'AC6, qui affichait encore `mode=1777` après que la passe 1 l'eut retiré du compose.**

⚠️ **Ce dernier point mérite d'être retenu.** C'est la § *Propagation post-patch* prise en défaut sur mon propre artefact : j'avais corrigé le compose sans rejouer la preuve écrite dans le story file. Il a été trouvé **par accident**, par une lentille qui cherchait l'inverse — le genre de trouvaille qu'aucune méthode ne garantit, et qui rappelle pourquoi on fait tourner des lentilles orthogonales plutôt qu'une relecture de plus.

**⚠️ LE GATE DE CETTE PASSE EST D'ABORD TOMBÉ ROUGE — 34 échecs sur 2210 — et il faut le dire.**

Aucun des quatre correctifs de la passe 3 ne touche du code de test : le rouge venait d'ailleurs. Les 34 échecs sont tous dans `kesh-db repositories::journal_entries::tests::*`, tombent en **7 ms** chacun (donc au montage, pas sur leur assertion), et le panic est net : `delete_all_by_company` échoue sur `ForeignKeyViolation … fk_invoices_journal_entry`. La base partagée `kesh` portait **10 factures dont 2 liées à des écritures**, laissées par le gate précédent.

Le premier test en échec est `test_check_constraint_rejects_debit_and_credit_same_line` — **nommément celui que le `CLAUDE.md` cite** comme symptôme d'une base de gate piégée (précédent 16-3a). La règle a fait gagner le diagnostic.

**Mais la règle, telle qu'écrite, ne couvre pas ce cas.** Elle vise le gate « tué en vol » ; ici le gate précédent s'est terminé **normalement, en vert**, et a quand même laissé la base inutilisable pour le suivant. Les trois gates verts de ce cycle avaient tous suivi un redémarrage de conteneur, pour des raisons sans rapport (éprouver le `mode`, éprouver le GRANT) — celui-ci est le premier lancé sur une base déjà servie. La condition est **préexistante et latente**, pas introduite par cette story.

Après remise à zéro (restart du conteneur, migrations, seed — une quinzaine de secondes depuis le tmpfs) : **2210/2210, 0 flake, 91,3 s**. Le geste est désormais documenté dans `docs/testing.md`, avec l'argument que le tmpfs rend nouveau : la remise à zéro est assez bon marché pour être faite **avant chaque gate**, et non seulement après un incident.

**Tracé et codifié, sur arbitrage de Guy** : **KF-039 ([#310](https://github.com/guycorbaz/kesh/issues/310))** — avec les trois voies de correction durable, dont la seule qui ferme la classe entière (basculer les 154 tests de base partagée vers `#[sqlx::test]`, ce que le commentaire de `ci.yml` désigne depuis la Story 6-4 comme la remédiation pérenne) — et la règle du `CLAUDE.md` **étendue au gate TERMINÉ** — au point que son titre a changé, de « Un gate interrompu laisse la base piégée » à « **Un gate laisse la base piégée — et pas seulement quand il est interrompu** ». Sa question de déclenchement (« le run précédent a-t-il été interrompu ? ») était précisément le mauvais filtre : la remise à zéro y devient inconditionnelle.

**Bilan du cycle de revue** : `0/1/23/36` → `0/1/8/6` → `0/0/0/4`. Modèles : Opus ×3 → Sonnet ×3 → Haiku ×3. Les trois passes n'ont trouvé **aucun défaut dans le comportement livré** — le squash, la bascule et les mesures n'ont pas bougé depuis le commit de dev. Tout ce qu'elles ont corrigé tient dans deux familles : **le garde-fou pouvait se taire** (passe 1), et **mes propres correctifs rouvraient leur défaut ou n'étaient appliqués qu'à moitié** (passe 2). La passe 3, elle, ne trouve plus que des résidus de comptes rendus — la signature d'une convergence.
