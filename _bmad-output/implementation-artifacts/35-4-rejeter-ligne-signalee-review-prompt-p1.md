# Prompt — revue de code P1 de la Story 35-4 (#1023, #1024), trois lentilles, première passe complète

Tu relis la **Story 35-4** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : en réconciliation, **_Rejeter_ une sélection qui contient une ligne signalée « déjà
comptabilisée » passe par la confirmation** (D1), dont le texte dit le geste et sa portée — rejeter ne passe aucune écriture,
« Rattacher à une écriture existante » si elle est déjà aux livres, « Annuler le rejet » au détail de l'import (D2) ; et
**une relecture de la liste referme la confirmation ouverte** (`load()` pose `pendingGesture = null`, D3, #1024). Frontend
seul, trois clés ×4 locales, **aucune migration**, aucun changement du serveur.

Worktree `/home/gcorbaz/devel/kesh-e354`, branche `story/35-4-rejeter-ligne-signalee`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e354 diff 1ebe1240 16105cba` (`1ebe1240` = base `epic-35/base2` ; `16105cba` = dernier commit de
code, spec E2E seule ; `ec3c9e35` = dernier commit du code de production ; `94b255d9` ne porte que le compte rendu). Langue du
rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-4-rejeter-ligne-signalee.md` en entier (*Ce qui se lit au code*, D1 à D5, AC1 à AC…,
tests et mutations M1 à M9, *Ce qui doit rester vrai*, **Change Log** et **Dev Agent Record**) ; les registres
`e35-choix-autonomes-e350.md` (C-e350-8, -12, -18, -20) et `e35-choix-autonomes-e354.md` (constat de l'étape 0, C-e354-1, -2).
Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e354/` : `rouges.log`, `rouges-vitest.log`, `verts-vitest-1.log`, `gates.log`,
`gate-test-fast.log`, `gate-ci*.log`, `front-*.log`, `check-1.log`, `e2e-r0/` à `e2e-r5/`, `e2e-r5-lanceur.log`, `mutations.log`,
`mut/`, `make-user.log`, `user-manual-aplati.txt` (les scripts `*.sh` sont à **lire**, jamais à exécuter). **Capture** (thème
clair) : `captures/e354-confirmation-rejeter.png` — regarde-la si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
  aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **La garde (D1)** : `requestReject` remplace `onclick={onReject}` ; `GuardedGesture` gagne `{ kind: 'reject' }` ;
   `confirmGesture` l'exécute aussitôt ; `runGesture` est un `switch` exhaustif (`never`), **sans repli** ; *Rattacher* reste hors
   garde. La garde ne s'ouvre que si la sélection contient une ligne signalée ; *Rejeter quand même* envoie **toute** la sélection.
   Les autres gestes (*Accepter*, *Écriture manuelle*, *Éclater*) gardent-ils exactement leur comportement ?
2. **D3, la relecture** : `load()` pose `pendingGesture = null` **sans le lire avant** (sinon l'`$effect` qui appelle `load()`
   suivrait `pendingGesture` et refermerait chaque confirmation aussitôt) — vérifie au code qu'aucune lecture, directe ou par une
   fonction appelée avant le premier `await`, ne réintroduit la dépendance. `confirmTransactions` non vidé ; aucune région
   `role="status"`, aucun avis ; `if (!current) return;` et les gardes `selected.size === 0` restent. Une confirmation peut-elle
   encore être validée sur des lignes périmées par un autre chemin (sélection modifiée pendant que le dialogue est ouvert — les
   cases à cocher ne sont pas `disabled={busy}`) ?
3. **Le dialogue (D2)** : `gesture: 'write' | 'reject'`, `selectedCount` pris au moment de `guard` ; intro `-one` / `-other`
   selon le nombre de **cochées** (et non de signalées) ; bouton « Rejeter quand même », même `data-testid` ; la ligne signalée
   seule nommée quand d'autres sont cochées. L'en-tête du composant et celui d'`already-booked.ts` disent-ils vrai après la story ?
4. **i18n** : trois clés ×4 locales sous `reconciliation-labels-already-booked-`, à côté du bloc voisin ; apostrophe typographique ;
   les libellés cités égaux, **dans chaque locale**, à `reconciliation-link-button-label` et `reconciliation-unreject-button`
   (relis les deux clés dans les quatre `messages.ftl`) ; `{ $count }` dans `-other`, absent de `-one` ; replis littéraux égaux au
   fr-CH ; `sitesTotal` 3026 → 3029 recompté ; `already-booked-i18n.test.ts` (liste `CLES`, « huit/dix clés » mis à jour ?).
5. **Les manuels contre le code** (`user-manual.tex:3049-3050`, `:3097`, `:3107`), **le PDF aplati** (absence de « sans message
   (issue #1024) », présence de « Rejeter quand même ») ; le CHANGELOG `## [0.17.7]` *Corrigé*. Chaque phrase est-elle vraie au
   code ? Reste-t-il une phrase « *Rejeter* … ne demande rien » ailleurs (`git grep -n 'Rejeter' docs/manual/fr/*.tex`) ?
6. **Les E2E** : la spec neuve `reconciliation-already-booked-reject.spec.ts` (343 lignes) ; l'extraction `camt-synthetic.ts` et
   la retouche de `reconciliation-show-older.spec.ts` (C-e354-1) — même relevé, même comportement qu'avant pour la spec reprise ?
   Les ajouts à `reconciliation-deleted-import.spec.ts` et `reconciliation-reject-unmatched.spec.ts` (une confirmation à passer ?
   pourquoi, si leurs lignes ne sont pas signalées ?). C-e354-2 : le clic hors réessayé jusqu'à la fermeture — la boucle est-elle
   bornée, et peut-elle masquer un vrai défaut (un dialogue qui ne se ferme jamais rougit-il encore) ? Les données de test sont-elles
   synthétiques, isolées des autres specs (compte partagé `CH4431999123000889012`) ?
7. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`rouges.log` : 12 rouges Vitest, E2E d'AC7 rouge
   au binaire de départ) ? M1 à M9 tuées au **second** tirage (le premier tuait sur « No test files found ») — chaque mutation
   tuée par un test rouge **nommé** ? Le test remplacé (`:1737-1743` de la base, « aucune confirmation (M6) ») : remplacé par AC1,
   et aucun autre attendu existant retouché sans motif (`git diff 1ebe1240 16105cba -- '*.test.ts'`, lignes `-`) ? Propose des
   mutations plausibles non jouées (D3 surtout : lire `pendingGesture` dans `load()`).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `requestReject`, `guard`, `runGesture`, `confirmGesture`, `load()`, le dialogue, les clés, la spec neuve.
Cherche : un `switch` non exhaustif ou un repli ; une dépendance réactive introduite dans un `$effect` ; un état (`pendingGesture`,
`confirmTransactions`, sélection) qui survit à une relecture et rend un geste incohérent ; un test qui passe **à vide** ; une attente
E2E non bornée.

## Lentille E — Edge (chaque branche, chaque borne)

Sélection d'une seule ligne signalée ; une signalée et deux non signalées ; toutes signalées ; aucune ; sélection modifiée pendant
que le dialogue est ouvert (case cochée ou décochée derrière la modale, en jsdom comme au navigateur) ; *Rejeter* pendant `busy` ;
relecture déclenchée par le changement de `limit` ou de compte bancaire pendant la confirmation ; *Rejeter quand même* dont le lot
revient en `failed[]` partiel (ligne déjà rejetée dans un autre onglet) ; double clic sur *Rejeter quand même* ; Échap pendant
l'envoi. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC… : livré, testé, au lieu prescrit ? L'étape 0 (constat au navigateur sur le code de départ) jouée, consignée
au registre, et suffisante pour poster le commentaire de D5 sur #1024 tel quel ? Les gates : test-fast 5307, gate-ci 5314, Vitest 3197
au commit `ec3c9e35` ; E2E complet 406 passés / 7 KF-029 **après** `16105cba` — chaque chiffre au journal, au bon commit ? Le run
`e2e-r2` à 18 échecs (dix `supplier-*` en `pool timed out`) est-il dit exactement, et son explication étayée par le journal ? Les
écarts (script Python des mutations, captures d'autres specs) sont-ils exacts ? Le Dev Agent Record n'affirme-t-il que ce qui a
tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e354-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque rapport :
findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou
preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
