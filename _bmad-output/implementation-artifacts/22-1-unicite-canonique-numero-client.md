# Story 22.1 : Unicité canonique du numéro de client

## Status

backlog

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

1. `trim()`
2. retrait de **tout** caractère invisible — prédicat `is_invisible` déjà écrit deux fois dans le dépôt (`kesh-api/src/routes/contacts.rs`, `kesh-qrbill/src/pdf.rs`)
3. normalisation **NFKC** *(et non NFC : `NFKC` replie aussi les formes de compatibilité — chiffres pleine chasse, ligatures — qui sont visuellement le même numéro)*
4. repli de casse par `to_lowercase()`

Une valeur dont la forme canonique est **vide** est traitée comme absente : `client_number` reste stocké tel quel s'il porte du visible, mais si la canonique est vide, **les deux** colonnes valent `NULL`. C'est le prolongement de la garde de vacuité posée en 16-3b.

**D3 — Le prédicat `is_invisible` est factorisé, et le crate d'accueil est `kesh-core`.** *(arbitrage de Guy, 2026-08-14)* Il existe aujourd'hui en **deux exemplaires identiques** dans deux crates, avec une justification écrite qui a été **réfutée** en passe 3 de revue (`kesh-api` dépend bien de `kesh-qrbill`, `Cargo.toml:12`). Cette story en fait **une seule source** : `canonical_key` et `is_invisible` vivent dans **`kesh-core`** (logique pure, zéro I/O — c'est sa définition), et **`kesh-qrbill` ajoute la dépendance**. Aucun cycle possible : `kesh-core` ne dépend d'aucun crate interne. L'alternative `kesh-db` aurait fait dépendre `kesh-qrbill` de SQLx pour une fonction pure ; la duplication aurait violé la règle DRY du dépôt — deux `is_invisible` à tenir synchrones est précisément l'état que cette story ferme.

**D4 — La migration est BREAKING, et la procédure P3 s'applique en entier.** Elle remplace la colonne générée `client_number_uniq` et sa contrainte. C'est donc :

- `UPDATE _kesh_version SET kesh_version_min_required = '<version de cette PR>'` **en dernière instruction** de la migration (P2) ;
- **et** le bump de version Cargo de **tous** les crates du workspace dans le **même commit** (P2-bis) — sans quoi le binaire devient plus ancien que sa propre base et `check_downgrade_protection` **refuse le boot** ;
- **et** le gate **runtime** complet, seul à voir ce mode d'échec : les suites `admin_backup_e2e`, `admin_full_import_e2e` et `migrations_fresh_install`.

**D5 — Le backfill ne peut PAS se faire en SQL, et c'est le point dur de cette story.**

MariaDB ne sait ni normaliser en NFKC, ni retirer un jeu ouvert de caractères invisibles. Le remplissage de la colonne canonique pour le parc existant doit donc être fait **en Rust**, et il relève du garde-fou **P7** : toute migration qui écrit des données doit être triée, soit au registre `POST_RESTORE_BACKFILLS`, soit aux `EXEMPT_MIGRATIONS` avec justification écrite.

⚠️ **Et il faut choisir ce qu'on fait des collisions découvertes au backfill.** Deux contacts actifs de la même société peuvent aujourd'hui porter `CLI-1` et `CLI‹ZWSP›-1` : leur forme canonique est **la même**, et l'index unique refusera le second.

**Tranché : la migration REFUSE, en nommant les collisions.** *(arbitrage de Guy, 2026-08-14, entre trois branches : laisser le second à `NULL` en journalisant ; refuser ; renvoyer à la fusion 22-3, en veille.)* Le raisonnement tient au calendrier : Kesh est déployé mais **ne tient pas encore les comptes réels** (jalon « Première clôture d'exercice » ouvert) — refuser ne coûte donc rien aujourd'hui, et c'est la seule branche où **aucune déduplication silencieuse ne passe jamais** : une collision réelle arrête l'upgrade en la nommant, au lieu de survivre en `NULL` journalisé que personne ne lit. C'est la préférence constante du dépôt pour le fail-loud, au moment de l'histoire du projet où elle est la moins chère.

⚠️ **Conséquence assumée** : le jour où une installation porte des collisions, **son binaire à jour refuse de démarrer** jusqu'à correction des données. Le message d'échec DOIT donc nommer les contacts en collision (société, ids, valeurs affichées) — c'est lui, l'outil de réparation. Et le comptage sur la base du NAS **reste un geste de prudence avant l'upgrade de production** — il n'est simplement plus bloquant pour écrire la migration, la branche retenue étant sûre dans les deux cas.

## Acceptance Criteria

**AC1 — La forme canonique est une fonction pure, testée à ses frontières.**
Une seule fonction, dans un seul crate, appliquant D2 dans l'ordre. Couverte par des tests de table portant au minimum : casse, accents composés et décomposés (`É` NFC vs `E`+U+0301), formes de compatibilité (chiffres pleine chasse), `U+200B`, `U+FEFF`, `U+2060`, `U+00AD`, espaces de tête et de queue, chaîne vide, valeur intégralement invisible, et une valeur **mixte** visible+invisible.
*Preuve* : `cargo test` sur le crate d'accueil, et **la mutation jouée** — neutraliser chaque étape de D2 doit faire tomber au moins un test.

**AC2 — `is_invisible` n'existe plus qu'en un seul exemplaire.**
*Preuve* : `grep -rn "fn is_invisible" crates/` rend **une** ligne. Les deux appelants d'origine — la normalisation de `contacts.rs` et la garde de vacuité de `pdf.rs` — passent par elle, et leurs tests existants restent verts sans modification.

**AC3 — La colonne canonique existe et porte l'unicité.**
La contrainte `UNIQUE (company_id, <canonique>_uniq)` remplace celle posée en 16-3b, la colonne `_uniq` restant **générée** sur le patron du dépôt (`CASE WHEN active THEN … ELSE NULL END`), pour que l'archivage continue de **libérer** le numéro.
*Preuve* : les quatre cas de 16-3b restent tenus — deux `NULL` acceptés, doublon entre actifs rejeté, casse rejetée, numéro d'un contact archivé réattribuable — **plus** trois cas neufs : `CLÉ-1` NFD contre NFC rejeté, `CLI-1` contre `CLI‹ZWSP›-1` rejeté, et **deux sociétés distinctes acceptent le même numéro**.

**AC4 — La valeur saisie reste intacte partout où elle est vue.**
Ce que l'utilisateur a tapé est ce qui revient du `GET`, ce qui s'affiche dans la fiche, ce qui s'imprime sur le PDF et sur l'avoir, et ce que la recherche apparie.
*Preuve* : aller-retour `POST` → `GET /contacts/{id}` sur une valeur portant des accents décomposés — la réponse rend **la même séquence d'octets** que l'entrée.

**AC5 — Le comportement ne dépend plus de la collation du serveur.**
*Preuve* : un test qui pose la collation de la colonne canonique **explicitement** et vérifie que le rejet de `CLI-É1` contre `CLI-E1` **ne dépend pas** d'elle — la canonique ayant déjà replié la casse, l'égalité binaire suffit.
⚠️ Ce test est la contrepartie de celui qui donnait une fausse confiance ; il doit **échouer** si la canonicalisation est retirée.

**AC6 — Le parc existant est repris, et les collisions sont traitées selon la décision D5.**
Le backfill remplit la colonne canonique pour tous les contacts existants, et son triage P7 est fait — registre ou exemption justifiée.
*Preuve* : un test qui seede des contacts **avant** la migration, la joue, et vérifie la canonique de chacun ; plus un test du cas de collision, conforme à la branche retenue en D5.

**AC7 — La procédure P3 est appliquée en entier.**
`kesh_version_min_required` bumpé en dernière instruction de la migration, **et** les 10 crates du workspace à la même version dans le même commit, **et** la ligne ajoutée à `docs/migrations-idempotence-audit.md` avec ses cinq compteurs **recomptés depuis le tableau** (P5).
*Preuve* : `migrations_upgrade_path` vert, et les suites runtime `admin_backup_e2e` / `admin_full_import_e2e` / `migrations_fresh_install` vertes.

**AC8 — La documentation dit ce que le logiciel fait.**
Le manuel utilisateur — § *Le numéro de client sur la facture* — décrit la règle réelle : casse, accents, forme de composition et caractères invisibles ne distinguent pas. Le CHANGELOG le dit dans les mots de l'utilisateur. Les deux issues **#294** et **#295** sont fermées par la PR, avec un **mot-clé de fermeture** et non en prose.

## Tasks / Subtasks

- [x] **T1 — Trancher D5** (AC6). **Tranché le 2026-08-14, arbitrage de Guy : la migration REFUSE en nommant les collisions** (cf. D5 pour le raisonnement et la conséquence assumée). Le comptage sur la base du NAS reste un geste de prudence **avant l'upgrade de production**, mais n'est plus bloquant pour écrire la migration — la branche retenue est sûre que la base soit propre ou non.
- [ ] **T2 — La fonction canonique** (AC1, AC2). Choisir le crate d'accueil (cf. D3), y écrire `canonical_key` et `is_invisible`, brancher les deux appelants existants, et écrire les tests de table. **Jouer les mutations** — une étape neutralisée doit faire tomber un test.
- [ ] **T3 — Migration** (AC3, AC7). Remplacer la colonne générée et la contrainte, ajouter la colonne canonique, bumper `min_required` en dernière instruction, bumper les 10 crates dans le même commit. Ligne d'audit d'idempotence avec les **cinq** compteurs recomptés. Garde-fou **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` et inspecter **chaque** site.
- [ ] **T4 — Backfill** (AC6). En Rust, selon la branche retenue en T1. Triage **P7** : registre `POST_RESTORE_BACKFILLS` ou exemption **avec justification écrite**. Si la justification invoque « hors fenêtre », elle **doit** commencer par la chaîne `Hors fenêtre` — sinon elle échappe au contrôle.
- [ ] **T5 — Repository** (AC3, AC4). Écrire la canonique à la création et à la modification. ⚠️ **Le dépôt maintient SEPT listes de colonnes à la main** dans `repositories/contacts.rs` : `COLUMNS`, `FIND_BY_ID_SQL` (qui **duplique** `COLUMNS` mot pour mot), l'`INSERT`, l'`UPDATE`, `contact_snapshot_json`, `is_no_op_change`, et le helper de test `contact_to_update`. Une seule oubliée produit une perte **silencieuse** — la 16-3b l'a payé sur la septième.
- [ ] **T6 — Tests repository** (AC3, AC5). Les quatre cas de 16-3b **plus** les trois neufs d'AC3, plus le test d'indépendance à la collation d'AC5.
- [ ] **T7 — Route et erreur** (AC3). Le `409 CLIENT_NUMBER_ALREADY_EXISTS` doit continuer de porter son code propre — l'assertion de la **chaîne** vit dans `errors.rs`, seul endroit où le corps de la réponse est lisible.
- [ ] **T8 — Recherche** (AC4). La recherche apparie sur la valeur **affichée**, pas sur la canonique. ⚠️ **Il y a DEUX branches `LIKE`** dans `push_where_clauses` — celle du terme échappé vide et celle du cas courant : **les deux, ou aucune**.
- [ ] **T9 — Documentation** (AC8). Manuel utilisateur, CHANGELOG, et fermeture des deux issues **avec mot-clé**.

## Dev Notes

### Le rayon d'impact, mesuré et non supposé

`crates/kesh-db/src/repositories/contacts.rs` — les sept listes de T5, la clause de recherche (l. ~195 et ~206), et `repositories/reconciliation.rs` qui **réutilise** `contacts::COLUMNS` sans le savoir.
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

*(Les « Questions ouvertes » qui occupaient cette section ont toutes été résolues le 2026-08-14 : D5 et D3 par arbitrage de Guy — inscrits dans leurs décisions respectives —, le périmètre par le présent paragraphe.)*
