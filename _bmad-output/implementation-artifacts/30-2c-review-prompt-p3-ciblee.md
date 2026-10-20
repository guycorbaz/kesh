# Prompt — passe ciblée P3 de la revue de code, Story 30-2c (document joint et consulté)

*Versionné le 2026-10-20. **Passe ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, contexte frais,
**Opus**, braquée sur le **seul commit de remédiation de code** de la passe P2. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2c`. Le diff à relire, et lui seul :

```sh
git -C /home/gcorbaz/devel/kesh-30-2 show d799b5f9
```

`d799b5f9` = `fix(302c): remédiation de la passe P2 ciblée …` : deux fichiers, `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`
(une ligne `detachError = '';` et son commentaire, dans la branche « refus périmé, facture relue avec un document » de
`attach`) et `…/supplier-document-page.test.ts` (un test neuf, mutation M-c27). Rapport qui l'a motivée :
`/home/gcorbaz/devel/kesh-gate-logs/302c-review-p2-T.md` (T1 MEDIUM corrigé ; T2 LOW écart assumé, annotation datée de
C-30-2-53 au registre `_bmad-output/implementation-artifacts/epic-30-choix-autonomes-302.md`). Règles : `CLAUDE.md`
(§ *La passe ciblée*, § *Une passe qui ne déclare pas ses axes exercés ne compte pas*, § *Le prompt d'une passe doit NOMMER le manuel*).

## Lentille T — régression de la remédiation P2

1. **Tous les chemins qui laissent `detachError` affiché à tort** : relire `attach`, `detach`, `reloadAfterRefusal`,
   `load` et le bloc `{#if detachError}` du fichier final (`grep -nF "detachError"`). Reste-t-il un chemin où la fiche
   reçoit une facture **avec** document sans que `detachError` soit vidé (rattachement réussi, refus avec facture relue
   avec document, détachement réussi, rechargement de la page) ? Un chemin où la ligne ajoutée **efface à tort** un
   message encore vrai (refus du rattachement avec facture relue **sans** document : `throw err`, `detachError` doit
   rester) ? La ligne est-elle placée avant `return` et sans effet sur `showPicker`/`notifyWarning` ?
2. **Autres variables d'erreur de la page soumises au même `return` anticipé** (`payError`, `settlementCancelError`,
   `cancelError`, `errorMsg`) : l'une d'elles peut-elle rester périmée dans cette branche ? Le grep du symptôme a-t-il
   été fait sur tout le worktree (`grep -rnE "detachError|detach-error" frontend/src frontend/tests`) ?
3. **Le test neuf** : la séquence de mocks (`get` ×3, `detach` 404, `attach` 409) correspond-elle au parcours décrit ;
   l'assertion finale (`queryByTestId("supplier-invoice-detach-error")` nul) est-elle précédée d'une attente d'un état
   qui n'existe qu'après la relecture (nom du document) — donc ne passe pas à vide ? La mutation M-c27 (retrait de la
   ligne) meurt-elle sur ce test (`/home/gcorbaz/devel/kesh-gate-logs/302c-mutP2-M-c27.log`) ? Les autres tests du
   fichier sont-ils inchangés ?
4. **Registre et fiche** : l'annotation de C-30-2-53 est-elle datée, non réécrite, et dit-elle vrai sur `attachError`
   (variable du sélecteur) ? L'entrée « Pass 2 review (ciblée) » du Change Log annote-t-elle sans réécrire l'entrée P1 ?
   Décomptes recomptés aux deux bornes (`git show 9ad839ce:… | grep -cE "^\s*it\("` : 21 ; fichier final : 22).
5. **Manuel et PDF** : le correctif n'ajoute aucun texte visible (aucune chaîne i18n, aucun changement du manuel) ;
   confirmer par `git show --stat d799b5f9` et, si doute, contrôler le PDF aplati vers la **sortie standard**
   (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
6. **Mots-clés de fermeture** : `git log 9ad839ce..HEAD --format=%B | grep -niE "\b(close[sd]?|fix(e[sd])?|resolve[sd]?):?\s+#[0-9]+"` doit être vide.

## Ce que tu rends

Findings numérotés `T1…` avec sévérité CRITICAL / HIGH / MEDIUM / LOW, fichier et ligne, **preuve** (commande exécutée,
sortie copiée), correction proposée. ⛔ **La liste des axes exercés ET non exercés** (1 à 6) : un « 0 finding » sans elle
ne compte pas comme passe. Rapport écrit d'un seul jet, un seul fichier : `/home/gcorbaz/devel/kesh-gate-logs/302c-review-p3-T.md`.
Dernier message : chemin du rapport, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, ni dans `/tmp`. Lecture seule stricte : **aucun `git checkout`, `git switch`
ni autre forme** ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`, `cargo`, `npm`, `npx`, `node`, **`python3`, `python`,
`perl`, `ruby`**, **`sed -i`**, `docker`, **`mariadb`, `mysql`**, `git commit/add/stash/fetch/pull/reset/rebase/tag/push`,
`gh` en écriture. `pdftotext` vers la **sortie standard** seulement ; aucune redirection `>` vers un fichier. Aucune
donnée réelle de Guy. Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log/show/diff/blame`, `pdftotext … -`.
