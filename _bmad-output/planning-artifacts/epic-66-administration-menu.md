# Epic 66 — Administration et menu : l'e-mail d'essai et le menu en accordéon

*Kickoff du 2026-11-28 (décision du Project Lead du 2026-11-28). **Release intermédiaire du mardi 2026-12-01**
(milestone « Mardi 2026-12-01 » ; numéro attribué au tag — une route neuve sans migration ni rupture fait monter `c`).
Branche `epic/66-administration-menu`, partie de `main` à `287deea33` (la v0.20.1). **Aucune migration. Aucune rupture
de l'API** (une route neuve seulement).*

## Objet

1. **Vérifier la configuration de l'envoi des factures par e-mail avant le premier envoi réel** (#1771, CR, P2) :
   un e-mail d'essai, réservé à l'Administrateur, vers une adresse saisie librement, rendu par les modèles d'e-mail
   et accompagné d'une QR-facture « SPÉCIMEN » ; rien n'est écrit dans les livres.
2. **Un menu latéral en accordéon** (#852 P3, #959 P4) : un seul groupe déplié à la fois, et le groupe déplié d'office
   par la page courante ne reste plus ouvert à tort en navigation interne.

## Décisions du Project Lead (2026-11-28)

- **D1** — l'e-mail d'essai ne peut être envoyé que par un **Administrateur** (Comptable et Consultation : 403) ;
- **D2** — une **clé d'API ne peut pas** l'envoyer, même créée par un Administrateur (`403 API_KEY_ADMIN_FORBIDDEN`,
  comme les autres réglages d'administration) ;
- **D3** — l'adresse de destination est une **saisie libre**, préremplie (l'adresse de réponse de la société, si elle
  existe) ;
- **D4** — la pièce jointe est une **QR-facture « SPÉCIMEN »** aux données fictives (option a) ;
- **D5** — cadence bornée (un essai toutes les 30 secondes environ) : un double clic n'envoie pas deux courriels ;
- **D6** — la 66-2 (menu) **sort de la release sans discussion** si elle n'est pas close le lundi 2026-11-30 au soir ;
  l'e-mail d'essai garde la priorité.

## Stories

| Story | Issues | Détail |
|---|---|---|
| 66-1-email-essai | #1771 (ferme) | route `POST` neuve sous `/api/v1/admin/…` (dans `admin_routes`), transport SMTP existant (`mail/smtp.rs`, `MockMailer` en mode test), modèles d'e-mail, PDF spécimen, audit, écran *Paramètres* (Administrateur seul), manuels FR, `docs/api-external.md` |
| 66-2-menu-accordeon | #852, #959 (ferment) | `frontend/src/routes/(app)/+layout.svelte` (groupes `<details>`, état mémorisé), tests Vitest et E2E du menu |

Deux stories : l'epic tient dans la borne de six. Pas de story de solde séparée : les LOW de revue se corrigent dans
les stories (règle du 2026-11-27) ; un reste hors périmètre s'ouvre en issue. Pas de story de correction : une release
intermédiaire n'en porte pas (CLAUDE.md § *Cadence* : elle concerne les releases planifiées du vendredi).

## Fichiers et modules touchés

- 66-1 : `crates/kesh-api` (route, `mail/`, audit), `crates/kesh-qrbill` (spécimen, si le PDF s'y construit),
  `crates/kesh-i18n` (quatre locales), frontend `settings` (domaine des paramètres), `docs/manual/fr/`
  (`admin-manual.tex`, `user-manual.tex`), `docs/api-external.md`.
- 66-2 : frontend, gabarit `(app)/+layout.svelte` seul (et ses tests).
- Aucun recouvrement entre les deux stories ; aucun avec l'Epic 53 (release du 2026-12-04), sauf les fichiers communs
  (catalogues i18n, `sprint-status.yaml`, compteurs des gardes), résolus à l'intégration.

## Dépendances

Aucune. La release du 2026-12-04 la suit : `epic/53-ecritures-justes` reprendra `main` après la publication.
