# Prompt — revue de code P2 de la story 55-1a2 (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-1a2** de Kesh, remédiation de la P1 comprise. **Passe
complète, deuxième de la boucle. Modèle : Opus. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md`.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-1a2-selecteur-et-chargement-sans-generer.md`, et sa mère
  `55-1a-etat-des-rapports-dans-l-url.md`.
- Rapport de la P1 : `<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/revue-p1-rapport.md` (P1-1 HIGH,
  P1-2, P1-3 MEDIUM et le LOW P1-5 remédiés ; P1-4, P1-6 renvoyés en issues).
- Commits : ceux de la P1 (`e33426630`, `946347e16`, `5645d6890`, part de la 55-1a2) **et la remédiation `a50712ce2`**,
  à relire en priorité (`leavesToTablist`, `pointerTab`, `tabOfFocus`, `updatePeriodInvalid` dans `selectTab` et
  `commitPeriod`, `hiddenByMenu` étendu aux rapports par compte analytique en mode Exercice ; quatre tests Vitest).
- Mutations des correctifs : `<journaux>/s55-revue/mutations-p1/` (M2 à M4) ; Vitest des Rapports :
  `<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/vitest-p1-remed.log`.
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`) ; la remédiation ne
  touche pas le `.tex`.
- Rapport : **`<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/revue-p2-rapport.md`**.

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **La remédiation d'abord** : chaque chemin de sortie des dates (Tab, Maj+Tab, clic sur l'onglet actif, sur un autre,
  `pointerdown` relâché hors d'un onglet, Safari `relatedTarget` nul, flèches dans la liste, Entrée) — un chargement par
  geste, jamais deux pour une clé ; le **grand livre** (`onLedgerPeriodCommit`, même `leavesToTablist`, story 55-2) et
  les **rapports par compte analytique** (qui gardent *Générer*) ; `updatePeriodInvalid` à chaque `selectTab` (grand
  livre, balance âgée, flèches) : un message faux ou périmé ? `hiddenByMenu` étendu : exports des rapports par compte
  analytique, mode Cumulé, `fiscalYearChanged`, effet des réglages TVA.
- **Acceptation** : AC-14 à AC-26 (et la part de la 55-1a2 des autres critères), au code et aux tests.
- **Sélecteur, en-tête, six vues, `generate()`, exports, états** : comme en P1 (prompt P1 de la story), pour ce que la
  P1 n'a pas couvert ou que la remédiation touche.
- **Le manuel** (`.tex` et PDF aplati) : « Choisir la période », chargement, AC-26, contre le code remédié.
- **Tests** : les quatre tests neufs prouvent-ils ce qu'ils disent (rouges sous M2 à M4) ? Le test « focus qui part vers
  un onglet » (autre onglet : 0 requête) reste-t-il juste ?
- **i18n, propagation, interaction avec la fusion `c8fe4d0ac`** : comme en P1.
