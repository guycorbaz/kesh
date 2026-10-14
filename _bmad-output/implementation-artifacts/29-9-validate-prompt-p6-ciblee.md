# 29-9 — passe ciblée P6 de validation (une lentille)

**Objet** : le seul commit `dd885f3f` (remédiation de la passe ciblée P5, rapport
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p5-ciblee.md`). Lire `git show dd885f3f` en un
seul diff ; ne pas relire la story entière.

**Lecture seule stricte** : aucun `git checkout`/`switch`/`stash`/`reset`, aucune commande qui écrit,
compile ou exécute un test ou un script (`scripts/prepare-release.sh` compris) ; aucune redirection
vers un fichier, aucun fichier temporaire. Permis : `git show`, `git diff`, `git log`,
`git -C … show <rev>:<chemin>`, `grep`, `sed -n`. Seul fichier écrit : le rapport
`/home/gcorbaz/devel/kesh-gate-logs/29-9-validate-p6-ciblee.md`. Toute affirmation de présence ou
d'absence s'étaie par une sortie `grep -nF` copiée, sur le fichier FINAL.

## Axes

1. **P5-1** : l'affirmation fausse a-t-elle disparu de l'AC8 et de l'AC11 (grep `repli-divergent`,
   `REPLIS_A_SITE_UNIQUE` dans la fiche) ? Les précédents cités (`UnrejectTransactionAction.test.ts:31`,
   `ReconciliationProposals.test.ts:672`) lisent-ils réellement le catalogue fr-CH ? L'angle mort de
   l'AC8 est-il écrit comme tel, sans promesse ailleurs ?
2. **P5-2** : la description de chaque garde est-elle exacte au code (`i18n-keys.test.ts:635`, `:899`,
   valeur 31 de `sitesNonResolus`) ?
3. **P5-4/P5-5** : V23 existe-t-il, tue-t-il M30 ? Les champs `p.transaction.bookingDate`, `.amount`,
   `.currency`, `.counterpartyName` existent-ils dans le type de la proposition du composant concerné ?
4. **P5-6** : la commande de contrôle (:25) est-elle une regex valide qui reste vide sur
   `git log main..HEAD` et qui attraperait `closes #1`, `fixes guycorbaz/kesh#1`, une URL d'issue ?
5. **Propagation et recomptes** : V1–V23, M1–M30, C-29-9-1 à 25, 11 AC, T0–T10 ; aucune entrée datée
   réécrite ; aucun résidu vivant « V19-V22 » ou « M22-M27 ».

## Rapport

Findings CRITICAL/HIGH/MEDIUM/LOW avec fichier:ligne et sortie copiée ; **liste des axes exercés et
non exercés** ; décompte final.
