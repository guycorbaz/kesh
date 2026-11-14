# Prompt de validation CR-5 P5 ciblée — fiches de l'Epic 35

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `/home/beat/limu/compta/kesh`. Fiches visées : `35-2-cours-mensuels-afc.md`, `35-3a-ecritures-en-devise-modele.md`,
`35-3b-ecritures-en-devise-ecrans.md`, `35-5-reevaluation-fin-exercice.md`, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff `git diff de5f0f81 99bd1930 -- <fiches>` (lecture seule) : remédiation de CR-5 P4
  dans 35-2, et alignement sur l'arbitrage **A30** dans les trois autres. Lis A30 (et A28) au fichier d'epic
  (`_bmad-output/planning-artifacts/epic-35-multi-devises.md`) et les entrées « CR-5 P4 » et « alignement A30 » des Change Logs.
- Règle à tenir : une `source` inconnue (`RateOrigin::Other(String)`, p. ex. `ecb`) est lue sans erreur et se traite
  **partout comme `manual`**, sauf l'affichage et l'audit, qui rendent la valeur brute.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche, avec
  l'epic et avec les autres fiches (noms `lookup`, `find_company_rate_in_tx`, `sourceDate`, `expectedRate`, `afcRate`,
  codes d'erreur) ? Contradiction, exigence irréalisable, renvoi faux ?
- **Sites non résolus de A30** : grepe `manual` et `afc` dans les quatre fiches et dans `35-1`, `35-4` ; liste chaque
  site qui décide sur la source (`= afc` / `= manual`, `afc|manual`) et dis s'il traite une source autre comme `manual`.
- Hors périmètre : relire le reste des fiches.

## Interdits

N'écris rien sauf, en phase 2, dans les **fiches visées** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR5-P5-N` avec sévérité, lieu, constat (ligne citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs, ajoute au Change Log de chaque fiche touchée « 2026-11-14 — validation
CR-5 P5 ciblée (<modèle>) : <décompte> ». Message final bref.
