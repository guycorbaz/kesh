# Prompt — revue de code P2 de la story 55-1a1 (Epic 55), passe ciblée

Tu relis, **en lecture seule**, la seule remédiation de la P1 de la story **55-1a1**. **Passe ciblée, une lentille
(chasseur de régressions), modèle : Sonnet, contexte frais.** Langue du rapport : français.

**Lis d'abord** `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (interdits, garde-fou `grep -nF`,
manuel, forme du rapport ; la méthode en trois couches est remplacée ici par la lentille unique).

- Fiche : `_bmad-output/implementation-artifacts/55-1a1-etat-dans-l-url-et-retour.md`.
- Rapport de la P1 : `<journaux>/s55-revue/55-1a1-etat-dans-l-url-et-retour/revue-p1-rapport.md` (P1-1 MEDIUM remédié).
- Commit à relire : **`a50712ce2`** (`git show a50712ce2`), part de la 55-1a1 : `failureIfCurrent` et ses quatre appels
  dans `generate()` (`frontend/src/routes/(app)/reports/+page.svelte`), le test « revue P1-1 — échec du Compte de
  résultat APRÈS le Retour au Bilan » (`reports-page-url.test.ts`). Le même commit porte la remédiation de la 55-1a2,
  relue par sa propre passe : ne la juge que si elle casse la part de la 55-1a1.
- Mutations : `<journaux>/s55-revue/mutations-p1/` (M1 pour ce correctif) ; Vitest des Rapports :
  `<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/vitest-p1-remed.log`.
- Rapport : **`<journaux>/s55-revue/55-1a1-etat-dans-l-url-et-retour/revue-p2-rapport.md`**.

## Axes (chacun exercé ou déclaré non exercé)

1. **Le correctif est-il juste partout** : un échec de l'onglet actif s'affiche-t-il toujours (premier chargement,
   rechargement, balance âgée, grand livre, rapport par compte analytique, adresse collée) ? Un échec tu laisse-t-il un
   état incohérent (`loading`, `inFlightTab`, onglet gris, `aria-busy`) ? Revenu sur l'onglet en échec : redemandé ?
2. **Régressions** : `myTab` vaut-il bien l'onglet de la demande dans les quatre branches ; une branche où `activeTab`
   change pendant l'`await` sans être la demande (par exemple le rapport par compte analytique, qui lit `activeTab`
   après l'`await`) ; les autres `errorMsg = formatError` de la page (exports) sont-ils concernés ?
3. **Le test** prouve-t-il ce qu'il dit (rouge sous M1, relu au journal) ? AC-10 de la fiche relu contre le correctif.
4. **Propagation** : le même symptôme ailleurs (autres pages à onglets qui posent un message d'erreur après un `await`).
5. **Le manuel** (PDF aplati, étiquette `fusion`) : « Si un chargement échoue, le message d'erreur s'affiche » reste-t-il
   vrai ?
