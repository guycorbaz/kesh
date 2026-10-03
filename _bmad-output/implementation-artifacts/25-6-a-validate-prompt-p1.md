# Prompt — validation P1, Story 25-6-a (le tableau de bord dit vrai)

*Versionné le 2026-10-03. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-a-tableau-de-bord`. **Fiche :**
`_bmad-output/implementation-artifacts/25-6-a-tableau-de-bord.md` (commit `fbdf6617`). Issues : `gh issue view 388`,
`gh issue view 389`. Règles : `CLAUDE.md`. La fiche décrit du travail **à faire** : un finding qui reproche au code de ne
pas encore le porter sera rejeté.

## Lentilles

- **A — Justesse métier et RBAC.** Ce que la tuile montre est-il vrai pour chaque rôle (Admin, Comptable, Consultation)
  et pour une clé API ? Les trois endpoints sont-ils réellement ouverts à Consultation (lis `crates/kesh-api/src/lib.rs`,
  blocs `authenticated_routes` / `comptable_routes`) ? `due-dates` rend-il bien des **restes dus** de factures validées non
  soldées — et qu'en est-il d'une facture **soldée** (25-4-d, write-off) ou d'un avoir ? L'**écart** relevé / comptable
  (AC 4) est-il juste — même date, même signe (compte d'actif : débit − crédit), relevé en `DECIMAL(18,2)` contre un
  solde à 4 décimales ? Un compte bancaire **archivé** ? Plusieurs comptes bancaires liés au **même** compte de grand
  livre (le total les compterait deux fois) ?
- **B — Faisabilité dans le code.** Chaque référence `fichier:ligne` existe-t-elle et dit-elle ce que la fiche affirme
  (`grep -nF`, `sed -n`) ? Le DTO `BankAccountWithBalance` est-il rendu par d'autres routes que `GET /api/v1/bank-accounts`
  (inventaire : `grep -rn "BankAccountWithBalance\|list_by_company_with_balances" crates`) ? « Le calcul du solde comptable
  à une date partage la requête existante » (AC 6) est-il réalisable sans changer le résultat actuel ? `formatChfBalance`
  accepte-t-il une chaîne ? Où vit `i18nMsg` pour l'accueil (`features/onboarding`) et la garde `lint-i18n-ownership`
  accepte-t-elle des clés neuves `homepage-*` depuis `routes/(app)/+page.svelte` ? Le périmètre (quatre modules) est-il
  exact ?
- **C — Complétude et tests.** Chaque AC est-il testable, et le test prévu aurait-il échoué avant le patch ? Cas
  manquants : un relevé dont `period_to` est **postérieur** au solde comptable courant ; deux comptes bancaires ; un
  relevé importé **puis** un autre plus ancien (ordre d'import ≠ ordre des dates) ; mode guidé ; les E2E existants de
  l'accueil (`frontend/tests/e2e/homepage-*.spec.ts`) — un sélecteur cassé par le renommage ? Le manuel : relis
  `docs/manual/fr/user-manual.tex` (§ *Tableau de bord* et tout site qui parle de « liquidités » ou du solde bancaire) **et
  le PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers `target/gate-logs/`). Pars
  du symptôme : `grep -rn "liquidit\|solde bancaire\|Total liquid" docs frontend/src crates/kesh-i18n README.md website`.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence), correction proposée. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`,
`cargo test`/`nextest`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `target/gate-logs/`.
