# Prompt — validation P4 de la fiche 49-5, passe **ciblée** sur le retrait du lot C

Tu fais une **passe ciblée** de validation de la fiche de la story de solde de l'**Epic 49** de Kesh (comptabilité suisse ; Rust/Axum,
MariaDB, SvelteKit). **Une seule lentille, contexte frais, modèle Haiku.** Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-49-solde`, branche `story/49-5-solde-dette-epic`.
**Objet : le seul dernier commit de la branche**, qui retire de la fiche le lot C (#1499) et le déplace dans un fichier à part.
Tu reçois le **diff aplati** de ce commit, à produire toi-même en premier (lecture seule) :

```
cd /home/gcorbaz/devel/kesh-49-solde && git diff 7f435dbad HEAD
```

Fichiers : la fiche `_bmad-output/implementation-artifacts/49-5-solde-dette-epic.md` ; le fichier de report
`_bmad-output/implementation-artifacts/49-5-lot-c-reporte-1499.md` ; `sprint-status.yaml` ; `e0200-choix-autonomes-e49solde.md`.

## Ce que la passe établit

- **H1 — La fiche n'a plus aucune trace exécutable du lot C.** Aucun AC, aucune tâche, aucun test, aucune mutation, aucune
  décision, aucun gate, aucune note de version ne demande au développement de toucher `scripts/tests/recette-montee.test.sh`,
  `scripts/recette-montee.sh`, `docs/testing.md` ou `docs/ci.md`. Les seules mentions qui restent de #1499 sont **descriptives** :
  statut, tableau du périmètre (« retirée »), *Issues écartées*, textes des issues à ouvrir (I-1, nouveau corps de #1499), Change
  Log. Contrôle **par grep, ligne à ligne** (`grep -nF` obligatoire) :
  `grep -nF 'lot C' <fiche>`, `grep -nF '#1499' <fiche>`, `grep -nF 'recette-montee' <fiche>`, `grep -nF 'docs/ci.md' <fiche>`,
  `grep -nF 'docs/testing.md' <fiche>`, `grep -nF 'AC-C' <fiche>`, `grep -nF 'D-C' <fiche>`, `grep -nF 'port_libre' <fiche>`,
  `grep -nF 'KESH_RECETTE' <fiche>`, `grep -nE '\*\*(T5|T6|M4|M5|M6)\*\*' <fiche>` ; classe **chaque** ligne rendue :
  descriptive (admise) ou prescriptive (finding).
- **H2 — Les lots A (#1464) et B (#1466) sont inchangés.** Dans le diff, aucune ligne retirée ou ajoutée ne touche les sections
  « Lot A », « Lot B », D-A1 à D-A3, D-B1, D-B2, AC-A1 à AC-A4, AC-B1 à AC-B3, AC-T1, AC-T2, ni les lignes T1 à T4 et M1 à M3 du
  tableau des tests — sauf des phrases de cadre qui disaient « trois lots » (à citer). Pour toute ligne du diff dans ces zones,
  citer la ligne et dire si son sens a changé.
- **H3 — Rien n'a été perdu en route.** Le fichier de report contient D-C1 à D-C4, AC-C1 à AC-C4, les lignes T5, T6, M4, M5, M6,
  la tâche et la note du lot C, et les constats V3-1 à V3-4 avec leurs remèdes (`grep -nF` de chaque identifiant dans le report ;
  chacun présent au moins une fois). Comparer avec les lignes retirées de la fiche dans le diff : aucune ligne prescriptive du lot C
  retirée de la fiche n'est absente du report.
- **H4 — Cohérence des décomptes et des renvois.** Le Change Log (dernière entrée) dit vrai : « trois mutations (M1, M2, M3) et
  quatre tests (T1 à T4) » — recompte depuis le tableau des tests de la fiche ; « deux issues sur trois » ; I-2 à I-7 ; estimation
  3 h 30 = 1 h + 1 h + 1 h 30. `sprint-status.yaml` se parse (`python3 -I -c "import yaml; yaml.safe_load(open(...))"`) et dit la
  même chose que la fiche. Les renvois `§ lot C` ou `lot C, point N` ne subsistent nulle part dans les lots A et B.

**Règle de vérité** : tout finding CRITICAL ou HIGH qui affirme l'absence ou la présence d'un texte se prouve par `grep -nF "<chaîne>"
<fichier>` (sortie citée) ; un finding réfuté par grep est écarté, et tu le dis. Ne te fie pas à ta mémoire du diff : regrepe.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun
`cargo`, `npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `flock`, `bash scripts/…`, aucun `scripts/*` (en particulier
ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni
`scripts/recette-montee.sh`, ni `scripts/tests/recette-montee.test.sh`), aucun script de `kesh-gate-logs/`. **Jamais `pkill` ni
`killall`**, ni aucun `kill`. Aucun processus de fond, aucune sonde `bash -c`. Aucun `git commit`, `git push`, `git checkout`,
`git switch`, `git stash`, `git merge`, `git reset`, `git worktree`, `git tag`. Aucune commande `gh`. Rien sous `~/kesh/`
(production). Admis : `git diff`, `git show`, `git log`, `grep`, `sed -n`, `wc`, et `python3 -I -c` pour parser le YAML.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e49solde/validation-p4-ciblee.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding numéroté `V4-n`, sévérité, axe (H1 à H4), prémisse **citée** (sortie de `grep -nF`, ou ligne du diff), remède ;
- la liste des **axes exercés et non exercés** (H1 à H4), avec les commandes lancées pour chacun ; un « 0 finding » sans cette liste
  ne compte pas comme passe ;
- les commandes lancées et leurs sorties utiles (les greps de H1 en entier).
