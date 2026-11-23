# Prompt — revue de code P1 de la Story 48-12b (#975, #985), trois lentilles Opus, première passe complète

Tu relis la **Story 48-12b** de Kesh (comptabilité suisse, Rust/Axum, SvelteKit 5 runes, MariaDB), release **0.20.0**, Epic 48 :
**l'écran du refus de doublon d'une facture fournisseur, la réponse perdue d'un enregistrement, la complétion relue et le
repassage en brouillon d'une annulée en doublon**. Elle s'appuie sur la **48-12a** (même branche, avant elle), qui fait refuser au
serveur deux fois la même facture (`409 SUPPLIER_INVOICE_DUPLICATE`, `409 SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, `details :
{ supplierInvoiceId, internalNumber, status, confirmable }`, `confirmNotDuplicate`). L'écran est une **machine à états** décrite par
deux tables de la fiche, qui font foi : **table A** (formulaire de saisie, création, complétion, modification : E1 à E9) et **table
B** (fiche d'une facture annulée : F1 à F6), avec les **relevés C** (écritures asynchrones de la saisie) et **D** (gestes et
relectures de la fiche qui effacent l'avis F4). **Frontend, catalogues (quatre locales), manuel utilisateur `.tex`, spec E2E ;
aucun code Rust ; aucune migration.** Guy tient ses livres réels dans Kesh avec de vraies factures fournisseur : aucune saisie ne
doit se perdre, aucun second clic ne doit enregistrer deux fois, aucune écriture réussie ne doit être annoncée en échec.

Worktree `/home/gcorbaz/devel/kesh-48-12`, branche `story/48-12-reponses-perdues-fournisseurs`. Le diff de la 48-12b, **aplati** :
`git -C /home/gcorbaz/devel/kesh-48-12 diff d5b283d49 f35c56925` (dernier commit de la 48-12a → dernier commit de code de la 48-12b) — les commits sont
listés au Dev Agent Record de la fiche 48-12b (§ *Commits*) ; ceux de la 48-12a sont au Dev Agent Record de la fiche 48-12a.
Deux commits tardifs méritent un regard propre : `f7280c030` (style `disabled:opacity-60` du `fieldset`, trouvé par la recette
visuelle) et `f35c56925` (le décor de `supplier-internal-number.spec.ts` confirme ses brouillons sans numéro — vérifie que
la confirmation n'y masque aucun cas que le test voulait exercer).
Langue du rapport : français. Modèle : **Opus**, contexte frais, une lentille par agent.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md` en entier — « Ce qui se lit au code » 1 à 8, **D1**
(décisions de Guy du 2026-11-19 sur E6 et sa reprise, règles communes, relevé C, tables A et B, relevé D), **D2**, **D2bis**, **la
règle du drapeau**, D3, AC-1 à AC-10, tests et mutations, recette visuelle, manuels, Note de version, Change Log et Dev Agent Record
— et D7 de la fiche 48-12a (les trois textes partagés). Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le avec les
journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/` — journaux nommés au Dev Agent Record,
`mut-12b/*.log` et son script (lis-le, ne le lance pas), et les **captures** `captures/e4812b-*.png`.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`/`commit`/`reset`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`,
  `bash -c`/`sh -c`, `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
  `scripts/recette-montee.sh`, `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` ; ni `make`, `latexmk`,
  `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`, `svelte-check`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk` (sans
  redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, `tr`, et
  `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'` (seule forme du contrôle de PDF). Regarde les
  captures si ton outil lit les images.
- ⚠️ Dans ce shell, `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **AC et décisions de Guy** : table A ligne par ligne (`frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte`,
   `supplier-invoice-form.ts`, `SupplierInvoiceNotice.svelte`) — états figés (E2, E3, E5, E6, E8, E9) par **un** `<fieldset
   disabled>`, gardes synchrones (`submit` depuis E1, E4, E7 sans scan ni panneau ; confirmation depuis E5), instantané du corps
   gardé, bouton ⇔ `details.confirmable === true`, E6 sans bouton de confirmation ni d'envoi et sa reprise sans requête (décisions
   de Guy), variante « modification » (texte, lien vers la fiche, libellé), E9 (texte de la décision de Guy, lien nouvel onglet,
   ancien message absent) ; table B (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`) — F4, son bouton, F6, F3 ;
   D2 et D2bis (`new/+page.svelte` : `onRefusal` filtré, lectures `to_complete` puis `completed`, `{ retry: false }`, `locked`) ;
   **la règle du drapeau** (`unvalidateSupplierInvoice`, gestionnaires fléchés, `=== true`, forme d'appel sans troisième argument).
2. **Relevés A à D refaits par le symptôme** : **C** — par les `await`, `$effect`, `.then`, `setTimeout` et rappels de
   `SupplierInvoiceForm.svelte` et `SupplierCreatePanel.svelte` : chaque site qui écrit la saisie hors d'un état de saisie est-il
   résolu (garde primaire, et couche testée en unitaire ou par re-rendu, jamais par l'affaiblissement d'une garde primaire) ? un site
   neuf (C7, C8, ou autre) ? **D** — par les `onclick`, les conditions d'affichage par statut et les affectations de `invoice` de la
   fiche : chaque geste offert sur une annulée efface-t-il l'avis F4 comme la table le dit (`clearDuplicateRefusal`, appelée par
   `clearRefusals`, au début de `attach`, après la confirmation de `detach`, dans `reloadAfterRefusal`) ? une affectation de
   `invoice` qui ne passe pas par l'effacement ? **A** et **B** (relevés de la 48-12a) : seulement ce que l'écran en consomme (les
   rangs 91, 106, 107 du relevé B sont à cette fiche ; les rangs 68, 80, 84, 88 reçoivent un paragraphe neuf à la suite).
3. **Rejeu de `POST` et `PUT /invoices`** : sans objet à l'écran (D3ter de la 48-12a) ; vérifie seulement qu'aucun fichier de la
   facture **client** (`InvoiceForm.svelte`, `invoices/[id]`) n'est touché par le diff.
4. **Textes partagés entre les deux fiches** : les avis de doublon affichent les clés **du serveur** (`error-supplier-invoice-duplicate`,
   `-probable-duplicate`, `-duplicate-unvalidate`), jamais `err.message` ni une copie d'écran ; repli Svelte égal au catalogue
   fr-CH (et aux replis Rust de la 48-12a) ; clés neuves `supplier-invoices-*` aux quatre locales, justesse des traductions ;
   `lint-i18n-ownership` ; `sitesTotal` d'`i18n-keys.test.ts` et `i18n-un-repli-par-cle.test.ts` recomptés.
5. **Mutations** : `mut-12b/*.log` — chaque mutation de la fiche jouée et tuée par le test qu'elle vise ; celles déclarées
   **équivalentes** (défense en profondeur : deux défenses redondantes mutées **ensemble**) le sont-elles à raison ? aucune garde
   primaire affaiblie pour rendre une défense testable (règle de la fiche, sans exception) ? Propose des mutations non jouées.
6. **Manuels** : `docs/manual/fr/user-manual.tex` — paragraphes neufs à la suite des rangs 68, 80, 84, 88 ; encart « Si la réponse se
  perd » (rang 91) ; « Créer la facture d'un document » (rang 106) ; rang 107 réécrit — chaque phrase vraie au code ? Contrôles
   **positifs** au `.tex` (« Ce n'est pas un doublon », « peut-être abouti », « reprendre la saisie », « a peut-être été
   enregistrée », « déjà été enregistrée comme facture ») et **négatifs**, `.tex` et PDF aplatis (`tr '\n' ' ' | tr -s ' '`), commande
   de la fiche rejouée : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'relit pas encore'`.
   ⚠️ Le jeton « dans une autre session » figure aussi au rang 80 (paragraphe *Modifier*, propriété de la 48-12a, que la 48-12b ne
   touche pas) : le contrôle négatif de la fiche ne peut pas rendre 0 ; le Dev Agent Record dit la phrase discriminante employée —
   vérifie-la. Les PDF du dépôt ne sont **pas** régénérés par la story (jamais commités) : le contrôle sur le PDF régénéré a été fait
   au développement (journal nommé au Dev Agent Record) ; le PDF versionné est un point d'intégration, non un défaut.
7. **Factures fournisseur réelles existantes** : rien à l'écran ne refuse une facture existante à la lecture (liste, fiche,
   validation, paiement, lot, annulation inchangés) ; la saisie n'est jamais vidée (refus, issue inconnue, reprise) ; aucun fichier
   Rust, aucune facture client ni avoir ni PDF touché (`git diff --stat` de la 48-12b).
8. **Captures** (`/home/gcorbaz/devel/kesh-gate-logs/e4812/dev/captures/e4812b-*.png`, thème clair, base de démonstration jetable,
   prises par la spec E2E `frontend/tests/e2e/supplier-invoice-duplicate.spec.ts`) : E2, E3, E4, E5, E6 (création, modification,
   complétion), E7 « pour l'instant », E9, F4, E1 « Enregistrer » inactif (scan, panneau ouvert), notification de C2, E7 et F6 avec un
   doublon sans `details` — chacune montre-t-elle l'état que son nom annonce ? une capture manque-t-elle ?

## Lentille B — Blind Hunter (le diff seul, sans la fiche)

Relis ligne à ligne les fichiers de code, les catalogues, les tests et la spec E2E. Cherche : un contrôle de saisie resté actif en
état figé ; une double requête possible (deux événements avant le rendu, deux clics) ; un corps renvoyé qui n'est pas celui gardé ;
un drapeau parti sans clic ; un avis qui survit à tort ou disparaît à tort ; un lien sans `target="_blank" rel="noopener"` ; une
requête de relecture partie pour un refus de doublon ; un test qui passerait **à vide** ; un texte en dur hors i18n ; un sélecteur
E2E figé sur un libellé traduit.

## Lentille E — Edge Case Hunter (chaque branche, chaque borne)

Réponse perdue d'une **confirmation** ; 503 et `TypeError` ; `409` d'un des deux codes sans `details`, avec des `details` partiels
(identifiant en chaîne, `confirmable` absent) ; `confirmable: true` sur `SUPPLIER_INVOICE_DUPLICATE` (ce que rend la dévalidation) ;
complétion : document encore à traiter, complété sans facture, retiré, relecture en échec, `onRefusal` qui lève, document complété
par une autre session au **premier** envoi ; E6 → reprise → renvoi → `IMPORT_NOT_PENDING_COMPLETION` → E9 ; modification qui
rend `OPTIMISTIC_LOCK_CONFLICT` après une réponse perdue ; fournisseur « Dupont & Fils » (lien encodé) ; panneau « nouveau
fournisseur » annulé pendant la création puis résolu en E5 et en E6 ; scan qui se résout en état figé ; effet de l'échéance avec une
prop `suppliers` neuve en E5 ; F4 puis joindre / détacher (boîte annulée) / relire / repasser ; payer, annuler hors annulée ;
navigation d'une fiche à une autre. Chaque cas : comportement au code, test qui le couvre ou absence prouvée par `grep -nF`.

## Lentille A — Acceptance Auditor (la fiche contre le livré)

Chaque critère AC-1 à AC-10 : livré, testé, au lieu prescrit ? Chaque ligne du tableau « Tests à écrire » : le test existe-t-il au
fichier nommé, et sa mutation a-t-elle été jouée ? L'étape C0 (relecture des points 1 à 8 et des relevés C et D contre la branche à
jour) ; les écarts déclarés au Change Log (contrôle négatif « dans une autre session », fichiers de tests, consigne « outil Edit »)
sont-ils exacts et sans effet sur le livré ? Les gates (`test-fast`, `gate-ci`, frontend complet, **E2E complet** — chaque rouge
nommé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*) : chaque chiffre se retrouve-t-il au journal, au commit
annoncé ? La recette visuelle (AC, liste de la fiche) ; le statut `review` à la fiche et au `sprint-status.yaml` ; la Note de
version ; aucun module neuf.

## Rapport

Écris **un fichier par lentille** : `/home/gcorbaz/devel/kesh-gate-logs/e4812/review-12b-p1-B.md`, `…/review-12b-p1-E.md`,
`…/review-12b-p1-A.md`. Chaque rapport : findings numérotés (`B1`, `E1`, `A1`…) avec sévérité (CRITICAL / HIGH / MEDIUM / LOW),
lieu (`fichier:ligne`), ligne lue copiée, preuve, remède proposé ; **la liste des axes exercés ET non exercés** (les huit axes
ci-dessus, un par un) ; le décompte final **recompté depuis la liste**. Un « 0 finding » non adossé à cette liste ne compte pas
comme passe.
