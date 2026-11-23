# Prompt — revue de code P2 de la story 48-14 (#1263, #1262, #1255, #1261), Sonnet, passe complète

Tu relis la story **48-14-solde-dette-epic** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release
**0.20.0**, Epic 48 — story de solde, quatre issues, **aucune migration**. Kesh tient les livres réels du Project Lead : **le
règlement groupé porte des écritures réelles depuis la 0.19.0** ; rien d'existant (règlement, écriture, lettrage, audit) ne doit
changer. C'est la **deuxième passe** de la revue de code : **complète** (la remédiation de la P1 touche `kesh-db`, le frontend et
le manuel), **modèle Sonnet**, contexte frais, une seule passe qui exerce tous les axes. Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-48-14`, branche `story/48-14-solde-dette-epic`, partie de `epic/48-corrections-0-20-0` =
`59bdd104f` ; fiche validée à `0a7447001`.

## Lis d'abord, dans cet ordre

1. **Le commit de remédiation de la P1, aplati** : `git -C /home/gcorbaz/devel/kesh-48-14 show 752423715` (code, tests, manuel).
   Le commit suivant ne porte que la fiche, le registre et ce prompt :
   `git -C /home/gcorbaz/devel/kesh-48-14 diff --stat 752423715 HEAD`.
2. Les trois rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p1-{B,E,A}.md` (0/0/2/15). Pour chaque finding,
   juge au code si la remédiation le ferme **et** si elle n'introduit rien (la sévérité se déplace vers ce qu'on vient d'écrire).
3. La fiche `_bmad-output/implementation-artifacts/48-14-solde-dette-epic.md` en entier : D1 à D8 (**D2 et D3 révisées à la P1**),
   AC1 à AC15, tests T1 à T14, mutations M1 à M16, Note de version, **Dev Agent Record** (section « Remédiation de la revue de
   code P1 »), **Change Log** ; le registre `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4814.md`.
4. Puis le diff entier de la story, aplati : `git -C /home/gcorbaz/devel/kesh-48-14 diff 0a7447001 752423715`.

Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les journaux (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e4814/`. Ceux de la remédiation : `mut-m12.log` à `mut-m15.log`, `p1r-vitest.log`,
`p1r-vitest-2.log`, `make-user-manual-p1r.log`, `controle-pdf-p1r.log`, `p1r-rust-cible.log` (bases, fmt, nextest ciblé,
KF-063 rejoué, mutation M16) et son script `p1r-rust-cible.sh`, `p1r-front.log` (frontend complet), `p1r-e2e-gates.console`,
`p1r-e2e.log` (T6, T6 bis, T7), `p1r-clippy.log`, `p1r-gate-ci.log`, `p1r-gate-ci.console`, et le script `p1r-e2e-gates.sh`.
Ceux du développement sont nommés par le prompt de la P1 (`48-14-solde-dette-epic-review-prompt-p1.md`). La session de
remédiation a été tuée par systemd-oomd à 04:23 puis reprise à 09:30 : les journaux d'avant l'arrêt sont datés de 04:19 à 04:21.
Captures (thème clair, base jetable, données factices) : `captures/` (développement) et `captures-p1r/` (rejeu des E2E après la
remédiation).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**
  au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`add`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ;
  ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `p1r-rust-cible.sh`, `p1r-e2e-gates.sh`,
  `p1r-front.sh`, `e2e-e4814.sh`, `mut/muter-front.sh`, `mut/archiver-mutation-rust.sh` : ils se **lisent**) ; ni `make`,
  `latexmk`, `pdflatex`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`, `flock`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires

1. **La remédiation, finding par finding.** Les 17 findings de la P1 : fermé, fermé en partie, ou non fermé — avec la ligne lue.
   Ce que la remédiation a pu casser à côté.
2. **Lot B, concurrence — le mécanisme a changé.** Le drapeau `reloadPending` est devenu le compteur `settledInBackground` ;
   `reloadMustWait()` = `anyModalOpen() || busy` ; `load()` consomme le compteur avant son premier `await` et l'ajoute au bilan
   après ; une relecture en échec le rend à l'attente, une relecture dépassée le compte au bilan ; `flushPendingReload()` dans le
   `finally` d'`onAccept` / `onReject` ; `onGroupStale` efface le bilan quand il consomme. Suis **tous** les chemins : les cinq
   modales et leurs fermetures (`onOpenChange(false)`, `cancelGesture`, `confirmGesture`, succès, `closeStale`), *Accepter* /
   *Rejeter* (succès, échec, lancés depuis la confirmation), `$effect` de `limit`, réponses dépassées, modale démontée par
   `{#if loading}` dont l'envoi aboutit ensuite. Une relecture double, perdue, ou sous une modale ouverte ? Un bilan faux (compte
   en trop ou en moins, écrasé par une affectation `lastSuccessCount = …` d'un autre chemin) ? Une lecture de `lastSuccessCount`
   ou de `settledInBackground` avant le premier `await` qui en ferait une dépendance de l'`$effect` ? La doublure
   `ModalSuccessStubGroup.test.svelte` (`globalThis.__stubGroupBackground`) joue-t-elle un chemin réel ?
3. **Les écritures réelles (Lot C)** et **E4** : `group_cancelled_details` — la charge de `reconciliation.cancelled` est-elle
   toujours la même valeur JSON (T8/T9) ? La garde `DbError::Invariant` est-elle atteignable en production, et sinon sans effet ?
   L'ordre des verrous de chaque bras est-il inchangé ?
4. **Les gardes statiques.** `APPELANTS_LIEN_GROUPE`, `fautes_des_appelants_du_lien` et sa sonde : la définition est-elle bien
   écartée (et seulement elle) ; un appel qualifié (`self::`, `super::grouped_settlements::`) ou sous un alias échappe-t-il ;
   l'exigence « `ensure_open_in_tx(` avant l'appel » peut-elle être satisfaite par un commentaire ou une chaîne (copie masquée) ?
   Le cas « entre les deux audits » et M16. Inventorie les sites non résolus : ce que l'inventaire ne voit pas, écrit ou non comme
   angle mort.
5. **Lot A et le manuel.** `user-manual.tex:3579` (paragraphe *Un refus.*) contre `GroupSettlementModal.svelte` (`showAccount`,
   `submitBlocks`, `handleRefusal`) : les conditions s'excluent-elles, dans le bon ordre, pour **chaque** état (arrondi sans
   compte, plafond, illisible, non relue, écart non nul relu, écart nul relu) ? Le **PDF n'est pas régénéré dans la branche** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` rend la version d'avant (le
   dire, ne pas le compter comme défaut) ; le contrôle de la compilation locale est `controle-pdf-p1r.log` : discrimine-t-il ?
   `:3490` (confirmation) toujours vrai après la révision de D3 ? D2 de la fiche suit-elle le `.tex` ?
6. **Lot D.** `kf063_reproduction_deterministe` après E3/B5/A8 : la prémisse est-elle maintenant vérifiée sur ce qu'elle juge ?
   `crates/kesh-api/src/`, migrations et schéma de test intacts :
   `git -C /home/gcorbaz/devel/kesh-48-14 diff --stat 0a7447001 HEAD -- crates/kesh-api/src crates/kesh-db/migrations crates/kesh-db/test-schema`
   doit être vide.
7. **AC et décisions.** AC1 à AC15 et D1 à D8 contre le code, les tests et les journaux, après la remédiation (AC7 surtout).
8. **E2E et recette visuelle.** T6, T6 bis, T7 rejoués après la remédiation (`p1r-e2e.log`) ; sélecteurs (garde KF-043 : aucun
   `getByText` sur un libellé traduit).
9. **La fiche et le Dev Agent Record.** Décomptes de la remédiation recomptés depuis `git diff fbeee9e53 752423715` ; gates cités
   avec journal et commit ; ce qui n'a pas tourné écrit (E2E complet : à la clôture) ; mutations M12 à M16 et leurs journaux.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p2.md` (le seul fichier que tu écris). Pour chaque finding : identifiant (`P2-1`,
`P2-2`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif proposé. Verdict chiffré en
tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). La revue n'exécute rien ; les gates qui
ont tourné sont cités au Dev Agent Record avec leurs journaux.
