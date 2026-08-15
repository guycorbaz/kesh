# Story 22.5 : Le schéma de test se rejoue en un geste, plus en soixante et un

## Status

backlog

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
| `migrator = "kesh_db::MIGRATOR"` | **1080** | 3 crates | → `TEST_MIGRATOR`, sauf exclusions D2 |
| `migrations = "../kesh-db/migrations"` | **23** | `bank_profiles_e2e.rs` (14), `email_templates_e2e.rs` (9) | → `TEST_MIGRATOR` (même chemin réel par une autre graphie) |
| `migrations = false` | **17** | backfills à fenêtre, upgrade_path | inchangés (gèrent eux-mêmes leurs migrations) |
| `migrator = "crate::MIGRATOR"` | **12** | `kesh-db/src/{backup,test_fixtures}.rs` (tests internes au crate) | → `crate::TEST_MIGRATOR` (même bascule, graphie interne) |
| `migrations = "./migrations"` | **9** | `kesh-db/src/repositories/bank_profiles.rs` | → `TEST_MIGRATOR` |
| attribut **nu** (`InferredPath`) | **1** | `accounts_role_backfill.rs` | fichier d'exclusion D2 — à expliciter en `migrations = false` ou chemin réel, jamais laissé implicite |
| **Total** | **1142** | 718 kesh-api, 393 kesh-db, 31 kesh-report | la ventilation SOMME au total — c'est le contrôle |

*Méthode de recomptage (à rejouer, pas à relire)* : attributs réels seuls, doc-comments exclus —

```sh
python3 -c "
import re, pathlib
pat = re.compile(r'^\s*#\[sqlx::test(\(([^\]]*)\))?\]\s*\$')
print(sum(1 for p in pathlib.Path('crates').rglob('*.rs')
            for l in p.read_text(errors='replace').splitlines() if pat.match(l)))"
```

⚠️ **Cinq graphies distinctes atteignent le même chemin de 61 migrations.** La première rédaction n'en voyait qu'une (« 1092 attributs `migrator = "kesh_db::MIGRATOR"` ») — un nombre qui additionnait en réalité deux littéraux sans le dire, et laissait 33 tests hors de la bascule ET hors de l'audit. *(Relevé en passe 1 par deux lentilles ; recompté par l'orchestrateur, ventilation ci-dessus prouvée sommante.)* D'où **AC3 : la complétude est tenue par un TEST, pas par un grep.**

**Autres compteurs périmés que cette story rafraîchit** : `.config/nextest.toml` (« ~894 tests / 51 migrations », 2026-07-13), le commentaire CI « 84 tests » sur base partagée (réels : 154), **et la § *Plafonds mémoire* du `CLAUDE.md`** qui porte les mêmes « ~894 » et « 51 migrations » (ligne 114) — le site que la § *Propagation post-patch* aurait rendu au grep de `\b894\b`, nommé ici pour ne pas dépendre de la vertu de l'exécutant.

## Ce que l'exploration a fermé comme voies (contraintes dures, vérifiées dans les sources)

- **sqlx 0.8.6 n'offre aucun point d'accroche** pour un schéma pré-migré : `test_context()` fait `CREATE DATABASE` vide puis `migrator.run_direct` ; `TestSupport` est un impl global non surchargeable ; `snapshot()` est un `todo!()` pour MySQL ; `fixtures(...)` se rejoue *après* le migrator.
- **MariaDB ne clone pas une base** : ni `TEMPLATE` (Postgres), ni plugin `CLONE` (MySQL 8), ni `mariabackup` par schéma à chaud.
- **Le partage de base entre tests est ÉCARTÉ** (arbitrage du 2026-08-14) : modes d'échec « base piégée » payés deux fois sur l'Epic 16 ; les tests destructeurs ne peuvent rien partager. **L'isolation par test se conserve.**
- Les réglages de durabilité sont **serveur** (flags de démarrage), pas session : les pools de test sqlx n'ont pas d'`after_connect`.

**Et ce qu'elle a confirmé possible, sur pièces** : `Migration` a tous ses champs `pub` et un constructeur public (`sqlx-core-0.8.6/src/migrate/migration.rs:7-36`) — une migration **synthétique** au SQL arbitraire se construit hors de sqlx ; et `migrator = "<chemin>"` de l'attribut est un `syn::Path` quelconque (`test_attr.rs:278-295`) — `kesh_db::TEST_MIGRATOR` passe. *(Vérifié en passe 1 par la lentille ground-truth, dans le registry.)*

## Décisions

**D1 — Le template est un MIGRATOR DE SQUASH : une migration unique portant le dump DDL du schéma complet.**

`kesh-db` expose un `TEST_MIGRATOR` : un `Migrator` construit sur **une seule** migration synthétique dont le SQL est le schéma entièrement migré (dump DDL sans données), **plus la réinjection de la ligne d'amorçage `_kesh_version` (id=1)** — sans elle, `check_downgrade_protection` et le verrou d'installation (`FOR UPDATE` sur id=1) changent de comportement dans les tests. Les populations du recensement basculent selon la colonne « Sort » du tableau ; un test paie alors **un** batch DDL au lieu de 61 cycles.

⚠️ **Le schéma d'origine porte 7 INSERT répartis sur 3 migrations** (`20260428000001` ×4, `20260522000001` ×1, `20260614000001` ×2) — pas « un seul ». `mysqldump --no-data` les élimine tous ; seule la ligne `_kesh_version` doit être réinjectée à la main, les six autres étant des backfills conditionnés à `companies`, no-ops sur une base fraîche. *(La première rédaction disait « le seul INSERT » — faux, recompté en passe 1.)*

⚠️ **La base éphémère reste une base par test** — rien ne change à l'isolation, seul le chemin de construction change.

**D2 — Une liste d'EXCLUSIONS fermée ICI, au grain du fichier, et tenue par le test d'AC3.**

Restent sur le chemin réel des migrations — parce qu'ils testent **ce chemin lui-même** :

| Fichier | Motif |
|---|---|
| `kesh-db/tests/migrations_fresh_install.rs` | l'installation fraîche EST le sujet |
| `kesh-db/tests/migrations_upgrade_path.rs` | fenêtre d'upgrade partielle (17 `migrations = false` + sub-`Migrator`) |
| `kesh-db/tests/accounts_role_backfill.rs` | backfill à fenêtre (y compris l'attribut nu à expliciter) |
| `kesh-db/tests/invoice_lines_revenue_account_backfill.rs` | backfill à fenêtre |
| `kesh-db/tests/post_restore_class_a.rs`, `post_restore_transactionality.rs` | triage P7 |
| `kesh-db/tests/client_number_canonical_backfill.rs` | backfill D6 sur schéma réel |

Tout le reste bascule. Un fichier qui voudrait rejoindre cette liste le fera **en modifiant le test d'AC3**, qui la porte en dur — c'est le rappel automatique, pas la prose de cette spec.

**D3 — Le squash SE RÉGÉNÈRE, il ne s'édite JAMAIS — un garde-fou à DEUX jambes le tient, et ce garde-fou EST le mécanisme d'entretien.**

- `scripts/regen-test-schema.sh` produit `crates/kesh-db/test-schema/schema-squash.sql` : base jetable montée par le vrai `MIGRATOR`, dump **`--no-data` en excluant `_sqlx_migrations`** (sqlx crée la sienne avant d'appliquer le migrator — l'inclure ferait échouer la migration synthétique sur « table already exists » dès le premier test ; *relevé en passe 1, CRITICAL*), sortie normalisée (`AUTO_INCREMENT=` volatils et commentaires horodatés retirés), en-tête `FOREIGN_KEY_CHECKS=0` garanti par le script (l'ordre alphabétique des tables d'un dump ne respecte pas les FK). Le script détecte `mariadb-dump` **ou** `mysqldump` (les deux existent selon les machines ; le client MySQL 8.4 fonctionne contre MariaDB 10.11 — vérifié, colonnes `GENERATED`, `CHECK` nommés et collations explicites fidèlement rendus, seul un warning bénin de column-statistics).
- **Jambe structure** : un test monte les DEUX migrators sur deux bases éphémères et diffe `information_schema` — tables, colonnes (type, nullabilité, défaut, `EXTRA` généré), index, contraintes (CHECK et **actions FK `ON DELETE`/`ON UPDATE`** comprises), collations, **et vues/triggers/routines** (le schéma n'en porte aucun aujourd'hui — le diff les couvre pour que leur premier ajout ne passe pas sous le radar).
- **Jambe données d'amorçage** : le même test compare la **ligne** `_kesh_version` (id, `kesh_version_min_required`, `kesh_version_last_applied`) entre les deux bases — `information_schema` ne décrit que la structure, et c'est précisément la donnée que D1 déclare critique. *(Trou relevé en passe 1, CRITICAL : le garde-fou ne vérifiait pas ce que la spec désignait comme essentiel.)*
- Toute migration ajoutée sans régénération **rougit ce test en le disant** (« régénérez : scripts/regen-test-schema.sh »). **C'est LE mécanisme d'entretien pour les stories futures** — leur gate rougit, aucune checklist à tenir, aucune ligne d'audit supplémentaire. *(La première rédaction ajoutait un vœu « à inscrire dans la ligne d'audit P5 des stories à migration » — un vœu sans mécanisme, retiré : le garde-fou suffit et fait mieux.)*
- **Exception unique à « ne s'édite jamais »** : la mutation de preuve d'AC2 — jouée puis restaurée **par copie** avec `diff` de contrôle (jamais via git sur un arbre non commité — règle tirée deux fois dans ce cycle).

**D4 — Volet vitesse machine : MariaDB dev sur tmpfs, durabilité relâchée — la base JETABLE seulement.**

`docker-compose.dev.yml` gagne `tmpfs: /var/lib/mysql` et `command: --innodb_flush_log_at_trx_commit=0 --sync_binlog=0 --innodb-doublewrite=0`. **Conséquence assumée et documentée** : la base dev `kesh` perd sa persistance au restart du conteneur — le seed se rejoue (`docs/testing.md` le dira ; Guy a déjà pratiqué ce montage à la main pour la 14-3a, port 3307, jamais formalisé). **CI hors périmètre nominal** : `services:` GitHub Actions ne passe pas de `command:` mariadbd ; la piste `options: --tmpfs` (option de `docker create`) est un **spike optionnel non bloquant** — le gain CI vient de D1, qui s'applique partout.

**D5 — Les mesures se PUBLIENT, l'« avant » se REMESURE, et le plafond de threads ne bouge qu'APRÈS.**

Le « ~69 min » cité en ouverture est un **ordre de grandeur emprunté** au gate de convergence de la 22-1, mesuré sur SON commit — il ne vaut pas « avant » pour AC4. La référence AVANT se remesure sur le **commit de base de la bascule T4** (l'état juste avant T4, tout le reste de la story déjà posé), l'APRÈS au commit de T4 — deux runs qui ne diffèrent que par la bascule. Le plafond de 6 threads n'est **pas** touché dans cette story — s'il devient débloquable, c'est une re-mesure dédiée future, consignée.

## Acceptance Criteria

**AC1 — Aucun test perdu, aucun test dégradé.** Le compte de tests exécutés après la bascule = compte avant **+ les tests que cette story ajoute elle-même** (garde-fou D3, complétude AC3 — nommés au Dev Agent Record avec leur compte). Zéro échec.
*Preuve* : les deux comptes de runs, l'écart ventilé test par test. *(« Même compte » tout court était contradictoire avec T3, qui ajoute des tests — relevé en passe 1.)*

**AC2 — Le squash est indiscernable du vrai schéma, STRUCTURE ET AMORÇAGE.** Le garde-fou D3 (deux jambes) est vert.
*Preuve* : le test lui-même — **et ses mutations jouées, par copie** : une colonne retirée du squash → rouge nominatif ; une migration ajoutée sans régénération → rouge ; la ligne `_kesh_version` altérée dans le squash → rouge (la jambe données).

**AC3 — La complétude de la bascule est un TEST, pas un grep.** Un test de source balaie tous les attributs `#[sqlx::test]` du dépôt (méthode ancrée du recensement) et exige : chaque attribut est soit sur `TEST_MIGRATOR`/`crate::TEST_MIGRATOR`, soit `migrations = false`, soit dans un fichier de la liste D2 **portée en dur par ce test**. Une graphie nouvelle (`migrations = "…"`, attribut nu, alias) ou un fichier hors liste → rouge en nommant le site.
*Preuve* : le test, **et sa mutation** : un attribut re-basculé à la main vers `kesh_db::MIGRATOR` hors liste doit rougir. *(La première rédaction prouvait par un grep mono-littéral, aveugle à 4 graphies sur 5 et pollué par les doc-comments — relevé en passe 1 par deux lentilles, CRITICAL.)*

**AC4 — La mesure est publiée, et elle DÉCIDE.** Avant/après selon D5 (même machine, commits ne différant que par T4), gate complet et `nextest` seul.
*Preuve* : tableau au Dev Agent Record. **Règle de décision** : si le gain sur le `nextest` seul est **< 2×**, la story ne ferme PAS #251 (la PR passe en `refs`), et consigne l'analyse de l'écart + la suite proposée — un gain décevant se documente, il ne se déclare pas victoire.

**AC5 — Les compteurs périmés sont rafraîchis, aux TROIS sites nommés.** `.config/nextest.toml` (~894/51 → réels datés, et son commentaire de plafond renvoie à la re-mesure D5), commentaire CI « 84 tests » (→ 154), **`CLAUDE.md` § Plafonds mémoire** (~894/51 → réels).
*Preuve* : `grep -rnE '\b894\b|\b84 tests\b'` → zéro résidu hors historique daté.

**AC6 — Le volet tmpfs est actif en dev et DIT.** `docker-compose.dev.yml` porte tmpfs + flags ; `docs/testing.md` documente la non-persistance et le re-seed.
*Preuve* : `docker inspect kesh-mariadb-dev --format '{{json .Mounts}}' | grep -o '"Type":"tmpfs"'` rend une occurrence pour `/var/lib/mysql`, et la section de doc existe.

## Tasks / Subtasks

- [ ] **T1 — Script de régénération + squash initial** (D3, AC2). `scripts/regen-test-schema.sh` : détection `mariadb-dump`/`mysqldump`, `--no-data`, **exclusion `_sqlx_migrations`**, normalisation (`AUTO_INCREMENT=`, horodatages), en-tête `FOREIGN_KEY_CHECKS=0`, réinjection de la ligne `_kesh_version`. `schema-squash.sql` versionné.
- [ ] **T2 — `TEST_MIGRATOR` et `crate::TEST_MIGRATOR`** (D1). Migration synthétique unique (`Migration::new`, champs publics — vérifié dans le registry). ⚠️ P8 : la migration synthétique ne vit que dans les bases ÉPHÉMÈRES — l'écrire **dans le doc-comment de `TEST_MIGRATOR`**, avec la règle « se régénère, ne s'édite pas ».
- [ ] **T3 — Garde-fou anti-dérive à deux jambes** (D3, AC2). Diff `information_schema` complet (vues/triggers/routines/actions FK compris) + comparaison de la ligne `_kesh_version`. Messages actionnables. **Trois mutations jouées, par copie.**
- [ ] **T4 — Bascule des CINQ graphies** (D1, D2, AC1, AC3, AC4). Les populations du recensement, selon leur colonne « Sort » ; l'attribut nu d'`accounts_role_backfill.rs` explicité. **La mesure AVANT se prend au commit précédant immédiatement cette bascule, l'APRÈS au commit de la bascule** (D5).
- [ ] **T5 — Test de complétude d'AC3** (AC3). Balayage de source par attributs ancrés, liste D2 en dur, mutation jouée.
- [ ] **T6 — tmpfs + durabilité dev** (D4, AC6). Compose + `docs/testing.md`. ⚠️ À faire quand AUCUN gate ne tourne — le restart du conteneur tue tout run en vol.
- [ ] **T7 — Compteurs rafraîchis aux trois sites** (AC5). `nextest.toml`, commentaire CI, `CLAUDE.md` § Plafonds mémoire — avec dates.
- [ ] **T8 — Mesures publiées et règle de décision appliquée** (D5, AC4). Tableau avant/après ; verdict fermeture (#251 `closes` ou `refs`) motivé par le seuil d'AC4 ; plafond de threads intouché, renvoi explicite à une re-mesure future.

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

## Change Log

**2026-08-15 — `bmad-create-story validate`, PASSE 1 (Sonnet ×3, contextes frais : aveugle, ground-truth, audit).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Aveugle (spec seule) | 2 | 4 | 4 | 3 |
| Ground-truth (spec vs sources/registry/DB réelle) | 0 | 1 | 2 | 1 |
| Audit (checklist + CLAUDE.md + issue) | 2 | 0 | 2 | 3 |
| **dédupliqué** | **4** | **4** | **6** | **5** |

Déduplication notable : les graphies multiples d'attribut, vues par les TROIS lentilles sous trois angles (grep aveugle au grain, 44 tests invisibles, 12 `crate::MIGRATOR` dans le total même) → **un** CRITICAL, fermé par la refonte d'AC3 en test de complétude. Les deux lentilles se CONTREDISAIENT sur les décomptes (audit : « périmés, 1182 » ; ground-truth : « exacts une fois ancrés, 1142 ») — **tranché par recomptage de l'orchestrateur** : 1142 exact, ventilation à six populations qui somme juste ; le grep naïf comptait les doc-comments.

- **CRIT 1 — le dump embarquait `_sqlx_migrations`** : sqlx crée la sienne avant le migrator — collision « table already exists » sur la totalité des tests basculés. T1 exclut la table, piège nommé.
- **CRIT 2 — le garde-fou ne pouvait pas vérifier la ligne `_kesh_version`** : `information_schema` ne décrit que la structure, et c'est une DONNÉE que D1 déclare critique. Le garde-fou gagne une jambe données (+ mutation dédiée en AC2).
- **CRIT 3 — les décomptes étaient recopiés de l'exploration, pas recomptés** — sur une story dont le sujet EST un compteur de tests. Tableau refait au commit de la passe, méthode ancrée ÉCRITE dans la spec, ventilation sommante exigée.
- **CRIT 4 — cinq graphies atteignent les mêmes 61 migrations** (1080 + 23 + 12 + 9 + 1 nu), la substitution et sa preuve n'en voyaient qu'une. AC3 refondu : la complétude est un TEST fail-loud (liste D2 en dur, toute graphie nouvelle rougit) — la philosophie du dispositif de la 22-4a, appliquée au harnais de test.
- **4 HIGH** : AC1 « même compte » contradictoire avec T3 (reformulé : aucun test perdu, ajouts nommés) · grain fichier/attribut de D2 (résolu par le test d'AC3) · `migrations_upgrade_path` invisible au grep (idem) · tableau non sommant (recompté).
- **6 MED** : « le seul INSERT » réfuté (7 INSERT sur 3 migrations — seul `_kesh_version` à réinjecter, dit avec le vrai compte) · « 69 min » était la mesure d'une AUTRE story sur un AUTRE commit (rétrogradé en ordre de grandeur, l'« avant » se remesure — D5/T4) · exception de mutation à « ne s'édite jamais » écrite · périmètre du diff élargi (vues/triggers/routines/actions FK) · script robuste (mariadb-dump/mysqldump, FK_CHECKS=0) · seuil de décision d'AC4 (< 2× sur nextest seul → la PR ne ferme pas #251) · le « vœu P5 » remplacé par le vrai mécanisme (le garde-fou D3 EST l'entretien) · `CLAUDE.md` § Plafonds mémoire ajouté aux sites d'AC5.
- **5 LOW** : commande d'inspection d'AC6 explicitée · rationale du plafond de threads dans nextest.toml (renvoi D5) · emplacement de la note P8 (doc-comment de TEST_MIGRATOR) · fallback client de dump vérifié sur pièces · précédent `common/mod.rs` requalifié (sous-ensemble, pas synthétique).

**Vérifié conforme par le ground-truth, et c'est porteur** : `Migration` constructible hors crate (champs publics, `Migration::new`) et chemin d'attribut arbitraire accepté — **D1 est faisable sur pièces** ; le dump réel (client MySQL 8.4 contre MariaDB 10.11) rend fidèlement colonnes générées, CHECK nommés et collations explicites ; la liste D2 est complète (grep exhaustif des patrons de fenêtre) ; compose dev vierge de tout réglage, conforme aux notes.

**Patches appliqués (réécriture complète de la spec), prochaine passe : Haiku, contexte frais.**
