# Stories 32.12a et 32.12b : une sauvegarde antérieure reste importable (refs #722)

Status: 32-12a review (2026-10-24) ; 32-12b ready-for-dev — toutes deux **sous réserve de Q4** (et de la décision de
découpage, § *Découpage*)

| Clé | Contenu | AC | Ordre |
|---|---|---|---|
| `32-12a-regle-inventaire-et-textes` | fixture 0.15.0, règle d'inventaire, import qui l'applique, fenêtre des tests P7, et **tous** les textes qu'elles rendent faux | AC1-AC7, AC9-AC11, AC15-AC17, AC18a, AC19a | **la première** |
| `32-12b-bilan-ecran-manuel` | réponse et audit de l'import, écran, captures, manuel, CHANGELOG | AC8, AC12-AC14, AC20, AC21, AC18b, AC19b | après la **fusion** de la 32-12a |

**Ordre** : 32-12a, puis 32-12b ; **les deux avant la 32-5** (C-32-0-12, C-32-0-13, C-32-12-37).

> ⚠️ **Stories ajoutées par l'orchestrateur sous réserve du Project Lead** — question **Q4** du fichier d'epic
> (`planning-artifacts/epic-32-serveur-mcp.md` § *Questions pour Guy*), choix **C-32-0-13**. La fiche est écrite
> complète. **Si Guy préfère accepter la limite**, le travail se réduit à **l'écrire** : une entrée du CHANGELOG
> (`## [0.16.0] — Non publié`, *Modifié*) et un paragraphe du manuel d'administration (§ *Importer*, liste des refus),
> qui disent qu'après la mise à jour vers la 0.16.0 une sauvegarde prise en 0.15.0 ou avant — automatiques comprises —
> ne se restaure plus que par une image 0.15.0, et qu'il faut prendre une sauvegarde complète **juste après** la mise à
> jour ; la **32-12a tombe entière**, et la **32-12b** se réduit à ses AC20 (manuel) et AC21 (CHANGELOG), récrits dans
> ce sens (registre : C-32-12-1, C-32-12-37).

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-12`, branche `story/32-12` partie de
`275e9b97` ; **découpée** le 2026-10-24 après la validation P4 (§ *Découpage*). Registre :
`epic-32-choix-autonomes-3212.md` (C-32-12-1 à C-32-12-37 ; 15 à 21 à la validation P1, 22 à 29 à la validation P2, 30 à
36 à la validation P3, 37 au découpage). Clé courte : `3212`. 🗄️ La **32-12a** touche `kesh-db` (`backup.rs`,
`post_restore.rs`, `repositories/backup_schedule.rs`, `version.rs`) : **gate complet même en cours de boucle**
(`CLAUDE.md` § *Exception kesh-db*). La **32-12b** ne touche pas `kesh-db` : gate ciblé entre les passes, gate complet
au dernier commit de code. Aucune migration dans l'une ni l'autre.*

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

- **Aujourd'hui** : `crates/kesh-api/src/admin_backup/import.rs:232-250` — couverture (« chaque table de
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
  1. le premier test **rougirait sur l'entrée M1** (`20261010000001`, seule entrée du registre, `post_restore.rs:226`)
     et son message prescrit « Elle doit passer en exemption » — or une sauvegarde 0.12.x/0.13.0, antérieure à M1, reste
     importable et **a besoin** de ce rejeu. Suivre le message désactiverait le rejeu définitivement ;
  2. le second deviendrait **permissif** : une exemption « Hors fenêtre » d'une migration située entre le plancher et la
     32-5 passerait, alors qu'une sauvegarde 0.15.0 la déclencherait.
- **Donc** : la fenêtre des deux tests devient `IMPORT_FLOOR_MIGRATION` (AC9). Aujourd'hui les deux valeurs sont égales
  et la substitution est **équivalente** — elle ne devient observable qu'à la 32-5 (AC10, mutation M6 déclarée
  équivalente, AC19a).
- **Le registre** : M1 reste dedans ; aucune entrée n'entre ni ne sort. Mais une table neuve postérieure au plancher
  dont la migration **écrit des lignes** (amorçage par société, comme `20260917000001` pour les compteurs) ne peut plus
  invoquer l'argument de couverture : une sauvegarde 0.15.0 arrive sans la table, la table est vidée (point 3), et ses
  lignes d'amorçage manqueraient. Une telle migration **va au registre** (AC11). `missing_sentinels`
  (`post_restore.rs:721`) traite déjà une table absente comme sentinelle manquante (`None => true`) — mais son
  doc-comment (l. 718-720) dit l'état « inatteignable par le flux réel », ce qui devient faux (AC16).

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

- Aujourd'hui : réponse `200` de `full_import` (`crates/kesh-api/src/routes/admin.rs:262-270`) — `backupCreated`,
  `tablesRestored`, `rowsRestored`, `sourceVersion`, `sessionInvalidated`, `filesRestored`, `filesMissing` ; détail de
  l'audit `admin.full_import` (`admin.rs:581-610`). L'écran (`AdminRestorePanel.svelte:240-280`, employé par
  `/admin/restore` **et** `/onboarding/restore`) affiche le succès, les pièces restaurées, les pièces manquantes.
- Ajout : `tablesAbsent: string[]` (toujours présent, vide quand rien ne manque, ordre de création) à la réponse,
  `tables_absent` au détail d'audit, une ligne d'avertissement à l'écran quand la liste n'est pas vide (AC12-AC14).

### Point 5 — la fixture `.keshbackup` 0.15.0

- Précédent exact : `generate_fixture_0_14_0` (`crates/kesh-api/tests/admin_full_export_e2e.rs:534`, doc-comment
  l. 511-533, `#[ignore]`),
  exécuté une fois au tag `v0.14.0` dans un worktree temporaire ; fixture versionnée
  `crates/kesh-api/tests/fixtures/keshbackup-0.14.0-format1.keshbackup` (12 490 octets), lue par `include_bytes!`
  (`admin_full_import_e2e.rs:3608`), jamais régénérée.
- Ici, **pas de worktree temporaire** : la branche porte le code de la v0.15.0 (ci-dessus). La fixture se produit au
  **premier commit de développement, avant toute modification de code** (C-32-12-10), par un générateur jumeau
  `generate_fixture_0_15_0` (`#[ignore]`), même contenu synthétique que celui de la 0.14.0 (société de
  `seed_accounting_company`, « Client Fictif », facture figée, document fournisseur sans QR), via le vrai
  `GET /api/v1/admin/full-export` : format 2, `keshVersion` `0.15.0`, deux pièces sous `files/`. Aucune donnée réelle,
  rien de `~/kesh-test/` (`memory` *Données réelles hors dépôt public*).
- **Repli** (C-32-12-21) : si, au moment du développement, `git diff --stat v0.15.0 HEAD -- crates/` **n'est plus vide**
  (une autre story de l'epic a fusionné du code et la branche a été mise à jour), la fixture **ne se génère pas sur la
  branche** : elle se produit dans un worktree temporaire au tag `v0.15.0`, comme la 0.14.0 (fiche 31-1), place disque
  vérifiée d'abord et worktree retiré aussitôt après (`memory` *Disque et worktrees*). Jamais de génération avec un
  diff non vide.

## Critères d'acceptation

**Répartition** (C-32-12-37) : **32-12a** — groupes A, C, E, et AC5-AC7 (B), AC15-AC17 (F), AC18a, AC19a ;
**32-12b** — groupe D, et AC8 (B), AC20-AC21 (F), AC18b, AC19b. Chaque AC porte sa clé en tête. Un AC d'une clé qui en
cite un de l'autre le dit (« AC8, 32-12b ») : la citation désigne un critère tenu par l'autre story, non une tâche de
celle-ci.

### A — La règle d'inventaire (`kesh-db`) — 32-12a

1. **AC1 [32-12a] — table → migration créatrice, dérivée du `MIGRATOR`.** `kesh_db::backup::table_creating_migrations()` rend,
   pour chaque table de `TABLES_TO_TRUNCATE`, la version de la migration qui la crée — calculée une fois
   (`std::sync::OnceLock`) depuis `crate::MIGRATOR`, par la reconnaissance de `CREATE TABLE` aujourd'hui confinée aux
   tests (`creates_application_table`, `normalize`), **déplacée en production** dans `post_restore.rs`, à côté de
   `split_statements` (un seul exemplaire, DRY) : `creates_application_table(statement: &str) -> bool`
   (`post_restore.rs:978`, qui calcule le nom puis le jette) y devient **`created_application_table(statement: &str) ->
   Option<&'static str>`**, qui rend l'entrée de `TABLES_TO_TRUNCATE` créée par le statement (le nom en majuscules
   comparé sans casse, l'entrée de l'inventaire rendue) ; `normalize` devient `pub(crate)`, et `split_statements`
   (`post_restore.rs:772`, aujourd'hui privée) passe `pub(crate)` pour que `backup` l'appelle (C-32-12-23, C-32-12-31).
   Deux accesseurs dans `backup.rs` : **`table_creators() -> &'static BTreeMap<&'static str, Vec<i64>>`**, **toutes**
   les migrations créatrices de chaque table (liste vide pour une table sans créatrice connue), et
   **`table_creating_migrations()`**, la carte d'une version par table, qui prend le **minimum** de chaque liste non
   vide — sur cette seule carte, « deux créatrices » serait indiscernable de « une ». La fonction de test `last_table_creating_migration`
   (`post_restore.rs:958`) est **retirée** : après l'AC9 elle n'a plus d'appelant (l'AC2 (ii) lit le maximum de la carte),
   et `clippy -D warnings` refuserait une fonction morte. Tests (`backup.rs`, `mod tests`) : (i) **sur `table_creators()`**,
   **chaque** table de l'inventaire a exactement **une** migration créatrice — zéro ou deux font échouer le test en
   nommant la table (une table recréée par une seconde migration est un cas que la règle ne sait pas traiter : il doit
   se voir) ; (ii) la carte rend
   `20260917000001` pour `journal_entry_number_sequences`, `20260827000001` pour `invoice_settlements`,
   `20260428000001` pour `vat_rates`. En production, une table sans migration connue est **exigée** (repli sûr : refus
   plutôt que tolérance) ; une table à **deux** migrations créatrices (inatteignable aujourd'hui : chacune des 39 n'a
   qu'un `CREATE TABLE`, aucun `RENAME`/`DROP TABLE`/`CREATE … LIKE` au dépôt) prend la **plus ancienne** — la table
   paraît vieille, donc exigée : refus plutôt que tolérance —, et le test (i) reste ce qui rend le cas bruyant
   (C-32-12-18). Le doc-comment de la reconnaissance déplacée dit pourquoi `split_statements` (`post_restore.rs:772`),
   dont le propre doc-comment (l. 752-771) liste les dangers d'un analyseur naïf (`/* */`, `--` ou `;` dans un
   littéral), y suffit : seul l'**en-tête** du statement est lu (`CREATE TABLE <nom>`) ; une graphie ratée rend zéro
   migration créatrice et fait échouer (i) bruyamment ; le repli de production est « exigée » ; les blocs `/* */` sont
   tenus pour **toutes** les migrations par `migrations_contain_no_block_comment` (l. 1389), à citer. Il reprend, récrit,
   ce que disait le doc-comment de `last_table_creating_migration` (l. 942-957) : le filtre sur l'inventaire applicatif
   (une table **système** n'a pas de migration créatrice dans la carte) et la reconnaissance **large** (blancs normalisés,
   accents graves retirés) — mais non plus « une graphie ratée produit une fenêtre qui **recule** » : depuis l'AC9 la
   fenêtre est le plancher, et une graphie ratée rend zéro créatrice, ce qui fait rougir (i).
2. **AC2 [32-12a] — le plancher.** `pub const IMPORT_FLOOR_MIGRATION: i64 = 20260917000001;` dans `backup.rs`, avec un
   doc-comment qui dit : ce qu'il est (la fenêtre d'importabilité, gelée à la v0.15.0) ; que le **relever** referme la
   fenêtre et rend non importables les sauvegardes antérieures — décision du Project Lead, CHANGELOG à l'appui, jamais
   un geste de maintenance ; et le **contrat** des tables postérieures (valides vides, ou amorcées par le registre —
   point 3) ; et que les commentaires de **quatre** migrations publiées, antérieures à la 32-12, décrivent l'ancienne
   règle d'égalité et restent tels quels (P8) — la phrase les **nomme une à une** (`20261001000004`, `20261003000001`,
   `20261009000001`, `20261024000001`, AC16), pour qu'un oubli ne la rende pas fausse. Les tests (`backup.rs`, `mod
   tests`) emploient **deux objets distincts**, nommés tels dans le code (C-32-12-28) : `TOLEREES_ECRITES`, liste
   écrite en dur dans le test (**vide** aujourd'hui), et `tolerees_calculees`, l'ensemble calculé des tables de
   l'inventaire dont la migration créatrice est **postérieure** au plancher ; `exigees` est l'ensemble calculé des
   tables créées au plus tard par le plancher **ou sans migration connue** (repli de l'AC1). Tests : (i) le plancher est
   une migration du `MIGRATOR` qui crée une table de l'inventaire ; (ii) `IMPORT_FLOOR_MIGRATION <=` le maximum des
   valeurs de `table_creating_migrations()` (et non `last_table_creating_migration`, privée de `post_restore::tests` et
   retirée, AC1) ; (iii) **`exigees ∪ TOLEREES_ECRITES == TABLES_TO_TRUNCATE`** et **`exigees ∩ TOLEREES_ECRITES ==
   ∅`** — sur la liste **écrite**, ce qui n'est pas une tautologie : une table de l'inventaire créée après le plancher
   mais absente de la liste écrite n'est dans aucun des deux ensembles, et (iii) rougit ; forme stable, vraie aujourd'hui
   (liste vide : `exigees` = les 39 tables, ce qui prouve que le plancher couvre tout l'inventaire de la 0.15.0) et qui
   reste vraie à la 32-5 sans réécriture, une fois sa table inscrite à `TOLEREES_ECRITES` (C-32-12-16) ; (iv)
   `assert_eq!(tolerees_calculees, TOLEREES_ECRITES)` — chaque story qui ajoute une table l'inscrit à la liste écrite,
   et ce geste force à relire le contrat (patron du compteur `checked` de `post_restore.rs`). ⚠️ **(iii) est impliquée
   par (iv)** : `exigees` et `tolerees_calculees` partitionnent l'inventaire par construction, si bien que (iv) vraie
   entraîne (iii) vraie — (iii) ne rougit jamais **seule**. Elle reste pour son message (elle nomme la table qui
   n'est dans aucun des deux ensembles), non comme un second garde : une mutation qu'elle tue, (iv) la tue aussi, et
   l'AC19a ne les compte pas comme deux tueurs (C-32-12-33). Le mot « tolérées » sans
   qualificatif est réservé, dans cette fiche, aux tables **absentes du fichier** que l'AC3 laisse passer à l'exécution.
3. **AC3 [32-12a] — la vérification pure.** `kesh_db::backup::check_backup_inventory(present: &BTreeSet<&str>, regles:
   &InventoryRules) -> Result<Vec<&'static str>, InventoryError>` — les tables absentes tolérées, dans l'ordre de
   création, ou l'erreur. **Ordre défini** : par version de la migration créatrice, puis, pour deux tables nées dans la
   même migration, par leur rang dans l'inventaire (`TABLES_TO_TRUNCATE` en production) — `tablesAbsent` à l'écran et à
   l'audit ne dépend donc pas d'un tri laissé au hasard (C-32-12-34). `InventoryRules { inventaire: &'static [&'static
   str], creation: fn(&str) -> Option<i64>, plancher }` (ou équivalent),
   `InventoryRules::production()` construit depuis `TABLES_TO_TRUNCATE`, `table_creating_migrations()` et
   `IMPORT_FLOOR_MIGRATION` ; les tests injectent les leurs. Trois erreurs, chacune nommant la table :
   - `Inconnue(String)` — table présente au manifeste, hors inventaire (règle actuelle ; le nom vient du fichier, d'où
     un `String`) ;
   - `Exigee(&'static str)` — table absente et créée au plus tard par le plancher (ou sans migration connue) ;
   - `HorsOrdre { absente: &'static str, presente: &'static str }` — `absente` postérieure au plancher mais antérieure
     (ou égale) à la création d'une table `presente`. `Inconnue` est vérifiée **d'abord** : toute table présente qui
     passe appartient à l'inventaire, et `presente` est l'entrée de l'inventaire, d'où la durée de vie `'static`.
   Une table **présente sans migration connue** (`creation` rend `None` : inatteignable en production par l'AC1 (i),
   possible avec des règles injectées) ne contraint pas la règle du suffixe : elle compte comme créée au plus tard
   par le plancher, comme le repli « exigée » de l'AC1 la traiterait absente — aucune absente tolérée (postérieure au
   plancher) ne peut lui être antérieure.
   Tests unitaires (inventaire et carte injectés, sans base) : inventaire complet → `Ok(vec![])` ; une table postérieure
   absente → `Ok([t])` ; deux postérieures absentes → `Ok([t1, t2])` dans l'ordre de création ; deux postérieures nées
   dans la **même** migration, toutes deux absentes → `Ok` dans l'ordre de l'inventaire ; une table présente sans
   migration connue et une postérieure absente → `Ok([t])` ; la plus ancienne des deux
   absente, la plus récente présente → `HorsOrdre` ; une table du plancher absente → `Exigee` ; une table inconnue →
   `Inconnue` ; deux tables nées dans **la même** migration, une seule absente → `HorsOrdre` (même version : le
   fichier ne peut pas porter l'une sans l'autre) ; inventaire de production (`InventoryRules::production()`) et
   manifeste complet → `Ok(vec![])`.
4. **AC4 [32-12a] — une table absente est vidée, même si la destination avait des lignes.** Test `kesh-db`
   (`#[sqlx::test(migrations = "./test-schema")]` — **cette graphie exactement**, celle des 837 attributs de `kesh-db`
   et de `backup.rs:1150` : la garde `crates/kesh-db/tests/test_schema_guard.rs::every_sqlx_test_attribute_is_accounted_for`
   (l. 921) n'accepte que les deux graphies de `SQUASH_SPELLINGS` (l. 64, `"./test-schema"` et
   `"../kesh-db/test-schema"`) et rougit sur `"test-schema"`, que `sqlx` résoudrait pourtant ; `backup.rs` ou
   `crates/kesh-db/tests/`) : la destination porte un
   produit (`products`) ; `restore_tables_in_tx` reçoit une carte qui omet `products` et porte `companies` ; après,
   `products` est vide et `tables_restored` ne la compte pas. Le comportement existe (`backup.rs:666-675`) : le test le
   **fixe**, et le commentaire « Dans le flux réel, `parse_and_verify` garantit la couverture » est récrit (la
   couverture n'est plus complète : une table postérieure au plancher peut manquer, et elle est vidée **à dessein**).

### B — L'import (`kesh-api`) — AC5-AC7 32-12a, AC8 32-12b

5. **AC5 [32-12a] — `parse_and_verify` applique la règle.** Les deux boucles de `import.rs:232-250` sont remplacées par un appel
   à `check_backup_inventory` avec `InventoryRules::production()`, par une porte `pub(crate) fn
   parse_and_verify_with(reader, limits, regles)` que `parse_and_verify` enveloppe. Le module `admin_backup` est
   `pub(crate)` (`crates/kesh-api/src/lib.rs:10`, commentaire de sécurité de la revue 17-3a, l. 7-8 : **ne pas le
   toucher**) : la porte n'est visible que de la crate, et c'est voulu — son seul appelant de test est l'AC7, écrit dans
   la crate (C-32-12-15).
   Messages `InvalidBackupStructure` (→ 400) : `Exigee` garde **mot pour mot** l'actuel `table '{t}' absente du
   manifeste` (les tests existants qui le cherchent restent verts sans changement — `parse_rejects_missing_table_coverage`,
   `import.rs:899`) ; `Inconnue` garde l'actuel « présente dans le manifeste mais hors inventaire applicatif » ;
   `HorsOrdre` : `table '{absente}' absente du manifeste alors que '{presente}', créée par la même migration qu'elle
   ou après, y figure` — exact dans les deux cas que couvre l'erreur (« antérieure ou égale », AC3), y compris celui de
   deux tables nées dans la même migration (C-32-12-24). Les deux messages commencent par le même fragment `table '…'
   absente du manifeste` : un test qui ne cherche que ce fragment (`parse_rejects_missing_table_coverage`,
   `m.contains(…)`) ne les distingue pas, d'où l'égalité **exacte** exigée à l'AC7 (b).
   `ParsedBackup` porte `tables_absent: Vec<String>`. ⚠️ **Découpage** : à la 32-12a, aucun code de production ne lit
   encore ce champ (son lecteur, `run_backup_and_restore`, est l'AC12, 32-12b) — `clippy -D warnings` signalerait un
   champ jamais lu, l'AC7 ne le lisant qu'en `cfg(test)`. Il est lu par l'implémentation **manuelle** de `Debug` de
   `ParsedBackup` (`import.rs:97-106`), qui gagne `.field("tables_absent", &self.tables_absent)` — une lecture réelle,
   qu'un `Debug` dérivé ne serait pas (`dead_code` ignore les implémentations dérivées, non les manuelles) ; à constater
   au premier `clippy`. Si le lint parlait malgré tout, repli sur le précédent de `backup.rs:434`
   (`#[cfg_attr(not(test), allow(dead_code))]`), commentaire nommant la 32-12b, qui le retire.
6. **AC6 [32-12a] — les refus restent des refus, avant tout effet.** Tests HTTP (`admin_full_import_e2e.rs`) sur la fixture
   0.15.0 récrite par `ouvrir_tout`/`rezip_tout` (`admin_full_import_e2e.rs:3620` et suivantes) : (i) `products` retirée
   du manifeste et de l'archive → `400`, `error.code == "INVALID_BACKUP_STRUCTURE"` ; (ii) une table inconnue ajoutée →
   même refus. Le **motif** du refus ne traverse pas la réponse HTTP : `errors.rs:2171-2180` le journalise
   (`tracing::warn!`) et répond le libellé générique `error-invalid-backup-structure` (« Le fichier de sauvegarde est
   invalide ou corrompu. ») — un test HTTP ne peut donc pas asserter `absente du manifeste` ; le message exact se prouve
   hors HTTP, à l'AC7 (b) (C-32-12-24). Dans les deux cas, la base de destination est **intacte** (une ligne témoin relue) et
   **aucune** sauvegarde pré-import n'a été écrite (`KESH_ADMIN_BACKUP_DIR` vide).
7. **AC7 [32-12a] — la chaîne d'import tolère une table postérieure, bout à bout hors HTTP.** Test **dans la crate** —
   `crates/kesh-api/src/admin_backup/import.rs`, `mod tests`, `#[sqlx::test(migrations = "../kesh-db/test-schema")]`
   (précédents en `src/` : `auth/bootstrap.rs:482` et suivants) —, parce que `admin_backup` est `pub(crate)` et qu'un
   test de `tests/` (crate externe) ne voit ni `parse_and_verify_with`, ni `check_schema_compat`, ni `ParsedBackup` ;
   **aucune ré-exportation** (C-32-12-15). La fixture se lit par
   `include_bytes!("../../tests/fixtures/keshbackup-0.15.0-format2.keshbackup")` ; `products` s'en retire par une petite
   fonction locale au `mod tests` (ouvrir le zip, omettre `data/products.ndjson`, retirer l'entrée `tables.products`
   du manifeste, rezipper) — les helpers `ouvrir_tout`/`rezip_tout` de `tests/admin_full_import_e2e.rs` ne sont pas
   accessibles depuis `src/`. Le test **simule** une table postérieure au plancher, faute d'en avoir une avant la
   32-5 : règles injectées où `products` est créée après le plancher **et après toute autre table de l'inventaire**
   (version supérieure à la plus récente des créatrices, au-delà de `20260917000001`) — « après le plancher » seul ne
   suffit pas : si une table présente était créée après elle, la règle du suffixe rendrait `HorsOrdre` (AC3) ; fixture 0.15.0 privée de `products` (vide dans
   le seed, aucune ligne de facture ne la référence — à constater sur la fixture produite) ; destination qui porte un
   produit. `parse_and_verify_with` → `Ok`,
   `tables_absent == ["products"]` ; `check_schema_compat` → `Ok` ; `restore_tables_in_tx` → `products` **vide**, les
   autres tables restaurées (une facture de la fixture relue) ; `replay_post_restore_backfills` → `Ok`. Ce test
   n'emprunte pas le handler : la recopie de `tables_absent` dans la réponse et l'audit est l'affaire de la **32-12b**,
   prouvée par son AC8 (liste vide) et, avec une liste non vide, à la 32-5 (AC10) — d'ici là, une valeur codée en dur à
   `[]` survit (mutation M9, AC19b, report assumé).
   **(b) Le message exact du refus**, même `mod tests`, simple `#[test]` (sans base) : la même fixture privée de
   `products`, passée à `parse_and_verify_with` avec `InventoryRules::production()` (où `products`, créée par
   `20260415000001`, est exigée) → `Err(AppError::InvalidBackupStructure(m))` avec **`m == "table 'products' absente du
   manifeste"`** (égalité, non `contains`). C'est ce qui rend le message de `Exigee` observable — l'AC6 ne le peut pas —
   et ce qui distingue `Exigee` de `HorsOrdre` : sous M1, `products` tombe en `HorsOrdre` (des tables créées après elle
   sont présentes), dont le message commence par le même fragment et finit par `… y figure` (AC5, AC19a).
8. **AC8 [32-12b] — la fixture 0.15.0 s'importe, et le bilan le dit.** Emploie la fixture produite par la 32-12a
   (AC15) et le champ `ParsedBackup::tables_absent` qu'elle pose (AC5). `ac_la_sauvegarde_de_la_0_15_0_s_importe` (patron de
   `ac10_la_sauvegarde_de_la_0_14_0_s_importe`, `admin_full_import_e2e.rs:3881`) : `200`, `sourceVersion` `"0.15.0"`,
   `tablesAbsent` **présent et vide** (`[]`, non absent), `filesRestored` `2`, `filesMissing` `0` ; détail d'audit
   `tables_absent: []`, `source_format_version: 2`. Le test de la 0.14.0 gagne l'assertion `tablesAbsent == []`
   (même inventaire). Une fixture générée par un binaire ultérieur, qui porterait des tables que celui-ci n'a pas,
   tombe sous `Inconnue` : c'est le symétrique, inchangé.

### C — Le rejeu des backfills (`post_restore.rs`) — 32-12a

9. **AC9 [32-12a] — la fenêtre des tests est le plancher.** `registry_entries_are_within_import_window` et
   `exemptions_claiming_out_of_window_really_are_out_of_window` comparent à `crate::backup::IMPORT_FLOOR_MIGRATION`, et
   non plus à `last_table_creating_migration()` (retirée, AC1). Sont récrits, au même commit — *la fenêtre est le
   plancher, constante écrite et non plus calculée ; une table neuve ne la referme plus* :
   - le doc-module, § *La fenêtre d'importabilité* (`post_restore.rs:19-37` : « égalité **exacte, dans les deux sens** »
     l. 22, « dernière migration créatrice » l. 25-26, « referme la fenêtre » l. 35) ;
   - le commentaire de l'entrée M1 (l. 250-251, « aucune table applicative n'est créée depuis `20260917000001` » —
     faux dès la 32-5 : M1 est **postérieure au plancher**, c'est ce qui la garde dans la fenêtre) ;
   - le bloc l. 257-279, pour ses seules phrases au présent (l. 269-270 « recalcule depuis le `MIGRATOR` », l. 272-274
     « Toute nouvelle table applicative aura le même effet ») — ses relevés datés des Stories 24-2/24-3 (l. 257-266 et
     276-279, « Et ce cas s'est représenté à la story SUIVANTE ») restent ;
   - le doc-comment d'`EXEMPT_MIGRATIONS` (l. 375-381, « recalcule la fenêtre depuis le `MIGRATOR` », l. 379) ;
   - le doc-comment de `registry_entries_are_within_import_window` (l. 916-927 : « inventaire de tables est
     identique » l. 918, « dernière migration créatrice » l. 920, « **recalculée depuis le `MIGRATOR`**, jamais codée en
     dur : toute future migration créant une table applicative la referme » l. 925-927) et son message (l. 934) ;
   - les deux appels `let window = last_table_creating_migration();` (l. 930 et 1084), remplacés par le plancher ;
   - le message de `exemptions_claiming_out_of_window_really_are_out_of_window` (l. 1095-1096, « POSTÉRIEURE à la
     dernière migration créatrice … donc IMPORTABLE ») ;
   - le doc-comment de `last_table_creating_migration` (l. 942-957 : « ne déplace pas la fenêtre » l. 947, « une fenêtre
     qui **recule** » l. 952) part avec la fonction ; ce qui en reste vrai passe au doc-comment de la reconnaissance
     déplacée (AC1).
   Le compteur `checked` reste à **8**. Le commentaire du compteur (l. 1105-1121, relevés datés 24-2 et 25-2-c, dont la
   citation « *toute nouvelle table applicative aura le même effet* » rapportée **comme ce que le module annonçait**)
   reste : il dit vrai à sa date.
10. **AC10 [32-12a] — contrat écrit pour la 32-5.** (Les AC8 et AC14 qu'il cite sont tenus par la 32-12b ; la 32-5
    ne part qu'une fois **les deux** fusionnées.) Le doc-comment d'`IMPORT_FLOOR_MIGRATION` et le Dev Notes de cette fiche
    nomment ce que la 32-5 (et toute story qui ajoute une table) doit faire : inscrire la table à `TOLEREES_ECRITES`
    (AC2) ; récrire la justification de l'exemption `20260917000001` (`post_restore.rs:441`, *Contrat* point 5) ; constater que `registry_entries_are_within_import_window` reste **vert** sur M1 (c'est la preuve que l'AC9
    tient) ; reprendre l'AC8 pour la fixture 0.15.0 avec `tablesAbsent == [<sa table>]` et la table vide après
    l'import ; prendre les captures de l'état « tables absentes » (AC14).
11. **AC11 [32-12a] — une table postérieure amorcée va au registre.** Test `post_restore.rs` :
    `post_floor_table_seeds_are_replayed` — pour toute migration de version `> IMPORT_FLOOR_MIGRATION` qui porte un
    statement d'écriture (premier mot-clé `INSERT`/`REPLACE`/`UPDATE`/`DELETE` et nom de table qui suit
    `INTO`/`UPDATE`/`FROM`) dont la cible est une table de l'inventaire **créée après le plancher** — par cette migration
    **ou par une migration antérieure** (`table_creating_migrations()` donne la date de la cible) —, la migration figure
    à `POST_RESTORE_BACKFILLS`, pas à `EXEMPT_MIGRATIONS`. Le second cas est celui d'une story qui amorce ou corrige
    des lignes d'une table créée par une story précédente (la 32-6 écrivant dans une table de la 32-5) : une
    sauvegarde 0.15.0 arrive sans la table, la table est vidée (point 3), et l'argument « couverture » d'une exemption
    serait **faux** — que `every_data_backfill_migration_is_triaged`, qui accepte toute justification écrite, ne verrait
    pas (C-32-12-32). Vacant aujourd'hui (aucune table postérieure au plancher) : sa matière vient de l'AC19a (M7) et, à
    la 32-5, du réel. Un `UPDATE _kesh_version` dans la même migration n'est pas une écriture dans une table de
    l'inventaire et ne déclenche rien.
    ⚠️ **Une migration qui crée sa table et l'amorce ne s'inscrit pas au registre par un `include_str!` du fichier
    entier** : `registry_sql_contains_no_ddl` (`post_restore.rs:1355`) refuse tout `CREATE TABLE` dans le SQL rejoué
    (« une migration mixte DDL + données exige un EXTRAIT de ses seuls statements de données »). L'entrée porte un
    **extrait** de ses seuls statements d'écriture, sous `crates/kesh-db/src/post_restore/`, et
    `extract_carries_every_write_statement_of_its_source_migration` (l. 1307) en contrôle la cardinalité. Le *Contrat*,
    point 2, le reprend.

### D — Le bilan — 32-12b

12. **AC12 [32-12b] — réponse et audit, chacun dans sa fonction.** `parsed` est déplacé dans `run_backup_and_restore`
    (`admin.rs:260`), qui le reçoit **par valeur** (l. 343-346) et l'a encore en main quand il construit le détail
    d'audit (l. 581) : c'est **là** que s'ajoute `"tables_absent": parsed.tables_absent` (`run_backup_and_restore`). La
    **réponse**, elle, est construite dans `full_import` (l. 262-270) **après** le déplacement, depuis l'`ImportOutcome`
    rendu (`issue`) : `ImportOutcome` (l. 332-338) gagne `tables_absent: Vec<String>`, rempli par
    `run_backup_and_restore`, et `full_import` écrit `"tablesAbsent": issue.tables_absent`. ⚠️ Le test lexical
    `routes/admin.rs::tests::echecs_avant_sauvegarde_tous_convertis` (l. 1131) compte les `?` entre le début de
    `run_backup_and_restore` et l'appel `write_pre_import_backup(` : n'en **ajouter aucun** dans ce segment (le
    relevé de `tables_absent` n'en demande pas). `FullImportResponse`
    (`frontend/src/lib/features/admin-restore/admin-restore.api.ts:11`) gagne `tablesAbsent?: string[]`, optionnel
    (absent d'un serveur 0.15.0), doc-comment à la manière de `filesMissing`.
13. **AC13 [32-12b] — l'écran.** `AdminRestorePanel.svelte`, sous le bloc des pièces manquantes : quand `tablesAbsent` est non
    vide, un encart `role="status"` d'avertissement (mêmes classes que `admin-restore-files-missing`),
    `data-testid="admin-restore-tables-absent"`, clé `admin-restore-tables-absent` :
    « Données nées après la version de la sauvegarde, laissées vides : { $tables }. Ressaisissez-les depuis
    l'application si nécessaire. » (`$tables` : les noms joints par « , ») — le texte ne promet pas qu'elles « se
    recréent » : le contrat (AC2, AC10) n'exige d'une table postérieure que d'être valide vide ou amorcée par le
    registre (C-32-12-19). Rien quand la liste est vide ou absente. Clé dans les
    **quatre** catalogues (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, à côté de
    `admin-restore-files-missing`, l. 1769 / 1698). Vitest (`AdminRestorePanel.test.ts`) : réponse simulée avec
    `tablesAbsent: ['mcp_keys']` → l'encart et le nom ; `[]` → pas d'encart ; champ absent → pas d'encart.
14. **AC14 [32-12b] — recette visuelle.** Captures, thème clair, de `/admin/restore` après l'import de la fixture 0.15.0 :
    l'état nominal (pas d'encart). L'état « tables absentes » ne peut pas être produit **pour de vrai** avant qu'une
    table postérieure existe : sa capture est **transférée à la 32-5** (AC10), et la fiche le dit au Dev Agent Record —
    ne pas la fabriquer en simulant une réponse dans le navigateur (C-32-12-11). C'est une **dérogation déclarée** à
    `CLAUDE.md` § *Recette visuelle* ; le Dev Agent Record écrit le **risque résiduel** : l'encart pourrait être
    invisible à l'écran (précédent #640, bouton testé mais invisible faute de jeton de couleur), risque atténué par
    l'identité des classes avec `admin-restore-files-missing` (`border-warning bg-warning/10`, dont la visibilité est
    établie) et levé par la capture de la 32-5.

### E — La fixture — 32-12a

15. **AC15 [32-12a] — produite par le vrai exporteur de la 0.15.0, avant tout changement de code.** Premier commit de
    développement : `generate_fixture_0_15_0` (`#[ignore = "générateur de fixture : à n'exécuter qu'avec le code de la
    v0.15.0 (Story 32-12a, AC15)"]`, `admin_full_export_e2e.rs`), dont le seed est **partagé** avec
    `generate_fixture_0_14_0` (fonction extraite, DRY ; le générateur 0.14.0 garde son doc-comment et son `#[ignore]`) ;
    exécuté une fois, **après** avoir constaté `git diff --stat v0.15.0 HEAD -- crates/` vide (sinon : repli du
    point 5, worktree temporaire au tag, C-32-12-21) ; sortie
    `crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup`. Contrôle par `unzip -l` / `unzip -p … manifest.json` :
    `formatVersion: 2`, `keshVersion: "0.15.0"`, deux entrées `files/<sha>.pdf`, 39 tables. Taille et SHA-256 au Dev
    Agent Record ; le doc-comment du générateur dit pourquoi il est `#[ignore]` et ne doit jamais être relancé avec un
    code ultérieur (il produirait une sauvegarde qui n'est plus celle de la 0.15.0).

### F — Documentation et gates — AC16-AC17 32-12a, AC20-AC21 32-12b, AC18-AC19 dans chacune

16. **AC16 [32-12a] — les textes qui disent « identique ».** Récrits au même commit : doc-comment de `TABLES_TO_TRUNCATE`
    (`backup.rs:34-38`, « y ajouter une table rendrait non importable toute sauvegarde antérieure ») ; commentaire de
    `restore_body` (`backup.rs:647`, « L'en sortir rendrait tout backup existant inimportable » — vrai pour un
    **retrait**, à préciser) ; commentaire de `restore_body` l. 667-668 (« Dans le flux réel, `parse_and_verify`
    garantit la couverture », AC4) ; doc-module de `post_restore.rs` (AC9) ; doc-comment de `missing_sentinels`
    (`post_restore.rs:718-720`, « défensif seulement … inatteignable par le flux réel » : l'état devient atteignable,
    c'est le cas d'une table postérieure absente) ; en-tête de `import.rs` (l. 14, « couverture des tables ») et
    commentaire l. 232-233 (« chaque table applicative attendue doit figurer au manifeste ») ; message d'assertion de
    `table_systeme_hors_inventaire` (`crates/kesh-db/src/repositories/backup_schedule.rs:502`, « l'ajouter à
    l'inventaire rendrait toute sauvegarde antérieure non importable » — la raison qui reste vraie : une table système
    appartient à l'installation, non aux livres, et une table applicative de plus que la sauvegarde ne porte pas serait
    vidée à l'import) ; doc-comment de `check_import_version_compat` (`crates/kesh-db/src/version.rs:272-275`, fonction l. 281, « une
    source d'une version mineure plus récente mais non-breaking reste importable ») — **faux avant comme après** la
    story (une 0.16.0 qui ajoute une table n'est pas importable en 0.15.0 : `Inconnue`) ; récrit en « ce contrôle-ci ne
    refuse pas pour cette seule raison ; l'inventaire des tables et la compatibilité des colonnes peuvent refuser
    ailleurs » (C-32-12-26) ; dans `CLAUDE.md` § *Migration breaking policy* P7, **deux** paragraphes :
    - « Trier, ce n'est pas forcément inscrire au registre » (l. 827) : « inventaire **identique** au nôtre
      (`parse_and_verify` compare dans les deux sens) » → inventaire au moins égal à celui du plancher, absentes
      postérieures à toutes les présentes ; « antérieure à la dernière création de table applicative » → antérieure
      au plancher `IMPORT_FLOOR_MIGRATION` ; « Le test `registry_entries_are_within_import_window` **recalcule cette
      fenêtre depuis le `MIGRATOR`** » → compare au plancher, dont un test (AC2 (i)) vérifie qu'il est une migration
      créatrice du `MIGRATOR` ;
    - « L'exemption est l'issue la moins coûteuse… » (l. 829) : « le symétrique
      `exemptions_claiming_out_of_window_really_are_out_of_window`, qui **recalcule la fenêtre** pour toute
      justification invoquant l'argument » → qui compare au plancher toute justification invoquant l'argument ;
    — modification du `CLAUDE.md` soumise à Guy par la PR.
    ⛔ **Les commentaires des migrations ne se corrigent pas (P8)**, même devenus faux. **Quatre** migrations publiées
    décrivent l'ancienne règle d'égalité (inventaire par `grep -rniE "importab" crates/kesh-db/migrations`, AC17) :
    - `20261001000004_invoice_settlements_write_off.sql:14-15` (« une table neuve rendrait toutes les sauvegardes
      antérieures inimportables (l'inventaire des tables d'un backup doit être identique) ») ;
    - `20261003000001_invoices_frozen_pdf.sql:9-11` (« une table neuve rendrait toutes les sauvegardes existantes non
      importables (`post_restore.rs`, inventaire des tables) ») ;
    - `20261009000001_journal_entry_lines_lettering.sql:10-12` (« une table neuve rendrait toutes les sauvegardes
      antérieures inimportables (l'inventaire des tables d'un backup doit être identique) ») ;
    - `20261024000001_backup_schedule.sql:5-7` (« Une table applicative neuve ferait refuser à l'import TOUTE
      sauvegarde antérieure (le contrôle de couverture exige l'égalité des inventaires de tables) »).
    Toutes sont publiées (premier tag : v0.12.1, v0.12.1, v0.13.0, v0.15.0) ; en changer un octet fait refuser le démarrage à toute base qui les porte.
    La rectification va **ailleurs** : la phrase du doc-comment d'`IMPORT_FLOOR_MIGRATION` qui les **nomme une à une**
    (AC2) et les présentes Dev Notes. `20260915000001_audit_log_company_id.sql:23-24`, aussi rendue par le grep, parle
    d'une **colonne** `NOT NULL` sans défaut : elle reste vraie, hors de la classe. La ligne de `20261024000001` à
    `docs/migrations-idempotence-audit.md` (l. 105 : « elle ne déplace pas la fenêtre d'importabilité ») reste vraie et
    n'est pas touchée ; les lignes 85, 91 et 98 du même fichier sont des relevés datés (« sa création de table
    REFERME la fenêtre », « pour que les sauvegardes antérieures restent importables ») qui disent vrai à leur date
    (C-32-12-17, C-32-12-22).
17. **AC17 [32-12a ; relancé par la 32-12b] — la classe inventoriée par son symptôme, valeur comprise.** La classe : *toute phrase du dépôt qui affirme
    l'égalité des inventaires, la non-importabilité d'une sauvegarde antérieure, une fenêtre « recalculée depuis le
    `MIGRATOR` » / « jamais codée en dur » / qui « se referme » à chaque table, ou que `20260917000001` est « la dernière
    créatrice »*. Un seul motif s'est montré deux fois insuffisant (validations P1 et P2 : il manquait des phrases qui ne
    portaient aucun de ses mots) ; elle s'inventorie donc par **cinq familles**, chacune son `grep`, **toutes
    insensibles à la casse (`-i`)** — une graphie capitalisée (« REFERME la fenêtre », `docs/migrations-idempotence-audit.md:91`,
    membre de la classe) échappe sinon à la commande —, sur la portée
    `S="crates/kesh-db/src crates/kesh-db/migrations crates/kesh-db/tests crates/kesh-api/src crates/kesh-api/tests
    CLAUDE.md docs README.md website frontend/src scripts .config"` (`--exclude='*.pdf' --exclude='*.keshbackup'`),
    depuis la racine du worktree, après chaque patch (C-32-12-22, C-32-12-30). `scripts/` et
    `.config/` y sont parce que `scripts/prepare-release.sh:128` parle de la fenêtre d'importabilité. Chaque commande
    se lance **telle qu'écrite**, suivie de `| wc -l` pour le nombre, sans fichier intermédiaire :
    - **G1 — importabilité** : `grep -rniE "importab" --exclude='*.pdf' --exclude='*.keshbackup' $S` (**51** lignes) ;
    - **G2 — égalité** : `grep -rniE "inventaire[^.]{0,40}identique|\*\*identique\*\* au nôtre|doit être identique|égalité (\*\*exacte|des inventaires)|compare dans les deux sens|exacte, dans les deux sens" --exclude='*.pdf' --exclude='*.keshbackup' $S` (**7**) ;
    - **G3 — fenêtre calculée** : `grep -rniE "recalcul[a-zé]* (la |cette )?fenêtre|recalculée depuis le|codée en dur|ne déplace pas la fenêtre|fenêtre qui \*\*recule|referm[a-z]* (\*\*)?(la|cette) fenêtre|refermé la fenêtre|fenêtre refermée|last_table_creating_migration" --exclude='*.pdf' --exclude='*.keshbackup' $S` (**36**) ;
    - **G4 — dernière créatrice** : `grep -rniE "dernière (migration )?créatrice|dernière création de table|EST la dernière" --exclude='*.pdf' --exclude='*.keshbackup' $S` (**10**) ;
    - **G5 — couverture** : `grep -rniE "inatteignable par le flux|garantit la couverture|incomplet en tables|couverture des tables|Couverture :|contrôle de couverture" --exclude='*.pdf' --exclude='*.keshbackup' $S` (**18**) ;
    - **valeur** : `grep -rn "20260917000001" crates/kesh-db/src crates/kesh-api/src` (**5** : `post_restore.rs:212, 251, 418,
      427, 433`) — chaque occurrence relue contre le sens neuf (plancher et non plus « dernière migration créatrice »).
    Nombres **rendus par ces commandes exécutées telles qu'écrites** le 2026-10-24 sur `8a5a720a` (code identique à
    `ffd2d306` : la branche n'a changé que des fiches). Les familles se recoupent (une ligne peut sortir de deux greps) ;
    les nombres sont des relevés par famille, non un total. **Chaque ligne rendue reçoit un sort, écrit au Dev Agent
    Record** : l'inventaire attendu est le tableau *Inventaire de la classe* des Dev Notes, et les homonymes exclus y
    sont **nommés un à un** ; une ligne qui ne figure ni au tableau ni parmi les homonymes nommés est un site neuf, à
    classer avant de clore le patch.
    **Découpage** : la 32-12a récrit les sites du sort *Récrit par la 32-12a* ; les trois sites du manuel
    (`admin-manual.tex:2252-2263`, `:2273`, `:2370`) ont le sort *Récrit par la 32-12b* (AC20). La 32-12a ne les rend
    **pas** faux : aucune table postérieure au plancher n'existe avant la 32-5, si bien que la réponse et le manuel
    décrivent encore exactement ce que fait l'import ; la liste des refus (l. 2253-2263) ne nomme aucun refus d'inventaire
    de tables aujourd'hui — son item générique « le **schéma** de la sauvegarde est incompatible » (l. 2255) reste vrai
    sous la règle neuve, et l'item des tables est **ajouté** —, `tables\_absent` (l. 2273) n'existe à l'audit qu'avec l'AC12, et la
    l. 2370 est fausse avant comme après (AC20 (vi)). La **32-12b relance** les cinq greps et la valeur, telles
    qu'écrits, après son patch du manuel, et confronte chaque ligne au tableau.
18. **AC18 — gates au dernier commit de code, une forme par clé.**
    - **AC18a [32-12a]** — `kesh-db` touché : **gate complet même en boucle** — `scripts/test-fast.sh` sous verrou
      (`docs/agents-orchestration.md` § 5), base `kesh_3212` remise à zéro d'abord ; `cargo publish --dry-run -p
      kesh-import` n'est pas concerné ; gates frontend **sans objet** (aucun fichier de `frontend/` ni de
      `crates/kesh-i18n/` touché — à constater par `git diff --name-only main...HEAD`) ; **E2E complet** au dernier commit
      de code (le chemin d'import change côté serveur, et `frontend/tests/e2e/backup-restore-new-install.spec.ts`
      l'emprunte), jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Ne déclarer que ce qui a
      tourné.
    - **AC18b [32-12b]** — `kesh-db` non touché : gate **ciblé** entre les passes (`fmt` et `clippy` sur le workspace,
      `cargo nextest run -E 'binary(admin_full_import_e2e)'` et les binaires des six tests lexicaux de T4b) ; gate
      **complet** (`scripts/test-fast.sh` sous verrou, base remise à zéro) au dernier commit de code ; les quatre gates
      frontend (`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) ; garde de parité i18n
      (`parity_between_locales`) verte ; **E2E complet** (frontend et backend changent), jugé fichier par fichier ;
      `make fr` sous verrou et PDF aplati (AC20) ; les greps de l'AC17 relancés. Ne déclarer que ce qui a tourné.
19. **AC19 — mutations** (sous verrou ; restauration `git checkout -- f && touch f`) : chacune doit faire rougir au
    moins un test **nommé** au Dev Agent Record. Numérotation conservée d'une clé à l'autre (traçabilité des
    validations) : **AC19a [32-12a]** = M1, M2, M3, M4, M6, M7, M8 ; **AC19b [32-12b]** = M5, M9.
    - M1 : `check_backup_inventory` tolère toute table absente (condition « postérieure au plancher » retirée) → AC3
      (`Exigee` : `journal_entry_number_sequences` absente seule, aucune présente postérieure, serait tolérée) et AC7
      (b) rouges (`products` tombe en `HorsOrdre`, message différent). **Ni l'AC6 (i)** (le motif ne traverse pas la
      réponse HTTP) **ni `parse_rejects_missing_table_coverage`** (`contains("absente du manifeste")`, fragment commun
      aux deux messages) ne la voient : ne pas les nommer comme tueurs.
    - M2 : règle du suffixe retirée → AC3 (`HorsOrdre`, et le cas « même migration ») rouge.
    - M3 : `IMPORT_FLOOR_MIGRATION = 20260827000001` (plancher abaissé d'un cran) → AC2 (iii) rouge (`exigees` = 38
      tables, `∪ TOLEREES_ECRITES` vide ≠ 39) et (iv) rouge (`tolerees_calculees` = [`journal_entry_number_sequences`]
      ≠ `TOLEREES_ECRITES` vide) — **une seule condition** : (iii) est impliquée par (iv) (AC2), le Dev Agent Record
      nomme (iv) comme tueur et (iii) comme son écho ; aussi `exemptions_claiming_out_of_window_really_are_out_of_window` (`20260828000001`
      et `20260910000001` ne sont plus `<` plancher).
    - M4 : `restore_body` ne vide plus une table absente de la carte (`DELETE` conditionné à `tables.contains_key`) →
      AC4 et AC7 rouges.
    - M5 : la réponse de `full_import` omet `tablesAbsent` → AC8 rouge (champ absent ≠ `[]`) ; l'audit de
      `run_backup_and_restore` omet `tables_absent` → AC8 rouge (détail d'audit).
    - M6 : AC9 défait (fenêtre = maximum des valeurs de `table_creating_migrations()`, ce que calculait
      `last_table_creating_migration`) → **équivalente aujourd'hui** (les deux valeurs sont `20260917000001`) : la
      déclarer telle, avec la raison, et la reporter à la 32-5 (AC10).
    - M7 : test de l'AC11 nourri d'une migration **fictive** (registre et `MIGRATOR` injectés par une porte de test, ou
      fonction de détection testée sur un SQL littéral `CREATE TABLE t (...); INSERT INTO t ...` de version > plancher,
      **et** sur une seconde migration fictive, postérieure, qui ne porte que `INSERT INTO t ...` — le cas de l'AC11
      où la table vient d'une migration antérieure) → la détection rend « à inscrire » dans les deux cas ; sans la
      porte, déclarer l'AC11 vacant et le dire.
    - M8 : `created_application_table` ne retire plus les accents graves → AC1 (i) reste-t-il vert ? Si oui (aucune
      migration du dépôt n'écrit `` CREATE TABLE `x` ``), mutation équivalente sur le dépôt actuel, à déclarer.
    - M9 : `full_import` écrit `tablesAbsent` (réponse) et `run_backup_and_restore` `tables_absent` (audit) **codés en
      dur à `[]`** → **non
      tuée avant la 32-5** : l'AC8 n'asserte qu'une liste vide, et l'AC7 n'emprunte pas le handler. Report assumé
      (C-32-12-20), à déclarer au Dev Agent Record ; le test qui la tuera est la reprise de l'AC8 par la 32-5
      (`ac_la_sauvegarde_de_la_0_15_0_s_importe` avec `tablesAbsent == [<sa table>]` et `tables_absent` à l'audit,
      AC10).
20. **AC20 [32-12b] — manuel d'administration, et son PDF.** `docs/manual/fr/admin-manual.tex`, § *Importer (restaurer la même
    installation)* (l. 2242 et suivantes) : (i) à la liste des refus (phrase d'annonce l. 2252, `itemize` l. 2253-2263),
    l'item « une **table** que la
    version installée exige manque à la sauvegarde, ou la sauvegarde porte une table que la version installée ne
    connaît pas (sauvegarde d'une version plus récente) » ; (ii) un paragraphe *Une sauvegarde d'une version
    antérieure* : depuis la 0.16.0, une sauvegarde qui s'importait **avant** une mise à jour s'importe encore **après**,
    même quand la mise à jour a ajouté des tables (le manuel ne cite pas de version plancher : une colonne
    obligatoire ajoutée depuis à une table ancienne reste un refus de schéma, indépendant de cette story) ; ces tables
    sont restaurées **vides** — ce qu'elles contenaient sur l'installation est effacé, comme tout le reste — et le bilan
    les nomme ; **aucun exemple tiré d'une story à venir** (le serveur MCP de la 32-5 n'est pas dans `main` quand la
    32-12 y arrive, et l'AC21 interdit d'en rien promettre) : l'exemple concret est ajouté par la story qui crée la
    première table postérieure (*Contrat* point 3, C-32-12-35) ; (iii)
    § *Ce que dit l'écran à la fin de l'import* (l. 2277) : l'encart de l'AC13 ; (iv) § *Reprises de données rejouées à
    l'import* (l. 2305) : rien à changer **sauf** si une phrase y lie l'importabilité à l'inventaire identique (à
    vérifier) ; (v) § *Les pièces à l'import*, l. 2273 : l'énumération des champs de l'entrée d'audit
    `admin.full\_import` (« `files\_restored`, `files\_missing`, avec `source\_format\_version` ») gagne
    `tables\_absent` (AC12) ; (vi) § migration vers une autre machine, l. 2370 (« dans une version **égale ou plus
    récente** que celle qui a produit la sauvegarde --- l'import refuse une sauvegarde qui exige une version plus
    récente ») — faux aujourd'hui pour une sauvegarde 0.11.x et encore après la story : la phrase gagne « … et une
    sauvegarde trop ancienne pour la version installée (liste des refus, § *Importer*) », sans citer de version
    plancher, comme (ii). `make fr` sous verrou ; contrôle **aplati** du PDF :
    `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF "restaurées vides"` (et la phrase
    de l'encart). Manuels DE/IT/EN : sans objet (non traduits) ; manuel utilisateur : rien (l'import est réservé à
    l'Administrateur — à vérifier par `grep -n "keshbackup" docs/manual/fr/user-manual.tex`).
21. **AC21 [32-12b] — CHANGELOG.** `## [0.16.0] — Non publié` (créé s'il n'existe pas encore — union avec les autres
    stories de l'epic), *Modifié* : « Une sauvegarde complète qui s'importait avant une mise à jour reste importable
    après une mise à jour qui ajoute des tables : ces tables sont restaurées vides, et le bilan de l'import les nomme
    (Stories 32-12a et 32-12b, refs #722). » Le CHANGELOG ne promet rien de la 32-5. **Une seule entrée, écrite par
    la 32-12b** : la 32-12a seule n'a aucun effet visible tant qu'aucune table postérieure au plancher n'existe (le
    bilan n'est pas encore câblé), et son entrée annoncerait un bilan qu'elle ne livre pas.

## Tâches

Préfixe de clé : **a** = 32-12a, **b** = 32-12b. Les tâches de la 32-12b ne commencent qu'une fois la 32-12a
**fusionnée** dans `main` (elle lit `ParsedBackup::tables_absent` et la fixture 0.15.0 que la 32-12a pose).

- [ ] **T0 (a, b)** — 32-12a : worktree `kesh-32-12`, branche `story/32-12` (déjà branchée, porte la fiche) ; 32-12b :
  worktree et branche attribués par l'orchestrateur, partis de `main` **après** la fusion de la 32-12a. Pour chacune :
  base `kesh_3212`, `kesh_e2e_3212` (ou celles que l'orchestrateur attribue), port attribué,
  `CARGO_TARGET_DIR=<worktree>/target` (`docs/agents-orchestration.md` § 4). Disque : vérifier la place libre avant le
  premier build (`memory` *Disque et worktrees*).
- [ ] **T1 (a ; AC15)** — **avant tout code** : `git diff --stat v0.15.0 HEAD -- crates/` vide constaté (sinon : worktree
  temporaire au tag `v0.15.0`, point 5, C-32-12-21) ; extraction du
  seed partagé, `generate_fixture_0_15_0`, exécution unique (`cargo nextest run --run-ignored only -E
  'test(generate_fixture_0_15_0)'` ou équivalent), contrôle `unzip`, commit dédié de la fixture et du générateur.
- [ ] **T2 (a ; AC1-AC4, AC10)** — `kesh-db` : reconnaissance `CREATE TABLE` déplacée en production (`post_restore.rs`,
  `created_application_table`, `split_statements` en `pub(crate)`), `table_creators` et `table_creating_migrations`,
  `IMPORT_FLOOR_MIGRATION` et son doc-comment (quatre migrations nommées, contrat de l'AC10), `InventoryRules`, `check_backup_inventory`, tests unitaires (`TOLEREES_ECRITES`,
  `tolerees_calculees`) ; test de l'AC4 ; commentaires de `TABLES_TO_TRUNCATE` et `restore_body`, message de
  `backup_schedule.rs:502`, doc-comment de `version.rs:272-275` (AC16).
- [ ] **T3 (a ; AC9, AC11)** — `post_restore.rs` : fenêtre = plancher dans les deux tests, `last_table_creating_migration`
  retirée, sites de l'AC9 récrits un à un (doc-module, l. 250-251, 269-274, 375-381, 916-934, 1084, 1095-1096),
  `post_floor_table_seeds_are_replayed`.
- [ ] **T4a (a ; AC5-AC7)** — `kesh-api/admin_backup` : `parse_and_verify_with` (`pub(crate)`), messages,
  `ParsedBackup::tables_absent` lu par le `Debug` manuel (AC5) ; tests AC6 (`tests/admin_full_import_e2e.rs`), AC7 (a)
  et (b) (`src/admin_backup/import.rs`, `mod tests`). `routes/admin.rs` **non touché**. Gate **complet** (kesh-db
  touché par T2-T3).
- [ ] **T4b (b ; AC8, AC12)** — `kesh-api/routes/admin` : réponse et audit ; test AC8 (`tests/admin_full_import_e2e.rs`,
  et assertion ajoutée au test 0.14.0) ; si le repli `cfg_attr(…allow(dead_code))` de l'AC5 a été pris par la 32-12a,
  le retirer. Les **six** tests lexicaux qui lisent `routes/admin.rs` sont
  relancés au gate ciblé (C-32-12-25) — les quatre de son propre `mod tests` (`include_str!("admin.rs")`, l. 1095) :
  `avant_sauvegarde_branchee_aux_appels_de_l_import` (l. 1037, compte un appel de `check_schema_compat`),
  `echecs_avant_sauvegarde_tous_convertis` (l. 1131, compte les `?` de `run_backup_and_restore` jusqu'à
  `write_pre_import_backup(` : AC12, n'en ajouter aucun), `import_dans_une_tache_detachee` (l. 1176),
  `aucun_blanc_parasite_dans_le_code` (l. 1221) ; et `auto_backup.rs::tests::gardes_possedes` (l. 1202, compte
  `lock_owned` dans `full_import`), `tests/admin_pat_denied_e2e.rs::the_three_intrinsic_guards_are_still_wired`
  (l. 563, compte `ensure_not_pat(&current_user)?`). Gate ciblé entre les passes, complet au dernier commit de code
  (AC18b).
- [ ] **T5 (b ; AC12-AC14)** — frontend : type, encart, clé × 4, Vitest ; captures de l'état nominal.
- [ ] **T6a (a ; AC16, AC17)** — textes du code et `CLAUDE.md` P7 (l. 827 **et** 829) ; les cinq greps et la valeur de
  l'AC17, chaque ligne confrontée à l'*Inventaire de la classe* (sites du manuel : sort *Récrit par la 32-12b*).
- [ ] **T6b (b ; AC20, AC21, AC17)** — manuel FR (AC20 (i) à (vi)), `make fr`, PDF aplati ; CHANGELOG ; les cinq greps
  et la valeur de l'AC17 relancés après le patch du manuel.
- [ ] **T7a (a ; AC18a, AC19a)** — mutations M1-M4, M6-M8 ; gate complet et E2E complet au dernier commit de code ;
  Dev Agent Record : chiffres réellement exécutés, M6 (et M8 le cas échéant) déclarées équivalentes avec la raison.
- [ ] **T7b (b ; AC18b, AC19b)** — mutations M5, M9 ; gate complet, frontend, E2E complet au dernier commit de code ;
  Dev Agent Record : chiffres réellement exécutés, M9 déclarée non tuée et reportée à la 32-5 ; risque résiduel de la
  capture non prise (AC14).

## Dev Notes

### Sites

| Site | Rôle aujourd'hui | Ce qui change |
|---|---|---|
| `crates/kesh-api/src/admin_backup/import.rs:232-250` | couverture + réciproque, inventaire identique | `check_backup_inventory` |
| `crates/kesh-api/src/admin_backup/import.rs:14, 232-233` | en-tête « couverture des tables », commentaire « Couverture : » | récrits (AC16) |
| `crates/kesh-api/src/admin_backup/import.rs:88-` | `ParsedBackup` | `tables_absent` |
| `crates/kesh-api/src/admin_backup/import.rs:626-` (`mod tests`) | tests de parsing, `Fabrique` | test de l'AC7 (`#[sqlx::test]`, `include_bytes!` de la fixture 0.15.0) |
| `crates/kesh-api/src/lib.rs:7-10` | `pub(crate) mod admin_backup` (revue 17-3a) | **inchangé** — aucune ré-exportation (C-32-12-15) |
| `crates/kesh-api/src/routes/admin.rs:262-270, 581-610` | réponse, audit | `tablesAbsent`, `tables_absent` |
| `crates/kesh-api/src/routes/admin.rs:1037, 1131, 1176, 1221`, `crates/kesh-api/src/auto_backup.rs:1202`, `crates/kesh-api/tests/admin_pat_denied_e2e.rs:563` | les six tests lexicaux qui lisent `routes/admin.rs` | inchangés ; relancés au gate ciblé de T4 ; aucun `?` ajouté avant `write_pre_import_backup(` (l. 1131) |
| `crates/kesh-api/src/errors.rs:2171-2180` | `InvalidBackupStructure` : motif journalisé, libellé générique dans la réponse | inchangé — d'où l'AC6 au code seul et l'AC7 (b) au message exact |
| `crates/kesh-db/src/backup.rs:34-104` | `TABLES_TO_TRUNCATE` (39 tables) | doc-comment ; `IMPORT_FLOOR_MIGRATION`, carte, règle à côté |
| `crates/kesh-db/src/backup.rs:627-720` | `restore_body` — vide tout, insère le présent | commentaires l. 647 et 667-668 seulement (comportement fixé par AC4) |
| `crates/kesh-db/src/repositories/backup_schedule.rs:502` | message de `table_systeme_hors_inventaire` | récrit (AC16) |
| `crates/kesh-db/src/post_restore.rs:19-37, 250-251, 257-279` | « toute table neuve referme la fenêtre », « aucune table n'est créée depuis `20260917000001` » | plancher (AC9) ; relevés datés du bloc 257-279 (l. 257-266, 276-279) gardés |
| `crates/kesh-db/src/post_restore.rs:375-381` | doc d'`EXEMPT_MIGRATIONS`, « recalcule la fenêtre depuis le `MIGRATOR` » (l. 379) | récrit (AC9) |
| `crates/kesh-db/src/post_restore.rs:441` | justification de `20260917000001`, « EST la dernière créatrice » | vraie jusqu'à la 32-5 ; récrite par elle (*Contrat* point 5, C-32-12-27) |
| `crates/kesh-db/src/post_restore.rs:718-720` | doc de `missing_sentinels`, « inatteignable par le flux réel » | récrit : l'état devient atteignable (AC16) |
| `crates/kesh-db/migrations/20261001000004_…:14-15`, `20261003000001_…:9-11`, `20261009000001_…:10-12`, `20261024000001_backup_schedule.sql:5-7` | commentaires « une table neuve rendrait les sauvegardes inimportables » | **intouchables (P8)** ; rectification au doc-comment d'`IMPORT_FLOOR_MIGRATION`, qui les nomme (AC2, AC16) |
| `crates/kesh-db/src/post_restore.rs:772` | `split_statements`, privée | `pub(crate)` (AC1) |
| `crates/kesh-db/src/post_restore.rs:916-934, 942-958, 978, 1084-1096, 1484` | fenêtre des tests et leurs doc-comments, `last_table_creating_migration`, reconnaissance `CREATE TABLE` (`-> bool`) | plancher ; doc-comments récrits ; fonction retirée ; reconnaissance en production, `created_application_table -> Option<&'static str>` (AC1, AC9) |
| `crates/kesh-db/src/version.rs:272-275` | doc de `check_import_version_compat`, « plus récente mais non-breaking reste importable » | récrit (AC16, C-32-12-26) |
| `CLAUDE.md:827, 829` | P7, « identique au nôtre », « recalcule la fenêtre depuis le `MIGRATOR` » | récrits, soumis à Guy (AC16) |
| `docs/manual/fr/admin-manual.tex:2252-2263, 2273, 2277, 2370` | refus, champs d'audit, écran, migration de machine | AC20 (i), (v), (iii), (vi) |
| `crates/kesh-api/tests/admin_full_export_e2e.rs:509-~620` | générateur 0.14.0 | seed partagé, générateur 0.15.0 |
| `crates/kesh-api/tests/admin_full_import_e2e.rs:3608, 3881` | fixture 0.14.0 et son test | fixture 0.15.0, AC6-AC8 |
| `frontend/src/lib/features/admin-restore/` | type, panneau, test | `tablesAbsent`, encart |
| `crates/kesh-i18n/locales/*/messages.ftl` | `admin-restore-files-missing` (fr l. 1769, autres l. 1698) | `admin-restore-tables-absent` |

### Inventaire de la classe (relevé du 2026-10-24 sur `8a5a720a`, greps G1-G5 et valeur de l'AC17 tels qu'écrits)

Chaque site de la classe — phrase qui affirme l'égalité des inventaires, la non-importabilité d'une sauvegarde
antérieure, une fenêtre calculée ou refermée par toute table neuve, ou que `20260917000001` est la dernière créatrice —
reçoit **un** sort. Les lignes des greps qui n'appartiennent pas à la classe sont des **homonymes**, nommés ici un à
un (relevé des commandes telles qu'écrites, 14 lignes) :

- G2 : `crates/kesh-db/src/repositories/reconciliation.rs:802` (« égalité **exacte** en décimal ») ;
- G3 : `crates/kesh-api/tests/rejeu_interblocage_e2e.rs:184`, `crates/kesh-api/tests/onboarding_audit_e2e.rs:1294`,
  `crates/kesh-db/tests/migrations_upgrade_path.rs:134`, `scripts/regen-test-schema.sh:342` (« codée en dur » d'autres
  valeurs) ; `frontend/src/lib/shared/i18n-libelle-en-dur.test.ts:759` (« recalculée depuis les classes ») ;
- G4 : `crates/kesh-db/tests/accounts_role_backfill.rs:29` (« la dernière du dépôt »),
  `frontend/src/routes/(app)/journal-entries/[id]/journal-entry-page.test.ts:166` et `:173`,
  `frontend/src/lib/features/reports/GeneralLedgerView.test.ts:178` (« la dernière colonne / de sa rangée ») ;
- G5 : `crates/kesh-db/src/version.rs:331` (« Note couverture » de test), `crates/kesh-api/tests/bank_account_closed_inventory.rs:1606`
  (contrôle de couverture d'un autre inventaire), `docs/kesh-specifications.txt:2530` (« Couverture : 89/89 FRs »),
  `frontend/src/lib/shared/i18n-literal-reader.test.ts:4` (« pour la couverture »).

G1 ne rend aucun homonyme : chacune de ses 51 lignes est au tableau. `docs/migrations-idempotence-audit.md:91` est
rendue par G1, G3 (« REFERME la fenêtre », d'où le `-i`) et G5.

| Sort | Sites |
|---|---|
| **Récrit par la 32-12a** | `backup.rs:34-38`, `:643-647`, `:667-668` ; `repositories/backup_schedule.rs:502` ; `version.rs:272-275` ; `post_restore.rs:19-37` (l. 22, 25-26, 35), `:250-251`, `:269-270`, `:272-274`, `:375-381` (l. 379), `:718-720`, `:916-927` (l. 918, 920, 925-927), `:930` et `:1084` (appels remplacés par le plancher), `:934`, `:942-958` (avec la fonction), `:1095-1096` ; `admin_backup/import.rs:14`, `:232-233` ; `CLAUDE.md:827`, `:829` |
| **Récrit par la 32-12b** (que la 32-12a ne rend pas faux, AC17) | `admin-manual.tex:2252-2263` (item ajouté), `:2273`, `:2370` |
| **Intouchable (P8), rectifié ailleurs** | `20261001000004_invoice_settlements_write_off.sql:14-15`, `20261003000001_invoices_frozen_pdf.sql:9-11`, `20261009000001_journal_entry_lines_lettering.sql:10-12`, `20261024000001_backup_schedule.sql:5-7` — rectification : doc-comment d'`IMPORT_FLOOR_MIGRATION`, qui les nomme (AC2), et cette fiche |
| **Repris par la 32-5** (*Contrat* point 5) | `post_restore.rs:441` (« cette migration EST la dernière créatrice de table applicative — donc la fenêtre elle-même ») : vrai jusqu'à la 32-5, faux dès sa table ; l'entrée commence à `:433` (valeur), et son « refusé au contrôle de couverture » (l. 436-437, coupé sur deux lignes, que G5 ne rend pas) reste vrai : la table est le plancher, exigée |
| **Reste vrai : relevé daté** | `post_restore.rs:212-215`, `:257-266`, `:276-279`, `:338-340`, `:1105-1121` ; `admin_full_import_e2e.rs:747`, `:947`, `:955`, `:1091`, `:1174`, `:1221`, `:1314`, `:1386`, `:1500` (« fenêtre refermée » : entrées de la Story 24-2 retirées du registre, rejeu déclenché explicitement), `:1523-1524`, `:1544`, `:2324-2325`, `:2383`, `:2505`, `:2577` ; `post_restore_class_a.rs:78` ; `docs/migrations-idempotence-audit.md:85`, `:91`, `:98` — chacun dit ce qui était vrai à sa story |
| **Reste vrai : table au plus tard au plancher, donc exigée** | les « refusé au contrôle de couverture » de `post_restore.rs:263`, `:421`, `:497`, `:518`, `:530`, `:536`, `admin_full_import_e2e.rs:2393`, `:2587`, `docs/migrations-idempotence-audit.md:91` ; les exemptions « Hors fenêtre » `post_restore.rs:418`, `:427` (et les six autres, point 2) ; `post_restore.rs:473-474` (justification de `20260915000001` : « elle EST dans la fenêtre d'importabilité », argument de parc) ; `import.rs:900` (« couverture KO » : `Fabrique` incomplète en tables exigées) |
| **Reste vrai : autre contrôle que l'inventaire** | `backup.rs:1148` et `admin_full_import_e2e.rs:2649`, `:2834`, `:3064` (importabilité par **colonne**) ; `admin_backup/manifest.rs:25` (par **format**) ; `20260915000001_audit_log_company_id.sql:23-24` (colonne `NOT NULL` sans défaut) ; `admin_auto_backup_e2e.rs:328`, `:335`, `admin_full_export_e2e.rs:858` (une archive fraîche s'importe) |
| **Reste vrai : générique** | `post_restore.rs:81-84`, `:207`, `:281`, `:376`, `:402`, `:449`, `:1027`, `:1138` (la fenêtre existe toujours, elle est le plancher) ; `scripts/prepare-release.sh:128` (des exemptions qui « ne s'appuient ni sur la fenêtre d'importabilité ni … » : vrai sous le plancher) ; `docs/migrations-idempotence-audit.md:105` (table système) |

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

1. Inscrire la table à `TOLEREES_ECRITES` (AC2) — l'AC2 (iii), écrit en union et disjonction sur cette liste, et
   l'AC2 (iv) restent alors verts sans réécriture ; s'ils rougissent, c'est que la table n'a pas été inscrite, non que
   le test soit à assouplir.
2. La table est **valide vide** : le code lit l'absence de ligne comme l'état par défaut. Sinon, sa migration amorce des
   lignes **et** va au registre (AC11), en classe A gardée (`INSERT … SELECT … WHERE NOT EXISTS`) — la classe B
   exigerait une sentinelle reconnue par `adds_column`, qui ne lit que les `ALTER TABLE … ADD COLUMN`. L'entrée porte un
   **extrait** des seuls statements d'écriture, sous `crates/kesh-db/src/post_restore/` (`registry_sql_contains_no_ddl`
   refuse le `CREATE TABLE` d'un `include_str!` du fichier entier, AC11). De même pour une migration **ultérieure** qui
   écrit dans la table (AC11, second cas).
3. Reprendre l'AC8 avec la fixture 0.15.0 : `tablesAbsent == [<table>]`, table vide après l'import ; captures de l'état
   « tables absentes » (AC14) ; ajouter au manuel (§ *Une sauvegarde d'une version antérieure*, AC20 (ii)) l'exemple
   concret de sa table, que la 32-12 ne peut pas écrire avant qu'elle existe (C-32-12-35).
4. Constater `registry_entries_are_within_import_window` vert sur M1 : c'est la preuve que l'AC9 tient (M6 cesse
   d'être équivalente).
5. Récrire la justification de l'exemption `20260917000001` (`post_restore.rs:441`) : « cette migration EST la dernière
   créatrice de table applicative — donc la fenêtre elle-même » devient faux dès la table de la story ; écrire « EST le
   plancher d'import `IMPORT_FLOOR_MIGRATION` — donc la fenêtre elle-même » (C-32-12-27). Et relancer les cinq greps de
   l'AC17 (G4 en particulier) : la table neuve rend fausses, par construction, les phrases « dernière créatrice » qui
   restaient vraies jusque-là.

### Ce que la story ne fait pas, et ce qui reste non résolu (inventaire)

- **Le retour en arrière** : une sauvegarde 0.16.0 importée par une 0.15.0 reste refusée (table inconnue → 400) ; le
  retour à la 0.15.0 passe par la sauvegarde d'avant la mise à jour, **et** par l'image 0.15.0 (`sqlx` refuse de
  démarrer sur une base qui porte une migration inconnue, `CLAUDE.md` P1). Inchangé, déjà écrit pour la release.
- **Les sauvegardes antérieures au plancher** (0.11.x et avant) : toujours refusées — ce n'est pas un recul, elles
  l'étaient déjà.
- **Une table recréée** par une seconde migration : la règle ne sait pas dater une telle table ; l'AC1 (i) le rend
  bruyant au lieu de deviner, et la production, si le cas passait, retient la plus ancienne créatrice — la table
  reste exigée, refus bruyant plutôt que tolérance à tort (C-32-12-18).
- **Le câblage d'une liste non vide** dans la réponse et l'audit : non prouvé avant la 32-5 (M9, report assumé).
- **L'encart à l'écran** : prouvé par Vitest, non capturé avant la 32-5 (AC14, dérogation déclarée).
- **L'authenticité** du fichier : un manifeste édité à la main peut se donner l'inventaire qu'il veut ; la règle protège
  contre une sauvegarde **tronquée par accident** ou d'un ordre impossible, non contre un fichier forgé — qui, de toute
  façon, est importé par un Administrateur et remplace toute l'installation.
- **#722** reste ouverte : la story n'en traite que l'import d'une sauvegarde antérieure.

### Fichiers touchés (prévus), par clé

- **32-12a** : `crates/kesh-db/src/backup.rs`, `crates/kesh-db/src/post_restore.rs`,
  `crates/kesh-db/src/repositories/backup_schedule.rs` (message d'assertion), `crates/kesh-db/src/version.rs`
  (doc-comment), `crates/kesh-api/src/admin_backup/import.rs`, `crates/kesh-api/tests/admin_full_export_e2e.rs`
  (générateur), `crates/kesh-api/tests/admin_full_import_e2e.rs` (AC6),
  `crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup` (neuf), `CLAUDE.md` (P7).
- **32-12b** : `crates/kesh-api/src/routes/admin.rs`, `crates/kesh-api/tests/admin_full_import_e2e.rs` (AC8),
  `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
  `frontend/src/lib/features/admin-restore/{admin-restore.api.ts,AdminRestorePanel.svelte,AdminRestorePanel.test.ts}`,
  `docs/manual/fr/admin-manual.tex` et `.pdf`, `CHANGELOG.md` ; et `crates/kesh-api/src/admin_backup/import.rs`
  **seulement** si la 32-12a a pris le repli `cfg_attr(…allow(dead_code))` de l'AC5, à retirer.

Seul fichier commun aux deux : `crates/kesh-api/tests/admin_full_import_e2e.rs` — des tests distincts (AC6 dans la
32-12a, AC8 et l'assertion ajoutée au test 0.14.0 dans la 32-12b) ; la 32-12b part de `main` après la fusion de la
32-12a, aucun conflit à fusionner.

**Modules** (règle de découpage préventif, ≤ 5), comptés au grain de l'exemple de la règle (`kesh-api/routes/invoices`)
et comme l'était le total de **huit** de la story entière — les fichiers sous `tests/` d'une crate et les docs à part :

- **32-12a — cinq** : `kesh-db/backup`, `kesh-db/post_restore`, `kesh-db/repositories` (un message d'assertion),
  `kesh-db/version` (un doc-comment), `kesh-api/admin_backup`. **Au seuil, non au-delà.** ⚠️ Compter
  `kesh-api/tests` comme un module en donnerait **six** : ce grain n'a pas servi au total de huit, et il n'est pas
  retenu ici ; le Project Lead peut en juger autrement (Q4).
- **32-12b — trois** : `kesh-api/routes/admin`, `kesh-i18n`, `frontend/admin-restore` (quatre avec
  `kesh-api/admin_backup` si le repli de l'AC5 est à retirer).

Huit modules distincts au total (5 + 3, sans recoupement). Aucune migration dans l'une ni l'autre, donc ni P2/P3 (pas
de bump), ni P5 (pas de ligne d'audit d'idempotence), ni P6 (aucun `.sql` neuf ; les deux tests positionnels ne sont
pas touchés) ; **P7** est l'objet même de l'AC9 et de l'AC11, tous deux à la 32-12a.

### Découpage

*(Remplace la section « Dérogation règle de splitting », écrite à la validation P2 et récrite à la P3 ; son texte se
relit au commit `7d7ade33`. Choix C-32-12-37.)*

- **Motif — les deux conditions de l'amendement D5, toutes deux constatées.** `CLAUDE.md` § *Règle de splitting
  préventif*, amendement D5, cité tel qu'il est écrit : « Ce qui découpe, c'est le **recyclage** — un défaut qui revient
  sous une autre forme, ou qui naît du correctif précédent — et la dispersion dans plus de cinq modules. »
  - **Recyclage partiel de la classe « textes devenus faux sans être nommés »** : R1 = F1 de la P2 (quatre migrations
    publiées, la fiche en nommait trois) est né de la remédiation P1 (`544c2821`) ; R1 de la P3 (les greps de l'AC17
    sans `-i`) est né de la remédiation P2 ; la classe est revenue à chaque passe (F3 = R2 en P1, R2 = F3 en P2, R1 = F1
    et F9 en P3). Sévérités, lentille R puis F : P1 0/0/2/6 et 0/1/2/5 → P2 0/0/2/6 et 0/0/3/6 → P3 0/0/2/7 et 0/0/0/9
    → P4 (Haiku, ciblée) 0/0/2/1.
  - **Dispersion sur huit modules** (§ *Fichiers touchés*), pour un seuil de cinq.
- **Pourquoi la dérogation est retirée — P4-1 et P4-2** (`kesh-gate-logs/3212-validate-p4.md`) :
  - **P4-1 (MEDIUM)** : C-32-12-36 (« Écartée : découper d'office (décision du Project Lead, D5 : le signal se déclare,
    il ne découpe pas seul) ») invoquait D5 pour ne **pas** découper, en omettant la phrase qui fait du recyclage et de
    la dispersion ce qui découpe — alors que la dérogation constatait elle-même ces deux conditions. Ce n'était pas une
    application de D5, c'était une dérogation à D5 présentée comme son application.
  - **P4-2 (MEDIUM)** : son argument (a) — « aucune passe n'a trouvé deux fois un défaut de la règle, de la carte, de
    l'import ou du bilan » — est contredit par les rapports qu'elle citait : l'AC2 (iii) revient en P1 (F2, R3), P2 (F4,
    R6) et P3 (R6 = F7) ; la carte de l'AC1 en P1 (F5) et P3 (R4 = F3). Le recyclage n'était donc pas circonscrit aux
    textes.
  Sans l'argument (a), et D5 relu tel qu'écrit, rien ne fonde plus la dérogation.
- **Décision de l'orchestrateur** (2026-10-24) : la fiche est découpée selon la coupe de repli qu'elle proposait.
  **Soumise à Guy avec Q4** : s'il juge que la story devait rester une, les deux clés se recousent sans perte (même
  fiche, mêmes AC, numérotation conservée).
- **La coupe** — la règle reste avec son appelant et avec **tous** les textes qu'elle rend faux :
  - **32-12a** (`32-12a-regle-inventaire-et-textes`) : AC1-AC7, AC9-AC11, AC15-AC17 — fixture, règle, carte, plancher,
    import qui applique la règle, fenêtre des tests P7, registre des tables amorcées, textes du code et de `CLAUDE.md` ;
    AC18a, AC19a (M1-M4, M6-M8). Cinq modules. 🗄️ `kesh-db`.
  - **32-12b** (`32-12b-bilan-ecran-manuel`) : AC8, AC12-AC14, AC20-AC21 — réponse et audit, écran, captures, manuel,
    CHANGELOG ; AC18b, AC19b (M5, M9). Trois modules. Sans `kesh-db`.
  - Aucun AC ne tombe entre les deux : union = AC1-AC21 ; seuls AC18 et AC19 sont portés par les deux, chacun dans une
    forme propre à sa clé.
- **Ce que la coupe répond aux arguments de l'ancienne dérogation** : (b) aucun texte rendu faux par la règle ne
  part dans la 32-12b — les trois sites du manuel qu'elle récrit ne sont pas rendus faux par la 32-12a (AC17) ; (d) la
  32-12a ne livre pas une règle que rien n'emploie : l'import l'applique en production (AC5), et le contrat de la
  fixture (T1 avant tout code) reste entier dans la 32-12a.
- **Ordre** : 32-12a, puis 32-12b (qui lit `ParsedBackup::tables_absent` et la fixture 0.15.0), **les deux avant la
  32-5**. Entre les deux fusions, `main` porte une règle qui tolère des tables postérieures absentes sans le dire au
  bilan : sans effet, aucune table postérieure au plancher n'existant avant la 32-5 — d'où l'interdit de faire partir la
  32-5 avant la fusion de la 32-12b.
- **Risque accepté** : un défaut d'interaction entre la règle (32-12a) et le bilan (32-12b), qu'aucune des deux
  boucles de revue ne voit en entier ; atténué par l'AC7 (chaîne bout à bout hors HTTP, 32-12a), par l'AC8 (HTTP,
  32-12b, sur le code fusionné de la 32-12a) et par la reprise de l'AC8 à la 32-5 ; et un texte devenu faux qu'aucune
  famille de l'AC17 ne rend, atténué par la règle « une ligne absente du tableau est un site neuf » et par la relance
  des greps à la 32-12b.

### Risques

- **`kesh-db` et l'import** : la base d'une installation réelle passe par ce chemin. Gate complet à chaque passe.
- **Le message d'un test qui ment** : le risque principal de l'epic n'est pas dans cette story mais dans la 32-5, si
  l'AC9 n'est pas fait — `registry_entries_are_within_import_window` y prescrirait de retirer M1 du registre. D'où
  l'AC10 et la demande à l'orchestrateur (*Hors périmètre*).
- **Ordre d'épique** : 32-12a, puis 32-12b, toutes deux **avant** la 32-5 (C-32-0-12, C-32-0-13, C-32-12-37) ; si la 32-5 part d'une `main` sans cette
  story, sa table rend la fixture 0.15.0 non importable et son gate le dit (AC8 rouge) — signal voulu.
- **Fixture** : la générer après un changement de code produirait une sauvegarde qui n'est plus celle de la 0.15.0, et
  le test ne prouverait plus rien. T1 est le premier geste, en commit dédié ; si la branche ne porte plus le code de la
  v0.15.0, repli au worktree temporaire (point 5).
- **Encart invisible** : l'état « tables absentes » n'est pas capturé ici (AC14) ; risque du précédent #640, atténué par
  les classes partagées avec `admin-restore-files-missing`, levé par la capture de la 32-5.

### Hors périmètre — demandes à l'orchestrateur

- Mettre à jour, après merge, la prémisse 4, le texte de la 32-5 (« Risque … sauvegardes 0.15.0 non importables ») et le
  paragraphe *Release v0.16.0* du fichier d'epic, ainsi que le commentaire de `32-5-cles-et-reglages-mcp` dans
  `sprint-status.yaml` : la limite disparaît, le contrat de la § *Contrat* ci-dessus s'y ajoute.
- Faire développer la 32-12a **avant** que du code d'une autre story de l'epic n'atteigne la branche (sinon la fixture
  passe par le repli du point 5, plus coûteux en disque).
- Après la fusion de la 32-12a : ouvrir le worktree et la branche de la 32-12b depuis `main` ; ne faire partir la 32-5
  qu'après la fusion de la 32-12b (§ *Découpage*, *Ordre*).
- Soumettre à Guy, avec Q4, la décision de découpage (C-32-12-37) et le décompte de modules de la 32-12a (cinq, au
  seuil ; six si l'on compte `kesh-api/tests`).

### Références

`CLAUDE.md` § *Migration breaking policy* (P1, P2-bis, P7), § *Exception kesh-db*, § *Recette visuelle* ;
`docs/agents-orchestration.md` § 2.1, § 4, § 5 ; fichier d'epic § *32-12*, § *Ce que le code dit déjà* point 4,
§ *Questions pour Guy* Q4 ; registre `epic-32-choix-autonomes.md` C-32-0-12, C-32-0-13 ; fiche 31-1 (fixture 0.14.0,
AC10, `31-1-sauvegarde-complete.md:335-345, 1325-1335`).

## Dev Agent Record

*Un enregistrement par clé : chacune déclare ses seuls gates, mutations et fichiers.*

### 32-12a

#### Agent Model Used

Opus 5.5 (agent de développement), worktree `kesh-32-12a`, branche `story/32-12a` partie de `9ae93f4c` ; bases
`kesh_3212a` / `kesh_e2e_3212a`, port 3319, `CARGO_TARGET_DIR` du worktree. Développement du 2026-10-24, **sous
réserve de Q4**.

#### Completion Notes List

*Ne figure ici que ce qui a tourné ; journaux hors dépôt sous `kesh-gate-logs/3212a-*.log`.*

- **T1 / AC15 — fixture.** `git diff --stat v0.15.0 HEAD -- crates/ frontend/src` constaté **vide** avant tout code ;
  seed extrait (`generer_fixture`, partagé avec `generate_fixture_0_14_0`, qui garde son doc-comment et son
  `#[ignore]`) ; `generate_fixture_0_15_0` exécuté **une fois** (`cargo nextest run --run-ignored only -E
  'test(generate_fixture_0_15_0)'`) ; commit dédié `642c88f9`, **avant** tout changement de code. Contrôle `unzip` :
  `formatVersion` 2, `keshVersion` `0.15.0`, `keshVersionMinRequired` `0.14.0`, **39** tables, deux entrées
  `files/<sha>.pdf` ; `products` à 0 ligne. **91 977 octets, SHA-256
  `5729ce13f6ee8a7280f26b05c127ca47da2b3fa5a5ac43a08de615e3f0fad5c6`.** Contenu synthétique (société du seed, « Client
  Fictif », IBAN d'exemple SIX).
- **AC1-AC4** (`kesh-db/backup.rs`, `post_restore.rs`) : `created_application_table -> Option<&'static str>`,
  `normalize` et `split_statements` en `pub(crate)` (production, un seul exemplaire) ; `table_creators()` et
  `table_creating_migrations()` (`OnceLock`, minimum) ; `IMPORT_FLOOR_MIGRATION = 20260917000001`, doc-comment avec
  contrat (AC10) et les **quatre** migrations publiées nommées une à une ; `InventoryRules` (`creation` en pointeur de
  fonction, C-32-12-38), `InventoryError`, `check_backup_inventory` (`HorsOrdre` cite la plus récente des présentes,
  C-32-12-39). La fonction de test qui calculait la fenêtre est **retirée**. Tests : AC1 (i)
  `every_inventory_table_has_exactly_one_creating_migration`, (ii) `table_creating_migrations_dates_known_tables`,
  plus `created_application_table_reads_the_header_only` ; AC2 (i) `import_floor_is_a_real_table_creating_migration`,
  (ii) `import_floor_is_not_beyond_the_latest_creation`, (iii) `required_and_written_tolerated_partition_the_inventory`,
  (iv) `tolerated_tables_are_written_down` (`TOLEREES_ECRITES` vide, `tolerees_calculees`, `exigees`) ; AC3 : onze
  tests `inventory_*` (les dix cas de la fiche ; le cas « table du plancher absente » porte trois assertions, dont
  celle de M1) ; AC4 `restore_empties_a_table_absent_from_the_backup` (`./test-schema`).
- **AC5** : `parse_and_verify_with` (`pub(crate)`), `parse_and_verify` l'enveloppe ; messages `Exigee` et `Inconnue`
  inchangés mot pour mot, `HorsOrdre` à la lettre de l'AC5 ; `ParsedBackup::tables_absent` lu par le `Debug`
  manuel — **`clippy -D warnings` vert sans le repli `cfg_attr`** (aucun lint sur le champ) ; `lib.rs` non touché.
- **AC6** (`tests/admin_full_import_e2e.rs`) : `ac6_une_table_exigee_absente_est_refusee_sans_effet`,
  `ac6_une_table_inconnue_est_refusee_sans_effet` (400 `INVALID_BACKUP_STRUCTURE`, société témoin relue,
  `KESH_ADMIN_BACKUP_DIR` vide). **AC7** (`src/admin_backup/import.rs`, `mod tests`) :
  `ac7_une_table_posterieure_absente_est_restauree_vide` (`#[sqlx::test]`), `ac7_une_table_exigee_absente_est_refusee_par_son_message_exact`.
- **AC9, AC11** : fenêtre des deux tests P7 = `IMPORT_FLOOR_MIGRATION` ; sites récrits (doc-module, commentaire de M1,
  phrases au présent du bloc 24-2/24-3 — relevés datés gardés —, doc d'`EXEMPT_MIGRATIONS`, doc et messages des deux
  tests) ; compteur `checked` à **8**, son commentaire daté inchangé ; justification de l'exemption `20260917000001`
  **non touchée** (confiée à la 32-5). AC11 : `post_floor_table_seeds_are_replayed` (vacant) et sa détection
  `post_floor_write_targets`, nourrie de migrations fictives par `post_floor_write_targets_on_fictitious_migrations`
  (C-32-12-40).
- **AC16** : `TABLES_TO_TRUNCATE`, `restore_body` (l'audit_log hors inventaire ; le commentaire « garantit la
  couverture »), `missing_sentinels` (et le commentaire de son test), en-tête et commentaire de `import.rs`,
  `backup_schedule.rs` (message), `version.rs`, `CLAUDE.md` P7 (les **deux** paragraphes — **soumis à Guy par la
  PR**). Aucune migration ajoutée ni touchée (P5, P6, P8 sans objet : `git diff --stat story/32-12 HEAD --
  crates/kesh-db/migrations docs/migrations-idempotence-audit.md` vide).
- **Rouges avant correctif** (`3212a-rouge-kesh-db.log`, `3212a-rouge-kesh-api.log`) : sous bouchons
  (`table_creators` vide, plancher à 0, `check_backup_inventory` → `Ok(vec![])`, détection AC11 vide), **15 rouges**
  sur 24 tests visés : les tests AC1 (i)/(ii), AC2 (i)/(ii), neuf `inventory_*`, l'exemption P7, la détection
  fictive AC11. Restaient verts **par construction** et sont tués par mutation : `inventory_complete_is_ok`,
  `inventory_production_rules_complete_manifest_is_ok` (`Ok(vec![])` est leur réponse juste), AC2 (iii)/(iv) (une carte
  vide rend tout exigé, partition cohérente → tués par M3), `restore_empties_…` (comportement existant, AC4 le **fixe**
  → tué par M4), `post_floor_table_seeds_are_replayed` (vacant → tué par M3), `registry_entries_…` (M6). Côté
  `kesh-api` (ancienne double boucle, règles ignorées) : `ac7_…_restauree_vide` **rouge** (`table 'products' absente
  du manifeste`) ; AC6 (i)/(ii) et AC7 (b) verts — les refus existaient, le message aussi : tués par M1 (AC7 b).
  `created_application_table_reads_the_header_only` vert (reconnaissance réelle dès l'écriture) → tué par M8.
- **Mutations** (`3212a-mutations.log`, sous verrou, une à la fois, `git checkout -- f && touch f`, `git status`
  propre après) :
  - **M1** (filtre « postérieure au plancher » retiré) → tuée : `inventory_floor_table_absent_is_required`,
    `ac7_une_table_exigee_absente_est_refusee_par_son_message_exact`. Les AC6 restent verts (attendu, la fiche le dit).
  - **M2** (règle du suffixe neutralisée) → tuée : `inventory_older_absent_newer_present_is_out_of_order`,
    `inventory_same_migration_one_absent_is_out_of_order`.
  - **M3** (plancher à `20260827000001`) → tuée : `tolerated_tables_are_written_down` (tueur), avec son écho
    `required_and_written_tolerated_partition_the_inventory` ; aussi
    `exemptions_claiming_out_of_window_really_are_out_of_window` et `post_floor_table_seeds_are_replayed`
    (`20260917000001` écrit dans une table devenue postérieure).
  - **M4** (DELETE conditionné à `tables.contains_key`) → tuée : `restore_empties_a_table_absent_from_the_backup`,
    `ac7_une_table_posterieure_absente_est_restauree_vide`.
  - **M6** (fenêtre = maximum de `table_creating_migrations()`), appliquée aux **deux** tests séparément (M6, M6b) →
    **équivalente, déclarée** : les deux valeurs valent `20260917000001` aujourd'hui ; elle cesse de l'être à la
    première table postérieure (32-5, contrat point 4).
  - **M7** (détection AC11 : aucune cible jamais postérieure) → tuée : `post_floor_write_targets_on_fictitious_migrations`.
  - **M8** (accents graves non retirés) → **tuée** par `created_application_table_reads_the_header_only` ; l'AC1 (i)
    reste vert sous M8 (aucune migration n'écrit `` CREATE TABLE `x` ``) — d'où le test unitaire (C-32-12-41).
- **Gate complet** au dernier commit de code (`c70b80e6`), base `kesh_3212a` remise à zéro (drop, migrations du
  worktree, seed) : `scripts/test-fast.sh --ci` sous verrou — garde des droits, `fmt`, `clippy -D warnings`,
  **3996 exécutés, 3996 passés, 8 ignorés** (`3212a-gate-1.log`). Gates frontend **sans objet** :
  `git diff --name-only story/32-12 HEAD` ne touche ni `frontend/` ni `crates/kesh-i18n/`.
- **E2E complet** au même commit (port 3319, `kesh_e2e_3212a` recréée vide, dossiers `/tmp/kesh-e2e-3212a/*` passés
  au backend **et** au runner, `/health` `smtpConfigured: true`) : **315 passés, 7 échoués, 17 sautés** (14,2 min,
  `3212a-e2e.log`). Les sept, un à un contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`,
  `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ;
  ni KF-052 ni pollution. `backup-restore-new-install.spec.ts` (qui passe par l'import) : 3/3 passés. Backend arrêté
  par son PID.
- **Tests ajoutés**, recomptés aux deux bornes `story/32-12` → `HEAD` (`#[test]` + `#[sqlx::test]`) : `backup.rs`
  28 → 47 (**+19**), `post_restore.rs` 19 → 21 (**+2**), `admin_backup/import.rs` 23 → 25 (**+2**),
  `admin_full_import_e2e.rs` 55 → 57 (**+2**), `admin_full_export_e2e.rs` 20 → 21 (**+1**, le générateur
  `#[ignore]`) : **+26** attributs, dont **25** exécutés par le gate.
- **AC17, après réécriture (P5-5)** — les six commandes telles qu'écrites, depuis la racine du worktree, sur `HEAD`
  (`c70b80e6` + docs) : **G1 58, G2 5, G3 27, G4 5, G5 14, valeur 7** (avant : 51 / 7 / 36 / 10 / 18 / 5). Sort des
  lignes qui ont changé (comparaison ligne à ligne contre `story/32-12`) :
  - **disparues** (sites *Récrit par la 32-12a* du tableau, tous réécrits) : `backup.rs:34-38`, `:647`, `:667-668` ;
    `backup_schedule.rs:502` ; `version.rs:272-275` ; `post_restore.rs` doc-module l. 22/25-26/35, doc
    d'`EXEMPT_MIGRATIONS` l. 379, `missing_sentinels` l. 718-720, test l. 918/920/925-927/930/934/1084/1095-1096,
    fonction retirée l. 942-958 ; `import.rs:14`, `:232-233` ; `CLAUDE.md:827`, `:829` ;
  - **neuves, toutes vraies sous la règle neuve** (sort : *écrit par la 32-12a*) — G1 : `backup.rs:42`, `:47`
    (`TABLES_TO_TRUNCATE` : ajouter reste importable, retirer ne l'est pas), `:151`, `:160` (passé : « rendait »),
    `:165` (relever le plancher), `:198` (citation de l'ancienne règle des quatre migrations), `:879`
    (`audit_log` hors inventaire) ; `post_restore.rs:27`, `:262` (M1), `:825` (doc de la reconnaissance), `:996`,
    `:1111`, `:1127`, `:1806` (test fictif) ; `CLAUDE.md:827` récrit ; G2 : `backup.rs:197` (« l'ancienne règle
    d'égalité des inventaires », passé) ; G3 : `backup.rs:164` (« Le relever referme la fenêtre » : vrai du
    relèvement) ;
  - **valeur** (7) : `post_restore.rs:222`, `:434`, `:443`, `:449` (inchangés, tableau), `backup.rs:183` (contrat,
    point 3), `:205` (la constante), `:2563` (test AC1 (ii)) — chacune relue : plancher, non « dernière créatrice ».
  - **inchangées** : toutes les autres lignes gardent leur sort du tableau *Inventaire de la classe* (relevés datés,
    exigées, autre contrôle, génériques, les quatre migrations P8, `post_restore.rs:441`/`:457` confié à la 32-5, les
    trois sites du manuel confiés à la 32-12b) ; les 14 homonymes, inchangés. Aucun site neuf hors de ces sorts.
- **Écarts** : (1) une réécriture de code (le bloc du test `registry_entries_are_within_import_window` et le retrait
  des deux fonctions de test voisines) a été faite par un script `python3` de remplacement, contre
  `docs/agents-orchestration.md` § 7 — résultat relu aussitôt, conforme (C-32-12-42) ; toutes les autres éditions de
  code par l'outil d'édition. (2) La fiche (AC18a) nomme la base `kesh_3212` ; l'orchestrateur a attribué
  `kesh_3212a` — c'est elle qui a servi. (3) M8 n'est pas équivalente mais tuée (C-32-12-41).

#### File List

- `crates/kesh-db/src/backup.rs` — plancher, carte, règle, tests AC1-AC4, commentaires.
- `crates/kesh-db/src/post_restore.rs` — reconnaissance en production, fenêtre P7 au plancher, AC11, textes.
- `crates/kesh-db/src/repositories/backup_schedule.rs` — message d'assertion.
- `crates/kesh-db/src/version.rs` — doc-comment de `check_import_version_compat`.
- `crates/kesh-api/src/admin_backup/import.rs` — `parse_and_verify_with`, `tables_absent`, tests AC7.
- `crates/kesh-api/tests/admin_full_export_e2e.rs` — seed partagé, `generate_fixture_0_15_0`.
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — tests AC6.
- `crates/kesh-api/tests/fixtures/keshbackup-0.15.0-format2.keshbackup` — neuf.
- `CLAUDE.md` — P7, deux paragraphes.
- `_bmad-output/implementation-artifacts/3212a-review-prompt-p1.md` — neuf ; registre `epic-32-choix-autonomes-3212.md`
  (C-32-12-38 à 42) ; `sprint-status.yaml`.

### 32-12b

#### Agent Model Used

#### Completion Notes List

#### File List

## Change Log

- 2026-10-24 — Fiche créée (Opus 5.5), `ready-for-dev` **sous réserve de Q4**.
- 2026-10-24 — **Validation P1** (Sonnet 5.5, deux lentilles en contexte frais ; prompt `3212-validate-prompt-p1.md`) :
  lentille R **0/0/2/6**, lentille F **0/1/2/5** (CRITICAL/HIGH/MEDIUM/LOW). Convergences : **F1 = R1**, **F2 = R3**,
  **F3 = R2**, F4 = R5, F8 = R8. Aucun finding réfuté ; les sites ont été revérifiés au code par l'agent de
  remédiation (`lib.rs:10`, lignes citées, fixture 0.14.0 : `products` à 0 ligne). Sort :
  - **F1 (HIGH) = R1** — AC7 placé dans la crate (`import.rs`, `mod tests`, `#[sqlx::test(migrations =
    "../kesh-db/test-schema")]`, `include_bytes!` de la fixture, retrait de `products` par une fonction locale) ; porte
    `pub(crate)`, mention `#[doc(hidden)]` retirée ; aucune ré-exportation, `lib.rs` inchangé (C-32-12-15 ; C-32-12-12
    annoté).
  - **F2 (MEDIUM) = R3** — AC2 (iii) récrit `exigées ∪ tolérées == TABLES_TO_TRUNCATE`, `exigées ∩ tolérées == ∅` ;
    contrat, point 1, complété (C-32-12-16).
  - **F3 (MEDIUM) = R2** — commande de l'AC17 remplacée par un motif ciblé (28 lignes au relevé) ; AC16 et tableau des
    sites étendus à `backup_schedule.rs:502`, `post_restore.rs:718-720`, `import.rs:14` et `:232-233`,
    `backup.rs:667-668` ; les commentaires devenus faux de trois migrations publiées (`20261024000001:5-7`,
    `20261001000004:15`, `20261009000001:11`) **ne se touchent pas (P8)** : rectification au doc-comment
    d'`IMPORT_FLOOR_MIGRATION` et aux Dev Notes (C-32-12-17).
  - **F4 = R5 (LOW)** — numéros de ligne relevés : `import.rs:232-250`, `:88`, `admin.rs:262-270`, `:581-610`,
    `post_restore.rs:226`, `admin_full_export_e2e.rs:534`.
  - **F5 (LOW)** — deux migrations créatrices : la plus ancienne, table exigée, refus bruyant (AC1, C-32-12-18).
  - **F6 (LOW)** — texte de l'encart sans promesse de recréation (AC13, C-32-12-19).
  - **F7 (LOW)** — les deux tests lexicaux qui lisent `routes/admin.rs` nommés (T4, tableau des sites).
  - **F8 = R8 (LOW)** — report de la capture à la 32-5 écrit comme dérogation, risque résiduel et précédent #640 (AC14,
    *Risques*, inventaire).
  - **R4 (LOW)** — mutation M9 (`tablesAbsent` codé à `[]`) ajoutée, report assumé à la 32-5, test qui la tuera nommé
    (AC19, C-32-12-20).
  - **R6 (LOW)** — doc-comment de la reconnaissance déplacée : pourquoi `split_statements` suffit, garde
    `migrations_contain_no_block_comment` (AC1).
  - **R7 (LOW)** — repli de la fixture : worktree temporaire au tag si le diff n'est plus vide (point 5, AC15, T1,
    C-32-12-21).
  Passe suivante : **P2 complète, Opus**, lentilles R et F (`3212-validate-prompt-p2.md`) — la P1 a rendu un HIGH et
  deux MEDIUM.
- 2026-10-24 — **Validation P2** (Opus 5.5, deux lentilles en contexte frais ; prompt `3212-validate-prompt-p2.md` ;
  rapports `kesh-gate-logs/3212-validate-p2-{R,F}.md`) : lentille R **0/0/2/6**, lentille F **0/0/3/6**
  (CRITICAL/HIGH/MEDIUM/LOW). Convergences : **R1 = F1**, **R2 = F3**, **R3 = F2** (LOW chez R, MEDIUM chez F : retenu
  MEDIUM — le défaut ne se voit qu'au gate complet de `kesh-db`), R4 = F5, R6 = F4. Aucun finding réfuté ; chaque site
  revérifié au code par l'agent de remédiation. **Motif commun des MEDIUM** : des textes qui deviennent faux sans être
  nommés — la remédiation n'a donc pas corrigé site par site mais **inventorié la classe par son symptôme** (cinq
  familles de `grep` et la valeur, AC17 ; tableau *Inventaire de la classe* des Dev Notes, chaque site avec son sort ;
  C-32-12-22). ⚠️ L'entrée « Validation P1 » ci-dessus dit « trois migrations publiées » et « les deux tests
  lexicaux » : relevés faux à leur date, laissés tels quels et rectifiés ici (quatre migrations, six tests). Sort :
  - **R1 = F1 (MEDIUM)** — **quatre** migrations publiées (ajout de `20261003000001_invoices_frozen_pdf.sql:9-11`) à
    l'AC16, au tableau des sites, à l'inventaire et à C-32-12-17 (annoté) ; la phrase du doc-comment d'`IMPORT_FLOOR_MIGRATION`
    les nomme une à une au lieu de les compter (AC2) ; « ces trois migrations » ne subsiste nulle part hors du relevé P1.
  - **R2 = F3 (MEDIUM)** — AC9 énumère désormais chaque site récrit de `post_restore.rs` (doc-module, l. 250-251,
    269-276, 375-381, 916-934, 942-957, 1095) ; AC16 étend `CLAUDE.md` P7 à la l. 829 ; `post_restore.rs:441`
    (« EST la dernière créatrice ») reste vrai jusqu'à la 32-5 et lui est confié (*Contrat* point 5, AC10,
    C-32-12-27) ; famille G4 ajoutée aux greps.
  - **F2 = R3 (MEDIUM)** — AC4 en `#[sqlx::test(migrations = "./test-schema")]`, garde
    `test_schema_guard.rs::every_sqlx_test_attribute_is_accounted_for` et `SQUASH_SPELLINGS` nommés.
  - **F4 = R6 (LOW)** — deux objets nommés, `TOLEREES_ECRITES` et `tolerees_calculees` ; AC2 (iii) écrit sur la liste
    écrite (non tautologique : une table postérieure non inscrite n'est dans aucun ensemble) ; « tolérées » seul réservé
    à l'AC3 ; M3 précisée (C-32-12-28).
  - **F5 = R4 (LOW)** — message `HorsOrdre` exact dans le cas « même migration » (« créée par la même migration qu'elle
    ou après ») ; M1 tuée par l'AC3 (`Exigee`) et par un test neuf, AC7 (b), qui asserte l'égalité exacte du message
    d'`Exigee` (C-32-12-24). **Relevé de la remédiation, au-delà des deux rapports** : l'AC6 (i) assertait le message
    `absente du manifeste` dans une réponse HTTP qui **ne le porte pas** (`errors.rs:2171-2180` : motif journalisé,
    libellé générique) — critère invérifiable, que les deux lentilles avaient raisonné comme lisible ; l'AC6 n'asserte
    plus que le code, et le message passe à l'AC7 (b).
  - **F6 (LOW)** — reconnaissance déplacée dans `post_restore.rs`, `split_statements` en `pub(crate)` ;
    `last_table_creating_migration` retirée (sans appelant après l'AC9) ; AC2 (ii) sur le maximum de la carte ; M6
    récrite (C-32-12-23).
  - **F7 (LOW)** — les **six** tests lexicaux qui lisent `routes/admin.rs` nommés (T4, tableau des sites), dont
    `echecs_avant_sauvegarde_tous_convertis`, signalé à l'AC12 (aucun `?` ajouté) (C-32-12-25).
  - **F8 (LOW)** — AC12 : audit écrit dans `run_backup_and_restore` (depuis `parsed`), réponse dans `full_import`
    (depuis `ImportOutcome`) ; M5 et M9 nomment les deux fonctions.
  - **F9 (LOW)** — C-32-12-10 annoté (repli de C-32-12-21) ; AC20 (i) aux l. 2252-2263 ; modules comptés au grain de la
    règle — **huit** avec `version.rs` (sept au relevé de F) — et section *Dérogation règle de splitting* écrite
    (C-32-12-29).
  - **R5 (LOW)** — `version.rs:272-275` récrit au même commit (faux avant comme après) ; ajouté aux fichiers touchés
    (C-32-12-26).
  - **R7 (LOW)** — `admin-restore.api.ts:11` ; `post_restore.rs` 250-251 et 258-279.
  - **R8 (LOW)** — AC20 (v) : `tables\_absent` à l'énumération d'audit (l. 2273) ; AC20 (vi) : la promesse de la
    l. 2370 nuancée (sauvegarde trop ancienne), sans version plancher.
  Passe suivante : **P3 complète, Sonnet**, lentilles R et F (`3212-validate-prompt-p3.md`) — la P2 a rendu des MEDIUM,
  et la remédiation a touché des AC (AC2, AC6, AC7, AC9, AC12, AC16, AC17, AC20). Fiche toujours `ready-for-dev`
  **sous réserve de Q4**.
- 2026-10-24 — **Validation P3** (Sonnet 5.5, deux lentilles en contexte frais ; prompt `3212-validate-prompt-p3.md` ;
  rapports `kesh-gate-logs/3212-validate-p3-{R,F}.md`) : lentille R **0/0/2/7**, lentille F **0/0/0/9**
  (CRITICAL/HIGH/MEDIUM/LOW). Convergences : **R1 = F1** (MEDIUM chez R, LOW chez F : retenu MEDIUM — la commande
  manquait un membre de la classe), **R2 = F6** (idem, retenu MEDIUM), R3 = F2, R4 = F3, R6 = F7, R9 = F8 ; R7 et F4
  portent sur l'AC11 sous deux angles, R8 est un cas de F9. Aucun finding réfuté ; chaque site revérifié au code par
  l'agent de remédiation, et **les six commandes de l'AC17 exécutées telles qu'écrites** (lecture seule, sortie
  standard) : G1 **51**, G2 **7**, G3 **36**, G4 **10**, G5 **18**, valeur **5** — nombres écrits à l'AC17. Sort (dix
  entrées) :
  - **R1 = F1 (MEDIUM)** — `-i` sur les cinq familles (G2 compris, par hygiène : 7 dans les deux formes) ; portée `S`
    étendue à `scripts` et `.config` (F9 : `prepare-release.sh:128`) ; G3 gagne `fenêtre refermée` ; commandes écrites
    avec leurs exclusions, telles qu'à exécuter ; `docs/migrations-idempotence-audit.md:91` désormais rendue par G3 ;
    les **14** homonymes nommés un à un aux Dev Notes (C-32-12-30). Les « 29 / 10 / 18 » de la P2 étaient ceux d'une
    commande avec `-i` que la fiche n'écrivait pas — relevé laissé tel quel dans l'entrée P2.
  - **R2 = F6 (MEDIUM)** — *Dérogation règle de splitting* récrite : signal levé, **recyclage partiel** reconnu (R1 = F1
    de la P2 né de la remédiation P1, R1 de la P3 né de la remédiation P2, la classe des textes devenus faux revenue à
    chaque passe) ; non-découpage motivé autrement (recyclage circonscrit à une classe, découpage qui séparerait la
    règle de ses textes, huit modules liés par une règle) et coupe de repli proposée ; décision soumise à Guy avec Q4
    (C-32-12-36 ; C-32-12-29 annoté).
  - **R3 = F2 (LOW)** — AC9, T3, tableau des sites et inventaire : l. 257-279, 272-274, relevés datés 257-266 et 276-279.
  - **R4 = F3 (LOW)** — AC1 : `created_application_table -> Option<&'static str>`, `table_creators()` (toutes les
    créatrices), `table_creating_migrations()` en prend le minimum ; le test (i) lit `table_creators()` ; M8 renommée
    (C-32-12-31 ; C-32-12-23 annoté).
  - **R5 (LOW)** — AC7 (a) : `products` injectée après **toute** autre table.
  - **R6 = F7 (LOW)** — AC2 : (iii) dite impliquée par (iv) ; M3 une seule condition (C-32-12-33) ; AC10 porté par T2.
  - **R7 + F4 (LOW)** — AC11 étendu aux migrations qui écrivent dans une table postérieure au plancher créée par une
    migration antérieure ; extrait sous `post_restore/` exigé par `registry_sql_contains_no_ddl` (l. 1355),
    cardinalité par `extract_carries_every_write_statement_of_its_source_migration` (l. 1307) ; *Contrat* point 2 et
    M7 alignés (C-32-12-32).
  - **R8 + F9 (LOW)** — triés à l'inventaire : `post_restore.rs:473-474` (générique de parc), `:1095-1096` (récrit),
    `:930` et `:1084` (appels remplacés), `:433` (entrée confiée à la 32-5), `admin_full_import_e2e.rs:1091, 1174,
    1221, 1314, 1386, 1500` (relevés datés), `scripts/prepare-release.sh:128` (générique).
  - **R9 = F8 (LOW)** — AC3 : ordre défini (version, puis rang dans l'inventaire) et testé ; table présente sans
    migration connue sans contrainte de suffixe, testée ; `Inconnue(String)`, `Exigee` et `HorsOrdre` en `&'static str`
    (C-32-12-34).
  - **F5 (LOW)** — AC20 (ii) sans exemple MCP ; l'exemple passe au *Contrat* point 3, pour la story qui crée la
    première table (C-32-12-35).
  **Tendance des MEDIUM distincts** : P1 **3** → P2 **3** → P3 **2**, aucun HIGH depuis la P1. Passe suivante :
  **P4 ciblée, Haiku**, une lentille (regression hunter) sur le seul diff de cette remédiation, qui exécute les
  commandes de l'AC17 telles qu'écrites (`3212-validate-prompt-p4.md`) — la P3 a rendu deux MEDIUM, la boucle ne se
  clôt pas (`CLAUDE.md`, décision Q3 (c)). Fiche toujours `ready-for-dev` **sous réserve de Q4** (et de la décision de
  découpage, *Dérogation*).
- 2026-10-24 — **Validation P4** (Haiku 5.5, passe **ciblée**, une lentille — regression hunter — en contexte frais, sur
  le seul diff `git diff 8a5a720a 7d7ade33` ; prompt `3212-validate-prompt-p4.md` ; rapport
  `kesh-gate-logs/3212-validate-p4.md`) : **0/0/2/1** (CRITICAL/HIGH/MEDIUM/LOW). Axe 1 : les six commandes de l'AC17
  exécutées telles qu'écrites rendent **51 / 7 / 36 / 10 / 18 / 5**, égaux aux nombres de l'AC17 ; 14 homonymes
  recomptés ; aucune ligne hors du tableau. Axes 2 à 4 sans écart. Non exercés, déclarés par la lentille : rapports P1
  et P2 lus par leurs seuls bilans, mutations non rejouées. Sort :
  - **P4-1 (MEDIUM)** — C-32-12-36 invoquait D5 pour ne pas découper en en omettant la condition qui découpe ; prémisse
    vérifiée (`CLAUDE.md` § *Règle de splitting préventif*, amendement D5 : « Ce qui découpe, c'est le recyclage … et
    la dispersion dans plus de cinq modules »). Résolu par le **découpage** (entrée suivante) : la dérogation est
    retirée.
  - **P4-2 (MEDIUM)** — l'argument (a) de la dérogation (« aucune passe n'a trouvé deux fois un défaut de la règle, de
    la carte… ») est contredit par les rapports ; prémisse vérifiée sur les titres des rapports P1 à P3 (AC2 (iii) :
    P1 F2 et R3, P2 F4 et R6, P3 R6 et F7 ; carte de l'AC1 : P1 F5, P3 R4 et F3). Résolu par le découpage.
  - **P4-3 (LOW)** — **écarts de protocole des deux lentilles de la P3**, déclarés par leurs rapports et absents de
    l'entrée « Validation P3 » : la lentille R a écrit `/tmp/x` par une redirection (`3212-validate-p3-R.md` l. 7-8),
    la lentille F a redirigé une sortie vers `…/scratchpad/ct.txt` (`3212-validate-p3-F.md` l. 9-13) — contre
    l'interdit des redirections d'écriture (`docs/agents-orchestration.md` § 2.1). Fichiers hors dépôt, **déjà
    supprimés par l'orchestrateur** ; consigné ici, l'entrée P3 n'est pas réécrite. Le prompt P4 nommait déjà
    l'interdit ; la P4 n'a rien écrit hors de son rapport.
  La P4 rend deux MEDIUM : la boucle ne se clôt pas.
- 2026-10-24 — **Découpage** (orchestrateur ; agent de fiche Opus 5.5 ; choix **C-32-12-37**, C-32-12-36 annoté) : la
  dérogation est retirée et la fiche découpée selon sa coupe de repli — **32-12a** (`32-12a-regle-inventaire-et-textes` :
  AC1-AC7, AC9-AC11, AC15-AC17, AC18a, AC19a = M1-M4, M6-M8 ; cinq modules ; 🗄️ `kesh-db`) et **32-12b**
  (`32-12b-bilan-ecran-manuel` : AC8, AC12-AC14, AC20, AC21, AC18b, AC19b = M5, M9 ; trois modules). Motif : recyclage
  partiel de la classe « textes devenus faux » et dispersion sur huit modules, les deux conditions de D5 (P4-1, P4-2).
  Ordre : 32-12a, puis 32-12b, les deux avant la 32-5. Remaniés : en-tête (deux clés, tableau, encadré Q4 : la 32-12a
  tombe entière si Guy accepte la limite), clé portée par chaque AC, AC5 (`tables_absent` lu par le `Debug` manuel en
  attendant l'AC12 — sans quoi `clippy -D warnings` refuserait un champ jamais lu ; constaté au code :
  `import.rs:97-106`, `Debug` écrit à la main), AC7, AC8, AC10, AC17 (sites du manuel : sort *Récrit par la 32-12b*,
  non rendus faux par la 32-12a ; greps relancés par la 32-12b), AC18 en AC18a/AC18b (E2E complet dans les deux :
  `backup-restore-new-install.spec.ts` emprunte l'import), AC19 réparti, AC21 (une seule entrée, à la 32-12b), tâches
  (T4, T6, T7 scindées), fichiers touchés et modules par clé, *Découpage* à la place de *Dérogation*, *Risques*, *Hors
  périmètre*, Dev Agent Record par clé. `sprint-status.yaml` : `32-12-import-sauvegarde-anterieure` à `split`, deux
  clés neuves à `ready-for-dev` sous réserve de Q4. Décision **soumise à Guy avec Q4**. Passe suivante : **P5 ciblée,
  Sonnet**, une lentille sur le seul diff du découpage (`3212-validate-prompt-p5.md`).
- 2026-10-24 — **Validation P5** (Sonnet, passe ciblée sur le découpage `b427a0a8` ; rapport hors dépôt
  `3212-validate-p5.md`) : **0/0/0/7** — **validation terminée**. Tendance des MEDIUM distincts : P1 3 → P2 3 → P3 2 →
  P4 2 → P5 0. LOW consignés, **à appliquer au développement** (orchestrateur) : P5-1 (l'AC17 est aussi relancé par la
  32-12b) ; P5-2 (AC12 et AC14 s'appuient sur `tables_absent` (AC5) et la fixture (AC15) de la 32-12a) ; P5-3 (« AC16-AC17
  (F) ») ; P5-4 (T4b) ; **P5-5 : la 32-12a consigne au Dev Agent Record les nombres que rendent les commandes de l'AC17 et
  le tableau des sites après ses réécritures — la 32-12b s'y mesure, non aux nombres d'avant** ; P5-6 (axes de l'entrée
  P4) ; P5-7 (le fichier d'epic, l. 214 et 318, et C-32-0-13 disent encore une story unique : actualisés à la fusion).
- 2026-10-24 — **Développement de la 32-12a** (Opus 5.5, worktree `kesh-32-12a`), **sous réserve de Q4** : fixture 0.15.0 au
  premier commit (`642c88f9`), règle d'inventaire, plancher, import, fenêtre P7, AC11 et textes (`c70b80e6`) ; gate complet
  3996/3996, E2E 315 / 7 (KF-029) / 17, mutations M1-M4, M7, M8 tuées et M6 équivalente ; détail au Dev Agent Record
  § 32-12a ; clé à `review`. Passe suivante : revue de code P1, Sonnet, lentilles B, E, A (`3212a-review-prompt-p1.md`).

[#722]: https://github.com/guycorbaz/kesh/issues/722
