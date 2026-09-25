# Prompt — passe 3 de `bmad-create-story validate`, Story 25-3-a

*Versionné le 2026-09-24. Deux lentilles en contexte frais (Opus) — cycle Sonnet → Haiku → Opus.*

Ton objet est `_bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`.

Contexte : fille de la 25-3 (mère `25-3-annuler-reglement-et-rapprochement.md`, `split`, source des
faits) ; socle `reverse_in_tx` **mergé** (`crates/kesh-db/src/repositories/journal_entries.rs`).
Passe 1 (Sonnet) : 1 HIGH, 3 MEDIUM, corrigés. Passe 2 (Haiku) : 2 MEDIUM, corrigés. Entre les deux,
Guy a corrigé un arbitrage (la facture fournisseur s'annule dans tous les cas sauf exercice clos →
**25-3-c**, hors de cette story). Le Change Log de la fiche dit tout cela.

⚠️ **Ne conteste pas les arbitrages de Guy** (section « Arbitrages rendus »). **Q5 / AC 3-bis** est
une *position de la fiche*, non un arbitrage : tu peux la discuter.

## Lentille A — chasseur de régressions

⛔ Motif mesuré sur ce projet : **la sévérité se déplace vers ce qu'on vient d'écrire.** Ton
périmètre est ce que les remédiations ont **ajouté ou changé** :
`git diff d6a86ce6 HEAD -- _bmad-output/implementation-artifacts/25-3-a-annuler-reglement.md`.
Pour chaque ajout : est-il **vrai** (vérifie dans le code), **cohérent** avec le reste de la fiche,
**implémentable** tel qu'écrit, et **testable** sans passer à vide ? Cherche en particulier : un
motif de la précédence de l'AC 3-bis que rien ne peut produire, ou qu'un autre motif masque
toujours ; un champ de réponse prescrit que le frontend ne peut pas exploiter ; un test prescrit
sur un état que l'application ne produit pas (c'était le HIGH de la passe 1) ; une affirmation sur
le code non vérifiée.

## Lentille B — relecture adversariale complète, à froid

Lis la fiche comme le développeur qui devra l'implémenter demain, sans autre contexte. Pour chaque
AC : peut-on l'implémenter **sans deviner** ? Peut-on le **vérifier** ? Qu'est-ce qui cassera
ailleurs dans le système (appelants, écrans, exports, rapports, tests existants, 25-3-b qui
appellera `cancel_settlement_in_tx`) ? Le périmètre est-il raisonnable pour **une** story (règle de
découpage : 5 modules ; signal : sévérité qui ne décroît pas) — et si non, quel découpage ?
N'oublie pas le **manuel** (`docs/manual/fr/user-manual.tex` et son **PDF aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (code
  lu, commande et résultat), ce qu'il faut changer.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`.
Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` et `cargo check` sont autorisés.
