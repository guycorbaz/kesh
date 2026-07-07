# Epic 20 — Envoi des factures par e-mail

**Statut** : in-progress (kickoff 2026-07-07)
**Issue GitHub** : #224
**Cible release** : v0.6 (à confirmer)

## Objectif de l'epic

Permettre d'**envoyer une facture validée par e-mail** directement depuis Kesh (PDF QR-facture en pièce jointe), avec un **message personnalisable**, sans passer par un client mail externe et avec un suivi de l'envoi. Fondation de la communication client (préalable à l'envoi automatique des factures récurrentes, #223).

## Décisions figées (Guy, 2026-07-07)

1. **Réutiliser intégralement le SMTP de la story 17-4b** — variables d'environnement `KESH_SMTP_HOST/PORT/USER/PASSWORD/FROM/TLS`, module `crates/kesh-api/src/mail/`, fail-fast au boot. **Ne PAS créer de config SMTP parallèle** (« on ne réinvente pas la roue »).
2. **Config SMTP au niveau instance**, partagée entre tous les tenants (forward-compatible multi-tenant). Le *transport* reste instance-level ; seule l'*identité expéditeur* deviendra per-société au moment du multi-tenant.
3. **Expéditeur = `KESH_SMTP_FROM` global (v1)**. En mono-instance (réalité actuelle), l'opérateur y met sa propre adresse société → l'expéditeur *est* la société, envoyé via son propre relais → SPF/DKIM naturellement alignés, aucun souci de délivrabilité. **Forward-compat obligatoire** : l'identité expéditrice doit être **résolue au moment de l'envoi** (paramètre du pipeline), **pas codée en dur** → passer au multi-tenant devient un ajout de config, pas une réécriture. L'expéditeur **par-société** est une **limitation documentée** (cf. L20-1, territoire Issue #199).
4. **Destinataire = e-mail du contact** de la facture (champ `contacts.email`, déjà existant). Si le contact n'a pas d'e-mail → refus propre (message clair, pas de crash).
5. **Pièce jointe = le PDF QR-facture** déjà généré à la validation (réutiliser le générateur existant `invoice_pdf`).
6. **Message (objet + corps) configurable sous forme de template** : un **template par défaut multilingue** (FR/DE/IT/EN) est fourni et **configurable** dans les réglages de facturation (cohérent avec le « template de rappel » déjà mentionné). Support de **variables de personnalisation** (civilité/nom du contact, n° de facture, montant, date d'échéance…) permettant un texte type « Cher Monsieur xyz, veuillez trouver ci-joint la facture … ».
   - **Envoi manuel** : le template (variables résolues) est **pré-rempli dans la fenêtre d'envoi et éditable** avant expédition.
   - **Envoi automatique** (futur — factures récurrentes #223 / story 20-2) : le template est **appliqué tel quel, sans édition** (pas d'humain dans la boucle).
   - **Stockage** : le template par défaut (objet + corps, par langue) vit dans **`company_invoice_settings`** (mêmes réglages que le reste de la facturation, éditable Admin/Comptable+).
7. **Bouton « Envoyer par e-mail »** sur la fiche d'une **facture validée**, rôle **Comptable+** (Comptable et Administrateur ; cohérent avec « Créer un avoir »).
8. **Traçabilité** : marquer la facture **« envoyée »** (date/heure + destinataire) + entrée d'**audit** (`invoice.emailed` ou équivalent). Gestion des erreurs SMTP (serveur injoignable, adresse invalide) avec retour clair à l'utilisateur, sans laisser la facture dans un état incohérent.
9. **Documentation** : la config SMTP (`KESH_SMTP_*`) est **déjà documentée** dans le manuel admin (§ recovery 17-4b) — **étendre** la doc pour indiquer que ce même SMTP alimente désormais l'envoi des factures, et documenter côté user-manual le bouton « Envoyer par e-mail » + la configuration du gabarit de message. (Rappel Guy 2026-07-07.)

## Stories

### 20-1 — Envoi d'une facture validée par e-mail (#224)

**User story** : En tant que comptable/administrateur d'une PME, je veux envoyer une facture validée à mon client par e-mail (PDF joint, message personnalisé) directement depuis Kesh, afin de gagner du temps et de garder une trace de l'envoi.

**Acceptance criteria (BDD, à affiner en spec)** :
- **AC1** — Étant donné une facture validée dont le contact a un e-mail, quand j'ouvre la fiche et clique « Envoyer par e-mail », alors une fenêtre s'ouvre avec l'objet et le corps pré-remplis depuis le gabarit (variables résolues), le destinataire et le PDF en pièce jointe.
- **AC2** — Quand je confirme l'envoi, alors Kesh envoie l'e-mail via le SMTP configuré (17-4b), avec le PDF QR-facture en pièce jointe, et affiche un succès.
- **AC3** — Après un envoi réussi, la facture est marquée « envoyée » (date/heure + destinataire) et une entrée d'audit est créée.
- **AC4** — Lors d'un **envoi manuel**, je peux **éditer** l'objet et le corps du message avant l'envoi (le template n'est qu'un point de départ). *(Les envois automatiques futurs — 20-2 — appliqueront le template sans édition.)*
- **AC5** — Le template par défaut est **configurable** (réglages de facturation), multilingue FR/DE/IT/EN, avec variables de personnalisation documentées.
- **AC6** — Si le contact n'a pas d'e-mail → l'action est refusée avec un message clair (pas d'envoi, pas de crash).
- **AC7** — Si le SMTP n'est pas configuré (feature non activée) → le bouton est indisponible / l'action retourne une erreur explicite (cohérent avec le fail-fast recovery).
- **AC8** — En cas d'échec SMTP (serveur injoignable, refus), l'utilisateur voit une erreur claire et la facture **n'est pas** marquée « envoyée ».
- **AC9** — La doc (admin-manual : SMTP partagé recovery+factures ; user-manual : bouton + gabarit) est mise à jour dans la même PR.

**Notes techniques** :
- Réutiliser `crates/kesh-api/src/mail/` (mailer 17-4b) — étendre pour supporter une **pièce jointe** (PDF) si pas déjà supporté.
- Réutiliser le générateur `invoice_pdf` pour produire la pièce jointe.
- Nouveau champ de suivi sur `invoices` (ex. `emailed_at` + destinataire) — migration non-breaking (`ADD COLUMN` nullable).
- Endpoint type `POST /api/v1/invoices/:id/send-email` (Comptable+).
- Gabarit par défaut : stocké côté réglages facturation (`company_invoice_settings`) ou constantes i18n par défaut, éditable.

**Découpage envisagé** (à trancher en spec — potentiellement splittable si Medium-Large) :
- 20-1a : backend (mailer + attachment + endpoint + champ suivi + audit + gabarit défaut).
- 20-1b : frontend (bouton + fenêtre d'envoi éditable + réglage gabarit).
- 20-1c : doc + E2E.

### 20-2 (futur) — Envoi automatique des factures récurrentes

Consommera 20-1 quand #223 (factures récurrentes) sera implémenté. Hors scope de cet epic pour l'instant.

## Limitations documentées

- **L20-1** — Expéditeur unique global (`KESH_SMTP_FROM`) : pas d'identité expéditrice par-société. Acceptable en mono-instance (le `From` = l'opérateur, envoyé via son propre relais → SPF/DKIM alignés). À lever au **multi-tenant** (Issue #199), où l'enjeu principal est la **délivrabilité / le risque « spammeur »** — qui relève de l'**authentification de domaine (SPF/DKIM/DMARC)**, pas seulement du champ `From`. Deux patterns robustes à arbitrer le moment venu :
  - **(A) Expéditeur = domaine de la plateforme + `Reply-To` = tenant** — un seul domaine authentifié, nom du tenant en display-name. Modèle SaaS standard, meilleure délivrabilité, aucune config DNS par tenant. Le destinataire voit un domaine plateforme.
  - **(B) SMTP + `From` par tenant** — chaque tenant apporte son relais/ses identifiants et authentifie son propre domaine. Branding complet, mais config par tenant.
  Prérequis technique déjà posé en v1 (décision 3) : l'identité expéditrice est **résolue à l'envoi**, pas codée en dur → le passage à (A) ou (B) est un ajout de config, pas une réécriture du pipeline.
