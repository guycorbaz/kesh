# Prompt de validation P2 — fiches de l'Epic 38 (PayPal et socle des prestataires)

**Deuxième passe** (modèle différent de P1, Sonnet) (`bmad-create-story validate`, `CLAUDE.md` § *Review Iteration Rule*), contexte frais, sur une
fiche du worktree `/home/beat/limu/compta/kesh-wt40` (branche `epic/40-stripe` ; n'écris nulle part ailleurs). À lire
d'abord :

- `_bmad-output/planning-artifacts/epics-38-40-cadrage.md` (décisions de Beat **B1–B28**, qui priment) ;
- `_bmad-output/planning-artifacts/epic-40-stripe.md` (G*, C-40-*, fiches de stories) et la décision **P40-1** de
  `_bmad-output/implementation-artifacts/40-redaction-prompt.md` (clés `rk_test_`/`rk_live_` admises, `sk_` refusée) ;
- `_bmad-output/planning-artifacts/epic-38-paypal.md` (socle validé) et les fiches 38-* qu'étend la fiche visée ;
- la fiche visée et ses sœurs `40-*.md` ; les réponses réelles du compte de test
  `/home/beat/limu/compta/kesh/data-not-committed/stripe-test-samples/` (lecture seule) ;
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

Findings `<story>-P2-N` (sévérité CRITICAL/HIGH/MEDIUM/LOW, lieu, constat avec la ligne lue, correctif) ; **axes
exercés / non exercés** ; décompte. Phase 2 : applique les correctifs propres à la fiche ; Change Log « 2026-11-15 —
validation P2 (<modèle>) : <décompte> ; remédiés dans la fiche sauf <ids inter-fiches> » ; liste les findings
inter-fiches avec le texte proposé. Message final bref.

## Propre à la P2

- Périmètre : relis la fiche en entier, en insistant sur ce qui a changé depuis la rédaction : `git diff 040a811e 890f468f -- <fiche>`
  (correctifs P1, alignement inter-fiches, section « Validation P1 » de l'epic). La sévérité se déplace vers ce qu'on vient
  d'écrire.
- Points à appliquer s'ils visent ta fiche :
  - **40-4** : la borne de `unreconciled` est **exclue** (`booked_at < asOf`), alignée sur 38-9a AC-8 (b) (corrigée et
    close) et 38-3b AC-7 ; corrige les lignes qui disent `≤`, le test (a) et ferme Q-40-4-2.
  - **40-1** : l'Écart 1 dit encore que l'epic réserve `rk_test_` à `KESH_TEST_MODE` ; l'epic est désormais aligné sur
    P40-1 : mets l'Écart à jour.
