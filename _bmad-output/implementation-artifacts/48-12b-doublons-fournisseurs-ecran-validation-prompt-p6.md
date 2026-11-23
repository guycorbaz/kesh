# Prompt — validation P6 de la fiche 48-12b « Doublons de factures fournisseur — écran »

Passe **complète** (non ciblée), **modèle Sonnet** (la P5 était Opus), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Recette visuelle*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p1.md`, `validation-12b-p2.md`, `validation-12b-p3.md`, `validation-12b-p4.md`, `validation-12b-p5.md` (et ceux de la 48-12a à côté).
- La remédiation de la P5 : le dernier commit de la branche (`git show HEAD`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P5, une par une** (V5-1 à V5-9 de `validation-12b-p5.md`) : prémisse relue au code, puis fermée / fermée en partie / déplacée.
2. **Règle du drapeau et forme d'appel (V5-1)** : refais l'inventaire de tous les appels qui mènent à `confirmUnvalidate`, à `unvalidateSupplierInvoice` et à l'envoi de confirmation du formulaire. La forme fixée — `(id, version)`, ou `(id, version, true)` seulement si `=== true` — est-elle cohérente avec les trois assertions existantes (`supplier-invoice-draft-page.test.ts:265`, `:300`, `:378`) une fois les mocks élargis ? Le test unitaire neuf de `supplier-invoices.api.ts` est-il réalisable au patron de `imported-supplier-invoices.api.test.ts` ? Pour **chaque** mutation nommée (la mutation de page qui retire ses deux défenses à la fois ; la mutation de l'API), quel test rougit, et rougit-il **seul** ? Une mutation équivalente nommée, ou un test qui ne rougirait qu'en affaiblissant une garde, est un finding.
3. **Règle générale de défense en profondeur** (Règles communes) : est-elle appliquée partout (relevé C, règle du drapeau) ? Une couche dite « inatteignable » ou « inobservable » l'est-elle vraiment au balisage et au code ?
4. **Relevé C (C1 à C8)** : refais-le depuis le symptôme (`await`, `$effect`, `.then`, `setTimeout`, rappels, props réaffectées après le montage) ; C3 « atteignable par C2 mais idempotent » est-il exact ? le test par re-rendu tue-t-il « `$effect` sans garde » ?
5. **Relevé D et table B** (F1 à F6, F5 relecture en échec) : refais le relevé ; chaque case de la table B est-elle dite ?
6. **Texte de E9 (décision de Guy du 2026-11-19)** : « Cette pièce a déjà été enregistrée comme facture. Ouvrez-la pour vérifier qu'elle correspond à votre saisie. », avec le lien ; est-il propagé partout (table A, AC-5, AC-8, AC-9, tests, E2E, captures, manuel) et l'ancien texte « telle qu'envoyée la première fois » a-t-il disparu hors des entrées historiques du Change Log ? Le cas « autre session » (`user-manual.tex:2715-2717`) est-il traité ?
7. **Répartition du manuel avec la 48-12a (V5-8)** : chaque rang du relevé B appartient-il à une seule fiche, dans les bornes de son paragraphe (rangs 91, 106, 107 ici ; 68, 80, 84, 88 à la 48-12a avec paragraphes neufs à la suite) ? Compare au tableau du relevé B de `48-12a-doublons-fournisseurs-serveur.md` (colonne « oui — 48-12a / 48-12b »).
8. **La table d'états, complète** : matrice chaque état × chaque contrôle des tables A et B ; gardes ; transitions manquantes, cases muettes, contradictions.
9. **Chaque ligne a un AC et un test, chaque AC une ligne ou la mention « transverse »** ; chaque mutation est tuée par le test de sa ligne.
10. **Cohérence entre les deux fiches (contrat commun)** : codes, `details`, `confirmable`, `confirmNotDuplicate` sur les quatre corps, trois textes `error-supplier-invoice-*` mot pour mot.
11. **Manuel contre le code** : `docs/manual/fr/user-manual.tex` (sites cités) et **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, en pipe) ; contrôles positifs et négatifs d'AC-9, `.tex` aplati aussi pour le négatif.
12. **Propagation** : grep des symptômes corrigés en P5 (« telle qu'envoyée la première fois », « Résolution | », « :241-249 », « :309-314 », « :316-322 », « :373-386 », « règle du drapeau, D3 », « jamais un faux vert ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log peuvent les garder.
13. **Modules, recyclage, recette visuelle, usage réel** : cinq modules ; un défaut de la P5 revient-il sous une autre forme ? captures (E1 inactif, notification de C2 ; C1 dite impossible) ; un chemin mène-t-il à un second brouillon par un renvoi à l'aveugle, à une confirmation silencieuse, ou à une saisie perdue sans que l'écran l'ait dit ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm` ni `npx`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `awk` (sur la sortie standard), `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p6.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V6-n`, sévérité, axe, lieu dans la fiche (ligne de table comprise), **code lu** (ligne citée), preuve, remède.
- La matrice état × contrôle de l'axe 8, telle que tu l'as reconstruite (cases muettes marquées).
- Vérifications positives.
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
- Pour les relevés que tu as refaits (A et B, voies (a) et (b), pour la 48-12a ; C et D pour la 48-12b) : ton propre ensemble, ligne par ligne, comparé à celui de la fiche.
