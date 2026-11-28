# Prompt — validation Pv2p4 de la fiche 55-3b (passe ciblée)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit), en production chez son mainteneur. Epic 55, « Navigation de
l'interface » : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (il fait foi pour le périmètre et les
décisions D1 à D11). Worktree `<worktree>/kesh-e55`, branche `epic/55-navigation`, partie de
`origin/main` à `1e23edb55`. Le développement repartira de la PR de l'Epic 46, non fusionnée : lis-la par
`git show pr/46-rapports-groupes:<fichier>` (`511e61d54`), `git show story/46-export-api-groupe:<fichier>` (`8ca669b3f`),
`git show story/46-5:<fichier>` (`0f483c7bb`) ; l'Epic 47 par `git show epic/47-comptes-miroirs:<fichier>`.

Passe **P4 de la version récrite (v2)**, passe ciblée, **contexte frais**. Langue du rapport : français.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Review Iteration Rule* et ses sous-sections, § *Test Locally First* (E2E, recette visuelle),
   § *Travailler à plusieurs développeurs*, § *Issue Tracking Rule*, § *Priorités des défauts*.
2. Le fichier d'epic en entier.
3. La fiche : `_bmad-output/implementation-artifacts/55-3b-justificatifs-ecran.md`.
4. Le code qu'elle cite, **ligne par ligne** : chaque `chemin:ligne` se vérifie à la base indiquée.
5. La fiche 55-3a (contrat `documents`), `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte` et son test, `frontend/src/lib/shared/utils/pdf-error.ts`, `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (téléchargement du PDF), `docs/manual/fr/user-manual.tex` § 11.6.1 (`sec:pdf-fige`).

**Passe ciblée** : une seule lentille, braquée sur la remédiation de la passe précédente, commit `4c06fee90` (`git show 4c06fee90`). Vérifie que chaque correctif est juste au code, qu'il ne crée pas de défaut neuf, et qu'il est propagé (même symptôme ailleurs dans la fiche, le fichier d'epic et les autres fiches). Les axes A1 à A9 ne s'exercent que sur ce que touche ce commit.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Prémisses relues au code** : chaque affirmation sur le code (chemin, ligne, comportement, droits d'une route,
  champ d'un DTO) est-elle juste ? Recopie la ligne lue pour chaque point contrôlé.
- **A2 — Périmètre** contre le fichier d'epic et les autres fiches : recouvrements, trous, contradictions entre fiches
  (paramètres d'URL, composants, clés i18n, ordre des stories).
- **A3 — Critères constatables** : chaque AC dit-il comment on le constate (test nommé, commande, valeur attendue) ?
  Un AC invérifiable, ambigu ou vrai par construction est un finding.
- **A4 — Cas limites et modes d'échec** : courses, valeurs invalides, rôles, sociétés multiples, données absentes,
  boucles de navigation, interactions avec l'Epic 46 (et 47) que Git fusionnerait sans conflit.
- **A5 — Preuve du rouge et tests** : chaque test neuf rougit-il sans le correctif ? Un test muet possible ?
- **A6 — Manuel** : la fiche nomme-t-elle les bons paragraphes de `docs/manual/fr/user-manual.tex` (lis-les), et le PDF
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) contient-il une phrase que la story rendra
  fausse sans que la fiche le dise ?
- **A7 — Règles du projet** : DRY, i18n quatre locales et gardes, captures, splitting préventif (> 5 modules), note de
  version (aucun numéro d'epic dans le texte destiné aux utilisateurs, `refs #`), aucun mot-clé de fermeture.
- **A8 — Le composant** : `DocumentPreview` étendu sans changer ses appelants ; dialogue accessible ; `load` stable ;
  messages d'erreur par genre ; noms de fichiers.
- **A9 — E2E** : le chemin d'une écriture à deux justificatifs par l'API existe-t-il (Epic 43 : règlements groupés,
  `crates/kesh-api/src/lib.rs`, `routes/reconciliation_group.rs`) ? L'E2E du Journal rougit-il sans le correctif ?
- **A10 — Interactions** avec la 55-1 (URL, mémoire) et la 55-2 (mêmes vues, bloc `reports-nav-*`).

Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement, la vérification ou le produit ;
LOW pour la rédaction sans effet. Ne gonfle pas, ne minimise pas.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `push`, `checkout`, `switch`, `stash`, `merge`,
`reset`, `worktree`. **Aucun `gh`.** Ne touche à aucun port ni service. `grep`, `git show`, `git log`, `sed -n`, `pdftotext`,
`python3 -I` en lecture sont admis.

## Rapport

Écris **un seul fichier** : `$GATE_LOGS/s55-kickoff/55-3b-validation-pv2p4.md`, avec :
- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant `Vv2p4-N`, sévérité, ce qui est faux, la preuve (ligne lue), le correctif proposé ;
- la **liste des axes exercés et non exercés** (une passe qui ne la donne pas ne compte pas).
Puis rends ce même contenu en réponse, en entier.
