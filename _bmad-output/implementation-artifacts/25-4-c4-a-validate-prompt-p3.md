# Prompt — validation P3 ciblée, Story 25-4-c4-a

*Versionné le 2026-10-01. **Une lentille** (Sonnet), contexte frais, **passe ciblée** sur les remédiations
P1 et P2 : `git diff 18f63026 7d848dbc -- _bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md`.*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-c4-a-arrondi-fige-a-la-validation.md`.
CR : `gh issue view 494`. ⚠️ La fiche décrit du travail **à faire** : ne reproche pas au code de ne pas encore
le porter (un tel finding sera rejeté).

## Axes — tous obligatoires

1. **Le contexte de l'erreur** (`RoundingContext::Payment | Issuance`) : la liste des six sites est-elle
   complète (`grep -rn "RoundingAccountNotConfigured" crates/`, tests compris) ? Le mapping HTTP peut-il
   choisir la clé selon le contexte avec le patron actuel de `crates/kesh-api/src/errors.rs` (comment les
   autres variantes à champ sont-elles mappées) ? Le frontend ou un test E2E lit-il le **message** de
   l'erreur de la c3-b (`grep -rn "ROUNDING_ACCOUNT_NOT_CONFIGURED\|Paramètres" frontend crates/*/tests`) —
   et l'un d'eux casserait-il ?
2. **Le refus du total arrondi nul** : son emplacement (`invoices.rs:1926-1929`, après le refus HT) est-il
   juste ? Le TTC peut-il y être calculé (les `lines_before` portent-elles `vat_rate`) ? Peut-il exister une
   pièce au HT non nul dont le TTC arrondi est nul (oui : 0.02) — et une pièce dont le TTC **brut** est nul
   mais le HT non ? Le même refus est-il nécessaire à l'**avoir** (AC 6) ?
3. **Le message d'émission** : couvre-t-il exactement les deux appelants `Issuance` (validation, avoir) ?
   Un avoir d'une facture **antérieure** (arrondi 0) peut-il déclencher le refus à tort ?
4. **Propagation** : les phrases ajoutées par P1 et P2 contredisent-elles une autre phrase de la fiche ?
   Greppe `RoundingAccountNotConfigured`, `invoiceTotalZero`, `Issuance`, `Payment`, `paiement`, `pièce`,
   `:356`, `1925`, `2096` dans la fiche.

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (sortie de commande ou code cité `fichier:ligne`), correction.
⛔ La liste des axes exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `git show`/`diff`/`log`, `gh issue view`,
`cargo check`.
