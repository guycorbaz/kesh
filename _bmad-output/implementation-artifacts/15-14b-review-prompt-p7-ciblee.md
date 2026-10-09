# 15-14b — revue de code P7, passe ciblée (une lentille, Haiku)

**Périmètre : la seule remédiation de la P6**, `git diff 17883fa4^..92fdf518` (message « revenir » qui liste les dossiers de `avant-restauration/` au lieu de nommer `$SECURITE` ; G16 (c) lu sur plusieurs lignes ; recette 7-quater ; manuel, CHANGELOG, testing.md). Ne relis pas le reste de la story.

Lis d'abord `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` (consignes 4 et 5) et le rapport P6 `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p6-C.md`.

Lentille unique — **chasseur de régressions** : ce que le patch P6 a pu casser ou laisser incohérent. Axe 0 prioritaire : le message d'échec de l'étape 6 dit-il désormais juste dans tous les cas (premier passage, passages suivants, base vide sans dump de sécurité à ce passage, dossier avant-restauration/ absent ou vide, noms avec espaces) ? La commande qui liste les dossiers peut-elle elle-même échouer et, sous `set -e`, masquer le message ?
1. Refus « Kesh en marche » : tout état autre que `exited`/`created`/`dead` refuse-t-il réellement (`restarting`, `paused`, `running`, `removing`) ? Le filtre est-il cohérent avec ce que G16 (c) impose, et G16 rougirait-il si on revenait à `status=running` ?
2. Sonde à trois états (absente / vide / avec tables) : chaque branche fait-elle ce que le manuel dit ? Une erreur de la sonde elle-même (base inaccessible, mot de passe faux) peut-elle être prise pour « vide » et faire recharger sans dump de sécurité ?
3. Verrou et trap : libération sur sortie normale, erreur, SIGTERM ; aucune trap n'efface un dump de sécurité ou le dump source.
4. Manuel (`docs/manual/fr/admin-manual.tex` et le PDF aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG, recette : disent-ils exactement ce que fait le script livré ?
5. Grep du symptôme : reste-t-il ailleurs dans le dépôt une trace de `status=running` ou du comportement d'avant la P4/P5 ?

**Interdits** : aucune écriture, aucune commande qui compile, exécute un script du dépôt (`scripts/*`, `prepare-release.sh`, `recette.sh`) ou touche Docker. Lecture seule (`git diff`, `git show`, `grep`, `sed -n`, `pdftotext`).

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p7-C.md` — findings classés CRITICAL/HIGH/MEDIUM/LOW avec fichier:ligne et sortie `grep -nF` copiée pour toute affirmation de présence ou d'absence ; **liste des axes exercés et non exercés**.
