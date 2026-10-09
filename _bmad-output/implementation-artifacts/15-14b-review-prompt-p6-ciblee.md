# 15-14b — revue de code P6, passe ciblée (une lentille, Haiku)

**Périmètre : la seule remédiation de la P5**, `git diff b4d64596^..cf2a24c6` (fix de `scripts/synology/kesh-restore.sh`, manuel, CHANGELOG, recette, garde G16 (c)). Ne relis pas le reste de la story.

Lis d'abord `/home/gcorbaz/.claude/projects/-home-gcorbaz-devel-kesh/memory/consignes-agents-epic15.md` (consignes 4 et 5) et les rapports P5 `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p5-{A,B,E}.md`.

Lentille unique — **chasseur de régressions** : ce que le patch P5 a pu casser ou laisser incohérent.
1. Refus « Kesh en marche » : tout état autre que `exited`/`created`/`dead` refuse-t-il réellement (`restarting`, `paused`, `running`, `removing`) ? Le filtre est-il cohérent avec ce que G16 (c) impose, et G16 rougirait-il si on revenait à `status=running` ?
2. Sonde à trois états (absente / vide / avec tables) : chaque branche fait-elle ce que le manuel dit ? Une erreur de la sonde elle-même (base inaccessible, mot de passe faux) peut-elle être prise pour « vide » et faire recharger sans dump de sécurité ?
3. Verrou et trap : libération sur sortie normale, erreur, SIGTERM ; aucune trap n'efface un dump de sécurité ou le dump source.
4. Manuel (`docs/manual/fr/admin-manual.tex` et le PDF aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), CHANGELOG, recette : disent-ils exactement ce que fait le script livré ?
5. Grep du symptôme : reste-t-il ailleurs dans le dépôt une trace de `status=running` ou du comportement d'avant la P4/P5 ?

**Interdits** : aucune écriture, aucune commande qui compile, exécute un script du dépôt (`scripts/*`, `prepare-release.sh`, `recette.sh`) ou touche Docker. Lecture seule (`git diff`, `git show`, `grep`, `sed -n`, `pdftotext`).

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/15-14b-review-p6-C.md` — findings classés CRITICAL/HIGH/MEDIUM/LOW avec fichier:ligne et sortie `grep -nF` copiée pour toute affirmation de présence ou d'absence ; **liste des axes exercés et non exercés**.
