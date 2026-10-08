# Story 15.5d : Les comptes de réglage contrôlés à l'usage — créance, TVA, créanciers, et le compte créanciers à l'écran

Status: done

<!-- Quatrième sous-story de la 15-5, créée le 2026-10-08 à la passe de validation P4 de la 15-5b
     (finding F4-3, choix C33 de `epic-15-choix-autonomes.md`). Elle reprend de la 15-5b l'ancien AC20
     (garde à l'usage des comptes de réglage, choix C27 et C28, révision de la limite L2 de D-A0) et tout
     ce qui s'y rattache — la variante `DesignatedAccountsNotPostable`, sa clé i18n, ses tests, ses
     passages de manuel —, plus la remédiation des deux HIGH et du MEDIUM que cette règle a fait naître
     en P4 : le compte créanciers exposé à l'écran (C34, révise C25) et l'avoir exempté avec sa vraie
     raison (C35). Choix applicables : C6, C16, C19, C25 (révisé par C34), C27, C28 (texte révisé par
     C36), C29, C33, C34 (révisé par C45), C35, C36, C39 (révisé par C42), C40, C41, C42, C43 (révisé
     par C48, puis par C87 : verrou partagé), C44, C45, C46, C47, C48, C49, C50, C51, C52, C53, C65,
     C85 (étiquette d'achat révisée par C87), C87, C88.
     Découpée à son tour après la validation P3 (choix C52, ligne de partage C53) : le réalignement
     des flux de règlement sur l'ordre canonique des verrous est sorti en 15-5e, qui passe avant ;
     la 15-5e a été réécrite depuis (choix C54) : elle rejoue sur interblocage toutes les routes qui
     écrivent au journal, et ne garde de l'ordre que l'avance des réglages de la saisie fournisseur ;
     puis découpée (choix C61) en 15-5e1 (socle, dont cette story dépend seule — C65) et 15-5e2.
     Historique complet au Change Log.
     Statut `ready-for-dev` : convention du registre pour une fiche en cours de validation (les 15-5b et
     15-5c portent le même) — il n'existe pas de statut « en validation » dans `sprint-status.yaml`. Le
     développement attend la clôture de la boucle de validation ET le merge des 15-5a, 15-5b et 15-5e1
     (finding R1-7 de la P1 ; C52 ; C65 : la 15-5e a été découpée en 15-5e1 et 15-5e2, cette story ne
     dépend que de la première). **Les trois sont mergées** (15-5e1 : PR #559, `de1e1c26`) : restent la
     validation et le **merge de la 15-8b** (PR #560), qui touche des fichiers de la garde — cette
     story se développe après lui (finding R5-1 de la P5). **Numéros de ligne relevés sur `cecd5d1d`** (qui intègre
     `origin/main` `de1e1c26` : 15-5c, 15-5e1, 15-8a), par leur texte, à l'alignement sur le livré
     (C85), sauf mention contraire ; T0 les refait. -->

**Issues** : **ferme #429** (P1 — les comptes désignés dans les réglages de facturation ne sont
re-validés nulle part). La PR porte `closes #429` (mot-clé **dans la PR**, le dépôt merge en squash).
La 15-5b, qui garde ces comptes **à la désignation**, ne fait que `refs #429` : la désignation seule
laisse un réglage devenu non imputable recevoir des écritures (finding P3 F-1 de la 15-5b), et c'est
l'usage — cette story — qui ferme le défaut. Voisines citées, **non traitées ici** : **#473** et
**#523** (l'avoir relit la créance et le compte d'arrondi dans les réglages du moment — 15-6a),
**#525** (l'avoir contre-passe la TVA sur la TVA due des réglages du moment — report TVA).

**Dépend de la 15-5a** (le newtype `NonPostableAccounts`, son constructeur `details()` — C29 —, le
code `ACCOUNT_NOT_POSTABLE`) **et de la 15-5b** (l'AC19 : `defaultPayableAccountId` absent du corps
est préservé, `Option<Option<i64>>` ; l'AC10 : les six champs refusent un compte non imputable à la
désignation ; la parenthèse « de regroupement, de résultat ou de clôture » de `validate_account_of`).
**Et de la 15-5e1 — dépendance satisfaite** (`15-5e1-socle-rejeu.md`, mergée par la PR #559,
`de1e1c26` ; choix C52, réécrite selon C54, découpée par C61 — « Socle du rejeu sur interblocage »),
**et pas de la 15-5e2** (choix C65). Livré et relu sur `cecd5d1d` : le **rejeu de la validation
d'une facture** (`"invoices::validate"`) **et de la saisie d'une facture fournisseur**
(`"supplier_invoices::create"`), tous deux par l'enveloppe
`kesh_db::retry::retry_on_deadlock(operation, f)` (`routes/invoices.rs:880`,
`routes/supplier_invoices.rs:363`) ; l'avance des réglages de la saisie fournisseur avant les comptes
de charge (étape `(2 bis)` de `supplier_invoices::create_in_tx`) ; les doc-comments canoniques
« # Ordre des locks » de `validate_invoice` (`crates/kesh-db/src/repositories/invoices.rs:1916-1957`,
`# Erreurs` à `:1959` ; étiquettes `(1)`, `(1 bis)`, `(2)`, `(2 bis')`, `(2 quater)`, `(3)`, `(4)-(5)`,
`(7)` ; le code porte aussi `(2 bis)` (`:2043`, pièce entièrement à zéro), `(2 bis'')` et `(2 ter)`,
sans verrou) et « # Ordre des verrous » de
`supplier_invoices::create_in_tx` (`repositories/supplier_invoices.rs:249-296`, étiquettes `(0)`,
`(1)`, `(2)`, `(2 bis)`, `(2, suite)`, `(3)`, `(4)`, `(5)`) ; le commentaire « 5 bis »
(`invoice_settlements_write.rs:474-486`). Cette story **suppose ce rejeu et cette avance en place** et
y place son verrou ; elle n'en redécrit que ce qui la concerne et **renvoie à la 15-5e1** pour le
reste.
Le rejeu de la **complétion d'une facture importée** vient avec la **15-5e2**
(`15-5e2-rejeu-des-autres-flux.md`), dans un ordre de merge libre : si cette story merge avant elle,
cette route porte le verrou de l'accesseur sans être encore rejouée, et une victime d'interblocage y
rend `500` comme aujourd'hui jusqu'au merge de la 15-5e2 — une fenêtre de même nature que l'état
actuel, pas une régression de nature. Pour l'**ordre des verrous**, l'ordre de merge est indifférent :
le Pattern 5 de la 15-5e2 **renvoie** aux doc-comments canoniques que cette story met à jour (AC9),
il ne recopie pas l'ordre (choix C68). **Il ne l'est pas pour les fichiers que les deux stories
touchent** (finding R5-6 de la P5 ; `15-5e2-rejeu-des-autres-flux.md`, finding R5-1 de sa P5) : si la
15-5e2 merge avant, cette story se rebase dessus, résout `docs/manual/fr/user-manual.tex`,
`CHANGELOG.md` et le commentaire « 5 bis » (AC9) en gardant les deux côtés, et **régénère** les deux
PDF après le rebase (conflit binaire : aucun des deux PDF n'est à reprendre tel quel).
La 15-5e1 est mergée : ne retiennent plus le développement que la validation et le merge de la 15-8b (ci-dessous). **La 15-5c est
mergée** et **rien de ce qu'elle a livré n'est dans cette story** (vérifié à l'alignement, C85) : ses
libellés traduits des codes de `failed[]` (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts`,
dont la branche `ACCOUNT_NOT_POSTABLE`) ne servent qu'au rapprochement, que la garde de l'AC1 ne
touche pas (elle n'est posée qu'à la validation d'une facture et à la saisie fournisseur), et les
trois écrans de l'AC4 n'emploient pas ce module ; ses passages du manuel (rapprochement) sont
distincts de ceux de l'AC8. **La borne `sitesTotal`** de `frontend/src/lib/shared/i18n-keys.test.ts`
a bougé avec les 15-5c, 15-8a et 15-8b (**1904** sur `cecd5d1d` ; la 15-8b la relève encore) : elle se
**recompte sur l'état rebasé**, au moment du développement, jamais depuis un nombre de cette fiche.
**La 15-8b** (PR #560, branche `story/15-8b-supprimer-une-ecriture`, non mergée) **touche des
fichiers de la garde** (finding R5-1 de la P5, diff de rebase relu sur `013b67de`) :
`crates/kesh-db/src/repositories/invoices.rs` (**+5 lignes dans `unvalidate`, avant
`validate_invoice`** : tous les numéros de la vente cités ici glissent), `crates/kesh-db/src/errors.rs`
et `crates/kesh-api/src/errors.rs` (là où vont la variante et son bras), les **quatre**
`messages.ftl` (là où vont les deux clés neuves), `frontend/src/lib/shared/i18n-keys.test.ts` (la
borne de T3), `docs/api-external.md` et `CHANGELOG.md` (AC9), et les deux manuels avec leurs PDF.
**Cette story se développe sur `main` APRÈS le merge de la 15-8b** (ou se rebase dessus si elle est
déjà commencée) : T0 relocalise **par le texte** — jamais par les numéros de cette fiche —, les
conflits textuels se résolvent en gardant les deux côtés, et les PDF se régénèrent après le rebase.

## Story

En tant que **comptable ou administrateur d'une société tenue dans Kesh**,
je veux que **la validation d'une facture et la saisie d'une facture fournisseur refusent d'écrire
sur un compte de réglage devenu non imputable, avec un message qui dit où le corriger — et que cet
endroit existe à l'écran pour chacun de ces comptes**,
afin qu'**aucune écriture automatique n'atterrisse sur un compte de regroupement, de résultat ou de
clôture**, sans pour autant bloquer un flux sans recours dans l'application.

### Pourquoi une story à part

La garde à l'usage est une **règle métier neuve** — la révision de la limite **L2** de la décision
D-A0 (14-3b) pour quatre comptes —, pas le report d'un patron : elle a sa variante d'erreur, sa clé
i18n à sélecteur, deux flux de `kesh-db` sans rapport avec le rapprochement, deux écrans à relire et
deux manuels. Elle avait été ajoutée à la 15-5b par la remédiation de sa validation P3 ; la passe P4 a
trouvé **deux HIGH et un MEDIUM nés d'elle** (le refus des créanciers renvoyait à un champ absent de
l'écran ; l'avoir écrivait sur deux de ces comptes sans garde et la fiche le disait « snapshot de la
facture »), et a relevé qu'elle n'avait rien d'un rollout (finding F4-3). Rien du reste de la 15-5b
n'en dépend. D'où le découpage (choix C33).

## Acceptance Criteria

### La garde à l'usage (#429)

1. **AC1 — Un compte de réglage devenu non imputable est refusé quand un flux veut y écrire**
   (choix C27). L'exemption « inchangé » de l'AC11 de la 15-5b laisse en place un réglage devenu non
   imputable (sous-comptes créés — règle 14-3a —, case *imputable* décochée à l'écran, ou rôle
   « résultat de l'exercice » attribué : `effective_postable`, `accounts.rs:126-131` — finding F5-5) ;
   les flux #8 et #10 de l'inventaire (a) de la 15-5b
   y écrivent avec `enforce_postable = false`. Sur le patron du compte d'arrondi —
   `rounding_account_for_write` / `usable_designated_account`
   (`crates/kesh-db/src/repositories/company_invoice_settings.rs:400`, `:484-510`, requête `:499`),
   relu **au moment d'écrire**, dans la transaction, **sous verrou** — mais un verrou **partagé**
   (`LOCK IN SHARE MODE`), et non le `FOR UPDATE` du compte d'arrondi (choix **C87**, finding F5-1 de
   la P5 ; voir « L'ordre des verrous » ci-dessous) — :
   - **le mécanisme du prédicat — les rôles rendus par le générateur** (findings R1-2 / F2 de la P1,
     choix **C39**). Le générateur de lignes **expose l'ensemble des rôles de réglage qu'il a
     effectivement écrits**, et la garde ne contrôle **que les comptes de ces rôles** : aucun recalcul
     de la TVA hors du générateur (règle DRY), aucune inspection des `account_id` de `entry_lines`
     (ambiguë dès qu'un même compte joue deux rôles, ou qu'un compte de réglage coïncide avec un compte
     de produit ou de charge d'une ligne — les tests le font : `supplier_invoices_repository.rs:49-55`
     désigne `2000` comme compte créanciers alors que `seed_accounting_company` l'a déjà désigné comme
     TVA due, finding R2-7). Concrètement : un type `DesignatedRole` (`Receivable`, `VatPayable`,
     `Payable`, `VatRecoverable`) et un ensemble ordonné de rôles (`BTreeSet<DesignatedRole>` ou
     newtype équivalent) rendus **avec** les lignes, dans une structure dont la forme est **fixée**
     (finding F3-6 de la P3, choix **C50**) :
     `pub(in crate::repositories) struct GeneratedLines { pub lines: Vec<NewJournalEntryLine>, pub roles: BTreeSet<DesignatedRole> }`,
     avec **`#[derive(Debug)]`** — les tests appellent `unwrap_err()` sur le résultat
     (`supplier_invoices.rs:1432`, `invoices.rs:3123`), qui exige `T: Debug` — ; `DesignatedRole`
     dérive `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord` ; les deux types vivent dans
     `crates/kesh-db/src/repositories/company_invoice_settings.rs`, à côté de l'accesseur, en
     `pub(in crate::repositories)` — visibles d'`invoices`, de `supplier_invoices` et de l'accesseur,
     même visibilité que les générateurs de vente (finding F5-7 de la P5) ;
     le générateur de vente (et sa variante `_rounded`) rend `Result<GeneratedLines, DbError>`, celui
     d'achat `Result<(GeneratedLines, Decimal), DbError>` (le total TTC reste le second membre du
     tuple) ; les **19 tests** sont adaptés **mécaniquement** — ils lisent le champ `.lines` (p. ex.
     `generate_…(…).unwrap().lines`, ou `let (generated, ttc) = …; generated.lines` côté achat), sans
     autre changement de leurs assertions. `DesignatedRole` est un type **neuf** plutôt qu'`AccountRole`
     (`crates/kesh-db/src/entities/account.rs:90-102`), délibérément (finding F2-4, choix **C44**) :
     l'ensemble désigne des **champs des réglages** effectivement écrits, non le rôle que le plan
     attribue au compte (un compte désigné ne porte pas forcément ce rôle) ; ses **quatre** variantes
     se traduisent en champs par un `match` exhaustif, sans les neuf autres rôles d'`AccountRole` à
     écarter par un bras mort ; et `AccountRole` ne dérive pas `Ord`, qu'exige un `BTreeSet` —
     - côté vente, par `generate_invoice_journal_lines` (`invoices.rs:1784`) : `Receivable` toujours,
       `VatPayable` **dans la branche même** `if total_vat > Decimal::ZERO` (`:1851`) qui écrit les
       lignes de TVA due ; `generate_invoice_journal_lines_rounded` (`:1879`) le transmet ;
     - côté achat, par `generate_purchase_journal_lines` (`supplier_invoices.rs:105`, qui rend déjà un
       tuple `(lignes, total TTC)`) : `Payable` toujours, `VatRecoverable` dans la branche
       `if total_vat > Decimal::ZERO` (`:129`) ;
     - l'appelant traduit chaque rôle en l'identifiant qu'il a lui-même passé au générateur (créance,
       TVA due, créanciers, TVA récupérable des réglages) et appelle l'accesseur sur ces identifiants.
       Ventilation des 25 occurrences de
       `grep -rnE "generate_(invoice_journal_lines(_rounded)?|purchase_journal_lines)\(" crates | grep -v "///"`
       sur `ac1719b9`, **recompté sur `cecd5d1d` : 25, même ventilation** (findings R2-2 / F2-3) :
       **3 définitions** (`invoices.rs:1784`, `:1879`, `supplier_invoices.rs:105`), **1 transmission**
       (`invoices.rs:1886`, `_rounded` → générateur), **2 appels de production** (`invoices.rs:2275`,
       `supplier_invoices.rs:439`) — ceux que la garde
       adapte — et **19 tests** (14 dans `invoices.rs`, 4 dans `supplier_invoices.rs`, 1 dans
       `credit_notes.rs:933`, test de 16-1a qui compare la facture et sa contre-passation compte par
       compte). **L'avoir n'appelle pas ces générateurs** : il a le sien,
       `generate_credit_note_journal_lines` (`credit_notes.rs:187`, appelé à `:507-512`), que la story
       ne touche pas — c'est la forme exacte de son exemption (C35, AC3 ; révision de C39 consignée en
       **C42**) ;
   - un accesseur neuf de `company_invoice_settings`, **en deux temps** (finding F2-1 = R2-1, choix
     **C43** — voir « L'ordre des verrous » ci-dessous) :
     - **verrouiller** (p. ex. `lock_designated_accounts_in_tx(conn, company_id, ids: &[i64])`) :
       lit `id, number, active, postable` de **tous les comptes candidats du flux** — ceux que les
       réglages désignent pour les rôles que le flux **peut** écrire, qu'il les écrive ou non —, en
       **une seule requête** `WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`
       (syntaxe MariaDB 10.11 : **pas** `FOR SHARE`) — l'**ordre de verrouillage par identifiant** est
       fixé (finding R1-6) — et rend cet instantané, **sans refuser**. **Partagé, et pourquoi il
       suffit** (C87) : le but du verrou est qu'aucun archivage ni passage à non imputable ne s'insère
       entre le contrôle et l'insertion ; or ces gestes **écrivent** la ligne du compte —
       `accounts::update` (case *imputable*, rôle, retypage) et `accounts::archive` par un `UPDATE
       accounts … WHERE id = ?`, la création d'un sous-compte par `UPDATE accounts SET postable =
       FALSE WHERE id = ? AND company_id = ?` sur le parent (`accounts.rs:194`, `:558`, `:669`) —, et
       tout `UPDATE` prend un verrou **exclusif** de ligne, qui attend la fin d'une transaction tenant
       un verrou partagé. ⚠️ Ces routes ne verrouillent pas la ligne **avant** leur propre contrôle
       (lecture simple de l'instantané, `accounts.rs:466-470`, `:654-658`) : c'est l'`UPDATE` lui-même
       qui pose l'exclusif, et c'est lui qui attend ; ce que le verrou partagé garantit est donc
       exactement « aucune écriture de la ligne du compte entre le verrou de l'accesseur et le
       commit », ni avant (la lecture verrouillante lit la dernière version validée, et attend un
       `UPDATE` non encore validé). Un `FOR UPDATE` n'ajouterait rien à cette garantie, et il **entrait
       en conflit** avec les verrous partagés que la clé étrangère `fk_jel_account` pose sur ces mêmes
       comptes dans les flux qui prennent l'exercice **avant** (cycle de F5-1, Dev Notes) ;
       un identifiant `None` des réglages n'y entre pas ;
       **Un identifiant d'une autre société ne doit
       pas être verrouillé** (finding F3-5 de la P3, choix **C51**) : le dépôt a mesuré qu'une
       lecture verrouillante par clé primaire (`FOR UPDATE`, mesuré ; même mécanisme pour un verrou
       partagé) verrouille la ligne **avant** que le filtre `company_id` ne l'écarte (`opening_complement.rs:429-437`, revue de code P1 de la story du complément
       d'ouverture, B-F1). L'accesseur **adopte donc d'emblée** le patron `owned_account_ids`
       d'`opening_complement.rs:469-487` (finding R6-5 de la P6, choix **C88**, qui tranche ce que
       C51 laissait au résultat du test) : d'abord une lecture **non verrouillante** `SELECT id FROM
       accounts WHERE company_id = ? AND id IN (…)` — un compte ne change jamais de société, la
       lecture est exacte sans verrou —, puis la lecture verrouillante partagée sur ces **seuls**
       identifiants, le filtre `company_id` gardé en défense. ⚠️ Le patron d'origine lit sur le pool
       (hors transaction) ; ici la première lecture passe par la connexion de la transaction, où elle
       peut ouvrir la vue REPEATABLE READ — sans effet sur la garde, dont le contrôle porte sur les
       lignes **verrouillées**, lues fraîches. Un identifiant écarté par cette première lecture est
       **absent de l'instantané**, et le contrôle le refuse comme tel (`InactiveOrInvalidAccounts`,
       ci-dessous). Le test de l'AC7 reste, et passe par construction. Le **plan d'exécution** de la
       requête verrouillante (`EXPLAIN`, accès par clé primaire sur `id IN (…)`, aucun `filesort` qui parcourrait
       — donc verrouillerait — d'autres lignes : piège mesuré à `opening_complement.rs:274-288`) est
       relevé au Dev Agent Record. **Verrous d'intervalle** (finding F6-4 de la P6) : en REPEATABLE
       READ, une lecture verrouillante qu'InnoDB parcourt **par plage** pose des verrous *next-key* —
       la ligne étrangère reste libre, l'intervalle qui la jouxte non ; une recherche d'égalité sur la
       clé primaire, elle, ne verrouille que la ligne trouvée. Le doc-comment de l'accesseur le dit en
       une phrase, avec le **type d'accès** relevé (`const`/`eq_ref` contre `range` sur `PRIMARY`) et le
       renvoi au précédent du dépôt, qui a écarté `LOCK IN SHARE MODE` pour cette raison
       (`journal_entries.rs:1176`, C-15-8-23). Impact faible ici : aucune insertion dans `accounts`
       ne tombe au milieu de la table, hors restauration à identifiants explicites ;
     - **contrôler**, après la génération : sur l'instantané (les lignes sont verrouillées jusqu'au
       commit, il n'y a rien à relire), **seuls les comptes des rôles rendus par le générateur** (C39,
       inchangé). L'accesseur **refuse lui-même** (finding F3-4 de la P3, choix **C49**), dans cet
       ordre, écrit à son doc-comment :
       1. un compte d'un rôle écrit **absent de l'instantané** (inexistant, ou d'une autre société)
          **ou inactif** (archivé) → `InactiveOrInvalidAccounts`, le refus que `create_in_tx` rend
          aujourd'hui pour ce cas — même variante, même code, aucun contrat neuf ; la 15-5a interdit
          de nommer un tel compte, il n'est donc pas nommé ;
       2. sinon, les comptes **de la société, actifs et non imputables** →
          `DesignatedAccountsNotPostable`.
       Pourquoi l'accesseur et non `create_in_tx` : le contrôle de `create_in_tx`
       (`journal_entries.rs:117-121`) est une lecture **non verrouillante**, qui lit sous REPEATABLE
       READ l'instantané ouvert par la première lecture simple de la transaction — antérieure au
       verrou ; un compte archivé entre les deux y paraîtrait encore actif et l'écriture passerait.
       L'accesseur, lui, tient la ligne **fraîche** sous verrou (une lecture verrouillante, partagée
       comme exclusive, lit la dernière version validée, et non l'instantané) (même raison que le filtre
       `active = TRUE` de `rounding_account_for_write`, `company_invoice_settings.rs:497-499`, et que
       la mise en garde d'`opening_complement.rs:429-437`). Le contrôle de `create_in_tx` reste en
       place, inchangé, pour les autres comptes de l'écriture ;
   - **un même compte désigné pour deux rôles est nommé une fois** (finding F1 de la P1, choix
     **C40**). Le dédoublonnage n'est **pas** réécrit dans l'accesseur : il est garanti par
     `NonPostableAccounts::new`, que la 15-5a définit comme trieur **et dédoublonneur par
     identifiant** (`15-5a-refus-non-imputable.md`, AC1, l. 48-53 : « qui **trie par numéro** […]
     **puis par identifiant** à numéro égal, et **dédoublonne par identifiant** » ; l. 62-63 : « la
     variante ne peut être construite qu'avec une liste triée, dédoublonnée et non vide »). L'accesseur
     construit la variante par ce seul constructeur ; `count` vaut donc le nombre de comptes
     **distincts**, et le message prend le singulier quand deux rôles pointent sur le même compte.
     Un test le fige (AC7) ;
   - **validation d'une facture** (`crates/kesh-db/src/repositories/invoices.rs`, `validate_invoice`) :
     - **le verrou** : les candidats sont la **créance** (`default_receivable_account_id`, déjà exigée
       à l'étape (2), `:2036-2038`) et la **TVA due** (`default_vat_payable_account_id`, si désignée) ;
       ils sont verrouillés **après** le compte d'arrondi (`rounding_account_for_write`, `:2093`) et
       **avant** l'exercice (`fiscal_years::find_open_covering_date`, étape (3), `:2200`). Place exacte
       dans le code livré par la 15-5e1 (C85) : **juste après le bloc `let rounding = …`** qui clôt
       l'étape **(2 bis')** — donc après (2 bis'') (montant minimum, sans verrou), et **avant**
       (2 ter) (re-validation des comptes de produit, sans verrou), (2 quater) (verrou **partagé** sur
       le compte de produit par la clé étrangère d'`invoice_lines`, conditionnel) et (3). Les
       comptes désignés (actif, passif) et les comptes de produit (`Revenue`) sont disjoints par
       type : leur ordre relatif n'ouvre aucun cycle entre eux. Le doc-comment canonique le dit par
       une ligne **`(2 bis', suite)`** sous `(2 bis')` (AC9), et le code porte la même étiquette en
       commentaire — une étiquette du doc-comment doit exister au code (leçon du finding A1 de la
       revue de code de la 15-5e1, qui a dû retirer un `(2 ter)` inexistant côté achat) ;
     - **le contrôle** : **entre la génération des lignes** (`generate_invoice_journal_lines_rounded`,
       étape (7), `:2275-2281`) **et `create_in_tx`** (`:2283`) — finding R4-3/F4-5 —, sur les rôles rendus par le
       générateur : la **créance** toujours ; la **TVA due** **seulement si le générateur l'a écrite**,
       c'est-à-dire si `total_vat > 0` (`invoices.rs:1849-1852`, calculé sur les montants **arrondis
       par ligne**) — **pas** « une ligne à taux > 0 » : une facture à taux positif dont la TVA arrondit
       à zéro n'écrit rien sur la TVA due et n'est pas bloquée (finding F4-4 ; cas existant
       `gen_lines_rate_rounds_to_zero`), même si son compte a été verrouillé ;
     - **l'ordre des refus** (réécrit en P2, **écrit dans le doc-comment « # Erreurs » de
       `validate_invoice`**) : le verrou ne refuse rien, il ne déplace donc aucun refus ; le refus de
       cette story vient **après** tous les refus antérieurs — total nul, montant minimum, compte
       d'arrondi, comptes de produit des lignes (2 ter : `INVOICE_LINE_REVENUE_ACCOUNT_INVALID`,
       variante `InvalidRevenueAccounts`, raison `NotPostable` pour un compte non imputable — Story
       16-1a, `invoices.rs:640`, `:665`, `errors.rs:1021` ; **pas** `ACCOUNT_NOT_POSTABLE`, finding
       F5-2 de la P5),
       **exercice** (`FiscalYearInvalid`, bien que le verrou des comptes le précède désormais) — et
       après le `ConfigurationRequired("default_vat_payable_account_id")` du générateur (TVA due
       **absente**) ; il précède `create_in_tx`. « Le refus » désigne ici les deux refus de
       l'accesseur, dans leur ordre (C49 : compte absent ou inactif, puis non imputable) ;
   - **création d'une facture fournisseur** (`crates/kesh-db/src/repositories/supplier_invoices.rs`,
     `create_in_tx` — donc aussi la **complétion d'une facture importée**, qui l'appelle,
     `routes/imported_supplier_invoices.rs:243`) :
     - **les réglages** sont chargés après le projet et le fournisseur, **avant** les comptes de charge
       — étape `(2 bis)`, `supplier_invoices.rs:369-375`, avance posée par la **15-5e1** (son AC5),
       livrée ;
       l'exigence `ConfigurationRequired("default_payable_account_id")` reste à sa place (après
       l'exercice) : la priorité des refus ne bouge pas ;
     - **le verrou** : les candidats, **créanciers** (`default_payable_account_id`, valeur résolue
       par l'AC19 de la 15-5b) et **TVA récupérable** (`default_vat_recoverable_account_id`), lus dans
       les réglages de l'étape `(2 bis)`, sont verrouillés **après** les comptes de charge et **avant**
       l'exercice — **entre `(2, suite)`** (passe des comptes de charge, `:377-426`) **et `(3)`**
       (`find_open_covering_date`, `:427-430`). Le doc-comment canonique gagne une ligne à cette
       place, sous une étiquette **neuve, présente aussi en commentaire au code** — **`(2, désignés)`**
       (choix **C87**, qui révise C85 ; finding R5-4 de la P5) : **pas** `(2 ter)`, que la revue de code
       de la 15-5e1 a retirée de cette fonction (finding A1) et que le doc-comment de `validate_invoice`
       — auquel celui-ci renvoie — emploie pour une étape **sans** verrou ; l'exigence `(4)` (`ConfigurationRequired("default_payable_account_id")`, `:432-436`)
       ne bouge pas : un compte créanciers `None` n'est simplement pas candidat ;
     - **le contrôle** : **entre** `generate_purchase_journal_lines` (étape `(5)`, `:439-445`) **et**
       `journal_entries::create_in_tx` (`:446`), sur les rôles rendus : les **créanciers** toujours ;
       la **TVA récupérable** **seulement si le générateur l'a écrite** (TVA totale positive,
       `:129-131`). Le compte de **charge** non imputable est refusé plus tôt, par la 15-5a
       (passe des comptes `(2, suite)`, `supplier_invoices.rs:377-426`) : son refus passe avant ;
       l'exercice aussi ;
   - **l'ordre et le mode des verrous** (finding **F2-1 HIGH = R2-1**, choix **C43** ; mode révisé par
     **C87**, finding **F5-1 HIGH** de la P5 ; règle posée par la **15-5e1**, choix C54). La règle de
     la 15-5e1 est écrite au doc-comment de `validate_invoice` (`invoices.rs:1916-1957`, § « La
     règle ») : un ordre conventionnel des verrous **n'exclut pas** les interblocages, et **la défense
     est le rejeu des routes**. Cette story y place **son seul verrou neuf**, celui de l'accesseur,
     **partagé** : à la validation, **après** le compte d'arrondi et **avant** l'exercice ; à la saisie
     fournisseur, **après** les comptes de charge et **avant** l'exercice. Elle **n'affirme aucune
     baisse de fréquence** : elle dit, cycle par cycle, ceux que sa place et son mode **ne forment
     pas** et ceux qui **restent** (Dev Notes, « Les cycles examinés »).
     **Pourquoi avant l'exercice** : une garde qui verrouillerait la TVA due **après** l'exercice
     croiserait **à coup sûr** le solde du reste, qui la verrouille en exclusif avant (T1 valide la
     facture B : tient l'exercice, attend la TVA due — un partagé attend un exclusif ; T2 solde la
     facture A avec TVA : tient la TVA due, attend l'exercice), et, côté achat, un règlement
     fournisseur par compte interne égal au compte créanciers (compte, puis exercice : `pay_in_tx`,
     compte interne `FOR UPDATE` `supplier_invoices.rs:712-714`, exercice `:745`).
     **Pourquoi partagé** : les flux les plus fréquents — règlement client par virement, solde du
     reste, rapprochement, avoir, règlement fournisseur — prennent l'exercice **puis** reprennent la
     créance, la TVA due ou les créanciers par la clé étrangère `fk_jel_account`, en verrou
     **partagé**, à l'insertion de leurs lignes. Un verrou **exclusif** de l'accesseur, pris avant
     l'exercice, formait avec chacun d'eux un cycle **systématique** dès que les deux se chevauchent
     (la validation tient la créance en exclusif et attend l'exercice ; le règlement tient l'exercice
     et demande la créance en partagé) — c'est F5-1, introduit par C43. Deux verrous partagés étant
     compatibles, ce cycle ne se forme plus.
     **Pourquoi après l'arrondi** : la validation prend déjà l'arrondi en premier parmi ses comptes,
     et le solde du reste prend l'arrondi avant la TVA due (`invoice_settlements_write.rs:474-505`) :
     l'accesseur suit le même ordre.
     Un interblocage **restant** (Dev Notes) est **rejoué** par les routes (15-5e1 — livré — pour la
     validation et la saisie fournisseur, 15-5e2 pour la complétion d'import). D'où les deux temps de
     l'accesseur : **verrouiller tôt tous les candidats, contrôler tard les seuls rôles écrits**.
     Verrouiller un candidat que le générateur n'écrira pas (TVA due d'une facture sans TVA) coûte un
     verrou partagé superflu, jamais un refus ;
   - un seul refus nomme **tous les comptes de réglage** en défaut du flux (pas ceux des lignes, refusés
     plus tôt) : variante neuve `DbError::DesignatedAccountsNotPostable(NonPostableAccounts)`
     (`crates/kesh-db/src/errors.rs`), même code **`ACCOUNT_NOT_POSTABLE`**, HTTP 400, `details` =
     `NonPostableAccounts::details()` (C29) — un seul contrat pour l'intégrateur (choix C28).
2. **AC2 — Le message dit où agir, et qui peut le faire** (choix C28, texte révisé par C36). Clé neuve
   `error-designated-account-not-postable`, quatre locales, sélecteur `[one]` / `*[other]` inscrit à
   `SELECTEURS_RESOLUS_COTE_SERVEUR` (`crates/kesh-i18n/src/loader.rs:373`, patron de l'AC2 de la
   15-5a), `count` passé comme nombre ; texte FR **exact** :
   ```ftl
   error-designated-account-not-postable = { $count ->
       [one] Le compte { $numbers }, désigné dans Paramètres → Facturation, n’est pas imputable (compte de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à sa place un compte imputable.
      *[other] Les comptes { $numbers }, désignés dans Paramètres → Facturation, ne sont pas imputables (comptes de regroupement, de résultat ou de clôture) : un administrateur doit y désigner à leur place des comptes imputables.
   }
   ```
   - « un compte imputable » et non « un sous-compte » : le remède « sous-compte » ne vaut que pour un
     compte de regroupement, non pour un compte de résultat ou de clôture (finding R4-5) ;
   - « un administrateur doit » : la validation d'une facture et la saisie fournisseur sont ouvertes au
     Comptable, la page des réglages est réservée à l'Admin (`crates/kesh-api/src/lib.rs:207-211`) — le
     message vaut pour les deux rôles sans branche d'écran (finding R4-5/F4-6, choix C36) ;
   - DE/IT/EN sur le vocabulaire de la 15-5a (« bebuchbar », « registrabile », « postable ») et le
     libellé du menu tel que chaque locale l'affiche ; test Rust par locale, au singulier et au pluriel.
3. **AC3 — Ce qui ne change pas, et c'est écrit** (doc-comment de l'accesseur et Dev Notes) :
   - `create_in_tx` garde `enforce_postable = false` — la garde est en amont, sur les seuls comptes de
     réglage, et le **compte bancaire** (compte de configuration lui aussi) reste utilisable devenu non
     imputable (D-A0, AC15 de la 15-5b, C6) ;
   - le **compte de produit par défaut** reste exempté (D3-bis de la 16-1a, `invoices.rs:574-579`) ;
   - le compte de **décompte TVA** n'est lu par aucun flux d'écriture : seul l'export CSV le lit
     (`crates/kesh-api/src/exports/csv_tables.rs:932`) ; le rapport TVA **ne le lit pas** — il lit la
     TVA due et la TVA récupérable, sans écrire (`crates/kesh-report/src/vat_report.rs:173-174`)
     (finding F3 de la P1) ;
   - **le type n'est pas contrôlé à l'usage** — angle mort assumé, préexistant et hors du défaut de
     #429 (finding F5-8 de la P5) : l'accesseur contrôle présence, `active` et `postable`, et
     `create_in_tx` ne revoit pas le type ; un compte désigné **retypé** après sa désignation
     (`accounts::update` le permet sur confirmation, `confirm_retype`, `accounts.rs:540-555`) — une
     créance devenue `Expense`, par exemple — reste utilisé. Le compte d'arrondi, lui, contrôle son
     type (`usable_designated_account`, `company_invoice_settings.rs:498-503`, `account_type IN (?, ?)` à `:499`) ; l'aligner relève d'une
     autre story, que le dev **signale** au Dev Agent Record sans la traiter ;
   - le **solde du reste** garde son refus `ConfigurationRequired` pour une TVA due inutilisable
     (`vat_payable_account_for_write`, `company_invoice_settings.rs:454`, inchangé) — divergence de code assumée
     et écrite (C28) ;
   - **l'avoir est exempté de la garde à l'usage, délibérément** (choix C35, findings R4-2/F4-2). Fait
     vérifié : l'avoir ne reprend de la facture que ses **comptes de produit** (snapshot des lignes,
     D5-bis, `crates/kesh-db/src/repositories/credit_notes.rs:501-505`) ; la **créance** et la **TVA
     due** sont **relues dans les réglages du moment** (`:362-364`, `:507-512`), puis postées par
     `create_in_tx(…, false)` (`:548`). Il n'est pas gardé ici parce que ces deux lectures sont
     elles-mêmes **le défaut** à corriger, et qu'une garde posée dessus serait défaite par leur
     correction : la **créance** sera lue **sur l'écriture de vente** par la 15-6a (#473, et #523 pour
     le compte d'arrondi), et la **TVA due** relève de #525 (report TVA). Garder aujourd'hui le compte
     des réglages bloquerait l'annulation d'une facture sur un compte que l'avoir **ne devrait pas
     lire** — l'argument de l'AC15 de la 15-5b (une pièce émise doit rester annulable) s'y ajoute.
4. **AC4 — Les écrans affichent le refus.** Trois `catch`, et trois seulement (findings R1-1 / F3 de
   la P1) :
   - **validation d'une facture** : `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `catch` de
     `confirmValidate`, `:401-416` — `validateError = err.message` pour un `ApiError`, avec une branche
     propre à `CONFIGURATION_REQUIRED` (`:409-411`, suffixe « Configurez les comptes par défaut ») que
     `ACCOUNT_NOT_POSTABLE` **ne doit pas** emprunter : son message dit déjà où agir. **Le dialogue se
     ferme** sur `ACCOUNT_NOT_POSTABLE` (finding F2-6, choix **C46**) : le code est ajouté à la liste
     des « erreurs non-retryables » (`:428-433`, aujourd'hui `FISCAL_YEAR_INVALID` et
     `CONFIGURATION_REQUIRED`) — à la validation, `ACCOUNT_NOT_POSTABLE` ne peut venir **que** de la
     garde de cette story (`create_in_tx(…, false)` n'en émet pas, et le compte de produit d'une
     ligne non imputable rend `INVOICE_LINE_REVENUE_ACCOUNT_INVALID`, finding F5-2) : réessayer depuis
     le dialogue rend le même refus ; le message reste affiché par `notifyError`. Le dev **signale**
     au Dev Agent Record, sans le traiter, que `INVOICE_LINE_REVENUE_ACCOUNT_INVALID` ne ferme pas non
     plus le dialogue ;
   - **saisie d'une facture fournisseur** : `frontend/src/routes/(app)/supplier-invoices/+page.svelte:213-214`
     (`catch` à `:213`, `if (isApiError(err)) formError = err.message;` à `:214`). Les `catch` de `:103` (scan du QR,
     `supplier-invoices-scan-failed`) et de `:133` (chargement de la page) **ne sont pas** concernés :
     aucun des deux n'atteint `create_in_tx` ;
   - **complétion d'un import** : `frontend/src/routes/(app)/supplier-invoices/import/+page.svelte:261`
     (`formError = completeErrorLabel(err)`), dont le `switch` replie sur `err.message` par son
     `default:` (`:300`) — `ACCOUNT_NOT_POSTABLE` ne doit **pas** y recevoir de branche qui en
     changerait le texte.
   Le dev vérifie chacun à la lecture, le consigne au Dev Agent Record, et corrige ici un écran qui
   afficherait un repli générique. Aucune branche par rôle n'est ajoutée : le message de l'AC2 vaut pour
   les deux. **Un test d'écran le fige** (AC7, finding R1-3 / F4 de la P1, choix **C41**).

### Le compte créanciers, à l'écran (finding R4-1/F4-1)

5. **AC5 — `defaultPayableAccountId` s'affiche et s'enregistre dans *Paramètres → Facturation***
   (choix C34, qui révise C25). Sans lui, le refus de l'AC1 pour les **créanciers** renvoie à un champ
   qu'aucun écran ne montre : la saisie de **toute** facture fournisseur et la complétion de **tout**
   import échouent, et le seul recours est un `PUT` à la main avec un cookie de session Admin
   (`grep -rn "defaultPayableAccountId" frontend/src | grep -v test` → aucune sortie sur `007c4eb1`).
   Dans `frontend/src/routes/(app)/settings/invoicing/+page.svelte` :
   - un `<select>` **Compte créanciers (Passif)** dans la section *Comptes par défaut*
     (`:296-342`, titre `settings-invoicing-default-accounts-title`), **juste après la créance**
     (`:300-314`), son pendant côté achat — **pas** dans la section *Comptes TVA* (`:344-400`), où
     se trouve la TVA due : le compte créanciers n'est pas un compte de TVA (findings R2-6 / F2-5),
     `data-testid="settings-payable-account"`, clé neuve
     `settings-invoicing-payable-account` dans les quatre locales (« Compte créanciers (Passif) » ;
     DE/IT/EN sur le patron de `invoices-settings-vat-payable`, `fr-CH/messages.ftl:513`) ;
   - ses options : `withCurrentAccount(liabilityAccounts, payableId, accounts)` — **même filtre que la
     TVA due** (`liabilityAccounts` : `active && postable && accountType === 'Liability'`, `:70-72` ; le
     serveur exige le type `Liability`, AC19 de la 15-5b) ; la **valeur courante est préservée** même si
     le compte est devenu non imputable (patron #271, `:80-87`) : elle reste affichée et sélectionnée,
     et l'exemption « inchangé » de l'AC11 de la 15-5b la laisse passer à l'enregistrement ;
   - la valeur est **lue** au chargement (`onMount`, `:123-145`) et à la relecture sur conflit de
     version (`:199-214`), et **envoyée** par `updateInvoiceSettings` (`:174-189`) ;
   - les types `InvoiceSettingsResponse` et `UpdateInvoiceSettingsRequest`
     (`frontend/src/lib/features/invoices/invoices.types.ts:119-162`) gagnent
     `defaultPayableAccountId: number | null` — le serveur le rend déjà
     (`crates/kesh-api/src/routes/company_invoice_settings.rs:52`, `:79`) ;
   - le **mock** de `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
     (fonction `settings()`, `:60-75`, champs listés à partir de `:63` — `defaultVatPayableAccountId: null`
     à `:67`) gagne `defaultPayableAccountId: null` : sans lui, le champ devenu obligatoire du type casse
     `npm run check` (finding R1-5 de la P1).
   Les numéros de ligne de cet AC sont relevés sur `67c31c95` et **relus sur `cecd5d1d`** (écran des
   réglages inchangé ; C85) ; ceux de l'AC8 sont relocalisés sur `cecd5d1d` : T0 les refait
   (finding R1-7 de la P1).
   L'AC19 de la 15-5b (absent du corps → préservé) **reste** le filet pour un client qui ne l'envoie
   pas (onglet ouvert avant la mise à jour, intégration) ; choisir « — Sélectionner — » l'efface,
   comme pour les autres champs (`null`).
6. **AC6 — Les contournements des E2E restent, et leur motif est corrigé** (findings R2-3 / F2-2,
   choix **C45**, qui révise C34). L'inventaire se fait **par commande**, non par liste :
   `grep -rn "defaultPayableAccountId == null" frontend/tests/e2e` rend **trois** sites sur `ac1719b9`
   (identiques sur `cecd5d1d`) — `payment-batches.spec.ts:59`, `inbox-import.spec.ts:93` et `supplier-invoices.spec.ts:86`, cette
   dernière exerçant le flux même que l'AC1 garde ; T0 refait la commande. Chacun repose le compte
   créanciers par un `PUT` d'API quand il vaut `null`. La prémisse des passes précédentes (« le seed
   E2E le désigne ») est **fausse** : les presets `post-onboarding` et `with-company` appellent
   `seed_accounting_company` (`crates/kesh-api/src/routes/test_endpoints.rs:184`), **pas**
   `insert_with_defaults`, et son `INSERT INTO company_invoice_settings`
   (`crates/kesh-db/src/test_fixtures.rs:156-170`) ne pose que créance, produit, TVA due et TVA
   récupérable — jamais `default_payable_account_id`. Les contournements complètent donc un seed
   **indépendamment** de #521 : ils **restent**. Le dev **réécrit leur commentaire** dans les trois
   specs : « le seed `with-company` (`seed_accounting_company`) ne désigne pas le compte créanciers ».
   Le seed n'est **pas** étendu (rayon d'impact : tous les tests `kesh-db` et `kesh-api` qui l'emploient,
   dont des assertions sur le contenu des réglages — hors de cette story, C45). La suite E2E complète du
   gate D7 (T7) reste le juge de non-régression de ces trois fichiers, jugée fichier par fichier contre
   `docs/testing.md` § « Les échecs attendus ». Le résultat est consigné au Dev Agent Record.

### Ce qui doit être prouvé

7. **AC7 — Tests, un par compte, et le chemin de correction à l'écran.**
   - `kesh-db` (fichiers de tests de la validation de facture, p. ex.
     `crates/kesh-db/tests/invoices_validate_vat.rs`, et `crates/kesh-db/tests/supplier_invoices_repository.rs`) ;
     le compte de test ne diffère d'un compte accepté **que par `postable`** et est **désigné avant**
     d'être rendu non imputable (le chemin réel : l'exemption de l'AC11 de la 15-5b l'a laissé en
     place) :
     - **créance** non imputable → validation refusée, `DesignatedAccountsNotPostable` nommant
       `(id, n°)`, facture toujours brouillon, aucune écriture ;
     - **TVA due** non imputable : facture **avec** TVA → refusée ; facture **sans** TVA → validée ;
       facture **à taux positif dont la TVA arrondit à zéro** → validée (finding R4-3/F4-4) ;
     - **ordre** : facture dont le compte de produit **explicite** d'une **ligne** (distinct du compte
       de produit par défaut, exempté) et la **créance** sont non imputables →
       `InvalidRevenueAccounts` (`INVOICE_LINE_REVENUE_ACCOUNT_INVALID`, raison `NotPostable`, Story
       16-1a — finding F5-2 de la P5 ; patron `invoices_line_revenue_account.rs:515`) ; TVA due **absente** → `ConfigurationRequired` (finding F4-5) ; **aucun exercice
       ouvert** à la date de la facture **et** créance non imputable → `FiscalYearInvalid` (le verrou
       des comptes précède désormais l'exercice, leur refus non — finding F2-1, C43) ; côté achat,
       aucun exercice ouvert **et** créanciers non imputables → `FiscalYearInvalid` ;
     - **créanciers** non imputable → saisie de facture fournisseur refusée, rien d'écrit ;
     - **TVA récupérable** non imputable : facture fournisseur **avec** TVA → refusée ; **sans** TVA
       → acceptée ;
     - créance **et** TVA due non imputables, désignées sur **deux comptes distincts** (`1100` et
       `2000` de `seed_accounting_company`) → **un** refus nommant les deux, `count = 2`, pluriel
       (finding F1 de la P1) ;
     - **un même compte désigné pour deux rôles** (créance et TVA due posées sur le même identifiant
       par un `UPDATE company_invoice_settings` direct dans le montage, sur le patron de
       `supplier_invoices_repository.rs:49-55` — finding R2-7), rendu non imputable, facture avec TVA → **un** refus qui le nomme
       **une fois**, `count = 1`, message au **singulier** (choix C40 ; il prouve que le
       dédoublonnage de `NonPostableAccounts::new` est bien atteint par l'accesseur) ;
     - compte de réglage **archivé** → `InactiveOrInvalidAccounts`, désormais rendu par l'accesseur
       (C49 ; la variante neuve n'est pas émise). ⚠️ Ce test prouve le **résultat**, non **qui** le
       produit : sans la course d'archivage (fenêtre de quelques millisecondes entre l'instantané et
       le verrou), `create_in_tx` rendrait le même refus. Le refus de l'accesseur est donc vérifié
       **à la lecture** et consigné au Dev Agent Record ; sa mutation n'est pas attendue rouge, et
       c'est écrit ;
     - **un identifiant d'une autre société** (finding F3-5, choix C51) : un compte d'une seconde
       société, posé comme TVA due des réglages par un `UPDATE company_invoice_settings` direct ;
       une connexion ouvre une transaction et appelle l'accesseur de verrou sur les candidats (dont
       cet identifiant), **sans conclure** ; une seconde connexion verrouille la ligne étrangère par
       `SELECT id FROM accounts WHERE id = ? FOR UPDATE NOWAIT` → **réussit** (la ligne n'est pas
       verrouillée). L'accesseur ayant adopté d'emblée le patron `owned_account_ids` (AC1, C88), la
       sonde réussit par construction ; elle **rougit** sous la mutation qui le retire (verrou posé
       directement sur `company_id = ? AND id IN (…)`, la forme mesurée fautive) ; le plan (`EXPLAIN`)
       est relevé au Dev Agent Record. **Et le refus** (finding F6-5 de la P6, branche « absent de
       l'instantané » de C49, qu'aucun autre test ne nomme) : la même société valide ensuite une
       facture **avec** TVA, cette TVA due étrangère étant le compte écrit → `InactiveOrInvalidAccounts`,
       facture toujours brouillon, aucune écriture. ⚠️ Même limite que le test « archivé » : sans la
       branche de l'accesseur, le contrôle non verrouillant de `create_in_tx` (`WHERE company_id = ?
       AND id IN (…)`, `journal_entries.rs:117-121`) rendrait le même refus — le test prouve le
       **résultat**, non **qui** le produit ; sa mutation n'est pas attendue rouge, c'est écrit, et le
       refus de l'accesseur est vérifié **à la lecture** au Dev Agent Record (angle mort assumé) ;
     - **avoir** (`crates/kesh-db/tests/credit_notes_repository.rs`) sur une facture dont la
       créance des réglages est devenue non imputable → émis (C35 :
       l'exemption est voulue et un test la fige) ;
     - **place et mode du verrou de l'accesseur — trois tests de place et un test de mode**, qui
       figent cette **place** (findings F2-1 = R2-1, F3-2 ; choix C43, C53) et ce **mode** (finding
       F5-1, choix C87) — **pas** l'absence d'interblocage, que plus rien n'affirme (C54 ; le rejeu est
       à la route, 15-5e1 et 15-5e2). **Réécrits par C87** : chaque test **tient lui-même** un verrou
       concurrent sur une ligne précise — un `UPDATE` non validé, un `FOR UPDATE` ou un `LOCK IN SHARE
       MODE` — et constate l'attente (ou l'absence d'attente) de la tâche **sur une requête nommée** ;
       l'ancien test 1 (escompte ↔ validation, deux flux réels) est **retiré** : sous le verrou
       exclusif, il formait lui-même le cycle de F5-1 (son issue « succès ou 1213 » était un 1213
       certain), et deux flux réels font dépendre l'issue d'un ordre que le test ne fige pas. Patron
       commun : une transaction **bloqueuse**, une tâche de travail, des **sondes** `… FOR UPDATE
       NOWAIT` (rendent aussitôt une erreur de verrou si la ligne est tenue), et
       `attendre_une_requete_en_cours` (`crates/kesh-db/src/test_fixtures.rs:560` ; précédents **de ce
       helper** `opening_complement_repository.rs:736`, `supplier_invoices_repository.rs:1669`) — elle rend
       `false` si la tâche finit sans attendre, ce que le test asserte faux. **`NOWAIT` est employé ici
       pour la première fois dans le dépôt** (`grep -rniE nowait crates` vide ; finding R6-4 de la P6) :
       sous MariaDB 10.11, une sonde contre une ligne tenue rend aussitôt **`1205`** (*Lock wait timeout
       exceeded*, mesuré en P6 par la lentille R sur `kesh-mariadb-dev` 10.11.16 — et **non** le `3572`
       de MySQL) ; le test discrimine donc « sonde réussie » contre « `Err` portant le code `1205` »,
       et toute autre erreur le fait échouer. Motifs d'attente de
       l'accesseur, fixés par le dev et écrits au test : p. ex. `FROM accounts WHERE` et
       `LOCK IN SHARE MODE`. ⚠️ Le doc-comment du helper (`test_fixtures.rs:540-558`) dit aujourd'hui
       qu'« un verrou écrit autrement (`LOCK IN SHARE MODE`) ne serait pas vu » : la phrase vaut pour des
       motifs qui nomment `FOR UPDATE` — le helper ne fait que chercher les motifs dans le texte de la
       requête en cours (`INFO LIKE`) — et induirait en erreur ici ; elle est **réécrite** (finding R6-2
       de la P6) : « les motifs doivent figurer dans le texte de la requête : un verrou d'une autre forme
       n'est vu que si les motifs le nomment ». **Montage** (finding R6-1 de la P6, choix C88) : tous les
       tests de vente de cette story **sauf le test 3** posent `disable_rounding_to_5_centimes`
       (`test_fixtures.rs:236`, patron `invoices_validate_vat.rs:153`) après `seed_accounting_company` —
       sinon une facture dont le TTC n'est pas multiple de 0.05 réclame un compte d'arrondi
       (`invoices.rs:2093`) que ce montage ne désigne pas, et s'arrête **avant** l'accesseur ; choisi
       plutôt qu'un montant multiple de 0.05, parce que le test « TVA arrondie à zéro » ne peut pas le
       tenir et qu'un montage ne doit pas dépendre d'une arithmétique. Côté achat, sans objet : la saisie
       fournisseur n'a pas d'étape d'arrondi (`grep round crates/kesh-db/src/repositories/supplier_invoices.rs`
       vide). Aucun de ces tests ne fait concourir deux flux qui écrivent : une seule
       transaction de travail, rien avec quoi former un cycle :
       1. **vente — avant l'exercice, et lecture fraîche** — `crates/kesh-db/tests/invoices_validate_vat.rs`
          (montage `seed_accounting_company` : créance `1100`, TVA due `2000`, exercice ouvert ;
          facture brouillon avec TVA, patron `create_and_validate`, `:57-85`). La bloqueuse exécute
          `UPDATE accounts SET postable = FALSE WHERE id = <créance>` **sans conclure** (le geste réel
          de l'archivage ou du passage à non imputable, verrou exclusif) ; tâche : **validation** — vue
          en attente **sur le verrou de l'accesseur** ; **sonde** `SELECT id FROM fiscal_years WHERE
          id = ? FOR UPDATE NOWAIT` → **réussit** (puis annule) : la validation ne tient pas encore
          l'exercice. La bloqueuse **valide** ; la validation reprend, lit la ligne **fraîche** et
          rend `DesignatedAccountsNotPostable` nommant la créance — facture toujours brouillon, aucune
          écriture. Sous l'ordre fautif (verrou après l'exercice), la sonde échoue ; sous un accesseur
          sans verrou (lecture simple), la tâche n'est jamais vue en attente **sur l'accesseur** (elle
          lit l'instantané, puis bute sur l'insertion des lignes) : `attendre_une_requete_en_cours`
          rend `false` ou panique — le test rougit dans les deux cas ;
       2. **achat — avant l'exercice, et lecture fraîche** (finding F3-2) —
          `crates/kesh-db/tests/supplier_invoices_repository.rs`, montage de ce fichier (`setup`,
          `:36` ; compte créanciers désigné sur `2000`, `:49-55`) : même schéma — la bloqueuse
          exécute l'`UPDATE accounts SET postable = FALSE` sur le compte **créanciers** sans
          conclure ; tâche : **saisie d'une facture fournisseur à TVA** — vue en attente sur le
          verrou de l'accesseur ; sonde sur l'exercice → réussit ; la bloqueuse valide ; la saisie
          rend `DesignatedAccountsNotPostable`, rien d'écrit. Sous l'ordre fautif (verrou des
          créanciers et de la TVA récupérable après l'exercice), la sonde échoue ;
       3. **vente — la validation prend l'arrondi avant les comptes désignés** (C53) —
          `crates/kesh-db/tests/invoice_amount_due_parity.rs` (montage `seeded_with_rounding`, `:72` :
          compte d'arrondi désigné) : une facture brouillon **avec TVA et arrondie** (total TTC non
          multiple de 0.05). La bloqueuse verrouille la ligne du **compte d'arrondi** (`FOR UPDATE`) ;
          tâche : **validation** — vue en attente sur le compte d'arrondi (motifs de la requête de
          `rounding_account_for_write`) ; **sondes** `SELECT id FROM accounts WHERE id = ? FOR UPDATE
          NOWAIT` sur la **TVA due**, puis sur la **créance** → **réussissent** (puis annulent). Sous
          l'ordre fautif (verrou de l'accesseur avant l'arrondi), la tâche tient déjà ces deux comptes
          en partagé, et une sonde exclusive échoue aussitôt contre un partagé. La bloqueuse annule ;
          la validation réussit ;
       4. **vente — le verrou est partagé** (C87 ; le seul test qui rougit si l'accesseur repasse en
          `FOR UPDATE`) — `crates/kesh-db/tests/invoices_validate_vat.rs`, même montage que le test 1.
          La bloqueuse reproduit ce que tient un règlement client à l'insertion de ses lignes :
          l'exercice (`SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE`) **et** la créance et la
          TVA due **en partagé** (`SELECT id FROM accounts WHERE id IN (…) LOCK IN SHARE MODE`) ;
          tâche : **validation avec TVA** — vue en attente **sur l'exercice** (motifs de
          `find_open_covering_date`), donc **passée** le verrou de l'accesseur malgré les partagés de
          la bloqueuse. La bloqueuse annule ; la validation réussit. Sous un accesseur en `FOR UPDATE`,
          la tâche attend sur les comptes et non sur l'exercice : `attendre_une_requete_en_cours`
          panique au bout de dix secondes — le test rougit sans dépendre du minutage ;
   - `kesh-api`, deux fichiers (finding R1-3 de la P1) :
     - **validation d'une facture** — `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`,
       fichier **créé par la 15-5b** (mergé). Son `setup` (`:226-283`) **ne suffit pas** à valider une
       facture (findings F5-4 = R5-3 de la P5) : il crée une société nue par `companies::create`, un
       Admin, les douze comptes des six champs et un `PUT` des réglages — ni exercice, ni contact, ni
       taux de TVA (que `POST /api/v1/invoices` vérifie contre la base, `routes/invoices.rs:718`), ni
       compte d'arrondi (or `round_to_5_centimes` vaut `TRUE` par défaut). Le test de cette story
       **n'emploie donc pas `setup`** ; il reprend du fichier `spawn_app` (`:59`) et `set_postable`
       (`:215`), et monte le reste par des fixtures **existantes** :
       - **`seed_accounting_company`** (`kesh_db::test_fixtures`, `test_fixtures.rs:80`) — société,
         exercice ouvert 2020-2030, comptes `1000`-`4000`, réglages (créance `1100`, TVA due `2000`),
         quatre taux de `vat_rates` (dont 8.10) ; patron `invoice_unvalidate_e2e.rs:110-115` ;
       - **`disable_rounding_to_5_centimes`** (`test_fixtures.rs:236`) : la facture est émise sans
         arrondi, aucun compte d'arrondi n'est exigé avant la garde ;
       - un **contact client** par `contacts::create` (patron `seed_contact`,
         `invoice_unvalidate_e2e.rs:117-149`) ;
       - un **utilisateur Comptable** par `users::create` (`Role::Comptable`) et son jeton par
         `POST /api/v1/auth/login` (patron `invoice_unvalidate_e2e.rs:328-345` et `login`, `:97-108`) —
         le message doit valoir pour lui (C36) ; le fichier ne forge aujourd'hui qu'un jeton Admin
         (`forge_admin_jwt`, `:100`) ;
       - **`kesh_api::errors::init_error_i18n(i18n.clone(), config.locale)`** ajouté à `spawn_app`
         (patron `invoice_unvalidate_e2e.rs:75`) : sans lui, le `message` du refus n'est pas résolu ;
       une facture brouillon à une ligne de 100.00 à 8.10 % (TTC 108.10), par `POST /api/v1/invoices`
       (ou `invoices::create`, patron `create_draft`, `invoice_unvalidate_e2e.rs:151-175`), datée dans
       l'exercice ; la créance désignée puis rendue non imputable (`set_postable(…, false)`) ; route
       `POST /api/v1/invoices/{id}/validate` (`lib.rs:510`) avec le jeton du Comptable →
       400 `ACCOUNT_NOT_POSTABLE`, `details.rejected`, `message` contenant « Paramètres → Facturation »,
       le numéro, et « administrateur » ; facture toujours brouillon ;
     - **complétion d'une facture importée** — `crates/kesh-api/tests/inbox_import_e2e.rs`
       (existant), sur le montage de `complete_creates_invoice_and_marks_completed` (`:716` :
       `setup`, `seed_staging`, `complete_body`) : le compte créanciers des réglages rendu non
       imputable, `POST /api/v1/imported-supplier-invoices/{id}/complete` → 400
       `ACCOUNT_NOT_POSTABLE` ; le staging **reste** `to_complete` (`staging_status`) et
       `supplier_invoice_count` est inchangé (patron de `complete_closed_fiscal_year_keeps_to_complete`,
       `:792`) ;
   - **mutation** (objet précisé, finding R2-8) : pour chacun des quatre rôles, **ne pas l'insérer**
     dans l'ensemble rendu par le générateur → son test rougit ; **retirer l'appel du contrôle** dans
     chacun des deux flux → les tests de ce flux rougissent ; **une mutation d'ordre par flux**
     (finding F3-2) : dans `validate_invoice`, déplacer le verrou des comptes désignés après
     `find_open_covering_date` → le test de place 1 rougit ; dans
     `supplier_invoices::create_in_tx`, le même déplacement → le test 2 rougit ; dans
     `validate_invoice`, déplacer ce verrou **avant** le compte d'arrondi → le test 3 rougit ;
     **deux mutations du mode** (C87) : l'accesseur en `FOR UPDATE` → le test 4 rougit ; l'accesseur
     en lecture simple (sans verrou) → les tests 1 et 2 rougissent ; **retirer le patron
     `owned_account_ids`** (verrou posé directement sur `company_id = ? AND id IN (…)`) → la sonde du
     test « identifiant d'une autre société » rougit (C88 ; attendu d'après la mesure
     d'`opening_complement.rs:429-437`, et consigné tel qu'observé) ; poser
     le rôle `VatPayable` **hors** de la branche `total_vat > 0` du générateur → le test « TVA
     arrondie à zéro » rougit ; contourner `NonPostableAccounts::new` (liste construite sans
     dédoublonnage) → le test « même compte pour deux rôles » rougit ; consigné au Dev Agent Record,
     en touchant le fichier après restauration. Les mutations du rejeu des routes sont celles des
     **15-5e1** et **15-5e2** ;
   - **aucun test de rejeu dans cette story** : le rejeu de la validation et de la saisie
     fournisseur est livré et prouvé par la 15-5e1 (tests 4 et 5 de `rejeu_interblocage_e2e.rs`,
     avec témoin) ; les quatre tests de place et de mode travaillent **au niveau du dépôt**, sans
     rejeu, avec une seule transaction de travail : aucun n'a d'interblocage à accepter. Si le dev ajoute néanmoins un test qui exige un rejeu à la
     route, il prend le témoin de la 15-5e1 (`crates/kesh-api/tests/common/capture_rejeu.rs` :
     `CaptureRejeu::installer()`, puis `exiger_un_rejeu("invoices::validate")`) — un statut 200 seul
     ne prouve pas qu'un rejeu a eu lieu (finding F6-1 de la 15-5e1, C74) ;
   - **frontend** (`frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`,
     existant) : le compte créanciers en place s'affiche, **y compris devenu non imputable** (valeur
     préservée) ; un changement est envoyé dans `updateInvoiceSettings` ; la relecture sur conflit le
     reprend. C'est la preuve du **chemin de correction** que le message de l'AC2 désigne ;
   - **frontend, écran de validation** (AC4, choix C41) — fichier neuf
     `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts`, patron de
     `invoice-write-off-page.test.ts` (mocks hoistés avant l'import du composant) : la validation
     rejetée par un `ApiError` 400 `ACCOUNT_NOT_POSTABLE` affiche **`err.message` tel quel**, pour un
     utilisateur **Comptable** comme pour un **Admin** — ni le suffixe « Configurez les comptes par
     défaut », ni « Demandez à votre administrateur » de la branche `CONFIGURATION_REQUIRED`
     (`:409-416`) — **et le dialogue fermé** (C46). Mutations attrapées : `ACCOUNT_NOT_POSTABLE` ajouté
     à cette branche ; retiré de la liste de fermeture (`:428-433`). Les deux `catch`
     fournisseurs (`supplier-invoices/+page.svelte:213-214`, `import/+page.svelte:261`) n'ont pas de branche
     par code sur ce chemin : leur lecture est consignée au Dev Agent Record (AC4), sans test d'écran.
8. **AC8 — Le manuel dit l'usage, l'avoir, et le champ créanciers.** L'AC17 de la 15-5b a levé les
   réserves des encadrés pour la **désignation** ; cette story les complète pour l'**usage** :
   - `docs/manual/fr/user-manual.tex`, encadrés **`:379-385`** (`keshnote` du § *Rôles des comptes*,
     `:369`) et **`:393-395`** (`keshnote` du § *Les comptes de clôture*, `:387`) — relocalisés sur
     `cecd5d1d` par leur texte (`:380`, `:390` sur `67c31c95`) : l'item que la 15-5b y a écrit — un compte de réglage devenu non
     imputable **après** sa désignation reste utilisé — est **réécrit** : la validation d'une facture et
     la saisie d'une facture fournisseur le refusent désormais, avec un message qui renvoie à
     *Paramètres* → *Facturation* ; l'**avoir** fait exception (il relit la créance et la TVA due dans
     les réglages au moment de l'avoir, sans les contrôler) ; le **compte de produit par défaut** aussi
     (D3-bis). Aucune phrase ne dit que l'avoir « reprend les comptes de la facture d'origine » pour
     la créance et la TVA due : c'est **faux** (AC3). L'item est le cas **(4)** d'une énumération
     amorcée par « **Quatre** cas échappent encore à ce contrôle » (`user-manual.tex:382`) : l'amorce,
     l'énumération et la phrase qui suit (« Si vous scindez un tel compte, un administrateur peut
     désigner à sa place l'un de ses sous-comptes ») sont reprises **ensemble**, et le nombre de cas
     se **recompte** après réécriture — grep de la **valeur** `Quatre` dans le `.tex` et dans le PDF
     aplati (finding F5-6 de la P5) ;
   - § validation d'une facture (`user-manual.tex:898`, paragraphes du total arrondi `:911` et du
     montant minimum `:913`, qui disent ce qui empêche la validation : compte d'arrondi, montant
     minimum) : une phrase pour le refus de l'AC1 (finding F4-7) ;
   - § *Saisir une facture fournisseur* (`:1349`) : le compte de dette fournisseur est le **compte
     créanciers** désigné dans *Paramètres* → *Facturation* ; s'il n'est pas imputable (ou la TVA
     récupérable, pour une facture qui en porte), la saisie est refusée avec un message qui y renvoie ;
   - § *Importer des factures depuis un dossier* (`user-manual.tex:1437`, bouton *Compléter* `:1450`, « Kesh
     vérifie que le total TTC correspond […] avant de créer la facture fournisseur définitive ») : une
     phrase dit que la complétion est refusée de la même façon — la complétion crée une facture
     fournisseur par le même chemin que la saisie (`imported_supplier_invoices.rs:243`) — et renvoie au
     paragraphe de la saisie (finding F5 de la P1) ;
   - `docs/manual/fr/admin-manual.tex`, § *Configuration des comptes TVA* (`:2018-2027`) : la phrase
     que la 15-5b y ajoute (refus à la désignation) est complétée — un compte TVA désigné **devenu**
     non imputable bloque la validation d'une facture portant de la TVA (ou la saisie d'une facture
     fournisseur) avec un message qui renvoie aux paramètres ; et un paragraphe voisin nomme le **compte
     créanciers** (`2000` dans les plans livrés), désormais réglable sur la même page, sur le même
     patron « actif et imputable » ;
   - le passage de l'avoir (`user-manual.tex:1219`, « l'inverse exact de l'écriture de la facture (la
     créance client, le produit et la TVA due …) ») n'est **pas** réécrit ici : il relève de #473 et
     #525 ; le dev le **signale** au Dev Agent Record sans le corriger ;
   - les PDF (`user-manual.pdf`, `admin-manual.pdf`) sont régénérés (`latexmk -xelatex` dans
     `docs/manual/fr/`), commités, et **contrôlés aplatis en normalisant les ligatures** :
     `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g; s/ﬁ/fi/g; s/ﬂ/fl/g'`
     (le corps du PDF rend « ff » par la ligature `ﬀ` : un `grep` de « affiche » ou « différences » y
     rend un faux négatif — finding F-7 de la P2 de la 15-5c) ; les phrases nouvelles présentes, les
     phrases levées absentes.
9. **AC9 — Doc-comments, `docs/api-external.md`, CHANGELOG.**
   - Doc-comments de l'accesseur (ses deux temps, et pourquoi), de `validate_invoice` — « # Erreurs »
     (ordre des refus de l'AC1) **et « # Ordre des locks (canonique — Story 5.2, réécrit par la
     Story 15-5e1) »** (`invoices.rs:1916-1957`, livré) : le schéma gagne, sous `(2 bis')` (le compte
     d'arrondi), une ligne **`(2 bis', suite)  accounts  comptes désignés (créance, TVA due),
     LOCK IN SHARE MODE (partagé), ORDER BY id`**, avant `(2 quater)` et `(3)` (finding R2-1 de la 15-5d, et R2-1 de la
     P2 de la 15-5e ; place exacte à l'AC1) — et de `supplier_invoices::create_in_tx`
     (« # Ordre des verrous », `supplier_invoices.rs:249-296`, livré) : une ligne **`(2, désignés)
     accounts  comptes désignés (créanciers, TVA récupérable), LOCK IN SHARE MODE (partagé),
     ORDER BY id`** entre `(2, suite)` et `(3)` (étiquette fixée par C87, finding R5-4) ; la phrase « Les comptes de charge ne sont pas triés : les cycles qui
     restent … » est relue, et complétée si les comptes désignés y entrent. Chaque étiquette neuve
     existe **aussi** en commentaire au code, à sa place (C85, C87). Le doc-comment de l'accesseur dit
     **pourquoi partagé** (AC1 : il suffit contre l'archivage et le passage à non imputable, qui
     écrivent la ligne ; il est compatible avec les verrous partagés de `fk_jel_account` des flux qui
     prennent l'exercice d'abord) —
     ces deux doc-comments sont les **seuls lieux** où l'ordre de ces flux est écrit : le Pattern 5
     (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, réécrit par la 15-5e2) y renvoie sans le recopier
     (choix C68), si bien que cette story n'a pas à le toucher, dans quelque ordre que les deux
     merges arrivent ;
     la limite **L2** de D-A0 est citée comme **révisée** pour ces quatre comptes
     (`14-3b-consommateurs-roles.md:188`).
   - Le commentaire « 5 bis » du solde du reste (`invoice_settlements_write.rs`), **réécrit en place
     par la 15-5e1** (son AC5) sans le verrou de la validation sur la TVA due, qui n'existait pas
     encore, **gagne une phrase** : la validation verrouille désormais la TVA due (et la créance),
     **en partagé**, après le compte d'arrondi et **avant** l'exercice, dans le même ordre « arrondi,
     puis TVA due » que le solde du reste ; un interblocage
     restant entre eux est rejoué par les deux routes (validation : 15-5e1 ; solde du reste : déjà
     rejoué, migré vers l'enveloppe par la 15-5e2). Aucune autre phrase de ce commentaire
     n'est touchée ; il ne prétend à l'absence d'aucun cycle (finding F3-3, C54). ⚠️ La **15-5e2**
     réécrit, dans ce même commentaire (`invoice_settlements_write.rs:474-486`), la mention
     « (`write_off_invoice_handler`, `retry_with`) » (son AC1) : la seconde des deux à merger garde
     les deux changements.
   - `docs/api-external.md`, table du § 10 : la ligne `ACCOUNT_NOT_POSTABLE` (posée par la 15-5a,
     étendue par la 15-5b) gagne la **validation d'une facture** et la **saisie d'une facture
     fournisseur** quand un compte désigné dans les réglages n'est pas imputable — même `details`.
     Et la table du solde du reste (`docs/api-external.md:321`, « Compte de TVA due absent (escompte,
     perte) | `CONFIGURATION_REQUIRED` ») devient « Compte de TVA due **absent ou inutilisable**
     (archivé, non imputable) » : c'est ce que rend déjà `vat_payable_account_for_write`
     (`company_invoice_settings.rs:454-475`), et la divergence de code avec la validation (C28, AC3)
     doit se lire là où l'intégrateur lit (finding F2-7).
   - `CHANGELOG.md`, section `## [0.13.0] — Non publié` (créée par la 15-5a ; **la créer en tête si
     absente** — motif exact exigé par `scripts/prepare-release.sh:189`). **Deux phrases de cette
     section deviennent fausses avec cette story, et sont réécrites** (finding F5-3 de la P5 ; relevées
     sur `cecd5d1d`, à relocaliser par le texte) :
     - dans l'entrée **Corrigé** « Un compte non imputable est refusé partout où un client le désigne
       … (#427, #429) » (`CHANGELOG.md:23`), la « Limite » « et les écritures automatiques continuent
       de l'utiliser ; la garde à l'usage des comptes de réglage viendra dans une version suivante »
       devient : la validation d'une facture et la saisie (ou la complétion) d'une facture fournisseur
       refusent désormais un compte de réglage devenu non imputable (entrée ci-dessous) ; **restent**
       hors de cette garde l'avoir, le compte de produit par défaut et le compte comptable d'un compte
       bancaire déjà lié ;
     - dans l'entrée **Corrigé** de #521 (`CHANGELOG.md:25`), « L'écran *Paramètres → Facturation*
       n'envoie pas ce compte » devient « L'écran *Paramètres → Facturation* n'envoyait pas ce compte
       (il l'affiche et l'envoie désormais, voir *Ajouté*) » ;
     et s'y ajoutent :
     - rubrique **Corrigé** : la validation d'une facture et la saisie d'une facture fournisseur
       refusent un compte de réglage (créance, TVA due, créanciers, TVA récupérable) devenu non
       imputable (#429) ;
     - rubrique **Ajouté** : le compte créanciers se règle dans *Paramètres → Facturation* — la
       section `[0.13.0]` **n'a pas encore de rubrique `### Ajouté`** (elle porte `### Modifié` puis
       `### Corrigé`, relevé en P6, finding F6-3) : la **créer en tête de la section**, avant `###
       Modifié`, dans l'ordre de Keep a Changelog que suit `[0.12.1]` (`Ajouté`, `Modifié`,
       `Corrigé`) ; le renvoi « voir *Ajouté* » de l'entrée #521 n'est vrai qu'après cette création ;
     - **une ligne d'action** (finding F4-8 — précédent 16-1a-bis, où un « aucune action de votre
       part » a été pris en défaut) : « Après la mise à jour, vérifiez dans *Paramètres → Facturation*
       que les comptes désignés sont imputables : un compte devenu non imputable (sous-comptes créés,
       case *imputable* décochée…) doit y être remplacé, faute de quoi la validation des factures (ou
       la saisie des factures fournisseurs) est refusée. » (finding F5-5)

## Tasks / Subtasks

- [x] **T0 — Refaire les relevés** sur `HEAD` (15-5a, 15-5b, 15-5e1 mergées ; **la 15-8b aussi** — la
      branche part de `main` après son merge, ou se rebase dessus —, R5-1) : numéros de ligne de
      `invoices.rs` (génération, `create_in_tx`, `:1849-1852`), `supplier_invoices.rs`, `credit_notes.rs`,
      des écrans et des manuels ; refaire le grep des lecteurs des quatre comptes
      (`grep -rnE "default_(receivable|payable|vat_payable|vat_recoverable)_account_id" crates/*/src`, hors
      tests et module des réglages) — **un lecteur qui écrit, absent de l'AC1 et de l'AC3, bloque la
      story** ; refaire le grep des appelants des générateurs (AC1, 25 occurrences sur `ac1719b9` et sur `cecd5d1d`, ventilées) et
      celui des contournements E2E (AC6, trois sites) ; **vérifier que la 15-5e1 est en place** — rejeu
      de la validation et de la saisie fournisseur, réglages de la saisie fournisseur avant les comptes
      de charge, doc-comments canoniques de `validate_invoice` et de `supplier_invoices::create_in_tx`
      et « 5 bis » réécrits ; **relever si la 15-5e2 est mergée** (rejeu de la complétion d'import) et
      le consigner, sans en faire une condition (C65, C66) — et
      relever les deux cases où l'accesseur s'insère — après le bloc `let rounding` de l'étape
      `(2 bis')` de `validate_invoice`, entre `(2, suite)` et `(3)` de `supplier_invoices::create_in_tx`
      (AC1, « L'ordre des verrous ») ; **un écart bloque
      la story** (la 15-5d ne réordonne aucun flux existant) ; **confirmer** sur `HEAD` l'ordre des
      verrous du lot pain.001 et de l'acceptation par lot du rapprochement, que la P6 a établi par
      lecture (Dev Notes, cycle (a bis)) — un `FOR UPDATE` neuf sur un des quatre comptes désignés
      après l'exercice dans l'un d'eux bloque la story ; exécuter à la main, sur la base de dev,
      l'`EXPLAIN` de la requête verrouillante prévue (`SELECT id, number, active, postable FROM
      accounts WHERE company_id = ? AND id IN (…) ORDER BY id LOCK IN SHARE MODE`) et relever le type
      d'accès au Dev Agent Record (F6-4 ; refait sur la requête écrite en T1) ;
      **relever le texte exact que la 15-5b a laissé** dans les encadrés `user-manual.tex:379-385` et
      `:393-395` et à `admin-manual.tex:2018-2027` (numéros de `cecd5d1d` ; l'item que l'AC8 réécrit
      n'existait pas sur `67c31c95`),
      le copier au Dev Agent Record et, s'il ne contient aucun des motifs du grep de T6, **ajouter à ce
      grep** le motif qui le retrouve (finding R1-8 de la P1) ; relever le montage que la 15-5b a
      écrit dans `company_invoice_settings_postable_e2e.rs` (AC7 : `setup`, jeton Admin seul).
- [x] **T1 — La garde** (AC1, AC3) : les rôles rendus par les deux générateurs (`DesignatedRole`,
      branches de `total_vat > 0`), rendus par `GeneratedLines` (C50), leurs appelants adaptés — dont
      les 19 tests, mécaniquement ; l'accesseur de `company_invoice_settings` en deux temps — verrou
      de tous les candidats (`ORDER BY id LOCK IN SHARE MODE`, partagé — C87 —, avant l'exercice,
      sur les seuls identifiants de la société lus d'abord sans verrou — patron `owned_account_ids`,
      C88 —, `EXPLAIN` relevé, C51 ; phrase des verrous d'intervalle au doc-comment, F6-4),
      contrôle des rôles écrits (compte absent ou inactif → `InactiveOrInvalidAccounts`, puis variante
      construite par `NonPostableAccounts::new`, après la génération, C49) ; ses appels, aux places
      fixées à l'AC1 (après l'arrondi ou les comptes de charge, avant l'exercice) ; la variante `DesignatedAccountsNotPostable` (`error_code()` → `"ACCOUNT_NOT_POSTABLE"`,
      `match` exhaustif de `kesh-db/src/errors.rs`) et son bras dans `crates/kesh-api/src/errors.rs` (400,
      `t_args`, `details()`) ; doc-comments.
- [x] **T2 — Le message** (AC2) : la clé dans les quatre `messages.ftl`, inscrite à
      `SELECTEURS_RESOLUS_COTE_SERVEUR` ; tests Rust par locale, singulier et pluriel.
- [x] **T3 — Les écrans** (AC4, AC5) : lecture des trois `catch` de l'AC4, consignée ;
      `ACCOUNT_NOT_POSTABLE` ajouté aux codes qui ferment le dialogue de validation (C46) ; le `<select>`
      du compte créanciers, ses types, le mock de `settings-invoicing-page.test.ts`, sa clé i18n
      (quatre locales) ; borne `sitesTotal` de
      `frontend/src/lib/shared/i18n-keys.test.ts` relevée **délibérément**, **sur l'état rebasé**
      (1904 sur `cecd5d1d`, à ne pas reprendre : 15-5c, 15-8a et 15-8b l'ont fait bouger), ventilation recomptée
      (`sitesTotal`, `sitesNonResolus`, `relais`, `sitesGabarit`, `litterauxMin`, `clesDepuisTsMin`) et
      écrite au Dev Agent Record.
- [x] **T4 — Les E2E** (AC6) : les trois contournements gardés, leur commentaire réécrit (le seed
      `seed_accounting_company` ne désigne pas le compte créanciers) ; non-régression jugée sur la
      **suite E2E complète** de T7.
- [x] **T5 — Les tests** (AC7) : `kesh-db` (dont « même compte pour deux rôles » et « deux comptes
      distincts », ordre des refus avec l'exercice, identifiant d'une autre société — sonde **et**
      refus `InactiveOrInvalidAccounts` (F6-5) —, les **trois**
      tests de place du verrou — deux vers l'exercice, un vers l'arrondi — et le **test de mode**
      (verrou partagé, C87) ; montage de vente sous `disable_rounding_to_5_centimes` sauf le test 3,
      sondes `NOWAIT` jugées sur le code `1205`, doc-comment d'`attendre_une_requete_en_cours`
      réécrit — C88), `kesh-api` (`company_invoice_settings_postable_e2e.rs`, `inbox_import_e2e.rs`),
      Vitest (réglages, écran de validation) ; mutations consignées.
- [x] **T6 — Manuel, API, CHANGELOG** (AC8, AC9) ; PDF régénérés et contrôlés aplatis (ligatures
      normalisées) ; **grep du symptôme** (règle *Propagation post-patch*) :
      `grep -rnE "reprend les comptes de la facture|sous-compte imputable|inverse exact|n'est pas exposé|aucun autre chemin ne prend de verrou|aucun cycle n'est|garde à l'usage des comptes de réglage viendra|n'envoie pas ce compte|Quatre cas|#429" docs/manual/fr/*.tex docs/api-external.md crates frontend/src CHANGELOG.md`
      — chaque occurrence est réécrite ou justifiée au Dev Agent Record.
- [x] **T7 — Gates** : gate complet backend (`scripts/test-fast.sh`, base remise à zéro avant) — **même
      en cours de boucle de revue**, la story touchant des repositories `kesh-db` ; gate frontend
      complet ; **E2E Playwright complet au dernier commit de code** (décision D7), jugé fichier par
      fichier contre `docs/testing.md` § « Les échecs attendus ».

*(Décompte : 9 AC, 8 tâches T0–T7.)*

## Dev Notes

### La garde à l'usage et la doctrine D-A0 (vérifié en validation P3 de la 15-5b)

D-A0 (14-3b) exempte les flux automatiques de la garde de `create_in_tx` (`enforce_postable = false`),
et sa limite **L2** dit en toutes lettres : « un compte de config (créance/produit/dette) devenu
non-postable après configuration reste posté par les flux automatiques […] Remédiation éventuelle :
re-vérifier `postable` à la résolution avec message dédié — amélioration future si un besoin se
manifeste » (`14-3b-consommateurs-roles.md:188`). L'AC1 **est** cette remédiation, pour quatre
comptes : elle ne touche pas `create_in_tx` ni son drapeau (D-A0 tenu), elle **révise L2** pour la
créance, la TVA due, les créanciers et la TVA récupérable — le besoin s'est manifesté (#429). Elle ne
contredit pas l'angle mort « compte bancaire **à l'usage** » (C6) : le compte bancaire n'est pas un
réglage de facturation, il sert à **tous** les rapprochements et règlements, et le bloquer à l'usage
changerait D-A0 elle-même — ce qui reste hors de cette story.

**Les plans livrés** (finding R4-4, prémisse corrigée) : `insert_with_defaults` ne désigne d'office que
les rôles `Receivable`, `DefaultRevenue` et `Payable` (`company_invoice_settings.rs` ~`:562-590`) — soit
**`1100` et `2000`** pour la créance et les créanciers ; **`2200` et `1171`** existent dans les trois
plans et sont **choisis par l'utilisateur** (`admin-manual.tex:2026` : comptes « ajoutés aux plans »,
non désignés). Les quatre sont des **feuilles** dans les trois plans
(`crates/kesh-core/assets/charts/*.json`, `parentNumber`, vérifié par script en P4) : aucune société
fraîchement créée n'est bloquée.

**Risque d'interblocage — l'ordre et le mode des verrous** (réécrit en P2, findings F2-1 HIGH = R2-1
et F2-8, choix **C43** ; **déplacé en 15-5e** au découpage, choix C52/C53 ; **règle révisée par C54** ;
**mode révisé par C87**, finding F5-1 HIGH de la P5). La règle est celle de la 15-5e1
(`15-5e1-socle-rejeu.md`) : un ordre conventionnel des verrous n'exclut pas les interblocages — aucun
ordre « comptes avant exercice » ne tient de bout en bout, l'insertion des lignes reprenant chaque
compte écrit par la clé étrangère `fk_jel_account` après l'exercice — et **la défense est le rejeu
des routes**. Cette story n'affirme donc ni absence de cycle, ni baisse de fréquence : elle énumère
ci-dessous les cycles examinés. Ce qui revient à **cette** story, et seulement à elle :

| flux | ce que la 15-5d ajoute | sa place |
|---|---|---|
| validation d'une facture | **créance + TVA due** (`ORDER BY id LOCK IN SHARE MODE`, accesseur) | `(2 bis', suite)` : après l'arrondi (fin de `(2 bis')`, `invoices.rs:2093`), avant `(2 ter)`, `(2 quater)` et l'exercice `(3)` (`:2200`) |
| saisie / complétion fournisseur | **créanciers + TVA récupérable** (`ORDER BY id LOCK IN SHARE MODE`, accesseur) | `(2, désignés)` (C87) : après les comptes de charge `(2, suite)` (`supplier_invoices.rs:377-426`), avant l'exercice `(3)` (`:427-430`) — réglages avancés en `(2 bis)` par la 15-5e1 (`:369-375`) |

*(Lignes relues sur `cecd5d1d`, après le merge de la 15-5e1, à l'alignement sur le livré (C85) ;
étiquettes celles des doc-comments canoniques livrés, qui font foi ; T0 les refait — après le merge de
la 15-8b, qui décale la vente de cinq lignes.)*

**Les cycles examinés** (C87 ; établis par lecture du code de `cecd5d1d` et la sémantique des verrous
InnoDB — **aucun n'a été reproduit**). À la vente, la validation tient, dans l'ordre : la facture et la
ligne des réglages (exclusif), le compte d'arrondi s'il y a un écart (exclusif), **la créance et la TVA
due (partagé)**, le compte de produit matérialisé en `(2 quater)` (partagé), puis attend l'exercice. À
l'achat, la saisie tient la ligne des réglages et les comptes de charge (exclusif), **les créanciers et
la TVA récupérable (partagé)**, puis attend l'exercice.

*Les lettres* (finding R6-3 de la P6) sont héritées des passes successives et citées par C87 : elles
ne se renumérotent pas. (e) et (f) n'existent plus (cycles d'un état antérieur de la fiche, retirés) ;
**(c)** et **(c')** sont deux cycles distincts — (c) à la vente, règlement **client** par compte
interne ; (c') à l'achat, règlement **fournisseur** par compte interne ; **(a bis)** est le lot de
paiement et l'acceptation par lot, examinés en P6.

*Ne se forment plus, ou pas :*

- **(a) validation ↔ flux qui prennent l'exercice puis reprennent la créance ou la TVA due par
  `fk_jel_account`** — règlement client par virement (`settle_invoice`, exercice `:211`, puis
  `create_in_tx` `:225` avec « C créance »), solde du reste (créance lue sans verrou `:445-455`,
  reprise après l'exercice `:509`), rapprochement (acceptation d'une proposition de facture), avoir
  (créance et TVA due par la clé étrangère après l'exercice) ; à l'achat, **saisie ↔ règlement
  fournisseur par virement** (`pay_in_tx`, exercice `:745`, puis « D créanciers »). C'est le cycle
  **systématique** de F5-1 sous un verrou exclusif ; deux verrous partagés étant compatibles, le
  règlement obtient sa clé étrangère et va au bout. **Test 4** de l'AC7 (le seul qui rougit si
  l'accesseur repasse en `FOR UPDATE`).
- **(a bis) lot de paiement pain.001 et acceptation par lot du rapprochement** (examinés en P6 par la
  lentille F, finding F6-1 ; établis par lecture, non reproduits) — même forme que (a), « exercice
  puis partagé », sans aucun exclusif sur les quatre comptes désignés :
  - **lot pain.001** (`payment_batches::confirm_batch`, `payment_batches.rs:304`) : lot `FOR UPDATE`
    (`:316`), contrôle du compte bancaire source par lecture simple (`:331-333`), puis par facture
    `supplier_invoices::pay_in_tx` (`:360`) en `SettlementChoice::BankTransfer` **seul** (`:364`) :
    facture `FOR UPDATE`, `bank_accounts … FOR UPDATE` (`supplier_invoices.rs:674-675`), exercice
    (`find_open_covering_date`, `:745`), puis les lignes — créanciers en partagé par `fk_jel_account`.
    Le lot n'emploie jamais `InternalAccount` : aucun `FOR UPDATE` sur la ligne `accounts` des
    créanciers ni de la TVA récupérable. Que ses factures 2..n soient verrouillées après l'exercice de
    la première est une inversion **préexistante**, indépendante de cette story ;
  - **acceptation par lot du rapprochement** (`accept_batch`, `crates/kesh-api/src/routes/reconciliation.rs:1052`,
    savepoints dans une transaction rejouée par `retry_with`, `:868-888`) : l'exercice
    (`find_open_covering_date`, `:1568`) reste tenu pour la suite des propositions ; la créance est
    lue sans verrou sur l'écriture de vente puis reprise en partagé par la clé étrangère. Le seul
    exclusif sur un compte pris après un exercice tenu est le **compte d'arrondi**
    (`rounding_account_for_write`, `:1535`, à la proposition suivante) — cycle préexistant, couvert
    par **#536** (15-5e2), et qui n'est pas un des quatre comptes de la garde.
  Aucun cycle neuf : le partagé de l'accesseur est compatible avec les partagés de ces deux flux.
- **(b) validation ↔ solde du reste sur la TVA due** (F2-1) — le solde prend la nature, l'arrondi
  éventuel, puis la TVA due **en exclusif** avant l'exercice (`invoice_settlements_write.rs:441-505`) ;
  la validation prend l'arrondi, puis la TVA due en partagé, avant l'exercice : même ordre « arrondi,
  puis TVA due », et celui qui attend ne tient rien que l'autre demande. Le cycle que formerait une
  garde **après** l'exercice ne se forme pas. **Test 1** (place avant l'exercice).
- **(c') saisie ↔ règlement fournisseur par compte interne = compte créanciers** — `pay_in_tx` tient
  ce compte en exclusif (`supplier_invoices.rs:712-714`) puis attend l'exercice ; la saisie attend ce
  compte sans tenir l'exercice. **Test 2**.
- **(g) validation ↔ validation, saisie ↔ saisie** sur les comptes désignés : partagés compatibles (et
  la ligne des réglages, exclusive, ordonne déjà deux flux d'une même société).

*Restent possibles, rares, et rejoués par les routes (15-5e1 pour la validation et la saisie, 15-5e2
pour la complétion d'import) :*

- **(c) validation d'une facture arrondie ↔ règlement client par compte interne = la créance ou la TVA
  due, avec écart d'arrondi** (R3-1 = F3-1) — le règlement tient ce compte en exclusif
  (`invoice_settlements_write.rs:157-160`) puis demande l'arrondi (`:195-207`) ; la validation tient
  l'arrondi et demande ce compte : un exclusif contre un partagé, le mode n'y change rien. **Aucune
  place ne ferme à la fois (b) et (c)** : un accesseur avant l'arrondi fermerait (c) mais ouvrirait le
  symétrique de (b) contre un solde du reste avec écart (arrondi, puis TVA due). La place après
  l'arrondi suit le solde du reste (C53) ; (c) exige un choix de compte interne inhabituel (la créance
  ou la TVA due elles-mêmes), un écart d'arrondi **et** une validation de facture arrondie simultanés.
  **Test 3** fige la place après l'arrondi.
- **(b') validation ↔ solde du reste dont le compte de la nature (escompte, frais, perte) est aussi un
  compte de produit des lignes de la facture validée** — le solde tient la nature en exclusif puis
  demande la TVA due ; la validation tient la TVA due en partagé puis reprend le compte de produit
  (en `(2 quater)` ou à l'insertion). Né de l'accesseur (la validation ne tenait pas la TVA due avant
  lui) ; exige une configuration où la nature d'un solde **est** un compte de vente des lignes.
- **(d) trois parties, dont une modification du compte dans le plan** — la validation tient la créance
  en partagé et attend l'exercice ; un `accounts::update`, un archivage ou une création de sous-compte
  sur la créance attend son exclusif ; un règlement tient l'exercice et demande la créance en partagé,
  qu'InnoDB met en file **derrière** l'exclusif en attente. Exige une modification du compte de
  créance au même instant.

*Examinés en P6, ni chemin de défaut ni cycle* (axe déclaré non exercé par la lentille R ; relu au
code de `eef701ac`) :

- **le « remplacement du plan comptable »** — la plage que la lentille R désignait ainsi
  (`accounts.rs:1070-1216`) est en fait `accounts::delete_all_by_company` (`:1063-1086` : `UPDATE
  accounts SET parent_id = NULL WHERE company_id = ?`, puis `DELETE`), suivie du `mod tests` (`:1088-1089`).
  Cette fonction n'a **aucun appelant** dans le dépôt (`grep -rn "accounts::delete_all_by_company"
  crates` vide ; son doc-comment dit « utilisé par reset_demo », ce qui est faux — la même erreur que
  documente son homonyme, `journal_entries.rs:1800-1801`) : aucun chemin. `kesh_seed::reset_demo`
  (`crates/kesh-seed/src/lib.rs:233`) supprime bien tous les comptes, mais **en autocommit**, sans
  transaction (une instruction après l'autre sur une connexion dédiée) : chaque `DELETE` attend le
  partagé de l'accesseur sans rien tenir, il ne peut pas fermer de cycle. Le chargement d'un plan
  (`bulk_create`, `bulk_create_from_chart`, `:908`, `:977`) n'écrit que des `INSERT`, à l'onboarding
  d'une société qui n'a encore ni comptes ni réglages : aucune ligne désignée n'est touchée ;
- **le désarchivage** (`accounts::reactivate`, `:788` ; `UPDATE accounts SET active = TRUE …`,
  `:856-859`) — ses lectures préalables (le compte, le parent, le détenteur du rôle,
  `find_singleton_role_holder`, `:742`) sont **sans verrou** : la transaction ne tient rien avant son
  `UPDATE`, qui attend le partagé de l'accesseur comme celui de l'archivage ; elle ne peut pas être un
  maillon de cycle (un attendeur qui ne tient rien). Ni chemin de défaut : un compte désarchivé
  **avant** le verrou de l'accesseur est lu actif, et c'est vrai ; **après**, son `UPDATE` attend le
  commit ; un compte que la validation lit inactif est refusé (`InactiveOrInvalidAccounts`), ce qui
  était vrai à l'instant du verrou. Le cas (d) ne s'y forme pas non plus : un compte archivé n'est la
  cible d'aucun flux qui le reprenne en partagé (`create_in_tx` le refuse avant l'insertion).

**Entre les comptes d'une même requête**, l'ordre est celui des identifiants (`ORDER BY id`, finding
R1-6 de la P1). Les comptes de charge (type `Expense`) et les comptes désignés côté achat (passif,
actif) sont disjoints par type. `accounts::create` (qui rend le parent non imputable) ne prend aucun
verrou sur les réglages ni sur les factures (vérifié en P4 de la 15-5b, lentille F) ; son `UPDATE` du
parent attend le verrou partagé de l'accesseur, c'est ce qui le rend suffisant (AC1).

**L'inversion préexistante réglages / exercice** (finding F2-8, relevée en P2) : levée par la
**15-5e1** (son AC5, **livrée** : étape `(2 bis)`), qui avance les réglages de la saisie fournisseur
avant les comptes de charge. Cette story en a besoin — pour verrouiller les candidats avant
l'exercice, la saisie doit connaître les réglages avant l'exercice — et T0 le vérifie.

### Pourquoi le compte créanciers passe à l'écran ici, et pas dans la 15-5b

C25 l'avait écarté « sans nécessité pour fermer le défaut » (#521). C27 a créé cette nécessité : un refus
qui renvoie à un champ invisible est une **impasse**, pire que le défaut silencieux qu'il remplace —
avant la garde, la saisie postait (à tort) sur le compte de regroupement ; après, elle serait bloquée
sans recours à l'écran (findings R4-1/F4-1, HIGH). La garde et l'écran vont donc **ensemble**, dans la
même story (choix C34). La 15-5b garde l'AC19 (#521), qui reste nécessaire pour les clients qui
n'envoient pas le champ.

### Ce qui doit être préservé

- Le contrat d'erreur de la 15-5a : `ACCOUNT_NOT_POSTABLE` n'est émis que pour un compte **de la
  société, actif** ; `details` par `NonPostableAccounts::details()` seul (C29).
- L'exemption « inchangé » de l'AC11 de la 15-5b, à la désignation : ce n'est **pas** une tolérance à
  l'usage, et la garde de l'AC1 ne la contourne pas — un réglage inchangé reste enregistrable ; il est
  refusé quand un flux veut y écrire.

### Hors périmètre, et écrit

- **L'avoir** (C35) : créance → 15-6a (#473, #523) ; TVA due → #525. Le passage du manuel `:1219` (`:1155` sur `67c31c95`)
  (« l'inverse exact ») relève des mêmes issues.
- Le **compte bancaire à l'usage** (D-A0, C6) et le **compte de produit par défaut** (D3-bis).
- L'alignement du solde du reste (`ConfigurationRequired`) sur ce refus (C28).
- Le **rejeu sur interblocage** de toutes les routes qui écrivent au journal, et ce qui reste de
  l'ordre des verrous des flux existants : les **15-5e1** et **15-5e2** (C52, réécrite selon C54,
  découpée par C61). Les inversions **exercice puis arrondi**, préexistantes — le lot de
  rapprochement et l'avoir d'une facture arrondie — y sont couvertes par le rejeu (**#536**, fermée
  par la 15-5e2).

### Décisions consignées (registre `epic-15-choix-autonomes.md`)

- **C27** — garde **à l'usage** des comptes de réglage (créance, TVA due, créanciers, TVA
  récupérable) ; l'exemption « inchangé » ne vaut qu'à la désignation. Compatible avec D-A0.
- **C28** — forme du refus : variante `DesignatedAccountsNotPostable`, même code et même `details`,
  message propre qui dit où agir (texte révisé par **C36**).
- **C29** — `NonPostableAccounts::details()` (15-5a), seul constructeur du JSON `rejected`.
- **C33** — le découpage : la garde à l'usage sort de la 15-5b.
- **C34** — le compte créanciers exposé à l'écran (révise **C25**).
- **C35** — l'avoir exempté de la garde à l'usage, avec sa vraie raison.
- **C36** — le message : « un compte imputable », « un administrateur doit ».
- **C39** — le prédicat : les générateurs rendent les rôles de réglage effectivement écrits ; la garde
  ne contrôle que ceux-là.
- **C40** — le dédoublonnage : confié à `NonPostableAccounts::new` (15-5a), figé par un test.
- **C41** — l'AC4 prouvé par un test Vitest de l'écran de validation ; les deux `catch` fournisseurs
  par lecture consignée.
- **C42** — révision de C39 : l'avoir n'appelle pas les générateurs de la garde ; il a le sien.
- **C43** — l'ordre des verrous : tous les candidats verrouillés avant l'exercice, contrôle des seuls
  rôles écrits après la génération ; côté achat, les réglages avancés avant les comptes de charge.
- **C44** — `DesignatedRole`, type neuf plutôt qu'`AccountRole`.
- **C45** — les contournements E2E restent (le seed ne désigne pas le compte créanciers) ; le seed
  n'est pas étendu (révise C34).
- **C46** — le dialogue de validation se ferme sur `ACCOUNT_NOT_POSTABLE`.
- **C47** — le signal de découpage D5 de la P2 (MEDIUM → HIGH, recyclé) ne la découpe pas ; « si la
  P3 recycle encore, découpage » (voir le Change Log de la P3).
- **C48** — « l'arrondi d'abord » : le règlement client par compte interne et le solde du reste
  verrouillent le compte d'arrondi avant leur autre compte (révise C43, qui les disait conformes) ;
  appliqué par la **15-5e** depuis le découpage (C52), puis **retiré** par sa réécriture (C54, C55).
- **C49** — l'accesseur refuse lui-même un compte absent ou inactif (`InactiveOrInvalidAccounts`),
  avant le non imputable.
- **C50** — la signature des générateurs : `GeneratedLines { lines, roles }`.
- **C51** — un identifiant d'une autre société : test de non-verrouillage, `owned_account_ids` si
  besoin (**tranché par C88** : adopté d'emblée), `EXPLAIN` relevé.
- **C52** — la clause de C47 joue : l'ordre des verrous des règlements sort en 15-5e, qui passe avant.
- **C53** — la ligne de partage : la 15-5e réordonne les flux existants et porte leurs tests ; la
  15-5d garde la place de son accesseur et les trois tests qui la figent.
- **C54** — la défense contre l'interblocage est le rejeu (15-5e réécrite) ; les verrous de cette
  story restent, ses affirmations d'absence de cycle sont retirées.
- **C65** — cette story dépend de la 15-5e1 seule ; le rejeu de la complétion d'import vient avec la
  15-5e2, ordre de merge libre (**révisé par C66** : la saisie fournisseur est rejouée par la 15-5e1 ;
  **et par C68** : le Pattern 5 renvoie aux doc-comments canoniques que l'AC9 met à jour).
- **C85** — alignement sur le livré (15-5e1, 15-8a, 15-8b) : étiquettes `(2 bis', suite)` (vente) et
  `(2 ter)` (achat ; **révisée par C87** en `(2, désignés)`) pour les comptes désignés, présentes au
  doc-comment **et** au code ; rien de la
  15-5c à retirer ; `sitesTotal` recompté sur l'état rebasé.
- **C87** — verrou **partagé** (`LOCK IN SHARE MODE`) de l'accesseur, qui révise le `FOR UPDATE` de
  C43 (cycle systématique de F5-1) ; cycles examinés écrits aux Dev Notes ; tests de place réécrits
  (verrou concurrent tenu par le test), ancien test 1 retiré, test de mode ajouté ; étiquette d'achat
  `(2, désignés)` (révise C85, item 4) ; la 15-5d se développe après le merge de la 15-8b ; codes et
  montages corrigés (F5-2, F5-4 = R5-3).
- **C88** — remédiation de la P6 : cycles (a bis) — lot pain.001 et acceptation par lot — examinés,
  remplacement du plan et désarchivage examinés (ni chemin ni cycle) ; **dérogation écrite** au
  découpage sur le signal D5 (F5-1 recyclé), déclarée au Project Lead ; patron `owned_account_ids`
  adopté d'emblée (tranche C51) ; montage de vente sous `disable_rounding_to_5_centimes` ; `NOWAIT`
  et son code `1205` ; doc-comment d'`attendre_une_requete_en_cours` ; rubrique `### Ajouté` à
  créer ; verrous d'intervalle ; refus de l'identifiant étranger testé.

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/{company_invoice_settings,invoices,supplier_invoices,invoice_settlements_write}.rs`
(le dernier pour une phrase du commentaire « 5 bis », AC9 — réécrit en place par la 15-5e1, retouché
par la 15-5e2 ;
`invoices.rs`, `supplier_invoices.rs` et `credit_notes.rs` aussi pour les 19 tests des générateurs,
C50),
`crates/kesh-db/src/errors.rs` (variante), `crates/kesh-api/src/errors.rs` (bras),
`crates/kesh-db/src/test_fixtures.rs` (une phrase du doc-comment d'`attendre_une_requete_en_cours`,
AC7, finding R6-2),
`crates/kesh-i18n/locales/*/messages.ftl` (deux clés : `error-designated-account-not-postable`,
`settings-invoicing-payable-account`), `crates/kesh-i18n/src/loader.rs` (sélecteur),
`frontend/src/routes/(app)/settings/invoicing/{+page.svelte,settings-invoicing-page.test.ts}`,
`frontend/src/lib/features/invoices/invoices.types.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`
(borne), `frontend/tests/e2e/{payment-batches,inbox-import,supplier-invoices}.spec.ts` (AC6 :
commentaires des contournements), `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (AC4 :
fermeture du dialogue, C46) et, au besoin, les autres écrans de facture (AC4, si un `catch` est en
défaut), tests
(`crates/kesh-db/tests/{invoices_validate_vat,supplier_invoices_repository}.rs` (dont les tests de
place 1 et 2 et le test de mode, C87), `crates/kesh-db/tests/invoice_amount_due_parity.rs` (test 3 de
place du verrou), et `crates/kesh-db/tests/credit_notes_repository.rs` pour le test de C35 ; `crates/kesh-api/tests/{company_invoice_settings_postable_e2e,inbox_import_e2e}.rs` ;
`frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts`, neuf),
`docs/manual/fr/{user-manual,admin-manual}.{tex,pdf}`, `docs/api-external.md`, `CHANGELOG.md`.
**Aucune migration** (P1–P8 sans objet).

**Décompte des modules** (règle de splitting préventif ; recompté en P1, finding F6) : `kesh-db` (trois
dépôts, erreurs), `kesh-api` (erreurs, deux fichiers de tests), `kesh-i18n` (quatre locales, chargeur),
`frontend` (réglages, types, test de l'écran de validation, borne i18n), manuels (deux) — **cinq** en
comptant les specs Playwright avec le `frontend` et `docs/api-external.md` / `CHANGELOG.md` comme
compagnons de documentation ; **six ou sept** si l'on compte à part `frontend/tests/e2e` et les
documents de `docs/`. Le seuil (« plus de 5 modules ») est donc atteint ou franchi selon la convention
de compte. Le signal est **déclaré** au Change Log (amendement D5) et la dérogation écrite ci-dessous (C88) ; il ne déclenche pas de découpage :
la story porte une **seule règle métier** (la garde à l'usage) et l'écran qui la rend praticable,
inséparables (C34), et elle est revue en **passes complètes**.

### Dérogation règle de splitting

*(Écrite en P6, finding F6-2 ; choix **C88**. `CLAUDE.md` § « Règle de splitting préventif »,
amendement D5.)*

**Le signal, tel qu'il est.** Le critère de découpage D5 est le **recyclage** — « un défaut qui revient
sous une autre forme, ou qui naît du correctif précédent ». Il est **rempli** : le HIGH **F5-1** de la
P5 (le `FOR UPDATE` des comptes désignés avant l'exercice, cycle systématique avec les flux qui
reprennent ces comptes par `fk_jel_account`) est **né d'une remédiation** — C43, qui posait ce verrou
en réponse au F2-1 de la P2 — et le thème « ordre des verrous » est revenu à trois passes : **P2**
(F2-1 = R2-1), **P3** (R3-1 = F3-1, F3-2), **P5** (F5-1). L'exception de l'amendement (défauts distincts
**et** non issus d'une remédiation) ne s'applique pas. S'y ajoute le seuil des modules, atteint ou
franchi selon la convention de compte (ci-dessus). Le constat de la P5 (« traité localement, pas de
découpage ») décrivait l'étendue du correctif, non la nature du défaut : il ne répondait pas au
critère, et c'est ce que cette section corrige.

**Pourquoi ne pas découper.** La seule coupe disponible sépare l'**écran du compte créanciers et les
contournements E2E** (AC5, AC6) de la garde (AC1-AC4, AC7-AC9) : elle est propre — la dépendance C34
ne joue que dans le sens « l'écran avant la garde » —, mais elle **ne porte pas l'axe recyclé**.
L'ordre et le mode des verrous sont au **cœur** de la garde — l'accesseur, sa place, ses tests de place
et de mode —, et resteraient tous du même côté de la coupe. Découper réduirait la fiche sans toucher
la cause du recyclage ; la coupe précédente du thème (15-5e, puis 15-5e1/15-5e2) a déjà sorti tout ce
qui relevait des **autres** flux, et ce qui reste ici est la place de **ce** verrou, indivisible.

**Risque accepté.** Une remédiation de l'ordre ou du mode des verrous peut encore faire naître le
défaut suivant. Mitigations : P6 est la **dernière passe complète** ; la suivante est une **passe
ciblée** (une lentille, sur le seul commit de cette remédiation) ; le rejeu des routes (15-5e1, 15-5e2)
reste la défense contre tout interblocage résiduel (C54) ; si une passe ciblée trouve encore un défaut
de verrou né d'une remédiation, la coupe AC5/AC6 est appliquée sans nouvelle délibération et le
reste est soumis au Project Lead.

**Signal déclaré au Project Lead** (Guy) — par le registre des choix de fin d'epic
(`epic-15-choix-autonomes.md`, **C88**), à lui présenter avec la revue finale de l'Epic 15.

### Tests — ce qui rendrait un test vert sans rien prouver

- Un compte de test **désigné après** avoir été rendu non imputable : la désignation (AC10 de la 15-5b)
  le refuserait, et le test prouverait la garde de la 15-5b, pas celle-ci. Désigner, **puis** basculer
  `postable`.
- Asserter `400` sans le code ni `details.rejected` : un refus pour une autre raison passerait.
- Le test « TVA arrondie à zéro » doit utiliser un taux **positif** : un taux nul ne distingue pas les
  deux prédicats.
- Le test « même compte pour deux rôles » doit asserter `count = 1` **et** un seul élément dans
  `details.rejected` : un `400` seul passerait aussi avec un doublon. Et le test « deux comptes » doit
  utiliser deux comptes **distincts** : sur un compte commun, il prouverait le dédoublonnage, pas la
  réunion des deux rôles dans un seul refus.
- Le test Vitest du compte créanciers doit porter sur un compte **non imputable** : avec un compte
  imputable, l'affichage marcherait sans `withCurrentAccount`.
- Un test de place dont la tâche n'est **jamais vue en attente** : `attendre_une_requete_en_cours` rend
  `false` quand la tâche finit sans attendre ; ne pas asserter ce retour laisserait passer un accesseur
  sans verrou (C87). Et le test de mode doit attendre **sur l'exercice**, non sur les comptes : c'est
  la seule forme qui distingue le partagé de l'exclusif.

### References

- Issues : #429 (et son commentaire) ; #473, #523, #525 (avoir) ; #521 (15-5b).
- Fiches : `15-5-gardes-postabilite-serveur.md` (mère, `split`), `15-5a-refus-non-imputable.md`,
  `15-5b-gardes-surfaces-neuves.md`, `15-5e1-socle-rejeu.md` (le rejeu de la validation et de la
  saisie fournisseur, l'avance des réglages, les doc-comments canoniques — dépendance),
  `15-5e2-rejeu-des-autres-flux.md` (le rejeu des autres routes, dont la complétion d'import — sans
  dépendance), `15-5e-ordre-des-verrous-reglements.md` (index, `split`) ;
  `14-3b-consommateurs-roles.md` (D-A0, L2).
- `CLAUDE.md` : § *Inventorier les sites NON RÉSOLUS*, § *Le prompt d'une passe doit NOMMER le
  manuel*, § *Test Locally First* (exception `kesh-db`), § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), agent de développement, worktree `/home/gcorbaz/devel/kesh-15-5d`, branche
`story/15-5d-garde-usage-comptes-reglage` partie d'`origin/main` `ec675288` (+ `c005b074`, planification). Cible cargo
propre : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5d/target`. Bases dédiées : `kesh_155d` (gate, migrée et seedée),
`kesh_e2e_155d` (E2E, migrée), remises à zéro par `DROP`/`CREATE` (jamais de redémarrage du conteneur).

### Debug Log References

- Gate backend : `scratchpad/gate155d.log` ; Vitest : `scratchpad/vitest155d.log` ; E2E : `scratchpad/e2e155d/run1.log`
  (répertoire de session de l'orchestrateur, non versionné) ; journaux de mutation : `scratchpad/mut155d/*.log`.

### Completion Notes List

**T0 — relevés sur `HEAD` (`c005b074` : 15-5a, 15-5b, 15-5c, 15-5e1, 15-8a, 15-8b mergées).**

- *15-5e1 en place* : rejeu `"invoices::validate"` (`routes/invoices.rs:880`) et `"supplier_invoices::create"`
  (`routes/supplier_invoices.rs:363`) par `kesh_db::retry::retry_on_deadlock` ; `(2 bis)` (réglages) avant les comptes
  de charge (`supplier_invoices.rs:369-375`) ; doc-comments canoniques `invoices.rs:1921-1970` et
  `supplier_invoices.rs:249-291` ; « 5 bis » `invoice_settlements_write.rs:474-486`. **15-5e2 non mergée** (branche en
  développement ailleurs) : la complétion d'import porte le verrou de l'accesseur sans être encore rejouée — fenêtre
  annoncée par la fiche.
- *Cases d'insertion* : vente, fin du bloc `let rounding` (`invoices.rs:2095-2105`), avant `(2 ter)` `:2107`,
  `(2 quater)` `:2146` et `(3)` `:2204` ; achat, entre `(2, suite)` (`:377-425`) et `(3)` (`:427-430`). Aucun écart.
- *Générateurs* : `invoices.rs:1789` (branche TVA `:1856`), `_rounded` `:1884` (transmission `:1891`), achat
  `supplier_invoices.rs:105` (branche `:129`) ; appels de production `invoices.rs:2280`, `supplier_invoices.rs:439` ;
  `credit_notes.rs:933` (test 16-1a) ; l'avoir a son générateur (`credit_notes.rs:187`, appelé `:507`, réglages
  `:362`, `create_in_tx` `:548`). `grep … | grep -v "///"` : **25** occurrences, même ventilation (3 / 1 / 2 / 19).
- *Lecteurs des quatre comptes* (`grep -rnE "default_(receivable|payable|vat_payable|vat_recoverable)_account_id"
  crates/*/src`, hors module des réglages et tests) : `invoices.rs` (validation), `supplier_invoices.rs` (saisie),
  `credit_notes.rs` (avoir, exempté — AC3), `vat_report.rs:174` (lecture seule), `exports/csv_tables.rs` (lecture
  seule), commentaires de `post_restore.rs` et `errors.rs`. **Aucun lecteur qui écrive hors AC1/AC3.**
- *E2E* : `grep -rn "defaultPayableAccountId == null" frontend/tests/e2e` → **3** sites (`payment-batches.spec.ts:59`,
  `inbox-import.spec.ts:93`, `supplier-invoices.spec.ts:86`).
- *(a bis) confirmé* : `payment_batches::confirm_batch` n'appelle `pay_in_tx` qu'en `SettlementChoice::BankTransfer`
  (`payment_batches.rs:364`) ; l'acceptation par lot prend l'arrondi (`routes/reconciliation.rs:1535`) puis l'exercice
  (`:1568`) ; aucun `FOR UPDATE` neuf sur un des quatre comptes après l'exercice
  (`grep -rn "FROM accounts.*FOR UPDATE" crates/kesh-db/src/repositories` : seuls les `singleton_role` de
  `insert_with_defaults`).
- *`EXPLAIN` à la main* (base de gate peuplée de 1000 comptes sur deux sociétés, puis remise à zéro) :
  `SELECT id, number, active, postable FROM accounts WHERE company_id = 1 AND id IN (2, 3) ORDER BY id LOCK IN SHARE MODE`
  → **`range` sur `PRIMARY`**, *Using where*, aucun `filesort` ; un seul identifiant → **`const`**. Même plan sur la
  base à cinq comptes. Lecture non verrouillante des identifiants : `range` sur `PRIMARY`.
- *Mesures* (MariaDB **10.11.16**, deux sessions) : une sonde `SELECT … FOR UPDATE NOWAIT` contre une ligne tenue en
  partagé rend **`ERROR 1205`** (*Lock wait timeout exceeded*) — confirmé ; un partagé sur `IN (8, 12)` (PRIMARY,
  point par point) laisse **libres** les lignes 9 (autre société) et 10 (même société) entre les deux ; la forme
  fautive `company_id = 1 AND id IN (8, 9)` (9 étranger) **verrouille** la ligne étrangère (`1205`) — d'où le patron
  `owned_account_ids`. ⚠️ **Mesure neuve, qui contredit la lettre de l'AC7** : `SELECT id FROM accounts WHERE id IN
  (…) LOCK IN SHARE MODE` passe par l'index secondaire couvrant `fk_accounts_parent` et **ne verrouille pas** la clé
  primaire (sonde `FOR UPDATE NOWAIT` réussie) — voir test 4 ci-dessous et **C-15-5d-1**.
- *Texte laissé par la 15-5b* : `user-manual.tex:382` — « Quatre cas échappent encore à ce contrôle […] (4)~un compte
  \textbf{désigné dans les réglages de facturation} et devenu non imputable après sa désignation reste utilisé par
  les écritures automatiques (validation d'une facture, avoir, facture fournisseur). Si vous scindez un tel compte, un
  administrateur peut désigner à sa place l'un de ses sous-comptes, dans \emph{Paramètres} → \emph{Facturation}. » ;
  `user-manual.tex:394` (note des comptes de clôture : « Les cas qui échappent encore au contrôle — un compte devenu
  non imputable \emph{après} avoir été choisi — sont énumérés à la section «~Rôles des comptes~». ») ;
  `admin-manual.tex:2027` — « un compte non imputable est refusé au moment où vous le désignez --- un compte déjà
  désigné, devenu non imputable depuis, ne bloque pas l'enregistrement des autres réglages. » Le motif `Quatre cas`
  du grep de T6 retrouve le premier ; **ajouté au grep** : `reste utilisé par les écritures automatiques`.
- *Montage de `company_invoice_settings_postable_e2e.rs`* : `setup` (`:226-283`) — société nue, Admin, douze comptes,
  `PUT` des réglages, jeton Admin forgé (`forge_admin_jwt`) ; ni exercice, ni contact, ni taux : non employé (AC7).

**T1 — la garde.** `DesignatedRole` (4 variantes, `SALE` / `PURCHASE`, `designated_id` par `match` exhaustif,
`candidate_ids`), `GeneratedLines { lines, roles }` (`#[derive(Debug)]`), `DesignatedAccountsSnapshot`,
`lock_designated_accounts_in_tx` (lecture non verrouillante des identifiants de la société, puis `SELECT id, number,
active, postable … ORDER BY id LOCK IN SHARE MODE` sur eux seuls) et `check_written` (absent ou inactif →
`InactiveOrInvalidAccounts` ; sinon non imputables → `DesignatedAccountsNotPostable(NonPostableAccounts::new(…))`),
dans `company_invoice_settings.rs`, en `pub(in crate::repositories)`. Générateurs adaptés (rôle de TVA inséré dans la
branche `total_vat > 0`), 19 tests adaptés mécaniquement (`.lines`). Appels : `(2 bis', suite)` à la vente,
`(2, désignés)` à l'achat, mêmes étiquettes au code et aux doc-comments canoniques ; contrôle entre la génération et
`create_in_tx`. Variante `DbError::DesignatedAccountsNotPostable` (`ACCOUNT_NOT_POSTABLE`), bras HTTP 400 par
`account_not_postable_response`, partagé avec la variante de la 15-5a (C-15-5d-3). *Refus de l'accesseur vérifié à la
lecture* (tests « archivé » et « autre société », qui prouvent le résultat, non l'auteur) : `check_written` rend
`InactiveOrInvalidAccounts` dès qu'un rôle écrit n'a pas de ligne dans l'instantané ou une ligne `active = FALSE`,
**avant** `create_in_tx` — la lecture du code le confirme (`company_invoice_settings.rs`, `match self.0.iter().find(…)`,
bras `_ => return Err(DbError::InactiveOrInvalidAccounts)`).

**T2 — le message.** `error-designated-account-not-postable`, quatre locales, `[one]` / `*[other]`, inscrite à
`SELECTEURS_RESOLUS_COTE_SERVEUR` ; test `designated_account_not_postable_resolves_singular_and_plural_in_every_locale`
(singulier, pluriel, menu de chaque locale, « administrat/amministrat », texte FR exact au singulier et au pluriel).

**T3 — les écrans.** Lecture des trois `catch` : `invoices/[id]/+page.svelte` (`confirmValidate`, `:373`) affiche
`err.message`, branche propre à `CONFIGURATION_REQUIRED` (`:409`) que `ACCOUNT_NOT_POSTABLE` n'emprunte pas ;
`ACCOUNT_NOT_POSTABLE` **ajouté** à la liste de fermeture (`:430-431` sur la base) — C46. `supplier-invoices/+page.svelte:213-214`
(`if (isApiError(err)) formError = err.message`) et `supplier-invoices/import/+page.svelte:261`
(`completeErrorLabel`, `default: return err.message` `:300`) : `err.message` tel quel, aucune branche ajoutée.
Compte créanciers à l'écran : `<select>` `data-testid="settings-payable-account"` juste après la créance,
`withCurrentAccount(liabilityAccounts, payableId, accounts)`, lu au chargement et à la relecture sur conflit, envoyé ;
types `InvoiceSettingsResponse` / `UpdateInvoiceSettingsRequest` ; mock du test. Clé
`settings-invoicing-payable-account` (quatre locales). **Borne `sitesTotal` : 1913 → 1915** (+2, `grep -o 'i18nMsg('`
sur la page des réglages aux deux bornes : 47 → 49 ; la fiche facture reste à 93) ; `sitesNonResolus` 31, `relais` 6,
`sitesGabarit` 10, `litterauxMin` 1050, `clesDepuisTsMin` 5 inchangés (le test passe avec ces valeurs).

**T4 — E2E.** Les trois contournements gardés, leur commentaire réécrit (« le seed `with-company`
(`seed_accounting_company`) ne désigne pas le compte créanciers »). Jugés sur la suite complète (T7) : les trois specs
vertes (`supplier-invoices` 3/3, `payment-batches` 1/1, `inbox-import` 1 passé, 2 `skip` de conception).

**T5 — les tests** (périmètre : de `ec675288` au commit de développement `43205b2c`).
- `kesh-db`, `invoices_validate_vat.rs`, module `garde_usage_comptes_reglage` (**12** tests) : créance non imputable ;
  TVA due avec / sans TVA / arrondie à zéro ; deux comptes distincts (`count = 2`) ; un compte pour deux rôles
  (`len = 1`, un seul élément de `details.rejected`) ; ordre — compte de produit explicite (`InvalidRevenueAccounts`,
  `NotPostable`), TVA due absente (`ConfigurationRequired`), exercice clos (`FiscalYearInvalid`) ; archivé
  (`InactiveOrInvalidAccounts`) ; avoir exempté (C35) ; autre société (sonde + refus, C-15-5d-2) ; **test de place 1** ;
  **test de mode 4**. Montage sous `disable_rounding_to_5_centimes`.
- `kesh-db`, `supplier_invoices_repository.rs`, module `garde_usage_comptes_reglage` (**4**) : créanciers ; TVA
  récupérable avec / sans TVA ; exercice clos ; **test de place 2**.
- `kesh-db`, `invoice_amount_due_parity.rs` (**1**) : **test de place 3** (arrondi avant les comptes désignés).
- `kesh-i18n` (**1**) ; `kesh-api` (**2**) : `company_invoice_settings_postable_e2e.rs` (validation par un Comptable,
  `init_error_i18n` ajouté à `spawn_app`), `inbox_import_e2e.rs` (complétion : 400, staging `to_complete`, 0 facture).
- Vitest (**5**) : `settings-invoicing-page.test.ts` (+3 : affichage d'un compte non imputable en place, envoi,
  relecture sur conflit) ; `invoice-validate-page.test.ts` neuf (+2 : Comptable et Admin).
- Total recompté : **20** tests Rust neufs (12 + 4 + 1 + 1 + 2 ; gate 2834 → 2854) et **5** Vitest (1086 → 1091).
- Sondes `NOWAIT` jugées sur `1205` par `test_fixtures::sonde_verrou_nowait` (toute autre erreur panique) ;
  doc-comment d'`attendre_une_requete_en_cours` réécrit (R6-2).

**Mutations** (gate ciblé `test(garde_usage_comptes_reglage) | test(place_3)`, 17 tests ; fichier restauré par copie
puis `touch` après chacune ; `git status` propre vérifié) — toutes **rouges** :

| mutation | tests rouges |
|---|---|
| M1 rôle `Receivable` non inséré | créance ; deux comptes ; place 1 |
| M2 rôle `VatPayable` non inséré | TVA due avec/sans ; deux comptes |
| M3 rôle `Payable` non inséré | créanciers ; place 2 |
| M4 rôle `VatRecoverable` non inséré | TVA récupérable |
| M5 contrôle retiré (vente) | 5 tests de vente |
| M6 contrôle retiré (achat) | 3 tests d'achat |
| M7 verrou de vente après l'exercice | place 1 ; autre société |
| M8 verrou d'achat après l'exercice | place 2 |
| M9 verrou de vente avant l'arrondi | place 3 |
| M10 accesseur en `FOR UPDATE` | **mode 4** (après correction C-15-5d-1 ; avant, il restait vert) ; places 1 et 2 (motifs) |
| M11 accesseur sans verrou | places 1 et 2 ; autre société (témoin) |
| M12 patron `owned_account_ids` retiré | autre société (sonde `1205`) — conforme à la mesure d'`opening_complement.rs` |
| M13 `VatPayable` hors de la branche `total_vat > 0` | TVA due (cas « arrondie à zéro » et « sans TVA ») |
| M14 `NonPostableAccounts::new` sans dédoublonnage | un compte pour deux rôles |

Frontend : F1 (`ACCOUNT_NOT_POSTABLE` dans la branche `CONFIGURATION_REQUIRED`) → 2/2 rouges ; F2 (retiré de la
fermeture) → 2/2 ; F3 (compte non envoyé) → 1 ; F4 (non relu sur conflit) → 1 ; F5 (sans `withCurrentAccount`) → 3.
Comme écrit à l'AC7, les tests « archivé » et « autre société » (refus) ne sont **pas** attendus rouges sous la
suppression de la branche « absent ou inactif » : `create_in_tx` rendrait le même refus.

**T6 — manuel, API, CHANGELOG.** `user-manual.tex` : encadré *Rôles des comptes* (garde à l'usage, exceptions de
l'avoir et du compte de produit par défaut, « Trois cas »), note des comptes de clôture (renvoi), § *Validation d'une
facture* (paragraphe « Un compte des réglages devenu non imputable »), § *Saisir une facture fournisseur* (compte
créanciers, TVA récupérable, refus ; `\label{sec:saisie-facture-fournisseur}`), § *Importer des factures* (renvoi) ;
`admin-manual.tex` § *Configuration des comptes TVA* (blocage à l'usage) + paragraphe *Compte créanciers*. PDF
régénérés (`make -C docs/manual fr`, 0 référence indéfinie), contrôlés aplatis (`pdftotext -nopgbrk | tr | sed` +
ligatures et apostrophe typographique normalisées) : phrases neuves présentes (7 dans le manuel utilisateur, 3 dans
l'administrateur), « Quatre cas », « reste utilisé par les écritures automatiques », « Si vous scindez un tel compte »
absents ; valeur `Quatre` absente du `.tex`, `Trois cas` une fois. Brochure régénérée mais **non commitée** (source
inchangée). `docs/api-external.md` : ligne `ACCOUNT_NOT_POSTABLE` du § 10 (validation, saisie, complétion) ; table du
solde du reste « absent ou inutilisable — archivé, non imputable ». `CHANGELOG.md` `[0.13.0]` : rubrique **`### Ajouté`
créée en tête** (compte créanciers) ; entrée #427/#429 (« Limites ») et entrée #521 (« n'envoyait pas ce compte … voir
*Ajouté* ») réécrites ; entrée *Corrigé* neuve avec la ligne d'action. Commentaire « 5 bis » : une phrase. Doc-comments :
accesseur (deux temps, pourquoi partagé, `owned_account_ids`, verrous d'intervalle, ce qui n'est pas contrôlé, L2
révisée), `validate_invoice` (`(2 bis', suite)`, disjonction par type, « # Erreurs » : ordre des refus),
`supplier_invoices::create_in_tx` (`(2, désignés)`, phrase des cycles complétée). **Grep du symptôme** (motifs de T6
+ `reste utilisé par les écritures automatiques`) : restent `user-manual.tex:1234` (« l'inverse exact », passage de
l'avoir — non réécrit, #473/#525, **signalé**), des « inverse exact » sans rapport (`credit_notes.rs:611`,
`supplier_invoice_scan_qr_e2e.rs:9`, migration `20260729000001`, `invoices/[id]/+page.svelte:954`), des
« n'est pas exposé(e) » sans rapport (`reconciliation_rule.rs:105`, `test_fixtures.rs:44`, `reconciliation_e2e.rs:570`),
« sous-compte imputable » d'un test de la 15-5b (`bank-accounts-page.test.ts:62`), et les mentions `#429`
d'attribution (stories 15-5a, 15-5b, 15-5d) — toutes justes.

**T7 — gates, au commit de code `43205b2c`** (dernier commit de code ; le commit suivant ne porte que la fiche, le
registre et `sprint-status.yaml`) :
- base `kesh_155d` remise à zéro (`DROP`/`CREATE`, migrations, seed), attente des gates des autres agents
  (`wait-kesh.sh`), **`scripts/test-fast.sh`** (fmt + clippy `-D warnings` + nextest) : **2854 passés / 2854, 4
  ignorés**, 125 s ;
- frontend : `npm run check` 0 erreur (27 avertissements, aucun sur les fichiers touchés),
  `lint-i18n-ownership` PASS, **`test:unit` 112 fichiers, 1091 / 1091**, `build` vert ;
- **E2E complet** : backend `target/debug/kesh-api` (cible du worktree) sur le port **3006**, base `kesh_e2e_155d`,
  secrets générés (`openssl rand`), `KESH_COOKIE_SECURE=false`, `KESH_STATIC_DIR=frontend/build` du worktree, SMTP
  factices, répertoires inbox/documents de session ; runner `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` : **247 passés, 7 échecs, 19 `skip`** — les 7 sont exactement les
  KF-029 de `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150`) ; aucun échec hors liste. Backend arrêté après la suite.

**Signalés, non traités** (AC3, AC4, AC8) :
- le **type** d'un compte désigné n'est pas contrôlé à l'usage (angle mort préexistant ; le compte d'arrondi, lui, le
  contrôle — `usable_designated_account`) : à aligner dans une autre story ;
- `INVOICE_LINE_REVENUE_ACCOUNT_INVALID` ne ferme pas le dialogue de validation ;
- le passage de l'avoir du manuel (`user-manual.tex:1234`, « l'inverse exact … la créance client, le produit et la TVA
  due ») relève de #473 et #525 ;
- hors périmètre, relevé au développement : le piège de l'index couvrant (C-15-5d-1) vaut pour toute bloqueuse de test
  écrite `SELECT id … LOCK IN SHARE MODE` sur `accounts`.

**Choix consignés** : C-15-5d-1 à C-15-5d-4 (`epic-15-choix-autonomes.md`).

**Revue de code P1 — remédiation** (Claude Opus 5.5, worktree `/home/gcorbaz/devel/kesh-15-5d`,
`CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5d/target`, bases `kesh_155d` et `kesh_e2e_155d`). Commit de code
`a8bab77b` ; aucun commit sur `origin/main` depuis `ec675288` (`git fetch` le 2026-10-08) : rebase sans objet.
- *`EXPLAIN` de la requête épinglée* (`kesh_155d`, B-1, C-15-5d-5) : `SELECT id, number, active, postable FROM accounts
  FORCE INDEX (PRIMARY) WHERE company_id = 1 AND id IN (2, 3) ORDER BY id LOCK IN SHARE MODE` → `range` sur `PRIMARY`,
  `key_len` 8, *Using where*, aucun `filesort` ; un seul identifiant → `const`. Index secondaires de `accounts` relevés :
  `uq_accounts_company_number` (`company_id`, `number`), `uq_accounts_company_singleton_role`, `fk_accounts_parent`.
- *Tests neufs* (périmètre `43205b2c` → `a8bab77b`) : **2** — `archived_account_wins_over_a_non_postable_one`
  (`invoices_validate_vat.rs`, priorité C49 en mélange) et `mode_purchase_lock_is_shared`
  (`supplier_invoices_repository.rs`, bloqueuse lisant `name`, montage vérifié par sondes). Gate 2854 → **2856**.
- *Mutations* (gate ciblé `test(garde_usage_comptes_reglage) | test(place_3)`, 19 tests ; fichier restauré par copie
  puis `touch`) : M15 `check_written` qui nomme les non imputables avant de refuser l'inactif → priorité en mélange
  **rouge** ; M16 accesseur en `FOR UPDATE` → mode 4, mode d'achat, places 1 et 2 **rouges** ; M17 `FORCE INDEX` retiré
  → places 1 et 2 **rouges** (par leur motif de requête, non par un verrou d'intervalle — C-15-5d-5).
- *Manuels* : PDF régénérés (`make -C docs/manual fr`, 0 référence indéfinie), aplatis : « L'avoir, lui, n'est pas
  soumis à ce contrôle, et c'est voulu » présent (utilisateur) ; « configuration requise »). L'usage côté comptable »
  contigu (administrateur). Brochure non modifiée.
- *Gates au commit de code `a8bab77b`* : bases remises à zéro (`DROP`/`CREATE`, migrations, seed sur `kesh_155d`),
  `wait-kesh.sh` ; **`scripts/test-fast.sh`** : **2856 / 2856, 4 ignorés**, 121 s ; frontend : `check` 0 erreur (27
  avertissements), `lint-i18n-ownership` PASS, **`test:unit` 112 fichiers, 1091 / 1091**, `build` vert ; **E2E
  complet** (port 3006, base `kesh_e2e_155d`, secrets générés, `KESH_COOKIE_SECURE=false`, SMTP factices,
  inbox/documents de session, runner `KESH_TEST_MODE=true`) : **247 passés, 7 échecs, 19 `skip`** — les 7 KF-029 de
  `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
  `:77`, `:150`), aucun hors liste. Journaux : `scratchpad/gate155d-p1.log`, `front155d-p1.log`, `e2e155d/run-p1.log`.

#### Intégration sur `origin/main` (`9cb5083b`, 15-5e2) — 2026-10-08

Rebase des huit commits de la story sur `origin/main` `9cb5083b` (la 15-5e2 ; la 15-11a, PR #564, n'y était pas).
Cible cargo du worktree : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5d/target`. Choix C-15-5d-7.

- *Conflits et résolutions* : fiche de la story (la version de `main` est une version de validation antérieure,
  ancêtre de celle de la branche — `41f41e60` ⊂ `1ae63831` — : version de la branche retenue) ; registre des choix
  (union : `C-15-5e2-1..7` puis `C88`, aucune entrée perdue) ; `sprint-status.yaml` (union des lignes `last_updated`,
  celle de la 15-5d renumérotée (19) ; YAML valide, aucune clé en double) ; commentaire « 5 bis » de
  `invoice_settlements_write.rs` (formulation de la 15-5e2 — « enveloppe `retry_on_deadlock` » — gardée, le
  paragraphe de la 15-5d sur le verrou partagé de la TVA due ajouté à sa suite) ; PDF des deux manuels (binaires :
  provisoirement ceux de `main`, puis **régénérés** par `make admin user` sur l'état rebasé). Fusionnés sans conflit et
  relus : doc-comments canoniques de `validate_invoice` (`(2 bis', suite)`) et de `supplier_invoices::create_in_tx`
  (`(2, désignés)`) — la 15-5e2 n'y a touché que des commentaires voisins (`update`, étape (0)) ; `CHANGELOG.md`
  `[0.13.0]` (`### Ajouté` de la 15-5d, entrées de la 15-5e2 sous `Corrigé`) ; `docs/api-external.md` § 10 ; les deux
  `.tex`. Aucune mention `retry_with` introduite par la story (grep du diff `origin/main..HEAD`).
- *Registre des routes* : la story ne touche pas `audit_route_registry.rs` ; la partition de `main` (22 / 0 / 4 / 89)
  est tenue par son test, vert au gate. *Compteurs i18n* : `sitesTotal` 1915 et `CANDIDATES_ATTENDUES` 48, la 15-5e2
  ne touchant pas le frontend ; verts au Vitest de l'état rebasé.
- *PDF* : contrôlés aplatis (`pdftotext | tr '\n' ' '`) : « rejouée automatiquement » (15-5e2) et « compte
  créanciers » ×4 (15-5d) dans le manuel utilisateur ; `innodb_deadlock_detect` (15-5e2) et le paragraphe *Compte
  créanciers* dans le manuel d'administration ; plus aucune occurrence de `SERIALIZABLE`.
- *Gates sur l'état rebasé* : bases `kesh_155d` et `kesh_e2e_155d` remises à zéro (`DROP`/`CREATE`, migrations, seed
  de `kesh_155d`), `wait-kesh.sh` ; **`scripts/test-fast.sh`** (fmt, clippy `-D warnings`, nextest) : **2859 / 2859,
  4 ignorés**, 128 s ; frontend : `check`, `lint-i18n-ownership`, **`test:unit` 112 fichiers, 1091 / 1091**, `build`
  verts ; **E2E complet** (backend `target/debug/kesh-api` du worktree, port 3006, base `kesh_e2e_155d`, secrets
  générés, `KESH_COOKIE_SECURE=false`, SMTP factices, inbox/documents de session, runner `KESH_TEST_MODE=true`) :
  **247 passés, 7 échecs, 19 `skip`** — les 7 KF-029 de `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`), aucun hors liste. Backend arrêté par
  son PID. Journaux : `target/gate-logs/integration-{backend,frontend,e2e}.log`.

#### Intégration sur `origin/main` (`8f9811d8`, 15-11a) — 2026-10-09

Rebase des neuf commits de la story sur `origin/main` `8f9811d8` (la 15-11a : compose de production, `.env.example`,
refus des gabarits de secrets dans `config.rs`, manuel d'administration, CHANGELOG avec une rubrique Sécurité).
Cible cargo du worktree : `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-15-5d/target`. Choix C-15-5d-8.

- *Conflits et résolutions* : registre des choix (deux fois — union : `C-15-11a-1..7` de `main`, puis `C88` et
  `C-15-5d-1..7` de la branche ; aucun titre `## ` perdu de part ni d'autre, aucun en double, contrôlé par `comm`
  sur les titres) ; `sprint-status.yaml` (deux fois — union des lignes `last_updated`, celles de la 15-5d
  renumérotées (21) et (22) au-dessus de celle de la 15-11a (20) ; YAML valide, aucune clé en double) ; PDF
  d'administration (binaire : provisoirement celui de `main`, puis **régénéré**). Fusionnés sans conflit et relus :
  `CHANGELOG.md` `[0.13.0]` (une seule rubrique de chaque : *Ajouté*, *Modifié*, *Corrigé*, *Sécurité* ; l'entrée
  *Ajouté* et les deux entrées *Corrigé* de la 15-5d en place), `admin-manual.tex` (paragraphe *Compte créanciers*
  de la 15-5d ; tableaux de `sec:env-vars` et sous-section « Passer à la 0.13.0 » de la 15-11a intacts),
  `user-manual.tex`.
- *PDF* : `touch` des deux `.tex` puis `make admin user` ; aucune référence indéfinie, aucun débordement aux lignes
  ajoutées par la story. Contrôlés aplatis (`pdftotext | tr '\n' ' '`) : « Compte créanciers. », « Passer à la
  0.13.0 » (×2) et `GENERATE_ME` (×8) dans le manuel d'administration ; « Trois cas échappent », « la saisie est
  refusée avec un message qui nomme le compte » et « La complétion crée cette facture par le même chemin » dans le
  manuel utilisateur ; aucun `??`.
- *Gates sur l'état rebasé* (`844aa3c4`, code identique au commit poussé) : bases `kesh_155d` et `kesh_e2e_155d`
  remises à zéro (`DROP`/`CREATE`, migrations, seed de `kesh_155d` ; conteneur non redémarré), `wait-kesh.sh` ;
  **`scripts/test-fast.sh`** (fmt, clippy `-D warnings`, nextest) : **2879 / 2879, 4 ignorés**, 112 s ; frontend :
  `check` 0 erreur (27 avertissements), `lint-i18n-ownership` PASS, **`test:unit` 112 fichiers, 1091 / 1091**, `build`
  vert ; **E2E complet** (backend `target/debug/kesh-api` du worktree, port 3006, base `kesh_e2e_155d`, secrets
  **générés** par `openssl rand` — la 15-11a refuse `GENERATE_ME` et `<…>` —, `KESH_COOKIE_SECURE=false`, SMTP
  factices, inbox/documents de session, runner `KESH_TEST_MODE=true`) : **247 passés, 7 échecs, 19 `skip`** — les
  7 KF-029 de `docs/testing.md` (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150`), aucun hors liste. Backend arrêté par son PID. Journaux :
  `target/gate-logs/integration2-{backend,frontend,e2e,e2e-backend}.log`.

### File List

- `CHANGELOG.md`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`
- `crates/kesh-api/tests/inbox_import_e2e.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/src/repositories/credit_notes.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/invoices.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/src/test_fixtures.rs`
- `crates/kesh-db/tests/invoice_amount_due_parity.rs`
- `crates/kesh-db/tests/invoices_validate_vat.rs`
- `crates/kesh-db/tests/supplier_invoices_repository.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `crates/kesh-i18n/src/loader.rs`
- `docs/api-external.md`
- `docs/manual/fr/admin-manual.tex`, `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte`
- `frontend/src/routes/(app)/invoices/[id]/invoice-validate-page.test.ts` (neuf)
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`
- `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
- `frontend/tests/e2e/{inbox-import,payment-batches,supplier-invoices}.spec.ts`
- `_bmad-output/implementation-artifacts/15-5d-garde-usage-comptes-reglage.md`,
  `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`,
  `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- 2026-10-08 — **Créée à la passe de validation P4 de la 15-5b** (choix C33), sur le finding **F4-3**
  (MEDIUM, lentille F : l'ancien AC20 de la 15-5b est une règle métier neuve, pas un rollout, et les
  deux HIGH de la passe en sont nés). Historique de ce contenu avant sa sortie :
  - **Validation P3 de la 15-5b** (lentilles Sonnet R et F) : finding F-1 (MEDIUM) — un réglage
    « inchangé » devenu non imputable restait utilisé sans contrôle à l'usage, le défaut de #429
    subsistait → AC20 ajouté à la 15-5b, choix **C27** (garde à l'usage) et **C28** (forme du refus) ;
    révision de la limite L2 de D-A0 pour quatre comptes.
  - **Validation P4 de la 15-5b** (prompt versionné `15-5b-validate-prompt-p4.md` ; lentilles **Opus**
    R et F, contexte frais) — findings qui portaient sur ce contenu, et leur sort ici :

  | finding | sév. | objet | sort | origine (amendement D5) |
  |---|---|---|---|---|
  | R4-1 = F4-1 | HIGH | le refus des **créanciers** renvoie à un champ que l'écran n'a pas : impasse sans recours | **AC5** : le champ exposé à l'écran (C34, révise C25) ; AC6 (E2E) ; AC7 (test Vitest du chemin de correction) | **né de la remédiation P3** (C27 contre C25) |
  | R4-2 = F4-2 | MEDIUM | l'avoir écrit sur la créance et la TVA due **des réglages du moment**, sans garde ; la fiche le disait « snapshot de la facture » | **AC3** : exemption délibérée, avec la vraie raison (créance → 15-6a, #473, #523 ; TVA due → #525) ; test qui la fige (AC7) ; AC8 (manuel) (C35) | **d'origine** (l'inventaire #11 date de la création de la 15-5) ; texte de l'AC17 (iv) né de la remédiation P3 |
  | F4-3 | MEDIUM | l'AC20 n'est pas du rollout ; la 15-5b n'a plus de justification pour son ampleur | **cette story** (C33) | **né de la remédiation P3** |
  | R4-3 = F4-4 | LOW | « porte de la TVA » inconnaissable avant la génération ; prédicat à fixer | AC1 : `total_vat > 0`, garde entre génération et `create_in_tx` ; test « TVA arrondie à zéro » | — |
  | F4-5 | LOW | place de la garde dans l'ordre des refus non fixée | AC1 : ordre écrit au doc-comment ; « tous les comptes **de réglage** » ; tests d'ordre (AC7) | — |
  | R4-4 | LOW | « les plans livrés désignent d'office 1100, 2200, 2000 et 1171 » faux pour 2200 et 1171 | Dev Notes corrigées | — |
  | R4-5 = F4-6 | LOW | « désignez-y un sous-compte » ; message servi à un Comptable sans accès aux réglages | AC2 : « un compte imputable », « un administrateur doit » (C36) | — |
  | F4-7 | LOW | sections du manuel où la validation et la saisie fournisseur listent leurs refus non visées | AC8 : `:847-849` et `:1285-1293` | — |
  | F4-8 | LOW | CHANGELOG : changement qui peut bloquer après mise à jour, pas seulement « Corrigé » | AC9 : ligne d'action | — |

  Les autres findings de la P4 (R4-6, R4-7, F4-9) portent sur ce qui reste dans la 15-5b : voir son
  Change Log. **Décisions de l'orchestrateur** : C33 à C35 ; **C36** ajouté pendant la remédiation.
  **Propagation post-patch** : `AC20`, `DesignatedAccountsNotPostable`, `designated-account`, `C27`,
  `C28`, `sous-compte imputable`, `reprend les comptes`, `n'est pas exposé`, `closes #429` grepés sur les
  fiches 15-5, 15-5a (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml` ; les
  occurrences restantes sont historiques (Change Logs), des renvois à la 15-5d, ou — pour « sous-compte
  imputable » — l'énoncé de la story 15-5a (`15-5a-refus-non-imputable.md:27`), qui porte sur la saisie
  manuelle et que cette passe ne touche pas (fiche en développement sur une autre branche, signalé à
  l'orchestrateur).
  Décompte : **9 AC, 8 tâches T0–T7** (recompté). **Validation : à lancer** — une passe complète, la
  story portant une règle métier et un écran neuf.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-5d-validate-prompt-p1.md` ; deux
  lentilles **Sonnet** en contexte frais : **R** auditeur d'acceptation, **F** adversaire de périmètre
  complet ; rotation D6 : la P2 sera Opus). **0 CRITICAL, 0 HIGH.** Bruts : R 3 MEDIUM + 5 LOW
  (R1-1 à R1-8), F 1 MEDIUM + 5 LOW (F1 à F6). Doublons inter-lentilles : F2 = R1-2, F3 ≈ R1-1 (F3
  ajoute la phrase du rapport TVA de l'AC3, traitée avec), F4 ⊂ R1-3 → **4 MEDIUM et 7 LOW
  distincts** (recompté : 14 bruts − 3 fusions = 11). Trend : **P1 4 MEDIUM / 7 LOW** (première passe
  de cette fiche ; la P4 de la 15-5b, d'où elle est née, est à l'entrée précédente).

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | R1-1 ≈ F3 | MEDIUM | AC4 : `catch` `:103` (scan QR) et `:133` (chargement) hors sujet ; écran de complétion d'import sans chemin ; AC3 : le rapport TVA ne lit pas le compte de décompte | AC4 : trois `catch` nommés (`invoices/[id]/+page.svelte:402-415`, `supplier-invoices/+page.svelte:213`, `supplier-invoices/import/+page.svelte:267`, `default:` `:301`), `:103` et `:133` écartés ; AC3 : `vat_report.rs:173-174` lit TVA due et récupérable |
  | R1-2 = F2 | MEDIUM | AC1 : « identifiants effectivement présents dans `entry_lines` » sans mécanisme — `total_vat` local au générateur, ambigu quand un compte joue deux rôles | AC1 : les générateurs (`invoices.rs:1784`/`:1851`, `supplier_invoices.rs:105`/`:129`) **rendent les rôles** effectivement écrits ; la garde ne contrôle que ces comptes ; appelants (25 occurrences) adaptés, l'avoir ignore les rôles (**C39**) |
  | R1-3 ⊃ F4 | MEDIUM | AC7 : complétion d'import sans fichier ni montage ; cas e2e sans route ni montage ; AC4 sans test | AC7 : `inbox_import_e2e.rs` (montage `:716`, patron `:792`) ; `company_invoice_settings_postable_e2e.rs` (montage de la 15-5b, `POST /api/v1/invoices/{id}/validate`, Comptable) ; Vitest `invoice-validate-page.test.ts` neuf (**C41**) |
  | F1 | MEDIUM | un même compte pour deux rôles → « Les comptes 2000, 2000 » ? | **C40** : dédoublonnage par `NonPostableAccounts::new` (15-5a, AC1 : « trie […] et **dédoublonne par identifiant** ») ; test « même compte pour deux rôles → nommé une fois, singulier » ; deux comptes **distincts** (`1100`, `2000`) pour le test « créance et TVA due » ; mutation |
  | R1-4 | LOW | AC6 : retrait jugé sur deux specs isolées | AC6, T4 : jugé sur la suite E2E complète |
  | R1-5 | LOW | mock `settings-invoicing-page.test.ts:67` sans `defaultPayableAccountId` | AC5, T3 |
  | R1-6 | LOW | ordre de verrouillage entre comptes non fixé | AC1 : une requête, `ORDER BY id FOR UPDATE` ; Dev Notes |
  | R1-7 | LOW | statut `ready-for-dev` contre « Validation : à lancer » ; lignes relevées avant la 15-5b | en-tête : convention du registre écrite ; AC5 : lignes relevées sur `67c31c95`, T0 les refait |
  | R1-8 | LOW | l'item du manuel que l'AC8 réécrit n'existe pas encore | T0 : relevé du texte laissé par la 15-5b, grep de T6 complété au besoin |
  | F5 | LOW | § *Importer des factures depuis un dossier* (`user-manual.tex:1374-1397`) non visé | AC8 : une phrase, renvoi à la saisie |
  | F6 | LOW | décompte des modules imprécis | Dev Notes : décompte écrit, signal déclaré (ci-dessous) |

  **Signal de la règle de découpage** : sévérité sans objet (première passe). **Décompte des modules** :
  cinq selon la convention de la fiche, six ou sept en comptant à part `frontend/tests/e2e` et les
  documents de `docs/` — le seuil est atteint ou franchi. **Constaté et déclaré** (amendement D5) ; pas
  de découpage proposé : une seule règle métier, et l'écran qui la rend praticable (C34) ne s'en
  sépare pas. Déclaré au Project Lead par l'orchestrateur, à qui revient l'arbitrage.
  **Décisions de l'orchestrateur** : mécanisme des rôles (C39), dédoublonnage par la 15-5a (C40), test
  d'écran de l'AC4 (C41).
  **Propagation post-patch** : `:103`, `:133`, `effectivement présents`, `vat_report.rs:174`, `le
  rapport TVA le lit`, `deux specs`, `sans le contournement`, `total_vat > 0` grepés sur les fiches
  15-5a (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml` — les occurrences
  restantes sont historiques (Change Logs de la 15-5b et de cette fiche) ou exactes (le prédicat
  `total_vat > 0`, désormais porté par la branche du générateur).
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P2 suit** (des MEDIUM en P1),
  complète (Opus) : la remédiation change le mécanisme de la garde.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-5d-validate-prompt-p2.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de la remédiation P1, **F**
  adversaire de périmètre complet ; rapports `target/gate-logs/15-5d-p2-{R,F}.md`). Bruts : R
  3 MEDIUM + 5 LOW (R2-1 à R2-8), F 1 HIGH + 1 MEDIUM + 6 LOW (F2-1 à F2-8), soit 16. Doublons
  inter-lentilles : F2-1 = R2-1 (retenu HIGH), F2-3 = R2-2 (retenu MEDIUM), F2-5 = R2-6 → **1 HIGH,
  3 MEDIUM, 9 LOW distincts** (recompté : 16 bruts − 3 fusions = 13). Trend : **P1 4 MEDIUM / 7 LOW
  → P2 1 HIGH / 3 MEDIUM / 9 LOW**.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | F2-1 = R2-1 | HIGH | la garde verrouille la TVA due **après** l'exercice, le solde du reste **avant** : cycle neuf (1213 → 500) ; Dev Notes et `invoice_settlements_write.rs:463-470` affirmaient l'inverse | AC1 : accesseur en deux temps — **tous les candidats verrouillés avant l'exercice** (`ORDER BY id`), contrôle des rôles écrits après la génération (C39 inchangé) ; côté achat, réglages avancés avant les comptes de charge ; ordre des refus réécrit (AC1) ; AC7 : test « exercice absent + compte non imputable », test de non-interblocage à deux connexions, mutation « verrou après l'exercice » ; AC9 : doc-comment « Ordre des locks » (`invoices.rs:1916-1925`) et commentaire « 5 bis » ; Dev Notes réécrites (table des quatre flux) (**C43**) |
  | R2-2 = F2-3 | MEDIUM | `credit_notes.rs:933`, cité comme l'avoir appelant du générateur, est un test ; l'avoir a son propre générateur | AC1 : ventilation des 25 occurrences (3 définitions, 1 transmission, 2 appels de production, 19 tests) ; `generate_credit_note_journal_lines` (`credit_notes.rs:187`) nommé ; révision de C39 (**C42**) |
  | R2-3 | MEDIUM | AC6 : le seed E2E (`seed_accounting_company`) ne désigne pas le compte créanciers ; le motif « contournement de #521 » est faux | AC6 réécrit : contournements gardés, commentaire corrigé, seed non étendu (**C45**, révise C34) |
  | F2-2 | MEDIUM | AC6 : un troisième contournement (`supplier-invoices.spec.ts:86`) | AC6, T0, T4 : inventaire par commande, trois sites ; fichier ajouté aux fichiers touchés |
  | R2-4 | LOW | C34 parle encore de « specs rejouées » | C45 (le registre ne réécrit pas C34) |
  | R2-5 | LOW | AC4 : `import/+page.svelte:267` / `:301` faux ; `:213` est le `catch` | `:261`, `:300`, `:213-214` (AC4, AC7) |
  | R2-6 = F2-5 | LOW | AC5 : « section des comptes par défaut, à côté de la TVA due » désigne deux sections | AC5 : section *Comptes par défaut* (`:296-342`), après la créance (`:300-314`) ; `:70-72`, `:123-145` |
  | R2-7 | LOW | `test_fixtures.rs:158-169` cité pour un `UPDATE` et un double rôle qu'il ne fait pas | AC1, AC7 : `supplier_invoices_repository.rs:49-55` |
  | R2-8 | LOW | mutation « retirer chacun des quatre contrôles » ambiguë sous C39 | AC7 : par rôle (ne pas l'insérer), par flux (retirer l'appel du contrôle), et le verrou déplacé |
  | F2-4 | LOW | `DesignatedRole` double quatre variantes d'`AccountRole` | AC1 : type neuf justifié (champs des réglages, `match` sans bras mort, `Ord`) (**C44**) |
  | F2-6 | LOW | le dialogue de validation reste ouvert sur `ACCOUNT_NOT_POSTABLE` | AC4 : le code ferme le dialogue (`:428-434`) ; AC7 : asserté, mutation ; T3 (**C46**) |
  | F2-7 | LOW | `docs/api-external.md:281` : « TVA due absente » alors que le solde du reste refuse aussi un compte inutilisable | AC9 : « absent ou inutilisable (archivé, non imputable) » |
  | F2-8 | LOW | inversion préexistante réglages / exercice entre validation et saisie fournisseur | **fermée** par la remédiation de F2-1 (réglages avancés côté achat) — Dev Notes ; **aucune KF** à ouvrir |

  **Signal de la règle de découpage** (amendement D5) : la sévérité **monte** (MEDIUM → HIGH). Le HIGH
  n'est pas un défaut d'origine neuf : il vient de la **place** de la garde, fixée par la remédiation
  de la P4 de la 15-5b (R4-3 / F4-5), et R2-2 naît de la remédiation P1 (texte de C39) — c'est du
  **recyclage** au sens de l'amendement. **Signal levé et déclaré** au Project Lead par
  l'orchestrateur, à qui revient l'arbitrage ; la remédiation ne propose pas de découpage : le défaut
  porte sur l'unique règle métier de la story (sa place dans la transaction), qu'aucune sous-story ne
  séparerait de son ordre de verrous. Décompte des modules inchangé (`invoice_settlements_write.rs`
  relève de `kesh-db`, l'écran de validation du `frontend`).
  **Décisions de l'orchestrateur** : ordre des verrous (C43, mécanisme imposé par l'orchestrateur) ;
  C42, C44, C45, C46 consignés pendant la remédiation.
  **Propagation post-patch** : `après ces verrous`, `check_designated_accounts_postable_in_tx`,
  `credit_notes.rs:933`, `ignore les rôles`, `ignorent les rôles`, `test_fixtures.rs:158-169`,
  `import/+page.svelte:267`, `` `:301` ``, `rejouées`, `retirés s'ils`, `le seed E2E le`, `à côté de la
  TVA due`, `:355-369`, `deux specs`, `aucun autre chemin ne prend` grepés sur les fiches 15-5a
  (lecture seule), 15-5b, 15-5c, 15-5d, le registre et `sprint-status.yaml`. Restent : la ventilation
  (qui nomme `credit_notes.rs:933` comme test), le commentaire de code cité pour être réécrit (AC9, et
  ajouté au grep de T6), les Change Logs (historiques), et dans le registre C34 et C39, non réécrits
  par règle et révisés par C45 et C42. La 15-5b (`:359`) dit que « la 15-5d décide de leur retrait
  (son AC6) » : toujours exact (elle décide de les garder), laissé tel quel — fiche close en
  validation.
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P3 suit** (un HIGH et des
  MEDIUM en P2), complète (**Sonnet**, rotation D6) : la remédiation change l'ordre des verrous de
  deux flux et le flux fournisseur.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-5d-validate-prompt-p3.md` ; deux
  lentilles **Sonnet** en contexte frais, rotation D6 : **R** chasseur de régressions de la
  remédiation P2 (commit `34b308c4`), **F** adversaire de périmètre complet ; rapports
  `target/gate-logs/15-5d-p3-{R,F}.md`). Bruts : R 1 MEDIUM + 3 LOW (R3-1 à R3-4), F 2 MEDIUM + 4 LOW
  (F3-1 à F3-6), soit 10. Doublon inter-lentilles : R3-1 = F3-1 → **2 MEDIUM, 7 LOW distincts**
  (recompté : 10 bruts − 1 fusion = 9 = 2 + 7). Trend : **P1 4 MEDIUM / 7 LOW → P2 1 HIGH / 3 MEDIUM /
  9 LOW → P3 2 MEDIUM / 7 LOW**.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | R3-1 = F3-1 | MEDIUM | le règlement client par compte interne verrouille le compte, **puis** l'arrondi, puis l'exercice ; la validation, l'arrondi puis la créance et la TVA due : cycle neuf si le compte interne est la créance ou la TVA due ; la table des flux omettait l'étape d'arrondi et C43 disait ce flux conforme | AC1 : règle « l'arrondi d'abord » — le règlement client classe le paiement avant l'étape (3) et prend l'arrondi avant le compte interne ; **le solde du reste aussi** (arrondi avant la nature), faute de quoi le réordonnancement ouvrait le cycle symétrique ; ordres des refus écrits ; AC7 : test 3 de non-interblocage (compte interne = TVA due, sonde `NOWAIT`), tests d'ordre des refus, mutation ; Dev Notes : table des flux réécrite (étape d'arrondi, rapprochement, avoir) ; AC9 : commentaire « 5 bis » réécrit et déplacé (**C48**, révise C43) |
  | F3-2 | MEDIUM | le test de non-interblocage ne couvre que la vente ; le déplacement du verrou côté achat ne rougit rien | AC7 : test 2 (règlement fournisseur par compte interne = compte créanciers ↔ saisie fournisseur à TVA) ; une mutation d'ordre **par flux** |
  | F3-3 | LOW | le lot de rapprochement prend l'exercice puis l'arrondi (préexistant) ; la fiche et le commentaire réécrit affirmaient l'absence de cycle | table des flux et AC1 : ligne « rapprochement », renvoi à **#536** (ouverte par l'orchestrateur) ; le commentaire réécrit ne prétend plus à l'absence de cycle (AC9) ; motif `aucun cycle n'est` ajouté au grep de T6 |
  | F3-4 | LOW | l'archivage était laissé au contrôle non verrouillant de `create_in_tx`, qui lit un instantané antérieur au verrou | AC1 : l'accesseur refuse lui-même un compte absent ou inactif (`InactiveOrInvalidAccounts`), puis le non imputable ; ordre écrit ; AC7 : limite du test « archivé » écrite (**C49**) |
  | F3-5 | LOW | `WHERE company_id = ? AND id IN (…) FOR UPDATE` : un identifiant étranger peut être verrouillé (mesuré à `opening_complement.rs:431-440`) | AC1 : `EXPLAIN` relevé, patron `owned_account_ids` si la mesure le demande ; AC7 : test d'un identifiant d'une autre société, sonde `NOWAIT` (**C51**) |
  | F3-6 | LOW | signature des générateurs non fixée, 19 tests en dépendent | AC1 : `GeneratedLines { lines, roles }`, vente `Result<GeneratedLines>`, achat `Result<(GeneratedLines, Decimal)>`, 19 tests adaptés mécaniquement par `.lines` ; T1 (**C50**) |
  | R3-2 | LOW | place du chargement des réglages côté achat non fixée | AC1 : après (0) projet et (1) fournisseur, avant (2) — ordre projet → réglages identique des deux côtés ; table |
  | R3-3 | LOW | en-tête des choix incomplet (C42, C45 seulement « révisé par » ; C47 absent) | en-tête et « Décisions consignées » complétés (C42 à C51) |
  | R3-4 | LOW | montage du test de non-interblocage sous-spécifié (compte d'escompte, facture validée avec TVA et reste dû) | AC7 : test 1 dans `invoice_write_off.rs`, montage `company()` (`:35`, escompte et arrondi désignés) et `validated_invoice()` (`:57`) ; facture B par le vrai chemin |

  **Relevé hors des rapports, pendant la remédiation** : le grep des sites qui verrouillent le compte
  d'arrondi (`rounding_account_for_write|write_off_account_for_write`) a trouvé un cinquième site,
  l'**avoir** d'une facture arrondie (`credit_notes.rs:370` exercice, puis `:525` arrondi) — même
  inversion que le rapprochement, préexistante, hors périmètre (l'avoir est exempté, C35). Écrit à la
  table et à l'AC1 comme **à tracer** ; **signalé à l'orchestrateur** (issue à ouvrir, ou #536 à
  étendre).
  **Signal de la règle de découpage** (amendement D5) : la sévérité **baisse** (HIGH → MEDIUM) : le
  signal « égale ou supérieure » n'est pas levé. Mais C47 a posé une condition propre — « si la P3
  recycle encore, découpage » —, et la réponse est **oui, en partie** : **F3-2** (MEDIUM) naît de la
  remédiation P2 (le changement d'ordre côté achat, introduit par C43, sans test qui le fige) ;
  **R3-1 = F3-1** (MEDIUM) ne naît **pas** d'elle — le cycle tient à la garde elle-même (verrouiller
  la créance ou la TVA due après le compte d'arrondi), il existait sous la place de la P1 comme sous
  celle de la P2 —, mais c'est la remédiation P2 qui en a **certifié l'absence** (C43, table, AC9) ;
  parmi les LOW, F3-3 et R3-2 portent sur du texte de la P2, F3-4 sur l'argument (a) de C43, R3-3 sur
  C47, R3-4 sur le test de la P2 ; F3-5 et F3-6 remontent à la P1. **Déclaré à l'orchestrateur, à
  qui revient l'arbitrage** au regard de C47 ; la remédiation ne propose pas de découpage (le défaut
  porte toujours sur l'ordre des verrous de l'unique règle métier). Décompte des modules inchangé
  (`invoice_settlements_write.rs`, désormais modifié pour son code, relève de `kesh-db`).
  **Décisions** : C48 (décision de l'orchestrateur, étendue au solde du reste pendant la
  remédiation), C49, C50, C51 (décisions de l'orchestrateur, consignées).
  **Propagation post-patch** : `aucun cycle connu`, `aucun cycle n'est`, `le suivent aussi`, `comme le
  règlement client`, `suit le chemin d'aujourd'hui`, `:195-197`, `quatre flux`, `test de
  non-interblocage rougit`, `ou le fichier des tests de solde`, `ConfigurationRequired` (pour les
  refus d'arrondi et de nature, qui sont `RoundingAccountNotConfigured` et
  `WriteOffAccountNotConfigured`) grepés sur les fiches 15-5 (mère), 15-5b, 15-5c, 15-5d, le registre
  et `sprint-status.yaml`. Restent : C43 (registre, « les règlements par compte interne le suivent
  aussi »), non réécrit par règle et révisé par C48 ; le commentaire de code cité pour être réécrit
  (AC9) ; les Change Logs (historiques). Les numéros de ligne cités par F (`:195-197` pour le
  classement du paiement) sont faux : `:175-176`, corrigé.
  Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). **Une passe P4 suit** (des MEDIUM en P3),
  complète (**Opus**, rotation D6) : la remédiation touche l'ordre de deux flux de plus (règlement
  client, solde du reste), donc du code de production d'un autre module de flux — la passe ciblée ne
  suffit pas.
- 2026-10-08 — **Découpage après la validation P3** (choix **C52** de l'orchestrateur, ligne de partage
  **C53**). La clause de C47 (« si la P3 recycle encore, découpage ») a joué : F3-2 naît de la
  remédiation P2, et la remédiation P3 a dû étendre le réordonnancement à deux flux de production de
  plus. Le réalignement des flux **existants** sur l'ordre canonique des verrous sort en
  **15-5e** (`15-5e-ordre-des-verrous-reglements.md`, `refs #429`), qui passe **avant** cette story.
  **Sorti de cette fiche** : la règle « l'arrondi d'abord » appliquée au règlement client par compte
  interne et au solde du reste (C48), avec leurs ordres de refus ; l'avance des réglages de la saisie
  fournisseur avant les comptes de charge (C43) ; la table des flux et les inversions préexistantes
  (#536, avoir) ; le test d'ordre des refus des flux réordonnés ; le test de non-interblocage 3
  (règlement client ↔ validation) ; les mutations de ces flux ; le commentaire de l'étape (3) de
  `settle_invoice` ; la réécriture et le déplacement du commentaire « 5 bis ». **Gardé ici** : la garde
  à l'usage (accesseur en deux temps, C39/C42/C49), `GeneratedLines` (C50), le test d'une autre
  société (C51), l'écran du compte créanciers, l'avoir exempté, le manuel, `docs/api-external.md`, le
  CHANGELOG, `closes #429` ; la **place** de l'accesseur (après l'arrondi ou les comptes de charge,
  avant l'exercice) et les tests qui la figent — 1 et 2, inchangés, et un **test 3 neuf** (sonde : la
  validation tient l'arrondi sans tenir la créance ni la TVA due), seul à rougir si l'accesseur passe
  avant l'arrondi ; la phrase que le commentaire « 5 bis » gagne (la validation verrouille désormais la
  TVA due avant l'exercice) et l'étape 1 bis du doc-comment canonique. Dépendance ajoutée : 15-5e ; T0
  vérifie que son ordre est en place. Les findings de la P3 relatifs à l'ordre (R3-1 = F3-1, F3-2,
  F3-3, R3-2) sont repris au Change Log de la 15-5e.
  **Propagation post-patch** : `settle_invoice`, `write_off_invoice`, `RoundingAccountNotConfigured`,
  `WriteOffAccountNotConfigured`, `test 3`, `trois flux`, `arrondi d'abord`, `C48`, `5 bis`,
  `1916-1925`, `réordonn`, `invoice_settlement.rs`, `f15e5fbf` grepés sur le corps de cette fiche
  (hors Change Log) ; les occurrences restantes sont des renvois à la 15-5e ou la place de
  l'accesseur. Décompte : **9 AC, 8 tâches T0–T7** (recompté). **Validation** : une passe ciblée sur
  le découpage (le corps a perdu un pan et gagné un test), après celle de la 15-5e.
- 2026-10-08 — **Alignement sur C54** (réécriture de la 15-5e : « Rejeu sur interblocage des flux
  d'écriture »). **Rien de la garde ni de ses verrous ne change** : accesseur en deux temps, places
  (après l'arrondi ou les comptes de charge, avant l'exercice), refus, tests de refus. Changé :
  (1) la dépendance à la 15-5e décrit ce qu'elle livre désormais — le rejeu des routes, l'avance des
  réglages de la saisie fournisseur (son AC5, ex-AC4), les commentaires réécrits ; « l'arrondi
  d'abord » du règlement client et du solde du reste n'existe plus ; (2) l'AC1 « l'ordre des
  verrous » et les Dev Notes ne prétendent plus fermer de cycle : les places **réduisent la
  fréquence**, la reprise des comptes par `fk_jel_account` après l'exercice (finding F1-1 de la P1
  de la 15-5e) laisse des cycles possibles, que le rejeu rend invisibles ; « Pourquoi après
  l'arrondi » ne s'appuie plus sur l'ancien AC2 de la 15-5e ; (3) les trois tests « de
  non-interblocage » deviennent des tests **de place du verrou** : le critère reste l'attente vue sur
  l'accesseur (ou les sondes du test 3) ; l'assertion « les deux tâches réussissent » des tests 1 et 2
  est remplacée par « succès ou 1213, jamais une autre erreur » — au niveau du dépôt, sans rejeu, une
  1213 y est possible ; le renvoi au test de concurrence 1 de la 15-5e (retiré) est supprimé ; (4) le
  commentaire « 5 bis » est réécrit **en place** par la 15-5e (son AC5) ; (5) T0 vérifie le rejeu des
  routes de validation, de saisie et de complétion ; (6) C54 ajouté aux décisions. **Propagation
  post-patch** : `non-interblocage`, `aucun cycle`, `cycle (iii) est fermé`, `les deux réussissent`,
  `les deux tâches réussissent`, `son AC4`, `son AC3`, `son AC2`, `son AC6`, `concurrence 1`,
  `que la 15-5e prévient` grepés sur le corps de la fiche (hors Change Log) : les occurrences
  restantes sont l'AC4 de la **15-5a** ou de la **15-5b** (pas de la 15-5e) et le rappel « que plus
  rien n'affirme ». Décompte inchangé : **9 AC, 8 tâches T0–T7** (recompté). La passe ciblée sur le
  découpage, encore à lancer, porte aussi sur cet alignement.
- 2026-10-08 — **Alignement sur la P2 de la 15-5e** (finding R2-1 de cette passe) : la 15-5e fixe,
  à son AC5, la numérotation du doc-comment canonique de `validate_invoice` sur les étapes du code
  — (1) facture, (1 bis) projet lu sans verrou, (2) réglages, (2 bis') compte d'arrondi, (2 quater)
  matérialisation du compte de produit, (3) exercice, … L'étape « 1 bis. `accounts` » que les
  points de l'AC1 (validation, « le verrou ») et de l'AC9 étendaient n'existe plus : ils renvoient à
  l'étape **(2 bis')**. Aucun changement de fond (place des verrous inchangée : après l'arrondi,
  avant l'exercice). Propagation : `1 bis` grepé sur la fiche — seule la mention historique du
  Change Log reste. Décompte inchangé : 9 AC, 8 tâches T0–T7.
- 2026-10-08 — **Alignement sur le découpage de la 15-5e** (choix **C61**, **C65** ; finding F3-2 de
  la P3 de la 15-5e) : la 15-5e est découpée en **15-5e1** (socle : enveloppes, registre, rejeu de la
  validation, du règlement client et de son annulation ; avance des réglages de la saisie
  fournisseur, doc-comment canonique, « 5 bis ») et **15-5e2** (rollout : les autres routes, dont la
  saisie fournisseur et la complétion d'import ; Pattern 5, manuels). Cette story **dépend de la
  15-5e1 seule** : en-tête, § des dépendances, T0 (vérifie la 15-5e1, relève sans l'exiger la
  15-5e2), renvois de l'AC1, de l'AC7, de l'AC9, des Dev Notes et des References ventilés vers la
  15-5e1 (contenu dont elle dépend) ou les 15-5e1 / 15-5e2 (rejeu des routes) ; la fenêtre
  « saisie fournisseur verrouillée sans rejeu » si cette story merge avant la 15-5e2 est écrite.
  Aucun changement de fond. Propagation : `15-5e` grepé sur le corps (hors Change Log) — les
  occurrences restantes sont historiques (C48, C52, C53, C54, finding F1-1 « de la P1 de la 15-5e »,
  relevé « avant les 15-5a, 15-5b et 15-5e ») ou le nom de la fiche index. Décompte inchangé :
  9 AC, 8 tâches T0–T7.
- 2026-10-08 — **Alignement sur la validation P4 des 15-5e1 et 15-5e2** (choix **C66**, **C68**) :
  (1) la **saisie fournisseur** est désormais rejouée par la **15-5e1**, avec l'avance de ses réglages
  (finding F4-1 de la P4 de la 15-5e1) — en-tête des dépendances, « L'ordre des verrous », T0 et
  References : seule la complétion d'import reste à la 15-5e2 ; (2) le Pattern 5 de la 15-5e2
  **renvoie** aux doc-comments canoniques de `validate_invoice` et de `supplier_invoices::create_in_tx`
  au lieu de recopier l'ordre (finding F4-3 = R4-1 de la P4 de la 15-5e2) : vérifié que l'AC9 de cette
  story met **déjà** ces deux doc-comments à jour quand elle y place ses verrous ; l'AC9 le dit
  désormais, et dit que cette story n'a pas à toucher le Pattern 5 — l'ordre de merge 15-5d / 15-5e2
  est sûr dans les deux sens. Aucun changement de fond. Propagation : `saisie fournisseur et la
  complétion`, `15-5e2 pour la`, `de la saisie fournisseur et de la complétion` grepés sur le corps
  (hors Change Log) : plus d'occurrence qui confie la saisie à la 15-5e2. Décompte inchangé : **9 AC,
  8 tâches T0–T7** (recompté).
- 2026-10-08 — **Alignement sur le livré (15-5e1, 15-8a, 15-8b)** (choix **C85**, sans passe de
  validation ; code relu sur `cecd5d1d`, qui intègre `origin/main` `de1e1c26` : 15-5c, 15-5e1 et 15-8a
  mergées). Sections modifiées :
  - **en-tête** (commentaire) : 15-5a, 15-5b, 15-5e1 mergées ; numéros relevés sur `cecd5d1d` ; C85
    ajouté aux choix applicables ;
  - **Dépend** : dépendance à la 15-5e1 **satisfaite** — ce qu'elle a livré (rejeu de
    `invoices::validate` et de `supplier_invoices::create` par `retry_on_deadlock(operation, f)`,
    avance des réglages en `(2 bis)`, doc-comments canoniques avec leurs **étiquettes réelles**,
    « 5 bis ») ; paragraphe 15-5c réécrit : **rien de la 15-5c n'est dans cette story** (ses
    libellés de `failed[]`, `failed-proposal-label.ts`, ne servent qu'au rapprochement, que la garde
    ne touche pas) ; `sitesTotal` (1904 sur `cecd5d1d`) à **recompter sur l'état rebasé** ; 15-8b
    (manuels, PDF) ;
  - **AC1** : ventilation des générateurs **recomptée sur `cecd5d1d`** (25, même ventilation ;
    appels de production `invoices.rs:2275`, `supplier_invoices.rs:439`) ; **place du verrou à la
    validation** précisée sur le code livré — juste après le bloc `let rounding` qui clôt `(2 bis')`,
    avant `(2 ter)`, `(2 quater)` et `(3)`, étiquette `(2 bis', suite)` ; **à la saisie
    fournisseur**, entre `(2, suite)` et `(3)`, étiquette neuve `(2 ter)` ; étiquettes présentes au
    doc-comment **et** au code (leçon A1 de la 15-5e1) ; numéros relocalisés (`:2036-2038`, `:2093`,
    `:2200`, `:2275-2281`, `:2283`, `:369-375`, `:377-426`, `:427-430`, `:432-436`, `:439-445`,
    `:446`, `:1915-1951`, `:712-714`/`:745`, `journal_entries.rs:117-121`,
    `opening_complement.rs:429-437`, `invoice_settlements_write.rs:135-140`) ;
  - **AC3** : `credit_notes.rs:501-505`, `:362-364` ;
  - **AC4**, **AC5**, **AC6** : numéros des écrans relus (`:401-416`, `:428-433`,
    `fr-CH/messages.ftl:513`, `company_invoice_settings.rs:52`, `:79`) ; greps de l'AC5 et de l'AC6
    refaits (mêmes résultats) ;
  - **AC7** : montage livré de `company_invoice_settings_postable_e2e.rs` (`spawn_app`, `setup`,
    `set_postable` ; jeton **Admin** seul — un jeton Comptable est à ajouter), `lib.rs:510` ;
    rubrique neuve « aucun test de rejeu dans cette story » (rejeu prouvé par les tests 4 et 5 de la
    15-5e1 ; témoin `tests/common/capture_rejeu.rs` si un tel test était ajouté) ; `:409-416`,
    `:428-433` ;
  - **AC8** : numéros du manuel relocalisés par le texte (`user-manual.tex:379-385`, `:393-395`,
    `:898`/`:911`/`:913`, `:1349`, `:1437`/`:1450`, `:1219` ; `admin-manual.tex:2018-2027`) ;
  - **AC9** : lignes à ajouter aux deux doc-comments canoniques livrés, avec leurs étiquettes ;
    conflit annoncé avec la 15-5e2 sur « 5 bis » ;
  - **T0**, **T3** ; **Dev Notes** (table « ce que la 15-5d ajoute / sa place » réécrite sur les
    étiquettes livrées, `invoice_settlements_write.rs:500-505`, inversion réglages / exercice
    livrée, « Fichiers touchés », « Décisions » : C85, `:1219` du passage de l'avoir).
  Aucune règle, aucun AC ni aucun test ne change de fond. **Propagation post-patch** : `:2065`,
  `:2172`, `:2247`, `:2255`, `:2008`, `1916-1929`, `:365`, `:372`, `337-346`, `639-662`,
  `487-491`, `:380`, `:390`, `847-849`, `1285`, `1374`, `2016-2025`, `:1155`, `aura réécrit`,
  `aura laissé`, `f289414e` grepés sur le corps (hors Change Log) : restent les rappels
  « (`:380`, `:390` sur `67c31c95`) » et « (`:1155` sur `67c31c95`) », voulus. **Décompte**
  (recompté, `grep -c`) : **9 AC, 8 tâches T0–T7** ; AC7 : 11 rubriques `kesh-db` (dont celle des
  trois tests de place), 2 fichiers `kesh-api`, 2 fichiers Vitest — inchangés.
- 2026-10-08 — **Passe de validation P5** (prompt versionné `15-5d-validate-prompt-p5.md` ; deux
  lentilles **Opus** en contexte frais : **R** chasseur de régressions de l'alignement C85 (commit
  `dc3ee63e`), **F** adversaire de périmètre complet ; rapports `target/gate-logs/15-5d-p5-{R,F}.md`).
  Bruts : R 1 MEDIUM + 5 LOW (R5-1 à R5-6), F 1 HIGH + 3 MEDIUM + 5 LOW (F5-1 à F5-9), soit 15.
  Fusions : F5-4 = R5-3 (retenu MEDIUM), F5-9 ⊂ R5-2 → **1 HIGH, 4 MEDIUM, 8 LOW distincts**
  (recompté : 15 bruts − 2 fusions = 13 = 1 + 4 + 8). Trend : **P1 4 MEDIUM / 7 LOW → P2 1 HIGH /
  3 MEDIUM / 9 LOW → P3 2 MEDIUM / 7 LOW → P4 ciblée (Haiku, découpage) 0 → P5 1 HIGH / 4 MEDIUM /
  8 LOW**.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | F5-1 | HIGH | le `FOR UPDATE` des comptes désignés avant l'exercice forme un cycle **systématique** avec les flux qui prennent l'exercice puis reprennent ces comptes en partagé (`fk_jel_account`) ; la fiche disait « réduisent la fréquence » | AC1 : verrou **partagé** `ORDER BY id LOCK IN SHARE MODE`, pourquoi il suffit (vérifié : l'archivage et le passage à non imputable écrivent la ligne par `UPDATE`) ; « réduit la fréquence » retiré ; Dev Notes : cycles examinés (ne se forment plus / restent / non examinés) ; AC7 : tests de place réécrits (verrou concurrent tenu par le test), ancien test 1 retiré, **test 4 de mode** ; mutations du mode ; AC9, T1 (**C87**) |
  | F5-2 | MEDIUM | le compte de produit d'une ligne non imputable rend `INVOICE_LINE_REVENUE_ACCOUNT_INVALID`, pas `ACCOUNT_NOT_POSTABLE` | AC1 (ordre des refus), AC4 (justification de C46 ; signalement du dialogue), AC7 (test d'ordre, compte explicite distinct du défaut) |
  | F5-3 | MEDIUM | deux phrases du CHANGELOG `[0.13.0]` deviennent fausses (`:23`, `:25`) | AC9 : réécrites ; T6 : motifs `garde à l'usage des comptes de réglage viendra`, `n'envoie pas ce compte`, `Quatre cas` |
  | F5-4 = R5-3 | MEDIUM | AC7 `kesh-api` : le montage livré ne permet pas de valider une facture | AC7 : `setup` non employé ; `seed_accounting_company`, `disable_rounding_to_5_centimes`, contact, Comptable (`users::create` + login), `init_error_i18n` — patrons nommés |
  | R5-1 | MEDIUM | « la 15-8b ne touche aucun fichier de la garde » est faux | en-tête, § dépendances, T0 : liste des fichiers communs ; **développement après le merge de la 15-8b** (ou rebase), relocalisation par le texte |
  | R5-2 ⊃ F5-9 | LOW | numéros périmés | `invoices.rs:1916-1957`, `loader.rs:373`, `api-external.md:321`, `opening_complement_repository.rs:736`, `supplier_invoices_repository.rs:1669`, `admin-manual.tex:2026` |
  | R5-4 | LOW | `(2 ter)` côté achat contre « (2 ter) ne prend aucun verrou » | étiquette **`(2, désignés)`** (AC1, AC9, Dev Notes ; C87 révise C85) |
  | R5-5 | LOW | inventaire des étiquettes du code incomplet | `(2 bis)` ajouté |
  | R5-6 | LOW | « ordre de merge sûr pour la documentation » | restreint à l'ordre des verrous ; fichiers communs avec la 15-5e2 et PDF à régénérer |
  | F5-5 | LOW | « devenu non imputable » ne vient pas que d'un sous-compte | AC1 ; ligne d'action du CHANGELOG |
  | F5-6 | LOW | « Quatre cas » du manuel | AC8 : amorce, énumération et phrase suivante reprises ensemble ; recompte de la valeur |
  | F5-7 | LOW | dérivés de `GeneratedLines` / `DesignatedRole` | AC1 : `Debug` ; `Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord` ; module `company_invoice_settings.rs` |
  | F5-8 | LOW | le type n'est pas contrôlé à l'usage | AC3 : angle mort assumé, signalé au Dev Agent Record |

  **Signal de la règle de découpage** (amendement D5) : la sévérité **monte** (P4 ciblée 0 → P5
  HIGH), et le HIGH est **recyclé** — thème « ordre des verrous » (P2 F2-1, P3 R3-1/F3-1 et F3-2, P5
  F5-1), **né de la remédiation C43**. **Déclaré au Project Lead** par l'orchestrateur ; traité
  localement (le mode d'un verrou, dans la seule fiche), **pas de découpage** — décision de
  l'orchestrateur, consignée en C87. Décompte des modules inchangé.
  **Décisions** : **C87** (verrou partagé, cycles examinés, tests réécrits, étiquette d'achat, ordre
  vis-à-vis de la 15-8b ; décisions de l'orchestrateur).
  **Propagation post-patch** : `FOR UPDATE` (corps) — restent le compte d'arrondi, les sondes
  `NOWAIT`, les bloqueuses des tests, l'exercice, `pay_in_tx`, les mentions de la révision ;
  `réduit la fréquence` / `réduisent la fréquence` — plus aucune hors Change Log ; `(2 ter)` — restent
  l'étape de `validate_invoice` (sans verrou), la mention du retrait par la 15-5e1 et le rappel
  historique de C85 ; `ACCOUNT_NOT_POSTABLE` à la validation — ne reste que le code de la garde ;
  `1915-1951`, `loader.rs:367`, `api-external.md:281`, `:717`, `:1442`, `admin-manual.tex:2025`,
  `succès ou 1213`, `trois tests` — plus d'occurrence hors Change Log, sauf C53 (résumé historique) et
  la description de l'ancien test retiré. Grepés sur le corps de cette fiche (hors Change Log) ; la
  fiche 15-5e2 n'est pas touchée (remédiée en parallèle par un autre agent) ; un grep en lecture
  seule (`FOR UPDATE` croisé avec `15-5d`, `désign`, `accesseur`) n'y trouve aucune mention du mode
  du verrou de l'accesseur.
  **Décompte** (recompté, `grep -c`) : **9 AC, 8 tâches T0–T7** ; AC7 : 11 rubriques `kesh-db` (la
  rubrique de place compte désormais **quatre** tests : trois de place, un de mode), 2 fichiers
  `kesh-api`, 2 fichiers Vitest. **Une passe P6 suit** (un HIGH et des MEDIUM en P5), **complète**
  (**Sonnet**, rotation D6) : la remédiation change le mode d'un verrou et réécrit des tests — la passe
  ciblée ne suffit pas.
- 2026-10-08 — **Passe de validation P6** (prompt versionné `15-5d-validate-prompt-p6.md` ; deux
  lentilles **Sonnet** en contexte frais, rotation D6 : **R** chasseur de régressions de la remédiation
  C87 (commit `41f41e60`), **F** adversaire de périmètre complet ; rapports
  `target/gate-logs/15-5d-p6-{R,F}.md`). Bruts : R 0 MEDIUM + 5 LOW (R6-1 à R6-5), F 2 MEDIUM + 3 LOW
  (F6-1 à F6-5), soit 10 ; aucune fusion → **2 MEDIUM, 8 LOW distincts** (recompté : 10 = 2 + 8).
  La lentille R a **exécuté** deux expériences sur `kesh-mariadb-dev` (10.11.16) : `NOWAIT` rend
  `1205` contre un partagé tenu, et un partagé demandé derrière un exclusif en attente est mis en file
  (prémisse du cycle (d)) ; elle ne trouve aucun cycle neuf. Trend : **P1 4 MEDIUM / 7 LOW → P2 1 HIGH
  / 3 MEDIUM / 9 LOW → P3 2 MEDIUM / 7 LOW → P4 ciblée (Haiku, découpage) 0 → P5 1 HIGH / 4 MEDIUM /
  8 LOW → P6 2 MEDIUM / 8 LOW**. Aucun des deux MEDIUM ne naît de la remédiation C87 : F6-1 est un axe
  laissé ouvert par la P5, F6-2 porte sur le traitement du signal D5.

  | finding | sév. | objet | sort |
  |---|---|---|---|
  | F6-1 | MEDIUM | « non examinés » : le lot pain.001 et l'acceptation par lot étaient renvoyés au dev alors qu'ils se tranchent à la lecture | Dev Notes : cycle **(a bis)** (lot pain.001 : jamais d'`InternalAccount`, exercice puis partagé ; acceptation par lot : exercice tenu puis créance en partagé, seul exclusif = compte d'arrondi, #536) — aucun cycle neuf ; **remplacement du plan et désarchivage** relus au code (axe non exercé de R) : `accounts::delete_all_by_company` sans appelant, `reset_demo` en autocommit, chargement d'un plan par `INSERT` seuls, `reactivate` sans verrou préalable — ni chemin de défaut ni cycle ; T0 **confirme** au lieu de relire |
  | F6-2 | MEDIUM | le critère D5 (recyclage) n'était pas appliqué à la lettre au HIGH F5-1 | section **« Dérogation règle de splitting »** (Dev Notes) : signal écrit tel qu'il est (F5-1 né de C43, thème en P2, P3, P5), coupe disponible AC5/AC6 nommée et écartée (elle ne porte pas l'axe recyclé), risque accepté, P6 dernière passe complète ; signal déclaré au Project Lead par le registre (**C88**) |
  | R6-1 | LOW | l'arrondi n'est pas fixé dans le montage des tests 1, 2, 4 | AC7 : tous les tests de vente sauf le test 3 sous `disable_rounding_to_5_centimes` (choisi plutôt qu'un montant multiple de 0.05 : le test « TVA arrondie à zéro » ne peut pas le tenir) ; achat sans objet (aucune étape d'arrondi) |
  | R6-2 | LOW | le doc-comment d'`attendre_une_requete_en_cours` dit qu'un `LOCK IN SHARE MODE` ne serait pas vu | AC7 : phrase réécrite (les motifs doivent figurer dans le texte de la requête) ; `test_fixtures.rs` ajouté aux fichiers touchés ; T5 |
  | R6-3 | LOW | lettrage des cycles ((e), (f) absents ; (c) et (c')) | Dev Notes : paragraphe *Les lettres* — lettres héritées, citées par C87, non renumérotées ; (c) et (c') distingués |
  | R6-4 | LOW | `NOWAIT` présenté comme un patron du dépôt | AC7 : **premier emploi**, erreur attendue `1205` (non `3572`), discrimination sur ce code ; les précédents cités sont ceux du helper |
  | R6-5 | LOW | test « autre société » : la décision `owned_account_ids` laissée au rouge | AC1 : patron **adopté d'emblée** (lecture non verrouillante dans la transaction, puis partagé sur les seuls identifiants possédés) ; AC7 : la sonde passe par construction, mutation « patron retiré » ajoutée ; C51 tranchée par C88 |
  | F6-3 | LOW | `[0.13.0]` n'a pas de rubrique `### Ajouté` | AC9 : la créer en tête de section (ordre de `[0.12.1]`) ; le renvoi de #521 n'est vrai qu'après |
  | F6-4 | LOW | verrous d'intervalle de `LOCK IN SHARE MODE` sur `IN (…)` non discutés | AC1 : phrase au doc-comment, type d'accès `EXPLAIN`, renvoi à C-15-8-23 et `journal_entries.rs:1176` ; T0 : `EXPLAIN` à la main ; T1 |
  | F6-5 | LOW | la branche « absent de l'instantané » de C49 n'a pas de test nommé | AC7 : le test « autre société » asserte aussi le refus `InactiveOrInvalidAccounts` ; même limite que le test « archivé » (résultat, non auteur), écrite comme angle mort assumé ; T5 |

  **Signal de la règle de découpage** (amendement D5) : la sévérité **baisse** (P5 HIGH → P6 MEDIUM)
  et les MEDIUM de la P6 ne sont pas recyclés ; le recyclage de la P5 reste, et la dérogation est
  désormais écrite (section dédiée, C88). **Décisions** : **C88** (décisions de l'orchestrateur).
  **Propagation post-patch** — grepés sur le corps de cette fiche (hors Change Log) : `non examin`,
  `les relit`, `S'il échoue`, `si ce test montre`, `si la mesure le demande` — plus aucune occurrence
  (C51 porte désormais « tranché par C88 ») ; `ne serait pas vu` — seule la citation du doc-comment à
  réécrire ; motif `FROM accounts WHERE company_id` élargi en `FROM accounts WHERE` (la requête
  verrouillante ne commence plus forcément par `company_id`) ; `NOWAIT` — chaque occurrence est une
  sonde, désormais jugée sur `1205`. Lignes citées relues au code de `eef701ac` (`accounts.rs:742`,
  `:788`, `:856-859`, `:908`, `:977`, `:1063-1086`, `:1088-1089` ; `payment_batches.rs:304`, `:316`,
  `:331-333`, `:360`, `:364` ; `supplier_invoices.rs:674-675`, `:745` ; `routes/reconciliation.rs:868-888`,
  `:1052`, `:1535`, `:1568` ; `opening_complement.rs:469-487` ; `journal_entries.rs:117-121`, `:1176`,
  `:1800-1801` ; `kesh-seed/src/lib.rs:233` ; `test_fixtures.rs:236`, `:540-558` ;
  `invoices_validate_vat.rs:153` ; `invoices.rs:2093`). ⚠️ **Hors périmètre, signalé** : le
  doc-comment d'`accounts::delete_all_by_company` (`accounts.rs:1063`, « utilisé par reset_demo ») est
  faux, comme l'était celui de son homonyme de `journal_entries.rs` — à l'orchestrateur.
  **Décompte** (recompté, `grep -c`) : **9 AC, 8 tâches T0–T7** ; AC7 : 11 rubriques `kesh-db` (la
  rubrique de place compte quatre tests : trois de place, un de mode), 2 fichiers `kesh-api`, 2
  fichiers Vitest — inchangés. **La suite est une passe ciblée P7** (une lentille, **Haiku**, D6) sur
  le seul commit de cette remédiation : P6 est la dernière passe complète (dérogation, C88).
- **2026-10-08 — Validation P7 ciblée (Haiku, une lentille, prompt `15-5d-validate-prompt-p7-ciblee.md`) sur
  `de4cf826` : 0 finding**, axes exercés et non exercés déclarés ; aucun défaut d'ordre des verrous né d'une
  remédiation, donc la coupe AC5/AC6 prévue par la dérogation (C88) ne s'applique pas. Vérifié par l'orchestrateur sur
  l'axe le plus exposé (motif d'attente élargi) : `attendre_une_requete_en_cours` filtre sur `DB = DATABASE()` — chaque
  `#[sqlx::test]` a sa propre base, aucun autre test ne peut être capté — et sur une requête EN COURS ; une lecture non
  bloquée finit aussitôt et `interrompre` le détecte ; la sonde `NOWAIT` (1205) et le refus après commit complètent la
  preuve. **Validation CLOSE.** Trend : P1 4 MEDIUM → P2 1 HIGH / 3 MEDIUM → P3 2 MEDIUM → P4 ciblée 0 → alignement
  (C85) → P5 1 HIGH / 4 MEDIUM → P6 2 MEDIUM → P7 ciblée 0. Signaux D5 : C47, C88 (dérogation écrite).
- **2026-10-08 — Développement (`bmad-dev-story`, Claude Opus 5.5)**, commit de code `43205b2c`. Garde à l'usage des
  quatre comptes de réglage (accesseur en deux temps, verrou partagé aux places `(2 bis', suite)` et `(2, désignés)`,
  contrôle des rôles écrits), variante `DesignatedAccountsNotPostable`, message à quatre locales, compte créanciers à
  l'écran, dialogue de validation fermé sur `ACCOUNT_NOT_POSTABLE`, manuels, API, CHANGELOG. T0 : aucune mesure ne
  contredit l'intention de la fiche ; une contredit **la lettre** du test 4 (bloqueuse `SELECT id … LOCK IN SHARE MODE`
  servie par un index secondaire couvrant, qui ne verrouille pas la clé primaire — mutation `FOR UPDATE` restée verte),
  corrigée au plus près de l'intention (C-15-5d-1). 20 tests Rust et 5 Vitest neufs ; 14 mutations backend et 5
  frontend, toutes rouges. Gates au commit de code : backend 2854/2854 (4 ignorés), Vitest 1091/1091, E2E 247 passés /
  7 KF-029. Choix C-15-5d-1 à C-15-5d-4. Statut → `review`.
- **2026-10-08 — Revue de code P1** (Sonnet ×3, lentilles B, E, A ; prompt `15-5d-review-prompt-p1.md` ; rapports
  `target/gate-logs/15-5d-review-p1-{B,E,A}.md`) : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 10 LOW** (B 3 LOW ; E 1 MEDIUM, 5
  LOW ; A 2 LOW). Remédiation `a8bab77b` (Claude Opus 5.5) :
  | Finding | Sév. | Traitement |
  |---|---|---|
  | E1 | MEDIUM | `user-manual.tex`, paragraphe « Un compte des réglages devenu non imputable » : l'avoir n'est pas soumis au contrôle, raison (« une facture émise doit rester annulable ») et renvoi ; PDF régénéré et contrôlé aplati. L'encadré *Rôles des comptes* le disait déjà (« Deux exceptions, voulues ») — la preuve du rapport sur ce point est inexacte, le manque du paragraphe de la validation réel |
  | B-1 | LOW | `FORCE INDEX (PRIMARY)` sur la requête verrouillante ; `EXPLAIN` au Dev Agent Record ; motifs `ACCESSEUR` des tests de place alignés (C-15-5d-5) |
  | B-2 | LOW | doc-comments distincts réponse / requête (`invoices.types.ts`) |
  | B-3 | LOW | doc-comments de `validate_invoice` et `supplier_invoices::create_in_tx` : la vente n'a pas de cycle parce que les deux verrous sont partagés, non par le type ; l'achat écrit le cycle d'un compte désigné retypé en charge, angle mort couvert par le rejeu |
  | A-1 | LOW | la phrase de renvoi « Décompte TVA » revient au paragraphe des comptes TVA ; PDF régénéré |
  | E2 | LOW | test de priorité C49 en mélange ; test de mode partagé côté achat ; archivé / étranger / deux rôles côté achat acceptés (accesseur commun couvert — C-15-5d-6) |
  | E3, E6, A-2 | LOW | acceptés (C-15-5d-6) |
  | E4 | LOW | angle mort écrit au doc-comment de l'accesseur (faux refus sûr d'un compte créé et désigné après l'instantané) |
  | E5 | LOW | dépendance écrite : le rejeu de la complétion d'import relève de la 15-5e2 (C-15-5d-6) |
  Propagation : `grep -rniE "disjoint"` sur `crates/*/src` et `docs/` — plus de site affirmant la disjonction par
  type hors la phrase réécrite de l'achat ; reste la Dev Note de la fiche (§ ordre des verrous, texte de spécification
  historique, non réécrit). Gates au commit de code `a8bab77b` : backend 2856/2856, Vitest 1091/1091, E2E 247 / 7
  KF-029. Incident de procédure signalé par la lentille E (`git checkout --detach`, rétabli, sans effet). Statut
  maintenu à `review` : **passe ciblée P2** (une lentille, Haiku, D6) à lancer sur `a8bab77b` — la remédiation touche du
  code de production (une clause SQL, des doc-comments). Choix C-15-5d-5, C-15-5d-6.
- **2026-10-08 — Revue de code P2 ciblée (Haiku, une lentille, prompt `15-5d-review-prompt-p2-ciblee.md`) sur
  `a8bab77b` : 0 finding.** Rapport : `target/gate-logs/15-5d-review-p2-ciblee.md`. Repris par l'orchestrateur sur le
  point le plus risqué : le motif des tests de place (`crates/kesh-db/tests/invoices_validate_vat.rs:1080`,
  `FROM accounts FORCE INDEX (PRIMARY) WHERE company_id`) suit exactement la requête de l'accesseur
  (`company_invoice_settings.rs:738`), et les tests de place 1 et 2 sont verts au gate complet — l'attente est donc
  bien vue. **Boucle de revue CLOSE** (P1 Sonnet ×3 : 1 MEDIUM, 9 LOW → P2 ciblée Haiku : 0). Statut `done`.
- **2026-10-08 — Intégration** : rebasée sur `origin/main` `9cb5083b` (15-5e2) ; conflits résolus par union ou
  fusion des deux intentions (« 5 bis », registre, `sprint-status.yaml`), PDF régénérés ; gates sur l'état rebasé :
  backend 2859/2859 (4 ignorés), Vitest 1091/1091, E2E 247 / 7 KF-029. Choix C-15-5d-7.
