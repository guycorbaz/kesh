# Prompt — validation P5 ciblée, Story 31-4 (la répétition à blanc d'une clôture)

*Versionné le 2026-10-24. **Passe ciblée, une seule lentille**, contexte frais, modèle **Opus** (rotation après la P4 en
Sonnet, `CLAUDE.md` § *La passe ciblée* et § *Review Iteration Rule*). Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` ;
registre : `epic-31-choix-autonomes-314.md` (C-31-4-1 à 17 ; **C-31-4-15, 16 et 17 font loi**, C-31-4-17 remplaçant le point 1
de C-31-4-16). Règles : `CLAUDE.md`, `docs/agents-orchestration.md` § 6. Rapports de la P4 (à lire pour savoir ce qui a été
remédié) : `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p4-R.md` et `…-p4-F.md`.

Rapport, **un seul fichier, écrit d'un seul jet** : `/home/gcorbaz/devel/kesh-gate-logs/314-validate-p5-T.md`.

## Ton périmètre : la seule remédiation de la P4

**Une lentille (T), sur un seul diff** : la remédiation P4, commit `51095569`. Lis-le avec :

```sh
git -C /home/gcorbaz/devel/kesh-31-4 show 51095569 -- _bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md
git -C /home/gcorbaz/devel/kesh-31-4 show 51095569 -- _bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md
git -C /home/gcorbaz/devel/kesh-31-4 show 51095569 --stat
```

Le reste de la fiche n'est relu que pour **vérifier ce que ce diff touche** (contradiction avec un énoncé voisin, renvoi
déplacé, nombre non recompté). Un défaut de la conception antérieure, que ce diff n'a ni créé ni aggravé, se signale en
**une ligne « hors périmètre »**, sans sévérité.

**Le motif qui justifie cette passe** (`CLAUDE.md` § *La passe ciblée*) : la sévérité se déplace vers ce qu'on vient
d'écrire. Cherche ce que **la remédiation P4 a introduit** ou laissé :

1. **R3 — l'étape 0 après le contrôle 3.** La relocalisation est-elle cohérente partout ? Grep des renvois à « étape 0 »
   (`grep -nF "étape 0"`) : AC3 (contrôles 3, 4, 9), AC4, AC6, *La marche*, AC7 (d). Le contrôle 4 (« tag de la version
   relevée sur la production ») et le contrôle 9 (« version relevée à l'étape 0 ») restent-ils bien ordonnés — la version
   est-elle relevée **avant** le contrôle 4 ? Un chemin par lequel un fichier sensible de la production atterrirait quand
   même dans le dossier de téléchargements **sans** que l'arrêt le dise ? `$COPIE/telechargements/` : créé (contrôle 3,
   `mkdir -m 0700` sans `-p`) **avant** l'étape 0 ? Le contrôle de vacuité précède-t-il le `mkdir` ? Une contradiction
   avec l'énoncé « fichier téléchargé par le navigateur y est déplacé avant usage » ?
2. **R2 — l'arrêt, trois cas (AC7 (d)).** Le cas « dossier existant » (affecter `COPIE`, appliquer (b) avec sa garde) est-il
   praticable — la garde accepte-t-elle un dossier d'une **autre date** ? `down -v` avec des `-f` visant des fichiers qui
   ont peut-être disparu ? Le cas « dossier disparu » (identifiants rendus par les filtres d'étiquette) : le geste est-il
   **borné** (aucun `prune`, aucun nom écrit à la main, étiquette `kesh-repetition` seule) ? Peut-il atteindre un objet
   d'une autre pile ? Que fait `docker rm -f $(…)` si un filtre ne rend **rien** ? Le « éprouvé à blanc » (constat (u)) se
   joue-t-il sans danger ? L'arrêt avant `$COPIE` et l'arrêt avant le premier `up` sont-ils justes (rien à détruire ; `(b)`
   sans `down -v`) ?
3. **R1/F1 — la source préparée à l'écran (T1, AC4, AC5, État des lieux).** Les gestes sont-ils **exacts** contre le code :
   l'exercice suivant se crée où (*Administration → Exercices comptables*) ; la date d'une écriture « dernier jour de
   l'exercice courant » est-elle saisissable (exercice ouvert, date future acceptée ?) ; *Quotidien → Documents à traiter →
   Importer le dossier* existe tel que cité (`docs/manual/fr/user-manual.tex` l. 1772-1783) ; la fixture
   `frontend/tests/e2e/fixtures/spc_e2e_invoice.png` est bien une pièce **synthétique** acceptée par l'import (specs
   `inbox-import.spec.ts`, `supplier-documents.spec.ts`) ? Le dossier d'import de l'instance source (`314-source/inbox/`)
   correspond-il à `KESH_INBOX_DIR` de T1/T0 ? Le critère « *N* ≥ 1 et 0 manquante » est-il compatible avec ce que
   **compte** le bilan de l'import (une pièce importée par « Documents à traiter » est-elle une « pièce » de l'export de
   format 2 ?) — `crates/kesh-api/src/routes/admin_backup/`, `x-kesh-backup-files`. Le renvoi aux étapes 2 à 5 (ce qui
   s'applique, ce qui ne s'applique pas) est-il fidèle à `admin-manual.tex` l. 2013-2040 ?
4. **Les LOW — chacun contre son texte.** R4 (`mkdir` de `telechargements`) ; R5 (« seule exception, le contrôle 1 »
   cohérent avec l'en-tête AC3, le contrôle 1 et l'essai (ii) ?) ; R6 (un seul `if` pour les deux gestes sans `sudo` :
   lisible comme **un** bloc, la garde en tête, aucune ambiguïté ?) ; R8 (la regex du contrôle 1, trois variables de plus :
   `grep -E` valide ? l'alternation reste fermée ?) ; F2 (`COPIE` rétablie, relue au journal) ; F3 (port ≠ 80, > 1023 : le
   contrôle 8 et le modèle d'override concordent) ; F4 (« ôter le `#` » : `.env.example` l. 374-377) ; F5 (« machine
   Linux ») ; F6 (en-tête lu à l'onglet Réseau, « à constater ») ; F7 (commandes en listing / `\keshcommand`, G14).
5. **Propagation (grep de la valeur, pas de la formulation)** : `grep -nE "quatre cas|DOCKER_CONTEXT|étape 0|telechargements|
   N peut|0 manquante|\(a\) à \(t\)|\(a\) à \(u\)"` sur la fiche et le registre. Un site périmé qui dit encore l'ancien état
   (étape 0 avant le contrôle 3 ; critère « *N* peut valoir 0 » ; liste de variables sans les trois `DOCKER_*` ; constats
   « (a) à (t) » là où (u) existe) est un finding. Les sites **datés** (Change Log, annotations, registre passé) ne se
   corrigent pas : ils se **complètent**.
6. **Recomptes, commandes à l'appui** : AC (13 numéros, dont AC8 et AC11 retirés) ; tâches (9) ; contrôles de l'AC3 (9) ;
   mutations M1 à M16 (1 active) ; constats (a) à (u) (21 lettres, (e) retirée) ; choix C-31-4-1 à 17 ; une issue à demander.
   Le Change Log P4 dit-il juste ? A-t-il **réécrit** une entrée passée (`git diff 6515764b..HEAD` sur les lignes `-` du
   Change Log, du registre et de `sprint-status.yaml`) ? L'annotation datée de R7 : les deux textes d'origine y sont-ils
   cités fidèlement (`git show 96ff8892:…` pour les retrouver) ?
7. **Aucun outillage nouveau.** La remédiation a-t-elle ajouté un script, une garde, un test, une fixture, une mutation ?
   (Elle ne le devait pas, C-31-4-15.) Un geste « à journaliser » est-il devenu, par la rédaction, une commande jamais
   éprouvée présentée comme acquise ?
8. **Manuel** : la fiche **nomme-t-elle** toujours le manuel d'administration (`.tex` **et PDF**), les labels et les renvois
   pour les ajouts de la P4 (notamment ce qui doit y figurer : arrêt, étape 0 après le contrôle 3, PDF figé « non éprouvé »,
   machine Linux) ? G14, G15, G16 (e), G19 rougiraient-elles ? Les PDF se contrôlent aplatis
   (`pdftotext … - | tr '\n' ' ' | tr -s ' '`) — la sous-section n'est pas encore écrite ; dis-le.

⛔ **Ne propose AUCUN outillage nouveau** : ni script, ni fixture, ni test shell, ni garde de texte, ni réextension d'une
garde existante. **Le livrable de cette story est la procédure exécutée**, non un outillage (C-31-4-15). Un trou se signale
comme **angle mort à écrire dans la fiche** ou comme **geste à journaliser en T3/T8** ; une proposition d'outillage ne
compte pas comme finding.

## Ce que tu rends

Findings `T1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, ligne du dépôt, preuve (commande et sortie
copiée), correction proposée, et **la provenance** : introduit par la remédiation P4, ou antérieur. ⛔ **La liste des axes
exercés ET non exercés** — sans elle, « 0 finding » ne compte pas. Toute affirmation de présence ou d'absence s'appuie sur
une sortie `grep -nF` copiée ; un comportement de Docker, de Compose ou de la station que tu n'as pas pu exécuter se dit
*non exécuté* et se propose comme constat de la T3. Dernier message : chemin, bilan par sévérité, une ligne par finding
MEDIUM ou plus.

**Critère de clôture de la boucle** : une passe à **0 MEDIUM ou plus** dont la remédiation ne touche **aucune ligne de code
de production** (ici, aucune n'en touche : la story est de documentation). Tu ne conclus pas toi-même à la clôture : tu
rends tes findings et tes axes, l'orchestrateur décide.

## Interdits

⛔ **N'écris aucun fichier hors de ton rapport**, et ton rapport par un **chemin absolu** sous
`/home/gcorbaz/devel/kesh-gate-logs/`. Le dépôt principal `/home/gcorbaz/devel/kesh` est en **lecture seule** ; ses
fichiers non versionnés (`.env`, `target/`…) n'ont pas de copie — un `.env` y a déjà été écrasé par un agent, après un
`cd` échoué. Donc : **aucune écriture par chemin relatif** (`-o`, `>`, `cp`, `tee`) ; chaîner par `&&`, jamais par `;`
après un `cd` ; si une commande préalable échoue ou est refusée, **arrête-toi**, ne poursuis pas « ailleurs ».

Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou exécute : `scripts/*` (dont
**`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, **`docker` (aucune sous-commande,
`compose config`, `compose version` et `context` compris)**, **`curl`**, **`mkdir`**, `realpath` et `findmnt` sur un
chemin à créer, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture, `sed -i`. `pdftotext` vers la sortie
standard seulement (aucun fichier temporaire). Aucune donnée réelle de Guy (rien de `~/kesh-test/`, jamais ouvert).
Autorisés : lecture, `grep`, `sed -n`, `awk` en lecture, `git log|show|diff|blame`, `pdftotext … -`.
