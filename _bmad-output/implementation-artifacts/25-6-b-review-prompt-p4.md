# Prompt — revue de code P4 ciblée, Story 25-6-b

*Versionné le 2026-10-05. **Une lentille** (Haiku), contexte frais. Passe ciblée (`CLAUDE.md` § « La passe ciblée ») :
P3 n'a rendu que des LOW ; leur remédiation, `78b16da6`, touche la production — c'est elle seule qu'on relit.*

Dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-6-b-pdf-facture-archive`. **Diff à revoir : `git show 78b16da6`**
(un seul commit, à lire tel quel). Contexte : l'entrée « Revue de code P3 » du Change Log de
`_bmad-output/implementation-artifacts/25-6-b-pdf-facture-archive.md`.

## Lentille — Regression hunter

Pour chacun des quatre changements de production, dis s'il introduit un défaut :

1. `crates/kesh-api/src/routes/issued_invoice_pdf.rs`, `render_and_pose` : une facture déjà figée à la relecture est
   adoptée par `adopt`. Ce chemin peut-il servir un document à un **envoi** pour une facture non validée ? boucler ?
   court-circuiter une garde de `get_or_freeze` ?
2. Même fichier, `adopt` et `pose(…, usage)` : la garde `Usage::Send` est-elle tenue sur **chaque** chemin qui rend
   un PDF à un envoi ? Liste ces chemins (`grep -n "Adopted\|serve_frozen" crates/kesh-api/src/routes/issued_invoice_pdf.rs`).
3. Même fichier, fin de `refreeze` : le `match` sur le statut — chaque statut possible (`draft`, `validated`,
   `cancelled`, autre) donne-t-il un code juste ? lequel répond une facture **validée** dont l'empreinte a changé ?
4. `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `confirmRefreeze` : la relecture isolée — que voit
   l'utilisateur si le refigeage échoue ? si la relecture échoue ? `refreezeSubmitting` retombe-t-il dans les deux cas ?

Et : les tests ajoutés (`un_envoi_n_adopte_pas_le_gel_d_une_facture_annulee_entre_temps`, le Vitest « relecture
échoue ») prouvent-ils ce qu'ils disent ? Le CHANGELOG ajouté dit-il vrai ?

⚠️ **Pour tout finding affirmant qu'un code est absent ou présent, cite la sortie d'un `grep -nF` sur le fichier** —
pas un numéro de ligne lu dans le diff, qui peut être décalé.

## Ce que tu rends

Findings avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, preuve citée, scénario d'échec, correction.
⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier du dépôt hors `target/gate-logs/` ; aucune commande qui écrit dans le dépôt ou dans une base :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`, `sqlx`,
`cargo test`/`nextest`/`build`, `npm run`, `npx`, `gh issue create`/`comment`/`edit`. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh issue view`.
