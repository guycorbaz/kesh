# Story 15.6a : L'avoir contre-passe la créance et l'arrondi sur les comptes que la vente a mouvementés

## Status

ready-for-dev

<!-- Spécifiée le 2026-10-08 en autonomie (bmad-create-story), fille de la 15-6 découpée d'emblée
     (choix C-15-6-1). Choix propres : C-15-6-2 (révisé par C-15-6-7 et C-15-6-8). Validation P1
     (Sonnet, lentilles F et R) appliquée le 2026-10-08 : absorbe #523. Validation P2 (Opus, lentilles
     F et R) appliquée le 2026-10-08 : comptes de la vente verrouillés, refus nommé (C-15-6-16).
     Validation P3 (Sonnet, lentilles R et F) appliquée le 2026-10-08 : verrou en partage AVANT
     l'exercice, refus propre à l'avoir (C-15-6-24, C-15-6-25). Validation P4 (Opus, lentilles R et F)
     appliquée le 2026-10-08 : verrou étendu à tous les comptes écrits, rejeu de la route (C-15-6-29).
     Validation P5 (Sonnet, lentilles R et F) appliquée le 2026-10-08 : alignement sur la 15-5e telle
     que C54 l'a réécrite — la défense contre l'interblocage est le rejeu (la 15-5e rejoue la route de
     l'avoir), l'ordre des verrous une convention de fréquence ; verrou gardé pour la seule course de
     lecture, rien réordonné pour fermer un cycle, cycles connus nommés sans prétendre à l'absence
     d'autres, helper de verrou partagé avec la 15-5d (C-15-6-32). Validation P6 (Opus, lentilles R
     et F) appliquée le 2026-10-08 : un seul contrat pour le helper de verrou, critère DRY à exception
     écrite, doctrine résiduelle retirée, partenaires réels du cycle (iv) (C-15-6-35). -->

**Issues** : `closes #473, closes #523` (P1 toutes deux) — les mots-clés vont **dans la PR** (merge en
squash). **Mère** : `15-6-creance-juste-avoir-reglement.md`.
**Suivie de** : 15-6b (mêmes fichiers `invoice_settlements_write.rs` et `reconciliation.rs` — la 15-6b
emploie le lecteur que celle-ci pose).
**Après** les 15-5a à 15-5e (ordre de l'epic : 15-5 puis 15-6 ; la **15-5e** passe avant la 15-5d) —
dépendance **ferme** pour chacune, la **15-5d** comprise (son helper de verrou, son test de C35) : si
l'une n'est pas mergée au moment de T0, la story **attend** (finding R6-10 de la P6).
Ces fiches touchent les **mêmes fonctions** : `accept_one_invoice` (15-5b AC6 y pose un commentaire de
classement sur le bloc même que l'AC2 remplace), `settle_invoice` (15-5a AC4, 15-5b AC18),
`write_off_invoice` (**15-5e AC5** : commentaire « 5 bis » réécrit en place), `validate_invoice`
(15-5b, 15-5d, et **15-5e AC5**, qui réécrit son doc-comment « # Ordre des locks » — l'ordre réel,
**convention qui réduit la fréquence** des interblocages, et **la défense, qui est le rejeu** : la
règle de l'epic, choix C54, que cette fiche **suit** et ne réécrit pas, AC6, AC7), la **route
`POST /api/v1/credit-notes`** (**15-5e AC3** : rejouée sur interblocage par l'enveloppe `DbError`,
`kesh_db::retry::retry_on_deadlock` — la 15-6a ne pose donc **pas** de second rejeu, AC6), le
Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (15-5e AC5), `invoices_validate_vat.rs` (15-5b, 15-5d), le
passage du manuel sur l'avoir (15-5d AC8), le **helper de verrou d'une liste de comptes** que la 15-5d
pose (`lock_designated_accounts_in_tx`, AC6) et le **fichier de tests** `credit_notes_repository.rs`,
où la 15-5d ajoute un test (C35) que cette fiche ré-ancre (§ *Tests*, 13). Les numéros
de ligne de cette fiche sont ceux du commit `1920381e` : **T0 les refait tous** après ces merges.
**Angle mort tracé, hors story** : #525 (TVA due de l'avoir lue dans les réglages du moment — jalon de
la TVA, report assumé ; choix C-15-6-8).

## Story

En tant que **comptable d'une PME**,
je veux que **l'avoir contre-passe la créance et l'écart d'arrondi sur les comptes mêmes que la facture
a mouvementés**,
afin que **le compte débiteurs de la facture et le compte de différences d'arrondi reviennent à zéro**,
comme le promet le manuel, même si les comptes par défaut ont été changés entre la validation et
l'avoir.

## Le défaut, établi au code

- La validation d'une facture débite la créance sur `settings.default_receivable_account_id`
  (`crates/kesh-db/src/repositories/invoices.rs:2005-2010`) — c'est le geste qui **crée** la créance,
  il lit légitimement les réglages. Elle pousse l'écart d'arrondi en **dernière** ligne
  (`generate_invoice_journal_lines_rounded`, `invoices.rs:1892-1912`) : au **crédit** du compte
  d'arrondi si l'arrondi est positif, au **débit** s'il est négatif.
- L'avoir relit **les réglages du moment** :
  - pour la créance (`crates/kesh-db/src/repositories/credit_notes.rs:360-364`), créditée en ligne 0
    (`generate_credit_note_journal_lines`, `:187`, ligne poussée `:219`, appel `:509`) — #473 ;
  - pour le compte d'arrondi (`credit_notes.rs:525`, `rounding_account_for_write(.., Issuance)`,
    `company_invoice_settings.rs:400-412`) — #523 ;
  - pour la TVA due (`credit_notes.rs:511`, `settings.default_vat_payable_account_id`) — #525, hors
    story (choix C-15-6-8).
- Le règlement, le solde du reste et l'encaissement par rapprochement lisent, eux, la créance **sur
  l'écriture de vente** — première ligne au débit, `ORDER BY jel.id LIMIT 1` — par **trois copies**
  de la même requête : `invoice_settlements_write.rs:104-115` (`settle_invoice`),
  `invoice_settlements_write.rs:434-445` (`write_off_invoice`), `crates/kesh-api/src/routes/reconciliation.rs:1425-1452`
  (`accept_one_invoice`). La génération garantit que la créance est cette ligne-là
  (`invoices.rs:1870-1878`, doc de `generate_invoice_journal_lines_rounded`).
- Rien n'interdit de changer le compte débiteurs ou le compte d'arrondi par défaut entre-temps
  (`crates/kesh-api/src/routes/company_invoice_settings.rs:274-281`, seuls type et état actif).

**Conséquence** : l'avoir crédite le nouveau compte ; l'ancien garde le débit TTC ; la facture passe
`cancelled` et sort de la balance âgée — la créance reste au grand livre sans plus apparaître nulle
part. Même chose, au centime, pour le compte d'arrondi (écart ≤ 2,5 centimes par facture, mais une
écriture fausse). Le commentaire `credit_notes.rs:609-611` — « c'est ce qui rend la contre-passation
l'« inverse exact » même si le défaut société change ensuite » (cité `:574-576` par l'issue, sur une
version antérieure) — est vrai du compte de produit, recopié par ligne (Story 16-1a, D5), pas de la
créance ni de l'arrondi.

## Acceptance Criteria

1. **AC1 — Un lecteur partagé du compte de créance.** `crates/kesh-db/src/repositories/invoice_settlements.rs`
   expose `pub async fn sale_receivable_account(conn: &mut sqlx::MySqlConnection, company_id: i64, sale_entry_id: i64) -> Result<Option<i64>, DbError>`
   (signature indicative ; l'exécuteur suit le patron de `rounding_account_for_write`), qui exécute
   **exactement** la requête des trois copies actuelles (première ligne `debit > 0` de l'écriture,
   `ORDER BY jel.id LIMIT 1`, portée par `je.company_id`). Son doc-comment porte la règle — « la
   créance se lit sur l'écriture de vente, jamais sur les réglages » — et la raison de l'ordre (arrondi
   négatif au débit **après** la créance, 25-4-c4-a). `None` = écriture sans ligne de débit : chaque
   appelant garde **son** refus actuel (`DbError::Invariant` pour le règlement et le solde ;
   `failed[]` `INVOICE_SALE_ENTRY_MALFORMED` pour le rapprochement, `details` inchangés —
   `{ "reason": "no_debit_line_on_sale_entry", "saleEntryId" }`). **Erreur SQL** du lecteur dans le
   rapprochement : l'appelant la convertit, comme aujourd'hui, en `FailedProposal` `DATABASE_ERROR`
   avec `details = { "message": e.to_string() }` — **forme inchangée** ; le **texte** du message, lui,
   dérive désormais d'un `DbError` (`map_db_error`, `crates/kesh-db/src/errors.rs:796`) et non plus
   d'un `sqlx::Error` : son libellé peut différer. Le rejeu sur interblocage n'en dépend pas (il
   repose sur la perte du point de sauvegarde, `reconciliation.rs:1057-1068`).
2. **AC2 — Les trois copies disparaissent.** `settle_invoice`, `write_off_invoice` et
   `accept_one_invoice` appellent le lecteur ; `grep -rn --include='*.rs' "jel.debit > 0" crates/` ne
   rend plus que le lecteur et la doc de `generate_invoice_journal_lines_rounded`
   (`invoices.rs:1877`). Les `.sql` que le même jeton atteint —
   `crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql:293`,
   `crates/kesh-db/migrations/20260828000001_invoice_settlements_type.sql:51` et le rejeu
   `crates/kesh-db/src/post_restore/20260828000001_invoice_settlements_type.sql:25` — sont des
   backfills, **hors classe** et légitimes (une migration appliquée ne se modifie plus, règle P8).
   Comportement inchangé : les tests existants (`crates/kesh-db/tests/invoice_settlement.rs`,
   `invoice_write_off.rs`, `crates/kesh-api/tests/reconciliation_e2e.rs`) passent sans modification
   d'assertion. ⚠️ **Aucun test ne fige aujourd'hui `INVOICE_SALE_ENTRY_MALFORMED`** (vérifié à la
   remédiation P1 : `grep -rn "INVOICE_SALE_ENTRY_MALFORMED" crates/ frontend/src docs` ne rend que le
   site de `reconciliation.rs`) : un test l'ajoute avant le remplacement (§ *Tests*, 10), pour que la
   copie remplacée garde un témoin.
3. **AC3 — L'avoir crédite la créance de la vente.** `create_credit_note` lit la créance par le
   lecteur de l'AC1, sur `invoice.journal_entry_id` de la facture verrouillée ; `None` sur
   `journal_entry_id` ou sur le lecteur → `DbError::Invariant` (« facture validée sans écriture de
   vente » / « écriture de vente sans ligne de débit »), comme `settle_invoice`. La lecture se fait
   **après** le verrou de la facture (étape (1)) et la lecture des réglages (3), **avant** l'exercice
   (4) — elle ne dépend que de `invoice.journal_entry_id`, connu dès (1), et le verrou de l'AC6 en a
   besoin —, dans la même transaction. L'avoir **ne lit plus**
   `settings.default_receivable_account_id` : un réglage débiteurs vide n'empêche plus d'émettre un
   avoir sur une facture validée.
4. **AC4 — L'avoir contre-passe l'arrondi sur le compte de la vente (#523, choix C-15-6-7).** Un
   second lecteur, à côté du premier :
   `pub async fn sale_rounding_account(conn: &mut sqlx::MySqlConnection, company_id: i64, sale_entry_id: i64, rounding_amount: Decimal) -> Result<i64, DbError>`
   (signature indicative). Il lit la **dernière** ligne de l'écriture de vente
   (`ORDER BY jel.id DESC LIMIT 1`, portée par `je.company_id`) et la **recoupe** avec l'arrondi figé
   sur la facture : arrondi > 0 ⇒ `credit = rounding_amount` et `debit = 0` ; arrondi < 0 ⇒
   `debit = −rounding_amount` et `credit = 0`. Toute autre forme (pas de ligne, sens ou montant
   différent) → `DbError::Invariant` (« écriture de vente : la dernière ligne n'est pas l'arrondi de la
   facture »). Il n'est **appelé que si** `invoice.rounding_amount ≠ 0` — une facture émise sans
   arrondi, antérieure comprise, n'exige toujours aucun compte (validation P3 de la 25-4-c4-a). Il est
   appelé **à côté du lecteur de créance**, avant l'exercice (4) (AC3, AC6) ; l'étape (7 bis) de
   `create_credit_note` ne lit plus `rounding_account_for_write(.., Issuance)` et emploie l'id déjà lu ;
   la ligne produite (sens, montant, ajustement de la ligne 0) ne change pas. Son doc-comment dit
   pourquoi le recoupement (un lecteur positionnel qui ne vérifie rien rendrait n'importe quelle ligne
   en silence) et pourquoi **pas** `usable_designated_account` (doctrine « mêmes comptes que
   l'origine, seule l'inactivité bloque », cf. 6 ter de la 16-1a).
5. **AC5 — Le reste de l'avoir ne bouge pas.** Compte de produit par ligne (D5) et garde des comptes
   de produit archivés (6 ter), compte de TVA due (réglage courant — angle mort #525, AC8), libellé,
   numéro, journal, projet, snapshot des lignes : inchangés. L'ordre des lignes de l'écriture aussi
   (créance en ligne 0, arrondi en dernière ligne). L'avoir **exige toujours**
   `default_revenue_account_id` (`credit_notes.rs:365-367`, repli D-B2) : seule la créance cesse
   d'être lue dans les réglages.
6. **AC6 — Tous les comptes que l'avoir écrit sont verrouillés en partage avant l'exercice, et un
   compte archivé depuis est refusé en le nommant** (choix C-15-6-16, qui révise la voie (b) de
   C-15-6-7 ; mode, place et refus révisés par C-15-6-24 et C-15-6-25 ; périmètre étendu à tous les
   comptes écrits et ligne absente tranchés par C-15-6-29 ; ordre, cycles et rejeu — porté par la
   15-5e — révisés par C-15-6-32).
   - **Verrou.** Juste après la lecture des réglages (étape (3)) et les deux lecteurs (AC3, AC4),
     **avant** l'exercice (étape (4), `fiscal_years::find_open_covering_date`, qui pose un
     `FOR UPDATE`, `fiscal_years.rs:548`) et la séquence (étape (5),
     `credit_note_number_sequences::next_number_for`, `FOR UPDATE`), `create_credit_note` verrouille
     les lignes `accounts` de **tous les comptes que l'écriture de l'avoir visera** — tous connus à ce
     point, le snapshot des lignes (étape (2)) précédant les réglages :
     - la **créance** de la vente (AC3) ;
     - le compte d'**arrondi de la vente**, si l'arrondi ≠ 0 (AC4) ;
     - le compte de **TVA due** des réglages, si l'avoir émet au moins une ligne de TVA et si le réglage
       est posé — vide, il n'est pas verrouillé et le refus existant de la génération reste à sa place
       (ci-dessous) ;
     - les **comptes de produit effectifs** : ceux des lignes (`revenue_account_id`) et, si une ligne
       n'en porte pas, le produit de repli (`default_revenue_account_id`, D-B2) — exactement les ids
       que la 6 ter calcule aujourd'hui (`credit_notes.rs:435-446`).

     En **une** instruction, ids dédoublonnés, `ORDER BY id` — **aucun réordonnancement** pour
     fermer un cycle (choix C-15-6-32, sur C54 : la défense contre l'interblocage est le rejeu de la
     15-5e, pas l'ordre des verrous) :
     `SELECT id, number, active, postable FROM accounts WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`
     — **sans** filtre `active` : la ligne doit être prise même archivée, pour être nommée.
     `FOR SHARE` est une erreur de syntaxe sur MariaDB 10.11 ; **syntaxe** (pas patron : aucune de ces
     requêtes ne vise une liste de comptes) relevée à `crates/kesh-db/src/repositories/opening_complement.rs:502`
     et `:540` (mise en garde `FOR SHARE` à `:38-39`) — la 15-6c, qui pose `claim_accounts_in_share_mode`,
     passe **après** cette fiche (fiche d'index).
   - **Un seul helper de verrou d'une liste de comptes — un seul contrat** (DRY, finding F5-5 de la
     P5 ; contrat unifié par F6-1 = R6-1 de la P6 ; C-15-6-32, C-15-6-35). La 15-5d pose, pour la même
     forme en exclusif, `lock_designated_accounts_in_tx` (`WHERE company_id = ? AND id IN (…) ORDER BY
     id FOR UPDATE`, rend `id, number, active, postable`, sans refuser — 15-5d AC1, accesseur de
     `company_invoice_settings.rs`). Il n'y a **qu'une** fonction de verrou d'une liste de comptes, et
     **ce paragraphe est son seul contrat** dans cette fiche :
     - **paramétrée par le mode** : p. ex. `lock_accounts_in_tx(conn, company_id, ids: &[i64], mode:
       AccountLockMode)`, `AccountLockMode::{Exclusive, Share}` → `FOR UPDATE` / `LOCK IN SHARE MODE`
       (signature indicative) ;
     - **emplacement** : celui que la 15-5d lui a donné (`company_invoice_settings.rs`, T0 le
       constate) ; la 15-6a ne la déplace pas ;
     - **colonnes rendues** : `id, number, active, postable`, **toutes** les lignes trouvées, archivées
       comprises, ordre `ORDER BY id` ;
     - **elle ne refuse rien** : ni compte archivé, ni ligne absente. Le contrôle « ligne absente »
       (nombre de lignes rendues ≠ nombre d'ids dédoublonnés → `Invariant`) et le partage des lignes
       (créance, arrondi, TVA due → la variante ; produits → la 6 ter) vivent **chez l'appelant**,
       `create_credit_note` ;
     - **visibilité `pub`** : le test 17 (`crates/kesh-db/tests/credit_notes_repository.rs`, test
       d'**intégration**, hors de la crate) l'appelle directement — `pub(crate)` ne compilerait pas ;
     - **type de ligne** : un type **nommé, public**, distinct de `opening_complement::LockedAccount`
       (`opening_complement.rs:383`, **privé**, colonnes `account_type` et `role` en plus) — p. ex.
       `LockedAccountState { id, number, active, postable }` ; `LockedAccount` n'est **ni réutilisé**
       (forme différente, privé à un flux que l'epic ne touche pas) **ni repris comme nom** (deux types
       homonymes dans la même crate tromperaient la lecture). **Un seul type, jamais deux** : si la 15-5d a
       déjà nommé **et exposé (`pub`)** le type de ligne de son helper, la 15-6a l'emploie ; si elle l'a
       laissé privé ou anonyme, la 15-6a le rend public sous le nom qu'il porte (ou `LockedAccountState`
       s'il n'en a pas) et ne crée pas de second type (finding R7-1 de la P7 ; C-15-6-36).
     **Qui la crée** : la **15-5d**, mergée avant (dépendance **ferme**, en-tête ; finding R6-10 de la
     P6). Si T0 la trouve sans paramètre de mode, la 15-6a l'y ajoute (l'appel de la 15-5d passe
     `Exclusive`, son comportement ne change pas) au lieu d'écrire une jumelle.
     **Critère DRY, relevé au Dev Agent Record** (finding R6-2 de la P6) : `grep -rnE "LOCK IN SHARE
     MODE|FOR UPDATE" crates/kesh-db/src` lu site par site (requêtes multi-lignes et `QueryBuilder`
     compris) ne montre aucune autre requête qui verrouille une **liste** d'ids de comptes, **hors une
     exception écrite** : `opening_complement.rs:522-532` (`create_opening_complement`, `SELECT id,
     number, account_type, active, postable, role … WHERE id IN (…) AND company_id = ? ORDER BY id FOR
     UPDATE`), dont la forme diffère (deux colonnes de plus, pré-filtre `owned_account_ids` **hors**
     transaction) et qui sort du périmètre de l'epic — la refondre sur le helper serait un changement
     sans motif d'un flux livré. Toute **autre** occurrence est un site non résolu.
   - **Plan et identifiant étranger** (finding F5-4 de la P5, sur le patron de la 15-5d, choix C51) :
     sous REPEATABLE READ, un balayage ou un `filesort` qui parcourt d'autres lignes les **verrouille**
     (piège mesuré à `opening_complement.rs:274-288`), et un verrou par clé primaire peut prendre une
     ligne **avant** que le filtre `company_id` ne l'écarte (`opening_complement.rs:431-440`) — en
     partage comme en exclusif, un S bloque l'`UPDATE` d'un archivage de ce compte. L'`EXPLAIN` de la
     requête en mode `Share` (accès par clé primaire sur `id IN (…)`, aucun `filesort`) est **relevé au
     Dev Agent Record** ; un test d'identifiant d'une **autre société** est ajouté (§ *Tests*, 17) ;
     s'il montre la ligne étrangère verrouillée, le helper prend le patron `owned_account_ids`
     d'`opening_complement.rs` — pour les deux modes, donc aussi pour la 15-5d — et le Dev Agent
     Record le dit.
   - **« L'avoir émet de la TVA » : une seule source** (findings R5-4 et F5-2 de la P5 ; C-15-6-32).
     La condition vit aujourd'hui **dans** `generate_credit_note_journal_lines` (`total_vat > 0`, somme
     des TVA agrégées par taux, `credit_notes.rs:201-214`, `:240`), appelée à l'étape (7), **après**
     le verrou. Un `any(vat_rate > 0)` naïf en divergerait (ligne à taux positif dont la TVA s'arrondit
     à zéro). T3 **extrait** l'agrégation en une fonction pure (p. ex.
     `credit_note_vat_by_rate(lines) -> BTreeMap<Decimal, Decimal>`, à côté du générateur, sans I/O),
     appelée **par le générateur** et **par le calcul des ids à verrouiller** — aucune copie du calcul.
     **Place du refus `ConfigurationRequired("default_vat_payable_account_id")` : inchangée** — il reste
     rendu par le générateur, à l'étape (7), donc **après** `FISCAL_YEAR_INVALID` et après
     `ACCOUNT_ARCHIVED` (un réglage vide n'est pas verrouillé, rien ne le rend plus tôt). Avancer
     l'appel du générateur avant le verrou l'aurait fait passer devant l'exercice : changement de
     priorité sans motif, écarté (C-15-6-32).
   - **Pourquoi un verrou partagé (S), pas exclusif (X).** Le verrou a un but premier : qu'un
     archivage concurrent (`accounts::archive`, `accounts.rs:604-634`, un `UPDATE … SET active = FALSE`
     qui demande X et ne regarde pas les écritures) **attende la fin de l'avoir** au lieu de le
     précéder. Un S y suffit, et il est compatible avec les S que tout insert de ligne d'écriture pose
     déjà sur le compte parent (contrôle de clé étrangère des `INSERT INTO journal_entry_lines`). Un S
     **n'ajoute aucun cycle** par la créance : `accept_one_invoice` y tient un S (clé étrangère des
     lignes du règlement, jusqu'au commit du lot, `tx_outer`, `reconciliation.rs:887`), et S contre S
     ne bloque pas — un X y aurait ajouté un cycle. Mais un S **n'en retire aucun** non plus : face au
     **X** qu'un autre flux prend sur le même compte (compte d'arrondi du rapprochement, du règlement
     ou du solde du reste ; compte de nature et TVA due du solde du reste), il **attend**. Les cycles
     qui subsistent sont écrits plus bas. Corollaire : deux avoirs simultanés **ne se sérialisent pas**
     sur la ligne d'un compte (S + S) — ils se sérialisent déjà, et seulement, sur la ligne des réglages
     (étape (3), `get_or_create_default_in_tx`, `FOR UPDATE`, `company_invoice_settings.rs:109`). La
     lecture sous S est une lecture **verrouillante** : elle rend l'état **courant** de `active`, comme
     le ferait un X.
   - **Ordre des verrous** : facture (1) → règlements (garde 25-4-a) → avoir existant → réglages (3)
     → **comptes de l'avoir (S, `ORDER BY id`)** → exercice (4) → séquence (5) → écriture, dont l'insertion des lignes **reprend** un S sur chaque
     compte écrit (clé étrangère `fk_jel_account`). La règle de l'epic est celle que la **15-5e** écrit
     au doc-comment canonique de `validate_invoice` (son AC5, choix **C54**) : l'ordre des verrous est
     une **convention qui réduit la fréquence** des interblocages — il ne peut pas les exclure, puisque
     l'insertion des lignes reprend `accounts` **après** l'exercice — et **la défense est le rejeu des
     routes**. Cette fiche **suit** cette règle et ne la réécrit pas ; elle ne prétend à l'absence
     d'**aucun** cycle (C54 : « elles cessent d'affirmer l'absence de cycle et renvoient au rejeu de la
     15-5e »).
     **Pourquoi ce verrou, et pourquoi avant l'exercice** : il ferme une **course de lecture** — un
     archivage concurrent attend la fin de l'avoir au lieu de le précéder, et l'état `active` de
     chaque compte écrit (créance, arrondi, TVA due, produits — la 6 ter comprise) est lu **frais**,
     sous verrou, non dans l'instantané (C-15-6-29). C'est son seul rôle revendiqué. Sa place avant
     l'exercice est celle que la fiche avait depuis la P3 (C-15-6-24) et la convention que décrit le
     doc-comment canonique (comptes → exercice) ; elle n'est **pas** un instrument de fermeture de
     cycle, et la fiche n'en tire aucune garantie. La ligne des réglages `FOR UPDATE`
     (`company_invoice_settings.rs:109`), prise d'abord par l'avoir, la validation et la saisie
     fournisseur (15-5e AC5), les sérialise entre eux — fait relevé, non promesse. Le doc-comment de
     `create_credit_note` porte cet ordre et **renvoie** au doc-comment canonique pour la règle et au
     rejeu de la route (AC7, DRY).
   - **Cycles connus, nommés — non exhaustifs, couverts par le rejeu.** Les passes de validation en ont
     trouvé quatre ; d'autres peuvent exister, et la fiche ne prétend pas le contraire (C54). Tous sont
     détectés par InnoDB (1213) ; aucun ne naît de la créance :
     (i) **avoir ↔ lot de rapprochement** (**#536**, fermée par le rejeu de la 15-5e, C57) :
     `accept_one_invoice`, qui ne verrouille pas la facture en amont (`reconciliation.rs:1688-1691`),
     prend X sur le compte d'arrondi du moment si le paiement solde avec écart (`:1487`,
     `rounding_account_for_write` → `usable_designated_account`, `company_invoice_settings.rs:499`) et
     X sur l'exercice (`:1520`, `fiscal_years.rs:548`), **avant** son `UPDATE invoices` (`:1703`), et le
     lot garde l'exercice d'une proposition à la suivante (savepoints d'une seule transaction,
     `tx_outer`, `:887`) ; l'avoir tient X sur la facture et S sur l'arrondi de la vente, puis demande
     l'exercice. Préexistant (l'avoir prenait déjà l'exercice) ; le lot rejoue (`post_accept`), l'avoir
     aussi (15-5e).
     (ii) **avoir ↔ solde du reste d'une autre facture**, quand le compte de nature de celui-ci est
     aussi un compte de produit de l'avoir (p. ex. une association qui a désigné elle-même son 3800
     *Autres produits* comme compte d'escompte — le plan des associations n'en désigne aucun d'office,
     `admin-manual.tex:2029` —, ou un 3800 *Déductions sur ventes* de PME imputé sur une ligne de
     facture ; finding F6-3 de la P6) : le solde prend X sur la nature, puis l'arrondi, puis la TVA due ;
     l'avoir prend ses S par `id` (2200 avant 3800) — ordres opposés entre la nature et la TVA due.
     Rare. **Variante fréquente** (finding R5-2 de la P5) : sans nature commune, le solde d'un reste
     hors centime prend X sur l'arrondi puis sur la TVA due ; l'avoir d'une facture arrondie, par `id`,
     la TVA due (2200) avant l'arrondi (6940, plus loin dans le plan livré, ou créé après lui ; finding
     R6-8 de la P6) — ordres opposés entre deux comptes
     courants. **Non réordonné** (C-15-6-32, C54) : couvert par le rejeu des deux routes.
     (iii) **avoir ↔ règlement client par compte interne** (finding F5-3 de la P5), quand le compte
     interne choisi est un compte que l'avoir écrit (sa TVA due, un de ses produits) et que le paiement
     a un écart : le règlement prend X sur le compte interne puis sur l'arrondi ; l'avoir les prend
     par `id`. Configuration sans sens comptable, rare.
     (iv) **cycle à trois par un X en attente** : l'avoir tient S sur un compte et attend l'exercice,
     tenu par un flux qui prend l'exercice **sans passer par la ligne des réglages** — saisie manuelle
     (`journal_entries::create_in_tx`, exercice `journal_entries.rs:279-280`, puis l'insertion des
     lignes), règlement client (`invoice_settlements_write.rs:200`), solde du reste (`:493`),
     rapprochement (`reconciliation.rs:1520`, manuel et ventilé `:2083`, `:2432`) ; un `UPDATE
     accounts` sur ce compte (archivage, modification, création d'un sous-compte qui rend le parent non
     imputable) attend derrière le S ; ce flux insère une ligne sur ce compte, et sa demande S attend le
     X en attente (file InnoDB) → 1213. **Neuf** (sans le S, l'`UPDATE` passait), rare. La
     **validation** et la **saisie fournisseur** n'en sont **pas** : elles prennent la ligne des réglages
     `FOR UPDATE` **avant** l'exercice — établi pour la **validation** (`invoices.rs:2006`, puis
     `:2172`) ; pour la saisie fournisseur (`supplier_invoices.rs:359` pour les réglages), T0 **relève**
     l'ordre par rapport à son exercice —, et l'avoir la prend avant ses
     S (étape (3)) — une validation qui tient l'exercice tient donc les réglages, et l'avoir l'attend
     là, sans encore tenir de S (finding F6-2 = R6-4 de la P6).
     **Le rejeu** : la route `POST /api/v1/credit-notes` (`crates/kesh-api/src/routes/credit_notes.rs:183`)
     est **rejouée par la 15-5e** (son AC3 : enveloppe `DbError`, `kesh_db::retry::retry_on_deadlock`,
     doc-comment d'une ligne ; registre des routes, son test 5). La 15-6a **n'en pose pas** un second
     (C-15-6-32, qui retire ce point de C-15-6-29) ; T0 le **constate** sur `HEAD` — si la 15-5e n'est
     pas mergée, la story attend (ordre de l'epic). Le rejeu est sûr : la transaction est atomique et le
     numéro d'avoir se rembobine au rollback ; un 1213 ressort en `DbError::Sqlx` (`map_db_error`,
     `kesh-db/src/errors.rs:796`, ne le convertit pas). Le test 15 fige que ce rejeu couvre l'attente
     **neuve** que l'AC6 crée (le S sur les comptes de l'avoir).
   - **Ordre des refus qui en découle** : un compte de créance, d'arrondi ou de TVA due archivé (ce
     refus) l'emporte sur `FISCAL_YEAR_INVALID` (4), sur `creditNoteTotalZero` (6 bis) et sur
     `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` (6 ter) ; il vient **après** les refus de l'étape (1)
     (facture non validée, réglée, déjà annulée), après `ConfigurationRequired("default_revenue_account_id")`
     de l'étape (3) (`credit_notes.rs:365-367`, AC5) et après les `Invariant` des deux lecteurs (AC3,
     AC4). Le test 14 fige la précédence sur l'exercice ; les refus antérieurs ne demandent pas de test
     neuf (ils précèdent déjà toute lecture de compte).
   - ⚠️ **L'état `active` se lit DANS cette lecture verrouillante**, jamais dans une lecture simple
     ultérieure : sous REPEATABLE READ, une lecture non verrouillante rend l'instantané de la
     transaction — établi dès la première lecture simple, le snapshot des lignes (étape (2)), donc
     **avant** le verrou — et ne voit pas un archivage validé entre-temps. C'est le trou de la garde
     `active` de `create_in_tx` (`journal_entries.rs:96-98`, sans verrou), qui ne suffit donc pas
     seule, et c'est pourquoi la **6 ter** (comptes de produit) ne relit plus `active` par sa lecture
     simple (`credit_notes.rs:450-451`) : elle prend l'état actif dans le résultat du verrou, et garde
     son code (`CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED`), son message par ligne et **sa place** (après
     l'exercice et la 6 bis — l'ordre des refus du produit ne change pas).
   - **Ligne absente du résultat du verrou.** La requête porte `company_id = ?` : un id que le verrou ne
     rend pas (compte d'une autre société sur une écriture ou une ligne corrompue — **inatteignable par
     l'application** : les validations à la création d'une ligne de facture, anti-IDOR,
     `invoices_line_revenue_account.rs:365-367` `create_rejects_cross_company_account`, et à la
     désignation d'un réglage l'excluent, et les lectures sont portées par `je.company_id` ; **aucune**
     clé étrangère ne porte la société — `fk_jel_account`, `fk_cis_receivable` et celle de
     `revenue_account_id` ne référencent que `accounts(id)` —, si bien qu'un `UPDATE` direct y parvient,
     comme le fait le test 17 ; finding F6-4 de la P6) → `DbError::Invariant` (« compte de l'avoir
     introuvable dans la société »), dès que le nombre de lignes rendues diffère du nombre d'ids
     dédoublonnés. Tranché ainsi (C-15-6-29) plutôt que de laisser le filet anonyme de `create_in_tx`
     (`InactiveOrInvalidAccounts`) répondre. Pour un **compte de produit** d'une autre société, cela
     remplace aussi le `CreditNoteRevenueAccountsArchived` à `account_number: None` que la 6 ter rend
     aujourd'hui (`credit_notes.rs:471-495`) : changement de comportement **inatteignable par
     l'application** (atteignable par SQL direct seulement), écrit pour mémoire — T4 n'écrit **pas** de test dessus (finding F5-7 de la P5).
   - **Refus nommé, propre à l'avoir** (choix C-15-6-25, qui révise le « aucune variante » de
     C-15-6-16 ; étendu à la TVA due par C-15-6-29). Une ligne `active = FALSE` parmi la **créance**,
     l'**arrondi** de la vente ou la **TVA due** → variante neuve
     `DbError::CreditNoteAccountsArchived(Vec<ArchivedAccount>)` (`ArchivedAccount`,
     `crates/kesh-db/src/errors.rs:291`), à côté de `ReversalAccountsArchived` (`:604-614`), dont elle
     reprend la forme : **400**, **même code `ACCOUNT_ARCHIVED`** (`error_code()`, `errors.rs:736`,
     rend la même chaîne pour les deux, `:760`), `details.rejected[]` = `{ accountId, accountNumber }`
     par compte, ordre `ORDER BY id` (celui du verrou ; sa sœur `archived_accounts_in_tx` trie par
     `number` — sans effet pour un seul compte, dit pour qu'aucun test ne fige l'un pour l'autre). Un
     **compte de produit** archivé n'alimente **pas** cette variante : il reste nommé par ligne par la
     6 ter (ci-dessus). Seul
     le **message** diffère : clé neuve `credit-note-account-archived`, dans les **quatre** locales —
     fr : « Impossible d'émettre l'avoir — compte(s) archivé(s) : 1100. Réactivez le ou les comptes
     concernés. » —, sur le patron du bras `ReversalAccountsArchived`
     (`crates/kesh-api/src/errors.rs:2945-2973`). **Raison** : sur la route
     `POST /api/v1/credit-notes`, le compte de produit archivé rend déjà « Impossible d'émettre
     l'avoir — … » (`CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED`, clé `credit-note-revenue-account-archived`,
     `fr-CH/messages.ftl:291`) : un seul vocabulaire pour un seul geste, face au bouton « Créer un
     avoir ». Le **code**, lui, reste celui de la contre-passation : l'avoir en **est** une, et un
     intégrateur qui gère `ACCOUNT_ARCHIVED` (lire `details.rejected[]`, réactiver) le gère ici sans
     rien apprendre. Deux codes restent donc possibles sur la route — l'un par **ligne** de facture
     (produit, `details.rejected[].lineNumber`), l'autre par **compte** (créance ou arrondi de la
     vente, TVA due) — et le manuel comme le CHANGELOG le disent (AC8, AC9). **La lecture est celle du
     helper partagé** (puce « Un seul helper », ci-dessus — son seul contrat), appelé en mode `Share` :
     `create_credit_note` reçoit **toutes** les lignes verrouillées (`id`, `number`, `active` ; `postable`, rendue par le helper, est ignorée), pas
     seulement les archivées, fait lui-même le contrôle « ligne absente », puis les partage — créance,
     arrondi et TVA due archivés → la variante ; produits archivés → la 6 ter, à sa place.
     `archived_accounts_in_tx` (`journal_entries.rs:1723-1754`, non verrouillante, filtre
     `active = FALSE`) n'est **pas** le modèle : elle ne sert ici qu'à la comparaison de l'ordre de
     `details.rejected[]` (ci-dessus).
     La garde `active` de `create_in_tx` reste en place, filet de défense ; elle n'est plus le refus
     atteint pour aucun de ces comptes, **hors réactivation concurrente** (finding R5-7 de la P5) : un
     compte archivé à l'instantané (étape (2)) puis réactivé et validé avant le verrou est vu actif par
     le verrou, archivé par la lecture simple de `create_in_tx` (`journal_entries.rs:96-98`) — le refus
     anonyme `InactiveOrInvalidAccounts` reste alors possible, fenêtre de quelques millisecondes, sans
     conséquence sur les données. Dit pour qu'aucun test ne prétende l'exclure.
   - ⚠️ **Changement délibéré de deux refus existants.** (i) Un avoir sur une facture arrondie dont le
     compte d'arrondi a été archivé rendait `ROUNDING_ACCOUNT_NOT_CONFIGURED` (contexte `Issuance`,
     #486), dont le message renvoie à *Paramètres → Facturation* — remède **faux**, puisque l'avoir ne
     lit plus ce réglage ; il rend désormais `ACCOUNT_ARCHIVED`, qui **nomme** le compte à réactiver
     (le refus reste nommé : l'esprit de la contrainte de #523, « sans casser le refus nommé de #486 »,
     est tenu ; sa lettre — le code — ne l'est pas, et la fiche le dit). (ii) Un avoir dont le compte
     débiteurs (des réglages, aujourd'hui) est archivé rendait `INACTIVE_OR_INVALID_ACCOUNTS`
     anonyme ; il rend `ACCOUNT_ARCHIVED` nommant le compte **de la vente**. (iii) Un avoir dont le
     compte de TVA due des réglages est archivé rendait aussi `INACTIVE_OR_INVALID_ACCOUNTS` anonyme
     (garde de `create_in_tx`) ; il rend `ACCOUNT_ARCHIVED` nommant ce compte (C-15-6-29). Le test
     `invoices_validate_vat.rs:846` (`a_credit_note_is_refused_when_the_rounding_account_was_archived`)
     **change d'assertion** en conséquence, et le Change Log de la story le dit comme tel (ce n'est pas
     une régression masquée). La **validation** d'une facture garde
     `RoundingAccountNotConfigured { Issuance }` (`invoices.rs:2068`) : elle crée l'écart, elle lit le
     réglage.
7. **AC7 — Les commentaires disent vrai.** Le commentaire `credit_notes.rs:609-611` dit désormais ce
   qui est recopié (produit par ligne) et ce qui est relu sur la vente (créance, arrondi) ; l'étape (3)
   de `create_credit_note` dit pourquoi la créance ne vient plus des réglages (#473) et quels comptes en
   viennent encore (TVA due #525, produit de repli D-B2) ; l'étape (7 bis) dit pourquoi l'arrondi se lit
   sur la vente (#523) ; l'étape du verrou (AC6) dit pourquoi `active` se lit sous verrou, pourquoi
   en partage et non en exclusif (course de lecture, pas de verrou exclusif de plus), pourquoi
   **tous** les comptes écrits (la course de lecture), et que la
   6 ter lit `active` dans son résultat. Le doc-comment de `create_credit_note` reçoit une section
   **« Ordre des locks »** : facture → règlements → avoir existant → réglages → comptes de l'avoir
   (S, par `id`) → exercice → séquence → écriture — il
   **renvoie** au doc-comment canonique de `validate_invoice` pour la règle (convention de fréquence,
   défense par le rejeu — 15-5e AC5, C54) au lieu de la recopier, ne prétend à l'absence d'aucun
   cycle, et ne porte que ce qui est propre à l'avoir : la ligne des réglages qui le sérialise avec la
   validation et la saisie fournisseur, et le renvoi au rejeu de la route (AC6). Le doc-comment de la
   route `POST /api/v1/credit-notes` (« rejouée sur interblocage ») est celui que pose la **15-5e**
   (son AC3) : pas réécrit ici.
   **Ce que la 15-5e a déjà réécrit, et que cette story ne réécrit pas** (findings R5-1 = F5-1 de la
   P5) : le doc-comment canonique de `validate_invoice` (dont la phrase « Aucun chemin ne verrouille
   `accounts` avant `invoices` ou `fiscal_years` », `invoices.rs:1921-1922` au commit `1920381e`,
   retirée par la 15-5e AC5), le commentaire « 5 bis » de `write_off_invoice`
   (`invoice_settlements_write.rs:466-468` au même commit, réécrit en place par la 15-5e AC5, que la
   15-5d complète) et le Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (15-5e AC5). T0 les
   **lit** tels que la 15-5e et la 15-5d les laissent. La 15-6a n'y fait qu'**un** geste, et seulement
   s'il y a lieu : si l'un de ces textes décrit l'**ordre de l'avoir** tel qu'il était (l'exercice
   avant l'arrondi, ou des comptes lus sans verrou), cette phrase est corrigée — l'avoir prend
   désormais tous ses comptes, en partage, avant l'exercice ; le Dev Agent Record dit ce qui a
   été trouvé (y compris « rien »). Le doc-comment de `RoundingContext::Issuance` (`crates/kesh-db/src/errors.rs:729`, « Une pièce émise —
   facture validée, avoir — ») ne cite plus l'avoir ; le message
   `error-rounding-account-not-configured-issuance` (« Le total de cette pièce… ») reste exact pour la
   facture et n'est pas modifié. Le doc-comment de `generate_invoice_journal_lines_rounded`
   (`invoices.rs:1870-1878`), qui ne protège aujourd'hui que « la créance reste la PREMIÈRE ligne au
   débit », dit aussi que **l'arrondi reste la DERNIÈRE ligne** — l'avoir l'y relit
   (`sale_rounding_account`) ; une ligne ajoutée après lui bloquerait tous les avoirs de factures
   arrondies (`Invariant`). Le commentaire de classement que la 15-5b (son AC6) pose dans
   `accept_one_invoice` sur la lecture de la créance est **conservé** et réécrit pour désigner le
   lecteur `sale_receivable_account` au lieu d'une requête et de numéros de ligne qui n'existent
   plus. Les doc-comments et commentaires que la 15-5b et la 15-5d laissent sur l'**exemption de
   l'avoir** à la garde à l'usage (C35 : « la créance sera lue sur l'écriture de vente par la
   15-6a ») sont réécrits au présent : l'avoir lit la créance sur la vente ; seule la TVA due reste
   relue dans les réglages (#525).
8. **AC8 — Manuel.**
   - `docs/manual/fr/user-manual.tex` § *Avoirs et notes de crédit* (`\label{sec:avoirs}`, puce
     `:1155`) : la promesse « le solde du client revient à zéro » devient **vraie** et le texte dit
     pourquoi — l'avoir crédite le compte débiteurs que la facture a débité, et contre-passe l'arrondi
     sur le compte que la facture a mouvementé, même si les comptes par défaut ont changé depuis. Il dit
     aussi, **sans la présenter comme juste**, la limite connue (choix C-15-6-8) : la TVA de l'avoir est
     contre-passée sur le compte de TVA due **actuellement désigné** dans les réglages ; changer ce
     réglage entre une facture et son avoir est une limite connue de cette version.
   - Même section, un `keshwarning` neuf à côté de « Avoir refusé : compte de produit archivé »
     (`:1171`) : **« Avoir refusé : compte débiteurs, d'arrondi ou de TVA archivé »** — il nomme
     les trois comptes (celui que la facture a débité, celui qui a reçu son arrondi, le compte de TVA
     due désigné dans les réglages — C-15-6-29), cite le message tel
     quel (« Impossible d'émettre l'avoir — compte(s) archivé(s) : … », AC6) et renvoie à la **même
     procédure** en trois étapes : réactiver le compte (en retirant d'abord le rôle *Créances clients* du nouveau compte si nécessaire :
     la réactivation est refusée tant qu'un autre compte le porte, `user-manual.tex:358-359`), émettre
     l'avoir, ré-archiver. Pour le compte
     de **TVA due**, qui est celui **des réglages** et non celui de la vente (#525), il dit l'**autre
     issue** (finding F5-7 de la P5) : rétablir le réglage — un administrateur désigne dans
     *Paramètres → Facturation* un compte de TVA due actif, de préférence celui que la facture a
     crédité —, sans quoi la procédure ferait réactiver un compte que la facture n'a jamais mouvementé.
     Il dit que ce
     refus est distinct de celui du compte de produit (qui nomme une **ligne**) : le premier nomme un
     **compte**, et les deux peuvent se suivre (le compte débiteurs, d'arrondi ou de TVA est contrôlé
     d'abord, AC6).
   - `user-manual.tex:363` (avertissement « Avant d'archiver un compte de produit, pensez aux
     avoirs », `\label{sec:plan-comptable-archivage}`) : étendu au **compte débiteurs**, au **compte
     de différences d'arrondi** et au **compte de TVA due** — un compte que des factures validées ont
     mouvementé reste nécessaire tant qu'un avoir peut les viser.
   - `docs/manual/fr/admin-manual.tex:2027` (*Compte de différences d'arrondi*, « un avoir le reprend
     et l'annule ») : une phrase — changer de compte d'arrondi ne déplace pas les factures déjà
     émises : leur avoir contre-passe l'arrondi sur le compte qu'elles ont mouvementé ; gardez l'ancien
     actif tant qu'un avoir peut les viser (sinon : réactiver, émettre, ré-archiver).
   - Les phrases « un avoir le reprend et l'annule » (`user-manual.tex:847`, `admin-manual.tex:2027`)
     restent vraies — les relire, ne pas les réécrire ; de même `user-manual.tex:1754` (« changement du
     compte clients dans les réglages », balance âgée).
   - **Frontière 15-5b / 15-5d — lire, pas greper une formulation.** La 15-5b (son AC17 (iv)) et la
     15-5d (son AC8) écrivent au manuel que l'avoir **relit la créance et la TVA due dans les
     réglages** (« l'avoir fait exception … »), aux encadrés de `user-manual.tex` ~`:380` (§ *Rôles des
     comptes*) et ~`:390` (§ comptes de clôture), et la 15-5b touche `admin-manual.tex:2016-2029`
     (comptes par défaut, arrondi). Ce texte n'est **pas encore mergé** : sa formulation exacte est
     inconnue, et un grep de la formulation des **fiches** (« relit la créance ») peut rendre zéro et
     laisser la phrase fausse. **T0 lit donc ces encadrés et cette section en entier, après le merge
     des 15-5**, aux numéros de `HEAD` ; puis grepe la **valeur** — `avoir` à moins de 200 caractères
     de `créance`, `réglages`, `relu` — sur le `.tex` et sur les PDF aplatis
     (`pdftotext … - | tr '\n' ' ' | tr -s ' '`). Toute phrase qui dit que l'avoir relit la
     **créance** dans les réglages devient fausse : la corriger ici ; la TVA due reste vraie (#525).
   - PDF régénérés (`make fr` dans `docs/manual/`) et contrôlés **aplatis**, sur des fragments
     **absents du PDF actuel** (« revient à zéro » y est déjà : il ne prouve pas la régénération), et
     avec `.` à la place de chaque apostrophe dans le motif (le PDF emploie l'apostrophe typographique) :
     `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "<fragment de la phrase neuve sur la créance>[^.]*"`,
     puis le même `grep -o` sur un fragment de la phrase de la TVA, du `keshwarning` neuf, de
     l'avertissement étendu, et sur `admin-manual.pdf` pour la phrase neuve de l'arrondi.
9. **AC9 — CHANGELOG.** Sous `## [0.13.0] — Non publié`, section `### Corrigé` : une entrée qui dit
   l'effet pour l'utilisateur et renvoie à [#473](https://github.com/guycorbaz/kesh/issues/473) et
   [#523](https://github.com/guycorbaz/kesh/issues/523) ; section `### Modifié` : le refus d'un avoir dont
   le compte débiteurs ou d'arrondi de la facture, ou le compte de TVA due, a été archivé devient
   `ACCOUNT_ARCHIVED` (il était `INACTIVE_OR_INVALID_ACCOUNTS`, ou `ROUNDING_ACCOUNT_NOT_CONFIGURED`
   pour l'arrondi), nomme le
   compte dans `details.rejected[]` et dit « Impossible d'émettre l'avoir » ; la route garde
   `CREDIT_NOTE_REVENUE_ACCOUNT_ARCHIVED` pour un compte de produit de ligne (deux codes, un même
   libellé de geste — AC6) ; un réglage débiteurs vide ne refuse plus l'avoir (`CONFIGURATION_REQUIRED`) — changements
   visibles d'une intégration par clé d'API (`POST /api/v1/credit-notes` est ouverte aux clés,
   `docs/api-external.md` § 7), rubrique choisie comme la 15-5a. `docs/api-external.md` ne cite ni ces
   codes pour l'avoir ni la route (`grep -n "ROUNDING_ACCOUNT\|credit-notes" docs/api-external.md` vide
   au 2026-10-08) : **sans objet**, à écrire « vérifié » au Dev Agent Record.
   **Propriétaire de la section** : la **première
   story de la version mergée** la crée en tête des versions si elle est absente ; les suivantes y
   ajoutent leur entrée (règle commune aux 15-5a/b/c et 15-6a/b/c/d ; conflit de rebase attendu et
   trivial).
   **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, Pattern 5** (finding F5-6 de la P5) : le tableau
   *Where This Applies* (`:307-319` au commit `1920381e`), dans la forme que la **15-5e** lui donne
   (son AC5 : « Global Lock Order » devenu convention de fréquence, deux lignes de flux comptables
   corrigées, « Deny list » retirée), gagne **une** ligne pour `POST /api/v1/credit-notes`
   (`repositories/credit_notes.rs::create_credit_note`) — invoices → invoice_settlements →
   credit_notes → company_invoice_settings → accounts (S, par `id`) → fiscal_years → credit_note_number_sequences → journal_entries (lignes : S sur les comptes
   par la clé étrangère) ; rejouée sur interblocage (15-5e). T0 vérifie que la 15-5e ne l'a pas déjà
   ajoutée ; si oui, la ligne est seulement mise à jour (comptes verrouillés avant l'exercice).

## Tasks / Subtasks

- [x] **T0 — Rebase et recompte** *(fait le 2026-10-08 sur `9cb5083b` — Change Log « Alignement sur le livré (T0) » ; la story **attend le merge de la 15-5d**, C-15-6a-1)* avant de coder : rebase sur `main` après le merge des 15-5a à 15-5e ;
  **refaire tous les numéros de ligne** cités par cette fiche sur `HEAD` ; **lire** le doc-comment
  canonique de `validate_invoice`, le commentaire « 5 bis » et le Pattern 5 tels que la 15-5e (et la
  15-5d) les laissent (AC7, AC9) ; **constater** que la route `POST /api/v1/credit-notes` est rejouée
  par la 15-5e (AC6) ; relever au Dev Agent Record les ordres de verrous de la validation, du règlement,
  du solde du reste et de la saisie fournisseur (AC6) ; constater la forme du helper de verrou de
  liste de la 15-5d (mode paramétré ou non, AC6) ; relire le
  commentaire de classement posé par la 15-5b dans `accept_one_invoice` (AC7) ; **lire en entier** les
  encadrés `user-manual.tex` ~`:380` et ~`:390` et `admin-manual.tex:2016-2029` tels que les 15-5b/5d
  les laissent (AC8 — pas de grep d'une formulation de fiche) ; **recenser les tests que la 15-5d
  ajoute à `credit_notes_repository.rs`** (dont celui de C35, § *Tests*, 13) et le nombre total de
  tests du fichier ; **relever** au Dev Agent Record l'ordre des verrous que la 15-5d pose dans
  `validate_invoice` (comptes de réglage avant l'exercice) — un relevé, sans exigence de conformité :
  sous C54 l'ordre est une convention de fréquence, non une règle à laquelle l'AC6 se conformerait
  (finding R6-3 de la P6) ; recompter l'inventaire (§ *Inventaire*) par ses commandes — tout site
  neuf est résolu ici ou ajouté aux angles morts avec sa raison.
- [ ] **T1 — Lecteurs** (AC1, AC4) dans `invoice_settlements.rs`, doc-comments compris.
- [ ] **T2 — Remplacer les trois copies** (AC2) ; vérifier par le `grep --include='*.rs'` de l'AC2.
- [ ] **T3 — Avoir** (AC3, AC4, AC5, AC6, AC7) dans `credit_notes.rs` : verrou en une instruction
  `ORDER BY id`, par le helper de verrou de liste partagé avec la 15-5d en mode
  `Share` (AC6 : tous les comptes écrits, 6 ter lue dans son résultat, ligne absente → `Invariant` ;
  `EXPLAIN` relevé) ; agrégation de la TVA extraite en fonction pure partagée par le générateur et le
  calcul des ids (AC6) ; **pas** de rejeu à poser (la 15-5e le porte, AC6) ; phrase corrigée dans un
  texte de la 15-5e seulement s'il décrit l'ancien ordre de l'avoir (AC7) ; variante
  `DbError::CreditNoteAccountsArchived`, son `error_code()` (`ACCOUNT_ARCHIVED`), son bras dans
  `crates/kesh-api/src/errors.rs` et la clé `credit-note-account-archived` dans les **quatre**
  `messages.ftl` (AC6) ; doc-comments de `RoundingContext::Issuance`, de
  `generate_invoice_journal_lines_rounded` et de `create_credit_note` (ordre des locks).
- [ ] **T4 — Tests** (§ *Tests*) — chaque test neuf **rougit d'abord** sur le code non corrigé, sauf
  ceux des lecteurs (4, 5, 8 : fonctions neuves), le témoin 10, écrit en premier, et les tests 17
  (identifiant étranger : une mesure, dont l'issue attendue est le rouge puis `owned_account_ids`,
  sauf si la 15-5d l'a déjà adopté — § *Tests*, 17) et 18 (bras d'une variante neuve). Aucun test de
  « non-interblocage » (C54, 15-5e AC4).
- [ ] **T5 — Manuel + PDF** (AC8), **CHANGELOG** (AC9) ; `docs/api-external.md` vérifié sans objet
  (AC9) ; ligne de l'avoir au Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (AC9).
- [ ] **T6 — Gates** : gate complet backend (règle `kesh-db` : un dépôt est touché, le ciblage est
  interdit **même en boucle de revue**), base remise à zéro avant ; frontend non touché ; E2E complet
  au dernier commit de code (décision D7 — la spec `frontend/tests/e2e/credit-notes.spec.ts` passe par
  cet avoir).

## Inventaire — les sites qui lisent un compte de créance, de dette ou d'arrondi **après** la pièce

Règle « Inventorier les sites NON RÉSOLUS » : l'ensemble clos est celui des **écrivains d'écritures**,
pas une liste de formes. Commandes de recompte (à rejouer en T0) :

```sh
grep -rn "journal_entries::create_in_tx\|reverse_in_tx\|reverse_owned_in_tx" crates/kesh-db/src crates/kesh-api/src
grep -rn "INSERT INTO journal_entry_lines" crates/*/src
grep -rn "default_receivable_account_id\|default_payable_account_id\|rounding_account_for_write" crates/*/src
grep -n "create_in_tx(" crates/kesh-db/src/repositories/journal_entries.rs   # appels intra-module (create_opening_entry)
grep -rn "delete_in_tx" crates/kesh-db/src crates/kesh-api/src              # dévalidation
```

Relevé au 2026-10-08 (commit `1920381e`) :

| écrivain | geste | compte relu | verdict |
|---|---|---|---|
| `invoices.rs:2005-2010` `validate_invoice` | **crée** la créance et l'arrondi | réglages | légitime — c'est la vente |
| `credit_notes.rs:360-364` `create_credit_note` (créance) | après la vente | réglages | **résolu ici (AC3)** — #473 |
| `credit_notes.rs:525` `create_credit_note` (arrondi) | après la vente | réglages | **résolu ici (AC4)** — #523 |
| `invoice_settlements_write.rs:104` `settle_invoice` | après | écriture de vente | déjà juste ; copie → lecteur (AC2) |
| `invoice_settlements_write.rs:434` `write_off_invoice` | après | écriture de vente | déjà juste ; copie → lecteur (AC2) |
| `reconciliation.rs:1425` `accept_one_invoice` | après | écriture de vente | déjà juste ; copie → lecteur (AC2) |
| `reconciliation.rs:1487` `accept_one_invoice` (arrondi `Payment`) | après | réglages | légitime — écart **nouveau**, né au rapprochement (cf. ci-dessous) |
| `supplier_invoices.rs:358-362` `create_in_tx` | **crée** la dette | réglages | légitime — c'est l'achat (import automatique compris, même chemin) |
| `supplier_invoices.rs:582-593` `pay_in_tx` | après | écriture d'achat (ligne de crédit) | déjà juste |
| `journal_entries.rs` `reverse_in_tx` (annulation de facture fournisseur, annulation de règlement, contre-passation) | après | **inversion ligne à ligne** de l'écriture d'origine | juste par construction |
| `invoices.rs` `unvalidate` (dévalidation) — `journal_entries::delete_in_tx` | supprime l'écriture de vente | aucun compte relu | hors classe |
| `journal_entries.rs` `create_opening_entry` (appel intra-module à `create_in_tx`) | — | soldes saisis | hors classe |
| `reconciliation.rs` `accept_one_split`, `accept_one_rule`, `post_manual` | — | aucun compte de créance lu | hors classe |
| `opening_complement.rs:641`, saisie manuelle | — | lignes saisies | hors classe |

Le compte d'arrondi du **règlement** (`settle_invoice`, contexte `Payment`), celui du **rapprochement**
(`accept_one_invoice`, `reconciliation.rs:1487`) et les comptes des natures de solde
(`write_off_invoice`) sont des écarts **nouveaux**, nés au règlement : ils lisent légitimement les
réglages du moment (ils ne contre-passent rien).

**Angles morts assumés** (même classe — « un compte relu dans les réglages après la pièce ») :

- **TVA due de l'avoir** (`credit_notes.rs:511`, `settings.default_vat_payable_account_id`) : compte
  **courant**, convention partagée avec le solde du reste (`vat_payable_account_for_write`). Écriture
  fausse si le réglage a changé entre la facture et l'avoir : **tracé par #525** (P1, jalon de la TVA,
  report assumé — choix C-15-6-8). Le manuel dit la limite sans la présenter comme juste (AC8).
- **Produit de repli** des lignes sans compte (D-B2) : inchangé, documenté depuis la 16-1a-bis.
  **Tracé par #525**, depuis le commentaire de l'orchestrateur du 2026-10-08 (« Même famille … compte
  de produit de repli … À traiter avec la TVA due de cette issue ») — la P5 l'avait écrit « non
  tracé », vrai à l'écriture, faux trente-huit secondes après (finding R6-2 de la P6 de la 15-6b).

**Classe « un compte lu sans verrou pendant l'avoir »** — fermée par l'AC6 (C-15-6-29) : tous les
comptes que l'avoir écrit (créance, arrondi, TVA due, produits) sont verrouillés en partage avant
l'exercice, et leur état actif est lu dans ce verrou (la 6 ter comprise). L'angle mort que la P3
inscrivait ici (produits et TVA due en lecture simple) **disparaît** ; la dette « LOW acceptée » de la
validation (`invoices.rs` ~`:2096`) n'est pas touchée.

**Sites non résolus d'une autre classe — cycles d'interblocage de l'avoir** (AC6 ; **nommés, non
exhaustifs** — l'absence d'autres cycles n'est pas affirmée, choix C54 ; tous détectés par InnoDB et
couverts par le **rejeu de la route**, posé par la 15-5e et figé pour l'avoir par le test 15) :

| cycle | avec | né de | verdict |
|---|---|---|---|
| (i) facture X / exercice X ↔ arrondi | `accept_one_invoice` de la **même** facture (`reconciliation.rs:1487`, `:1520`, avant `:1703`), ou lot dont une proposition précédente tient l'exercice | préexistant (l'exercice) | **#536**, fermée par le rejeu des deux côtés (15-5e, C57) |
| (ii) S par `id` ↔ X nature / arrondi puis TVA due | `write_off_invoice` d'une autre facture : nature = produit de l'avoir (rare), ou reste hors centime et facture de l'avoir arrondie (fréquent, P5 R5-2) | préexistant (ordre après l'exercice), déplacé avant l'exercice | non résolu, non réordonné (C54) ; rejeu des deux côtés |
| (iii) S par `id` ↔ X compte interne puis arrondi | `settle_invoice` par compte interne = un compte de l'avoir, avec écart (P5, F5-3) | le S de l'AC6 avant l'exercice | non résolu, configuration sans sens ; rejeu des deux côtés |
| (iv) S tenu ↔ X en attente ↔ S de clé étrangère | `UPDATE accounts` + un flux qui tient l'exercice **sans** la ligne des réglages (saisie manuelle, règlement, solde du reste, rapprochement) — **pas** la validation ni la saisie fournisseur, sérialisées avec l'avoir par les réglages | **neuf** (le S de l'AC6) | non résolu, rare ; rejeu côté avoir et côté de ce flux (15-5e) |

## Tests

Tous en `#[sqlx::test(migrations = …)]` sur le squash — `"./test-schema"` dans `crates/kesh-db/tests`,
`"../kesh-db/test-schema"` dans `crates/kesh-api/tests` (tests 10, 12, 15 ; finding R6-6 de la P6) —,
y compris ceux des lecteurs, qui lisent une écriture réelle, **sauf le 18**, test unitaire du bras
d'erreur HTTP (sans base).

**`crates/kesh-db/tests/credit_notes_repository.rs`** (neuf tests au commit `1920381e`, lignes
d'attribut `#[sqlx::test]` : `:101`, `:183`, `:227`, `:268`, `:309`, `:352`, `:513`, `:555`, `:593` ;
**dix** après le merge de la 15-5d, qui y ajoute le test de C35 — T0 recompte), patron `credit_note_single_rate_reverses_invoice`
(`:101`) :

1. **`credit_note_credits_the_sale_receivable_after_settings_change`** — valider une facture (créance
   sur 1100), créer un compte d'actif imputable neuf (ex. `1101`), poser
   `default_receivable_account_id` sur lui, émettre l'avoir : la ligne de crédit vise **1100** pour le
   TTC ; solde de 1100 sur la société = 0 ; **aucune** ligne sur 1101. *(Ce test rougit sur le code
   actuel — le vérifier avant le correctif, pour qu'il ne soit pas muet.)*
2. **`credit_note_needs_no_receivable_setting`** — même montage, réglage **débiteurs** mis à `NULL`
   après la validation, **par SQL direct** : l'avoir est émis (il échouait jusqu'ici en `ConfigurationRequired`). Borné à
   la créance : le réglage de produit par défaut reste posé (AC5).
3. **`credit_note_refused_when_sale_receivable_archived`** — 1100 archivé après la validation par
   `UPDATE accounts SET active = FALSE` (comme le test `:846` cité plus bas ; `accounts::archive`,
   `accounts.rs:604-634`, ne contrôle que les sous-comptes actifs et permettrait aussi d'archiver un
   compte mouvementé — le SQL direct est retenu pour ne pas dépendre de la version optimiste) :
   `DbError::CreditNoteAccountsArchived` dont l'unique `ArchivedAccount` nomme **1100**
   (`account_number = Some("1100")`), **rien** d'écrit (ni avoir, ni écriture, ni numéro consommé,
   facture toujours `validated`). *(Rougit sur le code actuel : `InactiveOrInvalidAccounts`.)*
4. **`sale_receivable_account_is_scoped_to_the_company`** — le lecteur de l'AC1, appelé avec l'écriture
   de vente d'une société et l'id d'une **seconde** société → `None`. Seconde société créée par
   `companies::create` (patron `crates/kesh-db/tests/bank_accounts_repository.rs:401`).

**`crates/kesh-db/tests/invoices_validate_vat.rs`, module `arrondi_5_centimes`** (`:592`) — ses helpers
`setup` (`:608`, désigne 6940 par `designate_rounding_account`) et `create_and_validate` (`:57`) sont
privés à ce fichier, d'où ce placement :

5. **`sale_receivable_reader_skips_a_negative_rounding_line`** — facture à arrondi **négatif**
   (234.52 → −0.02, une seconde ligne de débit après la créance) : le lecteur de l'AC1 rend 1100, pas
   le compte d'arrondi.
6. **`a_credit_note_reverses_the_sale_rounding_after_redesignation`** (#523) — facture à arrondi positif
   (123.44 → +0.01) sur 6940 ; puis un second compte de charge inséré à la main (`6941` —
   `designate_rounding_account` insère toujours `6940` et ne peut pas servir deux fois) et désigné dans
   `default_rounding_account_id` ; l'avoir : la ligne d'arrondi vise **6940**, solde de 6940 = 0,
   **aucune** ligne sur 6941. *(Rougit sur le code actuel.)*
7. **`a_credit_note_needs_no_rounding_setting`** — même montage, `default_rounding_account_id` mis à
   `NULL` après la validation, **par SQL direct** : l'avoir est émis, ligne d'arrondi sur 6940. *(Rougit sur le code
   actuel : `RoundingAccountNotConfigured`.)*
8. **`sale_rounding_reader_refuses_a_mismatched_line`** — le lecteur de l'AC4 appelé avec l'arrondi de
   **signe opposé** à celui de la facture → `DbError::Invariant`.
9. **Modifié, délibérément** (AC6) : `a_credit_note_is_refused_when_the_rounding_account_was_archived`
   (`:846`) — l'assertion `DbError::RoundingAccountNotConfigured { context: Issuance }` devient
   `DbError::CreditNoteAccountsArchived` nommant **6940** ; les assertions suivantes (rien d'écrit,
   avoir accepté après réactivation avec le **premier** numéro) restent. Le doc-comment du test dit
   pourquoi (AC6) et **réécrit sa phrase sur le compteur** (`invoices_validate_vat.rs:842-844`, « la
   transaction emporte le compteur tiré plus tôt ») : le refus précède désormais la séquence (étape
   (5)), aucun numéro n'est tiré ; l'assertion « premier numéro après réactivation » fige donc
   l'**ordre** (refus avant la séquence), plus le rollback — le doc-comment le dit, pour que le test ne
   prétende pas prouver ce qu'il ne prouve plus.

**`crates/kesh-api/tests/reconciliation_e2e.rs`** :

10. **`invoice_proposal_with_a_foreign_sale_entry_is_malformed`** — non-régression de l'AC1/AC2, écrit
    **avant** le remplacement de la copie : l'écriture de vente d'une facture repointée en SQL
    (`UPDATE invoices SET journal_entry_id = …`) sur une écriture d'une **autre société** — le lecteur,
    porté par `je.company_id`, ne trouve aucune ligne : `failed[]` porte
    `INVOICE_SALE_ENTRY_MALFORMED` avec `details = { "reason": "no_debit_line_on_sale_entry",
    "saleEntryId" }`, HTTP 200, transaction bancaire toujours en attente. Il passe avant **et** après
    le correctif — c'est un témoin, pas un test rouge d'abord. **Montage plus léger, au choix** : le
    fichier ne crée qu'une société (`reconciliation_e2e.rs:145`) ; au lieu d'en construire une seconde
    avec exercice et écriture, un **en-tête d'écriture sans ligne** dans la même société, inséré en SQL
    et désigné par `UPDATE invoices SET journal_entry_id = …`, rend le même `None`.

**`crates/kesh-db/tests/credit_notes_repository.rs`** (suite) :

11. **`credit_note_waits_for_a_concurrent_archive_of_the_sale_receivable`** — la course de l'AC6,
    sur la forme exacte du patron `credit_note_waits_for_a_concurrent_settlement` (attribut `:593`,
    attente `:631`) : facture **sans arrondi** (pour que seul le verrou de l'AC6 lise `accounts` sous
    verrou) ; une transaction tenue à la main archive 1100 (`UPDATE accounts SET active = FALSE`) sans
    valider ; une tâche (`tokio::spawn`) émet l'avoir ;
    `kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["FROM accounts", "LOCK IN SHARE MODE"], || avoir.is_finished())`
    (`crates/kesh-db/src/test_fixtures.rs:560`) doit rendre `true` — l'avoir est **vu** en attente du
    verrou ; s'il rend `false` (l'avoir a fini sans attendre), `panic!` avec son issue. **Aucun délai
    fixe** : un « pas fini au bout de 300 ms » passerait à vide sur une machine chargée. La transaction
    concurrente n'est validée qu'**après** l'attente vue ; l'avoir rend alors
    `CreditNoteAccountsArchived` nommant 1100, rien d'écrit. Le doc-comment du test dit que ses motifs
    sont couplés à la forme du verrou (`LOCK IN SHARE MODE`) et pourquoi la facture est sans arrondi.
    *(Rougit sur le code actuel — mais pas par le `panic!` « l'avoir a fini » : sur ce code, l'avoir
    n'aboutit pas, son `INSERT INTO journal_entry_lines` (clé étrangère vers `accounts`) pose un S sur
    1100 et attend le X de la transaction de test ; aucune requête `FROM accounts … LOCK IN SHARE MODE`
    n'apparaît, et c'est l'`assert!` de délai d'`attendre_une_requete_en_cours`
    (`test_fixtures.rs:585-588`, dix secondes) qui rougit — symptôme à attendre, finding R5-3 de la
    P5.)*

**`crates/kesh-api/tests/invoice_echeancier_e2e.rs`** — seul fichier de `kesh-api/tests` qui émet un
avoir **par la route** (`POST /api/v1/credit-notes`, `:742` ; les autres appellent le dépôt) :

12. **`credit_note_route_names_the_archived_sale_receivable`** — la frontière HTTP du changement de
    contrat annoncé au CHANGELOG (AC9) : facture validée, 1100 archivé en SQL, `POST
    /api/v1/credit-notes` → **400**, `error.code == "ACCOUNT_ARCHIVED"`,
    `error.details.rejected[0].accountNumber == "1100"` (et `accountId`), `error.message` contient
    « Impossible d'émettre l'avoir » et « 1100 ». *(Rougit sur le code actuel :
    `INACTIVE_OR_INVALID_ACCOUNTS`.)* Aucun Vitest n'est en cause : l'écran affiche `err.message` sans
    lire de code (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:99-112`, seul appelant de
    `createCreditNote`, `credit-notes.api.ts:33`). Preuve à rejouer en T4, **code neuf compris** :
    `grep -rn "ROUNDING_ACCOUNT_NOT_CONFIGURED\|INACTIVE_OR_INVALID_ACCOUNTS\|ACCOUNT_ARCHIVED" frontend/src`
    — `ACCOUNT_ARCHIVED` y est lu par plusieurs sites (`journal-entries/[id]/+page.svelte:166`,
    `reconciliation-cancel.ts:67`, `settlement-cancel-blocked.ts:46`, `invoice-cancel.ts:50`…),
    **aucun** sur le chemin de création d'avoir ; les deux autres codes ne rendent que
    `JournalEntryForm.svelte:178`, hors avoir.

**`crates/kesh-db/tests/credit_notes_repository.rs`** (suite) :

13. **Modifié, délibérément — le test de C35 que la 15-5d ajoute** (15-5d, § *Tests* : « avoir sur une
    facture dont la créance des réglages est devenue non imputable → émis »). Après la 15-6a, l'avoir
    ne lit plus ce réglage : le test resterait vert **pour une autre raison** et ne figerait plus rien
    (passe à vide, forme exacte de la 16-1a). Il est **ré-ancré** : la créance **de la vente** (1100)
    rendue non imputable (`postable = FALSE`) après la validation → avoir émis, crédit sur 1100 (la
    doctrine « seule l'inactivité bloque », AC4/AC6) ; son nom et son doc-comment citent C-15-6-2 et
    C-15-6-7 au lieu de l'exemption de la garde à l'usage. Le Change Log de la story dit qu'il change de
    sens. La 15-5d est une dépendance **ferme** (en-tête) : si elle n'est pas mergée au moment de T0,
    la story **attend**, comme pour la 15-5e (finding R6-10 de la P6).
14. **`credit_note_archived_sale_receivable_is_refused_before_the_fiscal_year`** — l'ordre des refus
    de l'AC6 : 1100 archivé **et** date d'avoir hors de tout exercice ouvert →
    `CreditNoteAccountsArchived` (et non `FiscalYearInvalid`) : le verrou des comptes de l'avoir
    précède bien l'exercice. *(Rougit sur le code actuel : `FiscalYearInvalid`, l'exercice étant lu
    avant tout compte.)*

**`crates/kesh-api/tests/invoice_echeancier_e2e.rs`** (suite) :

15. **`credit_note_route_replays_when_it_is_the_deadlock_victim`** — le rejeu de la route, **posé par
    la 15-5e** (son AC3), étendu à l'attente **neuve** que l'AC6 crée (le S sur les comptes de
    l'avoir) — les tests de la 15-5e font de la validation, du règlement et de son annulation la
    victime, pas l'avoir. Placé dans ce fichier ou dans `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
    que la 15-5e crée (choix écrit au Dev Agent Record) ; sur le patron
    `accept_replays_the_batch_when_it_is_the_deadlock_victim` (`reconciliation_e2e.rs:4426`) : une
    transaction de test **s'alourdit** — InnoDB choisit pour victime la transaction la plus légère —
    **exactement comme le patron** (`reconciliation_e2e.rs:4482-4496`) : la table `lest_interblocage`
    créée par DDL **hors** de la transaction, puis `seq_1_to_50` inséré dix fois dans la transaction ;
    **jamais** un lest dans une table applicative, dont les clés étrangères poseraient des verrous sur
    `accounts` et changeraient le cycle (finding R5-8 de la P5) —, puis prend `SELECT … FROM accounts WHERE id = <1100> FOR
    UPDATE` ; une tâche émet l'avoir **par la route** ; `attendre_une_requete_en_cours(&pool, &["FROM
    accounts", "LOCK IN SHARE MODE"], …)` doit rendre `true` (l'avoir tient la facture et les réglages
    et attend 1100) ; le test demande alors `SELECT id FROM invoices WHERE id = ? FOR UPDATE` et
    **doit l'obtenir** (`.expect` — sinon c'est lui la victime et le test ne prouve rien, patron de la
    15-5e) → cycle, InnoDB annule l'avoir ; le test annule sa transaction ; le rejeu repart à neuf :
    **201**, un seul avoir, facture `cancelled`, premier numéro d'avoir. *(Rougit sur le code actuel —
    aucune attente `LOCK IN SHARE MODE` n'est vue — et sur la mutation qui retire l'enveloppe de la
    route : 500, consignée au Dev Agent Record.)* Il exerce le **prédicat** (`is_deadlock_error` sur le
    `DbError` que rend le dépôt), pas seulement la présence de l'enveloppe ; son doc-comment dit
    pourquoi le montage est déterministe.

**`crates/kesh-db/tests/credit_notes_repository.rs`** (suite) :

16. **`credit_note_refused_when_vat_payable_archived`** — le compte de TVA due des réglages archivé en
    SQL après la validation d'une facture avec TVA → `DbError::CreditNoteAccountsArchived` nommant ce
    compte (C-15-6-29), rien d'écrit. *(Rougit sur le code actuel : `InactiveOrInvalidAccounts`.)*

**`crates/kesh-db/tests/credit_notes_repository.rs`** (suite) — P5 (C-15-6-32) :

17. **`credit_note_lock_leaves_a_foreign_account_unlocked`** — l'identifiant étranger (AC6, patron du
    test de la 15-5d, choix C51) : un compte d'une **seconde** société, **sans** `UPDATE` des réglages (l'appel
    direct reçoit ses ids : le réglage n'y jouerait aucun rôle) ; une connexion ouvre une transaction et appelle **directement** le
    helper de verrou de liste en mode `Share` avec des ids explicites (la créance de la vente et
    l'identifiant étranger), **sans conclure** ; une seconde connexion tente `SELECT id FROM accounts WHERE id = ? FOR UPDATE NOWAIT`
    sur la ligne étrangère → **réussit**. S'il échoue, le helper prend le patron `owned_account_ids`
    (AC6) et le test reste. *(C'est une mesure, pas un test rouge d'abord. **Issue attendue, d'après la
    mesure du dépôt : rouge** — `opening_complement.rs:434-436` a mesuré à deux sessions qu'un verrou
    par clé primaire prend la ligne d'une autre société avant que le filtre `company_id` ne l'écarte,
    et sous REPEATABLE READ ce verrou n'est pas relâché ; un S suffit à faire échouer le `NOWAIT` —
    d'où, vraisemblablement, `owned_account_ids`. **Sauf** si la 15-5d l'a déjà adopté par son propre
    test de C51 : T0 le constate, et le test 17 est alors vert d'emblée. Finding R6-5 de la P6.)*
**`crates/kesh-api/src/errors.rs`, module de tests** (patron du test unitaire du bras
`CreditNoteRevenueAccountsArchived`, `:3715`) :

18. **Bras de `CreditNoteAccountsArchived`** (finding R5-9 de la P5) — la variante rend **400**,
    `code = "ACCOUNT_ARCHIVED"`, `details.rejected[0]` = `{ accountId, accountNumber }`, et un message
    construit sur la clé `credit-note-account-archived` (repli fr compris si l'i18n n'est pas
    initialisée). Les quatre locales restent contrôlées par la parité de clés seule ; la traduction se
    relit en revue.

Soit **16 tests neufs et 2 tests modifiés** (9, 13) — périmètre : du commit de spécification à la fin
du développement, avant revue ; le 13 n'existe qu'après le merge de la 15-5d.

Non-régression : les tests de `credit_notes_repository.rs` (neuf, dix après la 15-5d), `invoices_validate_vat.rs` (les deux
autres tests d'avoir arrondi, `:783` et `:820`), `invoice_settlement.rs`, `invoice_write_off.rs`,
`reconciliation_e2e.rs`, et **`crates/kesh-db/tests/invoices_line_revenue_account.rs`** — c'est lui
qui fige que la 6 ter, dont l'AC6 change la source de `active`, garde son code, son message par
ligne et sa place (finding F6-5 de la P6) : `credit_note_uses_materialized_account_not_current_default`
(`:279`), `credit_note_fails_when_snapshot_account_archived` (`:569`, `line_number == Some(1)`,
`account_number == Some("3200")`), `credit_note_ignores_postable_and_type_changes` (`:615`),
`null_and_explicit_default_merge_into_one_credit_line` (`:789`).

## Dev Notes

### Ce qui doit être préservé

- **Verrous et ordre** de `create_credit_note` : facture `FOR UPDATE` (1), règlements `FOR UPDATE`
  (garde 25-4-a), avoir existant `FOR UPDATE`, réglages (3), puis l'exercice (4) et la séquence (5).
  Les deux **lecteurs** s'insèrent après (3) et ne prennent aucun verrou : ils lisent des **lignes
  d'écriture**, gelées (une écriture enregistrée ne se modifie plus). L'**état des comptes**, lui, ne
  l'est pas : il est verrouillé à part, **en partage, juste après les lecteurs et avant l'exercice**,
  par l'AC6 — le gel de l'écriture ne dit rien de l'état actif du compte qu'elle vise. Ordre canonique
  de `validate_invoice` (`invoices.rs:1916-1925` au commit `1920381e`) ; sa phrase, citée exactement
  (`invoices.rs:1921-1922`) : « Aucun chemin ne verrouille `accounts` **avant** `invoices` ou
  `fiscal_years`. » — en tension avec son propre « 1 bis » (`accounts` avant `fiscal_years`), et
  retirée par la **15-5e** (son AC5), non par cette story (AC7). L'invariant juste, que suit l'AC6 : aucun verrou **explicite**
  (`FOR UPDATE` / `LOCK IN SHARE MODE`) sur `accounts` **après** `fiscal_years` dans ces flux ; les S
  de clé étrangère des insertions de lignes (`INSERT INTO journal_entry_lines`) viennent après
  l'exercice — la validation, l'avoir lui-même (étape (7)), `accept_one_invoice` (`:1581` après
  `:1520`) — et ne gênent pas un S ; ils attendent un X. L'AC6 place son verrou avant l'exercice pour
  la **course de lecture** (`active` lu frais, archivage concurrent qui attend), selon la convention
  du doc-comment canonique (comptes → exercice) ; elle n'en tire **aucune** règle de fermeture de
  cycle — la défense contre l'interblocage est le rejeu de la route (15-5e, C54 ; finding R6-3 de la
  P6).
- **`version` de la facture** : l'avoir l'incrémente déjà (invariant de `settle_invoice`, étape (8)) ;
  ne pas y toucher.
- **Garde de postabilité désactivée** (`create_in_tx(..., false)`, 14-3b D-A0) : inchangée. La
  créance et le compte d'arrondi de la vente ont été imputables au moment de la vente ; ne pas
  re-vérifier `postable` (même raisonnement que la 6 ter de la 16-1a : viser les **mêmes** comptes que
  l'écriture d'origine).

### Pièges

- **Ne pas changer `ORDER BY jel.id` en `line_order`** dans les lecteurs, même si la colonne existe
  (`journal_entries.rs:417`) : trois chemins éprouvés reposent sur `jel.id`, et le lecteur de créance
  doit être la même requête, pas une nouvelle ; le lecteur d'arrondi suit le même ordre.
- **Ne pas réutiliser `usable_designated_account`** pour l'arrondi de l'avoir : elle exige « charge ou
  produit, actif, imputable » d'un compte **désigné** ; la doctrine de l'avoir est « mêmes comptes que
  l'origine, seule l'inactivité bloque » (AC4, AC6).
- **Greper la valeur, pas la formulation** (propagation post-patch) : après T3, greper
  `default_receivable_account_id`, `rounding_account_for_write` et `Issuance` sur tout le dépôt (code,
  doc-comments, `docs/`, manuels) pour trouver les phrases qui disent encore que l'avoir lit les
  réglages.
- **Migration** : aucune. Pas de P5/P6/P7.
- **Frontière 15-5a à 15-5e** : deux fichiers `credit_notes.rs` sont à distinguer (finding R6-7 de la
  P6). Le **dépôt** `crates/kesh-db/src/repositories/credit_notes.rs`, que cette story réécrit, est
  touché par la **15-5d** (son test des générateurs, `:933`, choix C50 — « `credit_notes.rs` aussi pour
  les 19 tests des générateurs ») ; la **route** `crates/kesh-api/src/routes/credit_notes.rs` est
  enveloppée dans le rejeu par la **15-5e** (son AC3). Les autres ne touchent ni l'un ni l'autre, mais
  partagent des **fonctions** avec cette fiche —
  `accept_one_invoice` (15-5b AC6 : commentaire de classement sur le bloc même que l'AC2 remplace, à
  conserver et réécrire, AC7), `settle_invoice` (15-5a AC4, 15-5b AC18 ; route rejouée par la 15-5e),
  `write_off_invoice` (15-5e AC5, commentaire « 5 bis »), `validate_invoice` (15-5b, 15-5d, 15-5e AC5 :
  doc-comment canonique) et `invoices_validate_vat.rs` (15-5b, 15-5d), le passage du manuel sur
  l'avoir (15-5b AC17, 15-5d AC8), le helper de verrou de liste de la 15-5d (AC6), le Pattern 5
  (15-5e AC5) — et la 15-5d **touche** `credit_notes_repository.rs` (test de C35, ré-ancré ici,
  § *Tests* 13) et y laisse des doc-comments sur l'exemption de l'avoir (AC7). **Une seule règle
  d'ordre des verrous pour l'epic, celle de la 15-5e (C54)** : l'ordre est une convention de
  fréquence, la défense est le rejeu ; l'AC6 garde son verrou pour la course de lecture (réglages,
  comptes par `id`, exercice), ne réordonne rien pour fermer un cycle et ne prétend à l'absence
  d'aucun cycle. Les
  textes que la 15-5e réécrit ne sont **pas** réécrits ici (AC7). Rebase attendu avec conflits **de
  forme** ; T0 refait les numéros de ligne.

### Références

- [Source: crates/kesh-db/src/repositories/credit_notes.rs:187-258, :261-620 (dont :238-239, :365-367, :418-451, :507-545)]
- [Source: crates/kesh-db/src/repositories/opening_complement.rs:38-39, :502, :540 (patron `LOCK IN SHARE MODE`) ;
  crates/kesh-db/src/repositories/invoice_settlements_write.rs:431-495, :466-468 ;
  crates/kesh-api/src/routes/credit_notes.rs:183 ; crates/kesh-db/src/retry.rs:83 (`is_deadlock_error`), :105 (`retry_on_deadlock`, rejeu de la route par la 15-5e) ;
  crates/kesh-api/src/routes/reconciliation.rs:1487, :1520, :1581, :1703 ; crates/kesh-api/tests/reconciliation_e2e.rs:4426]
- [Source: issue #536 (cycle avoir ↔ rapprochement de la même facture) ; `epic-15-choix-autonomes.md` C-15-6-29, C-15-6-32]
- [Source: `15-5e-ordre-des-verrous-reglements.md` (dépôt principal, réécrite selon C54 « Rejeu sur interblocage
  des flux d'écriture » ; AC3 route de l'avoir rejouée, AC5 doc-comment canonique, « 5 bis » et Pattern 5) ;
  registre C54, C55, C57 ; `15-5d-garde-usage-comptes-reglage.md`
  (AC1 `lock_designated_accounts_in_tx`, C51 `EXPLAIN` et identifiant étranger) ;
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md:285-330` (Pattern 5) ; crates/kesh-db/src/repositories/opening_complement.rs:274-288, :431-440]
- [Source: crates/kesh-db/src/errors.rs:291, :604-614, :796 (`map_db_error`) ; crates/kesh-db/src/repositories/journal_entries.rs:96-98, :1622-1627, :1723-1754 ;
  crates/kesh-api/src/errors.rs:2945-2973, :3155-3185 ; crates/kesh-db/src/repositories/accounts.rs:604-634]
- [Source: crates/kesh-db/src/repositories/invoices.rs:1916-1925 (ordre canonique des locks) ;
  crates/kesh-db/src/repositories/fiscal_years.rs:548 ; crates/kesh-api/src/routes/reconciliation.rs:887, :1688-1706 ;
  crates/kesh-db/src/test_fixtures.rs:560 ; crates/kesh-api/tests/invoice_echeancier_e2e.rs:742 ;
  crates/kesh-i18n/locales/fr-CH/messages.ftl:291, :354]
- [Source: crates/kesh-db/src/repositories/invoice_settlements_write.rs:1-13 (doctrine), :97-115, :428-445]
- [Source: crates/kesh-api/src/routes/reconciliation.rs:1414-1452]
- [Source: crates/kesh-db/src/repositories/invoices.rs:1870-1912, :2005-2010, :2068]
- [Source: crates/kesh-db/src/repositories/company_invoice_settings.rs:400-412, :484]
- [Source: crates/kesh-db/tests/invoices_validate_vat.rs:592-930 ; crates/kesh-db/src/test_fixtures.rs:212]
- [Source: docs/manual/fr/user-manual.tex:363, :847, :1148-1180 ; docs/manual/fr/admin-manual.tex:2027]
- [Source: issues #473, #523, #525 ; `epic-15-choix-autonomes.md` C-15-6-1, C-15-6-2, C-15-6-7, C-15-6-8, C-15-6-16, C-15-6-24, C-15-6-25]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

**T0 — relevés (2026-10-08, sur `origin/main` `9cb5083b` ; branche de la 15-5d lue à `5624ca78`, non mergée).**

- **Rejeu de la route — constaté.** `crates/kesh-api/src/routes/credit_notes.rs:187` (`create_credit_note`)
  appelle `kesh_db::retry::retry_on_deadlock("credit_notes::create", …)` (`:193`), doc-comment « Rejouée sur
  interblocage (Story 15-5e2, #536) » (`:184-186`) ; registre `crates/kesh-api/tests/audit_route_registry.rs:220`
  : `("post", "credit_notes::create_credit_note", Traced, Rejouee)`. Le rejeu vient de la **15-5e2** (la fiche
  écrit « 15-5e » : la 15-5e a été découpée en 15-5e1/15-5e2, C61). `is_deadlock_error` : `crates/kesh-db/src/retry.rs:115` ;
  `retry_on_deadlock` : `:150`. Fichier des tests de rejeu : `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` (existe).
- **Helper de verrou de liste — forme sur la branche de la 15-5d** (`company_invoice_settings.rs`, ~`:595-740` à
  `5624ca78`) : `lock_designated_accounts_in_tx(conn, company_id, ids) -> Result<DesignatedAccountsSnapshot, DbError>`,
  **`pub(in crate::repositories)`** ; **déjà en partagé** (`LOCK IN SHARE MODE`, choix C87 de la 15-5d — il n'y a
  plus d'appelant `FOR UPDATE`) ; **déjà** le patron `owned_account_ids` (C88 : lecture non verrouillante des ids
  de la société, puis verrou sur ces seuls ids, `company_id` gardé en défense) ; plan épinglé
  `FORCE INDEX (PRIMARY)` ; ids triés et dédoublonnés, `ORDER BY id` ; ne refuse rien. Type de ligne :
  `LockedDesignatedAccount { id, number, active, postable }`, **privé**, enveloppé dans le newtype
  `DesignatedAccountsSnapshot(Vec<…>)`. Son doc-comment liste « l'avoir, qui relit la créance et la TVA due dans
  les réglages du moment sans ce contrôle (C35) » parmi « ce qui n'est pas contrôlé ici » (AC7 : à réécrire).
  **Absent de `HEAD`** (`grep -rn "lock_designated_accounts_in_tx" crates/` vide).
- **Ordres de verrous relevés** (relevé, sans exigence de conformité, R6-3) :
  validation `invoices.rs:1921-1957` (doc-comment canonique réécrit par la 15-5e1 : facture → réglages
  `FOR UPDATE` → arrondi `FOR UPDATE` → produit en partagé par clé étrangère → exercice `:2205` → séquence →
  écriture) ; la 15-5d y insère, sur sa branche, le S de la créance et de la TVA due **après** l'arrondi et
  **avant** l'exercice (`invoice_settlements_write.rs` « 5 bis » de sa branche) ;
  règlement client `invoice_settlements_write.rs:48` : facture, compte de contrepartie `FOR UPDATE`
  (`:122`, `:159`), arrondi (`:200`), exercice (`:211`) ;
  solde du reste `:367` : nature (étape 4), arrondi (`:474-499`), TVA due, exercice (`:509`) — commentaire
  « 5 bis » `:474-486` ;
  saisie fournisseur `supplier_invoices.rs` : réglages `:376` **avant** l'exercice `:429` (le cycle (iv) l'exclut à
  juste titre — R7-5 tranché).
- **Commentaire de classement de la 15-5b** dans `accept_one_invoice` : c'est le **doc-comment** de la fonction
  (`reconciliation.rs:1216-1222`, « la créance est lue **sur l'écriture de vente** de la facture »), sans numéro
  de ligne ni requête citée ; il reste vrai après l'AC2. Le bloc remplacé est `(b)` `:1464-1501`.
- **Textes de la 15-5e lus** : doc-comment canonique de `validate_invoice` (ne mentionne pas l'avoir) ; « 5 bis »
  (ne mentionne pas l'avoir) ; Pattern 5 `docs/MULTI-TENANT-SCOPING-PATTERNS.md:285-335` — **aucune ligne pour
  l'avoir**, et les lignes des flux comptables y sont désormais **par renvoi au doc-comment canonique** (« the
  order is written there only », C68). → aucune phrase décrivant l'ancien ordre de l'avoir (AC7 : « rien »).
- **Manuel** (lu en entier, `HEAD` et branche 15-5d) : sur `HEAD`, `user-manual.tex:382` (4) dit « un compte
  désigné … devenu non imputable … reste utilisé par les écritures automatiques (validation d'une facture,
  **avoir**, facture fournisseur) ». Sur la branche 15-5d, `:380` (« Deux exceptions, voulues : l'**avoir** relit
  la créance et la TVA due dans les réglages au moment de l'avoir, sans les contrôler ») et `:928` (§ « Un compte
  des réglages devenu non imputable » : « L'avoir, lui, n'est pas soumis à ce contrôle … même si sa créance ou sa
  TVA due est devenue non imputable ») — la phrase sur la **créance** deviendra fausse (AC8). `admin-manual.tex`
  : rien sur l'avoir et les réglages hors `:2035` (« un avoir le reprend et l'annule »), qui reste vrai.
  Numéros actuels : `sec:avoirs` `:1226`, puce « revient à zéro » `:1232`, `keshwarning` produit archivé `:1249`,
  `sec:plan-comptable-archivage` `:361` (avertissement `:364`), arrondi `:924`, balance âgée `:1836` ;
  `admin-manual.tex:2035`.
- **Tests** : `credit_notes_repository.rs` compte **9** `#[sqlx::test]` sur `HEAD` (`:101` … `:593`, inchangé) ;
  **la 15-5d n'y ajoute rien** : son test de C35 est `credit_note_is_exempt_from_the_guard`, dans
  `crates/kesh-db/tests/invoices_validate_vat.rs` (`~:1395-1416` sur sa branche, module des tests de la 15-5d) ;
  le test 13 se ré-ancre là. Son test `foreign_account_is_never_locked_and_is_refused` (même fichier) mesure déjà
  l'identifiant étranger : le test 17 sera **vert d'emblée** (cas prévu par la fiche).
- **Inventaire recompté** (commandes de la fiche) : deux écrivains neufs depuis `1920381e` —
  `journal_entries::update` (15-8a, `INSERT INTO journal_entry_lines` `journal_entries.rs:1480`) et
  `journal_entries::delete_in_tx` (15-8b, `:1648`). **Hors classe** (lignes saisies ; suppression) ; tous deux
  refusent une écriture **possédée par une pièce** (`journal_entries.rs:45`, `:1163`, motifs `:1955-1965`), donc
  l'écriture de vente d'une facture validée reste **gelée** — la prémisse des deux lecteurs tient. Ils
  s'ajoutent aux partenaires possibles du cycle (iv) (ils tiennent l'exercice sans la ligne des réglages) — liste
  déjà déclarée non exhaustive. Aucun autre site neuf ne relit créance, dette ou arrondi.

### File List

## Change Log

- 2026-10-08 — Spécification initiale (bmad-create-story, en autonomie). Statut `ready-for-dev`.
- 2026-10-08 — **Validation P1** (Sonnet 4.6, lentilles F et R ; prompt
  `15-6a-validate-prompt-p1.md`). F : 1 HIGH, 3 MEDIUM, 4 LOW ; R : 1 HIGH, 3 MEDIUM, 4 LOW (les deux
  HIGH convergent, deux MEDIUM aussi). Appliqués :
  - **HIGH F1 = R1** — #523 absorbée : lecteur de la dernière ligne avec recoupement sens + montant
    (AC4), refus du compte de la vente archivé par `create_in_tx` (AC6, **voie (b)** : le test `:846`
    change d'assertion, délibérément — la voie (a) aurait gardé un message renvoyant à tort aux
    paramètres, la voie (c) créé un refus neuf), deux tests neufs (réglage redésigné, réglage vidé), un
    test neuf du recoupement et le test `:846` modifié (compte archivé). Choix C-15-6-7. *(Phrase
    corrigée en P2, finding R-5 : elle comptait le test modifié parmi les neufs ; le total, lui, était
    juste.)*
  - **MEDIUM F3** — TVA due de l'avoir : angle mort tracé par #525 (ouverte par l'orchestrateur) ; le
    manuel dit la limite sans présenter l'écriture comme juste (AC8). Choix C-15-6-8.
  - **MEDIUM F2 = R2** — le `grep` de l'AC2 se limite aux `.rs` ; les backfills `.sql` sont nommés
    comme légitimes.
  - **MEDIUM F4 = R5** — l'erreur SQL du lecteur reste `DATABASE_ERROR` dans le rapprochement ;
    `INVOICE_SALE_ENTRY_MALFORMED`, qu'aucun test ne figeait, l'est par un test neuf (AC2, test 10).
  - **MEDIUM R3** — « dix tests » → neuf, lignes citées.
  - **MEDIUM R4 = F5** — tests des lecteurs en `#[sqlx::test]`, placés dans le fichier dont ils
    emploient les helpers privés ; seconde société désignée.
  - **LOW** — citation exacte du commentaire `:609-611` (R6) ; `accounts::archive` et choix du SQL
    direct (R7) ; `user-manual.tex:1754` à ne pas réécrire (R8) ; contrôle aplati de la phrase neuve
    (F6) ; test 2 borné à la créance, l'avoir exigeant toujours le produit par défaut (F8). Les trois
    LOW restants ne figurent pas dans le rapport condensé transmis à la remédiation : rien n'en a été
    appliqué, et ce n'est pas affirmé autrement.
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 10 tests** (9 neufs, 1 modifié).
- 2026-10-08 — **Validation P2** (Opus 5.5, lentilles R et F ; prompt `15-6a-validate-prompt-p2.md`).
  R : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW ; F : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW (R-2 = F-1,
  R-3 = F-2, R-1 ⊃ F-5, R-7 = F-6, R-8 ⊂ F-8). Tous appliqués, décisions de l'orchestrateur comprises :
  - **MEDIUM R-2 = F-1** — le verrou `FOR UPDATE` du compte d'arrondi (#486) n'est plus perdu : les
    comptes lus sur la vente (créance, arrondi) sont verrouillés **sans** filtre `active`, l'état actif
    lu sous verrou ; la course est fermée, testée (test 11, neuf). AC6, Dev Notes. Choix C-15-6-16.
  - **MEDIUM R-3 = F-2** — refus anonyme : le compte archivé est **nommé** par le refus existant de la
    contre-passation (`ReversalAccountsArchived`, 400 `ACCOUNT_ARCHIVED`, clé existante) — aucune
    variante ni clé neuve ; tests 3 et 9 suivent. Manuel : `keshwarning` neuf au § *Avoirs*,
    avertissement `:363` étendu, phrase `admin-manual.tex:2027` (AC8). Choix C-15-6-16. L'écart à la
    contrainte de #523 (le code change, le refus reste nommé) est écrit en AC6 ; **commentaire à
    poser sur #523** par l'orchestrateur.
  - **MEDIUM R-1 (⊃ F-5 LOW)** — frontière réelle avec les 15-5a/b/c/d écrite (en-tête, Pièges) ; T0
    rebase et refait tous les numéros de ligne ; commentaire de la 15-5b dans `accept_one_invoice`
    conservé et réécrit (AC7) ; phrase de la 15-5d sur l'avoir au manuel corrigée si elle est mergée
    (AC8).
  - **LOW** — R-4 : site `reconciliation.rs:1487` ajouté à l'inventaire ; F-4 : dévalidation
    (`delete_in_tx`) et `create_opening_entry` classés, deux commandes de recompte ajoutées ; R-5 :
    phrase du Change Log P1 corrigée ; R-6 : `:100` → `:101` ; R-7 = F-6 : invariant « l'arrondi est
    la DERNIÈRE ligne » écrit à la source (AC7) ; R-8 = F-8 : texte de `DATABASE_ERROR` dérivé de
    `DbError` (AC1), montage léger du test 10 proposé ; F-3 : contrôle PDF sur des fragments absents
    du PDF actuel, apostrophes en `.` (AC8) ; F-7 : `### Modifié` au CHANGELOG, `docs/api-external.md`
    sans objet (AC9). **R-9 non appliqué tel quel** : la consigne du registre interdit de réécrire une
    entrée existante ; la révision de C-15-6-2 reste portée par les titres de C-15-6-7, C-15-6-8,
    C-15-6-16 et par la fiche d'index.
  - Signal de découpage : sans objet (lentille F : quatre modules de code ; aucun défaut recyclé).
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 11 tests** (10 neufs, 1 modifié).
- 2026-10-08 — **Validation P3** (Sonnet, lentilles R et F ; prompt `15-6a-validate-prompt-p3.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 1 HIGH, 2 MEDIUM, 4 LOW ; F : 0 CRITICAL, 0 HIGH, 4 MEDIUM,
  6 LOW (R3-1 ⊃ F-5, R3-2 = F-4, R3-3 = F-10, R3-4 = F-7, R3-5 = F-6, R3-6 = F-2). Tous appliqués,
  décisions de l'orchestrateur comprises :
  - **HIGH R3-1 (+ MEDIUM F-1, LOW F-5)** — le verrou des comptes de la vente était placé **après**
    l'exercice et la séquence, à l'inverse de l'ordre canonique de `validate_invoice`
    (`invoices.rs:1916-1925`) et de celui de la 15-5d : cycle avec chaque validation sur la ligne de
    la créance. Il est désormais pris **juste après les réglages (3), avant l'exercice (4)**, et
    **en partage** (`LOCK IN SHARE MODE`) : un X sur 1100 ouvrait un cycle neuf avec
    `accept_one_invoice`, qui tient S sur la créance avant de demander X sur la facture. Ordre des
    refus fixé (compte de la vente archivé avant `FISCAL_YEAR_INVALID`), test 14 neuf ; section
    « Ordre des locks » au doc-comment de `create_credit_note` (AC7). La phrase « c'est celui de
    `validate_invoice` » et la « conséquence assumée » de C-15-6-16 étaient fausses : rectifiées par
    C-15-6-24 (le registre n'est pas réécrit).
  - **MEDIUM F-2 = R3-6** — test 11 sur `attendre_une_requete_en_cours` (motifs `FROM accounts`,
    `LOCK IN SHARE MODE`), facture sans arrondi, aucun délai fixe.
  - **MEDIUM F-3** — test 12 neuf, frontière HTTP de `POST /api/v1/credit-notes` (code, `details`,
    message), dans `invoice_echeancier_e2e.rs`, seul fichier qui émet un avoir par la route.
  - **MEDIUM F-4 = R3-2** — le test de C35 de la 15-5d est recensé en T0 et ré-ancré (test 13,
    modifié) ; doc-comments de l'exemption réécrits (AC7) ; « neuf tests » daté, dix après la 15-5d.
  - **MEDIUM R3-3 = F-10** — T0 **lit** les encadrés `user-manual.tex` ~`:380`/~`:390` et
    `admin-manual.tex:2016-2029` après le merge des 15-5, au lieu de greper une formulation de fiche ;
    grep de la valeur ensuite, PDF aplatis compris (AC8).
  - **LOW F-9** — un seul vocabulaire de refus sur la route : variante
    `DbError::CreditNoteAccountsArchived`, **même code** `ACCOUNT_ARCHIVED`, clé neuve
    `credit-note-account-archived` (« Impossible d'émettre l'avoir — compte(s) archivé(s) : … »)
    dans les quatre locales ; manuel et CHANGELOG disent les deux codes de la route (AC6, AC8, AC9).
    Choix C-15-6-25, qui révise le « aucune variante, aucune clé neuve » de C-15-6-16.
  - **LOW** — R3-4 = F-7 : `map_db_error` est dans `kesh-db/src/errors.rs:796` ; R3-5 = F-6 : ce que
    le verrou ne couvre pas (comptes de produit, TVA due) écrit à l'AC6 et à l'inventaire, angle mort
    assumé ; R3-7 : lignes d'attribut harmonisées, ordre `ORDER BY id` vs `number` de
    `details.rejected[]` dit ; F-8 : phrase « deux avoirs se sérialisent sur la ligne du compte »
    retirée (S + S ne sérialise pas ; les réglages sérialisent déjà).
  - **Signal D5 — levé et déclaré.** La sévérité remonte (P2 : 0 HIGH, 5 MEDIUM ; P3 : 1 HIGH,
    6 MEDIUM) et le HIGH **naît de la remédiation P2** (C-15-6-16) : c'est la forme « recyclage » de
    l'amendement D5, déclarée au Project Lead. Pas de découpage : trois modules de code (`kesh-db`,
    `kesh-api`, `kesh-i18n`) et la correction est un déplacement de verrou, tranché par
    l'orchestrateur. Une P4 est nécessaire (MEDIUM restants à cette passe) ; une passe **ciblée** sur
    l'AC6 et les tests 11-14 est la forme indiquée.
  - Trend : P1 (F+R) 2 HIGH, 6 MEDIUM → P2 0 HIGH, 5 MEDIUM → P3 1 HIGH, 6 MEDIUM.
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 14 tests** (12 neufs, 2 modifiés : 9 et 13).
- 2026-10-08 — **Validation P4** (Opus 5.5, lentilles R et F ; prompt `15-6a-validate-prompt-p4.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW ; F : 0 CRITICAL, 0 HIGH, 2 MEDIUM,
  4 LOW (R4-1 ≈ F-2, R4-2 = F-4, R4-6 = F-6, R4-9 = F-5). Tous appliqués, décisions de l'orchestrateur
  comprises (choix C-15-6-29) :
  - **MEDIUM F-1** — la TVA due et les comptes de produit demandaient leur S **après** l'exercice, face
    au X que le solde du reste y prend avant le sien : cycle préexistant que la 15-5d décrit et ferme
    pour la validation. Le verrou de l'AC6 couvre désormais **tous** les comptes que l'avoir écrit
    (créance, arrondi de la vente, TVA due si l'avoir porte de la TVA, produits effectifs, repli D-B2
    compris), en une requête `ORDER BY id`, avant l'exercice ; la 6 ter lit `active` dans ce verrou
    (son code, son message et sa place ne changent pas) ; un compte de TVA due archivé rend
    `ACCOUNT_ARCHIVED` nommé (il rendait `INACTIVE_OR_INVALID_ACCOUNTS`) — AC6, AC8, AC9, test 16 neuf.
    L'angle mort « autre classe » de l'inventaire **disparaît**.
  - **MEDIUM R4-1 ≈ F-2 (+ LOW F-3)** — l'analyse « S + S ne bloque pas » était fausse pour l'arrondi :
    réécrite (un S n'ajoute aucun cycle par la créance, il n'en retire aucun) ; trois cycles résiduels
    inscrits à l'AC6 et à l'inventaire — avoir ↔ rapprochement de la même facture, **préexistant, tracé
    par #536** (ouverte par l'orchestrateur) ; avoir ↔ solde du reste par ordres opposés entre deux
    comptes ; cycle à trois par un X en attente (neuf). La route `POST /api/v1/credit-notes` reçoit
    `retry_with` sur interblocage, figé par le test 15 neuf, qui exerce le prédicat. Le registre
    C-15-6-24 est rectifié par C-15-6-29 (non réécrit).
  - **MEDIUM R4-2 = F-4** — Dev Notes : la phrase de `invoices.rs:1921-1922` est citée exactement
    (« avant ») ; l'invariant juste est écrit (verrous **explicites** sur `accounts` avant l'exercice,
    S de clé étrangère après) ; le commentaire source et celui de `invoice_settlements_write.rs:466-468`
    sont à corriger (AC7).
  - **LOW** — R4-3 : `code()` → `error_code()` (`errors.rs:736`) ; R4-4 : patron existant
    `opening_complement.rs:502` ; R4-5 : doc-comment du test 9 réécrit (le refus précède la séquence) ;
    R4-6 = F-6 : ordre des refus complet (`ConfigurationRequired` du produit, `Invariant` des
    lecteurs) ; R4-7 : sérialisation par la ligne des réglages nommée ; R4-8 : grep du test 12 avec
    `ACCOUNT_ARCHIVED` ; R4-9 = F-5 : ligne absente du verrou → `Invariant` ; R4-10 : lecture de la 6 ter
    citée `:450-451` (et non `:445-447`, numéros relus au code).
  - Signal D5 : sévérité en baisse (P3 : 1 HIGH, 6 MEDIUM ; P4 : 0 HIGH, 4 MEDIUM dont deux convergents) ;
    F-1 est un défaut préexistant, R4-1/F-2 corrige un raisonnement écrit en P3 (recyclage partiel), sans
    nouveau module de code (`kesh-db`, `kesh-api`, `kesh-i18n`). Pas de découpage. Une P5 est
    nécessaire (MEDIUM à cette passe).
  - Trend : P1 (F+R) 2 HIGH, 6 MEDIUM → P2 0 HIGH, 5 MEDIUM → P3 1 HIGH, 6 MEDIUM → P4 0 HIGH, 4 MEDIUM.
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 16 tests** (14 neufs, 2 modifiés : 9 et 13).
- 2026-10-08 — **Validation P5** (Sonnet, lentilles R et F ; prompt `15-6a-validate-prompt-p5.md` ;
  remédiation Opus 5.5). R : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW ; F : 0 CRITICAL, 0 HIGH, 1 MEDIUM,
  6 LOW (R5-1 = F5-1, R5-4 = F5-2). Tous traités (choix **C-15-6-32**), selon la décision de
  l'orchestrateur — **une seule règle d'ordre des verrous pour l'epic, celle de la 15-5e** —, telle
  que la 15-5e est **réécrite par C54** pendant cette remédiation (la défense contre l'interblocage
  est le **rejeu** ; l'ordre, une convention de fréquence ; correction de consigne de l'orchestrateur
  reçue en cours de remédiation, appliquée) :
  - **MEDIUM R5-1 = F5-1** — la fiche ignorait la **15-5e** : dépendances « après 15-5a à 15-5e »
    (en-tête, T0, Pièges) ; l'AC7 **ne réécrit plus** le doc-comment canonique (`invoices.rs:1921-1922`)
    ni le commentaire « 5 bis » (`invoice_settlements_write.rs:466-468`), que la 15-5e réécrit (son
    AC5) ; elle ne corrige qu'une phrase qui décrirait l'ancien ordre de l'avoir, s'il y en a une ; la
    route `POST /api/v1/credit-notes` est **rejouée par la 15-5e** (son AC3) : la 15-6a ne spécifie
    plus son propre `retry_with` (révise C-15-6-29 sur ce point), son test 15 fige que ce rejeu
    couvre l'attente neuve de l'avoir.
  - **MEDIUM R5-2 (+ LOW F5-3)** — cycles avoir ↔ solde du reste (TVA due ↔ arrondi) et avoir ↔
    règlement par compte interne : **nommés**, non fermés par réordonnancement (C54 : ne pas
    réordonner l'avoir pour fermer des cycles) ; le verrou reste une instruction `ORDER BY id`, gardé
    pour la seule **course de lecture** ; toute affirmation d'absence de cycle est retirée (« ensemble
    clos », « pas un de plus ») ; les cycles connus sont listés comme non exhaustifs et renvoyés au
    rejeu.
  - **LOW F5-5 (DRY)** — un seul helper de verrou d'une liste de comptes, paramétré par le mode,
    partagé avec la 15-5d (`lock_designated_accounts_in_tx`), créé par la première story mergée (a
    priori la 15-5d). **LOW F5-4** — « patron existant » → « syntaxe » ; `EXPLAIN` relevé et test
    d'identifiant étranger comme la 15-5d (C51), test 17 neuf.
  - **LOW R5-4 = F5-2** — « l'avoir émet de la TVA » : agrégation extraite en fonction pure partagée
    par le générateur et le calcul des ids ; refus `ConfigurationRequired(TVA)` laissé à sa place
    (générateur, après l'exercice), écrit.
  - **LOW F5-6** — une ligne pour l'avoir au Pattern 5 de `docs/MULTI-TENANT-SCOPING-PATTERNS.md`,
    dans la forme que la 15-5e lui donne (AC9, T5).
  - **LOW** — R5-3 : symptôme exact du test 11 sur le code actuel ; R5-5 : « 15-5b AC12 » → AC18 ;
    R5-6 : test 14 « comptes de l'avoir » ; R5-7 : garde `active` de `create_in_tx` « hors réactivation
    concurrente » ; R5-8 : lest du test 15 repris tel quel du patron ; R5-9 : test unitaire du bras de la
    variante neuve (test 18) ; F5-7 : le manuel dit que rétablir le réglage de TVA due est l'autre
    issue, et le remplacement inatteignable du refus de la 6 ter par `Invariant` est écrit (sans test).
  - Propagation : jetons `15-5a, 15-5b, 15-5c et 15-5d`, `15-5a/b/c/d`, `15-5b AC12`, `comptes de la
    vente`, `arrondi d'abord`, `(S, seul)`, `retry_with`, `ensemble clos`, `formes A`, `patron
    **existant**`, `1921-1922`, `466-468` grepés sur la fiche — résidus restants légitimes (Change Logs
    P2-P4, références au commit `1920381e`, AC7 qui nomme ce que la 15-5e réécrit, références de code).
  - **Signal D5** : non levé par la sévérité (P4 0 HIGH, 4 MEDIUM → P5 0 HIGH, 3 MEDIUM, dont deux
    convergents). ⚠️ **R5-1 et R5-2 ne viennent pas d'un patch** : ils naissent d'un **changement
    extérieur** — la création de la 15-5e (C52) puis sa réécriture (C54), postérieures à la P4 —, non
    d'une remédiation de cette fiche ; ce n'est pas un recyclage. Aucun module de code ajouté. Pas de
    découpage.
  - Trend : P1 (F+R) 2 HIGH, 6 MEDIUM → P2 0 HIGH, 5 MEDIUM → P3 1 HIGH, 6 MEDIUM → P4 0 HIGH,
    4 MEDIUM → P5 0 HIGH, 3 MEDIUM. La remédiation P5 retire le rejeu propre à la route et réécrit la
    doctrine des verrous de l'AC6 : une **P6 complète** (Opus, rotation D6) est indiquée.
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 18 tests** (16 neufs, 2 modifiés : 9 et 13).
- 2026-10-08 — **Validation P6** (Opus 5.5, lentilles R et F ; prompt `15-6a-validate-prompt-p6.md` ;
  rapports `target/gate-logs/15-6a-p6-R.md`, `15-6a-p6-F.md` ; remédiation Opus 5.5). R : 0 CRITICAL,
  0 HIGH, 3 MEDIUM, 7 LOW ; F : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 5 LOW (F6-1 = R6-1 ; F6-2 = R6-4, MEDIUM
  côté F, LOW côté R — retenu MEDIUM ; F6-7 observation). **4 MEDIUM distincts**, tous traités (choix
  **C-15-6-35**) :
  - **MEDIUM F6-1 = R6-1** — deux contrats pour la fonction de verrou : le paragraphe « Refus nommé »
    (`lock_accounts_in_share_mode`, `pub(crate)`, « à côté de » `archived_accounts_in_tx`, « le
    contrôle « ligne absente » y vit ») contredisait la puce P5. Retiré ; la puce « Un seul helper »
    devient le **seul contrat** : paramétré par le mode, à l'emplacement de la 15-5d, colonnes `id,
    number, active, postable`, **ne refuse rien** (« ligne absente » et partage chez l'appelant),
    **`pub`** (le test 17 est un test d'intégration), type de ligne **nommé et distinct** de
    `opening_complement::LockedAccount` (privé, autres colonnes : ni réutilisé ni homonyme).
  - **MEDIUM R6-2** — le critère DRY était faux dès aujourd'hui : `opening_complement.rs:518-537`
    verrouille une liste d'ids de comptes. Nommé comme **exception écrite** (forme distincte, hors
    périmètre de l'epic) ; toute autre occurrence est un site non résolu.
  - **MEDIUM R6-3** — doctrine retirée en P5 restée aux Dev Notes (« d'où la règle de l'AC6 … se
    verrouille avant l'exercice ») et en T0 (« vérifier que l'AC6 lui reste conforme ») : réécrits —
    verrou avant l'exercice pour la course de lecture, aucune règle de cycle ; T0 **relève** l'ordre de
    la 15-5d sans exigence de conformité.
  - **MEDIUM F6-2 (= R6-4)** — cycle (iv) : la validation, sérialisée avec l'avoir par la ligne des
    réglages (la fiche le dit elle-même), ne peut pas en être le partenaire. Partenaires réels nommés —
    saisie manuelle, règlement, solde du reste, rapprochement (flux qui prennent l'exercice sans les
    réglages) ; validation et saisie fournisseur explicitement exclues ; table de l'inventaire alignée.
  - **LOW** — R6-5 : test 17, issue attendue **rouge** d'après `opening_complement.rs:434-436` (puis
    `owned_account_ids`), sauf si la 15-5d l'a adopté (T0), T4 aligné ; R6-6 : `"../kesh-db/test-schema"`
    pour les tests de `kesh-api` ; R6-7 : frontière des deux `credit_notes.rs` (dépôt : 15-5d ; route :
    15-5e) ; R6-8 : 6940 « plus loin dans le plan livré, ou créé après lui » ; R6-9 : `retry_with`
    retiré des Références, remplacé par `retry.rs:83, :105` ; R6-10 : la 15-5d est une dépendance
    **ferme** (en-tête, test 13 : la story attend) ; F6-3 : exemple du 3800 corrigé (le plan des
    associations ne désigne aucun escompte d'office) ; F6-4 : « inatteignable par l'application » au
    lieu de « par les clés étrangères » (aucune clé étrangère ne porte la société) ; F6-5 :
    `invoices_line_revenue_account.rs` (quatre tests de la 6 ter) à la non-régression ; F6-6 : relève
    de l'orchestrateur (commentaire sur #536 : l'avoir est couvert par le rejeu de la 15-5e, la 15-6a
    ne ferme aucun cycle) — **signalé**, rien dans la fiche.
  - **F6-7 — décompte au barème de la règle**, écrit ici et déclaré : modules métier de premier niveau
    touchés — `kesh-db/repositories/invoice_settlements` (lecteurs), `…/invoice_settlements_write`
    (règlement, solde), `…/credit_notes` (avoir), `…/invoices` (doc-comment seul),
    `…/company_invoice_settings` (helper, seulement si la 15-5d l'a posé sans mode),
    `kesh-api/routes/reconciliation` — **six**, plus la plomberie `kesh-db/errors`, `kesh-api/errors`,
    `kesh-i18n` : **plus de cinq** au barème de la règle (le décompte « trois ou quatre modules de code »
    des P2-P5 était par crate). Le critère 1 ne vaut qu'avant la validation ; il est **déclaré**, non
    rétroactif.
  - Propagation : jetons `lock_accounts_in_share_mode`, `LockedAccount`, `à côté d'elle`, `y vit`,
    `règle de l'AC6`, `reste conforme`, `se verrouille avant l'exercice`, `tenu par une validation`,
    `validate_invoice` (table des cycles), `vert attendu`, `clés étrangères`, `créé après le plan`,
    `Autres produits`, `./test-schema`, `retry_with`, `pas mergée` grepés sur les fiches 15-6 et le
    registre — résidus restants légitimes (Change Logs P2-P5 ; « clés étrangères » du lest du test 15,
    autre sens ; `LockedAccount` cité pour être écarté).
  - **Signal D5 — levé, déclaré au Project Lead, pas de découpage** (C-15-6-35). Sévérité P5 → P6
    MEDIUM → MEDIUM (2 → 4 distincts), et **trois des quatre MEDIUM recyclent la remédiation P5**
    (R6-1 = F6-1, R6-2, R6-3 : la refonte doctrinale de l'AC6 laissée incomplète — un paragraphe, un
    critère et une note non grepés) ; le quatrième (F6-2) date de la P4. Recyclage **contenu** : tous
    portent sur le **texte** de l'AC6 et de ses Dev Notes, aucun sur une règle métier ni un module ; la
    remédiation P6 retire des phrases et en unifie d'autres, elle n'ajoute aucun mécanisme. Découper
    l'AC6 n'en retirerait pas le défaut (une propagation incomplète) ; le geste qui le traite est le
    grep du symptôme, fait ici. Avec le critère de dispersion (F6-7), le signal est déclaré deux fois ;
    l'arbitrage revient à Guy.
  - Trend : P1 (F+R) 2 HIGH, 6 MEDIUM → P2 0 HIGH, 5 MEDIUM → P3 1 HIGH, 6 MEDIUM → P4 0 HIGH,
    4 MEDIUM → P5 0 HIGH, 3 MEDIUM → **P6 0 HIGH, 5 MEDIUM** (4 distincts). Modèles : P1 Sonnet 4.6,
    P2 Opus, P3 Sonnet, P4 Opus, P5 Sonnet, P6 Opus ; remédiations Opus 5.5. La boucle **ne s'arrête
    pas** : une **P7** est due (rotation D6 : Sonnet, passe **complète** — la remédiation réécrit le
    contrat d'un helper et un cycle, et trois MEDIUM de la P6 venaient d'une remédiation).
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 18 tests** (16 neufs, 2 modifiés : 9 et 13) —
    inchangé.

- **Validation P7** (Sonnet, lentilles R et F, prompt `15-6a-validate-prompt-p7.md`) : 0 CRITICAL, 0 HIGH,
  **1 MEDIUM**, 10 LOW (R : 1 MEDIUM + 4 LOW ; F : 6 LOW ; R7-2 = F7-1 : la même phrase).
  - **MEDIUM R7-1** — le type de ligne du helper : la fiche (« la 15-6a prend le sien ») et le registre
    (C-15-6-35 (1) : « ou celui de la 15-5d ») s'excluaient. **Un seul type, jamais deux** : emploi de celui de
    la 15-5d s'il est nommé et `pub`, sinon rendu public (AC6) ; C-15-6-36 révise C-15-6-35 (1).
  - **LOW** — R7-2 = F7-1 : phrase d'en-tête réparée ; R7-3 : quatre colonnes (`postable` incluse, ignorée) ;
    R7-4 : C-15-6-32 (6) et C-15-6-29 marqués révisés (C-15-6-36) ; R7-5 : cycle (iv), `invoices.rs:2006/:2172`
    cités pour la validation seule, T0 relève la saisie fournisseur ; F7-2 : test 17 en appel direct, sans
    `UPDATE` des réglages ; F7-3 : lignes de `credit_notes.rs` et `opening_complement.rs` corrigées ;
    F7-4 : `keshwarning` du manuel, rôle *Créances clients* à retirer du nouveau compte avant la réactivation ;
    F7-5 : « par SQL direct » aux tests 2 et 7 ; F7-6 : note de PR (`closes #523` renvoie au commentaire du
    2026-10-08), consignée en C-15-6-36.
  - Propagation : jetons `prend le sien`, `s'il existe`, `première story mergée`, `chaque validation`,
    `LockedAccountState`, `518-537`, `523-545` grepés sur les fiches 15-6 et le registre ; résidus restants
    légitimes (entrées historiques du registre, Change Logs P2-P6).
  - Trend : P1 (F+R) 2 HIGH, 6 MEDIUM → P2 0 HIGH, 5 MEDIUM → P3 1 HIGH, 6 MEDIUM → P4 0 HIGH, 4 MEDIUM →
    P5 0 HIGH, 3 MEDIUM → P6 0 HIGH, 5 MEDIUM (4 distincts) → **P7 0 HIGH, 1 MEDIUM**. Modèles : P1 Sonnet 4.6,
    P2 Opus, P3 Sonnet, P4 Opus, P5 Sonnet, P6 Opus, P7 Sonnet ; remédiations Opus 5.5. Le MEDIUM de la P7
    recycle le texte de l'AC6 (remédiation P6), sans règle métier ni module.
  - **Suite** : la **P8** (dernière passe du plafond de 8) sera une passe **ciblée** sur ce commit, la
    remédiation P7 ne touchant que la fiche et le registre (aucun code de production).
  - Décompte après passe : **9 AC, 7 tâches (T0–T6), 18 tests** (16 neufs, 2 modifiés : 9 et 13) — inchangé.

- 2026-10-08 — **Alignement sur le livré (T0)**, sur `origin/main` `9cb5083b` (15-5a, 15-5b, 15-5c, 15-5e1,
  15-5e2, 15-8a, 15-8b mergées ; **15-5d non mergée**, branche lue à `5624ca78`). Relevés au Dev Agent Record.
  Écarts (C-15-6a-1) :
  1. **15-5d non mergée — dépendance ferme : la story attend.** Le helper de verrou de liste
     (`lock_designated_accounts_in_tx`) et le test de C35 n'existent que sur sa branche. La fiche dit « si l'une
     n'est pas mergée au moment de T0, la story attend » (finding R6-10) et C-15-6-35 a écarté la branche « si la
     15-5d n'est pas mergée » : **développement suspendu après le T0**.
  2. **Helper déjà en partagé, sans mode** (C87 de la 15-5d) : le paramètre `AccountLockMode` de l'AC6 n'a plus
     d'objet — aucun appelant exclusif. L'AC6 prévoyait d'ajouter le mode « si T0 la trouve sans paramètre » pour
     que la 15-5d passe `Exclusive` ; elle passerait `Share` : la 15-6a **emploie le helper tel quel**, sans mode.
     Forme, non règle.
  3. **Type de ligne privé** (`LockedDesignatedAccount`, dans le newtype `DesignatedAccountsSnapshot`), helper
     `pub(in crate::repositories)` : cas prévu (AC6, C-15-6-36) — la 15-6a les rend **`pub`** sous ces noms (le
     test 17 l'appelle de `tests/`), sans second type.
  4. **`owned_account_ids` déjà adopté** (C88 de la 15-5d) : test 17 vert d'emblée (cas prévu par la fiche) ;
     plan épinglé `FORCE INDEX (PRIMARY)` — l'`EXPLAIN` de l'AC6 se relève sur ce plan.
  5. **Test 13 ailleurs** : le test de C35 est dans `invoices_validate_vat.rs`, pas dans
     `credit_notes_repository.rs` (qui reste à 9 tests) ; il se ré-ancre là.
  6. **Rejeu de la route par la 15-5e2** (et non « la 15-5e », découpée — C61) : constaté, rien à poser.
  7. **Pattern 5 par renvoi** (C68 de la 15-5e2) : la ligne de l'avoir (AC9) prendra la forme des autres flux
     comptables — renvoi au doc-comment « Ordre des locks » de `create_credit_note` —, non la séquence recopiée.
  8. **Deux écrivains neufs** (15-8a `update`, 15-8b `delete_in_tx`) : hors classe, et gardés par la pièce — les
     lignes de l'écriture de vente restent gelées.
  9. **Numéros de ligne** : ceux de `credit_notes.rs` ont peu bougé (`:362-364` créance, `:365-367` produit,
     `:429-440` ids de la 6 ter, `:442-446` filtre `active = TRUE`, `:449-498` refus, `:507-512` appel du
     générateur, `:525` arrondi, `:609-611` commentaire « inverse exact ») ; ailleurs : `invoices.rs` doc de
     `generate_invoice_journal_lines_rounded` `:1870-1884`, validation `:1971`, créance `:2041-2042`, arrondi
     `Issuance` `:2101`, exercice `:2205` ; copies de la requête de créance `invoice_settlements_write.rs:104-115`
     et `:445-456`, `reconciliation.rs:1475-1501` ; rapprochement : arrondi `:1537`, exercice `:1570`,
     `UPDATE invoices` `:1753` ; `fiscal_years.rs:550-555` ; `errors.rs` (kesh-db) `ArchivedAccount` `:403`,
     `ReversalAccountsArchived` `:846-856`, `RoundingContext::Issuance` `:972`, code `:1010`, `map_db_error`
     `:1048` ; `kesh-api/src/errors.rs` bras `ReversalAccountsArchived` `:3018-3050`, bras
     `CreditNoteRevenueAccountsArchived` `:3254`, son test `:3814` ; `accounts::archive` `:646` ;
     `archived_accounts_in_tx` `journal_entries.rs:2380` ; `test_fixtures.rs:560` (délai `:586`) ;
     `fr-CH/messages.ftl:304` (`credit-note-revenue-account-archived`), `:289` (`…-issuance`).
  **Aucun écart ne change une règle métier ni un AC sur le fond** (2, 3, 4, 5, 7 sont des cas que la fiche
  prévoyait ou des formes) ; le seul bloquant est le 1.
