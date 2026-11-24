# Prompt de rédaction des fiches de l'Epic 38 (PayPal et socle des prestataires)

Dépôt `<dépôt>`, branche `epic/38-paypal` (n'en change pas ; un autre agent travaille dans
`../kesh-wt37`, n'y touche pas). Kesh : comptabilité suisse en production (Rust/Axum `crates/`, SvelteKit `frontend/`,
MariaDB). Artefacts en **français**, concis et factuels.

## À lire d'abord

- `_bmad-output/planning-artifacts/epics-38-40-cadrage.md` (décisions du responsable de l'epic **B1–B26**, qui priment) ;
- `_bmad-output/planning-artifacts/epic-38-paypal.md` : décisions E*, contrats C-38-*, fiche de chaque story
  (périmètre, modules, migrations, FR/NFR déjà listées), section « Révision B22–B26 » ;
- `_bmad-output/planning-artifacts/epic-37-multi-devises.md` (ce que l'Epic 37 livre, dont 38 dépend) ;
- `.claude/skills/bmad-create-story/template.md` et `checklist.md` (forme d'une fiche BMAD) ; une fiche 37-* comme
  modèle de niveau de détail (p. ex. `37-2-cours-mensuels-afc.md`) ;
- `CLAUDE.md` : *Migration breaking policy* (P1–P8), *Pattern batch*, *Un appariement automatique propose, il ne crée
  jamais*, *Règle de splitting préventif*, *Test Locally First*, *Recette visuelle*, manuels.

## Pour chaque fiche de ta mission

Écris `_bmad-output/implementation-artifacts/<clé>.md` (clé exacte du registre `sprint-status.yaml`) :
- en-tête (statut `ready-for-dev`, dépendances), *Story* (en tant que…, je veux…, afin que…) ;
- `## Exigences` : les FR/NFR de la fiche d'epic (`FR-38-<n>-<k>`, `NFR-38-<n>-<k>`), complétées si un AC en ajoute,
  chacune renvoyant à son AC ;
- **critères d'acceptation numérotés** (AC-1…) précis et testables : routes (méthode, chemin, corps, statuts, codes
  d'erreur existants ou neufs, `details`), rôles (RBAC), schéma et migration (P1–P8, ligne d'audit d'idempotence,
  triage P7, `TABLES_TO_TRUNCATE`, sauvegarde/restauration, export CSV), audit (`audit_labels.rs`), i18n 4 locales,
  MCP (justifier toute exclusion), manuels (`docs/manual/fr/…`, PDF), E2E et captures si contrôle visible ;
- **chaque prémisse sur le code existant se vérifie au code** et se cite `fichier:ligne` ;
- Tâches / sous-tâches, Dev Notes (architecture, pièges, références), section Change Log
  (« 2026-11-15 — rédaction (Opus) »).
- Points de contact : reprends exactement les noms des contrats C-38-* et des fiches sœurs ; ne contredis pas
  l'epic. Si l'epic est faux au code, écris-le dans la fiche (« Écart à l'epic ») sans modifier l'epic.

## Interdits

N'écris que les fichiers de ta mission (Write/Edit). Pas de `sprint-status.yaml`, pas de fichier d'epic. Aucun script,
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français, bref)

Par fiche : nombre d'AC, de FR/NFR, migrations, écarts à l'epic, questions ouvertes.
