# Revue de code de l'Epic 55 — consignes communes à toutes les passes

Ce fichier est lu par chaque passe de revue de code des stories de l'Epic 55 (« Navigation de l'interface »). Le prompt de
la passe (`55-<story>-code-review-prompt-p<N>.md`, à côté de la fiche) nomme la story, ses commits, son périmètre et ses
axes ; les règles ci-dessous valent pour toutes.

## Base

`<dépôt>` désigne le worktree de la branche et `<journaux>` le dossier des journaux de gate, hors dépôt : l'orchestrateur
donne leurs chemins en lançant la passe.

- Dépôt (worktree) : `<dépôt>`, branche `epic/55-navigation`. Lis-y le code à la
  tête (`git rev-parse HEAD`, à citer au rapport). La branche contient la PR de l'Epic 46 (`pr/46-rapports-groupes-v2`,
  avec `main` et les Epics 39 et 40), fusionnée après le développement de l'epic : une ligne de l'Epic 46 ou de `main`
  n'est pas un finding de la story, sauf si la story l'appelle ou la contredit.
- Fichier d'epic : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (décisions D1 à D17).
- Fiche de la story : nommée par le prompt. **Elle fait foi**, avec son T0 et son Dev Agent Record (décisions du mainteneur
  et de l'utilisateur versées) : une décision écrite n'est pas un finding ; son application fausse en est un.
- Méthode : celle de `bmad-code-review`, menée par toi seul, en trois couches successives — **Blind Hunter** (le diff seul,
  sans la fiche : ce qui est faux, fragile ou incohérent), **Edge Case Hunter** (chaque branche et chaque borne : URL
  invalide, réponse périmée, double clic, échec réseau, liste vide, exercice absent, clé d'API, société d'un autre),
  **Acceptance Auditor** (chaque critère de la fiche, au code et aux tests, ligne lue citée). Ne lance pas la skill :
  applique sa méthode.
- Diff de la story : `git show <commit>` ou `git diff <a>^ <b> -- <fichiers>`, commits nommés par le prompt. Juge le code
  **à la tête** quand un commit postérieur l'a retouché (le prompt le dit).

## Le manuel — obligatoire

Toute story de l'epic touche l'interface : vérifie `docs/manual/fr/user-manual.tex` contre le code **et** le PDF compilé
de la tête, déjà aplati : `<journaux>/s55-revue/manuel/<étiquette>/user-manual-aplati.txt` (le
prompt donne l'étiquette ; `COMMIT` à côté dit de quel commit il sort). Greppe le texte aplati, jamais le PDF brut. Le PDF
versionné du dépôt n'est pas régénéré dans les branches d'epic : seul compte le PDF compilé ci-dessus. Un manuel qui
promet ce que le code ne fait pas, ou omet ce qu'il fait de visible, est un finding (MEDIUM s'il trompe l'utilisateur).
`docs/api-external.md` aussi, si la story change l'API.

## Garde-fou `grep -nF`

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu ou la présence d'un anti-pattern, donne la
commande `grep -nF "<chaîne>" <fichier>` (`-F` obligatoire) et sa sortie copiée ; sans cette preuve, le finding est
écarté. Cite toujours `chemin:ligne`, à la tête.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo`, **ni** `npm` / `npx` (dont `vitest`,
`playwright`), **ni** `make`, `latexmk`, `xelatex`, **ni** aucun script (`scripts/*`, dont `scripts/prepare-release.sh`,
`scripts/mem-guard.sh`, `scripts/gate-ci.sh` ; rien de `kesh-gate-logs/*.sh`, dont `heavy.sh`), **ni** `docker`,
`mariadb`, `sqlx`, `curl`, `wget`, **ni** `git commit`, `checkout`, `switch`, `stash`, `restore`, `reset`, `merge`,
`rebase`, `fetch`, `pull`, `push`, `worktree`, `add`, **ni** `gh` (aucun appel, même en lecture), **ni** `sed -i`,
`perl -pi`, `tee`, `>`, `>>` ou toute redirection vers un fichier, **ni** `kill`, `pkill`, `killall`, **ni** `python3` ou
un autre interpréteur. Aucun mail. Ne touche ni aux instances 8888 et 8889, ni à aucune base. Permis : `cat`, `grep`,
`sed -n`, `head`, `tail`, `ls`, `wc`, `find` (sans `-exec` ni `-delete`), `git log`, `git show`, `git diff`, `git grep`,
`git blame`, `pdftotext` vers la sortie standard, la lecture d'images (captures citées par la fiche).

Les gates ne sont pas à rejouer : leurs journaux sont nommés par la fiche (`kesh-gate-logs/s55-dev/<story>/`) et, après
la fusion de la PR de l'Epic 46, par `kesh-gate-logs/s55-revue/fusion/` et `s55-revue/fin/`. Lis-les si un axe en dépend.

## Rapport

Un seul fichier, nommé par le prompt, en français. Pour chaque finding : `P<N>-<n>`, sévérité (CRITICAL, HIGH, MEDIUM,
LOW), couche (Blind, Edge, Acceptance), constat avec la ligne lue citée (`chemin:ligne`), conséquence, remède proposé,
**et s'il touche la production** (oui / non). Sévérités : CRITICAL = écriture comptable fausse, perte de données,
sécurité ; HIGH = comportement faux visible sur un chemin courant ; MEDIUM = comportement faux sur un chemin moins courant,
critère non tenu, test qui ne prouve pas ce qu'il dit, manuel qui trompe ; LOW = lisibilité, commentaire, test manquant
sans défaut, cosmétique.

En fin de rapport : décompte **CRITICAL / HIGH / MEDIUM / LOW** recompté depuis tes findings ; **axes exercés** et **non
exercés** (avec la raison) ; les commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
