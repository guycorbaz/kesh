# Prompt — revue de code P3 de la story 55-1a2 (Epic 55), passe ciblée

Tu relis, **en lecture seule**, la seule remédiation de la P2 de la story **55-1a2**. **Passe ciblée, une lentille
(chasseur de régressions), modèle : Sonnet, contexte frais.** Langue du rapport : français.

**Lis d'abord** `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (interdits, garde-fou `grep -nF`,
manuel, forme du rapport ; la lentille unique remplace les trois couches).

- Rapport de la P2 : `<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/revue-p2-rapport.md` (P2-1 et
  P2-2 MEDIUM remédiés ; P2-3 à P2-5 renvoyés en issues).
- Commit à relire : **`5a3242d80`** (`git show 5a3242d80`) — `|| hiddenByMenu` sur les deux exports des rapports par
  compte analytique (`frontend/src/routes/(app)/reports/+page.svelte`), deux assertions au test « revue P1-3 (AC-26) »
  (`reports-page-url.test.ts`), la phrase des exports du manuel (`docs/manual/fr/user-manual.tex`, § « Choisir la
  période » et suivants), la note *Exports* de la fiche mère `55-1a-etat-des-rapports-dans-l-url.md`.
- Mutation et Vitest : `<journaux>/s55-revue/mutations-p2/` (`vert.log` : Vitest des Rapports ; `M5.log` : rouge sans
  le correctif).
- Manuel compilé à ce commit : étiquette `remed-p2` (`<journaux>/s55-revue/manuel/remed-p2/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/revue-p3-rapport.md`**.

## Axes (chacun exercé ou déclaré non exercé)

1. Les exports d'un rapport par compte analytique : actifs quand il est affiché (mode Exercice, un exercice ; mode
   Cumulé), inactifs quand il est retiré ; rien d'autre ne change (exports des onglets classiques, du grand livre).
2. La phrase du manuel est-elle **vraie** au code pour chaque onglet : `canExport` (classiques), boutons du grand livre
   (`generalLedger === null || activeBusy`), des rapports par compte analytique ; balance âgée (le manuel en dit-il
   quelque chose de faux ?) ; contradiction avec le reste du manuel (PDF aplati, `??`, renvois) ?
3. La note de la fiche mère dit-elle la même chose que le manuel et le code ?
4. Le test prouve-t-il ce qu'il dit (`M5.log`) ?
