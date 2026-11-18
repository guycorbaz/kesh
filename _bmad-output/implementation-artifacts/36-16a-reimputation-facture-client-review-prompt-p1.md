# Revue de code P1 — Story 36-16a, changer le compte analytique d'une facture client (#1167)

*Prompt versionné (CLAUDE.md, § « La passe ciblée » et « Une passe qui ne déclare pas ses axes exercés ne compte pas »).
Trois lentilles, **B** (Blind Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor), chacune en contexte frais, sur
Sonnet ou Opus (jamais Haiku pour une P1).*

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-36-16`, branche `story/36-16-reimputation-analytique`.
- Périmètre : les commits de la story, **hors** les trois fusions (`c0411138d`, release ; `8e228cda8`, la 36-15 non fusionnée —
  décision de l'orchestrateur, C-36-16a-1 ; `1cdf902e4`, la release après la fusion de la 36-15, C-36-16a-9). Le code :
  `4f103667b` (tests), `207389b77` (correctif), `4e07344bc` (corrections après le premier `test-fast --ci`) — lire
  `git show 4f103667b 207389b77 4e07344bc`, ou `git diff 1cdf902e4..HEAD` pour le seul `4e07344bc`.
- Gates déjà tournés, au code de `4e07344bc` (journaux dans `kesh-gate-logs/e36-16/`) : `test-fast --ci` 5542/0/13,
  `gate-ci` 5549/0/17, frontend complet, 17 mutations Rust et 6 frontend tuées (`mut/`), spec E2E de la story seule 3/3.
- Fiche : `_bmad-output/implementation-artifacts/36-16a-reimputation-facture-client.md` (décisions D0 à D13, AC1 à AC15, tests
  T1 à T13, Dev Agent Record) ; registre `_bmad-output/implementation-artifacts/e36-choix-autonomes-36-16-reimputation-analytique.md`
  (C-36-16-1 à 13, **C-36-16a-1 à 8**).
- Journaux : `/home/gcorbaz/devel/kesh-gate-logs/e36-16/` (rouges, mutations `mut/`, gates, captures `captures/`).
- Ce qui n'a **pas** tourné : l'E2E Playwright complet et `gate-ci` de clôture (à la clôture, au dernier commit de code) ; le PDF
  du manuel (régénéré à l'intégration).

## Interdits

**Lecture seule.** Aucune commande qui écrit, compile ou exécute un test : ni `cargo`, ni `npm`, ni `make`, ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `git commit|checkout|stash|restore`, ni
`docker`, ni `gh`, ni redirection (`>`), ni fichier temporaire. Lecture par `git show`, `git diff`, `git grep`,
`/usr/bin/grep -nF` (`grep` est une fonction vers `ugrep`), `sed -n`, `cat`. Toute affirmation de présence ou d'absence cite la
sortie `grep -nF` ou la ligne lue. Tout chemin cité se vérifie (`git cat-file -e HEAD:<chemin>` ou `ls`).

Le rapport s'écrit **seulement** dans `/home/gcorbaz/devel/kesh-gate-logs/e36-16/review-p1-<B|E|A>.md`, par l'outil d'écriture.

## Fichiers à lire

- `crates/kesh-db/src/repositories/project_reassignment.rs` (neuf) ; `crates/kesh-db/src/errors.rs` (`ClosedPeriodExposure`, deux
  variantes, `error_code`) ; `crates/kesh-db/src/repositories/mod.rs`.
- `crates/kesh-api/src/routes/invoices.rs` (`ReassignProjectRequest`, `reassign_project_handler`) ; `crates/kesh-api/src/lib.rs`
  (route) ; `crates/kesh-api/src/errors.rs` (`closed_period_response` et les deux bras) ; `crates/kesh-api/src/audit_labels.rs`.
- Tests : `crates/kesh-api/tests/project_reassignment_e2e.rs` (neuf) ; `crates/kesh-api/tests/audit_route_registry.rs`.
- i18n : le bloc `# Story 36-16a` en fin de `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`.
- Frontend : `frontend/src/lib/features/projects/ProjectReassignDialog.svelte` et `.test.ts` (neufs),
  `frontend/src/lib/features/projects/projects.api.ts`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte`,
  `invoice-display-page.test.ts`, `invoice-settlements-page.test.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`.
- Documentation : `docs/manual/fr/user-manual.tex` (sous-section *Changer le compte analytique après coup*,
  `\label{sec:reimputation-analytique}`), `docs/api-external.md` (section de la route, § 10, paragraphe « Interblocages »).
- À lire sans qu'ils soient modifiés : `crates/kesh-api/src/routes/reconciliation.rs` (`:2926-2975`, le verrou optimiste qui
  protège l'encaissement), `crates/kesh-db/src/repositories/fiscal_years.rs` (`close`, `find_later_closed_in_tx`),
  `crates/kesh-db/src/repositories/companies.rs` (`lock_books`), `crates/kesh-db/src/repositories/projects.rs`
  (`validate_taggable_in_tx`).

## Le manuel et les captures

- Vérifier le **`.tex`** contre le code : ce que le geste touche et ne touche pas, la réserve de l'Administrateur, les paires
  déliées, la trace. **Le PDF n'est pas régénéré dans la story** (à l'intégration) : le dire, et ne pas le contrôler ici. Forme
  du contrôle du PDF, à l'intégration seulement : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' |
  /usr/bin/grep -c 'Changer le compte analytique après coup'`.
- Captures : `/home/gcorbaz/devel/kesh-gate-logs/e36-16/captures/` (dialogue, avertissement, succès, rôle Lecture sans bouton),
  thème clair. Les regarder contre AC12.

## Lentilles

**B — Blind Hunter** (le diff seul, sans la fiche) : défauts de correction, d'ordre des verrous, d'atomicité, d'erreurs avalées,
de réponses qui mentent ; SQL `IN (…)` et liaisons ; `retry_on_deadlock` et sûreté du rejeu ; `serde` du corps ; dialogue
(états, double envoi, fermeture pendant l'envoi).

**E — Edge Case Hunter** : chaque branche et chaque borne — facture sans écriture (brouillon), sans règlement, avec avoir, payée ;
`projectId` `null`, égal à l'actuel archivé, inconnu, d'une autre société ; version périmée ; exercice clos, suivi d'un clos,
verrou au jour exact, verrou couvrant une partie des écritures ; clé `read`, clé d'un Administrateur, Comptable ; ligne
divergente ; paire déliée ; acceptation concurrente ; clôture concurrente ; échec de l'audit. Pour chacune : le code la traite-
t-il, un test l'exerce-t-il ?

**A — Acceptance Auditor** : AC1 à AC15 et D0 à D13 contre le code et les tests, un par un ; les mutations de la table des tests
(journaux `mut/`) — chacune a-t-elle tué ce qu'elle devait ? ; les écarts écrits au registre (C-36-16a-1 à 8) sont-ils fondés ?
le Dev Agent Record dit-il vrai (chiffres de gates contre leurs journaux, commits cités) ? la note de version ?

## Rapport

Pour chaque finding : identifiant (`B-n`, `E-n`, `A-n`), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, la sortie
lue qui le prouve, le remède proposé. Puis **la liste des axes exercés et celle des axes non exercés** — sans elle, la passe ne
compte pas. Puis le décompte par sévérité.
