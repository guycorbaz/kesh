# Epic 31 — registre des choix autonomes

Chaque choix tranché en autonomie : contexte, option retenue, alternatives écartées, réversibilité. Numérotation
`C-<story>-<k>`. Mise à jour **par union** (`docs/agents-orchestration.md` § 1) : chaque branche ajoute ses entrées dans
un registre propre à sa story (`epic-31-choix-autonomes-<clé>.md`), que l'orchestrateur verse ici au merge.

---

## Story 31-0 — Kickoff et actions des rétrospectives (doc seulement)

### C-31-0-1 — #503 d'abord, l'outillage et Rust en parallèle, la répétition à blanc en dernier

- **Contexte** : Guy fixe #503 en premier ; la 29-7 est reportée après #503 ; les actions C3, A2, A3, A10 et #577 sont
  reconduites ou laissées à l'orchestrateur.
- **Retenu** : 31-1 (#503) d'abord ; 31-2 (outillage) et 31-3 (Rust 1.97.1) **en parallèle** de la 31-1, dans leurs
  worktrees — elles ne touchent ni le schéma ni les écrans ; 31-4 (répétition à blanc) après le merge de la 31-1 ;
  une seule release v0.15.0 en fin d'epic.
- **Écartées** : publier la 31-1 seule en v0.14.1 — Guy reprend sa recette « à la release suivante », une seule suffit ;
  repousser l'outillage à un epic de dette — C3 et #577 touchent le filet des gates de cet epic même.
- **Réversible** : oui.

### C-31-0-2 — #577 : recréer le smoke E2E en CI, plutôt que corriger le texte

- **Contexte** : le `CLAUDE.md` affirme un smoke E2E en CI qui n'existe pas (#577, A-C7). Guy a répondu « ok » sans
  choisir entre recréer le smoke et corriger le texte ; l'orchestrateur avait recommandé de le recréer.
- **Retenu** : recréer un job minimal (login puis un appel API authentifié), dans la 31-2. C'est le seul filet si une
  story sans frontend casse le harnais, mode d'échec déjà payé (#285).
- **Écartée** : réécrire le `CLAUDE.md` pour dire que la CI ne lance aucun E2E — moins coûteux, mais laisse le
  harnais sans aucun contrôle automatique au moment où Kesh porte des livres réels.
- **Réversible** : oui (le job se retire).

### C-31-0-3 — Quatre petits volets d'outillage dans une seule story

- **Contexte** : C3 (droits du conteneur de dev), #577 (smoke), A2 (Docker authentifié), A3 (`.svelte-kit/`) sont
  petits, indépendants et sans code de production.
- **Retenu** : une story 31-2 à quatre volets, revue en une boucle. Rust 1.97.1 à part (31-3), parce que le nouveau
  clippy peut toucher tout le workspace.
- **Écartée** : quatre stories — quatre boucles de revue et quatre PR pour du YAML et un fichier SQL.
- **Réversible** : oui.

### C-31-0-4 — Les réglages nouveaux se font dans l'interface

- **Contexte** : Guy souhaite, à terme, que presque toute la configuration passe par l'interface et que le `.env` ne
  porte presque plus rien ; « pour l'instant, restons comme ça ».
- **Retenu** : l'Epic 31 ne migre aucun réglage existant ; mais tout réglage **nouveau** (fréquence et rétention de la
  sauvegarde automatique) se fait dans l'interface, pas par une variable d'environnement de plus — sauf impossibilité
  établie par la fiche.
- **Écartée** : ajouter des variables `KESH_BACKUP_*` — c'est le chemin que Guy veut quitter.
- **Réversible** : oui.

### C-31-0-5 — #695 ouverte au kickoff

- **Contexte** : la limite de 500 transactions de la réconciliation (choix C-29-9-3) n'avait pas d'issue, pour ne pas
  faire remonter le compte ; Guy, le 2026-10-23 : « si c'est nécessaire, on ouvre tout de même une issue ».
- **Retenu** : issue #695 ouverte (`enhancement`), hors périmètre de cet epic.
- **Réversible** : —

### C-31-0-6 — Le témoin du nombre de tables du NAS n'est pas bloquant

- **Contexte** : la requête des tables hors InnoDB rend « aucune ligne » sur le NAS (Guy, 2026-10-23), mais ne prouve
  rien si le nom de la base n'est pas `kesh` ; le témoin `COUNT(*) ≥ 40` reste à lancer.
- **Retenu** : la 31-1 ne le suppose pas — la sauvegarde automatique s'appuie sur l'instantané de la 28-3, qui vaut pour
  InnoDB ; la question reste posée à Guy à la liste de pré-release (action B6 de la rétrospective 28).
- **Réversible** : —
