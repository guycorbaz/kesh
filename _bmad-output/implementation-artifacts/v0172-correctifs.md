# Story v0172-correctifs : la 0.17.2 — l'import simultané (#840), la liste des factures fournisseur (#870), le menu (#851), le taux de TVA désactivé (#882), les dates (#880), six LOW (#883), la date d'une pièce importée (#885)

Status: ready-for-dev

*Fiche rédigée le 2026-11-04, en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-0172b`, branche
`story/v0172-correctifs` partie de `d7ec2ea2` (le merge de la v0.17.1 sur `main`). Release **0.17.2**, **sans migration**
(§ *Contrainte*). Liste acceptée par le Project Lead le 2026-11-04. Registre : `v0172-choix-autonomes.md`, choix
`C-v0172b-1` à `C-v0172b-28` (ceux de #876 déplacés vers la section `v0172s`). Clé courte : `v0172b` (port 3173, bases
`kesh_v0172b` / `kesh_e2e_v0172b`). Issues : **#840** (bug, P2), **#870**, **#851**, **#882** (bugs, P3), **#880**, **#883** (bugs, P4), **#885** (demande), en `refs`
seulement dans les commits ; la fermeture se porte sur la PR de release. Les numéros de ligne sont ceux de `d7ec2ea2`.
Une autre story de la 0.17.2 (`v0172-annulees`, #868) travaille en parallèle sur une autre branche : CHANGELOG et registre
se fusionnent **par union** à l'intégration. Remédiée le 2026-11-04 après les validations P1 et P2 (Change Log).
**#876 (la session) a été extraite à la validation P2 dans la fiche sœur `v0172-session.md`** (même branche, clé de statut
`v0172-session`) : cette fiche n'en garde que des renvois.*

## Découpage — trois sous-stories dans cette fiche, et une fiche sœur (règle de splitting préventif)

Le périmètre touche plus de cinq modules. #876 (la session) est développée dans la **fiche sœur `v0172-session.md`**
(extraite à la validation P2 : la sévérité stagnait, 12 MEDIUM puis 11, et les défauts recyclés se concentraient sur elle ;
C-v0172b-1). Cette fiche garde **trois sous-stories**, développées **dans l'ordre A2 → B → C** sur la même branche, chacune close
par ses propres commits, son gate (complet, E2E complet au dernier commit de code) et sa revue de code avant la suivante. L'ordre
relatif de `v0172-session` et de cette fiche est libre : elles ne touchent aucun fichier commun, hors CHANGELOG, registre et
manuels (sections distinctes). Fichiers de chaque sous-story, **complets** :

| Sous-story | Issues | Fichiers (modules) | Nature |
|---|---|---|---|
| **A2 — l'import simultané** | #840 (P2) | (1) `crates/kesh-api/src/routes/bank_imports.rs` ; (2) `crates/kesh-api/tests/bank_imports_e2e.rs`, et, si `take_account_lock` est partagé, `crates/kesh-api/tests/common/mod.rs` et `tests/bank_statements_manual_e2e.rs` (déplacement du helper) ; (3) `frontend/src/lib/features/bank-import/BankImportUpload.svelte` et son test (AC-A10, si correction) | concurrence, serveur |
| **B — les écrans de saisie et de liste** | #870 (P3), #851 (P3), #882 (P3), #885 (CR) | (1) `routes/(app)/supplier-invoices/+page.svelte` et `supplier-invoices-list-page.test.ts` + `lib/features/fiscal-years/FiscalYearChecks.svelte` (#870) ; (2) `routes/(app)/+layout.svelte` + `tests/e2e/sidebar-navigation.spec.ts` (#851) ; (3) `lib/components/invoices/InvoiceForm.svelte` (#882) ; (4) `lib/features/supplier-invoices/SupplierInvoiceForm.svelte` + `tests/e2e/supplier-invoice-draft-cycle.spec.ts`, `tests/e2e/helpers/inbox.ts` (commentaire) (#885) ; (5) les quatre `messages.ftl` ; manuel FR | écrans |
| **C — la forme** | #880 (P4), #883 (P4) | `lib/features/reports/reports.api.ts` (helpers), les 32 fichiers du § 23 et leurs tests, `routes/(app)/settings/vat-rates/+page.svelte` (§ 24 bis), `crates/kesh-api/src/qr_decode.rs`, `crates/kesh-api/tests/inbox_import_e2e.rs` | déploiement mécanique d'un motif + tests |

A2 compte trois modules, B cinq ; C est un *rollout* mécanique (un helper, puis son application site par site), revu fichier
par fichier. **Ordre** : le P2 d'abord ; C en dernier (#883.6 change `formatSwissDate`, dont dépend #880 ; aucun site de B n'en
dépend).

## Story

En tant que Comptable qui importe un relevé, je ne veux pas qu'un double clic l'enregistre deux fois ; en
tant que Comptable qui clôture, je veux atteindre chaque brouillon de facture fournisseur que le contrôle de clôture me
nomme ; en tant qu'Administrateur, je veux que le menu garde ses entrées ; en tant que Comptable, je veux pouvoir corriger
un brouillon de facture dont un taux de TVA a été désactivé, lire les dates des écrans comptables en forme suisse, et partir
de la date du jour — signalée comme telle — sur une facture fournisseur créée depuis un document.

## Contrainte — production et migrations

- **Kesh 0.17.1 tient des livres réels sur le NAS du Project Lead** (`kesh-gate-logs/prompts/v017-donnees-a-garder.md`) :
  aucun essai de cette story ne se joue sur le NAS ni sur des données réelles ; fixtures synthétiques, bases `#[sqlx::test]`
  jetables, base de démonstration jetable `kesh_v0172b` pour les captures.
- **Aucune migration** (C-v0172b-21) : `ls crates/kesh-db/migrations/*.sql | wc -l` vaut **88** avant et après (recompté le
  2026-11-04 ; `docs/migrations-idempotence-audit.md:17` porte « 88 migrations »). #840 se corrige par un **verrou nommé**, non
  par une contrainte (§ A2, C-v0172b-6) ; #876 (fiche sœur) n'ajoute qu'un **champ JSON** aux réponses d'authentification. Aucune
  donnée existante n'est lue autrement, aucune n'est réécrite. `kesh_version_min_required` reste à 0.17.0 ; **le retour à la
  0.17.1 reste possible sans restauration** (aucune ligne de `_sqlx_migrations` n'est ajoutée). Pas de P5, P6, P7.
- **Hors périmètre** (release, faits par l'orchestrateur) : bump de version Cargo, macros des manuels (`kesh-style.sty`),
  tag. Les PDF des manuels sont régénérés par le développeur pour les seuls textes modifiés ici.

---

# #876 — la session : voir `v0172-session.md`

Extraite à la validation P2 (C-v0172b-1). Conception, AC, tests, mutations, manuels (utilisateur `:95` et l'inventaire clos du
manuel d'administration sur les cookies, les jetons et les durées), CHANGELOG et recette : dans la fiche sœur, choix
`C-v0172s-1` à `14`.

---

# Sous-story A2 — #840 : deux envois simultanés du même fichier enregistrent deux fois le relevé

## Ce que le code dit (d7ec2ea2)

8. **Le contrôle « fichier déjà importé » et l'`INSERT` ne sont pas exclusifs.** `create` (`routes/bank_imports.rs:1157-`)
   ouvre une transaction (`:1229-1234`), lit `find_by_company_and_hash` (`:1240`), puis plus loin
   `find_in_dedup_window` (`:1302`) et `create_import_or_roll_back` (`:1378`), commit (`:1395`). Même schéma pour le CSV :
   `create_csv` (`:2060-`), transaction `:2068`, contrôle `:2084`, fenêtre `:2232`, `INSERT` `:2304`, commit `:2321`.
   L'unicité `(company_id, file_hash)` a été retirée par `20260507000001_bank_imports_relax_hash_unique.sql` (le
   réimport **confirmé** — `confirmDuplicateFile` — est un usage légitime), et le compte n'est pris qu'en `LOCK IN SHARE
   MODE` (deux lecteurs compatibles). Sous REPEATABLE READ, deux requêtes simultanées voient chacune une base sans
   l'autre : deux relevés identiques, **sans confirmation**. Le commentaire `:1212-1223` le reconnaît (« limitations v0.1
   L11 race acceptée »). `create_import_or_roll_back` n'est appelé qu'à `:1378` et `:2304` (et la saisie manuelle,
   `bank_statements_manual.rs:439`) : ce sont les deux seuls chemins d'écriture d'un relevé depuis un fichier.
9. **Le remède existe dans le dépôt** — la saisie manuelle (v016-2a, C-v016-2-24) :
   `routes/bank_statements_manual.rs:211-283` prend `with_account_lock` (`kesh_reconciliation::mutex`,
   `GET_LOCK('reconcile:<base>:<société>:<compte>')`, `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS = 5`, `:287`) sur une connexion
   **dédiée** (`lock_tx`), crée dans sa **propre** transaction (`create_under_lock`), commite, puis rend le verrou ; ses
   tests : `bank_statements_manual_e2e.rs:1431` (`create_waits_for_the_account_lock_and_sees_the_committed_twin`,
   entrelacement forcé par `take_account_lock`, `:1408`, et `kesh_db::test_fixtures::attendre_une_requete_en_cours`) et `:1519`
   (`two_identical_simultaneous_creations_give_one_statement`). La table d'appariement des statuts (409) y est déjà écrite.

## Conception retenue (C-v0172b-6)

Les deux chemins de **fichier** (`create`, CAMT, et `create_csv`) prennent **le même verrou nommé du compte**, sur le motif
exact de `bank_statements_manual::create` : la section qui va de l'ouverture de la transaction à son commit (contrôle du
fichier identique, analyse, contrôles de solde, fenêtre de doublons, `INSERT`, trace) passe **telle quelle** dans une
fonction `…_under_lock(pool, …)` appelée dans la fermeture de `with_account_lock` ; le verrou est rendu **après** le
commit. L'ordre des refus est **inchangé** (aucun contrôle ne change de place), l'empreinte d'aperçu CAMT reste vérifiée avant
(`:1200-1209`). Délai : `MANUAL_STATEMENT_LOCK_TIMEOUT_SECS` réutilisée (pas de constante neuve) ; un verrou tenu au-delà
rend `409 RECONCILIATION_ACCOUNT_LOCKED`, sans écriture. Le chemin manuel et les chemins de fichier se sérialisent donc aussi
entre eux sur un même compte — c'est voulu (fenêtre de doublons commune). **Aucune migration** : une contrainte
d'unicité interdirait le réimport confirmé, et `#819` (clé de doublon des relevés déjà importés) est hors périmètre.

## Acceptance Criteria (A2)

**AC-A7 — Entrelacement forcé, CAMT et CSV.** Dans `crates/kesh-api/tests/bank_imports_e2e.rs`, sur le modèle de
`bank_statements_manual_e2e.rs:1431` (helper `take_account_lock` déplacé dans `tests/common` et partagé par les deux fichiers,
ou recopié ; `attendre_une_requete_en_cours` vient de `kesh_db::test_fixtures`) : une connexion de test tient le verrou du
compte ; le `POST /api/v1/bank-imports` (multipart CAMT) est **vu en attente** sur `GET_LOCK` ; la connexion enregistre alors
un relevé de **même `file_hash`**, commite, rend le verrou ; le `POST` rend `422 BANK_IMPORT_DUPLICATE_FILE` ; **un seul**
relevé en base. Même test pour un fichier CSV (profil de test existant). **Rouges avant correctif** (le `POST` n'attend rien
et rend `201`, deux relevés).

**AC-A8 — Deux envois lancés ensemble.** `tokio::join!` de deux `POST` identiques (CAMT), puis CSV : statuts `[201, 422]`,
le refusé porte `BANK_IMPORT_DUPLICATE_FILE`, **un** relevé et ses transactions une seule fois. Vert après correctif ;
avant, il est rouge **ou** vert selon l'ordonnancement — il n'est donc pas la preuve, AC-A7 l'est.

**AC-A9 — Le réimport confirmé et le délai.** `confirmDuplicateFile=true` sur un fichier déjà importé enregistre toujours
un second relevé (`201`, modificateur `duplicate_file` à la trace) — comportement 8-3 inchangé ; un verrou tenu 6 s par une
connexion de test rend `409 RECONCILIATION_ACCOUNT_LOCKED` et n'écrit rien. Les tests existants de `bank_imports_e2e.rs`
restent verts sans changement de leurs assertions.

**AC-A10 — L'écran dit le 409.** `BankImportUpload.svelte` affiche le message du 409 (chemin d'erreur générique existant,
`isApiError` → `err.message`) : le développeur le vérifie par un test Vitest (mock 409 `RECONCILIATION_ACCOUNT_LOCKED`) ; si
le message est déjà affiché, le test le fige ; sinon, correction. Le texte du 409 parle de rapprochement (le message est celui
des gestes de rapprochement) : limite déclarée en Dev Notes, non corrigée (C-v0172b-25).

## Tests et mutations (A2)

| # | Test | Attendu |
|---|---|---|
| TA5 | entrelacement forcé CAMT et CSV (AC-A7) | **rouges avant correctif** |
| TA6 | deux envois ensemble (AC-A8) | vert après |
| TA7 | réimport confirmé, délai (AC-A9) | réimport vert avant et après ; délai rouge avant (aucun verrou n'est attendu) |
| TA8 | Vitest 409 de l'import (AC-A10) | selon le code |

**Mutations nommées (P2)** :
- **MA11** — `create` (CAMT) appelle `…_under_lock` **sans** `with_account_lock` : TA5 CAMT rougit seul.
- **MA12** — idem pour `create_csv` : TA5 CSV rougit seul.
- *(MA13 — « le verrou rendu avant le commit » — est **retirée** à la validation P2 : dans TA5, c'est la connexion de test qui
  tient le verrou ; que le `POST` rende le sien avant ou après son commit n'y est jamais observé, et AC-A8 ne la tue que selon
  l'ordonnancement. L'inverse — le `POST` tient le verrou, une connexion de test l'attend et lit la base dès qu'elle l'obtient —
  demanderait un point d'arrêt de test dans le code de production. **Angle mort déclaré** au Dev Agent Record : l'ordre « commit
  puis rendu du verrou » est garanti par la forme du code (le commit est dans `…_under_lock`, appelée dans la fermeture,
  comme `bank_statements_manual.rs:239-249`) et vérifié à la revue de code, non par un test ; C-v0172b-28.)*

---

# Sous-story B — les écrans

## B.1 — #870 : la liste des factures fournisseur

### Ce que le code dit

10. `routes/(app)/supplier-invoices/+page.svelte:71-73` : `listSupplierInvoices(showCancelled ? { limit: 100 } : { limit:
    100, status: WITHOUT_CANCELLED })` ; aucune lecture de `page.url.searchParams`, aucun `offset`, aucun bouton de page.
    **Le serveur pagine déjà** : `ListSupplierInvoicesQuery { limit, offset, status }` (`routes/supplier_invoices.rs:640-`),
    écrêtage `LIMITE_DEFAUT = 50`, `LIMITE_MAX = 200` (`services/factures_fournisseur.rs:28, :30, :116`), `total` **filtré
    par les statuts** (`repositories/supplier_invoices.rs:218-` : `SELECT COUNT(*) … WHERE si.company_id = ?{filter}`), et
    `status` accepte une liste (`statuts_du_filtre`, `services/factures_fournisseur.rs:65`). **Aucune modification serveur.**
11. Le contrôle de clôture K8 (`lib/features/fiscal-years/FiscalYearChecks.svelte:280-311`) nomme vingt brouillons, dit
    « … et N autre(s), dans la liste des factures fournisseurs » et lie `href="/supplier-invoices"` (`:308`), sans filtre.
    Au-delà de cent factures (plus récentes d'abord), un brouillon ancien n'est atteignable nulle part. Le manuel décrit K8 à
    `user-manual.tex:823` (« Chaque brouillon listé mène à sa fiche ; lien : Ouvrir les factures fournisseurs ») sans dire ni les
    vingt premières ni « … et N autres ». *(La phrase de `:833` — « Pour les brouillons, le lien ouvre la liste complète » —
    porte sur les **factures client** en brouillon, K4 : `:819` « Factures en brouillon … Des factures client », `:834` « Seules
    les factures client en brouillon sont comptées » ; son lien est `/invoices?status=draft&dateFrom=…&dateTo=…`
    (`FiscalYearChecks.svelte:116`), liste paginée : elle est **vraie** et ne change pas. La première rédaction de cette fiche la
    lisait comme une phrase de K8 — finding P7 de la validation P2.)*
12. **Le motif de pagination** à reprendre : `routes/(app)/invoices/+page.svelte` — `initFromUrl` (`:71-107`, lecture et
    validation de `status`, `limit`, `offset`), `syncUrl` (`:116-153`, `goto(…, { replaceState: true, keepFocus: true,
    noScroll: true })`), boutons *Précédent* / *Suivant* et total (`:451-471`).

### Conception (C-v0172b-7)

La liste lit `?status=` (**une** valeur parmi `draft`, `open`, `paid`, `cancelled`), `?cancelled=1` (la case *Afficher les
factures annulées*), `?offset=` et `?limit=` (défaut 50,
borné à [1, 200] comme le serveur — non les 20/100 de `invoices/+page.svelte`). **L'URL est la source de vérité, lue de façon
réactive** (C-v0172b-26) : `status`, `offset`, `limit` sont des `$derived` de `page.url.searchParams` (`$app/state`), et un
`$effect` relit la liste quand ils changent ; la case est **elle aussi** dérivée de l'URL (`showCancelled = cancelled === '1'`,
C-v0172b-26) ; boutons, case et liens ne modifient pas un état local mais **naviguent**
(`goto(nouvelle URL, { keepFocus: true, noScroll: true })`). Le motif `initFromUrl` à `onMount` de `invoices/+page.svelte`
**n'est pas** repris tel quel : il ne relit pas l'URL quand seule la requête change (même instance de page), si bien qu'un lien
`<a href="/supplier-invoices">` (bandeau, menu latéral) depuis `?status=draft` laisserait la liste filtrée. Après lecture, si la
page est vide alors que `total > 0` et `offset > 0` (dernière ligne supprimée, signet ancien), la page navigue vers la dernière
page (`offset = floor((total − 1) / limit) × limit`, `replaceState`). Sans `status`, comportement 0.17.1 : sans les annulées (`status: ['draft','open','paid']`) ; cocher la case navigue vers
`?cancelled=1` (`offset` remis à 0), qui lit sans filtre de statut ; la décocher retire le paramètre. Avec `status`, `cancelled`
est ignoré. Avec
`status`, la liste ne montre que ce statut, la case est **masquée** et un bandeau dit le filtre (« Seules les factures
« Brouillon » sont affichées, tous exercices confondus. » + lien *Afficher toutes les factures* → `/supplier-invoices`). Le
filtre ne porte que sur le statut : depuis K8, la liste montre les brouillons de **tous** les exercices — un sur-ensemble de ceux
que K8 compte « datés dans l'exercice » (`closing_checks.rs:18`), si bien que le total de la liste peut dépasser le compte de K8 ;
l'issue ne demande pas de filtre de dates (C-v0172b-26). Une valeur inconnue est
ignorée (liste par défaut), jamais envoyée au serveur. Pied : « N facture(s) », *Précédent*, *Suivant*. Le lien de K8 devient
`/supplier-invoices?status=draft` ; son libellé « Ouvrir les brouillons de factures fournisseurs ». Hors périmètre : la limite
de 200 factures **validées** proposées à un lot de paiement (`routes/(app)/payment-batches/+page.svelte:54`), défaut de même
famille, non mécanique (un sélecteur paginé) — **issue demandée** (C-v0172b-19).

### Acceptance Criteria (B.1)

**AC-B1** — `/supplier-invoices?status=draft` appelle `listSupplierInvoices({ limit: 50, offset: 0, status: ['draft'] })`,
affiche le bandeau et masque la case ; `?status=bogus` appelle la liste par défaut (`status: ['draft','open','paid']`).
**AC-B2** — avec `total = 120` et `limit = 50` : page 1 *Précédent* désactivé ; *Suivant* appelle `offset: 50` et réécrit
l'URL `?offset=50` (statut conservé) ; page 3 (`offset=100`) *Suivant* désactivé ; « 120 factures ». Bascule de la case ou
retrait du filtre ⇒ `offset` revient à 0. Le jeton `requestSeq` (`:65`) reste en place. **Navigation par lien** : de
`?status=draft&offset=50`, un clic sur le lien du bandeau (ou un changement de `page.url` simulé vers `/supplier-invoices`)
relit la liste par défaut (`status: ['draft','open','paid']`, `offset: 0`) — **rouge sous le motif `onMount` seul** ; retour
arrière du navigateur ⇒ la liste suit l'URL. **Page au-delà du total** : `?offset=500` avec `total = 120` ⇒ navigation vers
`offset=100`, vingt lignes affichées, pied présent.
**AC-B3** — K8 : `fiscal-year-checks-link-DRAFT_SUPPLIER_INVOICES` (`FiscalYearChecks.svelte:280`, `check.code === 'DRAFT_SUPPLIER_INVOICES'`) a
`href="/supplier-invoices?status=draft"`. **AC-B4** — E2E `supplier-invoices.spec.ts` : 55 brouillons créés par l'API
(fixture), la liste filtrée par l'URL montre 50 lignes puis 5 après *Suivant*.
**AC-B2 bis** — la case : cocher appelle `goto` vers `?cancelled=1` puis lit `{ limit: 50, offset: 0 }` ; *Suivant* garde
`cancelled=1` ; retour arrière vers l'URL sans le paramètre ⇒ la case décochée et `status: ['draft','open','paid']`.
Tests : `supplier-invoices-list-page.test.ts` (existant ; `$app/state` simulé avec une `page.url` **modifiable**, et le simulacre
de `goto` (`:12`, aujourd'hui `goto: vi.fn()`) **modifie** cette `page.url`, sans quoi les lectures réactives ne se rejouent pas).
**Assertions existantes à mettre à jour** (raison au Change Log : `limit` passe de 100 à 50, `offset` est envoyé) : `:50`, `:59`,
`:68`, `:71`, `:104`, `:113`, `:136` (`{ limit: 100, … }` ⇒ `{ limit: 50, offset: 0, … }`) ; `FiscalYearChecks.test.ts`
(existant ou créé). **Rouges avant correctif** : AC-B1, AC-B2, AC-B3. E2E (AC-B4) : après *Suivant*, un clic sur l'entrée
*Factures fournisseurs* du menu ramène la liste par défaut, page 1.

## B.2 — #851 : des entrées du menu disparaissent

### Ce que le code dit

13. **L'hypothèse de l'issue (rôle momentanément nul) est réfutée au code.** `isAdmin` (`routes/(app)/+layout.svelte:45`)
    lit `authState.currentUser?.role` ; `login()` remplace `_currentUser` d'un seul tenant (`auth.svelte.ts:111-127`), sans
    passer par `null` ; `hydrate()` n'est appelé qu'au démarrage (`hooks.client.ts:22`) et sur un message inter-onglets
    (`auth.svelte.ts:330-336`) ; et la garde de `/admin/mcp` (`routes/(app)/admin/mcp/+page.ts:12-14`) **redirigerait vers
    `/`** si le rôle manquait au chargement. Surtout, un rôle nul ferait disparaître **tout** le bloc `adminOnly` (huit
    entrées : `awk '/adminOnly: \[/,/^\t\t\},/' … | grep -c 'href:'` → 8), pas « une partie des entrées ». *(Seule réserve : la
    branche 401 de `hydrate()` pose `_currentUser = null` sans rediriger — `auth.svelte.ts:261-` —, déclenchée par un message
    `auth-change` d'un autre onglet ; elle viderait **tout** le bloc, pas une partie. AC-B6 la garde.)*
14. **La cause lue au code : le menu déborde sans défiler.** La barre latérale est
    `<nav class="sticky top-14 flex h-[calc(100vh-3.5rem)] w-56 flex-col …">` (`:374-376`) : **hauteur fixe**, **aucun**
    `overflow-y`. Le groupe Administration porte 8 + 2 + 8 = **18 entrées** pour un Administrateur (`:119-190` ; recompté par
    `awk '/key: .administration./,/function getItemLabel/' … | grep -c 'href:'` → 18), Quotidien 11, Mensuel 4 — **33** au
    total ; en mode guidé chaque lien a `min-height: 44px` (`app.css:57`) : ≈ 33 × 46 px ≈ 1 520 px de liens pour une fenêtre
    de 720 px. Ce qui dépasse la hauteur de la barre se dessine **sous** elle ; collée (`sticky`) au haut de
    l'écran, elle défile avec la page : sur une page **courte** (dont `/admin/mcp`), le bas du groupe — justement les
    entrées « Serveur MCP », « Envois », « Prompts » et voisines — reste hors d'atteinte ou se cache sous le pied de page ;
    sur une page longue, il réapparaît en bas de défilement. D'où, **vraisemblablement**, « en ouvrant Serveur MCP (et
    d'autres) » : la chaîne causale exacte (pourquoi le clic, et non l'état statique) n'est **pas** mesurée — la reproduction
    d'AC-B5 la tranche. Ouvrir une
    entrée d'Administration déplie ce groupe (`$effect`, `:258-266`), ce qui allonge le menu.

### Conception (C-v0172b-8)

**Reproduire d'abord le symptôme** (AC-B5, oracle du symptôme), puis corriger : la `<nav>` reçoit `overflow-y-auto` et
`overscroll-behavior: auto` (la molette fait défiler la barre, puis la page), sa hauteur devient un **maximum**
(`self-start max-h-[calc(100vh-3.5rem)]` à la place de `h-[calc(100vh-3.5rem)]`, `top-14` inchangé) : avec un bandeau affiché
(`DemoBanner`, `IncompleteBanner`, `BackupScheduleBanner`, `+layout.svelte:362-369`) la barre commence `B` pixels plus bas ;
la molette, arrivée au bas de la barre, fait défiler la page de `B` pixels, la barre se colle sous l'en-tête et son dernier
lien entre dans la fenêtre (finding P10 de la validation P2). **Si l'oracle du symptôme ne rougit pas**, ne rien corriger :
relever les entrées absentes par capture, le dire au Dev Agent Record et au rapport (l'orchestrateur commente l'issue) ;
l'hypothèse du rôle (§ 13) reste écartée sauf preuve. **AC-B5 bis** n'est qu'une garde du correctif. Hors périmètre : le dépliage exclusif des groupes (issue voisine citée par #851).

### Acceptance Criteria (B.2)

**AC-B5 — l'oracle du symptôme** (`tests/e2e/sidebar-navigation.spec.ts`, Administrateur, fenêtre 1280 × 720, mode guidé). On
définit **atteignable** une entrée qu'un utilisateur peut viser **par ses propres gestes** : la souris posée sur la barre, des
crans de molette (`page.mouse.wheel(0, 200)`, au plus 20, jusqu'à stabilité), puis le lien est dans la fenêtre et
`document.elementFromPoint` au centre de sa boîte rend le lien lui-même (ni le pied de page, ni le contenu, ni un bandeau).
Scénario de l'issue : sur `/`, groupes à leur état par défaut (Quotidien et Mensuel dépliés, Administration replié) ; (1) relever
les entrées visibles des groupes dépliés, toutes atteignables ; (2) **ouvrir le groupe Administration** (clic sur son titre) ;
(3) relever chacune des **18** entrées d'Administration (liste écrite en dur, recomptée depuis `navGroups`) : atteignable ?
(4) cliquer *Serveur MCP* ; sur `/admin/mcp`, relever de nouveau les 18 entrées et celles des deux autres groupes. Le test
**rougit si une entrée visible au relevé précédent n'est plus atteignable** — c'est « des entrées disparaissent à l'ouverture d'un
groupe ». Il est joué **deux fois** : sans bandeau, et **avec** un bandeau affiché (réponse de l'API de `BackupScheduleBanner`
simulée par `page.route` pour l'afficher — route exacte relue au code par le développeur — ou base de démonstration pour
`DemoBanner`). Le résultat **avant correctif** décide : rouge ⇒ corriger, et la puce du CHANGELOG le dit ; vert ⇒ branche « ne
rien corriger » ci-dessus. Les relevés (atteignable / non, par entrée et par étape) vont au Dev Agent Record.
**AC-B5 bis — la garde du correctif** : après correctif, AC-B5 vert dans ses deux variantes ; et sans faire défiler la page
(`window.scrollY` à 0, sans bandeau), `nav.scrollHeight > nav.clientHeight` et `getComputedStyle(nav).overflowY` vaut `auto`.
**AC-B6** — le nombre d'entrées `nav-link-*` du groupe Administration est 18 sur `/admin/mcp`, `/admin/mcp/sends`,
`/admin/mcp/prompts`, `/users` et `/` (garde de l'hypothèse du rôle, vert avant et après). Captures : menu déplié sur
`/admin/mcp`, avant (le bas manquant) et après (barre défilée jusqu'en bas), sans et avec bandeau. **La puce du CHANGELOG de
#851 ne s'écrit qu'après AC-B5**, selon son résultat.

## B.3 — #882 : une ligne au taux de TVA désactivé

### Ce que le code dit

15. `InvoiceForm.svelte:54-74` : `vatOptions = rates.map((r) => r.rate)` depuis `getVatRates()` — **taux actifs seulement**
    (`repositories/vat_rates.rs:55`, `active = TRUE`). Le `<select bind:value={line.vatRate}>` (`:810-820`, `bind:value` à `:811`) n'a d'option que
    pour eux : une ligne de brouillon (`initLines`, `:119` ; `reloadInvoice`, `:616`) ou d'article (`onProductSelect`,
    `:430`) qui porte un taux désactivé depuis s'affiche **sans option choisie**, et `validate` refuse « Ligne N : taux TVA
    invalide » (`:496-497`) sans dire pourquoi. Le repli `:83-92` (un `$effect`) réécrit **en silence** en `vatOptions[0]`
    toute ligne dont `vatRate === '8.10'` quand `8.10` n'est plus actif — lignes relues d'un brouillon et d'un article comprises :
    un brouillon au taux le plus courant changerait de TVA sans avertissement.
15 bis. **Aucune réactivation d'un taux n'existe** : `routes/(app)/settings/vat-rates/+page.svelte:231-252` n'affiche les
    boutons *Changer le taux* et *Désactiver* que pour un taux actif (`{#if r.active}`) ; un taux inactif n'affiche que
    « Inactif » ; aucune route ne réactive un **taux** (`git grep -nE 'reactiv|set_active' -- crates/kesh-api/src/routes/vat.rs
    crates/kesh-api/src/routes/vat_rates.rs crates/kesh-db/src/repositories/vat_rates.rs` : aucun résultat ; le motif sur tout
    `routes/` rend la réactivation d'un **compte**, `routes/accounts.rs:280, :350, :358`, sans rapport). L'écran s'appelle *Taux de TVA* (`/settings/vat-rates`).
16. **Le serveur refuse aussi** : `create_invoice` / `update_invoice` appellent `verify_vat_rates_against_db`
    (`routes/invoices.rs:661, :749` → `routes/vat.rs`, `… WHERE company_id = ? AND active = TRUE AND rate IN (…)`) ⇒ `400`
    « Taux TVA non autorisé pour cette entreprise. » (`VAT_REJECTED_MSG`, `vat.rs:49`). Proposer le taux à l'écran ne
    suffirait donc pas à enregistrer.
17. **Le symétrique fournisseur existe** : `SupplierInvoiceForm.svelte:187-196` (`extraRates`, revue P1 de la v017-4b,
    E-7) propose les taux des lignes absents des taux actifs ; le serveur des factures fournisseur ne vérifie pas les taux
    (aucun appel `verify_vat_rates_against_db` dans `routes/supplier_invoices.rs`).

### Conception (C-v0172b-9)

Côté client seulement, **sans changer la règle du serveur** (accepter un taux désactivé sur une facture cliente touche à la
TVA, dont les dix P1 du jalon « Vague 2 » sont reportés) : (1) le `<select>` d'une ligne dont le taux n'est pas actif
propose ce taux, **libellé** `7.70% (désactivé)` — même forme que les options existantes, `{v}%` sans espace
(`InvoiceForm.svelte:818`) —, afin que la ligne ne s'affiche jamais vide ; l'ensemble des taux
proposés est **dérivé des lignes** (`$derived`), non calculé une fois, pour couvrir `onProductSelect` et `reloadInvoice` ;
(2) `validate` rend, pour une telle ligne, « Ligne N : le taux de TVA 7.70% a été désactivé. Choisissez un taux actif. »
(clé i18n neuve, quatre locales) — **aucune** action inexistante n'est suggérée (pas de réactivation, § 15 bis) —, à la place de « taux TVA invalide » — qui reste
pour une valeur inconnue qui n'est **ni** active **ni** portée par la ligne d'origine (cas qui ne se produit plus) ;
(3) choisir un taux actif retire le libellé et le refus ; (4) **le repli `:83-92` ne touche plus que les lignes neuves** :
une ligne créée avec la valeur provisoire `'8.10'` avant le chargement des taux porte une marque d'interface (`_vatPlaceholder:
true`, retirée par `stripUiKey` comme `_uiKey`) et seule une telle ligne est repointée ; une ligne relue (brouillon,
rechargement) ou venue d'un article garde son taux, et tombe sous (1)-(2) s'il est inactif (C-v0172b-27).

### Acceptance Criteria (B.3)

**AC-B7** — `InvoiceForm.test.ts` : un brouillon dont une ligne porte `7.70` (taux absent de `getVatRates()` mocké à
`['8.10','2.60','0.00']`) s'ouvre avec le `<select>` sur `7.70`, option libellée `7.70% (désactivé)` ; *Enregistrer* n'appelle pas l'API et affiche le message de C-v0172b-9 ; choisir `8.10` puis *Enregistrer* envoie
`vatRate: "8.10"`. Même chose après `reloadInvoice` (conflit de version) et pour une ligne ajoutée depuis un article dont
`vatRate = '7.70'`. **Rouge avant correctif** (le `<select>` a la valeur `''` ou la première option ; le message est
« taux TVA invalide »). **AC-B8** — une ligne à taux actif : aucune option « (désactivé) » n'apparaît (test négatif). **Cas 8.10** : un brouillon
dont une ligne porte `8.10`, taux absent des taux actifs mockés (`['2.60','0.00']`), s'ouvre sur `8.10% (désactivé)` — la ligne
n'est **pas** réécrite en `2.60` ; une ligne neuve créée avant la résolution des taux (valeur provisoire `8.10`) est, elle,
repointée sur `2.60`. **Rouge avant correctif** (le repli réécrit la ligne du brouillon). Le
serveur est inchangé (`git diff d7ec2ea2 -- crates/kesh-api/src/routes/vat.rs crates/kesh-api/src/routes/invoices.rs` vide).
Captures : brouillon rouvert avec la ligne au taux désactivé et son message.

## B.4 — #885 : la date d'une facture fournisseur créée depuis un document

### Ce que le code dit

18. `SupplierInvoiceForm.svelte:144-185` (`startState`) — à la création **avec** document : `invoiceDate: ''`, `dueDate:
    ''` (`:159-160`) ; sans document : la date du jour, échéance vide. Le commentaire (`:146-151`) explique #853 : « la date
    du jour n'est presque jamais celle d'une facture reçue ». À l'édition, `initial` prime (`:170-172`).
19. **La date du jour est la date UTC** : `new Date().toISOString().slice(0, 10)` (`:153`) — entre minuit et 1 h (2 h en
    été) en Suisse, c'est la veille. Le dépôt a l'helper local `todayLocal()` (`lib/features/open-items/open-items.ts:26`).
    La forme UTC se retrouve ailleurs : `git grep -nE '\.toISOString\(\)\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts'`
    rend **16** lignes (dont celle-ci) — formes en un temps (`new Date().toISOString()…`, dix fichiers) et en deux temps
    (`InvoiceForm.svelte:110` `todayIso()`, `journal-entries/form-helpers.ts:63`, `reminders/reminder-sent-at.ts:24`,
    `settings/+page.svelte:219`, `invoices/invoice-helpers.ts:124`, `settings/api-keys/+page.svelte:51`) ; certaines sont du
    calcul de dates, non une « date du jour », et sont à trier. Hors périmètre, **issue demandée avec les 16 lignes**
    (C-v0172b-20) ; seul le site de ce formulaire passe à `todayLocal()`.
20. **Prémisse de l'issue à corriger** : « l'échéance aussi [vide] (sauf conditions de paiement du fournisseur) » — le
    formulaire fournisseur **n'emploie jamais** les conditions de paiement du fournisseur : `git grep -nE
    'defaultPaymentTermsDays' -- frontend/src ':!*.test.ts'` ne le trouve que dans `InvoiceForm.svelte:406-407` (facture
    **cliente**, #245 : `addDaysIso(date, c.defaultPaymentTermsDays)`, `lib/features/invoices/invoice-helpers.ts`) et dans
    l'écran des contacts. Le serveur ne la calcule pas non plus (`routes/supplier_invoices.rs` : aucun
    `default_payment_terms_days`). L'échéance fournisseur démarre **toujours** vide.
21. **Le contrôle de #853** : `dueBeforeInvoiceDate` (`supplier-invoice-form.ts:180`), affiché sous le champ et bloquant
    (`SupplierInvoiceForm.svelte:207-208, :337, :591-597`) ; le refus serveur `SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE`
    (`routes/supplier_invoices.rs:550-556`). Inchangés.
22. **Les tests qui figent l'état actuel**, à mettre à jour avec la raison au Change Log : `SupplierInvoiceForm.test.ts:574-586`
    (« avec un document, la date démarre VIDE … ») ; `:588-600` (« une date vide a son propre message » : il ouvre un formulaire
    **avec document** sans poser de date — il devra vider la date d'abord) ; `tests/e2e/supplier-invoice-draft-cycle.spec.ts:593-604`
    (`toHaveValue('')`, aide visible, `supplier-invoice-date-error` attendu, capture `completion-date-vide`) — à réécrire en
    entier ; le commentaire du helper `tests/e2e/helpers/inbox.ts:228-232` (« démarre vide … ») devient inexact, le helper reste
    valide. (`:150-156` est le commentaire du helper `setDate`, non un test.)

### Conception (C-v0172b-15, 16)

- **Date** — à la création avec document, `invoiceDate = todayLocal()` et un état `dateToVerify = true` ; sous le champ, à la
  place de l'aide actuelle, un avis **visible** (`data-testid="supplier-invoice-date-to-verify"`, fond `warning/10`,
  `aria-live="polite"`) : « Date du jour, proposée par défaut — ce n'est pas celle du document. Vérifiez-la. » avec un
  bouton *C'est la bonne date* (`supplier-invoice-date-confirm`). L'avis disparaît quand l'utilisateur **modifie** la date
  (`oninput`) ou **confirme**. Le champ porte `aria-describedby` vers l'avis. **L'enregistrement n'est pas bloqué** par une
  date non confirmée (C-v0172b-15 : l'avis est visible, ce qui distingue l'état de #853, où la date du jour restait sans le
  dire). Sans document et à l'édition : aucun avis (comportement inchangé). Si l'utilisateur vide la date, le message
  « Saisissez la date de la facture, telle qu'imprimée sur le document. » reste.
- **Échéance** — à la création (avec ou sans document) : un état `dueFollows = true` ; tant qu'il l'est, `fDueDate =
  invoiceDate + defaultPaymentTermsDays du fournisseur choisi` (`addDaysIso`, `invoice-helpers.ts`), ou `invoiceDate` si
  le fournisseur n'en a pas ou si aucun fournisseur n'est choisi ; elle **se recalcule** quand la date ou le fournisseur
  change. Une saisie dans le champ échéance (`oninput`) met `dueFollows = false` ; elle n'est plus jamais recalculée. Une date
  vide ⇒ échéance vide (tant que `dueFollows`). À l'édition (`initial`), `dueFollows = false`. Le contrôle de #853 reste :
  une échéance saisie antérieure à la date est signalée et bloque.
- **Écartée** : bloquer l'envoi tant que la date n'est pas confirmée — c'est rouvrir le champ vide de #853 avec un clic de
  plus ; réversible si le Project Lead le souhaite.

### Acceptance Criteria (B.4)

**AC-B9** — `SupplierInvoiceForm.test.ts`, **fuseau épinglé** (`process.env.TZ = 'Europe/Zurich'` posé avant le test et rétabli
après, `delete process.env.TZ` s'il était absent — précédent : `routes/(app)/bank-accounts/bank-accounts-page.test.ts:702-718`)
**et** horloge fixée (`vi.setSystemTime('2026-11-04T23:30:00Z')`, soit le 5 novembre à 00:30 à Zurich) — sans le fuseau
épinglé, le test passerait à Zurich et rougirait sur le runner de la CI (UTC), et MB5 y survivrait : avec document, la date vaut `2026-11-05` (date **locale**, non UTC — la moitié qui rougit sous
`toISOString`) et l'avis « à vérifier » est visible ; sans document, `2026-11-05` et aucun avis ; à l'édition, la date de
`initial` et aucun avis.
**AC-B10** — l'avis disparaît après une saisie dans la date, ou après *C'est la bonne date* ; il ne revient pas.
**AC-B11** — les trois cas de l'échéance (issue) : fournisseur à 30 jours ⇒ échéance `2026-12-05`, et `2026-12-10` après
avoir porté la date au `2026-11-10` ; fournisseur sans délai ⇒ échéance = date, et elle suit la date ; échéance saisie à la
main (`2026-11-20`) ⇒ changer la date ou le fournisseur ne la touche plus. Changer de fournisseur (30 → 10 jours) recalcule
tant que l'échéance suit.
**AC-B12** — #853 inchangé : échéance saisie `2026-11-01` pour une date `2026-11-05` ⇒ message sous le champ, rien ne part ;
la requête envoyée sans toucher à rien porte `invoiceDate: '2026-11-05'` et `dueDate` calculée. Les sites du § 22 sont mis
à jour (la date ne démarre plus vide).
Captures : complétion d'un document, avis visible ; après *C'est la bonne date* ; échéance calculée d'un fournisseur à délai.

## Tests et mutations (B)

| # | Test | Attendu |
|---|---|---|
| TB1 | liste filtrée par l'URL, pagination (AC-B1, B2) | rouges avant |
| TB2 | lien de K8 (AC-B3) | rouge avant |
| TB3 | E2E 55 brouillons (AC-B4) | rouge avant |
| TB4 | E2E oracle du symptôme, sans et avec bandeau (AC-B5) | **décide** : rouge ⇒ correctif ; vert ⇒ aucun correctif |
| TB4 bis | E2E garde du correctif (AC-B5 bis) | rouge avant, vert après |
| TB5 | E2E nombre d'entrées (AC-B6) | vert avant et après |
| TB6 | Vitest taux désactivé (AC-B7, B8) | B7 rouge avant ; B8 vert |
| TB7 | Vitest date et avis (AC-B9, B10) | rouges avant |
| TB8 | Vitest échéance (AC-B11) | rouges avant |
| TB9 | Vitest #853 (AC-B12) | vert avant (contrôle) ; tests mis à jour |

Mutations : **MB1** `offset` non relu de l'URL ⇒ TB1 rougit ; **MB2** lien K8 sans `?status=draft` ⇒ TB2 ; **MB3**
`overflow-y-auto` retiré ⇒ TB4 et TB4 bis ; **MB3 bis** `h-[…]` rétabli à la place de `max-h-[…]` et `self-start` ⇒ la variante
« avec bandeau » de TB4 ; **MB4** l'ensemble des taux proposés calculé une fois (non dérivé) ⇒ le cas
`onProductSelect` de TB6 rougit ; **MB5** `toISOString` rétabli ⇒ TB7 (date locale) ; **MB6** `dueFollows` jamais remis à
`false` ⇒ le cas « saisie à la main » de TB8 ; **MB7** l'avis non retiré à la saisie ⇒ TB7 (AC-B10) ; **MB8** la lecture réactive de l'URL remplacée par une lecture à
`onMount` ⇒ le cas « navigation par lien » de TB1 ; **MB9** la marque `_vatPlaceholder` ignorée par le repli (toutes les lignes
`8.10` repointées) ⇒ le cas 8.10 de TB6 ; **MB10** `process.env.TZ` non épinglé — non une mutation du code mais la sonde du test :
le développeur vérifie que, sous `TZ=UTC`, MB5 rougit TB7.

---

# Sous-story C — la forme

## C.1 — #880 : les dates ISO des écrans comptables et des clés API

### Ce que le code dit — inventaire clos

23. **Méthode.** Le périmètre est l'ensemble des `.svelte` (hors tests) de ces neuf dossiers — **32 fichiers** (`git ls-files
    'src/routes/(app)/journal-entries/*.svelte' 'src/lib/features/journal-entries/*.svelte'
    'src/routes/(app)/settings/fiscal-years/*.svelte' 'src/lib/features/fiscal-years/*.svelte'
    'src/routes/(app)/bank-import/*.svelte' 'src/lib/features/bank-import/*.svelte' 'src/routes/(app)/reconciliation/*.svelte'
    'src/lib/features/reconciliation/*.svelte' 'src/routes/(app)/settings/api-keys/*.svelte' | grep -v '\.test\.'`, depuis
    `frontend/`, rend 32 lignes). Sur eux, deux motifs, relus ligne à ligne :
    `M₁ = '\{[^}]*(\.(date|[a-zA-Z]*Date|[a-zA-Z]*At|periodFrom|periodTo|startDate|endDate|[a-zA-Z]*On)\b|slice\(0, ?10\))'`
    (interpolations) et `M₂ = '(\b(date|from|to|start|end|period|since|until)[A-Za-z]*: [a-zA-Z])|toLocale(Date)?String|formatDate|new Date\('`
    (paramètres de messages i18n et formats maison), en excluant `bind:value|value=\{|onchange|oninput`.
24. **Les sites à corriger** (affichage d'une date ISO brute) — **24 sites** :

    | # | Site | Valeur | Forme cible |
    |---|---|---|---|
    | 1 | `routes/(app)/journal-entries/+page.svelte:551` | `entry.entryDate` (liste) | `formatSwissDate` |
    | 2 | `journal-entries/+page.svelte:366` | `booksLockedThrough` (bandeau « Livres verrouillés jusqu'au ») | idem |
    | 3 | `journal-entries/[id]/+page.svelte:590` | `entry.entryDate` (fiche) | idem |
    | 4–5 | `settings/fiscal-years/+page.svelte:459, :460` | `fy.startDate`, `fy.endDate` (liste) | idem |
    | 6–7 | `settings/fiscal-years/+page.svelte:636, :642` | `renameTarget.startDate`, `.endDate` (dialogue) | idem |
    | 8 | `lib/features/bank-import/BankImportList.svelte:38` | `imp.importedAt` (horodatage) | `formatSwissDateTime` |
    | 9 | `BankImportList.svelte:40` | `imp.periodFrom → imp.periodTo` (deux dates, une ligne) | `formatSwissDate` ×2 |
    | 10 | `BankImportUpload.svelte:513` | `preview.selectedStatement.periodFrom → …periodTo` | idem ×2 |
    | 11 | `BankImportUpload.svelte:544` | `preview.warnings.duplicateFile.existingImportedAt` | `formatSwissDateTime` |
    | 12 | `BankImportUpload.svelte:799` | `tx.bookingDate` (aperçu) | `formatSwissDate` |
    | 13 | `ManualStatementForm.svelte:629` | `previousPeriodTo` (message `{ $date }`) | idem |
    | 14 | `ManualStatementForm.svelte:648` | `` `${o.periodFrom} → ${o.periodTo}` `` (message `{ $periods }`) | idem ×2 |
    | 15–16 | `DeleteBankImportAction.svelte:150-151, :161-162` | `from`/`to` des deux messages de suppression | idem |
    | 17 | `routes/(app)/bank-import/[id]/+page.svelte:49` | `from`/`to` du message de succès de suppression | idem |
    | 18 | `bank-import/[id]/+page.svelte:139, :144` | `detail.importedAt` (« Saisi le », « Importé le ») | `formatSwissDateTime` |
    | 19 | `bank-import/[id]/+page.svelte:149` | `detail.periodFrom → detail.periodTo` | `formatSwissDate` ×2 |
    | 20 | `bank-import/[id]/+page.svelte:179` | `tx.bookingDate` (transactions) | idem |
    | 21 | `lib/features/reconciliation/ReconciliationProposals.svelte:480` (cellule) et `:461, :471` (`aria-label` « Sélectionner la transaction du { $date } ») | `p.transaction.bookingDate` | idem (C-v0172b-11) |
    | 22 | `ManualMatchModal.svelte:173` ; `TransactionSplitModal.svelte:234` | `proposal.transaction.bookingDate` | idem |
    | 23 | `LinkSettlementModal.svelte:205, :290, :350` | `valueDate ?? bookingDate`, `c.settledOn`, `e.entryDate` | idem |
    | 24 | `routes/(app)/settings/api-keys/+page.svelte:74-77` (`formatDate`, `value.slice(0, 10)` — sert `:94, :99, :104, :371`) et `:80-83` (`formatDateTime`, `value.slice(0, 16).replace('T', ' ')` — sert `:372`) | `createdAt`, `revokedAt`, `expiresAt`, `lastUsedAt` | les deux fonctions locales **délèguent** aux helpers communs (DRY) |

    Les sites multiples d'une même ligne du tableau comptent pour une ligne ; le décompte des **lignes de code** touchées va au
    Dev Agent Record, recompté.
24 bis. **Hors des neuf dossiers, un site voisin intégré** (finding P22 de la validation P2) :
    `routes/(app)/settings/vat-rates/+page.svelte:228-229` (`{r.validFrom}`, `{r.validTo ?? '—'}`, dates pures, `NaiveDate`,
    `kesh-db/src/entities/vat_rate.rs:36-37`) ⇒ `formatSwissDate` ; un test (`settings/vat-rates`, ou création s'il manque).
    Ce fichier n'entre pas dans le contrôle d'AC-C3 (qui porte sur les 32 fichiers) : son test le garde.
25. **Conformes ou hors classe, justifiés** : `FiscalYearChecks.svelte:73, :82, :215, :248, :294` et
    `journal-entries/[id]/+page.svelte:673` (déjà `formatSwissDate`) ; `FiscalYearChecks.svelte:116` (URL de lien,
    `dateFrom=${c.startDate}` — valeur envoyée, non affichée) ; `journal-entries/+page.svelte:79-80`,
    `journal-entries/[id]/+page.svelte:290, :319`, `JournalEntryForm.svelte:53, :85`, `settings/fiscal-years/+page.svelte:202-203`,
    `api-keys/+page.svelte:51, :56` (valeurs d'état, de calcul ou envoyées à l'API) ; `BankProfileForm.svelte:83, :91`
    (`date: dateIdx`, index de colonne CSV) ; `UnrejectTransactionAction.svelte:39` (`Intl.DateTimeFormat(i18nLocale(),
    { dateStyle: 'medium' })` — forme localisée, non ISO, C-v0172b-12) et `:64` (son paramètre `{ date: rejectedOn }`, déjà
    formaté) ; `api-keys/+page.svelte:366, :375` (classe CSS et condition sur `revokedAt`, aucun affichage) ;
    `BankImportUpload.svelte:104` (`totalErrors`) et `ReconciliationProposals.svelte:546` (`total`), rendus par `M₂` sur « to… » ;
    `ManualStatementForm.svelte:647` (`periods:` — la mise en forme se fait à `:648`, site 14) ; les `<input type="date">` (le
    navigateur les affiche dans sa langue). La liste close des lignes que le contrôle rend après correction est à AC-C3.
26. **Tests qui figent l'ISO** (à mettre à jour, raison au Change Log) : `DeleteBankImportAction.test.ts:103, :268`,
    `LinkSettlementModal.test.ts:287, :289, :292`, `ReconciliationProposals.test.ts:398, :428, :542`. Restent ISO, à bon droit :
    `ManualMatchModal.test.ts:125, :147` (valeur d'un `<input type="date">`), `reconciliation.api.test.ts:133` (corps de requête),
    `JournalEntryForm.edit.test.ts:191-192`, `form-helpers.test.ts:157-158`, `query-helpers.test.ts:30-31` (bornes, paramètres
    d'URL). Le développeur greppe aussi `frontend/tests/e2e/*.ts` (assertions de texte portant une date ISO sur ces écrans) et
    joint la sortie au Change Log.

### Conception (C-v0172b-10)

`formatSwissDate` (`lib/features/reports/reports.api.ts`) **tolère un horodatage** : si l'entrée commence par
`AAAA-MM-JJ` seul, elle rend `JJ.MM.AAAA` (date pure, aucun fuseau) ; si elle est un **horodatage** (`AAAA-MM-JJ` suivi de `T`
ou d'une espace, avec ou sans fraction, avec ou sans `Z` ou décalage `±HH:MM`), elle rend la **date à Zurich** de cet instant ;
toute autre entrée est rendue telle quelle — c'est #883.6. Un helper **voisin** `formatSwissDateTime(iso)` rend `JJ.MM.AAAA
HH:mm` **à l'heure de Zurich** pour un horodatage, la date seule pour une date pure, l'entrée telle quelle sinon. **Les
horodatages du serveur sont de l'UTC** (finding P11 de la validation P2) : `imported_at = Utc::now().naive_utc()`
(`routes/bank_imports.rs:1284`), session MariaDB en `time_zone = '+00:00'` (`kesh-db/src/pool.rs:40`), `existing_imported_at:
DateTime<Utc>` sérialisé avec `Z` (`:206` ; fixture `BankImportUpload.test.ts:133`, `'2026-01-02T00:00:00Z'`) ; les champs
`api_keys` sont des `NaiveDateTime` (`routes/api_keys.rs:37-40`) écrits par `NOW(3)`, UTC. Un horodatage **sans** `Z` ni décalage
est donc lu comme UTC (on lui ajoute `Z`), puis affiché par `Intl.DateTimeFormat('fr-CH', { timeZone: 'Europe/Zurich', … })`
(C-v0172b-10, révisé). Aujourd'hui, `api-keys` affiche l'heure UTC brute (`:82`). Les sites de la 0.17.1 qui passent un
horodatage UTC par `.slice(0, 10)` (fiche client : `paidAt`, `pdfFrozenAt`, `emailedAt` ; relances) donnent la date UTC — même
famille que C-v0172b-20, portés à son issue ; non corrigés ici. `api-keys/+page.svelte` garde `formatDate`/`formatDateTime` comme noms locaux (ils portent
le repli `—` et « Jamais utilisée ») mais délègue la mise en forme aux helpers.

### Acceptance Criteria (C.1)

**AC-C1** — `reports.api.test.ts`, **fuseau du processus épinglé** à `UTC` puis à `America/New_York` (le résultat ne doit
dépendre que de `Europe/Zurich`, non du poste ni du runner ; `process.env.TZ` posé et rétabli comme
`bank-accounts-page.test.ts:702-718`) : `formatSwissDate('2026-01-29') = '29.01.2026'` (date pure, inchangée),
`('2026-11-03T17:40:45') = '03.11.2026'`, `('2026-11-03T23:30:00') = '04.11.2026'` (UTC 23:30 ⇒ Zurich 00:30, hiver),
`('2026-07-01T22:30:00.123') = '02.07.2026'` (été, +2 h), `('2026-11-03 17:40') = '03.11.2026'`, `('') = ''`, `('29.01.2026') =
'29.01.2026'`, `('n/a') = 'n/a'` ; `formatSwissDateTime('2026-01-29T10:15:00') = '29.01.2026 11:15'`,
`('2026-01-02T00:00:00Z') = '02.01.2026 01:00'` (**cas `Z`**), `('2026-07-01T22:30:00+00:00') = '02.07.2026 00:30'`,
`('2026-01-29') = '29.01.2026'`, `('bogus') = 'bogus'`. **Rouge avant** pour tout horodatage (`'03T17:40:45.11.2026'`).
**AC-C2** — chaque site du § 24 et du § 24 bis affiche la forme suisse (les horodatages, à l'heure de Zurich ; le fuseau du
processus épinglé à `UTC` dans les tests qui lisent un horodatage) : un test Vitest **par écran** (liste et fiche du journal ; liste et
dialogue de renommage des exercices ; liste, aperçu et fiche des relevés ; dialogue de suppression et message de succès ;
avertissements de la saisie manuelle ; propositions de rapprochement, y compris l'`aria-label` ; les trois modales ; clés
API, quatre dates et la dernière utilisation) : `JJ.MM.AAAA` présent, `not.toContain` de l'ISO. Un écran qui porte deux
sites (p. ex. `bank-import/[id]` : `importedAt` et `bookingDate`) les lit **tous deux** dans son test.
**AC-C3** — **Contrôle décidable, inventaire clos.** Les motifs `M₁` et `M₂` du § 23, rejoués ensemble (`-E "M₁|M₂"`) sur
**les 32 fichiers**, avec l'exclusion `bind:value|value=\{|onchange|oninput` puis le filtre
`| /usr/bin/grep -vE 'formatSwissDate|formatSwissDateTime|formatDate\(|formatDateTime\('`, rendent :
- **sonde positive, sur `d7ec2ea2`** : **51 lignes** (64 avant le filtre ; relevé du 2026-11-04) — les **31 lignes** de code des
  sites du § 24 (BankImportList `:38, :40` ; BankImportUpload `:513, :544, :799` ; DeleteBankImportAction `:150, :151, :161,
  :162` ; ManualStatementForm `:629, :648` ; LinkSettlementModal `:205, :290, :350` ; ManualMatchModal `:173` ;
  ReconciliationProposals `:461, :471, :480` ; TransactionSplitModal `:234` ; `bank-import/[id]` `:49, :139, :144, :149, :179` ;
  `journal-entries` `:366, :551` ; `journal-entries/[id]` `:590` ; `fiscal-years` `:459, :460, :636, :642`) et les **20 lignes**
  ci-dessous ;
- **après correction** : **exactement ces 20 lignes**, identifiées par leur **contenu** (les numéros, ceux de `d7ec2ea2`, se
  recalent au Dev Agent Record) — toutes conformes ou hors classe : `BankImportUpload.svelte:104` (`totalErrors:` — compteur,
  M₂ sur « to… ») ; `BankProfileForm.svelte:83, :91` (`date: dateIdx`, index de colonne) ; `ManualStatementForm.svelte:647`
  (`periods: warnings.overlappingStatements` — la mise en forme se fait à `:648`) ; `FiscalYearChecks.svelte:116` (URL de lien) ;
  `JournalEntryForm.svelte:53, :85` (type, calcul) ; `ReconciliationProposals.svelte:546` (`total:` — montant) ;
  `UnrejectTransactionAction.svelte:39, :64` (`Intl`, C-v0172b-12, et son paramètre `{ date: rejectedOn }`) ;
  `journal-entries/+page.svelte:79, :80` (paramètres de requête) ; `journal-entries/[id]/+page.svelte:290, :319` (état) ;
  `settings/api-keys/+page.svelte:51, :56, :366, :375` (calcul de l'expiration, classe CSS et condition sur `revokedAt`) ;
  `settings/fiscal-years/+page.svelte:202, :203` (envoi à l'API). Toute autre ligne rendue est un site oublié ; une ligne de
  cette liste qui disparaît se justifie au Dev Agent Record. Une expression écrite sur plusieurs lignes dont la suite passe par un
  helper peut ajouter sa première ligne : elle est listée et relue.

## C.2 — #883 : les six LOW de la revue de la 0.17.1

27. **#883.1** — `crates/kesh-api/src/qr_decode.rs:8` : la ligne fait 121 colonnes (`awk '{print length}'`) ; refluer le
    doc-comment `:7-9` sous 100 colonnes, sans changer le texte.
28. **#883.2** — « des lignes Svelte dépassent probablement la largeur Prettier » : **sans objet**. Le dépôt n'a **ni
    Prettier ni configuration de largeur** (`grep -nE 'prettier|eslint' frontend/package.json` ne rend rien ;
    `git ls-files | grep -iE 'prettier|editorconfig'` ne rend rien) : aucune règle n'est enfreinte, et refluer à la main
    sans règle produirait du bruit de diff. Rien n'est changé ; le rapport demande à l'orchestrateur de le dire à l'issue
    (C-v0172b-13).
29. **#883.3** — `crates/kesh-api/tests/inbox_import_e2e.rs:667` : `assert_ne!(entry["qr"], "none", …)` suit
    `assert_eq!(entry["qr"], "present", …)` (`:666`) : retirer la ligne redondante.
30. **#883.4** — `est_payload_spc` (`qr_decode.rs:39-41`) : ses bornes figées par un test `v0172_est_payload_spc_bornes` —
    `"SPC\r\n0200\n…"` (en-têtes mêlés) **retenu** ; `""` **refusé** ; `"\u{FEFF}SPC\n0200\n…"` (BOM en tête) **refusé** — c'est
    le comportement actuel, figé tel quel, et la norme SIX n'admet pas de BOM (C-v0172b-14) ; `"SPC\n0200"` sans suite
    **retenu** par le préfiltre — et `first_spc` sur ce payload rend `Err(InvalidSpcPayload)` (le parseur le refuse, moins de
    31 lignes).
31. **#883.5** — `v0171_t4_crlf_inexploitable` (`qr_decode.rs:1234-1244`) emploie `Faux::SpcCrlfInvalide` = `qr("SPC\r\n0200\r\n1\r\nPAS-UN-IBAN")`
    (`:1056`) : 4 lignes, refusé au compte de lignes avant tout en-tête. Remplacer la fixture par un payload CR+LF **de 31 lignes
    au moins** : produire le texte valide par `build_payload(&donnees_spc_test())` (qui **valide** l'IBAN,
    `kesh-qrbill/src/generator.rs:15`, `validate(data)?` : un IBAN invalide dans les données ferait échouer l'`unwrap()`),
    **puis** remplacer la ligne de l'IBAN dans le texte produit par une valeur invalide, **puis** passer en CR+LF ; attendre
    `Err(DecodeError::InvalidIban(_))`, et récrire le doc-comment du test (`qr_decode.rs:1231-1232`, « `InvalidSpcPayload`, comme
    son jumeau LF », devient faux) : il rougit alors **aussi** sous la mutation qui retire
    `trim_end_matches('\r')` du parseur (`kesh-qrbill/src/parser.rs:95`), qui rend `InvalidSpcPayload` dès l'en-tête `SPC\r`.
    Le jumeau LF existant reste.
32. **#883.6** — traité par C.1 (`formatSwissDate` tolère l'horodatage, AC-C1).

**AC-C4** — #883.1, .3 : `git diff` montre le reflux et la ligne retirée, rien d'autre ; #883.4 : le test des bornes, vert ;
#883.5 : le test rougit sous **MC3** ci-dessous ; #883.2 : aucun fichier Svelte modifié pour la largeur.

## Tests et mutations (C)

| # | Test | Attendu |
|---|---|---|
| TC1 | `formatSwissDate` / `formatSwissDateTime` (AC-C1) | horodatage rouge avant |
| TC2 | Vitest par écran (AC-C2) | rouges avant |
| TC3 | bornes de `est_payload_spc` (#883.4) | vert avant et après |
| TC4 | `v0171_t4_crlf_inexploitable` réécrit (#883.5) | vert avant et après ; rouge sous MC3 |

Mutations : **MC1** `formatSwissDate` rendu à sa forme actuelle (`split('-')`) ⇒ TC1 ; **MC2** retirer le helper d'un site
(une mutation par écran) ⇒ son test seul ; **MC3** retirer `.map(|l| l.trim_end_matches('\r'))` de `parser.rs:95` ⇒ TC4
rougit (il ne rougissait pas avec la fixture de 4 lignes) ; **MC4** `est_payload_spc` accepte tout préfixe `SPC` ⇒ TC3.

---

## Tâches

Par sous-story, dans l'ordre A2, B, C (#876 : fiche sœur) ; chacune : tests rouges d'abord (constatés et notés), correctif, mutations, gates,
captures, commit `refs #N`, revue de code.

1. **A2** — TA5 rouges ; verrou nommé des deux chemins d'import ; `take_account_lock` partagé ; AC-A10.
2. **B** — TB1–TB4, TB6–TB8 rouges ; liste paginée et filtrée, lecture réactive de l'URL ; lien K8 ; `overflow-y-auto` après
   reproduction ; taux désactivé et repli limité aux lignes neuves ; date et échéance ; i18n quatre locales
   (`crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`) ; `npm run lint-i18n-ownership` ; manuel.
3. **C** — TC1, TC2 rouges ; helpers ; les 24 sites ; tests ISO mis à jour ; #883.1, .3, .4, .5 ; AC-C3.
4. **Gates** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/test-fast.sh`
   (sous `flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock` et `mem-guard`), `cd frontend && npm run check && npm run
   lint-i18n-ownership && npm run test:unit && npm run build`, E2E complet au **dernier commit de code** de **chaque**
   sous-story (7 à 10 échecs attendus, `docs/testing.md`) ; base remise à zéro avant chaque gate complet. A2 touche
   `bank_imports.rs` (aucun repository, aucune migration) : gate ciblé permis en boucle, complet au push.
5. **Captures** (thème clair, base de démonstration jetable `kesh_v0172b`, dans
   `/home/gcorbaz/devel/kesh-gate-logs/v0172b/captures/`) : B — liste filtrée « Brouillon » avec son bandeau, pied de pagination,
   lien K8 ; menu déplié sur `/admin/mcp` avant/après ; brouillon au taux désactivé ; complétion d'un document avec l'avis, après
   confirmation, échéance calculée. C — un écran par ligne du tableau du § 24 (journal liste et fiche, exercices, relevés liste,
   aperçu, fiche, dialogue de suppression, saisie manuelle avec avertissement, rapprochement et ses trois modales, clés API).
   A2 : capture du message 409 si le texte change (AC-A10).

## Manuel

Le PDF de chaque manuel touché est régénéré par `make fr` dans `docs/manual/`, puis contrôlé aplati (`pdftotext
docs/manual/fr/<manuel>.pdf - | tr '\n' ' ' | tr -s ' '`, puis `grep -F` de chaque phrase neuve **et** de chaque phrase retirée).

**`docs/manual/fr/user-manual.tex`**
- **#870** — `:823` (K8) seulement : le panneau nomme les vingt premiers brouillons et dit « … et N autres » ; le lien
  *Ouvrir les brouillons de factures fournisseurs* ouvre la liste **filtrée sur les brouillons, de tous les exercices** (le compte
  de la liste peut donc dépasser celui de K8). `:833` (K4, factures client) **ne change pas** ; § des factures fournisseur (`:1888-1892`, « Les factures annulées dans la liste ») :
  la liste est paginée (50 par page, *Précédent* / *Suivant*) et se filtre par statut depuis l'adresse.
- **#885** — `:1607-1612` (« Les dates ») et `:2049-2053` (« Créer la facture d'un document ») : à côté d'un document, la date est
  **proposée au jour même et marquée « à vérifier »** jusqu'à ce que vous la modifiiez ou la confirmiez ; l'échéance suit la date
  (plus le délai de paiement du fournisseur, s'il en a un) tant que vous ne l'avez pas saisie ; une échéance antérieure reste
  signalée. Le rappel historique « depuis la version 0.17.0 : la date du jour n'est presque jamais celle d'une facture reçue »
  devient « depuis la version 0.17.2, elle est proposée et signalée ».
- **#882** — § *Création d'une facture* (`:1034`), puce *Lignes de facture* (`:1045`) : une ligne dont le taux a été désactivé
  l'affiche « (désactivé) » ; la facture ne s'enregistre qu'avec un taux actif, à choisir dans la ligne (un taux désactivé ne
  se réactive pas : *Paramètres → Taux de TVA*, `:1014`, ne propose que *Changer le taux* et *Désactiver* pour un taux actif).
- **#851, #880, #883** : rien à décrire (le manuel ne décrit ni la forme des dates de ces écrans — `grep -nE 'AAAA-MM-JJ|jj\.mm'
  docs/manual/fr/user-manual.tex` à joindre au Change Log — ni la hauteur du menu).

**`docs/manual/fr/admin-manual.tex`** : les phrases sur les cookies, les jetons et les durées relèvent de #876 (inventaire clos
dans `v0172-session.md`, § *Manuel*) ; aucune autre n'est touchée ici.

- `docs/manual/{de,en,it}/` : `ls` ne rend que `README.md` (aucun `.tex`) — rien à traduire.

## CHANGELOG

Section commune `## [0.17.2] — Non publié`, au-dessus de la 0.17.1 (l'autre story de la 0.17.2 y écrit aussi : **fusion par union**,
sans réécrire ses puces). Intro, si elle n'existe pas encore : « La 0.17.2 est une version corrective : **aucune migration**, aucune
donnée réécrite, **le retour à la 0.17.1 est possible sans restauration**. » (à vérifier contre la story voisine à l'intégration.)

`### Corrigé`
- **Un relevé importé deux fois en même temps n'est plus enregistré deux fois** ([#840](https://github.com/guycorbaz/kesh/issues/840)) :
  un double clic ou un envoi rejoué enregistrait deux relevés identiques sans confirmation ; le second est désormais refusé comme
  fichier déjà importé.
- **Toutes les factures fournisseur sont atteignables** ([#870](https://github.com/guycorbaz/kesh/issues/870)) : la liste se pagine
  (au-delà des cent premières, les plus anciennes n'apparaissaient nulle part) et se filtre par statut depuis l'adresse ; le lien du
  contrôle de clôture ouvre les brouillons, de tous les exercices.
- **Le menu garde toutes ses entrées** ([#851](https://github.com/guycorbaz/kesh/issues/851)) : *puce rédigée après l'oracle du
  symptôme (AC-B5), selon son résultat.*
- **Un brouillon de facture dont un taux de TVA a été désactivé se corrige** ([#882](https://github.com/guycorbaz/kesh/issues/882)) :
  la ligne affiche son taux « (désactivé) » au lieu d'un champ vide, le refus dit de choisir un taux actif, et un brouillon au taux
  de 8,1 % désactivé ne change plus de taux en silence.
- **Les dates des écrans comptables s'affichent `29.01.2026`** ([#880](https://github.com/guycorbaz/kesh/issues/880)) : journal,
  exercices, relevés bancaires, rapprochement, taux de TVA et clés d'API ; les heures (import d'un relevé, dernière utilisation d'une
  clé) s'affichent à l'heure suisse, non plus en UTC.
- Points de forme de la revue de la 0.17.1 ([#883](https://github.com/guycorbaz/kesh/issues/883)).

`### Ajouté`
- **La date d'une facture fournisseur créée depuis un document est proposée** ([#885](https://github.com/guycorbaz/kesh/issues/885)) :
  la date du jour, **marquée « à vérifier »** jusqu'à ce que vous la confirmiez ou la changiez ; l'échéance suit la date, plus le délai
  de paiement du fournisseur s'il en a un. Une échéance antérieure à la date reste signalée.

(La puce de #876 est prescrite par `v0172-session.md`.) `refs #840`, `refs #870`, `refs #851`, `refs #882`, `refs #880`, `refs #883`, `refs #885` aux commits ; **aucun mot-clé de
fermeture** (contrôle : `git log origin/main..HEAD --format=%B | /usr/bin/grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'`
vide). La PR de release porte les `closes`.

## Dev Notes

- **Jamais de donnée réelle** : fixtures CAMT/CSV synthétiques (celles de `bank_imports_e2e.rs`), contacts et factures de démonstration.
- **#840** : le verrou est celui du rapprochement — un import attend donc un rapprochement en cours sur le même compte (≤ 5 s), et
  inversement. C'est le comportement de la saisie manuelle depuis la 0.16. **Deux connexions par import** (`lock_tx` puis la
  transaction de `…_under_lock`) pour un pool de **5** (`crates/kesh-api/src/main.rs:83`, `.max_connections(5)`) : cinq envois
  simultanés sur un même compte peuvent occuper le pool en attente du verrou, et le détenteur attendre sa seconde connexion jusqu'à
  `acquire_timeout` — même risque, déjà accepté, que la saisie manuelle ; déclaré, non corrigé. Sur le refus « profil CSV
  introuvable », `resolve_csv_profile` (`bank_imports.rs:801-`) appelle `list_available_profiles(pool, …)` (`:822`, `:841`) :
  **trois** connexions le temps du refus. Un import CSV long (> 5 s sous le
  verrou) fait rendre au second clic `409 RECONCILIATION_ACCOUNT_LOCKED` (texte du rapprochement), non `422` (C-v0172b-25).
- **#851** : ne pas toucher `expandedState` ni la persistance des groupes.
- **#880** : ne pas toucher aux `<input type="date">` ni aux valeurs envoyées à l'API.
- **Aucune migration** : `ls crates/kesh-db/migrations/*.sql | wc -l` = 88 au dernier commit (comme à `d7ec2ea2`) ; pas de ligne à
  `docs/migrations-idempotence-audit.md`.

## Change Log

- 2026-11-04 — fiche créée (Opus 5.5, autonomie). Prémisses établies au code (`d7ec2ea2`) : #876 **confirmée et aggravée** (le
  renouvellement sur 401 échoue toujours avec 15/15, § 1-4) ; #840 confirmée (§ 8) ; #870 confirmée, serveur déjà paginé (§ 10) ;
  #851 : hypothèse de l'issue (rôle nul) **réfutée**, cause lue au code = débordement de la barre latérale (§ 13-14), à reproduire ;
  #882 confirmée, le serveur refuse aussi (§ 15-16) ; #880 inventaire clos de 24 sites (§ 23-26) ; #883 : .2 **sans objet**
  (aucun Prettier, § 28) ; #885 : prémisse « l'échéance vient des conditions du fournisseur » **fausse** (§ 20), date du jour en
  UTC (§ 19). Découpage en trois sous-stories A/B/C (C-v0172b-1). Registre C-v0172b-1 à 22. Aucune migration. Prochaine :
  validation P1 (Sonnet, lentille complète, `v0172-correctifs-validate-prompt-p1.md`).
- 2026-11-04 — **Validation P1 (Sonnet) : 0/0/12/10** (rapport `kesh-gate-logs/v0172b-validate-p1.md`, lentille complète R+F).
  Chaque prémisse relue au code avant application ; les 22 findings sont **appliqués**, aucun réfuté. Lignes lues : P12 `ls
  crates/kesh-db/migrations/*.sql | wc -l` → **88** (et non 86) ; P7 `awk '/adminOnly: \[/,/^\t\t\},/' …/+layout.svelte | grep -c
  'href:'` → 8, `awk '/key: .administration./,/function getItemLabel/' … | grep -c 'href:'` → 18 (11 + 4 + 18 = 33) ; P8
  `settings/vat-rates/+page.svelte:231-252` (`{#if r.active}` autour des boutons) ; P9 `InvoiceForm.svelte:83-92` ; P10
  `bank-accounts-page.test.ts:702-718` (`process.env.TZ = 'Europe/Zurich'`) ; P4 `admin-manual.tex:111, :778, :779, :2506,
  :2995, :2996` lus, plus `:3886` et le stockage en clair du jeton (`refresh_tokens.rs:30-34`) trouvés en refermant l'inventaire ;
  P11 sonde rejouée sur `d7ec2ea2` : 64 lignes avant filtre, 51 après, dont 31 sites et 20 hors classe ; P19 `git grep -nE
  '\.toISOString\(\)\.slice\(0, ?10\)'` → 16 lignes ; P17 `main.rs:83` `.max_connections(5)`. Sorts :
  **P1–P3** (A1) → conception refaite : renouvellement **à l'entrée**, au plus une fois par pas `T`, sans minuterie (borne
  d'inactivité « entre 14 et 15 minutes » par défaut, écrite au manuel, au CHANGELOG et à AC-A6) ; veille et onglet masqué couverts
  par l'horloge murale relue à l'entrée et par `visibilitychange` (AC-A3 f, g) ; échec transitoire du renouvellement anticipé sans
  déconnexion (AC-A3 h, C-v0172b-24) ; garde « 401 sans entrée » (AC-A3 i) ; mutations MA1–MA10 ; C-v0172b-2 révisé, C-v0172b-23
  et 24 ajoutés. **P4** → inventaire clos du manuel d'administration (tableau du § *Manuel*), issue demandée pour le jeton stocké
  en clair. **P5** → découpage en quatre sous-stories (A1 #876, A2 #840, B, C), fichiers complets au tableau, écouteurs posés par le
  store (non par le layout), C-v0172b-1 révisé. **P6** → lecture réactive de l'URL, test de navigation par lien, MB8 (C-v0172b-26).
  **P7** → 18 / 33 / 8 propagés (§ 13-14, AC-B5, AC-B6, C-v0172b-8). **P8** → message sans réactivation (§ 15 bis). **P9** → repli
  `8.10` limité aux lignes neuves, cas 8.10 à AC-B8, MB9 (C-v0172b-27). **P10** → `process.env.TZ` épinglé à AC-B9. **P11** →
  § 25 et AC-C3 : liste close des 20 lignes et sonde chiffrée. **P12** → 88 partout. LOW : **P13** → § 22 (`:588-600`,
  `draft-cycle.spec.ts:593-604`, `inbox.ts:228-232`) ; **P14** → page au-delà du total ramenée à la dernière page (AC-B2) ;
  **P15** → AC-B5 sans défilement de la page, puce #851 rédigée après reproduction ; **P16** → désarmement à `clearSession`/`logout`
  (AC-A3 j), module inerte dans Vitest, comparaison `≥` et horloge avancée (AC-A4 a) ; **P17** → Dev Notes ; **P18** → libellé
  `7.70% (désactivé)` ; **P19** → C-v0172b-20 porte les 16 lignes ; **P20** → « de tous les exercices » au bandeau, au manuel, au
  CHANGELOG ; **P21** → ancre `:1032`/`:1040` ; **P22** → `config.rs:688-708, :735-755`, `auth.rs:379`, `auth.svelte.ts:245`.
  Valeurs corrigées grepées dans la fiche, le registre et le prompt (`\b86\b`, `\b19\b`, `\b34\b`, « neuf entrées »). Registre
  C-v0172b-1 à 27. Prochaine : validation P2 (Opus, complète, `v0172-correctifs-validate-prompt-p2.md`).

- 2026-11-04 — **Validation P2 (Opus) : 0/0/11/12** (rapport `kesh-gate-logs/v0172b-validate-p2.md`, lentille complète).
  **Découpage** : signal de stagnation (12 MEDIUM puis 11, six des onze nés de la remédiation P1, les défauts recyclés concentrés sur
  #876) — **#876 extraite** dans la fiche sœur `v0172-session.md` (clé de statut `v0172-session`, choix renumérotés `C-v0172s-1` à
  `14`, correspondance au registre) ; cette fiche garde A2, B et C et ne porte plus de #876 que des renvois (titre, découpage,
  contrainte, § A1 remplacé, Tâches, Manuel, CHANGELOG, Dev Notes). Findings traités **ici**, prémisse relue au code :
  **P6** (MEDIUM) appliqué — MA13 retirée, angle mort déclaré (C-v0172b-28) ; ligne lue : AC-A7, la connexion de test tient le
  verrou. **P7** (MEDIUM) appliqué — `user-manual.tex:819`, `:823`, `:833`, `:834` lus : `:833` porte sur K4 (factures client) et est
  vraie ; seule `:823` (K8) change ; § 11 corrigé. **P8** (MEDIUM) appliqué — `?cancelled=1`, AC-B2 bis, assertions
  `supplier-invoices-list-page.test.ts:50, :59, :68, :71, :104, :113, :136` nommées, simulacre de `goto` (`:12`) qui modifie
  `page.url`. **P9, P10** (MEDIUM) appliqués — `+layout.svelte:362-369` (bandeaux) et `:376` lus : oracle du **symptôme** (entrées
  atteignables par la molette avant et après l'ouverture du groupe Administration puis de *Serveur MCP*, sans et avec bandeau), dont
  le résultat décide du correctif ; barre bornée par un maximum (`self-start max-h-[…]`) ; AC-B5 bis garde ; MB3 bis (C-v0172b-8
  révisé). **P11** (MEDIUM) appliqué — `bank_imports.rs:206, :1284`, `BankImportUpload.test.ts:133`, `pool.rs:40`, `api_keys.rs:37-40`
  lus : horodatages UTC affichés à l'heure de Zurich, cas `Z` et décalage, `process.env.TZ` épinglé à AC-C1 (C-v0172b-10 révisé).
  LOW : **P18** appliqué (tableau d'A2 complété, troisième connexion du refus de profil, `bank_imports.rs:801, :822, :841`) ; **P19**
  appliqué (`git grep` restreint aux fichiers des taux) ; **P20** appliqué (`user-manual.tex:1034`, `:1045` ; `syncUrl` à
  `invoices/+page.svelte:116`) ; **P21** appliqué pour C-v0172b-7 (C-v0172b-5 est devenu C-v0172s-4) ; **P22** appliqué
  (`vat-rates/+page.svelte:228-229` intégré à C, § 24 bis) ; **P23** appliqué (`generator.rs:15` lu : IBAN remplacé dans le texte
  produit ; doc-comment `qr_decode.rs:1231-1232`). Findings P1 à P5, P12 à P17 : traités dans `v0172-session.md` (son Change Log ;
  P16 en partie réfuté, `exports/global.rs:215-217`). Valeurs grepées : `:1032`, `:1040`, `:118`, `MA13`, `A1`, `#876`, `limit: 100`
  dans la fiche ; « 1 à 27 » → « 1 à 28 ». Prochaine : validation **P3** (Sonnet, complète, `v0172-correctifs-validate-prompt-p3.md`).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List
