# Story 22.5 : Le schéma de test se rejoue en un geste, plus en soixante et un

## Status

backlog

## Story

**As a** développeur de Kesh (humain ou agent) qui doit passer le gate complet avant chaque push,
**I want** que la suite d'intégration cesse de payer soixante et une migrations par test,
**so that** un gate complet coûte un quart d'heure et non pas l'heure qui, à huit petites modifications par jour, mange la journée.

Ferme **#251**. Story de l'**Epic 22 « Technical Debt Closure »**. Engagement explicite de Guy (2026-08-14) : « crée la story #251 dès que la 22-1 est bouclée » — elle l'est (PR #309).

## Contexte — le goulot est mesuré, et il n'est pas le CPU

Chaque test `#[sqlx::test]` crée une base éphémère **vide** puis rejoue le `MIGRATOR` entier — un cycle `INSERT _sqlx_migrations` / DDL à commit implicite / `UPDATE success=TRUE` répété **61 fois par test**, sérialisé par les metadata-locks MariaDB. C'est ce coût qui a figé le plafond nextest à **6 threads** (au-delà : contention, flakes mesurés) et limité le gain de nextest à 1,40×. Chaque migration ajoutée ralentit *tous* les tests — la 22-1 vient d'en ajouter une 61ᵉ.

**Chiffres frais, recomptés le 2026-08-14** (cf. `notes-251-exploration.md`, qui cite chaque source à la ligne) :

| Population | Compte | Note |
|---|---|---|
| `#[sqlx::test]` (bases éphémères) | **1142** | 718 kesh-api, 393 kesh-db, 31 kesh-report |
| — dont `migrator = "kesh_db::MIGRATOR"` | **1092** | la cible de cette story |
| — dont `migrations = false` | **17** | contrôlent eux-mêmes leurs migrations (backfills à fenêtre) |
| `#[tokio::test]` sur base **partagée** `kesh` | **154** | sérialisés par `shared-db-serial` — hors périmètre |
| Gate complet local (profil `ci`) | **~69 min** mesurées le 2026-08-15 | 2204 tests |

⚠️ **Les compteurs écrits dans le dépôt sont périmés et cette story les rafraîchit** : `.config/nextest.toml` annonce « ~894 tests / 51 migrations » (2026-07-13), le commentaire CI « 84 tests » sur base partagée (réels : 154).

## Ce que l'exploration a fermé comme voies (contraintes dures, vérifiées dans les sources)

- **sqlx 0.8.6 n'offre aucun point d'accroche** pour un schéma pré-migré : `test_context()` fait `CREATE DATABASE` vide puis `migrator.run_direct` ; `TestSupport` est un impl global non surchargeable ; `snapshot()` est un `todo!()` pour MySQL ; `fixtures(...)` se rejoue *après* le migrator.
- **MariaDB ne clone pas une base** : ni `TEMPLATE` (Postgres), ni plugin `CLONE` (MySQL 8), ni `mariabackup` par schéma à chaud.
- **Le partage de base entre tests est ÉCARTÉ** (arbitrage en discussion du 2026-08-14) : les modes d'échec « base piégée » ont déjà été payés deux fois sur l'Epic 16, et les tests destructeurs (import complet, upgrade partiel) ne peuvent rien partager. **L'isolation par test se conserve.**
- Les réglages de durabilité sont **serveur** (flags de démarrage), pas session : les pools de test sqlx n'ont pas d'`after_connect`.

Le seul point d'extension réel est `migrator = "<chemin Rust>"` — et le dépôt exerce déjà le patron `Migrator { migrations: Cow<[Migration]>, … }` deux fois (`migrations_upgrade_path.rs`, `tests/common/mod.rs`).

## Décisions

**D1 — Le template est un MIGRATOR DE SQUASH : une migration unique portant le dump DDL du schéma complet.**

`kesh-db` expose un `TEST_MIGRATOR` : un `Migrator` construit sur **une seule** migration synthétique dont le SQL est le schéma entièrement migré (dump DDL sans données), plus la ligne d'amorçage de `_kesh_version` (le seul INSERT du schéma d'origine, `20260522000001`). Les **1092** attributs `migrator = "kesh_db::MIGRATOR"` basculent sur `TEST_MIGRATOR` — substitution mécanique. Un test paie alors **un** batch DDL au lieu de 61 cycles.

⚠️ **La base éphémère reste une base par test** — rien ne change à l'isolation, seul le chemin de construction change.

**D2 — Une liste d'EXCLUSIONS fermée : les tests qui testent LE CHEMIN DES MIGRATIONS restent sur le vrai `MIGRATOR`.**

Restent sur le chemin réel : `migrations_fresh_install.rs`, `migrations_upgrade_path.rs`, les tests de backfill à fenêtre (`accounts_role_backfill`, `invoice_lines_revenue_account_backfill`), le triage P7 (`post_restore`), et `client_number_canonical_backfill.rs` (qui teste un backfill D6 sur schéma réel). La liste exacte est **fermée à l'implémentation** et justifiée test par test dans les Dev Notes — un test qui vérifie *la migration* ne peut pas courir sur un schéma qui ne migre pas.

**D3 — Le squash SE RÉGÉNÈRE, il ne s'édite JAMAIS — et un garde-fou anti-dérive le tient.**

- Un script (`scripts/regen-test-schema.sh`) produit `crates/kesh-db/test-schema/schema-squash.sql` : base jetable montée par le vrai `MIGRATOR`, `mysqldump --no-data` (+ l'amorçage `_kesh_version`), sortie normalisée (pas d'`AUTO_INCREMENT=` volatile).
- **Le garde-fou est un test** : il monte les DEUX migrators sur deux bases éphémères et diffe `information_schema` — tables, colonnes (type, nullabilité, défaut, `EXTRA` généré), index, contraintes (CHECK comprises), **collations**. Toute migration ajoutée sans régénération du squash **rougit ce test en le disant** (« régénérez : scripts/regen-test-schema.sh »). C'est le pendant du P5 : un artefact dérivé sans rappel automatique dérive en silence.
- Le squash est versionné ; sa régénération est un geste de la definition-of-done de toute story à migration (à inscrire dans la ligne d'audit P5 de ces stories).

**D4 — Volet vitesse machine : MariaDB dev sur tmpfs, durabilité relâchée — la base JETABLE seulement.**

`docker-compose.dev.yml` gagne `tmpfs: /var/lib/mysql` et `command: --innodb_flush_log_at_trx_commit=0 --sync_binlog=0 --innodb-doublewrite=0`. **Conséquence assumée et documentée** : la base dev `kesh` perd sa persistance au restart du conteneur — le seed se rejoue (le README/testing.md le dira, et Guy a déjà pratiqué ce montage à la main pour la 14-3a, port 3307, jamais formalisé). **CI hors périmètre nominal** : `services:` GitHub Actions ne passe pas de `command:` mariadbd ; la piste `options: --tmpfs` (option de `docker create`) est notée comme **spike optionnel non bloquant** — le gain CI vient de D1, qui s'applique partout.

**D5 — Les mesures se PUBLIENT, et le plafond de threads ne bouge qu'APRÈS re-mesure.**

La story publie, sur la même machine : durée du gate complet avant/après (référence : **69 min** le 2026-08-15), et le comportement à 6 threads. Le plafond de 6 n'est **pas** touché dans cette story — s'il devient débloquable (la contention venait du DDL), c'est une re-mesure dédiée, consignée, avec les seuils de flake de la § Plafonds mémoire en tête.

## Acceptance Criteria

**AC1 — Le gate complet reste VERT et IDENTIQUE en périmètre.** Même compte de tests exécutés qu'avant la bascule (aucun test perdu par la substitution), 0 échec.
*Preuve* : gate complet profil `ci` avant ET après sur le même commit de base, comptes comparés.

**AC2 — Le squash est indiscernable du vrai schéma.** Le garde-fou D3 (diff `information_schema` complet, collations comprises) est vert.
*Preuve* : le test lui-même — **et sa mutation jouée** : une colonne retirée du squash à la main doit le faire rougir en nommant la divergence ; une migration ajoutée sans régénération aussi.

**AC3 — Les exclusions D2 sont fermées, listées et justifiées.** Chaque test resté sur `kesh_db::MIGRATOR` porte sa justification ; `grep -rn 'migrator = "kesh_db::MIGRATOR"' crates/` rend exactement la liste des Dev Notes.
*Preuve* : le grep, confronté à la liste.

**AC4 — La mesure est publiée avec son périmètre.** Durée avant/après du gate complet (même machine, même profil), et durée du seul `nextest` hors fmt/clippy.
*Preuve* : tableau dans le Dev Agent Record, chiffres issus des runs réels.

**AC5 — Les compteurs périmés du dépôt sont rafraîchis.** `.config/nextest.toml` (894/51 → réels du moment), commentaire CI « 84 tests » (→ 154), et le commentaire de plafond de threads pointe la re-mesure D5.
*Preuve* : grep des anciennes valeurs → zéro résidu hors historique daté.

**AC6 — Le volet tmpfs est actif en dev et DIT.** `docker-compose.dev.yml` porte tmpfs + flags ; `docs/testing.md` documente la non-persistance et le re-seed.
*Preuve* : `docker inspect` (type `tmpfs` sur `/var/lib/mysql`), et la section de doc.

## Tasks / Subtasks

- [ ] **T1 — Script de régénération + squash initial** (D3, AC2). `scripts/regen-test-schema.sh`, sortie normalisée, `schema-squash.sql` versionné.
- [ ] **T2 — `TEST_MIGRATOR`** (D1). Migration synthétique unique dans `kesh-db`, patron `Cow<[Migration]>` (précédent : `tests/common/mod.rs`). ⚠️ Checksum stable : la migration synthétique vit dans les bases ÉPHÉMÈRES uniquement — P8 ne s'applique pas, mais l'écrire.
- [ ] **T3 — Garde-fou anti-dérive** (D3, AC2). Le test de diff `information_schema`, message actionnable, **mutations jouées** (colonne retirée ; migration ajoutée sans régénération).
- [ ] **T4 — Bascule des 1092 attributs** (D1, AC1, AC3). Substitution mécanique + liste d'exclusions D2 justifiée test par test. Gate complet avant/après sur le même commit de base (AC1, AC4).
- [ ] **T5 — tmpfs + durabilité dev** (D4, AC6). Compose + `docs/testing.md` (non-persistance, re-seed). ⚠️ À faire quand aucun gate ne tourne — le restart du conteneur tue tout run en vol.
- [ ] **T6 — Compteurs rafraîchis** (AC5). `nextest.toml`, commentaire CI, avec dates.
- [ ] **T7 — Mesures publiées** (D5, AC4). Tableau avant/après ; le plafond de threads reste à 6, renvoi explicite vers une re-mesure future.

## Dev Notes

### Références (chaque affirmation soursée à la ligne)

- **`notes-251-exploration.md`** (commit `e0e91eac`) — le rapport d'exploration complet : mécanique sqlx 0.8.6 (fichiers:lignes du registry), contraintes dures 1-8, comptages, état compose/CI.
- Issue **#251** ; `.config/nextest.toml` (plafond 6, historique 32→cassé) ; § *Plafonds mémoire* et § *Test Locally First* du `CLAUDE.md`.
- Précédent sub-`Migrator` : `crates/kesh-db/tests/common/mod.rs:50-83`.
- Précédent tmpfs manuel de Guy (14-3a, port 3307) : `14-3a-socle-roles-comptes.md:266,485,526`.

### Les pièges nommés d'avance

- **Le dump doit être NORMALISÉ** : `mysqldump` émet des `AUTO_INCREMENT=n` volatils et des commentaires horodatés — les retirer, sinon chaque régénération diffe pour rien.
- **`_kesh_version` fait partie du schéma d'amorçage** : sans sa ligne `id=1`, `check_downgrade_protection` et le verrou d'installation (`FOR UPDATE` sur id=1) changent de comportement dans les tests.
- **La collation `utf8mb4_bin` de la 22-1** (et toute déclaration explicite) doit survivre au dump/replay — c'est précisément ce que le garde-fou D3 diffe.
- **Ne pas basculer les 17 `migrations = false`** ni les exclusions D2 — la substitution mécanique doit être un motif exact (`migrator = "kesh_db::MIGRATOR"`), pas un sed large.
- **AC1 se mesure sur le MÊME commit de base** : un « avant » mesuré sur un autre état ne compare rien (§ Recompter, appliquée au chronomètre).

## Change Log
