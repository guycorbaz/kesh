# Prompt — revue de code P1 de la Story 36-18a (#1050, #1159, #1172, #1174), trois lentilles, première passe complète

Tu relis la **Story 36-18a** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB 10.11), release **0.18.0**,
Epic 36, phase 2 : **les montants « 1.250 » qui écrivent dans les livres**. Avant la story, quatre écrans lisaient un montant
autrement que l'écriture manuelle (Story 36-3) : les **soldes de départ** (`isValidAmount` + `replace(',', '.')`), le **prix
unitaire** et le **montant attendu** d'une facture fournisseur (`decimalForServer`), le **règlement** d'une facture client
(`Number(amount)`). « 1.250 » y partait tel quel, et le serveur le lisait 1,25 franc. Désormais ils lisent par
`readJournalAmount` : « 1.250 » refusé comme ambigu (message commun `common-amount-ambiguous`), « 1'250 » et « 1,25 » acceptés,
un montant du serveur rouvert jamais ambigu (`1.2340`). **Frontend, clés i18n (quatre locales) et manuel utilisateur (`.tex`)
seulement ; aucun changement du serveur ; aucune migration.**

Worktree `/home/gcorbaz/devel/kesh-36-18`, branche `story/36-18-montants-ambigus-autres-ecrans`. ⚠️ La branche porte deux fusions
faites à l'étape 0 (`origin/epic/36-0.18.0`, puis la 36-15 **en revue, non fusionnée** — écart décidé par l'orchestrateur,
C-36-18-12) : ne relis pas leur contenu. Le diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-36-18 diff 668556766 454a63942` (`668556766` = étape 0, après les fusions ; `454a63942` = dernier
commit de code). Les commits suivants ne portent que la fiche, le registre et le `sprint-status.yaml`. Langue du rapport :
français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/36-18a-montants-ambigus-livres.md` en entier (D1 à D10, AC1 à AC11, tests, mutations
M1 à M18, Dev Notes, **Note de version**, **Change Log** et **Dev Agent Record**) ; les registres `e36-choix-autonomes-e360.md`
(C-e360-6, -11, -17 ; C-e350-18, -19 cités) et `e36-choix-autonomes-36-18.md` (C-36-18-1 à 5, 11, 12). Ne prends pas le Dev
Agent Record pour argent comptant : recoupe-le avec les journaux hors dépôt (lecture seule) sous
`/home/gcorbaz/devel/kesh-gate-logs/e36-18/` : `etape0-inventaires.log`, `rouges.log`, `e2e-rouges.log`, `e2e-spec-vert.log`,
`vitest-cible-dev.log`, `check-dev.log`, `mut/resume.log` et `mut/M*.log`, `review-test-fast.log`, `review-gate-ci.log`,
`review-front.log`, `bases.log`. Les scripts qui les ont produits sont à côté (`vitest-cible.sh`, `bases-voie2.sh`,
`e2e-3618a.sh`, `mut/mutations.py`) : lis-les, ne les lance pas. **Captures** (thème clair, base jetable `kesh_e2e_e3618a`,
prises par la spec E2E) sous `/home/gcorbaz/devel/kesh-gate-logs/e36-18/captures/` (`e3618a-*.png`, treize fichiers) — regarde-les
si ton outil lit les images, et compare chacune à l'état que son nom annonce.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis tes trois rapports. **Aucun fichier temporaire**, nulle
  part ; **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se
  **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni **`pkill`**, ni **`killall`**, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`,
  `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`,
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (seule forme permise du contrôle de PDF, E6).
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les aides communes** (`frontend/src/lib/features/journal-entries/balance.ts` : `amountEntryPayload`, `amountEntryProblem`,
   retrait de `defaultAmountReader` et des valeurs par défaut de `read` ; `invoices/unit-price.ts`, ré-export et délégation ;
   `journal-entries/amount-entry-message.ts`, neuf) contre D1, D2, D3, C-36-18-11. Comportement de la 36-3b inchangé ? Un
   appelant de `computeBalance` / `classifyLine` sans lecteur reste-t-il (`git grep -n "computeBalance(\|classifyLine(" -- frontend/src`) ?
   Le module neuf `amount-entry-message.ts` (non nommé par la fiche) est-il justifié, et sans cycle d'import ?
2. **Les soldes de départ** (`routes/(app)/settings/opening-balances/+page.svelte`, `features/opening-balances/opening-balances-totals.ts`)
   contre D4, D5 : lecture brute non rognée (« ␣1250␣ » refusé comme avant), filtres `isEnteredLine`, `complementInvalid`,
   charge `journalAmountPayload(…) ?? '0'` (un `null` atteignable ?), message sous chaque ligne des **deux** grilles, `id` et
   `aria-describedby` (deux grilles exclusives : `id` uniques ?), totaux, contrepartie du complément.
3. **La facture fournisseur** (`supplier-invoice-form.ts`, `SupplierInvoiceForm.svelte`) contre D2, D6, D7 : `unitPriceOf`,
   `sumTtc`, `lineNotInCentimes`, `linesInvalid`, `firstLineUnitPriceProblem`, `linesForServer` (le repli `?? ''` hors garde),
   `formLineFromResponse`, `paymentFields` (contrat V1-4 : jamais la saisie brute) ; **ordre des gardes** de `submit` (date,
   échéance, montant attendu, prix, `linesInvalid`, centime) ; `expectedAmountProblem` en mode QR (nul) ; rouverture
   (`readableUnambiguousAmount`) ; le mode document (troisième route `…/imported-supplier-invoices/{id}/complete`, V2-4) ; la
   quantité hors de la règle.
4. **Le règlement** (`invoices/SettleInvoiceDialog.svelte`) contre D8 : ordre (dates, montant, échelle, reste dû, compte),
   `reading` sur la saisie rognée, comptage des décimales sur la forme canonique de la **charge** (#476 : « 10.0100 »,
   « 1250.0000 »), reste dû comparé sur la valeur, `onConfirm` qui envoie la charge, `aria-invalid` et `aria-describedby`
   (seulement quand le bandeau dit le refus du montant). Une saisie qui passerait `clientError` sans lecture claire ?
5. **Les textes** : les quatre `messages.ftl` (quatre clés neuves, voisines de `common-account-no-match` et de
   `supplier-invoices-err-line-not-in-centimes`), justesse des traductions et des exemples (`1.25`, non `1.2500`) ; replis égaux à
   `fr-CH` ; `{ $max }` interpolé (2 au règlement, 4 ailleurs) ; `lint-i18n-ownership` ; `i18n-keys.test.ts` (`sitesTotal`
   3082 → 3086 : recompte vrai ? le doublon fusionné du règlement est-il sans effet de bord ?).
6. **Le manuel contre le code** : `docs/manual/fr/user-manual.tex`, § *Reprise de comptabilité* (item *Saisissez les soldes*),
   § *Enregistrer et annuler un règlement* (paragraphe *Enregistrer*), § *Les montants* de la facture fournisseur — chaque phrase
   est-elle vraie au code (« sans espace avant ni après » pour les soldes ; « une saisie … à plus de quatre décimales écrites n'est
   pas lue » au règlement ; « pour un prix, le message nomme la ligne » ; « La quantité n'est pas concernée ») ? Contrôle AC9 sur le
   `.tex` : `/usr/bin/grep -n 'refusé comme ambigu' docs/manual/fr/user-manual.tex` (une ligne par section, plus `:1171` de la 36-3b)
   et `/usr/bin/grep -n 'tels quels' docs/manual/fr/user-manual.tex` (plus la ligne du § *Les montants*). **PDF** : non régénéré dans
   la story (AC9 : à l'intégration) — vérifie que le PDF actuel ne porte PAS encore la phrase neuve, par la forme E6 :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '10.000 est refusé'` (attendu : 0),
   et nomme-le comme point d'intégration, non comme défaut.
7. **Les tests et les témoins** : `balance.test.ts` (retrait du lecteur par défaut : attentes inchangées ?),
   `supplier-invoice-form.test.ts` (cas `3.333` → `3.3330`), `SupplierInvoiceForm.test.ts`, `SettleInvoiceDialog.test.ts` (mock
   d'`i18nMsg` aligné, V2-1 ; trois tests contraires réécrits, C-36-18-5), `opening-balances-page.test.ts`,
   `opening-balances-totals.test.ts` ; la spec E2E `frontend/tests/e2e/montants-ambigus.spec.ts` (méthode + route, relecture en
   valeur, `expect.soft` sur le message — un test qui passerait **à vide** ?). Les rouges (`rouges.log` : 49 ; `e2e-rouges.log` : les
   quatre requêtes parties avec `1.250`) ; les mutations M1 à M18 (`mut/resume.log`, `mut/M*.log`) : chacune tuée par le test
   qu'elle vise ? Propose des mutations plausibles non jouées.
8. **Le périmètre** (AC10) : `git diff --stat 668556766 454a63942` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
   `amount-ambiguity.ts`, `JournalEntryForm.svelte`, `VatPurchaseAssistant.svelte`, `TransactionSplitModal.svelte`,
   `InvoiceForm.svelte`, `bank-accounts/` ; aucun `CHANGELOG.md` ; la note de version de la fiche.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne les sept fichiers de code, les quatre `messages.ftl`, les tests et la spec E2E. Cherche : une saisie claire
refusée, une saisie ambiguë acceptée ; un montant envoyé sous une autre forme que celle lue ; un `$derived` dont le type ne se
rétrécit pas ; un texte en dur hors i18n ; un `id` dupliqué ; un test qui passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

« 0.500 », « 001.250 », « 1'250'000 », « 1’250 » (U+2019), « 12,500 », « 0,125 », « 10.01000 », « 1e3 », « .5 », « 5. »,
« -5 », blancs de tête ou de fin, chaîne de blancs ; quinze chiffres entiers et plus ; saisie de la quantité « 1.250 » avec un
prix ambigu ; ligne vide + ligne ambiguë (laquelle est nommée ?) ; deux lignes ambiguës ; montant attendu ambigu puis scan QR ;
brouillon rouvert au prix `0.1250`, `1000.1230`, `1.2340` ; règlement « 10.0050 » pré-rempli, « 2000 » sur un reste dû de 2000,
date antérieure + montant ambigu (que nomme `aria-describedby` ?) ; soldes : complément et génération (deux grilles), report à
zéro, ligne « 0 ». Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC11 : livré, testé, au lieu prescrit ? L'étape 0 (huit LOW de la P2, lignes lues citées au Change Log ;
l'écart de base C-36-18-12 et l'incident de la fusion `7ec5a5240` commitée avec ses marqueurs, corrigé par `7d48aff2f`) ; la
clôture de la validation écrite ; les rouges et la sonde ; M1 à M18 ; les gates du passage en revue (`test-fast --ci` en voie 2,
`gate-ci` en voie 1, frontend complet) — chaque chiffre se retrouve-t-il au journal, horodaté après le dernier commit de code ?
Ce qui n'a pas tourné (E2E complet, PDF) est-il dit comme tel ? Les écarts déclarés (module `amount-entry-message.ts`, aide
`firstLineUnitPriceProblem`, `linesForServer` qui rend `''` hors garde, fixture d'AC3 à la quantité 10, `expect.soft` dans la
spec) sont-ils exacts et sans effet sur le livré ? Les captures montrent-elles chacune l'état annoncé (AC11) ? Le Dev Agent
Record n'affirme-t-il que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e36-18/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**. Un « 0 finding » non adossé à cette liste ne compte pas comme passe.
