# Story v0171-correctifs — revue de code P3, passe ciblée (une lentille — modèle Sonnet)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, `npm`, `npx`, `make`, `latexmk`,
`python3`, `node`, `docker`, `curl`, `claude`, ni aucun script du dépôt (`scripts/prepare-release.sh` compris), ni `sed -i`,
`rm`, `pkill`, `killall`, `gh` qui écrit ; aucune redirection (`>`, `>>`, `tee`) ; aucun `git checkout`, `switch`, `reset`,
`stash`, `rebase`, `commit`. Permis : `git log`, `git show`, `git diff`, `git grep`, `/usr/bin/grep`, `sed -n`, `cat`, `ls`,
`wc`, `gh issue view`. Le **seul** fichier que tu écris est ton rapport. Aucune pièce réelle (`~/kesh/` interdit).

⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`.

Langue : français soigné. Contexte frais.

## Ce que tu relis

Les **deux commits `d5e44cd0` et `3d21d1aa`** du worktree `/home/gcorbaz/devel/kesh-0171` (branche `release/0.17.1`) :
`git -C /home/gcorbaz/devel/kesh-0171 show d5e44cd0 3d21d1aa`. Ce sont les remédiations, par l'orchestrateur et sans code,
de la revue P1 (`/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p1-{B,E,A}.md`) et de la passe ciblée P2
(`/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p2.md`) — lis les quatre rapports. Elles récrivent le Dev Agent Record de
la fiche `_bmad-output/implementation-artifacts/v0171-correctifs.md` et complètent l'issue **#883** (corps et commentaire :
`gh issue view 883 --comments`).

## Axes

1. **A MEDIUM-1** : la section « Rouges constatés avant tout correctif » dit-elle maintenant exactement ce que porte
   `/home/gcorbaz/devel/kesh-gate-logs/v0171/rouges-backend.log` (tests joués, rouges, vert, résumé) ? Cite les lignes du
   journal.
2. **Le raisonnement ajouté** : « parseur de production inchangé, seuls `mod tests` / `mod scan_qr_tests` reçoivent des
   ajouts » — vérifie-le par `git diff 910e5d5a..HEAD -- crates/kesh-qrbill/src/parser.rs crates/kesh-api/src/routes/supplier_invoices.rs`
   (hunks et leur position dans le fichier : sont-ils bien **dans** le module de test ?). La conclusion « T6 et T7 auraient
   rendu le même vert, sans l'avoir montré par exécution » est-elle honnête et juste ?
3. **Les LOW recalés** : treize fichiers Vitest à `it(` neufs et un quatorzième à assertions modifiées
   (`git diff 68374eb9..HEAD -- 'frontend/**/*.test.ts'` : compte les fichiers où des lignes `+` contiennent `it(`) ;
   cause du jumeau de T4 sous M3 (`parser.rs:90-100`, payload du test dans `qr_decode.rs`) ; écart à AC-2(c) ; fondement
   d'AC-9 ; en-tête « C-v0171-1 à 25 » contre `v0171-choix-autonomes.md`.
4. **Le Change Log** : l'entrée de la revue P1 — décomptes (B 0/0/0/3, E 0/0/0/3, A 0/0/1/4) recomptés depuis les trois
   rapports ; les LOW de code non traités sont-ils tous dans **#883** (`gh issue view 883`) ? un LOW des rapports oublié ?
5. **Aucune réécriture d'historique** : la remédiation a-t-elle modifié une entrée passée du Change Log au lieu d'en
   ajouter une ? Les rectifications du record sont-elles datées ?
6. **La remédiation P2** (`3d21d1aa`) : chaque finding de la P2 (MEDIUM-1, LOW-1, LOW-2) est-il traité ? Tous les LOW de
   **code** des quatre rapports non traités figurent-ils dans #883 (corps ou commentaire), y compris ceux de la lentille A ?
   L'entrée P2 du Change Log est-elle exacte et l'entrée P1 laissée telle quelle ? L'annotation de l'en-tête est-elle juste ?
7. **Commit** : `git log 910e5d5a..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit être vide.

## Rapport

`/home/gcorbaz/devel/kesh-gate-logs/v0171-review-p3.md`. Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW),
lieu, ligne lue copiée, défaut, remède. Toute affirmation de présence ou d'absence s'appuie sur une sortie de `grep`/`git`
copiée. Termine par les **axes exercés** et **non exercés** (avec la raison), puis le décompte
CRITICAL n · HIGH n · MEDIUM n · LOW n. Un « 0 » sans la liste des axes ne compte pas.
