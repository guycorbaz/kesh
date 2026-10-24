# Story 31-4 — revue de code P1 (trois lentilles B, E, A)

**Modèle : Sonnet.** Contexte frais, une lentille par agent. **Lecture seule stricte.**

## Ce qui est revu

- Worktree `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4` ; périmètre : `git -C /home/gcorbaz/devel/kesh-31-4 diff
  origin/main...HEAD` (diff aplati, un seul).
- Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (AC1 à AC13, *Principe directeur*, T0 à T8,
  Dev Agent Record) ; registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md` (C-31-4-1 à 20).
- **Le manuel d'administration est le livrable** : `docs/manual/fr/admin-manual.tex`, sous-section
  `\subsection{Répéter une clôture sur une copie}` (`\label{sec:repetition-cloture}`), le `\label{sec:test-restauration}`
  posé sur la sous-section voisine, le renvoi réécrit de l'encadré de `sec:backup-dsm` ; **et son PDF**
  `docs/manual/fr/admin-manual.pdf`. Aussi : `docs/manual/fr/user-manual.tex` et `user-manual.pdf` (AC13, deux chemins),
  `CHANGELOG.md` (AC12).
- **Le journal d'exécution, hors dépôt, est la preuve** : `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-constats-20261024.log`
  (constats à blanc), `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-20261024.log` (répétition réelle),
  `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-t8-20261024.log` (second passage, depuis le PDF). Ils contiennent des
  empreintes : **rien n'en est recopié dans un rapport au-delà de ce qu'il faut pour un finding** (aucun secret).
- Aucun code de production ne change ; aucun test n'est ajouté (C-31-4-15).

## Lire le PDF

Toujours aplati, vers la sortie standard, jamais vers un fichier :
`pdftotext /home/gcorbaz/devel/kesh-31-4/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
(`pdftotext` coupe les lignes : un `grep` naïf rend un faux négatif). Pour une commande du manuel, lire aussi le PDF **non
aplati** (`pdftotext -layout … - | grep -n …`) : c'est ce qu'un lecteur recopie.

## Lentilles

### B — Blind (le manuel seul, sans la fiche)

Lire la sous-section **dans le PDF** comme un administrateur qui va la jouer, sans la fiche. Chercher : une commande
qui ne marcherait pas telle qu'écrite (guillemets, continuation `\`, variable non posée, ligne coupée par le PDF), un
ordre qui fait agir avant de contrôler, un « arrêt » sans geste, une destruction qui pourrait viser autre chose que la
copie, un secret laissé hors du dossier de la copie, une promesse non tenue, un renvoi « ?? ». Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/314-review-p1-B.md`.

### E — Edge cases

Toutes les branches de la procédure : chaque contrôle 1 à 9 et son échec ; chaque cas d'arrêt de l'AC7 (d) ; la garde du
`rm -rf` comparée **mot à mot** à la forme de l'AC7 (branche `*) false` comprise) et ses six refus du journal ; variables
du shell qui l'emportent (le contrôle 1 a été **élargi à `DATABASE_URL`** au développement, constat imprévu : vérifier
qu'aucune autre variable interpolée par `docker-compose.yml` au tag n'échappe au filtre) ; navigateur confiné (snap) ;
port déjà pris ; image absente ; deux répétitions le même jour. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p1-E.md`.

### A — Acceptance

AC1 à AC13 un par un contre le `.tex`, le PDF et le journal : chaque commande du manuel figure-t-elle au journal, dans
cet ordre, avec sa sortie ? chaque écart du journal est-il au Dev Agent Record ? ce que l'AC10 exige comme « non
éprouvé » l'est-il ? les gardes G14, G15, G16, G19 (`crates/kesh-api/tests/configuration_transmise.rs`,
`textes_coherents.rs`) relisent-elles la sous-section comme la fiche le dit (lecture du code des gardes, sans
l'exécuter) ? les décomptes du Dev Agent Record se recomptent-ils (`grep -c`, `git show`) ? Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/314-review-p1-A.md`.

## Exigences de rapport (toutes lentilles)

- Findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec l'extrait **copié** (`grep -nF`, `sed -n`) qui l'établit.
- **Axes exercés, et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` copiée.

## Interdits

Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `docker` (aucune
sous-commande), ni `curl`, ni `python3`, ni `sed -i`, ni `perl -pi`, ni `scripts/prepare-release.sh`, ni aucun script du
dépôt ; aucun `git checkout`/`switch`/`reset`/`stash` sous aucune forme ; aucun fichier hors du rapport (pas de fichier
temporaire, `pdftotext` vers la sortie standard seulement) ; aucun accès à `/var/tmp/kesh-repetition-*`, à `~/kesh-test/`
ni au NAS. Commandes permises : `git log`, `git show`, `git diff`, `grep`, `sed -n`, `pdftotext … -`, `wc`, `ls`.
