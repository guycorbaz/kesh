# Story 25.1a : Rendre la piste de contrôle inaltérable — et cesser de promettre ce qui est faux

Status: done

⚠️ **Issue du SPLIT de la 25-1**, décidé à la passe 1 de validation (2026-09-10). La story
d'origine portait **quatre volets hétérogènes et 14 AC** ; ses seuls trous d'alimentation touchent
**six modules**, ce qui déclenche la § *Règle de splitting préventif* du `CLAUDE.md`. Découpage :

| | objet |
|---|---|
| **25-1a** *(celle-ci)* | fermer les chemins d'effacement, et corriger ce que quatre documents publiés promettent |
| **25-1b** | combler les six trous d'alimentation — rollout mécanique |
| **25-1c** | rendre la piste consultable — route et écran |

⛔ **L'ordre est imposé et il porte un raisonnement** : rendre la piste *lisible* (25-1c) avant de
la rendre *inaltérable* publierait un écran qui donne à voir une trace qu'un `reset` dément.

## Story

**En tant que** responsable de comptes soumis à la conservation (CO art. 957-964, OLICo art. 9),
**je veux** que le journal d'audit soit réellement inaltérable — et que rien de publié n'affirme
plus qu'il l'est déjà,
**afin que** les corrections rendues *apparentes* par la contre-passation et le gel soient
**imputables**.

**Couvre** : [#376], [#377] — III.3 de l'audit. *([#378] → 25-1c, [#379] → 25-1b.)*

## Acceptance Criteria

### Volet A — les TROIS chemins d'effacement

1. **L'import d'une sauvegarde ne détruit plus la piste.** ⛔ **Le geste évident est MORTEL et la
   passe 1 l'a démontré** : sortir `audit_log` de `TABLES_TO_TRUNCATE` rendrait **tout backup
   existant inimportable**. `export.rs:55` produit les backups en itérant cette constante — tous en
   portent une entrée — et `import.rs:129` rejette en 400 tout manifeste contenant une table « hors
   inventaire applicatif ». ⇒ **`audit_log` RESTE dans la constante** ; c'est son **traitement au
   restore** qui change.
2. ⛔ **Et « conserver la piste » est un piège symétrique — le RÉSULTAT est donc posé comme
   contrainte, pas le mécanisme.** Le restore remplace `users` sous `FOREIGN_KEY_CHECKS = 0`
   (`backup.rs:400`, `DELETE` à `:434`), puis rétablit **sans revalider l'existant**. Les
   identifiants ne sont pas réattribués par auto-incrément — le restore réinsère les `id` sources
   explicitement — mais **les espaces d'identifiants de deux instances se recouvrent** : une ligne
   locale conservée pointerait vers un `users.id` disparu, ou occupé par **quelqu'un d'autre**.

   **Résultat exigé, et il est vérifiable** : *après l'import d'un backup étranger, chaque entrée
   d'audit antérieure à l'import existe encore et nomme son auteur d'origine.*

   ⚠️ **Les deux mécanismes envisagés en passe 1 échouent, et il faut le savoir avant de coder** :
   *« fusion avec dédoublonnage des `id` »* traite les collisions de clé primaire et **ne touche
   pas** le problème des acteurs ; *« conservation avec remappage »* réécrirait `user_id` vers un
   survivant — c'est-à-dire ferait dire à la piste que **quelqu'un d'autre** a fait l'opération,
   le défaut exact que cette story ferme. ⛔ **Et aucune colonne n'accueille l'attribution
   d'origine** : `audit_log` porte `user_id NOT NULL FK users(id) ON DELETE RESTRICT`, et la
   migration `20260605000002_audit_log_actor.sql` **interdit explicitement** de la passer nullable
   (« Ne PAS le passer nullable »). ⇒ **T1 doit donc trancher un mécanisme qui préserve le libellé
   de l'acteur** — colonne textuelle, dépôt dans `details_json`, ou autre — et l'écrire avant de
   coder. *Relevé en passe 2, P2-2.*
3. **`reset_demo` ne supprime plus la piste sans un droit** — `kesh-seed/src/lib.rs:250`,
   `DELETE FROM audit_log` non scopé.
4. **La route `/api/v1/onboarding/reset` exige le rôle administrateur.** Elle est montée dans
   `authenticated_routes` (`lib.rs:773`), **sans `require_admin_role`** — vérifié à la passe 1.
   ⚠️ Ses gardes actuelles sont des **états** (`step_completed >= 7`, `!is_demo && > 2`, le drapeau
   `KESH_PRODUCTION_RESET`), pas des droits. *Un état se contourne en amenant le système dans
   l'état voulu ; un droit, non.*
5. ⛔ **LE TROISIÈME CHEMIN, que la spec d'origine ignorait** : `/api/v1/_test/seed` et
   `/api/v1/_test/reset` appellent `truncate_all` (`test_endpoints.rs:172`), qui réutilise
   `TABLES_TO_TRUNCATE` (`test_fixtures.rs:327`) — donc `audit_log`. Monté dès
   `KESH_TEST_MODE=true`, **sans rôle requis**. Le dommage est nul en pratique (jamais activé en
   déploiement réel, refus de démarrer hors loopback) — mais **il doit être nommé** : sans cela,
   l'AC 9 refabriquerait le mensonge que [#359] a dû corriger.
6. **Un test négatif par chemin fermé, éprouvé PAR MUTATION** : la garde retirée, le test rougit.

### Volet B — cesser de promettre ce qui est faux

7. ⛔ **SEPT sites de manuel parlent de l'inaltérabilité, et ils ne mentent pas tous dans le même
   sens.** *(Le décompte total, README et site web compris, est à l'item 10 — dix mentions.)* La vague 0 a corrigé l'en-tête du module ([#359]) et **laissé les cinq** :

   | site | ce qu'il dit | aujourd'hui | après la story |
   |---|---|---|---|
   | `user-manual.tex:1597` | « aucune entrée ne peut être modifiée ou supprimée » | **faux** | **vrai SEULEMENT sur une instance finalisée** — cf. AC 8 |
   | `marketing-brochure.tex:139` | « *audit-trail immutable* […] garantit l'**intégrité** légale au sens de l'OLICo Art. 9 » | **faux** | ⛔ **dépend de la 25-1b** — l'intégrité suppose une piste *complète* |
   | `admin-manual.tex:1951` | « Art. 3 : **intégrité** → garanti par `audit_log` insert-only » | **faux** | ⛔ **dépend de la 25-1b** |
   | `admin-manual.tex:1785` | « aucune **route API** ne permet de modifier ou supprimer **une entrée** » | exact au mot près, **trompeur en substance** | à reformuler — une route efface la table entière |
   | ⛔ `admin-manual.tex:1803-1808` | « **Le journal d'audit n'est PAS inaltérable en pratique.** Deux chemins l'effacent » | **vrai**, mais **incomplet** (trois chemins) | ⛔ **DEVIENDRA FAUX** — à réécrire |
   | ⛔ `user-manual.tex:1726` *(glossaire)* | « Journal **infalsifiable** de toutes les modifications comptables » | **faux** | **vrai seulement sur instance finalisée** — cf. AC 8 |
   | ⛔ `admin-manual.tex:2179` *(glossaire)* | « Trace **immuable** de toutes les actions métier » | **faux** | **vrai seulement sur instance finalisée** — cf. AC 8 |

   ⚠️ **Le cinquième site est celui que la story rend faux**, et il vit dans la **section de
   conformité OLICo** — celle qu'un réviseur lit. Sans lui, le PDF publié dirait à cent-cinquante
   lignes d'écart que la piste est effaçable *et* qu'elle garantit l'intégrité. *Relevé en passe 2,
   P2-1.*

8. ⛔ **LA NUANCE QUI REND `user-manual:1597` VRAI, ET SANS LAQUELLE ON REFABRIQUE LE MENSONGE DE
   [#359].** L'AC 3 ne **supprime pas** la suppression : elle la met derrière un droit. Après cette
   story, un administrateur peut encore effacer la table par `POST /api/v1/onboarding/reset`.
   **Mais seulement sur une instance NON FINALISÉE** : le handler refuse à `step_completed >= 7`
   (`onboarding.rs:257`, « *irreversible finalization — never reset* »). ⇒ **Sur une instance en
   service, la piste est déjà inatteignable**, et c'est cela que le manuel doit dire — non « aucune
   entrée ne peut être supprimée » tout court. *Relevé en passe 2, P2-3(i).*

9. ✅ **`admin-manual.tex:1951` et `marketing-brochure.tex:139` sont TRAITÉS ICI, et non différés
   — arbitrage du Project Lead du 2026-09-11.**

   Ils affirmaient une **conformité** (« garantissent l'intégrité légale au sens de l'OLICo
   Art. 9 », « intégrité → garanti par `audit_log` insert-only`), ce qui supposait une piste
   complète — donc la 25-1b. ⛔ **Mais différer une promesse fausse revient à la maintenir.**
   Plutôt que d'attendre de pouvoir la tenir, on la **retire** : la brochure décrit désormais ce
   que le journal *fait*, et le manuel admin **dit la réserve** (« la couverture du journal n'est
   pas complète : la gestion des utilisateurs et quelques autres opérations n'y sont pas encore
   inscrites »).

   ⚠️ **Calibrage donné par le Project Lead** : *« ne pas se focaliser sur une application stricte
   de l'OLICo — très peu de logiciels comptables la suivent strictement. »* La conformité
   réglementaire cesse d'être le moteur des décisions ; la règle qui demeure est celle de la
   vague 0, indépendante de tout texte de loi — **ne pas promettre ce que le code ne fait pas**.
   *La décision de fusion, elle, tient sans l'OLICo : une piste qu'un import efface ne sert à
   rien, et l'argument utile était le blanchiment, pas l'article 9.* La brochure et
   `admin-manual:1951` parlent d'**intégrité**, qui suppose une piste **complète** — or les six
   trous d'alimentation sont renvoyés à la **25-1b**. ⇒ **Ne pas les rétablir ici** ; la 25-1a
   corrige ce qui la concerne et laisse ces deux-là à la 25-1b, qui les reprendra. *Sans quoi la
   25-1a promettrait ce qu'elle ne tient pas — P2-3(ii).*

10. ⛔ **L'INVENTAIRE, RECOMPTÉ DEPUIS LA SOURCE — et sa ventilation se recoupe.**

    ```sh
    grep -rniE "inaltérable|immutable|insert-only|infalsifiable|immuable|inviolable" \
      docs/manual/fr/*.tex website/*.html README.md
    ```

    **Dix mentions, six fichiers** — dont **sept dans les manuels** :

    | | sites | sort |
    |---|---|---|
    | **à traiter ici** | `admin:1785`, `admin:1804`, `admin:2179`, `user:1597`, `user:1726`, `README:218` | **6** |
    | **différées à la 25-1b** | `admin:1951`, `brochure:139` — elles portent sur l'**intégrité** (AC 9) | 2 |
    | **sans action** | `website/index.html:106`, `website/roadmap.html:105` — redeviennent vraies | 2 |

    ⚠️ **`website/about.html:138` est un faux positif** du grep élargi : il parle des « immutable
    change logs » du **processus BMAD**, pas de la piste d'audit.

    ⛔ **`README.md:218` n'est pas une omission, c'est une affirmation POSITIVE et DOUBLEMENT
    FAUSSE** : « une piste d'audit **inaltérable et consultable** », au présent. Le second membre
    dépend de la **25-1c**. ⇒ le corriger avec la **même grille** que les deux sites différés :
    nuancer l'inaltérabilité *et* dire que la consultation reste à venir. *Un correctif qui ne
    retirerait que « inaltérable » laisserait la phrase fausse. P3-3.*

    ⛔ **QUATRIÈME décompte faux d'affilée sur cette story, et le motif change ENCORE** : le
    titre de l'AC 7 est passé de « CINQ » à « SEPT », et l'AC 9 a gardé son « des CINQ » — **dans
    le même document, à vingt-six lignes d'écart**. C'est la § *Propagation post-patch* prise en
    défaut sur son voisinage le plus proche : *corriger la thèse au site nommé et laisser ses
    applications ailleurs dans le même document* — exactement le mode d'échec que la 24-4c avait
    relevé comme « propre à cette story ».

    ⇒ **RÈGLE TIRÉE, ET APPLIQUÉE ICI** : *un énoncé qui dépend d'un TOTAL se périme à chaque
    révision du total ; un énoncé qui NOMME ses objets, non.* L'AC 9 nomme désormais ses deux
    sites au lieu de les compter. Les seuls chiffres qui subsistent sont ceux de la ventilation
    ci-dessus, qui **se recoupent entre eux** — et c'est leur recoupement, non leur exactitude
    supposée, qui les rend contrôlables.

    ⚠️ **Trois décomptes faux avant celui-là** — « quatre documents » (P1), puis
    « sept mentions, cinq à traiter » (P2), non dérivable de sa propre énumération. **Et le motif
    a changé à chaque fois** : d'abord un inventaire incomplet, puis un total incrémenté, enfin un
    mot-clé trop étroit — `inaltérable|immutable|insert-only` **ne pouvait pas** attraper
    « infalsifiable » ni « immuable ». *La § « greper la VALEUR, pas la formulation » a une
    jumelle : greper le CONCEPT, pas un de ses mots. Les deux glossaires ont traversé trois
    passes, dont une qui cherchait explicitement un site supplémentaire.* P3-1, P3-2.

11. **L'en-tête de `audit_log.rs:1-21` est réécrit**, et il **nomme les TROIS chemins** (AC 5).
    ⚠️ Il dit aujourd'hui l'inverse. *Une fois les chemins fermés, cette phrase devient fausse à
    son tour, dans l'autre sens.*

12. **PDF régénérés** (`make fr`) et commités. ⚠️ Les manuels `de/`, `en/`, `it/` ne contiennent
    **aucun `.tex`** — vérifié en passe 2 : rien à y traduire.

## Tasks / Subtasks

- [x] **T1 — Trancher le mécanisme du restore** (AC 1, 2) : lire `backup.rs:382-460`,
      `import.rs:112-140`, et les tests `admin_full_import_e2e.rs:330-346` /
      `admin_backup_e2e.rs:263-277` qui **encodent le comportement actuel**. Écrire la décision et
      son motif dans les Dev Notes **avant** de coder.
- [x] **T2 — Implémenter** (AC 1, 2) + tests, dont un prouvant qu'un backup **existant** reste
      importable et que l'export porte toujours `audit_log`.
- [x] **T3 — Fermer `reset_demo`** (AC 3, 4).
- [x] **T4 — NOMMER le chemin de test** (AC 5), sans alternative. ⛔ **Ne PAS le fermer** :
      `truncate_all` sert `/api/v1/_test/seed` et `/reset`, dont dépend **tout le montage de la
      suite E2E** (`frontend/tests/e2e/helpers/test-state.ts:83`). Le fermer casserait une suite
      que la CI n'exécute pas — personne ne le verrait avant le gate local. *P2-5.*
- [x] **T5 — Mutation** (AC 6) sur chaque garde neuve, **plus un test POSITIF de refus** :
      ⛔ **la mutation ne détecte pas une garde JAMAIS POSÉE.** `lib.rs:300-306` documente le
      piège : *« une route chaînée après le `route_layer` COMPILE, ne panique pas, et échappe aux
      DEUX couches »*. Un test « non-Admin ⇒ 403 » le voit ; une mutation, non. *P2-6.*
- [x] **T6 — Les SIX sites à traiter + l'en-tête du module** (AC 7 à 11), PDF régénérés (AC 12),
      et le README vérifié.
- [x] **T7 — SI le mécanisme de T1 touche le schéma** : garde-fous **P2-bis** (bump Cargo
      solidaire), **P3** (bump `min_required` si breaking), **P5** (ligne d'audit d'idempotence +
      recompte des deux totaux et des trois compteurs), **P6** (`grep` des sites positionnels),
      **P7** (triage au registre de rejeu). *P2-4.*
- [x] **T8 — Gate complet**, base remise à zéro. ⛔ `kesh-db` touché : **ciblage interdit**.

## Dev Notes

### Ce que la passe 1 de validation a établi, et qui n'était pas dans la spec d'origine

⛔ **`TABLES_TO_TRUNCATE` a SIX usages, pas deux** — c'est le fait central de cette story :

| site | usage |
|---|---|
| `backup.rs:400-408` | les `FOREIGN_KEY_CHECKS` du restore (le `DELETE` est à `:434`) |
| `export.rs:55` | **produire** l'export NDJSON |
| `export.rs:96` | `table_count` du **manifeste** |
| `import.rs:118` | couverture — chaque table attendue doit figurer au manifeste |
| `import.rs:129` | ⛔ **réciproque** — toute table du manifeste **hors** de la constante ⇒ **400** |
| `test_fixtures.rs:327` | `truncate_all` des tests, et de `/api/v1/_test/*` |

**La FK est le nœud.** `audit_log.user_id → users(id) ON DELETE RESTRICT`
(`20260413000001_audit_log.sql`). Le restore remplace `users` ; toute ligne d'audit conservée
pointe alors dans le vide — **sans qu'aucune erreur ne remonte**, puisque `FOREIGN_KEY_CHECKS=1`
ne revalide pas l'existant. *La piste mentirait silencieusement sur qui a fait quoi : exactement le
défaut que la story existe pour fermer.*

**Le patron de traçage a une réserve que la spec d'origine ignorait.** `NewAuditLogEntry` a **trois**
constructeurs (`user`, `for_actor`, `api_key`) et un trait `AuditActor::from_current_user`
(`kesh-api/src/audit.rs`) qui choisit selon `user.api_key_id`. Le patron de `lock_books` utilise
`::user(user_id, …)` avec un `i64` brut — **il perd l'attribution par clé API**. C'est sûr sur les
routes `admin_routes` (PAT exclus par `require_not_pat`), et **`lock_books` lui-même ne l'est
pas** : sa route vit dans `comptable_routes`, sans ce garde. ⇒ **dette à tracer séparément**, hors
périmètre ici. *Relevé par la lentille Sonnet, S3.*

**La table n'a pas de `company_id`** — elle est **globale** dans une application multi-société.
Sans conséquence pour cette story ; **bloquant pour la 25-1c**, qui doit trancher.

### ⛔ T1 — DÉCISION DE CONCEPTION : le mécanisme du restore

**Écrite avant de coder, comme l'AC 2 l'exige.** Le résultat à tenir : *après l'import d'un backup
étranger, chaque entrée d'audit antérieure existe encore et nomme son auteur d'origine.*

#### Ce qui rendait la question difficile, et ce qui la débloque

Deux patrons **existent déjà dans le dépôt** et résolvent chacun une moitié :

| patron | où | ce qu'il donne |
|---|---|---|
| **Exclusion du restore** | `onboarding_state` — dans `TABLES_TO_TRUNCATE` (donc au manifeste, donc l'import ne casse pas) mais **ni effacée ni restaurée** (`backup.rs:431,445`) | conserver la piste locale **sans toucher au format de backup** |
| **Pointeur logique sans FK** | `actor_api_key_id`, `entity_id` — *« l'audit survit 10 ans à la révocation/suppression de la clé »* | survivre à la disparition de l'acteur |

*La décision n'invente donc rien : elle applique à `audit_log` ce que le dépôt fait déjà ailleurs.*

#### Le scénario qui tranche

L'audit reproche que l'import « remplace intégralement la piste ». Le risque n'est pas la perte
d'un historique : c'est le **blanchiment** — effacer ses traces en important un backup. ⛔ **Une
option qui conserve la piste locale sans restaurer celle du backup ne ferme ce scénario qu'à
moitié** : elle protège l'instance courante et perd l'historique légitime de l'instance
sauvegardée, que l'OLICo demande de conserver dix ans.

#### Décision retenue — **fusion**

1. **`audit_log` sort du `DELETE`** du restore, comme `onboarding_state` — la piste locale est
   **conservée intégralement**.
2. **Les entrées du backup sont INSÉRÉES sans leur `id`** (auto-increment), et non avec : les
   espaces d'identifiants des deux instances se recouvrent. ⚠️ L'`id` n'est pas une donnée métier
   d'une piste ; l'ordre chronologique est porté par `created_at`.
3. **Une colonne dénormalisée `actor_label`** porte le nom de l'acteur au moment de l'écriture. La
   FK `user_id` devient un **pointeur logique** — patron `actor_api_key_id`. *C'est elle qui tient
   « nomme son auteur d'origine » quand le `users.id` a changé de titulaire.*
4. **Une entrée `admin.full_import`** documente la fusion, avec le compte des deux côtés.

#### Ce que cette décision coûte, et il faut le dire

- **Une migration** ⇒ garde-fous **P5** (ligne d'audit d'idempotence + recompte), **P6** (grep des
  sites positionnels), **P7** (triage du backfill d'`actor_label` au registre de rejeu).
- **Non breaking** (`ADD COLUMN` + `DROP` d'une FK — un binaire antérieur continue d'insérer des
  lignes valides) ⇒ **ni bump `min_required`, ni bump Cargo** (P1/P2-bis).
- **Deux tests existants encodent le comportement actuel** et **rougiront** :
  `admin_backup_e2e.rs:277-281` (`audit_log == baseline + 1`) et `admin_full_import_e2e.rs:343-355`.
  *Ce n'est pas une régression : c'est le comportement que la story change, et leurs assertions
  doivent être réécrites en même temps que le code.*

⚠️ **ARBITRAGE SOUMIS AU PROJECT LEAD** — cette décision engage la **conformité** (OLICo art. 9,
CO art. 958f), pas seulement l'implémentation. L'alternative, plus simple, serait de conserver la
piste locale **sans** restaurer celle du backup : moins de code, pas de fusion d'`id`, mais
l'historique de l'instance sauvegardée est perdu à chaque restauration.

### Ce que la story ne fait pas

- **Les trous d'alimentation** → 25-1b. Six modules, pas deux : `users` (4 opérations),
  `contact_persons` (3), `imported_supplier_invoices` (2), `profile`, `setup`, `companies::update`.
  ⚠️ *Le traçage se fait au **repository**, pas à la route : un `grep` au niveau du fichier de route
  conclut à un trou là où il n'y en a pas, et l'inverse.*
- **La route et l'écran de consultation** → 25-1c.
- **Aucun trigger SGBD ni `REVOKE`** : rendrait l'import et les `#[sqlx::test]` inopérants.
- **La dette d'attribution PAT de `lock_books`** — à tracer.

### References

- `audit-experts-2026-08-26.md` § III.3 · `epic-25-vague1-suite.md` § 25-1
- Issues : [#376], [#377] · voisines : [#378] (25-1c), [#379] (25-1b), [#359] (vague 0, close)
- Sites : `kesh-db/src/backup.rs:34` (la constante), `:400-408` (les `FOREIGN_KEY_CHECKS`), `:434` (le `DELETE`), `:582-609` (`backup_inventory_matches_schema`), `kesh-api/src/admin_backup/export.rs:55,96`,
  `import.rs:118,129`, `kesh-db/src/test_fixtures.rs:327`, `kesh-api/src/routes/test_endpoints.rs:172` (`/seed`) et `:306` (`/reset`),
  `kesh-seed/src/lib.rs:250`, `kesh-api/src/lib.rs:773`, `routes/onboarding.rs:257-289`,
  `kesh-db/src/repositories/audit_log.rs:1-21`, `migrations/20260413000001_audit_log.sql`,
  `migrations/20260605000002_audit_log_actor.sql` (« ne PAS passer `user_id` nullable »),
  `kesh-db/src/post_restore.rs` (rejeu des backfills après import — **à lire en T1**)
- Tests qui encodent le comportement actuel : `admin_full_import_e2e.rs:343-355` (le bloc O-1),
  `admin_backup_e2e.rs:277-281` (l'assertion `audit_log = baseline + 1`), et **le rayon d'impact de
  l'AC 4** : `onboarding_e2e.rs:343,382,444,486` + `frontend/src/lib/features/onboarding/onboarding.api.ts:21`

## Dev Agent Record

### Agent Model Used

- **Implémentation** (`bmad-dev-story`) : Claude Opus 5 (1M context).
- **Validation de spec**, 4 passes : Sonnet 4.6 → Haiku 4.5 → Opus 5 → Sonnet 4.6.
- **Revue de code**, passe 1 — trois lentilles en contexte frais : BlindHunter
  (Sonnet 4.6), EdgeCaseHunter (Haiku 4.5), AcceptanceAuditor (Sonnet 4.6).

### Debug Log References

Quatre garde-fous du dépôt se sont déclenchés pendant l'implémentation, **aucun
redondant** — chacun a attrapé un défaut qu'aucun autre ne voyait :

| Garde-fou | Ce qu'il a attrapé |
|---|---|
| `every_data_backfill_migration_is_triaged` (P7) | l'`UPDATE` de backfill non trié → entrée classe B au registre |
| `migrations.sha384` (P8) | checksum de la migration non inscrit |
| `migrations_upgrade_path` (P6) | site positionnel `total - N` décalé par la 67ᵉ migration |
| `admin_pat_denied_e2e` (marqueurs `KESH-ADMIN-ROUTES`) | compteur `(5, 21)` du bloc admin, devenu `(5, 22)` |

Trois échecs de gate ont par ailleurs été diagnostiqués et fermés :

- `Unknown column 'actor_label'` — squash de test périmé → `scripts/regen-test-schema.sh` ;
- `ColumnNotFound("actor_label")` — champ ajouté à l'entité sans l'ajouter à
  `const COLUMNS` du repository (le doc-comment du fichier en avertissait) ;
- `users.company_id` NOT NULL sans défaut — fixture de test incomplète, société
  créée explicitement.

### Completion Notes List

- **T1 — décision de conception arbitrée avant de coder** (cf. Dev Notes) : le
  restore **fusionne** `audit_log` au lieu de la remplacer. `audit_log` rejoint
  `onboarding_state` dans l'exclusion du `DELETE`, et son `id` est écarté de
  l'`INSERT` pour que les identifiants des deux instances puissent se recouvrir
  sans collision.
- **La FK `fk_audit_log_user` est retirée** : conserver une entrée locale alors
  que `users` est remplacée ferait pointer son `user_id` vers un identifiant
  disparu — ou réattribué à quelqu'un d'autre. `user_id` rejoint la famille des
  pointeurs logiques sans FK (`entity_id`, `actor_api_key_id`), et `actor_label`
  nomme ce que le pointeur ne garantit plus.
- **`actor_label` est un INSTANTANÉ, pas une jointure** — posé par un sous-SELECT
  dans l'`INSERT` du repository, il porte le `username` au moment de l'écriture
  et ne suit pas les renommages. Le cas est verrouillé par
  `full_import_preserves_archived_actor_label_when_column_is_present`.
- **AC 6 — la réinitialisation de démo exige un droit, pas un état** : la route
  `POST /api/v1/onboarding/reset` est passée dans le bloc `admin_routes`, et le
  bouton du `DemoBanner` est masqué hors rôle Admin (un contrôle visible mais
  mort fait chercher la panne du mauvais côté).
- **Volet B — les promesses retirées, non différées.** Huit sites affirmaient une
  inaltérabilité que le code ne tient pas ; le huitième (`entities/audit_log.rs`)
  a été manqué par l'inventaire initial, qui ne balayait que `docs/`, `website/`
  et `README.md` — pas le code. Leçon portée au Change Log.

**Décomptes, recomptés depuis la source, périmètre déclaré :**

| Mesure | `main` → dev (`fd49c77e`) | `main` → HEAD (revue P1 comprise) |
|---|---|---|
| tests `admin_full_import_e2e.rs` | 23 → 23 | 23 → **25** |
| tests `onboarding_e2e.rs` | 13 → **14** | 13 → **14** |
| tests `backup.rs` (`mod tests`) | 8 → **9** | 8 → **10** |
| **total de tests neufs** | **2** | **5** |

Migrations : **67** (`ls crates/kesh-db/migrations/*.sql | wc -l`), reportées aux
cinq sites de `docs/migrations-idempotence-audit.md` (7 `yes` + 60
`tracked-by-sqlx`).

**Gate** : le gate complet est lancé au push, conformément à l'exception
`kesh-db` de la § *« Pendant une boucle de revue »* — cette story touche
`migrations/`, `post_restore.rs` et un repository, le ciblage y est interdit.
Les résultats réellement obtenus sont consignés au Change Log, jamais anticipés.

### File List

**Backend — schéma et persistance**
- `crates/kesh-db/migrations/20260910000001_audit_log_actor_label.sql` *(nouveau)*
- `crates/kesh-db/src/post_restore/20260910000001_audit_log_actor_label.sql` *(nouveau)*
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/test-schema/0001_schema_squash.sql` *(régénéré, jamais édité)*
- `crates/kesh-db/src/backup.rs`
- `crates/kesh-db/src/post_restore.rs`
- `crates/kesh-db/src/entities/audit_log.rs`
- `crates/kesh-db/src/repositories/audit_log.rs`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-db/tests/fiscal_years_repository.rs`

**Backend — API**
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/routes/admin.rs`
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/admin_backup_e2e.rs`
- `crates/kesh-api/tests/admin_pat_denied_e2e.rs`
- `crates/kesh-api/tests/onboarding_e2e.rs`

**Frontend et i18n**
- `frontend/src/lib/shared/components/DemoBanner.svelte`
- `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`

**Documentation**
- `docs/migrations-idempotence-audit.md`
- `docs/manual/fr/{user-manual,admin-manual,marketing-brochure}.{tex,pdf}`
- `README.md`

## Change Log

### Passe 1 de `bmad-code-review` — trois lentilles, contexte frais

| Lentille | Modèle | Findings |
|---|---|---|
| BlindHunter | Sonnet 4.6 | 4 |
| EdgeCaseHunter | Haiku 4.5 | 5 |
| AcceptanceAuditor | Sonnet 4.6 | 2 |

**Après dédoublonnage : 0 CRITICAL, 0 HIGH, 7 MEDIUM, 2 LOW.** Trois findings
convergeaient (BH-1 / AA-2 / ECH-3), sur le même trou.

**Remédiations, dans l'ordre où elles ont été appliquées :**

- **ECH-2 / AA-1 (MEDIUM)** — l'AC 6 n'était tenue qu'à moitié : le seul test
  couvrant `POST /onboarding/reset` exerçait un PAT (garde anti-PAT), pas le
  RBAC. → `reset_by_comptable_is_forbidden`, éprouvé par mutation (route remise
  dans `authenticated_routes` → 200 au lieu de 403).
- **BH-2 (MEDIUM)** — l'en-tête de `entities/audit_log.rs` affirmait encore que
  « les entrées d'audit sont inamovibles » et que « la FK `ON DELETE RESTRICT`
  empêche… ». **Huitième site**, manqué parce que l'inventaire du volet B ne
  balayait que `docs/`, `website/` et `README.md` — pas le code. Réécrit pour
  renvoyer à la source unique. Grep élargi au code
  (`inamovible|inaltérable|infalsifiable|immuable|insert-only` sur `crates/`) :
  aucun autre site.
- **BH-3 (MEDIUM)** — le bouton « Réinitialiser pour la production » n'était pas
  gaté par rôle : depuis le déplacement de la route dans `admin_routes`, un
  Comptable le voyait, cliquait, et recevait un toast générique. → bouton masqué
  hors Admin, et le 403 distingué d'une panne (`demo-reset-forbidden`, quatre
  locales). ⚠️ La première rédaction importait `authState` depuis un chemin
  **inexistant** (`$lib/features/auth/…` au lieu de `$lib/app/stores/…`) et le
  test inline du 403 dupliquait `isApiError` : les deux corrigés avant gate.
- **BH-1 / AA-2 / ECH-3 (MEDIUM, convergents)** — aucun test HTTP ne couvrait
  « backup pré-25-1a → fusion → rejeu post-restore ».
  `restore_merges_audit_log_and_keeps_actor_names` exerce `restore_tables_in_tx`
  en direct, sur des lignes **déjà pourvues** du libellé : il prouve la fusion,
  jamais le rejeu. → deux cas neufs dans `admin_full_import_e2e.rs` :
  `full_import_replays_actor_label_when_column_is_absent` (C7) et
  `full_import_preserves_archived_actor_label_when_column_is_present` (C7-bis).
  **Éprouvés par deux mutations**, chacune tuant les deux tests : (1) retrait de
  l'entrée `20260910000001` du registre → C7 rougit sur son assertion de cœur
  (`1 != 0` libellé vide) ; (2) `audit_log` remise dans le `DELETE` du restore →
  C7-bis rougit sur la fusion (`["nom_d_alors"]` au lieu de deux).
- **ECH-4 (MEDIUM)** — le cas du backfill rejoué **hors flux d'import** n'était
  documenté nulle part. → en-tête de l'extrait
  `post_restore/20260910000001_*.sql`, qui énonce pourquoi la jointure ne se
  trompe pas *dans ce flux* (l'ordre : le rejeu suit le remplacement de `users` ;
  la garde : `actor_label = ''` n'atteint que les lignes d'archive) et pourquoi
  un rejeu manuel viole cette condition.
- **ECH-5 (LOW)** — cas limite `actor_label` à 64 caractères. Le recompte montre
  que `users.username` et `actor_label` sont tous deux `VARCHAR(64)`, donc
  **aucune troncature possible aujourd'hui** — mais rien ne verrouillait
  l'égalité. Le mode d'échec n'est d'ailleurs pas « un libellé tronqué » : le
  libellé venant d'un sous-SELECT, MariaDB en mode strict refuserait l'`INSERT`
  d'audit, donc **l'opération métier entière**. → test
  `actor_label_is_at_least_as_wide_as_username`, qui lit les deux largeurs dans
  `information_schema` plutôt que de répéter un nombre.
- **BH-4 (LOW)** — deux décomptes périmés dans `post_restore.rs` (« neuf »
  migrations écrivant des données, « sept » exemptées). Recompte depuis la
  source : **2 actives + 11 exemptées = 13**. Plutôt que de corriger les
  nombres, les deux énoncés ont été **réécrits sans total** — un énoncé qui
  dépend d'un total se périme, un énoncé qui nomme ses objets, non.

**Grep de propagation — et il a rendu six sites de plus.** Le symptôme corrigé
n'était pas un nombre mais une **affirmation** : « le registre de production est
vide ». Vraie entre les Stories 24-2 et 24-3, fausse depuis, elle vivait encore
dans le doc-comment de `replay_retired` **et dans six commentaires de cas**.
Tous corrigés dans le même patch.

⚠️ Un septième site a été inspecté et **volontairement laissé** :
`24-2-encaissement-client.md:343` (« l'import ne rejoue plus rien »). C'est un
compte rendu **daté** d'une story close, exact au moment où il a été écrit — le
réécrire falsifierait l'histoire plutôt que de corriger une erreur.

**Cinquième garde-fou du dépôt déclenché par cette story** : `i18n-keys.test.ts`
a rougi sur `1631 != 1630` dès l'ajout de `demo-reset-forbidden`. Compteur mis à
jour **avec sa ventilation** (+1 site, et non +1 clé : le `{#if}` du masquage
n'ajoute aucun site — il change qui voit la clé, pas combien de fois le code la
demande).

**Gate — ce qui a RÉELLEMENT tourné.**

| Gate | Résultat |
|---|---|
| `cargo fmt --all -- --check` | vert |
| `cargo clippy --workspace --all-targets -D warnings` | vert, 0 warning |
| `cargo nextest run` (profil `ci`, base remise à zéro) | **2306 passed, 0 failed, 4 skipped** — 92,8 s |
| `npm run check` | 0 erreur (27 warnings préexistants, aucun dans le diff) |
| `npm run lint-i18n-ownership` | vert |
| `npm run test:unit` | **740 passed**, 76 fichiers |
| `npm run build` | vert |

⚠️ **Le premier gate a rougi sur 34 tests `products` / `journal_entries`, et la
cause n'était pas la story** : la commande de remise à zéro avait vu son
`sqlx migrate run` échouer **en silence** (sortie redirigée vers `/dev/null`),
laissant la base de gate **sans schéma**. La doctrine du dépôt dit de
reconstruire la base avant de diagnostiquer ; ce cas montre qu'elle doit dire
aussi de **vérifier que la reconstruction a réussi** — une remise à zéro ratée
est indiscernable d'une régression, et coûte le même diagnostic.

⚠️ **Le second gate a rougi sur UN seul test, et c'était le mien** :
`actor_label_is_at_least_as_wide_as_username` décodait
`CHARACTER_MAXIMUM_LENGTH` en `i64` alors qu'`information_schema` le rend en
`BIGINT UNSIGNED`. Corrigé en `u64`, puis **éprouvé par mutation** (rétrécir
`actor_label` à `varchar(32)` dans le squash de test → rouge ; squash restauré
intact, `git diff` vide).

**La suite E2E Playwright reste à lancer** — elle est un prérequis du `push`,
non du commit, et n'a donc pas encore tourné sur cette branche.

### Passe 2 de `bmad-code-review` — lentille unique (Haiku 4.5), contexte frais

Prompt versionné : `25-1a-review-prompt-p2.md`. Périmètre : le seul commit de
remédiation de la passe 1 (`9dfbb527`).

**Rendu brut : 1 CRITICAL, 1 HIGH, 1 MEDIUM. Après vérification au sol : deux
réfutés, et UN défaut réel qu'aucun des trois ne nommait correctement.**

#### C1 (CRITICAL) — RÉFUTÉ

*« Les manuels n'ont pas été modifiés, les PDF non plus. »*

Faux. Les six sites ont été traités au commit `b48a9e00` et les trois PDF
régénérés. La lentille a inspecté `git show 9dfbb527:docs/manual/…` — le commit
de **remédiation**, qui ne touche pas les manuels — et en a conclu « texte
identique à `main` » **sans jamais comparer à `main`**.

⚠️ Le plus instructif : l'extrait qu'elle cite comme preuve à charge contient la
phrase qui la réfute — *« l'import d'une sauvegarde ne la remplace plus »*. C'est
le mode d'échec Haiku documenté au `CLAUDE.md` (§ *Haiku-specific guardrails*),
transposé du diff multi-commit au **choix de la base de comparaison**.

⛔ **Et la lentille déclarait cet axe « NON EXERCÉ » tout en en tirant son
CRITICAL.** Déclarer un axe non exercé et en produire un finding sont
contradictoires ; c'est un signal que l'orchestrateur doit traiter comme tel.
Contrôle : `git diff --stat main...HEAD -- docs/manual/` → 6 fichiers, 22
insertions, 12 suppressions.

#### H1 (HIGH) — RÉFUTÉ SUR SA PRÉMISSE

*« Une ligne d'audit LOCALE portant `actor_label = ''` serait réattribuée par la
jointure. »*

L'état de départ n'est pas atteignable : `grep -rn "INSERT INTO audit_log"
crates/ --include=*.rs` ne rend que **deux** sites, dont l'un est dans `mod
tests`. Le seul chemin de production est `repositories::audit_log::insert_in_tx`,
dont le sous-SELECT `COALESCE(…, '(inconnu)')` ne produit **jamais** de chaîne
vide. Aucune migration ni seed n'écrit dans la table.

#### Le défaut RÉEL, trouvé en vérifiant les deux précédents

H1 et M1 tournaient autour d'un trou véritable sans le nommer : **rien
n'exerçait la garde `WHERE actor_label = ''` dans le cas où le rejeu tourne.**

- **C7-bis** ne fait pas tourner l'`UPDATE` du tout — sentinelle présente,
  entrée `Skipped`. Il éprouve le **déclencheur**, pas la garde.
- **C7** le fait tourner, mais sa ligne locale porte `al_absent_user`, soit
  exactement ce que la jointure produirait : retirer la garde y est
  **indiscernable**.

⛔ **Et le dommage n'est pas celui que le finding décrivait.** Après le restore,
`users` est celle du **backup** : le `user_id` d'une entrée locale conservée y
désigne le porteur de cet identifiant dans l'instance *source*. Un rejeu sans
garde ne rendrait pas l'entrée anonyme — il l'attribuerait à **quelqu'un
d'autre**. C'est le mensonge silencieux que cette story existe pour fermer.

→ `full_import_replay_does_not_overwrite_a_local_actor_label` (C7-ter). L'état
de départ est atteignable en production : il suffit qu'un utilisateur ait été
renommé depuis l'écriture — ce que l'instantané est fait pour tenir.

**Mutation, et son résultat est le cœur de la démonstration** : garde remplacée
par `WHERE TRUE` → **C7 et C7-bis restent VERTS, seul C7-ter rougit**. C'est ce
qui établit que le trou existait.

#### Résidu trouvé en réfutant C1

`user-manual.tex:1603` affirmait encore *« Elle garantit la traçabilité conforme
à l'OLICo Art. 9 al. 1.b ch. 4 »* — une **promesse de conformité**, que
l'arbitrage du Project Lead demandait de retirer et non de différer. Le
`marketing-brochure` et l'`admin-manual` avaient été traités au volet B ; celui-ci
non, le grep d'alors ayant porté sur l'inaltérabilité et non sur la conformité.

Reformulé : *« contribue à la traçabilité attendue par l'OLICo Art. 9 al. 1.b
ch. 4 — sans que cela vaille attestation de conformité : celle-ci se juge sur
l'installation entière et son exploitation, non sur une table. »* PDF régénéré
(`make fr`) et **vérifié au `pdftotext` aplati** — l'ancienne formulation est
absente, la nouvelle présente.

Grep de propagation sur le symptôme (`garanti(t|ssent) .{0,40}(OLICo|CO Art)`
sur `docs/manual/`, `website/`, `README.md`) : **ce site était le seul**.

#### ⛔ Le grep du volet B était en FRANÇAIS — et le site public est en anglais

En reprenant l'axe 3 moi-même, deux affirmations non nuancées sont apparues sur
un support **publié automatiquement au push sur `main`** :

- `website/index.html:106` — « immutable audit log »
- `website/roadmap.html:105` — « immutable audit log »

Le grep du volet B portait sur
`inamovible|inaltérable|infalsifiable|immuable|insert-only` : **aucun de ces
jetons n'apparaît dans une page anglaise**. C'est, pour la **cinquième fois sur
cette story**, le même motif sous une forme neuve — *le mot-clé trop étroit* —, et
la variante est cette fois **la langue du support**, non le choix du synonyme.

Corrigé sans sur-promettre dans l'autre sens : `index.html` annonce désormais
« an audit log that a backup import no longer replaces » (ce que la story livre,
vérifiable), `roadmap.html` décrit l'E3 par « append-only audit log » (exact au
niveau de la table, sans revendiquer l'inaltérabilité du système).

⚠️ `website/about.html:138` mentionne « immutable change logs » — c'est le
**processus BMAD**, sans rapport avec `audit_log`. Inspecté, laissé.

**Règle qui en sort, pour la rétrospective** : un grep de propagation sur des
supports multilingues doit porter les jetons de **chaque langue publiée**, ou
mieux, partir des **fichiers** à couvrir plutôt que des mots à trouver.

#### Gate de la passe 2 — ce qui a RÉELLEMENT tourné

| Gate | Résultat |
|---|---|
| `cargo fmt` + `clippy --workspace --all-targets -D warnings` | vert, 0 warning |
| `cargo nextest run` (profil `ci`, base remise à zéro **et vérifiée**) | **2307 passed, 0 failed, 4 skipped** — 85,1 s |

⚠️ **La remise à zéro est désormais VÉRIFIÉE, pas seulement exécutée** : la
commande compte les lignes seedées et interrompt le gate si la base est vide.
C'est la leçon du premier gate de la passe 1, où un `sqlx migrate run` échoué en
silence avait produit 34 faux échecs indiscernables d'une régression.

#### M1 (MEDIUM) — mal fondé, mais convergent

Le raisonnement porte sur C7-bis, où l'`UPDATE` **ne tourne pas** (entrée
`Skipped`, `rows_affected == 0` asserté) : la mutation proposée n'y aurait rien
changé. La mutation qu'il suggère — `WHERE FALSE` — est en revanche la bonne
idée appliquée au mauvais cas ; c'est C7-ter qui la porte, sous la forme
`WHERE TRUE`, qui est le sens du dommage.

### Passe 3 de `bmad-code-review` — lentille unique (Sonnet 4.6), contexte frais

Prompt versionné : `25-1a-review-prompt-p3.md`, écrit pour **corriger les deux
erreurs de méthode de la passe 2** : nommer et justifier sa base de comparaison,
et vérifier que l'état de départ d'un scénario est atteignable.

**0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW.** Les sept axes sont déclarés exercés.

#### LOW-1 — retenu et corrigé

`website/roadmap.html:105` annonçait « append-only audit log » — formulation
posée par la passe 2 elle-même, et **littéralement démentie** par l'`UPDATE` du
rejeu post-restore. La portée pratique est faible (le rejeu ne complète que le
nom d'acteur d'une ligne fraîchement fusionnée, jamais le contenu métier, jamais
une entrée locale déjà attribuée), mais l'affirmation est fausse au mot près sur
un support **publié**. Aligné sur la formulation d'`index.html`, exacte et déjà
validée : « audit log that a backup import no longer replaces ».

⚠️ **C'est une remédiation de la passe 2 qui a introduit ce défaut** — le motif
mesuré du dépôt, *la sévérité se déplace vers ce qu'on vient d'écrire*, s'est
vérifié une fois de plus, cette fois sur une ligne de prose.

#### Observation écartée — le `LEFT JOIN` du backfill

Aucun test n'exerce le `COALESCE(u.username, '(inconnu)')` du rejeu : une
mutation `LEFT JOIN` → `JOIN` ne serait tuée par aucun des trois cas
`actor_label`. La lentille l'a **écartée elle-même** plutôt que d'en faire un
finding, et son raisonnement tient : (a) un backup pré-25-1a provient d'une
instance où `fk_audit_log_user` existait encore, donc chaque `user_id` y avait un
utilisateur ; (b) un backup post-25-1a est sauté par la sentinelle. Le chemin est
inatteignable — ce que la migration dit déjà en toutes lettres (« impossible
aujourd'hui […] mais la garde coûte une ligne »). **Rien à faire**, et le
signalement est correct : un état inatteignable n'est pas un défaut.

#### Ce que la passe 3 a revérifié depuis la source, et non depuis nos déclarations

- **P5** : `ls migrations/*.sql` = 67 = en-tête = lignes du tableau ;
  7 `yes` + 60 `tracked-by-sqlx` + 0 `no` = 67.
- **P6** : `migrations_upgrade_path.rs` passé de `total - 32` à `total - 33`,
  frontière 34 inchangée, test vert.
- **P7** : classe B, sentinelle valide, absente d'`EXEMPT_MIGRATIONS`.
- **P8** : migration neuve, `published_migrations_keep_their_checksums` vert.
- **La route déplacée** est bien enregistrée **avant** le
  `route_layer(require_admin_role)` — vérifié par **lecture directe**, et pas
  seulement par le test qui en dépend.
- La réserve du manuel « la couverture n'est pas complète (gestion des
  utilisateurs) » est **vraie** : `routes/users.rs` ne contient aucun appel à
  `insert_in_tx`.

⚠️ **Ce que la passe 3 n'a PAS vérifié, et le dit** : elle n'a pas relancé le
gate complet (elle a listé 2307 tests sans les exécuter tous) ni la suite
Playwright. Le « 4 skipped » du Change Log n'a donc pas été recontrôlé par elle —
il l'est par le gate de clôture ci-dessous.

---

## Boucle de revue de code — close

| Passe | Modèle(s) | Rendu | Réel après vérification |
|---|---|---|---|
| 1 | Sonnet 4.6 · Haiku 4.5 · Sonnet 4.6 (3 lentilles) | 0 C, 0 H, **7 M**, 2 L | 9 findings, tous fondés |
| 2 | Haiku 4.5 (lentille unique) | 1 C, 1 H, 1 M | **2 réfutés**, 1 mal fondé — mais **1 défaut réel** trouvé en les vérifiant |
| 3 | Sonnet 4.6 (lentille unique) | 0 C, 0 H, 0 M, **1 L** | 1 finding fondé, corrigé |

**Critère d'arrêt atteint** : plus aucun finding au-dessus de LOW, et la
remédiation de la passe 3 ne touche **aucune ligne de code de production** (une
ligne de prose sur une page statique). La boucle est close en 3 passes.

### Ce que cette boucle apprend, au-delà de la story

1. **Une passe peut payer sans avoir raison.** La passe 2 s'est trompée sur ses
   trois findings, et c'est en les réfutant qu'un trou réel est apparu — la garde
   `WHERE actor_label = ''` n'était exercée nulle part. *Vérifier un faux positif
   coûte peu et rapporte parfois plus que le finding lui-même.*
2. **Déclarer un axe non exercé et en tirer un finding est contradictoire.** La
   passe 2 l'a fait sur l'axe « manuels ». Le prompt de la passe 3 l'a interdit
   explicitement, et la passe 3 a exercé l'axe pour de bon.
3. **Un grep de propagation doit partir des FICHIERS à couvrir, pas des mots à
   trouver.** Cinq fois sur cette seule story, le mot-clé trop étroit a laissé
   passer un site : synonymes manqués (×4), puis la **langue du support**.
4. **Une remise à zéro de base qui échoue est indiscernable d'une régression.**
   Le premier gate a rendu 34 faux échecs parce qu'un `sqlx migrate run` avait
   échoué en silence. La remise à zéro doit être **vérifiée**, pas seulement
   exécutée.


---

## ✅ Gate E2E de clôture — 2026-09-11, 16:23 UTC

**214 passés / 19 skipped / 9 échoués, en 8 min 42 s**, montage complet de
`docs/testing.md` § *Prérequis Playwright local*, base `kesh_e2e` **détruite et
reconstruite** (67 migrations, dont `20260910000001_audit_log_actor_label`), et
`/health` contrôlé **avant** le lancement : `"smtpConfigured":true`.

**Jugement fichier par fichier contre `docs/testing.md` § *Les échecs attendus***
— jamais au nombre :

| Échec | Verdict |
|---|---|
| `mode-expert.spec.ts:26` et `:41` | **KF-029 (#97)** |
| `onboarding-path-b.spec.ts:65` et `:92` | **KF-029 (#97)** |
| `onboarding.spec.ts:57`, `:77`, `:150` | **KF-029 (#97)** |
| `products.spec.ts:166` | pollution d'état — **passe rejoué seul** |
| `sidebar-navigation.spec.ts:75` | pollution d'état — **passe rejoué seul** |

Compte attendu `7 + 0 (run d'après-midi) + (0 à 1 KF-046) + 1 à 2 de pollution`
= **8 à 10** ; observé **9**. ⇒ **Aucun échec surnuméraire, aucune régression.**

**Trois points qui se lèvent d'eux-mêmes** :

1. **`reminders.spec.ts:146` est VERT.** Il avait échoué dans les deux runs du
   matin pour **deux raisons différentes** — montage incomplet, puis backend
   arrêté — sans jamais tourner dans des conditions saines. La réserve du point
   de reprise tombe : il n'était pas un échec connu, il n'était pas un échec.
2. **La KF-045 (#421) ne s'est pas déclenchée**, `invoices.spec.ts:405` et `:429`
   passent — conforme à un run lancé **après 12:00 UTC**.
3. **La KF-046 (#424) non plus.** Elle est *déterministe* et **échoue rejouée
   seule** ; or `sidebar-navigation.spec.ts:75` **passe** rejoué seul, ce qui
   l'écarte au profit de la pollution. ⚠️ *Le même test porte deux causes
   distinctes selon le contexte : la liste nominative ne suffit pas, il faut le
   rejeu isolé pour trancher entre elles.*

⚠️ **La tranche alphabétique ≥ `reminders` — celle que le run interrompu du matin
n'avait jamais couverte — est donc exercée et verte**, à l'exception des deux
pollutions ci-dessus, toutes deux postérieures à `reminders` dans l'ordre
alphabétique et toutes deux réfutées par le rejeu isolé.

**Les trois gates de la story sont désormais verts** : backend **2307/2307**,
frontend **740/740**, E2E **214 passés, 9 échecs tous attendus**.
