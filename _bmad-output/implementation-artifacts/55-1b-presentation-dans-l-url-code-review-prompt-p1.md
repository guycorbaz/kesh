# Prompt — revue de code P1 de la story 55-1b-presentation-dans-l-url (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-1b-presentation-dans-l-url** de Kesh. **Passe complète, première de la boucle.
Modèle : Opus. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-1b-presentation-dans-l-url.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `c3f28d2f9` (code), `215fad3c8` (manuel et E2E), `07ff47e62`, `37a25b439` (tests). Le modèle repose sur celui de la 55-1a (`e33426630`), hors périmètre ici.
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-1b-presentation-dans-l-url/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : chaque critère de la fiche, un par un, au code et aux tests.
- **La présentation dans l'URL** (D4) : paramètres du bilan, du compte de résultat et de la balance (hiérarchie, niveau,
  sous-totaux, groupes) ; ce que la page écrit (par la réponse affichée, `writeUrlAtResponse`) et quand ; ce qu'elle lit
  à l'arrivée et à Retour ; compteurs (AC-7 amendé).
- **Courses** : une réponse périmée n'écrit pas l'adresse ; un `goto` pendant une navigation en cours ; la règle de la
  46-2 (`resyncActiveGroupedTab`).
- **Valeurs invalides** de présentation : ignorées, normalisées, jamais envoyées.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
