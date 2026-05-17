# Code Review — Chunk 1 Backend Core (Story 9-2b) — Blind Hunter Pass 1

**Reviewer** : Blind Hunter (adversarial, zéro contexte projet)  
**Date** : 2026-05-17  
**Scope** : `crates/kesh-api/src/{util.rs, errors.rs, lib.rs, exports/{mod,csv_tables,global,metadata}.rs, routes/{exports.rs, reports.rs (refactor), bank_imports.rs (refactor), mod.rs}}`  
**Diff** : `chunk-1-backend-core.diff` (~2495 lignes)

---

## Findings

---

### F01 — HIGH — Tracing span `_enter` avec `async` : guard dropped prématurément, span non couvre l'await

**Fichier** : `routes/exports.rs` — lignes ~1732–1734  
**Code** :
```rust
let _enter = span.enter();
let (zip_bytes, meta) = build_global_export(&state.pool, &company, locale_bcp47).await?;
```

**Problème** : `span.enter()` retourne un `Entered` guard synchrone. Dans un contexte `async`, le guard est dropped au premier point de suspension (`.await`), ce qui signifie que l'instrumentation ne couvre **pas** l'exécution asynchrone de `build_global_export`. Les champs `byte_size`, `csv_count`, `duration_ms` sont enregistrés après le retour de l'`await`, mais la span elles-même est inactive pendant tout le travail réel (les 16 queries SQL + sérialisation CSV). Les observateurs Jaeger/Honeycomb voient une span quasi-vide. Le pattern correct est `.instrument(span)` ou `#[instrument]` sur la future. Ceci n'est pas un simple cosmétique de tracing : dans un environnement multi-tenant sous charge, la corrélation span → logs ops est brisée pour cet endpoint critique.

---

### F02 — HIGH — Mémoire non bornée : accumulation de 16 `Vec<u8>` CSV en RAM + ZIP en RAM sans aucun budget

**Fichier** : `exports/global.rs` — lignes ~1162–1246 (`build_global_export`)  
**Problème** : L'orchestrateur alloue simultanément 16 `Vec<u8>` CSV (un par table), puis un ZIP in-memory via `Cursor<Vec<u8>>` (`build_zip`). Pour une PME avec 50 000 transactions bancaires, `bank_transactions.csv` seul peut dépasser 50 MB. Le total de tous les buffers simultanés (pré-ZIP + post-ZIP) peut tripler ce chiffre. Il n'existe aucun guard de taille, aucun limit de rows, aucun timeout sur les queries DB. Une requête légitime sur un tenant volumineux peut provoquer un OOM sur le serveur et affecter tous les autres tenants. La spec mentionne "export souveraineté" mais aucune borne mémoire n'est implémentée ni même documentée comme dette explicite.

---

### F03 — HIGH — `build_zip` : `zip.finish()` dans un bloc scoped, mais `cursor.into_inner()` récupère les bytes **avant** le flush complet du central directory dans certains cas zip 2.x

**Fichier** : `exports/global.rs` — lignes ~1066–1081  
**Code** :
```rust
let mut cursor = Cursor::new(Vec::<u8>::new());
{
    let mut zip = ZipWriter::new(&mut cursor);
    // ...
    zip.finish()
        .map_err(|e| AppError::GlobalExportFailed(...))?;
}
Ok(cursor.into_inner())
```

**Problème** : Le bloc `{}` scoped assure que `ZipWriter` est dropped avant `cursor.into_inner()`, ce qui est correct pour la plupart des cas. Toutefois, `zip.finish()` est appelé explicitement **et** le `ZipWriter` est dropped immédiatement après — `ZipWriter::drop` appelle aussi `finish()` si non déjà appelé. Si `finish()` retourne `Ok` mais le `Drop` tente de re-écrire (comportement dépendant de la version zip 2.x), les bytes dans le curseur peuvent être dans un état ambigu. Le problème plus immédiat : `zip.finish()` retourne `Result<W>` dans certaines versions de la crate `zip`, mais ici la valeur retournée est ignorée (`?` mappe vers `AppError` mais le `W` — qui contient l'état final du writer — est abandonné). Dans `zip 2.x`, `finish()` retourne `ZipWriter<W>` et c'est lui qui possède les derniers bytes. **Si le pattern de consommation de `finish()` est incompatible avec la version de `zip` utilisée**, le ZIP produit peut être tronqué ou corrompu sans que l'erreur soit détectée. Ce finding nécessite de vérifier le `Cargo.lock` (non fourni dans le diff) pour la version exacte.

---

### F04 — MEDIUM — `_ensure_companies_used()` : fonction dead-code avec `#[allow(dead_code)]` pour contourner un warning du compilateur — signal de design suspect

**Fichier** : `exports/global.rs` — lignes ~1269–1276  
**Code** :
```rust
#[allow(dead_code)]
fn _ensure_companies_used() {
    let _ = companies::find_by_id;
}
```

**Problème** : Ce pattern est un hack pour conserver un import inutilisé (`use kesh_db::repositories::companies`) en supprimant le warning compilateur via une fonction dead-code annotée. Ce n'est pas une pratique acceptable dans du code de production : soit l'import est utilisé, soit il est supprimé. Garder un import non-utilisé via un workaround délibéré indique que le design n'est pas stable. Si l'import est réellement prévu pour v0.2, il ne doit pas figurer dans le code mergé — utiliser un `// TODO(v0.2): ...` commentaire sans import actif. En l'état, cela crée un faux sentiment de sécurité (le compilateur ne warn pas) et pollue l'espace de noms.

---

### F05 — MEDIUM — `map_language_to_bcp47` accepte `&str` au lieu de l'enum `Language` : contrat de type affaibli sans justification réelle

**Fichier** : `util.rs` — lignes ~2349–2363  
**Commentaire dans le code** :  
> "Signature `&str` (PAS l'enum `Language`) car les sites d'appel actuels lisent `accounting_language` comme `String` via une `sqlx::query_as`"

**Problème** : Si `kesh_db::entities::company::Language` existe (elle est importée dans les tests de `metadata.rs` à la ligne ~1456), alors l'appel `company.accounting_language.as_str()` dans `exports.rs` ligne ~1716 montre que le champ est **bien** typé comme `Language`, pas comme `String`. Le commentaire justificatif est donc incorrect ou obsolète. La signature `&str` sacrifie la sûreté de type (le compilateur ne peut pas détecter l'ajout d'un variant `Language::Rm` pour le romanche suisse qui ne serait pas mappé dans le `match`) pour une "commodité" qui n'est pas réelle. C'est exactement le genre de bug silencieux qui produit `fr-CH` avec un warning log que personne ne surveille.

---

### F06 — MEDIUM — `serialize_journal_entry_lines_csv` : colonnes manquantes dans le header — `description`, `version`, `created_at`, `updated_at` absents

**Fichier** : `exports/csv_tables.rs` — lignes ~340–368  
**Header exporté** :
```
id, entry_id, account_id, line_order, debit, credit
```

**Problème** : Le CSV `journal_entry_lines` n'exporte que 6 colonnes. Si l'entité `JournalEntryLine` possède des champs additionnels (description de ligne, timestamps), ils sont silencieusement omis. Par contraste, `journal_entries` exporte `description`, `version`, `created_at`, `updated_at`. Un export de "souveraineté" (terme utilisé dans les commentaires) incomplet sur les lignes d'écriture comptable est problématique pour une reconstitution complète des données. Sans accès au schema `JournalEntryLine`, il est impossible de confirmer si des colonnes sont manquantes — mais l'asymétrie avec la table parente est un signal fort. Si les colonnes manquent réellement dans l'entité, la spec devrait le documenter explicitement.

---

### F07 — MEDIUM — `company_invoice_settings::get_or_create_default` appelé dans le chemin d'un export read-only : effet de bord en écriture non documenté + transaction implicite

**Fichier** : `exports/global.rs` — lignes ~1146–1150  
**Code** :
```rust
let cis_row = company_invoice_settings::get_or_create_default(pool, company_id)
    .await
    .map_err(map_db)?;
```

**Problème** : Un endpoint `GET` qui appelle une fonction "get_or_create" effectue potentiellement un `INSERT` en base de données. C'est une violation du principe d'idempotence des requêtes GET (RFC 7231). Si la ligne `company_invoice_settings` n'existe pas pour un tenant, chaque appel `GET /api/v1/exports/global.zip` crée silencieusement une ligne en DB. Ce comportement peut :
1. Créer des entrées fantômes dans `company_invoice_settings` non initialisées correctement (defaults potentiellement incorrects).
2. Générer des entrées `audit_log` parasites pour des creates déclenchés par des exports.
3. Interférer avec des tests ou des analyses qui comptent les lignes dans cette table.
Le pattern safe est `get_or_default_in_memory` : si absent, inclure une ligne de defaults sans écrire en DB.

---

### F08 — MEDIUM — `emit_global_export_audit` ouvre une **transaction** pour un seul INSERT alors que l'insertion directe hors-tx suffit

**Fichier** : `routes/exports.rs` — lignes ~1800–1819  
**Code** :
```rust
let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
kesh_db::repositories::audit_log::insert_in_tx(&mut tx, ...).await?;
tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
```

**Problème** : Ouvrir une transaction explicite pour un seul INSERT sur une table d'audit est inutilement coûteux (round-trip `BEGIN` + `COMMIT`). Si `audit_log::insert` possède une version sans-tx, elle devrait être utilisée. Si elle n'existe pas, un `INSERT` direct via `pool.execute()` suffit. Dans un contexte best-effort (l'erreur est ignorée avec `warn!`), la transaction n'apporte aucune garantie supplémentaire par rapport à un INSERT atomique natif InnoDB. Le seul effet est une connexion DB tenue plus longtemps sur le pool partagé.

---

### F09 — MEDIUM — `build_metadata_json` : `export_date` générée au moment de la **sérialisation du manifeste**, pas au début du pipeline — inconsistance temporelle avec `duration_ms`

**Fichier** : `exports/metadata.rs` — ligne ~1429 / `exports/global.rs` — ligne ~1242  
**Code dans `build_metadata_json`** :
```rust
export_date: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
```
**Code dans `build_global_export`** :
```rust
let start = Instant::now();
// ... 16 queries + serialize ...
let manifest_bytes = build_metadata_json(company, locale_bcp47, tables_meta)?;
// ...
let duration_ms = start.elapsed().as_millis() as u64;
```

**Problème** : `export_date` est capturée dans `build_metadata_json`, donc **après** toutes les queries SQL + sérialisations CSV. Sur un export long (plusieurs secondes), `export_date` peut différer de plusieurs secondes de l'`Instant::now()` capturé au début du pipeline. Le `metadata.json` devrait capturer le timestamp de **début** de l'export (représentatif de "quand les données ont été lues"), pas de fin de sérialisation. L'implémentation correcte passe `export_date` comme paramètre à `build_metadata_json` plutôt que de le générer en interne.

---

### F10 — MEDIUM — Aucun test d'intégration ou de contrat pour vérifier que les 16 noms de tables dans `push_csv!` correspondent exactement aux clés dans `metadata.json`

**Fichier** : `exports/global.rs` — lignes ~1181–1236  
**Problème** : Le macro `push_csv!` insère dans `tables_meta` la clé littérale passée en `$name` (ex: `"company.csv"`) et dans `files` le même nom. Si une typo ou une divergence survient entre le nom littéral et le nom réel attendu (ex: `"fiscal-years.csv"` vs `"fiscal_years.csv"`), le manifeste JSON contiendra une clé incorrecte. Il n'existe aucun test vérifiant que les 16 entrées de `tables_meta` correspondent aux 16 noms attendus (ni un test de round-trip "manifest contient exactement ces 16 clés"). L'unique test de `build_metadata_json` n'injecte que 2 tables (`accounts.csv` + `journal_entries.csv`). Un test avec les 16 noms canoniques est absent.

---

### F11 — MEDIUM — `hex_encode` utilise `format!("{b:02x}")` dans une boucle : allocations répétées (N allocs pour N bytes) au lieu d'un `write!` in-place

**Fichier** : `util.rs` — lignes ~2378–2383  
**Code** :
```rust
for b in bytes {
    s.push_str(&format!("{b:02x}"));
}
```

**Problème** : `format!("{b:02x}")` alloue une `String` temporaire pour **chaque byte**. Pour un hash SHA-256 (32 bytes → 64 chars), c'est 32 allocations temporaires. Si cette fonction est appelée 16 fois par export global (une par CSV), c'est 512 allocations pour les seuls hashes. Le pattern correct est `use std::fmt::Write; write!(&mut s, "{b:02x}").unwrap()` qui écrit directement dans le buffer pré-alloué sans allocation intermédiaire. Ce n'est pas un bug fonctionnel, mais dans un hot path d'export (potentiellement appelé fréquemment), c'est une performance inutilement dégradée pour du code trivial.

---

### F12 — LOW — `debug_assert_eq!(files.len(), 16, ...)` : assertion non active en release build — pas de protection production

**Fichier** : `exports/global.rs` — lignes ~1238–1239  
**Code** :
```rust
debug_assert_eq!(files.len(), 16, "16 CSV expected before manifest");
debug_assert_eq!(tables_meta.len(), 16, "16 TableMeta expected");
```

**Problème** : `debug_assert!` est désactivé en mode release (`--release`). Si le développeur ajoute une 17e table ou en omet une par erreur, cette assertion ne se déclenche pas en production. Pour une invariante critique (le manifeste doit contenir exactement N tables), un `assert_eq!` ou une vérification explicite avec retour `AppError::GlobalExportFailed` serait plus robuste. Le coût de deux comparaisons d'entiers en production est négligeable.

---

### F13 — LOW — `company_name` dans `metadata.json` peut contenir des caractères de contrôle (newlines, tabs) non sanitisés — log injection potentiel

**Fichier** : `exports/metadata.rs` — ligne ~1432 / `exports/global.rs` — ligne ~1029 (tracing)  
**Code** :
```rust
company_name: company.name.clone(),
```
Et dans `build_global_export`, `company_id` et le nom de company sont tracés via le span.

**Problème** : Si `company.name` contient un `\n` ou des caractères de contrôle (saisie utilisateur non sanitisée en amont), `serde_json::to_vec_pretty` les échappe correctement dans le JSON (`\n` → `\\n`). Toutefois, dans les logs tracing, si `company_name` est jamais loggé directement (ex: `tracing::info!("export for {}", company.name)`), un attaquant contrôlant son `company.name` peut injecter des fausses lignes de log. Ce n'est pas un vecteur exploitable dans le code actuel visible (le span ne logue que `company_id` numérique), mais la surface existe si un développeur ajoute un log du nom ultérieurement.

---

### F14 — LOW — `build_global_filename` non testée avec un nom de company contenant des caractères déjà slugifiables par le chemin `reports` mais qui diffèrent légèrement

**Fichier** : `routes/exports.rs` — lignes ~1844–1880 (tests)  
**Problème** : Les tests de `build_global_filename` couvrent UTF-8 de base (`Müller AG`), truncation, et fallback vide. Ils ne couvrent pas les cas limites de la fonction `slugify` spécifiques au contexte ZIP : caractères `+`, `#`, `%` dans un nom de company (rares mais légaux dans certains pays), slash `/` qui serait catastrophique dans un nom de fichier ZIP. Ces caractères sont normalement remplacés par `-` par `slugify`, mais aucun test ne le confirme explicitement pour le contexte `build_global_filename`.

---

### F15 — LOW — Test `build_zip_error_path_is_wired` est un no-op fonctionnel : il ne teste rien

**Fichier** : `exports/global.rs` — lignes ~1333–1344  
**Code** :
```rust
fn _check_signature(f: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError> {
    build_zip(f)
}
let _ = _check_signature;
```

**Problème** : Ce test ne s'exécute pas : `_check_signature` est définie mais jamais appelée (l'instruction `let _ = _check_signature` assigne la référence de fonction à `_` et la jette immédiatement). Ce n'est qu'une vérification de type à la compilation déguisée en test. Si l'objectif est vérifier que `build_zip` retourne `Result<_, AppError>`, c'est déjà garanti par le compilateur partout où `?` est utilisé. Ce test devrait soit être supprimé, soit remplacé par un vrai test fonctionnel.

---

## Résumé

| Sévérité | Nombre |
|----------|--------|
| CRITICAL | 0      |
| HIGH     | 3 (F01, F02, F03) |
| MEDIUM   | 6 (F04–F10, F11 compte comme MEDIUM par contexte hot-path) |
| LOW      | 4 (F12–F15) |

**Points d'attention prioritaires** :
1. **F01** (span async) : brisure de l'observabilité ops sur l'endpoint le plus critique de cette story.
2. **F02** (mémoire non bornée) : risque OOM réel sur des tenants avec volume de données significatif.
3. **F07** (GET avec effet de bord en écriture) : violation idempotence REST + INSERT parasite.
