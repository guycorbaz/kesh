# Prompt de validation CR-2 P3 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `~/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff de la fiche entre les commits `02b217ac` (avant CR-2 P2) et `cb15f2cd` (fin de CR-2 P2) :
  `git diff 02b217ac cb15f2cd -- <fiche>` (lecture seule). Lis aussi l’entrée CR-2 P2 du Change Log et les alignements de l’orchestrateur.
- Pour chaque phrase ajoutée ou changée : est-elle **vraie au code** (vérifie la ligne citée), **cohérente** avec le
  reste de la fiche, avec le fichier d'epic (`_bmad-output/planning-artifacts/epic-37-multi-devises.md`, registre C-37-*,
  D1–D8) et avec les **points de contact** des autres fiches `37-*.md` (codes d'erreur, statuts HTTP, `details`, noms de
  fonctions, qui fait quoi) ? A-t-elle introduit une contradiction, une exigence irréalisable ou un renvoi faux ?
- Hors périmètre : relire toute la fiche (les passes précédentes l'ont fait).

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR2-P3-N` avec sévérité, lieu, constat (ligne de code citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche, ajoute au Change Log « 2026-11-14 — validation CR-2 P3 ciblée
(<modèle>) : <décompte> » ; signale les findings inter-fiches sans les appliquer.

## Arbitrage à appliquer (37-2 seulement)

R4 est **amendé** dans le fichier d'epic (« R4 amendé ») : la tâche automatique vérifie **une fois par jour, dès le
1er du mois**, tant que le mois courant ou le mois suivant manque pour un exercice multi-devises ouvert, puis plus
rien jusqu'au mois suivant ; plus le démarrage et la connexion. Aligne la fiche 37-2 (AC-4, Tâches, tests,
`prochain_reveil`) : plus de fenêtre « à partir du 25 » ni de cadence 6 h. Cette révision est relue par la passe
suivante.
