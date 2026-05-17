# Chunk 3 — Blind Hunter Review : `exports_global_e2e.rs`

**Reviewer** : Blind Hunter (adversarial, aucun contexte projet)  
**Date** : 2026-05-17  
**Modèle** : claude-sonnet-4-6  
**Scope** : `crates/kesh-api/tests/exports_global_e2e.rs` — 21 tests `#[sqlx::test]` (a–u)

---

## Findings

---

### F1 — MEDIUM — BOM hardcodé sans garde contre encodage variable

**Fichier** : `exports_global_e2e.rs:525`

```rust
let body = std::str::from_utf8(&raw[3..]).unwrap();
```

Le BOM UTF-8 (`\xEF\xBB\xBF`) est sauté en slicing aveugle `[3..]`. Si l'implémentation change l'encodage (UTF-8 sans BOM, UTF-16-BE, latin-1) ou si le CSV commence par un header de longueur < 3 octets, le test panique avec un `unwrap()` non contrôlé qui masque l'erreur réelle dans un message générique "panicked at called `Option::unwrap()` on a `None` value". De plus, le test (b) est le *seul* qui essaie de lire les CSV comme du texte — le BOM n'est pas vérifié comme présent avant le slice. Si le BOM est absent (bug dans l'implémentation), le test consommerait silencieusement 3 octets de données réelles, corrompant la lecture de header et produisant un faux négatif total : le test passerait avec des données erronées.

**Impact** : assertion fragile, faux négatif si BOM absent. `unwrap()` masque l'erreur.

---

### F2 — MEDIUM — Séparateur `;` hardcodé, assertions CSV cassées si implémentation utilise `,`

**Fichier** : `exports_global_e2e.rs:529,535`

```rust
let cols: Vec<&str> = header.split(';').collect();
// ...
let cells: Vec<&str> = row.split(';').collect();
```

Le test (b) suppose que le séparateur CSV est `;`. Si l'implémentation produit des CSV avec `,` (standard RFC 4180), `split(';')` retourne un vecteur d'un seul élément. La recherche `position(|c| *c == "company_id")` retourne `None`, ce qui déclenche `expect("company_id column present")` et masque le vrai problème (mauvais séparateur) derrière un message trompeur. Aucun test ne vérifie explicitement quel séparateur est utilisé, ni ne s'adapte dynamiquement — c'est un couplage cassant entre tests et implémentation sans contrat explicite.

**Impact** : faux négatif complet si séparateur change. Pas de contrat de séparateur testé.

---

### F3 — MEDIUM — Tests (n) et (o) sont des no-ops sans assertions : couverture fantôme

**Fichier** : `exports_global_e2e.rs:1019-1024,1031-1033`

```rust
async fn export_global_zip_error_500_sql_wired(_pool: MySqlPool) {
    // Sentinelle "wired" — pas de scénario E2E SQL down trivial en sandbox.
}

async fn export_global_zip_error_500_zip_wired(_pool: MySqlPool) {
    // Idem (n) — couverture via tests unit ...
}
```

Ces deux tests compilent et passent toujours, inconditionnellement, même si le wiring d'erreur est complètement absent du handler. Ils ne vérifient rien. Le commentaire justifie par "couverture dans les tests unit", mais le nom du test (`_wired`) et son inclusion dans le rapport de couverture E2E créent une illusion de couverture. Si les tests unit en question sont supprimés ou renommés, ces sentinelles ne le détecteront jamais. Une sentinelle vide ne prouve pas le wiring — elle prouve seulement que le fichier compile.

**Impact** : faux positif de couverture. Le rapport "21 tests passent" inclut 2 tests qui ne testent rien.

---

### F4 — MEDIUM — Absence totale du rôle `Comptable` dans la matrice RBAC explicite

**Fichier** : `exports_global_e2e.rs:858-870`

Le test (i) couvre `Role::Consultation → 200`. Le test (a) utilise `Role::Comptable` implicitement dans le success path, mais sans label RBAC. Il n'existe aucun test labelisé `rbac_admin_200` ni `rbac_comptable_200`. La matrice RBAC testée est donc :

| Rôle | Code attendu | Testé ? |
|------|-------------|---------|
| Consultation | 200 | ✓ test (i) |
| Comptable | 200 | ✗ pas de test RBAC dédié |
| Admin | 200 | ✗ absent |

Si une régression restreint l'endpoint aux seuls `Admin`, le test (a) (qui utilise `Comptable`) échoue, mais aucun test dédié ne documente l'intention de permettre l'accès à `Admin`. L'AC RBAC est incomplète : un test `Admin` 200 et un `Comptable` 200 dédiés sont manquants.

**Impact** : couverture RBAC lacunaire — Admin non testé du tout.

---

### F5 — MEDIUM — Test (m) : `sleep(100ms)` pour attendre l'audit log — flakiness temporelle garantie

**Fichier** : `exports_global_e2e.rs:979`

```rust
tokio::time::sleep(std::time::Duration::from_millis(100)).await;
```

L'audit log est qualifié de "best-effort" dans le commentaire, mais le test (m) l'assert ensuite avec `expect("audit row inserted")`. Si la transaction d'audit prend plus de 100 ms (CI surchargée, lock MariaDB, contention I/O), le `fetch_one` retourne `Err(RowNotFound)` et le test échoue de façon non déterministe. Inversement, si l'audit est synchrone et que 100 ms est largement suffisant, le sleep est du gaspillage de temps CI. L'approche correcte est une boucle de poll avec timeout exponentiel, ou mieux, une assertion synchrone si l'audit est dans la même transaction que la réponse.

**Impact** : flakiness en CI chargée. La qualification "best-effort" est contradictoire avec une assertion ferme.

---

### F6 — MEDIUM — Test (b) : ne vérifie PAS les tables JOINées par contenu, seulement par count

**Fichier** : `exports_global_e2e.rs:549-554`

```rust
let jel_count = meta["tables"]["journal_entry_lines.csv"]["rowCount"]
    .as_u64()
    .unwrap();
assert_eq!(jel_count, 6, "expected 6 lines for A only (no B leak)");
```

Pour `journal_entry_lines`, le test vérifie que `rowCount == 6` (cohérent avec 3 entries × 2 lignes pour A). Mais un bug qui exporte 3 lignes de A + 3 lignes de B aurait également `rowCount == 6`. La vérification par count seul ne détecte pas de fuite si le nombre de lignes de A et B sont identiques. La bonne assertion serait de parser le CSV `journal_entry_lines.csv` et vérifier que tous les `entry_id` référencés appartiennent bien aux entries de A — ce que le test ne fait pas. Même commentaire pour `invoice_lines.csv` qui n'est pas vérifié du tout dans le test (b).

**Impact** : faux négatif IDOR si `|lines_A| == |lines_B|`.

---

### F7 — LOW — `unwrap()` non contrôlés dans les helpers : panics masquant l'erreur réelle

**Fichier** : `exports_global_e2e.rs:85,156,458,524,925`

Plusieurs `unwrap()` critiques dans les helpers :
- `String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap()` (ligne ~85) : panique si le secret contient des octets non-UTF8 — message générique.
- `archive.by_index(i).expect("read entry")` dans `assert_zip_response` : si le ZIP est corrompu, `expect` donne un message correct, mais les `unwrap()` alentour ne le font pas.
- `std::str::from_utf8(&raw[3..]).unwrap()` (ligne ~524) : voir F1.
- `cd[start..].find('"').expect(...)` (ligne ~925) : pas de message sur le contenu de `cd`, difficile à déboguer.

Ces panics en contexte de test `#[sqlx::test]` produisent des messages d'erreur non informatifs qui ralentissent le diagnostic. La convention recommandée dans un contexte de test adversarial est d'utiliser `expect("message contextuel")` systématiquement.

**Impact** : diagnostic difficile en CI. Sévérité LOW car comportement fonctionnel non altéré.

---

### F8 — MEDIUM — Test (f) : suppose que `company_invoice_settings` est auto-créé — assertion fragile sur comportement implicite

**Fichier** : `exports_global_e2e.rs:735-759`

```rust
("company_invoice_settings.csv", 1), // lazy-create defaults
```

Le test (f) compte sur un mécanisme de "lazy-create" de `company_invoice_settings` lors de l'export. Si ce comportement est supprimé, modifié, ou conditionnel (ex. seulement si invoqué depuis un endpoint différent), le test échoue avec `rowCount mismatch for company_invoice_settings.csv: expected=1 actual=0`. Le commentaire `// lazy-create defaults` ne cite pas la ligne de code responsable, ni le comportement de l'implémentation production. Ce test valide une *propriété implicite* du handler, pas un *contrat explicite* de l'AC. Si le comportement disparaît (refactor), l'échec du test n'indique pas clairement s'il faut corriger le test ou l'implémentation.

**Impact** : test couplé à une propriété implicite non documentée dans l'AC.

---

### F9 — LOW — Test (g) : mesure inclut le temps de seed (1000 entries en séquentiel)

**Fichier** : `exports_global_e2e.rs:815-830`

```rust
let start = std::time::Instant::now();
let resp = app.client.get(...).send().await.unwrap();
```

`Instant::now()` est correctement placé après le seed. Toutefois, le seed de 1000 entries est effectué via 1000 appels séquentiels à `post_entry` (boucle `for i in 0..1000`), ce qui est extrêmement lent (potentiellement 10-30s en soi). Le test est marqué `#[ignore]` ce qui atténue le risque en CI, mais le seuil de 10s pour l'*export* est mesuré correctement. En revanche, `entry_ids` est construit mais jamais utilisé (dead code). Cela ne nuit pas à la correction mais indique un résidu de refactoring.

**Impact** : dead code (`entry_ids`), pas d'impact fonctionnel. Seed séquentiel lent pour un test ignoré.

---

### F10 — MEDIUM — Test (k) : extraction filename par `find('"')` casse sur RFC 5987 `filename*=UTF-8'...'`

**Fichier** : `exports_global_e2e.rs:922-935`

```rust
let start_marker = "filename=\"";
let start = cd.find(start_marker).expect("filename in CD") + start_marker.len();
let end = cd[start..].find('"').expect("closing quote in CD");
let fname = &cd[start..start + end];
```

La valeur du header `Content-Disposition` peut être :
```
attachment; filename="kesh-export-CI-Test-Company-Pattern-2026-05-17.zip"; filename*=UTF-8'fr-CH'kesh-export-...
```
Si `filename*=UTF-8'fr-CH'` précède `filename=` dans la valeur, ou si le nom de la compagnie contient un `"`, le parsing maison casse. Plus subtil : le test (j) vérifie que `filename*=UTF-8'fr-CH'` est présent, et le test (k) extrait uniquement `filename=` pour valider le pattern de date. Ces deux tests sont donc en désaccord sur lequel est le "canonical filename". Si l'implémentation supprime le fallback `filename=` et ne fournit que `filename*=`, le test (k) échoue avec `expect("filename in CD")` sans indiquer que le vrai filename est dans `filename*`.

**Impact** : fragile sur les implémentations RFC 5987 modernes.

---

### F11 — LOW — Duplication massive de boilerplate entre tests (spawn_app + request + body)

**Fichier** : `exports_global_e2e.rs` — presque tous les tests (a, c, d, e, i, j, k, l, m, q, r, s, t)

Chaque test répète le même bloc :
```rust
let app = spawn_app(pool).await;
let resp = app.client.get(app.url("/api/v1/exports/global.zip")).bearer_auth(&ctx.jwt).send().await.unwrap();
let body = resp.bytes().await.unwrap();
let entries = assert_zip_response(&body);
```
Ce pattern est répété ~12 fois sans extraction en helper. Si l'URL ou l'endpoint change, 12 sites de modification. Le fichier `reports_export_e2e.rs` mentionné en commentaire d'en-tête a probablement ce même pattern, ce qui signifie la duplication s'étend sur deux fichiers de tests. Pas de helper `fetch_global_zip(&app, &jwt) -> Vec<(String, Vec<u8>)>`.

**Impact** : maintenabilité dégradée, risque de divergence silencieuse entre tests.

---

### F12 — MEDIUM — Test (u) : `list_all_lines_by_company` pour `invoice_lines` retourne 0 — assertion tautologique

**Fichier** : `exports_global_e2e.rs:1422-1426`

```rust
let invoice_lines_a = invoices::list_all_lines_by_company(&pool, a.company_id)
    .await
    .unwrap();
// 1 invoice sans ligne → 0 (l'INSERT direct SQL ne pose pas de invoice_lines)
assert_eq!(invoice_lines_a.len(), 0);
```

Cette assertion est tautologique : on a inséré une facture *sans lignes*, donc vérifier que `len() == 0` ne prouve pas que le scoping fonctionne — un bug qui retourne `[]` pour toutes les companies passerait également. Une assertion de scoping réelle nécessite **au moins 1 ligne pour A et 1 ligne pour B**, puis vérifier que l'appel pour A ne retourne que la ligne de A. Tel quel, ce test pour `invoice_lines` ne couvre pas l'IDOR.

**Impact** : faux négatif IDOR sur `invoice_lines_by_company`. Le scoping JOIN de cette fonction n'est pas validé.

---

### F13 — LOW — `TEST_ADMIN_PASSWORD` non utilisé dans les tests qui construisent les users manuellement

**Fichier** : `exports_global_e2e.rs:63,207,709`

```rust
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
// ...
password_hash: hash_password("password123").unwrap(),
```

La constante `TEST_ADMIN_PASSWORD` est déclarée mais les `seed_*` helpers utilisent hardcode `"password123"`. La constante n'est jamais référencée dans les seeds. Cela suggère soit un copier-coller oublié, soit une constante résiduelle de `reports_export_e2e.rs`. `"password123"` en clair dans les tests E2E n'est pas un risque sécurité (c'est un hash pour une DB éphémère), mais la divergence entre la constante et l'utilisation est un signal de code mort.

**Impact** : dead code, cohérence du pattern tests.

---

### F14 — MEDIUM — Forge JWT : `role` est une `&str` sans validation — RBAC bypass non testé

**Fichier** : `exports_global_e2e.rs:142-157`

```rust
fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
```

La fonction `forge_jwt` accepte n'importe quelle chaîne comme `role`. Aucun test ne vérifie le comportement de l'endpoint avec un JWT contenant un `role` invalide (ex. `"Superadmin"`, `""`, `"ADMIN"` en majuscules, `"comptable"` en minuscules). Si le middleware de validation de rôle est case-sensitive ou utilise une désérialisation stricte, un `role: "ADMIN"` pourrait soit passer (bypass RBAC), soit renvoyer 403 ou 401 selon l'implémentation. Ce vecteur n'est pas couvert — un test `role_invalid_string_should_reject` est manquant.

**Impact** : gap de couverture sécurité RBAC sur les JWT forgés avec rôle invalide.

---

### F15 — LOW — Test (c) : `assert_eq!(entries.len(), 17)` avant de vérifier la réponse HTTP 200

**Fichier** : `exports_global_e2e.rs:573-574`

```rust
let entries = assert_zip_response(&body);
assert_eq!(entries.len(), 17, "ZIP must contain exactly 17 entries");
```

Si la réponse HTTP est 500 (erreur serveur), `assert_zip_response` panique sur la signature ZIP (le body contiendrait du JSON d'erreur, pas un ZIP). Le message d'erreur serait "missing ZIP local file header signature PK\x03\x04" au lieu du code HTTP réel. Un `assert_eq!(resp.status(), 200)` avant la consommation du body est manquant dans le test (c) (présent dans (a) mais pas (c)).

**Impact** : message d'erreur trompeur si le serveur retourne 4xx/5xx.

---

## Récapitulatif

| # | Sévérité | Titre court |
|---|----------|-------------|
| F1 | MEDIUM | BOM slice aveugle — faux négatif si BOM absent |
| F2 | MEDIUM | Séparateur `;` hardcodé — casse si `,` |
| F3 | MEDIUM | Tests (n)+(o) no-ops sans assertions — couverture fantôme |
| F4 | MEDIUM | Matrice RBAC incomplète — Admin non testé |
| F5 | MEDIUM | Sleep 100ms pour audit log — flakiness temporelle |
| F6 | MEDIUM | IDOR JOINées vérifié par count seul — faux négatif si `\|A\| == \|B\|` |
| F7 | LOW | `unwrap()` sans message — diagnostic CI difficile |
| F8 | MEDIUM | Test (f) couplé à lazy-create implicite |
| F9 | LOW | Dead code `entry_ids` dans test (g) |
| F10 | MEDIUM | Parsing filename maison casse sur RFC 5987 moderne |
| F11 | LOW | Duplication boilerplate ×12 sans helper |
| F12 | MEDIUM | `invoice_lines` scoping tautologique — IDOR non prouvé |
| F13 | LOW | `TEST_ADMIN_PASSWORD` déclaré mais jamais utilisé |
| F14 | MEDIUM | Forge JWT avec rôle invalide non testé (RBAC bypass) |
| F15 | LOW | Status HTTP non vérifié avant parse ZIP dans test (c) |

**Total** : 15 findings — 9 MEDIUM, 6 LOW, 0 HIGH/CRITICAL
