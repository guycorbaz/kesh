# Prompt — validation P3, Story 32-3 (la couche de services : le motif, sur deux modules pilotes)

*Versionné le 2026-10-24. Passe **complète** (troisième passe ; la P2 a rendu cinq MEDIUM distincts, remédiés, dont
une **forme recyclée** d'un MEDIUM de la P1 — signal de recyclage déclaré au Project Lead). **Deux lentilles, R et F**,
chacune en contexte frais, **Sonnet**. Lecture seule stricte.*

Dépôt `/home/gcorbaz/devel/kesh-32-3`, branche `story/32-3` (partie de `275e9b97`). Fiche :
`_bmad-output/implementation-artifacts/32-3-services-motif.md` ; registre :
`_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md` (C-32-3-1 à 30) ; prompts des passes
précédentes : `323-validate-prompt-p1.md`, `323-validate-prompt-p2.md` ; rapports : `/home/gcorbaz/devel/kesh-gate-logs/
323-validate-p1-{R,F}.md` et `323-validate-p2-{R,F}.md`. Le diff de la remédiation P2 : `git show HEAD --
_bmad-output/` (commit `docs(32-3): validation P2 remédiée — …`). Contexte de l'epic :
`_bmad-output/planning-artifacts/epic-32-serveur-mcp.md` (§ *32-3*, § *Ce que le code dit déjà*, § *Points
d'attention*) ; spike : `_bmad-output/planning-artifacts/epic-32-spike-mcp.md` (§ 4.1-4.4, § 5.3.1, § 6) ; registres
`epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `epic-32-choix-autonomes-322.md`. Issues : #723 (epic), #729
(export du grand livre tracé en consultation), #730 (manuel, rôle Consultation et clés « sans écriture »). Règles :
`CLAUDE.md` (§ *Review Iteration Rule*, *La passe ciblée* — une boucle ne se clôt que sur une passe sans finding
au-dessus de LOW —, *Règle de splitting préventif* et son amendement du 2026-10-06 sur le **recyclage**, *Pendant une
boucle de revue* et son exception `kesh-db`, *Le prompt d'une passe doit NOMMER le manuel*, *Inventorier les sites NON
RÉSOLUS*, *Recompter ses propres comptes rendus*, *Propagation post-patch*, *Priorités des défauts*),
`docs/agents-orchestration.md` § 3, § 6 et § 6.1.

**Contraintes de la story, à tenir pour vraies et à contrôler** : le comportement de l'API ne change pas ; aucune
migration ; au plus cinq modules ; **la décision Q11 (lecture seule de production) est en attente de Guy et la story ne
doit pas en dépendre** ; aucune donnée réelle.

## ⚠️ Attention à la remédiation P2 — la sévérité se déplace vers ce qu'on vient d'écrire

La P2 a refondu deux mécanismes et en a ajouté plusieurs. Examine-les **en premier**, contre le code et contre le reste
de la fiche :

1. **L'inventaire clos de l'AC10 (v)** (C-32-3-24) — le cœur du signal de recyclage. Pour **chacun** des cinq relevés
   (v-a à v-e) :
   - la liste écrite est-elle **exacte** pour le code que la fiche prescrit (esquisse, AC3, AC4, AC6) ? Écris
     mentalement `poignees.rs`, `identite_societe` et `AppState::lecture()`/`trace()` tels que rustfmt les formerait,
     et compte les lignes que chaque motif rend. Une ligne **légitime** prescrite ailleurs par la fiche
     (`lecture.pool().acquire()`, l'appel `helpers::company_for_ids(lecture.pool(), …)`, les fonctions d'audit
     recevant `&Trace`, un `use` de `kesh_db`, un `map_db_error`) tombe-t-elle sous `MySql`, `Pool` ou `sqlx` sans
     être listée ? Un doc-comment est-il compté ?
   - le relevé est-il **fermé** ? Cherche activement un chemin vers le pool d'une poignée, ou une conversion
     `Lecture → Trace`, qui **compile** et ne fasse rougir **aucun** des cinq relevés ni l'AC10 (vi)-(vii) : trait
     implémenté par un bloc générique (`impl<T: …> X for T`), `impl` dans une macro, alias de type, ré-export
     (`pub use`), `Clone` puis déstructuration, fermeture qui reçoit le pool, `unsafe`/`transmute`, accès par
     `Deref` d'un **autre** type qui contiendrait une poignée… Pour chaque forme trouvée : est-elle déclarée au
     paragraphe *Ce que (v) ne voit pas* ? Si non, c'est un finding ; si la fiche la déclare « à la revue » alors
     qu'un relevé lexical simple la fermerait, dis-le.
   - les mutations M11, M11′, M11″ rougissent-elles bien le volet annoncé, et seulement pour la raison annoncée ?
   - la définition « ligne de code » (AC1) appliquée à (v-b), dont l'en-tête court « du mot `impl` au premier `{`,
     lignes jointes » : est-ce décidable sans ambiguïté (un `impl` dans une chaîne, un `where` sur plusieurs lignes,
     un `{` dans un générique `{ N }`) ?
2. **Les tests tabulaires** (C-32-3-25) — l'AC9 (b) (dix-huit lignes au service) et l'AC9 (b′) (la même table par la
   route) : confronte **chaque ligne** de la table aux sept corps d'audit (`routes/reports.rs:599-652, 1372-1405,
   1407-1455, 1463-1511, 1514-1545, 1548-1580`) et aux 18 sites d'appel (`report_type`, `format`, `journal_filter`,
   `account_ids`, `mode`, présence ou absence de `fiscal_year_id`) ; le format choisi par ligne est-il accepté par
   la route ; le filtre `Achats` est-il la valeur que `Journal::as_str` écrit ; le grand livre écrit-il les bornes de
   la **demande** ; une demande de projet en `cumulative` sans `fiscalYearId` est-elle valide ? Pour (b′) : le montage
   HTTP (`build_router`, `AppState::new_for_tests`, JWT) est-il constructible dans un fichier neuf sans toucher un
   test existant ; la règle d'`asof` « date UTC lue avant ou après » est-elle juste ; le rôle du jeton permet-il la
   route Comptable ; les bornes explicites suffisent-elles à rendre `debut`/`fin` déterministes ?
3. **L'AC10 (vii)** : la délimitation « de la ligne d'ouverture à la première ligne égale à `}` » rend-elle le corps
   exact de chaque handler (un handler qui contiendrait un bloc `}` en colonne 0, une macro) ? « Neuf de chaque » :
   recompte `pub async fn get_` et `pub async fn export_` dans `routes/reports.rs` (`grep -c`).
4. **L'AC8 (c-val)/(c-lec)** (C-32-3-26) : chaque cas (c-val) échoue-t-il **réellement** à la validation dans le code
   d'aujourd'hui (et non à une lecture) ; chaque (c-lec) passe-t-il la validation ; M3′ est-elle exprimable telle
   qu'écrite, et tue-t-elle (c-val) de `bilan` en laissant (c-lec) vert ? Le grand livre « sans cas (c-lec) » est-il
   juste ?
5. **Les mutations M1 à M13** et leurs primes, M1-témoin : chacune rougit-elle l'assertion **nommée**, et seulement
   celle-là quand la fiche dit « seule » (M1′, M13) ? M1 rougit maintenant aussi l'AC10 (v-d) : la fiche le dit-elle
   partout où M1 est décrite ? (b′) vert sous M1, (b) vert sous M13 : exact ?
6. **Les règles et arrêts de la P2** : R3 complétée (aide partagée), `identite_societe` et M2″ ; `ContexteAcces` à
   champs privés (les tests de l'AC8 se construisent-ils encore, par `depuis_api` et `mcp` seuls ?) ; `FormatExport`
   (dérives, visibilité, le `mod tests` intouché) ; mesures du `span` dans `rendre` ; fixture de TVA (`seed_accounting_
   company`, `crates/kesh-db/src/test_fixtures.rs` ; le rapport de TVA lit-il bien les lignes de factures validées et
   le compte d'impôt préalable) ; l'interdiction des `/* */` retirée — l'argument « un `/* */` ne peut produire qu'un
   faux rouge » est-il vrai pour **tous** les volets, y compris ceux qui exigent une **présence** (v-a, v-d, vii) ?
7. **Les huit gardes de sources** (AC7, AC13) : la liste et les neuf exclusions sont-elles justes
   (`grep -rlnE "include_str!|read_to_string|read_dir" crates/*/tests`, puis lecture) ? `test_schema_guard` : qu'exige-t-il
   d'un fichier de test neuf ? Une garde qui balaie `src/` par un autre moyen (`env!("CARGO_MANIFEST_DIR")`, `walkdir`,
   `glob`) a-t-elle échappé au relevé ?
8. **Le manuel et #730** : le *Hors périmètre* cite-t-il juste les deux sites (`admin-manual.tex:1464`, `:2897`) et
   les trois écritures (rapports, `exports.global`, `touch_last_used`) ? Vérifie le **PDF** aplati.
9. **Décomptes et propagation** : « cinq MEDIUM distincts », « onze LOW distincts », recoupements de la P2, « huit »
   lignes de (v-a), « dix-huit » lignes, « neuf » corps, « huit gardes », « dix-sept fichiers » relevés ; et les
   notions corrigées en P2 survivant sous une autre forme (« huit assertions », « équivalente » pour `AppState::trace()`,
   « interdit d'élargir », « issue P3 demandée », « méthodes de `Trace` », champs publics de `ContexteAcces`,
   `pub(in crate::services) MySqlPool`, « (iii) », « (iv) » hors des relevés datés). Greper la **valeur**.

## Lentille R — la spécification contre le code

Pour chaque affirmation de la fiche sur le code, ouvre le site cité (`sed -n`, `grep -nF`) et **copie la sortie**.
Axes, tous à exercer — ceux des passes précédentes (`323-validate-prompt-p2.md`, lentille R, axes 1 à 8), **plus** les
points 1 à 8 ci-dessus :

1. **Les faits 1 à 10** — revérifie-les, et ceux que la P2 a touchés (9).
2. **Le motif et sa faisabilité en Rust** — champs privés, accesseurs, fonctions d'audit libres sur `&Trace`, `tracer`.
3. **Comportement inchangé** — ordre des erreurs, `as_of`, mesures du `span`, deux régimes du grand livre.
4. **Le contrôle d'accès** — AC8 (a)-(e), (c-val)/(c-lec), `identite_societe`, R3.
5. **La trace et la séparation** — tables de l'AC9 (b)/(b′), ligne par ligne (point 2).
6. **Inventaire clos (AC11)** — dépôts appelés, `invoice_settlements`, fixtures constructibles.
7. **Mutations (AC12)** — point 5.
8. **Manuel et documentation** — `docs/manual/fr/admin-manual.tex`, `user-manual.tex`, `docs/api-external.md`, **et les
   PDF aplatis** : `pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oiE ".{80}(report\.generated|journal d.audit|aucune mutation|n.écrit rien).{80}"`
   (et `user-manual.pdf`) ; point 8.

## Lentille F — faisabilité et fausses prémisses

Axes des passes précédentes (`323-validate-prompt-p2.md`, lentille F, axes 1 à 6), **plus** les points 1 à 9
ci-dessus :

1. **Prémisses** — dont celles que la P2 a ajoutées (C-32-3-24 à 30). L'inventaire clos est-il **praticable** pour le
   développement et pour le rollout 32-4 (R8 : « s'amende ligne par ligne ») sans devenir une liste que chaque story
   recopie à l'aveugle ?
2. **Périmètre et découpage** — toujours quatre modules ? Le signal de recyclage déclaré : la P2 l'a-t-elle traité à
   la racine (inventaire clos) ou par une énumération plus longue ? Le test tabulaire par la route alourdit-il la story
   au point de changer le signal de découpage ? Dis ce que tu en penses ; l'arbitrage reste au Project Lead.
3. **Indépendance de Q11** — l'AC10 (v-d) et le *Hors périmètre* préjugent-ils de la forme ou du lieu ?
4. **Décomptes** — point 9, recomptés à la source.
5. **Cohérence interne** — AC1 à AC15, T0 à T6, règles R1 à R8, esquisse (conforme aux listes de (v) ?), *Hors
   périmètre*, *Fichiers touchés*, Risques, Change Log, registre (amendements datés de C-32-3-17 à 22 cohérents avec
   24 à 30), clé du sprint-status.
6. **Ce que la fiche omet** — appelants, gardes, recette visuelle, CHANGELOG, issues (#729, #730).

## Ce que tu rends

Findings `R1…` ou `F1…`, sévérité CRITICAL / HIGH / MEDIUM / LOW, ligne de la fiche, site du dépôt, preuve (commande et
sortie copiée), correction proposée ; pour chaque finding, dis s'il **naît de la remédiation P2**, de celle de la P1
ou de la rédaction d'origine, et s'il est une **forme recyclée** d'un finding antérieur (nomme-le). Un bilan d'en-tête
**cohérent avec le corps**, recompté. ⛔ **La liste des axes exercés ET non exercés** (R1 à R8, F1 à F6, et les points 1
à 9 de l'attention P2) — sans elle, « 0 finding » ne compte pas. Ce que la fiche **prescrit** au développement n'existe
pas encore : son absence n'est pas un défaut. Rapport écrit **d'un seul jet**, un seul fichier par lentille :
`/home/gcorbaz/devel/kesh-gate-logs/323-validate-p3-R.md` et `/home/gcorbaz/devel/kesh-gate-logs/323-validate-p3-F.md`.
Dernier message : chemin, bilan par sévérité, une ligne par finding MEDIUM ou plus.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport, **aucun fichier temporaire**, rien dans le scratchpad ; **aucune
redirection** (`>`, `>>`, `tee`), pas même vers un fichier jetable — un fichier créé par erreur se déclare au rapport,
il ne se supprime pas. Aucun `git checkout`/`switch` sous aucune forme ; aucune commande qui écrit, compile ou
exécute : `scripts/*` (dont **`scripts/prepare-release.sh`**), `make`, `latexmk`, `cargo`, `rustup`, `rustfmt`, `npm`,
`npx`, `docker`, `curl`, **`python3`, `python`, `perl`, `ruby`, `node`**, `mariadb`/`mysql`, `sed -i`,
`git commit|add|stash|fetch|pull|reset|rebase|tag|push`, `gh` en écriture. `pdftotext` vers la sortie standard
seulement. Aucune donnée réelle de Guy (rien de `~/kesh-test/`). Autorisés : lecture, `grep`, `sed -n`,
`git log|show|diff|blame|merge-base`, `gh issue view|list`, `pdftotext … -`, et les filtres en lecture seule `tr`,
`sort`, `cut`, `head`, `awk`, `wc`, `uniq` vers la sortie standard.
