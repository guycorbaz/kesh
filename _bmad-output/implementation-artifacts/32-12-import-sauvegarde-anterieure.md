# Story 32.12 : une sauvegarde antérieure reste importable (refs #722)

Status: ready-for-dev

> ⚠️ **Story ajoutée par l'orchestrateur sous réserve du Project Lead** — question **Q4** du fichier d'epic
> (`planning-artifacts/epic-32-serveur-mcp.md` § *Questions pour Guy*), choix **C-32-0-13**. La fiche est écrite
> complète. **Si Guy préfère accepter la limite**, la story se réduit à **l'écrire** : une entrée du CHANGELOG
> (`## [0.16.0] — Non publié`, *Modifié*) et un paragraphe du manuel d'administration (§ *Importer*, liste des refus),
> qui disent qu'après la mise à jour vers la 0.16.0 une sauvegarde prise en 0.15.0 ou avant — automatiques comprises —
> ne se restaure plus que par une image 0.15.0, et qu'il faut prendre une sauvegarde complète **juste après** la mise à
> jour ; seuls les AC20 (manuel) et AC21 (CHANGELOG) subsistent alors, récrits dans ce sens, et le reste de la fiche
> tombe (registre : C-32-12-1).

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-12`, branche `story/32-12` partie de
`275e9b97`. Registre : `epic-32-choix-autonomes-3212.md` (C-32-12-1 à C-32-12-14). Clé courte : `3212`. 🗄️ touche
`kesh-db` (`backup.rs`, `post_restore.rs`) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Exception
kesh-db*). Aucune migration.*

## Story

En tant que Guy, qui tient sa comptabilité réelle dans Kesh et la sauvegarde chaque jour en 0.15.0, je veux qu'une
sauvegarde `.keshbackup` prise **avant** une mise à jour qui ajoute des tables reste importable **après** elle, afin
qu'un retour à l'état de la veille ne m'oblige pas à réinstaller l'ancienne image — et que ce qui n'existait pas encore
dans le fichier soit restauré **vide**, et me soit dit.

## Origine

- Prémisse 4 du fichier d'epic (§ *Ce que le code dit déjà*) : `parse_and_verify` exige un inventaire de tables
  **identique dans les deux sens** ; la 32-5 ajoute la première table depuis la 0.12.0 ; sans cette story, toutes les
  sauvegardes 0.15.0 deviennent non importables par la 0.16.0.
- Issue [#722] (« ce qui manque à un redémarrage complet », action A4 de la rétrospective de l'Epic 31) : restaurer une
  sauvegarde de la veille après une mise à jour est un redémarrage complet. Cette story n'en ferme qu'un point — `refs`,
  jamais `closes`.

## Ce que le code établit (relevé du 2026-10-24, `275e9b97`)

Le code de la branche **est** celui de la v0.15.0 : `git diff --stat v0.15.0 HEAD -- crates/ frontend/src` ne rend
rien, et aucune migration n'a été ajoutée depuis le tag (83 fichiers, le dernier `20261024000001_backup_schedule.sql`).

### Point 1 — savoir qu'une table absente est postérieure, sans se fier au nom

- **Aujourd'hui** : `crates/kesh-api/src/admin_backup/import.rs:228-247` — couverture (« chaque table de
  `TABLES_TO_TRUNCATE` figure au manifeste », message `table '{t}' absente du manifeste`) puis réciproque (« hors
  inventaire applicatif »). Aucune autre information n'est consultée.
- **Ce que le fichier déclare** : `manifest.kesh_version` (texte libre, écrit par `env!("CARGO_PKG_VERSION")`,
  `admin_backup/manifest.rs:116`) et `kesh_version_min_required`. Le manifeste **n'est pas signé** : les SHA-256 par
  table protègent l'intégrité, non l'authenticité. Et la version Cargo **ne bouge pas avec une migration non breaking**
  (P2-bis ne lie au bump que le relèvement de `min_required`) : la 32-5 ajoutera sa table alors que les crates portent
  encore `0.15.0`. La version déclarée ne peut donc **pas** fonder la règle (C-32-12-3).
- **Ce que le code sait** : chaque table applicative est créée par une migration, et `kesh_db::MIGRATOR`
  (`crates/kesh-db/src/lib.rs:28`) embarque le SQL de chacune (`m.sql`). Le test `last_table_creating_migration`
  (`crates/kesh-db/src/post_restore.rs:958`, `mod tests`) sait déjà reconnaître un `CREATE TABLE` d'une table de
  l'inventaire (`creates_application_table`, l. 978 ; SQL décommenté et découpé par `split_statements`, l. 772,
  production ; blancs normalisés par `normalize`, l. 1484, `mod tests`). D'où la **table → migration créatrice**,
  dérivée du `MIGRATOR` et non écrite à la main.
- **La règle retenue** (C-32-12-2, C-32-12-4) : une table de l'inventaire peut manquer au manifeste **si et seulement
  si** (a) sa migration créatrice est **postérieure au plancher d'import** `IMPORT_FLOOR_MIGRATION` (= `20260917000001`,
  dernière migration créatrice de table applicative à la v0.15.0 — `journal_entry_number_sequences`), **et** (b) elle
  est **postérieure à la migration créatrice de chaque table présente** — l'ensemble des absentes est un *suffixe* de
  l'ordre de création. Un fichier qui porte une table née en V6 sans celle née en V5 n'a pas été produit par un binaire
  situé entre les deux : refusé. La preuve vient du **schéma que le fichier porte**, non de ce qu'il déclare.
- **Pourquoi un plancher** : sans lui, la règle (b) seule rendrait importable une sauvegarde de la 0.11 (à qui
  manquent `invoice_settlements` et `journal_entry_number_sequences`, absentes « en suffixe ») et rouvrirait les huit
  exemptions « Hors fenêtre » (point 2). Le plancher **gèle la fenêtre d'importabilité à sa position actuelle** au lieu
  de la laisser se refermer à chaque table neuve.

### Point 2 — ce que la tolérance fait aux exemptions et au registre de rejeu

**Avec un plancher égal à la fenêtre actuelle, la fenêtre ne s'élargit pas : elle cesse de se refermer.** Toute table
créée au plus tard par `20260917000001` reste exigée, si bien que ce qui était refusé au contrôle de couverture l'est
toujours.

Les **huit** exemptions marquées « Hors fenêtre » (`EXEMPT_MIGRATIONS`, `post_restore.rs:414` et suivantes ; compte gardé
par `assert_eq!(checked, 8)`, l. 1126-1127), une à une :

| Exemption | Table invoquée (migration) | ≤ plancher ? | Reste vraie ? |
|---|---|---|---|
| `20260828000001` | `journal_entry_number_sequences` (`20260917000001`) | oui (= plancher) | oui |
| `20260910000001` | idem | oui | oui |
| `20260722000001` | `invoice_settlements` (`20260827000001`) | oui | oui |
| `20260729000001` | idem | oui | oui |
| `20260428000001` | `vat_rates` (elle-même) | oui | oui |
| `20260613000001` | `supplier_invoices` (`20260628000001`) | oui | oui |
| `20260614000001` | idem | oui | oui |
| `20260628000001` | `supplier_invoices` (elle-même) | oui | oui |

Les autres fondements : `20260917000001` (« couverture », non marquée — la table est le plancher, exigée : vraie) ;
`20260909000001` et `20260915000001` (`PerishableSince(20260827000001)`, fait daté contrôlé par
`scripts/prepare-release.sh` : leur argument « un backup importable porte `invoice_settlements` » reste vrai, et le
contrôle du script ne dépend pas de la fenêtre) ; tables système, `is_required()`, choix délibérés : indépendants de
l'inventaire. **Aucune exemption ne devient fausse.**

**Ce qui deviendrait faux, en revanche, c'est le calcul de la fenêtre par les tests**, dès la 32-5 :

- `registry_entries_are_within_import_window` (l. 929) et `exemptions_claiming_out_of_window_really_are_out_of_window`
  (l. 1083) prennent pour fenêtre `last_table_creating_migration()` — la **dernière** migration créatrice, toutes tables
  confondues. Après la 32-5, cette valeur passerait à la migration de la 32-5 alors que la fenêtre réelle resterait au
  plancher. Deux effets, tous deux muets :
  1. le premier test **rougirait sur l'entrée M1** (`20261010000001`, seule entrée du registre, `post_restore.rs:229`)
     et son message prescrit « Elle doit passer en exemption » — or une sauvegarde 0.12.x/0.13.0, antérieure à M1, reste
     importable et **a besoin** de ce rejeu. Suivre le message désactiverait le rejeu définitivement ;
  2. le second deviendrait **permissif** : une exemption « Hors fenêtre » d'une migration située entre le plancher et la
     32-5 passerait, alors qu'une sauvegarde 0.15.0 la déclencherait.
- **Donc** : la fenêtre des deux tests devient `IMPORT_FLOOR_MIGRATION` (AC9). Aujourd'hui les deux valeurs sont égales
  et la substitution est **équivalente** — elle ne devient observable qu'à la 32-5 (AC10, mutation M6 déclarée
  équivalente, AC19).
- **Le registre** : M1 reste dedans ; aucune entrée n'entre ni ne sort. Mais une table neuve postérieure au plancher
  dont la migration **écrit des lignes** (amorçage par société, comme `20260917000001` pour les compteurs) ne peut plus
  invoquer l'argument de couverture : une sauvegarde 0.15.0 arrive sans la table, la table est vidée (point 3), et ses
  lignes d'amorçage manqueraient. Une telle migration **va au registre** (AC11). `missing_sentinels`
  (`post_restore.rs:721`) traite déjà une table absente comme sentinelle manquante (`None => true`).

### Point 3 — l'état des tables absentes après l'import

- `restore_body` (`crates/kesh-db/src/backup.rs:627`) **vide toutes** les tables de `TABLES_TO_TRUNCATE` (sauf
  `onboarding_state` et `audit_log`), puis n'insère que celles que porte la sauvegarde (`let Some(data) =
  tables.get(table) else { continue; }`, l. 666-675) : une table absente du fichier **finit vide** — ni les lignes de la
  destination, ni les lignes que sa migration avait posées à la création. C'est le bon état (C-32-12-6) : garder les
  lignes de la destination mêlerait deux installations (des clés MCP désignant des utilisateurs d'un autre espace
  d'identifiants), et rejouer l'amorçage est l'affaire du registre (point 2).
- **Contrat pour les tables à venir** (32-5, 32-6, 32-10a) : une table postérieure au plancher doit être **valide
  vide** — le code lit l'absence de ligne comme l'état par défaut (serveur MCP désactivé, aucune clé, prompts standard
  servis depuis le code) —, ou amorcée par une entrée du registre (AC11). Écrit au doc-comment de
  `IMPORT_FLOOR_MIGRATION` (AC2) ; la 32-5 le reprend (demande à l'orchestrateur, *Hors périmètre*).
- **Tables système** : `_sqlx_migrations`, `_kesh_version`, `_kesh_backup_schedule` ne sont **pas** dans l'inventaire —
  exclues **par leur nom**, liste explicite (`backup_inventory_matches_schema`, `backup.rs:825-826`, et non par le
  préfixe `_kesh_`) ; l'import ne les touche pas : la ligne singleton de `_kesh_backup_schedule` reste celle de la
  destination (manuel, § *La sauvegarde automatique*, « un import ne le modifie pas »). Une future table système suit la
  même voie et n'est pas concernée par cette story.
- **Colonnes** : inchangé. `check_schema_compat` (`import.rs:579`) ne parcourt que les tables **présentes** ; une
  colonne ajoutée depuis à une table ancienne, `NOT NULL` sans défaut, reste un refus 400 (P1 : c'est une migration
  breaking, qui relève `min_required`).

### Point 4 — le bilan d'import

- Aujourd'hui : réponse `200` de `full_import` (`crates/kesh-api/src/routes/admin.rs:259-267`) — `backupCreated`,
  `tablesRestored`, `rowsRestored`, `sourceVersion`, `sessionInvalidated`, `filesRestored`, `filesMissing` ; détail de
  l'audit `admin.full_import` (`admin.rs:579-604`). L'écran (`AdminRestorePanel.svelte:240-280`, employé par
  `/admin/restore` **et** `/onboarding/restore`) affiche le succès, les pièces restaurées, les pièces manquantes.
- Ajout : `tablesAbsent: string[]` (toujours présent, vide quand rien ne manque, ordre de création) à la réponse,
  `tables_absent` au détail d'audit, une ligne d'avertissement à l'écran quand la liste n'est pas vide (AC12-AC14).

### Point 5 — la fixture `.keshbackup` 0.15.0

- Précédent exact : `generate_fixture_0_14_0` (`crates/kesh-api/tests/admin_full_export_e2e.rs:511`, `#[ignore]`),
  exécuté une fois au tag `v0.14.0` dans un worktree temporaire ; fixture versionnée
  `crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (12 490 octets), lue par `include_bytes!`
  (`admin_full_import_e2e.rs:3608`), jamais régénérée.
- Ici, **pas de worktree temporaire** : la branche porte le code de la v0.15.0 (ci-dessus). La fixture se produit au
  **premier commit de développement, avant toute modification de code** (C-32-12-10), par un générateur jumeau
  `generate_fixture_0_15_0` (`#[ignore]`), même contenu synthétique que celui de la 0.14.0 (société de
  `seed_accounting_company`, « Client Fictif », facture figée, document fournisseur sans QR), via le vrai
  `GET /api/v1/admin/full-export` : format 2, `keshVersion` `0.15.0`, deux pièces sous `files/`. Aucune donnée réelle,
  rien de `~/kesh-test/` (`memory` *Données réelles hors dépôt public*).

## Critères d'acceptation

### A — La règle d'inventaire (`kesh-db`)

1. **AC1 — table → migration créatrice, dérivée du `MIGRATOR`.** `kesh_db::backup::table_creating_migrations()` rend,
   pour chaque table de `TABLES_TO_TRUNCATE`, la version de la migration qui la crée — calculée une fois
   (`std::sync::OnceLock`) depuis `crate::MIGRATOR`, par la reconnaissance de `CREATE TABLE` aujourd'hui confinée aux
   tests (`creates_application_table`, `normalize`), **déplacée en production** (`pub(crate)`, dans `post_restore.rs`
   à côté de `split_statements`, ou dans `backup.rs` — un seul exemplaire, DRY) et réemployée par
   `last_table_creating_migration` des tests. Tests (`backup.rs`, `mod tests`) : (i) **chaque** table de l'inventaire a
   exactement **une** migration créatrice — zéro ou deux font échouer le test en nommant la table (une table recréée
   par une seconde migration est un cas que la règle ne sait pas traiter : il doit se voir) ; (ii) la carte rend
   `20260917000001` pour `journal_entry_number_sequences`, `20260827000001` pour `invoice_settlements`,
   `20260428000001` pour `vat_rates`. En production, une table sans migration connue est **exigée** (repli sûr : refus
   plutôt que tolérance).
2. **AC2 — le plancher.** `pub const IMPORT_FLOOR_MIGRATION: i64 = 20260917000001;` dans `backup.rs`, avec un
   doc-comment qui dit : ce qu'il est (la fenêtre d'importabilité, gelée à la v0.15.0) ; que le **relever** referme la
   fenêtre et rend non importables les sauvegardes antérieures — décision du Project Lead, CHANGELOG à l'appui, jamais
   un geste de maintenance ; et le **contrat** des tables postérieures (valides vides, ou amorcées par le registre —
   point 3). Tests : (i) le plancher est une migration du `MIGRATOR` qui crée une table de l'inventaire ; (ii)
   `IMPORT_FLOOR_MIGRATION <= last_table_creating_migration()` ; (iii) **à cette story**, l'ensemble des tables exigées
   (créées au plus tard par le plancher) est **égal** à `TABLES_TO_TRUNCATE` — c'est ce qui prouve que le plancher
   couvre tout l'inventaire de la 0.15.0 ; (iv) la liste des tables **tolérées** (créées après le plancher) est
   comparée à une liste écrite en dur dans le test, **vide** aujourd'hui (`assert_eq!(tolerees, Vec::<&str>::new())`) :
   chaque story qui ajoute une table l'y inscrit, et ce geste force à relire le contrat (patron du compteur `checked`
   de `post_restore.rs`).
3. **AC3 — la vérification pure.** `kesh_db::backup::check_backup_inventory(present: &BTreeSet<&str>, regles:
   &InventoryRules) -> Result<Vec<&'static str>, InventoryError>` — les tables absentes tolérées, dans l'ordre de
   création, ou l'erreur. `InventoryRules { inventaire, creation: fn(&str) -> Option<i64>, plancher }` (ou équivalent),
   `InventoryRules::production()` construit depuis `TABLES_TO_TRUNCATE`, `table_creating_migrations()` et
   `IMPORT_FLOOR_MIGRATION` ; les tests injectent les leurs. Trois erreurs, chacune nommant la table :
   - `Inconnue(t)` — `t` présente au manifeste, hors inventaire (règle actuelle) ;
   - `Exigee(t)` — `t` absente et créée au plus tard par le plancher (ou sans migration connue) ;
   - `HorsOrdre { absente, presente }` — `absente` postérieure au plancher mais antérieure (ou égale) à la création
     d'une table `presente`.
   Tests unitaires (inventaire et carte injectés, sans base) : inventaire complet → `Ok(vec![])` ; une table postérieure
   absente → `Ok([t])` ; deux postérieures absentes → `Ok([t1, t2])` dans l'ordre de création ; la plus ancienne des deux
   absente, la plus récente présente → `HorsOrdre` ; une table du plancher absente → `Exigee` ; une table inconnue →
   `Inconnue` ; deux tables nées dans **la même** migration, une seule absente → `HorsOrdre` (même version : le
   fichier ne peut pas porter l'une sans l'autre) ; inventaire de production (`InventoryRules::production()`) et
   manifeste complet → `Ok(vec![])`.
4. **AC4 — une table absente est vidée, même si la destination avait des lignes.** Test `kesh-db`
   (`#[sqlx::test(migrations = "test-schema")]`, `backup.rs` ou `crates/kesh-db/tests/`) : la destination porte un
   produit (`products`) ; `restore_tables_in_tx` reçoit une carte qui omet `products` et porte `companies` ; après,
   `products` est vide et `tables_restored` ne la compte pas. Le comportement existe (`backup.rs:666-675`) : le test le
   **fixe**, et le commentaire « Dans le flux réel, `parse_and_verify` garantit la couverture » est récrit (la
   couverture n'est plus complète : une table postérieure au plancher peut manquer, et elle est vidée **à dessein**).

### B — L'import (`kesh-api`)

5. **AC5 — `parse_and_verify` applique la règle.** Les deux boucles de `import.rs:228-247` sont remplacées par un appel
   à `check_backup_inventory` avec `InventoryRules::production()`, par une porte `parse_and_verify_with(reader, limits,
   regles)` que `parse_and_verify` enveloppe (patron de `replay_with_registry` : porte de test, `#[doc(hidden)]`).
   Messages `InvalidBackupStructure` (→ 400) : `Exigee` garde **mot pour mot** l'actuel `table '{t}' absente du
   manifeste` (les tests existants qui le cherchent restent verts sans changement — `parse_rejects_missing_table_coverage`,
   `import.rs:899`) ; `Inconnue` garde l'actuel « présente dans le manifeste mais hors inventaire applicatif » ;
   `HorsOrdre` : `table '{absente}' absente du manifeste alors que '{presente}', créée après elle, y figure`.
   `ParsedBackup` porte `tables_absent: Vec<String>`.
6. **AC6 — les refus restent des refus, avant tout effet.** Tests HTTP (`admin_full_import_e2e.rs`) sur la fixture
   0.15.0 récrite par `ouvrir_tout`/`rezip_tout` (`admin_full_import_e2e.rs:3620` et suivantes) : (i) `products` retirée
   du manifeste et de l'archive → `400 INVALID_BACKUP_STRUCTURE`, message `absente du manifeste` ; (ii) une table
   inconnue ajoutée → 400 ; dans les deux cas, la base de destination est **intacte** (une ligne témoin relue) et
   **aucune** sauvegarde pré-import n'a été écrite (`KESH_ADMIN_BACKUP_DIR` vide).
7. **AC7 — la chaîne d'import tolère une table postérieure, bout à bout hors HTTP.** Test d'intégration
   (`admin_full_import_e2e.rs`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`) qui **simule** une table
   postérieure au plancher, faute d'en avoir une avant la 32-5 : règles injectées où `products` est créée après le
   plancher ; fixture 0.15.0 privée de `products` ; destination qui porte un produit. `parse_and_verify_with` → `Ok`,
   `tables_absent == ["products"]` ; `check_schema_compat` → `Ok` ; `restore_tables_in_tx` → `products` **vide**, les
   autres tables restaurées (une facture de la fixture relue) ; `replay_post_restore_backfills` → `Ok`. Ce test
   n'emprunte pas le handler : la recopie de `tables_absent` dans la réponse et l'audit est prouvée par l'AC8 (liste
   vide) et, avec une liste non vide, à la 32-5 (AC10).
8. **AC8 — la fixture 0.15.0 s'importe, et le bilan le dit.** `ac_la_sauvegarde_de_la_0_15_0_s_importe` (patron de
   `ac10_la_sauvegarde_de_la_0_14_0_s_importe`, `admin_full_import_e2e.rs:3881`) : `200`, `sourceVersion` `"0.15.0"`,
   `tablesAbsent` **présent et vide** (`[]`, non absent), `filesRestored` `2`, `filesMissing` `0` ; détail d'audit
   `tables_absent: []`, `source_format_version: 2`. Le test de la 0.14.0 gagne l'assertion `tablesAbsent == []`
   (même inventaire). Une fixture générée par un binaire ultérieur, qui porterait des tables que celui-ci n'a pas,
   tombe sous `Inconnue` : c'est le symétrique, inchangé.

### C — Le rejeu des backfills (`post_restore.rs`)

9. **AC9 — la fenêtre des tests est le plancher.** `registry_entries_are_within_import_window` et
   `exemptions_claiming_out_of_window_really_are_out_of_window` comparent à `crate::backup::IMPORT_FLOOR_MIGRATION`, et
   non plus à `last_table_creating_migration()` ; leurs messages d'échec et le doc-module (§ *La fenêtre
   d'importabilité*, `post_restore.rs:19-37` ; commentaires l. 251, 264-281) sont récrits : *la fenêtre est le plancher,
   et une table neuve ne la referme plus*. Le compteur `checked` reste à **8**. `last_table_creating_migration` reste,
   pour l'AC2 (ii).
10. **AC10 — contrat écrit pour la 32-5.** Le doc-comment d'`IMPORT_FLOOR_MIGRATION` et le Dev Notes de cette fiche
    nomment ce que la 32-5 (et toute story qui ajoute une table) doit faire : inscrire la table à la liste de l'AC2
    (iv) ; constater que `registry_entries_are_within_import_window` reste **vert** sur M1 (c'est la preuve que l'AC9
    tient) ; reprendre l'AC8 pour la fixture 0.15.0 avec `tablesAbsent == [<sa table>]` et la table vide après
    l'import ; prendre les captures de l'état « tables absentes » (AC14).
11. **AC11 — une table postérieure amorcée va au registre.** Test `post_restore.rs` :
    `post_floor_table_seeds_are_replayed` — pour toute migration de version `> IMPORT_FLOOR_MIGRATION` qui **crée** une
    table de l'inventaire **et** porte un statement d'écriture dont la cible est cette table (premier mot-clé
    `INSERT`/`REPLACE`/`UPDATE`/`DELETE` et nom de table qui suit `INTO`/`UPDATE`/`FROM`), la migration figure à
    `POST_RESTORE_BACKFILLS` — pas à `EXEMPT_MIGRATIONS`. Vacant aujourd'hui (aucune migration postérieure au plancher
    ne crée de table) : sa matière vient de l'AC19 (M7) et, à la 32-5, du réel. Un `UPDATE _kesh_version` dans la même
    migration n'est pas une écriture **dans** la table créée et ne déclenche rien.

### D — Le bilan

12. **AC12 — réponse et audit.** `full_import` ajoute `"tablesAbsent": parsed.tables_absent` à la réponse
    (`admin.rs:259`) et `"tables_absent"` au détail d'audit (`admin.rs:579`), relevés **avant** que `parsed` ne soit
    consommé ; `ImportOutcome` ou `ParsedBackup` les porte jusque-là. `FullImportResponse`
    (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:12`) gagne `tablesAbsent?: string[]`, optionnel
    (absent d'un serveur 0.15.0), doc-comment à la manière de `filesMissing`.
13. **AC13 — l'écran.** `AdminRestorePanel.svelte`, sous le bloc des pièces manquantes : quand `tablesAbsent` est non
    vide, un encart `role="status"` d'avertissement (mêmes classes que `admin-restore-files-missing`),
    `data-testid="admin-restore-tables-absent"`, clé `admin-restore-tables-absent` :
    « Données nées après la version de la sauvegarde, laissées vides : { $tables }. Elles se recréent depuis
    l'application. » (`$tables` : les noms joints par « , »). Rien quand la liste est vide ou absente. Clé dans les
    **quatre** catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, à côté de
    `admin-restore-files-missing`, l. 1769 / 1698). Vitest (`AdminRestorePanel.test.ts`) : réponse simulée avec
    `tablesAbsent: ['mcp_keys']` → l'encart et le nom ; `[]` → pas d'encart ; champ absent → pas d'encart.
14. **AC14 — recette visuelle.** Captures, thème clair, de `/admin/restore` après l'import de la fixture 0.15.0 :
    l'état nominal (pas d'encart). L'état « tables absentes » ne peut pas être produit **pour de vrai** avant qu'une
    table postérieure existe : sa capture est **transférée à la 32-5** (AC10), et la fiche le dit au Dev Agent Record —
    ne pas la fabriquer en simulant une réponse dans le navigateur (C-32-12-11).

### E — La fixture

15. **AC15 — produite par le vrai exporteur de la 0.15.0, avant tout changement de code.** Premier commit de
    développement : `generate_fixture_0_15_0` (`#[ignore = "générateur de fixture : à n'exécuter qu'avec le code de la
    v0.15.0 (Story 32-12, AC15)"]`, `admin_full_export_e2e.rs`), dont le seed est **partagé** avec
    `generate_fixture_0_14_0` (fonction extraite, DRY ; le générateur 0.14.0 garde son doc-comment et son `#[ignore]`) ;
    exécuté une fois, **après** avoir constaté `git diff --stat v0.15.0 HEAD -- crates/` vide ; sortie
    `crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup`. Contrôle par `unzip -l` / `unzip -p … manifest.json` :
    `formatVersion: 2`, `keshVersion: "0.15.0"`, deux entrées `files/<sha>.pdf`, 39 tables. Taille et SHA-256 au Dev
    Agent Record ; le doc-comment du générateur dit pourquoi il est `#[ignore]` et ne doit jamais être relancé avec un
    code ultérieur (il produirait une sauvegarde qui n'est plus celle de la 0.15.0).

### F — Documentation et gates

16. **AC16 — les textes qui disent « identique ».** Récrits au même commit : doc-comment de `TABLES_TO_TRUNCATE`
    (`backup.rs:34-38`, « y ajouter une table rendrait non importable toute sauvegarde antérieure ») ; commentaire de
    `restore_body` (`backup.rs:647`, « L'en sortir rendrait tout backup existant inimportable » — vrai pour un
    **retrait**, à préciser) ; doc-module de `post_restore.rs` (AC9) ; `CLAUDE.md` § *Migration breaking policy* P7,
    paragraphe « Trier, ce n'est pas forcément inscrire au registre » (« inventaire identique au nôtre » → inventaire
    au moins égal à celui du plancher, absentes postérieures, fenêtre = plancher) — modification du `CLAUDE.md` soumise
    à Guy par la PR.
17. **AC17 — symptôme grepé, valeur comprise.** Après chaque patch, depuis la racine du worktree :
    `grep -rnE "identique|dans les deux sens|referme la fenêtre|refermer la fenêtre|last_table_creating_migration" crates/kesh-db/src crates/kesh-api/src CLAUDE.md docs/manual/fr/admin-manual.tex`
    — chaque ligne rendue est soit récrite, soit justifiée au Dev Agent Record (les relevés historiques datés —
    « la création de `invoice_settlements` a refermé la fenêtre », Story 24-2 — restent : ils disent vrai à leur date).
    Et la valeur : `grep -rn "20260917000001" crates/kesh-db/src` — chaque occurrence relue contre le sens neuf
    (plancher et non plus « dernière migration créatrice »).
18. **AC18 — gates au dernier commit de code.** `kesh-db` touché : **gate complet même en boucle** — `scripts/test-fast.sh`
    sous verrou (`docs/agents-orchestration.md` § 5), base `kesh_3212` remise à zéro d'abord ; `cargo publish --dry-run
    -p kesh-import` n'est pas concerné ; les quatre gates frontend (`npm run check`, `lint-i18n-ownership`,
    `test:unit`, `build`) ; **E2E complet** (frontend et backend changent), jugé fichier par fichier contre
    `docs/testing.md` § *Les échecs attendus*. Garde de parité i18n (`parity_between_locales`) verte. Ne déclarer que ce
    qui a tourné.
19. **AC19 — mutations** (sous verrou ; restauration `git checkout -- f && touch f`) : chacune doit faire rougir au
    moins un test **nommé** au Dev Agent Record.
    - M1 : `check_backup_inventory` tolère toute table absente (condition « postérieure au plancher » retirée) → AC3
      (`Exigee`) et AC6 (i) rouges.
    - M2 : règle du suffixe retirée → AC3 (`HorsOrdre`, et le cas « même migration ») rouge.
    - M3 : `IMPORT_FLOOR_MIGRATION = 20260827000001` (plancher abaissé d'un cran) → AC2 (iii) et (iv) rouges
      (`journal_entry_number_sequences` deviendrait tolérée).
    - M4 : `restore_body` ne vide plus une table absente de la carte (`DELETE` conditionné à `tables.contains_key`) →
      AC4 et AC7 rouges.
    - M5 : `full_import` omet `tablesAbsent` → AC8 rouge (champ absent ≠ `[]`).
    - M6 : AC9 défait (fenêtre = `last_table_creating_migration()`) → **équivalente aujourd'hui** (les deux valeurs
      sont `20260917000001`) : la déclarer telle, avec la raison, et la reporter à la 32-5 (AC10).
    - M7 : test de l'AC11 nourri d'une migration **fictive** (registre et `MIGRATOR` injectés par une porte de test, ou
      fonction de détection testée sur un SQL littéral `CREATE TABLE t (...); INSERT INTO t ...` de version > plancher)
      → la détection rend « à inscrire » ; sans la porte, déclarer l'AC11 vacant et le dire.
    - M8 : `creates_application_table` ne retire plus les accents graves → AC1 (i) reste-t-il vert ? Si oui (aucune
      migration du dépôt n'écrit `` CREATE TABLE `x` ``), mutation équivalente sur le dépôt actuel, à déclarer.
20. **AC20 — manuel d'administration, et son PDF.** `docs/manual/fr/admin-manual.tex`, § *Importer (restaurer la même
    installation)* (l. 2242 et suivantes) : (i) à la liste des refus (l. 2250-2262), l'item « une **table** que la
    version installée exige manque à la sauvegarde, ou la sauvegarde porte une table que la version installée ne
    connaît pas (sauvegarde d'une version plus récente) » ; (ii) un paragraphe *Une sauvegarde d'une version
    antérieure* : depuis la 0.16.0, une sauvegarde qui s'importait **avant** une mise à jour s'importe encore **après**,
    même quand la mise à jour a ajouté des tables (le manuel ne cite pas de version plancher : une colonne
    obligatoire ajoutée depuis à une table ancienne reste un refus de schéma, indépendant de cette story) ; ces tables
    sont restaurées **vides** — ce qu'elles contenaient sur l'installation est effacé, comme tout le reste — et le bilan
    les nomme ; exemple : une sauvegarde 0.15.0 importée en 0.16.0 restaure un serveur MCP désactivé et sans clé ; (iii)
    § *Ce que dit l'écran à la fin de l'import* (l. 2277) : l'encart de l'AC13 ; (iv) § *Reprises de données rejouées à
    l'import* (l. 2305) : rien à changer **sauf** si une phrase y lie l'importabilité à l'inventaire identique (à
    vérifier). `make fr` sous verrou ; contrôle **aplati** du PDF :
    `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "restaurées vides"` (et la phrase
    de l'encart). Manuels DE/IT/EN : sans objet (non traduits) ; manuel utilisateur : rien (l'import est réservé à
    l'Administrateur — à vérifier par `grep -n "keshbackup" docs/manual/fr/user-manual.tex`).
21. **AC21 — CHANGELOG.** `## [0.16.0] — Non publié` (créé s'il n'existe pas encore — union avec les autres stories de
    l'epic), *Modifié* : « Une sauvegarde complète qui s'importait avant une mise à jour reste importable après une
    mise à jour qui ajoute des tables : ces tables sont restaurées vides, et le bilan de l'import les nomme
    (Story 32-12, refs #722). » Le CHANGELOG ne promet rien de la 32-5.

## Tâches

- [ ] **T0** — Worktree `kesh-32-12`, branche `story/32-12` (déjà branchée) ; base `kesh_3212`, `kesh_e2e_3212`, port
  attribué par l'orchestrateur, `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-12/target` (`docs/agents-orchestration.md`
  § 4). Disque : vérifier la place libre avant le premier build (`memory` *Disque et worktrees*).
- [ ] **T1 (AC15)** — **avant tout code** : `git diff --stat v0.15.0 HEAD -- crates/` vide constaté ; extraction du
  seed partagé, `generate_fixture_0_15_0`, exécution unique (`cargo nextest run --run-ignored only -E
  'test(generate_fixture_0_15_0)'` ou équivalent), contrôle `unzip`, commit dédié de la fixture et du générateur.
- [ ] **T2 (AC1-AC4)** — `kesh-db` : reconnaissance `CREATE TABLE` déplacée en production, `table_creating_migrations`,
  `IMPORT_FLOOR_MIGRATION` et son doc-comment, `InventoryRules`, `check_backup_inventory`, tests unitaires ; test de
  l'AC4 ; commentaires de `TABLES_TO_TRUNCATE` et `restore_body` (AC16).
- [ ] **T3 (AC9, AC11)** — `post_restore.rs` : fenêtre = plancher dans les deux tests, doc-module et messages récrits,
  `post_floor_table_seeds_are_replayed`.
- [ ] **T4 (AC5-AC8, AC12)** — `kesh-api` : `parse_and_verify_with`, messages, `tables_absent` ; réponse et audit ;
  tests AC6, AC7, AC8 (et assertion ajoutée au test 0.14.0). Gate **complet** (kesh-db touché).
- [ ] **T5 (AC12-AC14)** — frontend : type, encart, clé × 4, Vitest ; captures de l'état nominal.
- [ ] **T6 (AC16, AC17, AC20, AC21)** — `CLAUDE.md` P7, manuel FR, `make fr`, PDF aplati ; CHANGELOG ; greps de
  l'AC17.
- [ ] **T7 (AC18, AC19)** — mutations M1-M8 ; gate complet, frontend, E2E complet au dernier commit de code ; Dev Agent
  Record : chiffres réellement exécutés, M6 (et M8 le cas échéant) déclarées équivalentes avec la raison.

## Dev Notes

### Sites

| Site | Rôle aujourd'hui | Ce qui change |
|---|---|---|
| `crates/kesh-api/src/admin_backup/import.rs:228-247` | couverture + réciproque, inventaire identique | `check_backup_inventory` |
| `crates/kesh-api/src/admin_backup/import.rs:86-93` | `ParsedBackup` | `tables_absent` |
| `crates/kesh-api/src/routes/admin.rs:259-267, 579-604` | réponse, audit | `tablesAbsent`, `tables_absent` |
| `crates/kesh-db/src/backup.rs:34-104` | `TABLES_TO_TRUNCATE` (39 tables) | doc-comment ; `IMPORT_FLOOR_MIGRATION`, carte, règle à côté |
| `crates/kesh-db/src/backup.rs:627-720` | `restore_body` — vide tout, insère le présent | commentaire seulement (comportement fixé par AC4) |
| `crates/kesh-db/src/post_restore.rs:19-37, 251, 264-281` | « toute table neuve referme la fenêtre » | plancher |
| `crates/kesh-db/src/post_restore.rs:929, 958, 978, 1083, 1484` | fenêtre des tests, reconnaissance `CREATE TABLE` | plancher ; reconnaissance en production |
| `crates/kesh-api/tests/admin_full_export_e2e.rs:509-~620` | générateur 0.14.0 | seed partagé, générateur 0.15.0 |
| `crates/kesh-api/tests/admin_full_import_e2e.rs:3608, 3881` | fixture 0.14.0 et son test | fixture 0.15.0, AC6-AC8 |
| `frontend/src/lib/features/admin-restore/` | type, panneau, test | `tablesAbsent`, encart |
| `crates/kesh-i18n/locales/*/messages.ftl` | `admin-restore-files-missing` (fr l. 1769, autres l. 1698) | `admin-restore-tables-absent` |

### Pourquoi pas la version déclarée (C-32-12-3)

`kesh_version` est un texte que le fichier se donne, non signé ; et la 32-5 ajoute sa table sans bump de version Cargo
(migration non breaking, P2-bis ne s'applique pas). Une règle « absente admise si `keshVersion` < version du binaire »
refuserait donc, entre la 32-5 et la release, la sauvegarde 0.15.0 même — le cas que la story existe pour servir. Le
schéma porté par le fichier (quelles tables, nées quand) est la seule preuve disponible. Une preuve plus fine — les
**colonnes** ajoutées depuis à des tables anciennes, qui datent aussi le fichier — est écartée (C-32-12-5) : elle exige
de généraliser `adds_column` (aujourd'hui en `mod tests`) à la production pour un gain nul tant que les stories de
l'epic ne font que créer des tables.

### Ce que l'import fait des tables absentes, et pourquoi c'est sûr

Vider plutôt que garder : un `.keshbackup` restaure l'installation **entière** (manuel, *Opération destructrice*) ;
garder des lignes de la destination dans une table que le fichier ne porte pas mêlerait deux installations — et pour
les clés MCP de la 32-5, garderait vivantes des clés de l'installation remplacée, rattachées à des utilisateurs dont
les identifiants désignent, après l'import, d'autres personnes (le motif de `repair_installation_in_tx`). Vider est le
comportement actuel de `restore_body` : la story le fixe par un test (AC4) au lieu de le changer.

### Contrat pour les stories qui ajoutent une table (à reprendre par la 32-5, la 32-6 et la 32-10a)

1. Inscrire la table à la liste de l'AC2 (iv).
2. La table est **valide vide** : le code lit l'absence de ligne comme l'état par défaut. Sinon, sa migration amorce des
   lignes **et** va au registre (AC11), en classe A gardée (`INSERT … SELECT … WHERE NOT EXISTS`) — la classe B
   exigerait une sentinelle reconnue par `adds_column`, qui ne lit que les `ALTER TABLE … ADD COLUMN`.
3. Reprendre l'AC8 avec la fixture 0.15.0 : `tablesAbsent == [<table>]`, table vide après l'import ; captures de l'état
   « tables absentes » (AC14).
4. Constater `registry_entries_are_within_import_window` vert sur M1 : c'est la preuve que l'AC9 tient (M6 cesse
   d'être équivalente).

### Ce que la story ne fait pas, et ce qui reste non résolu (inventaire)

- **Le retour en arrière** : une sauvegarde 0.16.0 importée par une 0.15.0 reste refusée (table inconnue → 400) ; le
  retour à la 0.15.0 passe par la sauvegarde d'avant la mise à jour, **et** par l'image 0.15.0 (`sqlx` refuse de
  démarrer sur une base qui porte une migration inconnue, `CLAUDE.md` P1). Inchangé, déjà écrit pour la release.
- **Les sauvegardes antérieures au plancher** (0.11.x et avant) : toujours refusées — ce n'est pas un recul, elles
  l'étaient déjà.
- **Une table recréée** par une seconde migration : la règle ne sait pas dater une telle table ; l'AC1 (i) le rend
  bruyant au lieu de deviner.
- **L'authenticité** du fichier : un manifeste édité à la main peut se donner l'inventaire qu'il veut ; la règle protège
  contre une sauvegarde **tronquée par accident** ou d'un ordre impossible, non contre un fichier forgé — qui, de toute
  façon, est importé par un Administrateur et remplace toute l'installation.
- **#722** reste ouverte : la story n'en traite que l'import d'une sauvegarde antérieure.

### Fichiers touchés (prévus)

`crates/kesh-db/src/backup.rs`, `crates/kesh-db/src/post_restore.rs`, `crates/kesh-api/src/admin_backup/import.rs`,
`crates/kesh-api/src/routes/admin.rs`, `crates/kesh-api/tests/admin_full_export_e2e.rs`,
`crates/kesh-api/tests/admin_full_import_e2e.rs`, `crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup`
(neuf), `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
`frontend/src/lib/features/admin-restore/{admin-restore.api.ts,AdminRestorePanel.svelte,AdminRestorePanel.test.ts}`,
`docs/manual/fr/admin-manual.tex` et `.pdf`, `CHANGELOG.md`, `CLAUDE.md`.

**Modules** (règle de découpage préventif, ≤ 5) : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` (admin-restore), docs —
**5**, à la limite. Aucune migration, donc ni P2/P3 (pas de bump), ni P5 (pas de ligne d'audit d'idempotence), ni P6
(aucun `.sql` neuf ; les deux tests positionnels ne sont pas touchés) ; **P7** est l'objet même de l'AC9 et de l'AC11.

### Risques

- **`kesh-db` et l'import** : la base d'une installation réelle passe par ce chemin. Gate complet à chaque passe.
- **Le message d'un test qui ment** : le risque principal de l'epic n'est pas dans cette story mais dans la 32-5, si
  l'AC9 n'est pas fait — `registry_entries_are_within_import_window` y prescrirait de retirer M1 du registre. D'où
  l'AC10 et la demande à l'orchestrateur (*Hors périmètre*).
- **Ordre d'épique** : à développer **avant** la 32-5 (C-32-0-12, C-32-0-13) ; si la 32-5 part d'une `main` sans cette
  story, sa table rend la fixture 0.15.0 non importable et son gate le dit (AC8 rouge) — signal voulu.
- **Fixture** : la générer après un changement de code produirait une sauvegarde qui n'est plus celle de la 0.15.0, et
  le test ne prouverait plus rien. T1 est le premier geste, en commit dédié.

### Hors périmètre — demandes à l'orchestrateur

- Mettre à jour, après merge, la prémisse 4, le texte de la 32-5 (« Risque … sauvegardes 0.15.0 non importables ») et le
  paragraphe *Release v0.16.0* du fichier d'epic, ainsi que le commentaire de `32-5-cles-et-reglages-mcp` dans
  `sprint-status.yaml` : la limite disparaît, le contrat de la § *Contrat* ci-dessus s'y ajoute.

### Références

`CLAUDE.md` § *Migration breaking policy* (P1, P2-bis, P7), § *Exception kesh-db*, § *Recette visuelle* ;
`docs/agents-orchestration.md` § 2.1, § 4, § 5 ; fichier d'epic § *32-12*, § *Ce que le code dit déjà* point 4,
§ *Questions pour Guy* Q4 ; registre `epic-32-choix-autonomes.md` C-32-0-12, C-32-0-13 ; fiche 31-1 (fixture 0.14.0,
AC10, `31-1-sauvegarde-complete.md:335-345, 1325-1335`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-10-24 — Fiche créée (Opus 5.5), `ready-for-dev` **sous réserve de Q4**.

[#722]: https://github.com/guycorbaz/kesh/issues/722
