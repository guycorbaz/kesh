# Story 16.1a-bis : Backfill du compte de produit sur le parc de factures existant

## Status

review

## Story

**As a** utilisateur de Kesh dont l'instance contient déjà des factures validées (dont l'instance de production en fonction),
**I want** que la colonne `revenue_account_id` introduite par 16-1a soit **renseignée rétroactivement** sur mes factures et avoirs déjà validés, à partir de l'écriture comptable qu'ils ont réellement produite,
**so that** un avoir **émis à partir de maintenant** sur une facture ancienne **extourne le compte effectivement crédité** par cette facture, et non le compte de produit par défaut tel qu'il se trouve être configuré ce jour-là — ce qui laisserait un résidu permanent, invisible au bilan et faux au compte de résultat.

Issue : **#152**. Rattaché au CR **#265**. Sous-story de l'Epic 16 « Facturation avancée ».

**Dépend strictement de 16-1a** : la colonne `invoice_lines.revenue_account_id` et son pendant `credit_note_lines` sont créés là-bas. Cette story ne fait que les **remplir pour le passé**.

### Provenance du split (passe 6 de `validate` sur 16-1a)

Le backfill était initialement la décision **D2-bis** de 16-1a, née d'un finding CRITICAL de la passe 4. Sur les passes 5 et 6 de `bmad-create-story validate`, **7 findings sur 10 — dont les 3 HIGH — portaient sur lui seul**, tandis que le reste de 16-1a (schéma, moteur de ventilation, avoir, validation) n'était plus remis en cause depuis la passe 3. La sévérité de 16-1a a cessé de décroître (P5 : 1 HIGH → P6 : 2 HIGH), cochant le second critère de la § « Règle de splitting préventif » de `CLAUDE.md`.

**Arbitrage de Guy, 2026-07-26 : extraction en story dédiée.** Motif : le backfill a un **profil de risque étranger** au reste de 16-1a — c'est une migration de **données comptables réelles**, one-shot, non rejouable au sens métier, essentiellement du SQL, alors que 16-1a est du schéma et de la logique applicative testable unitairement. Les deux ne se revoient pas avec le même regard, et le backfill saturait les passes adversariales de 16-1a au point de masquer d'éventuels défauts ailleurs.

**Le split n'ouvre aucune fenêtre de corruption** : sans cette story, le parc antérieur conserve **exactement** le comportement d'aujourd'hui (repli sur le défaut courant au moment de l'avoir). Le backfill est **strictement additif** — il ferme un bug pré-existant, il n'en introduit aucun. C'est ce qui rend le split sûr, et c'est la différence avec le split 16-1a / 16-1b, où D5 devait impérativement rester avec sa cause.

---

## Contexte

### Le bug fermé par cette story

`settings.default_revenue_account_id` est **mutable** par l'utilisateur dans les Réglages (`company_invoice_settings.rs:173`). Pour une ligne de facture sans compte (`NULL`), la chaîne est : ligne `NULL` → snapshot d'avoir `NULL` → repli résolu **au moment de l'avoir**, sur la configuration relue à T2 (`credit_notes.rs:275-282`). D'où :

| T | Événement | Écriture |
|---|---|---|
| T1 | Facture validée, ligne `NULL`, défaut = 3000 | **crédit 3000** = HT |
| T1+ | L'administrateur change le défaut → 3200 | — |
| T2 | Avoir total émis | **débit 3200** = HT |

Résidu permanent au crédit de 3000 et au débit de 3200. **Bilan équilibré, compte de résultat faux, aucun signal.**

**Quelle population cette story protège exactement — à lire avec D-B7.** Le tableau ci-dessus se lit dans **deux temps distincts**, et la story n'agit que sur l'un des deux :

| Situation au moment du déploiement | Ce que fait cette story |
|---|---|
| Facture validée, **avoir pas encore émis** (T2 dans le futur) — **le cas de l'immense majorité des factures** | **Le bug est fermé.** La ligne est backfillée à 3000 ; l'avoir futur, qui copiera le compte depuis `invoice_lines` (16-1a D5) au lieu de relire la configuration, débitera 3000. Aucun résidu. |
| Facture validée **et avoir déjà émis** (T2 déjà passé) | **Rien n'est réparé** — l'écriture d'avoir existe, elle a débité 3200, c'est irréversible. Le backfill se contente d'**enregistrer fidèlement** les deux comptes réellement mouvementés (3000 et 3200), qui **diffèrent légitimement**. Cf. **D-B7**. |

Autrement dit, le backfill est **préventif**, pas curatif : il arme les factures passées pour que leurs avoirs **à venir** soient corrects. Corriger un couple déjà soldé exigerait une écriture de reclassement — un **acte comptable de l'utilisateur**, hors de portée d'une migration.

16-1a ferme ce trou pour les factures **futures**, en matérialisant le compte effectif dans `invoice_lines` à la validation (sa décision D2). Mais cette matérialisation ne se déclenche qu'à la transition `draft → validated` : une facture **déjà validée** n'y repasse **jamais** (`update` rejette tout statut ≠ `draft`, `invoices.rs:841`, `:1271`) et aucune autre écriture ne touche `invoice_lines` après validation. Après l'`ADD COLUMN` de 16-1a, ces lignes portent `NULL` **définitivement**.

Or `NULL` est **le seul cas qui existe en production aujourd'hui** — aucune facture existante ne porte de compte de ligne. **16-1a seule protège donc un ensemble vide à l'instant du déploiement**, et 100 % du parc validé reste exposé. C'est la raison d'être de cette story.

### Ce qui existe aujourd'hui

- **`invoices.journal_entry_id`** — lien facture → écriture, FK `ON DELETE RESTRICT` (`migrations/20260417000001_invoice_validation.sql:54-56`). Une facture `validated` a **toujours** une écriture : `chk_invoices_validated_has_je` (`migrations/20260417000002_invoice_validated_journal_entry_check.sql`) l'impose, et la FK `RESTRICT` interdit de supprimer l'écriture après coup.
- **`credit_notes.journal_entry_id`** — idem côté avoir, `CHECK (status <> 'issued' OR (credit_note_number IS NOT NULL AND journal_entry_id IS NOT NULL))` (`20260627000001_credit_notes.sql:54`).
- **Structure produite par `generate_invoice_journal_lines`** (`invoices.rs:1156`, doc `:1120-1155`) : `[0]` débit créance = TTC, `[1]` **crédit produit unique** = `total_ht`, `[2..N]` crédit TVA due par taux.
- **`journal_entry_lines.line_order`** — `INT NOT NULL`, `UNIQUE (entry_id, line_order)`, assigné séquentiellement `(idx as i32) + 1` à l'insertion (`journal_entries.rs:272`).
- **`generate_credit_note_journal_lines`** (`credit_notes.rs:139`) — « inverse exact » : la contre-passation débite ce que la facture a crédité.
- **Précédents de backfill en migration** : `20260628000001_supplier_invoices.sql:115` (`UPDATE … INNER JOIN accounts … WHERE … IS NULL`, multi-table) et `20260722000001_accounts_role_postable.sql` (**12 `UPDATE` de backfill**). Les deux sont classés `tracked-by-sqlx` et décrits comme **intrinsèquement idempotents** dans `docs/migrations-idempotence-audit.md`.

### Ce qui n'est PAS dans cette story

- La colonne elle-même, l'entité, l'API, le moteur de ventilation, la matérialisation à la validation, les avoirs futurs → **16-1a**.
- Toute la surface utilisateur → **16-1b**.
- La **réconciliation facture ↔ écriture après édition manuelle de l'écriture** (cf. D-B2) → hors périmètre, et hors périmètre de l'Epic 16.

---

## Décisions de conception

### D-B1 — La source de vérité est l'écriture générée, JAMAIS le défaut courant

**Décision** : le compte backfillé est celui que l'écriture comptable de la pièce a **effectivement** crédité (facture) ou débité (avoir).

**Pourquoi pas le défaut courant** : si l'administrateur a déjà changé `default_revenue_account_id` par le passé, backfiller avec la valeur d'aujourd'hui écrirait un compte que la facture n'a **jamais** crédité — on **fabriquerait** la corruption au lieu de la fermer. L'écriture générée est la **pièce probante**, et c'est l'argument même de la décision D2 de 16-1a.

### D-B2 — La pièce probante n'est pas garantie intacte : le backfill doit être CONSERVATEUR

**C'est la décision structurante de cette story, et l'origine du split.**

La structure `[0]` créance / `[1]` produit / `[2..]` TVA décrit ce que `generate_invoice_journal_lines` **produit**, pas ce que la table **contient** au moment de la migration.

**Ground-truth (vérifié 2026-07-26)** : l'écriture d'une facture validée est **éditable par l'utilisateur**.

- `journal_entries` ne porte **aucune** colonne `source` / `origin` / `is_auto` — une écriture générée par une validation de facture est **indiscernable** d'une écriture manuelle (schéma complet `migrations/20260412000001_journal_entries.sql:17-36`) ;
- `PUT /api/v1/journal-entries/{id}` est exposée (`kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805`) et n'a **aucune garde de provenance** : ses seules gardes sont l'exercice clos, le verrou optimiste, l'équilibre en partie double et la validité des comptes ;
- `journal_entries::update` fait `DELETE FROM journal_entry_lines WHERE entry_id = ?` puis ré-INSERT en **réattribuant les `line_order` de zéro** (`:1005`).

Sur une base réelle, l'écriture d'une facture peut donc porter ses lignes dans un autre ordre, sur d'autres comptes, en nombre différent. Les deux méthodes d'identification naïves échouent **silencieusement** :

- **positionnelle** (`line_order = 2`) — après ré-INSERT, la 2ᵉ ligne peut être n'importe quoi, y compris une ligne de TVA ou une charge ;
- **par élimination** (tout ce qui n'est ni la créance ni la TVA) — une écriture éditée peut en laisser **zéro** ou **plusieurs**.

Et l'erreur n'est pas neutre : un mauvais compte écrit dans `invoice_lines` devient **la** vérité de la facture, que 16-1a D5 recopiera dans tout avoir futur. **Un backfill approximatif fabrique exactement la corruption qu'il existe pour fermer**, en lui donnant en plus l'apparence d'une donnée établie. Sur les écritures non éditées — la quasi-totalité du parc — l'identification reste parfaitement fiable : le défaut est le manque de **discrimination**, pas la méthode.

**Décision — règle du backfill** : ne backfiller **que** si la ligne de produit est identifiable **sans ambiguïté**, c'est-à-dire s'il existe dans l'écriture **exactement une** ligne remplissant les trois conditions :

1. `credit > 0` (donc `debit = 0`, garanti exclusif par `chk_jel_debit_credit_exclusive`) ;
2. son `account_id` n'appartient pas à l'ensemble d'exclusion `E = { cis.default_receivable_account_id, cis.default_vat_payable_account_id }`, **les valeurs `NULL` étant ignorées** (une colonne de configuration non renseignée n'exclut rien) ;
3. son `credit` est **égal à `invoices.total_amount`** (le HT, `Σ line_total` — cf. docstring `invoices.rs:1128`, qui écrit littéralement « `total_ht` (HT, = `invoices.total_amount`, DC9) »). **Il n'existe pas de colonne `total_ht`** ; ne pas la chercher.

La condition (3) est le **discriminant** que ni la position ni l'élimination ne donnent : c'est elle qui distingue une écriture canonique d'une écriture retouchée. Zéro candidat, ou plusieurs → la ligne reste `NULL` et est **dénombrable** (D-B4).

**Miroir avoir** : `debit` au lieu de `credit`, et `credit_notes.total_amount` au lieu d'`invoices.total_amount`.

*(Redondance assumée : l'exclusion de la créance en (2) est inoffensive mais inutile — sur l'écriture facture la créance est un **débit**, déjà éliminée par (1) ; sur l'écriture d'avoir elle est un crédit, éliminée par le (1) miroir. Seule l'exclusion TVA porte réellement. Conservée pour lisibilité et défense en profondeur.)*

### D-B3 — La condition (2) DOIT être écrite en SQL NULL-safe : sinon le backfill no-ope sur 100 % du parc

**Piège dirimant, et le plus vicieux du dossier.** Les colonnes de configuration sont toutes nullables, et les **trois colonnes TVA sont `NULL` sur toute installation** :

- l'`INSERT` d'onboarding ne les énumère pas (`company_invoice_settings.rs:452-458` : seulement `invoice_number_format`, `default_receivable_account_id`, `default_revenue_account_id`, `default_payable_account_id`, `default_sales_journal`, `journal_entry_description_template`) ;
- le lazy-create insère `(company_id)` seul (`:92`, `INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)`) ;
- aucune migration ne les renseigne (`grep -rn "SET default_vat" crates/kesh-db/migrations/` → **vide** ; `20260614000001_vat_accounts_config.sql` crée bien les comptes `1171` / `2206` mais ne pointe jamais les colonnes de config dessus).

Écrite naïvement `jel.account_id <> cis.default_vat_payable_account_id`, la comparaison à `NULL` rend le prédicat `NULL` en logique ternaire SQL, la ligne n'est **jamais** candidate, et le backfill **no-ope intégralement** — migration en succès, décompte « très élevé ».

**Ce qui rend ce mode de défaillance particulièrement dangereux** : la présente spec **pré-autorise** explicitement un décompte élevé comme un comportement conservateur normal (D-B2). Le bug est donc **rigoureusement indiscernable du succès** sans un test dédié.

**Décision** : utiliser l'opérateur NULL-safe de MariaDB.

```sql
NOT (jel.account_id <=> cis.default_receivable_account_id)
AND NOT (jel.account_id <=> cis.default_vat_payable_account_id)
```

**Jamais** `<>` / `!=` / `NOT IN`, qui propagent `NULL`.

**Pourquoi `E` ne contient que ces deux comptes** : `default_vat_recoverable_account_id` et `default_vat_decompte_account_id` n'apparaissent **jamais** dans une écriture de vente — `generate_invoice_journal_lines` ne reçoit que le compte de TVA due (`invoices.rs:1387`). Les inclure n'ajoute rien et multiplie les occasions de propager un `NULL`.

### D-B4 — Le décompte des lignes non backfillées : aucun artefact nouveau

Une migration sqlx est un fichier `.sql` pur exécuté par `MIGRATOR.run()` (`kesh-db/src/lib.rs:23`, appelé depuis `kesh-api/src/main.rs:138`) : elle n'a **aucun canal de restitution**. Deux options ont été envisagées et **écartées** :

- *table de rapport* — **refusée** : toute nouvelle table fait tomber `backup_inventory_matches_schema` (`backup.rs:577-606`), impose de mettre à jour `TABLES_TO_TRUNCATE`, et fait donc entrer la table dans le périmètre de l'**export/import d'installation** — décision d'architecture qui serait prise par accident, pour faire passer un test ;
- *log applicatif au démarrage* — **refusée** : ajoute dans `main.rs` une requête de comptage rejouée à chaque boot indéfiniment, pour un besoin ponctuel de déploiement.

**Décision** : le décompte est obtenu par une **requête de diagnostic documentée**, exécutable à tout moment, sans état persistant :

```sql
-- Factures
SELECT COUNT(*) FROM invoice_lines il JOIN invoices i ON i.id = il.invoice_id
WHERE i.status = 'validated' AND il.revenue_account_id IS NULL;
-- Avoirs
SELECT COUNT(*) FROM credit_note_lines cnl JOIN credit_notes cn ON cn.id = cnl.credit_note_id
WHERE cn.status = 'issued' AND cnl.revenue_account_id IS NULL;
```

Elle est consignée au CHANGELOG au titre des notes de déploiement, et c'est **elle** que les tests assertent. **Aucune table, aucun log, aucune ligne dans `main.rs`.**

### D-B5 — Portée : `validated` / `issued` seulement ; les `cancelled` sont exclues délibérément

**Périmètre** : `invoice_lines` des factures `status = 'validated'` **et** `journal_entry_id IS NOT NULL` (ce second prédicat est une garde défensive **redondante** avec `chk_invoices_validated_has_je`), plus le miroir sur `credit_note_lines` des avoirs `issued`.

Les factures `draft` restent `NULL` — c'est le sens même de la liaison tardive (16-1a D2).

Les factures `cancelled` sont exclues **délibérément**, et **non** « parce qu'elles n'auraient pas d'écriture » — elles en ont toujours une. Le seul chemin qui produit ce statut est l'émission d'un avoir (`credit_notes.rs:398`, `UPDATE invoices SET status = 'cancelled' … AND status = 'validated'`). Le motif réel est que `uq_credit_notes_invoice UNIQUE (invoice_id)` (`20260627000001_credit_notes.sql:58`) interdit un **second** avoir : **aucun résidu futur n'est possible**, il n'y a rien à prévenir.

**Conséquence assumée** : sur une facture créditée, `credit_note_lines.revenue_account_id` sera renseigné alors que `invoice_lines.revenue_account_id` restera `NULL`. C'est visible dans l'export CSV (16-1a AC14) et **sans effet comptable**.

### D-B7 — Le backfill ENREGISTRE le résidu historique, il ne le RÉPARE pas

**Décision, et clarification cardinale du périmètre.** Pour un couple facture / avoir **antérieur au déploiement**, les deux comptes backfillés peuvent légitimement **différer** — et le backfill doit les écrire tels quels.

**Preuve du mécanisme** : `create_credit_note` relit la configuration **au moment de l'émission de l'avoir** — `company_invoice_settings::get_or_create_default_in_tx` puis `settings.default_revenue_account_id` (`credit_notes.rs:275-282`) — indépendamment de ce que la facture a réellement crédité. C'est **le bug décrit dans le § Contexte** : facture créditée sur 3000 à T1, avoir débité sur 3200 à T2 après changement du défaut.

Les avoirs déjà en base ont été produits par ce code. Leur écriture de contre-passation porte donc, dans ce cas, un compte **différent** de celui de la facture. Le backfill, dont la source de vérité est l'écriture réelle (D-B1), écrira fidèlement `3000` côté facture et `3200` côté avoir.

**C'est le comportement voulu.** Le résidu comptable **existe déjà dans les écritures** — il est passé, il est irréversible, et les pièces le documentent. Le rôle du backfill est de le rendre **explicite et lisible**, pas de le maquiller : réécrire l'un des deux pour forcer l'égalité falsifierait les pièces sans corriger la moindre écriture.

**Ce que la story corrige, et ce qu'elle ne corrige pas** :

- pour les avoirs **futurs**, 16-1a D5 supprime la relecture des `settings` et copie le compte depuis `invoice_lines` — le résidu ne se reproduira plus ;
- pour les avoirs **passés**, rien ne peut être corrigé sans passer une écriture de reclassement, ce qui est un **acte comptable** relevant de l'utilisateur, pas d'une migration.

**Conséquence pour 16-1b** : l'écran de détail peut afficher, pour une facture et son avoir, **deux comptes de produit différents**. Ce n'est pas une anomalie du backfill et l'UI ne doit ni le signaler comme une erreur, ni tenter de l'harmoniser.

**Ne PAS** écrire d'AC affirmant l'égalité des deux comptes : elle est fausse dans le seul scénario qui motive l'existence de cette story.

### D-B6 — Migration non-breaking et backfill intrinsèquement idempotent

Le backfill est un `UPDATE` gardé par `revenue_account_id IS NULL` et fondé sur un critère **déterministe** : un re-jeu recalcule le même résultat et n'a aucun effet. Il est donc **intrinsèquement idempotent**, exactement comme les backfills de `20260628000001_supplier_invoices.sql` et `20260722000001_accounts_role_postable.sql` (« **12 UPDATE de backfill** en revanche intrinsèquement idempotents »), tous deux classés `tracked-by-sqlx`.

**Ne pas** qualifier la migration de « non idempotente » : `docs/migrations-idempotence-audit.md` maintient l'invariant **« Idempotence `no` : 0 »** (`:71`), et un verdict `no` ferait diverger les compteurs.

Aucune opération `DROP` / `RENAME` / `MODIFY COLUMN` → migration **non-breaking** au sens P1 de `CLAUDE.md` → **pas** de bump `kesh_version_min_required`, donc **pas** de bump de version Cargo (P2-bis).

---

## Acceptance Criteria

- **AC-B1** — Migration de backfill : `invoice_lines.revenue_account_id` des factures `status = 'validated' AND journal_entry_id IS NOT NULL` reçoit le compte identifié par le **critère d'unicité en trois conditions** de D-B2 ; miroir sur `credit_note_lines` des avoirs `issued` (`debit`, `credit_notes.total_amount`).
  - **Ne PAS** backfiller depuis `settings.default_revenue_account_id` courant (D-B1).
  - **Ne PAS** identifier la ligne par `line_order` seul (D-B2).
  - **Ne PAS** écrire la condition (2) avec `<>` / `!=` / `NOT IN` (D-B3).
  - **Condition (4), volet FACTURE UNIQUEMENT** : le compte retenu doit être **imputable** (`accounts.postable = TRUE`). `invoice_lines.revenue_account_id` n'est pas un instantané du passé mais la source de vérité que 16-1a D5 recopie dans tout avoir futur ; y écrire un compte collectif produirait une donnée que la saisie elle-même refuse (`RevenueAccountRejection::NotPostable`). Le cas est atteignable — la validation d'écriture tourne avec `enforce_postable = false`. Le volet **avoir** n'a délibérément pas cette garde : une contre-passation doit viser les MÊMES comptes que l'écriture d'origine (`credit_notes.rs:405-409`). *(Ajouté en passe 2 de `bmad-code-review`, arbitrage Guy du 2026-07-29 ; la passe 1 avait écarté le point en invoquant le rationale de la contre-passation, qui ne vaut que pour le volet avoir.)*
- **AC-B2** — **Le backfill est délibérément incomplet, et c'est la spécification.** Toute ligne dont le compte n'est pas identifiable sans ambiguïté — écriture éditée, zéro ou plusieurs candidats — **reste `NULL`**, et la migration **réussit**. Une post-condition « aucune ligne validée ne reste `NULL` » serait **fausse** et pousserait le dev à relâcher le critère jusqu'à ce qu'elle passe, c'est-à-dire à écrire un compte arbitraire sur des données comptables réelles — l'inverse exact de l'objectif.
- **AC-B3** — Post-conditions testées, **sur base pré-remplie** (`migrations_fresh_install` ne prouve rien ici : il n'y a rien à backfiller sur une base vierge) :
  1. facture validée à écriture **canonique** → ligne backfillée avec le compte crédité par l'écriture, **même si `settings.default_revenue_account_id` a changé depuis**. Sans ce changement de défaut, le test passerait aussi avec un backfill depuis le défaut courant et ne prouverait rien (D-B1) ;
  2. facture validée dont l'écriture a été **éditée** de sorte qu'aucune ligne ne crédite exactement `total_amount` → la ligne reste `NULL`, la migration **réussit**, et la requête de diagnostic de D-B4 retourne le compte attendu ;
  3. **société dont `default_vat_payable_account_id` est `NULL`** — c'est-à-dire **toute** société non configurée manuellement, le cas par défaut — facture validée à écriture canonique → la ligne **est** backfillée. **C'est le seul test qui attrape la propagation `NULL` de D-B3** ; sans lui, un backfill qui no-ope intégralement est indiscernable d'un backfill conservateur ;
  4. facture `draft` → reste `NULL` ; facture `cancelled` → reste `NULL` (D-B5) ;
  5. miroir avoir : avoir `issued` à écriture canonique → ligne backfillée avec le compte **réellement débité par l'écriture de l'avoir** (D-B1), déterminé **indépendamment** de la facture d'origine ;
  6. **couple facture / avoir DIVERGENT — le cas qui compte (D-B7)** : facture validée à T1 sur défaut = 3000, défaut changé en 3200, avoir émis à T2 → le backfill écrit **3200 sur l'avoir**, tandis que **la facture reste `NULL`** — l'émission de l'avoir l'a basculée en `cancelled`, statut que D-B5 exclut délibérément. Ce qui **diffère, et c'est le résultat correct**, c'est le compte porté par l'avoir (3200) et celui que l'écriture de la facture a réellement crédité (3000, lisible dans `journal_entry_lines`, jamais recopié dans `invoice_lines`). Un test qui affirmerait l'égalité échouerait ici — et un test construit sans changer le défaut entre les deux passerait trivialement sans rien prouver, exactement l'écueil signalé au cas 1.
     *(Rédaction corrigée en passe 1 de `bmad-code-review`, arbitrage Guy du 2026-07-29. L'énoncé d'origine annonçait « le backfill écrit **3000 sur la facture** et 3200 sur l'avoir » : la première moitié est **structurellement inatteignable**. `create_credit_note` est l'unique chemin de création d'avoir et bascule inconditionnellement la facture en `cancelled` — `crates/kesh-db/src/repositories/credit_notes.rs:561-574`, où `rows == 0` lève `OptimisticLockConflict` et rollback toute la transaction. Une facture créditée n'est donc jamais `validated`, donc jamais éligible au backfill. D-B5 énonçait déjà cette conséquence ; l'AC la contredisait. Le dev l'avait signalée sans la corriger, se déclarant hors mandat. La substance reste verrouillée par `divergent_invoice_and_credit_note_are_recorded_as_is`, qui teste le comportement réel — c'est l'AC qui était fausse, pas le code.)*
  *(Le cas « facture validée **sans écriture** » n'est **PAS** testé : il est **inconstructible** — `chk_invoices_validated_has_je` l'interdit en base et `fk_invoices_journal_entry … ON DELETE RESTRICT` empêche de supprimer l'écriture après coup. Tenter la fixture ne produit qu'une violation de CHECK.)*
- **AC-B4** — **Idempotence** : rejouer le backfill sur une base déjà backfillée ne change **aucune** ligne (garde `IS NULL` + critère déterministe). Testé **sur les deux volets** : le test d'idempotence doit contenir à la fois des factures **et** des avoirs déjà backfillés. Un rejeu du second `UPDATE` sur un ensemble vide de `credit_note_lines` ne démontre rien — c'est vrai de n'importe quel SQL. *(Précision ajoutée en passe 1 de `bmad-code-review`.)*
- **AC-B5** — `docs/migrations-idempotence-audit.md` : ligne ajoutée au tableau détaillé avec verdict **`tracked-by-sqlx`** (justifié par l'absence d'`IF NOT EXISTS`, **pas** par le backfill, qui est idempotent — D-B6), **ET** récapitulatif agrégé de bas de fichier mis à jour en cohérence (`Total` et `Idempotence tracked-by-sqlx` chacun +1). L'invariant « Idempotence `no` : 0 » est **préservé**. Garde-fou **P5** de `CLAUDE.md`.
  - **Le TOTAL de migrations apparaît à DEUX endroits de ce fichier** — l'**en-tête de section** `## Table d'audit (N migrations)` et la ligne `Total` des « Statistiques ». S'y ajoutent **trois compteurs de partition** (`yes`, `tracked-by-sqlx`, `no`) qui ne valent **pas** le total : c'est leur **somme** qui doit l'égaler. Tout se recompte depuis le tableau. *(Passe 1 de `bmad-code-review` : l'en-tête était resté à 56 pour 57 réelles, seul site oublié — la dérive exacte que P5 existe pour empêcher. Passe 2 : la règle de contrôle alors écrite était elle-même fausse, elle alignait les compteurs de partition sur le total. Corriger un compteur et écrire la règle de son contrôle sont deux gestes distincts.)*
- **AC-B6** — Migration **non-breaking** → **pas** de bump `kesh_version_min_required`, donc **pas** de bump de version Cargo (P1/P2/P2-bis). Le vérifier explicitement.
- **AC-B7** — **Aucune table n'est créée par cette story** (D-B4). Corollaire vérifié : `backup_inventory_matches_schema` (`backup.rs:577-606`) reste vert sans toucher `TABLES_TO_TRUNCATE`, et le compteur d'entrées du ZIP d'export (`exports_global_e2e.rs:734`, `assert_eq!(entries.len(), 20, …)`) est inchangé.
- **AC-B8** — CHANGELOG `[Non publié]` : entrée orientée utilisateur **avec les requêtes de diagnostic de D-B4**, au titre des notes de déploiement, et mention explicite que les factures dont l'écriture a été modifiée manuellement ne sont pas reprises.
  - **ET** mention explicite du **seul effet de bord bloquant** du backfill : sur une facture ancienne dont le compte de produit a été **archivé** depuis sa validation, l'émission d'un avoir **échoue désormais** au lieu de se replier silencieusement sur le défaut société. Le refus est le comportement voulu (le repli recréerait le résidu que D5 combat) et l'erreur nomme le compte à réactiver, mais l'entrée ne doit pas promettre « aucune action de votre part » sans cette réserve. *(Ajouté en passe 1 de `bmad-code-review`, arbitrage Guy du 2026-07-29 : garder le refus, documenter.)*
- **AC-B9** — Gate « Test Locally First » backend complet vert (`cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`). Les suites `migrations_fresh_install` et `admin_backup_e2e` doivent passer — cette story touche une migration.

---

## Tasks / Subtasks

- [x] **T-B0** — **Ordonnancement de la migration.** Le fichier de migration de cette story DOIT porter un timestamp **strictement postérieur** à celui de l'`ADD COLUMN` de 16-1a : `sqlx::migrate!("./migrations")` (`crates/kesh-db/src/lib.rs:23`) exécute dans l'**ordre lexicographique du nom de fichier**, et un backfill jouant avant la création de la colonne échoue en `Unknown column 'revenue_account_id'`. Vérifier l'état de 16-1a **avant** de choisir le timestamp. Si les deux stories sont développées en parallèle sur des branches distinctes, **merger 16-1a en premier** ou regrouper les deux dans la même PR (cohérent `feedback_pr_grouping` : les deux stories touchent le même répertoire de migrations). L'échec est bruyant et attrapé en CI, pas silencieux — mais il coûte un cycle.
- [x] **T-B1** — Migration de backfill `invoice_lines` : `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, critère de D-B2, condition (2) en `<=>` NULL-safe (D-B3).
- [x] **T-B2** — Miroir `credit_note_lines` (`debit`, `credit_notes.total_amount`), même critère.
- [x] **T-B3** — Ligne du tableau **et** compteurs agrégés de `docs/migrations-idempotence-audit.md`, verdict `tracked-by-sqlx` + justification d'idempotence (AC-B5, D-B6).
- [x] **T-B4** — Tests sur base pré-remplie : les **6** cas d'AC-B3 + l'idempotence d'AC-B4 + la garde `postable` d'AC-B1 (condition 4, ajoutée en passe 2 de revue).
- [x] **T-B5** — CHANGELOG avec requêtes de diagnostic (AC-B8) + gate backend complet (AC-B9).

**Ordre conseillé** : T-B0 → T-B1 → T-B4 (partiel, cas facture) → T-B2 → T-B4 (complet) → T-B3 → T-B5.

### Review Findings

**Passe 1 de `bmad-code-review`** — 2026-07-29, Sonnet ×3 lentilles (Blind Hunter / Edge Case Hunter / Acceptance Auditor), orchestration + vérification ground-truth Opus 5. 9 findings bruts → 6 retenus (5 MEDIUM, 1 LOW), 3 écartés.

- [x] [Review][Decision] **Le backfill peut rendre IMPOSSIBLE l'émission d'un avoir sur une facture ancienne dont le compte a été archivé depuis** — Nouveau mode de défaillance **bloquant** introduit sur le parc existant, couvert par aucun AC et contredit par le CHANGELOG (« Aucune action de votre part »). Avant le backfill, une facture ancienne a toutes ses lignes à `NULL` : la garde de `create_credit_note` (`crates/kesh-db/src/repositories/credit_notes.rs:410-419`) ne collecte alors **que** le défaut société courant (`sites.push((0, revenue_account_id))`), donc seul ce compte-là est contrôlé `active` — l'avoir passe. Après le backfill, chaque ligne porte le compte historique ; si ce compte a été **archivé** depuis la validation (`accounts::archive` ne consulte pas `invoice_lines`, cf. D5-bis de 16-1a), le `SELECT … WHERE active = TRUE` (`:427`) le rejette et l'émission échoue en `DbError::CreditNoteRevenueAccountsArchived` (`:474`). Le refus est sans doute le comportement **voulu** (il évite de recréer le résidu que D5 combat) et l'erreur est bruyante et actionnable — mais c'est un arbitrage produit, pas un détail d'implémentation. Écarté par les lentilles sous l'angle `account_type` ; c'est `active` qui mord. **Arbitrage Guy du 2026-07-29 : garder le refus, documenter** — archiver est un acte délibéré, laisser une contre-passation le contourner en silence viderait l'archivage de son sens. Suite tracée en **issue #280** (avertir à l'archivage, sans le bloquer). Deux remèdes écartés au passage : *bloquer l'archivage d'un compte mouvementé* — il n'existe aucun endpoint de suppression d'un compte et la FK `ON DELETE RESTRICT` l'interdit en base, donc l'archivage est le **seul** moyen de retirer un compte et existe précisément pour les comptes mouvementés ; le bloquer rendrait tout compte ayant servi irretirable à vie ; et *relâcher la garde `active` sur le chemin de contre-passation*, écarté par Guy au motif que le refus respecte l'intention de l'archivage.
- [x] [Review][Decision] **AC-B3 cas 6 est structurellement inatteignable et n'a pas été corrigé** — L'AC annonce que le backfill écrit le compte d'origine **sur la facture**. `create_credit_note` est l'unique chemin de création d'avoir et bascule inconditionnellement la facture en `cancelled` (`crates/kesh-db/src/repositories/credit_notes.rs:561-574`, `rows == 0` → `OptimisticLockConflict` qui rollback toute la transaction) ; or D-B5 exclut délibérément les `cancelled`. Le dev a tranché en faveur de D-B5 et verrouillé la substance réelle par `divergent_invoice_and_credit_note_are_recorded_as_is`, mais a laissé l'AC intacte en se déclarant hors mandat. La spec porte donc une AC fausse. Convergence Blind Hunter + Acceptance Auditor.
- [x] [Review][Patch] **L'en-tête du tableau d'audit annonce toujours 56 migrations pour 57 réelles** [docs/migrations-idempotence-audit.md:17] — `## Table d'audit (56 migrations)`. Recompté : 57 fichiers `.sql` sur disque, 57 lignes dans le tableau, et la section « Statistiques » a bien été portée à 57/4/53/0. C'est le **troisième** site du même nombre dans le même fichier, et le seul oublié — exactement la dérive que le garde-fou P5 de `CLAUDE.md` a été codifié pour empêcher, et que la ligne 94 de ce fichier même met en garde.
- [x] [Review][Patch] **Aucun test ne croise le vrai pipeline de validation et le backfill** [crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs] — Toute la puissance discriminante du critère repose sur l'invariant `jel.credit == invoices.total_amount`, affirmé « par construction » dans le commentaire de la migration (`:79-85`). Or les 8 tests montent leurs fixtures en **SQL brut** avec le même littéral des deux côtés (`grep -c` de `validate_invoice`, `generate_invoice_journal_lines`, `compute_total` dans ce fichier = **0**), et réciproquement `invoices_line_revenue_account.rs` — qui, lui, passe par le vrai moteur — ne rejoue **jamais** le backfill (il efface la matérialisation *après* les migrations). L'invariant est donc reproduit à la main, jamais observé depuis le moteur : une divergence future d'un centime entre `total_amount` et le crédit réellement posé ferait no-oper le critère (3) sans qu'aucun test ne bouge. Le recours au SQL brut est **légitime** (le code actuel ne sait plus produire l'état pré-16-1a) — le manque est le test de jonction.
- [x] [Review][Patch] **5 sites de documentation décrivent encore 16-1a-bis comme non livrée** [crates/kesh-db/src/entities/invoice.rs:77 ; crates/kesh-db/src/repositories/invoices.rs:1687 ; crates/kesh-db/src/entities/credit_note.rs:67 ; crates/kesh-api/src/exports/csv_tables.rs:477 ; crates/kesh-db/src/repositories/credit_notes.rs:183] — Les deux premiers (« leur traitement **relève de** la Story 16-1a-bis ») sont désormais **faux** : le traitement est livré. Les trois autres (« validée avant 16-1a **et non traitée par** 16-1a-bis ») restent littéralement exacts mais se lisent au futur. Le Dev Agent Record revendique avoir grepé `16-1a-bis` sur tout le dépôt et corrigé 5 sites ; ces 5-ci portent le même symptôme et sont passés au travers. À noter pour la rétro : l'union des deux lentilles donne 5 sites, aucune ne les avait tous (Edge Case Hunter a manqué `credit_notes.rs:183`, l'Acceptance Auditor a manqué `invoices.rs:1687`).
- [x] [Review][Patch] **`backfill_is_idempotent` ne rejoue le backfill que sur des factures, jamais sur des avoirs** [crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs:863] — Le test rejoue bien les **deux** `UPDATE` du fichier de migration, mais n'insère aucun avoir : `insert_credit_note_with_lines` n'est appelé qu'aux lignes 717 et 826, dans deux autres tests. Le second `UPDATE` s'applique donc sur un ensemble vide — ce qui est vrai de n'importe quel SQL. Or `docs/migrations-idempotence-audit.md:79` affirme l'idempotence de « celui-ci » sans distinguer les deux `UPDATE`. Si la garde `WHERE cnl.revenue_account_id IS NULL` du volet avoir disparaissait dans un refactor, aucun test ne le verrait. Convergence Blind Hunter + Edge Case Hunter.

**Écartés en ground-truth (3)** :

- *Absence de contrôle `account_type` dans le critère d'exclusion* (Edge Case Hunter, MEDIUM) — **écarté** : `crates/kesh-db/src/repositories/credit_notes.rs:405-409` documente explicitement que ni `postable` ni `account_type` ne sont re-vérifiés, « la contre-passation doit viser les **mêmes** comptes que l'écriture d'origine, quelle qu'ait été leur évolution de configuration ». Écrire le compte que l'écriture a réellement crédité est précisément l'intention de D-B1. Le vrai risque de cet axe est `active`, remonté séparément ci-dessus.
- *Modification de `crates/kesh-api/src/routes/credit_notes.rs` hors périmètre annoncé* (Acceptance Auditor, LOW) — **écarté** : le diff n'y touche que des doc-comments (`:41-56`), aucune logique runtime. C'est exactement ce qu'exige la § « Propagation post-patch » de `CLAUDE.md`.
- *Absence de test sur un changement de `default_receivable_account_id`* (Blind Hunter, LOW) — **écarté** : l'exclusion de la créance est prouvablement redondante (sur l'écriture facture la créance est un **débit**, déjà éliminée par `jel.credit > 0` ; miroir exact côté avoir). Le commentaire de la migration `:131-134` l'énonce déjà comme défense en profondeur. **⚠️ Écartement partiellement RENVERSÉ en passe 2** : la clause est bien sans effet sur la *ligne de créance*, mais elle porte sur `jel.account_id` de **toute** ligne et peut donc retirer un vrai positif. Ce n'est pas de la défense en profondeur — cf. la passe 2 ci-dessous.

**Passe 2 de `bmad-code-review`** — 2026-07-29, Opus ×3 lentilles, contexte frais, diff aplati `4db9003e..HEAD` (contribution complète de la story, patches de passe 1 inclus, en un seul diff). ~24 findings bruts, dont **12 MEDIUM**. Axe imposé aux trois lentilles : **traiter la remédiation de la passe 1 comme le matériau le plus suspect**, la rétrospective 16-1a l'ayant mesurée comme première source de défauts.

- [x] [Review][Decision] **Restaurer un backup antérieur au backfill rouvre le bug définitivement** — `check_import_version_compat` ne refuse qu'un backup exigeant un binaire plus récent ; la colonne étant nullable, `ColumnInfo::is_required()` (`backup.rs:329-331`) est faux et le restore d'un dump sans la colonne est accepté ; et `_sqlx_migrations` est **exclue** du restore (`backup.rs:586`), donc `20260729000001` reste marquée appliquée et **ne repasse jamais**. Tout le parc restauré revient à `NULL`, définitivement. Aggravant : la requête de diagnostic remonterait un décompte élevé que le CHANGELOG qualifie de bénin — l'unique instrument de détection misattribue la cause. **Arbitrage Guy : rejouer le backfill après un restore** (sûr par construction, D-B6). Relève du chemin backup/restore (Epic 17), donc **issue #281** plutôt qu'un patch ici ; réserve ajoutée au CHANGELOG en attendant. *(Edge Case Hunter — aucune autre lentille ne l'avait vu.)*
- [x] [Review][Decision] **Le premier `UPDATE` écrivait dans `invoice_lines` sans contrôler `postable`** — et l'écartement de ce point en passe 1 s'appuyait sur le **mauvais chemin** : le commentaire cité (`credit_notes.rs:405-409`) justifie l'absence de contrôle pour la *contre-passation*, où les comptes sont recopiés d'une écriture existante. Or `invoice_lines.revenue_account_id` n'est pas un instantané du passé : c'est la source de vérité que 16-1a D5 recopie dans tout avoir futur. La validation d'écriture tournant avec `enforce_postable = false`, une écriture ancienne a pu créditer un compte collectif ; le backfill l'écrivait, produisant une donnée que la saisie elle-même refuse (`RevenueAccountRejection::NotPostable`). **Arbitrage Guy : exiger `postable = TRUE` sur le volet facture uniquement.** Condition (4) ajoutée, dissymétrie avec le volet avoir assumée et documentée. *(Blind Hunter.)*
- [x] [Review][Decision] **Verdict d'idempotence incohérent avec la légende du fichier** — `tracked-by-sqlx` est réservé aux migrations dont la re-exécution manuelle **échouerait**, code d'erreur MariaDB à l'appui ; celle-ci n'a aucun DDL, donc la rejouer réussit, et c'était la **seule ligne du tableau sans code d'erreur**. Un exploitant y lisait « ne pas rejouer » sur la seule migration du dépôt où la réponse est « oui sans risque ». **Arbitrage Guy : verdict `yes`.** Légende du verdict `yes` étendue au cas « sans DDL, `UPDATE` gardés et déterministes » ; compteurs recomptés → **5 `yes` / 52 `tracked-by-sqlx` / 0 `no` = 57**. *(Blind Hunter.)*
- [x] [Review][Decision] **« L'exclusion de la créance est redondante, conservée pour défense en profondeur » est faux** — le raisonnement ne vaut que pour la ligne de créance elle-même ; la clause porte sur `jel.account_id` de **toute** ligne. Si `default_receivable_account_id` désigne aujourd'hui un compte qui figure comme crédit de produit dans des écritures passées, une facture canonique perd son unique candidat et reste `NULL`. Une clause dont le meilleur cas est de ne rien faire et le pire de supprimer un vrai positif n'est pas de la défense en profondeur : c'est un risque net. **Arbitrage Guy : garder la clause** (pire cas conservateur, scénario inhabituel), **corriger le commentaire** qui la décrivait comme son contraire. *(Blind Hunter.)*
- [x] [Review][Patch] **TRIPLE CONVERGENCE — `backfill_is_idempotent` ne pouvait pas détecter la disparition de la garde `IS NULL`**, et mon patch de passe 1 n'y changeait rien [crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs] — les trois fixtures portaient une valeur qui est le **point fixe** du critère : retirer les deux gardes les réécrivait à l'identique, et un test qui ne compare qu'« avant re-jeu » à « après re-jeu » restait **vert**. La passe 1 avait ajouté du montage sans ajouter de discrimination, et un doc-comment qui affirmait le contraire. Corrigé par la facture `F-PRE`, dont la valeur stockée **diffère** du candidat, plus une assertion sur la valeur **absolue** (la garde tombe dès le premier passage, pas au re-jeu). **Mutation : les 2 gardes retirées → `backfill_is_idempotent` SEUL rouge.** *(Blind Hunter + Edge Case Hunter + Acceptance Auditor, indépendamment.)*
- [x] [Review][Patch] **Le volet avoir n'avait aucun filet contre le piège D-B3** [migration `:301`] — `backfills_when_vat_config_is_null` ne couvre que les factures ; les trois tests à avoirs tournent avec la config TVA *posée par la fixture*. Muter le seul `<=>` du second `UPDATE` laissait donc **toute la suite verte**, alors qu'en exploitation le backfill des avoirs no-operait en silence. Ajout de `backfills_credit_note_when_vat_config_is_null`. **Mutation : `<=>` → `<>` sur le seul volet avoir → ce test SEUL rouge.** *(Blind Hunter.)*
- [x] [Review][Patch] **La portée de D-B3 était surestimée, et le raisonnement faux** [migration `:101-113`] — « colonnes TVA `NULL` sur TOUTE installation » donc « no-op sur 100 % du parc ». Or `validate_invoice` passe `settings.default_vat_payable_account_id` à `generate_invoice_journal_lines` (`invoices.rs:1794`), qui échoue en `ConfigurationRequired` dès que `total_vat > 0` (`:1497-1500`) : une société dont la colonne est `NULL` n'a pu valider **que** des factures sans TVA. Le rayon réel est « les installations **exonérées de TVA** » — cas réel en Suisse sous CHF 100'000, cœur de cible de Kesh, donc le test garde toute sa valeur. Le raisonnement faux aurait pu conduire un mainteneur à la conclusion inverse (« mon install a la config, donc `<>` est sûr »). Propagé : migration, doc d'audit, tableau du module de test, et la fixture de `backfills_credit_note_when_vat_config_is_null` est montée **exonérée**, l'état réellement atteignable. *(Edge Case Hunter.)*
- [x] [Review][Patch] **L'avertissement du CHANGELOG sur le résidu avait disparu, et une ancre de code y renvoyait toujours** [CHANGELOG.md ; crates/kesh-api/src/routes/credit_notes.rs:60] — la puce ⚠️ de 16-1a a été remplacée par une formulation neutre (« sans effet sur vos écritures existantes ») alors que le risque **subsiste intégralement** pour les pièces à écriture retouchée, ce que le dépôt prouve lui-même (`invoices_line_revenue_account.rs:1290`, « résidu permanent »). Vérifié : `grep -cF "Aucun message ne le signale" CHANGELOG.md` = **0**. Avertissement rétabli, ciblé sur la population résiduelle. *(Acceptance Auditor.)*
- [x] [Review][Patch] **La règle de recompte P5 écrite en passe 1 était arithmétiquement fausse** [CLAUDE.md ; docs/migrations-idempotence-audit.md] — « les trois sites doivent donner le même nombre » : l'en-tête et `Total` valent 57, mais `tracked-by-sqlx` est un compteur de **partition**. Un relecteur appliquant la consigne à la lettre aurait « corrigé » un fichier correct et cassé l'invariant `yes + tracked-by-sqlx + no = Total`. Réécrit : **deux** sites portent le total, **trois** compteurs de partition dont la somme l'égale. *(Blind Hunter, et trouvé indépendamment par l'orchestrateur avant les rapports.)*
- [x] [Review][Patch] **Le CHANGELOG annonçait comme non repris un cas que le backfill reprend** [CHANGELOG.md] — « lignes réorganisées » figurait parmi les écritures laissées de côté, alors que le critère est **insensible à l'ordre par construction** (c'est précisément pourquoi l'identification positionnelle a été abandonnée). Un utilisateur croyait sa pièce intacte alors qu'elle est reprise. Corrigé, avec la précision explicite qu'un simple réordonnancement ne gêne pas. *(Blind Hunter.)*
- [x] [Review][Patch] **La requête de diagnostic masque la classe la plus nombreuse** [CHANGELOG.md] — elle ne compte que les factures `validated`, donc **aucune** facture `cancelled` : une installation ayant émis 200 avoirs a 200 factures dont toutes les lignes sont `NULL` par décision D-B5, invisibles. L'exploitant lit « 0 » et en déduit que rien n'a été laissé de côté. Deux limites d'interprétation ajoutées (classe `cancelled`, et le cas restore de #281). *(Blind Hunter.)*
- [x] [Review][Patch] **Deux compteurs de fenêtre périmés dans `migrations_upgrade_path.rs`** [:95 « sauf les 13 dernières » ; :212 « les 10 migrations restantes »] alors que `:140` calcule `total - 23` — dans le fichier même où la story revendique avoir aligné « les 5 sites », et dont un commentaire documente que « trois copies du même symptôme y ont été découvertes une par passe ». L'Edge Case Hunter a recompté **7** sites, pas 5. *(Edge Case Hunter, l'Acceptance Auditor n'en avait vu qu'un.)*
- [x] [Review][Patch] **AC-B3 cas 6, réécrite en passe 1, contenait une affirmation que rien n'assertait** — « le compte historique reste lisible dans `journal_entry_lines` ». La fixture le *construisait* sans l'*observer*. Assertion ajoutée à `divergent_invoice_and_credit_note_are_recorded_as_is`. *(Acceptance Auditor.)*
- [x] [Review][Patch] **Le commentaire de `csv_tables.rs` patché en passe 1 omettait la cause la plus fréquente** — son jumeau d'`entities/invoice.rs`, touché par le **même** patch, énumérait bien les factures `cancelled` ; celui-ci ne citait que brouillon et écriture retouchée. Or `cancelled` = toute facture ayant reçu un avoir, cas systématique. Le symptôme corrigé sur un site ne l'avait pas été sur l'autre. *(Blind Hunter.)*
- [x] [Review][Patch] **Ma « preuve » de passe 1 ne reproduisait pas** — le Change Log citait « `grep "INSERT INTO credit_notes"` ne retourne qu'une occurrence » à l'appui d'un arbitrage produit. Il y en a **trois** (`credit_notes.rs:519`, `invoices.rs:3347` sous `#[cfg(test)]`, et une fixture de test). La conclusion tient — `create_credit_note` reste l'unique chemin de **production** — mais une preuve invoquée doit être exacte. Reformulé. *(Acceptance Auditor, qui en avait compté 2 sur 3.)*
- [x] [Review][Patch] **Descripteurs de couverture périmés** — File List et T-B4 annonçaient « 8 tests » et « les 5 cas d'AC-B3 » pour **11** tests et **6** cas. Les occurrences restantes sont dans des entrées de journal **datées** et décrivent fidèlement l'état de leur étape : elles ne sont pas réécrites. Ancre AC-B7 corrigée (`exports_global_e2e.rs:734`, l'assertion réelle, `:621` étant un `.send()`). *(convergence des 3 lentilles.)*
- [x] [Review][Patch] **`backfill_is_idempotent` n'assertait pas la pré-condition `NULL` de sa facture témoin** — un relâchement du critère la faisant backfiller dès le premier passage aurait laissé le test vert, le rôle que son propre commentaire lui assigne n'étant tenu par personne. Assertion ajoutée. *(Blind Hunter.)*

**Reportés (2)** — tracés, non bloquants :

- *`backfill_skips_archived_accounts` reste muet* (Edge Case Hunter, LOW) — sa seule assertion positive passe aussi bien si le backfill de rôles 14-3a écarte correctement le compte archivé que s'il ne tourne pas. Le passage à la résolution **par version** ferme le mode d'échec *positionnel* (l'objet du garde-fou P6), pas la muettitude intrinsèque du test. Dette **14-3a**, hors périmètre de cette story ; à traiter au triage de la rétrospective Epic 16.
- ~~*Aucun test ne couvre le `LEFT JOIN` sur `company_invoice_settings`*~~ (Edge Case Hunter, LOW) — **REPORT LEVÉ, test livré le 2026-07-30 sur demande de Guy.** Et le report était une erreur d'appréciation : en écrivant le test signalé, la mutation a révélé que le **volet avoir n'avait aucun filet non plus** — muter son seul `LEFT` en `INNER` laissait les 12 autres tests verts. Ajouter le seul test demandé aurait reproduit exactement l'asymétrie facture/avoir que la passe 2 avait dû corriger sur le `<=>`. Deux tests livrés, chacun tuant sa mutation.

**Écartés en ground-truth (2)** :

- *Le volet « brouillon » de `skips_draft_and_cancelled_invoices` est tautologique* (Blind Hunter, LOW) — exact (un `draft` n'a pas d'écriture, donc l'`INNER JOIN` ne produit rien quoi qu'il arrive), mais la moitié `cancelled` du test travaille et le témoin prouve l'exécution. Rendre le volet brouillon discriminant exigerait un brouillon **portant** une écriture — un état que le code ne produit jamais. Le remède serait moins vrai que le défaut.
- *L'invariant « le helper pousse `credit = total_ht` » n'est plus littéralement vrai* (Acceptance Auditor, LOW) — exact : depuis 16-1a le helper émet **une ligne de crédit par compte**. Mais l'affirmation reste vraie de la **population visée** (parc pré-16-1a, toutes lignes `NULL` → un seul compte), et le test de jonction l'observe désormais sur la sortie réelle du moteur. La formulation est imprécise, le raisonnement est sain.

---

## Dev Notes

### Ancres ground-truth (vérifiées 2026-07-26, passes 5 et 6 de `validate` sur 16-1a)

| Élément | Emplacement |
|---|---|
| Lien facture → écriture (FK RESTRICT) | `crates/kesh-db/migrations/20260417000001_invoice_validation.sql:54-56` |
| **`CHECK` : facture `validated` ⇒ écriture non-NULL** | `crates/kesh-db/migrations/20260417000002_invoice_validated_journal_entry_check.sql` |
| Lien avoir → écriture + CHECK `issued` | `crates/kesh-db/migrations/20260627000001_credit_notes.sql:36`, `:47-48`, `:54` |
| `uq_credit_notes_invoice` (un seul avoir par facture) | `20260627000001_credit_notes.sql:58` |
| Schéma `journal_entries` / `journal_entry_lines` — **aucune colonne `source`** | `crates/kesh-db/migrations/20260412000001_journal_entries.sql:17-36`, lignes `:38-56` |
| `chk_jel_debit_credit_exclusive` | `20260412000001_journal_entries.sql:46` |
| **`line_order` réattribué à chaque `update`** | `crates/kesh-db/src/repositories/journal_entries.rs:1005` ; création `:272` |
| **`PUT /journal-entries/{id}` — aucune garde de provenance** | `crates/kesh-api/src/lib.rs:303` → `routes/journal_entries.rs:517` → `repositories/journal_entries.rs:805` |
| Structure de l'écriture facture (doc + fn) | `crates/kesh-db/src/repositories/invoices.rs:1120-1155` (doc), `:1156` (fn) |
| `total_ht` = `invoices.total_amount` (écrit littéralement) | `invoices.rs:1128` |
| `invoices.total_amount DECIMAL(19,4)` | `20260416000001_invoices.sql:24` |
| Compte de TVA due passé au helper | `invoices.rs:1387` |
| Helper avoir « inverse exact » | `crates/kesh-db/src/repositories/credit_notes.rs:139` |
| Bascule facture → `cancelled` (seul chemin) | `credit_notes.rs:398` |
| **Colonnes TVA de config — nullables** | `crates/kesh-db/migrations/20260614000001_vat_accounts_config.sql:54-56` |
| **`INSERT` onboarding — n'énumère PAS les colonnes TVA** | `crates/kesh-db/src/repositories/company_invoice_settings.rs:452-458` |
| **Lazy-create `(company_id)` seul** | `company_invoice_settings.rs:92` |
| Défaut société mutable par l'utilisateur | `company_invoice_settings.rs:173` |
| Précédent backfill multi-table en migration | `crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:115` |
| Précédent 12 `UPDATE` de backfill idempotents | `20260722000001_accounts_role_postable.sql` ; audit `docs/migrations-idempotence-audit.md` |
| Invariant « Idempotence `no` : 0 » | `docs/migrations-idempotence-audit.md:71` (compteurs `:68-70`) |
| `MIGRATOR` et son appel | `crates/kesh-db/src/lib.rs:23` ; `crates/kesh-api/src/main.rs:138` |
| `backup_inventory_matches_schema` | `crates/kesh-db/src/backup.rs:577-606` |

### Pièges, par ordre de coût

0. **La propagation `NULL` de la condition (2) (D-B3)** — le pire, parce qu'il est **indiscernable du succès** : la migration passe, le décompte est élevé, et la spec autorise explicitement un décompte élevé. Sans le cas 3 d'AC-B3, personne ne le verra jamais. Écrire `<=>`, pas `<>`.
1. **Le backfill trop confiant (D-B2)** — écrire un compte **faux** sur des factures validées réelles, que 16-1a D5 recopiera ensuite dans tout avoir futur. L'identification positionnelle paraît solide parce que le **code de génération** est déterministe — mais l'écriture est **éditable après coup**. Le discriminant obligatoire est le montant égal à `invoices.total_amount`.
2. **La post-condition « plus aucun `NULL` »** — elle est **fausse** (AC-B2) et un dev qui cherche à la faire passer ira mécaniquement écrire un compte arbitraire. Ne jamais tester par un `COUNT(*) = 0` global.
3. **`total_ht` n'est pas une colonne** — c'est `invoices.total_amount` (et `credit_notes.total_amount` côté avoir).
4. **Le verdict d'idempotence (D-B6)** — écrire « non idempotente » casse l'invariant `no : 0` du fichier d'audit et fait diverger les compteurs d'AC-B5. Le backfill **est** idempotent ; c'est l'absence d'`IF NOT EXISTS` qui justifie `tracked-by-sqlx`.
5. **Base vierge** — `migrations_fresh_install` ne prouve **rien** ici. Tous les tests d'AC-B3 exigent une base pré-remplie. Précédent direct et fonctionnel du pattern « appliquer n-1 migrations, insérer en SQL brut, appliquer la dernière » : `crates/kesh-db/tests/accounts_role_backfill.rs`.
6. **Vouloir « réparer » le couple facture / avoir divergent (D-B7)** — le réflexe naturel est d'affirmer que les deux comptes doivent être égaux. C'est **faux** pour tout couple antérieur au déploiement dont le défaut société a changé entre-temps : l'avoir a été généré en relisant `settings` à T2 (`credit_notes.rs:275-282`). Le backfill **enregistre** ce résidu, il ne le corrige pas — le corriger serait un acte comptable, pas une migration.
7. **Ordre des migrations (T-B0)** — un timestamp antérieur à l'`ADD COLUMN` de 16-1a fait échouer la migration. Bruyant, mais coûte un cycle CI.

### Faisabilité SQL (vérifiée)

- **MariaDB 10.11** (`docker-compose.yml:4`, `.github/workflows/ci.yml:34`).
- La restriction ER 1093 (« can't specify target table for update in FROM clause ») **ne s'applique pas** : la sous-requête de candidats lit `journal_entry_lines` / `invoices` / `company_invoice_settings`, jamais la table cible `invoice_lines` (miroir : jamais `credit_note_lines`).
- Précédent direct de multi-table `UPDATE` en migration : `20260628000001_supplier_invoices.sql:115`.
- sqlx exécute bien plusieurs instructions par fichier (`20260627000001_credit_notes.sql` en contient 5).
- **Ne pas utiliser de CTE (`WITH`)** — aucune migration du dépôt n'en contient (`grep -n "^WITH " crates/kesh-db/migrations/*.sql` → vide). Préférer `UPDATE … JOIN (SELECT … GROUP BY … HAVING COUNT(*) = 1) c`, qui a un précédent.

### Sûreté numérique de la condition (3) — vérifiée, ne pas ré-instruire

`journal_entry_lines.debit` / `credit` et `invoices.total_amount` / `credit_notes.total_amount` sont **tous** `DECIMAL(19,4)` — exact en MariaDB, même échelle des deux côtés, aucune comparaison flottante. Les deux valeurs sont identiques **par construction** : `generate_invoice_journal_lines` pousse `credit: total_ht = Σ line_total` (`invoices.rs:1178`, `:1198`), tandis qu'`invoices.total_amount` vient de `compute_total` = `Σ compute_line_total` (`:379-383`), la **même** fonction qui écrit `line_total` (`:393`). Une facture validée étant immuable, les deux ne peuvent pas diverger après coup.

**Aucune facture ancienne n'a de structure incompatible** : la seule version antérieure du helper (remplacée par l'Epic 18, commit `654dba7d`) produisait 2 lignes avec `credit = invoice_before.total_amount` — la condition (3) y est vraie *a fortiori*.

### Propagation post-patch (§ CLAUDE.md)

Après chaque patch de remédiation, **grep le symptôme sur tout le dépôt** avant la passe suivante. Symptômes propres à cette story : `revenue_account_id`, `total_ht`, `line_order`, `default_vat_payable`, `<=>`, `IS NULL`, « idempotent », les compteurs de `migrations-idempotence-audit.md`. **Greper aussi 16-1a et 16-1b** : les trois stories partagent le même vocabulaire et une correction ici a souvent un jumeau là-bas.

### References

- Issue **#152**, CR **#265**.
- Story **16-1a** — socle backend dont dépend celle-ci (colonne, entité, moteur, matérialisation D2).
- Story **16-1b** — surface utilisateur ; son AC6-bis doit tolérer un `revenue_account_id` `NULL` côté avoir pour les pièces antérieures non backfillables.
- Stories antérieures : **12-1** (avoirs et contre-passation), **18-1b** (helper d'écriture facture), **14-3a** (précédent de 12 `UPDATE` de backfill en migration), **12-2** (précédent de backfill multi-table).
- `CLAUDE.md` : politique de migration (P1-P5), Review Iteration Rule, propagation post-patch, règle de splitting préventif.

---

## Dev Agent Record

### Implementation Plan

Ordre suivi, conforme à l'ordre conseillé de la section Tasks : **T-B0 → T-B1 + T-B2 → T-B4 → T-B3 → T-B5**. T-B1 et T-B2 ont été écrites d'un seul tenant : ce sont deux `UPDATE` du même fichier de migration, et le miroir avoir n'est que la transposition `credit`/`debit` du critère de la facture — les séparer aurait produit deux fichiers de migration là où le dépôt en attend un.

**T-B0 (ordonnancement) — tranché en ground-truth avant toute écriture.** La migration de 16-1a est `20260727000001_invoice_lines_revenue_account.sql`, **dernière du dépôt** au moment du dev. Le backfill porte donc `20260729000001`, strictement postérieur en ordre lexicographique. Les deux stories vivant sur la même branche, la question du merge dans le mauvais ordre ne se pose pas.

### Debug Log

**Vérification de discrimination des tests (mutation testing) — le seul contrôle qui prouve que D-B3 est verrouillée.**

Les 8 tests passaient du premier coup, ce qui ne prouve rien : un test vert peut l'être parce que le code est correct **ou** parce qu'il n'observe rien. La spec affirme que la condition (2) écrite `<>` au lieu de `<=>` ferait no-oper le backfill *de façon indiscernable du succès*. J'ai donc muté la migration livrée (`<=>` → `<>`, les 2 occurrences des `UPDATE`) et rejoué la suite :

| | Résultat |
|---|---|
| Migration livrée (`<=>`) | **8/8 verts** |
| Migration mutée (`<>`) | **7 verts, 1 rouge** — `backfills_when_vat_config_is_null` seul |

> **⚠️ Renvoi ajouté en passe 3 de `bmad-code-review` — ne pas surinterpréter ce tableau.** Les deux occurrences ont bien été mutées, et un seul test est tombé. Mais l'unique test rouge ne couvrait que le volet **facture** : le volet **avoir** n'avait, à cet instant, **aucun** filet — muter son seul `<=>` aurait laissé toute la suite verte. La passe 2 l'a démasqué et a ajouté `backfills_credit_note_when_vat_config_is_null`. Le tableau ci-dessus mesure donc **un** volet sur deux, pas les deux. *(Entrée datée conservée telle quelle ; c'est le renvoi qui porte ce qu'on a appris depuis.)*

Message de l'échec du mutant : `left: [None, None]` / `right: [Some(4), Some(4)]`.

Deux enseignements, tous deux conformes à ce que la spec annonçait sans l'avoir démontré :

1. `backfills_when_vat_config_is_null` est bien le **seul** filet contre le piège n° 0. Sans lui, la régression passerait les 7 autres tests.
2. Le mode de défaillance est donc **empiriquement** indiscernable du succès, et non seulement en théorie.

Migration restaurée depuis sauvegarde après la mutation, `<=>` re-vérifié présent.

### Completion Notes

**Ce qui a été livré** — une migration de 2 `UPDATE`, 8 tests sur base pré-remplie, un module de test partagé, la ligne d'audit d'idempotence, l'entrée CHANGELOG, et la propagation sur 5 sites que la story rendait faux.

**Écart de conception assumé vs la spec — `LEFT JOIN` sur `company_invoice_settings`.** La spec ne se prononce pas sur le type de jointure. J'ai retenu `LEFT JOIN` : un `INNER JOIN` écarterait en silence toutes les factures d'une société **sans ligne de configuration**, reproduisant un cran plus haut exactement le mode de défaillance que D-B3 ferme (no-op indiscernable du succès). En pratique la ligne existe toujours pour une facture validée (`get_or_create_default_in_tx` fait `INSERT IGNORE` sur le chemin de validation), mais faire dépendre le backfill de cette propriété serait fragile. Absence de config = ensemble d'exclusion vide, exactement comme une colonne `NULL`.

**`MIN(jel.account_id)` n'est pas un arbitrage.** Le `HAVING COUNT(*) = 1` garantit un groupe d'une seule ligne : `MIN` **est** cette ligne. L'agrégat n'est là que parce que SQL l'exige sur une colonne non groupée.

**Extraction DRY — `tests/common/mod.rs`.** Le montage « appliquer N migrations, insérer en SQL brut, appliquer le reste » existait déjà dans `accounts_role_backfill.rs` (Story 14-3a). Plutôt que d'en faire une seconde copie, `apply_migrations_up_to` et la résolution **par version** ont été extraites dans `tests/common/mod.rs`, et le test 14-3a bascule dessus. Bénéfice au-delà du style : la résolution par version devient le **chemin unique et évident**, ce qui sert directement le garde-fou **P6**. La duplication résiduelle dans `migrations_upgrade_path.rs` est **laissée volontairement** — son helper porte un message d'assertion fail-loud propre à sa frontière positionnelle assumée, contrat différent de la résolution par version ; le fusionner diluerait précisément le garde-fou qu'il incarne.

**Garde-fou P6 appliqué (`grep -rn "migrations.len()\|apply_migrations_up_to" crates/`).** 3 sites inspectés. `migrations_upgrade_path.rs` est un site positionnel **assumé avec garde-fou fail-loud** : son `assert_eq!(total, 56)` a été porté à **57** et la fenêtre `total - 22` à `total - 23`, de sorte que la frontière reste **constante à 34** — c'est ce que son propre commentaire prescrit. Les 5 sites de nombres de ce fichier ont été mis à jour ensemble (assertion, message d'assertion du helper, doc-comment de la fonction, commentaire de frontière, prose historique) : le fichier documente lui-même qu'en 16-1a *trois copies du même symptôme y ont été découvertes une par passe*, faute d'avoir grepé à l'intérieur du fichier patché.

**Garde-fou P5 appliqué.** La ligne d'audit a été ajoutée **et** les compteurs **recomptés depuis le tableau** par script (57 lignes / `yes` 4 / `tracked-by-sqlx` 53 / `no` 0), avec contrôle croisé tableau ↔ fichiers `.sql` sur disque : **aucun écart**. Les compteurs annoncés avant modification étaient exacts (redressés en 16-1a).

**AC-B6 vérifié explicitement** : aucun `UPDATE _kesh_version` dans la migration, aucune opération `DROP`/`RENAME`/`MODIFY COLUMN`, aucune table créée → pas de bump `kesh_version_min_required`, donc pas de bump Cargo (workspace inchangé à **0.8.0**).

**Propagation post-patch — 5 sites que cette story rendait faux.** Le grep de `16-1a-bis` sur le dépôt a rendu 8 sites. Trois disent « validée avant 16-1a **et non traitée par 16-1a-bis** » et restent **exacts** (une ligne peut toujours ne pas être reprise). Ont été corrigés :

| Site | Ce qui devenait faux |
|---|---|
| `CHANGELOG.md` — puce « Factures validées avant cette version » | annonçait la limitation et « une mise à jour dédiée » : **remplacée** par la puce de reprise, sinon deux puces se contredisaient dans la même version |
| `CHANGELOG.md` — puce 16-1a, « pour toute facture validée **à partir de cette version** » | la ventilation miroir des avoirs couvre désormais aussi le parc antérieur backfillé |
| `routes/credit_notes.rs` — doc de `revenue_account_id` | affirmait « le backfill (16-1a-bis) **n'est pas livré**, ce qui couvre aujourd'hui l'essentiel du parc » |
| `routes/credit_notes.rs` — renvoi de test | pointait le nom d'avant renommage |
| `invoices_line_revenue_account.rs` — test de frontière 16-1a | cf. ci-dessous |

**Le test de frontière de 16-1a ne « change pas de verdict », et c'est correct.** Son doc-comment annonçait « le jour où le backfill est livré, ce test DOIT changer de verdict ». Il n'en est rien : le test remet la ligne à `NULL` **après** que les migrations ont tourné, donc le backfill ne la voit jamais. Ce qu'il décrit n'a jamais été le comportement du *parc antérieur* mais celui d'**une ligne `NULL`, quelle qu'en soit la raison** — ce qui reste vrai. Ce qui a changé est la **population** : de « tout le parc validé » à « les seules pièces dont l'écriture a été retouchée à la main ». Le test est donc conservé, **renommé** `legacy_invoice_…` → `null_line_credit_note_falls_back_to_current_default_known_limitation`, et son doc-comment ainsi que ses messages d'assertion recadrés. Le supprimer aurait perdu la seule caractérisation exécutable du résidu subsistant.

### ✅ Contradiction interne de la spec — AC-B3 cas 6 vs D-B5 — **RÉSOLUE**

**Constatée en écrivant le test, arbitrée en faveur de D-B5, non corrigée dans les AC** (le workflow interdit au dev de modifier les Acceptance Criteria).

> **Résolution — passe 1 de `bmad-code-review`, arbitrage Guy du 2026-07-29.** L'analyse ci-dessous a été **confirmée en ground-truth** par l'Acceptance Auditor (`create_credit_note` est l'unique chemin de création d'avoir **en production** et bascule inconditionnellement la facture en `cancelled`, `credit_notes.rs:561-574`). Guy a tranché : **corriger l'énoncé d'AC-B3 cas 6**, plutôt que consigner une dérogation. L'AC décrit désormais le comportement réel et la spec redevient opposable. Le doc-comment de `divergent_invoice_and_credit_note_are_recorded_as_is` a été recadré en conséquence — il n'y a plus d'écart spec ↔ code à y signaler.

- **AC-B3 cas 6** annonce : « facture validée à T1 sur défaut = 3000, défaut changé en 3200, avoir émis à T2 → le backfill écrit **3000 sur la facture** et 3200 sur l'avoir ».
- **D-B5** énonce l'inverse : « sur une facture créditée, `credit_note_lines.revenue_account_id` sera renseigné alors que `invoice_lines.revenue_account_id` **restera `NULL`** ».

**Ground-truth** : l'émission d'un avoir bascule **toujours** la facture d'origine en `cancelled` (`credit_notes.rs`, `UPDATE invoices SET status = 'cancelled' … AND status = 'validated'`, et un `rows == 0` fait échouer la transaction). Une facture créditée n'est donc **jamais** `validated`, et le backfill — qui ne traite que `validated` — ne la touche pas. **La première moitié d'AC-B3 cas 6 est inatteignable ; D-B5 dit vrai.**

Un test écrit littéralement d'après l'AC aurait échoué, et le réflexe naturel aurait été d'élargir le périmètre du backfill aux `cancelled` pour le faire passer — c'est-à-dire de casser D-B5 pour satisfaire une AC fausse.

La **substance** de D-B7 est intacte et reste verrouillée par `divergent_invoice_and_credit_note_are_recorded_as_is` : chaque pièce enregistre ce que **son** écriture a mouvementé, aucune harmonisation n'est tentée, et les deux comptes diffèrent. Le test assert les trois faits, dont `invoice_line_accounts == [None, None]` avec renvoi explicite à D-B5. *(Décision rendue — cf. l'encadré de résolution en tête de section.)*

## File List

**Ajoutés**

- `crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql` — les 2 `UPDATE` de backfill (T-B1, T-B2).
- `crates/kesh-db/tests/invoice_lines_revenue_account_backfill.rs` — **13** tests sur base pré-remplie (T-B4 ; 8 en `dev-story`, +1 en passe 1 de revue, +2 en passe 2, +2 après convergence sur demande de Guy).
- `crates/kesh-db/tests/common/mod.rs` — montage de fenêtre de migrations partagé, résolution par version (garde-fou P6).

**Modifiés**

- `crates/kesh-db/tests/accounts_role_backfill.rs` — bascule sur `tests/common` (extraction DRY).
- `crates/kesh-db/tests/migrations_upgrade_path.rs` — `total` 56 → 57, fenêtre `total - 22` → `total - 23` (frontière constante à 34), 5 sites de nombres alignés.
- `crates/kesh-db/tests/invoices_line_revenue_account.rs` — test de frontière renommé et recadré sur le résidu non reprenable.
- `crates/kesh-api/src/routes/credit_notes.rs` — doc de `revenue_account_id` (le backfill est livré) + renvoi de test renommé.
- `docs/migrations-idempotence-audit.md` — ligne d'audit + compteurs recomptés (T-B3, garde-fou P5).
- `CHANGELOG.md` — puce de reprise automatique avec les requêtes de diagnostic D-B4 (T-B5, AC-B8) ; puce de limitation 16-1a remplacée ; ventilation miroir des avoirs élargie au parc antérieur.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — statut de la story.

**Ajoutés en passe 1 de `bmad-code-review`**

- `crates/kesh-db/src/entities/invoice.rs` — doc de `revenue_account_id` : le backfill est livré, le résidu est nommé.
- `crates/kesh-db/src/entities/credit_note.rs` — idem, côté avoir.
- `crates/kesh-db/src/repositories/invoices.rs` — commentaire de `validate_invoice` (bloc de matérialisation).
- `crates/kesh-db/src/repositories/credit_notes.rs` — doc de `generate_credit_note_journal_lines`.
- `crates/kesh-api/src/exports/csv_tables.rs` — commentaire de la colonne exportée.
- `CLAUDE.md` — garde-fou **P5** : les **trois** sites du nombre de migrations énumérés + commandes de recompte.

---

## Change Log

### Post-convergence — 2026-07-30 : les deux tests du `LEFT JOIN` (demande de Guy)

**Le report de la passe 2 était une erreur d'appréciation, et la mutation l'a montré.**

La passe 2 avait signalé qu'aucun test ne couvrait le `LEFT JOIN` sur `company_invoice_settings` — « le repasser en `INNER JOIN` laisserait les 11 tests verts » — et je l'avais **reporté** au motif que le cas (société sans ligne de configuration) est inatteignable en production, `get_or_create_default_in_tx` faisant un `INSERT IGNORE` sur le chemin de validation. Argument doublement faible : c'est précisément celui que la story refuse (faire dépendre la correction du backfill d'une propriété d'un autre module), et surtout il portait sur la *valeur* du test sans rien dire de ce que son écriture allait révéler.

**Ce que la mutation a révélé** — en écrivant le test demandé, j'ai muté les **deux** `LEFT JOIN`, pas seulement celui signalé :

| Mutation | Résultat |
|---|---|
| `LEFT` → `INNER`, volet **facture** | `backfills_when_company_has_no_invoice_settings_row` **seul** rouge |
| `LEFT` → `INNER`, volet **avoir** | **les 12 tests verts** |

Le volet avoir n'avait **aucun filet non plus** — et personne ne l'avait signalé, ni les trois lentilles de la passe 2 qui ont trouvé le finding, ni les trois de la passe 3. C'est **la même classe d'asymétrie facture/avoir** que celle découverte en passe 2 sur le `<=>` NULL-safe : un garde-fou fileté d'un seul côté, l'absence de filet de l'autre indiscernable du succès.

**N'ajouter que le test demandé aurait donc reproduit la faute que cette boucle venait de corriger.** Deux tests livrés, chacun tuant sa mutation et rien d'autre. Le miroir est monté **exonéré de TVA**, seul état atteignable pour une société sans configuration.

**Enseignement, à verser à la rétro Epic 16** : un finding reporté au motif que « le cas est inatteignable » devrait quand même être **écrit** avant d'être reporté — c'est l'écriture du test, pas son analyse, qui a trouvé le second trou. Le coût du report n'était pas nul : il était de rater un garde-fou non fileté.

Gate complet vert : **2074/2074**, 4 skipped, exit 0.

### Passe 3 de `bmad-code-review` — 2026-07-30 (Haiku 4.5 ×3 lentilles) — **BOUCLE CONVERGÉE**

**13 findings bruts → 4 LOW retenus, 9 écartés dont 5 réfutés en ground-truth. Aucun finding au-dessus de LOW : le critère d'arrêt de la § Review Iteration Rule est atteint.**

**Trend : 6 → 24 → 0 au-dessus de LOW.** Rotation Sonnet → Opus → Haiku, contexte frais à chaque passe, diff unique aplati aux passes 2 et 3. Le pic de la passe 2 était bien un **effet de modèle**, pas une divergence de la story : la passe 3, sur un code stabilisé, ne trouve plus que de la documentation.

**Le profil Haiku est exactement celui que `CLAUDE.md` décrit, et la discipline de grep l'a contenu.** Cinq affirmations réfutées :

| Affirmation Haiku | Vérification | Verdict |
|---|---|---|
| « le verdict `yes` n'est pas dans le diff » (MEDIUM) | `grep -cF` = **1** dans le fichier **et** dans le diff | réfuté |
| « la mutation retirant la garde `IS NULL` n'a pas été exécutée » (MEDIUM) | présente au tableau de mutation de la passe 2, et rejouée | réfuté |
| « `migrations_upgrade_path.rs` n'est pas dans le diff » | **13** occurrences, dont les deux lignes `+` corrigées | réfuté |
| « condition (4) absente du volet avoir » (MEDIUM) | dissymétrie **voulue**, documentée — la lentille l'écrit elle-même deux lignes plus bas | non-finding |
| clause créance (**CRITICAL**) | re-litigation d'un arbitrage **déjà rendu** en passe 2, citant le Change Log de cette passe comme preuve | hors sujet |

Le rapport du Blind Hunter se contredit par ailleurs lui-même : sa section « sain » affirme que les deux volets sont couverts par mutation, ce que son propre finding n°2 nie.

**Un finding Haiku a néanmoins un vrai noyau, et il est bon.** Le Debug Log de `dev-story` annonce avoir muté « les 2 occurrences des `UPDATE` » pour « 7 verts, 1 rouge ». C'est factuellement exact, mais un lecteur en conclut que **les deux volets** sont filetés — or le seul test rouge ne couvrait que les factures, et le volet avoir n'avait alors **aucun** filet, ce que la passe 2 a démasqué. Renvoi ajouté sous le tableau daté, sans le réécrire.

**Le MEDIUM de l'Edge Case Hunter a été réfuté par lecture du code, pas écarté par argument.** Il signalait que le refus d'avoir sur compte archivé — arbitré en passe 1, tracé en #280 — n'était verrouillé par aucun test. Or `credit_note_fails_when_snapshot_account_archived` (`invoices_line_revenue_account.rs:568`) le teste **au niveau ligne** : ligne portant `revenue_account_id = 3200`, compte archivé après validation, avoir refusé, avec assertions sur `line_number`, `account_id` et `account_number`. La garde lit la colonne sans se soucier de son origine, et le backfill qui la remplit est verrouillé par ses propres tests ; composer les deux ne testerait que la composition de deux unités déjà couvertes.

**L'Acceptance Auditor rend CONFORME avec 0 finding**, et son rapport est **opposable** : matrice de couverture complète sur les 9 AC, les 7 décisions et les 11 tests, recompte P5 exécuté (57 fichiers, 57 lignes de tableau, 5 + 52 + 0), P6 vérifié site par site. C'est l'inverse des quatre rapports Haiku vides de la 16-1a — la contre-mesure « exiger une section *vérifié et jugé sain* » a produit l'effet attendu.

**4 LOW appliqués**, dont **deux trouvés par l'orchestrateur sur ses propres patches de passe 2** avant l'arrivée des rapports : le CHANGELOG n'énumérait pas l'exclusion du compte non-imputable alors que le critère venait de l'y ajouter ; et le message d'assertion du test de jonction énonçait une propriété de *toute* facture là où elle ne vaut que pour une facture mono-compte (résidu d'un finding informationnel de la passe 2 que j'avais laissé passer). Les deux autres : le renvoi du Debug Log ci-dessus, et le retrait d'un **nombre non sourcé** de mon Change Log de passe 2 (« l'Edge Case Hunter a recompté 7 sites ») — une affirmation chiffrée attribuée à une lentille n'est pas une vérification.

**Deux vérifications de mes patches de passe 2, faites avant les rapports** : le `JOIN` de la condition (4) ne peut pas fan-outer (`accounts.id` est `AUTO_INCREMENT PRIMARY KEY`, `20260411000001_accounts.sql:7`), donc `HAVING COUNT(*) = 1` reste sain ; et « seule migration du dépôt sans DDL » est vraie, **vérifiée par énumération des 57 fichiers** et non par affirmation.

**Enseignement pour la rétro Epic 16** — sur trois passes, **la remédiation a été la première source de défauts, deux fois sur trois**. La passe 2 a montré que le patch central de la passe 1 ne corrigeait rien *tout en documentant son succès* ; la passe 3 a trouvé deux résidus de la passe 2, et l'orchestrateur les avait devancés en auditant ses propres patches avant de lire les rapports. Ce geste — **auditer sa propre remédiation avant la passe suivante** — est ce qui manque encore à la § « Propagation post-patch » : elle demande de greper le symptôme, pas de rejouer l'argument du patch contre lui-même.

### Passe 2 de `bmad-code-review` — 2026-07-29 (Opus ×3 lentilles, contexte frais, diff aplati)

**~24 findings bruts, 12 MEDIUM → 4 arbitrages + 17 patches, 2 reports, 2 écartés.** Diff aplati `4od…` → `4db9003e..HEAD` en **un seul** diff, pour écarter le piège d'indexation multi-commit. Axe imposé aux trois lentilles : **traiter la remédiation de la passe 1 comme le matériau le plus suspect.** Le résultat justifie l'axe — la majorité des findings portent sur des artefacts produits en passe 1, y compris son patch central.

**Lire le chiffre avant de s'en alarmer.** Passe 1 (Sonnet) : 6 findings. Passe 2 (Opus) : 24. Ce n'est pas une divergence de la story mais un **changement de modèle**, et la 16-1a a exactement le même profil (9 → **21** → 6 → 1 → 14 → 5 → 2) avant d'avoir convergé. Le second critère de la § « Règle de splitting préventif » ne se déclenche donc pas à la lettre — il vise une sévérité non décroissante **à modèle comparable**. Point à trancher explicitement si la passe 3 ne redescend pas.

**TRIPLE CONVERGENCE — le patch central de la passe 1 ne corrigeait rien.** Les trois lentilles, indépendamment : `backfill_is_idempotent` était structurellement incapable de détecter la disparition de la garde `IS NULL`. Ses trois fixtures portaient une valeur qui est le **point fixe** du critère ; retirer les deux gardes les réécrivait à l'identique, et un test qui ne compare qu'« avant re-jeu » à « après re-jeu » restait vert. La passe 1 avait ajouté du montage — un avoir — sans ajouter de **discrimination**, et rédigé un doc-comment qui affirmait le contraire. C'est la démonstration la plus nette du mode d'échec que le projet a codifié : un patch qui déplace le symptôme sans toucher la cause, et qui documente son propre succès.

**TROIS MUTATIONS, TROIS TESTS TUÉS — un chacun, et le bon.** Les trois nouvelles gardes ne sont pas argumentées mais **mesurées** :

| Mutation appliquée à la migration livrée | Test rouge |
|---|---|
| les **2** gardes `WHERE … revenue_account_id IS NULL` retirées | `backfill_is_idempotent` **seul** |
| `<=>` → `<>` sur le **seul** volet avoir | `backfills_credit_note_when_vat_config_is_null` **seul** |
| garde `AND a.postable = TRUE` retirée | `leaves_null_when_credited_account_is_not_postable` **seul** |

**Un trou produit qu'aucune passe n'avait vu, et qui sort du périmètre.** Restaurer un backup antérieur au backfill rouvre le bug **définitivement** : la colonne est nullable donc `is_required()` est faux et l'import passe, et `_sqlx_migrations` est exclue du restore (`backup.rs:586`) donc la migration reste marquée appliquée et **ne repasse jamais**. Aggravant : la requête de diagnostic remonterait alors un décompte élevé que le CHANGELOG qualifie de bénin — l'unique instrument de détection misattribue la cause. Arbitré (rejouer le backfill après restore) et tracé en **issue #281**, avec réserve au CHANGELOG en attendant.

**Une affirmation centrale de la story était fausse.** « Les colonnes TVA sont `NULL` sur TOUTE installation, donc un `<>` no-operait sur 100 % du parc » — répété dans la migration, le doc d'audit, la spec et le Change Log de la passe 1. Or `validate_invoice` passe cette colonne au helper (`invoices.rs:1794`), qui **refuse de valider** dès qu'il y a de la TVA (`:1497-1500`) : une société dont la colonne est `NULL` n'a pu valider **que** des factures sans TVA. Le rayon réel n'est pas « 100 % du parc » mais **les installations exonérées de TVA** — cas réel en Suisse sous CHF 100'000, cœur de cible de Kesh, donc le mécanisme et le test gardent toute leur valeur. Mais le raisonnement faux menait à la conclusion inverse (« mon install a la config, donc `<>` est sûr »), et la fixture reconstituait un état que le moteur ne peut pas produire. Portée corrigée partout, fixture du nouveau test montée **exonérée**.

**Mon écartement de passe 1 s'appuyait sur le mauvais chemin.** J'avais écarté l'absence de contrôle `postable` en citant un commentaire qui justifie l'absence de contrôle pour la **contre-passation**. Or le premier `UPDATE` écrit dans `invoice_lines.revenue_account_id`, qui n'est pas un instantané du passé mais la source de vérité recopiée dans tout avoir futur. Condition (4) ajoutée sur le volet facture uniquement ; la dissymétrie avec le volet avoir est désormais explicite et motivée.

**Deux artefacts de la passe 1 étaient eux-mêmes faux** : la règle de recompte P5 (« les trois sites doivent donner le même nombre » — deux portent le total, le troisième est un compteur de partition ; l'appliquer aurait cassé l'invariant qu'elle protège), et la preuve invoquée à l'appui d'un arbitrage (`grep "INSERT INTO credit_notes"` → **3** occurrences, pas une). Plus une omission : le commentaire de `csv_tables.rs` ne citait pas la classe `cancelled` là où son jumeau, touché par le **même** patch, la citait.

**L'avertissement utilisateur avait disparu du CHANGELOG** en même temps qu'une ancre de code continuait d'y renvoyer (`routes/credit_notes.rs:60`). Vérifié : `grep -cF "Aucun message ne le signale"` = **0**. Le risque de résidu subsiste pourtant intégralement pour les pièces à écriture retouchée — ce que le dépôt prouve lui-même. Rétabli et reciblé sur la population résiduelle.

**Le verdict d'idempotence passe à `yes`** (arbitrage Guy) : sans DDL, la re-exécution manuelle **réussit**, et c'était la seule ligne `tracked-by-sqlx` du tableau sans code d'erreur MariaDB. Légende étendue, compteurs recomptés depuis la source → **57 = 5 `yes` + 52 `tracked-by-sqlx` + 0 `no`**.

**Sixième et septième sites de nombre périmés** dans `migrations_upgrade_path.rs` (`:95` « 13 dernières », `:212` « 10 restantes » pour une fenêtre de **23**) — dans le fichier même où la story revendiquait avoir aligné « les 5 sites », et dont un commentaire documente que « trois copies du même symptôme y ont été découvertes une par passe ». L'Edge Case Hunter annonçait en avoir recompté davantage que les 5 revendiquées ; son décompte exact n'a pas été reproduit et n'est donc pas repris ici — seuls les deux sites effectivement faux (`:95`, `:212`) sont corrigés. *(Nombre non sourcé retiré en passe 3 : une affirmation chiffrée attribuée à une lentille n'est pas une vérification.)*

**Checksum de migration — vérifié, sans risque.** Modifier un `.sql` déjà appliqué provoque `MigrateError::VersionMismatch`. Contrôle exécuté sur les trois bases locales : `kesh` et `kesh_e2e` n'ont **aucune** ligne pour `20260729000001`, `kesh_gate` n'a pas de table `_sqlx_migrations`. La migration n'étant par ailleurs pas publiée, la modifier est légitime.

**PROCHAINE = PASSE 3** (§ Review Iteration Rule). LLM ≠ Opus, contexte frais, diff aplati. Si la sévérité ne redescend pas nettement, appliquer la § « Règle de splitting préventif » plutôt que d'enchaîner.

### Passe 1 de `bmad-code-review` — 2026-07-29 (Sonnet ×3 lentilles, orchestration + ground-truth Opus 5)

**9 findings bruts → 6 retenus (5 MEDIUM, 1 LOW), 3 écartés.** Lentilles : Blind Hunter (diff seul), Edge Case Hunter (diff + dépôt), Acceptance Auditor (diff + spec + `CLAUDE.md`). Diff de commit **unique** (`4db9003e..c7a58635`), donc sans le piège d'indexation multi-commit.

**Le finding le plus sérieux ne vient d'aucune lentille.** L'Edge Case Hunter signalait l'absence de contrôle `account_type` — écarté, c'est un choix documenté et délibéré (`credit_notes.rs:405-409` : « ni `postable` ni `account_type` ne sont re-vérifiés, la contre-passation doit viser les **mêmes** comptes que l'écriture d'origine »). Mais en instruisant l'axe, c'est `active` qui mord : **le backfill peut rendre l'émission d'un avoir impossible.** Avant lui, une facture ancienne a toutes ses lignes `NULL`, et la garde de `create_credit_note` (`:410-419`) ne collecte que le défaut société courant — l'avoir passe. Après lui, elle contrôle le compte historique ; s'il a été **archivé** depuis, `CreditNoteRevenueAccountsArchived` (`:474`) refuse l'émission. Nouveau mode de défaillance **bloquant** sur du parc réel, couvert par aucun AC et **contredit par le CHANGELOG**, qui promettait « aucune action de votre part ». **Arbitrage Guy** : garder le refus (se replier recréerait le résidu que D5 combat) et le documenter → AC-B8 étendue + puce CHANGELOG avec la marche à suivre (réactiver, émettre, ré-archiver).

**AC-B3 cas 6 corrigée** (arbitrage Guy). L'AC exigeait que le backfill écrive le compte d'origine **sur la facture** — structurellement inatteignable, confirmé en ground-truth par l'Acceptance Auditor : `create_credit_note` est l'unique chemin de création d'avoir (une seule occurrence d'`INSERT INTO credit_notes`) et bascule inconditionnellement la facture en `cancelled` (`:561-574`). Le dev l'avait signalée sans la corriger, se déclarant hors mandat. L'AC décrit désormais le comportement réel ; la spec redevient opposable.

**Le trou de test qui comptait** — toute la puissance discriminante du critère tient à sa condition (3), `jel.credit = invoices.total_amount`, affirmée « par construction » dans le commentaire de la migration. Or les 8 tests montaient leurs écritures en **SQL brut avec le même littéral des deux côtés** : ils *supposaient* l'égalité au lieu de l'*observer* (0 occurrence de `validate_invoice` / `generate_invoice_journal_lines` / `compute_total` dans le fichier), et réciproquement `invoices_line_revenue_account.rs`, qui passe par le vrai moteur, ne rejoue jamais le backfill. Ajout de `validated_invoice_from_the_real_engine_is_recovered_by_the_backfill` : facture créée et validée par le **chemin applicatif réel**, montants non ronds (3 × 33.35 + 7 × 12.55 = 187.90) et deux taux de TVA distincts, puis retour à l'état pré-16-1a et backfill. **Discrimination prouvée par mutation** — `compute_total` muté en `Σ + 0.01` : **8 verts, 1 rouge, le nouveau test seul**, avec le message prévu (`obtenu : []`). Le mode de défaillance « le backfill no-opère sur 100 % du parc » est donc mesuré, plus seulement redouté.

**`backfill_is_idempotent` ne prouvait l'idempotence que d'un `UPDATE` sur deux** (convergence Blind Hunter + Edge Case Hunter) : aucun avoir dans sa fixture, donc le volet `credit_note_lines` se rejouait sur un ensemble vide — vrai de n'importe quel SQL — alors que `docs/migrations-idempotence-audit.md` affirme l'idempotence sans distinguer les deux. Volet avoir ajouté, AC-B4 précisée.

**Garde-fou P5 pris en défaut par la story qui le cite.** L'en-tête `## Table d'audit (56 migrations)` était resté à 56 pour **57** réelles — troisième site du même nombre dans le fichier, et le seul oublié des trois. Corrigé, puis **fix structurel** plutôt que ponctuel : `docs/migrations-idempotence-audit.md` et le P5 de `CLAUDE.md` **énumèrent désormais les trois sites** et donnent les deux commandes de recompte. La consigne « recomptez » ne suffisait pas — elle ne disait pas *quoi*.

**Propagation post-patch — 3 sites de plus, dont un qu'aucune lentille n'avait vu.** Le grep de `16-1a-bis` donne 5 sites de doc périmés (Edge Case Hunter en avait 4, l'Acceptance Auditor 4 aussi, **pas les mêmes** — l'union fait 5) ; deux d'entre eux (« leur traitement **relève de** la Story 16-1a-bis ») étaient devenus franchement faux. S'y ajoutent, trouvés par le grep post-patch et par **aucune lentille** : le doc-comment de `divergent_invoice_and_credit_note_are_recorded_as_is`, qui proclamait « AC-B3 cas 6 et D-B5 se contredisent » — rendu faux par **la correction de l'AC elle-même** ; et la ligne d'audit de `20260727000001`, qui annonçait 16-1a-bis « tant qu'elle n'est pas livrée ». Illustration directe de la seconde moitié manquante de la § Propagation, relevée en rétro 16-1a : greper le symptôme ne suffit pas, il faut greper **ce qui pointe vers ce qu'on vient de corriger**.

**3 findings écartés en ground-truth** : contrôle `account_type` (choix délibéré documenté, cf. supra) ; modification de `routes/credit_notes.rs` « hors périmètre » (doc-comments seuls, aucune logique runtime — c'est précisément ce qu'exige la § Propagation) ; absence de test sur `default_receivable_account_id` (exclusion prouvablement redondante — sur l'écriture facture la créance est un **débit**, déjà éliminée par `jel.credit > 0`).

**DEUX GATES COMPLETS VERTS, la baseline d'abord.** État pré-patches : **2069/2069, 4 skipped, exit 0** — l'annonce du dev est donc vérifiée, et non reprise de confiance. État post-patches : **2070/2070, 4 skipped, exit 0**. Les seules modifications postérieures au second gate sont des doc-comments et du markdown ; `cargo fmt --check` + `cargo clippy --workspace --all-targets -D warnings` re-passés verts, et les 3 fichiers de test touchés rejoués (9 + 25 + 8, tous verts). **Piège d'environnement rencontré** : deux lancements de gate perdus avant d'obtenir un résultat exploitable — `$TMPDIR` vide hors sandbox, puis mot de passe MariaDB erroné produisant 6 échecs `auth::bootstrap` en `Access denied` qui ressemblaient à une régression. Un gate rouge se lit d'abord dans son message d'erreur.

**PROCHAINE = PASSE 2** (§ Review Iteration Rule : 5 findings > LOW en passe 1). LLM différent de Sonnet, contexte frais, diff aplati `HEAD` vs `main`.

### `bmad-dev-story` — 2026-07-29 (Opus 5)

**Story implémentée bout-en-bout, 6/6 tâches.** Migration `20260729000001_invoice_lines_revenue_account_backfill.sql` (2 `UPDATE`, aucun DDL), 8 tests sur base pré-remplie, extraction DRY du montage de fenêtre de migrations, audit d'idempotence, CHANGELOG, propagation sur 5 sites.

**Le fait marquant : la discrimination des tests a été prouvée, pas supposée.** Les 8 tests passaient du premier coup. Pour vérifier que le filet de D-B3 mord réellement, la migration livrée a été **mutée** (`<=>` → `<>`) et la suite rejouée : **7 verts, 1 rouge — `backfills_when_vat_config_is_null` seul**. La spec annonçait que le mode de défaillance serait « indiscernable du succès » ; c'est désormais **mesuré**, pas argumenté. Migration restaurée, `<=>` re-vérifié.

**Contradiction interne de la spec constatée** — AC-B3 cas 6 (« le backfill écrit 3000 **sur la facture** ») est **inatteignable** : l'émission d'un avoir bascule toujours la facture en `cancelled`, statut que D-B5 exclut délibérément. **D-B5 dit vrai.** Arbitré en faveur de D-B5, AC non modifiée (hors mandat du dev), test écrit sur le comportement atteignable — la substance de D-B7 reste entièrement verrouillée. Détail et ground-truth en Dev Agent Record. ~~**En attente d'arbitrage de Guy.**~~ → **Tranché en passe 1 de `bmad-code-review` (2026-07-29) : l'AC a été corrigée.**

**Deux écarts de conception assumés vs la spec**, tous deux documentés en Dev Agent Record : `LEFT JOIN` sur `company_invoice_settings` (un `INNER JOIN` reproduirait le no-op silencieux de D-B3 un cran plus haut, pour les sociétés sans ligne de config) ; extraction de `tests/common/mod.rs` plutôt qu'une seconde copie du montage de 14-3a — la duplication résiduelle de `migrations_upgrade_path.rs` est laissée volontairement, son helper portant un garde-fou fail-loud propre à sa frontière positionnelle.

**Garde-fous de `CLAUDE.md` appliqués** : **P6** — 3 sites de couplage positionnel inspectés, `migrations_upgrade_path.rs` porté à `total == 57` / fenêtre `total - 23` (frontière constante à 34), ses **5** sites de nombres alignés d'un seul patch ; **P5** — ligne d'audit ajoutée et compteurs **recomptés depuis le tableau** par script avec contrôle croisé sur les `.sql` du disque, aucun écart (57 / 4 / 53 / 0) ; **P1-P2-P2-bis** — aucun `DROP`/`RENAME`/`MODIFY COLUMN`, pas de bump `min_required`, workspace inchangé à 0.8.0 (AC-B6).

**Propagation post-patch** : le grep de `16-1a-bis` a rendu 8 sites, dont **5 rendus faux par cette story** et corrigés dans le même patch — notamment le test de frontière de 16-1a, dont l'annonce « ce test DOIT changer de verdict » s'est révélée inexacte (il remet la ligne à `NULL` **après** les migrations, donc le backfill ne la voit jamais). Test conservé, renommé et recadré : ce qui a changé n'est pas son verdict mais la **population** qu'il décrit.

**Gate (AC-B9) — VERT sur l'état final** : `cargo fmt --all -- --check` ✅, `cargo build --workspace --all-targets` ✅, `cargo clippy --workspace --all-targets -- -D warnings` ✅ (0 warning, notamment aucun `dead_code` sur le module `tests/common`), suite workspace complète **2069/2069 passés, 4 skipped, exit 0** (2899 s sur DB `kesh_gate`). Les suites que la story met en risque sont toutes vertes : `migrations_fresh_install` (3), `migrations_upgrade_path` (8, dont `upgrade_path_preserves_data` et son `assert_eq!(total, 57)`), `accounts_role_backfill` (3), `admin_backup_e2e` / `backup_inventory_matches_schema` et `exports_global_e2e` (AC-B7 : aucune table créée, compteurs d'export inchangés), plus les **8 nouveaux** de `invoice_lines_revenue_account_backfill`.

**Le gate a été rejoué intégralement sur l'état final, il n'a pas été présumé.** Un premier gate avait été lancé puis **abandonné à 574/2069** : l'ajout tardif de l'assertion sur la seconde requête de diagnostic (celle des avoirs) rendait son binaire de test périmé pour cette suite. Composer « le gate d'avant + un `cargo test` ciblé » aurait été exactement le raccourci que la 16-1a s'était interdit. `kesh_gate` a été contrôlée après l'arrêt en vol (piège connu `postable = 0` sur le compte 1000 → 26 faux échecs) : **intacte**.

**Prochaine étape** : `bmad-code-review` (LLM ≠ Opus, contexte frais).

### Création par split de 16-1a — 2026-07-26 (passe 6 de `validate`)

Story issue de l'extraction de la décision **D2-bis** de 16-1a et de tout son corpus (critère de backfill, AC2-bis, volet backfill de T1, piège n°0), sur arbitrage de Guy après déclenchement du second critère de la § « Règle de splitting préventif » sur 16-1a (P5 : 1 HIGH → P6 : 2 HIGH, sévérité non décroissante).

**Motif** : sur les passes 5 et 6, **7 findings sur 10 — dont les 3 HIGH — portaient sur le seul backfill**, alors que le reste de 16-1a n'était plus remis en cause depuis la passe 3. Profil de risque étranger (migration de données comptables réelles vs schéma et logique applicative), et saturation des passes adversariales de 16-1a.

**Le contenu est repris dans l'état convergé des passes 4 à 6**, sans régression :

| Origine | Apport |
|---|---|
| P4 (CRITICAL) | Existence même du backfill — 16-1a seule protège un ensemble **vide** à l'instant du déploiement |
| P4 | Source = l'écriture générée, **jamais** le défaut courant (D-B1) |
| P5 (HIGH) | L'écriture est **éditable** (aucune colonne `source`, `PUT` sans garde de provenance, `line_order` réattribué) → critère d'unicité conservateur avec le **montant** pour discriminant (D-B2) |
| P5 | Suppression de la post-condition « plus aucun `NULL` », **fausse** et pousse-au-crime (AC-B2) |
| P6 (HIGH) | Condition (2) en **`<=>` NULL-safe** — les colonnes TVA sont `NULL` partout, un `<>` fait no-oper le backfill sur 100 % du parc de façon indiscernable du succès (D-B3) |
| P6 (MEDIUM) | Décompte tranché : **aucun artefact nouveau**, requête de diagnostic (D-B4) |
| P6 (MEDIUM) | Cas de test « facture validée sans écriture » retiré — **inconstructible** (`chk_invoices_validated_has_je`) |
| P6 (LOW) | `total_ht` → `invoices.total_amount` ; motif d'exclusion des `cancelled` corrigé (D-B5) ; verdict d'idempotence redressé (D-B6) |

**Statut de revue à la création** : le corpus avait été revu en passes 5 (Haiku + orchestrateur) et 6 (Opus) **au sein de 16-1a**, mais jamais comme un tout autonome — d'où la passe 1 ci-dessous.

### Passe 1 de `validate` — 2026-07-26 (Sonnet, contexte frais)

**2 findings : 1 HIGH, 1 MEDIUM.** Tous deux vérifiés en ground-truth avant application. Le HIGH est né **du split lui-même** : le miroir avoir, simple corollaire dans 16-1a, est traité ici au même rang que la facture — et ce changement de statut a révélé une affirmation fausse que personne n'avait interrogée en 4 passes.

| Finding | Verdict | Traitement |
|---|---|---|
| **HIGH — AC-B3 cas 5 affirmait que le compte backfillé de l'avoir « égale celui de la facture d'origine ».** C'est **faux précisément dans le scénario qui motive la story**. `create_credit_note` relit la configuration au moment de l'émission (`credit_notes.rs:275-282`, `get_or_create_default_in_tx` puis `settings.default_revenue_account_id`), indépendamment de ce que la facture a crédité — c'est le bug du tableau T1/T1+/T2 du § Contexte. Les avoirs déjà en base ont été produits par ce code : leur écriture débite légitimement un **autre** compte que la facture | **Réel.** L'AC ne laissait que deux issues, toutes deux mauvaises : un test sans changement de défaut, qui passe trivialement et **ne prouve rien** (l'écueil que le cas 1 signale explicitement, jamais repris au cas 5) ; ou un test fidèle au scénario réel, qui **échoue** — non parce que le backfill est bugué, mais parce que l'AC affirme une invariance inexistante | **D-B7 ajoutée** — « le backfill ENREGISTRE le résidu historique, il ne le RÉPARE pas » : le résidu existe déjà dans les écritures, il est passé et irréversible ; forcer l'égalité falsifierait les pièces sans corriger la moindre écriture. Ce que la story corrige (avoirs futurs, via 16-1a D5) et ce qu'elle ne corrige pas (avoirs passés, qui relèveraient d'une écriture de reclassement — acte comptable, pas migration) est explicité. **AC-B3 cas 5 réécrit** (compte déterminé indépendamment de la facture) et **cas 6 ajouté** : le couple divergent est le cas qui compte, avec les deux valeurs attendues. Piège n°6 ajouté. **Conséquence répercutée sur 16-1b** : l'écran de détail peut légitimement afficher deux comptes différents pour une facture et son avoir ; ne pas le signaler comme une anomalie, ne pas tenter de l'harmoniser |
| MEDIUM — la dépendance d'ordre avec la migration `ADD COLUMN` de 16-1a est affirmée en prose (« dépend strictement de 16-1a ») mais **jamais opérationnalisée**. `sqlx::migrate!` exécute dans l'ordre **lexicographique du nom de fichier** (`kesh-db/src/lib.rs:23`) : un timestamp mal choisi, ou un merge de PR séparées dans le mauvais ordre, fait jouer le backfill avant la création de la colonne | Réel. Échec **bruyant** (`Unknown column`), attrapé en CI — donc pas de corruption silencieuse, mais un cycle perdu, et un gap de complétude réel pour une story qui documente par ailleurs tous ses autres risques d'ordonnancement | **T-B0 ajoutée** : contrainte de timestamp explicite, vérification de l'état de 16-1a avant de choisir, et consigne de merger 16-1a en premier ou de regrouper les deux dans la même PR (cohérent `feedback_pr_grouping` — les deux touchent le même répertoire de migrations). Piège n°7 ajouté |

**Vérifié négatif (substantiel — ne pas ré-instruire)** : (1) **le compte de produit ne peut jamais collisionner avec le compte de créance** dans l'ensemble d'exclusion `E` — `validate_account` (`routes/company_invoice_settings.rs:94-120`) impose `account_type == expected` (`Asset` pour la créance, `Revenue` pour le produit) et `chk_accounts_type` (`20260411000001_accounts.sql:20`) ferme la liste : un même compte ne peut satisfaire les deux types. (2) **`credit_notes.total_amount` est bien du HT**, miroir strict d'`invoices.total_amount` (commentaire de schéma `20260627000001_credit_notes.sql:19` + code `total_ht = Σ line_total` inséré tel quel). (3) **Les avoirs partiels n'existent pas** : `create_credit_note` est la seule fonction publique de création et snapshot **toutes** les `invoice_lines` sans filtre — chaque avoir est nécessairement total. (4) Faisabilité MariaDB confirmée (ER 1093 inapplicable, aucune CTE dans le dépôt, précédent multi-table `20260628000001:115`). (5) **Pas de risque de scan complet** : `idx_jel_entry` sur `journal_entry_lines(entry_id)` (`20260412000001:47`). (6) **Aucune contrainte DB ne bloque un `UPDATE` sur une facture d'exercice clos** — la garde est purement applicative, hors périmètre d'une migration SQL. (7) Traçabilité D-B1..D-B7 → AC-B1..AC-B9 → T-B0..T-B5 complète, aucun orphelin. (8) **Précédent de test sur base pré-remplie** : `crates/kesh-db/tests/accounts_role_backfill.rs` valide le pattern exigé par T-B4.

**Trend** : passe 1 = 2 findings (1 HIGH, 1 MEDIUM). Sévérité au-dessus de LOW → **passe 2 requise** (contexte frais, modèle différent). Cible prioritaire : **D-B7 et les cas 5-6 d'AC-B3**, patch tout neuf — en particulier la cohérence du récit « enregistrer sans réparer » avec D-B1 et avec le § Contexte, et sa répercussion sur 16-1b.

### Passe 2 de `validate` — 2026-07-26 (Haiku 4.5, contexte frais + vérification orchestrateur)

**1 finding LOW.** Le reviewer a rendu « 0 finding » après avoir instruit les 6 angles imposés ; le LOW vient de la vérification d'orchestrateur — conformément à la leçon consignée en passe 5 de 16-1a (« un *rien trouvé* de Haiku n'est pas une preuve de convergence »).

| Finding | Verdict | Traitement |
|---|---|---|
| LOW — le § « **Le bug fermé par cette story** » présente le scénario T1/T1+/T2 dont **T2 est déjà passé** (« Avoir total émis »), c'est-à-dire exactement le cas que **D-B7 dit ne PAS réparer**. Le titre et le tableau promettent donc une fermeture que la décision structurante retire trois sections plus loin. La clause `so that` de l'en-tête portait la même imprécision (« un avoir émis plus tard », sans dire *plus tard que quoi*) | Réel, et **même racine que le HIGH de la passe 1** : le symptôme n'avait pas été entièrement grepé après le patch (§ « Propagation post-patch » de `CLAUDE.md`). Sévérité LOW et non HIGH parce qu'aucun AC ni aucune tâche n'en dépend — c'est du récit, pas une prescription | Tableau des **deux populations** ajouté au § Contexte : *avoir pas encore émis* (immense majorité) → **le bug est fermé** ; *avoir déjà émis* → **rien n'est réparé**, les deux comptes diffèrent légitimement (D-B7). Formule de synthèse : le backfill est **préventif, pas curatif** — il arme les factures passées pour que leurs avoirs **à venir** soient corrects. Clause `so that` de l'en-tête précisée (« un avoir **émis à partir de maintenant** sur une facture ancienne ») |

**Vérifié négatif (confirmations du reviewer, contrôlées)** : (1) **cohérence D-B7 ↔ D-B1 ↔ 16-1a D5/AC11** — 16-1a D5 fait bien copier le compte depuis la ligne de facture à la création de l'avoir, et AC11 impose la transformation du site d'appel `credit_notes.rs:320-328` en triplets ; les avoirs futurs ne reliront donc plus `settings`, ce qui rend le récit de D-B7 exact. (2) **Le cas 6 d'AC-B3 est constructible** : la configuration est mutable, `create_credit_note` relit bien à l'émission (`credit_notes.rs:276`), `uq_credit_notes_invoice` n'entrave pas le scénario, et le pattern de test sur base pré-remplie a son précédent (`accounts_role_backfill.rs`). (3) **T-B0 correcte** : `sqlx::migrate!` trie lexicographiquement, le nommage `YYYYMMDDNNNNNN_*.sql` du dépôt rend la contrainte de timestamp triviale à respecter. (4) Traçabilité D-B1..D-B7 → AC-B1..AC-B9 → T-B0..T-B5 complète, aucun orphelin. (5) Conformité `CLAUDE.md` : P1/P2/P2-bis (AC-B6), P5 audit d'idempotence (AC-B5), Test Locally First (AC-B9).

**Trend** : passe 1 = 2 findings (1 HIGH, 1 MEDIUM) → passe 2 = **1 LOW**. **Critère d'arrêt de la § « Review Iteration Rule » atteint** : plus aucun finding au-dessus de LOW. **16-1a-bis est convergée** en 2 passes.
