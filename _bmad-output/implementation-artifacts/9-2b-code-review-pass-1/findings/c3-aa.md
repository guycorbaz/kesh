# Chunk 3 — Acceptance Auditor findings (E2E tests)

**Auditeur** : Acceptance Auditor (Sonnet 4.6)
**Date** : 2026-05-17
**Diff** : `chunk-3-e2e-tests.diff` (fichier `crates/kesh-api/tests/exports_global_e2e.rs`, 1421 lignes, 21 tests)
**Spec référence** : `9-2b-export-global-zip.md` § AC #29 (a-u) + T9 + DoD gates
**Méthode** : Lecture diff + lecture spec AC #29 sub-cases + ground-truth comparée pour (b) IDOR, (f) empty company HashMap, (d) metadata shape, (m) audit log, (u) scoping.

---

## CRITICAL

_(aucun)_

---

## HIGH

### C3-AA-HIGH-01 — AC #29(f) : HashMap empty company diverge du preset `with-company-no-fy` — `accounts.csv=0` + `vat_rates.csv=0` au lieu de `5` + `4`

**AC violée** : AC #29(f)
**Fichier:ligne** : `exports_global_e2e.rs:735-759`

La spec stipule explicitement (après 3 passes adversariales + ground-truth `test_fixtures.rs:202-275`) que le preset `with-company-no-fy` injecte par défaut **5 accounts** + **4 vat_rates** + 1 company + 1 company_invoice_settings. La HashMap attendue est donc :

```
accounts.csv=5, vat_rates.csv=4, company.csv=1, company_invoice_settings.csv=1, 12 autres=0
```

Or le test implémenté (lignes 684-760) crée manuellement une company minimale **sans** appeler le preset `with-company-no-fy`, et asserte :
```rust
("accounts.csv", 0),
("vat_rates.csv", 0),
```

Ces deux assertions passent vrai sur le setup minimal mais échouent si le test est réécrit pour utiliser le vrai preset CI (`seed_accounting_company_no_fy` depuis `kesh_db::test_fixtures`). La spec (ligne 153-177) est explicite : "PAS d'assertion uniforme `== 0`" et donne la HashMap corrigée par Pass 3 ground-truth. Le commentaire en tête du test (ligne 732-734) reconnaît la divergence ("setup minimal manuel : company seule + lazy-create CIS") mais n'en documente pas la justification ni le delta par rapport au preset CI.

**Impact** : si un futur test ou une migration réutilise le vrai preset `with-company-no-fy`, ce test affirmera faussement 0 accounts alors qu'il en a 5. La couverture de la spec AC #29(f) est incomplète.

**Correction attendue** : soit appeler `seed_accounting_company_no_fy` (preset CI réel) et corriger la HashMap (`accounts.csv=5, vat_rates.csv=4`), soit documenter explicitement dans le test pourquoi le setup minimal diverge du preset et que la HashMap `=0` est délibérée pour ce variant (avec une note indiquant que le preset complet est couvert séparément).

---

### C3-AA-HIGH-02 — AC #29(b) : IDOR JOIN `invoice_lines.csv` non vérifié (seul `journal_entry_lines` testé)

**AC violée** : AC #29(b) — "2 CSV JOINées : `invoice_lines.csv` (assert aucun `invoice_id` de B) + `journal_entry_lines.csv`"
**Fichier:ligne** : `exports_global_e2e.rs:545-555`

La spec (ligne 148) exige la vérification des 2 tables JOINées :
- `journal_entry_lines.csv` : assert aucun `entry_id` de B → **testé** via `jel_count == 6`.
- `invoice_lines.csv` : assert aucun `invoice_id` de B → **absent** du test (b).

Le test AC #29(b) (`export_global_zip_multi_tenant_idor_scoping`, lignes 496-555) ne vérifie que `journal_entry_lines.csv` dans le metadata. La table `invoice_lines.csv` n'est ni vérifiée dans le contenu du CSV ni dans le rowCount metadata. Un bug de scoping SQL dans `list_all_lines_by_company` pour les `invoice_lines` ne serait pas détecté par ce test.

**Clarification** : Le test AC #29(u) (ligne 1422-1427) vérifie bien `invoice_lines_a.len() == 0` mais dans un contexte où les deux companies ont 0 lignes (aucune `invoice_lines` insérée). Ce n'est pas un test de scoping cross-company effectif — ça asserte uniquement que les deux results sont vides, sans garantir l'absence de fuite B→A.

**Correction attendue** : Dans le test (b), après avoir seedé une `invoice_lines` pour company B (via un invoice + invoice_line SQL direct), vérifier que `meta["tables"]["invoice_lines.csv"]["rowCount"].as_u64()` == 0 pour l'export de A. Ou a minima, documenter pourquoi `invoice_lines` est exclu de la vérification JOIN dans ce test.

---

## MEDIUM

### C3-AA-MEDIUM-01 — AC #29(g) : message d'assertion `perf` référence des ACs incorrects (`#20`, `#22` au lieu de `#29(g)`)

**AC violée** : AC #29(g)
**Fichier:ligne** : `exports_global_e2e.rs:828-835`

Les deux assertions du test perf (`export_global_zip_large_dataset_perf`) citent des ACs erronés dans leurs messages d'échec :
```rust
"AC #20 perf : export > 10s pour ~1000 entries (got {:?})"
"AC #22 perf : ZIP > 5 MB pour ~1000 entries (got {} bytes)"
```

AC #20 (dans la spec) est "Content-Type: application/zip" et AC #22 est "ZIP < 5 MB". La vérification de la durée < 10s correspond plutôt à AC #21. Cette confusion dans les messages d'erreur rend le diagnostic de régression difficile — un développeur recevant un échec "AC #20 perf" cherchera l'AC `Content-Type`, pas la contrainte de durée.

**Correction attendue** : Remplacer par `"AC #29(g) perf : ..."` (ou la référence correcte AC #21/#22) dans les deux messages `assert!`.

---

### C3-AA-MEDIUM-02 — AC #29(u) : `invoice_lines` scoping non effectivement testé (deux companies à 0 lignes)

**AC violée** : AC #29(u) — "assert `result[0].entry_id IN {A's entry_ids}` pour JOINées"
**Fichier:ligne** : `exports_global_e2e.rs:1422-1427`

Le test AC #29(u) (`export_global_zip_repo_scoping_all_list_all_by_company`) insère dans `invoices` une row pour A et une pour B, mais n'insère aucune `invoice_lines` pour l'une ou l'autre. L'assertion finale :
```rust
assert_eq!(invoice_lines_a.len(), 0);
```
Prouve uniquement que A n'a pas de lignes — pas qu'une ligne de B ne fuirait pas. Pour que ce test garantisse le scoping cross-company de `list_all_lines_by_company` (invoices), il faudrait qu'au moins une `invoice_lines` soit insérée pour B et que le résultat de A reste 0.

La spec (ligne 193) dit "invoice_lines via fixtures parent" — l'implémentation ne pose pas ce fixture parent pour invoice_lines.

**Correction attendue** : Insérer au moins 1 `invoice_lines` pour company B (via SQL direct sur l'invoice de B), puis assert `invoice_lines_a.len() == 0` — ce qui prouve réellement l'absence de fuite B→A.

---

### C3-AA-MEDIUM-03 — AC #29(d) : `exportDate` regex manque la validation des valeurs numériques (seuls les séparateurs sont vérifiés)

**AC violée** : AC #29(d) — "match regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`"
**Fichier:ligne** : `exports_global_e2e.rs:628-643`

La spec (ligne 151) exige que `exportDate` corresponde à la regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`. Le test vérifie :
- `date.len() == 20`
- `date.ends_with('Z')`
- Les séparateurs `-`, `T`, `:` aux bonnes positions

Il ne vérifie PAS que les segments sont composés de chiffres (`\d`). Une valeur comme `"XXXX-XX-XXTXX:XX:XXZ"` passerait le test. Bien que peu probable, l'utilisation d'une vraie vérification regex (ou de `date.chars().all(|c| c.is_ascii_digit() || "-T:Z".contains(c))`) correspondrait mieux à l'intent de la spec.

**Note** : Finding de priorité basse si `chrono` est fiable, mais la spec est explicite sur la regex. Garder MEDIUM car c'est un AC de validation de format.

**Correction attendue** : Remplacer les checks manuels par un assert regex ou au moins vérifier que chaque segment est numérique :
```rust
let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
assert!(re.is_match(date), "exportDate format invalid: {date}");
```
(avec `regex` en dev-dependency si pas déjà présent, ou via checks par segment numérique).

---

## LOW

### C3-AA-LOW-01 — AC #29(n)(o) : tests placeholders vides sans assertion sentinelle compilée

**AC violée** : AC #29(n) + AC #29(o)
**Fichier:ligne** : `exports_global_e2e.rs:1018-1024` et `1031-1033`

Les deux tests `export_global_zip_error_500_sql_wired` et `export_global_zip_error_500_zip_wired` sont des corps de fonctions entièrement vides — pas même une assertion `assert!(true)` ou un commentaire `// COVERED_BY: unit test T10(i)(j)`. Bien que la spec les accepte comme placeholders (ligne 851), un test vide sans aucun corps peut être confondu avec un test non-implémenté vs un test délibérément passthrough. Le pattern de la spec `reports_export_e2e.rs` Story 9-2a utilise des commentaires explicatifs dans les tests placeholder.

**Correction attendue** (nit) : Ajouter a minima un commentaire de traçabilité dans le corps :
```rust
// Couverture réelle : errors::tests::global_export_failed_* + exports::global::tests::build_zip_*
// Ce placeholder garantit que le fichier compile et que le test est enregistré.
let _ = (); // sentinel
```

---

### C3-AA-LOW-02 — AC #29(k) : calcul de `date_suffix` décalé d'1 caractère (len-14 vs len-15)

**AC violée** : AC #29(k) — "Filename pattern `kesh-export-.+-YYYY-MM-DD.zip`"
**Fichier:ligne** : `exports_global_e2e.rs:929-935`

Le test extrait le suffixe date via :
```rust
let date_suffix = &fname[fname.len() - 14..fname.len() - 4];
```
`.zip` = 4 chars → `fname.len() - 4` = fin avant `.zip`. ✓
`YYYY-MM-DD` = 10 chars → `fname.len() - 14` = 4 chars avant le `-` séparateur → extrait `YYYY-MM-DD` (10 chars). ✓ (mathématiquement correct)

L'assertion vérifie uniquement que le suffixe contient 2 tirets. Cela ne valide pas que les segments sont `\d{4}-\d{2}-\d{2}`. Un nom `kesh-export-foo-ab-cd-ef.zip` passerait. Finding de gravité basse (cosmétique) mais la spec exige `/^kesh-export-.+-\d{4}-\d{2}-\d{2}\.zip$/`.

**Correction attendue** (nit) : Renforcer la vérification pour valider les segments numériques de la date.

---

## Résumé

| Sévérité | Count | IDs |
|----------|-------|-----|
| CRITICAL | 0 | — |
| HIGH | 2 | C3-AA-HIGH-01, C3-AA-HIGH-02 |
| MEDIUM | 3 | C3-AA-MEDIUM-01, C3-AA-MEDIUM-02, C3-AA-MEDIUM-03 |
| LOW | 2 | C3-AA-LOW-01, C3-AA-LOW-02 |
| **Total** | **7** | |

**Findings bloquants (> LOW)** : 5 — une nouvelle passe est requise après correction selon la règle de remédiation CLAUDE.md.

**Point critique** : C3-AA-HIGH-01 est le finding principal identifié dans les instructions — la HashMap `accounts.csv=0, vat_rates.csv=0` du test AC #29(f) diverge de la spec `accounts.csv=5, vat_rates.csv=4` (ground-truth preset `with-company-no-fy`). Le test utilise un setup manuel minimal à la place du preset CI, ce qui invalide la couverture AC #29(f) dans le contexte du preset réel.
