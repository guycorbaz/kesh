# Epic 21 — Échéances & relances débiteurs

**Statut** : draft (kickoff 2026-07-12) — EN ATTENTE de critique adversariale
**Issues GitHub** : #245 (conditions de paiement structurées — prérequis) + #231 (rappels débiteurs, balance âgée, postes ouverts)
**Cible release** : v0.7 (à confirmer)
**Séquence validée par Guy (2026-07-11)** : #245 d'abord (story courte), puis #231 qui s'appuie dessus.

## Objectif de l'epic

Donner à une PME suisse le **cycle complet de suivi des débiteurs** : des **échéances fiables** calculées depuis les conditions de paiement du contact (#245), une **liste des factures à rappeler** alimentée automatiquement (échéance + grâce), un **système de rappels multi-niveaux** configurable (délais + frais par niveau) avec **envoi manuel par e-mail** via le socle de templates Epic 20, un **historique de relance par facture**, et une **balance âgée** des postes ouverts débiteurs (0-30/31-60/61-90/90+).

**Hors scope explicite (décisions Guy 2026-07-11, commentaire #231)** :
- **Pas de scheduler / d'envoi automatique** en v1 : Kesh alimente la liste, l'utilisateur revoit et déclenche. (Cohérent avec l'architecture actuelle : aucun job runner n'existe, tout est requête-driven.)
- **Pas d'écriture comptable à l'émission d'un rappel** : les frais de rappel sont **affichés** (message + total réclamé) et ne sont comptabilisés que s'ils sont effectivement encaissés (via la réconciliation/écriture manuelle existantes).
- **Intérêts moratoires (art. 104 CO)** : hors périmètre v1.
- **PDF de lettre de rappel dédiée** : différé (cf. décision D8) — le rappel v1 = e-mail + PDF de la **facture d'origine** joint.

## Décisions figées (Guy, commentaire #231 du 2026-07-11 + cadrage #245)

### A. Conditions de paiement structurées (#245)

1. **Nouveau champ `contacts.default_payment_terms_days INT NULL`** (migration ADD COLUMN nullable, non-breaking, pattern `20260709000001_contacts_language_salutation.sql`). Le champ texte libre existant `default_payment_terms` (VARCHAR(100), `contact.rs:194`) est **conservé en lecture** pour les contacts non migrés.
2. **Libellé auto-généré** depuis le nombre de jours (« 30 jours net » / « Zahlbar innert 30 Tagen » / …), dans la **langue du contact** (fallback `instance_language`), copié dans `invoices.payment_terms` à la création — c'est ce libellé qui s'imprime sur le PDF (`kesh-qrbill/pdf.rs:374-383`, inchangé).
3. **Échéance par défaut = date facture + délai (jours) du contact**, **modifiable** par le créateur (pré-remplissage, pas une contrainte). Site backend : `invoices.rs:519-522` (aujourd'hui `due_date = unwrap_or(date)`, commentaire « pas de calcul auto — décision Guy » **caduc**, remplacé par cette décision). Site frontend : `InvoiceForm.svelte:239-241` (`onContactSelect` copie déjà le texte — y ajouter le calcul de `dueDate`).
4. **Proposition à valider en critique** : ajouter une validation `due_date >= date` (422) à la création/édition — elle n'existe nulle part aujourd'hui et une échéance antérieure à la facture fausserait tout le cycle de relance.

### B. Système de rappels (dunning)

5. **Niveaux configurables et extensibles** (pas de nombre figé). Chaque niveau porte : un **délai en jours depuis l'étape précédente** (échéance+grâce pour le niveau 1, rappel N-1 ensuite) et des **frais de rappel** (CHF, ≥ 0). Table `dunning_levels` company-scoped calquée sur le gabarit collection CRUD `vat_rates` (repo multi-tenant + sentinel lock company pour les invariants cross-row : numéros de niveau uniques et contigus).
6. **Période de grâce configurable** (jours, company-scoped, scalaire) — singleton `company_dunning_settings` calqué `company_invoice_settings` (get-or-create-default, verrou optimiste, audit).
7. **Zéro-config** (invariant hérité Epic 20) : des **niveaux par défaut** sont créés à la première utilisation (proposition : 3 niveaux — 1er rappel +10j/0 CHF, 2e rappel +10j/20 CHF, mise en demeure +10j/40 CHF ; grâce 5j) pour qu'une PME puisse relancer sans jamais ouvrir les réglages. Valeurs par défaut à challenger en critique.
8. **Alimentation automatique, envoi manuel** : une facture est « à rappeler » si `status='validated' AND paid_at IS NULL AND dunning non suspendu` et si le délai du prochain niveau est écoulé (niveau 1 : `today >= due_date + grâce + délai_niveau_1` ; niveau N : `today >= date_rappel_N-1 + délai_niveau_N`). Liste **calculée à la volée** (pas de scheduler). Le niveau courant d'une facture = nombre de rappels déjà envoyés (table d'historique).
9. **Exclusion/suspension par facture** (litige) : colonne `invoices.dunning_paused_at DATETIME NULL` (ADD COLUMN non-breaking) + action toggle auditée. Facture suspendue = jamais dans la liste, historique conservé.
10. **Historique par facture** : nouvelle table **`invoice_reminders`** (append-only : `company_id`, `invoice_id` FK, `level_number`, `sent_at`, `sent_to`, `subject` snapshot, `fee_amount` snapshot, acteur) — **indispensable** car `invoices.emailed_at`/`emailed_to` sont écrasés à chaque envoi et `audit_log` n'est pas requêtable pour piloter un cycle. Affichage sur la fiche facture. + audit `invoice.reminder_sent`.
11. **Frais affichés, non comptabilisés** : le total réclamé dans le rappel = **TTC de la facture + frais cumulés des niveaux 1..N**. ⚠️ Piège identifié : `invoices.total_amount` stocke le **HT** — le TTC doit être recalculé depuis les lignes (`line_vat_amount`, pattern `repositories/invoices.rs:993-1063`) ou lu sur le débit créance de l'écriture. **Le QR de la facture jointe n'inclut PAS les frais** (documenté manuel + mention dans le corps par défaut).

### C. Canal e-mail (réutilisation socle Epic 20)

12. **Nouveau type de template `invoice_reminder`** dans `EmailTemplateType` : 6 sites Rust à étendre (`email_template.rs:25-99`), 4 défauts FR/DE/IT/EN (`email_template_defaults.rs` — le test `all_defaults_only_use_allowed_variables` garde la cohérence), **nouvelle migration** pour étendre le CHECK `template_type` (`20260708000001_email_templates.sql:42-43`). Le repository `list_effective_for_company` suit automatiquement.
13. **Variables du type `invoice_reminder`** : les 6 existantes (`salutation`, `contactName`, `invoiceNumber`, `amount`, `dueDate`, `companyName`) + `reminderLevel`, `reminderFee`, `totalDue` (TTC + frais cumulés), `daysOverdue`. Builder `build_reminder_vars()` calqué `build_invoice_vars()` (`invoice_email.rs:151-186`), mêmes pré-formatages suisses, même `salutation_line()`.
14. **Même mécanique d'envoi que les factures** : destinataire **verrouillé** = `contacts.email`, langue/civilité du contact, preview serveur rendue / envoi du subject+body édités, PDF de la facture joint via `invoice_pdf_service::render` (réutilisé tel quel — contraintes héritées : facture `validated`, ≤ 9 lignes, compte primary, adresses structurées), garde 412 SMTP, rate-limit (limiteur dédié même pattern), scoping anti-IDOR, marquage + audit dans la même transaction.
15. **Refactor page settings/email-templates** : elle est **mono-type codée en dur** (`+page.svelte:48,90-94` — ignore tout type au-delà du premier) → la passer multi-type (sélecteur de type + libellés i18n par type).
16. **Envoi unitaire ET par lot** (décision Guy « unitaires ou lot ») : l'endpoint batch suit le **pattern canonique `{ accepted, failed }` / `FailedProposal` per-proposal** (CLAUDE.md §Pattern batch — identifiant business = `invoice_id`, `error_code` canonique, HTTP 200 même en succès partiel). Erreurs per-facture typiques : e-mail contact manquant, facture non-PDF-ready, payée entre-temps, suspendue.
17. **Micro-amélioration embarquée** (reste Epic 20) : ajouter un **log INFO** sur envoi e-mail réussi (facture ET rappel) — aujourd'hui un envoi réussi ne laisse aucune trace dans le log fichier (audit DB seulement).

### D. Question de design OUVERTE — ton par niveau vs modèle unique

**À trancher en critique adversariale** (mandat Guy). Les niveaux sont dynamiques, les types de template statiques.

- **Option A (recommandation draft)** : **modèle unique `invoice_reminder`** avec variables (`{reminderLevel}`, `{reminderFee}`, `{totalDue}`, `{daysOverdue}`…). Le ton s'ajuste : (a) à l'envoi unitaire, l'utilisateur édite subject/body avant envoi (mécanique Epic 20 existante) ; (b) le défaut reste neutre-ferme, utilisable à tout niveau. **Pour** : réutilise intégralement le sous-système Epic 20 (validation au save, défauts 4 langues, audit, versioning, UI) ; pas d'explosion UI niveaux × 4 langues. **Contre** : l'envoi par lot part du template brut → pas d'escalade de ton automatique ; une mise en demeure a un registre juridique différent d'un 1er rappel courtois.
- **Option B** : **texte par niveau porté par `dunning_levels`** (× 4 langues). **Pour** : escalade de ton native, y compris en lot. **Contre** : bypasse le sous-système de templates (re-implémenter validation/défauts/audit/versioning ou les dupliquer), UI lourde (N niveaux × 4 langues × subject+body), migration des défauts par niveau.
- **Option C (médiane)** : modèle unique + **champ optionnel « texte d'intro » par niveau** injecté via une variable `{levelIntro}` — le gros du corps reste dans le template, seul le paragraphe d'escalade varie par niveau.

### E. Balance âgée & postes ouverts

18. **Extension de l'existant** : l'échéancier (Story 5.4 — `PaymentStatusFilter`, `due_dates_summary`, page `invoices/due-dates`, export CSV) couvre déjà « postes ouverts ». La **balance âgée** = nouveau rapport dans **`kesh-report`** (pattern `balance_sheet.rs` : struct Serialize camelCase, `generate(pool, company_id, …)`, route `GET /api/v1/reports/aged-receivables`, export CSV, vue frontend dans `reports/`). Buckets par retard : **0-30 / 31-60 / 61-90 / 90+ jours** (par rapport à `due_date`), lignes par contact + totaux, montants **TTC** (recalcul depuis les lignes, cf. D11). Audit `report.viewed` comme les autres rapports.
19. **Le solde dû est binaire en v0.x** (pas de paiement partiel : `paid_at` tout-ou-rien, avoir = annulation totale → `cancelled`) : la balance âgée liste le TTC entier de chaque facture ouverte. Limitation documentée (L21-x) — un futur modèle de paiements partiels raffinera.

## Découpage en stories (draft — à confirmer post-critique)

- **21-1 — Conditions de paiement structurées (#245)** : migration `contacts.default_payment_terms_days` + entité/repo + routes + libellé auto localisé + pré-calcul échéance (backend `invoices.rs:522` + frontend `InvoiceForm.svelte`) + champ sur la fiche contact + validation `due_date >= date` (si retenue). Story courte self-contained, ferme #245.
- **21-2 — Socle configuration rappels (backend)** : table `dunning_levels` (CRUD Admin, gabarit vat_rates, invariants cross-row sous sentinel lock) + singleton `company_dunning_settings` (grâce) + défauts zéro-config + type `invoice_reminder` (enum + 4 défauts + migration CHECK + variables).
- **21-3 — Réglages rappels + templates multi-type (frontend)** : page `settings/dunning` (liste niveaux CRUD + grâce, gabarit vat-rates) + refactor `settings/email-templates` multi-type + section index settings.
- **21-4 — Liste à rappeler + envoi de rappels (backend)** : requête « factures à rappeler » (niveau suivant + délais, à la volée) + table `invoice_reminders` (historique append-only) + `invoices.dunning_paused_at` + endpoints : liste, envoi unitaire, **envoi lot `{accepted, failed}`**, suspension toggle + `build_reminder_vars` + preview + audit + log INFO (D17).
- **21-5 — Relances débiteurs (frontend)** : page « Rappels » (liste à rappeler, sélection, envoi unitaire avec modale éditable / lot, badge niveau) + historique des rappels sur la fiche facture + toggle suspension.
- **21-6 — Balance âgée (rapport)** : `kesh-report::aged_receivables` + route + export CSV + vue frontend reports. (Indépendante de 21-2..5 — ne dépend que de `invoices` ; peut se paralléliser.)
- **21-7 — Doc + E2E** : manuels admin/user (réglages rappels, cycle de relance, « frais non compris dans le QR », balance âgée) + E2E Playwright round-trip (MockMailer : config niveaux → facture échue → liste → envoi rappel → historique ; balance âgée) + **A4 hérité Epic 20 : re-pointer `credit-notes.spec.ts` sur `api-fixtures.ts`**.

**Ordre** : 21-1 → 21-2 → (21-3 ∥ 21-4) → 21-5 → 21-7 ; 21-6 flottante (après 21-1).

> **Règle de splitting préventif** (CLAUDE.md) : chaque story ci-dessus touche ≤ 5 modules. 21-4 est la plus chargée (kesh-db entities+repo+migrations, kesh-api routes) — si la spec dépasse, split 21-4a (données+liste) / 21-4b (envoi).

## Limitations documentées (draft)

- **L21-1** — Pas de paiement partiel : le cycle de relance et la balance âgée raisonnent en tout-ou-rien (`paid_at`). Un acompte reçu n'arrête pas la relance (contournement : suspendre la facture). Remédiation = futur modèle de paiements partiels (hors epic).
- **L21-2** — Frais de rappel non comptabilisés à l'émission et **non inclus dans le QR** de la facture jointe (décision Guy) : le débiteur qui paie le QR paie le montant original. Documenté manuel + corps par défaut.
- **L21-3** — Contraintes PDF héritées (L20-2) : un rappel ne peut joindre le PDF que si la facture est `validated` ≤ 9 lignes avec adresses/compte OK. Rappel sans pièce jointe impossible en v1 → erreur per-facture explicite.
- **L21-4** — Pas d'envoi automatique planifié (pas de scheduler) : décision produit v1, pas une dette.
- **L21-5** — « Envoyée » = remise SMTP (hérite L20-3), pas d'accusé de réception ni suivi des bounces.

## Contexte technique (recherche 4 agents, 2026-07-12)

- **Contact/échéance** : `default_payment_terms` texte libre existe (`contact.rs:194`, VARCHAR(100)) ; backend `due_date = unwrap_or(date)` (`invoices.rs:519-522`) ; copie texte à la sélection contact (`InvoiceForm.svelte:239-241`) ; PDF imprime `payment_terms` seul (`kesh-qrbill/pdf.rs:374-383`) ; **aucune validation `due_date >= date`** ; pattern migration `20260709000001` (ADD COLUMN nullable + CHECK).
- **Socle e-mail Epic 20** : `EmailTemplateType` mono-variant, extension = 6 sites + defaults + migration CHECK ; moteur pur kesh-core (`extract_tokens`/`validate_tokens`/`render`) ; flux send (`invoice_email.rs:230-373`) : company → rate-limit → 412 SMTP → facture scopée → contact actif → destinataire verrouillé → langue → PDF → send → mark+audit tx (best-effort si facture disparue) ; **preview rendue serveur, send = payload édité brut** ; `invoice_pdf_service::render(pool, i18n, locale, &Company, invoice_id)` réutilisable tel quel ; page settings email-templates **mono-type hardcodée** ; **pas de table d'historique d'envois** (`emailed_at`/`emailed_to` écrasés).
- **État de paiement** : statuts `draft/validated/cancelled` + `paid_at` nullable (booléen, CHECK `paid_at ⇒ validated|cancelled`) ; réconciliation Epic 8 pose `paid_at` (`reconciliation.rs:1173-1221`), lien tx↔facture indirect via `journal_entries` ; helper `is_invoice_overdue` (`invoices.rs:196-200`) ; **échéancier existant** : `PaymentStatusFilter{All,Paid,Unpaid,Overdue}`, `due_dates_summary` (sans buckets d'âge), page `invoices/due-dates` + export CSV, index `idx_invoices_due_date (company_id, status, due_date)` ; **`total_amount` = HT**, TTC = recalcul lignes (`line_vat_amount`, `repositories/invoices.rs:993-1063`) ou débit créance ; avoirs = annulation totale (`UNIQUE(invoice_id)`, facture → `cancelled`), pas de solde partiel.
- **Settings/PDF/rapports** : gabarit collection = `vat_rates` (CRUD Admin + version + sentinel lock cross-row + feature frontend) ; gabarit singleton = `company_invoice_settings` ; `kesh-qrbill` printpdf sans moteur de texte libre (word-wrap inexistant — lettre PDF dédiée = chantier, différée) ; gabarit rapport = `kesh-report/balance_sheet.rs` + `routes/reports.rs` + vue frontend ; `format_swiss_amount`/`format_swiss_date` ; **aucun scheduler** (tout requête-driven).

## Risques identifiés (pré-critique)

- **R1 — HT/TTC** : toute confusion entre `total_amount` (HT) et le TTC dû fausse `{amount}`/`{totalDue}`/balance âgée. Traiter comme invariant testé (unit + intégration).
- **R2 — Éligibilité multi-niveaux** : le calcul « prochain niveau + date d'éligibilité » dépend de l'historique — la reconfiguration des niveaux (suppression/renumérotation) après des envois doit rester cohérente (les snapshots `invoice_reminders.level_number`/`fee_amount` protègent l'historique, mais la règle pour « et maintenant, quel niveau ? » doit être définie).
- **R3 — Concurrence lot** : un envoi par lot pendant qu'une réconciliation pose `paid_at` → re-check per-facture dans la transaction d'envoi (facture payée entre-temps = `failed[]`, pas d'e-mail).
- **R4 — Le refactor settings/email-templates multi-type** ne doit pas casser l'édition `invoice_send` existante (E2E sentinelle).
