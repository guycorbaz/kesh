# Prompt — validation P3 de la fiche `v0200-corrections-mineures` (#1348, #1336, #1342, #1349, #1343, #1375), passe complète

Tu valides, **en lecture seule**, la fiche de story `v0200-corrections-mineures` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), release **0.20.0** (coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead (Guy), qui
a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures,
numéros, PDF) ne doit changer. **Aucune migration** n'est permise à cette story.

- **Modèle : Opus. Contexte frais. Passe complète** (la P2 était Sonnet ; celle-ci n'est pas ciblée : elle relit toute la fiche, en
  commençant par les remédiations de la P2). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de
  `epic/48-corrections-0-20-0` = `a051e17e5`. Le code de la branche est celui de `a051e17e5` : seuls la fiche, ses prompts et
  `sprint-status.yaml` diffèrent (vérifie-le : `git diff --stat a051e17e5 HEAD`). Fiche :
  `/home/gcorbaz/devel/kesh-v0200-cm/_bmad-output/implementation-artifacts/v0200-corrections-mineures.md`.
- **Signal de recyclage en cours** : la P2 a trouvé (V2-1) une forme neuve d'un défaut de la P1 (V1-3) — l'inventaire des sites qui
  rendent une clé brute n'était pas clos. Si tu trouves encore un défaut de cette classe, dis-le en tête du rapport : il déclenche la
  règle de splitting préventif du `CLAUDE.md`.
- Lis d'abord :
  - le `CLAUDE.md` du worktree (Qualité du code, Recette visuelle, Review Iteration Rule, Inventorier les sites NON RÉSOLUS,
    Propagation post-patch, Recompter, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif, Documentation à
    synchroniser, Fusions et fichiers communs) ;
  - `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` ;
  - les rapports des passes précédentes : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p1.md` (V1-1 à V1-14) et
    `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p2.md` (V2-1 à V2-9) ;
  - les entrées « remédiation de la validation P1 » et « remédiation de la validation P2 » du Change Log de la fiche ;
  - le texte des six issues, copié pour toi (`gh` t'est interdit) : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/issues-p2.txt` ;
  - la fiche sœur `_bmad-output/implementation-artifacts/48-15-troncatures-silencieuses.md` (motif réutilisé, règle de décompte des
    modules) ; `docs/i18n-glossaire.md` ;
  - les fiches de la 48-12a et de la 48-12b, sur leur branche, par `git show` :
    `git show story/48-12-reponses-perdues-fournisseurs:_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`
    (et `48-12a-doublons-fournisseurs-serveur.md`).

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. En particulier, tu ne lances **ni** `cargo` (build, test, nextest, clippy, fmt),
**ni** `npm` / `npx` (check, test, build, lint), **ni** `make`, **ni** aucun `scripts/*` (dont `scripts/prepare-release.sh`,
`scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`, `scripts/recette-montee.sh`, `scripts/install-hooks.sh`,
`scripts/seed-dev-db.sql` par un client), **ni** `docker` / `docker compose`, **ni** `mariadb` / `sqlx`, **ni** `gh` (aucune
sous-commande, pas même en lecture : le texte des issues t'est fourni), **ni** `python3`, **ni** `sed -i`, `tee`, `perl -i`, **ni**
aucune redirection `>` ou `>>` vers un fichier, **ni** `git` qui écrit (`commit`, `checkout`, `switch`, `stash`, `restore`, `reset`,
`add`, `rm`, `mv`, `merge`, `rebase`, `cherry-pick`, `tag`, `branch` avec option d'écriture, `push`, `fetch`, `pull`, `worktree`,
`gc`). Rien sous `~/kesh/`. Permis : `git log`, `git show`, `git diff`, `git grep`, `git ls-tree`, `git branch --list`, `grep`,
`/usr/bin/grep -P`, `sed -n`, `awk` et `perl -ne` / `perl -0777 -ne` vers la sortie standard, `cat`, `ls`, `wc`, `find`,
`pdftotext … -` (vers la sortie standard).

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p3.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P2, une par une (V2-1 à V2-9).** Pour chaque finding : la remédiation déclarée au Change Log est-elle
   présente partout où elle doit l'être (décision, AC, mutation, tâche, Note de version, Dev Notes), juste au code, et sans défaut
   neuf ? En particulier :
   - **V2-1** : `error-invalid-input-generic` (`errors.rs:5985-5994`) au tableau, à D1 b et e (deux sites à la `TABLE` : la garde
     `les_replis_rust_suivent_le_catalogue` compte-t-elle bien les littéraux du fichier **entier**, module de test compris ? relis
     `textes_coherents.rs:1022-1060` et `errors.rs:7370-7380`) ; le **troisième extracteur** de D1 c (forme des littéraux, préfixes
     lus dans le catalogue, liste close d'exceptions, sonde, limites déclarées) : est-il réellement indépendant des deux autres, ou une
     de ses assertions est-elle vraie par construction (préfixes et clés sortant du même catalogue — juge si cela laisse passer une
     clé absente, et laquelle) ? M4c, M4d, M19, M20 : chacune tuée par un test nommé, et pour la bonne raison ?
   - **V2-2, V2-3** : décomptes 297 / 301 ; retrait des lignes de commentaire (règle de `util::code`, `crates/kesh-api/src/util.rs:303-310`,
     non importable d'un test d'intégration : le renvoi est-il juste ?) ; les cinq doc-comments à appels fictifs cités ; les comptes
     « jusqu'à 32 » et « 9 » sans retrait.
   - **V2-4** : D6 et Dev Notes — la relecture du second arrivé est portée par cette fiche seule ; est-ce exécutable par celui qui
     fusionne en second, sans rien demander à la 48-12b ?
   - **V2-5** : « neuvième story de l'Epic 48 » et la ligne des Dev Notes (dérogation à élargir sur décision de Guy du 2026-11-19).
   - **V2-6 à V2-9** : glossaire nommé ; `docs/api-external.md` (« rien à changer » : vérifie au guide) et `user-manual.tex` aux
     fichiers communs ; `createMock` hissé (`ContactPersonsManager.test.ts:10-15`) et AC14 f inconditionnel
     (`user-manual.tex:1389-1395`) ; propagation au commentaire de `contacts-page.test.ts:717-720`.
2. **Les remédiations de la P1 restent-elles vraies** après la P2 (V1-1 à V1-14, tableau de la P2 « Les remédiations de la P1 ») ?
   Une remédiation de la P2 a-t-elle défait l'une d'elles ?
3. **Inventaire des clés, refait par le symptôme, avec les trois extracteurs.** En lecture seule (`perl -ne` / `-0777 -ne` vers la
   sortie standard), sur `crates/kesh-api/src/**/*.rs`, modules de test et lignes de commentaire retirés, contre
   `crates/kesh-i18n/locales/fr-CH/messages.ftl` :
   - (a) famille `t` / `t_args` à clé littérale : compte (297 ?), clés absentes (vingt ?) ; à clé calculée, par fichier (28 ?) ;
   - (b) famille `.format(<arg>, "<littéral>"` sur une ou plusieurs lignes : compte (14 ?), absentes (aucune ?) ; à clé calculée (8 ?) ;
   - (c) littéraux de forme clé à préfixe du catalogue : absents = vingt et une clés + `mcp-protocol-version` + `setup-required`,
     rien d'autre ?
   - (d) clés composées à l'exécution (`format!` d'un gabarit à préfixe du catalogue, concaténation, `push_str`, `replace`) hors tests :
     ensemble clos = `audit_labels.rs:240`, `routes/reports.rs:957` ? Cherche toute autre façon de fabriquer une clé.
   - (e) appelants de `I18nBundle::format` hors `kesh-api` et `kesh-i18n` (workspace entier) ; et le frontend (`i18nMsg`) hors du symptôme ?
   Compare chaque compte à la fiche ; un site omis, un compte faux, ou une sonde qui passerait à vide est un finding. **Recompte aussi
   tous les nombres de clés de la fiche par grep de la valeur** (`vingt`, `vingt et une`, `vingt-deux`, `dix-neuf`, `\b20\b`, `\b21\b`,
   `\b22\b`, `sept`, `quatorze`) : chacun cohérent avec sa ventilation (21 clés serveur absentes, 22 clés neuves, 21 entrées de `TABLE`,
   21 codes dont sept absents de l'écran) ? Les mentions dans les entrées passées du Change Log sont l'historique : ne les compte pas
   comme défaut, sauf fausses à leur date.
4. **Décisions et périmètre.** D1 à D7 tiennent-ils au code ? Les élargissements (D1 b, D5) restent-ils cohérents ? La borne de
   **cinq modules** (règle de décompte de la 48-15) est-elle juste ? Un sixième module est un finding.
5. **Cohérence écran / serveur ; données existantes ; factures réelles.** Aucun refus muet ; limites de l'AC12 b ; aucun fichier de
   facture ni de PDF ; `errors.rs` seul site `:2759` (AC12 d) ; aucune migration.
6. **Critères et tests.** Chaque AC testable et rouge pour la bonne raison ; mutations M1 à M20 (avec M2b, M3b, M4b, M4c, M4d) uniques,
   chacune tuée par un test nommé ; sondes positives ; filets déclarés ; aucun test vrai par construction ; E2E pour chaque
   comportement visible ; sélecteurs `data-testid` ou rôle (garde KF-043).
7. **i18n.** Clés aux quatre locales (vingt-deux), `{ $max }`, `parity_between_locales`, `dette-parite-connue.txt`,
   `les_replis_rust_suivent_le_catalogue`, `lint-i18n-ownership`, `sitesTotal` ; glossaire ; messages serveur restés en français
   (#710) déclarés.
8. **Manuel, `.tex` et PDF.** `docs/manual/fr/user-manual.tex` (lignes 219, 1348-1360, 1389-1395 et voisines) **et** le PDF publié,
   aplati avant de greper :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oP '.{0,200}<motif>.{0,200}'`
   (motifs au moins : « Email », « Téléphone », « Adresse structurée », « grisé », « union conditionnel », « pays saisi seul »,
   « personnes de contact », « Impossible d'ajouter », « Compte comptable par défaut »). Ce que le manuel dit contredit-il le code,
   avant ou après la story ? L'AC14 (a à f) suffit-elle ? (Le PDF publié est antérieur au `.tex` récent : c'est normal, il se
   régénère à la release.)
9. **Notes de version et documentation.** Une puce par issue de comportement, sans nommer d'epic ; « Passer à la 0.20.0 » dit les
   changements de l'API (y compris `error.message` de vingt et un refus) ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`
   et variantes) dans la fiche ni dans les messages de commit (`git log origin/epic/48-corrections-0-20-0..HEAD --format=%B |
   /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide).
10. **Fichiers communs et coordination.** `errors.rs` avec la 48-12a ; `unknown-outcome.ts` et `user-manual.tex` avec la 48-12b ;
    catalogues, `textes_coherents.rs`, `i18n-keys.test.ts`, `sprint-status.yaml` (clé au bloc de l'Epic 48, statut `backlog`,
    commentaire à jour).
11. **Le Change Log de la remédiation P2.** Ses décomptes (9 findings = 1 MEDIUM + 8 LOW, tous confirmés ; décompte final des clés ;
    modules cinq) se recomptent-ils depuis la source ? Chaque « confirmé » cite-t-il une ligne lue ? Le signal de recyclage y est-il
    écrit ?

## Rapport

Pour chaque finding : identifiant `V3-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Un finding qui naît d'une remédiation de la P2 le dit (« né de V2-n ») ;
un défaut d'une passe précédente qui revient sous une autre forme le dit aussi (signal de recyclage). Pour tout CRITICAL ou HIGH qui
affirme l'absence d'un code ou la présence d'un anti-pattern, la commande `grep -nF` qui le prouve et sa sortie.

En fin de rapport :
- **verdict chiffré** : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings, avec sa ventilation par finding ;
  et la conclusion (« prête pour `ready-for-dev` » seulement si aucun finding au-dessus de LOW) ;
- la liste des **axes exercés** (numéros, et pour chacun ce qui a été lu) et des **axes non exercés ou exercés en partie**, avec leur
  raison — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les **commandes lancées**, toutes en lecture.
