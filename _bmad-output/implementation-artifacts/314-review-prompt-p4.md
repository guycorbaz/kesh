# Story 31-4 — revue de code P4 (passe ciblée, une lentille : regression hunter)

**Modèle : Haiku.** Contexte frais, un seul agent. **Lecture seule stricte.** Passe **ciblée** : la P3 (Sonnet, B 0/0/0/5,
E 0/0/1/7, A 0/0/0/4) n'a rendu qu'un MEDIUM, E M-1 = B3-1, né de la remédiation P2 ; ce qui reste à relire n'est plus la
story, c'est **la remédiation de la P3**. La boucle ne se clôt que sur une passe qui ne rend **aucun** finding au-dessus de
LOW.

## Ce qui est revu — le seul diff de la remédiation P3

- Worktree `/home/gcorbaz/devel/kesh-31-4`, branche `story/31-4`. Périmètre, **un seul diff aplati** (ne pas lire les
  commits un à un) :
  `git -C /home/gcorbaz/devel/kesh-31-4 diff b9c02b8d..HEAD`
  Pour le seul manuel : `git -C /home/gcorbaz/devel/kesh-31-4 diff b9c02b8d..HEAD -- docs/manual/fr/admin-manual.tex`.
- Ce que la remédiation dit avoir fait (à **vérifier**, non à croire) — fiche
  `_bmad-output/implementation-artifacts/31-4-repetition-a-blanc.md`, entrée *Revue de code P3* du Change Log et section
  *Revue de code P3 remédiée* du Dev Agent Record ; registre
  `_bmad-output/implementation-artifacts/epic-31-choix-autonomes-314.md`, C-31-4-28 et C-31-4-29 :
  - **E M-1** : la dernière commande du *contrôle final* (`sec:repetition-cloture`) est un `if … elif … else … fi` à
    quatre issues (`échec : COPIE n'est pas posée`, `échec : COPIE n'est pas le chemin de la copie`,
    `échec : … existe encore`, `copie détruite`) ; l'« Attendu » rend la ligne `copie détruite` **requise** ; la marche
    *À la destruction ou au contrôle final* juge sur « la seule ligne `copie détruite` » ;
  - E L-1 : terminal neuf (reposer `COPIE`, refaire le contrôle 1) ; cas *Reste* quand le dossier a disparu ;
  - B3-3 (« si vous le produisez » retiré) ; B3-4 (encadré *Ce que cette procédure n'a pas éprouvé*) ; B3-5
    (avertissement sur les valeurs affichées par `env | grep`) ; E L-5 (*Ce qu'elle ne fait pas*, « aucune donnée
    comptable », création des lignes de réglages par l'export de souveraineté) ;
  - fiche : annotations datées à l'AC1, l'AC3, l'AC7 (c) et (d), l'AC10, au Dev Agent Record P2 ; `sprint-status.yaml`.
- **Le manuel d'administration est le livrable** : `docs/manual/fr/admin-manual.tex` **et son PDF**
  `docs/manual/fr/admin-manual.pdf`. Le PDF se contrôle, pas seulement le `.tex`.
- Rapports de la P3, pour ce que la remédiation devait corriger : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p3-B.md`,
  `-E.md`, `-A.md`. LOW consignés sans être traités (Change Log) : B3-2 = E L-6 = première moitié d'A LOW-4, E L-2,
  E L-3, E L-7 — à ne relever que si la remédiation P3 les aggrave.

## Lire le PDF

Toujours vers la sortie standard, jamais vers un fichier. Aplati :
`pdftotext /home/gcorbaz/devel/kesh-31-4/docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF '…'`
(`pdftotext` coupe les lignes : un `grep` naïf rend un faux négatif). Non aplati, pour les commandes et les sauts de page :
`pdftotext -layout /home/gcorbaz/devel/kesh-31-4/docs/manual/fr/admin-manual.pdf - | grep -n …`.

## La lentille — regression hunter

Chercher **ce que la remédiation P3 a cassé ou laissé à moitié**, et seulement cela :

1. **Le nouveau test du contrôle final, lu comme un shell le lirait.** Chaque branche : `COPIE` vide, non posée, chemin
   relatif, autre nom sous `/var/tmp`, `/var/tmp` lui-même, barre finale, lien symbolique, lien pendant, dossier présent,
   dossier absent. Une seule de ces formes doit-elle rendre `copie détruite` quand le dossier existe encore ? Le `! case …
   esac` dans un `elif … ||` est-il valide en `bash` et en `sh` ? Le listing du PDF (`-layout`) est-il identique au `.tex`
   (barres obliques inverses, guillemets, `*) false ;;`) et **entier sur une page** ?
2. **Propagation du symptôme** : reste-t-il, dans le manuel, la fiche, le registre ou le CHANGELOG, une phrase qui juge le
   contrôle final sur « rien », « des lignes » ou `test ! -e` seul, ou qui cite encore
   `[ -n "$COPIE" ] && test ! -e` comme texte actuel du manuel ? (`grep -nF` sur chaque fichier ; une mention dans une
   entrée datée d'une passe passée est un relevé historique, non un résidu.)
3. **Les phrases nouvelles disent-elles vrai ?** « son nom porte la date de la répétition, celle du procès-verbal »
   (contrôle 3 : `COPIE=/var/tmp/kesh-repetition-AAAAMMJJ`) ; le cas *Reste*, « si cette date est inconnue, le chemin de
   la répétition que vous alliez commencer convient » — peut-il faire conclure à tort qu'une copie est détruite ? ;
   « aucune donnée comptable » et la création des lignes de réglages par l'export
   (`crates/kesh-api/src/exports/global.rs` l. 304 et 313, `crates/kesh-db/src/repositories/company_invoice_settings.rs`,
   `company_dunning_settings.rs`) — la phrase contredit-elle une autre phrase du manuel (« ne modifie aucune donnée »,
   § *Exporter*) ? ; l'avertissement de B3-5 ; l'encadré (le troisième listing et le dernier test du contrôle final
   y sont-ils dits « jamais joués », et l'essai sur dossiers synthétiques est-il décrit exactement ?).
4. **Les comptes rendus de la P3** (Change Log, Dev Agent Record, C-31-4-28/29, `sprint-status.yaml`) : chaque décompte
   se recompte-t-il (« onze formes », bruts B/E/A, tendance P1 8 → P2 5 → P3 1, gate ciblé) ? Les annotations datées
   réécrivent-elles une entrée passée au lieu de l'annoter ?
5. **Le PDF** : 0 renvoi `??` ; les phrases nouvelles présentes une fois aplaties ; aucun listing coupé par un saut de
   page dans `sec:repetition-cloture` ; la brochure et le manuel utilisateur n'ont pas changé sans raison
   (`git diff --stat b9c02b8d..HEAD`).

## Exigences de rapport

Rapport, **seul fichier écrit** : `/home/gcorbaz/devel/kesh-gate-logs/314-review-p4.md`.

- Findings classés CRITICAL / HIGH / MEDIUM / LOW, chacun avec l'extrait **copié** (`grep -nF`, `sed -n`) qui l'établit ;
  toute affirmation d'absence ou de présence s'appuie sur une sortie `grep -nF` copiée (`-F` obligatoire).
- Pour chaque finding : vient-il de la remédiation P3 (`b9c02b8d..HEAD`) ou d'un texte antérieur ?
- **Axes exercés, et axes non exercés**, listés (les cinq ci-dessus, un par un) : un « 0 finding » sans cette liste ne
  compte pas.
- Le décompte final (CRITICAL/HIGH/MEDIUM/LOW) se recompte sur la liste des findings du rapport.
- Tout écart à la liste des commandes permises se déclare en tête du rapport.

## Interdits

Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `docker` (aucune
sous-commande), ni `curl`, ni `python3`, ni `bash -c`/`sh -c` pour « essayer » le listing, ni `sed -i`, ni `perl -pi`, ni
`scripts/prepare-release.sh`, ni aucun script du dépôt ; aucun `git checkout`/`switch`/`reset`/`stash` sous aucune forme.
**Aucune redirection vers un fichier** (`>`, `>>`, `tee`, `-o`) **et aucun fichier temporaire**, pas même dans un
répertoire de travail de session : `pdftotext` et toute extraction vont à la sortie standard, en tube. Le seul fichier
écrit est le rapport. Aucun accès à `/var/tmp/kesh-repetition-*`, à `~/kesh-test/` ni au NAS ; le dépôt principal
`/home/gcorbaz/devel/kesh` n'est pas lu pour écrire. Commandes permises : `git log`, `git show`, `git diff`, `grep`,
`sed -n`, `pdftotext … -`, `wc`, `ls`, et, en aval d'un tube, les filtres en lecture seule vers la sortie standard `tr`,
`sort`, `cut`, `head`, `tail`, `awk`.
