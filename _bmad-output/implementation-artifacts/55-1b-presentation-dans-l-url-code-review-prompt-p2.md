# Prompt — revue de code P2 de la story 55-1b-presentation-dans-l-url (Epic 55), passe ciblée

Tu relis, **en lecture seule**, la seule remédiation de la P1 de la story **55-1b-presentation-dans-l-url** : **les deux phrases du manuel utilisateur (`docs/manual/fr/user-manual.tex`, § des rapports groupés : « … interrompt ce chargement, comme le fait l'arrivée par le retour arrière ou par un lien sur un rapport qui n'est pas déjà affiché dans cette présentation … » et « Une adresse sans présentation prend la présentation en cours … »), remèdes des LOW P1-4 et P1-5**. **Passe ciblée, une
lentille, modèle : Haiku, contexte frais.** Langue du rapport : français.

**Lis d'abord** `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (interdits, garde-fou `grep -nF`,
forme du rapport ; la méthode en trois couches est remplacée par la lentille unique).

- Commit à relire : **`b71869a66`** (`git show b71869a66`), la part de la story seulement.
- Rapport de la P1 : `<journaux>/s55-revue/55-1b-presentation-dans-l-url/revue-p1-rapport.md`.
- Manuel compilé à ce commit : étiquette `remed-p1` (`<journaux>/s55-revue/manuel/remed-p1/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-1b-presentation-dans-l-url/revue-p2-rapport.md`**.

## Axes (chacun exercé ou déclaré non exercé)

1. La phrase corrigée est-elle **vraie** contre le code de la page (`frontend/src/routes/(app)/reports/+page.svelte` : `loadActiveIfNeeded`, `applyPresentation`) et les tests de la 55-1b (`reports-page-url.test.ts`, `describe` « la présentation dans l’URL ») ? Cite les lignes lues.
2. N'introduit-elle pas une **contradiction** avec le reste du document (paragraphes voisins, renvois `\ref`, autre
   section qui dit la même chose autrement) ? Greppe le texte aplati du PDF.
3. Le PDF compilé porte-t-il la phrase telle quelle (aucun caractère cassé, aucun `??` de renvoi) ?
4. Pour toute affirmation d'absence, la preuve `grep -nF` (garde-fou des passes Haiku).
