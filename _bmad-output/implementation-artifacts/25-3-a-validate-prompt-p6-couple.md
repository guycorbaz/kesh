# Prompt — passe 6 de la 25-3-a-2 et passe 7 (ciblée) de la 25-3-a-1

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Haiku 4.5), cycle Sonnet → Haiku. Chaque
lentille lit un **seul** commit et des fichiers **entiers** — pas de diff multi-commit.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`, dans
`_bmad-output/implementation-artifacts/`.

⛔ Motif mesuré : **la sévérité se déplace vers ce qu'on vient d'écrire** — la passe précédente de la
25-3-a-2 a trouvé un HIGH **créé par la remédiation d'avant**. ⚠️ Ne conteste pas les arbitrages de
Guy listés dans chaque fiche. Un reproche au dépôt de ne pas **encore** faire ce que la fiche
prescrit n'est **pas** un défaut de la spec.

## Lentille A — passe ciblée, 25-3-a-1

Périmètre : `git show 514900dc -- _bmad-output/implementation-artifacts/25-3-a-1-annuler-reglement-client.md`
(tâches T2 et T5 réécrites, deux citations corrigées). Puis la fiche entière.
1. T2 et T5 disent-elles **exactement** ce que disent les AC 3 et 9 (noms de fonctions, familles de
   clés, emplacement des modules) ? Une autre tâche, Dev Note ou Référence décrit-elle encore
   l'ancien module unique ou une fonction unique ?
2. Les citations `crates/kesh-api/src/errors.rs:2471-2474` et `frontend/scripts/lint-i18n-ownership.js:246-251`
   sont-elles exactes ?

## Lentille B — passe complète, 25-3-a-2

La fiche entière, avec la 25-3-a-1 sous les yeux ; attention particulière au commit `514900dc`.
1. **`lastConfirmedBatch` (AC 3, 6, 9)** : calculable depuis le schéma réel
   (`crates/kesh-db/migrations/20260628000002_payment_batches.sql`, colonne `confirmed_at`) ? Le
   texte d'avertissement est-il **vrai dans tous les cas** (lot confirmé puis annulé ? lot
   confirmé dont la facture a été retirée ? `confirmed_at` nul sur un lot `confirmed` ?) ? Les
   tests à deux cycles prouvent-ils ce qu'ils disent ?
2. **La numérotation des rangs (AC 2)** est-elle maintenant cohérente avec la 25-3-a-1, partout dans
   la fiche (tests, textes, champs) ?
3. **Le reste de la fiche** : autorité dans le socle (AC 1), geste (AC 3), écran (AC 7), textes
   (AC 8), documentation (AC 10) — exacts contre le code, implémentables, testables ?
4. **Exactitude** de chaque `fichier:ligne`.

## Ce que tu rends

- Findings : sévérité (CRITICAL / HIGH / MEDIUM / LOW), endroit exact, **preuve** — pour tout
  CRITICAL/HIGH affirmant qu'une chose est absente ou présente, la commande `grep -nF` exécutée et
  son résultat —, correction.
- ⛔ **La liste des axes exercés ET non exercés.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base —
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`npm run`. Lecture, `grep`, `git show`/`log`/`diff` autorisés.
