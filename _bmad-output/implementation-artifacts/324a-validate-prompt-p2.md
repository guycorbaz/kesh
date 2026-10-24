# Prompt — validation P2, Story 32-4a (la couche de services : rollout des lectures — écritures, postes ouverts, contacts)

*Versionné le 2026-10-24. Passe **complète** (deuxième passe), sur la **fiche entière**. **Deux lentilles, R et F**,
chacune en contexte frais, **Opus** (la P1 était Sonnet ; rotation Sonnet ↔ Opus des passes complètes). Lecture seule
stricte. Motif de la passe complète plutôt que ciblée : les trois MEDIUM de la P1 touchent des **règles de garde** —
la table des traces, la garde lexicale, l'inventaire des écritures —, non une ligne isolée (`CLAUDE.md`, *La passe
ciblée* : « si le patch précédent […] change une règle, reprendre le protocole complet »).*

Dépôt `/home/gcorbaz/devel/kesh-32-4a`, branche `story/32-4a` (partie de `main` `89d68704`, la 32-3 fusionnée ; la
remédiation P1 est le commit `e9c343d9`). Fiche :
`_bmad-output/implementation-artifacts/32-4a-services-ecritures-contacts.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-324a.md` (C-32-4a-1 à 12). Rapports de la P1, à lire
pour savoir ce qui a été corrigé et comment : `/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p1-R.md` et
`324a-validate-p1-F.md` ; leur sort est au Change Log de la fiche (« validation P1 »). Diff de la remédiation :
`git show e9c343d9`. **Le motif à appliquer** : fiche `32-3-services-motif.md` (§ *Le motif*, AC7, AC8, AC10, Dev Notes
R1-R8), registre `epic-32-choix-autonomes-323.md` (C-32-3-34 à 41 surtout), et le code réel
`crates/kesh-api/src/services/` et `crates/kesh-api/tests/services_motif.rs`. Contexte de l'epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-4*, son annotation datée du 2026-10-24, § *32-9*) ;
registre `epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7). Issue des LOW de la 32-3 : `gh issue view 738 --json
body,comments`. Règles : `CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une
passe sans finding au-dessus de LOW —, *Règle de splitting préventif*, *Pendant une boucle de revue* et son exception
`kesh-db`, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON RÉSOLUS*, *Recompter ses propres
comptes rendus*, *Propagation post-patch*), `docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas ; aucune
migration, `kesh-db` non touché ; au plus cinq modules ; seules les lectures que serviront les outils (C-32-0-4) ;
aucune donnée réelle.

⚠️ **La sévérité se déplace vers ce qu'on vient d'écrire** (`CLAUDE.md`, *La passe ciblée*). La remédiation P1 a
ajouté des règles neuves — une dix-neuvième ligne à `TABLE_DES_TRACES`, un détecteur aplati, un compte des `async fn`
toutes visibilités, une mesure du `FOR UPDATE` en lecture seule (AC11 (c)), une fermeture des appels, deux mutations
(M16′, M17′), un second contrôle d'écrêtage (AC7). Chacune peut être fausse, inatteignable, ou contredire une autre
partie de la fiche. **Elles sont à lire comme du neuf**, contre le code, avec la même défiance que la rédaction
d'origine — mais la passe porte sur la fiche **entière** : un défaut d'origine que la P1 n'a pas vu reste un défaut.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer :

1. **Les faits 1 à 9** (§ *Ce que le code dit*) : 17 handlers, 7 lectures, 10 mutants (recompte-les) ; les 17 corps
   portent-ils tous `state.pool` ? RBAC des sept lectures dans `lib.rs` ; les trois régimes de pagination — défauts,
   plafonds, et qui écrête ; l'ordre et le texte des neuf refus de la liste des écritures ; la forme du détail d'une
   écriture et la garde lexicale qui la fige (lis-la entière) ; l'ordre des lectures des contacts et des postes
   ouverts ; le motif tel qu'il est dans `services/` ; les lignes citées de `services_motif.rs` ;
   `MULTI-TENANT-SCOPING-PATTERNS.md` et `api-external.md`.
2. **La fermeture des appels (fait 6, AC13)** — **neuve en P1** : suis toi-même chaque appel des cinq handlers jusqu'à
   ses requêtes, `const` SQL comprises (`kesh-db/src/repositories/`). La liste des appels directs et transitifs est-elle
   **close** — une fonction appelée qu'elle omet ? Une écriture, un `FOR UPDATE`, un `get_or_create` sur la fermeture,
   hors la branche `Lecture::Verrouillante` de `lettering_guard` (`journal_entries.rs:1182`) ? Cette branche est-elle
   vraiment non prise (l. 1261) ? Les numéros de ligne cités sont-ils exacts ? La commande de l'AC13 (fichiers,
   commentaires écartés, tri par fonction) rend-elle ce que la fiche annonce, et le « tri » est-il décidable ?
3. **Le `FOR UPDATE` en lecture seule (fait 6, AC11 (a) et (c))** : la fiche dit le refus **mesuré** pour `START
   TRANSACTION … READ ONLY` (`kesh-db/tests/closing_checks.rs:253-274`, 29-1) et **non établi** pour `SET SESSION
   TRANSACTION READ ONLY`. Est-ce exact (lis le test, le Dev Agent Record de la 29-1, l'aide `pool_lecture_seule` de
   `services_motif.rs:833-855`) ? La mesure de l'AC11 (c) est-elle réalisable telle qu'écrite (sur `pool_lecture_seule`,
   en autocommit **et** dans `begin()`, depuis un test d'intégration) ? Ses deux issues sont-elles toutes deux
   traitées par la fiche, sans qu'aucune ne laisse une phrase fausse ailleurs (AC1, AC13, doc-comment prévu) ?
4. **Inventaire clos des handlers de lecture** : une lecture des trois fichiers que la fiche ne nomme pas ? Hors de ces
   fichiers, une lecture des mêmes données que l'outil MCP servirait ? Le choix C-32-4a-1, son annotation à l'epic et
   la question Q1 (§ *Points à soumettre à Guy*) disent-ils la même chose, et la Q1 est-elle posée de façon que Guy
   puisse répondre ?
5. **Comportement inchangé** : pour chacun des cinq handlers, l'ordre des erreurs après déplacement ; les défauts serde ;
   le plafond commun des propositions ; la conversion `CoreJournal → DbJournal` ; le libellé des contacts. Les deux
   contrôles de l'AC7 (noms des constantes ; écrêtages `grep -nE 'clamp\(|\.max\(0\)'`) : lance-les sur le code actuel —
   rendent-ils ce que la fiche dit (huit lignes aujourd'hui), et l'état attendu (deux lignes de `get_lettering_proposals`)
   est-il celui qu'un correctif conforme produirait ? Refais le `grep` de l'AC8 (seize fichiers, comptes `sqlx::test`).
6. **Le contrôle d'accès et les erreurs attendues** : exigences du tableau du *Rollout* contre le RBAC ;
   `Portee::ContactsRead` et ses conséquences sur `services_motif.rs` ; les cas (a), (b), (c-val), (c-lec), (e) de
   l'AC9, et l'écriture exacte des erreurs (`AppError::Database(DbError::NotFound)`,
   `DbError::LetteringAccountNotLetterable`, `matches!`) — reste-t-il un `NotFound` nu qui ne compilerait pas, à l'AC9,
   l'AC10 ou l'AC11 ? La seconde société de l'AC11 (b) se pose-t-elle comme la fiche le dit (`companies::create`,
   `test_fixtures.rs:96-114`, `uq_users_username`) ?
7. **La garde lexicale (AC12)** — **refondue en P1** : le détecteur aplati (`\bstate\s*\.\s*pool\b` après
   `sans_commentaires`, `services_motif.rs:1998`) voit-il la forme coupée de `letterings.rs:243-244`, `:544-545`,
   `:566-567`, et seulement le champ (`lecture.pool()`, `// state.pool`, `self.state.pool`, une chaîne littérale) ? Les
   témoins prescrits sont-ils constructibles ? Le compte des `async fn` toutes visibilités
   (`^(pub(\([^)]*\))? )?async fn`, avant `#[cfg(test)]`) vaut-il bien 7 / 5 / 5 des deux côtés aujourd'hui, et rougit-il
   pour un `pub(crate) async fn` neuf ? Les définitions de « corps » de l'AC12 (jusqu'au `pub async fn` suivant ou au
   `#[cfg(test)]`) et de `corps_des_handlers` (l. 2011-2033 : jusqu'à la première ligne `}`) concordent-elles, ou un
   corps lu par l'un échappe-t-il à l'autre ? (viii) et son motif ; la v-2 à deux fabriques ; `services/` à huit.
8. **La garde de transaction déplacée (AC4, C-32-4a-7)** — **réécrite en P1** : les trois jetons, la règle du corps
   « débarrassé de ses blancs » pour `pool().begin()` (compte-t-elle juste, ou attrape-t-elle autre chose — un
   commentaire, un `lecture.pool().begin()` d'une autre fonction du même corps ?), les deux ancres ; le texte de la garde
   actuelle (`journal_entries.rs:1005-1080`) lu en entier contre ce que la fiche dit en garder.
9. **La table des traces (AC8 (1), AC14 M17/M17′, AC17, C-32-4a-10)** — **révisée en P1** : la dix-neuvième ligne
   (`journaux_affichage_sans_filtre`, `Svc::Journaux`, `Usg::Affichage`, `dem: BORNES`) est-elle jouable par les **deux**
   boucles qui parcourent `TABLE_DES_TRACES` (`services_motif.rs:780` par la route, `:1615` au service) — la route a-t-elle
   un `requete(ligne, …)` qui construit l'URL sans filtre, le service un `demande_journaux` qui rend `journal: None` ?
   Un autre site suppose-t-il dix-huit lignes, ou une ligne par couple (service, usage) ? M17 est-elle tuée par la ligne
   19, M17′ par la ligne 7 et **par elle seule** (relis `rapports.rs:370-412` et `:595-606`) ?
10. **Mutations (AC14)** : chacune des **35** est-elle réalisable telle qu'écrite et tuée par l'assertion nommée ? M8
    vraiment équivalente ; M14 (l'erreur 1792 remonte-t-elle) ; M16 en forme coupée et M16′ ; M18 reformulée (`let _t:
    Option<&crate::services::poignees::Trace> = None;` compile-t-il dans `carnet::detail` — chemin, visibilité — et (viii)
    le voit-il) ?
11. **Manuel et documentation** : la fiche dit « rien de visible ne change ». Vérifie qu'aucun texte ne décrit autrement
    ce qui bouge : `docs/api-external.md` (l. 213, 216-218, 397-460), `docs/manual/fr/user-manual.tex` et
    `admin-manual.tex` (carnet d'adresses, écritures, postes ouverts, pagination), et **les PDF aplatis** :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(postes ouverts|par page|contacts).{80}"`.
    La phrase prévue pour `MULTI-TENANT-SCOPING-PATTERNS.md` et le doc-comment prévu de `services/mod.rs` seront-ils
    vrais, quelle que soit l'issue de la mesure de l'AC11 (c) ?

## Lentille F — faisabilité et fausses prémisses

1. **Prémisses** : chaque « établi au code » tient-il, y compris ceux que la P1 a ajoutés (fermeture des appels,
   « aucun faux vert aujourd'hui » de la garde coupée, « 68 binaires » / « 29 », « 23 binaires » qui montent `common`) ?
   Une prémisse de l'epic reprise sans vérification ?
2. **Périmètre et découpage** : quatre modules — juste (`lib.rs`, `helpers.rs` vraiment non touchés) ; aucune migration
   ni dépôt `kesh-db` ; le déplacement de la garde de transaction ; l'extraction de `pool_lecture_seule` et
   `lecture_seule` dans `tests/common/lecture_seule.rs` — `lecture_seule` appelle `Lecture::new` : compile-t-il depuis
   `common/` (visibilité de `Lecture::new`, `dead_code` dans les 23 autres binaires), et la garde (v-2), qui compte les
   fabriques **dans `src/`**, reste-t-elle à deux ?
3. **Faisabilité en Rust** : les signatures de l'esquisse ; la mesure de l'AC11 (c) depuis un test d'intégration ; le
   détecteur aplati avec les aides existantes (`sans_commentaires`, `corps_des_handlers`) ; le compte sans blancs de la
   garde déplacée ; M18 ; la dix-neuvième ligne dans les `match` des boucles de la table.
4. **Décomptes** : recompte à la source chaque nombre — 17 handlers, 7 lectures, 10 mutants, 12 sites non résolus, six
   corps de (ii), 16 fichiers et leurs comptes, neuf refus, huit fichiers de `services/`, 19 lignes de la table pour 18
   couples, 17 AC, **35** mutations (5+2+4+2+1+4+3+2+1+1+1+1+1+1+1+1+1+1 + M16′ + M17′), huit lignes de l'AC7, quatre
   modules, C-32-4a-1 à 12 ; et le Change Log P1 : 18 findings, 14 distincts, 3 MEDIUM, 11 LOW — cohérents avec les
   rapports P1 ?
5. **Cohérence interne** : AC1 à AC17, T0 à T7, registre, *Hors périmètre*, *Risques*, *Points à soumettre à Guy*,
   *Fichiers touchés*, Change Log, annotation de l'epic, ligne de `sprint-status.yaml` — contradiction, critère
   invérifiable, commande mal formée (essaie mentalement **chaque** commande des AC7, AC8, AC10, AC12, AC13, AC16 contre
   le code actuel **et** contre un correctif plausible). La propagation de la P1 est-elle complète : un `33`, un « deux
   jetons », un `journal: None` pour la ligne 7, un `NotFound` nu, un « rougit par son nom » sans réserve, un « à
   confirmer à l'exécution » resté dans une entrée **vivante** ? (Les entrées datées du Change Log ne se réécrivent pas :
   ce n'est pas un défaut qu'elles gardent leur valeur d'origine.)
6. **#738** : la décision sur chacun de ses treize points (C-32-4a-10, AC17, *Hors périmètre*), B4 « repris en partie »
   compris — la frontière entre ce qui est repris et ce qui reste est-elle dite au même endroit de la même façon ?
7. **Ce que la fiche omet** : un appelant des constantes ou de `parse_as_of` hors de leur fichier ; une garde existante
   qui lit ces sources ou compte des fichiers ou des lignes de `TABLE_DES_TRACES` ; le frontend et l'E2E ; la recette
   visuelle ; le CHANGELOG ; l'ordre avec la 32-4b (*Risques*, C-32-4a-9) — tenable ? ; le défaut de plage de
   `dateFrom`/`dateTo` laissé hors périmètre (R10 de la P1) — le motif tient-il, et le texte d'issue prévu au Change Log
   est-il exact ?

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée. Signale explicitement si un finding naît de la **remédiation P1** (commit
`e9c343d9`) ou de la rédaction d'origine. Un bilan d'en-tête **cohérent avec le corps**. ⛔ **La liste des axes exercés
ET non exercés** (R1 à R11, F1 à F7) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au
développement n'existe pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier
par lentille : `/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p2-R.md` et
`/home/gcorbaz/devel/kesh-gate-logs/324a-validate-p2-F.md`. Dernier message : chemin, bilan par sévérité, une ligne par
finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `npm`, `npx`,
`docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Le dépôt principal `/home/gcorbaz/devel/kesh` est hors
de ton périmètre. Autorisés : lecture, `grep`, `sed -n`, `git log|show|diff|blame|merge-base`, `gh issue view|list`,
`pdftotext … -`, et les filtres en lecture seule `tr`, `sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie
standard.
