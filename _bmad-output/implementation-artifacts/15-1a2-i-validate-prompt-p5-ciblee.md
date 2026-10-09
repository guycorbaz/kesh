# Prompt — validation P5 ciblée de la spec, Story 15-1a2-i

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Sonnet), contexte frais, en
lecture seule, braquée sur ce que le dernier commit de remédiation (`18f25973`) a écrit dans CETTE fiche.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0`). **Objet** :
`git show 18f25973 -- _bmad-output/implementation-artifacts/15-1a2-i-lettrage-des-pieces-clients.md`, puis la fiche à `HEAD`. Fiches voisines (contexte) :
`15-1a2-0-lettrage-fige-avec-la-periode.md`, `15-1a2-i-lettrage-des-pieces-clients.md`, `15-1a2-ii-fournisseurs-et-rattrapage.md`,
`15-1b-0-propriete-des-lignes-par-lot.md`, `15-1b-vue-lignes-ouvertes.md`. Registre : `epic-15-choix-autonomes.md`.
Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-validate-p*-*.md`.

⚠️ Un défaut est ce que la fiche prescrit de faux AU CODE de `056997b0` (ou de ce que les fiches voisines créent), une
contradiction avec une fiche voisine, ou un test qui ne prouverait pas ce qu'il dit. Ce qu'une AUTRE story prescrit
n'est pas un manque. Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée.

## Lentille unique — chasseur de régressions de la remédiation

1. **AC18 neuf** (documentation publique du refus, venue de la 15-1a2-0) : relis-le contre ce que la 15-1a2-0 code (D2,
   ses douze textes) et contre le manuel (`.tex` ET PDF aplati : encadré `:588-594`, note `:626-631`, `:2337`, glossaire
   `:2424`, deux listes exhaustives de motifs), `admin-manual.tex:2101`, `api-external.md` (listes, table § 10, `:324`
   dont la 15-1a2-i est l'unique propriétaire), CHANGELOG en « et/ou » : chaque texte prescrit est-il vrai ?
2. **Fixture** sous `crates/kesh-db/tests/support/lettering_documents.rs` incluse par `#[path]`, états hérités en SQL brut
   (C-15-1a2-28) : constructible ? les gardes lexicales ne la voient-elles vraiment pas (elles lisent `crates/*/src`) ?
   `#![allow(dead_code)]` justifié ?
3. **Étape 3 terminale** (C-15-1a2-29) : plus aucun chemin où un compte non lettrable mène à une écriture de marque ?
   le cas ajouté au test `receivable_not_letterable_is_skipped` le prouve-t-il ?
4. Recomptes (13 AC, 7 tâches, 27 tests neufs + 1 étendu ; 5 modules).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés.** Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-validate-p5-ciblee.md` ;
dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*`, `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`, `cargo`, `npm`, `npx`, `docker`,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard.
