# Prompt — revue de code P1 de la story 55-1a1-etat-dans-l-url-et-retour (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-1a1-etat-dans-l-url-et-retour** de Kesh. **Passe complète, première de la boucle.
Modèle : Opus. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-1a1-etat-dans-l-url-et-retour.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `e33426630`, `946347e16`, `5645d6890` (communs avec la 55-1a2 ; la part de la 55-1a1 est donnée par les sections *Fichiers* et *Périmètre de revue* de sa fiche ; la fiche mère `55-1a-etat-des-rapports-dans-l-url.md` est la référence complète). Postérieurs qui retouchent ces fichiers : `c3f28d2f9`, `8e801d1aa`, `647b5b039` — juge le code à la tête pour la part de la 55-1a1.
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-1a1-etat-dans-l-url-et-retour/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : AC-1 à AC-11 et AC-13, tels qu'amendés (table *Ce que le relecteur vérifie* de la fiche), un par
  un, au code et aux tests.
- **Le modèle de l'URL** : `parseReportUrl`, `buildReportSearch`, `reportHref`, `ledgerHref` (sortie inchangée, trois
  cas) ; valeurs invalides (D5), normalisation en `replace`, sans boucle.
- **La page** (`routes/(app)/reports/+page.svelte`) : `applyUrl` et le `$effect` sur `page.url`, `writeUrl` et
  `lastHandledKey` (une URL écrite par la page jamais réappliquée), `selectTab` (empile / remplace, `goto` en dernier,
  jamais attendu, `keepFocus`/`noScroll`), `handleTabKeydown`, `loadActiveIfNeeded` (clé en vol, clé affichée, R-1,
  R-17), `displayedKeys`/`inFlightTab`, `invalidateInFlight`, `resyncActiveGroupedTab` après tout affichage sans requête.
- **Courses** (AC-8) : Retour pendant un chargement, autre exercice, réponse périmée affichée.
- **Accessibilité** (AC-10) : `aria-selected`, `tabindex`, focus, défilement.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
