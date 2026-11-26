# Prompt — validation P8 de la fiche 55-1a (ciblée, réponses de l'utilisateur)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit), en production chez son mainteneur. Epic 55, « Navigation de
l'interface » : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (il fait foi pour le périmètre et les
décisions D1 à D12). Worktree `<worktree>/kesh-e55`, branche `epic/55-navigation`, partie de
`origin/main` à `1e23edb55`. Le développement repartira de la PR de l'Epic 46, non fusionnée : lis-la par
`git show pr/46-rapports-groupes:<fichier>` (`511e61d54`), `git show story/46-export-api-groupe:<fichier>` (`8ca669b3f`),
`git show story/46-5:<fichier>` (`0f483c7bb`) ; l'Epic 47 par `git show epic/47-comptes-miroirs:<fichier>`.

Passe **8**, ciblée, réponses de l'utilisateur, **contexte frais**. Langue du rapport : français.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Review Iteration Rule* et ses sous-sections, § *Test Locally First* (E2E, recette visuelle),
   § *Travailler à plusieurs développeurs*, § *Issue Tracking Rule*, § *Priorités des défauts*.
2. Le fichier d'epic en entier.
3. La fiche : `_bmad-output/implementation-artifacts/55-1a-etat-des-rapports-dans-l-url.md`.
4. Le code qu'elle cite, **ligne par ligne** : chaque `chemin:ligne` se vérifie à la base indiquée.

**Les réponses de l'utilisateur à verser** (le commit ciblé les verse ; c'est contre elles que la passe juge la fidélité) :
1. Revenir à un onglet déjà généré montre déjà son contenu aujourd'hui ; reste qu'une adresse collée, rechargée ou atteinte
   par Retour affiche son rapport sans *Générer* (D3). L'AC-13 de la 55-1a est reformulé en ce sens ; rien ne doit laisser
   croire à une régénération au changement d'onglet.
2. « Pas de requête en double » (R-4 de la 55-1a) était une formulation technique, non une demande de l'utilisateur :
   lecture par défaut gardée comme exigence technique seulement.
3. D10 : un règlement montre le PDF de la facture réglée, fournisseur **et** client ; l'écriture d'une facture client montre
   son PDF. Plus tard (non livré) : un PDF produit hors de Kesh pour une facture créée par l'API s'affichera à la place du
   PDF généré — limite connue et évolution, le modèle de la liste ne doit pas l'empêcher.
4. Q7 : depuis le Journal et les Rapports, afficher une facture **ne la fige pas** ; bandeau « non figé » dans la
   visionneuse (terme de Kesh) ; voie d'aperçu sans figement à vérifier au code, sinon créée par la 55-3a, sans trace de
   téléchargement ni figement, droits *Consultation* respectés. Fiche de la facture : rien ne change (son affichage : question
   au mainteneur, non tranchée).
5. 55-3b : avec un seul document, aucune liste, la visionneuse l'ouvre directement ; la liste en haut à partir de deux.
Le fichier d'epic (décisions, réponses, questions Q1, Q3, Q5, Q6, Q8 à Q10) fait partie du périmètre de la passe.

**Passe ciblée** : une seule lentille, braquée sur le commit `67d3ce58c` (`git show 67d3ce58c`), qui verse ces réponses. Vérifie que chaque changement est juste au code (en particulier : existe-t-il déjà une voie d'aperçu d'une facture sans figement ? lis `crates/kesh-api/src/routes/issued_invoice_pdf.rs`, `invoice_pdf.rs`, `invoice_pdf_service.rs`, `lib.rs`), fidèle aux réponses, sans défaut neuf, et propagé (même symptôme ailleurs dans la fiche, le fichier d'epic et les autres fiches 55-*). Les axes ne s'exercent que sur ce que touche ce commit.

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
- **A8 — Le modèle d'URL** : le tableau des paramètres, l'ordre canonique, la table des gestes et D3 forment-ils un
  modèle cohérent et sans boucle (une écriture de `goto` relue par l'effet) ? `ledgerHref` reste-t-il égal à l'octet près ?
  Le comportement de SvelteKit 2 (`goto` sur la même route, `page.url` de `$app/state`, `load` non rejoué) est-il bien
  décrit (lis `<dépôt>/frontend/node_modules/@sveltejs/kit/src/runtime/client/client.js`, dépôt principal, en lecture si besoin) ?
- **A9 — Le regroupement de l'Epic 46** : l'intégration avec `reports-grouping.ts` et `acceptGrouped` (PR 46) est-elle
  juste et complète (mémoire, écho, export de la dernière réponse affichée) ?

Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement, la vérification ou le produit ;
LOW pour la rédaction sans effet. Ne gonfle pas, ne minimise pas.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `push`, `checkout`, `switch`, `stash`, `merge`,
`reset`, `worktree`. **Aucun `gh`.** Ne touche à aucun port ni service. `grep`, `git show`, `git log`, `sed -n`, `pdftotext`,
`python3 -I` en lecture sont admis.

## Rapport

Écris **un seul fichier** : `$GATE_LOGS/s55-maquettes/55-1a-validation-p8.md`, avec :
- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant `V8-N`, sévérité, ce qui est faux, la preuve (ligne lue), le correctif proposé ;
- la **liste des axes exercés et non exercés** (une passe qui ne la donne pas ne compte pas).
Puis rends ce même contenu en réponse, en entier.
