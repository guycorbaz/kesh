# Prompt — revue de code P3 CIBLÉE, Story 15-1c-ii (une lentille, la remédiation P2)

*Versionné le 2026-10-10. Une seule lentille (Haiku), contexte frais, lecture seule — `CLAUDE.md` § « La passe ciblée ».*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-15-1c-ii`, branche `story/15-1c-ii-lettrage-dans-kesh`.
**Le seul commit à revoir** : `git show 3ceae727` (remédiation de la revue P2 ; aucun code de production — manuels `.tex` et
leurs PDF, `CHANGELOG.md`, `journal-entry-page.test.ts`, `frontend/tests/e2e/open-items.spec.ts`, fiche, registre des
choix). Ce que la remédiation devait corriger : section « Revue de code P2 » du Change Log de
`_bmad-output/implementation-artifacts/15-1c-ii-lettrage-dans-kesh.md`, et les rapports
`/home/gcorbaz/devel/kesh-gate-logs/15-1c-ii-review-p2-{B,E,A}.md`.

**La lentille — R (Regression Hunter)** : la sévérité se déplace vers ce qu'on vient d'écrire. Pour chaque hunk du commit :

1. **Le texte neuf dit-il vrai ?** Chaque phrase ajoutée au manuel (`docs/manual/fr/user-manual.tex`, `admin-manual.tex`)
   et au `CHANGELOG.md` est confrontée au code : « payée ou non » (annulation d'une facture fournisseur —
   `crates/kesh-db/src/repositories/supplier_invoices.rs`, propriété par l'achat dans
   `crates/kesh-db/src/repositories/journal_entries.rs`) ; « le motif nomme le premier » groupe
   (`crates/kesh-db/src/errors.rs`, `ModificationGuard::Lettered`) ; l'ordre des causes qui passent avant le lettrage
   (rapprochement bancaire, paiement détaché) ; « code en lien, puis jour » (`frontend/src/lib/features/open-items/OpenItemsTable.svelte`) ;
   « ouvert seul, il offre le bouton *Afficher les postes ouverts…* » (`OpenItemsScreen.svelte`) ; « s'il est
   lettrable » ; l'entrée #532 du CHANGELOG (« à l'écran *Mensuel → Postes ouverts*, ou par `DELETE …` »).
2. **Références LaTeX** : les étiquettes neuves `sec:factures-fournisseurs` et `sec:exports` sont-elles uniques et
   chaque `\ref` défini ? Le **PDF** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` vers
   `/home/gcorbaz/devel/kesh-gate-logs/`) porte-t-il les phrases neuves, sans `??` ?
3. **Les tests** : le texte exact attendu par `journal-entry-page.test.ts` est-il celui que rend la page (repli de
   `journal-entries-modify-blocked-lettered` dans `frontend/src/lib/features/journal-entries/blocker-messages.ts`, espace
   et parenthèses) ? L'assertion E2E ajoutée au scénario (8) est-elle juste ?
4. **Propagation** : un symptôme corrigé (« non payée », `{code}`, « export comptable complet », « le lien *Afficher* »,
   « jour du lettrage et de son code ») reste-t-il écrit ailleurs — manuels, CHANGELOG, `api-external.md`, README, fiche ?
   `grep -rnF` sur le dépôt.
5. **Décomptes** déclarés par la remédiation (Dev Agent Record : 1 + 6 + 1 = 8 et 5 + 6 + 1 = 12 lignes « 518 » ;
   131 + 9 + 3 lignes « lettr ») : recompte-les (`git show 66935feb:CHANGELOG.md | grep -c 518`, etc.).

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1c-ii-review-p3-R.md`. Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** — pour toute affirmation de présence ou d'absence, la sortie d'un
`grep -nF` copiée —, correction proposée. ⛔ **Un finding qui dit qu'un livrable de la fiche manque doit citer la ligne de
la fiche qui le prescrit et le `grep -nF` qui ne le trouve pas.** ⛔ **La liste des hunks examinés ET non examinés, et des
points 1 à 5 exercés ou non** — un « 0 finding » sans elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport (et du PDF aplati) dans `/home/gcorbaz/devel/kesh-gate-logs/` ; aucune
commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`), `make`, `xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`, `sqlx`, `cargo`,
`npm`, `npx`, `node`, `docker`, `gh` en écriture, aucune requête SQL ni HTTP, aucune écriture dans `/tmp`. Autorisés :
lecture, `grep`, `sed -n`, `wc`, `git log`/`show`/`diff`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`.
