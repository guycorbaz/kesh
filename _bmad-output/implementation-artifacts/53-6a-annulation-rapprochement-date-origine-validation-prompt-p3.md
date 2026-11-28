# Prompt — validation P3 de la fiche 53-6a « Annuler un rapprochement contre-passe dans l'exercice de l'écriture, à sa date » (#1674, P1), passe complète

Tu valides la **fiche** de la story **53-6a** de Kesh (comptabilité suisse en production ; Rust/Axum, MariaDB, SvelteKit), Epic 53, release v0.21.0.
Troisième passe : **complète**, contexte frais, modèle **Opus** (P1 : Opus 0/1/1/10 ; P2 : Sonnet 0/0/1/6, tout remédié). Langue : français.
Worktree `/home/gcorbaz/devel/kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`. Fiche :
`_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md`. Rapports : `/home/gcorbaz/devel/kesh-gate-logs/e53/53-6a-validation-p1.md`
et `…-p2.md` (**à lire** ; recontrôle leurs remèdes au code). Code de référence : **`origin/main` à `0e99cd2db`**, lu par `git show origin/main:<chemin>`.
Q1 à Q6 sont tranchées (Project Lead, 2026-11-27) : tu contrôles leur application, tu ne les rouvres pas.

**Points les plus importants** (au code, ligne copiée ; faux = HIGH au moins) :
1. **M1 de la P2** : le rang 2 bis (`DocumentLetteringInClosedPeriods`, `settlement_cancellation.rs`, après le rang 2) précède le rang `PeriodLocked` ; pour un règlement
   **entier** la queue rend le lettrage figé. Est-ce exact pour R2 (règlement client), R3 (fournisseur), R4, R5 — quand un groupe de lettrage `document` est-il créé,
   et dissous (`letterings.rs`, `dissolve_*_document_group_in_tx`) ? L'état à monter pour AC-9 (règlement partiel, groupe dont une ligne est postérieure à la borne, état
   hérité) est-il réalisable, et le test de précédence annoncé tient-il ? Le texte de D5 (« c'est déjà le cas aujourd'hui »), l'AC-9, M6 et les notes de version
   disent-ils vrai ? Le motif visible à l'écran pour un règlement entier sous la borne (lettrage figé, texte existant) reste-t-il juste après la story (la date de
   l'inverse est celle de l'origine, déjà sous la borne) ?
2. Contrat du clic H1 (P1) : `409 PERIOD_LOCKED` sans `details` pour R4 et R5 (`grouped_settlements.rs:1812-1816`, `:1988-1992`, `kesh-api/src/errors.rs:6270-6273`),
   `400` du socle pour les autres bras ; cohérence de D5, AC-9, M6, AC-14, notes de version, commentaires des trois `match`, frontend `reconciliationCancelErrorMessage`.
   Le LOW L1 de la P2 (commentaire des familles `SettlementNotCancellable` / `SupplierInvoiceNotCancellable`, deux raisons) est-il exact ?
3. Les sites ajoutés à AC-14 par la P2 (`user-manual.tex:812-819`, `api-external.md:1635`) et par la P1 : cherche, du symptôme (« date du jour », « datée du jour »,
   « aujourd'hui », « verrou », « contre-pass », « Annuler le rapprochement »), ce qu'AC-14 omet encore — **`.tex` et PDF aplatis** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`
   sur `docs/manual/fr/{user,admin}-manual.pdf` de `origin/main`, extraits sous `/tmp/claude-1000/`), plus `docs/api-external.md`.
4. Les tests qui changent d'attendu ou perdent leur objet (`grouped_settlements.rs`, `group_unreconciliation_refuses_without_an_open_year_today` ; audits `:2896-2908` et
   `grouped_settlements_supplier.rs:2220-2227` ; `match` exhaustifs de `invoice_settlement.rs` et `supplier_invoices_repository.rs`) : inventaire complet ? Recompte
   par la commande de la fiche, cherche ce qu'elle omet.

## À lire d'abord
`CLAUDE.md` du worktree (§ *Ce qu'impose la production*, *Review Iteration Rule* dont *Inventorier les sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*,
*Une passe déclare ses axes*, *Propagation post-patch*, *Priorités des défauts*, *Versions de l'API*, *Recette visuelle*, *Règle de splitting préventif*) ;
`docs/agents-orchestration.md` § 2, 2.1, 3 ; l'epic `_bmad-output/planning-artifacts/epic-53-ecritures-justes.md` ; le code de la fiche (`journal_entries.rs`,
`reconciliation_cancel.rs`, `settlement_cancellation.rs`, `invoice_settlements_write.rs`, `supplier_invoices.rs`, `grouped_settlements.rs`, `letterings.rs`,
`fiscal_years.rs`, `companies.rs`, les deux `errors.rs`, `routes/reconciliation.rs`, `frontend/src/lib/features/reconciliation/`). L'issue #1674 : **au plus une requête
`gh` en lecture** dans la passe, ou aucune.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)
A1 prémisses P1 à P11 relues ; A2 écarts `main` / branche d'epic ; A3 inventaire clos R1 à R11 ; A4 datation D1 et D2 (cinq bras, devises, audit) ; A5 verrous et
concurrence (D3, AC-12) ; A6 queue commune, ordre des rangs (dont 2 bis avant `PeriodLocked`), écran, i18n quatre locales ; A7 restes de la détection (53-6b) et
renvois d'AC ; A8 décisions Q1 à Q6 ; A9 manuel PDF compris et guide de l'API ; A10 AC constatables, tests qui passeraient à vide, mutations M1 à M8, M10, M11 (chacune tue-t-elle
le test annoncé sur l'état monté ?) ; A11 périmètre (quatre modules), borne de six stories, notes de version, aucune donnée réelle. Dis si la fiche **recycle** (un MEDIUM ou plus
qui revient sous une autre forme ou naît du correctif de la P2).

## Ce que tu rends
Rapport français : par finding, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, `chemin:ligne` et ligne copiée, ce qui est faux, la correction ; un finding CRITICAL ou HIGH qui
affirme une **absence** se prouve par `grep -nF` cité. Puis axes **exercés et non exercés**, puis décompte C/H/M/L.

## Interdits
Aucune écriture (ni worktree, ni dépôt principal, ni `~/kesh/`) ; pas de `git commit`, `push`, `checkout`, `merge`, `stash` ; pas de `gh` en écriture ; ni `cargo`, `npm`,
`scripts/test-fast.sh`, `gate-ci.sh`, `prepare-release.sh`, `recette-montee.sh`, `mem-guard.sh`, ni script qui écrit ; ni `docker`, `sqlx migrate`, `mariadb` ; jamais la base du NAS ;
jamais `pkill`, `killall`, ni `kill` par motif. Fichiers temporaires sous `/tmp/claude-1000/` seulement.
