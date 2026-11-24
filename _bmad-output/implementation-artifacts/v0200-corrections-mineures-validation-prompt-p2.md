# Prompt — validation P2 de la fiche `v0200-corrections-mineures` (#1348, #1336, #1342, #1349, #1343, #1375), passe complète

Tu valides, **en lecture seule**, la fiche de story `v0200-corrections-mineures` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), release **0.20.0** (coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead (Guy), qui
a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures,
numéros, PDF) ne doit changer. **Aucune migration** n'est permise à cette story.

- **Modèle : Sonnet. Contexte frais. Passe complète** (la P1 était Opus ; celle-ci n'est pas ciblée : elle relit toute la fiche, en
  commençant par les remédiations). Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de
  `epic/48-corrections-0-20-0` = `a051e17e5`. Le code de la branche est celui de `a051e17e5` : seuls la fiche, ses prompts et
  `sprint-status.yaml` diffèrent (vérifie-le : `git diff --stat a051e17e5 HEAD`). Fiche :
  `/home/gcorbaz/devel/kesh-v0200-cm/_bmad-output/implementation-artifacts/v0200-corrections-mineures.md`.
- Lis d'abord :
  - le `CLAUDE.md` du worktree (Qualité du code, Recette visuelle, Review Iteration Rule, Inventorier les sites NON RÉSOLUS,
    Propagation post-patch, Recompter, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif, Documentation à
    synchroniser, Fusions et fichiers communs) ;
  - `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` ;
  - le rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p1.md` (findings V1-1 à V1-14) ;
  - l'entrée « remédiation de la validation P1 » du Change Log de la fiche ;
  - le texte des six issues, copié pour toi (`gh` t'est interdit) : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/issues-p2.txt` ;
  - la fiche sœur `_bmad-output/implementation-artifacts/48-15-troncatures-silencieuses.md` (motif réutilisé, règle de décompte des
    modules) ;
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
`/usr/bin/grep -P`, `sed -n`, `awk` et `perl -ne` / `perl -0777 -ne` vers la sortie standard, `cat`, `ls`, `wc`,
`pdftotext … -` (vers la sortie standard).

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p2.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P1, une par une (V1-1 à V1-14).** Pour chaque finding : la remédiation déclarée au Change Log est-elle
   présente dans la fiche, partout où elle doit l'être (décision, AC, mutation, tâche, Note de version, Dev Notes), juste au code, et
   sans défaut neuf ? En particulier :
   - **V1-1** : `errors.rs:2759-2762` (`&t(` + repli `format!`) ; la conversion prescrite en `t_args` (D1 b), la table de paramètres
     de la `TABLE` (D1 e ; relis `les_replis_rust_suivent_le_catalogue`, `textes_coherents.rs:724-1060`, pour vérifier que le
     littéral du repli `format!` sera bien trouvé une fois `{ $max }` remplacé par `{max}`) ; le test de l'AC2
     (`invoice_send_email_e2e.rs:1884`, `spawn_app` `:84-102`) rougit-il avant le correctif et sous M18, et seulement pour la bonne
     raison ? Fluent rend-il bien `{$max}` quand l'argument manque (`crates/kesh-i18n/src/loader.rs:110-124`) ?
   - **V1-2 / #1375** : D6, AC16, tâche 7 ; la prémisse « la 48-12b ne modifie pas `unknown-outcome.ts` » (fiche 48-12b `:145`,
     `:149`) ; le cas (ii) — un commentaire écrit d'après une fiche, à reprendre par le second arrivé — est-il un engagement tenable,
     et la règle de retour si la 48-12b sort de la release est-elle suffisante ?
   - **V1-3** : l'extension de l'extracteur à la famille `format` (D1 c), ses deux sondes, l'ensemble clos des 8 sites calculés et les
     14 appels littéraux (D1 d, « Ce qui se lit au code ») ; la réfutation partielle (`invoice_pdf_service.rs:1683`, `:1858` dans le
     module de test `:964-1871`) ; M2b, M3b, M4b tuées par un test nommé.
   - **V1-4** : D5 d (`ContactPersonsManager.svelte:69-70`, patron `SupplierCreatePanel.svelte:137-142`), AC11, M17 ; le test
     Vitest proposé est-il écrivable dans `ContactPersonsManager.test.ts` (mocks existants de `contact-persons.api`) ?
   - **V1-5** : la classification des vingt codes et la réécriture de la Note de version et de D1 b ; relis toi-même les fichiers de
     l'écran nommés.
   - **V1-6** : recompte (famille `t`, à clé calculée : `errors.rs` 24, total 28).
   - **V1-7** : l'exception retirée de D4 ; la coexistence d'un refus du serveur (`formError`) avec la ligne neuve.
   - **V1-8 à V1-14** : AC14 (manuels DE/IT/EN sans objet ; phrase d) ; contact étranger dont on retire l'adresse (AC7 en deux temps,
     AC14 a, Note de version) ; AC10 (chaque cas lit le début du message ; U+200B filet de message) ; lignes de `companies.rs` ;
     #1405 hors périmètre ; le message de D5 avec le geste (« — effacez le champ et saisissez-le de nouveau »), repris à l'AC11, à
     l'AC14 c et à la Note de version.
2. **#1375 ajoutée.** Périmètre (titre, statut, section du « Ce qui se lit au code », D6, AC16, tâches, Dev Notes, modules) ; la phrase
   actuelle du doc-comment (`frontend/src/lib/shared/utils/unknown-outcome.ts:40-68`) ; ce que la 48-12b couvre selon sa fiche
   (création, complétion, modification figée ; #986 reporté) ; la fiche promet-elle au doc-comment quelque chose que la 48-12b ne
   livre pas ?
3. **Inventaire des sites qui rendent une clé brute — ensemble clos.** Refais-le toi-même, en lecture seule, **à partir du symptôme**
   (`I18nBundle::format` rend la clé quand elle manque, `loader.rs:110-124`), non à partir de la liste de la fiche :
   - tous les appelants de `I18nBundle::format` dans le workspace (`crates/*/src`, modules de test exclus), quelle que soit la forme
     d'appel (`t`, `t_args`, `.format(` sur une ou plusieurs lignes, fonction intermédiaire qui reçoit une clé) ;
   - pour chacun : clé littérale (présente au catalogue fr-CH ?) ou calculée (d'où viennent ses clés ? le site se protège-t-il de la
     clé brute ?) ;
   - compare à la fiche : familles `t` (301 littéraux, 20 clés absentes, 28 calculés) et `format` (14 littéraux, 8 calculés). Un site
     omis, un compte faux, une famille non vue, ou une assertion de la garde vraie par construction (les deux côtés sortant du même
     extracteur) est un finding. Le frontend (`i18nMsg`, `frontend/src/lib/shared/utils/i18n.svelte.ts`) est-il bien hors du
     symptôme ?
4. **Décisions et périmètre.** D1 b et D5 (élargissements gardés par décision de l'orchestrateur) restent-ils cohérents avec leurs
   corrections ? D3, D4, D5 d, D6 et D7 tiennent-ils au code ? La borne de **cinq modules** (Dev Notes, règle de décompte de la 48-15 :
   `features/contacts` et `routes/(app)/contacts` comptent pour un domaine) est-elle juste ? Un sixième module est un finding.
5. **Cohérence écran / serveur.** Chaque borne et chaque refus de l'écran comptent comme le serveur ; aucun refus muet (y compris
   l'écran des interlocuteurs) ; un refus serveur s'affiche et la saisie reste.
6. **Données existantes et factures réelles.** Limites de l'AC12 b ; aucune facture ni aucun PDF ne change (AC12 a) ; `errors.rs`
   seul site `:2759` (AC12 d) ; aucune migration requise.
7. **Critères et tests.** Chaque AC testable et rouge pour la bonne raison ; mutations M1 à M18 (avec M2b, M3b, M4b) uniques, chacune
   tuée par un test nommé ; sondes positives ; filets déclarés ; aucun test vrai par construction ; E2E pour chaque comportement
   visible ; sélecteurs `data-testid` ou rôle (garde KF-043).
8. **i18n.** Clés aux quatre locales, `{ $max }` aux quatre, `parity_between_locales`, `dette-parite-connue.txt` (lis-le),
   `les_replis_rust_suivent_le_catalogue`, `lint-i18n-ownership` (que contrôle-t-il ? `ContactPersonsManager.svelte` est sous
   `src/lib/features`), `sitesTotal` ; messages serveur restés en français (#710) déclarés.
9. **Manuel, `.tex` et PDF.** `docs/manual/fr/user-manual.tex` (lignes 219, 1348-1360 et voisines ; et toute description de l'ajout
   d'une personne de contact) **et** le PDF publié, aplati avant de greper :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oP '.{0,200}<motif>.{0,200}'`
   (motifs au moins : « Email », « Téléphone », « Adresse structurée », « grisé », « union conditionnel », « pays saisi seul »,
   « personne de contact », « Impossible d'ajouter »). Ce que le manuel dit contredit-il le code, avant ou après la story ? L'AC14
   suffit-elle ? (Le PDF publié est antérieur au `.tex` récent : c'est normal, il se régénère à la release.)
10. **Notes de version et documentation.** Une puce par issue de comportement, sans nommer d'epic ; la phrase « Passer à la 0.20.0 »
    dit les changements de l'API (y compris `error.message`) ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves` et variantes)
    dans la fiche ni dans les messages de commit (`git log origin/epic/48-corrections-0-20-0..HEAD --format=%B | /usr/bin/grep -niE
    '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide).
11. **Fichiers communs et coordination.** `errors.rs` avec la 48-12a ; `unknown-outcome.ts` avec la 48-12b (conflit sémantique) ;
    catalogues, `textes_coherents.rs`, `i18n-keys.test.ts`, `sprint-status.yaml` (clé au bloc de l'Epic 48, statut `backlog`,
    commentaire à jour) ; la PR #1327 de Beat ne touche-t-elle aucun fichier de la story ?
12. **Le Change Log de la remédiation.** Ses décomptes (14 findings = 13 confirmés + 1 déjà tracé ; 1 réfuté en partie ; modules
    cinq) se recomptent-ils depuis la source ? Chaque « confirmé » cite-t-il une ligne lue ? Le grep du symptôme déclaré a-t-il laissé
    un reste hors historique (refais-le : anciens comptes, anciennes lignes, « quatre modules », attribution de #1375 à la 48-12b,
    exception de `formError`) ?

## Rapport

Pour chaque finding : identifiant `V2-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Un finding qui naît d'une remédiation de la P1 le dit (« né de V1-n ») ;
un défaut de la P1 qui revient sous une autre forme le dit aussi (signal de recyclage, `CLAUDE.md` § Règle de splitting préventif).
Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-pattern, la commande `grep -nF` qui le prouve et
sa sortie.

En fin de rapport :
- **verdict chiffré** : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings, avec sa ventilation par finding ;
  et la conclusion (« prête pour `ready-for-dev` » seulement si aucun finding au-dessus de LOW) ;
- la liste des **axes exercés** (numéros, et pour chacun ce qui a été lu) et des **axes non exercés ou exercés en partie**, avec leur
  raison — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les **commandes lancées**, toutes en lecture.
