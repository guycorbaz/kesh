# Prompt de validation CR-5 P6 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `/home/beat/limu/compta/kesh`. Fiches visées : `37-2-cours-mensuels-afc.md`, `37-3a-ecritures-en-devise-modele.md`,
`37-3b-ecritures-en-devise-ecrans.md`, `37-5-reevaluation-fin-exercice.md`, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff `git diff 929f1336 963ba8a5 -- <fiches>` (lecture seule) : remédiation de
  CR-5 P5 (4 MEDIUM, 4 LOW : ligne d'autre source au `reset` et au ↑, mentions de la grille, libellés « cours saisi
  (<source>) » de 37-3b AC-1 et 37-5 AC-6, `.keshbackup` futur au conditionnel). Lis A30 (et A28) au fichier d'epic
  (`_bmad-output/planning-artifacts/epic-37-multi-devises.md`) et l'entrée « CR-5 P5 ciblée » des Change Logs.
- Règle à tenir : une `source` inconnue (`RateOrigin::Other(String)`, p. ex. `ecb`) est lue sans erreur et se traite
  **partout comme `manual`**, sauf l'affichage et l'audit, qui rendent la valeur brute.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les autres fiches (noms `lookup`, `find_company_rate_in_tx`, `sourceDate`, `expectedRate`, `afcRate`,
  codes d'erreur) ? Contradiction, exigence irréalisable, renvoi faux ?
- **Sites non résolus de A30** : grepe `manual` et `afc` dans les quatre fiches et dans `37-1`, `37-4` ; liste chaque
  site qui décide sur la source (`= afc` / `= manual`, `afc|manual`) et dis s'il traite une source autre comme `manual`.
- Hors périmètre : relire le reste des fiches.

## Interdits

N'écris rien sauf, en phase 2, dans les **fiches visées** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR5-P6-N` avec sévérité, lieu, constat (ligne citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs, ajoute au Change Log de chaque fiche touchée « 2026-11-14 — validation
CR-5 P6 ciblée (<modèle>) : <décompte> ». Message final bref.
