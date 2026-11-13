# Prompt — revue de code P1 de la Story 35-1 (#932, #948), trois lentilles, première passe complète

Tu relis la **Story 35-1** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.17.7**, Epic 35
(« corrections par impact ») : **la numérotation des factures et des avoirs**. (1) Un format de facture ou d'avoir sans compteur
(`{SEQ}`, `{SEQ:NN}`) est refusé aux réglages — `FormatError::NoSeq`, testé en dernier —, mais seulement s'il **change**
(`validate_template_change`, D2) ; le miroir d'écran suit (`code: 'NO_SEQ'`, `formatBlocks`) et un format chargé sans compteur
reçoit une aide non bloquante. (2) Le **format des avoirs** devient un champ de *Administration → Facturation* (D3). (3) Un numéro
impossible à produire — rendu > 64, accolade dans le nom de l'exercice sous `{FY}`, numéro déjà attribué (`uq_invoices_number`,
`uq_credit_notes_number`), format illisible — rend **`400 NUMBER_FORMAT_UNRENDERABLE`**, `details { document, cause }`, au lieu de
`500 INTERNAL_ERROR` ou `409 RESOURCE_CONFLICT` (D4, D5) : **changement de contrat** pour les clés `read-write`
(`POST /invoices/{id}/validate`, `POST /credit-notes`). (4) Limite voisine (C-e351-2) : un nom d'exercice à accolade ne bloque plus
le rendu sous un format qui ne reprend pas `{FY}`. **Aucune migration** ; un format enregistré n'est pas réécrit.

Worktree `/home/gcorbaz/devel/kesh-e351`, branche `story/35-1-numerotation-sans-seq`. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-e351 diff 569d7bd41 1cc78ffb1` (`569d7bd41` = la base : la 35-2b terminée avec la lignée
35-5a0 fusionnée, qui porte la 35-3 et la 35-2a ; `21deafde5` = le PDF du manuel régénéré après la fusion ; `1cc78ffb1` =
l'implémentation, et le dernier commit qui touche du code ; les commits suivants ne portent que des comptes
rendus). Langue du rapport : français. Modèle : **Opus** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/35-1-numerotation-sans-seq.md` en entier (*Ce qui se lit au code*, D1 à D7, AC1 à AC8,
mutations M1 à M8, **Change Log** et **Dev Agent Record**) ; les registres `e35-choix-autonomes-e350.md` (C-e350-8, -18, -42 à -45,
-47, -51) et `e35-choix-autonomes-e351.md` (C-e351-1 à -6). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec
les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e351/` : `rouges.log`, `rouges-api.log`,
`rouges-core.log`, `rouges-vitest.log`, `cible-1.log`, `verts-vitest.log`, `i18n-keys.log`, `test-fast-ci.log`, `gate-ci.log`,
`front-*.log`, `make-user.log`, `user-manual-plat.txt`, `mut/` (dont `campagne-rust.log` et `M5.log`, `M6.log`, `M7.log`),
`captures.log`.

⚠️ **L'E2E complet n'a pas encore tourné** (il se joue une fois, à la clôture de la story, après la revue ; consigne de l'Epic 35).
**`gate-ci` a tourné** au passage en `review` (voir le Dev Agent Record et `gate-ci.log`). Le dire n'est pas un finding ; affirmer le
contraire au Dev Agent Record en serait un. Les scripts `*.sh`, `*.py`, `*.mjs` sont à **lire**, jamais à exécuter.
**Captures** (thème clair) sous `/home/gcorbaz/devel/kesh-gate-logs/e351/captures/` (`01-numerotation-champ-avoirs.png`,
`02-refus-ecran-f-year.png`, `02b-refus-ecran-bouton-desactive.png`, `03-formats-charges-sans-compteur.png`,
`04-refus-validation-numero-deja-attribue.png`) — regarde-les si ton outil lit les images. **Manuel** : `docs/manual/fr/user-manual.tex`
(*Numérotation des factures*, et l'item de l'avoir dans *Avoirs et notes de crédit*) et son PDF `docs/manual/fr/user-manual.pdf`.

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

1. **La grammaire (D1, C-e351-4)** : `validate_template`, `validate_template_rules`, `validate_template_change`,
   `validate_numbering_template` (`crates/kesh-core/src/invoice_format.rs`). `NoSeq` vient-il bien **en dernier** (aucun format trop
   long, à caractère interdit ou à placeholder inconnu ne rend `NoSeq`) ? `validate_numbering_template` rend-elle exactement ce
   qu'elle rendait (compare au `git show 569d7bd41:crates/kesh-core/src/invoice_format.rs`, et ses tests inchangés) ?
   `validate_description_template` (qui rend aussi `NoPlaceholder`) est-elle intacte ? `render` : le refus d'accolade n'a-t-il lieu
   que sous `{FY}`, et `parse` passe-t-il désormais **avant** ce contrôle sans changer un autre résultat ?
2. **La route (D2, C-e351-1)** : `update_invoice_settings` — le format client et le format d'avoir sont-ils jugés **contre la valeur
   enregistrée** (`current`, lue hors verrou : quelle course, et qu'en dit le verrou optimiste) ? Un format d'avoir **omis** (#216)
   vaut-il la valeur enregistrée, donc toléré ? Le numéro interne fournisseur reste-t-il strict ? Les doc-comments rendus faux par D3
   (`company_invoice_settings.rs`, `idor_multi_tenant_e2e.rs`) sont-ils tous corrigés (`git grep -n 'formulaire frontend'`) ?
3. **Le refus typé (D4, D5)** : `DbError::NumberFormatUnrenderable`, `NumberedDocument`, `NumberFormatCause::from_render_error`
   (exhaustif sur `FormatError`, `InvalidSeq` → `Invariant`), `number_render_failed`, `into_number_duplicate` (ne convertit **que** la
   violation qui nomme la contrainte — un autre `UNIQUE` de `credit_notes`, s'il existe, garde son refus ?). Les deux sites de
   `invoices.rs` (rendu, `UPDATE … invoice_number = ?`) et les deux de `credit_notes.rs` (rendu, `INSERT`). La transaction est-elle
   bien annulée (aucune écriture, compteur intact) dans chaque branche ? Une facture **dévalidée** qui reprend son numéro peut-elle
   heurter `uq_invoices_number` (une autre facture aurait-elle pu recevoir ce numéro entre-temps) — et que rend-elle alors ? Le
   message de MariaDB 10.11 pour l'erreur 1062 nomme-t-il bien la clé (`… for key 'uq_invoices_number'`) ? `error_code` ; le bras
   d'`IntoResponse` (`kesh-api/src/errors.rs`) : statut, code, `details`, six clés, replis égaux au catalogue (garde **G9**,
   `textes_coherents.rs`, y compris les `{"{"}FY{"}"}` → `{{FY}}`) ; la phrase est-elle juste pour chaque cause (« aucun numéro n'a
   été consommé » est-il vrai pour `duplicate`, où le compteur a avancé dans la transaction annulée ?).
4. **L'écran (D2, D3)** : `invoice-number-format.ts` (`code`, `SEQ_RE`, `formatBlocks`), `settings/invoicing/+page.svelte` : les
   **quatre** lecteurs du blocage (aperçu, champ, `save()`, bouton) lisent-ils tous `formatBlocked` / `creditNoteFormatBlocked` ? La
   référence « chargée » (`settings?.…`) suit-elle un enregistrement réussi et la relecture après `OPTIMISTIC_LOCK_CONFLICT` ? Un
   format chargé **non** `NO_SEQ` mais invalide (posé par SQL) : bloqué, comme au serveur ? `invoices.types.ts` : le champ ajouté aux
   deux types ; un autre appelant de `updateInvoiceSettings` ? `invoices/[id]/+page.svelte` : le code dans la liste des refus non
   rejouables. Les clés i18n neuves et le texte changé de `settings-invoicing-format-help` dans les **quatre** catalogues, replis
   égaux au fr-CH ; `i18n-keys.test.ts` (`sitesTotal` 3026 → 3031, ventilation du commentaire juste ?) ; `lint-i18n-ownership`.
5. **Le manuel, l'API, le CHANGELOG** : `user-manual.tex` (*Numérotation des factures* : cinq items ; l'avoir, « champ *Format des
   avoirs* ») et **le PDF aplati** (sondes : « Format des avoirs », « Le compteur est obligatoire », « Un numéro impossible à produire
   est refusé » ; le code `NUMBER_FORMAT_UNRENDERABLE` n'est **pas** au manuel utilisateur, comme aucun code d'erreur — est-ce un
   écart à AC7 ?) ; chaque phrase est-elle vraie au code — « la page le refuse à l'enregistrement », « il n'empêche pas d'enregistrer
   les autres réglages », « aucun numéro n'est consommé » ? `docs/api-external.md` : la section neuve *Valider une facture — un numéro
   impossible à produire*, la ligne du tableau de l'avoir, la ligne du § 10 (placée après `DESIGNATED_ACCOUNT_WRONG_TYPE`, règle
   d'union C-e350-8). CHANGELOG `## [0.17.7] — Non publié`, *Corrigé* : deux points, `refs #932` et `refs #948`, les deux passages
   `500` / `409` → `400` dits pour un client d'API.
6. **Les témoins et les mutations** : AC1 (unitaires `kesh-core`), AC2 à AC4 (`number_format_unrenderable_e2e.rs`, dix tests), AC5
   (Vitest) : chacun peut-il rougir, et a-t-il rougi au départ (`rouges.log`) ? Les deux verts de départ d'AC2 et le cas « chargé sans
   compteur » d'AC5 sont-ils prouvés par M2/M2b et M7 ? M1 à M8 (M2b, M3b compris) tuées chacune par une assertion **nommée** au
   journal (`mut/`) ? M4 (avance du compteur dans une transaction à part) est-elle une mutation réelle ? Un attendu existant retouché
   sans motif (`git diff 569d7bd41 1cc78ffb1 -- crates/kesh-api/tests crates/kesh-core frontend/src '*.test.ts'`, lignes
   `-`) ? Propose des mutations plausibles non jouées.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne `invoice_format.rs`, la route, `kesh-db/src/errors.rs` (types et fonctions neufs), les quatre sites des dépôts, le
bras d'`errors.rs` côté API, `invoice-number-format.ts`, la page des réglages, les tests neufs. Cherche : une erreur avalée ou mal
classée ; une `UniqueConstraintViolation` d'une autre contrainte prise pour un doublon de numéro ; une phrase recopiée au lieu d'être
appelée ; un test qui passe **à vide** (assertion qui ne peut pas rougir, `toContain` d'un repli qui serait vrai même sans le code) ;
une injection HTML (le message du serveur et le format sont-ils rendus en texte ?).

## Lentille E — Edge (chaque branche, chaque borne)

Format `{SEQ}` seul ; `{SEQ:1}`, `{SEQ:10}` ; `{SEQ:0}` inchangé et enregistré (refus d'écran et de serveur ?) ; format d'avoir omis,
`null`, vide, identique, identique à des blancs près ; nom d'exercice de 50 caractères **multi-octets** (`render` compte en octets,
`fiscal_years` en caractères) ; accolade fermante seule ; nom d'exercice à accolade sous `{YEAR}` seul ; deux exercices de même
année de début sous `F-{YEAR}-{SEQ:04}` (défaut) ; revalidation d'une facture dévalidée ; deux validations concurrentes sous un format
sans compteur ; rejeu sur interblocage (`retry_on_deadlock`) d'une validation qui rend ce refus ; clé d'API `read-write` ; locale
de l'installation ≠ fr-CH (de-CH, it-CH, en-CH : guillemets, phrase). Chaque cas : comportement au code, test qui le couvre (ou
absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC8 : livré, testé, au lieu prescrit ? L'étape 0 (écarts entre les lignes lues sur `main` et la branche, en-tête
« Validation non lancée » périmé) citée au Change Log ? Les gates au dernier commit de code (`test-fast --ci`, `gate-ci`, frontend
complet ; E2E complet à la clôture) : chaque chiffre au journal, avec son commit ? Les écarts déclarés (consigne Edit : script Python
de mutation ; placement de la branche `NO_SEQ`, C-e351-4 ; l'aide non bloquante, C-e351-6) sont-ils exacts et sans effet sur le
livré ? Les choix C-e351-1 à -6 sont-ils prudents et réversibles ? Le Dev Agent Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e351-review-p1-B.md`, `…-p1-E.md`, `…-p1-A.md`. Chaque
rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu (`fichier:ligne`), ligne lue
copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte final **recompté depuis la
liste**.
