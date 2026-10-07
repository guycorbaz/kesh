# Prompt — validation P1 de la spec, Story 25-7 (soldes de départ)

*Versionné le 2026-10-06. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-7-soldes-de-depart`. **Fiche à valider** :
`_bmad-output/implementation-artifacts/25-7-soldes-de-depart.md` (arbitrages 1–5, faits, AC 1 à 9, tâches, Dev
Notes). Issue : `gh issue view 445`. Règles : `CLAUDE.md`. Aucun code n'est encore écrit : on valide la **spec** contre
le **code actuel**.

## Lentilles

- **A — Les faits** : chaque `fichier:ligne`, chaque nom de fonction, de champ, de route, de colonne, de clé cité
  dans la fiche existe-t-il et dit-il ce que la fiche lui prête ? Vérifie au minimum :
  `crates/kesh-api/src/routes/opening_balances.rs` (ordre des gardes, DTO, status), `crates/kesh-api/src/lib.rs`
  (routes), `crates/kesh-db/src/repositories/journal_entries.rs` (`create_opening_entry`, `create_in_tx` : verrous,
  `books_locked_through`, audit), `crates/kesh-db/src/entities/account.rs` (`AccountRole`, `singleton_role`),
  `crates/kesh-db/src/repositories/accounts.rs`, `fiscal_years.rs`, `crates/kesh-core/assets/charts/*.json` (2970),
  `kesh-report/src/opening.rs`, l'écran `frontend/src/routes/(app)/settings/opening-balances/+page.svelte` et la
  feature `frontend/src/lib/features/opening-balances/`, `frontend/src/lib/features/accounts/accounts.types.ts`
  (le champ du rôle existe-t-il côté client ?), le manuel `docs/manual/fr/user-manual.tex` (`:383-404`,
  `:648-669`). Les décomptes (21, 10, 19, 2 tests ; 26 sites `i18nMsg(`) se recomptent depuis la source.
- **B — La conception et les cas limites** : les AC 3, 4 et 6 tiennent-ils ?
  - Concurrence : deux compléments simultanés du même compte ; un complément pendant qu'une écriture mouvemente le
    compte ; pendant une clôture d'exercice ; pendant la génération de l'ouverture. L'ordre des verrous
    (`companies → projects → fiscal_years`) est-il respecté ? Sous REPEATABLE READ, une lecture non verrouillante
    avant un verrou fige-t-elle l'instantané quelque part dans le plan proposé ?
  - L'arbitrage 2 (date) : chaque branche est-elle atteignable, et que se passe-t-il aux bords (premier exercice
    ouvert mais date dans la période verrouillée ; plusieurs exercices ouverts ; date du jour hors de tout exercice) ?
  - L'arbitrage 1 (« jamais mouvementé ») : exclut-il des cas légitimes (compte mouvementé par une écriture
    **contre-passée** ? par une écriture supprimée ?) ; un compte archivé puis réactivé ?
  - La contrepartie (AC 4 étape 6) : sens, montant, écart nul ; un compte `Asset` saisi au crédit.
  - L'AC 2 (totaux actif/passif, compte contre-nature) : la formule est-elle définie sans ambiguïté ?
  - Le rôle `RetainedEarnings` absent, archivé, non postable, ou porté par un compte `Asset`.
  - RBAC et clés d'API ; isolation entre sociétés.
  - Les erreurs de l'AC 5 : chaque refus a-t-il un code distinct ? Le mapping existant (`map_core_error`,
    `map_opening_balances_error`) les remonte-t-il, ou une variante d'erreur neuve est-elle nécessaire (à nommer) ?
- **C — Complétude et testabilité** : chaque AC est-il testable tel qu'écrit, et les tests de l'AC 7 auraient-ils
  échoué avant le patch ? Manque-t-il un site que le patch rend faux : le **manuel** (`.tex` **et PDF aplati**,
  `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`), le site web (`website/`), le README, les
  messages d'erreur existants qui proposent la contre-passation (`grep -rn "contre-pass" crates/kesh-i18n frontend/src
  docs/manual/fr/*.tex`), les gardes structurelles (`audit_route_registry.rs`, `i18n-keys.test.ts`,
  `lint-i18n-ownership`, `e2e-selecteurs-traduits`) ? La règle de découpage (§ « Règle de splitting préventif »)
  est-elle respectée ? Les arbitrages 1–5 sont-ils cohérents avec l'issue #445 — et quand ils s'en écartent, l'écart
  est-il dit ?

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne` (de la fiche et du code), **preuve** (commande et
sortie, ou code cité relu), correction proposée. Pour tout finding affirmant qu'un code est absent ou présent : la
sortie d'un `grep -nF`. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
