# Prompt — revue de code P1 de la Story 35-7 (#1000), trois lentilles, première passe complète

Tu relis la **Story 35-7** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **le champ de compte de contrepartie d'une écriture manuelle ou d'un éclatement ne montre plus un
texte qui contredit le compte envoyé au serveur**. Deux chemins : (D2) sur un champ de compte strict, le texte tapé puis
abandonné est jeté au `blur` et le champ réaffiche le compte retenu, sans jamais le nullifier (#271) ; (D1) un compte que le
dialogue ne propose plus (le compte de la banque, identifié après coup) est retiré de la sélection par
`keepOfferedCounterparty`, et le message existant s'affiche. **Frontend seul ; aucune clé i18n ; aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-e357`, branche `story/35-7-autocomplete-compte-hors-liste`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e357 diff 137993e5 926854f7` (`137993e5` = base `epic-35/base3` ; `926854f7` = dernier commit de
code ; `34f93cc9` ne porte que le compte rendu). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku).
Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-7-autocomplete-compte-hors-liste.md` en entier (D1 à D3, AC1 à AC7, tests et
mutations M1 à M7, Dev Notes, **Change Log** et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e350.md`
(C-e350-8, -36, -37) et `e35-choix-autonomes-e357.md` (C-e357-1 à 4). Ne prends pas le Dev Agent Record pour argent comptant :
recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e357/` : `rouges.log`,
`verts-cibles.log`, `test-fast-ci.log`, `gate-ci.log`, `front-*.log`, `e2e-r1-rouge.log`, `e2e-r2-complet.log`,
`e2e-r4-cible.log`, `e2e-r5-complet.log`, `manuel.log`, `mut/`. **Captures** (thème clair) sous
`/home/gcorbaz/devel/kesh-gate-logs/e357/captures/` (les trois `e357-*.png` ; les autres viennent d'autres specs) — regarde-les si
ton outil lit les images.

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

1. **D2, `handleBlur`** (`AccountAutocomplete.svelte`) : ordre mode dégradé → sans brouillon → strict (`draft = null`) →
   `allowClear` inchangé ; `onSelect` jamais appelé sur un champ strict ; le champ réaffiche le compte retenu, ou reste vide si
   la valeur est nulle. Interaction avec le `mousedown` d'une option (le `blur` part-il **avant** la sélection et jette-t-il un
   choix en cours ?), avec la touche Entrée/Échap, avec l'effet qui annule le brouillon sur une valeur venue de l'extérieur
   (`:183-190` de la base). Les textes « 4 consommateurs » réécrits ou gardés selon l'inventaire de D2 : `grep -nF` à l'appui.
2. **D1, `keepOfferedCounterparty`** (`reconciliation-counterparties.ts`) : les six cas d'AC2 ; un compte **absent** de la liste
   garde son identifiant (liste non chargée). Les deux `$effect` (`ManualMatchModal.svelte`, `TransactionSplitModal.svelte`)
   n'écrivent **que si la valeur change** : boucle réactive possible ? Lecture de `splits` dans l'effet qui écrit `splits[i]` :
   l'effet se relance-t-il sur sa propre écriture, et termine-t-il ? Une ligne restée proposée est-elle intacte ?
3. **Les messages et le bouton** : *Écriture manuelle* « Compte de contrepartie obligatoire » et bouton désactivé ; *Éclater*
   « Ligne N : compte requis », la première ligne en défaut, `clientError` ligne par ligne (compte puis montant). Aucun texte neuf,
   `sitesTotal` 3026 inchangé.
4. **Les consommateurs** : les sept sites d'`AccountAutocomplete` (cinq stricts, deux `allowClear`) — lesquels changent de
   comportement au `blur` ? `VatPurchaseAssistant` (`:159`, `:204`), `JournalEntryForm` : un geste utilisateur d'aujourd'hui
   (taper un numéro partiel, quitter, revenir) est-il cassé ? `git grep -n 'AccountAutocomplete' frontend/src` à l'appui.
5. **Les tests** : AC1 (mode dégradé : `onSelect` en `vi.fn()` nu, `value` non mise à jour, sans quoi M3 survit) ; AC3 par l'hôte
   `CounterpartyModalHost.test.svelte` (C-e357-1 : le `rerender` passait à vide — l'hôte prouve-t-il vraiment le passage du prop
   de `null` à l'identifiant, et non un montage déjà juste ?) ; le mock i18n interpolant (C-e357-2) ne masque-t-il pas un défaut
   de clé ? AC7 E2E : le corps du `POST /api/v1/reconciliation/manual` lu par `waitForRequest` porte l'identifiant **affiché** ;
   la description ajoutée par `926854f7` change-t-elle ce que le test prouve ? Un test qui passe **à vide** ?
6. **Le manuel contre le code** (`user-manual.tex:3186`, et `:3121`, `:3180` inchangés), **le PDF aplati** (« retiré du champ »
   présent) ; le CHANGELOG `## [0.17.7]` *Corrigé*, `refs #1000`. Chaque phrase est-elle vraie au code (« se vide » : la valeur
   aussi, ou seulement le texte ?) ?
7. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`rouges.log`, `mut/`) ? M1 à M7 tuées chacune par
   le test qu'elle vise ? Aucun attendu de test existant retouché sans motif (`git diff 137993e5 926854f7 -- '*.test.ts'`, lignes
   `-`) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `handleBlur`, `keepOfferedCounterparty`, les deux `$effect`, l'hôte de test, la spec E2E. Cherche : un ordre
d'événements (`blur` / `mousedown` / `click`) qui jette une sélection légitime ; un effet qui boucle ou qui écrase une saisie ; une
comparaison `null`/`undefined`/nombre/chaîne fragile ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

Champ strict, valeur nulle, brouillon vide ; valeur résolue et brouillon identique au libellé ; brouillon qui correspond
exactement à un compte sans qu'il ait été choisi (numéro complet tapé, puis `blur`) ; mode dégradé puis liste rechargée ;
`bankLedgerAccountId` qui passe de `null` à un identifiant puis revient ; liste des comptes qui arrive après la sélection ;
*Éclater* à trois lignes dont deux portent le compte de la banque ; compte archivé choisi avant archivage ; `allowClear` avec
brouillon. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC7 : livré, testé, au lieu prescrit ? L'étape 0 (le grep de D2) jouée et citée au Change Log ? Les rouges de
départ (Vitest 8 sur 78 ; E2E d'AC7 « zzz ») ; les mutations M1 à M7 ; les gates au dernier commit de code `926854f7` (test-fast
5307, gate-ci 5314, Vitest 3201, E2E 405 passés / 7 KF-029) — chaque chiffre se retrouve-t-il au journal, horodaté après le
commit ? Le premier E2E complet (9 échecs, au commit `635fc5f2`) et sa correction sont-ils dits exactement ? Les écarts déclarés
(`cat >>` contre la consigne Edit ; captures d'autres specs dans `e357/captures/` ; spec jetable hors dépôt) sont-ils exacts et sans
effet sur le livré ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e357-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque rapport :
findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue copiée, rejeu ou
preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la liste**.
