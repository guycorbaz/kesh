# Epic 44 — registre des choix autonomes de la story 44-1a (clé `44-1`)

Choix faits au développement de la 44-1a (2026-11-17, Opus 5.5, base `4b3a20b0b`), hors de ce que la fiche, le fichier
d'epic et le registre `e36-choix-autonomes-36-12.md` (C-36-12-1, -3 à -10) tranchent déjà. Chaque choix : contexte, option
retenue, options écartées, réversibilité.

### C-44-1-1 — Le message de huit lignes des fixtures P3 : mots de 17 et 16 `W` en alternance

- **Contexte** : la fiche (AC5) décrit un « message de 140 caractères en 8 lignes (mots de 17 `W`) ». Huit mots de 17 `W` et
  leurs sept espaces font 143 caractères, au-delà des 140 admis (`USTRD_MAX`, `validation.rs:17`) ; sept mots de 17 et un de
  14 font 140, mais le dernier tient alors à côté du septième (45,29 + 0,78 + 37,30 = 83,4 ≤ 87) : sept lignes.
- **Retenu** : quatre paires 17 + 16 `W`, 139 caractères : deux mots voisins font au moins 45,29 + 0,78 + 42,63 = 88,7 mm
  > 87, une ligne par mot, huit lignes (relevé par `kesh-gate-logs/e44/outils/coupe_afm.py`).
- **Écartées** : (a) huit mots de 17 `W` : refusés par la validation ; (b) un autre caractère que `W` : la fiche raisonne
  en `W` partout.
- **Réversible** : oui (fixtures de test et de capture seulement).
