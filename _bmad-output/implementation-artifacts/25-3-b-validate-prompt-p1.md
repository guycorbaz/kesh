# Prompt — passe 1 de `bmad-create-story validate`, Story 25-3-b

*Versionné le 2026-09-25. Deux lentilles en contexte frais (Sonnet) — cycle Sonnet → Haiku → Opus.*

Ton objet est `_bmad-output/implementation-artifacts/25-3-b-annuler-rapprochement.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-3-b-annuler-rapprochement`. Elle ferme **#418**
(annuler un rapprochement bancaire). Ses sœurs sont **mergées** : 25-3-a-1 (règlement client) et
25-3-a-2 (règlement fournisseur) — elle **appelle** `invoice_settlements_write::cancel_settlement_in_tx`
et **réutilise** la queue commune `settlement_cancellation::settlement_entry_cancel_blocker`.
Grand-mère, source des faits : `25-3-annuler-reglement-et-rapprochement.md`.

⚠️ **Ne conteste pas les arbitrages de Guy** (sections « Arbitrages de Guy qui s'appliquent ici » et
« Arbitrages de Guy sur cette fiche », Q1-Q3, Q5). Conteste leur **mise en œuvre**.

## Lentille A — la fiche contre le code

1. **Les faits 1 à 7** : chacun est-il **vrai** ? Refais les relevés : les cinq sites qui posent
   `matched_entry_id` (`grep -rn matched_entry_id crates`), ce que chaque chemin crée, la FK, la
   contrainte de statut, `auto_match_rejected_at`, le verrou de compte, le seul écran qui montre une
   transaction rapprochée. Un **sixième** site, un chemin qui rapproche un règlement fournisseur,
   une écriture partagée par deux transactions produite par l'application ?
2. **Le geste (AC 1)** : l'ordre est-il **implémentable** et **juste** ? Défaire le lien **avant**
   `cancel_settlement_in_tx` / `reverse_in_tx` lève-t-il vraiment le rang 3 sans autorité (lis
   `reverse_in_tx_inner` et `reversal_blockers`) ? `cancel_settlement_in_tx` verrouille lui-même la
   facture, la ligne, l'écriture et l'exercice : le geste qui les a **déjà** verrouillés dans le même
   ordre crée-t-il un problème ? Que se passe-t-il pour le rang 1 (facture créditée), refusé
   **après** que le lien a été défait ?
3. **La précédence (AC 3)** : chaque rang est-il **produisible** par un vrai chemin, pour chaque
   `kind` ? L'**exemption étroite** du rang 3 est-elle implémentable sans changer le comportement
   des deux sœurs ? Le rang 1 peut-il être réutilisé **sans copie** vu la forme actuelle de
   `settlement_cancel_blocker` ?
4. **La concurrence** : l'ordre des verrous du geste contre `accept_one_invoice`, `post_manual`,
   `post_split`, `accept_one_split`, `accept_one_rule`, `post_reject`, `cancel_settlement_in_tx`
   (route de la fiche facture), `fiscal_years::close` / `reopen`. Le piège 6-bis (interblocage) est-il
   réel ? Laquelle des deux options est la bonne, et est-elle testable ?
5. **Q2** : la fiche dit que ni l'acceptation ni la queue ne lisent l'exercice de la **facture**.
   Vrai ? Le test prescrit (facture d'un exercice clos, payée dans le suivant) est-il **montable**
   par les vrais chemins (fenêtre de 30 jours, exercices, clôture) ?
6. **Exactitude** : chaque `fichier:ligne`, fonction, décompte — depuis la source.

## Lentille B — relecture adversariale complète, à froid

Lis la fiche comme le développeur qui devra l'implémenter demain, sans autre contexte. Pour chaque
AC : peut-on l'implémenter **sans deviner** ? Peut-on le **vérifier** ? Qu'est-ce qui cassera
ailleurs (appelants, écrans, exports, rapports, tests existants, `reconciliation_e2e.rs`) ?
Les routes (AC 6-8) : rôles, clés API, registre (départ 108 / 90 / 15 / 3, 111), forme des
réponses. Les textes (AC 9) : la liste des sites est-elle **close** (grep de la clé, du code **et**
de la phrase dans les quatre locales et les replis) ? `lint-i18n-ownership` accepte-t-il les clés
prévues (lis `frontend/scripts/lint-i18n-ownership.js`) ? Les écrans (AC 10-11) : le frontend
a-t-il tout ce qu'il lui faut ? Un **seul** composant de dialogue pour deux pages est-il réaliste ?
Les tests (AC 12) : l'un d'eux passerait-il **à vide**, ou porte-t-il sur un état impossible ?
N'oublie pas le **manuel** (`docs/manual/fr/user-manual.tex` et son **PDF aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) : la liste de l'AC 13
est-elle close ? Périmètre : **une** story (règle de découpage : 5 modules) ?

## Ce que tu rends

- **Les findings** : sévérité (CRITICAL / HIGH / MEDIUM / LOW), l'endroit exact, **la preuve** (code
  lu, commande et résultat), ce qu'il faut changer.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`/`checkout`, `sqlx migrate`, `cargo test`/`cargo nextest`, `npm run`,
`npx playwright`. Lecture, `grep`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` (vers
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/5bef2124-6ec8-41d8-9004-39b0653420a3/scratchpad/`) et
`cargo check` sont autorisés.
