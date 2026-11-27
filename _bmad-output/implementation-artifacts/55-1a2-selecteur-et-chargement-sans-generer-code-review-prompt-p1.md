# Prompt — revue de code P1 de la story 55-1a2-selecteur-et-chargement-sans-generer (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-1a2-selecteur-et-chargement-sans-generer** de Kesh. **Passe complète, première de la boucle.
Modèle : Sonnet. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-1a2-selecteur-et-chargement-sans-generer.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `e33426630`, `946347e16`, `5645d6890` (communs avec la 55-1a1 ; la part de la 55-1a2 est donnée par les sections *Fichiers* et *Périmètre de revue* de sa fiche ; fiche mère `55-1a-etat-des-rapports-dans-l-url.md`). Postérieurs : `8e801d1aa` (55-2, Grand livre sur `ReportSelector`), `c3f28d2f9`, `647b5b039` — juge le code à la tête pour la part de la 55-1a2.
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-1a2-selecteur-et-chargement-sans-generer/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : AC-14 à AC-26 (et la part de la 55-1a2 des autres critères, fiche), un par un, au code et aux
  tests.
- **Sélecteur d'exercice** : `fiscal-year-menu.ts` (réplique de `nom_par_defaut`,
  `crates/kesh-core/src/accounting/exercice.rs`), `ReportSelector.svelte`, `resolveClassicPeriod`,
  `ReportPeriodHeading.svelte` et les six vues (en-tête de période).
- **Chargement sans Générer** (D16) : `generate()` (numéro de demande, `mySeq !== genSeq`), `onPeriodCommit` et le
  drapeau `pointerdown` (R-19), `selectFiscalYear`, `fiscalYearChanged`, `hiddenByMenu` (AC-26), `exportReport` (un
  export ne sort jamais une autre période que l'écran) ; un chargement par geste, jamais deux pour une clé ; aucune
  réponse périmée affichée ; focus gardé pendant un chargement ; états « Chargement… », grisé, `aria-busy`.
- **Les neuf specs E2E adaptées** : chaque clic sur *Générer* retiré est-il remplacé par une attente qui constate la
  même chose ?
- **Textes** : dix clés `reports-nav-*`, deux récrites, deux retirées ; manuel « Choisir la période » et phrases de
  l'Epic 46.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
