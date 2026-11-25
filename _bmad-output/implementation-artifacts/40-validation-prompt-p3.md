# Prompt de validation P3 — fiches de l'Epic 38 (PayPal et socle des prestataires)

**Troisième passe** (modèle différent de P2, Opus) (`bmad-create-story validate`, `CLAUDE.md` § *Review Iteration Rule*), contexte frais, sur une
fiche du worktree `<dépôt>-wt40` (branche `epic/40-stripe` ; n'écris nulle part ailleurs). À lire
d'abord :

- `_bmad-output/planning-artifacts/epics-38-40-cadrage.md` (décisions du responsable de l'epic **B1–B28**, qui priment) ;
- `_bmad-output/planning-artifacts/epic-40-stripe.md` (G*, C-40-*, fiches de stories) et la décision **P40-1** de
  `_bmad-output/implementation-artifacts/40-redaction-prompt.md` (clés `rk_test_`/`rk_live_` admises, `sk_` refusée) ;
- `_bmad-output/planning-artifacts/epic-38-paypal.md` (socle validé) et les fiches 38-* qu'étend la fiche visée ;
- la fiche visée et ses sœurs `40-*.md` ; les réponses réelles du compte de test
  `<dépôt>/data-not-committed/stripe-test-samples/` (lecture seule) ;
- `CLAUDE.md` : *Migration breaking policy*, *Pattern batch*, *Un appariement automatique propose, il ne crée jamais*,
  *Règle de splitting préventif*, *Recette visuelle*, manuels.

## Ce que tu cherches

1. **Prémisse fausse au code** : vérifie chaque `fichier:ligne` cité ; un écart est un finding.
2. **Contradiction** avec B*, G*, P40-1, C-40-*, les fiches 38-* ou une fiche sœur 40-* (noms de tables, colonnes, codes d'erreur, routes,
   fonctions, qui fait quoi, ordre des stories). 
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
`git` en écriture, aucun appel réseau (sauf WebFetch de docs.stripe.com).

## Rapport (français)

Findings `<story>-P3-N` (sévérité CRITICAL/HIGH/MEDIUM/LOW, lieu, constat avec la ligne lue, correctif) ; **axes
exercés / non exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-15 —
validation P3 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings
inter-fiches avec le texte proposé. Message final bref.

## Propre à la P3

- Périmètre : la remédiation depuis la P2 — `git diff 7efc2e9f 9ee719d9 -- <fiche>` (correctifs P2, alignement inter-fiches P2,
  section « Validation P2 » de l'epic). La sévérité se déplace vers ce qu'on vient d'écrire.
- Points à appliquer s'ils visent ta fiche :
  - **40-1 — décision D40-P3-1 (orchestrateur)** : le passage lit aussi `GET /v1/account` **au début de chaque passage**
    (une requête, hors plafond) et applique la même `account_decision` que « Tester » ; une clé sans droit de lecture du
    compte est refusée au passage comme à « Tester » (motif à nommer, i18n) ; les deux cas tracés au registre C-40-7 sont
    ainsi fermés : retire-les du registre (epic, une ligne) et ajoute les tests.
  - **40-6** : le texte inter-fiches des Questions ouvertes qui cite `nav-providers` et la phrase « préfixe attendu de la
    correction de 38-9a AC-10 » sont périmés : 38-9a (corrigée et close) retient `provider-connections-` et
    `nav-admin-providers` ; mets à jour (le Change Log daté reste tel quel).
