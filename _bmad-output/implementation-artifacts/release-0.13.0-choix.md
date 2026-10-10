# Release 0.13.0 — choix consignés

Choix faits en autonomie pendant la préparation de la release (2026-10-10), pour la revue
finale de Guy. Chacun : contexte, option retenue, alternatives, réversibilité.

## R1 — Le compose garde l'image `gcorbaz/kesh:latest`

- **Contexte** : les URL de téléchargement du compose, de `.env.example` et des scripts
  Synology sont désormais épinglées au tag `v0.13.0` (et non plus à `main`). On pouvait
  épingler de même l'image du compose (`gcorbaz/kesh:0.13.0`).
- **Retenu** : l'image reste `gcorbaz/kesh:latest`. La procédure de mise à jour du manuel
  (`docker compose pull` puis `up -d`) repose sur ce tag mobile ; l'épingler obligerait à
  modifier le compose à chaque version et changerait la procédure, ce que la release ne
  demande pas.
- **Alternatives** : épingler l'image (reproductibilité totale, mais procédure de mise à
  jour à réécrire) ; laisser les URL sur `main` (rejeté : `main` peut être en avance sur
  l'image publiée).
- **Réversibilité** : totale — une ligne dans les deux compose et la procédure du manuel.

## R2 — Les liens `github.com/…/tree/main/docs` restent sur `main`

- **Contexte** : deux renvois « Documentation en ligne » (manuel utilisateur, manuel
  d'administration) pointent sur `tree/main/docs`.
- **Retenu** : laissés tels quels. Ce sont des liens de lecture, non des fichiers que
  l'utilisateur télécharge pour les exécuter avec l'image publiée ; la garde G19 ne vise que
  les URL `raw.githubusercontent.com`.
- **Réversibilité** : totale.

## R3 — La garde G19 suit `\keshVersion`, pas la version Cargo

- **Contexte** : la garde pouvait comparer les URL à `CARGO_PKG_VERSION` ou à la macro des
  manuels.
- **Retenu** : la macro `\keshVersion`. Les crates sont bumpés en cours de cycle (P2-bis :
  la 15-1a-i les a portés à 0.13.0 bien avant la release) alors que les manuels décrivent la
  version publiée jusqu'au bump du point 4-bis : comparer à la version Cargo aurait rendu
  `main` rouge entre les deux.
- **Réversibilité** : totale.

## R4 — Le retour arrière écrit au manuel passe par la sauvegarde de la base, pas par un `.keshbackup`

- **Contexte** : la 0.13.0 relève `kesh_version_min_required` à 0.13.0. Le manuel devait dire
  comment revenir à la 0.12.x.
- **Retenu** : seule la sauvegarde **de la base** prise avant la mise à jour (dump
  `mariadb-dump`, copie du volume, ou dump `kesh-dump.sh` sur Synology), restaurée avec
  l'image 0.12.1, rend une base que la 0.12.1 accepte : le dump recrée `_kesh_version`.
  Un `.keshbackup` pris après est refusé par la 0.12.x (409) ; un `.keshbackup` importé dans
  la 0.13.0 n'abaisse pas l'exigence de la base (`_kesh_version` n'est pas restaurée).
- **Réversibilité** : texte du manuel seulement.
