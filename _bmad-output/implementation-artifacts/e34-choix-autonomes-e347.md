# Epic 34 — choix autonomes de la story 34-7 (clé `e347`, #945)

Registre des choix pris en autonomie au développement de la 34-7. Les choix de la fiche sont au registre de son groupe,
`e34-choix-autonomes-e343.md` (C-e343-8).

## C-e347-1 — l'en-tête (iii bis) du registre des routes nomme la réouverture et le renommage

- **Contexte.** L'AC-4 demande de relire l'en-tête qui énumère les preuves dynamiques
  (`audit_route_registry.rs`, point (iii bis)) et d'y ajouter le test neuf « s'il énumère les routes
  `SansEcritureAuJournal` rejouées de la même manière ». Il en nomme deux (`close_fiscal_year`, `create_fiscal_year`), et
  omettait déjà la réouverture (33-1a), tenue pourtant par un test du même fichier.
- **Retenu.** Une incise ajoute la réouverture et le renommage d'un exercice, avec leurs stories. Le nombre « huit routes
  des tests 2 à 5, 7, 9 et 10 » n'est pas touché : il compte ces tests-là, que la phrase ajoutée ne change pas.
- **Autres options.** N'ajouter que le renommage (laisse l'omission de la 33-1a, limite voisine courte) ; ne rien
  ajouter (l'en-tête parle de la colonne `Rejouee`, mais il nomme déjà deux routes `SansEcritureAuJournal`).
- **Réversibilité.** Totale : prose d'un doc-comment de test.

## C-e347-2 — les deux tests et le nom visé de l'AC-2

- **Contexte.** La fiche nomme le test de l'AC-1 et laisse celui de l'AC-2 libre.
- **Retenu.** `fiscal_year_rename_is_replayed_when_it_is_the_deadlock_victim` (AC-1) et
  `fiscal_year_valid_rename_succeeds_after_the_replay` (AC-2). Le nom visé de l'AC-2 est `"Exercice libre (rejeu)"`,
  inséré par la lourde avec les dates de 2012, hors de M (2010) et N (2011) et de l'exercice 2020-2030 du semis. Une aide
  `audits_renommage` compte les entrées `fiscal_year.updated`, partagée par les deux tests.
- **Établi à l'exécution** (point « à établir à l'étape 0 » de l'AC-2) : la lecture `FOR UPDATE` de doublon attend bien la
  ligne insérée non validée, et la route est la victime. Avant le correctif, la lourde a obtenu M (le `.expect` de
  `victime` a passé) et la route a rendu 500 (`kesh-gate-logs/e347/rouges.log`) ; après, 200 et un rejeu capturé.
- **Réversibilité.** Totale.

## C-e347-3 — le recompte du point (vi) se déclare avec sa méthode

- **Contexte.** La fiche demande de recompter à la source la liste du point (vi) avant d'écrire « vingt-cinq ».
- **Retenu.** Recompte par lecture de la table (`SansEcritureAuJournal`) et du corps de chaque handler de `src/routes/` :
  vingt-deux handlers portaient une enveloppe avant la story (deux faux positifs du motif `_once` écartés à la lecture :
  `backup_schedule::run_backup_now` et `reconciliation::post_unreject_transaction`, qui appellent une fonction
  `…_once` sans rejeu), plus `onboarding::seed_demo` et `onboarding::reset`, rejouées dans `kesh-seed` : vingt-quatre,
  vingt-cinq avec le renommage. La phrase de recompte s'ajoute à la fin du point (vi), sans réécrire « Vingt-trois avec
  elle — recompté le 2026-11-05 », relevé daté.
- **Réversibilité.** Totale.
