# Prompt — validation P5 de la fiche 48-12a « Doublons de factures fournisseur — serveur »

Passe **complète** (non ciblée), **modèle Opus** (la P4 était Sonnet), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Migration breaking policy*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p1.md`, `validation-12a-p2.md`, `validation-12a-p3.md`, `validation-12a-p4.md` (et ceux de la 48-12b à côté).
- La remédiation de la P4 : le dernier commit de la branche (`git show HEAD`, `git log -3`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P4, une par une** (V4-1 à V4-6 de `validation-12a-p4.md`), et l'extension du rejeu à `PUT /invoices/{id}` (condition : la modification change le projet **et** le contact, `invoices.rs:1145-1147`, `:1307-1308`) : prémisse relue au code (`grep -nF`, lecture directe), puis fermée / fermée en partie / déplacée.
2. **Relevé B, par deux voies.**
   - *(a) Rejoue la commande écrite dans la fiche* (section « Manuels et documentation », bloc `sh` : `awk '…' docs/manual/fr/*.tex docs/*.md`, qui n'écrit que sur la sortie standard) et compare sa sortie, rang par rang, au tableau de la fiche : 240 paragraphes, 27 « oui », 213 « non » en dix-sept groupes. Une différence de compte, un rang absent ou doublé, un « non » dont la raison est fausse, un « oui » dont le changement décrit est faux sont des findings. Lis réellement les paragraphes marqués « non » qui touchent une facture fournisseur (groupes N4 à N16), pas seulement leur classement.
   - *(b) Cherche par une autre voie ce que la commande manquerait*, sans partir de son motif : par exemple, aplatis les trois PDF (`pdftotext docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '`, en pipe) et cherche les phrases qui parlent des gestes de la story avec d'autres mots (« saisie », « double », « deux factures », « même numéro », « nouvel essai », « réessayer », « clé », « intégration », « annulée ») ; lis la table des matières des manuels et les intitulés de section de `docs/api-external.md` pour repérer une section sur les factures fournisseur, les clés d'API ou les documents importés que la commande n'aurait pas retenue ; regarde hors de `docs/` (`README.md`, `website/`) ce qui promet un comportement que la règle change. Tout site trouvé par (b) et absent de (a) est un finding, avec le motif qui l'aurait attrapé.
3. **Relevé A (D3bis)** : rejoue le `grep` du schéma et des écritures ; la ligne 11 par son critère, la ligne 6 avec sa condition, la ligne 7 (cycle par un autre exercice : la fiche dit « non établi, dépendant du plan d'exécution, couvert par le rejeu des deux routes ») — l'affirmation est-elle exactement ce que le code établit, ni plus ni moins ?
4. **D3ter et ses sites** : rejeu de `POST /invoices` et de `PUT /invoices/{id}` (sûreté, test « route victime », mutation) ; prose et compte de `crates/kesh-api/tests/audit_route_registry.rs` (`:163-190`, `:354`) ; Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md:342` ; `docs/api-external.md:1326`.
5. **Règle d'arrêt des modules (V4-5)** : les corrections de fixtures E2E ne comptent pas comme un module — la fiche le dit-elle sans contradiction ailleurs (S7, Dev Notes, Gates) ? Compte de modules : cinq, au grain du fichier d'épique (`git show origin/chore/kickoff-0.20.0:_bmad-output/planning-artifacts/epic-48-corrections-0-20-0.md`).
6. **Cohérence entre les deux fiches (contrat commun)** : codes, `details` et `confirmable`, `confirmNotDuplicate` sur les quatre corps, trois textes `error-supplier-invoice-*` mot pour mot, lignes du relevé B attribuées à la 48-12b (91, 106) contre ce que la 48-12b dit écrire.
7. **Décisions de Guy** (année civile, critère 2 confirmé et audité, dévalidation confirmée, rejeu de `POST /invoices`, sortie commune) et contraintes de la release (aucune migration : `ls crates/kesh-db/migrations/*.sql | wc -l` inchangé ; factures client émises et PDF intacts).
8. **Tests et preuve du rouge** : chaque ligne nomme un binaire qui peut appeler ce qu'elle teste ; chaque mutation est tuée ; aucune assertion vraie par construction.
9. **Propagation** : grep des symptômes corrigés en P4 (« trente-deux sites », « aucun** interblocage », « supprimé** par l'ordre », « hors les deux renvois en commentaire de S1** dans ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log peuvent les garder.
10. **Recyclage et effet sur l'utilisateur réel** : un défaut de la P4 revient-il sous une autre forme ? un chemin laisse-t-il passer un doublon payable deux fois, ou un geste légitime de Guy est-il fermé ou rendu en `500` ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, pas de `sqlx`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `awk` (sur la sortie standard), `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12a-p5.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V5-n`, sévérité, axe, lieu dans la fiche, **code lu** (ligne citée), preuve, remède.
- Vérifications positives (ce qui a été contrôlé et tient).
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
- Pour les relevés que tu as refaits (A et B, voies (a) et (b), pour la 48-12a ; C et D pour la 48-12b) : ton propre ensemble, ligne par ligne, comparé à celui de la fiche.
