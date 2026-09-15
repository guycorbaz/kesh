# Story 25.1c-zero : Rattacher chaque entrée d'audit à sa société

Status: ready-for-dev

⚠️ **Story-zéro de la 25-1c**, née de l'arbitrage du Project Lead du **2026-09-11** : `audit_log`
est **globale** alors que Kesh est multi-société, et une route de consultation exposerait les traces
de toutes les sociétés à l'administrateur d'une seule. Des trois issues possibles — ajouter la
colonne, restreindre la route, documenter la limite — **la colonne** a été retenue. Et parce qu'une
migration avec backfill arme à elle seule les garde-fous **P2-bis, P3, P5, P6, P7 et P8**, elle
sort de la 25-1c en story propre (`epic-25-vague1-suite.md` § *Arbitrage du 2026-09-11*).

| | objet | état |
|---|---|---|
| 25-1a | la piste survit à l'import | done |
| 25-1b | les trous d'alimentation | done |
| **25-1c-zero** *(celle-ci)* | **la colonne `company_id`, son remplissage, et rien d'autre** | ready-for-dev |
| 25-1c | la route et l'écran de consultation ([#378]) | backlog |

⛔ **Cette story ne livre AUCUN comportement visible.** Elle pose une donnée que seule la 25-1c
consommera. C'est le reproche fait à #144 à l'Epic 16 — *un champ persisté sans consommateur* —, et
il est ici **assumé par arbitrage** : le coût des garde-fous de migration justifie de ne pas le
mêler au code d'écran. ⇒ **Ne rien ajouter qui anticipe la 25-1c** : ni méthode de lecture filtrée,
ni route, ni type exposé au frontend.

## Story

**En tant que** Project Lead qui s'apprête à rendre la piste de contrôle consultable,
**je veux** que chaque entrée d'`audit_log` porte la société de son acteur,
**afin que** la future consultation (25-1c) puisse montrer à chaque société **ses** traces, et à
personne celles des autres.

**Couvre** : prérequis de [#378]. Aucune issue n'est fermée par cette story — le mot-clé de
fermeture de #378 appartient à la PR de la 25-1c.

## Le mécanisme est ARRÊTÉ — ne pas le rediscuter

Arrêté le 2026-09-11 après lecture ciblée, consigné dans `epic-25-vague1-suite.md`. Le rappel sert à
empêcher qu'on le « simplifie » en cours de route :

- **Un SOUS-SELECT dans l'`INSERT`**, sur le patron exact d'`actor_label`
  (`repositories/audit_log.rs:79-83`). ⇒ **aucun des 89 sites de construction d'une entrée ne
  bouge**, et `NewAuditLogEntry` ne gagne aucun champ.
- **Colonne `BIGINT NULL`, sans clé étrangère.**

**Ce qui le rend exact, vérifié et non supposé** :

- `users.company_id` est **NOT NULL**, écrit à la création, jamais modifié ; aucune table de
  jonction — un utilisateur appartient à **exactement une** société.
- **Aucune route n'écrit un audit sur une entité d'une autre société que celle de l'acteur** :
  aucun handler ne reçoit de `company_id` depuis la requête, et tout écart devient 404.
- Une mutation par clé API écrit `user_id` = le **créateur** de la clé (`entities/audit_log.rs:102-105`)
  ⇒ même société.
- `admin.full_import` écrit son entrée **après** le remplacement de `users`, dans la même
  transaction (`routes/admin.rs:444-455`), avec pour acteur le plus petit administrateur du **jeu
  restauré** : le sous-SELECT rend donc la société **restaurée** — *plus juste* qu'un champ, qui
  propagerait l'identifiant d'une société que le restore vient de détruire.
- **Aucune route ne crée de seconde société** (`companies::create` n'est appelée par aucune route ;
  seuls le bootstrap et l'onboarding en insèrent une). Le multi-société est une propriété **du
  schéma**, pas encore de l'usage — c'est ce qui rend la colonne peu risquée aujourd'hui, et
  nécessaire avant qu'une route ne l'expose.

⛔ **Les trois contraintes, chacune adossée à un mode d'échec du dépôt** :

1. **`NULL`, jamais `NOT NULL` sans défaut.** `check_schema_compat`
   (`kesh-api/src/admin_backup/import.rs:195-236`) rejette en 400 tout backup dont la source ne porte
   pas une colonne destination `NOT NULL` sans défaut : *une colonne mal déclarée rendrait
   inimportables toutes les sauvegardes existantes.*
2. **Aucune clé étrangère vers `companies`.** `companies` **est** remplacée au restore, alors que
   les entrées d'audit locales sont **conservées** depuis la 25-1a (`backup.rs:444-447`). C'est le
   scénario exact qui a fait retirer `fk_audit_log_user`. `company_id` rejoint la famille des
   pointeurs logiques : `entity_id`, `actor_api_key_id`, `user_id`.
3. **Ni `COALESCE`, ni rejeu post-restore.** `NULL` est la bonne réponse quand l'acteur n'existe
   pas : c'est un **état légitime et permanent** (« société indéterminable »), non un trou à
   combler. La garde d'`actor_label` (`WHERE actor_label = ''`) reposait sur une sentinelle
   textuelle ; **`NULL` n'en est pas une**.

## Acceptance Criteria

### Volet A — le schéma

**1. Une migration `crates/kesh-db/migrations/20260915000001_audit_log_company_id.sql`** ajoute à
`audit_log` :

- `company_id BIGINT NULL`, avec un `COMMENT` qui dit *pointeur logique, sans FK, `NULL` =
  société indéterminable* ;
- l'index **`idx_audit_log_company_date (company_id, created_at)`**.

Les deux clauses portent `IF NOT EXISTS` (précédents : `20260418000001_country_code.sql` pour la colonne, `20260419000001_invoice_paid_at.sql`
pour l'index ; MariaDB **10.11** en dev, en production et en CI — les trois `image:` vérifiées), ce qui
rend le DDL ré-entrant et fonde le verdict d'idempotence de l'AC 10.

⛔ **Interdits, et un seul suffit à casser quelque chose** : `NOT NULL` (contrainte 1), `FOREIGN KEY`
(contrainte 2), `DEFAULT` autre que `NULL`, et **`UPDATE _kesh_version`** (AC 13).

⚠️ **Pourquoi l'index est ici et pas dans la 25-1c** : la 25-1c filtrera par société et par période
— c'est la demande de [#378]. Poser l'index plus tard, c'est écrire une **seconde** migration et
réarmer P5 à P8 pour une ligne. Le coût ici est nul, puisque ces garde-fous sont déjà armés.

**2. Le backfill vit dans le MÊME fichier, en dernière instruction** :

```sql
UPDATE audit_log a
  JOIN users u ON u.id = a.user_id
   SET a.company_id = u.company_id
 WHERE a.company_id IS NULL;
```

- **`JOIN` et non `LEFT JOIN`**, **sans `COALESCE`** : une entrée dont l'acteur n'existe plus garde
  `NULL` (contrainte 3). ⚠️ *Ce cas est atteignable* : depuis la 25-1a, `user_id` n'a plus de FK.
- **Gardé par `IS NULL`** : une ré-exécution manuelle ne touche rien de ce qui est déjà rempli.

**3. L'en-tête de la migration est écrit EN ENTIER AVANT sa première application**, sur le modèle
de `20260910000001_audit_log_actor_label.sql` : le défaut qu'elle prépare, les trois contraintes,
P1/P3 (non breaking), P7 (le triage de l'AC 9), l'idempotence.

⛔ **P8 — une migration appliquée ne se modifie plus, pas même un commentaire.** Dès qu'elle a
tourné sur une base **persistante** (`kesh`, `kesh_e2e`), le checksum est enregistré, et toute
retouche empêche le binaire de démarrer. Les `#[sqlx::test]` ne le verront pas (bases éphémères) ;
seul un démarrage réel le révèle. ⇒ Relire l'en-tête **avant** le premier `sqlx migrate run` ; s'il
faut le retoucher après, **reconstruire** les bases de dev concernées et le dire au Change Log.
*(Précédent : passe 2 de revue de la 16-3b.)*

### Volet B — l'alimentation

**4. `insert_in_tx` remplit `company_id` par un sous-SELECT** (`repositories/audit_log.rs:79`) :

```sql
..., (SELECT company_id FROM users WHERE id = ?), ...
```

`user_id` est lié **une troisième fois**, comme il l'est déjà deux fois pour `actor_label`
(`:82-83`). **Sans `COALESCE`** : un acteur inexistant donne `NULL` et l'`INSERT` **réussit** —
une piste doit écrire plutôt que refuser d'écrire.

⛔ **Aucun appelant ne change.** Critère vérifiable : `NewAuditLogEntry` (`entities/audit_log.rs`)
et ses trois constructeurs sont **identiques** à `main` — `git diff main -- crates/kesh-db/src/entities/audit_log.rs`
ne touche que `AuditLogEntry` (AC 5).

**5. `AuditLogEntry` gagne `pub company_id: Option<i64>`, ET `const COLUMNS` le liste.**

⚠️ **Les deux, dans le même patch** : `AuditLogEntry` est un `FromRow`, et un champ absent de
`COLUMNS` (`repositories/audit_log.rs:52`) fait échouer **à l'exécution** en `ColumnNotFound`,
pas à la compilation. *C'est exactement l'échec de gate rencontré par la 25-1a sur `actor_label`.*
Le doc-comment du champ dit : pointeur logique sans FK, `NULL` = société indéterminable, et
**pourquoi `NULL` est permanent**.

### Volet C — l'export et l'import

Aucune ligne de `backup.rs`, `export.rs` ni `import.rs` ne devrait changer : l'export lit les
colonnes par `information_schema` (`backup.rs:97`, `non_generated_columns`), la fusion insère les
colonnes **du manifeste** (`backup.rs:491-500`). Les trois AC suivants **prouvent** que c'est vrai
plutôt que de le supposer.

**6. Un backup produit après cette story porte `audit_log.company_id`** — assertion sur le
`columnNames` du manifeste, dans `crates/kesh-api/tests/admin_full_export_e2e.rs`, sur le patron de
l'assertion `reconciliation_rules.active_uniq` (`:304-305`).

**7. Un backup SANS la colonne reste importable** (`check_schema_compat` l'accepte : elle est
nullable), et après import :

- les entrées **de l'archive** portent `company_id = NULL` ;
- les entrées **locales conservées** gardent leur valeur, inchangée.

Montage : `strip_column(&mut manifest, "audit_log", "company_id")` (`admin_full_import_e2e.rs:870`).
⚠️ Le helper ne retire la colonne **que du manifeste**, pas des lignes NDJSON — c'est suffisant,
les lignes étant lues **par nom de colonne** (`parse_ndjson_rows`), et c'est le montage qui a déjà
fait ses preuves sur `actor_label`. Précédent
`full_import_replays_actor_label_when_column_is_absent` (`admin_full_import_e2e.rs:1847`).

⚠️ **Ce `NULL` est le coût assumé de l'AC 9**, et il ne concerne aucune installation réelle — cf.
le fondement de l'exemption. Il doit être **écrit dans le test** comme tel, non découvert plus tard.

**8. Un backup AVEC la colonne — test de CARACTÉRISATION, nommé comme tel.**

- les entrées de l'archive gardent **verbatim** le `company_id` exporté ;
- les entrées locales conservées gardent le leur ;
- l'entrée `admin.full_import` porte le `company_id` de la société **restaurée**.

⛔ **Ce test ne valide pas un comportement souhaitable, il MESURE l'état que la 25-1c devra
traiter** — d'où « caractérisation » dans son nom et dans son doc-comment. Après un restore, les
entrées locales portent le `company_id` d'une société que le restore **a remplacée**. Deux
sous-cas, et ils ne se valent pas :

| identifiants | ce qu'une consultation scopée en ferait |
|---|---|
| **différents** | elle ne les montrerait **à personne** — le point ouvert de l'epic |
| **identiques** (le cas probable : une seule société, `id = 1` des deux côtés) | elle les montrerait à la société **restaurée** — traces d'une autre instance, présentées comme les siennes |

⚠️ **Le second sous-cas n'est écrit nulle part** : l'epic ne nomme que le premier. Le test doit
**couvrir les deux**, et le doc-comment renvoyer à la 25-1c. **Ne pas le résoudre ici.**

### Volet D — les garde-fous de migration

**9. P7 — triage : EXEMPTION PÉRISSABLE, et non registre de rejeu.**

La migration écrit des données ⇒ `every_data_backfill_migration_is_triaged` rougira tant qu'elle
n'est pas triée. Elle va dans `EXEMPT_MIGRATIONS` (`crates/kesh-db/src/post_restore.rs:364`) avec
**`ExemptionBasis::PerishableSince(20260827000001)`**, sur le précédent exact de `20260909000001`
(Story 24-5).

**Le fondement, à écrire dans la justification** :

- ⛔ **Elle N'est PAS « hors fenêtre », et la justification ne doit PAS commencer par `Hors
  fenêtre`.** Cette migration ne crée aucune table : un backup antérieur est **importable**. Le
  marqueur ferait échouer `exemptions_claiming_out_of_window_really_are_out_of_window`, et à
  raison.
- **L'argument porte sur le PARC.** Un backup est importable seulement s'il porte
  `invoice_settlements`, créée par `20260827000001`. Il manque la colonne seulement s'il précède
  `20260915000001`. L'intervalle `[20260827000001 .. 20260915000001)` **ne contient aucune version
  publiée** : la dernière est **v0.11.1, du 2026-08-24**, antérieure à la borne basse (vérifié :
  `git tag --sort=-creatordate`). L'instance en service exécute v0.11.1 (arbitrage du 2026-09-09).
- **Pourquoi pas le registre** : la classe B serait **techniquement disponible**, puisque le DDL et
  l'`UPDATE` sont dans le même fichier. Elle est écartée **par l'arbitrage du 2026-09-11**, « ni
  `COALESCE` ni rejeu post-restore ». Motif : un rejeu gardé `IS NULL` ne distingue pas une entrée
  d'archive d'une entrée **locale** restée `NULL`. Or après un restore, le `user_id` d'une entrée
  locale désigne le porteur de cet identifiant dans l'instance **source** : le rejeu l'attribuerait
  à la mauvaise société, le mensonge que la 25-1a a combattu.
- **Ce que cela coûte, dit en toutes lettres** : un backup pris dans l'intervalle, donc sur un build
  de développement non distribué, fusionne ses entrées avec `company_id = NULL` (AC 7).

**Trois nombres codés en dur changent, et c'est voulu** (`post_restore.rs`) :

| test | avant | après |
|---|---|---|
| taille d'`EXEMPT_MIGRATIONS` (`:1049`) | 11 | **12** |
| `the_release_script_sees_every_perishable_exemption` (`:1072`) | `vec![(20260909000001, 20260827000001)]` | `vec![(20260909000001, 20260827000001), (20260915000001, 20260827000001)]` — l'ordre suit celui du tableau : insérer l'entrée **juste après** `20260909000001` |
| « attendu 6 exemptions marquées Hors fenêtre » (`:971`) | 6 | **6, inchangé** — s'il bouge, la justification est fautive |

⛔ **Conséquence de release, à écrire dans la justification et au Change Log** : ce fondement **se
périme** si une version est taguée depuis `main` **avant** le merge de cette story (son arbre
porterait la borne sans la migration). `scripts/prepare-release.sh` le **contrôle** à la release
suivante et la refuse. ⚠️ **Ne PAS lancer ce script pendant la story** : il bumpe les versions
Cargo — cf. § *Une passe qui ne déclare pas ses axes*.

**10. P5 — `docs/migrations-idempotence-audit.md`.**

- Une ligne **à l'intérieur du tableau**, à sa place chronologique, **après** `20260910000001`.
- Verdict **`yes`** : DDL en `IF NOT EXISTS` et `UPDATE` gardé `IS NULL`. La justification précise
  que la migration **écrit des données** et renvoie à l'exemption de l'AC 9.
- **Recompter depuis la source**, jamais incrémenter :
  - `ls crates/kesh-db/migrations/*.sql | wc -l` doit rendre **68** ;
  - `grep -c '^| \`20' docs/migrations-idempotence-audit.md` doit rendre **68** ;
  - le total figure aux **deux** sites : `## Table d'audit (68 migrations)` et la ligne `Total`,
    qui gagne `+ 1 Story 25-1c-zero` ;
  - partition attendue : `yes` **8** (ajouter `audit_log_company_id` à l'énumération) +
    `tracked-by-sqlx` **60** + `no` **0** = 68.

**11. P6 — `crates/kesh-db/tests/migrations_upgrade_path.rs`** : `assert_eq!(total, 68)`,
`total - 34`, **frontière inchangée à 34**. Mettre à jour, dans le **même** patch :

- la généalogie (`// + audit_log_company_id (Story 25-1c-zero) = 68.`) ;
- la ligne `Story 25-1c-zero : 33 → 34, frontière inchangée (68 - 34 = 34).` ;
- le message d'assertion et le `.expect("apply_migrations_up_to(total - 34) failed")`.

⚠️ **Greper la VALEUR sur tout le fichier**, pas la formulation — `grep -nE "\b(33|67)\b" crates/kesh-db/tests/migrations_upgrade_path.rs`
— et trier à la main les occurrences historiques légitimes. *(Précédent : trois résidus manqués en
16-3b, parce que le grep cherchait la phrase.)* Le `grep -rn "migrations.len()\|apply_migrations_up_to" crates/`
ne rend aucun autre site positionnel : les trois autres fichiers résolvent **par version**
(`migrations_before`).

**12. P8 et le squash de test.**

- `crates/kesh-db/migrations.sha384` gagne la ligne de la migration — le test
  `published_migrations_keep_their_checksums` (`test_schema_guard.rs:556`) **imprime la ligne
  attendue** quand elle manque.
- `crates/kesh-db/test-schema/0001_schema_squash.sql` est **régénéré par `scripts/regen-test-schema.sh`**,
  jamais édité. Sans cela, 1102 tests échouent en `Unknown column 'company_id'`
  (`squash_matches_real_schema_structure` nomme la divergence).

**13. P1/P2/P2-bis/P3 — non breaking, donc aucun bump.** `ADD COLUMN` nullable et `ADD INDEX` : un
binaire antérieur ignore la colonne et continue d'insérer. ⇒ **ni** `kesh_version_min_required`,
**ni** version Cargo. À écrire dans l'en-tête de la migration et dans la ligne P5.

### Volet E — les tests, et leur épreuve

**14. Le backfill de migration** — nouveau fichier `crates/kesh-db/tests/audit_log_company_id_backfill.rs`,
montage de `closing_accounts_backfill.rs` : `#[sqlx::test(migrations = false)]`,
`apply_migrations_up_to(&pool, migrations_before(20260915000001, "audit_log_company_id"))`, données
en SQL brut, puis `kesh_db::MIGRATOR.run()`.

⛔ **Le fichier DOIT être inscrit à `ALLOWED_REAL_MIGRATOR_FILES`**
(`crates/kesh-db/tests/test_schema_guard.rs:38`), avec son motif en commentaire, sur le modèle de
l'entrée `closing_accounts_backfill.rs` (Story 24-5). Sans cela, `every_sqlx_test_attribute_is_accounted_for`
rougit sur `#[sqlx::test(migrations = false)]` : un test qui ne monte pas le squash doit être
**déclaré**, pas découvert. Le garde-fou n'impose qu'un **plancher** (`> 1100` attributs), aucun
total exact : rien d'autre à y bumper.

- (a) **deux sociétés**, un utilisateur chacune, des entrées de chacun ⇒ chaque entrée prend **la
  société de son acteur**, et **aucune** celle de l'autre ;
- (b) une entrée dont le `user_id` **n'existe pas** ⇒ `company_id` **reste `NULL`**, la migration
  ne s'arrête pas. *Il tue la mutation `JOIN` → `LEFT JOIN … COALESCE`* ;
- (c) **idempotence** : rejouer l'`UPDATE` de la migration est un no-op (`rows_affected == 0`) ;
- (d) ⛔ **assertion de montage** : avant `MIGRATOR.run()`, la colonne `company_id` **n'existe
  pas** dans `information_schema`. Sans elle, un test qui tournerait après la migration **passerait
  à vide** — le mode d'échec de `backfill_skips_archived_accounts` (16-1a).

**15. L'alimentation** — dans `repositories/audit_log.rs` (`mod tests`), en
**`#[sqlx::test(migrations = "./test-schema")]`** (graphie exigée par `test_schema_guard.rs:58` ;
dans `kesh-api`, c'est `"../kesh-db/test-schema"`) :

- (a) acteur utilisateur ⇒ `company_id` = celui de l'utilisateur ;
- (b) constructeur `api_key` ⇒ la société du **créateur** de la clé ;
- (c) `user_id` inexistant ⇒ l'`INSERT` **réussit**, `company_id = None`, `actor_label = "(inconnu)"`.

⚠️ Les trois tests existants du module utilisent `#[tokio::test]` avec une base **partagée**
(`test_pool()`) et l'administrateur du seed : ne pas les imiter pour les nouveaux cas. Une base
éphémère évite de laisser des résidus qui feraient rougir le gate suivant (KF-039, #310).

**16. Épreuve par mutation, résultats consignés au Change Log** — chaque mutation appliquée, le
test visé exécuté et **vu rouge**, puis la mutation retirée et `git diff` vérifié vide :

| mutation | test qui doit rougir |
|---|---|
| sous-SELECT de l'AC 4 remplacé par `NULL` | 15 (a) et (b) |
| `JOIN` → `LEFT JOIN` + `COALESCE(u.company_id, 0)` | 14 (b) |
| `WHERE a.company_id IS NULL` retiré, puis une valeur pré-posée à la main | 14 (c) |
| entrée retirée d'`EXEMPT_MIGRATIONS` | `every_data_backfill_migration_is_triaged` |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait** (acquis de la 25-1b). Chaque mutation
doit produire un **échec d'assertion**, pas une erreur de compilation — sinon elle ne prouve rien.

### Volet F — ce qui devient faux ailleurs

**17. Propagation — les affirmations que la story rend fausses**, à corriger dans le **même**
patch :

| site | ce qu'il affirme |
|---|---|
| `crates/kesh-api/src/routes/exports.rs:137-139` | « la table `audit_log` n'a pas de colonne `company_id`, donc les requêtes multi-tenant doivent passer par `details_json` ou par FK `users.company_id` » |
| `crates/kesh-api/tests/exports_global_e2e.rs:1144-1145` | « `audit_log` n'a PAS de colonne company_id ; on filtre par user_id » |
| `crates/kesh-db/src/repositories/audit_log.rs:70` | « toucher les quelque **trente** sites » — ils sont **89** ; la phrase est dans le commentaire même que l'AC 4 modifie |

⚠️ Le filtre par `user_id` du test d'export **reste correct** : il n'y a pas lieu de le réécrire,
seulement de corriger le commentaire qui le motive.

**Grep de contrôle, à rejouer après le patch** — il doit ne rendre **que** des occurrences
historiques datées :

```sh
grep -rnE "(n'a (PAS|pas) de|sans) (colonne )?\`?company_id" crates docs README.md
grep -rnE "\b(trente|30) sites\b" crates
```

⚠️ `routes/dunning_levels.rs:3`, `profile.rs:43`, `accounts.rs:246`, `reports.rs:288`,
`journal_entries.rs:701/1106` et `invoices.rs:2222` parlent **d'autres tables** : faux positifs à
laisser.

**18. Les manuels — AUCUN site à changer, et c'est une conclusion à VÉRIFIER, non à croire.**
`grep -nE "audit_log|company_id|journal d.audit" docs/manual/fr/*.tex` : aucune mention du schéma
de la table ni de sa portée par société. `user-manual.tex:498-502` (« ce qui manque encore, c'est
la consultation du journal d'audit ») **reste vrai** : c'est la 25-1c qui le rendra faux. ⇒ **Pas de
régénération de PDF.** Pas d'entrée CHANGELOG non plus : aucun effet visible. La 25-1c portera
l'une et l'autre.

## Tasks / Subtasks

- [ ] **T0 — Remise à zéro VÉRIFIÉE de la base de dev** avant tout (la machine a redémarré, le
      tmpfs est vide) : les trois étapes de `CLAUDE.md` § *Un gate laisse la base piégée*, **puis**
      contrôler qu'un `Admin` existe. *Compter les migrations ne prouve rien.*
- [ ] **T1 — La migration** (AC 1, 2, 3, 13). En-tête **relu avant** le premier `migrate run`.
- [ ] **T2 — Alimentation et entité** (AC 4, 5), commentaire `:70` corrigé au passage (AC 17).
- [ ] **T3 — Squash et checksum** (AC 12) : `scripts/regen-test-schema.sh`, ligne `migrations.sha384`.
- [ ] **T4 — P7** (AC 9) : entrée `PerishableSince`, justification sans le marqueur, trois nombres
      du tableau de l'AC 9.
- [ ] **T5 — P5 et P6** (AC 10, 11), **compteurs recomptés depuis la source**, valeurs grepées.
- [ ] **T6 — Tests** (AC 6, 7, 8, 14, 15). ⛔ *Une tâche qui décrit un test est une promesse ; la
      cocher sans l'avoir écrit la transforme en mensonge, et le gate reste vert* (acquis 25-1b).
- [ ] **T7 — Mutations** (AC 16), résultats **observés** consignés.
- [ ] **T8 — Propagation** (AC 17) et vérification des manuels (AC 18), les deux greps rejoués.
- [ ] **T9 — Gate complet**, base remise à zéro **et vérifiée**. ⛔ `kesh-db` touché : **ciblage
      interdit**, même en boucle de revue (exception `kesh-db` de `CLAUDE.md`). Backend :
      `scripts/test-fast.sh`. Frontend : non touché, gate non requis pour le commit — **mais** la
      suite E2E est un prérequis du push, et c'est **elle seule** qui démarre un binaire contre
      une base persistante (P8).

## Dev Notes

### Ce que la story touche, fichier par fichier

| fichier | nature |
|---|---|
| `crates/kesh-db/migrations/20260915000001_audit_log_company_id.sql` | **nouveau** |
| `crates/kesh-db/migrations.sha384` | +1 ligne |
| `crates/kesh-db/test-schema/0001_schema_squash.sql` | **régénéré, jamais édité** |
| `crates/kesh-db/src/repositories/audit_log.rs` | sous-SELECT, `COLUMNS`, commentaire `:70`, tests AC 15 |
| `crates/kesh-db/src/entities/audit_log.rs` | champ `company_id` sur `AuditLogEntry` **seulement** |
| `crates/kesh-db/src/post_restore.rs` | entrée d'exemption + 2 nombres codés en dur |
| `crates/kesh-db/tests/migrations_upgrade_path.rs` | 67 → 68, 33 → 34 |
| `crates/kesh-db/tests/audit_log_company_id_backfill.rs` | **nouveau** (AC 14) |
| `crates/kesh-db/tests/test_schema_guard.rs` | +1 entrée à `ALLOWED_REAL_MIGRATOR_FILES` (AC 14) |
| `crates/kesh-api/tests/admin_full_import_e2e.rs` | tests AC 7 et 8 |
| `crates/kesh-api/tests/admin_full_export_e2e.rs` *(ou `admin_backup_e2e.rs`)* | test AC 6 |
| `crates/kesh-api/src/routes/exports.rs`, `tests/exports_global_e2e.rs` | commentaires (AC 17) |
| `docs/migrations-idempotence-audit.md` | ligne + deux totaux + partition |

**Deux crates, aucun frontend, aucune clé i18n** — sous le seuil de la § *Règle de splitting
préventif*.

### Ce qui ne bouge PAS, et qu'il ne faut pas « améliorer »

- **Les 89 sites qui construisent une entrée d'audit**, et `NewAuditLogEntry`.
- **`backup.rs`, `export.rs`, `import.rs`** : les AC 6 à 8 prouvent qu'ils n'ont rien à changer. Si
  l'un d'eux doit changer pour que ces tests passent, **s'arrêter et le signaler** : l'analyse
  ci-dessus serait fausse.
- **`find_by_entity`** : ne pas lui ajouter de filtre par société. La méthode de lecture filtrée et
  paginée appartient à la 25-1c.
- **Aucun `COALESCE`, nulle part.**

### État actuel des fichiers modifiés

- **`repositories/audit_log.rs`** — `insert_in_tx` (`:61-121`) insère, puis relit la ligne par
  `SELECT {COLUMNS}` dans la même transaction. `actor_label` est déjà rempli par sous-SELECT avec
  `COALESCE(…, '(inconnu)')` : **ne pas copier ce `COALESCE` pour `company_id`**, les deux colonnes
  n'ont pas la même sémantique — un libellé doit toujours nommer quelque chose, une société peut
  être indéterminable. Les gardes `rows_affected == 0` et `last_insert_id == 0` restent.
- **`entities/audit_log.rs`** — `AuditLogEntry` dérive `Serialize` (camelCase) et `FromRow`.
  **Aucune route ne le sérialise aujourd'hui** (vérifié : `kesh-api` n'importe que
  `NewAuditLogEntry`) ; le champ ne change donc aucun contrat HTTP.
- **`post_restore.rs`** — `EXEMPT_MIGRATIONS` est un tableau `(version, ExemptionBasis, &str)`. La
  seule entrée `PerishableSince` existante (`20260909000001`, `:365-387`) est **le modèle à
  suivre**, justification comprise. `examples/perishable_exemptions.rs` lit le tableau comme une
  donnée : aucune modification à y faire, il imprimera deux lignes.
- **`backup.rs` `restore_body`** (`:425-521`) — `audit_log` est exclue du `DELETE` et insérée
  **sans son `id`**, colonnes prises dans le manifeste. L'ordre d'insertion est l'inverse de
  `TABLES_TO_TRUNCATE` : `companies` puis `users` sont réinsérées **avant** `audit_log`.

### Intelligence des stories précédentes

**25-1a** (même table, même famille de migration) :

- Quatre garde-fous se sont déclenchés en implémentation, **aucun redondant** : triage P7, checksum
  P8, site positionnel P6, compteur de routes admin. Les trois premiers se déclencheront ici.
- Échecs de gate rencontrés, à ne pas rediagnostiquer : **squash périmé** → `regen-test-schema.sh` ;
  **`ColumnNotFound`** → champ absent de `COLUMNS` ; **fixture sans société** → `users.company_id`
  est NOT NULL sans défaut.
- ⛔ **Une remise à zéro de base qui échoue en silence est indiscernable d'une régression** (34 faux
  échecs, `sqlx migrate run` redirigé vers `/dev/null`). La vérifier.
- `information_schema` rend `CHARACTER_MAXIMUM_LENGTH` en **`BIGINT UNSIGNED`** : décoder en `u64`.
  Même piège pour l'assertion de montage de l'AC 14 (d) si elle lit une longueur ou un compte.
- Le grep de propagation doit partir **des fichiers à couvrir**, pas des mots à trouver — cinq sites
  manqués sur cette seule story, dont un pour cause de **langue** du support.

**25-1b** :

- *Un inventaire ne se clôt ni par l'épuisement des fichiers, ni par celui des questions, mais par
  leur produit.*
- *Deux grandeurs différentes portant le même nombre sont indétectables à la relecture* — ici, 89
  désigne les **sites de construction** d'une entrée, pas les routes tracées (108 au registre).
- ⚠️ **Deux fautes de gate** : un seed sauté (29 tests sur « need at least one Admin user »), et un
  build frontend **en retard d'un périmètre** au gate E2E.

**24-5** (le précédent de l'exemption périssable) : le fondement « parc vide » a été **contrôlé par
`git tag`**, et ce contrôle est une tâche **refaite au gate de clôture**. Il le sera ici aussi.

### Ce que la story ne fait pas

- **La route et l'écran de consultation**, la méthode de lecture filtrée → **25-1c**.
- **Le sort des entrées au `company_id` d'une société remplacée** (AC 8, deux sous-cas) → **25-1c**,
  qui doit trancher avant d'afficher quoi que ce soit.
- **Ce qu'un écran scopé fait des entrées `NULL`** → **25-1c**.
- [#434] (onboarding), [#435] (session), [#431] (attribution par clé API) — hors épopée de la
  colonne.

### Project Structure Notes

- Migration : convention `AAAAMMJJNNNNNN_slug.sql`, un seul fichier DDL + backfill (condition de
  validité d'une éventuelle classe B, et règle de lisibilité du dépôt).
- Test de backfill : un binaire de test par migration de backfill dans `crates/kesh-db/tests/`,
  helpers de `tests/common/mod.rs`.
- Aucun conflit de structure détecté.

### References

- `_bmad-output/planning-artifacts/epic-25-vague1-suite.md` § 25-1, *Arbitrage du 2026-09-11*
- `_bmad-output/implementation-artifacts/25-1a-piste-inalterable.md` (décision de fusion, T1)
- Issue [#378] · voisines [#376], [#377], [#379] (closes), [#434], [#435], [#431]
- `crates/kesh-db/migrations/20260910000001_audit_log_actor_label.sql` — **patron de la migration**
- `crates/kesh-db/migrations/20260413000001_audit_log.sql`, `20260605000002_audit_log_actor.sql`
- `crates/kesh-db/src/repositories/audit_log.rs:52` (`COLUMNS`), `:61-121` (`insert_in_tx`)
- `crates/kesh-db/src/post_restore.rs:364-457` (`EXEMPT_MIGRATIONS`), `:971`, `:1049`, `:1072`
- `crates/kesh-db/src/backup.rs:34-77` (`TABLES_TO_TRUNCATE`), `:425-521` (`restore_body`)
- `crates/kesh-api/src/admin_backup/import.rs:195-236` (`check_schema_compat`)
- `crates/kesh-api/src/routes/admin.rs:444-455` (entrée `admin.full_import`)
- `crates/kesh-db/tests/closing_accounts_backfill.rs`, `tests/common/mod.rs` — **patron du test AC 14**
- `crates/kesh-api/tests/admin_full_import_e2e.rs:1847` — **patron des tests AC 7 et 8**
- `crates/kesh-db/tests/test_schema_guard.rs:556`, `scripts/regen-test-schema.sh`
- `docs/migrations-idempotence-audit.md` § *Maintenance future*
- `CLAUDE.md` § *Migration breaking policy* (P1 à P8), § *Test Locally First*, § *Propagation post-patch*

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-15** — Spécification créée (`bmad-create-story`). Story-zéro de la 25-1c, issue de
  l'arbitrage du 2026-09-11. Faits établis à la lecture et non repris de l'epic : ordre
  d'insertion du restore (`users` avant `audit_log`), **aucune route ne sérialise
  `AuditLogEntry`**, **aucune route ne crée de seconde société**, dernier tag publié **v0.11.1
  (2026-08-24)**, deux commentaires rendus faux (`exports.rs:137`, `exports_global_e2e.rs:1144`) et
  un décompte faux dans le commentaire modifié (`audit_log.rs:70`, « trente » pour 89). **Écart
  relevé avec l'epic** : le point ouvert du restore a **deux** sous-cas (AC 8), l'epic n'en nomme
  qu'un.
- **2026-09-15** — Contrôle contre `checklist.md` avant de sceller, **quatre affirmations
  vérifiées au sol** : un défaut réel corrigé — le test de backfill (AC 14) devait être inscrit à
  `ALLOWED_REAL_MIGRATOR_FILES`, faute de quoi `every_sqlx_test_attribute_is_accounted_for`
  rougissait — ; la graphie de l'attribut des tests AC 15 ajoutée ; `strip_column` confirmé
  suffisant (lignes lues par nom) ; `IF NOT EXISTS` confirmé sur MariaDB 10.11 aux trois sites.
