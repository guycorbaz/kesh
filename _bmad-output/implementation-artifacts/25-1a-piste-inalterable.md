# Story 25.1a : Rendre la piste de contrôle inaltérable — et cesser de promettre ce qui est faux

Status: ready-for-dev

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

9. ⛔ **`admin-manual.tex:1951` ET `marketing-brochure.tex:139` NE PEUVENT PAS ÊTRE RÉTABLIS PAR
   CETTE STORY.** *(Énoncé volontairement SANS chiffre — cf. l'avertissement de l'item 10.)* La brochure et
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

- [ ] **T1 — Trancher le mécanisme du restore** (AC 1, 2) : lire `backup.rs:382-460`,
      `import.rs:112-140`, et les tests `admin_full_import_e2e.rs:330-346` /
      `admin_backup_e2e.rs:263-277` qui **encodent le comportement actuel**. Écrire la décision et
      son motif dans les Dev Notes **avant** de coder.
- [ ] **T2 — Implémenter** (AC 1, 2) + tests, dont un prouvant qu'un backup **existant** reste
      importable et que l'export porte toujours `audit_log`.
- [ ] **T3 — Fermer `reset_demo`** (AC 3, 4).
- [ ] **T4 — NOMMER le chemin de test** (AC 5), sans alternative. ⛔ **Ne PAS le fermer** :
      `truncate_all` sert `/api/v1/_test/seed` et `/reset`, dont dépend **tout le montage de la
      suite E2E** (`frontend/tests/e2e/helpers/test-state.ts:83`). Le fermer casserait une suite
      que la CI n'exécute pas — personne ne le verrait avant le gate local. *P2-5.*
- [ ] **T5 — Mutation** (AC 6) sur chaque garde neuve, **plus un test POSITIF de refus** :
      ⛔ **la mutation ne détecte pas une garde JAMAIS POSÉE.** `lib.rs:300-306` documente le
      piège : *« une route chaînée après le `route_layer` COMPILE, ne panique pas, et échappe aux
      DEUX couches »*. Un test « non-Admin ⇒ 403 » le voit ; une mutation, non. *P2-6.*
- [ ] **T6 — Les SIX sites à traiter + l'en-tête du module** (AC 7 à 11), PDF régénérés (AC 12),
      et le README vérifié.
- [ ] **T7 — SI le mécanisme de T1 touche le schéma** : garde-fous **P2-bis** (bump Cargo
      solidaire), **P3** (bump `min_required` si breaking), **P5** (ligne d'audit d'idempotence +
      recompte des deux totaux et des trois compteurs), **P6** (`grep` des sites positionnels),
      **P7** (triage au registre de rejeu). *P2-4.*
- [ ] **T8 — Gate complet**, base remise à zéro. ⛔ `kesh-db` touché : **ciblage interdit**.

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

### Debug Log References

### Completion Notes List

### File List
