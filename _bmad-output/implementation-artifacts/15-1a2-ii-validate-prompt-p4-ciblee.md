# Prompt — validation P4 ciblée de la spec, Story 15-1a2-ii

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Sonnet), contexte frais, en
lecture seule, braquée sur ce que le dernier commit de remédiation (`76e7893a`) a écrit dans CETTE fiche.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0`). **Objet** :
`git show 76e7893a -- _bmad-output/implementation-artifacts/15-1a2-ii-fournisseurs-et-rattrapage.md`, puis la fiche à `HEAD`. Fiches voisines (contexte) :
`15-1a2-0-lettrage-fige-avec-la-periode.md`, `15-1a2-i-lettrage-des-pieces-clients.md`, `15-1a2-ii-fournisseurs-et-rattrapage.md`,
`15-1b-0-propriete-des-lignes-par-lot.md`, `15-1b-vue-lignes-ouvertes.md`. Registre : `epic-15-choix-autonomes.md`.
Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-ii-validate-p*-*.md`.

⚠️ Un défaut est ce que la fiche prescrit de faux AU CODE de `056997b0` (ou de ce que les fiches voisines créent), une
contradiction avec une fiche voisine, ou un test qui ne prouverait pas ce qu'il dit. Ce qu'une AUTRE story prescrit
n'est pas un manque. Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée.

## Lentille unique — chasseur de régressions de la remédiation

1. Les tests modifiés passent à 5 (`full_import_report_mirrors_the_production_registry`, issue déduite de
   `entry.trigger`) : la prescription est-elle juste pour une entrée de classe A ET pour une entrée exemptée ?
2. La réécriture de la définition de la classe A (`post_restore.rs:41-43`, `:121`, `post_restore_class_a.rs:358`) :
   nommée dans les tâches, et le texte cible est-il vrai ?
3. Le rang 2 bis porté à la 15-1a2-0 : la 15-1a2-ii ne garde-t-elle aucune prescription en double ou contradictoire
   (gestes fournisseurs, listes `api-external.md`) ?
4. `documentType = "supplierInvoice"`, `DocumentRef.number = None` : cohérents avec la 15-1a2-i et la 15-1b ?
5. Les LOW appliqués (F3-3, F3-4, F3-6, F3-7, R3-5, R3-7) : chacun est-il juste au code ?

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés.** Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-ii-validate-p4-ciblee.md` ;
dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*`, `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`, `cargo`, `npm`, `npx`, `docker`,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard.
