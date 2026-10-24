# Prompt — validation P6 ciblée, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Passe ciblée, une seule lentille**, contexte frais, modèle **Haiku** (passe ciblée de fin de
boucle, `CLAUDE.md` § *Review Iteration Rule* : Haiku est réservé aux passes ciblées). Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md` (C-31-4-1 à 18 ; **C-31-4-15, 16, 17 et 18 font loi**). Règles : `CLAUDE.md`,
`docs/agents-orchestration.md` § 6. Rapport de la P5 (ce qui a été remédié) :
`/home/gcorbaz/devel/kesh-gate-logs/314-validate-p5-T.md`.

Rapport, **un seul fichier, écrit d'un seul jet** : `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p6-T.md`.

## Ton périmètre : la seule remédiation de la P5

**Une lentille (T), sur un seul diff** : le commit de remédiation P5, `2d3394aac681c30bf77554c3d3e4ef328d4e087b`. Lis-le :

```sh
git -C /home/gcorbaz/devel/kesh-31-4 show 2d3394aac681c30bf77554c3d3e4ef328d4e087b --stat
git -C /home/gcorbaz/devel/kesh-31-4 show 2d3394aac681c30bf77554c3d3e4ef328d4e087b -- _bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md
git -C /home/gcorbaz/devel/kesh-31-4 show 2d3394aac681c30bf77554c3d3e4ef328d4e087b -- _bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md
```

Ce commit est un **diff unique** sur un seul niveau : lis les numéros de ligne dans le fichier à `HEAD` de la branche, non dans
le diff. Le reste de la fiche n'est relu que pour **vérifier ce que ce diff touche**. Un défaut antérieur que ce diff n'a ni
créé ni aggravé se signale en **une ligne « hors périmètre »**, sans sévérité.

**Cherche ce que la remédiation P5 a introduit ou laissé** :

1. **T1 — R3 propagé partout.** Le secret réel de la production entre dans `/var/tmp` dès l'étape 0, dans
   `$COPIE/telechargements/`. Cherche tout énoncé qui dit encore l'inverse : « aucun secret réel dans `/var/tmp` »,
   « téléchargé par le navigateur y est déplacé », un exemplaire « créé hors de `$COPIE` » présenté comme le flux nominal.
   Le geste qui pose `source.keshbackup` (`mv` depuis `$COPIE/telechargements/`, puis `chmod 0600`, après le contrôle 9) est-il
   écrit, dans le bon ordre, sans contradiction avec le contrôle 3, l'AC4, l'AC6, *La marche* et l'AC7 ? La garantie d'un
   arrêt est-elle partout « la destruction de `$COPIE` » ?
2. **T2 — deux critères, bien séparés.** Au manuel (AC4, AC5) : « 0 pièce manquante », « autant de pièces restaurées que
   l'export en a annoncé », écriture récente et liste des factures (étape 5). Au développement (T1 de la fiche, Dev Notes) :
   *N* ≥ 1. Reste-t-il, **dans un passage qui décrit ce que dit le manuel**, un critère ou un renvoi qui ne vaut que pour
   la sauvegarde synthétique (« la pièce de la T1 », « l'écriture de la T1 », *N* ≥ 1 comme condition d'échec du lecteur) ?
3. **T3 à T7 — chacun contre son texte.** T3 (« directement, si le navigateur le permet », constat (s) : plus de « jamais ») ;
   T4 (AC7 (d) : le décompte annoncé des précisions égale le nombre de puces ; arrêt au contrôle 3 ; déclencheur du `mkdir`
   refusé ; « contrôles 4 à 8 » ; repli (i) → (ii) si `down -v` échoue ; (i) rangé à l'AC10) ; T5 (essai (iii) : « passe le
   `case`, puis la garde le refuse au `-d` » ; « six sorties » reste juste) ; T6 (`&&` entre les deux gestes, et le
   cas « avant le premier `up` » ne contredit pas l'AC7 (b)) ; T7 (G14 : la justification écrite est-elle exacte contre
   `crates/kesh-api/tests/configuration_transmise.rs` l. 2660-2692 ?).
4. **Propagation (grep de la valeur, pas de la formulation).** `grep -nF` (un motif à la fois) de : `aucun secret`, `hors de`,
   `déplacé`, `jamais par le dossier`, `N* ≥ 1`, `contrôles 3 à 8`, `passe la garde`, `Trois précisions`, `Quatre précisions`.
   Un site périmé qui dit encore l'ancien état est un finding. Les sites **datés** (Change Log, registre passé, annotations)
   ne se corrigent pas : ils se complètent.
5. **Recomptes, commandes à l'appui** : AC (13 numéros, dont AC8 et AC11 retirés) ; tâches (9) ; contrôles de l'AC3 (9) ;
   constats (a) à (u) (21 lettres, (e) retirée) ; choix C-31-4-1 à 18 (`grep -c '^### C-31-4-'` sur le registre). Le
   Change Log a-t-il **réécrit** une entrée passée (`git show 2d3394aa` : lignes `-` du Change Log, du registre, de
   `sprint-status.yaml`) ?
6. **Aucun outillage nouveau** (script, garde, test, fixture, mutation). Ce que le commit prescrit au développement n'existe pas
   encore : ne le cherche pas dans le dépôt.

⛔ **Ne propose AUCUN outillage nouveau.** Le livrable de la story est la procédure exécutée (C-31-4-15). Un trou se signale
comme angle mort à écrire dans la fiche ou geste à journaliser en T3/T8.

⛔ **Garde-fous Haiku (grep ground-truth).** Pour tout finding qui affirme une **présence** ou une **absence** (« tel énoncé
n'a pas été corrigé », « tel geste n'est pas écrit »), copie dans le rapport la sortie de `grep -nF "<chaîne>" <fichier>` qui
l'établit, avec le numéro de ligne **du fichier à `HEAD`**. Sans sortie copiée, l'affirmation n'existe pas. Ne conclus jamais
à l'absence d'un patch d'après les numéros de ligne d'un diff. Ce que la fiche **prescrit au développement** (journal, manuel,
PDF, gardes G14/G15/G16/G19, `make fr`) **n'existe pas encore** : son absence au dépôt n'est pas un finding. Un comportement de
Docker, de Compose ou de la station que tu ne peux pas exécuter se dit *non exécuté*.

## Ce que tu rends

Findings `T1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, preuve (`grep -nF` copié), correction proposée,
et **la provenance** (introduit par la remédiation P5, ou antérieur). ⛔ **La liste des axes exercés ET non exercés** — sans
elle, « 0 finding » ne compte pas. Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.
Tu ne conclus pas toi-même à la clôture : l'orchestrateur décide (critère : 0 MEDIUM ou plus, remédiation sans ligne de code
de production).

## Interdits

⛔ **N'écris aucun fichier hors de ton rapport**, par un **chemin absolu** sous `/home/gcorbaz/devel/kesh-gate-logs/`. Le dépôt
principal `/home/gcorbaz/devel/kesh` est en **lecture seule**. **Aucune écriture par chemin relatif** (`-o`, `>`, `cp`, `tee`) ;
chaîner par `&&`, jamais par `;` après un `cd` ; si une commande préalable échoue, **arrête-toi**.

Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, **`docker` (aucune sous-commande)**, **`curl`**,
**`mkdir`**, `realpath` et `findmnt` sur un chemin à créer, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture, `sed -i`. Aucune donnée réelle de Guy (rien de
`~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
