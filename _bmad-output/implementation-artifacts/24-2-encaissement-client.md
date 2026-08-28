# Story 24.2 : L'écriture d'encaissement — que la banque cesse d'être sous-évaluée

## Status

done

## Story

**As a** personne qui tient ses comptes dans Kesh,
**I want** que l'encaissement d'une facture client produise l'écriture qui lui correspond,
**so that** le solde du compte bancaire de mes livres soit celui de mon relevé.

Ferme l'issue **#371**, le défaut fondateur de la vague 1.

## Le défaut, et pourquoi il est muet

Ni `mark_as_paid` (`crates/kesh-db/src/repositories/invoices.rs:1926`, dont le doc-comment dit
en toutes lettres « **Ne crée AUCUNE écriture comptable** ») ni `accept_one_invoice`
(`crates/kesh-api/src/routes/reconciliation.rs:1003`) ne passent d'écriture de règlement. La
réconciliation rapproche la transaction bancaire de **l'écriture de vente elle-même** —
`let journal_entry_id = invoice.journal_entry_id.unwrap();`, ligne 1137.

⚠️ **Le bilan reste équilibré**, la partie double est respectée, aucun contrôle interne ne
rougit. Mais deux postes sont faux du même montant en sens inverse : **débiteurs surévalués**,
**banque sous-évaluée**. L'écart au 31.12 vaut le total des encaissements clients depuis
l'origine — c'est-à-dire exactement le premier contrôle que fait un réviseur.

## Le gabarit existe et fonctionne — ne rien réinventer

⛔ **`supplier_invoices::pay_in_tx` (`crates/kesh-db/src/repositories/supplier_invoices.rs:528`)
est le symétrique complet.** La story en écrit le miroir.

| Étape de `pay_in_tx` | Le miroir client |
|---|---|
| (1) `SELECT … FOR UPDATE` + garde de statut | idem, garde `status = 'validated'` |
| (2) **compte ET montant lus sur la ligne de CRÉDIT de l'écriture d'achat** | la ligne de **DÉBIT** de l'écriture de vente |
| (3) contrepartie = `journal_account_id` du compte bancaire | idem |
| (4) `fiscal_years::find_open_covering_date` | idem |
| (5) `journal_entries::create_in_tx(…, false)` | idem |
| (6) `UPDATE` + verrou optimiste + `rows_affected` | idem |
| (7) relecture + audit `{before, after}` | idem |

⛔ **L'étape (2) est la plus importante et la moins évidente.** `pay_in_tx` ne lit **ni les
réglages, ni le total de la facture** : il prend le compte **sur l'écriture d'achat elle-même**.
C'est ce qui garantit que le compte créanciers **se solde exactement**, quoi qu'il soit arrivé
aux réglages entre l'enregistrement et le règlement.

**Vérifié au sol, et les trois écritures sont symétriques :**

| Écriture | Ligne sur le compte de créance | Position |
|---|---|---|
| Vente (`generate_invoice_journal_lines:1430`) | **débit** du TTC | 0, unique |
| Avoir (`generate_credit_note_journal_lines:187`) | **crédit** du TTC | 0, unique |
| Encaissement (à écrire) | **crédit** du montant encaissé | — |

La ligne de créance est **unique** dans chacune : c'est une invariante à **asserter**, pas à
supposer.

## D1 — Le sens et le montant

`D <compte bancaire> / C <compte de créance>`, journal **Banque**, à la **date de valeur de la
transaction bancaire**.

⛔ **Le montant est celui de la TRANSACTION BANCAIRE, jamais le total de la facture.** L'écriture
existe pour que le compte bancaire de Kesh égale le relevé : lui donner un autre montant
reproduirait le défaut qu'on corrige, d'un cran plus loin.

## D2 — Une facture se règle en PLUSIEURS fois, et éventuellement par un avoir

⚠️ **Arbitrage du Project Lead (2026-08-27)** : *« on doit pouvoir réconcilier une facture avec
plusieurs paiements et éventuellement une note de crédit. »*

Conséquence directe : **une colonne `settlement_journal_entry_id` au singulier ne convient
pas** (contrairement au fournisseur, où le règlement est unique par construction). Il faut une
**table de liaison** :

⛔ **Le gabarit est `invoice_reminders`** (`20260715000001_invoice_reminders.sql:33`) : l'autre
enfant récent de `invoices`. Il **porte un `company_id`** avec sa FK et son index composite
`(company_id, invoice_id)`.

⚠️ **Première rédaction de cette spec : « pas de `company_id`, comme `journal_entry_lines` ».
C'était le mauvais voisin** — `journal_entry_lines` est un enfant d'`journal_entries`, pas
d'`invoices`, et son omission est une exception documentée, pas la règle. Vérifié au sol et
corrigé avant d'écrire une ligne de code.

```sql
CREATE TABLE invoice_settlements (
    id BIGINT NOT NULL AUTO_INCREMENT,
    company_id BIGINT NOT NULL,
    invoice_id BIGINT NOT NULL,
    journal_entry_id BIGINT NOT NULL,
    amount DECIMAL(19,4) NOT NULL,
    settled_on DATE NOT NULL,
    created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    CONSTRAINT fk_invoice_settlements_invoice
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    CONSTRAINT fk_invoice_settlements_company
        FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT,
    CONSTRAINT fk_invoice_settlements_entry
        FOREIGN KEY (journal_entry_id) REFERENCES journal_entries(id) ON DELETE RESTRICT,
    CONSTRAINT chk_invoice_settlements_amount_positive CHECK (amount > 0),
    UNIQUE KEY uq_invoice_settlements_entry (journal_entry_id),
    INDEX idx_invoice_settlements_company_invoice (company_id, invoice_id),
    INDEX idx_invoice_settlements_invoice (invoice_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
```

⚠️ **`ON DELETE` asymétrique, et chaque branche a sa raison** : `CASCADE` sur la facture (patron
`invoice_reminders`, aligné sur la suppression définitive #219) ; `RESTRICT` sur l'écriture —
une écriture comptable référencée ne se supprime pas, elle se contre-passe.

⚠️ `UNIQUE` sur `journal_entry_id` : une écriture d'encaissement règle **une** facture. Le
règlement groupé — un virement qui solde trois factures — n'est **pas** couvert ici (cf. Hors
périmètre) ; la contrainte le rend impossible plutôt que silencieusement faux.

⛔ **Cette table est aussi le substrat du lettrage** (epic 15, gelé). Ne pas la concevoir contre
lui : c'est la même opération vue de plus loin.

## D3 — Le résiduel se CALCULE, il ne se stocke pas

Résiduel = TTC de la facture − avoir émis (s'il existe) − Σ des règlements.

⛔ **Aucun champ stocké.** Un montant dû rangé en colonne dérive du grand livre à la première
divergence, et on aurait recréé un chiffre qui ment — le défaut même de cette vague.

⛔ **Deux constantes SQL, exactement comme le TTC canonique**
(`INVOICE_TTC_SUBQUERY_SQL` / `INVOICE_TTC_DERIVED_JOIN_SQL`, `invoices.rs:161` et `:171`) : la
forme **scalaire corrélée** pour une facture, la forme **jointure dérivée** pour les listes et
les agrégats. Le doc-comment existant dit pourquoi — *« la corrélée serait ré-évaluée par ligne
et par CASE »* — et c'est un N+1 déguisé sur toute liste de factures.

⚠️ **L'avoir ne compte que s'il est `issued`.** Les statuts sont `draft / issued / cancelled`
(`chk_credit_notes_status`, vérifié) ; un brouillon n'a pas d'écriture et ne réduit rien.

## D4 — `paid_at` devient la projection d'un solde tombé à zéro

⚠️ **Arbitrage du Project Lead** : *« une facture est payée parce que la comptabilité le
montre. »* `paid_at` cesse d'être un drapeau posé par le premier paiement : il est posé
**quand le résiduel atteint zéro**, et pas avant.

**Ce que cela règle gratuitement, et c'est le bon signe** : `paid_at IS NULL` est le prédicat
« encore ouverte » à six endroits vérifiés — `dunning_eligibility.rs:87`,
`reconciliation.rs:112`, `invoices.rs:320`, `:331`, `:872`. Une facture partiellement payée
garde `paid_at` à `NULL`, donc elle **reste relançable et reste candidate à la réconciliation**
pour le solde. Aucun de ces cinq sites n'est à modifier.

**Deux cas tranchés** :

- Facture **entièrement compensée par un avoir**, jamais encaissée ⇒ **soldée**. La créance est
  éteinte ; `paid_at` signifie « date à laquelle le solde est tombé à zéro », pas « date
  d'encaissement ». C'est la date de l'avoir qui fait foi.
- Facture **partiellement payée** ⇒ reste relançable, **pour le solde**.

## D5 — Voir qu'une facture n'est que partiellement payée

⛔ **Sans cet AC, la story créerait un état que personne ne peut voir** — précisément le défaut
qu'elle corrige.

**Un quatrième état au badge existant.** `PaymentStatusBadge.svelte` porte `paid / unpaid /
overdue` ; on ajoute `partial`, avec le résiduel en regard.

⚠️ **Aucun nouvel état en base.** `chk_invoices_status` n'autorise que
`draft / validated / cancelled` (vérifié) ; « payée » est **déjà** dérivé de `paid_at`.
« Partielle » se dérive de la même façon : `paid_at IS NULL AND réglé > 0`. **Pas de migration
sur `status`, pas de `CHECK` à toucher.**

## D6 — `matched_entry_id` et le contrat d'API changent de sens

⛔ La transaction bancaire est rapprochée de **l'écriture d'encaissement**, pas de l'écriture de
vente. C'est le sens de la colonne : ce qui a bougé en banque.

⚠️ **`AcceptedProposal.journalEntryId` change avec elle.** Le frontend le consomme
(`reconciliation.types.ts:84`), et **deux assertions le figent sur l'écriture de vente** :
`reconciliation_e2e.rs:915` (réponse d'API) et `:925` (persistance). **Ces deux tests décrivent
le défaut ; ils se corrigent, ils ne se contournent pas.**

## D7 — Les factures déjà marquées payées sans écriture

**Décision : ne rien rétro-corriger, et ne rien masquer.** Aucun backfill — fabriquer des
écritures rétroactives, c'est écrire des écritures que personne n'a passées, à des dates que
personne n'a choisies, dans des exercices peut-être clos. Ces factures n'ont simplement aucune
ligne dans `invoice_settlements`, ce qui les rend **exactement identifiables**, et le grand
livre livré en 24-1 permet de les voir.

⚠️ **Tenable ici et seulement ici** : le `CLAUDE.md` établit qu'il n'y a **aucune donnée de
production à protéger**. Le jour où ce paragraphe changera, la même question appellera une autre
réponse.

## Critères d'acceptation

**AC1** — `accept_one_invoice` crée `D <journal_account_id du compte bancaire> / C <compte de la
ligne de débit de l'écriture de vente>`, du **montant de la transaction bancaire**, à sa date de
valeur, journal **Banque**.

**AC2** — Le compte de créance est lu **sur l'écriture de vente**, jamais sur les réglages ni
sur `invoices.total_amount`. ⛔ **Invariante** : après règlement intégral, la somme des
mouvements du compte de créance pour cette facture vaut **exactement zéro**.

**AC3** — Une ligne `invoice_settlements` est créée par encaissement, portant son montant.

**AC4** — **Plusieurs encaissements sur une même facture sont acceptés** et s'additionnent.

**AC5** — `paid_at` est posé **si et seulement si** le résiduel atteint zéro — jamais au premier
encaissement partiel.

**AC6** — Un avoir `issued` réduit le résiduel. Une facture entièrement compensée par un avoir
est **soldée**, `paid_at` = date de l'avoir.

**AC7** — Un encaissement qui **dépasse** le résiduel est refusé :
`FailedProposal { error_code: "RECONCILIATION_OVERPAYMENT" }`, avec résiduel et montant dans
`details`. ⛔ Sinon le compte de créance passerait **créditeur** — un solde contre nature que le
grand livre signalerait, mais après coup.

**AC8** — `bank_transactions.matched_entry_id` **et** `AcceptedProposal.journalEntryId` pointent
sur l'écriture d'**encaissement**.

**AC9** — L'API expose `amountSettled` et `amountDue`, **calculés**, sur la **fiche facture**.

⚠️ **La frontière n'est pas où elle paraît** (arbitrage du 2026-08-27 : « ok, pas urgent »). Le
badge d'AC10 a **besoin du calcul** ligne par ligne dans une liste — « partielle » se dérive de
« réglé > 0 et solde ≠ 0 ». La forme en **jointure dérivée est donc nécessaire de toute façon**,
et sans elle la liste ferait du N+1. Ce qui est reporté, c'est l'**affichage du montant en
colonne** dans l'échéancier et la balance âgée, pas son calcul.

**AC10** — `PaymentStatusBadge` porte un état `partial`, dérivé, avec le résiduel. Quatre
locales.

**AC11** — Compte bancaire sans `journal_account_id` ⇒ `BANK_ACCOUNT_NOT_CONFIGURED`
(**code existant**, `reconciliation.rs:1400` — le réutiliser, ne pas en inventer un second).

**AC12** — Aucun exercice **ouvert** ne couvre la date de valeur ⇒ `FISCAL_YEAR_INVALID`
(constante existante, `kesh-db/src/errors.rs:256`). ⛔ Jamais d'écriture dans un exercice clos.

**AC13** — Tout dans **la transaction existante** : écriture, liaison, `matched_entry_id`,
`paid_at`, audit. Un échec à n'importe quelle étape ne laisse **ni écriture orpheline, ni
facture soldée sans écriture**.

**AC14** — Migration : `CREATE TABLE` — **non-breaking**, donc **pas de bump
`min_required`** (P1/P2) ; **aucune écriture de données**, donc exemption `EXEMPT_MIGRATIONS`
justifiée par écrit (P7) ; ligne d'audit d'idempotence obligatoire (P5), **et ses cinq
compteurs recomptés depuis le tableau**.

**AC15** — Le doc-comment de `mark_as_paid` cesse de dire « l'écriture sera générée par la
réconciliation automatique (Epic 6) » — **faux depuis Epic 6** — et renvoie à #372 pour le
règlement hors banque, qui reste non couvert.

## Invariants testables

1. **La créance se solde.** Facture 100 → encaissements 60 puis 40 ⇒ mouvements du compte de
   créance = 0, et `paid_at` posé **au second seulement**.
2. **La banque égale le relevé.** Σ mouvements du compte bancaire = Σ montants des transactions
   rapprochées.
3. **L'avoir compte.** Facture 100, avoir `issued` 40, encaissement 60 ⇒ soldée. Le même avoir
   en `draft` ⇒ **non** soldée.
4. **Le trop-perçu est refusé.** Facture 100, encaissement 60, puis 50 ⇒ `failed[]`, **aucune
   écriture créée**, résiduel toujours 40.
5. **Concordance avec le grand livre** (24-1) : le compte de créance montre débit de vente,
   crédits d'encaissement, solde nul.
6. **`matched_entry_id` ≠ `invoice.journal_entry_id`** — l'assertion qui aurait attrapé le
   défaut d'origine.
7. **Pas de N+1** : lister N factures exécute un nombre de requêtes **indépendant de N**.
8. **Rien d'orphelin** : échec après création de l'écriture ⇒ ni écriture, ni liaison, ni
   `paid_at`.

## Hors périmètre, délibérément

Le **règlement hors banque** — espèces, compensation (#372, story 24-3), y compris ce que
devient `mark_as_paid`. Le **règlement groupé** — un virement soldant plusieurs factures (la
contrainte `UNIQUE` le rend impossible plutôt que faux). La **propagation du résiduel aux
rapports agrégés** — balance âgée et totaux de l'échéancier affichent le TTC, et les colonnes de
montant dû n'y sont pas ajoutées ici ; **issue séparée à ouvrir**, décidé avec le Project Lead
le 2026-08-27 (« ok, pas urgent »). Le **lettrage** (epic 15). Le
**dé-rapprochement** — annuler un encaissement demande une contre-passation, pas une
suppression. La **rétro-correction** des factures déjà payées (D7).

## Dev Notes

⛔ **Le gate ciblé est INTERDIT.** La story touche `crates/kesh-db/migrations/` et un
repository : les garde-fous **P6** et **P7** imposent le **gate complet** même en cours de
boucle de revue. Le mode d'échec qu'ils visent ne naît ni du code ni de la spec, mais de
l'**interaction** avec des tests que la PR ne touche pas.

⚠️ **La base de gate se remet à zéro AVANT chaque gate**, inconditionnellement (KF-039, #310).

⚠️ **Surface de régression RELEVÉE, non supposée** : `matched_entry_id` apparaît **3 fois** dans
`reconciliation_e2e.rs` et **4 fois** dans `reconciliation_manual_e2e.rs`. Ces dernières
relèvent du rapprochement **manuel**, qui crée déjà sa propre écriture et **n'est pas touché** —
à vérifier une par une plutôt qu'à supposer.

⚠️ **KF-038 (#228)** : `reconciliation_*_e2e` porte un flake connu sous contention. Un rouge
isolé sur `post_accept_skips_non_chf_transaction` se rejoue seul **avant** tout diagnostic.

⚠️ **Traiter chaque test rouge comme une question** — « ce test décrivait-il le défaut ? » — et
non comme un obstacle. C'est la différence entre corriger un bug et le déplacer dans les tests.

## Change Log

### Implémentation — 2026-08-27

**Livré** : migration `20260827000001_invoice_settlements.sql`, entité et dépôt, l'écriture
`D banque / C créance` dans `accept_one_invoice`, le résiduel calculé, `amountSettled` /
`amountDue` sur la fiche, le badge `partial` dérivé, quatre locales.

**Décomptes recomptés depuis la source** — périmètre `main…HEAD` :

| Mesure | Valeur | Commande |
|---|---|---|
| migrations | 62 | `ls crates/kesh-db/migrations/*.sql \| wc -l` |
| lignes du tableau d'audit | 62 | `grep -c '^\| `20' docs/migrations-idempotence-audit.md` |
| partition d'idempotence | 5 + 57 + 0 = 62 | recomptée depuis le tableau |
| tests de réconciliation | 27 | `cargo nextest run -E 'binary(reconciliation_e2e)'` |
| tests d'import | 19 | `binary(admin_full_import_e2e)` |
| clés i18n ajoutées × 4 locales | 3 | `payment-status-partial`, `invoice-amount-settled`, `invoice-amount-due` |
| sites `i18nMsg` ajoutés | 2 | ventilés dans `i18n-keys.test.ts` |

### ⛔ Sept garde-fous déclenchés, tous sur des fichiers que la story ne touche pas

C'est la démonstration de ce que la § *Dev Notes* annonçait en interdisant le gate ciblé :
**aucun** de ces sept n'aurait été vu autrement.

| # | Garde | Ce qu'il a exigé |
|---|---|---|
| 1 | `clippy -D warnings` | un champ de fixture jamais lu |
| 2 | `backup_inventory_matches_schema` | l'inventaire de sauvegarde connaît la table |
| 3 | `full_export_structure_manifest_and_integrity` | 37 → 38 fichiers `data/*.ndjson` |
| 4 | `registry_entries_are_within_import_window` | la fenêtre s'est refermée sur le registre |
| 5 | compteur des exemptions « Hors fenêtre » | 4 → 6, en relisant les justifications |
| 6 | `upgrade_path_preserves_data` (**P6**) | `total` et `N` incrémentés du même pas |
| 7 | `published_migrations_keep_their_checksums` (**P8**) | inscrire le checksum = déclarer publiée |

⚠️ **Le n° 4 est le plus profond, et n'a pas de correctif mécanique.** Créer une table
applicative **referme la fenêtre d'importabilité** — et elle s'est refermée au-delà des **deux**
entrées du registre de rejeu post-restauration. Un backup assez ancien pour les déclencher est
désormais dépourvu de `invoice_settlements`, donc refusé en 400 avant tout rejeu. Les y laisser
aurait produit du **code mort qui paraît fonctionner**, et pour celle de classe A, **exécuté à
chaque import**.

Sur arbitrage du Project Lead (« B »), la couverture est **préservée et non supprimée** :
`RETIRED_BACKFILLS` porte les deux entrées comme fixture, les six tests d'import gardent leur
montage HTTP réel et injectent le registre par la couture `replay_with_registry` — leurs
assertions passent du JSON d'audit au **rapport typé**, ce qui est un gain. Un cas neuf
verrouille la propriété de production : **l'import ne rejoue plus rien**.

### La fixture était creuse, et c'est ce qui a caché le défaut fondateur

`insert_fake_journal_entry` créait une écriture de vente **sans aucune ligne**, et le compte
bancaire de test n'avait pas de `journal_account_id`. Les tests rapprochaient donc une facture
**sans contrepartie comptable**. ⚠️ **On ne peut pas voir manquer une écriture d'encaissement là
où rien n'a de substance** — c'est ce vide qui a laissé le défaut de #371 vivre des mois.

Deux assertions **décrivaient le défaut** (`reconciliation_e2e.rs:915` et `:925`, l'écriture
rapprochée figée sur la vente) : elles ont été retournées contre lui, `assert_ne!` là où il y
avait `assert_eq!(je_id)`.

### KF-038 (#228) fermée en chemin — cause racine, pas contournement

`with_account_lock` nommait son verrou `reconcile:{company_id}:{bank_account_id}`. Or **`GET_LOCK`
est global au serveur MariaDB**, et chaque base éphémère de test repart à `company_id = 1`,
`bank_account_id = 1` : **tous les tests se disputaient `reconcile:1:1`**. Vérifié au sol sur
deux bases. La production n'était pas touchée (une seule base) ; la story aggravait le défaut,
l'écriture étant créée **à l'intérieur** du verrou.

Le nom de la base entre dans la clé. Mesure, binaire seul : **1 échec sur 3 avant, 0 sur 5
après**. ⚠️ Le test qui *fabrique* la contention connaissait l'ancien nom et est devenu
**déterministe rouge** — signal correct : il n'observait plus rien.

**Gates réellement exécutés** : `fmt`, `clippy -D warnings`, `scripts/test-fast.sh` complet,
base remise à zéro avant chaque run (KF-039) ; frontend `check` 0 erreur,
`lint-i18n-ownership`, `test:unit`, `build`.
