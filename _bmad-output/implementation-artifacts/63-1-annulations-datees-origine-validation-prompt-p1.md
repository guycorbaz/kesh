# Prompt — validation P1 de la fiche 63-1 « Les annulations depuis la fiche contre-passent dans l'exercice de l'écriture, à sa date » (#1752, #1753, #1754, P1), passe complète

Tu valides la **fiche** de la story **63-1** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), seule story de l'Epic 63
« Annulations datées de l'origine », **story de correction de la release v0.21.0** (vendredi 2026-12-04). Kesh est **en production** : le
Project Lead y tient ses livres réels ; une annulation depuis la fiche contre-passe aujourd'hui à la date du jour, et faite après la fin de
l'exercice de l'écriture, elle fausse deux exercices (bilan ; pour un solde ou une facture fournisseur, résultat et TVA). **Aucune migration**
annoncée. Première passe : **complète**, contexte frais, modèle **Sonnet** (la fiche est d'Opus). Langue du rapport : français.

Worktree `/home/gcorbaz/devel/kesh-63-1`, branche `story/63-1-annulations-datees-origine`, partie de
`story/53-6-annulation-rapprochement-exercice` à `6b71d4c21` (la 53-6a, en développement, qui pose le mécanisme réemployé). Fiche :
`_bmad-output/implementation-artifacts/63-1-annulations-datees-origine.md`. Kickoff : `_bmad-output/planning-artifacts/epic-63-annulations-datees-origine.md`.
La fiche **dépend** de la 53-6a (`_bmad-output/implementation-artifacts/53-6a-annulation-rapprochement-date-origine.md`, à lire : D2, D5, D6,
AC-11, inventaire R1 à R11, décisions **Q1 à Q6 du Project Lead du 2026-11-27**). Les décisions Q1 à Q6 **ne se rouvrent pas** : tu contrôles
que la 63-1 les applique aux trois gestes sans contradiction. Les questions Q-A à Q-D de la 63-1 sont **au Project Lead** : tu peux juger
leur prémisse et leur recommandation, non les trancher. Le code de référence est celui de la **branche** (`6b71d4c21`), où le mécanisme de la
53-6a existe ; lis aussi `origin/main` (`0e99cd2db`) quand la fiche s'y réfère.

**Les points les plus importants** — vérifie-les au code, ligne à ligne, avant tout le reste ; si l'un est faux, c'est un HIGH au moins :
1. **D1/D2** : passer `CancellationDating::OriginDate` aux trois gestes **et** à chacun de leurs lecteurs de motifs (`services::factures::reglements`,
   `supplier_invoices::get_settlement_view_in_tx` — deux bloqueurs —, `supplier_invoice_cancel_blocker`) suffit-il ? Un lecteur ou un appelant
   oublié (MCP, `commit_with_view` et ses neuf gestes, `imported_supplier_invoices.rs`, l'export, un test qui fige la forme) laisserait-il un
   écran dire autre chose que le clic ?
2. **D4/P6** : au clic, `PeriodLocked` est-il bien refusé par le socle en `400 PERIOD_LOCKED` (`details.lockedThrough`, `details.attempted`)
   pour les trois gestes, leurs listes fermées de refus ne l'admettant pas ? Les états de production qui atteignent `PERIOD_LOCKED` plutôt que
   le rang 2 bis (lettrage figé), geste par geste (règlement, solde, paiement fournisseur, facture fournisseur), sont-ils justes ? Vérifie au code
   si `POST /supplier-invoices/{id}/pay` accepte un paiement antérieur à la date de la facture.
3. **P10/AC-10** : l'inventaire est-il **clos** — après la story, `CancellationDating::Today` n'est-il produit en production qu'à
   `journal_entries::reverse` (R10) ? Aucune autre annulation (avoir, lot de paiement, repassage en brouillon, solde groupé, MCP) ne contre-passe-t-elle
   au jour sans être nommée ?

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Ce qu'impose la production*, *Qualité du code*, *Review Iteration Rule* (dont *Inventorier les sites NON
   RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*, *Propagation post-patch*, *Recompter*),
   *Priorités des défauts*, *Migration breaking policy* (pour constater qu'elle ne s'applique pas), *Règle de splitting préventif*, *Versions
   de l'API*, *Recette visuelle*, *Test Locally First*, *Pattern batch* (pour constater qu'il ne s'applique pas).
2. `docs/agents-orchestration.md` (§ 2 et 2.1 : interdits ; § 3 : processus).
3. La fiche 53-6a (ci-dessus) et la fiche 53-6b (`53-6b-detection-contre-passations-autre-exercice.md`, P10 : pour Q-B).
4. Le code, à la branche : `crates/kesh-db/src/repositories/journal_entries.rs` (`CancellationDating`, `ReversalDating`,
   `reversal_date_verdict`, `reverse_in_tx`, `reverse_owned_in_tx`, `reverse`, `reverse_in_tx_inner` étapes 1 et 4, `create_in_tx_inner`
   étape 1-bis), `settlement_cancellation.rs` (en entier), `invoice_settlements_write.rs` (`write_off_invoice`, `settlement_cancel_blocker*`,
   `settlement_cancel_head`, `cancel_settlement_in_tx`, `cancel_settlement`), `supplier_invoices.rs` (`supplier_invoice_cancel_blocker`,
   `cancel_in_tx`, `cancel`, `supplier_settlement_cancel_blocker*`, `cancel_settlement_in_tx`, `cancel_settlement`, `get_settlement_view*`,
   `commit_with_view`, `unvalidate_in_tx_with_policy`, la validation et le paiement — dates des écritures), `letterings.rs` (dissolutions,
   `document_group_frozen_by_periods`), `crates/kesh-db/src/errors.rs` (`SettlementCancelBlocker`) ; `crates/kesh-api/src/routes/invoices.rs`,
   `routes/supplier_invoices.rs`, `routes/imported_supplier_invoices.rs`, `services/factures.rs`, `services/factures_fournisseur.rs`,
   `services/mod.rs`, `errors.rs` (familles `SettlementNotCancellable`, `SupplierInvoiceNotCancellable`, `supplier_invoice_cancel_blocked_text`,
   `reconciliation_cancel_blocked_text`) ; `crates/kesh-report` (décompte de TVA : sur quoi il se fonde, pour P8) ;
   `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts`, `frontend/src/lib/features/invoices/` (`settlement-cancel.ts`,
   `InvoiceSettlements.svelte`), `frontend/src/lib/features/supplier-invoices/` (`invoice-cancel.ts`, `settlement-cancel.ts`), les pages
   `frontend/src/routes/(app)/invoices/[id]/+page.svelte` et `supplier-invoices/[id]/+page.svelte`.
5. Les issues #1752, #1753, #1754, **en lecture seule**, **au plus trois requêtes `gh` dans toute la passe, espacées d'au moins une minute** :
   `gh issue view <N> --json number,title,labels,body`. Leurs textes source sont aussi dans `/home/gcorbaz/devel/kesh-gate-logs/e53/53-6-issues-q3/`.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — prémisses P1 à P10 relues au code** : chaque `chemin:ligne` cité est-il juste à `6b71d4c21` (copie la ligne) ? P4 (la date du jour n'a
  plus d'autre usage dans la chaîne de vue fournisseur, ni dans `services::factures::reglements` ; les routes `imported_supplier_invoices.rs`
  s'en servent-elles ailleurs ?) ; P8 (le décompte de TVA se fonde-t-il sur les pièces, et la reprise d'un solde annulé tombe-t-elle bien, après
  la story, dans la période du solde ?) ; P9 (le repassage en brouillon d'une annulée).
- **A2 — inventaire clos (P10)** : rejoue la commande de la fiche et cherche ce qu'elle **omet** ; chaque site est-il changé, inchangé avec
  raison, ou angle mort assumé ? La garde lexicale d'AC-10 est-elle constructible (motif d'un inventaire existant) et a-t-elle une sonde
  positive ?
- **A3 — datation et lettrage (D1, D6)** : pour chacun des trois gestes et pour le solde, la date de l'origine est-elle bien celle que la fiche
  dit (règlement : `settled_on` ; solde ; paiement : `payment_date` ; achat : `invoice_date` ou autre — lis `supplier_invoices.rs:1831`) ? Le
  lettrage (dissolution du groupe `document`, puis `reversal` sous `Mode::System`) reste-t-il juste quand l'exercice tenu et celui de l'inverse
  sont le même ? Un règlement soldé avec écart, un règlement en devise, un règlement par virement vers un compte clos changent-ils quelque chose ?
- **A4 — verrous et concurrence (AC-12)** : reconstitue l'ordre des verrous de l'annulation de facture fournisseur, avant et après ; un cycle
  nouveau apparaît-il (`close`, `reopen`, `lock_books`, un repassage en brouillon concurrent, un paiement par lot) ? L'entrelacement d'AC-12
  est-il jouable sans `sleep` ?
- **A5 — refus et précédence (D4)** : la place de `PeriodLocked` après le rang 2 bis, geste par geste ; `FISCAL_YEAR_INVALID` disparaît-il
  vraiment de ces trois routes ; `LATER_FISCAL_YEAR_CLOSED` ne reste-t-il que dans l'état hérité ; les tests existants qui figent l'ancien
  contrat sont-ils tous repérés (recompte la commande des Dev Notes, avec son périmètre) ?
- **A6 — textes et écran (D5, D7)** : les deux clés neuves et les quatre confirmations disent-elles vrai, aux quatre locales ? La propriété
  des clés (`lint-i18n-ownership`) est-elle respectée (clé partagée dans `lib/shared`) ? Les replis Rust et TS sont-ils identiques au FTL fr-CH ?
  Le refus du clic (`400 PERIOD_LOCKED`) s'affiche-t-il lisiblement sur les deux fiches ? Le choix de textes **sans variable** est-il défendable
  (la date est-elle vraiment visible à côté du bouton sur les deux fiches ?) ? Le maintien de `NoOpenFiscalYearToday` (D7) laisse-t-il un
  `match`, une clé ou un test qui ment ?
- **A7 — AC constatables et tests** : chaque AC dit-il un test, une valeur attendue, un fichier ? Un test passerait-il à vide (horloge égale à
  l'origine ; `not.toContain` d'un ancien texte) ? AC-11 (inversion de l'AC-11 de la 53-6a) nomme-t-il tous les tests qui figeaient « datée du
  jour » pour ces gestes ? Les mutations M1 à M8 tuent-elles chacune le test annoncé, et seulement lui ? Une mutation manque-t-elle (par
  exemple : un seul des deux bloqueurs de la vue fournisseur resté sous `Today`) ?
- **A8 — manuel, PDF compris** : `docs/manual/fr/user-manual.tex` (encadré « Ce que le verrou n'empêche pas » `:854-875`, `sec:date-du-jour`
  `:895`, `:1152`, § *Enregistrer et annuler un règlement* `:1964-2000`, règlement fournisseur `:2618` et suivantes, encadré `:2489-2494`,
  § *Annuler une facture fournisseur* `:2688-2750`, décompte de TVA `:4736-4752`) et `docs/manual/fr/admin-manual.tex` (`:3556`) ; cherche dans
  les `.tex` **et** dans les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o -i '.\{120\}datée du jour.\{120\}'`,
  puis `date du jour`, `aujourd'hui`, `du jour de l'annulation`, `Annuler le règlement`, `Annuler la facture`, `Annuler le solde` ; idem pour
  `admin-manual.pdf`) toute phrase que la story rend fausse et que l'AC-14 ne nomme pas ; et `docs/api-external.md` (`:686-701`, `:799-819`,
  table générale des codes d'erreur). Les manuels `de`, `en`, `it` existent-ils ?
- **A9 — décisions et questions** : Q1, Q2, Q5, Q6 sont-elles appliquées partout (AC, tâches, notes de version) ? Les prémisses de Q-A
  (cas courant du refus pour une facture fournisseur d'un trimestre verrouillé), Q-B (la 53-6b ne lit que `reconciliation.cancelled` ; les
  trois gestes ont-ils bien `reversalJournalEntryId` dans leur audit ?), Q-C et Q-D sont-elles justes ?
- **A10 — périmètre, splitting, API, priorité** : le décompte des modules (cinq) est-il juste au sens de CLAUDE.md (un domaine du frontend =
  `frontend/src/lib/features/<domaine>` ; `lib/shared` et les pages `routes/` ?) ? Le découpage de repli (63-1a / 63-1b) est-il sans cycle ?
  Les notes de version disent-elles tout ce qui change pour un utilisateur et pour une intégration (trois routes, valeur `PERIOD_LOCKED` des
  vues, `dating` de l'audit) ? Aucune donnée réelle n'apparaît-elle dans la fiche (montants, IBAN, numéros d'écriture des livres de production) ?

## Ce que tu rends

Un rapport en français, écrit dans `/home/gcorbaz/devel/kesh-gate-logs/e63/63-1-validation-p1.md` (seul fichier que tu as le droit d'écrire ;
crée le dossier `e63` s'il manque) : pour chaque finding, **sévérité** (CRITICAL, HIGH, MEDIUM, LOW), axe, `chemin:ligne` et ligne copiée, ce
qui est faux, la correction proposée. Un finding CRITICAL ou HIGH qui affirme l'**absence** d'un code se prouve par un `grep -nF` cité. Puis la
liste des axes **exercés** et **non exercés** (avec la raison) ; un « 0 finding » sans cette liste ne compte pas. Puis le décompte `C/H/M/L`.

## Interdits

- **Aucune écriture** hors du rapport ci-dessus : ni dans le worktree, ni dans le dépôt principal (`/home/gcorbaz/devel/kesh`, lecture seule),
  ni sous `~/kesh/`. Pas de `git commit`, `git push`, `git checkout` d'une autre branche, `git merge`, `git rebase`, `git stash`, `git reset`.
- Ni `gh` en écriture (`gh issue create|edit|comment|close`, `gh pr …`, `gh api` en `POST`/`PATCH`), ni plus de trois `gh` en lecture.
- Ni `cargo`, ni `npm`, ni `npx`, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`, `scripts/recette-montee.sh`,
  `scripts/mem-guard.sh`, ni aucun script qui écrit ; ni `docker`, ni `docker compose`, ni `sqlx migrate`, ni `mariadb` ; **jamais la base du NAS**.
- Jamais `pkill`, `killall`, ni `kill` par motif.
- Fichiers temporaires : seulement sous ton répertoire temporaire de session.
