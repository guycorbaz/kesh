# Epic 62 — Lignes de facture : titres de section et désignation longue au PDF

*Créé le 2026-11-27 par découpage de l'Epic 52 (décision du Project Lead du 2026-11-27 : l'Epic 52 se réduit à la 49-2,
réordonnancement, pour la v0.21.0 ; les deux autres stories passent ici).* Release visée : **v0.22.0** (milestone
`v0.22.0`, date à fixer au lancement). Branche `epic/62-lignes-de-facture-titres`, à créer depuis `main` au lancement
— après l'entrée de la v0.21.0 si elle est publiée d'ici là (fichiers communs avec la 49-2, ci-dessous). **Migration :
aucune annoncée** : le type `heading` et ses contraintes sont posés depuis la v0.20.0 (migration `20261118000002` de la
49-1a) ; la 49-4b n'écrit rien en base.

## Objet

Finir la mise en page des lignes de la facture client commencée par l'Epic 49 (#1242) :

- **titres de section** — une ligne de texte sans montant, hors du total et des écritures, rendue en évidence sur toute la
  largeur du PDF, reprise dans les avoirs ; ouverture du type `heading` dans l'API (retrait de la garde) ;
- **désignation longue au PDF** — retour à la ligne au lieu de la troncature à 44 caractères suivis de « … » (#1363), et
  caractères que la police n'a pas signalés ou repliés au lieu de disparaître (#1400).

Les **sous-totaux de section** restent hors de cet epic (#1330, milestone « Plus tard », décision de Guy du 2026-11-18).

## Stories

| Story | Issues | Détail |
|---|---|---|
| 49-3-titres-de-section | #1242 | titre de section sans montant : saisie à l'écran, rendu PDF, reprise dans les avoirs ; **ouvre `heading`** ; fiche à écrire (`bmad-create-story`) |
| 49-4b-designation-longue-retour-ligne | #1363, #1400 | retour à la ligne de la désignation, caractères repliés ou signalés ; porte la dérogation à « les factures déjà émises se réimpriment à l'identique » pour une facture jamais téléchargée ou un regel ; fiche existante (`49-4b-designation-longue-retour-ligne.md`, `Status: backlog`), **à revalider entière** avant le développement (ses points à traiter de la validation P1 de la 49-4) |
| 62-solde-dette-epic | plafond fixé au lancement | la dette des deux stories |

Trois stories : l'epic tient dans la borne de six.

## Dépendances

- Dépend de : **Epic 49** (49-1a, 49-1b, 49-1c, 49-4a), entré dans `main` avec la v0.20.0 (`0e99cd2db`) — fait. La 49-3
  part de la 49-1c (écran, mapper `invoice-lines.ts`) et de la 49-1b (rendu PDF) ; la 49-4b part de la 49-4a (même code de
  rendu, `kesh-qrbill/src/pdf.rs`).
- **Ne dépend pas de la 49-2** (Epic 52, v0.21.0) : un titre se place par l'ordre des ajouts si la 49-2 n'est pas livrée,
  et se déplace par ses boutons si elle l'est. Si la 49-2 est publiée avant, la 49-3 la reprend par `main` et vérifie qu'un
  titre se déplace comme un commentaire (même cellule d'actions).
- Bloque : aucune. #1242 se ferme avec la dernière story qui la porte (la 49-3, si la 49-2 est déjà publiée), #1330
  portant les sous-totaux.

## Fichiers et modules touchés (repris du fichier de l'Epic 49 et de la fiche de la 49-4b)

| Story | Backend | Frontend | Docs, i18n, tests |
|---|---|---|---|
| 49-3 | `kesh-db/src/entities/line_type.rs` (garde `open_to_api`), `kesh-api/src/routes/invoices.rs` (validation d'une ligne `heading`) ; `kesh-qrbill/src/pdf.rs` (rendu du titre) | `lib/components/invoices/InvoiceForm.svelte`, `lib/features/invoices/`, `lib/features/credit-notes/`, routes `invoices` et `credit-notes` | `kesh-i18n` ×4, `docs/api-external.md`, manuel utilisateur, tests |
| 49-4b | `kesh-qrbill/src/pdf.rs`, `largeurs.rs`, `empreinte.rs` ; `kesh-api/src/routes/issued_invoice_pdf.rs`, `credit_notes.rs` | à confirmer à la revalidation | `docs/api-external.md`, manuel utilisateur (phrases de la 49-1c à reprendre : « disparaît du PDF sans avertissement », conseil du trait d'union, capacité), tests de rendu avant / après |

**Fichiers communs** : `kesh-qrbill/src/pdf.rs` entre la 49-3 et la 49-4b (le second arrivé résout) ;
`InvoiceForm.svelte` et `docs/manual/fr/user-manual.tex` avec la **49-2** (Epic 52) ; catalogues i18n, `sitesTotal`,
`sprint-status.yaml` avec tous les epics. **Splitting** : la 49-3 touche au moins cinq modules au grain de la crate
(`kesh-db`, `kesh-api`, `kesh-qrbill`, frontend, `kesh-i18n`) : à recompter à sa fiche (CLAUDE.md § *Règle de splitting
préventif*).

## Contraintes de production

Guy émet de vraies factures dans Kesh depuis novembre 2026 : aucune facture ni aucun avoir déjà émis ne change (lignes,
totaux, TVA, écritures, PDF figés) ; la 49-4b porte une dérogation écrite pour les PDF non figés, à faire confirmer à sa
revalidation.

## Décomptes au kickoff

À relever au lancement de l'epic (issues ouvertes, bugs, P1, depuis GitHub), avec le décompte du tag précédent et
l'objectif (CLAUDE.md § *Priorités des défauts*).
