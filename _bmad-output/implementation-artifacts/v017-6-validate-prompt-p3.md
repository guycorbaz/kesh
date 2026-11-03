# Prompt — validation P3 de la fiche v017-6 (release 0.17.0 : #832, #824), passe CIBLÉE, Opus

Tu valides la **fiche** de la Story v017-6 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). Passe
**ciblée** (`CLAUDE.md`, § *La passe ciblée*) : **une seule lentille**, braquée sur **le diff de la dernière
remédiation** — le commit « docs(v017-6) : remédiation de la validation P2 (refs #832, refs #824) » — et rien d'autre.
*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche les défauts **introduits** par cette remédiation, non ceux
de la conception d'origine. Langue du rapport : **français**. Modèle : **Opus**. **Contexte frais**.

Worktree `/home/gcorbaz/devel/kesh-0177`, branche `story/v017-6-petites-corrections`, partie de `8e126cfd`. Le diff à
relire : `git show HEAD` (le commit de tête, la remédiation P2) ; pour mémoire, `git show HEAD~1` est la remédiation P1
(hors périmètre de la lentille, sauf pour contrôler qu'une phrase P2 ne la contredit pas).

## Ce que la remédiation P2 a fait (à vérifier, pas à croire)

1. Borne « langue des documents » à **la facture, son PDF et son e-mail d'envoi** ; cite **#859** comme limite
   (l'avoir, le courrier de réinitialisation, les rapports) dans l'AC-5 (phrase prescrite), le guide (AC-6), le CHANGELOG
   (AC-9), la section *Issues*. Retire « courriers » et « PDF » non qualifiés.
2. « Installation » pour `KESH_LANG` (`admin-manual.tex:757` → « Langue de l'interface (locale de l'installation) ») ;
   « instance » retiré des phrases récrites ; § 11 amendé.
3. Intègre le message de `qr_iban.rs:42-43` (« doit être suisse ou liechtensteinois ») avec un test unitaire de
   `kesh-core` et une mutation M6 (AC-1, AC-4).
4. Ajoute `company.rs:164` et `i18n.rs:3` (doc-comments) à l'inventaire § 9 (désormais **dix** sites faux sur **onze**
   fichiers) et à l'AC-4.
5. Complète AC-7 (fragments, apostrophe typographique, référence `8e126cfd`), AC-8 (motif), AC-10 (plage
   `8e126cfd..HEAD`) ; annote C-v017-6-1, -3, -9, -10 ; Change Log « Validation P2 ».

## À lire

La fiche `_bmad-output/implementation-artifacts/v017-6-petites-corrections.md` en entier, le registre
`v017-6-choix-autonomes.md`, `git show HEAD`, l'issue **#859** (`gh issue view 859 --json title,body,comments`, lecture
seule), et le code et les textes que la remédiation affirme : `crates/kesh-core/src/types/qr_iban.rs` (en entier),
`crates/kesh-core/src/errors.rs`, `crates/kesh-api/src/routes/{credit_notes,auth,reports,invoice_email,issued_invoice_pdf,i18n}.rs`,
`crates/kesh-db/src/entities/company.rs`, `docs/manual/fr/{user,admin}-manual.tex`, `docs/user-guide/fr/getting-started.md`,
les PDF aplatis (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`, **vers la sortie standard seulement**, les deux graphies
d'apostrophe).

## Axes (tous sur le diff de la remédiation)

- **Vérité des bornes** : « la facture, son PDF et son e-mail d'envoi » est-il exact au code (`issued_invoice_pdf.rs`,
  `invoice_email.rs`) ? Le PDF d'un avoir, le courrier de réinitialisation et les rapports suivent-ils bien `KESH_LANG` /
  `accounting_language` aux lignes citées ? **#859** dit-elle la même chose que les phrases prescrites (titre, corps) ?
  Le rappel de facture ou un autre document (relevé, export) suit-il `instance_language` sans figurer dans la borne ?
- **Vocabulaire** : « instance » est-il vraiment absent de **toutes** les phrases récrites (AC-5, AC-6, AC-9, `.tex`,
  guide, README, catalogues) ? « installation » ne désigne-t-il que `KESH_LANG` ? La phrase de l'AC-5 reste-t-elle
  grammaticale et cohérente avec la `keshwarning` conservée et sa phrase retouchée ?
- **Le message de `qr_iban.rs`** : le nouveau texte « doit être suisse ou liechtensteinois » forme-t-il, avec le préfixe
  de `thiserror`, « QR-IBAN invalide : doit être suisse ou liechtensteinois » ? Un test, une spec, un manuel ou une
  clé i18n épingle-t-il l'ancien (« QR-IBAN doit être … ») ? `kesh-qrbill` (`types.rs:364-366`) porte-t-il la même phrase
  sans lien (à ne pas confondre) ? La mutation M6 rougit-elle bien le seul test annoncé, et AC-4 liste-t-elle exactement
  les fichiers touchés ?
- **L'inventaire** : « dix sites faux sur onze fichiers » : recompte depuis le tableau ; les deux doc-comments sont-ils
  faux au code, et leur retouche (AC-4) ne contredit-elle pas le helper QR-IBAN ou `Language` (`company.rs:70`) ?
- **AC-7, AC-8, AC-10** : chaque fragment annoncé « avant ≥ 1 » l'est-il dans les PDF actuels ? Le motif d'AC-8 retrouve-t-il
  la ligne 146 du manuel ? La plage d'AC-10 est-elle exécutable telle quelle ?
- **Les annotations** du registre : n'écrasent-elles aucune entrée passée ? Le Change Log est-il fidèle au diff ?
- **Régression** : la remédiation contredit-elle l'une des 12 remédiations de la P1 (helper QR-IBAN, libellé de
  *Paramètres*, contrôle du PDF aux deux bornes, `keshwarning`) ?
- **Décomptes** : tout nombre écrit par la remédiation (dix, onze, M1 à M6, nombre de fichiers, lignes) est recompté
  depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0176-validate-p3.md` :
1. **Les findings** `P1`, `P2`… avec sévérité, lieu, **la ligne lue citée** (`grep -nF` ou extrait copié) et remède.
   Une affirmation de présence ou d'absence sans sortie copiée ne compte pas.
2. **Les axes exercés** et **non exercés**, un par un, avec ce qui a été lu ou cherché. Un « 0 finding » sans cette liste
   ne compte pas comme passe.
3. Un décompte par sévérité, recompté.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` ailleurs, ni redirection (`>`, `>>`, `tee`),
  pas même vers un fichier « jetable » ni le scratchpad. Un fichier créé par erreur se **déclare** ; il ne se supprime pas.
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push`, `tag`, `rebase`, sous aucune forme.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, `npm`, `npx`, `make`, `latexmk`, `python3`, `node`, `sqlx`,
  `docker`, `curl`, `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`. Jamais `~/kesh-test/` ; aucune donnée réelle.
- Permis : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`, `git diff` (lecture),
  `pdftotext … -` vers la sortie standard, `gh issue view`.
