# Prompt — validation P5 de la fiche 48-12b « Doublons de factures fournisseur — écran »

Passe **complète** (non ciblée), **modèle Opus** (la P4 était Sonnet), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Recette visuelle*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p1.md`, `validation-12b-p2.md`, `validation-12b-p3.md`, `validation-12b-p4.md` (et ceux de la 48-12a à côté).
- La remédiation de la P4 : le dernier commit de la branche (`git show HEAD`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P4, une par une** (V4-1 à V4-7 de `validation-12b-p4.md`) : prémisse relue au code, puis fermée / fermée en partie / déplacée.
2. **Règle du drapeau `confirmNotDuplicate` (V4-1)** : refais toi-même l'inventaire de **tous** les gestionnaires et appels qui mènent à `confirmUnvalidate`, à `unvalidateSupplierInvoice` et à l'envoi de confirmation du formulaire (`onclick=`, `onsubmit=`, appels directs, `void …()`) ; chacun passe-t-il le drapeau explicitement, jamais un événement ? Le test prescrit (mock qui transmet tous ses arguments, `(id, version)` sans drapeau) tue-t-il la mutation « gestionnaire passé tel quel » ? Les mocks existants à deux arguments sont-ils tous nommés ?
3. **Relevé C (V4-2)** : refais-le depuis le symptôme (`await`, `$effect`, `.then`, `setTimeout`, rappels, props réaffectées après le montage) dans `SupplierInvoiceForm.svelte`, `SupplierCreatePanel.svelte`, `new/+page.svelte`, `[id]/edit/+page.svelte`. Pour chaque site : la garde primaire, **le seul chemin réel** que l'interface atteint (vérifie-le au balisage : ce qui est désactivé, ce qui ne l'est pas), la couche de défense en profondeur et sa voie de test. Une couche dite « inatteignable » qui l'est, ou l'inverse, est un finding ; un test qui ne pourrait se bâtir qu'en affaiblissant une garde primaire aussi. C2 : le contact créé entre-t-il bien dans `suppliers` sans être sélectionné, et cela est-il cohérent avec l'`$effect` (C3) ?
4. **Relevé D** : refais-le (`onclick`, conditions d'affichage par statut, affectations de `invoice`, relectures, boîtes de confirmation) ; D2 a-t-il désormais son test ?
5. **La table d'états, complète** : matrice **chaque état × chaque contrôle** des tables A (E1 à E9) et B (F1 à F6), bouton d'envoi nommé par ses clés, contrôle local en échec en E4/E7, E3 « jusqu'à la résolution de `onSubmit` », E9 ; deux gardes ; transitions manquantes, cases muettes, contradictions.
6. **Décisions de Guy du 2026-11-19 sur E6** et la conséquence à la complétion (V4-3 : renvoi → `IMPORT_NOT_PENDING_COMPLETION` → E8 → E9) : exacte au code serveur (48-12a D4, `imported_supplier_invoices.rs:190-193`) ?
7. **Chaque ligne a un AC et un test, chaque AC une ligne ou la mention « transverse »** ; chaque mutation est tuée par le test de sa ligne (503 et erreur hors `ApiError` en A-E6 ; D2 ; E7 sous AC-3).
8. **Cohérence entre les deux fiches (contrat commun)** : codes, `details`, `confirmable`, `confirmNotDuplicate` sur les quatre corps, trois textes `error-supplier-invoice-*` mot pour mot ; sites du relevé B de la 48-12a attribués à la 48-12b (`user-manual.tex:2362-2370`, `:2682-2704`) contre la section « Manuels et documentation » de cette fiche.
9. **Manuel contre le code** : `docs/manual/fr/user-manual.tex` (sites cités) et **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, en pipe) ; contrôles positifs **et négatif** d'AC-9 (« ne relit pas encore la facture » doit disparaître).
10. **Propagation** : grep des symptômes corrigés en P4 (« scan en attente, puis refus », « jusqu'à la navigation (neuf », « prop que la page peut réaffecter ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log peuvent les garder.
11. **Modules, recyclage, recette visuelle, usage réel** : cinq modules (`SupplierCreatePanel.svelte` non modifié ?) ; un défaut de la P4 revient-il sous une autre forme ? captures de chaque état neuf ou modifié ; un chemin mène-t-il encore à un second brouillon par un renvoi à l'aveugle, à une confirmation silencieuse, ou à une saisie perdue sans que l'écran l'ait dit ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm` ni `npx`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `awk` (sur la sortie standard), `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p5.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V5-n`, sévérité, axe, lieu dans la fiche (ligne de table comprise), **code lu** (ligne citée), preuve, remède.
- La matrice état × contrôle de l'axe 5, telle que tu l'as reconstruite (cases muettes marquées).
- Vérifications positives.
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
- Pour les relevés que tu as refaits (A et B, voies (a) et (b), pour la 48-12a ; C et D pour la 48-12b) : ton propre ensemble, ligne par ligne, comparé à celui de la fiche.
