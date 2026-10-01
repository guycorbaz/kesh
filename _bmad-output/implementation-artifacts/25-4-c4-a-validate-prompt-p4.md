# Prompt — validation P4 ciblée, Story 25-4-c4-a

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P3 : `git show 2ee6bfa0` (un seul commit, diff aplati).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md`.
⚠️ La fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera
rejeté. Ne juge que la **fiche**.

## Axes — tous obligatoires

1. **La garde de l'avoir** (`rounding_amount != 0` avant d'exiger le compte) : est-elle écrite de la même
   façon qu'à la validation (AC 3) et au règlement (`crates/kesh-db/src/repositories/invoice_settlements_write.rs:181-183`) ?
   Un autre endroit de la fiche (AC 4, AC 5, Dev Notes, tâches) exige-t-il encore le compte sans condition ?
   Greppe `compte d'arrondi`, `rounding_account_for_write`, `Issuance` dans la fiche.
2. **Le test ajouté** (avoir à arrondi nul sans compte) : discriminant — rougirait-il si la garde était
   retirée ? La mutation nommée correspond-elle ?
3. **Les corrections de références** : `crates/kesh-db/src/errors.rs:352` et `crates/kesh-db/src/repositories/company_invoice_settings.rs:312`
   disent-ils ce que la fiche affirme (`sed -n` ou `grep -nF`) ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (sortie de `grep -nF` / `sed -n`, ou passage de la fiche cité),
correction. ⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`.
