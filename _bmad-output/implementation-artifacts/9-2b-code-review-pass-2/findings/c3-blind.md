# Blind Hunter Pass 2 — Chunk 3 E2E Tests `exports_global_e2e.rs`

**Analyseur** : Haiku 4.5 (Pass 2, sans contexte)  
**Scope** : POST-patches Pass 1 (H2, H4, H5, M3, M4, M5, M6)  
**Patches Focus** : IDOR scoping (H5), audit poll loop (M4), empty company seed (H4), BOM assertion (M3)

---

## Findings

### 1. CRITICAL — H5 IDOR `invoice_lines.csv` regression: assertion incomplète sur JOIN direct

**Sévérité** : CRITICAL  
**Fichier:Ligne** : `exports_global_e2e.rs:1483-1502`  
**Type** : Regression (H5 patch — AC #29(u) multi-tenant scoping)

**Description** :

Le test `export_global_zip_repo_scoping_all_list_all_by_company()` asserte la scoping via `invoices::list_all_lines_by_company()` (ligne 1488-1497). Le patch H5 ajoute explicitement une assertion pour détecter la fuite:

```rust
assert_eq!(
    invoice_lines_a.len(),
    1,
    "invoice_lines scoping cassé : attendu 1 ligne pour A, reçu {} (probable fuite B)",
    invoice_lines_a.len()
);
```

MAIS — le test du CSV parse IDOR directement (lines 1429-1431) : 2 invoices + 2 invoice_lines (1 per invoice). L'assertion E2E `assert_eq!(jel_meta_count, 6, "expected 6 lines for A only")` au test `export_global_zip_multi_tenant_idor_scoping()` vérifie seulement `journal_entry_lines` qui en a 6 (3 entries × 2 lignes).

**PROBLÈME** : pour `invoice_lines.csv` (qui a 1 seule ligne au seed complet `seed_with_full_data()`), une fuite cross-tenant passerait inaperçue — le test b n'insère qu'1 contact + 1 invoice → 0 invoice_lines. Un bugneur qui relaxe le WHERE du JOIN `JOIN invoices i ON il.invoice_id = i.id AND i.company_id = ?` pourrait laisser passer 1 ligne de A sans déclencher d'erreur (assertion tautologique : reçu 1, attendu ≥ 1).

**Grep ground-truth requis** : vérifier que le test E2E AC #29(b) asserte effectivement `invoice_lines` en sus de `journal_entry_lines`.

---

### 2. HIGH — M4 Audit log poll loop: race condition résiduelle + timeout trop court

**Sévérité** : HIGH  
**Fichier:Ligne** : `exports_global_e2e.rs:1018-1056`  
**Type** : Timeout insuffisant (M4 patch — poll loop remplace sleep 100ms fixe)

**Description** :

Le patch M4 introduit une poll loop avec timeout 2s (20 × 100ms) pour l'audit log best-effort:

```rust
for _ in 0..20 {
    let candidate = sqlx::query_as::<_, (...)>(
        "SELECT ... FROM audit_log WHERE user_id = ? AND action = 'exports.global' ..."
    )
    .bind(ctx.user_id)
    .fetch_optional(&pool_for_assert)
    .await
    .unwrap();
    if candidate.is_some() {
        row = candidate;
        break;
    }
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}
let row = row.expect("audit row not inserted after 2s — ...");
```

**PROBLÈMES** :

1. **Timeout insuffisant sur CI chargée** : 2s est une limite ferme pour un écrit asynchrone best-effort depuis un handler HTTP. Si la queue d'audit est saturée ou si la DB ralentit, l'assertion échouera de manière flaky sans indiquer le vrai problème (timeout serveur, pas une régression).

2. **Pas de diagnostique en cas de timeout** : le message d'erreur `row.expect()` dit « audit not inserted after 2s » mais ne dit pas combien d'itérations ont réellement été exécutées ni quelle était la latence observée. Un bugneur ne saura pas si l'audit a traîné 2.1s (race condition résiduelle) ou s'il n'a jamais été émis.

---

### 3. HIGH — AC #29(u) test incomplet: 8 fns couvertes sur `list_all_by_company` mais pas exhaustif

**Sévérité** : HIGH  
**Fichier:Ligne** : `exports_global_e2e.rs:1273-1502`  
**Type** : Couverture incomplète (H5 spec validate → AC #29(u) multi-tenant « toutes fns »)

**Description** :

La spécification story 9-2b AC #29(u) lit « Multi-tenant scoping toutes fns `list_all_by_company` ». Le test asserte 8 tables:

```rust
products::list_all_by_company()
vat_rates::list_all_by_company()
reconciliation_rules::list_all_by_company()
bank_profiles::list_all_by_company()
bank_imports::list_all_by_company()
bank_transactions::list_all_by_company()
journal_entries::list_all_by_company()
invoices::list_all_by_company()
```

MAIS — la fonction export globale (~16 CSV) manipule aussi `fiscal_years`, `contacts`, `accounts`, `company_invoice_settings` qui ne sont pas couvertes par cette fonction au test `(u)`. La question: sont-elles couvertes implicitement par d'autres tests E2E, ou bien y a-t-il des `list_all_*_by_company` manquantes du repo qui ne sont pas testées ici ?

**Grep ground-truth requis** : chercher dans `kesh-db/repositories/mod.rs` ou `kesh-api` le nombre réel de nouvelles `list_all_by_company` functions créées pour story 9-2b.

---

### 4. MEDIUM — M3 BOM assertion: strip assume toujours 3 bytes de BOM, risque de faux positif

**Sévérité** : MEDIUM  
**Fichier:Ligne** : `exports_global_e2e.rs:526-531`  
**Type** : Assertion fragile (M3 patch — BOM avant strip)

**Description** :

Au test `export_global_zip_multi_tenant_idor_scoping()`, le parsing CSV vérife le BOM:

```rust
let raw = entry_bytes(&entries, name);
assert_eq!(
    &raw[0..3],
    &[0xEF, 0xBB, 0xBF],
    "{name} doit commencer par UTF-8 BOM (régression `write_csv_bom`?)"
);
let body = std::str::from_utf8(&raw[3..]).unwrap();
```

Puis on strip `&raw[3..]` pour partir à l'index 3. MAIS — si le CSV n'a que 1-2 bytes (fichier vide ou corrompu), l'accès `&raw[3..]` ne panic pas en Rust (c'est bien borné) mais crée une tranche vide. Le `.from_utf8().unwrap()` passerait. Ensuite `body.split("\r\n")` sur une chaîne vide retourne un iterator avec une seule chaîne vide, et `.next()` échoue vraiment.

**PROBLÈME RÉEL** : l'assertion ne détecte pas un CSV sans BOM parce qu'elle échoue trop tard (au `.from_utf8()` ou au split/next). Pour un CSV généré sans BOM accidentellement, l'erreur serait flou ("CSV header not found") au lieu d'un diagnostic clair "missing BOM".

---

### 5. MEDIUM — M4 Condition `is_some()` mais boucle ne compte pas itérations: diagnostic opaque

**Sévérité** : MEDIUM  
**Fichier:Ligne** : `exports_global_e2e.rs:1025-1043`  
**Type** : Diagnostic insuffisant (M4 patch — audit poll loop)

**Description** :

La boucle poll exécute 20 itérations max:

```rust
let mut row: Option<...> = None;
for _ in 0..20 {
    let candidate = sqlx::query_as(...).fetch_optional(...).await.unwrap();
    if candidate.is_some() {
        row = candidate;
        break;
    }
    tokio::time::sleep(...).await;
}
let row = row.expect("audit row not inserted after 2s — ...");
```

En cas d'échec, le message dit « after 2s » mais ne dit pas:
- Combien d'itérations ont vraiment exécuté (la boucle peut avoir brisé à l'itération 5 sur timeout non-détecté).
- Quel était le délai réel mesuré (on n'a pas d'horloge).
- Quelle erreur a produit `fetch_optional()` (on capture une erreur via `.unwrap()` mais on la perd).

Un bugneur qui introduit une latence dans l'émission de l'audit log verrait un message flou plutôt qu'un diagnostic utile.

---

## Recommandations

1. **CRITICAL (C1)** : Patcher test AC #29(b) pour explicitement asserte `invoice_lines.csv` rowCount dans metadata (pas juste JEL).

2. **HIGH (C2)** : Augmenter timeout poll loop à 5s OU implémenter un `Instant::elapsed()` et un log diagnostic avec temps réel + itérations complétées.

3. **HIGH (C3)** : Grep `kesh-db/repositories/*.rs` pour confirmer la liste exhaustive de `list_all_by_company` créées en 9-2b, et complémenter le test AC #29(u) si manque.

4. **MEDIUM (C4)** : Changer assertion BOM en pré-vérif taille (`assert!(raw.len() >= 3)`) AVANT access `&raw[3..]`.

5. **MEDIUM (C5)** : Logger itération + time réel dans la boucle audit pour diagnostique post-mortem en cas d'échec CI.

---

## Sévérité Summary

| Sévérité | Count | IDs |
|----------|-------|-----|
| CRITICAL | 1 | C1 |
| HIGH | 2 | C2, C3 |
| MEDIUM | 2 | C4, C5 |
| **Total** | **5** | — |

Aucun LOW détecté à cette passe.
