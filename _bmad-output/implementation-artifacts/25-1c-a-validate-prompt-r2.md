# Prompt — revalidation R2 CIBLÉE, Story 25-1c-a

*Versionné le 2026-09-15. Une lentille Sonnet en contexte frais, orthogonale à R1 (Opus).*

Chasseur de régressions. Objet : `_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt
`/home/gcorbaz/devel/kesh` (branche courante `story/25-1c-b-journal-audit-ecran`, qui contient la fiche à jour).
**Périmètre : la remédiation R1**, `git diff 46a3d54b 02d3d45f -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`.
Nomme cette base. Motif mesuré : la sévérité se déplace vers ce qu'on vient d'écrire.

⛔ Rien ne se croit sur parole. ⚠️ Ne conteste PAS les arbitrages (filtre strict, export non audité, types et
actions traduits dans la langue de l'interface, libellés écrits dans cette story).

## Axes (déclare lesquels tu as exercés)

1. **L'inventaire des sites indirects (AC 18) est-il exact et clos ?** Rejoue une extraction positionnelle en
   Python sur `crates/*/src` (hors `#[cfg(test)]`, commentaires masqués) selon la table des positions écrite
   dans la fiche : chaque site cité existe-t-il à la ligne dite, avec les valeurs dites ? En reste-t-il un non
   inventorié ? Le total est-il **92** actions et **28** types ?
2. **La garde est-elle implémentable telle que prescrite ?** Résoudre « une variable locale liée par une
   conditionnelle », suivre les appelants de `build_audit_entry`, masquer les commentaires, apparier les
   accolades de `#[cfg(test)] mod` : est-ce faisable depuis un test de `kesh-api` sans dépendance nouvelle
   (le dépôt a-t-il `syn` ? sinon, par texte) ? Le critère (d) « rouge sur site non résolu non inventorié »
   est-il décidable ? La mutation « résolution des conditionnelles retirée » rougit-elle vraiment (a) ?
3. **Module public (AC 16)** : `pub mod audit_labels;` avec `pub const` et `pub fn` — conflit avec une garde
   existante sur la surface publique de `kesh-api` (lis `lib.rs` et les tests qui l'inspectent) ?
4. **Langue du refus et du tag (AC 11, 12, 23, 24)** : `Locale::dir_name()` existe-t-il et rend-il `de-CH` ?
   `build_content_disposition` accepte-t-il ce tag ? Le test « message `RESULT_TOO_LARGE` allemand » est-il
   montable (10 001 entrées dans l'application `de-CH`) et discriminant ?
5. **Propagation** : chaque valeur modifiée (92, 120, 133, `pub`, `Config::with_locale`, `dir_name`) est-elle
   cohérente dans tout le corps, les tâches, les Dev Notes et le Change Log R1 (décomptes) ?

## Rendu

Findings avec sévérité, endroit, commande ou extrait probant, correctif ; **liste des axes exercés et non
exercés** — rien de « vérifié » sans exécution.

## Interdits

⛔ Aucune écriture dans le dépôt, aucune commande mutante : ni `scripts/*`, ni `make`, ni `git
commit/push/checkout/switch/add/stash/reset/rebase`, ni `npm install`/`npm run build`, ni `sqlx migrate`, ni
`cargo test/nextest/build`. Autorisés : lecture, `grep`, `git show/diff`, Python en lecture, copie jetable dans
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r2-a/`. `grep` est
`ugrep` (motifs complexes refusés) : Python pour les extractions à contexte.
