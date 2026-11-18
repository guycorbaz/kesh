# Story 43-6 — registre des choix (clé `e436`)

Registre de la story 43-6. Les décisions de l'épique (C-43-11, C-43-18, C-43-19) et la fiche font foi ; ce registre
consigne les choix tranchés par l'agent.

**C-e436-1 — Le refus vit dans la lecture verrouillante de l'étape (3), non dans une requête `EXISTS` à part**
(remédiation V1-3, 2026-11-18). La lecture `invoice_settlements … ORDER BY id FOR UPDATE` rend aussi
`journal_entry_line_id` ; le refus tombe avant la lecture des avoirs. Une seule requête, la discipline du module (lecture
sous verrou après la pièce), et `details.journalEntryId` en sort directement. Autre option : `EXISTS` puis une seconde
requête pour l'identifiant (lecture ordinaire, sûre seulement par une propriété non écrite). Réversible sans migration.

**C-e436-2 — `details.journalEntryId` = la plus petite `journal_entry_id` parmi les règlements à ligne non nulle**,
calculée sur le résultat de la lecture (l'ordre `ORDER BY id` ne la garantit pas). En 0.19.0, une facture n'a qu'un
règlement groupé (le groupe la solde) ; la règle couvre le cas futur.

**C-e436-3 — Le refus précède le « rien à faire »** (V1-4) : une facture groupée est refusée même quand rien ne
changerait. Cohérent avec le critère « la facture porte une ligne attribuée » ; `api-external.md` dira la précédence.
Autre option : laisser le `200` passer avant — il exigerait de verrouiller les lignes des autres factures pour le juger.

**C-e436-4 — Phrase de l'épique ajustée** (C-43-11 de `epic-43-transaction-plusieurs-factures.md`) pour la position
dans l'étape (3) et la précédence ; le reste de C-43-11 inchangé.

**C-e436-5 — Reprise après le plantage de la station (2026-11-18, 10 h 46) : deux retouches au commit `8a75833ec`.**
(a) L'espace après `=` de `projects-reassign-open` avait disparu dans les quatre `messages.ftl` au commit `600b9d75f`
(sans effet pour Fluent, mais hors périmètre) : rétabli. (b) La capture de l'E2E prenait le préfixe `36-16a-` du helper
`capture` de la 42-1 alors que son commentaire annonçait `43-6-refus-groupe` : elle s'écrit désormais par
`capturePath(testInfo, '43-6-refus-groupe')`, sans toucher au helper (les captures de la 42-1 gardent leur nom). Autre
option : paramétrer le préfixe du helper — plus large que la story. Réversible.

**C-e436-6 — Deux mutations ajoutées à la reprise, M2c et M2d.** Le test d'ordre aval
(`the_grouped_refusal_comes_before_target_and_closed_period`) enchaîne ses assertions : sous M2a (garde après l'étape 7)
il rougit dès la première (`404`, compte inconnu), si bien que le volet « période close » n'était prouvé par aucune
mutation. M2c (garde entre (5) et (6)) rougit encore sur le compte visé ; M2d (garde après (7) **et** contrôle du compte
visé neutralisé) isole la période close : `403` au lieu de `409`. Journaux `kesh-gate-logs/e436/mut/M2c.log`, `M2d.log`.
Autre option : scinder le test en trois — plus de montage, même preuve. Réversible. *Complété à la revue P1 (A1)* : M2e
(garde entre (5) et (6), contrôle « compte inconnu » neutralisé dans `projects::validate_taggable_in_tx`) isole le volet
« compte archivé » (`ILLEGAL_STATE_TRANSITION` au lieu du code de groupe, `mut/M2e.log`).

**C-e436-7 — Recette visuelle sur la base jetable de l'E2E** (revue P1, A4). AC7 et le `CLAUDE.md` disent « base de
démonstration » ; la capture `e436/captures/43-6-refus-groupe.png` est prise par l'E2E complet sur `kesh_e2e_e436`, base
jetable peuplée par la spec de données synthétiques. Le but de la règle — voir l'état touché, en thème clair, hors de
toute donnée réelle — est atteint ; la capture montre la valeur réelle du serveur. Autre option : relancer un backend sur
une base de démonstration (`kesh-seed`) et monter le groupe à la main — même image, sans preuve de plus. Réversible.

**C-e436-8 — Le message du refus garde son remède** (revue P1, B1/E2). « Annulez d'abord le rapprochement » est parfois
impossible (dé-rapprochement refusé : exercice clos, lettrage figé, facture créditée ou soldée…). Le message ne change
pas : le refus de l'annulation dit lui-même sa raison. La limite s'écrit au manuel (deux sites) et dans `api-external.md`
(deux sites), avec renvoi au § *Annuler un rapprochement*. Autre option : un second message pour ce cas — il exigerait
d'évaluer les refus du dé-rapprochement dans le geste. Réversible.
