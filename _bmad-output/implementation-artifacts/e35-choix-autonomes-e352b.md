# Epic 35 — choix autonomes de la story 35-2b (clé `e352b`)

Registre tenu par l'agent de développement de la 35-2b (Opus 5.5, worktree `kesh-e352b`), le 2026-11-13. Mise à jour
par union, jamais par écrasement.

### C-e352b-1 — Une seule fonction construit le refus du lot

- **Contexte** : D1 fait rendre `failed_with("INVOICE_NOT_PDF_READY", json!({ "message": … }))` par trois bras de
  `classify_render_error`.
- **Retenu** : `refus_pdf_au_lot(message: String) -> BatchItemError` (`invoice_email.rs`), appelée par les trois bras ;
  le code et la forme de `details` s'écrivent une fois (DRY). Les mutations M3, M5 et M5bis se jouent bras par bras, en
  remplaçant l'appel par `BatchItemError::failed("INVOICE_NOT_PDF_READY")`.
- **Écartée** : trois `failed_with` écrits en ligne, qui recopieraient le code et la forme.
- **Réversible** : oui.

### C-e352b-2 — L'écran affiche le message sans ses blancs de bord

- **Contexte** : D2.2 lit le message s'il n'est pas vide après `trim()`, sans dire si l'écran l'affiche rogné.
- **Retenu** : `reminderFailureText` affiche `message.trim()` : une phrase qui finirait par un blanc ne laisse pas
  d'espace après elle. Comportement visible ; prudent (le texte n'est pas changé, seuls ses bords) et réversible.
- **Écartée** : afficher la chaîne brute.
- **Réversible** : oui.

### C-e352b-3 — L'en-tête qui déborde, prouvé au lot sous catalogue réel

- **Contexte** : angle mort de la fiche (V2-5 c) — « si le montage de `invoice_pdf_e2e.rs:485-544` se réemploie sans
  frais pour un lot, l'ajouter ».
- **Retenu** : il se réemploie (deux `UPDATE`) ; test HTTP `send_reminder_batch_entete_qui_deborde_porte_le_message`,
  dans une société à part (le débordement touche toutes les factures de la société, il ne se mêle pas au lot d'AC1).
  L'angle mort se réduit à `InvoiceTooManyLinesForPdf`, dont la phrase n'est prouvée au lot que par le repli (AC3).
- **Écartée** : laisser l'angle mort tel quel.
- **Réversible** : oui.

### C-e352b-4 — La spec E2E dans un fichier à part

- **Contexte** : AC5 se fait « sur le modèle de `dunning-roundtrip.spec.ts:56-91` », sans dire où.
- **Retenu** : `frontend/tests/e2e/reminder-batch-refus-pdf.spec.ts`, un test seul : le parcours de bout en bout de
  `dunning-roundtrip` reste lisible, et un échec de l'une ne masque pas l'autre.
- **Écartée** : un pas de plus dans `dunning-roundtrip.spec.ts`.
- **Réversible** : oui.

### C-e352b-5 — `seed_sendable_ve_vey` rend `admin_id`

- **Contexte** : AC1 permet de faire rendre `admin_id` à cette fonction de la 35-2a.
- **Retenu** : elle rend `(admin_id, company_id, invoice_id)`, comme `seed_sendable` ; ses deux appelants de la 35-2a
  suivent (`let (_, _, invoice_id)`, `let (_, company_id, invoice_id)`). Le `UPDATE` de la localité n'est pas recopié.
- **Réversible** : oui.
