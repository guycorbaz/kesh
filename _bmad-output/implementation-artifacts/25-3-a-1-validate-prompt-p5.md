# Prompt — passe 5 de `bmad-create-story validate`, Story 25-3-a-1 — PASSE CIBLÉE

*Versionné le 2026-09-24. **Une** lentille en contexte frais (Haiku 4.5), braquée sur la seule
remédiation de la passe 4 — cf. CLAUDE.md § « La passe ciblée ». La boucle converge (P3 : HIGH +
MEDIUM → P4 : MEDIUM seulement) ; ce qui reste à relire est ce qu'on vient d'écrire.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-3-a-annuler-reglement`. **Ton périmètre est un
seul commit** : `git show 3494ea21` (deux fichiers : `25-3-a-1-annuler-reglement-client.md` et
`25-3-a-2-annuler-reglement-fournisseur.md`, dans `_bmad-output/implementation-artifacts/`).
Lis ensuite les deux fiches **en entier**, telles qu'elles sont à `HEAD`, pour juger la cohérence.

Ce que le commit a fait : permuté les rangs 4 et 5 de la table des motifs d'annulation (compte
archivé désormais **avant** l'exercice du jour, pour suivre l'ordre réel de `reverse_in_tx`) dans la
25-3-a-1, et les rangs 3 et 4 dans la 25-3-a-2 ; ajouté `README.md` à la documentation ; réparti la
correction du manuel `:1771-1774` entre les deux fiches ; ajouté une note sur le coût de la lecture.

## Axes

1. **La permutation est-elle juste ?** Vérifie dans `crates/kesh-db/src/repositories/journal_entries.rs`
   (fonction `reverse_in_tx`) l'ordre réel des contrôles « comptes archivés » et « exercice ouvert
   du jour », et les lignes citées (`:1460-1462`, `:1479-1481`).
2. **La permutation est-elle COMPLÈTE ?** Cherche dans les deux fiches **tout** ce qui désigne un
   rang par son numéro ou par sa place (« rang 4 », « rang 5 », « rangs 3-5 », « dernier »,
   « en dernier », textes, tests, montages, champs `cancelBlockedLabel` /
   `settlementCancelBlockedLabel`) et dis si chaque occurrence est cohérente avec le nouvel ordre.
   ⛔ Cherche le **jeton**, pas la phrase : `grep -nE "rang|dernier" <fiche>`.
3. **Une phrase antérieure contredit-elle le nouvel ordre ?** Par exemple une justification
   « le compte archivé reste dernier » laissée ailleurs.
4. **Les ajouts** (README, `:1771-1774`, coût de lecture) sont-ils exacts contre le dépôt
   (`README.md` ligne 40, `docs/manual/fr/user-manual.tex:1771-1774`) ?

## Ce que tu rends

- Findings : sévérité, endroit exact, **preuve** (commande `grep -nF` exécutée et son résultat pour
  tout CRITICAL/HIGH), correction.
- ⛔ **La liste des axes exercés ET non exercés.** Un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier, n'exécute aucune commande qui écrit dans le dépôt ou une base —
`scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, tout
`git commit`/`push`/`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`,
`npm run`. Lecture, `grep`, `git show`/`log`/`diff` autorisés.
