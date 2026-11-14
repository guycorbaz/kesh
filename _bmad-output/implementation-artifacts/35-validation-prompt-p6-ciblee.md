# Prompt de validation P6 ciblée — fiches de l'Epic 35

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `/home/beat/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff de la fiche entre les commits `c0801b02` (fin de P4) et `c08ccd9b` (fin de P5) :
  `git diff c0801b02 c08ccd9b -- <fiche>` (lecture seule). Lis aussi l'entrée P5 du Change Log.
- Pour chaque phrase ajoutée ou changée : est-elle **vraie au code** (vérifie la ligne citée), **cohérente** avec le
  reste de la fiche, avec le fichier d'epic (`_bmad-output/planning-artifacts/epic-35-multi-devises.md`, registre C-35-*,
  D1–D8) et avec les **points de contact** des autres fiches `35-*.md` (codes d'erreur, statuts HTTP, `details`, noms de
  fonctions, qui fait quoi) ? A-t-elle introduit une contradiction, une exigence irréalisable ou un renvoi faux ?
- Hors périmètre : relire toute la fiche (les passes P1–P5 l'ont fait).

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-P6-N` avec sévérité, lieu, constat (ligne de code citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche, ajoute au Change Log « 2026-11-14 — validation P6 ciblée
(<modèle>) : <décompte> » ; signale les findings inter-fiches sans les appliquer.
