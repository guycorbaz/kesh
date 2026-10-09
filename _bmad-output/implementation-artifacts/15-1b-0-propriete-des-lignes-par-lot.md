# Story 15.1b-0 : La propriété des lignes, par lot — une seule source, prouvée contre l'ancienne

## Status

ready-for-dev *(extraite de la 15-1b le 2026-10-09 à la remédiation de sa validation P2 — signal D5 levé
par un HIGH né d'une remédiation, C-15-1b-9 ; **validation P1 à mener avant tout développement**)*.

Story **patron** au sens de la § « Règle de splitting préventif » (« dégager d'abord un story-zero qui
pose le pattern ») : elle pose la lecture **par lot** de la propriété des écritures et réécrit dessus les
deux lecteurs du socle ; la **15-1b** (vue des postes ouverts, propositions) la consomme. **Ordre** :
15-1a-i → 15-1a-ii → 15-1a2-i → 15-1a2-ii → **15-1b-0** → 15-1b → 15-1c. Elle ne dépend **pas** de la
15-1a2 (aucune marque `document` n'y est lue) : elle pourrait la précéder ; l'ordre de l'epic la place
juste avant la 15-1b, qui la suppose mergée. ⛔ Pas de tag entre les merges (C124).

## Story

**As a** développeur de Kesh,
**I want** une fonction qui dise, pour un **lot** d'écritures, quelle pièce possède chacune — facture,
avoir, facture fournisseur, règlement, transaction bancaire —, et que les deux lecteurs actuels de cette
propriété (`reversal_blockers`, `first_document_owner`) la lisent par elle,
**so that** la vue des postes ouverts (15-1b) affiche la pièce de chaque ligne et filtre les lignes de
pièce **sans N+1 et sans seconde liste** des motifs de propriété — et que la refonte soit **prouvée**
contre le comportement d'avant, pas contre elle-même.

## Pourquoi cette story existe

La remédiation P1 de la 15-1b (C-15-1b-2) avait mis dans la vue la refonte d'une fonction du socle. La
validation P2 l'a montrée **non implémentable sans deviner** (15-1b R-4 = F-5 : un `E: Executor` pris par
valeur ne permet pas les deux requêtes que la fiche prescrivait ; appelants non inventoriés, dont un sur
le **pool** ; docstring « ⛔ Une seule requête » contredite) et sa garde **verte par construction** (F-4 :
le test 8 comparait `reversal_blockers` à `document_owners` après que l'un eut été réécrit sur l'autre —
exactement l'AC7 de l'Epic 23 que le `CLAUDE.md` décrit sous « Inventorier les sites NON RÉSOLUS »). Le
signal de découpage D5 était levé (HIGH né d'une remédiation, P1 0 HIGH → P2 1 HIGH). Décision de
l'orchestrateur : **extraire** cette refonte — un repository du socle, gate complet à chaque passe — dans
une story propre, la 15-1b gardant la vue et les propositions.

## Le modèle réel — relevé sur `056997b0`

*(Lignes citées sur `056997b0` ; elles bougeront — **citer et re-greper par le nom de fonction**.)*

**`journal_entries::reversal_blockers<'e, E: Executor>(executor: E, company_id, id) -> Result<Vec<ReversalBlockerHit>, DbError>`**
(`journal_entries.rs:2090`) : **une** requête, onze colonnes par sous-requêtes corrélées
(`:2117-2141`), consommée par `.fetch_optional(executor)` (`:2143`). Ordre des motifs = précédence
(`IsAReversal`, `AlreadyReversed`, `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`,
`OwnedBySettlement`, `MatchedBankTransaction`, `AccountArchived`). Chaque sous-requête de propriété est un
`LIMIT 1` **sans `ORDER BY`** : déterministe tant qu'une seule pièce de chaque type vise l'écriture, ce que
garantissent les contraintes pour les quatre pièces — mais **pas** pour `bank_transactions.matched_entry_id`
(deux transactions peuvent pointer la même écriture : c'est l'« exemption étroite » de la file des
annulations, `settlement_cancellation.rs` doc-comment du rang 3). Le doc-comment de `reversal_blocker`
(`:2051-2053`) pose « ⛔ **Une seule requête** : les sept causes se calculent par sous-requêtes corrélées,
jamais par sept allers-retours ».

**`letterings::first_document_owner`** (`letterings.rs:511`, privée) : boucle sur les écritures
distinctes du groupe et appelle `reversal_blockers` **une écriture à la fois** (`:522`), retenant
`OwnedByInvoice | OwnedByCreditNote | OwnedBySupplierInvoice | OwnedBySettlement` (R5).
`MatchedBankTransaction` n'en est pas.

**Appelants de `reversal_blockers` / `reversal_blocker`** — inventaire **fermé**, relevé par
`grep -rn "reversal_blockers\|reversal_blocker(" crates --include=*.rs` :

| site | exécuteur passé | contexte |
|---|---|---|
| `kesh-api/src/routes/journal_entries.rs:472` (`reversal_blocker`) | `&state.pool` | lecture pour l'écran — **hors transaction** |
| `kesh-db/src/repositories/journal_entries.rs:1068` (`modification_guard`) | `&mut *conn` | sous le verrou de l'écriture (C-15-8-24 : « une connexion, pas un `Executor` générique ») |
| `journal_entries.rs:2072` (`reversal_blocker`, enveloppe) | `executor` | délègue |
| `journal_entries.rs:2438` (`reverse_owned_in_tx`) | `&mut **tx` | dans la transaction du geste |
| `settlement_cancellation.rs:107` (file commune des annulations) | `&mut *conn` | prédicteur et gestes |
| `letterings.rs:522` (`first_document_owner`) | `&mut **tx` | acte 1 du lettrage, mode `Manual` |
| `kesh-db/tests/letterings.rs:588` | `&mut *conn` | test |
| `kesh-db/tests/invoice_settlement.rs:579`, `:584` | `&pool` | test |
| `kesh-db/tests/supplier_invoices_repository.rs:298`, `:724`, `:1300` | `&pool` | test |

S'y ajoutent deux **mentions** sans appel, à relire : `kesh-db/tests/journal_entries_modification.rs:104`
(la constante `QUOI_FAIRE` renvoie à `reversal_blockers` pour trier une référence) et
`kesh-api/tests/journal_entry_reversal_e2e.rs:1061` (commentaire). **Six** sites de production (dont
l'enveloppe), **six** appels de test dans **cinq** fonctions et **trois** fichiers, **deux** mentions dans
deux autres fichiers — **cinq** fichiers de tests en tout.

## Décisions

### D1 — `document_owners`, par lot, une connexion

```rust
/// Le type d'une pièce propriétaire, dans l'ordre de précédence de `reversal_blockers` (rangs 3 à 7).
pub enum DocumentKind { Invoice, CreditNote, SupplierInvoice, Settlement, BankTransaction }

pub struct DocumentOwner {
    pub kind: DocumentKind,
    pub id: i64,
    /// Numéro de la pièce quand elle en a un (facture, avoir, facture fournisseur), sinon `None`.
    pub number: Option<String>,
    /// Pour `Settlement` : la facture réglée (`invoice_settlements.invoice_id`) et son numéro ; `None` sinon.
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
}

pub async fn document_owners(
    conn: &mut MySqlConnection, company_id: i64, entry_ids: &[i64],
) -> Result<BTreeMap<i64, Vec<DocumentOwner>>, DbError>;
```

- **Une requête ensembliste par tranche de 500** écritures (`WHERE je.id IN (…) AND je.company_id = ?`),
  sans N+1 ; liste vide → `Ok(BTreeMap::new())` sans requête.
- `Vec<DocumentOwner>` **dans l'ordre de précédence** (Invoice, CreditNote, SupplierInvoice, Settlement,
  BankTransaction) ; une écriture sans propriétaire est **absente** de la table (ou présente avec un
  vecteur vide — à fixer au développement et à écrire dans le doc-comment ; le test l'asserte).
- **Au plus un propriétaire par type et par écriture, le plus petit `id`** — là où la requête d'aujourd'hui
  prend un `LIMIT 1` arbitraire. Écart **nommé** : pour les quatre pièces il ne change rien (une seule
  pièce par type) ; pour `BankTransaction`, il rend déterministe ce qui ne l'était pas.
- ⛔ **`&mut MySqlConnection`, pas un `Executor` générique** (C-15-1b-10) — patron C-15-8-24 de
  `modification_guard` : la fonction enchaîne plusieurs requêtes, ce qu'un exécuteur consommé par valeur
  interdit. Une transaction se passe déréférencée (`&mut **tx`), une connexion se reprête (`&mut *conn`).

### D2 — `reversal_blockers` réécrit dessus, signature `&mut MySqlConnection`

- `reversal_blockers(conn: &mut MySqlConnection, company_id, id)` : **deux** requêtes — la sienne, réduite
  aux rangs 1, 2 et 8 (`reverses_entry_id`, `reversed_by`, `archived_account_number`), et
  `document_owners(conn, company_id, &[id])` pour les rangs 3 à 7 —, puis la même liste, dans le même
  ordre, avec les mêmes identifiants et étiquettes (numéro de pièce ; `None` pour règlement et transaction).
  `NotFound` inchangé (écriture absente ou d'une autre société).
- `reversal_blocker` (enveloppe) suit la même signature.
- **Le doc-comment « ⛔ Une seule requête » est réécrit** : deux requêtes, **un instantané** dès que
  l'appelant tient une transaction (`REPEATABLE READ` : l'instantané se fige à la première lecture et vaut
  pour la seconde). Tous les appelants de production en tiennent une, **sauf la route** :
- **La route** (`routes/journal_entries.rs:472`) ouvre une transaction de **lecture** (`state.pool.begin()`,
  lecture puis `rollback`) et y appelle `reversal_blocker(&mut *tx, …)` — sans quoi les deux requêtes
  liraient deux instantanés, et un motif pourrait s'afficher contre un état qu'aucun instant n'a connu.
  Patron : la vue des postes ouverts (15-1b AC1, « une transaction, vue unique »).
- Les tests sur `&pool` passent une connexion acquise (`&mut *pool.acquire().await?`) — deux instantanés
  y sont indifférents (aucun écrivain concurrent dans ces tests).

### D3 — `first_document_owner` en un lot

Un seul appel `document_owners(&mut **tx, company_id, &entrées_distinctes)` au lieu de la boucle ; la
première ligne (dans l'ordre des lignes, comme aujourd'hui) dont l'écriture a un propriétaire de type
Invoice, CreditNote, SupplierInvoice ou Settlement rend `LetteringLineOwnedByDocument` avec **le même**
`blocker`, `document_id` et `document_label` qu'aujourd'hui. Sous les verrous de l'acte 1 : inchangé.

### D4 — La parité se prouve contre un ORACLE INDÉPENDANT (C-15-1b-10)

Deux oracles, aucun ne sortant de `document_owners` :

1. **L'ancien code, gelé dans un module de test** — `crates/kesh-db/tests/common/reversal_blockers_frozen.rs` :
   la requête de `reversal_blockers` **copiée telle qu'au commit `056997b0`** (SQL et décodage, ordre des
   rangs), avec un en-tête qui dit d'où elle vient, pourquoi elle ne doit **pas** suivre le code de
   production, et le commit de référence. Pour **chaque** écriture d'une fixture qui exerce les huit
   rangs, seuls et cumulés (règlement encaissé par rapprochement : `OwnedBySettlement` **et**
   `MatchedBankTransaction` ; facture fournisseur achat **et** règlement ; contre-passation et contre-passée ;
   compte archivé), `reversal_blockers` (nouveau) == oracle gelé — motif, identifiant, étiquette, ordre.
   ⚠️ La fixture ne contient **pas** deux transactions pointant la même écriture (le `LIMIT 1` sans
   `ORDER BY` de l'oracle n'y serait pas déterministe) ; ce cas a son test propre, à valeurs écrites.
2. **Des valeurs écrites à la main** : pour une écriture de chaque type, le `DocumentOwner` attendu
   (type, id, numéro, facture du règlement) est écrit en dur dans le test — ce qui garde l'oracle gelé
   contre un défaut qu'il partagerait avec le nouveau code — ; et le cas « deux transactions » rend la
   plus petite.

### D5 — La mutation est ÉPROUVÉE, pas supposée

Au développement, trois mutations de `document_owners`, chacune appliquée puis annulée (fichier **touché**
après restauration : cargo garde sinon le binaire muté) : (a) inverser deux rangs (CreditNote avant
Invoice) ; (b) retirer la jointure de la facture d'un règlement ; (c) retirer le filtre `company_id`. Chacune
doit faire **rougir** au moins un test de D4 ; le Dev Agent Record nomme, pour chacune, le test qui a rougi.
Une mutation qui reste verte est un finding.

## Critères d'acceptation

**AC1** — `document_owners` existe avec la signature et le type de D1 ; une liste de 1 200 écritures est
lue en **trois** requêtes (tranches de 500), mesuré par un compteur de requêtes ou par la journalisation de
test ; une liste vide n'émet aucune requête ; une écriture d'une autre société n'a aucun propriétaire.

**AC2** — `reversal_blockers` et `reversal_blocker` prennent `&mut MySqlConnection` ; leurs résultats sont
**identiques** à l'oracle gelé sur toute la fixture de D4 (1) ; leur doc-comment ne dit plus « une seule
requête » et dit l'instantané.

**AC3** — `first_document_owner` fait **un** appel à `document_owners` par groupe (plus de boucle sur
`reversal_blockers`) ; les tests de R5 existants (`kesh-db/tests/letterings.rs`,
`supplier_invoices_repository.rs`) restent verts **sans modification de leurs assertions**.

**AC4** — **Inventaire fermé des appelants** : les six sites de production et les six appels de test du
tableau sont adaptés ; la route lit dans une transaction de lecture (D2) ; `grep -rn
"reversal_blockers\|reversal_blocker(" crates --include=*.rs` ne rend plus aucun appel qui passe un pool.

**AC5** — **Oracle indépendant** (D4) et **mutations éprouvées** (D5), consignées au Dev Agent Record.

**AC6** — **Aucun changement visible** : aucune route, aucun code d'erreur, aucun texte, aucune clé i18n ;
`CHANGELOG.md`, `api-external.md` et les manuels **inchangés** (refonte interne, sans effet de contrat —
contrôlé par `git diff --stat` sur `docs/`, `CHANGELOG.md`, `crates/kesh-i18n/`, `frontend/`).

## Tasks

- [ ] **T0** — Re-greper l'inventaire des appelants sur la base réelle du développement (le tableau est
      relevé sur `056997b0` ; la 15-1a2 aura pu en ajouter, par exemple dans la file commune) ; tout site
      neuf entre au tableau et à AC4.
- [ ] **T1** (D1, AC1) — `DocumentKind`, `DocumentOwner`, `document_owners`, doc-comments.
- [ ] **T2** (D2, D3, AC2, AC3) — `reversal_blockers` / `reversal_blocker` réécrits ; `first_document_owner`
      en un lot ; doc-comment « une seule requête » réécrit.
- [ ] **T3** (AC4) — Les appelants : route en transaction de lecture, tests sur connexion acquise, mentions
      relues (`journal_entries_modification.rs:104`, `journal_entry_reversal_e2e.rs:1061`).
- [ ] **T4** (D4, D5, AC5) — Module gelé, fixture, valeurs écrites, mutations et leur procès-verbal.

**Tests prévus** (4 neufs) — `crates/kesh-db/tests/document_owners.rs` (neuf, `test-schema`) :
`owners_match_the_frozen_reversal_blockers` (AC2, D4 (1)), `owners_match_handwritten_expectations` (D4 (2),
dont deux transactions sur une écriture), `owners_are_read_by_batches_of_500` (AC1),
`owners_are_scoped_by_company` (AC1). Plus le module de test `common/reversal_blockers_frozen.rs` (oracle,
pas un test). Tests existants **adaptés sans changement d'assertion** : `letterings.rs:588`,
`invoice_settlement.rs:579`, `:584`, `supplier_invoices_repository.rs:298`, `:724`, `:1300`.

*(Recompte depuis cette liste : **4 fonctions de test neuves** ; **6 appels de test adaptés**, dans
**5 fonctions** — `invoice_settlement.rs:579` et `:584` sont dans la même,
`annuler_l_unique_reglement_remet_la_facture_a_regler` — et **3 fichiers**, relevés par `awk` sur
`056997b0`.)* ⚠️ À recompter au T0 depuis la source : le nombre de **fonctions** de test touchées n'est pas
celui des **appels**.

## Dev Notes

- **Gate `kesh-db` complet, même en cours de boucle** (repository du socle — § « Exception `kesh-db` » du
  `CLAUDE.md`) ; base remise à zéro avant ; E2E complet au dernier commit de code (la route de
  contre-passation est exercée par l'écran).
- **Aucune migration** : P5–P8 sans objet.
- **Modules** : `kesh-db` (`journal_entries.rs`, `letterings.rs` ; `settlement_cancellation.rs`,
  `modification_guard` et `reverse_owned_in_tx` passent déjà une connexion ou une transaction déréférencée,
  et restent tels quels), `kesh-api` (une route). **Deux**, très en deçà du seuil.
- **Ce que la story ne fait pas** : elle n'expose rien à l'API ; la vue et les propositions sont la 15-1b.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P2 de la 15-1b, en autonomie)

Story **extraite** de la 15-1b (C-15-1b-9) : la refonte du socle que la remédiation P1 de la 15-1b y
avait placée (ancienne T2, ancien test 8, C-15-1b-2), réécrite pour corriger ce que la validation P2 y
a trouvé — signature inapplicable et appelants non inventoriés (R-4 = F-5, MEDIUM), garde de parité verte
par construction (F-4, MEDIUM). **6 critères** (AC1–AC6), **5 tâches** (T0–T4), **4 tests neufs** —
recomptés depuis ce fichier. Choix consignés : C-15-1b-9, C-15-1b-10 (registre). Prochaine passe :
validation **P1** (Sonnet, contexte frais, complète).
