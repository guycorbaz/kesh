# Prompt — validation P2, Story 32-4b (la couche de services : rollout des lectures — factures)

*Versionné le 2026-10-25. Passe **complète** (deuxième passe, après la remédiation de la P1). **Deux lentilles, R et F**,
chacune en contexte frais, **Opus**. Lecture seule stricte.*

**Ce qui a changé depuis la P1** : la remédiation est au Change Log de la fiche (entrée « validation P1 ») et dans
`git diff ced11a88 HEAD`. Les rapports de la P1 sont `/home/gcorbaz/devel/kesh-gate-logs/324b-validate-p1-{R,F}.md` —
lis-les **après** ta propre lecture, pour vérifier que chaque finding est réellement traité et que la remédiation n'en
introduit pas : la sévérité se déplace vers ce qu'on vient d'écrire. À éprouver en priorité : l'AC17 (3) étendue à six
sites et son `grep` sur `crates/` entier ; `lib.rs` et `tests/common/lecture_seule.rs` entrés au périmètre, et le
recompte des modules (4) ; l'AC18 (`seed_base`) et ses contrôles ; M21d déclarée équivalente ; les deux montages de
fixtures (C-32-4b-15) ; l'annotation de l'epic et la Q2 ; les comptes 11 / 8 / 8 de l'AC13.

Dépôt `/home/gcorbaz/devel/kesh-32-4b`, branche `story/32-4b` (partie de `main` `592ee430`, la 32-4a fusionnée). Fiche :
`_bmad-output/implementation-artifacts/32-4b-services-factures.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324b.md` (C-32-4b-1 à 17). **Le modèle direct** : la fiche
`32-4a-services-ecritures-contacts.md` (AC, Change Log, Dev Agent Record, trois passes de revue de code) et son registre
`epic-32-choix-autonomes-324a.md` ; le code `crates/kesh-api/src/services/` et les tests
`crates/kesh-api/tests/services_rollout.rs` et `services_motif.rs`. Le motif : `services/mod.rs` (R1-R8). Contexte :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-4*, l. 304-321 ; § *32-9*) ; `epic-32-spike-mcp.md`
(l. 925-926, 961, 1061-1067 — la réponse de Guy à Q11) ; registre `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7).
Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans
finding au-dessus de LOW —, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`,
*Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes
rendus*), `docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas (statuts, corps,
ordre des refus, messages au caractère près) ; aucune migration, `kesh-db` non touché ; au plus cinq modules (la fiche en compte quatre) ; seules les
lectures que serviront les outils (C-32-0-4) ; **Q11** — lecture seule tenue par l'application (la poignée `Lecture`)
plus le mode de session `READ ONLY`, **aucune hypothèse d'utilisateur MariaDB** ; aucune donnée réelle.

⛔ **Aucune commande `gh`**, pas même en lecture : les issues citées (#738, #742) ne se consultent pas dans cette passe.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 10** (§ *Ce que le code dit*) : 20 handlers, 7 lectures, 12 mutants (recompte-les), 15 et 8 `async
   fn` toutes visibilités ; RBAC des sept lectures dans `lib.rs` ; les trois régimes de pagination — défauts, plafonds,
   **qui refuse et qui écrête**, et la confirmation qu'**aucun** dépôt n'écrête (relis `invoices::list_by_company_paginated`
   et `supplier_invoices::list`) ; l'ordre et le texte **exact** des douze refus (cinq de la liste, sept de l'échéancier) ;
   les défauts du tri et les constantes de l'échéancier ; l'ordre des lectures de chaque handler (société, entité,
   réglage d'arrondi) ; la transaction des règlements et son `commit` ; `load_with_settlement_cancellation` et ses
   appelants ; les deux aides hors corps et le compte 40 / 38 / 2 et 15 / 15 du détecteur `\bstate\s*\.\s*pool\b`
   (recompte-le à la main, `awk` sur les corps puis `tr '\n' ' ' | grep -oE`) ; la balance âgée déjà servie ; les
   lignes citées de `services_motif.rs` et `services_rollout.rs`.
2. **Inventaire clos des lectures** : les deux tableaux (visées / laissées, hors fichiers) sont-ils complets ? Une route
   `GET` qui lit des factures, des échéances, des règlements ou des factures fournisseur, que la fiche ne nomme pas
   (`lib.rs` entier, `routes/*.rs`) ? Une lecture servie par la 32-9a (« factures clientes (liste, détail, échéances,
   impayées), balance âgée, factures fournisseur (liste, détail) ») qui manquerait au *Rollout* ? Le choix C-32-4b-1
   (export et scan laissés) tient-il ?
3. **Comportement inchangé** : pour chacun des six handlers, l'ordre des erreurs après déplacement (extracteur `Query`,
   `exiger`, société, validation, entité) ; la conversion des enums de route (`PausedParam`, `PaymentStatusParam`) ; la
   fonction pure partagée avec l'export et l'ordre de l'export ; la mise en forme `from_view` ; l'aperçu d'arrondi (un
   brouillon seulement) ; le compte `AppError::Validation` 34 → 22 de l'AC8. Un test existant pourrait-il rougir ? Refais
   le `grep` de l'AC9 : 25 fichiers ?
4. **Le contrôle d'accès** : exigences du tableau contre le RBAC ; `InvoicesRead` pour les factures fournisseur
   (C-32-0-7, C-32-4b-3) ; les cas (a), (b), (c-val), (c-lec), (e) de l'AC10 sont-ils constructibles, non vides, chacun
   distinct de `Forbidden` sans le contrôle, et les « sans objet » justes ?
5. **Lecture seule et inventaire des écritures (AC12, AC14)** : relis chaque dépôt appelé, **transitivement** —
   écriture, `FOR UPDATE`, `get_or_create`, transaction que `SET SESSION TRANSACTION READ ONLY` refuserait (le `commit`
   des règlements et de `get_settlement_view` y compris). Le fait 7 dit « aucune occurrence dans la fermeture » : est-ce
   vrai ? Les fixtures annoncées exercent-elles la transaction des règlements (un règlement existe-t-il) ?
6. **La garde lexicale (AC13)** : (ii) à douze ; (iii) avec son cinquième champ — la définition d'une « aide » (ligne
   qui **commence** par `fn ` ou `async fn `) attrape-t-elle les deux aides et rien de faux ? l'égalité 30 = 28 + 2 et
   11 = 11 + 0 au dernier commit est-elle juste ? (ix) — les formes permises suffisent-elles aux six handlers de la
   32-4a (relis leurs corps) et aux six neufs ? ses témoins rougiraient-ils pour `f(&state, x)`, `state.clone()`, une
   déstructuration ? une forme permise masquerait-elle un accès au pool (`state.lecture()` passé à une fonction hors
   `services/`) ? (vi) à dix, (viii) à cinq, (v-2) à deux.
7. **Mutations (AC15)** : chacune des 100 est-elle réalisable telle qu'écrite, et tuée par l'assertion nommée ? Une
   passerait-elle à vide (M9 : le `query_scalar` compile-t-il et rend-il la société de la facture ? M21b : quel test la
   tue ? M22a : `the_preview_follows_the_setting` passe-t-il par `GET /invoices/{id}` ? M26 : (ii) reste-t-il vraiment
   vert ? M28 : une aide appelée par un **mutant** rougit-elle (iii) ?) Le décompte 16 + 7 + 15 + 44 + 4 + 4 + 5 + 5 est-il
   cohérent avec la liste, et M21d vraiment équivalente (relis `due_dates_summary`) ?
8. **Manuel et documentation** : la fiche dit « rien de visible ne change ». Vérifie qu'aucun texte ne décrit autrement
   ce qui bouge : **`docs/api-external.md`** (l. 205-240, 545-570, 715-760 : factures, fiche fournisseur, règlements),
   **le manuel** `docs/manual/fr/user-manual.tex` et `admin-manual.tex` (factures, échéancier, règlements, factures
   fournisseur, pagination), et **les PDF aplatis** :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(échéancier|par page|factures fournisseur).{80}"`.
   Les deux doc-comments que l'AC17 (3) corrige (`services/mod.rs:132-139`, `services/poignees.rs:9-12`) disent-ils
   bien aujourd'hui le contraire de Q11 ? La phrase prévue pour `MULTI-TENANT-SCOPING-PATTERNS.md` sera-t-elle vraie ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il ? Une prémisse de l'epic reprise sans vérification (« balance âgée :
   appel seulement », « 3 avec `services` », « `clamp(1, 200)` ») ? Une leçon de la 32-4a mal transposée (les listes
   clientes refusent, elles n'écrêtent pas) ?
2. **Périmètre et découpage** : quatre modules (`services`, `routes/invoices`, `routes/supplier_invoices`, la racine
   `lib.rs` pour deux doc-comments) — juste (`helpers.rs`, `routes/reports.rs`, `routes/imported_supplier_invoices.rs`
   vraiment non touchés ? `services/acces.rs` et `poignees.rs` pour leur seul
   doc-comment comptent-ils comme un module de plus ?) ; aucune migration ni dépôt ; la garde (ix) neuve et l'extension
   de (iii) sont-elles sûres pour les trois fichiers de la 32-4a ?
3. **Faisabilité en Rust** : les signatures de l'esquisse compilent-elles sans contorsion (`InvoiceListResult`,
   `DueDatesSummary`, `SettlementCancelHit` et `SupplierInvoiceSettlementView` exportés par leurs dépôts ?
   `tokio::join!` sur `lecture.pool()` ; la transaction et `&mut tx` / `&mut *tx` dans un service qui ne reçoit qu'une
   `Lecture` ; `from_view` appelée depuis la méthode et depuis le handler) ? `services/factures.rs` et
   `kesh_db::repositories::invoices` — collision de noms ?
4. **Les tests (AC10 à AC12)** : les fixtures sont-elles constructibles sur `seed_accounting_company` (exercice
   2020-2030, réglages de facturation : compte débiteurs, compte d'arrondi ? une facture fournisseur exige-t-elle un
   compte créanciers posé à la main ?) ; une facture **en retard** et une **à échoir** sans dépendre de l'horloge ; cinq
   ordres de tri distincts deux à deux sur trois factures — possible ? ; le résumé « B15 » observable ? ; une ligne
   de l'AC11 (c) jouée **avant** T2 rendrait-elle ce que la fiche annonce sur le code d'aujourd'hui (essaie chacune
   contre le code) ? Le renommage du test (C-32-4b-13) casse-t-il une référence ?
5. **Décomptes** : recompte à la source chaque nombre — 20 handlers, 7 lectures, 12 mutants, 13 / 15 et 7 / 8, 40 / 38 /
   2 et 15, douze refus (5 + 7), 34 → 22, 25 fichiers, douze handlers migrés, dix fichiers de `services/`, onze services
   de la matrice, 18 AC, 100 mutations dont 99 à tuer, quatre modules, C-32-4b-1 à 17, six sites Q11 (neuf lignes).
6. **Cohérence interne** : AC1 à AC18, T0 à T7 (T3-bis compris), registre, inventaires, *Ce que les gardes ne voient pas*, *Fichiers
   touchés*, Change Log — contradiction, critère invérifiable, commande mal formée (essaie mentalement **chaque**
   commande des AC8, AC9, AC13, AC17 contre le code actuel **et** contre un correctif plausible : rend-elle ce que la
   fiche annonce ?).
7. **Ce que la fiche omet** : un appelant des constantes ou de `build_due_dates_query` hors de leur fichier (commentaires
   compris) ; une garde existante qui lit ces sources ou compte des fichiers ; le frontend et l'E2E (un écran lit-il un
   champ qui pourrait changer) ; une donnée réelle dans un exemple ; les *Points à tracer* sont-ils exacts (relis
   `invoice_echeancier_e2e.rs:112-117`, l. 402, 453, 500) ?

## Ce que tu rends

D'abord, pour chaque finding de la P1 : **traité / non traité / mal traité**, avec la preuve. Puis tes findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés ET
non exercés** (R1 à R8, F1 à F7, et la reprise des findings de la P1) — sans elle, « 0 finding » ne compte pas comme passe. Ce que la fiche **prescrit** au
développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier par
lentille : `/home/gcorbaz/devel/kesh-gate-logs/324b-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/324b-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; **aucune commande qui écrit, compile ou
exécute** : `scripts/*` (dont **`scripts/prepare-release.sh`** et `scripts/test-fast.sh`), `make`, `latexmk`, `cargo`,
`rustup`, `npm`, `npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, et **`gh` sous toutes ses formes**. `pdftotext` vers la sortie
standard seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh`
est hors de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `pdftotext … -`,
et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie standard.
