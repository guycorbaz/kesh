# Prompt — revue de code P3 de la story 31-2 : passe CIBLÉE (une lentille T, modèle Sonnet)

Tu relis **la seule remédiation de la passe ciblée P2** de la story 31-2 de Kesh (comptabilité suisse) : le garde des
droits du MariaDB de dev, `scripts/check-dev-grants.sh`, et son test, `scripts/tests/check-dev-grants.test.sh`.
Worktree `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2`. Langue du rapport : français. Contexte frais.

**Le diff à relire, et lui seul** (diff unique aplati) :

```sh
git -C /home/gcorbaz/devel/kesh-31-2 diff 931df37a 28db7768 -- scripts/
```

Lis aussi les fichiers **entiers** à l'état final (`git -C /home/gcorbaz/devel/kesh-31-2 show 28db7768:scripts/check-dev-grants.sh`,
idem pour le test). Le finding auquel la remédiation répond : `/home/gcorbaz/devel/kesh-gate-logs/312-review-p2-T.md`
(T1 MEDIUM, `masquer` sensible à la casse et aux seules apostrophes ; T2 et T3 LOW, `#` dans une chaîne ou un
identifiant). Ce qui a tourné : le Change Log et le Dev Agent Record de
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md`. Pour le seul point 6 :
`git diff 931df37a HEAD -- docs/testing.md`.

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier — pas même temporaire —, hormis ton rapport.
  Compare par des commandes enchaînées (pipe).
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`).
- **N'exécute rien qui écrit, compile ou lance du code du dépôt** : ni `scripts/prepare-release.sh`, ni
  `scripts/test-fast.sh`, ni `scripts/check-dev-grants.sh`, ni `scripts/tests/*.test.sh`, ni `scripts/mem-guard.sh`,
  ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni **`python3`** (même « en lecture »), ni **`sed -i`**, ni `perl`, ni
  `bash` sur un script du dépôt. **Aucune commande `docker`** (conteneur partagé). Aucune commande `gh` qui écrit.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `wc`, et `awk`/`mawk` sur un
  **extrait passé par un pipe** (par exemple le programme awk extrait par `git show … | sed -n 'a,bp'`), sans rien écrire.

## La lentille T — la remédiation a-t-elle introduit le défaut suivant ?

1. **`masquer` fail-closed** (awk, `tolower`, troncature au premier `identified` ou `using`) : est-il POSIX (mawk et
   gawk) ? Peut-il **laisser passer** un secret (mot-clé coupé par un espace insécable, une tabulation, un saut de ligne
   dans une même valeur ; un secret placé AVANT le mot-clé ; une forme sans `identified` ni `using`, comme
   `PASSWORD('…')`) ? Peut-il **effacer de l'information nécessaire au diagnostic** — un privilège, un compte, `WITH
   GRANT OPTION` — dans les cas que le garde doit nommer (droits globaux, droit en plus, compte fautif) ? La ligne
   « droits globaux : attendu …, relevé … » est une seule ligne : une troncature dans l'attendu masque-t-elle le relevé ?
   Le multi-octet `<masqué>` et `substr` sous mawk ?
2. **Refus d'un marqueur de commentaire dans une chaîne ou un identifiant** (code 4, fonction `impair`, ligne
   `CHAINE<tab>`) : la parité est-elle comptée sur la bonne portion (`sortie` déjà retenue + le préfixe de `ligne`) ? Une
   ligne légitime du fichier réel (`git show 28db7768:scripts/mariadb-init/01-dev-grants.sql`) ou du fichier de
   référence du test peut-elle être refusée à tort (apostrophe dans un commentaire `--` en français, comme « l'entrypoint »,
   précédée d'un autre marqueur) ? Un `exit 4` dans le corps puis le passage par `END` : le code de sortie et la ligne
   `CHAINE` sont-ils garantis sous mawk et gawk ? `gsub` avec un motif d'un caractère `` ` `` ou `\047` : correct ?
   Une apostrophe échappée (`''` ou `\'`) dans une chaîne ?
3. **Les cas neufs** (secret en minuscules, guillemets doubles, `identified via … using` dans le relevé, instruction
   refusée en minuscules, `#` dans une chaîne suivi d'une instruction, `#` dans un identifiant) : chacun peut-il rougir,
   sur la bonne assertion ? Les assertions positives sur `<masqué>` sont-elles exactes ?
4. **Les cas existants** : le changement de `masquer` en modifie-t-il un en silence (une assertion `expect_out` devenue
   plus faible, une `expect_no_out` devenue triviale) ?
5. **Mutations** : le Dev Agent Record déclare M32 (retour à l'ancien `sed`), M33 (casse), M34–M36 tuées ; une mutation
   évidente de la remédiation survivrait-elle (seuil `u < i` inversé, `substr(…, i)` au lieu de `i - 1`, parité sur
   `substr(ligne, 1, m - 1)` seul sans `sortie`) ?
6. **Le texte** (en-tête du garde, `docs/testing.md`) : « toute forme de secret reste masquée » et l'angle mort de la
   chaîne ouverte sur une ligne précédente disent-ils ce que fait le code, ni plus ni moins ?

## Garde-fous

- **Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée** (`-F` obligatoire). Pour un
  bloc, `grep -nFA <N>`.
- **Lis le fichier à l'état final** avant d'affirmer qu'un patch manque : les numéros de ligne d'un hunk ne sont pas ceux
  du fichier final.
- Un reproche d'« absence d'un livrable » se vérifie d'abord au `grep -nF`.
- **Vérifie aussi ton remède** : un correctif proposé ne doit pas casser un cas existant.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/312-review-p3-T.md` :

- bilan en tête : `CRITICAL n / HIGH n / MEDIUM n / LOW n` ;
- findings numérotés `T1…`, chacun avec sévérité, fichier:ligne (état final), **preuve copiée**, correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (parmi les six ci-dessus), avec la raison ;
- rien d'autre n'est écrit nulle part.
