# Epic 35 — choix autonomes de la story 35-8a2 (clé `e358a2`, #431)

Registre tenu par l'agent de développement. Mise à jour par union, jamais par écrasement.

## Base de la branche

### C-e358a2-1 — Fusion de la 35-3 : le côté de la base est gardé dans les onze fichiers de texte

- **Contexte** : la fusion de `story/35-3-comptes-designes-type` dans la base (35-5c, 35-5b, 35-8a1 avec la 35-1) rend
  onze conflits de texte (`CHANGELOG.md`, `sprint-status.yaml`, les deux `errors.rs`, `textes_coherents.rs`, les quatre
  `messages.ftl`, `docs/api-external.md`, `invoices/[id]/+page.svelte`). Dans chacun, le bloc du côté 35-3 est **vide** :
  la variante `DesignatedAccountsWrongType`, ses clés, ses lignes de garde, sa ligne d'API et son entrée de CHANGELOG
  sont déjà dans la base (portées par la 35-1, partie de la 35-3), et le bloc de la base ajoute ce que la 35-1 pose
  ensuite (`NUMBER_FORMAT_UNRENDERABLE`).
- **Retenu** : l'union, qui se réduit ici au côté de la base, marqueurs retirés par l'outil Edit. Comparé au même fichier
  de l'intégration à blanc (`e2e1c39c4`) : les deux `errors.rs`, `textes_coherents.rs` et la page Svelte sont identiques ;
  les écarts des `messages.ftl`, d'`api-external.md` et du CHANGELOG viennent des stories absentes ici (35-4, solde
  d'ouverture #1025-#1028, #1000) et de la 35-5c, présente ici et non dans l'intégration à blanc.
- **Écartées** : prendre le côté 35-3 (perdrait la 35-1) ; réécrire à la main.
- **Réversible** : oui (commit de fusion `4531669f8`).

### C-e358a2-2 — `sprint-status.yaml` : 35-2a et 35-2b gardées à `done`

- **Contexte** : seul conflit qui ne soit pas une union — la même clé, `done` dans la base, `ready-for-dev` du côté 35-3
  (relevé antérieur à leur développement).
- **Retenu** : `done`, l'état le plus récent, comme l'intégration à blanc.
- **Réversible** : oui.

### C-e358a2-3 — PDF des manuels : côté de la base, puis régénérés

- **Contexte** : `admin-manual.pdf` et `user-manual.pdf` en conflit binaire.
- **Retenu** : `git checkout --ours`, puis `make -B user admin` dans `docs/manual/` sous `mem-guard`
  (`kesh-gate-logs/e358a2/base-pdf.log`) : `-B` parce que le PDF repris est plus récent que les `.tex` et que `make`
  l'aurait cru à jour.
- **Réversible** : oui.
