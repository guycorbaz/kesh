# Story 25.1a : Rendre la piste de contrôle inaltérable — et cesser de promettre ce qui est faux

Status: ready-for-dev

⚠️ **Issue du SPLIT de la 25-1**, décidé à la passe 1 de validation (2026-09-10). La story
d'origine portait **quatre volets hétérogènes et 14 AC** ; ses seuls trous d'alimentation touchent
**six modules**, ce qui déclenche la § *Règle de splitting préventif* du `CLAUDE.md`. Découpage :

| | objet |
|---|---|
| **25-1a** *(celle-ci)* | fermer les chemins d'effacement, et corriger ce que quatre documents publiés promettent |
| **25-1b** | combler les six trous d'alimentation — rollout mécanique |
| **25-1c** | rendre la piste consultable — route et écran |

⛔ **L'ordre est imposé et il porte un raisonnement** : rendre la piste *lisible* (25-1c) avant de
la rendre *inaltérable* publierait un écran qui donne à voir une trace qu'un `reset` dément.

## Story

**En tant que** responsable de comptes soumis à la conservation (CO art. 957-964, OLICo art. 9),
**je veux** que le journal d'audit soit réellement inaltérable — et que rien de publié n'affirme
plus qu'il l'est déjà,
**afin que** les corrections rendues *apparentes* par la contre-passation et le gel soient
**imputables**.

**Couvre** : [#376], [#377] — III.3 de l'audit. *([#378] → 25-1c, [#379] → 25-1b.)*

## Acceptance Criteria

### Volet A — les TROIS chemins d'effacement

1. **L'import d'une sauvegarde ne détruit plus la piste.** ⛔ **Le geste évident est MORTEL et la
   passe 1 l'a démontré** : sortir `audit_log` de `TABLES_TO_TRUNCATE` rendrait **tout backup
   existant inimportable**. `export.rs:55` produit les backups en itérant cette constante — tous en
   portent une entrée — et `import.rs:129` rejette en 400 tout manifeste contenant une table « hors
   inventaire applicatif ». ⇒ **`audit_log` RESTE dans la constante** ; c'est son **traitement au
   restore** qui change.
2. ⛔ **Et « conserver la piste de l'instance importatrice » est un piège symétrique.** Le restore
   remplace `users` sous `FOREIGN_KEY_CHECKS = 0` (`backup.rs:400`), puis les rétablit — sans
   revalider les lignes existantes. Une piste conservée référencerait des `user_id` disparus, **ou
   pire, réattribués par auto-incrément à une autre personne**. Le test
   `admin_full_import_e2e.rs:330-346` encode déjà ce piège (« l'audit import porte `user_id` =
   MIN(admin) **source** […] le caller B n'existe plus → aurait violé la FK »). ⇒ **Trancher, et
   écrire le mécanisme AVANT de coder** : conservation avec remappage explicite des acteurs, ou
   fusion avec dédoublonnage des `id`. *Aucune des deux n'est gratuite ; ce qui est interdit, c'est
   de ne pas choisir.*
3. **`reset_demo` ne supprime plus la piste sans un droit** — `kesh-seed/src/lib.rs:250`,
   `DELETE FROM audit_log` non scopé.
4. **La route `/api/v1/onboarding/reset` exige le rôle administrateur.** Elle est montée dans
   `authenticated_routes` (`lib.rs:773`), **sans `require_admin_role`** — vérifié à la passe 1.
   ⚠️ Ses gardes actuelles sont des **états** (`step_completed >= 7`, `!is_demo && > 2`, le drapeau
   `KESH_PRODUCTION_RESET`), pas des droits. *Un état se contourne en amenant le système dans
   l'état voulu ; un droit, non.*
5. ⛔ **LE TROISIÈME CHEMIN, que la spec d'origine ignorait** : `/api/v1/_test/seed` et
   `/api/v1/_test/reset` appellent `truncate_all` (`test_endpoints.rs:172`), qui réutilise
   `TABLES_TO_TRUNCATE` (`test_fixtures.rs:327`) — donc `audit_log`. Monté dès
   `KESH_TEST_MODE=true`, **sans rôle requis**. Le dommage est nul en pratique (jamais activé en
   déploiement réel, refus de démarrer hors loopback) — mais **il doit être nommé** : sans cela,
   l'AC 9 refabriquerait le mensonge que [#359] a dû corriger.
6. **Un test négatif par chemin fermé, éprouvé PAR MUTATION** : la garde retirée, le test rougit.

### Volet B — cesser de promettre ce qui est faux

7. ⛔ **QUATRE documents publiés affirment l'inaltérabilité, et ils sont faux AUJOURD'HUI.** La
   vague 0 a corrigé l'en-tête du module ([#359]) et **laissé les quatre** :

   | site | ce qu'il dit | statut |
   |---|---|---|
   | `user-manual.tex:1597` | « **aucune entrée ne peut être modifiée ou supprimée** » | **faux** |
   | `marketing-brochure.tex:139` | « *audit-trail immutable* […] **garantit l'intégrité légale** au sens de l'OLICo Art. 9 » | **faux**, et publié en marketing |
   | `admin-manual.tex:1951` | « Art. 3 : intégrité → **garanti par `audit_log` insert-only** » | **faux**, affirmation de conformité |
   | `admin-manual.tex:1785` | « aucune **route API** ne permet de modifier ou supprimer **une entrée** » | exact au mot près, **trompeur en substance** — une route efface la table entière |

   ⇒ Une fois les chemins fermés, ces phrases redeviennent vraies **sauf la quatrième**, dont la
   formulation reste à corriger : elle décrit une garantie plus étroite que celle qu'on donne.
8. **Le manuel utilisateur `:496-503`** (« Ce qui manque encore… ») et **`:1597-1603`** sont relus
   ensemble : le second annonce une page de consultation « prévue pour une version ultérieure » —
   c'est la **25-1c**, pas celle-ci. Ne pas l'annoncer livrée ici.
9. **L'en-tête de `audit_log.rs:1-21` est réécrit**, et il **nomme le chemin de test** (AC 5).
   ⚠️ Il dit aujourd'hui l'inverse — « les entrées NE sont PAS inamovibles en pratique ». *Une fois
   les chemins fermés, cette phrase devient fausse à son tour, dans l'autre sens.*
10. **PDF régénérés** (`make fr`) et commités.

## Tasks / Subtasks

- [ ] **T1 — Trancher le mécanisme du restore** (AC 1, 2) : lire `backup.rs:382-460`,
      `import.rs:112-140`, et les tests `admin_full_import_e2e.rs:330-346` /
      `admin_backup_e2e.rs:263-277` qui **encodent le comportement actuel**. Écrire la décision et
      son motif dans les Dev Notes **avant** de coder.
- [ ] **T2 — Implémenter** (AC 1, 2) + tests, dont un prouvant qu'un backup **existant** reste
      importable et que l'export porte toujours `audit_log`.
- [ ] **T3 — Fermer `reset_demo`** (AC 3, 4).
- [ ] **T4 — Nommer ou fermer le chemin de test** (AC 5).
- [ ] **T5 — Mutation** (AC 6) sur chaque garde neuve.
- [ ] **T6 — Les quatre documents publiés + l'en-tête du module** (AC 7, 8, 9), PDF régénérés (AC 10).
- [ ] **T7 — Gate complet**, base remise à zéro. ⛔ `kesh-db` touché : **ciblage interdit**.

## Dev Notes

### Ce que la passe 1 de validation a établi, et qui n'était pas dans la spec d'origine

⛔ **`TABLES_TO_TRUNCATE` a SIX usages, pas deux** — c'est le fait central de cette story :

| site | usage |
|---|---|
| `backup.rs:400-408` | le `DELETE`+`INSERT` du restore, sous `FOREIGN_KEY_CHECKS=0` |
| `export.rs:55` | **produire** l'export NDJSON |
| `export.rs:96` | `table_count` du **manifeste** |
| `import.rs:118` | couverture — chaque table attendue doit figurer au manifeste |
| `import.rs:129` | ⛔ **réciproque** — toute table du manifeste **hors** de la constante ⇒ **400** |
| `test_fixtures.rs:327` | `truncate_all` des tests, et de `/api/v1/_test/*` |

**La FK est le nœud.** `audit_log.user_id → users(id) ON DELETE RESTRICT`
(`20260413000001_audit_log.sql`). Le restore remplace `users` ; toute ligne d'audit conservée
pointe alors dans le vide — **sans qu'aucune erreur ne remonte**, puisque `FOREIGN_KEY_CHECKS=1`
ne revalide pas l'existant. *La piste mentirait silencieusement sur qui a fait quoi : exactement le
défaut que la story existe pour fermer.*

**Le patron de traçage a une réserve que la spec d'origine ignorait.** `NewAuditLogEntry` a **trois**
constructeurs (`user`, `for_actor`, `api_key`) et un trait `AuditActor::from_current_user`
(`kesh-api/src/audit.rs`) qui choisit selon `user.api_key_id`. Le patron de `lock_books` utilise
`::user(user_id, …)` avec un `i64` brut — **il perd l'attribution par clé API**. C'est sûr sur les
routes `admin_routes` (PAT exclus par `require_not_pat`), et **`lock_books` lui-même ne l'est
pas** : sa route vit dans `comptable_routes`, sans ce garde. ⇒ **dette à tracer séparément**, hors
périmètre ici. *Relevé par la lentille Sonnet, S3.*

**La table n'a pas de `company_id`** — elle est **globale** dans une application multi-société.
Sans conséquence pour cette story ; **bloquant pour la 25-1c**, qui doit trancher.

### Ce que la story ne fait pas

- **Les trous d'alimentation** → 25-1b. Six modules, pas deux : `users` (4 opérations),
  `contact_persons` (3), `imported_supplier_invoices` (2), `profile`, `setup`, `companies::update`.
  ⚠️ *Le traçage se fait au **repository**, pas à la route : un `grep` au niveau du fichier de route
  conclut à un trou là où il n'y en a pas, et l'inverse.*
- **La route et l'écran de consultation** → 25-1c.
- **Aucun trigger SGBD ni `REVOKE`** : rendrait l'import et les `#[sqlx::test]` inopérants.
- **La dette d'attribution PAT de `lock_books`** — à tracer.

### References

- `audit-experts-2026-08-26.md` § III.3 · `epic-25-vague1-suite.md` § 25-1
- Issues : [#376], [#377] · voisines : [#378] (25-1c), [#379] (25-1b), [#359] (vague 0, close)
- Sites : `kesh-db/src/backup.rs:55` et `:382-460`, `kesh-api/src/admin_backup/export.rs:55,96`,
  `import.rs:118,129`, `kesh-db/src/test_fixtures.rs:327`, `kesh-api/src/routes/test_endpoints.rs:172`,
  `kesh-seed/src/lib.rs:250`, `kesh-api/src/lib.rs:773`, `routes/onboarding.rs:257-289`,
  `kesh-db/src/repositories/audit_log.rs:1-21`, `migrations/20260413000001_audit_log.sql`
- Tests qui encodent le comportement actuel : `admin_full_import_e2e.rs:330-346`,
  `admin_backup_e2e.rs:263-277`

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
