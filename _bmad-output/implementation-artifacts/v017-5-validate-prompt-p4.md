# Prompt — validation P4 de la fiche v017-5 (release 0.17.0 : #854), passe ciblée, Opus

Tu valides la **dernière remédiation** de la fiche de la Story v017-5 de Kesh (comptabilité suisse, Rust/Axum,
SvelteKit, MariaDB 10.11) : lire la QR-facture d'un document **scanné** à l'import de pièces (#854). Passe **ciblée** :
une seule lentille, braquée sur le diff de la remédiation de la validation P3, en **contexte frais**. Langue du rapport :
**français**. Modèle : **Opus**.

Worktree `/home/gcorbaz/devel/kesh-0176`, branche `story/v017-5-qr-scanne`. Le diff à relire :

```sh
git -C /home/gcorbaz/devel/kesh-0176 diff ec6cfbac HEAD -- _bmad-output/implementation-artifacts/
```

(`ec6cfbac` = la remédiation de la P2 ; `HEAD` = la remédiation de la P3, commit
`docs(v017-5) : remédiation de la validation P3 (refs #854)`). Lis la fiche
`_bmad-output/implementation-artifacts/v017-5-qr-scanne.md` et le registre `v017-5-choix-autonomes.md` **en entier** pour
le contexte, mais ne cherche de défauts que dans ce que ce diff a écrit ou dans ce qu'il aurait dû propager. Ne lis
**aucun** rapport de passe précédente (`kesh-gate-logs/v0175-validate-p*.md`).

## Ce que la remédiation P3 a changé, et ce qu'il faut y chercher

1. **Le libellé « Aucune QR-facture exploitable »** (AC-7, AC-10, AC-12, AC-13, captures, Q1, registre C-v017-5-7) :
   est-il vrai dans les quatre cas énumérés en Q1 et en AC-7 (vérifie chaque cas au code : `qr_decode.rs:98-113`,
   `inbox_import.rs:539-576`, `map_decode_error`, `+page.svelte:224-240`, `:293-315`) ? Q1 dit-elle juste ce que le
   rapport affiche dans chaque cas ? Les huit traductions sont-elles correctes et cohérentes avec `badge-qr` et
   `report-qr-unreadable` de chaque locale (`crates/kesh-i18n/locales/*/messages.ftl`) ? AC-12 garde-t-il à juste
   titre « aucune QR-facture lue » pour la valeur d'API `none` ? Reste-t-il, **hors des entrées datées du Change Log et
   des passages historiques du registre**, une occurrence de l'ancien libellé (« Aucun QR lu », « Aucune QR-facture
   lue » au sens du badge) ?
2. **Les délais** (AC-10 *Un import long*, AC-11, AC-13) : la phrase sur le proxy DSM est-elle vraie au manuel
   (`admin-manual.tex:638-661`) ; les deux messages d'erreur cités (`fr-CH/messages.ftl:2251`, `:2509`) sont-ils ceux
   que l'écran affiche dans chaque cas (`+page.svelte:90-110`, `api-client.ts`, abandon par `AbortController`) ; « la
   liste n'est pas rechargée » est-il vrai ; la borne de 500 pages (`admin-manual.tex:822`) est-elle bien citée ?
3. **AC-6-bis** : `IS_FREE_LOCK('kesh_inbox_import:<base>')` — la clé est-elle celle de `inbox_import.rs:180`, l'argument
   de ré-entrance de `GET_LOCK` est-il correct pour MariaDB 10.11, l'assertion est-elle exécutable telle qu'écrite (quelle
   base, quelle connexion) ? L'exception « fichier en cours, ligne validée, part en `DUPLICATE` » est-elle exacte au code
   (`inbox_import.rs:400-720`, ordre `commit` puis `remove_inbox_file`) ?
4. **Le point d'accroche** `decode_spc_from_pdf_bytes_observed` (AC-5) : cohérent avec `decode_pages` (AC-3) et le
   banc (AC-6) ? implémentable sans changer la signature publique ?
5. **Les fixtures lues à l'exécution** (AC-1, AC-5, AC-6) : `std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), …))`
   résout-il l'amorçage du générateur ; reste-t-il une mention d'`include_bytes!` qui la contredit ? Le palier (3)
   « binarisé avant JPEG » et le repli à 200 dpi de `scan-sans-qr.pdf` sont-ils cohérents avec le plafond de 1 Mio et
   avec T2/T3 ?
6. **Les commandes pdfium** (tâche 1) : URL, somme et chemin d'extraction égaux à `Dockerfile:30-38` ; le répertoire
   `lib/` produit est-il celui de `LD_LIBRARY_PATH` d'AC-6 ?
7. **Tests et mutations neufs** : T9 (fichier `crates/kesh-db/tests/imported_supplier_invoices_repository.rs`, données
   différentes, `supplier_invoice_id` résiduel — constructible ? clé étrangère et contrainte d'unicité
   `uq_imported_supplier_invoice`), M12e, M13b ; T10 au commit rouge ((b) et (g) rouges) ; chaque mutation rougit-elle
   le test annoncé ?
8. **L'inventaire étendu** (AC-7, troisième ligne du tableau) et la phrase ajoutée à `user-manual.tex:1844` : justes ?
9. **Le registre** : les marques « remplacé / précisé » des corps de C-v017-5-2, -5, -7 et le recompte de C-v017-5-10
   (dérogation de splitting) sont-ils exacts et suffisants ?
10. **Les décomptes** de l'entrée « Validation P3 » du Change Log (2 MEDIUM, 22 LOW) et des tableaux modifiés : recompte.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0175-validate-p4.md` :

1. **Les findings**, numérotés `P1`, `P2`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu
   (`fichier:ligne`), **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation
   de présence ou d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Les axes exercés** (1 à 10 ci-dessus) et **non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0
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
