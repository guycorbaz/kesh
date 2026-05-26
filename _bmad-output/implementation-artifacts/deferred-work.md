# Deferred Work — Code Review Findings

Cumulé des items reportés en v0.2+ depuis les passes de code-review BMAD.
Source de vérité unique pour la dette technique non-bloquante post-merge.

---

## Deferred from: code review of 9-5-3-process-codification-claude-md (2026-05-18)

Pass 1 + Pass 2 code-review (Sonnet 4.6 × 3 reviewers + Haiku 4.5 × 2 reviewers), 30 findings bruts cumulés → 16 patches appliqués + 2 deferred ci-dessous + 12 dismiss.

- **E14 — Story de remédiation Catégorie B bloquée ou annulée — mécanisme de réévaluation périodique manquant** (MEDIUM, §Tech debt management — Catégorie B) : si la story de remédiation d'une dette B est elle-même bloquée (dépendance amont reportée) ou fermée `wontfix`, la dette B reste tracée indéfiniment sans révision. Acceptable v0.1 (cohérent zero carry-forward, la rétrospective d'epic est le point de contrôle implicite). À traiter v0.2+ : ajouter un mécanisme operational (e.g. revue trimestrielle du backlog v0.2-milestone GitHub Milestone, ou règle « si la story B est `wontfix`, la dette revient en A et doit être triée »). Hors scope CLAUDE.md durable — relève d'un processus operational projet.
- **P2-F6 — Workflow Project Lead indisponible** (MEDIUM, §Tech debt — Triage hors fenêtre rétrospective) : la règle dit que l'arbitrage est fait par le Project Lead, mais aucun fallback si le Project Lead est absent/indisponible au moment de la découverte d'une dette A en cours d'Epic. Pour v0.2+ : ajouter un workflow d'escalade type « ouvrir une issue GitHub `[TRIAGE-NEEDED]` avec scénario d'impact, attendre triage humain avant action — ne pas auto-classer ». Hors scope codification CLAUDE.md durable — processus opérationnel rare.

---

## Deferred from: code review of 9-2b-export-global-zip (2026-05-17)

Pass 1 Sonnet 4.6 × 12 reviewers (4 chunks × 3 layers BH+ECH+AA), 108 findings bruts → 5 deferred ci-dessous + 15 patches appliqués + 31 dismiss.

- **D1 — `journal_entries.list_all_by_company` `ORDER BY entry_date, id` vs index `(company_id, entry_date DESC)` → filesort systématique** : perf concern non-bloquant v0.1 (PME ≤ 5k écritures). À combiner avec L4 streaming v0.2 (option : passer à `ORDER BY id` ou ajouter index `(company_id, entry_date ASC, id ASC)` dédié export).
- **D2 — `zopfli` transitive dep ~120 Ko + license restriction clause** (`Cargo.lock` zopfli 0.8.3 tiré par `zip 2.4 features = ["deflate"]` malgré `default-features = false`) : audit license v0.2 + vérifier si `zip 2.5+` corrige le gating. Alternative envisageable : passer à `async-zip` si streaming v0.2 (D1).
- **D3 — `hex_encode` perf — `format!("{b:02x}")` par byte = 32 allocs par hash** (`crates/kesh-api/src/util.rs::hex_encode`) : pour les 16 SHA-256 par export = 512 allocs inutiles. Fix v0.2 : `use std::fmt::Write; write!(&mut s, "{b:02x}").unwrap()` in-place.
- **D4 — `export_date` capturée en fin de pipeline (post-queries SQL + serialize)** : spec non explicite sur "start vs end of pipeline". Écart théorique < 10s sur dataset référence — acceptable v0.1. v0.2 : passer `export_date` en paramètre à `build_metadata_json` depuis `Instant::now()` du début handler.
- **D5 — `aria-busy` manquant sur bouton `disabled` pendant export** (`frontend/src/routes/(app)/export/+page.svelte:export-global-start`) : WCAG 2.1 SC 4.1.3 Status Messages. Dette a11y cohérente avec KF-027 #91 (`#bits-c1` DropdownMenu pré-existant). v0.2 : ajouter `aria-busy={exporting}` + `aria-label` conditionnel selon état.

---

## Deferred from: code review of 10-5-httponly-tokens-security (2026-05-26)

Pass 3 Opus 4.7 × 3 reviewers (BH + ECH + AA), 19 findings post-dédup → 3 deferred ci-dessous + 12 patches appliqués + 4 decisions résolues.

- **BH3-L1∪ECH3-L2 — `STORAGE_KEY_*` dead exports + redéclaration drift risk dans `test-state.ts`** (`frontend/src/lib/app/stores/auth.svelte.ts:38-40` + `frontend/tests/e2e/helpers/test-state.ts:35-38`) : les 3 constantes `STORAGE_KEY_ACCESS_TOKEN` / `STORAGE_KEY_REFRESH_TOKEN` / `STORAGE_KEY_EXPIRES_IN` sont exportées depuis `auth.svelte.ts` mais le store n'écrit plus jamais en localStorage post-Story-10-5. Le test helper `test-state.ts` redéclare localement ces constantes avec un commentaire "must match auth.svelte.ts — if keys change there, update here too" → drift risk reconnu mais perpétué. v0.2 cleanup : (a) retirer les 3 `export const` du store (garder `const` privé pour `localStorage.removeItem` defensive seulement), OU (b) importer depuis le store dans test-state.ts pour éliminer la redéclaration.
- **BH3-L2 — `AUTH_EXCLUDED_URLS` dead code post-buildHeaders refactor** (`frontend/src/lib/shared/utils/api-client.ts:buildHeaders`) : la constante n'est plus référencée depuis le retrait de l'injection `Authorization: Bearer` header dans `buildHeaders` (Story 10-5 T7). Commentaire `// La constante AUTH_EXCLUDED_URLS est conservée pour traçabilité mais n'a plus de rôle actif` admet le dead code. v0.2 scope cleanup : retirer la déclaration + l'import si externe.
- **ECH3-L1 — Regex JWT trop large dans `xss-token-protection.spec.ts` Scénario (a)** (`frontend/tests/e2e/security/xss-token-protection.spec.ts:3307`) : `expect(cookieString).not.toMatch(/[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/)` matche n'importe quelle chaîne 3-segments dot-separated alphanum. Si une future feature ajoute un cookie non-HttpOnly visible JS avec valeur `kesh.session.tracking` (3 segments), le test échouerait à tort. Faux-positif futur seulement, pas un bug actuel. v0.2 : restreindre le regex à pattern plus discriminant (e.g. `[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}` qui exige base64 long) OU asserter explicitement `not.toContain("kesh_access_token=")` sans regex JWT générique.
