# Prompt — validation P4 de la fiche v017-6 (release 0.17.0 : #832, #824), passe CIBLÉE, Sonnet

Tu valides la **fiche** de la Story v017-6 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11). Passe
**ciblée** (`CLAUDE.md`, § *La passe ciblée*) : **une seule lentille**, braquée sur **le diff de la dernière
remédiation** — le commit « docs(v017-6) : remédiation de la validation P3 (refs #832, refs #824) » — et rien d'autre.
*La sévérité se déplace vers ce qu'on vient d'écrire* : cherche les défauts **introduits** par cette remédiation, non ceux
de la conception d'origine. Langue du rapport : **français**. Modèle : **Sonnet**. **Contexte frais** : ne lis aucun rapport de passe antérieure.

Worktree `/home/gcorbaz/devel/kesh-0177`, branche `story/v017-6-petites-corrections`, partie de `8e126cfd`. Le diff à
relire : `git show HEAD` (le commit de tête, la remédiation P3) ; pour mémoire, `git show HEAD~1` est la remédiation P2
(hors périmètre de la lentille, sauf pour contrôler qu'une phrase P3 ne la contredit pas).

## Ce que la remédiation P3 a fait (à vérifier, pas à croire)

C'est la passe 4 : la remédiation est **minimale**, et ses deux MEDIUM venaient de la remédiation précédente.

1. **Borne des documents élargie** : « la facture, son PDF et son e-mail d'envoi, ses rappels (e-mail et PDF) et les
   libellés des conditions de paiement », en § 7, § 11, AC-4 (doc-comment de `company.rs:164`), AC-5 (texte prescrit),
   AC-6 (guide), AC-9 (CHANGELOG), registre. Elle s'appuie sur `resolve_language` (`invoice_email.rs:71`) appelée aussi
   aux lignes 410, 461, 1147, `invoices.rs:697`, `contacts.rs:519`. #859 a été corrigée par l'orchestrateur.
2. **« Instance »** : interdit seulement pour désigner `KESH_LANG` ; l'incise « langue de l'instance » = langue des
   documents rétablie (AC-5, § 11) ; C-v017-6-10 annoté.
3. **Inventaire** : trois doc-comments de plus (`backup_schedule.rs:125`, `mail/smtp.rs:102`, `mail/mod.rs:74`), AC-4
   étendu ; décompte « douze sites faux sur quatorze fichiers » (AC-6, § 9) ; AC-8 : motif uni à `fixée` (lignes 146 et
   151).
4. **AC-3 / AC-2** : M6 et le test unitaire de `kesh-core` ; **dérogation de splitting** récrite ; deux lignes de tableau
   réindentées ; Change Log « Validation P3 ».

## À lire

La fiche `_bmad-output/implementation-artifacts/v017-6-petites-corrections.md` en entier, le registre
`v017-6-choix-autonomes.md`, `git show HEAD`, l'issue **#859** (`gh issue view 859 --json title,body,comments`, lecture
seule), et le code et les textes que la remédiation affirme : `crates/kesh-core/src/types/qr_iban.rs` (en entier),
`crates/kesh-core/src/errors.rs`, `crates/kesh-api/src/routes/{credit_notes,auth,reports,invoice_email,issued_invoice_pdf,i18n}.rs`,
`crates/kesh-db/src/entities/company.rs`, `docs/manual/fr/{user,admin}-manual.tex`, `docs/user-guide/fr/getting-started.md`,
les PDF aplatis (`pdftotext <pdf> - | tr '\n' ' ' | tr -s ' '`, **vers la sortie standard seulement**, les deux graphies
d'apostrophe).

## Axes (tous sur le diff de la remédiation)

- **Vérité de la borne élargie** : chaque appelant de `resolve_language` cité l'est-il bien à la ligne dite, et la
  fonction rend-elle `instance_language` quand le contact n'a pas de langue ? La liste « facture, PDF, e-mail d'envoi,
  rappels (e-mail et PDF), conditions de paiement » est-elle **complète** : cherche tout autre appelant de
  `resolve_language` et tout autre consommateur de `instance_language` ou de `company.instance_language` (relevé de
  `grep -rn "resolve_language\|instance_language" crates --include=*.rs`, tests exclus) et dis si un document manque ou si
  un item listé est faux. La phrase est-elle **la même** dans § 7, § 11, AC-4, AC-5, AC-6, AC-9, registre (grep de la
  valeur « rappels » et « e-mail d'envoi »), sans reste de l'ancienne borne ni de « rien d'autre » / « rien de plus » ?
- **Les cas d'« instance »** : le § 11 et l'AC-5 disent-ils la même règle ? L'incise rétablie est-elle exacte
  (`user-manual.tex:1140, 1181-1182, 1238`, clé `contact-form-language-inherited`) ? Aucune phrase prescrite n'emploie
  « instance » pour `KESH_LANG` ? L'annotation de C-v017-6-10 écrase-t-elle une entrée passée (elle ne le doit pas) ?
- **L'inventaire des doc-comments** : les quatre sites de `grep -rn "langue de l'instance" crates --include=*.rs` sont-ils
  tous des emplois pour `KESH_LANG` (lis chaque commentaire) ? D'autres emplois, ailleurs dans `crates/` ou dans
  `frontend/src`, restent-ils non classés ? « Douze sites sur quatorze fichiers » : recompte depuis le tableau § 9.
- **AC-3, AC-2, AC-4** : M6 est-elle définie une fois, rougit-elle le seul test annoncé ? AC-4 liste-t-il exactement les
  fichiers que la fiche demande de toucher (aucun de plus, aucun de moins) ?
- **AC-8** : le motif uni retrouve-t-il bien 146 et 151 de `user-manual.tex` (grep ligne à ligne) sans bruit
  ingérable ?
- **Dérogation de splitting** : le compte des ensembles (cinq de code, la documentation, six) est-il exact au contenu des
  AC ? La justification tient-elle ?
- **Régression** : la remédiation contredit-elle une remédiation antérieure (helper QR-IBAN, libellé de *Paramètres*,
  contrôle du PDF, `keshwarning`, vocabulaire « installation ») ou une phrase non touchée de la fiche ?
- **Décomptes** : tout nombre écrit par la remédiation (douze, quatorze, M1 à M6, lignes) est recompté depuis la source.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, à `/home/gcorbaz/devel/kesh-gate-logs/v0176-validate-p4.md` :
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
