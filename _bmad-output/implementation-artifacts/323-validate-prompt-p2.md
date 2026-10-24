# Prompt — validation P2, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **complète** (deuxième passe ; la P1 a rendu six MEDIUM distincts, remédiés). **Deux
lentilles, R et F**, chacune en contexte frais, **Opus**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3` (partie de `275e9b97`). Fiche :
`_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md` (C-32-3-1 à 23) ; prompt de la P1 :
`323-validate-prompt-p1.md` ; rapports de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p1-R.md` et
`-F.md`. Le diff de la remédiation P1 : `git show HEAD -- _bmad-output/` (commit `docs(32-3): validation P1
remédiée — …`). Contexte de l'epic : `_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-3*, § *Ce que le
code dit déjà*, § *Points d'attention*) ; spike : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 4.1-4.4,
§ 5.3.1, § 6) ; registres `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `epic-32-choix-autonomes-322.md`.
Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans
finding au-dessus de LOW —, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception `kesh-db`,
*Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres comptes
rendus*, *Propagation post-patch*, *Priorités des défauts*), `docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas ; aucune
migration ; au plus cinq modules ; **la décision Q11 (lecture seule de production) est en attente de Guy et la story ne
doit pas en dépendre** — la fabrication d'une `Lecture` en lecture seule (forme et lieu) est à la 32-6 ; aucune donnée
réelle.

## ⚠️ Attention à la remédiation P1 — la sévérité se déplace vers ce qu'on vient d'écrire

La P1 a introduit des mécanismes neufs. Chacun est un candidat au défaut suivant ; examine-les **en premier**, contre le
code et contre le reste de la fiche :

1. **`Lecture::pool()` en `pub(in crate::services)`** (C-32-3-17), le pool de `Trace` à même visibilité, et la
   lecture du contexte PDF passée en service `rapports::identite_societe(&ContexteAcces, &Lecture)` (exigence
   `{ Consultation, None }`). Compile-t-il ? Tous les appelants du pool (générateurs de `kesh-report`,
   `letterable_account_ids` sur connexion acquise, `helpers::company_for_ids`) sont-ils bien **dans** `services/` ?
   L'ordre des erreurs d'un export reste-t-il celui d'aujourd'hui (le contexte PDF lu **après** la génération) ? Le
   `SELECT` et son `map_db_error` sont-ils identiques à `routes/reports.rs:1223-1245` ? Une exigence sur une aide de
   présentation est-elle cohérente avec R2/R3, et l'AC8 la teste-t-il correctement ?
2. **L'AC10 réécrit** — volets (iii) étendu à tout `src/` hors `services/` et `lib.rs`, (iv) « état de la 32-3 », (v)
   contre l'élargissement de la visibilité, (vi) la règle de l'AC1. Essaie **chaque** volet contre le code attendu et
   contre un correctif plausible : rend-il ce que la fiche annonce ? Le volet (v) attrape-t-il bien `pub(crate)` sur
   un accesseur **et** sur un champ de tuple, et ne rougit-il pas sur l'écriture prescrite par l'esquisse
   (`pub(in crate::services) MySqlPool`) ? Un texte existant de `src/` contient-il déjà `Lecture::new(` ou
   `Trace::new(` ?
3. **La garde de l'AC1** (C-32-3-20) : la commande `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services
   | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'` rend-elle **exactement** les deux lignes annoncées pour un `acces.rs`
   plausible (import, signature de `depuis_api`) ? Que rend-elle si le corps de `depuis_api` nomme un champ, si une
   ligne de code porte un commentaire de fin de ligne, ou si `rapports.rs` importe `crate::errors::AppError` ?
4. **M1 par échafaudage** (C-32-3-18) : le champ `lecture_m1`, le constructeur commun `Rapport::nouveau`, la
   substitution aux deux appels de `tracer` dans `afficher`/`exporter`, M1′ gardée par
   `matches!(details, DetailsTrace::GrandLivre { .. })`. Est-ce exprimable en Rust tel qu'écrit (visibilités,
   `Clone`, emprunt de `self` consommé) ? Tue-t-elle bien les huit assertions de l'AC9 (b), et M1′ la seule assertion
   `…_729` ? La déclaration « mutation de `AppState::trace()` équivalente » est-elle juste ? M11 et M12 : bien formées,
   honnêtes ?
5. **Les actions littérales** (C-32-3-19) : relis `crates/kesh-api/tests/audit_label_registry.rs` (balayage, `litteral()`,
   `SITES_INDIRECTS`, test `tout_site_dont_l_action_n_est_pas_litterale_est_inventorie`) : l'aiguillage `tracer` et
   les sept fonctions recopiées passent-ils sans entrée neuve ? `textes_coherents.rs` (G4-ter), `admin_pat_denied_e2e.rs`
   (`the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide`, l. 439), `audit_route_registry.rs` : la story
   peut-elle en faire rougir un ? Y a-t-il **une autre** garde de sources que la P1 n'a pas nommée ?
6. **Les huit `details_json` en littéral** (tableau de l'AC9 (b)) : confronte chaque clé et chaque valeur aux sept
   corps (`routes/reports.rs:599-615, 626-645, 1372-1400, 1407-1435, 1463-1495, 1514-1538, 1548-1573`) et aux 18 sites
   d'appel (valeurs `report_type`, `journal_filter`, `mode`, `format`, `period_*` de la période **résolue**,
   `fiscal_year_id` de la demande). L'`as_of` lu sur le `Rapport` (C-32-3-23) : `AgedReceivables.as_of` existe-t-il, et
   le contrat est-il tenable ?
7. **Q11** (C-32-3-22) : la fiche, le registre et la clé du sprint-status préjugent-ils encore de la forme **ou du
   lieu** de la `Lecture` en lecture seule (Story, Origine, AC3, AC10 (iv), AC15, *Hors périmètre*, esquisse) ?
8. **Les décomptes neufs** : fait 9 (trois tests, 17 handlers sur 18, sept chemins de trace sur huit), AC7 (neuf
   fichiers, sept qui frappent une route, neuf comptes `sqlx::test`), AC13 (neuf binaires + quatre gardes), Change Log
   de la P1 (R 0/0/5/8, F 0/0/3/10, « six MEDIUM distincts », recoupements). Recompte à la source.
9. **Propagation** : une notion corrigée en P1 survit-elle sous une autre forme ailleurs (« seize », « issue
   demandée », `pub(crate)`, « une seule règle », « `load_pdf_context(&Lecture` », « dix services » là où il en faut
   onze fonctions, « point commun ») ? Greper la **valeur**, non la formulation.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer — ceux de la P1 (`323-validate-prompt-p1.md`, lentille R, axes 1 à 8), **plus** les points 1 à 6
ci-dessus :

1. **Les faits 1 à 10** — revérifie ceux que la P1 a touchés (5, 9) et ceux qu'elle n'a pas touchés.
2. **Le motif et sa faisabilité en Rust** — avec les visibilités et l'aiguillage de la P1.
3. **Comportement inchangé** — dont l'ordre « génération → contexte PDF → rendu → trace » et l'`as_of` de la balance
   âgée (nom de fichier, contenu, trace).
4. **Le contrôle d'accès** — dont `identite_societe` et l'AC8.
5. **La trace et la séparation** — dont le tableau de l'AC9 (b).
6. **Inventaire clos (AC11)** — dont la phrase « établi par la lecture statique et confirmé sur les branches des
   fixtures » : est-elle juste, et les fixtures exigées (facture ouverte, écriture sur un compte de TVA) sont-elles
   constructibles avec `kesh_db::test_fixtures::seed_accounting_company` ?
7. **Mutations (AC12)** — M1 à M12, M1′, M2′, M1-témoin : réalisables telles qu'écrites, tuées par l'assertion
   **nommée** ?
8. **Manuel et documentation** — la fiche dit « aucun changement visible ». Vérifie qu'aucun texte ne décrit ce qui
   bouge : `docs/manual/fr/admin-manual.tex` et `user-manual.tex` (journal d'audit, `report.generated`, rapports,
   exports, rôle Consultation), `docs/api-external.md`, et **les PDF aplatis** :
   `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(report\.generated|journal d.audit|aucune mutation).{80}"`
   (et `user-manual.pdf`). La P1 a constaté que `admin-manual.tex:1464` dit du rôle Consultation « aucune mutation, à
   une exception près » : la fiche l'inscrit au *Hors périmètre*, issue P3 demandée. Le constat est-il juste et
   complet (le PDF dit-il la même chose ; y a-t-il d'autres écritures d'un Consultation que la phrase omet) ?

## Lentille F — faisabilité et fausses prémisses

Axes de la P1 (`323-validate-prompt-p1.md`, lentille F, axes 1 à 6), **plus** les points 1 à 9 ci-dessus :

1. **Prémisses** — dont celles que la P1 a ajoutées (C-32-3-17 à 23).
2. **Périmètre et découpage** — l'aide `identite_societe` et l'aiguillage alourdissent-ils `services/rapports.rs` au
   point de changer le compte de modules ou le signal de découpage ? Toujours quatre modules, sous le seuil de cinq ?
3. **Indépendance de Q11** — point 7 ci-dessus.
4. **Décomptes** — point 8 ci-dessus, et tous ceux de la P1.
5. **Cohérence interne** — AC1 à AC15, T0 à T6, registre (les amendements datés de C-32-3-2, 6, 8, 10 sont-ils
   cohérents avec C-32-3-17 à 23 ?), *Hors périmètre* (la phrase réécrite affirme-t-elle encore une fermeture non
   prouvée ?), esquisse, *Fichiers touchés*, Change Log, clé du sprint-status.
6. **Ce que la fiche omet** — appelants, gardes, recette visuelle, CHANGELOG, issues (#729, l'issue P3 du manuel).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour chaque finding, dis s'il **naît de la remédiation P1** ou de la rédaction
d'origine. Un bilan d'en-tête **cohérent avec le corps**, recompté. ⛔ **La liste des axes exercés ET non exercés** (R1
à R8, F1 à F6, et les points 1 à 9 de l'attention P1) — sans elle, « 0 finding » ne compte pas. Ce que la fiche
**prescrit** au développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un
seul fichier par lentille : `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie standard.
