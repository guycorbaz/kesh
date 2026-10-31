# Prompt — validation P4 de la fiche v017-4 (#849 : factures fournisseur brouillon → validée → payée), deux lentilles, Opus

Tu valides la **fiche** de la Story v017-4 de Kesh (comptabilité suisse, Rust/Axum, SvelteKit, MariaDB 10.11) et ses
deux sous-fiches. **#849 (P2)** : une facture fournisseur passe aujourd'hui son écriture d'achat dès sa création ; une
faute de saisie ne se corrige qu'en l'annulant (contre-passation) et en la ressaisissant. La fiche retient : un statut
`draft` sans écriture (le statut stocké `open` garde le sens « validée, à payer »), `PUT` et `DELETE` du brouillon,
`validate` et `unvalidate` — la dévalidation **supprime** l'écriture d'achat, règle de la facture client —, une
migration breaking `20261031000001_supplier_invoices_draft` (écriture d'achat nullable, `min_required` reposé à
`0.17.0`), un contrôle de clôture K8, et quatre défauts du même formulaire ajoutés par le Project Lead (#853, #830, #835,
#831). Les questions Q1 à Q5 sont tranchées (C-v017-4-21 à 25) : les factures annulées sont **masquées** dans la liste, leur
suppression réelle n'est pas dans la 0.17.0 ; candidate `v0.17.0-rc2` ; l'état validé (valeur stockée `open`)
s'affiche **« Validée »** aux quatre locales ; #853 se ferme avec la release. Des livres réels sont
tenus sur la `v0.17.0-rc1` : montée sans perte et sauvegardes antérieures importables sont des exigences.

Worktree `/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon`, partie de `8e126cfd`.
Langue du rapport : **français**. Modèle : **Opus**. Passe **complète** (quatrième passe de validation : la remédiation de la P3 change des règles d'exécution — rejeu, décodage des lots). **Contexte
frais** pour chaque lentille.

La validation P3 (Sonnet) a rendu 0 CRITICAL, 0 HIGH, 6 MEDIUM, 14 LOW (rapports
`/home/gcorbaz/devel/kesh-gate-logs/v0174-validate-p3-R.md` et `…-F.md`), tous remédiés (Change Log de la parente, entrée
« Validation P3 » ; registre C-v017-4-37 à 40, et l'annotation datée de C-v017-4-28). Elle a rendu le rejeu à la
création, au `PUT` et au `DELETE`, rendu tolérant le décodage d'une facture dans la création d'un lot, et piloté les
tests de course.

## À lire

1. La fiche parente `_bmad-output/implementation-artifacts/v017-4-facture-fournisseur-brouillon.md`, **en entier**, et
   les sous-fiches `v017-4a-facture-fournisseur-brouillon-backend.md` et `v017-4b-facture-fournisseur-brouillon-ecran.md`.
2. Le registre `_bmad-output/implementation-artifacts/v017-4-choix-autonomes.md` (C-v017-4-1 à 40), et les rapports
   des P1 à P3 (`kesh-gate-logs/v0174-validate-p{1,2,3}-{R,F}.md`, lecture seule).
3. Les issues, **en lecture seule** : `gh issue view <N> --json title,body,comments` pour 849, 853, 830, 835, 831, 687.
4. La contrainte du Project Lead : `/home/gcorbaz/devel/kesh-gate-logs/prompts/v017-donnees-a-garder.md`.
5. Le code : `crates/kesh-db/src/repositories/supplier_invoices.rs` (en entier), `invoices.rs` (`update`, `delete`,
   `unvalidate`, `validate_invoice`), `journal_entries.rs` (`delete_in_tx`, `modification_guard`, `document_owners`),
   `imported_supplier_invoices.rs`, `payment_batches.rs`, `reconciliation.rs`, `letterings.rs`, `contacts.rs`,
   `closing_checks.rs`, `post_restore.rs` ; `crates/kesh-db/migrations/20260628000001_supplier_invoices.sql`,
   `20260628000002_payment_batches.sql`, `20260629000001_imported_supplier_invoices.sql`,
   `20261030000001_bank_imports_manual_source.sql` ; `crates/kesh-db/tests/migrations_upgrade_path.rs`,
   `test_schema_guard.rs` ; `crates/kesh-api/src/lib.rs` (routes), `routes/supplier_invoices.rs`,
   `routes/imported_supplier_invoices.rs`, `routes/invoices.rs`, `errors.rs`, `audit_labels.rs`,
   `mcp/outils/factures_fournisseur.rs`, `mcp/outils/commun.rs`, `exports/csv_tables.rs` ; `crates/kesh-api/tests/`
   (`audit_route_registry.rs`, `admin_full_import_e2e.rs`, `inbox_import_e2e.rs`, `closing_checks_e2e.rs`) ;
   `crates/kesh-seed/src/pieges.rs` ; `frontend/src/lib/features/supplier-invoices/`,
   `frontend/src/routes/(app)/supplier-invoices/`, `frontend/src/routes/(app)/payment-batches/+page.svelte`,
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (patron de la dévalidation),
   `frontend/src/lib/features/journal-entries/balance.ts`, `frontend/src/lib/features/bank-import/manual-statement.ts`,
   `frontend/src/lib/features/invoices/invoice-helpers.ts`, `frontend/src/lib/features/fiscal-years/` ;
   `frontend/tests/e2e/` (specs qui créent des factures fournisseur).
6. Le **manuel** : `docs/manual/fr/user-manual.tex`, § `sec:factures-fournisseurs` (l. 1565-1840 environ),
   § *Numérotation* (l. 740-755), `sec:matching-fournisseur` (l. 2480), `sec:import-factures` ; `admin-manual.tex`
   § *Passer à la 0.17.0* (l. 2486) ; les PDF aplatis par `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' |
   tr -s ' '` (idem `admin-manual.pdf`), **vers la sortie standard seulement**. Et `docs/api-external.md`.
7. `CLAUDE.md` du worktree (*Migration breaking policy* P1–P8, *Règle de splitting*, *Pattern batch*) et
   `docs/agents-orchestration.md`.

## Axe obligatoire, pour les deux lentilles : la remédiation de la P3

Pour chaque finding de la P3 (R1 à R11, F1 à F9) : le remède est-il dans la fiche, **exact au code** (relis la ligne
qu'il cite), et n'a-t-il pas introduit un défaut neuf ? En particulier :
- **le rejeu** (§ *Verrous et rejeu*, C-v017-4-37) : `retry_on_deadlock` sur la création, le `PUT` et le `DELETE` est-il
  sûr (aucune écriture hors de la transaction rejouée — audit, fichier, détachement du document) ? Le classement du
  registre `audit_route_registry.rs` et ses décomptes (AC-18) suivent-ils ? Le cycle décrit (comptes croisés, sentinelle
  `bank_accounts.rs:1617`) est-il exact ?
- **les lots** (`payment_batches.rs:376-410`, C-v017-4-38) : le décodage tolérant et l'ordre « statut puis écriture »
  laissent-ils un autre chemin du même fichier décoder l'écriture d'achat en `i64` (`grep -n purchase_journal_entry_id`) ?
  Le refus sort-il bien en `failed[]` ?
- **les tests pilotés** (AC-9, AC-10, M20, M20-quater) : la chorégraphie est-elle réalisable avec
  `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs`) et les étapes publiques de `attach_in_tx` ;
  tue-t-elle le mutant à coup sûr, et l'ordre prescrit passe-t-il à coup sûr ?
- **la migration** (C-v017-4-39) : les trois essais du repli, le nom effectif de la clé propagé au squash et à P8 ;
- **les textes** (AC-19 `:837-839`, AC-20 `:2979` et CHANGELOG, AC-25 `case`, L8 au manuel) : exacts et complets ?

## Deux lentilles, en contexte frais chacune

### Lentille R — la règle, les données et leur preuve

- **Les prémisses** : chaque `fichier:ligne` cité par la fiche et le registre dit-il ce qu'on lui fait dire ? En
  particulier la règle client (suppression, non contre-passation), `min_required` déjà à 0.17.0, les filtres
  `status = 'open'`, les clés étrangères vers `supplier_invoices`.
- **La migration (AC-1)** : P1 à P8 tenus ? Le `MODIFY COLUMN` d'une colonne portée par une clé étrangère est-il admis
  par MariaDB 10.11, et la fiche prévoit-elle le repli ? Les `CHECK` neufs passent-ils sur **toute** ligne existante
  possible (factures payées puis annulées, règlement détaché, `paid_at`) ? Les compteurs et lignes cités
  (`migrations_upgrade_path.rs`, `post_restore.rs`, `migrations-idempotence-audit.md`) sont-ils les bons — recompte-les.
  Le squash, `migrations.sha384`, `test_schema_guard` ?
- **Les données à garder (AC-2, AC-3)** : la montée depuis la 0.16.x **et** depuis la rc1 est-elle prouvée pour tout
  ce que porte la base de Guy (factures ouvertes, payées, annulées, payées puis annulées, documents, lots, lettrage) ?
  Une sauvegarde 0.16.x ou rc1 s'importe-t-elle dans la rc2 (contraintes vérifiées à l'insertion) ? La limite rc2 → rc1
  (L1) est-elle exacte au code de l'import (`admin_backup/import.rs`) ?
- **Le cycle (AC-6 à AC-13)** : les contrôles répartis entre l'enregistrement et la validation laissent-ils un trou
  (une donnée acceptée au brouillon, jamais rejugée) ? La validation passe-t-elle **exactement** l'écriture d'aujourd'hui ?
  La dévalidation : précédence, verrous, lecture verrouillante, `delete_in_tx` sans garde de propriété — un chemin
  d'écriture (rapprochement, lettrage, lot, règlement détaché, document) est-il oublié ? Un état atteignable rend-il la
  dévalidation destructrice en silence ? La suppression d'un brouillon : clés étrangères, document, historique de lot ?
- **L'inventaire clos (AC-4, AC-5)** : cherche **tout** lecteur de `supplier_invoices.status` et de
  `purchase_journal_entry_id` (production, seed, frontend) ; un site absent de la table est un finding.
- **#853 au serveur (AC-14)**, filtre de liste (AC-15), K8 (AC-16), MCP (AC-17), audit et registres (AC-18).
- **Les mutations M1 à M20** : chacune rougit-elle le test annoncé ?
- **Le découpage** : chaque AC dans une seule sous-fiche ; la 4b ne dépend que de ce que la 4a livre.

### Lentille F — l'écran, les textes et ce qu'ils affirment

- **Le manuel (AC-20, AC-29)** — axe obligatoire, au `.tex` **et** au PDF aplati : chaque phrase devenue fausse
  est-elle nommée (« à l'enregistrement, Kesh passe l'écriture », « ouverte : annulez-la puis ressaisissez », « deux
  chemins creusent la numérotation », factures « ouvertes » proposées, complétion d'un document) ? Une phrase manque-t-elle
  à l'inventaire de l'AC-29 ?
- **L'écran (AC-21 à AC-27)** : actions par statut et par rôle cohérentes avec les rôles serveur (Comptable,
  Administrateur, clés) ; la page d'édition et le mode « document avec QR » ; la date vide avec un document ; les
  messages des refus ; le libellé « Validée » (C-v017-4-24) : l'inventaire de l'AC-21 et de l'AC-29 trouve-t-il **toutes**
  les occurrences où l'écran, les catalogues, les manuels (PDF aplati), `api-external.md` ou le CHANGELOG nomment encore cet
  état « ouverte » / « offen » / « aperta » / « open » ? Une occurrence au sens du poste ouvert d'un compte reste-t-elle
  bien distinguée ?
- **Les quatre défauts ajoutés (C-v017-4-20)** : les prémisses de #853, #830, #835, #831 sont-elles justes au code ? Le
  remède de #830 (parser une fois) couvre-t-il **tous** les champs numériques du formulaire (quantité, prix, taux,
  montant attendu) et le total vivant ? Un banc de la page de liste (#831) est-il bien prescrit ?
- **i18n (AC-18, AC-28)** : clés aux quatre catalogues, replis égaux au fr-CH, glossaire.
- **`docs/api-external.md` (AC-19), CHANGELOG (AC-20, AC-32), README** : le changement de contrat est-il dit, avec la
  conduite d'une intégration ? « sept contrôles », « une migration », « six stories » relevés ?
- **E2E et captures (AC-30, AC-31)** : le cycle complet jusqu'au paiement par réconciliation est-il jouable avec ce que
  Kesh offre (relevé saisi à la main par l'API) ? Les specs existantes à adapter sont-elles toutes listées ?
- **Les décisions sur Q1 à Q5** (C-v017-4-21 à 25) : sont-elles portées partout dans la fiche, sans contradiction avec
  le texte d'origine laissé tel quel ? La limite L6 (factures annulées gardées dans les livres) est-elle écrite au manuel ?
- **Les règles du dépôt** : aucun mot-clé de fermeture prescrit dans un message de commit ; aucune donnée réelle ;
  défaut voisin renvoyé à une issue s'il n'est pas mécanique.
- **Les décomptes et numéros de ligne** : recompte-les.

## Ce qu'il faut rendre

Un rapport en Markdown, écrit **par toi, une fois**, au chemin de ta lentille :
`/home/gcorbaz/devel/kesh-gate-logs/v0174-validate-p4-R.md` (lentille R) ou
`/home/gcorbaz/devel/kesh-gate-logs/v0174-validate-p4-F.md` (lentille F). Il contient :

1. **Les findings**, numérotés `R1`… ou `F1`…, chacun avec sa sévérité (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`), le lieu,
   **la ligne lue citée** (sortie `grep -nF` ou extrait copié) et le remède proposé. Une affirmation de présence ou
   d'absence sans sortie `grep -nF` copiée ne compte pas.
2. **Pour l'axe obligatoire**, un tableau finding P3 → remède vérifié (oui / non / régression), ligne lue citée.
3. **Les axes exercés** et **les axes non exercés**, un par un, avec ce que tu as lu ou cherché. Un « 0 finding » sans
   cette liste ne compte pas comme passe.
4. Un décompte final par sévérité, **recompté** depuis ta liste.

## Interdits — lecture seule stricte

- **Aucune écriture**, hors de ton seul rapport : ni `Edit`, ni `Write` sur un autre fichier, ni redirection (`>`, `>>`,
  `tee`) — pas même vers un fichier « jetable », ni dans le scratchpad. Un fichier créé par erreur se **déclare** au
  rapport, il ne se supprime pas.
- Aucun `git checkout`, `git switch`, `git reset`, `git stash`, `git commit`, `git push`, `git tag`, `git rebase`.
- Aucune commande `gh` qui écrit ; `gh issue view` seulement.
- Aucune commande qui **compile ou exécute** : ni `cargo`, ni `npm`, ni `make`, ni `latexmk`, ni `python3`, ni `node`,
  ni `sqlx`, ni `docker`, ni `curl`, ni `claude`.
- Jamais `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`,
  `scripts/regen-test-schema.sh`, ni aucun script du dépôt.
- Jamais `rm`, `pkill`, `killall`.
- Jamais `~/kesh-test/` ; aucune donnée réelle dans le rapport.
- Rien dans `/home/gcorbaz/devel/kesh` : lecture seule.
- Commandes permises : `cat`, `sed -n`, `grep`, `rg`, `ls`, `find`, `wc`, `head`, `tail`, `git log`, `git show`,
  `git diff` (en lecture), `pdftotext … -` vers la sortie standard, `gh issue view`.
