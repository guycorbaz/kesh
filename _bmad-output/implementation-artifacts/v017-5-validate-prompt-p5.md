# Prompt — validation P5 de la fiche v017-5 (release 0.17.0 : #854), passe ciblée, Sonnet

Tu valides la **dernière remédiation** de la fiche de la Story v017-5 de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : lire la QR-facture d'un document **scanné** à l'import de pièces (#854). Passe **ciblée** :
une seule lentille, braquée sur le diff de la remédiation de la validation P4, en **contexte frais**. Langue du rapport :
**français**. Modèle : **Sonnet**.

Worktree `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`. Le diff à relire :

```sh
git -C /home/gcorbaz/devel/kesh-0176 diff d8fbb5e2 HEAD -- _bmad-output/implementation-artifacts/
```

(`d8fbb5e2` = la remédiation de la P3 ; `HEAD` = la remédiation de la P4, commit
`docs(v017-5) : remédiation de la validation P4 (refs #854)`). Lis la fiche
`_bmad-output/implementation-artifacts/v017-5-qr-scanne.md` et le registre `v017-5-choix-autonomes.md` **en entier** pour
le contexte, mais ne cherche de défauts que dans ce que ce diff a écrit ou dans ce qu'il aurait dû propager. Ne lis
**aucun** rapport de passe précédente (`kesh-gate-logs/v0175-validate-p*.md`).

## La règle de recherche — aplatir avant de greper

Une expression coupée sur deux lignes échappe à `grep` ligne à ligne : la P4 a trouvé ainsi un libellé périmé que les
passes précédentes n'avaient pas vu. **Avant chaque recherche d'une expression** (libellé, phrase, nombre suivi d'un mot),
aplatis le fichier, et cite la sortie :

```sh
tr '\n' ' ' < _bmad-output/implementation-artifacts/v017-5-qr-scanne.md | tr -s ' ' | grep -oE '.{60}<expression>.{40}'
```

(idem pour le registre, et `pdftotext … - | tr '\n' ' ' | tr -s ' '` pour un PDF). Un « absent » sans cette sortie ne
compte pas.

## Ce que la remédiation P4 a changé, et ce qu'il faut y chercher

1. **Le libellé** : reste-t-il, hors des entrées datées du Change Log et des passages historiques du registre, une
   occurrence de l'ancien libellé du badge (« Aucune QR-facture lue », « Aucun QR lu », « Sans QR-code » comme valeur
   prescrite) — fiche **aplatie** ?
2. **Les cinq cas** (AC-7, limite assumée, Q1, registre C-v017-5-7 et son amendement P4) : vérifie chacun au code
   (`qr_decode.rs:84-113`, `inbox_import.rs:539-576`, `map_decode_error`) ; les numéros de cas sont-ils cohérents entre
   AC-7, la limite, Q1 et le registre ; la phrase « « Sans QR-code » est faux aux cas 2, 3 et 4 ; « Aucun QR lu » aux cas
   2 et 4 ; « Aucune QR-facture lue » au cas 4 » est-elle exacte ; ce que Q1 dit du rapport pour chaque cas est-il vrai
   (`+page.svelte:224-240`) ?
3. **AC-12** : la nouvelle définition de `none` à l'API est-elle exacte, et cohérente avec le registre (« elle couvre les
   trois premiers ») ?
4. **AC-10, les trois messages** : chacun sort-il bien dans le cas que la fiche lui attribue (`api-client.ts:230-330`,
   `parseErrorResponse`, `toFetchApiError`, `serverStatusMessage`, `+page.svelte:96-110`) — un 504 de nginx ou de DSM
   porte-t-il un corps non JSON ; un corps JSON d'erreur changerait-il le message ? Le retrait d'« Erreur inattendue… »
   est-il juste ?
5. **P8 et P9** : le compteur du second temps (0 pour le témoin et D4, un par page pour D1 à D3) est-il cohérent avec
   AC-5 et AC-6 ; la conduite du dépassement à 200 dpi (issue demandée) et la page de D1 engendrée à 300 dpi en mémoire
   sont-elles cohérentes avec AC-1 et AC-6 ?
6. **Le registre** : l'amendement P3 ajouté à C-v017-5-5, le titre de C-v017-5-7, le recompte de C-v017-5-10 (cinq
   fichiers de production) — exacts ?
7. **L'inventaire** : `admin-manual.tex:1492` ajouté ; reste-t-il une forme « sans QR » du manuel non inventoriée
   (`.tex` et PDF aplatis) ?
8. **Les décomptes** de l'entrée « Validation P4 » du Change Log (3 MEDIUM, 6 LOW) : recompte.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p5.md` :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation
   de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** (1 à 8 ci-dessus) et **non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0
   finding » sans cette liste ne compte pas comme passe.
3. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport ; il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh issue view` et `gh issue list` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `latexmk`, ni `python3`, ni
  `node`, ni `sqlx`, ni `docker`, ni `curl`, ni `mariadb`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- **Aucune pièce réelle** : n'ouvre ni `~/kesh/`, ni `~/kesh-test/`, ni le dossier de projet du Project Lead, ni
  Paperless, ni aucun PDF hors du dépôt ; aucune donnée réelle dans le rapport.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`, `gh issue list`.
