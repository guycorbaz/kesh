# Prompt — passe 1 de `bmad-create-story validate`, Story 25-2-b-1

*Versionné le 2026-09-21. Une lentille en contexte frais (Opus), orthogonale à l'auteur de la
fiche — également un Opus : ne fais AUCUNE confiance au fait qu'un passage « vient d'une fiche déjà
validée trois fois ».*

Ton objet est `_bmad-output/implementation-artifacts/25-2-b-1-devalidation-depot-api.md`, dépôt
`/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` (tirée de `main` au
`b7c65be8`, qui porte déjà la 25-2-b-zero).

⛔ **C'est une fille du découpage de la 25-2-b.** La fiche mère `25-2-b-devalidation-facture.md`
(statut `split`) porte les faits établis par **trois** passes de validation ; la fille la
**référence** au lieu de la recopier. **Le risque propre au découpage est donc ce que la référence
PERD** : un renvoi trop vague pour être implémenté, un critère tombé entre les deux filles, un
prérequis implicite, un décompte recopié d'une fiche qu'une story ultérieure a périmée.

⚠️ **Ne conteste pas les arbitrages de Guy** (rôle Administrateur **et** Comptable ; clés API
admises, « même approche que Bexio » ; `emailed_at` en refus sec ; numéro conservé ; brouillon
numéroté qui ne change pas d'exercice ; le découpage lui-même). Conteste leur mise en œuvre.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Ce que le découpage a perdu.** Confronte la fille à la mère, critère par critère : chaque
   critère de la mère est-il dans l'une des deux filles
   (`25-2-b-2-retrait-suppression-ecran-manuels.md`), sans trou ni doublon contradictoire ? Un
   renvoi (« cf. fiche mère, AC 3 ») suffit-il pour implémenter sans rouvrir un débat tranché ?
2. **L'état intermédiaire assumé.** Entre b-1 et b-2, deux chemins détruisent l'écriture d'une
   facture. Est-ce réellement sans danger — audit, numérotation, verrou de période ? Quels tests
   existants rougiraient **dès b-1 seule**, alors que la fiche les attribue à b-2 ?
3. **Exactitude.** Chaque `fichier:ligne`, chaque nom, chaque compteur (105→106, 108→109,
   `traced` 87→88, « 123 → 124 libellés ») vérifié **depuis la source**, sur `main` actuel.
4. **La route et les clés API.** `comptable_routes` donne-t-il bien Administrateur **et** Comptable,
   et une clé `read-write` y passe-t-elle vraiment ? Quelle garde ou quel registre devra bouger ?
5. **Les empêchements et leur précédence**, tels que la mère les fixe : réalisables dans
   `unvalidate` sans dupliquer les lectures de `delete_in_tx` ? Le test de précédence est-il
   constructible de part et d'autre de l'appel ?
6. **Le numéro conservé et la garde d'exercice du `PUT`** : réalisables ? Quels sites lisent
   `invoice_number` et supposent à tort qu'un numéro implique un statut validé ?
7. **Tests, décomptes, périmètre** : la fille reste-t-elle sous le seuil de découpage, et
   n'anticipe-t-elle rien de b-2 ?

## Ce que tu rends

- **Les findings** : sévérité, l'endroit exact, **la preuve**, ce qu'il faut changer. Un reproche au
  code de ne pas encore faire ce que la story prescrit n'est **pas** un défaut de la spec.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.**

## Interdits

⛔ N'écris aucun fichier du dépôt, n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante — `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`, `scripts/install-hooks.sh`,
`scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make` dans `docs/manual/`, tout `git commit`/`push`/
`add`/`stash`/`reset`/`rebase`, `sqlx migrate`, `cargo test`/`cargo nextest`. Lecture, `grep`,
`git log`/`show`/`diff`, `gh issue view`, `pdftotext` et `cargo check` sont autorisés.
