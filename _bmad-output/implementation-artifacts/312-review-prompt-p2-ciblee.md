# Prompt — revue de code P2 de la story 31-2 : passe CIBLÉE (une lentille T, modèle Haiku)

Tu relis **la seule remédiation de la revue de code P1** de la story 31-2 de Kesh (comptabilité suisse) : le garde des
droits du MariaDB de dev, `scripts/check-dev-grants.sh`, et son test, `scripts/tests/check-dev-grants.test.sh`.
Worktree `/home/gcorbaz/devel/kesh-31-2`, branche `story/31-2`. Langue du rapport : français. Contexte frais.

**Le diff à relire, et lui seul** (diff unique aplati, pas la séquence de commits) :

```sh
git -C /home/gcorbaz/devel/kesh-31-2 diff 453a5d8c 365940a9 -- scripts/
```

Pour situer un passage, lis le fichier **entier** à l'état final : `git -C /home/gcorbaz/devel/kesh-31-2 show
365940a9:scripts/check-dev-grants.sh` (idem pour le test). Les findings de la P1 auxquels la remédiation répond sont
dans `/home/gcorbaz/devel/kesh-gate-logs/312-review-p1-{B,E,A}.md` ; le Change Log et le Dev Agent Record de
`_bmad-output/implementation-artifacts/31-2-outillage-dit-vrai.md` disent ce qui a été corrigé et ce qui a tourné.

## Interdits absolus

- **Lecture seule stricte.** N'écris, ne modifie, ne crée aucun fichier — pas même temporaire —, hormis ton rapport.
  Compare par des commandes enchaînées (pipe), jamais par un fichier intermédiaire.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`).
- **N'exécute rien qui écrit, compile ou lance du code** : ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`,
  ni `scripts/check-dev-grants.sh`, ni `scripts/tests/*.test.sh`, ni `scripts/mem-guard.sh`, ni `cargo`, ni `npm`,
  ni `npx`, ni `make`, ni **`python3`** (même « en lecture »), ni **`sed -i`**, ni `perl`, ni `bash` sur un script du
  dépôt. **Aucune commande `docker`** (le conteneur `kesh-mariadb-dev` est partagé). Aucune commande `gh` qui écrit.
- Permis : `git diff`, `git show`, `git log`, `git grep`, `grep`, `sed -n`, `cat`, `wc`, et `awk`/`mawk` **sur un
  extrait passé par un pipe** (comme l'a fait la lentille B de la P1), sans rien écrire.

## La lentille T — la remédiation a-t-elle introduit le défaut suivant ?

Le motif mesuré du dépôt : *la sévérité se déplace vers ce qu'on vient d'écrire.* Cherche donc, dans ce seul diff :

1. **Le filtre `masquer`** (nouveau) : ses trois substitutions `sed -E` sont-elles correctes (échappements de `\(`, `'`,
   ordre) ? Peut-il **effacer de l'information utile** (un privilège, `WITH GRANT OPTION`, un nom de base) d'une ligne
   affichée ? Une forme de secret lui échappe-t-elle encore ? Toutes les sorties qui affichent une donnée passent-elles
   par lui (droits globaux, manquant, en plus, instruction refusée) ?
2. **Le refus d'instruction** passe désormais par une ligne `REFUS<tab>…` sur la sortie standard et le code 3 :
   `sed -n 's/^REFUS\t//p'` (GNU) retrouve-t-il la ligne ? Le cas `rc -eq 3` précède-t-il bien le cas général ? Une
   instruction `GRANT` légitime peut-elle commencer par `REFUS` ?
3. **`#` reconnu partout** dans l'awk : la logique « le plus tôt de `/*`, `--`, `#` » est-elle juste quand `#` précède
   `/*` sur la même ligne, ou figure dans un bloc `/* … */` ouvert ? Un `#` dans un nom entre backticks (le fichier réel
   n'en a pas) ?
4. **`tr -d '\r'` et le filtre des lignes vides** sur le relevé : ordre et effet sous `set -euo pipefail`.
5. **`set +x`** : est-il bien la première commande exécutée, avant toute lecture du mot de passe ?
6. **Le test** : chacun des cas neufs (C-31-2-16 conforme et `USAGE` en trop, lignes vides et `\r`, `#` en milieu de
   ligne, `grant` minuscule, secret en clair, instruction refusée portant un secret, `bash -x`, compose absent, clé en
   double, clé vide, valeur entre guillemets et commentaire) **peut-il rougir**, et sur la bonne assertion ? Un cas
   hérite-t-il d'un état du cas précédent (fichier modifié par `printf >>`, `DOCKER_CMD`, `STUB_PS`) ? Le cas `bash -x`
   duplique la commande de `lancer` : divergence ? Les nouvelles bornes de câblage (`grep -nE '^[^#]*…'`, `|| true`,
   `fail "borne introuvable"`) et l'exigence d'invocation au premier niveau sont-elles justes contre le vrai
   `scripts/test-fast.sh` (`sed -n 45,70p scripts/test-fast.sh`) ?
7. **Les nouveaux angles morts écrits** (en-tête du garde, `docs/testing.md`) disent-ils ce que fait le code, ni plus ni
   moins ? `git diff 453a5d8c HEAD -- docs/testing.md CLAUDE.md` est permis pour ce seul point.

## ⚠️ Garde-fous propres à cette passe (Haiku)

- **Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée** (`-F` obligatoire : le code
  porte des métacaractères). Pour un bloc multi-ligne, `grep -nFA <N>` sur une ligne discriminante.
- **Lis le fichier à l'état final** (`git show 365940a9:<chemin>`) avant d'affirmer qu'un patch « n'a pas été
  appliqué » : les numéros de ligne d'un hunk ne sont pas ceux du fichier final.
- **Un MEDIUM qui reproche l'absence d'un livrable** se vérifie d'abord au `grep -nF` : s'il est prescrit ailleurs ou
  hors de ce diff, ce n'est pas un défaut de la remédiation.
- **Vérifie aussi ton remède** : un correctif proposé ne doit pas casser un cas existant.

## Rapport

Écris ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/312-review-p2-T.md` :

- bilan en tête : `CRITICAL n / HIGH n / MEDIUM n / LOW n` ;
- findings numérotés `T1…`, chacun avec sévérité, fichier:ligne (de l'état final), **preuve copiée**, correctif proposé ;
- **la liste des axes réellement exercés ET de ceux qui ne l'ont pas été** (parmi les sept ci-dessus), avec la raison —
  un « 0 finding » sans cette liste ne compte pas comme passe ;
- rien d'autre n'est écrit nulle part.
