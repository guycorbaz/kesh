# Prompt — passe ciblée P2 de la revue documentaire, release 0.13.0 (Haiku)

Une seule lentille, braquée sur **le seul commit de remédiation** `468a3628`
(`git -C /home/gcorbaz/devel/kesh-release show 468a3628 -- CHANGELOG.md README.md website docs/manual/fr/admin-manual.tex`).
Contexte : les rapports P1 `/home/gcorbaz/devel/kesh-gate-logs/release-0.13.0-review-p1-A.md` et `-B.md`.
Langue : français.

## Interdits absolus

Lecture seule. N'exécute AUCUN script du dépôt (`scripts/prepare-release.sh`, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`), ni `make`, `latexmk`, `cargo`, `npm`. Aucun `git` qui écrit, aucune commande
`gh` qui écrit. Permis : `git show`, `git diff`, `grep`, `sed -n`, `cat`, `pdftotext … -` (sortie standard).
N'écris rien hors de ton rapport.

## Axes

1. **A1** — la phrase neuve du CHANGELOG sur l'installation à neuf est-elle vraie pour
   `docker-compose.yml` et pour `docker-compose.prod.yml` (lire les deux compose et `.env.example`
   à HEAD) ? Contredit-elle la liste numérotée qui la précède ?
2. **A3** — la phrase neuve du manuel (voie de retour) : est-elle cohérente avec la procédure de
   mise à jour et `sec:maj-0-13` (re-télécharger le compose) ? Un compose de la 0.12.1 avec
   l'image `:0.12.1` démarre-t-il sur la base restaurée (aucune variable que la 0.12.1 refuserait) ?
   Le PDF aplati (`docs/manual/fr/admin-manual.pdf`) porte-t-il la phrase ?
3. **A5, B1** — les deux retouches (site, README) sont-elles justes ? Numéros #232, #235.
4. La remédiation a-t-elle introduit une incohérence ailleurs (grep des formulations touchées
   sur `CHANGELOG.md`, `README.md`, `website/`, `docs/manual/fr/*.tex`) ?

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/release-0.13.0-review-p2-F.md` : findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), fichier:ligne, **sortie `grep -nF`/`sed -n` copiée** comme preuve ;
**liste des axes exercés ET non exercés** (un « 0 » sans cette liste ne compte pas).
