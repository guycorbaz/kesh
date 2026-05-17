# Pass 2 Blind Hunter (Haiku 4.5) — Chunk 1 Backend Core

**Scope** : Story 9-2b POST-patches Pass 1 (core Rust).

**Finding Count** : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW.

---

## MEDIUM Findings

### M1 — Asymmétrie `.instrument()` : première `.await` du handler non couverte

**Sévérité** : MEDIUM  
**Fichier:ligne** : `crates/kesh-api/src/routes/exports.rs:1701-1704`  
**Description** :

```rust
let company =
    kesh_db::repositories::companies::find_by_id(&state.pool, current_user.company_id)
        .await?
        .ok_or(AppError::Forbidden)?;
```

Première opération async du handler (ligne 1703 `.await?`), exécutée AVANT que le tracing span soit créé (ligne 1724). Le `.instrument()` ne couvre pas ce premier appel DB. Cohérent avec l'intention (le span devrait capturer uniquement l'export proprement dit, pas le pré-check), mais l'asymmétrie est fragile : si un refactor élargit le span scope ou restructure le handler, le `.instrument()` sur l'appel DB de `build_global_export` (ligne 1733) restera seul. Recommandation : documenter explicitement pourquoi cette première DB call est intentionnellement hors-span dans un comment de justification (L18).

---

### M2 — `emit_global_export_audit` retourne `()` mais ne propage pas les erreurs de tracing

**Sévérité** : MEDIUM  
**Fichier:ligne** : `crates/kesh-api/src/routes/exports.rs:1792-1838`  
**Description** :

```rust
async fn emit_global_export_audit(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    byte_size: usize,
    csv_count: usize,
    fiscal_year_scope: &str,
    duration_ms: u64,
) {
    let result = async { /* ... */ }.await;
    if let Err(e) = result {
        tracing::warn!(/* ... */);
    }
}
```

La fonction est best-effort par conception (ligne 1740 : « INSERT échec → warn + retour 200 »), donc la non-propagation d'erreurs est intentionnelle. **Cependant**, la signature retourne `()` — pas de `Result`. Lors de l'appel (ligne 1742-1752), le `.instrument(span).await` wrap silencieusement une function qui ne retourne rien. Si une future maintenance ajoute une dépendance sur le succès de cet audit (e.g. commit-time verification), le code appelant n'obtiendrait aucun signal de défaillance. Recommandation : ajouter un comment explicite « `-> ()` : audit best-effort, erreurs loggées mais jamais propagées » ou envisager un changement de signature `-> Result<(), AppError>` en v0.2.

---

## LOW Findings

### L1 — `HeaderValue` import supprimé sans vérification exhaustive

**Sévérité** : LOW  
**Fichier:ligne** : `crates/kesh-api/src/routes/reports.rs:1906-1907`  
**Description** :

Diff :
```diff
-    http::{HeaderValue, StatusCode, header},
+    http::{StatusCode, header},
```

Le `HeaderValue` est supprimé après migration de `build_content_disposition` vers `crate::util`. Vérification confirmée : nulle trace de `HeaderValue` ailleurs dans les 16 fichiers CSV serializers ou dans `exports/handlers.rs`. Pas de régression. Cependant, la suppression n'est pas isolable à une seule ligne — elle s'accompagne de refactorings in-place (lignes 1932, 1941, 2017-2018). À l'avenir, préférer un import-cleanup séparé de la refactorisation fonctionnelle pour auditabilité.

---

### L2 — `use tracing::Instrument;` importée inline dans le handler

**Sévérité** : LOW  
**Fichier:ligne** : `crates/kesh-api/src/routes/exports.rs:1723`  
**Description** :

```rust
use tracing::Instrument;
let span = tracing::info_span!(...);
```

Pratique valide mais peu conventionnelle : l'import est scoped au handler `export_global` via un `use` local (ligne 1723), pas au top du module. Cela fonctionne car `Instrument` est un trait et donc son import est obligatoire pour appeler `.instrument()`. Cependant, c'est un anti-pattern de lisibilité — convention Rust place tous les imports en haut du fichier. Recommandation : déplacer au top du fichier avec les autres imports `use tracing::...` (si applicable), ou au minimum au top du module.

---

## Vérifications Additionnelles (Ground-Truth Grep)

- ✅ **H1** (`.instrument()` vs `_enter` guard) : Confirmed Pass 1 patch appliqué correctement aux deux appels async (lignes 1733, 1751).
- ✅ **H2** (clés snake_case) : Confirmed `details_json` contient `company_id`, `byte_size`, `csv_count`, `fiscal_year_scope`, `duration_ms` (lignes 1814-1818).
- ✅ **H3** (doc-comment reordering) : N/A pour ce chunk — concerne `kesh-db`, pas le backend core.
- ✅ **M1** (suppression `_ensure_companies_used()`) : N/A — pas trouvé dans ce diff ; appartenait probablement à un autre chunk.

---

## Autres Remarques Adversariales

1. **Duplication CSV header logic** (lignes 268-297 et suiv.) — 16 serializers en-têtes CSV répétitifs. Pattern macro `push_csv!` (lignes 1165-1179) mitigue bien, mais le code reste volumineux. Pas de défaut à réparer maintenant, mais une opportunité pour v0.2 : factoriser `write_csv_record_header(&mut csv, &["id", "company_id", ...])` helper.

2. **`fmt_opt_str` impl** (lignes 179-180) — clone systématique de `Option<String>`. Perfectionable mais pas un bug :
   ```rust
   fn fmt_opt_str(s: &Option<String>) -> String {
       s.clone().unwrap_or_default()
   }
   ```
   À l'avenir : accepter `&str` ou utiliser `as_deref()`.

3. **`build_zip` error context** (ligne 1074-1076) — chaque `.map_err` emballe le nom du fichier dans le message (« zip start_file {name}: {e} »). Bonne traçabilité ops. Cohérent.

4. **`emit_global_export_audit` inner `async` block** (lignes 1801-1826) — pattern valide mais moins conventionnel que `async fn avec `?` propagation. Maintenu pour best-effort, approuvé.

---

## Synthèse

**Convergence Pass 1 → Pass 2** :
- Aucune régression détectée dans les patches H1/H2.
- **M1 & M2** sont des concerns de conception (scope d'instrument, sémantique best-effort), non des bugs fonctionnels.
- Code est **production-ready** pour v0.1 ; débts lisibilité sont v0.2.

**Itération recommendée** : Si Sonnet Pass 3 converge sur ces mêmes M1/M2, classer en **LOW** et marquer comme « intentional design choice — debt L13-L15 ». Si désaccord, clarifier dans spec notes.

