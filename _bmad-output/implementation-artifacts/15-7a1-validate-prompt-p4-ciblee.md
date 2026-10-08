# Prompt — validation P4 CIBLÉE de la spec, Story 15-7a1

*Versionné le 2026-10-08. Une lentille (Haiku), contexte frais, lecture seule. Passe ciblée sur la remédiation P3 (commit `327ea9df`), fiche seulement ; la P3 n'avait laissé qu'un MEDIUM (attributs `test-schema`). Haiku réservé à ce type de passe (D6).*

Worktree `/home/gcorbaz/devel/kesh-15-7`. Fiche : `_bmad-output/implementation-artifacts/15-7a1-socle-transactions-onboarding.md`.

Lis `git diff c5a57db4 327ea9df -- _bmad-output/implementation-artifacts/15-7a1-socle-transactions-onboarding.md`, puis la fiche. Pour chaque hunk : contradiction introduite ? fait faux (au code, `grep -nF` ; en particulier les attributs `migrations = "./test-schema"` / `"../kesh-db/test-schema"` contre `crates/kesh-db/tests/test_schema_guard.rs:62`) ? test ou mutation qui ne compilerait pas ou ne mordrait pas ? recompte (AC, tâches, tests, mutations) ?

⛔ Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée. ⛔ Un « 0 » sans la liste des axes exercés ET non exercés ne compte pas. Rapport dans `target/gate-logs/15-7a1-p4-ciblee.md` ; dernier message : chemin, bilan, une ligne par MEDIUM+. ⛔ Lecture seule, aucune commande qui écrit, compile ou exécute.
