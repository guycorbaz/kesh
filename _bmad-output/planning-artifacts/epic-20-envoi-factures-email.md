# Epic 20 — Templates d'e-mail (métier) & envoi des factures

**Statut** : in-progress (kickoff 2026-07-07)
**Issue GitHub** : #224 (envoi factures) + généralisation « templates d'e-mail métier ».
**Cible release** : v0.6 (à confirmer)
**Recadrage** : plan révisé le 2026-07-07 après **critique adversariale de 3 agents** (architecture RECADRER, sécurité RECADRER, produit GO-avec-ajustements). Décisions ci-dessous = version post-critique.

## Objectif de l'epic

Mettre en place un **sous-système de templates d'e-mail métier** — **company-scoped**, multilingue (FR/DE/IT/EN), par **type d'envoi**, configurable via une **section Admin dédiée** — et livrer son premier consommateur : l'**envoi d'une facture validée par e-mail** (PDF QR-facture joint, message personnalisé). Fondation extensible pour les rappels et l'envoi automatique des factures récurrentes (#223).

**Hors scope explicite (décision post-critique + Guy)** : le mail de **récupération de mot de passe reste inchangé** (FTL/Fluent, instance-level, non éditable). Il n'entre PAS dans ce sous-système — il est instance-scoped (aucune company au point d'envoi, cf. `auth.rs:815`), et le rendre éditable créerait un risque de DoS d'authentification + phishing pour une valeur nulle (unanimité des 3 agents). Rendre le recovery configurable sera **traité éventuellement plus tard** (issue dédiée, sujet sécurité à part entière) **si le besoin se confirme** au dogfooding (Guy 2026-07-07).

## Décisions figées (Guy + recadrage agents, 2026-07-07)

### Transport SMTP
1. **Réutiliser le SMTP de 17-4b** (vars d'env `KESH_SMTP_*`, module `crates/kesh-api/src/mail/`). Étendre le trait `Mailer` (aujourd'hui `send_password_reset` seul, texte seul) pour : envoi **avec objet+corps fournis** + **pièce jointe PDF** (`MultiPart::mixed`, lettre 0.11 feature `builder` déjà active).
2. **Identité expéditrice résolue à l'envoi** (pas codée en dur) : en v1, `From` = `KESH_SMTP_FROM` **avec display-name = nom de la société** (lever l'interdiction actuelle du display-name, `config.rs:1105`) + **`Reply-To` = e-mail de la société**. Per-société complet = limitation L20-1 (multi-tenant #199).
3. **Découpler le gate SMTP de `KESH_FEATURE_FORGOT_PASSWORD`** (`config.rs:154`) : une PME doit pouvoir envoyer des factures sans activer le recovery. Introduire un signal « SMTP configuré » (helper `Config::smtp_configured()`). **Dégradation gracieuse** : si SMTP non configuré → bouton « Envoyer par e-mail » **grisé** + tooltip (pas de fail-fast boot pour l'envoi facture ; le fail-fast reste réservé au flag recovery existant).

### Sous-système de templates (company-scoped)
4. **Table dédiée `email_templates`** : `id PK`, **`company_id BIGINT NOT NULL`** + FK CASCADE (calqué `bank_profiles.sql:24-37`), `type`, `language`, `subject TEXT`, `body TEXT`, **`version INT` (verrou optimiste)**, timestamps. **`UNIQUE (company_id, type, language)`** (pas de NULL → unicité fiable en MariaDB). Migration **non-breaking** (ADD TABLE) → **pas de bump `kesh_version_min_required`** + ligne dans `docs/migrations-idempotence-audit.md` (garde-fou P5).
5. **Enum de types v1 = `invoice_send` uniquement**. Futurs : `invoice_reminder`, `recurring_invoice` (#223), `offer_send` (module devis non prévu). Chaque type **déclare ses variables autorisées** (facture : `{salutation}`, `{contactName}`, `{invoiceNumber}`, `{amount}`, `{dueDate}`, `{companyName}`…).
6. **Moteur = substitution maison `{var}`** (PAS Fluent : Fluent ignore silencieusement les variables inconnues → invalidable ; et mélanger `{ $var }` FTL et `{var}` serait incohérent). **Single-pass** (une passe regex token→valeur, jamais re-scanner le résultat → pas d'injection par double-substitution). **Validation au SAVE** : chaque token `{…}` de subject+body ∈ set déclaré du type, sinon 422 listant les inconnues. **Rendu infaillible** à l'envoi (token inconnu → laissé littéral, jamais d'`Err`). Variables **pré-formatées suisse** (`formatting.rs` : apostrophe U+2019, dates dd.mm.yyyy) avant substitution.
7. **Défauts** : textes par défaut FR/DE/IT/EN en **constantes Rust** (ou FTL dédiées emails-company, syntaxe `{var}`). Si aucune ligne d'override en base pour (company, type, langue) → fallback défaut. **« Restaurer le défaut »** = supprimer la ligne d'override.
8. **Sémantique de sauvegarde** : édition par (type, langue) ; un save partiel (une langue) **ne wipe pas** les autres langues (préservation par-ligne, classe du bug #216). **Audit** `email_template.updated` (before/after) + court-circuit no-op (KF-004) + verrou optimiste `version`.
9. **Texte seul** (`text/plain`, comme le socle) — pièce jointe PDF à part. **HTML explicitement hors scope** Epic 20 (documenté).

### Section Admin de configuration
10. **Section Admin dédiée « Modèles d'e-mail »** (`Paramètres → Modèles d'e-mail`, **Admin-only**) : liste des types ; par type, éditeur **multilingue** (onglets FR/DE/IT/EN) objet + corps, **panneau des variables** du type, action **« restaurer le défaut »**. Distincte de la config SMTP transport (env vars, manuel admin). Invariant testé : **une PME doit pouvoir envoyer une facture correcte sans jamais ouvrir cette section** (fallback défaut zéro-config).

### Langue & civilité (contact)
11. **Langue par contact** : champ `language` sur `contacts` (CHECK `IN ('FR','DE','IT','EN')`, calqué `companies.instance_language`), défaut = `companies.instance_language`. Porte sur le **contact principal** de la facture. Le corps **ET le PDF** sont rendus dans cette langue (passer la locale contact au service PDF, `invoice_pdf.rs:107` accepte déjà un param locale).
12. **Civilité/genre par contact** : champ `salutation` sur `contacts` (enum : `Monsieur` / `Madame` / `Neutre`, défaut `Neutre`). Variable `{salutation}` résolue **genre × langue × type** (Personne : « Cher Monsieur » / « Chère Madame » / « Sehr geehrter Herr »… ; Entreprise : « Madame, Monsieur » / neutre). Sans civilité renseignée → formule neutre (« Bonjour », « Madame, Monsieur »).

### Envoi de facture
13. **Destinataire VERROUILLÉ = `contacts.email`** (refus propre si absent). **NON éditable dans la modale** (sécurité : un `to` éditable = exfiltration du PDF financier + relais de spam authentifié). Pour changer l'adresse → éditer le contact. Seuls **objet + corps** sont éditables à l'envoi manuel.
14. **Pièce jointe = PDF QR-facture**, via un **service factorisé** `(company_id, invoice_id, locale) → Result<Vec<u8>, AppError>` (extrait du handler `invoice_pdf.rs`). Contraintes héritées : facture **validée**, ≤ 9 lignes, contact + adresse structurée, compte bancaire **primary**. Le service porte les ≥6 variantes d'erreur → remontent proprement.
15. **Endpoint `POST /api/v1/invoices/{id}/send-email`** dans `comptable_routes` (**Comptable+**). **Scoping company obligatoire** (`get_company_for` + fetch facture filtrée `company_id` → anti-IDOR). **Rate-limit** par user+company (réutiliser `rate_limiter`). Body éditable = `{ subject, body }` uniquement (pas `to`).
16. **Marquage « envoyée »** : `invoices.emailed_at` (nullable, ADD COLUMN non-breaking) + destinataire ; **renvoi autorisé** (bouton « Renvoyer » si déjà envoyée ; écrase `emailed_at`, chaque envoi audité). **Audit `invoice.emailed`** avec le `to` réellement envoyé + objet. Facture **non** marquée si échec SMTP (`AppError::SmtpSendFailed` → 500, texte clair).

### Documentation
17. Manuel **admin** : SMTP sert aussi aux factures + **avertissement délivrabilité** (« utilisez le SMTP de votre fournisseur de messagerie, dont le domaine est déjà authentifié SPF/DKIM ; n'usurpez pas un domaine tiers ») + section « Modèles d'e-mail ». Manuel **user** : bouton « Envoyer par e-mail », config templates, **« envoyée = remise au serveur SMTP, pas accusé de réception »**. Régénérer les PDF (gate release 4-bis au tag).

## Découpage en stories (révisé post-critique)

- **20-1 — Socle templates d'e-mail (backend)** : table `email_templates` (company_id NOT NULL) + enum type (`invoice_send`) + déclaration variables + moteur `{var}` single-pass + validation au save + rendu infaillible + défauts + résolution override→défaut + repo (version, audit, no-op) + CRUD Admin. **Story-zéro**. (Recovery NON touché.)
- **20-2 — Section Admin « Modèles d'e-mail » (frontend)** : page `settings/email-templates` (Admin-only), éditeur multilingue par type, aide variables, restaurer défaut. Consomme 20-1.
- **20-3a — Service PDF factorisé** : `invoice_pdf_service::render(pool, company_id, invoice_id, locale) → Result<Vec<u8>, AppError>` (déplace loading + 4 validations + mapping + generate ; `get_invoice_pdf` devient un thin wrapper). Story mécanique, revue file-by-file.
- **20-3b — Envoi de facture par e-mail** : extension mailer (générique + pièce jointe + display-name + Reply-To) ; champs `contacts.language` + `contacts.salutation` (+ UI fiche contact) ; découplage gate SMTP + `smtp_configured()` ; endpoint `POST send-email` (Comptable+, rate-limité, scoping, destinataire verrouillé) ; `invoices.emailed_at` + audit + renvoi ; bouton + **modale éditable (objet/corps seulement)**. Consomme 20-1 + 20-3a.
- **20-4 — Doc + E2E** : admin-manual (SMTP+délivrabilité+templates) + user-manual (bouton+config+« envoyée≠reçue ») + E2E round-trip (`MockMailer`, gate rôle Comptable+ 200 / Consultation 403 + destinataire verrouillé + fallback zéro-config).

**Ordre** : 20-1 → (20-2 ∥ 20-3a → 20-3b) → 20-4.

> **Règle de splitting préventif** : > 5 modules → split obligatoire (CLAUDE.md). PDF factorisé sorti en 20-3a (sinon 20-3 surchargée).

## Limitations documentées

- **L20-1** — Expéditeur global (`KESH_SMTP_FROM`, display-name société ajouté) : pas d'adresse d'expédition par-société. Mono-instance OK (relais de l'opérateur, SPF/DKIM alignés). Multi-tenant (#199) : enjeu délivrabilité = **auth de domaine (SPF/DKIM/DMARC)**. Patterns **(A)** domaine-plateforme + `Reply-To` tenant (valider Reply-To/display-name contre usurpation) vs **(B)** SMTP+`From` par tenant (credentials en base **chiffrés at-rest**, jamais renvoyés/loggés). Hook « identité résolue à l'envoi » déjà posé → interdire dès v1 un `From`/display-name/`Reply-To` fourni par un rôle non-opérateur.
- **L20-2** — Contraintes PDF héritées : facture validée, ≤ 9 lignes (mono-page A4).
- **L20-3** — SMTP sortant simple : « envoyée » = remise au serveur SMTP, **pas** accusé de réception ; les bounces asynchrones ne sont pas remontés (documenté).
- **Différé (non-v1)** : BCC/archivage expéditeur, envoi groupé (batch → futur endpoint `{accepted, failed}` pattern CLAUDE.md), gestion structurée des bounces, HTML, per-société sender.

## Synthèse critique adversariale (3 agents, 2026-07-07)

- **Architecture (RECADRER)** : erreur de modèle `(company_id, type, language)` pour recovery (instance-scoped, `auth.rs:815`) → `company_id NOT NULL` + recovery hors socle ; Fluent inutilisable pour valider les variables (`loader.rs:14` ignore l'inconnu) → moteur `{var}` + validation au save ; factorisation PDF sous-estimée → story 20-3a dédiée ; `version`+audit manquants ; retirer `offer_send` v1.
- **Sécurité (RECADRER)** : destinataire éditable = exfiltration/relais spam → **verrouiller** `contacts.email` ; pas de rate-limit → **ajouter** (sinon blacklist domaine casse aussi le recovery) ; recovery éditable = DoS auth/phishing → **hors socle** ; single-pass + rendu infaillible + IDOR scoping. Socle actuel sain (builder typé anti-CRLF, secrets protégés, anti-énumération, text-only).
- **Produit (GO ajusté)** : civilité/genre manquant → « Cher Monsieur » impossible (DE-CH) → champ `salutation` ; `From` sans display-name + pas de `Reply-To` → à lever ; gate SMTP couplé au recovery + fail-fast boot → découpler + dégradation gracieuse ; renvoi ; doc délivrabilité + « envoyée ≠ reçue ». Invariant : fallback défaut zéro-config testé.

## Contexte technique (recherche 4 agents, 2026-07-07)

- **Mailer** : `mail/mod.rs` trait objet-safe (`Arc<dyn Mailer>`), `SmtpMailer`/`NoopMailer`/`MockMailer`. lettre 0.11 rustls/STARTTLS, `MultiPart`/`Attachment` (feature `builder` OK). `AppError::SmtpSendFailed`→500. `from` parsé au boot (`smtp.rs:51`), display-name refusé (`config.rs:1105`). Builder typé = anti-injection CRLF (à préserver).
- **PDF** : `kesh_qrbill::generate_qr_bill_pdf(&QrBillData,&InvoicePdfData,&QrBillI18n)→Result<Vec<u8>>` pur. Handler `invoice_pdf.rs:44-265` : 5 chargements DB + 4 validations + mapping privé `build_qrbill_inputs` + i18n `state.config.locale`. Locale param possible.
- **Settings/i18n** : `company_invoice_settings` (aucun template email). `kesh-i18n` Fluent, `Locale{FrCh,DeCh,ItCh,EnCh}`, fallback FrCh, **ignore les variables inconnues** (`loader.rs:14-19`). Runtime locale globale (`config.locale`). `companies.instance_language`/`accounting_language` CHAR(2) CHECK. Précédent JSON validé `bank_profiles`. Aucun éditeur multilingue frontend.
- **Front/routes** : fiche `invoices/[id]/+page.svelte` (`canManage` Comptable+, modales Dialog). Handler modèle `mark_invoice_paid_handler`. Route Comptable+ dans `comptable_routes` (`lib.rs`, path `{id}`). E2E modèle `invoice_delete_e2e.rs` + `MockMailer`. `contacts` : `contact_type`, `first_name`/`last_name` (#213), `email` Option, **pas de genre/civilité**. `rate_limiter_recovery` (`auth.rs:650`) = modèle rate-limit.
