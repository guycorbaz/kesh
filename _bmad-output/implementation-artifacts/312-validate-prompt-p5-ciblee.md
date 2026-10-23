# Prompt — validation P5 CIBLÉE de la spec, Story 31-2

*Versionné le 2026-10-24. **Une seule lentille**, modèle **Haiku** (passe ciblée de fin de boucle, `CLAUDE.md` § *La passe
ciblée*), contexte frais, lecture seule stricte. Elle est braquée sur **le seul commit de la remédiation P4**, non sur la fiche
entière : la sévérité se déplace vers ce qu'on vient d'écrire. Une boucle ne se clôt que sur une passe à 0 MEDIUM+ ; ce prompt
ne dit pas qu'elle l'est.*

Dépôt `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2` (ne change pas de branche). **Le diff à relire, et lui seul** :

```
git diff HEAD~1..HEAD --stat
git diff HEAD~1..HEAD
```

(`HEAD` est le commit de **prompt** si celui-ci a été posé après la remédiation ; dans ce cas la remédiation est
`git log --oneline -3`, son sujet commence par « docs(31-2): validation P4 remédiée » : utilise alors
`git diff <ce hash>~1..<ce hash>`.) Il touche la fiche `_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`, le
registre `epic-31-choix-autonomes-312.md` (amendements du 2026-10-24) et `sprint-status.yaml`. Les rapports de la P4 sont
`/home/gcorbaz/devel/kesh-gate-logs/312-validate-p4-{R,F}.md` : ce que la remédiation prétend corriger.

## Lentille unique — régression de la remédiation P4

Cherche **ce que ce diff a cassé ou laissé incohérent**, non ce que la fiche contenait avant. Axes, dans cet ordre :

1. **Masquage et libellés** (1b « Attendu », AC1, AC2, AC3). Le masquage du condensé s'applique à **toute ligne `ON *.*`**, par
   un jeton `<condensé masqué>` espace précédente comprise ; l'affichage retire la clause. Est-ce **implémentable sans deviner** ?
   Une contradiction entre « la comparaison porte sur les lignes normalisées (jeton compris) » et « la sortie ne contient jamais
   le jeton » ? Les assertions **positives** d'AC3 (`ALL PRIVILEGES`, `PROCESS`, `` `kesh`@`localhost` ``, `autre`) sont-elles
   cohérentes avec ce que le libellé affiche réellement ? Les fixtures « remplaçantes » sont-elles cohérentes avec le cas
   `WITH GRANT OPTION` et avec le cas « droit en moins » ?
2. **Test hermétique** (AC3, 1b « Une branche qui modifie… », AC5, registre). La fiche dit : mini-dépôt pour **chaque** cas,
   `01-dev-grants.sql` de référence écrit par le test ; seule l'assertion de câblage lit le vrai `test-fast.sh`. Reste-t-il une
   phrase qui dit encore le contraire (« les cas qui changent l'attendu », « lisent le vrai fichier ») ? Le conseil d'AC7 (le
   garde réel contre le conteneur) reste-t-il la seule chose qui lit le vrai fichier de droits ?
3. **Mutations M1 à M20** : chacune a-t-elle son cas dans AC3/AC4, chaque cas d'AC3/AC4 sa mutation ? M1 est rattachée au
   cas « hors niveau global » ; M14 à M20 sont neuves. Une mutation dont l'« Attendu » ne rougirait pas (ex. M20, M19) ?
   Recompter depuis la source : AC, tâches, mutations ; vérifier que **chaque occurrence** de « M13 », « M1–M », « M6–M »,
   « 13 mutations », « 20 mutations » est cohérente (sauf entrées **datées** du Change Log, qu'on ne réécrit pas).
4. **Ligne nue d'AC4** : la regex `^[[:space:]]*(bash )?(\./)?scripts/check-dev-grants\.sh[[:space:]]*$` est-elle compatible avec
   l'assertion « exactement une ligne non commentée contenant `check-dev-grants.sh` » et avec la forme réelle des lignes de
   `scripts/test-fast.sh` (lis-le) ? Aucun `echo` existant ne devrait nommer le script.
5. **Remise en état en texte arrêté** (1b « Sortie en cas d'échec », AC2, AC5) : les commandes citées (`docker compose -f
   docker-compose.dev.yml up -d --force-recreate mariadb`, `sqlx migrate run --source crates/kesh-db/migrations`,
   `scripts/seed-dev-db.sql`) existent-elles ? Contredisent-elles le § 2 de `docs/agents-orchestration.md` ou les interdits de la
   fiche (« Ce que le dév ne fait pas ») ?
6. **Inventaires ajoutés** : `docs/testing.md:52` et `:101` (AC5), la forme de `git grep -nI "429\|anonymement" -- '*.md'
   '*.yml'` (AC14) : ré-exécute-les **en lecture** et dis ce qu'ils rendent (`CLAUDE.md:275` attendu).
7. **Extraction du fichier de droits** (1b « Attendu ») : la règle écrite (commentaires `--`, `#`, `/* */`, `;`, casse,
   multilignes ; instruction contenant `GRANT` sans commencer par lui refusée) est-elle non ambiguë sur le **vrai**
   `scripts/mariadb-init/01-dev-grants.sql` (lis-le) ? Le fichier réel serait-il refusé par erreur ?
8. **Propagation** : greper les **valeurs** que la remédiation a changées — `ligne USAGE`, `IDENTIFIED BY PASSWORD`, `-u kesh`,
   `MARIADB_USER`, `M13`, `13 mutations`, `healthy`, `hermétique` — sur la fiche, le registre 312 et le registre commun.
9. **Manuel** : la remédiation ne change aucun texte visible, mais confirme-le : `docs/manual/` (`.tex` ; PDF aplati par
   `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) parle-t-il du garde, de la CI, des droits de dev ?

## Garde-fous Haiku (obligatoires)

- ⛔ **Toute affirmation de PRÉSENCE ou d'ABSENCE** d'un texte dans la fiche s'appuie sur une sortie **`grep -nF` copiée** dans le
  rapport (`-F` obligatoire : les motifs contiennent `*`, `.`, `[`, `$`, `\`). Un finding CRITICAL/HIGH/MEDIUM sans cette
  sortie est rejeté par l'orchestrateur.
- ⛔ **La fiche prescrit au dév ce qui n'existe pas encore** : `scripts/check-dev-grants.sh`, `scripts/tests/check-dev-grants.test.sh`,
  `crates/kesh-db/tests/check_dev_grants_script.rs`, `smoke.spec.ts`, le job `e2e-smoke` n'existent **pas** — leur absence n'est
  pas un finding. Ne signale que ce que la **fiche** dit de contradictoire, d'incomplet ou de faux contre ce qui existe.
- Les numéros de ligne d'un diff sont ceux du fichier **post-commit** ; vérifie par `grep -nF` ou `sed -n` sur le fichier réel
  avant de citer une ligne.
- Un point que tu qualifies de « robuste » doit avoir été **lu** (cité), non supposé. Une « catégorie d'erreur » (confondre
  ce que la fiche prescrit et ce qui existe) est un finding invalide.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie copiée), correction proposée.
⛔ **La liste des axes exercés ET non exercés** (parmi les neuf ci-dessus) — un « 0 finding » sans elle ne compte pas ; l'orchestrateur
reprendra les axes non exercés.
⛔ **Écris ton rapport complet** dans `/home/gcorbaz/devel/kesh-gate-logs/312-validate-p5-T.md` — c'est le seul fichier autorisé —
et rends comme dernier message **seulement** : ce chemin, le bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ Lecture seule stricte. Aucun `git checkout`/`switch`/`stash`/`reset`/`add`/`commit`, sous aucune forme. Aucune commande qui
écrit, compile ou exécute : `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/check-dev-grants.sh` et
`scripts/tests/*.test.sh`), `make`, `latexmk`, `cargo`, `sqlx`, `npm`, `npx`, `node`, `python3`, `gh issue create|comment|edit|close`,
toute requête SQL d'écriture, tout `GRANT`/`REVOKE`, `pkill`/`killall`. **Aucun `docker` d'aucune sorte** (ni `ps`, ni `exec`, ni
`inspect`), aucun restart du conteneur. Aucun fichier temporaire : comparer par pipe. Autorisés : `grep -nF`, `sed -n`, `cat`,
`ls`, `git log|show|diff|ls-files|grep`, `gh api` en lecture, `pdftotext … -` vers la sortie standard.
