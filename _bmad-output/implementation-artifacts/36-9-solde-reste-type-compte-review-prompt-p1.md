# Prompt — revue de code P1 de la Story 36-9 (refs #1044), trois lentilles, première passe complète

Tu relis la **Story 36-9** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.18.0**, Epic 36,
phase 2 : **le solde du reste d'un règlement contrôle le type du compte de TVA due**. En escompte ou en perte sur une facture qui
porte de la TVA, `write_off_invoice` écrit la correction de TVA sur le compte de TVA due **des réglages** ; s'il a été retypé depuis
sa désignation (n'est plus un passif), le solde est désormais refusé `400 DESIGNATED_ACCOUNT_WRONG_TYPE`, le refus de la
validation d'une facture (Story 35-3), sans rien écrire. Le contrôle vit dans l'accesseur
`company_invoice_settings::vat_payable_account_for_write` (`crates/kesh-db/src/repositories/company_invoice_settings.rs`), sous
son verrou `FOR UPDATE` : absent ou archivé (`CONFIGURATION_REQUIRED`) → type → non imputable (`CONFIGURATION_REQUIRED`). La
phrase d'action du refus devient neutre (« l'opération ») dans les quatre langues et son repli Rust. **Aucune migration.** Kesh
tient des livres réels : un compte de TVA due de mauvais type fausse le décompte.

Worktree `/home/gcorbaz/devel/kesh-36-9`, branche `story/36-9-solde-reste-type-compte`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-9 diff ce95b8b5a 84246f671` (`ce95b8b5a` = l'étape 0, fiche et registre, sur la fiche
validée `4c5019c5d` ; `dec591150` = le code, les tests et les documents ; `84246f671` = le dernier commit de code, une spec E2E
seule — les gates backend et frontend ont tourné à `dec591150`) ; `git log --oneline 4c5019c5d..HEAD` liste les commits. La
branche est en retard de trois commits documentaires sur `origin/epic/36-0.18.0` (`CLAUDE.md`, `ci.yml`, fichier d'epic) : sans
effet sur ce diff. Langue du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-9-solde-reste-type-compte.md` en entier (*Ce qui se lit au code*, l'inventaire par le
symptôme, D1 à D5, AC1 à AC12, mutations, **Change Log** et **Dev Agent Record**) ; le registre `e36-choix-autonomes-36-9.md`
(C-36-9-1 à -6) ; la fiche de la 35-3 (`35-3-comptes-designes-type.md`) pour le patron. Ne prends pas le Dev Agent Record pour
argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-9/` :
`validation-p1.md`, `rouges.log`, `rouges-ac7-bis.log`, `cible-vert.log`, `clippy-dev.log`, `mut/M*.log` (scripts `mut/muter.py`,
`mut/jouer.sh`), `test-fast-ci-*.log`, `gate-ci-*.log` (deux runs), `front-*.log`, `e2e-spec-write-off*.log`, `backend-e2e-r1.log`. Déjà tracés, à ne pas rapporter de nouveau : **#525** (l'avoir
ne contrôle pas la TVA due), **#1047** (avertissement au retypage d'un compte désigné).

**Livrables visibles** : le dialogue *Solder le reste* de la fiche facture montre le refus du serveur (aucun changement d'écran :
`handleWriteOffConfirm`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte`) ; la capture
`/home/gcorbaz/devel/kesh-gate-logs/e36-9/captures/write-off-tva-due-retypee.png` (thème clair, base jetable ; regarde-la : le
message est-il lisible, entier, dans le dialogue resté ouvert ?). **Le manuel** :
`docs/manual/fr/user-manual.tex` (le paragraphe de l'indicateur *postable*, « Le solde du reste … depuis la 0.18.0 », et *Solder le
reste*) ; `docs/manual/fr/admin-manual.tex` (section neuve *Passer à la 0.18.0*, paragraphe « Pour les intégrations » ; *Après la
mise à jour : les comptes désignés* de la 0.17.7 ; *Configuration des comptes TVA* ; *Comptes du solde du reste*). ⚠️ **Les PDF ne
sont pas régénérés dans la branche** (`CLAUDE.md`, § *Travailler à plusieurs développeurs* : jamais commités dans une branche
`story/…`) : contrôle le `.tex` ; le PDF publié dit encore l'ancien texte, à la forme E6, sortie à l'écran :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'limite connue, #1044'` (attendu `1`
avant l'intégration, `0` après). `docs/api-external.md` (§ *Solder le reste d'une facture*, tableau des refus ; § 10, ligne
`DESIGNATED_ACCOUNT_WRONG_TYPE`). La **note de version** de la fiche — la branche n'écrit pas `CHANGELOG.md` (C-e360-11). Le
manuel dit-il **exactement** ce que le code fait, route par route, nature par nature ? C'est en vérifiant le manuel qu'on trouve
les chemins oubliés.

⚠️ **Ce qui n'a pas tourné** : l'**E2E complet (Playwright)** et le `gate-ci` de clôture — à la clôture (consignes de l'Epic
35). Seule la spec `frontend/tests/e2e/invoice-write-off.spec.ts` a été jouée, isolément, pour la capture (`e2e-spec-write-off-r3.log`,
2 passés au fichier de `84246f671`) ; son premier run (`e2e-spec-write-off.log`) a rougi sur une assertion du test, corrigée
(*Écarts* du Dev Agent Record). L'interaction du scénario neuf avec les autres specs (base partagée, compte de TVA due du seed
retypé puis rendu dans un `finally`) n'est pas éprouvée : juge-la à la lecture.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part.
- **Aucune redirection, sous aucune forme** : ni `>`, `>>`, `<`, `tee`, ni `2>`, `&>`, **ni `2>&1`**, **ni `2>/dev/null`**, ni
  `> /dev/null`. **Aucune substitution de processus** (`<(…)`, `>(…)`). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`reset`/`commit`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut/jouer.sh`, `mut/muter.py` : à **lire**, non à
  lancer) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
  `pdftotext` est permis, sortie à l'écran.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext … -`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`ls` ou `git cat-file -e <ref>:<chemin>`, E9).

## Les axes obligatoires, pour chaque lentille

1. **L'accesseur** `vat_payable_account_for_write` : ordre des refus (absent/étranger/archivé → type → non imputable) ; le type et
   l'imputabilité lus dans **la** requête `FOR UPDATE` (aucune lecture séparée) ; le type attendu tiré de
   `DesignatedRole::VatPayable.expected_type()` (aucune seconde source) ; le décodage `AccountType` de la colonne ; la liste
   `WrongTypeDesignatedAccounts::new` à une entrée (sa précondition « non vide » tenue).
2. **Son seul appelant** (`invoice_settlements_write.rs`, `write_off_invoice`) : l'ordre des verrous (nature, arrondi, TVA due,
   exercice) inchangé ; le refus de type avant `refuse_if_claim_account` ; ce qui ne corrige pas la TVA (`shares.is_empty()`) ne lit
   pas le compte ; rien n'est écrit sur un refus (`version`, audit, lettrage). Un **autre** chemin d'écriture vers le compte de TVA
   due des réglages, sans contrôle de type, que l'inventaire de la fiche aurait manqué (pars du symptôme, pas de la liste) ?
3. **La réponse HTTP** : bras générique `DbError::DesignatedAccountsWrongType` de `crates/kesh-api/src/errors.rs`, phrase
   d'action neutre ; la route du solde (`routes/invoices.rs`, `write_off_invoice_handler`) et son `retry_on_deadlock` ; la même
   requête, même `version`, rejouée après correction.
4. **i18n** : la clé `error-designated-account-wrong-type-action` dans les quatre `messages.ftl` (justesse des trois traductions,
   typographie de chaque langue) ; le repli Rust égal à fr-CH (G9, `crates/kesh-api/tests/textes_coherents.rs`) ; le simulacre
   `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts` aligné ; l'ancienne phrase greppée sur `crates`,
   `frontend`, `docs` (zéro occurrence attendue hors fiches et registre).
5. **Les tests neufs** : `crates/kesh-db/tests/invoice_write_off.rs`, module `tva_due_type` (AC1 à AC5) ;
   `crates/kesh-api/tests/designated_account_type_e2e.rs` (AC7, et le champ `confirmAccountRetype` du helper `retype`, corrigé :
   `confirmRetype` était ignoré par la route — un compte **sans** écriture ne demande pas de confirmation, ce qui l'avait masqué à la
   35-3) ; le scénario E2E (AC9). Rouges prouvés sur le code de départ pour la **bonne** raison (`rouges.log`,
   `rouges-ac7-bis.log`) ? Un test qui passerait à vide (AC3 : la liste des lignes vide ; AC5 : la tâche jamais vue en attente) ?
   Les motifs d'AC5 uniques (ni le verrou partagé de la validation, ni celui des natures) ?
6. **Mutations** M1 à M6 (`mut/*.log`) : chacune tuée par les tests nommés à la fiche ; chaque journal finit par « restauré » ; la
   restauration par `git restore --worktree` puis `touch` (E5). M4 reformulée à l'étape 0 (V1-4) : la mutation jouée dit-elle ce que
   la fiche annonce ? Propose des mutations plausibles non jouées.
7. **Manuel (.tex), `api-external.md`, note de version** (AC11, AC12) : chaque phrase dit-elle le code, nature par nature
   (escompte et perte, non frais ni arrondi ; facture **à TVA** seulement) ? Le changement de contrat écrit trois fois sans
   contradiction (API, manuel d'administration, note de version), y compris le cas `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` →
   refus de type (V1-3) ? La section *Passer à la 0.18.0* ouverte par la story (C-36-9-5) se compile-t-elle à la lecture
   (`\subsubsection`, `\label` unique : `git grep -n 'sec:maj-0-18-0'`, environnements `sloppypar` équilibrés) ?
8. **Le compte rendu** : chaque chiffre (inventaire, tests, gates) recompté depuis la source, avec son périmètre et son journal ? Le
   Dev Agent Record n'affirme-t-il que ce qui a tourné (E2E : **non lancé**) ?

## Lentille B — Blind (le diff seul, sans la fiche)

Relis l'accesseur, son appelant, les commentaires et doc-comments reformulés (`errors.rs` des deux crates,
`invoice_settlements_write.rs`, `company_invoice_settings.rs`), les catalogues, puis les tests. Cherche : un refus qui fuit
(`?` qui rend autre chose), un commentaire devenu faux (« sans contrainte de type », « seule garde », « la validation veut y
écrire »), un ordre de refus qui contredit son doc-comment, un test qui n'établit pas ce que son nom promet.

## Lentille E — Edge (chaque branche, chaque borne)

TVA due retypée en actif avec ou sans rôle, en charge, en produit ; retypée et archivée ; retypée et non imputable ; non imputable
au bon type ; désignée sur la créance (actif, ou retypée en passif) ; nature égale à la créance ; facture à deux taux ; facture sans
TVA ; frais bancaires ; reste d'arrondi (nature `rounding`, et fraction de centime d'une autre nature) ; retypage concurrent (AC5) ;
rejeu sur interblocage ; clé d'API `read-write`. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par
`grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC12 : livré, testé, au lieu prescrit ? Les LOW V1-1 à V1-5 de la validation traités à l'étape 0, ligne lue
citée ? Les écarts déclarés (C-36-9-5, C-36-9-6, le helper `retype` corrigé) exacts ? Les gates du passage en `review` : chaque
chiffre au journal, au dernier commit de code, horodaté après lui ? L'E2E déclaré « à la clôture » ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-9/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**.
