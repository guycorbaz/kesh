# Story 25.1c-zero : Rattacher chaque entrée d'audit à sa société

Status: review

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
  (`repositories/audit_log.rs:79-83`). ⇒ **aucun des 106 sites qui écrivent une entrée ne
  bouge** (38 fichiers, tous hors tests — recompté le 2026-09-15 par
  `grep -rnE "audit_log(_repo)?::insert_in_tx\(" crates/*/src --include=*.rs | wc -l` ; l'epic
  annonçait 89 sur 32), et `NewAuditLogEntry` ne gagne aucun champ.
- **Colonne `BIGINT NULL`, sans clé étrangère.**

**Ce qui le rend exact, vérifié et non supposé** :

- `users.company_id` est **NOT NULL**, écrit à la création, jamais modifié ; aucune table de
  jonction — un utilisateur appartient à **exactement une** société.
- **Aucune route n'écrit un audit sur une entité d'une autre société que celle de l'acteur** :
  aucun handler ne reçoit de `company_id` depuis la requête, et tout écart devient 404.
- Une mutation par clé API écrit `user_id` = le **créateur** de la clé (`entities/audit_log.rs:101-104`)
  ⇒ même société.
- `admin.full_import` écrit son entrée **après** le remplacement de `users`, dans la même
  transaction (`routes/admin.rs:442-453`), avec pour acteur le plus petit administrateur du **jeu
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

## ✅ Arbitrages du Project Lead — 2026-09-15

1. **Aucune release ne part avant le merge de cette story.** C'est la condition qui tient vrai le
   fondement de l'exemption périssable (AC 9) : une version taguée depuis `main` avant le merge se
   placerait dans l'intervalle `[20260827000001 .. 20260915000001)` que la justification déclare
   vide. ⚠️ **Si cette condition venait à être levée**, l'exemption devrait être reprise avant le
   tag — `scripts/prepare-release.sh` refuserait sinon la release **suivante**.
2. **Le partage avec la 25-1c est validé.** Cette story **mesure** les deux sous-cas du restore
   (AC 8, test de caractérisation) ; la 25-1c **tranche** ce qu'un écran scopé en fait. Ne rien
   résoudre ici.

## Acceptance Criteria

### Volet A — le schéma

**1. Une migration `crates/kesh-db/migrations/20260915000001_audit_log_company_id.sql`** ajoute à
`audit_log` :

- `company_id BIGINT NULL`, avec un `COMMENT` qui dit *pointeur logique, sans FK, `NULL` =
  société indéterminable* ;
- l'index **`idx_audit_log_company_date (company_id, created_at)`**.

Les deux clauses portent `IF NOT EXISTS` (précédents : `20260418000001_country_code.sql` pour la colonne, `20260419000001_invoice_paid_at.sql`
pour l'index, mais sous la forme `CREATE INDEX IF NOT EXISTS` — la forme `ALTER TABLE … ADD INDEX IF
NOT EXISTS` retenue ici n'a pas de précédent au dépôt ; elle a été **exécutée** sur MariaDB 10.11 en
passe 2 de validation, note 1061 au rejeu ; MariaDB **10.11** en dev, en production et en CI — les trois `image:` vérifiées), ce qui
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

**Montage — un seul aller-retour produit les deux sous-cas.** Un export puis un import sur la même
base ne donne **que** « identiques », puisque `restore_body` réinsère `companies` avec les `id` de
l'archive. Pour obtenir aussi « différents » :

1. `seed_admin("a")` crée la société C1 et l'administrateur U1. U1 écrit **deux** entrées,
   d'actions distinctes : `test.identiques`, laissée intacte, et `test.verbatim`, sur laquelle
   **seule** on pose ensuite un `company_id` que le sous-SELECT ne produirait jamais — une société
   inexistante, **hors de la plage des identifiants que le montage crée** (par exemple `999` : une petite
   valeur coïnciderait avec l'`id` que prendra C2), sur le patron de `nom_d_alors`
   (`admin_full_import_e2e.rs:1944`). Puis **export**.
   ⛔ *Deux entrées et non une* : la valeur posée pour l'assertion « verbatim » **écraserait** celle
   que l'assertion « identiques » exige, sur une ligne que la fusion conserve telle quelle.
2. `seed_admin("b")` crée **ensuite** C2 et U2, absents de l'archive ; U2 écrit une entrée sous une **troisième** action,
   `test.differents`. ⛔ Ne pas reprendre l'action d'une entrée de U1 : la règle « pour une même
   action, la copie locale est celle de plus petit `id` » suppose **une seule** entrée locale par
   action.
3. **Import** avec le JWT de U2. Le handler l'accepte : l'importateur « peut ne pas exister dans la
   source » (`routes/admin.rs:346-347`).

**Ce qu'on asserte, et pourquoi chaque assertion tranche** :

- l'entrée **locale** `test.identiques` porte C1 : c'est le sous-cas « identiques » ;
- l'entrée **locale** `test.differents` porte C2, que le restore a supprimée : c'est le sous-cas « différents » ;
- l'entrée `admin.full_import` porte **C1**, alors que l'importateur est dans **C2**. L'acteur est le
  plus petit administrateur **restauré** (`admin.rs:354-363`), et c'est cette différence qui rend
  l'assertion discriminante ;
- **« verbatim »** : après import, les **deux** copies de `test.verbatim` portent la valeur posée à
  l'étape 1, telle quelle — la locale, conservée, et celle **fusionnée depuis l'archive**, qui
  prouve que l'import n'a rien recalculé.

⚠️ **La fusion DUPLIQUE chaque entrée exportée** : `audit_log` n'est pas vidée au restore
(`backup.rs:444-447`), et les entrées de l'archive sont réinsérées sans leur `id`. Pour une même
action, la copie **locale** est donc celle de plus petit `id`, la copie **d'archive** celle d'`id`
neuf. Les assertions « locale » ci-dessus visent la première ; les compter sans les distinguer ne
tranche rien.

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
  jamais édité. Sans cela, tout test monté sur le squash qui écrit ou relit une entrée d'audit échoue en
  `Unknown column 'company_id'`
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
  société de son acteur**, et **aucune** celle de l'autre.
  ⛔ **Identifiants DÉSALIGNÉS, posés explicitement** (par exemple sociétés `10` et `20`,
  utilisateurs `101` et `202`, `entity_id` `7`) : dans une base neuve, la première société et le
  premier utilisateur prennent tous deux l'`id` 1, et un backfill fautif `SET a.company_id = a.user_id`
  ou `= u.id` passerait alors au vert ;
- (b) une entrée dont le `user_id` **n'existe pas** ⇒ `company_id` **reste `NULL`**, la migration
  ne s'arrête pas. *Il tue la mutation `JOIN` → `LEFT JOIN … COALESCE`* ;
- (c) **idempotence** : **après** la migration, poser à la main sur une entrée un `company_id` que le
  backfill ne produirait pas (celui de l'autre société), puis **rejouer le SQL embarqué** —
  `kesh_db::MIGRATOR.migrations` filtré sur la version `20260915000001`, fichier entier, et **jamais
  une copie** recopiée dans le test —, et asserter que la valeur posée **n'a pas bougé**.
  ⚠️ *Pourquoi ces trois précisions* : une copie resterait gardée quand la migration ne l'est plus,
  et la mutation 3 de l'AC 16 passerait au vert ; la valeur ne peut pas être posée **avant** la
  migration, la colonne n'existant pas (c'est l'assertion (d)) ; et l'assertion porte sur la
  **valeur**, pas sur `rows_affected`, dont le sens dépend du drapeau `CLIENT_FOUND_ROWS` de la
  connexion. Rejouer le fichier entier est sans risque : le DDL rejoué rend 0 ligne et les notes
  1060 et 1061 (vérifié en passe 2) ;
- (d) ⛔ **assertion de montage** : avant `MIGRATOR.run()`, la colonne `company_id` **n'existe
  pas** dans `information_schema`. Sans elle, un test qui tournerait après la migration **passerait
  à vide** — le mode d'échec de `backfill_skips_archived_accounts` (16-1a).

**15. L'alimentation** — dans `repositories/audit_log.rs` (`mod tests`), en
**`#[sqlx::test(migrations = "./test-schema")]`** (graphie exigée par `test_schema_guard.rs:58` ;
dans `kesh-api`, c'est `"../kesh-db/test-schema"`) :

- (a) acteur utilisateur ⇒ `company_id` = celui de l'utilisateur ;
- (b) constructeur `api_key` ⇒ la société du **créateur** de la clé ;
- (c) `user_id` inexistant ⇒ l'`INSERT` **réussit**, `company_id = None`, `actor_label = "(inconnu)"`.

⛔ **Même exigence qu'en AC 14 (a) : identifiants désalignés.** Le squash de test n'insère **que** la
ligne `_kesh_version` (`0001_schema_squash.sql:1110`) ; `test_fixtures::seed_accounting_company`
crée la société 1 puis les utilisateurs 1 et 2. Un sous-SELECT fautif `SELECT id FROM users`, ou
un `?` lié dans le mauvais ordre, rendrait alors la bonne valeur par coïncidence. ⇒ Créer d'abord
une société **factice**, et placer l'acteur dans la **seconde** : `companies.id`, `users.id`,
`entity_id` et `actor_api_key_id` doivent être **deux à deux distincts**.

⚠️ Les trois tests existants du module utilisent `#[tokio::test]` avec une base **partagée**
(`test_pool()`) et l'administrateur du seed : ne pas les imiter pour les nouveaux cas. Une base
éphémère évite de laisser des résidus qui feraient rougir le gate suivant (KF-039, #310).

**16. Épreuve par mutation, résultats consignés au Change Log** — chaque mutation appliquée, le
test visé exécuté et **vu rouge**, puis la mutation retirée et `git diff` vérifié vide :

| mutation | test qui doit rougir |
|---|---|
| sous-SELECT de l'AC 4 remplacé par `NULL` | 15 (a) et (b) |
| `JOIN` → `LEFT JOIN` + `COALESCE(u.company_id, 0)` | 14 (b) |
| `WHERE a.company_id IS NULL` retiré **de la migration** | 14 (c) — parce qu'il rejoue le SQL **embarqué** |
| sous-SELECT `SELECT company_id FROM users` → `SELECT id FROM users` | 15 (a) **et (b)** — même `INSERT` pour les deux constructeurs ; **seulement** si les identifiants sont désalignés |
| backfill `SET a.company_id = u.company_id` → `= a.user_id` | 14 (a) — **seulement** si les identifiants sont désalignés |
| entrée retirée d'`EXEMPT_MIGRATIONS` | `every_data_backfill_migration_is_triaged` |

⛔ **Un test qui ne compile pas ne rougit pas : il se tait** (acquis de la 25-1b). Chaque mutation
doit produire un **échec d'assertion**, pas une erreur de compilation — sinon elle ne prouve rien.

### Volet F — ce qui devient faux ailleurs

**17. Propagation — les affirmations que la story rend fausses**, à corriger dans le **même**
patch :

| site | ce qu'il affirme |
|---|---|
| `crates/kesh-api/src/routes/exports.rs:137-139` | « la table `audit_log` n'a pas de colonne `company_id`, donc les requêtes multi-tenant doivent passer par `details_json` ou par FK `users.company_id` » |
| ⛔ `docs/manual/fr/admin-manual.tex:1786` **et son PDF** | la liste des champs d'`audit_log`, puis « Il n'y a pas de colonne `company_id` : la table est globale. Une consultation par société l'exige, et c'est le sujet de l'issue #378. » — cf. AC 18 |
| `crates/kesh-api/tests/exports_global_e2e.rs:1144-1145` | « `audit_log` n'a PAS de colonne company_id ; on filtre par user_id » |
| `crates/kesh-db/src/repositories/audit_log.rs:70` | « toucher les quelque **trente** sites » — ils sont **106** ; la phrase est dans le commentaire même que l'AC 4 modifie |

⛔ **Ne PAS remplacer « trente » par un autre nombre** dans ce commentaire : le réécrire **sans
total** (« tous les sites qui appellent ce repository »). *Un énoncé qui dépend d'un total se
périme à chaque route ajoutée ; un énoncé qui nomme ses objets, non* (règle tirée de la 25-1a). Le
nombre recompté vit dans cette spec, daté et adossé à sa commande — et c'est parce que le « 89 » de
l'epic n'avait jamais été recompté qu'il était faux.

⚠️ Le filtre par `user_id` du test d'export **reste correct** : il n'y a pas lieu de le réécrire,
seulement de corriger le commentaire qui le motive.

**Grep de contrôle, à rejouer après le patch** — résultat **attendu** ci-dessous, tiré de
l'exécution réelle du grep et non d'une supposition :

```sh
grep -rnE "(n'a (PAS|pas) de|sans) (colonne )?\`?company_id" crates docs README.md
grep -nE "trente|\b(30|89|106)\b" crates/kesh-db/src/repositories/audit_log.rs
grep -rnE 'pas de colonne .{0,20}company(\\)?_id' docs/manual/fr/*.tex website README.md
pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "pas de colonne company_id"
```

- **Premier grep** : aujourd'hui **8 lignes**. Après le patch, il en reste **6**, qui parlent toutes
  **d'autres tables** et sont des faux positifs à laisser :
  - `kesh-api/src/routes/dunning_levels.rs:3` ;
  - `kesh-api/src/routes/profile.rs:43` ;
  - `kesh-api/tests/profile_e2e.rs:122` ;
  - `kesh-db/src/repositories/invoices.rs:2222` ;
  - `kesh-db/src/repositories/journal_entries.rs:701` ;
  - `kesh-db/src/repositories/journal_entries.rs:1106`.

  Les deux lignes qui disparaissent sont `exports.rs:138` et `exports_global_e2e.rs:1144`.
- **Second grep** : **aucune ligne** après le patch — ni l'ancien total, ni un nouveau.
- **Troisième et quatrième** (le manuel, `.tex` puis PDF aplati) : **aucune ligne** après le patch.
  ⛔ **En LaTeX, le souligné s'écrit `\_`** : un motif `company_id` ne voit **jamais**
  `company\_id`. C'est ainsi que l'AC 18 a d'abord conclu « aucun site » — sur un grep aveugle par
  construction. D'où le `(\\)?` et le contrôle du PDF.

⚠️ Les numéros de ligne des faux positifs peuvent dériver d'ici le développement : on juge le
résultat **fichier par fichier**, pas au nombre de lignes.

**18. Le manuel d'administration — UN site, que la story rend faux.**

`admin-manual.tex:1786` (§ *Audit-trail (audit\_log)*) énumère les champs de la table et affirme :
*« Il n'y a pas de colonne `company_id` : la table est globale. Une consultation par société
l'exige, et c'est le sujet de l'issue #378. »* Au merge, la liste devient incomplète et la phrase
fausse. ⇒ **dans le même patch** :

- ajouter `company_id` à la liste des champs, avec sa nature en une incise : *la société de
  l'auteur au moment de l'écriture, vide quand elle est indéterminable* ;
- réécrire l'avertissement **sans sur-promettre dans l'autre sens** : la colonne existe, mais
  **la consultation par société n'existe pas encore** — elle reste l'objet de l'issue #378 ;
- régénérer les PDF (`make fr` dans `docs/manual/`), commiter le PDF, et le **vérifier aplati** —
  l'ancienne phrase absente, la nouvelle présente.

⚠️ **Ce qui reste vrai et ne doit PAS être touché** : `user-manual.tex:498-502` (« ce qui manque
encore, c'est la consultation du journal d'audit ») — c'est la 25-1c qui le rendra faux.
`marketing-brochure.tex`, `README.md` et `website/` ne disent rien de la portée de la table
(vérifié par le grep de l'AC 17). Pas d'entrée CHANGELOG : aucun effet visible pour l'utilisateur
— la 25-1c la portera.

## Tasks / Subtasks

- [x] **T0 — Remise à zéro VÉRIFIÉE de la base de dev** avant tout (la machine a redémarré, le
      tmpfs est vide) : les trois étapes de `CLAUDE.md` § *Un gate laisse la base piégée*, **puis**
      contrôler qu'un `Admin` existe. *Compter les migrations ne prouve rien.*
      ⚠️ **À prévoir, pas à diagnostiquer** : entre T2 et T9, les trois `#[tokio::test]`
      historiques de `audit_log.rs`, qui lisent la base **partagée** `kesh`, échoueront en
      `Unknown column 'company_id'` tant qu'elle n'a pas reçu la 68ᵉ migration. Ce n'est pas une
      régression ; la remise à zéro de T9 la leur apporte.
- [x] **T1 — La migration** (AC 1, 2, 3, 13). En-tête **relu avant** le premier `migrate run`.
- [x] **T2 — Alimentation et entité** (AC 4, 5), commentaire `:70` corrigé au passage (AC 17).
- [x] **T3 — Squash et checksum** (AC 12) : `scripts/regen-test-schema.sh`, ligne `migrations.sha384`.
- [x] **T4 — P7** (AC 9) : entrée `PerishableSince`, justification sans le marqueur, trois nombres
      du tableau de l'AC 9.
- [x] **T5 — P5 et P6** (AC 10, 11), **compteurs recomptés depuis la source**, valeurs grepées.
- [x] **T6 — Tests** (AC 6, 7, 8, 14, 15). ⛔ *Une tâche qui décrit un test est une promesse ; la
      cocher sans l'avoir écrit la transforme en mensonge, et le gate reste vert* (acquis 25-1b).
- [x] **T7 — Mutations** (AC 16), résultats **observés** consignés.
- [x] **T8 — Propagation** (AC 17) et manuel d'administration (AC 18) : `.tex` corrigé, PDF
      régénéré par `make fr` et vérifié aplati, les quatre greps de l'AC 17 rejoués.
- [x] **T9 — Gate complet**, base remise à zéro **et vérifiée**. ⛔ `kesh-db` touché : **ciblage
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
| `docs/manual/fr/admin-manual.tex` + `.pdf` (et les deux autres PDF que `make fr` régénère) | § *Audit-trail* : champ ajouté, avertissement réécrit (AC 18) |

**Deux crates et un manuel, aucun frontend, aucune clé i18n** — sous le seuil de la § *Règle de splitting
préventif*.

### Ce qui ne bouge PAS, et qu'il ne faut pas « améliorer »

- **Les 106 sites qui écrivent une entrée d'audit**, et `NewAuditLogEntry`.
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
  est NOT NULL sans défaut. ⚠️ **Et `users` porte `CHECK (OCTET_LENGTH(password_hash) >= 20)`**
  (`20260404000001_initial_schema.sql:36`) : le montage en SQL brut de l'AC 14, avec un hachage
  factice court, échoue en `ERROR 4025` (relevé en passe 2).
- ⛔ **Une remise à zéro de base qui échoue en silence est indiscernable d'une régression** (34 faux
  échecs, `sqlx migrate run` redirigé vers `/dev/null`). La vérifier.
- `information_schema` rend `CHARACTER_MAXIMUM_LENGTH` en **`BIGINT UNSIGNED`** : décoder en `u64`.
  Même piège pour l'assertion de montage de l'AC 14 (d) si elle lit une longueur ou un compte.
- Le grep de propagation doit partir **des fichiers à couvrir**, pas des mots à trouver — cinq sites
  manqués sur cette seule story, dont un pour cause de **langue** du support.

**25-1b** :

- *Un inventaire ne se clôt ni par l'épuisement des fichiers, ni par celui des questions, mais par
  leur produit.*
- *Deux grandeurs différentes portant le même nombre sont indétectables à la relecture* — ici, 106
  désigne les **appels à `insert_in_tx`**, pas les routes tracées (108 au registre), ni les
  **constructeurs** de `NewAuditLogEntry` (110 appels, dont 16 par le trait `from_current_user`) — trois
  grandeurs voisines, trois nombres différents.
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
- `crates/kesh-api/src/routes/admin.rs:442-453` (entrée `admin.full_import`)
- `crates/kesh-db/tests/closing_accounts_backfill.rs`, `tests/common/mod.rs` — **patron du test AC 14**
- `crates/kesh-api/tests/admin_full_import_e2e.rs:1847` — **patron des tests AC 7 et 8**
- `crates/kesh-db/tests/test_schema_guard.rs:556`, `scripts/regen-test-schema.sh`
- `docs/migrations-idempotence-audit.md` § *Maintenance future*
- `CLAUDE.md` § *Migration breaking policy* (P1 à P8), § *Test Locally First*, § *Propagation post-patch*

## Dev Agent Record

### Agent Model Used

- **Implémentation** (`bmad-dev-story`) : Claude Opus 5 (1M context).
- **Validation de spec**, 4 passes : Sonnet + Haiku 4.5 → Opus → Sonnet → Opus (ciblée).

### Debug Log References

Aucun échec de gate à diagnostiquer pendant l'implémentation. Trois points **prévus par la spec**
et rencontrés tels quels :

- **P6** — le grep de la VALEUR (`\b(33|67)\b`) sur `migrations_upgrade_path.rs` a rendu, après les
  premières retouches, **quatre résidus** que l'énumération de l'AC 11 ne nommait pas : le message
  d'assertion d'`apply_migrations_up_to` (`total - 33`, `total == 67`), le doc-comment du test
  (`total - 33`, « les **33** dernières ») et l'étape 3 (« les 33 migrations restantes »). Tous
  corrigés ; le grep ne rend plus que des occurrences historiques (généalogie, lignes de story).
- **Base partagée** — la 68ᵉ migration a été appliquée à `kesh` avant le premier `nextest`, pour que
  les trois `#[tokio::test]` historiques de `audit_log.rs` ne rougissent pas en
  `Unknown column 'company_id'` (T0). L'en-tête de la migration était alors définitif (P8).
- **Grep de l'AC 17, second motif** — `grep -nE "trente|\b(30|89|106)\b"` rend **trois lignes**,
  toutes dans les tests neufs de l'AC 15 : `\b30\b` attrape l'`id` de la société **factice** (30).
  C'est un faux positif **du motif**, non un résidu : le total a bien disparu du commentaire. Le
  test n'a pas été modifié pour contenter le grep.

### Completion Notes List

- **T1 — migration** `20260915000001_audit_log_company_id.sql` : `ADD COLUMN IF NOT EXISTS company_id
  BIGINT NULL` + `ADD INDEX IF NOT EXISTS idx_audit_log_company_date`, puis `UPDATE … JOIN users …
  WHERE a.company_id IS NULL`. Aucun `NOT NULL`, aucune FK, aucun `COALESCE`, aucun
  `UPDATE _kesh_version`.
- **T2 — alimentation** : un troisième sous-SELECT dans `insert_in_tx`, `user_id` lié une troisième
  fois ; `COLUMNS` et `AuditLogEntry` gagnent `company_id` dans le même patch. Le commentaire qui
  annonçait « quelque trente sites » est réécrit **sans total**. `NewAuditLogEntry` et ses trois
  constructeurs sont inchangés : **aucun des 106 sites d'écriture ne bouge**.
- **T3** — squash régénéré par son script (rejeu vérifié, 39 tables) ; checksum ajouté à
  `migrations.sha384`, calculé par `sha384sum` après contrôle du mode de calcul sur la migration de
  la 25-1a.
- **T4 — P7** : exemption `PerishableSince(20260827000001)`, justification ouverte par « Parc
  vide », placée juste après `20260909000001` ; taille du registre 11 → 12, inventaire périssable à
  deux entrées. Le compteur « Hors fenêtre » reste à 6.
- **T5** — P6 : 68 / `total - 34`, frontière inchangée à 34, quatre résidus corrigés (cf. Debug
  Log). P5 : recompté **depuis la source** — `ls` = 68, lignes du tableau = 68, les deux totaux à
  68, partition `yes` 8 + `tracked-by-sqlx` 60 + `no` 0 = 68.
- **T6 — huit tests neufs**, dont l'existence et la sélection ont été **vérifiées par
  `nextest list`** avant les mutations :
  - `audit_log_company_id_backfill.rs` (AC 14) : `backfill_attributes_each_entry_to_its_actors_company`,
    `backfill_leaves_null_when_the_actor_no_longer_exists`,
    `backfill_is_idempotent_and_never_overwrites_a_set_value` — identifiants désalignés (10/20,
    101/202), assertion de montage factorisée dans `mount()`, rejeu du SQL **embarqué** par
    `sqlx::raw_sql` ; fichier inscrit à `ALLOWED_REAL_MIGRATOR_FILES` ;
  - `repositories/audit_log.rs` (AC 15) : `insert_sets_the_company_of_a_user_actor`,
    `insert_sets_the_company_of_an_api_key_creator`, `insert_with_an_unknown_actor_writes_a_null_company`
    — sociétés 30 (factice) et 40, utilisateur 501, `entity_id` 7, clé 9 ;
  - `admin_full_import_e2e.rs` (AC 7, 8) : `full_import_without_company_column_merges_archive_entries_as_null`
    et `characterization_full_import_keeps_company_ids_as_written` ;
  - l'AC 6 est une **assertion ajoutée** au test existant `full_export_structure_manifest_and_integrity`,
    non un test neuf.
- **T7 — mutations** : six sur six tuées, chacune par un **échec d'assertion**, fichiers restaurés et
  vérifiés identiques par `cmp` (détail au Change Log).
- **T8** — les deux commentaires rendus faux (`exports.rs`, `exports_global_e2e.rs`) réécrits ; le
  manuel d'administration corrigé (champ ajouté, avertissement reformulé sans sur-promettre) ; `make
  fr` a régénéré `admin-manual.pdf` et `user-manual.pdf` (la brochure est sortie identique). **PDF
  vérifié aplati** : l'ancienne phrase est absente, la nouvelle présente.
- **`backup.rs`, `export.rs`, `import.rs` : aucune ligne modifiée** — les AC 6 à 8 passent sans eux,
  ce qui confirme l'analyse du volet C.

**Décompte des tests, périmètre `main` → arbre de travail** : **+8** tests (3 + 3 + 2) ; **+1**
assertion dans un test existant.

### File List

**Backend — schéma et persistance**
- `crates/kesh-db/migrations/20260915000001_audit_log_company_id.sql` *(nouveau)*
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/test-schema/0001_schema_squash.sql` *(régénéré, jamais édité)*
- `crates/kesh-db/src/repositories/audit_log.rs`
- `crates/kesh-db/src/entities/audit_log.rs`
- `crates/kesh-db/src/post_restore.rs`
- `crates/kesh-db/tests/audit_log_company_id_backfill.rs` *(nouveau)*
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-db/tests/test_schema_guard.rs`

**Backend — API**
- `crates/kesh-api/src/routes/exports.rs` *(commentaire)*
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/admin_full_export_e2e.rs`
- `crates/kesh-api/tests/exports_global_e2e.rs` *(commentaire)*

**Documentation**
- `docs/migrations-idempotence-audit.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.pdf` *(régénéré par `make fr`, source inchangé)*

**Suivi**
- `_bmad-output/implementation-artifacts/25-1c-zero-audit-company-id.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

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

### Passe 1 de `bmad-create-story validate` — deux lentilles, contexte frais

Prompt versionné : `25-1c-zero-validate-prompt-p1.md`.

| Lentille | Modèle | Rendu brut | Après vérification au sol |
|---|---|---|---|
| Sonnet | Sonnet | 0 C, 0 H, **2 M**, 1 L | 3 findings, **tous confirmés** |
| Haiku | Haiku 4.5 | 0 C, 0 H, 0 M, 1 L | son LOW était déjà traité par l'AC 17 ⇒ **0** |

**Bilan : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 1 LOW.** Au-dessus de LOW ⇒ passe 2 requise.

- **M1 — le « 89 sites » était faux** : **106** appels à `insert_in_tx`, sur **38** fichiers, tous
  hors tests (vérifié : aucune occurrence après un `#[cfg(test)]`). Hérité de l'epic, jamais
  recompté. ⛔ **La spec allait faire remplacer un total faux par un autre total faux**, dans le
  commentaire même qu'elle modifie. → AC 17 exige désormais un commentaire **sans total** ;
  nombre corrigé à trois sites de la spec, dans l'epic (avec note datée) et dans le suivi.
- **M2 — la liste des faux positifs de l'AC 17 était supposée, pas exécutée** : deux entrées
  (`accounts.rs:246`, `reports.rs:288`) ne sortent pas du grep annoncé, et `profile_e2e.rs:122` en
  sort sans être nommée. → liste **régénérée depuis l'exécution réelle**, avec le résultat attendu
  avant et après le patch.
- **L3 — deux dérives de ligne** : `admin.rs:444-455` → `442-453`, `entities/audit_log.rs:102-105`
  → `101-104`.

⚠️ **La lentille Haiku a déclaré l'axe 3 exercé sans montrer l'inventaire des sites non résolus**,
qui en était l'objet : elle a relu les sites que la spec nommait. L'orchestrateur l'a donc refait
lui-même, par `grep -rnwE "67|33"` sur `crates/`, par la recherche de `SELECT *` sur `audit_log` et
par la recherche d'usages dans le frontend : **aucun site non listé**. Ce complément confirme la
spec, mais c'est lui, et non le « 0 finding » de la lentille, qui ferme l'axe.

**Leçon, et c'est la troisième fois qu'elle se présente sur l'Epic 25** : *un nombre recopié d'un
document de planification n'est pas un nombre vérifié* — le « 89 » avait été produit **pour
corriger** une estimation fausse (« ~30 »), et il était faux à son tour.

### Passe 2 de `bmad-create-story validate` — lentille unique (Opus), contexte frais

Prompt versionné : `25-1c-zero-validate-prompt-p2.md`. Base de comparaison déclarée par la
lentille : `git diff 08187010 -- _bmad-output/`. La syntaxe SQL a été **exécutée** sur une base
jetable créée puis supprimée par la lentille.

**Rendu : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 4 LOW — tous retenus après vérification au sol.**
Sévérité maximale **MEDIUM → HIGH** : la passe 2 est plus sévère que la passe 1. ⚠️ Ce n'est pas le
signal de non-convergence de la § *Règle de splitting préventif* : le HIGH ne vient **ni** d'une
remédiation de la passe 1, **ni** d'une décision de conception, mais d'un **axe que la passe 1 a
exercé avec un outil aveugle**.

- **H1 — l'AC 18 était FAUX : le manuel d'administration dit le contraire de ce que la story
  livre.** `admin-manual.tex:1786` énumère les champs d'`audit_log` et affirme « Il n'y a pas de
  colonne `company_id` : la table est globale ». Confirmé dans le PDF aplati. ⛔ **Pourquoi deux
  lentilles et l'auteur l'ont manqué** : en LaTeX le souligné s'écrit `\_`, et
  `grep "company_id" *.tex` ne voit **jamais** `company\_id`. L'AC 18 concluait « aucun site » sur un
  grep aveugle par construction ; le PDF, lui, l'aurait montré. → AC 18 réécrit (champ ajouté,
  avertissement reformulé sans sur-promettre, PDF régénéré et vérifié aplati), AC 17 complété de
  deux greps (`company(\\)?_id` sur le `.tex`, puis le PDF), T8 et tableau des fichiers mis à jour.
- **M2 — les tests AC 14 (a) et 15 ne distinguaient pas `company_id` de `user_id`.** Le squash
  n'insère que `_kesh_version` (`0001_schema_squash.sql:1110`, vérifié) : première société et premier
  utilisateur prennent l'`id` 1, et `SELECT id FROM users` passerait au vert. → identifiants
  **désalignés et deux à deux distincts** exigés, deux mutations ajoutées à l'AC 16.
- **M3 — la mutation 3 de l'AC 16 pouvait rester verte** si le test rejouait une **copie** de
  l'`UPDATE`. → (c) rejoue le SQL **embarqué** dans `MIGRATOR`, pose sa valeur **après** la
  migration, et asserte sur la **valeur** plutôt que sur `rows_affected` (sqlx 0.8.6 active
  `FOUND_ROWS`, vérifié à `stream.rs:46` — l'assertion ne doit pas en dépendre).
- **M4 — l'AC 8 ne disait pas comment obtenir le sous-cas « différents »**, ni comment rendre ses
  assertions discriminantes. → montage en trois temps (C1/U1, export, C2/U2, import par U2), validé
  contre `routes/admin.rs:346-347` (« l'importateur peut ne pas exister dans la source »).
- **M5 — un « 89 » a survécu dans `sprint-status.yaml:318`**, alors que le Change Log de la passe 1
  déclarait le suivi corrigé. ⛔ **Faute de propagation de l'orchestrateur** : la ligne portait
  **deux** occurrences, et la correction a visé **la phrase** au lieu du **jeton** — la leçon même de
  la 16-3b, que le `CLAUDE.md` codifie (« greper la VALEUR, pas la formulation »). Le grep avait
  été lancé ; son résultat, tronqué à 230 caractères par un `cut`, ne montrait pas la seconde.
- **L6** « 1102 tests » périmé (1203 montent le squash aujourd'hui) et mal nommé → énoncé sans total.
- **L7** précédent d'index cité sous une autre forme (`CREATE INDEX`) → citation corrigée.
- **L8** `CHECK (OCTET_LENGTH(password_hash) >= 20)` sur `users`, piège du montage SQL brut → Dev Notes.
- **L9** trois `#[tokio::test]` sur base partagée rougiront entre T2 et T9 → T0 le prévient.

**Grep de propagation après patch** (jetons `89` et `1102`, et toute conclusion « aucun manuel » ou
« pas de PDF » dans la spec) : aucun résidu.

**Deux leçons pour la rétrospective** :

1. ⛔ *Un grep a la graphie de son support.* Après la **langue** (25-1a, site web anglais), c'est
   la **syntaxe** : un manuel LaTeX échappe le souligné. Contrôler le **PDF aplati** n'est pas une
   précaution de plus, c'est le seul contrôle qui voit ce que lit l'utilisateur.
2. ⛔ *Un `cut` sur la sortie d'un grep de propagation peut masquer le résidu qu'on cherche* — une
   ligne longue du suivi en portait deux, l'affichage n'en montrait qu'une.

### Passe 3 de `bmad-create-story validate` — lentille unique (Sonnet), contexte frais

Prompt versionné : `25-1c-zero-validate-prompt-p3.md`. Base déclarée : `git diff 3fccc3fc --
_bmad-output/`. Six axes déclarés exercés ; idempotence du DDL et du backfill **exécutée** sur base
jetable, les quatre greps de l'AC 17 **rejoués** — ils rendent les comptes annoncés.

**Rendu : 0 CRITICAL, 1 HIGH, 1 MEDIUM — les deux confirmés : ils portent sur le texte même de la
remédiation de la passe 2.**

- **H1 — le montage de l'AC 8 était contradictoire.** U1 écrivait **une** entrée, qui devait à la
  fois porter C1 (sous-cas « identiques ») et recevoir, **avant l'export**, une société inexistante
  (assertion « verbatim ») ; la fusion conservant la ligne locale telle quelle, les deux ne pouvaient
  être vraies ensemble. → U1 écrit **deux** entrées d'actions distinctes, et la spec dit comment
  distinguer la copie locale de la copie d'archive, que la fusion **duplique**.
- **M2 — la table de mutations sous-déclarait 15 (b)** pour la mutation `SELECT id FROM users` :
  l'`INSERT` est commun aux deux constructeurs. → ligne corrigée.

**Sévérité maximale : HIGH (passe 2) → HIGH (passe 3).** ⚠️ Au sens littéral de la § *Règle de
splitting préventif*, c'est le signal de non-convergence — **signalé au Project Lead**. La boucle
continue sur deux motifs : le périmètre (deux crates, un manuel) n'est pas celui que la règle vise, et
**aucun finding des passes 2 et 3 ne met en cause la conception** ; ils suivent le motif mesuré du
dépôt, *la sévérité se déplace vers ce qu'on vient d'écrire* — le HIGH de la passe 3 est **né** de
la remédiation de la passe 2. Précédents de même courbe, convergés sans découpage : 24-4c et 24-5
(CRITICAL → HIGH → HIGH → rien).

⇒ **La passe 4 est une PASSE CIBLÉE**, braquée sur le seul correctif de la passe 3
(`25-1c-zero-validate-prompt-p4.md`). La passe 3 n'a rien trouvé hors du patch précédent : c'est
la condition d'emploi de la § *La passe ciblée*.

✅ **Arbitrage du Project Lead, 2026-09-15** : informé du signal HIGH → HIGH, **pas de découpage** —
« continue ».

### Passe 4 de `bmad-create-story validate` — PASSE CIBLÉE, lentille unique (Opus)

Prompt versionné : `25-1c-zero-validate-prompt-p4.md`. Base déclarée :
`git diff c3f91d8a -- …/25-1c-zero-audit-company-id.md`. Trois axes exercés, le reste de la spec
relu pour contradiction seulement. ⚠️ **Le montage de l'AC 8 a été EXÉCUTÉ** sur une base jetable
(MariaDB 10.11.16), séquence complète jusqu'à `admin.full_import` : les `id` obtenus confirment que la
copie locale est celle de plus petit `id`, qu'aucune entrée parasite ne fausse le décompte
(`admin.full_export` est écrite **après** l'archive ; `books.restored` n'est pas écrite, la borne
étant `NULL`), et que chaque assertion rougirait sous le défaut qu'elle vise.

**Rendu : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW.**

- **L1** — l'action de l'entrée de U2 n'était pas nommée ; reprendre celle d'une entrée de U1
  briserait la règle « une entrée locale par action ». → `test.differents` nommée, valeur verbatim
  imposée hors de la plage des identifiants créés (`999`).

⚠️ **Observation retenue sans sévérité** : dans les tests e2e, le squash aligne les identifiants
(C1 = U1 = 1, C2 = U2 = 2), si bien que la mutation `SELECT id FROM users` laisserait l'AC 8 **vert**.
Ce n'est pas un défaut — l'AC 16 confie cette mutation aux tests 15 (a) et (b), et l'AC 8 caractérise
un état, il ne chasse pas de mutation. ⇒ **Ne pas l'ajouter à la table de l'AC 16.**

---

## Boucle de validation — close en 4 passes

| Passe | Modèle(s) | CRITICAL | HIGH | MEDIUM | LOW | Origine des findings > LOW |
|---|---|---|---|---|---|---|
| 1 | Sonnet · Haiku 4.5 | 0 | 0 | 2 | 1 | spec d'origine (un total hérité de l'epic, une liste supposée) |
| 2 | Opus | 0 | 1 | 4 | 4 | spec d'origine (le HIGH : un grep aveugle au `\_` de LaTeX) **et** faute de propagation de la passe 1 |
| 3 | Sonnet | 0 | 1 | 1 | 0 | **tous nés de la remédiation de la passe 2** |
| 4 | Opus — **ciblée** | 0 | 0 | 0 | 1 | — |

**Critère d'arrêt atteint** : aucun finding au-dessus de LOW, et la remédiation de la passe 4 ne
touche **aucune ligne de code de production** — c'est une spec.

**Findings réfutés au sol : aucun.** Les **15** findings retenus des quatre passes — 3 + 9 + 2 + 1,
le LOW de la lentille Haiku, déjà traité par l'AC 17, n'étant pas compté — ont tous été vérifiés
avant application ; le seul rapport à 0 finding exploitable (Haiku, passe 1) a vu son axe 3 repris
par l'orchestrateur.

**Ce que la boucle apprend, pour la rétrospective de l'Epic 25** :

1. ⛔ *Un nombre recopié d'un document de planification n'est pas un nombre vérifié* — le « 89 » de
   l'epic avait été écrit **pour corriger** une estimation, et il était faux à son tour.
2. ⛔ *Un grep a la graphie de son support* : après la **langue** (25-1a), la **syntaxe** — LaTeX
   échappe le souligné. Seul le PDF aplati montre ce que lit l'utilisateur.
3. ⛔ *Un `cut` sur la sortie d'un grep de propagation peut masquer le résidu cherché.*
4. *La sévérité se déplace vers ce qu'on vient d'écrire* — vérifié une fois de plus : le HIGH de la
   passe 3 est né de la passe 2, et la passe ciblée l'a confirmé clos en exécutant le montage plutôt
   qu'en le relisant.

---

## Implémentation — `bmad-dev-story`, 2026-09-15

### Épreuve par mutation (AC 16) — résultats OBSERVÉS

Script versionné hors dépôt (scratchpad de la session) : chaque mutation appliquée par `sed` ou
Python, **nombre de sites mutés contrôlé par `grep -c`**, test visé exécuté par `nextest`, puis
fichier restauré depuis une copie. Pour les mutations de la migration, `lib.rs` est touché afin que
le SQL embarqué par `sqlx::migrate!` soit recompilé — sans quoi une mutation paraîtrait survivre à
tort.

| mutation | test attendu rouge | observé |
|---|---|---|
| M1 — sous-SELECT → `(SELECT NULL FROM users WHERE id = ?)` | 15 (a), 15 (b) | **FAIL** (a) et (b) sur assertion ; (c) PASS, attendu (`NULL` dans les deux cas) |
| M2 — `LEFT JOIN` + `COALESCE(u.company_id, 0)` | 14 (b) | **FAIL** sur assertion |
| M3 — `WHERE a.company_id IS NULL` retiré de la migration | 14 (c) | **FAIL** sur assertion |
| M4 — `SELECT company_id` → `SELECT id` | 15 (a), 15 (b) | **FAIL** (a) et (b) sur assertion |
| M5 — `SET a.company_id = a.user_id` | 14 (a) | **FAIL** sur assertion |
| M6 — entrée retirée d'`EXEMPT_MIGRATIONS` | `every_data_backfill_migration_is_triaged` | **FAIL** |

**Six sur six tuées, aucune par une erreur de compilation.** Restauration vérifiée par `cmp` sur les
trois fichiers mutés : identiques.

⚠️ **M1 n'est pas le « `NULL` nu » de la spec, et c'est délibéré** : remplacer le sous-SELECT par
`NULL` laisse un paramètre lié en trop, et le test échouerait sur une **erreur de requête** — un
échec qui ne prouve rien. `(SELECT NULL FROM users WHERE id = ?)` garde le même nombre de
paramètres et produit la valeur fautive que l'assertion doit voir.

### Gate — ce qui a RÉELLEMENT tourné

Base de dev **remise à zéro et vérifiée** juste avant : conteneur redémarré (tmpfs vidé),
`sqlx migrate run` — **68** migrations appliquées —, seed chargé, **1** administrateur contrôlé.

| Gate | Résultat |
|---|---|
| `cargo fmt --all -- --check` | vert |
| `cargo clippy --workspace --all-targets -- -D warnings` | vert, 0 warning |
| `cargo nextest run` via `scripts/test-fast.sh` | **2330 passed, 0 failed, 4 skipped** — 85,6 s |

**Le total se recoupe** : 2322 au gate de la 25-1b, **+ 8** tests neufs de cette story = 2330.

**Ce qui n'a PAS tourné, et pourquoi** :

- **Frontend** — non touché par la story (aucun fichier sous `frontend/`) : gate non requis pour le
  commit.
- ⚠️ **Suite E2E Playwright — non lancée.** C'est un prérequis du **push**, non du commit, et c'est
  **la seule** qui démarre un binaire contre une base persistante : elle seule verrait un défaut P8
  (checksum) que les bases éphémères de `nextest` ne voient pas. **À lancer avant tout push**, base
  `kesh_e2e` reconstruite.
