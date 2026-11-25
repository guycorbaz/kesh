# Prompt — validation P3 de la fiche 48-12a « Doublons de factures fournisseur — serveur »

Passe **complète** (non ciblée), **modèle Opus** (la P2 était Sonnet), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Migration breaking policy*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p1.md`, `validation-12a-p2.md` (et ceux de la 48-12b à côté).
- La remédiation de la P2 : le dernier commit de la branche (`git show HEAD`, `git log -3`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P2, une par une** (V2-1 à V2-11 de `validation-12a-p2.md`) : pour chacune, lis la prémisse au code (`grep -nF`, lecture directe), puis dis si la fiche la ferme, la ferme en partie, ou la déplace. En particulier :
   - V2-1 : `lock_draft_and_supplier_in_tx` — est-il appelable depuis la route **avant** la lecture du document joint (`routes/supplier_invoices.rs:817-818`) avec le contact du corps ? Le contact vient-il bien toujours du corps (`new_from_document`, `to_new_supplier_invoice`) ? Le test HTTP de `kesh-api` peut-il tenir une transaction brute sur le pool de l'application pendant qu'une requête part dans une tâche (montage des tests de `kesh-api` à lire) ? La mutation « lecture du document avant le verrou » est-elle tuée ?
   - V2-2 : `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:593`) — les motifs prescrits correspondent-ils au SQL prescrit de `lock_supplier_in_tx` et à `FIND_SCOPED_SQL` ? Reste-t-il une mention prescriptive d'`innodb_lock_waits` ou d'`INNODB_TRX` hors historique ?
   - V2-5 : l'ordre « facture puis fournisseur » au `PUT` et à la dévalidation, « fournisseur » seul à la création, « staging puis fournisseur » à la complétion — cherche toi-même une arête fournisseur → facture (ou fournisseur → staging) sur tout le dépôt (`git grep` des `FOR UPDATE` et `LOCK IN SHARE MODE`, des `INSERT`/`UPDATE` qui prennent des verrous de clé étrangère sur `contacts`) ; le test « pas d'interblocage » est-il réalisable et sa mutation tuée ?
   - V2-6 : la place du contrôle entre les étapes (6) et (7) de `unvalidate_in_tx` (lis les dix étapes) : avant toute écriture ? après toutes les lectures ordinaires nécessaires ? la précédence écrite en D4 est-elle exacte contre le doc-comment « Précédence » et `docs/api-external.md:671` ?
   - V2-8 : `lock_supplier_in_tx` qui rend `false` — la précédence de `check_draft_input_in_tx` (`:502-`) est-elle vraiment inchangée ? Un test existant épingle-t-il une paire que le verrou déplacerait ?
   - V2-9 : les signatures conservées et les variantes `…_with_policy` ; recompte `git grep -c "unvalidate_in_tx(\|supplier_invoices::unvalidate(" -- 'crates/*/tests'` (la fiche écrit 31 sites dans 7 fichiers).
2. **Décision de Guy du 2026-11-19 — dévalidation d'une annulée qui ferait doublon** : refus par défaut, levé par confirmation explicite (`confirmNotDuplicate`, pour les deux règles), consigné à l'audit (`par(acteur)`), geste du manuel « repasser en brouillon puis supprimer » possible. Vérifie : cohérence de la justification du même champ ; `details.confirmable` ; le message propre (`error-supplier-invoice-duplicate-unvalidate`, sans « 15-b ») ; la conséquence écrite (brouillon validé sans changement non contrôlé) ; qu'un comptable confirme mais que seul un administrateur supprime (lis le code de `delete_draft` et sa garde de rôle dans la route) ; que le scénario fermé de D6bis l'est encore.
3. **Décision de Guy du 2026-11-19 — issue inconnue à l'écran (48-12b)** : la 48-12a fournit-elle tout ce dont l'état E6 et la table B de la 48-12b ont besoin ? Rien dans la 48-12a ne suppose-t-il encore un renvoi après une issue inconnue ?
4. **Cohérence entre les deux fiches (contrat d'API commun)** : codes, `details : { supplierInvoiceId, internalNumber, status, confirmable }` et la valeur de `confirmable` par geste et par code, `confirmNotDuplicate` sur les quatre corps (création, `PUT`, complétion, dévalidation) et son effet, les clés de message et leur texte, la règle de sortie commune. Toute différence de formulation entre les deux fiches est un finding.
5. **Règles de la story** (décisions déjà prises, à ne pas rouvrir, mais à contrôler dans le texte) : critère 1 par année civile, critère 2 avec confirmation auditée, règle 3 (« l'une au moins sans numéro »), annulées exclues, numéro facultatif, rejeu à la modification sur les quatre éléments seulement, aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` inchangé ; aucune instruction de la fiche n'en exige une), sortie commune.
6. **Tests et preuve du rouge** : chaque ligne du tableau nomme-t-elle un binaire qui peut appeler ce qu'elle teste ? chaque mutation est-elle tuée par le test de sa ligne (raisonne sous REPEATABLE READ) ? une assertion vraie par construction ?
7. **Manuel et documentation contre le code** : `docs/manual/fr/user-manual.tex` (les sites cités : `:2072-2090`, `:2195`, `:2242-2265`, `:2477`, `:2512`, `:4735-4738`), `docs/manual/fr/admin-manual.tex` (« Pour les intégrations »), `docs/api-external.md` (`:659`, `:661`, `:664`, `:671`, `:1015`, `:1021`) ; **PDF compris**, aplati avant de greper : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (pipe vers `grep`, **jamais** de redirection vers un fichier). Chaque numéro de ligne cité par la fiche est-il juste ? Une phrase du manuel actuel contredit-elle la règle à venir sans que la fiche la désigne ? Inventorie l'ensemble clos des phrases du manuel qui parlent du repassage en brouillon d'une annulée et de la suppression d'un brouillon.
8. **Propagation** : grep des symptômes corrigés en P2 (`innodb_lock_waits`, `INNODB_TRX`, `:888`, « fournisseur d'abord », « sans numéro des deux », « test unique », « dès après `pool.begin()` ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log et les décisions écartées peuvent les garder.
9. **Modules et splitting** : le décompte de cinq modules ; le signal de recyclage (un défaut de la P2 revient-il sous une autre forme, né de cette remédiation ?).
10. **Effet sur l'utilisateur réel** (Guy, livres réels, lots pain.001) : un chemin laisse-t-il encore passer un doublon payable deux fois ? un geste légitime est-il fermé sans sortie ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, pas de `sqlx`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p3.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V3-n`, sévérité, axe, lieu dans la fiche, **code lu** (ligne citée), preuve, remède.
- Vérifications positives (ce qui a été contrôlé et tient).
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
