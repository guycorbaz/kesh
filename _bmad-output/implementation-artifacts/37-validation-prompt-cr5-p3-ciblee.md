# Prompt de validation CR-5 P3 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `~/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff de la fiche entre les commits `3b058509` (fin de CR-5 P1) et `dc6c1c5a` (fin de CR-5 P2) :
  `git diff 3b058509 dc6c1c5a -- <fiche>` (lecture seule). Lis aussi l’entrée CR-5 P2 du Change Log et les alignements de l’orchestrateur.
- Pour chaque phrase ajoutée ou changée : est-elle **vraie au code** (vérifie la ligne citée), **cohérente** avec le
  reste de la fiche, avec le fichier d'epic (`_bmad-output/planning-artifacts/epic-37-multi-devises.md`, registre C-37-*,
  D1–D10, R1–R10, S1–S6, A2–A29, T1–T6, U1–U6) et avec les **points de contact** des autres fiches `37-*.md` (codes d'erreur, statuts HTTP, `details`, noms de
  fonctions, qui fait quoi) ? A-t-elle introduit une contradiction, une exigence irréalisable ou un renvoi faux ?
- Hors périmètre : relire toute la fiche (les passes précédentes l'ont fait).

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR5-P3-N` avec sévérité, lieu, constat (ligne de code citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche, ajoute au Change Log « 2026-11-14 — validation CR-5 P3 ciblée
(<modèle>) : <décompte> » ; signale les findings inter-fiches sans les appliquer.



## Alignement à appliquer (37-2 seulement)

A30 (fichier d'epic) : la lecture d'une `source` inconnue est tolérante (`RateOrigin::Other(String)`, affichée telle
quelle, jamais d'erreur interne) dès cette version ; réécrire en conséquence le passage de l'AC-1 sur A28 (plus de
relèvement de `min_required` à prévoir pour une source neuve) et ajouter un test (ligne `source = 'ecb'` posée par
SQL : lue, affichée, aucune erreur).
