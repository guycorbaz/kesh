# Story 53-1b : La conversion de devises ne passe pas deux fois

Status: drafted (fiche écrite le 2026-11-25 au découpage de la 53-1, décidé par le Project Lead sur le signal de recyclage V2-1 ; aucune passe de validation lancée)

Issue : **#1365** (conversion de devises, `createConversion`) — `bug`, **P1**. Epic 53 (« Écritures justes »), release visée **v0.21.0, vendredi 2026-12-04**. **Dépend de la 53-1a** (`53-1a-saisie-pas-deux-fois.md`, #1355) : elle en reprend le motif et les pièces communes, et **se fusionne après elle**. Branche à créer depuis `epic/53-ecritures-justes` **après** la fusion de la 53-1a (p. ex. `story/53-1b-conversion-unique`).

**Aucune migration.** Le contrat de l'API change (refus neuf sur `POST /journal-entries/conversions`), annoncé ⚠️ en tête des notes de version avec celui de la 53-1a (décision du Project Lead du 2026-11-25, Q5).

## Story

En tant que comptable qui passe ses conversions entre devises par l'assistant *Conversion*,
je veux que le serveur refuse par défaut une conversion dont une écriture est identique à une écriture déjà enregistrée le même jour — sauf confirmation explicite, consignée — et que l'assistant, quand la réponse se perd, se fige et m'invite à vérifier au journal au lieu d'annoncer un échec,
afin qu'un second clic après une réponse perdue ne passe pas deux écritures de plus au journal (#1365) sans que je l'aie vu et confirmé.

## Ce que la 53-1b reprend de la 53-1a (sans le refaire)

- La règle D1 (même date, même journal, même libellé, même multiensemble de lignes ; contre-passées exclues ; aucune fenêtre ; toujours levable) et ses décisions Q1 à Q5 ; **Q7** (décision du Project Lead du 2026-11-25) : une conversion dont **une seule** des deux écritures est identique est refusée (levable).
- `DuplicatePolicy` du module commun ; `DbError::JournalEntryProbableDuplicate` et son `409 JOURNAL_ENTRY_PROBABLE_DUPLICATE` ; le message `error-journal-entry-probable-duplicate` ; l'audit `journal_entry.duplicate_confirmed` ; l'entrée `journal_entries::create_in_tx_with_policy` ; la lecture de la borne déplacée après le verrou de l'exercice ; le module d'états pur de l'écran (`entry-submit-state.ts`) — **que la 53-1b étend d'un champ facultatif `entry` : la lecture de `details.entry` est à sa charge** (V3-12 de la 53-1a) — et les clés `journal-entries-duplicate-existing`, `journal-entries-duplicate-confirm`, `journal-entries-duplicate-edit`, `journal-entries-unknown-outcome-link`.
- L'inventaire clos des sites (53-1a, lignes 2 et 3) et l'argument « tout geste qui change une donnée comparée tient l'exercice » (53-1a, inventaire des modifications).

## Ce qui se lit au code

Lu à `4d540a6ea`.

1. **Écran.** `ConversionAssistant.svelte` : `submit()` (`:249`), garde synchrone (`:250-251`) ; `await createConversion({ entries: sentPlan.entries })` (`:266`) ; une erreur qui n'est pas une `ApiError` affiche « La conversion n'a pas pu être passée. » (`:270-273`) — **affirmation fausse** quand l'issue est inconnue ; `finally { submitting = false; }` (`:295-297`) ; bouton d'envoi `:500`. Relevé C : `refreshRate` (`:101-115`, `.then` → `setCol`) et l'attente des `inflight` (`:254-260`). La page porte le retour au journal (`conversion/+page.svelte:76`).
2. **Route.** `create_conversion` (`routes/journal_entries.rs:1028`), corps `CreateConversionRequest { entries, complete_difference }` (`:933-944`), extracteur `ConversionBody` (`:946-950`), rejeu `retry_app_on_deadlock` (`:1062-1077`), refus d'une écriture portant `details.entry` (`conversion_refusal`, `:1001-1007`). `entries[i]` est un `CreateJournalEntryRequest`, que la 53-1a laisse sans le champ `confirmNotDuplicate` (corps propre à la saisie).
3. **Transaction de la conversion** (`journal_entry_conversions.rs`). `create_conversion` (`:226`) → `create_conversion_in_tx` (`:271`) : `lock_company_currency_in_tx` (`:283`, `companies` `LOCK IN SHARE MODE`, `accounts.rs:2358-2359`) ; `check_fx_accounts_in_tx` (`:298`, verrouillant) ; `conversion_line_accounts_in_tx` (`:329`, puis `:627` dans `complete_entry`, appelé `:391`) — **la sous-requête `EXISTS (SELECT 1 FROM accounts o …)` (`accounts.rs:2303`) est une lecture cohérente** : le doc-comment le dit (`accounts.rs:2278-2282`, « le verrou porte sur les comptes lus, **non** sur les lignes `o` … peut n'être pas vue ») ; **c'est elle qui fait naître l'instantané** ; `validate_fx_accounts_in_tx` (`:744`) ; puis, pour chaque écriture dans l'ordre chronologique (`:405-436`), `find_covering_date_in_tx` (`:413`, **lecture ordinaire**, `fiscal_years.rs:960-962`) et `journal_entries::create_in_tx` (`:432`).
4. **Second appelant** : `create_provider_conversion_in_tx` (`:508`) appelle `create_conversion_in_tx` dans **sa** transaction, branche `ConversionPlan::Double` (`:593`) ; il lit son exercice par `find_covering_date_in_tx` (`:536`) et reprend la société (`:568`). Flux gardé par l'état de la transaction bancaire (53-1a, inventaire, ligne 3).
5. **Aucun test `kesh-db` de la conversion** : `crates/kesh-db/tests/journal_entry_conversions.rs` n'existe pas ; les tests sont HTTP (`crates/kesh-api/tests/journal_entries_fx_ecrans_e2e.rs`, constante `ROUTE` `:698` ; `fx_recette_api_e2e.rs`, deux `POST` vers la conversion).

## Décisions

**D1. La règle sur la conversion (Q7).** Chaque écriture de la conversion passe la règle de la 53-1a **dans l'ordre de création** (chronologique) ; la première identique fait refuser **toute** la conversion (tout ou rien, inchangé), `409 JOURNAL_ENTRY_PROBABLE_DUPLICATE` avec `details.entry` = son rang **dans le corps** (`conversion_refusal`). Une seule identique suffit. Les deux écritures d'une même requête ne sont jamais comparées entre elles (le compte de conversion est au débit net de l'une et au crédit net de l'autre ; à écrire en commentaire). Avec `confirmNotDuplicate: true` **au niveau du corps** (`CreateConversionRequest`), une entrée `journal_entry.duplicate_confirmed` par écriture levée. `entries[i].confirmNotDuplicate` reste ignoré (la 53-1a ne l'a pas ajouté à `CreateJournalEntryRequest`). La chaîne de `impl_validation_extractor!` (`:946-950`) nomme le champ neuf.

**D2. Le correctif de V2-1 : le pré-verrou des exercices juste après la société.** Pour que l'instantané naisse après le verrou de **chaque** exercice comparé :
1. `create_conversion_with_policy` (et `create_conversion`, qui l'appelle avec `Refuse`) lit les exercices couvrant les deux dates **sur le pool, avant `begin`**, dans la fermeture rejouée ;
2. dans `create_conversion_in_tx`, **juste après** `lock_company_currency_in_tx` (`:283`) et **avant** `check_fx_accounts_in_tx` (`:298`) et `conversion_line_accounts_in_tx` (`:329`), les exercices trouvés se verrouillent **par clé primaire** (`SELECT id FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE`, une ligne chacun), **dans l'ordre chronologique** ;
3. la boucle de création ne relit plus l'exercice par `find_covering_date_in_tx` dans la transaction : elle reçoit les exercices pré-lus et évalue, pour chaque écriture **à son tour**, `NoFiscalYear` (aucun exercice trouvé) puis les refus de `create_in_tx_inner` (dont `FiscalYearClosed`, relu sous le verrou de l'étape 1, mappé `:432-437`) : **la précédence publiée ne change pas** (`docs/api-external.md:1217-1222`) ;
4. un exercice créé entre la lecture sur le pool et la transaction rend `NO_FISCAL_YEAR` comme une course perdue (bénin) ; des bornes changées sont relues sous verrou (étape 1, `DateOutsideFiscalYear`).
**Ordre des verrous** : `companies` (S) → exercices (X, chronologique) → comptes (S, `check_fx_accounts_in_tx`, `conversion_line_accounts_in_tx`) → [par écriture : projets (étape 0) → exercice repris → comptes (S) → compteur]. C'est l'ordre de la création ordinaire (exercice puis comptes en partage) ; un verrou ne refuse rien, donc aucun refus ne change.
**Inventaire écrit des lectures qui précèdent le pré-verrou** (à vérifier à l'étape 0) : `lock_company_currency_in_tx` (`:283`, `LOCK IN SHARE MODE`) — et rien d'autre ; **aucune lecture ordinaire**. Toute lecture ajoutée plus tôt rouvre la course (M2b).
*Écartée* : rendre la sous-requête `EXISTS` verrouillante (`accounts.rs:2303`) — pose des verrous partagés sur les comptes `o` parcourus, sans régler le cas d'une autre lecture cohérente future.

**D3. L'appelant de prestataire (`:593`, V2-4).** `create_conversion_in_tx` reçoit `duplicates: Option<DuplicatePolicy>` **obligatoire** et `exercices: Option<…>` (pré-lus) : pour `create_provider_conversion_in_tx`, `None` et `None` — **aucune comparaison, aucun pré-verrou**, la boucle garde la lecture actuelle (`find_covering_date_in_tx`) : l'ordre des verrous de ce flux automatique est **inchangé**. (Ou deux fonctions distinctes au lieu de deux `Option` couplées : au choix du développement, l'invariant étant « politique ⇔ exercices pré-verrouillés », qu'un type unique peut porter — p. ex. `Option<(DuplicatePolicy, ExercicesPreVerrouilles)>`.)

**D4. L'inversion projets/exercice (V2-5).** Avec le pré-verrou, l'étape 0 (sentinelle `companies` `FOR UPDATE` puis projets, `validate_taggable_in_tx`) de **chaque** écriture portant un projet vient **après** les exercices. Aujourd'hui, la première écriture tient la convention de Pattern 5 (sentinelle et projets avant l'exercice) ; seule la seconde l'inversait. **C'est une aggravation neuve** pour la première écriture (et la promotion S → X de `companies` se fait en tenant deux exercices) : un cycle possible avec un flux qui prend la sentinelle ou un projet puis un exercice ; défendu par le rejeu des deux côtés (la route est `Rejouee`, `audit_route_registry.rs:539`). Écrit au doc-comment « Cycles de verrous » du module (`:34-40` et suivantes).

**D5. L'assistant : table K.** Mêmes règles que la table J de la 53-1a, avec le module d'états commun :
| État | Entrée | Affiché | Actif | Sorties |
|---|---|---|---|---|
| **K1 Saisie** | ouverture ; « Modifier la saisie » depuis K5 ; reprise depuis K6 | l'assistant | tous les contrôles ; « Passer les écritures » selon le plan | envoi → K2 |
| **K2 Envoi** | depuis K1, K7 ; confirmation depuis K5 | **figé** | rien | succès → K3 ; `409 JOURNAL_ENTRY_PROBABLE_DUPLICATE` avec `confirmable` lisible → K5 ; sans `details` lisible → K7 ; `isSettlementOutcomeUnknown` → K6 ; autre refus → K7 (traitement actuel par colonne et par frais, `:274-294`) |
| **K3 Fait** | succès | toast actuel, `onDone()` | — | actuel |
| **K5 Doublon probable** | K2 | **figé** ; `journal-entries-conversion-probable-duplicate` : « L'écriture { $side } de cette conversion existe déjà à l'identique (écriture n° { $number } du { $date }). Si la conversion est déjà passée, ne la passez pas une seconde fois ; sinon, confirmez que ce n'est pas un doublon. » (`$side` : « du débit » pour `details.entry === 1`, « du crédit » pour 2) ; lien vers l'écriture (texte : `journal-entries-duplicate-existing`), nouvel onglet ; boutons `journal-entries-duplicate-confirm`, `journal-entries-duplicate-edit` | les deux boutons ; le lien | confirmation → K2, **corps gardé** `{ entries, confirmNotDuplicate: true }`, une fois ; « Modifier la saisie » → K1 |
| **K6 Issue inconnue** | K2 | **figé** ; `journal-entries-conversion-unknown-outcome` : « La conversion a peut-être été passée. Vérifiez dans le journal si ses deux écritures existent avant de la passer de nouveau. » ; lien (texte : `journal-entries-unknown-outcome-link`) vers `/journal-entries?dateFrom=min&dateTo=max` des deux dates, nouvel onglet ; bouton `journal-entries-conversion-unknown-outcome-resume` (« J'ai vérifié, elle n'a pas été passée — reprendre la saisie ») | le lien, la reprise | reprise → **K1, rien n'est envoyé** |
| **K7 Erreur ordinaire** | autre refus | saisie intacte ; traitement actuel | tous | → K2 |
Le retour au journal de la page (`conversion/+page.svelte:76`) reste actif dans tous les états (navigation, saisie perdue). **Figé** : dates, comptes, montants, cours, frais des deux colonnes, bouton d'envoi. Relevé C : `refreshRate` (`:101-115`) et l'attente des `inflight` (`:254-260`) n'écrivent rien hors K1/K7. **Le message faux disparaît** : une erreur qui n'est pas une `ApiError` mène à K6 ; la clé `journal-entries-conversion-error` se retire des quatre catalogues si elle n'a plus d'appelant (V2-10 : clés de K toutes nommées).

## Critères d'acceptation

1. **AC-1 (refus).** Le renvoi à l'identique d'une conversion passée rend `409 JOURNAL_ENTRY_PROBABLE_DUPLICATE`, `details.entry` (rang dans le corps de la première écriture comparée, ordre chronologique), `confirmable: true` ; **aucune** des deux écritures n'est écrite.
2. **AC-2 (une seule identique).** Une conversion dont une seule écriture est identique à une écriture existante est refusée ; la confirmation la fait passer.
3. **AC-3 (confirmation).** `confirmNotDuplicate: true` au corps : les deux passent, une entrée `journal_entry.duplicate_confirmed` par écriture levée ; non booléen : `400 VALIDATION_ERROR` (message de l'extracteur) ; `entries[i].confirmNotDuplicate` ignoré.
4. **AC-4 (concurrence, deux exercices).** Piloté : A crée par `create_in_tx` une écriture identique à la **seconde** écriture (chronologique) d'une conversion, dans F2, sans valider ; la conversion B (dates dans F1 et F2) attend le verrou de F2 (`attendre_une_requete_en_cours`) ; A valide ; B est refusée avec `details.entry` de la seconde. Même test pour la **première** (F1).
5. **AC-5 (précédence inchangée).** Les tests existants de chaque refus de la conversion restent verts ; `NO_FISCAL_YEAR` d'une écriture garde son `details.entry` ; un refus propre à la conversion (`FX_CONVERSION_TRANSIT_UNBALANCED`…) précède le doublon. Limite assumée : le doublon de la première écriture est rendu avant les refus de création de la seconde (exercice, période, comptes).
6. **AC-6 (prestataire inchangé).** Deux conversions de prestataire identiques (transactions distinctes, branche double) passent ; l'ordre des verrous de `create_provider_conversion_in_tx` est inchangé.
7. **AC-7 (écran).** K6 sur une réponse perdue (coupure réelle), reprise sans requête ; K5 nomme l'écriture du débit ou du crédit et renvoie le corps gardé ; jamais « La conversion n'a pas pu être passée » ; deux événements d'envoi : une requête.
8. **AC-8 (textes, manuel, API).** Clés de D5 dans les quatre locales, `lint-i18n-ownership`, `sitesTotal` recompté ; manuel utilisateur § *Convertir d'une devise à une autre* (`user-manual.tex:1333`, paragraphe `\textbf{Si la réponse se perd.}`) ; `docs/api-external.md` (`:1217-1222`) ; « Pour les intégrations » de la v0.21.0 complété.

## Tests et mutations

- **`kesh-db`** : **fichier neuf** `crates/kesh-db/tests/journal_entry_conversions.rs` (V2-8) — AC-1 à AC-6 côté dépôt, par `create_conversion_with_policy` (publique) et `create_in_tx` ; AC-4 piloté.
- **`kesh-api`** : `journal_entries_fx_ecrans_e2e.rs` complété (AC-1 à AC-3, AC-5 par HTTP) ; relevé des `POST` de conversion existants qui collisionneraient (`ROUTE`, `fx_recette_api_e2e.rs`), rendus distincts ou confirmés.
- **Vitest** : `ConversionAssistant.test.ts` ; **Playwright** : la spec de la 53-1a, ou un test de composant si le décor multi-devises ne se réutilise pas (dit).
- **Mutations** : M1 la conversion ne transmet pas la politique — AC-1 ; **M2b toute lecture ordinaire avant le pré-verrou** (p. ex. le pré-verrou remis après `conversion_line_accounts_in_tx`, ou `find_covering_date_in_tx` rétabli dans la transaction avant le pré-verrou) — **AC-4** ; M3 `details.entry` absent — AC-1 ; M4 l'appel `:593` passe une politique — AC-6 ; M5 K6 remplacé par le message d'échec — AC-7 ; M6 la confirmation de K5 renvoie un corps reformé — AC-7 ; M7 seule une écriture sur deux comparée — AC-2/AC-4.

## Tâches

**Étape 0** — relire chaque citation ; vérifier l'inventaire des lectures avant le pré-verrou (D2) ; vérifier que la 53-1a est fusionnée et que ses pièces communes ont la forme attendue.
- **S1** D2 et D3 dans `journal_entry_conversions.rs` ; doc-comments du module (ordre neuf, D4).
- **S2** Route : champ au corps, chaîne de l'extracteur, `create_conversion_with_policy` dans le rejeu.
- **S3** Tests Rust (fichier neuf), mutations M1 à M4, M7.
- **F1** Table K dans `ConversionAssistant.svelte` avec le module d'états commun, étendu de la lecture de `details.entry` (test unitaire) ; **F2** textes ; **F3** Vitest, M5, M6 ; **F4** Playwright ou test de composant ; **F5** recette visuelle (K1, K5, K6, thème clair, base de démonstration) ; **F6** manuel et API.

## Gates

Comme la 53-1a : gate complet et E2E complet au dernier commit de code ; repository touché : gate complet même en cours de boucle.

## Note de version

Avec celle de la 53-1a : **⚠️** `POST /api/v1/journal-entries/conversions` refuse désormais une conversion dont une écriture est identique à une écriture du même jour (`409 JOURNAL_ENTRY_PROBABLE_DUPLICATE`, `details.entry`) ; `confirmNotDuplicate: true` au corps lève le refus. **Corrigé (P1)** : l'assistant *Conversion* ne présente plus une issue inconnue comme un échec, et ne repasse pas une conversion sans confirmation (#1365).

## Dev Agent Record

*(À remplir au développement.)*

## Change Log

- 2026-11-25 — Fiche écrite au découpage de la 53-1 (décision du Project Lead, signal de recyclage V2-1 de la validation P2, `kesh-gate-logs/e531/validation-p2.md`). Porte le correctif de **V2-1** (pré-verrou juste après `lock_company_currency_in_tx`, ordre `companies` S → exercices X chronologiques → comptes S, inventaire écrit des lectures qui le précèdent, M2b redéfinie), **V2-4** (appelant `:593` : ni politique ni pré-verrou), **V2-5** (inversion projets/exercice neuve pour la première écriture, écrite comme aggravation), **V2-8** (fichier de test neuf) et la part conversion de **V2-10** (clés de K nommées). Décision Q7 du Project Lead écrite. Aucune passe de validation lancée.
- 2026-11-25 — Frontière avec la 53-1a précisée (V3-12 de la validation P3 de la 53-1a, `kesh-gate-logs/e531/validation-1a-p3.md`) : la lecture de `details.entry` appartient à la 53-1b, qui étend le module d'états commun.
