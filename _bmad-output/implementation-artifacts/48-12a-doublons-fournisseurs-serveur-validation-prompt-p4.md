# Prompt — validation P4 de la fiche 48-12a « Doublons de factures fournisseur — serveur »

Passe **complète** (non ciblée), **modèle Sonnet** (la P3 était Opus), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Migration breaking policy*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p1.md`, `validation-12a-p2.md`, `validation-12a-p3.md` (et ceux de la 48-12b à côté).
- La remédiation de la P3 : le dernier commit de la branche (`git show HEAD`, `git log -3`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P3, une par une** (V3-1 à V3-10 de `validation-12a-p3.md`) : prémisse relue au code (`grep -nF`, lecture directe), puis fermée / fermée en partie / déplacée.
2. **Relevé A (D3bis) — ensemble clos des chemins qui verrouillent une ligne de `contacts`** : refais-le toi-même **depuis le symptôme**, sans partir du tableau de la fiche : tables enfants de `contacts` (`grep -niE "references +contacts *\(" crates/kesh-db/migrations/*.sql`), toutes les écritures de ces tables et de `contacts` (`git grep -nE "(INSERT INTO|UPDATE|DELETE FROM|REPLACE INTO) +(invoices|credit_notes|supplier_invoices|contact_persons|contacts)\b" -- 'crates/*/src/*' 'crates/*/src/**'`), tout verrou explicite. Pour chaque chemin de production : ce qu'il tient **avant** le contact, et s'il forme un cycle avec un chemin de la story (création, complétion, `PUT`, dévalidation d'une annulée). Un chemin absent du tableau, une ligne mal résolue ou un angle mort non justifié est un finding. Vérifie la sémantique invoquée (verrou partagé de clé étrangère à l'`INSERT`, à l'`UPDATE` seulement si la colonne change) contre la documentation d'InnoDB que tu connais, et dis-le si tu ne peux pas la vérifier.
3. **Décision de Guy du 2026-11-19 (D3ter, AC-14)** : `POST /invoices` rejoue par le mécanisme en place (`kesh_db::retry::retry_on_deadlock`, patron `routes/supplier_invoices.rs:739-741`) ; lis `routes/invoices.rs:668-812`, `repositories/invoices.rs:686-745` et `:1113-1310` : un rejeu y est-il sûr (une seule transaction, rien hors d'elle, entrée clonable) ? L'**extension à `PUT /invoices/{id}`** est-elle justifiée par le relevé ? Le test « route victime » est-il réalisable au patron de `kesh-api/tests/rejeu_interblocage_e2e.rs` (lis son en-tête et un test), et sa mutation tuée ? Le registre `audit_route_registry.rs` (`:550-551`) doit-il changer ? Le paragraphe `docs/api-external.md:1326` est-il désigné ? La contrainte de production (factures client déjà émises, PDF) est-elle citée ?
4. **Ordre des verrous de la dévalidation** : le fournisseur après l'étape (3) (`repositories/supplier_invoices.rs:1308-1311`) et avant (4) (`:1314`) — est-ce bien avant toute lecture ordinaire ? le cycle avec la création d'un avoir (`credit_notes.rs:557`, `:742`) disparaît-il ? le test « pas d'interblocage avoir/dévalidation » est-il réalisable et sa mutation tuée ? un cycle neuf naît-il de ce déplacement ?
5. **Relevé B (« Manuels et documentation ») — ensemble clos des phrases qui parlent de créer, compléter, repasser en brouillon ou supprimer une facture fournisseur** : refais-le depuis le symptôme sur `docs/manual/fr/user-manual.tex` (chapitre `:2059-2801` et hors chapitre), le **PDF aplati** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, en pipe), `docs/manual/fr/admin-manual.tex` (« Pour les intégrations ») et `docs/api-external.md` (tableau des codes d'erreur compris). Toute phrase absente du tableau, ou marquée « non » alors que la règle la rend fausse, est un finding ; vérifie aussi chaque numéro de ligne cité et le recompte (trente-deux sites, dix-huit à changer).
6. **Décision de Guy du 2026-11-19 (dévalidation confirmée) et V3-3** : le retrait ne promet plus la suppression ; les deux cas (clé d'API ; brouillon passé par un lot : validation puis annulation) sont-ils exacts au code (`lib.rs:355-361`, `SUPPLIER_INVOICE_HAS_PAYMENT_BATCH_HISTORY`) et au manuel (`:2221-2228`, `:2316-2322`) ?
7. **Un seul texte par refus, pour les deux fiches (V3-4)** : les trois clés `error-supplier-invoice-*` sont-elles bien celles que la 48-12b affiche (E4, E5, F4), mot pour mot ? `lint-i18n-ownership.js` admet-il l'espace `error-*` dans `features/` ? le texte de la règle 1 dit-il « si c'est la même facture » avant « 15-b » ?
8. **Cohérence entre les deux fiches (contrat d'API commun)** : codes, `details : { supplierInvoiceId, internalNumber, status, confirmable }`, valeur de `confirmable` par geste et par code, `confirmNotDuplicate` sur les quatre corps, textes, règle de sortie commune ; la sortie de E6 décidée par Guy (48-12b) ne suppose-t-elle rien que le serveur ne fournit pas ?
9. **Règles de la story et contraintes** : critère 1 par année civile, critère 2 avec confirmation auditée, règle 3, annulées exclues, numéro facultatif, rejeu à la modification sur quatre éléments, aucune migration (`ls crates/kesh-db/migrations/*.sql | wc -l` inchangé), sortie commune ; **compte de modules** : cinq (le rejeu de `routes/invoices.rs` est dans `kesh-api`) — vérifie-le au grain du fichier d'épique (`git show origin/chore/kickoff-0.20.0:_bmad-output/planning-artifacts/epic-48-corrections-0-20-0.md`).
10. **Tests et preuve du rouge** : chaque ligne nomme un binaire qui peut appeler ce qu'elle teste ; chaque mutation est tuée (raisonne sous REPEATABLE READ) ; aucune assertion vraie par construction.
11. **Propagation** : grep des symptômes corrigés en P3 (« se supprime ensuite », « quatre sites », « quatre modules », « dans le module déplacé », « ne rend que `bank_imports », « confirmez seulement ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log, les décisions écartées et les mutations peuvent les garder.
12. **Recyclage et effet sur l'utilisateur réel** : un défaut de la P3 revient-il sous une autre forme, né de cette remédiation ? un chemin laisse-t-il passer un doublon payable deux fois, ou un geste légitime de Guy (livres réels, lots pain.001, factures client émises) est-il fermé ou rendu en `500` ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, pas de `sqlx`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p4.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V4-n`, sévérité, axe, lieu dans la fiche, **code lu** (ligne citée), preuve, remède.
- Vérifications positives (ce qui a été contrôlé et tient).
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
- Pour les relevés que tu as refaits (A et B pour la 48-12a ; C et D pour la 48-12b) : ton propre ensemble, ligne par ligne, comparé à celui de la fiche.
