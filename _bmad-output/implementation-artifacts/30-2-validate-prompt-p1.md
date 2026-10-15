# Prompt — validation P1 de la spec, Story 30-2 (des documents à traiter à la facture fournisseur)

*Versionné le 2026-10-15. Deux lentilles, contexte frais chacune, en lecture seule stricte. Rotation (D6, rétro 25) :
passes complètes Sonnet ↔ Opus ; Haiku réservé à une passe ciblée de fin de boucle. P1 : **Sonnet** pour les deux
lentilles (la fiche a été écrite par Opus).*

Dépôt `/home/gcorbaz/devel/kesh-30-2`, branche `story/30-2` (base `15686d55`, kickoff de l'Epic 30).
Fiche : `_bmad-output/implementation-artifacts/30-2-documents-factures-fournisseur.md` (une fiche, **trois** stories :
30-2a AC1-AC9, 30-2b AC10-AC17, 30-2c AC18-AC23). Registre de la story :
`_bmad-output/implementation-artifacts/epic-30-choix-autonomes-302.md` (C-30-2-1 à 19). Epic :
`_bmad-output/planning-artifacts/epic-30-factures-fournisseur.md` § 30-2 ; registre de l'epic
`epic-30-choix-autonomes.md` (C-30-0-1 à 3). Issue #685 — **conception de Guy, elle fait foi** (`gh api
repos/guycorbaz/kesh/issues/685`, lecture seule) ; voisines #503, #235, #199, #315. Rétrospective
`epic-28-retro-2026-10-14.md` § 4 et § 7. Règles : `CLAUDE.md` (§ Review Iteration Rule et ses sous-sections — le
manuel nommé, les axes déclarés, les sites NON RÉSOLUS, la propagation post-patch, le recompte des comptes rendus ;
§ Règle de splitting préventif ; § Migration breaking policy P1-P8 ; § Un appariement automatique propose ; § Issue
Tracking Rule) ; `docs/agents-orchestration.md`.

⚠️ La Story 30-1 (#686, réconciliation des débits) est rédigée en parallèle dans un autre worktree et sera développée
**avant** la 30-2a : la fiche ne doit pas toucher `kesh-reconciliation`, et sa migration doit rester valable si la
30-1 en ajoute une.

## Axes prioritaires (communs aux deux lentilles)

1. **L'inventaire est-il vrai ?** Chaque ligne du tableau des prémisses (P-a à P-j) et de « Ce qui existe et se
   réutilise » se revérifie au code (`grep -nF`, `sed -n`) : refus d'un document sans QR, colonnes `NOT NULL`, refus
   liés au QR, absence d'aperçu, saisie manuelle, fichier qui quitte le dossier **à l'import** (P-d, prémisse de
   l'issue corrigée), bouton « Voir la facture d'origine » toujours affiché, RBAC des deux lectures, écriture sans
   lien, `.keshbackup` sans fichiers. Une prémisse corrigée à tort, ou un point du parcours de #685 laissé sans AC,
   est au moins **HIGH**.
2. **Périmètre et découpage** : chaque AC se rattache-t-il à un point du parcours de #685 ? Le décompte des modules
   par sous-story tient-il ? Chaque sous-story est-elle livrable et testable seule (en particulier : après la 30-2a,
   un document sans QR se complète-t-il vraiment par le formulaire existant ?) Une exclusion (table *Dehors*,
   C-30-2-13, C-30-2-16) qui laisserait #685 fermée sans que sa demande soit tenue est au moins **HIGH**.
3. **Migration** (P1-P8) : la migration de l'AC4 est-elle correctement qualifiée breaking ? `MODIFY COLUMN … NULL`
   sur MariaDB 10.11 conserve-t-il types, `CHARSET` et `COLLATE` tels qu'écrits ? La contrainte « tout ou rien »
   est-elle exprimable telle quelle (comparaison de booléens `IS NULL`) ? Le remplacement de l'index par une unicité
   peut-il échouer sur des données existantes ? Bump `min_required` + crates (P2-bis), audit P5 et ses compteurs, P6,
   triage P7 (précédent cité `post_restore.rs:578-581` et commentaires l. 1196-1204), squash de test. L'inventaire
   des lecteurs des sept champs devenus `Option` est-il clos (cherche au-delà de la liste : `kesh-seed`, tests,
   `exports/global.rs`, sauvegarde, frontend) ?
4. **Manuel contre code** (obligatoire) : `docs/manual/fr/user-manual.tex` § *Importer des factures depuis un dossier*
   (l. 1712-1741), § saisie d'une facture fournisseur (`sec:saisie-facture-fournisseur`, l. 1568), contrôle K5
   (l. 801, 808) ; `docs/manual/fr/admin-manual.tex` (l. 793, 1456, 1599 : sauvegarde de `KESH_DOCUMENTS_DIR`). Ce que
   le manuel dit **aujourd'hui** est-il vrai au code ? Ce que les AC9, AC17 et AC23 prescrivent d'écrire sera-t-il vrai
   au code que la fiche prescrit (dont les angles morts : pas de détachement, #503, limite des propositions) ?
   Contrôle aussi le **PDF**, aplati vers la **sortie standard seulement** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` — jamais vers un fichier.
5. **Recomptes** : 23 AC, T-a0 à T-a9, T-b1 à T-b7, T-c1 à T-c6, M-a1 à M-a10, M-b1 à M-b8, M-c1 à M-c8, huit parcours
   E2E, C-30-2-1 à 19, chaque numéro de ligne cité — recompte depuis la source.

## Lentilles

- **R — Relecture contre le code au sol (auditeur d'acceptation)** : `.claude/skills/bmad-create-story/checklist.md` ;
  chaque point du parcours de #685 couvert par un AC testable, chaque AC par une tâche et par un test **qui
  rougirait** ; chaque mutation tuée par le test nommé (une mutation équivalente sans le dire est un finding — M-c1 et
  M-c7 en particulier) ; chaque fait, chemin et numéro de ligne cité revérifié. Les E2E de parcours portent la recette
  (Guy ne testera qu'en fin d'epic) : couvrent-ils les six demandes de l'orchestrateur — document avec QR → facture
  pré-remplie ; sans QR → saisie à la main, document visible ; suppression d'un non-facture ; facture sans document ;
  document joint après coup ; document consultable depuis la facture et l'écriture, sorti de la liste et du dossier ?
  Le PDF en Chromium sans tête est-il traité honnêtement ? Les textes prescrits (i18n ×4, retrait de la clé
  `no-qr-code-found`, K5, manuel, CHANGELOG, `api-external.md`) confrontés à ce que le code fera ; les gardes
  existantes qui changeront de sens ou rougiront (`inbox-import.spec.ts`, `supplier-invoices.spec.ts`,
  `supplier-invoice-scan.spec.ts`, `FiscalYearChecks.test.ts`, `i18n-keys.test.ts`, `lint-i18n-ownership`,
  `boutons-rouges-inventory.test.ts` — qui recense un bouton de `supplier-invoices/import/+page.svelte` —,
  `audit_route_registry.rs`, `audit_label_registry.rs`, `textes_coherents.rs`, `csv_tables.rs`, tests de
  `closing_checks`) — la fiche les nomme-t-elle toutes ? Règle « un appariement propose, il ne crée jamais » : un
  chemin quelconque peut-il rattacher un fournisseur ou un document sans geste humain explicite ? Écrans : rôles,
  états vide / erreur / chargement / 410, largeur étroite, clair et sombre.
- **F — Faisabilité, sécurité, verrous et naissance de la vue (adversaire de la conception)** :
  - **Sécurité** : la CSP élargie à `blob:` (`img-src`, `frame-src`) ouvre-t-elle une surface (contenu actif d'un PDF
    dans une `<iframe>` de même origine, image piégée, `X-Content-Type-Options` absent) ? Le type MIME servi vient-il
    toujours de l'extension de la liste blanche ? La lecture ouverte à tout rôle (C-30-2-14) reste-t-elle bornée à la
    société ? Le rattachement (AC19) est-il protégé contre l'IDOR des deux côtés (staging et facture) ? Les bornes de
    l'import (taille, pages, liens symboliques, traversée) restent-elles intactes quand un document sans QR est
    accepté — en particulier le chemin `PDF_RENDER_ERROR`, qui couvre aussi « pdfium absent » et « cap de pages » ?
  - **Verrous et naissance de la vue** (axe B5 de la rétrospective 28) : relis `complete_import_once`
    (`crates/kesh-api/src/routes/imported_supplier_invoices.rs`), `discard_import`, la réactivation de
    `inbox_import.rs`, `supplier_invoices::pay_in_tx` et les annulations de facture et de règlement. L'ordre staging
    puis facture de l'AC19 forme-t-il un cycle avec un chemin existant ? L'étape 3 (lecture verrouillante d'un
    document déjà lié) est-elle réalisable sans verrou d'intervalle inattendu sur l'index d'unicité de
    `supplier_invoice_id` (et ce verrou peut-il bloquer une complétion concurrente) ? La complétion étendue (corps de
    création, C-30-2-10) change-t-elle l'ordre ou la nature des lectures dans sa transaction ?
  - **Faisabilité** : le contrôle de longueur avant `INSERT` (AC2) est-il juste en caractères et non en octets
    (`VARCHAR` en `utf8mb4`) ? L'extraction de `SupplierInvoiceForm.svelte` laisse-t-elle le formulaire de la liste
    strictement inchangé dans son comportement (scan 12.4, routage QR-IBAN, projet) ? Les propositions de fournisseur
    (AC14) sont-elles réalisables avec `listContacts` tel qu'il est (`search` + `isSupplier`) ? Le pré-remplissage
    d'adresse depuis un QR de type `S` correspond-il aux champs de `StructuredAddressInput`
    (`crates/kesh-api/src/address_input.rs`) ? La route neuve contre `docs/api-external.md` (clés d'API, liste des
    routes rejouées l. 788) et contre le registre des routes.

## Ce que tu rends

Findings numérotés (`R1-…` ou `F1-…`) avec sévérité **CRITICAL / HIGH / MEDIUM / LOW**, `fichier:ligne`, **preuve**
(commande exécutée et sortie copiée), correction proposée. Toute affirmation de présence ou d'absence = sortie de
`grep -nF` copiée. ⛔ **La liste des axes exercés ET non exercés** — un « 0 finding » sans elle ne compte pas comme
passe. Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/302-validate-p1-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/302-validate-p1-F.md` (lentille F) ; dernier message : le chemin du rapport, le
bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/`. Lecture seule stricte :
**aucun `git checkout`, `git switch`, `git checkout --detach` ni aucune autre forme** ; aucune commande qui écrit,
compile ou exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**, `scripts/test-fast.sh`,
`scripts/mem-guard.sh`, `scripts/regen-test-schema.sh`), `make`, `latexmk`, `xelatex`,
`git commit`/`add`/`stash`/`fetch`/`pull`/`reset`/`rebase`/`tag`/`push`, `cargo`, `npm`, `npx`, `node`,
**`python3`, `python`, `perl`, `ruby` et tout autre interpréteur**, `docker`, **`mariadb`, `mysql` et toute requête
SQL**, `bash` sur un script du dépôt, `gh` en écriture. `pdftotext` vers la **sortie standard** seulement, jamais
vers un fichier. Aucune donnée réelle de Guy ne se lit ni ne se cite (rien de `~/kesh-test/`). Autorisés : lecture,
`grep`, `sed -n`, `awk` en lecture, `git log`/`show`/`diff`/`blame`, `gh api` en lecture, `pdftotext … -`.
