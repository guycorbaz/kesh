# Prompt — validation P1, Story 25-5-b (la balance porte un solde d'ouverture)

*Versionné le 2026-10-03. Trois lentilles (Sonnet), contexte frais chacune.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-5-b-balance-solde-ouverture`. **Fiche à valider :**
`_bmad-output/implementation-artifacts/25-5-b-balance-solde-ouverture.md` (commit `ac40f66b`). Issue : `gh issue view 385`.
Règles : `CLAUDE.md`. La fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter
sera rejeté.

## Lentilles

- **A — Exactitude comptable.** La règle d'ouverture (bilan depuis l'origine, résultat depuis le début de l'exercice)
  est-elle juste pour une balance suisse ? La ligne calculée du résultat reporté rend-elle **réellement** les colonnes
  d'ouverture et de clôture équilibrées — refais le calcul sur deux exercices (vente et charge en exercice 1, vente en
  exercice 2, balance de l'exercice 2 entière puis d'une période mars–juin) : Σ des ouvertures en sens débit −
  `retained_earnings` = 0 ? Et la clôture ? Le résultat **de l'exercice en cours avant la période** (janvier–février)
  est-il compté une fois, pas deux (dans les ouvertures des comptes de résultat ET dans `retained_earnings`) ? Le
  contrôle de l'AC 5 est-il une vraie vérification ou une identité ? La concordance avec le bilan tient-elle quand la
  période ne finit pas au dernier jour de l'exercice ? Un compte physique de rôle `RetainedEarnings` (report d'ouverture
  d'un migrant) est-il compté deux fois avec la ligne calculée ?
- **B — Faisabilité dans le code.** Chaque référence `fichier:ligne` de la fiche existe-t-elle et dit-elle ce que la fiche
  affirme (`grep -nF`, `sed -n`) ? L'extraction de l'AC 2 est-elle possible sans changer le comportement du grand livre
  (lis `general_ledger.rs` en entier) ? `fetch_retained_earnings` prend-il la bonne date ? La requête agrégée unique
  est-elle écrivable (règle d'inclusion sur un solde d'ouverture non nul : `HAVING` ou sous-requête ?) ? Les six colonnes
  tiennent-elles en A4 portrait avec la police et les positions actuelles (`pdf.rs`) ? Tous les consommateurs de
  `TrialBalance`/`TrialBalanceRow` et de la clé `reports-trial-balance-period-note` sont-ils listés (`grep -rn`) ? Le
  périmètre (trois modules de code) est-il exact ?
- **C — Complétude et tests.** Chaque AC est-il testable, et le test prévu aurait-il échoué avant le patch ? Manque-t-il
  un cas (période d'un seul jour ; premier exercice, `retained_earnings` = 0 ; exercice sans écriture ; compte de bilan
  sans aucun mouvement ni ouverture ; écriture datée le jour même du début de période) ? Le manuel : relis
  `docs/manual/fr/user-manual.tex` § *Balance des comptes*, § *Grand livre*, glossaire **et le PDF aplati**
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
  `/tmp/claude-1000/-home-gcorbaz-devel-kesh/379e6f94-8029-42cb-9720-fa27c2fb204c/scratchpad/`) : la fiche liste-t-elle
  **tous** les sites que le patch rendra faux ? Pars du symptôme : `grep -rn "mouvement de la période\|balance de vérification\|Balance des comptes" docs website README.md crates/kesh-i18n frontend/src`.
  CHANGELOG, README, site.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), endroit, **preuve** (commande ET sortie, `grep -nF` pour toute
présence ou absence ; calcul chiffré pour la lentille A), correction proposée. ⛔ La liste des axes exercés ET non
exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande qui écrit dans le dépôt ou dans une base : `scripts/*` (dont
`scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`,
`npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`, `sed`, `git log`/`show`/`diff`,
`gh issue view`, `pdftotext` vers le scratchpad.
