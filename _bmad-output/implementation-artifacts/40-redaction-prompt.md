# Prompt de rédaction des fiches de l’Epic 40 (Stripe)

Worktree `/home/beat/limu/compta/kesh-wt40`, branche `epic/40-stripe` (n'écris nulle part ailleurs). Kesh :
comptabilité suisse en production (Rust/Axum `crates/`, SvelteKit `frontend/`, MariaDB). Artefacts en **français**,
concis et factuels.

## À lire d'abord

- `_bmad-output/planning-artifacts/epics-38-40-cadrage.md` (décisions de Beat **B1–B28**, qui priment) ;
- `_bmad-output/planning-artifacts/epic-40-stripe.md` : décisions G*, contrats C-40-*, fiche de chaque story (périmètre,
  modules, FR/NFR) ; **décision de l'orchestrateur P40-1** : G4 admet les clés restreintes `rk_live_` **et** `rk_test_`
  (compte de test de Beat), refuse toute clé `sk_` ; le mode (`livemode`) est montré à l'écran et contrôlé à chaque réponse ;
- `_bmad-output/planning-artifacts/epic-38-paypal.md` (socle validé) et les fiches 38-* qu'une story 40 étend (38-2,
  38-3b, 38-6a, 38-6b, 38-7, 38-8, 38-9a, 38-10 surtout) : reprends exactement leurs noms ;
- réponses réelles du compte de test : `/home/beat/limu/compta/kesh/data-not-committed/stripe-test-samples/` (lecture
  seule ; README d'abord) ; documentation `docs.stripe.com` par WebFetch si besoin (cite les URL) ;
- `.claude/skills/bmad-create-story/template.md` et `checklist.md` ; une fiche 38-* comme modèle (p. ex. 38-7) ;
- `CLAUDE.md` : *Migration breaking policy*, *Pattern batch*, *Un appariement automatique propose, il ne crée jamais*,
  *Règle de splitting préventif*, *Recette visuelle*, manuels.

## Pour chaque fiche de ta mission

Écris `_bmad-output/implementation-artifacts/<clé>.md` (clé exacte du registre `sprint-status.yaml`) :
- en-tête (statut `ready-for-dev`, dépendances), *Story* (en tant que…, je veux…, afin que…) ;
- `## Exigences` : les FR/NFR de la fiche d'epic (`FR-40-<n>-<k>`, `NFR-40-<n>-<k>`), complétées si un AC en ajoute,
  chacune renvoyant à son AC ;
- **critères d'acceptation numérotés** (AC-1…) précis et testables : routes (méthode, chemin, corps, statuts, codes
  d'erreur existants ou neufs, `details`), rôles (RBAC), schéma et migration (P1–P8, ligne d'audit d'idempotence,
  triage P7, `TABLES_TO_TRUNCATE`, sauvegarde/restauration, export CSV), audit (`audit_labels.rs`), i18n 4 locales,
  MCP (justifier toute exclusion), manuels (`docs/manual/fr/…`, PDF), E2E et captures si contrôle visible ;
- **chaque prémisse sur le code existant se vérifie au code** et se cite `fichier:ligne` ;
- Tâches / sous-tâches, Dev Notes (architecture, pièges, références), section Change Log
  (« 2026-11-15 — rédaction (Opus) »).
- Points de contact : reprends exactement les noms des contrats C-40-* et des fiches sœurs ; ne contredis pas
  l'epic. Si l'epic est faux au code, écris-le dans la fiche (« Écart à l'epic ») sans modifier l'epic.

## Interdits

N'écris que les fichiers de ta mission (Write/Edit). Pas de `sprint-status.yaml`, pas de fichier d'epic. Aucun script,
aucun `cargo`/`npm`, aucun `git` en écriture, aucun appel réseau.

## Rapport (français, bref)

Par fiche : nombre d'AC, de FR/NFR, migrations, écarts à l'epic, questions ouvertes.
