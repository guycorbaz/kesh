# Epic 20 — Templates d'e-mail & envoi des factures

**Statut** : in-progress (kickoff 2026-07-07)
**Issue GitHub** : #224 (envoi factures) — la généralisation « templates d'e-mail » élargit le scope (décision Guy 2026-07-07).
**Cible release** : v0.6 (à confirmer)

## Objectif de l'epic

Mettre en place un **sous-système générique de templates d'e-mail** — multilingue (FR/DE/IT/EN), par **type d'envoi**, configurable via une **section Admin dédiée** — puis livrer ses premiers consommateurs : la **récupération de mot de passe** (migrée sur le socle) et l'**envoi d'une facture validée par e-mail** (PDF QR-facture joint). Fondation de toute la communication e-mail client/utilisateur (préalable aux rappels et à l'envoi automatique des factures récurrentes, #223).

## Décisions figées (Guy, 2026-07-07)

### Transport SMTP
1. **Réutiliser le SMTP de 17-4b** — vars d'env `KESH_SMTP_*`, module `crates/kesh-api/src/mail/`, fail-fast boot. **Pas de config parallèle.** Étendre le trait `Mailer` (aujourd'hui limité à `send_password_reset`, corps texte seul) pour : (a) un envoi **générique** (objet + corps rendus depuis un template), (b) une **pièce jointe** (PDF via `MultiPart::mixed` — lettre 0.11, feature `builder` déjà active).
2. **SMTP niveau instance, partagé multi-tenant.** L'**identité expéditrice est résolue au moment de l'envoi** (paramètre du pipeline), pas codée en dur. Expéditeur = `KESH_SMTP_FROM` global (v1). Per-société = **limitation documentée** L20-1 (multi-tenant #199 ; enjeu = délivrabilité/SPF/DKIM ; patterns A domaine-plateforme+Reply-To vs B SMTP-par-tenant).

### Sous-système de templates
3. **Table dédiée `email_templates`** (`company_id`, `type`, `language`, `subject`, `body`) — 1 ligne par (société, type, langue). PAS dans `company_invoice_settings` (les templates sont transverses, pas facturation-spécifiques). Précédent de migration non-breaking + JSON validé : `bank_profiles`.
4. **Enum de types d'e-mail** extensible : `password_reset`, `invoice_send` (v1), puis `invoice_reminder`, `recurring_invoice`, `offer_send` (**envoi d'offres/devis — module non encore prévu, Guy 2026-07-07** ; illustre la valeur du socle : un futur type = 1 variant + ses variables, zéro réécriture)… Chaque type **déclare ses variables autorisées** (recovery : `{resetUrl}`, `{ttlMinutes}` ; facture : `{contactName}`, `{invoiceNumber}`, `{amount}`, `{dueDate}`, `{companyName}`…).
5. **Défauts dans les FTL** (`crates/kesh-i18n/locales/*/messages.ftl`, pattern `email-password-reset-*` existant) : si aucun override en base pour (société, type, langue) → fallback sur le défaut FTL. Le rendu = substitution des variables (moteur Fluent ou substitution `{var}` maison, à trancher en spec).
6. **Recovery migré sur le socle** : `send_password_reset` passe par le système de templates (type `password_reset`), devient éditable. **Garde-fou sécurité** : le template `password_reset` **doit contenir `{resetUrl}`** (validation au save, refus sinon) — un admin ne doit pas pouvoir casser le flux de réinitialisation.

### Section Admin de configuration
7. **Section Admin dédiée « Modèles d'e-mail »** (`Paramètres → Modèles d'e-mail`, Admin-only) : liste des types ; par type, éditeur **multilingue** (onglets/sélecteur FR/DE/IT/EN) objet + corps, **panneau des variables** disponibles pour ce type, action **« restaurer le défaut »** (revient au FTL). Distincte de la config SMTP transport (env vars, manuel admin).

### Langue
8. **Langue par contact** : nouveau champ `language` sur `contacts` (préférence du client ; défaut = langue de la société `companies.instance_language`). L'e-mail de facture est rendu dans la langue du contact. En **envoi manuel**, éditable avant expédition ; en **envoi auto** (futur récurrentes), appliqué tel quel dans la langue du contact.

### Envoi de facture
9. **Destinataire** = `contacts.email` (refus propre si absent). **Pièce jointe** = PDF QR-facture (générateur `kesh_qrbill::generate_qr_bill_pdf` — **à factoriser** en service réutilisable `(company_id, invoice_id) → Vec<u8>`, aujourd'hui enfoui dans `invoice_pdf.rs`). Contraintes héritées du PDF : facture **validée**, ≤ 9 lignes, contact + adresse structurée, compte bancaire **primary** configuré.
10. **Bouton « Envoyer par e-mail »** sur la fiche facture validée, rôle **Comptable+**. Endpoint `POST /api/v1/invoices/{id}/send-email` dans `comptable_routes`. **Marquage « envoyée »** (`invoices.emailed_at` nullable + destinataire) + entrée **audit** (`invoice.emailed`). Erreurs SMTP claires (`AppError::SmtpSendFailed` → 500), facture **non** marquée envoyée si échec.
11. **Doc** : manuel admin (SMTP sert aussi aux factures + section Modèles d'e-mail) + manuel user (bouton « Envoyer par e-mail » + config des templates). SMTP transport déjà documenté (§ recovery 17-4b) → étendre.

## Découpage en stories

- **20-1 — Socle sous-système templates d'e-mail** (backend) : migration `email_templates` + enum types + déclaration des variables par type + résolution (override base → défaut FTL) + moteur de rendu (substitution + validation variables obligatoires, garde `{resetUrl}` recovery) + **migration du recovery sur le socle** (comportement inchangé, désormais template-driven) + repo/service + endpoints CRUD templates (Admin). Story-zéro qui pose le pattern.
- **20-2 — Section Admin « Modèles d'e-mail »** (frontend) : page `settings/email-templates` (Admin-only), éditeur multilingue par type (objet+corps), aide variables, restaurer défaut. Consomme 20-1.
- **20-3 — Envoi d'une facture par e-mail** : factorisation du service PDF → `Vec<u8>` ; extension mailer (envoi générique + pièce jointe) ; champ `contacts.language` (+ UI fiche contact) ; endpoint `POST send-email` (Comptable+) ; `invoices.emailed_at` + audit ; bouton + **modale d'envoi éditable** (destinataire/objet/corps pré-remplis depuis le template résolu, éditables). Consomme 20-1.
- **20-4 — Doc + E2E** : admin-manual + user-manual + E2E round-trip (`MockMailer`, inspection `sent()`), gate rôle (Comptable+ 200 / Consultation 403).

**Ordre** : 20-1 (socle) → 20-2 // 20-3 (parallélisables une fois le socle posé) → 20-4.

> **Règle de splitting préventif appliquée** : le besoin touche > 5 modules (kesh-db migration/entité/repo, kesh-qrbill/invoice_pdf refactor, kesh-api mail+routes, kesh-i18n FTL, frontend settings + fiche facture + fiche contact) → split obligatoire (cf. CLAUDE.md).

## Limitations documentées

- **L20-1** — Expéditeur unique global (`KESH_SMTP_FROM`) : pas d'identité expéditrice par-société. Acceptable en mono-instance (le `From` = l'opérateur, envoyé via son propre relais → SPF/DKIM alignés). À lever au **multi-tenant** (Issue #199), où l'enjeu principal est la **délivrabilité / le risque « spammeur »** — qui relève de l'**authentification de domaine (SPF/DKIM/DMARC)**, pas seulement du champ `From`. Deux patterns robustes à arbitrer :
  - **(A) Expéditeur = domaine de la plateforme + `Reply-To` = tenant** — un seul domaine authentifié, nom du tenant en display-name. Modèle SaaS standard, meilleure délivrabilité, aucune config DNS par tenant.
  - **(B) SMTP + `From` par tenant** — chaque tenant apporte son relais/ses identifiants et authentifie son propre domaine. Branding complet, config par tenant.
  Prérequis technique déjà posé en v1 (décision 2) : identité expéditrice **résolue à l'envoi** → passage à (A) ou (B) = ajout de config, pas réécriture.
- **L20-2** — Contraintes PDF héritées : facture validée uniquement, ≤ 9 lignes (mono-page A4). Une facture > 9 lignes ne peut être envoyée (comme elle ne peut être imprimée aujourd'hui).

## Contexte technique (recherche 2026-07-07, 4 agents)

- **Mailer** : trait `crates/kesh-api/src/mail/mod.rs` (objet-safe, `Arc<dyn Mailer>` dans AppState), impls `SmtpMailer`/`NoopMailer`/`MockMailer`. `lettre` 0.11 (rustls/STARTTLS), attachments via `MultiPart`/`Attachment` (feature `builder` déjà active). `AppError::SmtpSendFailed`→500. Pas de helper « SMTP configuré » → en ajouter un. `from` parsé au boot, cloné à l'envoi ; validation refuse le display-name (perso va dans le corps).
- **PDF** : `kesh_qrbill::generate_qr_bill_pdf(&QrBillData, &InvoicePdfData, &QrBillI18n) → Result<Vec<u8>>` (pur). Mapping DB→structs (`build_qrbill_inputs`, etc.) privé dans `invoice_pdf.rs` → factoriser. Données à charger : facture+lignes (`invoices::find_by_id_with_lines`), contact (`contacts::find_by_id`), company (`get_company_for`), compte primary (`bank_accounts::find_primary`), pays.
- **Settings/i18n** : `company_invoice_settings` (aucun template e-mail existant). `kesh-i18n` = Fluent, `Locale{FrCh,DeCh,ItCh,EnCh}`, `I18nBundle::format(locale,key,args)` fallback FrCh. **Locale runtime globale** (`config.locale`) — d'où le besoin d'une résolution par contact. `companies.instance_language`/`accounting_language` (CHAR(2)) existent. Emails transactionnels actuels = FTL (`email-password-reset-*`). Aucun contenu multilingue en base aujourd'hui (précédent JSON validé = `bank_profiles`). Aucun éditeur multilingue frontend → à créer.
- **Front/routes** : fiche `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (dérivé `canManage` Comptable+, modales Dialog, `invoices.api.ts`). Handler modèle `mark_invoice_paid_handler` (`routes/invoices.rs`). Route Comptable+ dans `comptable_routes` (`lib.rs`, path `{id}` Axum 0.8). E2E modèle `invoice_delete_e2e.rs` + `MockMailer`. PUT settings = Admin.
