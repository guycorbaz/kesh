# Prompt — passe ciblée P3 de la revue de code de la story 48-14 (Haiku, une lentille, contexte frais)

Tu relis **un seul commit** de la story **48-14-solde-dette-epic** de Kesh (comptabilité suisse ; Rust, SvelteKit 5 runes) :
la remédiation des quatre LOW de la revue P2. Kesh tient des livres réels ; ce commit ne touche ni le code d'écriture comptable
ni les migrations. Langue du rapport : français. **Modèle : Haiku**, contexte frais, **une seule lentille** : *la sévérité se
déplace vers ce qu'on vient d'écrire* — cherche ce que ce commit casse ou ne ferme pas.

Worktree `/home/gcorbaz/devel/kesh-48-14`. Le diff unique, aplati, à lire en entier d'abord :

```
git -C /home/gcorbaz/devel/kesh-48-14 show 02387000b
```

Puis le rapport qui l'a motivé : `/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p2.md` (findings P2-1 à P2-4). Pour le
contexte seulement : D3 et AC7 de la fiche `_bmad-output/implementation-artifacts/48-14-solde-dette-epic.md` (telles que le
commit les réécrit), et la section « Remédiation de la revue de code P2 » de son Dev Agent Record. Journaux (lecture seule) dans
`/home/gcorbaz/devel/kesh-gate-logs/e4814/` : `p2-rouge-p21.log`, `p2-vert.log`, `mut-m17.log` à `mut-m20.log`,
`p2-rust-cible.log` (M21), et les gates de clôture `clot-front.log`, `clot.console`, `clot-clippy.log`, `clot-gate-ci.log`,
`clot-gate-ci.console`, `clot-e2e.log`.

## Ce que tu vérifies

1. **P2-1** (`ReconciliationProposals.svelte`) : `flushPendingReload()` est retiré du `finally` d'`onAccept` et d'`onReject`.
   Un règlement abouti en arrière-plan peut-il maintenant rester **sans relecture** dans un cas que D3 réécrite ne déclare pas ?
   Une relecture double ? Le bilan, les refus et l'erreur d'un lot restent-ils ? Les deux tests neufs rougissaient-ils sur le
   code d'avant (`p2-rouge-p21.log`) et pour quelle raison ?
2. **P2-3** : chacun des tests neufs (« relecture en échec », « relecture dépassée en échec », « fenêtre périmée qui consomme un
   règlement en attente », l'`it.each` qui lit le bilan) tue-t-il le mutant que le DAR lui attribue (`mut-m17.log` à
   `mut-m20.log` : le test qui rougit est-il le bon) ? Un test qui passerait à vide (assertion vraie par construction) ?
3. **P2-2** (`bank_account_closed_inventory.rs`) : le cinquième cas construit monte-t-il vraiment une garde **seulement** en
   commentaire et en chaîne (le `replacen` a-t-il eu lieu : `assert_ne!`) ? M21 (`p2-rust-cible.log`) rougit-elle sur ce cas ?
   Les trois angles morts écrits au doc-comment sont-ils exacts au code ?
4. **P2-4** : la fiche réécrite dit-elle ce que fait le code (D3, AC7, T5) ? Reste-t-il une phrase que le commit contredit
   (`git grep -nE 'finally|fin du lot|fin d.un lot' -- _bmad-output/implementation-artifacts/48-14-solde-dette-epic.md`) ?
   #1417 et #1418 sont-ils cités ? Les décomptes de la section P2 du DAR se recomptent-ils depuis le diff ?
5. **Gates de clôture** : les journaux `clot-*` s'ouvrent-ils sur `02387000b` avec un arbre de code propre ? L'E2E complet
   (`clot-e2e.log`) n'a-t-il que des échecs attendus (`docs/testing.md` § *Les échecs attendus* : sept KF-029, fichier par fichier) ?

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire** ; **aucune
  redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se déclare au rapport.
- **Aucun `rm`**, ni `kill`, `pkill`, `killall`.
- **Aucun `git` qui écrit** (`checkout`, `switch`, `restore`, `stash`, `add`, `commit`, `reset`, `merge`, `rebase`, `push`).
- **Rien qui exécute, compile ou écrit** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script de `kesh-gate-logs` (`p2-cloture.sh`, `p2-front.sh`, `p2-rust-cible.sh`,
  `e2e-e4814.sh`, `mut/muter-front.sh` se **lisent**) ; ni `make`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**,
  `sed -i`, `playwright`, `vitest`, `flock`. **Aucune commande `gh`**, aucun accès réseau.
- Permis : `git show`, `git diff`, `git log`, `git grep`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection
  ni `system()`), `cat`, `ls`, `wc`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` (avec `-F` pour une chaîne littérale) ou `git grep`.
- **Tout finding CRITICAL ou HIGH qui affirme une absence ou un anti-pattern** s'appuie sur une sortie `grep -nF` (ou `sed -n`)
  copiée au rapport.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/e4814/review-p3-haiku.md` (le seul fichier que tu écris). Verdict chiffré en tête ; pour
chaque finding : identifiant (`P3-1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario,
correctif proposé. **Liste des axes (1 à 5) exercés ET non exercés** : un « 0 finding » sans elle ne compte pas.
