# Story v017-5 — revue de code P1 (trois lentilles B, E, A — modèle Opus)

⛔ **Lecture seule stricte.** Aucune commande qui écrit, compile ou exécute : ni `cargo`, ni `npm`, ni `npx`, ni `make`,
ni `latexmk`, ni `python3`, ni `docker`, ni `curl`, ni `claude`, ni aucun script du dépôt (`scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`… compris), ni `sed -i`, ni `rm`, ni `pkill`, ni `killall` ; **aucune
redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad ; aucun `git checkout`,
`switch`, `reset`, `stash`, `rebase` sous aucune forme. Permis : `git log`, `git show`, `git diff`, `grep`, `rg`, `sed -n`,
`cat`, `ls`, `wc`, `pdftotext f.pdf -` vers la sortie standard seulement. Le **seul** fichier que tu écris est ton
rapport. ⛔ **Aucune pièce réelle** : ni `~/kesh/`, ni `~/kesh-test/`, ni le dossier de projet de Guy, ni Paperless — les
fixtures de la story sont synthétiques et suffisent.

Langue : français soigné. Contexte frais : tu n'as lu aucune passe précédente.

## Ce que tu relis

- Worktree : `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`, partie de `8e126cfd`.
- Le diff : `git -C /home/gcorbaz/devel/kesh-0176 diff 8e126cfd..HEAD` (code, tests, fixtures, textes, manuels).
- La fiche : `_bmad-output/implementation-artifacts/v017-5-qr-scanne.md` (AC-1 à AC-13, tests T1 à T12, mutations M1 à
  M14, Dev Agent Record) ; le registre : `v017-5-choix-autonomes.md` (C-v017-5-1 à C-v017-5-18).
- Issue : #854 (P2) — le QR d'une QR-facture scannée n'est pas lu à l'import.
- Les journaux du développement : `/home/gcorbaz/devel/kesh-gate-logs/v0175/` (lecture seule).
- Les captures : `/home/gcorbaz/devel/kesh-gate-logs/v0175/captures/` (ouvre-les, thème clair).

## Les trois lentilles

Une lentille par agent ; chacune écrit son rapport :

- **B (Blind Hunter)** — le diff seul, sans la fiche : défauts de code, de tests et de textes visibles au diff.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0175-review-p1-B.md`.
- **E (Edge Case Hunter)** — chaque branche et chaque borne : `decode_pages` (première et seconde passe, cap, erreurs des
  deux temps, `Ok(None)` d'un rendu), `render_scale` (`NaN`, infini, nul, pages hors normes), la relecture à la
  réactivation (`has_qr`, `ImageDecode`, course à zéro ligne, ligne `completed` détachée), la garde du verrou de run.
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0175-review-p1-E.md`.
- **A (Acceptance Auditor)** — chaque AC contre le code livré et le Dev Agent Record : ce qui est déclaré a-t-il tourné,
  les décomptes se recomptent-ils, les écarts à la fiche sont-ils écrits ?
  Rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0175-review-p1-A.md`.

## Axes obligatoires (chaque lentille les exerce ou dit pourquoi non)

1. **La cause et sa preuve** : T4 au commit rouge (`67727ad1`) et au correctif ; les paliers de dégradation joués
   (C-v017-5-16) — l'ordre écrit d'avance a-t-il été suivi, sans réglage sur le résultat ?
2. **Les deux passes** : un document lu au premier temps ne paie rien de plus ; une erreur du second rendu ne change jamais
   le diagnostic ; les pages en erreur au premier temps ne sont pas rendues au second.
3. **Bornes anti-DoS non relâchées** : `DecodeConfig::default()` (20 pages, 2000 px), `MAX_IMAGE_DIMENSION`,
   `MAX_IMAGE_ALLOC`, `KESH_INBOX_MAX_PDF_PAGES` ; aucun rendu au-delà de 2000 px de côté ; le plafond de 1 Mio des
   fixtures (C-v017-5-17).
4. **Données existantes** : rien n'est relu ni réécrit d'office ; la relecture n'écrit que les colonnes QR d'une ligne
   `discarded` sans QR (garde `creditor_iban IS NULL`, une instruction) ; aucune migration, aucun bump ; une ligne avec QR
   ou `to_complete`/`completed` n'est jamais réécrite.
5. **Le verrou de run (AC-6-bis)** : la fuite constatée (T12 rouge avec un pool réglé comme la production), la garde
   `close_on_drop`, ses effets de bord (coût, pool), et la fidélité de T12 au pool de production (`main.rs`).
6. **Les textes** : le libellé « Aucune QR-facture exploitable » dans les quatre locales, les replis Svelte, la garde
   G13 (`REPLIS_A_SITE_UNIQUE`, commentaire « quatre-vingt-seize », recompte), `README.md`, l'E2E E2-a.
7. **Les manuels — `.tex` ET PDF** : `docs/manual/fr/user-manual.tex` (§ *Documents à traiter*, *Faire relire le QR d'un
   document*, *Un import long*) et `admin-manual.tex` (*Réglage `KESH_INBOX_MAX_PDF_PAGES`*, *La durée d'un import*,
   *Passer à la 0.17.0*, *Intégrations par l'API (0.17.0)*), et **leurs PDF**. Chaque phrase doit être vraie au code
   livré. Lis les PDF aplatis : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ; pour compter
   des occurrences, `timeout 60 sh -c "pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF 'motif' | wc -l"` —
   jamais un motif `{0,N}`. Le grep de la valeur « Sans QR-code » / « sans QR-code » doit rendre zéro occurrence
   d'écran dans les `.tex` et les PDF (seuls les emplois génériques de l'inventaire d'AC-7 restent).
8. **`docs/api-external.md` et `CHANGELOG.md`** : `none` redéfini, réactivation relue, durée de la route ; entrée
   *Corrigé* en `refs #854`, sans mot-clé de fermeture.
9. **Les commits** : `git log 8e126cfd..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
   doit être vide.
10. **Les mutations** déclarées (M1 à M14) : chacune a-t-elle un test qui la tue, et la déclaration est-elle crédible au
    code des tests ?

## Le rapport

Pour chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), ligne lue copiée, défaut,
remède proposé. Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée. Termine par :

- la liste des **axes exercés** et celle des **axes non exercés**, avec la raison ;
- le décompte final : CRITICAL n · HIGH n · MEDIUM n · LOW n.

Un « 0 finding » sans la liste des axes exercés ne compte pas comme passe.
