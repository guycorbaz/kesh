# Prompt — revue de code P1 de la story 55-4-solde-dette-epic (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-4-solde-dette-epic** de Kesh. **Passe complète, première de la boucle.
Modèle : Sonnet. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-4-solde-dette-epic.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `647b5b039` (les deux correctifs retenus : comptes de TVA désignés relus au prochain affichage après un échec ; test de la clé écrite par la page).
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-4-solde-dette-epic/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : chaque critère de la fiche, un par un ; la liste des candidates et des exclusions (T0) contre
  `kesh-gate-logs/s55-dev/issues-a-ouvrir.md`.
- **Le premier correctif** : la relecture des réglages TVA (comptes désignés) après un échec — retentée au prochain
  affichage, jamais en boucle, aucune requête de trop quand elle a réussi, ligne des comptes jamais affichée fausse.
- **Le second** : le test de la clé écrite par la page tue-t-il la mutation « `lastHandledKey` non posé avant `goto` »
  (preuve du rouge relue) ?
- **Périmètre** : plafond d'issues de la story, aucune régression des stories 55-1a à 55-3b.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
