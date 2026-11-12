# Story 35-5b : la date du jour est celle de Zurich partout ailleurs dans la production — accueil, échéances, rappels, noms de fichiers, démonstration, et le serveur MCP (#929)

Status: ready-for-dev

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
  **rend avec la page** ; la route (`:622`) convertit avec elle. Le CSV de l'échéancier (`:1435`, `:1459`) lit la date **une fois** dans la route et la
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
   `payment_status.unwrap_or(PaymentStatusFilter::Unpaid)`) (le squelette : `10.03.2026`).
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
`Utc::now().date_naive()` de `dunning_eligibility.rs:98` disparaissent (la date vient d'un paramètre : 9 → 6 et 1 → 0), et des
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

- [ ] Étape 0 : partir de la 35-5a fusionnée ; recompter les sites restants, les tests 35-5b et les appelants des fonctions
      à paramètre neuf ; reconfirmer qu'aucune spec E2E n'est à cette story ; Change Log.
- [ ] Squelette branché au jour UTC de l'instant de l'horloge, tests, rouges consignés ; sites 9–16, 18–28, 16 bis, 22 bis ;
      SQL ; graine ; banc MCP à horloge (D8) ; tests 35-5b.
- [ ] Commentaires, docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit de code, `scripts/gate-ci.sh` au dernier, frontend complet, E2E
      complet au dernier commit de code.

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
`:1023-1036`, `:1754`) et la **35-2b** (`:1260-1269`) — hunks distincts ; `docs/api-external.md` aussi avec la **35-2b** ;
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

### Debug Log References

### Completion Notes List

- Fiche rédigée par le workflow `bmad-create-story` (analyse du code de `release/0.17.6`) ; validation à lancer.

### File List

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
