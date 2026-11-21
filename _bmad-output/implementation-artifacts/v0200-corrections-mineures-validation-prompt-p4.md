# Prompt — validation P4 de la fiche `v0200-corrections-mineures` (#1348, #1336, #1342, #1349, #1343, #1375), passe complète après découpage

Tu valides, **en lecture seule**, la fiche de story `v0200-corrections-mineures` de Kesh (comptabilité suisse ; Rust/Axum, MariaDB,
SvelteKit 5 runes), release **0.20.0** (coupure le jeudi 2026-11-26 au soir). Kesh tient les livres réels du Project Lead (Guy), qui
a **déjà émis de vraies factures remises sur papier** : rien de ce qui existe (facture, avoir, lignes, totaux, TVA, écritures,
numéros, PDF) ne doit changer. **Aucune migration** n'est permise à cette story.

- **Modèle : Sonnet. Contexte frais. Passe complète** : le périmètre a changé (découpage décidé par Guy le 2026-11-19 — la garde
  générale de l'i18n est sortie vers l'issue #1406, milestone v0.21.0). Relis toute la fiche, en commençant par le découpage et les
  remédiations de la P3. Langue du rapport : français.
- Worktree : `/home/gcorbaz/devel/kesh-v0200-cm`, branche `story/v0200-corrections-mineures`, partie de
  `epic/48-corrections-0-20-0` = `a051e17e5`. Le code de la branche est celui de `a051e17e5` : seuls la fiche, ses prompts et
  `sprint-status.yaml` diffèrent (vérifie-le : `git diff --stat a051e17e5 HEAD`). Fiche :
  `/home/gcorbaz/devel/kesh-v0200-cm/_bmad-output/implementation-artifacts/v0200-corrections-mineures.md`. Version d'avant le
  découpage, pour comparer : `git show HEAD~1:_bmad-output/implementation-artifacts/v0200-corrections-mineures.md`.
- Lis d'abord :
  - le `CLAUDE.md` du worktree (Qualité du code, Recette visuelle, Review Iteration Rule, Inventorier les sites NON RÉSOLUS,
    Propagation post-patch, Recompter, Priorités des défauts, Issue Tracking Rule, Règle de splitting préventif, Documentation à
    synchroniser, Fusions et fichiers communs, Cadence) ;
  - `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` ;
  - les rapports des passes précédentes : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p1.md`, `validation-p2.md`,
    `validation-p3.md` (V3-1 à V3-8) ;
  - la section « Découpage » et l'entrée « découpage et remédiation de la validation P3 » du Change Log de la fiche ;
  - le texte des six issues, copié pour toi (`gh` t'est interdit) : `/home/gcorbaz/devel/kesh-gate-logs/v0200cm/issues-p2.txt` ;
    l'issue #1406 s'intitule « Garde générale : aucune clé i18n appelée par le code ne doit manquer au catalogue (tous les crates,
    QR-facture comprise) », milestone v0.21.0, ouverte ;
  - la fiche sœur `_bmad-output/implementation-artifacts/48-15-troncatures-silencieuses.md` (règle de décompte des modules) ;
    `docs/i18n-glossaire.md` ;
  - la fiche de la 48-12b, sur sa branche :
    `git show story/48-12-reponses-perdues-fournisseurs:_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`.

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

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/v0200cm/validation-p4.md`** (le seul fichier que tu écris, par l'outil d'écriture).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Le découpage est propre.** Aucune trace de la garde générale dans le corps de la fiche (titre, statut, « Ce qui se lit au code »,
   D1 à D7, AC, tableau des mutations, filets, tâches, gates, Note de version, Dev Notes) : ni extracteur, ni ensemble clos de sites à
   clé calculée, ni sonde de garde, ni test `chaque_cle_appelee_par_le_serveur_existe_au_catalogue`, ni mutation M2 à M4d ou M20
   référencée comme active. **Rien d'orphelin** : chaque AC renvoie à une décision existante et à des mutations qui existent ;
   chaque mutation du tableau est tuée par un test nommé dans un AC ; chaque renvoi interne (« D1 c », « D1 d », « AC3 »…) désigne le
   bon objet après la renumérotation ; aucune tâche, aucun gate, aucune Note de version ne parle d'un travail sorti. #1406 est cité
   au hors-périmètre avec sa raison (règle de splitting, trois passes). Compare à `HEAD~1` pour lister ce qui est sorti et vérifier
   que rien d'utile n'a été perdu en route (par exemple : la `TABLE`, la conversion de `:2759`, le test du quota, la classification
   des codes). Les entrées passées du Change Log sont l'historique : elles peuvent nommer la garde.
2. **Le test des 21 clés n'est pas vrai par construction** (D1 d, AC3). Sa liste ne doit pas sortir du même extracteur, ni du même
   fichier, que ce qu'elle vérifie. Vérifie : la liste est écrite en dur, recopiée du tableau de la fiche ; le test lit les vrais
   catalogues (`I18nBundle::load`) ; il échoue avant le correctif sur les 21 clés ; l'assertion de montage (21 clés, distinctes)
   attrape une liste tronquée. Relis toi-même au code, à `a051e17e5`, que les 21 clés du tableau sont bien **absentes** des quatre
   `crates/kesh-i18n/locales/*/messages.ftl` et bien **appelées** à leur site de `errors.rs` (ligne citée), et qu'aucune n'est en
   double ou mal orthographiée. Les mutations M1, M19, M21 tuent-elles ce test pour la bonne raison, avec la parité verte quand la
   fiche le dit ? La `TABLE` (D1 c) : deux sites pour `error-invalid-input-generic` (`errors.rs:5991`, `:7377`), paramètre `{ $max }`
   pour le quota — juste au code (`textes_coherents.rs:724-1060`) ?
3. **Les remédiations de la P3, une par une (V3-1 à V3-8).** V3-1 traité par le découpage (aucune clé de
   `kesh_qrbill::types::I18N_KEYS` absente aujourd'hui : vérifie-le) ; V3-2 (alerte au-dessus de la liste,
   `ContactPersonsManager.svelte:103-149`) ; V3-3 (textes du catalogue, `fr-CH/messages.ftl:533`, `:536`, `:2656`, `:2658`) ;
   V3-4 (comptes de rouges de M8 et M9, recomptés contre les AC7 et AC8) ; V3-5 (relecture du doc-comment de `unknown-outcome.ts` à
   la fusion de la seconde story dans l'épique, par l'orchestrateur — exécutable ? sans renvoi à une « revue d'intégration ») ;
   V3-6 (dérogation : « ajout de ces issues à la 0.20.0 décidé par Guy le 2026-11-19 ; élargissement de la dérogation à neuf stories
   à confirmer au compte rendu », sans attribution par inférence — en-tête, Dev Notes, `sprint-status.yaml`) ; V3-7 (lignes citées) ;
   V3-8 (sans objet).
4. **Les remédiations de la P1 et de la P2 restent-elles vraies** après le découpage (V1-1, V1-4, V1-5, V1-7, V1-10, V1-11, V1-14 ;
   V2-2, V2-4, V2-6 à V2-9) ? Le découpage en a-t-il défait une ?
5. **Décisions, périmètre, modules.** D1 à D7 tiennent-ils au code ? La borne de **cinq modules**, recomptée après le découpage
   (règle de la 48-15), est-elle juste ? Un sixième module est un finding ; un module devenu inutile aussi.
6. **Critères et tests.** Chaque AC testable et rouge pour la bonne raison ; mutations M1, M5 à M19, M21 uniques, chacune tuée par un
   test nommé ; sondes positives ; filets déclarés ; E2E pour chaque comportement visible ; sélecteurs `data-testid` ou rôle (garde
   KF-043) ; comptes de rouges justes.
7. **i18n.** Vingt-deux clés neuves aux quatre locales, `{ $max }`, `parity_between_locales`, `dette-parite-connue.txt`,
   `les_replis_rust_suivent_le_catalogue`, `lint-i18n-ownership`, `sitesTotal` ; glossaire ; messages serveur restés en français
   (#710) déclarés. **Recompte tous les nombres de clés de la fiche par grep de la valeur** (`vingt`, `vingt et une`, `vingt-deux`,
   `dix-neuf`, `\b20\b`, `\b21\b`, `\b22\b`, `sept`, `quatorze`) : chacun cohérent avec sa ventilation.
8. **Manuel, `.tex` et PDF.** `docs/manual/fr/user-manual.tex` (lignes 219, 1348-1360, 1388-1395 et voisines) **et** le PDF publié,
   aplati avant de greper :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -oP '.{0,200}<motif>.{0,200}'`
   (motifs au moins : « Email », « Téléphone », « Adresse structurée », « grisé », « union conditionnel », « pays saisi seul »,
   « personnes de contact », « Impossible d'ajouter », « Compte comptable par défaut »). Ce que le manuel dit contredit-il le code,
   avant ou après la story ? L'AC14 (a à f) suffit-elle, et ses phrases décrivent-elles l'écran tel qu'il sera ? (Le PDF publié est
   antérieur au `.tex` récent : c'est normal, il se régénère à la release.)
9. **Notes de version et documentation.** Une puce par issue de comportement, sans nommer d'epic ; « Passer à la 0.20.0 » dit les
   changements de l'API (y compris `error.message` de vingt et un refus) ; aucun mot-clé de fermeture (`closes`, `fixes`, `resolves`
   et variantes) dans la fiche ni dans les messages de commit (`git log origin/epic/48-corrections-0-20-0..HEAD --format=%B |
   /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` doit être vide).
10. **Fichiers communs et coordination.** `errors.rs` avec la 48-12a ; `unknown-outcome.ts` et `user-manual.tex` avec la 48-12b ;
    catalogues, `textes_coherents.rs`, `i18n-keys.test.ts`, `sprint-status.yaml` (clé au bloc de l'Epic 48, statut `backlog`,
    commentaire à jour avec le découpage et #1406).
11. **Le Change Log du découpage.** Ce qui est dit sorti l'est-il vraiment, ce qui est dit gardé est-il là ? Décomptes (8 findings =
    1 MEDIUM + 7 LOW ; 21 / 22 / 21 / 21 / 21 ; modules cinq) recomptés depuis la source ; chaque « confirmé » cite une ligne lue.

## Rapport

Pour chaque finding : identifiant `V4-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), axe, constat **avec la ligne lue citée**
(`chemin:ligne` et l'extrait), conséquence, remède proposé. Un finding qui naît du découpage ou d'une remédiation de la P3 le dit
(« né du découpage », « né de V3-n ») ; un défaut d'une passe précédente qui revient sous une autre forme le dit aussi (signal de
recyclage). Pour tout CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-pattern, la commande `grep -nF` qui le
prouve et sa sortie.

En fin de rapport :
- **verdict chiffré** : décompte **CRITICAL / HIGH / MEDIUM / LOW**, recompté depuis tes findings, avec sa ventilation par finding ;
  et la conclusion (« prête pour `ready-for-dev` » seulement si aucun finding au-dessus de LOW) ;
- la liste des **axes exercés** (numéros, et pour chacun ce qui a été lu) et des **axes non exercés ou exercés en partie**, avec leur
  raison — un « 0 finding » sans cette liste ne compte pas comme passe ;
- les **commandes lancées**, toutes en lecture.
