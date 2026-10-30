# Release 0.17.0 — registre des choix autonomes de la Story v016-2

*(Première rédaction : « Release 0.16.1 ». La release est devenue la 0.17.0 le 2026-10-30, C-v016-2-15.)*

Registre propre à la story (`docs/agents-orchestration.md` § 1), à verser **par union** à l'historique des correctifs au
merge. Fiche : `v016-2-releve-manuel.md`. Numérotation `C-v016-2-k`. Issue : **#816** (enhancement), en `refs`
seulement. Les numéros de ligne cités sont ceux de `c5101b07` (tête de `story/v016-1-mcp-sorties`, candidate
`v0.16.1-rc1`). Chaque choix : contexte, option retenue, alternatives écartées, réversibilité. Les choix marqués **(Q)**
figurent aussi parmi les questions à soumettre à Guy avant la validation (fiche, § *Questions au Project Lead*).

---

### C-v016-2-1 — Le modèle : un relevé saisi est un import de source `MANUAL`

- **Contexte** : #816 demande des transactions « rapprochables, comme un import » et une saisie « distinguable dans la
  piste de contrôle ». Le circuit bancaire entier lit `bank_imports` et `bank_transactions` : rapprochement
  (`kesh-reconciliation`, routes `reconciliation*.rs`), suppression d'un import (`bank_imports::delete_unreconciled_in_tx`),
  tuile du tableau de bord et contrôle K3 de la clôture (le relevé « à solde », `bank_accounts.rs:1752-1810`,
  `closing_checks.rs:455-470`), export global (`exports/csv_tables.rs:693`), sauvegarde (générique, table par table).
  `bank_imports.source_format` est un `VARCHAR(32)` **sans** contrainte de valeur, délibérément
  (`20260504000001_bank_imports.sql:16-17`, « un futur `'CSV_UBS_PROFILE_2026'` doit pouvoir être ajouté sans
  migration ALTER ») ; seule l'énumération Rust `BankImportSourceFormat` (`entities/bank_import.rs:26-31`, `from_str`
  `:53-60`) borne les valeurs.
- **Retenu** : un relevé saisi est une ligne de `bank_imports` de `source_format = 'MANUAL'` (variante `Manual` de
  `BankImportSourceFormat` et de `kesh_core::bank_imports::SourceFormatTag`), et ses lignes des `bank_transactions`
  ordinaires, créées par **la même** fonction `bank_imports::create_with_transactions` que l'import. Aucune table,
  aucune colonne. La route nouvelle n'écrit aucun SQL : la garde du compte clos/archivé (et sa revérification
  verrouillante), l'atomicité entête + lignes + audit, la suppression, le rapprochement et ses annulations sont
  hérités tels quels.
- **Écartées** :
  - (b) une table `manual_statements` et ses lignes : un second chemin de rapprochement, de suppression, de sauvegarde
    et de clôture — tout le circuit à doubler (DRY) ;
  - (c) générer un CSV côté client et le passer à l'import : exige un profil CSV, perd la distinction dans la piste
    (`source_format = 'CSV'`), et le CSV n'a pas de solde de clôture (la tuile et K3 ne verraient pas le relevé) ;
  - (d) une variante `SourceFormat::Manual` dans `kesh-import` : crate publiable (`cargo publish --dry-run -p
    kesh-import` en CI), dont l'API publique décrit des **fichiers** ; la saisie n'est pas un format de fichier.
- **Réversible** : oui tant qu'aucun relevé n'est saisi ; ensuite, la valeur `MANUAL` est dans les données (voir
  C-v016-2-2).

### C-v016-2-2 — Une migration qui relève `kesh_version_min_required` (Q)

*Amendé le 2026-10-30 par C-v016-2-15 : la valeur relevée est `0.17.0`, et non `0.16.1` ; les dix crates passent à
`0.17.0` dans le commit de la migration. Le texte ci-dessous est celui de la première rédaction (0.16.1).*

- **Contexte** : une ligne `source_format = 'MANUAL'` ne se décode pas dans un binaire 0.16.0 ou antérieur
  (`from_str`, `entities/bank_import.rs:53-60` : `other => Err(...)`). Sans migration, la 0.16.0 **redémarre** sur une
  base passée par la 0.16.1 (sqlx ne refuse que les migrations inconnues) et rend une erreur 500 sur la liste des imports,
  leur détail, leur suppression et l'export global dès qu'un relevé saisi existe. Et une sauvegarde `.keshbackup` de la
  0.16.1 s'importe dans la 0.16.0 (mêmes tables ; `kesh_version_min_required` de la base vaut `0.14.0`,
  `20261015000001_imported_documents_without_qr.sql:54`), avec le même effet. C'est exactement le cas que
  `kesh_version_min_required` existe pour refuser (`CLAUDE.md` § *Migration breaking policy*, P1 : « un état qu'un
  binaire antérieur ne peut plus consommer correctement » — ici un état des **données**, non du schéma).
- **Retenu** : une migration `20261030000001_bank_imports_manual_source.sql` dont la **seule** instruction est
  `UPDATE _kesh_version SET kesh_version_min_required = '0.16.1' WHERE id = 1;` (P2 : en dernière instruction, version
  figée). Les dix crates sont déjà en `0.16.1` (`c5101b07`, `dd8be435`) : P2-bis tenu sans bump. Effet : la 0.16.0
  refuse de démarrer sur la base (sqlx **et** `check_downgrade_protection`) et l'import d'installation d'une sauvegarde
  0.16.1 dans la 0.16.0 est refusé (`VERSION_INCOMPATIBLE`, `routes/admin.rs:217-223`). P5 (ligne d'audit), P6
  (couplages positionnels), P7 (triage : `EXEMPT_MIGRATIONS`, justification « UPDATE sur _kesh_version », comme
  `20261009000001`), squash `test-schema` régénéré, P8 (le fichier ne se modifie plus une fois appliqué).
- **Écartées** :
  - (b) **aucune migration** : retour à la 0.16.0 « sans sauvegarde », comme le CHANGELOG de la rc1 le promet — mais
    une 0.16.0 qui démarre et casse en 500 sur les écrans d'import dès le premier relevé saisi, et une sauvegarde qui
    s'importe là où elle ne se lit pas ; un défaut muet contre un refus net ;
  - (c) une contrainte `CHECK (source_format IN (…))` : contredit le choix d'extensibilité de la 8-1b, n'ajoute rien
    à l'énumération Rust, et ne protège pas la sauvegarde (le `CHECK` ne change pas les tables) ;
  - (d) une colonne `origin` : redondante avec `source_format`, et la valeur nouvelle resterait illisible des binaires
    antérieurs si l'on ne relève pas `min_required`.
- **Coût assumé** : la phrase du CHANGELOG `[0.16.1]` (« Il n'ajoute **aucune migration** : la 0.16.0 redémarre sur une
  base passée par la 0.16.1, et une sauvegarde de l'une s'importe dans l'autre ») et le § *Passer à la 0.16.1* du
  manuel d'administration deviennent faux et se récrivent à la release (fiche, § *Ce que la release doit contenir*).
  ⚠️ La rc1 porte déjà la version Cargo `0.16.1` : une sauvegarde de la rc2 s'importe dans la rc1 (même version), où
  un relevé saisi ne se lit pas — la rc1 n'est qu'une candidate, installée sur une base neuve ; à écrire dans les notes
  de la rc2. *(Levé par C-v016-2-15 : avec `0.17.0`, la rc1, de version Cargo `0.16.1`, refuse base et sauvegardes.)*
- **Réversible** : non une fois publiée (P8) ; avant publication, oui.

### C-v016-2-3 — Le relevé : ce qu'on saisit

- **Contexte** : une transaction importée porte `booking_date` (requis), `value_date`, `amount` signé
  `DECIMAL(18,2)` (positif = crédit du titulaire, `20260504000001_bank_imports.sql:7`, `:62`), `currency`,
  `reference` (`VARCHAR(255)`), `details` (`TEXT NOT NULL`), `counterparty_name`, `counterparty_iban`,
  `end_to_end_id`, `transaction_id`. Le rapprochement note la référence à 40 % et la contrepartie à 10 %
  (`kesh-reconciliation/src/matching.rs:183`) ; les règles d'affectation lisent la contrepartie (`rules.rs:49-50`).
  L'entête porte `period_from`, `period_to`, `opening_balance`, `closing_balance` (`DECIMAL(18,2)`), `statement_id`.
- **Retenu** : entête — compte bancaire, période (du, au), numéro du relevé (facultatif → `statement_id`), solde de
  départ, solde d'arrivée (requis tous deux, signés : un découvert est négatif). Lignes — date (comptabilisation,
  requise), date de valeur (facultative), libellé (requis, ≤ 500 caractères après `trim`), contrepartie (facultative,
  ≤ 255), référence (facultative, ≤ 255), **montant positif et sens** (`credit` = entrée, `debit` = sortie) ; à
  l'écran, deux colonnes *Débit* et *Crédit*, comme sur le papier. Montants et soldes : au plus deux décimales, au plus
  treize chiffres entiers ; montant d'une ligne strictement positif. Devise : CHF, seule devise de cette version
  (comme l'import). De 0 à 500 lignes (zéro : un mois sans mouvement, dont le solde reste utile à la tuile et à K3).
  Pas d'IBAN de contrepartie, pas d'identifiants de bout en bout.
- **Écartées** : un montant signé saisi directement (erreur de signe plus probable qu'un choix de colonne) ; la date de
  valeur obligatoire (souvent absente des relevés papier ; le rapprochement retombe sur `booking_date`,
  `split.rs:72`) ; un plafond de lignes plus haut (un relevé papier mensuel en compte rarement plus de cent ; 500 garde
  le corps JSON sous la limite par défaut d'Axum).
- **Réversible** : oui.

### C-v016-2-4 — L'équilibre : exact au centime, bloquant, sans confirmation

- **Contexte** : l'import tolère un écart d'**un centime** (`validate_balance`, `kesh-core/src/bank_imports.rs:254-271`,
  `diff > Decimal::new(1, 2)`) et l'admet sur confirmation (`confirmBalanceMismatch`). Pour un fichier, l'écart vient de
  la banque ; pour une saisie, l'utilisateur tape soldes et lignes depuis le même document : un écart est une faute
  de frappe.
- **Retenu** : `solde de départ + Σ crédits − Σ débits = solde d'arrivée`, **égalité exacte** (montants à deux
  décimales : pas d'arrondi en jeu), refus `422 BANK_STATEMENT_UNBALANCED` sans confirmation possible ; refusé aussi
  à l'aperçu. Chaque date de comptabilisation dans `[du, au]` (`400`). Fonction pure **nouvelle** de `kesh-core` ;
  `validate_balance` reste celle de l'import.
- **Écartées** : réutiliser `validate_balance` (sa tolérance d'un centime laisserait passer une faute de frappe de
  0.01) ; une confirmation « malgré l'écart » (enregistrerait un relevé faux en connaissance de cause, que la tuile et
  K3 compareraient ensuite aux livres).
- **Réversible** : oui.

### C-v016-2-5 — Le solde de départ attendu : signalé, à confirmer, jamais bloqué (Q)

- **Contexte** : le relevé précédent d'un même compte se définit comme le fait déjà la tuile : le plus tardif **à
  solde** (`closing_balance IS NOT NULL`), ordre `period_to DESC, imported_at DESC, id DESC`
  (`bank_accounts.rs:1802`), ici borné à `period_to < du`. Un écart peut être légitime (premier relevé saisi après des
  imports CSV sans solde ; relevé précédent faux, à corriger ensuite) ou une faute de frappe.
- **Retenu** : l'aperçu signale `openingBalanceGap { expected, entered, previousImportId, previousPeriodTo }` ; la
  création le recalcule et refuse en `422 BANK_STATEMENT_OPENING_BALANCE_GAP` sans `confirmOpeningBalanceGap: true`,
  puis l'enregistre avec le modificateur `opening_balance_gap` (et `expected_opening_balance`) dans la trace. Sans
  relevé précédent à solde : rien. Lecture **sans verrou** par une fonction de dépôt nouvelle (un `SELECT`, aucune
  écriture).
- **Écartées** : bloquer (empêche la saisie d'un relevé tant qu'un précédent est faux, et tout relevé qui suit un import
  CSV) ; signaler sans confirmation (une API passerait outre sans l'avoir vu) ; préremplir le solde de départ (le
  solde est imprimé sur le papier : le préremplir inviterait à ne pas le relire).
- **Réversible** : oui.

### C-v016-2-6 — Doublons et chevauchements : signaler, ne jamais fusionner

*Annoté le 2026-10-30, validation P2 (R4 = F1) : l'annotation P1 ci-dessous est **fausse pour le CAMT**. La clé d'une
ligne prend le **premier identifiant non vide** parmi référence, identifiant de bout en bout et identifiant de
transaction (`kesh-core/src/bank_imports.rs:365-377`) ; une ligne CAMT en porte presque toujours un
(`kesh-import/src/camt053/mod.rs:412`, `:629-630`). Une ligne saisie sans référence y entre donc presque toujours deux
fois ; le cas « absente des deux côtés » ne vaut que pour un CSV sans identifiant. La première rédaction ci-dessous
(« une ligne saisie sans référence n'a pas la même empreinte que la même ligne du fichier, qui en porte une ») disait
juste.*

*Annoté le 2026-10-30, validation P1 (R9) : « saisi d'abord, importé ensuite » compte **trois** cas, non un — référence
égale des deux côtés ou absente des deux côtés : la clé est la même, le doublon est reconnu et **ignoré par défaut** à
l'import ; références différentes ou une seule présente : non reconnu, le mouvement entre deux fois. La fiche (AC-27,
§ *Ce que les gardes ne voient pas*) porte la version corrigée.*

- **Contexte** : l'empreinte d'une ligne est `(booking_date, amount, première référence non vide parmi reference →
  end_to_end_id → transaction_id, compte)` (`dedup_key_scalar`, `kesh-core/src/bank_imports.rs`), comparée aux
  transactions de la fenêtre `[du, au]` du compte (`find_in_dedup_window`, `bank_transactions.rs:83-112`). À
  l'import, les doublons sont **ignorés par défaut** (`ConfirmDuplicateLines::Skip`, `routes/bank_imports.rs:389-397`).
  L'empreinte du fichier (`file_hash`, SHA-256) détecte un fichier réimporté (`find_by_company_and_hash`).
- **Retenu** :
  - **lignes** : la même empreinte, la même fenêtre ; mais **pas d'option « ignorer »** — retirer une ligne d'un relevé
    équilibré le déséquilibrerait. Sans `confirmDuplicateLines: true`, `422 BANK_STATEMENT_DUPLICATE_LINES` avec les
    lignes en cause (rang 1-based, transaction existante) ; avec, tout est enregistré, modificateur
    `duplicate_lines_imported` ;
  - **relevé entier** : `file_hash` = SHA-256 d'une sérialisation canonique du contenu (préfixe de domaine
    `kesh-manual-statement-v1`, compte, période, numéro, soldes, lignes dans l'ordre), si bien qu'un relevé saisi deux
    fois à l'identique est refusé `422 BANK_IMPORT_DUPLICATE_FILE` sans `confirmDuplicateFile: true` (code et
    sémantique de l'import) ;
  - **période qui chevauche** un relevé existant du compte : information de l'aperçu (`overlappingStatements`), sans
    confirmation — ce sont les lignes qui portent le risque, et elles sont déjà contrôlées ;
  - **saisi d'abord, importé ensuite** (la banque se met au CAMT) : à l'import du fichier, une ligne saisie **sans
    référence** n'a pas la même empreinte que la même ligne du fichier, qui en porte une — elle n'est **pas** reconnue
    comme doublon, et le mouvement entre deux fois, sans avertissement. Aucune fusion automatique (règle
    *Un appariement automatique propose, il ne crée jamais*). La conduite s'écrit au manuel : supprimer le relevé
    saisi avant d'importer la même période (possible tant qu'il n'est pas rapproché), ou importer à partir du lendemain.
    Un avertissement à l'import (« cette période chevauche un relevé saisi à la main ») est **demandé en issue**, hors
    périmètre (il touche l'aperçu CAMT et CSV).
- **Écartées** : « ignorer » pour la saisie (casse l'équilibre) ; rapprocher automatiquement la ligne saisie de la
  ligne importée (faux rattachement muet) ; `file_hash` aléatoire (perdrait la détection du relevé saisi deux fois).
- **Réversible** : oui.

### C-v016-2-7 — Corriger : supprimer le relevé entier, puis le ressaisir (Q)

- **Contexte** : `DELETE /api/v1/bank-imports/{id}` (Story 27-1) supprime un import et toutes ses lignes tant
  qu'**aucune** n'est rapprochée (`routes/bank_imports.rs:1484-1576`, `409 BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`
  sinon), trace `bank_import.deleted` avec `source_format`, et est fermée à toute clé d'API. Après rapprochement,
  l'annulation du rapprochement (#418, fermée) remet la transaction à l'état non rapproché et contre-passe l'écriture.
  L'équilibre exact (C-v016-2-4) arrête toute faute sur un montant ; restent les fautes de date, de libellé, de
  contrepartie ou de référence, et une erreur de sens compensée par une autre.
- **Retenu** : le minimum — **aucune** correction de ligne. Une faute se corrige en supprimant le relevé entier par la
  route existante (inchangée), puis en le ressaisissant ; après un rapprochement, en l'annulant d'abord. Le formulaire
  reste entièrement modifiable **avant** l'enregistrement, aperçu compris. Aucune écriture comptable fausse n'est
  possible : une transaction bancaire n'écrit rien aux livres tant qu'elle n'est pas rapprochée.
- **Écartées** :
  - (b) modifier une ligne (`PATCH`) : un nouvel écrivain de `bank_transactions` (inventaire
    `bank_account_closed_inventory.rs`, garde du compte clos, piste), et une ligne modifiée après rapprochement
    désynchroniserait l'écriture ;
  - (c) « Corriger » = formulaire prérempli depuis le relevé, enregistré en remplacement atomique
    (`replacesImportId` : suppression et création dans une transaction) — confortable pour un relevé long ; laissé à
    l'arbitrage de Guy.
- **Réversible** : oui (C-v016-2-7 (c) s'ajoute sans rien défaire).

### C-v016-2-8 — L'écran : `/bank-import/manual`, aperçu puis enregistrement

- **Contexte** : `/bank-import` porte le dépôt d'un fichier (`BankImportUpload.svelte`) et la liste des imports
  (`BankImportList.svelte`, colonne *Fichier* = `filename`). L'équilibre d'une écriture se calcule déjà côté client par
  `big.js` (`journal-entries/balance.ts`, `AMOUNT_RE` à quatre décimales).
- **Retenu** : sur `/bank-import`, à côté du dépôt, un lien **« Saisir un relevé à la main »** (Comptable+) vers une page
  `/bank-import/manual`. Formulaire : compte (comptes ouverts non archivés de `GET /companies/current`, comme le dépôt),
  période, numéro, soldes, puis une table de lignes au clavier — *Ajouter une ligne*, retrait d'une ligne, ordre de
  tabulation naturel ; sous la table, en continu : total des débits, total des crédits, solde calculé, solde d'arrivée
  saisi, **écart** (au centime, en `big.js`). *Vérifier* appelle l'aperçu (avertissements du serveur), *Enregistrer*
  n'est actif que si l'écart est nul et toutes les lignes valides, puis montre les cases de confirmation requises
  (doublons, relevé identique, solde de départ). Succès : retour à `/bank-import`, le relevé en tête de liste. Dans la
  liste et au détail, un relevé `MANUAL` s'affiche **« Saisie manuelle »** (traduit) au lieu de son `filename`, et la
  boîte de suppression en parle comme d'un relevé saisi. La validation du format des montants **généralise**
  `isValidAmount` (paramètre du nombre de décimales) au lieu d'en écrire une seconde (DRY).
- **Écartées** : un onglet dans `BankImportUpload` (composant déjà chargé de trois flux) ; un enregistrement sans aperçu
  (les doublons et l'écart de solde de départ ne se savent qu'au serveur) ; la saisie dans une modale (un relevé de
  cinquante lignes y est à l'étroit).
- **Réversible** : oui.

### C-v016-2-9 — Rôles et clés d'API : ceux de l'import

- **Contexte** : `POST /bank-imports/preview` et `POST /bank-imports` sont montés aux routes Comptable+
  (`lib.rs:792-803`) et ouverts aux clés `read-write` (`docs/api-external.md:223`) ; seule la suppression est fermée aux
  clés (`routes/bank_imports.rs:1490-1493`).
- **Retenu** : les deux routes nouvelles au même endroit, mêmes rôles (Comptable, Administrateur), ouvertes aux clés
  `read-write`, fermées aux clés `read` (middleware). Lecture (liste, détail) inchangée, tout rôle. La suppression d'un
  relevé saisi reste fermée aux clés.
- **Écartées** : fermer la saisie aux clés (asymétrie avec l'import sans motif : une clé qui importe un fichier peut déjà
  créer des transactions).
- **Réversible** : oui.

### C-v016-2-10 — La piste de contrôle : une action propre, `bank_import.entered_manually`

- **Contexte** : l'import écrit **une** trace canonique `bank_import.created` avec `details.source_format` et
  `details.modifiers` (`insert_canonical_audit_log`, `routes/bank_imports.rs:1698-1750`). Le journal d'audit se filtre
  par action ; ses libellés viennent de `audit_labels.rs` (`ACTIONS`, `:92-93`), gardés par
  `tests/audit_label_registry.rs` (diff bilatéral, quatre catalogues, inventaire des sites indirects).
- **Retenu** : une action distincte **`bank_import.entered_manually`** (« Relevé bancaire saisi à la main »), même type
  d'entité `bank_imports`, mêmes détails que l'import (`filename`, `transaction_count`, `source_format: "MANUAL"`,
  `modifiers` triés) plus `period_from`, `period_to`, `opening_balance`, `closing_balance`, `statement_id`, et les
  valeurs liées aux modificateurs. `insert_canonical_audit_log` reçoit l'action en paramètre (DRY) — le site devient
  **indirect** et s'inscrit à `SITES_INDIRECTS` avec ses deux codes. La suppression garde `bank_import.deleted`
  (ses détails portent déjà `source_format`).
- **Écartées** : `bank_import.created` avec `source_format: "MANUAL"` seulement (distinguable au seul dépliage des
  détails, non au filtre) ; une seconde fonction d'audit recopiée (DRY).
- **Réversible** : non une fois des traces écrites (le code reste dans `audit_log`) ; avant publication, oui.

### C-v016-2-11 — `filename` et empreinte d'un relevé saisi

- **Contexte** : `bank_imports.filename VARCHAR(255) NOT NULL`, `file_hash CHAR(64) NOT NULL` avec
  `chk_bank_imports_hash_len` (`20260504000001_bank_imports.sql:23-24`, `:51`).
- **Retenu** : `filename` = `saisie-manuelle-<du>-<au>` (ASCII, neutre, dates ISO) — jamais affiché tel quel à l'écran
  (C-v016-2-8), lisible dans la trace et l'export ; `file_hash` = C-v016-2-6, hexadécimal minuscule de 64 caractères.
- **Écartées** : un nom traduit stocké (figé dans la langue du jour) ; un nom saisi par l'utilisateur (un champ de plus
  pour rien ; le numéro du relevé en tient lieu).
- **Réversible** : oui.

### C-v016-2-12 — L'API : JSON sous `/api/v1/bank-imports/manual`, sans empreinte d'aperçu

- **Contexte** : l'import est multipart et compare une empreinte d'aperçu (`bank_import_fingerprint.rs`, Story 27-3),
  qui protège contre un **profil** ou un fichier changé entre l'aperçu et la création.
- **Retenu** : `POST /api/v1/bank-imports/manual/preview` (aucune écriture : sommes, écart, avertissements) et
  `POST /api/v1/bank-imports/manual` (création, drapeaux `confirmDuplicateFile`, `confirmDuplicateLines`,
  `confirmOpeningBalanceGap`), corps JSON identique, une seule fonction de validation pour les deux. Codes nouveaux :
  `BANK_STATEMENT_UNBALANCED` (422), `BANK_STATEMENT_INVALID` (400, avec rang de ligne et champ),
  `BANK_STATEMENT_DUPLICATE_LINES` (422), `BANK_STATEMENT_OPENING_BALANCE_GAP` (422) ; repris de l'import :
  `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (404), `BANK_ACCOUNT_CLOSED` (409), `BANK_IMPORT_DUPLICATE_FILE` (422). Code dans
  un module de routes nouveau (`routes/bank_statements_manual.rs`), qui appelle les aides de `bank_imports.rs`
  rendues `pub(crate)`.
- **Écartées** : une empreinte d'aperçu (le corps **est** le contenu ; aucun profil n'intervient) ; un seul `POST` sans
  aperçu (C-v016-2-8) ; ajouter ces handlers à `bank_imports.rs` (2307 lignes).
- **Réversible** : oui avant publication.

### C-v016-2-13 — MCP : rien à changer, et la prémisse se corrige

*Annoté le 2026-10-30, validation P2 (R9) : « aucun outil ne lit `bank_transactions` » est faux dans sa forme absolue —
`postes_ouverts` la lit indirectement (`document_owners`, `journal_entries.rs:2276-2279`), `id` et `matched_entry_id`
seulement, jamais l'origine. La conclusion tient.*

- **Contexte** : le mandat suppose que les outils MCP lisent les transactions bancaires. Au code, **aucun** outil ne lit
  `bank_transactions` ni `bank_imports` (`grep -rln "bank_transactions\|bank_imports" crates/kesh-api/src/mcp/` : rien ;
  les quinze outils de `mcp/outils/` lisent livres, factures, contacts, pièces).
- **Retenu** : rien à changer. Ce que l'agent voit d'un relevé saisi, ce sont les **écritures** que son rapprochement
  produit — par les outils `ecritures`, `grand_livre`, `rapports`, identiques à celles d'un import.
- **Réversible** : sans objet.

### C-v016-2-14 — Pas de découpage

*Annoté le 2026-10-30, validation P3 (2a R6) : **levé par C-v016-2-18** (découpage en v016-2a et v016-2b).*

*Annoté le 2026-10-30, validation P1 (R5) : au grain des exemples de la règle, la story touche **six** modules, non
cinq (`frontend/src/lib/features/journal-entries` en plus) ; le seuil est franchi. Décision de l'orchestrateur : pas de
découpage ; la dérogation est écrite à la fiche (§ *Dérogation règle de splitting*). Le texte ci-dessous est celui de la
première rédaction.*

- **Contexte** : règle de splitting préventif (plus de cinq modules distincts).
- **Retenu** : une seule story. Modules : `kesh-core` (`bank_imports`), `kesh-db` (entité, dépôt, une migration),
  `kesh-api` (routes, erreurs, audit), `kesh-i18n` (catalogues), `frontend` (`features/bank-import`, une page) — **cinq**,
  pas plus, tous du seul domaine de l'import bancaire ; la route est mince (elle compose des fonctions existantes). Un
  découpage backend / frontend ne livrerait rien d'utilisable seul et doublerait le cycle de revue d'une release
  candidate attendue (rc2 à la première rédaction ; `v0.17.0-rc1` depuis C-v016-2-15).
- **Écartées** : `v016-2a` (backend) puis `v016-2b` (écran, manuels, E2E).
- **Réversible** : si la validation montre une non-convergence (sévérité qui stagne sur des défauts recyclés), découper
  selon cette ligne.

### C-v016-2-15 — La release devient la 0.17.0 (amendement du 2026-10-30, décision du Project Lead)

- **Contexte** : décision de Guy du 2026-10-30, transmise par l'orchestrateur après la première rédaction de la fiche :
  la release n'est plus la 0.16.1 mais la **0.17.0**. Elle réunit le correctif v016-1 (#811, #812), publié comme
  candidate `v0.16.1-rc1` le 2026-10-29 mais jamais en version finale, et cette story ; première candidate
  `v0.17.0-rc1` ; la recette Claude Code et les tests se jouent une seule fois, sur elle. À `c5101b07`, les dix crates
  portent `version = "0.16.1"` (`grep -n '^version' crates/*/Cargo.toml`), et `CHANGELOG.md:11` porte
  `## [0.16.1] — 2026-10-29`, dont l'en-tête dit « Il n'ajoute **aucune migration** ».
- **Retenu** :
  - la migration de C-v016-2-2 relève `kesh_version_min_required` à **`0.17.0`** ; **P2-bis** : les dix crates passent
    à `0.17.0` **dans le même commit** que la migration (fiche, AC-2), sinon le binaire refuserait de démarrer sur sa
    propre base ; le développeur grepe la valeur `0.16.1` et trie chaque occurrence à la main (à `c5101b07`, dans
    `crates/`, hors `Cargo.toml`, seulement des commentaires « depuis la 0.16.1 » qui datent le correctif v016-1 : ils
    passent à la 0.17.0) ;
  - le test de refus de version porte sur `0.16.1` (la `v0.16.1-rc1` en est une), non plus sur `0.16.0` ;
  - le CHANGELOG `[0.16.1]` devient `[0.17.0] — Non publié`, avec *Ajouté* (#816), *Modifié* et *Corrigé* (#811,
    #812) ; la phrase « aucune migration » est supprimée ; une note dit la candidate `0.16.1-rc1`, publiée en pré-version
    puis non poursuivie ; le § *Passer à la 0.16.1* du manuel d'administration devient *Passer à la 0.17.0* — posés par
    le **développeur** (fiche, AC-29), parce qu'ils décrivent la migration qu'il écrit ; macros, URL `v0.16.1`, PDF,
    feuille de route et site restent à la release (garde G19) ;
  - le nom et la clé de la story restent `v016-2-releve-manuel` / `v0162`.
- **Conséquence heureuse** : la limite « une sauvegarde de la rc2 s'importe dans la rc1 sans s'y lire » (C-v016-2-2)
  tombe : `0.16.1 < 0.17.0`, la rc1 refuse la base et les sauvegardes de la 0.17.0.
- **Écartées** : garder `min_required = '0.16.1'` avec des crates en 0.17.0 (la rc1, de version `0.16.1`, accepterait
  alors les sauvegardes de la 0.17.0 sans en lire les relevés saisis) ; laisser le CHANGELOG à la release (il dirait
  « aucune migration » sur la branche jusque-là, et la migration est l'affaire du développeur).
- **Réversible** : avant publication de `v0.17.0-rc1`, oui ; ensuite, non (P8).

### C-v016-2-16 — Les données de la 0.16.1-rc1 se gardent (contrainte du Project Lead du 2026-10-30)

- **Contexte** : Guy, le 2026-10-30 : « 0.16.1 contient des données que j'aimerais garder : prends-y garde lors des
  développements. » L'installation du NAS tourne en `v0.16.1-rc1` (schéma de la 0.16.0, 86 migrations) et passera à
  `v0.17.0-rc1`. La migration de C-v016-2-2 ne porte qu'un `UPDATE _kesh_version` ; aucun test ne prouve aujourd'hui
  qu'une base 0.16.x **peuplée** d'imports bancaires se relit à l'identique après elle, ni qu'une sauvegarde 0.16.x
  s'importe dans la 0.17.0 (`check_import_version_compat`, `routes/admin.rs:216-231`, ne refuse que si le
  `min_required` du manifeste dépasse le binaire ; la 0.16.x écrit `0.14.0`).
- **Retenu** :
  - **montée sans perte** (fiche, AC-2-bis) : un test monte jusqu'à la migration exclue, pose des imports CAMT et CSV,
    des transactions `pending`, `reconciled` et rejetée, applique la suite, et relit tout à l'identique en base **et**
    par le code (décodage `from_str` compris) ; la migration ne porte aucune autre instruction que le relèvement,
    établi par test ;
  - **restauration croisée** (AC-3-bis) : la 0.17.0 n'ajoute aucune table (établi par test) ; une sauvegarde au
    manifeste de 0.16.x s'importe et se relit ; l'inverse est refusé ;
  - **la sauvegarde d'avant mise à jour** — base, `.keshbackup`, dossier des documents — est écrite au § *Passer à la
    0.17.0* (AC-29) et dans les notes de `v0.17.0-rc1` : le retour à la rc1 ne passe que par elle ;
  - **rien de jetable sur la base du NAS** : recette Claude Code, écriture de démonstration `4321.09`, clés de recette,
    captures se jouent sur une installation jetable (§ *Ce que la release doit contenir*, point 5-bis — *point 5 du bloc A
    depuis la remédiation P1*). La fiche ne
    prescrivait aucune écriture sur la base du NAS ; la règle est désormais écrite.
- **Écartées** : se fier au test `upgrade_path_preserves_data` existant (il ne pose pas de lignes bancaires dans l'état
  0.16.0 ni ne relit par le code) ; prouver la restauration croisée par la seule égalité des inventaires de tables (ne
  prouve pas la relecture) ; une fixture tirée de la base du NAS (données réelles : interdit, dépôt public).
- **Réversible** : sans objet — c'est une contrainte, non un choix de conception.

### C-v016-2-17 — Remédiation de la validation P1 (2026-10-30)

- **Contexte** : validation P1 (Sonnet, lentilles R et F), 0 CRITICAL, 0 HIGH, 12 MEDIUM, 17 LOW ; chaque prémisse
  vérifiée au code (fiche, Change Log, entrée « Validation P1 »).
- **Retenu** :
  - **la piste** : `insert_canonical_audit_log` reçoit `ImportAuditAction` (défini dans `routes/bank_imports.rs`, deux
    littéraux dans son `match`) ; une entrée `SITES_INDIRECTS` pour ce fichier avec les deux codes — la seule forme qui
    passe les deux sens de `audit_label_registry.rs` (`:534-545`) *(annoté à la validation P2, R1 : **avec** l'exclusion
    de `"upload.xml"` à `PAS_DES_CODES`, que le tamis du sens 2 retient dès que le fichier est inventorié)* ;
  - **les longueurs** en caractères (`chars().count()`), non en octets : `VARCHAR(255)` compte des caractères en
    MariaDB ; précédent `routes/api_keys.rs:130` ;
  - **l'empreinte canonique** encodée champ par champ, longueur sur quatre octets gros-boutiens puis octets UTF-8 ;
  - **dates et montants en chaînes** analysées par la route, pour que toute faute de forme rende
    `BANK_STATEMENT_INVALID` avec son rang *(annoté à la validation P4, 2a P6 : sauf un champ d'un autre type JSON que
    chaîne, rendu en `400 VALIDATION_ERROR` sans `field` — validation P3, 2a R2)* ; ensembles fermés de `field` et `reason` ;
  - **`isValidAmount`** à options (`maxDecimals`, `maxIntegerDigits`, `allowNegative`, `allowEmpty`), défauts du journal
    inchangés ;
  - **les rangs** : le formulaire retire les lignes vides et traduit tout rang du serveur en ligne de l'écran ;
  - **l'E2E** sur un compte bancaire propre à l'exécution (IBAN synthétique unique), sans relevé antérieur *(annoté à la
    validation P5 de la 2b, P4 : IBAN synthétique **fixe**, la base étant tronquée par `seedTestState('with-company')`,
    qui ne crée aucun compte bancaire — cf. 2b, AC-26)* ;
  - **la release** en deux temps : macros, URL G19 et PDF restent à 0.16.1 jusqu'au tag final (décision de
    l'orchestrateur, comme pour la `0.16.1-rc1`) ; `prepare-release.sh` au tag final seulement ;
  - **pas de découpage** (décision de l'orchestrateur), dérogation écrite *(annoté à la validation P3, 2a R6 : levé par
    C-v016-2-18)* ;
  - **Q7, Q8** (équilibre exact ; CHF seule, comptes en devise → #233) : choix gardés en attendant la réponse de Guy.
- **Écartées** : longueurs en octets (refuse à tort un texte accentué que la base accepte) ; séparateur textuel dans
  l'empreinte (collision par contenu) ; une seconde expression régulière de montant (DRY) ; refuser les lignes vides
  internes (moins commode qu'une correspondance de rangs) ; bumper les macros à la candidate (décision de l'orchestrateur ;
  le manuel de la candidate serait titré juste, mais les URL viseraient un tag `v0.17.0` encore inexistant — les deux
  options laissent, pendant la candidate, des URL vers un tag qui n'existe pas, ce que les notes de la candidate disent).
- **Réversible** : oui, avant le développement.

### C-v016-2-18 — Le découpage en v016-2a et v016-2b (validation P2, 2026-10-30)

- **Contexte** : la validation P2 (R6 = F4) relève que la § *Dérogation règle de splitting* invoquait des motifs que la
  règle n'admet pas : au-delà de cinq modules, le `CLAUDE.md` ne garde la story unique que pour un cycle de dépendance
  Cargo ou des fusions intermédiaires impossibles à tester en isolation — et la ligne de repli de la fiche montrait un
  découpage sans cycle, dont la première moitié se teste seule en HTTP.
- **Retenu** (décision de l'orchestrateur ; Guy informé — s'il préfère une story unique, un amendement suivra) :
  - **`v016-2a-releve-manuel-backend`** : `kesh-core`, `kesh-db` (migration, bump 0.17.0 des crates par P2-bis, montée
    sans perte AC-2-bis, sauvegarde 0.16.x AC-3-bis), `kesh-api` (routes, erreurs, audit, registres),
    `docs/api-external.md`, tests Rust, CHANGELOG et manuel d'administration liés à la migration et à la version
    (AC-1 à AC-18, AC-28, AC-29 hors son point 2) ;
  - **`v016-2b-releve-manuel-ecran`** : frontend, clés de l'écran, manuel utilisateur, E2E, captures, entrée *Ajouté* du
    CHANGELOG, README *Fonctionnalités* (AC-19 à AC-27, AC-29-bis — l'ancien point 2 de l'AC-29 —, AC-30) ; elle dépend
    de la 2a ;
  - même worktree, même branche, la 2a d'abord ; la fiche d'origine devient la **parente** (`split`), qui garde le
    contexte, les décisions, le contrat de l'API, les limites, la liste de la release et les questions Q1 à Q8 ; chaque
    AC dans une seule sous-fiche, sous son numéro d'origine (table de correspondance de la parente) ; ce registre reste
    commun.
- **Écartées** : la story unique avec dérogation (hors de l'exception de la règle) ; trois sous-fiches (la 2a ne se
  scinde pas sans séparer la migration de la route qui écrit la valeur nouvelle).
- **Réversible** : oui, avant le développement de la 2a.

### C-v016-2-19 — Remédiation de la validation P3 : trois décisions de l'orchestrateur (2026-10-30)

*Entrée écrite à la validation P4 (2a P1 = 2b P10) : ces décisions, prises à la P3, n'avaient d'écrit que dans les
fiches.*

- **Contexte** : validation P3 des sous-fiches (Sonnet), trois MEDIUM — 2a R1 (la parente promettait `415` sans
  `Content-Type` et `413` au-delà de la limite, que l'extracteur prescrit ne rend pas), 2a F1 (`README.md:235` dirait
  « Aucune migration » et « ✅ Publiée » pendant la candidate), 2b R1 (l'ensemble fermé des `reason` n'existait qu'en
  prose, la table TypeScript et son test en étant une copie sans lien avec le serveur).
- **Retenu** :
  - **(a) une règle unique `400 VALIDATION_ERROR`** pour tout rejet du corps — JSON illisible, `Content-Type` absent,
    dépassement de la limite —, alignée sur l'existant ; la macro `impl_validation_extractor` de
    `routes/bank_accounts.rs:196-226` **extraite** dans un module partagé (`src/extractors.rs`), réutilisée par
    `bank_accounts.rs` (comportement inchangé) et par la route de saisie ; un test par cas ; les deux autres copies
    (`test_endpoints.rs:111-…`, `reconciliation.rs:914-…`) tracées en #829 ;
  - **(b) l'épinglage Rust** : `ManualStatementReason` et `ManualStatementField` dans `kesh-core` (`code()`, `pub const
    ALL`, hors `cfg(test)`), un test qui les épingle sur les chaînes littérales de la parente, un test d'exhaustivité
    (`match` sans joker) qui lie `ALL` à l'énumération, et, posé par la 2b, un test Rust qui lit la table TypeScript et
    l'égale à `ManualStatementReason::ALL` dans les deux sens ;
  - **(c) `README.md:235`** : la phrase « Aucune migration … » retirée, un statut de candidate (non « ✅ Publiée »)
    pendant `v0.17.0-rc1`, la bascule à « ✅ Publiée » au tag final (liste B de la release).
- **Écartées** : (a) les statuts natifs d'Axum `415` et `413` par un extracteur propre (un second comportement à côté
  de l'existant, et une quatrième copie ou une variante de la macro) ; une copie locale de la macro (DRY) ; (b) une
  liste recopiée côté TypeScript seule, sans lien avec le serveur (verte par construction, D4-ter) ; (c) « ✅ Publiée »
  posé dès la candidate, ou la ligne laissée en `v0.16.1` jusqu'au tag.
- **Réversible** : oui, avant le développement de la 2a.

### C-v016-2-20 — L'exhaustivité de `ALL` sans circularité (développement de la 2a, étape 0, 2026-10-30)

- **Contexte** : validation P5 de la 2a (P1) et de la 2b (P2) : un `match` exhaustif sans joker appelé en itérant `ALL`,
  ou un tableau écrit à la main égalé à `ALL`, reste circulaire — on ajoute le bras (ou la ligne) sans ajouter la
  variante à `ALL`, et le test reste vert. Précision de l'orchestrateur : forme `index()` + `COUNT` + test.
- **Retenu** (pour `ManualStatementReason` et `ManualStatementField`, `kesh-core/src/bank_imports/manual.rs`) :
  - `index(self) -> usize` par `match` exhaustif sans joker ; `pub const COUNT` écrit à côté ; `pub const ALL` ;
  - un test : `ALL.len() == COUNT` et `ALL[i].index() == i` pour tout `i` (ordre et absence de doublon) ;
  - **en plus**, constat fait au développement : si l'on ajoute une variante et son bras d'`index` (indice `COUNT`)
    sans toucher **ni** `ALL` **ni** `COUNT`, les deux assertions précédentes restent vertes ; un second test compte
    donc les variantes **dans le texte source** de l'énumération (`include_str!("manual.rs")`, corps de
    `pub enum ManualStatementReason { … }`, une variante par identifiant en tête de ligne hors commentaire) et l'égale à
    `COUNT` et à `ALL.len()`. Il ne dépend que du fichier, non de `ALL` : M23-bis (variante et bras ajoutés, `ALL`
    inchangée) rougit à l'exécution.
- **Écartées** : la dérive `strum::EnumIter` (dépendance nouvelle pour un seul test) ; `std::mem::variant_count`
  (instable) ; le seul couple `index`/`COUNT` (circulaire dans le cas ci-dessus).
- **Réversible** : oui.

### C-v016-2-21 — L'analyse du corps vit dans `kesh-core` (développement de la 2a, 2026-10-30)

- **Contexte** : la parente dit « la route analyse ces chaînes vers le type de `kesh-core` ». L'analyse (dates strictes
  `AAAA-MM-JJ`, nombres stricts, sens, absence ≡ chaîne vide) est pure, et ses refus portent les ensembles fermés de
  `kesh-core`.
- **Retenu** : le type reçu (`ManualStatementRaw`, `ManualLineRaw`, `Deserialize`, `Option<String>`) et
  `parse_manual_statement` vivent dans `kesh_core::bank_imports::manual`, à côté de `validate_manual_statement` ; la
  route ne fait que les appeler et traduire l'erreur. Les refus de forme (`required`, `invalid_date`, `invalid_number`,
  `unknown_direction`, et `too_many_lines` avant l'analyse des lignes) viennent de l'analyse, les règles 1 à 6 de la
  validation ; une partie entière de plus de treize chiffres significatifs rend `too_many_digits` dès l'analyse (elle
  pourrait déborder `Decimal`).
- **Écartées** : l'analyse dans la route (non testable sans HTTP, et les codes de refus s'y écriraient une seconde fois).
- **Réversible** : oui.

### C-v016-2-22 — La sauvegarde 0.16.x : moyen (a), le manifeste récrit (développement de la 2a, 2026-10-30)

- **Contexte** : AC-3-bis, moyens (a) ou (b), par ordre de préférence.
- **Retenu** : (a) — `full_import_accepts_a_0_16_backup` exporte depuis la 0.17.0 une base peuplée d'un import CAMT à
  soldes, d'un import CSV et de transactions (une rapprochée), récrit le manifeste (`keshVersion` `0.16.1`,
  `keshVersionMinRequired` `0.14.0`) et l'importe. Licite parce que la 0.17.0 n'ajoute ni table ni colonne
  (`bank_imports_manual_source_adds_no_table`) : les données d'une 0.16.x ont exactement cette forme.
- **Écartées** : (b) une fixture générée au tag `v0.16.1-rc1` (un worktree de plus, et une preuve qui n'ajoute rien
  tant que le schéma ne change pas).
- **Réversible** : oui ; (b) s'imposera le jour où une version ajoutera une colonne aux tables bancaires.

### C-v016-2-23 — Le statut de la ligne `v0.17.0` du README : « 🚧 Candidate » (développement de la 2a, 2026-10-30)

- **Contexte** : C-v016-2-19 (c) laisse le sigle à l'orchestrateur, « p. ex. 🚧 Candidate ».
- **Retenu** : `🚧 Candidate`, la ligne renommée `v0.17.0`, la phrase « Aucune migration » remplacée par la migration et
  la sauvegarde d'avant la mise à jour, la candidate `0.16.1-rc1` nommée.
- **Réversible** : oui (bascule à « ✅ Publiée » au tag final, liste B de la release).

### C-v016-2-24 — L'écran de saisie : raisons, lignes à deux montants, aperçu périmé (développement de la 2b, 2026-10-30)

- **Contexte** : AC-20 à AC-22 fixent la séquence de la raison unique, l'instantané de l'aperçu et la table des rangs,
  sans dire le texte des raisons, ni ce que part dans le corps d'une ligne à deux montants ou sans montant, ni comment
  l'écran montre un aperçu périmé.
- **Retenu** :
  - dix raisons, une clé chacune (`bank-import-manual-blocked-*`) : compte ; période vide ou illisible ; période
    inversée (la séquence « période (vide, illisible, inversée) » en deux messages) ; numéro ; soldes ; nombre de
    lignes ; ligne fautive (son rang affiché, une phrase qui dit les quatre règles) ; écart ; aperçu ; cases ;
  - une ligne à deux montants part avec le montant du débit et **sans sens** (`direction: null`), une ligne sans montant
    sans montant : le serveur la désigne (`required`), et l'écran la marque déjà fautive ;
  - un montant ou un solde hors forme part **tel que saisi** (trimé), pour que le serveur le désigne ; au format, il
    part normalisé (`toFixed(2)`) ;
  - un aperçu dont le relevé a changé tombe (cases décochées) et un avis « le relevé a changé depuis la vérification »
    s'affiche jusqu'à la vérification suivante ; un refus affiché tombe de même ;
  - un refus de **création** qui demande une case (relevé identique, doublons, écart de solde de départ, apparus depuis
    l'aperçu) retire l'aperçu : on vérifie de nouveau pour voir la case.
- **Écartées** : une raison générique « formulaire incomplet » (contraire à l'AC-21) ; bloquer « Vérifier » sur une ligne
  fautive (contraire à l'AC-21, les refus du serveur ne seraient plus atteints) ; garder l'aperçu après un 422 de
  création (la case nouvelle ne s'afficherait pas).
- **Réversible** : oui.

### C-v016-2-25 — Les textes : en-têtes de la liste, variantes de suppression, détail (développement de la 2b, 2026-10-30)

- **Contexte** : AC-23 demande des en-têtes « qui valent pour les deux origines » et des variantes propres de la boîte
  de suppression, sans en fixer ni les mots ni le nom des clés.
- **Retenu** : en-têtes « Enregistré le » et « Relevé » (de-CH « Gespeichert am », « Auszug » ; it-CH « Registrato il »,
  « Estratto » ; en-CH « Recorded on », « Statement ») ; les variantes de suppression sous `bank-import-manual-delete-*`
  (et non `bank-import-delete-manual-*`), si bien que toutes les clés statiques neuves de l'écran tombent sous la
  famille `bank-import-manual-*` que nomme l'AC-24 ; au détail, « Saisie manuelle » sert de nom **et** de format (une
  seule clé, `bank-import-detail-manual-name`, rendue par `bank-import-display.ts`, source unique de la règle côté
  frontend) ; le titre « Détail du relevé bancaire » et le `<h2>` « Relevé #… » valent pour les deux origines.
- **Écartées** : garder « Fichier » pour un relevé saisi (faux) ; une clé de format distincte au même texte (DRY).
- **Réversible** : oui (textes).

### C-v016-2-26 — Le test croisé des `reason` vit dans `textes_coherents.rs` (G24) (développement de la 2b, 2026-10-30)

- **Contexte** : AC-24 place le test croisé dans `crates/kesh-api/tests/`, sur le patron `lire` de
  `textes_coherents.rs:46-48`.
- **Retenu** : deux tests ajoutés à `textes_coherents.rs` (garde **G24**, inscrite au tableau d'en-tête), qui réemploient
  `racine()` et `lire()` : `g24_l_extracteur_de_la_table_des_reason` (témoins vert et rouges de l'extracteur : bloc
  absent, non fermé, vide) et `g24_la_table_des_reason_egale_l_ensemble_ferme_du_serveur` (égalité des deux ensembles
  dans les deux sens, absence de doublon, `COUNT`).
- **Écartées** : un fichier de test neuf (un binaire de plus à lier, et une seconde copie de `lire`, contraire au DRY).
- **Réversible** : oui.

### C-v016-2-27 — La recette visuelle est un test E2E sauté hors captures (développement de la 2b, 2026-10-30)

- **Contexte** : AC-25 énumère une vingtaine d'états à capturer, sur une base de démonstration.
- **Retenu** : un troisième test de `bank-manual-statement.spec.ts`, `test.skip` sauf si `KESH_E2E_CAPTURES_DIR` est posé
  (patron de `backup-schedule.spec.ts`), qui parcourt les états et les capture ; il tourne sur la base E2E du worktree,
  jamais sur une installation réelle.
- **Écartées** : des captures prises à la main (non rejouables) ; un parcours de captures exécuté à chaque suite E2E
  (quatre minutes de plus pour des images que personne ne regarde).
- **Réversible** : oui.
