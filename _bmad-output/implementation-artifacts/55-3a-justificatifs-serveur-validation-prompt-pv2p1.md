# Prompt — validation Pv2p1 de la fiche 55-3a (passe complète sur la version récrite)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit), en production chez son mainteneur. Epic 55, « Navigation de
l'interface » : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (il fait foi pour le périmètre et les
décisions D1 à D11). Worktree `<worktree>/kesh-e55`, branche `epic/55-navigation`, partie de
`origin/main` à `1e23edb55`. Le développement repartira de la PR de l'Epic 46, non fusionnée : lis-la par
`git show pr/46-rapports-groupes:<fichier>` (`511e61d54`), `git show story/46-export-api-groupe:<fichier>` (`8ca669b3f`),
`git show story/46-5:<fichier>` (`0f483c7bb`) ; l'Epic 47 par `git show epic/47-comptes-miroirs:<fichier>`.

Passe **P1 de la version récrite (v2)**, passe complète sur la version récrite, **contexte frais**. Langue du rapport : français.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Review Iteration Rule* et ses sous-sections, § *Test Locally First* (E2E, recette visuelle),
   § *Travailler à plusieurs développeurs*, § *Issue Tracking Rule*, § *Priorités des défauts*.
2. Le fichier d'epic en entier.
3. La fiche : `_bmad-output/implementation-artifacts/55-3a-justificatifs-serveur.md`.
4. Le code qu'elle cite, **ligne par ligne** : chaque `chemin:ligne` se vérifie à la base indiquée.
5. `crates/kesh-db/src/repositories/imported_supplier_invoices.rs` (`find_source_document_of_entry`), `crates/kesh-report/src/journal_report.rs`, `general_ledger.rs`, les migrations citées, `crates/kesh-api/src/lib.rs` (droits des routes), `frontend/src/lib/features/imported-supplier-invoices/DocumentPreview.svelte`.

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
- **A8 — Le rattachement** : l'inventaire des documents rattachés à une écriture est-il **clos** (cherche toute colonne
  `*journal_entry_id*` et tout fichier stocké : `storage_path`, `pdf_*`, et sur la PR 46 les imports PayPal/Wise/Stripe) ?
  Les règles (statuts, règlement annulé, facture annulée, règlement groupé) sont-elles justes au schéma ?
- **A9 — La requête** : `UNION ALL` bornée par période et société : sans N+1, juste pour le grand livre (période libre,
  plusieurs exercices) ? Le contrat additif (`skip_serializing_if`) garde-t-il vraiment les exports et le MCP inchangés ?
- **A10 — Sécurité et droits** : IDOR, rôle *Consultation*, effet de bord du figement d'une facture par la route PDF.
- **A11 — Backend** : la fiche dit-elle assez clairement au mainteneur que l'epic n'est plus « frontend seulement » ?

Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement, la vérification ou le produit ;
LOW pour la rédaction sans effet. Ne gonfle pas, ne minimise pas.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `push`, `checkout`, `switch`, `stash`, `merge`,
`reset`, `worktree`. **Aucun `gh`.** Ne touche à aucun port ni service. `grep`, `git show`, `git log`, `sed -n`, `pdftotext`,
`python3 -I` en lecture sont admis.

## Rapport

Écris **un seul fichier** : `$GATE_LOGS/s55-kickoff/55-3a-validation-pv2p1.md`, avec :
- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant `Vv2p1-N`, sévérité, ce qui est faux, la preuve (ligne lue), le correctif proposé ;
- la **liste des axes exercés et non exercés** (une passe qui ne la donne pas ne compte pas).
Puis rends ce même contenu en réponse, en entier.
