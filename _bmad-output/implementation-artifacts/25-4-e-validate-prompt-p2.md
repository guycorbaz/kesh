# Prompt — validation P2 ciblée, Story 25-4-e

*Versionné le 2026-10-01. **Une lentille** (Haiku), contexte frais, **passe ciblée** sur la seule
remédiation P1 (entrée « Validation P1 » du Change Log de la fiche).*

Dépôt `/home/gcorbaz/devel/kesh`. Fiche : `_bmad-output/implementation-artifacts/25-4-e-montant-minimum-facture.md`.
⚠️ La fiche décrit du travail **à faire** : un finding qui reproche au code de ne pas encore le porter sera rejeté.

## Axes — tous obligatoires

1. **L'inventaire des sites de l'AC 4** : est-il maintenant complet ? Pars du symptôme —
   `grep -rn "round_to_5_centimes\|roundTo5Centimes\|default_rounding_account_id" crates/ frontend/src` — et liste
   chaque site qui énumère les champs des réglages (dépôt, route, réponse, requête, exports, types frontend,
   montages de test). En manque-t-il un dans la fiche ?
2. **Les références ajoutées** (`company_invoice_settings.rs:28-34, :36-53, :493, :617`, test `:58`) disent-elles
   ce que la fiche affirme (`sed -n`) ?
3. **Contradiction** : une autre phrase de la fiche dit-elle encore qu'un site unique suffit ?

## Ce que tu rends

Findings avec sévérité, endroit, **preuve** (sortie de `grep -nF` / `sed -n`), correction. ⛔ La liste des axes
exercés ET non exercés.

## Interdits

⛔ N'écris aucun fichier du dépôt ; aucune commande mutante (`scripts/*` dont `scripts/prepare-release.sh`,
`make`, `latexmk`, `git commit`/`add`/`checkout`, `sqlx`, `cargo test`/`nextest`, `npm run`, `npx`,
`gh issue create`/`comment`/`edit`). Autorisés : lecture, `grep`, `sed -n`, `git show`/`diff`/`log`.
