# Prompt de validation P3 — fiches de l'Epic 38 (PayPal et socle des prestataires)

**Troisième passe** (modèle différent de P2, Opus) (`bmad-create-story validate`, `CLAUDE.md` § *Review Iteration Rule*), contexte frais, sur une
fiche du dépôt `<dépôt>` (branche `epic/38-paypal` ; Kesh en production : Rust/Axum, SvelteKit,
MariaDB). À lire d'abord :

- `_bmad-output/planning-artifacts/epics-38-40-cadrage.md` (décisions du responsable de l'epic **B1–B26**, qui priment) ;
- `_bmad-output/planning-artifacts/epic-38-paypal.md` : E*, C-38-*, fiches de stories, révisions, et la section
  **« Arbitrages de l'orchestrateur après la rédaction des fiches » (O1–O12), qui prime sur la fiche** — en particulier
  **O1** : l'unique migration breaking est en **38-2** (contrainte d'IBAN + `match_type`), plus en 38-3 ;
- la fiche visée (ta mission) et les fiches sœurs `38-*.md` pour les points de contact ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (ce que 37 livre) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8), *Pattern batch*, *Un appariement automatique propose, il ne crée
  jamais*, *Règle de splitting préventif*, *Recette visuelle*, manuels (le prompt d'une passe doit nommer le manuel).

## Ce que tu cherches

1. **Prémisse fausse au code** : vérifie chaque `fichier:ligne` cité ; un écart est un finding.
2. **Contradiction** avec B*, O*, E*, C-38-* ou une fiche sœur (noms de tables, colonnes, codes d'erreur, routes,
   fonctions, qui fait quoi, ordre des stories). Aligne la fiche sur O1–O12.
3. **Sites non résolus** (inventaire, ne suppose pas) : migration (P1–P8 : breaking ou non, `min_required`, P2-bis,
   P5 ligne d'audit d'idempotence et compteurs, P6 tests positionnels, P7 triage), sauvegarde/restauration
   (`TABLES_TO_TRUNCATE`, fenêtre d'import, liste R1, export CSV, remise à zéro, `kesh-seed`), audit
   (`audit_labels.rs`, registre des routes `LIB_ROUTES`), RBAC et clés d'API, i18n 4 locales, MCP (exclusion
   justifiée), manuels FR (`.tex` et PDF), E2E, captures.
4. **Comptabilité** : écritures proposées équilibrées, TVA (B6, B18, O5, O6), frais (B3), conversions (B4, A31),
   remboursements (B7, B20), aucune écriture d'office (B5).
5. **Sécurité** : secrets (jamais réaffichés, jamais journalisés), réponses du prestataire tenues pour hostiles
   (tailles, formats), aucun appel réseau en CI (B12).
6. **Exigences** : chaque FR/NFR vraie au regard de son AC ; point majeur d'AC sans exigence.
7. **Tests manquants** pour chaque AC ; **taille** de la story (≤ 5 modules, sinon proposer un découpage).

## Interdits

N'écris que dans **la fiche visée**, par **Edit** seulement. Aucun script (ni ad hoc), aucun `cargo`/`npm`, aucun
`git` en écriture, aucun appel réseau (sauf WebFetch de la documentation officielle PayPal pour 38-7/38-8).

## Rapport (français)

Findings `<story>-P3-N` (sévérité CRITICAL/HIGH/MEDIUM/LOW, lieu, constat avec la ligne lue, correctif) ; **axes
exercés / non exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-15 —
validation P3 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings
inter-fiches avec le texte proposé. Message final bref.

## Propre à la P3

- Périmètre : **ce qui a changé depuis la P2** — `git diff 27b81289 94d39ad9 -- <fiche>` (correctifs P2, alignement
  inter-fiches `38-p2-inter-fiches.md`, découpage de 38-6c en **38-6c1** et **38-6c2**). La sévérité se déplace vers ce
  qu'on vient d'écrire ; pour une fiche neuve (38-6c1, 38-6c2), relis-la en entier. Fiches `split` (38-3, 38-6c) : ne
  pas valider.
- **Renumérotation** : 38-6c2 a renuméroté ses AC (ex-AC-12 à AC-15 → AC-1 à AC-4) ; les fiches sœurs ont pu garder
  les anciens numéros (« 38-6c2 AC-15 ») : vérifie chaque renvoi à 38-6c1/38-6c2 et corrige-le dans ta fiche.
- **D-P2-5 (orchestrateur, à appliquer en 38-6a, 38-6b, 38-9b, et dans l'epic C-38-8 si ta fiche est 38-6a)** : sur un
  compte prestataire, une transaction qui correspond à une **facture** de Kesh mais porte des frais (`fee_amount ≠ 0`)
  ne reçoit **aucune proposition** : elle va à la file « à examiner » avec le motif `invoice_with_fee` (calculé, pas
  stocké). Raison : la proposition `provider` passerait le paiement d'une facture existante en produit (doublon).
