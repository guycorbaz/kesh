# Prompt — passe 2 de `bmad-create-story validate`, Story 25-2-b-2

*Versionné le 2026-09-21. Une lentille en contexte frais (Opus) — la passe 1 était sur Sonnet, l'auteur
des patches est un Opus. Ne fais AUCUNE confiance au fait qu'un passage « vient d'être corrigé ».*

Ton objet est `_bmad-output/implementation-artifacts/25-2-b-2-retrait-suppression-ecran-manuels.md`,
dépôt `/home/gcorbaz/devel/kesh`, branche `story/25-2-b-1-devalidation-depot-api` — les deux fiches
filles y vivent, `main` étant au `b7c65be8`.

⛔ **C'est une fille du découpage de la 25-2-b.** La fiche mère `25-2-b-devalidation-facture.md`
(statut `split`) porte les faits établis par **trois** passes de validation ; la fille la
**référence** au lieu de la recopier. **Le risque propre au découpage est ce que la référence
PERD** : un critère tombé entre les deux filles, un prérequis implicite, un décompte recopié d'une
fiche qu'une story ultérieure a périmée.

⚠️ **Ne conteste pas les arbitrages de Guy** (rôle Administrateur **et** Comptable pour dévaliser ;
clés API admises ; `emailed_at` en refus sec ; numéro conservé ; asymétrie des deux sorties — seul
l'Administrateur efface ; le découpage lui-même). Conteste leur mise en œuvre.

## Ce qui a changé depuis la passe 1

La passe 1 a rendu, pour cette fiche, des findings **tous imputables au découpage** — un renvoi trop
vague, un critère orphelin, un décompte périmé, des citations héritées sans être rouvertes. Les
patches ont réécrit plusieurs critères, ajouté des mécanismes qui manquaient, renommé un code
d'erreur, déplacé un compteur d'une fille à l'autre et une tâche d'une story à l'autre. Le Change
Log de la fiche les énumère.

⛔ **Le motif mesuré de ce dépôt : la sévérité se déplace vers ce qu'on vient d'écrire.** Relis
**d'abord** les passages que le Change Log dit avoir été réécrits, et vérifie que la correction n'a
pas créé son propre défaut — notamment : un mécanisme décrit qui ne serait pas réalisable, un
transfert entre les deux filles qui laisse un trou ou un doublon, un décompte corrigé qui reste faux
ailleurs, une citation corrigée qui pointe maintenant autre chose.

⚠️ **Et le signal à qualifier**, puisqu'il décide de la suite : un finding de **découpage** est
attendu et se corrige ; un finding de **conception** — qui vaudrait aussi pour la fiche mère —
relance le critère de non-convergence. **Dis, pour chaque finding, de quelle nature il est.**

## Les axes, et tu déclareras lesquels tu as exercés

1. **Ce que le découpage a perdu.** Confronte la fille à la mère, critère par critère : rien ne doit
   tomber entre elle et `25-2-b-1-devalidation-depot-api.md`, ni y figurer deux fois en se
   contredisant.
2. **L'ordre des deux filles.** La fiche dit que b-1 doit être mergée d'abord. Est-ce **vrai et
   suffisant** ? Un critère de b-2 dépend-il d'autre chose que b-1 ne livre pas ?
3. **Exactitude.** Chaque `fichier:ligne` et chaque nom de test, vérifié **depuis la source**, sur
   `main` actuel.
4. **Les SEPT sites de test à réécrire (AC 3) : sont-ils les bons, et les seuls ?** Refais
   l'inventaire par `grep` de tout ce qui supprime une facture **validée** — tests Rust et
   Playwright compris. Chacun est-il réécrivable contre la dévalidation, ou certains n'ont-ils plus
   d'objet ?
5. **L'écran.** Les trois points de l'AC 4 (bouton, résidu de modale, garde `isAdmin && !paidAt`)
   sont-ils exacts au regard du code actuel ? L'asymétrie « le Comptable dévalide mais n'efface
   pas » est-elle bien celle du code (`admin_routes`, refus des clés API) ? Un E2E existant
   rougira-t-il ?
6. **Les manuels.** La liste des sites est-elle **close** ? Contrôle les **PDF aplatis** des deux
   manuels FR, et cherche un site que la fiche ne nomme pas — `README.md`, `website/`,
   `docs/api-external.md`, la brochure.
7. **Tests, décomptes, périmètre** : la fille reste-t-elle sous le seuil de découpage, et
   n'empiète-t-elle pas sur b-1 ?

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
