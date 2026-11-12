# Story 35-2b : le rapport d'un lot de rappels dit pourquoi le PDF d'une facture est refusé (#955)

Status: ready-for-dev

*Fiche née le 2026-11-12 du découpage de la 35-2 à la validation P2 (C-e350-9), par l'agent de kickoff de l'Epic 35
(Opus 5.5), worktree `kesh-e35`. Elle **part de la 35-2a fusionnée**, dont elle reprend le message localisé (la 35-2a garde
`AppError::InvoiceNotPdfReady(String)`, D3 ; aucun contrat `details` lisible par machine, C-e350-11). Release **0.17.7** ; Guy n'emploie pas encore les rappels (première facture client avant le
2026-11-30) : elle suit la 35-2a sans échéance propre. **Aucune migration.** Code lu sur **`release/0.17.6`** (`4e1c768a`).
Registre de la story : `e35-choix-autonomes-e352b.md`, à créer au premier choix.*

## Story

En tant que titulaire qui envoie un lot de rappels, quand une facture du lot est écartée parce que son PDF est refusé, je
veux lire dans le rapport **ce qu'il faut corriger**, comme sur la fiche de la facture, et non « Facture non imprimable en
PDF » seul.

## Ce qui se lit au code (`release/0.17.6`)

- La boucle du lot rend chaque PDF (`crates/kesh-api/src/routes/invoice_email.rs:1260-1269`) et classe l'échec par
  `classify_render_error` (`:1023-1036`) : `InvoiceNotPdfReady`, `InvoiceTooManyLinesForPdf` et
  `InvoicePdfHeaderOverflow` deviennent tous `BatchItemError::failed("INVOICE_NOT_PDF_READY")`, **sans `details`**, alors
  que `BatchItemError::failed_with(code, details)` existe (`:964-990`) et que `FailedReminder` porte un champ `details`
  (`:944-948`). Le bras final rend `DATABASE_ERROR` (règle du doc-comment `:1010-1022`) ; test
  `classify_render_error_ne_deguise_pas_un_refus_en_panne` (`:1749`).
- L'écran : `ReminderBatchReport.svelte:30` affiche `reminderErrorLabel(f.errorCode)` seul, « Facture non imprimable en
  PDF » (`reminder-error-label.ts:20`). Le composant n'a pas de test.
- Le manuel (`user-manual.tex:1251`) dit que le lot affiche ce libellé seul (issue #955) ; la 35-2a réécrit la phrase du
  téléchargement, celle-ci la complète.

## Décisions

- **D1 — Le message, seul.** `classify_render_error` rend, pour `AppError::InvoiceNotPdfReady(msg)`,
  `BatchItemError::failed_with("INVOICE_NOT_PDF_READY", json!({ "message": msg }))` : `details.message` est une phrase
  **destinée à l'utilisateur**, non un contrat lisible par machine (C-e350-11). Ceux nés de `InvoiceTooManyLinesForPdf` et
  `InvoicePdfHeaderOverflow`, qui partagent le code, restent sans `details`.
- **D2 — L'écran** affiche « Facture non imprimable en PDF — *message* » quand `details.message` est là, le libellé seul
  sinon. Le séparateur suit le patron existant du rapport ; aucune clé frontend neuve sauf nécessité (`lint-i18n-ownership`).

## Critères d'acceptation

1. **AC1 — Par HTTP** (`crates/kesh-api/tests/invoice_send_email_e2e.rs`, sur le modèle de
   `send_reminder_batch_partial_success`, `:2019`) : un lot de deux factures, l'une au client de localité `Ve–vey`, l'autre
   au client **sans adresse** : deux `failed[]` en `INVOICE_NOT_PDF_READY`, chacun avec `details.message` — le premier
   contient « localité du client » et « U+2013 », le second la phrase de `invoice-pdf-error-client-address-required` —,
   et rien d'autre dans `details`. Rouge sur le code de départ (`details` à `null`).
2. **AC2 — Pas de fausse panne** : le refus reste `INVOICE_NOT_PDF_READY`, jamais `DATABASE_ERROR` ; le test `:1749`
   asserte en plus `details.message` (frontière : la 35-2a ne change ni la variante ni ce test, D3 de la 35-2a).
3. **AC3 — Les deux autres codes** : un échec `InvoiceTooManyLinesForPdf` reste sans `details`.
4. **AC4 — Écran** : Vitest de `ReminderBatchReport.svelte`, avec et sans `details.message`.
5. **AC5 — Documentation** : `docs/api-external.md`, la ligne de la 35-2a dit que, dans `failed[]` du lot, `details` porte
   en plus `message` ; manuel `:1251` complété, PDF régénéré, contrôlé aplati ; CHANGELOG `[0.17.7]`, *Corrigé*, refs #955
   (règle C-e350-8 : un point, après celui de la 35-2a). La PR porte `closes #955` (la 35-2a, `refs #955`).
6. **AC6 — Capture** : la ligne du rapport d'un lot (thème clair, base jetable, `kesh-gate-logs/e352b/captures/`).

**Mutations** : M1 `details.message` omis pour un refus sans champ ; M2 `details.message` non lu par le composant ; M3 le
bras `InvoiceNotPdfReady` de `classify_render_error` rendu à `failed` sans `details`.

## Tâches

- [ ] Étape 0 : partir de la 35-2a fusionnée ; relire les lignes citées.
- [ ] Tests, rouges consignés ; serveur ; composant ; documentation ; capture.
- [ ] Gates : `test-fast.sh --ci` et `scripts/gate-ci.sh`, frontend complet, E2E complet au dernier commit de code.

## Dev Notes

- Fichiers : `invoice_email.rs`, `ReminderBatchReport.svelte` (+ test), `api-external.md`, `user-manual.tex`,
  `CHANGELOG.md` ; communs avec la 35-2a, qui la précède.
- Fixtures synthétiques seulement. Pattern batch du `CLAUDE.md` : aucune erreur per-facture n'escalade.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-12 — Née du découpage de la 35-2 à la validation P2 (C-e350-9). Validation à lancer.
- 2026-11-12 — Réajustée au recadrage de la 35-2a (validation P3, C-e350-11) : `details` du lot réduit à `message`,
  phrase destinée à l'utilisateur ; aucune `reason` ni `field` ; frontière avec la 35-2a écrite (AC2) ; `closes #955` ici.
