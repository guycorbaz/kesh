# Story 22.1 : Unicité canonique du numéro de client

## Status

review

## Story

**As a** indépendant ou fiduciaire qui attribue des numéros de client,
**I want** que deux numéros **identiques à l'œil** soient traités comme un seul par Kesh, quelle que soit la casse, les accents, la forme de composition Unicode ou les caractères invisibles collés au passage,
**so that** la garantie d'unicité que le manuel m'annonce tienne réellement — et que je ne me retrouve ni avec deux contacts au même numéro, ni avec un `409` sur un numéro que je crois neuf.

Ferme **#294** et **#295**. Action **A8** de la rétrospective de l'Epic 16. Première story de l'**Epic 22 « Technical Debt Closure »**.

## Contexte

La Story 16-3b a livré le numéro de client avec une unicité **partielle par société**, portée par une colonne générée qui vaut `NULL` quand le contact est archivé. Le manuel utilisateur annonce, en toutes lettres :

> « Le numéro doit être **unique parmi vos contacts actifs** — c'est ce qui lui permet d'identifier. La **casse ne distingue pas** : `CLI-1` et `cli-1` sont considérés comme le même numéro. »

**Cette garantie ne tient pas.** Trois passes de revue de code l'ont défaite par trois chemins distincts, tous du même genre : *deux valeurs distinctes pour la base, identiques pour l'œil*.

| Chemin | Ce qui se produit | Issue |
|---|---|---|
| **Composition Unicode** | `CLÉ-1` saisi depuis macOS (NFD, `E`+U+0301) et depuis Windows (NFC) sont deux valeurs pour MariaDB, qui compare des séquences d'octets. Les deux contacts sont acceptés et s'affichent **identiques**. | #294 |
| **Caractère invisible encastré** | `CLI-1` et `CLI‹U+200B›-1` coexistent. `U+200B` n'appartient pas à WinAnsi et ne marque **rien** à l'impression : la fiche, la liste et le PDF affichent la même chose. Le ZWSP et `U+00AD` sont extrêmement courants dans un copier-coller depuis un courriel HTML ou un document Word — c'est-à-dire le geste exact que la fonctionnalité invite à faire. | #294 |
| **Collation non déclarée** | La table `contacts` est l'une des **deux seules** du dépôt sans `COLLATE` explicite. La garantie d'insensibilité à la casse dépend donc du défaut de la base **à sa création**. Sous une collation UCA (`utf8mb4_unicode_ci`, ou `uca1400_ai_ci` sur MariaDB 11.x), les `_ci` sont **aussi accent-insensibles** : `CLI-É1` et `CLI-E1` se percutent en 409 là où ils coexistent ailleurs. Même code, comportements opposés selon l'installation. | #295 |

⚠️ **Le test existant donne une confiance qu'il ne mesure pas.** `client_number_uniqueness_is_case_insensitive` compare `CLI-CASE-1` et `cli-case-1` : il passe sous `general_ci` **comme** sous `unicode_ci`, puisque toutes deux sont insensibles à la casse. Aucun test ne couvre un caractère accentué.

## Décisions

**D1 — Colonne de comparaison canonique.** *(arbitrage de Guy, 2026-08-12)*

L'unicité porte sur une colonne **dédiée à la comparaison**, distincte de la valeur affichée. La forme canonique est calculée **en Rust à l'écriture** — MariaDB ne sait pas normaliser en NFC —, et la valeur saisie par l'utilisateur reste **intacte** à l'affichage, sur le PDF et à la recherche.

Les deux autres pistes ont été écartées, et il vaut de savoir pourquoi :

- *Normaliser à la saisie* — Kesh modifierait ce que l'utilisateur a tapé, et la casse comme les accents resteraient à la merci de la collation : **#295 resterait ouvert**.
- *Pinner la collation + NFC* — traiterait la casse, les accents et la composition, mais **laisserait passer le caractère invisible encastré**, qui est le cas le plus courant.

**D2 — La forme canonique est définie ici, et une seule fois.** Dans l'ordre :

1. retrait de **tout** caractère invisible — prédicat `is_invisible` déjà écrit deux fois dans le dépôt (`kesh-api/src/routes/contacts.rs`, `kesh-qrbill/src/pdf.rs`)
2. normalisation **NFKC** *(et non NFC : `NFKC` replie aussi les formes de compatibilité — chiffres pleine chasse, ligatures — qui sont visuellement le même numéro, et replie les espaces exotiques vers l'espace simple)*
3. `trim()`
4. repli de casse par `to_lowercase()`

⚠️ **Le `trim()` vient en TROISIÈME, et l'ordre a été réfuté par exécution — pas ajusté par goût.** La première rédaction ouvrait par `trim()` : sur `"CLI-1 ‹U+200B›"` — un espace de queue **masqué** par un invisible collé au copier-coller — `trim()` s'arrête au ZWSP (qui n'a pas la propriété `White_Space`), le retrait des invisibles expose ensuite l'espace, et plus rien ne le retire : canonique `"cli-1 "` ≠ `"cli-1"`, soit **le chemin d'attaque #294 réintroduit par l'algorithme lui-même** — et la formulation d'AC1 (« une valeur mixte visible+invisible ») pouvait être satisfaite par un cas central qui ne touche aucun bord. Démontré en Rust réel en passe 1 de validate. Le `trim()` placé après le retrait des invisibles **et** après NFKC nettoie aussi les blancs que NFKC vient de produire. *(Relevé en passe 1, CRITICAL, confirmé par exécution.)*

Une valeur dont la forme canonique est **vide** est traitée comme absente : `client_number` reste stocké tel quel s'il porte du visible, mais si la canonique est vide, **les deux** colonnes valent `NULL`. C'est le prolongement de la garde de vacuité posée en 16-3b.

**D3 — Le prédicat `is_invisible` est factorisé, et le crate d'accueil est `kesh-core`.** *(arbitrage de Guy, 2026-08-14)* Il existe aujourd'hui en **deux exemplaires identiques** dans deux crates, avec une justification écrite qui a été **réfutée** en passe 3 de revue (`kesh-api` dépend bien de `kesh-qrbill`, `Cargo.toml:12`). Cette story en fait **une seule source** : `canonical_key` et `is_invisible` vivent dans **`kesh-core`** (logique pure, zéro I/O — c'est sa définition), et **`kesh-qrbill` ajoute la dépendance**. Aucun cycle possible : `kesh-core` ne dépend que de **`kesh-import`** — et d'aucun autre crate interne —, `kesh-import` ne dépend d'aucun crate interne, donc **rien dans la fermeture transitive de `kesh-core` ne revient vers `kesh-qrbill`**. *(La première rédaction affirmait « kesh-core ne dépend d'aucun crate interne » — prémisse FAUSSE, réfutée au `Cargo.toml` en passe 1 ; la conclusion tenait, pas la phrase. Une décision qui corrige une justification fausse ne peut pas en porter une elle-même.)* L'alternative `kesh-db` aurait fait dépendre `kesh-qrbill` de SQLx pour une fonction pure ; la duplication aurait violé la règle DRY du dépôt — deux `is_invisible` à tenir synchrones est précisément l'état que cette story ferme.

**D4 — La migration est BREAKING, et la procédure P3 s'applique en entier.** Elle remplace la colonne générée `client_number_uniq` et sa contrainte. C'est donc :

- `UPDATE _kesh_version SET kesh_version_min_required = '<version de cette PR>'` **en dernière instruction** de la migration (P2) ;
- **et** le bump de version Cargo de **tous** les crates du workspace dans le **même commit** (P2-bis) — sans quoi le binaire devient plus ancien que sa propre base et `check_downgrade_protection` **refuse le boot** ;
- **et** le gate **runtime** complet, seul à voir ce mode d'échec : les suites `admin_backup_e2e`, `admin_full_import_e2e` et `migrations_fresh_install`.

**D5 — Le backfill ne peut PAS se faire en SQL, et c'est le point dur de cette story.**

MariaDB ne sait ni normaliser en NFKC, ni retirer un jeu ouvert de caractères invisibles. Le remplissage de la colonne canonique pour le parc existant doit donc être fait **en Rust**, et il relève du garde-fou **P7** : toute migration qui écrit des données doit être triée, soit au registre `POST_RESTORE_BACKFILLS`, soit aux `EXEMPT_MIGRATIONS` avec justification écrite.

⚠️ **Et il faut choisir ce qu'on fait des collisions découvertes au backfill.** Deux contacts actifs de la même société peuvent aujourd'hui porter `CLI-1` et `CLI‹ZWSP›-1` : leur forme canonique est **la même**, et l'index unique refusera le second.

**Tranché : la migration REFUSE, en nommant les collisions.** *(arbitrage de Guy, 2026-08-14, entre trois branches : laisser le second à `NULL` en journalisant ; refuser ; renvoyer à la fusion 22-3, en veille.)* Le raisonnement tient au calendrier : Kesh est déployé mais **ne tient pas encore les comptes réels** (jalon « Première clôture d'exercice » ouvert) — refuser ne coûte donc rien aujourd'hui, et c'est la seule branche où **aucune déduplication silencieuse ne passe jamais** : une collision réelle arrête l'upgrade en la nommant, au lieu de survivre en `NULL` journalisé que personne ne lit. C'est la préférence constante du dépôt pour le fail-loud, au moment de l'histoire du projet où elle est la moins chère.

⚠️ **Conséquence assumée** : le jour où une installation porte des collisions, **son binaire à jour refuse de démarrer** jusqu'à correction des données. Le message d'échec DOIT donc nommer les contacts en collision (société, ids, valeurs affichées) — c'est lui, l'outil de réparation. Et le comptage sur la base du NAS **reste un geste de prudence avant l'upgrade de production** — il n'est simplement plus bloquant pour écrire la migration, la branche retenue étant sûre dans les deux cas.

**D6 — Le mécanisme d'exécution du backfill : une fonction Rust idempotente, appelée au BOOT et en fin d'IMPORT.**

⚠️ **Ce mécanisme n'existait nulle part, et la spec le désignait comme « le point dur » sans le résoudre** — démontré en passe 1 : le `MIGRATOR` de sqlx ne rejoue que des fichiers `.sql` (`kesh-db/src/lib.rs:24`), le registre P7 est structurellement SQL (`PostRestoreBackfill { sql: &'static str }`, `post_restore.rs:127-136`), et la séquence de boot (`kesh-api/src/main.rs`, après `MIGRATOR.run`) ne comporte aucune étape Rust. T3 (« migration ») et T4 (« backfill en Rust ») étaient irréconciliables tels qu'écrits.

Le montage retenu, et il découle des décisions déjà prises :

- **La migration SQL est du DDL PUR** : elle ajoute la colonne canonique (nullable, collation explicite, cf. T3), remplace la colonne générée `_uniq` et sa contrainte. Elle n'écrit **aucune donnée** — le détecteur P7 ne la classera même pas, et c'est correct.
- **Une fonction `backfill_client_number_canonical(pool)` dans `kesh-db`**, idempotente : elle charge les contacts dont `client_number IS NOT NULL AND <canonique> IS NULL`, calcule la canonique en Rust (`kesh-core`), **pré-scanne les collisions sur l'ensemble des contacts actifs de chaque société** et, s'il y en a, rend une erreur qui les **nomme toutes** (société, ids, valeurs affichées) sans rien écrire ; sinon elle remplit. Coût quand tout est rempli : une requête qui rend zéro ligne.
- **Appelée au boot**, juste après `MIGRATOR.run` : c'est ici que « la migration refuse » (D5) prend sa forme concrète — **le boot refuse**, avec le rapport de collisions pour message. Idempotente, elle ne coûte rien aux boots suivants.
- **Appelée en fin d'import d'installation** (`admin.rs`, au voisinage de l'appel existant `replay_post_restore_backfills`, `:295`) : un `.keshbackup` antérieur à cette story arrive avec la colonne vide — c'est l'esprit de P7, tenu **sans** entrée au registre SQL. En cas de collision dans le backup, l'import est refusé en `400` avec le même rapport — un backup en collision ne s'installe pas, il se répare d'abord.

⚠️ **Triage P7 formel** : la migration étant DDL pur, ni le registre ni les exemptions ne la concernent — mais la **raison d'être** de P7 (une restauration rouvre le trou que le backfill fermait) est tenue par l'appel dans le chemin d'import. L'écrire dans les Dev Notes de l'implémentation pour que le test de triage n'ait pas à être contourné.

*(Décision de remédiation de la passe 1 de validate — elle matérialise D5 sans rien y changer : « refuser la migration » devient « refuser le boot, et refuser l'import », seuls points où du Rust s'exécute. À confirmer par Guy si la sémantique de refus au boot appelle discussion.)*

## Acceptance Criteria

**AC1 — La forme canonique est une fonction pure, testée à ses frontières.**
Une seule fonction, dans un seul crate, appliquant D2 dans l'ordre. Couverte par des tests de table portant au minimum : casse, accents composés et décomposés (`É` NFC vs `E`+U+0301), formes de compatibilité (chiffres pleine chasse), `U+200B`, `U+FEFF`, `U+2060`, `U+00AD`, espaces de tête et de queue, chaîne vide, valeur intégralement invisible, une valeur **mixte** visible+invisible, **et le cas de BORD qui a réfuté l'ordre initial : un invisible en bord masquant un espace (`"CLI-1 ‹U+200B›"` → `"cli-1"`, pas `"cli-1 "`)**.
*Preuve* : `cargo test` sur le crate d'accueil, et **la mutation jouée** — neutraliser chaque étape de D2 doit faire tomber au moins un test, **et inverser l'ordre (trim d'abord) doit faire tomber le cas de bord** : c'est lui qui épingle l'ordre, pas seulement les étapes.

**AC2 — `is_invisible` n'existe plus qu'en un seul exemplaire.**
*Preuve* : `grep -rn "fn is_invisible" crates/` rend **une** ligne. Les deux appelants d'origine — la normalisation de `contacts.rs` et la garde de vacuité de `pdf.rs` — passent par elle, et leurs tests existants restent verts **à comportement et assertions inchangés** — seuls les chemins d'import (`use`) changent, puisque la fonction déménage. *(« Sans modification » à la lettre était insatisfaisable : déplacer une fonction change au minimum l'import de ses appelants. Relevé en passe 1.)*

**AC3 — La colonne canonique existe et porte l'unicité.**
La contrainte `UNIQUE (company_id, <canonique>_uniq)` remplace celle posée en 16-3b, la colonne `_uniq` restant **générée** sur le patron du dépôt (`CASE WHEN active THEN … ELSE NULL END`), pour que l'archivage continue de **libérer** le numéro.
*Preuve* : les quatre cas de 16-3b restent tenus — deux `NULL` acceptés, doublon entre actifs rejeté, casse rejetée, numéro d'un contact archivé réattribuable — **plus** trois cas neufs : `CLÉ-1` NFD contre NFC rejeté, `CLI-1` contre `CLI‹ZWSP›-1` rejeté, et **deux sociétés distinctes acceptent le même numéro**.

**AC4 — La valeur saisie reste intacte partout où elle est vue.**
Ce que l'utilisateur a tapé est ce qui revient du `GET`, ce qui s'affiche dans la fiche, ce qui s'imprime sur le PDF et sur l'avoir, et ce que la recherche apparie.
*Preuve* : aller-retour `POST` → `GET /contacts/{id}` sur une valeur portant des accents décomposés — la réponse rend **la même séquence d'octets** que l'entrée.
⚠️ **Une exception, et une seule, écrite ici pour qu'AC4 et D2 ne se contredisent pas** : une valeur dont la canonique est **vide** (intégralement invisible, ou vide après retrait) est traitée comme **absente** — les deux colonnes valent `NULL` et le `GET` rend `null`, PAS la séquence saisie. C'est le prolongement délibéré de la garde de vacuité de 16-3b : un « numéro » que personne ne peut voir n'identifie rien. Ce cas reçoit son **test d'intégration** (route ou repository), pas seulement son test de table en AC1. *(Contradiction AC4 ↔ D2 relevée en passe 1.)*

**AC5 — Le comportement ne dépend plus de la collation du serveur.**
*Preuve* — deux jambes, et le mot juste sur chacune :
- **coexistence** : `CLI-É1` et `CLI-E1` sont **acceptés tous deux** — leurs canoniques (`cli-é1`, `cli-e1`) diffèrent, D2 ne replie pas les accents, et c'est voulu : deux clients légitimement distincts par un accent ne fusionnent pas. Ce résultat doit être **le même quelle que soit la collation par défaut du serveur** — c'est le bug #295 (fusion accidentelle sous collation UCA accent-insensible) qui disparaît ;
- **collision** : deux saisies de **même** canonique (`CLI-É1` NFC contre sa forme NFD) sont rejetées `409`, pareillement indépendamment de la collation.
⚠️ **La première rédaction disait « le REJET de `CLI-É1` contre `CLI-E1` »** — un comportement que D2 ne produit jamais, et qui aurait poussé l'implémenteur soit vers un test insatisfaisable, soit vers un repli d'accents hors-D2 fusionnant des clients distincts. Le mot juste est la **coexistence**. *(Relevé en passe 1 par deux lentilles indépendamment.)*
⚠️ Ce test est la contrepartie de celui qui donnait une fausse confiance ; la jambe « collision » doit **échouer** si la canonicalisation est retirée.
**Et la garantie a un support matériel** : la colonne canonique est déclarée avec une **collation binaire explicite** (`utf8mb4_bin`, cf. T3) — l'égalité d'index est l'égalité d'octets, tout ce qui devait être replié l'ayant été en Rust. Sans cette déclaration, une collation UCA du serveur pouvait reproduire #295 **sur la colonne neuve**. *(Relevé en passe 1.)*

**AC6 — Le parc existant est repris, et les collisions sont traitées selon la décision D5, par le mécanisme D6.**
La fonction de backfill (D6) remplit la colonne canonique pour tous les contacts existants, est appelée au boot et en fin d'import, et l'esprit de P7 est tenu par le chemin d'import — la migration, DDL pur, n'écrit rien.
*Preuve* : un test qui seede des contacts, joue la migration, **appelle la fonction de backfill**, et vérifie la canonique de chacun *(« jouer la migration » seul ne remplit RIEN — c'est la fonction Rust qui remplit, cf. D6 ; l'ancienne formulation le laissait croire)* ; un test du cas de collision — la fonction rend le rapport **nommant les contacts** et n'écrit **rien** ; un test d'**idempotence** — second appel sur base remplie, zéro écriture ; et un test du chemin d'**import** — un backup sans la colonne remplie ressort backfillé, un backup en collision est refusé en `400` avec le rapport.

**AC7 — La procédure P3 est appliquée en entier.**
`kesh_version_min_required` bumpé en dernière instruction de la migration, **et** les 10 crates du workspace à la même version dans le même commit, **et** la ligne ajoutée à `docs/migrations-idempotence-audit.md` avec ses cinq compteurs **recomptés depuis le tableau** (P5).
*Preuve* : `migrations_upgrade_path` vert, et les suites runtime `admin_backup_e2e` / `admin_full_import_e2e` / `migrations_fresh_install` vertes.

**AC8 — La documentation dit ce que le logiciel fait.**
Le manuel utilisateur — § *Le numéro de client sur la facture* — décrit la règle réelle : casse, accents, forme de composition et caractères invisibles ne distinguent pas. Le CHANGELOG le dit dans les mots de l'utilisateur. Les deux issues **#294** et **#295** sont fermées par la PR, avec un **mot-clé de fermeture** et non en prose.

## Tasks / Subtasks

- [x] **T1 — Trancher D5** (AC6). **Tranché le 2026-08-14, arbitrage de Guy : la migration REFUSE en nommant les collisions** (cf. D5 pour le raisonnement et la conséquence assumée). Le comptage sur la base du NAS reste un geste de prudence **avant l'upgrade de production**, mais n'est plus bloquant pour écrire la migration — la branche retenue est sûre que la base soit propre ou non.
- [x] **T2 — La fonction canonique** (AC1, AC2). Dans `kesh-core` (D3 tranché), écrire `canonical_key` et `is_invisible`, brancher les deux appelants existants, et écrire les tests de table. **Jouer les mutations** — une étape neutralisée doit faire tomber un test, l'ordre inversé aussi (AC1). ⚠️ Réécrire aussi le **doc-comment de `contacts.rs:291-301`** : il documente encore le motif de duplication d'`is_invisible` dont la justification a été **réfutée** en passe 3 de 16-3b — le laisser serait le résidu documentaire type de la § *Propagation post-patch*.
- [x] **T3 — Migration, DDL PUR** (AC3, AC5, AC7). Ajouter la colonne canonique **nullable avec `COLLATE utf8mb4_bin` explicite** (le support matériel d'AC5 — sans lui, la collation UCA du serveur reproduit #295 sur la colonne neuve), remplacer la colonne générée `_uniq` et sa contrainte, **aucune écriture de données** (le remplissage est D6). Bumper `min_required` en dernière instruction, bumper les 10 crates dans le même commit. Ligne d'audit d'idempotence avec les **cinq** compteurs recomptés. Garde-fou **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspecter **chaque** site.
- [x] **T4 — Backfill, mécanisme D6** (AC6). La fonction idempotente `backfill_client_number_canonical` dans `kesh-db`, appelée **au boot** (après `MIGRATOR.run`, `main.rs`) et **en fin d'import** (`admin.rs`, au voisinage de `replay_post_restore_backfills`). Refus fail-loud avec rapport nommant les collisions, aux deux chemins. La migration étant DDL pur, elle échappe au détecteur P7 **et c'est correct** — consigner dans les Dev Notes d'implémentation que l'esprit de P7 est tenu par l'appel du chemin d'import.
- [x] **T5 — Repository** (AC3, AC4). Écrire la canonique à la création et à la modification. ⚠️ **Le dépôt maintient SEPT listes de colonnes à la main** dans `repositories/contacts.rs` : `COLUMNS`, `FIND_BY_ID_SQL` (qui **duplique** `COLUMNS` mot pour mot), l'`INSERT`, l'`UPDATE`, `contact_snapshot_json`, `is_no_op_change`, et le helper de test `contact_to_update`. Une seule oubliée produit une perte **silencieuse** — la 16-3b l'a payé sur la septième.
- [x] **T6 — Tests repository** (AC3, AC5). Les quatre cas de 16-3b **plus** les trois neufs d'AC3, plus le test d'indépendance à la collation d'AC5.
- [x] **T7 — Route et erreur** (AC3). Le `409 CLIENT_NUMBER_ALREADY_EXISTS` doit continuer de porter son code propre — l'assertion de la **chaîne** vit dans `errors.rs`, seul endroit où le corps de la réponse est lisible.
- [x] **T8 — Recherche** (AC4). La recherche apparie sur la valeur **affichée**, pas sur la canonique — donc en principe **zéro changement de code**. Le livrable est un **test de non-régression** : un contact au numéro portant des accents décomposés est retrouvé par sa graphie saisie, et la recherche ne consulte pas la colonne canonique. ⚠️ Si un changement s'avérait nécessaire : **il y a DEUX branches `LIKE`** dans `push_where_clauses` (`contacts.rs:197` et `:206`) — les deux, ou aucune. *(La première rédaction était une mise en garde sans livrable — une revue ne pouvait pas juger T8 « fait ». Relevé en passe 1.)*
- [x] **T9 — Documentation** (AC8). Manuel utilisateur, CHANGELOG, et fermeture des deux issues **avec mot-clé**.

### Review Findings — passe 2

Aucun finding retenu — **convergence en 2 passes**. Les trois lentilles Haiku concluent indépendamment à zéro défaut au-dessus de LOW ; détail au Change Log, entrée passe 2.

### Review Findings — passe 1

- [x] [Review][Patch] CRITICAL — expansion NFKC/casse : canonique > VARCHAR(50), boot en boucle au message brut [`text.rs`, `contacts.rs:445`, `backfill.rs`]
- [x] [Review][Patch] HIGH — canonique périmée jamais réparée : le backfill devient une réconciliation [`backfill.rs`]
- [x] [Review][Patch] MEDIUM — décompte « 7 mutations » incohérent avec sa ventilation et le compte exécuté (5) [story, sprint-status]
- [x] [Review][Patch] LOW — convention d'audit des backfills documentée en tête de `backfill.rs`

## Dev Notes

### Le rayon d'impact, mesuré et non supposé

`crates/kesh-db/src/repositories/contacts.rs` — les sept listes de T5, la clause de recherche (`client_number LIKE` aux lignes **197** et **206**), et `repositories/reconciliation.rs` qui **réutilise** `contacts::COLUMNS` sans le savoir (`super::contacts::COLUMNS`, l. 203).
`crates/kesh-api/src/routes/contacts.rs` — `normalize_optional` (partagé avec `email`, `phone`, `default_payment_terms` **et l'e-mail de compte** via `users.rs:116`, donc `POST /setup/admin` : tout changement de son comportement sort du périmètre nominal), `validate_common`, `map_contact_error`.
`crates/kesh-qrbill/src/pdf.rs` — `is_invisible` et la garde de vacuité de `build_meta_lines`.

### Trois précédents de colonne générée, à lire avant d'écrire

`20260513000001_reconciliation_rules.sql`, `20260722000001_accounts_role_postable.sql` et `20260810000001_contacts_client_number.sql`. Le patron est établi ; pré-requis **MariaDB ≥ 10.6** pour un `UNIQUE` sur colonne `VIRTUAL`, satisfait partout (10.11 en CI, en release et en compose).

### Ce que la sauvegarde fait toute seule

`non_generated_columns` (`crates/kesh-db/src/backup.rs:100`) filtre `EXTRA NOT LIKE '%GENERATED%'` : la colonne `_uniq` sort de l'export **sans qu'on ait à s'en occuper**. En revanche la colonne canonique, elle, est une colonne **réelle** — elle sera exportée et réimportée, ce qui est le comportement voulu.

### Le piège que cette story doit éviter à son tour

⚠️ **Une migration appliquée ne se modifie plus, pas même un commentaire** (garde-fou **P8**, né de la 16-3b). Le checksum `sqlx` change et le backend refuse de démarrer sur toute base l'ayant appliquée — **y compris la base de dev**. Le gate backend ne le voit pas : `#[sqlx::test]` recrée une base neuve à chaque test. Seul un boot réel le révèle.

### Conventions de test

Mutations **jouées, pas raisonnées** — le dépôt l'exige et l'a payé. Pour chaque garde ajoutée, neutraliser la garde et vérifier que le test tombe, **et sur le bon cas**.
Les affirmations d'absence se vérifient au `grep -nF` avant d'être écrites.
Les décomptes des Change Logs se **recomptent depuis la source**, avec leur **périmètre de mesure déclaré**.

### References

- Issues **#294** (normalisation NFC + invisible encastré) et **#295** (collation de `contacts`).
- Story **16-3b** — `16-3b-numero-client-pdf.md`, § *Dette technique* et Change Log des quatre passes de revue.
- Story **22-2** (#301) — la prévention à la saisie, prioritaire : ce qui rend la réparation inutile.
- Story **22-3** (#300) — la fusion de doublons, **en veille**.
- Rétrospective **Epic 16** — `epic-16-retro-2026-08-11.md`, action **A8**.
- `CLAUDE.md` — § *Migration breaking policy* (P2, P2-bis, P3, P5, P6, P7, P8), § *Propagation post-patch*, § *Recompter ses propres comptes rendus*.

## Périmètre — écrit pour qu'une passe de revue ne le rouvre pas

La canonicalisation ne concerne que **`client_number`**. Le numéro **IDE** porte sa propre validation (format `CHE-###.###.###`, jeu de caractères fermé — aucun des trois chemins d'attaque ne s'y applique) et n'en relève pas. `email` et `phone` n'ont **aucun index unique** : aucune garantie d'unicité annoncée, donc rien que cette story doive tenir. Étendre la canonique à d'autres champs serait une story neuve, pas un élargissement de celle-ci.

**`contact_persons`** — l'issue #295 propose de « pinner la table entière — et `contact_persons` avec elle » : hors périmètre **parce qu'aucune unicité n'y est portée** — pinner sa collation serait de l'hygiène sans garantie à tenir, à reprendre si une contrainte y naît un jour. *(Écrit pour qu'une passe ne le rouvre pas en lisant #295.)*

**Réactivation d'un contact archivé** — le scénario « A archivé libère `CLI-1`, B le prend, on réactive A » est **inatteignable aujourd'hui** : `update` rejette un contact archivé (`repositories/contacts.rs:488`, `AND active = TRUE`, test `test_update_rejects_archived_contact`), et aucune route ne réactive. Si une réactivation naît un jour, elle devra passer par le même mapping `409 CLIENT_NUMBER_ALREADY_EXISTS` — c'est écrit ici pour elle. *(Relevé en passe 1, vérifié au code.)*

*(Les « Questions ouvertes » qui occupaient cette section ont toutes été résolues le 2026-08-14 : D5 et D3 par arbitrage de Guy — inscrits dans leurs décisions respectives —, le périmètre par le présent paragraphe.)*

## Dev Agent Record

### Ce qui a été implémenté (2026-08-14, dev Fable — D6 confirmée par Guy au lancement)

- **T2** — `kesh_core::text` : `is_invisible` (déménagé, sémantique 16-3b intacte) + `canonical_key` (D2, ordre invisibles → NFKC → trim → casse). Les deux appelants rebranchés (`contacts.rs`, `pdf.rs` — qui perd sa copie et son doc-comment réfuté), `kesh-qrbill` prend la dépendance `kesh-core` (D3, second arbitrage de Guy : la mention « standalone, publishable » du manifest — propriété jamais exercée — est retirée plutôt que de dupliquer). Dépendance `unicode-normalization` ajoutée à `kesh-core` — déjà présente dans l'arbre (kesh-api), pas une dépendance nouvelle au sens du workflow.
- **T3** — migration `20260814000001`, DDL pur : colonne canonique `VARCHAR(50) COLLATE utf8mb4_bin`, `_uniq` régénérée sur la canonique, contrainte au même nom, bump `min_required = '0.10.0'` en dernière instruction. **P2-bis : les 10 crates à `0.10.0`** dans le même commit. P5 : ligne d'audit + les cinq compteurs recomptés (61 = 61 lignes de tableau, 5+56+0). P6 : les deux sites inspectés — `migrations_upgrade_path.rs` bumpé (61/27, frontière 34 constante, **7 sites de valeurs périmées corrigés au value-grep** dans le même geste), `accounts_role_backfill` version-résolu, rien à faire.
- **T4** — `kesh_db::backfill::backfill_client_number_canonical` : pré-scan des collisions sur les actifs (rapport nominatif, invisibles **échappés** — on ne répare pas ce qu'on ne voit pas), zéro écriture en cas de refus, remplissage des seuls `NULL` (idempotence), vacuité D2 appliquée au parc. Appelée au **boot** (`main.rs` 4-bis, `process::exit(1)` avec le rapport) et en fin d'**import** (`admin.rs` 5-ter, dans la transaction de restore). P7 : entrée `EXEMPT_MIGRATIONS` pour le bump `_kesh_version` (même triage que `20260714000002`).
- **T5** — `client_number_columns(raw)` dans le repository : couple (valeur intacte, canonique) calculé au seul endroit qui écrit, vacuité → double `NULL`. INSERT et UPDATE portent la colonne.
- **T7** — la contrainte garde son nom → `map_contact_error` inchangé, zéro test modifié. L'import gagne un variant dédié **`400 IMPORT_CLIENT_NUMBER_COLLISION`** (`details.report`) + clé i18n sur les 4 locales — le 500 générique `ADMIN_FULL_IMPORT_FAILED` aurait rendu le rapport invisible à l'exploitant, en contradiction avec AC6.
- **T9** — manuel utilisateur (§ numéro de client réécrit : identiques à l'œil = même numéro, un accent distingue, la saisie reste intacte) + PDF régénéré (vérifié au `pdftotext`) ; CHANGELOG `[Unreleased] / ### Corrigé` avec l'avertissement de refus au boot.

### Écarts à la spec — trois, tous documentés au moment du geste

1. **Le retrait de l'étape 1 de D2 porte sur le sous-ensemble LARGEUR NULLE d'`is_invisible`, pas sur le prédicat entier.** `is_invisible` inclut `is_whitespace()` (sémantique de vacuité 16-3b) : l'utiliser tel quel pour le retrait fusionnerait `"CLI 1"` et `"CLI1"`, visuellement distincts — l'inverse du défaut que la story ferme. D'où `is_zero_width = is_invisible && !is_whitespace`, les blancs restant traités par NFKC (repli des espaces exotiques) et `trim()` (bords). Cas `("CLI 1", "CLI1")` épinglé DISTINCT dans les tests de table.
2. **Le backfill lit l'EXISTENCE de la canonique, pas sa valeur** (`IS NOT NULL`) : la collation `utf8mb4_bin` fait exposer la colonne comme VARBINARY par sqlx, et seule l'existence compte (idempotence) — la valeur est recalculée en Rust. Les tests qui lisent la valeur passent par `CAST(... AS CHAR)`.
3. **AC6 « refusé en 400 »** exigeait le variant d'erreur dédié ci-dessus (T7) — le chemin d'erreur générique d'import rend un 500 au corps générique.

### Mutations jouées — 5 au dev, 5 rougissements, chacune isolée puis restaurée

M1-M4 (chaque étape de D2 neutralisée) → 1 à 3 tests de table rouges chacune ; M-AC5 (canonicalisation remplacée par `trim().to_lowercase()`) → les deux tests de collision (composition, ZWSP) rouges. **Cinq mutations EXÉCUTÉES** — l'ordre de D2, lui, n'est pas une mutation jouée : il est épinglé EN PERMANENCE par le test dédié `the_old_order_is_really_wrong` et le cas de bord de la table. *(La première rédaction déclarait « 7 » : un total incohérent avec sa propre ventilation ET avec le compte exécuté — la § Recompter prise en défaut sur ce Change Log même, relevée en passe 1 par l'Acceptance Auditor. Périmètre : 5 au commit de dev ; la passe 1 de revue en a joué 2 de plus sur ses propres patches, M-a garde de route et M-b réconciliation.)* ⚠️ Une première salve de mutations s'était ACCUMULÉE (fichier neuf, `git checkout --` silencieusement sans effet) — détecté, rejouée proprement avec sauvegarde/restauration par copie et `diff` de contrôle final.

### Tests neufs — périmètre : commit de dev, recomptés depuis la source

`kesh-core/src/text.rs` : 4 (table, vacuité, ordre, sémantique du prédicat). `repositories/contacts.rs` : 7 (composition, ZWSP, cross-company, accents distincts, collation binaire des DEUX colonnes via information_schema, vacuité repository, recherche T8). `tests/client_number_canonical_backfill.rs` : 5 (nominal, refus nominatif sans écriture, idempotence, vacuité legacy, archivés hors collision). `admin_full_import_e2e.rs` : 2 (backfill via import, refus 400 + rollback). **Total : 18.** Les 4 cas de 16-3b restent verts sans modification de leurs assertions (le doc-comment du test de casse, dont le MÉCANISME a changé, est mis à jour).

### Gates exécutés

| Gate | Résultat |
|---|---|
| fmt + clippy workspace `-D warnings` | verts *(un premier run a attrapé un `explicit_auto_deref` réel — et mon invocation à pipe masquait l'exit code, le piège documenté par la 22-4a : relancé sans pipe, `EXIT=` explicite)* |
| tests ciblés — text, contacts repo, backfill, import e2e, upgrade_path | verts (4 + 15 + 2, cf. ci-dessus) |
| frontend `check` / `lint-i18n` / `test:unit` / `build` | 0 erreur / PASS / **512/512** / ok |
| **gate backend complet profil `ci` — RUN DE CONVERGENCE** | **2204/2204, 0 skip-échec, EXIT=0** (69 min), au dernier commit de la boucle de revue |
| **suite E2E Playwright rejouée** | **180 ✓ / 39 ✘ / 19 ignorés** (13,7 min) — les 39 échecs relèvent TOUS des familles structurelles #287 (mêmes têtes que la référence du 2026-08-13 : bank-import 6, bank-csv 4, reminders/onboarding/invoice-send-email/fiscal-years/bank-import-confirms 3 chacune) ; **les deux specs `contact-client-number` sont VERTES**, dont « le numéro survit à la modification d'un champ sans rapport » — la surface exacte de l'UPDATE modifié ; grep des jetons du périmètre (client_number, canonical, backfill) sur tout le journal : seules ces deux lignes vertes. Différentiel ARGUMENTÉ contre la référence 181/38/19, pas mesuré sur un worktree main — même réserve déclarée que la 22-4a. **Et le boot lui-même est prouvé** : le backend 0.10.0 a migré et backfillé `kesh_e2e` (base persistante réelle) sans refus — le chemin P8 et le chemin D6-boot exercés en vrai |

### File List

- `crates/kesh-core/src/text.rs` — **neuf** (module + 4 tests)
- `crates/kesh-core/src/lib.rs`, `crates/kesh-core/Cargo.toml` — module + dépendance `unicode-normalization`
- `crates/kesh-qrbill/Cargo.toml` (dépendance kesh-core, description amendée), `crates/kesh-qrbill/src/pdf.rs` (copie supprimée)
- `crates/kesh-api/src/routes/contacts.rs` — import du prédicat, doc-comment réfuté réécrit
- `crates/kesh-db/migrations/20260814000001_contacts_client_number_canonical.sql` — **neuve**
- `crates/kesh-db/src/backfill.rs` — **neuf** ; `crates/kesh-db/src/lib.rs` (module)
- `crates/kesh-db/src/post_restore.rs` — entrée EXEMPT_MIGRATIONS
- `crates/kesh-db/src/repositories/contacts.rs` — `client_number_columns`, INSERT/UPDATE, 7 tests, doc-comment du test de casse
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — 61/27 + 7 sites de valeurs
- `crates/kesh-db/tests/client_number_canonical_backfill.rs` — **neuf** (5 tests)
- `crates/kesh-api/src/main.rs` — appel boot 4-bis
- `crates/kesh-api/src/routes/admin.rs` — appel import 5-ter
- `crates/kesh-api/src/errors.rs` — variant `ImportClientNumberCollision`
- `crates/kesh-api/tests/admin_full_import_e2e.rs` — 2 tests
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — clé `error-import-client-number-collision`
- `crates/*/Cargo.toml` ×10 — version `0.10.0` (P2-bis)
- `docs/migrations-idempotence-audit.md` — ligne + 5 compteurs
- `docs/manual/fr/user-manual.tex` + `.pdf` (+ `admin-manual.pdf`/`marketing-brochure.pdf` régénérés par `make fr`)
- `CHANGELOG.md` — `[Unreleased] / ### Corrigé`

## Change Log

**2026-08-15 — `bmad-code-review`, PASSE 2 (Haiku ×3, contextes frais, diff aplati unique). CONVERGENCE : 0 finding > LOW retenu, critère d'arrêt ATTEINT en 2 passes.**

| Lentille | retenu après ground-truth |
|---|---|
| Blind Hunter (diff seul) | **0** — ses sections « CRITICAL/HIGH » sont des auto-vérifications qui concluent chacune « conforme, présent, testé », y compris sa propre question sur le rejeu des mutations post-incident, qu'il réfute lui-même |
| Edge Case Hunter (diff + code) | **0** — les six bords ciblés des patches de passe 1 vérifiés sains : pré-scan refuse en bloc avant toute écriture, `canonical_key` unique aux trois sites de calcul, actif/archivé cohérent avec la contrainte, idempotence de la vacuité par le WHERE, versions cohérentes aux trois niveaux |
| Acceptance Auditor (spec + CLAUDE.md + issues) | **0** — les 4 patches de passe 1 vérifiés présents au grep, compteurs P5 recomptés (61 = 5+56+0), AC1-AC8 rejouées, `\b0.7.0\b` sans assertion résiduelle, périmètre 4 crates sous le seuil de split |

**Trend : passe 1 `1/1/1/1` → passe 2 `0/0/0/0`. Modèles du cycle : dev Fable → P1 Sonnet ×3 → P2 Haiku ×3.** Mutations du cycle : 5 au dev + 2 en passe 1 = 7, 7 rougissements, périmètres déclarés.

**Reste avant la PR** : le gate backend complet de CONVERGENCE (celui qui fera foi d'un run entièrement vert — lancé au commit de cette entrée), la suite E2E rejouée (le code de production a changé : boot, import, route contacts), puis la PR unique portant `closes #294, closes #295`.

---

**2026-08-14 — `bmad-code-review`, PASSE 1 (Sonnet ×3, contextes frais — l'implémenteur était Fable).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Blind Hunter (diff aplati seul) | 0 | 1 | 0 | 1 |
| Edge Case Hunter (diff + code, exécutions réelles) | 1 | 1 | 0 | 0 |
| Acceptance Auditor (+ spec, CLAUDE.md, issues) | 0 | 0 | 1 | 1 |
| **dédupliqué** | **1** | **1** | **1** | **1** |

Déduplication : l'expansion NFKC, vue par les TROIS lentilles (BH-HIGH par raisonnement, ECH-CRIT par exécution Rust + MariaDB réels, AA-LOW) → **CRITICAL**, l'exécution faisant foi.

### Le CRITICAL — la canonique peut être PLUS LONGUE que la colonne qui la reçoit

`canonical_key` ALLONGE : NFKC décompose les ligatures (`ﬁ` → `fi`), `to_lowercase` étend `İ` en `i`+U+0307 — **50 caractères saisis peuvent en canoniser 100**, au-delà du `VARCHAR(50)`. Prouvé par exécution (Rust réel + INSERT MariaDB 10.11 sous le `sql_mode` du pool : `ERROR 1406`). Trois chemins d'échec, tous opaques : `POST /contacts` → 400 générique au message trompeur ; parc legacy au boot → **boot en boucle sur une erreur SQL brute ne nommant AUCUNE fiche**, en contradiction frontale avec la promesse D5 ; import → 500 générique. **Fermé à trois étages** : constante unique `kesh_core::text::CLIENT_NUMBER_MAX_CHARS` (la route s'y aligne) ; garde de route au message qui nomme la cause réelle ; catégorie **`overlong` dans le rapport de refus du backfill** — la fiche est nommée, aux deux chemins (boot et 400 d'import). Propriété d'expansion épinglée par un test dédié dans `text.rs`. **Mutation M-a jouée** : garde retirée → rouge.

### Le HIGH — le backfill ne réparait jamais une canonique périmée

La contrainte d'unicité ne compare que la canonique **stockée** ; un `client_number` modifié par SQL direct (cas explicitement anticipé par le dépôt) laissait une canonique périmée — **#294 rouvrable en silence**, pour des mois sur un NAS sans redémarrage. **Le backfill devient une RÉCONCILIATION** : canonique recalculée pour chaque ligne, divergence réparée, détection des collisions toujours par recalcul (les stockées mensongères sont ignorées). Coût inchangé — la fonction chargeait déjà tout le parc. Trois tests neufs : réparation, idempotence après réparation, collision masquée par des stockées divergentes. **Mutation M-b jouée** : réconciliation ramenée au remplissage → rouge.

### Le MEDIUM — mon propre décompte de mutations, faux des deux côtés

« 7 mutations jouées » : incohérent avec sa ventilation (6 items) ET avec le compte exécuté (**5** — l'ordre de D2 est épinglé par un test permanent, pas par une mutation jouée). La § *Recompter ses propres comptes rendus* prise en défaut sur le Change Log de la story qui la cite en Dev Notes. Corrigé aux deux sites (story, sprint-status), périmètre déclaré.

### Le LOW — l'audit du NULLing de vacuité

Le backfill ramène à `NULL` un `client_number` intégralement invisible sans écrire d'`audit_log` — **conforme à la convention des backfills P7**, qui n'auditent pas (maintenance de schéma, aucun acteur au boot). Documenté en tête de `backfill.rs` pour que la question ne se repose pas.

### Incident d'outillage, consigné parce qu'il a coûté deux fois

Un `git checkout --` de restauration post-mutation a **écrasé les patches non commités** de `contacts.rs` (le fichier étant suivi, la commande a réussi — et le fallback prévu ne s'est donc pas exécuté). Les trois édits ont été réappliqués et re-vérifiés verts. C'est le second incident de restauration du cycle (le premier : mutations accumulées sur fichier non suivi). **Règle tirée : les mutations se jouent avec sauvegarde/restauration PAR COPIE et `diff` de contrôle, jamais via git sur un arbre non commité** — appliquée à M-b.

**Gate ciblé après patches : 12/12** (backfill étendu + canonical + garde de route), fmt et clippy workspace verts. **Gate complet joué ENTIER après le commit de cette passe : 2201/2204** — les 3 seuls échecs étaient les assertions figées à `0.7.0` de `migrations_fresh_install`/`migrations_upgrade_path`, **la famille exacte que P2-bis annonce comme seule à voir un bump `min_required`** (invisible en statique, révélé au runtime). Les 4 sites corrigés au value-grep (`\b0\.7\.0\b`), rejoués verts 3/3. Le gate complet de convergence sera rejoué en fin de boucle, avant le push — c'est lui qui fera foi d'un run entièrement vert. Trend : passe 1 = `1/1/1/1` → **passe 2 requise** (rotation : Haiku).

---

**2026-08-14 — `bmad-create-story validate`, PASSE 1 (Sonnet ×3, contextes frais : lentille aveugle, ground-truth, audit de conformité).**

| Lentille | CRIT | HIGH | MED | LOW |
|---|---|---|---|---|
| Aveugle (spec seule) | 2 | 1 | 3 | 2 |
| Ground-truth (spec vs code) | 0 | 0 | 1 | 2 |
| Audit (checklist + CLAUDE.md + issues) | 1 | 1 | 1 | 2 |
| **dédupliqué** | **2** | **1** | **5** | **4** |

Déduplication : le mécanisme du backfill (aveugle-HIGH + audit-CRIT → **CRIT**) ; AC5 (aveugle-CRIT + audit-HIGH → **HIGH**, la remédiation étant une reformulation). Un écarté : « T1 cochée sous Status backlog » — c'est la convention du dépôt pendant un validate (précédent 22-4).

- **CRIT 1 — l'ordre de D2 réintroduisait #294** : `trim()` en tête laisse un espace de queue masqué par un invisible (`"CLI-1 ‹ZWSP›"` → `"cli-1 "`). **Confirmé par exécution Rust réelle** avant patch. Réordonné (invisibles → NFKC → trim → casse), cas de bord ajouté à AC1 avec mutation d'ordre.
- **CRIT 2 — le mécanisme du backfill Rust n'existait nulle part** : `MIGRATOR` ne rejoue que du SQL, le registre P7 est `sql: &'static str`, aucun hook de boot — T3 et T4 étaient irréconciliables. **D6 créée** : fonction idempotente, appelée au boot et en fin d'import, refus fail-loud nommant les collisions aux deux chemins ; migration DDL pur ; esprit de P7 tenu par le chemin d'import. *(Matérialise D5 sans le changer — signalé à Guy pour confirmation.)*
- **HIGH — AC5 prescrivait le « rejet » de `CLI-É1` contre `CLI-E1`**, comportement que D2 ne produit jamais : réécrite en deux jambes (coexistence / collision), toutes deux indépendantes de la collation.
- **5 MED** : `COLLATE utf8mb4_bin` explicite sur la colonne neuve (sans lui, #295 renaissait sur la colonne canonique — T3) · contradiction AC4 ↔ D2 sur la valeur intégralement invisible (exception écrite + test d'intégration) · « kesh-core ne dépend d'aucun crate interne » réfuté au Cargo.toml (dépend de kesh-import ; conclusion sauve, prémisse corrigée) · AC2 « sans modification » insatisfaisable (reformulée : comportement et assertions inchangés, imports exceptés) · réactivation d'un archivé — vérifiée **inatteignable** (`active = TRUE` dans l'UPDATE, test dédié) → note de périmètre au lieu d'une AC.
- **4 LOW** : T8 sans livrable (test de non-régression défini) · doc-comment réfuté de `contacts.rs:291-301` nommé en T2 · `contact_persons` écrit au périmètre · lignes `LIKE` exactes (197/206).

**Ce que le ground-truth a vérifié CONFORME, et c'est substantiel** : les sept listes de colonnes (exactement sept, lignes citées) · les deux branches `LIKE` · `contacts`/`contact_persons` seules tables sans COLLATE · les trois seules migrations `GENERATED ALWAYS` du dépôt sont bien les trois citées · `non_generated_columns` à `backup.rs:100` · `normalize_optional` partagé via `users.rs:116` · MariaDB 10.11 partout (≥ 10.6 requis) · les 10 crates à 0.9.0 · la citation du manuel mot pour mot · P2/P2-bis/P5/P6/P8 correctement appliqués · splitting : 4 crates touchés, sous le seuil de 5 · #294/#295 couvertes en entier.

**Patches appliqués, symptômes grepés (zéro résidu), prochaine passe : Haiku, contexte frais.**

**2026-08-14 — `bmad-create-story validate`, PASSE 2 (Haiku ×3, contextes frais). CONVERGENCE : 0 finding > LOW retenu, critère d'arrêt ATTEINT en 2 passes.**

- **Aveugle** : zéro finding — cohérence D2 ↔ AC1 ↔ exception de vacuité, D5 ↔ D6 ↔ T4, AC5 ↔ T3 vérifiée point par point, aucune régression des patches de passe 1.
- **Ground-truth** : toutes les affirmations neuves des patches confirmées au code réel (MIGRATOR, `PostRestoreBackfill { sql }`, absence de hook au boot, `admin.rs:295`, chaîne de dépendances kesh-core → kesh-import → rien, lignes LIKE, `AND active = TRUE`, les trois migrations `GENERATED ALWAYS`) ; **l'ordre neuf de D2 rejoué en Rust réel, aucun contre-exemple trouvé** (NFD/NFC, invisibles, pleine chasse, cas vides). Deux « CRITICAL » écartés comme auto-réfutés : ils constataient l'absence de `canonical_key` et du hook de boot — le code que la story doit écrire, état normal d'une spec en backlog (la lentille le notait elle-même : « tâche non démarrée »).
- **Audit** : Change Log de passe 1 **recompté exact** contre sa propre ventilation ; D6 conforme à P7 **vérifié dans le code du test de triage** (`split_statements` + `writes_data` : une migration DDL pur passe sans entrée) ; les 4 suites citées par AC7 existent ; toutes les AC exécutables et fail-loud ; #294/#295 couvertes en entier. Trois notes LOW sans action (assertions P2-bis à détailler au dev, § du manuel à créer en T9, fichier de test d'AC3 au choix du dev).

**Trend : `2/1/5/4` → `0/0/0/0` retenu. Modèles : rédaction Opus (2026-08-12) + arbitrages Guy → P1 Sonnet ×3 → P2 Haiku ×3.** Statut → `ready-for-dev`. Reste ouvert pour Guy : la note de D6 (« refuser la migration » prend la forme « refuser le boot / refuser l'import » — seuls points où du Rust s'exécute) est signalée, à lever d'un mot au lancement du dev.
