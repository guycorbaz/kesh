# Prompt de validation CR-6 P5 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `~/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est `git diff 0bcdaeac 19ec8732 -- <fiche>` (lecture seule) et l'entrée « validation CR-6 P4 ciblée »
  du Change Log. Contexte : arbitrages **A31** (ligne de différence d'une conversion calculée seulement sur
  `completeDifference: true`) et **A32** (cours déduit ramené au cours retenu `afc` si son CHF est redonné), section
  « CR-6 » de `_bmad-output/planning-artifacts/epic-37-multi-devises.md`.
- Pour chaque phrase ajoutée ou changée : vraie au code (ligne citée vérifiée), cohérente avec le reste de la fiche,
  avec l'epic et avec les points de contact des autres fiches `37-*.md` (37-3a ↔ 37-3b ↔ 37-5 : appariement au `PUT`,
  ordre des refus, conversion dans les deux modes, recette V7) ? Contradiction, exigence irréalisable, renvoi faux,
  comptabilité fausse, test manquant ?
- Hors périmètre : relire le reste de la fiche.

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée**, par **Edit** seulement. Aucun script (`scripts/*`,
`cargo`, `npm`, ni script ad hoc, ni heredoc), aucun `git` en écriture, aucun appel réseau.

## Rapport (français)

Findings `<story>-CR6-P5-N` (sévérité, lieu, constat avec la ligne lue, correctif) ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs ; Change Log « 2026-11-14 — validation CR-6 P5 ciblée (<modèle>) :
<décompte> » ; signale les findings inter-fiches sans les appliquer. Message final bref.
