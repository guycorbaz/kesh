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
