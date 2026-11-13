# Prompt — revue de code P1 de la Story 35-2a (#964, part de #955), trois lentilles, première passe complète

Tu relis la **Story 35-2a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **le refus du PDF d'une facture (`400 INVOICE_NOT_PDF_READY`) dit, dans un message localisé, quoi
corriger** — le champ, le caractère refusé (par `escape_debug` et son point de code), l'adresse incomplète (#964), le pays hors liste
ISO —, par une fonction pure (`refus_pdf`, ensemble clos `CauseRefusPdf`) et une construction unique (`refus`) ; le texte de
`kesh-qrbill` ne va plus qu'au journal ; l'écran affiche le message du serveur (`pdf-error.ts`). **39 clés neuves ×4 locales, deux
retirées, aucune migration, aucun `details` ajouté au contrat.** C'est l'écran des premières factures QR de l'utilisateur réel,
prévues avant le 2026-11-30 : un message faux ou vide y coûte cher.

Worktree `/home/gcorbaz/devel/kesh-e352a`, branche `story/35-2a-refus-pdf`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e352a diff 049b04668 fe88ce8cc` (`049b04668` = base `epic-35/base` = 0.17.6 + fiches ;
`fe88ce8cc` = dernier commit de code ; `dcac5283b` = étape 0, `b262e34de` = compte rendu seuls). Langue du rapport : français.
Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-2a-refus-pdf-contrat-details.md` en entier (*Ce qui se lit au code*, D1 à D11, AC1 à
AC14, tests et mutations, *Ce qui doit rester vrai*, **Change Log** et **Dev Agent Record**) ; les registres
`e35-choix-autonomes-e350.md` (C-e350-1, -8, -11) et `e35-choix-autonomes-e352a.md` (C-e352a-1 à -7). Ne prends pas le Dev Agent
Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e352a/` :
`rouges.log`, `rouges-front.log`, `cible-{1,2,3}.log`, `test-fast-ci.log`, `gate-ci.log`, `front-*.log`, `e2e-complet.log`,
`e2e-chaine.log`, `make-user.log`, `mut/` (dont `campagne.log`). Les scripts `*.sh` et `*.py` sont à **lire**, jamais à exécuter.
**Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e352a/captures/` (cinq `.png`) — regarde-les si ton outil lit les
images.

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

1. **La fonction pure et la construction unique (D3, D5)** : `CauseRefusPdf` est-il un ensemble clos, chaque variante a-t-elle sa
   clé, ses arguments et son repli ? Le `match` est-il exhaustif, sans bras attrape-tout qui masquerait un champ neuf de
   `kesh-qrbill` (et si un repli « nom inconnu » existe, que dit-il, et un test l'exerce-t-il) ? `map_qrbill_error` : chaque
   erreur de `kesh-qrbill` (relis son type d'erreur à la source, `crates/kesh-qrbill/src`) aboutit-elle à la bonne cause ? Le
   texte de `kesh-qrbill` va-t-il **seulement** au journal (`tracing`), jamais au message ? La QRR qui ne se construit pas devient
   une panne interne (500 ?) : est-ce juste, et le chemin `:338` (C-e352a-7, angle mort assumé) est-il vraiment inatteignable
   par l'utilisateur ?
2. **Le caractère (D4)** : `escape_debug` et le point de code `U+XXXX` — un caractère hors BMP (emoji, U+1F600), un caractère
   combinant, un caractère de contrôle (`\n`, `\t`, U+0000), un `{` ou un `$` dans le champ (injection Fluent ?), une isolation
   bidi U+2068/U+2069 : que montre le message ? `max`, `got`, `code` passent en chaînes : la longueur se compte-t-elle comme
   `kesh-qrbill` la compte (caractères, non octets) ?
3. **#964 et le pays (D6, D7)** : `champ_vide_pour_qr` à NPA et localité, le premier vide nommé ; la phrase est-elle la même
   que le refus vienne de Kesh ou de `kesh-qrbill` (M3/M4 dites équivalentes, V5-3 — vérifie-le au code) ? `pays_iso` sur
   chaque partie : un pays en minuscules stocké (test `a_lowercase_stored_country_is_not_known`), un pays vide, `None` ; le
   contrôle se place-t-il avant ou après un autre refus qui le masquerait (C-e352a-3) ? **Un refus qui n'existait pas en 0.17.6
   apparaît-il** (une facture qui se rendait hier et ne se rend plus aujourd'hui, par un contrôle plus strict que celui de
   `kesh-qrbill`) ? Si oui, c'est un finding au moins MEDIUM : nomme le cas.
4. **Les routes (AC3, AC11, AC14)** : téléchargement, envoi par e-mail, rappel seul, refigeage, avoir — chacune passe-t-elle par
   `refus` ? `classify_render_error` (`invoice_email.rs`) classe-t-il encore correctement ? La garde AC14 (appariement
   d'accolades, C-e352a-4) : un `#[cfg(test)]` sur une fonction seule, une accolade dans une chaîne ou un commentaire, un
   `mod tests;` externe la trompent-ils ? La garde AC13 (`init_error_i18n`, exactement deux) idem.
5. **i18n (AC8)** : 39 clés neuves ×4, deux retirées (plus aucune référence : `git grep -n 'company-address-empty\|client-address-empty'`),
   trois reprises aux replis réécrits égaux au fr-CH ; G9 à 9 entrées, 42 clés ; apostrophe typographique ; les quatre locales
   **traduites** (pas le fr-CH recopié) ; les mots « entreprise » / « client » et le nom du champ dans chaque phrase ;
   « Paramètres → Organisation » et la fiche du client (C-e352a-1) : ces chemins d'écran **existent-ils** sous ces noms dans
   l'interface (`git grep` des libellés du menu dans `messages.ftl` et le frontend) ?
6. **L'écran (D1, AC7)** : `pdfErrorMessage` affiche `err.message` s'il n'est pas vide ; message vide ou blanc → clé fixe ; les
   autres codes inchangés. Le message serveur est-il échappé à l'affichage (texte, pas `{@html}`) ? Les fiches facture **et**
   avoir, la modale d'envoi, le rappel : passent-elles toutes par `pdfErrorMessage` ?
7. **Le manuel contre le code** (`user-manual.tex:1251`), **le PDF aplati** (ligatures : grepe des mots sans `ff`/`fi`/`fl`) ; le
   CHANGELOG `## [0.17.7] — Non publié`, *Corrigé* (deux points : #964, part de #955, « le texte du message a changé ») ;
   `docs/api-external.md` (ligne `INVOICE_NOT_PDF_READY`). Chaque phrase est-elle vraie au code ?
8. **Les témoins et les mutations** : chaque AC a-t-il un test qui peut rougir (`rouges.log` : `invoice_pdf_e2e` 12/15 rouges, les
   3 verts expliqués) ? M1, M2a, M2b, M7, M9, M11, M12, M13, M14 tuées chacune par une assertion **nommée** au journal ? Aucun
   attendu existant retouché sans motif (`git diff 049b04668 fe88ce8cc -- crates/kesh-api/tests frontend/src '*.test.ts'`, lignes
   `-`) ? L'écart d'AC9 (troisième assertion « Pays invalide », `company_address_e2e.rs:505`) est-il juste ? Propose des
   mutations plausibles non jouées (une cause mal aiguillée dans `map_qrbill_error`, un champ de l'entreprise rendu « client »).

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `CauseRefusPdf`, `refus_pdf`, `refus`, `map_qrbill_error`, `adresse_complete`, `pays_iso`, les gardes de
`textes_coherents.rs`, `pdf-error.ts`, la spec E2E. Cherche : un bras de `match` qui avale une erreur ; un `unwrap`/`expect` sur
une donnée de l'utilisateur ; une phrase qui nomme la mauvaise partie ; une injection dans un argument Fluent ; un test qui passe
**à vide** ; une garde textuelle contournable.

## Lentille E — Edge (chaque branche, chaque borne)

Nom de 70 et de 71 caractères (et 71 caractères multi-octets) ; NPA vide, blanc, `\u{AD}`, `\u{200B}` ; localité seule vide ; client
sans adresse, sans contact ; entreprise sans compte principal, IBAN invalide, QR-IBAN avec référence QRR, QR-IBAN sans ; pays `SU`,
`ch` minuscule, vide ; numéro de facture à caractère refusé (D9) ; plusieurs défauts à la fois (lequel est nommé, et est-ce
stable ?) ; locale de l'utilisateur de-CH, it-CH, en-CH ; facture déjà figée (aucun rendu, aucun refus) ; avoir. Chaque cas :
comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC14 : livré, testé, au lieu prescrit ? L'étape 0 (inventaire de 16 lignes, 13 constructions de production)
recomptée et citée au Change Log ? Les gates au dernier commit de code `fe88ce8cc` : test-fast 5331, gate-ci 5338, Vitest 3188
(232 fichiers), E2E 404 passés / 7 KF-029 — chaque chiffre au journal, horodaté après le commit ? Les écarts déclarés (`cat >>`
contre la consigne Edit ; scripts Python des mutations et du dérivé E2E) sont-ils exacts et sans effet sur le livré ? Les choix
C-e352a-1 à -7 sont-ils prudents et réversibles ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e352a-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
