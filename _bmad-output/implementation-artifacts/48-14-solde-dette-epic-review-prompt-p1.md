# Prompt — revue de code P1 de la story 48-14 (#1263, #1262, #1255, #1261), trois lentilles Opus, première passe complète

Tu relis la story **48-14-solde-dette-epic** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release
**0.20.0**, Epic 48 — story de solde, quatre issues, **aucune migration**. Kesh tient les livres réels du Project Lead :
**le règlement groupé porte des écritures réelles depuis la 0.19.0** ; rien d'existant (règlement, écriture, lettrage, audit) ne
doit changer. Quatre lots :

- **Lot A — #1263** : après un refus `difference_account_required` et la relecture des restes dus, quand le sélecteur du compte
  de l'écart n'est pas affiché (arrondi sans compte d'arrondi, écart au-delà du plafond, montant illisible), la fenêtre ne dit
  plus « Choisissez le compte de l'écart » mais « Rien n'a été enregistré : voyez ce qui retient encore l'enregistrement. »
  (`GroupSettlementModal.svelte`, `handleRefusal` ; clé `reconciliation-group-reread-blocked`, quatre locales ; manuel).
- **Lot B — #1262** : un règlement groupé qui aboutit alors que sa fenêtre a été fermée pendant l'envoi appelle la prop neuve
  `onSettledInBackground` ; le parent (`ReconciliationProposals.svelte`) relit aussitôt si aucune des **cinq** modales n'est
  ouverte (quatre fenêtres et la confirmation « déjà comptabilisée »), sinon arme `reloadPending` et relit **à la fermeture**, une
  seule fois (`flushPendingReload` par `queueMicrotask`, drapeau éteint dans `load()` avant son premier `await`).
- **Lot C — #1255** : `grouped_settlements.rs` — trois aides privées (`lock_entry_fiscal_year_in_tx`,
  `unlink_group_transaction_in_tx`, `audit_group_cancelled_in_tx`) remplacent le code dupliqué de `cancel_in_tx` et
  `supplier_cancel_in_tx` ; gardes statiques de `bank_account_closed_inventory.rs` ajustées.
- **Lot D — #1261 (KF-063)** : tests seulement (`admin_auto_backup_e2e.rs`) — l'écrivain `PleinApres` juge la limite sur la
  position (une réécriture d'en-tête de zip réussit) ; balayages de 128 limites ; sonde et reproduction déterministe ignorée.

Worktree `/home/gcorbaz/devel/kesh-48-14`, branche `story/48-14-solde-dette-epic`, partie de `epic/48-corrections-0-20-0` =
`59bdd104f` ; fiche validée à `0a7447001`. Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-48-14 diff 0a7447001 96f160d4a` — les commits de code : `58e7f8682` (Lots A et B, manuel,
i18n), `dbe67c669` (caractérisation **avant** réécriture ; Lot D), `2f0ab63b0` (réécriture du Lot C, gardes), `8077fb851` (E2E),
`96f160d4a` (T7 corrigé, **dernier commit de code**) ; les commits suivants ne portent que la fiche, le `sprint-status.yaml`, le
registre et ce prompt — vérifie par `git -C /home/gcorbaz/devel/kesh-48-14 diff --stat 96f160d4a HEAD`. Langue du rapport : français. **Modèle : Opus**, contexte frais,
trois lentilles en parallèle.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-14-solde-dette-epic.md` en entier (D1 à D8, AC1 à AC15, tableau des tests T1 à T14 et
des mutations M1 à M11, Note de version, **Dev Agent Record**, **Change Log**), le registre
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4814.md`, et le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e4814/validation-p3.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4814/` — rouges et verts
(`rouge-t1-t5.log`, `vert-t1-t5.log`, `rouge-t12.log`, `t13-ancien-ecrivain.log`, `t14-ancien-ecrivain.log`,
`carac-releve-ancien-code.log`, `passage-2.log` (caractérisation sur l'archive de l'ancien code, reproduction de KF-063),
`carac-apres.log`, `inventaire-rouge.log`, `rouge-t11bis.log`, `inventaire-vert.log`), mutations (`mut-m1.log` … `mut-m11.log`,
scripts dans `mut/`), E2E (`e2e-cible-t6-t7.log`, `e2e-t7-bis.log`, `e2e-m1.log`, `e2e-m2.log`, `e2e-m1.diff`,
`e2e-m2.diff`, scripts `e2e-e4814.sh`, `e2e-cible.sh`, `e2e-t7-puis-gates.sh`), étape 0 (`etape0-echap-pendant-envoi.log` et la
sonde `etape0-echap-envoi-e4814.test.ts`), KF-063 (`kf063-repro.log`), gates (`bases-review.log`, `test-fast-ci.log`,
`gate-ci.log`, `gate-ci.console`, `front-final.log`, `front-garde-build.log`, `clippy-1.log`), manuel (`make-user-manual.log`),
les scripts `jouer-archives.sh`, `gates-review.sh` et `alerte.md`. Les gates qui ont tourné : `test-fast --ci` 6175/6175,
`gate-ci` 6192/0/19, frontend complet 4805/4805, à `96f160d4a` ; l'**E2E complet n'a pas tourné** (il tourne à la clôture).
**Captures** (thème clair, base jetable `kesh_e2e_e4814`, données factices) : `/home/gcorbaz/devel/kesh-gate-logs/e4814/captures/`
— `48-14-refus-apres-relecture-arrondi.png`, `…-plafond.png`, `…-non-relue.png` (AC1 à AC3),
`48-14-rouverte-intacte-liste-perimee.png`, `48-14-liste-relue-apres-fermeture.png` (AC6, AC8),
`48-14-confirmation-restee-ouverte.png` (AC7). Regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**
  au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `jouer-archives.sh`, `e2e-e4814.sh`,
  `e2e-cible.sh`, `mut/muter-front.sh`, `mut/archiver-mutation-rust.sh`, `mut/builds-e2e.sh` : ils se **lisent**) ; ni `make`,
  `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les AC et décisions.** AC1 à AC15 et D1 à D8 contre le code, les tests et les journaux. Un AC sans test qui peut rougir est
   déclaré filet, avec sa raison. Les écarts à la fiche sont-ils déclarés au Dev Agent Record (dont : décompte anti-muet 45 → 44,
   `APPELS_D_AUDIT`, la preuve du rouge naturel de T6/T7 remplacée par les rejeux M2/M1, l'archive nextest de l'ancien code) ?
2. **Les écritures réelles (Lot C).** Le diff de `grouped_settlements.rs` bloc par bloc : l'ordre des verrous de **chaque** bras
   est-il inchangé (factures, écriture et exercice, `ensure_open_in_tx`, lectures, refus, écritures, `recheck_open_locked_in_tx` en
   dernier, audits) ? La charge de `reconciliation.cancelled` est-elle la même à l'octet de la valeur JSON (clés, `serde_json` sans
   `preserve_order`, valeurs) ? T8/T9 sont-ils **verts sur l'archive de l'ancien code** (`passage-2.log`) **et** sur le nouveau
   (`carac-apres.log`) ; les valeurs attendues sont-elles en dur (et non relues par le même extracteur des deux côtés) ? Le second
   groupe est-il vraiment intact ? M5 à M8 tuées pour la bonne raison (M7 : `OptimisticLockConflict` au geste) ?
3. **Les gardes statiques.** `bank_account_closed_inventory.rs` : registre, `GESTES_VERROUILLANTS_CODE` (7), `APPELS_D_AUDIT`, le
   minimum des positions dans `fautes_en_dernier`, les douze cas de `the_last_acquisition_rule_sees_each_locking_token`, le cas
   « suit l'aide d'audit ». Un angle mort neuf : une aide appelée sous un autre nom, un appel d'aide placé **après** la
   revérification, la troisième aide oubliée par une évolution future ? `letterings_lexical.rs` reste-t-il juste ?
4. **Lot B, concurrence.** Tous les chemins de fermeture des cinq modales : `onOpenChange(false)` des quatre, `cancelGesture`,
   `confirmGesture` (*Continuer quand même*), succès, `closeStale`, refus de la transaction, relecture par `limit`, *Accepter* /
   *Rejeter* (et leur échec, cas assumé). Une relecture double ? Une relecture perdue ? Une relecture sous une modale ouverte ?
   `reloadPending` non réactif : un `$effect` qui en dépendrait ? La modale démontée pendant un `load()` (branche `{#if loading}`)
   et l'envoi qui aboutit ensuite ? Les doublures de T5 jouent-elles l'ordre réel des appels ?
5. **Lot A.** Les trois états (et `unreadable`) atteignent-ils la branche neuve, et elle seule ? Un état où le sélecteur est caché
   mais où un compte débloquerait l'envoi ? La phrase cohabite-t-elle avec la liste « non relue » et l'avis d'arrondi ? Clé aux
   quatre catalogues, `sitesTotal` 3503 → 3504 juste ?
6. **Lot D.** L'écrivain de D6 : partiel, `Err(StorageFull)` à la limite, jamais `Ok(0)` ; T12 ; les balayages couvrent-ils ce
   qu'ils prétendent (fenêtre `[S − 160, S − 32)` dans le répertoire central ; `S` mesuré avant les dépôts) ? La reproduction de
   KF-063 est-elle vraiment déterministe (prémisse « deux tours identiques jusqu'au manifeste » vérifiée par le test) ? Aucun
   fichier de `crates/kesh-api/src/` touché :
   `git -C /home/gcorbaz/devel/kesh-48-14 diff --stat 0a7447001 HEAD -- crates/kesh-api/src crates/kesh-db/migrations crates/kesh-db/test-schema`
   doit être vide.
7. **Le manuel** (`docs/manual/fr/user-manual.tex`, `\label{sec:reglement-groupe}`, paragraphe *Un refus.*) : dit-il ce que fait le
   code (sélecteur seulement si une facture n'a pas pu être relue ; relus tous, vérifier l'écart ; enregistrement retenu → « rien
   n'a été enregistré ») ? Plus aucune phrase « le sélecteur s'affiche quand même » ? `:3490` (confirmation) toujours vrai après D3 ?
   Le **PDF n'est pas régénéré dans la branche** : forme du contrôle,
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'quand même'` — le PDF de la branche
   est celui d'avant (le dire, ne pas le compter comme défaut) ; la compilation locale a été contrôlée au développement
   (`make-user-manual.log`).
8. **E2E et recette visuelle.** T6, T6 bis, T7 : rougissent-ils sous M2 / M1 (`e2e-m2.log`, `e2e-m1.log`) pour la raison de
   l'AC ? Les captures montrent-elles chaque état nommé ? Les sélecteurs respectent-ils la garde KF-043 (aucun `getByText` sur un
   libellé traduit) ?
9. **La fiche, la Note de version, le Dev Agent Record.** Décomptes recomptés depuis le diff (tests ajoutés par fichier), gates
   cités avec journal et commit, ce qui n'a pas tourné écrit (E2E complet : à la clôture).

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche) — défauts de code, d'état, de concurrence, de régression ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites, axes 2 à 6 surtout.
- **A — Acceptance Auditor** : chaque AC et chaque décision contre le code, les tests, les journaux et les captures (axes 1, 7, 8, 9).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md` (le seul
fichier que tu écris). Pour chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et
ligne, extrait cité, scénario, correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding »
sans elle ne compte pas). La revue n'exécute rien ; les gates qui ont tourné sont cités au Dev Agent Record avec leurs journaux.
