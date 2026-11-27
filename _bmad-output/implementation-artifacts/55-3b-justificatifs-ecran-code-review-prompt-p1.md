# Prompt — revue de code P1 de la story 55-3b-justificatifs-ecran (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-3b-justificatifs-ecran** de Kesh. **Passe complète, première de la boucle.
Modèle : Sonnet. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-3b-justificatifs-ecran.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `b2cb1a42a` (code), `939bfa438` (manuel), `48b92e713` (visionneuse, E2E, compteurs), `9efde054f` (erreur « document vide »). La route serveur est de la 55-3a (`174fa5385`), hors périmètre ici sauf pour son appel.
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-3b-justificatifs-ecran/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : chaque critère de la fiche, un par un, au code et aux tests.
- **L'écran** : `EntryDocuments.svelte`, `EntryDocumentsViewer.svelte`, `entry-documents.ts`, `DocumentPreview.svelte`
  (visionneuse partagée avec les factures fournisseur importées : non-régression), `JournalReportView.svelte`,
  `GeneralLedgerView.svelte` (colonne « Justificatifs »), `reports.api.ts` ; un seul justificatif sans liste ; « PDF pas
  encore émis » sans aperçu ni lecture (une facture non figée ne se fige pas à l'affichage) ; document vide ; erreur
  réseau ; fermeture et focus ; libération des URL `blob:`.
- **Courses** : deux icônes cliquées vite, réponse d'une écriture précédente affichée pour une autre.
- **Accessibilité** : nom accessible de l'icône, dialogue (titre, fermeture, focus), lecteur d'écran.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
