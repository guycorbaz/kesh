# Prompt — revue de code P1 de la Story 48-10 (#1173, #1175, #1176, #1177), trois lentilles, première passe complète

Tu relis la **Story 48-10** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB), release **0.20.0**, Epic 48 :
**les montants « 1.250 » des réglages, du catalogue et des filtres**. Avant la story, quatre écrans lisaient un montant autrement
que l'écriture (Story 36-3) : le **prix d'un article** (`PRICE_RE` + `replace(',', '.')` : « 1.250 » partait et le serveur le
lisait 1,25 franc, repris ensuite par chaque ligne de facture — **P1**), les **frais d'un niveau de rappel** (`Number(fFee)`, saisie
envoyée brute ; le serveur refusait « 1.250 » d'un « plus de 2 décimales » qui poussait à écrire `1.25`), le **montant minimal
d'une facture** (saisie rognée envoyée brute, « 1.250 » accepté et lu 1,25 — **P1**), les **bornes de montant du filtre des
écritures** (envoyées brutes : liste fausse sans le dire, ou vide sur un 400 tu). Désormais ils lisent par `readJournalAmount` :
« 1.250 » refusé comme ambigu (message commun `amountEntryMessage`), « 1'250 » et « 1,25 » acceptés, la borne de filtre ambiguë
omise en le disant. **Frontend, une clé i18n (quatre locales), manuels `.tex` seulement ; aucun changement du serveur ni de l'API ;
aucune migration.** Guy tient ses livres réels dans Kesh et emploie le catalogue, les rappels et la facturation : aucune forme
juste ne doit devenir refusée, aucune saisie ne doit se perdre, aucun montant enregistré ne doit changer.

Worktree `/home/gcorbaz/devel/kesh-48-10`, branche `story/48-10-montants-ambigus-reglages-filtres`, partie de `af4e0e313` (tête de
`epic/48-corrections-0-20-0` avec la 48-13 et la 48-15 fusionnées). Les commits `bae6cc6ca` à `3cba47602` ne portent que la fiche
(validation). Le diff à relire, **aplati** : `git -C /home/gcorbaz/devel/kesh-48-10 diff 3cba47602 e4bf86564`
(`e4bf86564` = dernier commit de code, spec E2E seule ; `e076b9913` = dernier code de production ; `90a6e08b0` et `7d8c779ca` ne
portent que les manuels et ce prompt ; le commit qui suit ne porte que la fiche et ce prompt — Dev Agent Record, § *Commits*). Langue
du rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-10-montants-ambigus-reglages-filtres.md` en entier (D1 à D6, AC1 à AC12, tests, mutations
M1 à M20, Dev Notes, **Note de version**, **Change Log** et **Dev Agent Record**) ; le registre
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4810.md` (C-e4810-1 à 5) ; les décisions communes de la 36-18a
(`36-18a-montants-ambigus-livres.md`, D1 à D4, D9). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4810/` : `etape0-inventaires.log`,
`etape0-dunning.log`, `etape0-minimum.log`, `etape0-manuels.log`, `rouges.log`, `verts-cibles-1.log`, `check-1.log`, `check-2.log`,
`e2e-verts-1.log`, `e2e-verts-2.log`, `e2e-rouges.log`, `mut/resume.log` et `mut/M*.log`, et les journaux des gates du passage en
revue : `bases.log`, `review-test-fast.log` (un rouge, `textes_coherents`, né du manuel), `textes-coherents-1.log`,
`textes-coherents-2.log`, `review-gate-ci.log`, `review-front.log` (un rouge, garde des sélecteurs E2E traduits),
`garde-selecteurs.log`, `review-front-2.log`, `e2e-verts-3.log`. Les scripts qui les ont produits sont à côté (`e2e-e4810.sh`, `mut/mutations-e4810.py`) : lis-les,
ne les lance pas. **Captures** (thème clair, base jetable `kesh_e2e_e4810`, prises par la spec E2E) sous
`/home/gcorbaz/devel/kesh-gate-logs/e4810/captures/` : les douze fichiers `e4810-*.png` (article ambigu, illisible, enregistré,
rouvert à `1.2340` ; frais ambigus, illisibles, enregistrés ; minimum ambigu, illisible, enregistré et relu ; filtre ambigu ; signet
`?amountMin=1.250`). Les `e3618a-*.png` du même dossier viennent des tests de la 36-18a, rejoués par la même spec : hors périmètre.
Regarde les `e4810-*.png` si ton outil lit les images, et compare chacune à l'état que son nom annonce.

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

1. **Le prix d'un article** (`frontend/src/lib/features/products/product-helpers.ts`, `routes/(app)/products/+page.svelte`) contre
   D1 : `classifyPriceInput` (ordre vide → négatif → lecture ; `negative` jugé sur la saisie rognée), `normalizePriceInput` qui rend
   `string | null`, la garde `if (unitPrice === null) return;` ; retrait de `PRICE_RE` et `isValidPrice` (aucun autre appelant :
   `git grep -n "isValidPrice\|PRICE_RE\|normalizePriceInput" -- frontend`) ; `priceMessage` sous le champ, `formValidation` qui le
   rend, pied du formulaire (`formError !== priceMessage`, `formValidation !== priceMessage`) — le motif du **nom** (48-15) et du
   **prix vide** reste-t-il visible ? ; `onPriceInput` (C-e4810-1 : comparaison à **tous** les messages du prix, écart à la formule
   littérale de D1 — l'ordre des écouteurs `bind:value` / `oninput` délégué est-il bien la cause ? un message du serveur ou des taux
   de TVA peut-il être effacé à tort ?) ; rouverture (`openEdit`, inchangée : `1.2340`, `1.25`, `1500.00`, `0.1230`).
2. **Les frais de rappel** (`routes/(app)/settings/dunning/+page.svelte`) contre D2 : `feeReading` dérivé de la saisie rognée,
   `feeError` sous le champ **hors** du bloc `formError`, ordre des gardes de `submitForm` (délai, vide, lecture), charge
   `fee.payload` à la création **et** à la modification ; `aria-invalid`, `aria-describedby`, `id` (`$props.id()`).
3. **Le montant minimal** (`routes/(app)/settings/invoicing/+page.svelte`) contre D3 : `minimumReading`, `minimumBlocked` dans le
   `disabled` du bouton, garde de `save()` (`notifyError` du même texte), charge `payload` ou `null` (le vide reste « aucun seuil ») ;
   le minimum part à chaque enregistrement : une forme rendue par le serveur (`5.0000`, `1.2500`, `etape0-minimum.log`) peut-elle
   devenir ambiguë ou changer de valeur ? Le plafond du règlement groupé (43-7) est-il intact ? `aria-describedby` nomme le refus
   **et** l'aide (C-e4810-3).
4. **Les filtres** (`routes/(app)/journal-entries/+page.svelte`) contre D4 : `amountBound` (charge ou `undefined`), `buildQuery`, le
   `$effect` d'URL qui en dérive, le message sous chaque borne (commun + `journal-entries-filter-amount-not-applied`, un seul
   paragraphe, C-e4810-2) ; la saisie jamais réécrite ; le signet `?amountMin=1.250` (message, borne non envoyée, URL réécrite sans
   elle) et `?amountMin=1,25` (désormais appliqué) ; `parseQueryFromUrl` inchangé ; le rechargement attendu (debounce) quand une
   borne devient ambiguë.
5. **Les textes** : les quatre `messages.ftl` (une clé neuve à la suite de `journal-entries-filter-amount-max`), justesse des
   traductions (D5) ; repli Svelte égal à `fr-CH` ; `{ $max }` interpolé (2 aux frais et au minimum, 4 au prix — qui garde
   « Format de prix invalide » pour l'illisible — et aux filtres) ; `lint-i18n-ownership` ; `i18n-keys.test.ts` (`sitesTotal`
   3495 → 3496 : un seul site neuf ?).
6. **Les manuels contre le code** : `docs/manual/fr/user-manual.tex` (item *Montant* de la recherche d'écritures, item *Prix
   unitaire HT*, § *Le montant minimum d'une facture*) et `docs/manual/fr/admin-manual.tex` (item *Frais* des rappels, § *Montant
   minimum d'une facture*) — chaque phrase est-elle vraie au code (« le champ garde votre saisie », « un lien enregistré qui porte
   une telle borne s'ouvre de même », « Avant, … `1'250` vidait la liste sans le dire », « Un prix de 1 à 999 francs, sans
   apostrophe, à trois décimales s'écrit avec la quatrième », « *Enregistrer* reste inactif », « le formulaire reste ouvert ») ?
   Contrôle AC9 sur le `.tex` : `/usr/bin/grep -n 'refusé comme ambigu' docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex`
   (quatre lignes de départ, `etape0-manuels.log`, plus une par section citée). **PDF** : non régénéré dans la story (AC9 : à
   l'intégration, jamais commité) — vérifie, aplati, que le PDF actuel ne porte PAS encore les phrases neuves (le texte aplati
   tient sur une ligne : `grep -c` y rend 0 ou 1) :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé comme ambigu'` (attendu 0 :
   aucune occurrence au départ, `etape0-manuels.log`) et
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Une borne ambiguë ou illisible'`
   (attendu 0 ; le manuel utilisateur porte déjà quatre occurrences de `refusé comme ambigu`, d'où une phrase propre à la story) ;
   nomme-le comme point d'intégration, non comme défaut. Les manuels DE, IT et EN ne décrivent pas la règle (`git grep -il "ambigu\|mehrdeutig\|ambiguous"
   -- docs/manual/de docs/manual/it docs/manual/en`, lancé **depuis la racine du worktree**).
7. **Les tests et les témoins** : `product-helpers.test.ts` (tests de `isValidPrice` retirés avec lui, AC10), `products-page.test.ts`
   (mock d'`i18nMsg` aligné ; `beforeEach(cleanup)`, C-e4810-5), `dunning-page.test.ts` (neuf), `settings-invoicing-page.test.ts`,
   `journal-entries-list-page.test.ts` (adresse mutable par `vi.hoisted`, `goto` lu) ; la spec E2E
   `frontend/tests/e2e/montants-ambigus.spec.ts` (quatre tests neufs : méthode + route, relecture en valeur, `expect.soft`, en-tête
   réécrit) — un test qui passerait **à vide** ? Les rouges (`rouges.log` : 59 ; `e2e-rouges.log` : les requêtes parties avec
   `1.250`) ; les mutations M1 à M20 (`mut/resume.log`, `mut/M*.log`) : chacune tuée par le test qu'elle vise ? Propose des
   mutations plausibles non jouées.
8. **Le périmètre** (AC7, AC10) : `git diff --stat af4e0e313 e4bf86564` ne nomme ni `crates/` hors
   `kesh-i18n/locales/`, ni `balance.ts`, `amount-entry-message.ts`, `unit-price.ts`, `group-settlement-max.ts`, ni `InvoiceForm`, ni
   `invoices/[id]` ; aucun `CHANGELOG.md`, aucun `sprint-status.yaml`, aucun PDF ; la note de version de la fiche.

## Lentille B — Blind (le diff seul, sans la fiche)

Relis ligne à ligne les cinq fichiers de code, les quatre `messages.ftl`, les tests et la spec E2E. Cherche : une saisie claire
refusée, une saisie ambiguë acceptée ; un montant envoyé sous une autre forme que celle lue ; un message qui reste affiché à tort
ou disparaît à tort ; un `$derived` dont le type ne se rétrécit pas ; un texte en dur hors i18n ; un `id` dupliqué ; un test qui
passe **à vide**.

## Lentille E — Edge (chaque branche, chaque borne)

« 0.500 », « 001.250 », « 1'250'000 », « 1’250 » (U+2019), « 12,500 », « 0,125 », « 12.125 », « 12.1250 », « 1234.125 »,
« 10.01000 », « 1e3 », « .5 », « 5. », « -5 », « - 5 », « −5 » (signe moins U+2212), blancs de tête ou de fin, chaîne de blancs ;
quinze chiffres entiers et plus ; article : nom vide **et** prix ambigu (que montre le pied ?), doublon de nom refusé par le serveur
puis retouche du prix, prix vide après Entrée ; frais : délai invalide **et** frais ambigus ; minimum : vide, `0`, rouvert à
`5.0000`, conflit de version (relecture) ; filtres : min ambigu et max clair, les deux ambigus, réinitialisation, signet portant
les deux bornes. Chaque cas : comportement au code, test qui le couvre (ou absence, prouvée par `grep -nF`).

## Lentille A — Acceptance (la fiche contre le livré)

Chaque critère AC1 à AC12 : livré, testé, au lieu prescrit ? L'étape 0 (relecture des lignes citées, inventaires rejoués, constats
AC2 et AC3 par `curl` sur base jetable, valeurs de départ des manuels ; les LOW de la P3 de validation déjà traités dans la fiche) ;
les rouges ; M1 à M20 ; les gates du passage en revue (`test-fast --ci`, `gate-ci`, frontend complet) — chaque chiffre se
retrouve-t-il au journal, horodaté après le dernier commit de code ? Ce qui n'a pas tourné (E2E complet, PDF) est-il dit comme tel ?
Les écarts déclarés (C-e4810-1 à 5 ; ajout du bloc de test des filtres par `cat >>` au lieu de l'outil Edit, Change Log) sont-ils
exacts et sans effet sur le livré ? Les captures montrent-elles chacune l'état annoncé (AC11) ? Le Dev Agent Record n'affirme-t-il
que ce qui a tourné ?

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e4810/review-p1-B.md`, `…/review-p1-E.md`,
`…/review-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW), lieu
(`fichier:ligne`), ligne lue copiée, rejeu ou preuve, remède proposé ; **la liste des axes exercés ET non exercés** ; le décompte
final **recompté depuis la liste**. Un « 0 finding » non adossé à cette liste ne compte pas comme passe.
