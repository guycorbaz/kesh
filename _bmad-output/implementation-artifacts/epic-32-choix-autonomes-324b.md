# Epic 32 — registre des choix autonomes de la story 32-4b (couche de services : factures)

À verser par **union** dans `epic-32-choix-autonomes.md` au merge (`docs/agents-orchestration.md` § 1). Fiche :
`implementation-artifacts/32-4b-services-factures.md`. Date : 2026-10-25. Agent de fiche : Opus 5.5, en autonomie.
Relevé au code de `592ee430` (32-4a fusionnée).

---

### C-32-4b-1 — Six lectures migrées ; l'export CSV de l'échéancier et le scan QR restent au handler

- **Contexte** : l'epic nomme « `routes/invoices` (liste, détail, échéances, règlements), `routes/supplier_invoices`
  (liste, détail) » ; ses deux fichiers portent sept handlers de lecture (fiche, fait 1) : les six nommés, plus
  `export_due_dates_csv_handler` (`GET /invoices/due-dates/export.csv`, Comptable+), et `scan_qr_supplier_invoice`,
  qui ne lit pas la base. La 32-9a ne rend aucun fichier CSV. Précédent : la 32-4a a laissé deux lectures de lettrage
  sans outil, lecture que Guy a confirmée le 2026-10-24 (fiche 32-4a, Change Log, « Q1 confirmée »).
- **Retenu** : migrer les six ; laisser l'export et le scan dans leur handler, **nommés** à l'inventaire des sites non
  résolus et figés par la garde (iii) ; l'export emploie la validation **du service** (C-32-4b-4), si bien qu'aucune
  règle de l'échéancier n'existe en deux exemplaires.
- **Écartées** : migrer l'export (un service sans consommateur, une exigence `Comptable` / `None` jamais servie au MCP,
  contraire à C-32-0-4) ; laisser l'export sur sa propre copie de `build_due_dates_query` (deux copies des sept refus).
- **Réversible** : oui ; un outil qui servirait l'export le migrerait — aucune échéance planifiée.
- *(Complété le 2026-10-25, validation P1, F3 = R12.)* **Écart à la lettre de l'AC de l'epic** (« aucun handler de
  lecture des modules visés n'appelle plus un dépôt directement ») : l'export reste sur `state.pool`. La confirmation de
  Guy pour la 32-4a ne couvrait que les deux lectures du lettrage ; l'écart est donc **soumis à Guy** (fiche, Q2) et
  l'epic est annoté d'une ligne datée, comme pour la 32-4a ; le développement part sur cette lecture. **Asymétrie
  assumée** : les règlements (`GET /invoices/{id}/settlements`) sont migrés parce que l'epic les nomme, bien que la
  32-9a ne les nomme pas ; l'export est laissé bien que l'epic nomme « échéances », parce qu'il rend un fichier, non une
  lecture que l'outil servirait. Le critère de C-32-0-4 (« ce que serviront les outils ») cède donc ici à la lettre de
  l'epic dans un sens (les règlements), et la lettre cède au critère dans l'autre (l'export) — écrit pour être
  contesté.

### C-32-4b-2 — Noms des modules : `factures`, `factures_fournisseur`

- **Contexte** : la 32-3 et la 32-4a nomment en français (`rapports`, `plan_comptable`, `ecritures`, `postes_ouverts`,
  `carnet`) ; `routes/invoices.rs` importe `kesh_db::repositories::invoices`, qu'un `services::invoices` heurterait.
- **Retenu** : `services/factures.rs` (factures clientes : liste, fiche, échéancier, règlements) et
  `services/factures_fournisseur.rs` (liste, fiche) — le vocabulaire du manuel utilisateur.
- **Écartées** : un seul module `factures` pour les deux (deux dépôts, deux jeux de types, une confusion de noms
  `lister`) ; un module `echeancier` à part (l'échéancier est une vue des factures clientes, sur le même dépôt et les
  mêmes contrôles de page).
- **Réversible** : oui (renommage interne).

### C-32-4b-3 — `InvoicesRead` pour les six, factures fournisseur comprises ; aucune portée neuve

- **Contexte** : C-32-0-7 range sous `invoices:read` « factures clientes et fournisseur, échéances, balance âgée » ;
  `Portee::InvoicesRead` existe (`services/acces.rs:26-27`) et sert déjà la balance âgée. Mais la liste porte le nom du
  contact et la fiche fournisseur l'IBAN du créancier, alors que `contacts:read` existe « parce qu'ils portent des noms
  et des IBAN » (Q5 de l'epic, sans réponse).
- **Retenu** : `InvoicesRead` pour les six exigences ; le doc-comment de la variante gagne « clientes et fournisseur,
  échéancier, règlements ». La question des champs de contact est posée à Guy (fiche, Q1) ; elle se traite à l'outil
  de la 32-9a ou par une constante d'exigence.
- **Écartées** : `ContactsRead` en plus pour la fiche fournisseur (une `Exigence` ne porte qu'**une** portée,
  `acces.rs:56` ; deux portées changeraient le motif pour un point que Guy n'a pas tranché) ; une portée
  `SupplierInvoicesRead` (absente de C-32-0-7).
- **Réversible** : oui (une constante par service).

### C-32-4b-4 — La validation passe au service, écrite une fois ; une fonction pure partagée avec l'export

- **Contexte** : `list_invoices` et `build_due_dates_query` portent les mêmes contrôles de `limit`, d'`offset` et de
  `search`, recopiés (fait 4) ; l'export CSV appelle `build_due_dates_query` ; le MCP n'a pas le DTO d'Axum.
- **Retenu** : les contrôles communs deviennent des fonctions **privées** de `services/factures.rs`, appelées par la
  liste et par l'échéancier **dans l'ordre de chacun** (la liste : `limit`, `offset`, `search`, `status`, dates ;
  l'échéancier : `limit`, `offset`, plafond, `search`, dates, `dueBefore` ×2) ; `requete_echeances` est `pub`, **pure**
  (aucun accès ni lecture), appelée par `echeances` après `exiger` et par l'export après sa lecture de société ; la
  conversion `ListDueDatesQuery → DemandeEcheances` est **une** fonction du handler, partagée par la liste et l'export.
  Constantes `pub` du service (`LIMITE_DEFAUT`, `LIMITE_MAX`, `RECHERCHE_MAX`, `OFFSET_MAX_ECHEANCES`), qui
  remplacent `DEFAULT_LIST_LIMIT`, `MAX_LIST_LIMIT`, `MAX_SEARCH_LEN` et le `MAX_OFFSET` local.
- **Écartées** : déplacer les deux copies telles quelles (le DRY du dépôt) ; laisser la validation au handler (le MCP
  la contournerait) ; faire de `requete_echeances` un service à exigence (l'export a son RBAC et ne passe pas par le
  canal MCP).
- **Réversible** : oui.

### C-32-4b-5 — La fiche fournisseur : `from_view` pure, `load_with_settlement_cancellation` inchangée

- **Contexte** : `load_with_settlement_cancellation(pool, company_id, id)` (`supplier_invoices.rs:204-226`) lit
  `get_settlement_view` puis met en forme ; elle sert le handler migré, trois mutants et trois sites de
  `routes/imported_supplier_invoices.rs`.
- **Retenu** : la mise en forme devient `SupplierInvoiceResponse::from_view(vue)` (pure, `pub`) ; la méthode garde sa
  signature et se réduit à `get_settlement_view` + `from_view` ; le handler migré reçoit la vue du service et appelle
  `from_view`. Une mise en forme, deux chemins ; `imported_supplier_invoices.rs` non touché.
- **Écartées** : faire rendre `SupplierInvoiceResponse` au service (un type de présentation dans `services/`, R6) ;
  faire appeler le service par les mutants (une lecture sur `state.lecture()` après une écriture, que la 32-6 pourrait
  rendre distincte du pool d'écriture).
- **Réversible** : oui.

### C-32-4b-6 — Les listes clientes refusent hors bornes, comme aujourd'hui ; le fournisseur écrête ; aucun plafond ne change

- **Contexte** : la liste et l'échéancier **refusent** (400) une taille hors de `1..=100` et un `offset` négatif ; les
  factures fournisseur **écrêtent** (`clamp(1, 200)`) — deux régimes, aucun dépôt n'écrêtant (fait 3). La spike veut des
  réponses MCP « sous 10 000 jetons » (l. 961).
- **Retenu** : chaque service garde le régime et les valeurs de sa route ; le MCP reçoit les mêmes refus. Les bornes
  « sous 10 000 jetons » sont celles de **l'outil** (32-9a), sous les plafonds du service — jamais un plafond d'écran
  abaissé par le service.
- **Écartées** : écrêter au service pour le MCP et refuser au handler (deux régimes pour une même lecture, et un refus
  de l'API qui se déplacerait au handler, hors du service) ; aligner les régimes (changement de comportement de l'API).
- **Réversible** : oui.

### C-32-4b-7 — Les demandes portent les types du dépôt pour les filtres, de `kesh-core` pour le sens ; les DTO ne changent pas

- **Contexte** : `PausedParam` et `PaymentStatusParam` sont des enums de route (kebab-case serde), convertis vers
  `PausedFilter` et `PaymentStatusFilter` du dépôt ; `InvoiceSortBy` est celui du dépôt ; `SortDirection` vient de
  `kesh-core` ; les dates sont des `NaiveDate` analysées par l'extracteur `Query`. La 32-4a a pris le type de
  `kesh-core` pour le journal (C-32-4a-15), faute d'autre ; ici il n'existe pas d'équivalent `kesh-core` des filtres.
- **Retenu** : `DemandeListeFactures` et `DemandeEcheances` portent `Option<PausedFilter>`, `Option<PaymentStatusFilter>`,
  `Option<InvoiceSortBy>`, `Option<SortDirection>`, des `Option<NaiveDate>` ; le handler convertit par les `From`
  existants. Les DTO gardent leur forme : une date ou une énumération invalide reste un refus de l'extracteur, avant le
  service ; l'outil MCP analysera vers les mêmes types.
- **Écartées** : faire dépendre `services/` des enums de route (inverse de la couche) ; des chaînes brutes au service
  (un changement de l'ordre des refus : l'analyse passerait après `exiger` et la société).
- **Réversible** : oui.

### C-32-4b-8 — Les ordres de lecture d'aujourd'hui sont conservés, deux lectures sans société comprises

- **Contexte** : `list_invoices`, les règlements et les deux lectures fournisseur lisent la société d'abord ;
  `get_invoice` et l'échéancier n'en lisent **aucune** (fait 5) ; le document des motifs écrit « Every handler
  validates company exists ».
- **Retenu** : ordres conservés tels quels (un jeton orphelin rend aujourd'hui un 404 sur la fiche et une page vide sur
  l'échéancier, non un 500) ; figés par l'AC12 (c) et (d) et les mutations M6 à M8 ; la note de
  `MULTI-TENANT-SCOPING-PATTERNS.md` nomme les deux exceptions.
- **Écartées** : ajouter le contrôle de société (changement de comportement, hors d'un rollout mécanique) ; le taire.
- **Réversible** : oui ; si l'on veut le contrôle, une issue le demandera.

### C-32-4b-9 — Le trou de l'aide appelée par `&state` : le service lit le réglage d'arrondi ; une garde (ix) ; les aides inventoriées

- **Contexte** : `get_invoice` passe `&state` à `with_draft_rounding_preview`, qui lit `state.pool` (fait 6) ; le
  détecteur (ii) de la 32-4a ne cherche que `state.pool` dans le corps et ne voit pas cet appel ; le compte au niveau du
  fichier de (iii) suppose qu'aucune occurrence n'est hors d'un corps, faux pour `invoices.rs` (deux aides des mutants).
- **Retenu** : (1) `factures::detail` lit le réglage pour un brouillon et le rend en `Option<bool>` ; le handler applique
  `with_rounding_preview` ; `with_draft_rounding_preview` reste, inchangée, aux trois mutants. (2) Garde **(ix)** :
  dans les douze handlers migrés, `State(state)` présent et **chaque** jeton `state` sous une forme permise
  (`State(state)`, `state.lecture()`, `state.trace()`, `state.i18n`) — ce qui ferme, pour eux, l'aide appelée par
  `&state`, la liaison renommée et la déstructuration (#738 B4, pour ces formes et ces handlers). (3) `SITES_NON_RESOLUS`
  gagne un champ « aides hors corps » par nom, et l'égalité du fichier compte handlers **et** aides.
- **Écartées** : changer la signature de l'aide en `&MySqlPool` (touche trois mutants pour une garde) ; un compte
  d'occurrences hors corps sans nom (une aide neuve en remplacerait une autre sans rien rougir) ; faire lire le réglage
  toujours, sans condition (une requête de plus sur chaque fiche validée, pour éviter une condition) ; laisser le trou
  aux limites (la story même le rencontre).
- **Réversible** : oui.

### C-32-4b-10 — Toutes les recopies sont testées par la route et toutes leurs mutations jouées ; fixtures par les dépôts et la route

- **Contexte** : la revue de la 32-4a a trouvé trois fois le même défaut : une recopie (`sort_by`, `sort_dir`,
  `offset`) que la fiche disait couverte et qu'aucune ligne n'exerçait ; son échantillonnage des mutations du handler
  (C-32-4a-20) reposait sur une table qu'on croyait complète.
- **Retenu** : une ligne de test par la route **par champ recopié** et par sens de tri, et les **44** mutations de
  recopie (M15 à M19) **jouées**, aucune échantillonnée ; le pilote ne relinke que `services_rollout` quand la mutation
  le permet. Fixtures synthétiques posées dans chaque test : factures par `invoices::create` et
  `invoices::validate_invoice` (comme `invoice_echeancier_e2e.rs:162-194`), règlements par la route
  (`internal_account` sur 1000), factures fournisseur au patron de `supplier_settlement_cancel_e2e.rs:109-177` —
  **non** par l'aide `seed_base` d'`invoice_echeancier_e2e.rs`, dont trois appelants inversent le couple (corrigé par
  l'AC18, C-32-4b-17). *(Précisé à la validation P1, F4 : la matrice reçoit ses fixtures de factures par `poser_livres`,
  par les dépôts — C-32-4b-15 ; les tests de l'AC11 (d) à (g) posent les leurs.)*
- **Écartées** : échantillonner (le défaut répété de la 32-4a) ; des tests au service seulement (ils ne voient pas la
  recopie du handler).
- **Réversible** : oui.

### C-32-4b-11 — Les règlements gardent leur transaction et son `commit` ; une garde lexicale neuve la fige

- **Contexte** : `list_invoice_settlements_handler` lit la liste et chaque motif dans **une** transaction, refermée par
  `commit` (fait 5 ; « un seul instantané », revue P1 de la 25-3-a-2) ; aucune garde ne fige cette forme, et le
  déplacement est l'occasion de la perdre.
- **Retenu** : `factures::reglements` garde la transaction et le `commit` (un `commit` de transaction en lecture seule
  ne fait rien d'observable ; le changer en `rollback` serait un écart sans motif) ; une garde
  `reglements_lisent_dans_une_transaction` au `mod tests` du service, au patron de `get_journal_entry_reads_in_one_transaction`
  (32-4a, C-32-4a-7) ; mutation M23.
- **Écartées** : ne rien figer (une mutation qui lirait hors de la transaction survivrait à tout test) ; passer au
  `rollback` pour s'aligner sur le détail des écritures (comportement modifié sans gain).
- **Réversible** : oui.

### C-32-4b-12 — Q11 tranchée : les doc-comments de `services/mod.rs` et de `poignees.rs` le disent

- **Contexte** : Guy a répondu à Q11 le 2026-10-24 au soir (`epic-32-spike-mcp.md` l. 1061-1067) : lecture seule tenue
  par l'application, mode de session `READ ONLY` en complément, aucun second utilisateur MariaDB, « par construction »
  retiré. `services/mod.rs:132-139` et `poignees.rs:9-12` disent encore la décision à venir, entre trois formes, et « la
  garantie réelle […] celle de la base ». La story modifie `services/mod.rs`.
- **Retenu** : les deux paragraphes disent la décision prise, sans préjuger de la fabrication (32-6) : la `Lecture`
  est la seule poignée que reçoivent les outils ; la connexion du MCP se déclare `READ ONLY` ; la garantie repose sur
  le code de Kesh et ses tests ; aucun utilisateur MariaDB limité. `poignees.rs` (même module `services`) est touché
  pour son seul doc-comment.
- **Écartées** : laisser la correction à la 32-6 (le texte que lit le développeur de la 32-6 serait faux jusque-là, sur
  un fichier que cette story réécrit) ; décrire la fabrication (32-6).
- **Réversible** : oui.
- *(Étendu le 2026-10-25, validation P1, R4 : la correction porte sur **tous** les sites du texte antérieur à Q11, non
  sur deux — C-32-4b-16.)*

### C-32-4b-13 — La matrice de `services_rollout.rs` s'étend ; un test renommé

- **Contexte** : `services_rollout.rs` porte une matrice `Svc` / `SERVICES` / `jouer` à cinq services, que ses tests
  AC9 parcourent ; la 32-4b doit le compléter sans le recréer (C-32-4a-9) ; un test s'appelle
  `ac11_a_les_cinq_services_lisent_en_lecture_seule`.
- **Retenu** : six variantes ajoutées à la matrice (une seule matrice, onze services) ; les bras et lignes de la 32-4a
  restent identiques ; le test est renommé `ac11_a_les_services_lisent_en_lecture_seule` (un nom qui dirait « cinq »
  mentirait), son ancien nom écrit au Dev Agent Record pour la traçabilité des journaux de la 32-4a.
- **Écartées** : une seconde matrice pour la 32-4b (les boucles de refus en double) ; garder le nom.
- **Réversible** : oui.

### C-32-4b-14 — Ni CHANGELOG, ni manuel, ni `docs/api-external.md` : rien de visible ne change

- **Contexte** : comme la 32-3 (C-32-3-41) et la 32-4a (C-32-4a-12), aucun statut, corps, borne ni ordre ne change.
- **Retenu** : contrôle exigé (AC17) — `api-external.md` l. 215, 552-566, 723-757, les manuels et leurs PDF —, aucune
  modification ; la note du document des motifs, les doc-comments de `services/` et des deux fichiers de routes.
- **Réversible** : oui.

---

## Choix de la remédiation de la validation P1 (2026-10-25, Opus 5.5, en autonomie)

### C-32-4b-15 — Deux montages de fixtures : `poser_livres` étendu pour la matrice, fixtures propres aux tests de l'AC11

*(Validation P1, F4 et F5.)*

- **Contexte** : `jouer(svc, acces, lecture, livres, cas)` reçoit `&Livres` (`services_rollout.rs:409-452`) ; les six
  services neufs ont besoin d'une facture, d'une facture réglée et d'une facture fournisseur. Les huit tests de la
  32-4a qui appellent `poser_livres` (l. 486, 546, 639, 683, 863, 941, 995, 1048) comptent des lignes de façon
  relative (`total >= 1`, total sans filtre, `total == 0` pour la seconde société).
- **Retenu** : `Livres` gagne les identifiants d'un brouillon, d'une facture validée **partiellement** réglée et d'une
  facture fournisseur, posés **par les dépôts** dans `poser_livres` (`invoices::create`, `validate_invoice`,
  `invoice_settlements_write::settle_invoice` en `InternalAccount`, `supplier_invoices::create`) ; la signature de
  `jouer` ne change pas ; l'étape 0 relit chaque assertion des huit tests et confirme qu'elle reste vraie avec ces
  écritures et ces contacts de plus. Les tests de l'AC11 (d) à (g) posent leurs propres fixtures : **au moins quatre**
  factures, `created_at` posé par `UPDATE` explicite (le départage `i.id DESC` et `DATETIME(3)` rendraient `CreatedAt`
  ambigu), échéances en 2021 et en 2030.
- **Écartées** : une seconde structure passée en paramètre de plus à `jouer` (toutes ses lignes d'appel modifiées) ; les
  fixtures de la matrice par la route (un `TestApp` dans chaque test de refus, sans gain).
- **Réversible** : oui.

### C-32-4b-16 — Tous les sites du texte antérieur à Q11 ; `lib.rs` et `tests/common/lecture_seule.rs` entrent au périmètre

*(Validation P1, R4 ; décision de l'orchestrateur, pour que deux stories ne corrigent pas les mêmes lignes.)*

- **Contexte** : le relevé de la valeur sur `crates/` entier (`grep -rnE 'limité au .SELECT.|utilisateur .SELECT.|l.imposera
  par|sera celle de la base|par Guy \(Q11\)|Guy sur Q11|la 32-6 \(Q11\)' crates/`) rend six sites, neuf lignes :
  `services/mod.rs:52` (« la 32-6 l'imposera par la base »), `:135-136`, `:139` ; `services/poignees.rs:10-11` ;
  `lib.rs:148` et `:159` (doc-comments d'`AppState::lecture()` et `trace()`) ; `tests/common/lecture_seule.rs:9`. Les
  autres « par la base » et « ou les deux » de `crates/` (sauvegarde, interblocage, lettrage) sont sans rapport, relus.
- **Retenu** : la 32-4b les corrige **tous** (AC17 (3)), sans employer les chaînes du contrôle (« aucun second compte
  MariaDB ») ; le contrôle porte sur `crates/` entier et doit rendre rien. `lib.rs` (deux doc-comments seuls) compte
  comme un module de plus : **4**, sous le seuil de 5 ; `tests/common/lecture_seule.rs` entre à la liste fermée de
  l'AC9. **La fiche 32-5** (`kesh-32-5`, `32-5-cles-et-reglages-mcp.md`, AC de la partie **b** l. 464-467 :
  `poignees.rs:9-12` et `services/mod.rs:93-95` ; partie **c** l. 506-507 : `lib.rs:147-148` et `:158-159`) prévoyait les
  mêmes corrections — ses « 93-95 » de `services/mod.rs` sont, sur sa base `89d68704`, les lignes 94-98 du paragraphe
  aujourd'hui aux l. 132-139 : **elle les trouvera faites à son étape 0**, et l'orchestrateur le lui signale.
- **Écartées** : ne corriger que les deux sites de `services/` (R4 : quatre autres sites diraient encore la décision à
  prendre) ; laisser `lib.rs` à la 32-5c (deux stories sur les mêmes lignes) ; tracer `lecture_seule.rs:9` au lieu de
  le corriger (un fichier de test hors de la liste fermée — l'orchestrateur l'y fait entrer).
- **Réversible** : oui.

### C-32-4b-17 — Le défaut de `seed_base` corrigé dans la story, non ouvert en issue

*(Décision de l'orchestrateur, validation P1.)*

- **Contexte** : `invoice_echeancier_e2e.rs:112-117`, `seed_base` rend `(seeded.admin_user_id, seeded.company_id)` ; les
  l. 402, 453, 500 le lient en `(company_id, admin_id)`, la l. 538 en `(_company_id, _admin_id)` (relevé
  `grep -n 'seed_base(&pool)'` : 32 appels, ces quatre inversés). Verts par coïncidence d'identifiants.
- **Retenu** : AC18 — les quatre liaisons rétablies, aucune autre ligne ; le fichier entre à la liste fermée de l'AC9 ;
  une ligne au Change Log. Pas de mutation (correction d'un test).
- **Écartées** : une issue P3 (proposée à la rédaction ; le budget d'issues et la taille du correctif plaident pour
  l'intégration) ; corriger aussi `seed_base` pour rendre `(company_id, admin_id)` (touche les 28 autres appels).
- **Réversible** : oui.
