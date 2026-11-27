# Prompt — revue de code P2 de la story 55-3a-justificatifs-serveur (Epic 55), passe ciblée

Tu relis, **en lecture seule**, la seule remédiation de la P1 de la story **55-3a-justificatifs-serveur** : **la phrase de `docs/api-external.md` sur `hasDocuments` (« … ou un règlement de cette facture, partiel ou total, groupé compris … »), remède du LOW P1-4**. **Passe ciblée, une
lentille, modèle : Haiku, contexte frais.** Langue du rapport : français.

**Lis d'abord** `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (interdits, garde-fou `grep -nF`,
forme du rapport ; la méthode en trois couches est remplacée par la lentille unique).

- Commit à relire : **`b71869a66`** (`git show b71869a66`), la part de la story seulement.
- Rapport de la P1 : `<journaux>/s55-revue/55-3a-justificatifs-serveur/revue-p1-rapport.md`.
- Manuel compilé à ce commit : étiquette `remed-p1` (`<journaux>/s55-revue/manuel/remed-p1/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-3a-justificatifs-serveur/revue-p2-rapport.md`**.

## Axes (chacun exercé ou déclaré non exercé)

1. La phrase corrigée est-elle **vraie** contre l'union des rattachements (`crates/kesh-report/src/documents.rs`) et la section de la route `GET /journal-entries/{id}/documents` du même document ? Cite les lignes lues.
2. N'introduit-elle pas une **contradiction** avec le reste du document (paragraphes voisins, renvois `\ref`, autre
   section qui dit la même chose autrement) ? Greppe le texte aplati du PDF.
3. Le PDF compilé porte-t-il la phrase telle quelle (aucun caractère cassé, aucun `??` de renvoi) ?
4. Pour toute affirmation d'absence, la preuve `grep -nF` (garde-fou des passes Haiku).
