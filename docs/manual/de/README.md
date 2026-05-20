# Manuels Kesh — Traductions allemandes (Deutsch)

⚠️ **Statut** : Traductions allemandes **non encore créées**. La version canonique est le français dans `../fr/`.

## Comment contribuer une traduction

Voir le guide complet dans `../README.md` section « Traductions DE/IT/EN ».

Résumé pour DE :

1. Copier le fichier source canonique depuis `../fr/` :
   ```bash
   cp ../fr/admin-manual.tex admin-manual.tex
   cp ../fr/user-manual.tex user-manual.tex
   cp ../fr/marketing-brochure.tex marketing-brochure.tex
   ```
2. Adapter le préambule pour charger babel allemand :
   ```latex
   \usepackage[ngerman]{babel}
   ```
3. Traduire intégralement le contenu (titres, sections, paragraphes, listes, captions).
4. Conserver les références techniques inchangées (variables, chemins, commandes shell).
5. Lancer `cd .. && make all-langs` pour vérifier la compilation.
6. Reviewer par native speaker DE (référence Story 9-1 L4 v0.2).

## Stories de traduction prévues

Les traductions DE/IT/EN seront livrées comme stories séparées, planifiées Epic 10+ ou Epic dédié « Internationalisation Documentation v0.2 ». Voir feuille de route projet `_bmad-output/implementation-artifacts/sprint-status.yaml`.
