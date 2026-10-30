# Story v016-2 : saisir un relevé bancaire à la main

Status: split

*Fiche rédigée le 2026-10-30, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0162`, branche
`story/v016-2-releve-manuel` partie de `c5101b07` (tête de `story/v016-1-mcp-sorties`, candidate `v0.16.1-rc1`, PR #815
brouillon) ; la story se fusionne sur cette branche de release. Registre : `v016-2-choix-autonomes.md` (C-v016-2-1 à
C-v016-2-19 ; borne portée à la validation P5 de la 2b). **Amendée le 2026-10-30 : la release devient la 0.17.0** (décision du Project Lead, C-v016-2-15) — la
clé et le nom de la story restent `v016-2-releve-manuel`. **Contrainte du Project Lead du même jour (C-v016-2-16)** :
les données de l'installation `v0.16.1-rc1` du NAS se gardent — montée sans perte, sauvegarde 0.16.x importable, rien
de jetable sur la base de Guy (AC-2-bis, AC-3-bis, AC-29, § *Ce que la release doit contenir*). **Remédiée après la
validation P1** (2026-10-30, Change Log ; choix C-v016-2-17). **Découpée à la validation P2 en `v016-2a` et `v016-2b`** (§ *Découpage*, C-v016-2-18). Clé courte : `v0162`. Issue : **#816** (enhancement), en `refs` seulement, dans les commits **et** dans la
PR de la story ; la fermeture se porte sur la **PR de release** (§ *Ce que la release doit contenir*). Les numéros de
ligne cités sont ceux de `c5101b07`.*

## Story

En tant que Guy, dont un compte bancaire réel n'offre ni CAMT.053 ni CSV — l'installation du NAS porte des données à
garder —, je veux
**saisir un relevé à la main** — période, soldes de départ et d'arrivée, puis ses lignes — afin que ce compte entre dans
le circuit bancaire de Kesh comme les autres : transactions à rapprocher, propositions, annulation, suppression,
solde du relevé confronté aux livres sur la tuile et au contrôle de clôture, et une trace qui dit, dans la piste de
contrôle, que ce relevé a été tapé et non importé.

## Origine

- **#816** (2026-10-30, recette de la 0.16.1-rc1) : « Certaines banques ne fournissent aucun de ces formats pour un
  compte donné […]. Pour ce compte, le relevé n'existe que sur papier ou en PDF. » Critères de l'issue : (1) un relevé
  saisi produit des transactions rapprochables, comme un import ; (2) soldes de départ et d'arrivée contrôlés contre la
  somme des lignes ; (3) une saisie se distingue d'un import dans la piste de contrôle ; (4) les manuels décrivent les
  deux voies.
- **Décision du Project Lead du 2026-10-30** : la fonctionnalité entre dans la release en cours. *Première rédaction :
  la 0.16.1, par une candidate `v0.16.1-rc2`.* **Amendement du même jour (C-v016-2-15)** : la release devient la
  **0.17.0** — elle réunit le correctif v016-1 (#811, #812), publié comme candidate `v0.16.1-rc1` mais jamais en version
  finale, et cette story ; première candidate `v0.17.0-rc1`, sur laquelle la recette Claude Code et les tests se jouent
  une seule fois.
- Voisinage : **#418** (annuler un rapprochement, fermée) et **#526** (une transaction rejetée se retrouve et se
  dé-rejette, fermée) — le circuit dont un relevé saisi hérite ; **#671** (P2, ouverte) et **#687** (P3, ouverte) —
  défauts du rapprochement, sans lien avec l'origine d'une transaction : ni aggravés ni réglés ici.

## Ce que le code dit (relevé le 2026-10-30, `c5101b07`)

### Prémisses de #816 — confirmées

- **« Les transactions bancaires n'entrent dans Kesh que par l'import d'un fichier »** : vrai. Le seul créateur de
  `bank_transactions` hors tests est `bank_imports::create_with_transactions`
  (`crates/kesh-db/src/repositories/bank_imports.rs:72`), appelé par `create_import_or_roll_back`
  (`crates/kesh-api/src/routes/bank_imports.rs:83-98`) aux deux chemins de `POST /api/v1/bank-imports` (CAMT `:1154`,
  CSV `create_csv` `:2017`). L'inventaire `crates/kesh-api/tests/bank_account_closed_inventory.rs` le tient clos.
- **« Taper un CSV à la main puis l'importer : possible aujourd'hui »** : vrai, mais il faut un profil CSV, et un import
  CSV ne porte **aucun solde** (`user-manual.tex:2040-2041`, « Seul le CAMT.053 porte le solde de clôture […] ; un
  import CSV n'en a pas ») : la tuile du tableau de bord et le contrôle K3 de la clôture ne verraient jamais ce compte
  comme ayant un relevé (`bank_accounts.rs:1798-1804`, `closing_balance IS NOT NULL` ; `closing_checks.rs:450-457`,
  motif `NO_BALANCE_STATEMENT`).
- **« Passer directement les écritures au journal : le compte échappe au rapprochement »** : vrai ; et, plus
  précisément, il échappe aussi à la tuile (aucun relevé) et à K3.

### Prémisses du mandat — une corrigée

- **« Les transactions saisies sont-elles lues par les outils MCP comme les autres ? (Elles devraient l'être sans rien
  changer) »** : **inexact dans sa forme**. Aucun outil MCP ne lit `bank_imports` ; un seul lit `bank_transactions`,
  **indirectement** : `postes_ouverts` passe par `services::postes_ouverts` → `open_items` → `document_owners`
  (`journal_entries.rs:2276-2279`), qui n'en lit que `id` et `matched_entry_id`, jamais l'origine — une transaction
  saisie y paraît comme une importée *(précisé à la validation P2, R9 : le `grep` sur le seul répertoire `mcp/`,
  `grep -rln "bank_transactions\|bank_imports" crates/kesh-api/src/mcp/`, ne voit pas cette lecture)* ; `mcp/outils/` porte
  `balance_agee`, `contacts`, `ecritures`, `exercices`, `factures`, `factures_fournisseur`, `grand_livre`,
  `kesh_version`, `pieces`, `plan_comptable`, `postes_ouverts`, `rapports`. Ce que l'agent voit d'un relevé, saisi ou
  importé, ce sont les **écritures** que produit son rapprochement. Conclusion inchangée : rien à faire côté MCP
  (C-v016-2-13).

### Ce sur quoi la conception s'appuie

- `bank_imports.source_format` : `VARCHAR(32) NOT NULL`, **sans** contrainte de valeur, délibérément
  (`20260504000001_bank_imports.sql:16-17`, `:25`) ; bornée par l'énumération Rust `BankImportSourceFormat`
  (`crates/kesh-db/src/entities/bank_import.rs:26-31`), dont `from_str` refuse toute autre valeur (`:53-60`).
  ⚠️ D'où la migration de C-v016-2-2 : une valeur nouvelle ne se décode pas dans un binaire antérieur.
- `bank_transactions.amount DECIMAL(18,2)` signé, positif = crédit du titulaire (`20260504000001_bank_imports.sql:7`,
  `:62`) ; `details TEXT NOT NULL` (`:65`) ; `reference VARCHAR(255)` (`:64`).
- `bank_imports.filename VARCHAR(255) NOT NULL`, `file_hash CHAR(64) NOT NULL` avec
  `CHECK (CHAR_LENGTH(file_hash) = 64)` (`:23-24`, `:51`) ; plus d'unicité depuis la 8-3 (le doublon de fichier est
  refusé par la route, `find_by_company_and_hash`).
- L'équilibre de l'import tolère un centime (`kesh-core/src/bank_imports.rs:254-271`, `diff > Decimal::new(1, 2)`) et
  s'admet sur confirmation (`routes/bank_imports.rs:1259-1278`).
- Les doublons de lignes de l'import sont **ignorés par défaut** (`ConfirmDuplicateLines::Skip`,
  `routes/bank_imports.rs:389-398` ; `apply_duplicate_lines_filter` `:1644-1696`).
- La trace de l'import : une action canonique `bank_import.created` (`insert_canonical_audit_log`,
  `routes/bank_imports.rs:1698-1750`) ; la suppression : `bank_import.deleted`, dont les détails portent déjà
  `source_format` (`:1540-1549`).
- Le rapprochement note référence 40 %, montant 50 %, contrepartie 10 % (`kesh-reconciliation/src/matching.rs:183`) ;
  aucun chemin de rapprochement ne lit `source_format`, `filename` ni `import_id` autrement que comme colonne
  (`grep -rn "source_format\|filename" crates/kesh-api/src/routes/reconciliation*.rs crates/kesh-db/src/repositories/reconciliation*.rs crates/kesh-reconciliation/src`
  : rien).
- Rôles : `POST /bank-imports/preview` et `POST /bank-imports` aux routes Comptable+ (`crates/kesh-api/src/lib.rs:792-803`),
  ouverts aux clés `read-write` (`docs/api-external.md:223`) ; la suppression fermée à toute clé
  (`routes/bank_imports.rs:1490-1493`).

## Décisions (détail et alternatives au registre)

| # | Décision | Registre |
|---|---|---|
| D1 | Un relevé saisi est un import de `source_format = 'MANUAL'`, créé par `create_with_transactions` ; aucune table, aucune colonne | C-v016-2-1 |
| D2 | Une migration dont la seule instruction relève `kesh_version_min_required` à `0.17.0` ; les dix crates passent à `0.17.0` dans le même commit (P2-bis) | C-v016-2-2, C-v016-2-15 **(Q1)** |
| D3 | Entête : compte, période, numéro facultatif, soldes signés ; lignes : date, valeur facultative, libellé, contrepartie et référence facultatives, montant positif + sens ; deux décimales ; 0 à 500 lignes ; CHF | C-v016-2-3 **(Q4)** |
| D4 | Équilibre **exact**, bloquant, sans confirmation ; dates de comptabilisation dans la période | C-v016-2-4 |
| D5 | Solde de départ ≠ clôture du relevé précédent : signalé à l'aperçu, confirmation exigée à la création, jamais bloqué | C-v016-2-5 **(Q3)** |
| D6 | Doublons de lignes : mêmes empreinte et fenêtre que l'import, **sans** « ignorer » — confirmer ou corriger ; relevé identique : `BANK_IMPORT_DUPLICATE_FILE` ; chevauchement : information ; saisi puis importé : décrit, issue demandée | C-v016-2-6 **(Q6)** |
| D7 | Correction : suppression du relevé entier (route existante), puis ressaisie ; aucune modification de ligne | C-v016-2-7 **(Q2)** |
| D8 | Écran `/bank-import/manual`, lien depuis `/bank-import`, aperçu puis enregistrement, totaux et écart en continu | C-v016-2-8 |
| D9 | Rôles et clés : ceux de l'import (Comptable+, clés `read-write`) | C-v016-2-9 **(Q5)** |
| D10 | Trace : action `bank_import.entered_manually` ; suppression : `bank_import.deleted` inchangée | C-v016-2-10 |
| D11 | `filename` = `saisie-manuelle-<du>-<au>` ; `file_hash` = SHA-256 du contenu canonique | C-v016-2-11 |
| D12 | API JSON `POST /api/v1/bank-imports/manual/preview` et `POST /api/v1/bank-imports/manual`, sans empreinte d'aperçu | C-v016-2-12 |
| D13 | MCP : rien à changer | C-v016-2-13 |
| D14 | *(Première rédaction : pas de découpage.)* **Découpage en `v016-2a` (backend) et `v016-2b` (écran)**, décision de l'orchestrateur à la validation P2 | C-v016-2-14, C-v016-2-18 |

### Le corps de requête (préview et création)

```json
{
  "bankAccountId": 3,
  "periodFrom": "2026-09-01",
  "periodTo": "2026-09-30",
  "statementNumber": "9/2026",
  "openingBalance": "1250.40",
  "closingBalance": "980.15",
  "lines": [
    { "bookingDate": "2026-09-03", "valueDate": null, "details": "Prélèvement assurance",
      "counterpartyName": "Assurance Exemple SA", "reference": null, "amount": "320.25", "direction": "debit" },
    { "bookingDate": "2026-09-15", "valueDate": "2026-09-15", "details": "Virement reçu",
      "counterpartyName": null, "reference": "F-2026-0042", "amount": "50.00", "direction": "credit" }
  ],
  "confirmDuplicateFile": false,
  "confirmDuplicateLines": false,
  "confirmOpeningBalanceGap": false
}
```

(Données d'exemple **synthétiques**.) Montants et soldes en **chaînes** décimales (comme partout dans l'API) ; l'aperçu
ignore les trois drapeaux. `statementNumber`, `valueDate`, `counterpartyName`, `reference` : facultatifs, une chaîne
vide ou faite de blancs vaut absence. La transaction enregistrée porte `amount = +montant` pour `credit`, `−montant`
pour `debit`, `currency = "CHF"`, `end_to_end_id`, `transaction_id`, `counterparty_iban` à `NULL`.

### Réponse de l'aperçu (`200`, aucune écriture)

`{ "totalCredits", "totalDebits", "computedClosingBalance", "transactionCount", "warnings": { "duplicateFile":
DuplicateFilePayload | null, "duplicateLines": [{ "line": <rang 1-based>, "existingTransactionId", "key" }],
"openingBalanceGap": { "expected", "entered", "previousImportId", "previousPeriodTo" } | null,
"overlappingStatements": [{ "importId", "periodFrom", "periodTo", "sourceFormat" }] } }`. Un relevé déséquilibré ou
invalide rend à l'aperçu **la même** erreur qu'à la création (une seule fonction de validation).

### Codes d'erreur

| Code | HTTP | Quand | Charge utile |
|---|---|---|---|
| `BANK_STATEMENT_INVALID` | 400 | une règle de l'AC-4 (r. 1 à 5) violée ; `field` et `reason` pris dans les ensembles **fermés** ci-dessous | `{ "line": <rang 1-based dans le tableau envoyé> \| null, "field": "<nom JSON>", "reason": "<code>" }` |
| `BANK_STATEMENT_UNBALANCED` | 422 | `départ + crédits − débits ≠ arrivée` | `{ "opening", "closing", "computedClosing", "diff" }` |
| `BANK_STATEMENT_DUPLICATE_LINES` | 422 | doublons sans `confirmDuplicateLines` | `{ "lines": [{ "line", "existingTransactionId" }] }` |
| `BANK_STATEMENT_OPENING_BALANCE_GAP` | 422 | écart de solde de départ sans `confirmOpeningBalanceGap` | `{ "expected", "entered", "previousImportId", "previousPeriodTo" }` |
| `BANK_IMPORT_DUPLICATE_FILE` | 422 | relevé identique déjà enregistré, sans `confirmDuplicateFile` | celle de l'import |
| `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` | 404 | compte absent, d'une autre société ou archivé | — |
| `BANK_ACCOUNT_CLOSED` | 409 | compte clos | celle de l'import |

**Ensembles fermés de `BANK_STATEMENT_INVALID`** (validation P1, F7) — `field` ∈ { `periodFrom`, `periodTo`,
`statementNumber`, `openingBalance`, `closingBalance`, `lines`, `bookingDate`, `valueDate`, `details`,
`counterpartyName`, `reference`, `amount`, `direction` } (`line` non nul pour les sept derniers) ; `reason` ∈ {
`required` (absent ou vide), `invalid_date` (chaîne qui n'est pas `AAAA-MM-JJ`), `invalid_number` (chaîne qui n'est pas
un décimal), `period_inverted` (r. 1), `too_many_lines` (r. 2), `out_of_period` (r. 3), `too_long` (r. 4),
`not_positive` (r. 5, montant), `too_many_decimals` (r. 5), `too_many_digits` (r. 5), `unknown_direction` }. Dates et
montants arrivent en **chaînes** et sont analysés par la route, afin que toute faute de forme rende ce code avec son
rang (et non un rejet Axum sans ligne) *(annoté à la validation P4, 2a P6 = 2b P9 : toute faute de forme **d'une
chaîne** ; un champ d'un autre type JSON rend `400 VALIDATION_ERROR` sans `field`, ci-dessous)*. Côté frontend, une table `reason → clé i18n` littérale
(`bank-import-manual-reason-*`, une clé par `reason`) et un test de couverture réciproque (chaque `reason` du serveur a
sa clé, chaque clé a son `reason`), sur le patron de `CSV_LINE_ERROR_LABELS` — *(citation corrigée à la validation P2,
F12)* la table vit dans `frontend/src/lib/features/bank-import/csv-line-error-label.ts:23`, son test de couverture dans
`csv-line-error-label.test.ts:22`, et sa famille est déclarée à `MOTIFS_DYNAMIQUES` et comptée à `CARDINALITES` de
`frontend/src/lib/shared/i18n-keys.test.ts` (`:882`) : la famille `bank-import-manual-reason-` y reçoit les deux mêmes
entrées. **Épinglage côté Rust** (décision de l'orchestrateur à la validation P3, 2b R1) : le patron cité compare sa
table à une liste **recopiée** dans son test (`csv-line-error-label.test.ts`), sans lien avec le serveur ; ici, les
ensembles fermés vivent dans deux énumérations de `kesh-core` (`ManualStatementReason`, `ManualStatementField`, `code()`,
`ALL`) qu'un test Rust épingle sur ces chaînes (2a, AC-4), et un test Rust de la 2b lit la table TypeScript (fichier du
dépôt, patron `lire` de `crates/kesh-api/tests/textes_coherents.rs:46-48`) et l'égale à `ManualStatementReason::ALL`
dans les deux sens (2b, AC-24).

**Type d'entrée de la route et extracteur** (validation P2, R3) : tous les champs du relevé et des lignes en
`Option<String>` (dates, montants, sens, textes), `lines: Option<Vec<…>>`, seul `bankAccountId: i64` requis — si bien que
tout champ chaîne **requis** absent ou vide, ou tout champ chaîne mal formé *(précisé à la validation P4, 2b P9 : un
champ facultatif vide vaut absence)*, atteint `BANK_STATEMENT_INVALID` avec son `field` et son `reason` *(limite
écrite à la validation P3, 2a R2 : un champ d'un autre type JSON que chaîne, `lines` qui n'est pas un tableau,
`bankAccountId` absent ou non entier rendent `400 VALIDATION_ERROR` sans `field` ni `reason`)* ; la route analyse ces
chaînes vers le type de `kesh-core`. Le corps passe par l'extracteur de `routes/bank_accounts.rs:196-226`,
**extrait dans un module partagé** et réutilisé tel quel (décision de l'orchestrateur à la validation P3, 2a R1 ; rejet →
`400 VALIDATION_ERROR`) et **jamais** par le `Json` nu d'Axum 0.8, qui rend
`422` sur une erreur de données — le statut des refus métier. Taille : 500 lignes de 500 + 255 + 255 caractères de
quatre octets approchent 2,1 Mo, au-delà de la limite par défaut d'Axum (2 Mio ; rejet que l'extracteur rend en `400`, *annoté à la validation P4, 2a P6*) : les deux routes se montent avec
`DefaultBodyLimit::max(bank_import_max_mib)` comme l'import (`lib.rs:792-803`).

Ordre des refus à la création, chacun **sans écriture** : authentification et rôle (middlewares) ; corps JSON illisible ou
de mauvaise forme, `Content-Type` absent ou corps au-delà de la limite — tous en `400 VALIDATION_ERROR` (extracteur
ci-dessus ; *la première rédaction annonçait `415` et `413`, que l'extracteur ne rend pas — corrigé à la validation P3,
2a R1*) ; compte (`404`, `409`, pré-vol sur le pool comme l'import, `routes/bank_imports.rs:1161-1180`) ;
`BANK_STATEMENT_INVALID` ; `BANK_STATEMENT_UNBALANCED` ; puis, **dans** la transaction ouverte, relevé identique,
doublons de lignes, écart de solde de départ — dans cet ordre, le premier qui manque de sa confirmation l'emporte.

## Découpage (validation P2, 2026-10-30)

*Décision de l'orchestrateur, C-v016-2-18 : la règle de splitting préventif du `CLAUDE.md` n'admet la story unique, au
delà de cinq modules, que pour un cycle de dépendance Cargo ou des fusions intermédiaires intestables ; aucune des deux
ne vaut ici, et la ligne de repli de la § *Dérogation* (première rédaction) se testait en isolation. Guy en est informé ;
s'il préfère une story unique, un amendement suivra.*

- **`v016-2a-releve-manuel-backend.md`** — `kesh-core`, `kesh-db` (migration, bump 0.17.0 des crates, montée sans perte,
  sauvegarde 0.16.x), `kesh-api` (routes, erreurs, audit, registres), `docs/api-external.md`, tests Rust, CHANGELOG et
  manuel d'administration liés à la migration et à la version.
- **`v016-2b-releve-manuel-ecran.md`** — le frontend, les clés de l'écran, le manuel utilisateur, l'E2E, les captures,
  l'entrée *Ajouté* du CHANGELOG et le README *Fonctionnalités*. **Dépend de la 2a.**

Même worktree, même branche ; la 2a d'abord. Ce qui reste dans cette fiche parente vaut pour les deux : contexte,
décisions, contrat de l'API, limites, liste de la release, questions au Project Lead. Chaque AC d'origine est dans **une
seule** sous-fiche, sous son numéro d'origine :

| AC d'origine | Sous-fiche |
|---|---|
| AC-1 | `v016-2a` |
| AC-2 | `v016-2a` |
| AC-2-bis | `v016-2a` |
| AC-3 | `v016-2a` |
| AC-3-bis | `v016-2a` |
| AC-4 | `v016-2a` |
| AC-5 | `v016-2a` |
| AC-6 | `v016-2a` |
| AC-7 | `v016-2a` |
| AC-8 | `v016-2a` |
| AC-9 | `v016-2a` |
| AC-10 | `v016-2a` |
| AC-11 | `v016-2a` |
| AC-12 | `v016-2a` |
| AC-13 | `v016-2a` |
| AC-14 | `v016-2a` |
| AC-15 | `v016-2a` |
| AC-16 | `v016-2a` |
| AC-17 | `v016-2a` |
| AC-18 | `v016-2a` |
| AC-19 | `v016-2b` |
| AC-20 | `v016-2b` |
| AC-21 | `v016-2b` |
| AC-22 | `v016-2b` |
| AC-23 | `v016-2b` |
| AC-24 | `v016-2b` |
| AC-25 | `v016-2b` |
| AC-26 | `v016-2b` |
| AC-27 | `v016-2b` |
| AC-28 | `v016-2a` |
| AC-29 (points 1, 3 à 8) | `v016-2a` |
| AC-29 point 2 | `v016-2b`, sous **AC-29-bis** |
| AC-30 | `v016-2b` |

| Mutations | Sous-fiche |
|---|---|
| M1, M2, M3, M4, M5, M6, M7, M8, M9, M10, M11, M12, M16, M17, M18, M20, M21, M22 | `v016-2a` |
| M13, M14, M15, M19 | `v016-2b` |
| M23, M24 (ajoutées à la validation P3), M23-bis (ajoutée à la validation P4 ; ligne complétée à l'étape 0 de la 2a, validation P5, P4) | `v016-2a` |
| M25 à M30 (ajoutées à la validation P3) | `v016-2b` |

## Ce que les gardes ne voient pas

- **Saisi puis importé** : l'import d'un fichier qui couvre une période déjà saisie à la main n'avertit pas de la
  période. Ligne par ligne, la clé est (date, montant, premier identifiant non vide parmi référence, identifiant de bout
  en bout, identifiant de transaction) : une ligne d'un **CAMT**, qui porte presque toujours un identifiant, n'est
  reconnue que si la référence saisie égale ce premier identifiant ; une ligne saisie sans référence y entre presque
  toujours **deux fois**. Seul un CSV sans identifiant reconnaît la ligne saisie sans référence (et l'ignore par défaut).
  *(Récrit à la validation P2, R4 = F1 : la version P1 promettait la reconnaissance « absente des deux côtés », qui ne
  se présente pas en CAMT.)* Écrit au manuel ; avertissement à l'import demandé en issue.
- **Le relevé voisin** : le solde de départ n'est comparé qu'au relevé **précédent** à solde, dont la période finit
  **strictement** avant la nouvelle (`period_to < du`) ; deux relevés successifs qui partagent leur jour-frontière ne
  se voient pas, et un mois saisi entre deux relevés existants n'est jamais confronté au solde d'ouverture du
  **suivant** (validation P1, R14).
- **Doublons dans un même relevé** : deux lignes identiques d'un même relevé saisi ne sont pas signalées (limitation
  L15 de l'import, `kesh-core/src/bank_imports.rs`, « doublons intra-import non détectés ») — légitime pour deux retraits
  du même montant le même jour.
- **L'heure du serveur** : aucun contrôle de date future ; une année tapée de travers (2062) passe si l'équilibre tient.
  La tuile et K3 la montreraient. Écarté délibérément (C-v016-2-3).
- *(Première rédaction : « Un binaire 0.16.1-rc1 accepte une sauvegarde de la rc2 ». Levé par l'amendement
  C-v016-2-15 : avec `min_required = 0.17.0`, la rc1, de version Cargo `0.16.1`, refuse la base et les sauvegardes de la
  0.17.0.)*

## Ce que la release doit contenir (orchestrateur ; ne pas prescrire au développeur)

*Amendé le 2026-10-30 (C-v016-2-15) : la release est la **0.17.0**, première candidate `v0.17.0-rc1`. Le CHANGELOG
`[0.17.0]` et le § *Passer à la 0.17.0* sont posés par le développeur (AC-29) ; la release les relit.*

*Remédié à la validation P1 (F3) : la liste distingue ce qui précède la candidate de ce qui attend le tag final.
Constat, `docs/ci.md:165-169` : un tag `vX.Y.Z-rcN` ne pousse que `:X.Y.Z-rcN` (`:latest` ne bouge pas) et « les
candidates servent à jouer une recette sur l'image publiée avant de figer les textes du tag final » ;
`kesh-style.sty:72-74` porte `\keshVersion{0.16.1}`, `\keshReleaseDate{2026-10-29}` (date de la `0.16.1-rc1`),
`\keshTargetRelease{v0.16}` ; `git tag -l 'v0.16*'` rend `v0.16.0` et `v0.16.1-rc1` — le tag `v0.16.1` n'existe pas et
n'existera pas.*

**A. Avant `v0.17.0-rc1`**

1. **CHANGELOG `[0.17.0] — Non publié`** : relire ce que pose AC-29 (sections *Ajouté* #816, *Modifié*, *Corrigé* #811
   et #812 ; la phrase « aucune migration » supprimée ; la migration et le retour par la sauvegarde ; la note sur la
   candidate `0.16.1-rc1`). **Ne pas lancer `scripts/prepare-release.sh` pour la candidate** : il daterait l'entrée
   (« ne date pas deux fois ») et figerait la date de la candidate dans celle de la version finale. Son pré-vol des
   exemptions datées est le seul contrôle des `PerishableSince` : il n'est pas joué pour la candidate, **sans effet
   aujourd'hui** — les deux exemptions périssables ont pour borne `20260827000001` (`post_restore.rs:455`, `:478`), et
   tout tag nouveau porte à la fois la borne et la migration exemptée (validation P2, F10). Si Guy retient Q1
   (b), la migration et ses phrases tombent — à arbitrer avant le développement.
2. **Manuel d'administration**, § *Passer à la 0.17.0* et § *Sauvegarde et restauration* (`:2270`) : relus au `.tex`.
3. **Macros et URL inchangées pour la candidate ; PDF régénérés sur le texte de la 0.17.0** *(première rédaction :
   « Macros, URL et PDF : inchangés » ; corrigé à la validation P2, F3 — les PDF suivent leur `.tex` au commit du
   développeur, AC-29 point 5)* (décision de l'orchestrateur, comme pour la `0.16.1-rc1`) :
   `\keshVersion{0.16.1}` et les URL `raw.githubusercontent.com/…/v0.16.1` restent jusqu'au tag final, et la garde G19
   reste verte. ⚠️ Conséquence écrite : pendant la candidate, ces URL visent un tag `v0.16.1` qui n'existe pas (404), et
   la page de titre des manuels dit 0.16.1 alors qu'ils décrivent la 0.17.0 — à dire dans les notes de la candidate
   (point 4), avec le lien vers les fichiers du tag `v0.17.0-rc1`.
4. **Notes de pré-version de `v0.17.0-rc1`** : **avant la mise à jour, une sauvegarde complète** (base, `.keshbackup`,
   dossier des documents) de l'installation `0.16.1-rc1` ; le retour à la rc1 ne passe que par elle (`min_required`
   0.17.0) ; une sauvegarde `.keshbackup` de la 0.16.x s'importe dans la 0.17.0 ; la migration ; l'image à tirer :
   `gcorbaz/kesh:0.17.0-rc1` (`:latest` ne bouge pas : une installation qui suit `:latest` reste en 0.16.0) ; la
   `0.16.1-rc1` non poursuivie (une installation passée par elle se met à jour normalement, puis la rc1 refuse la base,
   sa version Cargo étant `0.16.1`) ; la recette Claude Code et les tests joués une seule fois, sur cette candidate ; les
   manuels encore titrés 0.16.1 (point 3).
5. **Rien de jetable sur la base du NAS** (C-v016-2-16) : la recette Claude Code, toute écriture de démonstration
   (`4321.09` compris), les clés de recette et les captures se jouent sur une installation **jetable** ; la base de
   Guy ne reçoit que la mise à jour elle-même, après sa sauvegarde. Aucune procédure de cette fiche n'écrit dans la base
   du NAS (les captures d'AC-25 se prennent sur une base de démonstration, l'E2E sur `kesh_e2e_v0162`).

**B. Au tag final `v0.17.0`**

6. `scripts/prepare-release.sh` (pré-vol, exemptions datées ; il trouve les crates déjà en `0.17.0`, saute le bump et
   date l'entrée du CHANGELOG du jour de la publication finale).
7. Macros `\keshVersion{0.17.0}`, `\keshReleaseDate{<date de publication Docker Hub du final>}`,
   `\keshTargetRelease{v0.17}`, URL `raw.githubusercontent.com/…/v0.16.1` portées à `v0.17.0` (garde G19) ; PDF des
   trois manuels régénérés.
8. README : la ligne de feuille de route est déjà passée à `v0.17.0` au développement (AC-29 point 8 de la 2a), sous
   un statut de candidate — la relire, et **basculer son statut à « ✅ Publiée »** au tag final (validation P3, 2a F1) ;
   site `website/` : `index.html` (`:70` porte `0.16.1`), `roadmap.html`, **`issues.html`** (liste des KF
   et de la feuille de route connue, liste de contrôle pré-release du `CLAUDE.md`, point 3) ; brochure
   (`marketing-brochure.tex:165`, `:388`, « Import natif des extraits CAMT.053 … et CSV ») si le positionnement le
   demande (F14 c).

**C. Communs**

9. PR de release : `closes #816` dans le **titre ou le corps**, à côté de `closes #811` et `closes #812`.
10. Décompte des issues au tag (objectif, non barrière) ; l'issue demandée ci-dessous comprise, et **#829** (les trois
    copies de l'extracteur, ouverte pendant la validation P3 — ne pas la rouvrir ; validation P4, 2a P7).
11. Issue à ouvrir (demandée par la fiche) : **« L'import d'un fichier ne signale pas une période déjà saisie à la main »**
   — enhancement ; C-v016-2-6.
12. Issue à ouvrir (validation P5 de la 2b, P7 — on trace toujours) : **« Clé i18n morte :
   `homepage-bank-no-transactions` »** — dette, P4 ; constat : la clé (« Aucune transaction importée ») figure aux quatre
   catalogues (`fr-CH:166`, et `:166` des trois autres) et n'est employée par aucun site (`grep -rn "no-transactions"
   frontend/src crates/*/src` hors `.ftl` : rien) ; à supprimer des quatre catalogues, ou à récrire si elle est
   réemployée (elle dirait alors « importée » d'un relevé saisi).

## Dev Notes (communes aux deux sous-fiches)

*(Les fichiers touchés, la § « Dérogation règle de splitting » de la première rédaction et la liste « À recontrôler »
sont passés aux sous-fiches ou retirés au découpage, validation P2.)*

### Le manuel (axe de revue)

Le PDF se contrôle aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`). La revue vérifie
que chaque promesse de la sous-section nouvelle est tenue par le code (pas d'« ignorer » ; équilibre exact ; confirmation
du solde de départ ; suppression puis ressaisie) et qu'aucune phrase ancienne ne dit encore « seul le CAMT.053 porte un
solde ».

### Propagation

Après chaque correctif de texte, greper la **valeur** : `deux formats`, `Seul le CAMT`, `Seuls les relevés`,
`CSV n'en a` *(au PDF aplati, l'apostrophe est typographique : greper `n’en a` ou `n.en a` en `grep -E` — validation P3,
2b F11)*, `aucune migration` (pour le signaler à la release, non pour le corriger), `bank_import.created` (les sites
qui le comptent comme seule trace de création d'un import : vues, tests, manuels).

### Références

- `CLAUDE.md` : *Migration breaking policy* (P1–P8), *Un appariement automatique propose, il ne crée jamais*,
  *Pattern batch* (sans objet : création unitaire, pas de `{ accepted, failed }`), *Règle de splitting préventif*,
  *Recette visuelle*, *Test Locally First*.
- `docs/agents-orchestration.md` § 2 à § 7.
- Stories : 8-1b, 8-3 (doublons), 27-1 (suppression d'un import), 27-3 (empreinte d'aperçu), 28-4a/29-4a (compte clos),
  25-6-a (tuile), 29-1a (K3).

## Questions au Project Lead

Tranchées au registre et soumises à Guy ; à la validation P2, aucune réponse n'est encore consignée. Les réponses
se consignent ici, datées, à leur arrivée ; une réponse qui change un choix appelle un amendement.

- **Q1 (C-v016-2-2, C-v016-2-15)** — Une migration qui relève `kesh_version_min_required` à 0.17.0 : la 0.16.x
  (candidate `0.16.1-rc1` comprise) refuse alors la base et les sauvegardes de la 0.17.0. Sans elle (option b), la
  0.16.0 redémarrerait mais rendrait une erreur sur les écrans d'import dès qu'un relevé saisi existe, et une sauvegarde
  de la 0.17.0 s'y importerait sans s'y lire. Retenu :
  la migration. D'accord, sachant que l'en-tête du CHANGELOG de la rc1 (« aucune migration ») change ?
- **Q2 (C-v016-2-7)** — Corriger une faute (date, libellé) après l'enregistrement : retenu, supprimer le relevé et le
  ressaisir. Faut-il, dès cette version, un bouton « Corriger » qui rouvre le relevé prérempli et le **remplace** en une
  fois ?
- **Q3 (C-v016-2-5)** — Un solde de départ différent de la clôture du relevé précédent : retenu, signalé et à confirmer.
  Préférez-vous qu'il bloque ?
- **Q4 (C-v016-2-3)** — Deux colonnes Débit / Crédit comme sur le papier (plutôt qu'un montant et un sens), et une
  colonne Contrepartie facultative (elle aide les propositions et les règles d'affectation) : cela vous convient-il ?
- **Q5 (C-v016-2-9)** — Une clé d'API `read-write` peut saisir un relevé, comme elle peut importer un fichier. À fermer ?
- **Q6 (C-v016-2-6)** — Si cette banque passe un jour au CAMT, l'import d'une période déjà saisie ne le signalera pas :
  la fiche demande une issue pour un avertissement à l'import. D'accord pour l'ouvrir (elle fera monter le décompte) ?
- **Q7 (C-v016-2-4)** — *(ajoutée à la validation P1, F12/R15 ; soumise à Guy, réponse attendue)* L'équilibre est
  **exact, bloquant, sans confirmation** : un relevé papier dont les soldes imprimés ne se retrouvent pas (page
  manquante, ligne illisible, report) ne s'enregistre pas tant que la ligne manquante n'est pas saisie. Recommandation
  retenue en attendant : garder ce choix (une confirmation « malgré l'écart » enregistrerait un relevé faux que la tuile
  et K3 compareraient ensuite aux livres). Si Guy tranche autrement, un amendement suivra.
- **Q8 (C-v016-2-3)** — *(ajoutée à la validation P1, F12 ; soumise à Guy, réponse attendue)* Devise : **CHF seule**,
  comme l'import ; si le compte sans relevé électronique est en devise étrangère, la saisie ne lui sert pas. Les comptes
  en devise relèvent de **#233**. Recommandation retenue en attendant : CHF seule. Si Guy tranche autrement, un
  amendement suivra.

## Change Log

- 2026-10-30 — Fiche créée (Opus 5.5, autonomie), registre C-v016-2-1 à 14, prompt de validation P1
  (`v016-2-validate-prompt-p1.md`). Prochaine étape : questions Q1–Q6 à Guy, puis validation P1 (Sonnet, lentilles R
  et F).
- 2026-10-30 — **Amendement (C-v016-2-15)** : la release devient la **0.17.0** (décision du Project Lead), première
  candidate `v0.17.0-rc1`. La migration relève `min_required` à `0.17.0` ; les dix crates passent à `0.17.0` dans le
  même commit (P2-bis) ; le tri des occurrences de `0.16.1` est prescrit (AC-2) ; le CHANGELOG `[0.16.1]` devient
  `[0.17.0]` et le § *Passer à la 0.16.1* devient *Passer à la 0.17.0*, posés par le développeur (AC-29) ; § *Ce que la
  release doit contenir*, M16, Q1 et la limite de la rc1 mis à jour. Occurrences de `0.16.1` laissées à dessein :
  l'historique (`c5101b07`, candidate `v0.16.1-rc1`, recette de la rc1, première rédaction de l'origine), les versions
  sources d'un remplacement prescrit, et `0.16.1` comme version antérieure refusée.
- 2026-10-30 — **Contrainte du Project Lead (C-v016-2-16)** : les données de l'installation `v0.16.1-rc1` du NAS se
  gardent. Ajoutés : AC-2-bis (montée depuis une base 0.16.x peuplée, imports CAMT et CSV, transactions et rapprochements
  relus à l'identique, en base et par le code ; la migration ne porte que le relèvement), AC-3-bis (une sauvegarde
  `.keshbackup` 0.16.x s'importe dans la 0.17.0, aucune table ajoutée), M17 et M18 ; AC-29 et les notes de la candidate
  disent la sauvegarde complète d'avant la mise à jour ; point 5-bis de la release : rien de jetable sur la base du NAS.
- 2026-10-30 — **Validation P1** (Sonnet, lentilles R et F, contexte frais ; rapports
  `kesh-gate-logs/v0162-validate-p1-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 12 MEDIUM, 17 LOW** (R : 5 MEDIUM, 10 LOW ;
  F : 7 MEDIUM, 7 LOW ; recoupements R3 = F4, R10 ≈ F5, R15 ≈ F12). Prémisses vérifiées au code avant chaque remède
  (lignes lues : `audit_label_registry.rs:105-116`, `:525-545` ; `audit_route_registry.rs:909`, `:915`, `:939`, `:944`,
  `:1324` ; `admin-manual.tex:2966`, `:2270`, `:2483`, `:2489`, `:82` ; `post_restore.rs:1214-1220` ; `balance.ts:23-35` ;
  `routes/bank_accounts.rs:176-187` ; `bank_accounts_repository.rs:2252`, `:2481` ; `user-manual.tex:49`, `:813-814`,
  `:2046-2052`, `:2097`, `:2126`, `:2216`, `:1925`, `:1958`, `:2029`, `:2462`, `:3265` ; `docs/ci.md:160-169` ;
  `kesh-style.sty:72-74` ; `textes_coherents.rs:1305-1350` ; `git tag -l 'v0.16*'`). Traitement :
  - **MEDIUM** — R1 appliqué (AC-15 : `ImportAuditAction` défini dans `bank_imports.rs`, entrée `SITES_INDIRECTS`) ;
    R2 appliqué (AC-15 : compteurs du registre des routes, phrase `admin-manual.tex:2966` ; fichiers touchés) ; R3 = F4
    appliqué (AC-21 : options de `isValidAmount`, solde vide, virgule) ; R4 appliqué (AC-4 r. 5 : `"50.000"`) ; R5 :
    pas de découpage (décision de l'orchestrateur), § *Dérogation règle de splitting* écrite (six modules) ; F1
    appliqué (AC-27 : inventaire complété, `:803` corrigé en `:813` ; `:2363` écarté après lecture, il ne nomme pas le
    fichier) ; F2 appliqué (AC-29 points 6 et 7) ; F3 appliqué (§ *Ce que la release doit contenir* en A / B / C ;
    macros et URL à 0.16.1 jusqu'au tag final, sur décision de l'orchestrateur, vérifié à `docs/ci.md` et à G19 ;
    `website/issues.html` ajouté) ; F5 (≈ R10) appliqué (AC-20 : correspondance des rangs) ; F7 appliqué (ensembles
    fermés de `field` et `reason`, table de libellés) ; F12 (≈ R15) : choix gardés, Q7 et Q8 ajoutées avec leur
    recommandation (décision de l'orchestrateur, questions soumises à Guy).
  - **LOW, tous appliqués** (remède mécanique) — R6 (compteur P7 21 → 22), R7 (255 caractères, non octets), R8 (encodage
    par longueur + test d'injectivité), R9 (trois cas « saisi puis importé »), R10 (avec F5), R11 (AC-26 : dates,
    compte propre à l'exécution), R12 (noms JSON d'AC-18 ; `statement_id` et champs non nuls semés en AC-2-bis ; AC-14
    au niveau dépôt), R13 (M19 à M22), R14 (limite du relevé voisin), R15 (avec F12), F6 (liste fermée des causes
    d'inactivité, cases), F8 (gardes i18n nommées), F9 (inventaire des chaînes de suppression et du détail), F10
    (captures), F11 (`api-external.md` `:243`, `:866`, `:878`, `sourceFormat`), F13 (voie normale, annulation d'abord),
    F14 (FAQ ; `README.md:49` à AC-29 ; brochure à la release).
  - **LOW non pris** : aucun.
  Choix nouveaux au registre : C-v016-2-17. Prochaine étape : validation P2 (`v016-2-validate-prompt-p2.md`, Opus,
  passe complète, lentilles R et F).
- 2026-10-30 — **Validation P2 et découpage** (Opus, lentilles R et F, contexte frais ; rapports
  `kesh-gate-logs/v0162-validate-p2-{R,F}.md`) : **0 CRITICAL, 0 HIGH, 10 MEDIUM, 17 LOW** (R : 6 MEDIUM, 9 LOW ; F :
  4 MEDIUM, 8 LOW ; recoupements R4 = F1, R5 ≈ F2, R6 = F4). Prémisses vérifiées au code avant chaque remède (lignes
  lues : `routes/bank_imports.rs:473`, `:603` ; `audit_label_registry.rs:20`, `:575-585` ; `onboarding_audit_e2e.rs:869-872` ;
  `routes/bank_accounts.rs:198-226` ; `kesh-import/src/camt053/mod.rs:412`, `:629-630` ; `crates/kesh-db/migrations.sha384`
  (présent) ; `audit_route_registry.rs:1310-1314` ; `journal_entries.rs:2276-2279` ; `migrations_upgrade_path.rs:568-571` ;
  `helpers/api-fixtures.ts:190` ; `admin_full_import_e2e.rs:3684` ; `admin_full_export_e2e.rs:536` ;
  `bank_account_closed_inventory.rs:628-634` ; `post_restore.rs:455`, `:478` ; `BankImportList.svelte:22-27` ;
  `csv-line-error-label.ts:20-25` ; `i18n-keys.test.ts:878-884` ; `README.md:235`). Aucune prémisse fausse ; aucun
  finding rejeté. Traitement :
  - **MEDIUM** — R1 appliqué (AC-15 : `"upload.xml"` à `PAS_DES_CODES`, « huit » → « neuf ») ; R2 appliqué (M22 : déclencheur
    `BEFORE INSERT ON audit_log`, patron `FAIL_TRIGGER`) ; R3 appliqué (type d'entrée en `Option<String>`, extracteur
    `400 VALIDATION_ERROR` au lieu du `422` d'Axum, limite de corps, test `"direction": "out"` et `lines` absent) ;
    R4 = F1 appliqué (la clé réelle : premier identifiant non vide ; en CAMT, une ligne saisie sans référence entre
    presque toujours deux fois — AC-27, § gardes, registre C-v016-2-6) ; R5 = F2 appliqué (tableau des propriétaires de
    chaque `0.16.1`, `admin-manual.tex:2486`, `:3220`, `:3238`, `:3349`, `:3351`, phrase de l'AC-2 renvoyée au tableau) ;
    R6 = F4 : **découpage** en `v016-2a` et `v016-2b` (décision de l'orchestrateur, C-v016-2-18 ; § *Découpage*, table de
    correspondance ; la § *Dérogation* retirée) ; F3 appliqué (les deux PDF FR régénérés au commit de leur `.tex`,
    macros inchangées ; point 3 de la release).
  - **LOW, tous appliqués** — R7 (`migrations.sha384`), R8 (`sans_ecriture` 110 → 112), R9 (prémisse MCP : `postes_ouverts`
    lit `bank_transactions` par `document_owners`), R10 (sémantique de `BinaryAhead`), R11 (`isPrimary: false`), R12
    (patrons de l'AC-3-bis), R13 (M16, M17 : bases éphémères seulement), R14 (`both_import_calls_map_the_archived_account`
    étendu), R15 (asymétrie client / serveur), F5 (`:232`, `:802`, `:821`, `:2095`, `:2292`), F6 (forme de
    `admin-manual.tex:2270`), F7 (`README.md:235` au développement, AC-29 point 8), F8 (`toFixed(2)`, `[...s].length`
    après `trim`, aperçu « réussi »), F9 (en-têtes de liste, tout le détail en clés, message « fichier » non affiché),
    F10 (pré-vol des exemptions : sans effet aujourd'hui, écrit), F11 (la story ne dit plus que Guy tient ses livres
    réels dans Kesh : « l'installation du NAS porte des données à garder »), F12 (renvois : registre jusqu'à
    C-v016-2-18, `CSV_LINE_ERROR_LABELS` et `CARDINALITES`, « point 5-bis » annoté au registre, intro des questions).
  - **LOW non pris** : aucun.
  Statut de cette fiche : `split`. Sous-fiches `v016-2a-releve-manuel-backend.md` et `v016-2b-releve-manuel-ecran.md`
  (`ready-for-dev`). Prochaine étape : validation P3, un prompt par sous-fiche (`v016-2a-validate-prompt-p3.md`,
  `v016-2b-validate-prompt-p3.md`, Sonnet, passe complète).
- 2026-10-30 — **Validation P3** des sous-fiches *(entrée ajoutée à la validation P4, 2a P1)* : remédiée aux Change Logs
  de `v016-2a-releve-manuel-backend.md` et `v016-2b-releve-manuel-ecran.md` ; les trois décisions de l'orchestrateur
  (règle unique `400` et extracteur partagé, épinglage Rust des `reason` et `field` avec test croisé, statut de candidate
  de `README.md:235`) sont au registre, **C-v016-2-19**. Cette fiche parente y a été touchée (§ *Ensembles fermés*,
  § *Type d'entrée*, § *Ordre des refus*, liste B de la release, § *Propagation*, table des mutations).
- 2026-10-30 — **Validation P4** des sous-fiches (ciblée, Opus) : remédiée aux Change Logs des deux sous-fiches ;
  ici, annotations de `:169`, `:185`, `:192` et #829 au point 10 de la liste C.
