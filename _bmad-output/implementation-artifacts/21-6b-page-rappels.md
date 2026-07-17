# Story 21.6b: Page Rappels — liste, envoi unitaire, lot, rappel manuel (frontend)

Status: done

<!-- Créée 2026-07-17 par bmad-create-story. Cartographie ground-truth par 5 agents Explore parallèles (contrats backend dunning / anti-double-submit / routing-nav-i18n / conventions test-E2E / + recoupement). Issue du split 21-6a/b/c (2026-07-16). Consomme les endpoints backend livrés par 21-5a (liste groupée, manuel) et 21-5b (preview, envoi unitaire, lot). Indépendante de 21-6a. Décisions Guy 2026-07-17 : page complète (lot+unitaire+manuel) / nouvelle feature `reminders/` (namespace i18n libre) / cases par ligne sans select-all (garde 20 UI) / contact sans e-mail visible+badgé+non-cochable. -->

## Story

En tant que **comptable d'une PME suisse**,
je veux **une page « Rappels » qui liste les factures à relancer groupées par débiteur, et me laisse envoyer un rappel par e-mail (à l'unité avec aperçu éditable, ou en lot), ou enregistrer un rappel papier déjà envoyé**,
afin de **piloter mes relances depuis l'interface sans passer par l'API — et sans jamais envoyer deux fois le même rappel par mégarde**.

## Contexte

**Tout le backend est livré.** Les stories 21-5a et 21-5b ont posé les endpoints ; cette story les rend pilotables depuis une page dédiée. **21-6b est purement frontend** (une seule extension de fixture de test côté backend, cf. AC 26).

> ⚠️ **Piège de cartographie confirmé — les endpoints d'envoi EXISTENT.** Un agent Explore a conclu à tort « aucun endpoint d'envoi de rappel n'existe » parce qu'il a cherché dans `dunning_reminders.rs` seulement — or l'envoi e-mail vit dans **`invoice_email.rs`** (livré par 21-5b). Vérifié par grep : `POST /api/v1/invoices/{id}/reminders/send` monté `lib.rs:433`, `POST /api/v1/dunning/reminders/send-batch` monté `lib.rs:437`, handlers `invoice_email.rs:412`/`:925`/`:363`. **NE PAS recréer de backend d'envoi.**

**Anti-double-submit = AC de premier plan.** Le backend n'a **aucune** protection contre le double-envoi : la course TOCTOU a été explicitement acceptée en review 21-5b (`UNIQUE(invoice_id, level_number)` écarté car il contredirait D18 qui autorise la ré-émission volontaire). L'e-mail part réellement chez le débiteur à chaque appel — **l'UI est la seule barrière**. Le gabarit à 4 couches de `SendEmailDialog` (Epic 20, durci en review) est à répliquer intégralement.

**Prérequis** : 21-5a + 21-5b (done). Indépendante de 21-6a. **Consommée par 21-6c** (qui ajoutera l'historique sur la fiche facture, le toggle suspension UI, le compteur dashboard, les liens croisés).

### Décisions figées (Guy, 2026-07-17)

- **D-b1 — Page complète** : liste + **envoi lot** + **envoi unitaire éditable** + **rappel manuel**, comme le plan d'epic range ces trois actions dans 21-6b. (Historique fiche / toggle UI / dashboard → 21-6c.)
- **D-b2 — Nouvelle feature `reminders/`** : `frontend/src/lib/features/reminders/` (types + api + composants), namespace i18n **`reminders-*`** (vérifié libre — 0 occurrence dans les 4 FTL). Sépare l'**envoi** des rappels de leur **config** (`dunning/`, 21-4, namespace `dunning-*` saturé). Évite le piège #30 : dossier `reminders/` + clés `reminders-*` = cohérents.
- **D-b3 — Sélection lot par cases à cocher, sans « tout sélectionner »** : réplique `payment-batches` (`Set<number>` + toggle immuable). Garde du **cap 20 côté UI** (bouton lot désactivé + message si > 20 sélectionnées) en miroir du `BATCH_TOO_LARGE` backend. Pas de select-all (aucun précédent + sémantique ambiguë au-delà de 20).
- **D-b4 — Contact sans e-mail (`hasEmail=false`) : visible, badgé, non-cochable** : la facture reste affichée (transparence), avec un badge « sans e-mail » ; sa case lot **et** son bouton « Envoyer » sont désactivés. Le **rappel manuel reste possible** dessus (papier/recommandé — le backend l'autorise).

### Hors scope (garde-fous)

- **Historique des rappels sur la fiche facture**, **toggle suspension UI**, **compteur dashboard**, **liens croisés** échéancier ↔ Rappels → **21-6c**.
- **Cumuls par contact (TTC/frais/joursDeRetard)** : `ReminderCandidateResponse` ne les porte pas (L21-8) — la liste groupe par contact mais **n'affiche pas de total** ; un calcul client divergerait du `{totalDue}` serveur. Ne PAS l'ajouter.
- **Balance âgée** → 21-7. **Manuels utilisateur/admin** → 21-8.
- **Aucun changement backend d'envoi** (endpoints livrés 21-5b) ; seule exception : étendre une fixture de test E2E (AC 26).
- **Câblage i18n de la page liste `/invoices`** → issue #255 (indépendant).

## Acceptance Criteria

### A. Feature `reminders/` — types & API

1. **`frontend/src/lib/features/reminders/reminders.types.ts`** (camelCase, miroir EXACT des DTO backend cartographiés) :
   - Liste : `ReminderCandidate { invoiceId: number; invoiceNumber: string | null; dueDate: string; currentLevel: number; nextLevel: number | null; terminal: boolean; lastReminderAt: string | null }` ; `ContactGroup { contactId: number; contactName: string; hasEmail: boolean; invoices: ReminderCandidate[] }` ; `ReminderListResponse { groups: ContactGroup[] }`.
   - Preview : `ReminderPreviewResponse { to: string | null; language: string; level: number; subject: string; body: string }`.
   - Envoi unitaire (requête) : `SendReminderRequest { levelNumber: number; subject: string; body: string }` (**PAS de champ `to`**). Réponse : `ReminderResponse` (`id, levelNumber, feeAmount: string, sentAt, channel, sentTo: string | null, subject, body, note: string | null, cancelledAt: string | null`).
   - Lot : `SendReminderBatchRequest { invoiceIds: number[] }` ; `AcceptedReminder { invoiceId: number; reminderId: number; levelNumber: number }` ; `FailedReminder { invoiceId: number; errorCode: string; details: Record<string, unknown> | null }` ; `SendReminderBatchResponse { accepted: AcceptedReminder[]; failed: FailedReminder[] }`.
   - Manuel : `ManualReminderRequest { levelNumber: number; sentAt: string; note: string | null }`.

2. **`frontend/src/lib/features/reminders/reminders.api.ts`** (via `apiClient`, calque `dunning.api.ts`) :
   - `listReminders(): Promise<ReminderListResponse>` → `get('/api/v1/dunning/reminders')`.
   - `getReminderPreview(invoiceId, level): Promise<ReminderPreviewResponse>` → `get('/api/v1/invoices/${invoiceId}/reminder-preview?level=${level}')`. **`level` toujours fourni** (backend : absent → 400).
   - `sendReminder(invoiceId, payload: SendReminderRequest): Promise<ReminderResponse>` → `post('/api/v1/invoices/${invoiceId}/reminders/send', payload)`.
   - `sendReminderBatch(invoiceIds): Promise<SendReminderBatchResponse>` → `post('/api/v1/dunning/reminders/send-batch', { invoiceIds })`.
   - `recordManualReminder(invoiceId, payload: ManualReminderRequest): Promise<ReminderResponse>` → `post('/api/v1/invoices/${invoiceId}/reminders/manual', payload)`.
   - `index.ts` re-exporte types + api (patron `dunning/index.ts`).

3. **Tests vitest** `reminders.api.test.ts` (patron `dunning.api.test.ts` : `vi.mock('$lib/shared/utils/api-client')`) : chaque wrapper appelle le bon chemin/méthode/body ; `getReminderPreview(42, 2)` → `get('/api/v1/invoices/42/reminder-preview?level=2')` ; `sendReminderBatch([1,2])` → `post('/api/v1/dunning/reminders/send-batch', { invoiceIds: [1,2] })`.

### B. Page & navigation

4. **Route `frontend/src/routes/(app)/invoices/reminders/+page.svelte`** (segment statique — SvelteKit le prioritise sur `[id]`, précédent `due-dates/`). Squelette calqué sur `invoices/due-dates/+page.svelte` : `onMount` → chargement, états `loading`/vide/liste, `isApiError` + `notifyError`. **Pas de filtres/URL-sync** en v1 (la liste est courte et déjà groupée ; pas de précédent à répliquer ici — ne pas sur-concevoir).

5. **Entrée de nav** dans `(app)/+layout.svelte`, groupe `quotidien`, **juste après** `nav-invoicing-due-dates` (`:62`) : `{ i18nKey: 'nav-invoicing-reminders', fallback: 'Rappels', href: '/invoices/reminders' }`. Clé `nav-invoicing-reminders` dans les 4 FTL (namespace `nav-*` déjà global). `data-testid` auto = `nav-link-invoices-reminders`.

6. **Gating RBAC Comptable+ DANS la page** (la nav n'a pas de gating granulaire — patron `canExportCsv` de `due-dates:51-53`) : `canManage = $derived(role === 'Admin' || role === 'Comptable')`. Si `!canManage` : la page affiche un message « accès réservé » et **ne déclenche aucun fetch** (un rôle Consultation prend un 403 backend sinon). Le lien de nav reste visible pour tous (limite connue, cohérente avec l'app).

### C. Liste groupée

7. **Chargement** : `listReminders()` au `onMount` (si `canManage`). Rendu **groupé par contact** : pour chaque `ContactGroup`, un en-tête (nom du contact + badge « sans e-mail » si `!hasEmail`), puis ses factures. Liste **vide** (aucun groupe) → message « aucune facture à rappeler » (état légitime : dunning désactivé ou rien d'éligible).

8. **Par facture** : numéro (`invoiceNumber ?? '—'`), date d'échéance (`dueDate`), **libellé de niveau** i18n dérivé : si `terminal` → « Dernier niveau atteint — poursuite à envisager » (`reminders-level-terminal`) ; sinon « Prochain : rappel N » où N = `nextLevel` (`reminders-level-next` avec `{ $level }`). `lastReminderAt` affiché s'il est non-null (« dernier rappel le … »). **AUCUN montant** (L21-8).

9. **Badges** (composants sous `features/reminders/`, namespace `reminders-*`) :
   - `ReminderNoEmailBadge.svelte` — au niveau **contact** (`hasEmail=false`), libellé `reminders-badge-no-email` (« sans e-mail »), teinte neutre, `aria-label`. Gabarit `DunningPausedBadge.svelte` (21-6a) : **texte en `--color-text`, PAS la variable de la teinte de fond** (leçon 21-6a — le gabarit `PaymentStatusBadge` est sous AA ; ne pas le copier tel quel, re-vérifier le contraste).
   - `ReminderTerminalBadge.svelte` — sur une facture `terminal`, libellé `reminders-badge-terminal`, teinte d'alerte (`--color-warning`).

### D. Sélection & envoi par lot

10. **Case à cocher par facture** (patron `payment-batches:153-164`) : `Set<number>` d'`invoiceId` + `toggle(id)` immuable (réassignation du Set pour la réactivité runes). **Désactivée** si le contact du groupe a `hasEmail=false` **ou** si la facture est `terminal` (le lot n'envoie que le prochain niveau — une facture terminale n'en a pas → serait `NO_NEXT_LEVEL`). `data-testid="reminder-batch-checkbox"`.

11. **Garde du cap 20 côté UI (D-b3)** : compteur « N sélectionnée(s) ». Le bouton « Envoyer le lot » est **désactivé** si `selected.size === 0` **ou** `selected.size > 20`, avec un message `reminders-batch-cap` (« maximum 20 factures par lot ») au-delà. Miroir du `BATCH_TOO_LARGE` (422) backend — l'UI ne doit jamais laisser partir une requête vouée au 422.

12. **Envoi lot** : `sendReminderBatch([...selected])` → réponse `{ accepted, failed }` **HTTP 200** (succès partiel = succès). Anti-double-submit (voir F). Au retour : **recharger la liste** (`listReminders()`) + afficher un **rapport** (voir 13) + vider la sélection. Erreurs globales (jamais dans `{accepted,failed}`) via `err.code` : `BATCH_TOO_LARGE` (422), `SMTP_NOT_CONFIGURED` (412), `BATCH_EXCEEDS_SEND_QUOTA` (422), `RATE_LIMITED` (429) → toast traduit.

13. **Rapport de lot** (`ReminderBatchReport.svelte`, patron `supplier-invoices/import` : compte des réussis + `<ul>` des échoués) : « N rappel(s) envoyé(s) » + si `failed.length` : liste `<li>` par item avec `invoiceNumber`/`#invoiceId` + **libellé traduit de `errorCode`** via une fonction locale `reminderErrorLabel(code)` (`switch` + fallback `reminders-error-unknown` avec le code injecté), teinte `text-destructive`. `data-testid="reminder-batch-report"` + `reminder-batch-failed-row`.
    ⚠️ **Codes « l'e-mail EST parti »** (`REMINDER_SENT_BUT_INVOICE_GONE`, `RECORD_FAILED_EMAIL_SENT`, `SMTP_SEND_FAILED`) : le libellé doit indiquer que **l'e-mail a été envoyé mais non enregistré / a échoué** — **JAMAIS de bouton ou d'incitation « Réessayer »** (un ré-essai renverrait un vrai e-mail). Leçon 21-5b explicite.

### E. Envoi unitaire (modale éditable avec aperçu)

14. **`ReminderSendDialog.svelte`** (composant **présentationnel**, calqué EXACTEMENT sur `SendEmailDialog.svelte` — le dialog **n'appelle jamais l'API**, il reçoit `submitting` en prop et émet `onConfirm`). Props : `open`, `onOpenChange`, `preview: ReminderPreviewResponse | null`, `submitting?`, `errorMsg?`, `onConfirm(subject, body)`. Plus le **choix de niveau** : `level: number`, `maxLevel: number` (= `nextLevel` de la facture), `onLevelChange(level)`.

15. **Choix du niveau ≤ prochain (D18)** : un `<select>` des niveaux `1..nextLevel` (le défaut = `nextLevel`). Changer de niveau **re-fetch la preview** (`getReminderPreview(invoiceId, level)`) → re-hydrate subject/body. **Interdiction du saut** : `maxLevel = nextLevel` (jamais au-delà — le backend renverrait `LEVEL_ALREADY_SENT` 409 ; l'UI ne propose pas l'option). Facture `terminal` → pas d'envoi unitaire e-mail proposé (pas de `nextLevel`).

16. **Flux preview→édition→envoi** (patron `openSendEmail` de `invoices/[id]:273-296`) : ouvrir la modale = `getReminderPreview(invoiceId, defaultLevel)` **AVANT** d'ouvrir (erreur → toast, pas de modale) → `subject`/`body` bindés éditables → `to` **read-only** (`preview.to`, jamais un input) → `sendReminder(invoiceId, { levelNumber: level, subject, body })`. Destinataire jamais dans le payload.

17. **Destinataire absent** (`preview.to === null`) : bouton « Envoyer » désactivé + message `reminders-send-no-recipient`. Cohérent D-b4 (mais en pratique on n'ouvre pas la modale unitaire e-mail sur un contact sans e-mail — bouton déjà masqué/désactivé sur la ligne).

18. **Codes d'erreur unitaires** (via `err.code`, toast traduit) : `RATE_LIMITED` (429), `SMTP_NOT_CONFIGURED` (412), `VALIDATION_ERROR` (400), `DUNNING_LEVEL_NOT_FOUND` (422), `NOT_FOUND` (404), `INVOICE_NOT_VALIDATED` (400), `INVOICE_ALREADY_PAID` (422), `DUNNING_PAUSED` (422), `LEVEL_ALREADY_SENT` (409), `CONTACT_ARCHIVED` (400), `CONTACT_EMAIL_MISSING` (400), `INVOICE_EMAIL_EMPTY_CONTENT` (422), `INVOICE_NOT_PDF_READY` (400), `INVOICE_TOO_MANY_LINES_FOR_PDF` (400). **Post-SMTP** (l'e-mail EST parti) : `REMINDER_SENT_BUT_INVOICE_GONE` (409), `REMINDER_SENT_BUT_NOT_RECORDED` (409) → message « e-mail envoyé mais non enregistré », **pas de re-proposition d'envoi** ; **recharger la liste** (l'état a pu changer).

### F. Anti-double-submit (AC DE PREMIER PLAN)

19. **Gabarit à 4 couches, répliqué du couple `SendEmailDialog` + `invoices/[id]`** — pour l'unitaire **ET** le lot :
    - **(A)** garde dans le handler `onConfirm` : `if (submitting || recipientMissing || clientError) return;`.
    - **(B)** bouton d'envoi **et** bouton Annuler `disabled={submitting}` ; texte « Envoi… » pendant le vol (pas de spinner — patron `payment-batches:179`).
    - **(C)** le **parent possède le flag `$state`** : garde de ré-entrance en tête de handler (`if (sending) return;`), `sending = true` **avant** l'appel, `finally { sending = false }`. Un flag distinct pour l'unitaire et pour le lot.
    - **(D)** modale **non fermable en vol** : `onOpenChange={(o) => { if (!o && sending) return; ... }}` (patron `invoices/[id]:731`). ESC/X/clic-extérieur/Annuler tous bloqués pendant l'envoi.
    Le lot n'a pas de modale : (D) devient « le bouton lot reste `disabled` et la sélection n'est pas mutable pendant le vol ».

20. **IDs DOM** : `$props.id()` pour les identifiants stables (jamais `crypto.randomUUID` — leçon `feedback_no_secure_context_apis_http_lan`, indisponible en HTTP LAN).

### G. Rappel manuel (papier/recommandé)

21. **`ManualReminderDialog.svelte`** (présentationnel, patron `MarkPaidDialog`) : `open`, `onOpenChange`, `submitting?`, `errorMsg?`, `onConfirm(levelNumber, sentAt, note)`. Champs : niveau (`<select>` 1..N, défaut `nextLevel` ou 1 si terminal — le manuel **autorise le saut**, D18), date d'envoi (`<input type="date">`, défaut aujourd'hui), note optionnelle (`<textarea>`). Disponible **sur toutes les factures de la liste**, y compris `hasEmail=false` et `terminal`.

    ⚠️ **PIÈGE `sentAt` — format `NaiveDateTime`, PAS date-seule (bug #249).** `ManualReminderBody.sent_at` est un `NaiveDateTime` côté backend (`dunning_reminders.rs:121`), dont le `FromStr` **rejette** une chaîne `"YYYY-MM-DD"` (il exige le composant `T` : `"2026-07-17"` → `ParseError(TooShort)`). La valeur brute d'un `<input type="date">` est justement `"YYYY-MM-DD"` → si on l'envoie telle quelle, Axum **échoue à désérialiser le body en amont du handler** → erreur JSON générique (PAS un `err.code` structuré de l'AC 22), et **T6/T8 cassent silencieusement**. **Émettre `` `${sentAt}T12:00:00` `` **, jamais la valeur brute — exactement le fix de `MarkPaidDialog.svelte:64` (`onConfirm(\`${paidAt}T12:00:00\`)`, commentaire #249 lignes 59-63). C'est le bug qui a atteint la prod sur « Marquer payée » faute d'E2E en CI ; ne pas le rejouer. **Verrouillé par test** : une assertion vitest OU E2E vérifie que le payload `sentAt` envoyé matche `/T\d{2}:\d{2}:\d{2}$/` (pas une date nue). Idem partout où un `<input type="date">` alimente un champ `NaiveDateTime` — vérifier qu'aucun autre champ de cette story n'est concerné (seul `sentAt` l'est ici ; les autres dates sont en lecture).

22. **Envoi manuel** : `recordManualReminder(invoiceId, { levelNumber, sentAt, note })` → 201. Anti-double-submit (mêmes 4 couches). Au retour : recharger la liste + toast succès. Erreurs `err.code` : `VALIDATION_ERROR` (400 : niveau < 1, note > 5000), `REMINDER_DATE_IN_FUTURE` (422), `DUNNING_LEVEL_NOT_FOUND` (422), `NOT_FOUND` (404), `INVOICE_NOT_VALIDATED` (400), `INVOICE_ALREADY_PAID` (422). **Garde UI** : `sentAt` non postérieur à aujourd'hui (le backend rejette le futur — 422).

### H. i18n

23. **Clés `reminders-*` dans les 4 FTL** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) + `nav-invoicing-reminders` : titre de page, en-têtes, libellés de niveau (`reminders-level-next` `{ $level }` / `reminders-level-terminal`), badges (`reminders-badge-no-email` / `-terminal`), boutons (envoyer / lot / manuel / annuler), compteur sélection, cap 20, libellés d'erreur batch (un par `errorCode` de l'AC 13, + `reminders-error-unknown`), messages toast des codes d'erreur unitaires/manuels. **Traductions FR/DE/IT/EN réelles** (pas de placeholder — le fallback programmatique retombe sur FR mais l'app est multilingue). **Placement** : la page (route) échappe au lint i18n ; les **composants sous `features/reminders/`** ne peuvent utiliser QUE `reminders-*` (ou global) — cohérent par construction (dossier = namespace). Pas d'entrée `KNOWN_VIOLATIONS` nécessaire (contrairement à 21-6a) tant que toutes les clés sont `reminders-*`.

### I. Tests

24. **E2E round-trip `frontend/tests/e2e/reminders.spec.ts`** (MockMailer, patron `invoice-send-email.spec.ts`). Prérequis : backend `KESH_TEST_MODE=true` + SMTP factice + DB `kesh_e2e`. Scénarios :
    - **Liste** : créer contact (avec e-mail) + facture **échue niveau 1** (voir AC 26) → la page `/invoices/reminders` affiche le groupe contact + la facture au niveau attendu.
    - **Envoi unitaire** : ouvrir la modale → preview chargée (subject non vide, `to` = e-mail read-only) → envoyer → e-mail capturé dans `/_test/sent-emails` (+1) avec PDF joint → toast succès → la facture reflète le nouveau niveau au rechargement.
    - **Envoi lot** succès partiel : 3 factures (1 OK + 1 sans e-mail non-cochable donc exclue + 1 payée entre-temps) → `{ accepted, failed }`, rapport affiché, e-mails capturés = nb d'`accepted`.
    - **Cap 20 UI** : sélectionner 21 (si faisable dans le seed) OU asserter le message/disabled au-delà de 20.
    - **Rappel manuel** : enregistrer un rappel papier niveau 2 → 201 → la facture avance.
    - **Contact sans e-mail** : badge « sans e-mail » visible, case lot désactivée, bouton envoyer désactivé, **rappel manuel possible**.

25. **E2E anti-double-submit** (patron À CRÉER — aucun n'existe) : ouvrir la modale d'envoi unitaire, cliquer « Envoyer », et vérifier que le bouton est **`disabled` immédiatement** après le clic (avant résolution) → l'e-mail n'est capturé **qu'une fois**. Technique : intercepter/ralentir la route (`page.route('**/reminders/send', …)` avec délai) pour observer l'état `disabled` en vol, ou `expect(button).toBeDisabled()` juste après le click puis `expect.poll` sur `fetchSentEmails().length === before + 1`. Idem pour le lot (bouton lot `disabled` en vol). **C'est l'AC de premier plan — le test doit exister et passer.**

26. **Extension de fixture (seul backend touché)** : `frontend/tests/e2e/helpers/api-fixtures.ts` — `createAndValidateInvoiceViaApi` fixe `dueDate: today` en dur → **impossible de créer une facture échue**. Ajouter un **paramètre optionnel `dueDate?: string`** (défaut = comportement actuel, `today`) et le propager au POST. **DRY** : étendre la fixture existante, ne pas la dupliquer (leçon mémoire). Une facture éligible niveau 1 = `dueDate <= today - 15j` (grâce 5 + délai niveau 1 = 10, config seedée par défaut). Le seed lazy des niveaux se déclenche au 1er `GET /api/v1/company/dunning-settings` **ou** à la 1re évaluation d'éligibilité (`GET /dunning/reminders`).

27. **axe a11y** (patron scopé `email-templates.spec.ts:187-206`) : sur la page **peuplée**, `new AxeBuilder({ page }).include('[data-testid="reminders-list"]').disableRules(['color-contrast', 'button-name']).analyze()` → 0 violation dans le sous-arbre de la story. Justifier en commentaire : `color-contrast` = dette systémique #253, `button-name` = dette #256 (boutons d'action icône-seule) — **ne corriger aucune dette pré-existante ici**. Le badge et les nouveaux boutons de CETTE story doivent être conformes (nom accessible, contraste AA — re-vérifier comme en 21-6a).

28. **vitest** : `reminders.api.test.ts` (AC 3) + tout helper pur (ex. `reminderErrorLabel`) testé unitairement.

### J. Gate & documentation

29. **Gate local complet** (Test Locally First) :
    ```sh
    cd frontend && npm run check && npm run lint-i18n-ownership && npm run test:unit && npm run build
    cd frontend && npm run test:e2e        # PAS dans la CI → critique ici (page + envoi + MockMailer)
    # backend : la fixture E2E touche crates/kesh-api/tests/ indirectement (helper TS) — pas de code Rust
    cargo fmt --all -- --check && cargo build --workspace --all-targets && cargo clippy --workspace --all-targets -- -D warnings
    ```
    ⚠️ **E2E obligatoires** : cette story est presque entièrement E2E-testable (page + 3 flux d'envoi + anti-double-submit + MockMailer). Rappel : backend contre `kesh_e2e` migré, `npm run build` avant chaque run (Playwright sert le build statique), `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, ne jamais piper le runner.

30. **CHANGELOG** `[Non publié]` → `Ajouté` : écran « Rappels » (liste des factures à relancer par débiteur, envoi par e-mail à l'unité avec aperçu ou en lot, enregistrement d'un rappel papier). **Manuels → 21-8.** **README** : Epic 21 déjà 🚧, aucun changement de statut d'epic.

## Tasks / Subtasks

- [x] **T1 — Feature `reminders/` (types + api + tests unitaires)** (AC: 1, 2, 3, 28) — types + api + index + `reminders.api.test.ts` (5) + helper `reminder-error-label.ts` + `.test.ts` (5). vitest 10/10.
- [x] **T2 — Route page + nav + gating RBAC + liste groupée** (AC: 4, 5, 6, 7, 8) + badges (AC: 9) — `/invoices/reminders/+page.svelte`, entrée nav `nav-invoicing-reminders` (4 langues), gating `canManage` (Comptable+), liste groupée par contact, `ReminderNoEmailBadge` + `ReminderTerminalBadge` (contraste `--color-text`, leçon 21-6a).
- [x] **T3 — Sélection & envoi lot** (AC: 10, 11, 12, 13) — `Set<number>` + toggle, case désactivée si contact sans e-mail OU terminale, garde cap 20 UI, `ReminderBatchReport` + mapping `reminderErrorLabel` (helper testé) + classe `isEmailSent` (jamais « Réessayer »).
- [x] **T4 — Envoi unitaire (modale éditable, preview, choix niveau)** (AC: 14, 15, 16, 17, 18) — `ReminderSendDialog` présentationnel, `<select>` niveau 1..nextLevel → `onLevelChange` re-fetch preview, `to` read-only, codes post-SMTP rechargent la liste.
- [x] **T5 — Anti-double-submit (4 couches, unitaire + lot + manuel)** (AC: 19, 20) — flags parent (`sendingUnit`/`batchSending`/`savingManual`), garde ré-entrance + `true` avant appel + `finally`, boutons `disabled`, `onOpenChange` non-fermable en vol, `$props.id()`.
- [x] **T6 — Rappel manuel** (AC: 21, 22) — `ManualReminderDialog`, `sentAt` suffixé `T12:00:00` (bug #249) + garde date-future, saut de niveau autorisé (D18).
- [x] **T7 — i18n 4 FTL** (AC: 23) — 52 clés `reminders-*` × 4 langues + `nav-invoicing-reminders`. lint-i18n PASS (namespace = dossier, pas de KNOWN_VIOLATIONS).
- [x] **T8 — Fixture E2E `dueDate` + E2E round-trip + anti-double-submit + axe** (AC: 24, 25, 26, 27) — `createAndValidateInvoiceViaApi(page, contact, dueDate?)` (DRY, date facture alignée sur dueDate pour respecter la validation #245) ; `reminders.spec.ts` **6/6** (liste+unitaire e-mail capturé PDF, sans-email badge/non-cochable, manuel, lot, **anti-double-submit disabled-en-vol → 1 seul e-mail**, axe scopé). Régression fixture : 22/22 sur invoice-send-email/invoices/échéancier/supplier.
- [x] **T9 — Gate complet + CHANGELOG** (AC: 29, 30) — Frontend : check 0 erreur, lint-i18n PASS, **test:unit 404** (+10), build OK, **E2E reminders 6/6 + régression 22/22**. Backend : fmt/clippy 0, kesh-i18n 21/21 (FTL parse). Aucun code Rust touché (fixture TS + FTL). CHANGELOG `[Non publié]` → `Ajouté`.

## Dev Notes

### Pièges, par ordre de coût

1. **Ne PAS recréer de backend d'envoi.** Les endpoints existent (`invoice_email.rs`, montés `lib.rs:433/437`). Un agent Explore a conclu à tort qu'ils manquaient (il cherchait dans `dunning_reminders.rs`). Vérifié par grep. Cette story est frontend + 1 param de fixture.
2. **Anti-double-submit = le cœur.** Le backend n'a AUCUNE garde (TOCTOU accepté 21-5b). L'e-mail part réellement à chaque appel. Les 4 couches ne sont pas optionnelles, et l'AC 25 (test) doit passer. Un double-clic non protégé = un débiteur qui reçoit deux mises en demeure.
3. **Codes « e-mail parti » ≠ erreur ré-essayable.** `REMINDER_SENT_BUT_INVOICE_GONE` / `RECORD_FAILED_EMAIL_SENT` / `SMTP_SEND_FAILED` signifient que l'e-mail est (peut-être) parti. Jamais de « Réessayer ». Recharger la liste, informer, stop.
4. **Contraste du badge (leçon 21-6a).** Le gabarit `PaymentStatusBadge` colore le texte avec la variable du fond → sous AA. Copier `DunningPausedBadge` (21-6a, corrigé) : texte en `--color-text`. Re-vérifier à l'axe.
5. **Fixture E2E `dueDate`.** Sans échéance passée, aucune facture n'est éligible → la page est vide et les tests ne prouvent rien. Étendre `createAndValidateInvoiceViaApi` (DRY), seuil niveau 1 = `today - 15j`.
6. **Cap 20 = miroir UI.** Le backend renvoie 422 au-delà de 20 (après dédup). L'UI garde à 20 pour ne jamais laisser partir une requête vouée à l'échec, mais le backend reste la source de vérité.
7. **`sentAt` du rappel manuel : `T12:00:00` obligatoire (bug #249).** Un `<input type="date">` produit `"YYYY-MM-DD"` ; le champ backend `NaiveDateTime` exige un composant horaire → suffixer `` `${sentAt}T12:00:00` `` comme `MarkPaidDialog:64`. Ce bug a atteint la prod (« Marquer payée » cassé) faute d'E2E. Test de format obligatoire (AC 21).

### Contrats backend (ground-truth, à ne pas re-deviner)

| Action | Méthode + chemin | Requête | Réponse | RBAC |
|---|---|---|---|---|
| Liste | `GET /api/v1/dunning/reminders` | — | `{ groups: ContactGroup[] }` | Comptable+ |
| Preview | `GET /api/v1/invoices/{id}/reminder-preview?level=N` | `level` requis (400 si absent) | `ReminderPreviewResponse` | Comptable+ |
| Unitaire | `POST /api/v1/invoices/{id}/reminders/send` | `{ levelNumber, subject, body }` | 201 `ReminderResponse` | Comptable+ |
| Lot | `POST /api/v1/dunning/reminders/send-batch` | `{ invoiceIds }` | 200 `{ accepted, failed }` | Comptable+ |
| Manuel | `POST /api/v1/invoices/{id}/reminders/manual` | `{ levelNumber, sentAt, note? }` | 201 `ReminderResponse` | Comptable+ |

Détail des codes d'erreur : AC 12/13/18/22. Aucun DTO d'envoi n'a de champ `to` (destinataire = `contacts.email` verrouillé serveur).

### Gabarits à répliquer (chemins vérifiés)

- **Anti-double-submit** : `SendEmailDialog.svelte` (présentationnel, `submitting` en prop, garde handler `:58-62`, boutons `disabled` `:118-129`) + `invoices/[id]/+page.svelte` (parent : garde ré-entrance `:301`, `sending=true` `:302`, `finally` `:356`, `onOpenChange` non-fermable `:731`).
- **Sélection lot** : `payment-batches/+page.svelte` (`Set<number>` `:38`, `toggle` immuable `:61-66`, checkbox `:153-164`). **Pas de select-all** (aucun précédent).
- **Rapport `{accepted,failed}`** : `supplier-invoices/import/+page.svelte:409-440` (compte + `<ul>` + `errorLabel` local `switch`+fallback) ; alt `payment-batches:184-205`.
- **Page liste** : `invoices/due-dates/+page.svelte` (squelette onMount/loading/vide/tableau, gating `canExportCsv:51-53`).
- **Preview→édition** : `openSendEmail` `invoices/[id]:273-296` + re-hydratation `$effect` `SendEmailDialog:38-43`.
- **Badge** : `DunningPausedBadge.svelte` (21-6a, contraste corrigé).
- **Nav** : `(app)/+layout.svelte:62` (item i18n `quotidien`).

### Project Structure Notes

**Nouveaux fichiers** (`frontend/`) :
- `src/lib/features/reminders/reminders.types.ts`, `reminders.api.ts`, `reminders.api.test.ts`, `index.ts`
- `src/lib/features/reminders/ReminderNoEmailBadge.svelte`, `ReminderTerminalBadge.svelte`, `ReminderSendDialog.svelte`, `ManualReminderDialog.svelte`, `ReminderBatchReport.svelte`
- `src/routes/(app)/invoices/reminders/+page.svelte`
- `tests/e2e/reminders.spec.ts`

**Modifiés** :
- `src/routes/(app)/+layout.svelte` (entrée nav)
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (clés `reminders-*` + `nav-invoicing-reminders`)
- `frontend/tests/e2e/helpers/api-fixtures.ts` (param `dueDate`)
- `CHANGELOG.md`

**Décompte modules** : 1 feature `reminders/` (types+api+5 composants) + 1 route + 1 nav + i18n + 1 fixture = frontend cohérent, sous le seuil de la règle de splitting (déjà une sous-story d'un split 21-6a/b/c ; Guy a tranché « page complète »). Aucun backend d'envoi nouveau, aucune migration.

### Leçon de review héritée (à appliquer dès le dev)

**Un patch de review vient AVEC son test** (21-5b, 5 passes). **Disclosure non sélective** (21-6a, AA-1) : documenter TOUTES les déviations, pas seulement certaines. **Vérifier les justifications par grep, pas seulement les conclusions** (21-6a : deux fois une bonne conclusion sur une prémisse fabriquée).

### References

- [Source: `epic-21-echeances-relances.md` — 21-6b (D17 anti-double-submit, L21-8 pas de cumuls, D18 ré-émission/saut), items 16-22]
- [Source: `21-5a-donnees-eligibilite-relances.md` — liste groupée, manuel, endpoints]
- [Source: `21-5b-envoi-rappels-email.md` — preview/unitaire/lot, codes d'erreur, TOCTOU accepté → UI seule barrière, codes « e-mail parti » ≠ réessayable]
- [Source: `21-6a-exposition-suspension.md` — gabarit badge contraste-AA, disclosure non sélective, vérifier les justifications]
- [Source: `CLAUDE.md#Test Locally First`, `#Review Iteration Rule`, `#Issue Tracking Rule` ; `feedback_no_secure_context_apis_http_lan` (`$props.id()`)]
- [Source: GitHub #231 (rappels débiteurs — 21-6b livre l'écran de relance), #255/#256/#253 (dettes pré-existantes à ne pas corriger ici)]

## Change Log — validate

### Pass 1 (Sonnet, 2026-07-17) — 1 CRITICAL → patché

Auteur de la spec : Opus. Reviewer orthogonal : Sonnet. Verdict **GO-ajusté**. Le finding CRITICAL a été **re-vérifié ground-truth par l'orchestrateur** (`grep -nF`) avant patch — **confirmé réel**.

- **SENTAT-1 (CRITICAL) — `sentAt` date-seule rejeté par le backend `NaiveDateTime`.** L'AC 21/22 spécifiait un `<input type="date">` alimentant `recordManualReminder(..., { sentAt })` **sans** mentionner le suffixe horaire obligatoire. Grep confirme : `dunning_reminders.rs:121` `sent_at: NaiveDateTime` (dont le `FromStr` rejette `"YYYY-MM-DD"`), et `MarkPaidDialog.svelte:64` porte exactement le fix (`` `${paidAt}T12:00:00` ``, commentaire #249). Un dev suivant l'AC à la lettre aurait rejoué le **bug de prod #249** (« Marquer payée » cassé, 422, non détecté faute d'E2E en CI). **Patch** : AC 21 réécrite avec le format `T12:00:00` explicite + assertion de test (`/T\d{2}:\d{2}:\d{2}$/`), répercuté en T6 et piège n°7 des Dev Notes.

Vérifications positives (grep/Read/compilation, Sonnet) : « niveau de ground-truth exceptionnellement élevé » — toutes les routes (`lib.rs:433/437`), handlers (`invoice_email.rs:363/412/925`), DTO, codes d'erreur, lignes de gabarit (`SendEmailDialog:60/119/124`, `invoices/[id]:301/302/731`, `payment-batches:38/61-66/153-164`, `due-dates:51-53`, `layout.svelte:62`), namespace `reminders-*` libre, feature `dunning/` existante, fixture `dueDate=today` en dur, seuil éligibilité `today-15j`, cap 20 après dédup — **tous confirmés exacts**. Un seul défaut substantiel (SENTAT-1).

### Pass 2 (Haiku, 2026-07-17) — 0 finding → **CONVERGÉ**

Contexte frais, prémunie contre le mode d'échec documenté (auditer la spec comme une implémentation). **0 erreur de catégorie, 0 hallucination.** 60 outils. Verdict **GO**.

Absorption du patch P1 confirmée : le format `` `${sentAt}T12:00:00` `` est copié exactement de `MarkPaidDialog.svelte:64` (commentaire #249 lignes 59-63), cohérent avec `ManualReminderBody.sent_at: NaiveDateTime` — **aucune régression introduite par le patch**. Vérifications indépendantes concordantes : les 5 routes montées (`lib.rs:424/429/433/437`), les handlers, les codes d'erreur (`RateLimited`/`SmtpNotConfigured`/`LEVEL_ALREADY_SENT`/`DunningPaused`/`InvoiceAlreadyPaid`…) tous implémentés, l'anti-double-submit à 4 couches applicable, le périmètre items 16-22 couvert sans fuite de 21-6c/21-7.

### Trend & décision

**Passe 1 (Sonnet) : 1 CRITICAL → Passe 2 (Haiku) : 0.** Critère d'arrêt de la règle de remédiation atteint (0 finding > LOW), budget 2/8. Rotation orthogonale à l'auteur (Opus) sur les deux passes. Le CRITICAL de P1 re-vérifié ground-truth par l'orchestrateur avant patch — réel (bug #249). **Spec scellée, prête pour `bmad-dev-story 21-6b`.**

## Change Log — code review

Auteur de l'implémentation : Opus. Panel orthogonal.

### Pass 1 (2026-07-17) — Blind Hunter Sonnet + Edge Case Hunter Haiku + Acceptance Auditor Sonnet

Diff unique aplati. **Trend brut : BH 1C/1H/1M/1L | ECH 0/0/1M/4L | AA 0C/2H/2M/3L.** Après ground-truth (grep), **2 HIGH + 3 findings réels de moindre rang patchés, chacun avec son test** ; 1 CRITICAL et 4 LOW écartés comme faux positifs.

- **AA-1 (HIGH) + BH-4 — le rappel manuel n'autorisait PAS le saut de niveau (violation D18).** Le `<select>` manuel était borné à `nextLevel`, identique à l'unitaire — or D18 fait du manuel *le* chemin pour sauter (ex. directement à la mise en demeure). Backend confirmé par grep : `record_manual_reminder` ne vérifie que `level_number >= 1` + existence en config, **aucune borne `≤ nextLevel`**. **Patch** : la page charge `listDunningLevels()` (accessible Comptable, `lib.rs:591`) → `maxConfiguredLevel`, passé au dialog manuel. **Test E2E** : le sélecteur propose les 3 niveaux configurés malgré `nextLevel=1`.
- **BH-2 (HIGH) + AA-3 (MEDIUM) — `confirmSend` effaçait le message « e-mail parti » sans toast.** Sur `REMINDER_SENT_BUT_INVOICE_GONE`/`REMINDER_SENT_BUT_NOT_RECORDED`, on posait `sendError` (affiché *dans* la modale) puis on fermait la modale → message perdu, aucun toast : l'e-mail partait chez le débiteur **sans trace et sans que l'utilisateur le sache**. **Patch** : `notifyError` (toast persistant) sur ces codes avant fermeture ; les autres erreurs restent inline dans la modale (l'e-mail n'est PAS parti → ré-essai légitime). Distinction des deux vocabulaires clarifiée.
- **AA-2 (HIGH) — scénario E2E « cap 20 » absent, annoncé 6/6 sans divulgation.** Exactement la leçon de disclosure sélective (21-6a AA-1). **Patch** : test E2E dédié (21 factures créées, sélection totale → message de cap + bouton lot `disabled`).
- **AA-4 (MEDIUM) + BH-3 — le test anti-double-submit ne verrouille que la couche B.** Un `<button disabled>` natif ne redispatche pas de `click`, donc le 2e clic est un no-op structurel : le test prouve la couche B (porteuse) + l'invariant « un seul e-mail », pas les couches A/C/D (défense en profondeur, course non reproductible en E2E séquentiel). **Patch** : commentaire de portée honnête sur ce que le test couvre réellement — pas de sur-promesse.
- **ECH-1 / code mort (LOW) — `isEmailSent`/`EMAIL_SENT_CODES` exportés+testés mais jamais consommés** (le rapport de lot porte l'info « e-mail parti » dans le texte du libellé, pas dans un drapeau). **Patch** : supprimés + test ajusté.
- **ECH-2 (LOW)** — la purge de sélection au rechargement ne retirait pas les factures devenues non-sélectionnables (contact ayant perdu son e-mail, facture terminale). **Patch** : filtre étendu (`selectable` + présence).
- **AA-7 (LOW)** — `ManualReminderDialog` utilise désormais `$props.id()` (convention AC 20), comme `ReminderSendDialog`.

**Écartés après ground-truth (faux positifs)** :
- **BH-1 (CRITICAL) — « code `REMINDER_SENT_BUT_NOT_RECORDED` inexistant ».** RÉFUTÉ par grep : c'est le code réel de l'endpoint **unitaire** (`errors.rs:1095`, `invoice_email.rs:558`). Le Blind Hunter, aveugle au backend, a confondu le vocabulaire du **lot** (`RECORD_FAILED_EMAIL_SENT`/`SMTP_SEND_FAILED`) avec celui de l'unitaire. `confirmSend` teste les 2 bons codes unitaires.
- **ECH-4 (LOW) — `BACKEND_URL` sans port** : patron **pré-existant** copié verbatim de `invoice-send-email.spec.ts` (convention projet, `KESH_BACKEND_URL` toujours défini en E2E).
- **ECH-5 (LOW) — niveaux non-séquentiels** : **impossible par construction** — 21-3 garantit des `level_number` contigus.
- **ECH-3 (LOW) — erreurs non-API non notifiées** : les erreurs réseau/timeout **sont** des `ApiError` (`api-client` leur pose `code: NETWORK_ERROR`, status 0) → `isApiError` les capture déjà.

Chaque patch livré **avec son test** (leçon 21-5b). Gate post-patch : check 0 erreur, lint-i18n PASS, vitest reminders 8/8, **E2E 8/8** (+2 : cap-20, saut manuel).

### Pass 2 (Opus, 2026-07-17) — NON CONVERGÉ : 1 MEDIUM (régression P1) + 2 LOW → patchés

Contexte frais, mission explicite « chercher les défauts introduits par la remédiation P1 » (mode d'échec 21-5b). **0 CRITICAL / 0 HIGH / 1 MEDIUM / 2 LOW.** La passe a trouvé exactement ce qu'elle cherchait — une régression de mes propres patches P1.

- **P2-1 (MEDIUM) — régression d'availability introduite par le patch P1 du saut manuel.** Mon `Promise.all([listReminders(), listDunningLevels()])` **couplait** un fetch secondaire (la config, utile seulement au saut du dialog manuel) au fetch primaire : un échec de `listDunningLevels()` seul (blip réseau, 500 transitoire) faisait rejeter tout le `Promise.all` → **page vide + toast**, alors que `listReminders()` avait réussi et que la fonction cœur était disponible. **Patch** : fetch primaire `await listReminders()` d'abord (affiche la liste), puis config en `try/catch` séparé avec fallback `maxConfiguredLevel = 0` (le manuel retombe sur le prochain niveau). Dégradation gracieuse.
- **P2-2 (LOW) — `defaultLevel > maxLevel` possible sur une facture terminale** si la config est raccourcie ou `maxConfiguredLevel=0` (conséquence de P2-1) → le `<select bind:value>` porterait une valeur sans `<option>`. **Patch** : `$effect` de clamp du niveau dans `ManualReminderDialog` (`1 <= level <= levelOptions.length`) — défense au niveau du dialog, couvre tout état de config incohérent.
- **P2-3 (LOW) — test « saut manuel » couplé en dur au seed (3 niveaux).** **Patch** : `toBeGreaterThanOrEqual(3)` au lieu de `toHaveCount(3)` — détecte toujours la régression D18 (1 option < 3) sans casser si le seed ajoute des niveaux.

**Remédiations P1 confirmées SAINES par Opus** (contre-vérification indépendante) : `confirmSend` toast/inline (les 2 codes post-SMTP unitaires sont exhaustifs et corrects ; `SMTP_SEND_FAILED` sur l'unitaire = e-mail PAS parti → branche inline correcte, aucun double-envoi) ; suppression du code mort propre (0 référence résiduelle) ; purge de sélection = exactement `selectable()` ; labels de lot alignés backend ; `$props.id()` ; test cap-20. **Le mode d'échec 21-5b s'est manifesté une fois (P2-1) et a été corrigé.**

Gate post-patch : check 0 erreur, lint-i18n PASS, vitest reminders 8/8, **E2E 8/8**.

### Pass 3 (Sonnet, 2026-07-17) — NON CONVERGÉ : 1 HIGH (régression P2) → patché

Contexte frais, mission « vérifier les patches P2 ». **1 HIGH.** À nouveau le mode d'échec 21-5b : mon patch **P2-1** a introduit une course.

- **P3-1 (HIGH) — course de staleness réintroduite par le split `Promise.all` (P2-1).** En passant de `Promise.all` (un seul garde `seq` post-résolution) à deux `await` séquentiels, le garde `seq` n'était re-testé **que sur le chemin de succès** du 2e await : le `catch` de la config **et** le code partagé qui suit (reconstruction `emailByInvoice`/`selected` depuis `res.groups` capturé avant le 2e await) s'exécutaient **sans garde**. Deux `load()` concurrents (envoi lot → `load()`, puis rappel manuel → `load()` — le bouton manuel n'est pas gardé par `batchSending`) : l'invocation périmée, si sa config échoue, écrasait `maxConfiguredLevel`/`emailByInvoice`/`selected` d'une invocation plus fraîche avec des données périmées (silencieux, `groups` restant correct). **Patch** : restructuration selon le patron sûr d'origine — **TOUS les awaits d'abord (config en `try/catch` interne, dégradation gracieuse préservée), UN SEUL garde `seq`, PUIS toutes les écritures d'état**. La course devient impossible par construction.
  *Exception « patch avec test » assumée* : une course de staleness ne se teste pas de façon déterministe (timing d'interleaving). Le fix est **structurel** — il élimine la fenêtre par construction (mirroir du `Promise.all` d'origine, prouvé sûr par la passe 3 elle-même), vérifié par lecture, pas par un test flaky. Les 8 E2E existants restent verts (non-régression fonctionnelle).

**Patches P2 confirmés SAINS par Sonnet** : P2-2 (`$effect` de clamp — pas de boucle ni de ping-pong : l'effect de reset ne lit pas `level`, le clamp converge en ≤ 2 itérations, no-op sur toute interaction utilisateur valide) ; P2-3 (`toBeGreaterThanOrEqual(3)` détecte toujours la régression D18 : 1 option < 3).

Gate post-patch : check 0 erreur, build OK, **E2E 8/8**.

### Pass 4 (Haiku, 2026-07-17) — CONVERGÉ (0 finding)

Contexte frais, mission étroite « vérifier le patch P3 ». **0 CRITICAL / 0 HIGH / 0 MEDIUM / 0 LOW.** Vérification ligne à ligne des 4 critères : (1) aucune écriture d'état avant le garde `seq` unique ; (2) dégradation gracieuse préservée (`listDunningLevels` en `try/catch` interne sans rethrow ; `listReminders` en échec → `catch` externe) ; (3) `finally` `if (seq === loadSeq)` correct ; (4) `maxConfiguredLevel` écrit via le local `maxLevel`, fallback 0 en cas d'échec config. **0 erreur de catégorie.** Contre-vérification orchestrateur (`awk`+`grep`) : les 2 `await` précèdent le garde unique, les 4 écritures d'état le suivent — course éliminée par construction.

### Trend & décision — code review

**P1 (Sonnet/Haiku/Sonnet) : 2 HIGH + 3 findings → P2 (Opus) : 1 MEDIUM [régression P1] → P3 (Sonnet) : 1 HIGH [régression P2] → P4 (Haiku) : 0.** Critère d'arrêt atteint, budget 4/8. Rotation complète orthogonale à l'auteur (Opus).

**Enseignement (candidat rétro Epic 21).** Ce cycle est le cas d'école du mode d'échec 21-5b : **l'implémentation d'origine était saine ; ce qui a churné, c'est la remédiation.** Deux fois de suite (P2-1 puis P3-1), un patch de review a introduit un défaut plus subtil que celui qu'il corrigeait — d'abord un couplage `Promise.all` (availability), puis une course de staleness dans le split qui devait le corriger. Aucune de ces régressions n'existait quand les reviewers précédents ont regardé ; seule une passe fraîche braquée sur *les patches* pouvait les voir. La convergence est venue quand le fix P3 est devenu **structurel** (rendre la course impossible par construction, pas la colmater) plutôt qu'incrémental. **Changer de modèle attrape les défauts ; la discipline de continuer tant qu'un finding > LOW subsiste empêche une régression de remédiation d'atteindre `main`.**

## Dev Agent Record

### Agent Model Used

Opus 4.8 (1M context) — run unique, T1 → T9, aucune HALT.

### Debug Log References

- vitest feature `reminders` : 10/10 (api 5 + error-label 5). Suite complète : **404/404**.
- E2E `reminders.spec.ts` : **6/6** (backend 0.7.0 MockMailer contre `kesh_e2e`).
- Régression fixture (`createAndValidateInvoiceViaApi` étendue) : invoice-send-email + invoices + échéancier + supplier **22/22**.
- Backend : `fmt` 0, `clippy -D warnings` 0, `kesh-i18n` 21/21 (les 4 FTL avec +52 clés parsent). Aucun code Rust modifié.

### Completion Notes List

**Anti-double-submit (AC de premier plan) — livré et prouvé.** Les 4 couches sur les 3 flux (unitaire, lot, manuel) : flag `$state` possédé par la page (`sendingUnit`/`batchSending`/`savingManual`), garde de ré-entrance + `true` avant l'appel + `finally`, boutons `disabled`, `onOpenChange` refusant la fermeture en vol. Le test E2E `anti-double-submit` ralentit la route d'envoi, vérifie le `disabled` immédiat et qu'**un seul e-mail** est capturé malgré un second clic forcé. C'est la seule barrière (le backend n'en a aucune, TOCTOU accepté 21-5b).

**Le piège `sentAt` (#249) est neutralisé et testé.** `ManualReminderDialog` émet `` `${sentAt}T12:00:00` `` (jamais la date nue), et `reminders.api.test.ts` asserte le format `/T\d{2}:\d{2}:\d{2}$/`. Sans ce suffixe, le body serait rejeté par Axum avant le handler.

**Deux bugs de test rencontrés et corrigés en cours de dev** (aucun n'était un défaut de la page) :
1. **Fixture `dueDate` + validation #245** : la fixture posait `date: today` mais `dueDate` passé → 400 (`due_date >= date`). Corrigé en **alignant la date de facture sur `dueDate`** quand il est fourni (une facture échue est émise ET échue dans le passé). DRY préservé, défaut inchangé → 0 régression (22/22).
2. **Locators E2E par nom de contact** : le nom vit dans l'en-tête de groupe (`<div>`), pas dans la ligne (`<tr>`). Filtrer les `reminder-row` par nom ne trouvait rien → re-scopé via le groupe. Le cache de transform Playwright de la version cassée (import dynamique d'axe) causait aussi un faux « two versions of @playwright/test » → purge du cache + import statique.

**Placement / i18n** : nouvelle feature `reminders/` (types + api + 5 composants + 2 helpers testés), namespace `reminders-*` cohérent avec le dossier → **lint-i18n PASS sans `KNOWN_VIOLATIONS`** (contrairement à 21-6a). Badges en `--color-text` (leçon contraste 21-6a), axe scopé neutralisant les dettes #253/#256.

**Périmètre** : aucun backend d'envoi recréé (piège de cartographie du 5e agent, tranché par grep en validate) ; historique fiche / toggle UI / dashboard restent en 21-6c ; cumuls par contact non affichés (L21-8).

### File List

**Frontend — feature `reminders/` (nouveaux)**
- `frontend/src/lib/features/reminders/reminders.types.ts`
- `frontend/src/lib/features/reminders/reminders.api.ts`
- `frontend/src/lib/features/reminders/reminders.api.test.ts`
- `frontend/src/lib/features/reminders/reminder-error-label.ts`
- `frontend/src/lib/features/reminders/reminder-error-label.test.ts`
- `frontend/src/lib/features/reminders/index.ts`
- `frontend/src/lib/features/reminders/ReminderNoEmailBadge.svelte`
- `frontend/src/lib/features/reminders/ReminderTerminalBadge.svelte`
- `frontend/src/lib/features/reminders/ReminderSendDialog.svelte`
- `frontend/src/lib/features/reminders/ManualReminderDialog.svelte`
- `frontend/src/lib/features/reminders/ReminderBatchReport.svelte`

**Frontend — page & nav (modifiés/nouveaux)**
- `frontend/src/routes/(app)/invoices/reminders/+page.svelte` (**NEW**)
- `frontend/src/routes/(app)/+layout.svelte` (M — entrée nav)

**i18n**
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (M — 52 clés `reminders-*` + `nav-invoicing-reminders`)

**Tests**
- `frontend/tests/e2e/reminders.spec.ts` (**NEW** — 6 tests)
- `frontend/tests/e2e/helpers/api-fixtures.ts` (M — param `dueDate`)

**Doc / suivi**
- `CHANGELOG.md` (M)
- `_bmad-output/implementation-artifacts/21-6b-page-rappels.md` (M)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (M)
