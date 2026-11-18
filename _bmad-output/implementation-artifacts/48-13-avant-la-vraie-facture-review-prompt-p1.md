# Prompt — revue de code P1 de la story 48-13 « Avant la vraie facture » (#1233, #1243, #1226), trois lentilles, première passe complète

Tu relis la story **48-13** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release **0.20.0**. Kesh tient
les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir,
lignes, totaux, TVA, écritures, numéros, PDF) ne doit changer. Trois corrections, **aucune migration** :

1. **#1233** — `InvoiceForm.svelte` : un compteur de génération du client (`contactGen`) ; la réponse tardive du chargement initial
   du client d'un brouillon (`getContact`) est jetée, succès comme échec, si l'utilisateur a entre-temps choisi un client
   (`onContactSelect`), tapé un texte (`onContactDraftChange(true)`, D10) ou rechargé sur conflit (`reloadFromServer`, D2, défensif).
2. **#1243** — `settings/+page.svelte` : retrait des sept `maxlength` (cinq champs de l'adresse, téléphone, site web) qui coupaient en
   silence un texte collé ; le serveur refuse déjà, l'écran affiche son message et garde la saisie. L'e-mail **garde** son
   `maxlength={320}` (48-15, #1333).
3. **#1226** — `onboarding.rs`, `set_coordinates` : le nom (composé « prénom nom » s'il y a lieu) est jugé par
   `validate_company_name_for_qr`, l'adresse par `validate_required_for_qr` (jeu de caractères de la QR-facture), au lieu de
   `is_empty` et `validate_required`.

Worktree `/home/gcorbaz/devel/kesh-48-13`, branche `story/48-13-avant-la-vraie-facture`. Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-48-13 diff 78d8c4c55 b41df52da` (`78d8c4c55` = fiches validées, pointe avant le code ;
`b41df52da` = dernier commit de code). Les commits suivants ne portent que la fiche, le registre et ce prompt — vérifie-le par
`git -C /home/gcorbaz/devel/kesh-48-13 diff --stat b41df52da HEAD`. La branche porte aussi la fiche de la **48-15**
(`48-15-troncatures-silencieuses.md`) : **hors de cette revue**. Langue du rapport : français. Modèle : **Opus ou Sonnet**
(première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-13-avant-la-vraie-facture.md` en entier (« Ce qui se lit au code », D1 à D8 et D10, AC1 à
AC13, tableau des tests et des mutations, Note de version, **Dev Agent Record**, **Change Log**) ; le registre
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4813.md` (C-e4813-1 à 6) ; le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e4813/validation-p4-ciblee.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4813/` : `rouge-rust.log`, `rouge-vitest.log`
(les rouges avant correctif), `vert-vitest-cible.log`, `vert-rust-cible.log`, `clippy.log`, `check.log`, `e2e-neufs.log` (+ `.backend`),
`mut/` (le script `muter.sh`, un journal par mutation `mut-m1.log` … `mut-m8.log` et les E2E mutés `e2e-m1.log`, `e2e-m5.log`,
`e2e-m6.log`), `test-fast-ci-review.log`, `gate-ci-review.log`, `front-*-review.log`, et les scripts `e2e-4813.sh`,
`captures-4813.sh`, `captures-4813.mjs`. **Captures** (thème clair, base de test jetable `kesh_e2e_e4813`, données factices) :
`/home/gcorbaz/devel/kesh-gate-logs/e4813/captures/` — `01-adresse-rue-71-non-coupee.png`, `02-adresse-refus-serveur-saisie-gardee.png`,
`03-telephone-trop-long-refuse.png`, `04-brouillon-client-choisi-garde-apres-reponse-tardive.png`,
`05-assistant-nom-71-refuse-toast.png` ; regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `muter.sh`, `e2e-4813.sh`, `captures-4813.sh` : ils se
  **lisent**) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`, **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **#1233, la garde du client.** `contactGen` (`frontend/src/lib/components/invoices/InvoiceForm.svelte`) : la réponse initiale,
   `.then` **et** `.catch`, est-elle jetée après chaque geste qui fixe le client, et seulement après eux ? Parcours toutes les
   séquences : choix sans frappe, frappe puis choix, frappe puis champ vidé (limite assumée D10), second choix, choix pendant
   « Recharger », double « Recharger », réponse initiale en échec, facture sans geste (la réponse doit s'appliquer comme avant, et le
   bandeau d'échec s'afficher comme avant). L'`$effect` du chargement initial peut-il se rejouer (il lit `selectedContact` et
   `invoice`) et capturer une nouvelle génération qui jetterait à tort, ou au contraire relancer un `getContact` ? `ContactPicker`
   appelle-t-il `onDraftChange(true)` dans un cas où il ne faudrait pas jeter (montage, client changé par le parent) ? Le rechargement
   (`reloadFromServer`, garde `reloadSeq`) reste-t-il voulu et intact ?
2. **#1243, les champs de la société.** Les sept `maxlength` sont-ils tous retirés, l'e-mail gardé ? Chaque champ trop long est-il
   refusé par le serveur **avant toute écriture** (`crates/kesh-api/src/address_input.rs`, `check_lengths` ; `companies.rs`,
   `MAX_PHONE_LEN`, `MAX_WEBSITE_LEN`, `normalize_contact_field`) et compté comme le serveur le compte (caractères, après rognage) ?
   Un pays de plus de deux lettres, un NPA de 17, une localité de 36 : refus nommé ? La saisie reste-t-elle sur tous les chemins
   d'erreur de `saveAddress` et `saveContactDetails` (400, 409 et relecture, échec de relecture) ? Un texte trop long rendu à un
   **conflit de version** (409 → `reloadAfterConflict`) se comporte-t-il bien ? Un autre écran ou composant de la société borne-t-il
   encore en silence ces sept champs (hors 48-15) ?
3. **#1226, l'assistant.** `set_coordinates` (`crates/kesh-api/src/routes/onboarding.rs`) : le nom jugé est-il celui qui est
   **enregistré et rendu au PDF** (composé « prénom nom », rogné) ? `validate_company_name_for_qr` et `validate_required_for_qr`
   jugent-ils exactement ce que le PDF jugera (`kesh-qrbill`) ? Un prénom ou un nom portant à lui seul un caractère refusé ou un U+00AD ?
   Le prénom et le nom enregistrés (`first_name`, `last_name`) restent-ils cohérents avec le nom rogné ? L'ordre des refus (nom,
   adresse, IDE, puis verrou d'étape) est-il celui de la fiche (D5) ? `validate_required` a-t-il encore un appelant de production, et
   ses tests ont-ils encore un sens ? **AC12** : une donnée existante peut-elle être refusée à l'enregistrement par une route qu'elle
   traverse (la route n'agit qu'à l'étape 5, `lock_state_at_step(&mut tx, 5, true)`) ?
4. **Factures réelles (AC9, AC12).** Le diff touche-t-il, directement ou par effet, une facture ou un avoir existants, leur PDF
   (`invoice_pdf_service.rs`, `kesh-qrbill`, `kesh-report`), leurs écritures, leur numéro ? Un brouillon ouvert puis enregistré sans
   geste envoie-t-il le même corps qu'avant (test AC9 a) ? `git diff 78d8c4c55 b41df52da --stat -- 'frontend/src/routes/(app)/invoices/[id]/+page.svelte' crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report` est-il vide ?
5. **Les tests et les mutations.** Chaque AC a-t-il un test qui peut rougir, ou est-il déclaré filet (AC2, AC4 « part entier »,
   AC6 « accepte », AC8 a, AC9 a/d) avec sa raison ? Les mutations M1 à M8 (pas de M5 en Vitest ; M5 en E2E ; M1 et M6 rejouées en E2E)
   sont-elles tuées **pour la bonne raison** (lis les journaux `mut/`, dont la première série M1/M2/M3/M8, fausse, rejouée — Dev Agent
   Record) ? Un test vert par construction (mock qui renvoie ce qu'on attend, comparaison de vide à vide, attente qui ne laisse pas la
   réponse arriver) ? Dans `invoice-contact-tardif.spec.ts`, l'attente après `liberer()` laisse-t-elle vraiment la réponse se traiter
   avant l'assertion ? Les E2E rendent-ils l'état partagé (adresse de la société) tel qu'ils l'ont trouvé, même en échec ?
6. **Le manuel** (`docs/manual/fr/user-manual.tex`, l'item « Coordonnées légales » de l'assistant, § *Corriger votre adresse* et
   § *Corriger le nom de votre entreprise*) : dit-il ce que fait le code (bornes 70/16/16/35, refus nommé et non coupe, saisie gardée,
   téléphone et site web refusés avec le maximum, contrôles de l'assistant, « se corrigeait ainsi … avant la version~0.20.0 ») ? Les
   manuels de/en/it portent-ils un passage équivalent devenu faux (`git grep` sur `docs/manual/`) ? Le **PDF n'est pas régénéré dans la
   branche** (il le sera à la release) : forme du contrôle,
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé ensuite par la QR-facture se corrige ainsi'`
   — dans la branche il rend encore `1`, c'est attendu (après régénération : `0`) ; le dire, ne pas le compter comme défaut. Même
   forme pour `'jamais coupé'` (`0` attendu dans la branche).
7. **i18n et compteurs** : aucune clé neuve attendue (`crates/kesh-i18n/locales/*/messages.ftl`, `frontend/src/lib/shared/i18n-keys.test.ts`
   et son `sitesTotal` inchangés) ; vérifier que le diff n'ajoute aucun littéral d'interface qui devrait passer par une clé.
8. **La fiche, le registre, la Note de version** : le Dev Agent Record dit-il vrai (journaux et commits cités, décomptes de tests
   recomptés depuis le diff : `#[sqlx::test]` et `it(`/`test(` ajoutés) ? La Note de version dit-elle vrai ? Les choix C-e4813-1 à 6
   sont-ils cohérents avec le code ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de sécurité ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et séquences (axes 1, 2 et 3 surtout), états limites (réponses dans tous les ordres,
  échecs, conflits, prénom/nom, caractères limites, données déjà enregistrées).
- **A — Acceptance Auditor** : chaque AC (AC1 à AC13) et chaque décision (D1 à D8, D10) contre le code, les tests, les journaux et les
  captures (axes 4 à 8).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e4813/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md`. Pour
chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario,
correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). Ce
qui n'a pas tourné reste écrit : la revue n'exécute rien ; `test-fast --ci`, `gate-ci` et le frontend complet ont tourné au passage en
revue (journaux cités au Dev Agent Record) ; l'E2E complet et le `gate-ci` de clôture **non** — seules les trois specs touchées
(`invoice-contact-tardif.spec.ts`, `company-address.spec.ts`, `onboarding-motif.spec.ts`) ont tourné.
