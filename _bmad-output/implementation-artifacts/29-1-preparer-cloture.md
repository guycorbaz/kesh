# Story 29.1 : Préparer la clôture d'un exercice — contrôles préalables, puis transitoires

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur
`16eeed21` (kickoff de l'Epic 29), worktree `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1`. Validation P1 à
lancer (`29-1-validate-prompt-p1.md`, Sonnet, lentilles R et F).

⛔ **Deux stories, une fiche, deux PR successives** (C-29-1-1, précédents C-26-2-15 et C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`29-1a-controles-prealables`** | lecture `closing_checks` (kesh-db), route `GET /api/v1/fiscal-years/{id}/closing-checks`, panneau *Préparer la clôture* et rappel dans le dialogue de clôture, i18n ×4, manuel utilisateur (PDF), `api-external.md`, CHANGELOG | AC1 à AC9 | **`refs #232`** — ⛔ jamais `closes` |
| **`29-1b-contre-passation-transitoires`** | contre-passation datée du premier jour de l'exercice suivant (kesh-db, route), bouton sur la fiche d'une écriture, i18n ×4, manuels utilisateur **et** d'administration (répétition à blanc), E2E de parcours complet, `api-external.md`, CHANGELOG | AC10 à AC19 | **`closes #232`** |

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
| Contrôles préalables : **TVA réconciliée** | **Écarté** : report assumé des P1 de la TVA (CLAUDE.md § *Priorités des défauts*) ; le seul utilisateur réel n'est pas assujetti. Le rapport TVA porte déjà un statut d'écart (`kesh-report/src/vat_report.rs:190`). | demande d'issue (rapport final) |
| Contrôles préalables : **comptes d'attente soldés** | **Prémisse sans objet au code** : Kesh n'a **ni rôle, ni drapeau, ni compte d'attente** (aucun dans `AccountRole`, `kesh-core/src/chart_of_accounts/mod.rs:51-76`, ni dans les trois plans `kesh-core/assets/charts/*.json`, ni dans la réconciliation). Rien à contrôler sans inventer un rôle (migration **breaking** : un binaire antérieur ne décode pas un rôle inconnu). | écarté, C-29-1-4 |
| Génération des **écritures de clôture** (virement du résultat au bilan) | **Écarté par conception** (14-1) : le modèle virtuel calcule ce virement ; une écriture qui le doublerait fausserait le bilan, et 2979 est non imputable. | C-29-1-2 |
| **Report à nouveau automatique** | **Existe** (calculé). | — |
| **Aide aux transitoires** avec contre-passation automatique à l'ouverture de N+1 | Retenu : la contre-passation existe mais se date **du jour** ; il manque la contre-passation **au premier jour de l'exercice suivant**. | 29-1b |
| Intégration des **amortissements** (#222) | **Hors périmètre** (consigne de l'orchestrateur ; #222 / #408). | — |
| (candidat de l'orchestrateur) écriture d'**affectation du résultat** | **Écarté** (C-29-1-2) : sans objet pour une personne physique ou un indépendant (le résultat s'accumule aux fonds propres, ce que le report calculé montre déjà) ; pour une société, l'affectation votée par l'assemblée est une écriture **manuelle** déjà possible (débit du compte physique `RetainedEarnings`, crédit dividendes ou réserve) dont seule la **présentation** au bilan est perfectible — demande d'issue. | demande d'issue (rapport final) |

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
3. les manuels (utilisateur ; administration pour la répétition à blanc), l'i18n quatre locales, `api-external.md`,
   le CHANGELOG ; un E2E de parcours complet.

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
| Corriger la capture `% TODO capture` du manuel l. 734 | Hors périmètre ; les captures de la story vont hors dépôt (convention 28-4b). |

**Décompte des modules** (§ *Règle de splitting préventif*) — ensemble : `kesh-db` (`closing_checks` neuf,
`bank_accounts`, `journal_entries`), `kesh-api` (`routes/fiscal_years`, `routes/journal_entries`, `lib.rs`,
`errors.rs`), `kesh-i18n`, frontend (`features/fiscal-years`, `routes/settings/fiscal-years`,
`routes/journal-entries/[id]`, `features/journal-entries`), `docs/manual` : **plus de cinq** → découpage (C-29-1-1).
**29-1a** : `kesh-db`, `kesh-api`, `kesh-i18n`, frontend fiscal-years, `docs/manual` = **cinq**, la borne. **29-1b** :
`kesh-db`, `kesh-api`, `kesh-i18n`, frontend journal-entries, `docs/manual` = **cinq**, la borne. Pas de 29-1c :
l'affectation du résultat est écartée, la répétition à blanc appartient à la 29-1b (dernière sous-story).

## Critères d'acceptation

### Partie a — `29-1a-controles-prealables`

**AC1 — Les contrôles, l'ensemble clos.** Une fonction `kesh_db::repositories::closing_checks::for_fiscal_year(conn,
company_id, fiscal_year_id) -> Result<ClosingChecks, DbError>` rend, pour l'exercice `Y` (`start_date`, `end_date`),
**exactement** les sept contrôles suivants, dans cet ordre, chacun avec un `code` stable (SCREAMING_SNAKE_CASE), une
`severity` et un `status` `ok` | `attention` :

| Code | Sévérité | `attention` quand… | Données rendues |
|---|---|---|---|
| **K1 `EARLIER_FISCAL_YEAR_OPEN`** | `blocking` | un exercice de la société dont `start_date < Y.start_date` est `Open` | le **plus ancien** (`id`, `name`) — même prédicat que `FIND_EARLIER_OPEN_SQL` (`fiscal_years.rs`), **sans** `FOR UPDATE` |
| **K2 `PENDING_BANK_TRANSACTIONS`** | `warning` | une `bank_transactions` de la société au statut `pending` a `booking_date BETWEEN Y.start_date AND Y.end_date` | par compte bancaire : `bankAccountId`, nom, `count`, dont `rejectedCount` (`auto_match_rejected_at IS NOT NULL`) ; total |
| **K3 `BANK_STATEMENT_AT_YEAR_END`** | `warning` | pour un compte bancaire **à vérifier** (ci-dessous) : le relevé de référence — le relevé portant un solde dont `period_to` est le plus tardif **sans dépasser la borne** (`Y.end_date`, ou `closed_on` si le compte est clos pendant Y) — est absent ou antérieur à `Y.start_date` (`MISSING`), ou s'arrête avant la borne (`ENDS_BEFORE_YEAR_END`), ou l'écart `solde comptable à period_to − closing_balance` est non nul (`GAP`) | par compte : `statementDate`, `statementClosingBalance`, `ledgerBalanceAtStatement`, `gap`, et un motif `MISSING` / `ENDS_BEFORE_YEAR_END` / `GAP` / `NOT_COMPARABLE` (compte sans compte lié, ou compte lié partagé — `sharing > 1`, comme le tableau de bord) |
| **K4 `DRAFT_INVOICES`** | `info` | une facture client `status = 'draft'` a `date BETWEEN Y.start_date AND Y.end_date` | `count`, et au plus 20 `{id, label}` (numéro absent d'un brouillon : le libellé est celui de la liste des factures) |
| **K5 `IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE`** | `info` | une `imported_supplier_invoices` de la société est `to_complete` | `count` — ⚠️ **non borné à l'exercice** : la table n'a pas de date métier, seulement `created_at` (écrit au manuel) |
| **K6 `INCONSISTENT_ENTRIES`** | `anomaly` | une écriture **de Y** a moins de deux lignes, ou `SUM(debit) <> SUM(credit)` ; **ou** une écriture de Y a `entry_date` hors `[Y.start_date, Y.end_date]` ; **ou** une écriture d'un **autre** exercice a `entry_date` dans `[Y.start_date, Y.end_date]` | `count` par motif (`UNBALANCED`, `TOO_FEW_LINES`, `DATE_OUTSIDE_ITS_YEAR`, `OTHER_YEAR_DATED_INSIDE`) et au plus 20 `{id, entryNumber, fiscalYearName}` |
| **K7 `NEXT_FISCAL_YEAR`** | `info` | aucun exercice de la société n'a `start_date > Y.end_date` | `{id, name, startDate, status}` du suivant s'il existe |

**Comptes à vérifier pour K3** : tout compte bancaire de la société, **sauf** archivé (il n'a aucune transaction,
`archive_for_company`) et **sauf** clos avant le début de Y (`closed_on < Y.start_date`). Un compte clos pendant Y
est vérifié (son dernier relevé doit aller jusqu'à sa clôture : motif `ENDS_BEFORE_YEAR_END` mesuré contre
`MIN(Y.end_date, closed_on)`).

`status = ok` ⇔ la condition d'`attention` est fausse ; le compteur ou la liste sont alors vides. K1 à K7 sont
**l'ensemble clos** : un contrôle neuf est un changement de cette fiche, et le test T-a1 compare la liste des codes
rendus à cette table (ordre compris).

**AC2 — Une seule vue, aucune écriture, aucun verrou.** `for_fiscal_year` s'exécute sur la connexion d'un
`kesh_db::snapshot::ReadSnapshot` (Story 28-3, `snapshot.rs:61`), ouverte par la route et close par `finish()` :
toutes les lectures voient la base au même instant ; l'instantané est `READ ONLY` (toute écriture refusée par le
serveur) ; **aucune** lecture verrouillante (`FOR UPDATE`, `LOCK IN SHARE MODE`). Garde statique T-a2 : le texte
source de `closing_checks.rs` hors `#[cfg(test)]` ne contient ni `FOR UPDATE`, ni `LOCK IN SHARE MODE`, ni
`INSERT`, `UPDATE `, `DELETE`, `REPLACE` en tête d'instruction SQL.

**AC3 — K3 réutilise la requête du tableau de bord, bornée.** La requête de `list_by_company_with_balances`
(`bank_accounts.rs:1392`), seule à savoir corriger l'écart des décalages date de valeur / comptabilisation, est
**extraite** en une fonction privée du module (`balances_rows(executor, company_id, statement_bound:
Option<NaiveDate>)`) dont le texte SQL est **unique** : le relevé de référence devient « portant un solde, et
`period_to <= ?` si une borne est donnée ». Le tableau de bord l'appelle avec `None` — **résultat inchangé**, prouvé
par les tests existants de `list_by_company_with_balances` restés verts **sans modification** — et `closing_checks`
avec `Some(Y.end_date)` (ou `MIN(Y.end_date, closed_on)`). ⛔ Pas de copie de la requête (règle DRY du dépôt). La
fonction accepte un exécuteur générique (`E: Executor<'_, Database = MySql>`) pour tourner dans l'instantané.

**AC4 — La route.** `GET /api/v1/fiscal-years/{id}/closing-checks`, montée dans `authenticated_routes` à côté de
`GET /api/v1/fiscal-years/{id}` (`lib.rs:938`) : **tout rôle**, Consultation et clés d'API comprises (lecture, comme
les rapports, `lib.rs:973`). 404 si l'exercice n'est pas de la société (anti-énumération, même ordre que
`get_fiscal_year`). 200 avec :

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
*Préparer la clôture* (`data-testid="fiscal-year-checks-{id}"`), visible de **tous les rôles**. Il ouvre un panneau
(`Dialog`, `data-testid="fiscal-year-checks-dialog"`) qui liste les sept contrôles dans l'ordre de l'AC1 : une ligne
par contrôle, son libellé, son état (*Rien à signaler* / le compteur et le détail), une icône et une couleur par
sévérité **lisibles en clair et en sombre** (jetons de thème existants, C-28-1-7), et un **lien** vers l'écran où l'on
agit : *Rapprochement* (K2), *Comptes bancaires* (K3), *Factures* filtrées sur les brouillons (K4), *Factures
fournisseur importées* (K5), la fiche de chaque écriture listée (K6), *Nouvel exercice* (K7, Admin/Comptable
seulement). Erreur de chargement : le panneau le dit (« indisponible ») — jamais « rien à signaler » par défaut.
Montants en `big.js`, formatage suisse du dépôt.

**AC6 — Le rappel dans le dialogue de clôture.** Le dialogue existant (`+page.svelte:575-612`) charge les mêmes
contrôles à son ouverture et affiche, **au-dessus** du texte de confirmation actuel, soit « Aucun point à vérifier »,
soit « N point(s) à vérifier » avec la liste courte des contrôles en `attention` et un lien vers le panneau. Le bouton
*Clôturer* **reste actif** quel que soit le résultat (rien ne bloque, C-29-1-3) ; pendant le chargement il reste
actif aussi (on ne retarde pas un geste que le serveur accepte) ; un échec de chargement s'affiche sans bloquer.
K1 n'a pas de cas visible ici : le bouton *Clôturer* est déjà désactivé quand un antérieur est ouvert (`:408`).

**AC7 — Textes.** Clés i18n neuves préfixées `fiscal-year-checks-` (titre, bouton, un libellé et un texte d'aide par
contrôle, motifs de K3 et K6, états, erreur de chargement, rappel du dialogue), dans les **quatre** locales
(`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`), en français soigné et traduites réellement
(pas de recopie du français) ; `npm run lint-i18n-ownership` et `i18n-keys.test.ts` verts. Aucune clé d'erreur
serveur neuve (la route ne refuse rien de neuf).

**AC8 — Manuel utilisateur.** `docs/manual/fr/user-manual.tex`, § *Clôture d'un exercice* (l. 732) : nouvelle
sous-section *Préparer la clôture* (`\label{sec:preparer-cloture}`) **avant** la marche de clôture — à quoi sert le
panneau, qu'il ne bloque rien, les sept contrôles un par un (ce qu'ils signalent, ce qu'on fait), les **limites** :
K5 non borné à l'exercice ; K3 ne compare que les relevés CAMT (un CSV ne porte pas de solde, § *Import bancaire*) ;
K6 ne peut signaler que des données héritées ou restaurées (aucun chemin d'écriture actuel n'en produit) ; le panneau
lit à un instant, il ne suit pas les saisies concurrentes. La phrase l. 728 « Les comptes du bilan sont
automatiquement reportés depuis le bilan de clôture de l'exercice précédent **(si clôturé)** » est **fausse** au
code (le report est calculé que l'exercice précédent soit clos ou non, `balance_sheet.rs` l. 7-11) : corrigée dans
le même commit (défaut voisin **mécanique**, B3). Le résumé annuel l. 2992 gagne « contrôles préalables ». **PDF
régénéré** (`make fr` dans `docs/manual/`, sous le verrou de gate) et contrôlé aplati.

**AC9 — `api-external.md` et CHANGELOG.** `docs/api-external.md` documente la route (rôles, forme, codes, la
remarque sur K5). CHANGELOG : entrée `### Ajouté` sous une section non publiée, « Préparer la clôture d'un
exercice : sept contrôles en lecture seule… » (#232), sans annoncer la 29-1b.

### Partie b — `29-1b-contre-passation-transitoires`

**AC10 — La datation, paramètre interne de la contre-passation unique.** `reverse_in_tx_inner`
(`journal_entries.rs:2595`) reçoit un paramètre `dating: ReversalDating` (`enum ReversalDating { Today,
NextFiscalYearStart }`, documenté). **Toutes** les entrées existantes (`reverse`, `reverse_in_tx`,
`reverse_owned_in_tx`) passent `Today` — **comportement inchangé**, prouvé par les tests existants de la
contre-passation et des quatre annulations restés verts sans modification. ⛔ Aucune copie de la fonction : la
contre-passation reste « écrite une seule fois » (doc l. 2572).

**AC11 — Ce que `NextFiscalYearStart` fait, à la place de l'étape (4) seulement.**

- l'étape (1) lit en plus `fy.end_date` de l'exercice de l'origine (colonne ajoutée au même `SELECT … FOR UPDATE`,
  l. 2603-2607 — mêmes lignes verrouillées, aucun verrou ajouté) ;
- à l'étape (4), **au même endroit** que le calcul de `today` (après `reversal_blockers`, l. 2664, première lecture
  ordinaire de la transaction), une lecture ordinaire rend le **premier exercice de la société dont `start_date >
  end_date de l'origine`** (`ORDER BY start_date ASC LIMIT 1`) ; absent → `DbError::NextFiscalYearMissing` ;
- la date retenue est son `start_date` ; `find_open_covering_date(tx, company_id, start_date)` — lecture
  **verrouillante**, donc sur l'état validé le plus récent — la confirme : `None` (exercice clos depuis, ou supprimé
  par `onboarding::reset`) → `DbError::NextFiscalYearClosed { fiscal_year_id, fiscal_year_name }` avec l'exercice lu
  à l'étape précédente ;
- la suite est **celle de `Today`** : `create_in_tx_inner` avec `entry_date = start_date`, donc **tous** ses refus —
  `PeriodLocked` si `start_date <= books_locked_through`, `LaterFiscalYearClosed` si un exercice postérieur à N+1 est
  clos —, lettrage de l'étape (5), audit `journal_entry.reversed` qui porte en plus `"dating": "nextFiscalYearStart"`
  (le chemin `Today` garde son JSON actuel, octet pour octet) ;
- le libellé est celui de la contre-passation (`Contre-passation écriture n° X (Exercice N)` : l'exercice diffère
  toujours) ;
- origine dans un exercice **clos** : accepté (comme la contre-passation du jour) ; N+1 doit être ouvert.

**AC12 — Ordre des verrous et naissance de la vue : rien ne bouge (critère hérité de l'AC7 de la 28-4, B5).** Le
chemin `NextFiscalYearStart` prend **les mêmes verrous, dans le même ordre**, que `Today` : origine et son exercice
(1), lignes de l'origine (2), puis le parcours de `find_open_covering_date` (4), puis ceux de `create_in_tx_inner`.
La seule lecture ajoutée est **ordinaire** et placée **après** `reversal_blockers` : la vue est déjà née, elle ne
naît pas plus tôt. Les cycles connus (avec `close`, avec `create` d'un exercice antérieur, avec le lettrage —
doc l. 2574-2592) sont **de même nature** et se résolvent par le même rejeu. Le doc-comment de
`reverse_in_tx_inner` le dit. Garde : T-b4 (aucune lecture ordinaire neuve avant `reversal_blockers`, par lecture du
source) et T-b5 (concurrence, ci-dessous). **Conséquence utile, à écrire au doc-comment** : l'étape (1) tient
l'exercice N de l'origine en exclusif, et `fiscal_years::close(N+1)` doit verrouiller N en (b') ; la clôture de N+1
**attend** donc la contre-passation, qui ne peut pas écrire dans un N+1 clos depuis sa lecture. La réouverture de N+1
(N+1 clos ⇒ N clos, invariant I) ne prend pas N : elle peut changer l'état lu, ce que la confirmation verrouillante
couvre.

**AC13 — La route.** `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start`, **sans corps**, montée
dans `comptable_routes` à côté de `/reverse` (`lib.rs:383`) : Admin et Comptable ; Consultation 403 ; clés
`read-write` admises comme pour `/reverse`. Handler calqué sur `reverse_journal_entry` (`routes/journal_entries.rs:508`),
**rejoué** (`retry_on_deadlock("journal_entries::reverse_at_next_fiscal_year_start", …)`), réponse `201
JournalEntryResponse`. Inscrite au registre `audit_route_registry.rs` (statut d'audit tracé, rejeu `Rejouee`) ; le
compte des routes de l'inventaire et du registre **se recompte depuis la source** (doc du test : 118 / 121 à
`16eeed21`, à revérifier). `docs/api-external.md` l. 730 : la route s'ajoute à la liste des routes rejouées.
⛔ `/reverse` n'accepte **pas** de corps neuf (clients existants, C-29-1-10).

**AC14 — Les refus, traduits.** Deux codes neufs, 409 `CONFLICT` :
`NEXT_FISCAL_YEAR_MISSING` (« Aucun exercice ne suit l'exercice « N » : créez l'exercice suivant, puis
recommencez. ») et `NEXT_FISCAL_YEAR_CLOSED` (« L'exercice suivant, « N+1 », est clôturé : la contre-passation ne peut
pas y être datée. », `details: {fiscalYearId, fiscalYearName}`). Mapping au `match` de `errors.rs` (proche de
`FiscalYearInvalid`, l. 3988), clés `error-next-fiscal-year-missing` / `-closed` ×4 locales. Tous les autres refus
sont ceux de la contre-passation, sans changement (`EntryNotReversable`, `ReversalAccountsArchived`, `PeriodLocked`,
`LaterFiscalYearClosed`).

**AC15 — Le bouton sur la fiche d'une écriture.** `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte` :
à côté de *Contre-passer* (`data-testid="reverse-entry"`, l. 330-336), un second bouton *Contre-passer au début de
l'exercice suivant* (`data-testid="reverse-entry-next-year"`), sous **les mêmes conditions** (`entry.reversable`,
rôle Admin/Comptable) **et** quand la liste des exercices (`GET /api/v1/fiscal-years`, client existant
`fiscal-years.api.ts`) contient un exercice **ouvert** dont `startDate` est la plus petite `> endDate` de l'exercice
de l'écriture (`entry.fiscalYearId`). Confirmation qui **nomme la date et l'exercice** (« La contre-passation sera
datée du 01.01.2027, dans l'exercice « Exercice 2027 ». ») ; succès → navigation vers l'écriture créée, comme
*Contre-passer*. Le serveur reste l'autorité : un refus (AC14) s'affiche traduit, sans « [object Object] » (#520).
Les deux boutons disparaissent ensemble une fois l'écriture contre-passée (`reversable` faux).

**AC16 — Textes.** Clés neuves (bouton, confirmation avec `{ $date }` et `{ $name }`, libellé d'aide) dans les quatre
locales ; lints i18n verts.

**AC17 — Manuel utilisateur.** § *Corriger une écriture : la contre-passation* (l. 628) : nouvelle sous-section
*Les transitoires de fin d'exercice* — l'usage (charge payée d'avance, produit à recevoir, facture à recevoir : actifs
1300 et passifs 2300 transitoires des plans livrés), la marche (saisir l'écriture au dernier jour de N, puis
*Contre-passer au début de l'exercice suivant*), ce que Kesh exige (exercice suivant **créé et ouvert**, premier
jour hors période verrouillée), ce qui se fige : l'écriture et sa contre-passation ne se modifient ni ne se
suppriment plus (une écriture contre-passée ne se supprime pas, `journal_entries.rs:1866-1875`, ni ne se modifie, garde de
modification) — une erreur se corrige par une **écriture complémentaire** ; et l'angle mort : **Kesh ne repère pas
une transitoire que vous avez oublié de contre-passer** (C-29-1-14). La phrase du verrou de période « La
contre-passation est datée du jour » (l. 614-615) gagne l'exception. Le panneau *Préparer la clôture* (§ de la 29-1a)
renvoie à cette sous-section. **PDF régénéré**, contrôlé aplati.

**AC18 — E2E de parcours complet (porte la charge de la recette, fichier d'epic § *La recette*).** Spec neuve
`frontend/tests/e2e/fiscal-year-closing-journey.spec.ts`, sur une société de démonstration montée par l'API
(fixtures **synthétiques**) :

1. deux exercices ouverts N et N+1 ; une transaction bancaire importée non rapprochée datée de N ;
2. *Préparer la clôture* de N : K2 en `attention` (1), K7 `ok` (N+1 existe) ;
3. rapprocher la transaction ; rouvrir le panneau : K2 `ok` ;
4. saisir une écriture de régularisation au dernier jour de N (débit 1300, crédit charge) ; *Contre-passer au début
   de l'exercice suivant* : l'écriture créée est datée du premier jour de N+1, dans N+1, liée à l'origine ;
5. bilan de N au dernier jour : 1300 porte le montant ; bilan de N+1 au premier jour : 1300 est **nul** ;
6. clôturer N (le dialogue affiche « Aucun point à vérifier ») ; N est clos ;
7. un administrateur rouvre N avec un motif ; N est ouvert ; le panneau se relit.

Plus un cas de refus : sans exercice suivant, le bouton n'apparaît pas, et l'appel API direct rend
`409 NEXT_FISCAL_YEAR_MISSING`.

**AC19 — Répétition à blanc au manuel d'administration, `api-external.md`, CHANGELOG.**
`docs/manual/fr/admin-manual.tex`, après § *Test de restauration périodique* (l. 1564) : sous-section *Répéter la
clôture sur une copie* (`\label{sec:repetition-cloture}`) — exporter une sauvegarde `.keshbackup` de l'instance de
production (§ *Export/import d'installation*, l. 1817) ; démarrer une **instance séparée** sur une autre machine (ou
un autre projet docker-compose : autre port, autre volume de base, **jamais** la base de production) ; y importer la
sauvegarde ; y ouvrir *Préparer la clôture*, passer les transitoires, clôturer, vérifier bilan et compte de résultat ;
**ne rien rapporter** de la copie vers la production (les gestes se refont sur la production une fois validés) ;
supprimer la copie. Avertissements : les documents (justificatifs, PDF de factures) ne sont pas dans le
`.keshbackup` (#503) ; la version de la copie doit être **celle de la production** (une sauvegarde d'une version plus
récente ne s'importe pas dans une plus ancienne). **PDF régénéré.** `api-external.md` : la route de l'AC13. CHANGELOG :
`### Ajouté` (transitoires ; répétition à blanc), mention #232.

## Tasks / Subtasks

### 29-1a

- [ ] **Ta1** — Tests rouges d’abord : T-a1 à T-a13 (AC1-AC6).
- [ ] **Ta2** — `crates/kesh-db/src/repositories/closing_checks.rs` (neuf, `pub mod` dans `repositories/mod.rs`) :
      types `ClosingChecks`, `ClosingCheck`, enums `Severity`, `CheckStatus`, détails par code ; doc de module (les
      sept contrôles, l'instantané, ce que K5 et K6 ne disent pas) (AC1, AC2).
- [ ] **Ta3** — Extraction `balances_rows` dans `bank_accounts.rs`, SQL unique avec borne (AC3).
- [ ] **Ta4** — Route et handler `get_closing_checks` dans `routes/fiscal_years.rs`, DTO camelCase, montage
      `lib.rs` (AC4).
- [ ] **Ta5** — Frontend : types et client `fiscal-years.api.ts`, composant `FiscalYearChecks.svelte` dans
      `features/fiscal-years/`, bouton et dialogue dans la page, rappel du dialogue de clôture (AC5, AC6).
- [ ] **Ta6** — i18n ×4 (AC7).
- [ ] **Ta7** — Manuel utilisateur + PDF ; `api-external.md` ; CHANGELOG (AC8, AC9).
- [ ] **Ta8** — Mutations M-a1 à M-a10 ; captures clair et sombre du panneau et du dialogue (hors dépôt,
      `/home/gcorbaz/devel/kesh-gate-logs/29-1a-captures/`) ; gate complet et E2E complet au dernier commit de code.

### 29-1b

- [ ] **Tb1** — Tests rouges d’abord : T-b1 à T-b9 (AC10-AC15, AC18).
- [ ] **Tb2** — `ReversalDating`, paramètre de `reverse_in_tx_inner`, étapes (1) et (4), nouvelles variantes de
      `DbError`, fonction publique `reverse_at_next_fiscal_year_start` (enveloppe mince calquée sur `reverse`,
      l. 2847) ; doc-comment des verrous mis à jour (AC10-AC12).
- [ ] **Tb3** — Route, handler rejoué, mapping `errors.rs`, registre des routes (AC13, AC14).
- [ ] **Tb4** — Frontend : client `reverseJournalEntryAtNextFiscalYearStart`, bouton, confirmation, refus (AC15).
- [ ] **Tb5** — i18n ×4 (AC14, AC16).
- [ ] **Tb6** — E2E de parcours (AC18).
- [ ] **Tb7** — Manuels utilisateur et d'administration + PDF ; `api-external.md` ; CHANGELOG (AC17, AC19).
- [ ] **Tb8** — Mutations M-b1 à M-b9 ; captures clair et sombre (bouton, confirmation, refus) hors dépôt ; gate
      complet et E2E complet au dernier commit de code.

## Dev Notes

### Concurrence — ce que chaque partie fait aux verrous

- **29-1a** : **aucune** écriture, **aucun** verrou, une seule vue (`ReadSnapshot`). `fiscal_years::close` **n'est
  pas touchée** — le dialogue lit les contrôles par une requête séparée, avant le `POST /close`. Ce que cela implique,
  et qui est voulu : un contrôle lu `ok` peut être faux une seconde plus tard (une transaction importée entre le
  panneau et la clôture). C'est un **instrument de revue**, non une garantie ; le manuel le dit (AC8).
- **29-1b** : mêmes verrous, même ordre que la contre-passation du jour (AC12). Le **seul** écart d'ensemble lu est
  l'exercice cherché : *celui qui suit l'origine* au lieu de *celui du jour*. Dans le cas d'usage (régularisation de
  N passée en début de N+1), les deux coïncident souvent ; quand l'utilisateur le fait plus tard (N+1 encore ouvert,
  on est en N+2), le parcours de `find_open_covering_date` s'arrête plus tôt — il verrouille **moins** de lignes,
  jamais d'autres tables.
- ⚠️ Lecture ordinaire de l'exercice suivant **puis** confirmation verrouillante : sous `REPEATABLE READ`, la lecture
  ordinaire voit la vue née à `reversal_blockers` ; un exercice suivant **créé** après cette vue (un exercice qui
  s'intercalerait entre N et le suivant lu) n'est pas vu — la contre-passation se daterait alors au début du
  suivant lu, non de l'intercalé. Cas d'école (créer un exercice **antérieur** à un exercice existant est rare, et
  refusé si un postérieur est clos) ; angle mort à écrire au doc-comment, non au manuel (C-29-1-11).

### État actuel des fichiers modifiés — ce qui doit être préservé

- `bank_accounts.rs::list_by_company_with_balances` : sa forme « une sous-requête par somme, jamais
  `bank_transactions` dans le même `FROM` » (doc l. 1387-1391) et l'invariant « une transaction, une écriture » se
  conservent tels quels dans l'extraction ; le DTO du tableau de bord ne change pas.
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
que la lecture. `CHANGELOG.md`, `api-external.md`, `messages.ftl` ×4 et `user-manual.tex` : fusion par union.

### Tests

**29-1a**

- T-a1 — `for_fiscal_year` rend les sept codes, dans l'ordre de l'AC1, sur une société vierge (tous `ok` sauf K7 si
  aucun suivant).
- T-a2 — garde statique de l'AC2 (aucun verrou ni écriture dans `closing_checks.rs` hors tests).
- T-a3 — K1 : antérieur ouvert → `attention` avec le **plus ancien** ; antérieurs tous clos → `ok`.
- T-a4 — K2 : pending au premier jour, au dernier jour, veille, lendemain ; rejetée comptée dans `rejectedCount` ;
  rapprochée non comptée ; autre société absente.
- T-a5 — K3 : `MISSING` (aucun relevé, ou CSV seul), `ENDS_BEFORE_YEAR_END`, `GAP` (transaction non rapprochée),
  `ok` (relevé au dernier jour, rapproché) ; relevé **postérieur** à la fin d'exercice ignoré (la borne) ; compte
  archivé et compte clos avant Y exclus ; compte clos pendant Y mesuré à sa date de clôture ; `NOT_COMPARABLE`.
- T-a6 — l'extraction `balances_rows` : les tests existants de `list_by_company_with_balances` verts **sans
  modification** (à nommer dans le Dev Agent Record) + un test neuf qui compare `None` et une borne postérieure à
  tous les relevés (mêmes valeurs).
- T-a7 — K4 : brouillon dans Y compté, validé ou annulé non compté, brouillon hors Y non compté.
- T-a8 — K5 : `to_complete` compté, `completed`/`discarded` non.
- T-a9 — K6 : les quatre motifs, posés en SQL direct ; une écriture ordinaire n'y figure pas ; liste bornée à 20.
- T-a10 — K7 : sans suivant → `attention` ; avec suivant → `ok` et ses données.
- T-a11 — route : 200 tout rôle (Admin, Comptable, Consultation, PAT `read`), 404 autre société, 401 sans auth ;
  forme JSON (camelCase, montants en chaînes).
- T-a12 — Vitest : le panneau (sept lignes, liens, erreur ≠ « rien à signaler »), le rappel du dialogue (bouton actif
  pendant le chargement et après un échec).
- T-a13 — E2E `fiscal-years.spec.ts` (ajout) : ouvrir *Préparer la clôture* sur un exercice avec une transaction
  pending, voir K2 en attention et le lien vers le rapprochement.

**29-1b**

- T-b1 — `NextFiscalYearStart`, sur **trois** exercices ouverts N, N+1, N+2, origine dans N : date = `start_date` de
  N+1 (non de N+2), exercice = N+1, `reverses_entry_id` = origine, lignes inversées, lettrage des lignes libres,
  audit avec `"dating"`.
- T-b2 — `Today` inchangé : les tests existants de `journal_entry_reversal_e2e.rs`, des annulations
  (`reconciliation_cancel`, `supplier_invoices`, `invoice_settlements_write`) verts sans modification ; l'audit du
  chemin `Today` sans clé `dating`.
- T-b3 — refus : sans suivant (`NEXT_FISCAL_YEAR_MISSING`), suivant clos (`NEXT_FISCAL_YEAR_CLOSED` — état posé par
  SQL direct, l'invariant I empêchant de clore N+1 avec N ouvert par les transitions ; et cas légal : origine dans N
  **clos**, N+1 clos), premier jour verrouillé (`PERIOD_LOCKED`), déjà contre-passée, contre-passation d'une
  contre-passation, écriture de pièce.
- T-b4 — garde statique de l'AC12 : dans le corps de `reverse_in_tx_inner`, la requête de l'exercice suivant apparaît
  **après** l'appel `reversal_blockers(`.
- T-b5 — concurrence (deux connexions) : la clôture de N+1 **attend** la contre-passation. Origine dans N (clos),
  N+1 ouvert. La contre-passation tient, dès l'étape (1), la ligne de N en `FOR UPDATE` (`SELECT je… JOIN
  fiscal_years fy … FOR UPDATE`, l. 2603-2607) ; `fiscal_years::close(N+1)` verrouille ses antérieurs un par un en
  (b'), dont N. Montage : une connexion A ouvre la contre-passation et s'arrête après l'étape (1) (transaction tenue
  par le test, appel de `reverse_in_tx`-équivalent en `NextFiscalYearStart`) ; B lance la clôture de N+1 et
  **attend** (`attendre_une_requete_en_cours`, motif `LOCK_EARLIER_BY_ID_SQL`) ; A termine et valide : la
  contre-passation est dans N+1 ; B aboutit ensuite. Ce que le test établit : la contre-passation au début de N+1 et
  la clôture de N+1 se **sérialisent** par le verrou de l'exercice de l'origine — une contre-passation ne peut pas
  lire N+1 ouvert puis s'y écrire après sa clôture.
- T-b6 — origine dans un exercice gappé (suivant qui ne commence pas au lendemain) : date = `start_date` du suivant.
- T-b7 — route : 201, 403 Consultation, 404 autre société, PAT `read-write` admis, rejeu inscrit au registre.
- T-b8 — Vitest : le bouton apparaît/disparaît (pas de suivant ouvert, `reversable` faux, rôle Consultation), la
  confirmation nomme date et exercice (format suisse), refus traduit.
- T-b9 — E2E de parcours (AC18), et son cas de refus.

### Mutations

| # | Mutation | Tuée par |
|---|---|---|
| M-a1 | K2 : `BETWEEN` → `>` sur `start_date` | T-a4 (premier jour) |
| M-a2 | K2 : retirer le filtre `company_id` | T-a4 (autre société) |
| M-a3 | K3 : retirer la borne `period_to <= ?` | T-a5 (relevé postérieur) |
| M-a4 | K3 : ne pas exclure les comptes archivés | T-a5 |
| M-a5 | extraction : borne appliquée aussi à `None` (ex. `COALESCE(?, '1970-01-01')`) | T-a6 |
| M-a6 | K4 : compter `validated` | T-a7 |
| M-a7 | K6 : `HAVING SUM(debit) <> SUM(credit)` retiré | T-a9 |
| M-a8 | K6 : motif `OTHER_YEAR_DATED_INSIDE` retiré | T-a9 |
| M-a9 | K7 : `>` → `>=` sur `start_date` (Y se trouve lui-même) | T-a10 |
| M-a10 | panneau : erreur de chargement affichée comme « rien à signaler » | T-a12 |
| M-b1 | `NextFiscalYearStart` daté `today` | T-b1 |
| M-b2 | lecture du suivant en `ORDER BY start_date DESC` (rend le dernier exercice, non le suivant) | T-b1 (N+2 présent) |
| M-b3 | confirmation verrouillante retirée (date et exercice pris de la lecture ordinaire) | T-b3 (le code attendu est `NEXT_FISCAL_YEAR_CLOSED` ; sans confirmation, `create_in_tx_inner` rendrait `FISCAL_YEAR_CLOSED`) |
| M-b4 | `Some(id)` → `None` (liaison perdue) | T-b1 |
| M-b5 | audit `dating` ajouté aussi au chemin `Today` | T-b2 |
| M-b6 | `NextFiscalYearClosed` rendu en `FiscalYearInvalid` | T-b3 |
| M-b7 | route non rejouée | T-b7 (registre) |
| M-b8 | bouton affiché sans exercice suivant ouvert | T-b8 |
| M-b9 | lecture du suivant placée avant `reversal_blockers` | T-b4 |

### Project Structure Notes

- Lecture neuve dans `kesh-db/src/repositories/closing_checks.rs` (les rapports de `kesh-report` sont des états
  comptables ; ceci est un contrôle de données, proche des dépôts qu'il lit). Aucune migration, aucune table.
- Composant frontend `features/fiscal-years/FiscalYearChecks.svelte` ; la page n'en porte que l'appel.

### References

- Issue #232 ; fichier d'epic `planning-artifacts/epic-29-preparer-cloture.md` ; registre
  `epic-29-choix-autonomes.md` (C-29-0-1 à 3) et `epic-29-choix-autonomes-29-1.md` (C-29-1-1 à 15).
- Fiches `14-1-cloture-report-des-soldes.md` (modèle virtuel), `14-2-reouvrir-degiger-exercice.md`,
  `14-4-bilan-ouverture-soldes-depart.md`, `25-6-a-tableau-de-bord.md` (écart du relevé), `28-3-sauvegarde-instantane-commun.md` (instantané),
  `28-4-cloture-compte-bancaire.md` (critère de l'AC7, forme de fiche).
- Rétrospective `epic-28-retro-2026-10-14.md` § 4.2 (naissance de la vue), § 4.4 (mots-clés), § 7 (B3, B4, B5).

## Dev Agent Record

### Agent Model Used

(à remplir par `bmad-dev-story`)

### Debug Log References

### Completion Notes List

- 2026-10-14 — fiche créée (analyse de contexte complète, guide de développement), deux clés `ready-for-dev`.

### File List

## Change Log

- 2026-10-14 — création (Opus 5.5, autonomie) ; inventaire de l'existant, prémisses de #232 corrigées, découpage
  29-1a / 29-1b, 19 AC, T-a1 à T-a13, T-b1 à T-b9, M-a1 à M-a10, M-b1 à M-b9, choix C-29-1-1 à 15.
