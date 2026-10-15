# 29-9 — passe ciblée P5 de validation (une lentille)

**Objet** : le seul commit `5d930a20` (remédiation de la passe ciblée P4, rapport
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p4-ciblee.md`). Ne pas relire la story entière :
chercher ce que **cette remédiation** a introduit ou laissé.

**Lecture seule stricte** : aucun `git checkout`/`switch`/`stash`/`reset`, aucune commande qui écrit,
compile ou exécute un test ou un script (`scripts/prepare-release.sh` compris) ; aucune redirection
vers un fichier, aucun fichier temporaire — comparaisons en pipe. Permis : `git show`, `git diff`,
`git log`, `git -C … show <rev>:<chemin>`, `grep`, `sed -n`, `pdftotext … -`. Seul fichier écrit :
le rapport `/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p5-ciblee.md`.

## Axes

1. **Chaque correction P4 tient** (P4-1 à P4-8) : la lire dans le fichier final, citer `grep -nF`.
2. **P4-3 au code** : relire `frontend/**/i18n-entrees-a-variables.test.ts` et
   `i18n-repli-divergent-actif.test.ts` (et la troisième garde citée par l'AC11) ; la forme prescrite
   (deux `i18nMsg` littéraux sous un ternaire) est-elle bien retenue par chacune ? Le test neuf
   prescrit est-il réalisable sans toucher la borne des cinq clés de la 23-5 ? La mutation M30
   le fait-elle rougir ?
3. **M28, M29, V20, V21** : chaque mutation est-elle tuée par une vérification nommée ? M29 (`!= null`)
   est-elle bien distincte du code prescrit, et V20 couvre-t-elle `''` sans contredire la règle de
   véracité écrite ?
4. **Propagation** : grep des valeurs (`\b(3|4)\b` près de « défauts »/« issues », `C-29-9-2[0-4]`,
   `M(2[8-9]|30)`, `#678`) sur la fiche, le registre, le fichier d'epic, `sprint-status.yaml` :
   résidus vivants ? entrée datée réécrite à tort ?
5. **Recomptes** : 11 AC, T0–T10, V1–V22, M1–M30, C-29-9-1 à 24 — recompter à la source.
6. **Commande du mot-clé de fermeture** (l.22-25 de la fiche) : l'exécuter mentalement contre
   `git log main..story/29-9` (lecture) ; rend-elle bien vide, et attrape-t-elle un vrai `closes #N` ?
7. **Manuel** : la remédiation change-t-elle un texte visible ? Si oui, contrôler `docs/manual/` et
   le PDF aplati (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

## Rapport

Findings CRITICAL/HIGH/MEDIUM/LOW, chacun avec fichier:ligne et sortie de commande copiée ;
**liste des axes exercés et non exercés** (un « 0 » sans elle ne compte pas) ; décompte final.
