# Story 31-4 — revue de code P3 (passe complète, trois lentilles B, E, A)

**Modèle : Sonnet.** Contexte frais, une lentille par agent. **Lecture seule stricte.** Passe **complète** (non ciblée) :
la P2 a rendu cinq MEDIUM, et leur remédiation change l'ordre de la restauration, des contrôles et la destruction.

## Ce qui est revu

- Worktree `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4` ; périmètre : `git -C /home/gcorbaz/devel/kesh-31-4 diff
  origin/main...HEAD` (diff aplati, un seul).
- **Attention particulière au commit de remédiation de la P2**, `34ccce7e`
  (`git -C /home/gcorbaz/devel/kesh-31-4 show 34ccce7e -- docs/manual/fr/admin-manual.tex CHANGELOG.md`) : *la sévérité
  se déplace vers ce qu'on vient d'écrire*. Il change, dans `sec:test-restauration` et `sec:repetition-cloture` :
  - la phrase de `sec:test-restauration`, la fin de la comparaison et le procès-verbal : la répétition **ne remplace
    pas** le test de restauration périodique tant que l'issue #709 n'est pas tranchée ; l'export de souveraineté de la
    production n'est plus conditionné à « tenir lieu du test » ;
  - l'introduction des contrôles et le contrôle 4 : l'issue corrective (`unset`) vaut aux contrôles 1 **et 4** ;
  - l'étape 0 : le décompte `unzip -l` déplacé après le listing `mv`, « avant la restauration, qui le renomme
    `source.keshbackup` » ; le téléchargement d'un PDF jamais téléchargé nommé parmi les écritures à éviter ;
  - la restauration : étapes 2 à 4, puis l'export de souveraineté de la copie (« aussitôt reconnecté »), puis l'étape 5
    (« Puis faites l'étape 5 » à la fin de la comparaison) ; la cause « un PDF de facture téléchargé avant lui :
    `invoices.csv` diffère » ;
  - la destruction : « des trois listings qui suivent », `\needspace` devant les listings (paquet `needspace` chargé
    dans le préambule de `admin-manual.tex`) ; le contrôle final `[ -n "$COPIE" ] && test ! -e "$COPIE"` ;
  - *En cas d'arrêt* : « Au contrôle 3 » (dossier vide ou ne portant que `telechargements/`), cas *Reste* (listing
    nommé, contrôle final), entrée nouvelle **« À la destruction ou au contrôle final »** ;
  - l'encadré final (« jouée, dans une rédaction antérieure », puce des commandes jamais jouées, « lors de cette
    exécution ») ; les deux phrases sur le journal d'audit (connexions) ; le tag de `DEPOT` « contrôlé par rien » ;
  - le CHANGELOG (entrée *Ajouté*).
- Fiche : `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md` (AC1 à AC13 — annotations datées de la P2 à
  l'AC3, l'AC4 et l'AC6 —, Change Log — entrée *Revue de code P2* —, Dev Agent Record — *Revue de code P2 remédiée*) ;
  registre `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md` (C-31-4-1 à 27, annotation datée sous
  C-31-4-21).
- Rapports des passes précédentes, pour ne pas refaire ce qui est clos et contrôler ce qui l'est prétendument :
  `/home/gcorbaz/devel/kesh-gate-logs/314-review-p1-{B,E,A}.md` et `/home/gcorbaz/devel/kesh-gate-logs/314-review-p2-{B,E,A}.md`.
  LOW de la P2 consignés sans être traités (Change Log) : B2-L4 = E-LOW-2, B2-L5, E-LOW-7, A-LOW-4 — à ne relever que si
  la remédiation P2 les aggrave.
- **Retiré, à ne pas relever** : `9c59dec4` (annulé par `728012ba`) ; l'étape 3 du test périodique est l'issue #709.
- **Le manuel d'administration est le livrable** : `docs/manual/fr/admin-manual.tex` et **son PDF**
  `docs/manual/fr/admin-manual.pdf`. Aussi : `docs/manual/fr/user-manual.tex` et `user-manual.pdf` (AC13), `CHANGELOG.md`
  (AC12).
- **Le journal d'exécution, hors dépôt** : `/home/gcorbaz/devel/kesh-gate-logs/314-repetition-constats-20261024.log`,
  `314-repetition-20261024.log`, `314-repetition-t8-20261024.log`. ⚠️ Antérieurs aux remédiations P1 et P2 : l'encadré
  doit désormais le dire. Les journaux contiennent des empreintes : **rien n'en est recopié au-delà de ce qu'il faut pour
  un finding** (aucun secret).
- Aucun code de production ne change ; aucun test n'est ajouté (C-31-4-15).

## Lire le PDF

Toujours aplati, vers la sortie standard, jamais vers un fichier :
`pdftotext /home/gcorbaz/devel/kesh-31-4/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '`
(`pdftotext` coupe les lignes : un `grep` naïf rend un faux négatif). Pour une commande, lire aussi le PDF **non aplati**
(`pdftotext -layout … - | grep -n …`) : c'est ce qu'un lecteur recopie ; un listing coupé par un saut de page se voit là.

## Lentilles

### B — Blind (le manuel seul, sans la fiche)

Lire la sous-section **dans le PDF** comme un administrateur qui va la jouer, sans la fiche. Chercher : une commande qui ne
marcherait pas telle qu'écrite, un ordre qui fait agir avant de contrôler (en particulier le nouvel ordre étapes 2 à 4 →
export de la copie → étape 5 → gestes : se suit-il sans retour en arrière ? la liste « Ce qui est propre à la copie »
reste-t-elle juste là où elle est placée ?), un « arrêt » sans geste, une destruction qui pourrait viser autre chose que
la copie, un secret laissé hors du dossier, une promesse non tenue (l'encadré dit-il vrai ?), un renvoi « ?? », un listing
coupé par un saut de page. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p3-B.md`.

### E — Edge cases

Toutes les branches : chaque contrôle 1 à 9 et son échec ; chaque cas d'arrêt de l'AC7 (d) et de *En cas d'arrêt*, dont
l'entrée nouvelle *À la destruction ou au contrôle final* (son geste vaut-il quand le `.env`, l'override ou le dossier
manquent ? quand `COPIE` n'est plus posée ? renvoie-t-elle à des listings qui existent ?) ; la garde du `rm -rf` comparée
**mot à mot** à l'AC7 dans les trois listings ; le contrôle final avec `COPIE` vide ou non posée ; l'export de la copie
avant l'étape 5 : reste-t-il un autre geste, entre la reconnexion et l'export, qui écrive dans une table de l'export de
souveraineté (lecture du code des routes appelées, `crates/kesh-api/src/routes/`, et de `crates/kesh-api/src/exports/
csv_tables.rs`) ? Rapport : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p3-E.md`.

### A — Acceptance

AC1 à AC13 un par un contre le `.tex`, le PDF et les journaux, **avec les annotations P2** de l'AC3, l'AC4 et l'AC6 :
l'amendement de l'AC3 est-il justifié et consigné (C-31-4-24) ? l'AC6 est-elle tenue (aucune phrase du manuel ni du
CHANGELOG ne dit que la répétition vaut, tient lieu ou remplace le test de restauration) ? l'AC10 (non-éprouvés) et
l'encadré disent-ils vrai ? les gardes G14, G15, G16, G19 (`crates/kesh-api/tests/configuration_transmise.rs`,
`textes_coherents.rs`) relisent-elles la sous-section modifiée comme la fiche le dit (lecture du code, sans l'exécuter) ?
Le Change Log (*Revue de code P2*), le registre (C-31-4-23 à 27) et le Dev Agent Record (*Revue de code P2 remédiée* :
chaînes du PDF, gate ciblé 47/47, `314-gate-cible-p2.log`) disent-ils exactement ce que le commit fait, et leurs
décomptes se recomptent-ils ? `sprint-status.yaml` est-il à jour ? Rapport :
`/home/gcorbaz/devel/kesh-gate-logs/314-review-p3-A.md`.

## Exigences de rapport (toutes lentilles)

- Findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec l'extrait **copié** (`grep -nF`, `sed -n`) qui l'établit.
- Pour chaque finding, dire s'il **vient de la remédiation P2** (`34ccce7e`), de la remédiation P1 (`6186edfc`) ou du
  texte antérieur.
- **Axes exercés, et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` copiée.
- Le décompte final (CRITICAL/HIGH/MEDIUM/LOW) se recompte sur la liste des findings du rapport.
- Tout écart à la liste des commandes permises se déclare en tête du rapport.

## Interdits

Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `docker` (aucune
sous-commande), ni `curl`, ni `python3`, ni `sed -i`, ni `perl -pi`, ni `scripts/prepare-release.sh`, ni aucun script du
dépôt ; aucun `git checkout`/`switch`/`reset`/`stash` sous aucune forme ; aucun fichier hors du rapport (pas de fichier
temporaire, `pdftotext` vers la sortie standard seulement) ; aucun accès à `/var/tmp/kesh-repetition-*`, à `~/kesh-test/`
ni au NAS. Commandes permises : `git log`, `git show`, `git diff`, `grep`, `sed -n`, `pdftotext … -`, `wc`, `ls`, et, en
aval d'un tube, les filtres en lecture seule vers la sortie standard `tr`, `sort`, `cut`, `head`, `awk`.
