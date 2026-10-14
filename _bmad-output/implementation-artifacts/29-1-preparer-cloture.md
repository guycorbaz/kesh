# Story 29.1 : Préparer la clôture d'un exercice — contrôles préalables, puis transitoires

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur
`16eeed21` (kickoff de l'Epic 29), worktree `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`. Validation P1
faite (Sonnet, R 0/0/9/9, F 0/0/3/4 — remédiée, Change Log) ; validation P2 faite (Opus, R 0/1/7/8, F 0/0/5/9 —
remédiée, Change Log ; **signal D5 levé et déclaré**) ; validation P3 faite (Sonnet, R 0/0/2/12, F 0/0/1/2 —
remédiée, Change Log ; **signal D5 au troisième tour, deux formes retirées** : M-b13, volets (c) et (d) de T-a2) ;
passe ciblée P4 à lancer (`29-1-validate-prompt-p4-ciblee.md`, Haiku, une lentille).
**2026-10-14 — 29-1a développée** (`bmad-dev-story`, Opus 5.5) : clé `29-1a-controles-prealables` en **`review`** ;
dernier commit de code `7c643eba` ; revue de code P1 à lancer (`29-1a-review-prompt-p1.md`, Sonnet, lentilles B, E,
A). La 29-1b reste `ready-for-dev`.

⛔ **Deux stories, une fiche, deux PR successives** (C-29-1-1, précédents C-26-2-15 et C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`29-1a-controles-prealables`** | lecture `closing_checks` (kesh-db), route `GET /api/v1/fiscal-years/{id}/closing-checks`, panneau *Préparer la clôture* et rappel dans le dialogue de clôture, écart du relevé servi au tableau de bord, i18n ×4, manuel utilisateur (PDF), `api-external.md`, CHANGELOG | AC1 à AC9 | **`refs #232`** — ⛔ jamais `closes` |
| **`29-1b-contre-passation-transitoires`** | contre-passation datée du premier jour de l'exercice suivant (kesh-db, route), bouton sur la fiche d'une écriture, i18n ×4, manuel utilisateur, E2E de parcours complet, `api-external.md`, CHANGELOG | AC10 à AC18 | **`closes #232`** |

*(Validation P2, 2026-10-14, décision de l'orchestrateur : la répétition à blanc au manuel d'administration — ancienne
AC19 — est **retirée** de cette fiche ; elle devient la **Story 29-7** (doc), écrite en l'exécutant sur la station.
C-29-1-23.)*

**Ordre** : la 29-1a d'abord ; la 29-1b part de `origin/main` **après** la fusion de la 29-1a (le parcours E2E de
l'AC18 passe par le panneau des contrôles). La clé mère `29-1-preparer-cloture` passe au statut `split`. Une release
entre les deux est **possible** (la 29-1a se suffit : elle ne fait que lire) mais n'est pas prévue — la cible du
bouclement est la **v0.14.0** (fichier d'epic, § *Releases*).

⛔ **Mots-clés de fermeture** (rétrospective 28, § 4.4, B4) : aucun `closes`/`fixes` dans un **message de commit**,
même en prose, même au futur — tous les commits des deux stories portent `refs #232`. Le `closes #232` ne s'écrit que
dans le **titre et le corps de la PR de la 29-1b** ; le message de squash de la PR de la 29-1a se relit avant merge.

## Story

**As a** personne qui tient ses livres dans Kesh (personne physique, indépendant, petite structure) et s'apprête à
clôturer son premier exercice,
**I want** voir, avant de clôturer et à tout moment de l'année, ce qui reste à faire ou paraît anormal dans l'exercice
— transactions bancaires non rapprochées, relevé de fin d'année manquant ou en écart, brouillons non comptabilisés,
écritures incohérentes héritées —, puis passer mes écritures de régularisation (transitoires) et les faire
contre-passer par Kesh au premier jour de l'exercice suivant,
**so that** je clôture un exercice dont je sais qu'il est complet, sans tableur à côté, et que mes régularisations de
fin d'année s'annulent d'elles-mêmes au bon jour, dans le bon exercice.

Issue : [#232](https://github.com/guycorbaz/kesh/issues/232) (CR, `enhancement`, **P1**, 2026-07-09). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § 29-1. Cible : **v0.14.0**.

## Inventaire de l'existant — ce que #232 croit, et ce que le code fait (C-29-0-3)

Le texte de #232 date du **2026-07-09**. Chaque phrase y est confrontée au code de `16eeed21`.

### Ce qui existe déjà

| #232 dit | Le code fait | Ancre |
|---|---|---|
| « `FiscalYearStatus` gère Open/Closed (immutabilité) » | Vrai, et davantage : la clôture est un **verrou** posé **dans l'ordre** (refus `EARLIER_FISCAL_YEAR_OPEN` tant qu'un antérieur est ouvert), **réouvrable** par un administrateur avec motif tracé, en ordre inverse (LIFO) ; un exercice ne se crée pas avant un exercice clos. Rejouée sur interblocage. | `crates/kesh-db/src/repositories/fiscal_years.rs` `close` (l. 1054), `reopen` (l. 1197), doc de module l. 11-40 ; `crates/kesh-api/src/routes/fiscal_years.rs` l. 296-347 |
| « pas d'écritures de clôture automatiques » | **Choix d'architecture, non un manque** : modèle « temps réel virtuel » (Story 14-1, décision de Guy du 2026-07-21). La clôture **ne génère aucune écriture** ; le résultat de l'exercice et le résultat reporté sont **deux lignes calculées** du bilan ; le compte 2979 (`CurrentYearResult`) est **non imputable**. | `crates/kesh-report/src/balance_sheet.rs` doc l. 1-60 ; `_bmad-output/implementation-artifacts/14-1-cloture-report-des-soldes.md` § *Décision d'architecture*, AC E ; `crates/kesh-db/src/repositories/accounts.rs:3082` |
| « pas de report à nouveau (soldes d'ouverture de N+1) » | **Faux depuis la v0.8.0** : les comptes de bilan sont **cumulés depuis l'origine** (`entry_date <= date d'arrêté`, tous exercices), le bilan de N+1 sans écriture équilibre et reflète N. | `balance_sheet.rs` doc l. 3-31 ; manuel utilisateur § *Clôture d'un exercice* l. 740 |
| (non dit) reprise d'une ancienne comptabilité | Écran *Soldes de départ* : écriture d'ouverture équilibrée, complément des comptes oubliés. | `crates/kesh-db/src/repositories/opening_complement.rs`, `journal_entries::create_opening_entry` (l. 651) ; manuel § *Reprise de comptabilité* l. 775 |
| (non dit) correction d'une écriture d'un exercice clos | **Contre-passation**, datée **du jour**, dans l'exercice ouvert qui couvre ce jour ; origine et inverse liées (`reverses_entry_id`, `UNIQUE`) ; lettrage automatique des lignes libres. | `journal_entries.rs` `reverse_in_tx_inner` (l. 2595), datation l. 2728-2731 ; migration `20260828000002_journal_entries_reversal.sql` |
| (non dit) figer une période en cours d'année | **Verrou de période** `companies.books_locked_through` (borne inclusive), refusé à la date du jour ou au futur. | `companies.rs` `lock_books` (l. 925) ; `journal_entries.rs` l. 325, 403-409 |
| (non dit) confronter banque et livres | Le **tableau de bord** confronte le solde comptable du compte lié, à la date du dernier relevé CAMT, au solde de clôture de ce relevé, corrigé des décalages date de valeur / comptabilisation. | `bank_accounts.rs` `list_by_company_with_balances` (l. 1392) ; `frontend/src/lib/features/homepage/homepage.ts:63` |

**La clôture de l'exercice 2026 est donc déjà possible**, et le bilan de 2027 sera juste sans aucune écriture. Ce
qui manque l'outille et la sécurise.

### Ce que #232 demande encore réellement

| Demande de #232 | Verdict | Où |
|---|---|---|
| Contrôles préalables : **équilibre** | Retenu, sous la forme qui a un sens : une écriture déséquilibrée ne peut naître d'aucun chemin d'écriture (vérifiée après l'INSERT, `journal_entries.rs:517-533`) — **mais** aucun `CHECK` ne la garantit en base (`20260412000001_journal_entries.sql:12`) et la restauration insère les lignes telles quelles (`kesh-db/src/backup.rs:633`). Contrôle d'**anomalie héritée**. | 29-1a, K6 |
| Contrôles préalables : **TVA réconciliée** | **Écarté** : report assumé des P1 de la TVA (CLAUDE.md § *Priorités des défauts*) ; le seul utilisateur réel n'est pas assujetti. Le rapport TVA porte déjà un statut d'écart (`kesh-report/src/vat_report.rs:190`). | noté sur **#398** (jalon « Vague 2 »), sans issue neuve |
| Contrôles préalables : **comptes d'attente soldés** | **Prémisse sans objet au code** : Kesh n'a **ni rôle, ni drapeau, ni compte d'attente** (aucun dans `AccountRole`, `kesh-core/src/chart_of_accounts/mod.rs:51-76`, ni dans les trois plans `kesh-core/assets/charts/*.json`, ni dans la réconciliation). Rien à contrôler sans inventer un rôle (migration **breaking** : un binaire antérieur ne décode pas un rôle inconnu). | écarté, C-29-1-4 |
| Génération des **écritures de clôture** (virement du résultat au bilan) | **Écarté par conception** (14-1) : le modèle virtuel calcule ce virement ; une écriture qui le doublerait fausserait le bilan, et 2979 est non imputable. | C-29-1-2 |
| **Report à nouveau automatique** | **Existe** (calculé). | — |
| **Aide aux transitoires** avec contre-passation automatique à l'ouverture de N+1 | Retenu : la contre-passation existe mais se date **du jour** ; il manque la contre-passation **au premier jour de l'exercice suivant**. | 29-1b |
| Intégration des **amortissements** (#222) | **Hors périmètre** (consigne de l'orchestrateur ; #222 / #408). | — |
| (candidat de l'orchestrateur) écriture d'**affectation du résultat** | **Écarté** (C-29-1-2) : sans objet pour une personne physique ou un indépendant (le résultat s'accumule aux fonds propres, ce que le report calculé montre déjà) ; pour une société, l'affectation votée par l'assemblée est une écriture **manuelle** déjà possible (débit du compte physique `RetainedEarnings`, crédit dividendes ou réserve) dont seule la **présentation** au bilan est perfectible. | **#667** (ouverte le 2026-10-14, `enhancement` ; priorité à poser par l'orchestrateur) |

⚠️ **Prémisses de #232 à corriger** (à commenter sur l'issue par l'orchestrateur) : le report à nouveau existe ; les
écritures de clôture sont écartées par conception ; les comptes d'attente n'existent pas dans Kesh ; le contrôle TVA
est reporté avec la vague 2.

## Périmètre minimal — ce qui ferme #232, et ce qui en est exclu

**Dedans** :

1. **29-1a** — une lecture **en lecture seule**, par exercice, des contrôles K1 à K7 (AC1), servie à tout rôle
   authentifié, affichée dans un panneau de l'écran des exercices et rappelée dans le dialogue de clôture ; elle ne
   **bloque rien** (la clôture garde son unique refus existant, K1) ;
2. **29-1b** — un geste *Contre-passer au début de l'exercice suivant* sur une écriture : même machinerie que la
   contre-passation (liaison, lettrage, refus), datée du **premier jour de l'exercice qui suit** celui de l'écriture ;
3. le manuel utilisateur, l'i18n quatre locales, `api-external.md`, le CHANGELOG ; un E2E de parcours complet.

**Dehors**, et pourquoi :

| Exclu | Raison |
|---|---|
| Bloquer la clôture sur un contrôle | « Signaler, jamais bloquer » (CLAUDE.md § *Un appariement…*, corollaire) ; la clôture se défait (réouverture motivée) ; et surtout **aucun verrou ni aucune lecture ne s'ajoute à `fiscal_years::close`** — rien à relire sur l'ordre des verrous ni la naissance de la vue (B5). C-29-1-3. |
| Marquer une écriture « transitoire » (colonne) et générer les contre-passations **à la clôture** | Écrirait sous les verrous de la clôture (cycle neuf avec les écrivains de N+1) et exigerait une migration ; « un appariement propose, il ne crée jamais » vaut aussi pour une génération en masse. C-29-1-8. |
| Contrôle « transitoires non contre-passées » | Sans marque, rien ne distingue une écriture de régularisation. Angle mort assumé, écrit au manuel (AC17). C-29-1-14. |
| Contre-passation à **date libre** | Le verrou de période repose sur « la contre-passation corrige depuis le présent » (manuel l. 612-621) ; une date libre rouvrirait la question. Seule la date « premier jour de l'exercice suivant » est ajoutée. C-29-1-9. |
| Contrôles TVA, comptes d'attente, verrou de période vs fin d'exercice, postes ouverts débiteurs/créanciers, factures fournisseur ouvertes | C-29-1-4 (raisons une par une). |
| Écriture d'affectation du résultat ; écritures de clôture | C-29-1-2. |
| Amortissements (#222, #408) | Consigne de l'orchestrateur. |
| Manuels DE / IT / EN | `docs/manual/{de,en,it}/` ne portent qu'un `README.md`. |
| Répétition à blanc au manuel d'administration | Retirée en validation P2 (décision de l'orchestrateur) : chaque correction de sa marche y a produit un défaut, dont un qui pouvait écraser la base de production. **Story 29-7** (doc), écrite en l'exécutant réellement. C-29-1-23. |
| Corriger la capture `% TODO capture` du manuel l. 734 | Hors périmètre ; les captures de la story vont hors dépôt (convention 28-4b). |

**Décompte des modules** (§ *Règle de splitting préventif*) — ensemble : `kesh-db` (`closing_checks` neuf,
`bank_accounts`, `fiscal_years`, `journal_entries`), `kesh-api` (`routes/fiscal_years`, `routes/journal_entries`, `lib.rs`,
`errors.rs`), `kesh-i18n`, frontend (`features/fiscal-years`, `routes/settings/fiscal-years`,
`routes/journal-entries/[id]`, `features/journal-entries`), `docs/manual` : **plus de cinq** → découpage (C-29-1-1).
*(Liste complétée par le ⚠️ ci-dessous : l'écart servi au tableau de bord ajoute `routes/bank_accounts`,
`features/bank-accounts` et `features/homepage`, sans crate neuve — validation P3, R11.)*
**29-1a** : `kesh-db`, `kesh-api`, `kesh-i18n`, frontend, `docs/manual` = **cinq** au grain de la crate, la borne.
**29-1b** : les mêmes cinq. Pas de 29-1c : l'affectation du résultat est écartée.

⚠️ **Ce décompte tient au grain de la crate, et à ce grain seulement** (P2 R8). Au grain du `CLAUDE.md`
(`kesh-api/routes/invoices`, `frontend/src/features/invoices`), la 29-1a touche déjà plus de cinq sous-modules :
`kesh-db` (`closing_checks`, `bank_accounts`, `fiscal_years`), `kesh-api` (`routes/fiscal_years`,
`routes/bank_accounts`), frontend (`features/fiscal-years`, `routes/settings/fiscal-years`, `features/bank-accounts`,
`features/homepage`). L'écart servi au tableau de bord (AC3, C-29-1-25) n'ajoute **aucune crate** ; il ajoute
`routes/bank_accounts` et `features/bank-accounts` au grain fin, en ajout d'un champ ou en lecture. Dispersion
**déclarée au Project Lead** avec le signal D5 de la validation P2 (Change Log).

## Critères d'acceptation

### Partie a — `29-1a-controles-prealables`

**AC1 — Les contrôles, l'ensemble clos.** Une fonction `kesh_db::repositories::closing_checks::for_fiscal_year(conn,
company_id, fiscal_year_id) -> Result<ClosingChecks, DbError>` lit **elle-même** l'exercice `Y`, sur l'instantané,
par une constante de son `ALL_SQL` (`FISCAL_YEAR_SQL`, `WHERE id = ? AND company_id = ?`) — Y n'est donc jamais lu à
un autre instant que les contrôles, et `find_by_id_in_company`, qui prend un `&MySqlPool`, n'est pas appelée (P2 F L6) ;
absent ou d'une autre société → `DbError::NotFound`. Elle rend, pour `Y` (`start_date`, `end_date`), **exactement**
les sept contrôles suivants, dans cet ordre, chacun avec un `code` stable (SCREAMING_SNAKE_CASE), une
`severity` et un `status` `ok` | `attention` :

| Code | Sévérité | `attention` quand… | Données rendues |
|---|---|---|---|
| **K1 `EARLIER_FISCAL_YEAR_OPEN`** | `blocking` | un exercice de la société dont `start_date < Y.start_date` est `Open` | le **plus ancien** (`id`, `name`) — lu par `fiscal_years::find_earliest_earlier_open` (neuve, sans verrou), dont le texte est **celui** de l'étape (d) de la clôture moins `FOR UPDATE` (constante partagée, AC3-bis) |
| **K2 `PENDING_BANK_TRANSACTIONS`** | `warning` | une `bank_transactions` de la société au statut `pending` a `booking_date BETWEEN Y.start_date AND Y.end_date` | par compte bancaire : `bankAccountId`, nom, `count`, dont `rejectedCount` (`auto_match_rejected_at IS NOT NULL`) ; total |
| **K3 `BANK_STATEMENT_AT_YEAR_END`** | `warning` | un compte bancaire **examiné** (ci-dessous) **et vérifiable** a l'un des motifs `MISSING`, `ENDS_BEFORE_YEAR_END` ou `GAP` (définitions et précédence ci-dessous) | `accounts` : par compte en défaut, `bankAccountId`, nom, motif, `statementDate`, `statementClosingBalance`, `ledgerBalanceAtStatement`, `gap` ; `notCheckable` : par compte examiné mais **non vérifiable**, `bankAccountId`, nom, motif `NOT_COMPARABLE` ou `NO_BALANCE_STATEMENT` — **sans effet sur le statut** (C-29-1-16) |
| **K4 `DRAFT_INVOICES`** | `info` | une facture client `status = 'draft'` a `date BETWEEN Y.start_date AND Y.end_date` | `count`, et au plus 20 `{id, label}` (numéro absent d'un brouillon : le libellé est celui de la liste des factures) |
| **K5 `IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE`** | `info` | une `imported_supplier_invoices` de la société est `to_complete` | `count` — ⚠️ **non borné à l'exercice** : la table n'a pas de date métier, seulement `created_at` (écrit au manuel) |
| **K6 `INCONSISTENT_ENTRIES`** | `anomaly` | une écriture **de Y** a moins de deux lignes, ou `SUM(debit) <> SUM(credit)` ; **ou** une écriture de Y a `entry_date` hors `[Y.start_date, Y.end_date]` ; **ou** une écriture d'un **autre** exercice a `entry_date` dans `[Y.start_date, Y.end_date]` | `count` = nombre d'écritures **distinctes** en défaut ; `countByReason` par motif (`UNBALANCED`, `TOO_FEW_LINES`, `DATE_OUTSIDE_ITS_YEAR`, `OTHER_YEAR_DATED_INSIDE` — une écriture peut compter sous deux motifs, la somme peut donc dépasser `count`) ; au plus 20 écritures **dédupliquées**, triées `entry_date ASC, id ASC`, chacune `{id, entryNumber, fiscalYearName, reasons: […]}` |
| **K7 `NEXT_FISCAL_YEAR`** | `info` | aucun exercice de la société n'a `start_date > Y.end_date` | `{id, name, startDate, status}` du suivant s'il existe — lu par `fiscal_years::find_next_after` (neuve, sans verrou, `ORDER BY start_date ASC LIMIT 1`), **la même** que la 29-1b emploie à l'étape (4) de la contre-passation (C-29-1-18) |

**K3 — comptes examinés** (C-29-1-16) : tout compte bancaire de la société qui n'est **pas archivé** (il n'a aucune
transaction, `archive_for_company`), **pas clos avant le début de Y** (`closed_on IS NULL OR closed_on >=
Y.start_date`) **et a une activité dans Y** — au moins une `bank_transactions` dont `booking_date` est dans
`[Y.start_date, Y.end_date]`, **ou** au moins un `bank_imports` portant un solde (`closing_balance IS NOT NULL`) dont
`period_to` est dans cet intervalle, **ou** au moins une ligne d'écriture sur le compte de grand livre lié
(`journal_entry_lines.account_id = ba.journal_account_id`) dont l'écriture (`journal_entries`, même `company_id`) a
`entry_date` dans cet intervalle (C-29-1-24, P2 R3 = F2 : une écriture passée à la main sur le compte lié, sans
mouvement bancaire importé, fait quitter au solde comptable le solde bancaire — c'est précisément le relevé de fin
d'année non confirmé que K3 doit signaler). Trois `EXISTS` séparés, par compte bancaire : jamais `bank_transactions`
dans le même `FROM` que les lignes du grand livre (forme de `list_by_company_with_balances`, doc l. 1387-1391). Un
compte sans compte lié n'a pas de troisième critère (il ne passe que par les deux premiers, puis `NOT_COMPARABLE`).
Le critère d'activité écarte, sans autre règle, le compte ouvert après Y et le compte sans mouvement, ni relevé, ni
écriture dans Y. ⛔ **Pas** de critère sur `bank_accounts.created_at` : une reprise de comptabilité crée le compte en
N+1 et y importe les relevés de N — ce compte doit être examiné pour N.

**K3 — motif de chaque compte examiné**, le **premier** qui s'applique dans cet ordre (précédence écrite et testée,
T-a5) :

| Ordre | Motif | Quand | Liste | Effet sur le statut |
|---|---|---|---|---|
| 1 | `NOT_COMPARABLE` | sans compte lié, ou compte lié partagé (`sharing > 1`, comme le tableau de bord — lu sur `BalancesRow::sharing`, AC3) | `notCheckable` | aucun |
| 2 | `NO_BALANCE_STATEMENT` | aucun `bank_imports` du compte ne porte de solde, **toutes dates** (compte alimenté par CSV seulement) — lu par un **quatrième `EXISTS`**, **sans borne de date**, rendu par la requête de population de `closing_checks` à côté de ses trois `EXISTS` d'activité, sans entrer dans le critère d'examen (constante de `ALL_SQL` : `bank_imports` du compte, même `company_id`, `closing_balance IS NOT NULL`) ; la lecture bornée de l'AC3 ne le dit pas, car elle rend `None` aussi pour un compte dont tous les relevés à solde sont **postérieurs** à Y — celui-là est `MISSING` (P3 R8) | `notCheckable` | aucun |
| 3 | `MISSING` | le relevé de référence (AC3, borne `Y.end_date`) est absent, ou son `period_to < Y.start_date` | `accounts` | `attention` |
| 4 | `ENDS_BEFORE_YEAR_END` | son `period_to < MIN(Y.end_date, closed_on)` (la clôture d'un compte clos pendant Y est la borne de ce motif, comparée **en Rust** — la borne SQL reste `Y.end_date` pour tous, R18) | `accounts` | `attention` |
| 5 | `GAP` | `bank_accounts::statement_gap(&balances)` (AC3) n'est pas `None` | `accounts` | `attention` |
| — | (aucun) | sinon | — | — |

`status = attention` ⇔ `accounts` n'est pas vide ; `notCheckable` s'affiche (« non vérifiable : … ») sans changer le
statut.

`status = ok` ⇔ la condition d'`attention` est fausse ; le compteur ou la liste sont alors vides. K1 à K7 sont
**l'ensemble clos** : un contrôle neuf est un changement de cette fiche, et le test T-a1 compare la liste des codes
rendus à cette table (ordre compris).

**AC2 — Une seule vue, aucune écriture, aucun verrou.** `for_fiscal_year` s'exécute sur la connexion d'un
`kesh_db::snapshot::ReadSnapshot` (Story 28-3, `snapshot.rs:61`), ouverte par la route et close par `finish()` :
toutes les lectures voient la base au même instant ; l'instantané est `READ ONLY` (toute écriture refusée par le
serveur) ; **aucune** lecture verrouillante (`FOR UPDATE`, `LOCK IN SHARE MODE`, `FOR SHARE`). Garde statique
T-a2, **inversée** (C-29-1-26, P2 R4 = F1) : au lieu d'énumérer les formes interdites — énumération ouverte, que le
turbofish `query_as::<_, T>(…)`, l'appel coupé par rustfmt, `r#"…"#`, `concat!` ou `format!` contournaient —, elle
inventorie les **sites qui ne résolvent pas** (§ *Inventorier les sites NON RÉSOLUS*). Elle lit le source de
`closing_checks.rs` **hors `#[cfg(test)]`** (tout ce qui précède le premier `#[cfg(test)]`) et vérifie :

- **(a) tout appel de requête a pour premier argument une constante de `ALL_SQL`.** Le source est d'abord mis sur une
  seule ligne (sauts de ligne remplacés par des espaces, ce qui couvre l'appel coupé par rustfmt) ; chaque occurrence
  du mot `query`, `query_as` ou `query_scalar` suivi (après espaces) de `::<` ou de `(` est un **site** ; un
  turbofish se saute en **équilibrant** `<` et `>` (une signature `(i64, Option<String>)` en contient plusieurs) ;
  après la parenthèse ouvrante et les espaces, le site doit porter un identifiant `[A-Z][A-Z0-9_]*_SQL` dont le nom
  figure parmi les constantes énumérées par `const ALL_SQL: &[&str]`. Tout autre site — littéral, `&format!(…)`,
  `concat!(…)`, variable — est un **non résolu**, et le test le nomme en rougissant. Le nombre de `const …_SQL: &str`
  du module égale `ALL_SQL.len()` (inventaire clos) ;
- **(b) aucun mot de verrou dans le source décommenté.** Les lignes dont le premier non-blanc est `//` (donc `///` et
  `//!`) retirées, le texte mis en majuscules ne contient ni `FOR UPDATE`, ni `LOCK IN SHARE MODE`, ni `FOR SHARE` —
  la doc de module, qui dit justement « aucun `FOR UPDATE` », ne le fait donc pas rougir. (`format!` et `concat!` ne
  sont pas interdits dans tout le module — un libellé peut s'y construire — : comme argument d'une requête, ils sont
  des non résolus du volet (a), et une `const …: &str` ne peut pas recevoir un `format!`).

La garde n'a **que ces deux volets** (validation P3, décision de l'orchestrateur, C-29-1-26). ⚠️ **Angle mort
assumé** : les fonctions de dépôt **externes** que `closing_checks` appelle — `bank_accounts::balances_by_bank_account`,
`bank_accounts::statement_gap`, `bank_accounts::is_shared` (AC3), `fiscal_years::find_earliest_earlier_open` et
`fiscal_years::find_next_after` (AC3-bis) — et le texte SQL qu'elles exécutent **ne sont pas lus par T-a2** : ils sont
**nommés dans le doc-comment du module** `closing_checks` (liste écrite à la main) et **vérifiés en revue de code**
(chacune sans verrou, sur l'exécuteur reçu). Aucun test ne le garantit.

Le `READ ONLY` de l'instantané couvre les écritures ; le comportement de MariaDB 10.11 devant un `SELECT … FOR
UPDATE` dans une transaction `READ ONLY` **n'a pas été établi** — d'où la garde statique, qui ne dépend pas de lui.
Le dev le **mesure** en T0 (Ta1, P2 F L3) et consigne l'issue, erreur ou succès, au Dev Agent Record ; la garde
reste quelle que soit l'issue.

**AC3 — K3 réutilise la requête du tableau de bord, bornée.** La requête de `list_by_company_with_balances`
(`bank_accounts.rs:1392`), seule à savoir corriger l'écart des décalages date de valeur / comptabilisation, est
**extraite**, avec son passage de la ligne au résultat, en trois éléments de `bank_accounts.rs` :

- `const BALANCES_SQL`, **privée** (son `pub(crate)` ne servait que le volet (d) de T-a2, retiré en validation P3) —
  le texte **unique** ; la borne s'ajoute **dans** la table dérivée `st`, **avant**
  le `ROW_NUMBER()` (sinon `rn = 1` désignerait le relevé postérieur, puis serait filtré) : `WHERE company_id = ? AND
  closing_balance IS NOT NULL AND (? IS NULL OR period_to <= ?)`. La liaison s'insère au **troisième** rang, entre
  `agg` et `shared` : `.bind(company_id)` ×2, `.bind(company_id)` (`st`), `.bind(bound)` ×2, `.bind(company_id)` ×2
  — ordre écrit en commentaire ligne à ligne, comme aujourd'hui (F6) ;
- `pub(crate) async fn balances_by_bank_account<'e, E: Executor<'e, Database = MySql>>(executor, company_id,
  statement_bound: Option<NaiveDate>) -> Result<HashMap<i64, BalancesRow>, DbError>` — `BalancesRow` passe
  **`pub(crate)`** (une `pub(crate) fn` qui rend un type privé déclenche `private_interfaces`, rouge sous
  `clippy -D warnings` — P2 R11) ;
- `fn balances_from_row(journal_account_id: Option<i64>, row: Option<&BalancesRow>) -> BankAccountBalances` — le
  `match` actuel (partage, `at + plus − minus`, compte sans lien), **déplacé**, non recopié ;
- `pub(crate) fn is_shared(row: Option<&BalancesRow>) -> bool` — le partage du compte lié lu sur `BalancesRow::sharing`
  (`sharing > 1`), à côté de `balances_from_row` et par le même critère : K3 en tire le motif `NOT_COMPARABLE`, que
  `BankAccountBalances` ne distingue pas d'un compte sans relevé (`ledger_balance_at_statement = None` dans les deux
  cas, P2 R11).

`list_by_company_with_balances` les appelle avec `None` — **résultat inchangé**, prouvé par les tests existants de
`crates/kesh-db/tests/bank_account_statement_gap.rs` et de `crates/kesh-api/tests/bank_accounts_e2e.rs` (côté route :
`list_bank_accounts_exposes_the_last_statement`, `deleting_a_statement_recomputes_the_last_statement`…) restés verts
**sans modification** (P2 R10 : `bank_accounts_repository.rs` ne l'appelle pas) — et `closing_checks` avec
`Some(Y.end_date)` pour **tous** les comptes (la
clôture d'un compte n'intervient qu'en Rust, motif 4 de K3). La correction des décalages date de valeur /
comptabilisation est relative à `st.period_to`, donc juste pour une borne antérieure. ⛔ Pas de copie de la requête
ni du `match` (règle DRY du dépôt).

**L'écart, une seule définition côté serveur** (C-29-1-17) : `pub fn statement_gap(b: &BankAccountBalances) ->
Option<Decimal>` dans `bank_accounts.rs` — `None` si `ledger_balance_at_statement` ou `statement_closing_balance`
manque, sinon l'écart des **deux termes arrondis au centime** par `kesh_core::types::Money::round_to_centimes`
(`MidpointAwayFromZero`, l'arrondi du dépôt), `None` s'il est nul. C'est la règle de `statementGap`
(`frontend/src/lib/features/homepage/homepage.ts:63`, `toCentime` = `round(2, roundHalfUp)`, demi loin de zéro) : les
lignes du grand livre sont en `DECIMAL(19,4)`, le relevé en `DECIMAL(18,2)`, et sans cet arrondi K3 signalerait un
écart de `0.0030` que le tableau de bord n'affiche pas. Le **panneau ne recalcule rien** : il affiche le `gap` et le
statut rendus par le serveur.

**Le tableau de bord lit le même écart, servi par le serveur** (C-29-1-25, décision de l'orchestrateur, P2 R8) — une
seule définition, en Rust ; la seconde, en TypeScript, disparaît :

- `crates/kesh-api/src/routes/bank_accounts.rs` : `BankAccountWithBalance` (l. 166-183) gagne `statement_gap:
  Option<Decimal>` (doc : « `bank_accounts::statement_gap`, au centime ; `null` si nul ou non calculable »), rempli
  dans le handler de liste (l. 541-548) par `bank_accounts::statement_gap(&balances)` ;
- `frontend/src/lib/features/bank-accounts/bank-accounts.api.ts` : `statementGap: string | null` dans
  `BankAccountSummary`, `statementGap?: string | null` dans `BankAccountSummaryRaw` (absent des réponses de mutation,
  comme les champs du relevé), `statementGap: raw.statementGap ?? null` dans `parseBankAccount` ;
- `frontend/src/lib/features/homepage/homepage.ts` : `statementGap(account)` rend `account.statementGap === null ?
  null : new Big(account.statementGap)` — même signature, `BankAccountsCard.svelte` **inchangé** ; `toCentime`
  **supprimé** (son seul appelant est `statementGap`) ; le doc-comment renvoie à `bank_accounts::statement_gap` ;
- `homepage.test.ts` : les quatre cas d'arrondi (l. 57-80) **partent en Rust** — T-a14 les reprend à l'identique —,
  remplacés par un test de lecture du champ (`null` → `null`, `"-200.00"` → `Big(-200)`) ; les fabriques de test qui
  construisent un `BankAccountSummary` (`homepage.test.ts`, `HomepageCards.test.ts`, `bank-accounts.api.test.ts`,
  `bank-accounts-page.test.ts`, `BankAccountJournalLinkForm.test.ts`, `payment-batches-page.test.ts` — liste
  recomptée au dev par `grep -rln "ledgerBalanceAtStatement" frontend/src` et nommée au Dev Agent Record) gagnent
  `statementGap: null` là où le type l'exige ; ⚠️ **un cas de `HomepageCards.test.ts` change d'attente**, non de
  fabrique seulement (P3 R3) : « affiche le relevé, et l'écart seulement s'il est non nul » (l. 166-188 à
  `16eeed21`) attend `homepage-bank-gap-1` d'un écart que la carte calculait (`800.0000` contre `1000.00`) — son
  premier rendu reçoit `statementGap: '-200.00'`, le second (écart nul) garde `statementGap: null` ;
- `crates/kesh-api/tests/bank_accounts_e2e.rs` : un test **neuf** vérifie `statementGap` dans `GET
  /api/v1/bank-accounts` (écart réel en chaîne, `null` sans relevé) — les tests existants restent sans modification.

Les cas du test Rust de `statement_gap` (T-a14) sont **ceux de `homepage.test.ts`** l. 57-80 à `16eeed21`
(`800.0000` / `1000.00` → écart de `-200` ; `100.0049` / `100.00` → aucun écart ; `100.005` / `100.01` → aucun ;
`-100.005` / `-100.01` → aucun). `docs/api-external.md` ne documente pas les champs du relevé (`ledgerBalanceAtStatement`
absent) : rien à y ajouter. Frontend et backend partent dans la même image : aucun décalage de version possible.

**AC3-bis — Les lectures d'exercice partagées, sans copie** (`fiscal_years.rs`, C-29-1-18) :

- `const EARLIER_OPEN_SQL` (privée, comme `NEXT_AFTER_SQL` : leur `pub(crate)` ne servait que le volet (d) de T-a2,
  retiré en validation P3) = le texte de `FIND_EARLIER_OPEN_SQL` **sans** ` FOR UPDATE`, et
  `fn earlier_open_locking_sql() -> String` = `format!("{EARLIER_OPEN_SQL} FOR UPDATE")`, **seule source** du texte
  verrouillant : l'étape (d) de `close` l'exécute (`query_as::<_, FiscalYear>(&earlier_open_locking_sql())`, à la
  place de `FIND_EARLIER_OPEN_SQL` l. 1115), et T-a15 l'appelle — le test voit donc **ce que `close` exécute**, non
  une reconstitution (P2 R12). **Texte exécuté identique octet pour octet** à aujourd'hui (les motifs des tests 15-8a
  et 15-12a reconnaissent ce texte ; T-a15 compare la chaîne rendue à l'ancienne constante, recopiée **dans le test**
  comme référence figée), sur le modèle de `OPEN_COVERING_DATE_SQL` / `find_open_covering_date` /
  `has_open_covering_date` (l. 710-760). `FIND_EARLIER_OPEN_SQL` disparaît ; son nom se **propage** aux doc-comments de
  `fiscal_years.rs` l. 868 et l. 999 et au commentaire de la page des exercices (`+page.svelte` l. 118), qui le citent
  (`grep -rnF "FIND_EARLIER_OPEN_SQL" crates frontend/src` vide après le patch). Aucun verrou ni aucune lecture ne
  s'ajoute ni ne se déplace dans `close` ;
- `pub async fn find_earliest_earlier_open<'e, E>(executor, company_id, start_date) -> Result<Option<FiscalYear>,
  DbError>` : lecture **sans verrou** de ce texte (K1) ;
- `const NEXT_AFTER_SQL` = « exercice de la société dont `start_date > ?`, `ORDER BY start_date ASC
  LIMIT 1` », et `pub async fn find_next_after<'e, E>(executor, company_id, end_date) -> Result<Option<FiscalYear>,
  DbError>`, **sans verrou** — K7 (29-1a) et l'étape (4) de la contre-passation (29-1b) l'appellent tous deux.

**AC4 — La route.** `GET /api/v1/fiscal-years/{id}/closing-checks`, montée dans `authenticated_routes` à côté de
`GET /api/v1/fiscal-years/{id}` (`lib.rs:938`) : **tout rôle**, Consultation et clés d'API comprises (lecture, comme
les rapports, `lib.rs:973`). 404 si l'exercice n'est pas de la société (anti-énumération, même ordre que
`get_fiscal_year`) — décidé **dans l'instantané**, par le `DbError::NotFound` de `for_fiscal_year` (AC1), sans lecture
préalable sur le pool. 200 avec :

```json
{ "fiscalYearId": 7, "fiscalYearName": "Exercice 2026", "startDate": "2026-01-01", "endDate": "2026-12-31",
  "checks": [ { "code": "PENDING_BANK_TRANSACTIONS", "severity": "warning", "status": "attention",
                "count": 4, "details": { … } }, … ] }
```

Montants en **chaînes décimales** (convention du dépôt), clés en camelCase. Aucun audit (lecture, comme
`get_balance_sheet` hors export — C-29-1-5). Aucun rejeu (pas d'écriture). La route ne figure pas au registre des
routes mutantes (`audit_route_registry.rs` : GET).

**AC5 — Le panneau « Préparer la clôture ».** Sur l'écran *Exercices comptables*
(`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`), chaque exercice **ouvert** porte un bouton
*Préparer la clôture* (`data-testid="fiscal-year-checks-{id}"`), visible de **tous les rôles**. ⚠️ Aujourd'hui la
colonne d'actions n'existe que sous `canMutate` (`+page.svelte` l. 371-373 pour l'en-tête, l. 393 pour la cellule,
`colspan={canMutate ? 5 : 4}` l. 459) : un rôle Consultation n'a **aucune** colonne d'actions. La colonne est donc
rendue **pour tous les rôles** (en-tête *Actions* et cellule), le bouton *Préparer la clôture* y vit seul pour
Consultation, les boutons existants (*Renommer*, *Clôturer*, *Réouvrir*) y restent sous leurs conditions actuelles
(`canMutate`, `isAdmin`) sans changement, et le `colspan` de la ligne « aucun exercice » passe à `5` pour tous. Il ouvre un panneau
(`Dialog`, `data-testid="fiscal-year-checks-dialog"`) qui liste les sept contrôles dans l'ordre de l'AC1 : une ligne
par contrôle, son libellé, son état (*Rien à signaler* / le compteur et le détail), une icône et une couleur par
sévérité **lisibles en clair et en sombre** (jetons de thème existants, C-28-1-7), et un **lien** vers l'écran où l'on
agit : *Rapprochement* (K2), *Comptes bancaires* (K3), *Factures* filtrées sur les brouillons (K4), *Factures
fournisseur importées* (K5), la fiche de chaque écriture listée (K6), *Nouvel exercice* (K7, Admin/Comptable
seulement). Erreur de chargement : le panneau le dit (« indisponible ») — jamais « rien à signaler » par défaut.
**Pendant le chargement**, le panneau affiche un état d'attente (« Lecture des contrôles… ») et **aucune** ligne de
contrôle — ni *Rien à signaler*, ni compteur (P3 R13 : un « rien à signaler » avant la réponse est la même faute que
M-a10).
Montants en `big.js`, formatage suisse du dépôt.

**AC6 — Le rappel dans le dialogue de clôture.** Le dialogue existant (`+page.svelte:575-612`) charge les mêmes
contrôles à son ouverture et affiche, **au-dessus** du texte de confirmation actuel, soit « Aucun point à vérifier »,
soit « N point(s) à vérifier » avec la liste courte des contrôles en `attention` et un lien vers le panneau. Le bouton
*Clôturer* **reste actif** quel que soit le résultat (rien ne bloque, C-29-1-3) ; pendant le chargement il reste
actif aussi (on ne retarde pas un geste que le serveur accepte) ; un échec de chargement s'affiche sans bloquer.
**Pendant le chargement**, le rappel affiche le même état d'attente que le panneau (« Lecture des contrôles… ») et
**jamais** « Aucun point à vérifier » ni « N point(s) à vérifier » : ces deux textes ne s'affichent qu'après la
réponse (même règle que le panneau, R13 ; P4-1).
K1 n'a pas de cas visible ici : le bouton *Clôturer* est déjà désactivé quand un antérieur est ouvert (`:408`).
**Ce que compte « N point(s) »** (C-29-1-19) : les contrôles au statut `attention`, **toutes sévérités**, `info`
compris — K4 (brouillons), K5 (pièces à compléter) et K7 (exercice suivant absent : les transitoires ne peuvent pas
être contre-passées) sont des points réels avant une clôture. `notCheckable` de K3 ne compte pas (sans effet sur le
statut, AC1).

**AC7 — Textes.** Clés i18n neuves préfixées **`fiscal-years-checks-`** (titre, bouton, un libellé et un texte d'aide
par contrôle, motifs de K3 et K6, états, attente et erreur de chargement, rappel du dialogue). ⚠️ Le préfixe est au **pluriel**,
celui du dossier : `frontend/scripts/lint-i18n-ownership.js` déduit la fonctionnalité du dossier
(`features/fiscal-years/` → `fiscal-years`) et exige `key.startsWith('fiscal-years-')` ; un préfixe singulier
`fiscal-year-checks-` rougirait dans `FiscalYearChecks.svelte` (c'est la dette #30 des `invoice-*` de
`features/invoices/`). **Aucune** entrée neuve dans `KNOWN_VIOLATIONS`. Les `data-testid` (AC5) ne sont pas des
clés et gardent leur forme. Dans les **quatre** locales
(`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), en français soigné et traduites réellement
(pas de recopie du français) ; `npm run lint-i18n-ownership` et `i18n-keys.test.ts` verts. Aucune clé d'erreur
serveur neuve (la route ne refuse rien de neuf).

**AC8 — Manuel utilisateur.** `docs/manual/fr/user-manual.tex`, § *Clôture d'un exercice* (l. 732) : nouvelle
sous-section *Préparer la clôture* (`\label{sec:preparer-cloture}`) **avant** la marche de clôture — à quoi sert le
panneau, qu'il ne bloque rien, les sept contrôles un par un (ce qu'ils signalent, ce qu'on fait), les **limites** :
K5 non borné à l'exercice ; K3 ne compare que les relevés CAMT (un CSV ne porte pas de solde, § *Import bancaire*) —
un compte alimenté par CSV seulement, sans compte lié ou à compte lié partagé est dit « non vérifiable », sans
compter comme point à vérifier —, n'examine que les comptes qui ont, dans l'exercice, une transaction importée, un
relevé ou une écriture sur leur compte de grand livre lié (phrase à écrire telle quelle : « activité » seule serait lue
« des mouvements », sans dire lesquels), compare le relevé qui **se termine au plus tard le dernier jour de
l'exercice** — un relevé à cheval sur la fin d'année (du 15 au 15, ou une période choisie à l'export) ne compte pas, et
le compte reste « relevé arrêté trop tôt » ou « relevé manquant » : il faut importer un relevé **arrêté au dernier jour
de l'exercice** (P2 F L8) —, et ignore un écart inférieur au centime, comme la page d'accueil (qui lit le même écart,
AC3) ; K6 ne peut signaler que des données héritées ou restaurées (aucun
chemin d'écriture actuel n'en produit), et ne couvre pas une ligne dont le compte appartient à une autre société que
l'écriture (état que seul un SQL direct produirait) ; le panneau lit à un instant, il ne suit pas les saisies
concurrentes. La phrase l. 728 « Les comptes du bilan sont
automatiquement reportés depuis le bilan de clôture de l'exercice précédent **(si clôturé)** » est **fausse** au
code (le report est calculé que l'exercice précédent soit clos ou non, `balance_sheet.rs` l. 7-11) : corrigée dans
le même commit (défaut voisin **mécanique**, B3). Le résumé annuel l. 2992 gagne « contrôles préalables ». **PDF
régénéré** (`make fr` dans `docs/manual/`, sous le verrou de gate) et contrôlé aplati.

**AC9 — `api-external.md` et CHANGELOG.** `docs/api-external.md` documente la route (rôles, forme, codes, la
remarque sur K5). CHANGELOG : entrée `### Ajouté` sous la section **`## [0.14.0] — Non publié`** — la forme exacte
que `scripts/prepare-release.sh` attend (l. 237, une seule section par version, l. 241-246) —, « Préparer la clôture
d'un exercice : sept contrôles en lecture seule… » (#232), sans annoncer la 29-1b ; la 29-1b **réutilise** cette
section, elle n'en ouvre pas une seconde. ⚠️ Si une **v0.13.4** (29-2 à 29-4) est taguée **après** la fusion de la
29-1a, son image porte le code de la 29-1a : l'entrée passe alors dans la section 0.13.4 — décision de
l'orchestrateur au moment du tag, à rappeler dans le corps de la PR de la 29-1a.

### Partie b — `29-1b-contre-passation-transitoires`

**AC10 — La datation, paramètre interne de la contre-passation unique.** `reverse_in_tx_inner`
(`journal_entries.rs:2595`) reçoit un paramètre `dating: ReversalDating` (`enum ReversalDating { Today,
NextFiscalYearStart }`, documenté). **Toutes** les entrées existantes (`reverse`, `reverse_in_tx`,
`reverse_owned_in_tx`) passent `Today` — **comportement inchangé**, prouvé par les tests existants de la
contre-passation et des quatre annulations restés verts sans modification, **plus une assertion neuve** de T-b2 sur
le JSON d'audit du chemin `Today` (le test existant de `journal_entry_reversal_e2e.rs` l. 1120-1146 ne vérifie que la
**présence** de l'identifiant : il resterait vert si une clé s'ajoutait, P3 R6). ⛔ Aucune copie de la fonction : la
contre-passation reste « écrite une seule fois » (doc l. 2572).

**AC11 — Ce que `NextFiscalYearStart` fait, à la place de l'étape (4) seulement.**

- l'étape (1) lit en plus `fy.end_date` de l'exercice de l'origine (colonne ajoutée au même `SELECT … FOR UPDATE`,
  l. 2603-2607 — mêmes lignes verrouillées, aucun verrou ajouté) ;
- à l'étape (4), **au même endroit** que le calcul de `today` (après `reversal_blockers`, l. 2664), une lecture
  ordinaire — `fiscal_years::find_next_after(&mut **tx, company_id, origin_fy_end_date)` (AC3-bis, la lecture de K7)
  — rend le **premier exercice de la société dont `start_date > end_date de l'origine`** ; absent →
  `DbError::NextFiscalYearMissing { fiscal_year_name }` (l'exercice de l'origine, pour le message). ⚠️ Sur **ce**
  chemin, appelé seulement par `reverse_at_next_fiscal_year_start[_in_tx]` **avec `authority = None`**, aucune
  lecture ordinaire ne précède `reversal_blockers` : l'étape (1) est verrouillante (elle ne fait pas naître la vue)
  et l'étape (1-bis) ne s'exécute pas. La phrase **ne vaut pas en général** : avec une autorité fournisseur, l'étape
  (1-bis) (`SELECT id FROM supplier_invoices …`, l. 2641) est une lecture ordinaire antérieure, et un appelant de
  `reverse_in_tx` a souvent lu avant l'appel. Elle ne s'écrit donc qu'avec sa condition, au doc-comment comme ici ;
- la date retenue est son `start_date` ; `find_open_covering_date(tx, company_id, start_date)` — lecture
  **verrouillante**, donc sur l'état validé le plus récent — la confirme : `None` (exercice clos depuis, ou supprimé
  par `onboarding::reset`) → `DbError::NextFiscalYearClosed { fiscal_year_id, fiscal_year_name }` avec l'exercice lu
  à l'étape précédente ;
- ⛔ **le statut de N+1 n'est jugé QUE par cette confirmation verrouillante** (C-29-1-27, P2 R5 = F3) : la lecture
  ordinaire `find_next_after` ne sert qu'à **trouver** l'exercice et sa date ; son champ `status` n'est **pas lu**
  par la contre-passation (pas de `if next.status == Closed`, même en défense supplémentaire) — un tel test, placé sur
  la lecture ordinaire, rendrait la confirmation invisible aux tests (M-b3) ;
- la suite est **celle de `Today`** : `create_in_tx_inner` avec `entry_date = start_date`, donc **tous** ses refus —
  `PeriodLocked` si `start_date <= books_locked_through`, `LaterFiscalYearClosed` si un exercice postérieur à N+1 est
  clos —, lettrage de l'étape (5), audit `journal_entry.reversed` qui porte en plus `"dating": "nextFiscalYearStart"`
  (le chemin `Today` garde son JSON actuel, octet pour octet — prouvé par l'assertion neuve de T-b2, AC10) ;
- le libellé est celui de la contre-passation (`Contre-passation écriture n° X (Exercice N)` : l'exercice diffère
  toujours) ;
- origine dans un exercice **clos** : accepté (comme la contre-passation du jour) ; N+1 doit être ouvert.

**AC12 — Ordre des verrous et naissance de la vue : rien ne bouge (critère hérité de l'AC7 de la 28-4, B5).** Le
chemin `NextFiscalYearStart` prend **les mêmes verrous, dans le même ordre**, que `Today` : origine et son exercice
(1), lignes de l'origine (2), puis le parcours de `find_open_covering_date` (4), puis ceux de `create_in_tx_inner`.
La seule lecture ajoutée est **ordinaire** et placée **après** `reversal_blockers` : la vue est déjà née **au plus
tard** à `reversal_blockers` (sur ce chemin, sans autorité, exactement là — AC11), elle ne naît pas plus tôt. Les
cycles connus (avec `close`, avec `create` d'un exercice antérieur, avec le lettrage — doc l. 2574-2592 —, et avec
`create` d'un exercice **postérieur** quand un exercice M antérieur à N existe : son `find_overlapping`, `start_date <=
?` (`fiscal_years.rs` l. 789-799), prend M puis demande N que la contre-passation tient, et celle-ci demande M dans le
parcours de `find_open_covering_date` — cycle déjà présent sur le chemin `Today`, rejoué des deux côtés, non neuf,
mais à **ajouter** à la liste du doc-comment, P2 F L4) sont **de même nature** et se résolvent par le même rejeu. Le doc-comment de `reverse_in_tx_inner` le dit. Garde : T-b4
(aucune lecture ordinaire neuve avant `reversal_blockers`, par lecture du source) et T-b5 (concurrence, ci-dessous).

**Sur quoi repose la sûreté — à écrire tel quel au doc-comment** (F2) : *une contre-passation n'écrit jamais dans un
N+1 clos* parce que la date et l'exercice sont **confirmés** par `find_open_covering_date` — lecture
**verrouillante** de N+1, qui lit l'état validé le plus récent et le tient jusqu'au `COMMIT`. Elle donne le refus
nommé (`NEXT_FISCAL_YEAR_CLOSED`) et ne dépend d'aucun détail de `fiscal_years::close` ; l'écriture dans un exercice
clos est de toute façon refusée, en second lieu, par le re-verrou de l'exercice dans `create_in_tx_inner`
(`journal_entries.rs` l. 333-345, `FISCAL_YEAR_CLOSED`) (P3 F3). Le verrou de N tenu dès
l'étape (1) en a une **conséquence utile, non une garantie** : comme `close(N+1)` verrouille aujourd'hui ses
antérieurs en (b') (`fiscal_years.rs` l. 1081-1090, boucle `LIST_EARLIER_SQL … ORDER BY start_date ASC`), la clôture
de N+1 **attend** une contre-passation en cours — **si N est le premier exercice de la société**. S'il existe un
exercice M antérieur à N, `close(N+1)` tient M puis demande N, tandis que la contre-passation tient N puis demande M
(parcours de `find_open_covering_date`, qui part du premier exercice) : c'est le **cycle connu** avec la clôture
(test 13 d, `fiscal_years_repository.rs` l. 1844-1857), résolu par le rejeu des deux routes — non une attente. La
réouverture de N+1 (N+1 clos ⇒ N clos, invariant I) ne prend pas N : elle peut changer l'état lu entre la vue et la
confirmation, et c'est la confirmation qui décide.

*(Validation P4, 2026-10-14, décision de l'orchestrateur — P4-4.)* **Angle mort assumé** : aucune mutation ni aucun
test ne tue une confirmation **non verrouillante** de N+1 (lecture ordinaire au lieu de `find_open_covering_date`) ;
M-b3 ne tue que la suppression totale de la confirmation. Ce trou est vérifié **en revue de code** (le doc-comment de
`reverse_in_tx_inner` le dit) et non par un test ; aucune mutation ni aucun test n'est ajouté pour le couvrir (la
forme M-b13 a été retirée pour recyclage, C-29-1-27, C-29-1-31).

*(Validation P3, 2026-10-14, décision de l'orchestrateur : le paragraphe consacré à l'état que garde la
confirmation est retiré, avec la mutation M-b13 qu'il fondait — C-29-1-27.)* M-b3 reste tuée par T-b3, par la
différence de code d'erreur avec `create_in_tx_inner`, grâce à la prescription de l'AC11.

**AC13 — La route.** `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, **sans corps**, montée
dans `comptable_routes` à côté de `/reverse` (`lib.rs:383`) : Admin et Comptable ; Consultation 403 ; clés
`read-write` admises comme pour `/reverse`. Handler calqué sur `reverse_journal_entry` (`routes/journal_entries.rs:508`),
**rejoué** (`retry_on_deadlock("journal_entries::reverse_at_next_fiscal_year_start", …)`), réponse `201
JournalEntryResponse`. Inscrite au registre `audit_route_registry.rs` (statut d'audit tracé, rejeu `Rejouee`). Les
compteurs **se recomptent depuis la source** ; à `16eeed21` (revérifié en validation P1), les sites qui bougent sont :
`crates/kesh-api/tests/audit_route_registry.rs` l. 659 `assert_eq!(LIB_ROUTES.len(), 118, …)` → 119 (et son
message), l. 662 `traced, 112` → 113 (avec la ligne d'historique de l'assertion, s'il y en a une), l. 682 et
l. 1012 `121` → 122, l. 994 `rejouees, 24` → 25 (et son message d'historique, qui énumère les routes rejouées — P2
R9), et la doc du fichier l. 27, 31, 32 (« 118 routes », « (118 routes) », « (121) »). Greper la **valeur**
(`\b(118|112|121)\b` sur le dépôt, `\b24\b` — trop fréquent ailleurs — sur `audit_route_registry.rs` et
`docs/api-external.md`) avant de conclure ; le registre indexe par nom, non par position — aucun
couplage positionnel (P6) à `16eeed21`, à revérifier au moment du patch. `docs/api-external.md` l. 730 : la route
s'ajoute à la liste des routes rejouées. Les GET neufs (AC4) n'entrent ni dans l'inventaire ni dans le registre.
Le dépôt expose **deux** entrées publiques, comme pour la contre-passation du jour (`reverse_in_tx` l. 2488 /
`reverse` l. 2847) : `reverse_at_next_fiscal_year_start_in_tx(tx, company_id, id, user_id)` — sur une transaction
de l'appelant, qui valide ou non (T-b5 la tient ouverte) — et `reverse_at_next_fiscal_year_start(pool, …)`, enveloppe
`begin` / appel / `commit` que la route appelle.
⛔ `/reverse` n'accepte **pas** de corps neuf (clients existants, C-29-1-10).

**AC14 — Les refus, traduits.** Deux codes neufs, 409 `CONFLICT` :
`NEXT_FISCAL_YEAR_MISSING` (« Aucun exercice ne suit l'exercice « N » : créez l'exercice suivant, puis
recommencez. », `details: {fiscalYearName}` — l'exercice **de l'origine**, que le message nomme) et `NEXT_FISCAL_YEAR_CLOSED` (« L'exercice suivant, « N+1 », est clôturé : la contre-passation ne peut
pas y être datée. », `details: {fiscalYearId, fiscalYearName}`). Mapping au `match` de `errors.rs` (proche de
`FiscalYearInvalid`, l. 3988), clés `error-next-fiscal-year-missing` / `-closed` ×4 locales. Les replis Rust des
deux messages (`errors.rs`) égalent **mot pour mot** la valeur fr-CH (garde G9 `les_replis_rust_suivent_le_catalogue`
de `textes_coherents.rs`, P2 R15) — ⚠️ G9 ne compare que les clés de sa `TABLE` **fermée** (l. 683 et suiv.) : les
deux clés y sont **ajoutées**, sous `crates/kesh-api/src/errors.rs`, `("error-next-fiscal-year-missing", 1, NOM)` et
`("error-next-fiscal-year-closed", 1, NOM)` (`NOM` = `{ $name }` → `{fiscal_year_name}`, l. 680 ; le repli Rust
nomme donc sa variable `fiscal_year_name`), faute de quoi un repli divergent passerait en silence (P3 R4). Les deux codes entrent dans la table d'erreurs de `docs/api-external.md` § 10
(l. ~694-728, à côté de `EARLIER_FISCAL_YEAR_OPEN`, P2 R16). Tous les autres refus sont ceux de la contre-passation,
sans changement (`EntryNotReversable`, `ReversalAccountsArchived`, `PeriodLocked`, `LaterFiscalYearClosed`).

**AC15 — Le bouton sur la fiche d'une écriture.** `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` :
à côté de *Contre-passer* (`data-testid="reverse-entry"`, l. 330-336), un second bouton *Contre-passer au début de
l'exercice suivant* (`data-testid="reverse-entry-next-year"`), sous **les mêmes conditions** (`entry.reversable`,
rôle Admin/Comptable) **et** quand la liste des exercices (`GET /api/v1/fiscal-years`, client existant
`fiscal-years.api.ts`) contient un exercice **ouvert** dont `startDate` est la plus petite `> endDate` de l'exercice
de l'écriture (`entry.fiscalYearId`). Confirmation qui **nomme la date et l'exercice** (« La contre-passation sera
datée du 01.01.2027, dans l'exercice « Exercice 2027 ». ») ; succès → navigation vers l'écriture créée, comme
*Contre-passer*. Le serveur reste l'autorité : un refus (AC14) s'affiche traduit, sans « [object Object] » (#520).
Les deux boutons disparaissent ensemble une fois l'écriture contre-passée (`reversable` faux).

**Le bouton absent s'explique** (règle de la page, commentaire l. 305-311 : « Les boutons sont ABSENTS, pas
désactivés… Le motif est affiché à leur place »), pour Admin/Comptable quand `entry.reversable` est vrai
(C-29-1-22) — un motif `data-testid="reverse-entry-next-year-reason"`, sous les boutons, comme
`reverse-blocked-reason` :

| État de la liste des exercices | Bouton | Motif affiché |
|---|---|---|
| en cours de chargement | absent | **aucun** (rien de trompeur) |
| échec de `GET /api/v1/fiscal-years` | absent | « La liste des exercices n'a pas pu être lue : rechargez la page pour contre-passer au début de l'exercice suivant. » |
| aucun exercice ne suit celui de l'écriture | absent | « Pour contre-passer au début de l'exercice suivant, créez d'abord l'exercice suivant. » |
| l'exercice qui suit est clôturé | absent | « L'exercice suivant, « {name} », est clôturé : la contre-passation ne peut pas y être datée. » |
| l'exercice qui suit est ouvert | **présent** | aucun |

Le rôle Consultation ne voit ni le bouton ni le motif (C-15-8-14, inchangé). L'échec de la liste n'empêche pas
*Contre-passer* (du jour), qui n'en dépend pas.

**AC16 — Textes.** Clés neuves préfixées `journal-entries-reverse-next-year-` (bouton, confirmation avec `{ $date }`
et `{ $name }`, libellé d'aide, les trois motifs de l'AC15) — le préfixe de `journal-entries-reverse-action`, conforme
à `lint-i18n-ownership` si une clé est appelée depuis `features/journal-entries/` — dans les quatre locales ; lints
i18n verts.

**AC17 — Manuel utilisateur, brochure, README, `api-external.md`, CHANGELOG.** § *Corriger une écriture : la contre-passation* (l. 628) : nouvelle sous-section
*Les transitoires de fin d'exercice* — l'usage (charge payée d'avance, produit à recevoir, facture à recevoir : actifs
1300 et passifs 2300 transitoires des plans livrés), la marche (saisir l'écriture au dernier jour de N, puis
*Contre-passer au début de l'exercice suivant*), ce que Kesh exige (exercice suivant **créé et ouvert**, premier
jour hors période verrouillée), ce qui se fige : l'écriture et sa contre-passation ne se modifient ni ne se
suppriment plus (une écriture contre-passée ne se supprime pas, `journal_entries.rs:1866-1875`, ni ne se modifie, garde de
modification) — une erreur se corrige par une **écriture complémentaire** ; et l'angle mort : **Kesh ne repère pas
une transitoire que vous avez oublié de contre-passer** (C-29-1-14). La phrase du verrou de période « La
contre-passation est datée du jour » (l. 614-615) gagne l'exception. La phrase du § *Exercices dans le désordre*
(l. 764) « Une contre-passation, datée du jour, n'est refusée que si l'exercice du jour est lui-même suivi d'un
exercice clôturé » gagne la sienne : la contre-passation au début de l'exercice suivant est refusée si **cet**
exercice est clôturé ou suivi d'un exercice clôturé. Le paragraphe du même § « Des boutons offerts, puis refusés au
clic » (l. 766), qui énumère les boutons affichés puis refusés dans l'état hérité, gagne *Contre-passer au début de
l'exercice suivant* : avec N+1 ouvert suivi d'un exercice clos, le bouton est affiché (AC15) puis refusé en
`LATER_FISCAL_YEAR_CLOSED` (P2 R16). Le panneau *Préparer la clôture* (§ de la 29-1a) renvoie à
cette sous-section. **PDF régénéré**, contrôlé aplati. Propagation à greper au même commit (« datée du jour »,
« contre-passation » : `grep -rn "datée du jour\|daté du jour\|celle du jour" docs/ README.md website/ crates/`) —
`crates/` compris (P3 R10) : deux commentaires du code donnent la date du jour comme raison de traverser le verrou de
période, et deviennent faux pour le geste neuf — `crates/kesh-db/src/repositories/journal_entries.rs` l. 398-402
(« sa date est celle du jour ») et `crates/kesh-db/src/repositories/companies.rs` l. 909-912 (« celle-ci étant datée
du jour »), ainsi que le commentaire de test `crates/kesh-api/tests/period_lock_e2e.rs` l. 367-369 (« la
contre-passation est datée du **jour** », P4-2) ; ils gagnent l'exception dans le commit de la 29-1b (Tb2) ; la brochure
(`docs/manual/fr/marketing-brochure.tex` l. 418 : « Le bouclement d'exercice — écritures de clôture, report à
nouveau du résultat, transitoires — reste à venir ») est corrigée **dans ce commit** — elle annonce « à venir » ce
qui sera livré, et les écritures de clôture sont écartées par conception — avec son PDF ; le README (l. 47,
« Clôture d'exercice & report à-nouveau ») gagne les contrôles préalables et les transitoires dans le même commit
(règle « Synchroniser le planning du README »). Les numéros de compte cités (1300, 2300) et leur nom passent la garde
G4-bis (`les_comptes_cites_en_exemple_existent_dans_les_plans_livres`, `textes_coherents.rs`), qui lit le manuel
utilisateur et les quatre `messages.ftl` contre les plans livrés ; une référence `#232` ajoutée au README est un lien
(garde G7) (P2 R15).

**`docs/api-external.md`** : la route de l'AC13 (liste des routes rejouées, § 10) et les deux codes de l'AC14 (table
d'erreurs, § 10) ; **et** une sous-section au § 7, à la suite de celle qui décrit la contre-passation (`### Modifier une
écriture`, paragraphe « Une écriture refusée se corrige par contre-passation », l. 284), qui **décrit** la route comme
ses voisines (P3 R14) : `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, **sans corps**, clés
`read-write` admises ; la contre-passation est datée du **premier jour de l'exercice qui suit** celui de l'écriture,
dans cet exercice ; réponse `201` et `JournalEntryResponse`, comme `/reverse` ; refus `409 NEXT_FISCAL_YEAR_MISSING`
(`details.fiscalYearName`, l'exercice de l'écriture) et `409 NEXT_FISCAL_YEAR_CLOSED` (`details.fiscalYearId`,
`details.fiscalYearName`), plus ceux de la contre-passation, dont `PERIOD_LOCKED` si ce premier jour est dans la
période verrouillée et `LATER_FISCAL_YEAR_CLOSED` ; interblocage rejoué par le serveur. **CHANGELOG** : `### Ajouté` (contre-passation au début de l'exercice suivant, pour les
transitoires), mention #232, dans la section `## [0.14.0] — Non publié` ouverte par la 29-1a (AC9). *(Ces deux
livrables étaient portés par l'ancienne AC19 ; ils passent ici et restent dans la 29-1b — C-29-1-28.)*

**AC18 — E2E de parcours complet (porte la charge de la recette, fichier d'epic § *La recette*).** Spec neuve
`frontend/tests/e2e/fiscal-year-closing-journey.spec.ts`, sur une société de démonstration montée par l'API
(fixtures **synthétiques**) :

0. **fixtures, nommées** (R3) — K3 est le contrôle qui empêcherait l'état vert de l'étape 6, d'où leur précision :
   `seedTestState('with-company-no-fy')` (`helpers/test-state.ts` : la base est **vidée** puis le préréglage posé —
   aucune pollution d'une autre spec, et aucun exercice) ; la spec crée N puis N+1 par l'API, si bien que **N est le
   premier exercice** de la société (K1 `ok` ; la clôture de N n'est pas refusée par un antérieur ouvert) ; le
   préréglage (`test_fixtures::seed_accounting_company_no_fy`) ne pose que cinq comptes — `1000 Caisse CI`, `1100
   Banque CI`, `2000 Capital CI`, `3000 Ventes CI`, `4000 Charges CI` — et **aucun** compte bancaire (P2 R7 = F4),
   d'où, **créés par l'API à l'étape 0** (`POST /api/v1/accounts`, puis `POST /api/v1/bank-accounts`) :
   `1300 Actifs transitoires` (`Asset`, débité à l'étape 4), `1020 Banque` (`Asset`, compte de grand livre lié, **non
   partagé**) et un compte bancaire **lié à 1020** — ⛔ **pas 1100**, compte débiteurs par défaut du préréglage
   (`company_invoice_settings.default_receivable_account_id`), refusé comme compte lié
   (`BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT` ; comptes de créance lus par `claims_for_target`, `routes/bank_accounts.rs`
   l. 448-466), ni 1000 (accepté, mais compte d'impôt préalable par défaut du préréglage : la spec ne mêle pas la TVA
   au relevé) ; la charge créditée à l'étape 4 est `4000 Charges CI` (C-29-1-29) ; un relevé
   **CAMT** synthétique neuf, `frontend/tests/e2e/fixtures/camt053_v04_year_end.xml` (IBAN de test, sur le modèle de
   `camt053_v04_minimal.xml`), période du 01.12.N au **31.12.N** (dernier jour de N), solde d'ouverture `OPBD`
   **0.00**, une entrée créditrice de 100.00 au 15.12.N, solde de clôture `CLBD` **100.00** — solde d'ouverture nul :
   aucune écriture de soldes de départ n'est requise pour que le relevé égale les livres ; **aucune** facture en
   brouillon ni pièce importée à compléter (K4, K5 — K5 n'est pas borné à l'exercice : seul le vidage du préréglage
   le garantit) ;
1. deux exercices ouverts N et N+1 ; le relevé ci-dessus importé (sa transaction non rapprochée) ;
2. *Préparer la clôture* de N : K2 en `attention` (1), K3 en `attention` (`GAP` : 0 aux livres contre 100.00 au
   relevé), K7 `ok` (N+1 existe) ;
3. rapprocher la transaction (écriture créée par le rapprochement, débit du compte lié) ; rouvrir le panneau : K2 et
   K3 `ok` ;
4. saisir une écriture de régularisation au dernier jour de N (débit 1300, crédit 4000) ; *Contre-passer au début
   de l'exercice suivant* : l'écriture créée est datée du premier jour de N+1, dans N+1, liée à l'origine ;
5. bilan de N au dernier jour : 1300 porte le montant ; bilan de N+1 au premier jour : 1300 est **nul** ;
6. clôturer N (le dialogue affiche « Aucun point à vérifier ») ; N est clos ;
7. un administrateur rouvre N avec un motif ; N est ouvert ; le panneau se relit.

Plus un cas de refus : sans exercice suivant, le bouton n'apparaît pas **et le motif
`reverse-entry-next-year-reason` le dit** (AC15), et l'appel API direct rend `409 NEXT_FISCAL_YEAR_MISSING`.

**AC19 — Retirée (validation P2, 2026-10-14, décision de l'orchestrateur).** La répétition à blanc de la clôture au
manuel d'administration n'est plus une exigence de cette fiche : elle devient la **Story 29-7** (doc,
`29-7-repetition-a-blanc` au registre de sprint), écrite **en l'exécutant réellement** sur la station avec une
sauvegarde synthétique. Son cahier des charges est porté par la clé de sprint (findings R1 et R2 de la lentille R, F5
de la lentille F, validation P2 : `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p2-{R,F}.md`). Les deux autres
livrables de l'ancienne AC19, `api-external.md` et CHANGELOG de la 29-1b, sont à l'AC17. La 29-1b ne touche **pas**
`docs/manual/fr/admin-manual.tex`. C-29-1-23.

## Tasks / Subtasks

### 29-1a

- [x] **Ta1** — Tests rouges d’abord : T-a1 à T-a16 (AC1-AC6, AC3-bis). En T0, **mesurer** le comportement de
      MariaDB 10.11 devant `SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE` exécuté sur `ReadSnapshot::conn()` (un test
      d'une ligne) et consigner l'issue — erreur ou succès — au Dev Agent Record ; la garde T-a2 reste (AC2, P2 F L3).
- [x] **Ta2** — `crates/kesh-db/src/repositories/closing_checks.rs` (neuf, `pub mod` dans `repositories/mod.rs`) :
      types `ClosingChecks`, `ClosingCheck`, enums `Severity`, `CheckStatus`, détails par code ; doc de module (les
      sept contrôles, l'instantané, ce que K5 et K6 ne disent pas, et la **liste des fonctions de dépôt externes**
      appelées — angle mort de T-a2, vérifié en revue de code) (AC1, AC2).
- [x] **Ta3** — Extraction `BALANCES_SQL` / `balances_by_bank_account` / `balances_from_row` dans
      `bank_accounts.rs`, SQL unique avec borne, `BalancesRow` `pub(crate)`, `is_shared` et `statement_gap` (AC3) ;
      `EARLIER_OPEN_SQL`, `earlier_open_locking_sql`, `find_earliest_earlier_open`, `NEXT_AFTER_SQL`,
      `find_next_after` dans `fiscal_years.rs`, `FIND_EARLIER_OPEN_SQL` retirée et son nom propagé aux doc-comments
      l. 868 et 999 et au commentaire de la page l. 118 (AC3-bis).
- [x] **Ta4** — Route et handler `get_closing_checks` dans `routes/fiscal_years.rs`, DTO camelCase, montage
      `lib.rs` (AC4) ; champ `statement_gap` de `BankAccountWithBalance` (`routes/bank_accounts.rs`) et son test neuf
      dans `bank_accounts_e2e.rs` (AC3, T-a16).
- [x] **Ta5** — Frontend : types et client `fiscal-years.api.ts` (`getClosingChecks`), composant
      `FiscalYearChecks.svelte` dans `features/fiscal-years/`, colonne d'actions rendue pour tous les rôles, bouton
      et dialogue dans la page, rappel du dialogue de clôture (AC5, AC6). **Test existant à adapter** :
      `frontend/src/routes/(app)/settings/fiscal-years/fiscal-years-page.test.ts` l. 42-48 mocke
      `fiscal-years.api` **par liste d'exports** — `getClosingChecks` y serait `undefined` : l'ajouter au mock
      (succès, échec, en attente), et ajouter le rôle Consultation à ses cas (colonne d'actions). Le Dev Agent
      Record nomme ce fichier parmi les tests **modifiés** (la clause « verts sans modification » ne vaut que pour
      les tests Rust de l'AC3). **Écart du tableau de bord** (AC3) : `statementGap` dans `bank-accounts.api.ts`,
      `homepage.ts::statementGap` lit le champ, `toCentime` supprimé ; `homepage.test.ts` (cas d'arrondi remplacés par
      un test de lecture) et les fabriques de `BankAccountSummary` adaptées — fichiers nommés au Dev Agent Record parmi
      les tests modifiés.
- [x] **Ta6** — i18n ×4 (AC7) ; gardes à garder vertes : `lint-i18n-ownership`, `i18n-keys.test.ts`, G4-bis de
      `textes_coherents.rs` (numéros de compte cités dans les `messages.ftl`, P2 R15).
- [x] **Ta7** — Manuel utilisateur + PDF ; `api-external.md` ; CHANGELOG (AC8, AC9) ; garde G4-bis verte (manuel
      utilisateur, P2 R15).
- [x] **Ta8** — Mutations M-a1 à M-a22 (M-a9 déclarée équivalente, non exécutée) ; captures clair et sombre du panneau et du dialogue (hors dépôt,
      `/home/gcorbaz/devel/kesh-gate-logs/29-1a-captures/`) ; gate complet et E2E complet au dernier commit de code.

### 29-1b

- [ ] **Tb1** — Tests rouges d’abord : T-b1 à T-b9 (AC10-AC15, AC18).
- [ ] **Tb2** — `ReversalDating`, paramètre de `reverse_in_tx_inner`, étapes (1) et (4) (lecture par
      `fiscal_years::find_next_after`, AC3-bis), nouvelles variantes de `DbError`, fonctions publiques
      `reverse_at_next_fiscal_year_start_in_tx` (sur la transaction de l'appelant, comme `reverse_in_tx` l. 2488) et
      `reverse_at_next_fiscal_year_start` (enveloppe pool calquée sur `reverse`, l. 2847) ; doc-comment des verrous
      mis à jour, avec la phrase sur la sûreté de l'AC12, la réouverture de N+1 qui peut changer l'état lu (la
      confirmation décide) et le cycle de `create` d'un exercice postérieur (AC10-AC12) ; les deux commentaires qui
      donnent la date du jour comme raison de traverser le verrou de période — `journal_entries.rs` l. 398-402 et
      `companies.rs` l. 909-912 — gagnent l'exception du geste neuf (AC17, P3 R10) ; l'audit `journal_entry.reversed` garde son action **littérale**
      — une action choisie par `match` sur `dating` serait un site indirect neuf pour `audit_label_registry.rs` (P2
      R15).
- [ ] **Tb3** — Route, handler rejoué, mapping `errors.rs` (replis égaux à fr-CH, garde G9 : **ajouter les deux clés
      à la `TABLE` de G9**, avec `NOM`, AC14, P3 R4), registre des routes et ses compteurs (AC13, AC14).
- [ ] **Tb4** — Frontend : client `reverseJournalEntryAtNextFiscalYearStart`, lecture de la liste des exercices,
      bouton, motifs, confirmation, refus (AC15). **Test existant à adapter** :
      `frontend/src/routes/(app)/journal-entries/[id]/journal-entry-page.test.ts` l. 34-38 mocke
      `journal-entries.api` par liste d'exports (sans le client neuf) : le compléter. Il **mocke déjà**
      `fiscal-years.api` (l. 49, `listFiscalYears: vi.fn()`, pour le mode édition) mais ce `vi.fn()` rend `undefined`
      (P3 R7) : le chargement neuf, fait à l'**ouverture** de la page et non au clic, recevrait `undefined` dans chaque
      cas existant — lui donner un comportement par défaut (liste vide résolue, posée dans le `beforeEach`) et trois
      comportements pour T-b8 (succès, échec, en attente). `startEdit` (l. 211-235) **garde sa propre lecture** au
      clic, inchangée : son échec a sa dégradation propre, et la liste a pu changer depuis l'ouverture (C-29-1-30).
      Fichier nommé au Dev Agent Record parmi les tests modifiés.
- [ ] **Tb5** — i18n ×4 (AC14, AC16) ; gardes G4-bis et G9 de `textes_coherents.rs` (G9 étendue des deux clés, Tb3),
      lints i18n verts.
- [ ] **Tb6** — E2E de parcours (AC18).
- [ ] **Tb7** — Manuel utilisateur + PDF ; brochure + PDF et README ; `api-external.md` (route décrite au § 7, liste des routes rejouées, deux codes) ;
      CHANGELOG (AC17) ; gardes G4-bis et G7 vertes.
- [ ] **Tb8** — Mutations M-b1 à M-b12 ; captures clair et sombre (bouton, confirmation, refus) hors dépôt ; gate
      complet et E2E complet au dernier commit de code.

## Dev Notes

### Concurrence — ce que chaque partie fait aux verrous

- **29-1a** : **aucune** écriture, **aucun** verrou, une seule vue (`ReadSnapshot`). `fiscal_years::close` ne
  change **pas de comportement** — seul le texte de son étape (d) est factorisé (AC3-bis), le SQL exécuté restant
  identique octet pour octet ; aucun verrou ni aucune lecture ne s'y ajoute ni ne s'y déplace — et le dialogue lit
  les contrôles par une requête séparée, avant le `POST /close`. Ce que cela implique,
  et qui est voulu : un contrôle lu `ok` peut être faux une seconde plus tard (une transaction importée entre le
  panneau et la clôture). C'est un **instrument de revue**, non une garantie ; le manuel le dit (AC8).
- **29-1b** : mêmes verrous, même ordre que la contre-passation du jour (AC12). Le **seul** écart d'ensemble lu est
  l'exercice cherché : *celui qui suit l'origine* au lieu de *celui du jour*. Dans le cas d'usage (régularisation de
  N passée en début de N+1), les deux coïncident souvent ; quand l'utilisateur le fait plus tard (N+1 encore ouvert,
  on est en N+2), le parcours de `find_open_covering_date` s'arrête plus tôt — il verrouille **moins** de lignes,
  jamais d'autres tables.
- ⚠️ Lecture ordinaire de l'exercice suivant **puis** confirmation verrouillante : sous `REPEATABLE READ`, la lecture
  ordinaire voit la vue née à `reversal_blockers` (sur ce chemin, sans autorité — AC11) ; un exercice suivant **créé** après cette vue (un exercice qui
  s'intercalerait entre N et le suivant lu) n'est pas vu — la contre-passation se daterait alors au début du
  suivant lu, non de l'intercalé. Cas d'école (créer un exercice **antérieur** à un exercice existant est rare, et
  refusé si un postérieur est clos) ; angle mort à écrire au doc-comment, non au manuel (C-29-1-11). Précision à y
  porter (P2 F L5) : sur le chemin **pool** (`reverse_at_next_fiscal_year_start`), l'intercalé ne peut naître après la
  vue qu'au prix d'un plan atypique — sa création parcourt N par `find_overlapping`, et A tient N dès l'étape (1),
  avant la naissance de sa vue : la création attend A, ou s'est validée avant le verrou, et la vue la voit. L'angle
  mort est réel pour `reverse_at_next_fiscal_year_start_in_tx`, dont l'appelant a pu lire avant l'appel.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `bank_accounts.rs::list_by_company_with_balances` : sa forme « une sous-requête par somme, jamais
  `bank_transactions` dans le même `FROM` » (doc l. 1387-1391) et l'invariant « une transaction, une écriture » se
  conservent tels quels dans l'extraction ; le DTO du tableau de bord (`BankAccountWithBalance`) ne change que par
  l'**ajout** du champ `statement_gap` (AC3, C-29-1-25) — ses champs actuels et leurs valeurs restent identiques.
- `fiscal_years.rs::close` : texte exécuté à l'étape (d) inchangé (les tests de concurrence des Stories 15-8a et
  15-12a le reconnaissent à ses motifs) ; `LOCK_EARLIER_BY_ID_SQL` et `LOCK_IN_COMPANY_SQL` intacts (T-b5 et les
  tests 15-12a s'appuient sur leurs listes de colonnes).
- `reverse_in_tx_inner` : précédence des refus (`reversal_blockers`), autorités fournisseur (1-bis), refus nommant
  les comptes archivés (3), libellé, lettrage (5) — inchangés. L'audit du chemin `Today` garde son JSON.
- La page des exercices : bandeau de désordre (`:343`), bouton *Clôturer* désactivé par `earliestEarlierOpen`
  (`:123`, `:408`), dialogue de réouverture : inchangés.
- La fiche d'écriture : bouton *Contre-passer*, motif de blocage à sa place (`:350`) : inchangés.

### Pièges connus du dépôt, applicables ici

- **Tests muets** : chaque contrôle a un test `ok` **et** un test `attention` sur des données qui le déclenchent
  réellement ; pour K6, les états hérités se posent par **SQL direct** dans le test (aucun chemin applicatif ne les
  produit — c'est le sujet) : écriture à une seule ligne, lignes déséquilibrées, date hors exercice.
- **Bornes de date** : `BETWEEN` inclusif aux deux bouts (K2, K4, K6) ; tests aux jours **limites** (premier et
  dernier jour de Y, veille et lendemain).
- **Multi-société** : chaque requête est scopée `company_id` ; un test par contrôle avec une seconde société qui
  porterait l'anomalie (elle ne doit pas apparaître).
- **Montants** : `Decimal` côté Rust, chaînes au JSON, `big.js` au frontend.
- **Base de gate** : remise à zéro inconditionnelle avant le gate complet (CLAUDE.md, KF-039) ; `kesh-db` touché →
  gate complet même en boucle de revue.
- **Données réelles** : aucune ; fixtures synthétiques (IBAN de test, montants ronds).
- **Mutation restaurée** : `git checkout -- f` **puis `touch f`** avant le gate suivant.

### Fichiers communs avec les autres stories de l'epic

`routes/reconciliation.rs` et les chemins d'écriture des comptes bancaires (29-2 à 29-4) ne sont **pas** touchés.
`bank_accounts.rs` l'est par la 29-1a (extraction de lecture seule) et par la 29-4 (#651, écrivains) : zones
distinctes, rebase attendu sans conflit ; si conflit, la 29-4 a priorité sur l'ordre des verrous, la 29-1a ne touche
que la lecture. `fiscal_years.rs` (AC3-bis) : la 29-1a n'y ajoute que des lectures sans verrou et factorise le
texte de l'étape (d) de `close`, sans en changer le SQL exécuté ; si une autre story de l'epic y touche, elle a
priorité, la 29-1a rebase. `CHANGELOG.md`, `api-external.md`, `messages.ftl` ×4 et `user-manual.tex` : fusion par
union.

### Tests

**29-1a**

- T-a1 — `for_fiscal_year` rend les sept codes, dans l'ordre de l'AC1, sur une société vierge (tous `ok` sauf K7 si
  aucun suivant).
- T-a2 — garde statique de l'AC2, **inversée** (C-29-1-26), ses **deux** volets : (a) tout site d'appel `query`,
  `query_as`, `query_scalar` du source hors tests — turbofish équilibré et appel coupé par rustfmt compris — a pour
  premier argument une constante de `ALL_SQL`, tout autre site est nommé comme non résolu ; nombre de `const …_SQL`
  = `ALL_SQL.len()` ; (b) aucun `FOR UPDATE` / `LOCK IN SHARE MODE` / `FOR SHARE` dans le source hors tests
  décommenté. Les fonctions de dépôt externes et leur SQL sont un **angle mort** de T-a2 (AC2 : nommées au
  doc-comment du module, vérifiées en revue de code). Le test porte
  aussi un **cas de contrôle** de son détecteur : appliqué à une chaîne de source synthétique qui contient
  `sqlx::query_as::<_, (i64, Option<String>)>(\n "SELECT 1 FOR UPDATE")`, le volet (a) rend un non résolu et le volet
  (b) un mot de verrou — un détecteur qui ne voit pas la forme dominante du dépôt rougit avant de servir.
- T-a3 — K1 : deux antérieurs ouverts → `attention` avec le **plus ancien** ; antérieurs tous clos → `ok`.
- T-a4 — K2 : pending au premier jour, au dernier jour, veille, lendemain ; rejetée comptée dans `rejectedCount` ;
  rapprochée non comptée ; autre société absente.
- T-a5 — K3, un cas par ligne de la table des motifs et par règle de population : `MISSING` (relevés à solde tous
  antérieurs à Y), `ENDS_BEFORE_YEAR_END`, `GAP` (transaction non rapprochée), `ok` (relevé au dernier jour,
  rapproché) ; relevé **postérieur** à la fin d'exercice ignoré (la borne) ; écart de `0.004` (aucun) et de
  `0.005` (écart de `0.01`) aux livres en `DECIMAL(19,4)` ; compte archivé, compte clos avant Y, compte **sans
  activité** dans Y, compte dont la seule activité est **postérieure** à Y : absents des deux listes ; compte créé
  (`created_at`) **après** Y mais portant des transactions de Y : examiné ; compte clos pendant Y mesuré à sa date de
  clôture ; `NOT_COMPARABLE` et `NO_BALANCE_STATEMENT` dans `notCheckable`, **statut `ok`** s'ils sont seuls ;
  **précédence** : un compte sans compte lié **et** sans relevé à solde rend `NOT_COMPARABLE` (non
  `NO_BALANCE_STATEMENT`), un compte CSV seul rend `NO_BALANCE_STATEMENT` (non `MISSING`), un compte lié dont
  **tous** les relevés à solde sont **postérieurs** à Y, avec une transaction dans Y, rend `MISSING` (non
  `NO_BALANCE_STATEMENT` : le quatrième `EXISTS`, sans borne, voit ces relevés — P3 R8) ; autre société absente.
  **Activité au grand livre** (C-29-1-24) : compte lié dont le seul relevé à solde est de l'exercice précédent, aucune
  transaction bancaire dans Y, une écriture manuelle datée dans Y sur le compte lié → examiné, `MISSING` ; même compte
  sans cette écriture → absent. **Fixtures des exclusions qui ne passent pas à vide** (P2 R6) : un compte archivé
  n'a, par la route d'archivage, aucune transaction (`count_transactions_for_bank_account`, `routes/bank_accounts.rs`
  l. 912) et un compte clos n'en reçoit plus après `closed_on` — le critère d'activité les écarterait donc **de
  lui-même**, et M-a4 survivrait. Les deux cas se posent par **SQL direct** avec une activité dans Y : compte
  `archived = TRUE` portant un `bank_imports` à `closing_balance` non nul et `period_to` dans Y **et** une
  `bank_transactions` datée dans Y ; compte à `closed_on` antérieur à `Y.start_date` portant une `bank_transactions`
  datée dans Y ; chacun absent des deux listes. Le test vérifie, **avant** d'appeler `for_fiscal_year`, que ces
  lignes existent (assertion de montage : sans elles, le cas ne prouve rien). ⚠️ **Et qu'examiné, le compte serait
  listé** (P3 R12) : un compte archivé dont le relevé s'arrête au dernier jour de Y, sans écart, ne produirait aucun
  motif et serait absent **avec ou sans** l'exclusion — M-a4 survivrait encore. Le relevé du compte archivé s'arrête
  donc **avant** `Y.end_date` (son compte lié, non partagé, le ferait passer en `ENDS_BEFORE_YEAR_END`), et un
  **cas de contrôle** pose la même fixture avec `archived = FALSE` : ce compte-là figure dans `accounts`, motif
  `ENDS_BEFORE_YEAR_END`. Le compte clos avant Y, sans `bank_imports`, figurerait, examiné, dans `notCheckable`
  (`NO_BALANCE_STATEMENT`) : son absence des deux listes est donc distinctive telle quelle.
- T-a6 — l'extraction (`balances_by_bank_account`, `balances_from_row`, `is_shared`) : les tests existants de
  `list_by_company_with_balances` — `crates/kesh-db/tests/bank_account_statement_gap.rs` et
  `crates/kesh-api/tests/bank_accounts_e2e.rs`, à nommer un par un dans le Dev Agent Record (liste recomptée par
  `grep -rln "list_by_company_with_balances\|/api/v1/bank-accounts" crates/*/tests/`) — verts **sans
  modification** ; un test neuf qui compare `None` et une borne postérieure à tous les relevés (mêmes valeurs), et un
  qui vérifie qu'une borne **antérieure** au dernier relevé désigne le relevé précédent (filtre **avant**
  `ROW_NUMBER`).
- T-a7 — K4 : brouillon au premier et au **dernier** jour de Y compté, validé ou annulé non compté, brouillon la
  veille et le lendemain de Y non compté.
- T-a8 — K5 : `to_complete` compté, `completed`/`discarded` non.
- T-a9 — K6 : les quatre motifs, posés en SQL direct, dont une écriture d'un autre exercice datée du **dernier** jour
  de Y et une écriture de Y datée du lendemain ; une écriture ordinaire n'y figure pas ; une écriture à la fois
  déséquilibrée et hors exercice compte **une fois** dans `count` et sous **deux** motifs de `countByReason` ; liste
  bornée à 20, dédupliquée, dans l'ordre `entry_date, id`.
- T-a10 — K7 : sans suivant → `attention` ; avec **deux** suivants N+1 et N+2 → `ok` et les données de **N+1** ;
  avec un antérieur seulement → `attention` (l'antérieur n'est pas un suivant).
- T-a11 — route : 200 tout rôle (Admin, Comptable, Consultation, PAT `read`), 404 autre société, 401 sans auth ;
  forme JSON (camelCase, montants en chaînes).
- T-a12 — Vitest : le panneau (sept lignes, liens, erreur ≠ « rien à signaler », **chargement en attente : l'état
  d'attente, aucune ligne de contrôle ni « rien à signaler »** — P3 R13, `notCheckable` affiché sans compter), le bouton *Préparer la clôture* visible pour le rôle **Consultation** (colonne d'actions rendue, sans
  *Renommer* / *Clôturer* / *Réouvrir*) et pour Admin, le `colspan` de la ligne vide, le rappel du dialogue (compte
  `info` compris, bouton actif pendant le chargement et après un échec, **texte d'attente pendant le chargement, ni « Aucun point à vérifier » ni « N point(s) »** — P4-1) — dans `fiscal-years-page.test.ts` adapté
  (Ta5) et un test du composant.
- T-a13 — E2E `fiscal-years.spec.ts` (ajout) : ouvrir *Préparer la clôture* sur un exercice avec une transaction
  pending, voir K2 en attention et le lien vers le rapprochement ; le bouton est présent pour un utilisateur
  Consultation.
- T-a14 — `statement_gap` (test unitaire de `bank_accounts.rs`) : les cas de `homepage.test.ts` l. 57-80 à
  `16eeed21`, à l'identique, plus `None` sans relevé ou sans solde comptable — ils y **remplacent** les cas
  TypeScript, qui partent (AC3).
- T-a15 — test unitaire de `fiscal_years.rs` : `earlier_open_locking_sql()` — la fonction même que `close` exécute à
  l'étape (d) — rend exactement l'ancienne constante `FIND_EARLIER_OPEN_SQL`, recopiée dans le test comme référence
  figée.
- T-a16 — l'écart servi au tableau de bord (AC3, C-29-1-25) : test **neuf** de `bank_accounts_e2e.rs` — `GET
  /api/v1/bank-accounts` rend `statementGap` en chaîne pour un écart réel et `null` sans relevé ; Vitest de
  `homepage.test.ts` — `statementGap` rend `null` pour `statementGap: null` et `Big(-200)` pour `"-200.00"`, sans
  recalcul.

**29-1b**

- T-b1 — `NextFiscalYearStart`, sur **trois** exercices ouverts N, N+1, N+2, origine dans N : date = `start_date` de
  N+1 (non de N+2), exercice = N+1, `reverses_entry_id` = origine, lignes inversées, lettrage des lignes libres,
  audit avec `"dating"`.
- T-b2 — `Today` inchangé : verts sans modification les tests existants de
  `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` et des annulations qui passent par la contre-passation —
  `crates/kesh-db/tests/reconciliation_cancel.rs`, `crates/kesh-db/tests/invoice_settlement.rs`,
  `crates/kesh-db/tests/supplier_invoices_repository.rs`, `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs`
  (la liste exacte se recompte au dev par `grep -rln "reverse_in_tx\|reverse_owned_in_tx\|/reverse" crates/*/tests/`
  et se nomme au Dev Agent Record) ; **assertion neuve** (P3 R6) — le seul test d'audit existant
  (`journal_entry_reversal_e2e.rs` l. 1120-1146) ne vérifie que la **présence** de l'identifiant : un test neuf, ou une
  assertion ajoutée à côté, exige que `details_json` du chemin `Today`, décodé, **égale exactement**
  `{"reversalJournalEntryId": <id>}` (objet à une seule clé, `journal_entries.rs` l. 2771-2773 à `16eeed21`).
- T-b3 — refus : sans suivant (`NEXT_FISCAL_YEAR_MISSING`), suivant clos (`NEXT_FISCAL_YEAR_CLOSED` — état posé par
  SQL direct, l'invariant I empêchant de clore N+1 avec N ouvert par les transitions ; et cas légal : origine dans N
  **clos**, N+1 clos), premier jour verrouillé (`PERIOD_LOCKED`), déjà contre-passée, contre-passation d'une
  contre-passation, écriture de pièce.
- T-b4 — garde statique de l'AC12 : dans le corps de `reverse_in_tx_inner`, la requête de l'exercice suivant apparaît
  **après** l'appel `reversal_blockers(`.
- T-b5 — concurrence (deux connexions), **N premier exercice de la société** (sans exercice antérieur à N — sinon
  c'est le cycle du test 13 d, déjà couvert, et non une attente : AC12), origine dans N (clos), N+1 ouvert. La vraie
  fonction est exercée, jamais une reconstitution à la main de ses étapes (précédent à ne pas suivre :
  `fiscal_years_repository.rs` l. 1844-1857, où W refait les temps de `reverse_in_tx_inner` — un tel test ne tuerait
  aucune mutation du code). Deux ordres :
  - **(i) la contre-passation d'abord** : A ouvre une transaction que le test possède et y exécute
    `reverse_at_next_fiscal_year_start_in_tx` **jusqu'au bout, sans valider** (inutile de s'arrêter à mi-fonction :
    les verrous pris sont tenus jusqu'au `COMMIT`) ; B lance `retry_on_deadlock(… fiscal_years::close(N+1))` dans une
    tâche ; le test attend B **en (b') sur N** par `attendre_une_requete_en_cours(&pool, &["SELECT id FROM
    fiscal_years WHERE id = ", "FOR UPDATE"], …)` — motif de `LOCK_EARLIER_BY_ID_SQL` (`fiscal_years.rs` l. 178-179),
    absent de l'étape (c) (`LOCK_IN_COMPANY_SQL`, l. 162-163, « SELECT id, company_id, … ») et de `create_in_tx_inner`
    (« SELECT id, status … ») : il distingue « B attend N » de « B attend N+1 » ; A valide ; B aboutit ; la
    contre-passation est dans N+1, N+1 est clos. Ce que l'ordre (i) établit : les deux gestes se **sérialisent**, sans
    interblocage, quand N est le premier exercice. ⚠️ **Il ne tue aucune mutation de la 29-1b, et on le déclare** :
    c'est un test de comportement, non de mutation. (A tient N à l'étape (1) ; il le tient peut-être aussi par le
    parcours de `find_open_covering_date` à l'étape (4), **si** le plan d'exécution lit N — ce qui dépend de l'index
    choisi, P2 F L2 : la fiche ne s'appuie pas sur ce second verrou) ;
  - **(ii) la clôture d'abord** : B ouvre une transaction du test qui prend le verrou de `LOCK_EARLIER_BY_ID_SQL` sur
    N puis celui de `LOCK_IN_COMPANY_SQL` sur N+1 et passe N+1 à `Closed` (les effets de `close`, qui n'a pas de
    variante en transaction), **sans valider** — ces deux constantes sont `const` **privées** de `fiscal_years.rs`
    (l. 162, 178) : le test de `crates/kesh-db/tests/` en **recopie** le texte, et ce couplage se déclare au
    doc-comment du test (les motifs du test 15-12a dépendent déjà de leurs listes de colonnes, Dev Notes « État
    actuel », P2 F L7) ; A lance la vraie `reverse_at_next_fiscal_year_start` (pool) dans une tâche ; le
    test attend A **à l'étape (1)** (motifs `"JOIN fiscal_years fy"`, `"FOR UPDATE"`) ; B valide ; A rend
    **`NEXT_FISCAL_YEAR_CLOSED`** (la vue de A naît après le `COMMIT` de B, la confirmation verrouillante lit N+1
    clos). Ce que l'ordre (ii) établit : une contre-passation qui attend une clôture ne s'écrit pas dans l'exercice
    qu'elle vient de clore, et le refus est **le sien**. Il **tue M-b11**. ⚠️ Il **ne tue pas M-b3 par la course** et
    ne la revendique plus (C-29-1-27, P2 R5 = F3) : la vue de A naissant après le `COMMIT` de B, il ne distingue rien
    que T-b3 ne distingue déjà — M-b3 est tuée par T-b3 seul, grâce à la prescription de l'AC11. Le côté B est monté à
    la main parce qu'il n'est pas le code testé.
- T-b6 — origine dans un exercice gappé (suivant qui ne commence pas au lendemain) : date = `start_date` du suivant.
- T-b7 — route : 201, 403 Consultation, 404 autre société, PAT `read-write` admis, rejeu inscrit au registre.
- T-b8 — Vitest (dans `journal-entry-page.test.ts` adapté, Tb4) : les cinq lignes de la table de l'AC15 (chargement
  en attente : ni bouton ni motif ; échec : motif d'indisponibilité ; pas de suivant ; suivant clos, nommé ; suivant
  ouvert : bouton), `reversable` faux et rôle Consultation (ni bouton ni motif), *Contre-passer* présent quand la
  liste échoue, la confirmation nomme date et exercice (format suisse), refus traduit.
- T-b9 — E2E de parcours (AC18), et son cas de refus.

### Mutations

| # | Mutation | Tuée par |
|---|---|---|
| M-a1 | K2 : `BETWEEN` → `>` sur `start_date` | T-a4 (premier jour) |
| M-a2 | K2 : retirer le filtre `company_id` | T-a4 (autre société) |
| M-a3 | K3 : retirer la borne `period_to <= ?` | T-a5 (relevé postérieur) |
| M-a4 | K3 : ne pas exclure les comptes archivés | T-a5 (compte archivé **avec activité dans Y posée par SQL direct** — sans elle, le critère d'activité l'écarte de lui-même et M-a4 survit, P2 R6 — et relevé arrêté **avant** `Y.end_date`, si bien qu'examiné il serait listé ; cas de contrôle non archivé, P3 R12) |
| M-a5 | extraction : borne appliquée aussi à `None` (ex. `COALESCE(?, '1970-01-01')`) | T-a6 |
| M-a6 | K4 : compter `validated` | T-a7 |
| M-a7 | K6 : `HAVING SUM(debit) <> SUM(credit)` retiré | T-a9 |
| M-a8 | K6 : motif `OTHER_YEAR_DATED_INSIDE` retiré | T-a9 |
| M-a9 | K7 : `>` → `>=` sur `start_date` | ⛔ **équivalente, déclarée** : les deux prédicats ne diffèrent que pour un exercice commençant **le jour même** de `Y.end_date` ; il chevaucherait Y, ce qu'aucun chemin de création ne laisse naître (P2 R14) : `create` (l. 313) et `create_for_seed` (l. 394) appellent `find_overlapping` (intervalles fermés, l. 789 et suiv.) et refusent le chevauchement ; `create_if_absent_in_tx` (l. 438) n'insère que si la société n'a **aucun** exercice (`WHERE NOT EXISTS`), donc sans chevauchement possible ; l'import d'installation réinsère telles quelles des lignes issues d'une base tenue par ces mêmes chemins — aucun état atteignable ne la distingue, aucun test ne peut la tuer. (La justification d'origine, « Y se trouve lui-même », était fausse : `Y.start_date < Y.end_date`.) Remplacée, pour K7, par M-a11 |
| M-a10 | panneau : erreur de chargement affichée comme « rien à signaler » | T-a12 |
| M-a11 | K7 / `find_next_after` : `ORDER BY start_date DESC` (rend le dernier exercice, non le suivant) | T-a10 (N+1 et N+2 présents) |
| M-a12 | K2 : borne haute `<= Y.end_date` → `< Y.end_date` | T-a4 (dernier jour) |
| M-a13 | K4 : borne haute `<= Y.end_date` → `< Y.end_date` | T-a7 (dernier jour) |
| M-a14 | K6 : `OTHER_YEAR_DATED_INSIDE` borné `< Y.end_date` | T-a9 (écriture d'un autre exercice datée du dernier jour) |
| M-a15 | K3 : précédence — `MISSING` évalué avant `NO_BALANCE_STATEMENT` (un compte CSV seul passe en `attention`) | T-a5 (précédence) |
| M-a16 | `statement_gap` sans arrondi (écart brut des deux termes) | T-a14 (`100.0049` / `100.00`) et T-a5 (`0.004`) |
| M-a17 | K3 : critère d'activité retiré (tout compte non archivé ni clos avant Y examiné) | T-a5 (compte sans activité dans Y) |
| M-a18 | K1 : `find_earliest_earlier_open` en `ORDER BY start_date DESC` (le plus récent des antérieurs ouverts) | T-a3 (deux antérieurs ouverts) |
| M-a19 | `closing_checks.rs` : une requête écrite en littéral turbofish coupé par rustfmt, `sqlx::query_as::<_, (i64,)>(` puis `"SELECT … FOR UPDATE"` à la ligne suivante | T-a2 volets (a) et (b) (P2 R4 = F1) |
| M-a20 | K3 : critère d'activité au grand livre retiré (seuls transactions et relevés comptent) | T-a5 (écriture manuelle sur le compte lié, relevé ancien : `MISSING` attendu) |
| M-a21 | DTO du tableau de bord : `statement_gap: None` en dur dans le handler de liste | T-a16 (`bank_accounts_e2e.rs`, écart réel attendu) |
| M-a22 | panneau : « Rien à signaler » affiché sur chaque ligne pendant le chargement ; **et rappel du dialogue : « Aucun point à vérifier » affiché pendant le chargement** (P4-1) | T-a12 (chargement en attente, P3 R13, panneau et rappel) |
| M-a23 | page : garde du jeton retirée au succès de `openChecks` *(revue de code P1, E1 = A1 ; M-a23 à M-a41 ajoutées par la remédiation)* | `fiscal-years-page.test.ts` (réponse tardive réussie, panneau) |
| M-a24 | page : garde du jeton retirée à l'échec de `openChecks` | `fiscal-years-page.test.ts` (réponse tardive en échec, panneau) |
| M-a25 | page : garde du jeton retirée au succès de `loadCloseChecks` | `fiscal-years-page.test.ts` (réponse tardive réussie, rappel) |
| M-a26 | page : garde du jeton retirée à l'échec de `loadCloseChecks` | `fiscal-years-page.test.ts` (réponse tardive en échec, rappel) |
| M-a27 | K6 : `UNBALANCED` non borné aux écritures de Y | `t_a9_k6_perimetre_de_y_et_bornes` |
| M-a28 | K6 : `TOO_FEW_LINES` non borné aux écritures de Y | `t_a9_k6_perimetre_de_y_et_bornes` |
| M-a29 | K6 : `DATE_OUTSIDE_ITS_YEAR` sans la borne basse (`entry_date < ?` neutralisé) | `t_a9_k6_perimetre_de_y_et_bornes` (veille de Y) |
| M-a30 | K6 : `entry_date < ?` → `<=` | `t_a9_k6_perimetre_de_y_et_bornes` (premier jour de Y) |
| M-a31 | K6 : `OTHER_YEAR_DATED_INSIDE` borné `> Y.start_date` | `t_a9_k6_perimetre_de_y_et_bornes` (premier jour de Y) |
| M-a32 | K6 : `COALESCE(l.n, 0) < 2` → `l.n < 2` | `t_a9_k6_perimetre_de_y_et_bornes` (écriture sans ligne) |
| M-a33 | K6 : `WHERE` de population élargi (`… OR TRUE`) | ⛔ **équivalente, démontrée** (C-29-1-44) : chaque drapeau porte sa condition sur Y, le `HAVING` écarte toute écriture hors population |
| M-a34 | K3 : `closed_on >= ?` → `>` | `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs` (clos le premier jour de Y) |
| M-a35 | K3 : `measured_through` sans `min(Y.end_date)` | `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs` (clos après Y) |
| M-a36 | K3 : motifs 4 et 5 permutés | `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs` (arrêté trop tôt et en écart), `t_a6_…` |
| M-a37 | `NEXT_AFTER_SQL` : portée société neutralisée | `t_a10_k7_exercice_suivant` (exercice d'une autre société) |
| M-a38 | `EARLIER_OPEN_SQL` : portée société neutralisée | `t_a3_k1_…` (antérieur ouvert d'une autre société), `close_ignores_the_open_years_of_another_company` |
| M-a39 | K4 : `ORDER BY` retiré (et M-a39b : `ORDER BY i.id`) | `t_a7_k4_liste_bornee_et_ordonnee` |
| M-a40 | K4 : départage `i.id DESC` | `t_a7_k4_liste_bornee_et_ordonnee` |
| M-a41 | K4 : plafond ignoré (`LIMIT` lié à 2000) | `t_a7_k4_liste_bornee_et_ordonnee` |
| M-a42 | K6 : borne haute de `DATE_OUTSIDE_ITS_YEAR` (`je.entry_date > ?` → `>= ?`) *(revue P2 ciblée, F2)* | `t_a9_k6_perimetre_de_y_et_bornes` (écriture de Y datée du dernier jour) |
| M-b1 | `NextFiscalYearStart` daté `today` | T-b1 |
| M-b2 | lecture du suivant en `ORDER BY start_date DESC` (rend le dernier exercice, non le suivant) — ⚠️ la lecture étant `find_next_after`, partagée avec K7, c'est la mutation M-a11 vue du chemin de la 29-1b | T-b1 (N+2 présent) |
| M-b3 | confirmation verrouillante retirée, **aucun** contrôle de statut (date et exercice pris de la lecture ordinaire) | T-b3 (le code attendu est `NEXT_FISCAL_YEAR_CLOSED` ; sans confirmation, `create_in_tx_inner` rendrait `FISCAL_YEAR_CLOSED`) — tuée **parce que** l'AC11 interdit de juger le statut sur la lecture ordinaire ; T-b5 (ii) ne la revendique plus (P2 R5 = F3) |
| M-b4 | `Some(id)` → `None` (liaison perdue) | T-b1 |
| M-b5 | audit `dating` ajouté aussi au chemin `Today` | T-b2 (assertion **neuve** d'égalité exacte du JSON, P3 R6 — le test existant ne la tuerait pas) |
| M-b6 | `NextFiscalYearClosed` rendu en `FiscalYearInvalid` | T-b3 |
| M-b7 | route non rejouée | T-b7 (registre) |
| M-b8 | bouton affiché sans exercice suivant ouvert | T-b8 |
| M-b9 | lecture du suivant placée avant `reversal_blockers` | T-b4 |
| M-b10 | exercice suivant absent : repli sur la date du jour au lieu de `NextFiscalYearMissing` | T-b3 (sans suivant : `409 NEXT_FISCAL_YEAR_MISSING` attendu, non une écriture créée) |
| M-b11 | étape (1) : le `JOIN fiscal_years fy` **retiré** de la requête verrouillante (seule l'écriture verrouillée) ; le nom et `end_date` de l'exercice de l'origine lus par une requête **séparée, sans `FOR UPDATE`** — retirer seulement `end_date` du `SELECT` laisserait le `JOIN` verrouillant en place et T-b5 (ii) vert (P3 F2) | T-b5 ordre (ii) (A n'attend plus à l'étape (1) : l'attente au motif `"JOIN fiscal_years fy"` ne se produit pas, le test rougit) |
| M-b12 | motif de l'AC15 « aucun exercice ne suit » affiché pendant le chargement de la liste | T-b8 (chargement : ni bouton ni motif) |

*(Validation P3, 2026-10-14, décision de l'orchestrateur : la ligne M-b13 est retirée de cette table, et rien ne la
remplace — C-29-1-27.)*

### Project Structure Notes

- Lecture neuve dans `kesh-db/src/repositories/closing_checks.rs` (les rapports de `kesh-report` sont des états
  comptables ; ceci est un contrôle de données, proche des dépôts qu'il lit). Aucune migration, aucune table.
- Composant frontend `features/fiscal-years/FiscalYearChecks.svelte` ; la page n'en porte que l'appel.

### References

- Issue #232 ; fichier d'epic `planning-artifacts/epic-29-preparer-cloture.md` ; registre
  `epic-29-choix-autonomes.md` (C-29-0-1 à 3) et `epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 30).
- Fiches `14-1-cloture-report-des-soldes.md` (modèle virtuel), `14-2-reouvrir-degiger-exercice.md`,
  `14-4-bilan-ouverture-soldes-depart.md`, `25-6-a-tableau-de-bord.md` (écart du relevé), `28-3-sauvegarde-instantane-commun.md` (instantané),
  `28-4-cloture-compte-bancaire.md` (critère de l'AC7, forme de fiche).
- Rétrospective `epic-28-retro-2026-10-14.md` § 4.2 (naissance de la vue), § 4.4 (mots-clés), § 7 (B3, B4, B5).

## Dev Agent Record

### Agent Model Used

29-1a : Opus 5.5 (`bmad-dev-story`, autonomie, worktree `/home/gcorbaz/devel/kesh-29-1`, bases `kesh_291` /
`kesh_e2e_291`, port 3042). 29-1b : (à remplir).

### Debug Log References

Journaux (hors dépôt) `/home/gcorbaz/devel/kesh-gate-logs/291a-*` : `291a-red.log` (phase rouge), `291a-gate1.log`
(gate backend intermédiaire), `291a-vitest1.log`, `291a-e2e-target1.log`, `291a-mutations.log` (deux passes),
`291a-final-{summary,backend,check,lint,vitest,build,e2e}.log` (gate final au dernier commit de code). Scripts
(scratchpad de la session, non versés) : `291a-reset-db.sh`, `291a-e2e.sh`, `291a-mutations.py`,
`291a-final-gate.sh`. Captures : `kesh-gate-logs/29-1a-captures/`.

### 29-1a — compte rendu de développement (2026-10-14)

**Commits** (branche `story/29-1`, base `fcd2ae75`) : `ae6d9419` (serveur), `bf5f8238` (écran, i18n, documentation),
`7c643eba` (T-a15 : constante renommée) — **dernier commit de code** — puis un commit de comptes rendus (fiche,
registre, `sprint-status.yaml`, prompt de revue).

**T0 (AC2, mesure)** : MariaDB 10.11.16 **refuse** `SELECT 1 FROM fiscal_years LIMIT 1 FOR UPDATE` sur
`ReadSnapshot::conn()` — erreur **1792** (`Cannot execute statement in a READ ONLY transaction`, SQLSTATE `25006`),
mesuré au client `mariadb` puis fixé par le test `t0_un_verrou_dans_l_instantane_est_refuse_par_le_serveur`. La garde
statique T-a2 reste, comme prescrit.

**Tests rouges d'abord** — constatés sur un talon (`for_fiscal_year` rendant `NotFound`, `statement_gap` rendant
`None`), sortie de `291a-red.log` :

```
     Summary [   1.449s] 22 tests run: 4 passed, 18 failed, 1201 skipped
        PASS  kesh-db repositories::fiscal_years::tests::le_texte_verrouillant_de_la_cloture_est_inchange
        PASS  kesh-db repositories::closing_checks::tests::t_a2_le_detecteur_voit_la_forme_dominante
        PASS  kesh-db::closing_checks un_exercice_d_une_autre_societe_est_introuvable
        PASS  kesh-db::closing_checks t0_un_verrou_dans_l_instantane_est_refuse_par_le_serveur
        FAIL  kesh-db repositories::bank_accounts::tests::l_ecart_du_releve_est_arrondi_au_centime
        FAIL  kesh-db repositories::closing_checks::tests::t_a2_aucune_requete_hors_all_sql_et_aucun_verrou
        FAIL  kesh-db::closing_checks t_a1 … t_a10 (16 tests : « called `Result::unwrap()` on an `Err` value: NotFound »)
```

Les quatre verts le sont par nature (T-a15 : non-régression d'une factorisation déjà faite ; le cas de contrôle du
détecteur ; le `NotFound` attendu ; la mesure T0). ⚠️ **Écart** : T-a11 (`closing_checks_e2e.rs`), T-a16
(`bank_accounts_e2e.rs`), les Vitest de T-a12 et l'E2E T-a13 ont été écrits **après** le code qu'ils couvrent ; leur
pouvoir de détection est établi par les mutations M-a21 (T-a16), M-a10 et M-a22 (T-a12), non par une phase rouge.

**Fichiers de test existants** — restés **sans modification** (AC3, T-a6) : `crates/kesh-db/tests/bank_account_statement_gap.rs`
(recompte `grep -rln "list_by_company_with_balances\|/api/v1/bank-accounts" crates/*/tests/` : ce fichier,
`crates/kesh-api/tests/api_keys_e2e.rs`, `bank_accounts_closed_e2e.rs`, `bank_accounts_e2e.rs` — et
`crates/kesh-db/tests/closing_checks.rs`, neuf ; `git diff --stat fcd2ae75 7c643eba` sur ces fichiers ne montre que
`bank_accounts_e2e.rs`, **+71 lignes sans suppression** — T-a16 ajouté — et le fichier neuf).
**Modifiés** (Ta5) : `fiscal-years-page.test.ts` (mock `getClosingChecks`, dix cas neufs) ; `homepage.test.ts` (les
quatre cas d'arrondi partis en Rust, deux cas de lecture du champ) ; `HomepageCards.test.ts` (fabrique, et le cas
« l'écart seulement s'il est non nul » reçoit `statementGap: '-200.00'`) ; fabriques complétées de `statementGap: null`
dans `bank-accounts.api.test.ts` (plus deux assertions), `bank-accounts-page.test.ts`,
`BankAccountJournalLinkForm.test.ts`, `payment-batches-page.test.ts` (liste recomptée par `grep -rln
"ledgerBalanceAtStatement" frontend/src`) ; bornes des gardes `i18n-keys.test.ts` (`sitesTotal` 2107 → 2162) et
`i18n-libelle-en-dur.test.ts` (63 → 68 candidates, `conforme` 49 → 54), ventilées dans leurs commentaires.

**Tests neufs, recomptés aux deux bornes `fcd2ae75` → `7c643eba`** (`git show <ref>:<f> | grep -cE …`) : Rust
**25** — `kesh-db/tests/closing_checks.rs` 0 → 18, `closing_checks.rs` (module) 0 → 2, `bank_accounts.rs` 1 → 2,
`fiscal_years.rs` 0 → 1, `kesh-api/tests/closing_checks_e2e.rs` 0 → 2, `bank_accounts_e2e.rs` 42 → 43 ; Vitest **+19**
— `FiscalYearChecks.test.ts` 0 → 11, `fiscal-years-page.test.ts` 12 → 22, `homepage.test.ts` 10 → 8 ; E2E **+3**
exécutés — `fiscal-years.spec.ts` 6 → 8 déclarations `test(`, dont une boucle clair / sombre (6 → 9 exécutés).
Cohérents avec les gates : nextest 3604 → 3629, Vitest 1610 → 1629.

**Gate final au dernier commit de code `7c643eba`** (bases remises à zéro d'abord ; `291a-final-*.log`) :
- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **3629 passés, 6 ignorés**, vert ;
- `npm run check` : 0 erreur, 29 avertissements, **aucun** dans un fichier de la story ;
- `npm run lint-i18n-ownership` : vert ; `npm run test:unit` : **147 fichiers, 1629 tests**, verts ; `npm run build` :
  vert ;
- **E2E complet** (`npm run test:e2e`, port 3042, base `kesh_e2e_291`) : **283 passés, 7 échecs, 19 ignorés** (11,4 min).
  Les 7 échecs, fichier par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les sept de la **KF-029 (#97)**,
  aucun autre ; ni huitième variable, ni KF-052 sur ce run. `fiscal-years.spec.ts` : 9/9 dans le run complet (et 9/9 au
  gate ciblé, `291a-e2e-target1.log`).
- Gates ciblés intermédiaires : `291a-gate1.log` (backend complet sur `ae6d9419`, 3629 / 6 ignorés), Vitest complet
  (`291a-vitest1.log`, trois gardes de décompte rouges, corrigées), gardes `kesh-i18n` et `textes_coherents`
  (55 / 55 après reformulation, C-29-1-40).

**Mutations** (`291a-mutations.py`, journal `291a-mutations.log`) — chacune par motif vérifié **unique**, restaurée par
`git checkout -- <f>` puis `touch <f>` ; **21 exécutées, 21 tuées**, M-a9 déclarée équivalente (fiche), non exécutée :

| # | Tuée par |
|---|---|
| M-a1, M-a2, M-a12 | `t_a4_k2_transactions_en_attente` |
| M-a3 | `t_a5_k3_motifs_et_borne`, `t_a6_…`, `t_a5_k3_non_verifiables_et_precedence` |
| M-a4 | `t_a5_k3_population_des_comptes_examines` |
| M-a5 | `t_a6_la_borne_designe_le_releve_precedent` et quatre tests de `bank_account_statement_gap.rs` |
| M-a6, M-a13 | `t_a7_k4_brouillons` |
| M-a7, M-a8, M-a14 | `t_a9_k6_ecritures_incoherentes` |
| M-a10 | `FiscalYearChecks.test.ts` (échec « indisponible ») et `fiscal-years-page.test.ts` |
| M-a11 | `t_a10_k7_exercice_suivant` |
| M-a15 | `t_a5_k3_non_verifiables_et_precedence` |
| M-a16 | `l_ecart_du_releve_est_arrondi_au_centime` (T-a14), `t_a5_k3_ecart_au_centime` |
| M-a17 | `t_a5_k3_population_…`, `t_a5_k3_activite_au_grand_livre` |
| M-a18 | `t_a3_k1_le_plus_ancien_anterieur_ouvert`, `le_texte_verrouillant_de_la_cloture_est_inchange` |
| M-a19 | `t_a2_aucune_requete_hors_all_sql_et_aucun_verrou` |
| M-a20 | `t_a5_k3_activite_au_grand_livre` |
| M-a21 | `list_bank_accounts_serves_the_statement_gap` (T-a16) |
| M-a22 (a : panneau, b : rappel) | `FiscalYearChecks.test.ts`, `fiscal-years-page.test.ts` (chargement en attente) |

⚠️ Première passe : M-a3, M-a5, M-a18 **non appliquées** (motif trouvé deux fois — un doc-comment, la référence figée
de T-a15) ; M-a19 en échec de **compilation** (`query_as::<_, …>` d'une requête non exécutée ne s'infère pas), donc
**non comptée** comme tuée. Seconde passe, motifs précisés et turbofish `<sqlx::MySql, (i64,)>` : les quatre tuées par
un test, `exit 100`.

**Recette visuelle** : `29-1a-panneau-{clair,sombre}.png` et `29-1a-rappel-cloture-{clair,sombre}.png` (E2E T-a13) ;
les « sombres » sont **identiques octet pour octet** aux claires (`cmp`) — Kesh n'a pas de thème sombre (C-28-1-7,
C-29-1-39).

**Manuel** : § *Préparer la clôture* (`\label{sec:preparer-cloture}`), phrase « si clôturé » corrigée, marche de
clôture, résumé annuel ; **PDF régénéré** (`make fr`, sous verrou) et contrôlé aplati — « Préparer la clôture » ×4,
« se termine au plus tard le dernier jour de l’exercice » ×2, « arrêté au dernier jour de l’exercice »,
« ne sont pas bornées à l’exercice », « que celui-ci soit clôturé ou non », « autre société » ×0. Les PDF du manuel
d'administration et de la brochure, recompilés sans changement de source, sont restaurés (C-29-1-42).

**Propagation** : `grep -rnF "FIND_EARLIER_OPEN_SQL" crates frontend/src docs` → 0 (la référence figée de T-a15
renommée en `REFERENCE_FIGEE`, commit `7c643eba`) ; `grep -rn "toCentime" frontend/src` → 0 ; replis du frontend
égaux aux valeurs fr-CH : 55 appels comparés, 0 écart ; 53 clés × 4 locales (`grep -c '^fiscal-years-checks-'`).

**Écarts à la fiche** :
1. Tests écrits après le code (voir *Tests rouges d'abord*).
2. Le module de tests de `closing_checks.rs` (garde T-a2) a été **ajouté en fin de fichier par `cat >>`** (ajout,
   aucune substitution de motif) — la consigne veut l'outil d'édition seulement. Le contenu a été relu et compilé.
3. `balances_from_row` est `pub(crate)` (C-29-1-32), l'AC3 l'écrivait privée.
4. Dans `+page.svelte`, le bloc des boutons *Clôturer* / *Réouvrir* garde une tabulation de trop après le déplacement
   (cosmétique, sans effet au rendu ; une tentative de réindentation par l'outil d'édition n'a pas trouvé le motif).
5. `api-external.md` reçoit une ligne *Exercices comptables* au tableau des ressources et nomme `statementGap` dans la
   note ¹⁰ (C-29-1-41) ; l'AC3 disait « rien à y ajouter » pour les champs du relevé.
6. Trois formulations reprises pour G8 / G8-bis, G11 et G18 (C-29-1-40).

**Issues à ouvrir** : aucune. **Demandé à l'orchestrateur** : push de `story/29-1`, PR « Story 29-1a : préparer la
clôture — contrôles préalables (refs #232) », **jamais** `closes` (le message de squash se relit avant merge) ;
rappeler dans le corps de la PR que l'entrée du CHANGELOG est sous `[0.14.0] — Non publié` et passe en 0.13.4 si ce tag
suit la fusion (AC9) ; revue de code P1 (`29-1a-review-prompt-p1.md`).

### 29-1a — remédiation de la revue de code P1 (2026-10-14)

*Annotations datées 2026-10-14 sur le compte rendu ci-dessus, qui n'est pas réécrit :*
- *Le rebase de `story/29-1` sur `origin/main` (`c36f39bc`, Story 29-2) a changé les identifiants : `ae6d9419` →
  `7b74c14c`, `bf5f8238` → `76e3a8fd`, `7c643eba` → `49da014a`, `32598e69` → `34750f1f`. Conflits réunis par union
  (`sprint-status.yaml` à chaque commit ; CHANGELOG, les deux sections ; PDF du manuel, régénéré ensuite).*
- *F2 (revue P1, lentille A) : « `grep -rn "toCentime" frontend/src` → 0 » était inexact — ce grep rend deux lignes,
  `frontend/src/lib/features/bank-accounts/format.ts:35` et `:39` (`toCentimeNumber`, autre fonction, légitime). Ce qui
  valait 0, c'est le jeton `toCentime(` : `grep -rnF "toCentime(" frontend/src` → 0.*

**Commits** : `8b5e1a3f` (tests et doc-comments — **dernier commit de code**), `1c15363e` (CHANGELOG, `api-external.md`,
manuel `.tex` et PDF), puis ce compte rendu. **Aucune ligne exécutable de production n'a changé** : sur
`git diff -U0 34750f1f 1c15363e -- crates/kesh-db/src frontend/src/lib frontend/src/routes ':(exclude)*.test.ts'`, toute
ligne ajoutée ou retirée est un commentaire (filtre `grep -vE '^[+-]\s*//'` : sortie vide).

**Tests neufs**, recomptés aux deux bornes `34750f1f` → `8b5e1a3f` (`git show <ref>:<f> | grep -c …`) :
`crates/kesh-db/tests/closing_checks.rs` 18 → **21** `#[sqlx::test]` (`t_a5_k3_bornes_de_cloture_et_ordre_des_motifs`,
`t_a9_k6_perimetre_de_y_et_bornes`, `t_a7_k4_liste_bornee_et_ordonnee`) et deux tests étendus (`t_a3_…` : antérieur
ouvert d'une autre société ; `t_a10_…` : exercice suivant d'une autre société) ; `fiscal-years-page.test.ts` 22 → **26**
`it(` (bloc « Jeton de requête »). Écrits contre un code déjà juste : ils passent au premier lancement, et ce sont les
mutations qui établissent qu'ils mordent.

**Mutations rejouées** (`291a-p1-mut-front.py` / `291a-p1-mut-back.py`, scratchpad ; journaux
`kesh-gate-logs/291a-p1-mutations-{front,back}.log`), chacune par motif vérifié unique, restaurée par
`git checkout -- <f>` puis `touch`, sous verrou : **20 exécutées** (M-a23 à M-a41, M-a39 en deux formes), **19
tuées**, **1 équivalente** (M-a33, démontrée, C-29-1-44) :

| # | Tuée par |
|---|---|
| M-a23, M-a24, M-a25, M-a26 | chacune par **son** cas du bloc « Jeton de requête » (`1 failed | 27 passed`) |
| M-a27 à M-a32 | `t_a9_k6_perimetre_de_y_et_bornes` |
| M-a33 | survit (`77 passed`) — équivalente : chaque drapeau porte sa condition sur Y |
| M-a34, M-a35 | `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs` |
| M-a36 | `t_a5_k3_bornes_de_cloture_et_ordre_des_motifs`, `t_a6_la_borne_designe_le_releve_precedent` |
| M-a37 | `t_a10_k7_exercice_suivant` |
| M-a38 | `t_a3_k1_le_plus_ancien_anterieur_ouvert`, `close_ignores_the_open_years_of_another_company` |
| M-a39, M-a39b, M-a40, M-a41 | `t_a7_k4_liste_bornee_et_ordonnee` |

Portée société d'`EARLIER_OPEN_SQL` (E4) : déjà tenue par `close_ignores_the_open_years_of_another_company` (Story
15-12a, texte partagé avec `close`), et désormais aussi par `t_a3_…` — M-a38 rougit les deux.

**Gate complet au dernier commit de code** (rejoué sur `1c15363e`, qui n'ajoute que des documents ; bases remises à
zéro d'abord ; `kesh-gate-logs/291a-p1-final-*.log`) :
- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **3639 passés, 6 ignorés**, vert (3629 au P1 + 3
  tests neufs + ceux de la 29-2 apportés par le rebase) ;
- `npm run check` : 0 erreur, 29 avertissements ; `npm run lint-i18n-ownership` : vert ; `npm run test:unit` : **147
  fichiers, 1633 tests**, verts (1629 + 4) ; `npm run build` : vert ;
- **E2E complet** (port 3042, `kesh_e2e_291`) : **283 passés, 7 échecs, 19 ignorés** (11,5 min). Les 7 échecs, fichier
  par fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — les sept de la KF-029 (#97), aucun
  autre. `fiscal-years.spec.ts` : 9/9.
- Gates ciblés intermédiaires : Vitest du fichier de la page (28/28), `binary(closing_checks)` (21/21),
  `binary(textes_coherents)` (15/15), clippy du workspace.

**Manuel** : PDF régénéré (`make fr`, sous verrou, `291a-p1-make-fr.log`) et contrôlé aplati — « Créer l’exercice
suivant » ×1, « liste que les vingt premières » ×1, « non les avoirs » ×1, « même s’il laisse un trou » ×1, « sans
écriture de report » ×1 ; « Un écart inférieur au centime » ×0 ; « à la clôture. » ×0.

**Findings traités** (rapports `kesh-gate-logs/29-1a-review-p1-{B,E,A}.md`) — MEDIUM : E1 = A1 (jeton), E2 = A1 (K3),
E3 = A1 (K6), E4 (portée société) : **tests et mutations, aucun changement de production**. LOW : B-1 (avoirs, C-29-1-45),
B-2 = E LOW-4 = F7 (formes de T-a2, C-29-1-46), B-3 = E LOW-5 (coût de K6, C-29-1-47), E LOW-1 = A1 (K4 ordre et
plafond : test), E LOW-2 = F9 (troncature au manuel et à l'API), E LOW-3 (demi-centime), E LOW-6 (K7 après un trou),
F2 (annotation ci-dessus), F3 (glossaire), F4 (bouton K7 au manuel), F5 (`///` des types publics, forme JSON
inchangée), F6 (`sprint-status.yaml`), F8 (v0.13.4, C-29-1-43). **Refusé** : aucun. **Non fait, et dit** : la
troncature n'est pas dite **à l'écran** (F9 la classait « amélioration, pas écart ») — le manuel et l'API la disent ;
l'afficher changerait une ligne de production.

**Passe suivante** : passe ciblée P2, `29-1a-review-prompt-p2-ciblee.md` (Haiku, une lentille, sur le seul diff
`34750f1f..1c15363e`).

**Passe ciblée P2, remédiée** (rapport `kesh-gate-logs/29-1a-review-p2-ciblee.md`, 0/0/2/2 ; commit de code `7b1fd8b8`,
dernier commit de code de la story) :
- **F1** : le mécanisme de K3 est décrit exactement — chacun des deux soldes est arrondi au centime *avant* la
  soustraction — dans la doc du module (`closing_checks.rs`) et au manuel (`user-manual.tex`, PDF régénéré par `make fr`
  sous verrou, `291a-p2-make-fr.log`, contrôlé aplati : « des livres à 100,004 et un relevé à 99,996 (différence brute
  0,008) » ×1). Les deux exemples (100,004 / 99,996 → aucun écart ; 100,005 / 100,004 → 0,01) sont **assertés** dans
  `l_ecart_du_releve_est_arrondi_au_centime` (`bank_accounts.rs`, module `tests`, verts).
- **F2** : une écriture de Y datée du 31/12/2026 ajoutée à la fixture de `t_a9_k6_perimetre_de_y_et_bornes` (attendue
  absente) ; **M-a42** (`je.entry_date > ?` → `>= ?`) appliquée, `1 failed` sur ce test (`binary(closing_checks)` :
  20 passés, 1 échoué), restaurée par `git checkout --` puis `touch`. Cumul des mutations de la 29-1a : 21 exécutées
  depuis la revue P1 (M-a23 à M-a42, M-a39 en deux formes), 20 tuées, 1 équivalente (M-a33).
- **F3** : section `## [0.14.0] — Non publié` vide retirée du CHANGELOG (la 29-1b la recréera). **F4** : commentaire de
  `FiscalYearChecks.svelte` aligné sur « Créer l’exercice suivant ».
- Aucune ligne exécutable de production changée : le diff de production est un doc-comment, un test de module et un
  commentaire Svelte.

**Gates au dernier commit de code `7b1fd8b8`** (bases remises à zéro, migrations et seed rejoués ;
`kesh-gate-logs/291a-p2-*.log`) :
- `scripts/test-fast.sh` : **3639 passés, 6 ignorés**, vert ;
- `npm run check` : 0 erreur, 29 avertissements ; `lint-i18n-ownership` : vert ; `npm run test:unit` : **147 fichiers,
  1633 tests**, verts ; `npm run build` : vert ;
- **E2E complet** (port 3042, `kesh_e2e_291`) : **282 passés, 8 échecs, 19 ignorés** (12,4 min). Contre `docs/testing.md`
  § *Les échecs attendus* : les sept de la KF-029 (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`,
  `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) **plus** `sidebar-navigation.spec.ts:75`, listé (KF-052 ou pollution) :
  **rejoué seul, il passe** (4/4, `291a-p2-e2e-sidebar.log`) — c'est donc le huitième variable de pollution, non la
  KF-052 ni une régression. `fiscal-years.spec.ts` : aucun échec.

**Registre versé** : le contenu de `epic-29-choix-autonomes-29-1.md` (à partir de son premier titre) est ajouté à la fin
de `epic-29-choix-autonomes.md`, précédé de `---` ; le fichier propre est conservé (la 29-1b le complétera).

### Completion Notes List

- 2026-10-14 — fiche créée (analyse de contexte complète, guide de développement), deux clés `ready-for-dev`.
- 2026-10-14 — validation P1 remédiée (25 findings, Change Log) ; validation P2 à lancer.
- 2026-10-14 — validation P2 remédiée (30 findings, Change Log), signal D5 levé et déclaré ; AC19 retirée (Story 29-7) ;
  validation P3 à lancer.
- 2026-10-14 — validation P3 remédiée (17 findings, Change Log), signal D5 au troisième tour : M-b13 et volets (c)/(d)
  de T-a2 retirés (décisions de l'orchestrateur) ; passe ciblée P4 à lancer.
- 2026-10-14 — **29-1a développée** : AC1 à AC9 et AC3-bis ; gate complet et E2E complet verts au dernier commit de
  code `7c643eba` (E2E : les sept échecs attendus de la KF-029) ; 21 mutations tuées sur 21, M-a9 équivalente ; clé en
  `review`, revue de code P1 à lancer.
- 2026-10-14 — revue de code P1 remédiée (tests et mutations, doc-comments, textes) ; passe ciblée P2 à lancer.
- 2026-10-14 — passe ciblée P2 remédiée (0/0/2/2), revue close, clé `29-1a-controles-prealables` en `done` ; registre
  versé à celui de l'epic.

### File List

**29-1a** — neufs : `crates/kesh-db/src/repositories/closing_checks.rs`, `crates/kesh-db/tests/closing_checks.rs`,
`crates/kesh-api/tests/closing_checks_e2e.rs`, `frontend/src/lib/features/fiscal-years/FiscalYearChecks.svelte`,
`FiscalYearChecks.test.ts`, `closing-checks-labels.ts`, `closing-checks.test-fixtures.ts`. Modifiés :
`crates/kesh-db/src/repositories/{bank_accounts,fiscal_years,mod}.rs`, `crates/kesh-api/src/lib.rs`,
`crates/kesh-api/src/routes/{fiscal_years,bank_accounts}.rs`, `crates/kesh-api/tests/bank_accounts_e2e.rs`,
`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, `frontend/src/lib/features/fiscal-years/fiscal-years.{api,types,helpers}.ts`,
`frontend/src/lib/features/bank-accounts/bank-accounts.api.ts`, `frontend/src/lib/features/homepage/homepage.ts`,
`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`, les tests listés au compte rendu,
`frontend/tests/e2e/fiscal-years.spec.ts`, `docs/manual/fr/user-manual.{tex,pdf}`, `docs/api-external.md`,
`CHANGELOG.md` — 38 fichiers hors `_bmad-output` (`git diff --stat fcd2ae75 7c643eba -- . ':(exclude)_bmad-output'`).

## Change Log

- 2026-10-14 — création (Opus 5.5, autonomie) ; inventaire de l'existant, prémisses de #232 corrigées, découpage
  29-1a / 29-1b, 19 AC, T-a1 à T-a13, T-b1 à T-b9, M-a1 à M-a10, M-b1 à M-b9, choix C-29-1-1 à 15.
- 2026-10-14 — **validation P1** (Sonnet 5.5, deux lentilles, contexte frais, lecture seule ; rapports
  `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p1-{R,F}.md`) : **R 0 CRITICAL / 0 HIGH / 9 MEDIUM / 9 LOW**,
  **F 0 / 0 / 3 / 4** — 12 MEDIUM, 13 LOW, **25 findings, tous remédiés** dans la fiche (Opus 5.5), aucun refusé.
  Deux remèdes sont partiels et le disent : R6 (une seule définition de l'écart **côté serveur** ; le tableau de bord
  garde son calcul TypeScript, aligné par les mêmes cas de test — issue demandée pour le servir depuis le serveur) ;
  R11 (#667 et #398 reportés ; la priorité de #667 revient à l'orchestrateur). Ce qui change :
  - **29-1a** — préfixe i18n `fiscal-years-checks-` (R1) ; colonne d'actions rendue pour tous les rôles (R2) ;
    K3 : population par l'activité dans Y, table de précédence `NOT_COMPARABLE` > `NO_BALANCE_STATEMENT` >
    `MISSING` > `ENDS_BEFORE_YEAR_END` > `GAP`, liste `notCheckable` sans effet sur le statut (R5, C-29-1-16), borne
    SQL unique `Y.end_date` et clôture du compte comparée en Rust (R18), filtre de borne **dans** la table dérivée
    avant `ROW_NUMBER` et ordre des liaisons (F6), écart arrondi au centime par `statement_gap` (R6, C-29-1-17) ;
    extraction de la requête **et** du `match` (`balances_by_bank_account`, `balances_from_row`) ; AC3-bis neuf :
    `EARLIER_OPEN_SQL` / `find_earliest_earlier_open` (R13) et `NEXT_AFTER_SQL` / `find_next_after`, partagée par K7
    et la 29-1b (C-29-1-18) ; garde T-a2 sur les constantes SQL, `BALANCES_SQL` comprise (R12, F3) ; K6 dédupliqué
    et ordonné (R16) ; rappel du dialogue comptant `info` (R16, C-29-1-19) ; limites de K3 et K6 au manuel (F5) ;
    CHANGELOG `## [0.14.0] — Non publié`, section réutilisée par la 29-1b, et cas d'une v0.13.4 taguée après la
    fusion (R15) ; tests de page à adapter nommés (R7).
  - **29-1b** — AC11 / AC12 : « `reversal_blockers`, première lecture ordinaire » écrit **avec sa condition**
    (`authority = None`) (F4, R14) ; la sûreté repose sur la confirmation verrouillante de N+1, le verrou de N n'est
    qu'une conséquence, et la clôture de N+1 n'attend que si N est le premier exercice — sinon cycle connu (F2) ;
    `reverse_at_next_fiscal_year_start_in_tx` public (F1) ; T-b5 refondu en deux ordres sur la vraie fonction
    (C-29-1-20) ; `NEXT_FISCAL_YEAR_MISSING` porte le nom de l'exercice de l'origine ; compteurs du registre des
    routes nommés site par site (R15) ; motif à la place du bouton absent, états de chargement et d'échec (R9,
    C-29-1-22) ; préfixe `journal-entries-reverse-next-year-` ; manuel l. 764, brochure l. 418 et README l. 47 (R10) ;
    fixtures de l'E2E nommées — préréglage `with-company-no-fy`, relevé CAMT synthétique au dernier jour de N, solde
    d'ouverture nul, K3 en `GAP` puis `ok` (R3) ; répétition à blanc exécutable, `.keshbackup` traité comme un secret
    (R8, F7, C-29-1-21) ; fichiers de tests exacts de T-b2 (R17).
  - **Mutations** (R4, F1) : M-a9 **déclarée équivalente** (justification corrigée : non-chevauchement des
    exercices) ; ajoutées M-a11 à M-a18 (K7 `DESC`, bornes hautes de K2 / K4 / K6, précédence et activité de K3,
    arrondi, K1 `DESC`) et M-b10 à M-b12 (`NEXT_FISCAL_YEAR_MISSING`, étape (1) sans verrou de l'exercice — tuée par
    T-b5 ordre (ii) —, motif pendant le chargement) ; M-b3 tuée aussi par T-b5 ordre (ii).
  - **Décomptes, recomptés depuis la fiche après remédiation** : 20 AC (19 + AC3-bis), T-a1 à T-a15 (15), T-b1 à
    T-b9 (9), M-a1 à M-a18 (18, dont une équivalente), M-b1 à M-b12 (12), choix C-29-1-1 à 22 (22, au registre de
    la story ; C-29-1-2, 4 et 11 annotés, non réécrits).
  - **Propagation** : valeurs grepées sur le worktree après remédiation — `balances_rows`, `fiscal-year-checks-`
    (restent les seuls `data-testid`, voulus), « première lecture ordinaire » (reste au registre, sous annotation),
    « demande d'issue » (idem), « s'arrête après l'étape », l. 1879 (corrigée en 1868).
  - Validation P2 à lancer : `29-1-validate-prompt-p2.md` (Opus, lentilles R et F).
- 2026-10-14 — **validation P2** (Opus 5.5, deux lentilles, contexte frais, lecture seule ; rapports
  `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p2-{R,F}.md`) : **R 0 CRITICAL / 1 HIGH / 7 MEDIUM / 8 LOW**,
  **F 0 / 0 / 5 / 9** — 1 HIGH, 12 MEDIUM, 17 LOW, **30 findings**. Remédiation par Opus 5.5 (session distincte des
  lentilles), sur les décisions de l'orchestrateur, consignées comme telles au registre (C-29-1-23 à 26 ; C-29-1-27 à
  29 sont des choix de la remédiation).
  - ⚠️ **Signal D5 levé, déclaré au Project Lead** (§ *Règle de splitting préventif*, amendement D5) : la sévérité
    **monte** (P1 : 0 HIGH / 12 MEDIUM → P2 : 1 HIGH / 12 MEDIUM) **et** elle **recycle** — 13 des 16 findings de la
    lentille R et les 5 MEDIUM de la lentille F naissent de la remédiation P1, non de la conception d'origine. Le foyer
    est la répétition à blanc (AC19) : chaque correction de sa marche a produit le défaut suivant (P1 R8, F5, F7 ; P2
    R1 HIGH — sur Synology, « une copie de `docker-compose.yml` » est le compose de production et l'import pouvait
    remplacer la base de production —, R2, F5). Réponse de l'orchestrateur : **retirer** l'AC19 plutôt que la
    réécrire une troisième fois — elle devient la **Story 29-7** (doc), à écrire en l'exécutant réellement sur la
    station avec une sauvegarde synthétique ; clé `29-7-repetition-a-blanc: backlog` au registre de sprint, dont le
    commentaire cite R1, R2 et F5 de P2 comme cahier des charges (C-29-1-23). Hors de l'AC19, deux défauts
    **recyclent** aussi un finding de P1 sous une autre forme : la garde T-a2 (P1 R12 / F3 → P2 R4 = F1, une
    énumération de formes que la remédiation avait seulement déplacée) et la mutation M-b3 (P1 F2 / R14 → P2 R5 = F3,
    un ordre de test qui ne distinguait pas le mutant). Les deux sont traités par **changement de méthode**, non par
    une forme de plus : inventaire des sites non résolus (C-29-1-26), prescription qui rend la mutation décidable
    (C-29-1-27). Les autres findings sont distincts. Dispersion au grain fin écrite au décompte des modules (R8).
  - **Retiré avec l'AC19, sans objet ici, versé au cahier des charges de la 29-7** : R1 (HIGH), R2, F5, F L9, et la
    part de R15 qui visait G19 (URL de compose). Rien de la marche n'est réécrit : l'AC19 est un renvoi ; les tâches
    (Tb7), le tableau des modules, le périmètre et l'en-tête ne nomment plus le manuel d'administration ; les
    livrables `api-external.md` et CHANGELOG de la 29-1b passent à l'AC17 (C-29-1-28).
  - **29-1a** — K3 lit aussi le **grand livre** : un compte est examiné s'il a, dans Y, une transaction bancaire, un
    relevé à solde **ou** une ligne d'écriture sur son compte lié (R3 = F2, C-29-1-24), cas de T-a5, mutation M-a20,
    phrase du manuel écrite telle quelle (AC8), avec le relevé qui doit se terminer au dernier jour de l'exercice (F
    L8). Le **tableau de bord lit l'écart du serveur** (axe de l'orchestrateur, R8, C-29-1-25) : champ `statement_gap`
    de `BankAccountWithBalance`, `statementGap` lu par `bank-accounts.api.ts` et `homepage.ts`, `toCentime` supprimé,
    cas d'arrondi en Rust (T-a14), test neuf T-a16, mutation M-a21 ; une seule définition, la demande d'issue tombe.
    **T-a2 inversée** (R4 = F1, C-29-1-26) : tout site d'appel de requête a pour premier argument une constante de
    `ALL_SQL` (turbofish équilibré, appel coupé par rustfmt ; `format!` / `concat!` en argument y sont des non
    résolus), aucun mot de verrou dans le source décommenté, liste close des fonctions externes, cas de contrôle du détecteur, mutation M-a19 ; mesure de
    `FOR UPDATE` sous `READ ONLY` en T0 (F L3). Y lu dans l'instantané par une constante de `ALL_SQL`, 404 décidé
    dedans (F L6). Fixture de M-a4 par SQL direct avec activité dans Y, et assertion de montage (R6). `BalancesRow`
    `pub(crate)` et `is_shared` (R11). `earlier_open_locking_sql()`, source unique de `close` et de T-a15, nom
    `FIND_EARLIER_OPEN_SQL` propagé aux trois commentaires qui le citent (R12). Appelant réel de l'extraction côté
    route : `bank_accounts_e2e.rs` (R10). M-a9 : les quatre chemins de création nommés (R14). Gardes G4-bis, G7, G9 et
    `audit_label_registry.rs` nommées aux tâches (R15).
  - **29-1b** — **M-b3** (R5 = F3, C-29-1-27) : l'AC11 prescrit que le statut de N+1 n'est jugé **que** par la
    confirmation verrouillante ; M-b3 est tuée par T-b3 seul ; T-b5 (ii) ne la revendique plus ; la variante « statut
    lu sur la lecture ordinaire » devient M-b13, **déclarée équivalente** (état inatteignable sous `close` actuel) ;
    l'AC12 et le doc-comment disent que la confirmation est une défense en profondeur que la course n'atteint pas.
    Comptes de l'E2E créés par l'API à l'étape 0 — `1300 Actifs transitoires`, `1020 Banque` lié, ni 1100 (créance,
    refusé) ni 1000 —, réserve sur les comptes bancaires du préréglage retirée (R7 = F4, C-29-1-29). Compteur
    `rejouees, 24` → 25 ajouté aux sites qui bougent (R9). Paragraphe « Des boutons offerts » (l. 766) et table
    d'erreurs de `api-external.md` § 10 à l'AC17 / AC14 (R16). `LOCK_IN_COMPANY_SQL` l. 162-163 (R13 = F L1). « N tenu
    deux fois » réécrit au conditionnel, sans appui (F L2). Cycle de `create` d'un exercice postérieur ajouté à la
    liste de l'AC12 (F L4). Angle mort de l'intercalé précisé : chemin pool contre `_in_tx` (F L5). Constantes privées
    recopiées par T-b5 (ii), couplage déclaré (F L7). Replis Rust G9 et action d'audit littérale (R15).
  - **Refusés** : aucun.
  - **Décomptes, recomptés depuis la fiche après remédiation** : 19 AC vivants (AC1 à AC18 et AC3-bis ; AC19 retirée,
    son titre gardé en renvoi), T-a1 à T-a16 (16), T-b1 à T-b9 (9), M-a1 à M-a21 (21, dont une équivalente : M-a9),
    M-b1 à M-b13 (13, dont une équivalente : M-b13), choix C-29-1-1 à 29 (29 ; C-29-1-1, 13, 17 et 21 annotés, non
    réécrits). Modules : cinq par sous-story au grain de la crate (29-1b : `docs/manual` par le seul manuel
    utilisateur et la brochure).
  - **Propagation** : valeurs grepées sur le worktree après remédiation — `AC19`, « d'administration »,
    `admin-manual`, « issue demandée », « sixième module », « garde son calcul », `150-151`, `query*("`, « très
    probablement », « C-29-1-1 à 22 », `(AC17, AC19)` : ne restent que les renvois voulus et les entrées datées de la
    validation P1, non réécrites ; au fichier d'epic, la phrase « la répétition à blanc appartient à la dernière
    sous-story » est annotée (29-7).
  - Validation P3 à lancer : `29-1-validate-prompt-p3.md` (Sonnet, lentilles R et F).
- 2026-10-14 — **validation P3** (Sonnet 5.5, deux lentilles, contexte frais, lecture seule ; rapports
  `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p3-{R,F}.md`) : **R 0 CRITICAL / 0 HIGH / 2 MEDIUM / 12 LOW**,
  **F 0 / 0 / 1 / 2** — 3 MEDIUM, 14 LOW, **17 findings** (R1 et F1 désignent le même défaut). Remédiation par
  Opus 5.5 (session distincte des lentilles), sur les décisions de l'orchestrateur, consignées comme telles.
  - ⚠️ **Signal D5 au troisième tour, sur deux formes** (§ *Règle de splitting préventif*, amendement D5 — déclaré au
    Project Lead) : la mutation de la confirmation (P1 F2 / R14 → P2 R5 = F3 → P3 R1 = F1 : l'équivalence de M-b13 ne
    regardait qu'un sens de divergence, la réouverture de N+1 produit l'autre) et la garde T-a2 (P1 R12 / F3 → P2 R4 =
    F1 → P3 R2 : le volet (c) laissait passer les noms importés par `use`, et ni (c) ni (d) n'avaient de cas de
    contrôle). **Décision de l'orchestrateur : « une forme qui produit un défaut à chaque correction se retire. »**
    Les autres findings sont distincts et ne recyclent rien.
  - **Retrait 1 — M-b13** (R1 = F1, C-29-1-27 annoté) : la ligne sort de la table des mutations, **rien ne la
    remplace** ; la fiche ne porte plus aucune affirmation d'équivalence ni d'inatteignabilité à son sujet (le
    paragraphe « Ce que la confirmation garde n'est pas atteignable aujourd'hui » de l'AC12 est retiré, ainsi que la
    « défense en profondeur non atteignable » de Tb2). M-b3 reste, tuée par T-b3 ; l'AC11 garde sa prescription ;
    l'AC12 dit seulement que la réouverture de N+1 peut changer l'état lu entre la vue et la confirmation, et que
    c'est la confirmation qui décide. Les corrections que proposaient R1 et F1 (ordre (iii) de T-b5, nouveau test)
    ne sont **pas** reprises : ce serait remplacer la forme retirée par une autre.
  - **Retrait 2 — volets (c) et (d) de T-a2** (R2, C-29-1-26 annoté) : la garde est réduite à (a) — tout appel de
    requête a pour premier argument une constante de `ALL_SQL` — et (b) — aucun mot de verrou dans le source
    décommenté. Les fonctions de dépôt externes appelées par `closing_checks` sont un **angle mort assumé**, écrit à
    l'AC2 et à T-a2 : nommées dans le doc-comment du module (Ta2), vérifiées en revue de code. La correction que
    proposait R2 (développer les `use`, cas de contrôle de (c) et (d), mutation neuve) n'est **pas** reprise.
    Conséquence mécanique : `BALANCES_SQL`, `EARLIER_OPEN_SQL` et `NEXT_AFTER_SQL` redeviennent privées (leur
    `pub(crate)` ne servait que le volet (d)).
  - **LOW traités** — R3 : le cas « l'écart seulement s'il est non nul » de `HomepageCards.test.ts` (l. 166-188)
    nommé, `statementGap: '-200.00'` au premier rendu (AC3). R4 : les deux clés `error-next-fiscal-year-*` ajoutées
    à la `TABLE` fermée de G9, avec `NOM` (AC14, Tb3, Tb5). R5 : C-29-1-16 (renvoi à C-29-1-24) et C-29-1-20 (renvoi à
    C-29-1-27, M-b11 réécrite) annotés. R6 : assertion **neuve** de T-b2 — le JSON d'audit du chemin `Today` égale
    exactement `{"reversalJournalEntryId": …}` — qui fonde « octet pour octet » (AC10, AC11, M-b5). R7 : Tb4 réécrite
    (le mock de `fiscal-years.api` existe, son `vi.fn()` rend `undefined` : comportement par défaut et trois
    comportements ; `startEdit` garde sa lecture, C-29-1-30). R8 : source de `NO_BALANCE_STATEMENT` — un quatrième
    `EXISTS`, sans borne de date, de la requête de population ; cas de T-a5 (relevés tous postérieurs à Y →
    `MISSING`). R9 : F L9 et G19 ajoutés au commentaire de la clé `29-7-repetition-a-blanc`. R10 : grep de l'AC17
    étendu à `crates/` (et à « celle du jour »), les deux commentaires `journal_entries.rs` l. 398-402 et
    `companies.rs` l. 909-912 nommés (AC17, Tb2). R11 : annotation en tête de la clé `29-1b` du registre de sprint ;
    renvoi de la liste des modules au ⚠️ qui la complète. R12 : la fixture du compte archivé a un relevé arrêté
    **avant** `Y.end_date`, avec un cas de contrôle non archivé qui le liste (T-a5, M-a4). R13 : état de chargement du
    panneau (AC5, AC7), cas de T-a12, mutation **M-a22**. R14 : la route décrite au § 7 d'`api-external.md` (date,
    absence de corps, `201`, refus), AC17 et Tb7. F2 : M-b11 retire le `JOIN fiscal_years fy` de la requête
    verrouillante de l'étape (1). F3 : « c'est la seule qu'on invoque » remplacé — la confirmation donne le refus
    nommé, le re-verrou de `create_in_tx_inner` (`journal_entries.rs` l. 333-345) refuse de toute façon l'écriture
    dans un exercice clos.
  - **Refusés** : aucun. Non repris par décision de l'orchestrateur : les remèdes proposés par R1, F1 et R2 (voir les
    deux retraits).
  - **Décomptes, recomptés depuis la fiche après remédiation** : 19 AC vivants (AC1 à AC18 et AC3-bis), T-a1 à T-a16
    (16), T-b1 à T-b9 (9), M-a1 à M-a22 (22, dont une équivalente : M-a9), M-b1 à M-b12 (12, aucune équivalente),
    choix C-29-1-1 à 30 (30 ; C-29-1-16, 20, 26 et 27 annotés en P3, non réécrits).
  - **Propagation** : symptômes grepés sur le worktree après remédiation — `M-b13`, « équivalente », « atteignable »,
    « défense en profondeur », « hors d'atteinte », « liste close », `pub(crate)`, « quatre volets », « seule qu'on
    invoque », « ne mocke pas », « C-29-1-1 à 29 », « M-a1 à M-a21 », « M-b1 à M-b13 » : ne restent que les entrées
    datées des validations P1 et P2 (non réécrites), les annotations de retrait, M-a9 (équivalence propre, hors
    retrait) et les `pub(crate)` de `balances_by_bank_account`, `BalancesRow` et `is_shared`, appelés depuis
    `closing_checks`.
  - Passe ciblée P4 à lancer : `29-1-validate-prompt-p4-ciblee.md` (Haiku, une lentille, sur le seul diff de cette
    remédiation).
  - *Annotation datée 2026-10-14 (validation P4, P4-3 ; l'entrée P3 ci-dessus n'est pas réécrite)* : le décompte
    « 17 findings » se lit **17 entrées, 16 défauts distincts (R1 = F1)** ; les « 3 MEDIUM » se lisent 2 MEDIUM
    distincts et un doublon.
- 2026-10-14 — **validation P4 (passe ciblée)** (Haiku 4.5, une lentille « chasseur de régression », sur le seul diff
  de la remédiation P3 ; rapport `/home/gcorbaz/devel/kesh-gate-logs/29-1-validate-p4-ciblee.md`) : **0 CRITICAL /
  0 HIGH / 1 MEDIUM / 3 LOW**. Remédiation par Opus 5.5, **documents seulement** (fiche, registre, sprint-status).
  - **Trend complet** : P1 R 0/0/9/9, F 0/0/3/4 → P2 R 0/1/7/8, F 0/0/5/9 → P3 R 0/0/2/12, F 0/0/1/2 → P4 ciblée
    0/0/1/3.
  - **P4-1 (MEDIUM)** : le rappel du dialogue (AC6) affiche, pendant le chargement, l'état d'attente — jamais
    « Aucun point à vérifier » ni « N point(s) » ; T-a12 l'asserte ; M-a22 s'étend au dialogue. **P4-2** :
    `period_lock_e2e.rs` l. 367-369 ajouté à la liste de l'AC17 (et à Tb2 par renvoi). **P4-3** : voir l'annotation
    datée sous l'entrée P3. **P4-4** : décision de l'orchestrateur, consignée comme telle (C-29-1-31) — angle mort
    écrit à l'AC12, sans mutation ni test.
  - **Propagation** : `grep -nF "Aucun point à vérifier"` sur la fiche (AC6, AC-parcours, T-a12, M-a22) ; ne restent
    que des occurrences cohérentes.
  - **Validation close : la remédiation de la passe ciblée ne touche aucune ligne de production** (fiche, registre et
    `sprint-status.yaml` seulement). Dev à lancer.
- 2026-10-14 — **développement de la 29-1a** (Opus 5.5, `bmad-dev-story`, autonomie) : commits `ae6d9419`,
  `bf5f8238`, `7c643eba` (dernier commit de code), puis comptes rendus. AC1 à AC9 et AC3-bis livrés ; tests rouges
  constatés sur talon (18 rouges, 4 verts par nature) ; 25 tests Rust, 19 Vitest et 3 E2E neufs (recomptés aux deux
  bornes) ; gate complet (3629 / 6 ignorés ; Vitest 1629) et E2E complet (283 passés, les 7 échecs attendus de la
  KF-029) au dernier commit de code ; 21 mutations exécutées, 21 tuées, M-a9 équivalente ; choix C-29-1-32 à 42 ;
  six écarts déclarés au Dev Agent Record. Revue de code P1 à lancer : `29-1a-review-prompt-p1.md` (Sonnet, B, E, A).
- 2026-10-14 — **revue de code P1 remédiée** (Opus 5.5 ; rapports `kesh-gate-logs/29-1a-review-p1-{B,E,A}.md`, trois
  lentilles Sonnet) : B 0/0/0/3, E 0/0/4/6, A 0/0/1/8. Les MEDIUM convergeaient — le code est juste, des tests
  manquaient — : **tests et mutations seulement**, **aucune ligne exécutable de production changée** (doc-comments
  seuls). 3 tests Rust neufs et 2 étendus, 4 Vitest neufs ; 20 mutations rejouées, 19 tuées, M-a33 équivalente
  (C-29-1-44). LOW en texte : manuel (PDF régénéré), note ¹⁰, doc du module. La 29-1a part en **v0.13.4** (décision de
  l'orchestrateur, C-29-1-43). Rebase sur `c36f39bc` (29-2) d'abord. Gate complet 3639 / 6 ignorés, Vitest 1633, E2E
  283 passés et les 7 échecs de la KF-029, au dernier commit de code `8b5e1a3f` (rejoué sur `1c15363e`). Choix
  C-29-1-43 à 47. Passe ciblée P2 à lancer : `29-1a-review-prompt-p2-ciblee.md` (Haiku).
- 2026-10-14 — **revue P2 (passe ciblée)** (Haiku 4.5, une lentille « chasseur de régression », sur le seul diff
  `34750f1f..1c15363e` ; rapport `kesh-gate-logs/29-1a-review-p2-ciblee.md`) : **0 CRITICAL / 0 HIGH / 2 MEDIUM /
  2 LOW**. Remédiation par Sonnet 5.5, commit de code `7b1fd8b8`.
  - **Trend complet** : P1 B 0/0/0/3, E 0/0/4/6, A 0/0/1/8 → P2 ciblée 0/0/2/2.
  - **F1 (MEDIUM)** : mécanisme de l'écart K3 décrit exactement (chaque solde arrondi avant la soustraction), doc du
    module et manuel, PDF régénéré ; deux exemples assertés au test de `statement_gap`. **F2 (MEDIUM)** : écriture de Y
    du dernier jour dans la fixture de K6, M-a42 tuée. **F3 (LOW)** : section `[0.14.0]` vide retirée du CHANGELOG.
    **F4 (LOW)** : commentaire Svelte aligné.
  - **Propagation** : symptômes grepés sur le worktree (`demi-centime`, `Nouvel exercice`, `0.14.0`) : ne restent que
    des occurrences sans rapport avec K3 (`invoice_settlements`, `company_invoice_settings`, `pdf.rs`,
    `admin-manual.tex`) ou propres à la 29-1b.
  - Gates au dernier commit de code : 3639 / 6 ignorés ; Vitest 1633 ; E2E 282 passés, 8 échecs tous listés (sept
    KF-029 + `sidebar-navigation.spec.ts:75`, qui passe rejoué seul).
  - Registre de la story versé dans `epic-29-choix-autonomes.md` (fichier propre conservé).
  - **Revue close : la remédiation de la passe ciblée ne touche aucune ligne exécutable de production** (un
    doc-comment, un test de module, un commentaire Svelte, un test d'intégration, le manuel et le CHANGELOG).
