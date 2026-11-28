# Prompt de validation CR-1 P3 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `~/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff de la fiche entre les commits `796ae5a9` (fin de CR-1 P1) et `272a5289` (fin de CR-1 P2) :
  `git diff 796ae5a9 272a5289 -- <fiche>` (lecture seule). Lis aussi l’entrée CR-1 P2 du Change Log.
- Pour chaque phrase ajoutée ou changée : est-elle **vraie au code** (vérifie la ligne citée), **cohérente** avec le
  reste de la fiche, avec le fichier d'epic (`_bmad-output/planning-artifacts/epic-37-multi-devises.md`, registre C-37-*,
  D1–D8) et avec les **points de contact** des autres fiches `37-*.md` (codes d'erreur, statuts HTTP, `details`, noms de
  fonctions, qui fait quoi) ? A-t-elle introduit une contradiction, une exigence irréalisable ou un renvoi faux ?
- Hors périmètre : relire toute la fiche (les passes précédentes l'ont fait).

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR1-P3-N` avec sévérité, lieu, constat (ligne de code citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche, ajoute au Change Log « 2026-11-14 — validation CR-1 P3 ciblée
(<modèle>) : <décompte> » ; signale les findings inter-fiches sans les appliquer.
