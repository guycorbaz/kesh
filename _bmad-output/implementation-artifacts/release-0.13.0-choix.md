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

## Revue documentaire

- **P1** (Sonnet, deux lentilles, prompt `release-0.13.0-review-prompt-p1.md`, rapports
  `kesh-gate-logs/release-0.13.0-review-p1-{A,B}.md`) : A — 1 MEDIUM, 4 LOW ; B — 3 LOW.
  - A1 (MEDIUM) appliqué : la phrase « installation à neuf : points 3 et 4 seulement » du
    CHANGELOG était fausse pour `docker-compose.prod.yml` et omettait le point 2.
  - A3 (LOW) appliqué : la voie de retour dit comment remettre la 0.12.1 (compose et image
    `:0.12.1`, copie de l'ancien compose). A5 (LOW) et B1 (LOW) appliqués (site, README).
  - A2 (LOW) non retenu : choix R2. A4 (LOW) : ordre des opérations — les URL au tag
    rendent 404 tant que le tag `v0.13.0` n'est pas posé ; rien à corriger dans les fichiers.
  - B2, B3 (LOW) : antérieurs à la branche (intertitres de la brochure, ordre d'une ligne
    du README), laissés.
- **P2** (passe ciblée Haiku, prompt `release-0.13.0-review-prompt-p2-ciblee.md`, rapport
  `kesh-gate-logs/release-0.13.0-review-p2-F.md`) : 2 MEDIUM, 2 LOW, vérifiés par
  `git show v0.12.1:docker-compose.yml` — régressions de la remédiation A3.
  - F1 (MEDIUM, confirmé) : le `docker-compose.yml` de la 0.12.1 n'a pas de ligne `image:` pour
    Kesh (seulement `build:`) — la consigne « image `:0.12.1` au lieu de `:latest` » n'y était pas
    suivable. F2 (MEDIUM, confirmé) : il republie le port 3306. Remède : garder le compose de la
    0.13.0 et n'y changer que la ligne `image:` (présente dans les deux compose de la 0.13.0).
  - F3 (LOW) appliqué : « year-end » rétabli sur le site.
  - F4 (LOW) non traité : antérieur à la branche — le gabarit `.env` du manuel écrit les
    `MARIADB_*` décommentés avec chevrons, `.env.example` les commente. À ouvrir en issue P3.
- Trend : P1 1 MEDIUM → P2 2 MEDIUM (nés de la remédiation P1) → remédiation P2 texte du manuel
  seul, sans code de production. Boucle laissée à l'arbitrage de l'orchestrateur (une passe ciblée
  P3 sur cette dernière remédiation est possible).
