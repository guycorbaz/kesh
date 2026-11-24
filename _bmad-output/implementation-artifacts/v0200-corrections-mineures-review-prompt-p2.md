# Prompt — revue de code P2 de la story v0200-corrections-mineures (#1348, #1336, #1342, #1349, #1343, #1375), passe complète, Sonnet

Tu relis, en **seconde passe complète**, la story **v0200-corrections-mineures** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), release **0.20.0**, Epic 48. Kesh tient les livres réels du Project Lead, qui a **déjà émis de vraies factures
remises sur papier** : rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures, numéros, PDF) ne doit changer. **Aucune
migration.** **Modèle : Sonnet**, contexte frais, une seule lentille qui couvre les trois angles (code à l'aveugle, cas limites,
critères d'acceptation). Langue du rapport : français.

La passe est **complète** (et non ciblée) parce que la remédiation de la P1 touche trois modules : le frontend (page des contacts,
`ContactPersonsManager`), `kesh-i18n` (un catalogue) et les tests de `kesh-api`.

## Ce que la story fait (rappel)

- **#1348** — 21 clés appelées par `crates/kesh-api/src/errors.rs` et absentes de tout catalogue (le serveur rendait la clé brute)
  entrent aux quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`, texte fr-CH = repli du code) et à la `TABLE` de
  `les_replis_rust_suivent_le_catalogue` ; le refus du quota d'envoi passe de `t` à `t_args` ; test écrit en dur
  `cles_de_1348_traduites_dans_les_quatre_locales`. La garde générale est #1406 (hors périmètre).
- **#1336** — l'e-mail et le téléphone des contacts perdent leur `maxlength`.
- **#1342** — un pays seul, autre que « CH », est refusé au serveur (`address_input.rs`) et à l'écran (`countryAlone`).
- **#1349** — le motif d'un bouton d'enregistrement grisé s'affiche au-dessus des boutons (`contact-form-blocked-reason`).
- **#1343** — un prénom, un nom ou une raison sociale faits seulement de caractères invisibles sont refusés en nommant le champ
  (`helpers.rs::optional_visible`) ; l'écran des personnes de contact affiche le motif du serveur.
- **#1375** — doc-comment de `isSettlementOutcomeUnknown`, écrit d'après la **fiche** de la 48-12b (non fusionnée).

## Ce que la remédiation P1 a changé (commit `33047290d`)

La P1 (trois lentilles Opus, rapports `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/review-p1-{B,E,A}.md`) a rendu **0/0/0/13** après
réfutation de B1 (MEDIUM, faux positif vérifié par l'orchestrateur : la langue de l'interface est celle de l'installation,
`KESH_LANG`, `frontend/src/lib/shared/utils/i18n.svelte.ts:31-35`). Remédiés dans la story, sur décision de l'orchestrateur :

- **B3** : sur la page des contacts, le refus du serveur (`formError`) ne s'affiche plus que tant que la saisie égale celle qui a été
  soumise (`formErrorSignature`, `formSignature`, `formErrorShown`) ; test « B3 » et mutation M22 ;
- **B2** : `ContactPersonsManager.svelte` passe par `errorMessageOf` ;
- **A1** : `error-onboarding-reset-forbidden` en italien (« reimpostazione ») ;
- **A2** : le test `PUT` porte la raison sociale invisible et le geste du message ;
- **E1** : filet serveur `a_foreign_address_is_removed_by_clearing_the_country_1342` ;
- **E3, E4, A7** : manuel resserré (« caractères invisibles (espaces, traits d'union conditionnels…) », assistant nuancé, « sous le
  titre « Personnes de contact » ») ; **E2** : ligne aux Dev Notes ; **A3 à A6** : fiche et Dev Agent Record.

## Ce que tu lis

Worktree `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de `epic/48-corrections-0-20-0`
= `a051e17e5`.

1. **D'abord le commit de remédiation** : `git -C /home/gcorbaz/devel/kesh-v0200-cm show 33047290d`.
2. **Puis tout le diff, aplati** : `git -C /home/gcorbaz/devel/kesh-v0200-cm diff a051e17e5 33047290d` (code : `7eec3962e`,
   `cca96e590`, `d0efdb33e`, `33047290d`). Les commits suivants ne portent que la fiche, le `sprint-status.yaml` et les prompts :
   vérifie-le par `git -C /home/gcorbaz/devel/kesh-v0200-cm diff --stat 33047290d HEAD`.
3. La fiche `_bmad-output/implementation-artifacts/v0200-corrections-mineures.md` en entier (D1 à D7, AC1 à AC16, mutations, Note de
   version, Dev Agent Record, Change Log), et les trois rapports de la P1.
4. Les journaux, en lecture seule, sous `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/` : rouges (`rouge-*.log`), verts
   (`vert-*.log`, `gate-cible-serveur.log`), `mut/` (un journal par mutation, `M1.log` … `M22.log`, `e2e-M5.log`, `e2e-M9.log` ;
   scripts `run-mut-rust.sh`, `run-mut-front.sh`, `run-mut-e2e.sh`), `gate-ci.log`, `front-*.log`, `e2e-complet.log`, les journaux
   de la remédiation (`remed-p1-cible-rust.log`, `remed-p1-cible-front.log`, `gate-ci-remed-p1.log` et `.console`,
   `front-remed-p1-*.log`, `e2e-contacts-remed-p1.log`), `manuel-avant.txt`, `manuel-apres.txt`, `captures.log`, `captures-10.log`,
   et les captures `captures/01-…png` à `12-…png` (prises avant la remédiation P1 ; regarde-les si ton outil lit les images).

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part (ni
  scratchpad, ni `/tmp`) ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé
  par erreur se **déclare** au rapport ; il ne se supprime pas.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`/`merge`/`rebase`/`push`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `run-mut-rust.sh`, `run-mut-front.sh`,
  `run-mut-e2e.sh`, `backend-v0200cm.sh`, `captures-v0200cm.mjs` : ils se **lisent**) ; ni `make`, `latexmk`, `pdflatex`, `cargo`,
  `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires

1. **Le commit de remédiation, d'abord** (`33047290d`). B3 : `formErrorShown` peut-il cacher un refus qui s'applique encore, ou
   montrer un refus périmé ? `formErrorSignature` est-il figé au bon moment (avant l'envoi ; et si l'utilisateur modifie un champ
   pendant l'envoi) ? La signature couvre-t-elle **tous** les champs qui partent dans la charge (compare au corps construit par
   `submitForm`) — et un champ absent de la charge (`unknownCountryCode`, l'IDE affiché) ? Un refus `IDE_ALREADY_EXISTS` ou
   `CLIENT_NUMBER_ALREADY_EXISTS` : effacé au premier changement, est-ce juste ? Le conflit de version (`OPTIMISTIC_LOCK_CONFLICT`)
   et la réouverture du formulaire ? B2 : `errorMessageOf` change-t-il un comportement (journalisation, message blanc) que les tests
   figent ? A1 : la traduction suit-elle le glossaire et le catalogue italien ? A2, E1 : les tests neufs peuvent-ils rougir, ou sont-ils
   des filets déclarés ? Le manuel resserré (E3, E4, A7) dit-il exactement ce que fait le code, sans promesse de trop ?
2. **Les AC.** Chaque AC (AC1 à AC16) et chaque décision (D1 à D7, dont D4 annoté) contre le code et les tests. Un AC sans test qui
   peut rougir est-il déclaré filet ? Les écarts sont-ils déclarés au Dev Agent Record ?
3. **Le manuel** (`docs/manual/fr/user-manual.tex`, `.tex` seulement ; le PDF n'est pas régénéré dans la branche — forme du contrôle
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Personnes de contact'`, à juger comme
   non régénéré, pas comme défaut) : items « Nom », « Adresse structurée », « Email, Téléphone », paragraphe du bouton grisé, encart
   des personnes de contact, étape « Coordonnées légales » de l'assistant.
4. **Les mutations.** M1, M5 à M19, M21, **M22** : chacune tuée pour la bonne raison, par le test que la fiche nomme, journal avec le
   diff, le rouge et « == restauré » ; rejeux E2E de M5 et M9. Un test vert par construction ?
5. **Cohérence écran / serveur** : règle du pays seul (rognage, casse ASCII), nom invisible (écran `trim()` contre serveur
   `is_invisible`), affichage de chaque refus (page des contacts, assistant, interlocuteurs, panneau du fournisseur neuf).
6. **Contacts et sociétés existants** : qui peut être refusé à son prochain enregistrement, et est-ce écrit comme limite ? Aucune
   facture touchée :
   `git -C /home/gcorbaz/devel/kesh-v0200-cm diff a051e17e5...33047290d --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices`
   doit être vide.
7. **Les 21 clés et la clé d'écran aux quatre locales** : présence, ordre, texte fr-CH = repli, traductions selon
   `docs/i18n-glossaire.md`, `{ $max }` aux quatre ; `sitesTotal` 3503 ; `dette-parite-connue.txt` inchangé.
8. **La fiche** : Dev Agent Record et Change Log disent-ils vrai (journaux, commits, décomptes recomptés depuis le diff) ? La Note de
   version, issue par issue ?

## Rapport

Un seul rapport : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/review-p2.md` (le seul fichier que tu écris). Pour chaque finding :
identifiant (`P2-1`, `P2-2`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario, correctif proposé.
Verdict chiffré en tête (C / H / M / L). **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). La revue
n'exécute rien ; les gates de la remédiation sont cités au Change Log de la fiche avec leurs journaux, et l'E2E complet se rejoue à
la clôture.
