# Prompt — validation P4 ciblée de la spec, Story 15-1b

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Haiku), contexte frais, en
lecture seule, braquée sur ce que le dernier commit de remédiation (`76e7893a`) a écrit dans CETTE fiche.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0`). **Objet** :
`git show 76e7893a -- _bmad-output/implementation-artifacts/15-1b-vue-lignes-ouvertes.md`, puis la fiche à `HEAD`. Fiches voisines (contexte) :
`15-1a2-0-lettrage-fige-avec-la-periode.md`, `15-1a2-i-lettrage-des-pieces-clients.md`, `15-1a2-ii-fournisseurs-et-rattrapage.md`,
`15-1b-0-propriete-des-lignes-par-lot.md`, `15-1b-vue-lignes-ouvertes.md`. Registre : `epic-15-choix-autonomes.md`.
Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/15-1b-validate-p*-*.md`.

⚠️ Un défaut est ce que la fiche prescrit de faux AU CODE de `056997b0` (ou de ce que les fiches voisines créent), une
contradiction avec une fiche voisine, ou un test qui ne prouverait pas ce qu'il dit. Ce qu'une AUTRE story prescrit
n'est pas un manque. Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée.

## Lentille unique — chasseur de régressions de la remédiation

1. Les LOW P3 appliqués changent deux comportements prescrits — le TRI de la vue (aligné sur le Grand livre,
   `line_order`) et le filtre R7 « paires acceptables » placé AVANT l'appariement glouton : les AC et les tests
   (dont le test 12) disent-ils la même chose ? un test distingue-t-il les deux ordres ?
2. L'alignement sur `DocumentKind::blocks_manual_lettering()`, `as_str()` (15-1b-0) et `DocumentRef` (15-1a2-i) : les
   noms et signatures cités existent-ils dans ces fiches voisines, sous ces noms exacts ?
3. C-15-1b-13 (pas d'`offset` pour les propositions, pas d'audit de lecture) : écrit comme choix, sans contradiction
   avec un AC ?
4. Les recomptes de la fiche (12 AC, 6 tâches, 19 tests) : recompte-les depuis le fichier.

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés.** Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-validate-p4-ciblee.md` ;
dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*`, `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`, `cargo`, `npm`, `npx`, `docker`,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard.
