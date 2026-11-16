# Prompt — revue de code P1 de la Story 35-2b (#955), trois lentilles, première passe complète

Tu relis la **Story 35-2b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **le rapport d'un lot de rappels (`POST /dunning/reminders/send-batch`) dit pourquoi le PDF d'une
facture est refusé**. Au lot, `INVOICE_NOT_PDF_READY` porte désormais `details = { message }` pour les trois refus du PDF
(`InvoiceNotPdfReady`, `InvoiceTooManyLinesForPdf`, `InvoicePdfHeaderOverflow`) ; les deux phrases des routes unitaires sont
extraites d'`errors.rs` en deux fonctions (`message_trop_de_lignes_pdf`, `message_entete_pdf_deborde`), seule source pour la
réponse `400` et pour le lot (C-e350-46) ; l'écran ajoute la phrase au libellé, pour ce code seulement (`reminderFailureText`).
**Aucune migration, aucune clé i18n neuve.** Elle part de la 35-2a terminée (message localisé du refus, `refus_pdf`). Son
utilisateur réel enverra ses premières factures QR avant le 2026-11-30 ; les rappels viendront après.

Worktree `/home/gcorbaz/devel/kesh-e352b`, branche `story/35-2b-lot-rappels-refus-pdf`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e352b diff 506237558 24f69d217` (`506237558` = la 35-2a terminée, `9f8ef604`, avec
`epic-35/base3` fusionnée ; `24f69d217` = dernier commit de code ; `d765f9172` = l'implémentation ; les commits suivants ne
portent que des comptes rendus). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-2b-lot-rappels-refus-pdf.md` en entier (*Ce qui se lit au code*, D1 et D2, AC1 à AC7,
mutations M1 à M7, *Angle mort écrit*, **Change Log** et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e350.md`
(C-e350-8, -9, -11, -46) et `e35-choix-autonomes-e352b.md` (C-e352b-1 à -5) ; pour la frontière, la fiche de la 35-2a
(`35-2a-refus-pdf-contrat-details.md`, D3, AC14). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e352b/` : `rouges.log`, `rouges-front.log`,
`e2e-rouge.log`, `cible-1.log`, `test-fast-ci.log`, `gate-ci.log`, `front-*.log`, `make-user.log`,
`user-manual-plat.txt`, `mut/` (dont `campagne.log`).

⚠️ **L'E2E complet n'a pas encore tourné** (décision de Guy du 2026-11-13 : il se joue une fois, à la clôture de la story,
après la revue). Seule la spec neuve a tourné, **en rouge**, sur le code de départ (`e2e-rouge.log`) ; elle n'a pas encore
tourné en vert. Le dire n'est pas un finding ; affirmer le contraire au Dev Agent Record en serait un. Les scripts `*.sh` et `*.py` sont à **lire**, jamais à exécuter.
**Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e352b/captures/` (`35-2b-rapport-lot-refus-pdf.png`,
`35-2b-rapport-lot-refus-pdf-detail.png`) — regarde-les si ton outil lit les images. **Manuel** : `docs/manual/fr/user-manual.tex`
(`:1251`, `:1527`) et son PDF `docs/manual/fr/user-manual.pdf`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`, `jq` ; ni
  **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni
  aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`,
  `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `/usr/bin/grep`, `sed -n`, `awk` (sans redirection ni `system()`),
  `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '`.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport.

## Les axes obligatoires, pour chaque lentille

1. **Le serveur (D1)** : `classify_render_error` (`invoice_email.rs`) — les trois bras rendent-ils `INVOICE_NOT_PDF_READY` avec
   `details = { message }` et rien d'autre (`refus_pdf_au_lot`, C-e352b-1) ? Le bras final reste-t-il réservé aux pannes
   (`DATABASE_ERROR`) ? `classify_reminder_render_error` est-il inchangé ? Le lot reste-t-il un `200` avec `failed[]` (pattern batch
   du `CLAUDE.md`) — aucune erreur per-facture n'escalade ? `message` vient-il **toujours** de la phrase localisée (et jamais du
   texte de `kesh-qrbill`, du journal ou d'un `Display` d'erreur) ? La phrase est-elle dans la langue de l'**installation**, alors
   que le PDF est dans la langue du contact (`locale` de la boucle) — est-ce la même règle que la route unitaire, et est-ce dit ?
2. **Les deux fonctions extraites (C-e350-46)** : `message_trop_de_lignes_pdf`, `message_entete_pdf_deborde` — le texte, la clé,
   l'argument `count`, le repli sont-ils **identiques octet pour octet** à ceux des bras d'`IntoResponse` d'avant (compare au
   `git show 506237558:crates/kesh-api/src/errors.rs`) ? Les bras d'`IntoResponse` gardent-ils statut et code ? La garde **G9**
   (`textes_coherents.rs`, `error-invoice-pdf-header-overflow` : un site) et la garde **AC14** de la 35-2a (une seule occurrence de
   `InvoiceNotPdfReady(` hors tests dans `invoice_email.rs`) tiennent-elles, et pour la bonne raison ? Les marques d'isolation
   U+2068/U+2069 sont-elles retirées pour `count` (par `t_args`) ?
3. **L'écran (D2)** : `reminderFailureText` — lu **seulement** pour `INVOICE_NOT_PDF_READY`, seulement une chaîne non vide après
   `trim()` ; rendu en texte (`{…}`, jamais `{@html}`) ; séparateur « : » ; `message.trim()` à l'affichage (C-e352b-2). Les autres
   écrans qui affichent un `failed[]` de lot de rappels (`git grep -n 'reminderErrorLabel\|FailedReminder' -- frontend/src`)
   passent-ils tous par la nouvelle fonction, ou en reste-t-il un qui montre le libellé seul ? La garde des libellés en dur
   (`i18n-libelle-en-dur.test.ts`, 90 → 91, `conforme` 76 → 77) : le décompte est-il juste, la déclaration bien nommée ?
4. **Le manuel contre le code** (`user-manual.tex:1251`, `:1527`), **le PDF aplati** (les deux phrases neuves présentes, « (issue
   \#955) » absent à cet endroit ; ligatures : grepe des mots sans `ff`/`fi`/`fl`) ; le CHANGELOG `## [0.17.7] — Non publié`,
   *Corrigé*, le point #955 **complété** et non doublé (C-e350-8, point 1) ; `docs/api-external.md` (ligne
   `INVOICE_NOT_PDF_READY` : la clause du lot remplacée ; « aucun `details` » ramené aux réponses `400`). Chaque phrase est-elle
   vraie au code — en particulier « de même pour une facture qui a trop de lignes pour sa page ou dont l'en-tête déborde » ?
5. **Les témoins et les mutations** : AC1 (HTTP, deux causes, objet `details` entier, phrase lue au catalogue réel), AC2/AC3
   (unitaire, accesseur `details()`), le test HTTP ajouté de l'en-tête qui déborde (C-e352b-3), AC4 (Vitest, fonction et composant),
   AC5 (E2E `reminder-batch-refus-pdf.spec.ts`) : chacun peut-il rougir, et a-t-il rougi au départ (`rouges.log`, `e2e-rouge.log`) ?
   M1, M2, M3, M4, M5, M5bis, M6, M7 tuées chacune par une assertion **nommée** au journal (`mut/*.log`) ? Un attendu existant
   retouché sans motif (`git diff 506237558 24f69d217 -- crates/kesh-api/tests frontend/src '*.test.ts'`, lignes `-`) ? Propose
   des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `classify_render_error`, `refus_pdf_au_lot`, les deux fonctions d'`errors.rs` et leurs appelants,
`reminderFailureText`, `ReminderBatchReport.svelte`, les tests neufs (Rust, Vitest, E2E). Cherche : un bras qui avale une cause ;
une phrase recopiée au lieu d'être appelée ; un `unwrap` sur une donnée du serveur côté écran ; un test qui passe **à vide** (une
assertion qui ne peut pas rougir, un `find` qui ne trouve rien sans échouer) ; une injection HTML.

## Lentille E — Edge (chaque branche, chaque borne)

`details` absent, `null`, `{}`, `message` vide, blanc, nombre, objet, chaîne balisée, très longue (mise en page de la ligne) ; un
autre code avec `details.message` (`DATABASE_ERROR`, `RATE_LIMITED` qui porte `retryAfterSecs`, `REMINDER_CONTENT_TOO_LONG`) ; un
lot mêlant envoi réussi, PDF refusé et autre échec ; locale de l'installation ≠ locale de l'interface (de-CH, it-CH, en-CH : la
ligne mêle les langues, limite assumée D2.4) ; `InvoiceTooManyLinesForPdf` au lot sous catalogue réel (angle mort écrit : prouvé
par le repli seulement) ; une facture dont le PDF du **rappel** (`PdfDocument::Reminder`) est refusé alors que celui de la facture
passerait, et l'inverse. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC7 : livré, testé, au lieu prescrit ? L'étape 0 (écarts entre les lignes lues sur `release/0.17.6` et la
branche) citée au Change Log ? Les gates au dernier commit de code `24f69d217` (`test-fast --ci`, `gate-ci`, frontend complet ;
E2E complet à la clôture) : chaque chiffre au journal, avec son commit ? Les écarts déclarés (consigne Edit : scripts de mutation en Python ; le compteur de la garde des libellés) sont-ils exacts
et sans effet sur le livré ? Les choix C-e352b-1 à -5 sont-ils prudents et réversibles ? Le Dev Agent Record n'affirme-t-il que ce
qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e352b-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
