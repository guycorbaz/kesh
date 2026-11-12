# Story 35-5b : la date du jour est celle de Zurich partout ailleurs dans la production — accueil, échéances, rappels, noms de fichiers, démonstration, et le serveur MCP (#929)

Status: ready-for-dev

*Fiche rédigée le 2026-11-12 par l'agent de kickoff de l'Epic 35 (Opus 5.5), worktree `kesh-e35`, par le workflow
`bmad-create-story` ; remédiée et **recadrée** après la validation P1 le même jour (worktree `kesh-e355`, Change Log) : la
conversion des tests qui ne dépendent d'aucun site de cette story part à la **35-5c** (C-e350-22). Deuxième des trois
stories du découpage de la 35-5 (C-e350-14) : elle **part de la 35-5a fusionnée**, dont elle emploie `jour_suisse`, le trait
`Horloge`, `HorlogeFixe`, `AppState.horloge`, **`Lecture` à horloge** (C-e350-21), les helpers `aujourd_hui_suisse` et la
garde `date_du_jour_garde.rs`. Release **0.17.7**. **Aucune migration** (C-e350-16). Boucle complète (`kesh-db` touché). Code
lu sur **`release/0.17.6`** (`4e1c768a` ; les numéros de ligne se retrouvent à l'identique à `5e2a5818`). Registres :
`e35-choix-autonomes-e350.md` (C-e350-8, -14, -16, -21 à -25) ; `e35-choix-autonomes-e355b.md` pour la story, à créer au
premier choix.*

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

**Les signatures qui changent** (le compilateur donne la liste close) : `InvoiceResponse::from_parts(invoice, lines)`
(`routes/invoices.rs:333` ; appelée notamment par `invoice_email.rs:904`, `invoice_pdf.rs:93`, `credit_notes.rs`,
`payment_batches.rs`, `mcp/sortie.rs`) reçoit `aujourd_hui` ; `impl From<InvoiceListItem> for InvoiceListItemResponse`
(`:425`) devient une fonction à paramètre ; `services/postes_ouverts.rs:61-74` `parse_as_of(as_of)` reçoit `aujourd_hui` ;
dans `kesh-db`, la fonction de la liste paginée des factures et `due_dates_summary` (les trois `UTC_DATE()`), la liste des
candidats aux rappels (`dunning_eligibility.rs`), et `kesh_seed::seed_demo` (`lib.rs:187` ; appelée par
`auth/bootstrap.rs:1394` et `routes/onboarding.rs:258`) et les pièges (`pieges.rs:378`, `:461`) reçoivent une `NaiveDate`.
Les services `rapports::balance_agee` (`:726-732`), le rapport de projet (`:780`), `synthese_accueil` (`:870`) et
`postes_ouverts::lister` lisent `lecture.aujourd_hui()` (C-e350-21) — **sans** paramètre neuf : le MCP, qui les appelle
(`mcp/outils/balance_agee.rs:107`, `postes_ouverts.rs:247`, `factures.rs:100` → `services/factures.rs:267`, `:514-515`), reçoit
ainsi l'horloge de l'`AppState`.

Les commentaires qui justifient l'UTC deviennent faux : `dunning_eligibility.rs:9` (« **UTC** via `chrono::Utc`, cohérent
avec `UTC_DATE()` »), `invoices.rs:378` et `:396`, `invoice_email.rs:206`, `services/postes_ouverts.rs:44` et `:55`,
`services/rapports.rs:724-725` (« arrêtée à aujourd'hui (UTC) ») et `:862-866` (« ⚠️ La date est celle de l'horloge en UTC,
comme à la contre-passation… l'accueil lit encore l'exercice précédent »), `routes/invoices.rs:1029-1031` (« `Utc::now()` à
l'instant de la conversion… race midnight »), `routes/bank_accounts.rs:228` et `:1757-1759` (`closedOn` absent → « date du
jour **UTC** »). `kesh-report/src/aged_receivables.rs:27` (« `as_of` bindé (pas `UTC_DATE()` en dur) ») reste vrai.
`docs/api-external.md:252` (`closedOn` absent → « date du jour **UTC** du serveur »), `:292` (accueil, « la date du serveur,
en UTC »), `:443` (`asOf` par défaut « la date du jour, en UTC ») ; les descriptions des outils du MCP disent « arrêtée à
aujourd'hui » (`mcp/outils/balance_agee.rs:81`, `postes_ouverts.rs:215`), sans fuseau : vraies, à contrôler contre le manuel.

**Tests dont l'attendu change avec un site de cette story** (C-e350-22 : ils restent ici, sinon le gate rougit à son propre
commit) : le module de tests de `kesh-db/src/repositories/invoices.rs` (6 : `fn today()` `:3498`, cinq `CURDATE()` de
montage `:4379`, `:4393`, `:4418`, `:4679`, `:4745`) ; les fichiers de `crates/*/tests` marqués **35-5b** au verdict de la
fiche 35-5a (`dunning_reminders_e2e.rs`, `fiscal_years_e2e.rs`, `homepage_summary_e2e.rs`, `mcp_outils_e2e.rs`,
`onboarding_audit_e2e.rs`, `onboarding_e2e.rs`, `open_items_e2e.rs`, `services_motif.rs`, `kesh-db/tests/dunning_eligibility.rs`,
et `bank_accounts_closed_e2e.rs:678`, `:685`), plus les appelants de tests des fonctions à paramètre neuf (forcés par le
compilateur ; `kesh-db/tests/dunning_eligibility.rs`, `invoice_amount_due_parity.rs`, `invoice_ttc_parity.rs`, … : recompter) ;
les specs E2E qui assertent une date d'un site de cette story (accueil, échéances, rappels, postes ouverts, balance âgée,
noms de fichiers). Les autres tests vont à la **35-5c**.

**Le précédent de l'écran** : `frontend/src/lib/shared/utils/local-date-guard.test.ts` (#889) relève
`toISOString().slice(0, 10)` dans `src/**` hors tests, contre une allowlist d'entrées `{ fichier, fragment, motif }`. Le
helper E2E du jour suisse **existe déjà** : `frontend/tests/e2e/helpers/api-fixtures.ts:390-392`, `todayInZurich()`
(`toLocaleDateString('sv-SE', { timeZone: 'Europe/Zurich' })`), 11 emplois (dont `homepage-charts.spec.ts:52`).

## Décisions

- **D1 — Où naît la date** (C-e350-21, posé par la 35-5a) : les routes lisent `state.horloge.aujourd_hui()` ; les services
  `lecture.aujourd_hui()` ; les fonctions pures (`from_parts`, la conversion d'un élément de liste, `parse_as_of`) et
  `kesh-db` reçoivent une `NaiveDate` ; `kesh-seed` aussi. **Une lecture de l'horloge par requête** : une route ou un
  service lit la date une fois et la passe à tout ce qu'il appelle (la liste d'une requête juge toutes ses factures au
  même jour, ce qui ferme la « race midnight » de `routes/invoices.rs:1029-1031`). Aucun champ neuf dans `AppState`,
  `Lecture`, `EtatMcp` ou `ContexteMcp`.
- **D2 — Le SQL ne lit plus `UTC_DATE()`** : `invoices.rs:404`, `:970`, `:971` lient la date du jour en paramètre (`?`),
  reçue par la fonction du dépôt.
- **D3 — `kesh-seed`** reçoit la date de son appelant : `routes/onboarding.rs:258` (l'`AppState`) et `auth/bootstrap.rs:1394`
  (l'horloge de `main.rs`, passée à l'amorçage). `crates/kesh-seed` n'a ni binaire ni `[[bin]]` (`src/lib.rs`, `src/pieges.rs`) :
  **aucune exemption** de la garde pour la graine. Les pièges (`pieges.rs:378`, `:461`) reçoivent une `NaiveDate`.
- **D4 — Les deux sites « jour d'un instant »** (C-e350-25) : `dunning_eligibility.rs:128` compare `jour_suisse(lr.and_utc())`
  au jour du serveur, la même fonction des deux côtés ; `routes/invoices.rs:1525` écrit `jour_suisse(d.and_utc())`.
- **D5 — La garde se resserre** : sa liste d'exemptions ne porte plus **aucune** entrée de production (seule reste
  l'entrée permanente de `jour_suisse`), ni les modules de tests de `invoices.rs`, ni les fichiers 35-5b ; il reste les
  entrées 35-5c, à leur compte exact (égalité stricte, C-e350-24).
- **D6 — Les specs E2E** dont l'attendu vient d'un site de cette story prennent la date par `todayInZurich()` (aucun second
  helper) ; les autres, et l'extension de `local-date-guard.test.ts` à `tests/e2e/**`, vont à la 35-5c.
- **D7 — Les commentaires et la doc qui disaient UTC** sont réécrits (liste ci-dessus), puis `/usr/bin/grep -n UTC` sur
  chaque fichier touché, résultat au Change Log ; `api-external.md:252`, `:292`, `:443` disent « la date du jour, à Zurich »
  (hunks distincts de `:272`, 35-5a). Le manuel renvoie, pour l'accueil et les échéances, à la phrase « la date du jour est
  celle de Zurich » de la 35-5a ; il ajoute, au journal d'audit (`user-manual.tex:3955`), que le **nom** du CSV porte la date
  de Zurich alors que ses lignes restent en UTC (#669).

## Critères d'acceptation

1. **AC1 — Accueil la nuit du 1er janvier** (#929) : horloge fixée à `2025-12-31T23:30:00Z`, exercices 2025 et 2026 ouverts :
   `GET /reports/homepage-summary` lit l'exercice **2026** (rouge sur la 35-5a fusionnée : 2025).
2. **AC2 — Échéances** : facture échue le `2026-03-10`. Horloge à `2026-03-10T23:30:00Z` (00 h 30 le 11 mars) : la fiche
   (`is_overdue` vrai) et la liste la disent échue ; le filtre « échues » la rend et la synthèse de l'échéancier la compte
   (D2) ; le CSV de l'échéancier la porte comme échue. À `2026-03-10T22:30:00Z` (23 h 30 le 10) : non échue partout.
3. **AC3 — Rappels** (chiffré) :
   - `days_overdue` (`invoice_email.rs:381`) d'une facture échue le `2026-03-01` vaut **9** à `2026-03-10T22:30:00Z` et **10**
     à `2026-03-10T23:30:00Z` (la 35-5a fusionnée : 9 aux deux).
   - Éligibilité au niveau suivant : dernier rappel envoyé à `2026-03-10T23:30:00Z` (le 11 à Zurich), délai du niveau
     suivant **7** jours. À `2026-03-17T22:30:00Z` (23 h 30 le 17) : **non** éligible (la 35-5a fusionnée : éligible, `17 >=
     10 + 7`) ; à `2026-03-17T23:30:00Z` (00 h 30 le 18) : éligible. Vérifier l'arithmétique à l'écriture du test ; le point
     est que les deux jours sortent de `jour_suisse`.
   - Il n'y a **pas d'envoi automatique** (`list_reminder_candidates` n'est lu que par `GET`, `dunning_reminders.rs:166`) :
     l'effet est la liste « à rappeler », non un envoi.
4. **AC4 — Noms de fichiers et défauts**, à `2025-12-31T23:30:00Z` : la sauvegarde (`kesh-installation-2026-01-01.keshbackup`),
   l'export de souveraineté et le CSV d'audit portent `2026-01-01` ; `closedOn` par défaut vaut `2026-01-01` ; `asOf` par
   défaut des postes ouverts et `as_of` de la balance âgée valent `2026-01-01` ; le rapport de projet en `ModeProjet::Cumule`
   d'un projet sans date de fin, avec une écriture datée `2026-01-01`, **l'inclut** (fin = `2026-01-01` ; la 35-5a fusionnée
   l'exclut, fin = `2025-12-31`).
5. **AC5 — Le serveur MCP** (C-e350-21) : `AppState` à `HorlogeFixe(2025-12-31T23:30:00Z)`, banc MCP (`tests/common/mcp.rs`) :
   `kesh_balance_agee` rend `au` = `2026-01-01` ; `kesh_postes_ouverts` sans `asOf` lit au `2026-01-01` ; `kesh_factures`
   filtré sur les échues, à `2026-03-10T23:30:00Z`, rend la facture échue le `2026-03-10`.
6. **AC6 — La démonstration** : `seed_demo` à `2025-12-31T23:30:00Z` crée l'exercice « Exercice 2026 » et des pièces de 2026.
7. **AC7 — La garde** (D5) : aucune exemption de production hors l'entrée de `jour_suisse` ; les fichiers 35-5b à 0 ;
   sondes (positive, multi-ligne).
8. **AC8 — Le jour ne change rien** : à `2026-06-15T10:00:00Z`, mêmes résultats qu'avant ; tests existants verts.
9. **AC9 — Docs, CHANGELOG, PR** : D7 ; PDF régénéré, contrôlé aplati ; CHANGELOG `## [0.17.7] — Non publié`, *Corrigé* :
   la 35-5b **complète** le point #929 ouvert par la 35-5a, sans en écrire un second (C-e350-8). La PR porte `closes #929`.
   **Aucune clé i18n** ajoutée ni modifiée (`i18n-keys.test.ts` non touché) ; seul `docs/manual/fr` est régénéré —
   l'étape 0 constate que `docs/manual/{de,en,it}` ne portent aucune phrase « date du jour » à aligner.
10. **AC10 — Aucune migration**.

## Tests à écrire, et la preuve du rouge

AC1 à AC6 avec l'horloge fixée, rouges sur la 35-5a fusionnée (les sites lisent encore le jour UTC, les services du MCP
aussi) ; AC7 rouge sur la garde de la 35-5a (entrées de production encore présentes, comptes changés). Le squelette
commité d'abord branche les sites sur l'horloge et la date passée, les fonctions de calcul encore au jour UTC
(`d.date()`, `lr.date()`), de sorte que les rouges ne dépendent pas du jour réel. Journal `kesh-gate-logs/e355b/rouges.log`.

**Mutations** : M1 un site remis à `Utc::now().date_naive()` (la garde) ; M2 `UTC_DATE()` rétabli à `:404` (AC2) ; M3
`lr.date()` rétabli à `dunning_eligibility.rs:128` (AC3) ; M4 `synthese_accueil` sur `Utc::now()` (AC1, AC5 par le MCP) ;
M5 `seed_demo` sur l'année UTC (AC6).

## Tâches

- [ ] Étape 0 : partir de la 35-5a fusionnée ; recompter les sites restants, les tests 35-5b et les appelants des fonctions
      à paramètre neuf ; contrôler `docs/manual/{de,en,it}` ; Change Log.
- [ ] Squelette branché, tests, rouges consignés ; sites 9–16, 18–28, 16 bis, 22 bis ; SQL ; graine ; tests 35-5b ; specs E2E
      des sites de cette story.
- [ ] Commentaires, docs, manuel, CHANGELOG.
- [ ] Gates : `test-fast.sh --ci` complet à chaque commit de code, `scripts/gate-ci.sh` au dernier, frontend complet, E2E
      complet au dernier commit de code.

## Dev Notes

### Hors périmètre

- Les tests et specs E2E sans attendu lié à un site de cette story, et l'extension de la garde frontend aux specs :
  **35-5c** (C-e350-22).
- `reconciliation.rs:237` (`DATE(rmd.last_sent_at)` en SQL) : issue (C-e350-25). Les horodatages restent en UTC (#669),
  dont le nom horodaté des sauvegardes (`admin.rs:722`) : à côté du nom daté de Zurich (`admin.rs:99`), ce sont deux
  noms de deux natures (date contre instant).

### Fichiers communs — règle d'union C-e350-8

`bank_accounts.rs` avec la 35-5a (ici `:1802`, `:228`, `:1757`) ; `journal_entries.rs` (tests) avec la **35-5c** et la **35-8**,
qui attend la 35-5c ; `invoices.rs` (`kesh-db`) avec la **35-1** si son correctif touche `:2334-2340` (hunks distincts) ;
`routes/invoices.rs` avec la 35-5a (chaîne des appelants) ; `docs/api-external.md` (ici `:252`, `:292`, `:443` ; la 35-5a
`:272`, la 35-6 la grammaire du montant) ; `user-manual.tex` ; `CHANGELOG.md` ; `frontend/tests/e2e/helpers/api-fixtures.ts`
avec la **35-2a** (qui y ajoute un paramètre `city`) : hunks distincts.

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
