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
