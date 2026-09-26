# Prompt — passe 2 de `bmad-create-story validate`, Story 25-3-b

*Versionné le 2026-09-25. Deux lentilles en contexte frais (Haiku 4.5) — cycle Sonnet → Haiku → Opus.
⚠️ Haiku : le diff de la remédiation est fourni **aplati en un seul fichier**.*

Ton objet est `_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`. Contexte : cf. le prompt
de la passe 1, `25-3-b-validate-prompt-p1.md`. Le Change Log de la fiche (ligne `validate P1`) dit
ce que la passe 1 a trouvé (2 HIGH, 3 MEDIUM, 2 LOW) et comment c'est corrigé.

⚠️ **Ne conteste pas les arbitrages de Guy** (Q1-Q3, Q5). Conteste leur **mise en œuvre**.

## Lentille A — chasseur de régressions

⛔ Motif mesuré sur ce projet : **la sévérité se déplace vers ce qu'on vient d'écrire.** Ton
périmètre est la remédiation de la passe 1 :
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/25-3-b-p1-remediation.diff`
(173 lignes). Pour chaque ajout : est-il **vrai** (vérifie dans le code), **cohérent** avec le reste
de la fiche, **implémentable**, et **testable** sans passer à vide ? En particulier :
- `DbError::ReconciliationNotCancellable` : cohérent avec la table de l'AC 3, l'AC 9, les Dev Notes ?
  Reste-t-il une phrase qui dit l'inverse ?
- la requête dédiée de l'exemption étroite : juste (société, `id <> ?`), et compatible avec la
  signature actuelle de `settlement_entry_cancel_blocker` (`settlement_cancellation.rs`) ?
- le rejeu `retry_with` : lis `crates/kesh-db/src/retry.rs` et `routes/onboarding.rs:596-621`. La
  signature citée (`retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, …)`) est-elle
  **exacte** ? Rejouer une closure qui prend un **verrou nommé** (`with_account_lock`) est-il sûr ?
  Quel type d'erreur la closure rend-elle, et `is_deadlock_error` le reconnaît-il ?
- le contrat du dialogue : réaliste avec les gabarits cités (`SettleInvoiceDialog.svelte`,
  `SendEmailDialog.svelte`) ?

## Lentille B — relecture complète, à froid

Lis la fiche entière comme le développeur qui l'implémentera demain. Chaque AC : implémentable
**sans deviner**, **vérifiable** ? Deux affirmations qui se contredisent ? Un test prescrit sur un
état impossible, ou qui passerait à vide ? Une citation `fichier:ligne` fausse (vérifie-en au moins
dix) ?

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (code
  lu, commande et résultat), ce qu'il faut changer. ⛔ Pour tout CRITICAL ou HIGH affirmant qu'une
  chose est **absente** ou **présente** : la commande `grep -nF "<chaîne>" <fichier>` exécutée et
  son résultat.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` (vers le
scratchpad) et `cargo check` sont autorisés.
