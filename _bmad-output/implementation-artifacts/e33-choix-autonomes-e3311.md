# Registre des choix autonomes — Story 33-11, le filtre de CI (Epic 33)

Tenu le 2026-11-07 par l'agent de développement (Opus 5.5), en autonomie.

- **C-e3311-1 — Un code toujours écrit, jamais une absence.** *Contexte* : l'AC-2 admet « rend `code=true`, ou n'écrit rien ». *Retenu* : le script écrit toujours `code=true` et sort en 0 sur tout chemin d'échec, motif sur la sortie d'erreur : le job `changes` reste vert, et les quatre jobs tournent. Seul un test rouge, ou une panne du job, produit l'absence de sortie, qui rend de toute façon `true` par les conditions. *Écarté* : sortir en erreur, ce qui rendrait le job rouge sur une PR sans défaut (historiques sans base commune). *Réversibilité* : deux lignes.
- **C-e3311-2 — Le job s'affiche « Filtre des artefacts ».** *Contexte* : la fiche ne nomme pas le check ; il n'est pas requis, et ne doit pas le devenir (F2). *Réversibilité* : une ligne, tant qu'aucune protection ne le vise.
- **C-e3311-3 — La garde de `ci.yml` admet la seule ouverture `always()` en plus de `!cancelled()`.** *Contexte* : l'AC-5 prévoit que la garde suive l'AC-6 (d), si le relevé impose `always()`. *Retenu* : la garde lit l'ouverture du fichier, et exige la même aux quatre jobs ; passer à `always()` ne touche donc qu'à `ci.yml`. *Écarté* : deux versions de la garde. *Réversibilité* : une ligne.
