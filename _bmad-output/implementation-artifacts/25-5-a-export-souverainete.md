# Story 25.5-a : L'export de souveraineté porte enfin la comptabilité — et une garde l'y tient

Status: in-progress

**Issue : [#386]**, qu'elle **ferme** : `closes #386` dans le **titre ET le corps** de sa PR.

⛔ **Issue du DÉCOUPAGE de la 25-5** (arbitrage de Guy, 2026-09-23). La 25-5 portait **[#385]** (la
Balance qui ne concorde pas avec le Bilan) **et [#386]**. Les deux n'ont en commun que le mot
« état » : l'une corrige un **rapport** (`kesh-report`), l'autre complète un **export** et pose une
garde (`kesh-api/exports`, `kesh-db/repositories`). ⇒ **[#385] reste à la 25-5-b.**

## Story

En tant que **personne qui tient ses livres dans Kesh**,
je veux que **l'export CSV porte réellement ma comptabilité**,
afin de **pouvoir partir** — changer de logiciel, ou répondre à un contrôle — sans découvrir que la
moitié de mes pièces n'est pas dans le fichier.

## Ce que l'audit a trouvé, et ce que la vérification a trouvé de plus

L'issue annonce **dix** tables absentes. Il y en a **vingt et une**.

⚠️ *La première rédaction de cette fiche écrivait « neuf » — recompté sur le corps de l'issue : son
tableau porte bien **dix** noms. Un décompte non recompté depuis la source, dans la fiche même qui
en dénonce trois autres.*

⛔ **Et le trou s'est creusé DEPUIS l'écriture de l'issue** : `invoice_settlements` — les règlements,
livrés par l'Epic 24 — n'y figurait pas, parce qu'elle n'existait pas encore. *C'est exactement le
motif que cette story doit fermer : l'export se périme à chaque epic, en silence.*

**Établi le 2026-09-23, à ne pas re-dériver** :

| grandeur | valeur | comment |
|---|---|---|
| tables exportées | **19** | `push_csv!` comptés sur le source **aplati** |
| `TABLES_TO_TRUNCATE` | **39** | `crates/kesh-db/src/backup.rs` |
| absentes de l'export | **21** | différence **par NOM**, non par soustraction |

⛔ **Et 39 − 19 ne fait pas 21.** La différence se calcule **sur les noms**, pas sur les cardinaux :
`company.csv` est exportée mais ne s'appelle pas `companies`, et ne figure donc pas au registre.
*Lire « différence des deux » comme une soustraction donne 20 et fait chercher une table fantôme —
y compris à qui écrira la garde de l'AC 3.*

⚠️ **Le décompte se fait sur le source APLATI.** Les `push_csv!` sont multi-lignes ; un `grep`
ligne-à-ligne n'en rend que **6** sur 19. *C'est le précédent de `INSERT INTO … SELECT` du
`CLAUDE.md`, rencontré une fois de plus en préparant cette fiche.*

⚠️ **Le commentaire `// -------- 1. Queries 18 tables` (`global.rs:121`) dit DÉJÀ faux** — il y en a
19. Un décompte de plus à recompter, pas à recopier.

## Acceptance Criteria

1. **Onze tables comptables entrent dans l'export** — arbitrage de Guy : *« l'export CSV pour migrer
   vers une autre application n'a besoin que des données comptables ; seuls les backups de Kesh pour
   restaurer dans Kesh ont besoin de toutes les informations »* :
   `credit_notes`, `credit_note_lines`, `supplier_invoices`, `supplier_invoice_lines`,
   `payment_batches`, `payment_batch_items`, `invoice_settlements`, `projects`,
   `imported_supplier_invoices`, `contact_persons`, `audit_log`.

2. ⛔ **DIX tables restent dehors, et chacune porte sa justification ÉCRITE** — c'est le cœur de
   l'AC 3, pas une note de bas de page :

   | exclue | pourquoi |
   |---|---|
   | `invoice_number_sequences`, `credit_note_number_sequences`, `journal_entry_number_sequences` | compteurs **internes à Kesh** : les numéros sont déjà portés par les pièces exportées |
   | `api_keys`, `refresh_tokens`, `password_reset_tokens` | ⛔ **secrets** — les exporter dans un fichier destiné à être transmis serait une **fuite** |
   | `users` | l'audit porte le **nom de son auteur en instantané** depuis la 25-1a : rien ne se perd |
   | ~~`companies`~~ | ⚠️ **erreur de la fiche, corrigée au développement** : `companies` n'est pas exclue, elle est **exportée** sous `company.csv` (au singulier). La partition réelle est **11 + 9** (Dev Agent Record, Debug Log). *Barrée ici plutôt qu'effacée, à la revue P1 : le corps de l'AC contredisait sa propre correction.* |
   | `email_templates`, `onboarding_state` | **configuration**, non comptabilité |

3. ⛔ **LA GARDE D'EXHAUSTIVITÉ — c'est la vraie livraison de cette story.** Un test échoue dès
   qu'une table de `TABLES_TO_TRUNCATE` n'est **ni exportée, ni inscrite** à une liste d'exclusions
   **avec un motif non vide**.

   **Pourquoi c'est elle qui compte, et non les onze exports** : l'export a été écrit à la
   Story 9-2b, étendu une fois à l'Epic 21, et **jamais rattrapé** sur les Epics 12 (fournisseurs,
   avoirs), 19 (projets) ni 24 (règlements). *Trois epics ont rouvert le trou sans que rien ne
   rougisse.* Ajouter onze exports sans la garde, c'est réparer une fois ce qui se recassera au
   prochain epic.

   ⛔ **Elle s'écrit sur le patron de D4-ter — inventorier les sites qui NE RÉSOLVENT PAS**, et non
   énumérer les tables traitées. Le modèle existe à deux endroits du dépôt :
   `backup_inventory_matches_schema` (`backup.rs:626`, qui compare à `information_schema`) et les
   `EXEMPT_MIGRATIONS` de **P7** (`post_restore.rs`, motif écrit obligatoire).

   ⚠️ **Où la poser** : les deux patrons cités vivent dans `kesh-db`, mais le code sous test
   (`build_global_export`) vit dans `kesh-api` — qui dépend de `kesh-db`, donc `TABLES_TO_TRUNCATE`
   (public) y est accessible. La garde va donc **du côté de `kesh-api`**, au plus près de ce
   qu'elle mesure.

   ⚠️ **L'assertion ne doit PAS tirer ses deux membres de la même source** — c'est l'`AC7` de
   l'Epic 23, verte *par construction*. Un côté vient de `TABLES_TO_TRUNCATE`, l'autre de ce que
   l'export **produit réellement**.

4. **Chaque table exportée a son sérialiseur CSV** dans `exports/csv_tables.rs`, avec l'entrée
   correspondante au `metadata.json` (hash SHA-256, comme les 19 existantes).

4-bis. ⛔ **LE NOMBRE 19 EST CODÉ EN DUR À SEPT ENDROITS, ET DEUX FONT PANIQUER LES TESTS.**
   C'est le piège le plus concret de cette story : un développeur qui ajoute onze `push_csv!` et
   lance le gate voit rougir des choses qui n'ont rien à voir avec son travail. Les sites, relevés
   un par un — *ne pas se fier à cette liste sans la regreper, elle se périmera* :

   | fichier:ligne | forme | effet si oublié |
   |---|---|---|
   | `global.rs:286` | `debug_assert_eq!(files.len(), 19, …)` | ⛔ **panique** en debug et en test |
   | `global.rs:287` | `debug_assert_eq!(tables_meta.len(), 19, …)` | ⛔ **panique** |
   | `global.rs:302` | `csv_count: 19` — **littéral, pas un commentaire** | le manifeste **ment** : il annonce 19 CSV pour 30 |
   | `global.rs:196` | `Vec::with_capacity(20)` | sans effet fonctionnel, à ajuster |
   | `global.rs:15`, `:123` (et l'en-tête) | commentaires « 18 »/« 19 » | ⚠️ `:123` dit **déjà faux** (« Queries 18 tables » pour 19) |
   | `csv_tables.rs:1` | « Story 9-2b — **16 fonctions** » | déjà faux : il y en a **19** |
   | `tests/exports_global_e2e.rs:811`, `:1178` | `assert_eq!(…, 19)` | le gate rougit |

   ⛔ **Le geste attendu n'est pas de mettre à jour sept sites, c'est de rendre le nombre
   DÉRIVÉ** — il se compte depuis ce que l'export produit, et cesse d'être écrit à la main. Sinon
   la prochaine table rouvrira les sept. *Un compteur qu'aucun calcul ne tient se périme en
   silence ; c'est le motif de la garde de l'AC 3, appliqué au nombre lui-même.*

5. ⚠️ **Neuf fonctions de repository sont à ÉCRIRE** — c'est le vrai coût de cette story, et il
   n'est pas dans l'issue. Sur les huit repositories concernés, **six n'ont aucune fonction de liste
   exhaustive scopée par société** :

   | repository | fonction de lecture exhaustive ? |
   |---|---|
   | `projects` | **oui** — `list_by_company`, sans borne |
   | `audit_log` | ⛔ **NON, et le piège est là** — voir ci-dessous |
   | `credit_notes`, `supplier_invoices`, `payment_batches`, `invoice_settlements`, `imported_supplier_invoices`, `contact_persons` | **non — à écrire** |

   ⛔ **`audit_log` n'a AUCUN dump exhaustif, et ses deux fonctions sont des pièges** :
   `list_by_company_paginated` est paginée (`MAX_LIMIT = 200`), et `list_for_export` prend un
   `max_rows` **fourni par l'appelant**. La route qui l'utilise aujourd'hui
   (`routes/audit_log.rs:319`) lui passe `MAX_EXPORT_ROWS + 1` et **rejette la requête entière**
   en `ResultTooLarge` au-delà de **10 000** lignes.

   ⚠️ **Réutiliser ce chemin tel quel casserait l'export de souveraineté d'une société active** :
   soit il échouerait en entier, soit il tronquerait la piste d'audit **sans le dire** — le défaut
   muet que cette story existe précisément pour fermer, appliqué à la table que l'AC 6 traite avec
   le plus de soin.

   ⇒ **Trancher et l'écrire au Dev Agent Record** : un dump réellement non borné, ou une
   troncature **signalée dans le `metadata.json`** sur le patron de `invoices::list_for_export`,
   qui rend un drapeau `truncated` (`routes/invoices.rs:1260`). ⛔ **Ce qui est interdit, c'est de
   tronquer en silence.**

   Les tables **enfants** (`credit_note_lines`, `supplier_invoice_lines`, `payment_batch_items`)
   suivent le patron de `invoices::list_all_lines_by_company`.

   ⛔ **Scoping multi-tenant obligatoire** : chaque lecture est filtrée par `company_id`, y compris
   pour les tables enfants (par jointure sur le parent). *C'est la règle de la 7-1 ; une fonction
   d'export non scopée exfiltrerait les données d'une autre société.*

6. **`audit_log` est exportée, mais elle est scopée** : la colonne `company_id` existe depuis la
   25-1c-zero. ⚠️ **Le filtre est strict** — les entrées sans société ne sortent pas : c'est
   l'arbitrage de Guy du 2026-09-15, *« si on importe une sauvegarde, c'est que la base a disparu »*.

7. **L'écran d'export retire son avertissement, et le retire VRAIMENT.**
   `frontend/src/routes/(app)/export/+page.svelte` porte trois textes à reprendre :
   - `:67` — « ⚠️ Il ne couvre pas encore l'ensemble de votre comptabilité : lisez ci-dessous ce
     qu'il ne contient pas **avant de compter dessus pour migrer vers un autre logiciel** » ;
   - `export-global-content-includes` — la liste de ce qu'il contient, à compléter ;
   - `export-global-content-excludes` — la liste de ce qu'il **ne** contient **pas**, qui doit
     désormais énoncer les **dix exclusions de l'AC 2 avec leur motif**, et rien d'autre.

   ⚠️ **Les trois clés existent dans les quatre locales** : les réécrire toutes les quatre, replis
   en dur compris. ⛔ **Et vérifier le repli contre le FTL, mot pour mot** — la 25-2-b-2 a livré
   deux replis divergents de leur traduction, dont un qui posait une question là où le catalogue
   affirmait.

   ⛔ **Un de ces trois replis diverge DÉJÀ**, avant toute modification : celui de
   `export-global-content-includes` (`+page.svelte:76`) omet cinq éléments que porte son fr-CH
   (`messages.ftl:1253`) — exercices, historique des imports bancaires, taux de TVA, paramètres de
   facturation, profils d'import. *Le défaut que cet AC met en garde d'introduire est là avant
   lui.*

8. **Les manuels — et deux sites y sont DÉJÀ faux, avant cette story.**
   ⛔ `user-manual.tex:1583-1592` (« Contenu du ZIP ») annonce
   `\item \texttt{reports/} : tous les rapports PDF générés (bilan, P&L, balance, journal)` —
   **ce dossier n'existe pas** : `build_global_export` ne produit aucun PDF. *Le manuel promet un
   contenu inventé.* La même liste omet par ailleurs onze des dix-neuf tables déjà exportées.
   ⛔ `admin-manual.tex:1946` écrit « **19 tables sur 38** » : le registre en compte **39**, et la
   réserve entière tombe une fois l'AC 1 livré — `audit_log` voyagera avec l'export. **Retirer,
   non corriger.**
   ⚠️ **Un AC générique — « les manuels décrivent l'export » — n'attrape pas un contenu inventé** :
   c'est pourquoi ces deux sites sont nommés ici. ⛔ **Contrôler le PDF APLATI**
   (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`), l'apostrophe y étant U+2019.

8-bis. ⛔ **L'INJECTION DE FORMULE — la garde existe déjà, et cet export ne l'emploie pas.**
   `csv_sanitize` (`crates/kesh-api/src/util.rs:220`) préfixe les cellules commençant par `=`,
   `+`, `-` ou `@` ; elle a été **extraite** à la 25-1c-a précisément pour être réutilisée.
   `grep -c csv_sanitize` rend **0** dans `exports/csv_tables.rs` comme dans `exports/global.rs`.

   ⚠️ **Et cette story aggrave l'exposition** : elle ajoute des colonnes de **texte libre**
   — noms et descriptions de projets, personnes de contact, libellés d'avoirs et de factures
   fournisseurs, `actor_label` et `details_json` de la piste d'audit — à un fichier dont l'écran
   dit lui-même qu'on l'ouvrira dans un tableur pour migrer.

   ⇒ **Trancher, et l'écrire** : appliquer `csv_sanitize` aux colonnes de texte des tables
   ajoutées (a minima), ou **assumer le risque par écrit** au Dev Agent Record. ⛔ *Ce qui est
   interdit, c'est de ne pas se poser la question* — l'omission est antérieure à cette story, mais
   c'est elle qui élargit la surface.

9. **`CHANGELOG.md`** — entrée dans `## [0.12.1] — Non publié`, **créée par la 25-2-b-2**. ⛔ **Aucune
   ligne d'une section publiée ne se réécrit** — l'interdit porte sur la classe, pas sur un nombre.

10. **Tests** : un par table ajoutée (la table non vide **sort** dans le CSV), le scoping
    multi-tenant (une seconde société ne fuit pas), la garde d'exhaustivité **prouvée par mutation**
    — retirer une table de l'export sans l'exclure doit faire rougir —, et le `metadata.json`.

11. **Gate complet** — la story touche `kesh-db`, `kesh-api` et le frontend : gate backend complet,
    gate frontend, E2E complète avant le push.

## Tasks / Subtasks

- [x] **T1 — Les lectures** (AC 5, 6) : neuf fonctions de repository, scopées `company_id`.
- [x] **T2 — Les sérialiseurs et l'export** (AC 1, 4, **4-bis**) : onze entrées `csv_tables` +
      `push_csv!`, et le `metadata.json`.
- [x] **T3 — LA GARDE** (AC 2, 3) : liste d'exclusions à motif obligatoire, et le test qui refuse
      toute table ni exportée ni exclue. **Prouvée par mutation.**
- [x] **T4 — L'écran et les quatre locales** (AC 7).
- [x] **T4-bis — Les compteurs codés en dur** (AC 4-bis) : rendre le nombre **dérivé**, et non
      mettre à jour sept sites. Deux d'entre eux **paniquent** en test.
- [x] **T5 — Manuels, PDF, CHANGELOG** (AC 8, 8-bis, 9) — dont **deux sites de manuel déjà faux**
      (un dossier `reports/` inventé, un « 38 » pour 39) et l'arbitrage sur `csv_sanitize`.
- [x] **T6 — Gates complets** (AC 11), PR avec `closes #386`.

### Review Findings

*Revue de code **a posteriori** — la story a été mergée (PR #452) sans revue. Passe 1, 2026-09-26,
prompt `25-5-a-review-prompt-p1.md`.*

- [x] [Review][Patch] **HIGH** — Le manuel contredisait encore la story : la boîte « Ce que cet export couvre » disait qu'il ne contient pas les factures fournisseurs, les avoirs ni les projets [`docs/manual/fr/user-manual.tex:1670`]
- [x] [Review][Patch] **HIGH** — Test multi-sociétés : huit des onze tables neuves n'étaient vérifiées que **vides**, dont les trois enfants scopées par jointure [`crates/kesh-api/tests/exports_global_e2e.rs`]
- [x] [Review][Patch] **HIGH** — `creditor_line1` / `creditor_line2` absentes de `imported_supplier_invoices.csv` : une adresse « combinée » (type K) sortait sans adresse [`crates/kesh-api/src/exports/csv_tables.rs`]
- [x] [Review][Patch] **HIGH** — `sent_to` et `note` des rappels échappaient à `csv_sanitize` [`crates/kesh-api/src/exports/csv_tables.rs:843,846`]
- [x] [Review][Patch] **MEDIUM** — AC 4-bis : le doc-comment de module de `global.rs` portait encore 19 / 18 / 18 [`crates/kesh-api/src/exports/global.rs:4,247`]
- [x] [Review][Patch] **MEDIUM** — AC 11 : aucun gate déclaré au Dev Agent Record — gates exécutés et déclarés en revue
- [x] [Review][Patch] **MEDIUM** — CHANGELOG : « fait échouer la construction du logiciel » ; c'est un test qui échoue [`CHANGELOG.md:21`]
- [x] [Review][Patch] **MEDIUM** — Le commentaire du test de structure le disait « l'assertion la plus forte » alors qu'il compare l'export à sa propre source [`crates/kesh-api/tests/exports_global_e2e.rs`]
- [x] [Review][Patch] **MEDIUM** — L'AC 2 listait encore `companies` parmi les exclusions, contre sa propre correction
- [x] [Review][Patch] **LOW** — « 3 PDF » pour 2 ; « sept autres » pour huit ; « 17 entrées » dans l'en-tête et un intertitre du fichier de tests
- [x] [Review][Patch] **HIGH** (P2, orchestrateur) — Sept des dix-neuf tables d'origine omettaient des colonnes : noms et adresse structurée des contacts et de la société, formule d'appel, langue et numéro de client des contacts, `bank_accounts.archived`, `default_payable_account_id`, `credit_note_number_format` [`crates/kesh-api/src/exports/csv_tables.rs`] — colonnes ajoutées, **garde de colonnes** contre le schéma
- [x] [Review][Patch] **LOW** (P2) — Commentaire : une panique entre les deux `SET FOREIGN_KEY_CHECKS` est sans effet hors du test (« CRITICAL » de la lentille 1 reclassé)
- [x] [Review][Defer] **MEDIUM** (P2, hors story) — La QR-facture lit `country`, que rien n'écrit → **#466**
- [x] [Review][Defer] **MEDIUM** — L'export lit ses trente tables sans instantané commun [`crates/kesh-api/src/exports/global.rs`] — deferred, pre-existing → **#465**

## Dev Notes

### Ce que cette story ne fait pas

- ⛔ **Les justificatifs (les FICHIERS) n'y sont pas**, et c'est un arbitrage explicite de Guy. Ils
  ne sont dans **aucun** des deux mécanismes — ni l'export, ni la sauvegarde —, ce qui signifie
  qu'une restauration rend les écritures mais **pas les pièces qui les prouvent**. ⇒ **story
  séparée, sur la SAUVEGARDE.**
- Elle ne touche pas la sauvegarde `.keshbackup`, qui porte **déjà les 39 tables**.
- Elle ne traite pas **[#385]** (Balance vs Bilan) : 25-5-b.

### Les deux mécanismes, à ne pas confondre

*C'est la confusion qui a failli faire spécifier la mauvaise story.*

| | porte | pour |
|---|---|---|
| **export de souveraineté** (`exports/global.rs`, CSV) | 19 → **30** tables | **migrer** vers un autre logiciel |
| **sauvegarde administrateur** (`admin_backup`, NDJSON) | **39** tables | **restaurer Kesh** |

### Règle de splitting

Modules touchés : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` — **quatre**, sous le seuil de
cinq. ⚠️ **Le signal à surveiller** : l'AC 5 (neuf fonctions de repository) est le volet le plus
lourd ; si une passe de validation le fait grossir, le sortir en story-zéro plutôt que d'élargir
celle-ci.

### References

- [Source: crates/kesh-api/src/exports/global.rs] — `build_global_export`, le macro `push_csv!`.
- [Source: crates/kesh-db/src/backup.rs:34] — `TABLES_TO_TRUNCATE` ; **:626** —
  `backup_inventory_matches_schema`, le patron de la garde.
- [Source: crates/kesh-db/src/post_restore.rs] — `EXEMPT_MIGRATIONS`, le patron du motif écrit (P7).
- `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS* (D4-ter), § *Recompter ses propres comptes
  rendus*, § *Le prompt d'une passe doit NOMMER le manuel*.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — implémentation.

### Les deux arbitrages que la fiche laissait ouverts, et comment ils ont été tranchés

1. **`csv_sanitize`** : **appliquée**, non assumée. La fonction existait — extraite
   par la 25-1c-a *précisément pour être réutilisée* — et cet export ne l'employait
   nulle part. **Trente cellules** des dix-neuf tables déjà exportées partaient
   brutes. Fermé **avant** d'ajouter les onze tables, qui sont pleines de texte libre.
2. **`audit_log`** : **troisième fonction de lecture**, non réutilisation de
   `list_for_export`. Celle-ci rejette la requête au-delà de 10 000 lignes — juste
   pour un écran, faux pour la souveraineté. *Une piste d'audit tronquée dans le
   fichier même qui prouve ce qui s'est passé aurait l'air d'un export complet.*

### Debug Log References

- ⛔ **La garde m'a pris en défaut à sa première exécution** : `companies` figurait
  dans les **deux** registres. Elle n'est pas exclue — elle est exportée sous
  `company.csv`, au singulier. ⇒ **la fiche se trompait** : 20 tables absentes et non
  21, partition **11 + 9** et non 11 + 10. *Une garde qui corrige la spécification
  qui l'a demandée.*
- ⚠️ **La mutation « retirer un `push_csv!` » échappait aux tests unitaires** — le
  `debug_assert` vit dans une fonction qui exige une base. C'est le E2E **dérivé du
  registre** qui la tue. La garde unitaire seule aurait été rassurante et incomplète.
- ⚠️ Le montage du test multi-tenant a d'abord rougi pour rien : `chk_contacts_type`
  n'admet que `Personne` ou `Entreprise`.

### Completion Notes List

- **Trois passages du manuel étaient FAUX, deux avant cette story** : un dossier
  `reports/` de rapports PDF **qui n'existe pas** ; un écran de sélection
  (exercices, formats) alors qu'il n'y a **qu'un bouton** ; et ⛔ une commande de
  vérification d'intégrité lisant `.sha256_per_file[…]`, **clé inexistante** —
  *un lecteur qui la copiait obtenait « CORROMPU » sur chaque fichier de son
  archive.*
- ⚠️ **Signalé, hors périmètre** : quatre lignes `projects` (`P19-2-*`) subsistent
  dans la base de gate partagée, laissées par des tests de l'**Epic 19**. Mon test
  utilise une base éphémère.

### File List

| Fichier | Nature |
|---|---|
| `crates/kesh-db/src/repositories/{credit_notes,supplier_invoices,payment_batches,invoice_settlements,contact_persons,imported_supplier_invoices,audit_log}.rs` | **10** lectures exhaustives scopées |
| `crates/kesh-api/src/exports/csv_tables.rs` | **11** sérialiseurs (19 → 30), helper `txt` + 30 cellules sanitisées |
| `crates/kesh-api/src/exports/global.rs` | `TABLES_EXPORTEES` (30), `TABLES_HORS_EXPORT` (9), **3 gardes**, nombre dérivé |
| `crates/kesh-api/tests/exports_global_e2e.rs` | 2 tests dérivés du registre, 1 retargeté, 1 renommé, **1 neuf** (multi-tenant) |
| `crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl` | 3 textes réécrits × 4 |
| `frontend/src/routes/(app)/export/+page.svelte` | replis alignés mot pour mot |
| `docs/manual/fr/{user,admin}-manual.tex` + **2 PDF** (« 3 » corrigé en revue P1 : `git show --stat 91a20d76 \| grep -c '\.pdf'` rend 2) | section refondue, réserve levée |
| `CHANGELOG.md` | 3 entrées dans `[0.12.1]`, **12 lignes ajoutées / 0 supprimée** |

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Date | Étape | Note |
|---|---|---|
| 2026-09-26 | review P2 | **Trois lentilles Haiku 4.5** en contexte frais, diffs **aplatis** (story mergée + remédiation P1), prompt versionné `25-5-a-review-prompt-p2.md`, axes déclarés. **Lentilles : 0 au-dessus de LOW** — Blind : un « CRITICAL » reclassé LOW (une panique entre les deux `SET FOREIGN_KEY_CHECKS` rendrait la connexion sans contrôle des clés, mais base et pool sont propres au test `#[sqlx::test]`) et un MEDIUM ramené à LOW (test d'en-tête sans ligne, désormais couvert par la garde de colonnes) ; Auditor : 0, et « 34 » à préciser (fait) ; Edge : 0 — ⚠️ **mais axe 3 non exercé** (colonnes des trente tables : deux vérifiées, et un argument de repli **faux** : un écart en-tête / valeurs ne se voit pas à la compilation). ⛔ **Repris par l'orchestrateur, comme la règle l'exige — et c'est là qu'était le défaut** : un script confrontant les en-têtes des trente sérialiseurs au schéma de `test-schema/` a trouvé **sept tables d'origine incomplètes** — noms et adresse structurée des contacts et de la société, formule d'appel, langue et numéro de client, `bank_accounts.archived`, `default_payable_account_id`, `credit_note_number_format`. **HIGH** : l'export promettait de quoi migrer. Colonnes ajoutées ; ⛔ **garde de colonnes** `chaque_colonne_du_schema_est_exportee_ou_ecartee` (source indépendante : le schéma, que `test_schema_guard.rs` tient égal au réel), exemptions **motivées** (`COLONNES_HORS_EXPORT` : trois colonnes générées, une forme canonique, et `country` ×2) ; **mutation tuée** (`bank_accounts.archived` retirée ⇒ la garde la nomme). ⚠️ Le motif de `country` a d'abord été écrit « ni écrit ni lu » — **faux** : la QR-facture le lit (`invoice_pdf_service.rs:135-136`) alors que rien ne l'écrit ⇒ **#466** ouverte, motif corrigé. CHANGELOG : entrée *Fixed* (ces colonnes manquaient déjà à la v0.12.0 publiée). **Gate ciblé** : `fmt` + `clippy` verts ; `binary(exports_global_e2e) \| test(/csv_tables/)` **37/37** (23 E2E + 14 unitaires). Gate complet au push. Remédiation dans du code de production ⇒ **boucle non close**, passe 3. |
| 2026-09-26 | review P1 | **Revue de code A POSTERIORI** : la story avait été mergée (PR #452) sans revue ni gate déclaré. Branche `story/25-5-a-revue-code`. **Trois lentilles Sonnet** en contexte frais (auteur : Opus 5), prompt versionné `25-5-a-review-prompt-p1.md`, axes déclarés par chacune. **4 HIGH, 5 MEDIUM, LOW divers, 1 report** — chaque HIGH vérifié par `grep -nF` avant d'être retenu. ① Manuel : la boîte « Ce que cet export couvre » disait encore l'inverse de la story (factures fournisseurs, avoirs, projets absents) → réécrite, PDF régénéré et **contrôlé aplati** (0 occurrence de l'ancienne phrase). ② Test multi-sociétés : huit des onze tables neuves n'étaient vérifiées que **vides** → peuplées pour deux sociétés (SQL direct, clés étrangères suspendues sur une connexion dédiée, identifiants fictifs distincts par société — `uq_credit_notes_invoice` l'a exigé au premier run), douze tables assertées présence A / absence B ; **mutation tuée** : filtre de société retiré de `credit_notes::list_all_lines_by_company` ⇒ « FUITE MULTI-TENANT : `credit_note_lines.csv` ». ③ `creditor_line1` / `creditor_line2` absentes de `imported_supplier_invoices.csv` — une adresse de type K sortait **sans adresse** → ajoutées (27 colonnes sur 27 du schéma) ; test d'en-tête. ④ `sent_to` et `note` des rappels échappaient à `csv_sanitize` → `fmt_opt_str` ; test unitaire, **mutation tuée** (note remise brute ⇒ rouge). MEDIUM : doc-comment de `global.rs` (19 / 18 / 18) réécrit sans nombre ; CHANGELOG « construction du logiciel » → « tests » ; commentaire « assertion la plus forte » rendu exact (le test compare à sa propre source, c'est la garde unitaire qui confronte à `TABLES_TO_TRUNCATE`) ; `companies` barrée dans l'AC 2. LOW : « 3 PDF » → 2, « sept » → huit (disparu avec la réécriture), « 17 entrées » dans le fichier de tests. **Écarté** : `SELECT *` d'`imported_supplier_invoices::list_all_by_company` — c'est la convention de tout ce module (`:125`). **Reporté** : lectures sans instantané commun, défaut antérieur → **#465**. **Gate ciblé** : `fmt` + `clippy --workspace -D warnings` verts ; `nextest` `binary(exports_global_e2e) | test(/csv_tables/)` **34/34** (23 E2E + 11 unitaires de `csv_tables`, **avant** l'ajout des deux tests neufs — périmètre précisé en revue P2, une lentille ayant recompté 37), puis les 2 tests unitaires neufs verts. ⚠️ **Gate complet NON exécuté à ce stade** — au push. Aucun fichier `kesh-db` touché par la remédiation. |
| 2026-09-23 | validate P1 | **Passe 1, une lentille Sonnet en contexte frais**, checklist du workflow. **0 CRITICAL, 4 HIGH, 3 MEDIUM, 3 LOW** — tous vérifiés depuis la source avant d'être retenus. ⛔ **H1 — la fiche écrivait « neuf » tables annoncées par l'issue ; son tableau en porte DIX.** *Un décompte non recompté, dans la fiche même qui en dénonce trois autres.* ⛔ **H2/H3 — le nombre 19 est codé en dur à SEPT endroits**, dont deux `debug_assert_eq!` qui **paniquent en test** et un `csv_count: 19` **littéral** qui ferait mentir le manifeste ; plus deux assertions de test et l'en-tête de `csv_tables.rs`, qui annonce « 16 fonctions » pour 19. La tâche ne prescrivait que **le commentaire**. ⇒ AC 4-bis, et le geste attendu devient *rendre le nombre dérivé*, non corriger sept sites. ⛔ **H4 — « `audit_log` : oui » était FAUX et dangereux** : ses deux fonctions sont bornées, et la route qui les emploie **rejette la requête entière** au-delà de 10 000 lignes. Réutiliser ce chemin ferait échouer l'export d'une société active, ou **tronquer la piste d'audit en silence** — le défaut muet que cette story existe pour fermer. **MEDIUM** : le manuel utilisateur promet un dossier `reports/` **qui n'existe pas** — un AC générique ne peut pas attraper un contenu *inventé*, d'où deux sites nommés ; le manuel admin écrit « 38 » pour 39 et sa réserve entière tombe avec la livraison ; et ⛔ **`csv_sanitize` EXISTE** (`util.rs:220`, extraite à la 25-1c-a) **et cet export ne l'emploie nulle part**, alors que la story y ajoute des colonnes de texte libre. **LOW** : « différence des deux » invite à faire 39 − 19 = 20 — la différence se calcule **par NOM** ; la crate d'accueil de la garde n'était pas dite ; et un des trois replis de l'écran **diverge déjà** de son FTL. Vérifié exact et à ne pas refaire : 19 / 39 / 21, la partition 11 + 10 sans recouvrement, les six repositories sans lecture exhaustive, l'absence de `company_id` sur les trois tables enfants, `actor_label` comme instantané (migration `20260910000001`), `company.csv`, les trois clés dans les quatre locales, et la section `[0.12.1]` du CHANGELOG. |
| 2026-09-23 | spec | Story née du **découpage de la 25-5** (arbitrage de Guy) : [#385] et [#386] n'ont en commun que le mot « état ». ⛔ **Trois faits établis avant rédaction, et chacun corrige l'issue** : (1) **21** tables absentes et non 10 — et « neuf » écrit d'abord ici même, recompté depuis le corps de l'issue ; (2) le trou **s'est creusé depuis l'écriture de l'issue** — `invoice_settlements`, livrée par l'Epic 24 ; (3) **six repositories sur huit n'ont aucune fonction de liste exhaustive**, ce qui est le vrai coût de la story et ne figure nulle part dans l'issue. ⚠️ Le décompte des `push_csv!` **ne se fait que sur le source aplati** : un grep ligne-à-ligne en rend 6 sur 19 — *le précédent `INSERT INTO … SELECT`, rencontré en préparant cette fiche*. ⚠️ Et le commentaire `// Queries 18 tables` du fichier dit **déjà faux**. Arbitrage de Guy sur le périmètre : *« l'export CSV n'a besoin que des données comptables ; seuls les backups ont besoin de tout »* ⇒ 11 dedans, 10 dehors **avec motif écrit**, les justificatifs en story séparée sur la **sauvegarde**. |
