# Prompt — revalidation R1 de `bmad-create-story validate`, Story 25-1c-a (réouverte)

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale à la passe 5 (Sonnet). Première
passe de la boucle rouverte par trois arbitrages du Project Lead.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche **`story/25-1c-a-journal-audit-route`** (lis la fiche par `git show
story/25-1c-a-journal-audit-route:_bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`
si ta branche courante diffère). Ta mission : **trouver ce qui ferait échouer, dévier ou mentir**
l'implémentation.

**Base** : `git diff 529ab035 46a3d54b -- _bmad-output/implementation-artifacts/25-1c-a-journal-audit-route.md`
(la version validée contre la version rouverte). Nomme-la dans ton rapport. La section « Réouverture » du
Change Log dit ce qui a changé et pourquoi.

⛔ **Rien ne se croit sur parole** — ni la fiche, ni ses « faits vérifiés au sol ».
⚠️ **Ne conteste PAS les arbitrages du Project Lead** : filtre strict ; export non audité ; types d'entité
**et** actions traduits, dans le CSV et à l'écran, dans la langue de l'interface ; les libellés écrits
**dans cette story**. Conteste la **mise en œuvre**.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Les arbitrages sont-ils cités mot pour mot** contre `_bmad-output/planning-artifacts/epic-25-vague1-suite.md`
   (§ *fin de soirée*, sur la branche `story/25-1c-b-journal-audit-ecran`) ?
2. **Le module `audit_labels` (AC 16) est-il écrivable tel que prescrit ?** Type de `state.i18n` et de
   `state.config.locale` dans `AppState` ; signature réelle d'`I18nBundle::format` ; le test d'appartenance ;
   la **dérivation des clés** — deux codes réels produisent-ils la même clé ? (exécute la dérivation sur les
   codes réels, en Python dans le scratchpad).
3. **La garde de l'AC 18 est-elle faisable et discriminante ?** Lis les **signatures réelles** des quatre
   constructeurs (`entities/audit_log.rs`, `kesh-api/src/audit.rs`) : la position de l'action et du type
   est-elle la même partout ? Un test de `kesh-api` peut-il lire la source des autres crates, et comment
   exclure `#[cfg(test)]` ? L'inventaire des sites non lus est-il **complet** — cherche des actions passées par
   variable, constante, `format!`, ou helper non nommé. **Recompte 82 et 28 par exécution**, en lisant la
   position et non « tout littéral de l'appel ».
4. **Le CSV (AC 12)** : dix colonnes cohérentes partout (AC 12, 15, 23, Dev Notes) ; comment obtenir le tag
   BCP 47 depuis `state.config.locale` (`util.rs:170` `map_language_to_bcp47` prend-il un `Locale` ou une
   chaîne ?) ; `csv_sanitize` sur les libellés.
5. **La route de vocabulaire (AC 17)** et les refus sur trois routes : montage, mutation, tests.
6. **Les tests et les mutations (AC 22-24)** : pour chaque assertion, construis l'état où elle **devrait
   rougir** — le test de langue (application en `de-CH`, société en `accounting_language` française) est-il
   montable avec `spawn_app` / `AppConfig::with_locale` ? Le code inconnu (`zz.unknown_code`, 64 caractères
   max) ? Chaque mutation produit-elle un échec d'**assertion** ?
7. **Résidus de l'ancienne conception** dans le corps : `OR company_id IS NULL`, parenthèses, `companyId`,
   « onze colonnes », « douze clés », `audit_log.exported`, `HEAD`, `accounting_language` pour les en-têtes,
   « 25-1c-b » au singulier ; renvois internes entre AC (19-25), tâches T0-T11 contre les AC.
8. **Les gardes i18n du frontend** (AC 15, 25) : ajouter 123 clés `audit-log-*` au catalogue fait-il rougir
   un test frontend (orphelines, `lint-i18n-ownership`, parité, `duplicate-i18n-keys`) ? Vérifie par lecture
   des gardes, et si possible par exécution sur une copie jetable.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`checkout`/`switch`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`/`cargo build`. Autorisés : lecture, `grep`, `git show`/`git diff`,
Python en lecture, `npx vitest run` en lecture, et une copie jetable **dans le scratchpad**
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/r1-a/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes : utilise Python pour les extractions à
contexte.
