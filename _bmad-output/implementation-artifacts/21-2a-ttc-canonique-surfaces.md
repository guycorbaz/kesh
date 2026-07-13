# Story 21.2a: Montant TTC canonique — helper, SQL, QR, PDF, e-mail, échéancier

Status: review

<!-- Spec créée le 2026-07-12 (bmad-create-story, Fable 5). Source : planning epic-21-echeances-relances.md §« Décision préalable #246 » + issue #246 + cartographie 2 agents Explore (backend + frontend).
SPLIT 2026-07-12 (règle de splitting préventif CLAUDE.md — validate 4 passes sans convergence, friction concentrée sur la réconciliation) : cette story 21-2a = primitives (helper + SQL) + surfaces de présentation ; la RÉCONCILIATION (matching TTC, câblage, seeds, UI candidats) = story 21-2b-reconciliation-ttc.md, qui consomme 21-2a. #246 fermé par 21-2b (dernier morceau). -->

## Story

En tant que **PME assujettie à la TVA**,
je veux **que chaque montant présenté comme « total dû » (QR-facture, PDF, e-mail, échéancier) soit le TTC réellement dû**,
afin de **ne plus facturer le HT à mes clients par le QR et de pouvoir construire les rappels (#231) et la balance âgée sur des montants justes**.

## Contexte du bug (#246 — vérifié ground-truth)

`invoices.total_amount` = **HT** (`compute_total` = Σ `line_total`, `repositories/invoices.rs:287-292`) et DOIT le rester (source comptable : crédit produit = HT, DC9). Le débit créance comptable est TTC (`generate_invoice_journal_lines` :1044-1056 : `total_ht + Σ line_vat_amount par ligne`, DC7 « somme d'arrondis par ligne, jamais réarrondir une base agrégée »). Or 7 surfaces présentent le HT comme montant dû. Invisible en dogfooding (TVA 0 → HT = TTC). **L'avoir fait DÉJÀ juste** (`credit_notes.rs:230-235` : `Σ (line_total + line_vat_amount(...))`) — c'est le patron canonique à extraire.

## Acceptance Criteria

### A. Helper canonique (kesh-core)

1. **`invoice_total_ttc`** dans `crates/kesh-core/src/accounting/vat.rs` (à côté de `line_vat_amount` :39-43) : Σ par ligne de `line_total + line_vat_amount(line_total, vat_rate)`. Signature générique sur `(Decimal, Decimal)` (line_total, vat_rate) — itérateur ou slice, au choix du dev (les entités `InvoiceLine`/`CreditNoteLine` exposent les deux champs). Doc `///` : équivalence prouvée avec le débit créance (l'agrégation par taux du journal est associative — Σ_taux Σ_lignes = Σ_lignes), et l'interdit DC7 (jamais `round` sur une base agrégée).
2. **Tests unitaires kesh-core** : parité avec le calcul du journal (multi-lignes multi-taux, cas d'arrondi half-away 123.455, taux 0, base négative avoir, ligne unique 100.00 @ 8.1 → 108.10, deux lignes 0.05 @ 8.1 chacune → les arrondis PAR LIGNE s'additionnent — pas d'arrondi du total).
3. **DRY — l'avoir consomme le helper** : `credit_notes.rs:230-235` (calcul inline `ttc`) refactoré vers `invoice_total_ttc` (iso-comportement — les tests avoirs existants sont le filet). `supplier_invoices.rs:143` NE change PAS (modèle différent : `total_amount` fournisseur est déjà TTC persisté).

### B. Équivalent SQL + test de parité

4. **Expression SQL canonique** (constantes **`pub` exportées par kesh-db** — PAS `pub(crate)` : la balance âgée 21-7 vit dans `kesh-report`, crate séparé qui écrit son propre SQL et doit consommer la MÊME source de vérité, sinon elle dupliquerait le ROUND hors du filet de parité) — **DEUX formes, mêmes 2 arguments d'arrondi** :
   - **Forme scalaire par facture** (colonne de SELECT, filtre BETWEEN) — sous-requête corrélée, suppose l'alias `i` sur `invoices` :
     `(SELECT COALESCE(SUM(l.line_total + ROUND(l.line_total * l.vat_rate / 100, 2)), 0) FROM invoice_lines l WHERE l.invoice_id = i.id)`
   - **Forme agrégat multi-factures** (`due_dates_summary`) — **table dérivée jointe** :
     `LEFT JOIN (SELECT invoice_id, SUM(line_total + ROUND(line_total * vat_rate / 100, 2)) AS ttc FROM invoice_lines GROUP BY invoice_id) lt ON lt.invoice_id = i.id` puis `SUM(COALESCE(lt.ttc, 0))` (et le `CASE WHEN overdue THEN COALESCE(lt.ttc, 0)`).
     **Rationale (corrigé Pass 3)** : MariaDB 10.11 ACCEPTE `SUM((sous-requête corrélée))` et rend le bon résultat (vérifié à l'exécution) — le choix de la table dérivée est un choix de **performance** (la corrélée serait ré-évaluée par ligne, ×2 avec le `CASE` overdue ; la 21-7 fera 5 `CASE` bucketés sur la même expression), pas un interdit SQL.
   - **Intégration QueryBuilder** (pseudo-code) : la constante scalaire est un `&'static str` poussé tel quel — `qb.push("SELECT ")… qb.push(INVOICE_TTC_SUBQUERY_SQL); qb.push(" AS total_ttc …")` ; la forme dérivée est poussée dans la clause FROM. Les deux constantes documentent leur prérequis d'alias (`i` / `lt`).
   — `ROUND(x, 2)` MariaDB sur DECIMAL = half-away-from-zero = `MidpointAwayFromZero` de `line_vat_amount` ; `line_total` DECIMAL(19,4) × `vat_rate` DECIMAL(5,2) = exact avant ROUND (pas de f64). ⚠️ **Premier usage de `ROUND` SQL du projet** (grep vide) — d'où l'AC 5, qui doit couvrir LES DEUX formes.
   - **Factures legacy sans lignes** (théorique : `validate_lines` interdit 0 ligne via l'API, mais défense) : `COALESCE(…, 0)` → TTC 0 — comportement assumé et documenté (une telle facture n'a rien à payer).
5. **Test d'intégration de parité SQL ≡ Rust** (kesh-db, série) : seed facture multi-lignes à taux mixtes incluant des cas d'arrondi limites (ex. 0.05 @ 8.1 ×2, 123.455 @ 7.7, ligne @ 0) → **les DEUX formes SQL assertées indépendamment** (la scalaire via requête directe de la constante, la dérivée via `due_dates_summary`) == `invoice_total_ttc(lines)` == débit créance de l'écriture générée. Quatre voies, une valeur — chacune des deux formes porte son propre `ROUND`, tester l'une ne couvre PAS l'autre.

### C. Surfaces corrigées (backend)

6. **QR-facture** : `invoice_pdf_service.rs:213` `amount: Some(<TTC>)` — les lignes sont déjà chargées par le service (`find_by_id_with_lines`), appliquer le helper. Le débiteur qui scanne paie le TTC.
7. **PDF facture** : `invoice_pdf_service.rs:249` `total: <TTC>` — le libellé `invoice-pdf-total-ttc` (« Total TTC », `kesh-qrbill/pdf.rs:352-372`) dit enfin vrai. AUCUN changement dans kesh-qrbill (le renderer reçoit `total`, l'avoir le prouve). Le récap TVA détaillé reste #151 (hors scope).
8. **Variable `{amount}` e-mail** : `invoice_email.rs:180` → `format_money(&<TTC>)`. **Changement de signature explicite** : `build_invoice_vars(invoice, contact, company, language)` (`:155-160`) ne reçoit pas les lignes et l'entité `Invoice` ne les porte pas → ajouter un paramètre `lines` ; le handler preview (`:201`) fait `let (invoice, _lines) = …` — **cesser de jeter `_lines`** et les passer ; le chemin send appelle déjà `find_by_id_with_lines` et RÉUTILISE les lignes pour la réponse (vérifié Story 20-3b1) — les passer aussi, **aucun re-fetch**. **2 tests unitaires existants à adapter** (`vars_complete_et_formatage_suisse`, `vars_fallbacks_number_et_due_date`, `:576-604`) : `sample_invoice()` sans lignes → ajouter une fixture lignes et recalculer la valeur attendue de `vars["amount"]` (TTC). Pré-formatage suisse inchangé (pas de Fluent ici → pas de piège BiDi).
9. **`due_dates_summary`** (`repositories/invoices.rs:552,554`) : `SUM(i.total_amount)` et le `CASE WHEN` overdue → **forme agrégat AC 4** (table dérivée `lt` jointe, `SUM(COALESCE(lt.ttc, 0))`). Les KPI de l'échéancier (impayées / en retard) deviennent TTC.
10. **Export CSV échéancier** (`invoices.rs:1115` + header :964) : colonne total → TTC (cohérent avec les KPI). Le header i18n `echeancier-csv-header-total` reste (« Total » — pas de mention HT/TTC à changer, vérifier les 4 FTL au cas où).
11. **Items de liste échéancier + API** : `InvoiceListItem` (struct de projection dédiée à cette requête — PAS l'entité `Invoice`) **gagne un champ `total_ttc: Decimal`**, hydraté par la forme scalaire AC 4 ajoutée au SELECT de `list_by_company_paginated` (`repositories/invoices.rs:463-508`) avec `AS total_ttc` ; `InvoiceListItemResponse` (`invoices.rs:235-254`) expose `totalTtc: Decimal` ; `InvoiceResponse` (`:162-188`, `from_parts` a les lignes) expose aussi `totalTtc` (helper Rust). `total_amount` reste exposé inchangé (HT, compat). `DueDateItemResponse` = alias → hérite. **Compromis assumé (documenté)** : `list_by_company_paginated` sert AUSSI la liste factures générale qui n'affiche pas le TTC — chaque page paie la sous-requête corrélée. Accepté : pagination (≤ 100 rows), index `invoice_lines(invoice_id)` existant, et une seule requête à maintenir ; si un profil montrait un coût réel, la variante « colonne seulement pour due-dates » serait une optimisation ultérieure, pas un correctif.
12. **Réconciliation — DÉPLACÉ vers la story 21-2b** (`21-2b-reconciliation-ttc.md`) : matching TTC (`find_unpaid_invoices_for_window`, `amount_score`, câblage production, ~13 fixtures matching.rs, 2 helpers de seeds sans lignes, affichage `invoiceAmount` de l'UI candidats). Décision de split post-validate Pass 4 (règle de splitting préventif — 4 passes sans convergence, friction concentrée ici). 21-2b consomme les primitives AC 1-5 de cette story.
13. **Sites NON modifiés (décisions explicites)** : `invoices.total_amount` persisté (HT, source comptable) ; crédit produit HT (:1061-1064) ; `credit_note.total_amount` persisté (miroir HT) ; export souveraineté `csv_tables.rs:406,425` (dump brut de la colonne) ; `supplier_invoices` (déjà TTC) ; affichages d'audit `reconciliation.rs:1614,3090` (audit log réel sur `bt.amount`, sans rapport HT/TTC facture — vérifié Pass 4 ; ⚠️ `:523` `invoiceAmount` N'EST PAS de l'audit mais l'UI candidats → 21-2b) ; fixture `golden_test.rs` (assertions de déterminisme intra-run, indépendantes de la valeur) ; **réconciliation entière → 21-2b**.

### D. Frontend

14. **Types** (`invoices.types.ts`) : `totalTtc: string` ajouté à `InvoiceResponse` (:26) et `InvoiceListItemResponse` (:80) (`DueDateItem` hérite). Commentaire : montants string décimale, jamais Number (convention :6).
15. **Échéancier** (`due-dates/+page.svelte`) : la colonne par-ligne (:420) passe à `inv.totalTtc` — sinon Σ colonne ≠ KPI TTC du résumé (:307/:313, qui suivent automatiquement le backend). Libellé de colonne `due-dates-column-total` inchangé (« Total »).
16. **Surfaces qui RESTENT HT (décision documentée)** : liste factures (:362), détail facture tfoot (:617) et total temps réel du formulaire (`InvoiceForm.svelte:594`) — sommes des lignes affichées, internement cohérentes ; le total client-side du form est structurellement HT (Σ lignes sans TVA). Le récap TVA d'affichage = #151. Aucun libellé frontend ne prétend « TTC » sur ces surfaces (vérifié).

### E. Tests

17. **e2e kesh-api nouveau fichier** (`invoice_ttc_e2e.rs`, harnais 21-1/kf004) : facture 1 ligne 100.00 @ 8.1 → (a) `InvoiceResponse.totalTtc == 108.10` et `totalAmount == 100.00` ; (b) `due_dates_summary` `unpaidTotal == 108.10` ; (c) export CSV échéancier contient `108.10` ; (d) e-mail : via capture test-mode ou assertion du preview (`GET email-preview` renvoie le corps rendu — asserter que `{amount}` → `108.10`) ; (e) facture multi-lignes taux mixtes → parité avec le débit créance de l'écriture.
18. **PDF/QR** : dans `invoice_pdf_e2e.rs` (seeds déjà @ 7.70), pas d'assertion de montant possible sur les bytes — à la place, test d'intégration au niveau service si praticable OU assertion indirecte : le `qr_data.amount` passe par `generate_qr_bill_pdf` qui refuse un mismatch de devise, pas de montant — couvrir via le test de parité AC 5 + revue. Documenter le choix dans le story file.
19. **Réconciliation → 21-2b** (tests de match TTC et iso-comportement seeds inclus). Cette story garantit seulement que ses propres changements ne cassent PAS les suites réconciliation existantes (gate workspace).
20. **Commentaire faux pré-existant corrigé** : `invoice_echeancier_e2e.rs:425-427` prétend que `total_amount` inclut la TVA — le corriger en passant (l'assertion elle-même est robuste).
21. **Frontend** : `npm run check` + unit (types) ; pas d'assertion de montants rendus dans les specs existants (vérifié — rien ne casse) ; étendre `invoices_echeancier.spec.ts` d'une assertion : facture 100.00 @ 8.1 seedée → la ligne de l'échéancier affiche `108.10`.

### F. Doc & gate

22. `CHANGELOG.md` `[Non publié]` : entrée `Corrigé` (montants TTC — QR, PDF, e-mail, échéancier ; **refs #246**, PAS « closes » — #246 sera fermé par 21-2b qui livre le dernier morceau ; lien #151 pour le récap TVA). Manuels : différés à 21-8 (fix de justesse, mention CHANGELOG suffit).
23. **Aucune migration** (TTC dérivé, jamais persisté) → compteurs 51/audit P5/export/backup inchangés (vérifier qu'aucun test compteur n'est touché).
24. **Quality gate Test Locally First** complet (backend 4 checks + frontend 4 + E2E ciblés), runner jamais pipé.

## Tasks / Subtasks

- [x] **T1 — Helper kesh-core + DRY avoir** (AC 1-3)
- [x] **T2 — Expression SQL + parité** (AC 4-5) : constantes `pub` kesh-db (2 formes) + **test d'intégration 4-voies** (forme scalaire + forme dérivée assertées indépendamment ≡ helper Rust ≡ débit créance)
- [x] **T3 — Surfaces API** (AC 6-11) : QR + PDF total + {amount} (signature `build_invoice_vars` + lignes, send ET preview, 2 tests unitaires adaptés) + summary + CSV + `totalTtc` sur les 2 responses + colonne SQL liste
- [x] **T4 — Frontend** (AC 14-16) : types + colonne échéancier
- [x] **T5 — Tests** (AC 17-21) : `invoice_ttc_e2e.rs` + commentaire echeancier + spec Playwright échéancier (réconciliation → 21-2b)
- [x] **T6 — Doc-sync + gate** (AC 22-24)

## Dev Notes

### Pièges identifiés (ground-truth 2026-07-12)

- **DC7 inviolable** : additionner des `line_vat_amount` déjà arrondis par ligne ; ne JAMAIS `round(Σ base × taux)`. L'équivalence avec le journal (agrégation par taux) tient par associativité — la documenter dans le helper.
- **Seeds réconciliation sans `invoice_lines`** : le piège central de T4. Une sous-requête TTC sur des factures sans lignes = 0 → tous les matchs morts. La ligne `vat_rate 0, line_total = total_amount` rend le changement invisible aux tests existants. Balayer par grep `INSERT INTO invoices` dans les tests réconciliation (e2e + kesh-db).
- **`{amount}` : deux chemins** — `build_invoice_vars` sert le preview (`GET email-preview`) ET le corps par défaut ; les lignes sont disponibles dans les deux handlers (`find_by_id_with_lines`). Pas de Fluent ici (`format_money` de kesh-i18n formatting) → pas de marques BiDi (leçon 21-1 non applicable).
- **`InvoiceListItem` ne charge pas les lignes** → le TTC des items passe par la colonne SQL calculée (AC 11), PAS par le helper Rust (qui exige les lignes). Les deux voies sont réconciliées par le test de parité AC 5.
- **Premier `ROUND` SQL du projet** — aucune convention interne à copier ; le test de parité est le garde-fou, pas la revue visuelle.
- **kesh-reconciliation est un crate PUR** (pas de sqlx) : le TTC lui est passé par le caller — ne pas y introduire de dépendance DB.
- **Ne PAS toucher `compute_total`** ni la sémantique de `invoices.total_amount` : tout le plan comptable (crédit produit HT, avoirs miroir) en dépend.
- Leçons 21-1 reconduites : workspace série obligatoire (`--test-threads=1` kesh-db), `npm run build` avant re-run Playwright, jamais `runner | grep`, `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, backend E2E contre `kesh_e2e`.

### Patterns à réutiliser (ne PAS réinventer)

- **Patron TTC** : `credit_notes.rs:230-235` (exactement `Σ (line_total + line_vat_amount)`) — devient un simple appel au helper.
- Harnais e2e : `contact_payment_terms_e2e.rs` (21-1) / `kf004_no_op_e2e.rs` (spawn_app + login + `create_seeded_company` avec vat_rates seedés).
- Capture e-mail test-mode : `GET /_test/sent-emails` (20-4) si l'assertion corps est faite au send ; sinon `GET /api/v1/invoices/{id}/email-preview` renvoie le rendu (plus simple, pas de MockMailer).
- `format_money` (apostrophe U+2019) : `invoice_email.rs` l'utilise déjà — `108.10` s'assertera sans séparateur de milliers.

### Hors scope (anti-creep)

- Récap TVA sur le PDF et champs émetteur (#151) ; balance âgée (21-7, consommera l'expression SQL) ; `{totalDue}` des rappels (21-5b, consommera le helper) ; toute persistance du TTC ; renommage/relabel des « Total » HT du frontend (liste/détail/form).

### Project Structure Notes

- `crates/kesh-core/src/accounting/vat.rs` ; `crates/kesh-db/src/repositories/{invoices.rs, reconciliation.rs}` (+ constante SQL partagée) ; `crates/kesh-api/src/routes/{invoice_pdf_service.rs, invoice_email.rs, invoices.rs, credit_notes.rs}` ; `crates/kesh-reconciliation/src/matching.rs` ; `frontend/src/lib/features/invoices/invoices.types.ts` + `routes/(app)/invoices/due-dates/+page.svelte` ; tests : `crates/kesh-api/tests/invoice_ttc_e2e.rs` (nouveau) + seeds réconciliation + `invoices_echeancier.spec.ts`.
- 5 modules (kesh-core, kesh-db, kesh-api, kesh-reconciliation, frontend) — à la limite de la règle de splitting : périmètre volontairement borné (pas de rapport, pas de migration, frontend minimal). Si `validate` dépasse 4 passes → split T4 (réconciliation) en sous-story.

### References

- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md#Décision préalable — bug HT/TTC (#246)]
- [Source: GitHub #246] — inventaire initial ; étendu par la cartographie 2026-07-12 (réconciliation + CSV + items liste).
- [Source: story 21-1 Dev Agent Record] — leçons process reconduites.

## Dev Agent Record

### Agent Model Used

Fable 5 (claude-fable-5) — run 2026-07-12/13.

### Debug Log References

- **Bug de fond découvert et corrigé hors story (#249)** : le golden-path E2E échéancier échouait au mark-paid. Analyse ground-truth : `MarkPaidDialog.svelte` envoyait `paidAt` suffixé `Z`, le backend `MarkPaidRequest.paid_at: Option<NaiveDateTime>` le refusait → 422. Prouvé pré-existant (échoue sur baseline pur, tous changements 21-2a stashés). Corrigé dans un commit dédié `fix(#249)` (frontend, sans `Z`) + de-flake du sélecteur d'onglet « Payées » (regex ancrée, matchait « Im-payées ») et retrait de la re-sélection ContactPicker fragile du test #245.
- Piège tooling récurrent : **backend zombie sur 8181** (anciens `nohup`) répondait avec l'ancien binaire (sans `totalTtc`) → faux négatifs Playwright. Toujours `pkill -9 -f target/debug/kesh-api` + vérifier le port avant un run E2E.
- Sérialisation `Decimal` : les montants JSON portent leur scale (`"100.0000"`, `"108.1000"`) — assertions e2e adaptées (le frontend `formatInvoiceTotal` réduit à 2 décimales à l'affichage).

### Completion Notes List

- **T1** helper `invoice_total_ttc` (kesh-core, 7 tests dont arrondi par ligne DC7) + avoir refactoré DRY dessus.
- **T2** constantes SQL `pub` (forme scalaire corrélée + forme agrégat table dérivée, rationale perf) + **test de parité 4-voies** `invoice_ttc_parity.rs` (helper Rust ≡ SQL scalaire ≡ SQL agrégat ≡ débit créance) — vert.
- **T3** QR + `total` PDF (« Total TTC » dit enfin vrai) + `{amount}` e-mail (signature `build_invoice_vars` + lignes, preview inclus, 2 tests unit adaptés) + `due_dates_summary` (table dérivée) + CSV échéancier + `totalTtc` sur `InvoiceResponse`/`InvoiceListItemResponse` + colonne SQL `list_by_company_paginated`. `total_amount` reste HT (compat). CSV export souveraineté/`csv_tables.rs` NON touché (dump brut).
- **T4** frontend : `totalTtc` sur les 2 types + colonne échéancier (`inv.totalTtc`).
- **T5** `invoice_ttc_e2e.rs` (3 tests : totalTtc détail/liste/summary/CSV + preview {amount} TTC + parité multi-taux avec débit créance) + commentaire faux `invoice_echeancier_e2e.rs` corrigé + spec Playwright échéancier (colonne TTC).
- **T6** CHANGELOG [Non publié] (entrée TTC + entrée #249). Réconciliation NON touchée (→ 21-2b).
- **Gate** : fmt/clippy 0 · workspace série **94 suites / 1772 tests / 0 échec** (dont parity 4-voies + invoice_ttc_e2e) · frontend check 0 / unit **382** / build · **E2E Playwright invoices + échéancier 13/13** (golden-path vert après #249). Attention : reste des surfaces HT (liste/détail/form) = décision documentée (récap TVA = #151). **#246 NON fermé** (dernier morceau = réconciliation 21-2b).

### File List

- crates/kesh-core/src/accounting/vat.rs (helper + tests)
- crates/kesh-db/src/repositories/invoices.rs (constantes SQL, InvoiceListItem.total_ttc, due_dates_summary, list_by_company_paginated, list_for_export)
- crates/kesh-db/tests/invoice_ttc_parity.rs (nouveau)
- crates/kesh-api/src/routes/invoice_pdf_service.rs (QR + total TTC)
- crates/kesh-api/src/routes/invoice_email.rs (build_invoice_vars + lignes + tests)
- crates/kesh-api/src/routes/invoices.rs (InvoiceResponse/ListItem total_ttc, CSV)
- crates/kesh-api/src/routes/credit_notes.rs (DRY helper)
- crates/kesh-api/tests/invoice_ttc_e2e.rs (nouveau)
- crates/kesh-api/tests/invoice_echeancier_e2e.rs (commentaire corrigé)
- frontend/src/lib/features/invoices/invoices.types.ts (totalTtc ×2)
- frontend/src/routes/(app)/invoices/due-dates/+page.svelte (colonne TTC)
- frontend/tests/e2e/invoices_echeancier.spec.ts (test colonne TTC + de-flake onglet)
- CHANGELOG.md
- *(commit séparé #249 : frontend/src/lib/features/invoices/MarkPaidDialog.svelte, frontend/tests/e2e/invoices.spec.ts)*

## Change Log

### Validate Pass 4 (2026-07-12, Sonnet 4.6, contexte frais) → SPLIT

3 findings > LOW (1 HIGH + 2 MEDIUM), tous sur/around la réconciliation : **V4-3 HIGH** `UnpaidInvoiceCandidate` figé `pub(crate)` dans kesh-db mais consommé/destructuré par kesh-api (`routes/reconciliation.rs:452,503-510`) → ne compile pas (même classe que V3-2, récidive) → `pub` (patché dans 21-2b) ; **V4-1 MEDIUM** AC 13 mécaractérisait `reconciliation.rs:523` (`invoiceAmount`, rendu dans l'UI candidats `ReconciliationProposals.svelte:285` à côté du montant de la tx) comme « audit cosmétique » → site à patcher (21-2b), `:1614`/`:3090` restent de l'audit réel non-concerné ; **V4-2 MEDIUM** T2 « 3-voies » périmé après V3-3 → « 4-voies ». **Seuil de la règle de splitting préventif atteint (4 passes sans convergence, trend 4→5→3→3), friction concentrée sur T4-réconciliation → SPLIT appliqué** : cette story devient **21-2a** (primitives + surfaces présentation, AC 12/19 déplacés) ; la réconciliation entière devient **21-2b-reconciliation-ttc.md** (avec les patches V4-1/V4-3 intégrés). Les décisions figées par les 4 passes restent valables dans les deux stories.

### Validate Pass 3 (2026-07-12, Opus 4.8, contexte frais)

3 MEDIUM + 1 LOW, tous patchés : **V3-1** la justification « piège SQL » de V2-1 était FAUSSE (vérifiée à l'exécution sur MariaDB 10.11 : `SUM((corrélée))` accepté et correct) → la décision table dérivée reste pour la **performance** (rationale corrigé AC 4, transmissible à 21-7) ; **V3-2** `pub(crate)` inaccessible à `kesh-report` (crate séparé, consommateur désigné 21-7) → constantes **`pub`** ; **V3-3** le « ou » d'AC 5 laissait une des deux formes sans test de parité → les DEUX formes assertées indépendamment (4 voies, 1 valeur) ; **V3-4** `#[sqlx(flatten)]` = premier usage workspace non signalé → flaggé + alternative précédentée documentée-rejetée (duplication ~20 champs). Vérifiés OK : 3-tuple = extension minimale (aucune struct candidat existante), table dérivée compatible QueryBuilder, preview rend bien `{amount}` (défauts 4 langues), send garde ses lignes. Trend > LOW : 4 → 5 → 3 → à confirmer Pass 4 (rotation Sonnet — ⚠️ seuil règle de splitting à 4 passes sans convergence).

### Validate Pass 2 (2026-07-12, Haiku 4.5, contexte frais)

6 findings d'ambiguïté de design, tous tranchés dans la spec (0 hallucination, refs Pass 1 toutes confirmées, « aucune régression introduite par Pass 1 ») : **V2-1** (classé CRITICAL par la passe, retenu MEDIUM) sous-requête corrélée dans un `SUM()` externe = piège SQL → AC 4 scindé en **2 formes** (scalaire par facture / table dérivée jointe pour l'agrégat) + AC 9 précisé ; **V2-2** type du champ liste → `InvoiceListItem.total_ttc` (struct de projection, pas l'entité) ; **V2-3** signature `propose_matches` FIGÉE : tuple étendu `(Invoice, Option<Contact>, Decimal)` ; **V2-4** retour `find_unpaid_invoices_for_window` FIGÉ : wrapper `UnpaidInvoiceCandidate` `#[sqlx(flatten)]` ; **V2-5** intégration QueryBuilder → pseudo-code ajouté AC 4 ; **V2-6** chemin send → lignes déjà chargées et réutilisées (20-3b1), aucun re-fetch. + note legacy 0 ligne (COALESCE → TTC 0 assumé). Trend > LOW : 4 → 5 (ambiguïtés nouvelles issues du niveau de détail ajouté en Pass 1) → à confirmer Pass 3 (rotation Opus).

### Validate Pass 1 (2026-07-12, Sonnet 4.6, contexte frais)

5 findings, tous patchés : **V1-1 CRITICAL** fanout du câblage réconciliation non listé — 2 sites PRODUCTION (`routes/reconciliation.rs:503-510` propose_matches + `:1120-1124` re-score accept_one, risque de comparaison silencieuse au HT) + ~13 fixtures internes `matching.rs:268-436` + `reconciliation_repository.rs:154-181` (`insert_test_invoice` ×9) → AC 12 étendu + T4 ; **V1-2 MEDIUM** signature `build_invoice_vars` sans lignes + preview qui jette `_lines` + 2 tests unitaires à adapter → AC 8 ; **V1-3 MEDIUM** `find_unpaid_invoices_for_window` fait `FROM invoices` sans alias `i` → note alias AC 12 ; **V1-4 MEDIUM** sous-requête corrélée payée par la liste factures générale → compromis documenté AC 11 ; **V1-5 LOW** plage `compute_total` :287-292. Claims techniques vérifiées justes par la passe (ROUND MariaDB half-away-from-zero sur DECIMAL, arithmétique DECIMAL exacte, associativité, manual/split/rules sans seeds propres). Trend > LOW : 4 → à confirmer Pass 2.
