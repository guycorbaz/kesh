# Story 35-5b : la date du jour est celle de Zurich partout ailleurs dans la production — accueil, échéances, rappels, noms de fichiers, démonstration, et le serveur MCP (#929)

Status: done

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story` ; remédiée et **recadrée** après la validation P1 le même jour (worktree `kesh-e355`, Change Log) : la
conversion des tests qui ne dépendent d'aucun site de cette story part à la **35-5c** (C-e350-22). Deuxième des trois
stories du découpage de la 35-5 (C-e350-14) : elle **part de la 35-5a fusionnée** (donc de la 35-5a0, C-e350-28), dont elle
emploie `jour_suisse`, le trait `Horloge`, `HorlogeFixe`, `AppState.horloge`, **`Lecture` à horloge** (C-e350-21), le helper
`aujourd_hui_test()` (C-e350-29) et la garde `date_du_jour_garde.rs`. Release **0.17.7**. **Aucune migration** (C-e350-16).
Boucle complète (`kesh-db` touché). Code lu sur **`release/0.17.6`** (`4e1c768a` ; les numéros de ligne se retrouvent à
l'identique à `8424e36e`). Registres : `e35-choix-autonomes-e350.md` (C-e350-8, -14, -16, -21 à -26, -28, -29) ;
`e35-choix-autonomes-e355b.md` pour la story, à créer au premier choix.*

## Story

En tant que titulaire, je veux que l'accueil, les factures échues, les rappels, les noms de fichiers, la démonstration et
les réponses du serveur MCP suivent le jour suisse, même entre minuit et 1 h (2 h en été), afin que l'accueil du 1er
janvier montre le nouvel exercice et qu'une facture ne paraisse pas échue ou à l'heure selon l'heure de la nuit.

## Ce qui se lit au code (`release/0.17.6`)

L'inventaire complet est dans la fiche 35-5a (tableau des 28 sites, recompté). Le site 17 (`onboarding.rs:1043`, premier
exercice de l'assistant) est passé à la 35-5a (C-e350-23). Les **19 sites de cette story** :

| # | Site | Ce qu'il date |
|---|---|---|
| 9 | `kesh-api/src/routes/admin.rs:99` | nom du fichier de sauvegarde (`kesh-installation-AAAA-MM-JJ.keshbackup`) |
| 10 | `kesh-api/src/routes/audit_log.rs:420` | nom du CSV du journal d'audit |
| 11 | `kesh-api/src/routes/bank_accounts.rs:1802` | `closedOn` par défaut à la clôture d'un compte bancaire |
| 12 | `kesh-api/src/routes/exports.rs:58` | nom de l'export de souveraineté |
| 13 | `kesh-api/src/routes/invoice_email.rs:381` | jours de retard d'un rappel (`days_overdue`) |
| 14–15 | `kesh-api/src/routes/invoices.rs:333`, `:425` | `is_overdue` de la fiche et de la liste |
| 16 | `kesh-api/src/routes/invoices.rs:1459` | CSV de l'échéancier (`export_due_dates_csv_handler`, `:1411`) |
| 18 | `kesh-api/src/services/postes_ouverts.rs:74` | `asOf` par défaut des postes ouverts |
| 19 | `kesh-api/src/services/rapports.rs:732` | `as_of` de la balance âgée |
| 20 | `kesh-api/src/services/rapports.rs:780` | fin d'un rapport de projet sans date de fin |
| 21 | `kesh-api/src/services/rapports.rs:870` | synthèse de l'accueil (#929) |
| 22 | `kesh-db/src/repositories/dunning_eligibility.rs:98` | éligibilité aux rappels |
| 23 | `kesh-db/src/repositories/invoices.rs:404` | filtre « échues » de la liste, `i.due_date < UTC_DATE()` |
| 24–25 | `kesh-db/src/repositories/invoices.rs:970`, `:971` | compte et total des échues de la synthèse de l'échéancier, `UTC_DATE()` |
| 26 | `kesh-seed/src/lib.rs:279` | année de la démonstration |
| 27–28 | `kesh-seed/src/pieges.rs:378`, `:461` | données des pièges de démonstration |

Deux sites de la classe « jour d'un instant enregistré » (C-e350-25), attribués ici :

| # | Site | Ce qu'il calcule |
|---|---|---|
| 22 bis | `kesh-db/src/repositories/dunning_eligibility.rs:127-128` | `today >= lr.date() + délai` : le jour **UTC** du dernier rappel (`MAX(sent_at)`, `NaiveDateTime` UTC : `invoice_email.rs:661`, session `time_zone = '+00:00'`, `pool.rs:43`) comparé au jour du serveur |
| 16 bis | `kesh-api/src/routes/invoices.rs:1525` | `paid_at` du CSV de l'échéancier, `format_date(&d.date())` : jour UTC du paiement |

**Les signatures qui changent** (le compilateur donne la liste close) :
- `InvoiceResponse::from_parts(invoice, lines)` (`routes/invoices.rs:333`) reçoit `aujourd_hui` ; ses **dix** appelants,
  toutes des routes (`state` disponible) : `invoice_email.rs:904`, `invoice_pdf.rs:93`, `routes/invoices.rs:644`, `:735`,
  `:799`, `:835`, `:892`, `:1176`, `:1248`, `:1373` (`git grep -n 'InvoiceResponse::from_parts'` ; les `from_parts` de
  `credit_notes.rs:96`, `payment_batches.rs:66`, `mcp/sortie.rs:631` sont d'autres fonctions).
- `impl From<InvoiceListItem> for InvoiceListItemResponse` (`:425`) devient une fonction à paramètre ; ses emplois :
  `routes/invoices.rs:622` (liste) et `:1035` (échéancier).
- `services/postes_ouverts.rs:61-74` `parse_as_of(as_of)` reçoit `aujourd_hui`.
- `kesh-db invoices.rs` : `:404` vit dans `push_where_clauses` (`:327`), partagé par `list_by_company_paginated` (`:877`,
  `:897`) **et** `list_for_export` (`:2930`, appelée par `routes/invoices.rs:1435`, `kesh-db/tests/invoice_amount_due_parity.rs:406`,
  `:458`) ; `kf005_fulltext_index_e2e.rs:578` appelle la liste paginée ; `due_dates_summary` (`:960-978`) écrit ses deux
  `UTC_DATE()` dans le `format!` passé à `QueryBuilder::new`, **avant** les `push_bind` du `WHERE`. Toutes reçoivent la date.
- La liste des candidats aux rappels (`dunning_eligibility.rs`) reçoit la date.
- `kesh_seed::seed_demo` (`lib.rs:187`) : **un seul appelant de production**, `routes/onboarding.rs:258` (route,
  `lib.rs:1167`) ; `auth/bootstrap.rs:1394` est dans `mod tests` (`:464`), un test (validation P2, V2-1). Les pièges
  (`pieges.rs:378`, `:461`, `semer`) ne sont appelés que par les tests (`tests/common/mcp_pieges.rs:50`, `:53`).
Les services `rapports::balance_agee` (`:726-732`), le rapport de projet (`:780`), `synthese_accueil` (`:870`),
`postes_ouverts::lister` et `factures::echeances` lisent `lecture.aujourd_hui()` (C-e350-21) — **sans** paramètre neuf : le
MCP, qui les appelle (`mcp/outils/balance_agee.rs:107`, `postes_ouverts.rs:247`, et `kesh_echeancier`, `SCHEMA_ECHEANCIER`
`mcp/outils/factures.rs:296`, `paymentStatus` `overdue`, résumé `echues_nombre`, `echues_total` `:426-427`, par
`services/factures.rs:513-515`), reçoit ainsi l'horloge de l'`AppState`. `kesh_factures` (`SCHEMA_LISTE`, `:294`) n'a **pas**
de filtre « échues ». Le banc MCP construit son état lui-même (`tests/common/mcp.rs:215-217` `etat()` →
`AppState::new_for_tests`, `routeur_nu` `:221-226`, `common/mcp_pieges.rs:81-93` `banc_avec`) : aucun point d'entrée ne pose
une `HorlogeFixe` aujourd'hui.

Les commentaires qui justifient l'UTC deviennent faux : `dunning_eligibility.rs:9` (« **UTC** via `chrono::Utc`, cohérent
avec `UTC_DATE()` »), `invoices.rs:378` et `:396`, `invoice_email.rs:206`, `services/postes_ouverts.rs:44` et `:55`,
`services/rapports.rs:724-725` (« arrêtée à aujourd'hui (UTC) ») et `:862-866` (« ⚠️ La date est celle de l'horloge en UTC,
comme à la contre-passation… l'accueil lit encore l'exercice précédent »), `routes/invoices.rs:1029-1031` (« `Utc::now()` à
l'instant de la conversion… race midnight »), `routes/bank_accounts.rs:228` et `:1757-1759` (`closedOn` absent → « date du
jour **UTC** »), `routes/exports.rs:7` (« `kesh-export-{slug}-{YYYY-MM-DD}.zip` (UTC) »), `:57` (« (UTC, L13) »), `:118` (« La
date est en **UTC** (L13) »), `services/postes_ouverts.rs:55-56` (« l'écart UTC/heure suisse la nuit est hérité »),
`kesh-seed/src/lib.rs:278` (« un seul appel Utc::now() »). Restent vrais, à ne pas toucher : `audit_log.rs:89`, `:279`
(« Date (UTC) » : les lignes), `invoices.rs:109` (instant `paidAt`), `kesh-report/src/aged_receivables.rs:27` (« `as_of` bindé
(pas `UTC_DATE()` en dur) »).
`docs/api-external.md:252` (`closedOn` absent → « date du jour **UTC** du serveur »), `:292` (accueil, « la date du serveur,
en UTC »), `:443` (`asOf` par défaut « la date du jour, en UTC ») ; les descriptions des outils du MCP disent « arrêtée à
aujourd'hui » (`mcp/outils/balance_agee.rs:81`, `postes_ouverts.rs:215`), sans fuseau : vraies, à contrôler contre le manuel.

**Tests dont l'attendu change avec un site de cette story** (C-e350-22 : ils restent ici, sinon le gate rougit à son propre
commit) : le module de tests de `kesh-db/src/repositories/invoices.rs` (6 : `fn today()` `:3498`, son `Utc::now()` à
`:3499`, cinq `CURDATE()` de montage `:4379`, `:4393`, `:4418`, `:4679`, `:4745`) ; les fichiers de `crates/*/tests` marqués
**35-5b** au verdict de la fiche 35-5a (`dunning_reminders_e2e.rs`, `homepage_summary_e2e.rs`, `mcp_outils_e2e.rs`,
`onboarding_audit_e2e.rs`, `onboarding_e2e.rs`, `open_items_e2e.rs`, `services_motif.rs`, `kesh-db/tests/dunning_eligibility.rs`,
et les parts `fiscal_years_e2e.rs:1284`, `bank_accounts_closed_e2e.rs:678`, `:685`), plus les appelants de tests des
fonctions à paramètre neuf (forcés par le compilateur ; `kesh-db/tests/dunning_eligibility.rs`, `invoice_amount_due_parity.rs`,
`invoice_ttc_parity.rs`, `kf005_fulltext_index_e2e.rs`, `auth/bootstrap.rs:1394` (`mod tests`), `tests/common/mcp_pieges.rs`, … :
recompter). **Specs E2E : aucune** n'asserte une date d'un site de cette story (relevé de la validation P2, V2-6 :
`open-items.spec.ts:39` et `homepage-charts.spec.ts:52` lisent déjà Zurich ; `overdueDate` et `daysFromToday` prennent des
marges de 1 à 40 jours ; `export-global.spec.ts:103` ne teste qu'un motif ; `audit-log.spec.ts:56-59` doit **rester** au jour
UTC, son filtre comparant des horodatages UTC) — à reconfirmer à l'étape 0. Les autres tests vont à la **35-5c**.

**Le précédent de l'écran** : `frontend/src/lib/shared/utils/local-date-guard.test.ts` (#889) relève
`toISOString().slice(0, 10)` dans `src/**` hors tests, contre une allowlist d'entrées `{ fichier, fragment, motif }`. Le
helper E2E du jour suisse **existe déjà** : `frontend/tests/e2e/helpers/api-fixtures.ts:390-392`, `todayInZurich()`
(`toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' })`), 11 emplois (dont `homepage-charts.spec.ts:52`).

## Décisions

- **D1 — Où naît la date** (C-e350-21, posé par la 35-5a0) : les routes lisent `state.horloge.aujourd_hui()` ; les services
  `lecture.aujourd_hui()` ; les fonctions pures (`from_parts`, la conversion d'un élément de liste, `parse_as_of`) et
  `kesh-db` reçoivent une `NaiveDate` ; `kesh-seed` aussi. **Une lecture de l'horloge par requête** : une route ou un
  service lit la date une fois et la passe à tout ce qu'il appelle. **Quand une route convertit ce qu'un service a rendu,
  elle emploie la date du service** (V2-4 de la validation P2) : `Echeancier` (`services/factures.rs:160`) porte un champ
  `pub aujourd_hui: NaiveDate`, la date employée pour le filtre « échues » et pour le résumé ; la route de l'échéancier
  (`routes/invoices.rs:1018-1036`) convertit ses éléments avec elle, sans relire l'horloge. **De même pour la liste** (V3-7,
  tranché) : `factures::lister` (`services/factures.rs:236-267`) lit `lecture.aujourd_hui()`, la passe à
  `list_by_company_paginated` (qui ne l'emploie pas, la liste ne filtrant pas sur le paiement : `payment_status: None`) et la
  **rend avec la page**, dans un **type de service neuf** sur le modèle d'`Echeancier` — `ListeFactures { page:
  InvoiceListResult, aujourd_hui: NaiveDate }`, sans toucher `InvoiceListResult` de `kesh-db` (`invoices.rs:320-325`) ; la route
  (`:612`, conversion `:622`) convertit avec elle. Consommateurs à adapter (V4-2 de la validation P4) : l'outil MCP
  `mcp/outils/factures.rs:349` (qui recopie `items/total/limit/offset`), `tests/common/mcp.rs:435` et les appels de
  `tests/services_rollout.rs` (`:641`, `:1299`, `:1414`, `:2101`, `:2540`, … : recompter). Le CSV de l'échéancier (`:1435`, `:1459`) lit la date **une fois** dans la route et la
  passe à `list_for_export` et à la mise en forme. C'est ce qui ferme la « race midnight » de `routes/invoices.rs:1029-1031`
  (le commentaire est réécrit en ce sens). Aucun champ neuf dans `AppState`, `Lecture`, `EtatMcp` ou `ContexteMcp`.
- **D2 — Le SQL ne lit plus `UTC_DATE()`** : `invoices.rs:404`, `:970`, `:971` lient la date du jour **par `push_bind`**, dans
  l'ordre des arguments — les deux du `SELECT` de `due_dates_summary` avant ceux du `WHERE` (le `format!` d'aujourd'hui passé
  à `QueryBuilder::new` ne reçoit aucun lien : il se réécrit en `push` + `push_bind`) ; `:404` dans `push_where_clauses` sert
  à la fois la liste paginée et `list_for_export`.
- **D3 — `kesh-seed`** reçoit la date de son **seul appelant de production**, `routes/onboarding.rs:258`
  (`state.horloge.aujourd_hui()`) ; `auth/bootstrap.rs:1394`, dans `mod tests`, passe une **date fixe** (V3-9) et se range
  parmi les appelants de tests ; `ensure_admin_user` ne change pas. `crates/kesh-seed` n'a ni binaire ni
  `[[bin]]` (`src/lib.rs`, `src/pieges.rs`) : **aucune exemption** de la garde pour la graine. Les pièges (`pieges.rs:378`,
  `:461`) reçoivent une `NaiveDate` de leurs appelants de tests (`tests/common/mcp_pieges.rs:50`, `:53`).
- **D4 — Les deux sites « jour d'un instant »** (C-e350-25) : `dunning_eligibility.rs:128` compare `jour_suisse(lr.and_utc())`
  au jour du serveur, la même fonction des deux côtés ; `routes/invoices.rs:1525` écrit `jour_suisse(d.and_utc())`.
  *(Annoté le 2026-11-14, revue P1 E1 : `paid_at` n'est pas un instant — ses écrivains posent la date du règlement à minuit ;
  le site 16 bis revient à `d.date()`, C-e355b-6. Le site 22 bis reste tel qu'écrit.)*
- **D5 — La garde se resserre** : sa liste d'exemptions ne porte plus d'entrée de production **hors les deux entrées
  permanentes** — `kesh-core/src/horloge.rs` (1, le `.date_naive()` de `jour_suisse`) et `kesh-api/src/routes/admin.rs` (1,
  `:722`, `Utc::now().format("%Y%m%dT%H%M%S%3f")`, horodatage du nom de sauvegarde, #669) —, ni les modules de tests de
  `invoices.rs`, ni les fichiers 35-5b ; il reste les entrées 35-5c, à leur compte exact (égalité stricte, C-e350-24).
- **D9 — Les modules de tests de `src`** (V3-9) : les 6 sites de `kesh-db invoices.rs` (`fn today()` `:3498-3499`, cinq
  `CURDATE()`) et leurs appels de `list_by_company_paginated` / `due_dates_summary` (`:4000` à `:6055`), et
  `auth/bootstrap.rs:1394`, prennent une **date fixe** passée en argument : le test de nuit s'y fait par la valeur (une date
  de nuit fixée là où l'attendu en dépend), non par `KESH_TEST_INSTANT`, que ces modules ne lisent pas.
- **D6 — Les specs E2E** : aucune n'est à la 35-5b (relevé ci-dessus, à reconfirmer à l'étape 0 ; si une le devient, elle
  prend `todayInZurich()`, sans second helper). Les specs, et l'extension de `local-date-guard.test.ts` à `tests/e2e/**`,
  vont à la 35-5c — `audit-log.spec.ts` y garde un jour UTC, motivé.
- **D8 — Le banc MCP reçoit une horloge** (V2-2) : `tests/common/mcp.rs` gagne une entrée à horloge fixée
  (`routeur_avec_horloge(pool, instant)`, ou un paramètre `horloge` d'`etat`), et `common/mcp_pieges.rs` une
  `banc_avec_horloge` ; les entrées existantes gardent `new_for_tests` (donc `horloge_de_test()` et `KESH_TEST_INSTANT`,
  C-e350-26, -29). **Montage de l'AC5** (V3-8) : société **complète** (`societe_complete`, comme `mcp_outils_e2e.rs:1425`), sans
  les pièges — `societe_piegee` sème une facture validée à `due_date` = jour passé à `semer` (`pieges.rs:549-569`), qui
  fausserait `echuesNombre` — plus la seule facture du critère.
- **D7 — Les commentaires et la doc qui disaient UTC** sont réécrits (liste ci-dessus), puis `/usr/bin/grep -n UTC` sur
  chaque fichier touché, résultat au Change Log ; `api-external.md:252`, `:292`, `:443` disent « la date du jour, à Zurich »
  (hunks distincts de `:272`, 35-5a). Le manuel renvoie, pour l'accueil et les échéances, à la phrase « la date du jour est
  celle de Zurich » de la 35-5a ; il ajoute, au journal d'audit (`user-manual.tex:3955`), que le **nom** du CSV porte la date
  de Zurich alors que ses lignes et ses filtres restent en UTC (#669).

## Critères d'acceptation

1. **AC1 — Accueil la nuit du 1er janvier** (#929) : horloge fixée à `2025-12-31T23:30:00Z`, exercices 2025 et 2026 ouverts :
   `GET /reports/homepage-summary` lit l'exercice **2026** (rouge sur le squelette : 2025).
2. **AC2 — Échéances** : facture échue le `2026-03-10`. Horloge à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars) : la fiche
   (`is_overdue` vrai) et la liste la disent échue ; le filtre « échues » la rend, **avec `isOverdue: true`**, et la synthèse
   de l'échéancier la compte (D1, D2 : même date pour le filtre, le résumé et la conversion) ; le CSV de l'échéancier la
   porte comme échue. À `2026-03-10T22:30:00Z` (23 h 30 le 10) : non échue partout. **Paiement** (16 bis) : une facture
   réglée à `2026-03-10T23:30:00Z` (`paid_at`) porte `11.03.2026` dans la colonne de paiement du CSV, exporté **avec
   `paymentStatus=all`** (ou `paid` ; sans paramètre, l'export ne rend que les impayées : `requete_echeances`,
   `payment_status.unwrap_or(PaymentStatusFilter::Unpaid)`) (le squelette : `10.03.2026`). *(Annoté le 2026-11-14, revue P1
   E1 : prémisse fausse, aucun écrivain ne pose `paid_at` à 23:30 ; le critère devient « une facture réglée par la route de nuit,
   `settledOn` du 10, porte `10.03.2026` » — C-e355b-6.)*
3. **AC3 — Rappels** (chiffré) :
   - `days_overdue` (`invoice_email.rs:381`) d'une facture échue le `2026-03-01` vaut **9** à `2026-03-10T22:30:00Z` et **10**
     à `2026-03-10T23:30:00Z` (le squelette : 9 aux deux), lu par `GET /invoices/{id}/reminder-preview`
     (`invoice_email.rs:409`).
   - Éligibilité au niveau suivant : dernier rappel envoyé à `2026-03-10T23:30:00Z` (le 11 à Zurich), délai du niveau
     suivant **7** jours. À `2026-03-17T22:30:00Z` (23 h 30 le 17) : **non** éligible (le squelette : éligible, `17 >=
     10 + 7`) ; à `2026-03-17T23:30:00Z` (00 h 30 le 18) : éligible. Vérifier l'arithmétique à l'écriture du test ; le point
     est que les deux jours sortent de `jour_suisse`.
   - Il n'y a **pas d'envoi automatique** (`list_reminder_candidates` n'est lu que par `GET`, `dunning_reminders.rs:166`) :
     l'effet est la liste « à rappeler », non un envoi.
4. **AC4 — Noms de fichiers et défauts**, à `2025-12-31T23:30:00Z` : la sauvegarde (`kesh-installation-2026-01-01.keshbackup`),
   l'export de souveraineté et le CSV d'audit portent `2026-01-01` ; `closedOn` par défaut vaut `2026-01-01` ; `asOf` par
   défaut des postes ouverts et `as_of` de la balance âgée valent `2026-01-01` ; le rapport de projet en `ModeProjet::Cumule`
   d'un projet sans date de fin, avec une écriture datée `2026-01-01`, **l'inclut** (fin = `2026-01-01` ; le squelette
   l'exclut, fin = `2025-12-31`).
5. **AC5 — Le serveur MCP** (C-e350-21, D8) : banc MCP à `HorlogeFixe(2025-12-31T23:30:00Z)` : `kesh_balance_agee` rend
   `au` = `2026-01-01` ; `kesh_postes_ouverts` sans `asOf` lit au `2026-01-01`. Banc à `HorlogeFixe(2026-03-10T23:30:00Z)` :
   `kesh_echeancier` avec `paymentStatus: overdue` rend la facture échue le `2026-03-10`, et `echuesNombre = 1` ; à
   `2026-03-10T22:30:00Z`, aucune, `echuesNombre = 0`.
6. **AC6 — La démonstration** : `seed_demo` (par `POST /onboarding/seed-demo`, `routes/onboarding.rs:258`) à
   `2025-12-31T23:30:00Z` crée l'exercice « Exercice 2026 », du `2026-01-01` au `2026-12-31` (la graine ne crée aucune
   pièce : `kesh-seed/src/lib.rs:21-23` n'importe ni `invoices` ni `journal_entries`).
7. **AC7 — La garde** (D5) : aucune exemption de production hors les deux entrées permanentes (`kesh-core/src/horloge.rs` 1,
   `admin.rs` 1 pour `:722`) ; les fichiers 35-5b à 0 ; sondes (positive, multi-ligne).
8. **AC8 — Le jour ne change rien, et la nuit non plus** : à `2026-06-15T10:00:00Z`, mêmes résultats qu'avant ; tests
   existants verts. **Preuve de nuit** (C-e350-26), sur les **binaires d'intégration** (`crates/*/tests`) : ceux des fichiers
   35-5b, lancés une fois avec `KESH_TEST_INSTANT=2025-12-31T23:30:00Z` puis `2026-03-10T23:30:00Z`, sont verts ; et **tous**
   les binaires avec `KESH_TEST_INSTANT="$(date -u +%F)T23:30:00Z"`, de jour (la nuit réelle reproduite, comme à la 35-5a),
   chaque rouge rattaché à un fichier 35-5c ; journal `kesh-gate-logs/e355b/nuit.log`. Les `mod tests` de `src` ne lisent pas
   la variable : leur nuit se prouve par une date fixe (D9).
9. **AC9 — Docs, CHANGELOG, PR** : D7 ; PDF régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`, *Corrigé* :
   la 35-5b **complète** le point #929 ouvert par la 35-5a, sans en écrire un second (C-e350-8). La PR porte `closes #929`.
   **Aucune clé i18n** ajoutée ni modifiée (`i18n-keys.test.ts` non touché) ; seul `docs/manual/fr` est régénéré —
   `docs/manual/{de,en,it}` ne contiennent que leur `README.md` (constaté à la validation P2).
10. **AC10 — Aucune migration**.

## Tests à écrire, et la preuve du rouge

Sur la 35-5a fusionnée, ces sites lisent encore `Utc::now()` — l'horloge **réelle**, que `HorlogeFixe` n'atteint pas : les
tests d'AC1 à AC6 n'y rougissent pas pour la bonne raison (lancés le 2026-11-12, l'accueil lit 2026, `days_overdue` vaut ≈ 256…),
et ceux de `kesh-db` n'y compilent pas (validation P2, V2-3). D'où **le squelette, commité d'abord** : chaque site est
branché sur l'horloge de sa requête et la date passée, mais reçoit **le jour UTC de l'instant de l'horloge**
(`state.horloge.maintenant().date_naive()`, `lecture.horloge().maintenant().date_naive()` — `Lecture::horloge()` posé par la
35-5a0), et 16 bis / 22 bis gardent `d.date()`, `lr.date()`. **Le squelette réajuste les comptes de la garde** (V3-1 : ils
bougent, une forme ne « remplace » pas toujours l'autre dans le même fichier) : les `UTC_DATE()` de `kesh-db invoices.rs` et le
`Utc::now().date_naive()` de `dunning_eligibility.rs:98` disparaissent (la date vient d'un paramètre ; en **occurrences**, l'unité de la garde : `kesh-db invoices.rs` 10 → 7, `:4379` en portant
deux, et `dunning_eligibility.rs` 1 → 0), et des
`….maintenant().date_naive()` paraissent dans des fichiers qui n'en comptaient pas (`services/factures.rs`,
`routes/onboarding.rs`, `routes/dunning_reminders.rs`, `routes/reports.rs`, et davantage dans `routes/invoices.rs`) ; chaque
entrée prend le compte que rend l'exécution de la garde. Les rouges sont alors **déterministes** et justes : AC1 lit 2025 ;
AC3 : 9 aux deux, éligible au 17 ; AC4 : écriture du 1er janvier exclue ; AC2 : non échue à 23:30Z, paiement au `10.03.2026` ;
AC5, AC6 de même. L'implémentation remplace ces jours par `aujourd_hui()` et `jour_suisse(…and_utc())` ; **AC7 rougit alors sur
les seules entrées 35-5b dont le compte est encore non nul**, et passe quand toutes sont à 0. Journal
`kesh-gate-logs/e355b/rouges.log`.

**Mutations** : M1 un site remis à `Utc::now().date_naive()` (la garde) ; M2 `UTC_DATE()` rétabli à `:404` (AC2) ; M3
`lr.date()` rétabli à `dunning_eligibility.rs:128` (AC3) ; M4 `synthese_accueil` sur `Utc::now()` (AC1 ; l'accueil n'est pas
servi par le MCP, `services/mod.rs:278`, `services_motif.rs:1392`) ;
M5 `seed_demo` sur l'année UTC (AC6) ; M6 `d.date()` rétabli à `routes/invoices.rs:1525` (AC2, paiement) ; M7 la conversion
de l'échéancier prend une autre date que `Echeancier::aujourd_hui`. Pour que M7 soit attrapable, la conversion est une
fonction **de l'`Echeancier` seul** (sans paramètre de date) ; un test unitaire lui donne un `Echeancier` daté du
`2026-03-11` dont un élément échoit le `2026-03-10`, et attend `isOverdue: true` quel que soit le jour réel.

## Tâches

- [x] Étape 0 : partir de la 35-5a fusionnée ; recompter les sites restants, les tests 35-5b et les appelants des fonctions
      à paramètre neuf ; reconfirmer qu'aucune spec E2E n'est à cette story ; Change Log.
- [x] Squelette branché au jour UTC de l'instant de l'horloge, tests, rouges consignés ; sites 9–16, 18–28, 16 bis, 22 bis ;
      SQL ; graine ; banc MCP à horloge (D8) ; tests 35-5b.
- [x] Commentaires, docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit de code, `scripts/gate-ci.sh` au dernier, frontend complet, E2E
      complet au dernier commit de code. *(Annoté au passage en `review` : `test-fast` et `gate-ci` au dernier commit de code
      `e3e8256fa`, verts ; frontend non touché ; **E2E à la clôture**, règle de l'Epic 35. Le squelette `777664745` a eu son gate
      complet, rouge, corrigé au commit suivant — Change Log.)*

## Dev Notes

### Hors périmètre

- Les tests et specs E2E sans attendu lié à un site de cette story, et l'extension de la garde frontend aux specs :
  **35-5c** (C-e350-22).
- `reconciliation.rs:237` (`DATE(rmd.last_sent_at)` en SQL) : **#1053** (C-e350-25). **Limite connue**, écrite aux Dev Notes et
  au Change Log : après cette story, l'éligibilité aux rappels compte depuis le jour **suisse** du dernier rappel
  (`dunning_eligibility.rs:128`) tandis que la date d'exigibilité de la réconciliation garde son jour **UTC** — le mélange de
  deux horloges que `companies.rs:986-989` déconseille, jusqu'à #1053. Les horodatages restent en UTC (#669),
  dont le nom horodaté des sauvegardes (`admin.rs:722`) : à côté du nom daté de Zurich (`admin.rs:99`), ce sont deux
  noms de deux natures (date contre instant).

### Fichiers communs — règle d'union C-e350-8

`bank_accounts.rs` avec la 35-5a (ici `:1802`, `:228`, `:1757`) ; `journal_entries.rs` (tests) avec la **35-5c** et la **35-8**,
qui attend la 35-5c ; `invoices.rs` (`kesh-db`) avec la **35-1** si son correctif touche `:2334-2340` (hunks distincts) ;
`routes/invoices.rs` avec la 35-5a (chaîne des appelants) ; `docs/api-external.md` (ici `:252`, `:292`, `:443` ; la 35-5a
`:272` ; la 35-6 n'y touche plus, C-e350-19) ; `user-manual.tex` ; `CHANGELOG.md` ; `tests/common/mcp.rs` et
`common/mcp_pieges.rs` avec la 35-5a0 (qui touche déjà `mcp.rs` pour l'horloge de l'`AppState`) ;
`crates/kesh-api/tests/date_du_jour_garde.rs` (liste d'exemptions : 35-5a0, 35-5a, puis ici, puis 35-5c).
Et (V3-3) : `crates/kesh-api/src/routes/invoice_email.rs` (ici `:206`, `:381`, `:904`) avec la **35-2a** (`:808`,
`:1023-1036` ; `:1754` est un appel que la fiche 35-2a dit laisser inchangé, à constater) et la **35-2b** (`:1260-1269`) —
hunks distincts ; écrit aussi au registre (C-e350-8, suite) ; `docs/api-external.md` aussi avec la **35-2b** ;
`routes/onboarding.rs` (35-5a `:1043`, ici `:258`), `services/factures.rs` (35-5a0 `reglements` `:534-558`, ici `echeances`,
`lister`, `Echeancier`), `tests/services_motif.rs` et `tests/onboarding_audit_e2e.rs` (35-5a0 `Lecture::new(`, ici les traces et
`seed_demo`), `fiscal_years_e2e.rs` et `bank_accounts_closed_e2e.rs` (parts 35-5a et 35-5b) : ordre 35-5a0 → 35-5a → 35-5b,
chacune partant de la précédente fusionnée.

### Ce qui doit rester vrai

- Les horodatages restent en UTC (#669). Aucune donnée existante réécrite. Fixtures synthétiques seulement.
- La lecture seule du MCP (Q11) inchangée.

### Références

- Issue #929 ; fiches 35-5a et 35-5c ; #889 et `local-date-guard.test.ts`.
- `CLAUDE.md` : § *Exception `kesh-db`* ; § *Inventorier les sites NON RÉSOLUS* ; § *Test Locally First* (`cargo test`
  contre `nextest`).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (agent de développement, worktree `kesh-e355b`, clé `e355b`, voie 1), 2026-11-13/14.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e355b/` (environnement et commit en tête de chacun) :
`gate-squelette-test-fast.log` (squelette), `rouges.log` et `rouges-ac7.log`, `verts.log`, `mut/M1.log` à `mut/M7.log` et
`mut/resume.log`, `gate-test-fast.log`, `nuit.log` (synthèse d'AC8) et `nuit-{S1,N1,B1,B2,B3}.log`, `gate-ci.log` et
`gate-ci.journal`, `manuel-make.log`, `pdf-aplati.log`. Scripts : `reset-db-e355b.sh`, `test-fast-e355b.sh`, `cibles-e355b.sh`,
`nuit-e355b.sh`, `mut/mutations-e355b.sh`, `mut/mut-e355b.py`.

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.
- **Commits** : `777664745` squelette (sites branchés sur l'horloge, au jour UTC de son instant ; signatures ; banc MCP à horloge ;
  conversions des tests 35-5b ; module de tests de `kesh-db invoices.rs` en date fixe ; garde ramenée à l'exécution) ;
  `e3e8256fa` implémentation (jour de Zurich, D4, `FicheFacture::aujourd_hui`, tests neufs, garde finale, docs, manuel `.tex`,
  CHANGELOG, registre) ; puis le commit de compte rendu (fiche, PDF, `sprint-status.yaml`, prompt de revue). **Aucune migration**
  (`git diff --stat de2626a85 e3e8256fa -- crates/kesh-db/migrations` vide).
- **Tests** : 11 tests neufs, 0 retiré (`git diff de2626a85 e3e8256fa -- crates`, attributs `#[sqlx::test]`/`#[test]`/`#[tokio::test]`
  ajoutés : 11, retirés : 0) — `test-fast` passe de 5358 (35-5a, `527f57839`) à 5369.
- **Choix** : `e35-choix-autonomes-e355b.md`, C-e355b-1 à -4.
- **Gates du passage en `review`** au dernier commit de code `e3e8256fa` : `test-fast --ci` **5369/5369, 13 ignorés**
  (`gate-test-fast.log`) ; `gate-ci.sh kesh_ci_e355b` **5376 passés, 0 échoué, 17 ignorés** (`gate-ci.log`). Frontend **non
  touché** (`git diff --stat de2626a85 e3e8256fa -- frontend` vide) : non lancé. **E2E : non lancé**, à la clôture.
- Voir le Change Log (entrée du développement) pour l'étape 0, les rouges, l'inventaire par exécution et les mutations.
- *(Annoté le 2026-11-14, remédiation de la revue P1.)* Commit de code `73dc06b1e` (3 tests neufs, 0 retiré — attributs
  ajoutés/retirés de `git show 73dc06b1e -- crates` : 3 / 0 ; le test du paiement est remplacé sous un autre nom) ; `test-fast --ci`
  **5372/5372, 13 ignorés** au commit `73dc06b1e`, base remise à neuf (`p1-test-fast.log`). Choix C-e355b-5 et -6. **`gate-ci` et
  E2E : non lancés**, à la clôture (règle de l'Epic 35). Détail : entrée « Remédiation P1 » du Change Log.

### File List

**Production** : `crates/kesh-db/src/repositories/{invoices,dunning_eligibility}.rs` ; `crates/kesh-seed/src/{lib,pieges}.rs` ;
`crates/kesh-api/src/services/{factures,postes_ouverts,rapports}.rs` ; `crates/kesh-api/src/routes/{admin,audit_log,bank_accounts,
dunning_reminders,exports,invoice_email,invoice_pdf,invoices,onboarding}.rs` ; `crates/kesh-api/src/mcp/outils/factures.rs` ;
`crates/kesh-api/src/auth/bootstrap.rs` (`mod tests`).

**Tests** : neufs ou étendus — `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs` (AC1 à AC4), `mcp_outils_e2e.rs` (AC5),
`fiscal_years_e2e.rs` (AC6), `crates/kesh-db/tests/dunning_eligibility.rs` (AC3, dépôt), `routes/invoices.rs` (`mod tests`, M7) ;
banc — `tests/common/{mcp,mcp_pieges}.rs` ; conversions — `bank_accounts_closed_e2e`, `dunning_reminders_e2e`,
`homepage_summary_e2e`, `onboarding_audit_e2e`, `onboarding_e2e`, `open_items_e2e`, `services_motif`, `services_rollout` ;
appelants — `kesh-db/tests/{invoice_amount_due_parity,invoice_ttc_parity,kf005_fulltext_index_e2e}.rs` ; garde —
`date_du_jour_garde.rs`.

**Docs** : `docs/api-external.md` (`closedOn`, synthèse de l'accueil, `asOf`), `docs/testing.md:119`,
`docs/manual/fr/user-manual.{tex,pdf}` (note `sec:date-du-jour`, export CSV du journal d'audit), `CHANGELOG.md` (point #929
complété). Registre `e35-choix-autonomes-e355b.md`. Aucune clé i18n, aucune spec E2E, aucun fichier frontend.

## Change Log

- 2026-11-12 — Fiche rédigée (Epic 35, découpage préventif C-e350-14). Validation non lancée.
- 2026-11-12 — **Validation P1** (`kesh-gate-logs/e35-5b-validation-p1.md` : 1 HIGH, 7 MEDIUM, 4 LOW) remédiée par l'agent
  de rédaction (worktree `kesh-e355`). Prémisses lues au code de `release/0.17.6`.
  - **V1-1 (HIGH)** : la date des services naît de la `Lecture`, qui porte l'horloge (C-e350-21, posé par la 35-5a) ; le MCP,
    qui construit sa `Lecture` au câblage avec `state.horloge`, la reçoit ; signatures changées listées (`from_parts`,
    conversion d'un élément de liste, `parse_as_of`, dépôts, graine) ; AC5 (MCP).
  - **V1-2** : `dunning_eligibility.rs:128` en site 22 bis (D4), `routes/invoices.rs:1525` en 16 bis (C-e350-25) ; AC3
    chiffré.
  - **V1-3** : AC3 et AC4 chiffrés (`days_overdue` 9 / 10 ; éligibilité au 17 / au 18 ; rapport de projet `Cumule`).
  - **V1-4** : garde récrite à la 35-5a (C-e350-24) — multi-ligne, commentaires écartés, égalité stricte par fichier ; les
    5 sites coupés et les 2 commentaires (`aged_receivables.rs:27`, `invoices.rs:5040`) sont comptés ou écartés là-bas.
  - **V1-5** : 34 + 1 specs (non 36) ; `todayInZurich()` réutilisé, aucun second helper ; l'allowlist et les formes Node
    locales (`getFullYear`/`getMonth`/`getDate`, `overdueDate`) passent à la 35-5c, qui étend la garde.
  - **V1-6** : `api-external.md:252`, `:292`, commentaires `rapports.rs:724-725`, `:862-866`, `routes/invoices.rs:1029-1031`,
    `bank_accounts.rs:228`, `:1757-1759`, `postes_ouverts.rs:44` ajoutés (D7) ; descriptions des outils MCP relues.
    **Réfuté en partie** : `user-manual.tex:260` ne parle pas de « date du poste » (`/usr/bin/grep -n 'du poste'
    docs/manual/fr/user-manual.tex` : vide) ; la distinction poste / Zurich est écrite à la 35-5a quand même.
  - **V1-7** : l'écran du verrou part à la 35-5a (D7 et AC4 de la 35-5a), dont c'est la borne.
  - **V1-8** : découpage adopté (C-e350-22) — la 35-5b garde la production, le SQL, la graine, la plomberie, la doc et les
    tests dont l'attendu change avec ses sites ; la **35-5c** convertit le reste et ferme les gardes ; la 35-8 attend la
    35-5c. `sprint-status.yaml`, fichier d'epic et C-e350-14 mis à jour.
  - **LOW** : V1-9 (aucun binaire dans `kesh-seed` : pas d'exemption, D3) ; V1-10 (référence `5e2a5818` ; 35 specs) ; V1-11
    (nom du CSV d'audit au manuel, D7) ; V1-12 (aucune clé i18n ; manuels DE/EN/IT à constater à l'étape 0).
  - Aucune migration. Statut `ready-for-dev` ; la validation P2 est à lancer par l'orchestrateur, après celle de la 35-5a.
- 2026-11-12 — **Validation P2** (Opus, `kesh-gate-logs/e35-5b-validation-p2.md` : 0 HIGH, 6 MEDIUM, 5 LOW ; série P1 → P2 :
  1 HIGH + 7 MEDIUM → 6 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de
  `release/0.17.6`.
  - **V2-1** : `auth/bootstrap.rs:1394` est dans `mod tests` (`:464`, lu) ; `seed_demo` n'a qu'un appelant de production,
    `routes/onboarding.rs:258` (D3) ; « passée à l'amorçage » retiré.
  - **V2-2** : AC5 sur `kesh_echeancier` (`SCHEMA_ECHEANCIER`, `paymentStatus`, lu à `mcp/outils/factures.rs:296`) ;
    `kesh_factures` n'a pas de filtre « échues » ; banc MCP à horloge fixée (D8), `tests/common/mcp.rs` et `mcp_pieges.rs` aux
    fichiers communs.
  - **V2-3** : le squelette passe le jour UTC de l'instant de l'horloge (`Lecture::horloge()` ajouté à la 35-5a) ; « rouge
    sur la 35-5a fusionnée » remplacé par « rouge sur le squelette » dans AC1 à AC4.
  - **V2-4** : `Echeancier` porte la date employée, la route la réemploie ; le CSV lit une fois ; M7 (D1).
  - **V2-5** : AC2 pour `paid_at` (16 bis), M6.
  - **V2-6** : aucune spec E2E à la 35-5b (relevé de la validation, à reconfirmer) ; `audit-log.spec.ts` reste au jour UTC,
    porté à la 35-5c.
  - **LOW** : V2-7 (dix appelants de `InvoiceResponse::from_parts`) ; V2-8 (`push_where_clauses` partagé avec
    `list_for_export`, `push_bind` dans l'ordre) ; V2-9 (la graine ne crée que l'exercice) ; V2-10 (commentaires UTC de
    `exports.rs`, `postes_ouverts.rs:55-56`, `kesh-seed/src/lib.rs:278`) ; V2-11 (`:3499` ; manuels DE/EN/IT : `README.md`
    seul).
  - Ajout : preuve de nuit par `KESH_TEST_INSTANT` à AC8 (C-e350-26) ; la 35-6 ne touche plus `api-external.md`.
  - Aucun finding réfuté. Signal de découpage : non levé (trois crates de production, docs ; la validation juge la story
    tenable après C-e350-22) ; défauts distincts de ceux de P1. Aucune migration. Statut `ready-for-dev`.
- 2026-11-12 — **Validation P3** (Sonnet, `kesh-gate-logs/e35-5b-validation-p3.md` : 0 HIGH, 2 MEDIUM, 7 LOW ; série 1 HIGH + 7
  → 6 → 2 MEDIUM) remédiée par l'agent de rédaction (worktree `kesh-e355`). Prémisses lues au code de `release/0.17.6`.
  - **V3-1** : le squelette réajuste les comptes de la garde (`dunning_eligibility.rs` 1 → 0, `kesh-db invoices.rs` 9 → 6,
    formes neuves dans des fichiers à 0) ; AC7 rouge ensuite sur les seules entrées 35-5b non nulles.
  - **V3-2** : D5 et AC7 nomment les deux entrées permanentes (`kesh-core/src/horloge.rs`, `admin.rs:722`), comme la 35-5a et
    la 35-5c.
  - **LOW** : V3-3 (fichiers communs : `invoice_email.rs` avec 35-2a et 35-2b, `api-external.md` avec 35-2b, fichiers partagés
    avec 35-5a0/35-5a et leur ordre) ; V3-4 (l'issue existe : **#1053** ; limite écrite) ; V3-5 (M4 : AC1 seul, l'accueil
    n'étant pas servi par le MCP) ; V3-6 (CSV avec `paymentStatus=all`) ; V3-7 (`lister` rend sa date avec la page) ; V3-8
    (banc MCP sur `societe_complete`) ; V3-9 (D9 : dates fixes dans les `mod tests` de `src` ; AC8 limité aux binaires
    d'intégration, plus l'exécution « nuit réelle » sur tous les binaires).
  - Découpage de la 35-5a (C-e350-28) répercuté : la plomberie est à la 35-5a0 ; helper `aujourd_hui_test()` (C-e350-29).
  - Aucun finding réfuté. Signal de découpage : non levé. Aucune migration. Statut `ready-for-dev`.
- 2026-11-12 — **Validation P4 ciblée** (`kesh-gate-logs/e35-5b-validation-p4-ciblee.md`, lentille unique sur `0dda8159` :
  0 HIGH, 0 MEDIUM, 3 LOW). LOW appliqués : V4-1 (« 9 → 6 » devient « 10 → 7 » en occurrences, `:4379` en portant deux) ; V4-2
  (`lister` rend un type de service `ListeFactures { page, aujourd_hui }`, l'outil MCP `mcp/outils/factures.rs:349` et les bancs
  nommés) ; V4-3 (fichiers communs avec 35-2a et 35-2b portés au registre, extension de C-e350-8 ; `:1754` qualifié d'appel).
  Aucun finding réfuté.
  **Validation terminée.** Série : P1 1 HIGH + 7 MEDIUM → P2 6 MEDIUM → P3 2 MEDIUM → P4 ciblée 0 au-dessus de LOW. Modèles :
  P1 (complète), P2 Opus, P3 Sonnet, P4 ciblée. Les remédiations n'ont touché que la fiche, le registre, l'epic et le
  `sprint-status.yaml`. Aucune migration. Statut `ready-for-dev`.
- 2026-11-13/14 — **Développement** (Opus 5.5, worktree `kesh-e355b`, base `de2626a85` = 35-5a close, voie 1). Journaux :
  `/home/gcorbaz/devel/kesh-gate-logs/e355b/`.
  - **Étape 0.** Sites recomptés sur `de2626a85` (`git grep -nE 'date_naive\(|naive_utc\(\)\s*\.date\(|UTC_DATE|CURDATE|Local::|
    Utc::now\(\)\.(year|month|day|format)\(' -- 'crates/*/src/*'`, commentaires écartés) : les 19 sites de la fiche, aux lignes de
    `release/0.17.6` sauf `routes/invoices.rs:1459` → `:1463` (CSV), et les deux « jours d'un instant » (`invoices.rs:1525`,
    `dunning_eligibility.rs:128`). Les trois sites que la synthèse de la 35-5a laisse au jour UTC (seed `kesh-seed/src/lib.rs:279`,
    `closedOn` `bank_accounts.rs:1802`, balance âgée `rapports.rs:732`, borne du projet cumulé `rapports.rs:780`) sont bien les sites
    26, 11, 19 et 20 de cette fiche : aucun site hors périmètre *(annoté le 2026-11-14, revue P1 A3 : lire « les quatre sites »)*. Appelants à paramètre neuf : donnés par le compilateur (liste au
    File List). **Specs E2E reconfirmées** (relevé `/usr/bin/grep -rln 'isOverdue|overdue|asOf|…' frontend/tests/e2e`, lignes lues) :
    `invoices_echeancier.spec.ts:110-121` (échéances à −1 et +30 jours), `reports.spec.ts:190`, `:260-262` (−40 jours),
    `dunning-roundtrip`/`reminders`/`homepage-reminders` (`overdueDate()`, marges ≥ 1 jour), `open-items.spec.ts:39`
    (`toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' })`), `homepage-charts.spec.ts:52` (`todayInZurich`),
    `bank-accounts-closed.spec.ts:144` (`closedOn` envoyé par l'écran, non le défaut) : **aucune** n'asserte une date d'un site de la
    story ; liste vide, transmise à la 35-5c. Manuels DE/EN/IT : `README.md` seuls (inchangé).
  - **Squelette** (`777664745`) : routes sur `state.horloge.maintenant().date_naive()`, services sur
    `lecture.horloge().maintenant().date_naive()` ; `kesh-db` et `kesh-seed` reçoivent la date ; le SQL la lie (`push_bind`, D2 —
    les deux liens du `SELECT` de `due_dates_summary` avant ceux du `WHERE`, les fragments `INVOICE_*_SQL` et
    `amount_due_derived_joins()` sans `?`, relevé `grep`) ; `Echeancier::aujourd_hui`, `ListeFactures` (D1) ; réponse de l'échéancier
    en fonction de l'`Echeancier` seul (`reponse_echeancier`, M7) ; banc MCP `routeur_avec_horloge` / `banc_avec_horloge` (D8) ;
    module de tests de `kesh-db invoices.rs` en **date fixe** `2026-06-15` dès le squelette (D9, C-e355b-1 : la garde y tombe à 0,
    non à 7 comme la fiche le prévoyait) ; tests 35-5b convertis à `aujourd_hui_test()` ; comptes de la garde ramenés à
    l'exécution. **Gate complet du squelette** (`gate-squelette-test-fast.log`) : 5357/5359, **2 rouges** — (1)
    `services_motif::ac13_ix_les_seuls_usages_de_state_d_un_handler_migre` (`routes/invoices.rs : get_invoice — 3 occurrences de
    state, 2 sous une forme permise`) : la garde de la 32-4b refuse `state.horloge` dans un handler migré ; **défaut du squelette**,
    corrigé au commit suivant par `FicheFacture::aujourd_hui` (C-e355b-3, extension de D1 à la fiche) ; (2)
    `kesh-db::dunning_eligibility::le_jour_du_dernier_rappel_est_celui_de_zurich`, test neuf commité avec le squelette : son rouge
    est **le rouge d'AC3 côté dépôt** (`lr.date()`), voulu. Un intermittent hors story, `mcp_transport_e2e::
    abandon_au_delai_le_permis_reste_pris` (#1126), rouge au premier essai, vert à la reprise. **Écart** : le squelette a été commité
    avec ces deux rouges ; la fiche voulait un squelette vert, puis les rouges des tests neufs.
  - **Rouges** (`rouges.log`, arbre = squelette + `FicheFacture::aujourd_hui` + tests neufs, non commité ; en-tête) *(annoté le
    2026-11-14, revue P1 B5 : cet arbre portait deux fichiers de **production** modifiés non commités, `routes/invoices.rs` et
    `services/factures.rs`, dont le diff n'est pas au journal — le rejeu à l'identique est impossible ; les motifs relevés restent
    ceux du jour UTC, et M7 est prouvée par mutation)* : **10 rouges**
    sur les 11 tests neufs, chacun pour la bonne raison, motif lu — AC1 (`asOf` 2025-12-31, exercice 2025) ; AC2 (fiche, liste,
    filtre, résumé, statut CSV et nom du CSV à la veille : `false`, `None`, 0, `Impayée`, `echeancier-2026-03-10.csv`) ; AC2
    paiement (`10.03.2026`) ; AC3 HTTP (`9 jours de retard` aux deux instants ; l'assertion d'éligibilité, qui suit, n'est pas
    atteinte — l'éligibilité est prouvée rouge au dépôt et par M3) ; AC3 dépôt (éligible le 17) ; AC4 noms (`…-2025-12-31…` ×3) ;
    AC4 défauts (`closedOn`, deux `asOf` au 31 décembre, total du projet 0) ; AC5 ×2 (`au` 2025-12-31 ×2 ; `echuesNombre` 0 et
    facture absente à 00 h 30) ; AC6 (`Exercice 2025`). Le 11e, M7 (test unitaire), est vert sur le squelette par construction : la
    fonction de l'`Echeancier` seul y existe déjà (sa preuve est la mutation M7). **AC7** (`rouges-ac7.log`) : l'implémentation
    sous la liste du squelette rougit sur les **seules entrées 35-5b**, toutes à 0 (et `admin.rs` 2 → 1).
  - **Implémentation** (`e3e8256fa`) : `aujourd_hui()` partout (routes, services) ; `jour_suisse(lr.and_utc())` et
    `jour_suisse(d.and_utc())` (D4) ; garde : plus aucune exemption de production hors `kesh-core/src/horloge.rs` (1) et `admin.rs`
    (1, `:724`) — 31 → 10 entrées (recompté sur `de2626a85` et `HEAD` : 21 retirées, 11 de production et 10 de tests ;
    `admin.rs` 2 → 1 ; restent les deux permanentes et huit entrées de tests 35-5c, dont le module de tests de `journal_entries.rs`). **Verts** (`verts.log`) : 28/28 (les 11 neufs, les tests
    de nuit de la 35-5a, la garde et ses sondes).
  - **Inventaire par exécution (AC8)** — synthèse et tri ligne par ligne : `nuit.log`. **Sonde positive** (`nuit-S1.log`) : l'arbre
    du squelette sous `KESH_TEST_INSTANT=2026-11-13T23:30:00Z` rend **12 rouges** sur les binaires 35-5b, dont 10 mêlent un site au
    jour UTC 13 et un montage au jour suisse 14 (`clore_un_compte`, deux tests de l'accueil, quatre du MCP aux postes ouverts, deux
    des postes ouverts, la trace de la balance âgée) : la méthode voit. Sur `e3e8256fa` : **nuit réelle reproduite, tous les binaires,
    5369/5369** (`nuit-N1.log`) ; et `gate-test-fast.log` a tourné **réellement de nuit** (00 h 35 le 14 à Zurich), 5369/5369. Les deux
    instants de la fiche sur les 16 binaires 35-5b (`nuit-B1.log`, `nuit-B2.log`) : 308/309, un rouge,
    `services_rollout::ac11_a_les_services_lisent_en_lecture_seule` — montage à date fixe `2026-06-01` (`:206`) lu à l'`asOf` par
    défaut ; la sonde **de jour** au 11 mars 2026 (`nuit-B3.log`) rend le même : artefact de l'instant antérieur aux dates fixes de
    2026 (classe de #1125, fichier 35-5c), non un mélange de jours. **Écart à la lettre d'AC8** (« verts sous les deux instants ») :
    ce seul rouge, déclaré. Angles morts : § 3 de `nuit.log`. *(Annoté le 2026-11-14, revue P1 : « fichier 35-5c » est inexact —
    `services_rollout.rs` est touché par la 35-5b et la fiche 35-5c ne le nomme pas ; classe (b) de la 35-5a, jamais rouge à l'heure
    réelle, C-e355b-5 (A2). Le témoin **de jour** d'AC8 (`2026-06-15T10:00:00Z`) n'avait pas tourné et le compte rendu ne le disait
    pas (A1) : joué à la remédiation. L'angle mort E7 (`sent_at` sur l'horloge réelle) manquait au § 3 de `nuit.log`.)*
  - **Mutations** (`mut/`, restauration `git checkout --` puis `touch`, au commit `e3e8256fa`) : M1 (`exports.rs`, nom de l'export sur
    `Utc::now()`) 6 rouges — la garde et `ac4_les_noms_de_fichiers_la_nuit_du_nouvel_an` ; M2 (`UTC_DATE()` rétabli dans
    `push_where_clauses`) 7 — garde, `ac2_une_facture_echue_la_nuit`, `ac5_l_echeancier_du_mcp_juge_au_jour_suisse` ; M3 (`lr.date()`)
    2 — `le_jour_du_dernier_rappel_est_celui_de_zurich`, `ac3_les_rappels_comptent_en_jours_suisses` ; M4 (`synthese_accueil` sur
    `Utc::now()`) 6 — garde, `ac1_l_accueil_la_nuit_du_nouvel_an` ; M5 (`seed_demo` au jour UTC de l'instant, à la route) 6 — garde,
    `demo_a_minuit_le_1er_janvier_cree_l_exercice_suisse` ; M6 (`d.date()` du CSV) 1 — `ac2_la_date_de_paiement_du_csv_est_le_jour_suisse` ;
    M7 (réponse de l'échéancier sur `Utc::now()`) 6 — garde, `la_reponse_de_l_echeancier_juge_au_jour_de_l_echeancier`. **Toutes
    tuées**, chacune par un test nommé de la story. *(Annoté le 2026-11-14, revue P1 E1 : M6 et son test portaient sur une valeur de
    `paid_at` qu'aucun écrivain ne produit ; test remplacé, M6 remplacée par M6b — entrée « Remédiation P1 ».)*
  - **Docs** (D7, AC9) : `api-external.md` `:252` (`closedOn`), `:292` (accueil), `:443` (`asOf`) disent Zurich ; commentaires
    réécrits (`dunning_eligibility.rs:9`, `invoices.rs` `:378`, `:396`, `invoice_email.rs:206`, `postes_ouverts.rs:44`, `:55-56`,
    `rapports.rs:724-725`, `:862-866`, `routes/invoices.rs:1029-1031`, `bank_accounts.rs:228`, `:1757-1759`, `exports.rs:7`, `:57`,
    `:118`, `kesh-seed/src/lib.rs:278`). `/usr/bin/grep -n 'UTC\|Utc::now'` sur chaque fichier de production touché : ne restent que
    des **instants** (`admin.rs:724` horodatage, `audit_log.rs:89`, `:279`, `:421` lignes et filtres, `dunning_reminders.rs:267`,
    `invoice_email.rs:662` `sent_at`, `invoices.rs:109` `paidAt`, `:2587` `UTC_TIMESTAMP(6)`, `bootstrap.rs:806-809`, les
    `set_paid_at_for_test` du module de tests) et les commentaires qui disent l'instant UTC (`invoices.rs:1565`,
    `dunning_eligibility.rs:11`, `exports.rs:59`). Manuel : la note `sec:date-du-jour` énumère les sites convertis, dit que les heures
    restent en UTC et nomme la limite #1053 ; l'export CSV du journal d'audit dit son nom à la date de Zurich. PDF régénéré (`make
    user`, 3 passes, `manuel-make.log`), aplati (`pdf-aplati.log`) : « temps universel » 0, « restent pour l'instant au jour UTC » 0,
    les quatre phrases neuves 1 chacune. CHANGELOG : le point #929 de la 35-5a **complété** (titre et corps), sans second point.
    `docs/testing.md:119` mis à jour. Aucune clé i18n.
  - **Limite connue** (fiche, *Hors périmètre*) : l'éligibilité aux rappels compte depuis le jour **suisse** du dernier rappel quand la
    fenêtre de 30 jours de la réconciliation (`reconciliation.rs:237`, `DATE(rmd.last_sent_at)`) garde son jour **UTC**, jusqu'à
    #1053 — écrite au manuel et au CHANGELOG.
  - **Écarts** : le squelette commité rouge (ci-dessus) ; C-e355b-3 (la fiche d'une facture prend la date du service, non nommée par
    D1) ; AC8 (un rouge d'artefact sous les instants de la fiche). Consigne Edit : code et tests par l'outil Edit seulement (dont
    `replace_all` pour les remplacements identiques d'un fichier) ; `cargo fmt` pour les retours de ligne ; scripts Python seulement
    pour les mutations (`mut/mut-e355b.py`) ; `sed -i` employé une fois sur ce script hors dépôt (adaptation du nom de la story) ;
    `git checkout 777664745 -- crates` puis `git checkout HEAD -- crates` pour la sonde positive, tout le code étant commité avant.
  - **Gates du passage en `review`**, au dernier commit de code `e3e8256fa`, bases remises à neuf : `scripts/test-fast.sh --ci`
    **5369 exécutés, 5369 passés, 13 ignorés** (`gate-test-fast.log`) ; `scripts/gate-ci.sh kesh_ci_e355b` **5376 passés, 0
    échoué, 17 ignorés** (`gate-ci.log`, `gate-ci.journal`). Frontend non touché, non lancé. **E2E : non lancé**, à la clôture.
    Statut : `review`.
- 2026-11-14 — **Remédiation de la revue P1** (Opus 5.5, worktree `kesh-e355b`, voie 1). Revue P1 : Opus, trois lentilles
  (`kesh-gate-logs/e355b-review-p1-{B,E,A}.md`) — B : 6 LOW ; E : 1 MEDIUM, 6 LOW ; A : 1 MEDIUM, 4 LOW ; soit **2 MEDIUM distincts**
  (E1, A1 ; B1 = E1) et des LOW recoupés (B2 = B3 = E5 = A5, B4 = E2). Décisions de l'orchestrateur ; prémisse de chaque finding lue
  au code. Commit de code **`73dc06b1e`** ; journaux `kesh-gate-logs/e355b/p1-*`.
  - **E1 (MEDIUM) = B1** — lu : `invoice_settlements_write.rs:391-392` (`settled_on.and_hms_opt(0, 0, 0)`), `:745` (passage en
    perte), `routes/reconciliation.rs:2584-2585` (`paid_at_candidate.and_time(00:00)`) : `paid_at` porte la date du règlement à
    minuit, non un instant. Le CSV de l'échéancier revient à `format_date(&d.date())`, commentaire juste ; le test
    `ac2_la_date_de_paiement_du_csv_est_le_jour_suisse` (valeur `23:30:00` posée à la main) devient
    `ac2_la_date_de_paiement_du_csv_est_celle_du_reglement` : règlement réel par la route à 00 h 30 le 11 mars, `settledOn` du 10 →
    `paid_at` = `2026-03-10 00:00:00`, colonne `10.03.2026`. Doc d'`Invoice::paid_at` (« Horodate » → date du règlement à minuit),
    manuel (note `sec:date-du-jour` : le paiement n'est pas une heure mais une date), CHANGELOG (point #929 : plus de changement
    annoncé pour la date de paiement de l'export). `docs/api-external.md` n'en parlait pas (`/usr/bin/grep -n` sur « paiement »,
    `paidAt` : seules les lignes 520, 561, 648, 662, 728, sans rapport). Choix C-e355b-6 ; C-e350-25 annoté ; D4 et AC2 annotés.
    **Propagation** : `git grep -noE "d\.and_utc|date_de_paiement_du_csv[a-z_]*|16 bis|paid_at.{0,30}(instant|UTC)|date de paiement
    de l.export"` — ne restent que des relevés datés de la fiche, du prompt P1 et du registre (annotés), et des `d.and_utc` d'autres
    sites (`bank_imports.rs:345`, `reconciliation.rs:3032`, `:3044`, `:6162` : instants de réponses JSON, hors story).
  - **A1 (MEDIUM)** — le témoin de jour d'AC8 n'avait pas tourné. Joué : `KESH_TEST_INSTANT=2026-06-15T10:00:00Z`, les 16 binaires
    35-5b, commit `73dc06b1e` : **312/312** (`p1-nuit-jour.log` ; 309 de B1/B2 + 3 tests neufs ; `services_rollout::ac11_a…` vert,
    l'instant étant postérieur au 1er juin 2026). Écrit au § 5 de `nuit.log` ; le relevé de développement est annoté, non réécrit.
  - **B4 = E2 (LOW, production)** — `render_reminder` reçoit `today` ; `send_reminder_batch` lit `state.horloge.aujourd_hui()`
    **une fois** et la passe par `send_one_batch_reminder` ; l'aperçu lit la sienne. **Aucun test** : sous une horloge arrêtée,
    lecture unique et lecture répétée sont équivalentes — la garantie est structurelle (signature), mutation non jouée car
    équivalente sous tout banc.
  - **B2, B3 = E5 = A5 (LOW)** — `VueEcheance` lit `summary.overdueTotal` (108.10 la nuit, 0 la veille) et la présence de la
    facture dans `export.csv?paymentStatus=overdue` (présente / absente) : `ac2_une_facture_echue_la_nuit`.
  - **E3 (LOW)** — `ac1_l_accueil_la_nuit_du_nouvel_an_sans_exercice_neuf` : `noOpenFiscalYear`, `asOf` 2026-01-01, `fiscalYear`
    nul. **E4 (LOW)** — `ac4_la_fin_donnee_d_un_projet_l_emporte_sur_le_jour` : projet clos le 2026-06-30, écriture du
    2026-06-01 comptée à 00 h 30 le 1er janvier. **E6 (LOW)** — `ac2_minuit_pile_en_heure_d_ete` : fiche non échue à
    `2026-07-01T21:59:59Z`, échue à `22:00:00Z` (minuit le 2 juillet, UTC+2).
  - **E7 (LOW)** — angle mort écrit au § 4 de `nuit.log` : `sent_at` d'un rappel envoyé par la route sur l'horloge réelle
    (`invoice_email.rs`, `chrono::Utc::now().naive_utc()`), l'éligibilité sur l'horloge de l'état ; même horloge en production,
    mêlées sous `HorlogeFixe` ou `KESH_TEST_INSTANT`.
  - **A2 (LOW)** — C-e355b-5 ; « fichier hors 35-5b », « verdict 35-5c » annotés ici et au § 4 de `nuit.log` ; à tracer en
    commentaire de #1125 (orchestrateur). **A3** — « les trois sites » annoté (quatre). **A4** — renvois `\S\ref{sec:date-du-jour}`
    depuis l'accueil (« Quel exercice ») et l'échéancier (« Factures échues impayées »). **B5** — `rouges.log` avait tourné avec deux
    fichiers de production non commités (`routes/invoices.rs`, `services/factures.rs`), sans leur diff : déclaré par annotation à
    l'entrée « Rouges ». **B6** — commentaire de la garde : l'entrée `journal_entries.rs` sous `src` compte le seul module de tests.
  - **Rouges d'abord, mutations** (au commit `73dc06b1e`, arbre de code propre, restauration `git checkout --` puis `touch` ;
    `p1-mut-resume.log`, `p1-mut-*.log`) : M6b (colonne de paiement = jour de la requête) 1 rouge —
    `ac2_la_date_de_paiement_du_csv_est_celle_du_reglement` ; M8 (second lien du `SELECT` de `due_dates_summary` lié à la veille, sans
    forme gardée) 1 — `ac2_une_facture_echue_la_nuit` ; M9 (`list_for_export` reçoit la veille, sans forme gardée) 1 — même test ;
    M10 (`let end = today;`) 1 — `ac4_la_fin_donnee_d_un_projet_l_emporte_sur_le_jour` ; M11 (accueil au jour UTC de l'instant) 7 —
    garde (5) et les deux tests d'AC1 ; M12 (fiche au jour UTC de l'instant) 7 — garde (5), `ac2_minuit_pile_en_heure_d_ete`,
    `ac2_une_facture_echue_la_nuit`. **Toutes tuées.** Les rouges des tests neufs sont prouvés par ces mutations, non par un
    `rouges.log` sur arbre antérieur (les sites existaient déjà). La première exécution de M12 a été **tuée au plafond mémoire**
    pendant la compilation (`p1-mut-M12.log`, rc=137) ; rejouée seule (`p1-mut-M12-reprise.log`).
  - **Manuel** : `make user` (`p1-manuel-make.log`, 2 passes, stable) ; PDF aplati (`p1-pdf-aplati.log`) : « envoi d'un rappel,
    paiement » 0, « date de paiement de l'export » 0, les phrases neuves 1 chacune, renvois résolus (§7.6).
  - **Gates** au dernier commit de code `73dc06b1e` : ciblé `p1-cibles.log` 101/101 (arbre avant commit, mêmes fichiers) ;
    **`scripts/test-fast.sh --ci` 5372/5372, 13 ignorés** (`p1-test-fast.log`, base `kesh_e355b` remise à neuf). Frontend non
    touché, non lancé. **`gate-ci` et E2E : non lancés**, à la clôture.
  - **Écarts** : un premier lancement des mutations a été refusé par le garde-fou de la session — le script restaurait par
    `git checkout --` des fichiers qui portaient encore la remédiation non commitée ; le code a été commité d'abord, puis les
    mutations jouées. Consigne Edit tenue (code et tests par l'outil Edit ; `cargo fmt`) ; scripts hors dépôt pour les mutations et
    les journaux seulement (`p1-mutations.sh`, `p1-mutation-M12-reprise.sh`, `p1-cibles.sh`). Statut : `review`, revue P2 ciblée à
    lancer (prompt `35-5b-date-du-jour-reste-review-prompt-p2-ciblee.md`).
- 2026-11-14 — **Clôture : synthèse finale** (Opus 5.5, worktree `kesh-e355b`, voie 1). Statut : **`done`**.
  - **Tendance de la boucle de revue** : **P1** (Opus, trois lentilles B, E, A ; `kesh-gate-logs/e355b-review-p1-{A,B,E}.md`)
    **2 MEDIUM** — E1 (`paid_at` n'est pas un instant : site 16 bis de l'export CSV, son test, M6 et le manuel ; même constat que
    B1, classé LOW par sa lentille) et A1 (témoin « de jour » d'AC8 non tourné, non déclaré) — et une quinzaine de LOW, remédiés au commit `73dc06b1e`
    (entrée précédente) → **P2 ciblée** (Sonnet, une lentille sur `73dc06b1e` ; `kesh-gate-logs/e355b-review-p2-ciblee.md`)
    **0 au-dessus de LOW**, 2 LOW (C1 : doc-comment `today_utc` de `InvoiceResponse::is_overdue`, `routes/invoices.rs:251` ; C2 :
    `paid_at` des lignes antérieures à la v0.12, limite non annoncée), tracés en **#1138** et **#1139**. Série : 2 → 0 MEDIUM.
    Boucle close sur une passe sans finding au-dessus de LOW, dont la remédiation éventuelle ne touche aucune ligne de production
    (aucune n'a été appliquée).
  - **Chiffres de clôture**, au dernier commit de code `73dc06b1e` (joués à HEAD `ebd5b654c`, qui ne porte que des comptes rendus) :
    - `scripts/gate-ci.sh kesh_ci_e355b` (voie 1, verrous de la station) : **5379 passés, 0 échoué, 17 ignorés**, code 0 —
      `kesh-gate-logs/e355b/cloture-gate-ci.log` et `cloture-gate-ci.journal`, commit `ebd5b654c`. 5376 au passage en `review`
      (`gate-ci.log`, `e3e8256fa`) + les 3 tests neufs de `73dc06b1e` = 5379 : recompté.
    - **E2E complet** sur `kesh_e2e_e355b` recréée (91 migrations, `scripts/seed-dev-db.sql`), frontend du worktree construit
      (`cloture-front-build.log`), backend sur le port 3413 (montage du `CLAUDE.md`, dossiers `target/e2e-r1/`) : **403 passés,
      7 échoués, 18 ignorés** (22,0 min) — `kesh-gate-logs/e355b/cloture-e2e.log` (script `cloture-e2e.sh`, journal du backend
      `cloture-e2e-backend.log`), commit `ebd5b654c`. Les sept échecs sont, un par un, ceux de la KF-029 (#97) au tableau de
      `docs/testing.md` § « Les échecs attendus » : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
      `onboarding.spec.ts:57`, `:77`, `:150`. Aucun échec hors de la liste, aucune relance nécessaire.
    - Rappel : `test-fast --ci` **5372/5372, 13 ignorés** à `73dc06b1e` (`p1-test-fast.log`). Frontend non touché par la story.
  - **Issues** : #929 (refs ; la 35-5c reste à faire) ; LOW de revue en #1138 et #1139.
