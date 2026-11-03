# Prompt — revue de code P1 de la Story v017-6 (#832, #824), trois lentilles, Sonnet

Tu relis le développement de la **Story v017-6** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11),
*deux petites corrections* :

- **#832** — un refus d'IBAN, de QR-IBAN ou d'IDE ne répète plus son mot. Les trois sites IDE et IBAN
  (`crates/kesh-api/src/routes/onboarding.rs`, IDE et IBAN de l'assistant ; `routes/bank_accounts.rs`, IBAN de la
  création d'un compte) servent `e.to_string()` ; les deux sites QR-IBAN passent par un helper neuf,
  `crate::util::refus_qr_iban` (`crates/kesh-api/src/util.rs`) ; le message de `kesh-core`
  (`crates/kesh-core/src/types/qr_iban.rs`) perd son second « QR-IBAN » ; le doc-comment de `CoreError`
  (`crates/kesh-core/src/errors.rs`) est amendé. Tests : `onboarding_path_b_e2e.rs` (`coordinates_validates_ide`,
  `bank_account_validates_iban`), `bank_accounts_e2e.rs` (`ordre_des_refus_de_forme`), un test unitaire de
  `qr_iban.rs` (`foreign_iban_message_names_the_field_once`).
- **#824** — deux langues, un nom chacune : l'écran *Paramètres* nomme « Langue des documents » le champ
  `instance_language` (`settings-field-instance-language`, quatre `crates/kesh-i18n/locales/*/messages.ftl`, repli de
  `frontend/src/routes/(app)/settings/+page.svelte`) ; le manuel utilisateur (§ *Choix de la langue*, sa
  `keshwarning`, la liste des limites), le manuel d'administration (tableau des variables, `KESH_LANG` ; § *Configuration
  de la company via l'onboarding*), le guide `docs/user-guide/fr/getting-started.md`, le `README.md`, le `CHANGELOG.md`
  et sept doc-comments (`kesh-db/src/entities/company.rs`, `kesh-api/src/routes/i18n.rs` ×2,
  `routes/backup_schedule.rs`, `mail/smtp.rs`, `mail/mod.rs`, `routes/invoice_pdf_service.rs`).

**Aucune migration, aucune route, aucune table.** Kesh est en production depuis le 2026-10-31 : rien ne doit toucher une
donnée existante.

Worktree `/home/gcorbaz/devel/kesh-0177`, branche `story/v017-6-petites-corrections`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0177 diff 0ef623ae...HEAD` (`0ef623ae` = fin de la validation de la fiche ; la branche
part de `8e126cfd`, tête de `release/0.17.0`). Langue du rapport : français. Modèle : **Sonnet**. Passe **complète**
(première passe d'une revue de code). Contexte frais : ne t'appuie sur aucune passe antérieure.

⚠️ **Les trois propriétés centrales.**
1. **Un refus dit son mot une fois et désigne le bon champ** : « Numéro IDE invalide : … », « IBAN invalide : … »,
   « QR-IBAN invalide : … » — y compris pour un QR-IBAN mal formé, dont `QrIban::new` propage l'erreur d'IBAN.
2. **Les deux langues ont chacune un nom** : « langue de l'interface » = `KESH_LANG`, « de l'installation », réglée par
   l'administrateur ; « langue des documents » = `instance_language`, réglée au premier écran de l'assistant. Aucun texte
   livré ne dit plus que l'assistant règle la langue de l'interface ; « instance » ne désigne jamais `KESH_LANG`.
3. **La borne des documents est exacte** : la facture, son PDF et son e-mail d'envoi, ses rappels (e-mail et PDF), les
   libellés des conditions de paiement — quand le contact n'a pas sa langue ; le PDF d'un avoir et le courrier de
   réinitialisation suivent `KESH_LANG` (#859, limite citée).

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/v017-6-petites-corrections.md` — *Ce que le code dit* (§ 1 à 11, dont
l'inventaire clos § 3 et § 9), les **AC-1 à AC-11**, *Tâches*, *Dérogation*, le **Dev Agent Record** et le **Change
Log** ; puis le registre `v017-6-choix-autonomes.md`, **C-v017-6-1 à 10** (fiche) et **C-v017-6-11 et suivants**
(développement). Le Dev Agent Record déclare des rouges, des gates, des mutations, un E2E et des captures : **ne les
prends pas pour argent comptant**, recoupe-les avec les journaux hors dépôt (lecture seule), sous
`/home/gcorbaz/devel/kesh-gate-logs/v0176/` : `rouges.log`, `verts.log`, `mut/M*.log` (le script `mut/run.py` est à
**lire**, jamais à exécuter), `clippy*.log`, `test-fast.log`, `check.log`, `lint-i18n.log`, `vitest.log`, `build.log`,
`e2e-complet.log`, `make-user.log`, `make-admin.log`, `avant-user.txt`, `avant-admin.txt`, `apres-user.txt`,
`apres-admin.txt`, `captures-fr.log`, `captures-de.log`, et `test-fast-run1-oom.log` (premier gate tué par `systemd-oomd`,
déclaré au Dev Agent Record) ; le script E2E et captures `run-e2e.sh` est à **lire** seulement. Les captures (thème clair)
sont sous `/home/gcorbaz/devel/kesh-gate-logs/v0176/captures/` (regarde-les si ton outil lit les images), leur spec sous
`/home/gcorbaz/devel/kesh-gate-logs/v0176/capture-spec/` (à lire seulement). La validation P4 de la fiche, dont les
LOW sont appliqués à l'étape 0 : `/home/gcorbaz/devel/kesh-gate-logs/v0176-validate-p4.md`.

⛔ Ne lis rien sous `~/kesh-test/`.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, ni
  dans `kesh-gate-logs`, ni dans un scratchpad, ni sous `/tmp`, ni ailleurs ; **aucune redirection** (`>`, `>>`,
  `tee`, `2>`, `&>`, `2>&1`) : compare par des commandes enchaînées (pipe). Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- **Aucun `rm`, sous aucune forme** (ni `rm -f`, ni `rm -rf`, ni `unlink`, ni `find -delete`) ; ni **`pkill`**, ni
  **`killall`**.
- **Aucun `git checkout` ni `git switch`, sous aucune forme** (ni `git checkout -- <fichier>`, ni `git restore`), aucun
  `git` qui écrit (`commit`, `stash`, `reset`, `tag`, `push`, `rebase`, `worktree`, `add`, `fetch`, `merge`).
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python`/**`python3`**, ni `perl`, ni `ruby`,
  ni `node`/**`npm`**/`npx`, ni `bash -c`/`sh -c` d'un script, ni `jq` ; ni **`scripts/prepare-release.sh`**, ni
  `scripts/test-fast.sh`, ni `scripts/mem-guard.sh`, ni **aucun script du dépôt** ni de `kesh-gate-logs` (dont
  `mut/run.py`) ; ni **`make`**, ni `latexmk`, ni **`cargo`**, ni **`docker`**, ni `mysql`/`mariadb`, ni `sqlx`, ni
  **`curl`**, ni **`claude`** sous aucune forme, ni **`sed -i`**, ni `playwright`, ni `vitest`.
- **Aucune commande `gh`**, pas même en lecture. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `grep`, `sed -n`, `awk` (sans redirection ni
  `system()`), `cat`, `ls`, `find` (sans `-exec`, `-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`, et `git show <réf>:<f>.pdf | pdftotext - - | tr '\n' ' ' | tr -s ' '`.
- ⛔ **Sur le PDF aplati, jamais de `grep -oE` ni de `grep -E` à quantificateur `{0,N}` ou `{N,M}`** : `grep -oF … |
  wc -l` ou `grep -cF` sur des **motifs fixes**, sous `timeout 60`. Exemple :
  `timeout 60 pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oF 'langue de l’installation, que fixe' | wc -l`.
  ⚠️ Le PDF rend l'apostrophe en `’` (U+2019) : `grep -oF "l’interface"`, non `l'interface` — compte les deux graphies.
- **Toute affirmation d'absence ou de présence** s'appuie sur une sortie **`grep -nF`** (ou `sed -n`) copiée dans le
  rapport.
- **Données réelles** : aucune ; les fixtures sont synthétiques (IBAN d'exemple SIX, « Exemple Sàrl », « Banque
  Exemple »).

## Les axes obligatoires, pour chaque lentille

1. **Les manuels (`.tex` et PDF aplati) contre le code.** Le § *Choix de la langue* récrit (`user-manual.tex`), sa
   `keshnote`, sa `keshwarning` (« aux installations réglées dans une autre langue »), la puce des limites ; le tableau
   des variables (`KESH_LANG`) et le § *Configuration de la company via l'onboarding* (`admin-manual.tex`). Chaque
   affirmation est-elle vraie au code : l'interface servie en `state.config.locale` (`routes/i18n.rs`, `config.rs`,
   `KESH_LANG`, `fr` par défaut) ; le premier écran de l'assistant (`set_language`, `routes/onboarding.rs`) ne règle que
   `instance_language` ; la borne des documents (`resolve_language`, `routes/invoice_email.rs` et ses appels ;
   `routes/invoices.rs`, `routes/contacts.rs`, `routes/issued_invoice_pdf.rs`) ; l'avoir (`routes/credit_notes.rs`) et
   la réinitialisation (`routes/auth.rs`) hors borne ? « avant le choix du mode » est-il vrai à l'écran
   (`frontend/src/routes/onboarding/+page.svelte`) ? Le PDF dit-il la même chose que le `.tex` ? Les comptes « avant /
   après » du Dev Agent Record se reproduisent-ils (`git show 8e126cfd:docs/manual/fr/user-manual.pdf | pdftotext - - |
   tr '\n' ' ' | tr -s ' ' | grep -oF …` pour l'avant, jamais `HEAD`) ? Le PDF porte-t-il la date et la version
   d'avant (`\keshVersion` **non** bumpée) ? Un autre lieu des manuels, du guide, du `README.md` ou de `website/` dit-il
   encore que l'assistant règle la langue de l'interface, ou emploie-t-il « instance » pour `KESH_LANG`
   (`grep -rniE "langue.{0,20}interface|interface.{0,40}(onboarding|assistant)|(onboarding|assistant).{0,60}interface" docs README.md website`
   sur les **sources** seulement, jamais sur un PDF) ?
2. **Les témoins et les mutations.** Pour chaque site (IDE, IBAN et QR-IBAN de l'assistant ; IBAN et QR-IBAN du compte
   bancaire ; message de `qr_iban.rs`) : quel test le couvre, à l'égalité exacte ou au décompte, et peut-il passer **à
   vide** (une boucle `for` sur des cas dont l'un serait refusé par une autre règle avant d'atteindre le site ; une
   valeur attendue recopiée du code corrigé plutôt que du serveur d'avant) ? Les mutations M1 à M6 (et M3-bis, M3-bis
   du compte bancaire) sont-elles tuées **par le test attendu** (journaux `mut/M*.log`) ? Une mutation plausible non
   jouée survivrait-elle (le helper rendant `format!("QR-IBAN invalide : {e}")` pour `InvalidIban` ; la branche `autre`
   préfixée ; un `InvalidQrIban` d'un QR-IID non numérique) ?
3. **Les gardes et compteurs.** Les gardes i18n du frontend (`i18n-repli-divergent-actif`, `i18n-un-repli-par-cle`,
   `i18n-keys`, `i18n-libelle-en-dur`) et la parité des catalogues : une clé naît-elle, un compteur bouge-t-il, un repli
   diverge-t-il de la valeur fr-CH ? Le Dev Agent Record dit-il vrai sur les compteurs (aucun recalé, ou lesquels) ?
4. **Grep du symptôme, la valeur, sur tout le worktree** :
   `git grep -nE 'format!\("[^"]*(IDE|IBAN) invalide :? ?\{'` ;
   `git grep -nF 'QR-IBAN doit être'` ; `git grep -nF 'Langue interface'` ; `git grep -nF 'Oberflächensprache'` ;
   `git grep -nF "Lingua dell'interfaccia"` ; `git grep -nF 'Interface language'` ; `git grep -nF "langue de l'instance" -- crates` ;
   `git grep -nF 'locale instance'` ; `git grep -nF "fixée à l'onboarding"` ; `git grep -nF "choisie lors de l'onboarding"` ;
   `git grep -nF "choisie à l'onboarding"` — chaque occurrence restante est-elle justifiée (historique de spécification,
   doc-comment conforme, entrée datée) ?

## Lentille B — Blind (le diff seul, sans la fiche)

1. **Le code de production** : `refus_qr_iban` (signature, `match`, le `Display` de chaque variante, la doc), les cinq
   sites (`map_err`), le message de `qr_iban.rs`, les doc-comments touchés : chacun dit-il vrai au code qu'il commente ?
   Un import mort, un `use` manquant, un chemin `crate::util::` incohérent avec le reste du fichier ?
2. **Les tests** : les assertions neuves (valeurs littérales, messages d'échec, la condition `debut.ends_with("IBAN
   invalide")` de `ordre_des_refus_de_forme`), le test unitaire neuf ; la QR-IID attendue (« 762 ») est-elle bien celle
   que calcule `QrIban::new` sur l'IBAN d'exemple ?
3. **Les textes** : CHANGELOG (section `## [0.17.0] — Non publié`, *Corrigé*, liens), LaTeX (`\#859`, `\texttt{KESH\_LANG}`,
   tirets), le guide (typographie du fichier), le README ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`,
   « validation close ») dans les messages de commit (`git log 0ef623ae..HEAD --format=%B`).

## Lentille E — Edge cases

1. **Les refus** : IDE vide ou blanc (non validé), IDE à checksum faux, IBAN à blancs, minuscules, non-ASCII ; QR-IBAN
   d'un pays autre que CH/LI, QR-IID non numérique, QR-IBAN égal à l'IBAN ; l'ordre des refus (banque, description, IBAN,
   QR-IBAN) au compte bancaire et à l'assistant ; un `CoreError` d'une autre variante qui atteindrait le helper.
2. **Les consommateurs du texte** : le frontend lit-il le message pour décider d'un comportement (un `startsWith`, un
   `includes("IBAN invalide")`, une traduction par motif) ? `grep -rnF "IBAN invalide" frontend/src frontend/tests` ;
   l'intégration externe (`docs/api-external.md`) documente-t-elle le texte ?
3. **Les langues** : le libellé de *Paramètres* en de-CH, it-CH, en-CH (longueur, sens, apostrophes) ; la capture de-CH
   (`KESH_LANG=de`) montre-t-elle « Dokumentensprache » ? Une installation en `KESH_LANG=de` lit-elle encore, quelque
   part, « Langue de l'interface » pour `instance_language` ?

## Lentille A — Acceptance (chaque AC contre le dépôt)

Pour chaque AC (**AC-1 à AC-11**), rends « tenu », « non tenu » ou « non vérifiable en lecture seule », avec la preuve.
En particulier : AC-1 (trois sites `e.to_string()`, helper unique, message de `qr_iban.rs`, doc-comments de
`CoreError` et de l'assistant) ; AC-2 (les quatre assertions et le test unitaire, **rouges d'abord** : `rouges.log`) ;
AC-3 (M1 à M6, M3-bis : chacune tuée par le test attendu) ; AC-4 (la liste des fichiers de code serveur du diff, ni
plus ni moins : `git diff --stat 0ef623ae...HEAD -- crates`) ; AC-5 (le texte prescrit et ses trois points, la
`keshwarning`) ; AC-6 (les sites de l'inventaire, quinze depuis l'étape 0, et le libellé aux quatre locales) ; AC-7
(PDF aux deux bornes, deux graphies, `\keshVersion` non bumpée) ; AC-8 (les relevés rejoués et triés) ; AC-9
(CHANGELOG) ; AC-10 (mots-clés) ; AC-11 (gates, E2E, captures). L'étape 0 : les quatre LOW de la validation P4 sont-ils
appliqués, ligne lue citée au Dev Agent Record ? Les entrées passées du Change Log et du registre sont-elles restées
des relevés datés (annotées, non réécrites : `git diff 0ef623ae...HEAD -- _bmad-output | grep '^-'`) ?

Et : les choix C-v017-6-11 et suivants sont-ils justifiés, l'un contredit-il un AC, la fiche ou le registre ? Chaque
nombre du Dev Agent Record (tests, mutations, gates, E2E, sites) se recompte-t-il depuis sa source, avec son
périmètre ? Les **défauts observés** que le Dev Agent Record demande en issue sont-ils réels (prémisse lue au code) ?

## Rapport

Écris ton rapport dans **un seul** fichier : `/home/gcorbaz/devel/kesh-gate-logs/v0176-review-p1-B.md` (lentille B),
`/home/gcorbaz/devel/kesh-gate-logs/v0176-review-p1-E.md` (lentille E) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0176-review-p1-A.md` (lentille A), selon la lentille qui t'est confiée — par
l'outil d'écriture de fichier, **jamais** par une redirection. Contenu :

1. **Les findings**, chacun : identifiant (`B1`, `E1`, `A1`…), sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), fichier
   et ligne, la preuve (sortie de commande copiée, `grep -nF` pour toute présence ou absence), la correction proposée,
   et s'il est **né du développement** de la v017-6 ou **antérieur** (un défaut antérieur se **signale**, il ne se
   corrige pas ici).
2. **Les axes réellement exercés, et ceux qui ne l'ont pas été** — sans cette liste, un « 0 finding » ne compte pas.
3. **Le décompte** `CRITICAL / HIGH / MEDIUM / LOW`, recompté depuis ta liste.
4. Les fichiers créés par erreur, et toute commande lancée hors de la liste permise, s'il y en a.
