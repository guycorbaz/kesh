# Prompt de validation CR-5 P2 ciblée — fiches de l'Epic 37

Passe **ciblée** (`CLAUDE.md` § *La passe ciblée*) : **une seule lentille**, braquée sur **la dernière remédiation**.
Dépôt `~/limu/compta/kesh`. Fiche indiquée dans ta mission, dans `_bmad-output/implementation-artifacts/`.

## Périmètre

- La remédiation à relire est le diff de la fiche entre les commits `d59a1de9` (révision CR-5) et `3b058509` (fin de CR-5 P1) :
  `git diff d59a1de9 3b058509 -- <fiche>` (lecture seule). Lis aussi l’entrée CR-5 P1 du Change Log et les alignements de l’orchestrateur.
- Pour chaque phrase ajoutée ou changée : est-elle **vraie au code** (vérifie la ligne citée), **cohérente** avec le
  reste de la fiche, avec le fichier d'epic (`_bmad-output/planning-artifacts/epic-37-multi-devises.md`, registre C-37-*,
  D1–D10, R1–R10, S1–S6, A2–A29, T1–T6, U1–U6) et avec les **points de contact** des autres fiches `37-*.md` (codes d'erreur, statuts HTTP, `details`, noms de
  fonctions, qui fait quoi) ? A-t-elle introduit une contradiction, une exigence irréalisable ou un renvoi faux ?
- Hors périmètre : relire toute la fiche (les passes précédentes l'ont fait).

## Interdits

N'écris rien sauf, en phase 2, dans **la seule fiche visée** (Edit). Aucun script (`scripts/*`, `cargo`, `npm`), aucun
`git` en écriture.

## Rapport (français)

Findings `<story>-CR5-P2-N` avec sévérité, lieu, constat (ligne de code citée), correctif ; **axes exercés / non exercés** ;
décompte. Phase 2 : applique les correctifs propres à la fiche, ajoute au Change Log « 2026-11-14 — validation CR-5 P2 ciblée
(<modèle>) : <décompte> » ; signale les findings inter-fiches sans les appliquer.


## Alignements à appliquer en phase 2 (s'ils touchent ta fiche)

- **37-2** : la copie automatique (AC-3 bis) n'a **plus** lieu à l'activation du multi-devises (vide par construction,
  37-1 CR5-P1-2) : retirer la puce « à l'activation » et `companies::update_currency_settings` des appelants (AC-3 bis,
  Tâches, tests) ; garder les déclencheurs : récupération, création d'exercice multi-devises, bascule d'exercice en
  multi-devises, changement de genre, **compte qui reçoit une devise** (`accounts::create`, `accounts::update`).
