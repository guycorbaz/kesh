# Registre des choix autonomes — intégration de la release 0.17.7

Agent d'intégration finale (Opus), worktree `kesh-rel0177`, branche `release/0.17.7` depuis `origin/main` = `67efc51fd`.
Prompt : `kesh-gate-logs/prompts/rel0177-integration.md`. Chaque choix : contexte, option retenue, autres options,
réversibilité.

## C-rel0177-1 — Ordre des points du CHANGELOG

- **Contexte** : la branche 35-8a2 apportait une section `[0.17.7]` dont les points suivaient l'ordre des fusions
  (35-3, 35-5, 35-2, 35-1, 35-8a1, 35-8a2), non celui des numéros de story qu'exige la règle d'union C-e350-8.
- **Retenu** : à la fusion de la 35-4, la section est remise dans l'ordre des numéros (35-1, 35-2a/b, 35-3, 35-4, 35-5,
  puis 35-6, 35-7, 35-8a1, 35-8a2 aux fusions suivantes), une rubrique (*Corrigé*), un point par issue ; textes inchangés.
- **Autres** : garder l'ordre de fusion (contraire à la règle) ; réordonner une seule fois en fin d'intégration (commit
  de plus, même résultat).
- **Réversibilité** : totale (texte seul).

## C-rel0177-2 — Raccord du relevé synthétique partagé (35-4 × 35-5c)

- **Contexte** : la 35-4 a extrait `dateOf` et le relevé synthétique de `reconciliation-show-older.spec.ts` dans
  `frontend/tests/e2e/helpers/camt-synthetic.ts`, avec l'arithmétique d'avant ; la 35-5c avait converti `dateOf` à
  `ajouterJoursIso` dans la spec. Après fusion, la garde des dates des specs (`local-date-guard.test.ts`) relève
  `camt-synthetic.ts:30` sans entrée (sonde positive au journal `rel0177/raccord1-sonde-positive.log`).
- **Retenu** : conflit résolu côté 35-4 (le helper partagé), puis raccord séparé : `dateOfEntry` passe par
  `ajouterJoursIso('2025-01-01', i)`, même valeur pour tout `i`.
- **Autres** : entrée à `ALLOWLIST_SPECS` (la garde est close de trois entrées : ce serait rouvrir ce que la 35-5c a
  fermé) ; reprendre le côté 35-5c (perd l'extraction dont dépend `reconciliation-already-booked-reject.spec.ts`).
- **Réversibilité** : totale.

## C-rel0177-3 — PDF des manuels pendant les fusions

- **Contexte** : `user-manual.pdf` est en conflit binaire à chaque fusion.
- **Retenu** : côté HEAD provisoire à chaque fusion ; les trois PDF sont régénérés par `make fr` au bloc de release,
  après les macros de version — jamais fusionnés.
- **Réversibilité** : totale (régénérés depuis les `.tex`).
