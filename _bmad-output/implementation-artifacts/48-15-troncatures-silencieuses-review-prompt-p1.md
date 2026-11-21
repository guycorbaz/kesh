# Prompt — revue de code P1 de la story 48-15 « Troncatures silencieuses » (#1331, #1332, #1333, #1334), trois lentilles, première passe complète

Tu relis la story **48-15** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit 5 runes), release **0.20.0**. Kesh tient
les livres réels du Project Lead, qui a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir,
lignes, totaux, TVA, écritures, numéros, PDF) ne doit changer, et **aucun contact ni aucune société déjà enregistrés ne doivent être
refusés** à l'enregistrement. **Aucune migration.** Le geste est mécanique et répété, en deux lots ordonnés :

- **Lot A — serveur** (`crates/kesh-api`, `crates/kesh-i18n`) : le prénom et le nom d'une personne bornés **chacun** à 70 caractères
  (`PERSON_NAME_PART_MAX`, `helpers.rs`) dans `contacts.rs::validate_common` et dans `onboarding.rs::set_coordinates` (#1334) ;
  l'e-mail de la société borné à 320 dans `companies.rs::update_company_email`, clé `error-company-email-too-long` aux quatre
  catalogues (#1333) ; `MAX_EMAIL_LEN` de `contacts.rs` en `pub(crate)`.
- **Lot B — écrans** : retrait des `maxlength` qui coupaient en silence un texte **collé** — adresse du contact (cinq champs, #1331),
  pays du fournisseur neuf (#1332), e-mail de la société (#1333), prénom et nom du contact (#1334), et cinq champs qui finissent sur
  le PDF (nom d'article, numéro de client, conditions de paiement, raison sociale, délai de paiement). Sur la page des contacts, le
  motif d'un refus **de l'écran** (raison sociale > 255, délai hors 0–365) s'affiche désormais sous le champ (choix **C-e4815-4**) ;
  sur la page des articles, le motif s'affichait déjà au pied du formulaire.

Worktree `/home/gcorbaz/devel/kesh-48-15`, branche `story/48-15-troncatures-silencieuses`, partie de `epic/48-corrections-0-20-0`
= `45eb99ddd` (qui porte la 48-13 fusionnée). Diff à relire, **aplati** :
`git -C /home/gcorbaz/devel/kesh-48-15 diff 45eb99ddd 867be3a79` (`867be3a79` = dernier commit de code ; quatre commits de code :
`846746fbb` lot A, `865e54132` lot B, `81cd199fd` retrait d'un doublon sur la page des articles, `867be3a79` deux `data-testid`
pour l'E2E après un rouge de la garde KF-043). Les commits suivants ne portent que la fiche, le registre et ce prompt — vérifie-le
par `git -C /home/gcorbaz/devel/kesh-48-15 diff --stat 867be3a79 HEAD`. Langue du
rapport : français. Modèle : **Opus ou Sonnet** (première passe : jamais Haiku). Contexte frais.

Lis d'abord (lentilles E et A ; la lentille B lit le diff avant) : la fiche
`_bmad-output/implementation-artifacts/48-15-troncatures-silencieuses.md` en entier (« Ce qui se lit au code », D1 à D6, AC1 à
AC12, tableau des tests et des mutations, Note de version, **Dev Agent Record**, **Change Log**) ; le registre
`_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4815.md` (C-e4815-1 à 4) ; le dernier rapport de validation
`/home/gcorbaz/devel/kesh-gate-logs/e4813/validation-48-15-p4.md`. Ne prends pas le Dev Agent Record pour argent comptant : recoupe-le
avec les journaux hors dépôt (lecture seule) sous `/home/gcorbaz/devel/kesh-gate-logs/e4815/` : `rouge-rust.log`, `rouge-vitest.log`
(les rouges avant correctif), `cible-rust-lotA.log`, `clippy-lotA.log`, `fmt-lotA.log`, `vert-vitest-cible.log`,
`vert-vitest-products.log`, `vert-vitest-garde.log`, `e2e-contacts.log` et `e2e-contacts2.log` (+ `.backend`), `mut/` (scripts `mut-rust-4815.sh`, `mut-front-4815.sh`, `e2e-m6.sh` ;
un journal par mutation `mut-m1.log` … `mut-m19.log` ; le rejeu E2E de M6 `e2e-m6.log`, `e2e-m6.log.e2e`, `e2e-m6.log.build`),
`test-fast-ci-review.log`, `gate-ci-review.log` et `gate-ci-review2.log`, `front-*-review.log` (première prise, un rouge) et
`front-*-review2.log`, et les scripts `e2e-4815.sh`, `captures-4815.sh`,
`captures-4815.mjs`. **Captures** (thème clair, base de test jetable `kesh_e2e_e4815`, données factices) :
`/home/gcorbaz/devel/kesh-gate-logs/e4815/captures/` — `01-contact-rue-71-non-coupee.png`, `02-contact-rue-refus-saisie-gardee.png`,
`03-fournisseur-pays-suisse-refuse.png`, `04-societe-email-321-refuse.png`, `05a-contact-prenom-71-saisie-gardee.png`,
`05b-contact-prenom-71-refus-nomme.png`, `06-contact-delai-1000-refuse.png`, `07-contact-raison-sociale-256-motif.png`,
`08-article-nom-256-motif.png` ; `05-contact-prenom-71-refus-nomme.png` est une prise **périmée** (remplacée par 05a et 05b, laissée
faute de `rm`) : ne la juge pas. Regarde-les si ton outil lit les images.

⛔ Ne lis rien sous `~/kesh/` ni `~/kesh-test/`, ni le NAS. Aucune donnée réelle.

## Interdits absolus

- **Lecture seule.** N'écris, ne modifie, ne crée aucun fichier, hormis ton rapport. **Aucun fichier temporaire**, nulle part ;
  **aucune redirection** (`>`, `>>`, `tee`, `2>`, `&>`, `2>&1`, `> /dev/null` compris). Un fichier créé par erreur se **déclare**.
- **Aucun `rm`**, sous aucune forme ; ni `pkill`, ni `killall`, ni `kill`.
- **Aucun `git checkout`/`switch`/`restore`/`stash`**, aucun `git` qui écrit.
- **Aucun interpréteur, rien qui écrit, compile ou exécute du code** : ni `python3`, `perl`, `node`/`npm`/`npx`, `bash -c`/`sh -c`,
  `jq` ; ni **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script du dépôt ou de `kesh-gate-logs` (dont `mut-rust-4815.sh`, `mut-front-4815.sh`, `e2e-m6.sh`,
  `e2e-4815.sh`, `captures-4815.sh` : ils se **lisent**) ; ni `make`, `latexmk`, `cargo`, `docker`, `mariadb`, `sqlx`, `curl`,
  **`claude`**, `sed -i`, `playwright`, `vitest`.
- **Aucune commande `gh`**. **Aucun accès réseau.**
- Permis : `git diff`, `git show`, `git log`, `git grep`, `git ls-files`, `git cat-file -e`, `/usr/bin/grep`, `sed -n`, `awk`
  (sans redirection ni `system()`), `cat`, `ls`, `find` (sans `-exec`/`-delete`), `wc`, `sort`, `uniq`, `cmp`, et, pour un PDF, la
  seule forme `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c '<phrase>'`.
- ⚠️ `grep` est une fonction vers `ugrep` : emploie `/usr/bin/grep` ou `git grep`.
- **Toute affirmation de présence ou d'absence** s'appuie sur une sortie `grep -nF` (ou `sed -n`) copiée au rapport ; tout chemin
  cité se vérifie (`git cat-file -e <ref>:<chemin>` ou `ls`).

## Les axes obligatoires, pour chaque lentille

1. **Les bornes serveur (lot A).** Pour chaque champ dont l'écran perd son `maxlength`, le refus serveur existe-t-il, **avant toute
   écriture**, compté comme la colonne le compte (caractères, après rognage) et nommant le champ ? Lis : `validate_common`
   (`crates/kesh-api/src/routes/contacts.rs`, branche `Personne` et reste), `set_coordinates` (`onboarding.rs` : prénom ou nom
   **seul**, ou les deux), `update_company_email` (`companies.rs`), `check_lengths` (`address_input.rs`). Le prénom et le nom sont-ils
   bornés **sur la valeur enregistrée** (rognée) ? Une valeur de 70 caractères multi-octets passe-t-elle ? Le refus de l'e-mail
   précède-t-il bien `is_valid_email_simple` et le verrou optimiste ? Restent-ils des écrivains de `contacts.first_name`/`last_name`,
   `companies.first_name`/`last_name`/`email` qui ne bornent pas (inventaire **par le symptôme** : toute écriture de ces colonnes, de
   l'API à l'écran, MCP et restauration compris) ? L'ordre des refus de `set_coordinates` (prénom/nom, puis nom composé de la 48-13,
   puis adresse) est-il cohérent avec la fiche (D3 b) ?
2. **Aucune donnée existante refusée (AC9), aucune facture touchée (AC10).** Chaque borne neuve égale-t-elle sa colonne (schéma :
   `crates/kesh-db/migrations/20260705000002_person_names.sql`, `20260709000003_companies_email.sql`, et le squash
   `crates/kesh-db/test-schema/`) ? Une route qui **recopie** l'état (les autres routes de `companies.rs`, le PUT d'un contact
   réenregistré tel quel) peut-elle désormais refuser une valeur déjà en base ? Le diff touche-t-il une facture, un avoir, leur PDF ?
   `git -C /home/gcorbaz/devel/kesh-48-15 diff 45eb99ddd...867be3a79 --stat -- crates/kesh-api/src/routes/invoice_pdf_service.rs crates/kesh-qrbill crates/kesh-report 'frontend/src/routes/(app)/invoices' frontend/src/lib/components/invoices`
   doit être vide.
3. **Les écrans (lot B).** Les `maxlength` visés sont-ils **tous** retirés, et seulement eux (les sites hors périmètre — e-mail et
   téléphone du contact #1336, description de l'article #1340, `ContactPersonsManager` #1335 — restent) ? Chaque refus du serveur
   s'affiche-t-il, **saisie conservée**, sur chaque chemin (création, modification, conflit de version, refus de l'écran) ? Le choix
   **C-e4815-4** (`nameTooLongMsg`, `paymentTermsDaysMsg`, `contacts/+page.svelte`) : `formValidation` garde-t-il exactement son ordre
   et ses textes ? Le motif sous le champ apparaît-il et disparaît-il au bon moment (type `Personne` avec un `formName` long resté d'une
   bascule de type ; délai vidé ; délai « abc »)? Un motif peut-il s'afficher à l'ouverture d'une fiche existante, et est-ce juste ?
   Le compte de `formName.trim().length` (unités UTF-16) diverge-t-il du serveur (caractères) d'une manière qui gêne ? Le panneau du
   fournisseur neuf : « Suisse » atteint-il le serveur, et que se passe-t-il si seul le pays est rempli (#1342, hors périmètre) ?
4. **Les tests et les mutations.** Chaque AC a-t-il un test qui peut rougir, ou est-il déclaré filet avec sa raison ? Les mutations M1
   à M19 sont-elles tuées **pour la bonne raison**, chacune par son test titré (journaux `mut/`) ? M1 à M5 : le test lit-il
   `error.code` (sans quoi `DATA_LENGTH_OR_RANGE` le laisserait vert) ? Les assertions d'absence de `maxlength` ont-elles leur **sonde
   positive** (champ trouvé avant l'assertion) et le bon état de formulaire ? Le rejeu E2E de M6 rougit-il pour la raison de l'AC5
   (`e2e-m6.log.e2e`) ? Un test vert par construction (mock qui renvoie l'attendu, `fireEvent.submit` qui contourne un bouton
   désactivé, comparaison de vide à vide) ? L'E2E neuf (`frontend/tests/e2e/contacts.spec.ts`) rend-il l'état tel qu'il l'a trouvé,
   même en échec ?
5. **i18n et compteurs.** La clé `error-company-email-too-long` est-elle aux quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`),
   même place, traduction juste ? `sitesTotal` (`frontend/src/lib/shared/i18n-keys.test.ts`) : inchangé — est-ce juste, les appels
   `i18nMsg` de `contacts/+page.svelte` n'ayant fait que se déplacer ? Un littéral d'interface ajouté sans clé ? Les messages de D3
   (« Prénom : 70 caractères maximum ») sont en **français seul** : limite assumée (fiche D3, #710) — la juger, non la découvrir.
6. **Le manuel** (`docs/manual/fr/user-manual.tex`, item « Nom » et « Adresse structurée » du contact ; paragraphe de l'e-mail de la
   société dans « Vos coordonnées sur la facture ») : dit-il ce que fait le code (70 chacun, rue 70, numéro et NPA 16, localité 35,
   e-mail 320, refusé et non coupé, saisie gardée) ? Les manuels de/en/it portent-ils un passage équivalent devenu faux (`git grep` sur
   `docs/manual/`) ? Le **PDF n'est pas régénéré dans la branche** (il le sera à la release) : forme du contrôle,
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'Le prénom et le nom comptent 70 caractères'`
   — `0` attendu dans la branche (après régénération : `1`) ; le dire, ne pas le compter comme défaut. Même forme pour
   `'320 caractères au plus'`.
7. **La fiche, le registre, la Note de version.** Le Dev Agent Record dit-il vrai (journaux et commits cités ; décomptes recomptés
   depuis le diff : `#[sqlx::test]` et `it(`/`test(` ajoutés, tests exécutés par les boucles `for` de `contacts-page.test.ts`) ? Les
   décalages de l'étape 0 sont-ils exacts (lignes citées à `45eb99ddd`) ? La Note de version dit-elle vrai, champ par champ ? Les choix
   C-e4815-1 à 4 sont-ils cohérents avec le code ?

## Lentilles

- **B — Blind Hunter** : le diff seul d'abord (sans la fiche), défauts de code, d'état, de sécurité ; puis les axes.
- **E — Edge Case Hunter** : toutes les branches et états limites (axes 1 et 3 surtout) : valeurs à la borne et une au-delà,
  multi-octets, blancs autour, prénom ou nom seul, champ vidé, bascule de type, conflit de version, données déjà enregistrées.
- **A — Acceptance Auditor** : chaque AC (AC1 à AC12) et chaque décision (D1 à D6) contre le code, les tests, les journaux et les
  captures (axes 2, 4 à 7).

## Rapport

Un rapport par lentille : `/home/gcorbaz/devel/kesh-gate-logs/e4815/review-p1-B.md`, `…/review-p1-E.md`, `…/review-p1-A.md`. Pour
chaque finding : identifiant (`B1`, `E1`, `A1`…), sévérité (CRITICAL, HIGH, MEDIUM, LOW), fichier et ligne, extrait cité, scénario,
correctif proposé. Verdict chiffré en tête. **Liste des axes exercés ET non exercés** (un « 0 finding » sans elle ne compte pas). Ce
qui n'a pas tourné reste écrit : la revue n'exécute rien ; `test-fast --ci`, `gate-ci` et le frontend complet ont tourné au passage en
revue (journaux cités au Dev Agent Record) ; l'E2E complet et le `gate-ci` de clôture **non** — seule la spec touchée
(`contacts.spec.ts`) a tourné, et le rejeu E2E de M6.
