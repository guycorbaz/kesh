# Story 31-4 — revue de code P2 (passe complète, trois lentilles B, E, A)

**Modèle : Opus.** Contexte frais, une lentille par agent. **Lecture seule stricte.** Passe **complète** (non ciblée) :
la P1 a rendu des MEDIUM, et leur remédiation change des commandes du manuel.

## Ce qui est revu

- Worktree `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4` ; périmètre : `git -C /home/gcorbaz/devel/kesh-31-4 diff
  origin/main...HEAD` (diff aplati, un seul).
- **Attention particulière au commit de remédiation de la P1**, `6186edfc`
  (`git -C /home/gcorbaz/devel/kesh-31-4 show 6186edfc -- docs/manual/fr/admin-manual.tex`) : *la sévérité se déplace
  vers ce qu'on vient d'écrire*. Il ajoute ou change, dans la sous-section `sec:repetition-cloture` :
  - le filtre `env | grep` des contrôles 1 et final (`RUST_LOG=` ajouté) ;
  - au contrôle 4, l'**inventaire des variables interpolées** par le compose téléchargé
    (`grep -oE '\$\{?[A-Za-z_][A-Za-z0-9_]*' … | tr -d '${' | sort -u | while read -r n; do printenv "$n" …`) et la
    phrase sur les exemples `0.14.0` face au prérequis 0.15.0 ;
  - à l'étape 0, le `.keshbackup` enregistré sous `production.keshbackup`, les deux exports enchaînés **sans écriture**
    entre eux, et un listing `mv` à cibles nommées ;
  - le repli `mv` de l'export de la copie, en listing ;
  - la boucle de comparaison (`e=…`, `test -s "$e" && echo "$z : $(wc -l < "$e") tables"`) et son « Attendu » ;
  - la destruction en **trois listings** (sans `sudo`, avec `sudo`, `rm -rf` gardé seul avant le premier démarrage), le
    volume et le réseau nommés ;
  - « Arrêt » renvoyé à *En cas d'arrêt*, reprise avec un nouveau dossier (date ou suffixe `-2`) ; les puces « Au
    contrôle 3 » et « Après l'étape 0 » ; l'encadré final (espace disque, « sept formes »).
- **Retiré, à ne pas relever** : le commit `9c59dec4` (étape 3 du test périodique) a été annulé par `728012ba` ; le
  sujet est l'issue #709. Le manuel dit de nouveau que la répétition ne remplit pas l'étape 3 à la lettre.
- Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (AC1 à AC13, *Principe directeur*, T0 à T8,
  Change Log — dont l'entrée *Revue de code P1* —, Dev Agent Record — dont *Revue de code P1 remédiée*) ; registre
  `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md` (C-31-4-1 à 22).
- Rapports de la P1, pour ne pas refaire ce qui est clos et contrôler ce qui l'est prétendument :
  `/home/gcorbaz/devel/kesh-gate-logs/314-review-p1-B.md`, `-E.md`, `-A.md`.
- **Le manuel d'administration est le livrable** : `docs/manual/fr/admin-manual.tex`, sous-section
  `\subsection{Répéter une clôture sur une copie}` (`\label{sec:repetition-cloture}`), le `\label{sec:test-restauration}`
  de la sous-section voisine, le renvoi de l'encadré de `sec:backup-dsm` ; **et son PDF**
  `docs/manual/fr/admin-manual.pdf`. Aussi : `docs/manual/fr/user-manual.tex` et `user-manual.pdf` (AC13),
  `CHANGELOG.md` (AC12).
- **Le journal d'exécution, hors dépôt, est la preuve** : `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-constats-20261024.log`,
  `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-20261024.log`, `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-t8-20261024.log`.
  ⚠️ Ils sont **antérieurs** à la remédiation P1 : les commandes ajoutées par `6186edfc` n'y figurent pas, et n'ont été
  essayées que sur des fichiers synthétiques (Dev Agent Record). Les journaux contiennent des empreintes : **rien n'en
  est recopié dans un rapport au-delà de ce qu'il faut pour un finding** (aucun secret).
- Aucun code de production ne change ; aucun test n'est ajouté (C-31-4-15).

## Lire le PDF

Toujours aplati, vers la sortie standard, jamais vers un fichier :
`pdftotext /home/gcorbaz/devel/kesh-31-4/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
(`pdftotext` coupe les lignes : un `grep` naïf rend un faux négatif). Pour une commande du manuel, lire aussi le PDF **non
aplati** (`pdftotext -layout … - | grep -n …`) : c'est ce qu'un lecteur recopie.

## Lentilles

### B — Blind (le manuel seul, sans la fiche)

Lire la sous-section **dans le PDF** comme un administrateur qui va la jouer, sans la fiche. Chercher : une commande
qui ne marcherait pas telle qu'écrite (guillemets, continuation `\`, variable non posée, `$` ou `{` mal rendus, ligne
coupée ou débordant la marge), un ordre qui fait agir avant de contrôler, un « arrêt » sans geste, une destruction qui
pourrait viser autre chose que la copie, un secret laissé hors du dossier de la copie, une promesse non tenue, un renvoi
« ?? ». En particulier : les trois listings de destruction se lisent-ils sans ambiguïté sur celui qu'on tape ? les noms
`production.keshbackup`, `souverainete-production.zip`, `souverainete-copie.zip` sont-ils les mêmes partout où ils
sont produits, déplacés et lus ? Rapport : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p2-B.md`.

### E — Edge cases

Toutes les branches : chaque contrôle 1 à 9 et son échec ; chaque cas d'arrêt de l'AC7 (d) et de la rubrique *En cas
d'arrêt* ; la garde du `rm -rf` comparée **mot à mot** à la forme de l'AC7 (branche `*) false` comprise) **dans chacun
des trois listings**. Pour l'inventaire des variables du contrôle 4 : l'ensemble qu'il rend est-il bien l'ensemble clos
des variables que Compose interpole (formes `${X}`, `${X:-…}`, `${X:?…}`, `$X`, `$$` échappé, variables d'un `env_file`,
entrées `environment:` sans valeur), au tag `v0.14.0` (`git show v0.14.0:docker-compose.yml`) **et** au `HEAD`
(`docker-compose.yml` du worktree, futur compose d'une 0.15.0) ? un faux silence est-il possible (variable exportée vide,
`printenv` d'un nom non exporté) ? Pour la comparaison : deux archives absentes, une seule, une archive qui n'est pas un
export, deux `N` égaux mais des tables différentes. Écriture intercalée entre les deux exports de l'étape 0 : la
consigne suffit-elle ? Deux répétitions le même jour (suffixe `-2`) contre la garde. Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/314-review-p2-E.md`.

### A — Acceptance

AC1 à AC13 un par un contre le `.tex`, le PDF et le journal ; chaque écart du journal (et chaque commande ajoutée par
`6186edfc`, que le journal ne porte pas) est-il déclaré au Dev Agent Record ? ce que l'AC10 exige comme « non éprouvé »
l'est-il, et l'encadré final dit-il encore vrai après la remédiation ? les gardes G14, G15, G16, G19
(`crates/kesh-api/tests/configuration_transmise.rs`, `textes_coherents.rs`) relisent-elles la sous-section modifiée
comme la fiche le dit (lecture du code des gardes, sans l'exécuter) ? Le Change Log (*Revue de code P1*), le registre
(C-31-4-21, C-31-4-22) et le Dev Agent Record (*Revue de code P1 remédiée* : issues #705, #707, #709, gate ciblé
47/47) disent-ils exactement ce que le commit fait, et leurs décomptes se recomptent-ils (`grep -c`, `git show`) ?
Rapport : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p2-A.md`.

## Exigences de rapport (toutes lentilles)

- Findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec l'extrait **copié** (`grep -nF`, `sed -n`) qui l'établit.
- Pour chaque finding, dire s'il **vient de la remédiation P1** (`6186edfc`) ou du texte antérieur.
- **Axes exercés, et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` copiée.
- Le décompte final (CRITICAL/HIGH/MEDIUM/LOW) se recompte sur la liste des findings du rapport.

## Interdits

Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `docker` (aucune
sous-commande), ni `curl`, ni `python3`, ni `sed -i`, ni `perl -pi`, ni `scripts/prepare-release.sh`, ni aucun script du
dépôt ; aucun `git checkout`/`switch`/`reset`/`stash` sous aucune forme ; aucun fichier hors du rapport (pas de fichier
temporaire, `pdftotext` vers la sortie standard seulement) ; aucun accès à `/var/tmp/kesh-repetition-*`, à `~/kesh-test/`
ni au NAS. Commandes permises : `git log`, `git show`, `git diff`, `grep`, `sed -n`, `pdftotext … -`, `wc`, `ls`.
