# Findings — Acceptance Auditor — Chunk 1 Backend Core (Story 9-2b)

**Reviewer** : Acceptance Auditor (Sonnet 4.6)  
**Date** : 2026-05-17  
**Scope** : `exports/`, `routes/exports.rs`, `errors.rs`, `util.rs`, `lib.rs`, refactor `reports.rs` + `bank_imports.rs`  
**Méthode** : diff vs spec complète (32 ACs + 10 décisions verrouillées + 18 limitations)

---

## Findings

### MEDIUM

---

#### AA-MEDIUM-01 — AC #30(j) : test build_zip failure path est un stub compile-time, pas un test réel

**AC/Décision violée** : AC #30(j) — « `build_zip` failure path : appel `build_zip(&[(invalid_utf8_name_bytes, b"data".to_vec())])` ou simulation OOM via `Cursor` avec capacité fixée → assert `Err(AppError::GlobalExportFailed(detail))` avec `detail.contains("zip")` »

**Évidence** : `exports/global.rs:1333-1344`
```rust
fn build_zip_error_path_is_wired() {
    fn _check_signature(f: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError> {
        build_zip(f)
    }
    // ...
    let _ = _check_signature;
}
```

**Description** : Le test `build_zip_error_path_is_wired` ne provoque **aucune erreur réelle** — il compile-time-check la signature et sort. AC #30(j) exige un assert `Err(AppError::GlobalExportFailed(...))` avec `detail.contains("zip")`. Le commentaire reconnaît le problème mais ne propose pas d'implémentation alternative concrète. La spec autorisait « `Cursor` avec capacité fixée » comme implémentation alternative — ni l'un ni l'autre n'est utilisé.

---

#### AA-MEDIUM-02 — AC #24 : tracing span créé avec `span.enter()` au lieu d'un `instrument` ou d'un `.in_scope`

**AC/Décision violée** : AC #24 — span `tracing::info_span!("global_export")` avec `byte_size`, `csv_count`, `duration_ms` via pattern `tracing::field::Empty` + `span.record()` cohérent Story 9-2a T5.8

**Évidence** : `routes/exports.rs:1725-1732`
```rust
let span = tracing::info_span!(
    "global_export",
    company_id = current_user.company_id,
    byte_size = tracing::field::Empty,
    csv_count = tracing::field::Empty,
    duration_ms = tracing::field::Empty,
);
let _enter = span.enter();
```

**Description** : L'utilisation de `.enter()` dans un contexte `async` est **incorrecte et documentée comme anti-pattern** par la crate `tracing` : le guard `_enter` n'est pas `Send`, et si le future est suspendu sur un autre point `.await` (notamment `build_global_export(...).await`), le span peut être incorrectement associé à un autre thread. La spec dit « pattern cohérent Story 9-2a T5.8 » mais la correction recommandée par `tracing` est `.instrument(span)` ou `.in_scope(|| ...)` pour les sections synchrones. Ce bug peut provoquer une pollution des traces en production si l'executor multi-thread est utilisé.

---

#### AA-MEDIUM-03 — AC #23 / AC #29(m) : `details_json` utilise camelCase (`companyId`, `byteSize`) mais la spec prescrit des champs snake_case lisibles côté JSON query

**AC/Décision violée** : AC #23 — `details_json` incluant `company_id`, `byte_size`, `csv_count`, `fiscal_year_scope`, `duration_ms`. AC #29(m) — assert `details_json` JSON contient `company_id`

**Évidence** : `routes/exports.rs:1809-1815`
```rust
details_json: Some(serde_json::json!({
    "companyId": company_id,
    "byteSize": byte_size,
    "csvCount": csv_count,
    "fiscalYearScope": fiscal_year_scope,
    "durationMs": duration_ms,
})),
```

**Description** : La spec AC #23 et AC #29(m) utilisent systématiquement `company_id`, `byte_size`, `csv_count`, `fiscal_year_scope`, `duration_ms` en snake_case. L'implémentation utilise camelCase (`companyId`, `byteSize`, etc.). Le test E2E AC #29(m) (chunk 3) qui assertera `details_json` JSON contient `company_id` échouera si la clé est `companyId`. Ce n'est pas un CRITICAL car le comportement fonctionnel est correct, mais cela viole la spec textuelle et cassera le test prescrit.

---

### LOW

---

#### AA-LOW-01 — AC #30(g) : test orchestrateur `build_global_export` absent du diff Chunk 1

**AC/Décision violée** : AC #30(g) — « `build_global_export(state, company_id) -> Result<(Vec<u8>, GlobalExportMeta)>` orchestrateur — 1 test avec fixture company + 1 account + assert ZIP bytes valides + meta cohérente »

**Évidence** : Les tests de `exports/global.rs:1282-1345` couvrent uniquement `build_zip_signature_and_entries`, `build_zip_empty_input_still_valid`, `build_zip_error_path_is_wired` — aucun test pour l'orchestrateur complet `build_global_export`.

**Description** : AC #30(g) prescrit un test unit de `build_global_export` avec fixture Company + 1 account. Ce test est absent du diff. Il est possible qu'il soit prévu en E2E (chunk 3) mais la spec le prescrit explicitement comme test unit dans AC #30. La couverture unit du chemin principal de l'orchestrateur est manquante dans ce chunk.

---

#### AA-LOW-02 — AC #18 : message d'erreur ZIP packaging ne respecte pas le préfixe `"zip packaging: <detail>"`

**AC/Décision violée** : AC #18 — `AppError::GlobalExportFailed("zip packaging: <detail>")` cohérent UX-DR38

**Évidence** : `exports/global.rs:1074-1079`
```rust
zip.start_file(name.as_str(), options)
    .map_err(|e| AppError::GlobalExportFailed(format!("zip start_file {name}: {e}")))?;
zip.write_all(bytes)
    .map_err(|e| AppError::GlobalExportFailed(format!("zip write {name}: {e}")))?;
...
zip.finish()
    .map_err(|e| AppError::GlobalExportFailed(format!("zip finish: {e}")))?;
```

**Description** : La spec AC #18 prescrit le préfixe `"zip packaging: <detail>"` pour homogénéité. L'implémentation utilise `"zip start_file {name}: ..."`, `"zip write {name}: ..."`, `"zip finish: ..."` sans le préfixe `"zip packaging:"`. Ce n'est pas fonctionnellement incorrect (l'erreur est bien une `GlobalExportFailed` avec détail loggé) mais diverge du format prescrit. Impact minimal car ces messages ne sont jamais exposés au client.

---

#### AA-LOW-03 — `_ensure_companies_used()` : dead_code workaround présent dans le code livré

**AC/Décision violée** : (qualité code, pas un AC direct — CLAUDE.md DRY + propreté)

**Évidence** : `exports/global.rs:1269-1276`
```rust
#[allow(dead_code)]
fn _ensure_companies_used() {
    // L'orchestrateur reçoit la Company déjà chargée (caller fetch), donc le
    // module `companies` n'est pas utilisé dans le chemin nominal ici.
    // On garde l'import pour réutilisation future (filtrage par exercice v0.2).
    let _ = companies::find_by_id;
}
```

**Description** : La fonction `_ensure_companies_used` est un hack pour contourner `unused import` sur `companies`. Si le module `companies` est importé mais non utilisé, il suffit de retirer l'import. Garder un `#[allow(dead_code)]` avec une fn qui référence une fn inutilisée est anti-pattern et génèrera probablement un warning `clippy::dead_code` ou sera flagué par `cargo clippy -- -D warnings`. La suppression de l'import `companies` ou son remplacement par un commentaire `// kept for v0.2 ?filteryear` est la solution correcte.

---

#### AA-LOW-04 — AC #3 : `Content-Type: application/zip` utilisé mais RFC 6838 recommande `application/zip` ou `application/x-zip-compressed` — mineur

**AC/Décision violée** : AC #3 — `Content-Type: application/zip`

**Évidence** : `routes/exports.rs:1758`
```rust
.header(header::CONTENT_TYPE, "application/zip")
```

**Description** : La spec AC #3 prescrit `Content-Type: application/zip`. L'implémentation est conforme. Note informative uniquement : certains navigateurs anciens et certains middleware (proxy, CDN) reconnaissent mieux `application/octet-stream` ou `application/x-zip-compressed`. Pas un défaut — `application/zip` est le MIME type IANA officiel et la décision de la spec est correcte. Finding documenté pour complétude, pas de changement requis.

---

## Résumé

| Sévérité | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH     | 0 |
| MEDIUM   | 3 |
| LOW      | 4 |
| **Total** | **7** |

### Observations positives

L'implémentation est globalement de haute qualité et conforme à la spec :

- **AC #17/#18 `GlobalExportFailed`** : variant correctement défini, bras `IntoResponse` strictement aligné sur le pattern `PdfGenerationFailed`/`CsvGenerationFailed`, test unit `global_export_failed_maps_to_500_without_leaking_detail` couvre AC #30(i).
- **AC #6/#7 CSV format** : BOM UTF-8, délimiteur `;`, CRLF, RFC 4180 escaping, dates ISO 8601 — tous présents et testés.
- **AC #12/#13/#14/#15/#16 metadata.json** : shape camelCase, `BTreeMap` déterministe, `env!("CARGO_PKG_VERSION")` compile-time (pas de hardcode), `to_rfc3339_opts(SecondsFormat::Secs, true)` → suffixe `Z` strict, `locale_bcp47` résolu via `map_language_to_bcp47` — tous conformes.
- **AC #10 anti-IDOR** : route montée DANS `authenticated_routes` AVANT le `;`, commentaire explicatif présent.
- **AC #19 défensif** : guard `company_id <= 0 → 403 Forbidden` implémenté.
- **AC #23 audit best-effort** : `emit_global_export_audit` retourne `()`, aucun `?` qui ferait échouer le download.
- **AC #24 tracing** : pattern `tracing::field::Empty` + `span.record()` présent (mais problème async, cf. AA-MEDIUM-02).
- **Décision §hex-encoding** : `crate::util::hex_encode` promu depuis `bank_imports.rs`, pas de crate `hex` externe.
- **Décision §csv-table-serializer-location** : module `kesh-api/src/exports/`, DD-12 respecté.
- **Décision §version-source** : `env!("CARGO_PKG_VERSION")` utilisé.
- **Décision §error-variant** : `GlobalExportFailed` distinct de `CsvGenerationFailed`.

### Points bloquants pour la transition `review → done`

Aucun finding CRITICAL ou HIGH dans ce chunk. Les 3 MEDIUM sont à adresser avant merge :

1. **AA-MEDIUM-01** : test AC #30(j) `build_zip_error_path_is_wired` doit provoquer une erreur réelle ou être remplacé par un test alternatif valide.
2. **AA-MEDIUM-02** : span `tracing::info_span!` + `.enter()` dans contexte `async` est un anti-pattern documenté par `tracing` — remplacer par `.instrument(span)` ou restructurer le code.
3. **AA-MEDIUM-03** : `details_json` camelCase vs snake_case prescrit en spec — aligner avec AC #23/#29(m).
