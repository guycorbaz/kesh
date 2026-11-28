# Prompt — revue de code P1 de la story 55-2-lien-de-compte-partout (Epic 55), passe complète

Tu relis, **en lecture seule**, le code de la story **55-2-lien-de-compte-partout** de Kesh. **Passe complète, première de la boucle.
Modèle : Sonnet. Contexte frais.** Langue du rapport : français.

**Lis d'abord** les consignes communes : `_bmad-output/implementation-artifacts/55-code-review-brief-commun.md` (base,
méthode en trois couches, manuel, garde-fou `grep -nF`, interdits, forme du rapport). Elles s'appliquent entièrement.

- Fiche (fait foi) : `_bmad-output/implementation-artifacts/55-2-lien-de-compte-partout.md` — lis-la entière, T0, Dev Agent Record et Change
  Log compris.
- Commits de la story (diff à relire, puis code à la tête) : `8e801d1aa` (code), `7559f5f57` (balayage du style des liens, D17), `0a8f04d06` (manuel et E2E), `cdee9ebf5`, `8db734ef8` (tests). `9952b2d30` reprend un correctif du manuel (guillemets), hors story. Retouche postérieure : `647b5b039` (55-4, réglages TVA relus).
- Manuel compilé : étiquette `fusion` (`<journaux>/s55-revue/manuel/fusion/user-manual-aplati.txt`).
- Rapport : **`<journaux>/s55-revue/55-2-lien-de-compte-partout/revue-p1-rapport.md`** (le seul fichier que tu écris ; le dossier existe).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

- **Acceptation** : chaque critère de la fiche (dont AC-6 comptes de TVA, AC-16 balayage D17), un par un, au code et
  aux tests.
- **`AccountLink` et `LINK_CLASS`** (`features/reports/AccountLink.svelte`, `shared/navigation/link-class.ts`) : tous
  les sites du journal, du grand livre, de la TVA, du compte de résultat à plat ; l'inventaire clos des sites qui restent
  en texte ; la cible (`ledgerHref`) juste pour chaque site (exercice, période).
- **Le sélecteur d'exercice du Grand livre** sur `ReportSelector` (défaut d, #1608) : chargé seul une fois filtré (D16),
  aucune requête sans compte.
- **Le balayage D17** (soulignement léger) : chaque fichier touché par `7559f5f57` (boutons, authentification, profils
  bancaires, factures fournisseur) — aucun changement de comportement, contraste, focus visible.
- **Le manuel** (`.tex` et PDF compilé aplati) contre le code, pour ce que la story change de visible ;
  `docs/api-external.md` si l'API change.
- **Tests** : chaque test prouve-t-il ce qu'il dit (mutations du Dev Agent Record relues) ? un test vert par
  construction ? un critère sans test ?
- **i18n** : quatre locales, aucune concaténation, aucun texte en dur ; compteurs des gardes.
- **Propagation** : le même symptôme ailleurs dans le dépôt (code, quatre locales, manuel, docs).
- **Interaction avec la PR de l'Epic 46 fusionnée** (`c8fe4d0ac`) : ce que la fusion change à la story (même fichier,
  même règle, même clé), pour la seule part de la story.
