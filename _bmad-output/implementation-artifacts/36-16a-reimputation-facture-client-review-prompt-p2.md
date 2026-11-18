# Revue de code P2 — passe ciblée — Story 36-16a, changer le compte analytique d'une facture client (#1167)

*Prompt versionné (CLAUDE.md, § « La passe ciblée »). **Une seule lentille**, en contexte frais, braquée sur **les seuls commits
de la remédiation de la revue P1**. Sonnet ou Opus (une passe ciblée peut aller à Haiku, mais celle-ci touche la production :
Sonnet ou Opus de préférence).*

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-36-16`, branche `story/36-16-reimputation-analytique`.
- **Périmètre** : `git show 61f86ae3e 9af468c76 1ae14918f` (code et tests de la remédiation), `e037590cc` (fiche, D9
  annotée), et le commit de documentation qui suit (Change Log, registre C-36-16a-12). Le reste de la story a été revu en P1 :
  n'y revenir que si la remédiation le touche.
- Rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e36-16/review-p1-{B,E,A}.md` (B 0/0/0/3, A 0/0/2/4, E 0/0/0/6).
- Fiche : `_bmad-output/implementation-artifacts/36-16a-reimputation-facture-client.md` (Change Log, entrée « Remédiation de la
  revue P1 ») ; registre `_bmad-output/implementation-artifacts/e36-choix-autonomes-36-16-reimputation-analytique.md`.
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e36-16/` — `rem1-*.log`, `mut/synthese-rust-rem1.log` (M18 à M21),
  `mut/synthese-front-rem1.log` (F1 à F9), captures `captures/`.

## Interdits

**Lecture seule.** Ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `git commit|checkout|stash|restore|reset`, ni `docker`, ni `gh`, ni redirection (`>`), ni fichier
temporaire. Lecture par `git show`, `git diff`, `git grep`, `/usr/bin/grep -nF` (`grep` est une fonction vers `ugrep`),
`sed -n`, `cat`. Toute affirmation de présence ou d'absence cite la sortie `grep -nF` ou la ligne lue ; tout chemin cité se
vérifie (`git cat-file -e HEAD:<chemin>` ou `ls`).

Rapport **seulement** dans `/home/gcorbaz/devel/kesh-gate-logs/e36-16/review-p2-ciblee.md`, par l'outil d'écriture.

## La lentille : chasseur de régressions de la remédiation

Pour chaque changement de `61f86ae3e`, chercher ce qu'il **casse ou laisse faux** :

1. **`period_exposure_in_tx`** (`crates/kesh-db/src/repositories/project_reassignment.rs`) : deux boucles — verrou de tous les
   exercices par `start_date`, puis `find_later_closed_in_tx` pour chaque exercice ouvert. Les écritures exposées et
   `entryCount` sont-ils les mêmes qu'avant pour un exercice clos ? `fiscalYears` : l'exercice clos en cause, sans doublon,
   dans quel ordre ? Une écriture d'un exercice ouvert suivi d'un clos, une autre d'un exercice clos : que vaut la liste ? Le
   doc-comment du module, celui de la fonction, celui de `ClosedPeriodExposure::fiscal_years` (`crates/kesh-db/src/errors.rs`),
   `docs/api-external.md` (« les noms des exercices clos en cause ») et le manuel disent-ils la même chose que le code ?
2. **Le test `an_open_year_followed_by_a_closed_one_is_exposed`** : l'état est-il celui annoncé (l'écriture dans l'exercice
   semé, ouvert ; l'exercice 2031 clos) ? Les mutations M18 (retirer la recherche) et M19 (nommer l'exercice ouvert) le
   rougissent-elles (journaux `mut/M18.log`, `mut/M19.log`) ?
3. **La barre d'actions** (`frontend/src/routes/(app)/invoices/[id]/+page.svelte`) : `flex min-w-0 flex-wrap justify-end` sur
   les rangées validée et annulée, `items-start … gap-4` sur le conteneur. Le brouillon garde `flex gap-2` : est-ce un défaut ?
   La rangée de la facture payée tient-elle à 1280 px sur les captures reprises (`captures/36-16a-*.png`) ? Le bouton *Retour*
   reste-t-il à sa place ?
4. **`projectsLoaded`** : le bouton est masqué tant que la liste n'est pas chargée ou si elle échoue. Une autre partie de la
   page en dépend-elle ? Le test `comptes analytiques non chargés : pas de bouton` discrimine-t-il vraiment (F9) ?
5. **Le message du `403 …_ADMIN_ONLY`** (×4 locales et repli Rust dans `crates/kesh-api/src/errors.rs`) : juste pour un
   Comptable **et** pour une clé ? Cohérent avec `docs/api-external.md` § 10 et le manuel ?
6. **Les tests d'ordre des refus** : facture étrangère à la version 99 (M20), compte visé d'une autre société (M21), codes
   `FORBIDDEN` / `API_KEY_READ_ONLY`.
7. **La capture simulée** (`frontend/tests/e2e/project-reassignment.spec.ts`, `page.route` sur la réponse `409`) : est-elle
   déclarée comme telle, et n'affirme-t-elle rien que seul le serveur pourrait établir ?
8. **Le Dev Agent Record et le Change Log** de la remédiation : chiffres contre leurs journaux, commits cités, décomptes.

## Rapport

Pour chaque finding : identifiant `P2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la sortie lue qui le
prouve, le remède. Puis **la liste des axes exercés et celle des axes non exercés** (sans elle, la passe ne compte pas). Puis le
décompte par sévérité.
