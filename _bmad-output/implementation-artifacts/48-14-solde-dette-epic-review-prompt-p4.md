# Prompt — revue de code P4 de la story 48-14 (#1263, #1262, #1255, #1261), Opus, passe complète

Tu relis la story **48-14-solde-dette-epic** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), story de
solde de l'Epic 48, quatre issues, **aucune migration** ; elle partira dans la release **0.19.1**. Kesh tient les livres réels du
Project Lead : **le règlement groupé porte des écritures réelles depuis la 0.19.0** ; rien d'existant (règlement, écriture,
lettrage, audit) ne doit changer. C'est la **quatrième passe** de la revue de code, **complète**, **modèle Opus**, contexte frais,
une seule passe qui exerce tous les axes. Langue du rapport : français.

**Pourquoi complète.** P1 Opus (trois lentilles) 0/0/2/15 → P2 Sonnet complète 0/0/0/4 → remédiation `02387000b` (frontend +
test Rust + fiche) → P3 Haiku ciblée 0/0/1/2 : le MEDIUM P3-1 constate que `02387000b` touche deux modules et réécrit D3, ce
pour quoi CLAUDE.md exclut la passe ciblée. Les deux LOW P3-2 (commentaire de `ReconciliationProposals.svelte`, fin d'un lot) et
P3-3 (fiche : « drapeau » → compteur `settledInBackground`) sont corrigés au commit `8106fce83`.

Worktree `/home/gcorbaz/devel/kesh-48-14`, branche `story/48-14-solde-dette-epic`, partie de `epic/48-corrections-0-20-0` =
`59bdd104f` ; fiche validée à `0a7447001`.

## Lis d'abord, dans cet ordre

1. Les deux derniers commits de code, aplatis : `git -C /home/gcorbaz/devel/kesh-48-14 show 02387000b` puis `show 8106fce83`.
2. Les rapports : `/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p2.md` (P2-1 à P2-4) et `review-p3.md` (P3-1 à P3-3) ; ceux de
   la P1 (`review-p1-{B,E,A}.md`) pour mémoire.
3. La fiche `_bmad-output/implementation-artifacts/48-14-solde-dette-epic.md` en entier (D1 à D8, AC1 à AC15, T1 à T14, M1 à M21,
   Note de version, Dev Agent Record, Change Log) et le registre `e0200-choix-autonomes-e4814.md`.
4. Puis le diff entier de la story, aplati : `git -C /home/gcorbaz/devel/kesh-48-14 diff 0a7447001 HEAD`.

Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e4814/` : `p2-rouge-p21.log`, `p2-vert.log`, `mut-m17.log` à `mut-m20.log`,
`p2-rust-cible.log` (M21), et les gates de clôture `clot-front.log`, `clot.console`, `clot-clippy.log`, `clot-gate-ci.log`,
`clot-gate-ci.console`, `clot-e2e.log` — **ceux-ci peuvent être encore en cours d'écriture** : dis lesquels étaient complets.
Note : les gates de clôture tournent sur `02387000b` ; `8106fce83` ne change qu'un commentaire Svelte et la fiche — vérifie-le.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**
  au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`add`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `p2-cloture.sh`, `p2-rust-cible.sh`, `p2-front.sh`,
  `e2e-e4814.sh`, `mut/muter-front.sh`, `mut/archiver-mutation-rust.sh` : ils se **lisent**) ; ni `make`,
  `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`, `flock`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires

1. **Les remédiations P2 et P3, finding par finding** (P2-1 à P2-4, P3-2, P3-3) : fermé, fermé en partie, non fermé — ligne lue ;
   ce qu'elles ont pu casser à côté.
2. **Lot B, concurrence** (`ReconciliationProposals.svelte`). Le compteur `settledInBackground` ; `reloadMustWait()` ; `load()`
   qui consomme avant son premier `await` ; `flushPendingReload()` **retiré** des `finally` d'`onAccept`/`onReject`, appelé par
   `cancelGesture` et les quatre `onOpenChange(false)`. Suis tous les chemins (cinq modales et leurs fermetures, Accepter/Rejeter
   en succès et en échec, `$effect` de `limit`, réponses dépassées, modale démontée par `{#if loading}`) : relecture double,
   perdue hors des cas que D3 déclare, sous une modale ouverte ; bilan faux ; dépendance d'`$effect` involontaire. Le cas
   déclaré « fenêtre périmée après un lot » : trace côté serveur, **chemin groupé compris**, la garde qui rend un second règlement
   de la même transaction inoffensif pour les livres (`RECONCILIATION_ALREADY_RECONCILED` ou équivalent) — la P3 l'a laissé
   « plausible, non prouvé ».
3. **Les écritures réelles (Lot C)** : charge de `reconciliation.cancelled` inchangée (T8/T9), garde `DbError::Invariant`, ordre
   des verrous de chaque bras.
4. **Les gardes statiques** : `APPELANTS_LIEN_GROUPE`, `fautes_des_appelants_du_lien`, sa sonde, le cinquième cas construit de
   `bank_account_closed_inventory.rs` (P2-2, M21), `fonction_englobante` et `decouper` (non relus à la P3) ; sites non résolus,
   écrits ou non comme angles morts.
5. **Lot A et le manuel.** `docs/manual/fr/user-manual.tex` (paragraphe *Un refus.*, confirmation) contre
   `GroupSettlementModal.svelte` ; D2 et D3 de la fiche contre le `.tex` et le code. Le PDF n'est pas régénéré dans la branche :
   la forme `pdftotext … | /usr/bin/grep -c` rend la version d'avant (le dire, ne pas le compter comme défaut). Vérifie aussi les
   locales DE/IT/EN des manuels si la story les touche.
6. **Lot D** : `kf063_reproduction_deterministe` ; `git -C /home/gcorbaz/devel/kesh-48-14 diff --stat 0a7447001 HEAD --
   crates/kesh-api/src crates/kesh-db/migrations crates/kesh-db/test-schema` doit être vide.
7. **AC et décisions** : AC1 à AC15 et D1 à D8 contre le code, les tests et les journaux.
8. **E2E et recette visuelle** : T6, T6 bis, T7 ; sélecteurs (garde KF-043 : aucun `getByText` sur un libellé traduit) ; E2E
   complet de clôture s'il est fini.
9. **La fiche et le Dev Agent Record** : décomptes recomptés depuis la source, avec leur périmètre ; gates cités avec journal et
   commit ; ce qui n'a pas tourné, écrit.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p4.md` (le seul fichier que tu écris). Pour chaque finding : identifiant (`P4-1`,
`P4-2`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif proposé. Verdict chiffré en
tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). La revue n'exécute rien.
