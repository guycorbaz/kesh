# Story 33-1a : le numéro interne des factures fournisseur — colonne, format, attribution, numérotation de l'existant (#926)

Status: ready-for-dev

*Fiche rédigée le 2026-11-06, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche
`story/33-0-kickoff`, numéros de ligne de `4ef19901`. Registre : `e33-choix-autonomes-e331.md`, choix `C-e331-1` et suivants.
Clé courte `e331`. Décisions du Project Lead : `epic-33-numerotation-et-defauts.md` § *Décisions de Guy* et les
commentaires de #926 ; elles font foi. **Seule migration de l'Epic 33.** La 33-1b (écrans) suit cette story.*

## Story

En tant que comptable qui classe ses factures papier,
je veux que chaque facture fournisseur validée reçoive de Kesh un numéro unique et lisible,
afin de retrouver le papier sans dépendre du numéro du fournisseur, facultatif et parfois identique d'un fournisseur à l'autre.

## Ce qui est établi au code

| # | Fait | Source (à `4ef19901`) |
|---|---|---|
| F1 | Le seul numéro d'une facture fournisseur est celui du fournisseur, `VARCHAR(64) NULL`, sans unicité | `20260628000001_supplier_invoices.sql:31`, `supplier_invoices.rs:41` |
| F2 | **Une table neuve rendrait toute sauvegarde 0.17.4 inimportable** : le manifeste doit porter exactement `TABLES_TO_TRUNCATE`, dans les deux sens. Elle refermerait aussi la fenêtre d'importabilité, donc interdirait l'entrée P7 de cette migration | `admin_backup/import.rs:250-263`, `post_restore.rs:21-27` |
| F3 | Une colonne neuve nullable ou pourvue d'un `DEFAULT` n'est pas exigée d'une sauvegarde ancienne ; une colonne inconnue de la destination est refusée — donc une sauvegarde 0.17.5 est refusée par une 0.17.4 | `admin_backup/import.rs:613-627` |
| F4 | `validate_in_tx` tient déjà, pour toute validation, la ligne `company_invoice_settings` `FOR UPDATE` (2 bis) et la ligne `fiscal_years` `FOR UPDATE` (3) | `supplier_invoices.rs:1421`, `:1488` ; `company_invoice_settings.rs:107-114` ; `fiscal_years.rs:758` |
| F5 | Une validation non brouillon a toujours une écriture d'achat et son exercice : `chk_supplier_invoices_entry_iff_not_draft`, `journal_entries.fiscal_year_id NOT NULL`, clés `RESTRICT` | `20261031000001_supplier_invoices_draft.sql:44`, `20260412000001_journal_entries.sql:19,28` |
| F6 | La route de validation est rejouée sur interblocage | `routes/supplier_invoices.rs:870` |
| F7 | Deux seuls libellés à changer : achat et règlement | `supplier_invoices.rs:1511-1512`, `:1909` |
| F8 | `render` existe et rend `{YEAR}` depuis l'année de `start_date` ; il refuse un nom d'exercice à accolade **même si le format n'a pas `{FY}`** | `kesh_core::invoice_format::render` (`invoice_format.rs:206,215`) ; `credit_notes.rs:559-572`, `invoices.rs:2305-2322` |
| F9 | `validate_template` **accepte un format sans `{SEQ}`** : `F-{YEAR}` passe, et tous les numéros seraient identiques. Il borne le pire rendu à 64 : sans `{FY}`, un format validé se rend toujours | `invoice_format.rs:151-199` (seul `has_placeholder` est exigé) |
| F10 | Une facture cliente dévalidée reprend son numéro sans toucher au compteur : le patron à suivre | `invoices.rs:2293-2330` |
| F11 | Deux exercices d'une société peuvent commencer la même année civile : seuls le nom et la date de début sont uniques, et `chk_fiscal_years_dates` n'impose pas une année | `20260404000001_initial_schema.sql:49-51` |
| F12 | `fiscal_years` et `supplier_invoices` portent `updated_at … ON UPDATE CURRENT_TIMESTAMP` : un `UPDATE` de backfill le changerait, sauf `SET updated_at = updated_at` | `initial_schema.sql:47`, `20260628000001_supplier_invoices.sql:51` |
| F13 | `LPAD(1000, 3, '0')` **tronque** à `100` : `LPAD` seul ferait des doublons à la millième facture d'un exercice | comportement MariaDB ; à constater au test (AC7) |
| F14 | Le registre de rejeu est vide depuis la 32-5a, et un test exige ce vide ; le manuel d'administration dit « vide depuis la 0.16.0 » | `post_restore.rs:210-227`, `tests/admin_full_import_e2e.rs:1597` (`report.is_empty()`), `admin-manual.tex:2358` |
| F15 | Le service et le test de montée utilisent `supplier_invoices::list` ; les colonnes de `SupplierInvoice` sont écrites en trois listes | `services/factures_fournisseur.rs:119` ; `supplier_invoices.rs:41-47`, `:51-56`, `:2817-2822` |
| F16 | MariaDB est en 10.11 en production comme en dev | `docker-compose.yml:4`, `docker-compose.dev.yml:56` |
| F17 | `start_date` est immuable. Seule la réinitialisation supprime un exercice, et elle vide du même geste les factures fournisseur | `fiscal_years.rs:1047` ; `routes/onboarding.rs:309`, `kesh-seed/src/lib.rs:610-612`, `backup.rs:156` |
| F18 | Une lecture verrouillante par intervalle prend ses verrous dans l'ordre du parcours, choisi par l'optimiseur ; d'où le patron « un par un » de `close`. Création et clôture d'exercice sont rejouées sur interblocage | `fiscal_years.rs:1067-1072` ; `routes/fiscal_years.rs:283,343` |
| F20 | `audit_log.user_id` est `NOT NULL` (sa clé étrangère a été retirée par `20260910000001_audit_log_actor_label.sql:48`) : une migration, sans auteur, n'y écrit pas ; le manuel le dit déjà du lettrage posé par une mise à jour. À l'import, l'entrée `admin.full_import` nomme le rejeu et son nombre de lignes | `20260413000001_audit_log.sql:15`, `user-manual.tex:1390`, `admin.rs:526-540,639` |

## Décisions de conception (registre pour le détail)

- **Le compteur est une colonne de `fiscal_years`, non une table de séquence** (`C-e331-1`). La mécanique de
  `credit_note_number_sequences` (`credit_note_number_sequences.rs:17-76`) se réemploie dans son principe — lecture sous verrou,
  incrément, annulé par le rollback, jamais redescendu — mais pas dans sa forme : F2 l'interdit. Le compteur sur l'exercice ne
  prend **aucun verrou neuf** (F4) quand un seul exercice commence dans l'année, le cas courant (AC8), et se restaure avec l'exercice.
- **Concurrence** (`C-e331-2`) : la ligne de réglages, une fois tenue en exclusif, sérialise les attributions d'une société ; celle de
  l'exercice porte le compteur ; la lecture des compteurs et le contrôle de collision sont **verrouillants** (sous REPEATABLE READ, une
  lecture ordinaire rendrait un instantané périmé). Les compteurs des autres exercices de l'année se lisent verrouillés un par un, et
  seulement s'il y en a. L'`UNIQUE` reste le filet.
- **Un exercice dont le nom n'est pas une année** (`C-e331-3`) : `{YEAR}` vaut l'année de `start_date`, comme pour les factures
  clientes et les avoirs (exercice « 2025/2026 » commencé le 1.7.2025 : `FF-2025-001`). Le nom de l'exercice n'entre pas dans le numéro.
- **« Configurable »** (`C-e331-4`) : le format réglable y répond (préfixe, séparateur, largeur du compteur). Ni valeur de départ, ni
  choix du mode de remise à zéro. Le format fournisseur s'écrit **`<préfixe>{YEAR}<séparateur>{SEQ}`** (F9, `C-e331-16`).
- **Brouillon numéroté supprimé** (`C-e331-5`) : suppression permise, numéro perdu, trou accepté : le compteur ne redescend pas, et aucun
  exercice de la même année ne reprend le numéro (AC13) ; l'audit garde l'instantané. Les seuls retours possibles sont à `C-e331-23`.
- **Hors de tout exercice** (`C-e331-6`) : un brouillon hors exercice n'a pas de numéro et sa validation est refusée comme
  aujourd'hui (`FiscalYearInvalid`) ; à la migration, l'exercice se lit sur l'écriture d'achat, jamais sur la date : aucune
  facture n'en manque (F5).
- **Changement de format en cours d'exercice** (`C-e331-7`) : vaut pour les numéros à venir ; les numéros attribués ne se
  réécrivent pas, le compteur continue.
- **La séquence appartient à l'année de début** (`C-e331-8`) : deux exercices de la même société commençant la même année civile (F11)
  **partagent leur séquence**, quels que soient le format et le nom de l'exercice. La validation part du plus haut compteur de l'année,
  jamais d'un numéro libre plus bas ; la migration les numérote à la suite l'un de l'autre. Le seul cas où la séquence ne repart pas à 1
  au changement d'exercice. Le saut d'un numéro pris reste un filet, **vers le haut seulement**, pour une donnée posée hors de
  l'application. Ce qui reste ouvert est écrit à `C-e331-23`.
- **Brouillon numéroté déplacé dans un autre exercice** (`C-e331-9`) : garde son numéro, le compteur du nouvel exercice ne
  bouge pas. Décision de Guy appliquée à la lettre : le papier est déjà classé.
- **Libellé** (`C-e331-10`) : `Facture fournisseur FF-2026-001 (ABC-123) - Dupont SA`, et `Règlement fournisseur …` de même ; sans
  numéro du fournisseur, `Facture fournisseur FF-2026-001 - Dupont SA` ; sans aucun numéro, `Facture fournisseur #12 - Dupont SA`, ce
  qui ne se produit que par une insertion directe. Le numéro du fournisseur reste cherchable au journal.
- **Pas d'entrée d'audit pour la numérotation de l'existant** (`C-e331-24`) : une migration n'a pas d'auteur (F20). Le manuel et le
  CHANGELOG disent l'ordre appliqué.
- **Pas de `CHECK` « non brouillon ⇒ numéro »** (`C-e331-11`) : il ferait échouer la restauration d'une sauvegarde 0.17.4 avant le
  rejeu, et toute fixture qui insère une facture ouverte en SQL. L'absence se contrôle par test.
- **Migration non breaking** (`C-e331-14`) : `ADD COLUMN` nullable ou avec défaut, `ADD UNIQUE` ; ni `DROP`, ni `MODIFY`, ni
  `RENAME` (P1, P3). `kesh_version_min_required` reste à `0.17.0`, **aucun bump Cargo ici** (la release s'en charge). Comme toute
  migration, elle empêche la 0.17.4 de démarrer sur la base migrée (P1).

### Dérogation à la règle de splitting

La story touche **neuf modules** (`kesh-core/invoice_format`, `kesh-db/supplier_invoices`, `kesh-db/company_invoice_settings`,
`kesh-db/post_restore`, `kesh-api/routes/supplier_invoices`, `kesh-api/routes/company_invoice_settings`, `kesh-api/routes/fiscal_years`,
`kesh-api/exports`, `kesh-api/mcp`), au-dessus du seuil de cinq. **Dérogation assumée** (`C-e331-18`) : la migration, l'attribution, le réglage et le
rejeu forment un seul invariant, et les séparer laisserait dans `main` une colonne remplie à la montée que la validation
n'alimenterait pas, ou l'inverse. Le découpage en 33-1a (le patron) et 33-1b (le déploiement) est déjà celui que la règle
prescrit. Risque accepté : une revue plus large ; à la revue, s'en tenir aux lentilles nommées ci-dessous.

## Acceptance Criteria

### La migration (`20261106000001_supplier_invoice_internal_number.sql`)

**AC1 — Le schéma.** Après montée, et dans cet ordre d'instructions : `supplier_invoices.internal_number VARCHAR(64) NULL` ;
`fiscal_years.supplier_invoice_next_number BIGINT NOT NULL DEFAULT 1` avec `CHECK (… >= 1)` ;
`company_invoice_settings.supplier_internal_number_format VARCHAR(64) NOT NULL DEFAULT 'FF-{YEAR}-{SEQ:03}'` avec `CHECK` non vide
(patron de `chk_cis_cn_format_nonempty`) ; les deux `UPDATE` de l'AC2 et l'AC3 ; **en dernier**
`CONSTRAINT uq_supplier_invoices_internal_number UNIQUE (company_id, internal_number)`. Aucune table neuve : `TABLES_TO_TRUNCATE`
inchangé, et le test le dit (patron `draft_migration_adds_no_table`, `migrations_upgrade_path.rs:1679`).

**AC2 — La numérotation de l'existant.** Toute facture `open`, `paid` ou `cancelled` reçoit un numéro ; une facture `draft` n'en
reçoit pas. L'exercice est celui de son écriture d'achat (`journal_entries.fiscal_year_id`). Le rang se calcule par société et par
année de début d'exercice, dans l'ordre `fy.start_date, si.invoice_date, si.id`. Forme à essayer au développement :

```sql
UPDATE supplier_invoices si
  JOIN (
    SELECT s.id, YEAR(f.start_date) AS annee,
           ROW_NUMBER() OVER (PARTITION BY s.company_id, YEAR(f.start_date)
                              ORDER BY f.start_date, s.invoice_date, s.id) AS rang
      FROM supplier_invoices s
      JOIN journal_entries je ON je.id = s.purchase_journal_entry_id AND je.company_id = s.company_id
      JOIN fiscal_years f ON f.id = je.fiscal_year_id AND f.company_id = s.company_id
     WHERE s.status <> 'draft'
  ) n ON n.id = si.id
   SET si.internal_number = CONCAT('FF-', n.annee, '-', LPAD(n.rang, GREATEST(3, CHAR_LENGTH(n.rang)), '0')),
       si.updated_at = si.updated_at
 WHERE si.internal_number IS NULL;
```

*Si MariaDB 10.11 refuse la table cible dans la table dérivée, matérialiser autrement et le dire au Change Log ; la sémantique ne
change pas.* `ROW_NUMBER` a un précédent au dépôt (`20261010000001_lettering_documents_backfill.sql:162`).

**AC3 — Le compteur.** Chaque exercice reçoit `supplier_invoice_next_number` = plus grand rang de ses factures + 1, par la **même**
partition et le même ordre, `GREATEST` avec la valeur en place, `updated_at` préservé. Un exercice sans facture reste à 1.

**AC4 — Rien d'autre ne bouge.** Ni `version` ni `updated_at` de la facture ; **aucune ligne de `journal_entries`** : la description et
`updated_at` des écritures passées sont identiques à l'octet. Décision de Guy.

**AC5 — Montée sur une base 0.17.4 peuplée** (`migrations_upgrade_path.rs`, `#[sqlx::test(migrations = false)]`, montage
`migrations_before(20261106000001, …)`). ⚠️ Peupler en **SQL brut** et `journal_entries::create_in_tx` (patron `:1270-1301`), jamais
par `supplier_invoices::*` : `FIND_SCOPED_SQL` lit désormais une colonne absente de la base partielle. Le jeu couvre, en un seul
montage : deux sociétés ; exercices 2025 et 2026 ; **trois factures à la même date** (ordre par `id`) ; une facture de date
antérieure insérée après (ordre par date) ; `open`, `paid`, `cancelled`, `draft` ; numéro fournisseur présent, absent, identique
chez deux fournisseurs ; un exercice « 2025/2026 » commencé le 1.7.2025 ; **deux exercices commençant la même année civile**
(numérotés à la suite, aucun échec d'`UNIQUE`) ; un exercice sans facture. Assertions : numéro exact de chaque ligne, compteurs
exacts, AC4 ligne à ligne, aucune facture non brouillon sans numéro, brouillon à `NULL`. Assertion de montage : la base partielle
n'a pas la colonne. ⚠️ Pour AC4, `updated_at` des factures et des exercices est **épinglé à une date passée** à l'insertion : sans cela,
une mutation M7 survit si migration et insertion tombent dans la même milliseconde (`DATETIME(3)`).

**AC6 — Montée à vide.** Une base sans facture fournisseur monte sans erreur (`migrations_fresh_install`) ; le squash est régénéré
(AC21).

**AC7 — La millième.** Sur un exercice à 1 001 factures (`INSERT` en SQL, l'écriture d'achat peut être partagée : aucune unicité sur
`purchase_journal_entry_id`), le rang 1000 rend `FF-2026-1000` et non `FF-2026-100`.

### La validation

**AC8 — L'attribution.** `validate_in_tx`, sur une facture sans numéro, attribue le numéro **après** `check_written` (dernier refus de
forme) et **avant** la construction du libellé. Étape (3 bis), dans cet ordre :

1. **Les autres exercices de l'année** : lecture ordinaire des `id` des **autres** exercices de la société dont
   `start_date` est dans l'année civile de `fy.start_date`, intervalle semi-ouvert du 1er janvier au 1er janvier suivant, **tous statuts**
   (un exercice clos a émis des numéros), triés par `start_date` ; puis le compteur de chacun, lu `FOR UPDATE` **un par un** (patron de
   `close`, F18). Une seule ligne par année, le cas courant : aucun verrou neuf (F4).
2. **Le départ** : `seq` = le plus grand de ces compteurs et de celui de `fy`, lu en lecture verrouillante.
3. `render(format, année de fy.start_date, "", seq)`, contrôle de collision (AC13 f) ; pris, `seq + 1` et on recommence. Le nom d'exercice reste
   vide : le format n'a pas de `{FY}`, et `render` refuserait sinon un nom à accolade (F8). Une erreur de rendu ne vient que d'un format posé
   hors de l'application (F9) : `DbError::Invariant`, comme aux sites client et avoir.
4. Le compteur de **`fy` seul** passe à `seq + 1`, `updated_at = updated_at`. Les compteurs ne redescendent jamais : le plus grand d'entre
   eux non plus.

Le numéro part dans l'`UPDATE` de l'étape (6), celui qui passe à `open`. Le doc-comment des verrous (`:1259-1324`) porte l'étape (3 bis).

**AC9 — Le numéro survit.** `unvalidate` le garde : validée puis brouillon, annulée puis brouillon. Revalidée, la facture **reprend
le même numéro sans lire ni écrire le compteur** (patron F10), et ce même si le format a changé entre-temps.

**AC10 — Un refus ne consomme rien.** Validation refusée (exercice absent ou clos, compte archivé, compte non imputable, réglages
incomplets, conflit de version) : numéro `NULL`, compteur inchangé. Échec **après** l'attribution (`create_in_tx`) : idem, par le
rollback.

**AC11 — Séquence par année de début.** Validations entrelacées sur 2026 et 2027 : `FF-2026-001`, `FF-2027-001`, `FF-2026-002`.

**AC12 — Concurrence.** **Quatre** validations simultanées de la même société et du même exercice, par des transactions réelles : numéros
tous distincts, suite sans trou, compteur à 5. Le banc reboucle lui-même sur un interblocage (au plus dix fois) : `retry_on_deadlock` s'arrête à
trois tentatives, et son épuisement n'y serait pas un défaut. De même sur **deux exercices de la même année** : numéros tous distincts, en
suite. Et **une validation en cours avec une création de brouillon** : le contrôle de collision pose un verrou d'intervalle sur l'`UNIQUE`
quand la clé est absente, et l'insertion d'un brouillon (`internal_number NULL`) peut tomber dedans (première validation de la société) ;
elle **attend** puis réussit, sans interblocage, le validateur n'ayant besoin d'aucun verrou du brouillonneur. Test piloté par le patron de
`rejeu_interblocage_e2e.rs`. Une clôture qui croise une validation sur deux exercices de la même année peut interbloquer (ordre des dates contre
exercice de la facture d'abord) : les deux routes sont rejouées (F6, F18), à lire en revue, non à forcer au test.

**La réouverture passe au rejeu** (limite voisine, `C-e331-2`) : `reopen` tient l'exercice puis demande ses postérieurs
(`fiscal_years.rs:1258-1300`), et sa route n'est pas rejouée (`routes/fiscal_years.rs:398`), alors que la validation demande désormais les autres
exercices de l'année. `retry_on_deadlock("fiscal_years::reopen", …)` à la route, motif cloné ; la transaction est unique, audit compris, donc sûre à
relancer. Test route victime à `rejeu_interblocage_e2e.rs` (patron de `fiscal_year_close_is_replayed_when_it_is_the_deadlock_victim`, `:1376`, famille `DbError` : transaction lourde tenant l'exercice ouvert postérieur N, `reopen` de l'exercice clos M vu en attente sur `find_later_closed_in_tx`, la transaction demande M ; 200, rouvert, **une** entrée `fiscal_year.reopened`, témoin `warn!`). La phrase « `reopen` n'est pas rejouée » du doc-comment de
`close` (`fiscal_years.rs:1098-1099`) se corrige : l'ordre reste, le motif devient d'éviter le cycle plutôt que d'en dépendre.

**AC13 — Un numéro ne se réémet pas.** Testé à `supplier_invoices_internal_number.rs`, exercices A (1.1.2026-30.6.2026) et
B (1.7.2026-30.6.2027), format par défaut :

- **(a) Brouillon numéroté supprimé, l'autre exercice de l'année valide** : A émet `FF-2026-001` à `-003` ; la `-003` repasse en brouillon et se
  supprime ; B valide : `FF-2026-004`, **jamais `-003`**. Compteurs A = 4, B = 5. A valide ensuite : `FF-2026-005`.
- **(b) Base migrée** : A = 6, B = 10 après migration ; A valide : `FF-2026-010`, compteur A = 11 ; B valide : `FF-2026-011`.
- **(c) Exercice créé après coup** : A à 4 ; B, créé alors, part de son défaut 1 ; sa première facture porte `FF-2026-004`.
- **(d) Exercice clos** : même chose, A étant clôturé : son compteur compte.
- **(e) Format changé, puis repris, exercice renommé** : A « 2026 » émet `-001` à `-003` (A = 4), B « 2026-2 » émet `-004`, `-005` (B = 6) ; la `-005`
  repasse en brouillon et se supprime. Format `KF-{YEAR}-{SEQ:03}`, A valide : `KF-2026-006`, A = 7. B renommé `{2026}`, retour au défaut, B valide :
  `FF-2026-007`. La `-005` n'est jamais reprise, et le nom à accolade ne gêne pas (M24).
- **(f) Le filet, vers le haut** : un numéro posé en SQL au-dessus du compteur (`FF-2026-005`, compteur 5) : la validation rend `-006`,
  compteur 7. Saut borné : au-delà de 100 000 essais, `DbError::Invariant`.
- **(g) La lecture verrouillante, déterministe** (M11, M22). A a émis `FF-2026-001`, A = 2, B = 1. Une session de test tient `company_invoice_settings`
  `FOR UPDATE` ; la validation d'un autre brouillon de A, lancée en tâche, est **vue en attente** sur les réglages
  (`test_fixtures::attendre_une_requete_en_cours`, motifs `INSERT IGNORE` et `company_invoice_settings`). Son instantané est alors pris, `fetch_lines`
  (`:1356`) précédant les réglages (`:1421`). La session écrit `supplier_invoice_next_number = 5` sur A, ce que laissent des numéros émis puis supprimés,
  et commite. Attendu : `FF-2026-005`, A = 6 ; sur l'instantané, on lirait 2 et `-002`, libre, serait réémis. Même montage, la session écrivant 5 sur **B** :
  attendu `-005`, A = 6. Aucune de ces deux mutations ne se déclare équivalente : leur rouge est la preuve que le montage ne passe pas à vide.

Les cas restants, écrits : `C-e331-23` ; le manuel les dit (AC23, et AC12 de la 33-1b).

**AC14 — Personne d'autre n'écrit le numéro.** `create_draft`, `update_draft` et la complétion d'un document (`POST
/imported-supplier-invoices/{id}/complete`) l'ignorent, même si le corps porte `internalNumber`. Un `update_draft` le préserve. La
suppression d'un brouillon numéroté réussit, le numéro figure à l'instantané d'audit, et le compteur ne redescend pas : la facture
validée suivante prend le numéro d'après, jamais celui du supprimé (`C-e331-5`).

### Les libellés

**AC15 — Les nouvelles écritures.** Selon `C-e331-10`, dans les deux sites de F7, par **une** fonction pure testée, en `kesh-core`,
`supplier_invoice_long_label(interne, fournisseur) -> Option<String>`, avec sa sœur `supplier_invoice_short_label` que réemploie la 33-1b :
numéro interne seul ; avec numéro du fournisseur ; numéro du fournisseur blanc, qui vaut absent (règle de `:956-959`) ; numéro interne blanc ou
absent, repli sur le numéro du fournisseur ; **aucun des deux : `None`, et le libellé porte `#<id>`, les deux sites alignés**. C'est la fin de
`Facture fournisseur  - contact`, à double espace (`:1511`), et de `Règlement fournisseur <id> - contact` sans dièse (`:1875-1878`). Une
facture numérotée par la migration et réglée après porte son numéro au règlement. Revalidée, la nouvelle écriture d'achat aussi.
`journal_entries.description` (`VARCHAR(500)`) tient au pire cas : 64 + 64 + 255 plus le texte fixe.

### Le réglage, l'API, l'export

**AC16 — Le format.** `CompanyInvoiceSettings`, `COLUMNS`, l'instantané d'audit, la comparaison de non-changement, le `SET` et le
`SELECT` de repli de `company_invoice_settings.rs:37,53,247,350,1159` portent `supplier_internal_number_format`. `GET` et `PUT
/company/invoice-settings` portent `supplierInternalNumberFormat` ; **absent du corps : préservé** (patron #216,
`routes/company_invoice_settings.rs:295-302`), le formulaire ne l'envoyant qu'à partir de la 33-1b. Validation par une fonction neuve
`invoice_format::validate_numbering_template` : `validate_template`, puis la forme **`<préfixe>{YEAR}<séparateur>{SEQ}`** ou `{SEQ:NN}`, un seul `{YEAR}`
puis un seul compteur, préfixe libre (vide admis), séparateur non vide et sans chiffre, rien après le compteur, aucun `{FY}`
(`FormatError::NotNumberingShape`, message FR comme ses voisines, qui donne la forme et un exemple). Refus 400 pour un format vide, `FF-{YEAR}`, `FF-{SEQ:03}`,
`FF-{YEAR}-{SEQ:11}`, `FF-{FY}-{SEQ}`, `FF-{FY}-{YEAR}-{SEQ}`, `{YEAR}-{YEAR}-{SEQ}`, `{SEQ}-{YEAR}`, `FF-{YEAR}{SEQ}`, `FF-{YEAR}-1{SEQ}`, `FF-{YEAR}-{SEQ}-X` ; acceptés le défaut,
`KF-{YEAR}-{SEQ:04}`, `{YEAR}/{SEQ}`, `F1-{YEAR}-{SEQ}`. Les formats client et avoir gardent `validate_template` (`C-e331-16`).

**AC17 — L'API de la facture.** `internalNumber` (`string | null`) dans `SupplierInvoiceResponse` et `SupplierInvoiceListItemResponse`
(`routes/supplier_invoices.rs:75,303`), donc dans toutes les réponses qui les réemploient. `supplierInvoiceNumber` ne change ni de nom
ni de valeur. Les trois listes de F15 et `SupplierInvoiceListItem` suivent. `snapshot_json` porte `internalNumber`.

**AC18 — MCP.** `kesh_factures_fournisseur` et sa fiche (`mcp/outils/factures_fournisseur.rs:54,101,167,225`) ajoutent `numero_interne`,
`TexteNonFiable` comme `numero`, et `numero` garde son sens. `mcp_outils_e2e` et le test de contrat suivent. La pièce source de
`kesh_ecriture` et la pièce de `kesh_postes_ouverts` relèvent de la 33-1b.

**AC19 — L'export.** `serialize_supplier_invoices_csv` (`exports/csv_tables.rs:1206,1232`) gagne `internal_number` juste après
`supplier_invoice_number` ; `company_invoice_settings` gagne `supplier_internal_number_format` (`:961,985`). Le compteur n'est pas
exporté en CSV, comme les autres compteurs (`exports/global.rs:172-183`) : `serialize_fiscal_years_csv` ne change pas.
`exports_global_e2e` suit.

### La sauvegarde

**AC20 — P7, classe B.** Entrée au registre `POST_RESTORE_BACKFILLS`, version `20261106000001`, déclencheur
`Sentinels(&[("supplier_invoices", "internal_number")])`, SQL dans `post_restore/20261106000001_supplier_invoice_internal_number.sql`,
copie **verbatim** des deux `UPDATE` (tests `extract_statements_are_verbatim_substrings_of_source_migration`,
`extract_carries_every_write_statement_of_its_source_migration`, `registry_sql_contains_no_ddl`, `registry_sql_has_no_literal_hazard`,
`class_b_sentinel_column_is_added_by_its_own_migration`, `registry_entries_are_within_import_window`). Le commentaire « VIDE depuis la
Story 32-5a » (`post_restore.rs:211`) est remplacé, et la même affirmation corrigée à `admin_full_import_e2e.rs:954,1095,1178,1225,1318,1390,1504,1590` et
`post_restore_class_a.rs:81-82` ; les commentaires des migrations ne se touchent pas (P8). Dans `admin_full_import_e2e.rs` :

- **Une sauvegarde 0.17.4 reçoit les numéros** : export, `strip_column` (`:931`) de `supplier_invoices.internal_number`,
  `fiscal_years.supplier_invoice_next_number`, `company_invoice_settings.supplier_internal_number_format`, import par la **vraie route**
  et le **registre de production** ; numéros et compteurs ceux de la migration, format au défaut, entrée `REPLAYED_SENTINELS_ABSENT`
  ; la validation suivante prend le numéro d'après.
- **Une sauvegarde 0.17.5 garde les siens** : numéros non contigus (un brouillon numéroté supprimé), compteur au-delà du plus grand ;
  après import, numéros et compteur **inchangés**, entrée `SKIPPED`.
- **La sauvegarde 0.17.5 refusée par une 0.17.4** se constate à la répétition (AC22), non par un test du dépôt.
- `full_import_report_mirrors_the_production_registry` (`:1597`) perd son `report.is_empty()` : une entrée, `SKIPPED`, zéro ligne. La
  boucle n'est plus vacuement verte.

**AC21 — P5, P6, schéma de test.** P5 : ligne au tableau de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` (erreur
1060 au rejeu) ; **90** aux deux sites du total, `yes` 11, `tracked-by-sqlx` 79, `no` 0, recomptés. P6 : `grep -rn
"migrations.len()\|apply_migrations_up_to" crates/` inspecté et chaque site listé au Dev Agent Record ; à ce jour, à toucher :
`migrations_upgrade_path.rs:127` (`total` 89 → 90), `:219` (`total - 55` → `total - 56`, frontière figée à 34) et les commentaires
`:46,:124,:139,:149,:212`. Par version, sans changement : `pre_upgrade_backup/tests.rs:184,191,506`, `sauvegarde_avant_migration.rs:28,43`.
Squash : `scripts/regen-test-schema.sh`, **jamais édité à la main**, `test_schema_guard` vert. P8 : la migration ne se modifie plus
après son premier passage sur une base persistante, ou l'on reconstruit ces bases et on l'annonce.

**AC22 — La répétition réelle, sur données synthétiques.** Aucun test `#[sqlx::test]` ne démarre le binaire sur une base persistante
(P8). Base jetable `kesh_e331rep` : migrations jusqu'à `20261104000001`, seed de démonstration, factures fournisseur de tous statuts,
`kesh_version_last_applied = '0.17.4'`. (a) **Montée** : démarrage du binaire de la branche : `backup/pre-upgrade/` porte une sauvegarde, la
migration passe, `/health` répond, numéros présents. (b) **Reprise** : cette sauvegarde importée dans une seconde base au schéma à jour
redonne les mêmes numéros. (c) **Retour** : l'image `gcorbaz/kesh:0.17.4` refuse de démarrer sur la base migrée ; sur une base neuve, elle
importe la sauvegarde `pre-upgrade/` et démarre ; elle refuse une sauvegarde prise après la montée. Commandes et résultats au Dev
Agent Record, journal dans `kesh-gate-logs/e331/`. Ce que le manuel affirme du retour ne s'écrit qu'après ce passage (AC23).
⚠️ **Jamais le NAS, jamais `~/kesh/`.**

### Les textes

**AC23 — Manuel d'administration** (`.tex` et PDF, PDF aplati contrôlé) : `\subsubsection{Passer à la 0.17.5}\label{sec:maj-0-17-5}` avant
`:2524` : une migration, non breaking, seuil inchangé ; **la première sauvegarde automatique réelle** (`backup/pre-upgrade/`, #847) ; **prendre
aussi une sauvegarde manuelle complète** — base, `.keshbackup`, dossier des documents ; **le retour à la 0.17.4 passe uniquement par la
sauvegarde d'avant la mise à jour** : la 0.17.4 ne démarre plus sur la base migrée et refuse une sauvegarde de la 0.17.5 ; les factures déjà
validées reçoivent un numéro par ordre de date, et les écritures passées ne changent pas. `:2283` (*Compatibilité*) : une sauvegarde 0.17.4
s'importe dans la 0.17.5 et y reçoit les numéros ; pas l'inverse. **`:2358`, phrase « Depuis la 0.16.0, aucune de ces reprises n'est plus
rejouée » et « vide depuis la 0.16.0 » : récrites**, la numérotation est rejouée. `:2529` (« la prochaine version qui migre ») : relu, renvoi à
`sec:maj-0-17-5` ; `:2527` et `:2533` restent vrais de la 0.17.4, on n'y touche pas. Que la numérotation de l'existant **n'a pas d'entrée propre au journal
d'audit** à la montée, et qu'à l'import l'entrée `admin.full_import` nomme le rejeu et son nombre de lignes (`C-e331-24`). Au retour à la 0.17.4 : **les numéros écrits sur le papier depuis la mise à jour sont à refaire**. **§ *Importer*, `:2289`**, les trois cas de `C-e331-23`, donnés pour les seuls où un numéro peut revenir : une restauration ramène les
compteurs à la date de la sauvegarde, si bien qu'un numéro attribué après elle peut être réattribué, et le numéro écrit sur le papier depuis lors se vérifie ; une
sauvegarde 0.17.4 renumérote par date à l'import, de sorte qu'un retour à la 0.17.4 suivi d'une nouvelle montée peut décaler les numéros si des factures de date
antérieure ont été saisies entre-temps, et elle ne porte pas le format réglé, qui revient au défaut ; une réinitialisation des données fait repartir les numéros de 1,
le classement papier restant à la charge de qui l'ordonne. Le manuel utilisateur et le réglage à l'écran sont à la 33-1b.

**AC24 — CHANGELOG et API.** `## [0.17.5] — Non publié` (union) : *Ajouté*, le numéro interne, avec l'avertissement de migration et la
sauvegarde, l'ordre appliqué aux factures déjà saisies (`fy.start_date, invoice_date, id`), l'absence d'entrée d'audit propre à la montée et les trois cas où un numéro peut revenir, `refs #926`. `docs/api-external.md` : `internalNumber` et le cycle d'une facture. `supplierInternalNumberFormat` n'y figure pas : le
document ne décrit pas `/company/invoice-settings` (aucune occurrence de la route ni de `invoiceNumberFormat`), réglage d'écran resté hors du contrat
externe, comme le format client.

## Tests qui rougissent avant le correctif

À écrire d'abord et à constater rouges, note au Dev Agent Record : AC2/AC3/AC5 (colonne absente), AC8, AC9, AC13, AC15, AC16 (`FF-{YEAR}` et
`FF-{SEQ:03}` acceptés), AC20. Repository : `crates/kesh-db/tests/supplier_invoices_internal_number.rs`, sur le squash. Unitaire : `invoice_format.rs`
(`validate_numbering_template`) ; les fonctions de libellé.

**Tests existants qui rougiront, à mettre à jour au même commit** : `supplier_invoices_draft.rs:482` (libellé de l'achat). Les fixtures
`migrations_upgrade_path.rs:1292` et `support/lettering_documents.rs:802` écrivent elles-mêmes l'ancien libellé et n'assertent rien sur le code de
production : elles ne bougent pas. `git grep -nE "(Facture|Règlement) fournisseur " crates/*/tests crates/*/src` rejoué à l'étape 0.

## Mutations nommées (sous le verrou, restauration `git checkout -- <fichier>` puis `touch`)

| # | Mutation | Tuée par |
|---|---|---|
| M1 | Rang sans tri par date (`ORDER BY s.id`) | AC5 |
| M2 | Sans `s.id` en dernier tri | Probablement **aucune** : InnoDB parcourt dans l'ordre des `id` quel que soit l'ordre d'insertion, si bien que des `id` explicites n'y changent rien. À essayer sur AC5 (mêmes dates), et, si elle survit, à déclarer équivalente en pratique ; le tri reste écrit, l'ordre de `ROW_NUMBER` n'étant pas garanti sans lui |
| M3 | `LPAD(n.rang, 3, '0')` | AC7 |
| M4 | Partition par exercice au lieu de l'année civile | AC5 (même année : l'`UNIQUE` échoue) |
| M5a | `status <> 'draft'` réduit à `open` | AC5 |
| M5b | `status <> 'draft'` retiré | **Équivalente, à déclarer** : la jointure interne sur `purchase_journal_entry_id` exclut déjà les brouillons (`chk_supplier_invoices_entry_iff_not_draft`, F5) |
| M6 | Exercice lu par la date au lieu de l'écriture | AC5, ou le dire équivalente, preuve à l'appui (F5) |
| M7 | `updated_at = updated_at` retiré, facture ou exercice | AC4, `updated_at` épinglé (AC5) |
| M8 | Compteur non initialisé par la migration | AC5, AC20 |
| M9 | Revalidation qui tire un numéro neuf | AC9 |
| M10 | Saut de collision retiré | AC13 (f) |
| M11 | Lecture du compteur sans `FOR UPDATE` | AC13 (g) |
| M12 | Attribution avant `check_written` | AC10 : indiscernable par le rollback ⇒ à déclarer équivalente si c'est le cas |
| M13 | Libellé : le numéro du fournisseur à la place | AC15 |
| M14 | Sentinelle sur une autre colonne | AC20 |
| M15 | `validate_numbering_template` remplacé par `validate_template` | AC16 |
| M16 | Format absent du corps écrasé par le défaut | AC16 |
| M17 | `update_draft` qui efface le numéro | AC14 |
| M18 | Compteur décrémenté à la suppression | AC14 |
| M19 | Départ sans les autres exercices de l'année (compteur de `fy` seul) | AC13 (a), (b), (c), (e) |
| M21 | Autres exercices de l'année restreints aux ouverts | AC13 (d) |
| M22 | Compteur d'un autre exercice de l'année lu sans `FOR UPDATE` | AC13 (g) |
| M23 | Forme non contrôlée : présence de `{YEAR}` et `{SEQ}` seule | AC16 (`FF-{FY}-{YEAR}-{SEQ}`, `{YEAR}-{YEAR}-{SEQ}`, `{SEQ}-{YEAR}`, `FF-{YEAR}-1{SEQ}`, `FF-{YEAR}-{SEQ}-X`) |
| M24 | `render` reçoit `fy.name` au lieu du nom vide | AC13 (e) |
| M25 | Libellé sans aucun numéro : chaîne vide au lieu de `#id` | AC15 |
| M26 | `reopen` sans rejeu à la route | AC12 (témoin `warn!`) |

## Tâches

1. **Étape 0** : relire F1 à F18 et F20 au code, citer les lignes ayant bougé ; `git grep -n "struct SupplierInvoice\b\|SupplierInvoice {"`.
2. Tests rouges (ci-dessus).
3. Migration ; `regen-test-schema.sh` ; P5, P6, P7.
4. `invoice_format`, entité, listes de colonnes, attribution, libellés.
5. Réglages, routes (rejeu de `reopen` compris, doc-comment d'`audit_route_registry.rs:124-129,159-160` suivi), MCP, CSV.
6. Rejeu et tests d'import.
7. Mutations.
8. Gate complet, E2E complet, répétition (AC22).
9. Manuel (AC23, après AC22), PDF, CHANGELOG, `docs/api-external.md`.

## Gates

⛔ **Gate complet même en cours de boucle** : `crates/kesh-db/migrations/`, `post_restore.rs` et un repository sont touchés (§ *Test Locally
First*, exception `kesh-db`). Commandes lourdes : `flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock
scripts/mem-guard.sh <commande>`, une par prise, `free -g` avant. Bases du worktree recréées à neuf avant chaque gate complet,
`DATABASE_URL` forcée.

`cargo fmt --all -- --check` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `scripts/test-fast.sh --ci` ; `cargo build --workspace
--all-targets` ; côté frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` (rien ne devrait bouger : à constater). **E2E complet au
dernier commit de code**, 7 à 10 échecs attendus, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Vérification
après démarrage réel : `kesh_e2e_*` à la migration comprise (P8).

**Aucune capture** : aucun contrôle visible n'est ajouté. Les libellés du journal changent ; `supplier-documents.spec.ts:476` lit le lien de
la pièce source, qui ne bouge qu'à la 33-1b.

## Revue

Le prompt nomme : le **manuel d'administration** contre le code et le PDF aplati (§ *Le prompt d'une passe doit NOMMER le manuel*) ; la
migration sur une base peuplée ; la cohérence **migration / extrait de rejeu** ; les verrous de `validate_in_tx` ; les sites non résolus
(tout `format!` de libellé, toute liste de colonnes). Axes exercés et non exercés exigés au rapport ; `scripts/prepare-release.sh` interdit.

## Hors périmètre

Écrans, recherche, tri, autres sites d'affichage du numéro : 33-1b. Bump Cargo, macros des manuels, tag : release. Format client ou avoir sans
`{SEQ}` : issue demandée (`C-e331-16`).

## Dev Agent Record

*À compléter : sites P6 inspectés, tests rouges constatés, mutations, gates et chiffres, journal de la répétition.*

## Change Log

- 2026-11-06, étape 0 du développement (Opus 5.5) : les neuf LOW de `kesh-gate-logs/e33-validate-p3-e331.md`, prémisses relues.
  - **R-L1** : prémisse écrite à `C-e331-16` et `C-e331-23`, année de 1000 à 9999, que borne la 33-4 (`render` écrit `year.to_string()`, `invoice_format.rs:227`, sans remplissage ; `POST /fiscal-years` ne borne pas `start_date` à ce jour).
  - **R-L2** : AC12 cite le patron `:1376` (lu : `fiscal_year_close_is_replayed_when_it_is_the_deadlock_victim`), assertion d'une seule entrée d'audit ajoutée.
  - **R-L3** : F20 et `C-e331-24` : `fk_audit_log_user` retirée par `20260910000001_audit_log_actor_label.sql:48` (relu à la migration), `NOT NULL` suffit ; à l'import, `admin.full_import` nomme le rejeu.
  - **R-L4** : `{YEAR}-{YEAR}-{SEQ}` ajouté à AC16 et M23, miroir à la 33-1b.
  - **R-L5** : `audit_route_registry.rs` ajouté à la tâche 5.
  - **F-L1** : AC24 porte l'ordre, l'absence d'entrée et les trois cas. **F-L2** : dialogue de la 33-1b, renvoi au manuel. **F-L3** : phrase ajoutée à AC23.
  - **F-L4** : `internal_number` reste juste après `supplier_invoice_number` (AC19). Lu : l'export CSV porte une ligne d'en-tête et le dépôt n'a aucun lecteur par position (`exports_global_e2e` lit par nom) ; le manuel ne promet aucun ordre de colonnes.
  - Réserve de rédaction du rapport : « et valide » devient « et commite » à AC13 (g).

- 2026-11-06 : fiche créée, `ready-for-dev`.
- 2026-11-06, remédiation de la validation P1 (Opus 5.5, rapports `kesh-gate-logs/e33-validate-p1-e331-R.md` et `-F.md` ; passe 1 : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 13 LOW, dont dix findings touchant cette fiche, 1 MEDIUM et 9 LOW). Prémisses relues aux lignes citées, toutes justes sauf ce qui est dit ci-dessous.
  - **M-1 (lentille R)**, remédié, sur arbitrage de l'orchestrateur : format à `{SEQ}` et `{YEAR}` ou `{FY}` (AC16) ; départ au plus haut compteur des exercices frères, le saut ne servant plus que de filet (AC8, AC13). Le scénario du rapport devient AC13 (a), et le format sans année ne s'enregistre plus. **Changement de conception**, cas restants à `C-e331-23`.
  - **L-5**, sans objet : le format sans année est refusé, le saut ne parcourt plus l'exercice précédent.
  - **L-1 (R) et L4 (F)**, remédiés : variante et clé propres (AC13 h, F19), au lieu du texte générique.
  - **L-2**, remédié : M5 scindée, M5b déclarée équivalente ; M7 sur un `updated_at` épinglé. Pour M2, **la remédiation proposée est réfutée** : des `id` explicites hors ordre d'insertion ne changent pas l'ordre de parcours d'InnoDB, qui suit la clé primaire. M2 s'essaie et se déclare si elle survit.
  - **L-3**, remédié : prémisse vraie dans le cas d'une première validation de la société, où un `NULL` neuf tombe dans l'intervalle ; attente écrite et test à AC12.
  - **L-4**, remédié : le cas sans numéro est tranché (`#id`, AC15), test existant qui rougit nommé. `migrations_upgrade_path.rs:1292` et `support/lettering_documents.rs:802` fabriquent le libellé sans rien asserter sur la production : ils ne rougissent pas, contrairement à ce que laissait entendre le rapport.
  - **L-6**, remédié : limite acceptée (`C-e331-24`, F20). Nuance de prémisse : une écriture en SQL pur n'est pas impossible, c'est l'auteur qui manque (`audit_log.user_id NOT NULL`).
  - **L3 (F)**, remédié : `:2529` au lieu de `:2527`, et `:2533` nommé pour n'y pas toucher (lus).
  - **L5 (F)**, remédié : `supplierInternalNumberFormat` retiré de `api-external.md`, route hors du contrat (AC24).
  - Décomptes de cette entrée, recomptés sur cette fiche : faits F17 à F20 ajoutés (20 au total) ; mutations M19 à M25 ajoutées et M5 scindée (26 lignes au tableau).
  - Propagation : `grep` de `format sans année`, `sans {YEAR}`, `FF-{SEQ`, `requireSeq`, `:2527`, `ConfigurationRequired`, `premier libre`, `saute`, `aucun verrou neuf`, `réattribu` sur les deux fiches et le registre, exécuté après écriture ; résidus traités dans cette même remédiation : `aucun verrou neuf` (décisions de conception, C-e331-1), `jamais réattribué` au manuel utilisateur (33-1b, AC12, renvoi à la restauration). Les autres occurrences sont légitimes.
- 2026-11-06, remédiation de la validation P2 (Opus 5.5, rapport `kesh-gate-logs/e33-validate-p2-e331.md` ; passe 2 : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 9 LOW, dont neuf findings touchant cette fiche, 3 MEDIUM et 6 LOW). **Simplification de l'orchestrateur, proposée à Guy et valable sauf avis contraire** : `{FY}` et `numbering_scope` retirés, la séquence appartient à l'année de début. Prémisses relues au code, lignes citées ; deux détails de lignes corrigés (ci-dessous). Rien n'est exécuté : ni compilation, ni test, ni mutation.
  - **R-M2**, remédié par cette simplification. Le contre-exemple (A = 4, B = 6, `-005` supprimé) rend désormais `-006` et devient AC13 (e) ; le renommage ne joue plus, le nom n'étant plus passé à `render` ; `{FY}{SEQ}` est refusé. **Un autre chemin restait, hors du rapport** : un chiffre à côté du compteur (`FF-{YEAR}-1{SEQ}` au rang 3, puis `FF-{YEAR}-{SEQ}` au rang 13, même chaîne). Fermé par la forme `<préfixe>{YEAR}<séparateur>{SEQ}` (AC16, `C-e331-16`), ce qui rend le rendu injectif ; `C-e331-23` réécrit, trois cas d'utilisateur, et la fenêtre d'instantané dite pour mémoire.
  - **R-M1**, remédié : AC13 (g), M11 et M22 sans échappatoire. Prémisse juste (`fetch_lines` avant l'attente, `:1356`, `:1421`). Le montage diffère de celui proposé : la session écrit le compteur au lieu de rejouer émission et suppression, seule sa valeur étant en jeu, et l'issue ne dépend plus du saut.
  - **F-M1**, remédié : AC23 dit les trois cas, donnés pour les seuls ; la réserve proposée sur `{FY}` et le renommage est sans objet.
  - **R-L1**, remédié : `reopen` passe au rejeu (AC12, M26), le correctif tenant à la route. Module ajouté à la dérogation : neuf.
  - **R-L2**, remédié : N fixé à 4, le banc reboucle (AC12) ; `C-e331-2` précise la sérialisation. Le plafond de trois est à `kesh-db/src/retry.rs:72`, non `:60`.
  - **R-L3**, remédié : F17 réécrite, `reset_cleared_tables` lu (`backup.rs:156`, `fiscal_years` et `supplier_invoices` hors de `RESET_PRESERVED_TABLES`).
  - **F-L3**, remédié (AC20) : la ligne du rapport `:1592` est à `:1590`. **F-L4**, remédié (AC22). **F-L5**, remédié (AC23, `C-e331-23`).
  - **Retiré, devenu sans objet** : `numbering_scope` et l'étape de portée ; AC13 (e) ancienne, (f) fondue dans la nouvelle (e), (h) ; la variante `SupplierInternalNumberNotRenderable`, sa clé aux quatre locales et `C-e331-17`, un format validé sans `{FY}` se rendant toujours (F9) ; F19 ; M20 ; `NoSeqPlaceholder` et `NoYearPlaceholder`, remplacées par la seule `NotNumberingShape`. Le numéro n'attend plus le nom : `render` reçoit un nom vide (F8, M24 réécrite).
  - **Gardé** : le saut vers le haut (AC13 f, M10, verrou d'intervalle d'AC12). Il ne sert plus qu'à une donnée posée hors de l'application, mais sans lui une telle donnée bloquerait toute validation, le compteur étant annulé avec la transaction : `C-e331-8`.
  - Décomptes de cette entrée, recomptés sur cette fiche par `grep -c` : AC 24, inchangé, sous-cas d'AC13 de 8 à 7 ; mutations 26 lignes, inchangé (− M20, + M26 ; M23 et M24 réécrites) ; faits 20 → 19. **La fiche ne raccourcit pas au total** : 393 → 404 lignes avant cette entrée, 37 624 → 39 808 octets, AC13 (g), le rejeu de `reopen` et la liste d'AC16 pesant plus que ce qui part.
  - Propagation : `grep` de `{FY}`, `numbering_scope`, `frère`, `réattribu`, `NotRenderable`, `not-renderable`, `no-seq`, `no-year`, `F19`, `M20`, `CalendarYear`, `huit modules`, sur les deux fiches et le registre, exécuté après écriture. Restent légitimes : les entrées passées de ce Change Log, le format client (33-1b, AC12), les refus d'AC16, les restaurations de `C-e331-23`.
